# Task Status — Multi-Agent Coordination

> **⚠️ DEPRECATED (2026-11-01) — frozen archive.** Task claiming moved to the
> **branch-per-task workflow**: take a task → `git checkout -b <task-slug>` → work →
> merge into `main` resolving conflicts thoughtfully (see `agents.md` "READ THIS
> FIRST"). The branch list (`git branch -a`) is the live coordination record; this
> file is kept only as the historical log up to the 2026-11-01 architecture refactor.

**Last updated:** 2026-11-01T18:30Z  
**Purpose:** Track all porting tasks, assign to agents, avoid duplicate work

---

## 📋 Master Task Matrix

### Milestone 1.x — Rendering & Core Systems

| ID | Task | Status | Assigned To | Notes |
|----|------|--------|-------------|-------|
| M1.4-BG-1 | Background gradients (forest, mountain) | ❌ Pending | - | - |
| M1.4-BG-2 | Cloud generation and parallax | ❌ Pending | - | - |
| M1.4-BG-3 | Sun, moon, stars rendering | ❌ Pending | - | - |
| M1.4-BG-4 | Day-night cycle and sky colors | ❌ Pending | - | - |
| M1.4-BG-5 | Rain and snow systems | ❌ Pending | - | - |
| M1.4-BG-6 | Biome background variants | ❌ Pending | - | - |
| M1.4-BG-7 | Screen-wide cave backdrop + water pass restored | ✅ Done | Agent-Qoder | `7d9710a` anchored the backdrop to `worldSurface`; `70bf1eb` replaced the gradient with the real `Background_1/2` tiles + per-cell light (`DrawBackground_SurfaceTransitionBackground`/`DrawBackground_DirtBackground`); `Lookups` resolves `Misc/water_*` so `DrawNormalLiquids` emits water again. `M5-WSURF` then fixed `worldSurface` itself, which had been the mean column height and floated the sheet ~60 tiles high |
| M1.4-BG-8 | Walls: 32x32 sprites + corner light | ✅ Done | Agent-Qoder | `e4efa59`: `Main.DrawWalls` geometry (32x32 at 8 px up/left), own pass before tiles, `Lighting.GetCornerColors` per cell with a flat fast path, black-wall skip |
| M1.4-BG-9 | Surface parallax offsets and X phase | ✅ Done | Agent-Qoder | `f698971`: `pushBGTopHack`, `Background_50..52` -50, 2.68 third layer, desert 2.5/2.68, `bgStartX` from the screen's left edge |
| M1.4-BG-10 | Rock layer / magma / hell backgrounds | ✅ Done | Agent-GLM | `[M1.4-BG-10] Port the rock layer and hell background passes`: `DrawBackground` order restored (transition → dirt + its `Background_4` band → underworld black box → rock → magma), `DrawBackground_DrawRockLayer`/`_DrawMagmaLayer`/`_DrawMagmaTransition`/`_DrawUnderworldBlackBox` with the faithful cell skip chains (`ShouldDrawBackgroundTileAt`/`WallLightAt`/`InWorld`), `GetRockTransitionPoint` now truncates each row before the difference, `hellBackStyle` on `World` (rolled at generation, `WorldGen.cs:8247`) picking `Background_125..127`/`185..187`, `magmaBGFrame` 8-tick cycle. 9/4-slice smoothing collapsed to flat cells like the dirt pass; `undergroundBackground` pinned to the classic set (caveBackStyle/SceneMetrics unmodelled); backdrop cells are light/wall-gated, so natural walls hide them exactly like the original |
| M1.4-BG-11 | Sun/moon phases, clouds, stars | 🟡 Partial | Agent-DeepSeek | Sun/moon phases/scale/rotation done under `M1.4-BG-16` (see that row); cloud `Parse`/`GetParallax` and star density still pending |
| M1.4-BG-12 | Biome bgStyle selection + cross-fade | ✅ Done | Agent-Qoder | `91888cc`: `bgStyle`/`bgDelay`/`instantBGTransitionCounter` + 16-slot `bgAlphaFrontLayer`/`bgAlphaFarBackLayer`, 0.05 per-frame fade, 30/60-frame delay + 60-frame cooldown; beach→4, desert→2, snow→7, else 0 (same camera-biome tests as the water style). Full forest/desert/ocean/snow layer stack incl. desert mountains `22/23`; timer/alpha tests in `bg_transition_tests`. `SceneMetrics`-only styles, saved tree sets and `DrawChangeOverlay` remain documented deviations |
| TILE-DRAW-1 | Per-type draw metrics (GetTileDrawData) | 🔄 In Progress | sub-agent | `tr-core/src/drawdata.rs`: torch 20x20/top 4, plants 20+flip, herbs, tables, candles, palm, cactus |
| TILE-DRAW-2 | Torch flames + glow masks | ❌ Pending | - | `TextureAssets.Flames`, glow mask table |
| TILE-DRAW-3 | Wind: swaying plants, tree wind, leaves | 🔄 In Progress | sub-agent | `tr-core/src/wind.rs` + `DrawGrass` path, `EmitTreeLeaves` |
| TILE-DRAW-4 | Paint / coatings / echo | ❌ Pending | - | `TilePaintSystemV2`, wall paint, invisible flags |
| M1.5-LIGHT-1 | Star/Moon/Light flicker | ✅ Done | Agent-Nemotron | Ported mouseTextColor flicker (torch/lava/shimmer/crystal-wall light oscillation), committed |
| M1.5-LIGHT-2 | Per-type tile lighting | ✅ Done | Agent-DSV4 | Full `TileLightScanner.ApplyTileLight`/`ApplyWallLight` port: all 147 cases, paint-coloured light, disco/demon/shimmer dynamics, per-tile `FastRandom`. Scanner core in `4d42f34`; app wiring (`demonTorch` ramp, `DiscoRGB`, per-frame scanner seed, `LightParams`) in `d780b99` |
| M1.5-LIQUID-1 | Waterfall overlays | ✅ Done | Agent-DeepSeek | `waterfall.rs` ports `FindWaterfalls`/`UpdateFrame`/`DrawWaterfall`; wired between the non-solid and solid passes; lake A/B pixel-identical; see ANALYSIS.md "Waterfall overlay" |
| M1.5-LIQUID-2 | Water styles and shimmer | ❌ Pending | - | - |

### Milestone 2 — Player

| ID | Task | Status | Assigned To | Notes |
|----|------|--------|-------------|-------|
| M2-DRAW-1 | Player draw layers (armor/accessories) | ❌ Pending | - | - |
| M2-DRAW-2 | Composite arm rotation | ❌ Pending | - | - |
| M2-ANIM | Full itemAnimation/use timings | ❌ Pending | - | - |
| M2-FRAME | Player locomotion frames + held-item gate | ✅ Done | Agent-DeepSeek | `player.rs` ports `Player.PlayerFrame`: idle row 0 / airborne row 5 / walk rows `7..=19` (`legFrameCounter += |vx| * 1.3`, wrap 8.0), body copies the leg row, item-use rows `1→2→3`; held item drawn only while swinging (`itemAnimation > 0`); world render camera snapped to `round(cam*zoom)/zoom` (kills the moving pixel-grid shimmer); net `VERSION = 2` (`leg_frame`/`body_frame`). 3 new tests. Not ported: hold-style items, wings/mount/pulley branches |
| M2-BUFF | Buffs, hooks, mounts, gravity flips | ❌ Pending | - | - |

### Milestone 3 — Items & Crafting

| ID | Task | Status | Assigned To | Notes |
|----|------|--------|-------------|-------|
| M3-TOOL | Tool tiers and durability | ❌ Pending | - | - |
| M3-TORCH | Torch attachment frames (`CheckTorch`) | ✅ Done | Agent-DeepSeek | `tr-core/src/torch.rs`: frames 0/22/44, tall 66 offset, removal when unsupported; run from `place_tile` and `reframe_area` |
| M3-EQUIP | Equipment slots and paint | ❌ Pending | - | - |
| M3-ACTUATOR | Actuators | ❌ Pending | - | - |
| M3-CRAFT | Recipe.cs crafting menu | 🟡 Partial | Agent-controller + subagents | Panel chrome done (P3: `ui/crafting.rs`, commits b030f12..HEAD); recipe data/interaction later (ItemSlot contexts 4/33) |
| M3-PREFIX | Item prefixes system | ❌ Pending | - | - |
| M3-INV | Full inventory UI | ❌ Pending | - | - |

### Milestone 4 — NPCs & Combat

| ID | Task | Status | Assigned To | Notes |
|----|------|--------|-------------|-------|
| M4-NPC-1 | Remaining NPC types (780 total) | ❌ Pending | - | ~780 AI implementations |
| M4-SPAWN | NPC spawn/biome/night rules | ✅ Done | Agent-Nemotron | `tr-app/src/spawner.rs`: `Spawner.TrySpawnAnNPC` port — `GetSpawnRate` depth/night/town/nearby multipliers + clamps, `GetSpawnArea`/`FindSpawnTile` 50-candidate scan, `CheckNotSpawningOnScreen`, `HasTileSpawnSpace` (2×3), `FindGroundTile`, `PostCheckChosenSpawnTile`, surface day-slime/night-eye/zombie selection; wired into `update_npcs` per 60 Hz tick |
| M4-LOOT | Full loot tables/item drops | ❌ Pending | - | - |
| M4-PROJ | Projectile system (~1100 types) | ❌ Pending | - | - |
| M4-DAMAGE | Damage types/defense/crits | ❌ Pending | - | - |
| M4-BOSS | All bosses | ❌ Pending | - | - |
| M4-TOWN | Town NPCs and shops | ❌ Pending | - | - |

### Milestone 5 — World Generation

| ID | Task | Status | Assigned To | Notes |
|----|------|--------|-------------|-------|
| M5-BIOME | Corruption/Crimson/Jungle/Hallow | ❌ Pending | - | - |
| M5-WSURF | `worldSurface`/`rockLayer` from `TerrainPass` | ✅ Done | Agent-Qoder | `worldgen.rs`: `worldSurface = max(preCarve surfaceY) + 25` and `rockLayer` snapped up to a multiple of 6, matching `TerrainPass.ApplyPass` (was the mean surface, which floated the cave backdrop/surface parallax ~60 tiles high). See ANALYSIS.md "Surface backgrounds" (BUG-019) |
| M5-OCEAN | Full ocean biome | ❌ Pending | - | - |
| M5-DESERT | Desert structures and loot | ❌ Pending | - | - |
| M5-DUNGEON | Dungeons | ❌ Pending | - | - |
| M5-TEMPLE | Jungle temples | ❌ Pending | - | - |
| M5-LIVING | Living trees | ❌ Pending | - | - |
| M5-HARDFORCE | Hardmode spread | ❌ Pending | - | - |
| M5-CHEST | Chest generation and loot | ❌ Pending | - | - |

### Docs & Presentation

| ID | Task | Status | Assigned To | Notes |
|----|------|--------|-------------|-------|
| DOC-README | Player-facing README + final screenshot set | ✅ Done | Agent-DeepSeek | `README.md` rewritten for a non-technical player (what the project is, what you can do, how much faster it runs now, save location, env vars, build-from-your-own-`Content` note, licence/author). 17 regenerated screenshots in `docs/screenshots/` (old `spout_noon.png` removed); captions unchanged |
| DOC-GRAPHIFY | Knowledge-graph (graphify) setup for the 1:1 port | ✅ Done | Agent-DeepSeek | Indexed Terraria C# (`decompiled/`), ReLogic C# (`decompiled-libs/`) and Rust (`crates/`) + a merged graph; query workflow and recipes in `AGENTS.md`; C#→Rust map in `docs/PORT_MAP.md`; `/graphify-out/` ignored. Commits `04ea3e4`, `c89e236` |

### Milestone 6 — Audio & Polish

| ID | Task | Status | Assigned To | Notes |
|----|------|--------|-------------|-------|
| M6-MUSIC | XACT/WMA music playback | ✅ Done | Agent-Audio | `918b5a5`: XWB/XSB reader + MS-ADPCM decode (`tr-content/xact.rs`, `adpcm.rs`) and `audio::MusicSystem` (`Main.UpdateAudio`/`LegacyAudioSystem.UpdateCommonTrack` fade/selection/loop). See ANALYSIS.md "Audio (M6)" |
| M6-3DAUDIO | 3D audio attenuation | ✅ Done | Agent-Audio | `918b5a5`: `sound.rs` ports `LegacySoundPlayer.PlaySound` (2500 px falloff, pan, ambient volume); positional `play_at` + listener wiring in `main.rs` |
| M6-LOCALIZE | Localization system | ✅ Done | Agent-DeepSeek | `tr-core/src/localization.rs`: `GameCulture`/`LocalizedText`/`VariableText`/`LanguageManager` (JSON loader, `$`-variants, `{$Key}` copy commands, `{Name}`/conditions, `format_net`); 11 tests incl. the shipped `en-US` files. UI wiring still pending |

### Milestone 7 — UI (1:1 layout)

| ID | Task | Status | Assigned To | Notes |
|----|------|--------|-------------|-------|
| M7-UI-1 | Menu framework (DeathText, item scale, 5-pass shadow, hover) | ✅ Done | Agent-DeepSeek | `menu.rs`: `Main.DrawMenu` geometry + colours, 9 tests; see docs/UI.md |
| M7-UI-2 | Settings screens (menuMode 11 / 112 / 1112 / 1111) | ✅ Done | Agent-DeepSeek; P5: Agent-controller + subagents | General/Interface/Video rows, labels and toggles. Whole-UI P5: the 17 sub-screens ported visual-only — volume 26 / parallax 28 `DrawValueBar` sliders, cursor 1125 + colour screens 25/252 (HSL), resolution 111, effects 2008, language 1212/1213, controls 1127, keybind lists 27/272727, five live Interface selector rows — shared chrome in `ui/sliders.rs`. Commits `0e562e3..HEAD`. Deviations in ANALYSIS.md "Menu sub-screens (P5)"; see docs/UI.md, docs/PORT_MAP.md |
| M7-UI-3 | Main menu screen (logo, items, version, links) | ✅ Done | Agent-DeepSeek; P5: Agent-controller + subagents | Logo at (w/2,100) with scale/rotation easing, seven vanilla rows, `v1.4.5.8` in Mouse_Text. Whole-UI P5 multiplayer alignment: menuMode 12/13/131/30/31/889 at the original geometry (Join first, blinker input rows, port gate 1..=65535, password masking; old boxed Field UI deleted), commit `5cf6684` |
| M7-UI-4 | Inventory / ItemSlot cells exact sizes | ✅ Done | Agent-DeepSeek | Faithful `ItemSlot.Draw` port (`ui/slot.rs`): per-context `Inventory_Back*` background, `Size()*Main.inventoryScale` cell, icon (max 32), stack text at `(10, 26+LineSpacing)*scale`, hotbar digits ("10"→"0", selected `-2`), `Extra_54` empty-equip glyphs. Full `Main.DrawInventory` panels in `ui/inventory.rs` (grid/coins/ammo/equipment/misc/trash). Commits `213a7d4`,`353d917`,`35c3438`,`f8e01a5`. See docs/UI.md, deviations in ANALYSIS.md |
| M7-UI-6 | HUD: life/mana/breath/hotbar/tooltips | ✅ Done | Agent-DeepSeek | Whole-UI P1: `ui/` module extracted from `main.rs`, vanilla `Cursor_0`, hearts + mana stars, source-exact "Life x/y"/"Mana" text, `Main.mouseTextColor` tint. Commits `1c1eb1d`,`ea4ee14`,`e1265dd`,`7c17d76`,`704dec2`,`a08401a`,`16f0264`. Sources: `ClassicPlayerResourcesDisplaySet`, `Main.DrawCursor`, `Main.MouseText` |
| M7-UI-7 | Inventory grid, trash/coins/ammo, equipment | ✅ Done | Agent-DeepSeek | `ui/inventory.rs` + `ui/chrome.rs` at the original geometry: grid pitch 56/scale 0.85, coins/ammo scale 0.6, equipment columns at `screenWidth-186/-139/-92` with the loadout-1 tint, misc page pitch 47, trash `(448,258)`, page tabs, defence counter (beside the last accessory row), guide text, PVP icons. Commits `35c3438`,`8f5651d`,`f8e01a5` |
| M7-UI-8 | Minimap frame + boss bar | ✅ Done | Agent-controller + subagents | Whole-UI P4: `ui/minimap.rs` (9 `MinimapFrame` styles, anchor `(292,90)`, 244×244 backdrop, three zoom buttons with hover sheets + hitboxes, zoom 1.05/0.975/1.025 clamp 0.2..3), `Main.mH = 256` equipment push-up, `Main.DrawInventory`'s map-style icon row (`ui/chrome.rs`, clicks open the map / set `mapStyle`), and the faithful `BigProgressBarHelper.DrawFancyBar` (`ui/bars.rs`: bar `(572,839,456,22)`, even fill, head icon `(557,849)`, 5-pass `Item_Stack` text). Helper geometry pinned by source-cited oracles; `TR_MAPSTYLE` dev hook. Commits `7641e15..HEAD`. Deviations in ANALYSIS.md "Minimap frame and boss bar (P4)"; see docs/UI.md, docs/PORT_MAP.md |
| M7-UI-9 | Front-end: character/world select + creation, keyboard, credits | ✅ Done | Agent-controller + subagents | Whole-UI P6: `ui/char_select.rs`+`ui/char_creation.rs`+`ui/world_select.rs`+`ui/world_creation.rs`+`ui/keyboard.rs` (+ `ui/save_list.rs`, `ui/uistate.rs` = the `UIElement.cs:306-322` layout pass) — full geometry oracles, real player preview, real `.trwld` generation on Create, mode-40 discard round-trip, front XOR menu invariant (exhaustive test), `TR_UI`/`TR_FAKE_*` dev hooks. Legacy confirms 5/9/40 + credits 3000 in `menu/pages.rs`. Commits `02182dc..043e11c`. Deviations in ANALYSIS.md; see docs/UI.md, docs/PORT_MAP.md |
| REFACTOR-1 | Architecture refactor: split the god-classes | ✅ Done | Agent-controller + subagents | `main.rs` 10 497 → **86 lines** (bootstrap); new `app/` (mod/flow+FrontScreenBehaviours trait ≈ C# UIState/clicks/session/settings_ui/netglue/saveglue/loop/frame), `render/` (sky/bg/tiles/liquids/entities/foliage/debug, 4 493 lines), `world_sim/` (step FixedStepper + 9 sims), `item.rs`/`biome.rs`; `menu.rs` 4 187 → `menu/` (mod/pages/actions). tr-core: generator `tools/GenTileData` fixed against the current decompilation (regen diff empty) + `tiledata/` split; `map/` `tile_light/` (scanner split by theme) `lighting/` (+ `LightEngine` trait ≈ `ILightingEngine`) `waterfall/` `worldgen/` (passes; 4-world FNV byte-proof) `localization/`; `binio.rs` unified readers. ui dedup: `in_rect`/`hover_pair`/`push_whole`/`border_string`/`ui/cards.rs`. Determinism pinned (RNG streams, G6 key); post-refactor screenshot + SHA256 pair PASS. Plan `docs/superpowers/plans/2026-11-01-architecture-refactor.md`; commits `9c84351..HEAD` |
| M7-MAP | World map: per-tile colours, zoom/WASD pan, biome `MapBG`, head icon, `.map` persistence | ✅ Done | Agent-DeepSeek (SDD) | `Map/WorldMap`+`MapHelper` port: `tr-core/src/map.rs` (`MapTile`, `.map` v326 layout) + `tr-core/src/mapdata.rs` (colour tables) + `tr-app/src/map.rs` (reveal from the light map, world-sized texture, `Main.DrawMap` geometry, `MapBG`, head icon, deflate/IO). Commits `34cd054`…`2c8c0a1` + Task 9. Deviations in ANALYSIS.md "Minimap and full map (M7-MAP)". `docs/ROADMAP.md` left untouched (another agent's uncommitted edit) |
| M7-UI-5 | Settings persistence (`config.json`, `Main.SaveSettings`) | ✅ Done | Agent-DeepSeek | `settings.rs`: same keys as vanilla, round-trip test; LightingMode/Background/MapEnabled applied. `e51d318`: `Zoom` key (`Main.GameZoomTarget`, 1..2) + Video row; the mouse wheel now cycles hotbar slots instead of zooming |
| M5-LAKE | Lake basins with grass banks | ✅ Done | sub-agent | `9307400`: water line one tile below the local ground, 1..6-tile carved bowl with a solid dirt floor, walls cleared above the water, grass kept on the flush banks (was: water floating above the ground) |
| M1.4-BG-13 | Grass/dirt transition fidelity | 🔄 Research | sub-agent | `WorldGen.TileFrame` grass branches vs `framing.rs`, 9-slice light; fixes hard green/dirt seams |
| M1.4-BG-14 | Background stack order + zoom behaviour | ✅ Done | Agent-Qoder | `cb71721`: surface background laid out in unzoomed `screenPosition` space (zoom-invariant), +30 push for cloud/mountain passes, −150 only for front layers. Sky gradient formula/order still pending (BG-16) |
| M1.4-BG-16 | Sky gradient + sun/moon/clouds/stars pass | 🟡 Partial | Agent-DeepSeek | `DrawSunAndMoon` arc/scale/phase/rotation ported (`tr-core/src/sky.rs` + `draw_sky`): faithful `bgTopY`, `Sun`/`Moon_0..8` sheets, `moon_type`/`moon_phase` on `World`, save v2. Remaining: `Cloud.GetParallax` density, exact `DrawStarsInBackground` star density, horizon layer order |
| M1.4-BG-17 | Unlit wall black coverage + unbreakable wall 350 | ✅ Done | Agent-Qoder | `b1d07cd`: `DrawOwnBlacks` merging now covers every zero-light walled cell (`light_is_zero && (active || wall != 0)`) as `WallDrawing.DrawWalls` does, not only opaque wall frames; `edit::WALL_UNBREAKABLE = 350` (`WallID.UnbreakableBlockWall`) blocks hammering, tile mining/placement (`CanKillTile`/`IsTileReplaceable`) and wall replacement. Follow-up: echo walls (`WallID.EchoWall`, 318) are skipped like `WallDrawing` with invisible walls hidden — no sprite and no black |
| TILE-DRAW-5 | Full grass framing (WorldGen 82192-83395) | ❌ Pending | - | Port groups A-J + the `-2` marker and per-biome dirt ids; our `grass_frame` covers ~25 % and plateaus on hard frames |
| M1.4-BG-18 | Per-biome animated screen effects | ❌ Pending | - | `SceneState.ApplyVisuals` filters: desert-surface day `HeatDistortion` (`FilterHeatDistortion`, scrolling `Misc/noise`, intensity 4), underworld heat, Sun Scorch; also `Blizzard`/`Sandstorm`/`Aurora`/`WaterDistortion` and the monolith/story filters |

### Tile Objects & Special Tiles

| ID | Task | Status | Assigned To | Notes |
|----|------|--------|-------------|-------|
| TO-FURNITURE | TileObjectData (furniture, doors, chests) | ❌ Pending | - | - |
| TO-PAINT | Paint and coatings | ❌ Pending | - | - |
| TO-GEMSPARK | Gemspark/glow tiles | ❌ Pending | - | - |

---

## 🔥 Current Assignments (in progress)

```
M7-UI-1/2/3/5 Menu framework + settings — done 2026-10-29T18:45Z (Agent-DeepSeek)
M7-UI-6      HUD core (resources, cursor, tooltips) — done 2026-10-30 (Agent-DeepSeek)
M7-UI-4/7    Inventory panel + ItemSlot cells — done 2026-10-30 (Agent-DeepSeek)
M7-UI-7      Whole-UI P3 crafting panel (chrome, filter strip, search bar) — done 2026-10-31 (Agent-controller + subagents)
M7-UI-8      Whole-UI P4 minimap frame + boss bar (styles, zoom, icon row, mH, DrawFancyBar) — done 2026-10-31 (Agent-controller + subagents)
M7-UI-2/3    Whole-UI P5 menu sub-screens (sliders, 17 settings pages, multiplayer alignment) — done 2026-10-31 (Agent-controller + subagents)
M7-UI-9      Whole-UI P6 front-end (char/world select+create, keyboard, credits, legacy confirms) — done 2026-11-01 (Agent-controller + subagents)
REFACTOR-1   Architecture refactor: god-classes split (main 10.5k→86, menu/ app/ render/ world_sim/, tr-core catalogs, generator fixed) — done 2026-11-01 (Agent-controller + subagents)
M6-MUSIC      XACT/MS-ADPCM music playback — done 2026-10-29T19:50Z (Agent-Audio)
M6-3DAUDIO    SoundEngine 3D attenuation — done 2026-10-29T19:50Z (Agent-Audio)
```

---

## ✅ Recently Completed

```
DOC-GRAPHIFY Knowledge graphs (Terraria + ReLogic + Rust) + PORT_MAP  (Agent-DeepSeek)
DOC-README  Player-facing README + final screenshot set        (Agent-DeepSeek)
fc47e85  [M1.4-BG-16]    Sun/moon arc, phases, rotation + tint      (Agent-DeepSeek)
91888cc  [M1.4-BG-12]    Surface background biome cross-fade + layer stack (Agent-Qoder)
b1d07cd  [M1.4-BG-17]    Unlit wall black coverage + unbreakable wall 350 (Agent-Qoder)
8bc63a1  [M5-WSURF]       Derive worldSurface from the deepest column   (Agent-Qoder)
7d9710a  [M1.4-BG-7]     Cave backdrop anchoring + water pass restore (Agent-Qoder)
e0f1ea7  [M4-SPAWN]       Port the Spawner.TrySpawnAnNPC pipeline      (Agent-Nemotron)
918b5a5  [M6-MUSIC]      XACT/MS-ADPCM music engine + 3D attenuation (Agent-Audio)
6b8e36e  [M7-UI]         Main.DrawMenu menu + settings screens     (Agent-DeepSeek)
d780b99  [M1.5-LIGHT-2] Wire the per-type tile light into the app  (Agent-DSV4)
4d42f34  [M1.5-LIGHT-2] Port the full per-type tile light switch   (Agent-DSV4)
74bcf10  [M6-LOCALIZE]   Port the localization core                  (Agent-DeepSeek)
077f0d6  [M1.5-LIQUID-1] Port the WaterfallManager overlay          (Agent-DeepSeek)
79680f2  [M3-TORCH]      Port WorldGen.CheckTorch attachment frames  (Agent-DeepSeek)
```

---

## ⚠️ Coordination notes (2026-10-29T19:50Z)

- **The `rodio` `current_span_len` build error is fixed** in `918b5a5` (`current_frame_len`),
  and `cargo test --workspace` now passes (130 tests). The shared tree's remaining
  uncommitted work (M1.5-LIGHT-2 `mouseTextColor` relocation in `main.rs`,
  `lighting/tile_light/tiledata/liquid_draw/save`, `ROADMAP.md`) is Agent-DSV4's and was
  deliberately left uncommitted.

## ⚠️ Notes retained from 2026-10-29T18:40Z

- **`main.rs` was clobbered earlier.** At ~17:56 an in-flight edit replaced
  `crates/tr-app/src/main.rs` with a pre-waterfall version and it was restored. If you are
  mid-refactor, re-apply your hunk on top of the current file instead of saving an old
  buffer.
- `tools/GenTileData` no longer matches the current decompilation (`Main.`-prefixed
  assignments, `decompiled/Terraria/ID/…`); running it emits empty tables and overwrites
  `tiledata.rs`. See `docs/ANALYSIS.md` risk 7. Do not regenerate until it is fixed.
- Verify `cargo test --workspace` and take a screenshot **before** committing; the shared
  working tree is being edited by several agents at once.

---

## 📝 Usage Instructions for Agents

### Before Starting Work
1. Read this file completely
2. Find an unassigned task (marked "❌ Pending")
3. Verify no other agent is working on similar task
4. Check related documentation in `docs/`

### Starting a Task
1. Update task status to **"🔄 In Progress"**
2. Add your name/identifier in "Assigned To" column
3. Begin implementation following `agents.md` golden rule (port C# first)

### Completing a Task
1. Mark task as **"✅ Done"**
2. Add commit hash/reference
3. Update `docs/ROADMAP.md` if milestones shift
4. Commit with clear message referencing task ID (e.g., "[M1.4-BG-1] Background gradients")

### Important Rules
- NEVER work on same task simultaneously
- If uncertain about behavior → search C# source first
- Document non-obvious decisions in `docs/ANALYSIS.md`
- Verify against original game before committing

---

**File auto-generated for multi-agent coordination**  
**Last modification timestamp: 2026-10-29T00:00:00Z**
