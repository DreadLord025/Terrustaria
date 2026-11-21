# UNION-CONTRACT — the tile-draw pass family (g28/g29/g30/g31/g32) + the d56–d60 seam census

**STATUS: EXECUTED + ATTACHED (shadow)** (union-exec + attach phases,
same branch `task/W4-g31-tile-draw-pass-4`, code-only). The §1–§3
keep/drop/rebind prescription was carried out IN-LANE — see §7 for the
execution record. The attach phase censused the d-lane tips against
the union driver shape and wired the union into `render/tiles.rs`
behind `TR_TILEPASS` (default OFF, PLAN-ONLY) — see §8 for the verdict,
the exact seam signatures and the pixel-merge script (§4 made
concrete). The ctor-draw residual is CLOSED (`render/tile_ctor.rs`).

Merger assist for the TILE-PASS 5-WAY UNION. Written on `task/W4-g31-tile-draw-pass-4`
(reconcile phase, code-only — NO code changes to any lane; this document is the
merge prescription). All line/shape claims below were read from the pushed
branch tips, base `a725acb0` for every lane:

| Lane | Tip read | Code file(s) | State |
|---|---|---|---|
| g28 pass 1 | `3f1a40cf` | `render/tile_pass.rs` (721 ln), `tr-render/quality_light.rs` (225 ln), `torch.rs` pub lift | UNWIRED |
| g29 pass 2 | `9bb92d0b` | `render/tile_pass.rs` (1265 ln, SAME PATH as g28) | UNWIRED |
| g30 pass 3 | `cd7f7d31` | `render/special_points.rs` (838 ln), `torch.rs` pub lift | UNWIRED |
| g31 pass 4 | `3af480de` | `render/tile_drains.rs` (2156 ln) | UNWIRED |
| g32 pass 5 | `8b4fe36c` | `render/tile_draw_pass.rs` (921 ln) | UNWIRED |

**Stale-claim correction:** g32's ANALYSIS says "g29 (pass 2) and g31 (pass 4)
were EMPTY claims (branch tips = base, no commits)". That was true at g32's
fork time only. Both lanes have since pushed full ports (g29 `e2487e70` →
`9bb92d0b`, g31 `4a3e354d` → `3af480de`). The merge must NOT treat g32's
duplicated area/orchestration maths as the sole copy — the keep-one decisions
below supersede that note.

---

## 1. Per-file KEEP/DROP

### 1.1 `crates/tr-app/src/render/tile_pass.rs` — g28-vs-g29 SAME-FILE union

Both lanes CREATE this file from a base where it does not exist → **add/add
conflict on the whole file**. They are not complementary halves; g29 is a
superset orchestration with an executable driver, g28 is a head-only subset
plus a stand-in registry.

**BASE = g29's `tile_pass.rs` (tip `9bb92d0b`).** Rationale: it owns the
`draw_layer` driver + `TilePassSeams` trait that every sibling binds through,
the full prologue/epilogue order, `DrawBlackHelper`, the 12-step
`POST_DRAW_NON_SOLID` (incl. `EndBatch` + `NatureAfterAllObjects`), the
placement-preview kite fade, and the `detectCreature` frame flip. g28's own
module doc concedes the registry: "the duplicate registry below is the
pre-merge stand-in this module's witnesses run against" (g28 tile_pass.rs:73-74).

Per-symbol disposition of g28's file (all DROP unless noted):

| g28 symbol | Verdict | Canonical survivor |
|---|---|---|
| `TileCounterType`, `TILE_COUNTER_COUNT`, `MAX_SPECIALS` | DROP | g30 `special_points.rs` (`#[repr(usize)]`, discriminants = indices) |
| `SpecialPoints` (buckets + vine dedup + legacy Vec), `add`/`add_legacy`/`clear_counts`/`clear_legacy` | DROP | g30 `SpecialPoints` (adds `liquid_drop_cells`, `single_tiles`, 9000 cap); legacy list → g32 `LegacyCaches` (§1.3) |
| `screen_draw_area` (tuple return), `draw_area_loop_bounds` | DROP | g29 `get_screen_draw_area`/`draw_loop_bounds` (§2.1) |
| `is_tile_draw_layer_solid`, `should_sway_in_wind` | DROP | g30 (identical bodies; g30's sway gate is the 227-only faithful one) |
| `draw_layer_skips` | DROP, FOLD doc | g29's `:517` gate inside `draw_layer`; fold the `ShowUnbreakableWall && wall == 350` debug-arm note into the gate comment (toggle fed `false`, both lanes' recorded deviation) |
| `LAYER_*` constants | DROP | g29's (identical values 0/1/2/3; g32's copy also drops, §1.3) |
| `POST_DRAW_ORDER: [TileCounterType; 10]` | DROP | g29 `POST_DRAW_NON_SOLID` (same 10 + EndBatch + NatureAfterAll); bucket mapping → g31 `drain_pass_bucket` |
| `collect_reverse_vine_root`, `collect_vine_root` | DROP | g30 `collect_reverse_vine_root` + `apply_cell_class`'s `VineCrawl` arm (both delegate the forward crawl to the on-main `vines::collect_vine_root`, same as g28) |
| `clear_cached_tile_draws` | DROP | g29 `layer_cache_clears` (the `SpecialBlockCounts` step binds g30 `clear_special_block_counts`; solid arm binds g32 `LegacyCaches` clothing pair) |
| `DrawPass`, `begin_draw_pass` | DROP | g29 `draw_layer` sequences the same head (`:477` legacy clear → `:478` layer clear → `:479-495` tiers → `:496` area) plus the steps g28 omitted (`:472-476` critterCage/wind grid, `:497-503` black helper/glow/paint key) |
| g28 `mod tests` (14) | DROP with keep-one map | §3 witness table |

**KEEP from g28 outside tile_pass.rs:** `crates/tr-render/src/quality_light.rs`
(whole file, 6 witnesses) + its `lib.rs` `pub mod quality_light;` line — this
is g28's real payload; g29's `TilePassSeams::rebuild_quality_light_tiers`
binds to `quality_light_tiers(gfx_quality, dev_cheat)` exactly as its doc says.

### 1.2 `crates/tr-app/src/render/special_points.rs` — g30 CANONICAL

KEEP whole file (tip `cd7f7d31`, incl. the `#![allow(dead_code)]` convention
line). It is the only classify port: `classify_tile` (`:534-723` every arm),
`CellClass`, `apply_cell_class`, `cache_tree_point` (`:4260-4284`),
`collect_special_points`, the registry + `clear_special_block_counts`
(order 3,2,6,4,1,10,0,7,8,9 — BiomeGrass 5 skipped). No other lane ports the
switch; g28's stand-in drops (§1.1).

