# 24 — Spawner, invasions & world events

> Wave 1 · researched 2026-11-12 · sources: decompiled + crates + ROADMAP/ANALYSIS/PORT_MAP

## Scope

Everything that decides *which* enemies appear, *when*, and *how many*: the natural enemy
spawner (`NPC.Spawner` in `decompiled\Terraria\NPC.cs:143-5960` — the fields/ctor `:143-287`,
`SpawnNPC` `:289-342`, `CanSpawnEnemiesNear` `:349-360`, `ShouldSpawnInvasionEnemies`
`:442-471`, `GetSpawnRate` `:473-929`, `GetSpawnArea` `:931-967`, `FindSpawnTile` `:969-1010`,
`GetSpawnWallType`/`PostCheckChosenSpawnTile` `:1022-1048`, `SetSpawnFlagsForChosenTile`
`:1050-1280`, the `SpawnAnNPC` pool ladder `:1282-5238`, `GetBasicSlimeToSpawn` `:5625-5654`,
`FindGroundTile` `:5866-5885`, `HasTileSpawnSpace`/`CanSpawnInTile` `:5398-5434`,
`CheckNotSpawningOnScreen` `:5436-5450`); the four invasions + the Old One's Army
(`Main.UpdateInvasion` `Main.cs:64326-64402`, `InvasionWarning` `:64404-64412`,
`CanStartInvasion` `:64414-64433`, `StartInvasion` `:64435-64475`, the dawn invasion roll
`:65439-65462`); the world-event roll points in the clock (`UpdateTime_StartNight` blood moon
`:65333-65352`, `UpdateTime_StartDay` eclipse `:65421-65430`, slime-rain roll `:64925-64949`,
`StartSlimeRain` `:64742`, meteor night bookend `:65230`, Deerclops midnight `:65121-65134`,
Travelling Merchant `:65004-65042`); the invasion progress bar (`Main.cs:47599-47649`,
`:47687-47694`); and the despawn-encouragement family (`NPC.EncourageDespawn`
`NPC.cs:7249`, `DespawnEncouragement_AIStyle2_FloatingEye_IsDiscouraged` `:52752`,
`DespawnEncouragement_AIStyle3_Fighters_NotDiscouraged` `:60300`).

On the Rust side the module lives in `crates/tr-app/src/spawner.rs` (~2 290 lines with tests),
`crates/tr-app/src/world_sim/npcs.rs` (the invasion machine + the per-tick spawn wiring),
`crates/tr-app/src/world_sim/clock.rs` (dawn/dusk boundary rolls), `crates/tr-app/src/world_sim/weather.rs`
(slime rain / meteor resolution / Deerclops + Travelling Merchant rolls), `crates/tr-app/src/world_sim/lunar.rs`
(the cultist → pillars → Moon Lord chain), `crates/tr-core/src/world.rs` (the event/invasion state),
`crates/tr-core/src/dd2.rs` (lane rates), `crates/tr-core/src/worldgen/meteor.rs` and
`crates/tr-core/src/save.rs` (event-state persistence).

## Key entities

- `crates/tr-app/src/spawner.rs::get_spawn_rate` (`:202-368`) — the whole `Spawner.GetSpawnRate`
  multiplier chain (`NPC.cs:473-929`) with the vanilla cast widths; consumes no RNG (`:203`).
- `crates/tr-app/src/spawner.rs::SpawnFlags` (`:59-91`) — the scene-state snapshot the C# reads
  live off `Player`/`Main` (hardmode, blood/eclipse, moon event, the zone group, the candles,
  WoF active).
- `crates/tr-app/src/spawner.rs::SpawnContext` (`:93-144`) — the per-player spawn inputs
  (player pos/centre, `town_npcs`, `nearby_active_npcs`, `invaders`, `invasion_type`,
  `downed_martians`/`downed_golem`, `any_danger`, `probe_exists`, `sky_behind_player`,
  `player_life_max`, the three bound-NPC gates, `underworld_for_bats`).
- `crates/tr-app/src/spawner.rs::get_spawn_area` (`:155-177`) — `Spawner.GetSpawnArea`
  (`NPC.cs:931-967`): the 0.7× screen spawn rect and the 0.52× safe rect.
- `crates/tr-app/src/spawner.rs::try_spawn_an_npc` (`:588-675`) — one `Spawner.TrySpawnAnNPC`
  attempt (`NPC.cs:308-342` + `FindSpawnTile` `:969-1010`): the rate/max gates, the 50-candidate
  scan, the skyMob latch, `HasTileSpawnSpace`, `CheckNotSpawningOnScreen`, `FindGroundTile`,
  `PostCheckChosenSpawnTile`, then `spawn_npc_at`.
