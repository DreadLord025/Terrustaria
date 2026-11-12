# 41 — Render pipeline vs the C# graphics layer

> Wave 4 · researched 2026-11-12 · sources: decompiled (Main.cs, TileDrawing.cs, Graphics/TileBatch.cs, Graphics/Camera.cs) + decompiled-libs (ReLogic.Content, ReLogic.Graphics) + crates (tr-render, tr-app/src/render, tr-app/src/app/frame.rs, tr-content) + ROADMAP/ANALYSIS/PORT_MAP/AGENTS.md
>
> Module 02 owns the **phase order**; this module owns the **graphics layer beneath it**:
> the draw-call structure, the texture-atlas strategy, the blend/scissor/sampler state,
> the WGSL shaders and the render-target format & alpha semantics.

## Scope

C# side: the XNA/MonoGame-equivalent sprite stack — `Terraria/Main.cs` (67 317 lines;
`Main.spriteBatch` is used in 121 `Begin`/`End` pairs and 824 `Draw` calls),
`Terraria/GameContent/Drawing/TileDrawing.cs` (9 656 lines; 2 `Begin`, 3 `End`,
13 `Draw`), `Terraria/Graphics/TileBatch.cs` (681 lines — Terraria's *own* batcher,
not `SpriteBatch`), `Terraria/Graphics/Camera.cs` (the `GameViewMatrix`/screen-position
view), plus the ReLogic content pipeline in `decompiled-libs/ReLogic/ReLogic.Content/*`
(`AssetRepository`, `Readers/XnbReader.cs`, `Readers/PngReader.cs`,
`Sources/XnaContentSource.cs`) and `ReLogic.Graphics` (`DynamicSpriteFont.cs`,
`DynamicSpriteFontReader.cs`, `FontPage.cs`).

Rust side: `crates/tr-render/src/{sprite.rs (1218), post.rs (1082), atlas.rs (315),
water_mask.rs (356), gpu.rs (158), font_atlas.rs}` + the three WGSL files
`sprite.wgsl (232)`, `post.wgsl (207)`, `water_mask.wgsl (42)`; the app side
`crates/tr-app/src/render/{mod,tiles,liquids,entities,bg,sky,foliage,wires,debug,
player_layers}.rs` and `crates/tr-app/src/app/frame.rs`; the content side
`crates/tr-content/src/{xnb,lzx,dxt,texture,font,effect}.rs`.

Responsibility: turn the world's draw data into pixels — one atlas, one instanced
sprite pipeline, one post-filter chain — with the *same* blend, sampling, scissor,
format and alpha behaviour as the vanilla XNA stack, so the frame is pixel-identical.

## Key entities

- `Terraria/Main.cs::Main.spriteBatch` — the vanilla `SpriteBatch`. 121 `Begin` calls
  across `Main.cs` with **six different blend/sampler/rasterizer combinations**:
  - `Main.cs:22193` — `Begin(Deferred, BlendState.AlphaBlend, DefaultSamplerState,
    DepthStencilState.None, Main.Rasterizer, null, Main.Transform)` — the world pass.
  - `Main.cs:27473` — a custom "multiply" `BlendState` built at `Main.cs:27462-27471`:
    `ReverseSubtract`, `ColorSourceBlend = SourceAlpha`, `ColorDestinationBlend = One`.
  - `Main.cs:27490` — `BlendState.Additive` (Lightsbane projectile glow).
  - `Main.cs:40407` / `Main.cs:46458` / `Main.cs:43183` — UI batches with
    `Main.ScissorState` / `Main.GameViewMatrix.ZoomMatrix` / `SamplerState.LinearClamp`.
  - `Main.cs:23696`, `Main.cs:24242`, `Main.cs:38835` — `SpriteSortMode.Immediate`
    and `SamplerState.PointWrap` variants.
- `Terraria/Main.cs::Main.Transform` (2743) — `GameViewMatrix.TransformationMatrix`;
  `Main.Rasterizer` (485) is swapped between `CullCounterClockwise` (56690) and
  `CullClockwise` (62006); `Main.ScissorState` (2329) is the scissor-test rasterizer;
  `Main.DefaultSamplerState` (2780-2782) → `PointClamp` or `LinearClamp`.