Fold at merge (driver de-dup, per g29's module doc): g30's
`collect_special_points` loses its own loop/`:517` gate — g29's `draw_layer`
owns the walk and calls `classify_tile` + `apply_cell_class` per cell through
`TilePassSeams::classify_and_draw_cell`. Keep the per-cell half; the standalone
driver stays as the witness fixture (its 14 tests run against it unchanged).

### 1.3 `crates/tr-app/src/render/tile_draw_pass.rs` — g32-vs-g29 orchestration dup

Different file path → no textual conflict, but ~60% semantic duplication with
g29. **BASE = g29 `tile_pass.rs` for everything duplicated; g32's file survives
REDUCED to the pieces g29 does not have.** Per-symbol:

| g32 symbol | Verdict | Notes |
|---|---|---|
| `LAYER_*`, `MAX_SPECIALS_LEGACY` | DROP | g29's identical constants |
| `ScreenDrawArea`, `get_screen_draw_area`, `draw_loop_bounds` | DROP, FOLD flags | g29's struct/fn canonical (§2.1); fold g32's `needs_framing`/`needs_refresh` fields (`:6574-6585` section-framing deviation) onto g29's `ScreenDrawArea` + the two extra ctor inputs |
| `ensure_wind_grid_size` | KEEP | g29 only has the seam call; this is the real `WindGrid::set_size` binding (UNPADDED clamped extents, grow-only) |
| `martian_glow` ([u8;4]) | DROP, FOLD | g29 `martian_glow_byte` canonical (same `as i32 as u8` maths); the `[b,b,b,0]` quad construction folds into the `set_martian_glow` seam binding |
| `single_tile_layer`, `cell_batch_layers` | KEEP | g32-only (`:877-887` pick, `:521-526` SetLayer sequence) — S1-G-13 layer-target family inputs |
| `LegacyCaches` (+ `add_special_legacy_point`, `clear_legacy_cached_draws`, `clear_cached_tile_draws`) | KEEP | the storage canonical: all 8 dictionaries + the 1000-cap legacy list. g29's `LEGACY_CLEAR_ORDER`/`layer_cache_clears` enums stay the ORDER canonical and bind step-by-step onto these fields/methods; g29's `LegacySpecialPoints` DROPs (its cap witness rebinds here). The `clear_cached_tile_draws(false) == true` return is the g30 `clear_special_block_counts` seam flag |
| `DrawBlackHelper` ([f32;4] return) | DROP | g29's `BlackStrip`-returning copy canonical (§2.2); fold g32's offset/row-break witness |
| `DrawStep`, `draw_steps`, `placement_preview_draws`, `legacy_restarts_batch` | KEEP | `draw_steps` is the full C# head/tail enumeration witness (incl. `EffectsActiveGate`/`PerspectivePlayer` which g29 folds into `DrawLayerInputs`); `legacy_restarts_batch` (`:7895-7899` empty-cache early return) is g32-only. `placement_preview_draws` duplicates g29's `PlacementPreviewGate::draws` — KEEP as the truth-table witness only, bind the driver to g29's gate |
| `draw_liquid_behind_tiles_cells` | DROP | g29's `draw_liquid_behind_tiles` driver canonical (same null-only gate); fold g32's inactive-cell/OOB witness assertions |
| `PostPass`, `post_draw_tiles_passes`, `post_pass_bucket` | DROP, FOLD | §2.3 — g29 owns the outer order, g31 owns the bucket map; fold g32's `None` arms (NatureAfterAll/HatRacks/DisplayDolls) into the g31 mapping witness |
| `FrameTileStep`, `frame_tile_draw_order`, `render_target_tile_order` | KEEP | g32's tile-PHASE granularity (`LiquidBehindTilesPass` + Draw/Post pairs + capture reverse with NO post passes) complements g29's NPC-relative `walls_tiles_npcs_order` skeleton. g29's `render_target_layer_order() -> [bool;2]` DROPs in favour of g32's richer `render_target_tile_order`; fold its hideTiles/hideTiles2 gate notes (`:57334`/`:57355`) into the g32 doc |

### 1.4 `crates/tr-app/src/render/tile_drains.rs` — g31 drain bodies (this lane)

KEEP whole file (tip `3af480de`). The five MISSING bodies
(`drain_multi_tile_vines`/`multi_tile_vines_in_wind`,
`drain_multi_tile_grass`/`multi_tile_grass_in_wind`, `drain_grass`,
`drain_any_directional_grass`, `drain_reverse_vines`/`rising_vine_strip`)
exist nowhere else. Attach points (§4 bind order):

* `DrainBuckets.points: [Vec<(i32,i32)>; 11]` ↔ g30 `SpecialPoints.buckets` —
  identical shape; at merge, pass `&g30_reg.buckets` (or a thin view) instead
  of a second registry. `BUCKET_*` constants keep-one onto g30
  `TileCounterType as usize` (values verified identical 0..10).
* `DrainCellLight` → d56 `tr_render::animated_tiles`
  (`vision_changer_light` → `light_override`, gated emission → d60). The
  `can_do_dust` input is the drain's ALREADY-CONSUMED `_rand.Next(4)==0`
  flag — the binding must not re-roll (stream-order rule).
* `PortedBodyDrain` → on-main `foliage.rs` (tree_point), `vines.rs`
  (vine_root), `pylons.rs` (void_lens_point/pylon_point/trophy_point).
* `is_visible` — third copy of `IsVisible` (`:1411-1432`): g31's, d56's
  `tile_draw_gate` internal, d60's `is_visible`, plus the private `pylons.rs`
  one. **Keep-one: d56's `tr_render` copy** (lowest crate, already pub);
  g31/d60/pylons rebind at wiring. Until then all copies stay (code-only).
  *(CORRECTION, research/136 §4: d56's tip has NO standalone fn — the
  logic is inlined in `tile_draw_gate`. The mechanical keep-one is
  d60's `is_visible` PROMOTED to tr-render; d56's gate calls it.)*
* `post_draw_drain_order` driver: at merge, FOLD into a per-pass dispatch
  (`drain_one_pass(pass, ...)`) so g29's `post_draw_tiles` can bind each of
  its ten nature steps individually (§2.3). KEEP `POST_DRAW_DRAIN_ORDER` +
  `drain_pass_bucket` as the frozen block-order/bucket witnesses; the three
  driver witnesses (§3) re-run against the per-pass dispatch in the same
  order — the `_rand` stream sharing across passes must survive the fold.

### 1.5 Shared small files

* `crates/tr-core/src/torch.rs` — g28 and g30 both lift
  `top_edge_can_be_attached_to` to `pub` with different doc comments →
  trivial conflict. **Keep-one: `pub` + g30's doc, extended with g28's
  "(W4-g28 pass 1 / g30 pass 3 — identical visibility change)" note.** Body
  unchanged in both.
* `crates/tr-app/src/render/mod.rs` — five lanes add declarations at two
  insertion points. Union = keep ALL of: `special_points` (g30, after `sky`),
  `tile_drains` (g31, after `tile_cracks`), `tile_draw_pass` (g32, same
  spot), and ONE `tile_pass` declaration (g28/g29 add/add — keep g29's
  comment, mention g28's folded head). d57's `te_furniture` (after
  `special_tiles`) composes cleanly.
* `crates/tr-render/src/lib.rs` — g28 `pub mod quality_light;`, d56
  `pub mod animated_tiles;` + re-exports, d58 `pub mod chest_frames;` +
  re-exports: distinct lines, alphabetical order preserved; context-only
  conflict risk, no semantic overlap.
* `docs/ANALYSIS.md` / `docs/PORT_MAP.md` / `anchors.txt` — all five g-lanes
  append distinct sections/rows; union = keep all, EXCEPT: g32's ANALYSIS
  "g29/g31 EMPTY claims" paragraph gets the stale-claim correction note
  (top of this doc); the g28 anchors row (`begin_draw_pass`) repoints to
  `quality_light_tiers` + the g29 `draw_layer` row (its tile_pass.rs anchor
  symbol drops); g28's PORT_MAP tile_pass row merges into g29's.
* `docs/research/09-tile-drawing.md` — g29 and d56 edit DIFFERENT rows
  (row 2 screen-area loop vs the animated-tiles row); keep both.

---

## 2. The three contested dup resolutions

### 2.1 `GetScreenDrawArea` — three ports, one maths

g28 `screen_draw_area` (tuple), g29 `get_screen_draw_area` (`ScreenDrawArea`,
`off_screen_range: i32`), g32 `get_screen_draw_area` (`off_screen_range: f32`
+ framing flags). All three verified identical: `-1f` INSIDE the first
casts, `+2`/`+5` OUTSIDE the last casts, truncate-toward-zero `as i32`, four
clamps `max(4)` / `min(maxTiles - 4)` with NO first<=last reorder.

**Canonical: g29's** (the driver consumes it; `i32` matches C#
`Main.offScreenRange`). Fold g32's `needs_framing`/`needs_refresh` output
flags + its trunc-toward-zero (`trunc(-0.5)+2 NOT floor`) witness. DROP g28's
tuple version and g32's copy. g28's `begin_draw_pass` inclusive-rect fold
(`xe-1`, `ye-1`) is NOT carried over — the g30 driver de-dup (§1.2) means the
walk stays g29's exclusive `y0..y1`/`x0..x1` form.

### 2.2 `DrawBlackHelper` — two ports, one quirk

g29 returns `BlackStrip { position, size, layer }`; g32 returns `[f32;4]`.
Identical coalescing (`y == self.y && x == end_x`), identical i32-shift-before-
float maths, identical **no-reset `EndStrip` quirk** (second call re-emits;
g29's tip `9bb92d0b` re-derived that the re-emit after a break is the NEW
pending run `(14,6)`, not the flushed `(14,5)`).

**Canonical: g29's** (explicit `layer` field; `draw_layer` already threads it
through `classify_and_draw_cell` + the `:728` tail flush). DROP g32's copy;
fold its offset-maths/row-break witness onto `BlackStrip`. The wiring replaces
`tiles.rs`'s `merge_dark_row` (`tiles.rs:769+`) with this helper on
`LAYER_TILES` — contested with d56's `tile_draw_gate` (which owns the
`DrawBlack` FEED decision, `:988-990`); gate = d56, coalescer = g29.

