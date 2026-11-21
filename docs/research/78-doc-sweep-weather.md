# 78 — DOC-SWEEP lane 6: weather / events census

> W4-c40 DOC-SWEEP phase · 2026-11-21 · branch `task/W4-c40-sandstorm` ·
> CODE-ONLY (no cargo/tests/graphify; witnesses WRITTEN NOT RUN).
> Sources: decompiled C# + crates + the in-flight sibling branch diffs.

## Scope & verdict

Lane 6 re-censuses the five weather/event mechanics the brief named, against
the landed port AND the live sibling lanes, to find **real unported sim gaps**
(not visuals, not fall/draw claims). Verdict: **every one of the five is either
fully landed, owned by an active sibling lane, or the already-admitted
bolt-geometry lane — there is NO new in-lane sim gap to port.** The sweep's
real output is (a) this census, (b) the correction of a FALSE "dead code"
claim, and (c) one witness for an unwitnessed landed band.

| # | Mechanic | Verdict | Owner / evidence |
| --- | --- | --- | --- |
| 1 | Sandstorm severity ramp (start denominator + ongoing lerp) | **LANDED** | `scene.rs:1041-1149` (W4-c40 hardMode denominator + the 0.003/tick lerp); calm-intent bands witnessed this sweep |
| 2 | Wind → projectiles / particles | **LANDED / INERT-BY-DESIGN** | `Main.windPhysics=false` never enabled tree-wide; port matches (`set_wind` test-only). Rain-drift wind = d130; NPC bubble wind landed; Dandelion = d42 |
| 3 | Rain start/stop machine | **LANDED + sibling-owned remainder** | `rain.rs:654-838`; the slime/lantern gates (`Main.cs:64868`/`:64890`) are being landed by **f39** right now |
| 4 | Storm lightning strikes | **FLASH+THUNDER LANDED; strike = admitted bolt-geometry lane** | `rain.rs::thunder_step:516-640`; the damaging strike (`WorldGen.SpawnStormLightning`) needs `LightningGenerator.CanHitTarget` — the known "(L) bolt geometry" seed |
| 5 | Wind mill / pinwheel tile animation | **LANDED + WIRED + WITNESSED** | `drawdata_cases_b.rs:178-277` (489/490/493) + `drawdata_anim.rs:440-485` (the wind counters + weather-vane bob) |

## Lane boundaries (the claim doc's "0 commits" snapshot is STALE)

The W4-c40 claim doc (`W4-c40-sandstorm-claim.md`) recorded d130/f39/f42/d41/d42
as "0 commits". They are now all active and far ahead. The weather sim is
partitioned across them — **this sweep must not re-port their work**:

| Sibling branch | Owns (diff vs origin/main) |
| --- | --- |
| `W4-d130-weather-ambience` | `rain.rs` (1728-line diff: the drop-field, splash union, snowing), `wind.rs` (the `lanterns_up` gate), `main.rs` |
| `W4-f39-weather-slimerain` | `rain.rs` `update_time`'s slime/lantern gates (`:64868`/`:64890`), `weather.rs`, `npcs.rs`, saveglue |
| `W4-f42-sandstorm-2` | sandstorm second-half visuals (`player.rs`, `projectile.rs`, npc AI dust) |
| `W4-g67-weather-rain-splash` | the rain collision splash (transplanted into d130) |
| `W4-g68-weather-wind-leaves` | wind-driven leaf gore |
| `W4-g69-weather-clouds-parallax` | the live cloud system / parallax |
| `W4-g73-waterfall-draw` | waterfall draw + `render/liquids.rs` (the wind-driven `wFrCounter`/`wFrame`, `Main.cs:19547`) |
| `W4-g23-sounds-main-3` | the main sound pass (thunder cue playback) |
| `W4-d41/d42-npc-kinds-*` | the 541-546/580/581 + Dandelion(628) kind rows |

No sibling touches `tr-core/src/scene.rs` (verified by diff) — the Sandstorm
machine there is c40's, so the calm-intent witness below is in-lane.

## Item 1 — Sandstorm severity ramp: FULLY LANDED

`Sandstorm.cs` is 241 lines; the whole sim machine is ported in
`tr-core/src/scene.rs::Sandstorm`:

- `UpdateTime` (`:32-77`) → `update_time` (`scene.rs:1041-1081`): the
  `TimeLeft > 86400 → 0` reset, the `-dayRate` decay, the `15*dayRate`
  dead-wind penalty (`HasSufficientWind` = `|wind| >= 0.6`, `:22-25`), the
  `wind == 0 → TimeLeft = 0` instant stop, the start roll
  `Next(21600 * (hardMode ? 2 : 3))` per dayRate tick, the `Next(18000)`
  intention roll, and `UpdateSeverity`.
  - **The hardMode start denominator** (`:58-59`) — the brief's "your start
    denominator" — landed in W4-c40 (`scene.rs:1065`,
    `21600 * if hard_mode { 2 } else { 3 }`), fed `world.hard_mode` from
    `weather.rs:215`. Witnessed: `w4_c40_the_start_denominator_halves_in_hardmode`,
    `w4_c40_hardmode_does_not_touch_the_decay_or_intention_slots`.
- `UpdateSeverity` (`:99-115`) → `update_severity` (`scene.rs:1125-1149`):
  the NaN guards, `Math.Sign` approach at **0.003/tick**, `Clamp(0,1)`, and
  the overshoot snap. This is the "ongoing severity lerp" the brief asked to
  census — it is a **sign-step, not a proportional lerp** (the C# moves a
  fixed 0.003 toward the intent each tick and snaps on crossing). Witnessed:
  `test_DW_1_2_severity_approaches_at_0_003`, `_snaps_on_overshoot`,
  `_clamped_0_1`, `_nan_severity_resets`.
- `ChangeSeverityIntentions` (`:79-97`) → `change_severity_intentions`
  (`scene.rs:1110-1121`): happening → `0.4 + NextFloat`; not-happening →
  `Next(3)==0 ? 0 : NextFloat*0.3`. The happening band was witnessed
  (`test_DW_1_2_start_sets_duration_range_and_intent`); **the calm bands had
  no witness — added this sweep** (`w4_c40_the_calm_intent_bands_replay_the_rand_stream`,
  commit `993ecede`).
- `StartSandstorm`/`StopSandstorm`/`WorldClear` (`:117-129`/`:27-30`) →
  `start_sandstorm`/`stop_sandstorm`/`world_clear` (`scene.rs:1084-1106`).
  `world_clear` (clears ONLY `Happening`) landed W4-c40.