- `crates/tr-app/src/spawner.rs::spawn_npc_call` (`:472-484`) — `NPC.SpawnNPC`
  (`NPC.cs:289-307`): the `noSpawnCycle` latch + the dead-player gate, the named
  `Main.SwapRandom("SpawnNPC")` stream discipline (`Main.cs:17886`).
- `crates/tr-app/src/spawner.rs::spawn_npc_at` (`:706-1146`) — the `SpawnAnNPC` selection ladders
  for the ported kinds (`NPC.cs:1282-5238`): the skyMob dispatch (`:1354-1412`), the water
  ladder (`:1795-2082`), the day slime fallthrough, the night zombie/eye ladder
  (`:4520-4810`), the underground/cavern ladder (`:4846-5150`), the bound-NPC arms, the
  worms/Tim/Fire Imp, the probes (`:4164-4200`), and the bat/slime tails.
- `crates/tr-app/src/spawner.rs::is_ices_snow` (`:1155-1157`) / `spawn_martian_probe`
  (`:1164+`) / `has_tile_spawn_space` (`:372`) / `check_not_spawning_on_screen` (`:396`) /
  `find_ground_tile` (`:416`) / `post_check_chosen_spawn_tile` (`:436`).
- `crates/tr-app/src/world_sim/npcs.rs::can_start_invasion` (`:241`) / `::start_invasion`
  (`:248-276`) — `Main.CanStartInvasion` (`:64414-64433`) and `Main.StartInvasion`
  (`:64435-64475`) incl. the type-3 `+40+20·players` bonus and the type-4 `160+40·players`
  override.
- `crates/tr-app/src/world_sim/npcs.rs::update_invasion_state` (`:1338-1407`) —
  `Main.UpdateInvasion` (`Main.cs:64326-64402`) + the `DrawInvasionProgress` fade head
  (`:47712-47736`).
- `crates/tr-app/src/world_sim/npcs.rs::invasion_warning` (`:1208`) — `Main.InvasionWarning`
  (`Main.cs:64404-64412`).
- `crates/tr-app/src/world_sim/npcs.rs::should_spawn_invasion_enemies` (`:1749-1765`) —
  `NPC.ShouldSpawnInvasionEnemies` (`NPC.cs:442-471`).
- `crates/tr-app/src/world_sim/npcs.rs::invasion_spawn_pick` (`:1770-1900`) — the four
  invasion tables (`NPC.cs:1419-1540`).
- `crates/tr-app/src/world_sim/npcs.rs::report_invasion_kill` (`:1717`) /
  `::check_invasion_progress_display` (`:1443`) / `::npc_invasion_group` (`:1411`) —
  `Main.ReportInvasionProgress`/`CheckInvasionProgressDisplay` (`Main.cs:47687-47694`,
  `:47599-47649`) + `NPC.GetNPCInvasionGroup`.
- `crates/tr-core/src/world.rs::roll_blood_moon` (`:565-589`) / `::roll_eclipse` (`:594-611`) /
  `::roll_morning_invasion` (called from `world_sim/clock.rs:73`) / `::downed_mech_boss_any`
  (`:614`) — the boundary event rolls.
- `crates/tr-app/src/world_sim/clock.rs::advance_time` (`:13-77`) — `Main.time`'s day/night
  wrap with the dusk blood-moon and dawn eclipse/invasion rolls wired in.
- `crates/tr-app/src/world_sim/weather.rs::slime_rain_roll` (`:279`) / `::start_slime_rain`
  (`:295`) + the expiry decay (`:172-177`) — `Main.cs:64925-64949` / `:64742`.
- `crates/tr-core/src/worldgen/meteor.rs::night_roll` / `::resolve` — the meteor bookend
  (`Main.cs:65230`) and `HandleMeteorFall`'s dusk resolution; wired from `weather.rs:254-271`.
- `crates/tr-app/src/world_sim/lunar.rs::trigger_lunar_apocalypse` (`:35`) /
  `::update_lunar_apocalypse` (`:115`) / `::start_impending_doom` (`:154`) /
  `::update_moon_lord_countdown` (`:169`) / `::anyone_near_cultists` (`:190`).

## Dependencies

