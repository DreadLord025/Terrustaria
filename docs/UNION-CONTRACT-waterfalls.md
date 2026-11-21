# UNION-CONTRACT — the W4-g73/g74/g75 waterfall family merge (FINAL, retire-sealed)

**Status: FINAL UNION on `task/W4-g73-waterfall-draw` (THIS branch),
RETIRE-SEALED 2026-11-21.** The final falls fold `eee87f98` merges the g75
union tip `e30ad7ba` (itself the union-fold `9bbc3e1d`: g75 residuals
`2997a58c` + the g73 union `9a15f3d7`) into the g73 falls-adjacent tip
`155b56ba`, then keeps g73's faithful superset wherever the two lanes
ported the same C# arm differently. The closeout (`5d444f15`) folded the
g75d sweep; the RETIRE phase sealed the two remaining cross-lane items:
the g24 quit-stop DUP (keep-one adjudication, §-quit below) and the
`CalculateWaterStyle` fountain head arm (the 207-chain's last link,
UNCLAIMED census → ported here).

**The merger takes ONE branch: THIS one.** `task/W4-g75-honeyfall-draw`
(tip `41b64b04`, all six g75d sweep commits folded per-hunk) and
`task/W4-g74-lavafall-draw` (tip `cd462c1a`) are FULLY CONTAINED here —
their waterfall modules, consumers and witnesses are all in this tree.
Drop them wholesale at merge time; do not re-merge them. Every semantic
decision is already resolved in code, with witnesses.

**Retire-phase containment re-verify (2026-11-21, post-closeout):**
`git merge-base --is-ancestor cd462c1ae HEAD` → exit 0 and
`git log cd462c1ae..origin/task/W4-g74-lavafall-draw` → empty (g74 done,
nothing after the glow-fixture fix); `git log
41b64b04..origin/task/W4-g75-honeyfall-draw` → empty (the g75f/g75e
sessions landed NOTHING past the folded fold-list extension — g75 done).
The g75 branch tip's own ancestry check is moot (its commits were folded
per-hunk, not merged — the fold record below is the containment proof).

## §-quit — the g24/g73 quit-stop DUP: keep-one adjudication (RETIRE phase)

Two lanes wired `WorldGen.StopWaterfallAmbienceAudio` (`WorldGen.cs:
6689-6699`, the 34/35 (-1,-1,0) sentinel pair) into `save_and_quit`
(`:6704`): g24's doc-sweep `c3eccb77` (+ the `7ca8b087` dead-code allow)
and this branch's closeout `4e7e3f58`. **Verdict: g24b is the SUPERSET —
its shape is kept, ONE `:6704` call site and ONE stop fn survive.**

| Element | g24 `c3eccb77`+`7ca8b087` | g73 closeout `4e7e3f58` | Keep-one resolution |
| --- | --- | --- | --- |
| `stop_waterfall_ambience` body | direct sink kill + `liquid_ambience_stops` counter | route through `set_liquid_loop(ch, 0.0)` | **g24's** — the closeout's route no-ops HEADLESS (`set_liquid_loop` gates on the decoded cue + output handle BEFORE its stop arm), so its witness could never observe a live stop; the direct kill matches the C# sentinel's draw-free immediacy |
| `save_and_quit` `:6704` call | yes | yes (the DUP) | ONE call site (identical position, after the Menu_Close 11) |
| `StopTrackedSounds` `:6715` | yes (`stop_tracked_sounds`) | no | **g24's** — folded |
| `JustQuit` `:6627-6662` | PORTED (`just_quit` + the F3 World-tab row 15, the `/quit` dispatcher stand-in, `#[allow(dead_code)]`) | "stays unported" (the closeout missed the debug-command caller) | **g24's** — folded; the closeout's deferral note RETIRED |
| Witness support | `menu_accept_style`/`tracked_count_for`/`insert_test_cue`/`count_style_entries` | none | **g24's** — folded (test-only readers carry `#[allow(dead_code)]` on this branch; g24's `SecretSeed.Enable` production caller stays on the g24 branch) |
| Witnesses | `w4_g24_the_ambience_stop_pair_and_the_tracked_pool_drain` + `w4_g24_the_quit_paths_stop_the_ambience_and_the_tracked_pool` + `w4_g24_the_quit_row_runs_just_quit` | `the_quit_stop_kills_both_liquid_loop_channels` | **g24's three** — the closeout's witness retired INTO the first (its dead-slot/idempotence assertions carried over) |

