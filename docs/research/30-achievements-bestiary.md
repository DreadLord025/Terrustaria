# 30 — Achievements / bestiary

> Wave 3 · researched 2026-11-12 · sources: decompiled + crates + ROADMAP/ANALYSIS/PORT_MAP

## Scope

Two independent C# subsystems. **Achievements**: the framework
(`Terraria/Achievements/*`, 10 files, ~700 lines), the condition classes
(`Terraria/GameContent/Achievements/*`, 9 files, ~565 lines), the event surface
(`AchievementsHelper.cs`, 505 lines) and the 137-row catalogue
(`Terraria/Initializers/AchievementInitializer.cs`, 736 lines). **Bestiary**: the
database + entry model (`Terraria/GameContent/Bestiary/*`, 56 files), the NPC
catalogue (`BestiaryDatabaseNPCsPopulator.cs`, 2077 lines), the three unlock
trackers (`NPCKillsTracker.cs` 108, `NPCWasNearPlayerTracker.cs` 142,
`NPCWasChatWithTracker.cs` 100) and the drop-rate report
(`BestiaryDatabase.Merge` → `IItemDropRule.ReportDroprates`). The subsystem owns
two player-visible progress meters (the achievements menu, the bestiary screen)
and one persistent store per world (the `.wld` section 10 triple).

## Key entities

**C# — achievements**
- `Terraria/Achievements/Achievement.cs::Achievement` (11-176) — the row: a
  `Dictionary<string, AchievementCondition>` keyed by condition name, a
  `_completedCount`, `IsCompleted => _completedCount == _conditions.Count` (38).
- `Terraria/Achievements/AchievementManager.cs::AchievementManager` (17-242) —
  the registry (`Register` 189, `RegisterIconIndex` 195, `RegisterAchievementCategory`
  200), the Rijndael-CBC+BSON save (`Save` 65-88, key `"RELOGIC-TERRARIA"` 53) and
  the `OnAchievementCompleted` event (180-187) which **calls `Save()` on every unlock**.
- `Terraria/Achievements/AchievementCondition.cs::AchievementCondition` (7-61) —
  `Name`, `IsCompleted`, idempotent `Complete()` (37-47), `Load`/`Clear`.
- `Terraria/Achievements/AchievementTracker.cs::AchievementTracker<T>` (5-73) —
  `_value`/`_maxValue`; `SetValue` fires `OnComplete` when the value equals the max (39-56).
- `Terraria/Achievements/ConditionsCompletedTracker.cs` (6-31) — the "N of N"
  tracker; `AddCondition` raises `_maxValue` (12).
- `Terraria/Achievements/AchievementCategory.cs` (3-9) — `None = -1`, Slayer,
  Collector, Explorer, Challenger.
- `Terraria/GameContent/Achievements/AchievementsHelper.cs::AchievementsHelper`
  (3-505) — the five static events (39-47) and the `Notify*` entry points
  (`NotifyItemPickup` 57, `NotifyItemCraft` 73, `NotifyTileDestroyed` 49,
  `NotifyNPCKilled` 200, `NotifyProgressionEvent` 227) plus
  `TryGrantingBestiary100PercentAchievement` (81-87).
- `Terraria/GameContent/Achievements/NPCKilledCondition.cs` (6-82) — the static
  `_listeners` dictionary (10) and the two factories that mean **different
  things**: `Create(params short[])` (59) = **one** condition on many ids;
  `CreateMany(params short[])` (69) = **N separate** conditions, one per id.
- `Terraria/GameContent/Achievements/CustomIntCondition.cs` (7-76) /
  `CustomFloatCondition.cs` (7-76) — counted conditions; the `Value` setter
  clamps and calls `Complete()` at the max (21-32).
- `Terraria/Initializers/AchievementInitializer.cs::Load` (12-736) — the 137
  `new Achievement(...)` blocks (13-441), the `RegisterIconIndex` pass (445-582)
  and the `RegisterAchievementCategory` pass (584-736).

