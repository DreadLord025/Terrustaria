# 64d — the d69 capture-compose DELTA (W4-g72 closeout)

**Branch:** `task/W4-g72-bg-layers-3` (closeout). **Mode:** code-only.
**Audience:** `origin/task/W4-d69-layer-target-consumers` (the S1-G-13
capture/consume lane). **Question (the Lead's item 4):** d69's
Background-callback uses our seams; our `bg_style_pick_input` /
`draw_background` signature changed — does the d69 table still match?
**Verdict: the seven `bg_target.rs` seams STILL MATCH (re-verified at d69
tip `79f914fdb`); the `draw_background` CALL SIGNATURE IS BROKEN (10→14
args) — the mechanical delta is below.** d69's own `draw_surface_bg`
extraction is unaffected (it never calls our `draw_sky`).

## 1. The seven bg_target seams — STILL MATCH (no delta)

d69's ANALYSIS "g72b bg_target.rs seam cross-verification" (at our
`1f5c2b1e`) listed seven consumed seams. Re-read against d69's CURRENT
`crates/tr-render/src/targets.rs` (tip `79f914fdb`) — every one unchanged:

| g72b seam (bg_target.rs) | d69 definition (targets.rs) | Match |
| --- | --- | --- |
| `LayerCamera.screen_position/.screen_size/.zoom` | `:120-124` identical fields | ✓ |
| `family.layout.offscreen_range` | `TargetLayout.offscreen_range: i32` (`:40`) | ✓ (caller casts to f32) |
| `layer_composite_position(SceneLayer::Background, position, layer_size, screen_position, camera_center, cave_parallax)` | `:499-506` identical 6-arg order | ✓ |
| `sky_target_active(can_capture, foreground_sunlight_effects, draw_to_screen, map_fullscreen, only_draw_fancy_ui, lighting_not_retro)` | `:525-531` identical 6-bool order | ✓ |
| `SceneLayer::Background` / `BackgroundSwap` | `:309-317` | ✓ |
| `WorldSceneLayerTarget.position()` / `.size()` | `:155` / `:167` | ✓ |
| `super::BgTex` / `super::bg::push_screen` | `render/mod.rs:45`, `render/bg.rs:1413` (push_screen unchanged) | ✓ |

`bg_target.rs` itself is UNCHANGED on our branch since `1f5c2b1e` except the
`ug_crossfade_tint` doc-comment (item 1 added a LIVE note; the signature
`fn ug_crossfade_tint(base: [f32; 4], transition: f32) -> [f32; 4]` is
identical). The semantic assumptions d69 recorded (cavern tree in
`backgroundTarget` at zoom 1, underworld stack in `screenTarget`, the
`!CameraSpaceEffects` re-frame) still hold.

## 2. The BROKEN seam — `draw_background` 10→14 args

d69's two Background-callback call sites use the **merge-base (`a725acb04`)
10-arg** shape:

```rust
// d69 capture_render.rs:393 (CaptureWorld `cw`)
crate::render::bg::draw_background(
    renderer, layer_cam, 1.0, layer_screen, cw.world,
    cw.sky, cw.light, 0.0, cw.anim_time, &mut hell_bottom,
);
// d69 layer_targets.rs:426 (LayerFrame `frame`)
crate::render::bg::draw_background(
    renderer, layer_cam, 1.0, layer_screen, frame.world,
    frame.sky, frame.light, frame.cave_parallax, frame.anim_time,
    &mut hell_bottom,
);
```

Our HEAD (`6cb4e933f`) signature is **14 args** — four appended after
`hell_black_box_bottom` (bg.rs:468-497):

| # | New arg | Type | Source for the capture/layer callback |
| --- | --- | --- | --- |
| 11 | `show_invisible_walls` | `bool` | `crate::env::show_invisible()` (the `TR_SHOW_INVISIBLE` pin; d69's `LayerFrame.show_invisible_blocks` is the tile-side flag — the bg gate reads the same env pin, false in normal play) |
| 12 | `metrics` | `&tr_core::scene::SceneMetrics` | the frame's `App.scene_metrics` (mod.rs:918) — thread `&'a SceneMetrics` into `CaptureWorld`/`LayerFrame` (both already borrow App state) |
| 13 | `ug_back` | `&mut bg_styles::UgBackState` | the App's `ug_back` (mod.rs:1085) — thread `&'a mut UgBackState`; the capture may instead pass a FRESH `UgBackState { style: 0, old_style: 0, transition: 0.0 }` local (the C# `DrawCapture` re-runs `DrawBackground` with the live statics, but a fresh latch only affects the cross-fade overlay, not the base backdrop — documented approximation if chosen) |
| 14 | `shimmer_alpha` | `f32` | `0.0` (the port's shimmer ramp is unwired; the live frame feeds 0.0, draw.rs:1098) |

**The mechanical delta (append four args to each call):**

```rust
// capture_render.rs:393 — add `metrics`/`ug_back` fields to CaptureWorld,
// or pass a fresh local latch (see the table's note).
crate::render::bg::draw_background(
    renderer, layer_cam, 1.0, layer_screen, cw.world,
    cw.sky, cw.light, 0.0, cw.anim_time, &mut hell_bottom,
    crate::env::show_invisible(), cw.metrics, cw.ug_back, 0.0,
);
// layer_targets.rs:426 — add `metrics`/`ug_back` fields to LayerFrame.
crate::render::bg::draw_background(
    renderer, layer_cam, 1.0, layer_screen, frame.world,
    frame.sky, frame.light, frame.cave_parallax, frame.anim_time,
    &mut hell_bottom,
    crate::env::show_invisible(), frame.metrics, frame.ug_back, 0.0,
);
```

`bg_style_pick_input` itself (bg_styles.rs:153) is called INSIDE
`draw_background` — d69 never calls it directly, so its signature change
(the W4-g72-final adapter) is transparent to d69 once the four args above
are threaded. The `UgBackState` struct is `{ style: i32, old_style: i32,
transition: f32 }` (bg_styles.rs:395), `pub(crate)`.

## 3. The `draw_sky` / `draw_surface_bg` note (NOT a d69 break)

Our `draw_sky` (sky.rs:611) ALSO gained the same three tail args
(`metrics`/`ug_back`/`shimmer_alpha`, sky.rs:656-658) — but d69 does NOT
call `draw_sky`: its capture path calls its OWN `draw_surface_bg`
extraction (d69 sky.rs:967, a copy of the surface-chain body that never
threads the underground style latch). That extraction is d69's to keep;
it draws the surface chains only and is unaffected by our `draw_sky`
signature. If d69 later folds `draw_surface_bg` back into a shared
`draw_sky` helper, the three tail args apply there too — flagged, not
broken today.

## 4. Witnesses

Code-only: no new executable witness (this is a cross-lane delta). The
seven-seam match (§1) is a re-read of d69's targets.rs at `79f914fdb`; the
14-arg signature (§2) is our bg.rs:468-497 read at `6cb4e933f`. d69's own
`repeated_compute_into_the_same_rect_is_a_cache_hit` / capture-entity
witnesses are unaffected (they pin the tile/light/entity paths, not the
bg call arity).