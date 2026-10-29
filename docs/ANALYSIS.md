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
| Sky / backgrounds | `Main.cs` sky code, `Backgrounds/*` | ~3 000 | 🟡 | `Background_0` gradient + two parallax mountain layers (`Background_7`), sun/moon, stars, deterministic `Cloud_*` layer + the textured `Background_1/2` underground backdrop anchored to `worldSurface`; no biome backgrounds/rain/rock-layer backdrop yet |
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
| Map / minimap | `Map/*` | ~5 000 | 🟡 | Explored-tile bitset, original tile/wall colours, minimap and full map |

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

Not yet ported: the per-type special-case branches (grass edge override, moss, cactus,
vines, `TileFrameImportant` multi-tile objects), `TileID.Sets.TruncatesWalls`, paint and
coating. Those are listed in the roadmap.

`tools/GenTileData` parses `Terraria.ID/TileID.cs`, `Main.cs` and `WorldGen.cs` and emits
`tiledata.rs`; rerun it after touching those tables.

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

Ported in `tr-core/src/lighting.rs`:

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

`tr-core/src/tile_light.rs` is a case-for-case port of `TileLightScanner.ApplyTileLight`
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
  `FastRandom` in `tile_light.rs` reproduces the multiplier/mask, `NextBits`,
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
- The real parallax, cloud and underground background layers (`Main.DrawBackground`,
  `DrawSurfaceBG_*`, `DrawClouds_*`, `SkyManager`) are not ported. The blue is the game's
  own `Background_0.png` gradient anchored to `Main.worldSurface`; two seeded copies of
  `Background_7` are drawn as screen-space parallax mountain layers, and a deterministic
  `Cloud_*` layer uses half-speed horizontal parallax. Sun/moon/stars follow
  `DrawSunAndMoon`'s screen-space arc using `Sun`, `Moon_0` and `Star_0..3`. Biome-specific
  backgrounds, the foreground tree line and rain are missing; underground is a smooth
  per-column fade to cave darkness that starts at each column's first solid tile.
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

`tr-core/src/waterfall.rs` ports `Terraria.WaterfallManager`; `main.rs` runs it on the
60 Hz clock and emits the quads between the non-solid and solid tile passes
(`Main.DoDraw_Waterfalls`).

- `FindWaterfalls` keeps the 30-tick cadence (`findWaterfallCount < 30 && !forced`), the
  `waterfallDist = (int)(75 * gfxQuality) + 25` scan rectangle around the view, the
  `qualityMax` cap and the exact spout test: an active `halfBrick()` tile whose tile above
  has `liquid < 16` or is solid, with a neighbour holding `liquid > 160` while the other
  side is dry, non-solid and unsloped. The `196`/`460`/`717` fountain tiles spawn types
  `11`/`22`/`26` one row below when the cell underneath is open. The liquid kind check
  order is `tile2` (above) then `tile4` (right) then `tile3` (left), as in the source.
- `UpdateFrame` reproduces all five counters (regular 3-tick, slow 7-tick, rain,
  lava-rain and snow) plus `DoUpdate_AnimateDiscoRGB`. The `Main.mouseTextColor` flicker
  that the lava/shimmer light pulses use is owned by `lighting.rs` (`M1.5-LIGHT-1`) rather
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