- `Terraria/Graphics/TileBatch.cs` — Terraria's **own** batcher for tiles/walls:
  `_spriteTextures[2048]` (103), `_passTextures[512]` (127), `_textureIdLookup` (131),
  `SwitchBatch(texture, layerStack)` (485), `CreateBatch` (510), `GetNextSpriteIndex`
  (421). Batches are keyed by **(Texture, LayerStack)** — i.e. *per-texture* batches,
  never an atlas.
- `Terraria/GameContent/Drawing/TileDrawing.cs::DrawSingleTile` (864) — the per-tile
  draw; the flames/glows at `TileDrawing.cs:8626-8702` are the only `spriteBatch.Draw`
  calls in that file (sun orb at 7911).
- `Terraria/Graphics/Camera.cs` — `UnscaledPosition => Main.screenPosition` (8),
  `ScaledPosition` (12) = position + `GameViewMatrix.Translation`; `Rasterizer`
  (42) → `Main.Rasterizer`.
- `crates/tr-render/src/sprite.rs::SpriteRenderer` — the port's whole sprite stack:
  `Globals` (12-23: screen/cam/zoom/time/aberration/texel), `SpriteInstance`
  (25-47: pos/size/uv/color/rot/paint/dye/layer), `RING = 3` instance buffers (123),
  `SpriteRenderer::new` (129) — one atlas texture (131-150, `Rgba8UnormSrgb`), one
  nearest/clamp sampler (179-187), **one** bind-group layout + **one** pipeline
  (294-352), `draw_ranges` (1034) — the frame's draw loop.
- `crates/tr-render/src/sprite.rs::draw_ranges` (1034-1244) — writes the six ranges
  (`back`, `world`, `fx`, `map_back`, `map`, `ui`) into one instance buffer and issues
  **one `pass.draw` per non-empty range** (1195-1239).
- `crates/tr-render/src/post.rs::PostPipeline` (255) + `PASS_TABLE` (136-175) +
  `pass_index` (182) + `PassId` (103-127) — the offscreen scene target + the
  `ScreenShader.fx` pass table.
- `crates/tr-render/src/atlas.rs::Atlas` (56) + `SheetLayout` (13-33) — the shelf
  packer: `Tile = (18 px stride, 0 inset)`, `Wall = (36 px stride, 8 px inset)`,
  `Raw`; `PAD = 2` (9); `pick_plain_frame` (196).
- `crates/tr-render/src/water_mask.rs::WaterMask` (35) + `water_mask.wgsl` — the R8
  coverage target for the WaterDistortion pass.
- `crates/tr-render/src/gpu.rs::Gpu::new` (22) — the surface: **sRGB format preferred**
  (87-92), `alpha_mode = caps.alpha_modes[0]` (105), `offscreen_target` (145).
- `crates/tr-app/src/app/frame.rs::frame` — the only caller of `SpriteRenderer::render`
  (1702), `render_world_fx` (1644), `render_map_ui` (1697) and `WaterMask::render` (1667).

## Dependencies

- Calls into: `tr-render` (sprite pipeline, atlas, post chain, water mask),
  `tr-content` (XNB/LZX/DXT decode, font pages, effect bytes), `tr-core`
  (`LightMap`, `TileDrawData`, `sky`, `liquid_draw`, `waterfall`), `tr-app/src/assets.rs`
  (`Lookups`, atlas construction, `Atlas::build(images, 8192)` at `assets.rs:2639`),
  `world_sim` (camera, zoom, weather).
- Called by: `crates/tr-app/src/main.rs` → `app::run` → `app/loop.rs` (creates the
  `PostPipeline` at `loop.rs:113`) → `app/frame.rs::frame`; the screenshot/bench paths
  (`capture_frame`, `finish_bench` → `BENCH draws=…` at `frame.rs:1773-1779`).