**Severity consumers** (the brief's "ongoing severity lerp census"): every
`Sandstorm.Severity` reader is accounted for —
`EmitDust` (`Sandstorm.cs:162/175/176`, → `sand.rs`, f42/dust lane),
`SandstormSky.cs:53` (the sky tint — see the correction below),
`SceneState.cs:138-140` (the fg/bg filter params → `scene.rs:2329-2341`
`sandstorm_foreground_params`/`sandstorm_background_opacity`, landed),
`NetMessage.cs:408`/`MessageBuffer.cs:651` (MP sync, out),
`WorldFile.cs:171/1055` (the save quad, landed wld-roundtrip). **No unported
sim consumer.**

## Item 2 — Wind → projectiles / particles: INERT BY DESIGN (port matches)

The brief asked whether arrows take wind. **In vanilla 1.4.5.8 they do NOT**:
`Main.windPhysics` is `false` (`Main.cs:66841`) and **never set true anywhere
in the decompiled tree** (the only writes are the `false` initializer and the
`windPhysicsStrength = 0.1f` beside it). Both projectile wind paths gate on it:

- the general tail `Projectile.cs:16777-16781` (`ShouldUseWindPhysics()` →
  `if (Main.windPhysics)` at `:22103`), and
- the aiStyle-2 head `Projectile.cs:23207-23210` (`if (Main.windPhysics)`).

So both are dead in vanilla. The port matches exactly:
`ProjectilePool::wind_speed` defaults 0 (`projectile.rs:14911`) and
`set_wind` (`:14988`) is **called only in tests** (`:33075/33102/33122/33147/33184`)
— never in production (`world_sim/projectiles.rs:387` calls `update` without
it). The ported `wind_tail` (`:8950-8963`) + `uses_wind_physics` (`:13677`,
styles 2/8/24/28/29) + the aiStyle-2 head (`:2606-2608`) are faithful but
inert, exactly like the C#. The `WindPhysicsImmunity` set
(`ProjectileID.cs:198-340`, 70 kinds) is therefore also inert; the port's
style-subset is a recorded deviation that only matters if wind is ever
enabled. **No gap — enabling wind would be a NEW feature, not a port.**

Other wind consumers censused (all landed or sibling-owned):
- **Rain drop drift** (`Rain.cs:56` `-= wind*15*40`, `:93` `(wind*18, 14)`):
  the port uses a seed-time `Rain::wind` + sine (`rain.rs:932/948`) — a
  recorded deviation (ANALYSIS "Rain and weather", 16-weather deviation 5).
  **d130 owns rain.rs** — re-homing the live wind is theirs.
- **NPC Detonating Bubble** (`NPC.cs:34903`, aiStyle 70): landed
  (`npc_ai_style70.rs:260-262`), wind pre-stamped (`world_sim/npcs.rs:4245-4248`).
- **Dandelion** (`NPC.cs:46984-47001`, aiStyle 119, kind 628): the downwind
  target gate reads wind. **Kind 628 is not in the port roster** (d42's lane);
  the aiStyle-119 NPC arm is unported with it. Not in-lane.
- **Snow dust** (`Main.cs:13094-13099`), **clouds** (`Cloud.cs:391`),
  **waterfall frame** (`Main.cs:19547` `wFrCounter += wind*2`), **leaves**
  (`TileDrawing.cs:7563-7590`): d130 / g69 / g73 / g68 respectively.
- **Player wind push** (`Player.cs:19795-19817`): landed W4-c40
  (`player.rs::can_be_pushed_by_wind` + the push arm).
- **Wind ambience track 45** (`Main.cs:11570/11582`), **WaterDistortion**
  (`SceneState.cs:129`), **seashell growth** (`WorldGen.cs:72074`): all landed
  (`audio.rs:4518-4522`, `scene.rs:2405-2429`, `growth.rs:1033-1035`).

## Item 3 — Rain start/stop machine: LANDED; remainder is f39's

`Main.UpdateTime`'s rain section (`Main.cs:64833-64949`) is ported in
`rain.rs::update_time` (`:654-718`) + `start_rain`/`change_rain`/`stop_rain`
(`:725-838`), driven once per 60 Hz tick from `weather.rs:178-187`:

- the `ladyBugRainBoost` drain (`:64833-64836`),
- the raining branch: `rainTime -= dayRate`, `Next(86400/dayRate/24 * 2)`
  strength-change roll, `StopRain` at `<= 0` (`:64864-64888`),
- the not-raining start arms: `Next(num2*5.75)`, `cloudBGActive>=1 &&
  Next(num2*4.25)`, `ladyBugRainBoost>0 && Next(num2)` (`:64890-64923`),
- `StartRain`'s `RollLuck(25)` coin arm (consumed-not-spawned), the rainTime
  ladder (`Next(28800,86400)` + the six bonus rolls), the strength ladder
  (`:64659-64727`), `ChangeRain`'s three cloud bands, `StopRain`.

Witnessed: `test_DW_2_1_start_rain_ladder_matches_the_csharp_replay`,
`_change_rain_band_gates_pick_by_cloud_bg_stand_in`, `_stop_rain_clears_the_event_state`,
`test_DW_2_2_start_denominator_8280_at_day_rate_60_vs_496800_at_1`,
`_rain_time_zero_boundary_stops_rain_without_rolling`,
`test_update_time_night_start_roll_fires_without_a_day_time_gate`,
`test_lady_bug_boost_drains_by_day_rate_and_arms_the_third_start_roll`,
`test_DW_2_4_coin_rain_roll_is_consumed_without_a_spawn`,
`test_DW_3_4_coin_rain_luck_selects_the_branch_without_reordering`.

**The two gates the port folds to "always open" are being landed by f39**:
the `:64868-64871` lanterns-up `StopRain` arm inside the raining branch, and
the `:64890` `!slimeRain && !LanternsUp && !NextNightIsLanternNight` gate on
the start arms. f39's diff adds exactly these (new `slime_rain`/`lanterns_up`/
`next_night_lanterns` params to `update_time`). **Do NOT re-port — f39 owns
them.** The `numClouds`-pinned-100 stand-in (which pins `ChangeRain`'s bands)
is g69's live-cloud lane.

## Item 4 — Storm lightning: flash+thunder LANDED; the strike is the admitted bolt lane

The brief asked "strike mechanic?". There are TWO distinct chains:

1. **The flash + thunder chain** (`Main.UpdateWeather`, `Main.cs:58737-58829`
   + `NewLightning` `:58708-58735`): **LANDED** in `rain.rs::thunder_step`
   (`:516-640`) + `new_lightning` (`:350-364`), driven once per tick from
   `weather.rs:159-164`. The tiered thunder delay (`Next(3)`→`Next(1,31)` /
   `Next(2)`→`Next(31,121)` / else `Next(11,61)`), `thunderDistance`,
   `lightningDecay = NextFloat*0.05+0.008`, `lightningSpeed =
   NextFloat*0.05+0.05`, the flash rise/decay, the storming roll with the
   rain/wind tier multipliers (`600/1600` window × the 0.6/0.7/0.8 + 0.5/0.6/0.7
   bands), the graveyard `>= 0.9 && Next(120)` arm, and the M5-SEED-2
   `moreLightning`/`noLightning` gates. The strike cue (position, the case-43
   family re-roll, the pitch) is played via `audio.rs::play_thunder`.
   Witnessed: `thunder_step_emits_one_strike_when_the_delay_runs_out`,
   `the_lightning_micro_seeds_gate_the_storm_roll`,
   `thunder_step_is_deterministic_per_seed`,
   `test_DW_1_1_thunder_arm_runs_once_per_tick_not_per_day_rate_iteration`.
   The thunder SOUND playback is g23's main-sound lane.

2. **The damaging lightning STRIKE** (`WorldGen.SpawnStormLightning`,
   `WorldGen.cs:71864-72018`, called from `:71535`): **NOT ported — and it is
   the already-admitted "(L) bolt geometry" lane**, not a new find. It spawns
   projectile **1091 StormLightning** (500 dmg) via `TrySpawnLightningBolt`,
   which gates on `LightningGenerator.StormLightning.CanHitTarget`
   (`LightningGenerator.cs:243-245` → the full bolt-path `Generate` with
   `SolidTileCollision`), the `IsLightningAttractor` tile set
   (`TileID.cs:624`), `IsSafeFromRain` (`WorldGen.cs:72960`, a
   `PlotTileLine` cast against the rain fall velocity), the player safe-spot
   rects, and the 400-attempt attractor scan. The port's `thunder_step`
   explicitly stands in for this with the flash (rain.rs module note `:25-35`,
   16-weather deviation 7, the "(L) Lightning bolt geometry" task seed).
   Porting it requires the `LightningGenerator` bolt geometry + the 1091
   projectile AI — a large visual+sim lane, **out of a doc-sweep's scope and
   already admitted**. Recorded here so the next sweep does not re-flag it.

## Item 5 — Wind mill / pinwheel tile animation: LANDED + WIRED + WITNESSED

There is no "Windmill" tile in 1.4.5.8; the wind-animated tiles are
**PinWheel 489** (`TileID.cs:1607`), **WeatherVane 490** (`:1609`), and
**GolfCupFlag 493** (`:1615`). All three are ported in two layers:

- **The sim-side animation counters** (`Main.AnimateTiles`,
  `Main.cs:19288-19329`): `drawdata_anim.rs::animate_tiles` (`:440-454`) —
  the 489/493 wind-driven counters (`wind_push` = `clamp(floor(|wind|*10)*sign,
  ±5)`, mod 320 / mod 120) and `animate_weather_vane` (`:459-485`) — the
  verbatim `counter -= 6*counter` arithmetic, the 12-row wrap, the bob roll
  (`NextFloat < |wind|*0.5`, `++bob==8 → 0`). Wired via
  `App::animate_tiles_phase` (`app/frame/world.rs:430-440`). Witnessed:
  `wind_counters_489_493_signed_mod`, `weather_vane_counter_arithmetic_and_wrap`,
  `weather_vane_bob_roll_matches_the_rand_stream`.
- **The draw-side frame selection** (`TileDrawing.GetTileDrawData`):
  `drawdata_cases_b.rs::case_draw_data` — 489 (`:178-185`, the
  `InAPlaceWithWind(2,3)` gate + `(counter/5 + n32+n33)%16*54`), 490
  (`:189-242`, the two bob tables by wind strength + the `%12*36` row), 493
  (`:247-277`, the two wind tiers + the direction-picked row side). Wired via
  `render/tiles.rs:700-712` (the `CaseEnv` carries the live `Wind` +
  `TileAnimator`). Witnessed: `case_489_pinwheel_needs_wind_place`,
  `case_490_weather_vane_bob_tables`, `case_493_golf_flag_wind_tiers`.

The static `drawdata.rs` arms (`:286/293/588`) carry only the unconditional
`top=2` and cite the engine-driven remainder as delegated to slice B — the
"documented deviation" comments there are the pre-S1-G-1f wording; the cases
ARE now wired (the comments are stale but harmless). **No gap.** The 493
draw-flip (`TileDrawing.cs:8983-8997`) is a pure draw detail (d130/g7x lane —
the brief says skip fall/draw claims).

## The FALSE "SandstormSky is dead code" claim — CORRECTED

`16-weather.md:143` and `ANALYSIS.md:8681-8685` (and the ATLAS WEATHER-2 row)
claim the `SandstormSky` sky tint is **"dead code in 1.4.5.8 — registered but
never activated anywhere, so `_opacity` stays 0 and the quad never draws"**,
attributed to a WEATHER-2 "PASS". **This is FALSE.** The activation is
variable-based, so a grep for a literal `Activate("Sandstorm"` misses it:

- `SceneState.cs:97`: `ManageSpecialBiomeVisuals("Sandstorm",
  Main.UseStormEffects && Sandstorm.ShowSandstormVisuals())`.
- `SceneState.cs:313-317`: `if (SkyManager.Instance[biomeName] != null &&
  inZone != ...IsActive()) { if (inZone) SkyManager.Instance.Activate(biomeName,
  ...) }` — with `biomeName == "Sandstorm"`.
- `SkyManager.Instance["Sandstorm"]` IS registered (`ScreenEffectInitializer.cs:71`),
  `Main.UseStormEffects` defaults **true** (`Main.cs:66572`), and
  `ShowSandstormVisuals()` is true during a desert sandstorm. So the sky IS
  activated; `EffectManager.Activate` (`:52-62`) calls `OnActivate` (adds it to
  `_activeSkies`) + `SandstormSky.Activate` (`_isActive=true`). `Update` ramps
  `_opacity` to 1 over ~1 s; `Draw` (`SandstormSky.cs:49-56`) paints the
  full-screen `(0.85,0.66,0.33)*0.8*ColorOfTheSkies * _opacity *
  min(1, Severity*1.5)` quad. The same path activates `BlizzardSky`/`AuroraSky`.

**Consequence for the port:** the sandstorm sky tint quad is a REAL vanilla
visual the port does not draw (the port has the SceneState fg/bg filter pair
but no `CustomSky` tint layer). It is **f42's second-half-visuals lane** — this
sweep records it as a real gap for f42, corrects the false "dead code" claim,
and does NOT port it (visual, sibling-owned). The correction is applied to
`16-weather.md` and `ANALYSIS.md` in this sweep.

## What this sweep changed

1. **Witness** (`scene.rs`, commit `993ecede`):
   `w4_c40_the_calm_intent_bands_replay_the_rand_stream` — the not-happening
   `Next(3)==0` clear arm + the `NextFloat*0.3` ambient arm, replayed
   draw-for-draw on a twin `FastRandom` (the happening band was already
   pinned; the calm bands were not). WRITTEN NOT RUN (code-only).
2. **This report** (`78-doc-sweep-weather.md`).
3. **Doc corrections**: `16-weather.md:143` + `ANALYSIS.md:8681-8685` — the
   FALSE "SandstormSky dead code" claim flipped to "activated via
   `SceneState.cs:97→317`; the tint quad is a real f42-lane visual gap".

## Out of scope (recorded, NOT ported)

- f42: the SandstormSky tint quad + all second-half sandstorm visuals.
- d130: rain drop-field live wind, splash, snowing; wind.rs lanterns_up.
- f39: the rain start/stop slime/lantern gates.
- g67/g68/g69/g73: splash / leaves / clouds / waterfall-draw (incl. the
  wind-driven `wFrCounter`/`wFrame`).
- g23: thunder sound playback.
- d41/d42: the 541-546/580/581 + Dandelion(628) kind rows (and the aiStyle-119
  Dandelion NPC arm with them).
- The damaging lightning strike (1091 + LightningGenerator bolt geometry) —
  the admitted "(L) bolt geometry" lane.
- f42 visuals / d130 fall-draw claims (per the brief).
