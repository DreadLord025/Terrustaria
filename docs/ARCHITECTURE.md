# Architecture

```
                 assets/Images/*.png  (extracted from the retail .xnb pack)
                          │
  .wld file ──► tr-core ──┼──► tr-app ──► tr-render ──► wgpu / Vulkan ──► swapchain
                 │        │      │            │
        World, Tile      atlas packer     instanced quads
```

## Crates

### `tr-core` — data & formats

- `tile.rs` — `Tile`, 16 bytes: `kind: u16, wall: u16, frame_x: i16, frame_y: i16,
  liquid: u8, flags: u8, slope: u8, paint: u8, wall_col: u8, wall_row: u8`. The layout
  mirrors `Terraria.Tile` bit fields so saves can round-trip without lossy conversion
  (16 B/tile vs. a managed object).
- `world.rs` — row-major `Vec<Tile>`, bounds helpers, spawn scan, memory accounting,
  `frame_important` (the save's tile-frame-important bitset).
- `wld.rs` — sectioned `.wld` reader (see `ANALYSIS.md`). Only header + tile sections are
  decoded today; later sections are reachable via the section table.
- `framing.rs` — autotiling: a port of `WorldGen.TileFrameCosmetic` (block frames) and
  `Framing.WallFrame` (wall frames). See `ANALYSIS.md#autotiling`. Both passes run in
  128-row bands on Rayon: frames depend only on kinds/flags/slopes (never on other frames),
  so the immutable world is shared and results are copied into the tiles afterwards.
- `lighting.rs` — `LightMap::compute(rect)`: seeds each tile like `TileLightScanner`
  (surface, hell, wall, emissive, liquid) and blurs them with the original's two-pass,
  four-sweep carried-max algorithm and per-mask decay. Surface light seeds up to each
  column's own first solid tile (`World::column_surface`), which keeps hills above the
  average `worldSurface` in sunlight. `LightMode`/`LightStyle` describe the four original
  modes: `Color`/`White` slice solid blocks into 4/8/4 px quads whose tints come from
  `LightMap::slice9` (`Lighting.GetColor9Slice`) or `quad4` (`GetColor4Slice`); `Retro`
  quantises light, `Trippy` perturbs channels, and every mode applies
  `GlobalBrightness = 1.2`. Seeding and horizontal sweeps run on Rayon's global pool;
  the finished map is cached by rect, sky colour, hell glow, flicker and
  `World::revision`.
- `liquid.rs` — `LiquidSystem`: the original `Liquid.UpdateLiquid`/`Liquid.Update` port.
  A bounded active list (`maxLiquid = 25 000`) plus a membership bitset stands in for
  `Tile.checkingLiquid`; every 30 Hz call pumps one tenth of the list. Each cell falls
  into the cell below and levels out with up to six horizontal neighbours (the same
  averaging and edge rules as the C#), and entries retire after eight quiet passes.
  `add_water` mirrors `AddWater` + the `LiquidBuffer` overflow queue; block edits call
  `wake_around` so a broken dam starts flowing immediately.
- `worldgen.rs` — a compact generator modelled on `WorldGen`'s shapes: height-noise
  surface + spawn plateau, height smoothing to 1-tile steps, grass/dirt/stone/ash layers,
  hill slope flags, caves from two noise fields, dirt/stone walls, ore blobs, underworld
  hellstone, a forest lake, lava puddles on cave floors, a snow biome (snow surface, winter
  trees, pine background), common trees (22 px trunk frames, canopy anchor, branch markers)
  and tile-3 surface plants. Deterministic for a seed; 4200×1200 in ~165 ms.
- `save.rs` — native `.trwld`: magic/version/seed/header, `frame_important` table, and
  per-row RLE of the 14-byte `Tile`. Default path
  `Documents/My Games/Terrustaria/Worlds/Terrustaria.trwld`; saves are written to a
  temporary file and renamed.
- `tree.rs` — `TileDrawing.DrawTrees` data: 20×20 trunks on the 22 px `Tiles_5` grid,
  marker detection (`frameX` 22/44/66, `frameY >= 198`), biome sheet sections, and the
  foliage style/size table used to place `Tree_Tops_*`/`Tree_Branches_*` sprites.
- `edit.rs` — single-tile break/place: preserves the wall, bumps the revision, marks the
  chunk, recomputes the column surface and re-frames a 5×5 neighbourhood
  (`framing::reframe_area`), so edits do not need a full-world framing pass.
- `tiledata.rs` — **generated** (do not edit) by `tools/GenTileData`: bitset tables for
  `TileID.Sets.*`, `Main.tile*`, the directed merge relation extracted from
  `TileFrameCosmetic`, `LARGE_FRAMES` and `GRASS_DIRT`.

### `tr-render` — GPU

- `gpu.rs` — wgpu 30 instance/device/surface. Backend defaults to **Vulkan**
  (`TR_BACKEND=dx12|gl` to override), `COPY_SRC` enabled for screenshots, HighPerformance
  adapter, 8192 max 2D texture dimension.- `atlas.rs` — shelf packer. Sheets are sorted tallest-first, packed into a fixed-width
  (4096) atlas with 2 px padding. Each sheet also gets a **plain frame**: a fully opaque
  16×16 frame with the most uniform border, used for tiles whose frames are not stored.
  Wall sheets are laid out on a different grid (36 px stride, 8 px inset) — encoding this in
  `SheetLayout` was required for correct wall rendering.
- `sprite.rs` — one instanced pipeline, one bind group per layer:

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

- With no argument it opens the generated world (or generates it first) under
  `Documents/My Games/Terrustaria/Worlds`; an argument still loads an original `.wld` or
  a native `.trwld`. It runs the parallel `framing::frame_world` +
  `framing::frame_walls` (sloped and half-brick tiles included), computes the per-column
  surface (sky horizon + local sunlight) and the 64×64 chunk occupancy bitset, scans used
  tile/wall ids, decodes only those PNGs plus all 32 tree canopy/branch styles and the 42
  cloud textures in parallel, builds the atlas, then creates the window.
- `winit` 0.30 `ApplicationHandler`, `ControlFlow::Poll`.
- Player (`player.rs`): 20×42 AABB, fixed 60 Hz steps, gravity/acceleration/friction,
  1-tile auto-step, axis-separated solid-tile collision, ground-slope (1/2) support,
  half-brick (bottom 8 px) and ceiling-slope (3/4) collision, and water buoyancy. Survival follows `Player.cs`: 200-tick breath with drowning damage, fall
  damage over 25 tiles, lava damage and delayed HP regeneration; the HUD draws a breath bar
  while submerged. `TR_XY` teleports the player for screenshots. The camera
  follows the player's centre (unless `TR_CAM`/`TR_PAN` select the free debug camera).
  The starter player is drawn from the original 40×56 part strips at `(x-10, y-10)` after
  the world passes; `render_key` is part of the G6 frame key.
- Editing: LMB mines with a per-tile timer, RMB places the selected hotbar block after
  reach/support/overlap checks; `tr_core::edit` performs the world change and local
  re-frame. Broken tiles spawn `item.rs` drops (gravity, bounce, magnet to the player,
  first matching or empty slot). The 10-slot hotbar and target outline are drawn through
  the UI sprite batch; F9/close saves the native world and a `.trplayer` sidecar with
  position/HP/10 slots.
- NPCs (`npc.rs`): hopping slimes with gravity/AABB collision and flying demon eyes with
  steering/bobbing; both deal contact damage and have HP. LMB over an NPC performs a melee
  hit; night spawns include demon eyes. Up to four spawn near the player; the player has
  100 HP, hurt cooldown/knockback, respawn and a HUD health bar. NPC positions/animation
  are part of the G6 world-range key.
- Weather (`rain.rs`): `Rain.MakeRain`/`Rain.Update`/`Main.DrawRain` as a 60 Hz simulation.
  `cloudAlpha` ramps at 0.0005/tick toward the event's `maxRaining`; drops spawn above the
  view only into wall-free sky, fall at `(wind*18, 14)` px/tick and die on solid or wet
  tiles with a splash from the `Dust` water frame. `SetBackColor`'s
  `1 - cloudAlpha*0.9` dims the sky tint, surface sunlight and clouds. Drops are drawn in
  the UI range (screen space, rotated) so the G6 world instance range stays cached while
  raining. In a snow biome rain drops are suppressed and the same weather spawns ambient
  `Dust` type-76 flakes (count/wind scaled by `cloudAlpha`) that sway and die on contact.
- Camera is pan/zoom only (`camera` in world pixels). Visible tile range is derived from
  screen/zoom, then iterated chunk by chunk (empty chunks are skipped). Tile frames use a
  dense per-type 18 px UV grid; walls use a precomputed `[wall][13][5]` autotile table.
  Frames that do not sit on the 18 px grid (furniture) fall back to a small lazy cache.
  Tree trunks use their own 22 px/20×20 geometry and biome sheet section.
- Draw pass: sky (`Background_0` gradient + horizon fill), twinkling stars, sun/moon,
  the `DrawSurfaceBG` layer stack for the current biome (forest `7`/`8` mountains and
  `50..52` tree lines, snow `35`/`36` and `37..39` pines, ocean `28`), clouds, per-column
  underground fade, walls, tiles, liquids, then foliage. Solid blocks in `Color`/`White`
  modes are emitted as 9 (or 4) light-sliced quads; slopes use eight 2 px columns + a
  strip; half bricks the bottom 8 px. Tree marker tiles draw nothing; a separate pass
  places the canopy/branch sprites from a rect expanded by 5 tiles so canopies at the
  edge of the view are not clipped. Liquids use the `Misc/water_*` animation strips,
  opacity per kind, a depth tint and the instance flag that enables the shader ripple.
- A wall under a fully opaque, non-sloped, non-half block is skipped (A2). Cells whose
  quantised light is zero and whose drawn content is fully opaque are merged into greedy
  black rectangles (A3/A4) instead of per-tile quads.
- Per frame: compute a `WorldFrameKey` (camera/zoom/screen bits, sky tint, light params,
  light mode, liquid animation frame or time for time-dependent modes, `World::revision`).
  On a hit, `begin_frame_keep_world()` reuses the previous world instance range and only
  the UI is rebuilt (G6); on a miss, `LightMap::compute_into` (storage reused every frame,
  D2) for the visible rect and the tile loop.
- Debug overlay is drawn with the same sprite pipeline (no egui): frame-time graph, draw
  calls, instances, upload KB, camera/tile/zoom, world size and memory, seeded tiles,
  light-map/draw time, wall culling, dark-rectangle counters, quads, foliage, light mode,
  GPU info. Static lines are formatted once.
- Headless tooling: `TR_SHOT`/`TR_SHOT_FRAME` copies the swapchain texture to a mapped
  buffer and writes a PNG; `TR_BENCH=N` switches to `AutoNoVsync`, samples N frame deltas
  and prints avg/p50/p95/p99/max; `TR_ATLAS_DUMP` writes the packed atlas; `TR_NOCACHE`
  bypasses D2/G6 for A/B tests; `TR_PAN=<tiles/frame>` pans deterministically for
  moving-camera benchmarks; `TR_LIGHT_MODE`, `TR_NO_BG`, `TR_NO_WALLS`, `TR_REGEN`,
  `TR_SEED`, `TR_RAIN`, `TR_NO_WEATHER` control modes/world setup.

## Frame flow

1. `about_to_wait` → `request_redraw`.
2. `frame()`: advance `Main.time` unless `TR_DAY` freezes it; player/NPC updates; input →
   camera; compute `Main.ColorOfTheSkies` → `tileColor` (quantised to one-second steps);
   resolve the `LightStyle` for the current mode; check `WorldFrameKey`. On a miss:
   `draw_sky`, `renderer.begin_frame()`, light map for the visible rect, then the
   tile/slope/wall loop, liquids and the tree foliage pass. On a hit: keep the world range.
   Then the HUD/debug overlay (UI) and acquire the swapchain texture.
3. `renderer.render()` writes both globals buffers + the world/UI instance ranges, records
   one pass (clear + 2 draws) and submits.
4. `queue.present(frame)`.

## Design decisions

| Decision | Why |
| --- | --- |
| One atlas + instancing | 2 draw calls regardless of scene complexity; no per-tile state changes |
| Nearest filtering, no mips | Pixel-art identity; avoids shimmer from mip selection at integer zoom |
| CPU-side culling | Visible set is ~1–2 k tiles; a compute cull would cost more than it saves at this scale |
| Row-major tiles, 16 B each | 184 MB for 11.5 M tiles; cache-friendly column scans for the render loop |
| Generated tile tables | `TileID.Sets.*` and the merge relation are extracted from the decompiled source, so framing stays in sync with the game instead of drifting from hand-copied lists |
| 9-sliced solid blocks | Matches `DrawSingleTile_SlicedBlock`: outer 4 px bands average the block's light with its neighbour, which is what makes "Color"/"White" light look smooth instead of tiled |
| Liquid UV wave in the shader | One `flags` float per instance selects a travelling UV ripple inside the large `Misc/water_*` strip; doing it in the shader avoids a second draw pass and hides the 16 px source repetition |
| Screen-space sky, sun/moon, stars and parallax layers | Their layout follows `DrawSunAndMoon`/`DrawSurfaceBG` (parallax and screen sizes, not tile units), so zooming the camera does not stretch the sky. The helper converts the screen rectangle back into camera-relative world coordinates for the shared sprite batch |
| Axis-separated player AABB | Small (20×42) and slow (≤10 px/tick); sub-tile axis resolution plus a one-tile step is enough and keeps slope/tile interactions debuggable. Slopes currently collide as full cells |
| Local re-frame after edits | A mined/placed block can only change the frames of a 2-tile neighbourhood; re-framing 5×5 cells keeps editing at interactive speed instead of re-framing 5M tiles |
| Local column sunlight | The original seeds only `y <= worldSurface`; hills above the average would be dark and valleys below it black. Seeding per column keeps every surface slope lit without opening up caves |
| Native generator + save | The game's own generator is ~45 k lines; a compact deterministic model gives a real world to iterate on, and `.trwld` round-trips every tile field |
| Tree sprites outside the tile batch | Canopies are 80×80 and branches 40×40, anchored to transparent tiles; drawing them after the tile pass matches `DrawTrees` and keeps normal tiles on the fast path |
| Manual debug UI | Avoids pinning egui to wgpu 30 (version coupling), keeps binary small |
| Light and world range cached | A static view costs 0 ms of light/tile work (D2 + G6); panning still gets ~50 % light-cache hits, and `TR_NOCACHE` keeps the uncached path measurable |
| `TR_*` env instrumentation | Deterministic screenshots/benchmarks without desktop automation |

## Where the next systems plug in

- **Chunking / meshing** — replace the per-frame tile loop with cached chunk instance
  buffers keyed by (chunk, light revision), only rebuilt on change.
- **Biome skylines** — forest, snow and ocean use the vanilla layer sets with vertical
  parallax and measured ground-band anchoring. The corruption/crimson/jungle/hallow sets
  plug into the same `SkyAssets`/`draw_bg_layer` pair once those biomes exist; storms,
  lightning and snowfall are the remaining weather pieces.
- **Liquid surface smoothing** — the flow simulation is a direct port of `Liquid.Update`;
  the remaining render work is the original's four-corner `VisibleLiquidLevel` mesh (and
  the `Waterfall_*` overlay for half-brick spouts) so shorelines are diagonal instead of
  tile steps. The static water/lava/honey/shimmer pass and the wave shader already exist.
- **Tile entities / furniture** — `TileObjectData` port producing multi-tile sprites.
- **More lighting modes** — add a `LightMode` variant and a `LightStyle` (smoothing,
  grayscale/colour transform, levels, perturbation); the tile pass and shader are driven
  by the style, so no new draw path is needed.
- **Entities** — the player physics is the first fixed-step simulation. NPCs/projectiles
  will share the same update loop with deterministic order; player draw layers, items,
  health and inventory are the next M2/M3 work.