- Shared data structures: `Atlas`/`SheetRect`/`BgTex` (`render/mod.rs:18-24`),
  `SpriteInstance`, `Globals`, `FilterParams`/`PassId`, `MaskRect`, `TileDrawData`,
  `LightMap`/`LightParams`, `WorldFrameKey` (module 02's G6 cache).

## Port status

| C# subsystem | Status | Rust location | Evidence / notes |
| --- | --- | --- | --- |
| One instanced sprite pipeline over one atlas | PORTED | `tr-render/src/sprite.rs::SpriteRenderer` (129-425), `atlas.rs::Atlas` (64) | 262 144-instance ring, 3 buffers, one pipeline, one bind group; the atlas is shelf-packed at 8192 px wide (`assets.rs:2639`). AGENTS.md's "one atlas / one instanced pipeline" invariant holds. |
| Vanilla is **per-texture**, not an atlas | PORTED (divergent by design) | `sprite.rs::draw_ranges` (1034) vs `TileBatch.cs::SwitchBatch` (485) / `Main.spriteBatch.Draw` × 824 | Verified: the vanilla draws per texture (`TileBatch` batches by `Texture` + `LayerStack`, `TileBatch.cs:485-536`), no atlas exists in C#. The port merges every sheet into one 8192-wide atlas — a deliberate architectural divergence, pixel-equivalent only if sampling is texel-exact (see Deviations D-3, D-4). |
| Draw-call structure | PARTIAL | `sprite.rs::draw_ranges` (1195-1239) | Vanilla: 121 `Begin`/`End` pairs + one `SpriteBatch` flush **per texture switch** (hundreds of draw calls/frame) plus the `TileBatch`'s per-texture layer batches. Port: **one `pass.draw` per range** → up to **6** per frame (back, world, fx, map_back, map, ui); with `TR_POST=1` the split is 3 + 3 plus 1..`MAX_CHAIN`(4) filter passes. `ROADMAP.md:64`'s "2 draw calls" is **wrong** (see D-1). |
| Blend state: world `BlendState.AlphaBlend` | PORTED (documented approximation) | `sprite.rs:327-338`, `sprite.wgsl:232-242` | The pipeline is a single `(One, OneMinusSrcAlpha)` premultiplied-source blend; the fragment deliberately does **not** multiply rgb by alpha (`sprite.wgsl:236-239`), because the atlas is assumed premultiplied (the comment claims the content pipeline premultiplies and that `rgb <= a` was verified over the asset set). XNA's own `BlendState.AlphaBlend` is *not* in this repo's decompile (only ReLogic is) → the convention is asserted, not verifiable here (D-2). |
| Blend state: `BlendState.Additive` (`Main.cs:27490`) | MISSING | — | The port has exactly one blend state (`sprite.rs:327`). Lightsbane's additive magenta glow (`Main.cs:27491-27492`) and any other additive batch cannot be expressed. |
| Blend state: custom "multiply"/ReverseSubtract (`Main.cs:27462-27473`) | MISSING | — | Same single-blend limitation; the vanilla's `ReverseSubtract` multiply pass has no equivalent. |
| Blend state: `null` blend (`Main.cs:40415`, `43156`, `46458`) | PORTED (implied) | `sprite.rs:327` | `SpriteBatch.Begin(..., blendState: null, ...)` resolves to the default alpha blend in XNA/MonoGame; the port's single blend covers it. |
| Depth / rasterizer state | PORTED (by omission) | `sprite.rs:345-349`, `water_mask.rs:102-107` | Vanilla: `DepthStencilState.None` for the world, `DepthStencilState.Default` for the immediate batches (`Main.cs:23696`, `38835`), `RasterizerState.CullNone` in several places, `Main.Rasterizer` = `CullCounterClockwise`/`CullClockwise` elsewhere. Port: `depth_stencil: None`, `cull_mode: None` everywhere — draw order is the only ordering, which matches the vanilla's `CullNone` majority but silently accepts the two `CullCounterClockwise` batches (`Main.cs:57948`, `58037`) whose winding the port does not reproduce (D-6). |
| Sampler state: `PointClamp` | PORTED | `sprite.rs:179-187` | One nearest + clamp-to-edge sampler for the whole atlas; matches `Main.DefaultSamplerState = PointClamp` (`Main.cs:2780`). |
| Sampler state: `LinearClamp` (`Main.cs:45025`, `48204`, `55613`) | MISSING | — | The vanilla's UI/logo paths that sample linearly (and `Main.SamplerStateForCursor`, `Main.cs:45257`) cannot be reproduced; the port has one sampler for every draw. |
| Scissor rects | MISSING | — | The vanilla saves/restores `GraphicsDevice.ScissorRectangle` around scissored batches (`Main.cs:25403-25424`, `Main.cs:40405-40414`, `Main.ScissorState` at `Main.cs:2329`). The port has **no scissor anywhere** — `grep set_scissor` over `crates/` returns nothing, and `tr-app/src/ui/char_creation.rs:1859` admits it: "the port has no scissor, so they are skipped" (`UIList.OverflowHidden`). |
| Screen shake / camera offset | PARTIAL | `tr-app/src/settings.rs:180,291` (flag only) | Vanilla: `Main.UseScreenShake` (435, default true at 66575) plus the `MoonLordShake` **screen filter** (`Main.cs:64976-64984`, `Main.cs:52194`). The port keeps the option flag but the filter is inert: `PASS_TABLE` maps `"MoonLordShake" → FilterMoonLordShake` with pass index **0** = passthrough (`post.rs:147`). No camera-offset shake exists in vanilla either (the only `shake` hits are `UseScreenShake`, `MoonLordShake`, tree shakes, `punchCameraModifier.IsAScreenShake`), so the camera model matches; the *filter* is missing (D-7). |
| View transform / zoom | PARTIAL | `sprite.wgsl:85-89`, `frame.rs` (draw_cam) | Vanilla: `Main.Transform = GameViewMatrix.TransformationMatrix` (2743) — a CPU-side matrix; the zoom is `GameZoomTarget` clamped 1..2 (`Main.cs:16808`, `16812`) with `ForcedMinimumZoom` (243). Port: the transform is computed in the vertex shader from `Globals.cam`/`zoom`; the zoom clamp exists in the settings loader (`settings.rs:446`) but the *runtime* `GameZoomTarget` step and `ForcedMinimumZoom` have no located equivalent (module 01/02 territory). The port additionally snaps the camera to the zoom grid (`frame.rs:817` per module 02) — vanilla applies a raw scale matrix (D-8). |
| Render-target format & alpha semantics | PORTED | `gpu.rs:87-92,94-112,145-176`, `post.rs:512` | The surface picks the first **sRGB** format; the offscreen scene target reuses `gpu.config.format` (`gpu.rs:145`) so the scene pass sRGB-encodes on store and the filter pass round-trips byte-exact (`post.rs:20-25`). Atlas and map textures are `Rgba8UnormSrgb` (`sprite.rs:143`, `541`); the font pages are BC2/DXT3 (`sprite.rs:443`) — the retail `SurfaceFormat.Dxt3`. Alpha: `alpha_mode = caps.alpha_modes[0]` (opaque on desktop) and the atlas keeps the authored alpha; the water mask is `R8Unorm` (`water_mask.rs:20`). |
| Tiles 16 px on an 18 px sheet grid | PORTED | `atlas.rs::SheetLayout::Tile` (13-33) | `geometry() == (18, 0)` — matches AGENTS.md and `ANALYSIS.md:1416`. |
| Walls 36 px cells with an 8 px inset | PORTED | `atlas.rs::SheetLayout::Wall` (13-33) | `geometry() == (36, 8)`; `ANALYSIS.md:1417` documents the vanilla's 32×32 source rect offset by −8 landing the inner 16×16 on the tile. |
| Tree trunks 20×20 on a 22 px grid | PORTED | `tr-core/src/tree.rs:15,17` (`TRUNK_GRID = 22`, `TRUNK_SIZE = 20.0`), `assets.rs:2511-2512` | Matches AGENTS.md; trees bypass the 16×16 block-frame table (`assets.rs::rect_uv`). |
| WGSL tint pre-linearisation | PORTED (approximate) | `sprite.wgsl:231` — `pow(max(in.color.rgb, 0.0), vec3(2.2))` | The intent (XNA multiplies bytes in sRGB space; the atlas is sRGB-decoded and the swapchain re-encodes, so the tint must be linearised) is right, but `pow(x, 2.2)` is **not** the sRGB EOTF and the tint's alpha is correctly left un-linearised (`sprite.wgsl:239`). See D-5. |
| Paint recolour | PARTIAL | `sprite.wgsl:113-170` | 32-entry `paint_rgb` table + `paint_recolor`; the vanilla's compiled HLSL passes are not in the decompile — the port washes the texel with the hue and lets the luminance carry detail (`ANALYSIS.md:673-684`). The wash runs on **linear** samples (the atlas is sRGB-decoded), the vanilla multiplies sRGB bytes (D-5). |
| Armor-dye passes | PARTIAL | `sprite.wgsl:181-195`, `sprite.rs:54-63,722` | Two reconstructed modes (colour wash, gradient) over a 48-slot uniform table; the vanilla's `.fx` bytecode is not in the decompile (documented in `sprite.wgsl:172-180`). |
| Post-filter chain | PARTIAL | `post.rs` + `post.wgsl` | 9 of the 29 `Filters.Scene` rows are wired (Graveyard, Noir, BloodMoon, Sandstorm ×2, Blizzard ×2, Heat, Water); the other 20 resolve to the passthrough (`post.rs:136-175`). The distortion amplitudes are visual approximations (`post.wgsl:134-137`). |
| ReLogic content pipeline | PORTED (bypassed) | `tr-content/src/{xnb,lzx,dxt,texture,font,effect}.rs` | The vanilla `ReLogic.Content.Readers.XnbReader` defers to XNA's `ContentManager` (`ANALYSIS.md:1403-1405`); `PngReader` is a debug-only path. The port re-implements the container + LZX + DXT1/3/5 itself (`texture.rs:99-106`), which is why the atlas pixels are identical (`ANALYSIS.md:1460-1468`). The `ReLogic.Graphics.DynamicSpriteFont` reader is ported as the BC2 font array (`sprite.rs:430-478`). |

## Tests

`tr-render` has 22 unit tests, `tr-app/src/render/*` has 68, `tr-app/src/assets.rs`
has the atlas/content tests. **None of them covers the sprite pipeline's blend
equation, the tint pre-linearisation, the draw-call count or the atlas UV phase.**

- `tr-render/src/post.rs::tests` — `test_DW_2_4_pass_table_matches_initializer_rows`
  (679), `test_phase3_pass_indices_match_pass_names` (730),
  `test_DW_2_1_params_neutral_match_screendata_defaults` (759),
  `test_params_layout_is_pod_128_bytes` (780),
  `test_DW_2_2_target_recreated_when_size_changes` (789),
  `test_noise_sampler_address_mode_pot_rule` (801),
  `test_fallback_checkerboard_pattern` (811),
  `test_DW_4_4_passthrough_chains_are_identity` (969),
  `test_DW_4_4_water_pass_without_water_is_identity` (986),
  `test_DW_4_2_water_pass_distorts_only_inside_mask` (1003),
  `test_water_pass_is_deterministic_and_progress_driven` (1031),
  `test_DW_4_1_heat_pass_displaces_pixels` (1054),
  `test_chain_feeds_each_pass_the_previous_result` (1088),
  `test_empty_chain_is_rejected` (1135).
- `tr-render/src/water_mask.rs::tests` — `test_DW_4_2_water_mask_filled_over_liquid_rects`
  (259), `test_water_mask_respects_zoom` (290),
  `test_water_mask_offscreen_and_empty_are_all_zero` (314),
  `test_water_mask_release_resize_and_regrow` (348).
- `tr-render/src/atlas.rs::tests` — `build_ref_matches_build_bit_for_bit` (315)
  (the world-swap packing path must place every sheet where `build` would).
- `tr-app/src/render/*` — `bg.rs` 13, `sky.rs` 9, `tiles.rs` 5, `entities.rs` 5,
  `liquids.rs` 2, `player_layers.rs` 33, `wires.rs` 1 tests (geometry/tint maths).
- `tr-app/src/assets.rs::tests` — `test_DW_4_1_enumeration_sorted_deduped_deterministic`
  (2655), `enumeration_missing_dir_yields_empty_list` (2680),
  `test_DW_4_3_fallback_lookups_survive_without_item_sheets` (2686).
- Pixel-level coverage is only the deterministic screenshot contract
  (`TR_SHOT` + `tools/pixel_diff.py`, SHA256-equal repeats) — not exercised here.

## Deviations & risks

- **D-1 — "2 draw calls" in ROADMAP is wrong (documentation, cosmetic).**
  `ROADMAP.md:64` claims the renderer issues 2 draw calls; `sprite.rs::draw_ranges`
  issues **one `pass.draw` per non-empty range** (1195-1239) → up to 6 per frame, and
  with `TR_POST=1` the frame is 3 + 3 plus 1..4 filter passes (`post.rs::MAX_CHAIN`).
  The `BENCH draws=` line (`frame.rs:1774`) prints the real number. Not admitted in
  ROADMAP/ANALYSIS → fix the row text.
- **D-2 — One blend state for the whole frame (behavioural + pixel, medium).**
  The vanilla uses at least four blend configurations (`AlphaBlend` `Main.cs:22193`,
  `Additive` `Main.cs:27490`, the custom `ReverseSubtract` "multiply"
  `Main.cs:27462-27473`, and `null`→default `Main.cs:40415`). The port has exactly one
  (`sprite.rs:327-338`). Lightsbane's additive glow and the multiply pass cannot be
  reproduced. The port already compensates for one case (tile flames with an alpha-0
  tint, `tiles.rs:1393-1401` pushes alpha 1 instead of 0 and the doc comment states the
  residual "dims the flame slightly against vanilla's full add"). The *premultiplied*
  convention the single blend relies on is asserted in `sprite.wgsl:234-238` but the XNA
  `BlendState.AlphaBlend` definition is **not in this repo's decompile** (only ReLogic
  is), so the claim "premultiplied" cannot be verified from the sources at hand.