### 2.3 `PostDrawTiles` — three order ports + one body port

* g29 `POST_DRAW_NON_SOLID` (12 steps: 10 nature + `EndBatch` +
  `NatureAfterAllObjects`) / `POST_DRAW_SOLID` (HatRacks → DisplayDolls) +
  the `post_draw_tiles` driver.
* g32 `post_draw_tiles_passes` (11 steps, no explicit EndBatch) +
  `post_pass_bucket` (literal usize).
* g31 `POST_DRAW_DRAIN_ORDER` (10) + `drain_pass_bucket` + the
  `post_draw_drain_order` body driver.
* g28 `POST_DRAW_ORDER` (10, enum-typed).

All four pin the SAME C# order (`:421-430`): MultiTileVines, MultiTileGrass,
VoidLenses, TeleportationPylons, MasterTrophies, Grass, AnyDirectionalGrass,
Trees, Vines, ReverseVines; solid arm HatRacks BEFORE DisplayDolls.

**Resolution:** outer driver = **g29 `post_draw_tiles`** (only copy with the
`:431` EndBatch step and the `:433` nature flush as first-class steps). The
`post_draw_pass` seam binds: the ten nature steps → g31's per-pass drains
(§1.4 fold) over the g30 buckets via **g31 `drain_pass_bucket`** (keep-one
bucket map; g32's `post_pass_bucket` drops, its `None`-arm assertions fold
into g31's bucket witness); `EndBatch`/`NatureAfterAllObjects` → the batch/
nature seam; `DrawEntitiesHatRacks`/`DrawEntitiesDisplayDolls` → d57's
`te_furniture` doll/hat-rack passes. g28's and g32's order constants DROP as
duplicates; g31's stays as the frozen block-order witness.

---

## 3. Witness-union inventory (all WRITTEN NOT RUN — code-only mode)

Actual `#[test]` census at the tips (not the commit-message claims):

| File | Lane | `#[test]` count |
|---|---|---|
| `render/tile_pass.rs` | g28 | 14 |
| `tr-render/quality_light.rs` | g28 | 6 |
| `render/tile_pass.rs` | g29 | 16 |
| `render/special_points.rs` | g30 | 14 |
| `render/tile_drains.rs` | g31 | 22 |
| `render/tile_draw_pass.rs` | g32 | 20 |
| **Union before keep-one** | | **92** |

Keep-one map for the contested witnesses (DROP = covered by the named
survivor; REBIND = keep the test, retarget to the canonical symbol):

| g28 witness (all 14 drop with the file) | Survivor |
|---|---|
| `the_counter_discriminants_are_the_bucket_indices`, `add_appends_without_dedup`, `clear_counts_skips_biome_grass_and_legacy` | g30 `the_counter_discriminants…`, `add_special_point_appends_and_caps_at_9000`, `clear_skips_the_biome_grass_bucket` |
| `the_counts_clear_only_before_the_non_solid_pass` | g29 `layer_cache_clear_split` + g32 `the_non_solid_clear_chains_the_registry` |
| `the_screen_area_truncates_pads_and_clamps`, `the_offscreen_pad_widens_both_edges`, `the_loop_bounds_fold_the_margins` | g29 `screen_area_pads_and_offset`, `screen_area_clamps_at_the_four_cell_border`, `draw_loop_bounds_are_plus4_minus2` (+ g32 trunc REBIND below) |
| `the_solid_layer_override_beats_tile_solid`, `sway_gate_is_settings_set_and_227_only` | g30 same-named pair |
| `the_skip_gate_matches_the_layer_and_debug_arm` | g29 `cell_gate_and_solid_liquid_front_step` (fold the wall-350 debug-arm assertion) |
| `the_layers_and_post_draw_order_are_frozen` | g29 `layer_constants_and_legacy_cap` + `post_draw_orders` |
| `the_forward_crawl_roots_below_the_attachment`, `the_reverse_crawl_roots_above_the_attachment` | g30 `vine_threads_crawl_and_549_reverses` + `the_reverse_crawl_roots_above_the_attachment` |
| `the_draw_head_clears_and_builds_in_order` | g29 `draw_layer_prologue_epilogue_order` + g32 `the_non_solid_draw_step_order` |

g32's 20: KEEP 12 (`the_wind_grid_sizes_to_the_unpadded_extents`,
`the_single_tile_layer_pick`, `the_cell_layer_sequence`,
`the_clear_split_between_legacy_and_solid`,
`the_non_solid_clear_chains_the_registry`, `the_non_solid_draw_step_order`,
`the_solid_draw_step_order`, `the_preview_gate_truth_table`,
`the_frame_tile_phase_orders`, + REBIND `the_area_cast_truncates_toward_zero`
→ g29 area, `the_black_strip_offset_and_row_break` → g29 `BlackStrip`,
`the_legacy_points_append_and_cap_at_1000` → `LegacyCaches`); DROP 8 as dups
(`the_screen_area_truncates_and_clamps`, `the_walk_bounds_pad_the_area`,
`the_martian_glow_is_the_byte_ramp` (fold the `[b,b,b,0]` assertion into g29's
wrap witness), `the_black_helper_merges_strips`,
`the_behind_liquid_walk_includes_inactive_cells` (fold OOB-skip into g29's
`liquid_behind_tiles_has_no_active_gate`), `the_non_solid_post_pass_order`,
`the_solid_post_pass_order`, `the_post_pass_bucket_indices` (fold `None` arms
into g31 `post_draw_drain_order_and_buckets`)).

