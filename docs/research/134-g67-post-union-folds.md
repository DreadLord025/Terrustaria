# 134 — W4-g67 post-union fold-list (for W4-d130f)

**STATUS: RETIRED — R1–R4/R6-partial EXECUTED BY d130 `16a792d5`; R5/R6-caller/R7
+ the 2-hunk fold-addendum OPEN for d130f** (code-only,
`task/W4-g67-weather-rain-splash`).

## Fold-monitor re-check (g67 retire phase, 2026-11-21, tip `97ed44f5`)

The previous re-check (below, against `560a02eac`) is **STALE**: d130's
`16a792d5` (08:55) folded the R1–R7-direct residuals BEFORE g67's `134` FINAL
(`e777b3e2`, 09:13) was published — the d130 tip `97ed44f5` records the 134
absence and folds from the g67 commit bodies themselves. Re-verified per-hunk
against the NEW tip's actual file bodies (`git show
origin/task/W4-d130-weather-ambience:<path>` at `97ed44f5`):

| residual | d130-tip `97ed44f5` evidence | verdict |
| --- | --- | --- |
| R1 | `rain.rs:316` `is_it_raining`; feeds `frame/world.rs:192/259/503`, `netmodules.rs:486`, `weather.rs:116` — all `is_it_raining()` now | **EXECUTED** (`16a792d5`) |
| R2 | `rain.rs:325` `clear_rain`; `saveglue.rs:366` calls it after the save | **EXECUTED** (`16a792d5`) |
| R3 | `rain.rs:1500/1512` `splash_cull_rect`/`dust_in_cull_rect`; `draw_splashes:1410-1417` + `snow.rs:316-323` apply the else-kill (the snow lane d130 extended on its own) | **EXECUTED** (`16a792d5`) |
| R4 | `rain.rs:1132` `new_rain_forced` (fixed-slot scan, `#[allow(dead_code)]` — caller still unported there) | **EXECUTED** (`16a792d5`) |
| R5 | `rain.rs:1088` still reads `thresholds::SNOW_TILE_NORMAL as f32`; `:1083-1086` documents the gap ("the skyblock arm lands with …") | **OPEN → d130f** |
| R6 | seam (`new_rain_forced`) landed, but `player.rs` grep: still no `eyebrella`/`5101` caller arm | **caller OPEN → d130f** |
| R7 | `player.rs` grep: no `umbrella_hold_slow_fall` arm (only unrelated holdStyle-4 pose cites) | **OPEN → d130f** |

The "Already folded" table below still holds at the new tip (splash lane
`88cfbf38`, impact gate `50c845e5`, float attempt loop `spawn:996`,
`new_rain:1069`, f39 start gate via `cf4574e9`) — none regressed. d130's
`16a792d5` ALSO folded the gameMenu kill-only arm (`update_drops:948`,
`RainInputs.game_menu:235`) + the `:84` menu spawn-through disjunct
(`spawn:1046`) — the "Deliberate divergences" menu bullet below is thereby
RETIRED (d130 took g67's arm after all).

## Fold-addendum for d130f — the 2 draw hunks that POST-DATE the fold

g67's final phase 3 (`7aaf66f0`, 09:02) re-pinned `Main.DrawRain`'s light tile +
XNA byte tint AFTER d130's `16a792d5` fold (08:55). Re-reading the d130 tip
shows d130's own phase-2a core (`ea4863d9`, 01:49) ALREADY carried both fixes —
so the addendum is **1 semantic delta + 1 witness pin**, not 2 re-ports:

* **A1 — the tint ALPHA channel (SEMANTIC DELTA, fold required).** g67's
  `rain_drop_tint` (`rain.rs:1595`) byte-casts ALL FOUR channels but its
  witness `w4_g67_draw_rain_tint_is_the_xna_byte_chain` (`:3230`) asserts
  alpha stays 255 ("the C# never dims it") — that assertion is WRONG against
  the code AND against the C#: `Color * 0.85f` multiplies A too
  (`Main.cs:38813`, XNA `Color.operator*` scales all four bytes), so full
  light draws alpha 216, and shimmer 0.25 gives 162. d130's `rain_tint`
  (`rain.rs:1297`) is byte-identical to g67's fn body and its witness
  `the_draw_tint_is_the_xna_byte_chain` (`:1939`) pins the CORRECT values
  (`[216/255; 4]`, alpha 162 under shimmer 0.25). **d130f: keep d130's
  `rain_tint` + witness verbatim; g67's alpha-255 assertions are retired
  with the wholesale drop — do NOT port them.** (g67's own witness is
  internally inconsistent — written-not-run; the union's compile gate will
  never see it because the copy drops.)
* **A2 — the light-tile witness pin (NO code delta).** d130's `draw` inlines
  the `(pos+4)>>4` sample as the `light_tile` closure (`rain.rs:1250-1256`) —
  semantically identical to g67's `rain_light_tile` helper (`:1585`), but
  d130 has NO dedicated geometry witness. **d130f: re-pin g67's
  `w4_g67_draw_rain_light_tile_is_pos_plus_four_shifted` (`rain.rs:3211`)
  onto the closure's expression** (the 12..15-px-band crossing, the 63.9
  edge, the negative-coordinate truncation arms). Pure addition.

