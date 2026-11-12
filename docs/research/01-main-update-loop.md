# 01 — Main update loop / time / camera / input

> Wave 1 · researched 2026-11-12 · sources: `decompiled/Terraria/Main.cs`,
> `crates/tr-app/src/app/{loop,frame}.rs`, `crates/tr-app/src/world_sim/{step,clock,camera}.rs`,
> ROADMAP rows M1.4-BG-21 / M1.4-BG-22 + `docs/ANALYSIS.md`.

## Scope

The C# frame driver is `Main.DoUpdate` (`Main.cs:16828-17292`, ~465 lines) plus its
world half `DoUpdateInWorld` / `DoUpdateInWorld_Inner` (`Main.cs:17726-17822`) and the
`UpdateWorld_*` family (`Main.cs:17824-18157`). `Main.cs` itself is ~67k lines and is
only ever read by range here.

The Rust side is `App::frame` (`crates/tr-app/src/app/frame.rs:82-1760`) — one
per-frame function that mixes XNA-`DoUpdate`-style per-frame work (UI clock, disco,
autosave, audio listener) with a world branch, plus the fixed-step machinery
`FixedStepper` (`world_sim/step.rs:9-63`) with one stepper per subsystem
(`App` fields at `app/mod.rs:2475-2879`): `player_step`, `npc_step`, `item_step`,
`projectile_step`, `liquid_step`, `waterfall_step`, `wiring_step`, `te_step`,
`growth_step`, `world_tick_step`, `sunflower_wind_step`. Responsibility: advance the
world simulation on a 60 Hz accumulator while menus and the front end keep their own
per-frame update.

## Key entities

- `Terraria/Main.cs::DoUpdate` (16828-17292) — the frame entry point; runs in the
  menus *and* in the world; returns early on `gameMenu` (17166-17175) and on
  `CanPauseGame()` (17194-17199).
- `Terraria/Main.cs::DoUpdateInWorld_Inner` (17744-17822) — the ordered world pass.
- `Terraria/Main.cs::UpdateWorld_*` (17824-18157) — players / spawn / NPCs / gores /
  projectiles / items / dust / floating text / time / worldgen+invasion / net.
- `Terraria/Main.cs::UpdateTime` (64822+) + `UpdateTimeRate` (6199-6218) — day-night.
- `Terraria/Main.cs::DoDraw_UpdateCameraPosition` (62590-62649) — camera follow lives
  in the **draw** pass, not the update pass.
- `Terraria/Main.cs::UpdateCameraPan` (62680-62704), `UpdateCameraLerp` (18143-18157).
- `Terraria/Main.cs::UpdateViewZoomKeys` (16801-16815) — ±0.02 zoom-key step.
- `repo::app/frame.rs::App::frame` (82-1760) — the port's whole per-frame driver.
- `repo::world_sim/step.rs::FixedStepper` (9-63) — `tick` / `residual` / `cap_backlog`
  / `drop_backlog`, `step = 1/60`, `max_steps = 8`.
- `repo::world_sim/clock.rs::advance_time` (13-77) — `Main.time` wrap + event rolls.
- `repo::world_sim/camera.rs::pan_camera` (18-38) + `screen_size` (12-16).
- `repo::world_sim/weather.rs::update_world_tick` (105-159) — the `Main.dayRate`
  cluster inside a 6th stepper.

## Dependencies

- Calls into: `world_sim::{player,npcs,items,projectiles,liquids,weather,scene_scan,
  editing}` (one stepper each, all driven from `frame.rs` 354-556), `render::*` for the
  draw half, `menu`/`ui`/`front` for the non-world half.
- Called by: `app/loop.rs` `RedrawRequested` → `frame()` (`loop.rs:133-135`), i.e. the
  winit event loop replaces `Game.Update`/`Game.Draw` (`Main.cs:16794` `base.Update`).
- Shared data: `App` (118 fields, `app/mod.rs`), `tr_core::World` (time/day_rate), the
  per-subsystem RNG streams, `FixedStepper` accumulators.

## Port status