**C# — bestiary**
- `Terraria/GameContent/Bestiary/BestiaryDatabase.cs::BestiaryDatabase` (7-96) —
  `_entries` + `_byNpcId` (11-19), `Register` (27-38), `FindEntryByNPCID` (52-60,
  falls back to a shared trash entry), `Merge(ItemDropDatabase)` (62-68) →
  `ExtractDropsForNPC` (70-88) → `item.ReportDroprates(...)` (82).
- `Terraria/GameContent/Bestiary/BestiaryEntry.cs::BestiaryEntry` (8-117) —
  the four factories `Enemy` (21), `TownNPC` (59), `Critter` (82), `Biome` (105).
- `Terraria/GameContent/Bestiary/BestiaryDatabaseNPCsPopulator.cs` (1-2077) —
  `AddEmptyEntries_CrittersAndEnemies_Automated` (851-868) registers one entry
  per `ContentSamples.NpcsByNetId` (minus the `NPCBestiaryDrawOffset.Hide`
  exclusions, 870-886); 40 `TownNPC` + 2 `Enemy` + 1 `Critter` + 1 `Biome`
  (Hallow, unlocked by `Conditions.ReachHardMode`, 559) are registered by hand;
  the `UIInfoProvider` overrides set the boss `quickUnlock: true` rows
  (679-701) and the `HighestOfMultiple` / `GoldCritter` / `Salamander` providers.
- `Terraria/GameContent/Bestiary/NPCKillsTracker.cs::NPCKillsTracker` (10-107) —
  the kill counts keyed by bestiary credit id, `POSITIVE_KILL_COUNT_CAP` 999999999
  (14), `RegisterKill` (23-33), `SetKillCountDirectly` (40-51, calls
  `TryGrantingBestiary100PercentAchievement` on the first sighting of an id),
  `Save`/`Load` (59-81).
- `Terraria/GameContent/Bestiary/BestiaryUnlocksTracker.cs` (5-50) — the three
  sub-trackers `Kills`/`Sights`/`Chats` (7-11), saved in `.wld` section 10.
- `Terraria/GameContent/Bestiary/CommonEnemyUICollectionInfoProvider.cs` (6-81) —
  `GetKillCountNeeded` (21-34, the banner chain) and
  `GetUnlockStateByKillCount` (50-76, the 1/5 · 1/2 · full ladder).
- `Terraria/GameContent/Bestiary/TownNPCUICollectionInfoProvider.cs` (14-20) —
  unlock = **chatted** with. `CritterUICollectionInfoProvider.cs` (14-20) —
  unlock = **sighted**.
- `Terraria/Main.cs::GetBestiaryProgressReport` (65524-65540) — the completion
  fraction over the whole roster; `Terraria/GameContent/Bestiary/Filters.cs`
  (184), `SortingSteps.cs` (233), `SpawnConditionBestiaryInfoElement.cs` (35).

**Rust**
- `crates/tr-app/src/achievements.rs` (569) — `Category` (31), `CondKind` (96),
  `Condition` (130), `Achievement` (165), `AchievementManager` (264),
  `notify` (284), `save` (395), `load` (409).
- `crates/tr-app/src/achievements_data.rs` (1170) — `Cond` (33), `Row` (64),
  `ROWS` (73-1170, the 137 rows).
- `crates/tr-app/src/ui/achievements.rs` (913) — the menu.
- `crates/tr-core/src/bestiary.rs` (499) — `UnlockState` (11),
  `get_unlock_state_by_kill_count` (31-54), `KillTracker` (65),
  `WorldBestiary` (138), `npc_to_banner` (225), `kills_needed_for` (268),
  `banner_to_item` (288).
- `crates/tr-app/src/bestiary.rs` (1602) — `BestiaryEntryDef` (13),
  `ENTRIES` (31-1422, 172 rows), `unlock_state` (1427), `completion_percent`
  (1440), `entry_for_npc` (1482), `kill_progress` (1529).
- `crates/tr-app/src/ui/bestiary.rs` (1520) — the screen; the drop lines read
  `crate::loot_db::drop_lines` (1203).
- `crates/tr-core/src/wld/sections/bestiary.rs` (535) — the `.wld` section 10 codec.

## Dependencies