## Fold-monitor re-check (SUPERSEDED — was against tip `560a02eac`)

The table below was the final-phase reading before d130's `16a792d5` landed;
kept for the record — every row's verdict is superseded by the retire-phase
table above.

| residual | d130-tip `560a02eac` evidence (re-read 2026-11-21, pre-fold) | verdict then |
| --- | --- | --- |
| R1 | `app/frame/world.rs:188/253/494`, `app/netmodules.rs:484`, `world_sim/weather.rs:110` all fed `self.rain.is_raining()`; no `is_it_raining` in the fn-list | OPEN (now EXECUTED) |
| R2 | `app/saveglue.rs:346` `save_and_quit` — the "ClearRain is visual-only" comment, no `clear_rain()` call | OPEN (now EXECUTED) |
| R3 | `rain.rs:1308-1310` documented the 4×4 cull as "not modelled … a documented widening" | OPEN (now EXECUTED) |
| R4 | no `new_rain_forced` in the fn-list | OPEN (now EXECUTED) |
| R5 | `rain.rs:1048` `new_rain` read `thresholds::SNOW_TILE_NORMAL as f32` unconditionally | OPEN (still OPEN) |
| R6 | `player.rs` grep: no `eyebrella`/`5101` arm | OPEN (seam done, caller still OPEN) |
| R7 | `player.rs` grep: no `umbrella_hold` arm | OPEN (still OPEN) |

**STATUS (original): FOLD-LIST PUBLISHED** (code-only, `task/W4-g67-weather-rain-splash`).
The d130 union-exec (`88cfbf38`, research/83 §3) transplanted g67's **phase-1
splash lane** (`40128e5d`) and reconciled the impact gate (`50c845e5`), but its
tip census only covered g67 up to `006bc04d` (phase 2). g67 has since landed
**phases 3–6** (`d2dd8989`/`5c9398ea`/`2c452edd`/`112d1b20`) plus three follow-up
commits (`f8c4b659`/`356a1ac0`/`01ba6c7a`). This note enumerates, per file/hunk,
what the **current d130 tip `560a02eac`** still lacks from g67, so d130d/e can
fold the residuals without re-deriving them.

Every claim below was re-derived from the C# at the cited lines AND checked
against `origin/task/W4-d130-weather-ambience` at `560a02eac` (re-fetched), not
from commit messages. The d130 branch keeps its own `RainInputs`/fixed-750-slot
shape (research/83 §3 — g67's compacted-Vec `update_drops` is NOT taken
wholesale); the folds below are the **semantic residuals** that fit d130's shape.

## Already folded by d130 (do NOT re-fold)

