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
- The `Players[8]` undershirt passes (back arm, front arm/shoulder) are skipped:
  the starter skin has no underShirt colour; the palette lands with M3-EQUIP.
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
- `solid_above` on the Rust call site is `active && TILE_SOLID &&
  !TILE_SOLID_TOP` at `(x, y-1)` (`render/tiles.rs`); the C# analogue probed
  by case 4 is `WorldGen.SolidTile(x, y-1)` (`:4672-4679`). The predicate
  delta is a follow-up, documented on `draw_data` (Phase 4).
- `DoNotAdjustDrawPositionBasedOnTileWidth = {711}` and the positional switch
  of `DrawSingleTile` (`:1020-1159`) remain follow-ups (out of TILE-DRAW-1).

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
- The chest is a pure tile object: the `Chest.FindEmptyChest`/`Chest.
  AfterPlacement_Hook` hooks (`TileObjectData.cs:3464-3465`), chest indices
  and item storage are NOT ported — the M3-CHEST-UI seam.
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
  `CanDestroyChest`, `Chest.cs:609-623`; the store stays empty until
  M3-CHEST-UI) and the tile under a container {21,26,72,77,88,467,488}
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
  explosion/replication pass `|_, _| false` for the chest store (not
  reachable without the app — unobservable while the store is empty).
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
- **Persistence** in `.trwld`/`.map` is pending (M6-SAVE-1); the C# keeps TEs
  in the world-file section 7 (`WorldFile.cs`).
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

`wiring.rs` gains the statue half of `HitWireSingle`'s case 105
(`Wiring.cs:2125-2440`): the style decode (`num138 = frameX / 36 +
(frameY / 54 % 3) * 55`), the 3×2 `SkipWire`, the spawn anchor
`(num137·16 + 16, (num136 + 3)·16)` and the verbatim style → spawn map. The
NPC styles run `CheckMech(30)`; the item styles `CheckMech(600)` plus the
`Item.MechSpawn` group checks. The engine emits `StatueSpawn` requests; the
app layer (`world_sim/wiring.rs`) runs the `MechSpawn` caps and constructs the
entities.

Deviations:

- **The ported statue subset**: the Blue Slime statue (style 4 → NPC 1 →
  `NpcKind::Slime`/Blue), the Demon Eye statue (style 35 → NPC 2) and the
  Heart/Star/Bomb item statues (styles 37/2/17 → the C# item ids 58/184/166,
  dropped as `ItemDrop`s with the pickup ids). The remaining ~50 styles map
  to C# NPCs/items outside the port's content (the critters, the Eater of
  Souls, the special teleporting/jar statues) — they emit nothing; the map
  arms arrive with the NPC families (M4-NPC-1).
- **The `MechSpawn` caps are verbatim**: `NPC.MechSpawn` (`NPC.cs:7420-7505`,
  the rings 200/600 px, the caps 3/6/10, the same-type group merging — the
  port's kinds map 1:1 to their C# ids so the groups only matter once those
  NPCs land) and `Item.MechSpawn` (`Item.cs:49385-49405`, the rings 300/800
  px) — implemented in `crates/tr-app/src/wiring_stub.rs` over the app's
  entity lists.
- **The statue-spawned loot gating**: `Npc.spawned_from_statue` carries
  `NPC.SpawnedFromStatue` (`NPC.cs:6061`); the `NPCLoot`'s
  StatueSpawnedDropRarity/NoEarlymodeLootWhenSpawnedFromStatue checks
  (`NPC.cs:78849`) gate the drops — the port's statue-spawned slimes drop
  nothing until the AnyInteractions tracking lands (M4-NPC polish).
- **The trap projectile emission** (the dart/spear/flame/boulder traps'
  `Projectile.NewProjectile` calls, `Wiring.cs:1774-2000`) needs the
  projectile families (M4-PROJ-2+) — their `CheckMech` cooldowns (200/90/300/
  900) are already exercised by the queue; the emission arms are
  passthroughs.

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
nearby-chests toggle. Deviations: the store's population is M3-CHEST-UI's
(the chest tiles place with no inventories yet), the save/persistence is
M6-SAVE-2's, the banks/void bag are M3-STORE-1's; `Chest.IsLockedOrInUse`
passes for every chest (no chest locking); the scan order is raster (the
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

## Lihzahrd temple (M1.4-WGEN-6)

The `LihzahrdTemple` pass (`WorldGen.cs:15834+`) — the buried jungle
temple, the last big generation structure:

- **The placement** (`:15840-15848`): the y between the rock layer and
  `maxTilesY − 600` (the port clamps to the height), the x under the
  jungle band's deep half (`dungeonSide`'s opposite in the C#).
- **The box** (`LihzahrdTemple`'s brick fill): the outer shell of the
  Lihzahrd brick (226) with the unsafe walls (87), the carved central
  hall + the side rooms with the safe walls (112), the brick pillars
  inside the hall.
- **The altar** (`AddLihzahrdAltar`, `:32071-32100`): the 3×2 altar (237)
  with the frames `i·18/j·18` and the 3-wide brick base row in the altar
  room at the hall's west end.
- **The traps**: the dart traps (137, the case-137 wiring variants the
  wiring trip fires — the trap frameY selects the variant) embedded in
  the hall's brick walls every 12 columns.
- **The chests**: the corners of the hall (`place_chest`'s intact quads).

Deviations: the C#'s multi-room crawl with the doors
(`HellFort_AttemptToPlaceDoor`-style), the teleporter pads and the Golem
summon (the altar interaction needs the Plantera downed — M5-HARDFORCE)
are the follow-ups; the box + 3-room shape approximates the C#'s crawl.

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
<<<<<<< HEAD
>>>>>>> origin/main

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
M3-EQUIP-4's plumbing), the prefixes (M3-PREFIX's
`GrantPrefixBenefits`), the vanity slots (the vanilla grants the
functional 0..9 only), the balloon combos beyond the id list (the
exclusive-combination items land as their ids appear).
=======
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

Deviations: the sit POSE (the sliced-leg render, `DrawSittingLegs`) is the
render follow-up — the sitting player draws standing, shifted -4 px
(`GetSittingOffsetInfo`'s seat adjustment); the sleep state is out; the
multiplayer seat-stack manager is out (one player); the chest/dresser
inventories stay the M3-CHEST-UI seam; the exotic sub-tile styles (the
metal/lava variants 8/42/56-64 &c.) wait for the full-registry generator —
the style-0 wooden slice places. The DrawYOffset fields are preview-only
(`TileObject.cs:864` — the only consumer), so the deferral is invisible.

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
>>>>>>> origin/main
