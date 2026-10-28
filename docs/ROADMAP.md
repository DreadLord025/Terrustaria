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
   Sloped and half-brick tiles are framed and drawn with the original 2 px slices. Common
   trees are composed (`DrawTrees`: 22 px trunks, `Tree_Tops_*`/`Tree_Branches_*`), and
   tile-3 surface plants are generated and drawn. Grass now uses `WorldGen.TileFrame`'s
   side/top/corner override rectangles (grass overhang over a step) and the tree generator
   follows `GrowTree`'s trunk/branch frame table. Remaining: per-type special branches
   (moss, vines, cactus arms), `TileFrameImportant` multi-tile objects.
   *~2–4 agent-days left of ~15.*
2. **Lighting** — ✅ *second pass done*: `TileLightScanner` seeds (surface/hell/wall/
   emissive/liquid), `LightMap`'s two-pass four-sweep blur with per-mask decay,
   `Main.SetBackColor` → `ApplyColorOfTheSkiesToTiles`, per-column surface seeding,
   `GlobalBrightness = 1.2`, `GetColor9Slice`/`GetColor4Slice` block slicing and all four
   `LightMode`s (Color/White/Retro/Trippy, F2). Verified numerically with `TR_LIGHT_AT`.
   Remaining: sun/moon/star light, per-type frame-dependent lights, invisible
   blocks/walls, mouse-light flicker, shader chromatic aberration once instances carry a
   flag. *~1–2 agent-days of ~8 left.*
3. **Liquids** — ✅ *fourth pass*: `LiquidRenderer.InternalPrepareDraw` is ported as
   `liquid_draw.rs` (visible levels, edge walls, `FrameOffset`, source rect, waterfall-fade
   opacity, `IsSurfaceLiquid`), `Lighting.GetCornerColors` drives 3x3 bilinear light and the
   draw runs twice like `Main.DrawLiquid` (opaque behind walls/tiles, `DEFAULT_OPACITY`
   over entities) from the original `Misc/water_*` frames. The generator fills a forest lake
   and lava puddles; `Liquid.UpdateLiquid`/`Liquid.Update` is a 30 Hz active-cell simulation
   (gravity, six-neighbour leveling, kill/retire, overflow buffer) and edits wake neighbours.
   Remaining: the `Waterfall_*` overlay at half-brick spouts, water styles and shimmer
   sparkles. *~1–3 agent-days left of ~8.*
4. **Backgrounds & sky** — ✅ *third pass*: `Background_0` gradient, deterministic
   `Cloud_*` clouds, sun, moon, stars, the 15/9-minute day-night clock, the per-column cave
   fade, rain (`Rain.MakeRain`/`Rain.Update`/`Main.DrawRain`, `cloudAlpha` ramp, rotated
   drops, splashes, overcast `SetBackColor` dimming, snow-biome `Dust` snowfall, storm
   cloud banks and lightning flashes), and the vanilla `DrawSurfaceBG`
   composition for the biomes the world has: forest (`7`/`8` mountains + `50..52` tree
   lines), snow (`35`/`36` + `37..39` pines) and ocean (`28` sea horizon), with
   `backgroundTopMagicNumber` vertical parallax and measured ground-band anchoring.
   Remaining: corruption/crimson/jungle/hallow variants (no such biomes yet), wind gusts
   and weather audio, and the textured `Main.DrawBackground` underground layers.
   *~2–5 agent-days.* ← next
5. **Tile objects** — `TileObjectData` (multi-tile furniture, doors, chests), paint,
   coatings, gemspark/glow. Trees and slopes are done. *~8–18 agent-days.*

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

### Phase 3 results (in progress)

| Scene | Change | Measured result |
| --- | --- | --- |
| D3 | Light cache keeps a 32-tile valid region around the view; pans inside it reuse the map | Panning at zoom 0.25: light **3.31 → 0.03 ms**, draw **4.13 → 0.74 ms** (release, `TR_PAN=0.25`, 600 frames; `TR_NOCACHE=1` gives the old path) |
| F1 (step 1) | Sky/parallax moved to an always-rebuilt back range; the world range is keyed by camera tile + light generation instead of exact camera bits | Panning at zoom 1.5: **71–74 %** world-range hits, draw 0.02–0.03 ms; zoom 0.25 panning: draw **0.74 → 0.28 ms**; static: **98 %** hits, 0.00 ms. Static screenshots pixel-identical (0 differing pixels) |

