# PORT_MAP — C# → Rust correspondence

The single deliberate gap in the graphs: graphify indexes the C# source and the Rust
target **separately** and cannot link them across languages. This file *is* that link.
Keep it current — a wrong or missing row is the fastest way for an agent to
re-implement something that already exists, or to port the wrong file.

- **Source of truth (C#):** `decompiled/` (Terraria 1.4.5.8) and `decompiled-libs/ReLogic`
  (the game's own library — `.xnb`/LZX/DXT content, IO, OS, utilities).
- **Target (Rust):** `crates/`.
- **Graphs:** Terraria, ReLogic and the Rust target are each indexed, plus a merged graph
  of all three; see [`AGENTS.md`](../AGENTS.md) → "Knowledge graph (graphify)". In the
  merged graph ids are tagged `decompiled::`, `decompiled-libs::` and `repo::`.

**State legend:** `done` = ported and tested · `partial` = core present, gaps remain ·
`wip` = actively being ported · `todo` = not started · `n/a` = deliberately not ported.

## Core world / simulation (`crates/tr-core`)

| C# source (under `Terraria/`) | Rust target | State |
| --- | --- | --- |
| `Liquid.cs`, `LiquidBuffer.cs` | `crates/tr-core/src/liquid.rs` (sim + buffer), `liquid_draw.rs` (framing/render data) | done |
| `Lighting.cs` | `crates/tr-core/src/lighting.rs` (LightMap/9-slice), `tile_light.rs` (TileLightScanner) | done |
| `WorldGen.cs` | `crates/tr-core/src/worldgen.rs` | partial |
| `WorldBuilding/*` (`WorldGenerator.cs`, `WorldGenSnapshot.cs`, `WorldBuilding/*`) | `crates/tr-core/src/worldgen.rs` | partial |
| `Framing.cs`, `HitTile.cs`, `TileMergeAttempt*` | `crates/tr-core/src/framing.rs` | done |
| `Tile.cs` + tile tables/IDs | `crates/tr-core/src/tile.rs`, `tiledata.rs` (generated) | done |
| `Main.cs` (tile draw path), `SceneMetrics`/`SceneState` (framing inputs) | `crates/tr-core/src/drawdata.rs` | partial |
| `GameContent/Tiles/*` (waterfall/spouts) | `crates/tr-core/src/waterfall.rs` | partial |
| tree growth/rendering | `crates/tr-core/src/tree.rs` | partial |
| wind (`windCounter*`, `GameContent/.../Wind*`) | `crates/tr-core/src/wind.rs` | partial |
| torch/lamp light seeding | `crates/tr-core/src/torch.rs` | partial |
| `Map/*`, `MapRenderer.cs`, `Main.Map*` | `crates/tr-core/src/map.rs`, `mapdata.rs` (colours/exports) | partial |
| `IO/WorldFile.cs` (`.wld` save/load), world IO | `crates/tr-core/src/save.rs`, `wld.rs` | partial |
| `Main.cs` `DrawSunAndMoon` / `SetBackColor` (sun+moon placement & tint) | `crates/tr-core/src/sky.rs` | partial |
| `Lang.cs`, `Localization/*` | `crates/tr-core/src/localization.rs` | partial |
| block place/break/edit rules (`WorldGen.*`, `HitTile`) | `crates/tr-core/src/edit.rs` | partial |

## Game / presentation (`crates/tr-app`)

| C# source (under `Terraria/`) | Rust target | State |
| --- | --- | --- |
| `Main.cs` (loop, update, sky, draw orchestration) | `crates/tr-app/src/main.rs` | wip |
| `Player.cs` | `crates/tr-app/src/player.rs` | partial |
| `NPC.cs`, `NPCSpawnParams.cs`, `NPCSpawningFlagsForDualDungeons.cs` | `crates/tr-app/src/npc.rs`, `spawner.rs` | partial |
| `Projectile.cs` | `crates/tr-app/src/projectile.rs` | partial |
| `Item.cs`, `GetItemSettings.cs`, `NewItemOwnership.cs` | `crates/tr-app/src/item.rs` | partial |
| `Collision.cs`, `Physics/*` | `crates/tr-app/src/collision.rs` | partial |
| `Rain.cs`, `Cloud.cs` | `crates/tr-app/src/rain.rs` | partial |
| `Map/*` (client-side map view) | `crates/tr-app/src/map.rs` | partial |
| `UI/*`, `IngameOptions.cs`, menus | `crates/tr-app/src/menu.rs` | partial |
| in-game HUD/inventory (`Main.DrawInterface`/`DrawInventory`/`MouseText`, `ItemSlot.cs`, `ClassicPlayerResourcesDisplaySet`) | `crates/tr-app/src/ui/{mod,resources,cursor,tooltip,slot,inventory,chrome,text}.rs` | partial |
| settings/config (`Main.*Settings`, `Preferences`) | `crates/tr-app/src/settings.rs` | partial |
| asset lookup tables (`TextureAssets`, `SoundID`, `ItemID` caches) | `crates/tr-app/src/registry.rs`, `assets.rs` | partial |
| `Audio/*`, `SoundEngine` | `crates/tr-app/src/audio.rs`, `sound.rs` | partial |
| `Net/*`, `NetMessage.cs`, `Netplay.cs`, `MessageBuffer.cs` | `crates/tr-app/src/net.rs` | todo |
| `Main.Save*/Load*` character/persistence glue | `crates/tr-app/src/persist.rs` | partial |
| `Program.cs` (entry) | `crates/tr-app/src/main.rs` (`fn main`) | done |

## Content decoding (`crates/tr-content`) — ReLogic/XNA container formats

| Source (ReLogic / XNA) | Rust target | State |
| --- | --- | --- |
| `.xnb` container (`ReLogic/Content`) | `crates/tr-content/src/xnb.rs` | done |
| LZX decompression (`ReLogic/Content/Lzx*`) | `crates/tr-content/src/lzx.rs` | done |
| DXT texture decode | `crates/tr-content/src/dxt.rs` | done |
| `.xwb`/XACT sound bank + ADPCM (`ReLogic/.../Xact`) | `crates/tr-content/src/xact.rs`, `adpcm.rs`, `sound.rs` | partial |
| sprite font (`SpriteFont`) | `crates/tr-content/src/font.rs` | partial |
| texture surface | `crates/tr-content/src/texture.rs` | done |

## Renderer (`crates/tr-render`)

| Source | Rust target | State |
| --- | --- | --- |
| `SpriteBatch` / XNA draw calls | `crates/tr-render/src/sprite.rs`, `gpu.rs` | done |
| texture atlas packing (RebuildAtlas usage) | `crates/tr-render/src/atlas.rs` | done |

## Not ported (by design)

| C# source | Why |
| --- | --- |
| `Social/*` (Steam, Discord, etc.) | platform integrations; not needed for the core port |
| `BCrypt*` | third-party password hashing |
| `Achievements/`, `Initializers/` platform hooks | platform-specific |

## How to extend this file

When you start or finish a system:

1. Find the C# entry point in the matching graph:
   - Terraria: `cd decompiled && graphify explain "Terraria/<File>.cs::<Symbol>"`.
   - ReLogic: `cd decompiled-libs && graphify explain "<File>.cs::<Symbol>"` (paths are
     relative to `decompiled-libs/`, e.g. `ReLogic/ReLogic.Content/ContentManager.cs`).
2. Confirm the Rust counterpart exists once (not twice):
   `graphify query "<symbol or concept>" --graph graphify-out/merged-graph.json`.
3. Add a row here mapping **C# file(s) → Rust file(s)** and set the state.

Prefer one row per coherent system over one row per class — this file is a map, not a
line-by-line index (the graphs already give you file + line).
