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

- `tile.rs` — `Tile`, 12 bytes: `kind: u16, wall: u16, frame_x: i16, frame_y: i16,
  liquid: u8, flags: u8, slope: u8, paint: u8`. The layout mirrors `Terraria.Tile` bit
  fields so saves can round-trip without lossy conversion (12 B/tile vs. a managed object).
- `world.rs` — row-major `Vec<Tile>`, bounds helpers, spawn scan, memory accounting.
- `wld.rs` — sectioned `.wld` reader (see `ANALYSIS.md`). Only header + tile sections are
  decoded today; later sections are reachable via the section table.
- `framing.rs` — placeholder frame resolution (stored frame for important tiles, per-sheet
  plain frame otherwise). This is the module the real `TileFrameCosmetic` port will replace.

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

  Two instance buffers share one arena: `world` (camera transform) and `ui` (screen space).
  Both are uploaded with a single `write_buffer`, then drawn as 2 instanced calls of 6
  vertices — the entire scene is 2 draw calls. An 8×8 bitmap font (`font8x8`) is baked into
  the atlas for the debug overlay; a 4×4 white block provides solid-colour quads.
- `sprite.wgsl` — `world = pos + corner*size`, then `(world - cam) * zoom + screen/2`
  → NDC with a flipped Y (world Y grows downward). Alpha-blended, `discard` on a < 0.01.

### `tr-app` — application

- Loads the world (argument or most recent save), scans used tile/wall ids, decodes only
  those PNGs, builds the atlas, then creates the window.
- `winit` 0.30 `ApplicationHandler`, `ControlFlow::Poll`.
- Camera is pan/zoom only (`camera` in world pixels). Visible tile range is derived from
  screen/zoom, then iterated row-major; walls are pushed before tiles so the order is
  correct in one batch.
- Debug overlay is drawn with the same sprite pipeline (no egui): frame-time graph, draw
  calls, instances, upload KB, camera/tile/zoom, world size and memory, GPU info.
- Headless tooling: `TR_SHOT`/`TR_SHOT_FRAME` copies the swapchain texture to a mapped
  buffer and writes a PNG; `TR_BENCH=N` switches to `AutoNoVsync`, samples N frame deltas
  and prints avg/p50/p95/p99/max; `TR_ATLAS_DUMP` writes the packed atlas.

## Frame flow

1. `about_to_wait` → `request_redraw`.
2. `frame()`: input → camera; `renderer.begin_frame()`; sky strips; tile loop pushes
   instances; debug overlay; acquire swapchain texture.
3. `renderer.render()` writes both globals buffers + all instances, records one pass
   (clear + 2 draws) and submits.
4. `queue.present(frame)`.

## Design decisions

| Decision | Why |
| --- | --- |
| One atlas + instancing | 2 draw calls regardless of scene complexity; no per-tile state changes |
| Nearest filtering, no mips | Pixel-art identity; avoids shimmer from mip selection at integer zoom |
| CPU-side culling | Visible set is ~1–2 k tiles; a compute cull would cost more than it saves at this scale |
| Row-major tiles, 12 B each | 138 MB for 11.5 M tiles; cache-friendly column scans for the render loop |
| Manual debug UI | Avoids pinning egui to wgpu 30 (version coupling), keeps binary small |
| `TR_*` env instrumentation | Deterministic screenshots/benchmarks without desktop automation |

## Where the next systems plug in

- **Chunking / meshing** — replace the per-frame tile loop with cached chunk instance
  buffers keyed by (chunk, frame variant), only rebuilt on change.
- **Lighting** — a per-tile light map produced by a scanner (port of `LightingEngine`),
  sampled per vertex in the sprite shader (a third `color` component per corner) or applied
  as a second pass.
- **Liquid** — separate instance layer with a wave offset uniform and animated UVs.
- **Tile entities / furniture** — `TileObjectData` port producing multi-tile sprites.
- **Entities** — a fixed-step simulation (60 Hz) decoupled from rendering; player/NPC
  physics ported from `Player.cs`/`NPC.cs`.
