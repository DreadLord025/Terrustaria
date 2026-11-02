# Roadmap & honest scope

## Where agents take tasks from (branch-per-task)

**This file is the task catalog.** Pick any task from the catalog below (or take a
direct request from the user), create a branch named after it —
`git checkout -b <Task-Id>-short-slug` (e.g. `M1.4-BG-11-cloud-parallax`) — and follow
the workflow in `agents.md`: work on the branch, sync with main, resolve conflicts
thoughtfully, verify tests + screenshots on the merged state, merge into `main`,
delete the branch. A catalog row is "taken" when a branch with its id exists — check
`git branch -a` first. Rows marked 🟡 list what is *still missing*; read the linked
milestone section for the full context before starting.

## Task catalog

### Milestone 1 — visual fidelity

| Task | Branch | Scope | Notes |
| --- | --- | --- | --- |
| ✅ M1.4-BG-11/16-finish | `M1.4-BG-11-cloud-parallax` | cloud `addCloud`/`GetParallax` density, `DrawStarsInBackground` star density, horizon layer order | ✅ done 2026-11-01 (there is no `Cloud.Parse` in 1.4.5.8 — the spawn recipe is `Cloud.addCloud`; deviations in `ANALYSIS.md`) |
| ✅ M1.4-BG-13 | `M1.4-BG-13-grass-transition` | grass/dirt transition fidelity: `WorldGen.TileFrame` grass branches vs `framing.rs`, 9-slice light | done 2026-11-01: full `TileFrameCosmetic` grass chain (`WorldGen.cs:82184-84836`) ported; 9-slice light needed no change (tile-light scanner + `GetColor9Slice` already cover the seam); quirks in ANALYSIS |
| TILE-DRAW-1-finish | `TILE-DRAW-1-drawdata-finish` | per-type draw metrics in `tr-core/src/drawdata.rs`: finish torch 20×20/top 4, plants 20+flip, herbs, tables, candles, palm, cactus | 🟡 partially populated |
| TILE-DRAW-2 | ✅ merged `TILE-DRAW-2-torch-flames` | torch flames + glow masks: `tr-core/src/flame.rs` (`GetTileFlameData` + `DrawSingleTile_Flames` both tables, 48-bit LCG `TileFrameSeed` jitter deterministic via fixed seed), premultiplied `AlphaBlend` fix in tr-render | deviations in ANALYSIS.md "Torch flames (TILE-DRAW-2)" |
| ✅ TILE-DRAW-3-finish | `TILE-DRAW-3-wind-finish` | wind: finish swaying plants, tree wind, leaves (`tr-core/src/wind.rs`, `DrawGrass`, `EmitTreeLeaves`) | ✅ done 2026-11-02 (sway + leaves; the WindGrid push and the multi-tile/vine wind draws remain, see `ANALYSIS.md`) |
| TILE-DRAW-4 | `TILE-DRAW-4-paint-coatings` | `TilePaintSystemV2`, wall paint, coatings, echo/invisible flags | needs a paint byte on `Tile` (see E1) |
| TILE-DRAW-5 ✅ | `TILE-DRAW-5-grass-framing` | full grass framing (`WorldGen.cs:82184-84836`): groups A-J, the `-2` marker, per-biome dirt ids | done 2026-11-01 by the M1.4-BG-13 port (full override chains + the `-1`/`-2` tables) |
| M1.4-BG-18 | `M1.4-BG-18-screen-filters` | per-biome animated screen effects: `SceneState.ApplyVisuals` (desert heat distortion, underworld heat, blizzard/sandstorm/aurora/water distortion, monolith/story filters) | |
| ✅ M1.5-LIQUID-2 | `M1.5-LIQUID-2-water-styles` | water styles and shimmer: honey/shimmer `DrawWaterfall` passes (`Main.liquidAlpha[2..4]`), shimmer vertex-sparkle, `TrySparkling` dust, ambient lava-waterfall light | ✅ done 2026-11-02 (per-style waterfall + liquid passes, `DrawShimmer`, `AddLight`, `TrySparkling`; deviations in `ANALYSIS.md`) |
| M1.4-TO | `M1.4-TO-tile-objects` | `TileObjectData` (multi-tile furniture, doors, chests), gemspark/glow tiles | paint shares TILE-DRAW-4 |

