# 84 — Map reveal dup-reconcile: W4-e18 vs W4-f46 (2026-11-21, code-only)

> **STATUS: EXECUTED (2026-11-21, salvage-exec on `task/W4-f46-minimap-reveal-2`).**
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

---

## 10. ADDENDUM (2026-11-21, W4-e18 final sweep — the e18 tip moved past c6cf86bf)

> Written on the e18 branch (`task/W4-e18-minimap-reveal-queue`), which now
> carries this file as a cross-note copy of f46's `38c283a0` blob
> (byte-identical, `git hash-object` = `317ff706`). The reconcile above was
> written against e18 @ `c6cf86bf`; **the e18 tip is now `4ad0f041`** and the
> union's e18 side must include everything below. The f46 side (`f27a95f2` +
> this doc) is unchanged; the merge-base is still `c9841111`.

### 10.1 The two first-half pieces BEYOND c6cf86bf (union must include)

The reconcile's §8 KEEP/DROP table predates these; they are e18-only,
unclaimed by f46, and **KEEP e18** wholesale:

1. **The fullscreen close button** — e18 `ebd54f66`
   (`Main.cs:56014-56036`): `ui/chrome.rs::fullmap_close_button_pos`/
   `_appearance`/`draw_fullmap_close_button` (the `(10, screenHeight-40)`
   inclusive 32x30 box, `Map_0` grey-130 / `Map_4` white-255 hover flip),
   `app/clicks.rs::handle_fullmap_close` (`PlaySound(10)` +
   `mapFullscreen=false`), wired after the ping detector in the click
   cluster (`frame/world.rs`, the `:55486` < `:56024` order) and after the
   hover text in the draw (`frame/draw.rs`). 5 witnesses
   (`ui::chrome::tests::w4_e18_*` ×2, `app::clicks::w4_e18_fullmap_close_tests`
   ×3). f46 has none of it — no conflict, pure addition.
2. **The fullscreen vanilla cursor + thick border** — e18 `254f309b`
   (`Main.cs:56037-56040` + `DrawThickCursor` `:48064-48134` +
   `DrawCursor` `:48046-48048`): `ui/cursor.rs::draw_thick`/
   `thick_cursor_bonus`/`draw_with_bonus` (the 4 `Cursors[11]` rings at
   origin `(2,2)`, scale `cursorScale*1.1`, `MouseBorderColor` tint, the
   `(2,2)` bonus into the shadow/sprite), `Cursor_11` packed in
   `assets.rs`, the OS-pointer hide over the map in `app/frame.rs`
   (`FocusHelper.cs:160-163`), wired at the fullscreen draw tail
   (`frame/draw.rs`). 1 witness
   (`ui::cursor::tests::w4_e18_thick_cursor_gate_and_bonus`). f46 has none
   of it — pure addition. **`assets.rs` is a new union hot file**: f46 does
   not touch it, so e18's `Cursor_11` row keeps verbatim.

