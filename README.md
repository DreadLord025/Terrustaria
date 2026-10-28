# Terraria-Rust

A native reimplementation of Terraria 1.4.5.8 in Rust with a Vulkan renderer, built from a
clean-room analysis of the decompiled C# game.

> **Status: milestone 0/1 — foundation.** This is *not* a playable game yet. It loads a real
> `.wld` world, extracts the original textures from the game's `.xnb` files, and renders the
> tile grid with a single instanced draw call on Vulkan at 500+ FPS. Gameplay systems
> (player, items, NPCs, lighting, liquids, UI) are not implemented. See
> [`docs/ROADMAP.md`](docs/ROADMAP.md) for the honest plan.

![surface](docs/screenshots/surface.png)

## What works right now

- **World loading** — native `.wld` parser (format versions ≥ 87, section table, RLE tile
  section, header metadata). A 6400×1800 world with 11.5 M tiles loads in ~60 ms.
- **Asset pipeline** — `tools/XnbExtract` decompresses the game's XNA v5 `.xnb` containers
  (LZX) and writes PNGs. 13 993 textures extracted, 15 100 of 15 123 files OK.
- **Atlas** — all sheets used by the current world are shelf-packed into one 4096×6454
  texture at startup (105 MB), nearest filtering, no mipmaps.
- **Renderer** — wgpu 30 on the **Vulkan** backend, one instanced-quad pipeline, one atlas,
  2 draw calls per frame (world + UI), ~1600–1900 instances for a 1600×900 viewport.
- **Debug overlay** — FPS, frame graph, draw calls, instance counts, upload size, camera
  state, tile counts, memory, GPU/driver info.
- **Camera** — WASD/arrows pan, mouse wheel zoom, F5 respawn.
- **Headless tooling** — `TR_BENCH` (uncapped frame timing), `TR_SHOT` (deterministic
  screenshot without desktop capture), `TR_CAM`/`TR_ZOOM`, `TR_ATLAS_DUMP`.

## Measured (RTX 4070 Laptop, Ryzen AI 9 365, 1600×900)

| Metric | Value |
| --- | --- |
| Frame time, avg | **1.91 ms** (~523 FPS, no vsync) |
| p50 / p95 / p99 | 1.86 / 3.14 / 3.79 ms |
| Draw calls | 2 |
| Instances | ~1 900 |
| World load | ~60 ms |
| Atlas build | ~60 ms |

## What is *not* implemented (yet)

Autotiling frame selection (the biggest visual gap), lighting, liquids, players, items,
NPCs, projectiles, world generation, UI/inventory, audio, networking, save writing.
Details and estimates in [`docs/ROADMAP.md`](docs/ROADMAP.md).

## Build & run

Requirements: Rust 1.89+, a Vulkan-capable GPU and driver, Windows (Linux should work but is
untested).

```powershell
# 1. decompile the game (once) — requires ilspycmd
ilspycmd "C:\Program Files (x86)\Steam\steamapps\common\Terraria\Terraria.exe" -p -o decompiled

# 2. extract textures (once) — .NET 10 SDK
dotnet build tools/XnbExtract -c Release
.\tools\XnbExtract\bin\Release\net10.0\XnbExtract.exe `
  "C:\Program Files (x86)\Steam\steamapps\common\Terraria\Content\Images" assets\Images

# 3. build and run (no argument = most recent world in Documents/My Games/Terraria/Worlds)
cargo run --release -p tr-app
cargo run --release -p tr-app -- "C:\path\to\world.wld"
```

### Environment variables

| Variable | Meaning |
| --- | --- |
| `TR_BACKEND=dx12\|gl` | Use a different wgpu backend (default `vulkan`) |
| `TR_DEBUG=0` | Hide the debug overlay |
| `TR_CAM=x,y` | Start camera at tile coordinates |
| `TR_ZOOM=1.5` | Initial zoom factor |
| `TR_BENCH=600` | Render 600 frames uncapped, print p50/p95/p99, exit |
| `TR_SHOT=out.png` | Save a screenshot and exit |
| `TR_SHOT_FRAME=25` | Frame at which the screenshot is taken |
| `TR_ATLAS_DUMP=atlas.png` | Dump the packed atlas |
| `TR_DUMP_TILES=1` | Log tile ids/frames around spawn |

## Layout

```
crates/tr-core     world model, .wld parser, tile packing, framing stub
crates/tr-render   wgpu device, atlas packer, instanced sprite renderer, WGSL shader
crates/tr-app      winit app, asset loading, camera, debug overlay, bench/screenshot
tools/XnbExtract   C# XNB (LZX/DXT) -> PNG extractor
decompiled/        ILSpy output of Terraria.exe (not redistributed)
assets/Images/     extracted textures (not redistributed)
```

## Legal

The decompiled sources and extracted textures are derived from a copy of Terraria you own and
are **not** part of the project's source. They are excluded by `.gitignore` and must not be
redistributed. This project is a personal engineering experiment and is not affiliated with
Re-Logic.