- **D-3 — Atlas merges textures that the vanilla keeps separate (pixel, medium).**
  Vanilla batches per texture (`TileBatch.cs:485`); the port repacks every sheet into
  one 8192-wide atlas. Consequences: (a) the draw *order* between different source
  textures changes — vanilla emits all of texture A, then all of texture B (and
  `SpriteBatch` in `Deferred` mode additionally sorts by texture), while the port
  preserves push order — so overlapping translucent sprites from different sheets can
  composite in a different order; (b) the UV phase is normalised, which the project's
  own `ROADMAP.md:1259` (B10) quantifies: a repack "flips 166 background pixels at the
  forest horizon (surface, zoom 2.0)". The half-texel inset (`sprite.wgsl:99-104`)
  removes the 1-px hole artefacts but does not restore vanilla's per-texture phase.
- **D-4 — Nearest sampling across a packed atlas vs per-texture PointClamp (pixel, low).**
  The atlas has `PAD = 2` transparent padding (`atlas.rs:9`) and the shader insets by
  half a texel; vanilla samples the exact texel of its own sheet. Any sprite whose
  source rect abuts a mip/pad boundary can bleed one texel of a neighbour.
- **D-5 — Tint and recolour maths happen in the wrong colour space (pixel, medium).**
  `sprite.wgsl:231` uses `pow(tint, 2.2)` — an approximation of the sRGB EOTF, not the
  piecewise sRGB curve the GPU actually applies to the atlas, so
  `srgb_decode(tex) * pow(tint,2.2)` ≠ `srgb_decode(tex_srgb * tint_srgb)`: the
  composite drifts by a fraction of an LSB to ~1 % per channel. The vanilla multiplies
  **bytes** in sRGB space (`XNA Color * Color` = `(a*b)/255` integer division), which
  truncates — the project already records a 1-LSB divergence for one such case
  (`ANALYSIS.md:424-425`). `paint_recolor` (`sprite.wgsl:158-170`) and `dye_recolor`
  (181-195) additionally wash **linear** samples with sRGB-authored constants, while
  the vanilla's paint/dye HLSL multiplied sRGB bytes (the `.fx` bytecode is not in the
  decompile — `ANALYSIS.md:673-684` admits the paint approximation, but not the
  colour-space shift).