The census-close doc commit `670c948d` (2 anchor rows + PORT_MAP row +
ANALYSIS note + the research/14 first-half census section) rides along; its
anchor rows (`fullmap_close_button_pos`, `draw_thick`) are e18 rows the union
keeps (§8's "KEEP e18's 3 rows" becomes **5 rows**).

### 10.2 The final-sweep pieces beyond the census close (also e18-only)

3. **The minimap key rows + alpha** — e18 `249a0c1b`
   (`Player.cs:24909-24966`): `map.rs::minimap_key_scale` (×1.025/0.975,
   `:24913-24920`) / `minimap_alpha_step` (±0.015, `:24921-24928`),
   `app/mod.rs::update_minimap_keys` + the `map_style_prev` latch (the Tab
   style cycle `:24949-24965`, `PlaySound(12)`), `WorldMap::minimap_alpha`
   (`Main.cs:66753`) + `clamp_minimap_axes` (`Main.cs:55515-55530`) called
   from `prepare`, the tile-pass `(byte)(255*alpha)` tint (`:55532` →
   `MapRenderer.cs:438`), the backdrop + layer-walk alpha in
   `frame/draw.rs`, the per-update call in `frame.rs::frame_head`. D17
   closed. 4 witnesses. **Union interaction:** `map.rs`'s `WorldMap` gains
   `minimap_alpha` + the two clamp lines in `prepare` — f46's `map.rs`
   salvage (§6) does not touch either site, so both sides compose; the
   `prepare` clear-consume (e18) and the `draw_to_map` consume (f46) stay
   the §3/§8 adjudication, unchanged.
4. **`TryOpeningFullscreenMap`** — e18 `4ad0f041`
   (`Player.cs:29103-29119`): `app/loop.rs::try_open_fullscreen_map` (the
   M-key open arm closes the inventory, clears the talk-NPC latch + chat
   text, `PlaySound(10)`; scale 2.5 + recentre ride `set_open`), the close
   arms' `PlaySound(11)` (`:24973`/`:4976`), the inventory map-icon row
   (`Main.cs:41095-41120`) routed through the helper + the style icons'
   `PlaySound(12)`. 2 witnesses. f46 touches neither `loop.rs` nor
   `clicks.rs` — pure addition.

### 10.3 Updated union inputs

- **e18 tip: `4ad0f041`** (chain `c6cf86bf` → `ebd54f66` → `254f309b` →
  `670c948d` → `249a0c1b` → `4ad0f041`; 9 unique commits over the
  merge-base, was 5).
- **f46 tip: `38c283a0`** (the reconcile doc commit; code tip `f27a95f2`
  unchanged).
- §8's per-file table stands with these amendments: `ui/chrome.rs`,
  `ui/cursor.rs`, `assets.rs`, `app/loop.rs` join the KEEP-e18 column
  (f46 has no counterpart); `app/clicks.rs` KEEP e18 (close button +
  ping-order wiring + the icon-row helper routing); `app/frame.rs` KEEP
  e18 (OS-pointer gate + the `update_minimap_keys` call); `anchors.txt`
  keeps **7 e18 rows** (the 3 census rows + close button + thick cursor +
  `update_minimap_keys` + `try_open_fullscreen_map`), drops f46's 2.
- The salvage list (§6) is unchanged — none of the new e18 pieces overlaps
  f46's six salvage items. The `map.rs` merge now also carries
  `minimap_alpha`/`clamp_minimap_axes`/`minimap_key_scale`/
  `minimap_alpha_step` from e18's side.

### 10.4 Verification (this addendum)

Code-only, as tasked: no cargo, no tests, no graphify. Evidence:
`git diff c9841111..670c948d --stat` (19 files) + `git show ebd54f66/
254f309b/249a0c1b/4ad0f041` read in full; the copied blob's
`git hash-object` equals f46's `317ff706`. All witnesses cited are
written-not-run (code-only mode).

### 10.5 Re-fold status (2026-11-21, e18 RETIRE phase — f46 latest re-read)

**f46 tip at this read: `c50e1b00`** (closeout item 4; chain past the fold:
`af1bc80c` docs → `cc17880a` e18 re-fetch → `08b749bd` closeout census +
zoom/Tab ports → `8bb34f71` overlay cross-note → `c50e1b00` §10 FINAL).
Read in full from git objects: f46's research/84 (352 ln) + research/14
(625 ln) + `git show 08b749bd` (the closeout code).

**f46's active gaps at this read (what the re-fold must bring):**

1. **The e18-tip re-fetch claim is STALE.** `cc17880a` records "still
   `670c948d`, zero post-fold commits" — the e18 tip is now **`75289a2d`**,
   **10 commits past `670c948d`** (the final sweep P1–P6 + three doc
   commits). f46's own merger-checklist step 2 (re-fetch at merge time,
   fold each new commit per the research/14 recipe) therefore FIRES. The
   exact fold-list is §10.6.
2. **D17 (minimap alpha) is still open on the union.** f46's closeout
   explicitly left the alpha keys with D17/D18 ("porting the keys without
   the alpha state would be a no-op"); e18's `249a0c1b` carries the state
   + keys + draw-side clamps + tint + backdrop/layer alpha. The fold
   closes D17 on the union.
3. **The five non-key residuals are absent on the union:**
   `TryOpeningFullscreenMap` (`4ad0f041`), the `Load` catch arm
   (`029154fe`), `mapDelay` (`2fa072d0`), the draw-head dialog close
   (`8ec77f0f` + `2e1f97c8`), the draw-tail chat (`0d99d968`). None of
   f46's closeout touches those seams — pure additions at fold time.
4. **The zoom/Tab keys are now a THREE-way duplicate.** f46's `08b749bd`
   ported `apply_key_zoom` + `update_minimap_zoom_keys` + the `world_key`
   `MapAction::Style` arm; e18's `249a0c1b` ported `update_minimap_keys`
   (zoom + alpha + Tab latch); **g53's `a9774d2a`**
   (`origin/task/W4-g53-ui-achievement-screen`, read via `git show`)
   ported `update_map_style_keys` (zoom + alpha + the style-2 overlay
   state/keys + Tab latch). f46's docs do not mention g53 — the 3-way
   adjudication is unpublished on the union side; the e18-arm fidelity
   notes are §10.7. The merger keeps ONE input implementation.
5. **Unchanged, not e18's to close:** the f29/e17 overlay keep-one
   (f46 §10 step 3) and its two shared C# gaps (the movement-key pan
   cancel `Player.cs:24776-24791`, f29's missing `resetMapFull` pan clear
   `Main.cs:55418`).