| C# subsystem | Status | Rust location | Evidence / notes |
| --- | --- | --- | --- |
| `DoUpdate` phase order | PASS (world branch, S1-B-1) | `frame.rs::frame` | The world branch now follows `DoUpdateInWorld_Inner` (`Main.cs:17745-17822`): players → all-asleep → spawn/NPCs → **projectiles → items** → dust → **time** → `WorldGen.UpdateWorld` (`WorldGen.cs:71495-71630`: wiring → tile entities → liquid → growth) → waterfalls → the port's environment cluster (weather → rain → scene state → windy day → ambience) → editing → net → map. Pinned per frame by `app::frame::tests::the_world_branch_runs_in_the_csharp_update_order`. Two known remaining gaps: the port's environment cluster (weather/rain/ambience) belongs to `Main.Update`'s pre-world half (`:17166-17233`) in the C#, and this branch keeps it inside; the gores / leashed-entities / floating-text passes are still unported. |
| frame-skip accumulator / `TARGET_FRAME_TIME` | PARTIAL | `frame.rs:88-94` | C# 17006-17020 clamps `UpdateTimeAccumulator` into `[0, 3×TARGET]` and suppresses draws; the port clamps `dt` to `[0, 0.1]` (0.1 s = 6 steps, so `max_steps` 8 is never the binding cap) and never suppresses draws. No `successiveSkippedDraws`. |
| `FixedStepper` 60 Hz accumulator | PASS | `world_sim/step.rs:9-63` | Faithful re-creation of the accumulator shape; `max_steps = 8` at `app/mod.rs:2475-2879` and tests `step.rs::tests::tick_runs_at_most_max_steps_per_frame`. |
| adaptive `gfxQuality` → dust/liquid/light budgets | MISSING | — | C# 17054-17113 (`maxDustToDraw`, `Liquid.cycles`, `Liquid.curMaxLiquid`, `LegacyLighting.RenderPhases`) has no port; `settings.quality` exists as a menu toggle only. |
| `Main.time` / `UpdateTime` day-night wrap | PASS | `world_sim/clock.rs:13-77` | 54000/32400 halves, `day_rate()`, dial flag + cooldown at each boundary, moon phase wrap, overshoot discarded not wrapped (`Main.cs:65353-65354`, `:65382/65389`). Boundary rolls (blood moon, eclipse, angler swap, morning invasion) are reported/armed, with event resolution in the app pass. |
| `UpdateTime` rain/slime-rain/ladybug sections | PARTIAL | `world_sim/weather.rs:147-156` | `rain.update_time(day_rate, …)` covers `Main.cs:64864-64949`; the `slimeRainTime` drain (64848-64863) and `ladyBugRainBoost` drain (64833-64836) are folded across `rain`/`roll_morning_invasion`; `pumpkinMoon/snowMoon → bloodMoon = false` (64837-64845) not ported. |
| `gameMenu` vs in-game update split | PARTIAL | `frame.rs:328` | The whole world section is gated on `self.menu.is_none() && self.front.is_none()` — mirrors `DoUpdate`'s `if (Main.gameMenu) { UpdateMenu(); return; }` (`Main.cs:17166-17175`). What still runs in a menu (port): pointer clock, `team_cooldown`, disco, menu music, audio listener, autosave, `pan_camera`. C# additionally runs `UpdateUIStates`, `Filters.Scene.Update`, `Animation.UpdateAll`, `DoUpdate_AutoSave`, `Chroma`, `SkyManager` before the gameMenu branch (`Main.cs:17039-17152`). |
| pause (`CanPauseGame` / `DoUpdate_WhilePaused`) | MISSING | — | `Main.cs:17315-17385` sets `gamePaused` and still runs the inventory refresh/scroll/drop-check/effective-armour pass; the port has no pause gate — `inventory_open`/`ingame_options.open` never stop the fixed steps. |
| camera follow (`PlayerFocusedScreenPosition`, `cameraLerp`) | PARTIAL | `world_sim/player.rs:334,666` | Port lerps `app.camera` toward `p.center()` with `1 - exp(-dt*12)` **inside the player fixed step**; C# positions the camera in `DoDraw_UpdateCameraPosition` (`Main.cs:62590-62649`) at draw time (`Main.cs:61832` call site), with `cameraLerp` lerp (62634-62644), `cameraX/cameraY` decay (62596-62631) and `ClampScreenPositionToWorld` (63948). No `CurrentPan`, no `cameraLerp`. |
| camera pan (free-fly WASD) | PASS (deviation) | `world_sim/camera.rs:18-38` | Port's own free-fly (`900.0 * dt / zoom`, `TR_PAN` hook) — the C# `UpdateCameraPan` (62680-62704) is an object/player-controlled *pan*, not a debug fly camera; `DebugCameraPan` (62651-62674) is the nearest C# analogue but is 32 px/frame with Shift/Ctrl modifiers. |
| zoom (`GameZoomTarget`, `UpdateViewZoomKeys`) | PARTIAL | `settings.rs:446`, `loop.rs:93-97`, `frame.rs:817-844` | Zoom applied in the draw as `draw_cam = (camera*zoom).round()/zoom` (frame.rs:812-817) instead of a `GameViewMatrix.Zoom`; the ±0.02 key step (`Main.cs:16801-16815`) is **not ported** — zoom only changes via the settings/menu row and `TR_ZOOM` (which itself allows 0.25-8.0 vs the vanilla 1.0-2.0 clamp at `Main.cs:61999`). |
| per-frame NPC/projectile/item/liquid/weather pass | PASS (structurally) | see step.rs consumers | All present as steppers: `npcs.rs:2253`, `items.rs:64`, `projectiles.rs:178`, `liquids.rs:16`, `weather.rs:127`. Order and gating (see row 1) are where they diverge. |
| `ShouldUpdateEntities` / `WorldUpdateStepper.ShouldUpdateWorld` | MISSING | — | `Main.cs:17278-17821` gates the world pass on `WorldPreparationState.Ready && !generatingWorld` and the testing stepper (`Terraria/Testing/WorldUpdateStepper.cs:17`); the port has no equivalent gate. |
| `DoUpdate_WhilePaused` armour/inventory refresh | MISSING | — | no `gamePaused` concept in `App`. |
| ambient/misc per-frame C# work (Chroma, achievements advisor, credits roll, frame-skip FPS counter) | MISSING | — | `Main.cs:16847-17233`; none ported (mostly platform/QA-only). |

