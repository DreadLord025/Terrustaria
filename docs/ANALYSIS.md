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
| World model & save format | `IO/WorldFile.cs`, `Tile.cs` | ~2 600 | ✅ | `.wld` header + section table + RLE tile stream parsed; chests/signs/NPCs skipped. Native `.trwld` writer/reader for generated worlds. `Tile` is 12 bytes (slope/paint and wall cell packed); 5.04 M-tile world = 60.5 MB |
| Tile framing / autotiling | `WorldGen.TileFrame` (line 82852), `TileFrameCosmetic` | ~3 600 | 🟡 | Generic block chain + merge families ported; slopes/half-bricks framed; per-type grass edge/moss/vines/weird branches approximated |
| Wall framing | `Framing.WallFrame` (line 326) | 120 | 🟡 | Full `wallFrameLookup` + `centerWallFrameLookup` port; variant is a position hash, `TruncatesWalls`/invisible-wall neighbour rules and persisted `wallFrameNumber` are not modelled |
| Tile drawing | `GameContent.Drawing/TileDrawing.cs`, `WallDrawing.cs` | 9 700 + 1 300 | 🟡 | Quads, 9-sliced light, slopes, half bricks, tree canopies/branches, tile-3 plants; walls are 32x32 sprites at `-8` with corner lighting, drawn in their own pass; no paint, wind, glow, tile entities |
| Lighting | `Graphics/Light/LightingEngine.cs`, `LightMap`, `TileLightScanner` | ~5 300 | ✅ | Scanner + blur + sunlight; 4 modes, `GlobalBrightness`, 9/4-slice block smoothing; sun/moon/flicker still missing |
| Sky / backgrounds | `Main.cs` sky code, `Backgrounds/*` | ~3 000 | 🟡 | `Background_0` gradient + two parallax mountain layers (`Background_7`), sun/moon on `DrawSunAndMoon`'s arc (scale/phase/rotation), stars, deterministic `Cloud_*` layer, the textured `Background_1/2` underground backdrop anchored to `worldSurface` and the rock/magma/hell passes (`Background_3/4/6/125..127/185..187`, underworld black box) in `DrawBackground` order; cloud density and biome backgrounds/rain still partial |
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
- Not ported yet: the honey/shimmer `Draw()` passes (`Main.liquidAlpha[2..4]`), the
  shimmer vertex-sparkle pass (`SetShimmerVertexColors*`), `TrySparkling` dust and the
  ambient lava-waterfall light (`AddLight`). `TR_NO_WATERFALL=1` disables the pass for
  A/B screenshots; `TR_SPOUT=x,y` stamps a half-brick spout with liquid and a clear drop
  so the overlay can be captured deterministically.

Verification: unit tests in `waterfall/` (spout detection, lava kind, tile 196, frame
counters, emitted geometry, empty `anim_key`); the lake view at `TR_CAM=2240,360`,
`TR_DAY=1` is pixel-identical with and without the pass when no waterfall is visible
(`tools/pixel_diff.py`, 1 440 000 pixels); `docs/screenshots/spout_noon.png` shows a
spout at `TR_SPOUT=100,25`, `TR_CAM=100,32`, zoom 2 — the column samples the raw
`Waterfall_0` colours (`147,177,253` / `50,107,197` / `9,61,191`), i.e. a full-brightness
tint, as `GetAlpha` requires above `worldSurface`.


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

First-pass simplifications: a session-local random weather schedule instead of a saved
`rainTime`/`maxRaining` pair, no lightning bolt geometry or thunder audio, and no wind
gusts affecting the world (only the rain/snow drift).

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
Cloud density (`Cloud.GetParallax`) and the exact star density remain (BG-16).

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