- **Calls into:** `tr_core::World` (the event flags/invasion state, `world.rs:100-310`);
  `spawner::FastRandom` / `app::rand01`; `tr_core::tiledata::TILE_SOLID`; `Npc::spawn_*`
  constructors in `crates/tr-app/src/npc.rs`; `crate::town::quick_find_home`; `scene_scan`
  (the zone flags that feed `SpawnFlags`); `luck.rs` (only as a snapshot); `shop.rs`
  (`setup_travel_shop`).
- **Called by:** `world_sim/npcs.rs`'s per-tick `npc_step` block (`:2206-2260`, `:3381-3443`);
  `world_sim/clock.rs::advance_time` from the fixed-step weather tick; `world_sim/weather.rs`;
  `world_sim/lunar.rs`; `ui::draw_invasion_progress` from `frame.rs`; `save.rs` for the
  persisted half.
- **Shared data structures:** `World.invasion_type/invasion_size/invasion_size_start/invasion_x/
  invasion_warn/invasion_delay/invasion_arm_pending` + `invasion_goblins` (`world.rs:135-137`,
  `:275-290`); `World.blood_moon`/`eclipse`/`pumpkin_moon`/`snow_moon`/`slime_rain*`/
  `spawn_meteor`/`hard_mode`/`altars_smashed`/`shadow_orb_*` (`world.rs:112-218`); the `dd2_*`
  and `tower_*`/`lunar_*`/`moon_lord_countdown` event fields (`world.rs:220-263`); `wof_*`
  (`world.rs:291-300`); `Npc.home`/`homeless_despawn` (`npc.rs:2513-2527`); `App.spawn_no_cycle`
  (the `noSpawnCycle` latch), `App.invasion_progress_*`/`settings.event_progress` (the bar).

## Port status

