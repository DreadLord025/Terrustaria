# UNION-CONTRACT — the W4-g73/g74/g75 waterfall family merge

**Status: FINAL UNION on `task/W4-g75-honeyfall-draw` (the union-fold merge
`9bbc3e1d`, two parents: g75 residuals tip `2997a58c` + g73 union tip
`9a15f3d7`). The merger keeps THIS branch and drops the waterfall modules
of g73 (`task/W4-g73-waterfall-draw`) and g74
(`task/W4-g74-lavafall-draw`) wholesale.** This document is the contract
that makes the family merge mechanical: every semantic decision is already
resolved in code, with witnesses.

## Why g75 is canonical (the union-fold audit)

The g73 union (`06024d3b` + contract `9a15f3d7`) carried g75's dual-triple
shape but NOT the g75 RESIDUALS lane (`de0cb670`/`1d536d1b`/`2997a58c`).
The fold verified, per unique:

| g75 residuals unique | In g73 union? | Resolution in the final union |
| --- | --- | --- |
| `sound::liquid_ambience` (the `Main.Ambience` triple consumer: `(int)` casts, camera-centre stop arm, FIXED 960 pan denominator, out-of-range `None`) + 5 witnesses | NO (g73 kept the inline attenuation math with the window-width pan) | kept ours (`crates/tr-app/src/sound.rs`, `app/frame.rs` consumer) |
| `audio.rs` waterfall-loop re-pitch `2^-0.2` (`:1268`) / lavafall `-0f` (`:1298`) | NO | kept ours |
| Type-25 shimmer-fall draw arm (`WaterfallManager.cs:969-985`): `WaterfallDraw.cell`/`opacity`, the walk's `quad_mark` fill, `render/liquids.rs::draw_shimmer_fall`, the `render/tiles.rs` texture-25 dispatch + 5 shimmer witnesses | NO | kept ours |
| Sand-fall witnesses (`sand_fountain_tile_460_drops_below`, `sand_fall_draws_one_sheet_with_the_even_column_lead_and_fade`) | NO | kept ours |
| `:1120` `_shouldShowInvisibleBlocksAndWalls` latch (`set_show_invisible`, fed from `LightParams::draw_invisible_walls`) + the echo/glass-wall witness | NO (g73 used a `show_invisible` draw PARAMETER) | kept ours — the latch is the verbatim C# static-field shape (`WaterfallManager.cs:74`/`:1120`); the parameter was the lane-scope approximation |
| `an_actuated_dry_side_still_admits_the_spout` / `an_actuated_stone_in_the_column_does_not_stop_the_walk` witnesses | NO (g73 had the unit-level `an_actuated_tile_is_not_solid` only) | kept ours AND folded g73's unit witness back |
| g73's `get_alpha_wall_gate_matches_the_csharp`, `an_actuated_tile_is_not_solid` (draw.rs) + `an_empty_pass_stores_the_load_reset_triple`, `the_last_active_pass_overwrites_the_waterfall_triple`, `the_scan_visits_cells_the_quad_list_skips`, `slope_entry_and_exit_draw_the_transition_sheets`, `the_rain_arm_culls_against_the_raw_screen` + the `water_spout_world` fixture (mod.rs) | (g73-only) | folded into g75's modules in `9bbc3e1d`, adapted to the latch-form `draw` signature; the explicit `!tile3.inactive()` half restored on the `:555` break |
| g73's gates/foam/cull semantics (`:1037` wall gate, `nactive()`, `:749` white propagation, `:381` raw-screen cull) | YES | already present in g75's residuals code — verified equivalent by inspection, ours kept |

## The three lanes and what each contributed

