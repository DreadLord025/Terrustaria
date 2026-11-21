# 84 — Map reveal dup-reconcile: W4-e18 vs W4-f46 (2026-11-21, code-only)

> **STATUS: COMPLETE (2026-11-21 closeout — see §10 for the final status +
> merger checklist).** EXECUTED (2026-11-21, salvage-exec on
> `task/W4-f46-minimap-reveal-2`).
> The §8 union landed on this branch: e18's skeleton adopted verbatim
> (base commit `0bc3414d`), then the six f46 salvages + the
> `sections_left()` accessor committed per piece. **Fold phase (same day):
> e18's three post-`c6cf86bf` commits folded per-hunk — the union's e18
> base is now `670c948d` (e18-latest tip).** The merger keeps THIS branch's
> union ONLY and drops the separate e18/f46 versions. Execution notes:
>
> - The driver entry is `frame/world.rs::map_reveal_draw` — a field-borrow
>   associated fn (the draw region's 55-field destructure holds
>   `map`/`world`/`settings` out of `&mut self` reach; `section_manager`/
>   `debug_unlock_map` stay disjoint `self` fields). The §8 "adapted to
>   `app.map_reveal_driver(paused)`" shape was not borrowable at the f46
>   call site; same semantics, field-passing shape.
> - The sweep splits into `map_reveal_sweep_core(budget_ms)` (GPU-free
>   state half, the witnesses' seam) + the paint wrapper inside
>   `map_reveal_draw` (`checkMap` ensure → walk → per-section
>   `draw_to_map_section`). The consume splits the same way
>   (`map_reveal_consume` → `map_reveal_consume_fields`).
> - `unlock_map(&mut i32, bool)` takes the option by `&mut i32` (§4's
>   recommendation); the debug row routes through it; the e18 inline
>   arming is gone. The `WorldMap.sections`/`unlock_map_mode` fields never
>   existed on the union (e18 base).
> - The §7 clearMap-consume witness adaptation is covered by e18's
>   `prepare` consume + the load/tail witnesses (the consume owner in the
>   union is `prepare`, unchanged); f46's `consume_update_map`-shaped
>   witnesses were dropped with that fn per §7's duplicate rulings.
> - Two e18 witnesses had a latent `QueueChange`-after-`refreshMap` arming
>   bug (`MapRenderer.cs:290` no-op) — fixed while re-homing them.
> - Anchors: e18's 3 rows kept (the sweep row's marker follows the rename
>   to `map_reveal_sweep_fields`); f46's 2 rows dropped with the base
>   adoption; ONE new `draw_to_map_section` row added (§8).
> - Compile-by-inspection only (no cargo/tests/graphify, as tasked); all
>   witnesses written not run.
>
> **Fold phase notes (e18 advanced past the union base):**
>
> - Folded `ebd54f66` (fullscreen close button: `ui/chrome.rs` geometry/
>   hover/draw + `clicks.rs::handle_fullmap_close` wired after
>   `handle_fullmap_ping` in `in_world_click_cluster` — the `:55486` <
>   `:56024` C# order), `254f309b` (vanilla thick cursor:
>   `ui/cursor.rs::draw_thick`/`thick_cursor_bonus`/`draw_with_bonus`,
>   `Cursor_11` packed, the fullscreen draw tail after the close button,
>   the `frame.rs` OS-pointer gate now hides over the map) and `670c948d`
>   (census close: 2 anchor rows + PORT_MAP row + ANALYSIS note +
>   research/14 census section). The ANALYSIS conflict (both notes append
>   after the same paragraph) resolved by keeping both; all other files
>   auto-merged — the folded code is byte-identical to e18's tip in
>   `chrome.rs`/`cursor.rs`/`assets.rs`/`frame.rs`/`clicks.rs`.
> - Composition verified by inspection: f46's `map_reveal_draw` call sits
>   at `frame/draw.rs` ahead of `WorldMap::prepare` (before the fullscreen
>   draw branch); e18's close-button + cursor tail sits inside the
>   `map.open` draw branch after the hover text — no shared lines, no
>   double-draw (the HUD cursor `ui::draw_hud` is gated off by
>   `!fullscreen_map`, `draw.rs:1579`; the map tail is the only cursor
>   draw while the map is open). The S1-B-8 orchestrator pin is untouched:
>   `handle_fullmap_close` lives inside `in_world_click_cluster`, not the
>   pinned phase list.
> - **e18d contract (final-sweep still ACTIVE on
>   `task/W4-e18-minimap-reveal-queue` at fold time, tip `670c948d`):** any
>   further e18 commits (zoom-keys/pan-inertia etc.) must be folded into
>   THIS branch before the merger takes it. Fold-status note published in
>   `docs/research/14-map-minimap.md` "W4-e18 → union fold status" — e18's
>   next commits target the union's seams (`map_reveal_draw` field-borrow
>   driver on the draw half, `map.rs` statics/queues skeleton,
>   `handle_fullmap_close` after the ping in the click cluster, the
>   close-button/cursor tail inside the fullscreen draw branch). Re-fold
>   recipe: fetch the e18 tip, cherry-pick each commit past `670c948d`,
>   resolve per-hunk keeping the f46 driver placement + consume structure.
>   **FULFILLED (final-retire pass): the e18 tip `75289a2db`'s ten
>   final-sweep commits are all folded (§10.1 dup fold + §10.2); the
>   contract now reads "any commit past `75289a2db`".**

> Reconcile-phase report. NO code changed on either branch by this pass — this
> document is the only artifact. Inputs read directly from git objects:
>
> - **e18** = `origin/task/W4-e18-minimap-reveal-queue` @ `c6cf86bf`
>   (chain: `c9841111` merge of c35 → `354b50d2` grid → `a19852bb` driver →
>   `e5b12711` audit → `c5cc58b6` statics P1 → `c6cf86bf` census P2).
> - **f46** = `task/W4-f46-minimap-reveal-2` @ `f27a95f2`
>   (chain: `c9841111` → `8fa175ea` tr-core `MapDrawQueue` → `f27a95f2` driver).
> - Merge-base: `c9841111` (both branched off the same c35 merge; **neither
>   contains the other** — 2 unique commits f46, 5 unique commits e18).
> - C# ground truth re-read: `WorldSections.cs:1-359`, `Main.cs:61810-61904`,
>   `MapRenderer.cs:128-201` + `:306-355`, `LightingEngine.cs:246-270`,
>   `ToolkitDebugCommands.cs:480-503` (`Testing/ChatCommands/`),
>   `WorldGen.cs:6981-6986` + `:7150`, `Netplay.cs:770-778`.