| C# subsystem | Status | Rust location | Evidence / notes |
| --- | --- | --- | --- |
| `Spawner.GetSpawnRate` depth/night/eclipse chain (`NPC.cs:478-558`) | PORTED | `spawner.rs::get_spawn_rate` `:215-244` | verbatim order + cast widths; unit-pinned (`deep_rock_layer_multiplies` 240/9, `blood_moon_night_compounds` 108/10) |
| `GetSpawnRate` zone group (dungeon/sandstorm/UD/jungle-town/evil/meteor/temple/hallow-deep) (`:573-657`) | PORTED | `:248-291` | incl. the four jungle town tiers (`jungle_town_tiers` 96/13, 132/12, 168/11) |
| `GetSpawnRate` WoF override + nearby ramps + candles + player state (`:667-748`) | PORTED | `:292-349` | the `isNearFairy` arm (`:744-748`) is NOT ported |
| `GetSpawnRate` floor/cap (`:749-756`) | PORTED | `:350-357` | `the_rate_floor_and_cap_hold` (48 → 60, max 15) |
| `GetSpawnRate` moon-event raid pacing (`:771-775`) | PORTED | `:360-363` | `moon_event_forces_the_raid_pacing` (20, 11) |
| `GetSpawnRate` `this.invaders` override (rate 20, max `5·(2+0.3·players)`) (`:781-785`) | MISSING | — | no `invaders` branch in `get_spawn_rate`; the invasion path has its own unrated loop (see deviations) |
| `GetSpawnRate` pre-Skeletron dungeon rate 10 (`:786-789`) | PORTED | `:364-366` | `dungeon_pre_skeletron_is_rate_ten` (10 / 180 post) |
| `GetSpawnRate` getGoodWorld / Journey / Skyblock (`:757-770`, `:790-793`) | MISSING (documented constants-off) | `:197-201` | needs M5-SEED-1 / M6-CREATIVE-1 |
| `GetSpawnRate` town-NPC distribution block with `Main.rand` (`:799-923`) | PORTED | `get_spawn_rate` `:406-510` | the S1-F-11 block — `townNPCs` 1/2/≥3 × depth × graveyard → `noWorms`/`spawnFriendly` on `SpawnRate`, the ×1.25…×3 rescales with the vanilla casts; skyblock/shadow-candle/expertMode/OOA/remix constants-off, the graveyard input in `SpawnFlags` for the M2-BUFF scan; mutation-checked draw-order tests |
| `GetSpawnRate` `RollOnlyBadLuckExtreme(50)` tail (`:924-928`) | MISSING | — | luck does not reach the spawn stream |
| `GetSpawnArea` + `FindSpawnTile` + tile gates | PORTED (partial) | `get_spawn_area` `:155`, `try_spawn_an_npc` `:588-675`, `has_tile_spawn_space` `:372`, `check_not_spawning_on_screen` `:396`, `find_ground_tile` `:416`, `post_check_chosen_spawn_tile` `:436` | the scope/biome-gun range extension (`NPC.cs:943-967`) and the `wallHouse` arm of the solid check are unported (documented `:151-154`, `:622-624`) |
| `SpawnNPC` latch + named stream | PORTED | `spawn_npc_call` `:472-484` | `the_latch_consumes_one_call_and_clears`, `the_named_stream_is_deterministic_per_seed` |
| `SpawnAnNPC` night ladder (eyes/zombies) | PORTED (partial) | `spawn_npc_at` `:877-983` | `the_night_ladder_produces_both_eyes_and_zombies`, `the_full_moon_doubles_the_eye_roll`, `the_short_circuit_order_is_vanilla`, `hardmode_night_yields_the_wandering_eye` |
| `SpawnAnNPC` underground/cavern ladder | PORTED (partial) | `:984-1145` | worms/Tim/Fire Imp/skeleton/bat/slime arms land; `the_deep_cavern_spawns_the_skeleton_family_and_the_bat`, `the_depth_bands_split_tim_and_fire_imp` |
| `SpawnAnNPC` water ladder | PORTED (partial) | `:788-845` | shark/piranha/goldfish arms; `the_water_ladder_rolls_the_fish`; 63/103 and the critter scans documented unmodelled |
| `SpawnAnNPC` skyMob dispatch + Martian probes | PORTED | `:725-781`, `:1164+` | `the_sky_probe_arm_spawns_the_probe`, `the_probe_is_unique`, `the_surface_probe_arm_spawns_at_the_edge_band`, `the_probe_arms_refuse_danger_and_the_golem_gate` |
| `GetBasicSlimeToSpawn` + holiday slimes | PORTED (partial) | `spawn_basic_slime` | the `endlessHalloween`/`endlessChristmas` gates run under the micro-seed flags (`:1408-1462`) |
| Invasion state machine | PORTED | `can_start_invasion` `:241`, `start_invasion` `:248`, `update_invasion_state` `:1338-1407`, `invasion_warning` `:1208` | `start_invasion_sizes_the_army_and_picks_the_side`, `the_army_marches_and_warns`, `the_kill_credit_drains_and_defeats` |
| `ShouldSpawnInvasionEnemies` | PARTIAL | `should_spawn_invasion_enemies` `:1749-1765` | the ±3000-px window and the surface band are verbatim; the `invasionX` near `maxTilesX/2` town-NPC fallback (`NPC.cs:455-468`) is unported and its `Next(3)` draws nothing |
| Invasion spawn tables (types 1-4) | PARTIAL | `invasion_spawn_pick` `:1770-1900` | roll order preserved; the RayGunner 382 / BrainScrambler 381 / ScutlixRider 390 outcomes consume their rolls and map to the port's nearest kind; the Saucer 395 gate reads `false` (`:3400`) |
| Invasion spawn placement + cadence | PARTIAL | `world_sim/npcs.rs:3386-3443` | one pick per fixed step (no `Next(spawnRate)` gate, no `maxSpawns` cap), placed at `(playerX-600, playerY-400)` — not `FindSpawnTile` |
| Invasion progress bar | PORTED | `report_invasion_kill` `:1717`, `check_invasion_progress_display` `:1443`, `update_invasion_state` `:1339-1360` | `the_kill_credit_reports_the_bar_state`, `the_near_scan_refreshes_the_report`, `the_fade_head_ramps_the_alpha`, `bug_08_the_off_mode_zeroes_the_invasion_bar` |
| Blood-moon roll | PORTED | `world.rs::roll_blood_moon` `:565-589`, `clock.rs:37-38` | `the_dusk_boundary_rolls_and_reports_the_blood_moon`; `tenth_anniversary` and the eye-spawn gate are pinned `false` |
| Solar-eclipse roll | PORTED | `world.rs::roll_eclipse` `:594-611`, `clock.rs:69-72` | `the_dawn_boundary_rolls_and_reports_the_eclipse`; the C#'s `stopEvents` gate has no port (`clock.rs:69-71`) |
| Dawn invasion roll | PORTED | `world.rs::roll_morning_invasion` ← `clock.rs:73` | `the_morning_roll_arms_the_invasion` |
| Slime Rain | PORTED | `weather.rs::slime_rain_roll` `:279`, `start_slime_rain` `:295`, decay `:172-177`; `world.rs:194-201` | `the_slime_rain_roll_gates_and_king_counter` (the 32 400-54 000 band, the afternoon refusal, the King + hysteresis) |
| Meteor | PORTED | `worldgen::meteor::night_roll`/`resolve`, `weather.rs:254-271` | `the_meteor_roll_and_boss_death_flag`; no `Shimmer`/dust shower path |
| Orb-break reaction | PORTED | `tr-core/src/edit.rs::check_orb` | the 2×2 anchor snap + the third-break boss summon (M4-EVENT-3c) |
| Pumpkin / Snow Moon | PARTIAL | `world.rs:203-205`, `spawner.rs:63-64`, `:237`, `:360-363` | the flags + the raid pacing are ported; **no wave roster** — `spawn_npc_at` never reads `moon_event` (grep: only `SpawnFlags`, `get_spawn_rate` and tests) |
| Old One's Army (DD2) | PORTED | `tr-core/src/dd2.rs`, `world_sim/dd2` lane spawn, `world.rs:220-274` | the difficulty tiers, the wave rosters, the lane rate; `save.rs:437-535` persists the state |
| Cultist → pillars → Moon Lord | PORTED | `world_sim/lunar.rs` (`:35-212`), `world.rs:230-263` | `trigger_lunar_apocalypse`, the shield family, the countdowns, `anyone_near_cultists` |
| Lantern Night | MISSING | — | no state anywhere (`grep lantern_night` → 0 in `crates/`); C# `GameContent\Events\LanternNight.cs` |
| Birthday Party | MISSING | — | no party state; only `World.unlocked_party_girl` (the arrival flag) and `StockWhen::PartyGirlPresent` |
| Sandstorm event | PARTIAL | `scene_scan`/`audio.rs::SceneMetrics.ZoneSandstorm` `:437-438`, `spawner.rs:251-253` | the zone flag feeds the spawn-rate branch; the event's own scheduler (`GameContent\Events\Sandstorm.cs`) is unported |
| `EncourageDespawn` family | PARTIAL | `world_sim/npcs.rs:2262-2270` | one ad-hoc cull: the Eye of Cthulhu's day-time climb-away; `NPC.cs:7249`/`:52752`/`:60300` otherwise unmodelled |
| Travelling Merchant visit cycle | PORTED | `weather.rs:335-341`, `:343+` | `travel_visit_roll` mirrors `Main.cs:65024-65025`'s day-rate roll shape |
| Deerclops midnight spawn | PORTED | `weather.rs` (the `:330` spawn arm) | `Main.cs:65121-65134`'s crossing + gates |