g29 16, g30 14, g31 22, quality_light 6: KEEP all (g29's
`legacy_points_append_and_cap` REBINDs to `LegacyCaches` per §1.3; g31's three
driver witnesses re-run against the per-pass fold per §1.4).

**Post-union expected total: 6 + 16 + 14 + 22 + 12 = 70 witnesses** (92 − 14
g28-file − 8 g32 dups), every dropped assertion folded into its survivor.
Name-level inventory is the `fn w4_g*` census in each file's `mod tests`.

---

## 4. Bind-order checklist (tiers → area → classify → drains → post)

The wiring step drives `render/tiles.rs::draw_tiles` split into
`draw_layer(solid: false)` then `draw_layer(solid: true)` (screen path;
render-target capture is SOLID-first with NO post passes — g32
`render_target_tile_order`). Per layer, in C# order:

1. **Head** (g29 `draw_layer` prologue): critterCage reset (non-solid only,
   `:472-475`) → `ensure_wind_grid_size` (g32 fn, fresh UNPADDED area probe,
   `:476`) → `LEGACY_CLEAR_ORDER` 7 steps → g32 `LegacyCaches` (`:477`) →
   `layer_cache_clears` (`:478`; non-solid `SpecialBlockCounts` → g30
   `clear_special_block_counts`; solid → clothing pair).
2. **Tiers**: `TilePassSeams::rebuild_quality_light_tiers` → g28
   `tr_render::quality_light::quality_light_tiers(gfx_quality, false)`
   (`:479-495`; devLightTilesCheat fed false).
3. **Area**: g29 `get_screen_draw_area(!draw_to_screen, …)` (`:496`) → fresh
   `DrawBlackHelper(LAYER_TILES, draw_offset)` (`:497`) →
   `martian_glow_byte` (`:498-499`) → paint-key reset (`:500-503`) →
   `draw_loop_bounds` (+4/-2/+2, `:504-506`).
4. **Classify** (per cell, row-major): `:517` gate via g30
   `is_tile_draw_layer_solid` → solid-layer liquid-behind front step
   (`:521-525`, `liquids.rs`) → g30 `classify_tile` + `apply_cell_class`
   (registry side effects; `LiquidDrops` cells recorded for d60) →
   `DrawSingleTile` port for `SingleTile`/`DeferredAndSingle` cells, with
   d56 `tile_draw_gate` + `final_light` and g30 `cache_tree_point` at `:992`;
   dark cells feed the g29 black helper (d56 gate decides, g29 coalesces).
5. **Epilogue** (`:728-741`): black `end_strip` flush → RestartLayeredBatch →
   solid-only cracks hitReplace-then-hitTile + RestartSpriteBatch
   (`tile_cracks.rs`, wired) → `DrawSpecialTilesLegacy` (`special_tiles.rs`
   + d57 arms over `LegacyCaches.legacy_points`) → preview gate (g29
   `PlacementPreviewGate` + kite fade).
