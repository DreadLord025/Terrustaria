# 02 — Main draw pipeline / scene layers / effects

> Wave 1 · researched 2026-11-12 · sources: decompiled (Main.cs / Lighting.cs), crates (tr-app, tr-core, tr-render), ROADMAP/ANALYSIS/PORT_MAP rows for M2-DRAW*

## Scope

The C# side is `Terraria/Main.cs` (~57 k lines): `Draw` (`Main.cs:61644`), `DoDraw`
(`Main.cs:61707-62507`), `DoDraw_WallsTilesNPCs` (`63265-63308`), the off-screen target
machinery `RenderToTargets`/`DrawBlack`/`DrawWalls` (`57277-58046`), `InitTargets`/
`ReleaseTargets` (`58160-58230`), the background/sky family `DrawBG`,
`DrawBackground_*`, `DrawSurfaceBG_*`, `DrawSunAndMoon`, `DrawStarsInBackground`,
`DrawClouds_*` (`52684-61533`, `62848-64274`), plus `Terraria/Lighting.cs`
(`LightTiles`/`GetColor*`/`GetColor9Slice`). The Rust side is
`crates/tr-app/src/app/frame.rs::frame` (1824 lines, the whole draw phase),
`crates/tr-app/src/render/{sky,bg,tiles,liquids,entities,foliage,wires,debug}.rs`,
`crates/tr-core/src/{sky,drawdata,lighting}.rs`, `crates/tr-render/src/{post,water_mask,
sprite}.rs` + `post.wgsl`/`water_mask.wgsl`. Responsibility: composite sky, background
layers, walls, tiles, liquids, entities, particles, water and UI into one frame with
the correct order, zoom and lighting — the single largest "does the frame look right"
surface.

## Key entities

- `Terraria/Main.cs::Draw` (61644) — wrapper: `EnsureRenderTargetContent`, `DoDraw`, `OnPostDraw`, `TransferCompletedAssets`.
- `Terraria/Main.cs::DoDraw` (61707) — the phase list (see Deviations for the enumerated order).
- `Terraria/Main.cs::DoDraw_WallsTilesNPCs` (63265) — walls/blacks → WoF/back-gore → tiles non-solid → waterfalls → NPCs behind tiles → tiles solid → players behind NPCs → NPCs over tiles.
- `Terraria/Main.cs::DoDraw_Tiles_Solid` / `_NonSolid` (63322 / 63358) — draw `tileTarget`/`tile2Target` when off-screen mode is on (`Main.cs:63333`, `63367`), else `DrawTiles` direct.
- `Terraria/Main.cs::InitTargets` (58160) — creates the eight targets (`58187-58196`).
- `Terraria/Main.cs::RenderToTargets` (57277) — the layer cache: water/back-water/background(+swap)/wall/tile/tile2, partial-offscreen re-render (`57299-57325`), `ApplyPendingBackgroundTargetSwap` (`58047`).
- `Terraria/Lighting.cs::LightTiles` (100), `GetColor(x, y, Color)` (198), `GetColor9Slice` (244/274), `GetCornerColors` (287) — the light multiply consumed by every sprite path.
- `Terraria/Main.cs::DoDraw_UpdateCameraPosition` (62590), `DrawWaters` (57465), `DrawBackgroundBlackFill` (61521), `DrawLensFlare` (62509).
- `crates/tr-app/src/app/frame.rs::frame` (82) — the Rust phase function (sky → tiles → particles/foliage → wires → player → minimap → HUD → post → map UI → shot).
- `crates/tr-app/src/render/sky.rs::draw_sky` (581) + `color_of_the_skies` (52), `make_clouds` (270), `make_stars` (469), `atmosphere_factor` (154).
- `crates/tr-app/src/render/bg.rs::draw_background` (353), `draw_bg_layer` (1174), `draw_front_trees` (1112), `AmbiencePass` (1097), `cave_parallax` (19).
- `crates/tr-app/src/render/tiles.rs::draw_tiles` (256) — walls, tiles, tile light/seeds, waterfall quads; `style.shade(l, tx, ty, anim)` (743/790/842) folds light into vertex tint; wall 4-corner + bilerp (1075-1121).
- `crates/tr-app/src/render/liquids.rs::LiquidPass` (12), `draw_waters` (31), `draw_liquids` (85), `draw_shimmer` (199), `water_mask_rects` (393).
- `crates/tr-core/src/lighting/mod.rs::LightMap::compute_into_with_lights`-family (286-300) + `lighting/{params,blur}.rs`.
- `crates/tr-core/src/drawdata.rs` (2742) — tile/wall/plant DrawData port (`TileDrawData` construction, `dd` helper at 1380).
- `crates/tr-render/src/post.rs::pass_index` (182) — the `Overlays.Scene`/`Filters.Scene` pass table (Sepia, Noir, BloodMoon, Graveyard, Sandstorm/Blizzard, Heat/WaterDistortion…).

