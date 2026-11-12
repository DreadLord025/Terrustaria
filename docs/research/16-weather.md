# 16 — Weather

> Wave 1 · researched 2026-11-12 · sources: decompiled + crates + ROADMAP/ANALYSIS/PORT_MAP

## Scope

Module 5 of the audit = everything that decides the sky, the falling weather and the
scene metrics that the framing/filters read.

C# side:

- `decompiled/Terraria/Rain.cs` (208) — the rain-drop field: `MakeRain` `:30-89` (the
  `ZoneRain` gate `:32`, the `25 * (0.25 + cloudAlpha)` attempt rate `:36`, the
  `NPC.AnyDanger` 0.05 cut `:37-45`, the player-velocity spawn window `:49-57`) and
  `NewRain` `:166-207` (the `maxRain * cloudAlpha` slot budget, the `gfxQuality` cut
  `:176-180`, the snow-tile suppression `(1 - (snow/max)²)` `:181-182`, the `atmo⁹` /
  `atmo < 0.4 → 0` factor `:183-186`, the `waterStyle * 3 + rand(3)` frame type `:205`).
- `decompiled/Terraria/Cloud.cs` (451) — the live cloud array: `resetClouds`/`addCloud`
  `:39-293` (the `rand(1000) < 200*cloudAlpha` spawn `:116-127`, the clear-weather type
  branches `:128-139`, `RollRareCloud` `:140-147`, the spawn overlap rejection
  `:170-177`), `UpdateClouds` `:295-316` (parallax + drift + kill), `cloudColor`
  `:227-235`, the per-tick `windSpeedCurrent * 9 * GetParallax` drift `:391`.
- `decompiled/Terraria/SceneMetrics.cs` (945) — `Scan` `:336-351`, `ScanTiles`
  `:353-379` (the per-tile counts), the visual-scan arm `:4489-4501` (music boxes) and
  the banner arm `:416-445`, `AggregateTileCounts` `:611-624`, `CalculateZones`
  `:662-707`.
- `decompiled/Terraria/SceneState.cs` (492) — `ApplyVisuals` `:64-253` (the filter
  board) and the blizzard sound block `:189-253`.
- `decompiled/Terraria/GameContent/Events/Sandstorm.cs` (241) — the severity machine
  `UpdateTime` `:32-77` (the hardMode start denominator `:58-59`), `EmitDust` and the
  dust-268 AI.
- `decompiled/Terraria/Main.cs` clusters: `UpdateWeather` `:58698-59066`
  (`ResetWindCounter` `:58700-58706`, the thunder arm `:58737-58829`, the wind walk
  `:58850-58950`), `UpdateTime`'s rain section `:64833-64949`, `StartRain` `:64659`,
  `ChangeRain`/`StopRain` `:64644-64740`, `snowing()` `:13032-13051`,
  `UpdateWindyDayState` `:12990-13027`, the wind init `:10864-10881`,
  the `cloudAlpha` ramp `:61901-61920`, the `dayRate` rate pick `:6199-6206`,
  the blood-moon roll `:65333-65352`, the solar-eclipse roll `:65421-65430`,
  the Lantern Night force-stop `:64825-64831`.

Rust side: `crates/tr-core/src/wind.rs` (830) — the `Wind` global (the target walk, the
counters, the sway counters, the tile-draw sets); `crates/tr-app/src/rain.rs` (1851) —
the drop field + the rain event state machine + the thunder chain;
`crates/tr-app/src/world_sim/weather.rs` (1050) — the shared 60 Hz world tick that
drives the vanilla `Update` dayRate cluster; `crates/tr-core/src/scene.rs` (2689) —
`SceneMetrics`/`Zones`/`Sandstorm`/`SceneState`/the filter board and every filter's
parameter math; `crates/tr-app/src/world_sim/scene_scan.rs` (820) — the tile-window
scan driver; `crates/tr-app/src/sand.rs` (579) — `Sandstorm.EmitDust`;
`crates/tr-app/src/ambient_sound.rs` (557) — the blizzard loops + the thunder strike
cue. The clock (`crates/tr-app/src/world_sim/clock.rs::advance_time` `:13-77` +
`crates/tr-core/src/world.rs::roll_blood_moon` `:565` / `roll_eclipse` `:594`) owns the
dusk/dawn event rolls that this module's consumers (music, filters, ambience) read.

