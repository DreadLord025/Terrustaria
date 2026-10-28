# Roadmap & honest scope

## Why this is a long project

Terraria 1.4.5.8 is ~755 000 lines of decompiled C#. Most of it is *content behaviour*, not
engine: ~6 200 items, ~780 NPCs, ~1 100 projectiles and ~750 tile types, each with
hand-written logic, plus ~45 000 lines of world generation and ~35 000 lines of UI.

A faithful Rust port is therefore a **multi-year engineering effort** (a small team for
2–3 years; substantially longer for one developer). What exists now is the foundation: the
hard, low-level parts (save format, asset pipeline, GPU renderer) are working and measured.
The estimates below are in "focused agent-days" — a day of continuous work by an AI agent
with the decompiled sources at hand — and are honest order-of-magnitude figures with
uncertainty of roughly ±50 %.

## Milestone 0 — foundation ✅ (done)

| Deliverable | Result |
| --- | --- |
| Decompile game + ReLogic | 1 549 + 167 `.cs` files |
| XNB → PNG pipeline (LZX, DXT) | 15 100 / 15 123 textures |
| `.wld` parser | 11.5 M tiles in ~60 ms |
| Vulkan renderer (wgpu 30) | 2 draw calls, ~1 900 instances |
| Bench | 1.91 ms avg / 523 FPS @1600×900 |
| Autotiling data generator | 2 228 merge pairs, 149 tile sets extracted |
| Lighting (scanner + blur + daylight) | 0.3–0.6 ms per frame, verified against engine constants |
| Debug overlay, screenshots, bench harness | done |

## Milestone 1 — visual fidelity (in progress)

Goal: a still frame that is hard to distinguish from the original at the same camera.

1. **Autotiling** — ✅ *mostly done*: the generic 20-frame block chain, the merge families
   (`Dirt`/`Mud`/`Ash`/moss/grass/ore/`BlockMergesWithMergeAllBlock`) and 2 228 per-type
   merge pairs are ported; wall framing is complete (`Framing.WallFrame`, 20×3 lookup).
   Remaining: per-type special branches (grass edge overrides, moss, vines, cactus),
   `TileFrameImportant` multi-tile objects. *~3–6 agent-days left of ~15.*
2. **Lighting** — ✅ *first pass done*: `TileLightScanner` seeds (surface/hell/wall/
   emissive/liquid), `LightMap`'s two-pass four-sweep blur with per-mask decay, and
   `Main.SetBackColor` → `ApplyColorOfTheSkiesToTiles` for daylight. Verified numerically
   against the engine's own constants (`TR_LIGHT_AT`). Remaining: four-corner interpolation,
   `GlobalBrightness`, sun/moon/star light, per-type frame-dependent lights, invisible
   blocks/walls, mouse-light flicker. *~2–4 agent-days of ~8 left.*
3. **Liquids** — water/lava/honey/shimmer surfaces, waves, opacity blending.
   *~4–8 agent-days.* ← next
4. **Backgrounds & sky** — the real parallax layers, sun/moon/stars/clouds, biome
   backgrounds, `Main.DrawBackground` dirt/rock/magma layers. Today only the `Background_0`
   gradient plus a depth fade are drawn. *~5–10 agent-days.*
5. **Tile objects** — `TileObjectData` (multi-tile furniture, doors, chests), tree
   canopies/branches (`TileDrawing` composes those from `Tree_Tops_*`/`Tree_Branches_*`
   instead of tile frames), paint, coatings, slopes/half-blocks, gemspark/glow.
   *~10–20 agent-days.*

Estimated total: **~30–60 agent-days** remaining. Exit criterion: side-by-side screenshots
of the same world positions match texture-for-texture.

## Milestone 1.5 — performance & scalability (agreed)

**Product constraint:** this is a 1:1 reimplementation — same "body", different "engine".
The in-game zoom range is the original one (`Main.GameZoomTarget` clamped to **1.0–2.0**);
below that the original switches to the map, so **no LOD or reduced-detail mode is allowed**.
Settings, menu and all parameters stay identical to Terraria. Any optimisation below must
produce the same pixels at the same camera/time, verified by screenshot diff.

The map view is *not* the tile renderer: in the original it is a precomputed world texture.
It will be implemented separately (M1.4) and is out of scope here.

### Measured baseline (RTX 4070 Laptop, Ryzen AI 9 365, 1600×900, overlay off)