## Dependencies

- Calls into: `tr-render` (sprite batch/pipeline, `post.rs` filters, `water_mask.rs`),
  `tr-core` (lighting, sky, drawdata, liquid_draw, scene metrics, waterfall), assets/atlas
  lookups, `world_sim` (camera, weather, liquids) — the world range is cached on a
  `WorldFrameKey` that includes `zoom` (`frame.rs:953`).
- Called by: `crates/tr-app/src/main.rs` → `app::run` → `frame::frame`; screenshot/bench
  paths (`capture_frame`, `finish_bench`).
- Shared data structures: `LightMap`/`LightParams`/`LightStyle`, `TileDrawData`,
  `LiquidDraw`, `SceneMetrics`/`scene_state`, `WorldFrameKey`, the one `SpriteRenderer`
  instance (single instanced pipeline, tint pre-linearised in WGSL).

## Port status

| C# subsystem | Status | Rust location | Evidence / notes |
| --- | --- | --- | --- |
| `Draw`/`DoDraw` phase skeleton | PARTIAL | `tr-app/src/app/frame.rs::frame` | Phases exist but as one function with no per-phase `SpriteBatch.Begin/End`, no `TimeLogger`, no `_isDrawingOrUpdating` guard. |
| Off-screen layer targets (tile/tile2/wall/black/background/backWater/water/sky) | PARTIAL | `tr-render/src/post.rs` (main/swap views, 606-609), world range cache | `InitTargets` (`Main.cs:58187-58196`) is *two* target families: world-space `WorldSceneLayerTarget` caches and full-screen `screenTarget`/`screenTargetSwap`/`skyTarget`. The Rust port caches one world range + a screen swap ping-pong in `render_world_fx` (`frame.rs:1644`); no `wallTarget`, no `skyTarget`, no per-layer partial-offscreen re-render (`Main.cs:57299-57325`). |
| Sky: stars, sun/moon, clouds, horizon, atmosphere | PARTIAL | `render/sky.rs::draw_sky` (581), `make_stars` (469), `make_clouds` (270), `tr-core/src/sky.rs` | Stars/sun/moon/clouds/horizon-visibility are ported (`sky.rs` tests: `sun_peaks_centred_at_noon`, `bg_top_y_tracks_the_camera`, `colours_are_white_at_midday_and_blue_at_midnight`). `HorizonRenderer.DrawHorizon`/`DrawLensFlare` (`Main.cs:62080`, `62509`, `62320`) has no counterpart in `render/` — lens flare MISSING. |
| Background layers (surface biomes, dirt/rock/magma/underworld, black box) | PARTIAL | `render/bg.rs::draw_background` (353), `draw_bg_layer` (1174), `AmbiencePass` (1097), `draw_front_trees` (1112) | Biome surface passes are folded into `draw_sky`/`draw_bg_layer`; `DrawBackground_*` per-layer functions of `Main.cs:52684-53800` are not 1:1 named; cross-fade/transition (`DrawBG_HandleBackgroundTransition` `64065`, `DrawBG_ModifyBGFarBackLayerAlpha` `64258`) is approximated by `AmbiencePass` + `bg_alpha_front/far` — needs pixel A/B. |
| Tiles solid + non-solid order, tile entities | PARTIAL | `render/tiles.rs::draw_tiles` (256) | `draw_tiles` is called once per world-range rebuild (`frame.rs:1074`), covering solid and non-solid layers; the vanilla split across two targets (`Main.cs:63322`/`63358`) collapses into one cached pass — order *within* the batch is what matters, and it is not yet verified against `DoDraw_WallsTilesNPCs`. |
| Walls + blacks | PARTIAL | `render/tiles.rs` wall path (1075-1121) | `WallsRenderer`/`DrawBlack` (`Main.cs:58032`/`57941`) are one merged wall+black path; `wallTarget` cache absent. |
| Lighting compositing (LightMap multiply) | PARTIAL | `tr-core/src/lighting/*`, `render/tiles.rs` `style.shade` | `Lighting.GetColor`/`GetColor9Slice` (`Lighting.cs:189-274`) multiply sprite colours per cell; Rust folds the same light into vertex tints on the CPU (`tiles.rs:743/790/842`, wall bilerp 1104-1121) with `LightMap::compute_into_with_lights` + blur. Numerically equivalent family (`nine_slice_averages_with_neighbours`, `light_engine_seam_matches_the_map`, `parallel_light_is_bit_identical_to_sequential`) but the multiply happens at build time, not per draw — no `UpdateEveryFrame`/`drawToScreen` toggle equivalent of `Main.cs:61851-61862`. |
| Liquids: back water, front water, shimmer, waterfalls | PARTIAL | `render/liquids.rs::draw_waters`/`draw_liquids`/`draw_shimmer` (31/85/199), `tr-render/water_mask.rs` | Back-water/front-water split (`Main.cs:62196` vs `62298-62308`) collapses into one liquid pass plus a mask/distortion pass; waterfalls run before the light map (`frame.rs:833-876`, mirroring `Main.cs:63281` + one-frame-late `AddLight`). |
| Entity draw order (NPCs/players/projectiles/items/gore/dust/rain/particles) | PARTIAL | `frame.rs` (1074-1196), `render/entities.rs::draw_player`/`draw_npcs` | Only a subset of the C# cache buckets exist: `DrawCacheNPCsMoonMoon`, `BehindNonSolidTiles`, `ProjsBehindNPCs`, `NPCProjectiles`, `ProjsOverPlayers/OverWiresUI` (`Main.cs:63271-63413`) have no named equivalents; particles are partially there (`rain`, `sand`, `minecart_sparks` at `frame.rs:1169-1177`) but `ParticleSystem_World_BehindPlayers/OverPlayers` (`Main.cs:62252-62261`) is MISSING. |
| Screen effects (graveyard/noir/blood-moon/storm, heat & water distortion) | PARTIAL | `tr-render/src/post.rs::pass_index` (182), `post.wgsl` | Pass table mirrors vanilla `Filters.Scene` (`post.wgsl:1-9` comments claim 1:1 for the tint/noise subset; distortion passes are explicitly "not ported" for the compiled `FilterHeatDistortion`/`FilterWaterDistortion` HLSL). |
| Overlays / full-screen darkening (`ScreenDarkness`, `ScreenObstruction`, `MoonlordDeathDrama`, `DrawPaladinsShield`, `DrawInfernoRings`, `DrawLensFlare`) | MISSING | — | No symbols in `tr-app/src/render/*` or the `post.rs` table (search found only `Sepia`/`Noir`/… filters). `Main.cs:62316-62326` is unimplemented. |
| Capture mode (`CaptureManager`, `CaptureSettings`, `DrawCapture`, `Filters.Scene.CanCapture/EndCapture`) | MISSING | `render/debug.rs::capture_frame` (274) | The port only has a file-screenshot hook; `CaptureManager.Instance.DrawTick/Update/Draw` (`Main.cs:56673` capture, `61729`) has no equivalent. |
| Zoom & view transforms (1.0–2.0) | PARTIAL | `frame.rs` (817, 953, 1046-1196) | `draw_cam = (camera * zoom).round() / zoom` (`frame.rs:817`) pins pixel snapping at any zoom; `zoom` is in the `WorldFrameKey` (`:953`). The C# value chain `ForcedMinimumZoom` × `clamp(GameZoomTarget,1,2)` with `SupportWideScreen` override (`Main.cs:61990-61999`) was **not located** in `world_sim/camera.rs` or `settings_ui` — range clamp unverified. |
| `screen→world` math for UI/cursor | PARTIAL | `render/entities.rs::snap_to_screen_pub` (992), `draw_cam` | World-anchored snapping exists; `DoDraw_UpdateCameraPosition` (`Main.cs:62590`) is split between `world_sim/camera.rs` and `frame.rs` and was not compared line-by-line here (module 01 territory). |
| Fullscreen map / minimap layers | PARTIAL | `ui::minimap::*`, `app/map.rs`, `frame.rs:1243-1256, 1547-1560` | Owned by module 14; noted only for phase placement (`Main.cs:62147-62182`). |

