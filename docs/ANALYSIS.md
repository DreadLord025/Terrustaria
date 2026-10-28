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
| World model & save format | `IO/WorldFile.cs`, `Tile.cs` | ~2 600 | ✅ | `.wld` header + section table + RLE tile stream parsed; chests/signs/NPCs skipped. Native `.trwld` writer/reader for generated worlds |
| Tile framing / autotiling | `WorldGen.TileFrame` (line 82852), `TileFrameCosmetic` | ~3 600 | 🟡 | Generic block chain + merge families ported; slopes/half-bricks framed; per-type grass edge/moss/vines/weird branches approximated |
| Wall framing | `Framing.WallFrame` (line 326) | 120 | ✅ | Full `wallFrameLookup` + `centerWallFrameLookup` port |
| Tile drawing | `GameContent.Drawing/TileDrawing.cs`, `WallDrawing.cs` | 9 700 + 1 300 | 🟡 | Quads, 9-sliced light, slopes, half bricks, tree canopies/branches, tile-3 plants; no paint, wind, glow, tile entities |
| Lighting | `Graphics/Light/LightingEngine.cs`, `LightMap`, `TileLightScanner` | ~5 300 | ✅ | Scanner + blur + sunlight; 4 modes, `GlobalBrightness`, 9/4-slice block smoothing; sun/moon/flicker still missing |
| Sky / backgrounds | `Main.cs` sky code, `Backgrounds/*` | ~3 000 | 🟡 | `Background_0` gradient + two parallax mountain layers (`Background_7`), sun/moon, stars, deterministic `Cloud_*` layer + per-column cave fade; no biome backgrounds/rain/underground textures |
| Liquids | `Liquid.cs`, `LiquidRenderer.cs` | ~3 000 | 🟡 | Water/lava/honey/shimmer drawn from `Misc/water_*` with animation, waves, opacity, depth tint and lighting; `Liquid.Update` flow simulation ported (active cells, 30 Hz, leveling); no `Waterfall_*` overlay or corner-height shoreline mesh |
| Player physics | `Player.cs` | 55 600 | 🟡 | Gravity, acceleration/friction, jump, 1-tile auto-step, AABB collision, ground-slope support, water buoyancy, HP/hurt/respawn; no half-brick/ceiling slopes, items or buffs |
| Player rendering | `PlayerDrawLayers.cs` (+ 40 layer files) | ~12 000 | 🟡 | Starter head/eyes/torso/shirt/arm/legs/pants/shoes/hair from the original strips; no armor/items/equipment layers |
| Items & inventory | `Item.cs`, `UI/ItemSlot.cs`, `Recipe.cs` | ~70 000 | 🟡 | 4-slot hotbar, mining/placing, counts and native save; no item entities, tools, recipes, chests or full inventory UI |
| NPC AI | `NPC.cs` + `NPCs/*` | 95 900 | 🟡 | First pass: hopping blue slime (gravity, collision, contact damage, player melee, HP bars); ~700 NPC types remain |
| Projectiles | `Projectile.cs` | 81 000 | ⬜ | ~1 100 types |
| Combat / damage | spread across `Player`, `NPC`, `Projectile`, `Item` | ~15 000 | 🟡 | Player/slime HP, hurt cooldown, knockback, contact/melee damage and respawn; no defense, crits, projectiles or drops |
| World generation | `WorldGen.cs`, `Generation/*` | ~45 000 | 🟡 | Native compact generator: surface/layers/caves/ores/ash/trees/plants, forest lake, snow biome, ocean beaches; no corruption/crimson/jungle/desert/hallow or structures |
| UI / menus | `UI/*`, `GameContent.UI.*` | ~35 000 | ⬜ | |
| Audio | `Audio/*`, XACT banks | ~4 000 | ⬜ | `.xsb`/`.xwb` banks extracted later |
| Networking | `Netplay.cs`, `MessageBuffer.cs` | ~12 000 | ⬜ | Protocol for 1.4.5.8 |
| Map / minimap | `Map/*` | ~5 000 | ⬜ | |

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
     colours extracted by `tools/GenTileData`.
   - `ApplyTileLight` — the `EMISSIVE` table (43 types: torches, candles, campfires, glowing
     mushrooms, crystals, lamps…) scaled by torch flicker.
   - `ApplyLiquidLight` — lava/honey/shimmer self-illumination.
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

Deliberate simplifications:

- Sun/moon/star light, `Main.mouseTextColor` flicker, per-type frame-dependent lights and
  invisible blocks/walls are not ported.
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

`LiquidRenderer.DrawNormalLiquids` is ported as a first pass. Each visible tile with
`liquid > 0` draws one 16x16 sprite from the original `Images/Misc/water_*` strips:

- water → `water_0` (style 0), lava → `water_1`, honey → `water_11`, shimmer → `water_14`;
- source X = 16 (interior column), the 80 px animation blocks for body tiles, and the
  surface block at `Y = 1280` for the top layer; 16 frames at 8 fps;
