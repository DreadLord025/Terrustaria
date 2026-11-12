# 10 — Lighting

> Wave 1 · researched 2026-11-12 · sources: decompiled + crates + ROADMAP/ANALYSIS/PORT_MAP

## Scope

Lighting is the per-tile light simulation and its consumption by the tile renderer.

C#: `decompiled/Terraria/Lighting.cs` (586 lines — the *mode selector*, owns
`Lighting.Mode`, `AddLight`, `GetColor*`, `Get*Slice`, `GetCornerColors`),
`decompiled/Terraria/Graphics/Light/LightingEngine.cs` (281 — the new RGB engine, `ProcessArea`
split into 4 rotating engine states), `Graphics/Light/LightMap.cs` (267 — buffer layout + the
directional blur + per-medium decay), `Graphics/Light/TileLightScanner.cs` (3318 — the per-tile
seed light: `ApplyTileLight` is ~2800 lines of per-type dispatch), `Graphics/Light/LegacyLighting.cs`
(1442 — the White/Retro/Trippy legacy engine), `Graphics/Light/ILightingEngine.cs`, `LightMode.cs`,
`LightMaskMode.cs`. Config load/write of the mode lives in `Main.cs:4398-4403` (read) /
`Main.cs:4133` (write).

Rust: `crates/tr-core/src/lighting/` — `mod.rs` (842: `LightEngine` trait ≈ `ILightingEngine`,
`LightMap`, seeding, slice/corner helpers, cache), `params.rs` (312: `LightMode`, `LightStyle`,
`LightParams`), `blur.rs` (209: the two blur passes + `Mask` medium decay); `crates/tr-core/src/
tile_light/` — `mod.rs` (310: wall/surface/hell seeds, `light_blocked`), `scanner.rs` (1255:
`apply_tile_light` dispatch), `palette.rs` (327: `FastRandom`, torch/paint colour tables).
Renderer consumption: `crates/tr-app/src/render/tiles.rs` (1769: `push_lit`, `push_sliced`,
quality thresholds, light stats).

## Key entities

- `Terraria::Lighting.Mode` setter (`Lighting.cs:29-61`) — the mode selector: `Color` → `NewEngine`
  (and `LegacyEngine.Mode = 0`, `OffScreenTiles = 35`); `White`/`Retro`/`Trippy` → `LegacyEngine`
  with `Mode = 1/2/3`. Also `Lighting.cs:63-87` (`NotRetro`, `UsingNewLighting`,
  `UpdateEveryFrame`), `:89-98` `Initialize`, `:100-107` `LightTiles`, `:109-116`
  `UpdateGlobalBrightness` (1.2 default, 1.0 when `blind`), `:124-133` `GetSubLight`,
  `:165-173` `NextLightMode` (wraps to `White`).
- `Terraria.Graphics.Light.LightingEngine` (`LightingEngine.cs:76-108`) — 4-phase rotation
  (`MinimapUpdate` → `ExportMetrics` → `Scan` → `Blur`, `:110-113`), `AREA_PADDING = 28` (`:29`),
  `NON_VISIBLE_PADDING = 18` (`:31`), `ProcessScan` `area.Inflate(28,28)` + scanner `ExportTo`
  (`:115-126`), `UpdateLightDecay` (`:141-206`), `ApplyPerFrameLights` (`:208-235`, one-frame
  swap of the `AddLight` list), `ExportToMiniMap` byte = `max(r,g,b)*255` (`:246-270`).
- `Terraria.Graphics.Light.LightMap` (`LightMap.cs`) — `_colors: Vector3[]` + `_mask:
  LightMaskMode[]`, `IndexOf(x,y) = x*Height + y` (`:252-255`, **column-major**), `Blur()` =
  two `BlurPass` + `_random.NextSeed()` (`:86-91`), `BlurPass` = 4 line sweeps per axis
  (`:93-111`), `BlurLine` carries the per-channel max with the `0.0185` cutoff and a mask switch
  (`:113-250`); the `Water` mask multiplies by `_random.WithModifier((ulong)i).Next(98,100)/100f`
  (`:219`).