- **D-6 — Rasteriser culling not reproduced (pixel, low).** The vanilla flips
  `Main.Rasterizer` between `CullCounterClockwise` (56690) and `CullClockwise` (62006)
  and uses `RasterizerState.CullNone` for the immediate batches; the port is
  `cull_mode: None` everywhere (`sprite.rs:345`). Winding only matters if a quad is
  built clockwise, which the port's own winding never is — low risk, but the cull
  state is not 1:1.
- **D-7 — `MoonLordShake` filter inert (behavioural, medium).** `Main.cs:64976-64984`
  activates the `MoonLordShake` scene filter during the Moonlord fight;
  `post.rs:147` maps it to pass index 0 (passthrough), so the screen shake never
  happens. The camera itself is unaffected in vanilla either (no camera-offset shake
  exists), so this is purely the missing filter. `Main.UseScreenShake` is ported as a
  settings flag (`settings.rs:180,291`) but gates nothing.
- **D-8 — Zoom model differs from `GameViewMatrix` (pixel, medium).** Vanilla applies
  a raw scale matrix (`Main.Transform` = `GameViewMatrix.TransformationMatrix`,
  `Main.cs:2743`; zoom clamped 1..2 at `Main.cs:16808/16812`); the port snaps the
  camera to the zoom grid (`frame.rs:817`, per module 02) and clamps zoom in the
  settings loader only (`settings.rs:446`). `Main.ForcedMinimumZoom` (`Main.cs:243`)
  has **no located equivalent** — the port therefore renders a wider world view than
  vanilla on wide screens. Module 01/02 own the camera; flagged here because it is a
  *graphics-layer* contract.