| Lane | Branch tip | Contribution |
| --- | --- | --- |
| W4-g73 waterfall | `task/W4-g73-waterfall-draw` @ `efdf71f4` (pre-union) | `GetAlpha`'s `:1037` invisible-wall gate (`show_invisible` param, `TR_SHOW_INVISIBLE`), the `nactive()` actuated halves (`:555` break + `WorldGen.SolidTile` `WorldGen.cs:69957-69974`), the `:749` slope-entry white-foam `color5` propagation, the raw-screen rain cull (`:381`, `ScreenInfo`), the water-style ambience scan |
| W4-g74 lavafall | `task/W4-g74-lavafall-draw` @ `cd462c1a` | The lavafall ambience scan (`:333-337` locals, `:685-707` kind-1 arm incl. the verbatim `:702-705` `num8 = num35` Y quirk, `:963-965` store), the `drewLava` gates (`:354`/`:368`/`:531` + `Draw()`'s `:1125` reset), 6 lava witnesses; `cd462c1a` fixed the glow-floor fixture to a 140-row world so the fall reaches `GetAlpha`'s k>90 fade |
| W4-g75 honeyfall | `task/W4-g75-honeyfall-draw` @ `2e8b8722` (stacked on g74) | The dual-triple `(AmbientScan, AmbientScan)` return shape, the verbatim kind-14 honey exclusion (`:708` — honey feeds NEITHER ambience triple), the `AmbientScan` struct + `Default` load reset, the fixed g74 spout witnesses (the 51-cell LEFT-bend walk: half-brick head cannot fall straight, `:609`'s `!tile3.halfBrick()` gate + `:625-634` liquid-side bend ⇒ cells (10,10), (9,10), (9,11..59)), 6 honey witnesses |
| W4-g75 FALLS residuals | `task/W4-g75-honeyfall-draw` @ `2997a58c` (stacked on the honeyfall lane) | The full-file `WaterfallManager.cs` census: the type-25 shimmer-fall draw arm (`:969-985` — `cell`/`opacity` on every quad, `render/liquids.rs::draw_shimmer_fall`, the tiles-pass dispatch), the `:1037` invisibleWall/wall-318 gate re-derivation + the `:1120` `set_show_invisible` latch, `nactive()` in `solid_tile`/the spout scan, the `:749` `color5` mutation, `sound::liquid_ambience` + the `audio.rs` re-pitch, the sand-fall witnesses, the echo/glass-wall + actuated-walk witnesses (15 total) |

The PRE-RESIDUALS g74/g75 lanes lacked the g73 gates/foam/cull fixes — their
`draw.rs` regressed `get_alpha` to `wall != 0 && wall != 318` (no
invisible-sight arm), dropped `!t.inactive()` from `solid_tile` and the
`:555` break, kept the `:749` foam white draw-local (no propagation), and
their `frame/draw.rs` fed `false` for `show_invisible`. The g75 RESIDUALS
census independently re-derived ALL of those gates (with the latch form
instead of g73's parameter); the g73 union then folded the dual-triple
shape onto its own parameter form. The final union keeps the residuals'
latch form and folds g73's extra witnesses back — see the audit table above.

## Merger instructions (mechanical)

The family merge is ALREADY DONE inside `task/W4-g75-honeyfall-draw`
(`9bbc3e1d` merges `origin/task/W4-g73-waterfall-draw` `9a15f3d7` into
`2997a58c`). The merger only lands this branch on main; the resolutions
below record what the fold did (never `-X ours/theirs` wholesale — this
was a per-file, per-hunk union):

| Path | Resolution (already applied in `9bbc3e1d`) |
| --- | --- |
| `crates/tr-core/src/waterfall/draw.rs` | **g75's version + g73's two unit witnesses folded back.** The dual-triple walk with the residuals' `quad_mark` cell/opacity fill, the `:1037` gate reading `self.show_invisible_blocks_and_walls`, the `nactive()` halves (incl. the explicit `!tile3.inactive()` on the `:555` break), `color5` white propagation, raw-screen rain cull, `drew_lava` gates. Signature: `draw_waterfall(..., screen, drew_lava: bool, out, lights) -> (AmbientScan, AmbientScan)` — NO `show_invisible` parameter (the latch replaces it). |
| `crates/tr-core/src/waterfall/mod.rs` | **g75's version + g73's `water_spout_world` fixture and 5 witnesses folded back.** `AmbientScan`, `ScreenInfo`, `set_show_invisible`, `draw(..., screen, drew_lava: bool) -> (Vec<DynamicLight>, AmbientScan, AmbientScan)`; the `:1125` reset shadows `drew_lava` to false inside `draw`. Lane witness set: 34 tests (the 19 union + the 15 residuals; see the inventory). |
| `crates/tr-app/src/app/frame/draw.rs` | **g75's waterfall block** (~line 647): `waterfall.set_show_invisible(params.draw_invisible_walls)` then `waterfall.draw(..., screen_info, false)` destructures `(lights, waterfall_scan, lavafall_scan)`; both `App` triples stored from the scans; the quad-based lavafall approximation is DELETED. The g73 union's only hunks in this file were the same waterfall block — nothing else lost. |
| `crates/tr-app/src/app/frame.rs`, `audio.rs`, `sound.rs`, `render/liquids.rs`, `render/tiles.rs` | **g75's residuals versions** (auto-merged clean; g73 never touched them): the `sound::liquid_ambience` consumer, the `2^-0.2` re-pitch, `draw_shimmer_fall` + the texture-25 dispatch. |
| `crates/tr-app/src/app/mod.rs` | g75's `ambient_waterfall`/`ambient_lavafall` field docs (the corrected `Main.cs:2201-2205`/`:2207-2211` citations — identical to g73's; auto-merged). Fields/types unchanged. |
| `anchors.txt` | ALL THREE scan rows (`let mut num = 0.0f32;` g73, `let mut num6 = 0.0f32;` g74, `let mut num4 = -1i32;` g75 — all markers exist in the final `draw.rs`) + the residuals rows (`pub cell: [i32; 2]`, `pub fn set_show_invisible`, `pub fn liquid_ambience`, `pub(crate) fn draw_shimmer_fall`). |
| `docs/ANALYSIS.md` | The "Waterfall overlay" section carries the union scan/gate paragraphs (latch-form corrected) + the honeyfall and residuals lane paragraphs (34 witnesses). |
| `docs/PORT_MAP.md` | One waterfall/spouts row covering all three lanes + the residuals, naming this branch canonical. |

**Drop entirely:** g73's and g74's separate `crates/tr-core/src/waterfall/
{draw,mod}.rs` versions — every unique semantic they carry (dual-triple
return, lava scan + quirk, drewLava gates, honey exclusion, fixed 51-cell
walk fixtures, the 140-row glow fixture, the gates/foam/cull, all 19 union
witnesses) is already in this union. Do NOT re-apply the pre-residuals
`get_alpha`/`solid_tile`/foam simplifications: those were lane-scope
regressions against the C# (`WaterfallManager.cs:1037`,
`WorldGen.cs:69957-69974`, `WaterfallManager.cs:749/:945`). Do NOT
re-introduce g73's `show_invisible` draw parameter: the `:1120` latch is
the verbatim shape and the app already feeds it.

## Semantic decisions locked by the union (do not re-litigate)

1. **The invisible-sight latch + the `drew_lava` parameter.** The g73
   union's `show_invisible` draw parameter is SUPERSEDED by the residuals'
   `WaterfallManager::set_show_invisible` — the verbatim `:1120` head latch
   into `_shouldShowInvisibleBlocksAndWalls` (`:74`), read by `GetAlpha`'s
   default arm. The app calls it with `params.draw_invisible_walls`
   (`Main.cs:58023-58030`, `TR_SHOW_INVISIBLE`) before every `draw`.
   `drew_lava` (g74/g75) stays a parameter; the `:1125` reset makes it
   false for every pass inside `draw` (`let _entry_drew_lava = drew_lava;
   let drew_lava = false;`), so the gates are structurally present but
   behaviourally inert — exactly the C# order. The app passes `false`.
2. **`GetAlpha` keeps the full wall gate**: `wall != 0 && (show_invisible
   || (wall != 318 && !world.invisible_wall(x, y)))` OR below-surface, both
   dimming to `0.6 * alpha`; typed falls 1/14/25 bypass (1.0/0.8/0.75).
3. **`solid_tile` + the `:555` break keep `nactive()`** (`active &&
   !inactive`): actuated tiles never stop a fall.
4. **The `:749` foam sets `color5 = Color.White` and PROPAGATES** to the
   cell's later draws and the next cell's `color4` (`:945`) — not a
   draw-local.
5. **Rain cull (`:381`) reads raw `Main.screenPosition`/`screenWidth`**
   (float divides, ±20-tile margins) via `ScreenInfo`, not the find-pass
   scan rectangle.
6. **Both ambience Y quirks verbatim**: water arm tests `num39`, stores
   `num38` into `num3` (`:725-728`); lava arm tests `num36`, stores `num35`
   into `num8` (`:702-705`).
7. **Honey (kind 14) feeds NEITHER triple** (`:708` exclusion list
   1/14/25/11/12/22; fountain 26 IS included in the water arm).
8. **Return order `(water, lava)`** = the C# store order `:960-962` then
   `:963-965`. Last active pass overwrites both; no active pass leaves
   `AmbientScan::default()` = the `Main.cs:67230-67234` load reset.
9. **Spout fixtures use the FIXED 51-cell left-bend walk** (g75's
   re-derivation of g74's): cells (10,10), (9,10), (9,11..59) in the 80×60
   test world. The glow-floor fixture is the 140-row world (`cd462c1a`).
10. **Every quad carries the walked cell + `GetAlpha` opacity** (the
    `DrawWaterfall(int, ...)` overload's `(x, y, opacity)` arguments,
    `WaterfallManager.cs:969`): the walk fills them per step before the
    cell advances (`quad_mark`). The type-25 consumer
    (`render/liquids.rs::draw_shimmer_fall`) DISCARDS the stylized tint
    and rebuilds the `SetShimmerVertexColors` base + Y42 sparkle pair from
    them; the other consumers ignore the fields.
11. **The liquid-loop consumer is `sound::liquid_ambience`** (pure, per
    channel): `(int)` truncation of the triple (`Main.cs:13477`/`:13500`),
    the zero-strength camera-centre stop arm (`:13484`/`:13506`), the
    FIXED `MaxWorldViewSize.X * 0.5 = 960` pan denominator
    (`Main.cs:66714` — NOT the window width), the out-of-range `None`
    (the live loop keeps its last volume, `LegacySoundPlayer.cs:392-397`),
    the slider/quiet/menu/`(1 - shimmerAlpha)` scaling. `audio.rs`
    re-pitches channel 0 at `2^-0.2` (`:1268`) and channel 1 at `2^0`
    (`:1298`) on both the update and creation branches. The
    `ambientLava` blend (`Main.cs:13488-13497`) stays unported (the
    `Main.ambientLava*` fields ride the load reset).

## Complete witness inventory (34 lane witnesses; written, NOT run — code-only)

### The 19 union witnesses (g73's 8 + g74's fixed 6 + g75's 6, one shared name)

g73's 8 (adapted to the dual return AND the latch-form `draw` signature;
all present in the final union): `waterfall_scan_sums_strength_and_
tracks_the_nearest_cell` (merged with g75's same-named re-derivation),
`an_empty_pass_stores_the_load_reset_triple` (asserts BOTH triples),
`the_last_active_pass_overwrites_the_waterfall_triple`,
`the_scan_visits_cells_the_quad_list_skips`,
`get_alpha_wall_gate_matches_the_csharp` (draw.rs),
`an_actuated_tile_is_not_solid` (draw.rs),
`slope_entry_and_exit_draw_the_transition_sheets`,
`the_rain_arm_culls_against_the_raw_screen`.

g74's fixed 6: `lavafall_scan_sums_strength_and_tracks_the_nearest_cell`,
`water_falls_leave_the_lavafall_triple_at_the_reset`,
`the_last_active_pass_overwrites_the_lavafall_triple`,
`lava_frames_ride_the_slow_counter`,
`lava_quads_carry_the_190_self_glow_floor` (140-row fixture),
`lava_glow_flickers_with_mouse_text_color`.

g75's 6: `honey_spout_uses_type_fourteen`,
`honey_frames_ride_the_slow_counter`,
`honey_alpha_is_flat_point_eight_without_a_glow_floor`,
`honey_falls_emit_no_light_and_feed_neither_ambience_scan`,
`waterfall_scan_y_arm_keeps_the_num3_equals_num38_quirk`
(+ the shared `waterfall_scan_sums...` counted above).

### The 15 FALLS RESIDUALS witnesses (g75-only, kept by the fold)

Shimmer census (`crates/tr-core/src/waterfall/mod.rs`, 5):
`shimmer_spout_uses_type_twenty_five_and_honey_outranks_it`,
`shimmer_frames_ride_the_slow_counter`,
`shimmer_alpha_is_flat_point_seven_five`,
`shimmer_quads_carry_the_walked_cell_and_opacity`,
`shimmer_falls_emit_light_but_feed_neither_ambience_scan`.

Sand + gates census (`crates/tr-core/src/waterfall/mod.rs`, 5):
`sand_fountain_tile_460_drops_below`,
`sand_fall_draws_one_sheet_with_the_even_column_lead_and_fade`,
`echo_and_glass_walls_skip_the_get_alpha_damp_until_the_latch` (the
`:1120` latch witness — supersedes g73's parameter-form gate test at the
integration level; the unit-level `get_alpha_wall_gate_matches_the_csharp`
is kept too),
`an_actuated_dry_side_still_admits_the_spout`,
`an_actuated_stone_in_the_column_does_not_stop_the_walk`.

Ambience consumer (`crates/tr-app/src/sound.rs`, 5):
`liquid_ambience_plays_the_triple_with_int_truncation`,
`liquid_ambience_pan_rides_the_fixed_1920_view`,
`liquid_ambience_zero_strength_stops_at_the_camera`,
`liquid_ambience_out_of_range_leaves_the_loop_untouched`,
`liquid_ambience_scales_with_the_slider_menu_and_shimmer`.

Plus the pre-existing suite (`try_sparkling_grants_deterministically_for_
a_seed`, the 600-frame determinism test now comparing BOTH scans, the
spout-detection/lava-kind/tile-196/frame-counter/quad-emission tests, etc.).

## Final sweep (2026-11-21, code-only)

The post-union full-file re-census of `WaterfallManager.cs` (1182 lines) against
this branch. Verdict: **every spawn/draw/frame/light arm is present**; one
colour-source bug fixed, two census witnesses added, one residual recorded.

### Census results (C# arm → union location)

| C# arm | Lines | Union location | Verdict |
| --- | --- | --- | --- |
| `CheckForWaterfall` | 95-105 | `mod.rs::check_for_waterfall` | PORTED, **UNWIRED** (the `TileDrawing.cs:3858` behind-water refusal — research/09 task seed; the `inActive()` half belongs to g80) |
| `FindWaterfalls` half-brick spout + lava/honey/shimmer precedence | 157-203 | `mod.rs:391-414` | PRESENT (verbatim `:179` gate, `:182-197` else-if chain) |
| fountain 196 → type 11 | 204-219 | `mod.rs:415-425` | PRESENT (`waterfall_tile_196_drops_below`) |
| fountain 460 → type 22 | 220-235 | `mod.rs:426-436` | PRESENT (`sand_fountain_tile_460_drops_below`) |
| fountain 717 → type 26 | 236-251 | `mod.rs:437-447` | PRESENT — **witness added** (`lava_fountain_tile_717_drops_below`) |
| `UpdateFrame` 6 counters | 257-323 | `mod.rs::update_tick` | PRESENT (regular >2/>15, slow >6/>15, rain >0/>2, lava-rain ==1/==3/>3, snow >3/>7) |
| typed falls 1/14/25 slow-frame + drewLava/stopAtStep gate | 352-359 | `draw.rs:71-76` | PRESENT |
| rain arms 11/22/26 (cull, frame tables, two-sheet pairs, fade, liquid trim, walk) | 364-525 | `draw.rs:77-175` + `rain_frames` | PRESENT — **717 draw witness added** (`lava_fountain_draws_the_two_sheet_pair_and_lights_every_cell`: 27-bg 0.4 / 26-fg 0.9, white-127 tint, 3/2 lead, AddLight 26, `:708` water-ambience inclusion) |
| style 0 → Style, disco 2 gate | 527-537 | `draw.rs:176-188` | PRESENT |
| the walk (solid break, bends, num34 conversions 160→2 / 262-268→15-21, all quad arms, 189/196 range extension, k=1000 exits) | 539-958 | `draw.rs:190-754` | PRESENT (incl. `:749` white propagation, `:945` color4 carry) |
| ambience scans + stores | 685-729, 960-965 | `draw.rs:342-399, 756-769` | PRESENT (both Y quirks verbatim) |
| `DrawWaterfall(int,...)` type-25 overload | 969-985 | `quad_mark` fill + `render/liquids.rs::draw_shimmer_fall` | PRESENT |
| `StylizeColor` cases 1/2/15-21 | 987-1028 | `draw.rs::stylize_color` | PRESENT — **case 2 colour source FIXED** (see below) |
| `GetAlpha` | 1030-1044 | `draw.rs::get_alpha` | PRESENT (`:1037` latch gate) |
| `TrySparkling` | 1046-1065 | `draw.rs::try_sparkling` | PRESENT (frame-seeded stream, documented) |
| `AddLight` cases 1/26/2/15-21/25 | 1067-1116 | `draw.rs::add_light` | PRESENT |
| `Draw()` head latch + pass table | 1118-1174 | `mod.rs::draw` + `set_show_invisible` | PRESENT (12-pass table verbatim) |

### The fix: the disco fall's colour source

The C# manager carries NO `DiscoR/G/B` state — `StylizeColor` case 2
(`:1010-1012`) and `AddLight` case 2 (`:1084`) read the SHARED `Main.DiscoR/G/B`
statics `Main.DoUpdate_AnimateDiscoRGB` (`Main.cs:19591-19648`) steps. The
union's manager-local cycle copy was out of phase with the app's shared one
(`world_sim::clock::advance_disco`) and initialised `disco_r: 0` against
`Main.Initialize`'s `DiscoR = 255` (`Main.cs:66780`). The sweep removes the
private cycle, adds `set_disco_rgb` (the `set_show_invisible` latch pattern) and
feeds it from `params.disco` in `frame/draw.rs`. Commit `d118f7ba`; witnesses
`the_disco_fall_reads_the_shared_disco_latch`,
`update_tick_does_not_cycle_the_disco_rgb`,
`the_waterfall_block_latches_the_shared_disco_statics`.

### Ambience wiring — verified LIVE

`frame.rs:300-335` (the ¼ s `Main.Ambience` tick) feeds both App triples through
`sound::liquid_ambience` into `audio.rs::set_liquid_loop` (both re-pitch
branches, `2^-0.2` / `2^0`). The triples are stored by `frame/draw.rs:689-698`
from the walk's dual `AmbientScan`. Residual: the `ambientLava` blend
(`Main.cs:13488-13497`) stays unported — `Main.ambientLava*` is stored by
`DrawWaters` (`:57934-57936`), outside the waterfall lane.

### Tile-side fountain chain — verified complete

- tile 338 (FireworkFountain) placement: `tileobject/registry.rs` D_338 (1×2,
  bottom anchor), item 2738 (`itemdata.rs`).
- wiring case 338: `wiring.rs:2315` → `firework_fountain` (`:1660`,
  `Wiring.cs:1522-1546` verbatim: 2×2 anchor, SkipWire, `CheckMech(30)`,
  `419 + Next(4)`); the pool-side aiStyle-73 dedupe lives in
  `world_sim/wiring.rs:427-430` (`fountain_anchored`).
- tile 207 (WaterFountain) wiring: `wiring.rs:2544` → `switch_fountain`
  (`:2808`, `WorldGen.cs:51150-51193` ±72 frameY flip + the running-gated
  SkipWire block).
- The proj 419-422 AI arm (`ai_fountain`, aiStyle 73) rides the d8 lane
  (`task/W4-d8-proj-ai-71-80`, `e66b41b0` — not yet in main; not duplicated
  here).

### Residuals (recorded, not ported here)

1. `DrawTile_LiquidBehindTile`'s `:3858` `CheckForWaterfall` refusal arm —
   `check_for_waterfall` is ported but the renderer has no manager handle at
   that site (research/09 task seed; coordinate with g80's `inActive()` half).
2. The `ambientLava` blend (above) — a `DrawWaters` lane concern.
3. `Main.gfxQuality` fixed 1.0 (the documented convergence value, `mod.rs:24-27`).

## Verification the merger must run (this lane was code-only)

```powershell
cargo test --workspace -- --skip migrates_version_2_kinds_to_item_ids
cargo test -p tr-app --test main_anchors
```

Judge only by the `test result: ok` lines (cargo may exit 1 on success).
The final union was compile-checked by inspection only — if the 34
witnesses surface fixture drift, fix the FIXTURE against the C# walk (the
decisions above are locked; the numbers are hand-replays, not run results).
Known inspection notes for the verifier: the folded g73 witnesses were
adapted from the 10-arg parameter-form `draw` to the 9-arg latch form
(the `show_invisible` argument removed; the latch defaults false, which
is what every folded witness passed anyway). The shimmer consumer reads
the generic `lookups.waterfall_uv(texture, source)` (no per-texture rows
needed — the `Waterfall_25` sheet loads with the rest of the family).

---

# ADDENDUM - retire-contract fold-list for g73e (2026-11-21, W4-g75)

**Canonical has moved.** `origin/task/W4-g73-waterfall-draw` tip **82ae8b96**
(the g73 final fold: `eee87f98` merged this branch's union `e30ad7ba` INTO
g73, keeping g73's `liquid_loop_update` superset + scan_view/header-bit/
instant-rescan uniques; contract rewritten, canonical = g73). The sections
ABOVE predate that fold - where they say "canonical = g75" or "drop g73's
waterfall modules", the g73 contract wins; everything else (the semantic
decisions, the latch form, the witness fixtures) carried over unchanged.

**Verified NOT folded** (ancestry checks against 82ae8b96, exit 1 = absent):
this branch's three post-`e30ad7ba` commits are missing from canonical, and
g73's `waterfall/mod.rs` still carries the manager-local `update_disco_rgb`
cycle (the exact bug d118f7ba fixes). g73e folds EXACTLY the list below.

## The fold-list (3 commits, in order)

### 1. `d118f7ba` - disco shared-statics FIX (CRITICAL - behavioural)

Without this fold the canonical port's disco fall (tile 160 -> type 2)
tints from a PRIVATE cycle copy that is out of phase with the app's
`advance_disco` (init 0 vs `Main.Initialize`'s `DiscoR=255`, `Main.cs:66780`)
and double-steps it. The C# manager has NO disco state: `StylizeColor`
case 2 (`WaterfallManager.cs:1010-1012`) and `AddLight` case 2 (`:1084`)
read the SHARED `Main.DiscoR/G/B` that `DoUpdate_AnimateDiscoRGB`
(`Main.cs:19591-19648`) steps.

| File | Hunks to fold |
| --- | --- |
| `crates/tr-core/src/waterfall/mod.rs` | (a) struct-field doc re-point: the `disco_r/g/b` fields read the shared statics, no local cycle (`@@ -141,11`); (b) `new()`/reset comments (`@@ -190,10`); (c) NEW `pub fn set_disco_rgb(&mut self, disco: [u8; 3])` latch beside `set_show_invisible` (`@@ -214,6`); (d) `update_frame` doc: the disco cycle is NOT here (`@@ -237,9`); (e) REMOVE the `self.update_disco_rgb()` call + the whole `fn update_disco_rgb` (~48 lines, `@@ -296,54`); (f) 2 module witnesses `the_disco_fall_reads_the_shared_disco_latch` / `update_tick_does_not_cycle_the_disco_rgb` (`@@ -1864,4`, +74) |
| `crates/tr-app/src/app/frame/draw.rs` | (a) the waterfall block latches `waterfall.set_disco_rgb(params.disco)` beside `set_show_invisible(params.draw_invisible_walls)` (+8, `@@ -666`); (b) NEW `mod w4_g75_final_sweep_tests` with `the_waterfall_block_latches_the_shared_disco_statics` (+23, file tail) |
| `anchors.txt` | +1 row: `crates/tr-core/src/waterfall/mod.rs\|pub fn set_disco_rgb\|...` (file tail) |

### 2. `f4e3071f` - the 717 lava-fountain witnesses (tests only, +70)

| File | Hunks to fold |
| --- | --- |
| `crates/tr-core/src/waterfall/mod.rs` | (a) `lava_fountain_tile_717_drops_below` (+20, `@@ -683` - tile 717 -> type 26 spawn ONE ROW BELOW, `WaterfallManager.cs:236-251`, solid-underneath refusal); (b) `lava_fountain_draws_the_two_sheet_pair_and_lights_every_cell` (+50, `@@ -1745` - texture 27 bg 0.4 / texture 26 fg 0.9, flat white-127 tint `:463`, even-column 3/2 lava-rain frame lead `:398-408`, `AddLight` case 26 per cell `:464`/`:1077-1081`, the `:708` water-ambience inclusion - 26 NOT excluded, water triple only) |

### 3. `2b9ed81f` - the FINAL SWEEP docs (+126/-4, five files)

| File | Hunks to fold |
| --- | --- |
| `docs/UNION-CONTRACT-waterfalls.md` | the "Final sweep" section (+74, after the witness inventory): the full-file re-census table, the disco-fix rationale, the ambience-LIVE verification, the residuals list. g73e re-homes it under ITS contract structure (82ae8b96's 43-witness inventory + this addendum) |
| `docs/ANALYSIS.md` | the sweep paragraph (+41) |
| `docs/PORT_MAP.md` | the waterfall row's status extension (FINAL SWEEP sentence: shared disco latch, 717 witnesses, live consumer, `:3858` residual; 1 line replaced) |
| `docs/research/09-tile-drawing.md` | +row 18 (waterfall overlay PORTED), the `Draw_LiquidBehindTile` row's `:3858` census, +the CheckForWaterfall task seed (+9/-2) |
| `docs/research/32-sound-audio.md` | the two liquid-loop rows re-pointed at the live `sound::liquid_ambience` consumer (`frame.rs:300-335`), `ambientLava` residual (+2/-2) |

## Post-e30ad7ba commits on THIS branch (retire-contract phase, also for the fold)

- The `:3858` `CheckForWaterfall` wiring (`liquid_behind_spout_refused`,
  `render/liquids.rs` + the `tiles.rs` probe feed + witnesses + anchor row).
  g80 af6cd44f owns the `inActive()` half of the same line
  (`liquid_behind_refused`) and EXPLICITLY leaves the half-brick +
  `CheckForWaterfall` clause as a recorded deviation - the two halves are
  complementary, not contested. Merger: the g80 `liquid_behind_refused`
  gate and this probe compose (OR of both refusals); the manager handle
  rides `draw_tiles`' existing `waterfall` parameter, no new plumbing.
- The `sound.rs::liquid_ambience` DELEGATED-STATUS doc note: g73's
  `ambient_sound::liquid_loop_update` (the decision tier - Manhattan
  position pick + strength sum, `Main.cs:13488-13497`) delegates its
  volume walk to this pure fn. **The merger keeps BOTH fns, two-tier** -
  do not collapse or re-derive either. The `ambientLava` residual in the
  fn's doc is superseded UPSTREAM by the g73 tier; the pure fn itself
  stays blend-free.
- This addendum (fold-list + canonical pointer).