- `Terraria.Graphics.Light.TileLightScanner` — `ExportTo` (`:17-41`), `GetTileLight`
  (`:87-106`: surface light if `y <= Main.worldSurface`, else hell light if
  `y > Main.UnderworldLayer`, then wall, then active-tile, then liquid), `ApplyWallLight`
  (`:164-337`), `ApplyTileLight` (`:339-3129`), `ApplySurfaceLight` (`:3131-3238`),
  `ApplyHellLight` (`:3240-3317`), `LightIsBlocked` (`:151-162`), `GetTileMask` (`:66-85`).
- `Terraria.Graphics.Light.LegacyLighting` (`LegacyLighting.cs`) — `Mode` (`:202`),
  `UpdateLightToSkyColor` sun/moon seeding to `Main.worldSurface` (`:435-471`), `DoColors`
  (`:851`) with the four mode swipes `doColors_Mode0..3_Swipe` (`:1087/1192/1273/1331`),
  `negLight = 0.91`, `negLight2 = 0.56` (`:868-869`).
- `Terraria::Lighting.GetColor9Slice` (both overloads `:244-272` Color, `:274-285` Vector3),
  `GetColor4Slice` (`:366`, `:530-586`), `GetCornerColors` (`:287-365`), `GetColor`/`GetColorClamped`.
- Rust: `tr_core::lighting::LightEngine` (`lighting/mod.rs:48-62`), `LightMap`
  (`:79-425`: `get` `:131`, `slice9` `:150`, `corner4` `:171`, `quad4` `:199`,
  `compute_into` `:252`, `compute_into_with_lights` `:270`, `compute_into_impl` `:300-424`),
  `DynamicLight` (`:432`), `seed` (`:469-501`), `liquid_light` (`:507-540`);
  `lighting::params::{LightMode, LightStyle, LightParams, GLOBAL_BRIGHTNESS}` (`params.rs:12-260`);
  `blur::{blur, blur_line_v, blur_line_h, surface_cutoff, mask_of}` (`blur.rs:42-209`);
  `tile_light::{apply_wall_light, apply_surface_light, apply_hell_light, light_blocked,
  wall_is_light}` (`tile_light/mod.rs:28-276`), `tile_light::scanner::apply_tile_light`
  (`scanner.rs:36`), `tile_light::palette::FastRandom` (`palette.rs:19-100`).
- Renderer: `render/tiles.rs::push_lit` (`:1219-1332`), `push_sliced` (`:1340-1391`),
  quality thresholds `HIGH_LIGHT = 142/255` / `MEDIUM_LIGHT = 26/255` (`:363-366`),
  `waterfall_lights` previous-frame `AddLight` list (`:51-55`, `:320-334`).

## Dependencies

- Calls into: `tiledata` sets (`TILE_BLOCK_LIGHT`, `TILE_NO_SUN_LIGHT`, `WALL_LIGHT`,
  `SET_CRACKED_BRICKS`, `SET_ALLOW_LIGHT_IN_WATER`, `WALL_COUNT`), `tile_light::FastRandom`,
  `world::{column_surface, world_surface, world.height, wall_paint, invisible_block/wall}`,
  `rayon` (parallel seeding + horizontal sweeps), `LightParams` (the `Main` snapshot).
- Called by: `tr-app/render/tiles.rs` (`build_world_range`: light compute → `push_lit` /
  `push_sliced` / `push_lit_wall`), `menu` debug overlay (`env::light_at`, `light_ms`,
  `lit_tiles` stats), `tr-core::waterfall` (`DynamicLight` feed).
- Shared data structures: `LightParams`/`LightStyle` (params), `LightKey` (bit-exact cache
  identity), `DynamicLight` (≈ `Lighting.AddLight` seeds), `InvisibleBits` (Echo bits from
  `world.extras`), `LightMap` (≈ `LightMap` + `LightingEngine._activeLightMap`).

## Port status