### Phase 2 results (done)

Every change was A/B-tested against the Phase 1 binary. Surface, cave and hell frames at
zoom 1.0–2.0 and the zoom-0.25 stress views are **pixel-identical (0 differing pixels,
overlay off)**; framed tile/wall counts and instance counts are unchanged.

| ID | Change | Measured result |
| --- | --- | --- |
| C1 + C3 | Parallel light on Rayon's persistent global pool | Seeding and the horizontal sweeps are parallel; vertical sweeps stay serial because row-major storage makes them strided. Cave at zoom 0.25: light **7.67 → 3.05 ms**, full draw **9.09 → 3.80 ms**. Rects below 96k cells keep the sequential path: task dispatch cost more than it saved at 1.0–2.0 zoom (light 0.20 → 0.22 ms). |
| D2 | Light-map cache keyed by rect, sky, glow, flicker and world revision | Static view: light compute **0**. Panning at 0.25 tiles/frame still hits ~50 % of frames (tile rect changes every ~4 frames): cave 0.25 light **3.07 → 1.54 ms**. |
| G6 | Reuse the world instance range when camera/zoom/screen/light/world are unchanged | Static stress: cave 0.25 frame **9.73 → 2.02 ms**, surface 0.25 **4.01 → 2.05 ms** (600-frame averages). At 1.0–2.0 zoom the frame is present-floor bound, so the gain is CPU headroom rather than wall-clock. Panning always misses (camera bits change), as intended. |
| C4 | Banded parallel autotiling (128 rows) | Framing **295 → ~70 ms** (5 911 174 tiles in ~53 ms + 2 691 213 walls in ~19 ms) on the reference 6400×1800 world; outputs identical. |
| C5 | Parallel PNG decode | 258 sheets **68 → ~20 ms**; packed atlas (4096×6454) and every UV unchanged. |
| C2 | Parallel instance build | **Evaluated, not implemented.** The tile loop is already 0.5–0.9 ms at zoom 0.25 and ~0.05 ms at real zoom; the greedy dark-rect merge is order-sensitive and `Lookups::frame_cache` is a lazy `HashMap`, so a parallel build risks pixel identity for a gain hidden by the present floor. |
| B2 | Compress 48 B instances to 20–24 B | **Evaluated, deferred.** 42k instances = 2.0 MB/frame at the 0.25 stress zoom and ~140 KB/frame at real zoom; the frame is present- and CPU-bound, not upload-bound, and RGBA8 tint quantisation would break the exact-pixel constraint. |
| B9 | Sort instances by sheet/page | **Evaluated, deferred.** Everything samples one atlas through one bind group, so sheet sorting cannot improve texture locality; it would only risk changing blend order. |