Fold correction (C#-verified): g24's `s1_e_29_the_world_tab_keeps_its_
fifteen_rows` witness was left STALE on the g24 branch (still asserting
15 rows against its own 16-row panel) — this branch's fold renames it
`..._sixteen_rows` and pins row 15 = "Quit (no save)".

**Merger instruction for the g24 branch**: its `stop_waterfall_ambience`/
`just_quit`/`save_and_quit`/`debug_panel` hunks are ALREADY HERE with
identical semantics — at the g24 merge keep THIS branch's copies (or
g24's, byte-equivalent bodies); never land BOTH `:6704` call sites. The
`menu_accept_style` fn merges cleanly (g24 adds the `SecretSeed.Enable`
caller; drop this branch's `#[allow(dead_code)]` when that caller lands).
Re-fetch at retire-push time: g24 advanced past `7ca8b087` to `a0d046b7`
(the FINAL phase — `1dfd8d4d`/`c48f8005`/`dbb09fd5`/`60508137`/`5a770104`/
`a0d046b7`); the diff touches NONE of the quit-stop files (`audio.rs`,
`saveglue.rs`, `debug_panel.rs`, `tracked_sound.rs` all untouched), so the
adjudication above stands unchanged.

## §-fountain — the CalculateWaterStyle head arm (RETIRE phase)

The closeout's noted biome-lane gap is CLOSED on this branch:
`Main.cs:57390-57393`'s `ActiveFountainColor >= 0` head arm ported as
`biome::calculate_water_style_with_fountain`, consumers wired
(`draw_tiles` + the `:57475` liquidAlpha ramp + the legacy-light style +
the `Rain::water_style` latch — `Rain.cs:205` reads the STORED static).
Claim census: g46/g47's `biome.rs` byte-identical to the pre-port version
(empty diff, both branches), the same "no consumer yet" ANALYSIS note, no
biome lane (g40-g44/d129) touches the file — UNCLAIMED, so the falls lane
took it. `calculate_water_style` (no fountain) stays the
`ignoreFountains: true` capture arm (`CaptureBiome.cs:158`). 4 witnesses;
ANALYSIS "The fountain head arm". The tile-207 chain is now COMPLETE:
place → wire trip (`SwitchFountain`) → scene scan → head arm → recolour.

## Cross-note to g75d (the g75-branch final sweep)

g75d runs its final sweep on `task/W4-g75-honeyfall-draw`, whose contract
(`e30ad7ba`'s version of this file) claims g75 as canonical. **That claim
is SUPERSEDED by this document.** The g75 union `e30ad7ba` predates the
g73 falls-adjacent uniques (`7a408d98` source detection, `43296a24`
ambience consumers, census `155b56ba`), so it cannot be canonical: it
lacks the `scan_view` truncation, the pure header-bit kind probes, the
instant-BG forced rescan, the `ambientLava` Manhattan pick + strength sum,
and the live re-pan / create-arm-no-pitch fidelity. If g75d lands further
commits on the g75 branch after `e30ad7ba`, they must be folded HERE (or
re-derived here) before the family merge — the merger still takes only
this branch. Nothing on g75 after `e30ad7ba` is known to this fold; check
`git log e30ad7ba..origin/task/W4-g75-honeyfall-draw` at merge time.
**Closeout re-check (2026-11-21)**: the first re-fetch found the g75 tip
still `e30ad7ba`; the pre-push re-fetch found the g75d sweep had landed
six commits (`d118f7ba`/`f4e3071f`/`2b9ed81f`/`17161648`/`a3093a66f`/
`d441713d`/`41b64b04`) — ALL folded per-hunk into this branch (the disco
fix, the 717 witnesses with two C#-verified corrections, the :3858 spout
wiring, the DELEGATED-STATUS + census docs; see ANALYSIS "The g75d
final-sweep FOLD"). The g75 branch now ends at `41b64b04` and is fully
contained; re-check `git log 41b64b04..origin/task/W4-g75-honeyfall-draw`
at merge time.

## Why THIS branch is canonical (the final-fold audit)

The arm-by-arm comparison of the two liquid-loop consumers against the C#
(`Main.Ambience`, `Main.cs:13466-13509`; `LegacySoundPlayer.PlaySound`
`:360-437` + cases 34/35 `:1249-1307`):

| C# arm | g75 `sound::liquid_ambience` (e30ad7ba) | g73 `ambient_sound::liquid_loop_update` (155b56ba) | Final resolution |
| --- | --- | --- | --- |
| `(int)` PlaySound casts (`:13477`/`:13500`) | yes | yes | both faithful — kept in the shared volume half |
| zero-strength camera-centre stop (`:13484`/`:13506`) | yes (re-play at camera, strength 0 → vol 0) | yes (stop record; stopping a dead slot is a no-op) | equivalent — decision half owns the gate, volume half keeps the camera-centre arm |
| `ambientLava` Manhattan pick + strength SUM (`:13488-13497`) | NO — explicitly unported, caller feeds the lavafall triple alone | yes | **g73 kept** (faithful superset) |
| play gate = lavafall strength alone (`:13498`) | n/a | yes | **g73 kept** |
| FIXED `MaxWorldViewSize.X * 0.5` = 960 pan (`:395`) | yes | yes | shared volume half |
| out-of-range keep-last-volume (`:392-397` → `:415` skip) | yes (`None`) | yes (`volume_pan: None`, playing kept) | shared volume half; the decision half maps `None` to "untouched" |
| slider / QuietAmbientSounds / gameMenu / `(1 - shimmerAlpha)` (`:417-423`/`:1256`/`:1286`) | yes | yes | shared volume half |
| `Pitch = -0.2f` / `-0f` on the RE-VOLUME arm only (`:1268`/`:1298`); create arms set NO pitch (`:1272-1275`/`:1302-1305`) | NO — pinned `2^-0.2` on BOTH branches (documented deviation) | yes (`pitch: Option<f32>`, `None` on create) | **g73 kept** — the g75 deviation is retired |
| live re-pan of the PLAYING instance (`:1267`/`:1297`) | NO — pan baked at creation (documented deviation) | yes (`LiquidPanGains` atomic shared gains + `PannedShared` source) | **g73 kept** — the g75 deviation is retired |
| slot playing guard (`:1264`/`:1294`) | not modelled | yes (`Audio::liquid_loop_playing`) | **g73 kept** |

**Verdict: g73's `liquid_loop_update` is the faithful superset.** The final
code keeps BOTH functions in a two-tier stack (house style — no parallel
implementations): `ambient_sound::liquid_loop_update` owns the
`Main.Ambience` decision arms (pick/sum/gate/stop) and the slot/pitch
bookkeeping, and DELEGATES its per-channel volume walk to the pure
`sound::liquid_ambience` (which keeps its 5 g75 witnesses and its
`anchors.txt` canary row alive). `app/frame.rs` runs the decision half;
`audio.rs::set_liquid_loop(channel, volume, pan, pitch: Option<f32>)` is
g73's 4-arg live-re-pan version.

The g73 source-detection uniques (absent from `e30ad7ba` entirely):

| g73 falls-adjacent unique | C# anchor | Resolution |
| --- | --- | --- |
| `WaterfallManager::scan_view` — four `(int)` TRUNCATING casts of the raw unscaled `screenPosition`/screen size (no zoom divide, no floor/ceil) | `WaterfallManager.cs:119-122` | kept; `world_sim/liquids.rs` feeds it camera − screen/2 |
| pure header-bit kind probes — `any_liquid_kind` drops the invented `liquid > 0` guard | `Tile.cs:345-348/:362-365/:379-382`, `WaterfallManager.cs:182/:186/:190` | kept; g75's shimmer-witness comment corrected to match |
| `instantBGTransitionCounter == 10` forced rescan (`app.bg_instant`) + the Spawn writes | `Main.cs:61841`, `Player.cs:37569`, `Main.cs:62559` | kept (`world_sim/liquids.rs`, `world_sim/player.rs`, `recall_teleport.rs`) |
| `App::ambient_lava` triple at the load reset; all three App triples init (-1, -1, 0) not (0, 0, 0) | `Main.cs:2213-2217`/`:67229-67236` | kept (`app/mod.rs`, `app/flow.rs` test fixture) |
| the retro-only `oldDrawWater` ambientLava scan stays unported (LiquidRenderer path never runs it) | `Main.cs:57615-57634`/`:57932-57936` | SUPERSEDED by the closeout: the scan is PORTED (`tr_core::retro_water::ambient_lava_scan`, Retro/Trippy frames feed `App::ambient_lava`); the retro liquid DRAW halves stay unported |

Everything the g75 union carries that g73's `155b56ba` lacked is folded in
verbatim (the `9bbc3e1d` resolutions survive unchanged): the latch-form
`set_show_invisible` (`WaterfallManager.cs:74`/`:1120`), the type-25
shimmer-fall draw arm (`:969-985`, `WaterfallDraw.cell`/`opacity`,
`render/liquids.rs::draw_shimmer_fall`, the `render/tiles.rs` texture-25
dispatch), the sand-fall witnesses, the echo/glass-wall + actuated-walk
witnesses, the g74 140-row glow fixture, the fixed 51-cell left-bend
spout walk, the honey exclusion, both Y quirks.

## The lanes and what each contributed

| Lane | Branch tip folded | Contribution |
| --- | --- | --- |
| W4-g73 waterfall | `efdf71f4` (via union `9a15f3d7`) | the `:1037` invisible-wall gate, `nactive()` actuated halves (`:555`, `WorldGen.cs:69957-69974`), `:749` white-foam `color5` propagation, raw-screen rain cull (`:381`, `ScreenInfo`), the water-style ambience scan |
| W4-g74 lavafall | `cd462c1a` (stacked into g75) | the lavafall scan (`:333-337`, `:685-707` incl. the `:702-705` `num8 = num35` Y quirk, `:963-965` store), `drewLava` gates (`:354`/`:368`/`:531`, `:1125` reset), 6 lava witnesses, the 140-row glow fixture |
| W4-g75 honeyfall | `2e8b8722` (stacked into g75) | the dual `(AmbientScan, AmbientScan)` return, the verbatim kind-14 honey exclusion (`:708`), `AmbientScan::default()` load reset, the fixed 51-cell LEFT-bend walk fixtures, 6 honey witnesses |
| W4-g75 FALLS residuals | `2997a58c` (via union-fold `9bbc3e1d` + contract `e30ad7ba`) | the type-25 shimmer draw arm + consumer, the `:1120` latch, `sound::liquid_ambience` + 5 witnesses, the sand/echo/actuated witnesses (15 total) |
| W4-g73 falls-adjacent | `7a408d98`/`43296a24`/`155b56ba` (THIS branch) | `scan_view`, header-bit probes, instant-BG forced rescan, `liquid_loop_update` + `LiquidPanGains` live re-pan + create-arm-no-pitch, `App::ambient_lava`, 9 witnesses |

## Merger instructions (mechanical)

The family merge is ALREADY DONE inside `task/W4-g73-waterfall-draw`
(`eee87f98` merges `e30ad7ba` into `155b56ba`). The merger only lands this
branch on main. The resolutions below record what the fold did (never
`-X ours/theirs` wholesale — this was a per-file, per-hunk union):

| Path | Resolution (applied in `eee87f98`) |
| --- | --- |
| `crates/tr-core/src/waterfall/draw.rs` | **g75's version verbatim** (dual-triple walk, `quad_mark` cell/opacity fill, latch-reading `:1037` gate, `nactive()` halves incl. the explicit `!tile3.inactive()` on the `:555` break, `color5` propagation, raw-screen rain cull, `drew_lava` gates, g73's two unit witnesses). |
| `crates/tr-core/src/waterfall/mod.rs` | **g75's version + g73's source-detection uniques**: `scan_view`, the pure-flag `any_liquid_kind`, the forced-flag doc, the 3 source-detection witnesses; all 34 g75-union witnesses kept (the shimmer witness's `liquid > 0` comment corrected to the header-bit fact). |
| `crates/tr-app/src/ambient_sound.rs` | **g73's `liquid_loop_update`**, its volume walk now delegating to `sound::liquid_ambience` (dedupe; behaviour identical — both were the same C# walk). |
| `crates/tr-app/src/sound.rs` | **g75's `liquid_ambience` + 5 witnesses** kept as the shared pure volume half; doc updated — the ambientLava blend is resolved upstream by the decision half. |
| `crates/tr-app/src/audio.rs` | **g73's version**: `liquid_pans`/`LiquidPanGains`/`PannedShared`, 4-arg `set_liquid_loop` (pitch only on the re-volume arm), `liquid_loop_playing`, the live-pan witness. g75's pinned-both-branches speed is DELETED. |
| `crates/tr-app/src/app/frame.rs` | **g73's ¼ s tick**: the int-halved screen centre, `liquid_loop_update` per channel, the stop/untouched arms. g75's inline `liquid_ambience` loop is superseded (the function itself survives as the delegate). |
| `crates/tr-app/src/app/frame/draw.rs`, `render/liquids.rs`, `render/tiles.rs` | **g75's versions verbatim** (the latch feed + dual-scan store; `draw_shimmer_fall`; the texture-25 dispatch). |
| `crates/tr-app/src/app/mod.rs`, `app/flow.rs`, `world_sim/liquids.rs`, `world_sim/player.rs`, `recall_teleport.rs` | **g73's versions** (the `ambient_lava` field + load resets, `scan_view` + `bg_instant == 10` caller, the Spawn writes). |
| `anchors.txt` | ALL rows from both sides: the 3 draw-scan rows, the 4 g75-residuals rows (`pub cell`, `set_show_invisible`, `liquid_ambience`, `draw_shimmer_fall`), the 2 g73 falls-adjacent rows (`scan_view`, `liquid_loop_update`) — 9 waterfall-family rows, all markers present. |
| `docs/ANALYSIS.md`, `docs/PORT_MAP.md` | union of both sides' paragraphs; canonical-branch claims point HERE; witness total 43. |

**Drop entirely:** the waterfall-family versions on
`task/W4-g75-honeyfall-draw` and `task/W4-g74-lavafall-draw` — every unique
semantic they carry is in this union. Do NOT re-apply g75's
pinned-both-branches pitch or baked-at-creation pan (retired deviations,
see the audit table). Do NOT re-introduce the `show_invisible` draw
parameter (the `:1120` latch is the verbatim shape). Do NOT restore the
`liquid > 0` guard in `any_liquid_kind` (an invention — the C# probes are
pure header-bit tests).

## Semantic decisions locked by the union (do not re-litigate)

1. **The invisible-sight latch + the `drew_lava` parameter** — the `:1120`
   `set_show_invisible` latch fed from `LightParams::draw_invisible_walls`
   before every draw; `drew_lava` structurally present, behaviourally
   inert via the `:1125` reset (the app passes `false`).
2. **`GetAlpha` full wall gate**: `wall != 0 && (show_invisible || (wall
   != 318 && !world.invisible_wall(x, y)))` OR below-surface → `0.6 *
   alpha`; typed falls 1/14/25 bypass (1.0/0.8/0.75).
3. **`solid_tile` + the `:555` break keep `nactive()`** — actuated tiles
   never stop a fall.
4. **The `:749` foam sets `color5 = Color.White` and PROPAGATES** (`:945`).
5. **Rain cull (`:381`) reads raw screen values** via `ScreenInfo`.
6. **Both ambience Y quirks verbatim** (`:725-728` water, `:702-705` lava).
7. **Honey (14) feeds NEITHER triple** (`:708`; fountain 26 IS included).
8. **Return order `(water, lava)`**; last active pass wins; no pass leaves
   `AmbientScan::default()` = the load reset.
9. **Spout fixtures use the FIXED 51-cell left-bend walk**; the glow-floor
   fixture is the 140-row world.
10. **Every quad carries the walked cell + opacity** (`quad_mark`); the
    type-25 consumer rebuilds the shimmer base + Y42 sparkle from them.
11. **The liquid-loop consumer stack is TWO-TIER**:
    `ambient_sound::liquid_loop_update` (decision: `(int)` casts, the
    `ambientLava` Manhattan pick + strength sum with the lavafall-strength-
    alone play gate, the stop/keep-last-volume mapping, the slot guard) →
    `sound::liquid_ambience` (volume: the `:376-397` walk, the FIXED 960
    pan, the slider/quiet/menu/shimmer scaling) →
    `audio.rs::set_liquid_loop` (live re-pan via shared atomic gains;
    `2^-0.2`/`2^0` pitch on the RE-VOLUME arm only, create arms keep the
    default rate). The retro-only `oldDrawWater` ambientLava scan is
    PORTED as of the closeout (`tr_core::retro_water`, Retro/Trippy
    frames); in Color/White `App::ambient_lava` rides the load reset —
    structurally the same as a non-retro original.
12. **Source detection**: `scan_view`'s four truncating `(int)` casts
    (`:119-122`, margins inside `find_waterfalls`), the pure header-bit
    kind probes (no liquid-amount guard), the `instantBGTransitionCounter
    == 10` forced rescan (`Main.cs:61841`). No type-12 SOURCE exists —
    12 is the rain background texture (`:486`).

## Complete witness inventory (43 lane witnesses; written, NOT run — code-only)

### The 19 union witnesses (g73's 8 + g74's fixed 6 + g75's 6, one shared name)

g73's 8 (latch-form `draw` signature): `waterfall_scan_sums_strength_and_
tracks_the_nearest_cell` (shared with g75's re-derivation),
`an_empty_pass_stores_the_load_reset_triple`,
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

### The 15 FALLS RESIDUALS witnesses (g75, kept by the fold)

Shimmer census (`crates/tr-core/src/waterfall/mod.rs`, 5):
`shimmer_spout_uses_type_twenty_five_and_honey_outranks_it`,
`shimmer_frames_ride_the_slow_counter`,
`shimmer_alpha_is_flat_point_seven_five`,
`shimmer_quads_carry_the_walked_cell_and_opacity`,
`shimmer_falls_emit_light_but_feed_neither_ambience_scan`.

Sand + gates census (`crates/tr-core/src/waterfall/mod.rs`, 5):
`sand_fountain_tile_460_drops_below`,
`sand_fall_draws_one_sheet_with_the_even_column_lead_and_fade`,
`echo_and_glass_walls_skip_the_get_alpha_damp_until_the_latch`,
`an_actuated_dry_side_still_admits_the_spout`,
`an_actuated_stone_in_the_column_does_not_stop_the_walk`.

Volume half (`crates/tr-app/src/sound.rs`, 5):
`liquid_ambience_plays_the_triple_with_int_truncation`,
`liquid_ambience_pan_rides_the_fixed_1920_view`,
`liquid_ambience_zero_strength_stops_at_the_camera`,
`liquid_ambience_out_of_range_leaves_the_loop_untouched`,
`liquid_ambience_scales_with_the_slider_menu_and_shimmer`.

### The 9 falls-adjacent witnesses (g73, THIS branch's uniques)

Source detection (`crates/tr-core/src/waterfall/mod.rs`, 3):
`a_stale_liquid_flag_classifies_without_any_liquid_amount`,
`the_scan_view_truncates_like_the_csharp_int_casts`,
`the_forced_flag_bypasses_the_thirty_tick_rescan_gate`.

Decision half (`crates/tr-app/src/ambient_sound.rs`, 5):
`the_waterfall_loop_truncates_and_pans_against_the_1920_view`,
`an_out_of_range_position_keeps_the_live_loop_untouched`,
`zero_strength_stops_the_waterfall_loop`,
`the_lavafall_arm_picks_the_nearer_position_and_sums_strength`,
`the_menu_mutes_and_the_shimmer_factor_scales_the_loop`.

Live re-pan (`crates/tr-app/src/audio.rs`, 1):
`the_liquid_loop_pan_gains_update_the_live_source`.

Plus the pre-existing suite (`try_sparkling_grants_deterministically_for_
a_seed`, the 600-frame determinism test comparing BOTH scans, the
spout-detection/lava-kind/tile-196/frame-counter/quad-emission tests, etc.).

### The 9 CLOSEOUT witnesses (W4-g73 closeout, 2026-11-21 — written, NOT run)

Frame tables (`crates/tr-core/src/waterfall/mod.rs`, 4 → 2 after the g75d
fold's keep-one): `the_fountain_frame_counters_ride_their_own_rates`,
`rain_fountain_draws_two_lit_sheets_with_the_even_column_lead` survive;
the closeout's `lava_cloud_tile_717_drops_below` /
`lava_fountain_draws_two_white_sheets_and_lights_every_cell` were replaced
by the sweep's `lava_fountain_tile_717_drops_below` /
`lava_fountain_draws_the_two_sheet_pair_and_lights_every_cell` (counted
below; the unique tail-fade/quad-count assertions carried over).

Quit stop (`crates/tr-app/src/audio.rs`, 1 — RETIRED into the g24
keep-one's superset witness, see the retire inventory below):
~~`the_quit_stop_kills_both_liquid_loop_channels`~~ →
`w4_g24_the_ambience_stop_pair_and_the_tracked_pool_drain`.

Retro scan (`crates/tr-core/src/retro_water.rs`, 3):
`one_lit_lava_cell_sums_full_strength_at_the_screen_centre`,
`the_y_slot_keeps_the_num3_equals_num13_quirk`,
`the_dry_covered_unlit_and_drew_lava_gates_skip`.

Tile chain (`crates/tr-core/src/wiring.rs`, 1):
`the_water_fountain_trip_flips_the_2x4_body_frame`.

### The 8 g75d FINAL-SWEEP witnesses (folded mid-closeout — written, NOT run)

Disco (`crates/tr-core/src/waterfall/mod.rs` 2 + `app/frame/draw.rs` 1):
`the_disco_fall_reads_the_shared_disco_latch` (the AddLight cell corrected
to (9, 12) — `:548` runs before the `:663` conversion),
`update_tick_does_not_cycle_the_disco_rgb`,
`the_waterfall_block_latches_the_shared_disco_statics`.

Lava fountain (`crates/tr-core/src/waterfall/mod.rs`, 2):
`lava_fountain_tile_717_drops_below`,
`lava_fountain_draws_the_two_sheet_pair_and_lights_every_cell` (the
water-scan assertion corrected to the load reset — the rain arm
`continue`s at `:525` before the scan block).

Spout clause (`crates/tr-app/src/render/liquids.rs` 2 + `render/tiles.rs`
1): `w4_g75_a_spouting_half_brick_refuses_its_liquid_behind_draw`,
`w4_g75_a_bubble_neighbour_does_not_feed_the_spout_clause`,
`the_liquid_behind_call_feeds_the_waterfall_manager`.

Lane total after the closeout + sweep: 43 + 7 closeout-unique + 8 sweep =
58. The closeout's chain-status census (the 338/207 fountain chains — the
proj AI rides the d8 lane `50dcaa75` —, the g75d fold, the g24 stop claim)
is recorded in ANALYSIS "W4-g73 FALLS CLOSEOUT" + "The g75d final-sweep
FOLD".

### The RETIRE-phase witnesses (2026-11-21 — written, NOT run)

Quit-stop keep-one (3 g24-shape witnesses replace the closeout's 1 — the
dead-slot/idempotence assertions carried over; net +2):
`w4_g24_the_ambience_stop_pair_and_the_tracked_pool_drain`
(`crates/tr-app/src/audio.rs`),
`w4_g24_the_quit_paths_stop_the_ambience_and_the_tracked_pool`
(`crates/tr-app/src/app/saveglue.rs`),
`w4_g24_the_quit_row_runs_just_quit` (`crates/tr-app/src/debug_panel.rs`,
the `debug` feature's test module). The stale fifteen-rows witness is
corrected to `s1_e_29_the_world_tab_keeps_its_sixteen_rows` (a fix, not a
new witness).

Fountain head arm (4):
`the_fountain_head_arm_outranks_every_biome_arm` +
`no_fountain_falls_through_and_the_capture_path_ignores_fountains`
(`crates/tr-app/src/biome.rs`),
`the_stored_fountain_water_style_feeds_the_drop_frame_types`
(`crates/tr-app/src/rain.rs`),
`the_tile_pass_water_style_carries_the_fountain_head_arm`
(`crates/tr-app/src/render/tiles.rs`).

**FINAL lane total: 58 − 1 retired + 3 keep-one + 4 head-arm = 64
witnesses** (all written NOT run — code-only).

## Verification the merger must run (this lane was code-only)

```powershell
cargo test --workspace -- --skip migrates_version_2_kinds_to_item_ids
cargo test -p tr-app --test main_anchors
cargo test -p tr-app --features debug   # the F3 quit-row witness lives in the debug-feature module
```

Judge only by the `test result: ok` lines (cargo may exit 1 on success).
The final fold was compile-checked by inspection only — if the 64
witnesses surface fixture drift, fix the FIXTURE against the C# walk (the
decisions above are locked; the numbers are hand-replays, not run
results). Known inspection notes for the verifier:

* The g73 union witnesses ride the 9-arg latch-form `draw` (the
  `show_invisible` parameter removed; the latch defaults false, which is
  what every folded witness passed anyway).
* The shimmer consumer reads the generic `lookups.waterfall_uv(texture,
  source)` (the `Waterfall_25` sheet loads with the rest of the family).
* The delegation `liquid_loop_update` → `liquid_ambience` passes the
  pre-pick floats; `liquid_ambience` re-truncates internally exactly like
  the `(int)` PlaySound casts, so the decision half's `(xi, yi, si)` and
  the volume half's `(px, py, ps)` agree by construction. The one
  behavioural difference from g75's inline consumer: `liquid_loop_update`
  gates on `triple.0 <= 0` BEFORE the walk (the C# `Main.cs:13475`/
  `:13498` shape) — `liquid_ambience`'s own camera-centre stop arm remains
  for its direct witnesses.
* RETIRE: `draw_tiles` gained one parameter (`active_fountain_color:
  i32`, after `liquid_alpha`) — the single frame caller feeds
  `self.scene_metrics.active_fountain_color`; `Rain` gained the
  `water_style: u32` latch (init 0, the `update_rain` feed). The quit-stop
  witnesses run headless through the counter seam
  (`liquid_ambience_stop_count`), never a real sink.
* The g24 merge overlap is documented in §-quit: identical bodies, ONE
  `:6704` call site; the g46/g47 merges carry no `biome.rs`/`rain.rs`
  water-style hunks (their versions predate the head arm — this branch's
  copies win, §-fountain).