| Scene | Instances | Frame | Notes |
| --- | --- | --- | --- |
| Surface, zoom 1.5 | 1 612 | 1.86 ms | real gameplay zoom |
| Surface, zoom 1.0 | 3 744 | 1.61 ms | minimum real zoom |
| Cave, zoom 1.0 | 5 795 | 1.89 ms | most cost is light + present |
| Surface, zoom 0.25 (debug only) | 40 630 | 6.16 ms | stress test |
| Cave, zoom 0.25 (debug only) | 77 439 | 8.75 ms | walls = 20 187 instances; most tiles are black |
| Empty sky, zoom 8 | 1 | 1.33 ms | present/compositor floor (p50 1.10) |

Frame times above are 1 200-frame runs of the pre-optimisation binary. The present floor is
bimodal on this laptop (roughly 1.1 ms or 1.85 ms p50 depending on compositor/power state),
so CPU-side phase timings from `TR_STATS` are the reliable signal at real zooms.

### Phase 1 results (done)

All items below were verified against the pre-optimisation binary: screenshots of the same
surface/cave/hell cameras are **pixel-identical (0 differing pixels, overlay off)**. The
debug overlay changed on purpose (more counters).

| Scene | Instances before → after | Frame before → after | CPU tile+light after |
| --- | --- | --- | --- |
| Surface, zoom 1.5 | 1 612 → 1 502 | 1.86 → 1.33 ms | 0.22 ms |
| Surface, zoom 1.0 | 3 744 → 2 954 | 1.61–1.93 → 1.81–1.89 ms* | ~0.3 ms |
| Cave, zoom 1.0 | 5 795 → 3 544 | 1.89 → 1.79–1.90 ms* | 0.48 ms |
| Surface, zoom 0.25 | 40 630 → 23 200 | 6.16 → 3.70 ms | 2.84 ms |
| Cave, zoom 0.25 | 77 439 → 44 356 | 8.75 → 5.58 ms | 4.34 ms |
| Empty sky, zoom 8 | 1 → 1 | 1.33 → 1.33 ms | floor |

\* Within the present-floor noise; GPU instance work is lower, but wall-clock at real zooms
is dominated by presentation.

### Agreed items

| ID | Item | Phase | Status |
| --- | --- | --- | --- |
| A1 | Chunk occupancy (64×64) + skip empty chunks/rows; also used by light later | 1 | done |
| A2 | Skip walls fully covered by an opaque non-sloped block | 1 | done |
| A3 | Darkness culling: quantised light == 0 + opaque content → greedy black rectangles | 1 | done |
| A4 | Chunk-level dark shortcut (one black quad for fully dark chunks) | 1 | done (greedy rows merge full chunks) |
| B1 | Remove per-frame `collect::<Vec<_>>()`; upload world/UI as two ranges | 1 | done |
| B3 | Instance ring buffer (2–3 frames) to avoid GPU/CPU stalls | 1 | done (3 buffers) |
| B4 | Dense per-kind UV tables instead of per-tile `HashMap` | 1 | done |
| B5 | Precomputed wall UV table `[wall][col][row]` (col ≤ 12, row ≤ 4) | 1 | done |
| D1 | Reusable light-map scratch buffers (no 1.7 MB alloc/zero per frame) | 1 | done |
| E4 | Remove redundant bounds checks in the hot loop (row slices) | 1 | done |
| G1 | Measure the ~1.7 ms present/compositor floor, separate it from scene cost | 1 | done (1.10–1.33 ms) |
| G4 | Cache debug-overlay strings (only when the overlay is open) | 1 | done |
| H | `TR_STATS` per-phase timings + culling counters; screenshot-diff harness | 1 | done (`tools/pixel_diff.py`) |
| C1 | Parallel light: seeding + 4 sweeps via rayon (persistent pool) | 2 | planned |
| C2 | Parallel instance build: count → prefix sum → fill, no atomics | 2 | planned |
| C3 | Persistent rayon pool (no per-frame thread creation) | 2 | planned |
| C4 | Parallel world framing at load | 2 | planned |
| C5 | Parallel PNG decode at load | 2 | planned |
| B2 | Compress instances 48 → 20–24 B (u16 tile positions/UV, RGBA8 tint) | 2 | planned |
| B6 | Merge identical textured runs — only after B7; otherwise skipped | 3 | deferred |
| B9 | Sort instances by sheet/page for texture-cache locality | 2 | optional |
| D2 | Cache the light map by (rect, sky colour, flicker) when nothing changes | 2 | planned |
| D3 | Incremental light on camera movement (overlap reuse + dirty borders) | 3 | planned |
| D5 | GPU light: masks/seeds as textures, blur as parallel compute dispatches | 3 | planned |
| D6 | Deferred lighting: unlit chunk meshes + screen-space light texture | 3 | planned |
| F1 | Chunk meshes (32×32/64×64), rebuilt only on tile edits | 3 | planned |
| B7 | Texture array instead of atlas — internal GPU storage only, must be pixel-identical; enables B6 and cuts cache pressure | 3 | evaluate |
| B8 | Manual `fract` UV wrapping for merged runs (nearest, no mips) | 3 | evaluate |
| B10 | Crop/repack the atlas to frames actually used by the world | 4 | planned |
| E1 | Pack `Tile` 16 → 8–10 bytes (bitfields) | 4 | planned |
| E2 | SoA/chunked tile storage for the light and draw scans | 4 | planned |
| F3 | GPU tilemap (tile metadata in a texture, static grid, vertex-shader culling) | 4 | optional |
| G2 | Present-mode / frame-latency tuning under `AutoVsync` | 1 | done (default latency 1, `TR_LATENCY` override) |
| G3 | Push constants instead of the two globals UBOs | 1 | evaluated — 2×32 B/frame is below noise |
| G5 | Avoid recreating per-frame descriptors where the API allows | 1 | evaluated — surface views have no stable handle |
| G6 | Skip instance rebuild when camera/light/world are unchanged | 2 | planned |