## Tests

`grep "#\[test\]"` results (file :: test, with line):

**`crates/tr-app/src/spawner.rs` — 33 tests** (the spawn-rate chain and the pool ladders):
`::surface_day_is_the_plain_default` (`:1281`), `::night_surface_multiplies` (`:1289`),
`::deep_rock_layer_multiplies` (`:1297`), `::the_nearby_ramp_speeds_up_empty_areas` (`:1306`),
`::blood_moon_night_compounds` (`:1317`), `::jungle_town_tiers` (`:1328`),
`::dungeon_pre_skeletron_is_rate_ten` (`:1342`), `::the_rate_floor_and_cap_hold` (`:1359`),
`::moon_event_forces_the_raid_pacing` (`:1372`), `::the_endless_holiday_flags_gate_the_spawner_holiday_arms`
(`:1417`, a 4000-seed stream-fingerprint sweep), `::the_vampirism_pick_places_the_zoologist_at_the_spawn`
(`:1468`), `::the_night_ladder_produces_both_eyes_and_zombies` (`:1510`),
`::the_full_moon_doubles_the_eye_roll` (`:1532`), `::the_short_circuit_order_is_vanilla` (`:1561`),
`::hardmode_night_yields_the_wandering_eye` (`:1586`), `::the_underground_ladder_only_slimes` (`:1600`),
`::the_deep_cavern_spawns_the_skeleton_family_and_the_bat` (`:1629`),
`::the_ladder_is_deterministic_per_seed` (`:1675`), `::spawn_rate_matches_the_c_modifiers` (`:1711`),
`::spawns_land_outside_the_safe_area` (`:1757`), `::night_spawns_zombies_and_eyes_day_spawns_slimes`
(`:1777`), `::nearby_cap_blocks_spawns` (`:1807`), `::spawn_space_requires_clear_tiles` (`:1818`),
`::the_latch_consumes_one_call_and_clears` (`:1831`), `::a_missing_eligible_player_is_not_a_latched_call`
(`:1849`), `::the_named_stream_is_deterministic_per_seed` (`:1860`),
`::the_depth_bands_split_tim_and_fire_imp` (`:1929`), `::the_water_ladder_rolls_the_fish` (`:1985`),
`::the_cavern_rolls_worm_heads` (`:2071`), `::the_sky_probe_arm_spawns_the_probe` (`:2153`),
`::the_probe_is_unique` (`:2172`), `::the_surface_probe_arm_spawns_at_the_edge_band` (`:2186`),
`::the_probe_arms_refuse_danger_and_the_golem_gate` (`:2204`).