- **D-9 — Scissor clipping absent (behavioural + pixel, medium).** Vanilla scissors
  UI lists (`Main.ScissorState`, `Main.cs:2329`; `Main.cs:40405-40414`,
  `25403-25424`). The port has no scissor (`char_creation.rs:1859`); clipped UI
  regions (inventory/scroll lists, `UIList.OverflowHidden`) bleed or are skipped.
- **D-10 — Linear-clamp sampling paths unavailable (pixel, low).** `SamplerState.LinearClamp`
  (`Main.cs:45025`, `48204`, `55613`) and `Main.SamplerStateForCursor` (`Main.cs:45257`)
  have no equivalent; the port's single nearest sampler is used everywhere.
- **D-11 — Order-only batching ignores XNA's `Deferred` sort (pixel, medium).** The
  vanilla's `SpriteSortMode.Deferred` batches group sprites by texture and sort by
  depth within a batch; the port preserves push order into one buffer. For opaque
  content this is invisible; for overlapping translucent sprites (glass tiles,
  plants, liquids, item drops) the composite order can differ.
- **D-12 — The post chain's distortion amplitudes are approximations (pixel, low).**
  `post.wgsl:138` (`HEAT_AMPLITUDE_PX = 1.5`) and `post.wgsl:169`
  (`WATER_AMPLITUDE_PX = 2.0`) are visual guesses; the compiled
  `FilterHeatDistortion`/`FilterWaterDistortion` HLSL is not in the decompile. Admitted
  in `post.wgsl:134-137` and in the M1.4-BG-18 ROADMAP row.
