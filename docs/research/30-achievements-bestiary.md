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
| Condition catalogue (the 180 conditions) | PORTED | `achievements_data.rs::Cond` (33) | 180 entries (S1-E-18): the four `CreateMany` rows are expanded 1:1 (`STAR_DESTROYER` 4, `BUCKETS_OF_BOLTS` 3, `GELATIN_WORLD_TOUR` 24, `SLAYER_OF_WORLDS` 15), preserving folded groups where the C# does. |
| Condition evaluation order | PORTED | `achievements.rs::notify` / `notify_with` | Registry order, then condition order, matching the C#'s listener-list insertion order. The Rust returns only the **last** completed achievement where the C# fires `OnCompleted` for every one (`Achievement.cs:93-108`). |
| `AchievementTracker` / `ConditionsCompletedTracker` | PORTED (shape) | `achievements.rs::tracker_values` | Two tracker shapes; denominators now reflect the expanded 1:1 condition rows (S1-E-18). |
| `CustomInt` / `CustomFloat` counters | PARTIAL | `achievements.rs::{notify_custom_int,notify_custom_float,notify_with}` | Wrappers + `Value`-setter semantics landed (S1-E-18). Wired: `Pick` (edit), `Finish` flag/int on angler turn-in (`npcs.rs`), `Do` on acorn plant (`editing.rs`), `Move` on grounded running (`world_sim/player.rs`). `Peek` custom flag wired (`chest.rs`). `Pay` and other custom counters remain without owning systems. |
| Notify sites: NPC kill / pickup / craft / tile | PORTED | `world_sim/npcs.rs`, `items.rs`, `app/clicks.rs`, `editing.rs` | All four wired (single-player). |
| Notify sites: progression event / custom flag | PARTIAL | `achievements.rs`, `world_sim/{clock,editing,npcs,weather}.rs`, `chest.rs` | S1-E-18 wires progression 1/3/5/6/8/9/16 and selected custom sites (`Finish`, `Peek`, `Do`, `Move`). Remaining progression/custom rows are blocked on missing systems (e.g. 17 partial, 18/19/20, nurse/service/chat/locked-chest paths) or S1-F-15 (29). |
| `AchievementsHelper.TryGrantingBestiary100PercentAchievement` | PORTED | `achievements.rs::try_grant_bestiary_100` (378), `app/netglue.rs::try_grant_bestiary_100_net` | `NotifyProgressionEvent(29)` → `BOOK_WORM` is wired: the first-key hook fires from `set_kill_count_directly`, and S1-I-5 routes it over `98` on a host. The blocker (S1-F-15's `completion_percent` tier bug) is separate and still open — the hook fires but 100 % is reached too early. |
| `AchievementManager.Save`/`Load` (Rijndael+BSON) | PARTIAL (documented) | `achievements.rs::{save,load}`, `app/{mod,saveglue}.rs` | Port still writes `achievements.trdat` (plain text), not the vanilla Rijndael-CBC+BSON blob. S1-E-18 binds a save path at bootstrap/world swap and saves on completion and from `save_world`; headless test apps have no bound path. |
| Achievement unlock popup / advisor / chat tag | PARTIAL (popup DONE) | `ui/notifications.rs`, `app/mod.rs::drain_achievement_completions` | **S1-E-26 wave A**: `AchievementUnlockedPopup` (`UI/InGamePopups.cs:13-148`) + `InGameNotificationsTracker` (`:11-113`) ported — the 300-tick timer/curves/draw/click, the `netMode != 2` gate, the `AchievementInitializer.cs:728-735` chat line + tracked `AchievementComplete` cue, `OpenAchievementsAndGoto`'s scroll, `SaveAndQuit`'s `Clear`. `AchievementAdvisor` (`UI/AchievementAdvisor.cs`) and the chat tag's snippet rendering (`AchievementTagHandler`, `UI/Chat/`) still have no port (the line prints the raw `[a:NAME]` tag). |
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
| `NetBestiaryModule` (kill-count sync) | PORTED (Kill half) | `app/netmodules.rs::BestiaryKill`, `app/netglue.rs::{broadcast_bestiary_kill,send_npc_killed,notify_progression_net}` | S1-I-5: module 4 carries `byte 0 + (short)npcNetId + Write7BitEncodedInt(killcount)` (`NetBestiaryModule.cs:16-23`), broadcast from `NPCKillsTracker.RegisterKill`'s `netMode == 2` arm (`NPCKillsTracker.cs:23-33`) and applied by `SetKillCountDirectly` (`:40-51`). `OnPlayerJoining` (`:98-107`, called at `MessageBuffer.cs:871`) sends the joining slot the whole snapshot. The `Sight`/`Chat` halves (`NetBestiaryModule.cs:25-39`) stay unported — those trackers are not net-synced yet. The `97`/`98` kill-credit and progression wires landed alongside (`MessageBuffer.cs:3558-3569`). |

## Coverage

| Achievement / bestiary group | C# count | Rust | Verdict |
| --- | --- | --- | --- |
| Achievement rows | 137 | 137 | **PASS** (order, ids, icons 0..136, one `Category::None`) |
| Achievement conditions | 180 | 180 | **PASS** (S1-E-18, `CreateMany` expanded) |
| Condition kinds modelled | 10 | 10 | **PASS** (`CondKind`) |
| Condition kinds with a live notify site | 10 | 10 | **PARTIAL** — wrappers exist for all kinds; only a subset of rows have reachable owning-system sites (many progression/custom rows still blocked) |
| Rows that can ever complete | 137 | >50 (growing) | **PARTIAL** — additional progression/custom/int/float rows wired in S1-E-18; exact reachable count depends on ported systems |
| Tracker shapes (`UseTracker*`) | 2 | 2 | **PASS** (shape and expanded denominators) |
| Achievement save/load | 1 | 1 (text) | **PARTIAL** — different format (documented), but save is now wired (completion + `save_world`, path-bound bootstrap) |
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
| 1 | `CreateMany` rows expanded to 1:1 (S1-E-18): all 180 conditions present. | behavioural | Resolved (S1-E-18) |
| 2 | Save is now wired (S1-E-18): path bound at bootstrap/world swap, saved on completion and from `save_world`; headless apps have no bound path. Format remains the port's plain-text `achievements.trdat` (not vanilla Rijndael+BSON). | behavioural / interop | Partial — call sites wired, format differs by design |
| 3 | Notify wrappers and additional sites landed (S1-E-18), but many progression/custom rows remain unreachable because their owning systems/paths are not ported (e.g. 17 partial, 18, 19/20, nurse `Pay`, and most other progression/custom flags). Progression 29 is explicitly blocked on S1-F-15. | behavioural | Partly — expanded, not complete |
| 4 | `kills_needed_for` returns 0 for net ids outside the 11-pair banner table; 0 means "≥ full", so 164 of 172 entries report tier 4 (`CanShowDropsWithDropRates`) with zero kills. The C# default is 50. | behavioural | **No** |
| 5 | `quick_unlock` on 5 entries instead of every boss + the override rows; the town NPC `Guide` wrongly carries it (the C# uses the chat provider). | behavioural | **No** |
| 6 | The town-NPC (chat) and critter (sight) unlock providers are absent, and the `sights`/`chats` sets are never recorded. | behavioural | Yes — `tr-core/src/bestiary.rs:124-130`, ANALYSIS "the sightings/chats stay empty until the recording behaviour lands" |
| 7 | `ReportDroprates` / the drop-rate tables are absent: the bestiary's drop list is a 4-row hand-written table, so the tier-3/tier-4 drop lines are empty for 168 of 172 entries. | behavioural / pixel | Yes — ANALYSIS.md:2969 "not ported — M4-BEST-1 territory" |
| 8 | `TryGrantingBestiary100PercentAchievement` (→ progression event 29 → `BOOK_WORM`) never fires. | behavioural | Yes — `tr-core/src/bestiary.rs:82` "no achievement hook (M6-ACH-1's)" |
| 9 | The achievement save format is plain text (`achievements.trdat`), not the vanilla Rijndael-CBC+BSON `"RELOGIC-TERRARIA"`. | behavioural (interop) | Yes — `achievements.rs:390-394` |
| 10 | ~~The unlock popup, `AchievementAdvisor` and the chat tag handler are missing.~~ **Popup DONE (S1-E-26 wave A)** — `ui/notifications.rs` + the drain seam; the advisor and the chat tag's snippet rendering remain open. | cosmetic / behavioural | Yes — ANALYSIS "The achievement unlock popup (S1-E-26 wave A)" |
| 11 | ~~`notify` returns only the last completed achievement; the C# fires `OnCompleted` (and a save) for every one.~~ **FIXED (S1-E-26 wave A)** — `notify_with` queues every completing row into `completed_events` (save + event per row); the return value keeps the last-name shape for the existing callers. Pinned by `one_event_completing_two_rows_queues_both`. | behavioural (minor) | Resolved |
| 12 | `completion_percent` divides by the port's 172-entry roster, not the C#'s full roster; combined with #4 it reads ~95% at start, which gates the Zoologist's shop (`Chest.cs:3265`). | behavioural | **No** |
| 13 | `ENTRIES` (172) vs `BESTIARY_NPC_SHEET_IDS` (174) — 2 sheet slots have no entry row. | cosmetic | Partly — BUG-15 removed the literal drift, not the skew |
| 14 | The kill-count and sight/chat sets iterate in sorted order vs the C#'s unspecified `Dictionary`/`HashSet` order — deterministic, but a different byte order in the save. | determinism | Yes — documented in `tr-core/src/bestiary.rs:96-100, 132-136` |
| 15 | `banner_to_item` keeps only 3 bands (276+/88+/default); the C# has the 292/291/290/289/273/251 specials and the 186-275 bands. It agrees on the port's 11 banners but is wrong on extension. | behavioural (latent) | Partly — `tr-core/src/bestiary.rs:286-287` |
| 16 | ~~No MP sync of the bestiary kill counts (NetBestiaryModule).~~ **PORTED (S1-I-5)** - module 4 carries the kill count, the 97/98 wires carry the credit and the progression id; the Sight/Chat halves are still open. | behavioural (MP) | Yes - pp/netmodules.rs::BestiaryKill, pp/netglue.rs |
| 17 | `Player.HasAchievement`, `Main.achievementTracker` and a `BossAchievement` type **do not exist** in the 1.4.5.8 decompile. The boss kill requirement is the bestiary's banner→`KillsToBanner` chain, and the achievement query is `Main.Achievements.GetAchievement(name).IsCompleted`. | n/a | n/a |
| 18 | `Achievement.completed` (flag) vs `is_completed()` (derived) disagree for a zero-condition row before and after a load. | behavioural (minor) | **No** |
| 19 | `CustomFloat` sub-unit progress (`Condition.frac`) is in-memory only; `load_pairs` resets it because the text save stores the whole `value` only. | behavioural (minor) | **No** (documented, S1-E-18) |
| 20 | Altar hammer smash is routed through `break_tile` + `smash_altar` in the active-tile branch (`app.ore_tiers` updated), firing BEGONE_EVIL; `tile_drop(26)=None` (C# empty drop group). Self-hurt/wraith halves of the C# refusal are follow-ups. | behavioural (partial) | Partly (S1-E-18) |
| 21 | `StopSlimeRain` sets the negative cooldown clock (`Main.cs:64787-64820`); `stopMoonEvent` does not clear `bloodMoon` (cleared in `UpdateTime_StartDay`). | behavioural (closer) | Yes (S1-E-18) |

## W4-f30 tail census — the second notify batch (2026-11-21)

The co-lane reservation (c36's claim above) gives f30 the *tail* sites:
progression 33-41/43-46, `MechaMayhem_*` (21) and the `CheckResearchAchievement`
family (45). The full C# call-site census of those ids, each marked WIRED (the
owning system is ported and the notify fires) or CITED (the owning system is
unported — the site stays unwired with its C# anchor, per the lane rule "sites
neither lane's systems own stay unwired and cited"). Code-only mode: the 13
witness tests are WRITTEN NOT RUN.

| id | row | C# site | verdict |
| --- | --- | --- | --- |
| 21 | MECHA_MAYHEM | `AchievementsHelper.cs:447-483` (`MechaMayhem_Kill`), `NPC.cs:79842`/`:79847` (`CountKillForAchievements`), `:80664-80685` (`SpawnBoss` bookends) | **WIRED** — the machine on `AchievementManager` (`mecha_mayhem_{clear,start,kill}`), the summon bookends in `use_boss_summon_item` (544 per-eye, 556/557 shared), the death-path switch + the `DropEoWLoot` last-segment boss latch (`NPC.cs:78414-78430`). |
| 33 | FEAR_THE_SUN | `Player.cs:28647` (`UpdateSunScorch`, the vampire 124/85/72 set at `sunScorchCounter` 120) | **WIRED (W4-f30 system 2)** — `sun_scorch.rs`: the exposure scan + the clamp-300 ramp + the 120-crossing burn (the `Array.Clear`/24-23-32 triple/mount-dismount/wings-rocket zero) + the vampire-set check; the `update_equips`-tail call site fires 33, the Molten `buffImmune[24]` gate reads the live flag. The sizzle sound / VampireOnFire particle / PlayerFrame side effects cited. |
| 34 | ITS_SHALING_OUTSIDE | `WorldGen.cs:71694` (`SpawnFallingObjects`, the boulder-rain start on drunk+getGood+storm) | CITED — `SpawnFallingObjects` / `_isRainingBoulders` unported (no boulder-rain weather pass). |
| 35 | EXTRA_LIFE | `Player.cs:38782` (`HardcoreDeathPenalty`, `Main.surviveHardcoreDeath` + `statLifeMax -= 60`) | **WIRED (W4-f30 system 3)** — `death.rs::hardcore_death_penalty`: the `survive_hardcore_death` seed predicate (DST+Anniversary unless FTW), the `max_hp-60` clamp + the 35 notify on the surviving world, `kill_me_for_good`'s `.plr`/`.bak` erase + session-draft reset; the ghost flag (`update_dead`'s hardcore gate) never respawns. |
| 36 | GRAVE_MISTAKE | `Player.cs:20473` (`OnKillNPC`, a gravestone projectile kills a town NPC) + `:38836` (`KillMe`, a teammate's gravestone) | **WIRED (W4-f30 system 1)** — the gravestone is the REAL aiStyle-17 projectile (`projectile.rs::ai_gravestone`, the `DropTombstone` spawn with the FTW `friendly&&hostile` pair); the `OnKillNPC` town-kill latch in `damage_pve` + the `KillMe` teammate-gravestone arm (the single-player self-kill stand-in, the MP deviation documented) both fire 36. The landing places tile 85 + the epitaph sign. |
| 37 | SPICY_LICKS | `Player.cs:46574` (The Axe 1305), `:46588` (Carbon/Ivy guitars 4057/4372), `:46603` (Sparkle Guitar 4715) | **WIRED (4715)** — the Sparkle Guitar's note spawn fires 37 (`fire_non_ammo_shot`'s 4715 arm, the chord-use stand-in). The Axe + Carbon/Ivy guitars have no ported chord-use arm — those two sites CITED. |
| 38 | ON_FLEEK | `Player.cs:25693` (`UpdateArmorSets`, Sunglasses head-slot 12 in the day window) | **WIRED** — `update_player`'s armor tail: head-slot 12 + `!remix && !rain && dayTime && time in [3600,50400] && y < worldSurface*16`. |
| 39 | FORTUNE_FAVORS_THE_BOULD | `Main.cs:61828` (`_rainbowBoulderMusicFramesCounter > 0`, the rainbow-boulder draw) | CITED — the rainbow boulder (proj 1047/1090) draw + its music counter are unported (the pet flag exists, `buff_effects_misc.rs:206`, but no boulder projectile/draw). |
| 40 | TRAINING_DAY | `Player.cs:25697` (`UpdateArmorSets`, `combatBookWasUsed && combatBookVolumeTwoWasUsed`) | CITED — the port carries `world.combat_book_was_used` but the volume-two book (5336) use is unported (`ItemCheck_UseCombatBook` `Player.cs:45337-45349` has no port; the second flag never sets). |
| 41 | MINI_ME | `Player.cs:25701` (`UpdateArmorSets`, the Moon Lord set 187/249/217) | **WIRED** — `update_player`'s armor tail: effective head 187 + body 249 + legs 217 (the vanity-over-functional fold). |
| 43 | NEW_DIGS | `NPC.cs:53273` (`AI_007`'s shimmer-dunk transform, `townNpcVariationIndex == 1`) | **CORE PORTED (W4-f30 system 4)** — `town_shimmer.rs::town_shimmer_step`: the RNG-free state-25 decision core (the ai2 hold/climb/transparency-decay/completion variant-flip + the 43 gate). The full visual/teleport machine + the inline 43 wire are the liquid-shimmer lane's (W4-f47 `e5aebe3e`, on its unmerged branch); the town AI itself is unwired on this base (the `update_guide` stand-in) and the shimmer trigger (buff 353 → GetShimmered) is f47's — the core is exercised by the witnesses, the wiring cites it. |
| 44 | MY_PEOPLE_NEED_ME | `Player.cs:24076` (`BordersMovement`, the remix/forced-gravity top-world death) | CITED — `BordersMovement` is unported (no world-border movement pass; the port's player has no top-world kill). |
| 45 | TRASH_COMPACTOR | `AchievementsHelper.cs:387-399` (`CheckResearchAchievement`), `Player.cs:325` (`EnterWorld`, forced) + `:25703` (per tick) | **WIRED (W4-f30)** — `App.research` + `last_research_version`; the per-tick armor-tail check (`world_sim/player.rs`) + the forced `enter_loaded_world` check both call `check_research_achievement` → 45. The live sacrifice UI is unported, so the tracker never edits in play and the check is inert until the Journey research lands — but the helper, the count, the App field and both call sites are wired (PREP → WIRED). |
| 46 | INTERDIMENSIONAL_RECYCLING | `Main.cs:40294` (`DryadText_Do_StardewValleyBit`, the Joja-cola gift) | **WIRED (W4-f30 system 5)** — VERIFIED LIVE (`NPCInteractions.cs:517` registers `StardewValleyBit`; the dead wrapper is `DryadText_WorldPurityOrStardewValleyBit`). `dryad_stardew.rs`: `CanDryadPlayStardewAnimation`, the gift (`ConsumeItem(5275, reverseOrder)` + the Dryad state-24 + the Junimo 995 spawn + the 46 notify), the Purify button + the `OverrideChatTextWithShenanigans` reminder/reroll machine. The AI_192 Junimo animation / the 144 relay / the DryadPurification status button cited. |

Also landed: `tr_core::seeds::SeedFlags::mechdusa` (`SpecialSeedFeatures.cs:119-128`,
remix && for-the-worthy) — the gate the QUEEN_MACHINE (31) death arm
(`NPC.cs:78857-78869`, the Mechdusa last-mech kill) reads. Progression 31 itself
stays CITED: the Mechdusa summon (5334) + its combined-boss death are unported
(the summon seam handles 544/556/557 only; `SpecialSeedFeatures.Mechdusa`'s
`SpawnMechQueen` has no port).

**Net (after the W4-f30 owning-systems batch):** 9 progressions wired
(21, 33, 35, 36, 37-partial, 38, 41, 45, 46), 1 core ported (43 — the decision
core; the visual/teleport machine + the inline wire are f47's lane), 4 cited
(34, 39, 40, 44) + 31 cited. The e22 condition table (180 conditions) is
untouched — f30 wires notify sites only, never `achievements_data.rs`.

The owning-systems batch (2026-11-21, user order "port UNPORTED OWNING
SYSTEMS"): system 1 gravestone kills (36), system 2 sun-scorch (33), system 3
hardcore death penalty (35), system 4 town shimmer decision core (43, the
lane-split with f47), system 5 Dryad Joja bit (46, verified LIVE not dead
code), then the prepared research 45 wired. The boulder-rain (34),
rainbow-boulder (39), combat-book-v2 (40) and Mechdusa-summon (31) systems
stay with their active lanes (worldgen / d54 orch / g89b / boss-summon).

### W4-f30 RETIRE phase (2026-11-21) — `240-f30-systems-retire.md`

The sibling lanes landed their overlapping halves (c46 hardcore `7330cd4e`,
d70 sun-scorch `a7620eec`, c24 Journey sacrifice `ff99f148`, f47 shimmer
`1997be09`, d2 tombstones `f430ccf8` — all UNMERGED at this writing). The
retire verdicts, the exact keep-one drop-hunks and the compose meeting points
are published in **`240-f30-systems-retire.md`** (number per the d54 collision
registry 150): 35 → KEEP c46 (the verbatim two-file delete + the no-draft-clear
ruling + the seeded `update_dead`), 33 → KEEP f30's scan/33 wire, DROP d70's
method copies (its lens-flare/shadow/HeatDistortion halves survive on the kept
field), 45 → COMPOSE with c24 (its clamped `register_sacrifice` + f30's
`LastEditId` bump at the `MarkContentsDirty` slot; the forced EnterWorld check
is the load-path 45 site — C# `Load` does not dirty), 43 → KEEP f47's wired
machine, DROP f30's core (+ add the `netMode != 1` completion gate f47 omits),
36 → KEEP d2's verbatim flight + f30's 36 machinery/date/FTW-hostile folds.
The census table above stays the band's record; the FINAL systems-family
declaration (the lane is census-complete for its reserved band) is 240 §6.

## Task seeds

- ~~**S** — Model `NPCKilledCondition.CreateMany` faithfully~~ — **done (S1-E-18)**.
- ~~**S** — Call `achievements::save` from the autosave/F9 pass~~ — **done (S1-E-18)** (save on completion + `save_world`, path-bound bootstrap).
- **M** — Continue wiring notify sites for progression/custom rows as owning systems land (remaining: 17 partial, 18, 19/20, nurse `Pay`, most progression/custom flags; 29 blocked on S1-F-15). Partially done (S1-E-18). Dep: owning systems.
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
- ~~**S** — Add the achievement unlock toast and fire `OnCompleted` for every
  completed achievement (not only the last). Fixes #10, #11.~~ **DONE
  (S1-E-26 wave A, 2026-11-18)** — the popup + the per-row event queue; the
  advisor / chat-tag rendering halves of #10 stay open.
- **S** — Reconcile `ENTRIES` (172) with `BESTIARY_NPC_SHEET_IDS` (174) and add
  a test that every sheet slot has an entry and vice versa. Fixes #13.
- **DONE (S1-I-5, 2026-11-16)** — `NetBestiaryModule` is ported: the host broadcasts
  `SerializeKillCount` on a kill (`NPCKillsTracker.cs:23-33`) and syncs the counts on
  player join (`:98-107`), with the `97`/`98` achievement wires
  (`MessageBuffer.cs:3558-3569`). Fixes #16. The `Sight`/`Chat` halves remain open.
- **S** — Add a data test that pins every Rust row's condition count against
  `AchievementInitializer.cs` (137 rows / 180 conditions) so the `CreateMany`
  class of bug cannot return. Dep: the first S task.