## Key entities

- `crates/tr-core/src/wind.rs::Wind::new` (`:94-113`) — the seeded wind global:
  `speed_current = range(-800,801) * 0.001` (`Main.cs:10874`), `counter = range(900,2701)`, `extreme_counter = range(10,31)`.
- `crates/tr-core/src/wind.rs::Wind::update_rain` (`:168-264`) — the drift + the
  `windCounter`/`extremeWindCounter` tier walk, ≡ `Main.cs:58830-58950`; the 0.35 cap
  reads `self.any_player_120_life` (`:199`, `:211`, `:232`).
- `crates/tr-core/src/wind.rs::Wind::update_windy_day_state` (`:120-152`) — the sticky
  windy-day/storm flags, ≡ `Main.cs:12990-13027`; called per frame from
  `crates/tr-app/src/app/frame.rs:507-511`.
- `crates/tr-core/src/wind.rs::Wind::cycle` (`:284-312`) — `GetWindCycle` for the tile
  draw; plus `advance_counter` `:358`, `advance_tree_counter` `:365`,
  `advance_sunflower_counter` `:768`, `advance_vine_counter` `:775`,
  `leaf_frequency` `:736`, `wall_allows_wind` `:727`, `SWAYS_IN_WIND_BASIC` `:710`.
- `crates/tr-app/src/world_sim/weather.rs::update_world_tick` (`:105-276`) — the one
  60 Hz tick running, in C# order: the thunder arm once (`Main.cs:58739`), the wind
  walk once per `day_rate`, `Rain::update_time`, `Sandstorm::update_time`, the slime-rain
  event, the travelling merchant and the meteor/Deerclops bookends (`thunder_inputs`
  `:39-96`).
- `crates/tr-app/src/rain.rs::Rain::update_time` (`:571-635`) — `Main.UpdateTime`'s rain
  section (the `ladyBugRainBoost` drain, the `rainTime` drain + strength roll, the
  ×5.75/×4.25/ladybug start rolls).
- `crates/tr-app/src/rain.rs::Rain::start_rain` (`:642-708`), `change_rain` (`:712-745`),
  `stop_rain` (`:747-761`) — the `Main.cs:64644-64740` ladder.
- `crates/tr-app/src/rain.rs::Rain::thunder_step` (`:441-520`) — the `NewLightning`
  chain and the strike cue; `spawn` (`:798-837`) = `MakeRain`; `update_snow`/`spawn_snow`
  (`:839-920`) = the snow-biome dust; `draw` (`:922+`).
- `crates/tr-core/src/scene.rs::SceneMetrics` (`:25-133`) + `Zones` (`:155-198`) +
  `ZONE_SCAN_SIZE` (`:147` = 169×124) — the scan payload; `Sandstorm` (`:531-695`);
  `SceneState` (`:697-888`); `FilterBoard` (`:1107-1226`); the per-filter parameter
  functions `graveyard_progress` `:1303`, `blood_moon_opacity` `:1311`,
  `noir_intensity` `:1329`, `sandstorm_foreground_params` `:1369`,
  `sandstorm_background_opacity` `:1380`, `blizzard_foreground_params` `:1389`,
  `water_distortion_*` `:1437-1472`, `heat_*` `:1474-1530`, `parse_env_force` `:889`.
- `crates/tr-app/src/world_sim/scene_scan.rs::scan_scene_at` (`:45-100`) + `scan_tiles`
  (`:101-220`) — the window scan; `update_scene` (`:261-396`) — the driver that runs
  `Sandstorm::update_time`'s emit/tick, the `SceneState::update` fades and the
  `ApplyVisuals` filter board.

## Dependencies

- Calls into: `tr-core::tiledata` (the tile sets for the scan and the sway sets),
  `tr-core::World` (the `day_rate`, `hard_mode`, `blood_moon`, `eclipse`, `slime_rain`,
  `world_surface` fields the rolls read), `tr-core::loot::roll_luck` (the coin-rain
  gate), `tr-core::scene::parse_env_force` (the `TR_*` pins), `crate::biome::is_snow_biome`,
  `crate::audio`/`ambient_sound` (the thunder cue, the blizzard loops),
  `crate::render::sky` (`atmosphere_factor`, `make_clouds`) and the post-pass filter
  shaders (`app/filters.rs`).