- **D-13 — Twenty of 29 scene filters are inert (behavioural, medium).**
  `post.rs::PASS_TABLE` (136-175) gives pass index 0 to Nebula/Solar/Stardust/Vortex,
  the four Monolith rows, MoonLord(+Shake), testInvert, Sepia, the three Crystal rows,
  Test/Test2/Test3 and CRT. The ROADMAP row admits "towers/monoliths/Aurora/Sepia OUT";
  the MoonLord shake (D-7) is not in that list.
- **D-14 — `alpha_mode` is taken blindly from the adapter (behavioural, low).**
  `gpu.rs:105` takes `caps.alpha_modes[0]`, which may be `PreMultiplied` on some
  platforms; the vanilla's backbuffer is opaque.
- **D-15 — No mip chain (pixel, low, deliberate).** `mip_level_count: 1` everywhere
  (`sprite.rs:140`, `441`, `535`); the vanilla's `Texture2D` assets carry mips
  (`tr-content/src/texture.rs:84-97` reads them but keeps only mip 0). Acceptable
  because the camera zoom is ≤ 2, but a no-LOD guarantee must keep it that way
  (AGENTS.md's optimisation policy).

## Task seeds

- **S** — Fix `ROADMAP.md:64`'s "2 draw calls" to the measured range-draw count and
  cite `sprite.rs::draw_ranges`; add a `TR_STATS` line that prints the per-range draw
  count so the claim stays checkable. Depends on nothing.
- **S** — Add a headless GPU test in `tr-render` that renders one quad with a known
  RGBA8 texture through the sprite pipeline and asserts the exact blended output for
  `a = 0`, `a = 0.5`, `a = 1` tints (locks the D-2/D-5 blend contract). Needs a
  deterministic 4×4 fixture texture; the headless harness already exists
  (`tr-render/src/test_gpu.rs`). Depends on nothing.
- **M** — Verify the premultiplied-alpha claim end-to-end: dump the minimum
  `a - max(r,g,b)` over the decoded atlas (and one DXT3 sheet) and record the result
  in `docs/ANALYSIS.md`; if the atlas is *not* premultiplied, switch the pipeline to
  XNA's `SourceAlpha` source factor or pre-multiply in the shader. Depends on the S
  blend test.
- **M** — Implement a second blend state (an additive and a `ReverseSubtract`
  multiply pipeline) and port `Main.DrawProj_LightsBane` (`Main.cs:27455-27495`)
  onto it; the instance data already carries a per-instance tint, so the port needs a
  per-range or per-instance blend selector (a small pipeline switch in
  `draw_ranges`). Depends on the S blend test (to keep both equations provable).
- **M** — Add scissor support: a `wgpu::RenderPass::set_scissor_rect` driven by a
  per-range scissor rect in `SpriteRenderer`, then port the two vanilla sites
  (`Main.cs:40405-40414` UI, `Main.cs:25403-25424` world) and the
  `UIList.OverflowHidden` skip in `ui/char_creation.rs:1859`. Depends on nothing.
- **M** — Write down the *draw-order contract*: a golden list of the ranges and the
  vanilla buckets they correspond to (module 02 owns the phase order; this owns the
  intra-range order), plus a note in `docs/ANALYSIS.md` on D-11 (Deferred sorting).
  Depends on nothing.
- **L** — Reproduce the vanilla's *per-texture* batching semantics on top of the
  atlas: group the instances by source sheet and flush one draw per sheet (or per
  contiguous group) so the blend order matches `TileBatch`'s per-texture batches and
  XNA's `Deferred` grouping. This is the structural fix for D-3/D-11 and would also
  make the atlas packing irrelevant to ordering. Depends on the S blend test and the
  draw-order contract (M).
- **L** — Colour-space audit of the whole sprite path: replace `pow(tint, 2.2)` with
  the exact sRGB EOTF, decide whether the paint/dye washes belong in sRGB or linear
  space, and compare against the vanilla's byte multiply with a pixel-diff harness
  (a handful of reference tints × the 5 light modes). Depends on the S blend test.
- **M** — Locate or add the `Main.ForcedMinimumZoom` equivalent
  (`Main.cs:243`, `61991`) and the runtime `GameZoomTarget` step
  (`Main.cs:16808/16812`); this is the last half of D-8. Depends on module 01's
  camera work.