| g67 content | d130 home | verdict |
| --- | --- | --- |
| Phase-1 splash recipe (`spawn_splash` = `DustPool::new_dust` + the eight `Rain.cs:123-130` writes, sentinel included) | `rain.rs::spawn_splash` | FOLDED (`88cfbf38`) |
| `water_splash_arm` (`Dust.cs:1834-1874` family physics) in the shared `update_dust` hook | `rain.rs::water_splash_arm` + `minecart.rs:1113` | FOLDED |
| `draw_splashes` generic path (GetAlpha tint, (4,4)-origin rotated quad, GetColor overlay, Black kill) | `rain.rs::draw_splashes` + `app/frame/draw.rs:1293` | FOLDED (but see residual R3 — the cull rect) |
| The `Option<(DustPool,SpawnCtx)>` drain seam; `update_rain` hands `app.dust`+`spawn_ctx` | `rain.rs::update_drops`/`update` + `weather.rs::update_rain` | FOLDED |
| Impact gate: ONE combined `solid\|\|below\|\|wet` else-if, `Next(100)<gfxQuality*100` rolls for ALL THREE kills, `active=false` first, raw `+100f` bottom margin | `rain.rs::update_drops` (`:860-905`) | FOLDED (`50c845e5` reconcile — g67's reading was RIGHT) |
| Remix kill-only arm (`Rain.cs:110-116`, `+100f`, never rolls) | `rain.rs::update_drops` (`i.remix_world`) | FOLDED |
| `MakeRain` float attempt loop (`while attempt < num`), AnyDanger 0.05 throttle + `>1→1` floor, rising-player `num3` window, wind lead `wind*15*40` + `player.vx*40`, world-X clamp, open-sky gate | `rain.rs::spawn` (`:917-985`) | FOLDED (d130's loop is MORE faithful than g67's `spawn_acc` accumulator — keep d130's) |
| `NewRain` budget (`maxRain*cloudAlpha`, gfx `(1+q)/2` cut q<0.9, snow `(1-r²)`, `atmo^9`, `atmo<0.4` kill), discarded depth factor, live `GetRainFallVelocity` (`wind*18,14`), `waterStyle*3+Next(3)` | `rain.rs::new_rain` (`:990-1045`) | FOLDED |
| The `:64890` `!slimeRain && !LanternsUp && !NextNightIsLanternNight` start gate + the `:64868-64871` lanterns-up StopRain arm | `rain.rs::update_time` (the three sampled flags) + `weather.rs:250` | FOLDED via **W4-f39** (`cf4574e9`) — d130's `update_time` gained `slime_rain`/`lanterns_up`/`next_night_lanterns` params; g67's `start_gates_open` bool is the same gate, superseded |

## Residuals (the fold-list — receipts per the retire-phase table)

R1–R4 are **EXECUTED** by d130 `16a792d5` (receipts at tip `97ed44f5` in the
retire-phase table); their specs below are kept as the fold record. **R5, R6's
caller, R7 remain the d130f work-list**, plus the 2-hunk fold-addendum (A1/A2)
above.

### R1 — `IsItRaining` consumers (g67 phase 4, `5c9398ea`) — EXECUTED `16a792d5`

`Main.IsItRaining` = `cloudAlpha > 0` (`Main.cs:2597`), NOT the `Main.raining`
event flag. The ambience scheduler / music rain gates / storm-lightning gate
read it, so they stay "raining" through the post-`StopRain` cloud ramp-down.
d130 has **no `is_it_raining()` method** and feeds these consumers the event
flag (`is_raining()`):

| d130 file:line (current) | current | fold to |
| --- | --- | --- |
| `app/frame/world.rs:188` | `rain: self.rain.is_raining()` (music rain gate, `Main.cs:12100`) | `self.rain.is_it_raining()` |
| `app/frame/world.rs:253` | `quiet_sky: !self.rain.is_raining() && …` (`Main.cs:12670` `cloudAlpha==0`) | `self.rain.visual_alpha() == 0.0 && …` |
| `app/frame/world.rs:494` | `is_raining: self.rain.is_raining()` (ambience, `AmbienceServer.cs:32/50`) | `self.rain.is_it_raining()` |
| `app/netmodules.rs:484` | `is_raining: self.rain.is_raining()` (ambience net) | `self.rain.is_it_raining()` |
| `world_sim/weather.rs:110` | `raining: app.rain.is_raining()` (thunder gate, `WorldGen.cs:71872`) | `app.rain.is_it_raining()` |

d130 already has `visual_alpha()` (`rain.rs:286`, the `cloudAlpha` clamp) — add
`is_it_raining() -> bool { self.intensity > 0.0 }` beside it (g67 `rain.rs:316`).
**Witness:** g67 `w4_g67_clear_rain_and_is_it_raining` (the event flag off but
`cloudAlpha 0.5 > 0` → `is_it_raining()` true).

### R2 — `ClearRain` on save-and-quit (g67 phase 4, `5c9398ea`) — EXECUTED `16a792d5`

`Rain.ClearRain` (`Rain.cs:22-28`) deactivates every drop; the C# calls it from
`SaveAndQuitCallBack` (`WorldGen.cs:6672`) AFTER the save snapshotted the live
rain. d130's `save_and_quit` (`app/saveglue.rs:346`) still carries the old
"ClearRain is visual-only" comment and **never calls it** — the residual drop
field survives onto the menu.

* **Fold:** add `self.rain.clear_rain();` after `self.save_world();` in
  `save_and_quit` (g67 `saveglue.rs:361-368`). d130 needs a `clear_rain()` method
  on its fixed-slot `Rain` (deactivate all 750 slots — g67's compacted-Vec
  `drops.clear()` is the same semantic; adapt to d130's `[Drop; MAX_DROPS]`).
* The gen-side call (`WorldGen.cs:6747`) is a no-op (a fresh world boots a fresh
  empty `Rain`).
* **Witness:** g67 `w4_g67_clear_rain_and_is_it_raining` (the `clear_rain` half).

### R3 — the `DrawDust` cull rect + else-kill (g67 phase 5, `2c452edd`) — EXECUTED `16a792d5` (also `snow::draw`)

d130's `draw_splashes` (`rain.rs:1266`) still documents the 4×4 cull rect as
"not modelled — a documented widening". **That widening is WRONG** (g67 phase 5
verified it): `Main.DrawDust`'s live cull rect is the wide ±1000/±1050 slack box
(`Main.cs:38832` overwrites `:38831`'s ±500/±50 box; the trail types' `:38845`
restore copies the SAME wide rect), `:38847` tests a 4×4 dust box with XNA's
strict-`<` intersection, and the `:39020-39022` **ELSE arm DEACTIVATES** a dust
that misses — a kill, not a skip.

* **Fold:** port `splash_cull_rect(screen_pos, screen) -> (i32,i32,i32,i32)` =
  `(screenX-1000, screenY-1050, screenW+2000, screenH+2100)` and
  `dust_in_cull_rect(position, rect)` (strict-`<` on both axes), and apply the
  deactivation in `draw_splashes` before the draw (g67 `rain.rs:1531-1556` +
  the `draw_splashes` head). The rect is world-px (no zoom term); map it through
  the camera-centred view top-left (`camera - screen*0.5/zoom`).
* **Witness:** g67 `w4_g67_drawdust_cull_rect_is_the_wide_box_with_an_else_kill`
  (the wide-box edges, the 4×4 boundary, the shifted-view kill).

### R4 — `NewRainForced` (g67 phase 4, `5c9398ea`) — EXECUTED `16a792d5` (seam; caller = R6)

`Rain.NewRainForced` (`Rain.cs:135-164`) — the Eyebrella vanity drip's spawn
entry. d130 has **no `new_rain_forced`**. Unlike `NewRain` it ignores
`cloudAlpha`/snow/atmo: budget = `maxRain` cut by `(1+q)/2` when q<0.9
(`:139-143`), first inactive slot, returns `maxRain` when full (`:152-155`);
the drop fields are the same `:159-162` shape (scale `Next(-20,21)*0.01`,
`velocity*scale`, `atan2(vx,-vy)`, `waterStyle*3+Next(3)`).

* **Fold:** port `new_rain_forced` onto d130's fixed-slot pool (the first
  inactive slot within the gfx-cut budget; g67 `rain.rs:336-367`). The caller
  (the Eyebrella `UpdateEquips` arm) is g67 follow-up `356a1ac0` — see R6; d130
  can land the seam first (it is `#[allow(dead_code)]` until the caller folds).
* **Witness:** g67 `w4_g67_new_rain_forced_budget_and_shape` (q0 → cap 375, the
  376th refuses with the `maxRain` sentinel).

### R5 — the skyblock snow-threshold arm (g67 follow-up, `f8c4b659`) — OPEN → d130f

d130's `new_rain` snow suppression reads the **normal 1500** unconditionally
(`rain.rs:1048`, `SNOW_TILE_NORMAL as f32`). `SceneMetrics.SnowTileThreshold` is
skyblock-aware: 300 when `WorldGen.Skyblock.lowTiles`, else 1500
(`SceneMetrics.cs:154-163`).

* **Fold:** g67 added `World.skyblock_low_tiles` + `World::scan_skyblock_low_tiles`
  (the `Skyblock.ScanTiles`/`Calculate` lowTiles census, `WorldGen.cs:3210-3218`/
  `:3238-3265`, filled at load `WorldFile.cs:766` + gen completion) and
  `scene::snow_tile_threshold(low_tiles)` / `desert_tile_threshold`. d130's
  `new_rain` should read `snow_tile_threshold(world.skyblock_low_tiles)`; the
  scene scan's `ZoneSnow`/`ZoneDesert` read the same arm. Field shape adopted
  from W4-f24's `SkyblockState.low_tiles` (`9cbbf9de`). **Coordinate with f24**
  (the generator-side ledger) — the runtime flag lives on `World`.
* **Witnesses:** g67 `w4_g67_skyblock_low_tiles_census` (world.rs),
  `w4_g67_skyblock_thresholds_pick_300_over_1500` (scene.rs),
  `w4_g67_skyblock_low_tiles_lowers_the_snow_threshold` (rain.rs).

### R6 — the Eyebrella caller + `eyebrellaCloud` (g67 follow-up, `356a1ac0`) — caller OPEN → d130f

The `UpdateEquips` arm (`Player.cs:13083-13087`): head armor slot 0 or vanity
head slot 10 wearing 5101 fires `DoEyebrellaRainEffect` (`:14308-14331`) and
arms `eyebrellaCloud` (`ResetEffects` clears it, `:18461`). This is a
**player.rs / world_sim/player.rs** lane (not rain.rs) — d130 keeps its own
player files, so this folds independently of the rain union. The drip's
`Next(4)` cadence + `NextFloat` window ride the player's `UpdatePlayers`
stream; the position queues and drains into `new_rain_forced` (R4) with the
live `wind*3, 9` velocity.

* **Fold:** g67 `player.rs::do_eyebrella_rain_effect` + `eyebrella_cloud` field +
  the `UpdateEquips` arm + the `pending_eyebrella_rain` drain in
  `world_sim/player.rs`. Depends on R4 (`new_rain_forced`).
* **Witnesses:** g67 `w4_g67_eyebrella_drip_cadence_and_window`,
  `w4_g67_eyebrella_drip_refuses_solid_and_wet_tiles`,
  `w4_g67_eyebrella_cloud_arms_from_head_or_vanity_slot`.

### R7 — the umbrella held-item arm (g67 follow-up, `01ba6c7a`) — OPEN → d130f

The Umbrella (946) / Tragic Umbrella (4707) `holdStyle == 2` sim arm
(`Player.ItemCheck_ApplyHoldStyle`, `Player.cs:50476-50498`): at rest +
`!pulley`, the fall caps at 2 px/tick (no `controlDown`) + the fall origin
refreshes every tick. Also a **player.rs / render** lane, independent of the
rain union.

* **Fold:** g67 `player.rs::umbrella_hold_slow_fall` (called at the `ItemCheck`
  position, after `BordersMovement`) + the `hold_item_placement` holdStyle-2
  umbrella override in `render/player_layers.rs` (overhead at
  `width/2 - 16*dir, 22`, rotation 0).
* **Witnesses:** g67 `w4_g67_umbrella_hold_caps_the_fall_at_2`,
  `w4_g67_umbrella_hold_places_the_item_overhead`.

## Deliberate divergences (KEEP d130's, do not fold g67's)

* **The spawn cadence.** g67 accumulates the fractional rate (`spawn_acc`,
  floor per tick); d130 runs the C# float attempt loop (`while attempt < num`,
  ceil(num) attempts per call). d130's is the faithful shape — keep it.
* **The `gameMenu` kill-only arm — DIVERGENCE RETIRED.** g67 ports
  `Rain.cs:103-109` (`+2000f`) verbatim behind `DropSimInputs.game_menu`;
  d130 originally folded it out, but `16a792d5` took g67's arm after all
  (`update_drops:948`, `RainInputs.game_menu:235`, move-first no-roll no-probe)
  plus the `MakeRain:84` menu spawn-through disjunct (`spawn:1046`). The two
  copies now agree — nothing left to decide here.
* **The drop storage.** g67 uses a compacted `Vec<Drop>` (swap_remove); d130
  uses a fixed `[Drop; 750]` in slot order. Keep d130's (research/83 §3).

## g67 commit → file → hunk index (the four code commits + three follow-ups)

| commit | files | the unique hunks |
| --- | --- | --- |
| `40128e5d` (phase 1) | `rain.rs`, `minecart.rs`, `world_sim/weather.rs`, `app/frame/draw.rs` | splash lane — **FOLDED** by d130 `88cfbf38` |
| `d2dd8989` (phase 3) | `rain.rs`, `world_sim/weather.rs` | `DropSimInputs` seam + menu/remix kill-only arms + AnyDanger gathering — ALL **FOLDED** (menu arm via `16a792d5`, the fold-out retired) |
| `5c9398ea` (phase 4) | `rain.rs`, `app/frame/world.rs`, `app/netmodules.rs`, `app/saveglue.rs`, `world_sim/weather.rs` | MakeRain/NewRain verbatim (**FOLDED**), `NewRainForced` (**R4 EXECUTED**), `ClearRain` (**R2 EXECUTED**), `IsItRaining` consumers (**R1 EXECUTED**), `:64890` start gate (**FOLDED** via f39) |
| `2c452edd` (phase 5) | `rain.rs` | DrawDust cull rect + else-kill (**R3 EXECUTED**, d130 extended it to `snow::draw`) |
| `112d1b20` (phase 6) | `docs/ANALYSIS.md`, `docs/PORT_MAP.md` | docs-only — the census record (no code) |
| `f8c4b659` (follow-up) | `world.rs`, `scene.rs`, `scene_scan.rs`, `rain.rs`, `worldgen/mod.rs`, `app/mod.rs` | skyblock low-tiles + snow threshold (**R5 OPEN → d130f**) |
| `356a1ac0` (follow-up) | `player.rs`, `world_sim/player.rs`, `rain.rs` | Eyebrella caller + `eyebrellaCloud` (**R6 caller OPEN → d130f**, seam R4 done) |
| `01ba6c7a` (follow-up) | `player.rs`, `render/player_layers.rs` | umbrella held-item arm (**R7 OPEN → d130f**) |
| `7aaf66f0` (final phase 3) | `rain.rs`, `app/frame/draw.rs` | DrawRain light tile + XNA byte tint — d130 `ea4863d9` already carried both; **A1** (g67's alpha-255 witness assertions WRONG, d130's correct — keep d130's) + **A2** (light-tile witness re-pin) POST-DATE `16a792d5` — the fold-addendum |

**Fold order for d130f:** R5 (skyblock, coordinate f24) → R6 (Eyebrella caller,
seam already landed) → R7 (umbrella, independent) + A2 (light-tile witness
re-pin; A1 needs NO code — keep d130's `rain_tint`/witness verbatim). All g67
witnesses are WRITTEN NOT RUN (code-only mode) — re-pin them to d130's
`RainInputs`/fixed-slot shape on fold.

## KEEP-d130-CANONICAL instruction (FINAL-FINAL — g67 retire phase, fold-source tip `e777b3e2`)

The union-exec decision (research/83 §3, d130 `ec1d73c7`) stands: **the merger
keeps d130's `rain.rs` core wholesale; g67's `rain.rs` drops wholesale EXCEPT
the still-open folds (R5/R6-caller/R7) and the A2 witness pin.** The fold
record above is the ONLY g67 content that crosses the boundary, hunk by hunk —
never take a g67 fn body over a d130 one. Concretely:

1. **KEEP** d130's float attempt loop (`spawn:996-1067` at tip `97ed44f5`),
   fixed-750-slot pool in slot order, `RainInputs` shape, the spawn-then-update
   tick order (the C# net order — `Main.Update:17220` spawns, `DrawRain:38819`
   moves), the verbatim `new_rain` (`:1069-1130`), the `update_drops` impact
   gate (`50c845e5` reconcile), the gameMenu arm d130 took at `16a792d5`
   (`:948`, the earlier fold-out retired), and `crate::snow`'s `Main.snowing()`
   port (the `:13092` raining branch included — g67's `snow_mode` biome gate is
   the retired approximation, ANALYSIS "visual-overlap census").
2. **ALREADY FOLDED — do NOT re-fold:** R1 (`is_it_raining` + the five
   consumers), R2 (`clear_rain` + the `save_and_quit` call), R3 (cull rect +
   else-kill, splashes AND snow), R4 (`new_rain_forced` seam) — all executed by
   d130 `16a792d5`, receipts in the retire-phase table.
3. **STILL TO FOLD (d130f):** R5 (the skyblock snow threshold — d130's
   `new_rain:1088` still reads the unconditional 1500; the gap is documented
   at `:1083-1086`), R6's caller (the Eyebrella `UpdateEquips` arm +
   `eyebrella_cloud` + the `pending_eyebrella_rain` drain — the R4 seam is
   waiting `#[allow(dead_code)]`), R7 (the umbrella holdStyle-2 sim + render
   arms), and **A2** (the light-tile geometry witness re-pin). **A1 needs no
   code**: d130's `rain_tint` already dims alpha correctly — g67's
   alpha-stays-255 witness assertions drop with the wholesale copy.
4. **RE-PIN** the g67 witnesses to the fixed-slot shape on fold (the
   `pin_drop`/`RainInputs`/`drop_count` conventions d130 `88cfbf38` already
   established for the splash lane).

**Draw-side boundary (FINAL-FINAL):** the falling-drop draw (`Main.DrawRain`,
`Main.cs:38800-38826`) is d130's `draw`/`rain_tint` — canonical at tip
`97ed44f5` (the `(pos+4)>>4` light tile inlined at `rain.rs:1250-1256` + the
XNA byte chain `rain_tint:1297` + the shimmer arm, alpha INCLUDED — the C#
`Color * 0.85f` scales A too). g67's final phase 3 (`7aaf66f0`, POST-DATING
d130's fold `16a792d5`) re-pinned the same light-tile/tint semantics as
`rain_light_tile`/`rain_drop_tint` helpers, but its tint WITNESS wrongly pins
alpha at 255 (A1) — the fn bodies agree, the assertions do not; **d130's copy
and witness are the correct ones, keep them verbatim**. The merger keeps
d130's draw and drops g67's wholesale (the A2 geometry witness is the one
salvage). The splash draw (`draw_splashes`, the `DrawDust` generic path) is the
transplanted g67 lane d130 already carries — R3's cull fix landed in place at
`16a792d5`. No d130-draw_splashes branch exists on origin (re-checked
2026-11-21 at the retire phase); the splash draw is NOT unclaimed — it lives
in d130's tip via `88cfbf38`.

**Final-phase census additions (g67, for the merger's benefit):**
* `Rain.cs` fn-census COMPLETE: 6/6 methods + 7/7 fields mapped; the `alpha`
  field is dead in the C# (zero readers/writers) — no port gap (ANALYSIS
  "FINAL fn-census").
* `DrawRain` has NO cull rect and NO black-kill arm (unlike `DrawDust`) — the
  drop draw needs neither; the per-drop `Update` cadence fold is recorded.
* The wind-slant is `atan2(vx,-vy)` over the live `wind*18,14` at spawn — no
  draw-angle term; the dust rotation lane (`Dust.cs:2081`) is separate and
  already faithful on both branches (ANALYSIS "visual-overlap census").