### Milestone 2 — player

| Task | Branch | Scope |
| --- | --- | --- |
| M2-DRAW-1 🟡 | `M2-DRAW-1-player-layers` | full `PlayerDrawLayers`: armor/accessories draw | layering done 2026-11-02: order + weapon slot + composite front arm + bob (`player_layers.rs` builders, tests); armor/accessories go live with M3-EQUIP; composite shoulder rotation is M2-DRAW-2 |
| M2-DRAW-2 ✅ | `M2-DRAW-2-arm-rotation` | composite arm rotation | done 2026-11-02: `CompositeArmData` draw-side in the layer builders (columns 7/8 × stretch rows, pivot rotation, `PlayerDrawSet.cs:2246-2271`); the rotation source is a swing-arc stand-in until M2-ANIM |
| ✅ M2-ANIM-item-timings | `M2-ANIM-item-timings` | full `itemAnimation`/use timings (hold-style items) | done 2026-11-02: per-item `useTime`/`useAnimation`/`useStyle`/`holdStyle`/`autoReuse` from `Item.cs SetDefaults1`, `SWING_TIME` retired for the remaining-fraction plumbing, autoReuse/release cadence, torch rest-hold (`Player.cs:50275-50460`) + body row 3 (`:36455`); deviations in ANALYSIS "Item use timings (M2-ANIM-item-timings)" |
| ✅ M2-ANIM-foundation | `M2-ANIM-foundation` | animation foundation + fixes: `tr-core/src/anim.rs` primitives (`FrameCounter`, band wrap, useStyle-1 thirds), faithful player draw origin (`PlayerDrawSet` + gfxOffY — the "pixels shift" bug), held-item swing (`ItemCheck` `itemRotation`), `NPC.FindFrame` for the four ported kinds, HUD hotbar through the faithful `ItemSlot.Draw` path (one stack-text size/position, 32 px icon clamp) | done 2026-11-02; deviations in ANALYSIS "Animations (M2-ANIM-foundation)" + "Closed hotbar and mouse item"; per-item `useAnimation` timings stay with M2-ANIM-item-timings |
| M2-BUFF | `M2-BUFF-buffs-mounts` | buffs/debuffs, hooks, mounts, gravity flips |

### Milestone 3 — items, inventory, crafting

| Task | Branch | Scope |
| --- | --- | --- |
| M3-CRAFT-finish | `M3-CRAFT-recipes` | `Recipe.cs` crafting menu: recipe data + `ItemSlot` contexts 4/33 interaction (🟡 panel chrome is done — `ui/crafting.rs`) |
| M3-TOOL | `M3-TOOL-tiers` | tool tiers and durability, proper `Item.cs` use logic |
| M3-EQUIP | `M3-EQUIP-equipment` | equipment slots and paint |
| M3-ACTUATOR | `M3-ACTUATOR` | actuators |
| M3-PREFIX | `M3-PREFIX` | item prefixes system |
| M3-CHEST-UI | `M3-CHEST-UI` | chest UI (banks/chests were excluded from whole-UI P2) |

### Milestone 4 — NPCs, combat, projectiles

| Task | Branch | Scope |
| --- | --- | --- |
| M4-NPC-1 | `M4-NPC-1-more-npcs` | remaining `NPC.cs` types (780 total: AI states per family — split into per-family branches, e.g. `M4-NPC-1-fighters`) |
| M4-PROJ | `M4-PROJ-projectiles` | the rest of the projectile system (~1 100 types) |
| M4-LOOT | `M4-LOOT-loot` | full loot tables / item drops |
| M4-DAMAGE | `M4-DAMAGE-damage` | damage types / defense / crits |
| M4-BOSS | `M4-BOSS-bosses` | the remaining bosses |
| M4-TOWN | `M4-TOWN-town` | town NPCs and shops |

### Milestone 5 — world generation