6. **Drains / post** (after EACH layer's Draw, `Main.cs:21758-21761`): g29
   `post_draw_tiles(solid, seams)` — non-solid: ten nature steps → g31
   per-pass drains over g30 buckets (`drain_pass_bucket`), `_rand` ONE shared
   stream in pass order, `DrainCellLight` → d56 light chain, dust flags →
   d60; then `EndBatch` → `NatureAfterAllObjects`. Solid: HatRacks →
   DisplayDolls (d57).
7. **Frame skeleton**: g29 `walls_tiles_npcs_order(detect_creature)` for the
   NPC-relative order; g32 `frame_tile_draw_order` for the tile-phase order
   (behind-liquid pre-pass rides the background water, `Main.cs:57527`).

---

## 5. Census — d56/d57/d58/d59/d60 contested-seam attach points

None of the five d-lanes touches `tile_pass.rs`, `tile_draw_pass.rs`,
`tile_drains.rs`, `special_points.rs` or `torch.rs` (verified: empty diffs).
The contested seam is `render/tiles.rs::draw_tiles` + `special_tiles.rs`.

### d56 — `task/W4-d56-tile-draw-animated` (tip `5d80b38a`) — UNWIRED
* Module: `crates/tr-render/src/animated_tiles.rs` (1254 ln) + `lib.rs`
  `pub mod animated_tiles;` + pub re-exports (`final_light`, `tile_draw_gate`,
  `vision_changer_light`, `light_override`, `shine_color`, `should_tile_shine`,
  `tile_shine2`, `DrawGate`, `VisionDust`, `VisionLight`, dust-gate consts).
* Attach points (ANALYSIS, verbatim): "the gate replaces the dark-cell-merge
  + `IsVisible` gates (`tiles.rs:786-848`), and `final_light` replaces the
  actuation-ghost-only `l` (`tiles.rs:610-619`)"; "the seam stays contested by
  d57/d58/d59/d60". Dust rolls are `VisionDust` REQUESTS — "the wiring must
  reproduce that consumption" (caller-owned `_rand`).
* g31 bind: `DrainCellLight` → `vision_changer_light` (`:8481-8552`) →
  `light_override` (`:6624-6686`); g31's `is_visible` keep-ones onto d56's.
* Witnesses: **24** `#[test]` actual (anchors claim 13 + 10 = 23 — one-test
  claim/census drift; reconcile the anchors row at merge, code wins).

### d57 — `task/W4-d57-tile-draw-te-furniture` (tip `70f43aa7`) — UNWIRED
* Module: `crates/tr-app/src/render/te_furniture.rs` (2151 ln) + `mod.rs`
  declaration; `projectile_ai_spear.rs::spear_offset` lifted `fn` →
  `pub(crate) fn` (display-doll aiStyle-19 preview shares it).
* Attach points (module doc/ANALYSIS): the 395/520/471
  `DrawSpecialTilesLegacy` arms (`:7951-8000`) + `CacheSpecialDraws_Part1`
  anchor collection (`:4154-4247`) — "the `draw_tiles`/`app/frame/draw.rs`
  seam is contested"; anchors row: "the draw_tiles seam wiring is the
  d58/d59/d60 family merge step". Binds: g29
  `TilePassSeams::draw_special_tiles_legacy` + `post_draw_pass(HatRacks/
  DisplayDolls)`; g32 `LegacyCaches` TE-position dictionaries (its doc: "the
  TE lookups are the d57 furniture slice's seam"); g31's 698 dead-cells-jar
  `DrainJarItem` rare/expert deferral reads the TeItem store d57 ports.
* Witnesses: **44** `#[test]` actual in `te_furniture.rs`.

### d58 — `task/W4-d58-tile-draw-chest-frames` (tip `a13635dc`) — **WIRED**
* Modules: `crates/tr-render/src/chest_frames.rs` (195 ln, keep-one MOVE of
  `chest_frame_adjust`/`chest_anchor` OUT of `special_tiles.rs` — the
  S1-G-1e anchor row repointed), `chest.rs` +135 (`chest_draw_frame`,
  `chest_frame_key`), `tiles.rs` +87, `app/mod.rs` +63, `app/frame/draw.rs`
  +36 (G6 `WorldFrameKey::chest_frames`).
* Attach points (actual hunks): `draw_tiles` signature gains
  `chests: &[crate::chest::WorldChest]` (`tiles.rs:325` hunk); per-draw
  `_chestPositions` cache `HashMap<(i32,i32), Option<i32>>` (`:586` hunk,
  `TileDrawing.cs:4300-4303`, fresh per call = `ClearLegacyCachedDraws`
  `:6613-6615`); the BasicChest rewrite runs AFTER the `GetTileDrawData`
  dispatch, BEFORE the source rect (`:731` hunk, C# `:993` vs `:1007`).
* Merge conflicts: `special_tiles.rs` — d58 DELETES the chest section
  (`@@ -59,90 +59,14 @@`) and edits the module doc (`@@ -10,10`); d59 edits
  the SAME doc block (`@@ -4,19`) and its `FillNeighbor` hunk (`@@ -149`)
  sits immediately after the deleted `chest_frame_adjust`. Resolution: take
  d58's deletion + union both doc rewrites (d58: chest moved to tr-render;
  d59: side-fills wired). In `tiles.rs` the hunks do not overlap (d58 at
  :325/:586/:731/:1873, d59 at :1017/:1575/:2225) — d58's chest rewrite
  stays upstream of d59's slope family, matching C# order.
* g-lane bind: g32 `LegacyCaches.chest` is the orchestration-side key set;
  d58's per-draw HashMap is the wired implementation — keep d58's, the
  `LegacyCacheClear::ChestPositions` step binds its clear.
* Witnesses: 6 (`chest_frames.rs`) + 3 (`chest.rs` delta) + 1 (`tiles.rs`
  draw-feed) = **10** new.

### d59 — `task/W4-d59-tile-draw-platform-slopes` (tip `f70498be`) — **WIRED**
* Modules: no new file; `tiles.rs` +452 (the `DrawBasicTile` slope family at
  the `:1017` hunk: `DontDrawTileSlopes` gate → platform flat rect +
  `draw_platform_side_fill` → `HasSlopeFrames` flat draws → `push_slope`
  fallback → `half_brick_join`/`push_half_brick_join`, C# `:1510-1629`);
  `special_tiles.rs` +89 (`FillNeighbor::from_world` reads the real Echo bit
  via `World::invisible_block`; `platform_side_fill` updates; module doc
  flips to "Partially wired (W4-d59)").
* Attach point: inside `draw_tiles`' per-cell dispatch, downstream of d58's
  chest rewrite; the behind-tiles LAYER decision rides
  `PlatformSideFill.behind_tiles` until S1-G-13 — g32's `single_tile_layer`/
  `cell_batch_layers` are the constants that family will consume.
* Witnesses: 5 (`tiles.rs` delta) + 1 (`special_tiles.rs` delta) = **6** new.

### d60 — `task/W4-d60-tile-draw-particles` (tip `77429fe1`) — UNWIRED
* Module: `crates/tr-app/src/tile_draw_particles.rs` (2163 ln) + `main.rs`
  `mod tile_draw_particles;` — "the DrawSingleTile call-site gate + the
  inline 192-leaf reconciliation are the wiring wave".
* Attach points (ANALYSIS, verbatim): "The inline 192-leaf arm
  (`render/tiles.rs:1078-1122`) gates on … the wiring wave must reconcile
  the inline arm with it"; the rubble gate's "d54 KillTile_MakeTileDust
  seam deferred" (`RubbleDustPlan`). Two-stream RNG census: `_rand` gates vs
  `Main.rand` pool/luck/args — the `(x,y).Next(4)==0` quarter-roll
  consumption "is the wiring wave's obligation".
* g-lane binds: g30 `SpecialPoints.liquid_drop_cells` feeds `EmitLiquidDrops`
  (`:693`, `:6688+`); g31's drains pass the already-rolled `can_do_dust`
  through `DrainCellLight` — d56 computes light, d60 emits; neither re-rolls.
  d60's `is_visible` copy keep-ones onto d56's (§1.4).
* Witnesses: **22** `#[test]` actual (matches the claim).

---

## 6. Verification commands (for the merger; NOT run in this code-only phase)

```
# per-lane compile + witnesses after the keep-one folds
cargo test -p tr-render quality_light
cargo test -p tr-render animated_tiles
cargo test -p tr-render chest_frames
cargo test -p tr-app --lib render::tile_pass
cargo test -p tr-app --lib render::tile_draw_pass
cargo test -p tr-app --lib render::special_points
cargo test -p tr-app --lib render::tile_drains
cargo test -p tr-app --lib render::te_furniture
cargo test -p tr-app --lib tile_draw_particles
cargo test -p tr-app --lib render::tiles          # d58/d59 wired witnesses
# full gate
cargo test --workspace
```

Expected post-union witness census for the g-family: **70** (§3). Anchor
canaries after the merge: `quality_light_tiers`, `draw_layer` (tile_pass),
`collect_special_points`, `post_draw_drain_order`, `post_draw_tiles_passes`
(g32's row survives — the symbol is KEPT in the reduced file), plus the
d-lane rows unchanged.

*This contract was produced code-only: no cargo/rustc/tests were invoked; all
claims above cite `git show`/`git diff` reads of the pushed tips. No lane's
code was modified.*

---

## 7. EXECUTION RECORD (union-exec, this branch)

Executed on `task/W4-g31-tile-draw-pass-4` on top of the contract commit
`53603df1`, in four phases (code-only — no cargo/rustc/tests/graphify;
every seam compiled by inspection):

| Phase | Commit | Scope |
|---|---|---|
| A — land | `69675330` | g29 `tile_pass.rs` (`9bb92d0b`), g30 `special_points.rs` (`cd7f7d31`), g32 `tile_draw_pass.rs` (`8b4fe36c`), g28 `quality_light.rs` (`3f1a40cf`) landed verbatim via `git checkout <tip> -- <path>`; `render/mod.rs` union declarations (§1.5), `tr-render/lib.rs` `pub mod quality_light;`, `torch.rs` pub keep-one (g30 doc + g28 note). g28's stand-in `tile_pass.rs` NOT landed (§1.1 DROP). |
| B — g29 folds | `dd4f2948` | §2.1 framing flags onto `ScreenDrawArea` + 2 ctor inputs threaded through `DrawLayerInputs` and all three probes; `LegacySpecialPoints` DROP → g32 `LegacyCaches` (cap witness REBIND); `render_target_layer_order` DROP → g32 `render_target_tile_order` (gate notes folded); the `:517` wall-350 debug-arm comment fold; the 2 g32 REBIND witnesses (`the_area_cast_truncates_toward_zero`, `the_black_strip_offset_and_row_break`) land here; the martian `[b,b,b,0]` quad + the behind-liquid OOB/inactive assertions fold into their g29 survivors. |
| C+D — g32 reduction + g31 binds | `44f423cb` | §1.3 KEEP set only in `tile_draw_pass.rs` (wind-grid fn over the g29 area type, layer pick/batch sequence, `LegacyCaches` + per-step `clear_legacy_step`/`clear_layer_step` g29-enum bindings, `draw_steps`/`placement_preview_draws`/`legacy_restarts_batch`, `FrameTileStep` phase orders); all dup constants import from `tile_pass`. §1.4/§2.3 in `tile_drains.rs`: `BUCKET_*` derive from g30 `TileCounterType`; drivers take `&[Vec<(i32,i32)>; 11]` (`&g30_reg.buckets` binds directly, `DrainBuckets` stays the owned fixture); `drain_one_pass` per-pass dispatch + `drain_pass_for_post_step` (the 12-step map; g32 `None` arms + g28 frozen bucket order fold into the bucket witness); `post_draw_drain_order` = thin frozen loop, ONE shared `_rand`; the rand-stream witness re-runs the per-pass fold for exact flag equality. |
| E — docs | (this commit) | ANALYSIS union (g28 ×2 / g29 / g32 sections + the stale-claim correction + the union-exec section), PORT_MAP rows (g28 quality_light, g28→g29 merged tile_pass row, g32 reduced row), anchors (g28 row repointed to `quality_light_tiers` + `draw_layer`; g30/g32 rows; the g32 marker repointed `post_draw_tiles_passes` → `draw_steps` — the symbol DROPPED per §1.3, the §6 canary note superseded), research/09 row 2 (g29's edit; d56's row untouched — not on this branch). |

**Witness census (actual `#[test]` after the folds):** quality_light 6 +
tile_pass 18 (g29 16 + 2 g32 REBINDs) + special_points 14 + tile_drains 22
+ tile_draw_pass 10 = **70** — the §3 target met. Census delta vs the §3
name map: the map's "g32 KEEP 12" arithmetic (9 listed KEEP + 3 REBIND)
counted the two g32 REBINDs that landed in `tile_pass.rs`
(`the_area_cast_truncates_toward_zero`,
`the_black_strip_offset_and_row_break`) on BOTH sides — they live once,
in the canonical file (tile_pass 18 = 16 + 2); the reduced
`tile_draw_pass.rs` keeps its 9 listed KEEP + the third REBIND
(`the_legacy_points_append_and_cap_at_1000` onto `LegacyCaches`) = 10.
The g29 `legacy_points_append_and_cap` REBIND and the g32 legacy-cap
REBIND both pin the SAME `LegacyCaches` API from their own lanes'
angles (append/cap/clear vs append/cap/order) — both kept per §3's
"g29 16 KEEP all" rule. No dropped assertion is lost: all 14 g28-file
and 8 g32-dup witnesses fold into the named survivors (§3 tables).

**Deviations from the prescription (recorded, deliberate):**
* §1.3 said g32's `post_draw_tiles_passes` "survives — the symbol is
  KEPT in the reduced file" (§6 canary note) while the same table's
  verdict row says `PostPass`/`post_draw_tiles_passes`/`post_pass_bucket`
  DROP, FOLD. The table wins (it is the per-symbol disposition); the
  anchors row repoints to `draw_steps` so the canary stays armed.
* §1.4's "pass `&g30_reg.buckets` (or a thin view)": the drivers take
  the array slice type directly; `DrainBuckets` remains as the owned
  witness fixture only (its three driver witnesses pass
  `&buckets.points`).
* `is_visible` keep-one onto d56's `tr_render` copy is NOT executed —
  d56 is not on main (§1.4 "Until then all copies stay"). Same for the
  `DrainCellLight` → d56 and dust → d60 binds; `PortedBodyDrain` →
  foliage/vines/pylons binds at the tiles.rs wiring (the bodies need
  the renderer, the seam is the compile-time boundary).

**UNWIRED attach (contested, documented not executed):**
`render/tiles.rs::draw_tiles` untouched. The §4 checklist is the wiring
script; the §5 d-lane census (d58/d59 WIRED hunk zones, the
special_tiles.rs doc/chest-deletion resolution) applies at the family
merge with the d-lanes. *(Superseded by the attach phase below: the
shadow attach landed behind `TR_TILEPASS`, §8.)*

---

## 8. ATTACH-CONTRACT ADDENDUM (attach phase, this branch)

Executed on `task/W4-g31-tile-draw-pass-4` on top of the union-exec
(`c3027a38`), code-only. Two commits: `8d7a3532` (F1, the ctor-draw
residual) + `2f5e0698`/`a584eb57` (F2, the gated shadow attach). This
section is the census verdict, the exact seam signatures each d-lane
must call, and the merger wiring instructions for the PIXEL binding.

### 8.1 Census verdict — compatible for a SHADOW attach, not for a gate-flip pixel swap

Re-read the pushed tips (§5 census confirmed, d56 `5d80b38a` / d57
`70f43aa7` / d58 `a13635dc` / d59 `f70498be` / d60 `77429fe1`):

* **d56/d57/d60 UNWIRED** — module-only payloads (`animated_tiles.rs`,
  `te_furniture.rs`, `tile_draw_particles.rs`); their attach claims are
  REPLACEMENTS of fused-loop regions (d56: the dark-cell merge +
  `IsVisible` gates `tiles.rs:786-848` and the actuation-ghost `l`
  `:610-619`; d60: the inline 192-leaf arm `:1078-1122`; d57: the
  `DrawSpecialTilesLegacy` arms + the two TE post passes). None of them
  can bind until the fused loop is split — they are the
  `classify_and_draw_cell` / `post_draw_pass(TE)` seam IMPLEMENTATIONS.
* **d58/d59 WIRED** — unconditional per-cell hunks INSIDE the fused
  loop (d58: the `chests` parameter + the per-draw `_chestPositions`
  HashMap + the BasicChest rewrite after the drawdata dispatch; d59:
  the slope family inside the per-cell dispatch). No env gate — the
  "gate pattern the lanes use" is the `env.rs` named-accessor
  convention (`TR_NO_FOLIAGE`/`TR_SHOW_INVISIBLE`/…), which the shadow
  attach follows (`env::tilepass_shadow`, `TR_TILEPASS`, default OFF).
* **The union driver shape** (`draw_layer` + `TilePassSeams`, the
  12-step `post_draw_tiles` → `drain_pass_for_post_step` →
  `drain_one_pass` map) is a STRUCTURAL replacement of the whole fused
  walk: its `classify_and_draw_cell` seam must own the per-cell
  `DrawSingleTile` body — which exists ONLY as the fused loop itself
  (no standalone port). A pixel-binding attach therefore cannot be a
  drop-in call beside the fused loop; it is the §4 split, done at the
  family merge WITH the d-lane hunks folded into the seam impl.

**Consequence (executed):** the union is wired into `tiles.rs` as a
PLAN-ONLY shadow (`render/tile_attach.rs`, `TR_TILEPASS` default OFF)
at the C# `PostDrawTiles` position — the real orchestration, registry
and drain binds run over the live world every rebuild, proving the
seam shapes compile and the plans are frame-deterministic, while zero
pixels/streams/shared state move. The pixel binding is §8.3's script.

### 8.2 Exact seam signatures (the attach contract each lane binds)

All paths `crates/tr-app/src/render/` unless noted. These are the
EXECUTED union signatures (post-fold, this branch) — a d-lane binding
against any other shape is stale.

**The driver calls (g29 `tile_pass.rs`):**

```rust
pub(crate) fn draw_layer(
    world: &World,
    inputs: &DrawLayerInputs,
    is_tile_draw_layer_solid: fn(u16) -> bool,   // = special_points::is_tile_draw_layer_solid
    seams: &mut impl TilePassSeams,
) -> ScreenDrawArea;
pub(crate) fn draw_liquid_behind_tiles(world: &World, inputs: &DrawLayerInputs, seams: &mut impl TilePassSeams);
pub(crate) fn post_draw_tiles(solid_layer: bool, seams: &mut impl TilePassSeams);
```

`DrawLayerInputs` fields (all `pub`): `solid_layer`,
`into_render_targets`, `gfx_quality: f32`, `dev_light_tiles_cheat`,
`martian_light: f32`, `draw_to_screen`, `off_screen_range: i32`,
`scaled_position/scaled_size/screen_position: [f32; 2]`,
`allow_emit_effects`, `sway_enabled`, `any_unfinished_sections`,
`any_need_refresh`, `preview_gate: PlacementPreviewGate`,
`preview_is_kite`, `preview_hold_style: i32`,
`preview_frames_in_movement: i32`.

**`TilePassSeams` — every method is one C# call; the d-lane bind column
is the merger's obligation:**

| Method (exact signature) | C# site | Pixel-merge bind |
|---|---|---|
| `ensure_wind_grid_size(&mut self, width: i32, height: i32)` | `:476`/`:7773-7777` | g32 `tile_draw_pass::ensure_wind_grid_size(&fresh_area, &mut wind.wind_grid)` — the SHARED grid (the S1-G-8 update-site sizing `app/frame/draw.rs:862` then drops; the draw-site probe is C#'s) |
| `clear_legacy_cache(&mut self, step: LegacyCacheClear)` | `:477`/`:6613-6622` | g32 `LegacyCaches::clear_legacy_step(step)`; the `ChestPositions` step ALSO clears d58's per-draw HashMap (its wired implementation is the storage; §5 d58) |
| `clear_layer_cache(&mut self, step: LayerCacheClear)` | `:478`/`:6588-6599` | g32 `LegacyCaches::clear_layer_step(step)`; the returned `true` (`SpecialBlockCounts`) chains g30 `SpecialPoints::clear_special_block_counts()` |
| `rebuild_quality_light_tiers(&mut self, gfx_quality: f32, dev_cheat: bool)` | `:479-495` | g28 `tr_render::quality_light::quality_light_tiers(q, false)` → the impl stores `QualityLightTiers`; the per-cell sliced arm reads `tiers.slice_arm(tr_render::quality_light::tile_light_bytes(raw_l, style.brightness))` (replaces the fused loop's float `HIGH_LIGHT`/`MEDIUM_LIGHT` compares — research/09 deviation 11) |
| `reset_critter_cage(&mut self)` | `:472-475` | `Main.critterCage = false` — no port global yet; the special_tiles.rs:375 consumer binds when the critter family lands |
| `set_martian_glow(&mut self, b: u8)` | `:498-499` | store `[b, b, b, 0]` (the g28 quad fold); `martianLight` has no port global — fed 0.0 until the event lands |
| `reset_paint_lookup_key(&mut self)` | `:500-503` | the paint family's `_lastPaintLookupKey.TileType = -1` |
| `draw_tile_liquid_behind(&mut self, x: i32, y: i32)` | `:521-525` (solid), `:457` (pre-pass) | `liquids.rs::draw_tile_liquid_behind(r, world, lookups, light, style, anim_time, x, y, tile, water_style)` (the fused loop's existing call, moved into the seam) |
| `classify_and_draw_cell(&mut self, x: i32, y: i32, black: &mut DrawBlackHelper)` | `:526-724` | g30 `classify_tile(kind, fx, fy, sway_enabled)` + `apply_cell_class(world, x, y, kind, class, &mut reg)`; on `true` run the DrawSingleTile body = the fused loop's per-cell pipeline WITH d58's chest rewrite (after dispatch, before rect), d59's slope family, d56's `tile_draw_gate`/`final_light` (gate decides the black feed: `black.draw_black(x, y, screen_position)` → flush via `draw_black_strip`), d60's gated emission; `cache_tree_point(x, y, kind, dd-corrected fx, fy, &mut reg)` at `:992` |
| `draw_black_strip(&mut self, strip: BlackStrip)` | `DrawBlackHelper.cs:36` | push `strip.position/size` black on `strip.layer` (replaces the greedy `merge_dark_row` — §2.2: gate = d56, coalescer = g29) |
| `restart_layered_batch(&mut self)` | `:729` | the layered-batch restart (`TileDrawingBase.cs:34-40`) |
| `draw_tile_cracks(&mut self, site: CrackSite)` | `:732-733` | `tile_cracks.rs::draw_tile_cracks` — hitReplace THEN hitTile; the per-frame `draw_mining_cracks` overlay (S1-G-1 wave C) folds into this site |
| `restart_sprite_batch(&mut self)` | `:734` | `TileDrawingBase.cs:42-50` |
| `draw_special_tiles_legacy(&mut self)` | `:736` | `special_tiles.rs` + d57's 395/520/471 arms over `LegacyCaches.legacy_points` (d58's chest section already moved to `tr-render::chest_frames`) |
| `draw_object_preview(&mut self, opacity: f32)` | `:740` | `TileObject.DrawPreview` — the placement-preview family |
| `post_draw_pass(&mut self, step: PostDrawStep)` | `:421-439` | `tile_drains::drain_pass_for_post_step(step)` → `Some(pass)` ⇒ `drain_one_pass(...)` (below); `None` arms: `EndBatch`/`NatureAfterAllObjects` = the batch/nature seam, `DrawEntitiesHatRacks`/`DrawEntitiesDisplayDolls` = d57 `te_furniture` doll/hat-rack passes |

**The drain dispatch (g31 `tile_drains.rs`):**

```rust
pub(crate) fn drain_pass_for_post_step(step: tile_pass::PostDrawStep) -> Option<DrainPass>;
pub(crate) fn drain_one_pass(
    pass: DrainPass, world: &World, wind: &Wind, light: &LightMap,
    grass_counter: f32, sunflower_counter: f32, vine_counter: f32,
    rng: &mut UnifiedRandom,              // ONE stream across all ten passes
    seam: &mut impl DrainCellLight,       // d56 bind below
    ported: &mut impl PortedBodyDrain,    // foliage/vines/pylons bind below
    show_invisible: bool, disco: [u8; 3], master_color: f32,
    tile_frame_seed: u64, global_time: f32,
    buckets: &[Vec<(i32, i32)>; 11],      // = &reg.buckets (g30), NO second registry
    sprites: &mut Vec<DrainSprite>, fairy_jars: &mut Vec<DrainFairyJar>,
    rising: &mut Vec<RisingVineSegment>, jar_items: &mut Vec<DrainJarItem>,
    flames: &mut Vec<DrainFlame>,
);
```

* `DrainCellLight::cell_light(&mut self, world, x, y, kind, frame_x,
  frame_y, base: [f32;3], can_do_dust: bool) -> [f32;3]` — d56 binds
  `tr_render::animated_tiles::vision_changer_light` → `light_override`;
  `can_do_dust` is the drain's ALREADY-CONSUMED `_rand.Next(4)==0` flag
  (the binding must NOT re-roll); gated emission → d60.
* `PortedBodyDrain::{tree_point, vine_root, void_lens_point,
  pylon_point, trophy_point}(&mut self, world: &World, x: i32, y: i32)`
  — bind to `foliage.rs` per-marker, `vines.rs` per-root,
  `pylons.rs::{pylon,trophy,void_lens}_sprites` per-anchor (they need
  the renderer + `SpecialEnv`; the impl holds them).
* `is_visible` keep-one: d56's `tr_render` copy at the merge (§1.4);
  g31/d60/pylons rebind.

**The ctor residual (F1, closed):** `tile_ctor.rs` —
`noise_table() -> &'static [f32; 256]`, `linear_noise(x: f32) -> f32`,
`hash/hash2(u32) -> u32`, `dist_to_wandering_circle(pos: (i32,i32),
grid_size: i32, wander_dist: f32, cycles_per_tick: f32, seed: u32,
time_for_visual_effects: f32) -> f32`, `lava_light_a(world, tile_x,
tile_y, time_for_visual_effects) -> f32`. The 717 glowmask arm
(`:1193-1197`) binds `Color(v, v, v, v/2)` from `lava_light_a` at the
f15 glow-seam merge. Census: the static ctor's other draws were
already covered (layer constants = `tile_pass::LAYER_*`, `_zero` = the
origin literal, the instance-ctor registry/`_rand` = g30 + caller
streams).

### 8.3 Merger wiring instructions (the PIXEL binding — §4 made concrete)

Order of operations at the family merge (after d56-d60 + the g-union
are all on main):

1. **Split the fused walk.** `tiles.rs::draw_tiles` keeps its head
   (light compute, liquid pre-pass, walls, behind-tiles NPCs) and its
   tail (NPCs/player/items/UI); the tile walk becomes
   `draw_layer(non_solid)` → `post_draw_tiles(false)` →
   `draw_layer(solid)` → `post_draw_tiles(true)` driven by
   `walls_tiles_npcs_order(detect_creature)`; the render-target capture
   path uses g32 `render_target_tile_order` (SOLID-first, NO post
   passes). The G6 cached-range key is UNTOUCHED (the +4/-2 margins
   are C#'s; the cached range feeds the same inputs).
2. **One `TilePassSeams` impl** (the `ShadowSeams` shape in
   `tile_attach.rs` is the template — replace the counters with the
   real calls per §8.2's bind column). It owns: the g30 registry, g32
   `LegacyCaches` + d58's per-draw chest HashMap, the tiers, the ONE
   `_rand` (the fused pass's `leaf_rng` folds into it — the leaf rolls
   and the drain rolls share the C# `_rand` stream in call order),
   the renderer handles + `SpecialEnv` for the ported drain bodies.
3. **Fold the d-lane hunks into `classify_and_draw_cell`** — the
   mechanical per-hunk script is `docs/research/136-tilepass-d-hunks.md`
   (the d-hunk fold prep: per-lane per-hunk tables, the F0-F4 fold
   order, the C1-C7 conflict pairs, the per-witness re-pin map). Its
   §0 CORRECTS this step's original sketch against the C# re-read:
   the real per-cell order is the `:517` gate (driver-owned) → the
   dd dispatch (`:876`) → d56 vision floors (`:897-963`) → d60
   emission (`:965-968`, BEFORE the gate — it reads the post-vision
   light) → d56 `light_override` (`:969`) → d56 gate + black feed
   (`:970-991`) → d57 Part1 anchors (`:992`) → d58 chest rewrite
   (`:993`, AFTER the gate, not right after the dispatch) → the
   rect/anchor (`:1007-1013`) → d56 `final_light` (`:1019`) → d59
   slope family inside DrawBasicTile (`:1020-1176`) → the glowmask
   table pass (`:1178-1342`, the 717 bind = research/136 §3) → f15's
   record bind (`:1343-1403`). d58/d59's WIRED hunks MOVE verbatim at
   the split (F0) — never re-derived. The inline grass/vine/sunflower
   arms DROP (the drains own them); the inline 192-leaf arm DROPS
   onto d60's battery arm with g68's foliage.rs leaf body as the
   keep-one (research/136 C2).
4. **Stream order is the hard invariant**: ONE `UnifiedRandom` through
   classify (the per-cell `Next(4)` dust rolls) then the ten nature
   passes in `POST_DRAW_NON_SOLID` order; `can_do_dust` flags are
   passed, never re-rolled. Prove byte-equality of the drain plans
   against the pre-split witnesses (the g31 rand-stream witness is
   the template).
5. **A/B the gate**: `TR_TILEPASS` flips from shadow (plans only) to
   the pixel path; unset keeps the fused loop until the split is
   proven. Deterministic screenshots (`TR_SHOT`, zoom 1.0/1.5/2.0)
   must be SHA256-equal across two runs of each arm; the shadow's
   `TILEPASS shadow:` log line is the plan-count baseline.
6. **Drop the shadow** once the pixel path is proven (the
   `tile_attach.rs` module + the env pin + the tiles.rs block are the
   scaffolding; the witnesses' seam-order assertions rebind to the
   real impl).

### 8.4 Attach-phase witness census

F1 `tile_ctor.rs`: 6 `#[test]` (the seed-0 noise table golden bits,
LinearNoise floor/wrap + golden bits, the hash vectors, the
wandering-circle golden distances + the 24-bit mask, the cubic
SmoothStep, the LavaLightA bands). F2 `tile_attach.rs`: 4 `#[test]`
(the four-call frame order over the real binds, the registry→drain
dispatch, plan determinism, the gated attach-site position + RNG
separation). All WRITTEN NOT RUN (code-only). Union g-family total
70 (§7) + 10 attach = 80 on this branch.

### 8.5 The drip seam (the g68 co-lane, requested)

g68's deferred-4 (`07396db1c`, tip `b20480d72`) ported the ONE
unclaimed emission body — `EmitLiquidDrops` (`TileDrawing.cs:7594-7674`,
the vine-tip/stalactite/shimmer-candle drips) — as
`foliage.rs::emit_liquid_drops`, UNWIRED with 3 witnesses. The call
site is the `Draw` switch `:688-694` (kinds 373/374/375/461/709 →
emit then `continue`), which g30 records as
`CellClass::LiquidDrops`/`liquid_drop_cells`. **The pixel bind:**
`classify_and_draw_cell`'s LiquidDrops arm calls
`foliage::emit_liquid_drops(world, y, x, kind, water_style,
zone_corrupt, zone_crimson, zone_hallow, rand, main_rand, gore)`
DIRECTLY per cell — the C# emits INSIDE the switch, so the
`Next(num*2)` gate draws on the ONE `_rand` in row-major cell order
BEFORE the `continue`; draining the recorded cells later would
re-order the stream (research/136 C5). The wet-tile refusal precedes
the roll (no draw); the anti-cluster box scans the 600 gore slots;
the type pick is water-style then the kind/zone overrides (last zone
wins). The recorded cells stay as the shadow-plan census input. The
g68 deferred-2 cadence contract binds this seam too (emission out of
the cached range, the persistent `_rand`, the `DrawnThisUpdate`
mechanism — never widen G6). Full tables + the 717 glowmask
bind-contract (f15 co-lane): `docs/research/136-tilepass-d-hunks.md`.
