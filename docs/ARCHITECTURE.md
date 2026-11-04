# Architecture

```
  assets/Content/*.xnb ──► tr-content ──► tr-app ──► tr-render ──► wgpu / Vulkan ──► swapchain
                                      │       │            │
  .wld file ──► tr-core ──────────────┘       │       instanced quads
                 │                            │
        World, Tile                      atlas packer
```

The 2026-11-01 refactor (plan `docs/superpowers/plans/2026-11-01-architecture-refactor.md`)
split the two god files — `main.rs` 10 497 → **86 lines**, `menu.rs` 4 187 → three
files — into C#-shaped modules: one directory per subsystem, one free function or
`impl App` block per responsibility, and three explicit trait seams where the
original has an interface ([C# seams](#c-seams) below). The graph is rebuilt
(`graphify`: 3 440 nodes / 7 933 edges Rust-side; merged with the C# graphs
26 267 / 80 324). `README.md` is the player-facing view; this file is the map
an agent needs before touching code.

## Where to look

| If you are working on … | Module |
| --- | --- |
| Bootstrap (world path, Content dir, atlas build, window) | `crates/tr-app/src/main.rs` (86 lines) → `app/mod.rs` (`App::new`, `load_world`, `build_atlas`) |
| winit event loop, input routes, redraw pump | `app/loop.rs` |
| Frame orchestration: `Main.Draw`, light params, the G6 world cache, benchmark epilogue | `app/frame.rs` |
| HUD clicks, inventory stack moves, legacy `Action` dispatch | `app/clicks.rs` |
| Front-end UIState screens (character/world select + create, virtual keyboard) | `app/flow.rs` (+ `ui/char_select.rs`, `ui/world_select.rs`, `ui/char_creation.rs`, `ui/world_creation.rs`, `ui/keyboard.rs`, `ui/uistate.rs`, `ui/save_list.rs`) |
| Legacy menu (`Main.DrawMenu`): settings, multiplayer, confirms, credits | `menu/` — `mod.rs` (items/state) + `pages.rs` (rows) + `actions.rs` (semantics); `app/settings_ui.rs` |
| Fixed 60 Hz simulation | `world_sim/` — `step.rs` (`FixedStepper`), `player`, `npcs`, `items`, `projectiles`, `editing`, `clock`, `liquids`, `weather`, `camera` |
| Sky, sun/moon, clouds, surface background stack | `render/sky.rs`; style transitions in `biome.rs` |
| Underground backgrounds (`Main.DrawBackground` family) | `render/bg.rs` |
| Tile/wall pass, liquids draw, entities, tree foliage, debug overlay | `render/{tiles,liquids,entities,foliage,debug}.rs` |
| Player physics / collision | `player.rs`, `collision.rs`; the 60 Hz tick in `world_sim/player.rs` |
| NPCs, spawning, projectiles | `npc.rs`, `npc_buffs.rs` (NPC buff storage + the DoT tick), `spawner.rs`, `projectile.rs` (+ `world_sim/{npcs,projectiles}.rs`) |
| Items, drops, inventory slots | `item.rs`, `registry.rs`, `inv.rs`; drop tick in `world_sim/items.rs` |
| Lighting and tile light seeding | `tr-core/src/lighting/`, `tr-core/src/tile_light/` |
| Liquids: flow sim, draw cache, waterfalls | `tr-core/src/liquid.rs`, `liquid_draw.rs`, `waterfall/` |
| World generation | `tr-core/src/worldgen/` |
| Saves: `.trwld`, `.wld`, `.map`, binary primitives | `tr-core/src/{save,wld}.rs`, `map/file.rs`, `binio.rs`; sidecars in `persist.rs` + `app/saveglue.rs` |
| Multiplayer | `net.rs`, `app/netglue.rs` |
| Settings / `config.json` | `settings.rs`, `app/settings_ui.rs` |
| Rain and storms | `rain.rs` (field + draw), `world_sim/weather.rs` (the 60 Hz tick) |
| Audio | `audio.rs`, `sound.rs` |
| Fullscreen map / minimap | `map.rs` (app), `tr-core/src/map/`, `ui/minimap.rs` |
| HUD drawing (hotbar, hearts, inventory, tooltips, crafting, boss bar) | `ui/` — see `docs/UI.md` |
| `TR_*` env flags | spread across the modules that consume them (`app/mod.rs` helpers, `render/debug.rs`, `app/flow.rs` `TR_UI`, …); the planned `env.rs` consolidation is deferred |

## Crates

### `tr-core` — data & formats

- `binio.rs` — the shared little-endian cursor (`Reader`/`Writer`, `BinaryReader`-shaped:
  `Option`-returning primitives that hold the cursor on a short read, 7-bit-prefixed
  strings). It absorbed the three hand-rolled readers of `save.rs`/`wld.rs`/`map/file.rs`
  and the app-side `.trwld` header probe (`ui/save_list.rs`); each parser converts `None`
  into its own error type in one place.
- `tile.rs` — `Tile`, 12 bytes: `kind: u16, wall: u16, frame_x: i16, frame_y: i16,
  liquid: u8, flags: u8, meta: u8, wall_frame: u8`. `meta` packs slope (3 bits) and paint
  (5 bits); `wall_frame` packs the wall autotile column (4 bits) and row (3 bits). Accessors
  keep the public API readable and the on-disk `.trwld`/`.wld` fields unchanged.
  (12 B/tile vs. the managed `Terraria.Tile` object.)
- `world.rs` — row-major `Vec<Tile>`, bounds helpers, spawn scan, memory accounting,
  `frame_important` (the save's tile-frame-important bitset).
- `wld.rs` — sectioned `.wld` reader (see `ANALYSIS.md`). Only header + tile sections are
  decoded today; later sections are reachable via the section table.
- `save.rs` — native `.trwld`: magic/version/seed/header, `frame_important` table, and
  per-row RLE of the 12-byte `Tile`. Default path
  `Documents/My Games/Terrustaria/Worlds/Terrustaria.trwld`; saves are written to a
  temporary file and renamed.
- `framing.rs` — autotiling: a port of `WorldGen.TileFrameCosmetic` (block frames) and
  `Framing.WallFrame` (wall frames). See `ANALYSIS.md#autotiling`. Both passes run in
  128-row bands on Rayon: frames depend only on kinds/flags/slopes (never on other frames),
  so the immutable world is shared and results are copied into the tiles afterwards.
- `tiledata/` — **generated** (do not edit) by `tools/GenTileData`, split into `sets.rs`
  (the `TileID.Sets.*` / `Main.tile*` bitsets), `emissive.rs`, `merge.rs` (the 2 228-pair
  directed merge relation — the bulk of the output) and a hand-written `mod.rs` with the
  re-exports and `TILE_COUNT`/`WALL_COUNT`. `mapdata.rs` re-exports the canonical counts.
- `map/` — the world map data model: `mod.rs` (`MapTile`, map colours entry points,
  create/update/reveal), `colors.rs` (the generated `MapHelper` colour tables —
  `tools/gen_map_colors.py`), `option.rs` (the 540-line `GetTileBaseOption` port with
  ~25 `// MapHelper.cs:NNNN` anchors), `file.rs` (the `.map` v326 byte layout over
  `binio`).
- `lighting/` — `mod.rs` owns `LightMap` and the `LightEngine` trait (the seam, below);
  `params.rs` the `LightMode`/`LightStyle` descriptors and `LightParams` (the four
  original modes, `GlobalBrightness = 1.2`, 9/4-slice block smoothing via
  `LightMap::slice9`/`quad4`); `blur.rs` the two-pass, four-sweep carried-max blur and
  the tile-medium masks. `LightMap::compute(rect)` seeds each tile like `TileLightScanner`
  (surface, hell, wall, emissive, liquid) and blurs them. Surface light seeds up to each
  column's own first solid tile (`World::column_surface`), which keeps hills above the
  average `worldSurface` in sunlight. The finished map is cached by rect, sky colour,
  hell glow, flicker and `World::revision`.
- `tile_light/` — the per-type light seeding: `mod.rs` (the surface/hell/wall entry
  points), `palette.rs` (`FastRandom`, torch/hsl/paint colour maths), `scanner.rs` (the
  case-for-case port of `TileLightScanner.ApplyTileLight`, 147 top-level cases).
- `liquid.rs` — `LiquidSystem`: the original `Liquid.UpdateLiquid`/`Liquid.Update` port.
  A bounded active list (`maxLiquid = 25 000`) plus a membership bitset stands in for
  `Tile.checkingLiquid`; every 30 Hz call pumps one tenth of the list. Each cell falls
  into the cell below and levels out with up to six horizontal neighbours (the same
  averaging and edge rules as the C#), and entries retire after eight quiet passes.
  `add_water` mirrors `AddWater` + the `LiquidBuffer` overflow queue; block edits call
  `wake_around` so a broken dam starts flowing immediately.
- `liquid_draw.rs` — `LiquidRenderer.InternalPrepareDraw`: visible liquid levels
  (including seepage into empty cells), edge walls, `FrameOffset`, the source rectangle,
  waterfall-fade opacity, `IsSurfaceLiquid` and the half-brick visibility rule. The app
  consumes the records twice (`Main.DrawLiquid(bg)`), so this module is pure world logic.
- `waterfall/` — `WaterfallManager`: `mod.rs` (`FindWaterfalls` — the 30-tick rescan
  around the view, half-brick spouts plus the 196/460/717 fountain tiles,
  lava/honey/shimmer kinds — and the frame counters) and `draw.rs` (`DrawWaterfall`, the
  per-row geometry walk, `GetAlpha`/`StylizeColor`, the 28 `Waterfall_*` sheets, per-tile
  light tint). The former `LiquidKindChecks` trait artifact is now the free function
  `any_liquid_kind`. It emits `WaterfallDraw` quads in world pixels; the app turns them
  into sprites.
- `worldgen/` — a compact generator modelled on `WorldGen`'s shapes, split into passes:
  `mod.rs` (GenOptions + orchestrator; call order and every RNG draw are pinned by
  `deterministic_for_seed` — four FNV-1a hashes over the post-pass world state),
  `noise.rs` (the xorshift `Noise`/`Rng`, byte-exact with `wind.rs`), `terrain.rs`
  (height-noise surface + spawn plateau, smoothing to 1-tile steps, grass/dirt/stone/ash
  layers, hill slope flags), `caves.rs` (two noise fields, walls, ore blobs, underworld
  hellstone), `liquids.rs` (forest lake, lava puddles), `surface.rs` (snow biome, inland
  desert band, common trees, tile-3 plants). 4200×1200 in ~175 ms; deterministic for a
  seed.
- `tree.rs` — `TileDrawing.DrawTrees` data: 20×20 trunks on the 22 px `Tiles_5` grid,
  marker detection (`frameX` 22/44/66, `frameY >= 198`), biome sheet sections, and the
  foliage style/size table used to place `Tree_Tops_*`/`Tree_Branches_*` sprites.
- `edit.rs` — single-tile break/place: preserves the wall, bumps the revision, marks the
  chunk, recomputes the column surface and re-frames a 5×5 neighbourhood
  (`framing::reframe_area`), so edits do not need a full-world framing pass.
- `torch.rs` — `WorldGen.CheckTorch`: the attachment frame of every torch (`frameX` 0
  standing/wall, 22 on a left support, 44 on a right support, 66 preserved for the tall
  variant) and its removal when nothing holds it. Called from `edit::place_tile` and from
  `reframe_area`, like `WorldGen.TileFrame`.
- `sky.rs` — `Main.SetBackColor`/`DrawSunAndMoon` maths: the sky tint ramps,
  `ApplyColorOfTheSkiesToTiles`, the celestial-body arc/scale/phase/rotation,
  `moon_type`/`moon_phase`.
- `wind.rs` — the wind walk (`windCounter*`), with a deterministic `Main.rand` stand-in
  whose xorshift core is byte-exact with `worldgen`'s.
- `drawdata.rs` — per-type tile draw metrics (`GetTileDrawData`).
- `localization/` — `Terraria.Localization` core: `mod.rs` (`LocalizedText`,
  `LanguageManager`), `culture.rs` (`GameCulture`, the twelve shipped cultures),
  `format.rs` (`VariableText` `{Name}`/`{?Name}`/`{$Key}`, `format_net` for
  `string.Format`), `json.rs` (the trailing-comma-tolerant loader for the shipped
  `Terraria.Localization.Content.<culture>.*.json`). Not yet wired into the UI.

### `tr-render` — GPU

- `gpu.rs` — wgpu 30 instance/device/surface. Backend defaults to **Vulkan**
  (`TR_BACKEND=dx12|gl` to override), `COPY_SRC` enabled for screenshots, HighPerformance
  adapter, 8192 max 2D texture dimension.
- `atlas.rs` — shelf packer. Sheets are sorted tallest-first, packed into a fixed-width
  (4096) atlas with 2 px padding. Each sheet also gets a **plain frame**: a fully opaque
  16×16 frame with the most uniform border, used for tiles whose frames are not stored.
  Wall sheets are laid out on a different grid (36 px stride, 8 px inset) — encoding this in
  `SheetLayout` was required for correct wall rendering.
- `sprite.rs` — one instanced pipeline, three ranges per frame (always-rebuilt
  back/background, cached world, UI), one bind group per range:

  ```
  SpriteInstance { pos: vec2, size: vec2, uv0: vec2, uv1: vec2, color: vec4, flags: f32 }  // 52 B
  ```

  Two instance ranges share one ring buffer: `world` (camera transform) and `ui` (screen
  space), written as two `write_buffer` calls and drawn as 2 instanced calls of 6 vertices —
  the entire scene is 2 draw calls. The ring has three buffers so a frame never rewrites
  memory the GPU may still be reading. An 8×8 bitmap font (`font8x8`) is baked into the
  atlas for the debug overlay; a 4×4 white block provides solid-colour quads.
- `sprite.wgsl` — `world = pos + rot(corner*size)`, then `(world - cam) * zoom + screen/2`
  → NDC with a flipped Y (world Y grows downward). Alpha-blended, `discard` on a < 0.01.
  The tint is **pre-linearised** (`pow(color.rgb, 2.2)`) before multiplying the sRGB-decoded
  texture: XNA's `Color * Color` is a byte multiply on gamma-encoded values, so multiplying
  linear light would wash the scene out. The extra `rot` attribute rotates a sprite around
  its top-left corner (`SpriteBatch.Draw` origin zero); it is 0 for every existing pass, so
  their output is bit-identical, and rain uses it for the original slanted drops.

### `tr-app` — application

`main.rs` is only the bootstrap: logging + panic hook, the world path (`TR_WORLD`, argv
or the default `Terrustaria.trwld`), the Content-dir lookup, `app::load_world` (which runs
the parallel `framing::frame_world` + `framing::frame_walls`, computes the per-column
surface and the 64×64 chunk occupancy bitset, scans used tile/wall ids, decodes only those
PNGs plus all 32 tree canopy/branch styles and the 42 cloud textures in parallel),
`app::build_atlas`, then `EventLoop::run_app(&mut App)`. Everything else lives in:

- **`app/`** (~5.5 k lines) — the `App` struct, the port of `Terraria.Main`'s instance
  state. `mod.rs` holds the struct, the startup assembly and the `impl App` glue with no
  submodule (sim delegates, front-screen openers, keyboard tails); `loop.rs` the winit
  `ApplicationHandler` (`resumed`, `window_event`, `about_to_wait`); `frame.rs` the frame
  orchestrator (`Main.Draw`'s port: sim timers, animated colour cycles, light params, the
  G6 world-instance cache, draw, benchmark epilogue, the shared `next_rand`/`rand01`
  helpers); `flow.rs` the front-end flow (below); `clicks.rs` the HUD/menu click
  handling; `session.rs` the creation tails and erase handlers; `settings_ui.rs` the
  `TR_UI` presets and immediate settings apply; `netglue.rs` the network pump;
  `saveglue.rs` the autosave and map write/load.
- **`render/`** (~4.5 k lines) — the world-drawing passes, each a free function driven by
  `app/frame.rs`: `sky.rs` (the sky gradient, stars, sun/moon arc, clouds, the
  `DrawSurfaceBG` biome layer stack and its cross-fade), `bg.rs` (the `Main.DrawBackground`
  family: surface transition, dirt sheet, underworld black box, rock, magma — with the
  shared `PassPrologue`), `tiles.rs` (the tile/wall/slope loop), `liquids.rs` (the two
  `Main.DrawLiquid(bg)` passes), `entities.rs` (player/NPC sprite composition, held item,
  the map head icon), `foliage.rs` (tree canopies/branches), `debug.rs` (the overlay and
  the screenshot capture).
- **`world_sim/`** — the fixed-step simulation, each system a free function called in the
  original order: `step.rs` (`FixedStepper`, the one 60 Hz accumulator shared by what used
  to be six identical `while` loops), `player.rs`, `npcs.rs`, `items.rs`,
  `projectiles.rs`, `editing.rs` (mine/place/walls), `clock.rs` (`Main.time`, day/night),
  `liquids.rs`, `weather.rs`, `camera.rs`.
- **`menu/`** — the immediate-mode `Main.DrawMenu` port: `mod.rs` (the `Menu` state, item
  hit tests, slider drag), `pages.rs` (each screen's row list and `Page::layout`
  constants), `actions.rs` (the `Action`/`Toggle`/`Act` semantics the App dispatches).
  See `docs/UI.md`.
- **`ui/`** — the in-game HUD and the modern UIState screens (~15.5 k lines). In-game
  panels: `resources.rs`, `cursor.rs`, `tooltip.rs`, `slot.rs`, `inventory.rs`,
  `chrome.rs`, `minimap.rs`, `bars.rs`, `crafting.rs`. Front-end screens: `uistate.rs`
  (the `UIElement`/`UITextPanel`/spliced-panel kit), `cards.rs`, `char_select.rs`,
  `world_select.rs`, `char_creation.rs`, `world_creation.rs`, `keyboard.rs`,
  `save_list.rs`. Shared: `text.rs`, `panel.rs`, `color.rs`, `sliders.rs`, `resources.rs`.
- Leaf modules kept at the crate root: `player.rs` (the physics body and draw state),
  `collision.rs` (`Collision.cs` port), `npc.rs`, `spawner.rs`, `projectile.rs`,
  `item.rs` (drops + `ItemSlot`/`HOTBAR`/`INV`), `biome.rs` (the camera-following
  biome probes and `Main.CalculateWaterStyle`/`DrawBG` style transitions), `rain.rs`,
  `map.rs`, `net.rs`, `persist.rs`, `settings.rs`, `registry.rs`, `assets.rs`,
  `audio.rs`, `sound.rs`, `inv.rs`.

Player simulation details (20×42 AABB, fixed 60 Hz, gravity/step/collision, survival
mechanics), NPC AI, projectiles, editing and weather behaviour are described in
`ANALYSIS.md` (they did not move — only their tick entry points moved into
`world_sim/`).

## C# seams

The three interfaces the refactor introduced, each mirroring a C# abstraction:

| C# | Rust | Notes |
| --- | --- | --- |
| `Terraria.UI.UIState` (the modern menu screens) | `app/flow.rs`'s `FrontScreenBehaviours` | implemented for the `FrontScreen` enum; per-screen `header`/`draw`/`click`/`key` overrides replace the App-sized match arms. `FrontScreen` stays the state carrier (`App.front`, one variant live at a time — the single-mode invariant of `Main.menuMode`) |
| `Terraria.Graphics.Light.ILightingEngine` | `tr-core/src/lighting/mod.rs`'s `LightEngine` | the renderer/simulation only ever call `get_area`/`get_color`/`compute`; `LightMap` is the built-in engine behind the trait, the way the C# constructor accepts any engine. `Rebuild`/`AddLight`/`Clear` have no counterpart — the per-frame light list does not exist in this port |
| `Main.DrawBackground*`'s shared prologue (`bgStartX`/`bgLoops` + the half-tile source jitter) | `render/bg.rs`'s `PassPrologue` | one `pass_prologue` helper so the five underground passes cannot drift apart — the same shape the C# repeats verbatim per pass |

## Frame flow

1. `app/loop.rs`'s `about_to_wait` → `request_redraw`.
2. `app/frame.rs`'s `frame()`: advance `Main.time` unless `TR_DAY` freezes it (via the
   `world_sim::clock` stepper); player/NPC updates through `world_sim/`; input →
   camera; compute `Main.ColorOfTheSkies` → `tileColor` (quantised to one-second steps);
   resolve the `LightStyle` for the current mode; check `WorldFrameKey`. On a miss:
   `render::sky::draw_sky`, `renderer.begin_frame()`, light map for the visible rect, then
   `render::tiles::draw_tiles`, liquids and the tree foliage pass. On a hit: keep the world
   range. Then the HUD (`ui::draw_hud`) or the front-end screens, the debug overlay, and
   acquire the swapchain texture.
3. `renderer.render()` writes both globals buffers + the world/UI instance ranges, records
   one pass (clear + 2 draws) and submits.
4. `queue.present(frame)`.

## Design decisions

| Decision | Why |
| --- | --- |
| One atlas + instancing | 2 draw calls regardless of scene complexity; no per-tile state changes |
| Nearest filtering, no mips | Pixel-art identity; avoids shimmer from mip selection at integer zoom |
| Screen-pixel snapping for entities | Player/NPC sprites are drawn at interpolated sub-pixel world positions; nearest sampling then changes the source texel phase every frame (visible as a face row appearing/disappearing). Rounding the sprite's top-left to an integer screen pixel keeps the texel grid fixed while the camera still moves smoothly |
| Vanilla HUD textures and layout | The hotbar, hearts, breath and boss bar use the original `Inventory_Back`, `Heart`, `Bubble` and `UI_BossBar` sheets with the constants from `GUIHotbarDrawInner`, `ClassicPlayerResourcesDisplaySet` and `BigProgressBarHelper`, instead of custom panels |
| Half-texel UV inset in the vertex shader | At fractional camera/zoom alignments a sprite edge can land exactly on a pixel centre; nearest sampling at `uv = 1.0` then picks the transparent 2 px gap between 18 px frames, which showed as a 1 px grid of holes. Insisting on texel centres keeps every sample inside the source rect |
| CPU-side culling | Visible set is ~1–2 k tiles; a compute cull would cost more than it saves at this scale |
| Row-major tiles, 12 B each | ~60 MB for 5 M tiles; cache-friendly column scans for the render loop |
| Generated tile tables | `TileID.Sets.*` and the merge relation are extracted from the decompiled source by `tools/GenTileData`, so framing stays in sync with the game instead of drifting from hand-copied lists |
| 9-sliced solid blocks | Matches `DrawSingleTile_SlicedBlock`: outer 4 px bands average the block's light with its neighbour, which is what makes "Color"/"White" light look smooth instead of tiled |
| Two liquid draw passes | `Main.DrawLiquid(bg: true)` paints opaque water before walls and tiles (the parallax never shows through), then the foreground pass at `DEFAULT_OPACITY` tints entities, walls and the lakebed. Skipping the background pass is what made the lake look dark and transparent |
| Liquid behind shoreline slopes | `DrawTile_LiquidBehindTile` draws `LiquidSlope_*` masks behind blocks standing in liquid, so the transparent corner of a shore slope shows water. The original skips it when water is both above and beside the slope; we fill that case because it is exactly the visible vanilla shoreline gap |
| Liquid light via a 3×3 split | `Lighting.GetCornerColors` interpolates four corner averages on the GPU; the instanced pipeline carries one tint, so the visible rect is split into 3×3 sub-quads whose flat tints sample the same bilinear field, keeping tile-scale light banding out of the water |
| Screen-space sky, sun/moon, stars and parallax layers | Their layout follows `DrawSunAndMoon`/`DrawSurfaceBG` (parallax and screen sizes, not tile units), so zooming the camera does not stretch the sky. The helper converts the screen rectangle back into camera-relative world coordinates for the shared sprite batch |
| Axis-separated player AABB | Small (20×42) and slow (≤10 px/tick); sub-tile axis resolution plus a one-tile step is enough and keeps slope/tile interactions debuggable. Slopes currently collide as full cells |
| Local re-frame after edits | A mined/placed block can only change the frames of a 2-tile neighbourhood; re-framing 5×5 cells keeps editing at interactive speed instead of re-framing 5M tiles |
| Local column sunlight | The original seeds only `y <= worldSurface`; hills above the average would be dark and valleys below it black. Seeding per column keeps every surface slope lit without opening up caves |
| Native generator + save | The game's own generator is ~45 k lines; a compact deterministic model gives a real world to iterate on, and `.trwld` round-trips every tile field |
| Tree sprites outside the tile batch | Canopies are 80×80 and branches 40×40, anchored to transparent tiles; drawing them after the tile pass matches `DrawTrees` and keeps normal tiles on the fast path |
| Manual debug UI | Avoids pinning egui to wgpu 30 (version coupling), keeps binary small |
| Light and world range cached | A static view costs 0 ms of light/tile work (D2 + G6); panning still gets ~50 % light-cache hits, and `TR_NOCACHE` keeps the uncached path measurable |
| `TR_*` env instrumentation | Deterministic screenshots/benchmarks without desktop automation |
| One trait per C# interface, not per class | `FrontScreenBehaviours` and `LightEngine` exist because the C# has an interface there; elsewhere the refactor splits by file/module shape so the graph stays queryable without inventing abstractions the original does not have |

## Where the next systems plug in

- **Chunking / meshing** — replace the per-frame tile loop (`render/tiles.rs`) with cached
  chunk instance buffers keyed by (chunk, light revision), only rebuilt on change.
- **Biome skylines** — forest, desert, ocean and snow use the vanilla layer sets with
  vertical parallax and the original's `bgAlphaFrontLayer`/`bgAlphaFarBackLayer`
  cross-fade (`render/sky.rs` + `biome.rs`). The corruption/crimson/jungle/hallow/mushroom
  sets plug into the same `SkyAssets`/`draw_bg_layer`/`advance_bg_alphas` path once those
  biomes (and their `SceneMetrics` zones) exist; storms, lightning and snowfall are the
  remaining weather pieces (`rain.rs`).
- **Liquid surface smoothing** — the flow simulation is a direct port of `Liquid.Update`
  (`tr-core/src/liquid.rs`) and the renderer (`render/liquids.rs`) uses the original's
  `VisibleLiquidLevel`/edge-wall draw cache, so shorelines and partly filled cells taper
  like the original. The remaining pieces are the shimmer-specific `DrawShimmer`
  distortion and the honey/shimmer `Draw()` passes, which currently share the normal
  liquid path.
- **Tile entities / furniture** — `TileObjectData` port producing multi-tile sprites in
  `render/tiles.rs`.
- **More lighting modes** — add a `LightMode` variant and a `LightStyle`
  (`tr-core/src/lighting/params.rs`); the tile pass and shader are driven by the style, so
  no new draw path is needed.
- **Entities** — `world_sim/` owns the fixed-step order; new systems join as a module plus
  one `FixedStepper` tick in the original call order. Player draw layers, items, health
  and inventory are the next M2/M3 work.