| Task | Branch | Scope |
| --- | --- | --- |
| M5-BIOME | `M5-BIOME-evil-jungle` | corruption/crimson/jungle/hallow biomes (world gen + bg variants M1.4-BG-6) |
| M5-OCEAN | `M5-OCEAN` | the full ocean biome |
| M5-DESERT | `M5-DESERT-structures` | desert structures and loot |
| M5-DUNGEON | `M5-DUNGEON` | dungeons |
| M5-TEMPLE | `M5-TEMPLE` | jungle temples |
| M5-LIVING | `M5-LIVING-trees` | living trees |
| M5-HARDFORCE | `M5-HARDFORCE` | hardmode spread |
| M5-CHEST | `M5-CHEST-gen` | chest generation and loot |

### Milestone 6 — audio, localization, networking

| Task | Branch | Scope |
| --- | --- | --- |
| M6-MUSIC-finish | `M6-MUSIC-biome-tracks` | remaining biome music branches (need `SceneMetrics`), ambient rain/wind cues |
| M6-LOCALIZE-wire | `M6-LOCALIZE-wire` | wire the localization core into HUD/item names/menus (core is ported + tested) |
| M6-NET-protocol | `M6-NET-protocol` | the exact 1.4.5.8 network protocol; includes the multiplayer-flow backlog: net-Disconnect must not open the error menu over a live front screen (`self.front` unchecked in the ClientEvent::Disconnect guard — P6 whole-branch review minor 1), and the parked multiplayer Back targets (Join/Host & Play → character/world selects) |
| M6-SETTINGS | `M6-SETTINGS` | settings application parity beyond the ported keys |

### Engineering debt (from the 2026-11-01 architecture refactor)

| Task | Branch | Scope |
| --- | --- | --- |
| REFACTOR-env | `refactor-app-env-flags` | centralize the ~65 `TR_*` env flags into one struct with OnceLock "read once" semantics (`app/env.rs`; skipped during the refactor as risky) |
| REFACTOR-rng | `refactor-xorshift-unify` | unify the worldgen/wind xorshift cores byte-exactly |
| REFACTOR-saveheader | `refactor-save-list-reader` | absorb the last duplicate reader (`ui/save_list.rs` `HeaderReader`) into `tr_core::binio` |
| REFACTOR-cleanup | `refactor-allow-cleanup` | remove the inert `#[allow(dead_code)]` attributes in `ui/uistate.rs` (T1-era; the surface is consumed by the P6 screens) |

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
2. **Lighting** — ✅ *third pass done*: `TileLightScanner` seeds (surface/hell/wall/
   emissive/liquid), `LightMap`'s two-pass four-sweep blur with per-mask decay,
   `Main.SetBackColor` → `ApplyColorOfTheSkiesToTiles`, per-column surface seeding,
   `GlobalBrightness = 1.2`, `GetColor9Slice`/`GetColor4Slice` block slicing and all four
   `LightMode`s (Color/White/Retro/Trippy, F2). Verified numerically with `TR_LIGHT_AT`.
   The whole `ApplyTileLight` switch (147 cases) is ported in `tile_light.rs`, together
   with `ApplyWallLight`/`ApplySurfaceLight`/`ApplyHellLight`, the `FastRandom` per-tile
   jitter, paint-coloured lamps, disco/demon/shimmer dynamics and the animated
   `mouseTextColor` flicker (M1.5-LIGHT-1/2).
   Remaining: sun/moon/star light, invisible blocks/walls (not stored in the packed tile),
   the jellyfish-cage animation state, shader chromatic aberration once instances carry a
   flag. *~1 agent-day of ~8 left.*