Startup (excluding window/GPU init) went from **~475 ms to ~210 ms** on the reference
machine. `TR_NOCACHE=1` reproduces the uncached per-frame path for future A/B tests and
`TR_PAN=<tiles/frame>` gives a deterministic moving-camera benchmark.

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
| C1 | Parallel light: seeding + 4 sweeps via rayon (persistent pool) | 2 | done (seeding + horizontal sweeps; vertical serial by design) |
| C2 | Parallel instance build: count → prefix sum → fill, no atomics | 2 | evaluated — present-floor hidden, pixel-identity risk |
| C3 | Persistent rayon pool (no per-frame thread creation) | 2 | done (Rayon global pool) |
| C4 | Parallel world framing at load | 2 | done (~4.2× faster, identical frames) |
| C5 | Parallel PNG decode at load | 2 | done (~3.4× faster, identical atlas) |
| B2 | Compress instances 48 → 20–24 B (u16 tile positions/UV, RGBA8 tint) | 2 | evaluated — upload is not the bottleneck, tint quantisation risks pixels |
| B6 | Merge identical textured runs — only after B7; otherwise skipped | 3 | deferred |
| B9 | Sort instances by sheet/page for texture-cache locality | 2 | evaluated — one atlas and one bind, no locality win |
| D2 | Cache the light map by (rect, sky colour, flicker) when nothing changes | 2 | done (static: 0 ms; panning: ~50 % hits) |
| D3 | Incremental light on camera movement: keep a `CACHE_MARGIN = 32` tile valid region around the visible rect; a pan inside it reuses the map, and the visible rect always keeps the original 28-tile blur padding | 3 | done — surface/cave panning at zoom 0.25: light **3.31 → 0.03 ms**, draw **4.13 → 0.74 ms** (600-frame release runs, `TR_PAN=0.25`; `TR_NOCACHE=1` reproduces the old full path) |
| D3 | Incremental light on camera movement (overlap reuse + dirty borders) | 3 | done (32-tile valid-region reuse; panning stress light 3.31 → 0.03 ms) |
| D5 | GPU light: masks/seeds as textures, blur as parallel compute dispatches | 3 | evaluated — with D3 light is **0.01–0.03 ms** at zoom 0.25–1.5 while the present floor is ~1.1 ms; moving the blur to the GPU cannot improve real-zoom wall-clock and risks float-order pixel drift. Deferred until larger worlds/zoom need it |
| D6 | Deferred lighting: unlit chunk meshes + screen-space light texture | 3 | evaluated — same reason as D5; the CPU light is already cached and the frame is present-bound at zoom 1.0–2.0. Deferred |
| F1 | Chunk meshes (32×32/64×64), rebuilt only on tile edits | 3 | partial — sky split into a back range and the world range cached by tile/light generation (72 % reuse while panning, 98 % static). Per-chunk baking evaluated: the remaining rebuild is 0.02–0.03 ms at zoom 1.5 and 0.28 ms in the 0.25 stress view, below the present floor; deferred |
| B7 | Texture array instead of atlas — internal GPU storage only, must be pixel-identical; enables B6 and cuts cache pressure | 3 | evaluated — one atlas/one bind and no texture-bound cost measured; variable sheet sizes make a pixel-identical array require a layered atlas (same sampling cost) and would touch every UV. Deferred |
| B8 | Manual `fract` UV wrapping for merged runs (nearest, no mips) | 3 | deferred — only matters after B6 merged runs, which depends on B7 |
| B10 | Crop/repack the atlas to frames actually used by the world | 4 | evaluated — a deterministic skyline repack fits the same 186 sheets into 8192×1984 (**113.3 → 65.0 MB**, −43 %, 23 ms), and cave/lake screenshots stay pixel-identical, but the normalised UV phase moves by <1 texel and flips 166 background pixels at the forest horizon (surface, zoom 2.0). Layout-invariant pixel-space sampling is a prerequisite; deferred |
| E1 | Pack `Tile` 16 → 8–10 bytes (bitfields) | 4 | partial — 14 → **12 bytes** by packing slope/paint and the wall frame cell; world memory **70.6 → 60.5 MB**. Going below 12 needs narrower frame fields (only safe for our asset range) — deferred |
| E2 | SoA/chunked tile storage for the light and draw scans | 4 | planned — the 12-byte row-major tile already scans sequentially; a full SoA refactor is a world-model change with no real-zoom frame gain over D3 + the world cache |
| F3 | GPU tilemap (tile metadata in a texture, static grid, vertex-shader culling) | 4 | optional — 2–80 k instances; not needed at the current scale |
| G2 | Present-mode / frame-latency tuning under `AutoVsync` | 1 | done (default latency 1, `TR_LATENCY` override) |
| G3 | Push constants instead of the two globals UBOs | 1 | evaluated — 2×32 B/frame is below noise |
| G5 | Avoid recreating per-frame descriptors where the API allows | 1 | evaluated — surface views have no stable handle |
| G6 | Skip instance rebuild when camera/light/world are unchanged | 2 | done (static camera reuses world range) |

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

✅ *First pass*: fixed 60 Hz gravity/acceleration/friction/jump, 1-tile auto-step,
axis-separated AABB tile collision, water buoyancy, respawn, camera follow and the starter
character composed from the original `Player_0_*` strips with walk/jump frames.