## Tests

- `world_sim/step.rs::tests::tick_runs_at_most_max_steps_per_frame` — the only direct
  test of the accumulator (cap vs drop backlog, residual).
- `world_sim/clock.rs::tests` (10): `the_dusk_boundary_rolls_and_reports_the_blood_moon`,
  `the_dawn_boundary_rolls_and_reports_the_eclipse`, `day_night_clock_wraps`,
  `dw_1_1_ff_to_dawn_crosses_day_boundary`, `dw_1_1_ff_to_dusk_crosses_dusk_boundary`,
  `dw_1_2_flag_clears_at_boundary_and_day_rate_restores`,
  `the_all_asleep_flag_runs_the_clock_at_five_times`,
  `dw_1_3_cooldowns_decrement_once_per_transition`,
  `dw_1_3_cooldowns_saturate_at_zero`,
  `boundary_max_add_crosses_at_most_one_boundary`.
- `app/loop.rs::tests` (6): `escape_closes_the_ingame_options_window_first`,
  `escape_falls_through_when_the_options_window_is_shut`,
  `escape_closes_the_open_map_before_the_inventory`,
  `escape_falls_through_to_the_inventory_when_the_map_is_shut`,
  `the_map_key_still_toggles_the_map`,
  `the_smart_cursor_toggle_flips_the_runtime_flag` (world-key routing).
- `world_sim/camera.rs` — **no tests**. `frame.rs` gained its first in-module tests with
  S1-B-1: `app::frame::tests::the_world_branch_runs_in_the_csharp_update_order` (pins the
  per-frame `phase` trace against `Main.cs:17745-17822`) and
  `app::frame::tests::the_clock_only_advances_inside_the_world_branch` (S1-B-2). The frame
  driver is otherwise covered indirectly by the deterministic screenshot harness
  (`docs/ANALYSIS.md` "Minecart riding" notes the `TR_SHOT` SHA256-equal pattern).

## Deviations & risks

1. **Clock advanced while a menu/front screen is open** — *resolved (S1-B-2, landed with
   S1-B-1)*. The `UpdateTime` call moved from the top of `frame` (outside the world gate)
   to inside the world branch, where the `menu.is_none() && front.is_none()` gate mirrors
   `DoUpdate`'s `gameMenu` early return (`Main.cs:17166-17175`; `DoUpdateInWorld` is only
   reached at `:17278-17281`). Regression test:
   `app::frame::tests::the_clock_only_advances_inside_the_world_branch`.
2. **Camera follow runs in the fixed step, not the draw pass** — player.rs:334/666 vs
   `Main.cs:61832`/`62590-62649`. Severity: **behavioural/sub-pixel** (the port added
   `1-exp(-dt*12)` smoothing the vanilla does not have, plus `cameraX/cameraY`,
   `CurrentPan`, `cameraLerp` and the world clamp at `Main.cs:63948` are absent).
   ANALYSIS admits the interpolation intent (frame.rs:572-575) but not the missing
   clamp/pan. (S1-B-4.)