### 10.6 Exact fold-list — the 10 e18 commits past `670c948d`

Fold-source tip **`75289a2d`** (the retire-phase commits that follow are
docs/comments-only and carry no fold obligation beyond their notes).
Recipe per f46's research/14 "fold status": fetch the e18 tip →
cherry-pick each commit below IN ORDER → resolve per-hunk onto the union
seams (the `map_reveal_draw` field-borrow driver on the DoDraw half, the
`map.rs` statics/queues skeleton, `handle_fullmap_close` after the ping,
the close/cursor tail inside the fullscreen draw branch). Per-commit:

| # | Commit | Files | Fold notes |
| --- | --- | --- | --- |
| 1 | `249a0c1b` minimap keys + alpha (P1) | `app/mod.rs` (+161: `update_minimap_keys`, `map_style_prev`, ctor, 2 witnesses), `map.rs` (+98: `minimap_key_scale`/`minimap_alpha_step`/`minimap_alpha`/`clamp_minimap_axes`/tint, 2 witnesses), `app/frame.rs` (+5: the `frame_head` call), `app/flow.rs` (+1 fixture), `frame/draw.rs` (backdrop + layer alpha), `anchors.txt` (+1), ANALYSIS/PORT_MAP | **The 3-way conflict.** `map.rs`/`frame/draw.rs`/`flow.rs`/anchors hunks apply cleanly (the union has no `minimap_alpha`, no clamps in `prepare`, no backdrop alpha). `app/mod.rs` + the tick site + the Tab cycle collide with f46's `08b749bd` (`update_minimap_zoom_keys` in `world_sim/player.rs`'s else-arm, `apply_key_zoom` in `ui/minimap.rs`, the `world_key` Style arm) and g53's `a9774d2a` (`update_map_style_keys`). Resolve per §10.7 — keep ONE input fn; the alpha state/clamps/tint/backdrop/layer-alpha hunks are e18-unique and keep wholesale. If the e18 input fn wins, DROP f46's `apply_key_zoom`/`update_minimap_zoom_keys`/`world_key` Style arm + their 5 witnesses + 2 anchor rows as dups (never delete e18 rows); if f46's wins, extend it with the alpha rows + the latch fix (§10.7 row 6) and re-home the clamps draw-side. |
| 2 | `4ad0f041` TryOpeningFullscreenMap (P2) | `app/loop.rs` (+73: `try_open_fullscreen_map`, the M-key arm, the close arms' sound 11), `app/clicks.rs` (+65: the icon-row helper routing + style-icon sound 12), `anchors.txt` (+1) | Pure addition — f46 touches neither fn. The only `loop.rs` neighbour is f46's `world_key` Style arm (row 1's resolution decides whether it stays); the hunks are disjoint. |
| 3 | `0bd346e78` research/84 cross-note copy | `docs/research/84-map-reconcile.md` (+426) | **DROP the blob-copy hunk** — f46 owns this file and has evolved past the copied `317ff706` blob (`af1bc80c`/`c50e1b00`). The §10.1–10.4 facts are already covered: the two first-half pieces are folded (`c06bc2fe`/`6dea2b66`/`0fc306bc`), the final-sweep pieces are this fold-list. Keep this §10.5–10.8 retire record (append, no conflict). |
| 4 | `029154fe` Load catch arm (P3) | `map.rs` (+44: the `load` `Ok(false)` split on `path.exists()` + 1 witness), `anchors.txt` (+1) | Clean apply — the union's `load` is the e18 base's (full `WorldMap.cs:211-214` arms); the hunk lands inside the same `if !load_map_file(...)` head. |
| 5 | `2fa072d0` mapDelay (P4) | `map.rs` (+83: `map_delay` field/ctor/gate/`reset_map_delay` + 1 witness + 5 witness re-pins), `app/mod.rs` (+9: the `enter_loaded_world` re-arm), `anchors.txt` (+1) | Clean apply — the union's `update` head + ctor are the e18 base's; the `mod.rs` re-arm site is disjoint from f46's zoom fns. The 5 re-pins (`map_delay = 0`) target witnesses the union kept verbatim. |
| 6 | `8ec77f0f` draw-head dialog close (P5) | `frame/draw.rs` (+71: `close_dialog_behind_fullmap` + 1 witness), `anchors.txt` (+1) | The fn + witness apply cleanly; the call-site hunk lands at `draw_region`'s head — the union's driver call sits AHEAD of `prepare` in the same region, a different site. Keep the C# order: the close runs before `DrawMap` (`Main.cs:62150-62153` < `:62155`). |
| 7 | `2e1f97c8` P5 fix | `frame/draw.rs` (+5/-4) | Moves the row-6 call ahead of the pylon scan; applies after row 6. |
| 8 | `0d99d968` draw-tail chat (P6) | `frame/draw.rs` (+10: the `ui::draw_chat` call), `ui/mod.rs` (+4/-1: `draw_chat` pub), `anchors.txt` (+1) | Place the chat call AFTER the union's close/cursor/head-icon tail — `Main.cs:62160` runs after the whole `DrawMap` (whose internal tail is the close button `:56014` + cursor `:56037`), so chat-after-tail is the C# order. `draw_chat` pub-ification is disjoint from f46. |
| 9 | `0002fa1f` research/14 doc-sweep | `docs/research/14-map-minimap.md` (+125/-27) | Append-conflict resolution: keep BOTH sides' sections (f46's fold-status/closeout-census/composition notes + e18's remaining-census/final-sweep sections describe different passes). The port-status rows + D17/D18/D19/D22 rows collide: rewrite to the union names per f46's §8 rule, with D17 CLOSED once row 1 folds, D18's switcher half landed (f46 closeout), D22's mapDelay half closed (row 5). The retire-phase census close (§10.8's D18/D22 rulings) supersedes the stale "style-0 retro layout" wording. |
| 10 | `75289a2d` ANALYSIS + PORT_MAP rows | `docs/ANALYSIS.md` (+56), `docs/PORT_MAP.md` (+3) | Append keeping both (the ANALYSIS union note + the e18 final-sweep notes are different paragraphs). PORT_MAP rows citing `update_minimap_keys`/`map_reveal_driver` re-home to whichever names row 1's resolution + the union driver keep. |

Anchor accounting after the fold: the union's 8 rows (f46 §10 step 4) +
e18's `try_open_fullscreen_map`, the corrupt-`.map` witness row, the
`map_delay` witness row, `close_dialog_behind_fullmap`, `draw_chat` =
**13 rows**, plus row 1's winner (`update_minimap_keys` XOR
`apply_key_zoom` — already counted in the 8). Never delete rows to
silence the canary; the dropped dup's rows go only with a §10.7 ruling.

### 10.7 The 3-way zoom adjudication — e18-arm C#-fidelity notes

Ground truth re-read for this note: `Player.cs:24768-24989` (the input
block + its gates), `Main.cs:55515-55532` (the style-1 draw clamps +
tint), `Main.cs:55548-55566` (the style-2 clamps), `MapRenderer.cs:438`
(the tile tint). Arms: **e18** `249a0c1b` (`update_minimap_keys`,
`frame_head` per-update), **f46e** `08b749bd` (`update_minimap_zoom_keys`,
`world_sim/player.rs` else-arm + `world_key` Style), **g53c** `a9774d2a`
(`update_map_style_keys`, `world_sim/player.rs` else-arm).

| Row | C# | e18 (this arm) | f46e | g53c |
| --- | --- | --- | --- | --- |
| 1. Held keys | `:24770` `Triggers.Current.CopyInto` — the zoom/alpha rows are HELD, per `Player.Update` | held (`self.keys`) per update in `frame_head` beside `update_view_zoom_keys` ✓ | held per `update_player` tick ✓ | held per `update_player` tick ✓ |
| 2. Tick site | `Player.Update`'s input region (`:24768`) | `frame_head` (per-frame, outside the sim step) — placement deviation, same 60 Hz lockstep cadence | `update_player` tail — the C#-shaped site ✓ | same ✓ |
| 3. Block gates | `:24768` `!drawingPlayerChat && !editSign && !editChest && !blockInput`; `:24898` not-fullscreen; `:24911` `mapStyle == 1` — **NO `mapEnabled` check** | chat/menu/front gates + `!map.open` + style 1 ✓ (menu/front = the port's `blockInput` analog) | style 1 in-fn; not-fullscreen via the call-site else; chat/menu/front eaten upstream ✓; no `mapEnabled` gate ✓ (witnessed) | **adds `!self.settings.map_enabled` — a gate the C# does not have** (f46's witness pins the scale moving with the map dark); sign/chest editors unmodelled (all three arms) |
| 4. Zoom constants | `:24915/:24919` ×1.025/×0.975 | `minimap_key_scale` literals ✓ | `apply_key_zoom` via `ZOOM_IN/OUT_MULTIPLIER` ✓ | `minimap_key_scale` via the same constants ✓ |
| 5. Alpha ±0.015 | `:24923/:24927` `mapMinimapAlpha ±= 0.015f`, unclamped in the input | `minimap_alpha_step` ±0.015 unclamped ✓ + `WorldMap::minimap_alpha` (default 1, `Main.cs:66753`) ✓ | **absent** (left with D17) | `map_key_alpha` ±0.015 ✓ + the same field/default ✓ |
| 6. Tab cycle | `:24949-24965` fires in EVERY non-fullscreen style; `releaseMapStyle` latch — held frames never re-fire, the release frame re-arms | `map_style_prev` press-edge latch, every style ✓; held Tab does NOT re-cycle (witnessed) | `world_key` press edge — **winit delivers OS auto-repeat as separate `Pressed` events (`loop.rs:648-651`), so a HELD Tab re-cycles at the OS repeat rate**; the C# latch fires once per physical press. Fidelity miss | `release_map_style` latch — the verbatim C# structure (`if bound { if release {cycle} release=false } else {release=true}`) ✓ |
| 7. Clamp placement | the clamps are DRAW-side only (`Main.cs:55515-55530`, inside the style-1 branch; scale 0.2..3, alpha 0.01..1) | `clamp_minimap_axes` in `prepare` (draw half) ✓ — runs every in-world frame rather than only style-1 draws; idempotent projection, behaviourally equal (the value only leaves range during style-1 held frames, and both clamp the same frame) | `apply_key_zoom` clamps INPUT-side (deviation, same equivalence argument); no alpha clamp (no alpha) | clamps INPUT-side right after the steps (same deviation); also ports the style-2 clamps (`overlay_scale_clamp` `Main.cs:55550-55558`, `map_alpha_clamp` `:55559-55566`) ✓ |
| 8. Tile tint | `:55532` `b = (byte)(255*alpha)` → `MapRenderer.cs:438` `Color(b,b,b,b)` | `(255.0*a) as u32 as f32/255` ✓ (truncation = the byte cast; post-clamp range) | **absent** (tint stays 1.0) | `as u8` ✓ (saturating vs C# unchecked wrap — unreachable post-clamp) |
| 9. Backdrop + layer alpha | `MinimapFrame.DrawTo`'s backdrop reads `b`; the `:55703` layer walk takes `b` | backdrop ✓ AND the layer walk ✓ (`draw.rs:1504`/`:1558`) | absent | backdrop ✓; **the layer-walk alpha hunk is absent** (its `draw.rs` diff is the backdrop only) |
| 10. Style-2 rows | `:24930-24947` overlay zoom ×1.05/×0.95 + alpha ±0.015; state `Main.cs:66754-66755` (2.5/0.35) | absent (D18) | absent | **ported** — `overlay_scale`/`overlay_alpha` state + keys + clamps (the draw half stays absent everywhere) |

**e18-arm summary for the merger:** this arm is the only one with the
alpha state + keys + DRAW-side clamps + tile tint + backdrop AND
layer-walk alpha + a held-Tab-safe latch — it closes D17 outright. Its
two recorded deviations: the `frame_head` tick site (row 2) and the
every-frame `prepare` clamp (row 7, benign). **f46e** is the narrowest
(zoom + Tab only) and carries the auto-repeat Tab deviation (row 6).
**g53c** is the superset on style-2 state (row 10, salvage it when D18's
draw half lands) but adds the non-C# `mapEnabled` gate (row 3), clamps
input-side (row 7) and misses the layer alpha (row 9). Recommended
union: e18's `update_minimap_keys` as the keeper (re-home the call to
`update_player`'s else-arm if the C#-shaped site is preferred — the fn
body is site-agnostic), drop f46e's three pieces as dups, hold g53c's
overlay-state hunk as the D18 salvage. The ruling is the merger's; these
are the facts.

### 10.8 Verification (this retire note)

Code-only, as tasked: no cargo, no tests, no graphify. Evidence:
`git fetch origin task/W4-f46-minimap-reveal-2` + `git ls-remote` (tip
`c50e1b00`), f46's research/84 + research/14 read in full from git
objects, `git show 08b749bd`/`a9774d2a`/`249a0c1b` read in full,
`git log 670c948d..75289a2d` (the 10-commit list), and the C# windows
cited in §10.7 re-read directly. All witnesses cited are
written-not-run (code-only mode).