Verification: unit tests in `waterfall.rs` (spout detection, lava kind, tile 196, frame
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

`Main.DrawSurfaceBG` is ported for the three biomes the generator has. The far and near
mountain layers (`Background_7`/`8` for forest, `35`/`36` for snow) are drawn at the
original's `bgScale` (1·2 and 1.15·2) and 0.15/0.20 parallax, followed by the biome's three
tree layers (`50..52` forest, `37..39` snow) at 1.25·2/1.31·2/1.37·2 and 0.40/0.43/0.49
parallax; the middle tree layer is mirrored horizontally, exactly as
`DrawSurfaceBG_Forest` does with `SpriteEffects.FlipHorizontally`. The ocean edge
(`x < 200` or `x > width - 200`) suppresses the tree line and adds `Background_28`, the
sea horizon with islands. Layer tops use
`backgroundTopMagicNumber = -(screenTop - 300) / (worldSurface * 16)` plus `ComputeScAdj`,
exactly like the original, and — like the original — depend only on the camera's Y, never
on the terrain under the camera; an earlier clamp to the local column's surface made the
skyline jump at every hill and was removed (BUG-018). The background batch keeps the
original's zoom-1 matrix, so the layers hold their screen size at any game zoom. Clouds
are world objects at a fixed altitude above `worldSurface` and drift with the wind.

Not ported: the biome cross-fade (`bgAlphaFarBackLayer`/`bgAlphaFrontLayer` for
corruption/crimson/jungle/hallow/mushroom), `DrawSurfaceBG_DrawChangeOverlay`, the
cloud-background (`cloudBG`) storm layers, the far-cloud depth layers and the ocean
water-line ambience.

The underground backdrop is a port of `Main.DrawBackground_SurfaceTransitionBackground` +
`DrawBackground_DirtBackground`: `Background_1` supplies the one-row transition at
`worldSurface`, `Background_2` tiles as 96x96 blocks from `worldSurface + 1` down, both
scrolled with `Main.caveParallax` (0.9) and covering the whole screen. Every 16x16 cell
takes the light of the world tile it covers, so lit ground stays lit and caves go black —
this is the original's screen-wide dirt backdrop, not a per-column fill. Two deviations
from the source: the light sample uses the tile actually behind the pixel (the original
samples on the unscaled grid, which leaves the light map at zoom > 1), and below the rock
layer the same dirt pattern continues where the original switches to `Background_3` and
the magma layers. Anchoring the old gradient per column made every player-placed row of
blocks paint a dark "wall" underneath itself; that artifact is gone.

## Sky and daylight

`Main.SetBackColor` (day `0..54000`, night `0..32400`) is ported to `color_of_the_skies`:

- Day `< 13500`: sunrise ramp `(230,220,220)·t + (25,35,35)`.
- Day `13500..37800`: pure white — this is why vanilla daytime tiles are lit at 100 %.
- Day `> 37800`: the two sunset ramps (orange, then dusk).
- Night: `(5,5,5) + 30·t`, floored at `minimalLight = 15` (moon-phase dependent in the
  original; we use the full-moon value).

`ApplyColorOfTheSkiesToTiles` pulls each channel 70 % towards its own value
(`(sum + 7·c) / 10`), which is the colour passed to the surface light seed.

## World generation and native saves

`tr-core/src/worldgen.rs` is a compact model of the shapes the original generator produces,
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
`Player_Hair_1`) at `(x-10, y-10)`, using the default skin/clothing colours and the
walk/jump leg frames. Torso and undershirt always use composite cell `(0,0)` (only the
head and arms animate); the hotbar, hearts, breath bubbles and boss bar use the original
`Inventory_Back`, `Heart`, `Bubble` and `UI_BossBar` textures with the classic layout. Arms use the composite layout from `PlayerDrawSet.CreateCompositeData`
(the six-column swing table for the front arm, back arm two rows lower); the near-side
(item-holding) arm is drawn from column 8 because the per-item shoulder rotation is not
modelled yet. Entity sprites (player and NPCs) snap their top-left to an integer screen pixel before the
instance is uploaded, so nearest sampling keeps exactly the same source texels while the
camera moves; the camera itself still interpolates smoothly. NPC sprite offsets align the
last opaque art row (not the padded frame) with the hitbox bottom, which is what removed
the slime/zombie hover. The camera follows the player; `TR_CAM`/`TR_PAN` keep the free
debug camera, `TR_NOPLAYER` hides the player, `TR_XY` places them at a tile and
`TR_HOLD=right,jump` holds a key for deterministic walk screenshots. The selected
hotbar block is drawn in the hand (14×14 tile frame) with a 0.28 s swing arc from raised-
behind to forward-down when mining, placing or attacking; the world cache key includes the
kind and quantised phase. Not ported yet:
the full `itemAnimation` timings and tool-specific swings, armour/accessory layers, the
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
7. **`tools/GenTileData` is stale.** The current decompilation writes `Main.`-prefixed
   assignments (`Main.tileSolid[0] = true;`) and lives under `decompiled/Terraria/ID/`
   and `decompiled/Terraria/Graphics/Light/`, while the generator's regexes expect the bare
   names and the old flat paths. Running it today emits empty tables (`sets: 0`,
   `tileSolid=0`) and would destroy `tiledata.rs`; it was reverted. Until the generator is
   updated, new tables that the generated file does not carry are extracted by hand and
   documented where they live (e.g. `torch.rs`). Fix the generator before the next
   data-table port.

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

`menu.rs` draws the original-style title screen (Logo texture, Mouse_Text buttons,
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

## Minimap and full map (M6)

`map.rs` keeps a per-tile explored bitset revealed around the player four times a
second (a 73x45 tile window, matching what the camera sees). Map colours come from the
original textures: the average of each tile's plain 16x16 frame and each wall's inner
cell (`assets::tile_color_avg`/`wall_color_avg`), with liquid tinting (water/lava/honey/
shimmer). Rendering is chunked: 8x8-tile blocks average their explored colours, so the
minimap (2 px/tile, top right) and the full map (`M`, 3 px/tile, centred on the player)
stay at a few thousand quads. Unexplored areas stay dark; the player is a white marker.

## Localization (M6)

`tr-core/src/localization.rs` ports `Terraria.Localization`'s core:
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