## Tests

`grep "#\[test\]"` over the surveyed files returns **152 matches, all in `tr-core`**;
`tr-app/src/render/*`, `tr-render/src/{post,water_mask}.rs` and `app/frame.rs` contain
**no unit tests at all** — the draw phase order is only guarded by the deterministic
screenshot contract (`TR_SHOT` + `tools/pixel_diff.py`, SHA256-equality of two runs),
which this module did not execute.

- `crates/tr-core/src/sky.rs::tests` — `sun_peaks_centred_at_noon`, `sun_rises_at_left_edge_offscreen`, `moon_uses_the_night_period`, `bg_top_y_tracks_the_camera`, `colours_are_white_at_midday_and_blue_at_midnight`, `atmo_is_one_at_the_surface`, `atmo_fades_with_depth_and_clamps`.
- `crates/tr-core/src/lighting/mod.rs::tests` — `open_sky_is_lit`, `solid_ground_shadows_below`, `torch_lights_the_room`, `parallel_light_is_bit_identical_to_sequential`, `lava_glows_and_hell_is_warm`, `invisible_block_is_transparent_when_hidden`, `hidden_invisible_block_still_receives_sun`, `hidden_invisible_wall_admits_sun`, `nine_slice_averages_with_neighbours`, `panning_within_the_margin_reuses_the_cache`, `light_engine_seam_matches_the_map`.
- `crates/tr-core/src/lighting/params.rs::tests::light_styles_transform_samples`.
- `crates/tr-core/src/drawdata.rs::tests` — `test_DW_1_4_conditional_heights`, `test_DW_1_4_stride_2016`, `test_DW_1_4_banner_parity`, `test_DW_1_4_frame_overrides`, `test_DW_1_2_campfire_epilogue`, `test_DW_1_4_tile_frame_passthrough`, `test_DW_1_1_group_lists_complete`, `test_DW_1_1_starred_static_skeletons`, `test_DW_1_1_static_switch_sweep`, `test_DW_1_1_deviation_kinds_stay_default`, `i16_domain_extremes_do_not_panic`.