3. **items after projectiles** — *resolved (S1-B-1)*. The branch now draws projectiles
   (`:17976`) before items (`:18004`) exactly as the C#; the ordering regression test runs
   both phases on a real headless frame.
4. **`UpdateTime` ran before the entity pass** — *resolved (S1-B-1)*. The clock now
   advances in its `WorldGen`-preceding slot after dust, so `Main.time` readers in
   `Player`/`NPC` see the vanilla value for the phase.
5. **No `max_steps` bottleneck and no frame suppression** — dt clamp 0.1 s
   (frame.rs:88-94) silently turns the vanilla's `UpdateTimeAccumulator`/draw-skip
   policy into "drop at most 6 steps". Long stalls behave differently from
   `Main.cs:17006-17020`. Severity: **behavioural** (only visible below ~10 fps).
6. **No pause semantics** — with `inventory_open` the world keeps simulating; C# sets
   `gamePaused` (17194-17201) and runs only `DoUpdate_WhilePaused`. Severity:
   **behavioural** (dropped items/NPCs keep moving while the inventory is open, unless
   `autoPause` is off — which it is by default, so single-player diverges from default
   vanilla behaviour).
7. **Zoom accelerators and the 1.0-2.0 range** — `UpdateViewZoomKeys` unported; the
   `TR_ZOOM` debug pin allows 0.25-8.0. Severity: **cosmetic** (debug only) plus one
   missing vanilla feature.
8. **`gfxQuality` budgets absent** — dust/liquid/light caps (`Main.cs:17054-17113`)
   have no analogue; the port's `settings.quality` is decorative. Severity:
   **cosmetic/performance**, not determinism.

## Task seeds

- **M1.1-LOOP-1 (S) — DONE (S1-B-2, landed with S1-B-1)** — `advance_time` moved inside
  the `menu.is_none() && front.is_none()` world branch. Regression test
  `the_clock_only_advances_inside_the_world_branch`. Fixed deviation 1.
- **M1.1-LOOP-2 (M) — DONE (S1-B-1)** — `frame.rs`'s world branch reordered to the C#
  `DoUpdateInWorld_Inner` order (players → spawn NPCs → NPCs → **projectiles → items** →
  dust → **time** → WorldGen wiring/TE/liquid/growth → …). The gores / leashed /
  floating-text passes remain unported (no gore or floating-text module yet), so they are
  absent rather than reordered. Fixed deviations 3-4; pinned by
  `the_world_branch_runs_in_the_csharp_update_order`.
- **M1.1-LOOP-3 (M)** — port `gamePaused`/`CanPauseGame` semantics: a single
  `paused` flag on `App` set when `inventory_open || ingame_options.open` (SP,
  `autoPause`-equivalent), skipping the steppers while still ticking UI/audio, plus
  `DoUpdate_WhilePaused`'s inventory refresh/scroll/drop-check. Fixes deviation 6.
- **M1.1-LOOP-4 (S)** — port `UpdateViewZoomKeys` (±0.02 on the zoom bindings) into
  `frame.rs`, clamp `self.zoom` to 1.0-2.0 except under `TR_ZOOM`, and confine the
  `TR_ZOOM` widening behind the debug flag only. Add a `camera.rs` unit test.
- **M1.1-LOOP-5 (M)** — port the C# draw-time camera: `PlayerFocusedScreenPosition`
  (62567-62570) incl. `gfxOffY - 21`, `cameraX/cameraY` decay, `cameraLerp`
  (18143-18157) and `ClampScreenPositionToWorld` (63948); move the follow out of the
  player fixed step or pin the port's smoothing as a documented deviation.
- **M1.1-LOOP-6 (S)** — add `camera.rs` unit tests for `pan_camera` speed
  (900 px/s ÷ zoom at zoom 1) and `screen_size`'s deterministic 1280×720 fallback. The
  "frame-level integration test asserting the branch order" half is **DONE** with S1-B-1
  (`the_world_branch_runs_in_the_csharp_update_order`); the `camera.rs` half remains.
- **M1.1-LOOP-7 (L)** — `ShouldUpdateEntities`-equivalent gate plus the world-loading
  state machine (`WorldPreparationState`), so `frame()` cannot tick a half-loaded
  world; depends on module 12's WorldGen research.