### Explicitly excluded

- **A5 (zoom LOD)** — impossible by design: the original has no zoom below 1.0, and detail
  must never drop.
- **A6 (background/occlusion tricks)** — no measurable benefit in 2D; not worth the
  complexity or the risk to fidelity.
- **C6 (light one frame ahead)** — rejected: visible one-frame light lag and a source of
  entity/lighting bugs later.
- **D4 (light LOD)** — same reason as A5; zoom never goes below 1.0.
- **F2 (4-frame temporal cycle)** — vanilla needs it because it renders into two screen
  targets; our instanced path does not, so it would only add latency.
- **F5 (indirect/GPU culling)** — premature at 2–80 k instances.

### Entities (M4) — data-oriented plan

Entities will be **data-oriented**, not object-oriented: fixed 60 Hz simulation, arrays of
components (SoA) grouped by archetype (player, NPC, projectile, item drop, gore), spatial hash
for broad-phase queries, and deterministic update order. Parallelism only across independent
islands/archetypes with per-thread output buffers; no shared mutable state. This gives DOTS-
like cache behaviour without adopting a full ECS framework or rewriting the world model.
Terraria's own update order and RNG streams must be reproduced exactly, so "parallel by
default" is explicitly not a goal — determinism first.

## Milestone 2 — player

Physics (`Player.cs` subset), collision, input mapping, animation state machine, draw layers
for the default outfit, camera follow, spawn/respawn, simple HUD. **~15–30 agent-days.**

## Milestone 3 — items, inventory, crafting, UI

Item use logic (subset first), inventory grid, hotbar, chests, crafting query, tooltips,
prefixes, equipment slots, mouse interaction, drag/drop. Full item parity is ~50 000 lines;
a playable subset (tools, weapons, blocks, potions, ~200 items) is **~40–80 agent-days**.

## Milestone 4 — NPCs, combat, projectiles

Damage model, knockback, immunities, buffs/debuffs; projectile system (subset), a first
batch of enemies (slimes, zombies, demon eyes, skeletons), spawn logic, town NPCs + shops.
**~60–120 agent-days.** Full 780-NPC / 1 100-projectile parity: several times that.

## Milestone 5 — world generation

Port generation passes and structures (`WorldGen.cs` + `Generation/*`). Alternatively keep
loading existing `.wld` files, which already works. **~40–90 agent-days** for a passable
generator; full parity much more.

## Milestone 6 — audio, map, networking, polish

XACT bank playback (`xsb`/`xwb`), music, 3D-ish attenuation; minimap/full map; exact
1.4.5.8 network protocol for multiplayer; localization; settings; save writing.
**~40–90 agent-days.**

## Aggregated

| Target | Approx. effort |
| --- | --- |
| Convincing single-player sandbox (M1–M3 subset) | 3–6 months of agent time |
| Broad single-player parity (M1–M6) | 1.5–3 years of agent time |
| Full 1:1 including multiplayer & all content | 3+ years, realistically a team |

These numbers assume the decompiled sources stay available and the port stays mechanical.
They do **not** assume the port is easy: the content surface is the bottleneck, not the
engine.

## Recommended order of attack

1. M1.1 autotiling ✅ → M1.2 lighting ✅ (first pass) → M1.3 liquids ← next.
2. M2 player + M1.4 parallax → it starts to feel like a game.
3. M3 inventory + block placement/mining → the core loop.
4. M4 one enemy family + one boss → combat loop.
5. Everything else.

## Non-goals for now

- Modding API, tModLoader compatibility.
- Mobile/console parity.
- Exact network compatibility (single-player first).