The two branches ported the **same C# window twice** with different module
shapes. e18 is the **state superset** (full `WorldSections` bits, the four
`Main` map statics, `QueueChange` threshold, `/clearmap`, `UsingMap`,
`mapReady` gates); f46 is the **fidelity superset on three arms** (driver
placement in the draw half, the real `DrawToMap_Section` pixel paint, the
verbatim `/map` toggle) and the **better consumer structure** (the paused
`ChangesQueued` else-arm). The union keeps e18's skeleton and salvages six
f46 pieces. Details and per-file KEEP/DROP below.

---

## 1. Spiral `GetNextMapDraw` — arm-by-arm fidelity

e18: `crates/tr-app/src/map_sections.rs::MapSections::get_next_map_draw`
(506-line file, fn at :221-342).
f46: `crates/tr-core/src/map/sections.rs::MapDrawQueue::get_next_map_draw`
(349-line file, fn at :109-245).

| C# arm (`WorldSections.cs`) | e18 | f46 | Verdict |
| --- | --- | --- | --- |
| `:220-225` budget exhausted → out (-1,-1)/false | `map_sections_left <= 0 → None` | `sections_left <= 0 → None` | identical |
| `:226-233` scale `*= 0.0625f`, truncating `(int)` casts, `GetSectionX/Y` = `/200`,`/150` | `player_pos * 0.0625`, `as i32`, `section_x/y` via `net::SECTION_W/H` (200/150, plain `/` — truncating, NOT the wire's `div_euclid`) | same maths, literal `/ 200` `/ 150` | identical (constant vs literal only) |
| `:235-243` section-change restart: `vector = playerPos`, leg 4, x/y = player section | verbatim | verbatim | identical |
| `:244-251` same-section resume: leg/x/y/xDir/yDir from `prevMap` | verbatim | verbatim | identical |
| `:252-258` quadrant pick off `(num3+0.5)*200` midpoint, `<= 0 → +1` tie, only when `num == 0` | verbatim | verbatim | identical |
| **`:260` `bool flag = false` BEFORE the loop** | **`let mut flag = false; // :260`** | **`let mut flag = false`** | **BOTH match C#.** See note below. |
| `:261` `flag2 = false` | yes | yes | identical |
| `:264-279` full-lap fallback: second lap → throw; re-centre, re-pick quadrant **from `vector`** (not player), leg 4, num8 = 0 | verbatim, `panic!("Infinite loop in WorldSections.GetNextMapDraw")` | verbatim, same panic message | identical |
| `:280-287` in-bounds → `flag = false`, undrawn → break | `in_grid(x,y)` (x-first check order; C# checks y first — no semantic difference) | inline `y >= 0 && y < h && x >= 0 && x < w` (exact C# order) | identical behaviour |
| `:292-319` leg-4 corner kick: at centre `|num6| > |num7| → y -= num2 else x -= num`; on-axis extend `num9/|num9|`; then `num5 = 0; num8 = -2; flag = true` | verbatim | verbatim | identical code. Comment wording differs (e18 "step off the longer axis", f46 "kick along the SHORTER offset axis") — f46's prose is the accurate reading; the code is the same |
| `:320-333` axis turn: flip the on-axis direction (`<= 0 → +1`), step both, `num5++` | verbatim | verbatim | identical |
| `:334-341` `flag ? num8++ : flag = true` | verbatim | verbatim | identical |
| `:343-347` off-axis: step both directions | verbatim | verbatim | identical |
| `:349-357` mark bit 2, `mapSectionsLeft--`, persist `prevMap` (centre = **scaled** player pos) | verbatim (`|= BIT_MAP_DRAWN`) | verbatim (`drawn[..] = true`) | identical |
| `IterationState.Reset` `:22-30` centre (-3200,-2400), all fields 0 | `reset(&mut self)` + `Default` impl | `reset() -> Self` constructor | identical values |
| `ClearMapDraw` `:149-157` clear bit 2 everywhere, reset prevMap, refill to `data.Length` | verbatim | verbatim | identical |

**The E0381 question answered:** f46's `8fa175ea` commit message describes
fixing an uninitialized `let mut flag;` in its *own in-flight draft* of
`sections.rs`. e18's landed grid (`354b50d2`, verified via
`git show 354b50d2:crates/tr-app/src/map_sections.rs`) **already had
`let mut flag = false; // :260`** — e18 never had the bug. Both landed
versions match C# `:260` exactly; on the first out-of-bounds cell both take
the `else { flag = true }` arm at `:338-341` with a defined value. There is
no fidelity difference to adjudicate here — the fix was intra-branch, not a
divergence from e18.

**Spiral verdict: functionally identical, both verbatim.** Keep ONE copy.
e18's is the keeper because its host type carries the rest of `WorldSections`
(§2); f46's only extras are the `sections_left()` public accessor (e18's
counter is private) and the exact C# bounds-check order (cosmetic).

## 2. `WorldSections` (e18 `MapSections`) vs `MapDrawQueue` (f46) — module union

| Piece | C# | e18 `map_sections.rs` (tr-app) | f46 `tr-core/map/sections.rs` |
| --- | --- | --- | --- |
| Storage | `BitsByte[] data`, 4 bits/section (`:33-45`) | `Vec<u8>`, all four bit constants | `Vec<bool>` — **bit 2 only** |
| Ctor dims | `new WorldSections(maxTilesX/200, maxTilesY/150)` (`WorldGen.cs:7150`) | `new(world_w, world_h)` divides internally (truncating) | `new(width, height)` — caller divides (`map.rs:469`) |
| Resident-world arming | `SetAllSectionsLoaded` (`WorldGen.cs:6744/6821`) + `SetSectionFramed` (`:80363`) | ctor arms `BIT_LOADED \| BIT_FRAMED` every cell; `frameSectionsLeft = 0` (load-count then frame-drain collapsed, documented) | not modelled — doc says "SectionLoaded always true (SP resident)" |
| `SectionLoaded`/`TileLoaded` (`:105-121`) | bounds-checked bit 0; `TileLoaded` rides truncating `GetSectionX/Y` | ported + witness (negative tile → section 0) | absent |
| `SectionFramed`/`MapSectionDrawn`/`SectionNeedsRefresh` | bits 1/2/3 | all three | `map_section_drawn` only |
| `SetSectionAsRefreshed` / `SetAllFramedSectionsAsNeedingRefresh` / `AnyNeedRefresh` / `AnyUnfinishedSections` (`:57-103`) | refresh counters, **no-dedupe** double-count | ported + witness | absent |
| `mapSectionsLeft` accessor | private field | private (tests read it) | **`pub fn sections_left()`** |
| Ownership in the port | `Main.sectionManager` is a **Main static** (`Main.cs:399`), rebuilt at `WorldGen.cs:7150` | **`App.section_manager`** — matches the C# shape; rebuilt on boot (`app/mod.rs:3531`) and net world install (`netglue.rs:2040`) | **`WorldMap.sections`** — a deviation: the grid dies/rebuilds with the map object, not with the world |
| Witnesses | — | 8 (`grid dims`, `first draw own section`, `cover-all-then-exhaust`, `restart-on-move`, `ClearMapDraw`, `refresh double-count`, `TileLoaded truncation`, `quadrant pick`) | 4 (`cover-all`, `restart`, `ClearMapDraw`, `floor grid + bounds`) — all subsumed by e18's 8 except the explicit `sections_left()` reads |