| C# subsystem | Status | Rust location | Evidence / notes |
| --- | --- | --- | --- |
| Mode selector (Classic/RGB/White/Color) | PORTED | `lighting/params.rs::LightMode` + `LightStyle` (`:12-112`) | 4 modes with labels + `next()` ≈ `Lighting.Mode` setter (`Lighting.cs:29-61`) and `NextLightMode` (`:165-173`). Settings load (`Main.cs:4398-4403`) maps to `settings.rs`/`menu` (11 hits). |
| Legacy engine (White/Retro/Trippy) | PARTIAL | `params.rs::LightStyle::shade` (`:116-148`) | No port of `LegacyLighting.cs` (1442 lines, `:202`, `DoColors` `:851`, `doColors_Mode0..3_Swipe`). White is a luminance collapse, Retro a 16-level quantise, Trippy a sinusoidal channel mix — approximations of the legacy renderer, not the algorithm. |
| `LightingEngine` state machine | PARTIAL | `lighting/mod.rs::LightEngine::compute` (`:48-62`, `:300-424`) | Scan+Blur+Present semantics ported (seed → dynamic-light max → 2 blurs), but the 4-phase rotation (minimap export / scene metrics split over frames) is absent; the whole computation is one call. `Rebuild`/`AddLight`/`Clear` have no counterpart (documented `mod.rs:46-47`). |
| `LightMap` buffer + blur | PORTED | `lighting/blur.rs::blur/blur_line_v/blur_line_h` (`:42-163`) | Two passes × 4 sweeps, column-then-row order, `0.0185` cutoff, per-channel `done` flags, carried-max update — matches `LightMap.cs:86-250`. Row-major storage vs C# `x*Height+y` (`:252-255`) with serial vertical sweeps (`blur.rs:41`). |
| `LightMap` decay per medium | PARTIAL | `blur.rs::Mask::decay` (`:26-37`) | Air 0.91 / Solid 0.56 / CrackedBricks 0.8 / Honey / Water-hardcoded-to-style-0. C# `LightingEngine.UpdateLightDecay` (`:147-182`) varies water decay per `Main.waterStyle` (13 arm values); nightVision ×1.03 / blind ×0.95 / blackout+headcovered ×0.85 (`:184-203`) and `airLightDecay`/`solidLightDecay` (`:204-205`) are not in `LightParams` — missing. |
| `TileLightScanner` seeds | PORTED | `tile_light/{mod,scanner,palette}.rs` | Wall table (357/182/33/174/175/176/137/44/154-166/153/341-346 + paint), surface (wall gate 88-93/241 gem themes + `shimmerDarken`), hell glow `0.55 + sin(2t)*0.08`, liquid lava/shimmer, `light_blocked` echo rule — all match `TileLightScanner.cs:87-3317` incl. the invisible-wall/block gates (`:3142/3149/3246`). |
| Per-tile `FastRandom` palette | PORTED | `tile_light/palette.rs::FastRandom` (`:19-100`) | `25214903917/11/281474976710655`, `next_bits` 48-shift, `WithModifier(x,y)` (`:44-50`), `with_modifier_u64` = `next_seed ^ seed` (`:56-60`); test `fast_random_matches_the_original_stream` pins the stream. |
| `GetColor9Slice` / `GetColor4Slice` / `GetCornerColors` | PORTED | `LightMap::slice9`/`quad4`/`corner4` (`mod.rs:150-221`) | Slice index mapping `(dx+1)+3*(dy+1)` = C# num-stride pattern (`Lighting.cs:244-272`); darker-neighbour rule in `quad4` per `:530-586`; `corner4` ≈ `GetCornerColors` 4-way average (`:299-301`, `*63.75`). |
| Draw-path blending (LightMap → tints) | PARTIAL | `render/tiles.rs::push_lit` (`:1219`), `push_sliced` (`:1340`) | Thresholds match `_highQualityLightingRequirement`/`_mediumQualityLightingRequirement` (`:363-366` ≈ `TileDrawing.cs:3524/3605`); `SLICE_POS` 4/8/4 grid == C# `sourceRectangle` switch (`:3534-3577`). C# per-slice `tileLight = (slice+own)*0.5` (`:3578-3580`) — Rust pre-averages inside `slice9` (`mod.rs:157-161`), algebraically equal, **but** C# then runs `GetFinalLight` per slice (`:3581`) and the Rust slice path does not re-apply the actuated ×0.4 / `light_scale` (only `push_lit` does, `tiles.rs:525-531`). |
| `AddLight` point lights (NPC/projectile/dust) | MISSING | `DynamicLight` fed only by waterfalls (`render/tiles.rs:51-55`) | C# call sites: Projectile 107, Dust 94, NPC 80, WorldItem 69, Player 46, Mount 14, WaterfallManager 11… (`Lighting.cs:145-163`). Only the waterfall feed exists; entity/particle glows are not seeded. |
| Per-frame light swap (one-frame delay) | PORTED | `tiles.rs:320-334` | `waterfall_lights` is the previous frame's list, mirroring `ApplyPerFrameLights` (`LightingEngine.cs:208-235`), incl. the capture exception. |
| `subLight` bilinear sample | MISSING | — | `Lighting.GetSubLight` (`Lighting.cs:124-133`) not found in `render/` (nearest-tile `LightMap::get_px` only). |