- Called by: `crates/tr-app/src/app/frame.rs` — `update_world_tick` before `update_rain`
  (`frame.rs`), `scan_scene`/`update_scene` in the frame's scene block
  (`frame.rs:499`), `wind.update_windy_day_state` at `:507`, and the ambience context
  built at `:518-548` (`is_raining`, `storming`, `happy_windy_day`,
  `too_windy_for_butterflies`).
- Shared data structures: `tr_core::scene::SceneMetrics` (filled by the scan, read by
  the spawn flags, the music ladder, the filters and the auras), `tr_core::Wind` (the
  tile-draw sway, the sandstorm gate, the music tail), `tr_core::scene::SceneState`
  (the filter intensities), `crate::rain::Rain` (`max_raining` feeds both `Wind` and the
  `change_rain` bands).

## Port status

### ROADMAP DONE claims re-verified (PASS / PARTIAL / FAIL)

| ROADMAP row (marked DONE) | Verdict | Evidence |
| --- | --- | --- |
| M1.4-BG-11/16 — clouds/stars/horizon parallax | PARTIAL | `render/sky.rs::make_clouds` (`:270-342`) is a deterministic *stand-in*: `NUM_CLOUDS_STAND_IN = 100` (`rain.rs:57-63`), no live `Cloud.addCloud`/`UpdateClouds`, drift is a constant light wind scaled by `GetParallax` instead of `windSpeedCurrent * 9 * GetParallax` (`Cloud.cs:391`) — ANALYSIS "Clouds and stars" admits all of it, the ROADMAP row's "done" reading overstates it. |
| M1.4-BG-18 — screen filters | PASS | `tr-core/src/scene.rs` `:1107-1530` ports the filter board + every parameter function; 58 tests in `scene.rs` cover the ramps, the winner priority and the distortion gates. Admitted-out: the monolith/tower/aurora/sepia filters. |
| M1.4-BG-19 / BG-20 — ambient skies + emitters | PASS | `tr-app/src/ambience.rs`'s scheduler + `ambient_sound.rs` (`blizzard_loop_volumes`, the thunder cue, the critter table). Out of weather scope beyond the blizzard loops (`SceneState.cs:189-253`, ported at `ambient_sound.rs` + `scene.rs::blizzard_foreground_params` `:1389`). |
| M1.4-BG-21 — sundial/moondial + dayRate plumbing | PARTIAL | The dial plumbing is ported, **but the row claims the blood-moon/eclipse rolls and the dial-cooldown force-resets are "Out (M4)"** — they are in fact landed: `world_sim/clock.rs::advance_time` `:37-38` (blood moon) and `:72` (eclipse) calling `tr-core/src/world.rs::roll_blood_moon` `:565` / `roll_eclipse` `:594`. The stated remaining gaps (`tenth_anniversary` argument hardcoded `false`, `spawn_eye` hardcoded `false`, `stopEvents` absent) are unstated. |
| M1.4-BG-22 — the dayRate consumer cluster | PASS | `world_sim/weather.rs::update_world_tick` `:127-276` runs the cluster in C# order; `test_DW_1_2_wind_counter_walk_fires_within_one_tick_at_60_not_at_1`, `test_DW_2_2_update_time_runs_inside_the_world_tick`. Admitted-out: the live `numClouds` system, `updateCloudLayer`, the Lantern Night arm. |
| M1.4-WEATHER-1 — "wind gusts + thunder" | PARTIAL | The walk + the `NewLightning` chain port verbatim, but two claims fail: (a) "the 0.35 walk cap … already ported and verified" — `Wind::any_player_120_life` (`wind.rs:75`) is set `true` in `Wind::new` `:108` and **never written anywhere in `crates/`**, so `Main.cs:58863-58871/58889`'s low-wind limit is dead code and untested; (b) the lightning **bolt geometry** is still absent (admitted). |
| M1.4-WEATHER-2 — sandstorm dust | PARTIAL | `sand.rs` ports `EmitDust` + dust-268 and `scene.rs::Sandstorm::update_time` `:570-607` ports the state machine, but the **hardMode branch of `Sandstorm.cs:58-59` is not taken**: `scene.rs:591` hardcodes `21600 * 3` and the comment says "hardmode is not ported yet", while `World::hard_mode` exists (and `roll_eclipse` at `world.rs:601` uses it). |
| M2-BUFF-5 — the aura scan | PASS | `scene_scan.rs::scan_tiles`'s aura arm + the `Auras` snapshot + the scene-buff grants; `the_aura_scan_reads_the_frame_gates`, `the_sunflower_count_and_the_graveyard_cancel`. |
| M6-AUDIO-1 / M6-MUSIC-1 — thunder cue + the music tail | PASS | `ambient_sound.rs` plays the strike; `windy_music`/`quiet_sky` feed the ladder (`frame.rs:439`, `:475`). The port notes the pitch roll is consumed but unplumbed. |

