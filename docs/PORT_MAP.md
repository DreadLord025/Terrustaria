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
| `ObjectData/TileObjectData.cs` + `TileObject.cs` (`CanPlace`/`Place`), KillTile `Check*` cascade (`WorldGen.cs`) | `crates/tr-core/src/tileobject.rs` (registry, `can_place`/`place`, `object_tiles_at`), `crates/tr-core/src/edit.rs` (`place_item_tile`, `kill_tile` cascade) | partial (wooden slice 10/11/14/15/18/21; full-registry generator is the follow-up — ANALYSIS "Tile objects (M1.4-TO)") |
| `WorldGen.OpenDoor`/`CloseDoor` + the `Player.LookForTileInteractions` right-click chain | `crates/tr-core/src/edit.rs` (`interact_at`/`open_door`/`close_door`), hook in `crates/tr-app/src/world_sim/editing.rs` (`update_editing`) | done for doors (door net message not wired — ANALYSIS "Tile objects (M1.4-TO)") |
| `BinaryReader`/`BinaryWriter` (the LE primitives behind the world/map IO) | `crates/tr-core/src/binio.rs` (`Reader`/`Writer`) | done |

## Game / presentation (`crates/tr-app`)

| C# source (under `Terraria/`) | Rust target | State |
| --- | --- | --- |
| `Main.cs` (loop, update, draw orchestration) | `crates/tr-app/src/app/` (`loop.rs` the winit handler, `frame.rs` the `Main.Draw` orchestrator, `mod.rs` the `Main` state) | wip |
| `Program.cs` (entry) | `crates/tr-app/src/main.rs` (`fn main`, the bootstrap) | done |
| `Main.DrawBackground` underground tree (`DrawBackground_*`, `Main.cs:52798-53402`) | `crates/tr-app/src/render/bg.rs` (shared prologue `PassPrologue`) | done (classic style set; per-biome cave sets + 9-slice documented deviations) |
| `Main.DrawSurfaceBG`/`DrawSunAndMoon` (sky, clouds, parallax stack) | `crates/tr-app/src/render/sky.rs` (+ `biome.rs` for the style/alpha transitions) | partial |
| `GameContent/Skies/AmbientSky.cs` (SkyEntity + 18 creature classes + the manager) | `crates/tr-app/src/ambience.rs` (state + update + draw fns; render hooks in `render/sky.rs`/`bg.rs`) | done |
| `GameContent/Ambience/AmbienceServer.cs` (spawn scheduler) | `crates/tr-app/src/ambience.rs` (`Ambience::update_scheduler`) | done |
| `GameContent/Ambience/SkyEntityType.cs` | `ambience::SkyEntityType` | done |
| `GameContent/NetModules/NetAmbienceModule.cs` (spawn packet, seed) | inlined SP loopback — the seed is one draw from the scheduler's `Main.rand` stand-in (`Ambience::update_scheduler`) | done (MP when it lands) |
| `Graphics/Effects/SkyManager.cs` depth segments (`DrawToDepth`/`DrawRemainingDepth`) | `render/sky.rs` boundary calls with a local tracker (`amb_last`) + `ambience::draw_range_min/remaining` | done |
| `GameContent/Ambience/AmbientSkyDrawCache.cs` | `Underworld[5]` dead in 1.4.5.8 (no `SetUnderworldInfo` caller); ocean-line producer `Main.cs:59983` deferred (Q4) | partial |
| `Main.DoDraw_Waterfalls`, tile/liquid draw passes, `DrawTrees` | `crates/tr-app/src/render/{tiles,liquids,foliage}.rs` | partial |
| `Player.cs` | `crates/tr-app/src/player.rs` (physics), `collision.rs`, the 60 Hz tick in `world_sim/player.rs` | partial |
| `Player.cs` buff slots (`AddBuff` family `:5263-5480`, `UpdateBuffs` head `:9855-9876`, immune purge `:26012-26018`) | `crates/tr-app/src/buffs.rs` (`BuffSlots`, verbatim `Main.debuff`/pet/melee set tables) | done (slot engine; per-buff effects: DoT landed — M2-BUFF-3 (the per-frame `buff_immune` reset `:25561-25565` + expired sweep `:25608-25616` ride `Player::update`), stat/movement = M2-BUFF-4) |
| `Player.cs` regen engines (`UpdateLifeRegen` `:18942-19391`, `UpdateManaRegen`/`ApplyManaRegenerationDelay` `:19380-19562`) | `crates/tr-app/src/regen.rs` (`RegenState`/`RegenInputs`, wired in `update_survival`) | done (engine + the DoT drain: the −600/5 fast-lane, the `DotDeath` reasons and the per-branch derivation landed — M2-BUFF-3; aura/item inputs = M2-BUFF-5, M3-EQUIP) |
| `NPC.cs`, `NPCSpawnParams.cs`, `NPCSpawningFlagsForDualDungeons.cs` | `crates/tr-app/src/npc.rs`, `spawner.rs` (tick in `world_sim/npcs.rs`) | partial |
| `NPC.cs` buff handling (`AddBuff`/`DelBuff`/`FindBuffIndex` `:89910-90023`, the `UpdateNPC` buff block `:90373-90377` — set flags `:91965-92149`, clear expired `:91890-91909`, `DOTTally`/ApplyDOTs `:91594-91774`, `GetHurtByDebuff` `:91864-91888`), `NPCID.Sets.DebuffImmunitySets` (`NPCID.cs:6075+`) | `crates/tr-app/src/npc_buffs.rs` (`NpcBuffs`, ticked from the head of `Npc::update`) | done (roster slice + the derived rules 20⇒30/375, 69⇒36; deviations in ANALYSIS "DoT debuffs (M2-BUFF-3)") |
| `NPC.SpawnNPC`/`Spawner` call discipline (`:80170-80176`, the `SwapRandom("SpawnNPC")` stream `Main.cs:17886`, `noSpawnCycle` `:6179`, `CanSpawnEnemiesNear` `:349`) | `spawner.rs::spawn_npc_call` + `world_sim/npcs.rs` (the per-update stream advance, the latch, the dead-player gate) | done (engine; biome tables = M4-SPAWN-2) |
| `Wiring.cs` (the four wire colours, `TripWire` `:525`, `HitWire` `:837`, `HitWireSingle` `:996`, `HitSwitch` `:259`, `CheckMech`/`UpdateMech` `:455/:145`) | `tr-core/src/wiring.rs` (`Wiring` + `WireState`, the colour bits on `World`) | done (engine; device entity effects = M7-WIRE-2, multi-tile actuation = M7-WIRE-3) |
| `Wiring.cs` actuation (`Actuate`/`ActuateForced` `:391-416`, `DeActive`/`ReActive` `:3195-3232`) + `WorldGen.CanKillTile` (`:70237+`) | `tr-core/src/wiring.rs` (`de_active`/`re_active`/`can_kill_tile` + the sets) + `edit.rs::actuation_square_frame` + the consumers (`collision.rs` `!inActive`, `liquid.rs` `nactive()`, the `actColor` ghost tint) | done (ANALYSIS "Actuation & wire rendering (M7-WIRE-3)") |
| `Main.DrawWires` (`:58218-58701`, the `WireNew` masks, the actuator plate, `CanShowWireStuffHere`) | `tr-app/src/render/wires.rs` (`draw_wires`), hooked in `app/frame.rs` before the over-wire projectiles | done (defaults for `HideWires`/builder-acc; the tile-423 rect content-gated) |
| `Wiring.cs` teleporters & pumps (`Teleport` `:3120-3189`, the pad registration `:1066-1082`, `XferWater` `:475-520`, the pump registration `:2046-2125`) | `tr-core/src/wiring.rs` (the pad pairs + `xfer_water`) + `world_sim/wiring.rs` (the entity movement) | done (engine + app half; the Teleport effects = M7-WIRE-3 polish) |
| `Wiring.cs` statues (`HitWireSingle` case 105 `:2125-2440`, `NPC.MechSpawn` `:7420`, `Item.MechSpawn` `Item.cs:49385`) | `tr-core/src/wiring.rs::statue_trigger` + `world_sim/wiring.rs` (the caps) + `wiring_stub.rs` | done (the ported-kinds subset; the trap projectiles = M4-PROJ families) |
| `Projectile.cs` | `crates/tr-app/src/projectile.rs` (tick in `world_sim/projectiles.rs`) | partial |
| `Projectile.cs` pool + immunity (`NewProjectileSetup` `:10823-10880`, `Damage_PVE` `:12562-12624`, the on-hit application `:14087-14119`, `DecrementLocalImmuneTimeCounters` `:17258`) | `crates/tr-app/src/projectile.rs` (`ProjectilePool`: 1000 slots, per-owner/ID-static/local immunity) | done (engine + shuriken; families = M4-PROJ-2..11) |
| `WorldGen.cs` falling blocks (`SpawnFallingBlockProjectile` `:86039-86090`, `GetSandfallProjData` `:86514-86570`, `BlockBelowMakesSandFall`/`AllowsSandfall` `:86585-86624`) + `AI_010`'s tile-fall mode | `tr-core/src/falling.rs` (`check_falling`, the data map, the trigger predicates) + `tr-app/src/projectile.rs` (`falling_tile`, `spawn_falling`, the 0.41-gravity flight + the landing placement) + `world_sim/editing.rs` (the break hook) | done (ANALYSIS "Falling blocks (M7-FALL-1)") |
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