**`crates/tr-app/src/world_sim/npcs.rs` — 21 module-24 tests**: `::the_morning_roll_arms_the_invasion`
(`:4542`), `::start_invasion_sizes_the_army_and_picks_the_side` (`:4564`),
`::the_army_marches_and_warns` (`:4589`), `::the_kill_credit_drains_and_defeats` (`:4615`),
`::the_invasion_spawn_window_and_table` (`:4640`), `::the_snow_globe_arms_the_frost_legion` (`:4844`),
`::the_frost_kill_credit_drains_and_defeats` (`:4876`), `::the_frost_spawn_table_picks_the_trio` (`:4893`),
`::the_pirate_map_arms_the_invasion_with_the_bonus` (`:4945`), `::the_pirate_kill_credit_drains_and_defeats`
(`:4979`), `::the_pirate_spawn_table_picks_the_crew` (`:4999`), `::the_type4_machine_sizes_the_invasion`
(`:5201`), `::the_martian_kill_credit_drains_and_defeats` (`:5218`),
`::the_martian_spawn_table_picks_the_ground_four` (`:5235`), `::the_kill_credit_reports_the_bar_state`
(`:5096`), `::the_near_scan_refreshes_the_report` (`:5113`), `::the_fade_head_ramps_the_alpha` (`:5160`),
`::the_defeat_resets_the_bar_state` (`:5180`), `::bug_08_the_off_mode_zeroes_the_invasion_bar` (`:5312`),
`::the_probe_machine_detects_and_flashes` (`:5372`), `::the_probe_hover_gate_needs_the_player_below` (`:5398`).

**`crates/tr-app/src/world_sim/clock.rs`**: `::the_dusk_boundary_rolls_and_reports_the_blood_moon`
(`:160`), `::the_dawn_boundary_rolls_and_reports_the_eclipse` (`:179`) plus the sundial tests
(`:293`, `:303`, `:314`).

**`crates/tr-app/src/world_sim/weather.rs`**: `::the_slime_rain_roll_gates_and_king_counter` (`:880`),
`::the_meteor_roll_and_boss_death_flag` (`:1031`).

**`crates/tr-core/src/save.rs`** `:1280-1297` — the event-state round-trip (`dd2_*`, `slime_rain`,
`downed_moon_lord`, the invasion fields).

## Deviations & risks