✅ *Second pass*: breath/drowning, fall damage, lava damage and out-of-combat health
regeneration. ✅ *Third pass*: hold-to-jump height, composite front/back arms, screen-pixel
snapping and the classic HUD (hotbar, hearts, breath bubbles, boss bar).

The selected block is held in the hand with a swing arc while mining, placing or attacking.

Remaining: the full `PlayerDrawLayers` order (armor/accessories, composite-arm rotation),
the full `itemAnimation`/use timings, buffs, hooks/mounts/gravity flips, and the rest of
the 55k-line `Player.cs`.
*~12–24 agent-days.*

## Milestone 3 — items, inventory, crafting, UI

✅ *Second sandbox pass*: 10-slot dynamic hotbar, LMB mining with per-kind timers, RMB
placement with reach/support/overlap checks, drops as physics entities with magnet pickup
into matching/empty slots, placement consumed, local 5×5 re-framing, target outline and
F9/save-on-close to `.trwld` + a 10-slot `.trplayer` sidecar.

✅ *HUD pass*: classic hotbar/hearts/boss bar. Remaining: tool tiers/durability and proper
`Item.cs` use logic, the full inventory UI, chests, crafting (`Recipe.cs`), prefixes, equipment slots, paint and actuators. A playable subset (tools, weapons, blocks,
potions, ~200 items) is **~40–80 agent-days**; full item parity is ~50 000 lines.

## Milestone 4 — NPCs, combat, projectiles

✅ *First pass*: a hopping blue slime, a flying demon eye and a walking zombie (both night
spawns; the zombie burns in daylight), contact damage, player HP, hurt cooldown/knockback/
respawn, LMB melee, the first projectile (a throwable shuriken with tile bounces), the
first boss (King Slime: hops, teleport, minions, boss bar), coin loot into a purse,
overhead HP bars and a HUD health bar. NPCs
spawn near the player and respawn up to four alive. Data-oriented entity arrays are still
the plan for the full roster.

Remaining: the rest of `NPC.cs` (~780 types, AI states, spawn/biome/night rules), full
loot tables/item drops, the rest of the projectile system (~1 100 types), damage
types/defense/crits, buffs/debuffs, the rest of the bosses, town NPCs/shops. **~60–120 agent-days** for a first playable subset; full parity
several times that.

## Milestone 5 — world generation

A compact native generator and `.trwld` save exist and are used by default: height-noise
forest surface, dirt/stone/ash layers, caves, ores, walls, a forest lake, lava puddles,
common + snow-biome trees, surface plants, sand ocean beaches at both edges, a snow biome
(snow surface, winter trees, pine background) and an inland desert (dunes, sand/hardened
sand/sandstone, sandstone walls, dune background).

Porting the remaining generation passes and structures (`WorldGen.cs` + `Generation/*`:
corruption/crimson/jungle/hallow biomes, the full ocean biome, desert structures and loot,
dungeons, temples, living trees, hardmode spread, chests) is **~40–90 agent-days** for a
passable generator; full parity much more. Loading existing `.wld` files still works at any
time.

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

1. M1.1 autotiling ✅ → M1.2 lighting ✅ → M1.3 liquid flow ✅ → M1.4 sky/backgrounds/
   weather ✅.
2. M2 player ✅ movement, survival, half bricks, held item; next: full draw layers/use.
3. M3 hotbar/drops/walls ✅; next: inventory/crafting UI and a real item registry.
4. Pixel-exact UI fonts: `tr-content` ports the original XNB readers (container, LZX,
   `Texture2DReader`, ReLogic `DynamicSpriteFontReader` from `decompiled-libs/ReLogic`)
   so textures and fonts load from the retail `Content` directory at runtime instead of
   pre-extracted PNGs; then HUD/inventory text switches to `Mouse_Text`/`Item_Stack`.
   Status: in progress (crate `tr-content`, verification = pixel-identical decode of
   `Images/Tiles_0.xnb` vs the extracted PNG).
4. M4 three enemies + King Slime + shuriken ✅; next: more of `NPC.cs` and projectiles.
5. M5 forest/snow/desert ✅; next: corruption/jungle/structures and hardmode spread.
6. Everything else.

## Non-goals for now

- Modding API, tModLoader compatibility.
- Mobile/console parity.
- Exact network compatibility (single-player first).