3. **Liquids** — ✅ *fourth pass*: `LiquidRenderer.InternalPrepareDraw` is ported as
   `liquid_draw.rs` (visible levels, edge walls, `FrameOffset`, source rect, waterfall-fade
   opacity, `IsSurfaceLiquid`), `Lighting.GetCornerColors` drives 3x3 bilinear light and the
   draw runs twice like `Main.DrawLiquid` (opaque behind walls/tiles, `DEFAULT_OPACITY`
   over entities) from the original `Misc/water_*` frames. The generator fills a forest lake
   and lava puddles;    `Liquid.UpdateLiquid`/`Liquid.Update` is a 30 Hz active-cell simulation
   (gravity, six-neighbour leveling, kill/retire, overflow buffer) and edits wake neighbours.
   The `Waterfall_*` overlay is ported as `waterfall.rs` (`WaterfallManager.FindWaterfalls`
   /`DrawWaterfall`): the 30-tick rescan around the view, half-brick spouts and the
   196/460/717 fountain tiles, the lava/honey/shimmer types, the rain/snow/disco frame
   counters, the per-row geometry walk with its 1 px wobble, `GetAlpha`/`StylizeColor`
   and the per-tile light tint; the app draws the quads between the non-solid and solid
   tile passes (`Main.DoDraw_Waterfalls`).
   ✅ *fifth pass*: the per-style `DrawWaterfall` passes behind the `liquidAlpha`
   cross-fade, the `liquidAlpha` multi-pass liquids, `DrawShimmer` (the wave and
   rainbow-glitter vertex colours), `TrySparkling` sparkles and the waterfall
   `AddLight` glow. Remaining: the shimmer vertex-distortion shader
   (`Main.DrawTileInWater`'s full effect) and the `HorizonClouds` fall tint.
   *<1 agent-day left of ~8.*
4. **Backgrounds & sky** — ✅ *fourth pass*: `Background_0` gradient, the
   `Cloud.addCloud`/`GetParallax` cloud field drawn in the three vanilla `DrawSurfaceBG`
   passes, `Star.SpawnStars`/`DrawStar` stars, sun, moon, the 15/9-minute day-night clock,
   the per-column cave fade, rain (`Rain.MakeRain`/`Rain.Update`/`Main.DrawRain`,
   `cloudAlpha` ramp, rotated drops, splashes, overcast `SetBackColor` dimming, snow-biome
   `Dust` snowfall, storm cloud banks and lightning flashes), the vanilla `DrawSurfaceBG`
   composition for the biomes the world has: forest (`7`/`8` mountains + `50..52` tree
   lines), snow (`35`/`36` + `37..39` pines) and ocean (`28` sea horizon), with
   `backgroundTopMagicNumber` vertical parallax and measured ground-band anchoring, and
   the textured `Main.DrawBackground` underground layers. Remaining: the storm type
   branch/rare clouds and the `HorizonClouds` cloud shader (weather/boss state),
   corruption/crimson/jungle/hallow variants (no such biomes yet), wind gusts and
   weather audio. *~1–2 agent-days.*
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

## Milestone 6 — audio, map, networking, polish ✅ first passes landed

The runtime `SoundEffectReader` and eight original PCM cues play through `rodio`; the
explored-tile minimap/full map and a host-authoritative TCP host/join flow are working.
✅ XACT/MS-ADPCM music playback (`tr-content/xact.rs` + `adpcm.rs`, `audio::MusicSystem`)
ports `Main.UpdateAudio`'s fade/selection/loop over the retail wave bank, and `sound.rs`
ports `LegacySoundPlayer.PlaySound`'s 2500 px attenuation and pan (M6-MUSIC/M6-3DAUDIO).
✅ The localization core (`GameCulture`/`LocalizedText`/`VariableText`/
`LanguageManager`, `tr-core/src/localization.rs`) loads the shipped
`Terraria.Localization.Content.*.json` resources and is tested against the seven `en-US`
files; the strings are not yet wired into the HUD, item names or menus. Still to port:
the remaining biome music branches (they need `SceneMetrics`), the ambient rain/wind cues,
the exact 1.4.5.8 network protocol, and settings. **~30–80 agent-days.**

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
3. M3 hotbar/drops/walls ✅; inventory UI + 50 slots + `ItemID` registry ✅; next:
   `Recipe.cs` crafting menu (`ItemSlot` contexts 4/33) and equipment slots.
4. Pixel-exact UI fonts: `tr-content` ports the original XNB readers (container, LZX,
   `Texture2DReader`, ReLogic `DynamicSpriteFontReader`) so textures and fonts load from
   the retail `Content` directory at runtime instead of pre-extracted PNGs; HUD and
   inventory text now uses `Mouse_Text`/`Item_Stack`. ✅ (0-diff decode test for
   `Images/Tiles_0.xnb`; fonts parse as 212-glyph DXT3 pages).
5. M4 three enemies + King Slime + shuriken ✅; next: more of `NPC.cs` and projectiles.
6. M5 forest/snow/desert ✅; next: corruption/jungle/structures and hardmode spread.
7. Everything else.

## Non-goals for now

- Modding API, tModLoader compatibility.
- Mobile/console parity.
- Exact network compatibility (single-player first).