Gap: nothing asserts the *order* sky → background → walls → tiles ≥ NPCs ≥ players ≥
entities ≥ water → UI, nor the zoom clamp, nor filter ordering vs `EndCapture`.

## Deviations & risks

- **No off-screen target family (behavioural, pixel)** — vanilla renders walls/tiles/
  tiles2/background/backWater/water into cached `WorldSceneLayerTarget`s and composites
  them with parallax offsets (`Main.cs:63333`, `62213`, `62196`, `62308`); the port has
  one world range + immediate liquids. Any camera/parallax offset that depends on
  `Position - screenPosition + caveParallax vector` must be inside `draw_tiles` — likely
  divergence at zoom ≠ 1.0 and when the background target is only *partially* offscreen
  (`Main.rs:57311-57325` double-buffer swap `backgroundTargetSwap`).
- **`skyTarget` + `Filters.Scene.CanCapture` gating absent (pixel)** — vanilla clears and
  renders `skyTarget` only when a scene filter can capture (`Main.cs:62021-62027`,
  `62329-62332`). In the port, sky is always drawn into the main range, so filters must
  read the same texture — `render_world_fx` (`frame.rs:1644`) handles it, but there is no
  capture-correct skip path; screenshots with filters active are the only check.
- **Phase order not 1:1 verified (pixel, medium-high)** — the C# order interleaves
  projectiles/players/particles/NPC buckets in a specific sequence
  (`Main.cs:62249-62326`); the Rust order is `draw_tiles` → rain/sand/sparks → leaves →
  wires → player. No test or doc asserts equality; the ROADMAP row for entity ordering
  cannot be marked DONE on this evidence.
