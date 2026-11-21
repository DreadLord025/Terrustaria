# UNION-CONTRACT — the W4-g73/g74/g75 waterfall family merge

**Status: prepared on `task/W4-g73-waterfall-draw`. The merger keeps THIS
branch's waterfall modules and drops the separate `draw.rs`/`mod.rs`
versions of g74 and g75 wholesale.** This document is the contract that
makes the family merge mechanical: every semantic decision is already
resolved in code, with witnesses.

## The three lanes and what each contributed

| Lane | Branch tip | Contribution |
| --- | --- | --- |
| W4-g73 waterfall | `task/W4-g73-waterfall-draw` @ `efdf71f4` (pre-union) | `GetAlpha`'s `:1037` invisible-wall gate (`show_invisible` param, `TR_SHOW_INVISIBLE`), the `nactive()` actuated halves (`:555` break + `WorldGen.SolidTile` `WorldGen.cs:69957-69974`), the `:749` slope-entry white-foam `color5` propagation, the raw-screen rain cull (`:381`, `ScreenInfo`), the water-style ambience scan |
| W4-g74 lavafall | `task/W4-g74-lavafall-draw` @ `cd462c1a` | The lavafall ambience scan (`:333-337` locals, `:685-707` kind-1 arm incl. the verbatim `:702-705` `num8 = num35` Y quirk, `:963-965` store), the `drewLava` gates (`:354`/`:368`/`:531` + `Draw()`'s `:1125` reset), 6 lava witnesses; `cd462c1a` fixed the glow-floor fixture to a 140-row world so the fall reaches `GetAlpha`'s k>90 fade |
| W4-g75 honeyfall | `task/W4-g75-honeyfall-draw` @ `2e8b8722` (stacked on g74) | The dual-triple `(AmbientScan, AmbientScan)` return shape, the verbatim kind-14 honey exclusion (`:708` — honey feeds NEITHER ambience triple), the `AmbientScan` struct + `Default` load reset, the fixed g74 spout witnesses (the 51-cell LEFT-bend walk: half-brick head cannot fall straight, `:609`'s `!tile3.halfBrick()` gate + `:625-634` liquid-side bend ⇒ cells (10,10), (9,10), (9,11..59)), 6 honey witnesses |

g74/g75 LACK the g73 gates/foam/cull fixes — their `draw.rs` regressed
`get_alpha` to `wall != 0 && wall != 318` (no invisible-sight arm), dropped
`!t.inactive()` from `solid_tile` and the `:555` break, kept the `:749`
foam white draw-local (no propagation), and their `frame/draw.rs` fed
`false` for `show_invisible`. The union restores ALL of that while keeping
their dual-triple shape.

## Merger instructions (mechanical)

For each path, take **this branch's version** (`git checkout
task/W4-g73-waterfall-draw -- <path>` inside the merge resolution — never
`-X ours/theirs` wholesale; this is a per-file, pre-resolved union):

| Path | Resolution |
| --- | --- |
| `crates/tr-core/src/waterfall/draw.rs` | **THIS branch.** Union of g75's dual-triple walk + g73's `show_invisible` gate, `nactive()` halves, `color5` white propagation, raw-screen rain cull, plus g74/g75's `drew_lava` gates. Signature: `draw_waterfall(..., screen, show_invisible: bool, drew_lava: bool, out, lights) -> (AmbientScan, AmbientScan)`. |
| `crates/tr-core/src/waterfall/mod.rs` | **THIS branch.** `AmbientScan` (g75's shape), `ScreenInfo`, `draw(..., screen, show_invisible: bool, drew_lava: bool) -> (Vec<DynamicLight>, AmbientScan, AmbientScan)`; the `:1125` reset shadows `drew_lava` to false inside `draw`. Union witness set: 19 tests (g73's 8 + g74's fixed 6 + g75's 6, one shared `waterfall_scan_sums...` name merged). |
| `crates/tr-app/src/app/frame/draw.rs` | **THIS branch's waterfall block** (~line 647): `waterfall.draw(..., crate::env::show_invisible(), false)` destructures `(lights, waterfall_scan, lavafall_scan)`; both `App` triples stored from the scans; the quad-based lavafall approximation is DELETED. If other lanes touched this hot file, keep their non-waterfall hunks and take this waterfall block. |
| `crates/tr-app/src/app/mod.rs` | Take this branch's `ambient_waterfall`/`ambient_lavafall` field docs (g75's corrected `Main.cs:2201-2205`/`:2207-2211` citations). Fields/types unchanged. |
| `anchors.txt` | Keep ALL THREE rows (`let mut num = 0.0f32;` g73, `let mut num6 = 0.0f32;` g74, `let mut num4 = -1i32;` g75) — this branch already carries all three, all markers exist in the union `draw.rs`. Drop g74/g75's own variants of these rows if duplicated. |
| `docs/ANALYSIS.md` | This branch's "Waterfall overlay" section describes the union (both scans, both quirks, all gates, 19 witnesses). Drop g74/g75's separate lane paragraphs if they conflict. |
| `docs/PORT_MAP.md` | This branch's waterfall/spouts row covers all three lanes. |

**Drop entirely:** g74's and g75's separate `crates/tr-core/src/waterfall/
{draw,mod}.rs` versions — every unique semantic they carry (dual-triple
return, lava scan + quirk, drewLava gates, honey exclusion, fixed 51-cell
walk fixtures, the 140-row glow fixture) is already in this union. Do NOT
re-apply their `get_alpha`/`solid_tile`/foam simplifications: those were
lane-scope regressions against the C# (`WaterfallManager.cs:1037`,
`WorldGen.cs:69957-69974`, `WaterfallManager.cs:749/:945`).

## Semantic decisions locked by the union (do not re-litigate)

1. **Both param sets survive.** `show_invisible` (g73) AND `drew_lava`
   (g74/g75) are separate parameters on `draw`/`draw_waterfall`. The
   `:1125` reset makes `drew_lava` false for every pass inside `draw`
   (`let _entry_drew_lava = drew_lava; let drew_lava = false;`), so the
   gates are structurally present but behaviourally inert — exactly the
   C# order. The app passes `crate::env::show_invisible()` and `false`.
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

## Union witness set (19; written, NOT run — code-only lanes)

g73's 8 (adapted to the dual return): `waterfall_scan_sums_strength_and_
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

Plus the pre-existing suite (`try_sparkling_grants_deterministically_for_
a_seed`, the 600-frame determinism test now comparing BOTH scans, etc.).

## Verification the merger must run (this lane was code-only)

```powershell
cargo test --workspace -- --skip migrates_version_2_kinds_to_item_ids
cargo test -p tr-app --test main_anchors
```

Judge only by the `test result: ok` lines (cargo may exit 1 on success).
The union was compile-checked by inspection only — if the 19 witnesses
surface fixture drift, fix the FIXTURE against the C# walk (the decisions
above are locked; the numbers are hand-replays, not run results).

## Addendum — the falls-adjacent census (post-union, same branch)

Two residual lanes landed on top of the union (commits `7a408d98` /
`43296a24`), both code-only:

1. **Source detection** (`FindWaterfalls` :100-330 census): the pure-flag
   kind probes (`Tile.cs:345-382` — no `liquid > 0` guard), the truncating
   `scan_view` (:119-122) replacing the caller's floor/ceil ±1 derivation,
   and the `instantBGTransitionCounter == 10` forced rescan
   (`Main.cs:61841`, Spawn arms `Player.cs:37569`). Census finding: there
   is NO type-12 source in the C# — 12 is the rain arm's background
   texture (:486); the landed type set 0/1/14/25/11/22/26 is complete.
2. **Ambience consumers** (`Main.Ambience` :13466-13509): the verbatim
   `liquid_loop_update` decision half (int casts, ambientLava Manhattan
   pick + strength sum, the 960 pan divisor, the out-of-range
   keep-last-volume arm, the case-34 −0.2 pitch) + live re-pan gains in
   `audio.rs`. `App::ambient_lava` rides the load reset — its only C#
   writer is the retro-only `oldDrawWater` scan, unported by design.

The merger keeps these too (they are part of THIS branch's versions of
`waterfall/mod.rs`, `ambient_sound.rs`, `audio.rs`, `app/frame.rs`,
`app/mod.rs`, `app/flow.rs`, `world_sim/liquids.rs`,
`world_sim/player.rs`, `recall_teleport.rs`). 9 new witnesses, written
not run.