1. **`GetSpawnRate`'s town-NPC distribution block is unported** (`NPC.cs:799-923`) — *behavioural,
   high*. **CLOSED 2026-11-13** (`S1-F-11-town-spawn-rng`): the block is ported verbatim into
   `get_spawn_rate` — the gate, the underworld/surface ladders, the graveyard arms, the
   `noWorms`/`spawnFriendly` carry-outs on `SpawnRate` and the consumer wiring (skyMob `:980/:984`,
   piranha `:1929`, Devourer `:3701`, Giant Worm `:4816`); the discarded `rng` is consumed in the
   C# order (mutation-checked). Constants-off inside the block: skyblock `lowTiles`, the shadow
   candle's `townNPCs = 0`, `Main.expertMode`, `ZoneOldOneArmy`, `inRemixStartingArea`; the
   graveyard input landed in `SpawnFlags` (`zone_graveyard`) awaiting the M2-BUFF scene scan.
   **Errata found while porting**: the port's moon-event pacing arm (`spawner.rs:360-363`) applies
   unconditionally, but the C# gates it on `(Main.remixWorld || player.position.Y < worldSurface *
   16.0)` (`NPC.cs:771`) — an underground player during a moon event should keep the un-paced rate.
   Pre-existing, one row for a follow-up (S1-F-13's neighbourhood).
2. **No `invaders` spawn-rate override** (`NPC.cs:781-785`) — *behavioural, high*. C# forces
   `spawnRate = 20` / `maxSpawns = 5·(2+0.3·players)` while an invasion window is open. The port
   instead fires `invasion_spawn_pick` **once per fixed step** with no rate roll and no nearby cap
   (`world_sim/npcs.rs:3386-3423`), and skips the natural spawn call entirely when it fires
   (`:3418`). Net effect: ~20× the vanilla invasion density and no crowd control. ROADMAP M4-EVENT-1a's
   "2-second spawn pass" text does not describe the current code.
3. **Invasion enemies are not placed through `FindSpawnTile`** — *behavioural, medium*. The port
   spawns them at `(playerX - 600, playerY - 400)` clamped to a tile row
   (`world_sim/npcs.rs:3429-3443`); C# picks a validated tile through the 50-candidate scan with
   `HasTileSpawnSpace`, the safe-area and on-screen checks (`NPC.cs:969-1048`). Invasion NPCs can
   therefore materialise inside blocks or in mid-air.
4. **`ShouldSpawnInvasionEnemies`'s town fallback is missing** (`NPC.cs:455-468`) — *behavioural,
   low/medium*. The `invasionX` within ±5 tiles of `maxTilesX/2` + a resident town NPC inside
   3000 px arm (with its `Next(3)`) is unported; documented in the port's doc comment
   (`npcs.rs:1745-1748`).
5. **`invasion_spawn_pick` draws from `app.item_rng`, not the named `SpawnNPC` stream**
   (`world_sim/npcs.rs:3407`) — *determinism, medium*. C# draws every one of these from `Main.rand`
   inside `SpawnAnNPC`; the port keeps its own stream discipline but the streams cannot be compared
   tick-for-tick with the source, and the natural-spawn stream is left unadvanced on invasion ticks.
6. **Kind coverage gaps in the tables** — *behavioural, low-medium*. The type-4 table's RayGunner
   382 / BrainScrambler 381 / ScutlixRider 390 arms consume their rolls and land on the port's
   nearest kind, and the `!AnyNPCs(395)` Saucer gate is pinned `false` (`:3400`) — so the Saucer
   branch can never fire. Documented in the `invasion_spawn_pick` doc (`:1775-1779`, `:1800-1804`).
7. **Blood-moon roll inputs are pinned** — *behavioural, low*. `roll_blood_moon` receives
   `tenth_anniversary = false` and the eye-spawn gate `= false` (`clock.rs:38`), so the anniversary
   1/6 variant and the C#'s `:65335-65341` eye gate never apply. Not admitted in ROADMAP/ANALYSIS.
8. **Eclipse roll has no `stopEvents` gate** — *behavioural, low*, admitted (`clock.rs:69-71`): the
   port always evaluates the hardmode + mechs roll, so an eclipse can start while another stop-event
   (Lantern Night/party) would have suppressed it. Today the difference is unreachable because
   those events are unported — it becomes real the moment they land.
9. **Pumpkin/Snow Moon are pacing-only** — *behavioural, high*. `World.pumpkin_moon`/`snow_moon`
   exist (`world.rs:203-205`) and `get_spawn_rate` applies the raid pacing (`:360-363`), but
   `spawn_npc_at` never reads `moon_event` — the C# wave tables inside `SpawnAnNPC`
   (`NPC.cs:2560-2960`'s moon-event arms) are entirely absent, so a moon event floods the world
   with ordinary surface enemies instead of the wave roster. ROADMAP's M4-EVENT-2 row has no DONE
   entry; treat any claim of completion as unverified.
10. **Lantern Night, Birthday Party, the Sandstorm event and the cultist ritual delay are missing**
    — *behavioural, medium* (each). No state exists in `World` (grep for `lantern_night`/`party_time`/
    a sandstorm timer → nothing). Consequences beyond the absent visuals: `Luck`'s lantern-night
    term, `ShopHelper`'s party prices, `Sandstorm.StartSandstorm`'s gates and the
    `SpawnAnNPC` arms gated on `BirthdayParty.PartyIsUp` (`NPC.cs:1643`, `:2593`) and graveyard
    flags are inert.
11. **Despawn encouragement is a single ad-hoc cull** — *behavioural, medium*. Only the Eye of
    Cthulhu's day-time climb-away is modelled (`world_sim/npcs.rs:2262-2270`); `EncourageDespawn`
    (`NPC.cs:7249`) and the `DespawnEncouragement_*` family (`:52752`, `:60300`) are unported, so
    off-screen enemies persist forever instead of being recycled.
12. **Dual invasion flags** — *structural, medium*. `World.invasion_goblins` and
    `World.invasion_type` both encode "an invasion is running"; `update_invasion_state` re-derives
    the former from the latter every tick (`npcs.rs:1362`), but `can_start_invasion`/`start_invasion`
    gate and write `invasion_goblins` (`:242`, `:253-256`). Any future writer that updates one
    without the other silently diverges.
13. **M4-SPAWN-1/2's DONE rows over-claim** — *documentation, medium*. Re-verifying against the
    source: the ported chain is faithful through `NPC.cs:789`, but `:790-928` (skyblock, the town
    block, the luck tail) is not ported and no test covers it. Recommend the row be re-worded
    rather than read as complete.
14. **Minor**: the `get_spawn_area` doc (`spawner.rs:151-154`) and the `SetSpawnFlagsForChosenTile`
    liquid check (`post_check_chosen_spawn_tile` `:436-460`) are honest about their unported halves
    (scopes, the dual-dungeon/invasion tile cases); the `is_ices_snow` set is a hand-written
    `matches!` (`:1155-1157`) instead of the generated `TileID.Sets.IcesSnow` (`TileID.cs:561`) —
    *cosmetic/data*, already noted in the source comment.

## Task seeds

- **S — Port the `invaders` spawn-rate override and gate the invasion pick** (`GetSpawnRate`
  `NPC.cs:781-785` + a `nearby >= maxSpawns` check): add the `invaders` branch to `get_spawn_rate`
  and route `invasion_spawn_pick` behind `rng.next(spawn_rate)`; feeds deviation #2. No new state.
- **S — Port `ShouldSpawnInvasionEnemies`'s town fallback** (`NPC.cs:455-468`): needs the town
  roster's live NPC scan + `world.max_tiles_x`; consumes its `Next(3)` exactly where the C# does.
  Closes deviation #4.
- **M — Port `GetSpawnRate`'s town-NPC distribution block** (`NPC.cs:799-923`), including the
  graveyard arms and the `noWorms`/`spawnFriendly` writes; requires a `SpawnFlags`-adjacent
  carry-out struct plus a graveyard flag in the scene scan. Depends on the graveyard scene work
  (M2-BUFF family). Closes deviation #1. — **DONE 2026-11-13** (`S1-F-11-town-spawn-rng`): the
  carry-outs ride `SpawnRate` (`no_worms`/`spawn_friendly`), the graveyard flag landed in
  `SpawnFlags` (constants-off until the M2-BUFF scan feeds it), the consumers (skyMob, piranha,
  Devourer, Giant Worm) read the flags.
- **M — Moon-event wave rosters** (`SpawnAnNPC`'s pumpkin/snow-moon arms `NPC.cs:2560-2960`):
  the per-wave kind tables behind `World.pumpkin_moon`/`snow_moon` in `spawn_npc_at`, plus the
  wave counter. Bigger than it looks — most of the wave kinds are already ported by M4-EVENT-8..19.
- **M — Lantern Night + Birthday Party + Sandstorm event + the cultist ritual delay**: four small
  schedulers on `World` (flags + timers), the C# sources are `GameContent\Events\LanternNight.cs`,
  `BirthdayParty.cs`, `Sandstorm.cs`, `CultistRitual.cs`; they unlock the eclipse `stopEvents`
  gate (deviation #8), the party shop/luck terms and the `PartyIsUp` spawn arms (`NPC.cs:1643`,
  `:2593`). Sequence: these first, then the stopEvents gate.
- **M — Despawn encouragement**: port `EncourageDespawn` (`NPC.cs:7249`) + the two
  `DespawnEncouragement_*` helpers and call them from the per-NPC step; unblocks correct enemy
  recycling off-screen. Depends on nothing in this module.
- **S — Re-verify + re-word the M4-SPAWN-1/2 and M4-EVENT-1a/2 ROADMAP rows** to name the
  unported `GetSpawnRate` tail and the missing moon-event rosters, and to correct the "2-second
  spawn pass" claim (deviation #2/#9/#13).
- **S — Align the invasion pick's RNG with the named `SpawnNPC` stream** (or document why the
  item stream is deliberate) so the determinism story is uniform (deviation #5).
- **L — Faithful invasion spawn placement**: run the invasion arm through `FindSpawnTile` +
  `HasTileSpawnSpace`/`CheckNotSpawningOnScreen` instead of the `(x-600, y-400)` offset
  (deviation #3). Best done together with the rate-override task above.
- **S — `is_ices_snow` from the generated set**: move `TileID.Sets.IcesSnow` (`TileID.cs:561`)
  into `tools/GenTileData`'s output so the spawn ladder reads generated data (deviation #14).