## Tests

`crates/tr-core/src/lighting/mod.rs`: `open_sky_is_lit`, `solid_ground_shadows_below`,
`torch_lights_the_room`, `parallel_light_is_bit_identical_to_sequential` (bit-identical
seeding/blur on Rayon), `lava_glows_and_hell_is_warm`,
`invisible_block_is_transparent_when_hidden`, `hidden_invisible_block_still_receives_sun`,
`hidden_invisible_wall_admits_sun`, `nine_slice_averages_with_neighbours`,
`panning_within_the_margin_reuses_the_cache`, `light_engine_seam_matches_the_map`.
`lighting/params.rs`: `light_styles_transform_samples` (White collapse, Retro quantise).
`tile_light/mod.rs`: `shimmer_and_crystal_walls_flicker_with_mouse_text_color`.
`tile_light/palette.rs`: `fast_random_matches_the_original_stream`,
`hsl_to_rgb_matches_primary_colours`, `fast_random_next_n_matches_the_next_zero_min_range`.
`tile_light/scanner.rs`: `torch_variants_use_the_frame_row`,
`demon_and_disco_torches_are_animated`, `candles_and_lamps_follow_their_frame_tables`,
`gemspark_encodes_colour_in_the_frame_column`, `lantern_frame_sets_the_colour`,
`campfire_only_lights_its_lit_frames`, `dw_1_5_dial_light_needs_zero_cooldown`.
No `render/tiles.rs` test targets the sliced/flat light paths or the 9-slice geometry.

## Deviations & risks

1. **Entity `AddLight` sources missing** (behavioural, high) — 300+ C# call sites project light
   from NPCs, projectiles, dust, items; the Rust map only accepts waterfall lights. Glowing
   entities/explosions are dark at night. Not admitted in ROADMAP §4.
2. **Water decay is style-0 only** (pixel, medium) — `blur.rs:33` hardcodes
   `(0.88,0.96,1.015)*0.91`; C# `LightingEngine.cs:147-182` has 13 arms (blue/red water styles
   etc.). Any non-default `waterStyle` diverges. Not admitted.
3. **Debuff/buff decay multipliers absent** (behavioural, medium) — nightVision ×1.03, blind
   ×0.95, blackout/headcovered ×0.85, `airLightDecay`/`solidLightDecay`
   (`LightingEngine.cs:184-205`) are not in `LightParams`/`LightKey`. ROADMAP "Hidden" row
   (`:210-211`) admits the *legacy-engine* half (`LegacyLighting.cs:944-958`) but not the new
   engine's.