### Subsystem status

| C# subsystem | Status | Rust location | Evidence / notes |
| --- | --- | --- | --- |
| `UpdateWeather` wind walk + extreme re-rolls | PORTED | `tr-core/src/wind.rs:168-264` | `step = 0.0003 + \|target*(1+5/9*rain) - current\| * 0.0015` ≡ `Main.cs:58830-58849`; tier walk `:58860-58950`; `±0.8` clamp. `next_int` mods the high 32 bits (WEATHER-1 note). |
| `ResetWindCounter`'s RNG source | PARTIAL | `wind.rs:268-280`, `Wind::new:94-113` | C# `Main.cs:58700` mints a fresh `FastRandom.CreateWithRandomSeed()` per reset; the port keeps one seeded `Xorshift64` stream — the walk is shape-identical but not stream-identical (behavioural, unverifiable against C#). |
| low-wind limit (0.35 without a 120-HP player) | FAIL (dead code) | `wind.rs:75` `any_player_120_life`, `:199`, `:211`, `:232` | No writer exists in `crates/`; `Main.cs:58863-58871`'s `flag` is always "strong wind". No test covers it. |
| `UpdateWindyDayState` | PORTED | `wind.rs:120-152`, `frame.rs:507-511` | Hysteresis bands + the storm flag; the `remixWorld` override `:13022-13024` is out. |
| `UpdateWeather` thunder arm / `NewLightning` | PARTIAL | `rain.rs::thunder_step` `:441-520`, `weather.rs:127-133` | The delay ladder, the tier multipliers, `lightningSpeed`/`lightningDecay`, the graveyard ≥ 0.9 strike, the cue. **Missing: bolt geometry** and the `ParticleOrchestrator` storm-bolt spawner (`Main.cs:58822-58829`'s neighbours). |
| `UpdateTime` rain state machine | PORTED | `rain.rs:571-761` | `ladyBugRainBoost` drain, `rainTime` drain, the ×5.75/×4.25/ladybug arms, `StartRain`/`ChangeRain`/`StopRain`, the `86400/dayRate` denominators, the `RollLuck(25)` coin arm consumed-not-spawned. |
| `MakeRain`/`NewRain` drop field | PARTIAL | `rain.rs::spawn` `:798-837` | The attempt rate and the open-sky/spawn-window shape match `Rain.cs:36-89`, but `NewRain`'s `gfxQuality` cut (`:176-180`), the snow-tile suppression `(1-(snow/max)²)`(`:181-182`), the `atmo⁹` and `atmo < 0.4 → 0` gates (`:183-186`) and the `waterStyle*3` frame type (`:205`) are **not ported**; the drop velocity uses the frozen `self.wind` + a sine term instead of the live `Main.windSpeedCurrent * 18`. |
| Snow biome weather | PARTIAL | `rain.rs::update_snow`/`spawn_snow` `:839-920` | The snowflake dust is the port's own approximation; vanilla's `Main.snowing()` (`Main.cs:13032-13051` — the `pow(SnowTileCount/max, 4 - 3*cloudAlpha)` intensity, the `1 + 50*cloudAlpha` emitter count, the Deerclops `÷20`/`÷3` fades) is **not ported**. |
| `Cloud` live system | MISSING | `render/sky.rs::make_clouds` `:270-342` | No `addCloud` ladder, no `UpdateClouds` sort/kill, no rare clouds, no overlap rejection, no `cloudBGX` drift; `numClouds` is pinned at 100, which pins `change_rain`'s strength bands (`rain.rs:57-63`, `:713-722`). |
| `SceneMetrics` scan | PORTED | `scene_scan.rs:27-220`, `scene.rs:335-503` | 169×124 window ≡ `SceneMetrics.cs:336-379`; `AggregateTileCounts`/`CalculateZones` order preserved. Divergence: the music-box arm (`:4489-4501`) scans the 169×124 window instead of the visual screen rect (`scene_scan.rs:141-151`). |
| `Sandstorm` state machine + dust | PARTIAL | `scene.rs:570-695`, `sand.rs` | All of `UpdateTime`/`ChangeSeverityIntentions`/`UpdateSeverity` + `EmitDust` + dust-268; the hardMode denominator (`Sandstorm.cs:58-59`) is not taken; `gfxQuality` pinned 1.0, `SandstormSky` is dead code in 1.4.5.8 (WEATHER-2 verified this claim — PASS). |
| `SceneState` filters | PORTED | `scene.rs:697-888`, `:1107-1530` | The fades, the board, every parameter function; the blizzard `deerclops_smoothed` input is passed 0 (no M1 spawner) — both lerps no-op. |
| Blood moon / solar eclipse rolls + dial force-reset | PORTED | `world_sim/clock.rs:37-38`, `:72`; `world.rs:565-611` | `Main.cs:65333-65352` and `:65421-65430` including the cooldown resets. Partially folded: `tenth_anniversary`, `spawn_eye`, `netMode != 1`, the `stopEvents` gate, and the `moonPhase != 4` gate is ported but the pumpkin/snow-moon clearings (`Main.cs:64837-64845`) are not. |
| Lantern Night force-stop | MISSING | `rain.rs:609-610` (admitted) | `Main.cs:64825-64831`'s `cloudBGActive = 0` / `numClouds = 30` clamp and the `:64890` gate resolve as "never a lantern night". |
| `rainsForAYear` micro-seed | MISSING | `weather.rs:6-17` (admitted) | No `Rain::start_year_long_event` entry; the worldgen has no `World.raining` to set. |
| Rain loop / wind loop ambient tracks | PORTED | `ambient_sound.rs` | BG-20's `ZoneRain && !ZoneSnow` track 28 and `\|wind\| ≥ 20/50` track 45 with the ±0.005 fades. |

## Tests

144 `#[test]` functions across the module (static count, no cargo run):

- `tr-core/src/wind.rs` (17): `new_is_deterministic_and_in_source_ranges`,
  `dry_wind_stays_within_08`, `target_and_rain_scaled_current_stay_bounded`,
  `rain_increases_abs_current` (asserts the exact `0.4 * (1 + 5/9)` target),
  `advance_counter_matches_grass_formula`, `advance_tree_counter_matches_tree_formula`,
  `test_DW_4_1_sunflower_sway_counter`, `cycle_stays_within_source_bounds`,
  `cycle_varies_with_x_y_and_counter`, `cycle_respects_surface_and_sway_setting`,
  `same_seed_same_history` (50 000-tick bit-equality),
  `windy_day_holds_inside_the_band_and_flips_at_the_boundaries`,
  `storm_holds_inside_the_band_and_any_rain_kills_windy_day`,
  `too_windy_for_butterflies_is_a_pure_target_threshold`,
  `set_tests::sways_in_wind_basic_matches_the_csharp_list`,
  `set_tests::leaf_frequency_table_matches_the_source`, `rng_probe::probe_roll_distribution`.
- `tr-app/src/rain.rs` (24): `forced_rain_spawns_drops_that_die_on_the_ground`,
  `calm_weather_has_no_drops`, `snow_replaces_rain_in_a_snow_biome`,
  `storms_ramp_the_cloud_deck_and_lightning_decays`,
  `thunder_step_emits_one_strike_when_the_delay_runs_out`,
  `the_lightning_micro_seeds_gate_the_storm_roll`,
  `thunder_step_is_deterministic_per_seed`,
  `test_DW_1_1_thunder_arm_runs_once_per_tick_not_per_day_rate_iteration`,
  `test_DW_2_1_start_rain_ladder_matches_the_csharp_replay`,
  `test_DW_2_1_change_rain_band_gates_pick_by_cloud_bg_stand_in`,
  `test_DW_2_2_start_denominator_8280_at_day_rate_60_vs_496800_at_1`,
  `test_DW_2_3_cloud_alpha_ramp_moves_60x_at_day_rate_60`,
  `test_DW_2_4_coin_rain_roll_is_consumed_without_a_spawn`,
  `test_DW_2_2_update_time_runs_inside_the_world_tick`,
  `test_DW_1_2_stop_rain_clears_the_event_state`,
  `max_raining_is_the_ramp_target_not_the_visible_intensity`, …
- `tr-app/src/world_sim/weather.rs` (11):
  `test_DW_1_1_wind_loop_runs_before_the_sandstorm_in_one_tick`,
  `test_DW_1_1_sandstorm_gets_the_dead_wind_penalty_at_day_rate_1`,
  `test_DW_1_2_wind_counter_walk_fires_within_one_tick_at_60_not_at_1`,
  `test_DW_1_4_headless_tick_advances_wind_and_sandstorm_without_gpu`,
  `test_DW_2_2_tick_feeds_the_player_scan`, `the_travel_merchant_*`,
  `the_slime_rain_roll_gates_and_king_counter`, `the_deerclops_roll_gates_the_death_flag`,
  `the_meteor_roll_and_boss_death_flag`, …
- `tr-app/src/world_sim/scene_scan.rs` (18): `test_DW_1_1_snow_zone_counts`,
  `test_DW_1_1_desert_zone_counts`, `test_DW_1_1_rain_and_snow_both_true`,
  `test_DW_1_1_tile_center_outside_world_empty_scan`,
  `test_DW_1_1_window_clamped_at_world_edge`,
  `beach_sand_is_subtracted_from_the_desert_count`,
  `sandstorm_zone_requires_desert_surface_and_happening`,
  `thresholds_match_scene_metrics_statics`, `scan_counts_graveyard_minus_sunflowers`,
  `behind_backwall_reads_the_center_tile_wall`, `the_aura_scan_reads_the_frame_gates`,
  `test_DW_1_4_scene_forces_wire_through_the_glue`, `update_scene_scans_updates_state_and_drives_the_filter`,
  `update_scene_hands_the_pre_update_skip_flag_to_the_driver`,
  `update_scene_ramps_the_blizzard_loop_emitter`,
  `update_scene_without_conditions_runs_no_tint_filter`,
  `update_scene_feeds_the_driver_gates_from_settings`, …
- `tr-core/src/scene.rs` (58): the zone-threshold boundaries
  (`zones_at_exact_thresholds`, `corruption_crimson_hallow_jungle_thresholds_bound_their_zones`,
  `zone_rain_needs_raining_and_surface`, `zone_sandstorm_suppresses_zone_rain`), the
  sandstorm machine (`test_DW_1_2_severity_approaches_at_0_003`,
  `test_DW_1_2_time_left_decay_and_stop`, `test_DW_1_2_has_sufficient_wind_0_6`,
  `storm_starts_from_the_roll_branch_deterministically`), the filter board
  (`test_DW_3_4_board_activate_fade_in`, `test_DW_3_4_winner_priority_tie_activation_order`,
  `test_DW_3_1_blood_moon_opacity_depth_fade`, `test_DW_3_3_sandstorm_params`,
  `test_DW_3_3_blizzard_params`, `test_DW_4_3_water_*`, `test_DW_4_1_heat_*`).
- `tr-app/src/sand.rs` (7) + `tr-app/src/ambient_sound.rs` (9) cover `EmitDust`'s
  budget/cluster/colour arms, the dust-268 AI and the blizzard/thunder emitters.

Coverage gaps: nothing exercises the 0.35 low-wind cap, `any_player_120_life`, the
Lantern Night arms, `snowing()`'s intensity formula or a live cloud spawn — because none
of them exist.

## Deviations & risks

1. **`any_player_120_life` has no writer — the 0.35 wind cap is dead code** —
   *fixed 2026-11-13 (S1-L-2, `S1-L-2-rain-gates`)*: `world_sim::weather::update_world_tick`
   now writes `Wind.any_player_120_life` from the port's single-player
   `statLifeMax >= 120` scan (`Main.cs:58863-58871` / `:64899-64909`), and
   `the_world_tick_writes_any_player_120_life_into_wind` locks the flag. The
   ROADMAP row's "already ported and verified" claim is corrected.
2. **`Sandstorm.cs:58-59`'s hardMode denominator not taken** — `scene.rs:591` pins
   `21600 * 3` with the stale comment "hardmode is not ported yet" (`World::hard_mode`
   exists). Severity: **behavioural** (sandstorms start 1.5× too often in hardmode).
   Admitted in code, not in ANALYSIS.
3. **`Main.snowing()` not ported** — the snow dust in `rain.rs:839-920` is a separate
   approximation without the `pow(count/max, 4 - 3*cloudAlpha)` intensity, the
   `1 + 50*cloudAlpha` emitter count or the Deerclops `÷20`/`÷3` fades
   (`Main.cs:13044-13051`). Severity: **behavioural/pixel** (blizzard intensity in snow
   regions does not track the snow tile count). Only the ANALYSIS "Rain and weather"
   paragraph covers the dust, so partially admitted.
4. **The live `Cloud` system is a static stand-in** — `NUM_CLOUDS_STAND_IN = 100`
   (`rain.rs:57-63`) makes `change_rain`'s `numClouds > 150` / `> 100` bands unreachable
   (`Main.cs:64728`), so rain strength only ever reaches the storm band; `RollRareCloud`
   (`Cloud.cs:140-147`) and the overlap rejection (`:170-177`) never run. Severity:
   **behavioural**. Admitted (ANALYSIS "Clouds and stars", BG-22's row).
5. **Rain drops use a frozen wind snapshot** — `Rain::wind` (`rain.rs:124-128`,
   `:811`) vs vanilla's per-tick `Main.windSpeedCurrent` (`Rain.cs:56`,
   `Cloud.cs:391`). Severity: **behavioural/pixel** (sheeting rain does not bend with a
   gust). Admitted in a code comment only.
6. **`NewRain`'s spawn factors missing** — *partially fixed 2026-11-13 (S1-L-2)*:
   `Rain::spawn` now gates on `!ZoneRain || shimmerAlpha > 0` (`Rain.cs:30-35`) and
   applies the `NewRain` active budget with `cloudAlpha`, snow suppression
   `(1 - (SnowTileCount/SnowTileThreshold)^2)` (`:181-182`) and `atmo^9` / `atmo < 0.4`
   (`:183-186`), threaded through `Rain::update`; drop type uses `waterStyle*3+rand(3)`.
   Still unported: `gfxQuality` scaling (`:176-180`), `NPC.AnyDanger` throttle (`:37-45`),
   and full spawn-window/player-velocity shaping. Remaining: **pixel**.
   Not admitted in ANALYSIS.
7. **No lightning bolt geometry** — the flash is a full-screen overlay
   (`Main.lightning`); the `ParticleOrchestrator` bolt spawner and the
   `thunderSkipSound` writer are absent. Severity: **pixel**. Admitted (WEATHER-1,
   ANALYSIS "Rain and weather").
8. **Lantern Night force-stop missing** — `rain.rs:609-610` folds
   `!slimeRain && !LanternsUp && !NextNightIsLanternNight` (`Main.cs:64890`) to
   "always open" and skips `:64825-64831`'s `cloudBGActive = 0`/`numClouds = 30`. Also
   the wind walk's `LanternsUp` early-out (`Main.cs:58856-58859`) collapses into
   `Wind::frozen`. Severity: **behavioural** (an unported seasonal event; low priority).
   Admitted in code.
9. **Music-box / visual-scan window substitution** — `scene_scan.rs:141-151` scans the
   169×124 world window for tile 139 instead of the visual screen rect
   (`SceneMetrics.cs:4489-4501`). Severity: **behavioural** (a music box just off-screen
   can silence/override the track). Admitted in the code comment.
10. **`ResetWindCounter`'s per-reset `FastRandom` replaced by one shared `Xorshift64`**
    — `wind.rs:268-280` vs `Main.cs:58700`. Severity: **behavioural** (the re-roll
    streams are not reproducible against C#). Admitted only as WEATHER-1's
    `next_int`-bits note; the stream-sharing itself is unmentioned in ANALYSIS.
11. **Blood moon/eclipse rolls partially folded** — `clock.rs:37-38` passes
    `tenth_anniversary = false`, `spawn_eye = false` and no `stopEvents` gate;
    `Main.cs:64837-64845`'s pumpkin/snow-moon clearings are absent. Severity:
    **behavioural**, low frequency. The BG-21 row still claims these are "Out (M4)"
    although they are landed — **doc drift**.
12. **`M1.4-BG-23` has no catalog row at all** although its whole scope
    (the blood-moon/eclipse rolls + the dial-cooldown force-resets) is merged via
    `clock.rs`/`world.rs`. Severity: doc only.
13. **The `UpdateTime` rain gate's `!slimeRain` arm** — the port's start rolls run while
    the slime rain is not modelled in `Rain::update_time` (`rain.rs:609-610` passes
    `slimeRain` as always false), while `weather.rs:172-178` does model the slime-rain
    timer. A vanilla Lantern-less slime-rain night cannot start a rain event. Severity:
    **behavioural**, low.
14. **`deerclops_smoothed` is pinned 0** in `blizzard_foreground_params`
    (`scene.rs:1385-1388`), so the blizzard filter never halves near a Deerclops. Same
    shape for the `snowing()` Deerclops fade. Severity: **pixel/behavioural**, blocked
    on the M1 spawner. Admitted.

## Task seeds

- **(M) Wire `Wind::any_player_120_life` from the world tick** — set it in
  `update_world_tick` beside `any_player_120` (`weather.rs:111`), add a test asserting
  the `|target| ≤ 0.35` clamp when no 120-HP player exists, and correct the "verified"
  wording in ROADMAP M1.4-WEATHER-1. Depends on nothing.
- **(M) Take `Sandstorm.cs:58-59`'s hardMode branch** — thread `world.hard_mode` into
  `Sandstorm::update_time`'s start denominator (`scene.rs:570-600`), update the stale
  comment, add the `21600*2` arm to the seeded start-roll test. Depends on nothing.
- **(M) Port `Main.snowing()`** (`Main.cs:13032-13051`) as the snow weather's single
  source — move `rain.rs::spawn_snow`'s rate onto
  `500 * screenWidth/maxScreenW * (1 + 2*cloudAlpha) * pow(snow/max, 4-3*cloudAlpha)`
  with the Deerclops `÷20`/`÷3` arms, and let `update_snow` keep only the flake AI.
  Depends on a Deerclops-proximity query (the same one `blizzard_foreground_params`
  needs).
- **(L) Port the live cloud system** (`Cloud.cs`) — `addCloud`'s ladder, `UpdateClouds`
  (sort/kill), the `numCloudsTemp` walk (`Main.cs:58952-59036`), `RollRareCloud`,
  `cloudColor`, then replace `NUM_CLOUDS_STAND_IN` so `change_rain`'s bands
  (`Main.cs:64728`) become live. Depends on the boss flags for the rare clouds and the
  `atmo` input for `globalCloudAlpha`.
- **(M) Feed the live wind into the rain field** — replace `Rain::wind` with
  `wind.speed_current * 18` per drop and drop the sine term (`rain.rs:798-837`);
  A/B with `TR_SHOT` under a forced wind. Depends on nothing (determinism risk: pin the
  drop RNG shape first).
- **(S) Port `NewRain`'s spawn factors** — `gfxQuality`, the snow-tile suppression, the
  `atmo⁹`/`< 0.4` gate and the `waterStyle*3` frame type (`Rain.cs:166-207`).
- **(S) Fix the docs** — mark `M1.4-BG-23` as done (or open the row), correct
  `M1.4-BG-21`'s "Out (M4)" blood-moon/eclipse note, and record deviations 1, 2, 5, 6
  and 10 in ANALYSIS "Rain and weather".
- **(M) Port the Lantern Night arms** — the `cloudBGActive = 0`/`numClouds = 30` clamp,
  the `!slimeRain && !LanternsUp` gate and the wind-walk freeze
  (`Main.cs:64825-64831`, `:58856-58859`, `:64890`). Blocked on the Lantern Night event
  itself (M4).
- **(L) Lightning bolt geometry** — the `ParticleOrchestrator` bolt spawner + draw and
  the `thunderSkipSound` writer, so `thunder_step` stops standing in with a flash.
- **(S) Verify against a vanilla `.wld`/video** for one full in-game day of `windCounter`
  walks to confirm the `next_int` high-bit choice and the shared-stream deviation are
  the only divergences; no automated test can cover this.
