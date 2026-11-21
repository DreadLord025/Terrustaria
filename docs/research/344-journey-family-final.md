# 344 — The Journey family FINAL (W4-c24, code-only) + the JOURNEY-SEAL receipts

**Renumber chain: 308 → 344** (MICRO RENAME, §8.7 AMENDMENT 3 FROZEN table,
`150-research-number-registry.md` at `origin/task/W4-d54` `9dddb32b4`:
`c24 | 308-journey-family-final | 935c47617 16:43:22 | **344**`). The
308 self-pick below is SUPERSEDED — §8.5's g13 KEEPS 308 (d21→340 §8.6,
g65→346 §8.7); this doc is 344. No content change beyond the
header/banner/refs; the FINAL body stands as written at `935c47617`.

**Registry take (historical, superseded by the chain above):** 308.
research/150 §7.4 declared **308+ FREE** (the
round-3 extension consumed 304-307); a fresh sweep at write time confirmed:
`git rev-list --objects --all -- docs/research/` maxed at **298** (280-283
+ 298 above the round-3 block), and the on-disk Desktop worktrees maxed at
**280** (g56 274-void-family-bible, f21 276, f45 279, g61 273, g50 271).
No ref, no worktree carried 308 at write time. Claimed by W4-c24's final resume
(`task/W4-c24-journey-ui`).

Status: **FINAL for the single-player Journey family.** Every C# surface
below is landed on this branch with its commit hash; the merge-time
dedupe table (§8) is the merger's contract. All witnesses WRITTEN NOT
RUN (code-only waves — no cargo/tests/rustfmt/graphify invoked).

---

## 1. The family map — C# surface → port owner → commit