**Union ruling:** keep e18's `MapSections` type and its `App.section_manager`
placement (C#-shaped). DROP f46's `MapDrawQueue` type, file, and the 3-line
`mod sections; pub use sections::MapDrawQueue;` in `tr-core/map/mod.rs`.
Salvage the `sections_left()` accessor onto `MapSections` (f46's driver
witnesses read it). f46's `WorldMap.sections` field is dropped with the type;
the driver reads `self.section_manager` instead (e18 already does).

## 3. Driver keep-one — `Main.cs:61863-61899`

e18: `frame/world.rs::map_reveal_driver(paused)` + `map_reveal_sweep()`
(App methods, called from `update_world_frame` **outside** the pause gate).
f46: `map.rs::WorldMap::draw_map_sections` → `section_walk_core` +
`consume_update_map` → `draw_to_map` (called from `frame/draw.rs:1900-1914`).

| Arm | C# | e18 | f46 | Winner |
| --- | --- | --- | --- | --- |
| **Placement** | the block sits in **`DoDraw`** (after `MapUpdateQueue.Update()` `:61862`, beside `RenderToTargets`) | update side (`update_world_frame`) — runs per game-update, not per draw; deviation (same cadence in the port's lockstep loop, but wrong half) | **draw side** (`frame/draw.rs`, in-world gate `menu.is_none() && front.is_none()`) | **f46** |
| `:61863` `!Main.loadMap` gate | blocks the WHOLE block while a staged `.map` upload runs | `if !m.load_map` early return + witness | not modelled (no `load_map` field; load is synchronous `needs_full_upload`) | **e18** |
| `:61865` `!gameMenu && !generatingWorld` | sweep gate | port world-branch implies it | draw-site in-world gate implies it | tie |
| `:61867-61871` 5 ms wall-clock budget + `GetNextMapDraw(player.position)` | `TimeLogger` stopwatch | `Instant::now()`, hardcoded 5.0, player pos px, `[0,0]` fallback | same, `budget_ms` parameter (tests pass 10 000 to drain) | tie (f46's parameter is test-friendly) |
| `:61873-61876` `unlockMap != 0 && SectionLoaded(x,y)` → `UnlockMapSection` | both gates | **both gates verbatim** (`debug_unlock_map != 0 && section_manager.section_loaded`) | `unlock_map_mode != 0` only — `SectionLoaded` dropped as "SP resident" (behaviourally equal today, less verbatim) | **e18** |
| `:61877` `DrawToMap_Section(x,y)` | paints the 200×150 section into the target (`MapRenderer.cs:306-355`: colour cache row-major, `checkMap` on demand, blit) | **dirty-rect push only** (`section_rect` → `push_dirty`; upload rides `prepare`) — no pixel half, no `checkMap` analog | **real port**: `section_pixels` (GPU-free row-major `GetMapTileXnaColor` cache, world-clipped) + `draw_to_map_section` (`write_map_region`) + `ensure_map_texture` ahead of the walk (`checkMap :322` — creates the texture even when `mapEnabled` is off, like the C#) | **f46** |
| `:61878-61883` `num3` count; empty frame + `!refreshMap` → `unlockMap = 0` | verbatim | verbatim + 2 witnesses (empty-pass zero, pending-refresh hold) | verbatim + 1 witness | tie (e18 has the refresh-hold witness) |
| `:61886` `updateMap.HasValue && !gamePaused` | consume gate | `if paused { return }` after the sweep | `update_map.is_some() && !game_paused` inside `consume_update_map` | see next row |
| `:61888-61892` `refreshMap` → clear flag + `ClearMapDraw`, INSIDE the consume arm, BEFORE `DrawToMap`, regardless of `mapEnabled` | verbatim | verbatim + witness | verbatim + witness | tie |
| `:61893-61894` `DrawToMap(updateMap.Value)` then `updateMap = null` (nulls even when `mapEnabled` refused the draw) | verbatim | `draw_to_map_queues` gated on `map_enabled`; slot nulls unconditionally + witness | `draw_to_map(map_enabled)`; slot nulls unconditionally | tie |
| **`:61896-61898` `else if (ChangesQueued)` → `DrawToMap(Empty)`** | the else-if attaches to the WHOLE `HasValue && !gamePaused` condition — **it runs while paused** when `updateMap` is armed but the pause blocked the first arm, and whenever no rect is armed | **`if paused { return }` skips the else-arm entirely** — a paused frame with queued changes never draws them (deviation); otherwise `changes_queued()` (the real 5×2 lists) is the predicate | `else if !dirty.is_empty()` — **correct paused fall-through**; but `dirty` is a broader predicate than `ChangesQueued` (it also carries reveal tiles the C# never queues — `ExportToMiniMap`/legacy export call `UpdateLighting` directly, `LegacyLighting.cs:429`), so the empty-rect draw fires more often (harmless: same flag lifecycle, no area walk) | **f46 structure, e18 predicate** — union: e18's `changes_queued()` with f46's fall-through |
| `DrawToMap` internals (`MapRenderer.cs:128-201`) | `mapEnabled` gate `:130`; `clearMap\|\|refreshMap` clears change queues `:148-160`; `clearMap` wipe + `:172` reset; tail `:196-199` `mapReady=true; loadMapLastX=0; loadMap=false; loadMapLock=false` | `draw_to_map_queues`: clears the 5×2 lists, sets `map_ready`, lifts the load pair. `clearMap` consume rides **`prepare`** (`clear_map && map_enabled → needs_full_upload`, `:172` reset) — split across two fns, both gated | `draw_to_map`: one fn — gate, `dirty.clear()` on `clear\|\|refresh`, `clear_map → needs_full_upload` + reset. **No tail** (`map_ready`/load pair unmodelled — "structural no-ops" claim is wrong for `mapReady`, which gates `Main.DrawMap :55283`) | **e18** (the tail is live semantics, not a no-op) |
| `Main.DrawMap` `!mapReady` gate (`:55283`) | both draw branches | gates minimap + fullscreen on `map_ready` | absent (f46 removed the gates e18's base never had — it branched before P1) | **e18** |

**Driver verdict: KEEP e18's driver as the state machine** (load gate,
`SectionLoaded` gate, real `ChangesQueued`, `mapReady` tail + draw gates,
pause witness set). **Salvage from f46:**
1. the **call-site placement** — move the `map_reveal_driver` call from
   `update_world_frame` to the draw half (`frame/draw.rs`, f46's site);
2. the **paused else-arm fall-through** — replace `if paused { return }`
   with f46's `consume_update_map` condition structure (keep e18's
   `changes_queued()` predicate);
3. **`section_pixels` + `draw_to_map_section` + the on-demand
   `ensure_map_texture`** — replace the sweep's dirty-rect push with the real
   `DrawToMap_Section` port (the dirty push can stay as the upload bookkeeping
   or go; the pixel write is the C# behaviour);
4. the `budget_ms` parameter shape for testability (optional).

## 4. `/map` flag path — did both replace the bulk sweep?

C# (`ToolkitDebugCommands.cs:480-494`): `Main.clearMap = true` (**both**
toggle arms, `:483`); off→on: `unlockMap = pretty ? 2 : 1` + `refreshMap =
true`; on→off: `unlockMap = 0`. The reveal itself rides the section sweep.

| | e18 | f46 |
| --- | --- | --- |
| Debug row | `debug_panel.rs:815-834`: inline `m.clear_map = true` (both arms), `app.debug_unlock_map = 2` / toggle 0, `m.refresh_map = true` — **flag path, bulk sweep replaced at the call site** | `debug_panel.rs:824-825`: `m.unlock_map(&world, true)` |
| `WorldMap::unlock_map` | **still the OLD bulk sweep** (`map.rs:650-658`: ceil-divided section loop calling `unlock_section` directly) — now **dead code**, kept "for the canary row + a whole-world reveal primitive"; zero callers outside tests | **rewritten as the verbatim flag path** (`map.rs:627-637`): `clear_map = true`; toggle `unlock_map_mode` 1/2 + `refresh_map`, or 0. No ceil loop anywhere |
| Toggle-off witness | row witness covers the arming (`s1_l_3_the_reveal_row_unlocks_the_map`); no explicit toggle-off/flat-mode witness | `w4_f46_map_command_arms_the_flags_and_toggles_off` covers pretty, toggle-off, flat |
| Option home | `App.debug_unlock_map` (≈ `DebugOptions.unlockMap`, a static — closer to the C# shape) | `WorldMap.unlock_map_mode` (a map field — deviation, benign) |

**Both replaced the bulk sweep on the live path.** f46's version is the
cleaner union target: one verbatim `unlock_map` fn, no dead ceil-loop copy.
Salvage f46's `unlock_map` body + toggle witness; the debug row may keep
e18's inline arming or call the salvaged fn (identical flags). Keep e18's
`App.debug_unlock_map` field name/placement; f46's `unlock_map_mode` is the
same state under a different owner — pick ONE (recommendation: e18's App
field, rewrite the salvaged `unlock_map` to take the option by `&mut i32` or
move the arming into the row as e18 has it, keeping f46's toggle semantics
and witness).

## 5. e18-only pieces f46 lacks (census P1/P2 — all KEEP)

- **The four `Main` statics** `clear_map`/`map_ready`/`load_map_lock`/
  `load_map_last_x` (`Main.cs:789-805`) + `load_map` + the `DrawToMap` tail
  (`MapRenderer.cs:196-199`) in `draw_to_map_queues`; the `.map` load arms
  all three (`WorldMap.cs:211-213`) and disarms `refreshMap` (`:214`).
  f46's `load()` arms only `clear_map` + `refresh_map = false`.
- **`change_queues` (5×2) + `queue_change`'s 2048-per-target
  `ChangeRefreshThreshold`** (`MapRenderer.cs:288-304`) + `changes_queued`
  (`:37-56`), fed by `map_phase` (`for (x,y) in changed { m.queue_change }`).
  **f46 has no `QueueChange` at all** — its `map_phase` only `push_dirty`s,
  so heavy edits never arm `refreshMap` (only `/map` does). Functional
  regression if f46's map.rs were kept wholesale.
- **`WorldMap::clear`** (`WorldMap.cs:249-258`) + the **`/clearmap` debug row**
  (`ToolkitDebugCommands.cs:497-503`, row 13 "Clear Map", 16-row World tab).
  f46's tab is back to 15 rows.
- **The drag's `!CaptureManager.Instance.UsingMap` gate** (`Main.cs:55342`,
  whole block skips, grab anchor untouched) + witness
  (`w4_e18_map_drag_capture_tests`). f46's drag is unconditional.
- **`Main.DrawMap`'s `!mapReady` gates** on both draw branches (`:55283`).
- **`section_manager` wiring**: boot construction (`app/mod.rs:3531`), net
  world-install rebuild (`netglue.rs`), `flow.rs` test fixture, `main.rs`
  `mod map_sections;`.

## 6. f46-only fidelity wins (salvage list, consolidated)

1. **Driver placement in the draw half** (§3) — C# `DoDraw`, not `DoUpdate`.
2. **`section_pixels` / `draw_to_map_section`** — the real
   `DrawToMap_Section` (`MapRenderer.cs:306-355`) pixel half + on-demand
   `ensure_map_texture` (`checkMap :322`), with witness
   `w4_f46_section_pixels_paint_revealed_opaque_and_dark_transparent`.
3. **`consume_update_map`'s paused fall-through** to the `ChangesQueued`
   else-arm (§3) — e18's `if paused { return }` skips it.
4. **`unlock_map` verbatim flag path + toggle/flat witness** (§4); delete
   e18's dead bulk-sweep body.
5. **XNA `Rectangle.Intersect` empty arm**: f46's `update()` sets
   `update_map = Some([0,0,0,0])` on an empty intersect
   (`LightingEngine.cs:269` assigns the Intersect result, which is
   `Rectangle.Empty`); e18 arms the **inverted** rect (`Some([x0,y0,x1-x0,
   y1-y0])` with negative w/h). No behavioural difference today (both
   consumers ignore the rect's contents), but f46 is the faithful value.
   Witness: `w4_f46_the_export_arms_update_map_even_on_an_empty_intersect`.
6. **Ctor `clear_map = true`** — `WorldMap::new` is the world-load
   construction, and `WorldGen.clearWorld` arms `Main.clearMap = true`
   (`WorldGen.cs:6982`, re-read: `:6981-6986` also zeroes `mapTime`/
   `updateMap`/`mapReady`/`refreshMap`, which both ctors already default).
   e18's ctor leaves `clear_map = false` (Main-static default) and relies on
   the `.map` load / `/map` to arm it — a fresh world with no `.map` file
   never arms it. Pixel-equivalent (a fresh texture is already transparent)
   but f46 is verbatim. Note e18's census doc claims the fresh-App reset is
   "the same end state" — true for pixels, not for the flag.
7. `sections_left()` accessor (§2) and the `budget_ms` test parameter (§3).

Both branches wire the **legacy seam's `updateMap` rect**
(`LegacyLighting.cs:432`) into `WorldMap` in `frame/draw.rs` — e18
`a19852bb`/`e5b12711`, f46 `f27a95f2` (a cosmetic restructure of the same
consume). Keep e18's version (it carries the `:429` bypasses-`QueueChange`
citation); no salvage needed.

## 7. Witness union

| Witness | e18 | f46 | Union |
| --- | --- | --- | --- |
| Spiral: grid dims / first-draw / cover-all / restart / ClearMapDraw / refresh double-count / TileLoaded truncation / quadrant pick | 8 (`map_sections::tests`) | 4 subsumed (`sections.rs::tests`) | keep e18's 8; port f46's `sections_left()` reads if the accessor lands |
| Driver: sweep reveal + option reset / refresh-hold / refresh consume / paused skip / load gate / mapEnabled drain gate | 6 (`frame/world.rs`) | — | keep all 6; **extend the paused witness** to assert the `ChangesQueued` else-arm DOES run while paused (f46 structure) |
| `/map` arming + clearMap head | `s1_l_3_the_reveal_row_unlocks_the_map` (extended) | `w4_f46_map_command_arms_the_flags_and_toggles_off` | keep both — f46's adds toggle-off + flat mode |
| `/clearmap` row | `w4_e18_the_clearmap_row_wipes_the_tiles_and_arms_the_refresh` + 16-row tab pin | — (15-row pin) | keep e18's; f46's 15-row pin is dropped with the row count |
| Statics: draw tail / load arms / Clear zeroes | 3 (`map.rs`) | — | keep |
| QueueChange 2048 threshold / load staged gate / export arms rect | 3 (`map.rs`) | — | keep |
| Reveal window: mapEnabled gate / scene box / shared clamp | 3 (`map.rs`, shared base) | same 3 (shared base) | already common |
| UsingMap drag gate | 1 (`app/mod.rs`) | — | keep |
| `DrawToMap_Section` pixels | — | `w4_f46_section_pixels_paint_revealed_opaque_and_dark_transparent` | **salvage** |
| Empty-intersect `updateMap` | — | `w4_f46_the_export_arms_update_map_even_on_an_empty_intersect` | **salvage** (with §6.5) |
| clearMap full-sweep + mapEnabled gate | covered via prepare consume | `w4_f46_clear_map_arms_the_full_sweep_and_gates_on_map_enabled` | adapt to whichever fn owns the consume in the union (e18: `prepare`; f46: `draw_to_map`) |
| refresh re-arm through consumer | driver witness | `w4_f46_refresh_map_rearms_the_spiral_through_the_consumer` | duplicate of e18's driver witness — drop |
| section-walk unlock drain | driver witness | `w4_f46_section_walk_unlocks_and_drains_the_spiral` | duplicate — drop |

## 8. KEEP/DROP per file (union merge plan — for the Lead's merge pass)

| File | e18 | f46 | Ruling |
| --- | --- | --- | --- |
| `crates/tr-app/src/map_sections.rs` (506 ln) | full `WorldSections` | absent | **KEEP e18**; add `sections_left()` accessor |
| `crates/tr-core/src/map/sections.rs` (349 ln) | absent | `MapDrawQueue` | **DROP** (subset duplicate); 4 witnesses subsumed |
| `crates/tr-core/src/map/mod.rs` | base | +3 ln `mod sections` | **KEEP e18** (drop the 3 lines with the file) |
| `crates/tr-app/src/map.rs` | statics + queues + clear + dead bulk `unlock_map` | driver fns + `unlock_map_mode` + flag-path `unlock_map` + `section_pixels` | **KEEP e18 base**; SALVAGE f46's `unlock_map` body (delete the bulk sweep), `section_pixels`/`draw_to_map_section`, empty-intersect arm, ctor `clear_map = true`; DROP f46's `draw_map_sections`/`section_walk_core`/`consume_update_map`/`draw_to_map` as a parallel driver (fold the paused fall-through + placement into e18's driver instead); DROP `WorldMap.sections`/`unlock_map_mode` fields (App owns both in the union) |
| `crates/tr-app/src/app/frame/world.rs` | `map_reveal_driver`/`map_reveal_sweep` + 6 witnesses | no driver | **KEEP e18**; fix the paused else-arm; replace the sweep's dirty-rect push with the salvaged section paint; move the CALL to the draw half |
| `crates/tr-app/src/app/frame/draw.rs` | `map_ready` gates + legacy seam | driver call site + seam restructure | **KEEP e18 gates + seam**; SALVAGE f46's driver call-site block (adapted to `app.map_reveal_driver(paused)`) |
| `crates/tr-app/src/app/mod.rs` | `section_manager`/`debug_unlock_map` fields + UsingMap gate + witness | fields/gate removed | **KEEP e18** |
| `crates/tr-app/src/debug_panel.rs` | 16 rows, `/clearmap`, `/map` clearMap head | 15 rows, `/map` via `unlock_map` | **KEEP e18** (optionally route the row through the salvaged `unlock_map`) |
| `crates/tr-app/src/app/flow.rs`, `netglue.rs`, `main.rs` | section_manager wiring | wiring removed | **KEEP e18** |
| `anchors.txt` | 3 e18 rows (map_sections / map_reveal_sweep / clear_map statics) | 2 f46 rows pointing at dropped files | **KEEP e18's 3 rows**; DROP f46's 2 rows; if the section-paint salvage lands, add ONE row for `draw_to_map_section` (never delete e18 rows to silence the canary) |
| `docs/ANALYSIS.md` | "map section grid + reveal queue" section (85 ln) | section absent | **KEEP e18**; append a short union note (placement move, paused else-arm fix, section paint, `/map` fn, ctor flag, empty-intersect) |
| `docs/research/14-map-minimap.md` | residual census (93 ln) | port-status row updates (9 ln) | **KEEP e18 census**; merge f46's port-status rows (they describe the same landed machinery under f46 names — rewrite the Rust locations to the union's) |
| `docs/PORT_MAP.md` / `docs/ROADMAP.md` | e18 wording | f46 wording | **union rewrite** of the single map row each: credit e18 (grid, driver, statics, census) + f46 (section paint, `/map` flag path, placement fix) |
| `docs/research/84-map-reconcile.md` | — | this file | **KEEP** (the reconcile record) |

**Merge-order note:** the union is a semantic merge of two branches that
never saw each other — per MAIN-RULES, resolve `map.rs`, `frame/world.rs`,
`frame/draw.rs`, `debug_panel.rs` by re-reading the C# (done above), never
`-X ours/theirs`. e18's tip `c6cf86bf` is the base; f46 contributes the six
salvage pieces, not its module shape.

## 9. Verification performed

Code-only pass, as tasked: no cargo, no tests, no graphify. Evidence:
`git show`/`git diff` of both tips against merge-base `c9841111` (16-file
diff enumerated and read in full), direct reads of the C# windows listed in
the header, and `git show 354b50d2:crates/tr-app/src/map_sections.rs`
confirming e18's `flag = false` init. All witnesses cited are
written-not-run on both branches (their own commit messages say so).

## 10. FINAL STATUS + merger checklist (2026-11-21 closeout)

> **§10 addendum (2026-11-21, shared-gap + dup pass — `aed75d0b`,
> `3f858937`, `a352b16e`):** the two shared pan gaps are CLOSED on the
> union (movement-key cancel in `update_map_input`, `resetMapFull` clear in
> `WorldMap::recentre`; the union carries `pan_target` so the overlay
> keep-one folds only step+arming). The minimap-key DUP is resolved: the
> g53 `a9774d2a` superset + the e18 `249a0c1b` alpha state are folded
> arm-by-arm into the union's `update_minimap_zoom_keys` (keep-one table
> below). **The e18 tip MOVED past `670c948d` to `75289a2db`** (the e18d
> final sweep landed 10 commits) — checklist item 2's re-fold is now
> mandatory, with the key/alpha half of `249a0c1b` ALREADY folded by this
> pass (drop its duplicate fns, see the table).

**STATUS: COMPLETE.** The union on `task/W4-f46-minimap-reveal-2` is the
branch the merger keeps; the separate e18/f46 versions are dropped. Chain
since the EXECUTED header: fold phase (`c06bc2fe`/`6dea2b66`/`0fc306bc` +
docs `af1bc80c`), then the closeout: `cc17880a` (e18 tip re-fetch — still
`670c948d` AT THAT TIME, zero post-fold commits), `08b749bd` (the
minimap remaining census: pan inertia PHANTOM, fullscreen zoom keys already
landed, the style-1 minimap zoom keys + the Tab style-cycle key ported with
5 witnesses), `8bb34f71` (the f29/e17 overlay composition cross-note,
research/14), then the shared-gap + dup pass (`aed75d0b`/`3f858937`/
`a352b16e` + this refresh). The e18d final-sweep contract stays published
in research/14 "fold status": any later e18 commit re-folds per the recipe
(fetch tip → cherry-pick → per-hunk onto the union seams).

### §10.1 The minimap-key keep-one — g53 `a9774d2a` × e18 `249a0c1b` × the union, arm-by-arm vs `Player.cs:24909-24965`

Three implementations of the same not-fullscreen block existed: the
union's `update_minimap_zoom_keys` (style-1 zoom only, `08b749bd`) +
`world_key`'s Tab arm; g53's `update_map_style_keys` (`a9774d2a`, the
superset: both styles + alphas + state + tint, but a `mapEnabled` gate the
C# lacks); e18's `update_minimap_keys` (`249a0c1b`, style-1 zoom+alpha +
`minimap_alpha` state + draw clamps in `prepare` + tint). **Verdict: the
union keeps ONE fn — `update_minimap_zoom_keys` — now carrying the g53c
superset arms + the e18 alpha state + the union's gate detail. The merger
drops g53's and e18's variants wholesale.** Per arm:

| Arm (C# anchor) | g53 `a9774d2a` | e18 `249a0c1b` | union BEFORE | FOLD (kept, `a352b16e`) |
| --- | --- | --- | --- | --- |
| outer gate `:24768` (chat/sign/chest/blockInput) | absent | `chat.open \|\| menu \|\| front` | absent | **e18's gate** (the port's live overlays) |
| `mapEnabled` gate | `!settings.map_enabled` return — **WRONG**, `:24911`/`:24930` check style only | absent (correct) | absent (correct, the closeout's gate-detail witness pins it) | **union detail kept: NO mapEnabled gate** |
| fullscreen-open guard `:24898`/`:24909` | `m.open` return | `m.open` return | caller's else-arm only | guard added in-fn (witnesses call it directly) |
| style-1 zoom ×1.025/×0.975 `:24913-24920` | `minimap_key_scale` + clamp after | `minimap_key_scale` unclamped, clamp in `prepare` | `ui::minimap::apply_key_zoom` (clamps per step, like the chrome buttons' `apply`) | **union's `apply_key_zoom`** (input-time clamp ≡ draw-time clamp: only keys/buttons mutate the axis and both clamp; the g53/e18 helper fns drop) |
| style-1 alpha ±0.015 `:24921-24928` | `map_key_alpha` + `map_alpha_clamp` | `minimap_alpha_step` + `clamp_minimap_axes` | absent (D17) | **g53's `map_key_alpha`/`map_alpha_clamp` names** (shared by both styles), state field from e18/g53 (identical: seed 1, `Main.cs:66753`) |
| style-2 zoom ×1.05/×0.95 `:24932-24939` | `map_key_scale` reuse | absent (D18 deferral) | absent | **g53 arm**, via the existing fullscreen `map_key_scale` |
| style-2 alpha ±0.015 `:24940-24947` | present | absent | absent | **g53 arm**, `overlay_alpha` seed 0.35 (`:66755`) |
| overlay scale clamp `:55550-55558` | `clamp(sw/ww, 16*zoom)` — **two flaws**: bare settings zoom ≠ `RenderZoom` (`= ForcedMinimumZoom × Clamp(GameZoomTarget,1,2)`, `Main.cs:61999` + `SpriteViewMatrix.cs:98`), and `f32::clamp` panics on an inverted range | absent | absent | **corrected fold**: caller passes `forced_minimum_zoom(screen) * zoom`; `overlay_scale_clamp` runs the two ifs in C# order (cap wins on inversion) |
| alpha clamps `:55523-55530`/`:55559-55566` | 0.01..1 | 0.01..1 (in `prepare`) | absent | `map_alpha_clamp` per update (same equivalence as the zoom clamp) |
| Tab cycle `:24949-24965` | in-fn `release_map_style` tick latch (+ App field + flow.rs init) | in-fn `map_style_prev` tick latch | `world_key`'s `MapAction::Style` press-edge arm (rebindable, fullscreen-guarded, 2 witnesses + anchor) | **union's `world_key` arm** — the port's press-edge convention; BOTH tick-latch variants + their App fields drop (no `release_map_style`/`map_style_prev` on the union) |
| tile tint `Color(b×4)` `MapRenderer.cs:438`, `b=(byte)(255*alpha)` `:55532` | `draw_minimap` tint | same | `[1.0;4]` | **folded** (identical in both) |
| backdrop live alpha `MinimapFrame.cs:182` | `draw_background(m.minimap_alpha)` | same | hardcoded 1.0 | **folded** |
| layer-walk opacity `:55703` (`b` arg) | left 1.0 — **missed** | `m.minimap_alpha` | 1.0 | **e18's detail** folded (g53's miss corrected) |
| state fields | `minimap_alpha`/`overlay_scale`/`overlay_alpha` | `minimap_alpha` only | none | ONE set on `WorldMap` (seeds 1/2.5/0.35, `Main.cs:66753-66755`) |

Witnesses folded/adapted (written not run): the union's 5 closeout
witnesses stay (the gate witness's "style 2 leaves the minimap scale
alone" still holds — style 2 steps the OVERLAY axis); new
`app::w4_f46_minimap_zoom_key_tests::w4_f46_alpha_keys_step_the_minimap_alpha_with_the_clamp`
+ `w4_f46_style_two_keys_step_the_overlay_scale_and_alpha` (the g53
witness's 2.5-seed steps would be eaten by the `sw/ww` floor on the small
fixture — the fold parks the scale above the floor first) +
`map::tests::w4_f46_alpha_and_overlay_clamps_match_the_draw_side`. g53's 2
witnesses and e18's 4 key witnesses drop with their fns (the Tab halves
are already pinned by `app::loop::tests::w4_f46_*`).

**e18 re-fold note (checklist item 2, now live):** tip `75289a2db` =
`670c948d` + 10 final-sweep commits. `249a0c1b` (P1 keys) is superseded by
this fold — when re-folding, SKIP its duplicate fns/fields/witnesses per
the table above and keep only what the union lacks (nothing: all its arms
are folded). The other nine (`0bd346e78` docs addendum, `4ad0f041`
TryOpeningFullscreenMap, `029154fe2` Load catch arm, `2fa072d01` mapDelay,
`8ec77f0f8`+`2e1f97c83` draw-head dialog close, `0d99d968c` draw-tail chat,
`0002fa1fe`+`75289a2db` docs) are NOT folded by this pass — they re-fold
per the research/14 recipe at merge time.
**SUPERSEDED — EXECUTED by the final-retire pass (same day, §10.2 below):
all nine are now folded; checklist item 2 is CLOSED.**

**Merger checklist (in order):**

1. **Take THIS branch's union only** — never a plain e18 or f46 merge. The
   §8 KEEP/DROP table is the per-file ruling; the driver is
   `frame/world.rs::map_reveal_draw` called from `frame/draw.rs` on the
   DoDraw half ahead of `WorldMap::prepare` (NOT an update-half call — the
   S1-B-8 orchestrator pin walks the frozen phase list and the driver is
   deliberately absent from it).
2. ~~**Re-fetch `origin/task/W4-e18-minimap-reveal-queue` at merge time.** The
   tip HAS moved: `75289a2db` (the e18d final sweep, 10 commits past
   `670c948d`). Fold each new commit per research/14's recipe BEFORE
   merging — EXCEPT `249a0c1b`'s key/alpha arms, which the §10.1 fold
   already landed (skip its duplicate fns/fields/witnesses per the table).~~
   **EXECUTED (final-retire pass, same day — §10.2):** the e18 tip re-fetched,
   still `75289a2db`; all nine non-superseded commits folded per-hunk onto
   the union (`736c86ce`/`5566bf25`/`cd24857e`/`7d050b7c`/`b767ef91` + the
   docs). The merger only re-checks that e18 has not advanced past
   `75289a2db`; if it has, the recipe re-applies for the new commits only.
3. **The overlay siblings are a SEPARATE keep-one, not part of this union:**
   `origin/task/W4-f29-map-overlay-2` (`e18bf6dd`) and
   `origin/task/W4-e17-map-overlay-layers` (`8500c3e5`, revived) port the
   SAME nine overlay pieces twice. Take ONE implementation (research/14
   "W4-f46 × f29/e17 overlay composition" lists the seams: the draws insert
   before the hover text leaving the union's close/cursor tail last; the
   head icon re-homes to `:55931` with the 255-loop; the pan state lands
   twice — f29's `update_map_input` step vs e17's `prepare` lerp). The
   two shared gaps are ALREADY CLOSED on the union (`aed75d0b`/`3f858937`):
   the movement-key pan cancel (`Player.cs:24776-24791`, each held row
   clears `pan_target`) and the `resetMapFull` pan clear (`Main.cs:55418`,
   `WorldMap::recentre`). The union carries `pan_target: Option<[f32;2]>`
   (f29's representation) — the overlay pick folds only its lerp STEP +
   pylon-click ARMING hunks; its duplicate pan field/cancel code drops
   (e17's bool+end pair converts 1:1 if e17 wins).
4. **Anchors:** the union's rows are `map_sections.rs::get_next_map_draw`,
   `frame/world.rs::map_reveal_sweep_fields`, `map.rs::draw_to_map_section`,
   `map.rs::clear_map`, `ui/chrome.rs::fullmap_close_button_pos`,
   `ui/cursor.rs::draw_thick`, `ui/minimap.rs::apply_key_zoom`,
   `app/loop.rs`'s `MapAction::Style` arm, plus the shared-gap/dup rows
   `map.rs::pan_target`, `map.rs::recentre`, `map.rs::map_key_alpha` —
   plus the overlay pick's own rows.
   Never delete rows to silence the canary; the f46 pre-union rows pointing
   at dropped files were already removed by the base adoption.
5. **Run the catalog verification on the merged state** (this closeout was
   code-only as tasked — every witness on the branch is written-not-run):
   `cargo test --workspace -- --skip migrates_version_2_kinds_to_item_ids`,
   judge by the `test result: ok` lines. The new witnesses to watch:
   `ui::minimap::tests::w4_f46_key_zoom_*`,
   `app::w4_f46_minimap_zoom_key_tests` (incl. the alpha + style-2 pair),
   `app::w4_e18_map_drag_capture_tests::w4_f46_movement_keys_cancel_*`,
   `map::tests::w4_f46_the_reset_clears_*` +
   `w4_f46_alpha_and_overlay_clamps_*`, `app::loop::tests::w4_f46_*`, and
   the folded e18/f46 sets (§7).
6. **Docs are already union-shaped:** research/14 (censuses + fold status +
   composition note), this file, PORT_MAP's map rows, ROADMAP row 14,
   ANALYSIS's union note. The overlay pick brings its own doc updates —
   resolve the research/14 append conflicts by keeping both sections (the
   e18/f29/e17 censuses describe different passes, not rivals).

### §10.2 The final-retire re-fold — EXECUTED (2026-11-21, checklist item 2 closed)

The §10-item-2 "re-fold at merge time" instruction is executed NOW on this
branch. `origin/task/W4-e18-minimap-reveal-queue` re-fetched: tip **still
`75289a2db`** (`670c948d` + the 10 e18d final-sweep commits). Per-hunk fold
results, in e18 commit order:

| e18 commit | Piece | Union commit | Fold notes |
| --- | --- | --- | --- |
| `249a0c1b` (P1 keys) | minimap keys + alpha | ALREADY folded (`a352b16e`) | SUPERSEDED per §10.1 — its `update_minimap_keys`/`minimap_key_scale`/`minimap_alpha_step`/`clamp_minimap_axes`/`map_style_prev` + 4 witnesses + anchor row DROP; the union's `update_minimap_zoom_keys` carries the g53 superset + the e18 alpha state. Its anchor row is NOT copied. |
| `4ad0f041` (P2) | `TryOpeningFullscreenMap` | `736c86ce` | `loop.rs::try_open_fullscreen_map` + the M-key/Escape `PlaySound(11)` closes + the icon-row helper routing + style-icon `PlaySound(12)` + the stale audio comment retired; the union's rebindable `MapAction::Full` row kept; `:29113` `MapPylonTile` clear noted as the overlay pick's seam (the union's `set_open`→`recentre` already clears `pan_target`); 2 witnesses + anchor. |
| `0bd346e7` (docs) | research/84 addendum on the e18 side | this §10.2 + §10.3 | Their addendum was written against the OLD f46 blob (`38c283a0`) — its §10.1 close-button/thick-cursor pieces were ALREADY folded by the union's fold phase (`c06bc2fe`/`6dea2b66`), its §10.2 keys piece superseded by §10.1 above, its §10.3 "7 e18 anchor rows" count superseded (the `update_minimap_keys` row drops). The surviving content is recorded here + the checklist amendment; no byte-copy needed. |
| `029154fe` (P3) | Load catch arm | `5566bf25` | `map.rs::load`'s `Ok(false)` fold splits on `path.exists()`: `.bad` copy + `clear()`; the union's `clear_edges`/staged-statics load body untouched; 1 witness + anchor. |
| `2fa072d0` (P4) | mapDelay skip | `cd24857e` | `WorldMap::map_delay` (ctor 2) + `map_delay()`/`reset_map_delay()` + `update`'s head gate + `enter_loaded_world` re-arm; 1 witness + anchor; **6** witnesses re-pinned with `map_delay = 0` (e18's 5 + the union-only empty-intersect witness `w4_f46_the_export_arms_update_map_even_on_an_empty_intersect`, which e18's branch never had — its 5-pin list would have left that witness red). |
| `8ec77f0f` + `2e1f97c8` (P5 + fix) | draw-head dialog close | `7d050b7c` (folded as one) | `close_dialog_behind_fullmap` wired at `draw_region`'s head BEFORE the pylon scan (the fix's final position, not P5's original after-scan site); 1 witness + anchor. |
| `0d99d968` (P6) | draw-tail chat | `b767ef91` | `ui::draw_chat` made pub + called at the fullscreen branch tail after the union's head icon; the union's hover→close→cursor→head tail order kept; anchor, no new witness. |
| `0002fa1f` (docs) | research/14 doc-sweep | research/14 "W4-e18 final sweep" section (union-named) + the port-status/D17-D22/seed-row refresh | The stale MISSING rows (alpha/overlays/BuildMapAtlas/M-toggle) rewritten to the union's landed state; D19 marked closed-across-branches with the keep-one PENDING caveat; D22 partial; the census tables + Unity-potion seed carried. |
| `75289a2d` (docs) | ANALYSIS + PORT_MAP rows | ANALYSIS "The fullscreen open/close arms + …" section + 3 PORT_MAP rows | The e18-named minimap-keys PORT_MAP row NOT copied (the union's `a352b16e` row already covers that block under the union's names). |

**Post-fold state: the e18 branch is fully absorbed into this union.** The
merger takes THIS branch and drops `task/W4-e18-minimap-reveal-queue`
wholesale; item 2's merge-time duty shrinks to a tip re-check (if e18
advanced past `75289a2db`, re-fold only the new commits per the recipe).
All re-folded witnesses written not run (code-only, as tasked).