## Player (M2 first pass)

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
`body_frame`): idle is row 0 (the rest pose), airborne (`velocity.Y != 0`) is row 5, and
walking advances one row per `8.0` of `legFrameCounter += |velocity.X| * 1.3`, cycling the
walk band `7..=19` with the body copying the leg row; during an item swing the app applies
the `useStyle 1` body rows `1 -> 2 -> 3` via `apply_item_use_frame`. (An earlier version
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

Not ported: the full `Item.cs` use logic, tool requirements and mining speed, item sprite
sheet lookup, inventory windows/crafting/chests, paint, actuators/wiring, and liquid
re-simulation after an edit.

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

## Item registry (M3)

`tr-app/src/registry.rs` holds the `Terraria.ID.ItemID` entries the game currently uses
(dirt/stone/ores/torch/wood/walls/shuriken/coins/sand/ash/hellstone/cactus/snow) with
their vanilla names, `Item.maxStack` (coins cap at 100), and the tile/wall/projectile
they place or fire. Inventory slots store vanilla `ItemID`s: mining converts tile ->
`ItemID` (grass drops dirt), walls map through `by_wall`, weapon items expose
`Item.shoot`, and pickup uses `Player.PickupItem`'s order (stack into the hotbar, then an
empty hotbar slot, then the storage rows) with the per-item stack limit. The sidecar save
is version 3 (ItemIDs); versions 1/2 are migrated on load (`registry::migrate_kind`).
Item drops and the held item draw the original `Item_<id>` sprite, falling back to the
tile frame for ids without one.

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
* The reforge/guide/crafting slots are drawn from `Main.DrawInventory`'s constants but
  their menus are not opened, so they show as empty frames.

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

* `Main.MapScale` is `1.0` only. The whole minimap transform chain
  (`mapScale = Main.MapScale`, the `CreateScale(MapScale)` position/scale
  fixups, `Main.cs:55308-55329`, and the foreground batch at `:55768`) is
  evaluated at the reference value, where every `*= num16` fixup is a no-op;
  the `MapScale` config key still round-trips (load-clamped `0.5..1.0`,
  `Main.cs:4494`). The `Map Scale` Interface slider is a P5 hand-off, not ported.
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
* `BigProgressBarSystem.ShowText` is always on (`:259`); the boss-bar-numbers
  settings toggle does not exist in the port, so the health text always draws.
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
`TR_CRAFT=1`/`TR_NOCRAFT=1` (force the crafting panel on/off),
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
* **`"HidePassword"` vs vanilla `"HidePasswords"`** (`Main.cs:4144` Put,
  `:4422` Get): the port's config key drops the plural — pre-existing from the
  settings-persistence milestone, kept so existing port configs still load
  (a vanilla `config.json`'s `HidePasswords` entry is ignored by the port).
* **Stored without consumers yet** (visual-only, spec §9.1): the volume sliders
  (no audio engine reads the volumes), the six Effects toggles / smart-cursor
  flags / `FlashyEffects*` (no gameplay consumer), `boss_bar_text` (P4's boss bar
  draws the numbers unconditionally), `resource_sets` (the HUD always renders the
  Classic set — the default above) and `portrait_preference`. The Interface
  "Minimap Border" row is the exception — live, see below.
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
    Bar Numbers: On" by default, not the plan's "Off". The flag is stored
    only; P4's boss bar draws the numbers unconditionally (its note above
    still holds until a consumer lands).
* `resource_sets` defaults to `"Default"` (label "Classic") while vanilla's
  config default is `"New"` (`PlayerResourceSetsManager.cs:28`) — the planned
  deviation: the port renders the Classic set, so the row must agree with the
  HUD. A vanilla `config.json` value still round-trips unchanged.
* The Interface "Minimap Border" row is *live* behaviour, not visual-only:
  cycling rewrites `settings.minimap_frame`, which the game loop feeds
  through `ui::minimap::style_index` (`app/clicks.rs`), so the minimap frame
  re-renders immediately. Map Scale stays the P4 visual-only ruling
  (reference 1.0).

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

## Architecture refactor (2026-11-01)

The god-file split (`docs/superpowers/plans/2026-11-01-architecture-refactor.md`,
commits `9c84351..028daba`) moved code, gave the moved functions readable Rust
names (overriding the C#-name rule for this pass) and added three C#-shaped
trait seams. Behaviour and RNG streams were pinned unchanged: the worldgen
byte-proof's four FNV-1a state hashes and a post-refactor determinism pair
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
- **`env.rs` is deferred.** The planned consolidation of the ~65 `TR_*` flags
  into one module did not happen; the flags still live where they are
  consumed (`app/mod.rs` helpers, `render/debug.rs`, `app/flow.rs`,
  `rain.rs`, `render/sky.rs`, …).
- **The P6 stand-ins** this sweep documented (see "Front-end screens (P6)"):
  `GameTipsDisplay` not ported, the Crc32-instead-of-`Main.rand` world-name/
  seed composition and the `assign_random_world_name` stand-in.
- **Risk 7 above is closed**: the generator matches the current
  decompilation again, and `tiledata.rs` is generated from it (the hand
  extraction in `torch.rs` stays as-is).