| C# surface | Port | Commit(s) |
| --- | --- | --- |
| `CreativeUI.cs` toggle/categories (`:77-78` sheets, `:98-111` gate/draw, `:146-217` toggle) | `ui/journey.rs` (layout/draw), `app/journey_clicks.rs` (`journey_available`/`journey_toggle_menu`/`journey_close_menu`/`journey_tick`) | `0217b62fd` (sheets), `f8721ac8a` (UI) |
| `UICreativeInfiniteItemsDisplay.cs` research page (`:207-498`: 170x170 panel, cog trio, 9-frame pistons, sparks, progress text, confirm) | `ui/journey.rs` (draw + `tick_animation`/`stop_animation`), `app/journey_clicks.rs::journey_research_clicks` | `f8721ac8a` |
| `CreativeUI.SacrificeItem` (`:288-329`) + `SacrificeWhatYouCan` (`UICreativeInfiniteItemsDisplay.cs:441-458`) | `ui/journey.rs::sacrifice_amounts`/`sacrifice_confirm`, the excess return `journey_clicks.rs::return_sacrifice_slot` (`:320-323`) | `f8721ac8a` |
| `ItemsSacrificedUnlocksTracker.cs` counts/caps/newly (`:19-144`, `:208-216`) | `research.rs::ResearchTracker` (`register_sacrifice` cap-clamp `:131`, refused arms `:122-130`, `newly` set `:135-137`), `creative_sacrifices.rs::SACRIFICES` (6139 generated rows) | `0217b62fd`, `ad88ba946` (cap-clamp + newly) |
| `ItemsSacrificedUnlocksTracker.LastEditId`/`MarkContentsDirty`/`CountFullyResearchedItems` (`:23`, `:222-225`, `:60-75`) | `research.rs::last_edit_id`/`count_fully_researched` (bumps on registration + Reset only; Load never marks dirty) | `face9df72` (c24c) |
| `AchievementsHelper.CheckResearchAchievement` (`:387-399`) + callers (`Player.cs:325` forced, `:25703` per-tick) | `achievements.rs::check_research` (`last_research_version` instance latch — one manager per App), `app/mod.rs::enter_loaded_world` forced call, `world_sim/player.rs` post-borrow drain | `face9df72` (c24c); the f30b PREP `a7750cfe0` superseded (§8) |
| The sacrifice tooltip row (`Main.cs:21318-21333`: 17-context whitelist + difficulty 3 + `TryGetSacrificeNumbers` → `CommonItemTooltip.CreativeSacrificeNeeded` in `Colors.JourneyMode`) | `ui/tooltip.rs` (`TooltipContext.slot_context/journey_difficulty/research` + the row), `ui/mod.rs::hover_context` (HoverRef→context), `ui/journey.rs::Hover.context` (grid 29/34), `search_fits` keeps -1 (`Item.cs:117`) | `face9df72` (c24c) |
| `UIDynamicItemCollection.cs:38-97` duplication grid + `UICreativeItemGrid.cs:18-34` contexts 29/34 + `SortingSteps.cs:91-146` ByUnlockStatus | `ui/journey.rs::grid_items`/`grid_context`, `journey_clicks.rs::journey_duplication_clicks` (action-5 max-stack pickup, ctx-34 refusal) | `f8721ac8a` |
| `UISearchBar.cs`/`UIWrappedSearchBar.cs` search (`:63` 50-char cap, `:156-164` Enter/Escape, `:153-246` cancel/right-click/outside-stop) + `ItemFilters.cs:28-48` BySearch tooltip scan | `ui/journey.rs::search_fits` + the search bar draw/hit, `app/loop.rs` event.text seam, `journey_clicks.rs` search arms | `4e31ff97f`; the row-set gates re-seated `face9df72` |
| `CreativePowers.cs` FreezeTime (`:1441-1472`) / ModifyTimeRate (`:861-1000`) / the four clock buttons (`:769-860`) | `creative.rs` (`PowerId::FreezeTime/TimeRate/SetDay/SetNight/SetNoon/SetMidnight`, `time_rate`/`button_target`), `app/frame/world.rs::time_phase` (the rate folds the clock tick), `journey_clicks.rs::journey_time_clicks` | `f8721ac8a` (UI), the sim seam pre-existing |
| `WorldGen.GetWorldUpdateRate` (`:72020-72028`) + `Main.UpdateTimeRate` (`:6199-6226`) + `UpdateWorld`'s rate-0 return (`:71536-71540`) | `world_sim/growth.rs::update_growth` (powers-derived desired rate: dial 1 / asleep x5 / target; FreezeTime 0 gates `update_world` AND the hardUpdateWorld surrogate picks) | `face9df72` (c24c) |
| `GodmodePower` (`:692-728`) → `Player.creativeGodMode` (`Player.cs:25493-25497`, Hurt `:38203-38206`, KillMe `:38809-38812`, the pool refill `Main.cs:17869-17873`, the NPC contact gates) | `creative.rs::god_mode`, `world_sim/player.rs` tick copy (`power_god_mode || debug_god_mode`), `player.rs` Hurt/KillMe arms, `npc.rs` contact returns | pre-existing sim (`01d3cb93a`/`0f5371553` M6-CREATIVE-1c) + `f8721ac8a` toggle UI |
| `FarPlacementRangePower` (`:730-767`) → `Player.ResetEffects` tile range (`:18815-18828`) | `creative.rs::far_placement_range`, `player.rs::tile_range_x/y`, `craft.rs`/`app/clicks.rs` consumers | same M6 seam + `f8721ac8a` |
| `StopBiomeSpreadPower` (`:1540-1571`) → `WorldGen.AllowedToSpreadInfections` (`:71502-71507`) | `creative.rs::allowed_to_spread_infections`, `world_sim/growth.rs` feed | same M6 seam + `f8721ac8a` |
| `ModifyWindDirectionAndStrength` (`:1179-1309`) / `ModifyRainPower` (`:1311-1439`) / `FreezeWind` (`:1474-1505`) / `FreezeRain` (`:1507-1538`) | `creative.rs` (`wind_target_from_cache`/`wind_cache_from_target` unclamped read-back, the freeze toggles), `rain.rs::set_rain_power` (the StartRain/StopRain ladder `Main.cs:64644-64740`), `world_sim/weather.rs` (the tick's freeze copies `:140`/`:159`), `journey_clicks.rs` weather arms | `3c3e5a16c` |
| `SpawnRateSliderPerPlayerPower` (`:1573-1737`: remap `:1601-1608`, disable `:1584-1595`) → `NPC.CanSpawnEnemiesNear` (`:344-363`), `GetSpawnRate` (`:762-770`), SlimeRain denominator (`:5906-5913`) | `creative.rs::spawn_rate_multiplier`/`spawn_rate_disabled`, `spawner.rs::SpawnFlags.journey_spawn_rate` (the rate arm after the caps), `world_sim/npcs.rs` (the spawn gates + the slime-rain divide) | `f68acc89d` |
| `DifficultySliderPower` (`:1002-1177`: remap `:1018-1028` + banker's 1/20) → `Main.UpdateCreativeGameModeOverride` (`:17294-17305`) → `NPC.ScaleStats`/`_ByDifficulty`/`_Tweaks` (`:18146-18475`) | `tr-core wld/sections/creative_powers.rs::difficulty_strength_from_cache` (the shared remap), `creative.rs::game_mode_difficulty_override`, `npc.rs::scale_stats_by_difficulty` + the one-shot `difficulty_scaled` latch, `world_sim/npcs.rs` pre-pass, `app/frame/world.rs` step-head re-derive | `7b6e4c6eb` |
| `ItemsSacrificedUnlocksTracker.Save/Load` (`:157-196`) + `Player.Serialize/Deserialize` creativeTracker (`:54911`/`:55696-55699`) + `CreativePowerManager.SaveToPlayer/LoadToPlayer` (`:183-215`) | `research.rs::save_pairs`/`load_pairs` (ItemID-const-name keys via `itemdata::NAMES`, unknown-name drop, negative clamp), `player_file.rs` (`PlayerFile.research`/`powers`, `CreativePowersSave` — the non-Journey load reset `:211-214`), `app/saveglue.rs::save_player_plr` (preserves the stat halves), `app/flow.rs` CharPlay hydration | `cdcf5281e` |
| `Main.SelectPlayer` → `data.SetAsActive` (`Main.cs:66208-66216`) — the active character survives `LoadWorld` | `app/flow.rs` CharPlay read arm (research/powers/favorites/name/difficulty) + **`app/mod.rs::swap_world` carry** (this seal round — the App::new reset no longer wipes the hydrated per-character state) | `cdcf5281e` + this round |
| `Player.ToggleCreativeMenu` C key (`Player.cs:5091-5105`) + shift-sacrifice (`ItemSlot.cs:815-820`) + wheel scroll (`UIList.cs:107-114`) | `journey_clicks.rs::toggle_creative_menu_key`/`journey_shift_sacrifice`/`journey_wheel`, `app/clicks.rs` shift route (`:2149`) | `f8721ac8a` |
| `Research_Spark`/`Research_Button_Search` sheets + `Research_0..3` cues (`SoundID.cs:148-150`, `LegacySoundPlayer.cs:480-502`) | `assets.rs` lookups, `audio` bank rows | `ad88ba946` |
| Slot contexts 29/30/34 (`ItemSlot.cs:2253-2263`) | `item.rs`/`ui` slot-context arms | `ad88ba946` |

## 2. JOURNEY-SEAL receipt 1 — did c24c land?

**YES — `face9df72` (pushed to `origin/task/W4-c24-journey-ui`).** Verified
by `git log` + `git show face9df72 --stat` (13 files, +529/-25):

- **sacrifice-consumer**: the tooltip row (`ui/tooltip.rs` +
  `TooltipContext.slot_context/journey_difficulty/research`, the
  `hover_context` map, `Hover.context` 29/34, search keeps -1). The
  17-context whitelist re-read against `Main.cs:21318` verbatim
  (0,1,2,3,4,5,6,7,15,22,29,32,34,35,41,42,43) — the port's `matches!`
  set is exactly it. `Colors.JourneyMode` = `Color.Lerp(HotPink, White,
  0.1f)` (`ID/Colors.cs:113`) = the port's `CREATIVE_MODE_COLOR`
  `[255,120,187,255]` (`ui/uistate.rs:199`) — byte-equal.
- **TRASH_COMPACTOR (the f30b unlock-chain)**: `achievements.rs::
  check_research` + `research.rs::last_edit_id`/`count_fully_researched`
  + BOTH callers (`enter_loaded_world` forced `Player.cs:325`; the
  player tick's post-borrow drain `:25703`). Re-read against
  `AchievementsHelper.cs:387-399`: the version gate
  (`forced || _lastResearchVersion != LastEditId`), the latch write
  BEFORE the count (`:392`), `fully >= all / 2` integer divide (`:394`)
  — all faithful. The C# `_lastResearchVersion` is `private static int`
  (`:17`); the port's instance field on the one-manager-per-App is the
  documented lifetime-equivalent.
- **duplication**: the grid's search scan now carries the row-set gates
  (`journey_difficulty` + tracker) with `slot_context: -1` — the fresh
  `new Item()` entry (`Item.cs:117`) keeps the sacrifice row OUT of the
  search (the "more to unlock" never-match witness).
- **godmode/time-control**: godmode's sim consumers pre-date c24
  (M6-CREATIVE-1c `01d3cb93a`/`0f5371553`) and c24 wired the toggle UI
  (`f8721ac8a`); c24c closed the LAST time-control consumer — the growth
  loops (`GetWorldUpdateRate` `WorldGen.cs:72020-72028` re-read verbatim:
  `min(desired, 24)`, 0 frozen; `UpdateTimeRate` `Main.cs:6199-6226`
  re-read: dial desired-1, asleep x5, frozen 0, gameMenu 1). The
  rate-0 early return (`:71536-71540`) gates the port's hardUpdateWorld
  surrogate picks too. The default registry (TimeRate 1.0, FreezeTime
  off) keeps the DW-1.4 draw-count pins byte-identical (1 → 1).

## 3. JOURNEY-SEAL receipt 2 — f30b TRASH_COMPACTOR: inert → ACTIVE

f30's `a7750cfe0` wired the helper + both call sites but left the check
**inert**: its own `ResearchTracker` had no UI writer ("the live
sacrifice UI is unported, so the tracker never edits in play"). This
branch's chain activates it end-to-end:

1. **research-count producer**: `journey_research_clicks` confirm →
   `sacrifice_confirm` → `register_sacrifice` (cap-clamped, bumps
   `LastEditId`), and the shift-click whole-slot route
   (`journey_shift_sacrifice`, `ItemSlot.cs:815-820`).
2. **persistence**: `save_player_plr` → `save_pairs` → the `.plr`
   creativeTracker block; CharPlay → `load_pairs` (Load does NOT bump
   the version — the C# writes the dictionaries directly).
3. **unlock-chain**: the per-tick drain sees the bumped `LastEditId`,
   recounts (`count_fully_researched` over the 6139-row capped space),
   and at `fully >= all/2` fires progression 45 → `TRASH_COMPACTOR`
   (`achievements_data.rs:1204`) → the notification/chat/sound drain.
   The forced world-enter check grants it on a loaded past-half tracker
   without a new sacrifice.
4. **the swap carry (this round)**: `swap_world`'s `App::new` reset used
   to wipe the CharPlay-hydrated tracker/powers/difficulty — the
   `Main.ActivePlayerFileData` survival (`Main.cs:66208-66216`) now
   holds: the per-character halves ride the reset, the per-world powers
   re-load from the new world's section. Witness
   `w4_c24_the_character_journey_state_rides_the_world_swap` WRITTEN
   NOT RUN.

**Merger note**: f30's `check_research_achievement` free fn +
`App.last_research_version` field + its two call-site hunks are
SUPERSEDED by c24c's manager-owned shape (§8 keep-one). The f30 witness
`f30_the_enter_world_forced_check_fires_45` re-points onto
`app.achievements.check_research(&app.research, true)` at the merge.

## 4. JOURNEY-SEAL receipt 3 — e10c/f44g hydration

**LANDED on the f44 lane, NOT on this branch** — verified by reading the
pushed tips:

- `5c8bbb160` (W4-f44 SEAL 3, `origin/task/W4-f44-save-plr-2`): the
  hydration hunk CLOSED — CharPlay parks the card's `.plr` on
  `App.play_player`, `swap_world` carries the park, `App::new` gains the
  `play_player` param, `Player::hydrate_from_file` applies the stats /
  torch pair (slot-11 setter then the builder-array copy, `Player.cs:
  55242` then `:55635-55653`) / six used_upgrades + the five sibling
  fields / pearl LuckState mirror / super-cart BitsByte / voice pair
  (u8→i32, e10's live width) / counters; precedence character-FIRST,
  sidecar-SECOND-and-wins (132 §2c). 3 witnesses written not run.
- `0d1ee2045` (SEAL 4): 132 SEALED — the persist-family bible FINAL.
- e10's VOICE SEAL `01054aeb9` (`origin/task/W4-e10-player-sounds-wave`):
  the f44g mutual receipt — "e10 owes NO hydration code"; the u8
  transport keep-one (132 §3-L1 rule 7: DELETE e10's player_file.rs
  diff, KEEP the picker/draft/session with the as-u8 cast).

**This branch's CharPlay-hydration half** (`cdcf5281e` + this round) is
the Journey-slice of the same walk: research pairs + the three
per-player powers + favorites + name + difficulty hydrate at CharPlay
and now survive the swap. **Merge contract**: f44's `play_player` park +
`hydrate_from_file` is the CANONICAL character-load; this branch's
CharPlay read arm keeps ONLY its Journey-specific lines
(`research.load_pairs`, the three `creative_powers.set` rows,
`journey.spawn_rate_cache`, the draft difficulty) — the name/favorites
lines dedup onto f44's (`player_name` from the file, `favorite_slots`
from `inv_favorites`). The swap-carry hunk composes with f44's
`play_player` carry (both ride the same `App::new` reset; f44 re-reads
the file, this branch carries the live tracker — the tracker is NOT in
f44's `hydrate_from_file` scope, so both halves are needed).
The EXIT-SAVE (the full live-state `.plr` write) stays the one shared
remainder (132 §8 / 103 §2b condition (b)) — `save_player_plr` here
refreshes only research/powers over the preserved stat halves.

## 5. JOURNEY-SEAL receipt 4 — the journey powers all landed

Verified on this branch at `face9df72` + this round (each consumer
grepped live, not just the registry row):

| Power | UI writer | Sim consumer (verified) |
| --- | --- | --- |
| Weather wind slider | `journey_set_weather_slider`/the held drag (`journey_clicks.rs:217-245,762-775`) | `wind.speed_current/target` direct write (`CreativePowers.cs:1188-1191`), `world_sim/weather.rs` tick |
| Weather rain slider | same arms | `rain.set_rain_power` (the StartRain/StopRain ladder), `weather.rs:159` freeze copy |
| FreezeWind / FreezeRain | strip buttons 1/3 (`journey_weather_clicks`) | `weather.rs:140` (`wind.frozen`), `:159` (`set_freeze_rain`) |
| Spawn-rate x0-10 | personal strip + slider (`journey_personal_clicks`, the held drag) | `world_sim/npcs.rs:6529` (`spawn_rate_disabled` gate), `:6530/:6563` (the multiplier + slime-rain divide), `spawner.rs:485` (the rate arm after the caps) |
| Difficulty x0.5-3 | difficulty strip + icons + slider (`journey_difficulty_clicks`/`journey_set_difficulty`) | `app/frame/world.rs:99-101` (the override re-derive), `world_sim/npcs.rs:4284-4288` (the one-shot `scale_stats_by_difficulty` latch) |
| Search slider→box | the live search box (`4e31ff97f`) | `grid_items`→`search_fits` tooltip scan (the c24c row-set gates) |
| Tracker save | `save_player_plr` on every exit route (`saveglue.rs:100`) | the `.plr` creativeTracker + per-player powers blocks; CharPlay + swap-carry hydrate |
| FreezeTime / TimeRate / clock buttons | time strip (`journey_time_clicks`) | `frame/world.rs:372` (the clock rate), `world_sim/growth.rs` (the tile-update rate, c24c) |
| Godmode / FarPlacement / StopBiomeSpread | personal strip / main strip 5 | `world_sim/player.rs:376-377,577` (the tick copies), `player.rs` Hurt/KillMe/range, `growth.rs` allowed_to_spread |

## 6. The deliberate deviations (all recorded in ANALYSIS)

1. `ByCreativeSortingId` falls through to alphabetical (the generated
   group/order table is unported) — `f8721ac8a`/`e2e50a427`.
2. `load_pairs` drops unknown names + clamps negatives (the port's
   tracker IS the net-id cache; the C# keeps the string row) —
   `cdcf5281e`.
3. `CreativeResearchItemPersistentIdOverride` alias remap unported (the
   names map 1:1) — `research.rs` doc + c24c.
4. The teammate-credit halves (`_unlockedByTeammate`,
   `AnyNewUnlocksFromTeammates`, the `:21327-21331` tooltip arm, the
   toggle's notification icon `CreativeUI.cs:172-175`) — single-player
   inert, unported.
5. `ScaleStats` documented skips: expert-hardmode/money/getGood/defense
   arms, the single-player `ByPlayerCount` identity — `7b6e4c6eb`.
6. The search rides the `event.text` seam, not the c25 keyboard
   bindings — `4e31ff97f`.
7. `_lastResearchVersion` static → per-App instance field (one manager
   per App, the same lifetime) — `face9df72`.
8. The difficulty stand-in is the session draft
   (`active_player_difficulty`), not a live `Player.difficulty` — the
   f44 hydration lane owns the live field.
9. `save_player_plr` preserves the existing file's stat halves (the
   full live-state write is the exit-save remainder, §4).

## 7. Witness ledger (ALL WRITTEN NOT RUN — code-only)

- `f8721ac8a`: 24 UI witnesses (research display, grid, power toggles,
  C key, shift-sacrifice, wheel).
- `ad88ba946`/`0217b62fd`: tracker cap-clamp/newly + sheet/cue pins.
- `3c3e5a16c`/`f68acc89d`/`7b6e4c6eb`: weather/spawn/difficulty wiring
  witnesses (the remap endpoints, the freeze gates, the one-shot latch).
- `4e31ff97f`: search-box + slider-panel witnesses.
- `cdcf5281e`: save/load round-trip through the names, the unknown/neg
  arms, the powers Default-vs-written_fresh split.
- `face9df72` (c24c): 5 — the sacrifice-row gates, the edit-version
  bumps, the fully-count walk, the half-catalog 45 fire, the growth
  freeze/rate stream pins.
- This round: 1 — `w4_c24_the_character_journey_state_rides_the_world_swap`
  (the carry + the per-world re-load + the forced-check non-fire).

## 8. The merger keep-one table (the journey family)

| Surface | KEEP | DROP / re-point |
| --- | --- | --- |
| `ResearchTracker` | this branch's `research.rs` WHOLE (cap-clamp + newly + save/load + LastEditId + count — the f30 base's unclamped accumulator is a strict subset) | f30's `a7750cfe0` research.rs hunks (the free-fn `check_research_achievement` + its doc rewrite) |
| The research-45 check | `achievements.rs::check_research` (manager-owned latch, c24c) | f30's `App.last_research_version` field + its two call-site hunks; the f30 witnesses re-point onto the manager fn (`f30_the_enter_world_forced_check_fires_45` → `enter_loaded_world` + `achievements.check_research`) |
| The growth rate | c24c's `update_growth` powers-derived arm | any lane's hardcoded `(1, false)` feed |
| Character-load | f44's `play_player` park + `Player::hydrate_from_file` (the canonical walk) | this branch's CharPlay name/favorites lines (dedup onto f44); KEEP the Journey lines (research/powers/spawn-cache/difficulty) + the swap-carry hunk (composes — the tracker is outside f44's hydrate scope) |
| The voice pair | e10's live i32 fields + consumers; f44's hydrate cast-seam | e10's player_file.rs u8-transport diff (132 §3-L1 rule 7) |
| The `.plr` codec | f44's reader/writer superset (the full tail) | this branch's `player_file.rs` research/powers rows dedup textually onto f44's (same field names/shapes — `research: Vec<(String,i32)>`, `CreativePowersSave`) |
| The tooltip row | c24c's `TooltipContext` three fields + the row | f28's cited-drop note (the row is no longer unported); e16's research rows ride the same seam (`39babafa7` co-lane contract) |

## 9. Anchors

The family's canary rows (anchors.txt): `save_pairs`/
`CreativePowersSave` (c24), `set_rain_power`/`journey_spawn_rate`/
`scale_stats_by_difficulty`/`difficulty_strength_from_cache`/
`search_fits` (the wiring pass), `slot_context`/`check_research`/
`w4_c24c_freeze_time_stops_the_growth_loops` (c24c), and this round's
`let carried_research = std::mem::take` +
`w4_c24_the_character_journey_state_rides_the_world_swap` (the swap
carry). All markers verified present at write time.