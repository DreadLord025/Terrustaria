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
| `Graphics/Light/ILightingEngine.cs` | `crates/tr-core/src/lighting/mod.rs` (the `LightEngine` trait seam) | done |
| `Graphics/Light/Lighting.cs` (LightMap/9-slice) | `crates/tr-core/src/lighting/` (`mod`/`params`/`blur`) | done |
| `Graphics/Light/TileLightScanner.cs` | `crates/tr-core/src/tile_light/` (`scanner.rs` the 147-case switch, `palette.rs` `FastRandom`/colours, `mod.rs` surface/hell/wall) | done |
| `WorldGen.cs` (`GenerateWorld`, the passes) | `crates/tr-core/src/worldgen/` (`mod`/`noise`/`terrain`/`caves`/`liquids`/`surface`) | partial |
| `WorldBuilding/*` (`WorldGenerator.cs`, `WorldGenSnapshot.cs`, `WorldBuilding/*`) | `crates/tr-core/src/worldgen/` | partial |
| `Framing.cs`, `HitTile.cs`, `TileMergeAttempt*`, `TileFrameCosmetic` grass family (82184-84836) | `crates/tr-core/src/framing.rs` | done |
| `Tile.cs` + tile tables/IDs | `crates/tr-core/src/tile.rs`, `tiledata/` (generated: `sets`/`emissive`/`merge`) | done |
| `GameContent/Drawing/TileDrawing.cs` `GetTileDrawData` (`:4527-6032`, the type switch) + `Main.cs` (tile draw path), `SceneMetrics`/`SceneState` (framing inputs) | `crates/tr-core/src/drawdata.rs` | partial (`GetTileDrawData`'s static layer complete incl. the palm/cactus/dye/cog world corrections; engine-driven parts are documented deviations — ANALYSIS.md "GetTileDrawData deviations") |
| `GameContent/Tiles/*` (waterfall/spouts) | `crates/tr-core/src/waterfall/` (`mod`/`draw`) | partial |
| tree growth/rendering | `crates/tr-core/src/tree.rs` | partial |
| wind (`windCounter*`, `GameContent/.../Wind*`) | `crates/tr-core/src/wind.rs` | partial |
| torch/lamp light seeding | `crates/tr-core/src/torch.rs` | partial |
| `Map/*`, `MapRenderer.cs`, `Main.Map*` | `crates/tr-core/src/map/` (`mod`/`colors`/`option`/`file`), `mapdata.rs` (colour tables) | partial |
| `IO/WorldFile.cs` (`.wld` save/load), world IO | `crates/tr-core/src/save.rs`, `wld.rs` (over `binio.rs`) | partial |
| `Main.cs` `DrawSunAndMoon` / `SetBackColor` (sun+moon placement & tint) | `crates/tr-core/src/sky.rs` | partial |
| `Lang.cs`, `Localization/*` | `crates/tr-core/src/localization/` (`mod`/`culture`/`format`/`json`) | partial |
| block place/break/edit rules (`WorldGen.*`, `HitTile`) | `crates/tr-core/src/edit.rs` | partial |
| `BinaryReader`/`BinaryWriter` (the LE primitives behind the world/map IO) | `crates/tr-core/src/binio.rs` (`Reader`/`Writer`) | done |

## Game / presentation (`crates/tr-app`)

| C# source (under `Terraria/`) | Rust target | State |
| --- | --- | --- |
| `Main.cs` (loop, update, draw orchestration) | `crates/tr-app/src/app/` (`loop.rs` the winit handler, `frame.rs` the `Main.Draw` orchestrator, `mod.rs` the `Main` state) | wip |
| `Program.cs` (entry) | `crates/tr-app/src/main.rs` (`fn main`, the bootstrap) | done |
| `Main.DrawBackground` underground tree (`DrawBackground_*`, `Main.cs:52798-53402`) | `crates/tr-app/src/render/bg.rs` (shared prologue `PassPrologue`) | done (classic style set; per-biome cave sets + 9-slice documented deviations) |
| `Main.DrawSurfaceBG`/`DrawSunAndMoon` (sky, clouds, parallax stack) | `crates/tr-app/src/render/sky.rs` (+ `biome.rs` for the style/alpha transitions) | partial |
| `Main.DoDraw_Waterfalls`, tile/liquid draw passes, `DrawTrees` | `crates/tr-app/src/render/{tiles,liquids,foliage}.rs` | partial |
| `Player.cs` | `crates/tr-app/src/player.rs` (physics), `collision.rs`, the 60 Hz tick in `world_sim/player.rs` | partial |
| `Player.cs` buff slots (`AddBuff` family `:5263-5480`, `UpdateBuffs` head `:9855-9876`, immune purge `:26012-26018`) | `crates/tr-app/src/buffs.rs` (`BuffSlots`, verbatim `Main.debuff`/pet/melee set tables) | done (slot engine; per-buff effects = M2-BUFF-3/4) |
| `Player.cs` regen engines (`UpdateLifeRegen` `:18942-19391`, `UpdateManaRegen`/`ApplyManaRegenerationDelay` `:19380-19562`) | `crates/tr-app/src/regen.rs` (`RegenState`/`RegenInputs`, wired in `update_survival`) | done (engine; DoT/aura/item inputs = M2-BUFF-3/5, M3-EQUIP) |
| `NPC.cs`, `NPCSpawnParams.cs`, `NPCSpawningFlagsForDualDungeons.cs` | `crates/tr-app/src/npc.rs`, `spawner.rs` (tick in `world_sim/npcs.rs`) | partial |
| `Projectile.cs` | `crates/tr-app/src/projectile.rs` (tick in `world_sim/projectiles.rs`) | partial |
| `Item.cs`, `GetItemSettings.cs`, `NewItemOwnership.cs` | `crates/tr-app/src/item.rs` (tick in `world_sim/items.rs`) | partial |
| `Collision.cs`, `Physics/*` | `crates/tr-app/src/collision.rs` | partial |
| `Rain.cs`, `Cloud.cs` | `crates/tr-app/src/rain.rs` (field + draw), the tick in `world_sim/weather.rs` | partial |
| `Map/*` (client-side map view) | `crates/tr-app/src/map.rs` | partial |
| `UI/*` (UIState screens) | `crates/tr-app/src/ui/uistate.rs` (the element kit) + `app/flow.rs` (`FrontScreen`/`FrontScreenBehaviours` ≈ `UIState`) | partial |
| `UICharacterSelect.cs`, `UICharacterListItem.cs`, `UICharacter.cs` | `crates/tr-app/src/ui/char_select.rs` | done (visual; deviations in ANALYSIS.md "Front-end screens (P6)") |
| `UIWorldSelect.cs`, `UIWorldListItem.cs`, `AWorldListItem.cs` | `crates/tr-app/src/ui/world_select.rs` | done (visual; same deviations) |
| `UICharacterCreation.cs` | `crates/tr-app/src/ui/char_creation.rs` | done (visual; same deviations) |
| `UIWorldCreation.cs`, `UIWorldCreationPreview.cs` | `crates/tr-app/src/ui/world_creation.rs` | done (visual; same deviations) |
| `UIVirtualKeyboard.cs` | `crates/tr-app/src/ui/keyboard.rs` | done (visual; desktop layer only) |
| `Main.LoadPlayers`/`LoadWorlds`, `ErasePlayer`/`EraseWorld`, `UICharacterSelect.UpdatePlayersList`/`UIWorldSelect.UpdateWorldsList` | `crates/tr-app/src/ui/save_list.rs` | partial (no `.plr` reader; metadata stand-ins) |
| `Main.DrawMenu` (legacy menu + confirms + credits, modes 5/9/40/3000) | `crates/tr-app/src/menu/` (`mod.rs` + `pages.rs` + `actions.rs`) | partial |
| `Main.DrawMenu` settings/multiplayer branches (`menuMode` 26, 28, 25, 252, 1125, 111, 2008, 1212, 1213, 1127, 27, 272727, 12, 13, 131, 30, 31, 889 — the P5 sub-screens; 27/272727 reachable via `TR_PAGE` only) | `crates/tr-app/src/menu/` (+ `crates/tr-app/src/ui/sliders.rs`) | done (visual only) |
| `IngameOptions.DrawValueBar` (+ `Main.DrawMenu`'s HSL colour-slider block, `Main.cs:51102-51284`) | `crates/tr-app/src/ui/sliders.rs` | done |
| in-game HUD/inventory (`Main.DrawInterface`/`DrawInventory`/`MouseText`, `ItemSlot.cs`, `ClassicPlayerResourcesDisplaySet`) | `crates/tr-app/src/ui/{mod,resources,cursor,tooltip,slot,inventory,chrome,text}.rs` | partial |
| `GameContent/UI/Minimap/*` (`MinimapFrame.cs`, `MinimapFrameManager.cs`, `MinimapFrameTemplate.cs`) + `DataStructures/SelectionHolder.cs` (`CycleSelection`) | `crates/tr-app/src/ui/minimap.rs` (+ the `MinimapFrame`/`MapScale` config keys in `settings.rs`) | done |
| `GameContent/UI/BigProgressBar/*` (`BigProgressBarHelper` 4-arg `DrawFancyBar` + `DrawHealthText`, `CommonBossBigProgressBar`) | `crates/tr-app/src/ui/bars.rs` | partial |
| `Main.DrawInventory` map-style icon row (`Main.cs:41055-41126`, `Images/Map_0..7`) | `crates/tr-app/src/ui/chrome.rs` (`draw_map_style_icons`) | done |
| `GameContent/UI/NewCraftingUI.cs` (+ `GameContent/UI/Elements/UIWrappedSearchBar.cs`, `GameContent/UI/Elements/UICreativeItemsInfiniteFilteringOptions.cs`, `UI/CraftingUI.cs` `DrawGridToggle`, `Utils.DrawSelectedCraftingBarIndicator`) | `crates/tr-app/src/ui/crafting.rs` (+ 9-slice/colour primitives `ui/panel.rs`, `ui/color.rs`) | partial |
| settings/config (`Main.*Settings`, `Preferences`) | `crates/tr-app/src/settings.rs` | partial |
| asset lookup tables (`TextureAssets`, `SoundID`, `ItemID` caches) | `crates/tr-app/src/registry.rs`, `assets.rs` | partial |
| `Audio/*`, `SoundEngine` | `crates/tr-app/src/audio.rs`, `sound.rs` | partial |
| `Net/*`, `NetMessage.cs`, `Netplay.cs`, `MessageBuffer.cs` | `crates/tr-app/src/net.rs`, `app/netglue.rs` | todo |
| `Main.Save*/Load*` character/persistence glue | `crates/tr-app/src/persist.rs`, `app/saveglue.rs` | partial |

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
