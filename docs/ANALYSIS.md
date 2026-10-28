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
| World model & save format | `IO/WorldFile.cs`, `Tile.cs` | ~2 600 | ✅ | Header + section table + RLE tile stream parsed; chests/signs/NPCs skipped |
| Tile framing / autotiling | `WorldGen.TileFrame` (line 82852), `TileFrameCosmetic` | ~3 600 | 🟡 | Only a per-sheet "plain" frame; no merge/mask logic |
| Tile drawing | `GameContent.Drawing/TileDrawing.cs`, `WallDrawing.cs` | 9 700 + 1 300 | 🟡 | Simple quads; no paint, slopes, wind, glow, tile entities |
| Lighting | `Graphics/Light/LightingEngine.cs`, `LightMap`, `TileLightScanner` | ~5 300 | ⬜ | Currently a screen-space gradient only |
| Sky / backgrounds | `Main.cs` sky code, `Backgrounds/*` | ~3 000 | ⬜ | Solid gradient |
| Liquids | `Liquid.cs`, `LiquidRenderer.cs` | ~3 000 | ⬜ | Water/lava/honey/shimmer flags parsed but not drawn |
| Player physics | `Player.cs` | 55 600 | ⬜ | No player entity |
| Player rendering | `PlayerDrawLayers.cs` (+ 40 layer files) | ~12 000 | ⬜ | |
| Items & inventory | `Item.cs`, `UI/ItemSlot.cs`, `Recipe.cs` | ~70 000 | ⬜ | |
| NPC AI | `NPC.cs` + `NPCs/*` | 95 900 | ⬜ | ~700 NPC types, each with bespoke behaviour |
| Projectiles | `Projectile.cs` | 81 000 | ⬜ | ~1 100 types |
| Combat / damage | spread across `Player`, `NPC`, `Projectile`, `Item` | ~15 000 | ⬜ | |
| World generation | `WorldGen.cs`, `Generation/*` | ~45 000 | ⬜ | Passes, structures, dungeons, biomes |
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

Tile flags we map 1:1 into a packed 12-byte struct (`tr-core/src/tile.rs`): active,
half-brick, actuator, inactive, fullbright, liquid kind, liquid amount, slope, paint, wall.

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
- Non-important tiles store `frameX = frameY = -1` in the world; the original computes them
  every frame in `WorldGen.TileFrame`. Important tiles (torches, furniture, plants, trees…)
  keep their frames in the save file.
- Block tilesets also contain randomized variants selected per tile, which a single
  "plain frame" cannot reproduce — visible as repeated identical blocks in the prototype.

## Observations / risks

1. **Autotiling is the dominant visual factor.** One 16×16 frame per tile type produces
   repeating "cobblestone" seams. `TileFrameCosmetic` alone is ~3 400 lines with per-type
   merge groups; a faithful port is a multi-week task but is highly mechanical.
2. **Lighting is the second factor.** Without it the world is uniformly lit and caves show
   sky. The original engine uses a tile light scanner + per-corner colour interpolation.
3. **Content volume dominates long-term effort.** ~6 200 items, ~780 NPCs, ~1 100
   projectiles, ~750 tiles; most behaviour is hand-written per id, not data-driven.
4. **World generation** is a large but well-isolated subsystem (passes + structures) and can
   be ported independently once the world model supports all tile/object metadata.
5. **Networking** must match the exact 1.4.5.8 packet layout to interoperate; not required
   for single-player parity.