- opacity from `DEFAULT_OPACITY` (0.6 / 0.95 / 0.95 / 0.75), multiplied by the light map;
- partial tiles crop the source/destination height to `liquid / 255`;
- a per-column sine offset plus a shader UV ripple (per-instance `flags = 1`) breaks up
  the 16 px texture repetition; a small depth tint darkens deep water.

The system is completed by `tr-core/src/liquid.rs`, a port of `Liquid.UpdateLiquid` and
`Liquid.Update`: a 25 000-entry active-cell list (with the 50 000-entry overflow buffer),
one tenth pumped every second 60 Hz tick, per-cell gravity plus the six-neighbour
leveling passes (3/4/5/7-cell averages, edge openness, `Math.Round` banker's rounding),
and retirement after eight quiet passes. Falling water keeps the original's 254→255
quirk. Block edits wake the surrounding liquid cells, so a breach visibly drains.

Not ported: the `Liquid` wave mask shader, the `Waterfall_*` overlay drawn at
half-brick and waterfall-block spouts, the four-corner `VisibleLiquidLevel` render mesh
that gives the original's diagonal shorelines, water styles and shimmer sparkles. The
generator adds a forest lake and lava puddles so the pass is visible in the default
world; loaded `.wld` files expose their own liquids.

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

## Surface backgrounds

`Main.DrawSurfaceBG` is ported for the three biomes the generator has. The far and near
mountain layers (`Background_7`/`8` for forest, `35`/`36` for snow) are drawn at scale
1.6/1.85 and 0.15/0.20 parallax, followed by the biome's three tree layers
(`50..52` forest, `37..39` snow) at scale 0.85/0.90/0.95 and 0.40/0.43/0.49 parallax; the
middle tree layer is mirrored horizontally, exactly as `DrawSurfaceBG_Forest` does with
`SpriteEffects.FlipHorizontally`. The ocean edge (`x < 200` or `x > width - 200`) suppresses
the tree line and adds `Background_28`, the sea horizon with islands. Layer tops use
`backgroundTopMagicNumber = -(screenTop - 300) / (worldSurface * 16)`, which gives the
original's ~30 % vertical parallax, plus `ComputeScAdj`. Because the layers are drawn in
screen pixels instead of the original's zoomed sprite transform, each layer is lowered until
the texture's *painted ground band* (measured from the atlas alpha: the first fully opaque
row) starts at the local horizon; without that clamp the skyline floats above the terrain.

Not ported: the biome cross-fade (`bgAlphaFarBackLayer`/`bgAlphaFrontLayer` for
corruption/crimson/jungle/hallow/mushroom), `DrawSurfaceBG_DrawChangeOverlay`, the
cloud-background (`cloudBG`) storm layers, the far-cloud depth layers and the ocean
water-line ambience.

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
gravity `0.4`, max fall `10`, run `3`, ground/air acceleration `0.2/0.1`, jump `5.2`.
Movement is axis-separated AABB collision against `Main.tileSolid` tiles, with a one-tile
auto-step, simple water buoyancy and ground-slope support (slopes 1/2 follow the diagonal).
Half bricks collide only in the bottom 8 px (and the landing snap puts the feet on their
surface); ceiling slopes 3/4 test the AABB against the solid triangle above the diagonal.
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
`_1/_2` eyes, `_3` torso, `_6` shirt, `_7` back arm, `_10` legs, `_11` pants, `_12` shoes,
`Player_Hair_1`) at `(x-10, y-10)`, using the default skin/clothing colours and the
walk/jump leg frames. The camera follows the player; `TR_CAM`/`TR_PAN` keep the free debug
camera, `TR_NOPLAYER` hides the player and `TR_XY` places them at a tile. The selected
hotbar block is drawn in the hand (14×14 tile frame) with a 0.28 s swing arc from raised-
behind to forward-down when mining, placing or attacking; the world cache key includes the
kind and quantised phase. Not ported yet:
the full `itemAnimation` timings and tool-specific swings, full draw layers (front arm,
armor, accessories), buffs and the rest of `Player.cs`.

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

## NPCs and combat (M4 first pass)

`tr-app/src/npc.rs` adds a hopping blue slime (gravity, AABB collision, hop timer, wall
bounce, 30 HP, 12 contact damage), a flying demon eye that steers toward the player with a
sine bob (25 HP, 14 damage) and a zombie standing in for `AI_003_Fighters`: an 18×40 hitbox
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

Not ported: biome/event spawning rules and spawn rates, the other ~780 NPCs, AI state machines,
projectiles, loot/drops, defense/knockback resistances, crits, buffs/debuffs, boss logic
and multiplayer sync.

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
   still placeholders; the current sky + per-column cave fade avoids leaks and hard edges
   but does not cover every biome look.
4. **Content volume dominates long-term effort.** ~6 200 items, ~780 NPCs, ~1 100
   projectiles, ~750 tiles; most behaviour is hand-written per id, not data-driven.
5. **World generation** is a large but well-isolated subsystem (passes + structures). The
   compact generator and native save give a stable test world; full parity is M5.
6. **Networking** must match the exact 1.4.5.8 packet layout to interoperate; not required
   for single-player parity.

Note: tile id **0 is dirt**, not air. The asset usage scan must include it; the active flag
is the only air test.
