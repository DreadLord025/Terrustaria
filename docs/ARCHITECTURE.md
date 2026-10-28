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
  `Framing.WallFrame` (wall frames). See `ANALYSIS.md#autotiling`.
- `lighting.rs` — `LightMap::compute(rect)`: seeds each tile like `TileLightScanner`
  (surface, hell, wall, emissive, liquid) and blurs them with the original's two-pass,
  four-sweep carried-max algorithm and per-mask decay. Flat colour per tile for now; the
  corners come with the liquid pass. See `ANALYSIS.md#lighting`.
- `tiledata.rs` — **generated** (do not edit) by `tools/GenTileData`: bitset tables for
  `TileID.Sets.*`, `Main.tile*`, the directed merge relation extracted from
  `TileFrameCosmetic`, `LARGE_FRAMES` and `GRASS_DIRT`.

### `tr-render` — GPU

- `gpu.rs` — wgpu 30 instance/device/surface. Backend defaults to **Vulkan**
  (`TR_BACKEND=dx12|gl` to override), `COPY_SRC` enabled for screenshots, HighPerformance
  adapter, 8192 max 2D texture dimension.
- `atlas.rs` — shelf packer. Sheets are sorted tallest-first, packed into a fixed-width
  (4096) atlas with 2 px padding. Each sheet also gets a **plain frame**: a fully opaque
  16×16 frame with the most uniform border, used for tiles whose frames are not stored.
  Wall sheets are laid out on a different grid (36 px stride, 8 px inset) — encoding this in
  `SheetLayout` was required for correct wall rendering.
- `sprite.rs` — one instanced pipeline, one bind group per layer:

  ```
  SpriteInstance { pos: vec2, size: vec2, uv0: vec2, uv1: vec2, color: vec4 }  // 48 B
  ```

  Two instance ranges share one ring buffer: `world` (camera transform) and `ui` (screen
  space), written as two `write_buffer` calls and drawn as 2 instanced calls of 6 vertices —
  the entire scene is 2 draw calls. The ring has three buffers so a frame never rewrites
  memory the GPU may still be reading. An 8×8 bitmap font (`font8x8`) is baked into the
  atlas for the debug overlay; a 4×4 white block provides solid-colour quads.
- `sprite.wgsl` — `world = pos + corner*size`, then `(world - cam) * zoom + screen/2`
  → NDC with a flipped Y (world Y grows downward). Alpha-blended, `discard` on a < 0.01.
  The tint is **pre-linearised** (`pow(color.rgb, 2.2)`) before multiplying the sRGB-decoded
  texture: XNA's `Color * Color` is a byte multiply on gamma-encoded values, so multiplying
  linear light would wash the scene out.

### `tr-app` — application

- Loads the world (argument or most recent save), runs `framing::frame_world` +
  `framing::frame_walls`, computes the per-column surface (for the sky horizon) and the
  64×64 chunk occupancy bitset, scans used tile/wall ids, decodes only those PNGs, builds
  the atlas, then creates the window.
- `winit` 0.30 `ApplicationHandler`, `ControlFlow::Poll`.
- Camera is pan/zoom only (`camera` in world pixels). Visible tile range is derived from
  screen/zoom, then iterated chunk by chunk (empty chunks are skipped). Tile frames use a
  dense per-type 18 px UV grid; walls use a precomputed `[wall][13][5]` autotile table.
  Frames that do not sit on the 18 px grid (furniture) fall back to a small lazy cache.
- A wall under a fully opaque, non-sloped block is skipped (A2). Cells whose quantised
  light is zero and whose drawn content is fully opaque are merged into greedy black
  rectangles (A3/A4) instead of per-tile quads.
- Per frame: `draw_sky` (the `Background_0` gradient above the local surface, a per-column
  cave fade below it), then `LightMap::compute_into` (storage reused every frame) for the
  visible rect, then the tile loop multiplying each wall/tile instance by its tile's light
  colour.
- Debug overlay is drawn with the same sprite pipeline (no egui): frame-time graph, draw
  calls, instances, upload KB, camera/tile/zoom, world size and memory, seeded tiles,
  light-map/draw time, wall culling and dark-rectangle counters, GPU info. Static lines are
  formatted once.
- Headless tooling: `TR_SHOT`/`TR_SHOT_FRAME` copies the swapchain texture to a mapped
  buffer and writes a PNG; `TR_BENCH=N` switches to `AutoNoVsync`, samples N frame deltas
  and prints avg/p50/p95/p99/max; `TR_ATLAS_DUMP` writes the packed atlas.

## Frame flow

1. `about_to_wait` → `request_redraw`.
2. `frame()`: input → camera; compute `Main.ColorOfTheSkies` → `tileColor`; `draw_sky`;
   `renderer.begin_frame()`; light map for the visible rect; tile loop pushes lit instances;
   debug overlay; acquire swapchain texture.
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
| Manual debug UI | Avoids pinning egui to wgpu 30 (version coupling), keeps binary small |
| Light computed per frame, not cached | 0.3–0.6 ms for the visible rect; a cached light map is only worth it once tiles change at runtime |
| `TR_*` env instrumentation | Deterministic screenshots/benchmarks without desktop automation |

## Where the next systems plug in

- **Chunking / meshing** — replace the per-frame tile loop with cached chunk instance
  buffers keyed by (chunk, frame variant), only rebuilt on change.
- **Corner lighting** — one extra `vec4` per instance (four corner brightness values) and a
  bilinear sample in `sprite.wgsl`, matching `Lighting.GetColor`'s interpolation.
- **Liquid** — separate instance layer with a wave offset uniform and animated UVs; the light
  masks (`Mask::Water/Honey`) already exist.
- **Tile entities / furniture** — `TileObjectData` port producing multi-tile sprites.
- **Entities** — a fixed-step simulation (60 Hz) decoupled from rendering; player/NPC
  physics ported from `Player.cs`/`NPC.cs`.