- Calls into: `Main` (the `Main.Achievements` singleton, `Main.BestiaryDB`,
  `Main.BestiaryTracker`), `Player` (`HandleOnEquip` 239, `OnEnterWorld`),
  `NPC` (`GetBestiaryCreditId` `NPC.cs:95139`), `ItemDropDatabase`
  (`GetRulesForNPCID`), `Language` (the `Achievements.<NAME>_Name` keys),
  `SocialAPI.Achievements` (Steam), `WorldFile.SaveBestiary` (`.wld` section 10),
  `ContentSamples.NpcsByNetId` / `NpcBestiaryCreditIdsByNpcNetIds`,
  `ItemID.Sets.KillsToBanner` / `DefaultKillsForBannerNeeded`,
  `BannerSystem.NPCtoBanner` (457 cases) / `BannerToItem` (341-401),
  `NPCID.Sets.ShouldBeCountedAsBossForBestiary` (`NPCID.cs:10481`).
- Called by: `Player` (equip/pickup/mine hooks), `NPC` (the death → kill-count
  hook), `Main.UpdateTime_StartDay/StartNight`, `WorldGen` (hardmode/real-estate
  checks), `Chest` (the Zoologist's shop gate, `Chest.cs:3265`), the UI
  (`UIAchievementsMenu`, `UIBestiaryTest`), `AchievementAdvisor`, the chat tag
  handler.
- Shared data structures: the achievement **condition dictionary** (keyed by
  name — the save format's key), the bestiary **credit id** (`NPCID.Search.GetName`
  of the net id — the kill counter's key), the **banner → item → kill count**
  chain (the unlock ladder's denominator), the five-tier `UnlockState`.

## Port status

| C# subsystem | Status | Rust location | Evidence / notes |
| --- | --- | --- | --- |
| `Achievement` row model + `IsCompleted` | PARTIAL | `achievements.rs::Achievement` (165) | `is_completed()` = `all(conditions.completed)` (195) equals the C# count check, but the Rust `completed` flag is initialised `false` (188) while `is_completed()` on a zero-condition row is `true` — the two disagree after a save/load (`load_pairs` 350). |
| `AchievementCondition` + `Complete()` | PORTED | `achievements.rs::Condition` (130), `CondKind` (96) | One-shot completion via the `completed` flag (297); the 10 condition kinds are modelled. |
| `AchievementManager` registry + icon/category passes | PORTED | `achievements.rs::AchievementManager` (264), `achievements_data.rs::ROWS` | 137 rows, icons 0..136, exactly one `Category::None` row (`GAIN_TORCH_GODS_FAVOR`) — pinned by `the_registry_is_the_initializers_full_table`. |
| Condition catalogue (the 180 conditions) | **PARTIAL** | `achievements_data.rs::Cond` (33) | 139 entries vs the C#'s 180. `CreateMany` is unrepresentable: the 4 rows using it lose 41 conditions — `STAR_DESTROYER` 4→0, `GELATIN_WORLD_TOUR` 24→1, `SLAYER_OF_WORLDS` 15→3, `BUCKETS_OF_BOLTS` 3→1. |
| Condition evaluation order | PORTED | `achievements.rs::notify` (284-307) | Registry order, then condition order, matching the C#'s listener-list insertion order. The Rust returns only the **last** completed achievement (303) where the C# fires `OnCompleted` for every one (`Achievement.cs:93-108`). |
| `AchievementTracker` / `ConditionsCompletedTracker` | PARTED | `achievements.rs::tracker_values` (201-214) | The two tracker shapes exist; the denominators inherit the `CreateMany` loss (e.g. `SLAYER_OF_WORLDS` shows `(1,3)`, vanilla shows `(1,15)`). |
| `CustomInt` / `CustomFloat` counters | PARTIAL | `Cond::Int` / `Cond::Float` (56-59) | The rows exist (6 + 2) but only `CustomInt("Pick", 10000)` has a notify site (`world_sim/editing.rs:1791`). |
| Notify sites: NPC kill / pickup / craft / tile | PORTED | `world_sim/npcs.rs:2531`, `items.rs:200`, `app/clicks.rs:409`, `editing.rs:1788` | All four wired. The C# also gates the kill/pickup/tile listeners on `player.whoAmI == Main.myPlayer` (`NPCKilledCondition.cs:49`); the port has one player, so it is equivalent. |
| Notify sites: progression event / custom flag | **MISSING** | — | 43 `ProgressionEventCondition` + 37 `CustomFlagCondition` rows have **no** notify site anywhere in `crates/` — `NO_HOBO`, `BIG_BOOTY`, `WHERES_MY_HONEY`, `HOLD_ON_TIGHT`, … can never complete. |
| `AchievementsHelper.TryGrantingBestiary100PercentAchievement` | MISSING | — | `NotifyProgressionEvent(29)` → the `BOOK_WORM` row. Never called; `ProgressionEvent` has no notify site either. |
| `AchievementManager.Save`/`Load` (Rijndael+BSON) | PARTIAL (documented) | `achievements.rs::save` (395), `load` (409) | The port writes `achievements.trdat`, a plain text format (`achievements.rs:390-394`); the vanilla crypto format is not read. **But `save()` has zero call sites** — only `load` is wired (`app/mod.rs:1807`), so nothing is ever persisted. |
| Achievement unlock popup / advisor / chat tag | MISSING | — | `AchievementUnlockedPopup` (`UI/InGamePopups.cs:13`), `AchievementAdvisor` (`UI/AchievementAdvisor.cs`), `AchievementTagHandler` (`UI/Chat/`) have no port. |
| `SocialAPI.Achievements` (Steam stats) | OUT OF SCOPE | — | `PORT_MAP.md:268` marks the platform hooks as out of scope. |
| `BestiaryEntry` + the 4 factories | PORTED (simplified) | `tr-app/src/bestiary.rs::BestiaryEntryDef` (13) | One flat struct (credit id, net id, kind, variant, quick_unlock, name) instead of the C#'s `Info` list of `IBestiaryInfoElement`s (`BestiaryEntry.cs:24-30`). |
| NPC → entry mapping | PARTIAL | `ENTRIES` (31-1422), `entry_for_npc` (1482) | 172 entries vs the C#'s "one per `ContentSamples.NpcsByNetId`" (the C# roster is the whole NPC id space). The sheet table `BESTIARY_NPC_SHEET_IDS` has 174 slots (`assets.rs:1319`) — 2 more than the entries. |
| `NPCKillsTracker` (counts + cap + save) | PORTED | `tr-core/src/bestiary.rs::KillTracker` (65) | The 999999999 cap (57, 84), `register_kill` (76), `set_kill_count_directly` (83), `pairs`/`load_pairs` (101/106). |
| Kill-counter persistence (`.wld` section 10) | PORTED | `tr-core/src/wld/sections/bestiary.rs` (535) | All three sub-lists round-trip both ways; 17 tests. The sights/chats half stays empty (documented in `tr-core/src/bestiary.rs:124-130`). |
| Unlock ladder (1 / ⅕ / ½ / full + boss quick-unlock) | **PARTIAL** | `get_unlock_state_by_kill_count` (31-54) | The ladder itself is a faithful port of `CommonEnemyUICollectionInfoProvider.cs:50-76`, but its denominator `kills_needed_for` (268) returns **0** for every net id outside the 11-pair banner table, and 0 means "≥ full" → 164 of 172 entries read as tier 4 with zero kills. The C#'s default is 50 (`ItemID.Sets.DefaultKillsForBannerNeeded`). |
| `quickUnlock` per entry | PARTIAL | `BestiaryEntryDef.quick_unlock` | Only 5 entries carry it (KingSlime, EyeofCthulhu, Guide, Golem, Deerclops); the C# sets it on **every** boss (`BestiaryEntry.cs:37-41`) plus the 494-506 salamander and the `HighestOfMultiple` overrides. `Guide` is a **town** NPC and must not carry it. |
| Town-NPC / critter unlock providers | MISSING | — | `TownNPCUICollectionInfoProvider` (chat) and `CritterUICollectionInfoProvider` (sight) have no port; the `sights`/`chats` sets are never recorded. |
| `BestiaryDatabase.Merge` → `ReportDroprates` | **MISSING** | — | No `report_droprates` anywhere in `crates/`. The bestiary's "Drops:" list reads `loot_db::drop_lines` (`ui/bestiary.rs:1203`), a hand-written 4-row table, so 168 of 172 entries print "(nothing recorded yet)". |
| `GetBestiaryProgressReport` | PARTIAL | `completion_percent` (1440) | Same predicate (`unlock_state(...).known()`), but the denominator is the port's 172-entry roster, and the tier-4 bug inflates the value to ~95% at world start. The Zoologist gate (`Chest.cs:3265`) reads it. |
| Bestiary filters / sorting / flavour / spawn info | MISSING | — | `Filters.cs` (184), `SortingSteps.cs` (233), `FlavorTextBestiaryInfoElement`, `RareSpawnBestiaryInfoElement`, `SpawnConditionBestiaryInfoElement`, `HighestOfMultipleUICollectionInfoProvider`, `GoldCritterUICollectionInfoProvider`, `SalamanderShellyDadUICollectionInfoProvider` — none ported. |
| `NetBestiaryModule` (kill-count sync) | MISSING | — | No MP sync of the kill counts (`NetBestiaryModule.cs`). |

## Coverage

| Achievement / bestiary group | C# count | Rust | Verdict |
| --- | --- | --- | --- |
| Achievement rows | 137 | 137 | **PASS** (order, ids, icons 0..136, one `Category::None`) |
| Achievement conditions | 180 | 139 | **PARTIAL** — 41 lost, all in the 4 `CreateMany` rows |
| Condition kinds modelled | 10 | 10 | **PASS** (`CondKind`) |
| Condition kinds with a live notify site | 10 | 4 | **FAIL** — progression event, custom flag, custom int (5/6), custom float (2/2) are dormant |
| Rows that can ever complete | 137 | 50 | **FAIL** — 87 rows carry a condition kind the port never fires |
| Tracker shapes (`UseTracker*`) | 2 | 2 | **PASS** (shape), **FAIL** (the `ConditionsCompleted` denominators) |
| Achievement save/load | 1 | 1 (text) | **PARTIAL** — format differs (documented); `save()` is never called |
| Bestiary entry rows | whole NPC roster (1 per `NpcsByNetId`) + 44 manual | 172 | **PARTIAL** — port roster only; 2 sheet slots unmatched |
| Bestiary unlock tiers | 5 | 5 | **PASS** (the ladder's shape) |
| Entry unlock providers | 6 (`Common`/`Town`/`Critter`/`Gold`/`HighestOfMultiple`/`Salamander`) | 1 (`quick_unlock` bool) | **FAIL** — 5 providers missing |
| Kill counters (counts, cap, save) | 1 tracker | 1 tracker | **PASS** |
| Sight / chat trackers | 2 | 2 (never recorded) | **PARTIAL** (documented) |
| Bestiary drop tables + rates | `ReportDroprates` over every NPC | 4 hard-coded rows | **FAIL** |
| Bestiary progress report | 1 | 1 | **PARTIAL** (denominator + the tier bug) |
| Bestiary UI features (filters/sort/flavour/spawn info) | 4+ classes | 0 | **FAIL** |
| MP kill-count sync | 1 module | 0 | **FAIL** (MP) |

## Tests

- `crates/tr-app/src/achievements.rs::tests` (7): `the_registry_is_the_initializers_full_table`
  (438), `every_row_carries_a_distinct_condition_key` (464),
  `the_trackers_report_the_ports_progress` (478),
  `the_multi_id_condition_completes_on_any_listed_id` (493),
  `the_sorted_list_puts_completed_rows_first` (508),
  `the_ported_hooks_still_complete_their_rows` (520),
  `the_save_round_trips_the_condition_state` (535).
  Two of them **pin the divergence**: `the_trackers_report_the_ports_progress`
  asserts `SLAYER_OF_WORLDS == (1,3)` (vanilla: `(1,15)`) and
  `the_ported_hooks_still_complete_their_rows` asserts `GELATIN_WORLD_TOUR`
  completes on one gel kill (vanilla: 24 conditions).
- `crates/tr-app/src/ui/achievements.rs::tests` (9): the panel/list/back/filter/
  sort/row-hit/scroll/icon-frame layout tests (784-898).
- `crates/tr-core/src/bestiary.rs::tests` (7): `the_tier_ladder_is_the_half_and_fifth_steps`
  (335), `the_quick_unlock_skips_the_ladder` (378),
  `the_kill_tracker_caps_and_persists_pairs` (392),
  `the_world_bestiary_holds_the_three_credit_id_trackers` (415),
  `the_world_bestiary_resets_all_three_trackers` (437),
  `the_world_bestiary_iterates_deterministically` (451),
  `the_family_banner_chain` (483).
- `crates/tr-app/src/bestiary.rs::tests` (2): `every_ported_kind_maps_to_its_entry`
  (1541), `the_tiers_climb_with_kills` (1574).
- `crates/tr-app/src/ui/bestiary.rs::tests` (7): the panel/grid/sheet-slot/
  Solar-Selenian/entry-survival/button tests (1301-1475).
- `crates/tr-core/src/wld/sections/bestiary.rs::tests` (17): the `.wld` section-10
  codec, the version gates at 210/220, the hostile-input and round-trip tests
  (210-512).
- No test covers: the per-row condition count against `AchievementInitializer.cs`,
  the `notify` sites' reachability, the achievement save being called, or the
  bestiary's drop lines.

## Deviations & risks

| # | Deviation | Severity | Admitted? |
| --- | --- | --- | --- |
| 1 | `CreateMany` collapsed: `STAR_DESTROYER` 4→0 conditions (so it can never unlock and its `is_completed()` is vacuously true), `GELATIN_WORLD_TOUR` 24→1, `SLAYER_OF_WORLDS` 15→3, `BUCKETS_OF_BOLTS` 3→1. The port's own comment at `achievements.rs:162-163` calls the zero-condition `STAR_DESTROYER` "complete from the start, exactly as vanilla" — vanilla needs all four pillars. | behavioural | **No** |
| 2 | `achievements::save` has **no call site**; only `load` is wired (`app/mod.rs:1807`). ROADMAP M6-ACH-1 claims "the save rides the F9/autosave pass" — false in the current tree. | behavioural | **No** (contradicts a DONE row) |
| 3 | 87 of the 137 rows contain a condition kind with no notify site (43 progression, 37 custom flag, 5 of 6 custom int, 2 of 2 custom float). | behavioural | Partly — M6-ACH-1 says "the ported registry subset" |
| 4 | `kills_needed_for` returns 0 for net ids outside the 11-pair banner table; 0 means "≥ full", so 164 of 172 entries report tier 4 (`CanShowDropsWithDropRates`) with zero kills. The C# default is 50. | behavioural | **No** |
| 5 | `quick_unlock` on 5 entries instead of every boss + the override rows; the town NPC `Guide` wrongly carries it (the C# uses the chat provider). | behavioural | **No** |
| 6 | The town-NPC (chat) and critter (sight) unlock providers are absent, and the `sights`/`chats` sets are never recorded. | behavioural | Yes — `tr-core/src/bestiary.rs:124-130`, ANALYSIS "the sightings/chats stay empty until the recording behaviour lands" |
| 7 | `ReportDroprates` / the drop-rate tables are absent: the bestiary's drop list is a 4-row hand-written table, so the tier-3/tier-4 drop lines are empty for 168 of 172 entries. | behavioural / pixel | Yes — ANALYSIS.md:2969 "not ported — M4-BEST-1 territory" |
| 8 | `TryGrantingBestiary100PercentAchievement` (→ progression event 29 → `BOOK_WORM`) never fires. | behavioural | Yes — `tr-core/src/bestiary.rs:82` "no achievement hook (M6-ACH-1's)" |
| 9 | The achievement save format is plain text (`achievements.trdat`), not the vanilla Rijndael-CBC+BSON `"RELOGIC-TERRARIA"`. | behavioural (interop) | Yes — `achievements.rs:390-394` |
| 10 | The unlock popup, `AchievementAdvisor` and the chat tag handler are missing. | cosmetic / behavioural | **No** |
| 11 | `notify` returns only the last completed achievement; the C# fires `OnCompleted` (and a save) for every one. | behavioural (minor) | **No** |
| 12 | `completion_percent` divides by the port's 172-entry roster, not the C#'s full roster; combined with #4 it reads ~95% at start, which gates the Zoologist's shop (`Chest.cs:3265`). | behavioural | **No** |
| 13 | `ENTRIES` (172) vs `BESTIARY_NPC_SHEET_IDS` (174) — 2 sheet slots have no entry row. | cosmetic | Partly — BUG-15 removed the literal drift, not the skew |
| 14 | The kill-count and sight/chat sets iterate in sorted order vs the C#'s unspecified `Dictionary`/`HashSet` order — deterministic, but a different byte order in the save. | determinism | Yes — documented in `tr-core/src/bestiary.rs:96-100, 132-136` |
| 15 | `banner_to_item` keeps only 3 bands (276+/88+/default); the C# has the 292/291/290/289/273/251 specials and the 186-275 bands. It agrees on the port's 11 banners but is wrong on extension. | behavioural (latent) | Partly — `tr-core/src/bestiary.rs:286-287` |
| 16 | No MP sync of the bestiary kill counts (`NetBestiaryModule`). | behavioural (MP) | **No** |
| 17 | `Player.HasAchievement`, `Main.achievementTracker` and a `BossAchievement` type **do not exist** in the 1.4.5.8 decompile. The boss kill requirement is the bestiary's banner→`KillsToBanner` chain, and the achievement query is `Main.Achievements.GetAchievement(name).IsCompleted`. | n/a | n/a |
| 18 | `Achievement.completed` (flag) vs `is_completed()` (derived) disagree for a zero-condition row before and after a load. | behavioural (minor) | **No** |

## Task seeds

- **S** — Model `NPCKilledCondition.CreateMany` faithfully: add a "one condition
  per id" variant to `Cond` and re-transcribe the 4 rows (46 conditions, not 5).
  Fixes #1. Dep: none.
- **S** — Call `achievements::save` from the autosave/F9 pass and add a
  write→read round-trip test through the app path. Fixes #2. Dep: none.
- **M** — Add the missing notify sites: `NotifyProgressionEvent` (43 rows),
  the custom-flag `Complete()` calls (37 rows) and the `CustomInt`/`CustomFloat`
  increments. Fixes #3. Dep: the S task above (so the rows exist).
- **S** — Make `kills_needed_for` fall back to
  `ItemID_Sets_DefaultKillsForBannerNeeded` (50) instead of 0, and pin it with a
  regression test that a fresh tracker leaves a non-quick entry at
  `NotKnownAtAll`. Fixes #4.
- **M** — Replace the `quick_unlock` bool with a per-entry provider enum
  (`CommonEnemy` / `TownNPC` / `Critter` / `HighestOfMultiple` / `GoldCritter` /
  `Salamander`) and record sightings and chats. Fixes #5, #6. Dep: the S task
  above for the denominator.
- **L** — Port `ReportDroprates`: add a `report_droprates` method to the drop-rule
  traits (`tr-core/src/loot.rs`) and feed the resulting `DropRateInfo` rows into
  the bestiary entries as the C#'s `ItemDropBestiaryInfoElement`. Fixes #7.
- **S** — Fire `TryGrantingBestiary100PercentAchievement` from
  `KillTracker::set_kill_count_directly` once the progression-event notify site
  exists. Fixes #8. Dep: the M notify task.
- **M** — Bestiary UI parity: the filter list (`Filters.cs`), the sort steps
  (`SortingSteps.cs`), the flavour-text and rarity-star info elements and the
  spawn-condition element. Fixes #15 (UI half).
- **S** — Add the achievement unlock toast and fire `OnCompleted` for every
  completed achievement (not only the last). Fixes #10, #11.
- **S** — Reconcile `ENTRIES` (172) with `BESTIARY_NPC_SHEET_IDS` (174) and add
  a test that every sheet slot has an entry and vice versa. Fixes #13.
- **M** — Port `NetBestiaryModule`: broadcast `SerializeKillCount` on a kill and
  sync the counts on player join. Fixes #16. Dep: the netcode module's shape.
- **S** — Add a data test that pins every Rust row's condition count against
  `AchievementInitializer.cs` (137 rows / 180 conditions) so the `CreateMany`
  class of bug cannot return. Dep: the first S task.