- **Missing full-screen effects (cosmetic, user-visible)** — graveyard/noir are ported as
  filters, but `ScreenDarkness`, `ScreenObstruction`, `MoonlordDeathDrama`,
  `PaladinsShield`, `InfernoRings`, lens flare and `DrawLensFlare` (`Main.cs:62316-62326`)
  are absent → blood-moon/hell/eclipse darkness and boss drama will simply not appear.
- **Capture mode missing (behavioural)** — no `CaptureManager`; anything keyed on
  `IsCapturing`/`SceneMetrics` suppression (`Main.cs:62518`) cannot work.
- **Zoom clamp unverified (behavioural)** — policy requires 1.0–2.0 and
  `Main.ForcedMinimumZoom` semantics (`Main.cs:61990-61999`); only `self.zoom` consumers
  were found, no clamp site. Must be located before claiming M2-DRAW zoom DONE.
- **`drawToScreen`/`UpdateEveryFrame` branch missing (behavioural, minor)** — vanilla can
  bypass the target cache entirely (`Main.cs:61851-61862`, `63326`); the port always uses
  the cache. Behaviour-equivalent only while light mode is static.
- **`DrawData`-driven tile overlay gaps (pixel)** — `drawdata.rs` covers DrawData tables,
  but hit-tile/hit-replace animations (`Main.cs:63348-63349`), tile cracks
  (`61533`) and `DrawTileInWater` (`57536`) are not represented in the surveyed symbols.

## Task seeds

- **S** — Write `docs/research` companion note + test asserting the `frame.rs` phase order
  (a golden phase-name sequence vs `Main.cs:62229-62326`) so future refactors cannot
  reorder silently. Depends on nothing.
- **S** — Locate and cite the Rust zoom clamp equivalent of `Main.cs:61990-61999`
  (`ForcedMinimumZoom` × `clamp(1,2)`), or add it if absent. Blocks the M2-DRAW zoom row.
- **M** — Port the full-screen effect set: `ScreenDarkness.DrawBack/Front`,
  `ScreenObstruction.Draw`, `DrawPaladinsShield`, `DrawInfernoRings`, lens flare
  (`Main.cs:62316-62326`, `62509`) as post-pass or batch overlays, with `pass_index`
  entries. Depends on zoom clamp S.
- **M** — Capture mode: a minimal `CaptureManager` (selection rect, `IsCapturing`
  suppression, scene-metrics skip `Main.cs:62518`, write-out) reusing
  `render/debug.rs::capture_frame`. Independent.
- **L** — Re-introduce a layer-target equivalent (wall/black/background/backWater/water
  caches with partial-offscreen refresh, `Main.rs:57277-58046`) *only if* A/B screenshots
  at zoom 1.0/1.5/2.0 show divergence; otherwise document the single-range equivalence in
  `docs/ANALYSIS.md`. Depends on the phase-order golden test (S) and zoom clamp (S).
- **M** — Particle-system parity: `ParticleSystem_World_BehindPlayers/OverPlayers`
  (`Main.cs:62252-62261`) and the C# draw-cache buckets (`63271-63413`) → named buckets
  in `frame.rs`. Depends on the phase-order golden test.
</content>