4. **Water jitter is an approximation** (pixel, medium) — C#
   `_random.WithModifier((ulong)i).Next(98,100)/100` ∈ {0.98, 0.99} (`LightMap.cs:219`); Rust
   uses `(i*2654435761)%15 → 0.985..0.999` (`blur.rs:95-98`, `:136-139`). (Note C#'s
   `_random` is `CreateWithRandomSeed()` so vanilla's jitter is session-nondeterministic;
   Rust's is deterministic — arguably better, still a divergence.)
5. **`NON_VISIBLE_PADDING` clamped** (pixel, low) — `blur.rs:43` uses
   `min(18, min(w,h)/3)`; C# is a fixed 18 (`LightMap.cs:31`, `:99-108`). Only tiny light
   rects differ.
6. **Per-column surface cutoff instead of global `Main.worldSurface`** (behavioural, medium) —
   `lighting/mod.rs:490-494` seeds sunlight up to each column's own surface/SDF cutoff; C#
   `TileLightScanner.cs:92` uses the single global `Main.worldSurface`. Comment admits the
   intent (hills/valleys), but it *is* a pixel change; needs a deterministic A/B.
7. **Sliced path misses per-slice `GetFinalLight`** (pixel, medium) — C# applies
   `GetFinalLight(tileCache, typeCache, ref tileLight, ref tint)` on every slice
   (`TileDrawing.cs:3581`, `:4391-4397`); Rust `push_sliced` (`tiles.rs:1367`) only calls
   `LightStyle::shade`, so paint tint / `light_scale` / actuated ×0.4 do not reach the 9 slices.
8. **White/Retro/Trippy are shader approximations** (behavioural, medium) — the 1442-line
   `LegacyLighting` engine (sky-colour swipe, data shifting, `doColors_Mode*`) is unported;
   ROADMAP §4 (`:197-198`) claims "4 modes", which is true only of the style descriptors.
9. **4-phase engine split collapsed** (behavioural, low) — minimap light export and scene
   metrics run in separate phases in C# (`LightingEngine.cs:76-108`); Rust computes light on
   demand per drawn rect, so minimap lighting export ordering differs.
10. **`GetSubLight` (bilinear sub-tile sample) missing** (cosmetic, low) — used by a few
    per-pixel consumers in C# (`Lighting.cs:124-133`); not found in the renderer.

## Task seeds

- **[M] Feed entity/projectile/item `AddLight` sources** — port the hot C# sites
  (Projectile 107 / NPC 80 / Dust 94 / Player 46 / WorldItem 69) into a per-frame
  `Vec<DynamicLight>` and call `compute_into_with_lights` (depends on the fixed-step ordering
  in `world_sim/step.rs`; keep the one-frame-delay semantics of `LightingEngine.cs:208-235`).
  Highest-impact lighting gap.
- **[M] Water-style decay + buff/debuff decay in `LightParams`** — add `water_style`, the
  nightVision/blind/blackout/headcovered multipliers and `airLightDecay`/`solidLightDecay`
  to `params.rs` + `LightKey`, thread into `blur::Mask::decay`; unit-test style 2/6/9 arms.
- **[M] Legacy-engine golden test for White/Retro/Trippy** — capture `doColors_Mode*` byte
  output for a fixed light buffer (or hand-derive it) and assert `LightStyle::shade` matches
  per mode; decide whether the approximations stand or the legacy blit path must be ported.
- **[S] Exact water jitter** — replace the multiplicative hash with
  `FastRandom::new(seed).with_modifier(index).next_range(98, 100)/100` semantics in
  `blur_line_*`; precedent: `palette.rs` stream test.
- **[S] `slotGetFinalLight` in the sliced path** — apply actuation ×0.4 / `light_scale` /
  paint tint per slice in `push_sliced` (`tiles.rs:1340`), matching `TileDrawing.cs:3578-3601`.
- **[S] `NON_VISIBLE_PADDING` fixed 18** — drop the `/3` clamp in `blur.rs:43`.
- **[M] Per-column vs global surface cutoff A/B** — `TR_SHOT` pair with `world_surface`
  fallback; record the result in ANALYSIS.md; keep `surface_cutoff` only if the diff is
  intentional and documented.
- **[S] Sliced-path render test** — unit-test `push_sliced` quad count/geometry against the
  C# `sourceRectangle` table (4/8/4 grid) for both the high and medium quality arms.
