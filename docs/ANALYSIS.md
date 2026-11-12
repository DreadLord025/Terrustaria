# Source analysis — Terraria 1.4.5.8

All figures below come from decompiling the retail `Terraria.exe` (v1.4.5.8, 25.4 MB) with
ILSpy into `decompiled/` (1 549 `.cs` files) and `ReLogic.dll` into `decompiled-libs/ReLogic`
(167 files).

## Size

| Scope | Files | Lines |
| --- | --- | --- |
| `Terraria.exe` (decompiled) | 1 549 | **755 403** |
| `ReLogic.dll` (decompiled) | 167 | ~40 000 (excluding generated resources) |

Largest namespaces:

| Namespace | Lines |
| --- | --- |
| `Terraria` | 506 523 |
| `Terraria.ID` | 30 536 |
| `Terraria.GameContent` | 17 520 |
| `Terraria.GameContent.Drawing` | 15 419 |
| `Terraria.GameContent.UI.States` | 14 724 |
| `Terraria.DataStructures` | 12 436 |
| `Terraria.UI` | 11 260 |
| `Terraria.GameContent.UI.Elements` | 8 245 |
| `Terraria.Initializers` | 6 253 |
| `Terraria.GameContent.Generation.*` | 5 813 |
| `Terraria.IO` | 5 331 |
| `Terraria.Graphics.Light` | 5 283 |
| `Terraria.ObjectData` | 5 183 |

Largest single files (this is where most of the work lives):

| File | Lines | Role |
| --- | --- | --- |
| `Terraria/NPC.cs` | 95 930 | all NPC AI, spawning, shops, dialogue hooks |
| `Terraria/WorldGen.cs` | 87 372 | world generation, tile framing, growth, hardmode spread |
| `Terraria/Projectile.cs` | 81 034 | every projectile behaviour + drawing |
| `Terraria/Main.cs` | 65 984 | game loop, drawing orchestration, globals |
| `Terraria/Player.cs` | 55 641 | player physics, buffs, equipment, drawing state |
| `Terraria/Item.cs` | 49 694 | item behaviour, use logic, prefixes |
| `Terraria/Recipe.cs` | 16 731 | crafting recipes and query engine |
| `Terraria.GameContent.Drawing/TileDrawing.cs` | 9 685 | tile draw batching, framing, paint, wind |
| `Terraria.GameContent.Drawing/PlayerDrawLayers.cs` | 4 348 | every player equipment draw layer |
| `Terraria.ObjectData/TileObjectData.cs` | 5 183 | multi-tile object placement rules |
| `Terraria/DataStructures/PlayerDrawSet.cs` | ~4 000 | per-frame draw state |

Content id tables (generated, but they define the data model):

| Table | Entries |
| --- | --- |
| `ItemID` | 6 244 |
| `ProjectileID` | 1 137 |
| `NPCID` | 782 |
| `TileID` | 754 |
| `BuffID` | 400 |
| `WallID` | 367 |
| `PrefixID` | 97 |

## Systems inventory

Status legend: ✅ implemented in some form, 🟡 stub/partial, ⬜ not started.

| System | Original location | Size | Status | Notes |
| --- | --- | --- | --- | --- |
| World model & save format | `IO/WorldFile.cs`, `Tile.cs` | ~2 600 | ✅ | `.wld` read + written for all 11 sections (header/flags/tiles + chests/signs/NPCs/TEs/plates/TownManager/Bestiary/CreativePowers, every load fix-up ported), the app's load/save dispatch wired. Native `.trwld` writer/reader for generated worlds (no TE/chest sections — the M6-SAVE-1b follow-up). `Tile` is 12 bytes (slope/paint and wall cell packed); 5.04 M-tile world = 60.5 MB |
| Tile framing / autotiling | `WorldGen.TileFrame` (line 82852), `TileFrameCosmetic` | ~3 600 | 🟡 | Generic block chain + merge families ported; slopes/half-bricks framed; per-type grass edge/moss/vines/weird branches approximated |
| Wall framing | `Framing.WallFrame` (line 326) | 120 | 🟡 | Full `wallFrameLookup` + `centerWallFrameLookup` port; variant is a position hash, `TruncatesWalls`/invisible-wall neighbour rules and persisted `wallFrameNumber` are not modelled |
| Tile drawing | `GameContent.Drawing/TileDrawing.cs`, `WallDrawing.cs` | 9 700 + 1 300 | 🟡 | Quads, 9-sliced light, slopes, half bricks, tree canopies/branches, tile-3 plants; walls are 32x32 sprites at `-8` with corner lighting, drawn in their own pass; no paint, wind, glow, tile entities |
| Lighting | `Graphics/Light/LightingEngine.cs`, `LightMap`, `TileLightScanner` | ~5 300 | ✅ | Scanner + blur + sunlight; 4 modes, `GlobalBrightness`, 9/4-slice block smoothing; sun/moon/flicker still missing |
| Sky / backgrounds | `Main.cs` sky code, `Backgrounds/*` | ~3 000 | 🟡 | `Background_0` gradient + two parallax mountain layers (`Background_7`), sun/moon on `DrawSunAndMoon`'s arc (scale/phase/rotation), the `Cloud.addCloud` cloud field with `GetParallax` and the three `DrawSurfaceBG` passes, `Star.SpawnStars`/`DrawStar` stars, the textured `Background_1/2` underground backdrop anchored to `worldSurface` and the rock/magma/hell passes (`Background_3/4/6/125..127/185..187`, underworld black box) in `DrawBackground` order; biome backgrounds/rain still partial |
| Liquids | `Liquid.cs`, `LiquidRenderer.cs` | ~3 000 | 🟡 | Water/lava/honey/shimmer drawn from `Misc/water_*` with animation, waves, opacity, depth tint and lighting; `Liquid.Update` flow simulation ported (active cells, 30 Hz, leveling); no `Waterfall_*` overlay or corner-height shoreline mesh |
| Player physics | `Player.cs` | 55 600 | 🟡 | Gravity, acceleration/friction, hold jump, AABB collision, ground-slope support, water buoyancy, composite arms, HP/hurt/respawn, classic HUD; no half-brick/ceiling slopes, item use timings or buffs |
| Player rendering | `PlayerDrawLayers.cs` (+ 40 layer files) | ~12 000 | 🟡 | Starter head/eyes/torso/shirt/arm/legs/pants/shoes/hair from the original strips; no armor/items/equipment layers |
| Items & inventory | `Item.cs`, `UI/ItemSlot.cs`, `Recipe.cs` | ~70 000 | 🟡 | 4-slot hotbar, mining/placing, counts and native save; no item entities, tools, recipes, chests or full inventory UI |
| NPC AI | `NPC.cs` + `NPCs/*` | 95 900 | 🟡 | First pass: hopping blue slime (gravity, collision, contact damage, player melee, HP bars); ~700 NPC types remain |
| Projectiles | `Projectile.cs` | 81 000 | ⬜ | ~1 100 types |
| Combat / damage | spread across `Player`, `NPC`, `Projectile`, `Item` | ~15 000 | 🟡 | Player/slime HP, hurt cooldown, knockback, contact/melee damage and respawn; no defense, crits, projectiles or drops |
| World generation | `WorldGen.cs`, `Generation/*` | ~45 000 | 🟡 | Native compact generator: surface/layers/caves/ores/ash/trees/plants, forest lake, snow biome, ocean beaches; no corruption/crimson/jungle/desert/hallow or structures |
| UI / menus | `UI/*`, `GameContent.UI.*` | ~35 000 | ⬜ | |
| Audio | `Audio/*`, XACT banks | ~4 000 | 🟡 | Runtime `SoundEffectReader`, eight PCM cues through `rodio`, music selector; XACT/WMA playback pending |
| Networking | `Netplay.cs`, `MessageBuffer.cs` | ~12 000 | 🟡 | Custom host-authoritative TCP host/join with players, edits, NPCs, drops and time; not vanilla-protocol compatible |
| Map / minimap | `Map/*` | ~5 000 | 🟡 | Per-tile `MapTile` colours from `MapHelper.Initialize`, minimap + fullscreen map with zoom/WASD pan, biome `MapBG` backdrop, player head icon and `.map` v326 persistence. No cloud/team-icon map extras |

## World file format (`.wld`)

Reference: `Terraria.IO.WorldFile.LoadWorld_Version2` (line 1784) and `LoadWorldTiles`
(line 2543); v2 sectioned format, supported here for version ≥ 87 (current saves are 279+).

```
i32 version
[20 bytes FileMetadata]                (version >= 135)
i16 sectionCount
i32 sectionPointers[sectionCount]      header, tiles, chests, signs, NPCs, ...
u16 tileTypeCount
bit-packed bool[importance]            which tile types store explicit frames
--- section 0: header ---
string name
string seed, u64 worldGenVersion      (version >= 179)
16 bytes GUID                          (version >= 181)
i32 worldId, left, right, top, bottom
i32 maxTilesY      <-- note: Y comes first
i32 maxTilesX
... world flags
--- section 1: tiles ---
for x in 0..maxTilesX:                 column-major, RLE per column
  b4 flags: active, wall, liquid type, RLE length bits
  optional b1/b2/b3 bitfields (frame importance, paint, actuator, slope...)
  tile type (u8 or u16), frameX/frameY (i16) only for important types
  wall id, liquid amount, RLE repeat count
```

Tile flags we map 1:1 into a packed 16-byte struct (`tr-core/src/tile.rs`): active,
half-brick, actuator, inactive, fullbright, liquid kind, liquid amount, slope, paint, wall,
plus the computed wall autotile cell (`wall_col`, `wall_row`).

### The eleven sections (all eleven ported, both directions — M6-SAVE-1, 2026-11-09)

`SaveWorld_Version2:1150-1165` / `LoadWorld_Version2:1760-1851`: a 70-byte prologue
(`i32 326` + the 20-byte `FileMetadata` + `i16 11` + the eleven `i32` section **end**
offsets), then the sections in this order, then the footer triple
`(true, worldName, worldId)`. `wld/mod.rs` assembles/backfills the table;
`wld/sections/<name>.rs` is each codec; the reader's end-offset assertions are
reproduced (`bad section pointer N`).

| # | Section | Wire shape | Rust |
| --- | --- | --- | --- |
| 0 | file-format header | `u16 TileID.Count` + the 95-byte importance bitmap | `sections/header.rs` |
| 1 | world header + flags | name, seed text + generator version, GUID, world id + bounds, `maxTilesY` before `maxTilesX`, then the ~2200-byte flags block in `LoadWorldFlags` order | `sections/header.rs` + `flags.rs` |
| 2 | tiles | column-major RLE (see above) | `sections/tiles.rs` |
| 3 | chests | `i16 count` (+ the shared `i16` slot count below 294), then per chest `(i32 x, i32 y, string name, i32 maxItems, maxItems × i16/i16/i32/u8)`; the duplicate-position and tail passes, the `<115` dresser pass, `Chest.FixLoadedData`, and the save-side dead-anchor pre-pass | `sections/chests.rs` + `tr-app/chest.rs::{from_wld,to_wld,fix_dresser_chests,destroy_invalid_anchors}` |
| 4 | signs | `i16 count`, then `(string text, i32 x, i32 y)` per sign over `Main.sign[0..31999]`; the tile check + dup + tail passes on load | `sections/signs.rs` (the store lives on `World.signs`) |
| 5 | NPCs | the shimmer bitmap's truth indices (`>= 268`), the terminator-delimited town list (`i32 netID` or the `<190` name, `string GivenName`, `Vector2`, `bool homeless`, the home pair, a `BitsByte` gating `townNpcVariationIndex` `>= 213` and `homelessDespawn` `>= 315`), then the four-tower list (`>= 140`) | `sections/npcs.rs` + `tr-app/npc.rs::{from_wld,to_wld}` |
| 6 | tile entities | `i32 count`, then per entity `(u8 ty, i16 x, i16 y)` + the type's `ReadExtraData`; the id reassignment, the `InWorld`/`CheckValidTile` drop pass, `OnWorldLoaded`, and the display doll's payload skip-reads | `sections/tile_entities.rs` |
| 7 | pressure plates | `i32 count`, then `(i32 x, i32 y)` per pressed point (the `bool[255]` weights are runtime state) | `sections/pressure_plates.rs` (`World.pressure_plates`) |
| 8 | TownManager rooms | `i32 count`, then `(i32 npcID, i32 x, i32 y)` pairs | `sections/town_rooms.rs` + `tr-app/town.rs::TownRooms` |
| 9 | Bestiary | three sub-lists: kills (`i32 count` + `(string creditId, i32 count)`), sights and chats (`i32 count` + one credit id each) | `sections/bestiary.rs` + `tr-core/bestiary.rs::WorldBestiary` (`App.bestiary`) |
| 10 | CreativePowers | **terminator-delimited**: `{ bool true; ushort id; payload }` repeated until `bool false`; toggles write `bool Enabled`, the two sliders write the `[0,1]` `_sliderCurrentValueCache` | `sections/creative_powers.rs` + `tr-app/creative.rs` |

The C#'s own section numbering is 0-based in `positions[]` and 1-based in the prose;
`WorldFile.SaveBestiary:3396` is the **1-based 10th** section (pointer 9), and
`SaveCreativePowers:3412` is the 11th (pointer 10).

The app glue (M6-SAVE-1 phase 7): `app/mod.rs::load_world` calls
`wld::load_with_sections` (one read: world + sections) and applies the world-state
halves in the C#'s order — signs, tile entities, pressure plates — before the framing
passes; `App::new` applies the App-owned halves — chests (the file's store displaces the
load-time re-roll, the `<115` dresser fix-up gated on the file version), the NPC list
(replacing the startup spawns, then `FixAgainstExploits:1861`'s home re-validation with
the town rooms), the shimmer bitmap, the bestiary tracker and the creative powers.
`saveglue::save_world` dispatches on the extension: `.trwld` → the native writer plus the
sidecars (unchanged), `.wld` → `wld::save` with every section built from the live state
(the chests' dead-anchor pre-pass first, as the C# runs it). The autosave, F9, close and
quit paths all funnel through `save_world`.

### Deviations (every deliberate departure from the C#, phases 2-7)

- **Unowned flag fields** are written with the C# defaults derived from
  `LoadWorldFlags:1981`'s `< gate` branches rather than being modelled (assumption-1's
  fallback); the reader keeps its own width.
- **`LastPlayed` is 0** and the creation-time pair is zero-filled — the port has no
  play-time accounting; `FileMetadata`'s revision increments per save, as vanilla
  (`IncrementAndWrite`), and is the file's only clock-adjacent value.
- **Wires are not written**: no section carries the wiring state.
- **GUID and worldId are 0** — the port keeps neither (the footer's worldId likewise).
- **Seed text**: the port's `u64` seed is persisted as its decimal text; a non-numeric
  seed reads as 0 (`SeedText` is `WorldFileData.cs:355`'s own convention).
- **Non-important tiles and kind 144**: a non-important kind reads back with
  `frameX/frameY = -1` (`LoadWorldTiles:2605`) and kind 144's `frameY` is forced to 0 on
  write (`SaveWorldTiles`) — an asymmetry preserved verbatim.
- **Chests**: a banned item's `(type, prefix, stack)` triple is not written at all
  (unreachable through the app — the fix-up turns it to air first); the latent
  `> 8000`-chest count asymmetry between the writer's `i16` count and the displacement
  pass; a non-40 `maxItems` chest is not threaded into the click hit-test.
- **NPCs**: a `SavesAndLoads` tower record (422/507/493/517) is parsed, its bytes
  consumed and the entity discarded (no ported kind is a tower); the `version < 251`
  recompute of the eight `unlocked*Spawn` flags is not ported (live spawning state); a
  homeless record's stale home coordinates are not carried into live state.
- **Tile entities**: the dummy's and the logic sensor's payloads are carried from the
  file even though nothing reads their runtime halves; the display doll's payload is
  read and **dropped** (`is_storable`) because its `WriteExtraData` shape (three
  bitsBytes, a pose byte, up to nineteen triples) has no home in `TeData` and a hollow
  record would desynchronise a vanilla reader — so a `.wld` round-trip is not byte-exact
  for a world with dolls; the kite/critter anchors' `IsTileValidForEntity` is the anchor
  kind alone (`TEKiteAnchor.cs`/`TECritterAnchor.cs`), which the shared head check
  settles.
- **Signs and plates**: the store is the positional `Vec<WldSign>` (exactly the non-null
  signs in slot order), not the C#'s 32 000-slot array; a duplicate plate point is
  refused rather than overwritten (`LoadWeightedPressurePlates`' dictionary key).
- **Collections**: the C#'s `Dictionary`/`HashSet` enumeration order is unspecified; the
  port stores sorted maps/sets, so a save is deterministic and differs from a vanilla
  file only in order, never in content.
- **Bestiary**: the sightings/chats stay empty until the recording behaviour lands
  (owner decision, state + IO only); the kill clamp is one-sided (the file keeps the raw
  value, the in-memory tracker clamps it).
- **Creative powers**: the id-8 slider is a `[0,1]` cache on the wire and the ported rate
  1..24 in memory — the codec remaps both directions
  (`ModifyTimeRate.UpdateInfoFromSliderValueCache:877-880`); the three ids the port has
  not ported (freeze rain 9, freeze wind 10, difficulty 12) are never written.
- **Save/load asymmetry**: the C#'s `SaveChests:1609-1642` dead-anchor pre-pass mutates
  `Main.chest`; the port does the same to its live store (through `&mut self`), so the
  file and the store agree.
- **A `.wld` load carries no player or map sidecar**: the kills ride the world file
  (owner decision), the `.trplayer`/`.map` sidecars stay the `.trwld` path's — a vanilla
  player file has no port format yet (the follow-up row).
- **`.trwld` TE/chest parity** is not done: the port's own format carries neither, so a
  `.trwld` world loses its tile entities and chests on reload (M6-SAVE-1b).

## Autotiling

`num6` / `Tile.frameNumber` **is not stored in the save file** (the tile header only carries
frame X/Y for important tiles). Both `frameNumber` and the wall variant are generated at
load time in the original and always from `WorldGen.genRand`; we replace them with a
deterministic per-position hash, which is why two loads of the same world look identical.

Ported pieces (`tr-core/src/framing.rs`):

1. **Block frames** — the 20-entry generic `if/else` chain at `WorldGen.cs:85929–86319`,
   each row selecting one of three 16×16 frames on the 18 px tile grid. Tile-type
   normalisation (`Main.tileStone` → stone, `668` → dirt, `697` → `51`) and slope-side
   clearing match the original.
2. **Merge families** — `TileID.Sets.Dirt`, `Mud`, `Ash`, `ForcedDirtMerging`,
   `NeedsGrassFraming` (+ `NeedsGrassFramingDirt`), `OreMergesWithMud`, moss/moss-brick,
   `BlockMergesWithMergeAllBlock`, plus 2 228 directed pairs extracted mechanically from the
   per-type `switch (num)` cases in `TileFrameCosmetic`.
3. **Wall frames** — the complete `wallFrameLookup` (20 masks × 3 variants),
   `centerWallFrameLookup` and the 36 px cell geometry; comparisons use the wall's own
   visibility rules minus invisible walls.
4. **Large frames** — `Main.tileLargeFrames` 1 (4-row pattern) and 2 (2×2 pattern) override
   the variant instead of using the hash.
5. **Grass family** — the full `TileFrameCosmetic` grass path (`WorldGen.cs:82184-84836`):
   the merge preparation (dirt normalisation, grass↔ore, the Weird merge of non-solid
   neighbours into "open"), the base selection (`NeedsGrassFramingDirt`, mud for jungle
   grass, stone for moss), the five override chains (edges, interior corners, overhangs,
   base-corner mixes and the `(i + j) % 2` mowed checkerboard), the `-2` base marker
   (WorldGen.cs:83380), the `-1`/`-2` merge tables (83419-84242) and the generic chain
   fallback with the base rejoined (84398-84784). Grass no longer merges with its base
   dirt in the generic pass — the transition frames are what make grass blend into dirt
   masses and wrap steps. `tools/GenTileData` additionally emits `Sets.Ore`,
   `Conversion.Grass` and `ChecksForMerge` for the gates.

   Documented quirks and deviations of the grass port:
   - `TileMergeCulling` (WorldGen.cs:4065, applied at 82215/83392/84396) culls neighbours
     whose `invisibleBlock` state differs from the tile's own. The port stores no
     invisible blocks (coatings are a TILE-DRAW-4 item), so the cull is a no-op and is
     not ported.
   - The left-edge override's second branch (WorldGen.cs:82314) repeats the first
     condition verbatim and is unreachable in the original; the port keeps only the live
     branch. The right edge's odd `down == num && right == up` condition (82353) is
     ported verbatim.
   - `WorldGen.mergeUp/Down/Left/Right` are output flags consumed by
     `TileMergeAttemptFrametest` (the cross-type merge tests for snow/ice/sand — not the
     grass family) and by the neighbour re-frame cascade (84795), which the full-world
     re-frame and `reframe_area` already cover; they are not modelled.
   - `TileID.Sets.HasSlopeFrames` (84245) has no grass-family member; the slope-frame
     block is not ported. The large-frame `frameNumber == 3` row shift (84785) cannot
     trigger either — no grass-family tile sets `tileLargeFrames`.
   - The ore side of the grass↔ore merge (WorldGen.cs:82147-82150) still falls into the
     known gap of the merge-relation extractor (top-level `TileMergeAttempt` calls):
     ore tiles next to grass show edge frames where the original merges them.

Not yet ported: the per-type special-case branches (cactus, vines, `TileFrameImportant`
multi-tile objects), `TileID.Sets.TruncatesWalls`, paint and coating. Those are listed in
the roadmap.

`tools/GenTileData` parses `Terraria.ID/TileID.cs`, `Main.cs` and `WorldGen.cs` and emits
the `tiledata/` modules; rerun it after touching those tables.

## Slopes and half bricks

Slope/half-brick tiles are no longer skipped by the framer: the generic 20-rule chain frames
them with the same slope-aware neighbour rules as `TileFrameCosmetic` (a slope hides its two
open sides, and sloped neighbours stop contributing a side). The draw pass ports
`TileDrawing.DrawBasicTile`:

- slopes 1/2 (ground) and 3/4 (ceiling) are eight 2 px columns plus a 2 px strip, each
  sampling the tile's normal frame with the original source/destination offsets;
- half bricks keep the top 8 rows of the art and sit on the bottom 8 px of the cell.

Slope tiles still receive flat per-tile light, as the original's slope path does.

## Trees

Trees are tile id 5 columns (plus the gem/vanity/ash variants). The trunk sheet is not on
the 18 px block grid: `Tiles_5.png` uses **22 px cells with 20×20 sprites**, drawn at
`x*16 - 2`. `TileDrawing.GetTileDrawData` also shifts `frameX` by `176 × (biome + 1)`; the
biome comes from the ground tile under the trunk. Top and branch tiles (`frameX` 22/44/66,
`frameY >= 198`) are transparent markers, not art: `DrawTrees` replaces them with:

- canopy — `Tree_Tops_{style}`, source `treeFrame × 82, 0, 80×80` (jungle 114×96, hallow
  80×140), anchored at `(x*16 + 8 - w/2, y*16 + 16 - h)`;
- branches — `Tree_Branches_{style}`, source `(0 | 42, treeFrame × 42, 40×40)`, offset
  `(-24, -12)` or `(0, -12)`.

`tr-core/src/tree.rs` carries the marker/biome/style tables; the app draws foliage after
the tile pass. Forest world-style randomisation (`TreeTops.GetTreeStyle`, a later save
section we do not parse) is approximated as style 0; gem/vanity/palm trees are not composed.

### The tree kill cascade (BUG-16, 2026-11-11)

`TileFrameImportant` dispatches the tree kinds to `WorldGen.CheckTree` (the forest
tree 5), `CheckTreeWithSettings` (the gem/vanity/ash variants) and `CheckPalmTree`
(323) — `WorldGen.cs:85876-85911`. Those routines kill the tree cells that lost
their support; the `KillTile` chain is what makes a felled tree fall instead of
floating. The port's `tr-core/src/edit.rs::check_tree` / `check_palm_tree` run the
same machine; the ground test is the only parameter (`TreeVariant::ground_type` /
`ground_valid` — the forest tree folds the grass family onto plain dirt, the
variants use `GemTreeGroundTest`/`VanityTreeGroundTest`/`AshTreeGroundTest`).

Two deviations:

- **The palm's canopy re-roll** (`CheckPalmTree`, `:54385-54388`) draws
  `genRand.Next(7, 10) * 22` into the cell's `frameX`. The core has no RNG
  stream, so `TileKill::palm_frame_rolls` records the request and the app
  resolves it after the cascade. The C#'s neighbour ring does not fire for that
  arm (`frameY` is unchanged), so the deferral changes nothing but the moment
  the frame is written.
- **`noItem`.** The C#'s `CheckTree` calls `WorldGen.KillTile(i, j)` with the
  default `noItem: false`, so a tree cell killed by the cascade always drops
  its wood. The port propagates the ambient `noItem` like the other `Check*`
  kills, so the worldgen pass (which suppresses the drops) stays quiet. In
  play `noItem` is false either way.

The drop rolls follow the house RNG seam: one `item_rng` throw per event seeds
the `FastRandom` that stands in for the C#'s `genRand` stream and the
`Main.rand` roll spends the item stream directly. Note that in the C#
`WorldGen.genRand` **is** `Main.rand` (a property, `WorldGen.cs:4445`), so the
C# draws its four tree-drop rolls from one stream and the port draws them from
two — the same approximation the pot and pile loot already use.

## Lighting

Terraria's light is a **tile light map**, not a per-vertex shader effect:
`Terraria.Graphics.Light.LightingEngine` walks the visible area plus a 28-tile border,
`TileLightScanner` writes a seed colour per tile, and `LightMap.Blur` spreads those seeds with
directional sweeps. Everything is drawn multiplied by `Lighting.GetColor(x, y)`; in Color
mode solid blocks are further split into 3×3 (or 2×2) light slices.

Ported in `tr-core/src/lighting/`:

1. **Region** — `AREA_PADDING = 28` around the visible tile rect, `NON_VISIBLE_PADDING = 18`
   rows/columns that the sweeps stop before, exactly as `LightingEngine` does.
2. **Seeds** (`TileLightScanner.GetTileLight`), per tile in this order:
   - `ApplySurfaceLight` — for `y <= column_surface(x)` (the first solid tile of that
     column; the original uses the global `worldSurface`, which leaves hills above the
     average dark). The light colour is `Main.tileColor`
     (`ApplyColorOfTheSkiesToTiles` of the sky colour) and it only passes where
     `Main.wallLight[wall]` (or wall 73/227, or an invisible wall) and the tile does not block
     sun (`Main.tileNoSunLight`, with the slope/half-brick liquid rule), plus the gem-wall
     colour override.
   - `ApplyHellLight` — below `height - 200`: `0.55 + sin(t·2)·0.08` warm glow.
   - `ApplyWallLight` — the `Main.wallLight` table (16 glowing wall types) with per-type
     colours extracted by `tools/GenTileData`, plus the crystal-wall random flicker and the
     disco walls (44, 346).
   - `ApplyTileLight` — the full 147-case switch (see "Per-type tile light" below).
   - `ApplyLiquidLight` — lava/shimmer self-illumination; the original compares every
     channel against the unscaled base and then stores the scaled value.
3. **Decay per tile** (`LightDecayThrough*`): air `0.91`, solid `0.56`, cracked bricks `0.8`,
   water `(0.88, 0.96, 1.015)·0.91` with a per-cell jitter, honey `(0.75, 0.7, 0.6)·0.91`.
   This is why surface dirt is already ~half as bright one tile under the grass — `0.56` per
   solid tile is the game's own value, and it is what makes torches mandatory underground.
4. **Blur** (`LightMap.BlurPass`, run twice): four sweeping lines (top→bottom, bottom→top,
   left→right, right→left), each carrying the brightest colour seen so far and multiplying it
   by the next tile's decay. The original's `0.0185` per-channel cutoff is kept, so dim light
   stops propagating instead of tinting the whole map.
5. **Smooth blocks** (`Lighting.GetColor9Slice` / `GetColor4Slice`,
   `DrawSingleTile_SlicedBlock`): a solid block in Color/White mode is drawn as nine quads
   (4/8/4 px) whose tints average the block's light with the corresponding neighbour; below
   the medium threshold it is four 8×8 quads, and below that one flat quad. A slope or half
   brick is not sliced.
6. **Modes** — `LightMode::{Color, White, Retro, Trippy}` match
   `Terraria.Graphics.Light.LightMode`. `LightStyle` says whether blocks are sliced, whether
   light is grayscale, quantisation steps, time-dependent channel perturbation and the
   shader aberration hook. `GlobalBrightness = 1.2` is applied when shading (`Lighting`'s
   default). Adding a mode (Sepia, Monochrome, ...) is a variant plus a style; the draw pass
   needs no new path.

Verified numerically with `TR_LIGHT_AT`: at midday the stored world reports
`light at 3200,402 = 0.560,0.560,0.560` for the first dirt row under the grass — exactly
`Main.tileColor` (white) × `0.56`; the renderer multiplies that by `GlobalBrightness`.

### Per-type tile light (M1.5-LIGHT-2)

`tr-core/src/tile_light/scanner.rs` is a case-for-case port of `TileLightScanner.ApplyTileLight`
(147 top-level `case` labels in the decompiled source, ~2 800 lines), plus
`ApplyWallLight`, `ApplySurfaceLight` and `ApplyHellLight`. It replaces the earlier
`EMISSIVE`-table approximation, which only covered the 43 types whose colour is a plain
constant and silently missed every conditional arm.

What the port covers:

- **Frame-selected colours** — torches (4), candles (33), lamps (93), chandeliers (34),
  lanterns (42, 100, 173), campfires (215), chinese lanterns (405), living fire (592-594),
  gemspark (429), rainbow bricks (719), and the gem tiles whose colour lives in the parent
  tile's packed `frameY` bits (171).
- **Animated colours** — `Main.DiscoR/G/B` (disco torches, gemspark, painted lamps),
  `Main.demonTorch` (demon torch and its hsl variant), `Main.mouseTextColor` (every flicker
  term), `Main.GlobalTimeWrappedHourly` (rainbow bricks, the 620 pulse),
  `Main.timeForVisualEffects` (350), `Main.time`/`dayTime`/`raining` (83),
  `Main.sundialCooldown`/`moondialCooldown` (356/663) and `Main.shimmerDarken`
  (`ApplySurfaceLight`).
- **Paint** — `flag` in the original means "this lamp's colour came from its frame, so a
  paint coat replaces it". The 93/33/34/100/173 tables set it for their painted arms and
  `739`, `519` and the 70-72/190/348/349/528/578 family set it when painted.
- **Per-tile randomness** — `Terraria.Utilities.FastRandom`: the scanner keeps one LCG
  state, re-seeds it every frame and derives a per-tile stream with `WithModifier(x, y)`.
  `FastRandom` in `tile_light/palette.rs` reproduces the multiplier/mask, `NextBits`,
  `NextFloat` and both `Next` overloads. Our seed is `world.seed ^ frame * golden` instead
  of `Guid.NewGuid()`, so screenshots at a fixed frame are reproducible.
- **Tables** — the five largest arms (93, 33, 34, 100/173, 42: 1 535 lines of C#) are
  extracted mechanically by `tools/gen_tile_light_tables.py`, which also resolves the
  stacked `case 7: case 8:` labels and the four dynamic entries each table contains
  (paint, demon torch, shimmer, `0.5/0.9/1.0 + flag`). Re-run it after a source update.

Deliberate deviations, all invisible in the current content set:

- `tile.invisibleBlock()` / `tile.invisibleWall()` are not stored in the packed 12-byte
  tile; an invisible block/wall is treated as visible (the scanner's `_drawInvisibleWalls`
  off path).
- `tile.wallColor()` is not stored, so a painted glowing wall is treated as unpainted. The
  tile paint (`tile.color()`) *is* stored and drives the lamp override.
- The jellyfish-cage animation state machine (`Main.Update`'s `jellyfishCageMode`) is not
  ticked yet; `LightParams::cage_mode` is zeroed, which selects the dim cage colour.
- `new Color(230,230,230,0).MultiplyRGBA(hsl) * 0.4f` (type 620) is reproduced with the
  XNA byte multiply (`>> 8`) and byte truncation, which can differ from XNA by one LSB.
- The `Main.liquidAlpha[13]` blue boost needs `shimmer_alpha`, which the app still passes
  as 0 (the shimmer biome does not exist yet).
- `Main.time` is `f32` here (the original is `double`); the affected comparisons
  (`time > 40500`, `time < 16200`) are exact at these magnitudes.
- `demonTorch`, `DiscoR/G/B` and `mouseTextColor` advance once per *rendered* frame rather
  than per fixed 60 Hz tick, so at other frame rates their phase differs from the original.

Because the flicker inputs change every frame, the D2/D3 light cache now misses on every
frame (`TR_STATS` reports `reused=0%`). That matches the original, which recomputes the
whole light map per frame, but it gives back the "static view costs 0 ms" property the
M1.5 optimisation pass measured; `TR_NOCACHE=1` no longer changes the outcome.

Verified numerically with `TR_LIGHT_AT`: at midnight a `Torch` (`frameY 0`) tile reads
exactly `[1.000, 0.950, 0.800] = TorchID.TorchColor` (base torches are constant; only ids
7/14/23 animate), and surface air at noon reads `[1,1,1]`.

Remaining lighting gaps:

- Sun/moon/star light and the `Lighting.GetColor` shader path for light *behind* tiles are
  not ported.
- The shader has an aberration hook (`Globals::aberration`), but it is disabled for Trippy
  until instances carry a per-sprite flag: shifting full-screen sky quads samples past
  their atlas sheet.
- The real parallax, cloud and underground background layers are ported (see
  "Surface backgrounds" and "Sky and daylight"): the game's own `Background_0.png`
  gradient anchored to `Main.worldSurface`, the `DrawSurfaceBG` layer stack for the four
  generated biomes with its cross-fade, drifting `Cloud_*` sprites, the `Background_1/2`
  underground backdrop, and sun/moon/stars on `DrawSunAndMoon`'s screen-space arc.
- `TileDrawingBase.DrawOwnBlacks`: an exactly-zero-light cell paints opaque black before
  its sprite/wall. Vanilla emits that quad from `TileDrawing.DrawSingleTile` for every
  active tile and from `WallDrawing.DrawWalls` for every walled cell that is not a
  `FullTile`; the renderer folds both cases into one merged black rectangle per row
  (`light_is_zero && (active || wall != 0)`), which is what stops the parallax/underground
  backdrop bleeding through unlit caves. A fully dark tile is solid black either way, so
  skipping its sprite is pixel-identical (glow masks are not implemented yet). Echo walls
  (`WallID.EchoWall`, 318) are skipped entirely — no sprite and no black — exactly like
  `WallDrawing` with invisible walls hidden.
`Main.time` advances with a 15-minute day and 9-minute night (60 ticks/s); the sky colour
and light are quantised to one-second steps so the map/instance caches still hit.

## Liquids

`LiquidRenderer.InternalPrepareDraw` is ported in `tr-core/src/liquid_draw.rs`. For every
visible cell the cache computes:

- `VisibleLiquidLevel` (including seepage into empty cells between two liquid cells), the
  four edge-wall values and the edge flags, `FrameOffset`, the source rectangle and
  `LiquidOffset`, the waterfall-fade `Opacity`, `IsSurfaceLiquid` and the half-brick
  visibility rule. The waterfall pass keeps the original's 10-row lead, so only the bottom
  rows of an extended draw area see the fade;
- `DrawNormalLiquids` then runs twice like `Main.DrawLiquid(bg)`: opaque (`Opacity`, usually
  1.0) before walls/tiles and `DEFAULT_OPACITY` (0.6 / 0.95 / 0.95 / 0.75) after entities,
  so the parallax never shows through the lake and entities standing in water are tinted;
- water uses the biome style sheet (`Main.CalculateWaterStyle` → `Misc/water_0..14`). The
  atlas stores these sheets under their `Misc/` path, so `Lookups` must resolve
  `Misc/water_N`: the old `water_N` key returned `None` and silently skipped every
  `DrawNormalLiquids` quad — only the `Liquid_*` strips behind blocks were visible;
- the source frame is chosen by `FrameOffset`: interior water uses the plain row
  `(16, 48)`, surface tiles near `worldSurface - 40` snap to the still frame at `Y = 1280`,
  and edge/corner frames use the 3x18 px sheet columns. The old hand-written "always row
  32 + frame*80" pass, the per-column sine offset, the UV ripple flag and the depth tint
  are gone: they produced the repeating white squiggles and dark bands;
- `Lighting.GetCornerColors` is ported as `LightMap::corner4`; the instance format carries
  one tint, so the visible rect is split 3x3 and each cell samples the bilinear corner
  field. This is the original's smooth top-lit water surface.

Solid tiles that stand in liquid also get `DrawTile_LiquidBehindTile`: `LiquidSlope_*`
diagonal masks are drawn behind slopes (and 4 px `Liquid_*` strips behind block edges), so
the open corner of a shore slope shows water. Deviation: the original skips that quad when
water is both above and beside the slope and uses 0.5 opacity below `worldSurface`, which
leaves the known 1 px background gaps at shorelines; we always fill submerged corners and
use full opacity when the cell is under water, as requested. This affects only the
transparent part of shoreline slope/edge frames.

The system is completed by `tr-core/src/liquid.rs`, a port of `Liquid.UpdateLiquid` and
`Liquid.Update`: a 25 000-entry active-cell list (with the 50 000-entry overflow buffer),
one tenth pumped every second 60 Hz tick, per-cell gravity plus the six-neighbour
leveling passes (3/4/5/7-cell averages, edge openness, `Math.Round` banker's rounding),
and retirement after eight quiet passes. Falling water keeps the original's 254→255
quirk. Block edits wake the surrounding liquid cells, so a breach visibly drains.

Not ported: the `Liquid` wave mask/vertex displacement, the water-style fountains and
shimmer sparkles.
The generator adds a forest lake and lava puddles so the pass is visible in the default
world; loaded `.wld` files expose their own liquids.

### Waterfall overlay

`tr-core/src/waterfall/` ports `Terraria.WaterfallManager`; `world_sim/liquids.rs`
runs it on the 60 Hz clock and `render/tiles.rs` emits the quads between the
non-solid and solid tile passes (`Main.DoDraw_Waterfalls`).

- `FindWaterfalls` keeps the 30-tick cadence (`findWaterfallCount < 30 && !forced`), the
  `waterfallDist = (int)(75 * gfxQuality) + 25` scan rectangle around the view, the
  `qualityMax` cap and the exact spout test: an active `halfBrick()` tile whose tile above
  has `liquid < 16` or is solid, with a neighbour holding `liquid > 160` while the other
  side is dry, non-solid and unsloped. The `196`/`460`/`717` fountain tiles spawn types
  `11`/`22`/`26` one row below when the cell underneath is open. The liquid kind check
  order is `tile2` (above) then `tile4` (right) then `tile3` (left), as in the source.
- `UpdateFrame` reproduces all five counters (regular 3-tick, slow 7-tick, rain,
  lava-rain and snow) plus `DoUpdate_AnimateDiscoRGB`. The `Main.mouseTextColor` flicker
  that the lava/shimmer light pulses use is owned by `lighting/` (`M1.5-LIGHT-1`) rather
  than duplicated in the manager.
- `DrawWaterfall` is a line-by-line port of the 640-line geometry walk: `stopAtStep`,
  the `num11..num42` state, the slope/half-brick/liquid neighbour rules, the 1 px
  left/right wobble per row, the `num41 = liquid / 16` clipping of the source rectangle,
  `GetAlpha` (0.6 under a wall or below `worldSurface`, 1.0 above it; lava 1.0, honey 0.8,
  shimmer 0.75, both fading over the last 10 steps) and `StylizeColor` (lava floor 190,
  disco RGB for tile 160, white for 262..268).
- Tint is `style.shade(LightMap::get(x, y))` quantised to bytes, matching the byte
  multiplies of `Color` in XNA; the app turns each record into one atlas quad
  (`Lookups::waterfall_uv`).
- Deliberate approximation: `Main.gfxQuality` is fixed at 1.0 (the value a machine holding
  60 FPS converges to), so the scan distance is 100 tiles and the cap 1000 falls — the
  same as the original at full quality.
- `WaterfallManager.Draw` (`WaterfallManager.cs:1118-1174`) runs one
  `DrawWaterfall(style, liquidAlpha[i])` pass per water style with a non-zero alpha —
  `liquidAlpha[0]→0`, `[2]→3`, `[3]→4`, `[4]→5`, `[5..9]→6..10`, `[10]→13`, `[12]→23`,
  `[13]→24` — so a biome water-style transition cross-fades the falls between the two
  sheets, and the typed falls (lava 1, honey 14, shimmer 25) draw in every active pass
  with their own textures, exactly like the original. `Main.liquidAlpha` itself is the
  `DrawWaters` ramp (`Main.cs:57473-57489`, ±0.2 per frame while the current style is a
  plain water style — lava 1, honey 11 and shimmer 14 sit outside it), held on `App`
  and consumed by the liquid passes as well: `DrawNormalLiquids` draws every
  non-current water style at its alpha over water cells only, then the current style
  over everything (`Main.cs:57497-57506`).
- The walk collects `WaterfallManager.AddLight` (`WaterfallManager.cs:1067-1116`) per
  visited cell — the lava fall's warm flicker, the disco fall, the gemspark falls
  (15-21), the shimmer fall (25) and the fountain (26) — and the light map seeds it as
  a per-cell maximum before the blur (`LightMap::compute_into_with_lights`). Vanilla
  applies the lights one frame late (the engine keeps the previous frame's `AddLight`
  list when `LightTiles` runs); the port does the same, and the walk runs once per
  frame *before* the light map and outside the G6-cached world pass, so the light feed
  is a pure function of the frame index (a walk inside the cached pass made the feed
  depend on the reuse pattern).
- `WaterfallManager.TrySparkling` (`WaterfallManager.cs:1046-1065`): 50+ steps into a
  lit fall, a 1-in-20000 draw scaled by the brightest light channel spawns a `Dust` 43
  sparkle beside the stream (velocity zeroed). The sparkles ride the rain module's dust
  drawing (`Dust` frame 43, scale 0.5) and fade with the generic dust decay
  (`alpha -= 25` per tick) — the type's own AI is not ported.
- `LiquidRenderer.DrawShimmer` (`LiquidRenderer.cs:675-723`): shimmer liquid never
  takes the normal liquid path — every type-3 cell draws twice, the base quad with
  per-corner `GetShimmerBaseColor` wave colours (self-luminous: the light map is
  replaced, which is why shimmer glows in dark caves) and the sparkle overlay with the
  rainbow `GetShimmerGlitterColor` (`SimpleWhiteNoise` + `GetShimmerFrame`). The tile
  batch is premultiplied in XNA and the glitter colour carries alpha 0 (additive);
  this renderer blends non-premultiplied, so the sparkle encodes the premultiplied
  colour as `rgb/max(r,g,b)` with `alpha = max(r,g,b)` — the same source contribution
  with a slightly dimmed backdrop under the sparkle. `Main.DrawTileInWater` (the
  tile-518 submerged redraw) is not ported: the generator never places tile 518.
- Not ported: the `HorizonClouds`-style shader tint of `NextHorizonRenderer.CloudsEnd`
  for falls, `Main.ambientLava(X/Y/Strength)` (an audio-ambience feed, not lighting),
  the `player.lavaOpacity` factor and the storm/rare cloud type branches.
  `TR_NO_WATERFALL=1` skips the walk for A/B screenshots; `TR_SPOUT=x,y` stamps a
  half-brick spout with liquid and a clear drop (`TR_SPOUT_KIND=<0..3>` sets the
  liquid kind) and `TR_POOL=tx,ty,w,h,kind` fills a rectangle with liquid at load, so
  every style, the shimmer pass and the glow are capturable deterministically.

Verification: unit tests in `waterfall/` (spout detection, lava kind, tile 196, frame
counters, emitted geometry, empty `anim_key`); the lake view at `TR_CAM=2240,360`,
`TR_DAY=1` is pixel-identical with and without the pass when no waterfall is visible
(`tools/pixel_diff.py`, 1 440 000 pixels); `docs/screenshots/spout_noon.png` shows a
spout at `TR_SPOUT=100,25`, `TR_CAM=100,32`, zoom 2 — the column samples the raw
`Waterfall_0` colours (`147,177,253` / `50,107,197` / `9,61,191`), i.e. a full-brightness
tint, as `GetAlpha` requires above `worldSurface`.


## Wind sway and tree leaves (TILE-DRAW-3)

`tr-core/src/wind.rs` already carried `Main.windSpeedCurrent`'s walk and
`TileDrawing.GetWindCycle` (`TileDrawing.cs:7676-7700`); the app now owns a
`Wind` instance (seeded from the world seed, `world_surface` set at load) and
advances the `TileDrawing.Update` counters per frame — `_treeWindCounter`
(1/240 + num·2/240), `_grassWindCounter` (1/180 + num·4/180),
`_sunflowerWindCounter` (1/420 + num·5/420) and `_vineWindCounter`
(1/120 + num·0.4/120), with `num = GetLerpValue(0.08, 1.2, |wind|)`
(`TileDrawing.cs:367-375`).

- `TileDrawing.DrawGrass` (`TileDrawing.cs:8326-8375`): the `TileID.Sets
  .SwaysInWindBasic` tiles (hand-transcribed into `wind.rs` as
  `SWAYS_IN_WIND_BASIC` — `TileID.cs:549`; the tiledata generator is due its
  own regeneration pass and was not extended here) draw with the grass cycle:
  position `+= (cycle, |cycle|)`, rotation `cycle * 0.1` about
  `(tileWidth / 2, 16 - tileTop)`, zeroed when the tile's wall does not
  `WallID.Sets.AllowsWind` (hand-transcribed in `wind.rs`, `WallID.cs:95`) or
  `!WorldGen.InAPlaceWithWind(x, y, 1, 1)` (`WorldGen.cs:86858-86879`). The
  row-major tile loop draws them inline instead of deferring them to the
  `WindyGrass` special pass (a documented z-order approximation, like the
  waterfalls); the `WindGrid` gust push (`GetWindGridPush`) is not ported.
  The type-227 frame exceptions of `ShouldSwayInWind` apply.
- `DrawTrees` (`TileDrawing.cs:8137-8245`): every canopy top leans
  `pos += (cycle·2, |cycle|·2)` and rotates `cycle * 0.08` about its trunk
  anchor (the sprite's bottom centre); the frameX-44/66 branches shift
  directionally and rotate `cycle * 0.06` about `(40, 24)` / `(0, 30)`. The
  sway is skipped when the marker tile sits in front of a wall
  (`flag = tile.wall > 0`, line 8073). The renderer gained
  `push_sprite_origin_rot` (a `SpriteBatch.Draw` origin emulation).
- `TileDrawing.EmitTreeLeaves` (`TileDrawing.cs:7779-7891`): every visible
  top/branch rolls `1/leafFrequency` per frame for a leaf
  (`UpdateLeafFrequency`'s |wind| table × 7, halved for hallowed trees, ×3
  for branches, 10000 underground); `WorldGen.GetTreeLeaf`
  (`WorldGen.cs:29384-29598`) resolves the gore id per ground tile (fully
  transcribed, including the gem/vanity/ash overrides and the hallowed
  tables — unreachable in the generated worlds but data-complete), and the
  spawn offsets follow the branch side and `frameY` variants. The roll uses
  `TileDrawing._rand`'s port — a per-frame pinned `FastRandom` — and the
  offsets' `Main.rand` draws fold into the same stream (the original keeps
  two separate streams; the visible distribution is unchanged).
- `Gore_UpdateLeaf` (`Gore.cs:1085-1209`): the leaf falls along a growing
  angle (`velocity.Y += PI/180`; the movement `(−sin, |cos|)` plus
  `wind * 4` above `worldSurface·16`), collides through the port's
  `tile_collision`/`slope_collision` on the 4×4 probe (leading the position
  by 4 px, re-anchored by `vector2 = (−4, 0)` exactly as the source), rests
  on the ground and drifts `×0.94` per tick until the ground frees up or the
  fade completes (`goreTime` 600, then 255 of alpha). The leaves draw from
  the `Gore_910/911` sheets (32 paint columns × 8 variant rows; the port
  draws column 0) in the UI range like the rain, keeping the G6 world range
  out of it. The liquid rest behaviour (honey freeze, lava burn-up) is not
  ported. The G6 `anim` folds the sway counters only while
  `|wind| > 0.08` — the cycle is zero below that (`GetLerpValue(0.08, 0.18)`),
  so a calm world keeps its cache.
- Not ported: the `WindGrid` push system (`TileDrawing.cs:8540-8600`), the
  multi-tile grass/vine/sunflower wind draws (`DrawMultiTileGrassInWind`,
  `DrawVineStrip`, `DrawRisingVineStrip`, `DrawMultiTileVinesInWind` — they
  need the special-point deferral machinery), and the leaf liquid rest
  behaviour. `TR_WIND_FORCE=<speed>` pins `windSpeedCurrent` for screenshots;
  `TR_NO_WIND=1` disables the whole feature for A/B.

Verification: the leaf-frequency/set unit tests; the gust screenshots at
`TR_WIND_FORCE=1.0` (`TR_CAM=2100,375`) diff 6944 px against `TR_NO_WIND=1`
(leaning canopies, flying leaves); two identical runs are SHA256-equal.

## Paint and coatings (TILE-DRAW-4)

`Tile.paint()` (5 bits in `Tile.meta`, E1) already loaded from `.wld` and fed the
map colours and the painted-lamp light; this port adds the RENDER side, the wall
paint and the coatings.

- Storage: `World.extras` — a sparse `HashMap<u32, u8>` keyed by the tile index —
  holds the states that do not fit the packed 12-byte `Tile`: `wallColor()`
  (5 bits, `Tile.cs:335-343`) and the Echo/Glow wall coatings
  (`invisibleBlock`/`invisibleWall`/`fullbrightWall`, one bit each;
  `Tile.cs:528-548`, `:718-721`). Tile-side paint stays in `Tile.meta`; the tile
  Glow bit is the existing `FLAG_FULLBRIGHT`. `.wld` reads the wall paint
  (`WorldFile.cs:2589-2592`) and the coating chain (`:2664-2682`) into the map;
  the native `.trwld` persists it as a sorted sparse section (v3; v2 loads with
  no extras).
- Render: the sprite instance gained a `paint` id and the WGSL fragment recolors
  the texel through a `WorldGen.paintColor` table (`WorldGen.cs:43921-44034`)
  before the light tint multiplies it — the architectural twin of
  `TilePaintSystemV2`'s per-(type, style, paint) baked render targets
  (`TilePaintSystemV2.cs:43-51`), collapsed into a per-instance shader branch.
  Deviation: the vanilla recolor runs as compiled HLSL passes whose source is
  not part of the decompile; this port washes the texel with the paint hue while
  its sRGB luminance carries the texture detail. The basic/deep paint pairs
  (1-12/13-24) therefore shade identically (vanilla deepens saturation), and the
  grass "special group" hue bands are not modelled. Negative paint (30) inverts
  the pixel; illuminant (31) draws unpainted (the vanilla pass 0) and glows
  through the light path.
- Coatings: Echo blocks/walls are hidden (`TileDrawing.IsVisible`,
  `TileDrawing.cs:1411-1432`; `WallDrawing.cs:91`) — the port has neither the
  Echo Monolith nor the Echo sight item, so `CanSeeInvisibleBlocks` is always
  false and the coated state stays hidden. Glow walls sample white light
  (`WallDrawing.cs:96-99`); the tile-side Glow bit (`FLAG_FULLBRIGHT`) was
  already read from `.wld`.
- Light: a painted glowing wall takes the paint colour at the /765 scaling of
  `TileLightScanner.cs:318-324` (3x dimmer than the painted-tile override —
  the wall paint rides `World.extras` now, retiring the "not stored" note in
  `tile_light/mod.rs`).
- Paint tooling: `TR_PAINT=tx,ty,w,h,paint[,wall]` paints a rectangle at load
  for the A/B screenshots.

Verification: the painted-rect screenshots at `TR_PAINT=2080,352,40,6,<1|5|30>`
diff exactly inside the rectangle against the unpainted frame (3756 px each);
the unpainted branch is pixel-identical to main; the extras round-trip test
covers the sparse save.

## Living trees (M5-LIVING-trees)

`crates/tr-core/src/worldgen/living_trees.rs` ports the "Living Trees"
generation pass (`WorldGen.cs:15476-15715`) and `GrowLivingTree`
(`WorldGen.cs:27973-28627`): the anchor/patch placement (outside the spawn
plateau, `Next(4)` patches per side stepping `Next(13, 31)`), the tapering
trunk (`Next(2, 3)` per side, 1/5 single-side widen, patch narrowing, the
`Next(5, 15)` branch cadence and the `1/2` taper until the edges meet), the
side branches (`width * (1 + Next(20, 30) * 0.1)` long with the 2-tile
thickenings as leaf centres), the upper trunk (`Next(width*3, width*5)` rows,
branches `Next(width, width*3)` flagged for round canopies), the roots
(`Next(1, 6)` down-rows then `Next(2, width+1)` diagonal arms of
`Next(width*3.5, width*6)`), the elliptical/round canopy blobs
(`Next(5, 8) * (1 + width*0.05)`, round `Next(6, 12) + width`, the vertical
squash `2.0 - Next(5)*0.1`) with `GrowLivingTree_CanPlaceLeaves`
(`WorldGen.cs:28607-28627`), and the hollow passage (`MakePassage`,
`WorldGen.cs:28865-29133`: 3-wide air, wall 244 lining, the drift every 6 rows,
the shell, the patch tunnels).

- Ground/anchor gates: the ground must be dirt/grass/stone or ore
  (`WorldGen.cs:28004-28010` — sand is excluded, so beaches reject), the
  anchor cell must be inactive, `j >= 150`, and — replacing the vanilla's
  `j >= worldSurface → continue` scan gate (`WorldGen.cs:15549`), which does
  not translate to this port's terrain-vs-`world_surface` offset — the anchor
  cell and its ground must be free of liquid.
- Living wood/leaf rendering is plain-tile: `Tiles_191`'s transparent gaps +
  `tile_hides_wall`'s opacity check let wall 244 show through the bark (the
  `TileID.Sets.DrawsWalls[191]` effect), and the hollow is real geometry (air
  + wall 244). No special draw path exists in the vanilla either
  (`TileDrawing` has no 191/192 case).
- Framing: the generic merge dispatch gains the `WorldGen.cs:82070-82082`
  pairs — 192's Frametest merge against 191 (the port merges unconditionally;
  the vanilla re-frames the wood neighbour and reads `mergeDown/Up/...`, which
  needs the generic -2 openness tables — documented) and 191's
  `TileMergeAttempt(-2, 192)` leaf marker + the dirt fold.
- Leaves (192) are walk-through: `WorldGen.cs:15715` sets
  `tileSolid[192] = false` after the pass; the port adds the `kind != 192`
  exception to the collision helpers, the wall-hide check and the column-
  surface scans (the leaves never cut the sun), while
  `TILE_BLOCK_LIGHT[192]` stays true (the canopies self-shadow, matching the
  vanilla dark interiors).
- Not ported: the hollow room's furniture (door style 7, chair style 5, the
  Living Loom, the buried Living Wood Chest — `TileObjectData`, M1.4-TO), the
  leaf-pile decorations (tiles 187/185 with `PlaceTile` frame tables), the
  passage floor platforms (tile 19 style 23), the horizontal-tunnel doors, the
  secret-seed/remix variants and `PaintTheLivingTrees`.
- Determinism: the pass draws from the worldgen xorshift in the vanilla draw
  order; the unit test `living_trees_generate_for_seed` asserts the 191/192/
  wall-244 counts for seed 1.

## Rain and weather

`rain.rs` ports the shape of `Rain.MakeRain`/`Rain.Update`/`Main.DrawRain` and
`Main.UpdateWeather`'s intensity ramp: a weather event has a target (`maxRaining`),
`cloudAlpha` moves toward it at `0.0005` per 60 Hz tick, and drops spawn per tick with
`25 * (0.25 + cloudAlpha)` attempts per 1920 px of screen width, capped at
`Main.maxRain = 750`. A drop spawns only into a tile with no wall and no solid block,
falls at `(windSpeedCurrent * 18, 14)` px/tick, and dies on solid or wet collision or below
the view, spawning the water `Dust` frame with 10 % of its velocity. `SetBackColor`'s
`1 - cloudAlpha * 0.9 * atmo` multiplies the sky colour, which also dims surface sunlight
via `ApplyColorOfTheSkiesToTiles`. Drops are drawn with the `Rain` texture's 2x40 frames,
rotated by `atan2(vx, -vy)`, tinted by `Lighting.GetColor` * 0.85.

Storms add two pieces from `Main.DrawSurfaceBG`/`Main.SpawnStormLightning`: the
`cloudBG` layers (`Background_112/113`, scale 1.65/1.85, parallax 0.09/0.12) fade in with a
`cloudBGAlpha`-style ramp tied to the rain intensity, and heavy rain (> 0.65) throws a
lightning flash every 5–15 s. The flash is a 0.38-alpha full-screen white overlay that
decays over ~11 ticks; there is no bolt geometry or thunder audio yet.

Snow biomes use vanilla's `DustID.Snowflake` (type 76): rain drops are suppressed and the
ambient snow dust is spawned at `500 * screenWidth/1920` per second (scaled by
`1 + 2*cloudAlpha`), falling at 2–5 px/tick with a per-flake sine sway and the storm wind.
Flakes die on solid or wet tiles and are drawn from `Dust` frame 76 (X = 760, Y = 0/10/20).

First-pass simplifications, as updated by later tasks: the session-local weather
schedule is **retired** — M1.4-BG-22 landed the vanilla `rainTime`/`maxRaining` pair
and the `UpdateTime` start/stop rolls on the shared 60 Hz world tick (see "dayRate
consumers (M1.4-BG-22)" below); the wind-gusts gap was closed by WEATHER-1 (the
"gusts" ARE vanilla's `extremeWindCounter` tiered re-rolls plus the proportional
easing, already ported). Still first-pass: no lightning bolt geometry and no thunder
audio — the flash rides WEATHER-1's tiered thunder chain, not a bolt mesh.

## Biomes

The generator has three surface biomes plus beaches. The forest fills the centre, the snow
biome starts at two thirds of the width, and an inland desert band sits west of spawn
(`desert_center = width/2 - 380`, 340 tiles wide, disabled below 2000 tiles width). The
desert surface is blended toward smooth dunes (`sin(x*0.045)*5` plus low-frequency noise)
before the 1-tile smoothing pass, so it stays slope-friendly. Its strata are 16 px of sand,
14 px of hardened sand (`397`), 16 px of sandstone (`396`) and then stone, with sandstone
walls (`187`) instead of dirt/stone. The tree and plant passes only accept grass/snow, so
the desert stays clear. The background switches to `Background_21` (far, scale 1.25,
parallax 0.37) and `Background_20` (near, scale 1.34, parallax 0.49) when the sampled
surface is sand and the camera is away from the beaches. Two-to-four-tile cacti are placed
on flat sand: tile 80 is flagged important and framed by hand on its 7x3 grid (y=0 top,
18 middle, 36 base) because `WorldGen.CactusFrame`'s arm rules are not ported; cactus
items are collectible but not placeable. Desert structures and loot are not ported.

## Surface backgrounds

`Main.DrawSurfaceBG` is ported for the four biomes the generator has: forest (0),
desert (2), ocean (4) and snow (7). Every layer is gated and tinted by its
`Main.bgAlphaFrontLayer` / `Main.bgAlphaFarBackLayer` entry
(`ColorOfSurfaceBackgroundsBase * alpha`), so two biomes blend while `Main.bgStyle`
cross-fades; a zero alpha skips the layer.

The far pass (`DrawSurfaceBG_BackMountainsStep1`) draws forest `Background_7`, desert
`23` (only when `desertBG != 4`) and snow `35` at parallax 0.15, scale 1·2 and top
`magic·1300 + 1090 + scAdj + 30`. The near pass (`...Step2`) draws forest `8`, desert
`22`, ocean `28` (`Main.oceanBG`, default style 0) and snow `36` at parallax 0.20,
scale 1.15·2 and top `magic·1400 + 1260 + scAdj + 30`, in the original's index order
(0, 1, 3, 4). The front pass draws the biome's three layers: forest `50..52` and snow
`37..39` at 1.25·2/1.31·2/1.34·2, 0.40/0.43/0.49 and `magic·1800/1950/2100 +
1300/1550/1800` (forest; `-50` per texture and the middle layer mirrored horizontally)
resp. `+1350/1600/1850` (snow; no flip); desert `21`/`20` at 2.5/2.68, 0.37/0.49 and
`+1600/+2000`, with no tree line. Everything is laid out in `Main.screenPosition` space
(camera − screenSize/2, **not** divided by the game zoom, so changing `GameZoomTarget`
moves only the world) and the X phase is
`-IEEERemainder(screenLeft·parallax, scaledWidth) − scaledWidth/2` against the screen's
left edge, as in `DrawSurfaceBG`.

`Main.GetPreferredBGStyleForPlayer` is approximated with the same camera-column biome
tests the water style uses — beach → 4, desert → 2, snow → 7, otherwise 0 (the dungeon,
corruption etc. do not exist in our generator). `DrawBG_HandleBackgroundTransition`
waits 30 frames before adopting a new style (60 when entering a forest style), then sets
`bgDelay = -60` as a 60-frame cooldown (0 for a forest target);
`UpdateBGVisibility_BackLayer`/`_FrontLayer` move 16 alpha slots at
`backgroundLayerTransitionSpeed = 0.05` per drawn frame, and
`instantBGTransitionCounter = 2` forces the loaded biome on the first two frames. The
timer/alpha maths is unit-tested in `bg_transition_tests`. Layer tops use
`backgroundTopMagicNumber = -(screenTop - 300) / (worldSurface * 16)` plus `ComputeScAdj`,
exactly like the original, and — like the original — depend only on the camera's Y, never
on the terrain under the camera; an earlier clamp to the local column's surface made the
skyline jump at every hill and was removed (BUG-018). The background batch keeps the
original's zoom-1 matrix, so the layers hold their screen size at any game zoom. Clouds
are world objects at a fixed altitude above `worldSurface` and drift with the wind; the
storm `cloudBG` banks (`Background_112/113`) fade in with `cloudAlpha`.

Not ported: the styles that need `SceneMetrics` beyond our four biomes (corruption 1,
jungle 3, hallow 6, crimson 8, glowshroom 9, the extra forest sets 10..12 and the desert
evil variants 5/13/14), the saved `treeBG*` / `desertBackgroundSet` randomisation (tree
set 1 and the pure desert set are assumed), `DrawSurfaceBG_DrawChangeOverlay` (the black
flash on first entering a background), the far-cloud depth layers
(`DrawClouds_Distant/Closer`), the ocean water-line ambience and the `Background_49` fog.

Also not ported: the per-biome **animated screen effects**. In particular the day-time
desert **heat shimmer** — `SceneState.ApplyVisuals` enables the `HeatDistortion` filter
(`FilterHeatDistortion`, `Images/Misc/noise`, `UseIntensity(4)`, registered in
`ScreenEffectInitializer`) whenever `Main.UseHeatDistortion` and either the camera is on the
desert surface (`TileCenter.Y < Main.worldSurface && ZoneDesert && !ZoneRain &&
!ZoneSandstorm`) or in the underworld (`TileCenter.Y > maxTilesY - 320`) or the Sun Scorch
debuff is active; the shader scrolls the noise so the pixels drift a few pixels side to
side. The `Blizzard`, `Sandstorm`, `Aurora`, `WaterDistortion` and monolith/story filters
from the same block are likewise unimplemented.


The underground backdrop is a port of `Main.DrawBackground_SurfaceTransitionBackground` +
`DrawBackground_DirtBackground`: `Background_1` supplies the one-row transition at
`worldSurface`, `Background_2` tiles as 96x96 blocks from `worldSurface + 1` down, both
scrolled with `Main.caveParallax` (0.9) and covering the whole screen. Every 16x16 cell
takes the light of the world tile it covers, sampled at the un-jittered cell centre on the
*unscaled* grid (`(cellCentre + screenPosition) / 16`, exactly C#'s `num3`/`num4`), so lit
ground stays lit and caves go black at any zoom — this is the original's screen-wide dirt
backdrop, not a per-column fill. Two deviations from the source: a cell is tinted with one
flat colour from `Lighting.GetColor` rather than the original's four `Lighting.GetCornerColors`
(the tile-batch path, which only smooths the light across a single cell and never moves the
sheet); and the `Background_4` band that closes the clamped dirt sheet at the rock
transition was added with M1.4-BG-10. Anchoring the old gradient per column made every
player-placed row of blocks paint a dark "wall" underneath itself; that artifact is gone.

The rock layer and hell are a port of `Main.DrawBackground_DrawRockLayer`,
`_DrawMagmaTransition`, `_DrawMagmaLayer` and `_DrawUnderworldBlackBox` (`Main.cs:53100`,
`:53344`, `:52856`, `:53382`), driven by the same orchestration order as `Main.DrawBackground`
(`Main.cs:52844-52849`: transition → dirt → black box → rock → magma, all under one
`GlobalBrightness` ramp). `GetRockTransitionPoint` truncates `(int)rockLayer -
(int)worldSurface` per row before the difference and snaps up to the dirt block height
(`Main.cs:53088-53099`); `magmaLayer` truncates the span below the surface to 6-row blocks
(`Main.cs:52812-52813`). `hellBackStyle` lives on `World`, rolled at generation from the
same generator as `moonType` (`WorldGen.cs:8247`; vanilla re-rolls it per session at load —
it is not a `.wld` field), and picks the animated magma sheet `Background_125..127` (three
96 px frames cycled every 8 ticks, `Main.cs:62218-62224`) plus the hell floor row
`Background_185..187`. Unlike the dirt pass, the rock/magma cells carry the original's skip
chain: a cell is skipped when it is covered by an active tile but fully lit, when it is
dark, or when it has no `wallLight` wall and the parallax is zero (`Main.cs:53152-53159`),
with the hell variant letting rows below `maxTilesY - 300` draw even when dark
(`Main.cs:52907`). The result is faithful to the original's data: natural walls hide the
backdrop (the wall pass draws over it), so the rock bands and magma only show in open
air — the generated world places `Wall_1/2/187` behind everything underground, which hides
them exactly like vanilla's DirtUnsafe/Cave walls would; the underworld floor row and the
magma transition band draw unconditionally, and the black box fills the gaps above the
hell floor line (`hellBlackBoxBottom` is produced by the previous frame's magma pass).

Deviations kept for the rock/magma passes: the 9-slice/4-slice light smoothing
(`Lighting.GetColor9Slice/4Slice`) is collapsed to the flat per-cell colour, like the dirt
pass; `undergroundBackground` is pinned to the classic set (`Main.cs:53817-53824`) because
`PickUndergroundBackgroundStyle` needs `Main.caveBackX`/`caveBackStyle` and the
`SceneMetrics` zone counts, so the per-biome cave sets (`Main.cs:53825-53987`) and the
`ugBackTransition` cross-fade are dead code; XNA clamps out-of-range source rectangles to
the sheet edge (`SamplerState.PointClamp`, `Main.cs:2774`) and the port clamps the cell's
source offset instead — the quad-UV pipeline cannot smear per pixel, which only affects
the last 16 px column of each 128 px rock block. The draw pad `Main.offScreenRange` (the
padded render target of `WorldSceneLayerTarget`) cancels out of the draw positions in the
port's unpadded viewport and survives only as the dirt sheet's visibility padding.

The sheet is anchored to `Main.worldSurface`, so its value must match `TerrainPass.ApplyPass`:
`worldSurface = (int)(max(surfaceY) + 25.0)` — the *deepest* column, not the mean — with
`rockLayer` snapped up to a multiple of 6 above it (BUG-019). An earlier generator used the
mean column height, which floated the dirt sheet ~60 tiles (≈1000 px) above the true layer
boundary; the underground now begins where the original's does.

## Sky and daylight

`Main.SetBackColor` (day `0..54000`, night `0..32400`) is ported to `color_of_the_skies`:

- Day `< 13500`: sunrise ramp `(230,220,220)·t + (25,35,35)`.
- Day `13500..37800`: pure white — this is why vanilla daytime tiles are lit at 100 %.
- Day `> 37800`: the two sunset ramps (orange, then dusk).
- Night: `(5,5,5) + 30·t`, floored at `minimalLight = 15` (moon-phase dependent in the
  original; we use the full-moon value).

`ApplyColorOfTheSkiesToTiles` pulls each channel 70 % towards its own value
(`(sum + 7·c) / 10`), which is the colour passed to the surface light seed.

### Sun and moon (`DrawSunAndMoon`)

`crates/tr-core/src/sky.rs` holds the placement maths; `draw_sky` draws the bodies on
the camera's screen-space arc (`Main.cs:62848`):

- `celestial_body` computes the anchor `time/period · (screenWidth + 2·texWidth) − texWidth`
  (integer-truncated like the original), the arc `bgTopY + |2·t−1|²·250 + 180`, the scale
  `1.2 − |2·t−1|²·0.4` (the day branch multiplies by `1.1`) and the rotation `2·t − 7.3`;
  `period` is `54000` for the sun and `32400` for the moon.
- `sky_bg_top_y` is the in-game `SceneArea.bgTopY` (`Main.cs:62040`,
  `−screenPosition.Y / (worldSurface·16 − 600) · 200`), so the bodies ride the camera; they
  only draw while `screenPosition.Y / 16 < worldSurface + 2` (`Main.cs:62090`).
- The sun is drawn twice — the raw texture, then `sunColor` at alpha `sunColor.B` — through
  a new rotated background sprite (`push_back_sprite_rot`) so it spins about its centre.
  The moon picks `Images/Moon_0..8` by `Main.moonType` and the 50-px source row by
  `Main.moonPhase`. `sun_color`/`moon_color` port the base (forest) `SetBackColor` branches.
- `World` gained `moon_type`/`moon_phase`: the phase advances one step at each night→day
  rollover (`Main.cs:19982`, wrapping 7→0) and `moon_type` is drawn from the generator RNG
  (`Main.cs:10873`). The native save is now `TRWLD` v2 (v1 still loads with 0/0) and the
  `.wld` reader picks up both header fields.

Not ported: the eclipse (`Sun3`), the `head == 12` sun (`Sun2` + `Extra_280`), the event
moons (`Moon_Pumpkin`/`Moon_Snow`/`Moon_Smile`) and the title-screen grab-the-sun
interaction. The first sun pass is always `Color.White` because no eclipse is modelled.

### Clouds and stars (M1.4-BG-11/16)

`make_clouds`/`make_stars` port the spawn recipes; `draw_sky` ports the draw passes.

Clouds: the field is a deterministic stand-in for the live 200-slot array
(`Cloud.resetClouds`/`addCloud`/`UpdateClouds`, `Cloud.cs:39-293`) — the original's array
state (wind-drift integration, the spawn/kill cycle, the spawn-time overlap rejection of
`Cloud.cs:170-177`) is not simulated. Per-cloud attributes follow `Cloud.addCloud` in its
RNG order (scale `Next(70,131)·0.01` clamped 0.7..1.3 with the `Cloud.Update` 1/1.15 nudge
already applied, rotation `Next(-10,11)·0.01` clamped ±0.02, the flip coin, spawn heights
`-0.25h..0.15h` in the 600 px reference screen, and the clear-weather type branches of
`Cloud.cs:128-139`); the count is the clear-weather steady state `numClouds = 100` (the
`numCloudsTemp` clamp of `Main.cs:58980-58982`, which the `weatherCounter` jumps drive it
into). Anchors spread over a world-span band so every camera cuts about `numClouds` of
them. The draw runs as the three `DrawSurfaceBG` passes in the original order — distant
(`Main.cs:59334`), closer (`Main.cs:59449`), closest (`Main.cs:59520`) — with the
`DrawClouds_*` cY formulas (the closer pass reads the `bgTopY` `BackMountainsStep1`
leaves, `Main.cs:59808`; the closest the near-mountain setup, `Main.cs:59514`), and the
screen X is the world anchor minus `Cloud.GetParallax(scale)` (`Cloud.cs:318-338`) of the
camera pan, the world-space form of `Cloud.UpdateCloudParallax` (`Cloud.cs:295-316`).
Colour is `Cloud.cloudColor` (`Cloud.cs:227-235`) with the distant pass's
`OriginalColorsForCloud` pull (`NextHorizonRenderer.cs:254-263`).

Cloud deviations: the wind drift is a constant light wind scaled by `GetParallax` (the
original integrates `windSpeedCurrent·9·GetParallax` per tick, `Cloud.cs:391`); the storm
type branch (`Cloud.cs:116-127`) and `RollRareCloud` (`Cloud.cs:140-147`, needs the boss
flags) are not applied at generation; `globalCloudAlpha` (= `ProcessCloudAlpha · atmo`) is
1 because no active skies/`atmo` are modelled; and the `HorizonClouds` shader of
`NextHorizonRenderer.CloudsEnd` (sun-tinted cloud shading) is not ported.

Stars: `make_stars` ports `Star.SpawnStars` (`Star.cs:86-158`) in its RNG order — count
`Next(200,400)`, positions in the 1920x1200 design space, `rotation Next(628)·0.01`,
`scale Next(70,130)·0.006`, type `Next(0,4)`, the twinkle/rotation speeds with their sign
coins and the 1-in-40 giant-star roll. The original seeds it with
`FastRandom.CreateWithRandomSeed()` (fresh OS entropy per launch, `Star.cs:88`); the port
pins the stream to the world seed so screenshots stay deterministic. `draw_sky` ports
`DrawStarsInBackground`/`DrawStar` (`Main.cs:63030-63173`): the sky-colour visibility gate
`255·(1 − cloudAlpha·atmo) − skyR − 25 > 0` (fades the field out across dawn; the
graveyard dim is not modelled), the greyscale brightness formula, the
`position/1920·screenWidth + sceneArea.bgTopY` mapping with the frame wrap, and the
`tex.Size()·scale·twinkle` draw with `Star.Update`'s rotation/twinkle speeds (the 0.6..1
twinkle oscillation is sine-approximated around the same centre and range). Two known
deviations: `DrawStar` builds its colour from `default` and assigns only R/G/B
(`Main.cs:63083-63086`) — a zero alpha would make every star invisible, so the port draws
them fully opaque at the computed brightness — and the per-tick twinkle/rotation advance
is driven by the pinned animation clock instead of integrated state.

Screenshot harness: `TR_SHOT` runs pin `dt` to exactly 1/60 (so `anim_time`-driven phases
land on the same value per frame) and `TR_MOUSE=x,y` pins the OS pointer — the vanilla
cursor follows the real mouse and would otherwise differ between runs.

## World generation and native saves

`tr-core/src/worldgen/` is a compact model of the shapes the original generator produces,
not a port of all 45 000 lines. On first launch it builds a 4200×1200 forest world from a
seed in ~165 ms:

1. FBM value-noise surface around 30 % of the height, with a flattened spawn plateau and a
   smoothing pass that limits adjacent height differences to one tile.
2. Grass (top), dirt (16–22 rows), stone, then ash/hellstone for the bottom 170 rows; dirt
   walls above the rock layer, stone walls below.
3. Caves from a large-cavern field plus a worm field, with a 9-tile surface crust.
4. Dirt/stone/ash blobs, then copper/iron/silver/gold blobs by depth and hellstone in ash.
5. Slopes on every one-tile surface transition, a forest lake near spawn (a bowl with a
   raised rim, filled with water), small lava puddles on cave floors, sand ocean beaches at
   both edges and a snow biome in the eastern third (snow surface tiles, winter trees, pine
   background).
6. Common trees (22 px trunk frames, canopy marker, branch markers, base forks) and
   tile-3 grass/flower plants; snow-ground trees use the biome trunk section and `Tree_Tops_4`.

The result is saved as `Documents/My Games/Terrustaria/Worlds/Terrustaria.trwld`
(`tr-core/src/save.rs`): magic/version/seed + header fields + the `frame_important` bitset,
then per-row RLE of the 14-byte tile struct (~3.9 MB for the default world). Loading restores
every field; framing/lighting run afterwards exactly as for a `.wld`. `TR_REGEN=1`
regenerates, `TR_SEED=<n>` selects a seed. The other biomes, structures, the full ocean
biome and hardmode spread are not generated; the original generator remains an M5 milestone.

## Crafting (M3-CRAFT, subset-first)

`craft.rs` carries the subset recipe table (7 recipes over the registry's items,
each citing its `Recipe.cs` registration) and the pure craft actions:
`can_craft`/`try_craft` over `[ItemSlot; 50]` and `adjacent_tiles` — the
`Player.AdjTiles` port with `TileReachCheckSettings.Simple` reach (X 5 / Y 3).
Deviations: `RequireGroup(RecipeGroups.Wood)` recipes are expanded to concrete
ids (Wood → 9); consumption is a single index-ordered pass matching
`ConsumeItemsFrom` (`CraftingRequests.cs:244-290`) over **50 slots** (the C#
consumes over 58 incl. coin/ammo — no subset ingredient lives there); a full
inventory still crafts (the drop-on-full bridge is not wired — vanilla drops).
The panel (`ui/crafting.rs`) draws available recipes with a 44 px cell grid;
items without extracted sprites (the atlas carries only the world's used tile
frames) draw a grey placeholder. The roadmap's "contexts 4/33" is stale — the
crafting contexts are **41/42/43** (`ItemSlot.cs:126-130`). Clicks:
`handle_crafting_clicks` (`clicks.rs`) runs before inventory clicks and crafts
on the press (`Main.cs:42250` hover family); the rotation source for item use
stays with M2-ANIM.

## Ambient sky creatures (M1.4-BG-19)

`M1.4-BG-19` ports `GameContent/Skies/AmbientSky.cs` (the `SkyEntity` base, the
`FadingSkyEntity` envelope and the 18 live creature classes, plus the
`AmbientSky : CustomSky` manager) and `GameContent/Ambience/AmbienceServer.cs`
(the spawn scheduler) into `crates/tr-app/src/ambience.rs`; the draw side hooks
into `render/sky.rs`'s background passes at the C# `DrawToDepth` boundaries.
The single-player `NetAmbienceModule` loopback is inlined: the spawn seed is one
draw from the scheduler's `Main.rand` stand-in stream (`ambience_rng`), and every
entity attribute comes from the exact-C# `FastRandom(seed)`
(`tile_light/palette.rs`) in the constructor's original draw order — the order is
pinned by per-entity characterization tests. Per-tick updates consume no RNG.

Determinism/proven pairs (pristine `.trwld` copies per run, `TR_DAY=0.5`):
`TR_NOPLAYER` entity shots SHA256-equal at zoom 1.0/1.5/2.0 (`TR_SHOT_FRAME=600`,
`TR_AMBIENCE=Eyeball,7`); player runs at frames 200/300/540 equal pairs; the
`AirBalloon` (scheduler-spawned) visibly renders with its fade alpha. The forest
spawn area's canopy **hides** the EOC eyeball completely — entity shots there are
byte-identical to no-entity runs, which is itself the sandwich proof: the eyeball
draws behind the front tree rows, exactly where the C# depth boundaries put it.

Deviations, all deliberate:
- **Ocean-line fade deferred** (research Q4): `Helper_GetOpacityWithAccountingFor
  BackgroundsOff`'s sibling `Helper_GetOpacityWithAccountingForOceanWaterLine`
  (`AmbientSky.cs:72-79`) is constant 1; the producer
  (`Main.cs:59889` `UpdateOceanWaterLineForAmbience`) is cited for the follow-up.
  Only visible when entities cross the ocean water line.
- **The atlas grows**: loading the 28 `Images/Backgrounds/Ambience/*` sheets
  reshuffles the tallest-first shelf packer, so screenshots differ from pre-task
  binaries by 89 edge pixels (the B10 "normalised UV phase" class; the BG-11
  content set itself shifted these when it added sheets). Twin-run equality —
  the repo's screenshot gate — holds.
- **`tenthAnniversaryWorld` = false** (the spawn-timer halving is dead),
  **remix-world branches absent** (windy-state override `Main.cs:13022-13024`,
  ambience packet gate), both seeds the port does not generate.
- **`NPC.butterflyChance` sits at its reset value 0** (`NPC.cs:95172`): its
  update logic is NPC-spawn-owned (`NPC.cs:93378-93397`, M4) — the gate reads
  the field 1:1, so Butterflies spawn whenever their other conditions hold.
- **The Meteor sky is unreachable until M4 meteor events**: `SkyEntityType.
  Meteor` has no `_spawnConditions` entry and `ForceEntitySpawn` fires only from
  the landing window (`Main.cs:65471-65489`); the queue and window hook shape
  are ported.
- **`BoneSerpent` stays dead**: the enum value and the texture exist, but
  `AmbientSky.Spawn` has no case (`AmbientSky.cs:1394-1480`) — ported as-is.
- **The scheduler pool pins `_spawnConditions` declaration order**; the C#
  enumerates a `Dictionary`, whose order is implementation-defined.
- **dayRate** landed with BG-21 (ANALYSIS "Sundial/moondial fast-forward
  (M1.4-BG-21)"): the `max(1, dayRate)` shape at both sites (the scheduler timer
  decrement and the entity update repetition, `SkyManager.cs:24-46`) now consumes
  the real rate — 60 while a dial fast-forwards, 1 otherwise.
- Screenshot harness: `TR_AMBIENCE=<SkyEntityType>[,seed]` queues one forced
  spawn with a pinned seed at load (the `ForceEntitySpawn` path) — test-only.

## Screen filters (M1.4-BG-18)

`M1.4-BG-18` ports the `ScreenShaderData` post-process subsystem: a
`PostPipeline` in tr-render (`post.rs`/`post.wgsl`) renders the sprite pass
into an offscreen sRGB target and applies the active filter as a fullscreen
pass — one WGSL module mirroring `ScreenShader.fx`'s named techniques
(`PASS_TABLE` extracted from `ScreenShader.xnb` by the new tr-content Effect
reader; 22 passes ⊇ the initializer's table). The driver
(`app/filters.rs::FilterDriver::apply_visuals`) ports `SceneState.ApplyVisuals`
(`SceneState.cs:64-253`) over the Phase-1 sensors (`scene.rs`: SceneMetrics
window counts per `SceneMetrics.cs:373/595-616` with the
`GraveyardTileCount = _tileCounts[85] − _tileCounts[27]/2` rule, the Sandstorm
state machine from `Events/Sandstorm.cs:32-138`, SceneState fades/
MoveTowards `:472-491`). Deviations:

- WGSL filter bodies are documented approximations of the compiled HLSL
  (`ScreenShaderData.cs` semantics per pass; no bytecode translation).
- One winner at a time by `EffectPriority` then activation, visibility =
  fade × shader opacity > 0 (`Filter.IsVisible`/`CombinedOpacity`).
- Storm fg/bg pair: the dominant-opacity side runs (post cannot draw behind
  the scene); both WGSL passes exist.
- The filter wash covers the HUD (our single sprite pass renders UI into the
  same target; vanilla draws the HUD after the filter).
- Aurora is a SkyManager visual (`ScreenEffectInitializer.cs:65`), not a
  filter — not ported. Towers/monoliths/MoonLord are M4 content.
- Items without extracted sprites draw a grey placeholder in the crafting
  grid (M3-CRAFT, same atlas limitation).
- Harness: `TR_POST=1`, `TR_SANDSTORM`/`TR_GRAVEYARD`/`TR_BLOODMOON`/
  `TR_STORM`/`TR_XY` pins for deterministic per-filter shots.

## Player (M2 first pass)

### Composite arm rotation (M2-DRAW-2)

The layer builders consume `Player.CompositeArmData` (`Player.cs:219`): an enabled
arm frames from the composite columns — front 7 / back 8 — at the stretch row
(Full/ThreeQuarters/Quarter/None → 0-3, `PlayerDrawSet.cs:2246-2271`) and rotates
around `center + GetCompositeOffset_*` (front (−5, 0), back (+6, +2),
`PlayerDrawLayers.cs:4082-4090`). Deviations:

- The rotation **source** is a stand-in: `swing_stand_in` enables the front arm
  during the swing arc (`Full` mid-swing, `ThreeQuarters` at the edges). The
  vanilla angles live in `ItemCheck_ApplyUseStyle` (`Player.cs:50679`) and
  `AnimatePlayerAndGetItemFrame` (`:43372`) — M2-ANIM's territory; the builders
  consume whatever the state carries, so M2-ANIM only replaces the source.
- The XNA draw-origin semantics map to our top-left rotation pipeline as
  `pos = pivot − R·(size/2 + offset)`; at rotation 0 this collapses to the plain
  top-left, so disabled arms render exactly as M2-DRAW-1 did (pinned by
  `test_dw_2_3_idle_parity`).
- The back-arm composite activates only from state (`back_arm.enabled`); the
  starter never enables it (the vanilla back-arm composites are hold-style/pet
  poses — M2-ANIM/M3-EQUIP).
- `missingArm`/`missingHand` skin passes and `gravDir = -1` are not modelled.

### Layer builders (M2-DRAW-1)

The player composition is pure data: `render/player_layers.rs` builds a
`Vec<LayerSprite>` (`assemble_layers`) and `draw_player` only pushes it — the
`LegacyPlayerRenderer.cs:185-255` sequence with the held item in its
`weaponDrawOrder` slot (`PlayerDrawSet.cs:358`: the starter can only reach
`BehindFrontArm`). Deviations from the C#, all deliberate:

- The swing arc stays the rotation input — vanilla `itemRotation`/`itemAnimation`
  come from `ItemCheck_*` and belong to M2-ANIM. The item's grip origin is
  already the C# rule (`PlayerDrawLayers.cs:3161-3188`: bottom corner mirrored by
  direction; useStyle 8/9 handled), so the old centre-pivot arc pass is gone.
- The `Players[8]` undershirt passes (back arm, front arm/shoulder) draw with
  the vanilla `underShirtColor` (160,180,215) (`Player.cs:2562`); they are
  layered exactly as `PlayerDrawLayers.cs:1353-1362` (back arm 5, 8, 13) and
  `:3661-3706` (arm 7, 8, 13, 6; shoulder 7, 8, 13, 6) — M2-DRAW-4.
- Source-rect fidelity (M2-DRAW-4). The hair rides its own trimmed frame
  (`PlayerDrawSet.cs:1770-1786`: `Height -= 2`, `Y -= 336` = six body rows up,
  clamped at 0) instead of the body frame, so the art stays inside
  `Player_Hair_*`'s 14 rows on every pose; the helmet takes the normal-hat
  branch's `Height -= 4` crop (`PlayerDrawLayers.cs:2254-2261`). The row count
  of a part strip rounds up: `Player_0_0`/`_0_1`/`_0_2` are 1118 px = 19 full
  rows plus a 54-px row 19, and the walk cycle's fastest band is row 19 — a
  `floor` count clamped it back onto row 18 (the head lagged the legs by one
  frame). The one deliberate deviation: the quad's bottom UV is clamped to the
  sheet, so row 19 draws its 54 art px into a 56-px quad (a 2-px stretch) rather
  than sampling the neighbouring atlas sheet the way a packed atlas would.
  The torso/shirt sheets stay cropped to the two columns the draw can reach
  (`torso_cell`'s row-5 override is column 1, `PlayerDrawSet.cs:1940,1975`).
- Shoulder cells are constants ((0,1)/(1,1), `PlayerDrawSet.cs:1938-1939`) and
  the shoulder offset is assumed zero without body armour
  (`DetermineShoulderOffsets` is armour-driven); the composite is male-only
  (`Player_0_*`, the `!Male` Y+2 shift of `:2036-2040` not modelled).
- The UI character preview (`UICharacter`) keeps its own inline composition —
  it draws in screen space at fixed frames and does not need the builder yet.
- Harness note: TR_SHOT twin-run equality is timing-sensitive
  (`anim_time += dt` on real time, `frame.rs:47`, plus the 60 Hz stepper's
  accumulated ticks); a same-run A/B against the pre-builder binary shows the
  delta confined to the player region (858 px), while old-vs-old twin pairs
  drift by themselves (312k px). A tick-pinned screenshot mode is a separate
  harness task.

`tr-app/src/player.rs` implements a fixed 60 Hz step with the original movement constants:
gravity `0.4`, max fall `10`, run `3`, ground/air acceleration `0.2/0.1`, jump speed
`5.01` with `jumpHeight = 15`: pressing jump sets the upward speed and holding it keeps
resetting that speed for up to 15 ticks (`Player.JumpMovement`), so a tap jumps ~31 px and
a full hold ~95 px. A new jump needs a fresh press edge.
Movement is the original pipeline, ported in `tr-app/src/collision.rs` from
`Collision.cs`: `TileCollision` resolves the AABB against `Main.tileSolid` tiles and lets
slope triangles pass when the body approaches from their open side; `StepDown`/`StepUp`
run on the pre-collision velocity and lift the hitbox over one-tile ledges and down
ledges while walking; `SlopeCollision` runs after the move and slides the feet along
ramps (slope 1 rests on the diagonal at the hitbox's left edge, slope 2 at the right).
The instant lift of the step helpers is hidden by `Player.gfxOffY`, which eases back to
zero at `(1 + |vx| / 3) * stepSpeed` px per tick, and the sprite is drawn at
`position + gfxOffY` (`Player.VisualPosition`). Jumps gate on `velocity.Y == 0` after
collision, exactly like `Player.cs`. Water buoyancy, half bricks (bottom 8 px solid) and
ceiling slopes 3/4 (solid above the diagonal) follow the same code paths.
The player steps at a fixed 60 Hz and `Player::interpolated(alpha)` blends the previous and
current positions for rendering and camera follow, so a jump is not a 60 Hz staircase on a
high-refresh display. NPCs use their own fixed 60 Hz accumulator (`NPC.cs` update rate);
before that they advanced once per rendered frame and were far too fast at 500+ FPS.
Respawn returns to `spawn_x/spawn_y` from the world header.

Survival ports three `Player.cs` mechanics:

- **Breath** — `breathMax = 200`; while the centre tile is water/honey, one unit is lost
  every 7 ticks. At zero the player takes 2 damage per 7 ticks (no i-frames, like the
  direct `statLife -= 2`), and breath refills at 3/tick out of water. A blue HUD bar appears
  while breath is below maximum.
- **Fall damage** — the fall origin is recorded whenever the player is not falling or is
  wet; landing after more than 25 tiles deals `(tiles - 25) * 10` damage.
- **Lava and regeneration** — lava applies a knockback hit through the normal 0.8 s hurt
  cooldown; after 5 s without damage the player regenerates 1 HP every 0.5 s.

The character is composed from the original 40×56 part strips (`Player_0_0` head,
`_1/_2` eyes, `_3` torso, `_6` shirt, `_7` arms, `_10` legs, `_11` pants, `_12` shoes,
`Player_Hair_1`) at `(x-10, y-10)`, using the default skin/clothing colours and a faithful `Player.PlayerFrame` port
(`player.rs::update_frames`). The legs and body each carry a frame row (`leg_frame`/
`body_frame`), and since M2-DRAW-3 the whole of `PlayerFrame()` (`Player.cs:35755-36647`)
is ported arm for arm (see "The full PlayerFrame chain" below) rather than the three-arm
approximation this paragraph used to describe. (An earlier version
returned row 5 at rest, which made the idle character look like it was about to jump, and
used an unrelated `walk_time` counter that flickered between rows.) The legs/pants/shoes
draw with `leg_frame`, the torso/arms/head/eyes/hair with `body_frame`. Torso and undershirt
always use composite cell `(0,0)` (only the head and arms animate); the hotbar, hearts,
breath bubbles and boss bar use the original
`Inventory_Back`, `Heart`, `Bubble` and `UI_BossBar` textures with the classic layout. Arms use the composite layout from `PlayerDrawSet.CreateCompositeData`
(the six-column swing table for the front arm, back arm two rows lower); the near-side
(item-holding) arm is drawn from column 8 because the per-item shoulder rotation is not
modelled yet. Entity sprites (player and NPCs) snap their top-left to an integer screen pixel before the
instance is uploaded, so nearest sampling keeps exactly the same source texels while the
camera moves; the camera itself still interpolates smoothly, and the world render camera is
snapped to `round(cam * zoom) / zoom` before any world pass so integer world coordinates land
on whole screen pixels at every zoom (without it, tiles crawled sub-pixel under the
already-snapped player while moving, which read as a shimmering pixel grid). NPC sprite
offsets align the
last opaque art row (not the padded frame) with the hitbox bottom, which is what removed
the slime/zombie hover. The camera follows the player; `TR_CAM`/`TR_PAN` keep the free
debug camera, `TR_NOPLAYER` hides the player, `TR_XY` places them at a tile and
`TR_HOLD=right,jump` holds a key for deterministic walk screenshots. The selected
hotbar block is drawn in the hand (14×14 tile frame) with a 0.28 s swing arc from raised-
behind to forward-down **only while a swing is active** (`PlayerDrawLayers.DrawPlayer_27_HeldItem`
gates on `itemAnimation > 0 && useStyle != 0`), so the hand is empty at rest like the
original; the world cache key includes the kind and quantised phase. Not ported yet:
hold-style items (torches and other `holdStyle != 0` items are still hidden), the full
`itemAnimation` timings and tool-specific swings, armour/accessory layers, the
composite-arm rotation while holding items, buffs and the rest of `Player.cs`.

Escape opens the inventory instead of quitting: `Main.DrawInventory`'s 10x5 grid of
56 px cells at (20,20) at `inventoryScale = 0.85` (`ItemSlot.Draw` context 0: grey
`InventoryBack`, hotbar row `InventoryBack9`), while the closed hotbar is context 13
(grey back at 200/255, selected slot in yellow `InventoryBack14`, `ItemStack` digits with
slot 10 as "0"). Slot icons come from the original `Item_<id>` sprites through
`item::item_id` (the `ItemID` of each slot kind), falling back to the tile frame where no
sprite is loaded yet. UI text uses the original ReLogic `DynamicSpriteFont`s
(`Fonts/Mouse_Text.xnb` for labels and item names, `Fonts/Item_Stack.xnb` for stack counts
and slot numbers), decoded at runtime by `tr-content` and drawn with
`DynamicSpriteFont.DrawString` metrics (`assets::UiFont`, `push_font_text`).
Stack interaction is a port of the `ItemSlot.Handle` (context 0) subset: left click
takes, places or swaps a whole stack, right click takes half or places one, stacks merge
up to `Item.MAX_STACK = 999`, and the carried stack follows the cursor (`Main.mouseItem`).
On the closed hotbar a left click selects the slot. While the inventory is open, mining
and placing are suppressed like the original's `mouseInterface`. The model now holds all
50 `Player.inventory` slots (sidecar save version 2); pickup follows `Player.PickupItem`
(stack into the hotbar, then an empty hotbar slot, then the storage rows).

## Editing and hotbar (M3 first pass)

`tr-core/src/edit.rs` implements a single-tile break/place: the background wall is kept,
the world revision is bumped, the chunk is marked and the column surface is recomputed.
`framing::reframe_area` runs the generic frame chain and the wall lookup over a 5×5
neighbourhood, so a block edit does not need to re-frame the whole world. The app adds a
10-slot dynamic hotbar (kind/name/count, empty slots adopt a new kind), LMB mining with a
per-kind timer, RMB placement with reach/support/player-overlap checks, 1–9/0 selection, a
target outline and F9/save-on-close to the native `.trwld`. Placing consumes the slot.

Walls are editable too: `edit::break_wall` clears the wall and re-frames the 5×5 area,
`edit::place_wall` fills an empty wall cell, and both bump the revision. Wall items are
encoded as `10000 + WallID` inside the normal hotbar/drop kinds, so the sidecar format and
pickup logic need no extra field; the HUD and drops use an interior wall cell as the icon.
LMB mines an exposed wall when the cell has no active tile (0.35 s dirt / 0.7 s stone) and
RMB places the selected wall when a neighbour wall or the cell's own block supports it.

Broken tiles spawn an `item.rs` entity instead of teleporting into the inventory:
`ItemDrop` applies gravity `0.3`, axis-separated collision with the tile grid, 0.35/0.3
bounce damping and a ground bob. After a 0.35 s delay, drops within 42 px are pulled toward
the player and collected within 10 px into the first matching stack or empty slot. Drops
are drawn from the tile's plain 16×16 frame with the light map (separate `Item_*` sprites
are not extracted yet). A `.trplayer` sidecar stores position, HP and the ten slots (the
version-1 loader accepts the old four-slot tail) and is applied at load if the saved
position is not inside a solid tile.

`WallID.UnbreakableBlockWall` (350) is faithful too: `Player.CanPlayerSmashWall` refuses
it, `WorldGen.ReplaceWall` cannot overwrite it, and `CanKillTile`/`IsTileReplaceable` make
any tile in front of it unbreakable and unplaceable. `edit::WALL_UNBREAKABLE` guards
`break_wall`, `break_tile` (`remove_tile`) and `place_tile`; the dungeon's pre-Skeletron
wall rule is not needed because the dungeon is not generated.

Not ported: the full `Item.cs` use logic, item sprite sheet lookup, inventory
windows/crafting/chests, paint, actuators/wiring, and liquid re-simulation after
an edit. (The tool requirements and the mining speed landed since — `tr_core::tool`'s
pick gates + the pacing in the mining branch; the full `Item.cs` use logic stays out.)

### Torch attachment (`WorldGen.CheckTorch`)

`tr-core/src/torch.rs` ports `WorldGen.CheckTorch` with the helpers it calls
(`TopEdgeCanBeAttachedTo`, `PlatformProperTopFrame`, `IsTreeType`), because torches are
`frame-important`: their `frameX` is derived from the neighbours, not stored in the save.
`WorldGen.TileFrame` runs it for every `TileID.Sets.Torches` tile, so the port is called
from `edit::place_tile` (right after the block is written) and from
`framing::reframe_area` for every torch in the 5×5 area — mining the block a torch hangs
on therefore re-frames or removes it in the same edit, as in the original.

- `frameX` is `0` when the torch stands on a solid, attachable tile below (or a platform
  whose top edge can be attached to, `PlatformProperTopFrame`), `22` when it hangs on a
  solid/beam/tree trunk to its left, `44` for the right, and `0` when only a background
  wall holds it. An existing `frameX >= 66` (the tall/hanging variant) keeps its 66
  offset, matching the `short num = (tile.frameX >= 66) ? 66 : 0` in the source.
- When nothing holds it and the cell has no wall, the torch is removed
  (`WorldGen.KillTile`): `edit::remove_tile` keeps the wall and the liquid, and
  `place_tile` reports failure so the caller does not consume the item.
- Data tables: `Main.tileNoAttach` (63 ids), `TileID.Sets.IsBeam` and
  `TileID.Sets.IsATreeTrunk` are literal extractions held in `torch.rs`, because
  `tools/GenTileData` no longer matches the current decompilation (see risk 9).
- Deviation: the saved-world load path does not re-run `CheckTorch` (frames come from the
  file); only edits and placements do. Vanilla re-frames the whole world on load.

Verification: nine unit tests in `torch.rs` (standing, left/right wall, wall-only,
no-attach neighbour, platform below, half-brick below, unsupported removal, 66 offset);
in-game screenshots with `TR_PLACE` (now a `;`-separated list) show an upright torch on a
block and two torches rotated onto the left/right faces of a block, and an unsupported
torch never appears.

## NPCs and combat (M4 first pass)

`tr-app/src/npc.rs` adds a hopping blue slime (gravity, AABB collision, hop timer, wall
bounce, 30 HP, 12 contact damage) with the `AI_001_Slimes` water tail: the body counts as
wet when any liquid tile overlaps it, gravity drops to `0.2`/max fall `7`, and the slime is
pushed up by `0.5`/tick (capped at `-4`), bounces off the floor with `-2`, and flips
direction when it rises along the same x — so slimes float and bob at the surface. A
flying demon eye steers toward the player with a sine bob (25 HP, 14 damage) and a zombie standing in for `AI_003_Fighters`: an 18×40 hitbox
with 45 HP, 14 damage, a 0.6 px/tick chase toward the player, a jump when a one-tile ledge
blocks the path, and 6 damage/second while the world is in daylight (the vanilla burn).
Night spawns mix zombies, demon eyes and slimes; daytime spawns are slimes. Up to four NPCs
are kept near the player and respawn every six seconds.

King Slime (`NPC_50`, 174×120 frames) is the first boss: a 98×92 hitbox with 2000 HP and
40 contact damage. The simplified `AI_015_KingSlime` hops every 1.4-2.2 s at ±2.4 px/tick
with `-8` jump velocity, teleports next to the player when they are more than 70 tiles away
(with the same surface collision check as the spawn), and asks the app for a blue slime
minion each time it lands (capped at four slimes). `B` summons it; a top-centre bar shows
its name and HP. Not ported: the shrink/grow visuals, teleport dust, the full minion
roster, boss loot and the Expert/Master scaling.

Enemy loot is a first `Item.NewItem` pass: slimes drop 1-3 copper (5 % one silver),
demon eyes 1-2 copper, zombies 2-5 copper (20 % one silver) and King Slime 3-6 gold coins.
Each is an `ItemDrop` in the `COIN_ITEM_BASE + 0..3` range drawn from `Item_71..74`; coins
are collected straight into a copper-valued purse instead of a hotbar slot, and the HUD
formats it as `12g 34s 56c`. The purse is saved as a fourth optional line in the
`.trplayer` sidecar. A full loot table/`ItemDropDatabase` is not ported.

`projectile.rs` is the first `Projectile.cs` pass: the starting Shuriken item (a
`WEAPON_ITEM_BASE + 1` hotbar kind) throws `ProjectileID.Shuriken` (`Projectile_3`) toward
the cursor at 11 px/tick. The simulation is fixed 60 Hz with two sub-steps so the 6×6
hitbox cannot tunnel; solid tiles reflect the velocity with 0.6 damping (up to six
bounces), the first touched NPC takes 10 damage through the shared `Npc::hurt` path, and
the projectile expires after 2 s. It is drawn rotated in the UI range, tinted by the light
map. Not ported: the full `Projectile.cs` behaviours (homing, pierce, gravity per type,
`Projectile.NewProjectile` data) and projectile-vs-projectile logic. The player has 100 HP,
0.8 s hurt cooldown, knockback, instant respawn on death and a HUD health bar. LMB over a
slime within four tiles deals 15 melee damage with a 0.3 s cooldown; damaged slimes show
an overhead bar. The original `NPC_1` 32×26 frames are tinted blue and lit by the tile
light; NPC position/animation are hashed into the G6 frame key.

The instanced renderer is split into three ranges: an always-rebuilt back range (sky
gradient, parallax layers, clouds, cave backdrop), a cached world range (tiles, walls, slopes,
liquids, trees, items, player, NPCs) and the UI. The world range is keyed by the camera
floored to a tile, zoom/screen, light generation, light mode, animation step and the
entity hashes, so sub-tile pans and static views rebuild only the back range + UI. With the
D3 light valid region this gives 71–74 % world-range reuse while panning and 98 % when
static; a 32-tile light cache margin keeps panning light at 0.03 ms in the stress view.

Not ported: biome/event spawning rules and spawn rates, the other ~780 NPCs, AI state
machines, full loot tables, defense/knockback resistances, crits, buffs/debuffs, the other
bosses and multiplayer sync.

## Asset pipeline

- Terraria ships XNA **XNB v5** containers. Terraria's build uses Xbox `XMemCompress`
  (LZX, 16-bit window, headered blocks) rather than the usual XNA LZ4 — see
  `ReLogic.Content.Readers.XnbReader`, which defers to XNA's `ContentManager`.
- `tools/XnbExtract` (C#/.NET 10) implements the container parse plus a MonoGame-derived
  LZX decoder and the DXT1/3/5 decoders, then writes RGBA PNGs without external packages.
- Result: **15 100 / 15 123** textures extracted (13 993 PNG after `.xnb` dedupe of
  subfolders). The 23 failures are all `EndOfStreamException` while reading the pixel block
  of a few `Projectile_*` and `Star_4` — likely multi-mip or an unhandled surface format.

### Tileset layout (important for exact visuals)

| Sheet kind | Cell stride | Art size | Art inset | Notes |
| --- | --- | --- | --- | --- |
| `Tiles_*.png` | 18 px | 16×16 | 0 | 16 columns typical, 15 rows for the base set |
| `Wall_*.png` | 36 px | 16×16 | 8 px | Original draws a 32×32 source rect offset by −8; the inner 16×16 lands exactly on the tile |

- `Wall_1` does not exist (wall ids start at 1 → `Wall_1.xnb` is absent in some
  distributions; the pack maps ids directly and tolerates missing sheets).
- Block tilesets also contain randomized variants selected per tile (`frameNumber`), which
  the save file does not store; `framing.rs` substitutes a position hash. The same applies
  to the wall variant (`wallFrameNumber`).
- Non-important tiles store `frameX = frameY = -1` in the world; the original computes them
  on load in `WorldGen.TileFrame`. Important tiles (torches, furniture, plants, trees…)
  keep their frames in the save file.

## Observations / risks

1. **Autotiling is the dominant visual factor.** One 16×16 frame per tile type produces
   repeating "cobblestone" seams. The generic block chain and the merge families are now
   ported (~5 900 000 tiles framed in ~150 ms); slopes/half-bricks and tree composition are
   drawn, and the remaining gaps are per-type special branches, which affect a minority of
   tile types.
2. **Lighting is the second factor.** The scanner, blur, surface seeding, `GlobalBrightness`,
   9/4-slice block smoothing and the four original modes are ported (see `#lighting`).
   Sun/moon light, flicker and invisible blocks remain.
3. **Backgrounds are the third.** Parallax mountains, clouds, sun/moon and stars now exist,
   but biome-specific skylines, rain and the real textured underground background are
   still placeholders; the sky + textured `Background_1/2` underground backdrop avoids
   leaks and hard edges but does not cover every biome or the rock/hell layers.
4. **Content volume dominates long-term effort.** ~6 200 items, ~780 NPCs, ~1 100
   projectiles, ~750 tiles; most behaviour is hand-written per id, not data-driven.
5. **World generation** is a large but well-isolated subsystem (passes + structures). The
   compact generator and native save give a stable test world; full parity is M5.
6. **Networking** must match the exact 1.4.5.8 packet layout to interoperate; not required
   for single-player parity.
7. **`tools/GenTileData` was stale.** The current decompilation writes `Main.`-prefixed
   assignments (`Main.tileSolid[0] = true;`) and lives under `decompiled/Terraria/ID/`
   and `decompiled/Terraria/Graphics/Light/`, while the generator's regexes expected the
   bare names and the old flat paths. Running it then emitted empty tables (`sets: 0`,
   `tileSolid=0`) and would have destroyed the generated data; it was reverted. **Closed
   2026-11-01**: the generator matches the current decompilation again
   (`Architecture refactor` below) and `tiledata/` is generated from it; the hand-extracted
   tables in `torch.rs` stay as-is.

Note: tile id **0 is dirt**, not air. The asset usage scan must include it; the active flag
is the only air test.

## Content pipeline (`tr-content`)

Textures and fonts are read from the bundled `assets/Content` copy of the retail `.xnb`
files at runtime with the original pipeline (no pre-extracted PNGs and no fallback): `tr-content` ports the XNB v5 container, the
Xbox XMemCompress LZX decoder (`LzxDecoder.cs`, 16-bit window, headered blocks),
`Texture2DReader` (Color/DXT1/DXT3/DXT5) and ReLogic's `DynamicSpriteFontReader`.
Verified by test: `Images/Tiles_0.xnb` decodes pixel-identically to the old
`assets/Images/Tiles_0.png` (0 differing pixels). XNA `SurfaceFormat.Color` is RGBA in
memory (not BGRA), which the old C# extractor already wrote correctly.

The ReLogic in the shipped game differs from `decompiled-libs` in the primitive
encoding: the glyph character list is a UTF-8 string (not fixed-width chars) and the
page header carries extra fields. The font parser therefore locates the page texture and
each table structurally and validates the whole page (mip size == surface size, glyph
rects inside the page, one kerning entry per char, UTF-8 characters). Mouse_Text parses
as one 256x256 DXT3 page with 212 glyphs, line spacing 29; Item_Stack has 212 glyphs,
line spacing 24.

## Item registry (M3-ITEM-1, 2026-11-04)

`tr-app/src/itemdata.rs` (GENERATED by `tools/GenItemData/gen.py` — never hand-edit,
regen must produce an empty diff) holds all 6196 vanilla ids: the effective
`Item.SetDefaults(type)` result — `ResetStats` stock (maxStack = `CommonMaxStack`
**9999**, the 1.4.5 stack raise, `Item.cs:49017` + `:49966`; useTime/Animation 100;
damage −1; slots −1; stack 1) then the `SetDefaults1..5` case body
(`Item.cs:1387-47809`) then the dispatcher epilogue (`:48713-48925`: 30 value
overrides, bait rare ladder, projHook zeroing, drill/chainsaw ×0.6 min 1, food
holdStyle, the 1803-1807 redirect resolved generator-side, deprecated → TurnToAir
zeroing). `tr-app/src/registry.rs` is the hand-written accessor surface (`get`,
`name`, `max_stack`, tile/wall/projectile mapping, `net_default` −1..−48,
`on_created` 5437→5358); unknown-id fallbacks (≥ 6196: "Item", 999, 0.25 s) are
unchanged, id 0 is a real row (name "None", stack 0 per `Item.cs:48685-48688`).
`ItemVariants` (`GameContent/Items/ItemVariants.cs`) is ported in
`tr-app/src/variants.rs`: 5 variants × 4 world-flag conditions (remix wired from
`tr-core/src/wind.rs:97`; getGood/Mechdusa/skyblock default false), `select`
first-met, `apply` over the generated rows (22 `==` deltas + 3 `!=` restores); no
production call-sites yet — M3-ITEM-2+ adopt the seam. All `Item_<id>` icons load
at startup (Phase 4).

Deliberate deviations recorded:
- **Named skips** (`gen.py` `THIS_SKIP_FIELDS`): sound (`UseSound` bank/style),
  color, glowMask, stringColor, dye/hairDye (M3-DYES-1 owns the shader registry),
  mountType, tileWand, fishingPole, makeNPC, lifeRegen, quest/expert/shop flags,
  vanity/noWet/mech/noMelee/noUseGraphic/paint/flame, and the equip-slot tail
  (wing/hand/back/waist/face/balloon/neck/shield/front/beard/voice) are parsed but
  not provisioned — each skip carries a reason string; nothing is dropped silently.
  Consequence for the melee arm (BUG-09): `ItemDef` has no `noMelee`, so the
  vanilla's `damage > 0 && !noMelee` melee-hit gate (`Player.cs:43648`) is
  approximated by the editing path's `held_use_ammo == 0` — the ammo shooters are
  refused, but a non-ammo projectile weapon (a magic staff, the Terra Blade, a
  boomerang — all `noMelee = true` in `Item.SetDefaults`) can still swing its own
  melee damage at an NPC, which the vanilla refuses. Observed (throwaway rig, not
  a test): the Wooden Boomerang (284) takes 9 off a defence-2 blue slime on its
  first swing, because in the port's rig the throw is press-edge gated and the
  melee arm does not wait for it; the Space Gun (127) escapes it only because the
  `gunProj` id-keyed table gives it a `useAmmo`, so the `held_use_ammo == 0` gate
  refuses it. M3-ITEM owns the field; do not approximate by excluding every
  `projectile` item, because the Enchanted Sword (989) melees *and* shoots in
  vanilla.
- **`Main.infectedSeed` conditional** (Item.cs:2180, id 66): the seed branch is not
  evaluated; the base value is kept, the site is recorded by the generator.
- **Slots are i16** (−1 = none): slot 0 is real (the Familiar set, ids 269-271);
  consumers adapted (`render/entities.rs`, `render/player_layers.rs`).
- **Wall ids are the faithful 1.4.5 numbering** (`WallID.Dirt = 16`,
  `WallID.cs:137`); v2 save migration keeps the legacy wall→item map (1→26, 2→30).
- **`ProjectileID.Shuriken` const corrected 1→3** (`ProjectileID.cs:560`).
- **Names** are PascalCase-split `ItemID` identifiers ("Dirt Block"); localized
  display names stay out of scope.

## Main menu and multiplayer (M6)

`menu/mod.rs` draws the original-style title screen (Logo texture, Mouse_Text buttons,
random character name from the game's name syllables) with Singleplayer, Multiplayer
(Host on a port with a password, Join by IP + port + password) and Quit. Screenshots and
debug hooks skip the menu; `TR_MENU=1` forces it for captures.

`net.rs` implements a small length-prefixed TCP protocol (version 1): the host owns the
world, clock, NPCs and drops and broadcasts them at 10 Hz; each client simulates its own
player and sends its state at 10 Hz. On join the host sends a zlib-compressed `TRWLD`
world plus the clock; player joins/leaves, player states, tile edits, NPC snapshots, drop
snapshots and time are replicated. Edits are optimistic on the client and re-applied
idempotently when the host echoes them. The host binds `0.0.0.0` so LAN and internet
clients can connect by IP.

Because a joined world can use different tiles, receiving it saves `Remote.trwld` and
relaunches the executable (`TR_JOIN`/`TR_JOINED`), letting every world-derived system
(atlas, liquids, lighting) rebuild without tearing down the renderer mid-frame. Known
v1 limits: clients do not simulate NPCs (host snapshots only, no interpolation), liquid
flow is simulated locally on both ends, and drops are host-authoritative with an
optimistic client pickup.

## Minimap and full map (M7-MAP)

The map is a per-tile `MapTile` array (one entry per world cell, row-major
`y * width + x`, the same layout as `World::index`), not the old 8x8 averaged
blocks. Colours are the original `MapHelper` lookup tables
(`tr-core/src/mapdata.rs`, generated from `MapHelper.cs` by
`tools/gen_map_colors.py`), so each tile/wall shows a single flat colour;
`MapTile` also carries the `GetTileBaseOption` type index, the per-tile light
byte and the liquid kind. A tile is revealed by reading the light map
(`WorldMap.UpdateLighting` / `LightingEngine.ExportToMiniMap`): a 75x47 window
around the player four times a second (`map.rs::reveal_rect`), written into a
world-sized RGBA texture (one texel per tile) with `SpriteRenderer::
write_map_region`. The texture is drawn at `Main.DrawMap`'s geometry: the
fullscreen map (`M`, `Main.mapFullscreenScale` = 2.5 opened, wheel or key zoom,
WASD pan, clamped to 10/10/`maxTilesX-10`/`maxTilesY-19`, recentred on the view
centre) and the minimap (top right, 240x240 drawn panel, 1.05 px/tile, centred
on the view centre). Unexplored tiles stay transparent.

Behind the fullscreen map, `map_bg_index` ports
`Main.DrawMapFullscreenBackground` (`Main.cs:56342-56444`) to pick one of the
`Images/MapBG{index+1}` sheets (whitelisted in `assets.rs`), stretched over the
screen with the sky tint. The ported-later branch order is kept, but only the
zones this engine can evaluate are wired: the Lihzahrd wall test (WallID 87 =
`LihzahrdBrickUnsafe`, index 13/`MapBG14`), the deepest layer, the underground
wall chain (Hive 86/108, Granite 180/184, Marble 178/183, Spider 62/263), the
surface chain (snow/desert/beach/forest/space) and snow/desert/rock. Branches
whose zone we cannot evaluate take the unchanged `false` path and are **not**
implemented: `ZoneDungeon`, `ZoneGlowshroom`, `ZoneCorrupt`, `ZoneCrimson`,
`ZoneHallow`, `ZoneJungle`, `bloodMoon` (no doubling), `ZoneGraveyard`. Note that `Images/Map` (the wooden scroll frame) is
97 % opaque with a near-black interior, so at the default `mapFullscreenScale`
it covers the `MapBG` backdrop; the backdrop is only visible on screens where the
frame is smaller than the screen.

The player head icon is the Rust stand-in for vanilla's 84x84
`MapHeadRenderer`/`PlayerHeadDrawRenderTargetContent` render target (this engine
has no render targets): `map.rs::head_icon_pos` ports the icon position
(`Main.cs:55937-55944`) and `render/entities.rs::draw_player_map_icon` draws the head, eye
whites, eyes and hair (frame row 0, unlit — `MapHeadRenderer.cs:50` blits the
target with `Color.White`) plus the outline rings (black at Manhattan radius 4,
`Main.teamColor[0]` = white at radius 2) into the screen-space `ui` range, so it
lands on top of the map like vanilla's frame -> tiles -> head order
(`Main.cs:55485` -> `55609` -> `55954`). `Main.UIScale` is passed as `1.0`
(no UI-scale system here). The HUD is hidden while the fullscreen map is open,
matching vanilla's early return in `Main.Draw` (`Main.cs:62147-62181`, before
`DrawInterface` at `62432`). While the map is open the world input is modal
(`Player.cs:24798-24807`): movement and use/tile controls are cleared, so nothing
is mined, placed or attacked behind it, the middle-button camera drag is inert,
and the left button drags the map by the cursor delta (`Main.cs:55342-55359`).

Persistence: `tr-core/src/map/file.rs` owns the `.map` v326 byte layout (header,
option tables, RLE body) and `tr-app/src/map.rs` the raw-deflate wrapping and
file IO (`save_map_file`/`load_map_file`). The sidecar is `<world>.map` next to
the `.trwld` (`persist::map_path`), written whenever the world/player is saved —
F9, window close and the 10-minute autosave (`Main.cs:19757`:
`saveTime.ElapsedMilliseconds > 600000`) — and loaded at startup through
`WorldMap::load`, which arms one bulk texture upload. `Main.mapEnabled`
(`config.json`) gates both, and the `M` toggle (`Player.cs:29105`). Deviations
from `MapHelper`: no edge clearing
(`ClearEdges`), no cloud/team-icon map extras (`DrawMap_GetMapObjectName` rows,
pins), no `worldID` validation on load (seed + size are checked), the heat/snow
`sceneSnowiness` input is fixed at 0 and `invisible_block` wall flags are not
modelled (our 8-bit option byte cannot hold them). `MapBG`'s snow/desert branches
use the shared camera-centred `is_snow_biome`/`is_desert_biome` heuristics rather
than the player's `ZoneSnow`/`ZoneDesert` flags. The encoder closes a light run
that reaches a row's end with a full run (`num6 = 0`, `MapHelper.cs:3214-3225`,
is written as the run's length), so the port's own decoder round-trips it; a
byte-level vanilla `LoadMapVersion2` would misread that one case.

## Localization (M6)

`tr-core/src/localization/` ports `Terraria.Localization`'s core:
`GameCulture` (the twelve shipped cultures, their `en-US`-style names and legacy ids,
`FromName`/`FromLegacyId`/`FromCultureName` with the original fallbacks), `LocalizedText`
(`Value`/`UnformattedValue`/`EnglishValue`/`HasValue`/`Format`), the internal `VariableText`
and `LanguageManager` (`SetLanguage`/`ReloadLanguage`/`LoadLanguage`, the per-category key
lists, `$`-variants, `GetText`/`GetTextValue`, `Random`/`IndexedFromCategory`, `FindAll`).

Strings load from the same JSON the game embeds as
`Terraria.Localization.Content.<culture>.*.json` (a `Dictionary<string, Dictionary<string,
string>>`). Register each resource with `add_source(name, text)` and call `set_language`:
the loader mirrors `LoadLanguageFromFileTextJson`, creating categories only where
`canCreateCategories` is set, skipping the `.csv` path (mods only). Two compatibility
notes: the shipped files carry Newtonsoft-style trailing commas, so the parser strips a
comma that directly precedes a closing brace/bracket outside string literals; and
`ActiveCulture` starts unset (null in the original), so the first `set_language` always
loads even for English.

`VariableText` reproduces `{Name}` substitution and the `{?Name}`/`{!Name}` conditions via
a `Substitution` table (`set_substitution`/`remove_substitution`) that stands in for
`Lang.GetGlobalSubstitution`; a missing condition or variable leaves the raw text, as the
original returns `_value.ToString()`. `{$Category.Key}` copy commands expand to the
referenced `UnformattedValue`, iterated to a fixed point (100 passes, like the original).
`format_net` implements the `string.Format` holes the content uses (`{0}`, `{{`/`}}`), and
keeps an out-of-range token instead of throwing so a bad language file cannot crash a frame.
A test loads the seven shipped `en-US` files and checks `ItemName.DirtBlock`,
`CommonItemTooltip.UsesMana` and the `ItemName` category size (>1000). Wiring the strings
into the HUD, item names and menus is still to do.

## Audio (M6)

### XACT banks (`tr-content/xact.rs`)

SFX come from `Sounds/*.xnb` (`SoundEffectReader`, PCM) and music from `Wave Bank.xwb`
(MS-ADPCM) addressed through `Sound Bank.xsb`. The wave bank starts with a five-segment
header; `segments[0]` is `BANKDATA` (`flags`, `entryCount`, `name[64]`, `metaElemSize`,
`nameElemSize`, `alignment`), `segments[1]` is `ENTRYMETADATA` and `segments[4]` is
`ENTRYWAVEDATA`. Each metadata entry is 24 bytes: `flagsAndDuration`, a mini-format
(`codec`, `channels`, `sampleRate`, `blockAlign`, `samplesPerBlock`), the play region
(`offset`, `length`) and the loop region (`start`, `total`). The mini-format's codec tag
`2` is MS-ADPCM; the on-disk `blockAlign` field packs `(field + 22) * channels`, so stereo
music has 140-byte blocks and every play length is a whole number of blocks. The decoder
lives in `adpcm.rs`. Stereo blocks use a **field-interleaved** header
(`pL pR | dL dR | s1L s1R | s2L s2R`, 14 bytes), *not* two contiguous 7-byte channel
headers; each payload byte carries the left nibble in the high half and the right in the
low half, and the first two output frames are `sample2` then `sample1`. Reading the header
per channel seeds every 128-frame block with garbage (audible as broken, crackling music).
The port matches libsndfile's `msadpcm_decode_block` bit-for-bit on retail `Music_1`.
Tests cover a synthetic block header, per-channel smoothness and block-boundary continuity,
and decode `Music_1` through the bank.

The sound bank header stores the counts at `0x13`/`0x15`/`0x19`/`0x1C` and the section
offsets at `0x22` (simple cues), `0x42` (name table) and `0x4A` (bank name). A simple cue
is 5 bytes (`flags` + `soundOffset`), the name table entry is 6 bytes (`nameOffset` u32 +
`hash` u16), and the sound record holds the wave-bank track at `+0x09`. The cue → track
mapping is **not** `Music_id -> id-1` (e.g. `Music_1 -> 2`, `Music_3 -> 0`), so the sound
bank must be read to address the wave bank.

### Music engine (`tr-app/audio.rs`)

`MusicSystem` mirrors `Main.UpdateAudio` and `LegacyAudioSystem.UpdateCommonTrack`: one
rodio `Sink` per `AudioTrack`, a `musicFade` array of `MusicID.Count` (`105`) entries that
ramps by `±0.005` per frame, `volume = fade * musicVolume` with the `62..=88 ×0.9` and
`52 ×1.15` (clamped to 1) special cases, and `newMusic`/`curMusic` selection. The ported
branches are boss (King Slime → 93), underground (`ugMusic` alternates 4/31, `Main.cs:12830`),
rain (day 59/19, night 19) and surface day/night (1/18, 3). The biome branches still need
`SceneMetrics`. A track starts when its fade rises and stops when another track is audible
(`musicFade[curMusic] > 0.25`) and its own fade reaches 0; `curMusic == 0` snaps every other
fade to 0. Decoding is synchronous on track start (rare, ~5.6 MB of PCM per track, not
cached yet).

### 3D attenuation (`tr-app/sound.rs`)

`play_sound` ports `LegacySoundPlayer.PlaySound` (`LegacySoundPlayer.cs:360`): the sentinel
`x == -1 || y == -1` plays 2D, otherwise the distance to `Main.Camera.Center` must be under
`SoundAttenuationDistance = 2500` px, `volume = 1 - distance/2500`, and
`pan = dx / (MaxWorldViewSize.X/2)` clamped to `±1`. The final gain follows
`Main.soundVolume` (or `Main.ambientVolume` for the ambience sound ids, muted in the menu)
and is multiplied by the caller's `volumeScale`. `audio.rs` applies the pan to a rodio
source (the far channel is attenuated); the listener is refreshed each frame from the
camera centre and the three volume sliders.

### Ambience emitters (`tr-app/ambient_sound.rs`, M6-AUDIO-1)

The ambience *audio* of 1.4.5.8 is five emitters; `Main.Ambience` (the waterfall/lavafall
loops, M1.4-BG-20) and the rain/wind music-pool loops were already in main, so this task
ported the remaining three and verified the fourth:

* **Blizzard loops** (`SceneState.cs:189-253`). The two tracked global loops resume on
  `ZoneRain && ZoneSnow` — the raw zone pair, not the `UseStormEffects`-gated visual — and
  cross-fade through `_blizzardSoundVolume` (`MoveTowards ±0.01` per `SceneState.Update`,
  so it lives on the port's `SceneState` beside the other fades, not on a tick) split
  `min(1, cloudAlpha*2) * outsideWeatherEffectIntensity`. `audio.rs::set_ambient_loop` is
  the `ActiveSound.DetermineIntendedVolume` engine half: global → no attenuation,
  `clamp(Style.Volume × Volume × ambientVolume)`, `SoundID.cs:1598-1599`'s 0.5/1 style
  volumes. The sink is created on the first non-zero volume and then kept (muted at 0) —
  the C# `SlotId` policy.
* **Thunder strike** (`Main.cs:58747-58766`). The cue plays from the weather tick with the
  C# position math and the `case 43` family (`Thunder_0..5`) from Rain's own stream. Two
  documented gaps: the `case 43` re-roll-on-playing loop has no port model (no per-slot
  instance bookkeeping), and the pitch jitter is consumed but not applied (the rodio
  sources have no pitch control — the same gap as the BG-20 liquid loops).
* **Critter sounds** (`NPC.cs:92244-92340`) and **liquid-change sounds**
  (`WorldGen.cs:4632-4650`) land as pure, tested tables. Their call sites need the critter
  NPC kinds (M4-NPC — the roster has none today) and the liquid merge pass
  (`Liquid.CreateLiquidMergeTile`, `Liquid.cs:1324` — `tr-core/liquid.rs` ports the settle
  but not the merge tiles), so they run constants-off with citations, the M6-MUSIC-1
  pattern.

Deviations (deliberate):

* The tracked-sound layer (`ActiveSound`/`SlotId`/`SoundPlayer.PlayTrackedSound`) is not
  modelled; each emitter owns its sink or cue directly. The observable behaviour (start
  conditions, volume ramp, split, stop) matches; the C# GC of a dead slot has nothing to
  map to (a sink never dies in the port).
* The thunder/pitch and critter rolls come from the owning system's stream (the
  WEATHER-1/WEATHER-2 per-consumer convention), not the shared `Main.rand` the C# draws.
* `FocusHelper.UpdateBackgroundThunder`'s unfocused half and `StopAmbientSounds`
  (`Main.cs:11040-11045`) stay as BG-20 pinned them (the port always claims focus).
* The blizzard emitter's menu behaviour is the C#'s: `SceneState.Update` does not run in
  the menu, so the loops hold their last volume (the vanilla instance also keeps playing).

## In-game interface (M7-UI)

The in-game HUD and inventory are immediate-mode in the original (`Main.DrawInterface`,
`Main.DrawInventory`), so the port keeps one Rust function per original function in
`crates/tr-app/src/ui/` with the original's constants and pass order (see `docs/UI.md`).
The deviations below are deliberate and are the places the port is **not** pixel-exact.

### Resources (`ui/resources.rs`, `ClassicPlayerResourcesDisplaySet.cs`)

* `Player.statLifeMax`/`statLifeMax2` are collapsed into a single `Player.max_hp`, so
  the bonus-heart branch (`ClassicPlayerResourcesDisplaySet.cs:112-116`, only reached
  when `statLifeMax2 > statLifeMax > 400`) is unreachable and **not ported**. A fresh
  character has `max_hp = 100`, where the branch cannot trigger; the `Heart2` sheet is
  therefore never drawn.
* `Player.ghost` (`:39-42`) is not modelled, so a dead player still draws hearts.
* Breath bubbles are drawn at the heart row without the source's `+32` y offset and use
  the per-bubble alpha directly rather than `num2` (`:223-241`); the bubble count is
  correct.
* `Main.statLifeMax`-driven heart *count* and the per-heart scale/alpha/`Main.cursorScale`
  pulse are ported exactly.

### Cursor and mouse text (`ui/cursor.rs`, `ui/tooltip.rs`)

* `Main.ThickMouse` / `Main.mouseItem`'s `+34` tooltip offsets (`Main.cs:20162`) are not
  modelled; the tooltip is anchored at the pointer.
* Tooltip stat lines are placeholders (zeros) until `Item.ToolTip`'s data-driven lines
  are ported; the 9-slice box, the rarity colour and the 4-direction
  `DrawColorCodedStringWithPadding` shadow are exact. `rare` values below the ported
  table (`-12`, `-13`, `diff` colours) fall back to white.

### Inventory (`ui/inventory.rs`, `ui/chrome.rs`, `ui/slot.rs`)

* Vanilla keeps one 59-slot `Player.inventory` plus the `armor`/`dye`/`miscEquips`/
  `miscDyes` arrays. The port keeps the 50-slot grid in `App::inventory` (so the saved
  format in `persist.rs` is unchanged) and the rest in `inv::EquipSlots`.
* `Main.invAlpha` pulses `180..=240` at `±0.2`/frame (`Main.cs:45697-45708`) and tints
  every slot background; the port uses the fixed `220` (`Main.cs:67274`) so screenshots
  stay deterministic. The equipment columns keep the **loadout-1** colours
  (`ItemSlot.LoadoutSlotColors[0]`, `ItemSlot.cs:365-371`) since loadouts 2/3 are not
  modelled.
* The page tabs, defence counter, guide text and PVP icons are drawn but not
  interactive (`ItemSlot.Handle` is out of scope for this milestone), and
  `Player.statDefense` is not modelled, so the defence counter always shows `0`.
* The bestiary/emote/achievement shortcut buttons and the NPC-housing page
  (`EquipPage 1`) are not drawn yet.
* `Main.inventoryBack`-tinted labels use white instead of the pulsing
  `Main.mouseTextColor` (`ui/inventory.rs` `LABEL_COLOR`).
* The reforge slot's menu opened with M3-REFORGE-1 — the (50,270) slot, the
  (120,310) button and the cost label ride the text-panel fidelity at the
  vanilla coordinates (`ui/mod.rs` + `clicks.rs`; the Reforge[0]/[1] button
  textures and the coin breakdown stay out). The guide/crafting slots keep
  their empty frames.

### Closed hotbar and mouse item (`ui/mod.rs`, `ui/slot.rs`)

Both draw through the same `slot::draw_slot` (the `ItemSlot.Draw` port) as the open
grid: the closed hotbar at context 13 (`GUIHotbarDrawInner`, `Main.cs:43600-43639`) and
the cursor stack at context 21 (`DrawInterface_38_MouseCarriedObject`,
`Main.cs:45173-45195`). Remaining deliberate gaps:

* The context-13 **ammo/wand/bait count** (`num22`, `ItemSlot.cs:2577-2691` — the
  counter that replaces the stack text on bows, wands and fishing poles) is not
  ported; it needs the `useAmmo`/`bait`/`tileWand` inventory scans the item model
  does not have yet. Such items show their plain stack instead.
* `Inventory_Back20` (`ItemSlot.cs:2234-2237`, the white variant for
  `slot == player.selectedItemState.Hotbar`) and the context-36 override slot
  (`Main.cs:43641-43649`) are unreachable without `Player.SelectedItemState`
  overrides, which the port does not model; the selected slot always merges into the
  `Inventory_Back14` branch.
* The `Main.DoGlowingMouseItemDraw` shader pass over the cursor item
  (`Main.cs:45196-45220`) is not ported.
* `hotbarScale` keeps lerping while the inventory is open; vanilla freezes it because
  `GUIHotbarDrawInner` early-returns on `Main.playerInventory`
  (`Main.cs:43588-43591`) and the update lives in that draw pass. The targets
  (1.0 selected / 0.75 rest) and the `±0.05` rate match (`Main.cs:43604-43611`).

### Crafting panel (`ui/crafting.rs`, `NewCraftingUI.cs`)

The modern crafting panel is ported as draw-only chrome at the source's geometry and
z-order (`docs/UI.md` → "Crafting panel"). The deviations:

* The detail area (craft slot + `Images/UI/Craft` arrow) is drawn without a selected
  recipe; vanilla draws it only inside the
  `recipeEntry = _hoveredEntry ?? SelectedEntry` block (`NewCraftingUI.cs:447-448`,
  the `:448-482` range). With `Recipe.maxRecipes == 0` the port always renders it as
  static chrome.
* The panel ships open with the inventory (`NewCraftingUI.Visible`,
  `Main.cs:41706-41709`); `TR_NOCRAFT=1` hides it and `TR_CRAFT=1` forces it on
  (mirroring `TR_INV`).
* No filter, search or scrollbar interaction: the panel is draw-only, without
  `ItemSlot.Handle` (out of scope, as documented for the inventory panels above).
* The cancel button is always visible — `White * 0.4` at rest, `White` on hover.
  Vanilla's `UIImageButton` never hides the icon (`_visibilityInactive = 0.4f`,
  `UIImageButton.cs:13-15`); the port's plan wrongly assumed it hid while the search
  box was empty.
* The `Main.InGuideCraftMenu` variants (guide slot, "Place a material here" label,
  hidden nearby-chests toggle) are ported behind `crafting::draw`'s `guide_mode`
  parameter but never reached: the guide slot cannot be opened in the port yet, so
  the menu is never entered and `guide_mode` stays false.

### Minimap frame and boss bar (P4) (`ui/minimap.rs`, `ui/bars.rs`, `ui/chrome.rs`)

The minimap frame (`MinimapFrame`/`MinimapFrameManager`/`MinimapFrameTemplate`)
and the boss bar (`BigProgressBarHelper.DrawFancyBar`) are ported at the source
geometry (`docs/UI.md` → "Minimap frame" / "Boss bar"). The deviations and
behaviour notes:

* `Main.MapScale` is **live** as of M6-SETTINGS P4 (before that it was pinned to
  the reference `1.0`): the minimap anchor and the whole panel's render scale
  read the `MapScale` config key (load-clamped `0.5..1.0`, `Main.cs:4494`), and
  the port stands in for `Matrix.CreateScale(Main.MapScale)`
  (`Main.cs:55308-55329`, foreground batch at `:55768`) by keeping
  `ui::minimap`'s geometry in the source's transform space — the origin divided
  by the scale (`:55510-55511`), the pointer too (`:55512-55513`) — and
  multiplying back inside the two draw passes. The full map keeps
  `Main.mapFullscreenScale` (`Main.cs:55390`, default 4f `:66756`) and the zoom
  axis `Main.mapMinimapScale` (`Main.cs:55515-55522`) is unaffected. The
  Interface slider itself is still a P5 hand-off — the key is read from
  `config.json` only.
* No UI cue plays: the buttons' hover/click sounds `SoundEngine.PlaySound(12)`
  (`MinimapFrame.cs:167,174`) and the icon row's `PlaySound(10)/(12)`
  (`Main.cs:41100,41109-41119`) are not ported — no UI cue is loaded
  (`audio.rs:22-31`), consistent with the silent menus. `Main.DoStatefulTickSound`
  (`:41126`) is not ported either.
* `mapStyle == 2` (the overlay style) draws nothing in the minimap branch —
  the whole-UI P4 plan's Uncertainty 7: only the style-1 branch
  (`Main.cs:55503-55547`) is ported, and the style-2 overlay render has no
  Rust counterpart yet.
* The minimap **content** icons are not ported: NPC map icons, player heads,
  revenge/death markers (`TextureAssets.MapDeath`) and map pings
  (`TextureAssets.MapPing`) — the render entry points of research §F
  (`geometry-extraction.md`, `Main.cs:55619-55680`, `PingMapLayer.cs:35`).
  The frame chrome draws around the bare tile pass; a future `MapIcons` task
  owns them.
* The shielded 7-arg `DrawFancyBar` overload (`BigProgressBarHelper.cs:72-126`)
  and `DrawBareBonesBar` (`:11-23`) are not ported — the former has no
  reachable caller in this port, the latter is dead code.
* `BigProgressBarSystem.ShowText` is **live** as of M6-SETTINGS P4 (before that
  it was always on): `boss_bar_text` feeds `ui::bars::draw_fancy_bar`, which
  skips `DrawHealthText` when the flag is off (`BigProgressBarHelper.cs:53-55`)
  — the bar, the frame and the head icon keep drawing, only the `current / max`
  numbers disappear.
* The map-style icon row draws without the `Main.mapEnabled` gate
  (`:41056`) while its clicks keep the gate (the documented draw/click
  asymmetry in `chrome.rs`/`app/clicks.rs`): the row is visible with the map disabled,
  but only clicks when the map system is enabled. Visible only with
  `MapEnabled=false`.
* The icon-row clicks fire on the press frame; vanilla tests
  `Main.mouseLeft && Main.mouseLeftRelease` (`:41095`), i.e. on the release
  frame. One shot either way (the port's pre-existing press convention).
* Hovering a minimap zoom button suppresses mine/place/attack inputs — the
  port mirrors `MinimapFrame.Update`'s `mouseInterface = true`
  (`MinimapFrame.cs:164`) through its `minimap_hover` flag. Listed because it
  changes input behaviour; it is faithful, not a gap.

### Determinism hooks

`TR_MOUSE=x,y` (pin the (UI-space) pointer), `TR_PAGE=name` (open a menu screen
directly — the word aliases `settings|general|interface|video|multiplayer` and the
numeric `menuMode` of every P5 sub-screen: 26, 28, 25, 252, 1125, 111, 2008, 1212,
1213, 1127, 27, 272727, 12, 13, 131, 30, 31, 889), `TR_MENU=1` (keep the menu on
screen), `TR_INV=1` (open the inventory),
`TR_CRAFT=1`/`TR_NOCRAFT=1` (force the crafting panel on/off), `TR_PVP=1`
(open `DrawPVPIcons`'s multiplayer-only draw/click gates),
`TR_EQUIPPAGE=n` (select `Main.EquipPage`), `TR_MAPSTYLE=0|1|2` (select
`Main.mapStyle`; invalid/missing → 1, `Main.cs:66750`), `TR_BOSS` (spawn King
Slime — the boss bar's input), `TR_NOHUD=1` (hide the HUD, hotbar and minimap),
`TR_NOPLAYER=1`, `TR_NO_BG=1`, `TR_DAY=f`, `TR_SHOT=path`. Screenshots are
byte-identical across runs with `TR_NOPLAYER=1 TR_NO_BG=1 TR_DAY=0.5`.

## Menu sub-screens (P5) (`menu/`, `ui/sliders.rs`)

The seventeen `Main.DrawMenu` settings branches below menuMode 11 (26, 28, 25,
252, 1125, 111, 2008, 1212, 1213, 1127, 27, 272727, 13, 131, 30, 31, 889) and the
multiplayer branch (12) are ported as visual-only pages at the source geometry
(`docs/UI.md` → "Menu sub-screens (P5)"); `IngameOptions.DrawValueBar` and the
HSL colour-slider block live in `ui/sliders.rs`. Deviations and behaviour notes:

* **Config write timing.** Vanilla saves `config.json` when the menu closes;
  the port writes it immediately on every toggle and once per drag on slider
  release (the port's write-per-edit convention from the settings-persistence
  milestone) — same keys and values, different moments.
* `parallax_scroll` defaults to **60** with a fresh config, not 0: vanilla seeds
  `Main.caveParallax = 0.88f` (`Main.cs:66809`) and `LoadSettings`'s keyless
  `Get("BackgroundParallax", ref bgScroll)` derives `bgScroll = 60` from it
  (`:4405-4408`), so the screen shows "Parallax: 60" and the backdrop applies
  the 0.88 factor — matching vanilla's out-of-box look.
* **The colour screens** differ in exactly two flags: menuMode 25 clamps
  luminosity to `[0.15, 1]` and hides the alpha bar, 252 uses the full range and
  shows it (`Main.cs:51239-51276`). Each frame the branch copies `selColor` into
  the page's colour (`:50324` / `:50346`) before the sliders seed/drag/recompose
  it, so the stored colour trails the knob by one frame — the original's order.
  The cursor tint reads the stored colour per frame (`app/frame.rs`, before
  `cursor.update()`), so dragging re-colours the pointer live (`Main.cs:50324` →
  `:52490`).
* **Lock-on priority cycles 0 → 1 → 2 → 1**, not back to 0: `CycleUseModes`
  ThreeDS→TargetClosest (`LockOnHelper.cs:103-116`) skips `FocusTarget`, which is
  unreachable by cycling (a fresh `LoadSettings` also rewrites 0 to 2,
  `Main.cs:4496-4505`).
* **Multiplayer transitions are cut down to the menu.** Without a character/world
  select (P6): Multiplayer "Join"/"Host & Play" open the input screens directly
  instead of the character select (`Main.cs:50713-50730`); menuMode 131 Accept
  opens the password screen (31) instead of connecting (`OnSubmitServerPort`,
  `:52047-52054`); the password screens' Back rows return to Join Port /
  Multiplayer instead of cancelling the connect / the world select
  (`:51950-51957`, `ExitServerPasswordMenu :52018-52028`); 889 Back returns to
  Multiplayer instead of the world select (`:48804-48806`). The 889 "Allowed
  Users" rows stay blank — the Steam-only lobby branch (`:48749-48777`) never
  fills them without `SocialAPI.Network`. The port splits the host and join
  password strings where vanilla shares one `Netplay.ServerPassword`. Host keeps
  the vanilla fixed listen port 7777 (`HostAndPlay`, `:52011`); the old custom
  port row is gone (`TR_HOST` still overrides for tests).
* **M6-SETTINGS (config parity, 2026-11-09).** The settings layer now matches
  the source on all four axes:
  - **Defaults.** `Settings::default()` is the `static Main()` set:
    `musicVolume`/`ambientVolume`/`soundVolume` `1f` (`Main.cs:66857-66859`),
    `invasionProgressMode` `2` (`:67080`), `SettingPlayWhenUnfocused` `false`
    (`:66512`), `FrameSkipMode` `Subtle` (`:67204`) — the port's earlier
    `0.75`/`1`/`true`/`0` were plain parity breaks (the out-of-box screen
    labels and the first-run audio levels differed).
  - **Key names.** `to_json` writes the vanilla `SaveSettings` names —
    `ShowItemText` (`Main.cs:4135`), `GraphicsQuality` (`:4130`),
    `GoreVisualsAllowed` (`:4101`), `SettingsEnabled_MinersWobble` (`:4153`),
    `SettingsEnabled_TilesSwayInWind` (`:4157`),
    `GamepadDisableInstructionsDisplay` (`:4151`), `InvasionBarMode` (`:4096`),
    `VolumeMusic`/`VolumeSound`/`VolumeAmbient` (`:4105-4107`),
    `PlayWhenUnfocused` (`:4217`), `HidePasswords` (`:4144`) — and `parse`
    keeps every pre-parity port spelling as an alias, so a port-written config
    from before this pass still restores its values. The earlier
    `"HidePassword"`-singular note is superseded: the port now writes the
    vanilla plural and reads both spellings.
  - **The `LightingMode` ordinal.** The C# enum is
    `{White=0, Retro=1, Trippy=2, Color=3}`
    (`Terraria/Graphics/Light/LightMode.cs:3-9`) while the port's internal
    order is `{Color=0, White=1, Retro=2, Trippy=3}`
    (`tr-core/src/lighting/params.rs:12-17`), so `parse`/`to_json` translate in
    both directions and only the persisted byte is the vanilla ordinal
    (vanilla casts the stored int unchecked, `Main.cs:4398-4403`; an
    out-of-range byte keeps the default instead — a deliberate robustness
    deviation).
  - **Read-once.** `config.json` is read exactly once (`main.rs` →
    `App::new(settings, …)` → `Menu::new(settings, …)`, the menu holding a
    working clone); the four read sites the research listed (`main.rs`,
    `App::new`, `Menu::new` and `CursorState::default`) collapse to one, and the
    menu-close sync (`close_menu`/`park_menu` → `App.settings` +
    `apply_settings`) still carries the edited values back.
  - **Kept limitation:** a real vanilla `config.json`'s `MouseColor` /
    `MouseBorderColor` dictionaries are multi-line (Newtonsoft
    `Formatting.Indented`, `Preferences.cs:47-53`), which the port's line-based
    parser cannot read; the port writes one-line dicts itself, so its own
    round-trips work and a vanilla file's colours fall back to their defaults.
  - **Kept port-wide behaviour:** a malformed boolean coerces to `false`
    rather than vanilla's `Preferences.Get` keeping the field default
    (`Preferences.cs:176-190`); changing that is a parser-wide change, not a
    settings one, so it is recorded here instead.
* **Stored without consumers yet** (visual-only, spec §9.1): the volume sliders
  (no audio engine reads the volumes), the six Effects toggles / smart-cursor
  flags / `FlashyEffects*` (no gameplay consumer), `resource_sets` (the HUD
  always renders the Classic set — the default above) and
  `portrait_preference`. The Interface "Minimap Border" row is the exception —
  live, see below. `boss_bar_text` and `map_scale` left this list with the
  P4 consumers below.
  Flags whose *system* the port does not have at all stay stored-only, each with
  its own note (never a stand-in consumer — the P4 rule): a flag is either wired
  to the real system or written down here.
  - **`blood`** (`ChildSafety.Disabled`, key `GoreVisualsAllowed`, `Main.cs:4101`
    Put / `:4349` Get) is vanilla's gore gate — it also suppresses the dangerous
    dust (`Dust.cs:194`, `:445`) and the player-side blood
    (`Player.cs:38912`), with the gore check itself at `Gore.cs:1334`. The port
    has no gore system (only the ambient leaf gores of `render/foliage.rs`), so
    nothing exists for the flag to turn off: it stays stored-only until the gore
    port lands.
  - **`placement_preview`** (`Main.placementPreview`, key `PlacementPreview`,
    `Main.cs:4100`/`:4348`) drives the placement preview's opacity ramp —
    `Player.UpdatePlacementPreview` (`Player.cs:43402`) counts the frames the
    player has been moving and `GetPlacementPreviewOpacity`
    (`Player.cs:43389-43399`) remaps that count from full alpha to zero, which
    `TileObject.DrawPreview` draws (`TileDrawing.cs:740`). The port places tiles
    without drawing the tile-object preview, so the flag has no consumer yet.
  - **`mouse_border_color`** (`Main.MouseBorderColor`, key `MouseBorderColor`,
    `Main.cs:4179` Put / `:4462` Get, default `(64,64,64,64)` `:66514`) is the
    cursor's border tint: `Main.ThickMouse` (`Main.cs:2599`) thickens the
    smart-cursor outline with it and shifts the mouse-text block 6 px
    (`Main.cs:20203`; the other reader is `Utils.cs:2789`). The port draws
    neither a cursor border nor a smart-cursor outline, so the colour (like
    `mouse_color`, which the cursor tint does read) has no consumer.
  - **`smart_cursor_toggle`** (`Main.cSmartCursorModeIsToggleAndNotHold`, key
    `SmartCursorToggle`, `Main.cs:4094`/`:4330`, default `true` `:67178`) picks
    toggle vs hold for the smart cursor in `Player.TryToToggleSmartCursor`
    (`Player.cs:28814-28842`). The port has no `SmartCursorHelper` block
    selection, so there is no smart cursor to toggle.
  - **`smart_axe_first`** (`Player.SmartCursorSettings.SmartAxeAfterPickaxe`,
    `Player.cs:331`, key `UseSmartAxeAfterSmartPickaxe`, `Main.cs:4139`/`:4412`,
    default `false`) is the smart-cursor priority order — axe before pickaxe
    instead of after. Without the ported block selection there is no priority to
    order.
  - **`smart_blocks`** (`Player.SmartCursorSettings.SmartBlocksEnabled`,
    `Player.cs:330`, key `UseSmartCursorForCommonBlocks`, `Main.cs:4138`/`:4411`,
    default `false`) lets the smart cursor bridge to common blocks; it is the
    branch `SmartCursorHelper` takes when scanning for a target. The port has no
    such scan, so the flag is inert.
  All three smart-cursor flags share one missing system:
  `SmartCursorHelper` (`SmartCursorHelper.cs`) and its drawing half
  `Main.DrawSmartCursor` (`Main.cs:46609`) — the 16x16 outline around
  `Main.SmartCursorX/Y` plus the directional arrows (`:46650-46656`) — are not
  ported, so no consumer is honest until that task lands.
* **The menuMode 888 destinations stay hints**: the 1127 "Keybindings" row shows
  the not-ported notice because `ManageControlsMenu` is a `UIState` screen, out
  of P5 scope (spec §9.1) — the 27/272727 lists below cover the raw bind data.
* **The Video page carries a port-only "Zoom: …%" row** the original does not
  have (menuMode 1111 is Resolution, Parallax, Frame Skip, Lighting, Quality,
  Background, Blood and Gore, Miner's Wobble, Windy Environment, Effects, Back —
  11 rows, `Main.cs:49988-50096`); vanilla persists Zoom to `config.json`
  (`"Zoom"`, `Main.cs:4200`) but no `DrawMenu` row edits it — drop the row in a
  follow-up (or P6).
* The Settings "Language" row label is vanilla `Lang.menu[103]` and the screen
  it opens here is the 1213 list (`Main.cs:49759-49764`) — noted for the P6
  phase.

### Task 5 notes (C# facts that override the plan tables)

* `menuMode` 27 (Keybindings) and 272727 (Map Keys) are **unreachable
  upstream**: no `menuMode = 27` / `= 272727` assignment exists anywhere under
  `decompiled/` — the Settings "Controls" row opens 1127, whose "Keybindings"
  row opens the menuMode 888 `ManageControlsMenu` UIState
  (`Main.cs:50661-50666`, out of scope). Both list screens are ported for
  completeness and are reachable only via `TR_PAGE=27` / `TR_PAGE=272727`; no
  navigation leads to them.
* `Reset Keybindings` (`Main.ResetKeyBindings`, `Main.cs:52459-52475`) is only
  called by the mode-27 Reset row (`Main.cs:50561`); it restores
  `W S A D Space T Escape H J B E LeftShift LeftControl R`
  (`:52461-52474`). The 272727 Reset row inlines the map seeds
  `Tab M Add Subtract PageUp PageDown` (`:50451-50456` = the
  `:67190-67195` defaults). 1127 has no Reset row. The port keeps the binds
  session-only (vanilla saves them in the player file, not `config.json`);
  captured keys store winit debug names (`KeyW`) where XNA would store `W`.
* C# facts that override the plan table (plan arithmetic/reading slips, C# is
  truth):
  - 1127 Back draw y = `250 + 52*3 + 18` = **424**, not the plan's 464.
  - 272727 Back keeps **scale 1.0**: the branch assigns `array7[6] = 0.8f`
    twice (`Main.cs:50439-50440`) and never touches `array7[7]`
    (`:50441-50444`), unlike the mode-27 twin's 0.8 (`:50550`).
  - `SelectionHolder.CycleSelection` (`SelectionHolder.cs:44-58`) activates
    the option **before** the active one, wrapping the first to the last —
    the Interface selector cycles run *backwards* (Default → Valkyrie → … →
    Golden → Default; resource sets: Default → New → NewWithText → … →
    Default). The plan's "next-in-order" and its "cycling from Valkyrie lands
    on Default" oracle were both wrong; from Valkyrie the cycle lands on
    Retro.
  - `BigProgressBarSystem.ShowText` defaults **true** (static ctor,
    `BigProgressBarSystem.cs:259`), so `TR_PAGE=interface` shows "Boss Health
    Bar Numbers: On" by default, not the plan's "Off". The flag is live:
    `boss_bar_text` reaches `ui::bars::draw_fancy_bar`, which skips
    `DrawHealthText` when it is off (`BigProgressBarHelper.cs:53-55`).
* `resource_sets` defaults to `"Default"` (label "Classic") while vanilla's
  config default is `"New"` (`PlayerResourceSetsManager.cs:28`) — the planned
  deviation: the port renders the Classic set, so the row must agree with the
  HUD. A vanilla `config.json` value still round-trips unchanged.
* The Interface "Minimap Border" row is *live* behaviour, not visual-only:
  cycling rewrites `settings.minimap_frame`, which the game loop feeds
  through `ui::minimap::style_index` (`app/clicks.rs`), so the minimap frame
  re-renders immediately. Map Scale is the other live row: `map_scale` feeds
  the minimap's anchor (`52 + (int)(240 * MapScale)`, `Main.cs:55275-55276`) and
  the whole panel's render scale, through the port's stand-in for
  `Matrix.CreateScale(Main.MapScale)` (`Main.cs:55309-55329`): `ui::minimap`
  keeps its geometry in the source's transform space (the origin divided by the
  scale, `Main.cs:55510-55511`, the pointer likewise `:55512-55513`) and the two
  draw passes multiply back. The full map is untouched — it scales by
  `Main.mapFullscreenScale` (`Main.cs:55390`, default 4f `:66756`) — and so is
  the `mapMinimapScale` zoom axis (`Main.cs:55515-55522`).

## Front-end screens (P6) (`ui/`, `app/flow.rs`)

The whole-UI P6 front end (character/world select + create, the virtual
keyboard, credits — commits `02182dc..043e11c`) is ported at the source
geometry; the screens, coordinates and flow are in `docs/UI.md` →
"Front-end screens (P6)". The deviations and stand-ins:

* **No `.plr` reader/writer** (spec §9.4). Characters are the implicit
  session entry plus optional `TR_FAKE_CHARS` fakes; Create records the
  session draft instead of writing a character file. The `.plr` directory
  scan stays so a future reader slots into the same shape (`Main.cs:4007`);
  a scanned `.plr` is skipped like `Player.GetFileData`'s null return.
* **World metadata stand-ins.** Favourites are a `<world>.trfav` marker file
  (`Main.LocalFavoriteData` has no port); creation time is the file's mtime;
  there is no GameMode field (every entry reads Classic, so
  `sort_worlds`' `CanWorldBeJoinedByActivePlayer` Journey gate is vacuous);
  the world card icon is a fixed `IconCrimson` (no evil data to pick the
  sprite); the playtime panel is a fixed `00:00:00` (no play-time tracking).
  Invalid worlds bucket to Small (vanilla shows Unknown — no Unknown
  bucket); `erase_world` deletes the orphan `.map` (vanilla leaves it,
  plan-ordered); erases delete outright — no recycle bin
  (`FileUtilities.Delete`).
* **No cloud favourite buttons, `Rename` at card Left 52** —
  `SocialAPI.Cloud == null` in the port, exactly the branch the original
  takes without a cloud account.
* **No gamepad snap points** (the `SetSnapPoint` sites, e.g.
  `UIWorldListItem.cs:205`, have no port) and **no UI click sounds**: the
  `SoundEngine.PlaySound(10/11/12)` sites — the confirms' Yes/No included —
  stay silent, consistent with the P5 menus (`audio.rs:22-31`).
* **Rename caps** 20 (character) / 27 (world); the world-creator's
  `TrimDisplayIfOverElementDimensions(27)` display trim is not ported.
* **The keyboard draws the desktop layer only** (`ShowGamepadHints` false
  without a gamepad); the 50-key grid layer is kept behind `TR_VK_GRID=1`.
* **The Advanced seed tab is out of scope** (spec §9.4): the seed plate's
  click degrades to the plain seed keyboard and the special-seed ring effect
  is not drawn; `UIWorldLoad`/`UIWorldGenDebug`/`UIWorldCreationAdvanced`
  have no counterpart yet.
* **Credits** (`menuMode` 3000) is the world backdrop + the single Back row;
  the `SkyManager.Activate("CreditsRoll")` cinematic is not ported.
* **The legacy create flows** (menuModes 2/3/7/8/16/17-24/222/−7/−71/5000)
  and the colour-palette renderer (`Main.cs:51122-51450`) are not ported —
  dead from the modern menu; `Main.menuMode == 10` is a pass-through.
* **The confirms are ported as legacy pages** (5/9/40) whose Yes/No rows
  return `Action::{ConfirmYes, ConfirmNo}` — the App runs the erase and
  reopens the screen the confirm came from, instead of vanilla's inline
  `selectedMenu` branches (`Main.cs:49431/:49681/:49454`). Geometry note
  from review: the confirms are 250/80 (`Main.cs:48419/:48421`), not 220/52
  (220/52 is the mode-0 override, `:48911-48913`).
* **`GameTipsDisplay`** (`UICharacterCreation.cs:170`, the rotating hint
  line under the panel, `:1998-2005`) is not ported.
* **`assign_random_world_name` is a stand-in**: `AssignRandomWorldName`
  (`UIWorldCreation.cs:1002-1025`) composes a name from the
  `RandomWorldName_*` localization tables the port has not loaded; the
  stand-in is `"World {n+1}"` (`UIWorldSelect.cs:113`'s shape). A
  non-numeric world seed likewise hashes the name through the
  `ReLogic.Utilities.Crc32` port (poly 0xEDB88320) instead of `Main.rand` —
  deterministic per name, so the same draft always creates the same world.
* **`Color` multiplication wraps, not saturates** (`color.rs`'s pinned
  convention, re-pinned by the Task 8b review): `DarkGray * 1.2` =
  `[202, 202, 202, 50]` — the ghosted Master-difficulty bunny preview.

## UIElement framework (M7-UI-1, 2026-11-08)

The framework halves of `UIElement`/`UserInterface` that the screens cannot live
without — the pointer machine (`ui/pointer.rs`) and the flat hit order
(`ui/framework.rs`, `ui/select_hit.rs`) — landed on branch `M7-UI-1-uielement`
with the layout pass unchanged in `ui/uistate.rs`. The screen-model ruling stays:
an immediate port, no retained tree, no `UserInterface` object. Card double-click
play is the feature it buys, and it is **routing only**: the port has no per-card
selection state and its play path closes the front flow rather than loading the
entry (spec §9.4), so a double-click resolves to the same flow a Play button
click does.

Corrected deviations (both were geometry the layout pass would have got right,
and a third was the click gate the geometry feeds):

* **The state-change click lockout never expired (fix-world-select-clicks).**
  `App::enter_front` arms `UserInterface`'s 200 ms post-`SetState` disable
  (`ResetState`, `:178`, ported at `ui/pointer.rs:141`), but the *only* countdown
  site sat inside `frame()`'s `if self.menu.is_none() && self.front.is_none()`
  branch — the in-game regime — so `PointerState::tick` never ran while a front
  screen (`Main.menuMode == 888`) was live. `click_disabled_ms` therefore stayed
  at 200.0 and `clicks_disabled()` was permanently true, which dropped every
  press twice: once at `app/loop.rs:422` (no `PointerEvent` produced) and again
  at `handle_front_click` (`UserInterface.cs:214-218`, the gate the click tables
  run behind). **Every** row of **every** front screen — world select's cards,
  New/Back/Favorite/Rename/Delete, and the character-select twins — was dead
  until the app was restarted. The C# decrements unconditionally:
  `Main.UpdateUIStates` (`Main.cs:17039`/`:17487-17492`) reaches
  `MenuUI.Update` on every frame, and `UserInterface.cs:203-204` clears the
  countdown before reading the gate. The port now calls
  `App::tick_pointer_clock` from `frame()` on every frame, outside the
  in-game branch, with a citation naming the C# reach. This replaces the older
  note here that the 200 ms disable "is armed on the enter direction only
  (`enter_front`), not on the leave direction the C#'s `SetState(null)` also
  resets" — neither direction decayed.
  The existing double-click test had masked it by hand-ticking the lockout away
  (`app.pointer.tick(300.0)`); it now advances the real `frame()` seam.
  The screen geometry itself was never implicated: `world_select::card_hit`
  (`ui/world_select.rs:506`), the layout shared with char_select (`:42`, `:185`)
  and the framework's `get_element_at` (`ui/framework.rs:64`) are all 1:1 with
  `UIWorldListItem.cs:94-158` and `UIElement.cs:324-352`.
  **Re-landed 2026-11-10 (fix-world-select-clicks-2):** the M6-ACH-2 merge
  (a75dd14) silently reverted this fix - its auto-resolution restored the
  pre-fix frame() branch (the decay back inside menu.is_none() &&
  front.is_none()) and dropped the six regression tests, so both select
  screens locked out again (AGENTS.md 2026-11-08 incident class: a lossy
  merge dropping merged work). Cherry-pick of 7351e6c onto the ACH-2 tree;
  the tests ride along and now guard the regression; the ACH-2 draw arm in
  the same frame() region survives by construction (the cherry-pick touched
  only the lockout lines).

* **The bestiary panel was 180 px short.** `bestiary::panel_rect` hand-wrote
  `screenH − 220 − 180` (= `screenH − 400`) with a 560 width floor, while
  `UIBestiaryTest.cs:71-79` is `Top 180` + `Height.Set(-220f, 1f)` = `screenH − 220`
  tall (bottom at `screenH − 40`), `MaxWidth 900`, `MinWidth 700`
  (`600 + Utils.ToInt(true) * 100`). The container is now `uistate::resolve`'d
  verbatim; the old numbers stayed because nothing pinned the absolute bottom in
  a test.
* **The emotes wheel borrowed the bestiary panel.** `ui/emotes.rs` reused
  `bestiary::panel_rect` and a 44 px / 8-column grid, while `UIEmotesMenu.cs`
  has its own 590-wide container (`Top 120 / Height −120` in game,
  `Top 220 / −220` in the menu) and `EmotesGroupListItem.cs:15`/`:25` specify
  36 px cells at `maxEmotesPerRow = 14`. The wheel now owns its container and
  grid, and its scroll offset moved from standing still to `UIList.ScrollWheel`
  (`UIList.cs:107-114`) with `UIScrollbar`'s clamp (`:37-47`), reset on each open
  (`OnActivate`'s rebuild). The C#'s per-group titles/separators are **not** ported
  — the port keeps one flat roster — and the menu-side container split is
  unimplemented (the wheel is reachable only in-game).

Open gaps (deliberately out, each with its reason):

* **Event bubbling** (`UIElement.cs:414-546`: every mouse event fires the
  element's handler and then the *parent's*) — a no-op on flat screens; the port's
  click tables return one action.
* **`MouseOver`/`MouseOut` edges** — hover is draw-derived (`in_rect` at draw
  time) and the colour swaps are `ui::hover_pair`; the edge itself has no ported
  consumer, and the sound 12 it drives is a recorded deviation.
* **`OverflowHidden`** (`:47`, `:222-237`, 6 users: `UIList`,
  `UIBestiaryEntryButton`, `UIBestiaryNPCEntryPortrait`, `UIResourcePack`,
  `AWorkshopPublishInfoState`) — the select screens' clipped rows already emulate
  the clip by omitting the row from the element list.
* **`UIList`/`UIScrollbar` behaviour** (`UIList.cs:107-128`, `UIScrollbar.cs:37-143`):
  the draw-only helpers exist in `ui/panel.rs`, but there is no `view_position`,
  no drag and no handle geometry; the char/world select bars draw at a hard-coded
  ratio and the emotes wheel's scroll is a plain row offset.
* **`IsElementUnderMouse`/`MouseCaptured`** (`UserInterface.cs:113-124`, `:349-356`)
  — only the minimap's hover mirrors the mine/place suppression today.
* **`OnActivate`/`OnDeactivate` for the remaining screens** —
  `NewCraftingUI.cs:901`/`:911`, `UIAchievementsMenu.cs:177`, `UIManageControls.cs:694`,
  `UICreativePowersMenu.cs:97` have no ported screens; `UIEmotesMenu`'s
  `RemoveAllChildren` rebuild is only its scroll reset; and the 13 `front = None`
  exit sites have no `leave_front` seam, so the 200 ms disable is never re-armed
  on the leave direction the C#'s `SetState(null)` also resets
  (`open_keyboard_with_tail` at `app/mod.rs:1159` and the mode-40 `No` restore at
  `app/clicks.rs:990` set `front` without going through `enter_front`) — the
  inverse of the latch fixed in the corrected-deviations list above, and
  harmless because the countdown now always expires.
* **Snap points / gamepad** (already recorded as out of scope) and
  `PassThroughMouseInteraction` (declared but never set in 1.4.5.8).

Screenshot baselines (deterministic double-run pairs, SHA256, 1600x900):
`TR_CHEST` `d4ba148e…` (also byte-identical to the pre-phase HEAD build taken in
the same session), `TR_BESTIARY=0` `faf45d9e…` (new — the panel's bottom moved),
`TR_EMOTES` `9d00f7b0…` (new — the wheel's box and cell pitch moved), `TR_UI=charsel`
`39b1f697…` and `TR_UI=worldsel` `077087ab…` (both need `TR_NAME` — the session
name falls back to `menu::random_name()`, `app/mod.rs:1830`, so an unpinned pair
differs in the card name's pixels). The absolute HUD hash is environment-dependent:
two sessions produced `c4980496…` and `d4ba148e…` for the *same* code, so a
cross-session comparison must re-take the baseline beside the run.

## Architecture refactor (2026-11-01)

The god-file split (`docs/superpowers/plans/2026-11-01-architecture-refactor.md`,
commits `9c84351..028daba`) moved code, gave the moved functions readable Rust
names (overriding the C#-name rule for this pass) and added three C#-shaped
trait seams. Behaviour and RNG streams were pinned unchanged: the worldgen
byte-proof (four FNV-1a state hashes — at the time an out-of-tree harness;
now in-tree as `worldgen/world_hash.rs`'s pinned constants, M5-GEN-1) and a
post-refactor determinism pair
(SHA256-equal repeat screenshots) both pass; the workspace tests read
399/10/147. The new tree, the "where to look" map and the frame flow live in
`docs/ARCHITECTURE.md`; the `PORT_MAP.md` rows are re-pointed. The movements,
compressed:

- **tr-app.** `main.rs` 10 497 → 86 lines (bootstrap only). The `App` split
  into `app/` (`mod.rs` struct + startup, `loop.rs` the winit handler,
  `frame.rs` the `Main.Draw` orchestrator + G6 cache, `flow.rs` the front-end
  state machine + `FrontScreenBehaviours`, `clicks.rs`, `session.rs`,
  `settings_ui.rs`, `netglue.rs`, `saveglue.rs`); the world draw passes into
  `render/` (`sky`, `bg`, `tiles`, `liquids`, `entities`, `foliage`, `debug` —
  ~4.5 k lines); the fixed-step simulation into `world_sim/` (`step`, `player`,
  `npcs`, `items`, `projectiles`, `editing`, `clock`, `liquids`, `weather`,
  `camera`); leaf types out of the bootstrap into `item.rs` and `biome.rs`.
- **`menu.rs` 4 187 → `menu/`**: `mod.rs` (state, items, slider drag),
  `pages.rs` (the per-screen row tables), `actions.rs` (the `Action`
  semantics the App dispatches).
- **tr-core directories.** `tiledata.rs` regenerated by the fixed
  `tools/GenTileData` and split (`sets`/`emissive`/`merge` + a hand-written
  `mod.rs`); `map.rs` → `map/` (`mod`/`colors`/`option`/`file`); `tile_light.rs`
  → `tile_light/` (`mod`/`palette`/`scanner`); `lighting.rs` → `lighting/`
  (`mod`/`params`/`blur`) with the `LightEngine` trait; `waterfall.rs` →
  `waterfall/` (`mod`/`draw`, the `LiquidKindChecks` trait artifact replaced by
  the free function `any_liquid_kind`); `worldgen.rs` → `worldgen/`
  (`mod`/`noise`/`terrain`/`caves`/`liquids`/`surface`); `localization.rs` →
  `localization/` (`mod`/`culture`/`format`/`json`); `binio.rs` is the one
  little-endian cursor behind `save`/`wld`/`map` (the app-side `.trwld` header
  probe in `ui/save_list.rs` joined in this docs sweep).
- **Deduplication.** `ui::in_rect`/`in_rect_strict`/`hover_pair`/`push_whole`
  and the `text::border_string`/`draw_border_string`/`large_text_width`
  families replaced their per-screen copies; `ui/cards.rs` shares the select
  screens' card pieces; `world_sim/step.rs`'s `FixedStepper` replaced six
  identical 60 Hz accumulators; the `ReLogic.Utilities.Crc32` port lives once
  in `ui/world_creation.rs`.
- **C# seams** (the only new abstractions): `FrontScreenBehaviours` ≈
  `UIState` (`app/flow.rs`), `LightEngine` ≈ `ILightingEngine`
  (`tr-core/src/lighting/mod.rs`), and `render/bg.rs`'s `PassPrologue` ≈ the
  `bgStartX`/jitter prologue the `Main.DrawBackground*` passes repeat.

Deviations recorded by the refactor and this sweep:

- **`tools/GenTileData`'s `numN` local is approximated, not modelled.** The
  `TileID.Sets.NeedsGrassFramingDirt[num]` argument is a decompiler-renamed
  local (`num11` in the previous decompilation, `num9` in the current one);
  the fixed generator resolves it to the active `case` ids (the grass id
  itself) instead of modelling the local's data flow — the approximation the
  framer expects grass variants to merge against (`Program.cs:613-621`).
- **`env.rs` landed (2026-11-09, REFACTOR-env).** The ~112 `TR_*` flags
  consolidated into `crates/tr-app/src/env.rs` as named accessors: every pin
  resolves once through a `OnceLock` and keeps its original read semantics
  (`env::var` vs `env::var_os`, the presence/value/parse shape), so no
  screenshot or test behaviour changed. `TR_SUNDIAL`/`TR_MOONDIAL` deliberately
  stay live reads — they are the only flags a test mutates (`set_var` /
  `remove_var` in the DW-1.6 pair, serialized by `ENV_LOCK`) and a `OnceLock`
  would latch the first test's value into the second's. The tr-core /
  tr-render pins (`TR_NOCACHE`, `TR_PROBE`) stay with their crates — a
  cross-crate pins module is the follow-up.
- **The P6 stand-ins** this sweep documented (see "Front-end screens (P6)"):
  `GameTipsDisplay` not ported, the Crc32-instead-of-`Main.rand` world-name/
  seed composition and the `assign_random_world_name` stand-in.
- **Risk 7 above is closed**: the generator matches the current
  decompilation again, and `tiledata.rs` is generated from it (the hand
  extraction in `torch.rs` stays as-is).

## GetTileDrawData deviations (TILE-DRAW-1, 2026-11-02)

`TileDrawing.GetTileDrawData` (`TileDrawing.cs:4527`, the type switch at
`:4540-6032`) is ported in full as the static layer of
`crates/tr-core/src/drawdata.rs`: a pure table
`draw_data(kind, x, y, frame_x, frame_y, tile_frame, solid_above) -> TileDrawData`
plus `TileDrawData::apply_world_corrections` for the four world-probing
corrections (below). Every case of the switch has an explicit status — an arm
carrying its unconditional static part, or a documented deviation — nothing
falls through silently. `C# ref tileFrameX/Y` rewrites are expressed as
`frame_x_override`/`frame_y_override`; the caller (`render/tiles.rs`) builds
the source rectangle from the final frame (`TileDrawing.cs:1007`).

**Animation state.** `tile_frame` is `Main.tileFrame[kind]`; the animator is
not ported, so the production call passes 0 — exactly the vanilla t=0 state.
All `tile_frame * N` formulas are ported verbatim and activate unchanged when
the animator lands. The default `addFrY = Main.tileFrame[type] * 38`
(`TileDrawing.cs:4533`) is ported as the same formula. `tile_frame = 0` was
verified against the C# per formula (transcription tests in `drawdata.rs`).

### Status of every switch case (`TileDrawing.cs:4540-6032`)

**Ported in full** (the case is only its static part):

- `752` (`:4542`), `751` (`:4547`), `739|748` (`:4552`), `726` (`:4556`),
  `443` (`:4571`), `571` (`:4581`), `698` (`:4588`), `136` (`:4593`),
  `561` (`:4599`), `330-333` (`:4631`), `129` (`:4637`), `476` (`:4663`),
  `323` (20×20 static `:4667-4670`; the biome frame row is a world
  correction, below), `4` torch with the solid-above probe
  (`:4672-4679`), `78`-group
  (`:4680`), `100` (`:4697`), `705` (`:4724`), `33`-group (`:4819`),
  `3`-group (`:4850`), `20|590|595|615` (`:4864`), the 18 px group
  (`:4880`), `79` (`:4909`), `90` (`:4917`), `18` (`:4924`), `654`
  (`:4940`), `14|21|411|467|469` (`:4943`), `15|497` (`:4953`),
  `172|376` (`:4960`), `27` (`:4967`), `132|135` (`:4973`), `82-84`
  (`:4978`), `324` (`:4988`), `494` (`:4997`), vine family (`:5000`),
  `80|142|143` top 2 (`:5014`), `139` (`:5019`), `73|74|113` (`:5027`),
  `388|389` (`:5037`), `227` (static `:5052-5062` + `:5079-5082`; the
  `frameX == 204` biome rewrite is a world correction, below),
  `624|700` (`:5084`),
  `656|701` (`:5094`), `567` (`:5131`), `650` (`:5184`), `649` (`:5187`),
  `647|706` (`:5195`), `648` (`:5199`), `178` (`:5207`), `184` (`:5213`),
  `519` (`:5224`), `28`-group (`:5258`), `617` (`:5276`), `614` (`:5281`),
  `81` (`:5286`), `272` (`:5295`), `106` (`:5298`),
  `300-308|354|355|499` (`:5301`), `377` (`:5316`), `463|464` (`:5320`),
  `491` (`:5325`), `379` (`:5329`), `390` (`:5357`), `412` (`:5360`),
  `36` (`:5364`), `406` (`:5367`), `452` (`:5387`), `454` (`:5409`),
  `405` (`:5418`), `12|31|96|639|665|696` (`:5433`), `238` (`:5441`),
  `592` (`:5488`), `228|231|243|247` (`:5491`), `244` (`:5498`),
  `565` (`:5509`), `235` (`:5520`), `217|218` (`:5523`),
  `219|220|642` (`:5529`), `270|271|581` (`:5535`), `660` (`:5564`),
  `572` (`:5591`), `442` (`:5612`), `207` (`:5908`), `410` (`:5930`),
  `480|509|657|720|721|725` (`:5945`), `658` (`:5962`), `733` (`:5982`),
  `326`-group (`:5991`), `336`-group (`:6014`), `89|102|753` (`:6023`).
- The `583-589|596|616|634` tree-variant arm (`:4650-4662`) is complete but
  dead in the app: `tree::is_tree` routes these kinds through the dedicated
  tree path before `draw_data` (same for case `5`, below).

**Static part ported, engine-driven part a documented deviation:**

- `711` (`:4931`): the glow out-params are TILE-DRAW-2.
- `185|186|187` (`:5143-5183`): the strides/top are ported; the
  `Main.tileShine2` writes are not (shine system not ported).
- `349` (`:5332`): the `tileFrameX` rewrite needs `Animation.GetTemporaryFrame`.
- `441|468` (`:5342`): the `tileFrameY` rewrite needs
  `Animation.GetTemporaryFrame` (the `tileFrameY == 18` height is ported).
- `493` (`:5232`), `489` (`:4731`): the addFrY animation needs
  `Main.tileFrameCounter` (+ `InAPlaceWithWind` for 489); top 2 ported.
- `490` (`:4742`): the bob needs `InAPlaceWithWind` + `Main.WindForVisuals` +
  `Main.weatherVaneBobframe`; outside wind C# yields addFrY 0 — the same 0
  the port produces at t=0. Top 2 ported.
- `529` (`:4827-4849`): the `addFrY += 34 * <biome/beach pick> - tileFrameY`
  rewrite needs `WorldGen.GetBiomeInfluence` + the `beachDistance` band;
  32 px tall / top −14 / flip are ported.
- `530` (`:4705-4722`): the `addFrY += 36 * <biome pick>` rewrite needs
  `WorldGen.GetBiomeInfluence` (WorldGen.cs:47750); top 2 ported.
- `485` (`:4727-4730`): the `tileFrameCounter`-driven addFrY is not ported;
  top 2 is.
- `428` (`:5605`): a pressed `PressurePlateHelper` plate adds addFrX 18; the
  port treats plates as never pressed. `455` (`:5397`): the extra row applies
  while `BirthdayParty.PartyIsUp`; the port treats it as always false.
- `593` (`:5445`) / `594` (`:5466`): the static fallbacks
  (`tileFrame * 18/36` below the frame band, addFrX −18/−36 on later columns,
  top 2) are ported; the animated addFrY needs `Animation.GetTemporaryFrame`.
- Cage groups (`:5629-5755` big, `:5756-5832` small, `:5833-5897` water,
  `:5898-5907` jellyfish): top 2 ported; the per-animal `Main.*CageFrame`
  animation tables and the `Main.critterCage` side effect are not ported
  (the jellyfish-cage state machine was already recorded under "Trees").
- `564` (`:5523-5528`): shares the `217|218` arm in C#, but the glow switch
  then unconditionally clears addFrY (`:6105-6113`); the port folds that into
  the arm — top 2, addFrY always 0.

**No arm — default 16×16** (the case has no unconditional static part):

- `5` (`:4645-4649`): the frame rewrite needs `GetTreeBiome`, and the case is
  unreachable in the port (`tree::is_tree` intercepts kind 5).
- `518` (`:4604-4630`): needs `tile.liquid` + the block type below.
- `719` (`:4562-4570`): `Main.GlobalTimeWrappedHourly`.
- `579` (`:5104-5130`): `Main.LocalPlayer.Center.X`.
- `453|456` (`:5412-5417`), `507|508` (`:6007-6013`): `Main.tileFrameCounter`
  hard cycles.
- `541` (`:6004-6006`): `_shouldShowInvisibleBlocks` (invisible blocks are not
  stored by the port at all).

### Epilogues and cross-cutting

- Campfires epilogue (`:6033-6044`; `TileID.Sets.Campfires = {215}`,
  TileID.cs:616 — a local constant in `drawdata.rs`, `tiledata/` is
  generated and untouched): `addFrY = tileFrame*36` below frameY 36, else
  252; top 2.
- The halfBrick epilogue (`:6045-6048`) is the tile pass's separate
  half-brick draw path, not `draw_data`. The glow switch (`:6049-`) is
  TILE-DRAW-2, except 564's addFrY reset (`:6105-6113`) noted above.
- The caller-side source-rect corrections of `DrawSingleTile` (`:1125-1150`)
  are ported as `TileDrawData::apply_world_corrections`: palm 323
  `frameY = 22 * GetPalmTreeBiome` (`:1726`, `:4670`), dye plants 227 with
  `frameX == 204` rewritten per `WorldGen.GetCactusType`
  (`:5063-5078`, WorldGen.cs:47687), cactus 80 source-Y +54/108/162 for
  corrupt/hallowed/crimson ground (`:1132-1147`, kind 80 only — 142/143 are
  pumps), cog 272 parity shift ×90 (`:1125-1131`).
- **Bug 19b (the cactus shift's real consumer)**: the corrected offsets above
  are folded into `TileDrawData::src_y_shift` (renamed from
  `half_brick_src_y`) and the renderer applies them to the PLAIN flat draw
  only, via `flat_src_uv` (`render/tiles.rs`). The C# builds one local
  `rectangle` from `tileFrameX/Y + addFrX/addFrY` (`TileDrawing.cs:1007`),
  the switch mutates its `Y` (`:1129`, `:1137-1145`, `:1162`), and the plain
  arm of `DrawBasicTile` is the consumer (`normalTileRect`, `:1523`); the
  half-brick (`:1298`), slice (`:1560`), slope and edge arms rebuild their own
  rects and therefore drop the shift. The BUG-19 landing had the consumer
  backwards (half-brick only), so every standard cactus in the corrupt/
  hallowed/crimson desert drew the wrong row (0 px instead of 54/108/162).
  Tests: `drawdata::tests::bug19_the_cactus_shift_lands_on_the_flat_draw_rect`
  and `render::tiles::tests::test_BUG19b_the_cactus_flat_draw_carries_the_biome_row_shift`
  (the UV moves by ≥ 50 sheet px while the unshifted case reproduces the
  frame exactly).
- `solid_above` on the Rust call site is `active && TILE_SOLID &&
  !TILE_SOLID_TOP` at `(x, y-1)` (`render/tiles.rs`); the C# analogue probed
  by case 4 is `WorldGen.SolidTile(x, y-1)` (`:4672-4679`). The predicate
  delta is a follow-up, documented on `draw_data` (Phase 4).
- **The positional switch** (`DrawSingleTile` `:1020-1159`, landed as
  TILE-DRAW-5): `apply_world_corrections` now carries the switch's draw-local
  revisions on `TileDrawData` — `draw_dx`/`draw_dy` (the `vector` nudges: 751
  (+11, −8), 752 (+8), 136 (±2 by `frameX / 18`), 442 (`frameX / 22 == 3` →
  +2), 726 (−2 then the `Tile.blockType()` ledges ±6 and +2 for the
  half-brick/slope arms), 723/724 (the four anchor sides), 129 (the 2 px nudge
  away from the shard's cell) and 323 (`vector.X += frameY` for the palm
  rows)), `skip_draw` (751/752 off their `(0, 0)` frame), `light_scale` (51/697
  `finalColor = tileLight * 0.5`), `src_alpha` (129's 100/255, and 0 for the
  big `frameX >= 324` shards) and `no_width_adjust`
  (`TileID.Sets.DoNotAdjustDrawPositionBasedOnTileWidth = {711}`). The
  renderer adds the offsets to the cell origin before dispatching (every C#
  arm shares `vector`), honours `skip_draw` before the sprite (the `return`
  arms), folds `light_scale`/`src_alpha` into `push_lit`'s colour, and skips
  the `(tileWidth − 16) / 2` centring for 711. Parked: the disco arms
  (160/692, they need `Main.DiscoR/G/B` in the draw data), case 83's texture
  override, and case 323's `return` for the frond columns 88..=132 — the port
  has no palm canopy pass, so hiding those columns would delete the palm tops;
  they keep drawing in the tile pass (deviation). Test:
  `drawdata::tests::the_positional_switch_nudges_and_hides_the_sprites` (every
  arm, the half-brick/slope `blockType` table of 726, the shard alphas).

### Verification (Phase 3, branch `TILE-DRAW-1-drawdata-finish` @ 40fcf82)

- Workspace suite: 399 + 10 + 174, 0 failed (`--skip
  migrates_version_2_kinds_to_item_ids`).
- A/B against the pre-edit baseline (commit 35dce1b, master world
  `assets/test/master.trwld`, seed 0, 4200×1200): all three shots
  (forest spawn / forest-low / desert) pixel-identical — 0 of 1 440 000
  pixels differ — and **SHA256-equal to the baseline files**
  (forest `e9aedf24…`, forest-low `8f480cd0…`, desert `86d04b6e…`);
  a repeat run of each shot reproduces the same SHA256 (determinism pair).
- Why the diff is empty: the shot windows contain only kinds
  {0, 1, 2, 3, 5} (forest — dirt/stone/grass/plants/trees) and
  {53, 80, 396, 397} (desert — sand, 11 cacti, sandstone/hardened sand).
  None of
  these changed draw-data at t=0: 0/1/2/53/396/397/5 have no case in either
  build (5 goes through the tree path), the `3`-group and `80` arms are
  metric-identical to the pre-existing subport, and the cactus ground shift
  is 0 on plain sand 53. Globally the world has no palms (323) and no dye
  plants (227), so that wiring is covered by the Phase 2 unit tests only.
- Zooms 1.0/1.5/2.0 (TR_ZOOM): 1.0 is the A/B pair above; 1.5 and 2.0 each
  reproduce SHA256-equal pairs (`67b08fad…`, `f7c4cf0b…`), and all three
  zooms differ from each other (the pin takes effect).
- Scope of the evidence: the runs above verify the branch @ `40fcf82`
  against the pinned pre-edit baseline. `main` has since absorbed
  TILE-DRAW-2 (torch flames/glow), M1.5-LIQUID-2 (water styles) and the
  M2-DRAW-1 player layers (none of them touch `drawdata.rs`); after this
  branch syncs with `main`, the final A/B is repeated on the synced state —
  flame/glow pixels will then differ from this pre-TILE-DRAW-2 baseline by
  that work's own account.

## Torch flames (TILE-DRAW-2)

- `tr-core/src/flame.rs` ports `TileDrawing.GetTileFlameData` (`TileDrawing.cs:1847-2677`)
  and both inline tables of `DrawSingleTile_Flames` (`:2826-3462`); the two C# tables
  disagree in a few places (colour of styles 13/14, discarded offsets for sheet 7) and
  are ported literally, kept separate on purpose.
- The 48-bit LCG behind `Utils.RandomInt` is ported (`tile_frame_seed_for`); C# seeds
  `Main.TileFrameSeed` from `Guid.NewGuid().GetHashCode()` (`Main.cs:66567`) —
  unobservable randomness. The port starts from a fixed 32-bit constant with the same
  advance schedule (`Main.cs:61716-61721`), so flame jitter is deterministic across
  runs while distribution-identical.
- Not ported: type 85's graveyard flames (`:2724-2750`, needs
  `Main.GraveyardVisualIntensity`/`ZoneGraveyard`, not simulated); the separate
  `Main.tileGlowMask` table (`Main.cs:10024+`, consumed in `DrawSingleTile` at
  `TileDrawing.cs:1178`) is its own wide task.
- **Premultiplied alpha fix (tr-render):** the XNB textures are premultiplied by the
  XNA content pipeline (verified: a=0 texels are rgb=0, semi-alpha texels satisfy
  rgb <= a across Flames/Moon assets). Vanilla `BlendState.AlphaBlend` therefore uses
  factors (One, OneMinusSrcAlpha): a tint with alpha 0 (the tile flames'
  `Color(200,200,200,0)`) is a pure rgb add. The renderer's previous
  (SrcAlpha, OneMinusSrcAlpha) standard blend double-dimmed every semi-alpha texel and
  made a=0 tints invisible; the pipeline now uses XNA's factors and the shader no
  longer multiplies rgb by alpha. Binary-alpha tiles are unchanged; semi-alpha texels
  (panel fades, font edges) render brighter — the XNA-correct result.
- XNA nuance kept: `TileBatch` blends premultiplied, but the renderer's single
  alpha-blend pipeline cannot express rgb-add at a=0 for partial-alpha tints (the
  furniture/jar colours with a > 0 dim slightly against vanilla's full add). Fully
  a=0 tints (torches) are exact.

## Animations (M2-ANIM-foundation)

The shared primitives live in `crates/tr-core/src/anim.rs` (`FrameCounter` —
the `frameCounter += speed; while (>= threshold)` idiom of
`Player.cs:36259-36264` and `NPC.FindFrame`, `wrap_row_band`, the useStyle-1
thirds, the UI sine pulse). Deviations and ports:

- **Player draw origin** (`render/player_layers.rs` `gfx_off_y` state field):
  the 40x56 frame rides the
  20x42 hitbox at `(x - 10, y - 10 + gfxOffY)` — the resolution of
  `PlayerDrawLayers.cs:1150-1153` plus the `LegacyPlayerRenderer.cs:457,468`
  `position + (0, gfxOffY)` call. `gfxOffY` was previously ignored (the step
  ease popped vertically) and is now also part of the G6 player render key.
- **Composite arm cells** follow `PlayerDrawSet.CreateCompositeData`
  (`PlayerDrawSet.cs:1962-2023`): front arm `(X, Y)` per bodyFrame row, back
  arm the same column two rows down. The old front-arm cell hardcoded column
  8 (the composite *back*-arm use column), which mismatched the pose while
  walking and during useStyle-1 swings.
- **Item swing** (`Player.cs:50686-50852` + `PlayerDrawLayers.cs:3043-3215`,
  implemented inside the `render/player_layers.rs` builders that M2-DRAW-1
  introduced — `held_item_layers` consumes the vanilla `itemRotation` /
  `itemLocation` and the real atlas frame size):
  `itemRotation = (itemAnimation/itemAnimationMax - 0.5) * (-direction) * 3.5
  - direction * 0.3`, `itemLocation` thirds over the held frame's size, pivot
  at the drawn sprite's bottom-left. `SWING_TIME` (0.28 s) still stands in for
  the per-item `useAnimation` — the real timings remain the M2-ANIM-item-timings
  catalog row. The `UsesBetterMeleeItemLocation` nudge is not applied (no
  ported item belongs to that set).
- **NPC.FindFrame** for the four ported kinds now runs in the fixed 60 Hz sim
  (`world_sim/npcs.rs` state, `npc.rs::find_frame`): slime `NPC.cs:70801-70826`
  (2 frames, counter doubles while moving), King Slime `:73380-73414`
  (grounded 0..=3 with the landing squash 4 shown until the first advance;
  airborne 4 then 5), demon eye `:73994-74057` (2 frames + `rotation =
  atan2(vy, vx)`, `+ 3.14f` when moving left, frozen while `velocity.X == 0`),
  zombie `:76324-76389` (walk band 0,1,2,1 at 8/16/24/32 accumulated px;
  lurch frame 2 while airborne or backpedalling). The old wall-clock
  `anim * N` frame derivation is gone; `render_key` now hashes the rendered
  frame and the eye rotation instead of raw `anim` bits, so the G6 cache
  invalidates exactly when the baked pixels change.
- `aiAction == 4` (attacking) never fires for these kinds — the third counter
  tick of the slime/King Slime cases stays out until combat AI lands.
- The demon-eye draw rotates the quad about its centre (`Main.DrawNPC`
  origin = frame centre); the mirrored quad's corners are centre-symmetric,
  so the flip does not move them.

## Item use timings (M2-ANIM-item-timings)

The per-item `useTime` / `useAnimation` / `useStyle` / `holdStyle` /
`autoReuse` now live on `registry::ItemDef`, extracted from
`Item.cs SetDefaults1` for all 20 ported kinds (blocks and ores 10/15,
grass seeds 7/15, shuriken 15/15 with `autoReuse` off, coins with
`noMelee`-style zero damage, torch `holdStyle = 1`). Deviations and notes:

- The swing duration is `useAnimation` ticks at the 60 Hz rate
  (`Player.ApplyItemAnimation`, `Player.cs:4449-4451`); the fixed `SWING_TIME`
  stand-in is gone and the render builders consume the remaining *fraction*
  (`itemAnimation / itemAnimationMax`).
- The cadence follows `ItemCheck`: an `autoReuse` kind re-fires while the
  button is held; a non-autoReuse kind waits for a release
  (`Player.cs:42733`, `42819-42822`) — the shuriken throw is press-edge gated,
  and the release edges refresh every frame like `Player.releaseUseItem`.
  Placement (the RMB path) fires whenever `itemTime` hits zero inside a
  running swing (`Player.cs:39743` + `ApplyItemTime = useTime`,
  `Player.cs:4416-4418`) — dirt every 10 ticks inside the 15-tick swing,
  walls every 7; the legacy `place_cooldown = 0.12` retired. The LMB mining
  path keeps the port's own `mine_timer`/hardness model paced by the swing
  (`M3-TOOL` owns pick power), a documented deviation.
- A resting `holdStyle == 1` item (the torch) is drawn in the hand —
  anchor `position.X + width/2 + (frame.W/2 + 2)·direction`,
  `position.Y + 24`, `itemRotation = 0` (`Player.cs:50275-50460`) — and pins
  the body to row 3 (`Player.cs:36455-36459`).
- The M2-DRAW-2 `swing_stand_in` (rotated composite front arm during a swing)
  retired for useStyle 1: the original never calls `SetCompositeArmFront` in
  the useStyle-1 branch (`Player.cs:50686-50953` sets only
  `itemRotation`/`itemLocation`), so the arm pose rides the bodyFrame thirds
  and the item swings alone. The stand-in returns for styles 5/9/11 with the
  bow/thrown tasks.
- XNA pivot semantics (`PlayerDrawLayers.cs:3161`): the draw origin
  `(W/2 - W/2·direction, H)` positions the quad; the `direction < 0` flip
  swaps the UVs inside the quad, not the quad geometry (MonoGame
  `SpriteBatch.DrawInternal`). This also fixed the held-item mirror that
  main's `origin_drawn = width - origin` adjustment got backwards.
- The `itemLocation` thirds run on the DECREASING `itemAnimation`: the
  C# chain's first branch (`itemAnimation < max * 0.333`,
  `Player.cs:50709-50734`, Y+24 / X+10) is the swing END and the last
  (`:50776-50843`, X subtracted) the start — the port had them swapped since
  the M2-ANIM-foundation pass; corrected here (pre-existing, byte-identical
  visually at rest).
- The shuriken damage now matches `Item.cs:1876` (10, was 15) — a registry
  data fix spotted during the review; the remaining pre-existing registry
  mismatches (coins' damage/noMelee, Wood's `createTile`) stay with the
  item-data catalog work.

## Tile objects (M1.4-TO)

`tr-core/src/tileobject.rs` ports the `TileObjectData` registry as flat const
data (approach A — a full-registry generator is the follow-up) and
`tr-core/src/edit.rs` the logic on top of it: `TileObject.CanPlace`/`Place`
(`TileObject.cs:174-853` / `:29-172`), the KillTile destruction cascade through
the six per-type `Check*` routines (`CheckDoorClosed` `WorldGen.cs:86432`,
`CheckDoorOpen` `:86339`, `Check3x2` `:49187`, `Check1x2` `:43335`, `Check2x1`
`:46973`, `CheckChest` `:57923`; the origin reconstruction is generalized
through the registry: `dx = frameX/18 % width`, `dy = frameY/18 % height`, the
open-door hinge via the C# `frameX % 72` switch), and `WorldGen.OpenDoor`/
`CloseDoor` (`:37630-37784` / `:31805-31907`) behind `interact_at` — the
port's first world-tile interaction hook, embedded in the fixed-step editing
pass at the vanilla `Update` order point (interactions `Player.cs:27430` →
`ItemCheck` `:31827`). The registry slice is the wooden set — closed/open door
(10/11), table (14), chair (15), work bench (18), chest (21); every entry
cites its builder chain. Deviations and seams:

- The registry is a deliberate slice: sub-tiles are not ported (the table's
  styles 13/25, `TileObjectData.cs:3182-3190`) and the gemspark/glow tiles
  stay excluded (research decision — they are wall/light render work, not
  placement); adding objects is data-only until the generator lands.
- The chest is no longer a pure tile object (flipped 2026-11-07, M3-CHEST-UI):
  the item storage and the open/lock/unlock/rename interaction are ported —
  `crates/tr-app/src/chest.rs` owns the `Chest.cs` entity semantics and the
  world-chest store, `crates/tr-app/src/ui/chest.rs` the `ChestUI.cs` panel
  (see "Chest UI + world chests (M3-CHEST-UI)" below). The placement-time
  entry hooks (`Chest.FindEmptyChest`/`Chest.AfterPlacement_Hook`,
  `TileObjectData.cs:3464-3465`) stay unhooked — store entries are created
  lazily on first open instead (empty-entry-equivalent; that section).
- Net placement replication (`apply_edit` kind 1, `world_sim/editing.rs:548`)
  sends style 0 / dir 1: the generic replication path carries no
  style/alternate/direction (vanilla's `NetMessage.SendObjectPlacement`
  does), so a left-facing chair replicates mirrored.
- `CloseDoor`'s `genRand.Next(3)` closed-frame variant is pinned to 0
  (`edit.rs`, `close_door`): the port has no runtime RNG stream to draw from
  and the determinism invariant forbids introducing one; the two skipped
  variants are a closed-frame cosmetic only.
- The interaction hook consumes the click with no same-time fall-through to
  placement — the research contract is stronger than vanilla's nuance where a
  refused door toggle and a placement can land in one `Update`.
- Door sounds are wired (Door_Opened/Door_Closed cues); the door net message
  (vanilla 19) is NOT wired — there is no `apply_edit` kind for a toggle yet;
  multiplayer seam.
- The left-facing open door's frames come from the generic `TileObject.Place`
  math (style-row shift → frameY 54) while vanilla's hand-written `OpenDoor`
  uses a different hardcoded left-leaf layout (frameX +36,
  `WorldGen.cs:37695-37698`); both are ported as-is — a vanilla quirk, not a
  port bug.
- The wooden recipes expand the `RecipeGroups.Wood`/`IronBar` requirement
  groups to concrete ids (Wood → 9, Iron Bar → 22; `Recipe.cs:610-616` /
  `:4738`) — the subset recipe table carries no group machinery (`craft.rs`).
- LMB breaking of `frame_important` tiles was gated off in
  `world_sim/editing.rs` (the former `:204-210`/`:213` as of the M1.4-TO-5
  claim — `!important`, cacti exempted; the earlier `:158-165` citation had
  drifted after the M2 merges). Removed 2026-11-04 (M1.4-TO-5): vanilla's
  mining chain has no `tileFrameImportant` check at all (grep over
  `Player.cs` hits only `:40019`/`:40608`, both outside the mining chain —
  `Player.cs:45656` → `PickTile :53757`), so doors/chests/furniture reach
  `break_tile` through the mining branch with the full KillTile cascade and
  the object drop. Cactus 80 keeps its old path (`check_object`
  `_ => None`, the per-cell drop); the visible remainder is the missing
  `CanKillTile` refusals (next bullet, M1.4-TO-6).
- `CanKillTile` refusals landed 2026-11-06 (M1.4-TO-6): C# `CanKillTile`
  (`WorldGen.cs:62219-62326`) refuses a locked door type 10 (`:62290-62295` —
  the already-ported `IsLockedDoor` frames, `:69191-69197`), a chest 21/467/88
  holding items (`:62311-62323` — the app's chest store answers
  `CanDestroyChest`, `Chest.cs:609-623`; the store is populated since
  M3-CHEST-UI, so a stocked chest refuses its break for real) and the tile
  under a container {21,26,72,77,88,467,488}
  (`:62259-62270`); `edit::can_kill_tile` (`edit.rs:1798`) ports the three
  and gates the mining tick (`damage = 0`, `Player.cs:53830-53833`),
  `break_tile`, the replicated break (`apply_edit` kind 0) and the bomb
  explosions. Deviations: the C# `out bool blockDamaged` flavour collapses
  into one bool (the 2-arg overload discards it, `:62213-62217`); the same
  C# function's tree-trunk/palm/cactus anchors (`:62247-62257`,
  `:62271-62281`), the boulder scan (`:62283-62287`) and the teleporter 235
  (`:62297-62310`) stay unported; `kill_tile`'s body and the cascade's
  re-entries are not re-checked (the C# `KillTile` entry duplicates the
  refusals through `CheckTileBreakability` `:62377-62461` — the port chokes
  the outer break sources instead, one query shared by all of them); the
  per-tick `KillTile(fail: true)` sound/dust of a refused pick
  (`Player.cs:53822-53831`) has no ported path, so the refusal is silent;
  explosion/replication pass `|_, _| false` for the chest store — the
  M3-CHEST-UI task left this unchanged (the M1.4-TO-6 deviation stands; the
  store is populated now, so the widening is observable and is the PHYS-7
  follow-up).
- The mining break carries no net messages: vanilla syncs the break/fail
  with `SendData` 17/34/125 (`Player.cs:45685-45696`, `:53774-53819`); the
  SP port has no replication for it (the same seam as the door message 19
  above).
- Placement does not run the KillTile cascade: `reframe_area` (and
  `frame_world`) leave `frame_important` tiles alone, so a floating object at
  placement does not pop although vanilla's `PlaceTile` → `SquareTileFrame`
  would kill it; it resolves at the next neighbouring break.
- The object route bypasses the app's legacy single-tile pre-checks
  (neighbour/overlap) — `TileObject.CanPlace` owns the verdict;
  `BlockPlacementIfOverPlayers` (`Player.cs:41427`) is not ported for the
  wooden set.

Verification: the workspace suite is green (457+12+276, `--skip
migrates_version_2_kinds_to_item_ids`); deterministic `TR_FILL`/`TR_PLACE`/
`TR_TOGGLE` load hooks (the `TR_PAINT` precedent) stage the wooden set on a
painted strip; twin runs are SHA256-equal at zooms 1.0/1.5/2.0 (`49c07431…`,
`f9b4d310…`, `c1bacbc2…`); the door scenario closed→open→closed returns to the
byte-identical closed frame (`door2 == z1a == 49c07431…`) while the open frame
differs exactly inside the door column (pixel_diff 1664 px, bbox
(1016,434)-(1055,497)). There is no vanilla baseline for placed furniture —
geometry fidelity is carried by the registry/`CanPlace`/`Place` unit tests.

Verification (M1.4-TO-5, commit `52d5267`): the workspace suite is green
(`test result: ok` ×7: 673+16+393+19 passed, 0 failed, `--skip
migrates_version_2_kinds_to_item_ids`); the mining branch is covered by unit
tests in `world_sim/editing.rs`
(`test_DW_1_1_frame_important_door_breaks_through_the_mining_branch`,
`test_DW_1_1_frame_important_chest_breaks_through_the_mining_branch`,
`test_DW_1_2_cactus_80_still_breaks_with_the_per_cell_drop`,
`test_DW_1_2_non_frame_important_tile_still_breaks_as_before`,
`test_DW_1_5_releasing_mine_down_resets_the_mine_state`,
`test_DW_1_5_opening_the_inventory_resets_the_mine_state`, plus the torch
edge case `torch_frame_important_kind_breaks_single_cell_with_its_item`);
deterministic `TR_FILL` + LMB-break runs are SHA256-equal twins at zooms
1.0/1.5/2.0 (`abf1ce09…`, `0ac90a0a…`, `3e4b56fd…`; the pre-fix run shows
the no-break frame, `a4168a96…`), and pixel_diff against the pre-fix frame
at zoom 1.0 differs exactly in the door column + the drop item (1064 px,
bbox (1016,434)-(1053,495)).

Re-taken on the post-merge state (`32a1aed`, M6-MUSIC/M7-FALL/M7-WIRE in):
warm-run break twins are byte-equal at zooms 1.0/1.5/2.0 (`105a557f…` ×4,
`03a86efd…` ×3, `ca9f40fa…` ×3 — the hashes above are claim-time bytes, the
merged content changes pixels globally); the matched-camera before/after
diff at zoom 1.0 is 910 px, bbox (1016,434)-(1047,495) — the same door
column + drop. Harness notes for future shot runs: the first run after a
rebuild (or at a new zoom pipeline) fires the shot mid-camera-lerp (camera
`2025.26`/`2030.75` instead of the `2012.000` target — whole-frame twins
look unequal), and a warm no-break run can settle at that second camera
attractor; break runs always converged. Take twin pairs only on warm runs
and match the `shot camera px` log line before comparing frames.

Second sync-merge (`af4fe98`, 2026-11-04 — M4-PROJ-9 bombs, M2-BUFF-7 food,
the tool tiers, the itemdata regen): the mining branch keeps the ungated
condition and gains the tool-tier gates on top — the same union C# mines
with (`Player.cs:45656`: the pick on non-axe/non-hammer tiles, the axe on
`tileAxe`, the hammer on `tileHammer`); the DW-1.x fixtures hold the Iron
Pickaxe (the cactus test the Iron Axe), because the empty slot they used
before has no tool power and mines nothing. The itemdata regen also moves
item 276 to place `CactusBlock` 188 (`TileID.CactusBlock = 188`), which
emptied `by_tile(80)` and silently dropped the world-cactus break's item;
`tile_drop`'s `80 => 276` case restores it (`KillTile_GetItemDrops`,
`WorldGen.cs:64717-64720` — cases 80 and 188 both drop 276), commit
`423a503`. Suite: `test result: ok` ×7 (747+16+431+19, 0 failed). Re-taken
on the merged state: break twins byte-equal at zooms 1.0/1.5/2.0
(`6b613cb5…`, `c00e22e6…`, `9e7f6c85…` ×2 each; before `b6b4d136…`), and
the matched-camera before/after diff at zoom 1.0 is again 910 px,
bbox (1016,434)-(1047,495) — the merged content changes the frame bytes
globally, but the break's own pixel footprint is unchanged. Re-verified
twice more after the later sync-merges (M7-TE-2: logic sensors + the
training dummy; M4-TOWN-1: the Guide + the housing move-in — neither
pixel-bearing in the break scene, the Guide spawns off-frame): all four
hashes and the 910 px footprint are byte-identical, so the shot evidence
carries over as-is.

Verification (M1.4-TO-6): the workspace suite is green (`test result: ok` ×7:
910+16+569+19 passed, 0 failed, `--skip
migrates_version_2_kinds_to_item_ids`; the claim-time `7c39dee` read
902+16+569+19 before the M3-EQUIP-3 sync-merge); the
refusals are unit-covered in `tr-core/src/edit.rs`
(`the_container_family_above_refuses_can_kill_tile` — all seven ids plus the
same-kind control, `the_locked_door_refuses_can_kill_tile` — the 594/646 rows
and the `frameX >= 54` escape, `the_stocked_chest_and_dresser_refuse_can_kill_tile`
— the `frameX / 18 % wide` anchor math for 21/467/88,
`can_kill_tile_contract_head_refuses_air_inactive_and_wall_350`,
`kill_tile_itself_stays_ungated` — the Design-A split) and in
`world_sim/editing.rs` (`the_support_under_a_chest_refuses_the_mining_break`,
`a_stocked_chest_refuses_the_mining_break_until_emptied` — the DW-1.1 chest
first refuses, then breaks after `world_chests.clear()`,
`the_locked_door_refuses_the_break`). No shot re-take: the refusals only
remove effects — a refused break renders the pre-existing no-break frame, and
the non-refused path's pixel footprint is unchanged; determinism is carried by
the suite.

M1.4-TO-4 (herbs/pots/alch/sunflower/piles, 2026-11-05) extends the umbrella in
four layers. The registry slice grows by 27/28/78/82-84/185-187 (`tileobject.rs`,
every def citing its `TileObjectData.cs` builder chain; `TILE_ALCH = {82,83,84}` =
`Main.cs:7806-7808`; the planter box 380 stays out of the registry, as in C#);
placement ports `PlaceAlch` (`WorldGen.cs:45380-45479` — the per-style soil sets,
the liquid rules, `type=82, frameX=18*style, frameY=0`) as the seed branch of
`place_item_tile`, with the planter box at `:60226-60230` (`frameY=18*style`).
The KillTile cascade gains the per-type arms — `check_alch` (`:45972-46092`),
`check_sunflower` (`:53729-53792`), `check_pot` (`:56797-57016`), `check_pile` +
its 2x1 variant (`:46875-46971`), `check_3x2` for 186/187 (the Boulder refusal
`:38979-38994`, the 187→186 conversion `:49379-49393`) and `check_on_table_1x1`
for the clay pot (`:43503-43546`) — and the drop layer ports the herb harvest
matrix (`:65674-65698`, `IsAlchemyPlantHarvestable` `:66143-66174`), the full
`SpawnThingsFromPot` roll order (`:57089-57811`) and the harvest click
(`AdjustPlants`/`TryReplantingHerbs`, `Player.cs:41260-41293` / `:45824-45830`).
Deviations and seams:

- The RNG seam (owner decision 2026-11-05): the C# two-stream split survives, but
  neither stream is vanilla's — the event's `genRand` is a `FastRandom` seeded by
  ONE `item_rng` throw per event (a broken pot, a killed herb), and every
  `Main.rand` roll spends `item_rng` directly; the roll order is C# 1:1. This is
  the `CloseDoor` pinned-0 precedent (above) generalised: no free-running runtime
  stream exists, so a repeated seed reproduces the drops byte-for-byte.
- The special-world flags have no port state: `DropCtx` hardcodes
  `expert_mode: false` / `dont_starve: false` (`drop_ctx`; the tests flip them),
  the tenthAnniversary/getGood/remix/vampireSeed branches skip WITHOUT consuming
  rolls (the C# `&&` short-circuit keeps the streams aligned), the boss-downed
  coin multipliers ×1.1 read all-false, and the pot luck is 0 (one
  `Main.rand.Next(range)` per `Luck.RollLuck`).
- The Coin Portal (518) is a roll without a projectile: the luck roll is spent
  and the break returns as in C#, but `spawn_with_data` declines kind 518 (no
  projectile family ported yet) — the portal pixels wait for the M4-PROJ
  families.
- The pot gore shards (`Gore.NewGore` `:56878-56953`) are deferred — the port has
  no general gore mechanism (only the leaf `LeafGore`); the genRand style rolls
  (25-33) have no counterpart, and the per-event `FastRandom` re-seed keeps the
  loot rolls independent of the skip.
- The echo tiles 653/647-650 and BeachPiles 324 are deferred (owner decision
  2026-11-05); `CheckPot`'s only item drop IS the echo 653 (`:56955-57009`), so
  the port's `check_pot` returns no `drop` and the pot loot rides the event only.
- The sunflower `RandomStyleRange` roll (`Main.rand.Next(3)`, `TileObject.cs:787`)
  is not wired to placement: an item-placed sunflower always resolves style 0
  (the documented follow-up in the module doc).
- The replant anchor 579 (`RockGolemHead`) is carried text-for-text from
  `Player.cs:45826` — nothing in the ported game plants on 579; the decompile's
  condition is kept verbatim.
- The coin-shrink roulettes run on `Main.rand` in C# (not `genRand`, `:57751+`) —
  the plan text was imprecise; the port matches the source.
- The core kill contract grew additively: `ObjectDrop.item`, `TileKill.events`/
  `herb_events`/`pile_events` and the `HerbEvent`/`PileLootEvent` types (the new
  fields ride the existing `TileKill`, no signature broke) — outside phase 3's
  original file scope, a discovery decision.
- The `destroyObject` guard is per-check like the C#: only the multi-tile checks
  carry the entry guard (`Check1x2`/`2x1`/`3x2`/`Sunflower`/`Pot`/`Chest`/the door
  pair); `CheckAlch`, `CheckOnTable1x1` and `CheckPile`'s 1x1 body run unguarded,
  and their nested kills pass the flag through unchanged.

Verification (M1.4-TO-4, batch review 2026-11-05, HEAD `f9dd9f8`): the workspace
suite is green (`test result: ok` ×7 — 789+16+499+19 passed, 0 failed, 1 ignored,
`--skip migrates_version_2_kinds_to_item_ids`); the deterministic `TR_SHOT` scene
(`TR_TILES` staging 27/28/78/82-84/185-187/380 on a platform) gives SHA256-equal
double-run pairs at zooms 1.0/1.5/2.0 (`f2072783…`, `9e8f5e6d…`, `cd0502a5…`;
commit `9ec6f2e`), and the sway pair (`TR_SHOT_FRAME` 60 vs 480 at
`TR_WIND_FORCE=0.8`) differs in the flower zone; unit tests carry the geometry
(`test_DW_1_2_herb_pot_pile_geometry_matches_csharp`), the cascade scenarios and
the pot-loot roll order (fixed-seed `item_rng`).

## Slime variants and the demon-eye family (M4-NPC-1-slimes-eyes)

- The four hand-tuned NPC stats moved to the `SetDefaults` values: blue slime
  24x18 / 25 HP / 7 damage / defense 2 (`NPC.cs:8567-8580`; was 30x24/30/12),
  demon eye 30x32 / 60 / 18 / defense 2 / knockBackResist 0.8 (case 2; was
  30x26/25/14). Wandering Eye (133) joins the `aiStyle 2` machine with 4
  frames (`Main.cs:67082`) and 300/40/20; it is hardmode-gated in the night
  spawn roll (`NPC.cs:4555-4558`), so the pre-hardmode ambient pass never
  spawns it — the kind exists for the future M5-HARDFORCE.
- The slime variants are the negative net ids (`Lang.cs:520-533` names,
  `NPC.SetDefaultsFromNetId` stats): green -3, pinky -4, baby -5, black -6,
  purple -7, red -8, yellow -9. The hitbox rides the per-variant size scale,
  the sprite is the 32x26 frame scaled and anchored at the frame centre over
  the hitbox centre (`Main.DrawNPC`'s `origin = frame.Size() / 2`), tinted by
  the per-variant `NPC.color` through the premultiplied pipeline (the alpha
  channel renders the vanilla translucency).
- The ambient selection ports `Spawner.GetBasicSlimeToSpawn`
  (`NPC.cs:5625-5654`): underground 1/5 yellow, else 1/2 blue / red; forest
  surface 1/3 green (or within 200 tiles of spawn), 1/10 purple beyond 400
  tiles, else blue; then `Spawner.SpawnNPC`'s `RollLuck(180)` swap of any blue
  slime for Pinky (`NPC.cs:5238-5246`; the port rolls it un-lucked). Jungle
  tile 60 (-10) and snow 147/161 stay out until those biomes exist; black and
  baby slimes have no ambient roll in the forest branch.
- The loot follows `ItemDropDatabase.cs:1074-1082`: standard slimes (blue
  family) always drop 1-2 Gel, red/purple/black 2-5, green/baby/pinky none
  (Pinky drops 25-50 Pink Gel instead, `:1072`). Gel (23) and Pink Gel (3111)
  are registered as materials (empty `SetDefaults` cases -> `ResetStats`
  defaults: no use timings).
- The death check keeps the vanilla semantics: `life` is not clamped, death
  is `life <= 0` (`NPC.checkDead`).
- The coin drops keep the port's pre-existing denominations (the `value`
  copper amounts are a separate M4-LOOT row).

## The item drop rule engine (M4-LOOT-1)

- `tr-core/src/loot.rs` ports the `ItemDropRules` machinery: the
  `IItemDropRule` trait (+ `INestedItemDropRule` as `try_nested`), the three
  chain attempts (`Chains.cs` OnSuccess/OnFailedRoll/OnFailedConditions),
  `ItemDropResolver.ResolveRule`/`TryDropping`/`ResolveRuleChains`
  (`ItemDropResolver.cs:14-62`), `ItemDropDatabase`
  (register/register-multiple/remove-multiple), and the factories:
  `CommonDrop`, `CommonDropWithRerolls`, `OneFromOptionsDropRule`,
  `ItemDropWithConditionRule`, `DropBasedOnExpertMode`/`MasterMode`,
  `DropLocalPerClientAndResetsNPCMoneyTo0` (the `npc.value = 0` side effect
  becomes a `resets_money` flag on the [`DropRequest`]),
  `DropOneByOne`, `DropBasedOnExtraGel`, `DropNothing`.
- `Luck.cs`'s five rolls (`RollLuck`/`RollBadLuck`/`RollOnlyBadLuck`/
  `RollBadLuckExtreme`/`RollOnlyBadLuckExtreme`) port verbatim — the shrink /
  widen branches consume the stream exactly when the gate float passes.
- The engine is RNG-generic (`DropRng` = `Next(min, max)` exclusive +
  `NextFloat`); the app drives it from the same deterministic item stream as
  the coin rolls. `ReportDroprates` (the bestiary rate report) is not ported —
  M4-BEST-1 territory.
- The slime family's loot moved from the hand-rolled `spawn_gel` rolls to a
  real database slice (`crates/tr-app/src/loot_db.rs`): blue 1-2 gel always,
  red/purple/black 2-5, pinky 25-50 Pink Gel + `NormalvsExpert(1309, 100, 70)`
  (the Bladed Glove request is produced and then skipped until its item data
  registers), green/baby nothing. The executor drops only registry-known
  items.

## The Martian Madness (M4-EVENT-4)

The framework's fourth slice: `StartInvasion(4)` (`Main.cs:64463-64464`)
overrides the size to **160+40x** (the base 80+40x replaced — the
C#'s assignment, not a bonus); `Main.InvasionWarning`'s type-4 literals;
the defeat ride sets the new `World.downed_martians`
(`SetEventFlagCleared(ref downedMartians, 3)`).

The spawn table (`NPC.cs:1487-1540`) ports with the ground-four arms:
the else arm's GrayGrunt 385/GigaZapper 389/Officer 383 (the num9
Next(4) roll) and the num6<4 fallback. The remaining arms — the Saucer
395 (the Next(45) + the 30% progress gate), the ScutlixRider 390/the
RayGunner 382/the BrainScrambler 381 (num6 >= 6/4), the Drone 388 (the
aiStyle-74 flyer) and the Walker 520's walk-in gate (`!AnyNPCs(520)`)
— land with their AI families (the Saucer boss, the mounted Scutlix,
the ray flyer). The Martian Probe 399 trigger (the scout's
escape-when-spotted mechanic) is the event's entry — the follow-up.

The five kinds carry their verbatim `SetDefaults` rows (the Officer's
300-HP/def-50 body, the GrayGrunt's 750-HP/0-kb bruiser, the walker's
34×110 2000-HP body) and the 9/11-frame sheets NPC_383/520.

## The Martian Probe (M4-EVENT-4b)

The event's entry. `AI()`'s aiStyle-80 block (`NPC.cs:36532-36624`):
`ai[0] == 0` hovers (the 3·direction drift with the collideX flip, the
30-tile vertical scan's 15/20 rise-damp-sink bands); `ai[0] == 1` is
the 60-tick detect flash (the closest player within 352 px whose centre
sits below the probe); `ai[0] == 2` escapes (the `noTileCollide` rise
− 0.1/tick to −10, the `ai[3]` x-drift away from the player, the
leave-world/180-tick arm's `StartInvasion(4); active = false`). The AI
holds `&World`, so the resolve rides `wants_martian_invasion` — the sim
starts the invasion (the type-4's 160+40x army) and removes the probe.
`CheckActive`'s timeout latch is constants-off (no NPC timeLeft model).

Both spawn sites: the skyMob dispatch (`NPC.cs:1354-1412` — the
`FindSpawnTile` 0.35/0.45 bands' `skyMob` flag with the `Next(10)` roll,
`flag5`'s edge band + `skyBehindPlayer`, the water-candle 3/10 vs 8/30
tables, the `!downedMartians` short-circuit rolls, `AnyDanger`'s
refusal; the Drone arm consumes the dispatch position and the Wyvern arm
consumes its `Next(10)`, both returning nothing — the kinds land with
their AI families) and the surfaceSpawn arm (`:4164-4199`, the 100/400
rolls). The vanilla registers NO loot for the probe (neither
`ItemDropDatabase` nor `NPCLoot`), so killing it drops nothing and does
not trigger anything — only the escape does.

The task also landed a reconciliation pass over the node's parallel
slices: the `to_u16` id map's pirate/martian collision (both families
had taken 68-72 across merges) renumbered to 78-82/83 through all three
tables, the duplicated dead arms removed, the bestiary `sheet_i` table
re-derived from `bestiary_npc_sheets`' order at once (the frost trio's
48-50 slot had silently drifted every later agent's rows), the Walker's
11-frame slice corrected to the vanilla 8, and `FindSpawnTile`'s 0.45/
0.55 middle cut ported.

## The Martian Drone (M4-EVENT-4c)

The type-4 sky mob (`NPC.cs:13935-13953`): 40×30, dmg 60, def 16, 300
HP, kb 0.4, npcSlots 1.5, noTileCollide + noGravity.
`AI()`'s aiStyle-74 block (`:35218-35500`, the 388 parameter set):
`ai[0] == 0` hovers — the approach vector points 200 px above the player
at speed 10, damped into the velocity over 30 ticks, gated by the
line-of-sight + the ±π/8 angle cone (the cone only opens downward — the
drone dives under the players it cannot see, with the 120-tick argue
override); `ai[0] == 1` charges 30 ticks (the ×0.95 damp, then the
±50×0.04 jittered vector ×14); `ai[0] == 2` dashes (the 20-lerp toward
the player, the exit at velocity < 7 or the 100-px-past + 30-tick
window, the solid-collision fold to the explosion); and within 64 px of
the player the machine folds to `ai[0] == 3` — the 192×192 box, the 80
damage, the 3-tick fuse. The loot db registers no 388 rules, so the
explosion drops nothing.

The spawn: the skyMob dispatch's drone arm (`:1395-1398` — `invaders &&
invasionType == 4` → the Drone at the candidate tile) runs through the
spawner verbatim, and the ground table's roll chain is complete now:
`num6` `Next(7)` → the Saucer gate (`Next(45)` + the 30% progress +
`!AnyNPCs(395)`) → the num6 >= 6 rider arm (`Next(20)` + the gate → the
unported Scutlix Rider; the `Next(2)` split → the Engineer 386) → the
num8 `Next(5)` ray arm (the RayGunner 382 / Scrambler 381 unported,
`>= 4` → the Drone 388) → the else arm's ground chain (the `Next(4)`
with the 520 walk-in gate's `Next(3)` fallback). Every roll consumes
the stream where the C#'s short-circuit reaches it; the four unported
kinds' hits map to the ground fallback until their AI families land.

## The martian riders (M4-EVENT-4d)

The 381 BrainScrambler / 382 RayGunner pair (the grouped `SetDefaults`
row: 18×40, dmg 50, def 25, 350 HP, kb 0.35) walk through the walker
step and run `AI_003_Fighters`'s shooter block (`NPC.cs:59069-59623`):
the stop entry (an aim under 700 px with `ai[2] <= 0` and `ai[1] <= 0`
arms the 80-tick timer and the aim octant), the fire at `ai[1] == 40`
(the ±100×0.4 jitter re-scaled ×(0.85..1.15), the spawn at the centre
+ 6 — the 436 ray at 24 damage and speed 11 for the Scrambler, the
438 at 30 and 7 for the RayGunner), the hurt interrupt (the 30-tick
rest), the sleep's exit. The vanilla's `stealth`/`itemAnimation`
aggression gate is constants-off (the port's Player carries no swing
timer).

The 390 ScutlixRider (`AI_075`'s type-390 arm, `:35506-35745`) is a
MOUNT machine: it spawns its 391 Scutlix carrier once through the
sim (`wants_scutlix` → the carrier at the rider's feet), rides 14 px
above its centre with the carrier's velocity, and fires the same 438
ray at a 60/−60 cycle (the 700-px window, the carrier-direction match,
the ±50×(0.8..1.2) aim). The port has no NPC indices, so the link is
the sim's write-through (the WoF shape) in a pre-pass — the vanilla
reads the carrier's live centre inside its own pass, the port's is one
tick earlier (documented). The rider falls with its carrier: the
`timeLeft` latch has no port model, so the retirement rides the
carrier's death instead of the 750-tick decay.

The type-4 table is complete but for the Saucer 395 family: the
`num8` arm ports the RayGunner (<2) / the BrainScrambler (==2) / the
Drone (>=4), and the num6 >= 6 arm's `Next(2)` split ports the Rider.

## The Martian Saucer (M4-EVENT-4e)

The type-4 event's boss: the 395 Core (dmg 80, 10000 HP, aiStyle 76,
dontTakeDamage) plus its crew — the 392 Shell (150×80), the 393 Turrets
(40×16, 5000 HP) and the 394 Cannons (46×36, 3500 HP), all aiStyle-75
mounts. The core's init spawns the crew at its centre with the seat
`ai[1]`; the mounts ride the core (the shell at UnitY+2, the turrets at
UnitY+29 ± UnitX·60, the cannons at UnitY−13 ± UnitX·49) through the
pre-pass write-through link.

`ai[0]`'s states: the ±1/−2 despawn rides (the player out of the 5600-px
window), the 600-tick SCAN cycle (the six step bands — approach,
dive, park, brake — with the ground-clearance reads easing the dive,
the 20/260/279/439 force-jumps on the approach bands), the vulnerable
WOBBLE (150 ticks of the rotation ramp — the classic-only gate the
shell's bridge drives), the GUN (the 3600-tick cycle: the 60-tick
firing window per 120 fires the 447 projectile at 80 damage; the
8·ai[2] dash in the window, the 16-lerp outside), and the phase-3
death ride (`StrikeNPCNoInteraction(9999)` → the port's hp=0).

The type-4 table is now complete — every kind in `NPC.cs:1487-1540`
ports. The shell's missile battery and the core phase bridge (the
`flag80` classic/expert split) are the follow-ups: until they land,
the core stays in the scan/gun loop and is killed by damage.

## The saucer's guns (M4-EVENT-4f)

The mounts' batteries ride the core's mirrored `ai[3]` timer and `ai[0]`
phase through the pre-pass write-through (the vanilla reads
`Main.npc[localAI[i]]`'s sibling state directly — the port approximates
the crew's aliveness with the pre-pass counts). The 393 turret aims
with the 8-way table (the player vector's rotation, Y-clamped, mirrored
by the spriteDirection) between its 280-420 windows and fires the 449
rays (35, 16 speed, ±0.785/3 jitter) every 6 ticks inside; the 394
cannon fires the 448 shells (50, 8 speed, side·36 + 8 spawn, ±0.785
aim) every 20 ticks of its 440-580 window; the 392 shell fires the 447
entry shot (50) at the phase-0 window's edge and the 450 missile
battery (30, 3 speed — the turret side's wide ±0.26 arcs and the cannon
side's narrow ±0.785 arcs) in the windows: the core's gun phase (2)
opens them always, else the 280-400 and 440-580 bands.

The bridge: with the crew dead and the core open (the classic arm),
the shell drives the core's `ai[0] = 3` — the death ride — so a saucer
whose guns die falls out of the fight on its own.

### The EoC task's loss and re-land (M4-NPC-3n)

The smart-cursor agent's `5075a462` "sync" merge resolved its conflicts by
taking its own branch's wholesale tree, which predated `07c1d4e4` — main
lost the entire M4-NPC-3m task (the case-4 `find_frame`, the 110x166
frames, the `halfSize` override, the `eoc` hook name and the ROADMAP
row). The AGENTS.md hazard ("another agent's merge reverts a finished
feature — verify your markers after every merge") caught it in the audit
log (`git show origin/main:... | grep -c`), and the work was cherry-picked
back on this branch. The lesson joins the 2026-11-08 incidents list: a
"sync" merge whose message says it restores dropped work is exactly where
someone else's finished work gets dropped again.

## The draw-anchor overrides (M4-NPC-3n)

`Main.cs:23048`'s switch overrides the per-kind draw anchor in two ways,
and the port now models both:

- `halfSize` (the origin point that lands on the anchor) shifts the frame
  by `override − frame_h / 2`: the EoC/Twins' (55, 107) gives +24/+7, and
  the Plantera head's absolute 77 (over its 86-px row's 43) gives +34.
- `num47`/`num46` are flat pixel nudges: the Plantera head's +26, the
  Brain's `+50 · scale`, the DD2 group's negative shifts (the
  javelineers' −8, the bombers' −5, the Wither Beasts' −3).

`npc_anchor_nudge` sums them and is applied once after the draw match, so
every generic-anchored kind takes its own value; the towers, the saucer
family and the Deerclops compute their own positions in dedicated
`DrawNPCDirect` arms and are excluded, exactly like the C#. The still
unported arms (626/627's wet dual switch, 692's `num46 = wet ? 10 : 6`,
688/694's direction terms, 469's ai[2]-gated +14, and the DD2 sprite
effects/flip toggles) are the follow-ups.

## The Eye of Cthulhu's display (M4-NPC-3m)

The EoC's `find_frame` arm ran an 8-tick strip over all six rows of its
sheet, and its comment described a "pupil blink" that does not exist in
the decompile. `case 4/125/126` (`NPC.cs:76905-76926`) is ONE shared
machine — the Eye of Cthulhu, Retinazer and Spazmatism all ride it:

- The 20-tick counter walks the bands rows 0/1/2 (splits at 7/14/21,
  the 21st tick resetting), and `ai[0] > 1` offsets every row +3 onto the
  phase-2 rows 3/4/5. The Twins ported this in M4-NPC-3k; the EoC was
  still on the v1 strip, so the boss and its summonable twins animated
  differently.
- **The sheet is 110x996** (six 166-px rows, `Main.npcFrameCount[4] = 6`)
  and the v1 squeezed each row into a 110-px quad at the box's corner —
  a horizontally stretched, vertically cropped eye. `boss_draw` now
  takes the real 110x166 frames at the generic anchor (the same shape
  the Twins ride, whose 200-px rows come off the same helper).
- **The `halfSize` override** (`Main.cs:23051-23056`): the C#'s draw sets
  `halfSize = (55, 107)` for all three kinds — the point that lands on
  the NPC's anchor. For a 166-px row the sheet-derived half is 83, so the
  override pushes the frame's centre 24 px lower (the Twins' 200-px rows:
  7 px). `boss_half_size` folds that into the anchor as
  `override − frame_h/2`, which is the shift the C#'s formula produces.
- The rest of the C#'s override switch (the Plantera head's absolute 77
  with `num47 += 26`, the Brain's `+50·scale`, the small walkers' +4/+8,
  the DD2 families' negative nudges, the 626/627 wet switch) stays the
  documented follow-up — the `case 4/125/126` trio is the only override
  that is a plain (55, 107).

Screenshot: `eoc_eye.png` (the eye at its real 110x166 frame mid-flight),
SHA256-stable across runs.

## The WoF family and the WoF-side mobs (M4-NPC-3r)

- **`case 113/114`** (`NPC.cs:73516-73537`) is shared by the Wall of Flesh
  and its mirror eye: the plain phase (`ai[2] == 0`) cycles the 2-row strip
  every 12 ticks; every other phase pins row 0 with the counter parked at
  -60. The v1's "single frame — no case" comments hid a real animation
  (the WoF's mouth).
- **`case 115`** (the Hungry, `NPC.cs:74995-75017`): the 3-row strip rides
  a 15-tick sequence 0/1/2/1 — the bands at 3/6/12/15 with the counter
  reset landing exactly on the 15th tick (the row catches up on the next).
- **The Skeletron head/hand** (`case 35`/`case 36`, `NPC.cs:66767-66778`)
  have no normal-state animation at all: their only case arm pins row 1
  under the red-hat (Halloween) adjustment, so row 0 always rides in this
  port. The sheets are 2-row (the red-hat row is the second).
- Sheets measured and drawn through `boss_draw`: NPC_113 142x212,
  NPC_114 130x212, NPC_35 114x264, NPC_36 58x156, NPC_5 20x64, NPC_115
  44x114, NPC_117 18x30, NPC_134 50x102 — the v1 either read `t.h` as the
  frame height (a whole-strip "frame") or drew box-sized quads at the box
  corner.

Screenshot: `hungry_wof.png` (the Hungry at its real 44x38 frame),
SHA256-stable across three runs.

## The King Slime (M4-NPC-3q)

The King Slime's draw carried the right numbers as v1 magic: `[n.x - 38.0,
n.y - 25.0]` over a hard-coded `120.0` frame height, which is the generic
anchor (`[x + 49 - 87, y + 92 + 4 - 60]`) plus a 1-px fudge. Replaced with
`boss_draw`'s derived 174x120 rows (NPC_50 measured 174x720, six 120-px
rows over the 98x92 box), so the sheet's real geometry drives the draw.
`case 50`'s machine (`NPC.cs:73380-73414`) verified 1:1: the air snaps to
row 4 and holds 5, the landing falls back to 4, the grounded walk rides
0..3 with the `num2` boosts.

Screenshot: `king_slime.png` (the slime at its real 174x120 frame standing
on the grass), SHA256-stable across three runs.

## The Plantera and the Brain (M4-NPC-3o)

`case 262` (Plantera) and `case 266` (the Brain of Cthulhu) are the same
machine in the decompile — a 6-tick advance over the 8-row strip with the
first phase wrapping at row 3 and the second clamping into 4..7 — and both
npcFrameCount entries the port's comments quoted were wrong ([262] = [266]
= 8, not 5 and 1). The v1 pinned the Brain to row 0 with `frame_h = t.h`
(a whole-strip "frame") and cycled Plantera over rows 0..4 with `t.h / 5`.

- The shared `phase_frame` helper takes the gate per kind: the Plantera's
  `life > lifeMax / 2` and the Brain's `ai[0] >= 0 && localAI[2] == 0`.
  The port's brain machine does not model the creeper-shield count, so the
  Brain rides the shield-down rows 0..3 and the shielded 4..7 rows are the
  documented follow-up (same honesty as the EoC's mech-queen rows).
- Measured sheets: NPC_262 116x1232 (154-px rows), NPC_266 200x1456
  (182-px rows), NPC_267 34x34. All three draw at the generic anchor
  through `boss_draw`.
- The anchor overrides from M4-NPC-3n are now applied in ONE place after
  the draw match, gated by `uses_generic_anchor` — the Plantera's
  (halfSize 77, +26) and the Brain's (+50·scale) shift their real frames,
  while the towers, the saucer family and the Deerclops keep their own
  dedicated `DrawNPCDirect` formulas.

Screenshots: `plantera_flower.png` (the flower at its 116x154 frame) and
`brain_creepers.png` (the 200x182 brain), both SHA256-stable across runs.

## The Lunar pillars' display (M4-NPC-3l)

The four celestial pillars had the third instance of the atlas bug and a
shared-sheet bug: none of NPC_493/507/517 was in the load list (so every
pillar drew nothing — the bestiary portraits were blank too), and all
three kinds resolved through one `towers` field bound to NPC_493, so the
Nebula and Solar pillars rendered the Stardust artwork at a guessed
130x270 in the box's corner. Against `Main.cs:24159-24269`:

- **The dedicated arm**: the pillars draw their single 174x364 frame
  (`Main.npcFrameCount` 422/493/507/517 = 1) centred on the NPC's Centre
  with `origin = halfSize` — not the generic bottom anchor, which would
  sink the frame's 47-px vertical overhang into the ground. The new
  `tower_draw` computes it; the frame overhangs the 130x270 box by ~22 px
  horizontally and ~47 px vertically.
- **One sheet per kind**: Stardust 493, Nebula 507, Solar 517 — and the
  **missing Vortex pillar** (422, the fourth id in
  `WorldGen.TriggerLunarApocalypse`'s shuffle) now exists end-to-end: the
  kind, its stats (the same 20000/20/0/130x270 row as its siblings), the
  shared aiStyle-94 machine, its `downed_tower_vortex`/`tower_active_vortex`
  pair (the World fields already carried them, unset by any death until
  now), the bestiary entry, the netglue spawn and the debuff immunity.
- **The glow ghosts**: each pillar's arm draws four masked copies of its
  own sheet at `velocity.RotatedBy(i·π/2) · offset` — the offset
  `4 + |light − 0.5|·4` (the Solar's `2 + ·9` plus an extra +X 2). The
  port renders them with the glow sheets' own alpha; the vanilla's
  (64,64,64,0)·Opacity mask tint is a no-op in a premultiplied pipeline
  (the same trade the Prime masks made).
- **The pillars never flip** (no `spriteDirection` write in aiStyle 94),
  so they join the `flips_on_sprite_dir` predicate like the martians and
  the Prime family.
- **Two identity bugs fixed on the way**: the Cultist's bestiary entry
  carried the Vortex pillar's id 422 (the C#'s `CultistBoss` is 439) —
  corrected, with the draw keeping the NPC_422 stand-in art since the port
  ships no NPC_439 sheet; and the ui portraits' hand-maintained sheet
  indices had drifted twice (the towers read slots 104-107 instead of the
  true 110-113, the shards 19 rows off) — the index is now derived from
  the entry's `net_id` through `BESTIARY_NPC_SHEET_IDS` +
  `Lookups::bestiary_sheet_index`, which removes the drift class for
  good.
- Deviations: the Perlin force-field shield bubble
  (`Main.cs:24239-24269`, a shader pass the port has no seam for) and the
  towers' 180-tick death-fade dust (`NPC.cs:38970-39068`) stay
  unmodelled, as does `WorldGen.TriggerLunarApocalypse` itself (the
  pillars' spawn).

Screenshots: `towers_solar.png` and `towers_vortex.png` (distinct per-kind
art at the real 174x364 frames), both SHA256-stable across runs.

## The Twins' display (M4-NPC-3k)

Retinazer and Spazmatism had the same v1 shape as the martians did: a
blind 8-tick `(frame + 1) % 2` strip whose comment claimed
`npcFrameCount[125/126] = 2`, drawn with `frame_h = t.h / 2` (a 600-px
slice of the sheet as "one frame") over the 100x110 box at the box's
corner. Against the decompile:

- **The sheets are 6 rows of 200** (NPC_125/126 measured 110x1200,
  `Main.npcFrameCount[125/126] = 6`, `Main.cs:67096`) — the v1's count
  was wrong and its draw read a 600-px slice.
- **`case 125/126`** (`NPC.cs:76906-76924`) is a 20-tick beat, not an
  8-tick strip: the counter walks the bands 0/1/2 (splits at 7/14/21,
  the 21st tick resetting), and `ai[0] > 1` offsets every row +3 — the
  pair rides the live rows 3/4/5 from its first anchored tick onward.
- **`ai[0]` is the AI's hover anchor** (`AI_022`'s `NPC.cs:24758`:
  `ai[0] = position.X`), not a phase counter — the port's simplified
  hover writes it once per tick so the offset engages; the pre-anchor
  spawn ticks show the spawn rows 0/1/2, exactly like the vanilla.
- **The Twins never flip**: the aiStyle-22 block has no
  `spriteDirection` arm (the 24893 write belongs to type 169's eye), so
  the pair keeps the `SetDefaults` default -1 and joins the
  `flips_on_sprite_dir` predicate (with the martians and the Prime
  family) instead of flipping on the motion `facing`.
- The laser/spit phases and the second-form transformations stay the AI
  follow-ups; the port's `update_twin` is still the v1 EoC-cycle shape.

Screenshot: `twins_family.png` (both eyes at their real 110x200 frames
on the live rows), SHA256-stable across runs.

## The Prime family's display (M4-NPC-3j)

Skeletron Prime's head and four limbs had the port's earliest stub: the
head's `find_frame` arm was empty (its comment even claimed
`npcFrameCount[127] = 1`) and the four limbs all forced row 0 as "the
limbs' single frame". Against `NPC.cs:74498-74570`:

- **The head** (`case 127`) rides a 6-row 156-px strip: the hover phase
  alternates rows 0/1 every 12 ticks, the enraged/day/despawn phases hold
  the wide row 2, and the Mech-Queen variant (the head's `ai[3] != 0`)
  alternates 3/4 and pins 5.
- **The saw** (`case 129`) and **the vice** (`case 130`) spin 2-row strips
  at 2 and 8 ticks with the wrap at `npcFrameCount`'s 2, and both write
  `spriteDirection = direction` while grounded — on top of the AI's own
  `-(int)ai[0]` seat write, which the port's `sprite_dir` field now
  carries (the flip rides it, `Main.cs:23124-23128`).
- **The cannon and the laser have no `FindFrame` case at all** — row 0
  forever, matching the v1's outcome but now with the citation (a missing
  case with a tall sheet is exactly how the martian family's
  whole-strip-as-one-frame bug would look).
- **The motion was written nowhere**: `AI_032`'s head roll
  (`rotation = velocity.X / 15` in the hover, `direction * 0.3` in the
  charge/day, the `AngleLerp` easing for the queen) and every limb's
  `atan2` rotation (the saw/cannon/vice aim their "up" along the dock
  vector at +π/2 — the vice's rise at the wider 280 dock — and the laser
  aims at the player at -π/2) are ported; the head and the limbs joined
  the draw's rotation branch.
- **The draws took the whole strip as one frame** (`frame_h = t.h` over
  box-sized quads at the box's corner). Now the sheets' real rows
  (140x156, 38x76, 38x78, 40x70, 40x62) draw at the generic anchor, and
  the two glow masks (`Bone_Eyes` over the head, `Bone_Laser` over the
  laser, `Main.cs:25585-25642`) ride the body's own frame/rotation/anchor
  as a second pass — both sheets were never packed into the atlas, so the
  masks had nothing to draw from (the port renders the overlays with the
  sheets' own alpha: the vanilla's (200,200,200,0) mask tint is a no-op in
  a premultiplied pipeline).
- **The enabling half**: the head's init request (`wants_limbs`,
  `NPC.cs:27635-27661`) was set on the first tick and never drained — no
  limb existed outside the direct-call test. The sim resolves it now with
  the vanilla's seat signs (`ai[0]` -1/+1/-1/+1, the vice and laser with
  `ai[3] = 150`), and `TR_BOSS=prime` spawns the whole boss for the shots.
- Deviations: the Mech-Queen's `Chain12` links (`Main.cs:25590-25630`)
  and the family's `TrailingMode 6` after-images (`Main.cs:25124-25136`)
  stay unmodelled (the port has no trail pass); the glow masks' tint is
  the alpha-carried equivalent noted above.

Screenshots: `prime_family.png` (the head over the four limbs, all at
their real frame sizes) and `prime_limbs.png` (the dock-rotated limbs),
both SHA256-stable across runs.

## The martians' display (M4-NPC-3i)

The Martian Madness family's 16 kinds had a `find_frame` arm each and a
draw arm each, but both were the v1's blind `(frame + 1) % count` strip
and a box-sized quad at the hitbox corner. Against the C#:

- **The ground group's walk band** (`case 381/382/383/385/389`,
  `NPC.cs:71623-71662`) is not a free-running strip: the row clamps up to
  the band's floor 2, the `|vx| * 2` accumulator (the Gray Grunt's and the
  Gigazapper's 1.2) advances every 6, the end wraps at `npcFrameCount`'s 9
  (row 9 never shows), the idle snaps to row 0 and the air holds row 1 —
  and `spriteDirection = direction` only while grounded.
- **The engineer** (`case 386`) parks on rows 9/10 while `ai[2] > 0`
  (`ai[2] / 12 % 2` — the work animation), and its walk wraps at 9, not at
  the sheet's 11 rows.
- **The walker** (`case 520`) wraps to row 1 (its band's floor is 1, not
  2) and its slow walk (`|vx| < 0.2`) parks on the idle row only when an
  advance tick fires.
- **The Scutlix pair inverts.** Cases 391/390 write
  `spriteDirection = -direction` (`NPC.cs:71746-71753`/`:71795`): the
  carrier walks rows 1..5 at `|vx| * 0.5` with the AIR row 0 (not 1), and
  the rider's row is purely `ai[2]`-driven (the aim octant 1..5, row 2
  when it holds 0). The port folded `direction` into `facing`, so the
  martian kinds now carry the separate `Npc.sprite_dir` field
  (`NPC.cs:6515`'s default -1) that the draws flip on
  (`Main.cs:23124-23128`: `spriteDirection == 1` is the flip) — without it
  the Scutlix family drew mirrored.
- **The saucer family's rows ride the core's `ai[3]` clock**, which the
  mounts mirror (`:35832`/`:35860`/`:35906`): the core's
  `frameCounter % 66 / 6` (11 rows) double-increments and pins to
  [36, 54) during the [20, 260) attack window (rows 6-8); the shell's
  `% 12 / 3` (4 rows) doubles in the same window; the turret's 9 aim rows
  decode `ai[2]`'s rotation through the 5-band cascade with the +9 firing
  rows on the [280, 420) flash ticks (`(int)ai[3] % 6 <= 2`); the
  cannon's rows come off `ai[3] - 300` (idle 4, charge 4..7, the
  `(n/4) % 4` firing spin, the recoil tail).
- **The sheets were never packed.** `martian_walkers`/`martian_shooters`
  mapped four kinds onto `NPC_383` and two onto `NPC_382`, and none of the
  martian sheets were in the atlas load list — `sky_tex`'s `Option`
  returned `None` and the draw skipped, so **every martian in the family
  rendered as nothing at all** (the bestiary's icon cells were blank too).
  Now NPC_381/382/383/385/386/389/388/390/391/392/393/394/395/399/520
  (+ the Dutchman's 491/492, the same mount machine) are packed, one
  sheet per kind, and the draws take the sheets' real rows (48/46/46/44/
  46/46 … ) at the generic anchor instead of the 18x40 box.
- **The saucer family's draw block** (`Main.cs:24586-24612`) is its own
  path: the CENTRE-anchored `origin = halfSize` draw with the core's
  inherited `npc.rotation` (the whole family tilts as one body) and the
  turret's `UnitY * -8` origin lift. The pre-pass now carries the core's
  rotation and `spriteDirection` through `carrier_rot`/`carrier_facing`;
  the turret's `ai[2]` aim (`:35873-35880` — the player vector's
  rotation, Y-clamped at 0) is ported, so the 18-row sheet aims at the
  player instead of pinning row 0.
- **Two live deviations found on the way**: the core's rotation machine
  ran `sin` where the C#'s `Vector2.UnitY.RotatedBy(t).Y` is a `cos`
  (`:36394-36409`), and the Dutchman's `rotation = velocity.X * 0.025`
  (`:38951`) was never written (its cannons and drop offsets read a
  permanent 0). Both landed.

Screenshots: `martians_ground.png` (the ground group's per-kind sheets and
walk bands) and `martians_saucer.png` (the core at its real 210x96 frame,
the probe, the drone, the Scutlix pair and the mounts), both
SHA256-stable across runs.

## The Flying Dutchman (M4-EVENT-5a)

The pirate invasion's ship (`NPC.cs:14632-14650`): 350×120, aiStyle 93,
dmg 0, def 100, 50 own HP, dontTakeDamage. Its machine
(`:38858-38956`): the init spawns 4 cannons 492 across its centre
(`seat*40 - 150`); the crew-alive loop self-strikes when all four are
down (the pending-crew gate keeps the ship alive through its spawn
tick — the vanilla's `ai[i]` links point at the children immediately);
every 300 ticks a pirate drops off the rotated side offset with the
±5 X + -8.01 Y launch; the flight holds the 350-450 ground-clearance
band ahead of the direction, the X drift engages past 300 px, and the
rotation rides `velocity.X * 0.025`.

The type-3 table's half-points window (`:1460-1468`) spawns the ship —
the pirate invasion gains its boss-minion. The port's link model: the
pre-pass counts the ship's live cannons and links the cannons' ride to
the ship (the vanilla's `Main.npc[ai[i]]` indices have no port model).

## The pirate trio's shots (M4-EVENT-5b)

The aiStyle-3 shooter block now generalises across the event families:
`shooter_params` carries the vanilla's per-kind cooldown/speed/
projectile/damage rows — the pirate Archer 214's arrow 180 (25, 10, the
±45 jitter), the Corsair 215's musket ball 82 (35, 14), the Captain
216's arrow 180 (30, 9, the ±50 jitter), beside the martians' 436/438.
The block's shape: the stop entry (an aim under 700 px arms the octant
and the 80-tick timer), the fire at the half-cooldown (the per-kind
jitter re-scaled ×(0.85..1.15), the spawn at the centre + 6 y, the
speed-scaled velocity), the hurt interrupt, the sleep's exit.

The 180/82 projectile rows join `proj_data` — the pool's data gate
previously dropped every kind outside its hand-listed set, so the
family's shots silently never spawned.

## The Captain's wounded barrage (M4-EVENT-5c)

The pirate Captain 216 counts his shots (`localAI[2]` — the port's
`caster_counter`); past 20 the wounded arm swaps the plain 180 arrow for
the 240 shell: 100 damage, 15 speed, no jitter, on the rolling
`max(60, 180 - localAI[3]*20)` cooldown (`localAI[3]` = the barrage
count, the port's `cast_windup`). The barrage resets the shot counter.
The 240 shell rides the Dutchman's data row.

## The DD2 goblins (M4-EVENT-5d)

The Old One's Army attackers (`NPC.cs`'s 552-554): the improved-walker
harness with the tiers' adds and the wave acceleration — the
`localAI[3]` ramp (below 60 the accel element is
`0.01 + localAI[3]/60*0.05`, past 60 it holds at the value) mirrors the
DD2 event's escalating pressure. The port's `caster_counter` stands in
for `localAI[3]` (the counter ticks with the Wave).

The DD2 framework (the crystal 548, the tavernkeep 550, the
`DD2Event.EnemiesShouldChasePlayers` chase flag, the wave tiers' bosses
— the Dark Mage 564, Betsy 551) is the next slice.

## The DD2 support families (M4-EVENT-5e)

The Old One's Army's second echelon: the kobold bombers (555-557, kb
0.1, walk 0.88/1.12), the javelin throwers (561-563, 26×52, the spear
projectiles 662/685 at the 82-tick cadence) and the etherian skeletons
(566-567 with the materialize cycle — `localAI[3]`'s spawn-in counter,
the alpha fade over the 110-tick run; the walk rides the harness while
translucent).

The families' walk arms ride the port's harness with the vanilla's
per-kind speed rows; the bombers' bomb throw, the throwers' spears and
the skeletons' fade render ride their own slices.

## The DD2 weapons (M4-EVENT-5f)

The javelin throwers' arm: the 82-tick cadence throws the 662 spear
(the tier-3's 685) at the classic 30/60/80 damages from the centre -
14, the ballistic aim at the player. The walk rides the harness with
the vanilla's fast row (speed 8, accel × 3). The 681 bomb's row joins
the pool's data set — the bomber's throw arm rides it when it lands.

## The kobold bombers (M4-EVENT-5g)

The bomber's lob: the 18-tick cadence throws the 681 bomb at the
tiered classic damages (20/30/40) from the centre - 14, the ballistic
aim at the player (the vanilla's `num10 = 6` speed — the bomb's own
aiStyle-16 flight carries the arc). The walk rides the harness with
the tiers' speeds and arc weights.

## The Dark Mage (M4-EVENT-5h)

The Old One's Army's caster (564): materializes in from the portal
(60 ticks of `localAI[3]`), hovers with the slow damped approach, and
runs the cast cycle on `ai[0]`'s negative rest / positive march: the
summon (`ai[1] == 2` at 64 — the 673 marker + `RaiseGoblins`), the
shadowflame (`ai[1] == 0` at 32 — the 675 at 40), the heal (`ai[1] == 1`
at 40/48/56 — the 674 markers), then the pattern advance and the
97/127/183 re-arm.

## Betsy (M4-EVENT-5i)

The dragon's v1: the spawn pause, the phase-1 approach to the
300/-200 offset past the player (7.5 speed / 0.45 accel), the phase
rotation's pick (0/1/3 → 2, 2 → 3, 4/6 → 5, the 8-wrap) and phase 2's
687 breath at 35. The fireball 686, the circle flight and the dash
ride the attack family's slices.

## Betsy's attacks (M4-EVENT-5j)

The phase arms verbatim: the wind pause, the flame breath (the 600-px
reposition + the 687 at 35), the fireball volley (the 1500-px
reposition, the rotated muzzle, the 686 at 35 every 10 ticks), the
circle flight and the dash.

## The Wither Beast (M4-EVENT-5k)

The ethereal caster-beast: the walk with its row (speed 1.4, damp 0.7)
and the aura phase — the pulse counter, the ring dust at the 400-px
radius, the 60-tick self-heal (`lifeMax / 20` capped to the missing
hp). The 683 crystal shots ride the shooter block's slice.

## The tavernkeeper + the etherian crystal (M4-EVENT-5m)

The Old One's Army's entry: the tavernkeeper (a town NPC) and the
Eternia Crystal (548) with its 180-tick spawn-in shield — the
`localAI[0]` counter holds `dontTakeDamage` until the crystal opens,
then the event's waves ride it. The summon arm (the crystal item over
the stand, `SummonCrystal`'s frame-offset spawn) is the seam to
`DD2Event`.

## The crystal summon (M4-EVENT-5n)

The Old One's Army's entry point: the crystal item over the stand runs
`SummonCrystal` — the gates, the frame math's spawn point, the
consume, `StartInvasion` + the crystal. `World.dd2_event_ongoing`/
`dd2_wave` carry the event's seam.

## The Old One's Army's waves (M4-EVENT-5o)

`tr-core/src/dd2.rs` ports the wave engine: the rosters (552/555/561/
558/564 by wave) with the lane rates (90→40), the budgets (60/80/100/
120/140, the wave-5 cap, the wave-6 win sentinel) and `CheckProgress`'s
advance — the hold's early refusal, the budget's pass → wave+1 with
the 1800-tick break.

## The crystal's win-death (M4-EVENT-5p)

The event's end: the crystal's `ai[0] >= 600` arm fires
`DD2Event.StopInvasion` + `life = 0` — the port's `wants_event_stop`,
resolved by `update_dd2_event` before the death retain (the flag and the
crystal go in one pass).

## The DD2 tiers (M4-EVENT-5q)

The tier-2/tier-3 rosters and budgets (`Difficulty_2/3_*`): the tiers
swap the rosters (the T2 heavy units 553/562/572/568, the T3 leet
554/557/563 with Betsy 551 on wave 7), the budgets climb 60→220, and
wave 7's tier-3 budget is Betsy's own HP bucket.

## The Ancient Cultist (M4-EVENT-6)

The Lunar Events' summoner (422, aiStyle 94): the 180-tick ritual
dissolve, the sine-bob hover, the far-player regen and the two ritual
projectile arms (the 3240-px line-of-sight gate, the 1080-px below
gate). The pillar summons (Stardust 493 / Nebula 507 / Solar 517 with
their shield strengths) and the boss bar ride the next slice.

## The pillar shields (M4-EVENT-7)

The Lunar towers' shield lifecycle on `World`: the raise
(`ShieldStrengthTowerMax` = 100, / 2 post-Moon-Lord), the spend
(per-tower drain, the downed flag) and the reset. The 629 pulse fires
from the towers' hit path.

## The invasion progress bar (M4-EVENT-1b)

The UI half of the invasion framework. The report:
`Main.ReportInvasionProgress` (`Main.cs:47687-47694`) rides the kill
credit — the points (`invasionSizeStart - invasionSize`), the max
(`invasionSizeStart`), the icon (the `GetNPCInvasionGroup` group + 3 —
the goblins' 4) and the 160-tick display timer.

The near-scan: `Main.CheckInvasionProgressDisplay`
(`Main.cs:47599-47649`) runs per tick — a living NPC whose
`GetNPCInvasionGroup` matches the active `invasionType` within the
±5000-px window around the player marks `invasion_progress_near` and
refreshes the report while the icon is set (the C#'s moon/DD2 arms are
their own slices). The bar draws through
`Main.DrawInvasionProgress`'s non-wave arm (`:47807-47830`): the
200×45 `DrawInvBG` panel at `(screenWidth - 120, screenHeight - 40)`,
the `ColorBar` track, the yellow fill (255,241,51), the orange 2-px
edge (255,165,0,127), the black rest and the `{percent}%` +
group-label text. The fade head (the display timer's decay + the 0.05
alpha ramp) rides the World tick; the icon's `Extra[7..12]` texture
rows are the follow-up.

The pipeline lesson: the insertion pass's doc-comment `{label}
{label}`
escapes — a raw newline split the `///` comment and the orphan
`{percent}%` line broke the parser (the "prefix is unknown" errors),
which cascaded into an "unterminated string" at the chat line. The
heredoc-written doc text now avoids inline `
` escapes.

## The Pirate Invasion (M4-EVENT-3)

The framework's third slice: `StartInvasion`'s pirate bonus (`Main.cs:
64460-64464`) — the size 80+40x **plus 40+20x** per qualified player
(the single player → 180 points); `Main.InvasionWarning`'s type-3
literals; the defeat ride sets the new `World.downed_pirates`
(`SetEventFlagCleared(ref downedPirates, 2)`).

The spawn table (`NPC.cs:1460-1484`): the Flying Dutchman 491 spawns
through the half-points window (`invasionSize < invasionSizeStart/2` +
the `SolidTiles` clear-scan) — the boss-minion follow-up; the port's
table carries the Captain 216 1/30 (while none alive), the Deadeye 1/11,
the Corsair 1/9, the Archer 1/7, the CaptainsMate 1/3, else the
Deckhand. The kill points ride `:78238-78241`'s num9 table — the
Captain's 5-point kill.

The summon: the Pirate Map 1315 through
`ItemCheck_UseBossSpawners`'s pirate arm (`Player.cs:43957-43971`) —
the same `CanStartInvasion(3, ignoreDelay: true)` + roar + march-state
shape the Snow Globe uses. The map's own drop gate rides the loot db's
`Cond::PirateMap` (the depth/ocean band condition already ported in
M4-LOOT-2).

The five kinds (PirateDeckhand 212/PirateCaptainsMate 213/PirateArcher
214/PirateCorsair 215/PirateCaptain 216) carry their verbatim
`SetDefaults` rows (18×40, the aiStyle-3 walker, the Captain's
3000-HP/50k-value boss-minion body) and the 16/20-frame sheets
NPC_212/216.

Deviations: the Flying Dutchman (the 350×120 aiStyle-93 ship boss) and
the Deadeye/Corsair ranged shots are the follow-ups; the Captain's
`GetNPCInvasionGroup` 5-point kill rides the port's num9 table.

## The Frost Legion (M4-EVENT-2)

The invasion framework's second slice: `StartInvasion(2)` generalizes
the start fn over the invasion type (the goblin slice passes 1), the
warning fn carries the type-2 literals (`Main.InvasionWarning`'s
`Lang.misc[4..7]` — the en strings stand in), the defeat ride sets the
new `World.downed_frost` (`SetEventFlagCleared(ref downedFrost, 1)`),
and the spawn table gets the type-2 arm: the SnowmanGangsta 1/7, the
SnowBalla 1/3, else the MysterySnowman (`NPC.cs:1450-1459`).

The summon: `ItemCheck_UseBossSpawners`' Snow Globe arm
(`Player.cs:43941-43956`) — the Snow Globe 602 with the
`CanStartInvasion(2, ignoreDelay: true)` gate (no active invasion, a
200+ max-hp player; the delay folds out), the roar and
`invasionDelay = 0; StartInvasion(2)` — the march state rides the World
(the morning-roll resolve's shape; the size 80+40, the far-side roll).

The three kinds (SnowmanGangsta 143/SnowBalla 144/MysterySnowman 145)
carry their verbatim `SetDefaults` rows (26×40, 200/240/220 HP, dmg
50/65/55, def 20/26/22, kb 0.6, value 400) and ride aiStyle 38's
movement core through the walker (the per-type speeds 3/3.5/4 fold into
the walker params; the Gangsta's bottle throw, the Balla's kick and the
Mystery's jump arms land with the projectile work).

Deviations: the throw/kick/jump sub-arms are the projectile-work
follow-ups; the Mystery's jump-dodge `ai[1] >= 3` state rides the
walker's generic hop.

## The death-screen coin loss (M4-REVENGE-1)

The drop half rode the earlier death.rs (`drop_gems`/`drop_coins`/
`roll_tombstone`); the display half lands here:

- **`value_to_coins`** (`Main.ValueToCoins`, `Main.cs:55220-55262`) —
  the greedy copper→platinum split, the zero tiers skipped, the bare
  copper still prints (`"0 copper"`), the en currency literals stand in
  for the `Currency.*` keys.
- **The KillMe arm** (`Player.cs:38931-38936`): `lostCoins =
  DropCoins()` (the drop pass returns the dropped copper total),
  `lostCoinString = ValueToCoins(lostCoins)` and the full death
  broadcast — the App's `lost_coins`/`lost_coin_string`/`death_text`
  fields set at `on_player_death`.
- **The death screen** (`Main.DrawDeathText`, `Main.cs:45324-45341`):
  the broadcast through the `DeathText` font (centered 60 px above the
  screen's midpoint), the softcore `Dropped coins: X` line through the
  `MouseText` font 50 px lower — `ui::draw_death_screen`, drawn from
  frame.rs while the player is dead (`hp <= 0`). The death-tint alpha
  rides fixed full-bright (the respawn fade = the death-screen pass).

Deviations: the expert `/4` and master `0` drop tiers and the PvP
zero-coin arm wait for the difficulty/pvp flags; the C# drops the coins
from the INVENTORY coin stacks (`Item.IsACoin`), the port's aggregate
`App.coins` purse splits greedily into the coin item stacks (the same
denominations, the 999 caps).

## The Goblin Army invasion (M4-EVENT-1)

The event framework's first slice. The state machine (`Main.UpdateInvasion`,
`Main.cs:64328-64386`): at `invasionSize <= 0` the defeat rides
(`SetEventFlagCleared(ref downedGoblins, 0)` + the broadcast), else the
army's `invasionX` marches toward `spawnTileX` by `max(dayRate, 1)` —
the arrival broadcast fires on the reach and every 3600 ticks
(`Main.InvasionWarning`; the port's literals stand in for
`Lang.misc[0..3]`). The morning trigger (`UpdateTime_StartDay`,
`:65439-65462`): the smashed shadow orb arms the goblin rolls — 1/3
before the first defeat, 1/30 (hardmode) / 1/60 (the classic) after —
and the `StartInvasion` request resolves in the app pass (the
`CanStartInvasion` gates: no active type, no delay, the 200+ max-hp
player; the size 80+40 per qualified player; the march column on the
world's far side).

The spawn: `Spawner.ShouldSpawnInvasionEnemies` (`NPC.cs:442-463`) —
the px-space surface band (`position.Y < worldSurface*16 + sHeight` = 200)
plus the ±3000 px window around `invasionX` — overrides the natural roll
with the goblin table (`:1419-1450`): the hardmode Summoner 1/30 (while
none alive), the Sorcerer 1/9, the Thief 1/5, the Peon 1/3, the Warrior
1/3, else the Archer. The five kinds carry their verbatim `SetDefaults`
rows (18×38, the aiStyle-3 walker — the Sorcerer keeps its caster arm,
the Summoner rides it in hardmode).

The kill credit (`NPC.cs:78236-78279`): the dead goblin's
`GetNPCInvasionGroup` (1) matches the active `invasionType` →
`invasionSize -= num9` (the Summoner's 10-point kill, the rest 1) — the
progress `ReportInvasionProgress` fields land with the UI-bar slice.

The borrow shape: the invasion spawn ride reads the world/player/roster
BEFORE the npc_step split borrows and resolves the spawn AFTER the tick
(the wholesale-app pass) — the same shape the voodoo sacrifice uses.

Deviations: the town-NPC fallback window (the `maxTilesX/2 ± 5` arm of
`ShouldSpawnInvasionEnemies`) needs the town roster scan — the follow-up;
the UI progress bar (`DrawInvasionProgress`), the invasion-delay timer's
save persistence and the waves 2-4 (the Frost Legion/Pirates/Martians)
are the follow-ups.

## The lava burn (M1.5-ITEM-lava-burn)

`CheckLavaDeath`'s non-doll arm: the drops touching lava vanish
(`TurnToAirAndSync`) unless `ItemID.Sets.IsLavaImmuneRegardlessOfRarity`
(the 18-row set — the obsidian family, the Fireblossom, the
Hellstone-bar rows — rides the port's `LAVA_IMMUNE` const) or the item's
rarity clears the `rare 0/-1` gate. The rare drops keep their loot:
the rarity gate is the whole reason the mid-game items survive the hell
pools. The voodoo arm rides the same lava read; the burn lands inside
the drops pass before the pickup (a drop in lava vanishes the same tick
it would be collected — the C# runs `CheckLavaDeath` before the pickup
chain too, `WorldItem.cs:405-425`).

## The voodoo sacrifice (M4-BOSS-6c)

The gameplay hardmode gate: the dropped Guide Voodoo Doll (267) touching
lava (`WorldItem.CheckLavaDeath`'s arm — `lavaWet` → `VoodooDollLavaDeath`)
is consumed and strikes the town NPCs dead — the Guide FIRST
(`StrikeNPCNoInteraction(9999)` → the port kills outright; the death pass
picks the kill credit up), then the stack's remainder across the other
town NPCs (the C# `isLikeATownNPC` list walks a shuffled copy; the port
walks the spawn order). `NPC.SpawnWOF` ports the gates (the depth
`maxTilesY - 205`, one living wall), the 1200-px player-clearance scan
(the spawn x steps 16 px away from the world centre's side while a player
sits within 1200 px), the free-row search (the solid/liquid>=100 blocker,
the ±1000-step expand), the `UnderworldLayer+10..+80` row clamp and the
`Announcement.HasAwoken` broadcast (the literal stands in for the
localization key).

The borrow shape: the items pass (`update_items`) reads the tile under
each drop inside its tick closure and collects the lava-hit doll; the
sacrifice resolves AFTER the tick (the spawn/kill pass takes `app`
wholesale). The doll items never burn silently — the non-doll lava
consumption (`rare 0/-1` items without the lava-immunity set) is the
follow-up.

## The Wall of Flesh + the Hungry + the Leeches (M4-BOSS-6b)

The aiStyle-27 machine (`npc.rs::update_wof`, the classic path
verbatim over BOSS-6's v1): the off-edge kill, the underworld
draw-band scan (`UnderworldLayer = maxTilesY - 200`; the 15-tile
solid/liquid walks ease the band 1 px per tick with the 160-px clamp),
the Y pinning to the band midpoint, the slide speed by the hp
thresholds (1.5 → 2.75 across the 75/50/25/10% steps — the direction
locks and never damps), the dead-player abandonment (3 s dead → the
wall kills itself), the tongue phase (`ai[1]` 2700 with the hp bonuses
→ the 3-4 phase cycle → the Leech spawns) and the Hungry init (11
minions along the band).

A structural note: the AI arms hold `&World`, so the wall cannot write
the band — the wall's AI computes the scan targets onto its own
`wof_bottom_target`/`wof_top_target` fields and the world sim eases
`World.wof_draw_top/bottom` toward them, alongside
`wof_present`/`wof_hp_ratio`/`wof_x` (the Hungry's read set). The same
borrow shape carried the spawn requests (`wants_leech` with the
`CountNPCS(117) < 10` gate, `wants_hungry`).

**The Hungry (115)** — the aiStyle-29 verbatim: the anchor = the wall's
x + the band row at `ai[0]`, the chase toward the player clamped to the
leash (300/500/700 by the wall's hp; the dive cycle ×1.3 after 100
ticks), the defense/damage tiers (30/60 at <50%, 20/45 at <75%) through
the new `defense_override`/`damage_override` fields (the contact path
prefers the override), the justHit recoil (the hurt cooldown stands in
for the C# flag), the wall's death deactivates the minion.

**The Leeches (117/118)** — the aiStyle-6 worm family: the head rides
the worm engine's chain assembly (the `Next(3, 6)` budget, the AI_006
default tunables, the 14×14/60 HP segments).

Deviations: the eye/mouth mirrors (114, the 100×100 8000-HP wall parts)
are the render-machinery follow-up (the vanilla draws them inside the
wall's band); the expert burst block (the hp-scaled spawn odds) and the
`getGoodWorld` spawn are out; the roar timer (`localAI[3]`'s sound 4)
is cosmetic and skipped. The bestiary `card_at` now matches the draw's
centred lattice — the stale top-left guard dropped cards once the
roster's rows overflowed the panel.

## The Eye of Cthulhu (M4-BOSS-1)

The first boss lands through the aiStyle-4 block's classic path
(`npc.rs::update_eye_of_cthulhu`; the 841-line C# `if (this.aiStyle == 4)`
in `AI()`). The machine state rides the new `Npc.ai[0..4]` (the phase /
the sub-state / the timer / the dash-servant counter):

- **Phase 0** — the hover at 200 px above the player (5 speed, 0.04
  accel); `ai[2]` counts to 600 unconditionally and flips into the 3×
  dash chain (`ai[1] = 1→2` at 6 speed, 150-tick flights, the 0.98 damp
  after 40 ticks); while the eye is above + within 500 px the servant
  counter `ai[3]` reaches 110 and requests the servant spawn (10 px out
  along the aim, the launch velocity inherited — the
  `wants_servant` → the world-sim resolve).
- **Phases 1/2** — the spins: the rotation ramps `ai[2]` 0.005→0.5
  rad/tick while `ai[1]` counts to 100, then the phase increments (the
  getGood expert servant bursts + `reflectsProjectiles` are out).
- **Phase 3** — the final: `defense_override = 0` (the new `Npc` field
  the damage path reads), the faster machine (6.8 dashes, 130-tick
  chains).

Deviations: the expert rage arms (`flag2/flag3` at 12%/4% hp, the
`ai[1] = 3/4/5` spread-dash sub-states, the ×1.5 turn rate) are out with
the port's missing difficulty flags (the skips carry the C# cites);
`EncourageDespawn(10)`'s off-screen counter becomes the distance cull
(the dawn climb `velocity.Y -= 0.04` + the sim's removal above the
player); `SpawnOnPlayer`'s tile scan simplifies to the clamped
600-px-above position (the boss flies to the target from wherever it
appears); `WorldGen.spawnEye` (the natural night spawn) is not modeled —
the Suspicious Looking Eye (43) is the only summon path.

The summon rides the new boss-spawner arm
(`editing.rs::use_suspicious_looking_eye` — the night gate, the
`SummonItemCheck` aliveness refusal, the roar cue `("Roar_0", 15)` =
`SoundID.Roar` 15, the consumption + the swing timers) — hoisted before
the editing fn's `player` borrow (the arm takes `app` wholesale). The
kill sets `World.downed_boss1`, fed into the loot flags (the `Cond::`
gates read it); the boss bar and the `MusicID.Boss1` (5) music extend
the King Slime hooks (the 93-then-5 ladder order kept).

## The full drop database (M4-LOOT-2)

`tools/gen_loot.py` reads the decompiled
`GameContent/ItemDropRules/ItemDropDatabase.cs` `Populate()` and emits
`crates/tr-app/src/loot_db_gen.rs` - `loot_db::database()`, **1421
net-id registrations over the 643 C# register sites** in the C# order.
The C# `RegisterToNPC(type)` expands to the type itself + the negative
net-id family (`PrepareNPCNetIDsByTypeDictionary` over `NPCID.NetIdMap`
- e.g. the type-1 slime family = `-3..-10`); the removal wrappers
(`RemoveFromMultipleNPCs(RegisterToMultipleNPCs(...), ids)`) compute the
net id set minus the removals, keeping the tough-slime gel split. The
negative-id dedupe mirrors `TrimDuplicateRulesForNegativeIDs`.

The engine grew the pieces the registrations need:

- **`LootFlags` + `LootNpc`** - the world/event snapshot the conditions
  read (the downed bosses 1/2/3/any/plantera/plant-boss, the seasons,
  the special seeds, `NPC.waveNumber` + the pumpkin/frost moon formulas
  verbatim, the closest player's zones, the pirate-map/soul depth bands,
  the npc's `value`/`lifeMax`/`damage`/`defense`/`boss`/
  `SpawnedFromStatue`); `spawn_loot` fills it from
  `World.hard_mode`/`day_time` + the scene zones + the dying npc.
- **`Cond`** - the data-driven enum porting ~50 `Conditions.*` gates;
  the yoyo gates and the moon trophy tables carry their exact formulas.
- **`LeadingConditionRule`** (`LeadingConditionRule.cs`: the condition
  rides `CanDrop`, `TryDroppingItem` always `Success`), the free
  functions `chain_on_success`/`chain_on_failed_roll` (the C#
  extension methods; the `self: Box<Self>` receivers do not erase across
  `dyn`), `DropPerPlayerOnThePlayer`, `OneFromRulesRule`, the luck
  variants (`NotScalingWithLuck`/`ScalingWithOnlyBadLuck` flags on
  `CommonDrop`, the OneFromOptions pair), and the
  `Food`/`StatusImmunityItem`/`MasterModeCommonDrop`/
  `MasterModeDropOnAllPlayers`/`BossBagByCondition` factories (all
  compositions of the M4-LOOT-1 rules, exactly as `ItemDropRule.cs`
  composes them).
- **The global entries** - `RegisterToGlobal` (the biome keys, the
  souls, the goodie bags, the kite drops) resolve before every NPC's
  own rules (`GetRulesForNPCID(includeGlobalDrops = true)`).

Findings: `corruption`/`crimson` recipe-style flags do not exist here -
the evil drops gate on `Conditions.IsCorruption/CrimsonAndNotExpert`
which read `WorldGen.crimson` (the WORLD type). The skipped rows
(13, warned): the NamedNPC localization gates, the RedHat legendary
gate, the MechBossSpawners/SlimeBody/StatueMimic/
FromOptionsWithoutRepeats rule classes (each needs its own event state).
The unknown item ids (the ~20 warn-only OneFromOptions sets) emit and
resolve - the executor drops only registry-known items, so they surface
as the registry scale-up work.

## Buff slots (M2-BUFF-1)

`crates/tr-app/src/buffs.rs` ports the `Player.buffType[44]`/`buffTime[44]`
engine: the full `AddBuff` chain (`Player.cs:5263-5447`), `DelBuff`/`ClearBuff`/
`FindBuffIndex` (`:5250-5480`), the `UpdateBuffs` timer head (`:9855-9876`) and
the end-of-update immune purge (`:26012-26018`), hooked into the 60 Hz player
step (tick at the top of `Player::update`, purge at the end — the C# calls
`UpdateBuffs` at `:24535` and purges at `:26012`, so the interleave becomes
meaningful only once per-buff effects land with M2-BUFF-3/4). The verbatim
`Main.debuff`/`lightPet`/`vanityPet`/`meleeBuff`/`pvpBuff` tables are extracted
from `Main.cs:8867-9316`.

Deviations:

- **Multiplayer branch skipped** — `AddBuff`'s `Main.netMode == 1` remote path
  (`Player.cs:5269-5275`, msg 55 + `Main.pvpBuff`) is unreachable in the
  single-player port; the `pvpBuff` table is kept as data for M6-NET.
- **Difficulty multiplier is expert-only** — `AddBuff_DetermineBuffTimeToAdd`
  samples `GameDifficultyData.DebuffTimeMultiplier` (Classic 1 → Expert 2 →
  Master 2.5, `GameDifficultyData.cs:96-100`); the port has no difficulty modes
  yet, so `expert_mode` applies ×2 and master waits for M6-CREATIVE/difficulty.
  `resist_cold` and `dead_cells_potion_station` are `BuffSlots` fields defaulted
  `false` — the rules are ported and unit-tested but have no sources yet.
- **Timer-head effect branches deferred** — the mount refresh
  (`buffTime = 10`, `:9881-9884`) and the per-buff effect chain (`:9875-12000`)
  land with M2-MOUNT and M2-BUFF-3/4; the tick only decrements today.
- **Quirks preserved verbatim**: `DelBuff`'s compaction runs to `maxBuffs - 1`
  (deleting from a full bar leaves a hole at 42 while slot 43 keeps its buff —
  `Player.cs:5457`); the pet-removal loops have **no** `i--` rescan while the
  melee loop does (`:5377-5397` vs `:5347-5360`); a bar of 44 debuffs rejects
  new buffs of any kind (the first non-debuff scan finds nothing, `:5331-5334`);
  a debuff added to a mixed bar evicts the first *positive* buff; the immune
  purge does not rescan after `DelBuff`. Each is covered by a unit test naming
  the line.
- **`buffImmune` sizing** — `BuffID.Count = 401` (`BuffID.cs:997`), so ids are
  valid only below 401; the C# would throw on higher ids and the port panics the
  same way (tests synthesise bars from real ids via a `safe_ids()` helper).

## Zoologist/Princess (M4-TOWN-19)
## Dye Trader/Party Girl (M4-TOWN-20)

Two late-roster town NPCs. Deviations: the Dye Trader's `item.dye > 0` scan
has no ported field (the `ItemSlot` carries only kind/count/prefix), so the
arrival reads the two id bands the C# names literally (the dye materials
1107-1120 and the Strange Plants 3385-3388); the equipment's dye slots
(`GetEffectiveDye`, `NPC.cs:7138-7145`) are not ported, so that half of the
gate is off. The Party Girl's shop rows gated on the golfer score, the
Pirate, the Golem, hardmode and the BirthdayParty ride systems the port
lacks — the BirthdayParty block (12 rows) lands with the party event. Her
arrival's `num40 >= 20` town-NPC count is counted live in the town pass (the
C# counts all 24 per-type counters in `UpdateTime_SpawnTownNPCs`).


Both are late-roster town NPCs. Deviations: the Zoologist's shop has the
fairy-torch row (`Chest.BestiaryGirl_IsFairyTorchAvailable`, `:3544`), the
tower/solar-gated rows and the remaining completion tiers outside the ported
subset; the Princess's shop carries the tenth-anniversary block (a custom
seed) and the slime-kings/Moon-Lord rows gated on `downedQueenSlime`
(no field) and `downedMoonlord` (unported). The Princess's arrival counts the
vanilla's 24 per-type town counters (`Main.cs:65651-65706`); the port folds
them to its own roster through `bestiary::all_town_kinds_present`, so she
arrives when the ported roster is complete rather than the vanilla's 24.

## Golfer (M4-TOWN-18)

The pair and its shop. The whole shop's variable stock keys off
`Player.golferScoreAccumulated` (`Chest.cs:3158`), which only the golf system
grows (`GolfHelper.PutBallInCup` → `AccumulateGolfingScore`, `Player.cs:4932`);
the port carries the score on the shop flags and the accumulator, so the rows
are faithful but stay at their score-0 tier until M4-GOLF-1 ports the balls,
cups and `GolfState` scoring. The bound-Golfer spawn's desert-wall gate
(`SpawnTileOrAboveHasAnyWallInSet(..., AllowsUndergroundDesertEnemiesToSpawn)`,
`NPC.cs:1680`) has no port table yet, so the port spawns it from the depth band
and rate alone. The Golfer's attack machine (projectile 721, `NPC.cs:54378`) and
the 480-tick shot lifetime (`:54868`) ride the town-attack task; the chatter
tiers (`GolferQuestsChatter*`, `NPC.cs:94792`) ride M6-LOCALIZE.

## Stylist/Tax Collector (M4-TOWN-17)

The three kinds and the Tax Collector's coin system. Deliberate gaps: the tax
purse is per-player in the C# (`Player.taxMoney`, saved at `Player.cs:54788`) —
the port carries it on the World until the player-file section exists; the Tax
Collector's "Andrew" easter egg (`NPC.cs:52968`'s 200-defense and
`:55033-55037`'s doubled attack) waits for the town-attack task; the hair-window
UI (`NPCInteractions.cs:425-441`'s `StylistHairWindow` → `Main.OpenHairWindow`)
lands with the hair system; the bound Tax Collector (534) and his
Purification-Powder release (`Projectile.cs:14728-14733`) wait for M4-NPC's
hostile-bound archetype.

## Wizard/Mechanic (M4-TOWN-16)

The pair and their bound forms port the 1.4.5.8 spawn/transform/shop slice. Deliberate gaps, all
in the ROADMAP row: the attack machines (`AI_007_TownEntities`'s `ai[0] == 10` arm for 124,
`NPC.cs:54450-54459`, and `ai[0] == 14` for 108, `:54865-54875` + the 108's multi-projectile
`SelectRandom(1,1,1,1,2,2,3)` at `:54927`) wait for the town-attack task; the Wizard's
hardmode blood-moon deck (5461) and halloween hat (1739) and the Mechanic's graveyard goggles
(4409) are constants-off (no season/graveyard stock gates yet); the Mechanic's Rod gate reads the
moon phase only — the C# also needs `NPC.AnyNPCs(369)` (the Angler), which the port's shop
engine carries through `StockWhen::PlayerHasItem`-style flags only, so the Angler-presence half
sits with the town-count gate's follow-up; the bound Mechanic's depth band rides the player's
tile without the Dungeon biome gate (`NPC.cs:2643-2652`'s `hardDungeon` branch); the
`hasTownNPC`-style flags the C# counts for the arrival (`num10`/`num12`, `Main.cs:65651-65658`)
fold into the port's per-kind absence check.

## Angler quest (M4-TOWN-15)

The quest state lives on the statics in the C# (`Main.anglerQuest`,
`anglerQuestFinished`, `anglerWhoFinishedToday`, `Player.anglerQuestsFinished`); the port
carries the first three on the World (the name list collapses to the single-player
`angler_quest_turned_in`) and the counter on the Player. Deliberate gaps, all documented in the
row: the quest **chat text** (`Main.NPCChatText_DoAnglerQuest`'s `Lang.AnglerQuestChat` result and
`NPC.GetChat`'s 369 dialog tree at `NPC.cs:94758-94760`) is not ported — the port has no NPC
dialog UI; the reward's accessory filter scans the inventory only (the C# also walks the armor,
the four banks and the three loadouts — the port has none); the price adjustment is pinned at
1.0 (the vanilla reads `currentShoppingSettings.PriceAdjustment`, which the town roster's mood
rows never change); the `crimson` gate reads `false` (the World carries no evil-kind field, so
the corruption-side quest rows always re-roll exactly like a corruption world); the world-file
save fields (`IO/WorldFile.cs:1329-1335`/`:2152-2166`) and the 74/75/76 messages land with
M6-SAVE-1/M6-NET; the Sleeping Angler's world spawn rides the port's 2-second town cadence
instead of the vanilla's `SpawnAnNPC` ladder position (same gates, same `Next(20)`).

## Deterministic screenshots: the OS pointer (M1.5-LIQUID-2-det-fix)

The follow-up flagged by the M4-LOOT-3 work ("the waterfall sparkle pass breaks
pure-world screenshot determinism, `TR_NO_WATERFALL=1` restores SHA256
equality") did **not** reproduce as a waterfall bug. The investigation:

- Two `TR_SHOT` runs of the same command differed by exactly 290 px at the
  forest lake — and *both* the `TR_NO_WATERFALL=1` and `TR_NO_WEATHER=1` pairs
  differed the same way. The diff was a small blue arrow (the in-game
  `Main.DrawCursor` sprite) at a different screen position per run: the **OS
  mouse pointer**. `App.mouse_pos` starts at `Vec2::ZERO` and follows real
  `CursorMoved` events, so the desktop pointer's position at capture time is
  run-to-run harness noise.
- An important false lead on the way: without a pristine world copy each run,
  the default `Terrustaria.trwld` is **rewritten by the autosave**, so run N+1
  loads run N's settled liquids and the comparison is meaningless. Deterministic
  pairs must `cp` a pristine `.trwld` per run (or use `TR_SPOUT`, which builds
  its fall by edits after load).
- The waterfall walk itself was audited and is deterministic: all state runs on
  fixed steppers (`waterfall_step`, `liquid_step`), `find_waterfalls` rescans on
  a 30-tick gate, and `TrySparkling`'s single randomness (`WaterfallManager.cs:
  1046-1065`) is pinned to a frame-derived `FastRandom` seed
  (`frame_count * 0x9E3779B97F4A7C15 ^ world.seed`). New unit tests pin this:
  `the_draw_walk_is_deterministic_for_the_same_seed_chain` (two managers, 600
  seeded frames, equal quads/lights/sparkles every frame) and
  `try_sparkling_grants_deterministically_for_a_seed` in `draw.rs` (the grant is
  a pure function of the seed; dark falls never roll).

Fix (harness semantics, a deliberate deviation from the C#): during a `TR_SHOT`
capture without an explicit `TR_MOUSE`, the logical pointer is parked offscreen
(`-64,-64`) and `CursorMoved` is ignored — `deterministic_pointer_park()`
(`app/mod.rs`). The pointer is harness *input*, not scene content; the original's
`Main.DrawCursor` still draws wherever the player's pointer is when `TR_MOUSE`
pins it (cursor/tooltip screenshots keep working).

Verified byte-identical SHA256 pairs (pristine world copy per run):
- forest lake, waterfalls on, no `TR_MOUSE` — before the fix this pair differed;
- `TR_SPOUT=2240,160` with the fall live in view for 1200 frames, no `TR_MOUSE`;
- the same `TR_SPOUT` scene with `TR_MOUSE=200,200` (explicit pin still wins).

## Regeneration engines (M2-BUFF-2)

`crates/tr-app/src/regen.rs` ports the life and mana regen accumulators:
`UpdateLifeRegen` (`Player.cs:18942-19391`) — the DoT clamps, the ramp tiers at
300/600/900/1200/1500/1800/2400/3000, the 3600 cap, moving ×0.5 vs standing
×1.25, the final `(int)Math.Round(num3 * (statLifeMax2/400·0.85 + 0.15))` add
(banker's, via the shared `tr_core::damage::round_bankers`, now `pub`), the
±120 chunk counters and the generic −120 drain (≤4 hp per examined chunk) —
and `UpdateManaRegen` / `ApplyManaRegenerationDelay` (`:19380-19562`) — the
delay curve `((1−mana/max)·240+45)·0.7`, the standing/grappling/buff double
decrement, the `maxMana/3 + 1` base with the still/grapple double-add, the
`num9 = 0.5` fill curve and the floor of 2. Wired into `update_survival`
behind an `hp > 0` gate; `Hurt` resets `lifeRegenTime` (`:38395`) while the
direct `statLife -=` paths (drowning, fall) do not — the old placeholder reset
in both and was corrected.

Deviations:

- **Inputs for systems that do not exist yet.** The DoT branches (M2-BUFF-3),
  campfire/heart-lantern auras (M2-BUFF-5), item multipliers and
  `manaRegenDelayBonus`/arcane crystal (M3-EQUIP), sitting/sleeping
  (M7-INTERACT) and the grapple flag (M2-PHYS-6) are neutral-default
  `RegenInputs` fields the engine already applies in the original order.
  The shinyStone standing-still package (`:19104-19108`, `:19126-19134`) folds
  into `extra_time`/`extra_regen`/`extra_ramp_scale` when M3-EQUIP lands.
- **Death from a DoT drain clamps at 0 hp** — resolved by M2-BUFF-3: the
  `KillMe` reasons (`:19361-19385`) now flow through `regen.rs`'s `DotDeath`
  (see "DoT debuffs (M2-BUFF-3)" below).
- **The full-mana sound + dust** (`SoundEngine.PlaySound(25)`, `:19550`) are
  deferred to M6-AUDIO; `update_mana_regen` returns the refill trigger.
- **TheDebugOptions.ManaV2 block** is a debug cheat and is not ported.
- **Integration-test pace corrected** — `health_regenerates_after_a_delay`
  bar belonged to the placeholder (1 hp/0.5 s). Faithful pace at
  `statLifeMax2 = 100`: the ramp scale is `0.3625`, tiers 1-2 round to 0, and
  20 s standing yields ≈ +4 hp; the test now asserts `50 < hp ≤ 56`.

## DoT debuffs (M2-BUFF-3)

The DoT set (poison/fire/blood; ichor is *not* a DoT in 1.4.5.8 — it is the
armor-penetration debuff and stays with M2-BUFF-4) landed on top of the
M2-BUFF-1/2 engines:

- **Player side** — `player.rs` derives the DoT deltas as a pure
  `derive_dot_inputs(&self, expert) -> DotDerivation` in the exact C# branch
  order (`Player.cs:18949→19095`: poisoned −4, venom/chlorophyte spore −30,
  the fire family −8/−16/−24/−60 each doubled while dripping slime (buff
  137) is held, suffocation −40, electrified −8 plus −32 more while
  running, tongued −100 expert-only; bleeding (30) only zeroes the ramp
  time, `:19178-19181`; honey (48) softens a negative sum by 4 clamped at
  0, `:19096-19103`) and feeds the five `RegenInputs.dot_*` fields.
  `regen.rs` runs the drain: the fast lane (burned || suffocating ||
  tongued && expert, `:19313`) chunks −600 → 5 dmg with an early return
  (`:19313-19335`), the generic lane keeps the ≤4 hp per −120 chunk; both
  return a `DotDeath` (`ByOther` 7/8/9/10/12/22/23 — the priority chains
  7→12→8 and 22→9→10→23→8 are precomputed by the caller,
  `:19321-19331`/`:19362-19385`) that `death.rs::death_text_for` renders.
  `update_survival` hosts the demo inflictions/cures: lava contact →
  `add_buff(24, 420)` (`:27710-27711`), water deletes OnFire/OnFire3
  (24/323, `:27750-27761`), Wet 103 re-grants the fire immunities every
  frame (`:11740-11751`), and the expired-buff sweep (`:25608-25616`) +
  per-frame `buff_immune` reset (`:25561-25565`) ride `Player::update` in
  the C# frame order (reset → tick → sweep → … → immune purge,
  `Player.cs:25555→26016`).
- **NPC side** — `npc_buffs.rs` ports the NPC chain as its own module (the
  C# keeps the Player and NPC `AddBuff` chains separate, so the port does
  too): the 20-slot `NpcBuffs` (`buffType`/`buffTime` `NPC.cs:6193-6197`,
  `NPC.maxBuffs = 20` `:95189`, `lifeRegenCount` `:6249`), the verbatim
  `NPC.AddBuff` chain (`:89926-89983`: immune guard → same-id extend →
  evict the first non-debuff → reject at 20 debuffs), the `UpdateNPC` buff
  block ticked from the head of `Npc::update` at the C# position
  (`:90373-90377`: set flags → clear expired → apply DOTs) with the
  `DOTTally` sum (`NPC.cs:59-141`, `:91594-91774` — Oiled (204) adds a
  flat +50 in combination with a fire debuff, AcceleratePoisons (398)
  doubles the boostable half via `SupportAcceleration`) and
  `GetHurtByDebuff`'s direct `life -=` drain that bypasses
  `StrikeNPC`/i-frames/knockback (`:91864-91888`). Immunities are the
  roster's slice of `NPCID.Sets.DebuffImmunitySets` (slime 20, King Slime
  20+31, demon eye/zombie/wandering eye none — `NPCID.cs:6079-6086`,
  `:6386-6392`, `:6905`) plus the derived rules of `NPC.cs:17818-17838`
  (immune 20 ⇒ 30 + 375, immune 69 ⇒ 36).

Deviations:

- **Honey does not extinguish fire** — the C# curing loop is driven by
  `WetCollision`, whose tile scan sets `Collision.honey` as readily as
  `Collision.wet` (`Collision.cs:1608-1618`), so standing in honey *would*
  douse OnFire in the original. The plan assumed the cure required actual
  water; checked against the source it was wrong in the C#'s favour of
  honey, and the port takes the plan fallback (DW-2.2): the cure is
  conditioned on a water probe distinct from honey
  (`in_water && !in_honey && !in_lava`).
- **The NPC DoT drain stops at hp ≤ 0** — the C# `GetHurtByDebuff` clamps
  `life = 1` and finishes the kill with `StrikeNPCNoInteraction(9999)`
  (`NPC.cs:91880-91887`); the port instead breaks the drain loop at
  `hp <= 0` and lets the existing hp ≤ 0 collect (`world_sim/npcs.rs:38`)
  run the normal death path (coins/gel drop exactly once).
- **The player-side DoT drain stops at the first hp ≤ 0 crossing** — the C#
  while-loops keep chunking a dead player until the counter empties (fast
  lane `:19315-19334`, generic `:19357-19385`); the port returns one death
  per tick and clamps at 0 (`regen.rs:236-241` fast lane, `:251-255`
  generic) — death lands the same tick, only the transient negative
  `statLife` differs.
- **The `dontTakeDamage` gate is omitted** — `UpdateNPC_BuffApplyDOTs` is
  wrapped in `if (!this.dontTakeDamage)` (`NPC.cs:91596`); the flag has no
  Rust counterpart yet, so the port always applies DOTs.
- **The last tick of an expiring debuff does not drain** — the C# keeps the
  boolean flags (`poisoned`, `onFire`, …) set until the end of the frame,
  so `UpdateLifeRegen` (`:25977`) still drains on the tick a buff expires;
  the port re-derives the flags by scanning the slots *after* the
  expired-buff sweep, so a debuff whose timer hit 0 this tick contributes
  nothing. One tick of drain lost per expiry; the sweep itself is verbatim.
- **Wet self-deletion runs one tick early** — the C# branch tests the
  previous frame's `lavaWet` (`:11741-11744`); the port tests the current
  `in_lava` probe, so Wet is deleted one tick sooner and OnFire from lava
  is granted immediately while Wet is held (the C# rejects it through the
  immune guard for one frame).
- **Pre-existing gaps made visible by the DoT path**: the lava contact hit
  is 30 dmg vs the C#'s 80 (`Player.cs:27687`), and `Player::respawn` does
  not wipe buffs while the C# `Spawn` clears every non-persistent buff
  (`Player.cs:16995-17002`) — both predate this task and stay open.
- **Death texts are EN stand-ins** — `death_text_for` hardcodes one English
  string per `DotDeath` after the `DeathText.*` keys of `Lang.cs`'s
  `CreateDeathMessage` switch (`:1073-1126`, e.g. `DeathText.Suffocated_*`);
  the localization pass replaces them.
- **Deferred pieces**: the starving lane and the honey regen aura
  (M2-BUFF-7/M2-BUFF-5), the Daybreak/Dryad's Bane spear sources, the
  `ApplyDOTs`-only flags (javelined/eelwhip/celled/…), `SuperNatural`
  lifeRegen, `TryAddingRepeatedBuff`, `vampireSeed` (nothing grants buff 22
  or sets the vampire-slime `ai[1] = 9` in the roster), ichor → M2-BUFF-4,
  and the presentation half — combat text, dust and the buff-bar HUD icons.

## Luck value (M2-BUFF-6)

`crates/tr-app/src/luck.rs` holds the full `RecalculateLuck` formula
(`Player.cs:29137-29164`) in `LuckState`, the three `Main.dayRate` decay
timers (`:18103-18110` coinLuck ×0.9999^dayRate floored <0.25, `:18836-18854`
ladyBugLuckTimeLeft draining to 0 from either sign, `:29166-29182`
brokenMirrorBadLuckTime driving the −0.25 flag), the torch-luck scan
(`TryRecalculatingTorchLuck`/`UpdateTorchLuck_ConsumeCountersAndCalculate`
`:17731-18088` — one tile row per player tick, style `frameY/22`,
`frameX<66` unsnuffed, the 13-verdict biome table, `torchLuck` floored at 0),
the coin tiers (`:18115-18153` incl. the duplicated unreachable
`>24900 → 0.175` branch, kept verbatim), the ladybug windows
(`NPC.cs:81566-81652`: kill 400 px linear / nearby 800 px `(1−d/800)^6`,
gold ×2, extremes win) and the buff-driven inputs (257 tier by remaining
time `:10285-10301`, 120 stinky `:10276-10280`, 383 kite tier
`buffTime/10800` clamp 0..3 `:9920-9924`). `update_luck` runs at the C#
`UpdateLuck` position (`Player.cs:25645` → `:29041`, after the buff effects);
the value is **never clamped** — the −0.7/1.0 caps are UI-only
(`NormalizedLuck`, `:4107-4122`). Consumers wired: the gel drop rules and
`drop_money` read `player.luck()` (`CommonDrop.cs:36`, `NPC.cs:79622`), the
melee `DamageVar` passes the raw value (`Player.cs:44342`), and
`start_rain`'s coin gate reads it as `GetPlayerWithHighestLuck().luck`
(`Main.cs:64668`).

Deviations, all deliberate:
- **Absent writers are neutral-default inputs, terms stay live**: lantern
  night (`LanternNight.LanternsUp` — M4-EVENT-4), the garden gnome
  (tile-567 SceneMetrics count, M4), `usedGalaxyPearl` (item 5340), the
  broken-mirror writer (projectile 1085 / item 5577), the accessory bonus
  (clover 5574-5576 / horseshoe 158 / lucky coin 855 — M3-EQUIP) and the
  kite items (`IsAKite`) all default to 0/false; `RecalculateLuck` applies
  every term in the original order so those tasks only fill the inputs.
- **`usedGalaxyPearl` is not persisted** — the C# writes it to the player
  file (`Player.cs:54784/:55258`); the port has no player-file writer yet.
- **The ladybug/coin writers have no live caller** — the ladybug NPC
  (604/605) and gold critters are the M4 critter row, the town NPCs the M6
  row; the setters (`on_lady_bug_killed/nearby`, `on_town_npc_death`,
  `add_coin_luck`) are unit-tested callables. `on_lady_bug_killed` returns
  the raw `ladyBugRainTime` add for the future `rain.rs` writer instead of
  touching `Main.ladyBugRainBoost` (the C# applies the 18000 cap at the
  add site, `NPC.cs:81568-81572`).
- **The Torch God event is out** — the `happyFunTorchTime` /
  `torchGodCooldown` head of the scan and the `:18066-18086` trigger tail
  are not ported; the `nearbyTorches` counter still runs.
- **The temple zone is unmodelled** — the skip gate
  `ZoneDungeon || ZoneLihzhardTemple` (`:17923`) runs on dungeon only.
- **`ZoneLihzhardTemple`-free torch inputs** — the zones ride a per-tick
  `SceneMetrics::zones()` snapshot on the player (the `auras()` pattern),
  refreshed beside `p.auras` in `world_sim/player.rs`.
- **No kite-luck reset** — the C# never zeroes `kiteLuckLevel` (assigned
  only inside the buff-383 branch, `:9920-9924`); the port matches:
  stale-until-recomputed.
- **Net-only halves skipped** — `luckNeedsSync`, packet 134
  (`NetMessage.cs:1557-1568`), the `oldLuckPotion` diff and the buff-120
  `talkNPC = -1` dialogue half (host-authoritative single player).
- **The first lucky branches diverge the streams C#-faithfully** — with
  luck 0 every wired consumer stays byte-identical to its pre-wiring plain
  roll; at luck > 0 the hit branches consume their extra draws exactly as
  the C# branch shape implies (pinned by `test_DW_3_4`'s luck-1.0 stream
  replay and the luck-0 equality tests).
- **C# float widening quirks preserved** — the coin tier compare
  `(double)coinLuck > 2.49` returns 0.075 at `coinLuck = 2.49f` (the
  widened float exceeds the f64 literal), pinned in the tier-matrix test.

## Mounts framework (M2-MOUNT-1)

`crates/tr-app/src/mount.rs` holds the `MountData` port (a 12-row ground table with the
exact `Mount.cs:696-2567` constants — Rudolph, Bunny, Slime, Scutlix, Unicorn, Basilisk,
the three horses, PogoStick, Wall-of-Flesh Goat, Queen Slime), the `MountState` instance
core (`SetMount`/`Dismount`/`CheckMountBuff`/`Reset` — `Mount.cs:6196-6305`/`:6142-6194`/
`:5275-5281`/`:675-694`, keeping the vanilla quirk that `fatigue`/`abilityCooldown`/
`abilityDuration` survive `Reset`), the saddle-item and mount-buff lookups (the
`Item.mountType` assignments `Item.cs:20779+`; `BuffID.Sets.MountType` as the inverse of
each row's `buff`), and the frame animator (`UpdateFrame` `Mount.cs:3581-4506`, states
0-4; the dash case 5 waits for M2-MOUNT-4). The per-tick block lives in `player.rs`
(`update_mount` — the vanilla `Player.cs:26623-26688` with the `UpdateEffects` head
`Mount.cs:4795-4803`/`:4949-4981` and the `UpdateJumpHeight` merge `:19619-19631`),
wired at the verified tick points: the merge insertion before `HorizontalMovement`, the
jump statics, the flight section (the `ResetFlightTime` refill `:26776-26781` + the
mount flight thrust `:26962-27015`), the fall-damage addendum (`:25301-25367` — the
`extraFall` threshold, the `FallDamage` multiplier, the bunny nullify `:25315-25318`,
the `CanFly` zero) and the CC dismount (`:26799-26812`). The buff is the source of
truth: `SetMount` adds the 3600-tick buff, the buff tick re-affirms the mount and holds
`buffTime = 10` (`:9881-9884` — the M2-BUFF-1 deferral), and a missing buff dismounts.
Equip paths: the saddle item use (`Player.cs:42768-42771` + the `CanMount` space gate
`Mount.cs:6581-6604`), the quick-mount key (`QuickMount` `:6045-6100`, wired to the
`controlMount` bind edge) and the `TR_MOUNT=<id|name>` pin (the `pinned_*` stand-in
pattern). A grapple fire dismounts (`QuickGrapple` `:6334-6337`). Draw:
`DrawPlayer_02_MountBehindPlayer`/`DrawPlayer_23_MountFront` order
(`PlayerDrawLayers.cs:211`/`:2912`) with the `Mount.Draw` frame math (`frameH =
textureHeight/totalFrames`, `Height -= 2` `:5722`, the center-anchored position
`:5554-5561`, the `xOffset*direction` mirror) in `render/player_layers.rs`; Rudolph
hides its front strip while idle (`:5520-5523`). The hitbox grows to `42 + heightBoost`
bottom-anchored (`:6264-6277`) and restores through the shrink path of
`Collision.TryChangingSizeFromBottomCenter` (`Collision.cs:2486-2513`).

Deviations:
- **Partial table (12 of `MountID.Count` 66)** — `set_mount` rejects ids outside the
  table where vanilla fills every row; the remaining categories land with
  M2-MOUNT-2/3/4.
- **No net sync** — `SetMount`/`Dismount`'s msg 13 (`Mount.cs:6222-6225`, `:6172-6175`)
  has no multiplayer surface.
- **No gravDir branch** — the flip block gated `!mount.Active` (`Player.cs:26690-26733`)
  has no port counterpart (the port has no gravDir field).
- **Idle-pick RNG stand-in** — `Main.rand.Next(900, 1500)` (`Mount.cs:4165`) runs on the
  animator's own splitmix64 stream (the port-wide `Main.rand` deviation pattern); the
  same tick sequence always picks the same delays.
- **No dust/sounds** — `DoSpawnDust`/`DoFailedDismountDust` and the equip/dis mount
  sounds wait for the particle/audio tasks.
- **FailedNoSpace unreachable** — every ground row's `heightBoost >= 0` makes the
  dismount probe a bottom-anchored shrink (always fits), so `QuickMount`'s 3-fail force
  + `PlayerNoSpaceTeleport` (`:6052-6070`) cannot fire; the grow-probe half of
  `Collision.TryChangingSizeFromBottomCenter` (`Collision.cs:2490-2508`) returns `None`
  until a consumer appears.
- **Cart/ability branches reserved** — the OnTrack/roller-skate/Fishron/drill arms of
  the per-tick block are comment-marked (M2-MOUNT-3/4); the SuperCart stat overrides
  read 0; `Mount.Hover` (hover mounts) cannot fire while no row sets `usesHover`.
- **Charge/stomp damage deferred** — the dash-contact damage (`Player.cs:20108-20170`)
  and the slime stomp (`:20533-20560`) wait for M2-MOUNT-4 with the other mount combat.
- **Queen Slime draw** — the render-target swap and the extra passes (`Mount.cs:5536-5544`,
  `:5840-5953`) are not ported; the row draws its base `Extra_204` frames. The WoF
  goat's `GlowMask[284]` glow pass has no content file (`GlowMask_284` does not exist
  in `assets/Content/Images`), so the row draws without the glow.
- **`miscEquips[3]`** — the equipment mount slot is not ported (M3-EQUIP); the
  quick-mount scan walks the inventory only.

Verification: the unit tests across `mount.rs`/`player.rs`/`render/player_layers.rs`
(the table constants, the set/dismount guards, the merge modes, the jump/fall
interplay, the hitbox grow/restore, the buff lifecycle, the quick-mount toggle, the
animator states, the draw math); `TR_SHOT` twin runs SHA256-equal with
`TR_MOUNT=unicorn` over `assets/test/master.trwld` at frame 120.

## Projectile pool (M4-PROJ-1)

`crates/tr-app/src/projectile.rs` now hosts the `Main.projectile[1000]` engine:
`NewProjectileSetup`'s first-free-slot spawn with the `FindOldestProjectile`
recycle (`Projectile.cs:10811-10880`), the per-tick `timeLeft--`/`penetrate == 0`
kill bookkeeping (`:17199-17208`), the `1 + extraUpdates` AI sub-step loop
(`:16668`), `DecrementLocalImmuneTimeCounters` (`:16674`, `:17258-17277`) and
the PVE walk (`Damage_PVE` `:12562-12600`, the per-owner gate `:12620-12624`)
with the on-hit immunity application (`:14087-14119`). All three C# immunity
systems are live: the per-owner `NPC.immune[owner] = 10` (stored on `Npc`,
decremented per NPC update, `NPC.cs:90454-90459`), the per-projectile
`localNPCImmunity` cooldowns (a flattened pool table) and the ID-static
`perIDStaticNPCImmunity[immunityIdentity = type, npc]` stamps against
`Main.GameUpdateCount` (`:604-607`, `:13972`). The shuriken moved into the pool
as its first AI family; its placeholder lifetime (`age > 2 s || bounces > 6`)
was replaced by the faithful `timeLeft = 3600` / `penetrate` rules.

Deviations:

- **The projectile path bypasses the melee `hurt_cd` gate** —
  `Npc::hurt_from_projectile` shares `hurt`'s body without the shared-cooldown
  check, because a projectile's right to hit is decided by the three C#
  immunity systems the pool now implements; the hit still arms `hurt_cd`, so
  melee stays gated afterward (in the C# every `StrikeNPC` writes the same
  `NPC.immune[owner] = 10` window).
- **NPC immunity is indexed by the NPC's `Vec` position**, which is stable
  between death compactions; the C# indexes stable NPC slots and clears them in
  `ResetNPCSlotData` (`:609-618`). After a compaction a live local-immunity
  entry may point one slot off for at most its remaining cooldown — revisit
  with the data-oriented NPC slots (M4 plan).
- **Multiplayer bookkeeping is not ported**: `ProjectileKey`/
  `slotGenerations`, `netSpam`/`netImportant` (the recycle treats every slot as
  recyclable) and the owner-sync asserts — single-player.
- **`DebugOptions` branches** (`Shared_RandomizeProjectileSlots`, `ManaV2`
  equivalents) are debug cheats and are not ported.
- The shuriken's damage/knockback stay the first pass's stand-ins (registry
  data lands with M3-ITEM-1); the pool itself is content-agnostic.

## Spawn call discipline (M4-SPAWN-1 completion)

The spawn-rate multiplier chain and the caps landed earlier (2026-11-03); this
completion ports the **call discipline** of `NPC.SpawnNPC`
(`NPC.cs:80170-80176`) and its named RNG stream:

- **The stream** — `Main.SwapRandom("SpawnNPC")` (`Main.cs:17886`) swaps in a
  persistent per-stage stream for every world update. The port keeps
  `App.npc_rng` as that stream's state and advances it (the xorshift LCG step)
  once per update, **before** the latch/caps, so a suppressed or capped tick
  still consumes stream state exactly like the C#'s swap; each call draws from
  `FastRandom::new(state)`. Deterministic per world seed; verified by
  `the_named_stream_is_deterministic_per_seed` (two identical runs agree
  tick-for-tick over 900 moon-event night calls, and a different seed
  diverges). The LCG step is the port's own xorshift core (agents.md's
  "deterministic stand-in" pattern) — the C#'s exact `UnifiedRandom` sequence
  is not reproduced, as everywhere else in the port.
- **The `noSpawnCycle` latch** (`NPC.cs:6179`, consumed at `:80171-80174`) —
  `App.spawn_no_cycle`; while set, one whole `SpawnNPC` call is suppressed and
  the flag clears. No setter exists yet (the C# sets it from NPC
  release/catch and the revenge spawn at `:77993/78137/91553`) — those systems
  flip the flag when they land.
- **The dead-player gate** — `CanSpawnEnemiesNear`'s `player.dead` branch
  (`NPC.cs:349-351`); the journey spawn-rate-disable and Moon-Lord-distance
  branches have no sources yet.
- **Pacing quirk preserved**: the port's `npc_timer` (a 4 s spawn hold after
  respawn, `world_sim/player.rs`) freezes the stream advance while held — the
  C# has no such hold, so during it the port's stream lags the C#'s tick
  count; deterministic either way, and the hold clears.

## Wiring engine (M7-WIRE-1)

`tr-core/src/wiring.rs` ports the wiring core: the four colour passes of
`TripWire` (`Wiring.cs:525-620`), the `HitWire` flood fill with its `SkipWire`
set, the `_toProcess` multi-trigger counter — 4 on the seed cells, 3 on
enqueue, `b = 0` through junction boxes (424/445), removed at zero so a
5+-path convergence re-fires — and the `[2, max-2]` bounds (`:837-995`),
`HitWireSingle`'s device subset and `HitSwitch`'s rect table
(`:259-330`), plus `CheckMech`/`UpdateMech` (`:455-471`, `:145-258`) with the
tile-144 timer periods 60/180/300/30/15 (by `frameX / 18`) and the 411
lamp-pair frame flip.

Deviations:

- **Wire colours live in a sparse side table** (`World.wires: WireState`) —
  the C# packs them as `sTileHeader` bits (`Tile.cs:623-657`) and the port's
  12-byte `Tile` has no header space (E1). Persistence in `.trwld`/`.map` is
  pending (M6-SAVE-1); synthetic test worlds set wires through
  `World.wires.set`.
- **Device entity effects are passthroughs** — the mine (210), geyser (443),
  statue spawns, pumps (`XferWater`), teleporters and the cannons' projectiles
  belong to M7-WIRE-2/5; their `HitSwitch`/`HitWireSingle` cases currently do
  nothing beyond the signal walk.
- **The multi-tile actuation walk** (`ActuateForced` over `TileObjectData`
  regions) is M7-WIRE-3; the engine toggles the single tile's
  `FLAG_INACTIVE` (`Tile.flags` already carries the actuator/inactive bits).
- **The wire pulse animation** (case 419's `frameX` shift in
  `HitWireSingle`) is render-side state on a side-table colour, not a tile
  frame — deferred with the wire renderer (M7-WIRE-3).
- **Timers pulse indefinitely** while armed: the re-trip's own tile is
  SkipWire'd for its own pass (the seeds pre-enter `_wireSkip`), so
  `HitSwitch`'s frameY toggle does not disarm the armed timer — verified by
  `the_timer_pulses_on_its_period_then_pops` (two pulses 60 ticks apart).
- **Sounds** (`SoundEngine.PlaySound(28, …)`) are not wired to the audio
  system yet (M6-AUDIO polish).

## Tile entities (M7-TE-1)

`tr-core/src/tile_entities.rs` ports the `TileEntity` framework and the
item-holding entity halves: the registration-order ids
(`TileEntitiesManager.cs:27-40` — TrainingDummy 0 … CritterAnchor 10, the
discriminant IS the wire/save id), the `ByID`/`ByPosition` books
(`TileEntity.cs:15-22`), `place`/`remove_at` with the per-TE `ValidTile` frame
checks (`TEItemFrame.cs:37-43` tile 395 `frameY == 0 && frameX % 36 == 0`;
`TEWeaponsRack.cs:33` tile 471 `% 54`; `TEFoodPlatter.cs:32` tile 520
`frameY == 0`; `TEHatRack.cs:407` tile 475 `% 54`), the item payload as the
`(short type, byte prefix, short stack)` triple the wire/save format carries
(`TEItemFrame.cs:47-56`) and the HatRack presence-bitsByte round trip
(`TEHatRack.cs:59-80`). The books live on `World.tile_entities`.

Deviations:

- **The behaviour halves are M7-TE-2**: the dummy's wired hit tracking, the
  logic sensors, the display doll rendering, the pylon network, the Dead
  Cells jar and the leashed anchors are registered (ids + placement
  validity) with empty payloads.
- **The food gate** (`ItemID.Sets.IsFood`, `TEFoodPlatter.FitsItemFrame`-style
  placement check) is applied by the caller — the item registry lives in the
  app layer.
- **Items are `(type, prefix, stack)` triples**, not the app's `ItemSlot`s —
  the UI layer maps between them; the triples are exactly what
  `WriteExtraData`/`ReadExtraData` and msg 121 carry.
- **Persistence** for `.wld` landed with M6-SAVE-1 (world-file section 7,
  `SaveTileEntities:3302`/`LoadTileEntities:3315`, with the id reassignment, the
  `CheckValidTile`/`InWorld` drop pass and `OnWorldLoaded`); the port's own `.trwld`
  still carries no TE section, so its worlds lose the entities on reload — the
  M6-SAVE-1b follow-up.
- **`MaxEntitiesPerChunk = 1000`** (`TileEntity.cs:14`) is not enforced — the
  per-chunk cap exists for the C#'s save-section packing; the port's books
  are unbounded maps.

## Logic gates (M7-WIRE-4)

`wiring.rs` gains the logic-gate half of `Wiring.cs`: `LogicGatePass`
(`Wiring.cs:682-730`), `CheckLogicGate` (`:733-831`) and `PokeLogicGate`
(`:380-386`). The tiles: LogicGateLamp = 419, LogicGate = 420
(`TileID.cs:1467-1469`) — note the port's M7-WIRE-1 tests used kind 419 as a
wire-tile filler; the engine reads the colour bits from `World.wires`, so the
filler kind is inert, but the id belongs to the lamp. The mechanics:

- A wire hit on a lamp toggles its frame (`case 419` verbatim:
  `num50 = 18; if (frameX >= 18) num50 = -18; if (frameX == 36) num50 = 0;`)
  and pokes the gate below through the lamp column walk
  (`CheckLogicGate` descends past lamps to the first 420).
- The gate counts the lamp stack above it (`frameX == 18` = on; a `frameX ==
  36` lamp stops the count and marks the gate broken → its frame becomes 36),
  evaluates the type by `frameY / 18` (the C#'s case order: 0 AND, 1 OR,
  2 NAND, 3 NOR, 4 XOR, 5 XNOR) and writes `frameX = 18·output`.
- A state change enqueues the gate into `_GatesNext`; the pass re-trips the
  gate's own 1×1 wire (`_GatesDone` guards repeats) — that flood walks the
  gate's whole wire line, toggling every logic lamp on it, which is how gates
  chain downstream.
- The faulty lamp (the poked lamp at `frameX == 36` — `flag6`) rolls the
  output: `Main.rand.Next(num2) < num3` (`:812`). The port consumes its own
  xorshift stream seeded per world (`Wiring::seed_faulty_rng`) — the C#'s
  `Main.rand` sequence is not reproduced (the port-wide RNG deviation), but
  the roll is deterministic per seed.
- A gate with a lamp on its own tile flip-flops within one pass: the output
  flip re-trips the gate's wire, the flood re-toggles the stack lamp and the
  gate re-evaluates to the opposite — the verbatim C# outcome, covered by
  `the_gate_output_flip_re_trips_its_own_wire_line`.
- The logic sensors (tiles 593/594) emit wire signals from world state —
  M7-WIRE-2 (they need the player/liquid probes).

## Teleporters & pumps (M7-WIRE-5)

`wiring.rs` gains the teleporter and pump halves of `Wiring.cs`:

- **The teleporter pads** (tile 235, `HitWireSingle`'s case 235 at
  `:1066-1082`): the wall-87 Lihzahrd guard, the pad registration (the first
  hit fills `_teleport[0]`, a hit at a different position fills
  `_teleport[1]`, the halfBrick adds 0.5 to the anchor y), reset to (-1, -1)
  per colour pass. `Teleport()` (`:3120-3178`) runs up to four times per
  `TripWire` (once per colour pass that registered both pads): the too-close
  reject rule (`:3122-3127`), the two 48×48 pad boxes anchored at
  `(X·16, Y·16 − 48)` and `TeleporterHitboxIntersects`' union check
  (`:3181-3189`).
- **The entity half** lives in the app layer (`world_sim/wiring.rs`): the
  engine emits the pad pairs; the app moves the living player (gated by
  `blockPlayerTeleportationForOneIteration`, `Wiring.cs:3126/704-708`) and
  the NPCs (`lifeMax > 5`, no bosses — King Slime in the port's roster, and
  the `TeleportationImmune` set — none of the implemented kinds is immune,
  `NPCID.cs:10433`).
- **The pumps** (tiles 142 in / 143 out, `HitWireSingle` `:2046-2125`): the
  2×2 body tiles SkipWire'd and registered into the per-pass queues (capped
  at 19 entries each), `XferWater` (`:475-520`) after every colour pass with
  both queues filled. **The C# re-reads the in tile's liquid after every out
  transfer** (`liquid = Main.tile[num, num2].liquid`, `:506`) — the overflow
  continues into the next out tile; the port's first cut captured the level
  once and was corrected (the cap test asserts the 55+45 split).
- The pump `SkipWire` of the 2×2 body means a pump tile reached as a flood
  seed does not register — the trip must come from a switch/wire line, as in
  vanilla builds.

Deviations: the entity movement is the port's positional teleport (the C#'s
`Player.Teleport`/`NPC.Teleport` effects — the dust, the styles — are
M2-DEATH/M7-WIRE-3 polish); the `DebugOptions` and MP branches are skipped.

## Traps & statues (M7-WIRE-2)

The wire-driven device effects landed in two layers: `tr-core/src/wiring.rs`
ports every device arm of `HitWireSingle`/`HitSwitch` verbatim (`Wiring.cs`
citations on each block) and emits typed requests; `tr-app/src/world_sim/
wiring.rs` resolves them through the port's spawn engines. Landed: the statue
device (case 105, `:2125-2478`, both spawn tables, the `flag6`/`flag7` gates,
the mushroom swap 34, the teleporting statues 40/41, the mushroom statue 349
`:2480-2508`, the boulder statue 531 `:1994-2013`), the traps (case 137,
`:1760-1982`, all eleven variants), the cannons (209 `:1237-1343` → `WorldGen.
ShootFromCannon :50586-50697`, 212 snowball `:1345-1420`), the fireworks
(216/335/338), the explosives/mine/geyser (141/210/443, the latter two in
`HitSwitch` too, `:288-296`), the toilets (497/15) and the volcano traps
(593/594). The requests: `StatueSpawn::{Npc, Item, Teleport, Boulder}`,
`TrapRequest::{Shot, Spear{count_kind}}` and `DeviceShot{kind, x, y, vx, vy,
damage, knock_back, ai0, ai1}`.

Deviations:

- **The ported statue kinds**: the app spawns the statue NPC styles whose C#
  id has a ported `NpcKind` — 1 BlueSlime, 2 DemonEye, 6 EaterofSouls,
  21 Zombie, 24 FireImp, 42 Hornet, 49 CaveBat, 55 Goldfish, 58 Piranha,
  65 Shark — at the verbatim bottom-center anchor (`NPC.NewNPC`'s
  `Bottom = (X, Y)`, `NPC.cs:80770`). The rest (the critters 51-62, the
  birds 74/297/298, the WallCreeper 164/165, the Bunny 46/540, the Crab 67,
  the BlueJellyfish 63, the Mimic 85, the dragonflies, the macaws,
  UndeadViking 167, Pigrons 170/171/180, Medusa 480, GraniteGolem 482, … )
  emit their id and drop at the drain with one note; each kind's family
  lands the arm.
- **The projectile families that dropped** (requests emitted verbatim, the
  app spawn drops them — `proj_data` has no row): the fireworks rockets
  167-174 and 415-418 (aiStyle 34), the fountains 419-422 (aiStyle 73), the
  cannonball 162 (aiStyle 2), the bunny 281 (aiStyle 49), the snowball 166
  (aiStyle 2), the geyser 654 (aiStyle 126), the toilet 733 (aiStyle 153),
  the boulder 99 (aiStyle 25) and the flame jet 187 (aiStyle 38). Landed
  rows: 108/164 (the aiStyle-16 blast family, the explosives/mine), 178 and
  601 (the cannon's aiStyle-1 ammo) and 980 (the venom dart, aiStyle 1).
- **The cap/dedupe gates run app-side (decision D1)**: the spear's
  proximity scan, `BunnyCannonCanFire` and the fountain's anchored-projectile
  dedupe read the app's pools, so the engine emits the request with its roll
  results and the app gates the spawn. Consequence: a blocked spawn still
  consumes the engine's rolls (the C# evaluates its cap before its rolls) —
  invisible outside the port's stand-in RNG stream, which is not byte-equal
  to `Main.rand` anyway (ANALYSIS "Wiring engine (M7-WIRE-1)"). The cannon's
  static/per-tile cooldowns are assigned before the app-side gate, as in the
  C#, so a bunny-blocked shot still leaves them standing.
- **The ceiling spiky ball's shared cap decision**: the C# evaluates the cap
  once and spawns the projectile twice (in-case `:1961` and in the shared
  tail `:1977-1981`); the port emits two identical `Spear` requests and the
  drain applies one cap decision to both.
- **The floor spiky ball's cap is a verbatim no-op (F1)**: the C#'s cap loop
  counts `type == num95` with `num95` still 0 (`Wiring.cs:1815-1822`) —
  `ProjectileID.None`, which no live slot can hold. The request carries
  `count_kind = 0` and the cap never blocks.
- **`Item.MechSpawn` filters by type** (`Item.cs:49385-49406`): the caps
  travel with the request (`cap_ids` — the spawn id when empty, plus the
  seasonal cap ids 58/{58,1734,1867}, 184/{184,1735,1868}, 166/{166} — F6)
  and each id must pass on its own exact-type count. `NPC.MechSpawn`
  (`:7420-7506`) carries the C#'s group table (birds, bunnies, ducks,
  seagulls, turtles, goldfish, dragonflies) — the port's kinds map 1:1 to
  their C# ids, so the groups bite only for the ids the port actually holds.
- **The first-table anchor correction**: the WIRE-1 scaffold spawned style 35
  at `num141 - 12`; the C# first table spawns at `(num140, num141)`
  (`Wiring.cs:2270`) — corrected with a regression test.
- **The spawn flags**: `spawned_from_statue` carries `NPC.SpawnedFromStatue`
  (`NPC.cs:6061`) and gates the loot (`NPCLoot`'s
  StatueSpawnedDropRarity/NoEarlymodeLootWhenSpawnedFromStatue checks,
  `NPC.cs:78849` — the port's statue-spawned NPCs drop nothing until the
  AnyInteractions tracking lands). The C#'s other three flags
  (`value = 0`, `npcSlots = 0`, `CanBeReplacedByOtherNPCs`) have no port
  field yet.
- **`PartyIsUp` and `getGoodWorld` are unmodelled**: style 9 always drops the
  plain Bunny (46, `Wiring.cs:2303-2307`) and `NewNPC`'s 46→614/62→66
  getGoodWorld remap never fires (`NPC.cs:80752-80762`).
- **The traps' spec is now complete but 187 waits on aiStyle 38**: the trap
  table (98/184/187/185/186/980 — `ProjectileID.cs:750/922/926/928/924/2514`;
  the WIRE-1 const names 185 "TRAP_SPEAR"/980 "TRAP_SPIKY" were wrong
  against the C# and are renamed) is emitted verbatim; the port's own names
  in the earlier docs (dart/superdart/flame/spear/boulder/spiky) did not match
  the C# table and are corrected in the code comments.
- **The visual layer stays deferred**: `PoofOfSmoke` (the blocked-spawn puff),
  `Animation.NewTemporaryAnimation` (the volcano traps, the mushroom swap)
  and `SoundEngine.PlaySound(28, …)` have no port target, exactly as the
  WIRE-1 burn-out poof and sounds were deferred.
- **`HitSwitch`'s 132/411 lever arm** (the lever flip + `CheckMech(60)` + the
  2×2 detonator lamp flip, `Wiring.cs:335-372`) is still unported — a WIRE-1
  gap outside this row's scope.

## Falling blocks (M7-FALL-1)

`tr-core/src/falling.rs` ports the trigger half of
`WorldGen.SpawnFallingBlockProjectile` (`WorldGen.cs:86039-86090`); the flight
half rides the app projectile pool (`tr-app/src/projectile.rs`), because in
the C# the falling block **is** a projectile (`AI_010`'s `ai[0] == 1f`
mode) — the sand at the broken support becomes projectile 31 and falls with
`velocity.Y += 0.41f` per tick (the MP branch `:86074-86082` additionally
dedupes falls within 60 ticks per column; the port is SP-layer, so the
dedupe is not modelled).

- **The trigger** (`check_falling`): the tile is one of
  `TileID.Sets.Falling` {53, 234, 112, 116, 224, 123, 330-333, 495}, the
  below passes `BlockBelowMakesSandFall` (`:86585-86604` — the below
  inactive, or the two-below inactive with a non-solid below, or the
  hardened-sand blend 165 below), and `AllowsSandfall` (`:86612-86624`) —
  the chest/statue-anchor kinds above hold the column. The caller (the
  break path in `world_sim/editing.rs`) clears the above tile — the C#'s
  `tileCache.ClearTile()` — and spawns the projectile at
  `(i·16 + 8, j·16 + 8)`.
- **The projectile data** (`GetSandfallProjData`, `:86514-86570`): the
  kind→(projectile, damage) map — the sand default 31, crimsand 56, mud 59,
  sandstone 67, silt 71, ebonsand 241, the snow variants 411-414 with
  damage 0, the desert fossil 812; damage 10 otherwise.
- **The flight**: the gravity `+0.41` per sub-step capped at 10
  (`TILE_FALL_GRAVITY`), and the position integrates the velocity per
  sub-step (the generic `position += velocity` of `Projectile.Update`) —
  the port's first cut only accumulated the velocity and the block hovered
  in the air; the chain test caught it.
- **The landing** (`landed` + `can_place_back`): when the box's bottom
  edge (`y + 10`, the 10×10 sand-ball box) enters a solid tile row, the
  tile is placed back **at the box center** (`(x + 5, y + 5)` — the C#'s
  `AI_010` lands the projectile via the tile collision and
  `Kill()`→`PlaceTile` at the box's tile), the liquid under it is
  displaced (`liquid = 0` at the placed cell — the sink-in is a
  simplification; the C# has the splash variants), and the projectile is
  killed. The chain refill works: the placed tile supports the next
  falling block only via the next break/trigger pass (the port does not
  re-run `SpawnFallingBlockProjectile` from the landing itself — the C#
  lands on the column below and stops).
- The `timeLeft` default 3600 matches `SetDefaults`.

Deviations: no MP dedupe (SP layer); the sink-into-liquid and the
`Collision` niceties (the tile-collide step of the C#'s generic movement,
which also kills on the side collision) are simplified to the
bottom-edge landing; the projectile-vs-NPC contact damage of the falling
ball works through the pool's ordinary `Damage_PVE` (the `damage` input
of `spawn_falling`).

## Actuation & wire rendering (M7-WIRE-3)

The actuation completes `HitWireSingle`'s first actuator hook (`Wiring.cs:
1001-1003`): any wire-hit tile that carries an actuator runs
`ActuateForced` (`:407-416`) — inactive → `ReActive`, otherwise
`DeActive`. The public gated twin `Actuate` (`:391-403`) exists for the
placement paths and is not consumed yet.

- **`DeActive`'s guard chain** (`:3195-3222`): the tile must be active;
  tile 226 (the Plantera bulb) below `Main.worldSurface` is untouchable
  before `NPC.downedPlantBoss` — the port has no boss-progress flags yet
  (M5-HARDFORCE), so the guard is always in the strict state. The solidity
  gate `tileSolid && !NotReallySolid` (`TileID.cs:592` = {387, 388, 10})
  then the forced `flag = false` set {314, 379, 386-389, 476}. The
  placement rule `!above.active() ||
  (!PreventsActuationUnder[above.type] && WorldGen.CanKillTile(i, j))`
  (`:3210-3211`) — `PreventsActuationUnder` (`TileID.cs:569`) = {21, 467,
  26, 77, 88, 470, 475, 237, 597, 441, 468} (the chests/dolls/sensors
  would pop through the reframing, so their support never actuates).
- **`CanKillTile`'s slice** (`WorldGen.cs:70237-70330`): the bounds, the
  inactive tiles refuse, wall 350 refuses, the anchors above block —
  `IsATreeTrunk` (`TileID.cs:487`) with the frameX 66/88 + frameY 198
  window, the cactus 323 (the base frames 66/220), the chest family
  {21, 26, 72, 77, 88, 467, 488}, the lantern 80 (the anchor slots
  `frameX / 18` in {0, 1, 4, 5}) — then the boulder set
  (`TileID.cs:504`) refuses. The buried-chest scan (`CheckBoulderChest`)
  is content-gated out (the port spawns no buried-chest boulders); the
  locked-door case passes (`IsLockedDoor` — the port's doors carry no
  lock state yet); `CanDestroyChest` passes (no chest inventories yet,
  M7-TE-2).
- **The `SquareTileFrame(resetFrame: false)` tails** (`:3212`, `:3229`):
  the port runs the object-cascade + reframe halves
  (`edit.rs::actuation_square_frame`) — a work bench whose anchor
  actuated away pops and its drop queues on the engine
  (`take_actuation_drops`), spawned by the app layer through
  `registry::tile_drop` like the break path's drops.
- **The consumers**: `Collision`'s every solid test carries
  `!tile.inActive()` in the C# (`Collision.cs:445/:461/:471/:549/:565/:
  574/...`) — the port's `solid()` and the `StepUp`/`StepDown` support
  gates gained the conjunct, so actuated blocks are walk-through and
  liquids read `nactive()` (`Liquid.cs:171/:236/:457`) and seep through.
  The draw side applies `Tile.actColor` (`Tile.cs:272-290`) — RGB ×0.4 —
  via the tile light.
- **The wire overlay** (`render/wires.rs`, hooked after the liquids
  before the over-wire projectiles per `Main.cs:57028-57030`):
  `Main.DrawWires` (`Main.cs:58218-58701`) verbatim in structure — the
  per-colour neighbour masks (+18 up, +36 right, +72 down, +144 left on
  the `WireNew` sheet's 16 px columns, the colour rows at +0/+18/+36/+54
  plus the junction-box column offset for 424/445), the
  `color * (1 / num7)` dimming when several colours share a tile, the
  6 px inset strips where a later colour claims a side an earlier one
  already holds, the actuator plate (`TextureAssets.Actuator`) on every
  actuator tile that receives light, and `CanShowWireStuffHere`'s
  pre-Golem wall-87 temple guard (`Player.cs:46946-46961`).

Deviations: `WiresUI.Settings.HideWires` and the mechanic-accessory
`builderAccStatus` per-colour channels (White/×0.5/Transparent) are fixed
at the defaults — no wires UI or accessory equip yet, so `num = 1` and
every channel draws at the light colour; the tile-423 LightSeaGreen
special rect is content-gated; the MP branches are skipped.

## Arrows & bullets (M4-PROJ-2)

The aiStyle-1 family turns out to cover **both** the arrows and the
bullets — the musket ball (projectile 14) rides `AI_001` with the arrows
(`SetDefaults` `Projectile.cs:950-961`), so "arrows & bullets" is one
generic arm plus the per-type data.

- **The generic arm** (`Projectile.cs:57572-57583`): `ai[0]++` per tick
  (the `flag3` set — the special types opt out, `:55945-56037`), and when
  `ai[0] >= 15f` the gravity `velocity.Y += 0.1f` starts (the counter pins
  at 15). The ±16 velocity clamp rides the tail (`:57756-57763`), and the
  generic rotation is the flight direction + 1.57 (`:57725-57728`).
- **The tile contact**: the C# `Update`'s tileCollide movement blocks the
  hit axis; for the aiStyle-1 projectiles the blocked hit kills (the
  arrows break on contact). The port runs the per-axis
  `collides_rect` check and kills on either.
- **The aiStyle-14 spear family** (the trap spear 185): the default arm
  (`:25143-25158`) — the straight flight, `ai[0] > 5f` →
  `velocity.Y += 0.2f` with the ground-slide `velocity.X *= 0.97f` (and
  the <0.01 zero), `rotation += velocity.X * 0.1f`; the tile-kill scan
  (`:25726-25737`) probes the box inset `(x + 4, y + 4)` with the +8 px
  center box against solid, non-127 tiles; `lavaWet` and the shimmer
  kill (`:25739-25750`).
- **The data** (`proj_data`): the `SetDefaults` slice — the wooden arrow
  1 / the flame arrow 2 (10×10, `timeLeft` 1200, `arrow`), the musket
  ball 14 (4×4, `timeLeft` 600, `extraUpdates` 1), the trap darts 98 /
  184 and the spiky ball 980 (10×10, `penetrate` −1, `trap`), the
  Blowgun's poison dart 645 (`tileCollide` false, `extraUpdates` 5, the
  local immunity cooldown −1) and the trap spear 185 (14×14, aiStyle 14,
  `timeLeft` 900). The pool's kind ids migrated from the M4-PROJ-1
  internal table to the verbatim `ProjectileID` (the shuriken 1 → 3) —
  the internal 1 collided with the C# wooden arrow.
- **The trap emission** (`Wiring.cs:1763-1993`): the six traps are FRAME
  VARIANTS of tile 137 (`frameY / 18`: 0 dart → 98/20 dmg/12 speed,
  1 superdart → 184/40/12, 2 flame → 187/40/5, 3 spear → 185/40/4±1,
  4 boulder → 186/60/8, 5 spiky → 980/30/12); the direction decodes from
  `frameX` (0 → −1 X, 18 → +1 X, 36-71 → −1 Y, ≥72 → +1 Y), the spawn
  offsets ±10 px (±14 spear, ±18 boulder), the wall-mounted negatives
  (−10..−6) emit with the fixed offsets. The `CheckMech` cooldowns
  200/300/90 run in the engine; the requests (`TrapRequest`) drain
  app-side where the spear's 1000-slot proximity cap applies (`:1825-1843`
  — the 200 base count decrements by the distance rings
  50/100/200/300/400/500/700/900/1200 px; the C#'s `j * 18 + 8` center y
  is verbatim, typo included).

Deviations: the flame arrow's water transform (`TransformType(1)`,
`:16825-16827`) needs the projectile wet tracking — M4-PROJ polish; the
boulder 186 (aiStyle 37) and the flame jet 187 (aiStyle 38) requests are
emitted verbatim but dropped at the spawn boundary until their AI
families land; the arrow item recovery on kill does not exist in 1.4.4
(no `Kill` drop path found); the spear spread's rand order (engine-side
pre-roll vs the C#'s post-cap roll) differs by one stream draw and is
MP-invisible.

## Boomerangs (M4-PROJ-3, the boomerang half)

`AI_003_Boomerang` (`Projectile.cs:38866-39545`) is a two-state machine on
`ai[0]`/`ai[1]`; the ported kinds 6 (Enchanted), 52 (Wooden) and 113
(Ice) all ride the generic arms — the per-type parameter table only
starts at the special boomerangs (the Light Disc 106's 45-tick window is
the one verbatim override kept).

- **The OUT state** (`ai[0] == 0`, `:39024-39331`): `ai[1]++` per tick;
  at `ai[1] >= 30f` the return state sets (`ai[0] = 1f, ai[1] = 0f`).
  The straight flight keeps `tileCollide`; **the outbound tile hit does
  not kill** — it reverses the velocity and switches to the return
  (`:14594-14602` in `Kill()`'s collision path, mirrored at
  `:13068-13080` for the NPC hits).
- **The RETURN state** (`ai[0] == 1`, `:39468-39545`): `tileCollide =
  false` — the boomerang flies home through terrain. The seek builds the
  owner-center vector, kills past 3000 px (`:39500-39505`), scales it to
  the speed 9 (`num26`; the per-type overrides 12-20 are the special
  kinds) and steers per-axis at the accel 0.4 (`num27`) with the ±2×
  boost when crossing the target velocity (`:39515-39537`). The
  `meleeSpeed` remap (`:39487-39497`, ×1-10) is ×1 at the port's default
  melee speed (M3-EQUIP brings the stat). The owner's 20×42 box catching
  the boomerang kills it (`:39538-39544`).
- **The generic `Item.shoot` firing**: the throw path was the M4-PROJ-1
  shuriken hardcode; it is now `Player.ItemCheck_Shoot` (`:47056+`) —
  the aim vector normalized to the item's `shootSpeed`
  (`:47274-47292`; the melee `speed /= meleeSpeed` is ×1), the spawn at
  the player center + 10 px toward the cursor, `weaponDamage` (the item
  damage × the class multiplier), the item's knockback, the crit roll
  from the item's damage class and the `consumable` decrement (the
  boomerangs stay in the inventory; the shuriken is consumed).
- The pool's `update` carries `owner: Option<(f32, f32)>` — the seek
  input (`Main.player[owner]` reads); `None` keeps the boomerangs
  outbound until the timer flips them.

Deviations: the flails (aiStyle 15, `AI_015_Flails` `:43724-44600` — the
877-line per-type parameter table + the chain state machine + the
channel coupling) are the explicit follow-up (M4-PROJ-3b); the boomerang
sounds/dust are render-side; the homing variants (182/866) and the
phaseblade subset are out of the ported kinds.

## Bombs (M4-PROJ-9)

The aiStyle-16 thrown bombs (`AI_016_Bombs`, `Projectile.cs:46904-47545`)
split into a small flight core and a heavy kill path.

- **The flight**: `ai[0]++` per tick; the gravity tail (`:47505-47520`)
  waits `ai[0] > 5f` (the thrown arc) and then adds
  `velocity.Y += 0.2f` with the ground-slide `velocity.X *= 0.97f`
  (the dynamite 29 another `*0.99f`, the tiny-x zero). The tile bounce
  rides the `Update` movement's aiStyle-16 arm (`:19737-19762`): the hit
  axis reflects at −0.4 (the dynamite another ×0.8 → −0.32). The
  rotation `+= velocity.X * 0.1f`. The **flight damage zeroes** every
  tick (`:47175-47176` — the kinds outside the keep-damage list; every
  ported bomb is) — the explosion re-arms it.
- **The sticky set** {37, 519} (`:46955`): no plain tile collide — the
  scan (the box inset ±4 px against solid, non-127, non-platform tiles)
  snaps `velocity = (0, -0.2f)` against a wall, so the bomb hovers and
  detonates in place.
- **The fuse seeds** live in `NewProjectileSetup`, not `SetDefaults`
  (`:10970-10993`): 180 ticks (the bombs 28/37/516/519) and 300 (the
  dynamite 29) — the AI's `owner == myPlayer && timeLeft <= 3` fires
  the kill path (`:47165-47167`).
- **The kill path** (`Kill()`'s aiStyle-16 branch, `:70662-70690`):
  `PrepareBombToBlow` (`:46770-46844`) recentres the box to 128×128 and
  re-arms damage 100 / knockback 8 (the bombs) or 250×250 / 250 / 10
  (the dynamite), `tileCollide = false`; `Damage()` walks the NPCs
  through the resized box (the pool's ordinary PVE walk); then
  `Kill_ExplodeTiles` (`:77265-77380`) digs the crater — the radius 4
  circle for the bombs, 7 for the dynamite, each tile passing
  `CanExplodeTile` (`:78640-78700`) before `WorldGen.KillTile`, and the
  3×3 wall walk around each killed tile (wall 350 = the echo wall
  exempt).
- **The firing needs nothing new**: the bombs flow through the generic
  `Item.shoot` path from M4-PROJ-3 — `registry::projectile(166) = 28`,
  the consumable decrement, the item's `shootSpeed` 5.

Deviations: `SelfHurtPlayers()` (the bomb's self-damage) and the
explosion visuals (dust/gore/sound) are M2-DEATH/M4-PROJ polish; the
hardmode `CanExplodeTile` gates stay closed until M5-HARDFORCE; the wall
breaks discard the wall item (the C# `KillWall` drops nothing in the
explosion path either); the water-bomb transforms (784-807) and the
special bombs (910/911, 102, 773) are out of the ported kinds.

## Logic sensors & training dummy (M7-TE-2)

The `RequiresUpdates` half of the tile entities (`TileEntity.PerformUpdates`
runs from `UpdateWorld`, `WorldGen.cs:71507`): the logic sensors land fully
and the training dummy lands as the engine state machine.

- **The logic sensor** (`TELogicSensor.cs`, tile 423): the check type rides
  `frameY / 18` — 0 Day, 1 Night, 2 PlayerAbove, 3 Water, 4 Lava, 5 Honey,
  6 Liquid (`FigureCheckType` `:168-200`); the LED rides `frameX` (0 off /
  18 on). `GetState` (`:202-267`): the Day/Night kinds read `Main.dayTime`;
  the PlayerAbove kind scans the 82×162 box `(x*16 - 33, y*16 - 161)` vs
  the live players' hitboxes; the liquid kinds read the anchor tile's own
  liquid, the type match rejecting the wrong fluids (water rejects
  honey/lava/shimmer), with the 15-tick `CountedData` grace keeping the On
  state after the liquid disappears.
- **The trip discipline**: the Day/Night kinds trip the wire only on the
  rising edge (`!On && state`, `:109-118`); the rest trip on every state
  change. `UpdateEndInternal` (`:135-152`) fires the collected trips
  through `HitSwitch` **after** the scan — the port keeps that order — and
  the PlayerAbove trips set `blockPlayerTeleportationForOneIteration` for
  their hit (the teleporter-into-a-sensor-loop guard).
- **`OnPlaced → FigureCheckState`**: the placement reads the check type
  from the tile frame and settles the LED at the current state without a
  trip (`figure_sensor_state`).
- **The training dummy** (`TETrainingDummy.cs`, tile 378): the entity
  carries `npc` (−1 = free); the free dummy spawns when a player's
  hitbox intersects the 32×48 box inflated by 1600 px (`:56-74`), the
  spawn anchor `(x*16 + 16, y*16 + 48)`; the tied dummy verifies its NPC
  each tick and frees on the lost tie (`:45-53`). The engine half is
  complete; **the app's NPC 488 spawn waits for the roster** — the port's
  `NpcKind` has 83 match sites across 10 files, and the TargetDummy
  (passive, no movement, no loot) lands with the M4-NPC families rather
  than rippling the roster here.
- **The behaviour tiles registered**: the dummy 378
  (`frameY == 0 && frameX % 36 == 0`), the sensor 423 (`frameY % 18 == 0
  && frameX % 18 == 0`), the jar 698, the kite anchor 723, the doll 470,
  the pylon 597.

Deviations: the display doll (970 lines — the armor poses, the dye slots,
the mannequin interactions) is its own follow-up; the pylon network is
M7-PYLON-1; the leashed critter/kite anchors need the critter entities;
the dummy's DPS-meter hook rides the NPC 488 landing.

## Pylon network (M7-PYLON-1)

The pylon system is the network-validity engine
(`TeleportPylonsSystem.cs`) — the port's slice covers the whole gate
ladder; the UI and the town-NPC fields plug into it from their tasks.

- **The types** (`TeleportPylonType.cs:5-17`): 11 kinds whose enum values
  ARE the tile styles (`GetPylonTypeFromPylonTileStyle` =
  `(TeleportPylonType)style`, `TETeleportationPylon.cs:175-178`), decoded
  from `frameX / 54` on the tile 597's anchor
  (`frameY == 0 && frameX % 54 == 0`, `:135-139`). The item map
  (`:116-131`): the surface purity 4876, the jungle 4875, the hallow
  through the mushroom 4916-4921, the victory 4951, the underworld/
  shimmer 5652/5653.
- **The NPC-home rule** (`DoesPositionHaveEnoughNPCs`, `:203-219`): the
  town NPCs' `homeTileX/Y` inside the `ZoneScanSize` rectangle centred on
  the pylon **and** each home within 100 *tiles* of its own NPC's current
  centre (`Vector2.Distance(homeTile, Center/16) < 100f` — the
  home-and-NPC proximity, easy to misread as a pylon distance; the port
  carries the pairs). The count: 2 for every type except the Victory
  pylon's 0 (`HowManyNPCsDoesPylonNeed`, `:309-316`).
- **The gate ladder** (`HandleTeleportRequest`, `:93-186`): the near
  check (the tile 597 within the 60-tile pylon reach,
  `TileReachCheckSettings.Pylons` `:86-90`), the destination's NPC homes,
  the Lihzahrd wall guard (pre-PlantBoss + below the surface + wall 87),
  the destination's biome gate, then the other-usable-pylon stage ladder
  (1 = the NPCs missing at the current pylon, 2 = the biome missing,
  else not-near) — the first failed gate wins the message, and the port
  short-circuits in that order.
- **The biome gates** (`DoesPylonAcceptTeleportation`, `:240-306`): the
  purity pylon requires above-surface, outside the 380-tile ocean bands
  and NO other biome's `EnoughTilesFor*` at the spot; the jungle 140 /
  hallow 125 / corruption+crimson 300 / shimmer 300 / mushroom 100 tile
  thresholds; the underground `y >= worldSurface`; the underworld
  `y >= maxTilesY - 200`; the Victory pylon always accepts. `scene.rs`
  gained the six missing `EnoughTilesFor*` gates and the mushroom count
  (the blocks 70/71/72/528).

Deviations: the remix branches (`:244-290`) are skipped (the port
generates no remix worlds); the NPC-home list feeds from M4-TOWN (the
`isLikeATownNPC && !homeless` fields); the map-click teleport UI and the
`NetTeleportPylonModule` are P4-UI/MP scope; `player.Teleport`'s dust and
styles are M2-DEATH polish.

## Sundial/moondial fast-forward (M1.4-BG-21)

`M1.4-BG-21` ports the dial fast-forward and the `dayRate` plumbing. `World`
carries the dial state: `fast_forward_to_dawn`/`fast_forward_to_dusk` and the
u8 cooldowns saved in TRWLD v4 (`Main.sundialCooldown`/`moondialCooldown`,
`Main.cs:2161/2165`); the port's save keeps only the cooldown bytes, while
vanilla persists the FF flags too since v264 (`WorldFile.cs:1342`, read gate
`:2451-2455`) — here they are transient by nature (see deviations).
`is_fast_forwarding`/`day_rate` port `IsFastForwardingTime`/`UpdateTimeRate`
(`Main.cs:6160-6167`, `:6199-6206`): 60 while a dial runs, 1 otherwise.
`clock.rs::advance_time` applies the full `dt·60·dayRate` add before the
boundary checks (`Main.cs:64962-64964`); the transition arms port
`UpdateTime_StartDay`/`UpdateTime_StartNight` — the dial's flag clears, its own
cooldown decrements once (saturating at 0), `time = 0.0`, the day half flips,
and `moonPhase` advances at dawn (`Main.cs:65368-65372` + `65382`/`:65393` +
`:65394-65401` for day, `65213-65220` + `65357-65358` for night). The triggers:
`interact_at` dispatches `DialStarted`/`DialRefused` for tiles 356/663 behind
the C# gate `!fastForward && cooldown == 0` with the click consumed always
(`Player.cs:32636-32653`; the SP arms `Main.cs:6169-6196` — the
`SendData(51)` client branches are MP), and `hit_wire_single` decodes the
2×3 anchor and runs the same gate (`Wiring.cs:1137-1175`); the cue is
`SoundID.Item4`. Consumers: the `LightParams`/`FlameEnv` cooldowns mute the
dial glow while cooling (the scanner's kind 356/663 light arms), and
`AmbienceCtx.day_rate` feeds the BG-19 sky scheduler its real rate.

Deviations, all deliberate:

- **The dayRate consumers scale — via M1.4-BG-22.** Vanilla runs `UpdateWeather`
  dayRate× per frame (`Main.cs:17228`) and divides several denominators by dayRate —
  the rain chance (`Main.cs:64898-64914`), the cloud alpha lerp, the sandstorm tick,
  stars, the invasion delay, luck — so in vanilla a fast-forward also rushes the
  weather. Landed the same day on the shared 60 Hz world tick (the weather cluster,
  the real rain rolls, sandstorm); see "dayRate consumers (M1.4-BG-22)" below —
  growth deliberately stays 1× (the growth-trap note there).
- **Scaled add, not a 60-iteration loop; one boundary per call.** The port
  keeps the C# shape — one `time += dt·60·dayRate` add per call, then one
  boundary check (`Main.cs:64964`, `:65137`, `:65148`) — instead of replaying
  `UpdateTime` 60×, which would light every transition RNG roll 60×. The dt
  clamp (0.1 s) caps the add at 360, below the 32400 night length, so one call
  crosses at most one boundary (pinned by
  `boundary_max_add_crosses_at_most_one_boundary` in `clock.rs`).
- **The `>=` boundary quirk (pre-existing, now dial-relevant).** The port's
  boundary checks are `time >= 54000/32400` while C# checks
  `time > 32400.0`/`> 54000.0` (`Main.cs:65137`, `:65148`) — an add landing
  exactly on the boundary fires one tick earlier here. The dawn-crossing
  screenshot pair below arms exactly this case.
- **The transition discards the overshoot** (`time = 0.0`, `Main.cs:65357-65358`
  and `:65382`) — replacing the former wrap-the-overshoot-into-the-new-half
  carry; vanilla restarts each half at 0 the same way.
- **No transition event rolls.** Blood moon at dusk (`Main.cs:65342-65351`) and
  solar eclipse at dawn (`:65419-65430`) roll from `Main.rand` and force-reset
  both cooldowns when they fire (`:65350-65351`, `:65422-65423`); the rolls are
  M4 territory, the arms carry citation hooks, and the force-resets stay
  unreachable until then.
- **The dials render as plain tiles.** They have no `TileObjectData` def in the
  port's registry (the trigger arms are literal kinds 356/663, the same shape
  `place_item_tile`'s legacy route produces), so a placed dial is a single
  active cell drawing its sheet's default frame instead of the C# 3×2 framed
  object (`Wiring.cs:1139` decodes `frameX % 36 / 18` from that object layout).
  The cooldown-gated glow is the faithful part; the registry def is a
  tiledata-registry follow-up, render code untouched by this task.

Determinism/proven pairs (pristine `.trwld` copies per run, pointer park, zoom
2.0, `TR_NOHUD`/`TR_NO_BG`): a `TR_DAY=0.25` night shot with both dials placed
by `TR_FILL` — the cooldown-0 glow lighting the ground around them —
SHA256-equal double run (`695bab0a…`); and `TR_TIME=32340` + `TR_SUNDIAL=1`
arming the fast-forward from a day-half time (`app/mod.rs:1603` maps it to
`day_time`, so the add is read only by the `>= 54000` dusk check) — with the
shot dt pinned to 1/60 (`frame.rs:48-49`) each frame adds 60 ticks, landing
dusk exactly on `time` = 54000 at frame 361 (`clock.rs:26`) and, 540 frames
later (the 32400-tick night), dawn exactly on `time` = 32400 at frame 901
(`clock.rs:39`) — both boundary-exact, the `>=` case above — with the
frame-3600 capture showing the new morning (the dial cleared at dawn, the
rate back to 1), SHA256-equal double run (`fca1e1d0…`).

## dayRate consumers (M1.4-BG-22)

`M1.4-BG-22` scales the weather cluster the way vanilla does while a dial
fast-forwards. One shared 60 Hz accumulator (`FixedStepper` `App.world_tick_step`,
60 Hz, cap 8) drives `world_sim/weather.rs::update_world_tick`; `frame()` calls it
once per drawn frame with the raw dt — after `update_growth`, before `update_rain` —
replacing the draw-block wind tick (which also means headless sessions tick now; the
old draw-block path never ran without a gpu). Per tick, the exact C# `Main.Update`
order (`Main.cs:17228-17231`): the `UpdateWeather` thunder arm ONCE (`:58739`'s
`currentDayRateIteration == 0` — the tiered WEATHER-1 chain, never per iteration),
then `for _ in 0..day_rate()` × `wind.update_rain(max_raining())` (`:58830-58950`;
`num8`'s real `5/9·maxRaining` boost at `:58832` — the binary `Wind::update(raining)`
wrapper is deleted), then `Rain::update_time` (`:64833-64949`), then
`Sandstorm.UpdateTime` with `day_rate` as its first argument (`:64966`, moved out of
`scene_scan`; the TR_SANDSTORM pin freezes the arm exactly as before). One
accumulator pins the order and keeps the consumers in lockstep — vanilla runs the
cluster inside one `Main.Update` sequence, and per-system accumulators would let rain
rolls fire between wind steps. Driving off the clock path (`advance_time`) is out for
the same reason BG-21 chose the scaled add: the clock advances per drawn frame with
dt scaling, while discrete chance rolls cannot dt-scale.

The rain state machine replaces the WEATHER-1 session schedule — a deliberate
behaviour change at all rates (the retirement note in "Rain and weather" above).
`Rain` now carries `rain_time`/`lady_bug_rain_boost` (`Main.rainTime`/`ladyBugRainBoost`,
`:64874`/`:64833-64836`); while raining it drains `rainTime` by `dayRate` and rolls
the strength change (`:64864-64888`), and when not it rolls the ×5.75/×4.25/ladybug
start arms with the `86400/max(1, dayRate)` denominators (`:64892`) — no `dayTime`
gate on the start branch (only the slime roll is day-gated, `:64925`).
`StartRain`/`ChangeRain`/`StopRain` (`:64644-64740`) port the `rainTime` bonus ladder
(`Next(3..8)` arms, `:64679-64702`) and the strength ladder (`num5`, `:64703-64719`)
verbatim; `RollLuck(25)` consumes the Rain stream through the loot engine's
`roll_luck` (`:64668` — luck 0 today, plain semantics), and the coin-rain value roll
(`Next(75,151)·100·100 ×maxTilesX/4200`, `:64670-64673`) is consumed but not stored —
coin projectiles are unported. The cloud-alpha ramps (`cloudAlpha` and the
`cloudBGAlpha` stand-in) scale by `max(1, day_rate)` (`:61902-61906`,
`:13187/13205` shapes).

RNG stays per-consumer (the WEATHER-2 convention): Rain's splitmix is the single
rain-consumer stream (the thunder tiers, the `update_time` rolls, the
Start/Change/Stop ladders); wind keeps its xorshift64 (the high-32 modulo, the
WEATHER-1 fix) and walks it `day_rate` times per tick; sand.rs keeps its own splitmix
(vanilla's `Sandstorm` rolls the shared `Main.rand`). Iteration cardinality is a pure
function of the dial flags — `day_rate()` is 60|1 — so the 60× FF stream consumption
is deterministic and replayable; vanilla's stream alignment was already surrendered
by the per-consumer convention.

Deviations, all deliberate:

- **The growth trap.** `Main.desiredWorldTilesUpdateRate = 1` during a fast-forward
  (`Main.cs:6204`) — vanilla deliberately does NOT scale tile growth with dayRate,
  and `WorldGen.GetWorldUpdateRate` (`WorldGen.cs:72020-72028`) only clamps it at 24
  and freeze-gates it. `world_sim/growth.rs:18` passes the literal `1`; never feed it
  `day_rate()`.
- **The `numClouds` stand-in is pinned at 100** (`NUM_CLOUDS_STAND_IN`, matching
  `render/sky.rs` `make_clouds`' static `num_clouds = 100.0`): with exactly 100 the
  `numClouds > 150`/`> 100` ChangeRain bands are dead and only the storm gate
  (`cloudBGActive >= 1f` → `cloud_bg >= 1.0`) reaches the heavy bands; the same
  `cloud_bg` stand-in reads the ×4.25 start arm (`:64916`). The live cloud-count
  system (`Main.cs:58952-59036`) is unported — revisit both with the
  weather-completion row.
- **`Rain.wind` (the drop drift) is static now.** The retired session schedule was
  its last writer; vanilla's drops read the live `Main.windSpeedCurrent` per tick
  (`:58830-58950` feeds `Wind` directly). The field stays seed-time-fixed until a
  consumer re-homes it.
- **`thunder_inputs`/`any_player_120` snapshot once per frame.** Vanilla reads the
  Main statics live inside `UpdateWeather` and scans all 255 players for
  `statLifeMax >= 120` (`:64899`); the port gathers the cross-system inputs and the
  one-player `max_hp >= 120` bool once per `update_world_tick` call. Bounded — every
  input is frame-stable (wind speed/target, maxRaining, the scene-state alphas) —
  and documented at the gather site.
- **`gameMenu ×20` on the cloud lerp is out** (`:61906-61909`) — the port has no
  menu-side cloud-lerp path; the ramps run only in-world.
- **`IsRainingForever` persistence is out** (no save field; the `StartRain`/
  `ChangeRain` gates and the `max(val, 0.1f)` floor skip it, cited inline) —
  creative-power territory.
- **FreezeRain/LanternNight/slimeRain gates resolve to the ported values** — the
  powers do not exist yet, so the `!FreezeRainPower.Enabled`/`!LanternsUp`/
  `!slimeRain` conjuncts are all false (the C# shape with the absent systems'
  values); the `forced` rain pin keeps precedence over `update_time`.
- **Documented out** (unported counterparts — nothing to scale): stars
  (`Star.cs:175-209`, the starfall loop `WorldGen.cs:71832-71856` with the
  `RollLuck(15)` target scan), the invasion `invasionX` walk
  (`Main.cs:64366-64396`, M4-EVENT-1), the luck decay timers (`Player.cs:18107`
  coinLuck, `:18840` ladyBugLuckTimeLeft, `:29172` brokenMirrorBadLuckTime —
  M2-BUFF-6), the cloud entities + the `cloudBGX` drift (`Cloud.cs:391-411`,
  `Main.cs:59360-59422`), the `updateCloudLayer` trigger (`Main.cs:13174-13230`),
  the slime-rain rolls (`Main.cs:64925-64949`, M4 events), the travel/homeless rolls
  (`Main.cs:65017-65060`), the tax timer (`Main.cs:64997-65000`), the bucket fill
  (`WorldItem.cs:548-571`), the Deerclops rain-night spawn arm (`Main.cs:65123`,
  behind the `!IsFastForwardingTime()` gate at `:65067`), the cultist/mystic timers
  (`CultistRitual.cs:27/32`, `MysticLogFairiesEvent.cs:48/85`), and the fallen-star
  projectile's `ai[0]` clock (`Projectile.cs:53101`).

Determinism/proven pairs: the dry-day dayRate=1 screenshot pair is SHA256-equal
old-vs-new (`347d1457…` — with nothing raining, the wind/cloud-alpha/sandstorm
shapes are byte-identical to the pre-change binary); the per-consumer streams keep
the port's determinism (iteration counts are pure functions of the dial flags, so a
FF run replays exactly); the dial-FF (`TR_TIME=32340` + `TR_SUNDIAL=1`) and zoom
1.0/2.0 pairs follow the standing double-run SHA256 rule. Rain pairs are
new-binary double-runs only — the schedule retirement is a deliberate behaviour
change at all rates.

## Sandstorm (M1.4-WEATHER-2)

The research corrected the row: the severity machine, the SceneState
filters, the music-40 ladder, the zone gate and the bgDelay tracking
already landed (BG-18 / WEATHER-1), and the `SandstormSky` sky tint is
**dead code in 1.4.5.8** — registered (`ScreenEffectInitializer.cs:71`)
but never activated anywhere, so `_opacity` stays 0 and the quad never
draws. Documented out; the visible sky wash is the SceneState
filter/overlay pair.

The task's real delta is `Sandstorm.EmitDust` (`Main.cs:17247`) — the
flying sand, in `crates/tr-app/src/sand.rs`:

* the budget: `1000*(screenWidth/1920)*(gfxQuality*0.5+0.5) +
  1000*screenWidth/1920*0.1 − Dust.SandStormCount` per tick, `20*Severity`
  spawn attempts behind the `Next(num6/4)` gate (`Next(0)` rolls 0 — XNA),
  the windward screen-edge rolls (both `Next(3)`s always evaluate, C# `&&`
  order), the off-screen-x y-spread, the wall gate (`tile.wall != 0` skips),
  and the cluster shuffler: `Next(maxValue)` re-rolls the position by
  `RandomVector2(±10) + velocity*-1.1` and spawns again until the roll hits
  or the budget dies (a rejected `NewDust` slot still burns budget and
  still feeds the shuffle its velocity — `EmitDust` sets the properties on
  the returned index blindly);
* the four sand-family weighted colours (`53/396/397` desert,
  `112/400/398` crimson, `234/401/399` jungle, `116/403/402` snow;
  `WeightedRandom` falls through to the first entry on a zero total);
* the dust-268 AI (`Dust.cs:1789-1818` + the generic tail): the move,
  the `1.0050251` speed-up, the scale drift (`+0.01`, `−0.05` without
  `ShowSandstormVisuals`), the spawn-`fadeIn` one-off `+0.03` then the
  generic `−0.01` (with the storm on the three cancel and the scale holds
  at ~0.94; without it the net `−0.05`/tick fades the orphans out in ~17
  ticks), the player-fall parallax (`vel.y` lerps toward the player's fall,
  `pos.y += vy*0.2`), the solid damp (`scale*0.9`, `vel*0.25`, the spawn
  `fadeIn` keeps the first tick damp-free) vs the sine fall
  `sin(x*0.0043982295)*2+3`, the off-bottom and `scale<0.1` deaths, and
  the black-light kill at draw (`Main.cs:39015-39017`);
* the draw: the shared `Dust` sheet's second band — `dust.frame` maps type
  268 to x = 10*268−2000, y = 60+10*rand(3) (`Dust.cs:177-182`), the sheet
  the atlas already loads — at `10*scale` px with the light base pass plus
  the `GetColor` overlay (`colour + light − 255` per channel, floored at
  0, skipped when every channel is 0).

`update_scene` runs emit+tick after `Sandstorm.UpdateTime` and ahead of
the rescan (the vanilla call order), spawning off the stored
`SceneMetrics` (vanilla's last-scan persistence); the draw rides the
frame's UI-range batch next to the rain.

Deviations: the RNG is the module's own splitmix stream (the port's
per-consumer convention — vanilla rolls the shared `Main.rand`); the
`NewDust` initial-velocity rolls are not consumed (`EmitDust` overwrites
both components); `gfxQuality`/`dCount` pinned `1.0`/`0` — no pool-pressure
governor, the budget and the deaths bound the population; the multiplayer
`SendData(7)` tail is MP scope; the spawn multipliers stay out (no
spawner depth); the WindPushed buff is M2-BUFF-4.

Harness notes: (1) the seed-1 desert band's ground sits ~4 tiles below
`world_surface` (361), so a standing player loses
`IsSurfaceForAtmospherics` (`ZoneSandstorm` dies with it) — verify desert
shots with `TR_NOPLAYER` + `TR_CAM=<x,y>` (the scan then centres on the
camera above the line); (2) fixed since: the `TR_POST` pass used to cover
the whole composite including the UI (the map panel showed the filter's
noise under an active storm) — the draw now splits per the vanilla order
(see "World-effect draw range" below).

Verification: 7 sand tests (the gates, the budget, the weighted colours,
the scale hold/fade, the sine, the bottom death, seed determinism);
`TR_SANDSTORM=1` desert twins SHA256-equal (two runs byte-identical,
a third run in a fresh dir matched and one earlier run raced — the known
first-run gotcha), the off-pin A/B clean.
## Fishing engine (M7-FISH-1)

The rule core of the fishing system — the power, the pond and the rolls —
without the content (the tables, the bobber flight, the rods):

- **The power** (`Player.cs:42197-42300`): `bait + pole + fishingSkill`
  plus the situational +5s (the 25 fishing buff, `canFloatInWater && wet`,
  sitting — the sitting bonus rides `PlayerSittingHelper`), scaled by
  `Fishing_GetPowerMultiplier` (`:42233-42263`): the rain ×1.2, the
  background clouds ×1.1, the early-morning/late-afternoon daylight
  (`time < 5400 || time > 48600`) ×1.3, the midday (`16200..37800`) ×0.8,
  the mid-night (`6480..25920`) ×0.8, the moon phases (the full ×1.1, the
  gibbous pair ×1.05, the crescent pair ×0.95, the new ×0.9) and the
  blood moon ×1.1.
- **The pond** (`GetFishingPondState`/`GetFishingPondWidth`,
  `Projectile.cs:21576-21642`): the liquid span along the bobber's row
  (the solids and the 10-tile world margins bound it), then the columns
  down to the floor; the honey pond counts ×1.5; the lava/honey flags
  ride the tiles.
- **The attempt gates** (`FishingCheck`/`TryBuildFishingContext`,
  `:20583-20760`): the hard 75-tile floor, the chum bonuses (+11/+6/+3
  for 1/2/3+ chums), `waterNeededToFish = 300 × atmo` with the
  atmosphere `(y - (60 + 10·num²)) / (worldSurface / 6)` (`num =
  maxTilesX / 4200`) clamped 0.25..1, the quality scaling
  (`level × quality` when the pond is short) and the depth ladder
  (0 space / 1 surface / 3 underground / 2 cavern with the 50 % re-roll
  to the surface / 4 underworld). The junk roll: both `Next(50) > level`
  and the pond below the need (`:20743`).
- **The rolls**: the bite accumulator (`AI_061` `:53821-53847` —
  `Next(300) < level` adds `Next(1,3)`, `+= level/30`, `+= Next(1,3)`,
  `Next(60)==0` adds 60; past 660 the `FishingCheck` fires), the hit roll
  `(level + 75) / 2` of 100 (`:20587-20591`) and the drop levels
  (`:21512-21574` — the 150/300/1050/2250/4500 ÷ level windows with the
  mins 2/3/4/5/6 and the crate percentage `Next(100) < 10 [+15 potion]`).

Deviations: the drop tables (`GameContent/FishDropRules/*` — the 10-file
populator), the bobber projectile's flight (`AI_061`'s water-line states),
the rod/bait items (`Item.fishingPole`/`bait` — the registry growth) and
the quest fish are M7-FISH-2; the chum bonuses take the count from the
caller (the chum projectile book is FISH-2); the RNG rides `FastRandom`.

## Growth ticks (M7-GROWTH-1)

`WorldGen.UpdateWorld`'s rate-driven tile picks (`WorldGen.cs:71540-71632`)
with the two growth families the port's content exercises:

- **The rates** (`GetWorldUpdateRate` `:72020-72030`): `min(dayRate, 24)`
  (0 while frozen); `dayRate` = the time rate ×5 while every player sleeps
  (`Main.cs:6210-6220`). The overground loop
  `maxX·maxY · 3e-5 · rate` (×1.5 raining), the underground
  `maxX·maxY · 1.5e-5 · rate`. Each overground pick rolls the alch-seed
  gate `Next(num7·100) == 0` → skip, `num7 = Lerp(151, 151·2.8,
  clamp(maxTilesX/4200 − 1, 0, 1))` — the small worlds roll 1/15100, the
  large 1/42280.
- **`GrowAlch`** (`:45777-45890`): the herb lifecycle over the tileAlch
  set {82, 83, 84}. The 82 sprout blooms on the generic `Next(50) == 0`
  (the frame 108 needing the extra `Next(2)`) — **the day/night/rain/
  depth cue arms are the chain's fallbacks**, not the only path (the
  stage-0 sprout blooms at night through the generic arm). The wrong
  liquid kills the herb unless the stage matches (the lava keeps the
  stage-5 shape, the water the 1/4 stages).
- **The crystal shards** (`hardUpdateWorld`'s arm, `:69603-69643`): the
  `CanGrowCrystalShards` stones {117, 116, 164, 402, 403} below the rock
  layer roll `Next(5)`, pick a neighbour — **the C#'s `default` case
  resolves `num3 = +1` for both switch values 2 and 3**, so the shards
  never grow upward — need the cell empty, keep the radius-6 density
  under 2 and place tile 129 with the style `Next(18)` (the big shards
  `18 + Next(6)` at 1/50).
- **The app pass** (`world_sim/growth.rs`): the 60 Hz `growth_step` after
  the TE pass, the `growth_rng` stream seeded from the world seed, the
  rain state from `rain.is_raining()`.

Deviations: the vines/moss (the grass-growth pass `:74269+`), the
tree/cactus growth and the biome spread are M7-GROWTH-2; the
notTheBees/remix loop variants are skipped (no such seeds); the
`hardUpdateWorld` shards run unconditionally in the port (the hardmode
gate waits for M5-HARDFORCE); the NetMessage sides are MP scope.

## Worldgen pass pipeline (M5-GEN-1)

The scaffold wraps the already-landed passes in the vanilla pipeline shape
(`worldgen/pipeline.rs`) without touching a single pass body or stream constant:

- **Two tables.** `VANILLA_CHAIN` declares the vanilla chain as data — 109
  `AddGenerationPass` registrations (106 for a classic world; the skyblock
  `Jungle`+`Skyblock` pair and the dual-dungeons `DitherSnakePass` are
  conditional) with the `GenPassNameID` display names, the `PassLegacy._weightMap`
  progress weights and the `WorldGen.cs` anchors. It is the model and the slot
  registry for future passes; the runtime never walks it. `EXECUTED_PIPELINE`
  declares the port's 39 steps in the pre-scaffold `generate()` call order,
  each an adapter over the unchanged `pub(crate)` pass fns.
- **The order is NOT vanilla, deliberately.** The port's execution order
  deviates from the vanilla chain in several slots (the evil band runs before
  the mushroom patches; the floating islands run near the end; the slopes
  precede the living trees; ...) — the shared xorshift streams make any
  reorder byte-breaking, and the world contract pins the order. The vanilla
  chain stays the declaration of record; unported vanilla passes simply have
  no executed step yet (vanilla's disabled-pass shape).
- **The GenVars slice** — `SurfaceMap`/`Layers`/`BiomeSpans` + the shrine
  spots — moved into `pipeline::GenVars`; the three seed constants moved into
  `pipeline::Streams` verbatim; the frame-important boot statics into
  `pipeline::boot_statics`. `generate()` is now the `Reset`-equivalent setup
  (boot statics, `Noise`, streams, the parsed config) + `pipeline::run`.
- **The config layer** (`worldgen/config.rs`): the game's embedded
  `Terraria.GameContent.WorldBuilding.Configuration.json` copied verbatim
  (it IS in the decompile root — the old ROADMAP note said otherwise) next to
  the module, parsed through a BOM/`//`-comment prefilter into typed
  lookups: `pass(name)`/`biome(name)` sections (`GameConfiguration.Get`),
  `WorldGenRange::get_random` with the exact C# scaling (WorldArea
  `maxTilesX·maxTilesY/5040000`, WorldWidth `maxTilesX/4200`, truncating) and
  the inclusive-max `Next(min, max+1)`. The runner hands every step its
  config view like vanilla `RunPass` (`WorldGenerator.cs:501`); today's
  adapters ignore it — the shape-level passes keep their own pinned constants
  (e.g. the living-tree count stays `Next(4)` per side, not the vanilla
  `Micro Biomes.LivingTreeCount 6-11`) — future passes read their options
  from there.
- **Determinism, proven.** Vanilla gives every pass a fresh
  `UnifiedRandom(worldSeed)` (`WorldGenerator.cs:497`) — pass streams are
  independent, which is what makes its fixed chain insertion-safe. The port
  keeps its three contract streams (main `seed ^ 0xA076_1D64_78BD_642F`,
  detail `seed ^ 0x5DEE_CE66_D1A4_9B5F`, structures
  `seed ^ 0x9E37_79B9_7F4A_7C15`); a FUTURE pass slots between existing
  steps only with its own stream (`Rng(seed ^ fnv(pass_name))`), never by
  joining these. The proof: `worldgen/world_hash.rs` ports
  `WorldGenerator.HashWorld` (`:517-545`) — the per-column
  `rotl13(h)*5 + 3864292196` FNV fold over every stored `Tile` field — and
  pins the constants captured from the PRE-scaffold output (512×512/seed 42
  `3510339375`, the default 4200×1200/seed-1 world `3248003836`); the pins
  held through the rewiring. The seam test additionally proved the runner
  tile-for-tile equal to the old straight-line body before it was replaced.

## Biome passes (M1.4-WGEN-1)

The overworld biomes the compact generator lacked — the jungle, the evil
band, the glowing mushroom patches, the granite/marble caves and the
dungeon — land as `worldgen/biomes.rs` in the C# pass order (before the
caves, so the caves cut through them; the dungeon carves after the
liquids to keep its shafts dry):

- **The jungle** (`JunglePass` + `MudCavesToJungleGrass`): the west-edge
  band (`beach + 40 .. width/3`) converts the dirt/stone surface band to
  the mud (59) with the jungle grass (60) on top and the mud walls (64)
  down to the rock layer, plus the leaky mud blobs past the band edges;
  the surface dips every ~40 columns carve the shallow water pockets
  (`SurfaceWaterInJungle`'s shape).
- **The evil band** (`CorruptionAndCrimson`): the stone+grass band east of
  spawn converts to the evil set — the corruption (the corrupt grass 23 /
  the ebonstone 25 / the walls 3) or the crimson (199/203/192) by
  `GenOptions.crimson`, fed from the UI's `_optionEvil` (the Random choice
  rolls from the seed's high bit, `roll_crimson`). The chasms: 3-4
  wandering widening shafts with the evil-stone sheaths, cut down past the
  rock layer.
- **The mushroom patches** (`GlowingMushroomPatches`): 1-2 domes in the
  cavern layer near the jungle — the mud cores with the mushroom grass
  (70) shells and the mushroom walls (74).
- **The granite/marble caves** (`Granite`/`Marble`): 1-2 blob caves each
  in the stone (the granite 368 / the marble 367, the walls 180/178).
- **The dungeon** (`Dungeon` + `DungeonCrawler`): the entrance tower at
  the snow side, the carved shaft, and 3 corridor levels stepping to the
  bottom with the brick lining (41/43/44 by the world's dungeon roll) and
  the dungeon walls (7).
- The tree placement grew the soil set (the jungle/corrupt/crimson/
  mushroom grasses grow trees like the forest grass).

Deviations: the port reproduces the vanilla SHAPES with its own
deterministic noise/blob machinery — the exact tile-level layouts of the
C# `ShapeRunner`/`ShapeFloodFill`/`DungeonCrawler` chains are not 1:1; the
jungle shrines, the temple, the pyramids and the biome chests are the
follow-up tasks; the hallow is the hardmode strip (M5-HARDFORCE); the
tiny worlds collapse the evil band and guard every range.

## World-effect draw range (FIX-FX-under-ui)

The weather particles (rain drops/snow/splashes/sparkles, leaf gores, the
sandstorm dust) rendered in the UI sprite range, which draws **after** the
map batches — so the rain streaks and the sand specks painted over the
minimap, the fullscreen map and the HUD. Vanilla draws them as world
content: the world, then `DrawMap`, then `DrawInterface`.

Fix: `SpriteRenderer` gained an `fx` range — the UI camera transform
(screen-space, so the G6-cached world range stays cacheable — the moving
particles must not ride it), drawn between `world` and `map_back` — and
the post path splits: `render_world_fx` (background + world + fx, cleared)
feeds the filter chain, `apply_filters` lands on the screen, then
`render_map_ui` (the map batches + the interface, loaded, no clear) draws
over the filtered frame. The vanilla filter position is exactly there —
the filters cover the world, never the map or the HUD.

`Rain::draw` (drops, snow, splashes, sparkles, the lightning flash),
`foliage::draw_leaves` and `Sand::draw` push the `fx` range now.
Verification: rain/sand/leaf shots with the fullscreen map and the
minimap — no particle over either; the sandstorm filter no longer paints
the minimap (the twins SHA256-equal); the workspace suite 1261/0.
- **The jungle chest cycle** (`GetNextJungleChestItem`, `WorldGen.cs:10170-10196`
  — the M5-CHEST-gen jungle slice): the jungle family's slot-0 main item
  rides the `JungleItemCount % 4` table {211, 212, 213, 964} with the two
  overrides (`Next(15) == 0` → 2292 first, else `Next(20) == 0` → 3017) and
  `Prefix(-1)`, replacing the depth-band weapon approximation. Recorded
  approximations: the counter advances once per *stocked* jungle anchor in
  the load-time sweep's scan order (vanilla advances once per *attempted*
  shrine chest in the `JChestX[i]` order, `ChestsInJungleShrines`
  `:17219` — the sweep cannot see the failed attempts), and the port's
  single `ctx.jungle` flag merges vanilla's flag4 (jungle, `chestStyle == 10`
  `:36089`) and flag6 (living-tree, wall 244 `:36144`) families, so the
  cycle now covers both — vanilla's slot-0 arm (`:36229`) writes `num10`
  only under `(flag23 && (flag | flag6)) | flag11`, which leaves the deep
  shrine chests' vanilla slot-0 question open for the style-dispatcher
  follow-up.

- **The chest family extras** (`AddBuriedChest`'s two `num10 > 0` arms,
  `WorldGen.cs:36236-36257` surface / `:36806-36847` deep — the M5-CHEST-gen
  flag-extras slice): the surface arm's flag6 (living-tree) extras
  (5629 on 1/3, 4429/4427 on 1/6 with the 1/2 pick, 5528 on the 2/3
  complement) and the deep arm's flag2 (ice) 3199 (1/5), flag3
  (underground desert) 4346 (1/7) + 4066 (1/15), flag4 (jungle) 3360/3361
  (the once-per-world `gennedLivingMahoganyWands` fairness — the first
  jungle chest grants unconditionally, later ones on 1/5), 4426 and 5525
  (1/10 each) — all at the vanilla draw order, replacing the approximations.
  The port's `Context.jungle` (flag4, jungle grass 60 at the anchor) and
  `Context.living_tree` (flag6, wall 244) are the split of the merged flag
  the approximations ran on: the pre-existing pottery/2198 rows keep the
  combined `wood_family` gate, the exact arms use the split families. Still
  parked: the tenth anniversary's flag11 rows (M5-SEED-1), flag5's rows, the
  remaining family rows the probes approximate (2198/669, 2197/751,
  2766/2767), and the vanilla slot-0 arm's band question (ANALYSIS's jungle
  chest cycle note).

- **The chest 21-block remaining rows** (`AddBuriedChest`'s
  `itemIndex > 0 && chestTileType == 21` block, `WorldGen.cs:37384-47500` —
  the M5-CHEST-gen flag4/flag13 slice): the flag4 pair (the 1/4 and 1/50
  rows — both chest-tile blocks carry the same pair, so they apply to
  either anchor kind), the flag13 globe (unconditional), the style-13
  pottery row (5234, its own 1/2 roll — the vanilla keeps it separate from
  the style-23-27 row), plus the re-landed style-23-27 row and flag13 water
  pair. **The re-landing is the notable part**: those two rows merged once
  (`600dafdb`) and a later merge's resolution dropped them silently — the
  determinism suite passed because the roll is only observed in chests the
  affected families stock, and no test counted the rows. The rows now carry
  their citations; the parked items stay parked (the mahogany-tile 467
  counterparts need the chest kind plumbed, the seed-flavour rows wait for
  M5-SEED-1).

- **The chest 21-block kind gates** (`AddBuriedChest`'s two
  `itemIndex > 0` blocks, `WorldGen.cs:37384-47400` (tile 21) and
  `:47466-47500` (tile 467)): the mahogany block repeats only the
  flag4/flag2/flag12 rows — the Cloud 751 stack, the flag13 globe (2195),
  the flag9 dungeon row (2192, 1/8 — the `getGoodWorld` 5515 arm parks with
  M5-SEED-1), the style 5234 rows and the flag13 water pair are 21-only, so
  the port gates them on the anchor kind (`stock_chest` now carries it, the
  sweep's `t.kind`). The port's worldgen places no 467/341 anchors (no pass
  writes that tile), so the gates only bite the hand-placed/fixture
  mahogany chests; a kind-341 anchor keeps just the repeated rows. The
  mahogany block itself needs no new rows.

- **The vanity-tree growth** (`GrowTreeWithSettings`,
  `WorldGen.cs:30388-30847` + `TryGrowingTreeByType` `:30343-30387` — the
  M5-SPREAD-1 sapling slice): the settings-driven tree grow lands — the
  sapling-stack walk, the liquid/ground/wall gates (the wall test is the
  hand-transcribed `WallID.Sets.AllowsPlantsToGrow` set, `WallID.cs:97`),
  the 7-12 height roll, the trunk's `num5` branch-shape frame table with
  the `num4` column pick and the re-roll guard, the left/right branch arms,
  the side-ground caps (the `IsTileTypeFitForTree` neighbours with the
  1/3 kills) and the 1/13 top cap, closing with `RangeFrame`.
  `AttemptToGrowTreeFromSapling`'s 595/615 arms now dispatch it (the
  Sakura/Willow profiles); the kind-20 style-10 arm grows the ash profile.
  Deviations: `treeHeightAddon`/`ignoreWalls` parked (all callers pass
  0/false); the tenthAnniversary colour roll and the errorWorld height arm
  park with M5-SEED-1; the gem sapling's 590 arm is underground-only (the
  underground dispatcher = GROWTH-2); the kind-20 styles 6-9 palms park
  (no `GrowPalmTree` port); the paint cache copies the ground's paint (the
  C#'s `UseBlockColors` coating bits ride the port's paint field alone).

- **The palm sapling arm** (`GrowPalmTree`, `WorldGen.cs:27116-27202`, wired
  through `AttemptToGrowTreeFromSapling`'s kind-20 styles 6-9 arm
  `:72838-72848` → `TryGrowingTreeByType(323)`): the palm trunk grows on the
  sand ground (53/234/116/112) behind the 2×1 ground box + the 3-wide
  `num - num2..num - 3` box, the 10-20 height roll, the base 66-column, the
  top 22·Next(4,7) column and the middle 22·Next(0,3) columns with the lean
  walk's frameY offsets. The twin port `worldgen::ocean::grow_palm_tree`
  rides the worldgen `Rng` stream; the growth pass keeps its `FastRandom`
  per-consumer stream, so the arm is ported locally here (the
  `growth::spread_grass`/`evil::spread_grass` precedent). The decompiled lean
  guard is an always-true chain (`… || true`) — the advance fires at
  `j / num3 >= 0.25` while the `Next(13)`/`Next(9)`/`Next(5)` draws stay in
  their short-circuit order. `treeHeightAddon`/`ignoreWalls`, the
  anniversary colour roll and `RangeFrame`/`NetMessage` park (documented).

- **The underground growth dispatcher** (`UpdateWorld_UndergroundTile`,
  `WorldGen.cs:73214-73614` — the M5-SPREAD-1/GROWTH-2 slice): the driver's
  underground picks run the dispatcher (the overground parity call retires):
  the clamped box, the alch/hard-mode heads, the `SpreadUnderground` grass
  growth (`TileID.cs:622`) with the 32/352 spikes, the lily→74 conversion
  (`:73296`), the five vine families (through `grow_vine_arm`), and behind
  `allowed_to_spread` the wall arms. `SpreadGrassWalls`/`SpreadDesertWalls`
  (`:74884`/`:74984`) carry the three flavour branches each — the
  `SpreadsCorruption`/`Crimson`/`Hallow` wall sets (`WallID.cs:93-95`), the
  neighbour-family gates (63-68, the corruption arm's 63/65/66/68), the
  `wallDist`-3 source scan and `Convert`'s wall chain (`:55261`/`:55176`/
  `:55100` — the Grass/Stone/HardenedSand/Sandstone/NewWall1..4 sets
  `WallID.cs:37-48` per conversion type). The cattail family landed on top
  of it (`GrowCatTail`/`CheckCatTail`, the next bullet); the pass also carries
  the creative stop-spread gate into the hard-mode head (`allowed_to_spread`
  had been hardcoded `true` there, so `M6-CREATIVE-1c`'s power never reached
  the dispatcher). The seaweed and web arms landed on top of it (the
  M5-SPREAD-1c bullet below). The dye-plant head landed 4th (the
  M5-SPREAD-1d bullet below) and the tombstone arm 5th (the M5-SPREAD-1e
  bullet below). Parked arms: the 655 life-fruit
  kill (the NPC store), the
  444/396 moss and oyster arms (`PlayerLOS`/`AnchorValid`/
  `CountNearBlocksTypes` have no ports), `TrySpawningTownNPC`, the
  net sides, the creative-power override on `AllowedToSpreadInfections` (the
  port passes the gate through `allowed_to_spread` instead), and
  the C#'s dual `genRand`/`Main.rand` streams (the port's per-consumer stream
  carries every draw — the dye head's `Main.rand.Next(2500)`/`Next(10000)`
  now draw from it too).

- **Abigail's flower** (`TryGrowingAbigailsFlower` `WorldGen.cs:73049-73073`,
  `HasValidGroundForAbigailsFlowerBelowSpot` `:73026-73046`,
  `NoNearbyAbigailsFlower` `:73076-73095`, and the tombstone arm of
  `UpdateWorld` `:72063-72067` — the M5-SPREAD-1e slice): a grave cell (85)
  rolls `Main.rand.Next(30)`; on a hit the arm draws one spot in the 20x20 box
  around the grave (`max(10, i − 10) .. min(maxTilesX − 10, i + 10)`, same on
  `j`) — twice on a Don't Starve world (`Main.dontStarveWorld` →
  `world.seeds`' `DONT_STARVE`) — and the first spot whose tile below is a
  mushroom grass 70 / ash grass 633 / `Conversion.Grass` member
  (`SET_GRASS`/`SolidTileAllowBottomSlope`) and which holds no active 624
  within the 120-tile box (clamped to the 10-tile margin) takes
  `PlaceTile(624)` with frame 0/0. The flower is now in the boot
  frame-important statics (`pipeline.rs`, `Main.cs:7366-7378`), so `reframe_area`
  keeps the 0/0 frame instead of autotiling a 1x1 flower. Deviations: the gate
  and the two spot draws share the port's single `rng` stream (the vanilla
  splits `Main.rand`/`WorldGen.genRand`), and `Check1x1`
  (`WorldGen.cs:45237-45252`, the "flower dies when its ground is converted"
  arm) is not ported — no 1x1 framing port exists yet, so a 624 whose ground
  goes away floats. Tests: `the_tombstone_grows_abigails_flower` (the box, the
  grass seat, the 0/0 frame, the no-second-flower rule) and
  `the_abigails_flower_ground_and_distance_rules` (the ground table, the
  2-tile margin, the 120-tile box).

- **The dye plants** (`plantDye` `WorldGen.cs:71286-71436`, `PlaceDye`
  `:53988-54065`, and both dispatchers' heads — the M5-SPREAD-1d slice):
  every `nactive` pick rolls `Next(2500)` (underground) or `Next(3000)`
  (overground) and, in hard mode, `Next(10000)`/`Next(15000)` for the exotic
  family (the overground exotic arm also needs `x` outside the middle
  40..60 % of the world). `plantDye` runs three passes behind the 95-tile
  margins: the surface band (`j < worldSurface`), the underground band
  (`worldSurface <= j < height − 200`), and the hanging style 7 below an
  empty cell. Each band first refuses when its ±`num` box (90, 240 exotic)
  already holds a 227 of its own `frameX < 272` / `>= 272` family, then picks
  the style by soil: dirt/grass 2/109 → 3 or 4 (`Next(4)`), jungle 60 → 2 or 4
  (`Next(2)`, the underground band's 2 / default / 1 cascade), sand 53 in
  full water → 2, mushroom grass 80 (11×21 box clear of 112/234) → 6,
  dirt/stone/clay 0/1/59 → default or 1 (`Next(2)`), any full-water cell → 5,
  the grass/moss/dirt set → `Next(8, 12)` exotic, and the hanging pass →
  style 7 under a kind-0 cell. `PlaceDye` re-checks the seat per style
  (free-above/free-below sets including the 1.4.5 697/703, the mushroom pair,
  the full non-lava/honey/shimmer liquid for style 5, the open wall for 3/4)
  and writes `frameX = 34 * style`, `frameY` 0. Deviation: vanilla's
  `Main.rand` draws ride the port's `growth_rng` (the stream-position
  deviation all growth arms share).

- **The seaweed and web arms** (`UpdateWorld_UndergroundTile`'s
  `:73272-73282` and the tail `else`'s `:73803-73806` — the M5-SPREAD-1c
  slice): `GrowCheckSeaweed` (`:58917-58973`) runs for every picked 549 and,
  on a sand 53 with a flat top (no `topSlope`, no half brick) and a free cell
  above, behind `Next(20)`. A stand out of water (`liquid < 200` on the cell
  or the one above) dies at `Next(2)`; in water the arm stands down when the
  cell above or two above is occupied, when the `Next(1)` gate fails (vanilla
  still draws it — `maxValue` is 1), when either cell above is not full water
  (`liquid == 255`), when the 9×13 box holds more than 30 seaweed, or when the
  sea floor is further than `17 - Next(20)` rows below; otherwise it writes
  the 549 above. `GrowWeb` (`:74088-74122`) takes one `Next(10)` per pick and
  fills an empty cell whose `Next(2, 4)` box touches a solid when the wall is
  62 and the cell holds no liquid. Deviations: the notTheBees/getGoodWorld
  branch of the web box test is dead (the port generates no such seeds), the
  net sides drop, and both writes go through the port's `reframe_area`.

- **The cattail family** (the dispatcher's 519 arm `WorldGen.cs:73263-73271`,
  `CheckCatTail` `:58744-58930`, `GrowCatTail` `:58974-59082` — the
  M5-SPREAD-1b slice): the arm runs on every picked 519 cell — the validity
  sweep, then `Next(2) == 0` → one growth step and a second sweep (the C#
  order). The sweep walks down to the first solid non-solid-top ground and up
  through the water column, and kills the whole run (`KillTile` +
  `SquareTileFrame` over four cells above the lowest stalk cell through the
  ground) when: the water column is deeper than `catTailDistance` (8,
  `:86957`) above the ground; the ground kind is off the frame-row table
  (2/477 → 0, 53 → 18, 199/234/662 → 54, 23/112/661 → 72, 70 → 90) or not
  `nactive()`; a foreign block or a non-water liquid sits in the column; the
  cell above the ground holds `liquid < 127` and `Next(4) == 0`; a ≥180-row
  tip sits in >127 water and `Next(4) == 0`; the base and tip rows disagree
  (a 108..162 tip is re-armed to 90 instead); or the tip stands more than 4
  above the ground with the `num+3`/`num+4` cells dry under a 519. A valid run
  writes the soil row into every stalk cell's `frameY`. The growth step walks
  the base 0 → 18 → `18 · Next(2, 5)` and then the tip arms — the submerged
  108 tip, or the dry `Next(3)` pick that either dies or takes the 126/144/162
  tip with its 180/198/216 partner — spawning the cell above and reframing
  the pair. Deviations: `SquareTileFrame` is the port's `reframe_area` (the
  3×3 autotile walk), the net sides drop, and the port's `growth_rng` carries
  every draw. The slice also fixed a boot gap it depended on: 518/519/529/549
  are frame-important (`Main.cs:7366-7378`) but
  `worldgen::pipeline::boot_statics` never marked them, so `reframe_area`
  overwrote exactly the `frameX`/`frameY` rows the growth arms write — the
  four kinds now carry the bit (the classic world hashes are unchanged: no
  worldgen pass places them before the framer runs). 3 tests.

- **The beach-shells arm** (`UpdateWorld_OvergroundTile`'s
  `WorldGen.cs:72073-72135`): the last M5-SPREAD-1 row item. The weather
  plumbing it needs landed with it — `growth::update_world` /
  `update_world_overground_tile` now carry the real rain intensity and wind
  speed (the app passes `rain.max_raining()` and `wind.speed_current`; the
  drivers before used rain as a bool only). The arm: an inactive beach cell
  under the surface (`j < worldSurface + 10`, inside the `beachDistance` 380
  margins) rolls `3000 − |wind| · 1250 − maxRaining · 1250` (floor 300), scans
  down ≤15 rows for sand on a `SolidTileAllowBottomSlope` seat, stands down on
  an existing pile, counts the 324/81 tiles in the ±`Next(8, 11)` box against
  the `Next(2, 5)` cap and writes the coral (81, on ≥230 water at 1/2) or the
  seashell (324 with the `RollRandomSeaShellStyle` roller — the ocean twin
  keeps the worldgen stream).

- **Bug 16 — tree drops** (`KillTile`'s case 5/596/616/634 →
  `KillTile_GetTreeDrops`, `WorldGen.cs:65214-65231`/`:66189-66261`):
  `growth.rs::tree_drops` ports the whole drop arm — `tree_bottom`
  (`GetTreeBottom`'s frame-column decode `:63451-63473` + the trunk walk
  `:63474-63480`), `tree_type` (the ground-family map `:63080-63111`),
  `TreeTypeDropsAcorns` (`:63112-63119`), the canopy acorn roll and the
  per-family wood variants (ash 5215 / ebonwood 619 / shadewood 911 / jungle
  620 / pearlwood 621 / mushroom 183 at 1/2 / boreal 2503). The app's
  `cell_drops` gained the tree kinds (the `genRand` stream rides the seeded
  FastRandom, the pot-event convention). Deviation: the `bonusWood` roll
  (the closest player's axe + `Main.rand.Next(3) == 0`, `:66250-66256`) parks
  with the player seam. A second deviation, recorded because the audit's fix
  plan assumed it: vanilla `KillTile` has **no whole-tree collapse** — the
  trunk above a felled base stands until struck (each struck cell dropping
  through these arms), so the report's "collapse the whole trunk" item was
  dropped against the golden rule rather than ported.

- **Bug 17 — front-end sounds** (the `FadedMouseOver`/activation cues):
  `handle_front_click` plays the flow's cue after the click table resolves —
  `Menu_Close` (11) for the Back-family clicks (`UICharacterSelect.cs:118`,
  `UIWorldSelect.cs`'s `GoBackClick`), `Menu_Open` (10) for the activations
  (`UICharacterListItem.cs:198/:248`, `UIWorldListItem.cs:365`,
  `UICharacterSelect.cs:110`). The hover edge — the front screens' framework
  element enter (`UICharacterSelect.cs:124`, `UIWorldSelect.cs:126`) and the
  legacy menu's row enter (`Main.DoStatefulTickSound`, `Main.cs:3244`) — plays
  one `Menu_Tick` per enter, tracked by the App's `front_hover_hit` (the
  framework element id) and the menu's own `hover` field. `Audio` gained a
  `played` counter (`played_count()`) as the test seam — the headless Apps have
  no output device. Deviation from the report's plan: the menu activation cue is
  `Menu_Tick` (12), not `Grab` (7) — the vanilla's Grab is the chest/inventory
  slot-movement cue (`ItemSlot.cs:759-769`).

- **Bug 18 — the emote Back label** (`UIEmotesMenu.cs:78`'s
  `UITextPanel(Language.GetText("UI.Back"), 0.7f, large: true)`): the shared
  `ui::bestiary::back_label()` accessor now feeds the emote wheel's Back panel
  (`ui/emotes.rs`) and the bestiary's own (which carried the same hardcoded
  literal), measured and drawn as one string through the existing
  `text::hud_text_width` + `uistate::draw_text_panel` path — the C#'s
  `TextHAlign 0.5` centring against the localized `_textSize.X`. Two source
  corrections: the key is `UI.Back` (not a `Lang.menu` row) and the manager's
  missing-resource fallback answers the key itself, so a locale without the
  bundled content jsons reads the key (documented; the resource loading is
  `lang.rs`'s existing path).

- **Bug 19 — the biome-cactus frame** (`DrawSingleTile`'s case 80,
  `TileDrawing.cs:1132-1147`): the port folded the `rectangle.Y += 54/108/162`
  shift into the shared `add_fr_y`, which the FLAT build path also consumes
  (`render/tiles.rs`'s source rect `fy + add_fr_y`) — so every cactus on any
  ground shifted, while the C# mutates only the local `rectangle` (`:1007`)
  whose single consumer is the half-brick arm (`:1298`). `TileDrawData` now
  carries `half_brick_src_y` for that rect's shift; the half-brick draw
  (`push_half_shifted`) applies it and the flat path stays unshifted. The
  `cactus_source_y_offset` helper stays as the data layer.

- **Bug 20 — the trash slot** (the vanilla's destroy-on-quick-trash
  container): the quick-trash — `GetAlternateClickAction`'s else-chain tail
  (`ItemSlot.cs:679-682`) lights `AlternateClickAction.Trash`'s cursor 6 on a
  Shift-held hover over a non-favorited, non-air inventory slot when no
  crafting/reforge/chest arm matched (the chain tail sits after the chest
  arm — an earlier placement stole the chest's shift-click), and the click arm
  (`OverrideLeftClick`'s cursor-6 arm `:759-775`) replaces the trash with the
  stack and empties the slot; the cues are `TrashItem` (no port asset — the
  recorded gap) and `Grab`. `on_player_death` clears the trash
  (`Player.DropItems`' first line, `Player.cs:54304`); the event-item purge
  (`:24729-24736`) rides the unported event items and the wholesale clear
  covers it. The slot's rect shifts to (453, 426) at 0.755 while
  `Main.ChestOrShopUIVisible` (`Main.cs:42349-42366`) through one
  `trash_rect` helper shared by the draw and the hit test. Deviations: the
  Journey research exception (`:762`) and the shop's sell cursor wait for
  their models.

## Structures (M1.4-WGEN-2)

The structure passes ride their own RNG stream (`opts.seed ^
0x9E37_79B9_7F4A_7C15`) — the shared worldgen stream feeds the
living-tree/tree passes after them, and consuming it early shifted the
tree placements to empty results on the small worlds.

- **The jungle shrines** (`JungleShrines`, `WorldGen.cs:15977-16140`):
  7-12 hut rings of the brick 119 (`GenVars.jungleHut`'s first pick,
  `:11319`) in the underground jungle — the carved interiors carry the
  shrine walls (24) and a torch (the tile 4, the style 3), the mud
  pillars fill under the floor (`:16092-16106`) and the sloped brick caps
  close the top (`:16108-16120`). The C# retry loop (`num4`, `:15991`)
  maps to 8 attempts per shrine — the caves carve the mud band before
  this pass, so most raw spots miss. The net brick is the shell ring
  only (~25-32 tiles per shrine — the interior does not count).
- **The chests** (`ChestsInJungleShrines` + the cavern scatter): the
  shrine spots take the chests first (`place_chest` writes the intact
  2×2 tile-21 quads with the frame corners), then the cave floors
  scatter `maxTilesX / 40` more.
- **The altars** (`DemonAndCrimsonAltars`, `:15750-15800`): the count
  `tiles × 3.3e-6` (the small worlds clamp the 281-tile ocean margins),
  the 3×2 tile 26 with the style frames, the middle-10 % exclusion; the
  body may carve the loose cells (`Place3x2`'s free placement).
- **The pyramids** (`DunesAndPyramidLocations`, `:15367+`): 1-2 stepped
  hulls of the sandstone brick 480 over the desert surface with the base
  chambers, the entrance tunnels and a chest.
- **The pots** (`PotsGraveyardsAndBoulderPiles`, `:17996+`): the pot 28
  with the 7 styles, scattered with a 20-row seat scan (the point draws
  rarely hit the cave floors — the C#'s denser caves accept more).

Deviations: the chest inventories ride the storage task (the C#
`Chest.CreateChest` book); the multi-chamber pyramid crawl, the boulder
piles and the graveyards are the follow-ups; the layouts are shape-level.

## Biome detail passes (M1.4-WGEN-3)

The decorator passes over the finished biomes — the ocean caves, the
fragile ice, the underground jungle trees, the mushroom/jungle plants and
the vines — ride their own `drng` stream (the shared worldgen stream stays
reserved for the biome + tree passes; the stream discipline bit in
WGEN-2's review: consuming it early shifted the living-tree placements to
empty results on the small worlds).

- **The ocean caves** (`OceanCaves`, `WorldGen.cs:15167-15191`): one
  winding shaft per ocean side at the `Next(3)` gate, the first-active
  surface scan, the walkable carve down past the sea floor into the
  terminal bubble room (the C# `oceanCave`'s winding shaft shape is
  approximated).
- **The fragile ice** (`FragileIceOverIceBiomeWater`, `:16667-16690`): the
  ice band's water tiles break their skin — the 1-3 wide holes at 1/4 per
  tile (the C# `MakeWateryIceThing`'s open-pocket shape).
- **The underground jungle trees** (`UndergroundJungleTrees`,
  `:17840-17856`): the `Next(10)` per column over the mud band, the 5-8
  tall trunks (the tile 5) with the leaf balls (the tile 6), rooted in the
  mud with the two-row clearance.
- **The plants** (`GlowingMushroomPlantsUndergroundAndJunglePlants`,
  `:20122-20158`): the glowing mushrooms (71) on the free mushroom-grass
  cells below the surface (the C#'s triple tree retry collapses to the
  plant placement in the port — the mushroom trees are the tile-71 stands),
  the jungle plants (61) on every free jungle-grass cell.
- **The vines** (`ActionVines` + the grass-growth vine arms): the
  overhangs hang the jungle vines 62 (the cap 20), the regular vines 52
  (12) and the mushroom vines 528 (12) at the 1/3 gate.
- The desert test learned two vanilla-legitimate exceptions: the pyramid
  entrances open the surface (`Pyramid` carves into the dunes) and the
  living trees grow on sand (`GrowLivingTree` gates only the anchor air,
  not the soil).

Deviations: the hallowed/crimson vine arms wait for the hallow
(M5-HARDFORCE); the C# `oceanCave`'s exact winding shape and the
`GrowUndergroundTree` canvas are approximated; the mushroom trees are the
tile-71 stands (the full tree generator = the follow-up).

## Crafting groups, environment, chests (M3-CRAFT-2)

The research corrected the row: 1.4.5.8 has **no Condition-object system** —
the recipe environment inputs are hardcoded bool fields on `Recipe`
(`needWater/needHoney/needLava/needSnowBiome/needGraveyardBiome/
needTorchGodsFavor/needMechdusa`, `Recipe.cs:72-100`), checked by
`PlayerMeetsEnvironmentConditions` (`:322-358`); there is no
`DataStructures/Condition.cs` and no `AddCondition` (that name lives on
achievements only). The liquid proximity rides the same 5×3 tile region as
the stations (`Player.AdjTiles`, `Player.cs:35661-35699`): `tile.liquid > 200`
by type plus the `CountsAsWaterForCrafting` tiles {172 sink, 207 fountain}.

The group system (`Recipe.SetupRecipeGroups`, `Recipe.cs:499-533`): 34
groups, the registration order is the id; an ingredient whose item belongs
to an accepted group substitutes the whole member set
(`CreateRequiredItemQuickLookups`, `:15162-15191`) and the C# folds the
members into a fake item id (`1000000 + index`,
`AddFakeCountsForItemGroups :409-416`). The port carries the accepted group
on the `Requirement` directly — the same totals without the fake-id
indirection. `Requirement::matches` reproduces `RequiredItemEntry.Matches`
(`:36-43`), so the drain (`CraftingRequests.Consume :218-254`) takes any
member by slot order: the inventory first, then each chest in collection
order, `ConsumeItemsFrom`'s partial-stack-then-clear shape (the port's
`consume_kind`/`consume_chest`).

The nearby chests are real 1.4.5 vanilla:
`Recipe.CollectItemsFromChests` (`:375-407`) = the open container + the
void bag + (with `Player.Settings.CraftFromNearbyChests`, default true,
`Player.cs:406`) the container projectiles/banks in a ±39-tile rect and the
world chests within 600 px of the player's feet
(`NearbyChests.GetChestsInRangeOf`, `NearbyChests.cs:10-31`). Port subset:
the tile chests (21/341) within 600 px over an in-memory `WorldChest` store
gated by the new `craft_from_nearby_chests` setting wired to the panel's
nearby-chests toggle. Deviations: the store's population landed with
M3-CHEST-UI (2026-11-07 — `crates/tr-app/src/chest.rs`; entries appear
lazily on first open, so a never-opened chest still contributes nothing to
the scan — "Chest UI + world chests (M3-CHEST-UI)"), the save/persistence
is M6-SAVE-2's, the banks/void bag are M3-STORE-1's; `Chest.IsLockedOrInUse`
is ported with M3-CHEST-UI but the craft scan still passes every in-range
chest (the lock gate is unwired here — a latent note, not fixed in that
task); the scan order is raster (the
vanilla iterates `Main.chest[]`'s instance order — the port has no instance
registry); the group drain walks members in the group's registration order
when the inventory pass runs per-member (the vanilla drains one flat pass
matching `Matches` — the same totals, a different slot walk).

`available` now implements `UpdateRecipeList`'s composition
(`Recipe.cs:230-259`): environment × enough-ingredients — the port's older
"station adjacent and any ingredient present" partial list is gone (the
classic pane shows craftable rows only). Station requirements inherit
`Recipe.TileCountsAs` (`:559-571`): near a work bench (17) satisfies a
sawmill (215) requirement, the chain walks 302→17→215. The 8 new subset
recipes exercise every ported flag: Bottled Water/Honey, the
Wet/Lava/Honey Bomb trio, Snow Cloud Block (Sky Mill + snow), Wood Shelf
(Bone Welder + graveyard), Glass (the Sand group); the 11 pre-existing
recipes re-authored as group entries — the old "Wood → 9 / IronBar → 22"
expansion deviation is retired. Out: the alchemy discount (M3-CRAFT-3),
the torch-god/mechdusa flags (no systems), the guide-item mode
(`CollectGuideRecipes`), the `SendData(32)` chest syncs (multiplayer).
## Floating islands (M1.4-WGEN-4)

The `FloatingIslands` pass (`WorldGen.cs:12924-13133`) — the classic sky
islands over the 10-90 % band:

- **The placement** (`:12975-13033`): the x avoids the spawn ±150 and the
  ±180 spacing from the placed islands; the y = `Next(90, worldSurface −
  100)` — `worldSurface = surface_max + 25` carries the ground margin, so
  the disc bottoms stay ~75+ tiles over the deepest terrain column. The
  open-sky clearance scans the ±75-vertical window (the C# `:13015-13028`).
- **The island body** (`CloudIsland`, `:78132-78540`): the elliptical
  cloud disc (the tile 189) with the ragged boundary (the C#'s
  `Next(80, 120) · 0.01` wobble on the rim cells), the grass cap 2 rows
  on the top surface. Every third island is a `CloudLake` sky lake — the
  rain-cloud (196) bowl sunk 3 deep with the water inside.
- **The houses** (`FloatingIslandHouses`, `:17862+`): the planks-9 box on
  the island's top with the cloud-wall (73) interior and the chest.
- **The living-tree interaction**: the islands occupy the sky the C#'s
  empty skies don't — the living-tree anchor scan starts at the
  height-map row (not from y=0, which anchored on the island discs), and
  the grow rides the anchor-attempt loop (`:15553-15555` — the clearance
  failures retry the next candidate; the port's original structure wasted
  the whole tree on one failed clearance). The desert test skips the sky
  columns wholesale (the islands legitimately hang over the dunes; the
  dirt caps read as the surface via `surface_of`).

Deviations: the snow/desert cloud-island variants
(`SnowCloudIsland`/`DesertCloudIsland`) and the house furniture sets are
the follow-ups; the disc shape approximates the C#'s shrinking-ring walk.

## Hives & hell forts (M1.4-WGEN-5)

- **The beehives** (`Beehives` + `BeeLarvaInBeehives`, `WorldGen.cs:
  15927-16000`): 1-8 hive domes in the underground jungle band (the count
  scales `maxTilesX / 4200` like the C#'s `Next(5·num, 8·num)`), the
  hive-block (225) shells with the honey pockets (the liquid 2) carved in
  the cores, roughly half carrying the larva (231) — `AddBeeLarva`'s
  3×4 clear with the hive base row (`:32050-32068`).
- **The hell forts** (`HellFort`, `:32683+`): the obsidian-brick (75 /
  the wall 14) boxes in the hell layer — the shell with the two carved
  room floors, the lava pool strips on the floors and the chests. The C#'s
  multi-room layout with the doors (`HellFort_AttemptToPlaceDoor`) is
  approximated by the two-floor shape.

Deviations: the HiveBiome/HoneyPatchBiome placement machinery is the
port's blob dome (the shape-level port); the Queen Bee summon from the
larva and the fort doors = the follow-ups.

## Biome spread (M5-SPREAD-2)

`hardUpdateWorld`'s infection arms (`WorldGen.cs:69645-69860`) — the
living world's evil/crimson/chlorophyte evolution:

- **The spread sets** (`TileID.cs:580/:589`): the corruption {23, 661, 25,
  112, 398, 400, 163, 32, 636, 24} and the crimson {199, 662, 203, 234,
  399, 401, 200, 352, 205, 201} — the corrupt grass, the ebonstone/
  crimstone, the ebonsand/crimsand, the hardened sand, the sandstone, the
  ice and the thorn kinds.
- **The Convert map** (`Convert`, `:55139-55205`): the grass family (2,
  477, 60, 69) → the corrupt grass 23 / the crimson grass 199 (the jungle
  grass 60 → 662 in the crimson), the stone + the moss → the ebonstone 25
  / the crimstone 203, the sand 53 → the ebonsand 112 / the crimsand 234,
  the ice 161 → the purple 163 / the red 200, the hardened sand 397 →
  398/399, the sandstone 396 → 400/401. The natural walls convert too —
  the stone wall 1 → the ebonstone wall 3 / the crimson wall 192, the
  dirt wall 2 → 69/193 (the natural-wall slice of the C#'s
  `WallID.Sets.Conversion` tables).
- **The spread loop** (`:69696-69860`): the ±3 random target, the
  chlorophyte defence (`nearbyChlorophyte` within 5 → the loop breaks —
  `ChlorophyteDefense` itself converts only in the remix worlds,
  `:69456-69460`), the sunflower (27) suppression within 2 tiles
  (`CountNearBlocksTypes(i, j, 2, 1, 27)`), and the conversions with the
  `Next(2) == 0` re-roll chain (the spreading tile walks the ±3 window
  converting until the roll fails).
- **The chlorophyte** (`:69646-69685`): the 211 chlorophyte + the 346
  chlorophyte-jungle-grass sources grow into the mud (59) and the jungle
  grass (60) via `Convert(..., 8)`, gated by `CanChlorophyteGrow`
  (`:69464-69500` — the chlorophyte count in the radius-35 box ≤ 40 (the
  box ×1.5 and the count cap /2 above the rock layer) and ≤ 130 in the
  radius-85 box) plus the ±6 random re-scan.
- **The gates**: the post-PlantBoss worlds halve the spread (`Next(2) !=
  0`, `:69688-69691`); `AllowedToSpreadInfections` (the creative
  spread-stop power) freezes it.
- **The app call**: the spread rides the growth pass's underground picks
  (`world_sim/growth.rs`) — one `hard_update_world` per tick on a random
  cavern-layer column.

Deviations: the thorn bushes (32/352) spread by the set membership but
grow no bush tiles (the plant placement = the follow-up); the wall
conversion covers the natural-wall slice (the C#'s tables span the brick
walls too); the remix `ChlorophyteDefense` conversion is out (no remix
worlds); the hardmode gate waits for M5-HARDFORCE (the port spreads
unconditionally).

## Lihzahrd temple (M1.4-WGEN-6 / M5-TEMPLE)

The temple is now the faithful C# port, landed as three pipeline steps over
the M5-BIOME seam:

- **The placement search** (`LihzahrdTemple` pass, `WorldGen.cs:15834-15926`):
  the y between `(int)Main.rockLayer` and `maxTilesY − 600`, the x from the
  dungeon-opposite reach `((NextDouble() · num3 + 0.1) · −dungeonSide + 0.5) ·
  maxTilesX`, the tile-60 acceptance, the 1,000,000-attempt widening valve
  (0.25 → 0.30 → 0.35, 11 cap stages) and the fallback — the M5-BIOME
  phase-6 port, unchanged.
- **The interior** (`makeTemple`, `:33435-34237`): the room walk
  (`Next(10,16)` rooms at `scale = maxTilesX/4200`, the staggered
  horizontal/vertical steps with the `Next(1,3)` flip period and the −10
  downward drift), `makeTemple_GenerateBricks` (the per-depth `DungeonBounds`
  union, `Inflate(10)`, the `ClearEverything` brick fill and the four
  corner-to-corner `Line_StrictAngles`/`Line`/`FillY`/`FillX` walkers), the
  jitter carve (the two counter-sweeps over the drifting inset edges), the
  `templePather` corridors, the four `outerTempled` sweeps (the unsafe-wall
  skin conversion), the entrance shaft with the Lihzahrd door (tile 10,
  style 11, `frameY = 594/612/630`), the `templeCleaner` sweeps, the wall-87
  widen pass, the altar (`PlaceTile(237)` → `Place3x2`, the published
  `lAltarX/lAltarY` from the frames, the hand-built 1000-try fallback) and
  the wooden-spike (232) staircase loop.
- **The part-2 content** (`LihzahrdTemplePart2` pass, `:17701-17709` →
  `templePart2`, `:34464-34658`): the dart traps (`mayanTrap` — the pressure
  plate 135 style 6 over the dart column 137 with the `AddWireFromPointToPoint`
  wire run), the buried chests (`AddBuriedChest`, style 16, the Pygmy-staff
  main item is M5-CHEST-gen's), the statues (105, styles 43-45), the
  workbench/table/chair furniture (`Place2x1`/`Place3x2`/`Place1x2` with the
  faithful frame strides) and the one painting (240, style 88).
- **The altar re-write** (`LihzahrdAltar` pass, `:21943-21976`): the 3×2
  frames + the 3-wide brick base row + six `SquareTileFrame`, as the late
  anchor safety.

Deviations:

- **The stream model**: the C# shares one `genRand` across the world; the
  port re-seeds per pass (`pass_rng("Jungle Temple")` / `"Temple"` /
  `"Lihzahrd Altars"` — the step slugs map to the vanilla chain rows
  "Jungle Temple" / "Temple" / "Lihzahrd Altars"). The draws INSIDE each
  step follow the C# statement order 1:1 (pinned by the draw-ledger tests);
  the loot tables' draws in `AddBuriedChest` are M5-CHEST-gen's — until they
  land, the part-2 stream carries their absence (a documented sub-vanilla
  gap, same class as the earlier phases).
- **The `tileSolid` windows**: the port has no global table. The part-2
  wrapper runs with 162 (BreakableIce) non-solid / 226 solid for the whole
  step, and the wood spike 232 solid only during the trap loop (the C#'s
  `:34499` flip); the C# leaves 162/232 non-solid for the rest of the chain
  until `BeeLarvaInBeehives` — the port's later passes keep their own
  predicates (recorded delta; the temple's spikes are the only 232 tiles).
- **The executed-pipeline order**: `temple part 2` runs immediately after
  `lihzahrd temple` (the C# runs it after the liquid/tile passes — the port's
  compact chain delta), and `lihzahrd altar` sits between `cacti` and
  `water plants` (vanilla `:21310` → `:21943` → `:21963`). 50 → 52 steps.
- **The sub-vanilla guards**: the C#'s unguarded writes (`templePather`'s
  carve squares, the altar fallback's floor scan, `array[num2-1]` on a
  world whose walk rolls no room) would throw on the port's smaller test
  worlds; the port's `in_world` guards keep the writes total.
- **`PaintTheTemple`** (`:24529`) is celebrationmk10-only
  (`FinishTenthAnniversaryWorld` runs under `Main.tenthAnniversaryWorld`) —
  M5-SEED-1's scope, not a deviation here.
- **The drunk/good/remix branches** (×3/×2 room counts, the drunk spike
  second row, `placeTNTBarrel`'s twin) and the dualDungeons 232 variants are
  M5-SEED-1/2's scope.
- **The Golem-summon altar interaction** (Plantera gate) is M5-HARDFORCE /
  Golem-BOSC-19's; the traps fire through the wiring engine's case-137
  variants (the frameY selects the variant, the trap column's wires are
  laid by the pass itself).
- **The world hashes** are re-pinned (every world on every seed changes by
  design): `SEED42_512 = 1248943602`, `DEFAULT_WORLD = 3359703751` (the
  phase-3/final pair; the phase-1 pins 2067651109/2366599377 and the phase-2
  pins 2109138558/948698174 are recorded in the commits). Screenshot
  determinism pair (seed 1, the altar room at `TR_CAM=1836,703`
  `TR_SHOT_FRAME=600` `TR_NOPLAYER=1 TR_NO_BG=1 TR_DAY=0.5`): zoom 1.0
  `8a75a259…` and zoom 2.0 `352859a2…`, each byte-equal on its repeat run
  (taken from the merged `main` build); the worktree-built pair differed
  from these by 900 px (max channel 9, the near-black altar-room corner) —
  a cross-build f32-order delta in the render path, not a world-state or
  determinism difference (the world hashes are pinned by the tests). A
  zoom-2.0 frame-900 pair separately showed the known `TR_SHOT` harness
  timing sensitivity (the ANALYSIS harness note) in the HUD hotbar region
  only.

## Polish passes (M1.4-WGEN-7)

The worldgen rounding-out:

- **The dungeon colour** (`pass_dungeon`): the brick roll (the blue 41 /
  the green 43 / the pink 44) now drives the walls too (7/8/9) — the
  port's walls were pinned to the blue 7.
- **The ice shell** (the fill): the snow band's underground is the ice
  (161) from the surface to the rock layer (the `IceBiome` shape — the
  port's fill previously kept the dirt under the snow surface, leaving
  the ice-biome decorators nothing to sit on).
- **The exposed gems** (`ExposedGemsInIceBiome`, `WorldGen.cs:20697-20735`):
  `maxTilesX · 0.25` attempts — the y between `(worldSurface +
  rockLayer) / 2` and the lava line, the x in the snow band, the anchor
  tile from the snow/ice family (147/161/162/224); the gem patch (the
  tile 178) fills the `Next(1, 4)`-half-sized rectangle of the free
  cells, the styles weighted `Next(12)`: <3 → 0 (the amethyst), <6 → 1,
  <8 → 2, <10 → 3, ≥11 → 5 (the diamond), else 4.

Deviations: the gem torches (the ice biome's torch placements) and the
ice-cave water pools are the follow-ups; the ice shell's dirt pockets
ride the cave pass's blob machinery.

## The alchemy discount (M3-CRAFT-3)

`AddRecipe` auto-sets `Recipe.alchemy` iff `requiredTile == 13`
(`Recipe.cs:16722-16726`); `Player.SetAdjTile` raises `player.alchemyTable`
for tiles 355/699 (`Player.cs:35647-35649`) — the `TileCountsAs` pairs
355→13/699→13 (CRAFT-2) already satisfy a placed-bottle station requirement
near the table. The discount itself: `GetIngredientCraftingDiscount`
(`Recipe.cs:214-228`) rolls `Main.rand.Next(3) == 0` per ingredient unit and
`GetIngredientsForOneCraft` (`:196-212`) subtracts the survivors from the
consumed stacks — the availability check counts full stacks (the split is
the vanilla's). Port: `try_craft` subtracts the rolled-off units from each
requirement's drain (group entries discount on the folded stack), the rolls
come from the App's `craft_rng` stream (the per-consumer RNG convention —
vanilla rolls the shared `Main.rand`; the seed is fixed like the other
streams, so crafting is seed-deterministic). The subset gains the two
placed-bottle potions: Lesser Healing Potion ×2 (1 Mushroom + 2 Gel +
2 Bottle, `Recipe.cs:1586-1597`) and Healing Potion (2 LHP +
1 Glowing Mushroom, `:1606-1611`). The 1/3 rolls consume stream state, so
the discount depends on every prior craft — the documented determinism
caveat mirrors the vanilla (Main.rand's shared stream).
## Hardmode switch (M5-HARDFORCE)

The generation half of the hardmode switch — `WorldGen.StartHardmode` →
`initializeHardMode` (`WorldGen.cs:31570-31697`):

- **The runners** (`GERunner`, `:76169-76330`): the V-shaped strip walker —
  the position starts at the sky, the velocity `(3·num5, 5f)` per step
  with `num5 = ±1` (`:31585-31590` — the 3:5 slope verbatim), each step
  converting the Manhattan disc of the radius `Next(200, 250) · scale ·
  0.5` with the rim wobble `Next(-10, 11) · 0.015`.
- **The hallow map** (the good runner): the hallowed grass 109 (from the
  grass 2 and the corrupt/crimson grass 23/199), the pearlstone 117 (from
  the stone 1 / the ebonstone 25 / the crimstone 203), the hallowed sand
  116 (from all the sands 53/123/112/234), the hallowed ice 164 (from
  161/163/200), the hallowed sandstone 403 and the hardened 402 — plus
  the purify arm: the corrupt/crimson jungle grass 661/662 revert to the
  jungle grass 60 (the hallow and the evil cancel on the jungle). The
  walls: the sandstone wall 187 → 222, the ebonstone/crimstone walls
  3/83 → the hallowed stone wall 28.
- **The evil runner** reuses `Convert`'s map by the world's flavour (the
  corruption or the crimson — `initialize_hard_mode(corruption)`).
- **The app trigger**: `TR_HARDMODE=1` (the env dev hook) sets
  `World.hard_mode` and runs `initialize_hard_mode` once — the WoF kill
  rides the M4-NPC families.

Deviations: the wall-lining pass (the 25·scale cave cavities lined with
the hallowed walls 200-203, `:31668-31697`) is the follow-up; the altar
ore drops (the adamantite/etc.) and the WoF summon/kill are the
M4-NPC/M4-LOOT tasks; the remix jungle-V arm is out (no remix worlds).

## Hardmode ores (M5-HARDFORCE-ores)

The altar-smash ore seeding — the second generation half of the hardmode
switch (`WorldGen.cs:48920-49090`):

- **The tiers** (`:48947-49040`): the altar count % 3 → the cobalt 107 /
  the palladium 221 (the count ×1.05), the mythril 108 / the orichalcum
  222, the adamantite 111 / the titanium 223 (the alternates bump the
  blob strength +9 via `num4` and the count ×0.9); the 50/50 rolls save
  into `SavedOreTiers` (−1 = not rolled).
- **The blob count** (`:48935-48937`): `(maxTilesX/4200 · 310 − 85·tier)
  · 0.85 / (tier + 1)` — the tier-3 blobs need the full-scale world (the
  small worlds genuinely get 0 blobs for the adamantite — the C#'s
  minimum 4200-wide world yields ~39).
- **The bands** (`:49045-49058`): the cobalt/palladium from
  `worldSurface`, the mythril/orichalcum from `rockLayer`, the
  adamantite/titanium from `(2·rockLayer + maxTilesY) / 3`, all down to
  `maxTilesY − 150` — the bands stay separated by construction.
- **`OreRunner`** (`:48831-48900`): the random walk with the velocity
  `Next(-10, 11) · 0.1`, placing the shrinking Manhattan discs —
  `strength · (steps_left / steps)` with the rim wobble
  `Next(-10, 11) · 0.015` — over the replace set {0, 1, 23, 25, 40, 53,
  57, 59, 60, 70, 109, 112, 116, 117, 147, 161, 163, 164, the moss,
  199, 200, 203, 234, 396-403, 225 unless the hive wall 108}.
- **The app smash**: the hammer hit (the mining path) on the altar 26 in
  hardmode runs `smash_altar` — `World.altars_smashed` counts, the App's
  `ore_tiers` saves the rolls.

Deviations: the hammer-power gate (≥ 80) and the WoF prerequisite are the
follow-ups (the port gates on `World.hard_mode`); the drunk-world tier
swapping is out; the ore walls (`wallType`) are out.

## Micro-biomes (M1.4-WGEN-8)

The micro-biome decorators over the finished world:

- **The cave houses** (`CaveHouseBiome`, `:34-92`): 8-12 wood-framed
  cabins in the underground dirt/stone band — the 25-tile chest/wire
  exclusion scan (`:38-52`), the wood-frame box (the tile 27), the carved
  interior with the wood walls (9), the platform row (19), the torches and
  the chest (`place_chest`'s quads). The C#'s `HouseBuilder` chest
  variants (the ice/jungle/granite/marble/mushroom/desert chances) resolve
  to the plain gold chest — the port's chests hold no inventories yet.
- **The desert campsites** (`CampsiteBiome`, `:9-55`): the disc of the
  radius `Next(6, 10)` with the solid-count gate (the solid cells within 5
  of the total), the tent tiles (196 + `Next(4)`) replacing the sand
  family {53, 396, 397, 404}.
- **The jungle spike pits** (`SpikePitBiome`, `:10-57`): the shrinking
  stone circles offset down (the C#'s 4+4+4 circle walk with the blotch
  modifiers), the carved hollows — the spike tile placement is the
  follow-up (the jungle spikes = the wiring trap family).
- **The sword shrines** (`EnchantedSwordBiome`, `:19-113`): the 1-wide
  shaft down from the surface into the buried 7×5 chamber with the sword
  tile (197 — the C#'s sword sprite tile; the breakable item drop = the
  follow-up).

Deviations: the HouseBuilder room variants (the biome walls, the furniture
sets) are the shape-level approximation; the spike tiles and the sword
item drop are the follow-ups; the structure-map exclusions are the
per-pass neighbour scans.

## Ore pools (M1.4-WGEN-9)

The pre-hardmode ore alternates — the vanilla pairs roll one kind per
world (`TileID.cs`: the copper 7 / the tin 166, the iron 6 / the lead
167, the silver 9 / the tungsten 168, the gold 8 / the platinum 169):

- **The pool roll** (`OrePools`/`roll_ore_pools`): the 50/50 draw per
  pair on the shared worldgen stream at the ore pass — the rolled kinds'
  veins generate, the other pair's kinds are absent from the world.
- **The gem veins**: the amethyst 63 .. the diamond 68 — the small 1-3
  blobs in the stone band (280 attempts), one kind per vein.
- The dungeon colour test accepts the roll (the biome test's stream
  shifted with the new draws).

Deviations: the TileRunner-strength bands (the C#'s vein shapes carry the
depth-dependent strengths) are the follow-up; the ore walls are out.

*Superseded by "OresAndShinies fidelity (M5-ORES-1)" below — the
TileRunner-strength follow-up landed and the roll moved into
`ores.rs::SavedOreTiers` + `worldgen/caves.rs::roll_tiers` (which also
fixed the flip polarity: the C# rolls the *alternate* on `Next(2) == 0`,
the pools had it inverted).*

## OresAndShinies fidelity (M5-ORES-1)

The ore step now runs the vanilla `OresAndShinies` pass (`WorldGen.cs:
13182-13606`, the normal branch `:13397-13603`) on the M5-GEN-1 scaffold
— step 12 of 39, the same stream, the same anchor:

- **The tier coin-flips** (`roll_tiers`, `WorldGen.cs:11274-11297`): four
  `Next(2)` draws in the C# order copper → iron → silver → gold, the
  *alternate* on `== 0`, writing the new `SavedOreTiers.{copper,iron,
  silver,gold}` (the C# class carries all seven metals, `:3283-3297`).
  The `dontStarveWorldGen` gate on iron/gold is dead without the seed;
  the `GenVars.copperBar`-style bar mirrors need no roll (the port's
  recipes carry both bar families statically).
- **The `TileRunner` port** (`worldgen/tile_runner.rs`, `WorldGen.cs:
  76770-77142`): the velocity walker with the shrinking Manhattan disc,
  the per-candidate wobble draw testing against the *original*
  `strength * 0.5`, the frame-important skip, the `flag4` protection
  chain (`CanBeClearedDuringGeneration` — generated by `tools/GenTileData`
  from `TileID.cs:573` — plus the tileStone / sand-53 / ebonstone-family /
  sandstone-yields-to-ore cases), the type-write without activation, and
  the flat threshold-table speed-up ladder. The normal-world draw ledger
  (2 velocity draws, the no-op `Next(4)` lottery, the wobbles, 2 drift
  draws per step) is pinned by a stream-equality test. Unported until a
  caller needs them: the `mudWall` branch, the `type < 0` liquid path,
  the 367/368 walls, the `type == 59` branches reading
  `Main.rockLayer`/`GenVars.waterLine`, and every secret-seed modifier.
- **The band table** (`WorldGen.cs:13399-13573`): twelve
  `(kind, density, y0, y1, strength, steps)` loops with the per-tile-area
  blob counts and the `GenVars` quartet bounds (truncated like the C#
  casts), closed by the evil demonite/crimtane band from
  `Main.rockLayer`. The remix branch (`:13189-13396`) differs only in the
  silver/gold/evil bands and is out until remix worlds exist; the drunk
  per-blob re-rolls are dead without drunk worlds.
- **The `GenVars` quartet**: `Layers` gained
  `surface_low/high` + `rock_low/high` (`TerrainPass.cs:230-234`) —
  min/max of the per-column surface and of the model's per-column rock
  start `top + dirt_depth`.
- **The C# ≥ 20 recentering** (`TerrainPass.cs:217-228`) is deliberately
  skipped: it guards a degenerate case of the C#'s deep rock walk (the
  rock line sits 6-35 % of the world height below the surface), which the
  compact model's shallow line (16-22 tiles) would trigger on every
  world, dragging `surface_high` below the mean surface and shifting the
  surface ore bands underground. A normal C# world never recenters, so
  the raw quartet reproduces the normal-world band geometry.
- **The gems and hellstone** keep their shape-model placement after the
  ore bands in the same step — vanilla runs them in the separate `Gems`
  (`WorldGen.cs:15050`) and `Underworld` (`:13805`) passes.
- **The world hashes re-pinned** (`world_hash.rs`): every world on every
  seed changes — the pre-M5-ORES-1 output was the recorded
  approximation, not a byte contract with vanilla.

Deviations: the remix/drunk/secret-seed branches above; the
`SavedOreTiers` pre-hardmode quartet is not persisted in `.trwld`
(vanilla saves it in the world header — a save-migration follow-up); the
gems/hellstone placement (their vanilla passes are other rows). Found
during the work: the underground jungle canopy places bare active
*iron-ore* tiles (`biome_detail.rs` cites "TreeBranches" for kind 6 —
in vanilla kind 6 is the iron ore; the canopy belongs to the jungle-tree
tile family) — pre-existing, needs its own row.

## Progression recipe batch (M3-CRAFT-4)

The first progression batch over the group-enabled table (20 recipes with
the `Recipe.cs` registrations):

- **The wooden tools** (the work bench 18): the sword 24 = 7 Wood (the
  group `:6800-6806`), the hammer 196 = 8 Wood (`:6810-6814`), the bow 39
  = 10 Wood (`:6815-6818`), the arrow 40 ×25 = 1 Wood + 1 Stone
  (`:2077-2082`); the platform 94 ×2 = 1 Wood by hand (`:2376-2378`).
- **The bars** (the furnace 17): the copper 20 = 3 Copper Ore 12
  (`:9652-9656`), the silver 21 = 4 Silver Ore 14 (`:10076-10080`), the
  gold 19 = 4 Gold Ore 13 (`:10236-10240`) — the ore item ids are the
  vanilla pair (the iron 11 already in the table).
- **The metal armor** (the anvil 16): the copper set (the helmet 89 = 12
  bars `:9708-9712`, the chainmail 80 = 20 `:9714-9717`, the greaves 76 =
  16 `:9718-9721`), the iron set via the iron-bar group (the helmet 90 =
  15, the chainmail 81 = 25, the greaves 77 = 20, `:9882-9895`), the
  silver set (91/82/78 = 15/25/20, `:10127-10140`), the gold set
  (92/83/79 = 20/30/25, `:10287-10300`).

Deviations: the iron armor requirements author the iron bar 22 with the
accepted IRON_BAR group (the C#'s `RequireGroup` shape — the group
members 22/704 both accepted); the remaining ~1950 C# rows wait for the
SetupRecipes generator.

## Craft grant sound + the utility batch (M3-CRAFT-5)

The craft sound: `CraftingEffects.SpawnEffects_BeforeGrantingItem` fires
`SoundEngine.PlaySound(7)` before the result enters the inventory
(`CraftingEffects.cs:51`, called from `OnCraft` — the grant happens once
per successful craft, not per ingredient consumed). Legacy id 7 = Grab
(`SoundID.cs`), the `Sounds/Grab.xnb` cue — the port adds the
`("Grab", 7)` cue to `SFX_CUES` and fires `audio.play("Grab")` at the
successful `try_craft` in `handle_crafting_clicks` (the 2D `play`, no
attenuation — the C# passes no position).

The utility batch (7 recipes, each verified against its `SetupRecipes`
row; `ItemID.cs` resolves the ids — the wiki-familiar ids are wrong in
several places):

- **Chain 85 ×15** = 1 Iron Bar (the group), at the anvil 16 — 1.4.5.8
  makes **15** per bar (the older 10-stack is the stale number).
- **Sawmill 363** = 10 Wood (the group) + 2 Iron Bar (the group) + 1
  Chain, at the work bench 18.
- **Loom 332** = 12 Wood (the group), at the sawmill 106.
- **Sign 171** = 6 Wood (the group), at the work bench 18.
- **Cooking Pot 345** = 10 Iron Bar (the group) + 2 Wood (the group), at
  the anvil 16.
- **Bowl 356** = 2 Clay Block 133, at the furnace 17 (the glass is NOT
  the ingredient — that is the pre-1.4 recipe).
- **Gold Candle 105** = 1 Gold Bar 19 + 1 Torch 8, at the work bench 18.

The anvil 16 and the sawmill 106 enter the station list for the first
time — `scan_adjacent` collects every active tile kind, so the new
stations needed no engine change. The chain/sawmill pair makes the
station chain craftable end-to-end from the vanilla iron bar.

Deviations: the platinum candle variant waits for the platinum-bar rows
(the generator batch); the cauldron 1791/piano 333 skip (the cauldron
crafts at the cooking-pot tile, the piano needs bone + book).

## The accessory effect flags (M3-EQUIP-3)

The vanilla chain: `Player.Update` → `ResetEffects` (`Player.cs:25491`) →
`UpdateEquips` (`:25633`) → for the slots 0..9
(`UpdateEquips_CanItemGrantBenefits`, `:12869-12890` — the slots 0/1/2
require the head/body/leg items, the slots 3..9 require `Item.accessory`):
`GrantPrefixBenefits` + `GrantArmorBenefits` (all the slots), and
`ApplyEquipFunctional` (the slots 3..9). The grants are the literal
`type == N` chains — no `Item.UpdateEquip` virtual exists in this
decompile. The consumers run later in the same tick (the movement
`:26688`, the jumps `:26744`, the fall damage in `Update`), so the
same-tick grants are visible.

The port: `Player.equipped: [u16; 20]` (the kinds snapshot filled from
`App.equip.armor` per tick in world_sim/player.rs), and the grant pass
inside the update's reset block — the flags clear then the equipped kinds
re-set them, exactly the `ResetEffects` → `UpdateEquips` → consumers
order. The grants (the verbatim sites): the bottles arm their
`has_jump_options` (`:14997-15001`, :14793-14811), the balloons set
`jump_boost` (the horseshoe balloons + `noFallDmg`), Hermes/Flurry
`acc_run_speed = 6.0` (the vanilla leaves `maxRunSpeed` — the boots widen
the acceleration window; the continuation covers the top speed), the
Rocket Boots `rocket_boots = 1`, the Ice Skates `ice_skate`, the Aglet /
Anklet `move_speed += 0.05 / 0.1`, the Lucky Horseshoe `no_fall_dmg`, the
Cobalt Shield `no_knockback` (a new field gating `Hurt`'s knockback — the
vanilla `Player.cs:38519`), the Band of Regeneration → the
`UpdateLifeRegen` base via the new `accessory_life_regen` → the regen's
`extra_regen` input.

The generic `Item`-stat-driven grants (`Player.cs:13121-13122`:
`statDefense += armorPiece.defense; lifeRegen += armorPiece.lifeRegen`)
ride the port's existing flows: the defense via the M3-EQUIP-2 block (the
whole `equip.armor` array's `ItemDef.defense` sums — the Shackle's +1 and
the Cobalt Shield's +1 already flowed), the lifeRegen via the id-keyed
`accessory_life_regen` (the port's `ItemDef` has no `lifeRegen` field).

The debug pins (`TR_JUMPS`/`TR_BOOTS`/`balloon`/`TR_WINGS`/`TR_DASH`)
moved to the `pinned_*` fields re-applied after the flag clear — the
vanilla's grants re-set per tick too, so the pins remain the equipment
stand-ins.

Deviations: the Obsidian Skull's `fireWalk` (the hot-tile touch damage is
not ported — `Collision.cs:3220`), the Shark Tooth's `armorPenetration`
(the NPC-defense consumer is M4's), the Band of Starpower (no grant site
in this decompile's equip pass — do not invent; the real-vanilla +20 max
mana is absent here), the wings' `wingSlot` grant (no `ItemDef` field —
M3-EQUIP-4's plumbing; M3-PREFIX's `GrantPrefixBenefits` landed — the
62-80 accessory prefixes ride the `equipped_prefix` snapshot in the
grant pass), the vanity slots (the vanilla grants the
functional 0..9 only), the balloon combos beyond the id list (the
exclusive-combination items land as their ids appear).
### M1.4-TO-4 merge sync (2026-11-06)

- The M4-TOWN furniture merge registered the piano 87, which re-routed the
  DW-2.6 actuated-table fixture: a lone tile-87 cell is now a registry kind
  (`check_3x2`), so the fixture stages an intact style-0 registry table (14)
  with its own support row; the pinned semantics (the `CheckOnTable1x1`
  anchor disjunct reads `active()`, not `nactive()`) are unchanged.
- The M1.4-WGEN-10 `tile_drop(187) -> 989` arm and this task's empty-group
  `None` were reconciled onto `Check3x2`'s `case 187` (`WorldGen.cs:49828-49839`):
  only the sword-in-stone styles (frameX 918..=970) drop — 1/30 Terragrim 4144
  else the Enchanted Sword 989 on the `Main.rand` stream; the rest stay in the
  empty `GetItemDrops` group.

## The furniture logic (M1.4-TO-3)

The registry grew from the wooden set (10/11/14/15/18/21) to the common
furniture family — `tileobject.rs`: the candle (33, StyleOnTable1x1 —
`AnchorBottom = Table`), the candelabra (100, Style2x2+Table), the
chandelier (34, 3x3 hanging — `AnchorTop (SolidTile|SolidSide, 1, 1)` +
`AnchorBottom Empty`, the ceiling-anchor path), the hanging lantern (42,
Style1x2Top + the platform alternate), the lamp (93, Style1xX), the
bookcase (101, Style3x4), the piano (87)/dresser (88) (Style3x2 — the
dresser with the chest's invalid-anchor tiles + `LavaDeath=false`), the
sink (172, Style2x2+18px), the grandfather clock (104, Style2xX h5), the
bed (79)/bathtub (90) (Style4x2 + the `PlaceRight` alternates with
`Style = 1` + the bed's `CoordinatePaddingFix (0,-2)`). The table's
sub-tile styles 13 (Obsidian, lava-safe) / 25 (Dynasty, 3x1) resolve
through `get_tile_data`'s sub-tile walk (the row's "3x3/6x3 + styles 13/25"
gap — the vanilla has no 3x3/6x3 furniture kinds; the row's ids were stale:
torches 4 (not 8), tables 14 (not 19), beds 79, pianos 87, clocks 104).

The kill checks: `CheckOnTable1x1` (:45237), `Check1xX` (:37788), `Check2xX`
(:37993), `Check3x4` (:50076), `Check1x2Top` (:46706), `Check4x2`
(:47408), `Check2x2` (:48397), `Check3x3` (:53059) ported per-shape;
`Check3x2` (:49187) parametrized over 14/87/88. The per-cell scans set
`flag` and CONTINUE (no early break — an early break loses the sweep's cell
list and leaves the remainder alive). The `Check*` closing `TileFrame`
rings exist per-check in vanilla — only `Check3x2` (:49841-49849),
`Check3x4` (:50161-50168) and `Check4x2` (:47499-47506) close with one; the
port walks the ring for exactly those (with a depth cap), so a neighbouring
piece whose anchor died inside a sweep reaches its own `Check*`; the other
checks' remainders float until a later break — the vanilla deferred-pop
semantics (DW-3-1's guard test pins it).

The interactions (`edit::interact_at`'s `Interaction` arms): the candle
lit-toggle (33/49 — `frameX ± 18`, `Player.cs:32378-32390`; the flame
overlay and the light emission follow the frame), the chair sit (15 —
`PlayerSittingHelper.SitDown`'s seat decode `PlayerSittingHelper.cs:163-182`
+ `:317-319`; the player's bottom snaps to the seat, the facing flips with
`frameX`, `UpdateSitting` stands up on any control input or the chair going
away, the regen reads the `sitting_or_sleeping` flag `:19232-19236`), the
bed (79 — the bottom half sets/removes the spawn through `CheckSpawn`
(:54533-54590: the bed above, the 3x3 free, `StartRoomCheck` via the
housing module) with the "Spawn point set!/removed!" texts; the top half is
consumed-but-inert — the sleep state is not ported), the grandfather clock
(104 — the `h:mm AM/PM` read, `Player.cs:32725-32759`).

Deviations: the sit POSE (the sliced-leg render, `DrawSittingLegs`) and the
sleep state landed with M7-INTERACT-1 (the section below); the
multiplayer seat-stack manager is out (one player); the chest/dresser
inventories stay the M3-CHEST-UI seam; the exotic sub-tile styles (the
metal/lava variants 8/42/56-64 &c.) wait for the full-registry generator —
the style-0 wooden slice places. The DrawYOffset fields are preview-only
(`TileObject.cs:864` — the only consumer), so the deferral is invisible.

## The sit/sleep poses and the bed spawn (M7-INTERACT-1)

The furniture interaction family completed: the right-click chain's
chairs/benches/thrones/picnic tables/toilets sit, the beds sleep and set the
spawn, the spawn survives saves and re-validates at revive.

**Sitting** — the full `PlayerSittingHelper.GetSittingTargetInfo`
(`PlayerSittingHelper.cs:144-321`): the per-kind anchors (chairs/toilets
15/497 share the arm with the style-27 seat drop; thrones 102 the
`frameX/18`+`frameY/18` shifts; picnic 487 the `frameX % 72 / 18` pick; the
benches 89 the per-style `frameX / 54` seat-offset table with the `zero2`
X-nudge and the `zero` (-1, 0) position shift), the 40-px snap gate
(`IsWithinSnappngRangeToTile`, `Player.cs:17365-17372` — the tile centre to
`(Center.X, Bottom.Y - 16)`), and `UpdateSitting`'s exit mask
(`PlayerSittingHelper.cs:62-65` — any control input, the mount, or a facing
mismatch; the port's merged jump binding covers `controlUp`, W/ArrowUp feed
`control_jump`).

**Sleeping** — `PlayerSleepingHelper`: `StartSleeping`'s anchor decode
(`:192-224`, the anchor column `leftmost + 1` unflipped / `+ 2` flipped, the
style = `frameY / 36`), the 96-px gate, `CanSnapToPosition` =
`!Collision.SolidCollision` (`Player.cs:33401-33403`; the port's
`SolidCollision` copy carries the `-1/+2` sample expansion, the half-brick
cut and the `maxTilesY - 40` bottom clamp), the fully-asleep counter
(`timeSleeping >= 120`) that arms `UpdateTimeRate`'s ×5 (`Main.cs:6209-6215`
— the dial's 60 outranks it), and the wake set: the input mask, the blood
moon night, the held-item swing (the counter reset) — the damage does NOT
wake, matching vanilla.

**The poses** — `DrawSittingLegs` (`PlayerDrawLayers.cs:1536-1707`): the
standing legs cell (row 0) re-sliced into three strips (the 10-px bottom
block fanned +4 px along the facing over two 2-px knee slices, dropped
42 px, `+2·direction/+2` pre-shift, `-2` lift, `seatYOffset`), the torso
sandwich drop (`PlayerDrawSet.cs:343-347`'s `torsoOffset` over the
`LegacyPlayerRenderer.cs:183-225` ranges — torso/back arm/head drop, the
front arm does not), and the sleep rotation (`TransformDrawData`,
`:4092-4119` — every sprite's position rotates around `fullRotationOrigin`
= the hitbox centre, `PlayerSleepingHelper.cs:73-74`'s `PI/2 * (-direction)`)
plus `SetOffsetbyBed` (`:226-279`) verbatim.

**The bed spawn** — the port keeps it world-side (`World::bed_spawn`), TRWLD
**v5** (a presence byte + the cell; a corrupt/out-of-range payload degrades
to `None`), while the vanilla persists it in the *player* file's spX/spY
arrays (`Player.cs:54877-54888`) — the single-player shape, recorded. The
re-validation runs the same `CheckSpawn` at revive (`Player.cs:37571-37575`)
with the failure texts on the chat; the load-time fallback is silent (the
chat monitor does not exist at join — the C# prints via `CheckSpawn`'s
`Main.NewText`).

**The housing flood fix (a real port bug the task caught):** the port's
`CheckRoom` settled the fill on ANY active tile; the C# settles only on
SOLID tiles and the open gates (11 at the frameX ends 0/54/72/126, 389, and
386 at the open frame pairs — `WorldGen.cs:6176-6195`). The bed-spawn check
starts its flood ON the bed's cell (`StartRoomCheck(x, y - 1)`), so the old
semantics failed every bed room with "This room is too small." Furniture
(non-solid actives) now keeps the fill running, exactly like vanilla; the
town-NPC housing inherits the fix (rooms with furniture validate).

**Deviations:** the 2-players-per-tile seat/sleep stack managers
(`Main.sittingManager`/`sleepingManager`) are the multiplayer registries —
SP no-op; the Skeletron-hat bench easter egg (`UpdateSitting`'s night arm)
and the toilet's `TryToPoop` regen hook stay out (the NPC/regen sides); the
armor-specific sitting rows (legs 106/143/226/217) wait for those armor
sets; the `ForcesBreaksSleeping`/damaging-item wake arm waits for M3-ITEM's
item stats (the swing itself already resets the counter); the eclipse wake
condition waits for the hardmode state; the tile-379 solidity keeps the
generated set's base state (solid) — the vanilla's per-frame non-solid
window during `UpdateWorld_Players` (`Main.cs:17753`/`:17799`) is not
emulated, which is exactly what `CheckSpawn`'s forced-solid window needs.
`TR_POSE=sit|sleep` pins the debug scene (a dirt ledge + the furniture + the
snap beside the spawn; the load-time NPC spawn is suppressed under the pin).

## The full SetupRecipes table (M3-CRAFT-6)

`tools/gen_recipes.py` reads the decompiled `Terraria/Recipe.cs`
`SetupRecipes` and emits `crates/tr-app/src/recipes_gen.rs` - the whole
table as ONE `&[Recipe, ...]` expression spliced via
`include!` into `RECIPES`. 2764 rows in the C# registration order
(= the UI listing order):

- the main body (2475 `AddRecipe` calls) + the 35 `Add*Furniture`
  helpers spliced at their call positions + the `AddCritterStatueRecipe`
  rows (50 Stone + 5 critter at tile 283, needGraveyardBiome);
- the `for` loops (`<=` and `<` forms), the `int[,] array` colour table,
  the `ItemID.Sets.TextureCopyLoad[i]` lookups, the shared locals
  (`int type = 3234;`) and the `(int)((float)num * 2.5f)` casts are all
  expanded/evaluated;
- 284 derived reverse rows: `CreateReversePlatformRecipes` +
  `CreateReverseWallRecipes` (`Recipe.cs:16648-16694`) flip every
  single-ingredient recipe whose result is a placed platform
  (`TileID.Sets.Platforms` {19, 427, 435-439}) or a placed wall
  (`createWall > 0`, the ingredient not a wall) and insert it right
  after the source with `notDecraftable` - the port carries that exact
  flag as `Recipe.not_decraftable` (the craft direction ignores it).

Findings the hand table had wrong (the generator is the truth now):

- The C# authors the work bench, the doors, the chairs, the tables and
  the furniture with the **plain item 9** - no `RequireGroup(Wood)` on
  those rows. Rich Mahogany does NOT craft a work bench. The group
  substitution only lives where the row says so (the torch, the arrow,
  the chest's iron bars, ...).
- The iron armor rows author the plain bar 22 (not the IRON_BAR group).
- `RequireGroup` binds to a slot by membership
  (`CreateRequiredItemQuickLookups`, `:15162-15191`): a group whose
  members do not contain the authored item stays unbound and the row
  requires the exact item (the Lesion furniture rows: item 3955 is not
  in the Wood group - they require lesion wood exactly).
- `corruption`/`crimson` never gate crafting - they only feed
  `ItemID.Sets.IsCraftedCorruption/Crimson` for the decrafting UI
  (`:15252-15257`); the port ignores them.
- 1.4.5.8's Chain craft makes 15 per bar (the C# `createItem.stack`)
  and the arrow crafts at the work bench - both differ from the older
  wiki/hand-table expectations.

The generator validates every create/ingredient id against the port's
`itemdata.rs` and every group against the port group table - the run
reports 0 warnings and 1 skipped row (the `needTorchGodsFavor` gate; no
port system). `--check` mode pins the generated file to the sources.

Deviations: `needMechdusa` never fires in 1.4.5.8's setup body (the
flag exists, no row sets it); the deco/shimmer variant rows
(`AddCustomShimmerResult`) keep their craft direction and drop the
shimmer extras; the alchemy auto-set rides the same `requiredTile == 13`
rule (`AddRecipe`, `:16720-16724`).

## The use styles + body dispatch (M3-ITEM-2)

The `ItemUseStyleID.cs` constants: Swing 1, EatFood 2, Thrust 3, HoldUp 4,
Shoot 5, DrinkLong 6, DrinkOld 7, GolfPlay 8, DrinkLiquid 9, Hidden 10,
MowTheLawn 11, Guitar 12, Rapier 13, RaiseLamp 14, HoldOrb 15, PlaySound
16. The body dispatch = `PlayerFrame`'s per-style chain
(`Player.cs:36307-36444`); the held-item draw = `ItemDraw`'s per-style
branches (`:50686-51243`).

The port: `apply_item_use_frame(remaining, use_style, aim_relative)` ports
the chain verbatim — the quirks preserved: style 3's both branches are row
3 (the else is dead code in C#), style 5 reads `itemRotation *
direction` (the aim) for rows 2/3/4 with the gravDir mirrors dead, style
13 mirrors style 1, style 6's `num20 = progress * 6` clamps at 1 and the
row flips at 0.5 (progress > 1/12), style 9/8 pin row 0. The real
useStyle flows: `item::use_style(kind)` from the ItemDef, carried at the
swing start (`start_swing_extras` — the `Player.ApplyItemAnimation` tail
:4449-4451 with the 7 route sites) and per tick for the aim
(`swing_aim` = the mouse angle, updated while a style-5 swing runs).

The held-item draw: `use_style` in the composite state (was hardcoded 1)
with the per-style branches — style 4 (rotation 0 + the
beside-shoulder anchor), style 3 (the first third tucks the item
offscreen — the vanilla `itemLocation = -1000` — then the `0.8 * dir`
rotation with the X-thrust clamp ±8), style 5 (the rotation = the aim +
the centre anchor). The composite arms: `swing_stand_in` grew the real
per-style states — style 2's both arms −π/2 with the stretch Full→None
over the quarters (`Player.cs:50953-50954`), style 11's front Quarter
−π/4 + back Full −π/16 (`:51006-51007`), style 13's front at −π/2 by
thirds (`:51195`); the others stay DISABLED (style 1 never calls
`SetCompositeArm` in vanilla — `:50686-50953`).

Deviations: the gravDir −1 mirrors are dead (no flipped gravity); style
5's staff anchor (`Item.staff`) and loot special cases are out (no staff
items in the port's registry subset); the display-doll/claw sub-branches
(`:50689-50723`) are display-doll-only; styles 10/14/15/16 have no ported
items yet (Hidden is the remote-only, RaiseLamp/HoldOrb are M4's,
PlaySound is M4-TOWN-8's). The held-item draw's useStyle-2/13 item
positions ride the generic centre anchor (the C# positions them via the
composite arms' stretch-dependent offsets — the arms carry the pose; the
item offset differences are sub-10px polish for the render pass).

## Chest UI + world chests (M3-CHEST-UI)

The storage loop landed 2026-11-07 (phases 1-5, commits `3d953e4`,
`37949d5`, `ac0b326`, `7956aa9` + fix `1bfe0a4`, `c5eadd2`):
`crates/tr-app/src/chest.rs` owns the `Chest.cs` entity — the indexed
world-chest store over `App.world_chests` (`Chest.cs:522-533` find/find-empty
with the coord-collision −1), the locked-frame decode `is_locked`
(`Chest.cs:307-313`), `is_locked_or_in_use` (`Chest.cs:178-188`),
`Lock`/`Unlock` frame shifts ±36/±180 (`Chest.cs:346-495`), the open-click
path (`Player.cs:33056-33260`: the key-unlock arm with the frame-derived key
ids 327/329/1533+(col−23)/4714, the unlock-consumes-the-click rule, the
open/re-open sounds 10/12), the per-tick range close `tick_open`
(`Player.cs:31015-31124`, sound 11), `OpenChest` (`Player.cs:33505`), the
item-movement helpers (`try_placing_in_chest` ≈ `TryPlacingInChest`
`ChestUI.cs:1209-1289`, `quick_transfer_from_slot`) and the rename flow
(`ChestUI.cs:827-869`). `crates/tr-app/src/ui/chest.rs` mirrors the
`ChestUI.Draw` half (the name line, the rename pair, the 10×4 slot window at
scale 0.755); `ui/slot.rs` gained the context-3 `Inventory_Back5` arm
(`ItemSlot.cs:2215-2221`) and `clicks.rs` the context-3 click arm. The
rename rides the existing `FrontScreen::Keyboard` with a
`KeyboardTail::ChestRename` session. Suite 1038/16/570/19 ok; `TR_CHEST`
double-run SHA256 pairs `adbdaeac…` (phase 3) and `db882ea0…` (phase 5)
equal, the latter byte-identical to the pre-change HEAD.

Deviations (deliberate, in plan order):

- **Lazy store entry on first open, not at placement.** The C# placement
  path books the chest through `Chest.FindEmptyChest`/`Chest.
  AfterPlacement_Hook` (`TileObjectData.cs:3464-3465`); the port's placement
  hooks are unreachable without `editing.rs` edits, so the open path creates
  the entry on first open (`chest.rs` `try_open_chest_at_cursor`'s
  `create_chest` miss arm). Empty-entry-equivalent: a never-opened chest
  behaves exactly like an absent store entry everywhere the store is read
  (the craft scan, `CanDestroyChest`, `using_chest`), so the divergence is
  player-invisible.
- **No stale-entry sweep.** A broken chest leaves its store entry behind
  (the sweep would need another `editing.rs` hook); every consumer filters
  by live tile presence instead — `chests_in_range` structurally excludes
  dead tiles (the tile-active scan) and a stocked chest refuses its break
  through `can_kill_tile`/`CanDestroyChest` (M1.4-TO-6), so a stale entry is
  always an empty one.
- **The rigged/dead-man's chest branch is out** (`Player.cs:33253-33258` —
  the explosive trap on chest open). It is a wiring system; the port has no
  wiring trap to arm it with, and no owning task has claimed it yet.
- **The chest lid animation is out.** `Chest.frame`/`UpdateChestFrames`
  (`Chest.cs:3603`) animates the open/close lid through the tile draw; the
  port's chest tiles render statically. Render follow-up — the open state
  itself is correct.
- **The PHYS-7 coordination lines.** Exactly two simulation-file lines ride
  this task: the one-line `try_open_chest_at_cursor` interception beside
  `try_open_shop_at_cursor` (`world_sim/editing.rs:582`'s pattern — the
  tile-interaction dispatch, NOT the mining/break paths, which stay
  untouched) and the one-line `crate::chest::tick_open(app);` per-tick hook
  in `world_sim/player.rs` (the `HandleBeingInChestRange` slot,
  `Player.cs:25290`/`:31015-31124`). Both are the plan's sanctioned
  additive minimum (Decision Log #6/#9; the research doc's categorical
  "no editing.rs edits" was waived by the owner — "делай как лучше").
- **Explosion/replication `|_, _| false` unchanged.** The bomb-explosion
  and apply-edit break paths still skip the chest-store refusal (the
  M1.4-TO-6 deviation stands — the store was empty when it was written).
  With the store populated the widening is now observable; it is the PHYS-7
  follow-up, not done here.
- **The `downed_plant_boss` gate has no world flag.** `spread.rs:232`'s
  flag is a worldgen parameter, not the runtime world flag; `lock_chest`/
  `unlock_chest` take a `plant_boss: bool` parameter and the call sites
  pass `true` (Decision Log #8). Flip to the real flag when M4 lands it.
- **Chest names are hardcoded strings, not localized.** `default_chest_name`
  derives the vanilla names from the generated `itemdata::ITEMS`
  (tile + place style) with the `"Locked " + twin` rule for the lock-shifted
  frames (`Chest.Lock` shifts the style by +1; `default_chest_name(21, 72)`
  = "Locked Gold Chest"). No localization content exists on disk
  (`LegacyChestType.*` is embedded in the retail exe) — the house precedent
  for hardcoded UI labels (Decision Log #11).
- **The rename keyboard is a front screen, not an overlay.** The port opens
  `FrontScreen::Keyboard` mid-game, so the world sim pauses and the HUD
  hides while renaming; vanilla draws the keyboard as a `LegacyGameInterfaceLayer`
  ("Vanilla: Fancy UI", `Main.cs:44778`) inside the same interface pass as
  the inventory (`:44839`), with the game UI live underneath. The caret arm
  (`ChestUI.cs:165-184`), the `inter[47]` "Save" label and the panel-button
  clicks are ported and unit-tested but visually unreachable until an
  overlay rework. The submit sounds converge: the port's unified submit
  plays 10+12 where C#'s Enter path plays 12+11 (`UIVirtualKeyboard.cs`
  submit tail). The panel-cancel closes the keyboard with the edit
  discarded (`RenameChestCancel`, sound 12).
- **The entity folds are parked at M6-SAVE-2.** `Chest.name` rides the
  `App.chest_names` side map (the frozen `editing.rs:2680` literal keeps the
  `WorldChest` name widening out until then); `maxItems` is the
  `DEFAULT_MAX_ITEMS = 40` const, not a per-chest field. The prefix clause
  is resolved (flipped 2026-11-07, M3-CHEST-INT: `ChestSlot` is gone — the
  slots are `ItemSlot` and the prefix survives deposit/take; the "player
  items have none" premise died with M3-PREFIX). The name and `maxItems`
  fold into the entity when M6-SAVE-2 ports the `.wld` chest sections
  (`WorldFile.cs` `SaveChests`/`LoadChests` `:1606`/`:2723`).
- **Favorites are no longer stateless (flipped 2026-11-07, M3-CHEST-INT).**
  The `canFavoriteAt`/`retainFavoriteOnMoveAt` RULE tables are joined by the
  state: `App.favorite_slots` (slot-positional) with the C# Alt+left-click
  toggle (`ItemSlot.cs:602-625`, sound 12) and the shift-transfer refusal
  (`:631-634`) — see "Chest/bank integration (M3-CHEST-INT)" for the retain
  semantics; M7-UI-2 still owns the per-item flag, and the cursor-override
  TEXTURE display stays deferred (`Cursor_8/9` are on disk but not in the
  packed atlas — the 8/9 state machinery is fully ported, Decision Log
  #14).
- **The context-0 inventory keeps its half-take.** Right-click in context 0
  stays the half-take — the stack-split machinery (30/7/superFast) is
  context-3 only, as in vanilla. `Player.ItemSpace`'s ammo-`FillAmmo`/
  void-vault/`IsAPickup` arms (`Player.cs:39148-39347`) and `DoCoins`
  (`:39246`) are unported; the keyboard's `Main.blockKey` guard
  (`ChestUI.cs:868`), the IME composition (`UIVirtualKeyboard.cs:747`) and
  the `Main.clrInput()` reset (`IngameFancyUI.cs:128`) are out.
- **The open set vs the craft scan.** The chest interaction uses
  `TileID.Sets.BasicChest` {21, 467} (`TileID.cs:570`); the craft scan's
  `CHEST_TILES` froze [21, 341] (`craft.rs:392`) — 341 is
  `LivingDemonFire` (`TileID.cs:1311`), not a chest. Latent M3-CRAFT-2
  note, deliberately not fixed in this task (its file scope is closed).
- **The wheel gate is dead at 40 slots.** The scroll clamp is
  `0..ceil(maxItems/10) − 4` (`Main.cs:17436-17452`) — at the default 40
  slots that is `0..0`, so the wheel never scrolls. 1:1 with vanilla.

The stale notes flipped with this landing (located by content; the cited
line numbers had drifted): "Tile objects (M1.4-TO)"'s "the chest is a pure
tile object" bullet and the M1.4-TO-6 `CanKillTile` entry's "the store stays
empty until M3-CHEST-UI" both now point at the landed implementation, and
the M3-CRAFT-2 section's "the store's population is M3-CHEST-UI's"
deviation is retired the same way.
## The ammo system (M3-ITEM-3)

`Player.PickAmmo` (`Player.cs:53458-53676`): the pick order is
`AmmoSlotOrder_Default` (`:57141-57144`) — the coin slots 50-53, the ammo
slots 54-57, the main grid 0-53; the first slot whose item's `ammo` field
matches the weapon's `useAmmo` and whose stack is positive wins (the
vanilla's `ammoCyclingMode` defaults to None — no cycling). The drain
lives INSIDE `PickAmmo` (`:53667-53672`: `if (!flag &&
item.consumable) { item.stack--; TurnToAir at 0 }`) — `flag` starts as
the `dontConsume` argument (`ItemID.Sets.gunProj`, only 4 endgame items —
`ItemID.cs:1327`) and the not-consume rolls set it.

The port: `crates/tr-app/src/ammo.rs` — `pick` returns an `AmmoSource`
enum (Coin/Ammo/Grid) over `equip.coins` → `equip.ammo` →
`App.inventory`; `drain` decrements and clears; `not_consume` ports the
rolls for the shooters the registry carries (the Minishark 1/3, the
Megashark 1/2, the Gatligator 1/2, the Candy Corn Rifle 1/3, the Chain
Gun 2/3 and the SDMG 2/3 — the code's `Next(3) != 0` is verbatim even
where the wiki disagrees).

The shooter wire (world_sim/editing.rs): the shooters (`useAmmo > 0`)
skip the melee-hit and throw arms; on the press the use starts (the
swing + `item_time` + `shot_fired = false`); the shot fires at
`ItemTimeIsZero` mid-swing (`Player.cs:43544`'s
`itemAnimation > 0 && ItemTimeIsZero` gate) — `PickAmmo` → the
not-consume roll → the drain → the projectile = the ammo's `Item.shoot`,
the damage = the weapon's + the ammo's through the class multiplier
(`:53541-53546`), the speed = the weapon's + the ammo's `shootSpeed`
(`:53544`). No ammo → a dry swing (the use plays, nothing spawns).

The reach gate's player borrow was restructured: the old single `&app.player`
borrow spanned the whole `update_editing` body and blocked the shooter's
mutable `App` access — the reach check became `player_in_reach` via
`is_some_and` (the borrow ends at the statement) and the food/potion/melee
arms re-borrow through `app.player`.

Deviations: the specific-pair table (`PickAmmo_TryFindingSpecificMatches`,
`Player.cs:53448`) waits for its combo items; the ammo-cycling modes are
M3-STORE-2's; the accessory ammo saves (the magic quiver, the ammo box,
the ammo potion, the Chlorophyte 80%) wait for their accessories/buffs;
the volley specials (the Clockwork's 3-shot volley, the Fairy Queen's
first-8-ticks window) have no port volley state — every shot is the
first, so those items consume every shot (documented); the gunProj
`dontConsume` covers only 4 endgame items (`ItemID.cs:1327`).
## Minecart riding (M2-PHYS-7)

The 1.4.5.8 minecart (tile 314 riding) landed as the crate-root leaf
`crates/tr-app/src/minecart.rs` (the `grapple.rs` pattern) with the ride
integrated into the fixed step at the exact C# tick order:

- **The track state machine** — the 36-frame decode tables
  (`Minecart.cs:103-547`: the per-frame side connections, the
  `_tileHeight[frame][8]` 2-px sub-columns with the −1..−4 sentinels
  (bumper / bouncy / ramp / open end), `_trackType` (pressure = frames
  20-23, booster = 30-35) and `_boostLeft`); `track_collision`
  (`Minecart.cs:562-941`: the bottom-center magnet marched in 2-px steps,
  the OnFront/OnBack ambiguity resolved by followDown/followUp, the Y snap
  rounded to 2 decimals `:935`, X never moved here); `on_track`/
  `get_on_track`/`track_rotation` (`:1200-1265`).
- **The ride** — the `TR_CART` env pin mounts (`world_sim/player.rs`);
  the tick slots follow `Player.cs:26623-28161` — the cart stats 13/13/0.04
  with jump 15/5.15 (`Mount.cs:2747-2799`), boost ±4 applied next tick
  pre-input and clamped (`:19818-19833`), input accel gated on
  `velocity.Y == 0` (`:19849`), the coast clamp + friction 0.04 on-track /
  0.2 off with the direction-key dismount (`:26665-26681`), the jump nudge
  −0.001 (`:20672-20675`), the `cartRampTime` gravity freeze
  (`:27352-27359`), `TrackCollision` before the integration (X moves in
  `DryCollision`), `TileCollision` at `height − 10` on rails
  (`:23596-23603`), fall damage zeroed on rails (`:25311-25314`), the
  switch/pressure trip → `Wiring::hit_switch` (local; net 59 = MP).
- **Sparks + sounds** — the dust-213 emitter (`DelegateMethods.cs:223-242`
  with the `Dust.NewDust` internal rolls, 14 rolls per spark) queued on the
  Player's named `spark_rng` stream (`world.seed ^ 0x0CA4_75B1_A5E5_DD1D`),
  drained by `world_sim` into the App-owned `Sparks` pool (tick + draw in
  `app/frame.rs`, the rain/sand pattern); the rolling thresholds 33/66 % of
  `maxRunSpeed` (`Player.cs:19868-19958`) and the landing 10-spark burst
  (`Minecart.cs:809-813`); Item53/55/56 registered in `SFX_CUES`.

Deviations: no `minecartDelay`/`MinecartObjectiveCheck` in 1.4.5.8 (grep
zero hits — 1.4.4-era mechanics, not ported; off-track dismount = key
release, not a timer); carts are NOT in the mount `JumpHeight`/`JumpSpeed`
scaling cases in this decompile — fixed 15/5.15 (verified
`Mount.cs:2809-2858`; the |vx| scaling cases are bunny/slime types 0/1);
the rolling-spark gate is **direction-opposite** (`Player.cs:19868`: the
left arm's gate is `velocity.X >= 0f`, `:19926`: the right arm's is
`velocity.X <= 0f`) — sparks + Item55 fire while braking a same-sign roll
or at rest, not while accelerating forward; the spark pool caps at 500 and
drops new sparks when full with the C# pool-pressure rolls pinned off (the
6000-slot `Main.dust` never reaches pressure at ≤13 sparks/tick — the
sand.rs precedent); the sound-cue availability risk the plan flagged did
not materialise — `Item_53/55/56.xnb` exist and load through the
`SFX_CUES` rows (no silent-cue deviation); the rail sprite does not
render — the port's tile renderer treats `frame_x/frame_y` as pixel sheet
coords and skips `frame_y < 0` (`render/tiles.rs:147-159`) while tile
314's frames are the 0..35 track INDICES (`Minecart.cs:1632-1650`) —
pre-existing frame semantics; the track sprite draw is M2-MOUNT-1/3 work;
`followUp` is pinned false (the port's input has no up bit; single-rail
rides never read it — M2-MOUNT-3 adds the key); the base-cart dismount
cannot be geometrically blocked (52→42 bottom-anchored shrink,
`Collision.cs:2505-2510`; `DontDismountWhenCCed` excludes carts,
`MountID.cs:47`) — the `CanDismountWithResult` space check is ported 1:1
and unit-tested at the growth direction instead. Batch-review (3-4)
verified-equivalent notes, applied forward: the `sparks_one` side-roll
ternary is transcribed inverted vs `DelegateMethods.cs` (the two arms
swapped; the per-roll distribution is identical, so the RNG stream
position is unchanged); the spark draw anchors the 8×8 dust frame at
its top-left where the C# centres it with origin (4,4) — a 4·zoom px
offset shared with the existing sand-dust baseline; the below-view
spark death check is tighter than the C# screen-reject at zoom > 1
(the emitter has no camera at tick time — the camera is taken at the
per-frame tick); the `itemAnimation == 0` gate at `Player.cs:19868` is
omitted (the port has no item-animation field; the gate is dead for a
riding player in every reachable state).

## The dye plumbing (M3-EQUIP-4)

The vanilla chain: `Player.dye[10]` mirrors the first ten armor slots
(`Player.cs:1490`, the pair `dye[i] ↔ armor[i]` / `armor[i+10]`), the tick
refills the `c*` cache in `UpdateDyes` (`:9587-9614`, right after
`ResetEffects`, before `UpdateEquips`), and the draw applies one shader pass
per draw entry — `DrawData.shader` → `PlayerDrawHelper.SetShaderForData`
(`PlayerDrawHelper.cs:29-60`) → `GameShaders.Armor.Apply`. The port: the
registry (`dye.rs`, the 120 `BindShader` bindings of
`DyeInitializer.LoadArmorDyes` + `LoadBasicColorDyes` verbatim with their
1-based ids), the tick cache (`Player.c_head/c_body/c_legs` + the misc five;
`snapshots` filled from `App.equip` next to `equipped`), and the render
channel (`SpriteInstance.dye` → a 48-row uniform table uploaded once — the
C# bakes the parameters at `BindShader` time too). The layer→cache mapping
(`apply_dye`): helmet=`cHead`, the composite torso/shoulder/arm armor
=`cBody`, the armor legs=`cLegs`, the mount strips=`cMount`
(`Mount.currentShader`, `PlayerDrawSet.cs:432-437`).

**HLSL reconstruction (the .fx bytecode is not in the decompile —
`Main.Assets["PixelShader"]`, `Main.cs:10804`):** the port implements two
cores in `sprite.wgsl` — `colored` = dye color × texel luminance (the same
shape as the tile-paint `paint_recolor`), `gradient` = vertical
uColor→uSecondaryColor lerp (uColor at the top) then the wash. The C#
colors are constants (the item's own color is never read —
`DyeInitializer.cs:16-140`). Deviations: `ArmorColoredAndBlack` and
`ArmorColoredAndSilverTrim` ride the `colored` core (their dark/trim masks
are inside the unavailable HLSL), the `ArmorBrightness*` family likewise;
every other pass (rainbow/flame/ocean/noise/reflective/team/void/hades/
wisp/... — 48 of the 120 bindings) renders **plain** until its pass is
ported; `uSaturation` is folded on the CPU — for every binding either the
color is already fully saturated or the value is the 1.0 default (pinned
by `dye::tests::vanilla_dye_colors_are_saturated`; the brown bright
variants are the only S<1 colors and bind with the default). The plain
path is byte-identical: `dye == 0` returns the texel untouched (empty-dye
screenshot SHA256-equal to the baseline; two dyed runs SHA256-equal; the
diff vs the baseline is confined to the player's bbox).

**Cache scope:** `UpdateItemDye`'s ~27 accessory effect-slot fields
(`handOnSlot→cHandOn`, `backSlot→cBackpack/cTail/cBack`, ...) are not
ported — the port's `ItemDef` has no effect-slot fields and the accessory
visuals are not drawn yet; the `wearsRobe → cLegs = cBody` rule
(`:9596-9599`) waits for robes. The misc five mirror `:9600-9604`; only
`cMount` has a draw consumer today. Hair dyes are the separate legacy CPU
channel (`GameShaders.Hair.GetColor` → the vertex `colorHair`) — out of
this task. Loadouts (`GetEffectiveDye`'s shared-dye lookup, `:5742-5764`)
don't exist in the port — the slot itself is the source.

**Persistence:** sidecar v6 writes the ten dye kinds on their own line
after the bestiary kills (the vanilla order keeps dye straight after the
armor, `Player.cs:54818-54822`, `type, prefix, favorited` without a
stack); the port persists kinds only (the slot model carries no
prefix/favorited) and restores each as a single stack. The v4/v5 files
load unchanged. `miscEquips`/`miscDyes` stay unpersisted (they already
were).

**Slot interaction:** the click equipping (the `ItemSlot` case-2 swap
paths, `DyeSwap`'s round-robin with the per-frame counter) stays with the
inventory-interaction task, together with the armor equipping — the C#
handles armor and dye in one click path, so a dye-only branch would be an
asymmetric invention. Testability: `TR_DYE=k1,k2,...` fills the dye slots
positionally (overriding the save, like `TR_ARMOR`), `TR_DMOUNT=<kind>`
stands in for `miscDyes[3]`; the UI dye column (context 12) already
renders the slots.

## Localization runtime + retail fonts (M6-LOCALIZE-wire-fonts, 2026-11-07)

**Resource source:** the C# embeds its 84 strings files as assembly manifest
resources (`Terraria.Localization.Content.<culture>[.<Group>].json`); the port
reads the extracted resources off disk (`TR_LOCALE` → the `../decompiled`
sibling of `Content` → the working dir → the exe dir). This is the same
retail-derived, gitignored content model the `.xnb` assets already use;
without any of them the menus keep their bundled English strings, which is
the vanilla "unknown key prints the key" behaviour — the 1:1 rule only
reaches as far as the resources present.

**Font encoding:** the shipped ReLogic writes a different `DynamicSpriteFont`
byte layout than the decompiled `DynamicSpriteFontReader` documents —
default char 7-bit + `Int32` page count, reader-indexed tables with `Int32`
counts, and the character list as a UTF-8 string (details in
`crates/tr-content/src/font.rs` docs). A font is accepted only when the walk
consumes the payload exactly; `Mouse_Text` is 159 pages / 38 636 glyphs.

**Font upload:** XNA samples the font pages as `SurfaceFormat.Dxt3`; the
port uploads the shipped BC2 blocks verbatim into a texture array
(`tr-render/src/font_atlas.rs`, one 8192² layer per shelf set, ~220 MB —
the same order of magnitude as the vanilla ~226 MB, where the RGBA
decompression would cost ~900 MB). The shader selects between the two
textures per instance, so the draw order stays exactly one instanced pass.
Glyph UVs bake the half-texel inset the main-atlas path applies in the
shader (the both textures' layers are 8192 px wide).

**Missing glyphs:** the original falls back to the font's default char
(`'*'`); `UiFont::glyph` does the same and the 8×8 bitmap fallback still
drops non-ASCII — it now only serves the no-retail-content build.

**Test gate:** the M1.4-BG-18 leftover `tmp_dump::dump_flame0_tiles4_rgba`
writes into a hardcoded scratch directory and failed wherever it does not
exist (pre-existing on main); it now skips instead of failing.

## Biome passes fidelity (M5-BIOME)

The evil + jungle worldgen replaced its shape-level approximation with the
vanilla passes, each with its `WorldGen.cs` citation and a draw-order
ledger test: `CorruptionAndCrimson` (`WorldGen.cs:13866-14554` + the
`ChasmRunner`/`Crim*` walker family `:75259-76111` + `AddShadowOrb`
`:32095`) in `worldgen/evil.rs`; `JunglePass` (`JunglePass.cs:19-257`) +
`MudCavesToJungleGrass` (`WorldGen.cs:12468`) + `DirtToMud` (`:13123`) in
`worldgen/biomes.rs`; the jungle detail family — the faithful
`GrowUndergroundTree` (`:30849-31299`), `SurfaceWaterInJungle` ("Wet
Jungle", `:15810`), the plants arms (`:20122-20192` incl.
`JunglePlantsPart2`), `DirtWallsIntoMudWallsInJungleAndJungleMinMax`
(`:20813`) — in `worldgen/biome_detail.rs`; and the `LihzahrdTemple`
placement search (`:15834-15926`) in `worldgen/temple.rs`. The WGEN-1
shape-level jungle/evil band painters (their "before the caves, so the
caves cut through them" description) are retired by this; that entry
remains the record for the still-shape-level mushroom / granite-marble /
dungeon passes (M5-BIOME-2's and M5-DUNGEON's rows).

- **The executed-order moves** (`pipeline.rs` `EXECUTED_PIPELINE`, now 44
  steps): the faithful steps sit at vanilla-adjacent slots — `dirt to mud`
  between `dirt-in-stone blobs` and `ash blobs`; `evil` between
  `ores and gems` and `liquids` (the vanilla `Shinies` →
  `CorruptionAndCrimson` → `Lakes` pair, so the chasms carve through the
  finished caves instead of the caves cutting the band); `wet jungle`
  immediately after `altars` (the vanilla `Altars :15750` →
  `Wet Jungle :15810` → `Jungle Temple :15834` triple); `jungle plants`
  (the `JunglePlantsPart2` detritus sweep) between `mushroom and jungle
  plants` and `vines` (the vanilla `:20122`/`:20165` → `Vines :20193`
  order); `muds walls in jungle` between `vines` and `floating islands`
  (`:20813`). Two pre-existing executed-order deviations stay the pinned
  world contract: `underground jungle trees` runs before `lihzahrd
  temple` — vanilla runs the temple (`:15834`) first — and `jungle
  shrines` (`:15977`) likewise precede it in the port.
- **The per-pass stream model**: every faithful pass draws from its own
  `Streams::pass_rng(name) = Rng(seed ^ fnv1a64(name))` — the port's
  stand-in for vanilla's per-pass re-seed
  `UnifiedRandom(worldSeed)` (`WorldGenerator.cs:497`): insertions are
  stream-isolated and the vanilla re-seed shape is represented without
  disturbing the three shared contract streams' remaining consumers. The
  trees step moved off `drng` onto `pass_rng("Jungle Trees")` and the
  temple off `srng` onto `pass_rng("Jungle Temple")` — their later
  shared-stream consumers shifted draw positions (accepted world change;
  the pins re-pinned per phase).
- **`jungle_origin_x` is drawn in `plan biome spans`** — the port's Reset
  stand-in: the C# draws the jungle origin in `Reset`
  (`WorldGen.cs:11341-11376`), not inside `JunglePass`, so the port draws
  `width · Next(15,30) · 0.01` from a fresh `pass_rng("Jungle")` in the
  spans step and the pass consumes it without redrawing; `dungeonSide` is
  fixed `Right` (the port's dungeon sits right — the C# `Next(2)` side
  draw is not portable), and the interim `BiomeSpans.jungle_*` band is
  re-derived ±700·worldSurface around the origin so the four
  still-shape-level consumers (shrines, beehives, mushroom patches;
  underground trees until phase 5) stay aligned.
- **`roll_crimson` stands**: the evil flavour remains the deterministic
  seed-high-bit draw (`mod.rs::roll_crimson`, `(seed >> 56) & 1`) feeding
  the UI `_optionEvil` Random choice — the C# rolls `WorldGen.crimson` at
  menu/Reset time (`WorldGen.cs:11305-11315`); the port's
  seed-side draw is the recorded stand-in, consumed by
  `GenOptions.crimson` for the crimson twins.
- **The research correction (altars)**: the `Next(10,15)` altar scatter is
  crimson-branch-only (`:14186-14234`, `Place3x2(…, 26, 1)` behind the
  crimstone-ground check) — the corruption branch has none; its altars
  come from `ChasmRunner`'s `flag3` search (`:75989-76036`), whose never-re-armed
  quirk (`:75991`) accumulates the multi-altar tails and is ported
  verbatim.
- **The temple-stub scope** (`temple.rs` module docs): the verbatim/shape
  seam for M5-TEMPLE — verbatim: the placement search (the y band, the
  `num3 = 0.25` x formula with the 1M-attempt widening to the exact-0.35
  cap and the 11-stage give-up `:15900-15912`, acceptance on tile 60, the
  fallback `x = width − dungeon_x` `:15914-15923`); shape-level:
  `makeTemple`'s non-intersecting room crawl (`:33444-33548`),
  `GenerateBricks` (`:34241+`), the carve sweeps, `templePather`
  corridors, the style-11 door, the spike/platform loop, `templeCleaner`.
  The stub publishes `GenVars.temple` (the filled extent ± 10, `t_rooms`)
  and the altar anchor `l_altar_x/l_altar_y` (`:34000-34001` semantics) —
  the fields `LihzahrdTemplePart2`/`LihzahrdAltar` consume.
- **The sub-vanilla valves/gates** (deterministic skips where vanilla's
  4200×1200 minimum never degenerates): the mud-wall arm bridges the
  port's terrain-baseline walls — vanilla's pre-jungle wall writer stops
  at `worldSurface + 10` ("Dirt Wall Backgrounds", `:11858-11880`) while
  the port's `fill_terrain` pre-walls every solid row, so
  `place_mud_arm_wall` treats the baseline as vanilla's wall-0 region and
  the verbatim wall-above gate keeps the 15/64 lining to the
  stone-baseline rows; the `MudWallRunner` holes re-roll valve (200 000
  failed tries → skip — the C# re-roll is unbounded and hangs on
  sub-vanilla worlds); the evil band-retry valve (200 000 retries →
  accept-and-skip); the ≤1000-wide width gate (vanilla's
  `Next(500, maxTilesX − 500)` throws below it — the port skips the whole
  placement loop with no draws); the temple height valve
  (`maxTilesY − 600 <= 0` → straight to the C# fallback); the two clamped
  hang/crash spots — the crimson surface fill's empty `CanEvilReplace`
  failure body (`:75706-75713` — an unprotected tile spins the loop
  forever in C#) breaks instead, and the sideways Demonite tail's
  unbounded down-walk (`:75429`) clamps at the world floor.
- **The engine-state toggles**: `Main.tileSolid[379] = true` (the Bubble
  toggle the trees pass performs, `TileID.cs:1387`) is skipped — the port
  has no mutable tileSolid table and no consumer (the plan's Notes
  authorize modelling toggles only where a consumer needs one); the
  `liquidType(0)` (water) write is the flag-bit model — the port's `Tile`
  carries the liquid amount plus the lava/honey/shimmer flag bits, so
  water is `liquid = 255` with those bits cleared
  (`biome_detail.rs:426-441`).
- **The unported-by-scope list**: the TileRunner case-53 desert arm
  (`:76934` — the port carries no desert rect, the arm draws nothing),
  the 367/368 marble/granite wall placements (no in-scope caller), the
  mushroom half of the plants pass (M5-BIOME-2), and every secret-seed
  branch (`flipInfections` `:11300-11304`, the drunk per-blob re-rolls,
  the remix branch `:15852-15876`).
- **The `growth.rs::spread_grass` finding** (pre-existing, outside this
  task's file scope — SPREAD-2's row, its own follow-up): the port's
  neighbour gate is inverted vs the C# — `growth.rs:601` proceeds only
  when NO 3×3 neighbour is active+solid and wipes the whole `Tile` (the
  wall lost), while `SpreadGrass` (`WorldGen.cs:75183-75197`) proceeds
  when ANY neighbour is inactive/non-solid (lava re-arms + breaks the
  inner loop only) and writes `.type` only. This task's consumers port
  the correct slice locally (`biomes.rs::spread_jungle_grass`,
  `evil.rs::spread_evil_grass`); `growth.rs` keeps its inverted guard
  for its own callers' phase.
- **The kind-6 canopy resolution** (the M5-ORES-1 finding): the
  underground jungle "canopy" now places the faithful tile-5 tree frames
  on the 22 px trunk grid (`GrowUndergroundTree`'s frame table, off
  `pass_rng("Jungle Trees")`) — the fictional kind-6 "TreeBranches" leaf
  ball is gone from every generated world (the dirty regression test
  sweeps the 512/4200 widths on three seeds). The ore-pair tests keep
  their `wall != 0` filter for the real reason: the `OresAndShinies`
  runner itself retypes wall-0 pocket tiles to bare iron (M5-ORES-1's
  pinned behaviour, pre-existing).
- **The pins** (`world_hash.rs`): re-pinned per world-changing phase
  (3 → 4 → 5 → 6, each cause in the provenance comment); the final
  constants are `SEED42_512 = 992682600` / `DEFAULT_WORLD = 1041482865`.
  One test contract followed the world: the seed-44 sword-shrine count is
  now 1 (< the old ≥ 2 threshold) because the corrected JunglePass RNG
  order reshapes the cave terrain the shape shrines accept on —
  `the_micro_biomes_place_the_content` recalibrated to `>= 1` with the
  citation (the batch-3-4 review's authorized fix-forward).

Deviations: the executed order is the port's byte contract, not the
vanilla chain — the deltas above are the recorded moves and
`VANILLA_CHAIN` stays the declaration of record; the per-pass xorshift
streams are not `UnifiedRandom(worldSeed)` — the re-seed shape is
represented, the exact draw values are not vanilla's; the two accepted
shared-stream shifts (`drng` behind the trees step, `srng` behind the
temple) re-rolled every downstream placement on those streams; the
sub-vanilla valves exist only where vanilla's 4200×1200 minimum cannot
degenerate; the still-shape-level neighbours (the jungle shrines, the
hives, the mushroom patches, the pyramids/pots, the dungeon, the temple
interior) wait for their own rows (M5-STRUCT / M5-BIOME-2 / M5-DUNGEON /
M5-TEMPLE); the `growth.rs` inverted guard is SPREAD-2's follow-up, not
fixed here.

## Chest/bank integration (M3-CHEST-INT)

The three parallel storage merges (M3-CHEST-UI, M3-STORE-1/2, M3-STORE-3)
left the loop split in two — world chests rode `App.chest` with `ChestSlot`
tuple slots, banks rode `Player.chest` with `ItemSlot`s — and this task
stitched them back together 2026-11-07 (commits `9069134`, `3418a59`→
`c62e63b`, `56e46d3` + fixes `1acd887`/`01620fd`, `9b76a84`; final suite
`test result: ok` ×7, 1882 passed / 0 failed):

- **One open-container state.** `Player.chest/chest_x/chest_y` is the single
  index (−1 closed, −2..−5 `bank_for_chest` → `Player.banks[i]`, ≥0 the
  `App.world_chests` store index — `Player.cs:2876/:2878/:2880`); `App.chest`
  is deleted and every reader re-pointed. Bank open/close rides
  `chest::toggle_open` (the shared C# block `Player.cs:33233-33252`, sounds
  10/11/12) behind the C# prologue (`close_shop` `:33087-33096`,
  `bank_open_origin` `:33060-33079` incl. the 463/491 ±1/+=2 adjustment),
  and `tick_open` is the full `HandleBeingInChestRange` port
  (`:31015-31097`, minus the projectile trackers — remote opens are out)
  with the per-tile hitbox table (`:31099-31124`, the Empty-rect two-if
  `ClosestPointInRect` chain): banks now auto-close out of range or on a
  dead tile, sound 11. The ctx-3 world-chest surfaces gate to `chest >= 0`
  (banks keep the STORE-1 panel — see the ownership deviation below).
- **World chests hold `ItemSlot`s.** `WorldChest.slots: Vec<ItemSlot>`
  (implicit-air semantics kept, `ChestSlot` gone); the prefix survives
  deposit/take end-to-end (the stale M3-CHEST-UI note is flipped below);
  `bank::can_stack` is the shared `Item.CanStack` port
  (`Item.cs:49865-49871`); `open_chest` materializes the browsed chest's
  40-slot window (`Chest.cs:107-109`) so the transfers' free-slot walks see
  the real window; `try_placing_in_chest`/`quick_transfer_from_slot`
  retyped (`ChestUI.cs:1209-1289`).
- **The world-chest panel gained its transfer buttons.** Anchors 0-3 draw
  Loot All / Deposit All / Quick Stack / Restock (`ChestUI.cs:206-228`),
  `chest_button_hit` generalises the hit rules, and the button dispatch
  runs before the slot grid (`:281-308`) gated to `chest >= 0`. MoveCoins
  lives at the dispatch layer: the bank call site runs it before DepositAll
  (`:543-562`) and QuickStack (`:670-685`), the world chest's QuickStack
  gets none. `move_coins` gained the no-container-coins early return
  (`:1044`/`:1048-1051`) and the pInv coin-row favorite space
  (`:1052-1054`); `restock` mirrors the demand walk (pInv 57..=0 with the
  ammo rows 54-57 first, coin types 71-74 gated out of demand AND empties,
  `:899-917`; the ammo-into-empty bonus inside the `:921` gate,
  `:955-989`); `quick_stack`'s empty-slot phase is the C# per-slot
  consolidation (`:772-812`); `loot_all` gained the `CanGoIntoVoidVault`
  overflow arm (`GetItemSettings.cs:55-56`, `Player.cs:39395-39398`) via a
  `vault: Option<&mut BankStore>` parameter.
- **STORE-3 is wired per the C# binding.** Favorite = FavoriteKey LeftAlt
  (`Main.cs:67196`) + left-click via the cursorOverride-3 arm
  (`ItemSlot.cs:602-625`/`:745-755`/`:873-877`), sound 12, the
  `can_favorite_at` gate (contexts 0/1/2/32); shift-transfer refuses
  favorited slots (`:631-634`); the favorited visual is the real gold sheet —
  `Inventory_Back10` (`:2152-2156`) now packs into the content atlas and
  `ui::slot::draw_slot_favorited` swaps it in ahead of the context table (the
  C# chain puts the favorite arm before the hotbar-row `_9` arm), tinted
  `Main.inventoryBack` as the C# leaves it; the old `OurFavoriteColor` tint
  stand-in is gone. Out of scope: `Inventory_Back17` (`:2331-2335`, the gamepad
  UI-point highlight) and `_19` (context 32, the loadout panel) — neither
  surface is ported; the trash slot (448, 258) is clickable — left-click
  swaps held ↔ trash (`MouseItemSwap` `:1255-1263`, the Grab cue per
  `:1008-1011`).

Verification: `TR_CHEST` closed-panel double-run SHA256 `166e5f6a…` equal
and byte-identical to the pre-change HEAD; the open-panel labels verified
with `TR_XY=2103,354` (pair `3ed3c984…` equal); bank pixels unchanged
(`ui/inventory.rs`'s bank feed untouched).

Deviations (deliberate, in plan order):

- **The favorite binding follows the C#, not the row's wording.** The
  M3-STORE-3 row said "right-click toggles"; the C# has no right-click
  favorite on PC — `ItemSlot.cs:602-625`/`:745-755` bind favorite to
  FavoriteKey (LeftAlt, `Main.cs:67196`) + left-click via cursorOverride 3,
  right-click stays the half-take ladder. The port follows the C# and the
  row is corrected. FavoriteKey is LeftAlt only (the `KeyFavoriteModifier`
  rebind has no port surface).
- **Two-panel ownership.** The C# draws ONE panel for every container
  (`ChestUI` regardless of the chest sign); the port keeps the STORE-1 bank
  panel for banks (−2..−5) and the world-chest panel for `chest >= 0` — the
  ctx-3 surfaces gate everything (labels, hit, dispatch) to `chest >= 0`,
  so no phantom panel and no double button row. The C# single-panel
  behaviour stays the standing STORE-1 panel-fidelity deviation (recorded
  there); bank screenshots unchanged by design.
- **DepositAll's MoveCoins pre-pass is a fix, not an addition.** The port's
  `deposit_all` was the post-MoveCoins tail and the bank call site never
  ran `move_coins` — the C# runs it for every container kind
  (`ChestUI.cs:543-562`). Both call sites gained the pre-pass at the
  dispatch layer.
- **The `quick_stack` hoist.** MoveCoins lived folded inside the port's
  `quick_stack`; hoisting keeps the bank behaviour byte-equal while the
  world chest correctly skips it (the C# runs MoveCoins for banks only,
  `:670-685`), and the empty-slot phase became the C# per-slot
  consolidation (`:772-812`).
- **LootAll's void-vault overflow arm.** With `IsVoidVaultEnabled` the
  leftover fills the vault first (`GetItemSettings.cs:55-56`,
  `Player.cs:39395-39398`) before staying in the chest.
- **The prefix fix.** The `ChestSlot = (u16, u32)` tuple dropped the prefix
  byte on every deposit/take; the `ItemSlot` retype preserves it (the
  stale M3-CHEST-UI note flipped below).
- **Bank auto-close is live.** Before the unification only world chests
  range-closed; `tick_open` now covers banks (the `HandleBeingInChestRange`
  port) — out of range or a dead tile closes with sound 11.
- **The coin-row favorite gap.** `toggle_favorite`'s `idx >= INV` guard
  keeps the coin/ammo rows unfavoritable: the port keys favorites by grid
  index (`App.favorite_slots`), so `canFavoriteAt[1]/[2]` has no key space
  until the flag moves onto the item (the BUG-03 `.plr` side is already
  slot-keyed). The rows' *movement* landed in M7-UI-2b (below); the favorite
  toggle and the shift-into-chest transfer still need that move.
- **The trash swap is a plain swap.** `trash_swap`'s doc claimed a
  favorited guard; the C# swap (`MouseItemSwap` `ItemSlot.cs:1255-1263`)
  has none — favorited held items CAN be trashed there; the can't-drop rule
  lives in `DropSelectedItem` (`Player.cs:5170-5184`, the sell/drop pass's
  seam). The doc is fixed. The separate `TrashItem` cue (`:1007`) has no
  port cue; the swap plays Grab (7) per `:1008-1011`.
- **Slot-positional retain semantics.** The favorite stays on the slot
  through context-0 moves (`retainFavoriteOnMoveAt[0]`); the
  `ItemSlot.cs:1728-1734` stack-1 carry and the `:1258-1261` non-retain
  clearing are unrepresentable in the v1 slot-index model (a held favorite
  has no index; the trash slot is not favoritable) — M7-UI-2 owns the
  per-item flag.
- **The `UpdateDead` reset is not ported.** The C# death path sets
  `chest = -1` (`Player.cs:17020`); the port's `update_dead` resets only
  deadTime/respawn timer — a death-path follow-up.
- **The restock ammo bonus' empty-slot destination.** For the notAmmo
  family (Acorn 27, Sand 169/370/408/1246 — `Item.FitsAmmoSlot`,
  `ItemSlot.cs:1499-1503`) the bonus lands on the ammo row where vanilla
  uses the first empty grid slot; a notAmmo/FitsAmmoSlot seam in the
  generated item data is the future pass.
- **The `move_coins` refill is single-pass largest-first** vs the C#
  three-pass (`:1090-1196`); they coincide without carry, with a carry the
  carry tier can land in a different slot. Bounded deviation.

Out (unchanged): QuickStackAllChests (the 600 px scan), Sort/ItemSorting
(anchor 4 stays label-less), Cursor_8/9 + cursorOverride display,
persistence (M6-SAVE), net syncs, chest loot (M5), favorited sell/drop
rules, remote opens (Void Lens/Money Trough), the keyboard overlay rework,
the lid animation, `IsBlockedFromTransferIntoChest` (`ChestUI.cs:1291-1302`,
bank-only — noted at `chest.rs`).

The stale notes flipped with this landing (located by content): the
M3-CHEST-UI entry's "`ChestSlot` carries no prefix byte (the port's player
items have none)" clause and its "Favorites are stateless" bullet now point
at the landed implementation, and the M3-STORE-3 ROADMAP row's right-click
favorite wording is corrected the same way.

## Multiplayer protocol core (M6-NET-1)

The handshake and the wire frame are 1:1 ports; the deliberate deviations
(all resolved in M6-NET-2/3, where noted):

- **`15 WorldChunk` is a port-internal id** (dead in 1.4.5.8): the whole-world
  zlib blob (`save::to_bytes` + flate2) streams as `[i32 index][i32 count][u32
  len][bytes]` slices under the 65535-byte frame cap, announced by `9 Status`
  and closed by `49 WorldComplete`. The C# streams `10 TileBlock` sections
  instead (`NetMessage.cs:1895-2240` `CompressTileBlock`); M6-NET-3 replaces
  the producer — the client install hook stays.
- **`NetworkText` is literal-mode only** (mode byte 0 + the string,
  `NetworkText.cs:65-70`). Keyed/format texts and the substitution list
  (`:72-80`) are not ported; every kick/status string is a literal. The keys
  themselves (`Lang.mp[1..5]`, the `Net.*` name keys) ride as English text
  with the C# strings quoted in `netglue.rs`'s `TXT_*` constants.
- **`5 PlayerInventory` is accepted and dropped server-side.** The server
  keeps no peer-inventory state yet (the C# `Main.player[i].inventory`); the
  storage lands with M6-NET-2's peer-state sync. The client's push order is
  already the C# one (`MessageBuffer.cs:230-282`'s list, adapted to the
  port's slot layout: 59 grid + 20 armour + 10 dye + 5 misc dyes + 5 misc
  equips + 4×40 banks + trash).
- **`7 WorldInfo` fields without a port home ride as defaults** with the C#
  write order preserved: the 11 background styles, `ice/jungle BackStyle`,
  the tree/cave backups, the 13 tree-top bytes, the 14 world-flag groups,
  the ore tiers, the dungeon point, the spawn-point list and `lobbyId`
  (0 without a social API). The port-sourced fields are time/day flags,
  the moon phase/type, the world size, the spawn, the surface/rock layer,
  the dial cooldowns and the hell back style; `generator_version` carries
  `save::VERSION` (the closest marker) pending the generator-version home.
- **The clock rides the internal `205 Time` broadcast** (host → clients at
  10 Hz). The C# sends `7 WorldInfo` only at join
  (`MessageBuffer.cs:469`/`:670`) and syncs the clock through the world
  flags; the exact 1:1 clock path is pinned in M6-NET-2.
- **The name gate is the ported C# gate** (`MessageBuffer.cs:360-396`): empty,
  `> Player.nameLen` (20, `Player.cs:57105`) or already taken by another slot
  → the C# kick texts. The creative-mode mismatch checks
  (`Net.PlayerIsCreativeAndWorldIsNotCreative`) have no port flags yet and
  are deferred.
- **`4 PlayerInfo` carries the 1:1 body** (skin/voice, hair, colours, the
  three `BitsByte` groups, `NetMessage.cs:147-197`); only the name has a port
  home (`App.player_name`), the rest is vanilla defaults until the character
  milestone.
- **The in-process join replaces the relaunch.** The old flow saved the world
  blob and re-executed the binary with `TR_JOINED` (`netglue.rs`'s
  `relaunch_into_world`, deleted here); the client now swaps `App.world` and
  rebuilds the world-derived systems (`LiquidSystem`, `WorldMap`, the player
  spawn) in place — the C#'s `WorldGen.clearWorld` arm
  (`Netplay.cs:484-505`) has no port equivalent.
- **The idle sweep is the C# shape, port-tuned**: reader threads stamp a
  `last_seen` per slot and a frame-paced sweep drops peers idle for
  [`net::IDLE_TIMEOUT`] (120 s), the C#'s `TimeOutTimer > 7200` ticks
  (`Main.cs:64610-64618`).
## Extractinator (M7-EXTRACT-1)

`tr-core/src/extract.rs` ports `ExtractinatorHelper.RollExtractinatorDrop`
(`ExtractinatorHelper.cs:5-525`) as an RNG-injected chain — the branch order is
the roll order and the port keeps it a straight-line `else if` chain rather than
a table, so no draw is reordered. `Player.PlaceThing_ItemInExtractinator`
(`Player.cs:41665-41697`) is item data (the mode map `ItemID.cs:1321` and the
`ItemTrader.ChlorophyteExtractinator` first-match table `ItemTrader.cs:82-108`);
the tile side is `TileObjectData.cs:4042-4045` (`Style3x3`, `DrawYOffset = 2`)
plus `Check3x3`'s bottom-support arm (`WorldGen.cs:53114-53158`).

Deliberate deviations and notes:

- **The loot engine is not used.** Vanilla rolls the plain `Main.rand` sequence
  here, not `IItemDropRule`; forcing the table through `loot.rs`'s rule types
  would reorder draws. The port keeps a separate cited function and rolls on
  `app.item_rng` (the `Main.rand` seam, PORT_MAP's item-loot convention).
- **The interaction is swing-anchored, not click-anchored.** The C# arm runs
  inside `PlaceThing` (re-firing every `useTime` inside a held swing,
  `Player.cs:39591`/`:41669`), so the port routes it through the same
  `starting || inside_swing` cadence the tile/wall placement uses
  (`world_sim/editing.rs`); the press-edge `interact_tile` falls through for the
  new variant (the chest/shop/door seam is untouched — `interact_at` only gained
  one registry arm). `ApplyItemTime`'s shape (`Player.cs:4406-4413`) makes the
  swing and the action time the same value, ×0.33 on tile 642 (`:41673-41677` →
  3 frames), which the apply mirrors.
- **The drop spawns at the cursor pixel** (`DropItemFromExtractinator`,
  `Player.cs:42081-42089`); the smart-cursor/gamepad player-centre branch
  (`:42084-42087`) has no port mechanic yet — with smart cursor on, vanilla
  spawns at the player centre and the port still spawns at the cursor.
- **The input is consumed per action by the arm itself.** The C#'s generic
  consumable path (`Player.cs:43280`) decrements one per swing-start while the
  arm re-fires mid-swing; the port consumes one per extraction (the convention
  `place_item_and_consume` already uses). The C# also never checks weapon-ness
  in this arm, and the port's `item_gate_blocks` would refuse a `projectile`-set
  item on the place route — the arm's own decrement keeps the behaviour right.
- **Reach** is the plain `TileReachCheckSettings.Simple` region (5/3,
  `Player.cs:32025-32030`): every extractinator input has `tileBoost = 0`
  (verified in the C# `Item.SetDefaults` bodies), and the port has no
  `blockRange` (builder-potion buff 107 / gizmo-pack 407).
- **Item-data bug found (pre-existing, not fixed here):** `itemdata.rs:3465-3466`
  carries 1.4.4-era melee-thrower data for 3347/3348 (`projectile: Some(610)`,
  `consumable: false`) where the C# `Item.cs:30521-30549` has a consumable
  placeable (useTime 10, createTile 404 / createWall 311). The extractinator
  path is immune (the arm's own decrement), but the fossil items are wrong
  elsewhere — regenerating `tools/GenItemData` is the fix.
- **Out of scope:** the wired extractination (`Wiring.cs:2554-2584` —
  `CheckMech` 60, `TryFindChestForExtractinator`'s 6×6 chest scan, the
  first-non-air chest item, the wiring broad-cast drop) = M7-WIRE-2/3; the tile
  animator (`Main.cs:18659-18678`: 10 frames for 219, 6 for 642, advancing
  every 5 ticks; `drawdata.rs` already ports the `tileFrame * 54` offset but
  `render/tiles.rs` passes the t=0 state) = TILE-DRAW-3; smart cursor
  (`Player.cs:17595-17614`'s strategy 7) = M7-SMARTCURSOR-1.
## Shimmer transforms (M7-SHIMMER-1)

`tr-core/src/shimmer.rs` + `shimmer_tables.rs` port the transform data and the
resolution order (`ShimmerTransforms.cs:88-111`, `ShimmerHelper.cs`);
`tr-app/src/shimmer.rs` ports the whole chain (`WorldItem.cs:1691-1849`) with
its branch order intact, and `world_sim/items.rs` + `item.rs` port
`WorldItem.UpdateShimmer`'s ramp on the `ItemDrop` state.

Deliberate deviations and notes:

- **The crimson/corruption decraft variant is not ported.** The port's recipe
  data (`recipes_gen.rs`/`craft.rs`) carries no `crimson`/`corruption` tags, so
  `GetDecraftingRecipeIndex` (`ShimmerTransforms.cs:22-29`) falls back to the
  plain `IsCrafted` index. The 15 variant pairs (`Recipe.cs`'s `crimson`/
  `corruption` flags — extracted for the follow-up: e.g. 5092 creates the
  corruption/crimson pair at recipe indices 90/91) need the generator to grow
  the tags.
- **`customShimmerResults` are absent** from the port's recipe data
  (`Recipe.cs:176-183`) — the decraft spends the plain `requiredItem` list.
- **`Item.makeNPC` has no port field**, so the release arm
  (`WorldItem.cs:1747-1772`) never fires from real items: the override map
  (`NPCID.cs:10470`), the ≤50 attempt budget and `shimmerTransparency = 1f` are
  ported and unit-tested, but the item-side input and the NPC roster (676
  Shimmer Slime / 677 shimmered town NPC / 681 Rainbow Slime) wait for
  `GenItemData` + M4-NPC.
- **The ramp's transform tick** follows the C#'s f32 `+= 0.01f` accumulation
  and fires inside the same update (the port marks a pending flag and applies
  after the fixed step, so the spawn/flag side-effects do not run mid-borrow).
- **`unlocked_slime_rainbow_spawn` is a `World` field** standing in for the
  C#'s `Main`-global (`NPC.unlockedSlimeRainbowSpawn`) — the save format does
  not carry it yet.
- **The shimmered item's dust/light arms** (`WorldItem.cs:459-482`: dust 309 at
  1/30 with the four colours, `AddLight` `(1 - shimmerTime) * 0.8`) are render
  layer and skipped.
- **Out:** the player's §7 rules (the buff 353 "Shimmering" frozen/emerge cycle,
  `ShimmerUnstuckHelper`'s unstuck — the helper is ported, the driver is not);
  `NPC.GetShimmered`'s own transform (`NPC.cs:91548`, the map is ported as a
  table); `Collision.shimmer`-driven entity wet flags beyond the item's box; the
  shimmer tile's rendering/animation.
## Ocean passes fidelity (M5-OCEAN)

The full ocean family landed 2026-11-08 (branch `M5-OCEAN`, commits
`470672a`→`c1759e8`): the vanilla beach geometry (P1), `Ocean Sand` +
`Beaches` + `Gravitating Sand` (P2), the faithful `oceanCave` walker (P3),
`Remove Water From Sand` + `Shell Piles` + the `Water Plants` seaweed arm
(P4) and `Cactus, Palm Trees, & Coral` + the `AddTrees` beach palms (P5).
Final pins: `SEED42_512 = 3862881231`, `DEFAULT_WORLD = 2822219158`
(`world_hash.rs` provenance block has the per-phase history). Suites green
(`cargo test --workspace -- --skip migrates_version_2_kinds_to_item_ids`).

Deviations and accepted changes:

- **Reset-stand-in position.** Vanilla draws `leftBeachEnd`/`rightBeachStart`
  in `Reset` (`WorldGen.cs:11415-11440`) before every pass; the port draws
  them at the head of the terrain height-map step (the first consumer —
  `plan biome spans` is later) from `pass_rng("Beaches")`, mirroring the
  M5-BIOME spans-step stand-in. Dungeon side is fixed Right (M5-BIOME), so
  the left side always adds `beachSandDungeonExtraWidth` (40) and the right
  side takes the `beachSandJungleExtraWidth` (20) arm — the `−= 40` arm is
  the dead `dungeonSide == Left` branch. The tenth-anniversary fixed 340 is
  skipped (no seed framework).
- **Terrain hybrid.** `TerrainPass`'s absolute beach band `[0.17, 0.23]·
  maxTilesY` (`TerrainPass.cs:131-134`, `:193-201`) is not portable onto
  the compact interior curve; the shelf holds the model's sea level flat
  over the beach width and blends out over 30 columns. The compact t²·30
  edge ramp retires (the seafloor profile is `TuneOceanDepth`'s curve).
- **`tileSolid[192] = true` deferred.** `RemoveSurfaceWaterAboveSand`'s tail
  mutation (`WorldGen.cs:16240`, `TileID.LeafBlock`) has no port
  counterpart — the tile sets are generated statics (`tiledata/`); the owner
  is `tools/GenTileData`.
- **`drng` → `pass_rng`.** The ocean caves moved from the shared `drng` to
  their own stream (the M5-BIOME phase-3/6 accepted-shift class); the later
  `drng` consumers shift accordingly.
- **512-wide degeneracy.** `range(0, 1)` returns 0 without a draw (.NET
  `Next` semantics, world re-pinned once), and the ocean-zone widths on
  sub-vanilla worlds behave as vanilla never sees them (vanilla min 4200) —
  pinned, not valve'd. `leftBeachEnd > rightBeachStart` overlap on 512 is
  accepted (the living-tree anchors order their span instead of panicking).
- **Vanilla quirks ported verbatim.** The Shell Piles right branch's offset
  arms write `shellStartXLeft` (`WorldGen.cs:16381`, `:16385`); the
  `ShellPile` wobble clamps discard their result (`:10430-10431`); the
  stalactite's `Next(3)` widening writes without the `badOceanCaveTiles`
  check (`:76623-76628`); the `Cactus, Palm Trees, & Coral` middle zone's
  unused `num5`/`num6` draws still burn; `PlaceTile(81)` places crystal on
  liquid tiles (`:59460-59474`).
- **Deferred with citations.** The `Sand Patches` pass (global sand patches,
  M5-DESERT-structures); the `Water Plants` lily/cattail/bamboo arms
  (out of ocean scope — their draws are deferred with them, so the
  per-pass stream diverges from vanilla by exactly those draws); the
  `Oasis` pass and its `GenVars.numOasis` writer (the CactusPalm oasis
  block is dead in the port); the full `AddTrees` per-tile forest arms (the
  compact forest model keeps its density — the beach palm arm is faithful);
  the `PyrX/PyrY/numPyr` slots are published but unconsumed (the faithful
  Pyramids pass is M5-DESERT-structures); `UnderwaterChests` is
  M5-CHEST-gen (this task publishes `GenVars.ocean_cave_treasure` only).
- **KillTile slice.** `GrowCheckSeaweed`'s dry-out arm uses `KillTile`'s
  generation-time strip (inactive + type 0, frames 0); the item-drop /
  `SquareTileFrame` / net arms do not exist in the port.
- **Cactus arm.** `plant_cacti`'s compact desert scatter retires (the
  faithful middle zone's `PlantCactus` covers the desert's cacti); the
  pass re-declares `frame_important[80]` (the boot static the retired
  scatter used to own).
- **World re-rolls by design** at every phase — the per-phase pin history
  lives in `world_hash.rs`; the executed pipeline grew 44 → 50 steps
  (`the_executed_pipeline_is_50_named_steps` + the `test_DW_2_3`/
  `test_DW_4_3`-style vanilla-position tests for the ocean slots).

## Crystal-shard wiring (M5-SPREAD-1, part 2)

`growth::grow_crystal_shards` (the `hardUpdateWorld` `:69604-69644` arm — the
shard-capable stones 117/116/164/402/403 below the rock layer, the `Next(5)`
gate, the 4-dir pick that never grows upward, the leaf-quiet neighbour, the
radius-6 tile-129 density under 2, the `Next(18)` style with the 1/50 big
shard 18+Next(6)) was ported and unit-tested but unreferenced: the app driver
runs `spread::hard_update_world`, which carried only the chlorophyte arms.
The shard arm now runs at that function's head — the C#'s own position — so
the app's growth tick grows shards. Runtime-only, so the worldgen hashes are
untouched.

## Dungeon crawler wiring (M5-DUNGEON)

The pipeline's `Dungeon` step moved from the shape-level approximation
(`biomes::pass_dungeon` — a 5-wide tower, the 3-wide shaft and three
corridor levels) to the faithful crawler (`worldgen/dungeon.rs`
— `MakeDungeon`'s setup, the `LegacyDungeonLayoutProvider`'s room/hall
walk, the hall/room brick+carve geometry), on its own
`pass_rng("Dungeon")` hall seed; the retired approximation is kept for its
tests. The world-hash pins are re-taken (`SEED42_512 = 608156777`,
`DEFAULT_WORLD = 3295755475`) — the same accepted-world-change class as the
M5-OCEAN/M5-DESERT stream shifts.

The row's remaining scope is re-stated: the entrance-hall precalculation
(`MakeDungeon_GenerateNextEntranceHall_Precalculated` plus the
`DungeonGlobal*` feature passes — spikes/doors/platforms/chests/lights/
traps/furniture), the biome styles (`dungeonGenerationStyles` — populated
only under the `dualDungeons` secret seed, so M5-SEED-1/2's), the room
chest/book-shelf/pit-trap features (M5-CHEST-gen's) and the Lihzahrd brick
phase.

## Grass growth's enclosure gate (M5-SPREAD-1)

`growth::spread_grass` carried an inverted neighbour gate: it refused the
conversion when the 3×3 box was all-empty and proceeded when it was all-solid
— the exact inverse of the C# (`WorldGen.cs:75186-75201`'s `flag`, which
starts true, is cleared by any non-solid tile, re-armed by a lava tile with an
early break, and refuses only while it stays set). The growth tick therefore
refused the open-soil conversions the C# allows and allowed the enclosed ones
it refuses; the neon-moss worldgen sweep (`worldgen/spider_gem_moss.rs`)
inherits the same function, so both moved. Fixed 2026-11-09 with the C#
semantics; the world-hash pins are re-taken (`SEED42_512 = 1586971478`,
`DEFAULT_WORLD = 4240603701`) and `worldgen/biomes.rs`'s divergence note
updated — its own `spread_jungle_grass` already carried the C# gate.

## Chest loot rows + the family probes (M5-CHEST-gen)

The remaining `AddBuriedChest` tail rows landed 2026-11-09 with two
corrections to the earlier partial port:

- **The voice-change row was mis-ported.** The 1/12 row at `:37543-37547`
  calls `WorldGen.AddVoiceChangeItemToChest` → `Item.GetRandomVoiceItem`
  (`Item.cs:49427-49450`) — a fourteen-item table (`Next(14)` picking
  5499/5500-5509/5484/5485/5534) with `Prefix(-1)`. The partial port had
  hard-coded 2195 there (2195 is `flag13`'s guaranteed Lihzahrd Furnace,
  `:37419-37422`, a different row).
- **Three family probes read walls the port never writes.** The skyware
  probe read 216/187 (the vanilla's cloud walls) — the port's floating-island
  houses write `WALL_CLOUD` = 73 (`islands.rs:26`/`:133`), and the probe now
  reads that; reading 216 also made it fire inside the underground desert,
  whose shell wall is 216 (`desert.rs`). The dungeon probe read wall 24 —
  the port's dungeon pass writes 7/8/9 (`biomes.rs:1754`) and the probe now
  reads `tr_core::scene::is_dungeon_wall`. The pyramid probe's 187/216 pair
  is now both written (the pyramid shell 187, the desert hive 216), so it
  stands.
- **The anchor gate widened.** The load-time sweep required `frame_x == 0`,
  so every styled chest was skipped; the left chest cell now decodes as
  `frame_x / 18` (the port's tile-object frame convention,
  `tileobject.rs:2072-2082`) and the gate keeps the left half only (the
  right half is `18 + 18 * style`, also a multiple of 18 — hence the
  no-chest-to-the-left check). The style feeds the temple rows (16) and will
  feed the water (17) / jungle (10) families as their chests land.
- **Odds corrected:** the ice family's Ice Machine is 1/7 (was 1/2) and the
  Fish 1/50 is new; the skyware pottery row moved from the `wood_family`
  approximation at 1/2 to the C#'s `flag12` gate (unconditional) plus the
  Cloud stack.

## Desert passes (M5-DESERT-structures)

The `Dunes` carver and the `Full Desert` underground desert landed
2026-11-09 (branch `M5-DESERT-structures`): the dune Bézier hills
(`desert_dunes.rs`), the underground desert family — `SurfaceLine`,
`DesertGeom`, `SandMound`, `ClusterGroup`, the hive density ladder and
`AddTileVariance` (`desert.rs`) — and the four entrances with the
`PlotLine`/`Tail`/`Circle`/`Rect` shape primitives ported inline
(`desert_entrances.rs`). Both passes run as their own pipeline steps at
their vanilla slots (`Dunes` after the spans planner, `Full Desert` after
Mud Caves To Grass). Final world-hash pins: `SEED42_512 = 139436343`,
`DEFAULT_WORLD = 3348406208` — both re-pinned because the passes change
the generated worlds. Suites green.

Deviations and accepted changes:

- **Special seeds skipped.** The C# arms for `remixWorldGen`,
  `tenthAnniversaryWorld`, `SecretSeed.surfaceIsDesert`,
  `notTheBees`, `dualDungeons`, `extraLiquid` and `skyblock` have no port
  model (no `GenOptions` field), so their branches and their extra draws
  never run: `DesertDescription`'s remix height, the anniversary hive gap,
  `RowHasInvalidTiles`'s mud/jungle gate condition, the drunk pyramid
  chance, the `Full Desert` remix sandification sweep and every DesertHive
  remix wall/liquid flip. The vanilla-chain rows for `M5-SEED-1/2` carry
  them.
- **`Blotches` and `Smooth` skipped.** `Modifiers.Blotches` is a `genRand`
  noise pass over the shape and `Actions.Smooth` is the port's missing
  slope smoother; both are noted at their call sites
  (`desert_entrances.rs`), so the anthill/larva chambers and the pit keep
  their hard geometry where the vanilla jitters and smooths it.
- **The pyramid-slot anchors are recorded, not consumed.**
  `step_dunes` appends the `GenVars.PyrX/PyrY/numPyr` anchors (the C#'
  `:11570-11572`), but `structures::pass_pyramids` still rolls its own
  anchor inside the compact desert band — the rewrite that consumes the
  recorded slots (and moves them out of the band, as vanilla does) is a
  follow-up.
- **The per-pass RNG stream.** The C# drives the desert off the single
  global `genRand` (`GenBase._random` IS `genRand`); the port gives each
  pass its own `pass_rng` stream (the documented `WorldGenerator.cs:497`
  re-seed model), so the port's worlds are deterministic but not
  byte-identical to vanilla's.
- **The hive's FastRandom is exact.** `FastRandom::with_modifier_u64`
  + `next_double` were added to `tile_light/palette.rs` (the existing
  verbatim LCG port) so the hive's per-tile sand kind and its `> 0.25`
  probabilistic shell replay the C# bit-for-bit.
- **The band test widened.** `desert_band_has_sand_dunes_and_no_trees`
  now accepts the desert tile family (53/397/396) at the band surface:
  the underground desert legitimately places itself under the compact band
  (the vanilla retry loop lands opposite the dungeon, which is this band),
  and `AddTileVariance` hardens the exposed sand there.

## Entity sync (M6-NET-2)

The entity messages and the smoothing are 1:1 ports; the deviations:

- **`13 PlayerState` carries the port's render extension.** The C# derives
  each peer's animation frames and on-ground state from its local simulation
  of that player (`Main.player[i].Update` runs on every peer); the port
  simulates remote players nowhere, so the wire body appends
  `on_ground`, `leg_frame`, `body_frame`, `hp`, `max_hp` after the C# fields
  (two-client compatibility only). The control BitsBytes ride as flags with
  no port-side replay: each peer's input is local.
- **`23 NPCInfo` is the C# body minus the fields the port lacks**: `ai[]`
  (the Rust NPC keeps `anim`/`frame` instead), `difficulty`,
  `statsAreScaledForThisManyPlayers`, `releaseOwner`, `target` (u16 0) and
  the per-type `SyncAnchor` (position is sent raw). The `generation` byte
  bumps on a kind change so the client re-creates the slot — the C#'s
  `existing.generation != generation` spawn test. `hurt_cd` stays 0 on
  clients (`28 NPCStrike` is not ported), so a networked NPC shows no hit
  flash yet.
- **The cadence is the port's tick, not the C#'s per-NPC `netSpam` gate.**
  The host streams every live NPC at 10 Hz (the `netSpam` 30/5-tick per-NPC
  gate and the section culling are the M6-NET-3 follow-ups); the player
  cadence is change-driven + a 42-tick heartbeat (the C#'s 420 ticks at
  60 Hz = 7 s). The client's despawn is a 200 ms staleness prune (the host
  sends nothing when an NPC dies — the C# broadcasts life = 0 and keeps the
  slot; the port has no inactive-slot state, so the slot is dropped).
- **Item ids are host-Vec indices**, not the C#'s fixed 400 slots: a removal
  shifts the higher indices, and the next tick's `21` diff re-announces them
  (the fixed-slot scheme is M6-NET-5). `22 ItemOwner` is consumed by the
  client's grab gate but not yet emitted by the host — the port's pickup
  request validates and removes in one step, so no reservation window
  exists yet.
- **The peer slots are seeded at the world spawn** on first sight (the C#'s
  `Main.player[i]` exists at the spawn before its first `13`), so the first
  snapshot's lag accrues from there rather than from (0, 0).
## Localized item tooltips (M6-LOCALIZE-wire)

The "wire the localization core into the HUD/menus" row's item-hover slice:
`lang.rs` grew the `Lang.tip`/`Lang.inter` accessors (the `LegacyTooltip.{i}` /
`LegacyInterface.{i}` keys, `Lang.cs:497`/`:485` — `menu`/`item_name` were
already there) and `ui/tooltip.rs` replaced its four hardcoded zero rows with
`MouseText_DrawItemTooltip_GetLinesInfo`'s real rows (`Main.cs:20631-20800`),
computed from the port's own `ItemDef` values.

Deliberate deviations and notes:

- **The player-side layers are out** (the accessory/M3 layer): the C#'s
  `GetWeaponDamageMultiplier` (the port shows the raw item damage), the
  player crit bonuses (the item's own `crit`), `minionKB`, the magic-quiver
  ×1.1 knockback and the 3106 stealth arm — each defaults to its neutral value
  with a comment.
- **`notAmmo` and `ToolTipDamageMultiplier` are not ported** (no fields), so
  the damage row's gate is the plain `damage > 0` (the coin-type exception and
  the ammo gate ride along when the fields land).
- The line geometry (`tooltip_box_rect`, the `(14, 9)` inset, the shadow
  copies) is unchanged — only the row list changed.
- Without the shipped `Terraria.Localization.Content.*.json` (the gitignored
  resources the loader probes) the keys show through — vanilla's own
  unknown-key behaviour.


## Tile sections (M6-NET-3)

The `10`/`20` codecs and the section flow are 1:1 ports; the deviations:

- **`frame_important` rides the wire.** The C# reads its global
  `Main.tileFrameImportant[]` table on both sides of `10`/`20`; the port's
  table is per-world (`World::frame_important`) and the reader thread has no
  world, so the flag rides in a previously-unused bit (`10`'s `b4` bit 0,
  `20`'s third flag byte bit 4). Two-client only.
- **`wall_frame` is the one added byte** in `10`'s payload (after the wall
  low byte): the C# covers it through `CopyFrom` inside a run, the port needs
  it on the wire so a section's walls render identically.
- **The chest/sign/tile-entity tail of `CompressTileBlock` is not ported**
  (`NetMessage.cs:2208-2230`): the port's chests are world chests synced by
  the item stream and its signs/tile-entities have no sync path yet — the
  section stream covers tiles, walls, liquids, the four wire colours, paint
  and the invisible/fullbright quad.
- **The deflate stream is wire-compatible, not byte-identical**: the port
  uses flate2's raw deflate (a .NET `DeflateStream` peer decodes it), while
  byte-for-byte equality would need .NET's encoder internals.
- **The NPC/chest pushes inside `SendSection` ride the cadence tick**: the
  C# streams `23` for the section's town NPCs and the chest contents inline
  (`NetMessage.cs:2715-2753`); the port's 10 Hz tick covers both within a
  tick of the section arriving.
- **The per-NPC `netSpam` gate and the section culling of `23` are not
  ported** (all live NPCs stream each tick); the client's despawn is a 200 ms
  staleness prune rather than the C#'s life-to-0 broadcast.
- **`20 SectionRange` replicates the 1x1 edit**: the C# widens door/object
  edits to their frame group (`SendTileSquare(-1, x, y, 2)`); the port's
  multi-tile objects arrive as per-tile edits, so the widening waits for the
  door/object sync. `changeType` rides 0 (no switch family yet).
## The Golem family's display (M4-BOSS-19b)

The port draws the Golem family over the extracted sheets' real rows, but
several vanilla details are deliberately not carried:

- **The 1-px flight row.** `FindFrame` case 245's flight arm writes
  `frame.Y = 1` (`NPC.cs:72938`) — a literal pixel into a field whose unit
  is the row height (174 px for 245). The port tracks rows, so the flight
  draws row 0: a 1-px vertical offset exists in vanilla and is not
  reproduced.
- **The glow passes.** The C# draws `TextureAssets.Golem[1..3]` and
  `Extra[106/107]` over the body/head/free-head (the eye glows, the
  `Main.cs:25729-25768` arms) — those textures have no port entries; the
  base sheets only.
- **The head's `timeLeft = 300`** on the volley beams (`NPC.cs:31326`)
  rides the projectile pool's timed-spawn arm only for the Destroyer
  laser kind; the Golem beams take the pool default.
- **The cadences that consume `Main.rand`.** The head's volley trigger
  (`60 + Next(600)`, `:31273`) and the free head's barrage
  (`100 + Next(4800)`, `:31506`) fold to flat averages (300 / 1500) —
  the RNG stream discipline shared with the other folded cadences.
- **The fists' `collideX/collideY` dash exit** (`NPC.cs:19445`): the
  port's sim carries no per-NPC tile-collision flags, so the dash ends
  on the 700-px anchor distance only.
- **The fist's state 3** (the `ai[0] == 3` hover, `:19450-19503`): no
  port code sets `ai[0] = 3` for the fists — the arm is dead in the
  vanilla AI itself and is not carried.
- **`GetMyBalance`** (`NPC.cs:18485`): 1.0 for the solo unscaled player;
  the multiplayer/good-world multipliers (the +3, the `*2` outside the
  temple/jungle) stay off like the other difficulty branches.
- **The head's tether** (`aiStyle 46`) converges under a ±7 velocity
  clamp where the C# snaps the raw delta inside 100 px
  (`NPC.cs:31148-31160`) — the anchor holds within the clamp.
- **The anchor's freshness.** The C# parts read `Main.npc[golemBoss]`
  live (the body's lower slot updates first in the same tick); the port's
  pre-pass hoist feeds the head/fists the body's end-of-previous-tick
  centre/velocity — the ratio is exact (damage lands outside the NPC
  pass), the tether carries the one-tick shape.
- **`alpha` on the whole NPC draw.** `Npc::alpha` (new) tints every NPC
  through `GetAlpha`'s `(255 - alpha)/255`; only the Golem family's
  spawn fades set it today.

## The Deerclops' display (M4-BOSS-24)

- **The sheet is a grid, not a column.** `DrawNPCDirect_Deerclops` decodes
  `npc.frame` as a 5x5 cell index (`cell / 5` column, `cell % 5` row) over
  the 1090x1200 sheet's 218x240 cells drawn 220x242 (`Utils.Frame`'s +2/+2
  size offsets, `Main.cs:26338`). `Main.npcFrameCount[668] = 8` only feeds
  `SetFrameSize` and the dead `DrawNPCDirect_DeerclopsLeg`; the live draw
  never uses it. The C#'s `frame.Width = 180` write is vestigial.
- **The flip is inverted vs the generic NPCs.** The vanilla passes
  `npcSpriteEffect ^ FlipHorizontally`, cancelling the generic
  `spriteDirection == 1` flip — the sheet faces RIGHT by default, so the
  port flips on `facing < 0` (the generic path keeps `facing > 0`).
- **The anchor.** The cell is drawn at the NPC's Bottom with the origin
  (106, 238) (or (114, 238) flipped): the cell's centre sits 4 px against
  the facing direction and its bottom 4 px below the hitbox — the
  `origin.Y -= 4` sink of the vanilla's art.
- **The damage/phase overlays not carried.** The `localAI[3]` red shake
  copies and the ai[0]==7/8 purple ghosts (`Main.cs:26366-26404`) read
  `Main.GlobalTimeWrappedHourly` — wall-clock, so they cannot ride the
  port's deterministic frame cache. The port's `hurt_cd` tint covers the
  flash; the sequence cells are the visible part.
- **The step sounds.** `FindFrame` case 668 plays `SoundID.DeerclopsStep`
  at the 4/9 stride rows (`NPC.cs:66833-66836`); `find_frame` runs without
  audio access, so the cue rides the follow-up like the other NPC steps.


## NetModules (M6-NET-4)

The envelope and the ported bodies are 1:1; the deviations:

- **The chat relay broadcasts to everyone including the sender.** The C#'s
  `ChatHelper.BroadcastChatMessageAs` does the same; the port's host also
  runs the sender's line through its own `ChatMonitor::send` locally, so the
  author's slot rides only for the line's colour (player chat is one colour
  in the port until the team/chat channels land).
- **The Ping module echoes back to the sender.** The C# skips the sender
  (`ignoreClient = userId`) because its diagnostics UI counts bytes, not
  round trips; the port measures an RTT for the client (`net_rtt_ms`), which
  the C# does not need.
- **`SkyEntity` grew `net_seed`/`net_sent`** so the host can relay the exact
  spawn; the `SkyKind` → `SkyEntityType` mapping is the port's declaration
  order until the C#'s `SkyEntityType.cs` values are pinned.
- **The NetText module's keyed/format `NetworkText` modes are not resolved**:
  a non-zero mode byte keeps the raw text (the substitution list is skipped).
- **The unmapped module ids ride reserved** (0 Liquid, 4 Bestiary, 5/7
  CreativeUnlocks+Report, 6/10 CreativePowers+Permissions, 8
  TeleportPylon): their bodies land with the systems' multiplayer passes
  (the creative registry's per-player split, the pylon system, the
  npc-netId → credit-id table). Unknown ids are silently ignored exactly as
  `NetManager.Read` does.
- **The `82` envelope is not whitelisted in the accept gate**, matching the
  C#: a pre-`IN_WORLD` slot's module message is booted
  (`MessageBuffer.cs:161`'s whitelist does not contain 82).

## The humanoid walkers' display (M4-NPC-3c)

- **The zombie never animates.** `FindFrame` has no case 9 and
  `npcFrameCount[9] = 1`: the vanilla zombie's sprite is the single 30x48
  NPC_9 frame, its walk expressed purely in the body motion. The port's
  v1 ran the case-3 group's walker (the 0/1/2 band cycle) on it — rows
  1/2 did not exist in the sheet.
- **The town NPCs and goblins are not 3-frame walkers.** They belong to
  `FindFrame`'s humanoid-group arm (`NPC.cs:68547-69587`), which walks
  rows 2..npcFrameCount-1: 26 rows for the Guide/Witch Doctor, 25 for
  the Merchant/Arms Dealer/Demolitionist/Painter/Tinkerer, 23 for the
  Nurse/Clothier, 21 for the Dryad and the Goblin Archer, 16 for the
  other three goblins, 15 for the cave walkers. The v1's 3-frame machine
  showed the idle rows while walking.
- **The sit/talk/attack zones are the follow-up.** The group's
  `ai[0]` 2/3/4/5/6/7/11/18/19/23 arms (the sit, the talking, the
  downed, the shop-attack cycles over `AttackFrameCount`) ride the town
  AI states the port does not model; the default (idle/walk) arm is
  what the port draws.
- **The sheets' rows are 56 px (70 for the Tim)**, not the v1's fixed 48,
  and their widths differ per town NPC (40/46/48/50/52/56) — the draw
  takes each sheet's real frame instead of a stretched 38x48.

## The yoyo family (M4-PROJ-4)

- **The lifetime table's -1 default is verbatim, not a gap.**
  `ProjectileID.Sets.YoyosLifeTimeMultiplier` (`ProjectileID.cs:425`)
  defaults to **-1f — no timeout** — and 547/550/551/554/555/603 are
  unlisted: those yoyos return only on the channel release or the 1.3x
  range cap (`Projectile.cs:67957`), never by age. The port keeps the
  source's default (the C# `CreateFloatSet(-1f, …)` semantics), so the
  Kraken flies until released.
- **The dust rolls are skipped.** `AI_099_2_Yoyos`'s 545/553 dust
  (`:67868-67881`) and the beam's `Lighting.AddLight` (`:34416`) are
  render-only side effects the pool cannot emit — no roll is consumed
  for an unrendered effect, and the RNG stream stays aligned for the
  behaviour rolls. The yoyo sprite's string draw (the `FishingLine`
  stretch in `Main.DrawProjDirect`, `Main.cs:28039`, the
  `stringColor`/29 rainbow variants) is the M4-PROJ-4c render row.
- **The pose block's `direction` write is ported; the rest is stubbed.**
  `:67838-67847` writes `this.direction = ±1` from the ±2 px X
  comparison — the rotation (`:68096`) and the dropped twin's nudge
  (`:68057-68066`) consume it, so it lands. `heldProj`,
  `SetDummyItemTime(2)` and `player.ChangeDir` are cosmetic player-pose
  state: stubbed until M3-ITEM-2d's dummy-time gadget.
- **`meleeSpeed = 1.0` stays in the formulas.** The range/top-speed
  division (`(1 + meleeSpeed * 3f) / 4f`, `:67886-67887`) and the
  lifetime divisor (`:67750`) are neutral exactly at the default; the
  terms stay so M3-EQUIP-3 only has to supply the accessories' values
  (yoyoString: range x1.25 + 30, lifetime x1.5; magicString: the -2
  dropped twin). The `CCed` release arm (`:67920`) is stubbed false (no
  CC system ported). The Terrarian beam block's `ai[0] != 3f` gate
  (`:67776`) is verbatim from the source — only an aim exactly at
  x = 3 is excluded.
- **The rolls ride the app's item stream.** The jitter
  (`rand.Next(10, 31)`, `:67748`), the beam's three rolls (`:67786-67793`)
  and the dropped twin's `penetrate = rand.Next(3, 7)` (`:68041`) draw
  from a `DropRng` parameter threaded through `pool.update` — the
  `LootRng(&mut app.item_rng)` seam the throw's crit roll already uses,
  the port's `Main.rand` analogue (the C# `FastRandom` vs `rand01`
  difference follows the house's existing deviation record).
- **The beam scan's `CanHit` is a segment walk.** The C#
  `Collision.CanHit` is a box-vs-box LOS (`Collision.cs:381+`); the
  port walks the two centers' segment against the solid non-platform
  tiles. The Terrarian's beams can therefore target through partial
  cover where the vanilla boxes would not.
- **The counterweights are the other family.** 556-561/1079 ride
  `AI_099_1_Counterweights` (`:67380`, they fire musket balls) and stay
  with M3-EQUIP-3's accessory grants; the yoyo arm's `flag2` guard
  (`:67819-67823`) never sees them.
- **GenItemData's break-in-if fix (the 693-row regen).**
  `Item.cs:30037-30041`'s guard — `default: if (type < 3315 || type >
  3317) { break; } goto case 3262;` — was executed wrong: the
  interpreter's `break` exited only the if-body, so the statement list
  fell through into the goto and the yoyo case body overwrote every id
  the yoyo switch did not list. 693 rows carried the yoyo placeholder
  (`shoot = 541 + type - 3278`), including the Terrarian 3389
  (StaticHook 652, damage 15) and the copper tools 3505/3506/3507/3509
  (hand-patched over the placeholder in `50bfc51`). `exec_stmts` now
  returns on `ctx.brk` (the C# switch semantics) and the regen restores
  the source rows; the already-correct yoyo weapon rows are
  byte-identical and `test_gen.py` passes. The SEARCH for other
  `default: … goto case` switches found this the only one in `Item.cs`.
  The rows whose generated schema has no field (`stringColor`,
  `hasVanityEffects`, `handOffSlot`, `vanity`, `mountType`) stay
  committed as improved-but-incomplete and are the follow-up row's
  scope (M3-EQUIP-3 consumes the accessory flags).

## Ammo mastery layer (M3-ITEM-6)

Port of the M3-ITEM-3 "Out" list: `Player.PickAmmo`'s mastery half
(`Player.cs:53458-53749`) — the pair table, the projToShoot resolution chain,
the not-consume arms, the `gunProj` dontConsume four, the volley specials and
the ammo-cycling modes. Deliberate deviations and traps:

- **The four `gunProj` weapons read their data from a wire-side id-keyed table,
  not the registry.** `ItemID.Sets.gunProj` (`ItemID.cs:1327` — the VortexBeater
  3475 / Phantasm 3540 / DD2PhoenixBow 3854 / Celeb2 3930) makes `PickAmmo` run
  with `dontConsume = true` (`Player.cs:47071`): the pick still gates on having
  ammo, but the ammo's damage/speed fuse is discarded (`:47069-47081`) and the
  spawn uses the fixed gun projectile (615/630/705/714, `:48986-49018`). No drain
  anywhere in the shoot path — the only drain sits behind `!flag` with `flag`
  seeded true (verified by reading the whole of `ItemCheck_Shoot`). All four
  carry placeholder rows in the scaled `itemdata.rs` class (`use_ammo: 0`), so
  the wire keys their vanilla defs (`Item.cs:31159`/`:31728`/`:34048`/`:34741`)
  by id; the `tools/GenItemData` fix is a separate M3-ITEM-1 follow-up.
- **The 1.4.5.8 tick semantics the windows read.** On the press tick
  `ItemCheck_StartActualUse` sets `itemAnimation = itemAnimationMax` and leaves
  `itemTime` at 0; the same tick's decrements run and the shot gate
  (`itemAnimation > 0 && ItemTimeIsZero`) fires the first shot at
  `itemAnimation == max - 1` — which IS `ItemAnimationJustStarted`
  (`Player.cs:4066`). The port arms one tick on the press frame to land there
  and derives the three window booleans in integer ticks (its item timers are
  per-frame seconds — the C# compares integers). Derived counts: the Clockwork
  434 (12/4) fires at anim 11/7/3 with the first consuming; the Fairy Queen 4953
  (30/2) fires 15 shots with the four free ones at anim 29/27/25/23; the
  Clentaminator (30/5) / Flamethrower (30/6) sprays consume on the first shot
  only; the Acorn Slingshot 5629's `flag = false` is the chain's last arm and
  clears the Ammo Box / Ammo Reservation saves (order trap).
- **The `gunProj` four still spend their own `Main.rand` draw** even though the
  save is discarded — the arms run unconditionally (`:53579-53594`) — so the
  golden-stream test pins 0 draws for the 434 arm and 1 for each roll-carrying
  shooter.
- **Dead and parked arms:** the Ale Throwing Glove 3821's 69% roll never fires
  (`useAmmo = 0` → `PickAmmo` never runs) — not ported; the Snowball Cannon
  1319's remix arm is parked (no celebration-seed stand-in in the port); the
  1946/3930 projectile formulas (`:53475-53482`) are covered by the pair table
  and not ported.
- **The fuse multipliers stay 1.0-based in `resolve_projectile`:**
  `GetWeaponDamageMultiplier`'s bow/gun/specialist scalars (`:53386-53416`) need
  the quiver/archery systems (M3-EQUIP-3/M2-BUFF); the magicQuiver knockback
  ×1.1 scales the weapon's own value only (it precedes `KnockBack +=
  item.knockBack`, `:53568`) and the NanoBullet 285's ×1.25 scales the FUSED
  damage (the wire applies the weapon's part too).
- **The projectile rows land with the unported AI styles** (147 Celebration,
  75/10/181/193/31 the rest) as stand-ins per the house `projectile.rs` pattern
  — the rows stop the silent `spawn_with_data` no-op; the AI ports are M4-PROJ's
  territory. The visual-only vanilla fields (alpha/scale/ignoreWater/drawLayer/
  hide) have no port consumer and are dropped; the Snowman rockets' `scale = 0.9`
  keeps the 14x14 box. `ProjData` gained `uses_local_immunity` +
  `local_hit_cooldown` (the runtime already carried both) for the immunity pairs.
- **`ammo.rs` is ~1800 lines** — a single C# function's port (the pair table,
  the resolution chain and the roll chain share the module); the plan's
  fallback split behind the same `pub` surface is deferred to a refactor-shaped
  follow-up.
## Smart cursor (M7-SMARTCURSOR-1)

`tr-core/src/smart_cursor.rs` ports `SmartCursorHelper.SmartCursorLookup`
(`GameContent/SmartCursorHelper.cs:61-165`) as a step chain over the caller's
reachable rect — grass seeds (`:266`), moss (`:334`), pigronata (`:969`), torch
(`:2795` + `IsValidSpotForTorch` `:2867`), lawn mower (`:2895`), alchemy seeds
(`:492`), planter box (`:437`), clay pots (`:381`), staff of regrowth (`:224`),
each voting its candidates by distance-to-mouse (`:2849`'s `_targets` pick) and
committing through `Collision.InTileBounds` (`Collision.cs:3776-3783`).

- **The edge convention is deliberately stricter than vanilla.** `Main.tile` is
  exactly world-sized (`Main.cs:66984`) and the reachable rect never touches the
  border, so vanilla never indexes out of world. `World::tile` returns `None` at
  the edge and every step skips such a cell, while a *neighbour* read — the
  flag/flag2 halves, `:236-239` — counts an out-of-world cell as the inactive
  tile the C#'s default `Tile` would read. `InTileBounds` rejects any
  OOB-derived candidate, so the two skip semantics converge with the C# for
  `Step_PlanterBox`'s side extensions.
- **`Collision.EmptyTile`'s entity arms are stubbed.** The clay-pots filter
  (`:418`/`:428`) calls it with `ignoreTiles: true`
  (`Collision.cs:1359-1386`), so the tile half is skipped; the player/NPC hitbox
  arms have no tr-core store behind them and `collision_empty_tile` returns
  `true` until the app-side wiring lands.
- **`WeatherCtx` is a seam, not a deviation.** `IsAlchemyPlantHarvestable`
  (`WorldGen.cs:66148-66175`) reads `Main.raining`/`cloudAlpha`/`remixWorld` —
  session state, not `World` fields — so the caller passes them
  (`growth.rs`'s `WeatherCtx`); every other input (`dayTime`, `bloodMoon`,
  `moonPhase`, `worldSurface`, `time`) is saved world state.
- **The C#'s `&`/`|` bool operators become `&&`/`||`.** `Step_StaffOfRegrowth`'s
  `(tile.type == 0) & flag2` (`:238`) is redundant — the outer clause already
  tests `tile.type == 0`, so it reduces to `flag2` — and
  `IsAlchemyPlantHarvestable`'s `(!Main.raining | flag)` (`:66173`); both
  operands are pure, so the short-circuit forms are provably equivalent.
- **`Step_BlocksFilling` (`:2692-2793`) is unreachable in vanilla.** Its `:2709`
  gate opens with `SmartBlocksEnabled ||` — reached with the setting on,
  already returned at `:2694` with it off — so the body is dead code. Not
  ported; do not "fix" it by dropping the flag.
- **`Step_Pickaxe_MineShinies`'s priority filter (`:2315-2327`) is inert.** Both
  loops iterate the just-cleared `_targets2`, so the
  `SmartCursorPickaxePriorityOverride` comparison and the removal never see a
  candidate; when the pickaxe family lands, port the observable behaviour (a
  no-op filter), not the intended one.
- **The torch step's gates were corrected to the real sets.** `:2812` reads
  `ItemID.Sets.Torches` (24 ids, `ItemID.cs:1371`), the placement test
  `Main.tileCut` (41 ids) with `TileID.Sets.BreakableWhenPlacing`, and
  `IsValidSpotForTorch`'s anchors read `TileID.Sets.NotReallySolid` {387, 388,
  10} (`TileID.cs:592`). This changes the previously merged torch behaviour; no
  caller existed, so there is no runtime impact.
- **`edit::solid_tile` gained the `!inActive` term.** The C#'s `nactive()`
  (`WorldGen.cs:69957-69969`) and the indexed overload's explicit `!inActive()`
  (`:70111-70131`) both exclude framed-off tiles, so an actuated tile is no
  longer solid (vanilla parity). `Step_Pigronata`'s seat test
  (`smart_cursor.rs:582`) reads it; the worldgen-only callers are unaffected —
  worldgen has no actuated tiles.
- **The paint lookup is a seam, like `WeatherCtx`.** `Step_PaintBrush`/
  `Step_PaintRoller` (:727/:772) read `SmartCursorUsageInfo.paintLookup`/
  `paintCoatingLookup` (:31-32), which the C# fills with
  `TryFindingPaintInplayerInventory` (:177-191) — a scan of the player's
  inventory (`Player.FindPaintOrCoating`, `Player.cs:41927-41944`). The
  inventory is app-side, so the port's `Lookup.paint_lookup`/
  `paint_coating_lookup` stay caller-passed u32s until the app wiring lands;
  `Step_PaintScrapper` (:678) never reads them — :182 fills them only for the
  brush/roller ids (1071/1543/1072/1544).
- **The coating is a raw u32, not a port-side type.** `paintCoatingLookup`
  carries 1 = Glow (the fullbright flags — GlowPaint 4668, `Item.cs:38496`) and
  2 = Echo (the invisible flags — EchoCoating 5344, `Item.cs:42416`), decoded
  by `WorldGen.paintCoatTile`/`paintCoatWall` (`WorldGen.cs:44134-44217`). The
  port had no coating type before this pass, so the steps compare the raw
  values (`coating == 1 && !fullbright_block()`,
  `coating == 2 && !invisible_block(i, j)` and the wall pair) and leave the
  encoding to the caller.
- **The scraper's second target is LongMoss 184.** `Step_PaintScrapper`'s tile
  arm (:692-697) also accepts a LongMoss tile — the scraper's moss arm lives
  on the player (`Player.cs:41822-41824`'s
  `PlaceThing_PaintScraper_LongMoss`) — so the scan is `tile.color() > 0`,
  a `kind == 184` LongMoss tile, `fullbright_block()` or
  `invisible_block()`, OR-ed with the wall arm behind the cell
  (`wallColor() > 0`, `fullbright_wall` or `invisible_wall`).
- **`empty_tile_check`'s ignore arm was corrected to the C#'s two-part
  flag.** The C# arms are `flag = ignoreID != -1 && CommonSapling[ignoreID]`
  (`WorldGen.cs:31471-31475`) and `if (flag && !CommonSapling[cell.type])`
  (`:31501`): when the ignoreID is a sapling, EVERY `CommonSapling`
  {20, 590, 595, 615} cell passes the box. The port had compared the single
  passed kind, so a gem/sakura/willow sapling (590/595/615) wrongly blocked the
  box; the seed steps pass `ignore = Some(20)` (and `Step_GemCorns` will pass
  the profile's sapling), so the corrected arm matters for them. The worldgen
  callers (`growth.rs`' tree arm) pass the same `Some(20)` and never see a
  590/595/615 cell — worldgen spawns only the common sapling — so their
  behaviour is unchanged.
- **`Step_PumpkinSeeds` was ported but never dispatched.** The seed/herb pass
  landed the step (`:1027-1098`) without adding it to the chain's array, so no
  held pumpkin seed could reach it; it now runs at its C# place (`:137`, before
  `Step_GrassSeeds` `:138`). Nothing else in the repo called the step, so the
  omission had no runtime effect — but the ported tests for it were only
  reachable directly. Found while wiring `Step_Acorns`: the chain's arity and
  order now mirror the C# exactly (12 → 14 steps).
- **`Step_Acorns`'s ground gate is `SolidTile2`, not `SolidTile`.** The step's
  `:2054` reads `WorldGen.SolidTile2(tile3)` (`:70134-70147` — the
  solid-top-tolerant overload, widened to `pub(crate)` for the port), unlike
  the pigronata's `SolidTile`; the two C# overloads differ on exactly that, and
  the port keeps both.
- **The sapling window is `flag2 = flag || (±2/±3)`, so the immediate ±1
  neighbours block the grass arm too.** The C#'s `:2058-2060` defines `flag2`
  over `flag`, so the effective blocked window is ±3 columns (the sand arm's
  `!flag` reads the inner ±1 only). The tests pin the vote falling outside the
  window (a ±2 sapling at 27 makes the spot 25 drop and the vote take 23; an
  immediate sapling at 26 widens the block to 23..=29 and the vote takes 22).
- **The tree-seed profiles are the C#'s struct, minus the wall arm.**
  `GrowTreeSettings` (`WorldGen.cs:3825-4048`) carries two delegates per
  profile; no smart-cursor step reads the wall one (the C#'s worldgen callers
  do), so the port keeps `GroundTest` (as the `GroundTestKind` enum over the
  three ported functions, `WorldGen.cs:30304-30341`) and documents `WallTest`
  out pending the `WallID.Sets.AllowsPlantsToGrow` table. The gem trees are
  one shared profile shape (12/7/4/590) differing only in `TreeTileType`;
  the ash profile (634/20) has no item entry — `TryGetFromItemId`
  (`:3853-3884`) lists the nine gem/vanity seeds only, and the ash tree's only
  door is `TryGetFromTreeId`'s tree tile.
- **The two `Conversion` tile sets landed through the generator, not by hand.**
  The ground tests read `TileID.Sets.Conversion.Stone` (`TileID.cs:90` =
  {1, 25, 117, 203}) and `Conversion.Moss` (`:96` = the 11 moss tiles);
  `Conversion.Grass` was already generated as `SET_GRASS`. Per the house rule
  (`AGENTS.md`: never hand-edit `tiledata/`), the sets went through
  `tools/GenTileData`'s `wantedSets` with a baseline regen first — the
  baseline is a byte-empty diff and the new run adds exactly the two bitsets
  (the `mod.rs` re-exports stay hand-written, as the generator documents).
- **The generator defaults to the primary checkout's path.**
  `tools/GenTileData/Program.cs`'s root is the hardcoded
  `C:\Users\DreadLord\Desktop\Terraria-Rust` unless args[0] overrides it, so a
  task worktree must pass its own root (`dotnet run --project tools/GenTileData
  -- <worktree-root>`) or the regeneration lands in the shared primary. The
  baseline run used by this pass confirmed the primary stayed clean (the
  rewrite is content-identical), but a task should always pass the override.
- **The smart-blocks setting is a seam, like the paint lookup.**
  `Step_BlocksLines`' gate reads `Player.SmartCursorSettings.SmartBlocksEnabled`
  (`Player.cs:331`); the app owns the toggle (`settings_ui`/`menu` persist it),
  so the port's `Lookup::smart_blocks_enabled` is caller-passed. The steps'
  callers do not exist yet, so the seam's default (`false`) keeps the step
  inert until the wiring lands.
- **`collision_empty_tile` implements the tile arm it previously stubbed.**
  The clay-pot filter ran with `ignoreTiles: true`, so the early port's
  always-true body was exact there; `Step_BlocksLines`' commit filter runs
  with the default `ignoreTiles: false` (`Collision.cs:1372-1374`), where an
  occupied tile must fail — the port now checks it and leaves the player/NPC
  hitbox arms (the two loops over `Main.player`/`Main.npc`) to the app-side
  wiring, which is why `Step_BlocksLines`' vote can still accept a cell an
  entity stands in until that lands.
- **`Step_EmptyBuckets`' liquid box counts the same kind only.** The sum
  (`:644`) adds `Main.tile[k, l].liquid` only when the neighbour's
  `liquidType()` matches the centre cell's, so a lava pocket inside a water
  pool does not lift the water total (and vice versa) — the port compares
  `liquid_kind()` and sums `liquid`; an out-of-world cell reads as liquid 0
  of kind `Water`, which only ever contributes a zero.
- **The wall step's line walk keeps the C#'s sequential-if clamps.**
  `Collision.CanHitWithCheck` (`Collision.cs:493-518`) clamps its two centre
  tiles with four sequential `if`s (`if (num <= 1) num = 1; if (num >= maxTilesX) num = maxTilesX - 1;` …), which on a world shorter than 40 rows pushes a row to 0 — past the lower bound the first `if` set. The port reproduces the sequence verbatim; a Rust `clamp(1, maxTilesY - 40)` would PANIC on the port's small test worlds (min > max) and, worse, hide the vanilla's row-0 behaviour.
- **`can_hit_with_check` lives in-module until the app-side collision lands.**
  The C#'s `Collision.cs` maps to the app's `collision.rs`, which `tr-core`
  cannot reach; the smart-cursor slice needs the walk from the Rust core, so
  the overload is ported inside `smart_cursor.rs` (with the
  `DelegateMethods.NotDoorStand` check as a closure and the C#'s inline
  "solid full block" test as `solid_full`). When the app-side Collision port
  arrives, the walk moves next to its LOS siblings and this entry flips to
  done.
- **The player hitbox is a caller-passed seam, like the settings.** The wall
  step reads `providedInfo.position` + `player.width/height` as its line
  walk's origin (`Step_Walls` :1100-1102); the player entity is app-side, so
  `Lookup::player_position`/`player_width`/`player_height` arrive from the
  caller. The default fixture parks the player at the tile (30, 22) — the
  walk's own `maxTilesY - 40` clamp needs a real world height, which is why
  the wall tests run on a 100-row world (the 40-row fixture clamps every row
  to 0 and the walk fails on its first out-of-world read, exactly as the C#
  would).
- **The continuity locks are the module's first mutable state.** The C#'s
  `SmartCursorHelper._lockedDesiredDirection`/`_lockedContinuityCoords`
  (`:55`/`:57`) are static fields; the port models them as a
  `ContinuityState` the caller holds across lookups, and
  `smart_cursor_lookup` takes `&mut ContinuityState` — it runs the reset
  (`:66-67`: the use button held, the smart cursor on, the held item a
  platform) and lets `Step_Platforms` write both locks mid-chain
  (`:1383-1384`). `tile_target_desired` is the public read the C#'s
  `Player.PlaceThing_Tiles` gate (`Player.cs:39721`) consults; the
  `LockedDesiredDirection` arrow (`Main.cs:46643`) stays an app-side render
  concern.
- **The platform step works FROM the platform cells, not from empty ones.**
  `Step_Platforms`' `flag` (`:1337-1343`) is `!IsPlatform(i, j)` and the cell
  is skipped while the flag stands — so a plain empty cell is only a
  candidate through the horizontal-join exception (an active, non-solid cell
  beside a platform row, with the mouse bearing horizontal). The candidate
  written to `_targets` is one step beyond the platform cell in the desired
  direction, and the vote's second key (`num5 > num3`) picks the direction
  most along the mouse's bearing on distance ties.
- **`GetDesiredDirectionFrom` keeps the C#'s two truncations.** The bearing
  is `ToRotation`'s atan2 (`Utils.cs:1655-1658`), shifted to [0, 2π), bucketed
  into eighths with the half-bucket offset, then `(int)`-truncated, scaled
  back to an angle, converted through `ToRotationVector2` × 1.5 and
  truncated once more by `ToPoint` (`:1817-1820`) — so the diagonal buckets
  land on ±1/±1 (the ±1.41 vectors truncate inward) and the flat ones on
  ±1/0 and 0/±1. A standing lock replaces the whole computation (`:1434`).
- **The wire channels ride the port's sparse `WireState`, not `Tile`
  headers.** The C#'s `Tile.wire()`/`wire2()`/`wire3()`/`wire4()` are
  `sTileHeader` bits (`Tile.cs:623/640/657/437`); the port's packed
  12-byte `Tile` carries no header, so the set bits live in
  `wiring::WireState` (`wiring.rs`) and the steps read them through
  `WireState::get` per `WireColor` (Red = wire, Blue = wire2, Green =
  wire3, Yellow = wire4 — the mapping the C#'s `Step_MulticolorWrench`
  :1733-1746 pins). The wire steps' anchors need no tile at all: only the
  channel bits decide.
- **The wrench settings are caller-passed seams.** `Step_ColoredWrenches`
  maps the item id to a channel (509 → Red, 850 → Blue, 851 → Green, 3612
  → Yellow, `:1886-1902`) and `Step_MulticolorWrench` reads
  `WiresUI.Settings.ToolMode` (`WiresUI.cs:23`, an app-side static) — the
  port carries the raw bitfield on `Lookup::tool_mode` (`multi_tool_mode`
  holds the flag values, `WiresUI.cs:13-21`), and the rod's
  `ActuationRodLock`/`ActuationRodLockSetting` (`Player.cs:2616/:2618`)
  likewise. The C#'s `&`/`|` bool operators in the multi-wrench's
  predicates (`:1766`, `:1773` — the non-short-circuit bool AND) read as
  `&&` over pure operands; the enum math keeps its `!Cutter` mask and
  `!= 0` channel tests.
- **The wrench scans anchor on any cell carrying the channel.** The C#'
  `Step_ColoredWrenches`/`Step_MulticolorWrench` never read the anchor
  cell's tile state — only its wire bits — so an empty air cell with a wire
  still extends the line, and the candidates are the four neighbours
  lacking the channel (the C#'s border reads never happen; the port's
  out-of-world reads return false, which only ever adds a candidate the
  `InTileBounds` gate then drops).
- **The track step's bearing comes from the player's CENTRE, not the
  cursor.** `Step_MinecartTracks`' first block (`:1170-1171`) normalises
  `Main.MouseWorld - providedInfo.Center` — the player's box centre
  (`Entity.cs:50-54`'s `position + (width/2f, height/2f)`, the FLOAT
  halves) — into the eight direction flags, so the track's growth
  direction follows where the player LOOKS rather than where the cursor
  points. `SafeNormalize`'s zero/NaN default is `Vector2.UnitY` (downward).
  The second block (the pressure track, `:1268`) ignores the bearing and
  extends left/right only.
- **The track scan anchors on EVERY track cell, not the line's end.** The
  C# adds the six directional neighbours for every track-314 cell in the
  reachable rect, so a diagonal jog in the track extends from each jogged
  cell; the shared pick then keeps only the freestanding candidates
  (`:1250-1256`: no track directly above or below) and the closest to the
  mouse. The tests pin both halves: a track under a candidate drops it, and
  the line's tail (the second track cell) extends in its own direction.
- **`Step_ForceCursorToAnyMinableThing` is a seat, not a scan.** The step
  writes the mouse's own tile (`Utils.ToTileCoordinates`' arithmetic `>> 4`
  shift — the port keeps the shift, not a float floor) when that tile is
  minable and not a full solid; the `WorldGen.CanKillTile` half runs
  `edit::can_kill_tile` with the app-side chest store's closure passing
  `false` (the same convention as the wiring slice — the chest store fills
  with M3-CHEST-UI), and the three tool demotions read the new
  `Main.tileHammer` (4 ids) / `Main.tileAxe` (16 ids) tables, transcribed in
  `tileobject.rs` beside the `TILE_CUT` precedent because the generator's
  `wantedSets` does not carry them (the arrays are plain `Main.tile*`
  assignments, not `TileID.Sets`).
- **The tool powers are caller-passed on `ItemPort`.** The step reads
  `item.pick`/`item.axe`/`item.hammer` (`Item.cs:161/163/165`); the item
  definition data lives in the app's `itemdata.rs`, so the three powers ride
  `ItemPort` next to `createTile`/`createWall`, defaulting to 0 (the
  force-cursor step then refuses every tile — the gate is `pick < 1`).
- **`Step_Axe` is the one step whose winner can leave the reachable rect.**
  The C# has no candidate list and no final `InTileBounds` commit gate for
  this step — the tree/cactus/vine walk-downs carry their own bounds checks,
  and the vote compares the FINAL `(num2, k)` positions (the column's base).
  The port keeps both off: a trunk column at the rect's bottom row votes its
  base at the row below the rect. The trunk arm's right/left frame shifts are
  also asymmetric in the C# (the right arms test `num2 + 1`, the left
  `num2 - 1` — a quirk, kept verbatim), and each frame condition re-reads the
  CURRENT cell (the earlier shift may already have moved it). The axe step
  runs at two call sites, per the `SmartAxeAfterPickaxe` settings toggle
  (`Player.cs:333`, caller-passed on `Lookup`).
- **`hit_line` is ported in-module until the app-side Collision lands.** The
  C#'s `Collision.HitLine` (`Collision.cs:780-969`) is a Bresenham-ish walker
  whose `ignoreX`/`ignoreY` arms turn off the perpendicular hit checks (the
  mine step passes the walk's own direction so the side it came from never
  blocks) and whose `ignoreTargets` (the grapple hooks) never block. The
  port keeps the sequential-if clamps (a true `clamp` panics on the port's
  small test worlds, where `maxY - 40 < 1`), the C#'s catch arm as a
  `Result` error returning the START cell, and the two floating-point
  fractional strides (`num5`/`num6`) exactly. When the app-side Collision
  port arrives, the walker moves next to its LOS siblings.
- **`Step_Pickaxe_MineSolids`' beam is a start-grid → end-grid walk.** The
  step fills `_points` from the player's edge and `_endpoints` from the
  mouse's edge (the `InWorld(num8, num9, 40)` fluff gate on the endpoints —
  the port checks `40..=width-40`/`40..=height-40`, `WorldGen.cs:9688-9696`),
  then pops the pair head and walks it with `HitLine`, adding every hit
  twice (the off-endpoint hit and the full-solid re-hit — the C#'s two adds,
  the grapple targets excluded). The unkillable targets are dropped with
  `edit::can_kill_tile` (the same `|_, _| false` chest-store convention as
  the force-cursor step), and the vote runs from the player's centre — on a
  client the neighbouring miners perturb that centre by ±12 px per parity
  (`MpMinerCounts`, caller-passed; the C# scans `Main.player[0..255)`).
- **The app wiring runs the lookup once per frame in the editing path.**
  `world_sim/smart_cursor.rs`'s `repoint` is the `Player.cs:25446` call
  site: it builds the core `Lookup` from the App (the held item via
  `registry::get/place_tile/place_wall`, the player's position/hitbox, the
  rain's `is_raining`/`visual_alpha`), calls `smart_cursor_lookup` with the
  `App`-carried `ContinuityState` (the C#'s two static locks), and
  `update_editing` re-points `(tx, ty)` right after its reach gate so the
  mining/placement arms see the smart target. The gate is the vanilla
  per-player setting — `settings.smart_cursor_toggle` (`Main.cs:67178`) —
  with `TR_SMARTCURSOR=0` as the debug kill switch.
- **The reach rect is `TileReachCheckSettings.Simple`.** `reachable_rect`
  ports `GetTileRegion` (`TileReachCheckSettings.cs:48-56`) over
  `GetRanges` (:22-45): the 5/3 default ranges (`Player.cs:57113-57114`)
  capped by the 20-tile `TileReachLimit`, the near edge truncating and the
  far edge ceiling (`Math.Ceiling`). The C#'s `TB` boost (`tileBoost` +
  `blockRange`, `SmartCursorHelper.cs:94-101`) stays 0 — neither is ported
  (both are potion/gizmo state, like the extract slice's note).
- **The wiring's remaining seams are the inventory, the entities and the
  overlay.** The paint steps stand down (their `paint_lookup` is 0 — the
  `Player.FindPaintOrCoating` inventory scan is app-side and unported), the
  `Collision.EmptyTile` entity arms still answer true without the
  player/NPC store, the grapple arms are empty lists, and the gamepad /
  MP inputs are singleplayer defaults (`grav_dir` 1.0, `direction` 1, no
  nav directions). The `Cursor_1` highlight overlay lands with the UI pass.
## Projectile + strike sync (M6-NET-5)

The projectile and strike messages are 1:1 ports; the deviations:

- **This build's msg 17 is TileChange, not CreateProjectile.** The spawn+update
  payloads are unified in `27` (`NetMessage.cs:780-841`) with the
  `ProjectileKey`'s owner/slot/generation packing, and the despawn is `29`
  (`MessageBuffer.cs:1825-1850`). The roadmap's earlier "17 CreateProjectile"
  row was wrong; the port's tile edits ride `20` (ported in M6-NET-3) and no
  `17` body is implemented.
- **The host owns the server's hostile shots (owner 255)** instead of the
  single-player quirk (owner 0), so a peer's slot index can never collide
  with the host's player id.
- **The strike's damage is the sender's value, not the server's recompute.**
  The C#'s `StrikeNPC_Inner` recomputes through `CalculateDamageNPCsTake`;
  the port applies the strike as sent (the host validates the generation and
  the target) and the pending-damage/ack accounting (`msg 162`,
  `NPC.PushPendingDamage`) is not ported — the client's local prediction is
  replaced by the relayed strike.
- **The client applies the strike's hp/hurt_cd/flash, not the full inner.**
  Knockback rides the `23` snapshots (the C# applies it locally through
  `StrikeNPC_Inner`).
- **The section culling of `27` is not ported**: the host broadcasts every
  changed projectile of its own (the budget-gated diff covers the spam
  control), and the `netImportant` set (type 12/pets/aiStyle 11) rides as
  ordinary traffic — the port's section stream (M6-NET-3) already covers the
  tiles.
- **The projectile `netSpam` budget is the C#'s numbers, port-tuned**: 60
  budget, 5 cost, refill 10 per net tick (the C#'s 1/tick at 60 Hz); a
  parked projectile retries on the next tick (`netUpdate2`'s park).## Cave decorations (M5-STRUCT-5)

Ported from `decompiled/Terraria/WorldGen.cs` (the four cave-decor passes) into `crates/tr-core/src/worldgen/spider_gem_moss.rs`: `Spider Caves` (`:17356-17427`), `Gem Caves` (`:17428-17471`), `Moss` (`:17472-17700`) and `Webs And Honey` (`:19995-20063`), plus the helpers `countTiles`/`nextCount` (`:9546-9635`), `Spread.Spider` (`:3682-3789`), `Spread.Gem` (`:3594-3680`), `Spread.Gemmable` (`:3791-3798`), `Spread.Moss` (`:3526-3592`), `gemCave` (`:9719`), `randGem` (`:9737`), `randGemTile` (`:9747`), `randMoss` (`:9764`), `neonMossBiome` (`:9783`), `setMoss` (`:9878`), `PlaceTight` (`:38303`), `PlaceUncheckedStalactite` (`:38327-38641`), `PlacePot` (`:54066`) and `PlaceSmallPile` (`:46815`).

Deliberate deviations, all in the module header:

- **The wall-0 gates read the port's pre-laid walls as the C#'s 0.** The port's `fill_terrain` writes a wall under every solid row (dirt 2 / stone 1 / sandstone 187 — `terrain.rs:284-289`); the C# leaves them 0 until `Cave Walls` runs (`WorldGen.cs:17711`, after these passes). Every `wall != 0` test in `nextCount`, `Spread.Spider`, `Spread.Gem` and `Spread.Moss` is therefore evaluated against the baseline set via `BASELINE_WALLS`; only a wall the port actually painted (jungle/mud/hive/gem/moss) counts as "walled". Without this the passes abort on the first cell and place nothing.
- **The RNG is the port's per-pass xorshift stream**, not the shared `UnifiedRandom`: `Streams::pass_rng("Spider Caves")`, `("Gem Caves")`, `("Moss")`, `("Webs And Honey")`. The draw ORDER is faithful, the byte stream is not (the port's worldgen contract is already xorshift).
- **`Spread.Spider`'s chest arm uses the port's `place_chest`** — the buried chest's stocked loot is M5-CHEST-gen territory, so the C#'s `chestStyle 15` / `mainItemInChest 939` book is out of this task's scope.
- **`PlaceTile`-shaped writes use the port's single-tile idiom** (`frame_x = style * 18`, `frame_y = 0`) instead of the C#'s 2x2/3x2 `TileObjectData` frames; the port's framer recomputes object frames at the pipeline end.
- **`neonMossBiome`/`SpreadGrass` ride the port's one-shot `growth::spread_grass`** (`WorldGen.cs:75146-75270`): the C#'s bounded 3x3 recursion (`grassSpread < 1000`) is not modelled, so the sweeps call it per neighbour (arm F) / per qualifying tile (`neonMossBiome`).
- **Arm D's unbounded loop is capped.** The C#'s `num13` loop (`:17612-17623`) decrements only on a hit and relies on the vanilla cave field always offering another exposed stone cell; the port caps the attempts at `6 * maxTilesX` so a degenerate world cannot spin.
- **No seed framework**: the port carries no remix / not-the-bees / tenth-anniversary / drunk / for-the-worthy flags, so those arms take the vanilla normal path; the secret-seed `noSpiderCaves` pair (`:269-291`, `:2618`) and `DoNoSpiderCavesILiedMoreSpiderCaves` are out of scope.
- **No `GenVars.shimmerPosition`/`shimmerSafetyDistance`** in the port (shimmer is not a worldgen liquid yet) — the moss pass's shimmer-exclusion arms (`:17585`, `:17604`, `:17614`) are skipped.
- **The hive arm of `Webs And Honey` is inert**: it keys on wall 86 (`WallID.HiveUnsafe`), which the port's beehive pass does not paint (`structures.rs:404` uses its own walls). Left as a documented no-op until the beehive wall lands.
- **The stalactite frame table keeps the arms the port's world can produce** (spiders, the ice family, stone/moss, hive, the sandstone pair, granite, marble) with `CopyPaintAndCoating` folded into the port's frame write; `Main.tileSolid[379] = false` (`:17360`) rides inside the pass's own `solid_tile`.
- **`Plain` counts**: `count_tiles` is an explicit-stack rewrite of the recursive `nextCount` (the fill carries no RNG, so the traversal order is free), and `Spread.*` walks the frontier lists rather than the C#'s per-level `List`/`HashSet` pair — the visited-set semantics and the `SolidTile`/wall gate are the C#'s.

Hash re-pins recorded in `world_hash.rs` (`SEED42_512 = 3074238937`, `DEFAULT_WORLD = 4238218118`): every world on every seed changes by design, because the cave pockets now carry spider/gem/moss walls and decor.

## The pillar shards (M4-EVENT-9)

The celestial pillars spawn six mobs (`NPC.cs`'s tower region of
`SetDefaults`: 405/406/407/423/425/429). Every row was re-read from the
decompile before porting — the first pass' stats (the 407 at 1200 HP/def
50, the 423 at 44×44/3000 HP, the 425 at 22×56) were wrong, as was the
aiStyle map: **425/429 ride aiStyle 3 (the walker) and 423 rides aiStyle
26 (`AI_026_Unicorns`)** — only 405/421/467 feed aiStyle 85, so the
"shard hover" machine belongs to the Star Cell alone.


## Micro-biome passes (M5-BIOME-2)
### The honey patches (M5-BIOME-2, second slice)

`HoneyPatchBiome` (`HoneyPatchBiome.cs:9-83`) rides the hive pass: every
placed hive rolls `Next(5)` patch points on the 30-90 px ring around its
centre and each runs the full `Place` — the origin gate, the down-80 find,
the 15-disc scan (75 % mud/hive + 2 hive tiles), the 8-disc solid gate
(20+), the important-location scan and the four generation steps
(the radial-dithered 229 fill, the honey pocket with the liquid kind 2,
the mud rim, the honey cap). The port's structure map is the caller's
`protected` rect list (the same neighbour-rect idiom the cave houses
document). `CorruptionPitBiome` is never instantiated in the decompile —
its `Place` is dead code, so no pass is ported for it (audited, not
ported). Both world hashes re-pinned: a classic world's hives now carry
their pockets.



The three micro-biome passes are 1:1 ports; the deviations:

- **`tile_is_ore` is a hardcoded kind list**, not the generated
  `TileID.Sets.Ore` bitset (`TileID.cs`): the port's `tiledata` carries no
  Ore set, so the slab map's ore preservation uses the vanilla's ore ids
  literal (`MarbleBiome.cs:142`, `GraniteBiome.cs:224`).
- **No `PlaceTight` pots** (`MarbleBiome.cs:151-158`): the port has no pot
  tile yet — the pots land with M5-STRUCT's pots step.
- **No `structures.AddStructure(8)`**: the port has no `StructureMap`; the
  cave-house voters read the tiles directly, so the registration has no
  consumer yet.
- **The C# `PlaceSlab`'s `Tile.SmoothSlope` call is not ported** — the port
  has no worldgen `SmoothSlope`; the marble caves carry no slopes yet (the
  slabs' corner states flatten to the active pattern).
- **The granite's `ShouldUseLava` is off** (`GraniteBiome.cs:171-196`): the
  port's lava line is not tracked yet, so `PlaceGranite`'s lava arm rides off.
- **`PlaceDecorations` (the FastRandom stalactite pass) is not ported**
  (`GraniteBiome.cs:296-330`): the stalactites land with the spider/gem-cave
  decor family.
- **The granite step moved after the cave carve** — the vanilla order
  (`WorldGen.cs:11100`-`12600` caves → `:12757` Marble → `:12839` Granite).
  The pass's own isolated stream keeps the move RNG-neutral for the other
  steps; the world contract's per-step pins hold.
- The three passes draw from their own `pass_rng` streams; the shared
  `streams.rng` is untouched by them.- **The 406's aiStyle-95 growth** (`NPC.cs:39384-39429`): the preferred
  velocity damps to 0.95 above 4 px/tick then drifts by the flat 0.99;
  `scale = 1 + 0.3 * clamp(ai[0]/300, 0, 1)` rides the counter, and at
  300 the vanilla `Transform(405)` — the port swaps `kind` in place (the
  vanilla's `SetDefaultsFromNetId` keeps `life`; both are 300 HP so the
  numbers agree) and resets the ramp. `npcFrameCount[405/406]` = 4/2.
- **The 407's aiStyle-96 flyer** (`:39430-39472`): `TargetClosest`, the
  aim vector is the player's centre lifted 250 px, the bands are <20
  (hold the current velocity) / <40 (1.75) / <80 (3.25) / else 5, the
  `moveSpeed` 0.15 smoothing, the rotation `velocity.X * 0.1`, and the
  70-tick cycle throws the 539 at 60 damage from the ring: `UnitY`
  rotated by a random ±π/2 and scaled by (5, 3), resampled until
  `|X| >= 1.5` (the port's 64-iteration guard stands in for the vanilla's
  unbounded loop). 5 frames.
- **The 405's aiStyle-85 hover** (`:36930-37220`): five `ai[0]` phases.
  Phase 0 probes (sighted → 1; 800 px → 2; the 8-px corner probe →
  3 with the player centre as the waypoint; else the drift → 4). Phase 1
  steers at `8 + dist/100` (the 405's base) over the 50-tick smoothing
  `(velocity * 49 + desired) / 50` and drops back to 0 when the line of
  sight breaks. Phases 2/3 charge/homing at 6 and 3 over the 3-tick
  divisor; phase 4 bounces with the 0.8 restitution and re-probes every
  180 ticks. The separator loop pushes 0.05 per axis off the player
  within a width. The 421-only attachment arm and the dust arms are
  unmodelled.
- **The 423's `AI_026_Unicorns` arm** (`:62366-62924`): the machine is
  the unicorn/wolf/hellhound/tumbleweed walker with per-type speed/accel
  tails — the NebulaBeast's is the 10-px cap, the 0.2 accel and the 0.85
  turnaround damping. Its own arms sit at `:62497-62557`: the charge
  (`ai[2] == 1`) damps `velocity.X` by 0.7 each tick, fires the 575 at 50
  damage (`GetAttackDamage_ForProjectiles(50, 35)` — the classic
  constant) from 20 px behind the centre at `spriteDirection * -7` on
  the 30th tick, and releases at 60 by setting `ai[1] = -rand(320, 600)`.
  The idle arms the charge on the grounded 180-tick idle inside the
  500-px reach. The ledge-jump arm's tile probes (the −8.5/−7.5/−7/−6/−8
  ladder at `:62806-62903`) fold into the walker's single jump.
- **The 425's `AI_003_Fighters` arm** (`:58463-58621`): `localAI[3]`'s
  one-shot latch rearms `ai[3]` to −120 on the first tick (the port's
  `rifleman_rearmed` field — without the latch the idling timer retriggers
  the init forever). Grounded clears the leap state `ai[2]`; in the air
  the homing aims 300 px to the side of the player and 20 px off its
  bottom, with the 0.3/8 vertical and 0.3/7 horizontal accel/cap pairs.
  The leap arm fires when the player's centre is 100 px below with line
  of sight (`velocity.Y = -5`). The arming window advances `ai[3]` from
  0 to 30 only inside the 800-px horizontal line of sight, the 31..40
  windup wraps back to −150, and the 30th tick fires the four 577s at 75
  damage from 30 px behind the centre — only the first shot is exact.
- **The 429 rides the plain walker** with the 4-px cap (`:57681`) and
  has no AI attack; its 578 lightning is a `HitEffect`-death spawn
  (`:83161-83203`) the port leaves out with the dust/gore arms.

Deviations: the dust/gore arms (229/241/242, the 778-810 gores),
`DoDeathEvents`'s 629 shield-hit projectile (`:79290`), the 427/428
metamorphoses, the VortexHunter's shared shooter block and the tower
spawn weights/caps (`:1327-1337`) are unmodelled — the pillar spawn
tables themselves stay open.
## The Moon Lord family (M4-EVENT-10)

**The ids are 396/397/398/400/401, not 517** — `NPCID.cs:11469-11479`: the
Moon Lord is `MoonLordHead` 396, `MoonLordHand` 397, `MoonLordCore` 398,
`MoonLordFreeEye` 400 (the True Eye of Cthulhu) and `MoonLordLeechBlob`
401. 517 is the Solar Tower (M4-EVENT-8). The boss bar reads 396/397/398
(`MoonLordProgressBar.ValidIds`) and the boss-head index for 396 is 8.

- **The core (aiStyle 77, `NPC.cs:41157-41575`)** runs six `ai[0]` phases.
  The spawn-in (-1) arms `wants_moon_parts` at the 60th tick → the two
  hands at `Center.X + seat*800 - 400, Center.Y - 100` with `ai[2] =
  seat`, the head at `Center.Y - 400`, all carrying the core's index in
  `ai[3]` (the port stores them in `moon_parts[0..2]`). The cruise (0/1)
  hovers 130 px above the player at the 8-px cap over the 0.5 smoothing;
  past 2400 px the core teleports to the player lifted 150 px and the
  parts ride it in the pre-pass. The crash drama (2) rises at the 0.98
  lerp toward (0, -0.5), throws the 622 smoke every 15 ticks inside
  [90, 480) and ends at 600 with `life = 0` → `downed_moon_lord`
  (`DoDeathEvents`'s 398 row). The despawn (3) clears the parts at 60.
  A missing part opens the death ride — the port's pre-pass mirrors the
  live parts into `moon_parts` so the check reads them.
- **The hands (aiStyle 78, `:41577-42082`)** run `MoonLordAttacksArray`'s
  two rows (the 600-tick loop): attack 0 the hover at the (±350, -100)
  side slot, 1 the Phantasmal Eyes — the 56-tick eyelid cycle with the
  452 (30) every 4 ticks over the ring, 2 the flood — six 454s (40) on
  the 30-tick cadence, the 20-px travel and the release wave (the 462 at
  30 from 350 px above the hand) at the 292nd tick, 3 the retargeting
  laser — the 462 (30) at the -14/-7/0 ticks. The frame box clamps the
  hand to the core-centred wander limits.
- **The head (aiStyle 79, `:42084-42457`)** sits on the fixed 400-px slot
  and runs the head row: the deathray attack fires the 455 (75) on the
  180th in-attack tick sweeping at ±2π/540, the leech attack fires the
  456 per player within 3000 px and grows a 401 clot per leeched player
  at 120/180/240, and the laser attack throws the 462 (30).
- **The True Eye (aiStyle 81, `:42459-42880`)** runs
  `MoonLordAttacksArray2`: the 53-tick drift attacks sandwich the aimed
  laser (462, 35), the orbit (the inward 454 ring then the release) and
  the ring's 452 (35) volley every 10 ticks in the 150-tick window. A
  dying hand or head spawns one (`checkDead`'s 397/396 rows), each with
  the phase offset `188 + 400 * (1 + already-spawned)`.
- **The leech blob (aiStyle 82, `:36629-36756`)** glides 90 ticks to the
  head, then drains 1000 hp in the order head → core → hand0 → hand1
  and dies.

Deviations: the 452/454/455/456 projectile engines (the vanilla's
aiStyles 82-85 — the homing eye, the sphere's lock, the deathray's
LaserScan, the leech's buff application) are unmodelled; the pool's
`proj_data` rows carry their boxes/flags only. `MoonlordDeathDrama`'s
light/explosion/pieces event is unmodelled (the port's drama is the
timer + the 622 smoke). The core's `SpawnOnPlayer`/`MoonLordCountdown`
(`Main.cs:64987-64993`), the Celestial Sigil (item 3601 →
`StartImpendingDoom(720)`), the boss bar/head textures and the 3544/58
loot rows stay open. The `FindFrame` cases port their counters (the
396's 4×8 accumulator with the rand boost folded into a hash, the 397's
frameCounter/7, the 398's 30/6, the 400's 16/4) — the 398's gore burst
on its first frame is unmodelled.

## The Vortex hive trio (M4-EVENT-11)

The Vortex pillar spawns 426/427/428 (`NPC.cs:1327`'s `SelectRandom(429,
429, 429, 429, 427, 427, 425, 425, 426)` — the 426 caps at 3 alive, the
429 at 4). All three ride aiStyle 3.

- **The hunter (426)** is the only vortex mob in the shared shooter block
  (`NPC.cs:59069`): `flag17` (`:59072`) lets it shoot mid-air, its
  cooldown is 60 with the shot at the 59th tick (`:59163-59192`), the
  muzzle sits at `spriteDirection * 2, -12` and the speed is 7
  (`:59304-59308`), the spread is `rand(-30..31) * 0.3` on both axes
  (`:59374-59378`), the projectile is the 581 at 60 damage
  (`:59480-59484`, the classic constant of `GetAttackDamage_ForProjectiles
  (60, 45)`), and its walk runs the 4-px cap / 0.6 accel / 0.95 drag
  (`:59751-59756`). `UpdateNPC_UpdateGravity`'s 426 arm (`:90910`) runs
  gravity 0.1 with the 3-px fall clamp. The interception aim
  (`:59529-59542`) folds into the centre aim.
- **The alien (427)** flies the same dive skeleton as the rifleman with
  its own constants (`:58622-58711`): landing clears `ai[2]` and flattens
  the rotation, airborne the rotation rides `velocity.X * 0.1`; the dive
  homing accelerates 0.015 (X) and 0.15 (Y) over the ±6 caps with the
  0.98 damping and the 20-px deadband (no 300-px aim offset); the 0.05
  separation pushes off the player; the leap (`:58705-58709`) throws
  `velocity.Y = -5` when the player's centre is 100 px below. At the
  1200-tick counter (`localAI[0]++ + |vx|/2`) behind the empty 5×5 box it
  `Transform(426)`s (`:56886-56901`). Gravity 0.1 / 4-px clamp while
  `ai[2] == 1` (`:90902-90908`).
- **The headcrab (428)** rides the generic walker with no speed override
  and rolls its rotation (`velocity.Y == 0` → 0, else `+= velocity.X *
  0.08`, `:58832-58841`). At the 300-tick counter behind the empty 3×2
  box above its centre it `Transform(427)`s (`:56855-56869`).
- **`Transform(newType)`** (`NPC.cs:81084-81114`) re-runs `SetDefaults`
  with `withReposition` and rescales `life`; both hive arms pre-set
  `life = lifeMax`, so the port's in-place kind swap sets the new max,
  heals and clears the ai slots (the vanilla's own `localAI[0] = 0`
  resets the counter).

Deviations: the dust cadences (`:56871`'s 16/8/4/2/1 and `:56903`'s
32/16/6/2/1), the hunter's `GetChaseResults` interception, the 426-only
`ai[2]` re-pick by the velocity ratio and the `stealth`/`itemAnimation`
gate are unmodelled; the walker's snapshot accel folds the vanilla's
per-type 0.6 accel / 0.95 drag into its speed cap.

## The Nebula pillar's mobs (M4-EVENT-12)

`NPC.cs:1301`'s `ZoneTowerNebula` picks `SelectRandom(424, 424, 424,
423, 423, 423, 421, 421, 421, 420, 420)` with the alive caps 424 ≤ 3,
423 ≤ 3, 420 ≤ 3 (421 uncapped).

- **The floater (420, aiStyle 97, `NPC.cs:39473-39615`)** is the teleport
  mover. Its first 180 ticks (`localAI[2]`) throw the 574 (0 damage) at
  the 60-tick cadence from the ring `UnitY.RotatedByRandom(π/2) *
  (4, 2.5)` resampled until `|X| >= 1.5`. The aim point is `Center +
  (direction * 20, 6)`; inside 400 px with line of sight it damps down
  (`* 0.95`), outside it cruises at the 7-px cap. The 480-tick counter
  (`ai[0]`) arms the 20-tick fade (`ai[1] = 20`), and the hop lands the
  floater at `ai[2..3]`'s tile with zeroed velocity; the vanilla's
  `AI_AttemptToFindTeleportSpot(aim, 20, 12, 1, solidTileCheckCentered:
  true, teleportInAir: true)` folds into the port's first non-solid tile
  near the aim. Its roll is `|velocity.X| * direction * 0.1` and its
  sprite direction flips.
- **The suckler (421, aiStyle 85)** is the hoverer family's third member
  — the same five phases as the star cell but with its own constants
  (`num1187 = 5.5`, phase 2's 3 px, phase 3's 2 px, phase 4's 2 px) and
  the 421-only arms: the reach vector reads `player.Top - Center`, and
  under 40 px with no same-type 421 already attached to the same target
  it snaps `Center = player.Top`, zeroes the velocity, sets `ai[0] = 5`
  and (phase 5) rides `player.Top + direction * 4`, applies
  `AddBuff(163, 59)` (the Obstructed buff) every tick and hides itself.
- **The predictor (424, aiStyle 3)** is the nebula shooter of the shared
  block (`:59069`): its cycle is 180 ticks with the shot at the 179th,
  the projectile the 573 at 60 damage (the classic constant of
  `GetAttackDamage_ForProjectiles(60, 45)`), and the fire throws four
  shots in the `(-3 + 2*i) * 0.15` fan from `Center - (spriteDirection *
  4, -6)` with the `-rand(0..3) * 0.2 - 0.1` rise. Its walk has no speed
  override (the generic `num180 = 1`, `num181 = 0.07`, `num182 = 0.8`).

Deviations: the 242-dust bursts, the on-hit early teleport
(`:39497-39505`'s `localAI[1]` 1-in-6 arm), the owned-574 shave
(`:39577-39590`) and the `hide` flag while attached are unmodelled; the
attached suckler stays visible and the port's Player takes the obstruct
through the `wants_obstruct` intent. The tower spawn tables themselves
(`:1301-1318`) stay open with the M4-EVENT-7 shield work.

## Secret seeds (M5-SEED-1)

The framework layer, the runtime feeds and the worldgen arms that landed on
`M5-SEED-1-classic-seeds` (merge `b460db76`-class follow-ups noted at the end).

### The registry

`tr-core/src/seeds.rs` ports the 1.4.5 option table, not the 1.4.4 flag
switches: the classic secret seeds are `AWorldGenerationOption`s
(`WorldBuilding/WorldGenerationOptions.cs:30-41`), each with a
`SpecialSeedNames` list and `SpecialSeedValues` aliases; a seed *text*
resolves through `GetOptionFromSeedText` (`:66-83`) — `WorldFileData.
TranslateSeed` (`:386-397`: numeric → `Math.Abs(int)`, else `Crc32`) for the
value match, `Regex.Replace(text, "[^a-z0-9]+", "")` for the name match, in
registration order. The port mirrors that with `OPTION_TABLE` +
`option_for_seed_text` + `SeedFlags::from_seed_text`, the flags carried as
the `serializedSeedsSum` bitmask (`WorldFileData.cs:157-197`: drunk 1,
bees 2, FTW 4, anniversary 8, DST 16, remix 32, no-traps 64, zenith 128,
skyblock 256).

Two structural notes:

- `getfixedboi` is the *Everything/Zenith* option (`WorldSeedOption_Everything.
  cs:15-58`), which force-enables its seven dependencies (remix, drunk,
  bees, no-traps, the Constant, anniversary, FTW — skyblock is NOT one).
  `SeedFlags::from_seed_text` reproduces the cascade, so a
  "get fixed boi" world's flags are the full mix.
- The 38 modular 1.4.5 micro seeds use a different registry entirely —
  `WorldGen.SecretSeed` with BCrypt codes (`WorldGen.cs:3022-3058`,
  `Utilities/Secrets.cs:7-27`). They stay with M5-SEED-2; the port's
  `SeedFlags` has no room for them and the worldgen reads them as off.

The UI half (`OnFinishedSettingSeed`'s option ring, the Advanced seed tab,
the plate icons) stays a documented deviation — the resolution itself is
pure, so `create_world_in` now feeds the flags into `GenOptions` and the
created (and saved) world carries them.

### Persistence

- TRWLD gains v6: the nine booleans as the port's own bitmask u16
  (`WorldFile.SaveWorldFlags`'s order, `WorldFile.cs:1238-1250`). A v5 file
  loads as the classic world it was.
- `.wld`: `load_header_flags` used to `skip(1)` nine times; it now reads
  them into `World.seeds` at their introduction versions (222/227/238/239/
  241/249/266/267/302 — the same gates `WorldFile.LoadWorldFlags`
  `:1981-2041` uses), plus the pre-267 `zenithWorld = remix && drunk`
  inference (`:2020`). The port still stores the seed as text only and
  re-derives the int on load, exactly like the C#.

### The worldgen arms

Every arm is flag-gated; the classic pipeline is byte-identical (both
pinned world hashes pass unchanged, and no world_hash re-pin was needed —
the M5-BIOME-2-style ledger note does not apply). Three new steps join the
executed chain (56 → 59): `wavy caves` (the Constant's own pass, vanilla
`:12271`), `constant lava layer` (the Dungeon-tail `LavaLayerCaverer`
call, `:14867`) and `seed finish` (the `GenerateWorld` tail, `:22431-22444`);
all three draw nothing for a classic world.

| Seed | Arm | C# anchor |
|---|---|---|
| drunk | `TileRunner` entry pair — `strength`/`steps` ×(1 ± 0.8) with two `Next(-80,81)` draws | `:76772-76788` |
| drunk | the 1-in-30 step teleport (both axes), the ladder `Next(3)` gate, the per-step X-velocity nudge | `:76823-76827`, `:77003`, `:77088-77091` |
| drunk | the ash-candidate skip and the ash-column runner floor | `:76866-76868` |
| drunk | both evil biomes at half the count, both passes forced, the half-restricted anchors (`crimsonLeft`), `num19 = 100` | `:13938-14304` |
| drunk | the dungeon start row pinned to `worldSurface + 70` | `:14804`/`:14854` |
| drunk | the living-trees count `+2×`, the side-patch count `+Next(2,5)` | `:15504`/`:15628` |
| drunk | the ocean-cave chance gate OR'd open | `:15178` |
| drunk | `FinishDrunkGen` — the `Next(13,25)` dungeon palette, the per-brick bands, wall 25/87, the jungle→mud smoothing, the temple band at 16 | `:26226-26315` |
| drunk+FTW | `flipInfections` — ebonstone 25/3 ↔ crimstone 203/83 ids through the evil pass | `:11298-11304` |
| FTW | `TileRunner` `strength` ×(1 ± 0.015) + `steps += Next(3)` for every non-ash runner | `:76772-76788` |
| FTW | spider caves ×3, neon-moss radius/length ×1.5, the moss 0.42-0.48 band | `:17378`, `:9802`, `:17503` |
| FTW | `ShroomPatch` scale ×1.5 (the `World.seeds` stand-in for `!remix`) | `:79866-79870` |
| FTW | `FinishGetGoodWorld` — the palette re-roll, tile 226/137 paint 17, wall 87 → 25, the 1-in-15 ash-lava pockets, the surface-grass conversion to 199/23 with the plant kills | `:25799-25983` |
| anniversary | `FinishTenthAnniversaryWorld` — `PaintTheDungeon/LivingTrees/Temple/Clouds/Sand/Pyramids` and the 1-in-4 boulder 138→665 | `:24237-24282` |
| anniversary | the jungle-origin band widened to `Next(25,35)` | `:11360-11370` |
| remix | the jungle-origin band widened to `Next(20,35)` | `:11247` |
| remix | the spawn point moved to the bottom centre (the first air scanning up from `maxTilesY - 10`) | `:19574-19584` |
| constant | iron/gold pinned (no lead/platinum) unless drunk | `:11280`/`:11292` |
| constant | `WavyCaves` — the DST-only pass, 100-try band rejection then `WavyCaverer` | `:12271-12311` |
| constant | `LavaLayerCaverer` — the Dungeon-tail two-row lava caver | `:14867`, `:73950-74060` |
| bees+constant | the runner's honey wall-spread variant | `:76806` |

`GenVars.crimsonLeft` is drawn in the pipeline's Reset stand-in from its own
`pass_rng("Reset")` stream (vanilla's `genRand.Next(2)` at `:11216`); the
evil pass re-derives it from the same stream contract rather than taking it
through the step signature — the derivation is exact today and is flagged
in its doc comment.

### Deviations + deferred

- The remix inverted-terrain family (~211 `remixWorldGen` sites: the
  inverted bands of `TerrainPass`, the doubled cave counts, the NPC swaps,
  `FinishRemixWorld`'s cloud/wall conversions) is deferred — it is a
  shape-level rewrite of the compact terrain model, not a branch add. The
  row's remix coverage is the flags + the two `Reset`-range arms + the
  spawn override.
- The not-the-bees hive world (`NotTheBees`/`FinishNotTheBees`'s ~20 sites)
  is deferred the same way; its flags resolve and its runtime conditions
  read them.
- The drunk per-blob ore re-rolls (the 22 `if (drunkWorldGen) { Next(2) }`
  tier sites `:13194-13577`) and the drunk double evil-ore bands
  (`:13370-13378`, `:13577-13586`) stay deferred: the port's ore band table
  carries one tier per pair and the re-rolls would need a per-blob tier
  pair, i.e. a table rewrite.
- `ImproveAllChestContents` (`:24716-24765`, the celebration prefix pools)
  is loot-side; the port places chests as tiles without inventories, so
  the chest arms of the three Finish passes (5001/678 drops) draw and write
  nothing.
- `ConvertSkyIslands` (`:24243`) needs the C# `Convert` Hallow helper the
  port does not have; the `PaintThe*` sweeps and the 138→665 conversion
  run without it.
- The drunk `getGoodWorld`/`notTheBees` combo first branches (spider caves
  `:17369-17375`, moss `:17510-17516`'s re-roll) stay out: the port's
  classic arm draws once, so porting the C#'s extra re-roll would break
  the classic byte-identity the whole branch is built on.
- Runtime follow-ups for the flags now that they are live: the FTW AI/damage
  family (`Main.getGoodWorld`'s ~100 `NPC.cs` sites), the anniversary
  party, the Constant's starving/darkness 

## The tower mobs (M4-EVENT-13)

The Stardust tower rolls `SelectRandom(411, 411, 411, 409, 409, 407, 402,
405)` (`NPC.cs:1349`, no alive caps — the cell/worm caps live in
`SpawnStardustMark_StardustTower`); the Solar tower rolls
`SelectRandom(518, 419, 418, 412, 417, 416, 415)` with the 518 and 412
rerolls (`:1355-1384`). 410 and 402 never come from the table: 410 only
from 409's attack.

- **The big spider (409)** is in the shared shooter list but fires no
  projectile: its 60-tick cycle (`:59155-59160`) spawns the 410 at its
  centre with the carried velocity (`:59585-59589`), and its walk runs
  the 2-px cap / 0.5 accel (`:59746-59750`).
- **The small spider (410)** is an aiStyle-26 unicorn walker (`:62470`)
  with the 6-px / 0.2-accel tail (`:62706-62713`): the 240-tick counter
  (or a player 800 px below inside a 20-px column) fires three 538s (50,
  `X ∈ [-1, 1]`, `Y ∈ [-14, -4]`) and then self-destructs.
- **The soldier (411)** runs the 330-tick cycle (`:59151-59153`) firing
  the 537 at 60 damage from the muzzle (`:59271-59274`, `:59470-59473`,
  `:59574-59577`); its [120, 240) window (`:59076-59081`'s
  `flag16`/`num160`/`num161`) is the aerodynamic phase and its walk runs
  the 2-px cap (`:59741-59745`).
- **The drakomire (415)** rides the walker with the 4/0.1/0.95 jump
  assist (`:58214-58219`) and hides while a 416 links to it
  (`:58422-58431`).
- **The rider (416, aiStyle 75)** spawns its own 415 mount at spawn
  (`:35523-35537`), rides it at the `(-spriteDirection * 10, -30)`
  offset with the mirrored direction/sprite (`:35662-35679`), and
  `Transform(518)`s when the mount dies (`:36060-36061`).
- **The sroller (417, aiStyle 39)** aggro-counts to 400 → the wind-up
  (`:29394-29419`'s 30 ticks) collapses it into the ball (the 32-px
  height, `ai[2] = rand(2, 4)` revolutions), the ball doubles the defense
  and scales the damage ×1.8 (`:29532-29580`), and the unroll restores
  the 52-px height (`:29634-29668`).
- **The corite (418, aiStyle 74)** approaches at the 8-px cap
  (`:35238-35276`'s own parameter set), charges to the 9-px dash and
  then cools down; its state-3 arm (or the 64-px contact) triggers the
  implosion: the 192-box, the zeroed velocity, 80 damage
  (`GetAttackDamage_ScaledByDifficulty`) and the 3-tick death
  (`:35447-35497`).
- **The selenian (419)** walks with the 6/0.15/0.85 set (`:58220-58224`).

Deviations: the dust/gore `HitEffect` rows, the corite's lighting/sound
arms, the sroller's `ai[3]` FX layer and the tower spawn tables are
unmodelled; the sroller's height collapse (46×52 → the ball) rides the
defense/damage overrides rather than a box change.
## Xorshift core unification (refactor-xorshift-unify)

One xorshift64 core now feeds the worldgen streams and the wind walk alike
(`crates/tr-core/src/rng.rs`, `pub struct Xorshift64(pub u64)`), replacing two
private cores and two inlined copies of the same step. Before:

- **`worldgen/noise.rs`'s `Rng`** — the worldgen contract: raw-XOR seeding
  (`Rng(seed ^ const)` for the three `pipeline::Streams`, `Rng(seed ^
  fnv1a64(name))` per pass), the high-32-bit word draw `next_u32`, plus
  `range`/`chance`/`next_f64`.
- **`wind.rs`'s private `Rng`** — the splitmix64 seeding (`from_seed`: the
  `0x9E3779B97F4A7C15` add, the `0xBF58476D1CE4E5B9` and `0x94D049BB133111EB`
  multiplies, the `0x2545F4914F6CDD1D` all-zero guard) with `next_int` taking
  `next_u64`'s high half modulo.
- **The duplicates** — `worldgen/temple.rs`'s `next_double` (= `next_f64`) and
  `menu/mod.rs::random_name`'s inlined 13/7/17 triple.

The step and the word extraction were already identical in both cores, so the
unified struct carries both draw conventions: `next_u64` returns the full word,
`next_u32` returns its HIGH 32 bits (the worldgen contract), `from_seed` is the
splitmix seeding, and `range` (an empty range returns `lo` without a draw —
wind's `next_int` was this function) / `chance` / `next_f64` round it out. The
worldgen surface is unchanged: `noise.rs` re-exports the core as `Rng`
(`pub(crate) use crate::rng::Xorshift64 as Rng`), so every pass's import and the
seeding contract stay as they were, while `wind.rs`, `temple.rs` and
`random_name` call the core directly.

Byte-exactness is pinned, not assumed: `rng.rs`'s tests replay the
pre-refactor cores **verbatim** — the worldgen `old_next_u32` (7 seeds × 512
draws), wind's `old_next_int` over its real call-site ranges (5 seeds, 16
range windows) and the splitmix chain over the seeds 0..4096 — while the
suites that consumed the old cores (`deterministic_for_seed`, the worldgen
replay-pair tests and the wind tests) stay green unchanged.

- **Out:** the ported C# stream `FastRandom` (`tile_light/palette.rs`, the
  48-bit LCG) stays as it is — the `UnifiedRandom` replay, not this core;
  `audio.rs`'s xorshift64*** (the 12/25/27 triple with the `0x2545F4914F6CDD1D`
  multiplier) is a different core and stays; the ~13 other copies of the
  splitmix64 finalizer — `spawner.rs`'s splitmix core misnamed `FastRandom`, the
  `next_rand`/`rand01` streams of `rain.rs`/`sand.rs`/`chest_gen.rs`/
  `prefix.rs`/`mount.rs`/`app/frame.rs`, `wiring.rs`'s LCG + finalizer — and
  `flame.rs`'s second 48-bit LCG copy are recorded as a follow-up engineering
  row.

## Magic staves (M4-PROJ-6)

The port's caster families land in `crates/tr-app/src/projectile.rs` beside
the arrow/boomerang/bomb arms. The slot state the machines need
(`localAI`/`alpha`/`scale`) and the row defaults the spawn applies ride the
C# `SetDefaults` reset group; the dispatch arms 8/9/18/22/24/28/29/47/48/50/
129 plus the aiStyle-1 rider gate sit at `ProjectilePool::update`'s
`match data.ai_style`.

Deliberate deviations and skips (all one-line-cited in the code):
- **Dust and sound are skipped in every arm** — the pool has no dust layer
  and no sound sink; the `Main.rand`/`SoundEngine` calls inside the C# AI
  blocks are recorded per arm. The frame animations (`frameCounter`/`frame`)
  are skipped the same way: no frame fields on the slot and no frame slicing
  in the projectile draw. Both are the render/Audio follow-up.
- **The Magnet Sphere's candidate pick is the first in `Main.npc` order** —
  the C#'s `Main.rand.Next(num)` pick has no pool RNG (the determinism rule);
  identical to the vanilla for the single-candidate case the spawn usually
  sees. The rider damage/penetrate bumps (the lasers' `rand.Next` rolls) are
  skipped for the same reason.
- **AllGems2's seven-bolt branch is ported but unreachable in this
  decompilation** — only `AllGems` (the Bejeweled Staff 6171) has a writer in
  `Player.cs`, so bit 9 is never set; the initial `NextFromList` draw the
  branch's loop overwrites is therefore not consumed.
- **Wind physics runs only for the new arms** (styles 8/24/28/29, the C#'s
  `ShouldUseWindPhysics` `:22101-22124`, applied after the AI sub-steps). The
  ported arrow/bomb arms keep their pre-existing no-wind behaviour — the fix
  is behaviour-affecting and belongs to a follow-up row. `Main.windPhysics`
  is false in this decompile, so the pool's `wind_speed` defaults to inert
  and the faithful behaviour is "no wind" either way. The C#'s
  `MathHelper.Clamp` on the tail line discards its return (a no-op in the
  source) — ported as written.
- **The PVE walk's fixed centre point stands in for the boxes** in the
  normal (flight-size) pass; the resized death bursts (the gem bolts' AoE)
  ride the new `damage_pve_boxed` rectangle walk `Projectile.Damage` actually
  runs — without it the burst's 160×160 box would deal no damage.
- **The channel reads a one-tick-old aim**: the frame runs
  `update_projectiles` before `update_editing`, so the aim the spell reads
  was written on the previous tick — the same net state as the C#, where
  `ItemCheck` precedes `UpdateWorld_Projectiles` and the owner block reads
  the previous tick's player state too.
- **The Bejeweled Staff 6171 rides `registry::ITEM_CORRECTIONS`** — the
  GenItemData placeholder band (3263-3989) still emits the zero row for it
  (the generator misses its case); the id-keyed correction is the `place_tile`
  3813 precedent. The band's other in-scope staves (3377/3779/3787) were
  fixed by the M6-SAVE-1 `itemdata.rs` regen and now ride the generated rows.
- The style-22 owner tile writes (the Ice Rod's `PlaceTile`/`KillTile`) run
  at the owner branch through `tr_core::edit`; the frozen block variant of
  the hostile 291 style-50 branch stays out (no ported row).

The batch review's three spell-arm findings (the missing
`MathHelper.Lerp(0f, 0.2f, …)` wrap on the aim blend, the 800f target-search
default and the retarget's closest-wins loop) were fixed forward against the
decompilation and pinned with tests.
## .trwld v6 — the world flags, wires and tile entities (M6-SAVE-1) (re-landed)

Re-landed: the `M5-CHEST-gen-flag-extras` merge (`07d786ab`) dropped this work from
main with its merge resolution. The section and its code are restored verbatim here;
the v7 chest section builds on it.

The native `.trwld` (`crates/tr-core/src/save.rs`) grew these sections after the v5
bed-spawn byte. They landed as the v6 tail (`VERSION` 5 → 6), but a later merge
dropped them and claimed the v6 slot for the secret-seed flags; the re-landed
sections therefore ride the v7 gate (`VERSION` 7 — the seeds u16 at v6, then the
flags block, the wires and the tile entities at v7, the chests behind them).
Before them the three state groups were simply dropped by the save: the recorded gaps were the wires
("Persistence in `.trwld`/`.map` is pending", the "Wiring engine (M7-WIRE-1)"
deviations), the tile entities ("Persistence in `.trwld`/`.map` is pending
(M6-SAVE-1)", the "Tile entities (M7-TE-1)" deviations) and the `SaveWorldFlags`
progression fields — the world's `downed_*`/invasion/hardmode state only ever
survived until the next reload, and `unlocked_slime_rainbow_spawn` had no save
slot at all (the "Shimmer transforms (M7-SHIMMER-1)" note). Each group rides its
own version gate, so a v5 (or older) file still loads, with the `World::new`
defaults standing in exactly as they do for the v3/v4/v5 tails.

**1. The world flags block** — `save.rs::write_flags_v6`/`read_flags_v6`: a fixed
**108 bytes** = two 16-byte bool groups + 14 i32s + 2 f64s + 1 f32 (the second
group's last byte is a reserved 0 that keeps both groups 16 bytes; a back-compat
test truncates exactly `V6_FLAGS_LEN` bytes). It carries the port's `World`
fields that the C# `SaveWorldFlags` persists (`WorldFile.cs:1238-1430`), grouped
with the C#'s own line citations:

- bool group A (16): `_tempBloodMoon`/`_tempEclipse` (`:1277-1278`), the
  downed-boss ladder (`:1282-1294` — Boss1/2/3, QueenBee, MechBoss1/2/3,
  PlantBoss, GolemBoss, SlimeKing, savedGoblin; the port has no
  MechBossAny/savedWizard/savedMech state, so those slots are not written),
  `downedGoblins`/`downedFrost`/`downedPirates` (`:1296-1299`).
- bool group B (15 + the reserved byte): `shadowOrbSmashed`/`spawnMeteor`/
  `hardMode` (`:1300-1304`), the DD2 event flags (`DD2Event.Save`,
  `DD2Event.cs:151-156`), `downedMoonlord` (`:1346`), `TowerActive*`
  (`:1357-1360`) and `downedTower*` (`:1352-1355`).
- the i32 block: `shadowOrbCount`/`altarCount` (`:1302-1303`), the invasion state
  (`invasionDelay`/`invasionSize`/`invasionType` `:1306-1309` +
  `invasionSizeStart` `:1339`), the four tower shields,
  `lunar_shield_power_normal`, and the port-only slime-rain/DD2 counters; the
  two f64s `invasionX` and `slimeRainTime` (`:1309-1310`) and the f32
  `dd2_wave_kills`.

**2. The wire side table** — `World.wires` (`wiring.rs::WireState::write`/`read`):
per colour — the `WireColor` declaration order Red/Blue/Green/Yellow, which is
also `TripWire`'s four sequential pass order (`Wiring.cs:525`) — a u32 cell count
then the (i32, i32) pairs **sorted by (x, y)** (`HashSet` order is unstable, the
file bytes must not be). The C# has no wire section at all: the four colours ride
per-tile `sTileHeader` bits packed into the tile stream (`Tile.cs:623-657`) and
the port's 12-byte `Tile` has no header space, so the side table needs its own
section — a port-native shape, recorded as a deviation.

**3. The tile entities** — `World.tile_entities`
(`tile_entities.rs::{TileEntity::write/read, TeData::write/read(r, kind),
TileEntities::write_all/read_all(r, world)}`): a u32 count, then per record a
u8 kind, a u32 id, an i16 x/y pair and the per-kind payload — `TileEntity.Write`/
`Read` (`TileEntity.cs:173-207`, the `WriteInner` half of the save path: the id
rides, the network send omits it) plus the world-file section
`SaveTileEntities`/`LoadTileEntities` (`WorldFile.cs:3302-3357`), including the
load-side re-validation: a record whose cell left the world (`WorldGen.InWorld`,
`:3334`) or whose `CheckValidTile` frame test fails (`:3338`) is dropped, the
payload emptied. `TeData::write`/`read(r, kind)` mirror `WriteExtraData`/
`ReadExtraData` per kind — the item triple `short type, byte prefix, short stack`
behind the port's presence byte (`TEItemFrame.cs:47-56`), the HatRack
presence-bitsByte (`TEHatRack.cs:59-80`), the dummy's `npc`
(`TETrainingDummy.cs:119-126`) and the sensor's `logicCheck`/`On`
(`TELogicSensor.cs:380-396`).

Deviations and notes:

- **The format is the port's own, not the C#'s 11-section pointer table.** The
  `.wld` writer emits an `i16 sectionCount` + `i32 sectionPointers[]` index that
  the reader then walks; `.trwld` appends each new group in order behind its
  version gate, so a format bump is one new tail and old files keep loading. All
  three groups are port-native sections even where the C# folds them into another
  stream — the flags into a flat `BinaryWriter` walk with no section markers at
  all, the wires into the per-tile header bits.
- **Port-only persisted state**: the slime-rain flag and kill counter
  (`slime_rain`/`slime_rain_kills` — the C# writes only `slimeRainTime` and
  derives the rest on load), the DD2 wave counters (`dd2_wave`, `dd2_spawn_hold`,
  `dd2_wave_kills`), and the tower shields (`tower_shield_*`,
  `lunar_shield_power_normal`) as live values — the C# restores the four tower
  shields from `ShieldStrengthTowerMax` on the load side
  (`WorldFile.cs:2237-2249`), so saving and restoring the numbers is a different
  contract rather than a port of it.
- **Deliberate widenings on the TE record**: the dummy's npc int16 → i32 (the
  port keeps the tie as the NPC slot) and the logic sensor's `counted` grace
  (`CountedData`, runtime-only in the C#) is persisted so a load restores the LED
  state exactly.
- **The port keeps the stored TE ids** instead of reassigning them on load (the
  C# does `tileEntity.ID = num2++` and then sets `TileEntitiesNextID`,
  `WorldFile.cs:3323`/`:3330`), so a save round-trips them and `next_id` runs to
  `max(id) + 1`; a duplicated cell keeps the `place`/`place_unchecked` insert
  semantics (the last record is the one the position book reaches) and a repeated
  id in a hand-written file keeps the first record and drops the rest.
- **The runtime-only fields stay out**: the WoF/Golem/boss write-through anchors
  (`wof_*`, `golem_*`, `skeletron_*`, `brain_*`, `prime_*` — live boss-encounter
  state the C# rebuilds on spawn), `travel_shop`, and the App-side weather block
  (`_tempRaining`/`_tempRainTime`/`_tempMaxRain`, `:1312-1314`), which stays in
  the app layer until the load-side weather block is ported. The C# seed-flavour
  pair (`vampireSeed`/`infectedSeed` and the `Main.*Seed` family) and the saved
  NPC/bought-pet writes have no port fields either.
- **Verification**: `save.rs`'s `dw_2_1_the_v6_flags_wires_and_tes_round_trip`
  (one flag from every group + the counters + the four wire colours + the
  dummy/frame/sensor/pylon payloads), `dw_2_1_a_v5_payload_loads_defaults` (the
  old file keeps the defaults), `dw_2_1_a_truncated_v6_tail_bails` and
  `dw_2_1_the_wire_and_te_sections_are_deterministic` (byte-identical re-writes),
  plus the updated version-gate tests. `tile_entities.rs` adds 5 (the round-trip
  with the `write_all` layout pin, the invalid-cell drops, the unknown-kind
  refusal, the duplicate-cell semantics); `wiring.rs` adds 3 (the round-trip over
  every colour, determinism, the truncated read). Full suite: 2327 passed /
  0 failed (`cargo test --workspace -- --skip migrates_version_2_kinds_to_item_ids`).

Out:

- the **chests** — the App-side `App.world_chests` store is re-rolled at load by
  `chest_gen` (the `M5-CHEST-gen` row's documented conflict: the generator and the
  saved contents disagree), so the section needs both a save path and a load gate;
- the **signs** — the port has no sign store at all;
- the **NPCs/homes/names** — the NPCs live App-side (`App.npcs`) with `Npc.home`
  in memory (the `M4-TOWN-1` recorded note), so the section follows the NPC
  persistence;
- the **bestiary kill pairs** — today they ride the `.trplayer` sidecar v5
  (`bestiary.rs::KillTracker`), while vanilla keeps them in the world file
  (section 11), so moving them means a world-side field;
- the **banner counts** — `banners.rs` is a runtime tally with no save;
- **SavedOreTiers** — the worldgen-local `ores.rs` struct needs a `World` field
  first (the "OresAndShinies fidelity (M5-ORES-1)" note records the gap), and the
  vanilla quartet has no port representation yet;
- the **angler quest state** — `Main.anglerQuest`/`anglerWhoFinishedToday` have no
  port fields (`fishing.rs` carries the mechanics only);
- the **TownManager rooms** — the room store follows the housing/persistence work;
- the **weighted pressure plates** — no port store for the pressed-plate state;
- the **creative powers/research** — `M6-CREATIVE-1/2`'s per-player state has no
  persistence yet.

Each is its own scope; none is implied by this tail.

## .trwld v7 — the world chests (M6-SAVE-1b)

The chest section rides the store's data as neutral records (`tr_core::save::ChestItem` /
`ChestRecord` — the crate cannot see `App`): a `u32` count, then per chest `i32 x`,
`i32 y`, the name (`u32` length + UTF-8 — the file's own string convention) and a
`u32` slot count, then per slot `i32 stack` and, while the stack is non-zero, `u16
kind` + `u8 prefix`. The C# shape is `SaveChests` (`WorldFile.cs:1606-1676`) / `LoadChests`
(`:2723-2805`).

- **The deviations.** The vanilla always writes `maxItems` slots (40) and a 7-bit
  length string, and its load re-assigns ids; the port keeps its implicit-air
  compressed vec (a `stack == 0` entry is an empty slot, trailing airs dropped — the
  store materializes the 40 on open), its own string encoding, and the record's
  anchor is the identity. The stack is `i32` where the C# writes `int16` (the vanilla
  9999 cap fits, the port's `u32` counts saturate on write). The 8000 cap is not
  written: the port's store already refuses past `MAX_CHESTS` (`chest.rs:40`).
- **The app wiring.** `save_world` serializes `App.world_chests` — removed entries
  skipped, the effective name from `chest::chest_name` (the rename or the derived
  default), the slots mapped through `registry::name`. `App::new` takes the records,
  `chest_gen::store_from_records` culls the stale anchors (the vanilla's pre-count
  validity sweep) and maps the slots, and the records seed `merge_store` ahead of the
  `TR_CHEST` fixture — **the saved contents win over the chest-gen re-roll**, so a
  player's stock survives the reload; the names ride `chest_names` by the post-merge
  index (`find_chest` on the anchor). The autosave (10 s), F9 and quit write the
  section; the MP world transfer does not carry chests (the tile sections do).
- **The recorded gap, fixed 2026-11-09 (`fix-chest-frame-important`).**
  `boot_statics` had never set the chest family's bits while the vanilla does
  (`Main.cs:7655-7659`: the sapling 20, the chest 21, the 467/441/468 container kinds),
  so `frame_world` re-framed every chest tile through its object def — the 2x2 anchor
  frames were lost and the anchor scans missed every chest in a loaded world (the roll
  only fired in a freshly generated, unframed one). The bits now ride the boot statics
  beside the other family groups; the test pins the survival with a control arm (the
  same world with the bit cleared loses the anchor).
- Tests: `dw_2_2_the_chests_round_trip_in_v7` (names, interior airs, the prefixes),
  `dw_2_2_a_v6_payload_loads_no_chests`, `dw_2_2_a_truncated_chest_section_bails`,
  `dw_2_2_the_chest_section_is_deterministic`, and the app pair
  `dw_2_2_a_saved_chest_round_trips_through_the_app` (contents/prefix/name survive,
  the no-roll-overwrite control) / `dw_2_2_a_record_with_a_missing_anchor_is_dropped`
  (the cull). Suite 2409 / 0.

Out: the remaining C# sections — signs (no port store), the NPC block (names/homes/
homeless, App-side `App.npcs`), the bestiary kill pairs (today the `.trplayer` sidecar
v5; moving them into the world file), the banner counts, `SavedOreTiers`
(worldgen-local — needs a `World` field), the angler quest state (no port fields),
the TownManager rooms, the weighted pressure plates (no port store) and the creative
powers/research — each needs its own scope.

## The achievements menu + the 137-row registry (M6-ACH-2)

- **The registry is one initializer, three passes.** `AchievementInitializer.Load`
  (`:13-441`) constructs 137 `Achievement` objects (`Achievement.Id` is the
  construction counter), then `RegisterIconIndex` (`:445-582`) assigns the icon
  frame each row draws from `Images/UI/Achievements` and
  `RegisterAchievementCategory` (`:584-736`) the filter category. The port keeps
  all three as one `Row` table (`achievements_data.rs`) so the icon order and the
  `Id` order can never drift (they differ: `BENCHED` is constructed 2nd but
  carries icon 88).
- **One condition can list many ids.** `NPCKilledCondition.Create(13, 14, 15)`
  is a *single* condition named `NPC_KILLED_13` that completes on **any** of the
  three (`NPCKilledCondition.cs:20-30`, the `_listeners` fan-out). The port's
  `CondKind` therefore splits into the single-id event variants and the `…Any`
  multi-id stored variants, with `matches_kind` doing the membership test. The
  pre-M6-ACH-2 registry modelled these as one condition per id (all-of), which
  is not the vanilla behaviour.
- **`ItemCraftCondition` reuses the `ITEM_PICKUP_` name prefix**
  (`ItemCraftCondition.cs:8`), so a craft condition and a pickup condition in the
  same achievement would collide on `Achievement._conditions`' dictionary key.
  No shipped row does (each row's first ids differ), which the
  `every_row_carries_a_distinct_condition_key` test pins.
- **`STAR_DESTROYER` has no conditions at all** — vanilla's
  `IsCompleted => _completedCount == _conditions.Count` is `0 == 0`, so the row
  is always complete (it always sorts to the top of the list). The port keeps
  that quirk.
- **The category filter cannot name the `None` row.** Vanilla's `FilterList`
  indexes `_categoryButtons[(int)achievement.Category]`, and
  `GAIN_TORCH_GODS_FAVOR` is the one row `RegisterAchievementCategory` never
  touches (`AchievementCategory.None = -1`) — the C# would throw an
  `ArgumentOutOfRangeException` on a filter click. The port keeps the secret row
  visible in every filtered view instead of crashing (documented deviation).
- **The vanilla save is not written.** `AchievementManager.Save` is
  Rijndael-CBC+BSON under the "RELOGIC-TERRARIA" key (`:40-79`); the port keeps
  its own `achievements.trdat` text format (the M6-ACH-1 deviation) — writing
  the vanilla format would touch a real install's `achievements.dat`.
- **The menu's `largeForOtherLanguages` flag is always `true`** (`flag = true` in
  `InitializePage`), which is what fixes the row height at `66 + 16 + 20 = 102`
  and the `MinWidth`/`MaxWidth` at 700/900. The in-game arm (`OnActivate`'s
  else) drops the container to `Top 120 / Height -120`; the menu arm keeps
  `220/-220`. The port picks the arm from whether the legacy menu is live.
- **Deviations:** no word wrap (`CreateWrappedText`) — the row's description
  shrinks to fit the text width instead of growing to a second line; the
  progress bar is two flat quads rather than the `ColorBar` 9-slice + the
  `MagicPixel` fill/blip splices; the scroll rides the mouse wheel instead of the
  `UIScrollbar`; the gamepad link points (`SetupGamepadPoints`, ids 3000+) are
  out of scope; there are no UI click sounds.

## The pillar worms (M4-EVENT-14)

The Crawltipede (412/413/414) is the Solar tower's worm: the head
`ai[3] = whoAmI` + `realLife = whoAmI` (`:51369-51392`), `num34 = 30`
segments, `num36 = 413` swapped to `414` at index 29, each segment
linking forward (`ai[0]`) and back (`ai[1]`), `CopyInterrelations` from
its predecessor. The port's worm pass carries the same shape (`worm_id`,
`seg_index`, `worm_left`), so the chain rides it with the crawltipede's
30-segment budget and the 10/0.3 steering pair. The death guards kill a
segment whose link died (`!Main.npc[ai[1]].active || aiStyle mismatch` →
`life = 0; HitEffect(); checkDead()`), and the body/tail don't count for
the enemy cap (`dontCountMe`) while the tail takes ×10 damage
(`takenDamageMultiplier = 10`, consumed in `StrikeNPC_Inner`).

The StardustWormHead (402) is a lone mega-worm (no segments in this
build — 403/404 have no `SetDefaults` rows) whose aiStyle-6 arm
(`:51030-51073`) runs the 600-tick cycle: `ai[2]++`, the
`Remap(ai[2], 285, 570, 1, 0.5)` deceleration while `ai[2] <= 570`, the
mark fire at 570 (`SpawnStardustMark_StardustWorm`) with the velocity
re-normalised to 6, and the reset at 600. Its own movement pair is
`num45 = 9` / `num46 = 0.3` (`:51740-51743`).

`SpawnStardustMark_StardustWorm` (`:43909-43993`) picks from the list
405/402/407 (capped at 2 / 3 / 1 alive) and throws the 540 marker
projectile with `ai[0] = -num5 * 10f` and `ai[1] = 0.5..1.0f` which
spawns the chosen NPC on its way; the port folds the hop into the sim's
resolve (`wants_stardust_mark`) because the 540 projectile AI is not
ported.

Deviations: the dust arms, the ground-avoidance tile scan
(`:51784-51798`) and the same-type repulsion of the crawltipede are
unmodelled (the port's worm pass has its own steering), the 540
projectile hop and the mark's 3-5 chain-length roll are unmodelled.

## Minions & sentries (M4-PROJ-8)

The summon system's harness: the per-player slot budget (`maxMinions`/`numMinions`/
`slotsMinions`, `Player.cs:1260-1264`), the keep-alive buff arms, `AI_062`'s
follow/target/return machine and the sentry cap/lifetime/targeting.

Deviations and notes:

- **The 0-time summon grant is a ghost.** The summon staves' `Item.buffTime` rows are 0
  (itemdata, the C# `Item.SetDefaults` cases set only `buffType`), so `ItemCheck`'s grant
  (`Player.cs:42879`) adds a 0-time buff — and the C#'s arm guard (`Player.cs:9868`,
  `buffTime[j] <= 0` skips) plus the expired sweep (`:25611`) drop it on the next tick.
  The port repeats this verbatim: a freshly-granted minion buff is swept one tick later,
  and the arm's 18000 refresh only reaches buffs that entered with a non-zero time (the
  regression tests seed one). The minion itself lives on its row's `timeLeft * 5` and the
  pool's cap check either way.
- **The port's arms run after the player tick's expiry sweep** (the Phase-2 seam): the C#
  runs them inside `UpdateBuffs` between the reset and the sweep. The consequence is the
  ghost above; the cap effects (110/150/348) still land after `ResetEffects` zeroed the
  budgets, the order that keeps them.
- **The Stardust Dragon chain (625-628) is excluded** — the relink arms in the kill-oldest
  and the free-up are stubbed; the kinds carry rows only where the summon slice needs them.
- **DD2 persistence stubbed**: `tr_core::dd2::event_ongoing()` reports false until
  M4-EVENT-5, so the O'A towers (663-693) are wipable like the vanilla sentries.
- **Dust, sound, light and frame machines unported**: `AI_062`'s dust spawns and their
  `Main.rand` rolls, 963's cry/attack cues, 613's `Lighting.AddLight`, the per-kind
  `frameCounter` machines and `strongBees` (373's bee cooldown keeps the default 90) —
  the port has no general `Main.dust` pool and the projectile slot carries no frame
  fields. The cooldown machines' `rand(1,4)`/`rand(3)` extra increments stand at their
  deterministic minimum.
- **LOS approximation**: `ai_flying_minion`/`sentry_npc_targeting` route `CanHitLine`/
  `CanHit` through the shared tile walk (`can_hit_line`, the yoyo's `yoyo_can_hit`
  renamed) and `SolidCollision` through a five-point box test (`box_solid`) — the C#'s
  exact Bresenham+rect semantics differ at the sub-tile margins.
- **The attack shots ride back as `MinionShot`** and drop until the shot families
  (374/376/408/433/614) have `proj_data` rows; the state machines run either way.
- **261/266 note**: the Slime minion (266) lands its row with aiStyle 26 (the hop family
  unported — the dispatch no-ops like the other unported styles), 24x16, `penetrate = -1`,
  `timeLeft * 5` and the idStatic-12 pair.
- **Phase 5's scan guard**: `AI_053_HandleSentryNPCTargeting`'s full scan runs only when
  the whip target supplied no answer (`if (!flag)`) — ported verbatim.
## Whips & tag damage (M4-PROJ-7)

The whip family ports `AI_165_Whip` (`Projectile.cs:47667-48270`) in two halves:
the pure geometry (`tr-core/src/whip.rs`: `GetWhipSettings` `:48337-48419`,
`FillWhipControlPoints` `:48278-48335` and the `Colliding` whip arm `:15716-15737`)
and the state (`crates/tr-app/src/projectile.rs`'s `ai_whip`, the 18 `proj_data`
rows off `DefaultToWhip()` `:10704-10718`). The tag-damage system is
`crates/tr-app/src/tag_effects.rs` (`TagEffectStack.cs` + `TagEffectState.cs`),
the tag table's `TagDamage` column is the `WhipTagEffect` initializer
(`ItemID.cs:1473-1580`) — NOT the item damage — and the tagged-hit math is the
`Projectile.cs:12924` line, gated by `WhipTagEffect.CanRunHitEffects`'s
minion/sentry shot test.

Deviations recorded here:

1. **The arm anchor.** C# anchors the curve on `Player.GetArmPosition()` =
   `OffsetsPlayerOnhand[bodyFrame.Y / 56] * 2` (a 20-row table,
   `Main.cs:66615`); the port's player has no body frame, so the anchor is the
   player center (`Player::center()`). The whip's hitbox shifts by the on-hand
   offset's order (≤12 px); the render pass can adopt the frame row. The sweep
   itself (the curve's shape, the reach formula, the union test) is 1:1.
2. **The whip never self-crits.** `DefaultToWhip` sets no melee/ranged/magic
   flag, so `Projectile.Damage`'s crit block (`:12892-12899`) never fires for a
   whip — the vanilla whip crits only through a tag's `CritChance` (4679: 10,
   5478: 5, 5479: 15, 4914: 10, 5480: 10). The generic throw path rolls crit at
   spawn for every other shot; the whip branch passes `crit = false` so a
   proc-less tag cannot force the variance.
3. **The local-immunity decay.** C#'s `DecrementLocalImmuneTimeCounters` decays
   only **positive** counters (`:17274-17277`), so the whip's
   `localNPCHitCooldown = -1` sentinel never expires: each whip projectile hits
   each NPC once. The pool's Rust decay was `(v - 1).max(0)`, which turned the
   sentinel into 0 on the next tick (every whip re-hitting each NPC per tick);
   it now decays positives only, and `the_whip_hits_each_npc_once_across_the_swing`
   pins the behaviour.
4. **The press edge.** The use path compared `mine_released`/`mine_prev_down`
   against the *same* frame's input (the assignments run at the top of
   `update_editing`), so the non-`autoReuse` gate
   (`Player.cs:42733`'s `controlUseItem && releaseUseItem`) was dead. The
   previous frame's edges are captured before the overwrite and the gates read
   them — the whip's one-swing-per-press behaviour (and the shuriken's) depend
   on it.
5. **Deferred (documented follow-ups).** The tag effects' projectile spawns —
   916 (DarkHarvest), 917 (CoolWhip snowflake), 918 (Firecracker explosion),
   1036 (CobWhip spark), 1037 (Starcrash meteor), 1038 (FlowerWhip petals),
   1039 (Constellation star fan), 1045 + `SpawnMoonLordWhipProc`'s `Next(1,4)`
   eye beams and the ElectricEel's chain damage (`TryHittingNPC`) — wait for
   their projectile kinds; the whip's tile-cutting arm
   (`Projectile.cs:15210-15220`), the melee-enchant `StatusNPC` arms, the
   half-flight and proc sounds, the dust/particle orchestras, the draw ribbon
   (`Main.cs:36547`'s `GetWhipAnimationPercent`), `heldProj`/`drawLayer`,
   `MinionAttackTargetNPC`'s consumers (M4-PROJ-8), the player-side melee tag
   hook (`Player.cs:44448-44451`) and the accessory modifiers
   (`whipRangeMultiplier`, `maxTagEffects`, `tagEffectDuration` — the defaults
   1/1/1.0 port, the M3-EQUIP arms wait) stay out until their own rows.
6. **The tag gate.** No ported shot is a minion/sentry shot, so
   `CanRunHitEffects` admits nothing in-game until M4-PROJ-8; the
   `is_summon_shot` slot flag carries the gate and the tests set it directly,
   so the math is proven end-to-end (`a_tagged_summon_shot_takes_the_damage_bonus`).
## The lunar chain (M4-EVENT-15)

The apocalypse is a state chain, not a machine: the cultist's death →
`WorldGen.TriggerLunarApocalypse` (the four towers) → the towers' deaths
→ `WorldGen.UpdateLunarApocalypse` + `StartImpendingDoom` → the countdown
tick → the Moon Lord's spawn.

- **The cultist's death** (`DoDeathEvents`'s 439 row,
  `NPC.cs:79393-79395`) clears `downedAncientCultist` and raises the
  apocalypse. The port's `AncientCultist` is the tower-shaped kind (its
  machine is the aiStyle-94 tower machine), so the vanilla's 439 row maps
  onto it directly.
- **`TriggerLunarApocalypse`** (`WorldGen.cs:86626-86691`) draws the four
  tower ids `{517, 422, 507, 493}` without replacement into the four world
  slots at `maxTilesX / 5 * (1 + j)`; the ±100-tile jitter retries 30
  times against the vertical clearance scan, with the
  `(num2 - 40) * 16` fallback when all fail. All four `TowerActive*` flags,
  `LunarApocalypseIsUp` and the four `ShieldStrengthTower*` (to
  `ShieldStrengthTowerMax` — 100, halved post-Moon-Lord) raise together.
  The port's fresh worktree brings the four towers up in the same
  draw-out order and clears the slot with its own tile lookup.
- **The towers' deaths** (`NPC.cs:79321-79344`) set the `downedTower*`,
  clear the `TowerActive*` and call `UpdateLunarApocalypse`, then
  `MessageLunarApocalypse` (`WorldGen.cs:86778-86801`) picks
  `Lang.misc[43 + down]` from the inactive count.
- **`UpdateLunarApocalypse`** (`:86693-86748`) re-derives the four flags
  from the live NPCs (plus the 398/396/397/400 boss family) and, when all
  four are down and the boss isn't up, `StartImpendingDoom(3600)`.
- **`StartImpendingDoom`** (`:86750-86760`) ends the apocalypse, arms
  `MaxMoonLordCountdown`/`MoonLordCountdown` and (in the vanilla)
  `GetRidOfCultists` kills the 437/438/439 ritual kinds — constants-off
  in the port (its ritual support isn't ported).
- **The countdown tick** (`Main.cs:64987-64994`) decrements once per world
  update and at zero calls `NPC.SpawnOnPlayer(Player.FindClosest(maxTilesX
  / 2, worldSurface / 2), 398)`; the port's single player stands in. The
  Celestial Sigil's route (`Player.cs:44028-44034`) is the same function
  with the item's 720 (`NPC.cs:6037`'s `ItemMoonlordCountdownTime`) and
  the Golem/hardmode/danger gates the vanilla's `CanStartInvasion`-like
  family uses.

Deviations: the Moon Lord's boss bar/head textures, the `MoonLordShake`
scene filter, the music ducking and the countdown UI are unmodelled; the
`GetRidOfCultists` ritual-cleanup arm is constants-off; the remix/no-
surface branch of the tower-place retry is constants-off.## .trwld v8 — the banner kill counts (M6-SAVE-1c)

The banner tally now rides the world file. The entry points grew a
`WorldSide { chests, banners }` (rather than more return tuples — new sections
extend the struct) and the v8 tail is a `u32` count, then per banner the id and
the kill count: the C# `BannerSystem.Save` shape (`int32` per banner id,
`BannerSystem.cs:172-186`).

- **The deviations.** The vanilla rides these counts inside the world-flags
  block as a dense array; the port's `BannerSystem` keeps them sparse
  (`BTreeMap` by banner id), so each record carries its id. `Load`
  (`BannerSystem.cs:188-212`) replaces the tally wholesale — the port's
  `load_pairs` inserts the recorded entries, and a recorded **0 stays in the
  map** so a zeroed banner round-trips (the C# keeps only the entries its
  count array carries). `claimableBanners` (the ushort array `:172-186`) is
  not modelled in the port and stays out.
- **The app wiring.** `save_world` collects `banners.pairs()` (ascending
  banner-id order) beside the chests; `App::new` restores the tally through
  `BannerSystem::load_pairs` at construction, so the placed banners' strength
  (`banner_buff_for`) reads the real counts after a reload.
- Tests: `dw_2_3_the_banner_counts_round_trip_in_v8` (zeroed and
  999999999-range counts), `dw_2_3_a_v7_payload_loads_no_banners`,
  `dw_2_3_a_truncated_banner_section_bails`,
  `dw_2_3_the_banner_section_is_deterministic`, and the app pair
  `dw_2_3_a_saved_banner_tally_round_trips_through_the_app` (two NPC
  families' counts survive, with an empty-store control). Suite 2632 / 0.

Out: the remaining C# sections — signs (no port store), the NPC block
(names/homes/homeless, App-side `App.npcs`), the bestiary kill pairs (today
the `.trplayer` sidecar v5; moving them into the world file), `SavedOreTiers`
(worldgen-local — needs a `World` field), the angler quest state (no port
fields), the TownManager rooms, the weighted pressure plates (no port store),
the creative powers/research — each needs its own scope.
surface branch of the tower-place retry is constants-off.
## The lunar battery projectiles (M4-EVENT-16)

The Moon Lord's and the pillars' shots run through six aiStyles, all ported
in `projectile.rs`'s pool dispatch.

- **452 PhantasmalEye (aiStyle 82)** drifts in three legs: 45 ticks, then
  the drift angle negates; 90 more, then `ai[1]` becomes the target player
  index; then the homing — the 14-px target over the 0.6 lerp with the Y
  floor of 6 and an accel that ramps 0.4 → 0 as `ai[2]` ages 0→90, killing
  at 30 px. Every leg's drift rotates `velocity.X` off `ai[1]`, clamps to
  ±6, drifts Y by −0.08 (−0.2 when rising) and floors at −7.
- **454 PhantasmalSphere (aiStyle 83)** fades its alpha by 6 and sets
  `scale = 1 - alpha/255`; while `ai[0] < 30` it locks its position to
  `Main.npc[ai[1]].Center - halfbox - velocity` (the hand's held charge),
  then decays at 0.96 and cycles its 2 frames. The damage suppression
  (`Projectile.cs:15406-15412`) rides the same window.
- **455 PhantasmalDeathray (aiStyle 84)** derives its position from the
  host every tick — the (27, 59) ellipse for the Moon Lord head, (30, 30)
  for the True Eye — scaled by the host's `localAI[1]` open factor and
  rotated by its `localAI[0]` aim. Its heading sweeps by `ai[0]` per tick
  (the ±2π/540 the head's spawn sets), it lives 180 ticks with the
  `sin(t·π/180)·10` scale curve, and its length smooths toward the port's
  tile ray at 0.5 (0.75 when the host can't see its player).
- **456 MoonLeech (aiStyle 85)** runs the outbound leg at the 16-px cap
  toward its player, applies buff 145 (840 classic / 960 expert) once at
  the 20-px contact, and after 330 ticks (or the player's death) flips
  `ai[0]` negative for the return leg that dies at the head.
- **540 StardustMark (aiStyle 103)** fades in over 30 ticks and spawns the
  NPC id carried in `localAI[0]` (the tower's mark: 405/402/407 picked
  from the culled list), then fades out. The port's pool can't grow the NPC
  vec, so the spawn rides `wants_mark_spawn` out to `drain_mark_spawns`.
- **574 NebulaEye (aiStyle 102)** is dragged along its host's frame delta
  while `ai[0] < 180`, homing at the ±0.15/±0.075 accel clamped to 6 px.
- **629 TowerDamageBolt (aiStyle 122)** homes at 5 px toward the host tower
  in `ai[0]` and dies with it; on contact the dispatch spends the tower's
  `ShieldStrengthTower*` (clamped at 0) — the M4-EVENT-7 shield's drain.

Deviations: the pool's spawn carries no `ai[1]` whoAmI, so the machines
link their host by kind (the nearest living MoonLordHead/FreeEye,
MoonLordHand or NebulaFloater); the dust/light/shade tails are unmodelled;
the 452's `Player.FindClosest` folds into the owner's centre; the 456's
expert duration stays classic until the difficulty modes land.## .trwld v11 — the saved ore tiers (M6-SAVE-1f)

`WorldGen.SavedOreTiers` (`WorldGen.cs:3283-3297`) moved onto the World and
into the world file — the four pre-hardmode metals the setup rolls once per
world (`Main.cs:11266-11297`) plus the hardmode trio the altar smashes fill.
Before this, the runtime copy started at the static defaults on every load:
the alternate-metal flips (tin/lead/tungsten/platinum) and any smashed hardmode
tier were lost on reload (the recorded ANALYSIS "save-migration follow-up").

- **The shape.** One presence byte, then the seven `i32` tiers in the
  `SavedOreTiers` field order (copper/iron/silver/gold/cobalt/mythril/
  adamantite), the -1 sentinels included. The C# rides them in the
  world-flags block (`WorldFile.cs:1315-1321` + `:1444-1450`); the port's own
  section, the same values.
- **The plumbing.** `World.ore_tiers` is the per-world owner (the worldgen's
  `ores_and_shinies` pass writes its setup roll there; `caves.rs`'s pass), the
  App reads it back at construction (`app/mod.rs`'s `ore_tiers_seed`), and
  `save_world` writes the App's live tiers. A file written before v11 leaves
  the World at the static defaults — the C#'s own reset arm
  (`WorldFile.cs:796-814`).
- Tests: `dw_2_6_the_ore_tiers_round_trip_in_v11`,
  `dw_2_6_a_v10_payload_loads_default_tiers`,
  `dw_2_6_a_truncated_ore_section_bails`, `dw_2_6_the_ore_section_is_deterministic`,
  and the app pair `dw_2_6_the_ore_tiers_round_trip_through_the_app` /
  `dw_2_6_a_v10_world_starts_at_the_static_defaults`. Suite 2719 / 0.

Out: the remaining C# sections — signs (no port store), the angler quest state
(no port fields), the TownManager rooms, the weighted pressure plates (no port
store), the creative powers/research, the `SaveNPCs` blocks A/C, and the
bestiary's Sights/Chats sub-blocks — each needs its own scope.

expert duration stays classic until the difficulty modes land.

## The Vortex ritual projectiles (M4-EVENT-17)

The vortex tower's ritual arms (the port's `update_ancient_cultist`'s
`:39273-39336` machine) throw two markers, both carried by the pool's
aiStyle-108 machine.

- **579 (the portal)** and **578 (the lightning)** share a 32×32 box with
  `tileCollide = false` and `ignoreWater`. The marker's timeline
  (`Projectile.cs:33679-33988`) is `ai[0] <= 50` (the wind-up, dust only),
  `<= 90` (the charge: `scale = (ai[0] - 50)/40`, `alpha = 255 -
  255·scale`, `rotation -= π/20`), `<= 120` (the full charge: `scale = 1`,
  `alpha = 0`, `rotation -= π/60`) and the dissolve (`scale = 1 -
  (ai[0] - 120)/60`, `alpha = 255 - 255·scale`, `rotation -= π/30`, dying
  at `alpha >= 255`). The 578's first tick stores the aim rotation
  (`Player.FindClosest`'s centre → `ai[1]`, the zero fallback `UnitY`).
- **At the 90th tick** the 578 spawns the **580 VortexLightning** — the aim
  vector × 8 behind the centre, damage 50 (`:33841`'s call), knockback 1,
  `ai0 = ai[1]` (the bolt's spin) and `ai1 = rand(80)` (the wand's seed);
  the 579 spawns **two VortexAliens (427)** at its centre with
  `-UnitY.RotatedByRandom(2π) * rand(4, 9) - UnitY * 2`.
- **580 (aiStyle 88, 14×14, `extraUpdates = 4`, `ignoreWater`)** runs the
  ritual bolt: the `localAI[0] == 0 && ai[0] >= 900` wrap latch
  (`ai[0] -= 1000`, `localAI[0] = -1`), then the moving arm integrates the
  velocity and stops it on a solid tile (`:32308`), the stopped arm dies
  after `extraUpdates * 2` ticks (`:32156-32174`) and the moving arm's
  wand (`:32195-32249`) draws the per-step lateral jitter with the
  100-roll rejection (`Y <= -0.02`, `|X| <= 40`) and accumulates
  `localAI[0]`.
- The resolution seam: the projectile can't grow the NPC vec, so the 579's
  aliens ride the M4-EVENT-16 intent (`wants_mark_spawn`, now carrying
  the (kind, x, y, vx, vy) payload) out to the world_sim's
  `drain_mark_spawns`; the 580's spawn rides `wants_ritual_bolt`.

Deviations: the dust arms (229/240/5 and the `ai[1] == 1f` burst's 25
dusts), the sounds and the light are unmodelled; the wand's
`UnifiedRandom((int)ai[1])` reseed folds into the shared stream; the
late-stage clone spawn (`:32251-32257`) is unmodelled.

## SpreadGrass's conversion arms (M5-SPREAD-2)

`WorldGen.SpreadGrass`'s last unported arms landed (`WorldGen.cs:75207-75221`):
the `flag2 = dirt > 0 && TileID.Sets.Conversion.Grass[dirt]` dispatch (the set
is 2/23/199/109/477/492, `TileID.cs:86`) and `TryConvertingOrKillingTreesAboveIfTheyWouldBecomeInvalid`
(`:54802-54860`).

- **The dispatch.** A conversion-grass dirt spreads through the flavor's map
  instead of the plain type write: corruption for 23/661 (`Convert(i, j, 1,
  tiles: true, walls: false)`), crimson for 199/662 (type 4), hallow for 109
  (type 2). The port's new `spread::convert_cell_flavor` is that single-cell
  Convert arm over the port's three conversion tables; `walls: false` is the
  caller's flag (the SpreadGrass calls never convert walls).
- **The tree arm.** A forest tree (5) standing on the spread cell survives
  only a grass/jungle/snow/mushroom ground (the C#'s `Conversion.Grass` +
  `JungleGrass` + `Snow` + `MushroomGrass` sets — the port's `SET_GRASS` plus
  the literal members 60/661/662/147/70), a palm (323) only sand
  (`SET_CONVERSION_SAND`), and the profile trees (the gem 583-589 / sakura
  596 / willow 616 / ash 634) their `GroundTest`'s kind. An invalid ground
  kills the tree (the port clears the cell). The 596/616
  `ConvertTreeAndGround` re-grounding arm is the port's Out — the tree-
  conversion machinery isn't ported.
- Tests: `spread_grass_2_the_conversion_dispatch_and_the_tree_kill`
  (jungle keeps the tree, sand kills it, the plain write lands) and
  `spread_grass_2_the_conversion_arm_converts_through_the_flavor` (the
  corruption map over a conversion-grass dirt, the walls untouched).
  Suite 2759 / 0.

Out: the C#'s `NetMessage.SendTileSquare(-1, i, j, 3)` (no MP netcode on this
pass), the `UseBlockColors(TileColorCache)` arm, the neighborhood recursion
(`repeat` with the `grassSpread < 1000` cap — the port's world-wide growth
loop is its equivalent, the cap recorded here), and the vines/moss arms
(`GrowMoreVines`, `SpreadGrassWalls`, `MossConversion`) still open.
  party, the Constant's starving/darkness engine, skyblock.

## Micro seeds (M5-SEED-2)

The 1.4.5 modular registry (`WorldGen.SecretSeed`) and the arms that landed
on `M5-SEED-2-micro-seeds`.

### The registry and the hash

**There are 37 micro-seeds, not 38** — `WorldGen.cs:341-413` declares 37
fields and the static ctor `:3022-3058` is 37 `SecretSeed.Register(...)`
lines; the "38" in the earlier notes is `AllSecretSeeds` (`:339`) or the
`Register` method definition (`:483`). The port's `MICRO_SEED_ORDER`
carries all 37 in registration order (which is also the `AllSecretSeeds`
walk order, so a future first-match tie-break matches).

Recognition is the hash, not a word list: `SecretSeed.Check`
(`WorldGen.cs:468-481`) normalises the text (`[^a-z0-9]`, lower-cased),
hashes it and compares to the stored code. `Utilities/Secrets.cs:16-26`:
`CryptRaw(utf8, salt, 4)` (BCrypt cost 4, the game's static salt decoded
from `"fT2JQQzNMJl2NRoMbo9RjA=="` = `7d3d89410ccd30997635 51a0c6e8f518c`),
then 1000 rounds of the deterministic `Utils.Swap` shuffle, then a second
`CryptRaw`, then standard Base64. The port is
`crates/tr-core/src/bcrypt.rs` — the full `CryptRaw` (the pi-derived
P/S tables transcribed mechanically from `BCrypt.cs:186-296`,
`Encipher`/`StreamToWord`/`Key`/`EKSKey`/`CryptRaw` verbatim).

Two things about the hash that cost the most debugging time and are now
pinned by tests:

1. **The additions wrap at 32 bits twice.** The C#'s
   `((S0 + S1) ^ S2) + S3` is unchecked u32 arithmetic; a u64 intermediate
   carries past the first overflow and every later byte diverges.
2. **`EKSKey`'s salt offset is a single counter across BOTH schedules**
   (`BCrypt.cs:110-133`) — resetting it before the S loop flips the state.
   (`Key` also had a duplicated P-encipher loop in the first draft.)

The golden vectors prove the semantics: `to_secret("pumpkinseason") ==
"Z4Odmvd5lScy/KGXHUO2nvqA9l3KRvm8"` (halloweenGen's registered code —
the word the game itself reveals in `SetSeedToRandomWithCurrentEvents`,
`WorldFileData.cs:409`) and `to_secret("arachnophobia") ==
"SPlOdka0fv8wUovao6u3VB7ZS+IbcPDu"` (noSpiderCaves). Those also prove the
port's key walk is the game's non-NUL-terminated one: the stock `bcrypt`
crate treats the key as NUL-terminated and would differ.

### Storage and the wire

`SeedFlags` (a `u16` mirror of `serializedSeedsSum`) is left byte-identical;
the 37 flags live in a new `MicroSeedFlags` (u64) beside it. They ride
`World.micro_seeds` / `GenOptions.micro_flags`, TRWLD **v7** (the u64
appended after the v6 u16; a v6 file loads as a classic world) and the
`.wld` seed text — the C# writes **no** micro-seed bytes at all
(`SaveWorldFlags`, `WorldFile.cs:1238-1250`), and recovers them from the
text at load (`WorldFileData.SetSeed` + `CheckInputForSecretSeed`), which
is what `wld.rs` now does instead of discarding the string. The port
cannot reproduce `GetFullSeedText`'s `size.difficulty.evil.sum.` prefix
form (no seed text is persisted) — recorded as the one deviation.

`InitializeSecretSeeds` (`WorldGen.cs:553-587`) maps eight of the 37 onto
`Main.*` statics (vampirism → `vampireSeed`, endlessHalloween →
`forceHalloweenForever`, …). The port reads those eight directly from the
flags at their use sites instead of a static pass, and the worldgen arms
read `MicroSeedFlags` directly — the net effect for each arm is the same
flag value.

### The arms

Ported (all gated, classic worlds byte-identical): `DoDigExtraHoles` and
the `surfaceIsInSpace` terrain parameters (the vanilla Grass slot,
`WorldGen.cs:12423`); `extraLivingTrees`
(`:15494-15623` + `extraLivingTreesReducedAmount`),
`extraFloatingIslands` (`:12942-13119` + its two Variations),
`biggerAbandonedHouses` (`:433-447`), `noSpiderCaves`' pass skip and the
`ILied` wall sweep (`:17358`, `:2618-2630`); the surface conversions
`DoSurfaceIsDesert` (+ Finish), `DoSurfaceIsInSpace`,
`DoSurfaceIsMushrooms`, `DoWorldIsFrozen`, `DoNoInfection`,
`DoHallowOnSurface`, `DoWorldIsInfected`, `DoStartInHardmode` and
`DoPooEverywhere` — the three Convert-driven ones go through
`spread.rs`'s opened `convert_tile`/`convert_wall`/`convert_tile_hallow`/
`convert_wall_hallow` (the port's own conversion tables, applied per cell
with the C#'s per-column type decision, including the drunk
`GenVars.crimsonLeft` half split); the `actuallyNoTraps` kill and
treasure-ring sweeps; the `paintEverythingGray`/`Negative` and
`coatEverythingEcho`/`Illuminant` sweeps with all nine `Variations`
predicates ported (`WorldGen.cs:43-337`, the count gates reading
`active_secret_seed_count`); `DoRainbowStuff`'s kinds; and the runtime
arms (the endless-holiday loot + spawner arms, vampirism's shop/pot/
regen/Zoologist, `halloweenGen`'s `PlacePumpkin`/`GrowPumpkin`/surface
body, and the `SpawnStormLightning` gates).

### Deferred families (each a whole subsystem)

- `noSurface` (`DoNoSurface` + `DoNoSurfaceFillTheTop` + the ~25 pass
  gates + the missing SpawnPoint/RandomizeWorldSpawn family);
- `roundLandmasses` / `errorWorld` (`DoRoundLandMasses`,
  `DoErrorWorldShuffleBlocks`, the chest half, the missing shimmer-tile
  table);
- `extraLiquid` (`DoExtraLiquidAddLiquid`/`AddBubbleBlocks`/`Finish` — the
  port has no worldgen `QuickWater` settle);
- `dualDungeons` (a second dungeon generator + the bounds system ~35 sites
  read — overlaps M5-DUNGEON);
- `teamBasedSpawns` (`ExtraSpawnPointManager`), `randomSpawn`
  (`RandomizeWorldSpawn`/`Spawn_IsAreaValidSpawn`), `addTeleporters`
  (`TeleporterGenerator`), `graveyardBloodmoonStart` (the port has no
  graveyard biome);
- the chest/NPC rows inside the ported passes (the port generates chests
  as tiles without inventories: echo's 1-in-25 item 4409, the frozen
  finish's Present 1869 + the Guide→Santa swap, `DoPortalGunInChests`);
- `rainsForAYear`'s world side (`Main.rainTime`/`cloudAlpha` live in
  tr-app's `Rain`, which the worldgen cannot reach);
- the `DoSurfaceIsMushrooms` `Convert(k, l, 3, 0)` arms (no mushroom
  conversion table in the port — the direct-equivalent writes are ported);
- `DoRainbowStuff`'s frame re-writes and the thunder-bolt projectile arms
  of `moreLightning`/`noLightning` (`Projectile.cs:12776/14823/39871`).
 2397c90 (M5-SEED-2: the docs — the ROADMAP row, the ANALYSIS section and the PORT_MAP rows)

## Yoyo on-hit effects (M4-PROJ-4b)

The yoyo family's hit-damage and hit-spawn arms, ported into the pool's
`damage_pve`.

### The shared counter

`Player.yoyoHitCounter` (`Player.cs:1220`'s `-1` start) lives on the port's
`Player` with `every_some_yoyo_hits(amount, countUp)` — the C#'s
`Player.EverySomeYoyoHits` (`Player.cs:4308-4316`). Three properties that
shaped the whole task:

1. **It is never reset.** No `ResetEffects` arm, no death/throw reset, no
   per-projectile reset — the counter is monotonic for the life of the
   `Player` object (the port therefore writes no reset code either).
2. **It is per-player, not per-projectile.** Every yoyo the player owns
   shares it.
3. **The -1 start shifts the schedule.** `-1 % 7 == -1` (C#'s truncated
   remainder, and Rust's `i32 %` matches it), so the first hit never fires
   — and the counter's *0* makes the **second** hit fire for every period.
   The doubling and the spawn therefore land on hits 2, 9, 16, … (555) and
   2, 5, 8, … (546), not 7/3.

### The hook order

The C# evaluates the doublings **before** `StrikeNPC`
(`Projectile.cs:13389-13396`) and the spawns **after** it
(`:13707-13809`) — and both read the same counter value on one hit. The
port folds that into `damage_pve`'s per-NPC loop: the doubling peeks
(`countUp: false`) into the `dmg` local just before
`hurt_from_projectile`, and `yoyo_on_hit` increments inside the `landed`
block. Getting these in one place is what keeps the periods aligned.

### The arms

| yoyo | effect | detail |
|---|---|---|
| 555 (TheEyeOfCthulhu) | `dmg * 2` every 7th hit | pre-strike peek |
| 546 (Chik) | `dmg * 2` every 3rd hit | pre-strike peek |
| 545 (Cascade) | spawns 1125 every 2nd hit | `damage * 2`, the `ai1` floor `dist/200` with the C#'s 208-seeded scan degenerate, the `localNPCImmunity` **-1** sentinel, the `ai[0] != -2` dropped-yoyo guard |
| 554 (Kraken) | spawns 3× 1135 every 4th | `damage * 0.5`, `l ∈ {-2, 0, 2}` at ±0.05 turns |
| 546 (Chik) | spawns 5× 1134 every 3rd | `ai0 = 1` for the four followers, the idStatic-20 stamp |
| 555 (TheEyeOfCthulhu) | spawns 1130 every 7th | `damage * 2`, 14-tile initial speed, `ai1` floor 80/200, the -1 sentinel |

The four shots carry their `SetDefaults` rows verbatim (1125:
32×32/aiStyle-189/`penetrate = 4`/local-immunity −1; 1130:
100×100/aiStyle-210/`timeLeft = 600`/local −1; 1134:
6×6/aiStyle-24/`penetrate = 1`/idStatic-20; 1135:
100×100/aiStyle-210/idStatic-10), and `AI_210_YoyoShots` ports the shared
fade lifecycle of 1130/1135: the 0.92 velocity decay, the
`MultiLerp`-driven alpha ramp (255→0 over 20 ticks) and the 20-tick
owner-local `Kill`.

Deviations: the counterweight on-hit spawn path (`Player.Counterweight`,
`Player.cs:12356-12415` — the yoyo-glove/counterweight scan) is a separate
accessory arm and stays deferred; `AI_210`'s one-shot sound and frame cycle
are the render row's (the port's cue table has no `Item20`); the spawned
shots' own aiStyle-189 (1125) and aiStyle-24 (1134) families are their own
tasks — the shots exist with their data, their children-AIs do not run.
## The Old One's Army's lane spawn (M4-EVENT-18)

The etherian portal (549) is the gate every lane's monster climbs out of,
and its pick is the whole wave's spawn table — three difficulties, each a
per-wave `switch` with its own caps and rolls.

- **The gate's machine** (`AI()`'s aiStyle-106 block, `NPC.cs:40970-41047`)
  is a counter plus an idle timer. `ai[0]++` runs only while
  `!EnemySpawningIsOnHold` (the event's `_timeLeftUntilSpawningBegins`), and
  at `ai[0] >= LaneSpawnRate` the gate asks for a spawn; the 3x reset is
  written _before_ the modulo test, so the reset frame's `0 % rate == 0`
  opens an attempt too. `localAI[0]` caps at 180, and at 180 the arm either
  returns (with `dontTakeDamage = true`) while `NPC.AnyNPCs(548)` — the
  etherian crystal — stands, or flips `ai[1]`, zeroes `ai[0]` and enters the
  fade: `scale = Lerp(1, 0.05, GetLerpValue(500, 600, ai[0]))`, dying at
  `ai[0] >= 550`. The lane side rides `ai[2]` (`leftGate = ai[2] == 0`), and
  `LaneSpawnRate` is the wave's own rate — the tier-1 table's
  90/60/55/50/40, the tier-2's 90/70/60/55/50/45/42 and the tier-3's
  85/75/60/60/55/60/90.
- **The picks are three separate functions**
  (`Difficulty_1/2/3_SpawnMonsterFromGate`). Two traps mattered when porting
  them. First, the C#'s short-circuit: a branch whose _first_ condition is a
  cap check does not draw its `Main.rand.Next` at all when the cap is full,
  so the port's roll order has to match condition for condition — several
  arms had the roll hoisted ahead of the cap. Second, the `num`/`num8`/
  `num9` pair: the squire branch always spawns the squire and _also_ the
  second body on a roll (the 556 beside the 553, the 557 beside the 554),
  which `LaneSpawn` carries as `first`/`second`.
- **The bosses each have their own shape.** The Dark Mage (564, tier-1 wave
  5) rides a separate `if ((!leftGate || Next(2) == 0) && kills > required ·
  0.5 && !AnyNPCs(564))` — but the `if/else if` chain that follows it
  re-assigns `num4` whenever it fires, so the boss only survives when the
  goblins already sit at their cap. The Ogre (576, tier-2 wave 7) heads the
  chain instead. Tier-3 wave 6 runs _two independent chains_: the second
  (`:1708`) always executes and overwrites the first's pick.
- **The resolve** lives in the sim, which owns the NPC vec and therefore the
  live pool the caps count (`NPC.CountNPCS`). `FindProperDifficulty` picks
  the tier from `hardMode` / `downedMechBossAny` / `downedGolem`,
  `GetInvasionStatus` supplies the kill pair, and the pick's kinds spawn at
  the gate's bottom. The port's DD2 roster is partial — the wyvern, the
  archers, the drakes, the mystics, the ogres, the hounds, the sapper and
  the glider have no kind yet, so those picks drop out until they land
  (`nid_of`/`dd2_pool_id`).

## Yoyo string + sprite (M4-PROJ-4c)

The row's pure half is `crates/tr-app/src/yoyo_string.rs`; the draw call is
`Projectile::draw_with_hand` (frame.rs hands it the player's centre).

### The colour chain

`Main.TryApplyingPlayerStringColor` (`Main.cs:35474-35519`) is ported as
`string_color(string_color_id, disco, rainbow)` → RGBA 0..1:

- the paint path (`WorldGen.paintColor` — the port's
  `tr_core::tile_light::paint_color`) with the **75-per-channel floor**
  (`:35479-35490`), which also lifts the explicit dark rows;
- the special ids 13 (20,20,20), 0/14 (200,200,200), 28 (163,116,91),
  30 (226,228,142), 27 (the animated `Main.DiscoR/G/B` — the caller's
  per-frame triple) and 29 (the rainbow: the `(mouseTextColor-190)/65` lerp
  with the `±40` channel jitter, `:35511-35516`);
- the 0.4 alpha (`:35410`); the id-0 path's alpha comes from the caller's
  `:28038` instead, so both end at 102/255.

The item→colour table is `Item.cs:30220-30255` (3293→1 … 3304→12, 3305→28,
3306→14, 3307→27, 3308→13) plus `Item.cs:43716`'s 30 — `string_color_for_item`.

### The walk

`string_segments(hand, target, sag_factor)` is the 12-px walk of
`Main.cs:27966-28046`: the remaining-length step (with the partial final
step), the rotation `Atan2(dy,dx) - π/2` (`:28036`), the sag bow scaled by
the remaining distance (`:27995-28035` — the speed curvature is an input
the caller folds in), and the per-segment light tint (`segment_color`'s
0.5 light multiply + the `num10` half dim, `:28043`/`:28045`).

The draw path emits one `push_sprite_origin_rot` per segment
(`render/sprite.rs`'s `origin`-anchor analogue of
`EntitySpriteDraw(origin: (W*0.5, 0))`, `:28042-28045`) and then the yoyo
sprite quad centred and rotated by the slot's `rot`/`scale`/`alpha`, with
the `FishingLine` sheet's 12-px slice as the source rectangle
(`new Rectangle(0, 0, Width, (int)num5)`).

### Deviations

- `player.stringColor` rides M3-EQUIP-3 (the accessory system): the row
  ships the id-0 white path (`A=102`) with the table ready, exactly as the
  AI row shipped `meleeSpeed = 1.0`.
- `drawLayer` 7's player-pass interleave (`Main.CacheProjDraws` +
  `PlayerDrawLayers.DrawHeldProj`, `Main.cs:22196-22202`,
  `PlayerDrawLayers.cs:4185-4235`) has no port analogue — the port has one
  projectile loop; the counterweight-before-yoyo order is encoded by the
  loop's push order, and the layer-8 counterweight-over-arm case is not
  modelled.
- `Lighting.GetColor`'s alpha behaviour (the XNA `Color(Vector3)` drops to
  255) is not reproduced: the port's `LightMap` is RGB-only and the row's
  alpha chain is explicit (documented in the module doc).
- The drawn shots' frame/sound arms and the counterweight's own string are
  the render rows' scope; `ApplicationItemPositionOffsetFromMount` is a
  mount row's input (the port has no mounts yet).

## Grappling hooks (M4-PROJ-5)

The hook family landed as a pool aiStyle-7 projectile: `grapple.rs` holds the
state machine and the tables, `ProjectilePool::update` drives it, and
PHYS-6's single-hook stand-in is retired. All 32 vanilla kinds carry a
`proj_data` row (18x18 for the Hook through 6x6 for the Skeletron Hand,
20x16 for the Worm Hook, `timeLeft *= 10` = 36000, `extraUpdates` 1 for 256
and 3 for 935).

- **The update order is the source's own.** `Main` runs players before
  projectiles, so `GrappleMovement` reads `grappling[]` written by the
  *previous* tick's pass and `Player.Update` clears it at its end
  (`Player.cs:28300-28301`). The port clears at the top of the projectile
  pass — the same one-tick latency, preserved rather than optimized away.
- **The latch scan is the source's box, not a tile under the tip.** The
  10x10 box at `Center - (5,5)` over the x-major range `[tile(Center-21),
  tile(Center+37)]`, first match wins; the position pins to
  `GetTileVisualHitbox`'s center (`:66577-66595`) and stops integrating.
- **Two fire-time tables disagree and are ported verbatim.** `QuickGrapple`'s
  `num6`/`num7` (in-flight cap and the recycle) and `AI_007`'s `num14` attach
  cap differ for 73/74, 652 and 372; and 73/74 are not symmetric with each
  other (only `num == 73` is special-cased at `:6369-6372`, so 74 keeps the
  default budget while both share the attach cap of 1).
- **The pull averages every registered hook** and takes its speed cap from
  `grappling[0]`'s type (`:22832-22851`), as the source does — a mixed set
  uses the first hook's cap, not each hook's own.
- **The Static Hook hangs the player still with no input held.** With no
  direction its contribution is the player's own center, so the pull
  collapses; the 6 px slack only engages once a direction is held.
- **The cracked-brick 1/16 break rides the hook's own `u64` stream.** The
  C# draws `Main.rand.Next(16) == 0`, whose interleaving with the other
  consumers of that global stream is not replayable in this port. The hook
  therefore owns a per-consumer stream seeded from the world seed beside
  `ambience_rng` (the house pattern) instead of sharing the item stream —
  a documented deviation, chosen so the roll is deterministic and testable.
- **PHYS-6's four wrong constants are corrected, and its tests untouched.**
  The max distance for 256 (425) and 935 (500) and the pull caps for 487
  (12) and the Lunar four (16) had been inherited as the defaults; PHYS-6's
  grapple tests pin type 13 (whose values were and stay right) and are left
  as the regression anchor, with the new values asserted separately.

Deferred to their own rows: the chain/sprite rendering and the `light` /
`projFrames[652]` animation (nothing in `render/` reads `proj_data`), the
item-driven fire (`QuickGrapple_GetItemToUse` + `ItemCheck_TryStartUse`),
the latch blacklist (`UpdateBlacklistedTilesForGrappling`),
`DoQueenSlimeHookTeleport`, the TrackHook→minecart transfer, and the net
messages (13/17/29).
## Thorn bushes + the wall-brick conversion tables (M5-SPREAD-2c)

The M5-SPREAD-2 row's deferred "next slice".

### The thorns

`TileID.Sets.SpreadOverground` (`TileID.cs:621`) is the gate the port's
overground arm reads; it was missing the thorn tiles 32/352 and the
golf-grass 477/492, so the arm the C# triggers on the *thorn* itself was
unreachable. With the set fixed, `:72362-72370`'s arms land: the corruption
thorn (32) grows into the corrupt grass 23 and the crimson thorn (352) into
the crimson grass 199, each at `genRand.Next(3)` — drawn only for a 32/352
cell — plus the jungle `:73658-73661`'s thorn-69 arm (no roll). The
underground twin (`:73280-73294`) stays dead code in both the C# and the
port (`Sets.SpreadUnderground` excludes 32/352).

`grow_spike` carried two deviations, both fixed:

- the "converted grass" map's sentinel was `0` (the C# `-1`); it is now
  `u16::MAX`, which is what the knife-edge `t.kind != converted` compare
  needs;
- the write replaced the whole `Tile`, zeroing the frames and the liquid;
  the C# (`:45476-45481`) sets the kind and the active bit on the existing
  tile, `CopyPaintAndCoating`s from the seed tile and runs
  `SquareTileFrame` — the port now does all three.

### The wall chains

`convert_wall_by_case` (growth.rs) was the three evil types' flat
Grass/Stone/HardenedSand/Sandstone/NewWall ladder. It now carries every
conversion type's wall arm in the C#'s order: the missing `Dirt`/`Snow`/`Ice`
sets (`WallID.cs:39-41`), the glowing-mushroom set (`WallID.cs:88`), and the
0/3/5/6/7/11/12 arms — cleanse's grass-family `Next(10)` surface roll and
262/274/292-295 switch, desert's two set unions (187/216), snow's (71/40),
purify's natural-wall 349 plus its dirt/NewWall arms, jungle-mud's variant
of the same, and corruption-II's four separate NewWall guards followed by
the ebonstone fallback. `spread::convert_wall` routes through it, so the
evil callers now get the C#'s chain: the stone wall 1 converts to 3/83 (the
port's old natural-slice 3/192 was the deviation) and the dirt wall's arm
lands on the cleanse types.

### Deviations

- The torch arms (`TileID.Sets.Conversion.Torch`) of every conversion type
  are still deferred: one shared helper would cover them but they need the
  frame-x/y predicate.
- `SpecialSeedFeatures.RainbowSandAndBlackSandWalls` (the paint reset inside
  `Convert_ActuallyConvertWall`, `:56329-56343`) is not modelled — it is a
  secret-seed runtime flag.
- The `NetMessage.SendTileSquare` broadcast is not ported (single-player).
## The Old One's Army's tier-2/3 families (M4-EVENT-19)

The lane spawn table's unported members landed: the 13 kinds the D2/D3
rosters pick (the port ids 169-181), each with the vanilla row and its
machine arm.

- **The diving flyers** (`AI_108_DivingFlyer`, `NPC.cs:65392-65788`) are a
  new machine in the port — the etherian wyverns (558/559/560) and the
  kobold gliders (574/575). It is a five-state machine on `ai[0]`: the
  approach (the 500-px engage gate with the `num4 = 90` smoothing toward a
  point `num2 = 400` above the player and the `ai[3]` blocked-tick
  counter), the wind-up (20 ticks for the wyverns, 30 for the gliders, at
  0.95 damping with the gliders' `-0.1` drift), the dive (the `num11`
  homing with the `num12 × num11` drag — 3.0 for the wyverns, 0.5 for the
  gliders — and the 30-tick / 150-px / min-speed exits), the 45-tick
  post-dive rest and the gliders' impact. The families differ by flags not
  by numbers: the wyverns' `flag2 = false` keeps them out of the crash
  state entirely, while the gliders' `num13 = 0` means their only dive
  exits are the solid-collision and the 64-px proximity, both of which open
  the 192-box impact — an 80-damage blast, the `alpha = 255` full-box
  flash and the death at the third tick (the port's `wants_blast` intent +
  `damage_override`, read by the contact path).
- **The walkers** ride the port's `AI_107` arms: the Wither Beast T3 (569)
  is the T2's aura/self-heal — it charges at `num5 = 110` frames with the
  1.4/0.16/0.7 walk tuning and heals `lifeMax / 20` every 60 ticks; the
  Drakins (570/571) run the flag12 stand-and-shoot, where `num8 = 60`
  opens the burst, `num9` re-writes itself every 6 (571) or 9 (570) ticks
  between 10 and 40 so they fire repeatedly, and the 671 leaves from the
  22-px muzzle with the `Math.Abs(v.X) * num15` ballistic drop at 35/60;
  the Kobold walkers (572/573) are the ignite-and-explode kamikaze — the
  lit state sets `ai[1] = 1`, the 192-box blast follows on the third tick
  of `ai[1] == 2`, and the sprint's `num23 = 4` (four times the 0.88 walk)
  rides the flag5 charge arm; the Ogres (576/577) carry all three attack
  arms at once — the flag5 chase (`velocity.X *= 0.7`, the 60-tick
  cooldown), the flag3 throw (the 676 spit at 40 from the offset, the 683
  ground pound at 60) and the flag4 hop (`velocity.Y = -13` at `num = 56`),
  with `ai[1]`'s sub-state machine picking chase vs spit vs pound on the
  distance and the 300-tick `localAI[0]` chase latch.
- **The ports' roster is now complete** for the spawn tables: `nid_of` and
  `dd2_pool_id` round-trip the D1/D2/D3 rosters (558-577, 548-551), so the
  lane gate's pick resolves to a real kind and the pool's live caps count
  them. The remaining DD2 families (the Lightning Bug 580-family and the
  Old One's Skeleton's 566/567, already ported) sit outside the spawn
  tables.


## The moss spread arm (M5-SPREAD-2, remaining scope)

`UpdateWorld_OvergroundTile`'s moss branch (`WorldGen.cs:72371-72424`) is the
last arm of the M5-SPREAD-2 row: a moss-family tile (`Main.tileMoss` =
179/180/181/182/183/381/534/536/539/625/627, `Main.cs:7007-7032` — the port's
`TILE_MOSS`; or `tileMossBrick` = 512/513/514/515/516/517/535/537/540/626/628)
with a `NextDouble() < 0.5` roll converts the dirt/pearlstone neighbours in its
clamped box through `WorldGen.MossConversion` (`:71457-71503`): a moss-brick
over pearlstone 38 or a moss over dirt 1 keeps itself (the guards), the family
pairs (182↔515, 180↔513, 179↔512, 381↔517, 534↔535, 536↔537, 539↔540, 625↔626,
627↔628, 183↔516, 181↔514) map when both guards miss, and 0 skips the cell.
Each conversion re-frames its cell (`SquareTileFrame`), and a `Next(6)` roll
sprays the loose moss 184 into a free neighbour.

- Tests: `moss_conversion_3_the_map_and_its_guards` (the two guards beating the
  map, the family pairs, the unmapped 0) and
  `moss_conversion_3_the_moss_arm_converts_the_neighbours` (the seeded arm
  spreads the moss onto a dirt neighbour). Suite 3110 / 0.
- Out: the netcode (`NetMessage.SendTileSquare`), the paint/coating copy (the
  port's growth writes carry no color cache) and `SpreadGrass`'s `color`
  argument; `SpreadGrassWalls`/`SpreadDesertWalls` landed earlier in the row.

## The chest style main items (M5-CHEST-gen)

The `num10` (slot-0 main item) dispatcher of `AddBuriedChest`
(`WorldGen.cs:35892-36205`) and the caller-side mains that pre-set it landed
2026-11-10 as the `Context` probes over the anchor's surroundings:

- **The `Pyramid` structure's own chest** (`WorldGen.cs:27853-27873`): the
  caller pre-sets the main with the `Next(3)` pick and a second `Next(3)` on
  the zero, so the Pharaoh'sMask 848 lands only on two zeros (1/9) and the
  SandstorminaBottle 857 / FlyingCarpet 934 otherwise; `Main.tenthAnniversaryWorld`'s
  `0 → 1` rides M5-SEED-1. `flag11`'s companion row writes the
  Pharaoh'sRobe 866 into slot 1 right after the mask (`:36234-36239`) — the
  only family whose main carries a second slot in the same write.
- **`flag3`'s underground-desert arm** (`:36023-36047`): the shallow four
  {4056, 4055, 4262, 4263} or the deep three {4061, 4062, 4276}, split by
  `flag20 = num7 > (desertHiveHigh*3 + desertHiveLow*4)/7`. **Deviation:**
  the hive bounds ride the worldgen-only `GenVars` and are gone by the
  load-time sweep, so the port keeps the `GenVars` reset values
  (`High = maxTilesY`, `Low = 0`) — the deep table below three sevenths of
  the world height. The vanilla's second flag3 disjunct (`chestTileType == 467
  && chestStyle == 10`) overlaps the port's jungle family, which keeps the
  `JungleCycle` main; the desert table rides only the hardened-sand (216)
  probe.
- **The probe split**: the old single `pyramid` flag read both 216 and 187;
  it now splits into `pyramid` (187 — the `Pyramid` structure's own chamber)
  and `desert` (216 — the hive's shell), because the vanilla's two arms write
  different mains and the flag3 extras (`:36814-36823`) belong to the desert
  family alone. The pyramid's chest is NOT a flag3 chest — its main is
  pre-set, so the `num10 != 0` disjuncts all fail.
- **`flag2`'s ice arm** (`:36064-36088`): the seven-item table with the 1/20
  Extractinator 997; the remix's `1319 → 725` swap and the FTW `52`
  overrides stay parked with their seeds.
- **`flag8`'s hell arm** (`:36115-36118`): the world-shuffled
  `GenVars.hellChestItem` (`WorldGen.cs:11246-11258` — the `while (Count > 0)`
  pick-and-remove over {274, 220, 112, 218, 3019}, the remix row swapping the
  112 for the 683) walked once per place with the `hellChest` counter's wrap
  (`:36210-36217`). **Deviation:** the vanilla shuffles the worldgen
  `genRand` at the reset; the port shuffles the sweep's own per-world stream
  as its first consumer, so a world re-loads with the same table (a
  different order than vanilla, which the port cannot share).
- **`IslandHouse`'s sky arm** (`:79344-79363`): the `skyIslandHouseCount`
  cycle 159/65/158/2219 for the first four houses and `Next(4)` after.
  **Deviation:** the vanilla's counter advances once per island house built
  (`:79344`); the port advances once per sky-flagged anchor stocked in the
  scan order.
- **The chain order**: the vanilla's arms are sequential `if`s whose last
  write wins on `num10`, and the water 939 arm is gated on `num10 == 0`
  (`:36197-36200`). The port keeps one `if/else if` chain with the water arm
  last, so the other families' mains win where both probes read true. The
  band rows' draws still run first (the port's pre-existing inversion of the
  vanilla's num10-then-band order) — the stream is the port's own splitmix,
  so the order matters for determinism, not for parity.
- **Parked with their seeds**: the anniversary's flag11 row (848,
  `:36267-36271`), the remix's flag12 `num9` style rows (`:36179-36195`),
  error's chest row, and `GrowLivingTreePassageRoom`'s style-12 main
  (832/4281, `:29303-29306`) — the port's living trees and pyramids place
  plain style-0 chests, so the style-gated arms have no anchor to read.

## The flail family (M4-PROJ-3b)

`AI_015_Flails` (`Projectile.cs:43724-44169`) ports as `ai_flail` in
`crates/tr-app/src/projectile.rs`, driven by the `FLAIL_PARAMS` per-kind
table (`:43761-43823`'s switch — num/num2/num5/num7/num12/num15 over the
defaults 10/24/16/48/15/30, with `num2/4/5/6/7/8/9` scaled by
`1/meleeSpeed`). The ten kinds (25/26/35/63/154/247/757/947/948/1058) ride
`DefaultToFlail` (`:10753-10762`: aiStyle 15, drawLayer 7, friendly,
`penetrate = -1`, melee, `usesLocalNPCImmunity` with cooldown 10) and the
three helpers (928 aiStyle 14 pen 2 timeLeft 250, 248 aiStyle 1, 405
aiStyle 70 pen 1 alpha 255 timeLeft 120) land rows beside them.

The state machine (`ai[0]` 0..6) runs inside one tick per
`ProjectilePool::update`; the C#'s `HandleMovement` peer is the port's
`flail_move` (the face-clamped per-axis advance with the blocked
component zeroed) followed by `ai_flail_bounce`
(`AI_015_HandleMovementCollision` `:43655-43717`), gated on the velocity
change exactly like the C# `flag` (`:17896-17904`) — without the gate the
handler's unconditional state-1 arm would stagger every fly-out tick.
The state-1 bounce's `position -= vector` undoes the whole move; the port
steps the blocked axis 1 px at a time instead of `Collision.Move`'s
sub-steps (agrees to the sub-pixel; the only numeric approximation in the
family). The throw path gains `speed /= meleeSpeed` (`Player.cs:47063`,
×1 at the port's default bonus) and the Flairon's 1 px aim offset
(`:47239-47245`); `Player::melee_speed()` and `Player::mounted_center()`
(`Player.cs:3335`, equal to `center()` with no mount offsets) back the
AI's reads. The `948 → 947` water morph reads the mid-tile liquid kind
(the port's Player liquid-pass idiom) for `wet && !lavaWet`; the map-
fullscreen kill and the `noItems` gate have no port arms (documented).

Hit side: the held state's first 12 ticks deal nothing
(`:12448`), the state-0 hitbox is the 55-px ellipse around the owner's
mounted center with the 0.8/0.4 Y squash (`:15336-15356`, replacing the
box test), the held damage is ×1.2 with the Blue Moon/Sunfury dual ×0.8
(`:14447-14465`), the knockback ×0.35 held / ×0.5 spinning with the hit
direction from the player's center (`:13614-13619`). The atlas §17 note
"damage ×0.35/×0.5" is wrong: those are knockback multipliers (recorded
here so the row can be corrected).

Deviations (all deferred to a decor/render follow-up, none simulated):
`AI_015_Flails_Dust` (`:44399-44474`), `CreateImpactExplosion` +
`CreateImpactExplosion2_FlailTileCollision` (`:69633`/`:69774`), the
bounce's `Collision.HitTiles` ×count (no generic projectile→tile seam in
the port) and `SoundEngine.PlaySound(0, …)` (the AI has no audio
access), the `player.heldProj` / `SetDummyItemTime` / `itemRotation`
writes and `ownerHitCheck` (no consumers), the drawLayer 7 vs the 26/35
`ai[2] == 1` branch (no drawLayer consumer; `ai[2]` is only written by
the sentry-backpack tail, unreachable for flails), and the chain/string
render (cf. the yoyo string, M4-PROJ-4c). The `player.ChangeDir` writes
ride back in `FlailTick.facing` with the same-tick reads served by a
local `eff_dir` (the port's facing is run-key driven). The Flairon's
bubble cadence and jitter (`Main.rand.Next(3, 6)`, `Next(45, 65)`,
`NextDouble`) draw from the pool's rng stream — the item stream by the
yoyo convention. 22 seeded tests cover the data table, the per-kind
rows, the hold orbit, the release/flight/retract/wall-bounce/reel-in
transitions with the per-state factors, the kill gates, the morph, the
Flairon bubbles, the Drippler's extra ball, the damage gates and a full
hold→throw→bounce→reel lifecycle.
## The full `PlayerFrame` chain (M2-DRAW-3, 2026-11-11)

`Player::update_frames` now ports `PlayerFrame()` (`Player.cs:35755-36647`) arm for arm
instead of the three-branch approximation (idle / air / walk) it started with. The chain
order is the vanilla `else if` ladder, and that order is the point: the item-use arm comes
BEFORE the rope/hold/shield/mount/grapple/water/air/walk arms, so a swing no longer leaks
the walk row, and the held-item rows no longer have to be patched in at draw time (the old
`apply_item_use_frame` + `frame.rs` post-pass are gone; the app feeds the same inputs in
through `player::FrameInputs` before the fixed step, beside `swing_aim`).

Legs (`:36219-36291`): the swim paddle (`swimTime > 0`, a flat `+2.0` counter with no
velocity term), the jump/fall/hang row 5 (`velocity.Y != 0 || grappling[0] > -1`) with the
three always-animated wing sets (22/28/45 inside `ShouldDrawWingsThatAreAlwaysAnimated`)
keeping row 0, the walk cycle with the slippery no-input and roller-skate
(`MountID.Sets.IsRollerSkates` = 57-60) rest-row pins and the skates' `num19 = 2f` lock at
`maxRunSpeed`, the rest row, then the Magic-Carpet (`carpetFrame >= 0`) and sandstorm
pins and the post-chain Moon Lord legs (`legs == 140`: `velocity.Y != 0 ? 5 : 0`, row 7
when `Main.OffsetsPlayerHeadgear[bodyRow].Y == 0`, row 0 for wings 22/28) and the sitting
legs (`legs == 217`) row 5.

Body (`:36312-36633`): the full useStyle table (1/0, 7, 2, 11, 9, 6, 3, 4, 8, 12, 13, 5
with the `type == 281/986` carve-out and the `gravDir`-flipped aim rows), the rope pose
(`pulleyDir == 2 ? 1 : 2`), holdStyles 1/2/3/5/7 and 4 (row 3/2/3/3/11/0 with the
`CanVisuallyHoldItem` and dry-torch gates), the raised shield row 10, the mount body row,
the grapple lean (the hooks' centroid vs the body centre → rows 2/4/3), the water float row
10, the swim window (>20 → 0, >10 → 5, else 0), the airborne rows (sliding 3, sandstorm /
carpet 6, EOC dash 6, wings: 22/28/45 → 0 and otherwise 6 rising / 6 under thrust / 5
falling, plain 5), the walk rows (`legs == 140` runs its own `|velocity.X| * 0.5` counter,
everything else copies the leg row after a `|velocity.X| * 1.5` body counter) and the rest
row 0.

The draw side needed no new offsets. The port's part tables were audited against the C#
and already match: `PlayerDrawSet.CreateCompositeData`'s front-arm switch
(`PlayerDrawSet.cs:1952-2023`), the back arm's `+2` rows (`:2034`), the
`compShoulderOverFrontArm` clears at rows 1/2/5 (`:1961,1966,1976`), the torso column-1
override at row 5 (`:1975`), `UpdateCompositeArm`'s stretch columns 7/8 (`:2246-2274`),
`GetCompositeOffset_BackArm (6,2)` / `_FrontArm (-5,0)` applied to BOTH the position and
the origin (`PlayerDrawLayers.cs:4082-4090`, `:1272-1277`) and the
`Main.OffsetsPlayerHeadgear` composite shift (`2 - Y`). The visible breakage was that the
chain fed those tables wrong rows; with the rows correct the existing tables are correct
too. `render/player_layers.rs` now reads the headgear table from `player.rs` instead of
duplicating it, and `every_row_the_frame_chain_can_emit_has_draw_tables` pins all 20 rows.

Deviations, all `TODO`-flagged in the code rather than silently approximated: the inputs
whose granting equipment has no port field yet stay coded but unreachable — Sandstorm in a
Bottle (`sand_storm`), the Magic Carpet (`carpet_frame`), flippers/merman
(`ShouldFloatInWater` returns false), the pulley/rope item (`pulley`) and the raised shield
(`shield_raised`). `Player.sliding` exists as a field nothing sets yet (the slope/ice slide
pose). `Item.noWet` and `happyFunTorchTime` have no item-table columns, so the two held-item
dry gates are always open. `gravDir == -1` arms are coded as the `== 1` value only — the
port has no gravity flip. The armour-set dust/VFX side effects inside `PlayerFrame`
(`:35819-36140`, ~400 lines of per-set particles) stay with the armour tasks.

## The player-audit fixes 11-15 (2026-11-10)

The five S1/S2/S3 rows of `BUG_REPORT.md` closed on one branch:
the branch mechanics below; every C# site is in the report itself.

- **BUG-11** — the in-game options window. `ui/ingame_options.rs` ports
  `Main.DrawSettingButton`'s button (`RefreshSettingsButtonStatus`'s fixed
  `UIScreenWidth - 110` / `UIScreenHeight - 20` rect on the port's fixed
  960×600 UI canvas — the same space the bestiary icon and the emote button
  ride) and the `IngameOptions` panel: the 670×480 origin at `screen/2 -
  size/2` with the 20-px hover padding, the `num5 = 70` row pitch, the three
  volume bars over `sliders::value_bar_rects` (music/sound/ambient with the
  `rightLock` drag), the map-scale/auto-pause/auto-save toggles, and the
  `IngameOptions.Close` close (Escape's first arm, persisting through
  `Settings::save`). Deviations: the vanilla two-column navigation (the left
  column's general/interface/video/controls/language rows) is not mounted —
  the port carries those rows in the front-end `Menu` (M7-UI-4's field
  parity); the volume bars' click-to-set is the slider's drag only.
- **BUG-12** — the Escape chain's map arm. The vanilla order is
  options → bestiary → emotes → map → inventory (`Player.ToggleInv`,
  `Player.cs:4973-4978`). The chain was extracted into
  `App::world_key(KeyCode)` so the winit event path and the tests share one
  body.
- **BUG-13** — the Smart Cursor split. `App.smart_cursor_used` is the
  runtime state (`Main.SmartCursorWanted_Mouse`, default off) and the
  persisted `smart_cursor_toggle` remains the config key it has always
  been — the bug was the setting serving as the runtime gate. The toggle is
  the left-Shift press edge with the `Menu_Tick` cue.
- **BUG-14** — the button strips. `draw_sheet_frame` now takes the frame
  grid and slices `w / frames_x` like `Utils.Frame`, exposed as the pure
  `sheet_frame_uv`; the emote button's sheet was never packed into the
  atlas (the actual root cause of the invisible button — the hitbox and the
  click handler existed, the sprite did not).
- **BUG-15** — the off-by-one. The Solar Selenian is NPC 419
  (`NPCID.cs:11515`), which resolves through `bestiary_sheet_index` to slot
  135 of the 174-entry table; the report's suggested 625 is `SeaTurtle`.
  All literal sheet indices are gone from the code (doc comments keep the
  history).
4051db50 (BUG-11: the in-game options window — the Settings button, the panel and the Escape close)

## The followup caster families (M4-PROJ-6b)

The port's remaining caster AI families land beside the PROJ-6 set (rows,
arms, shoot specials), plus the render pass: the frame state and the
frame-sliced draw, and the trails as the C#'s cheap fallback strips.

Deliberate deviations and skips:
- **The Spectre Wrath 356's spawn stays out** — the kind is item-fired by
  nobody in vanilla; it comes from the Spectre ghost armour set bonus
  (`Projectile.cs:12012-12028`'s `ghostHurt`, spawn `:12060`), which needs
  the M3-EQUIP set-bonus seam. The 59 arm itself landed, so the kind stays
  rowless-but-armed until the set bonus lands.
- **aiStyle 75/84 scoped to the Last Prism pair** — the guns (439/445/460/
  595/600/611/615/630/705/714/735/927) and the remaining hostile lasers
  (461/537/642) keep their out-of-scope status; the 633 holder and the 632
  laser run their full machines.
- **The trails are the `Main.DrawTrail` approximation, not the VertexStrip
  pipeline** — the renderer has no dynamic-geometry strip; the ported strips
  are the C#'s own fallback shape (stretched quads, the verbatim anchor and
  `rot = delta + PI/2` geometry) with the drawers' width curves and
  StripColors per segment. The true strip (the wgsl pass, the CatmullRom
  padding, the TrailCache cycle) is the render row's.
- **The drawers' shader alpha quirks are dropped** (`A /= 2`, `A /= 8`,
  `A = 0`) — they feed the VertexStrip/shaders, and verbatim they blank
  the Rainbow Rod's strip; the end fade alone drives the port's tint.
- **The Nebula arm 618's parent-follow keeps the last-seen parent position
  in `local_ai[0..2]`** — the C# reads `proj.oldPos[1]`, and the port has no
  per-slot history until Phase 7's TrailingMode rings; the arm's accel
  (`sign(delta) * 0.15`, clamp 6) and the parent-death kill are 1:1, with
  one tick of spawn slack recorded.
- **The frame-sliced and trail quads push into the ui range** like the
  ported shuriken path — the C# draws the projectiles in the world layer,
  but the port's world range is retained by the G6 cache
  (`begin_frame_keep_world`) and the yoyo merge's world-range draw sits on
  a latent staleness hole the current `WorldFrameKey` has no projectile
  input for; the unification (and the cache-key widening) is the render
  row's.
- **The `projFrames` port is an in-scope slice** — every animated followup
  kind (206/409/633 + the trail families' 34) plus the default-1 case; the
  overrides outside the ported rows (the source's ~200-entry init) default
  here exactly like the source's own init default.
- Dust/sound spawns stay skipped across the new arms (no dust or sound
  layer in the pool); the seeds' rng draws are consumed where the C# rolls
  them mid-arm, so the ported rolls land at the source's stream positions.## The in-game options window (M7-UI-4)

`IngameOptions.Draw` (`IngameOptions.cs:103-1441`) is the vanilla's in-game
settings window: a 670x480 panel (`:125-129`) split into a **left navigation
column** and a **right page of setting rows**, both animated per row.

**What landed.** The window is now the C#'s two-column shape instead of the
port's flat six-row list:

- the left column is the eight navigation rows — `Lang.menu[114]` General,
  `[210]` Interface, `[63]` Video, `[218]` Controls, `[66]` Controls→keybinds,
  `[131]` Achievements, `[118]` Close Menu and `Lang.inter[35]` Save & Exit
  (`:214-330`). The `Invite Friends` row (`:257-270`) is the social-API arm and
  stays out (AGENTS.md non-goals), so `num7 + 1 = 8` slots;
- the right column carries the four pages as static tables — 17/14/15/15 rows
  (`page()`, the C#'s `:400-1441` sequential `DrawRightSide` calls). Each row
  is a `RowAct`: a header, a value bar, an on/off toggle, a multi-state cycle,
  or a `Parked` row for a vanilla setting the port does not model;
- both columns reproduce `DrawLeftSide`/`DrawRightSide` (`:1457-1504`): the row
  position is `anchor + offset * (1 + i)`, the label is **centred** on it
  (`origin 0.5, 0.5`), the tint is `Color.Lerp(Gray, White, scaleFraction)`
  with **Gold** for the selected category, and the per-row scale animates at
  `0.01`/frame in the left column (`0.7..0.8`, `:157-199`) and at
  `0.001`/frame on the right (`1.0..1.001`, `:333-390`) — including the
  culture shrink for the long translated labels (`:167-185`, `:345-357`:
  ru/pt/pl/fr shrink the left to `0.4..0.44` and pull the right column to the
  2/3 mark; German `0.55..0.6`; the It/Sp and German deltas are
  category-bound);
- the hover pass emits the vanilla's cues — `SoundEngine.PlaySound(12)` on a
  hover change (`:187`, `:191`) and `(10)` on a category click (`:222`) —
  through `App::audio` from the frame loop (`update()`, called beside
  `tick_pointer_clock`), and clears `rightLock` on the button release
  (`:385-388`);
- the value bars drag while locked (`Main.musicVolume = ...`, `:424`) and the
  geometry is the C#'s `valuePosition.X = vector3.X + vector2.X - num3/2 - 20`
  (`:437`) with the label hanging 70 px left (`vector6.X -= num`, `:417`).

**Deviations (all deliberate, each row cited in `page()`):**

1. **Parked rows.** A vanilla row whose state the port has no field for is
   drawn dimmed and inert rather than invented: the UI scale (`:561`), the
   door-opening preference (`:644`), auto-reuse-all (`:677`), the map frame /
   health style / portrait preference / opaque tooltip box (`:751-840`), the
   window mode + resolution (`:876-910`), the cursor/border colour pickers
   (`:1075-1371`). Their settings are the named follow-ups of M7-UI-3
   (equipment/dye), M7-UI-5 (keybinds) and the window backend row.
2. **The video page's slot count.** The C# sets `num11 = 14` for the video
   category but draws 15 rows and compensates with `vector7.Y -= 2f`
   (`:362-365`); the port keeps both verbatim (`right_slots(2) == 14`,
   `right_pitch(2) - 2`).
3. **The wave-quality label.** `:1022-1036` switches on the level with a
   per-level `GameUI.*` key; the port renders `GameUI.WaveQuality` plus the
   level ordinal instead of inventing key names.
4. **Keybinds / Save & Exit.** The C# calls `IngameFancyUI.OpenKeybinds()` and
   `WorldGen.SaveAndQuit()` inline (`:252-256`, `:300-310`); the port records
   the action on `IngameOptions::pending` and closes, because the keybinding
   screen is M7-UI-5 and the quit path belongs to the front-end flow.
5. **Text metrics.** The vanilla measures with the retail font
   (`DrawBorderStringBig`); the port centres with `text::hud_text_width` and
   the font's `line_spacing`, which is the same box within a pixel.

**Verification.** 13 new tests (the slot counts, the two books' pitches and the
`1 + i` anchor, the eight navigation rows, the four pages' row counts, the
scale bands and their clamps, the bar/toggle/cycle round trips, the label
resolution of every row on every page, and the in-panel bounds of every row).
The `TR_OPTIONS=1` pin opens the window at boot for the `TR_SHOT` fixture; the
captured 1600x900 frame shows the title, the eight left rows (the selected
category in gold) at the 42.3 px pitch and the seventeen General rows inside
the panel.
## The Settings button and window redraw (M7-UI-4b)

`M7-UI-4` gave the in-game options window its vanilla *content*; the *chrome*
was still the port's own. Three things were wrong and are now the C#'s.

### The button is text, not a box

`ui/ingame_options.rs` used to draw a 30x30 `Inventory_Back` sheet at a
hard-coded `(850, 580)`, which is the vanilla's spot at 1600x900 and nowhere
else. Vanilla never draws a box at all: `Main.DrawSettingButton`
(`Main.cs:45532-45544`) draws the localised `Lang.inter[62]` ("Settings") in the
**death font** with a four-way black outline, at scale `0.8`, growing to a cap
of `0.96` by `0.02` a frame while the mouse is over it. `App` now stores that
scale (`IngameOptions::button_scale`) and the previous frame's `mouseOver`
latch (`button_mouse_over`), so the ramp starts one frame after the hover — the
C#'s own lag.

The **position** is `Main.RefreshSettingsButtonStatus` (`:45492-45511`), whose
whole point is to keep the word clear of the equipment column:

| condition | effect |
| --- | --- |
| always | `posX = UIScreenWidth - 110`, `posY = UIScreenHeight - 20` |
| `player.dead` | `posX += 40` |
| `PushEquipmentAreaUp` | `posY -= 30` |
| options window **closed** | `num = 174 + Main.mH` (+ the extra-accessory slop), `num2 = num + 47.6 * (8 + extra)`; if `posY - 28 <= num2` then `posX -= 140`, `posY = num2 - 47 + 30`, `_settingsButtonIsPushedToSide = true` |

`settings_button_pos` returns the point and the pushed flag; `button_layout`
resolves it against `measure_button` (the death-font width of the word) and
`ButtonLayout::hit` is the strict `x > pos[0] && x < pos[0] + w && y > pos[1] &&
y < pos[1] + h / 2 - 10` box of `:17018-17026` — note the shaved bottom third.
`app/clicks.rs` tests that box instead of the old constant rect, and
`ui/mod.rs` draws the button from the stored layout both while the inventory is
open and while the window is.

### The panel is the nine-slice

The window's background was a stretched `Inventory_Back` sheet; the vanilla uses
`Utils.DrawInvBG(Main.spriteBatch, new Rectangle(x - 20, y - 20, 670 + 40,
480 + 40), new Color(33, 15, 91, 255) * 0.685f)` (`IngameOptions.cs:127`), i.e.
`Inventory_Back13` cut into nine. Both columns are `DrawInvBG` too, at the
default tint `(63, 65, 151, 255) * 0.785f`. `ui/tooltip::draw_inv_bg` already
ported that helper, so the window and its columns now call it.

The title is `GameUI.SettingsMenu` ("Settings Menu") **centred** on the panel's
top edge — `vector2.X = (screenWidth / 2 - 670 / 2) ... DrawString(...,
vector2 - font.MeasureString(text) / 2 + new Vector2(0, -4), ...)`
(`:132-137`) — where the port had left-aligned it. And the right column's **bar
rows only** are shifted 70 px left (`vector6.X -= num`, `:417`); the port now
applies the shift in `right_row_anchor` and centres `right_row_rect` on the
drawn anchor.

### Three rows that were placeholders

The `General` page drew `UI.DoorOpeningPreference` and `UI.AutoReuseAllOn` as
dimmed, inert `Parked` rows — and `UI.DoorOpeningPreference` is **not a key in
the en-US file at all**, so it rendered as the literal string. The vanilla's
three rows are back to being settings:

| row | C# source | port field | save key |
| --- | --- | --- | --- |
| Smart Doors | `IngameOptions.cs:644-657`, cycling `DoorOpeningHelper.PreferenceSettings` (`DoorOpeningHelper.cs:262-276`: `Disabled -> EnabledForEverything -> EnabledForGamepadOnly`) | `Settings.smart_doors: u8` | `DoorAutoOpeningMode` (`Main.cs:4208`/`:4545`) |
| Hover Controls | `:659-675`, `Player.Settings.CycleHoverControl` (`Player.cs:379-383`); the label is `Hold` only for `ButtonControlMode.Hold`, else `Click` | `Settings.hover_control: u8` | `HoverControlMode` (`:4209`/`:4546`) |
| Autofire | `:677-684`, toggling `Main.SettingsEnabled_AutoReuseAllItems` | `Settings.auto_reuse_all: bool` | `SettingsEnabled_AutoReuseAllItems` (`:4158`/`:4544`) |

That also corrected `General`'s row count: the port had merged the doors and
hover preferences into one row, so it drew 17 rows where the C# draws 18
(`num11 = 17` is the *last index*, exactly like the video page's quirk).

### Other fixes in the same pass

- `GameUI.GameZoom` takes `Math.Round(Main.GameZoomTarget * 100f)`, not the
  bar's fraction, so the row reads "Zoom: 100%" instead of "Zoom: 0%"
  (`:512` vs `:529`).
- `ui/text.rs` grew `ui_font_width` / `death_font_width`, needed by
  `measure_button` and by the title's centring.
- `ui/inventory.rs::extra_accessory_slots()` exposes `EXTRA_ACCESSORY_SLOTS` for
  the button's column-overlap test.

**Deviations.** The port has no `PlayerInput.UsingGamepad`, so the
`EnabledForGamepadOnly` state behaves as plain "smart doors on" would not —
it is stored and named faithfully but nothing reads it yet (the door-opening
helper itself is unported). `HoverControlMode` is likewise stored and named but
unread, since the port's hover control is implicitly "hold". Both are recorded
here rather than silently wired to a guess.

## The PVP toggle + team select cells (M7-UI-7)

`Main.DrawPVPIcons` (`Main.cs:42654-42786`) was ported as a **draw-only** stub:
the toggle was always frame `(false, 0)` and both icon groups had no hit test,
hover text, click or cooldown, so the two controls the user pointed at (the PVP
toggle and the six team cells beside the accessory columns) were decorative.

The port now covers the whole function:

- **Gates.** `ShouldPVPDraw` (`Main.cs:2909`) is the port's `App::is_client()`
  (`Main.netMode == 1`); `ShouldTeamSelectDraw` (`:2911-2919`) is that same
  client gate, since `Main.teamBasedSpawnsSeed` (`:351`) is not ported (no
  team-spawn seeds until the M5-seed rows) and its single-player arm therefore
  reads `false`. `Main.hidePVPAndTeamIcons` (`:2357`) is a new `App` field: the
  C# sets it from the NPC-heads column overflow (`:42046`) and clears it on
  every non-`EquipPage == 1` draw (`:42656-42659`) — the port does not draw the
  NPC-heads column yet, so the flag only ever arrives `false`, but the early
  return and the clear site are ported so the wiring exists when that panel
  lands.
- **Frames.** `PVP_0.Frame(2, 7)` is sliced at `(hostile, team)` (`:42664`,
  `:42688`) and the hovered toggle swaps to the row-`PlayerTeamID.Count` (6)
  cell at full white (`:42696-42700`); the six `PVP_1` cells narrow to
  `rectangle2.Width - 2` and the `PVP_2` ring draws at `Location + (-2, 0)`
  (`:42770-42773`).
- **`teamCooldown`.** `Main.teamCooldown` / `teamCooldownLen = 300`
  (`:965`/`:66781`) are `App::team_cooldown` / `chrome::TEAM_COOLDOWN_LEN`; the
  timer decays once per update (`:17048-17051`) in `frame()` beside
  `tick_pointer_clock`, **not** on the fixed world step — the C# decrements it
  in `Update()`. While it runs both icon groups draw at `White * 0.25`
  (`:42672`) and every hover/click is refused.
- **Hover.** The text keys are the vanilla ones and needed no new
  localization work: `UI.PvpIcon_Active`/`_Inactive` (`:42685`) and
  `UI.TeamIcon{None,Red,Green,Blue,Yellow,Pink}_{Active,Inactive}`
  (`:42734-42758`), all already present in
  `Terraria.Localization.Content.en-US.json:348-360`. They are drawn through a
  new `ui::tooltip::draw_mouse_text` (the single-line
  `MouseTextHackZoom`/`MouseText` shape, same box + shadow + `mouseTextColor`
  tint as the item tooltip). The C# also sets `player.mouseInterface = true`
  while an icon is hovered, which suppresses the slot hover underneath; the
  port mirrors that by skipping the item tooltip when a PVP hit is live.
- **Clicks.** `App::handle_inventory_clicks` routes a press through
  `chrome::pvp_hit` (same geometry as the draw) and applies
  `chrome::pvp_click`'s gate chain: `teamCooldown == 0` for both, plus
  `player.team != i` for a team cell (`:42689`, `:42753`). The effect is
  `teamCooldown = 300` + `SoundEngine.PlaySound(12)` (the ported `Menu_Tick`
  cue, `audio.rs`'s `SFX_CUES`) + the `hostile` flip / `team` write.

**Deviations (all recorded as unported, not approximated):**

- `NetMessage.SendData(30)` (the PVP-state broadcast, `:42694`) and
  `SendData(157)` (the team change, `:42766`) have no counterpart — M6-NET's
  message table carries neither id. The local state change is the whole visible
  effect in single player; the MP broadcast rides M6-NET when those ids land.
- `Player.TeamChangeAllowed()` (`Player.cs:4517-4520`) is a literal
  `return true` in 1.4.5.8, so the `Lang.misc[84]` "cannot change team" refusal
  (`:42757-42759`) is unreachable dead code and is not ported. If a future
  patch gives the method a body, the arm comes back here.
- `Main.teamBasedSpawnsSeed`'s `Spawn(PlayerSpawnContext.TeamSwap)` arm
  (`:42768-42769`) has no port counterpart (no team-based spawn seeds), so a
  team change on a seedless world does not respawn the player.
- `PlayerInput.UsingGamepad` hovers the icons during the cooldown in the C#
  (`:42680`, `:42725`); the port's pointer-driven hit test has no gamepad
  branch, so a cooled-down icon reports no hover for either input.
- The `UILinkPointNavigator.SetPosition(1550..1556, …)` registrations
  (`:42703`, `:42781`) belong to the gamepad link graph, which the port does
  not model.

**Screenshot harness.** `TR_PVP=1` (`env::pvp_icons`) opens the two
multiplayer-only gates so a single-player capture can show the icons: without
it nothing draws, because the vanilla only shows them inside a client. The pin
is resolved once into `App::pvp_icons_forced` (a field, so tests can open the
gate without racing the `OnceLock`). The A/B proof: with the pin the
capture differs from the unpinned one over exactly `(1371, 437)-(1523, 555)`
— the toggle at `(1373, 435)`, the team grid below it and the hover tooltip
box — and two pinned runs are SHA256-equal (`e2cbe1ef…` at 1600x900,
`TR_MOUSE=1380,500`, i.e. team cell 0 hovered).

**Finding (pre-existing, not fixed here).** The older inventory HUD hit tests
in `app/clicks.rs` (the bank transfer buttons, `bank_button_at(mouse, m_h)`)
compute `m_h` as `(screenH - 600) / 2`, while the draw path uses `Main.mH`
(`ui::minimap::main_m_h`, 256 or `screenH - 600`). At 720 px those are 60 and
120, so a live minimap puts those hit regions 60 px off the drawn buttons.
This task added `App::main_m_h()` (the draw path's own value) and used it for
the new PVP icons; converting the older call sites belongs to M7-UI-2, which
owns the inventory click surface.
### BUG-08 — the invasion progress bar (`DrawInvasionProgress`)

- **Where the fade head lives.** The C# runs `Main.invasionProgressDisplayLeft`'
  decay and the `invasionProgressAlpha` ramp **inside the draw function**
  (`Main.cs:47700-47728`), so the fade is per rendered frame. The port keeps
  the head in `world_sim::npcs::update_invasion_state` on the fixed step, so
  the ramp is deterministic and framerate-independent (the BUG-11-15 merge
  had dropped that arm once already; the test now pins it). The two gates
  that the C# puts *after* the ramp — `invasionProgress == -1` (`:47698`) and
  `alpha <= 0` (`:47729`) — are ported verbatim in the draw
  (`ui::invasion_bar_alpha`), which is what actually stops the panel from
  being permanently on screen.
- **`invasionProgressMode`.** The C#'s mode 0 (Off) zeroes the timer and the
  alpha (`:47718-47722`); modes 1 (Timed) and 2 (On) are the same to the
  draw, and the difference lives in `CheckInvasionProgressDisplay`, which the
  C# gates on `mode == 2` before scanning for nearby invasion NPCs. The port
  reads the same setting through `Settings::event_progress`.
- **The DD2 wave report.** `DD2Event.CheckProgress` calls
  `Main.ReportInvasionProgress(currentKillCount, requiredKillCount, 3,
  currentWave)` (`DD2Event.cs:376-392`); the port's `dd2_report_kill` now
  reports through the same four values, so the Old One's Army draws with the
  `Extra[79]` icon and the `Game.WaveMessage` arm. The moon events' waves are
  still the parked half of that row (no Frost/Pumpkin Moon wave driver).
- **The group plate.** The C# draws a **second** panel at
  `(screenWidth - num13, screenHeight - 80)` with the group's `Extra` icon and
  its `* 0.5f`-tinted `Inventory_Back13` plate (`:47831-47845`). The port had
  only a flat label line under the bar; it now draws the plate, the icon
  (`num * 0.8`, origin `(0, h/2)` so the icon straddles the plate's top edge —
  the C#'s own geometry) and the right-aligned label (`num * 0.9`, anchor
  `(1, 0.4)`).
- **The label table.** The pre-BUG-08 draw had icons 5 and 6 swapped (icon 5
  is `Lang.inter[87]` "Frost Legion", icon 6 `Lang.inter[86]` "Pirate
  Invasion") and no Martian row at all. Both are fixed and pinned by
  `bug_08_the_icon_dispatch_matches_the_source_table`.
- **`Utils.DrawBorderString`'s anchor.** The C# origin is
  `(anchorx, anchory) * MeasureString(text)` in **unscaled** pixels, then the
  sprite batch scales it, so the destination top-left is
  `pos - origin * scale`. `ui::draw_anchored_text` implements that; the rest of
  the HUD uses the zero-anchor `text::border_string`.

## The projectile sprites (M4-PROJ-6b render)

The AI port drew every non-yoyo projectile with the same sprite: `draw_with_hand`
asked `Lookups::shuriken_uv()` for the 22x22 `Projectile_3` frame and stamped it at
a fixed `SIZE x SIZE` quad. The vanilla instead draws
`TextureAssets.Projectile[type].Value.Frame(1, Main.projFrames[type], 0, frame)`
(`Main.cs:27929-27943`) at `projectile.scale`. That gap is now closed for the kinds
the port can spawn:

- **Sheets.** `assets.rs` packs `Images/Projectile_{id}` for every kind
  `projectile::spawnable_kinds()` reports — the set derived from the `proj_data`
  rows themselves (a probe over the id space, `0..=1200`) rather than a second
  hand-maintained list. ~180 small sheets; a missing file just leaves the kind
  unpacked and the shuriken fallback covers it (the `SHURIKEN` 3 dev shot has no
  `proj_data` row and stays on the stand-in).
- **Frames.** `projectile::proj_frames(kind)` carries the `Main.projFrames[type]`
  rows (`Main.cs:8441-8816`): every type defaults to 1 and only 18 of the spawned
  ids exceed it (34 → 6, 645 → 7, 254 → 5, …). `Lookups::proj_frame` slices the
  packed sheet into that many equal rows, clamps the frame index and returns the
  frame's **own** pixel size, so the quad is the art's size, not the shuriken's.
- **State.** `Projectile.frame` (`:451`, reset at `SetDefaults` `:805`) exists so
  the AI arms can advance it; the vanilla advances `frame`/`frameCounter` per
  family (there is no generic animator: `:23477`, `:25213`, …), so each ported AI
  arm owns its own cadence — until one lands, a multi-frame kind draws frame 0.
- **Scale.** The draw multiplies the frame size by `Projectile.scale` (the C#
  `float scale = projectile.scale`), which the yoyo family already set and the
  rest now honour.

**Verification.** One test asserts that >100 sheets pack, that each kind's slice
height equals `sheet_height / proj_frames`, that frame `frames-1` stays inside the
sheet, that the 34/645 rows hold, and that an unpacked kind answers `None` while
`shuriken_uv` stays live. The screenshot fixture is `TR_THROW=<id>` (one shot from
the player's centre on the first tick, re-armed when the pool empties); a
pixel-diff of `TR_THROW=645` against the same frame without it isolates a 35x28
sprite in the poison dart's greens `(34,221,151)`/`(208,253,235)` at the spot the
22 px grey shuriken used to be — `docs/screenshots/m4-proj-6b-poison-dart.png`.

**Still open** (`M4-PROJ-6b`): the caster AI families themselves (Unholy Trident
27, Spectre 59/51, Leaf Blower 40, Razorblade Typhoon 71, Nebula Arcanum 118/119,
Last Prism 75/84) and the GemStaff/MagicMissile/RainbowRod/FlameLash trail
shaders.

## The Leaf Blower petal (M4-PROJ-6b-leaf)

The first of the leftover caster families. `ProjectileID.LeafBlower` (206,
`Projectile.cs:2937-2945`) is the 14x14 petal the Leaf Blower (item 1178, whose
registry row already carried `projectile: Some(206)`) fires at `shootSpeed` 14;
`AI_040_LeafBlower` (`:38003-38082`) is a compact, self-contained arm, so it landed
whole:

- **The row.** 14x14, aiStyle 40, penetrate 1, `timeLeft` 360, `tileCollide` true
  and the `alpha = 255` the AI fades in — the state group carries that override.
- **The fade.** `localAI[0]` counts up; the alpha holds for the first 3 ticks,
  drops 50/tick to 0, then from tick 50 rises 2/tick and `Kill()`s at 255 — the
  petal is visible ~50 ticks and gone by ~153.
- **The flap.** 3 ticks per frame over `projFrames[206] = 5` (`Main.cs:8596`),
  which the sprite pass' `proj_frames` table now carries.
- **The flight.** `ai[0]`/`ai[1]` are drawn once, at tick 1, as
  `Main.rand.Next(-100, 101) * 6E-05`; `localAI[2]` stores the launch-direction
  sign; past tick 10 the velocity is rotated by
  `localAI[2]·|ai[0]|·localAI[1]·0.35` and damped 0.97/tick, and the
  `|vx|+|vy|` band clamps it into 12..16 px/tick (×1.05 below, ×0.95 above).
- **The two RNG draws ride the pool's shared `Main.rand` stand-in** (`hook_rng`,
  the stream the hook family's cracked-brick roll already uses): they steer the
  trajectory, so skipping them would desync the flight; dust — which they *also*
  gate in the C# — is SKIPPED, as in the other M4-PROJ-6 arms (the pool draws no
  dust).

**Verification.** Two tests: the row/frames/registry wiring, and a 100-tick walk
of the AI that pins the alpha on 255 through tick 3 and 0 by tick 10, sees all 5
frames within 30 ticks with `frame_counter < 3`, keeps `ai*` inside
`±100·6E-05`, holds the speed band, matches the `+3.14` rotation, and then runs
the petal to its fade-out kill in under 200 ticks. The screenshot fixture
`TR_THROW=206` isolates a 20x16 sprite in the petal's greens
`(40,146,96)`/`(113,168,24)`/`(63,116,7)` against the same frame without it —
`docs/screenshots/m4-proj-6b-leaf-blower.png`.

## The equipment-page tabs (M7-UI-3a)

The three icons to the right of the inventory grid (`UI/DisplaySlots_2…10`) were
painted by `ui/chrome.rs::draw_page_icons`, which already returned the hovered
tab number — and **every caller threw the value away**. `App.equip_page` could
only be set by the `TR_EQUIPPAGE` debug pin, so all three tabs were decoration.

`Main.DrawPageIcons` (`Main.cs:42787-42868`) is one function with two halves; the
port now has both.

**The geometry** was already correct and stays untouched: the anchors are
`(screenWidth - 162, yPos)` with `x += 82` for the first tab and `- 48` each
after (`:42790-42791`), i.e. `screenWidth - 80 / -128 / -176`, drawn at scale
`0.9` with the state-dependent sheet (`(EquipPage == 2) ? 3 : 2`, `== 1 ? 5 : 4`,
`== 3 ? 10 : 8`, `:42792`, `:42804`, `:42816`) and the highlight sheets
`6 / 7 / 9` at origin `(2, 2)` except tab 3's `(0, 0)`.

**What was missing is the gates.** `Collision.CheckAABBvAABBCollision` runs
against the *native* (unscaled) sheet, so the hit rect is the sheet size, not the
0.9-scaled one — `page_tab_hit` takes the three native sizes (`page_tab_sizes`
reads them from `Lookups::page_icons`). Each tab then applies its own
mouse-item gate, and they are *not* the same:

| tab | `EquipPageSelected` | gate (`Main.cs`) |
|---|---|---|
| first (`DisplaySlots_2/3`) | 2 | `mouseItem.stack < 1 \|\| mouseItem.dye > 0` (`:42793`) |
| middle (`DisplaySlots_4/5`) | 1 | `mouseItem.stack < 1` (`:42805`) |
| right (`DisplaySlots_8/10`) | 3 | `mouseItem.stack < 1` (`:42817`) |

The first tab is the only one a held dye does not block, because that is the tab
that reaches the dye page.

**The click** (`:42828-42858`) is a toggle, not a switch: pressing the page you
are already on falls back to page 0, pressing another selects it. `page_tab_click`
returns exactly that, and returns `None` for tab 3 because tab 3 is the
capture-mode entry (`:42835-42847` sets `CaptureManager.Instance.Active`), not a
page. The sound is `SoundEngine.PlaySound(10)` (`:42857`), the ported `Menu_Open`.

**The page actually drawn is not the page selected.** `Main.EquipPageSelected` is
the player's wish; `Main.EquipPage` is what renders, and it is re-derived *every
frame* by `ItemSlot.SelectEquipPage` (`ItemSlot.cs:4622-4652`) and then

```csharp
if (Main.EquipPage == -1) Main.EquipPage = Main.EquipPageSelected;   // Main.cs:42864-42866
```

`SelectEquipPage` sets `EquipPage = -1` first and then overrides it from the
cursor stack: hooks, mounts and pets force page 2; armour, accessories and (when
the dye page was the selection) a held dye force page 0. The port keeps that shape
— `App::resolve_equip_page` runs from `frame()` and `select_equip_page` returns
`-1` for "no opinion".

**Deviations.**

* Only the air arm and the armour/accessory arm of `SelectEquipPage` are ported.
  The other arms need item fields `ItemDef` does not carry (`Item.shoot` for the
  `Main.projHook` lookup, `Item.mountType`, `Item.dye`) plus the
  `Main.vanityPet` / `Main.lightPet` buff sets. `itemdata.rs` is generated by
  `tools/GenItemData/gen.py` and is marked "Never hand-edit", so adding them is a
  generator change, not an addendum here. Until then a held dye is treated as an
  ordinary held stack and the first tab is refused (`page_tab_hit`'s `has_dye`
  argument stands in for the missing `Item.dye > 0`).
* `Main.CaptureModeDisabled` (`:42821-42825`) is not modelled — tab 3 is drawn
  white, and clicking it does nothing.
* Tab 1 selects `EquipPageSelected = 1`, but the port has no page-1 draw:
  `Main.cs:41368` hands page 1 to `DrawNPCHousesInUI`, the housing panel, which is
  its own subsystem. Page 1 falls through to the equipment columns exactly as
  page 3 already did.
* `TR_EQUIPPAGE` now seeds `equip_page_selected` as well as `equip_page`, so the
  pin survives the per-frame resolution.

**Verification.** 7 new tests (the half-open edges, the per-tab gates, the toggle,
`select_equip_page`, and two `handle_inventory_clicks` end-to-end arms including
the held-armour auto-jump). The suite is 2027 + 18 + 1 + 1236 + 21, 0 failed. The
hover glow is checked as a pixel statistic rather than a screenshot diff: moving
the cursor onto the first tab raises the mean RGB of the `1516..1546 × 138..164`
band from `(3.1, 2.5, 2.1)` to `(24.6, 5.9, 9.5)`, the red-dominant
`Main.OurFavoriteColor`. A whole-frame A/B is useless here because `TR_MOUSE`
also moves the lighting, so the diff bbox covers the whole frame.

### The journey powers' sim wiring (M6-CREATIVE-1c)

The registry from M6-CREATIVE-1 already owned the states and the clock
multiplier; this pass gives the three sim-facing powers their consumers,
ported from the same `CreativePowers.cs` classes:

- **`GodmodePower`** (`:692-727`) → `Player.creative_god_mode`. The vanilla
  clears `Player.creativeGodMode` in `ResetEffects` (`Player.cs:18789`) and
  re-reads the manager a few lines later (`:25493-25497`); the port refreshes
  the flag at the head of its fixed step (`world_sim::player::update_player`),
  which is the same "reset, then read the power" order. Consumers:
  `Player::hurt_inner` returns before the iframes/defense/knockback
  (`:38203-38206`), `Player::start_death` refuses outright
  (`:38809-38812`), `Npc::contact` returns before the whole
  `Update_NPCCollision` body — including the EoC shield-dash arm
  (`:31321-31324`) — and the fixed step refills `hp`/`mana`/`breath` to
  their maxima (`Main.UpdateWorld_Players`, `Main.cs:17869-17873`).
- **`FarPlacementRangePower`** (`:730-767`) → `Player::far_placement_range`,
  which makes `Player::tile_range_x/y` return the C#'s
  `DefaultTileRangeX/Y` (`5`/`3`, `:57113-57114`) or the journey form
  `* 2 + 8` (`18`/`14`, `:18815-18828`). The `update_editing` reach gate —
  the port's documented circular stand-in for
  `IsInTileInteractionRange(..., TileReachCheckSettings.Simple)` — now
  derives its radius from `tile_range_x() + 1`, so the base gate is the
  previous 6-tile constant unchanged and the power widens it exactly as the
  C# widens the rectangle.
- **`StopBiomeSpreadPower`** (`:1540-1571`) →
  `CreativePowers::allowed_to_spread_infections`) =
  `WorldGen.AllowedToSpreadInfections` (`WorldGen.cs:71502-71507`);
  `tr_core::growth::update_world` gained the flag as an explicit parameter
  (it had been hardcoded `true`) and `world_sim::growth::update_growth`
  feeds it to both `update_world` and `spread::hard_update_world`, which
  already carried it.

Deliberate deviations:

- The port has no journey-difficulty flag (`Main.IsJourneyMode`), so the
  `FarPlacementRangePower` toggle alone is the gate, and its registry
  default stays `false` — in the vanilla `_defaultToggleState = true` is only
  reachable inside a journey world, whose creation screen the port does not
  have. Enabling the power from the (unported) powers UI or a load is what
  turns it on.
- The godmode refill runs inside the fixed step instead of
  `Main.UpdateWorld_Players`' frame tail; the two coincide at 60 Hz and the
  pools are only ever read after the step.
- The C# NPC-side gates (`NPC.cs:37157`, `:44305`, `:56483`, `:63404`,
  `Projectile.cs:12110`, `:31804`) only stop bosses/minions from *targeting*
  a godmode player. They are not ported: the damage and death arms above
  already make the player invulnerable, and the targeting arms change no
  observable state the port models.
- Still open on the row: the powers UI (`UICreativePowersMenu`) and the
  per-player persistence (`IPersistentPerPlayerContent`, `godmodePowerEnabled`/
  `farPlacementRangePowerEnabled` in the `.plr`).

## The friendly-fire melee gate (BUG-10)

`ProcessHitAgainstNPC` (`Player.cs:44262-44304`) is the one place the player's
melee decides whether an NPC may be hurt. The gate its friendly half needs:

```csharp
if (nPC.friendly && (nPC.type != 22 || !this.killGuide) &&
    (nPC.type != 54 || !this.killClothier) &&
    (!nPC.isLikeATownNPC || (sItem.type != 5129 && sItem.type != 3351)))
    return;
```

The earlier partial port compared `NpcKind::is_town_npc`, which is the *housing*
subset (the ten kinds with a ported house/shop row), so eleven seller kinds —
Angler, Stylist, Tax Collector, Golfer, Zoologist, Princess, Dye Trader, Party
Girl, Wizard, Mechanic, Travelling Merchant — and the six bound/rescue forms
(Bound Goblin/Wizard/Mechanic, Webbed Stylist, Sleeping Angler/Golfer) were all
hittable. The gate now reads the real vanilla flag:
`NpcKind::is_friendly` (`NPC.cs:8519` default false plus the `SetDefaults` arms
`:8983-9060`, `:10257`, `:10493-10507`, `:13502-13516`, `:13796-13809`,
`:16652-16665`) and `NpcKind::is_town_npc_like` for the C# `isLikeATownNPC`
(`:6856-6864` — `townNPC || type == 453`; the Skeleton Merchant is not shipped,
so the set is exactly the town kinds).

**The exceptions.**

* The doll flags are armed by `ApplyEquipFunctional` (`Player.cs:14368`), which
  `UpdateEquips` calls for the accessory slots 3..10 (`:12948-12954`), and
  `ResetEffects` clears them every tick (`:18365-18366`). The gate reads the
  per-tick `Player.equipped` snapshot (`world_sim/player.rs` fills it from
  `App.equip.armor`), so the previous inventory-wide scan — a doll sitting in the
  bag used to arm the easter egg — is gone. The snapshot is the closest the
  current equip model gets: the vanilla skips locked accessory slots through
  `IsItemSlotUnlockedAndUsable`, which the port does not model (every slot is
  treated as unlocked, the default character's 5 + the Demon Heart's 6th).
* The Flymeal (5129) and the Tax Collector's Stick of Doom (3351) are the only
  melee items the C# whitelists against town NPCs (`:44274`). Both are hitbox
  melee items with no `shoot`, so the melee arm is their only damage path.
* The cane's hit is forced to one damage (`:44300-44304`) and the Flymeal doubles
  its roll, doubles again on the Nurse (`:44344-44353`) and takes
  `armorPenetrationPercent = 1f`. The penetration value is added to the damage
  like the C#'s `num2 += nPC.checkArmorPenetration(...)` (`:44383`) through the
  already-ported `tr_core::damage::check_armor_penetration`, with the player's
  own `GetArmorPenetration` passed as 0: the prefix table carries
  `armor_penetration` (`prefix.rs`) but no consumer sums it into the player yet,
  and the NPC-side ichor / broken-armor / Betsy's-curse terms are not surfaced by
  the port's NPC buff store. Both are neutral defaults, not approximations of a
  shipped behaviour.

**Out of scope (the same defect class, own row).** Hostile/friendly projectiles
run the same refusal through `Damage_PVE_Inner`'s `flag` (`Projectile.cs:12601`)
and `flag2`'s hostile-vs-friendly arm (`:12605-12608`); `projectile.rs`'s
`damage_pve` still filters on geometry only, so a player arrow can hurt a town
NPC. Tracked as BUG-23 in the catalog.

**Verification.** 3 tests in `world_sim/editing.rs`: the Guide refused and the
worn doll arming the hit while the bagged doll does not; the 17 seller/bound
kinds refused through `is_friendly`; the two item exceptions (the Flymeal's
doubled roll, the cane's exact one damage) and the hostile still hittable.

### The projectile half (BUG-23)

`Damage_PVE_Inner`'s refusal (`Projectile.cs:12601-12608`) is the projectile
twin of the melee gate, and it reads the same two doll flags — the port now
shares them through `Player::doll_flags` (the melee arm and the pool both call
it). The C# expression is ported verbatim:

```csharp
bool flag = !targetNPC.friendly | (this.type == 318) |
    (targetNPC.type == 22 && this.owner < 255 && Main.player[this.owner].killGuide) |
    (targetNPC.type == 54 && this.owner < 255 && Main.player[this.owner].killClothier);
if (this.owner < 255 && !Main.player[this.owner].CanNPCBeHitByPlayerOrPlayerProjectile(targetNPC, this))
    flag = false;
...
if ((!this.friendly || (!flag && !NPCID.Sets.ZappingJellyfish[targetNPC.type])) && !(this.hostile & flag2))
    return;
```

The gate runs in **both** PVE walks: the plain point walk (`damage_pve`) and the
resized death bursts (`damage_pve_boxed` — the gem bolts' AoE and the bombs'
`PrepareBombToBlow` walk), because the C# funnels all of them through
`Projectile.Damage()` → `Damage_PVE` → `Damage_PVE_Inner`. The refusal sits
ahead of the per-owner immunity gate, matching the C# early `return`.

Neutral arms (no ported system to read):

* `NPCID.Sets.ZappingJellyfish` — 63/64/103/242; no jellyfish kind is ported, so
  the "poke the jellyfish and get zapped instead" arm has nothing to act on.
* `this.hostile & flag2` — hostile projectiles damaging friendly NPCs (the
  invasion flavour). The port's hostile shots (`friendly == false`) run the
  player-contact pass only, so there is no hostile-vs-town path to gate yet.
* `CanNPCBeHitByPlayerOrPlayerProjectile`'s critter block (`:20513-20529`) — it
  needs `dontHurtCritters` (the Guide to Critter Companionship item) or a
  minion/sentry shot; neither exists in the port (the summon system is the
  blocked M4-PROJ-8 row). With both false the function returns `true`, so
  `flag` is unchanged — the port passing the critter kinds is faithful.

**Verification.** 1 test (`projectile::tests::bug_23_…`): a shuriken passes a
Guide without consuming itself, the same shot lands once the Guide Voodoo Doll
sits in accessory slot 3, and the `refuses_friendly` matrix is pinned for the
Rotten Egg (318), the Clothier-with-curse arm, the bound goblin (no arm — the
C# `isLikeATownNPC` excludes it) and a hostile slime.

### The coin and ammo rows (M7-UI-2b)

`Main.DrawInventory` draws the coin row (context 1) and the ammo row (context
2) with `ItemSlot.Handle` over every cell it draws (`Main.cs:41752-41782`), so
the port's draw-only rows were dead to the pointer — the click pass only knew
the 10×5 grid (`slot_at_mouse`) and the closed hotbar.

The rows now resolve through one shared geometry, `ui::inventory::coin_ammo_hit`
(x 497 / 534, scale 0.6, `(int)(85 + row·56·0.6 + 20)`), which the draw pass
also hovers with — the two copies of the formula cannot drift. The click path
(`App::coin_ammo_click`) is the C# cell handler with the C# gates:

* left → the shared action-0 move (`left_click_movement`), behind
  `PickItemMovementAction`'s accepted-kind gate (`ItemSlot.cs:1369-1374`): the
  coin row takes air and 71-74 only, the ammo row takes `FitsAmmoSlot`;
* right → the `ItemSlot.cs:1700-1772` stack split (half up, one down);
* the rows are inert while the inventory is closed (they draw only in the open
  panel).

`FitsAmmoSlot` (`Item.cs:1369-1376`) is ported over the flags the port carries —
air, `ammo > 0`, `bait > 0`. Deviations: the `PaintOrCoating` arm, `notAmmo`,
and the three literal exceptions (353/530/849) have no port counterpart, so a
held paint bucket is refused where vanilla takes it.

The shift arms landed in **M7-UI-2c**: `ItemSlot.GetAlternateClickAction`'s
ctx-1/ctx-2 cases (`ItemSlot.cs:657-665`) call the same `TryPlacingInChest` with
the context's array, so `chest::try_placing_in_chest` was split into
`try_placing_from(app, Source, just_check)` with `Source::Inv | Coin | Ammo`
reading and writing the right array (`App.inventory` vs `App.equip.coins`/`ammo`
— the C# has one `Player.inventory[0..58]`), and the old signature delegates with
`Source::Inv`. The cursor-9 light and the cursor-9 click both resolve the row
first (`App::chest_transfer_source`).

The Alt favorite toggle landed in **M7-UI-2d**: the port keeps favorites as a
slot-index set (`App.favorite_slots`), so the rows borrow the vanilla numbering
instead of a flag on the item — `inv::INV_SLOTS` (58), `inv::coin_slot_index`
(50+k) and `inv::ammo_slot_index` (54+k), which is exactly what
`Player.inventory[0..58]` gives the C#. `toggle_favorite`'s guard moved from
`INV` to `INV_SLOTS`, `hydrate_favorites`/`favorite_bits` cover the row band (the
`.plr` reader already collected 58 bools, `player_file.rs:275-291`, so the
round-trip is now consistent), the Alt gate and the click arm cover contexts 1/2,
and a favorited row draws the gold sheet and refuses the shift transfer
(`:631-634`). Known limit: the port's `.plr` writer still stores zeros for the
coin/ammo rows' items, so a row favorite is session-only until that writer lands
— the grid favorites have the same shape.

## The housing page (M7-UI-3b)

The inventory's second tab (`EquipPage == 1`) is `Main.DrawNPCHousesInUI`
(`Main.cs:41993-42107`). The port drew the armour columns there instead — the
`if equip_page == 2 { misc } else { armour }` shape had no page-1 arm, so the tab
showed equipment that vanilla shows on page 0. `ui::housing` ports the page now:

* **Which heads** (`:42008-42023`): the C# walks `Main.maxNPCs`, resolves every
  active NPC's head through `TownNPCProfiles.GetHeadIndexSafe` →
  `NPC.TypeToDefaultHeadIndex` (`NPC.cs:7510-7553`), drops a head that is `<= 0`,
  out of range or in `NPCHeadID.Sets.CannotBeDrawnInHousingUI` (21, 80), and keeps
  only the first *type* it sees per head. `housing::heads_for` does the same over
  `App::npcs`, which is the port's `Main.npc[]`. Two port-specific traps: the
  bound forms (`BoundGoblin`/`BoundWizard`/`BoundMechanic`) borrow the freed
  row's `net_id` for the bestiary (`bestiary.rs:1502-1506`), so they would inherit
  a housing cell vanilla never gives them — `housing::head_of` rejects them
  explicitly; and the sheets are packed per head id, so `Lookups::npc_heads` is a
  0-based table of 81 entries with `None` where the port ships no NPC.
* **Order and layout** (`:42044-42058`): `NPCHeadID.Sets.HeadListOrder` is the
  draw order (Guide first, "?" fifth in the array but drawn per its position), not
  the head numbering. The cell pitch is `56 * inventoryScale`, the origin is
  `(screenWidth - 64 - 28, inventoryTop)` where `inventoryTop = 174 + m_h`, and a
  row that would land past `screenHeight - 80` starts a new column 48 px to the
  left, re-based on `inventoryTop` (`num3 -= 48; num2 -= num9 - inventoryTop`).
  The C# truncates the row to an int *before* adding that offset, so the port's
  `layout` does the same — a float version drifts the wrapped column by up to a
  pixel.
* **Draw** (`:42079-42095`): `Inventory_Back11` tinted `Main.inventoryBack`
  (0.863 grey, like every other inventory background), then the head sheet
  centred on `(x + 26*scale, y + 26*scale)` and shrunk to `36 / max(w, h)` when it
  is larger than 36 px. The shipped content pack's heads are 24x22, so they draw
  1:1.
* **Hover** (`:42059-42065`, M7-UI-3c): the cell under the pointer prints a
  name through the tooltip layer — `Lang.inter[8]` for the query cell, the NPC's
  name otherwise. The C# prints `Main.npc[n].FullName`, i.e. the *given* name
  plus the type ("Andrew the Guide"); the port stores no per-NPC given names, so
  it prints the type name from the `NPCName.<Kind>` localization key
  (`housing::name_key`) — the same string the bestiary entry carries.
* **Not ported**: the click (`Main.SetMouseNPC` / `SetMouseNPC_ToHousingQuery`),
  because the port has no `Main.mouseNPC` housing query at all; the
  `Inventory_Back14` white swap, which is the gamepad UI-point highlight the port
  still lacks (the same reason the grid's `_17` highlight is open).
* **The PVP-icon cut** (`:42050-42056`, M7-UI-3d): a wrap that pushes the column
  offset to `-144` sets `Main.hidePVPAndTeamIcons`, and `DrawPVPIcons` — which
  runs after the housing page in the same frame — then draws nothing.
  `housing::hide_pvp_icons` reports it on `InventoryHit` and `ui::draw_hud` skips
  the icons. Only reachable in a world with ~30 town NPCs (three columns).

## The defense stat (M3-EQUIP-5)

`Player.statDefense` had a consumer since M3-EQUIP-2 (the `Hurt` reduction) but
two writers that disagreed: `Player::update`'s buff/prefix/equipment passes added
into it, and `world_sim::player` then **assigned** `pieces + set` over the result
right before the tick ran. So the armor pieces worked, and every other contributor
was silently thrown away: Ironskin (buff 5, +8), Ichor (69, −15), the Cat
Bastet's Blessing (215, +5), the Dryad's Ward (165, +8), Tipsy (25, −4), Weak
(33, −4 per slot), the fed tiers (26/206/207 = +2/+3/+4) and the accessory
prefixes 62-65 (`GrantPrefixBenefits`, +1..+4).

The fix is an ordering fix, not a new table: the field is now derived entirely
inside `Player::update`, in the C#'s own order — `ResetEffects` zeroes it
(`:18327`), `UpdateBuffs` adds its arms (`:9962`, `:10064`, `:10108`, `:10306`,
`:11915`, `:11921`, `:11934`, `:11950`, `:11966`), `UpdateEquips`'s
`GrantArmorBenefits` adds every wearable's `Item.defense` over the ten functional
slots (`:13121`, so the seven accessories count — a Shackle +1, an Ankh Shield
+4) plus the completed set's bonus, `GrantPrefixBenefits` adds the accessory
prefix rows, and the tail clamps `statDefense < 0` to 0 (`:25927`) so a stack of
debuffs can never *raise* incoming damage.

Not ported yet, and why: buff 28 (Werewolf, +3) needs the `wolfAcc` item flag the
port does not carry; the `/2` halvings at `:26021-26025` belong to armor-set
abilities that are still unported (see M3-EQUIP-2's early-metal-only set table),
as does the `:25716` +20 row.

## The mob draw audit (M4-NPC-3t)

The lunar-event families were the worst-served art in the port: the
`NpcAssets` struct loaded **one shared sheet per family** and pointed every
member at it, so eight different mobs drew the same strip —

| family field (v1) | sheet | member kinds |
| --- | --- | --- |
| `tower_mobs` | `NPC_409` | all eleven stardust/solar tower mobs |
| `nebula_trio` | `NPC_420` | floater, brain suckler, predictor |
| `vortex_trio` | `NPC_426` | hunter, alien, headcrab |
| `shards` | `NPC_406` | big/small star cell, jellyfish |
| `shard_hovers` | `NPC_425` | nebula beast, vortex soldier/rifleman |
| `moon_lord_parts` | `NPC_396` | all five Moon Lord parts |

Vanilla gives every one of them its own `TextureAssets.Npc[type]`
(`NPCID.cs:11481-11533`, `:11713`) and slices it by
`Main.npcFrameCount[type]` (`Main.cs:67082-67131`, the divisor
`NPC.SetFrameSize` uses, `NPC.cs:90780-90800`). The port now ports that
table (`render/entities.rs`'s `NPC_FRAME_COUNT`), loads `NPC_402-407`,
`NPC_409-414`, `NPC_415-421`, `NPC_423-429` and `NPC_518` into
`NpcAssets::npc_sheets`, and every arm reads `npc_sheet(texture_net_id(kind))`
at `npc_rows(...)` rows.

`texture_net_id` exists because the port's `NpcKind::net_id` is
**bestiary-driven** (`bestiary::entry_for_npc(...).net_id`) and several of the
lunar rows were keyed on the art's first member rather than the drawn kind
(`MoonLordHead` resolves to 397, `MoonLordLeechBlob` to 212). Those bestiary
ids are a separate audit; the draw uses the vanilla texture ids.

**The generic anchor was wrong for every generic-anchored kind.** The C#'s
generic path draws with `origin = halfSize` at
`position = Center - frame.Size()/2 + halfSize + offsets`
(`Main.cs:25505-25516`), so with the default `halfSize = frame.Size()/2` the
frame's **centre lands on the NPC's Centre**. The v1 put the frame's centre at
`box_bottom + 4`, i.e. `h/2 - 4` (≈ 24 px on a 40-px humanoid) too low — every
humanoid's feet were buried in the grass (the TR_NPC lineup screenshot was the
give-away). All twelve such arms (the humanoid/martian/prime/boss/generic
helpers plus the inline arms) now use `n.y + h * 0.5 - fh * 0.5`; the
half-`Size` overrides (`boss_half_size`, `npc_anchor_nudge`) still apply after
the match, as in the C#.

**The Moon Lord head is a grid, not a strip.** `Main.npcFrameCount[396]` is 1,
which only means the dead generic path would show the whole 573x804 sheet;
the real draw is `Main.cs:24846-24927`:
`value40.Frame(3, 3, frame.Y / 3, frame.Y % 3)` at `scale = 2` from the origin
`(191, 130) / 2` — a 3x3 grid of 191x268 cells with the **column** indexed by
`frame.Y / 3` and the **row** by `frame.Y % 3` (so `FindFrame`'s 0..8 cell
counter is right). `moon_lord_head_draw`/`moon_lord_head_uv` port exactly that.

**Row-count fixes.** `FindFrame`'s arms for the shards/towers/nebula/vortex
groups were placeholders with hand-written counts (`StardustSpiderSmall` walked
11 rows over a 5-row sheet, five rows of which do not exist). They now ride
`npc_rows(texture_net_id(kind))`. The DD2 rows were corrected against
`Main.npcFrameCount`: the Dark Mage 20 → 9 (564-567), the Bartender 16 → 25
(550), the Wither Beast 14 → 17 (568/569).

**The per-kind `FindFrame` machines (M4-NPC-3u).** The arms above still shared
one `frame_counter += |vx| + 0.6` accumulator, so the walk cadences, the air rows
and the attack bands were invented. Every lunar machine is now ported from its
own `case` (`NPC.cs:70119-70803`), one per vanilla id:

| id | kind | machine |
| --- | --- | --- |
| 405/406 | star cells | 4-tick full wrap |
| 407 | big jellyfish | 6-tick full wrap |
| 409 | big stardust spider | `ai[1]` 1..30 pins the last row, else 6-tick walk wrapping two rows early |
| 410 | small stardust spider | row folds every 10 px of `|vx|`, wraps at the end |
| 411 | stardust soldier | `ai[1]` 120..240 pins the last row + flips; walk from row 2, wraps one early |
| 415 | solar drakomire | `ai[2] < 0` charge flips rows 8/9 off the 18-tick counter; walk rows 2..count-2 at `|vx| * 0.5`; air row 1; stand row 0 |
| 416 | drakomire rider | copies its 415's row in the C# — the same machine |
| 417 | solar sroller | `ai[0] == 0` rolls `counter / 6` over 48 ticks; `ai[0]` 1/5 climb 9..14 off `ai[1] / 6`; else row 14 |
| 418 | solar corite | `ai[0]` 0/±1 walks 0..4 at 6 ticks; `ai[0] == 2` splits `ai[1]` into 5/6/7; `ai[0] == -2` spins 5..7 at 12 |
| 419 | solar selenian | `ai[2] < 0` charge climbs 2..8; walk from row 9, wraps at the end; air row 1 |
| 420/421 | nebula floater / brain suckler | 5-tick wrap; the suckler's `ai[0] == 5` double-ticks the counter |
| 423 | nebula beast | `ai[2] == 1` pins rows 2/3 off `ai[1]` 30..45; air row 1; walk rows 4.. at `1 + |vx| / 2`, flips off `-sign(vx)` |
| 424 | nebula predictor | `ai[2] > 0` pins the last two rows off `ai[1] > 90`; walk rows 2..count-2; air row 1 |
| 425 | vortex rifleman | `ai[2] == 1` cycles 10..12 at 3 ticks; walk rows 2..9 at `|vx|`; air row 1; flips off `-direction` |
| 426 | vortex hornet queen | `ai[1] >= 45` cycles 12..15 at 2 ticks (row 8 grounded); air 9..11; walk 1..7 at 10 px |
| 427 | vortex hornet | air cycles 6..8 at 2 ticks; walk 1..5 at 8 px |
| 428 | vortex larva | walk wraps one row early at 10 px; the fall latches row 4 after 10 ticks or `vy > 0.6` |
| 429 | vortex soldier | walk from row 2 at `|vx| + 1`, wraps at the end; air row 1 |
| 518 | solar spearman | the same shape as 429 |
| 402, 412-414 | stardust worm head, crawltipede links | no `case` at all — row 0 |

The three row helpers (`rows`, `next_row`, `last_row`) read
`npc_rows(texture_net_id(kind))`, so a machine can never walk past its sheet;
`drakomire_frame` is the shared 415/416 body.

**Parked (documented follow-ups).**
* The Moon Lord hand (397), core (398) and free eye (400) custom blocks
  (`Main.cs:24701-24951`) — their extra parts (the arms, the tail, the eye's
  laser sprites) stay undrawn; the parts draw one frame each at the generic
  anchor.
* `spriteDirection` write sites outside the ported machines (`NPC.cs`'s 24
  `spriteDirection =` sites) — the flip rides `n.sprite_dir`, which several
  machines never write.
* The kinds whose bestiary `net_id` disagrees with their texture id:
  `MoonLordHead` resolves to 397, `MoonLordLeechBlob` to 212, and the lunar
  families are keyed on the art's first member. The draw uses
  `texture_net_id`, the bestiary keeps its own ids — the two need one audit.
* The solar drakomire's `IsABestiaryIconDummy` arm (416) and the 416's
  parent lookup are not modelled: the port runs the parent machine on the rider
  instead of copying the parent's row.
