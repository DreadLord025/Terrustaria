# 84 — Map reveal dup-reconcile: W4-e18 vs W4-f46 (2026-11-21, code-only)

> **STATUS: EXECUTED (2026-11-21, salvage-exec on `task/W4-f46-minimap-reveal-2`).**
> The §8 union landed on this branch: e18 `c6cf86bf`'s skeleton adopted
> verbatim (base commit `0bc3414d`), then the six f46 salvages + the
> `sections_left()` accessor committed per piece. The merger keeps THIS
> branch's union and drops the separate e18/f46 versions. Execution notes:
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
