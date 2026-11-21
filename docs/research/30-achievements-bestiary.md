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
  `WorldBestiary` (138), `npc_to_banner` (215), `kills_needed_for` (979),
  `banner_to_item` (1043).
- `crates/tr-app/src/bestiary.rs` (1602) — `BestiaryEntryDef` (13),
  `ENTRIES` (30-2173, 249 rows — W4-c36 re-probe), `unlock_state` (2179),
  `completion_percent` (2192), `entry_for_npc` (2306), `kill_progress`.
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
| Notify sites: progression event / custom flag | PARTIAL | `achievements.rs`, `world_sim/{clock,editing,npcs,weather}.rs`, `chest.rs` | S1-E-18 wires progression 1/3/5/6/8/9/16 and selected custom sites (`Finish`, `Peek`, `Do`, `Move`); the c36 batches add the rest of the wired table below. Remaining progression/custom rows are blocked on missing systems (the 20-row gap list, re-probed below) — 19/20 and nurse `Pay` landed via S1-E-18b/nurse-pay. |
| `AchievementsHelper.TryGrantingBestiary100PercentAchievement` | PORTED | `achievements.rs::try_grant_bestiary_100` (417), `app/netglue.rs::try_grant_bestiary_100_net` (2986) | `NotifyProgressionEvent(29)` → `BOOK_WORM` is wired: the kill hook fires from the death seam (`world_sim/npcs.rs:5274`), and S1-I-5 routes it over `98` on a host. W4-c36 re-probe: the S1-F-15 tier-4 denominator bug is FIXED on main (the ladder row below), so a fresh world reads 0% — the residual early-100% risk is the town/critter provider fold (the flat kill-ladder treats every entry as a Common enemy; c37's provider enum owns the fix), NOT the denominator. |
| `AchievementManager.Save`/`Load` (Rijndael+BSON) | PARTIAL (documented) | `achievements.rs::{save,load}`, `app/{mod,saveglue}.rs` | Port still writes `achievements.trdat` (plain text), not the vanilla Rijndael-CBC+BSON blob. S1-E-18 binds a save path at bootstrap/world swap and saves on completion and from `save_world`; headless test apps have no bound path. |
| Achievement unlock popup / advisor / chat tag | PARTIAL (popup DONE) | `ui/notifications.rs`, `app/mod.rs::drain_achievement_completions` | **S1-E-26 wave A**: `AchievementUnlockedPopup` (`UI/InGamePopups.cs:13-148`) + `InGameNotificationsTracker` (`:11-113`) ported — the 300-tick timer/curves/draw/click, the `netMode != 2` gate, the `AchievementInitializer.cs:728-735` chat line + tracked `AchievementComplete` cue, `OpenAchievementsAndGoto`'s scroll, `SaveAndQuit`'s `Clear`. `AchievementAdvisor` (`UI/AchievementAdvisor.cs`) and the chat tag's snippet rendering (`AchievementTagHandler`, `UI/Chat/`) still have no port (the line prints the raw `[a:NAME]` tag). |
| `SocialAPI.Achievements` (Steam stats) | OUT OF SCOPE | — | `PORT_MAP.md:268` marks the platform hooks as out of scope. |
| `BestiaryEntry` + the 4 factories | PORTED (simplified) | `tr-app/src/bestiary.rs::BestiaryEntryDef` (13) | One flat struct (credit id, net id, kind, variant, quick_unlock, name) instead of the C#'s `Info` list of `IBestiaryInfoElement`s (`BestiaryEntry.cs:24-30`). |
| NPC → entry mapping | PARTIAL | `ENTRIES` (30-2173), `entry_for_npc` (2306) | 249 entries (W4-c36 re-probe count) vs the C#'s "one per `ContentSamples.NpcsByNetId`" (the C# roster is the whole NPC id space). The sheet table `BESTIARY_NPC_SHEET_IDS` has 181 slots (`assets.rs:1863`, W4-c36 re-probe) — the entries exceed the sheet (the DD2/moon families carry rows without sheet slots). |
| `NPCKillsTracker` (counts + cap + save) | PORTED | `tr-core/src/bestiary.rs::KillTracker` (65) | The 999999999 cap (57, 84), `register_kill` (76), `set_kill_count_directly` (83), `pairs`/`load_pairs` (101/106). |
| Kill-counter persistence (`.wld` section 10) | PORTED | `tr-core/src/wld/sections/bestiary.rs` (535) | All three sub-lists round-trip both ways; 17 tests. The sights/chats half stays empty (documented in `tr-core/src/bestiary.rs:124-130`). |
| Unlock ladder (1 / ⅕ / ½ / full + boss quick-unlock) | **PORTED** (S1-F-15 landed; W4-c36 re-probe re-verified) | `get_unlock_state_by_kill_count` (31-54), `kills_needed_for` (979) | The ladder is a faithful port of `CommonEnemyUICollectionInfoProvider.cs:50-76` and the denominator is FIXED: `kills_needed_for` walks `npc_to_banner → banner_to_item → kills_to_banner_item`, whose default arm returns `ItemID_Sets_DefaultKillsForBannerNeeded` (50, `ItemID.cs:1303`) — bannerless ids read `BannerToItem(0)` = 1614 → the same 50. Pinned by `the_family_banner_chain` (`kills_needed_for(50/17) == 50`) + `kills_needed_for_honors_kills_to_banner_overrides`. The old "0 → tier 4 with zero kills" bug is gone. |
| `quickUnlock` per entry | PARTIAL | `BestiaryEntryDef.quick_unlock` | 24 entries carry it (W4-c36 re-probe count: the boss family + Deerclops + EaterofWorldsHead); the C# sets it on **every** boss (`BestiaryEntry.cs:37-41`) plus the 494-506 salamander and the `HighestOfMultiple` overrides. `Guide` no longer carries it (the town rows read `quick_unlock: false`) — c37's provider-enum task owns the rest. |
| Town-NPC / critter unlock providers | MISSING | — | `TownNPCUICollectionInfoProvider` (chat) and `CritterUICollectionInfoProvider` (sight) have no port; the `sights`/`chats` sets are never recorded. |
| `BestiaryDatabase.Merge` → `ReportDroprates` | **MISSING** | — | No `report_droprates` anywhere in `crates/`. The bestiary's "Drops:" list reads `loot_db::drop_lines` (`ui/bestiary.rs:1203`), a hand-written 4-row table, so 168 of 172 entries print "(nothing recorded yet)". |
| `GetBestiaryProgressReport` | PARTIAL | `completion_percent` (2192) | Same predicate (`unlock_state(...).known()`); the tier-4 inflation is FIXED (S1-F-15) — a fresh tracker reads 0.0 (pinned, `ui/bestiary.rs:2311`). The remaining deviation is the denominator: the port's 249-entry roster (W4-c36 re-probe count), not the C#'s full NPC id space, and the flat kill-ladder fold for town/critter entries. The Zoologist gate (`Chest.cs:3265`) reads it. |
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
| Bestiary entry rows | whole NPC roster (1 per `NpcsByNetId`) + 44 manual | 249 (W4-c36 re-probe) | **PARTIAL** — port roster only; the entries now exceed the 181-slot sheet |
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
| 3 | Notify wrappers and additional sites landed (S1-E-18), but many progression/custom rows remain unreachable because their owning systems/paths are not ported (e.g. 17 partial, 18, and the 20-row gap list below). Progression 29's hook FIRES (S1-E-18p29); the old S1-F-15 denominator blocker is fixed (see #4). | behavioural | Partly — expanded, not complete |
| 4 | ~~`kills_needed_for` returns 0 for net ids outside the 11-pair banner table; 0 means "≥ full", so 164 of 172 entries report tier 4 with zero kills.~~ **FIXED (S1-F-15, re-verified W4-c36)** — the default arm returns 50 (`ItemID_Sets_DefaultKillsForBannerNeeded`), pinned by `the_family_banner_chain` + the override test. | behavioural | Resolved (S1-F-15) |
| 5 | `quick_unlock` on 24 entries (W4-c36 re-probe count) instead of every boss + the override rows; the town `Guide` no longer carries it. The salamander/HighestOfMultiple overrides ride c37's provider enum. | behavioural | Partly — c37 owns the provider fold |
| 6 | The town-NPC (chat) and critter (sight) unlock providers are absent, and the `sights`/`chats` sets are never recorded. | behavioural | Yes — `tr-core/src/bestiary.rs:124-130`, ANALYSIS "the sightings/chats stay empty until the recording behaviour lands" |
| 7 | `ReportDroprates` / the drop-rate tables are absent: the bestiary's drop list is a 4-row hand-written table, so the tier-3/tier-4 drop lines are empty for 168 of 172 entries. | behavioural / pixel | Yes — ANALYSIS.md:2969 "not ported — M4-BEST-1 territory" |
| 8 | ~~`TryGrantingBestiary100PercentAchievement` (→ progression event 29 → `BOOK_WORM`) never fires.~~ **FIRES (S1-E-18p29)** — the death-seam hook (`world_sim/npcs.rs:5274` → `try_grant_bestiary_100_net`) + the `98` MP wire; the gate is the completion fraction (see #12). | behavioural | Resolved (S1-E-18p29) |
| 9 | The achievement save format is plain text (`achievements.trdat`), not the vanilla Rijndael-CBC+BSON `"RELOGIC-TERRARIA"`. | behavioural (interop) | Yes — `achievements.rs:390-394` |
| 10 | ~~The unlock popup, `AchievementAdvisor` and the chat tag handler are missing.~~ **Popup DONE (S1-E-26 wave A)** — `ui/notifications.rs` + the drain seam; the advisor and the chat tag's snippet rendering remain open. | cosmetic / behavioural | Yes — ANALYSIS "The achievement unlock popup (S1-E-26 wave A)" |
| 11 | ~~`notify` returns only the last completed achievement; the C# fires `OnCompleted` (and a save) for every one.~~ **FIXED (S1-E-26 wave A)** — `notify_with` queues every completing row into `completed_events` (save + event per row); the return value keeps the last-name shape for the existing callers. Pinned by `one_event_completing_two_rows_queues_both`. | behavioural (minor) | Resolved |
| 12 | `completion_percent` divides by the port's 249-entry roster (W4-c36 re-probe), not the C#'s full roster; the #4 tier-4 inflation is FIXED so a fresh world reads 0% (pinned). The residual early-100% risk is the flat kill-ladder fold for town/critter entries (one kill = portrait tier where the C# needs a chat/sight) — c37's provider enum owns it. The Zoologist shop (`Chest.cs:3265`) reads the fraction. | behavioural | Partly — denominator documented, provider fold c37's |
| 13 | `ENTRIES` (249) vs `BESTIARY_NPC_SHEET_IDS` (181) — W4-c36 re-probe: the entries now EXCEED the sheet (the DD2/moon families carry rows without sheet slots); the old "2 sheet slots have no entry" skew inverted. | cosmetic | Partly — BUG-15 removed the literal drift, not the skew |
| 14 | The kill-count and sight/chat sets iterate in sorted order vs the C#'s unspecified `Dictionary`/`HashSet` order — deterministic, but a different byte order in the save. | determinism | Yes — documented in `tr-core/src/bestiary.rs:96-100, 132-136` |
| 15 | ~~`banner_to_item` keeps only 3 bands (276+/88+/default).~~ **FIXED (re-verified W4-c36)** — the port now carries the 292/291/290/289 specials, the 276+/274-275/273/267+/257+/252+/251/249+/186+/88+ bands and the default (`tr-core/src/bestiary.rs:1043-1087`, the full `BannerSystem.cs:341-401` shape). | behavioural (latent) | Resolved |
| 16 | ~~No MP sync of the bestiary kill counts (NetBestiaryModule).~~ **PORTED (S1-I-5)** - module 4 carries the kill count, the 97/98 wires carry the credit and the progression id; the Sight/Chat halves are still open. | behavioural (MP) | Yes - pp/netmodules.rs::BestiaryKill, pp/netglue.rs |
| 17 | `Player.HasAchievement`, `Main.achievementTracker` and a `BossAchievement` type **do not exist** in the 1.4.5.8 decompile. The boss kill requirement is the bestiary's banner→`KillsToBanner` chain, and the achievement query is `Main.Achievements.GetAchievement(name).IsCompleted`. | n/a | n/a |
| 18 | `Achievement.completed` (flag) vs `is_completed()` (derived) disagree for a zero-condition row before and after a load. | behavioural (minor) | **No** |
| 19 | `CustomFloat` sub-unit progress (`Condition.frac`) is in-memory only; `load_pairs` resets it because the text save stores the whole `value` only. | behavioural (minor) | **No** (documented, S1-E-18) |
| 20 | Altar hammer smash is routed through `break_tile` + `smash_altar` in the active-tile branch (`app.ore_tiers` updated), firing BEGONE_EVIL; `tile_drop(26)=None` (C# empty drop group). Self-hurt/wraith halves of the C# refusal are follow-ups. | behavioural (partial) | Partly (S1-E-18) |
| 21 | `StopSlimeRain` sets the negative cooldown clock (`Main.cs:64787-64820`); `stopMoonEvent` does not clear `bloodMoon` (cleared in `UpdateTime_StartDay`). | behavioural (closer) | Yes (S1-E-18) |

## Lane claim — W4-c36 (achievement notify + save), 2026-11-20

**LANDED 2026-11-21** on `task/W4-c36-achievement-notify-save`: the first
batch of missing `AchievementsHelper.Notify*`/`Handle*` sites wired per the
C# call-site census, plus the `AchievementManager.Save` wire audit.
**Co-lane reservation for W4-f30 (achievement-notify-2, second batch):** f30
owns the *tail* sites — progression 33-41/43-46 (research, golf, fishing
milestones, shimmer/void bag, nurse 46, dryad 27 special-event),
`MechaMayhem_*` (progression 21) and the `CheckResearchAchievement` family;
c36 took the first batch. Sites neither lane's systems own stay unwired and
cited below. Condition-table work is W4-e22's (its branch pins the
180-condition table) — c36/f30 touch **notify sites only**, never
`achievements_data.rs`.

### W4-c36 landed sites (C# → Rust)

| C# site | Rust seam | Row(s) |
| --- | --- | --- |
| `GetCondition(row, cond).Complete()` (`AchievementManager.cs:220-227`, `AchievementCondition.cs:37-47`) | `achievements.rs::complete_condition` — the row-targeted primitive; save + event on the last condition | all |
| Drowned-sip `HandleSpecialEvent(25)` (`Player.cs:5545`/`:42873`) | `potions.rs` both drink paths re-pointed off the lossy `notify_custom("Use")` broadcast | DRINK_BOTTLED_WATER_WHILE_DROWNING |
| Shadow-chest `HandleSpecialEvent(16)` (`Player.cs:33245`) | `chest.rs::toggle_open` re-pointed | I_AM_LOOT |
| `HandleAnglerService` (`Main.cs:40260`, `AchievementsHelper.cs:411-418`) | `world_sim/npcs.rs` — the flag completes by name, the four `Finish` counters keep the shared dispatch | SERVANT_IN_TRAINING + the four counters |
| Natural blood moon / eclipse (`Main.cs:65352`/`:65425`) | `world_sim/clock.rs::advance_time_with_events` — progressions 4/2 (no shipped row listens; faithful no-op wires) | — |
| `HandleOnEquip` (`AchievementsHelper.cs:239-280`) | `app/achievement_hooks.rs::handle_on_equip` at `equip_slot_click`/`vanity_swap`/`swap_equip_click` (`ItemSlot.cs:955-975`/`:1880-1897`/`:3297-3355`) | HOLD_ON_TIGHT, THE_CAVALRY, HEAD_IN_THE_CLOUDS, MATCHING_ATTIRE, FASHION_STATEMENT, DYE_HARD |
| `OnPlayerEnteredWorld` (`AchievementsHelper.cs:94-198`) | `app/achievement_hooks.rs::on_player_entered_world` at `enter_loaded_world` — the pickup fan-out + the stat/equip flags | STAR_POWER, TOPPED_OFF, the equip rows, GAIN_TORCH_GODS_FAVOR, the pickup rows |
| Blood Moon Starter 4271 / Solar Tablet 2767 (`Player.cs:44009-44027`/`:43988-44008`) | `world_sim/editing.rs::use_blood_moon_starter`/`use_solar_tablet` | progression 4 (no-op wire) |
| Mana Crystal / Life Fruit / Life Crystal (`Player.cs:45274-45320`) | `world_sim/editing.rs::use_permanent_powerup` | STAR_POWER, GET_A_LIFE, TOPPED_OFF |
| Torch God's Favor 5043 (`Player.cs:45226-45242`) | `world_sim/editing.rs::use_torch_gods_favor` + the bestiary TorchGod credit | GAIN_TORCH_GODS_FAVOR |
| Yoyo/kite shoot tail (`Player.cs:49454-49461`) | `world_sim/editing.rs::fire_non_ammo_shot` tail — aiStyle 99 / 160+windy | THROWING_LINES, FLY_A_KITE_ON_A_WINDY_DAY |
| `FreeUpPetsAndMinions` 9-slot test (`Player.cs:51826-51829`) | `projectile.rs::free_up_pets_and_minions` returns the test; the use-start caller fires | YOU_AND_WHAT_ARMY |
| `UpdateBiomes` reach tail (`Player.cs:16732-16776`) + `BordersMovement` (`:24099`/`:24125`) | `app/achievement_hooks.rs::reach_check` at the player pass — walls 86/62, special seed, underworld/glowshroom/graveyard ladder + the funkytown cooldown, the border reads | WHERES_MY_HONEY, JEEPERS_CREEPERS, PLAY_ON_A_SPECIAL_SEED, ITS_GETTING_HOT_IN_HERE, FUNKYTOWN, FOUND_GRAVEYARD, INTO_ORBIT, ROCK_BOTTOM |
| `KillMe` trap + rigged-chest hooks (`Player.cs:38822-38828`) | `death.rs::on_player_death` head; the 600-tick window arms at `chest.rs` (`:33114`/`:33257`), decays in the player tick (`:24617-24620`); the trap flag arms at `projectile.rs::damage_player` (`Projectile.cs:14872-14879`) and clears in the buff tail (`Player.cs:25569-25572`) | WATCH_YOUR_STEP, DIE_TO_DEAD_MANS_CHEST |
| Fall-damage LUCKY_BREAK (`Player.cs:25354-25357`/`:25371-25374`) | `player.rs` fall block queues `pending_lucky_break`; the player pass drains | LUCKY_BREAK |
| Moon wave 15 (`NPC.cs:78569-78572`/`:78703-78706`) | `world.rs::moon_event_kill_walk` returns the wave-15 advance; the death pass notifies 14/15 | ICE_SCREAM, BALEFUL_HARVEST |
| Bee set + Bee Gun (`ArmorSetBonuses.cs:23-30`) | the player pass reads the completed set + the swinging 1121 | NOT_THE_BEES |
| Unicorn mount + Rainbow Gun (`Mount.cs:4955-4958`) | the player pass reads mount 10 + the swinging 1260 | RAINBOWS_AND_UNICORNS |
| Lava fishing (`Projectile.cs:20798-20801`) | `fishing.rs::set_fishing_check_results` reports the condition; the bobber queues, the projectile pass drains | GO_LAVA_FISHING |
| Max-happiness talk (`Player.cs:4391-4394`) | `world_sim/npcs.rs::try_open_town_shop_at_cursor` — housed + `PriceAdjustment <= 0.82` | TALK_TO_NPC_AT_MAX_HAPPINESS |
| Orb/heart smash (`WorldGen.cs:54331`) | `edit/kill.rs::check_orb` sets `World::orb_smash_notify_pending`; `break_tile` drains → progression 7 | SMASHING_POPPET |

### Save-wire audit (W4-c36)

The C#'s ONLY external `Main.Achievements.Save()` call site is
`Player.InternalSavePlayer`'s first statement (`Player.cs:54700`); the other
`Save()` calls are the manager's own (`AchievementCompleted`
`AchievementManager.cs:180-187`, `Clear`/`ClearAll` `:152-178` — no retail
caller, the debug menu's). The port's wires match: `save_world`'s head
(`saveglue.rs`, every exit route funnels through it — autosave/F9/close/quit),
the per-completion save in `notify_with` AND the new `complete_condition`,
the bootstrap/world-swap path bind (`bind_achievements_save_path`), and the
`App::new` load. `Clear`/`ClearAll` have no retail caller — unported, cited.
Witnesses: `save_world_writes_the_achievement_condition_state`,
`completing_an_achievement_writes_the_file_immediately`,
`w4_c36_the_row_targeted_completion_writes_the_file`,
`w4_c36_save_and_quit_writes_the_achievement_state`.

### Sites left unwired (owning systems absent — f30 or later lanes)

- Progression 17/26 (`WorldGen.CheckAchievement_RealEstateAndTownSlimes`,
  `WorldGen.cs:5659-5677`): **check PORTED (DOC-SWEEP)** —
  `achievement_hooks.rs::check_real_estate_and_town_slimes`, wired at the
  arrival (`:5609`), the fished-out 682 (`Player.cs:52435`) and the
  rainbow-slime release (`WorldItem.cs:1742`). UNREACHABLE in play: Truffle
  160, Steampunker 178, Cyborg 209, Pirate 229 (FOUR kinds — the W4-c36
  re-probe: Bartender 550 IS ported, `DD2Bartender`) and the eight
  town slimes 670/678-684 have no NpcKind rows.
- Progression 18 (Truffle arrival, `WorldGen.cs:5580`): no Truffle NpcKind.
- ~~Progression 22 (`WorldGen.UnlockDoor`, `WorldGen.cs:37627`)~~ **PORTED
  (DOC-SWEEP)** — `editing/doors.rs::try_unlock_temple_door` + `unlock_door`:
  the Temple Key 1141 pay (inventory + void-bag bank4), the +54 frame shift,
  sound 22, progression 22. The fairy NPC sight (`NPC.cs:48647`,
  HandleSpecialEvent 22 → FIND_A_FAIRY "Do") still has no fairy kind.
- Progression 24 gnome-to-statue (`NPC.cs:55896`): no gnome AI.
- `HandleSpecialEvent(9)` minecart kill (`Player.cs:27602`): the minecart
  ride is ported (`minecart.rs`) but the NPC-collision damage block
  (`GetMinecartDamage` + `ApplyDamageToNPC`, `Player.cs:27585-27604`) is
  not — VEHICULAR_MANSLAUGHTER stays unwired; `(21)` pet-the-pet
  (`Player.cs:33350`/`:33397`): no petting system.
- Progressions 33-46 (f30's reserved tail): see the f30 census below.
- `DoClassicTitleScreenAchievement` (`AchievementsHelper.cs:382-385`,
  called at `Main.cs:63018` — the title-screen sun/moon grab): the classic
  title screen is unported → GOING_OLDSCHOOL unwired.

## W4-c36 DOC-SWEEP — lane 32 (2026-11-21)

Co-lane reads (pushed tips): **f30** `f8cef8cb` (tail census: 21/37-partial/
38/41 WIRED, 45 PREP, 31/33/34/35/36/39/40/43/44/46 CITED — reproduced in
its "W4-f30 tail census" section on its branch); **g53** `a27adca8` (the
achievements-screen sweep — the POPUP toast is already DONE on main via
S1-E-26 wave A `d6b4fa09`, draw wired at `app/frame/draw.rs:1858`, and no
g53 commit touches `ui/notifications.rs` — **skipped, nothing to take**).

### 5043 Torch God's Favor — the 3-way keep-one note (c36 / f33b / g90b)

The three branches touch ADJACENT C# methods, not the same logic — no
keep-one deletion is needed, the split is by responsibility:

| lane | owns | C# anchor |
| --- | --- | --- |
| **c36 (this branch)** | the USE path: `use_torch_gods_favor` (`editing.rs`) — `unlockedBiomeTorches` set, GAIN_TORCH_GODS_FAVOR completion (the ONLY achievement-credit path in the three), the bestiary TorchGod 664 credit, the stack pay | `Player.ItemCheck_UseTorchGodsFavor` `Player.cs:45226-45242` |
| **f33b** (`333479e0`) | the EVENT machine: `torch_god.rs` happyFunTorchTime + the 949 rain + the survival 5043 DROP (`RequestNewItem`), and the trigger-gate flag READS — `has_torch_gods_favor` (the inventory scan, `Player.cs:18069-18076`) and `ctx.unlocked_biome_torches` (`:18066`). f33 completes NO achievement (grep: no `complete_condition`/`notify` in its diff) | `Player.cs:17718-17910`/`:18066-18086`, `Projectile.cs:42787-42822` |
| **g90b** (`1f1b88f0`) | the NEIGHBOUR item: Artisan Loaf 5326 `use_artisan_loaf` at the same i37 press-edge seam — a different flag (`ateArtisanBread`), no 5043 code | `Player.ItemCheck_UseArtisanLoaf` `Player.cs:45216-45224` |

Merge contract: keep c36's flag WRITE + credit and f33's flag READS (the
event gate needs the use flag); the `editing.rs` use-seam arms are disjoint
`if held.kind == …` branches (mechanical conflict at most). The C#'s second
flag `UsingBiomeTorches = true` (`:45231`) stays folded into
`unlocked_biome_torches` (the c36 batch-6 deviation, `player_lights.rs:97`).

### DOC-SWEEP landed sites (C# → Rust)

| C# site | Rust seam | Row(s) |
| --- | --- | --- |
| Coffin-cart mount (`Mount.cs:6301-6304`, `_type == 38`) | `player.rs::set_mount` queues `pending_grave_cart`; the player pass drains → progression 32 | ROLLIN_IN_YOUR_GRAVE |
| FTW Moon Lord kill (`NPC.cs:79398-79402`, `Main.getGoodWorld` = the ForTheWorthy bit, `WorldGen.cs:11118`) | `world_sim/npcs.rs`'s MoonLordCore strike-death arm → progression 30 | BOULDER_LORD |
| Locked Lihzahrd door (`Player.cs:32792-32834` + `WorldGen.UnlockDoor` `:37599-37627`) | `editing/doors.rs::try_unlock_temple_door`/`unlock_door` — the 1141 key pay + bank4 fallback, the frameY-594 top walk, the +54 shift, sound 22, progression 22 | TEMPLE_RAIDER |
| `CheckAchievement_RealEstateAndTownSlimes` (`WorldGen.cs:5659-5677`) | `achievement_hooks.rs::check_real_estate_and_town_slimes` at the arrival (`:5609`), the fished-out 682 (`Player.cs:52435`), the rainbow-slime release (`WorldItem.cs:1742`) | REAL_ESTATE_AGENT + ALL_TOWN_SLIMES (faithful no-op wires — the roster kinds are unported) |

Witnesses (WRITTEN NOT RUN, code-only):
`w4_c36_the_coffin_cart_mount_completes_rollin_in_your_grave`,
`w4_c36_the_temple_key_unlocks_the_lihzahrd_door`,
`w4_c36_the_ftw_moon_lord_kill_fires_boulder_lord`,
`w4_c36_the_real_estate_check_needs_the_full_roster`.

### The 180-condition notify-path census (gap list)

Method: every `Cond::Progression/Flag/Int/Float` row in
`achievements_data.rs` (86 conditions) mapped to its live Rust notify site;
the `Npc/Item/Tile/Craft` families (94 conditions) all have a live DISPATCH
(`world_sim/npcs.rs` kill, `items.rs` pickup, `editing.rs` break,
`clicks.rs` craft + the enter-world fan-out) — their per-id reachability is
a content-coverage question (the NPC-kind/item tables), not a notify-path
gap. Progression 0/2/4 have no listening row (faithful no-op wires).

**Progression/flag/int/float conditions with NO notify path anywhere (the
gap list, after c36+f30):**

| Row | Cond | C# site | Missing owning system |
| --- | --- | --- | --- |
| IT_CAN_TALK | 18 | `WorldGen.cs:5580` | Truffle kind 160 |
| TURN_GNOME_TO_STATUE | 24 | `NPC.cs:55896` | gnome AI |
| QUEEN_MACHINE | 31 | `NPC.cs:78866` | Mechdusa summon 5334 + combined-boss death (f30 cited; the seed gate landed) |
| FEAR_THE_SUN | 33 | `Player.cs:28647` | vampire sun-scorch (f30 cited) |
| ITS_SHALING_OUTSIDE | 34 | `WorldGen.cs:71694` | boulder rain (f30 cited) |
| EXTRA_LIFE | 35 | `Player.cs:38782` | hardcore death penalty (f30 cited) |
| GRAVE_MISTAKE | 36 | `Player.cs:20473`/`:38836` | striking gravestones + MP teammate deaths (f30 cited) |
| FORTUNE_FAVORS_THE_BOULD | 39 | `Main.cs:61828` | rainbow boulder draw (f30 cited) |
| TRAINING_DAY | 40 | `Player.cs:25697` | combat book vol. 2 use 5336 (f30 cited) |
| NEW_DIGS | 43 | `NPC.cs:53273` | town shimmer transform (f30 cited) |
| MY_PEOPLE_NEED_ME | 44 | `Player.cs:24076` | remix top-world death (f30 cited) |
| TRASH_COMPACTOR | 45 | `AchievementsHelper.cs:396` | Journey sacrifice UI (f30 PREP) |
| INTERDIMENSIONAL_RECYCLING | 46 | `Main.cs:40294` | Dryad Joja-cola gift (f30 cited) |
| SPICY_LICKS (partial) | 37 | `Player.cs:46574`/`:46588` | The Axe 1305 / Carbon+Ivy 4057/4372 chord arms (the 4715 arm wired by f30) |
| VEHICULAR_MANSLAUGHTER | "Hit" | `Player.cs:27602` | minecart NPC-collision damage (`GetMinecartDamage` block) |
| PET_THE_PET | "Do" | `Player.cs:33350`/`:33397` | petting system |
| FIND_A_FAIRY | "Do" | `NPC.cs:48647` | fairy kind |
| PURIFY_ENTIRE_WORLD | "Do" | `Main.cs:40276` (`DryadText_WorldPurityOrStardewValleyBit`) | Dryad world-status dialog (`GetDryadWorldStatusDialog` + the status button) |
| TO_INFINITY_AND_BEYOND | "Do" | `Projectile.cs:70861` (1020 kill at Y ≤ 640) | Jim's Drone aiStyle 195 (the id row exists, the AI does not) |
| GOING_OLDSCHOOL | "Do" | `Main.cs:63018` → `AchievementsHelper.cs:384` | classic title-screen sun/moon grab |

**Reachable-but-gated (the wire exists, the gate cannot pass in play):**
REAL_ESTATE_AGENT 17 + ALL_TOWN_SLIMES 26 (the DOC-SWEEP roster check —
4 town kinds + 8 slime kinds missing; the W4-c36 re-probe corrected the
Bartender: 550 IS ported), BOOK_WORM 29 (the hook fires; the S1-F-15
tier-4 denominator bug is FIXED on main — the residual gate is the
town/critter provider fold, c37's task; see the re-probe below).

Everything else in the 86 has a live notify site on this branch or f30's
(the full wired table: the c36 site table above + f30's tail census).

## W4-c36 RE-PROBE — the 20-row gap list vs the CURRENT pushed tips (2026-11-21)

Method: `git fetch origin --prune`, then per-lane
`git merge-base --is-ancestor origin/task/W4-<lane> origin/main` + a symbol
grep of each tip's diff. **Result: every gap's owning system is still
BRANCH-ONLY — none is in `origin/main` (`a725acb04`, this branch's base).**
No gap can be wired in this tree without stacking unmerged lanes (banned by
AGENTS.md), so this pass records the exact post-merge seams instead. The
band lanes the lead expected to be porting are NOT porting yet: **d36
`65e481f96` and d41 `4ed22f0b3` are CLAIM-only commits; d39/d43 tips equal
main (zero port commits pushed); d42 `88df58090` is a non-compiling WIP
checkpoint** (its own message: "Does not compile", superseded by the
research/108 union plan). No `d41r/d42r/d43r/d36r/d39r` or `f30b` remote
branches exist.

### Per-row re-probe (tip hashes as of this fetch)

| Row | Cond | Owning lane / tip | In main? | Post-merge seam |
| --- | --- | --- | --- | --- |
| IT_CAN_TALK | 18 | d36 `65e481f96` (CLAIM only); d119 `04d7f9fd2` adds the `unlocked_truffle_spawn` housing gate but NO Truffle kind and NO notify 18 | no | the arrival switch (`world_sim/npcs.rs:4949`'s cited comment) — `net_id() == 160` arm → 18 + `unlockedTruffleSpawn` (`WorldGen.cs:5579-5583`) |
| TURN_GNOME_TO_STATUE | 24 | none — d40's 473 row is BigMimicCorruption, NOT the gnome | no | gnome AI (`NPC.cs:55896`) unowned |
| QUEEN_MACHINE | 31 | f30 `f8cef8cb8` (seed gate + mecha_mayhem machine; 31 CITED) | no | Mechdusa summon 5334 + combined-boss death (`NPC.cs:78857-78869`) unowned |
| FEAR_THE_SUN | 33 | f50 `ff56e6e51` does NOT port `UpdateSunScorch` (grep: no sun_scorch writer); the `vampire_burning_in_sunlight` flag still has no play writer (`armor_sets.rs:345`) | no | `Player.cs:28647` unowned |
| ITS_SHALING_OUTSIDE | 34 | d15 ports `SpawnFallingObjects`' starfall loop ONLY — the boulder-rain arm folds false (its own comment); f45 carries the header fields | no | `WorldGen.cs:71694` boulder-rain start unowned |
| EXTRA_LIFE | 35 | none (all-tip grep: no `surviveHardcoreDeath`/`HardcoreDeathPenalty`) | no | `Player.cs:38782` unowned |
| GRAVE_MISTAKE | 36 | none — `death.rs` tombstones are the falling visual, not striking projectiles | no | `Player.cs:20473`/`:38836` unowned |
| FORTUNE_FAVORS_THE_BOULD | 39 | none (all-tip grep: no rainbow-boulder music counter) | no | `Main.cs:61828` unowned |
| TRAINING_DAY | 40 | **a6 `8ed96ee5d` ports `use_combat_book` 4382/5336 + `world.combat_book_volume_two_was_used`** — but fires NO notify 40 | no | after a6+f30 merge: the `update_player` armor tail (f30's 38/41 seam) reads `combat_book_was_used && combat_book_volume_two_was_used` → 40 (`Player.cs:25697`) |
| NEW_DIGS | 43 | **f47 `507a0da5c` ports AI_007's shimmer state 25 completion + sets `out.shimmer_town_progression`** (`npc_ai_town.rs:1821`) — but NO consumer fires 43 | no | after f47 merge: the npcs-pass drain of `shimmer_town_progression` → `notify_progression_net(43)` (`NPC.cs:53273`) |
| MY_PEOPLE_NEED_ME | 44 | none — no lane ports `BordersMovement`'s remix top-world kill | no | `Player.cs:24076` unowned |
| TRASH_COMPACTOR | 45 | **c24 `e2e50a427` ports the Journey UI + the `register_sacrifice` consumer** (`journey_clicks.rs`) — no notify 45; f30 `f8cef8cb8` has the PREP (`LastEditId` + `count_fully_researched` + `check_research_achievement`) | no | after c24+f30 merge: the sacrifice click → `check_research_achievement` → 45 (`AchievementsHelper.cs:387-399`); the enter-world forced call (`Player.cs:325`) + the per-tick `:25703` |
| INTERDIMENSIONAL_RECYCLING | 46 | none — `npc_chat.rs` has the cola REMINDER line only; no gift path (all-tip grep negative) | no | `Main.cs:40294` unowned |
| SPICY_LICKS (partial) | 37 | f30 wires the 4715 arm; NO lane ports `ItemCheck_PlayInstruments` (a1 grep negative) — the 1305/4057/4372 arms stay cited | no | `Player.cs:46574`/`:46588` unowned (the instruments use system) |
| VEHICULAR_MANSLAUGHTER | "Hit" | none (all-tip grep: no `GetMinecartDamage`); the prerequisites ARE in main (`mount::is_cart`, `damage_var`, `minecart.rs`) | no | `Player.cs:27585-27604` unowned |
| PET_THE_PET | "Do" | none — the petting stop is cited-unported in `player.rs:9178` | no | `Player.cs:33350`/`:33397` unowned |
| FIND_A_FAIRY | "Do" | d42 `88df58090` WIP adds the FairyCritter enum variants (non-compiling, superseded) | no | fairy kinds + the sight seam (`NPC.cs:48647`, HandleSpecialEvent 22) |
| PURIFY_ENTIRE_WORLD | "Do" | f22 `e7756fa70` does NOT port the Dryad status dialog (grep negative) | no | `Main.cs:40276` unowned |
| TO_INFINITY_AND_BEYOND | "Do" | **d20 `13de9ccf6` ports AI_195 Jim's Drone + the altitude/held-5451 kill gates** — but NO achievement wire, and main's `proj_data` has no 1020 row (the spawn drops) | no | after d20 merge: the 1020 kill arm → `complete_condition("TO_INFINITY_AND_BEYOND", "Do")` when `pos.y - height <= 640 && owner == myPlayer` (`Projectile.cs:70857-70862`) |
| GOING_OLDSCHOOL | "Do" | none — the classic title screen is unported on every tip | no | `Main.cs:63018` → `AchievementsHelper.cs:382-385` unowned |

**Net: 0 of 20 wireable in-tree today; 4 rows (40, 43, 45, TO_INFINITY) have
their owning system + the exact notify seam identified on pushed tips** —
c36 wires them the moment those lanes land in main (the next sync's
re-probe). The other 16 stay cited.

### (2) The 29 / S1-F-15 tier-bug census — VERIFIED FIXED, no bug to fix

- Progression 29 = **BOOK_WORM** (`AchievementInitializer.cs:378-380`,
  `Cond::Progression(29)` in `achievements_data.rs:1052-1057`). There is NO
  "bullseye" achievement in the C# initializer (grep negative) — the lead's
  note conflated rows; nothing to re-tier.
- The tier condition is NOT wrong: `get_unlock_state_by_kill_count`
  (`tr-core/src/bestiary.rs:31-54`) is line-for-line
  `CommonEnemyUICollectionInfoProvider.cs:50-76` (quick>0 → 4, ≥full → 4,
  ≥half → 3, ≥fifth → 2, ≥1 → 1, else 0).
- The old denominator bug (0 → tier 4 at zero kills) is **already fixed on
  main by S1-F-15**: `kills_needed_for` → `kills_to_banner_item`'s default
  arm returns 50; `the_family_banner_chain` pins bannerless ids at 50 and
  `completion_percent(&fresh) == 0.0` is pinned (`ui/bestiary.rs:2311`).
  `banner_to_item` carries the full band table (deviation #15 resolved).
- The RESIDUAL gate is the provider fold: main's `unlock_state` runs every
  entry through the Common kill-ladder, so a town NPC's first kill reads
  portrait-tier where the C# needs a CHAT (`TownNPCUICollectionInfoProvider`)
  and a critter needs a SIGHT. **c37 `3e138525e` owns exactly this** (its
  `bestiary_data::Gate` enum: Common/Town/Critter/Highest + the
  `WorldBestiary`-based `completion_percent`). Not c36's to fix — recorded
  as the dedupe below.

### (3) The 17/26 roster no-op wires — automatic activation VERIFIED

`check_real_estate_and_town_slimes` collects `n.kind.net_id()`, and
`NpcKind::net_id()` (`npc/data.rs:1186`) resolves through
`bestiary::entry_for_npc` → `ENTRIES` — **no hardcoded id list anywhere in
the scan**. `kind_from_net_id` (`npc/data.rs:3996`) likewise scans
`ENTRIES`. So when d36/d39/d41/d43 land their kinds + bestiary rows, the
roster check activates with ZERO c36 edits. Re-probe corrections landed
this pass: **Bartender 550 IS ported** (`DD2Bartender`, `ENTRIES` row +
`dd2_pool_id`), so the missing roster is FOUR town kinds (160/178/209/229)
+ the eight slimes — the witness now carries the Bartender and pins
`net_id() == 550`. The bands are not porting yet (claim-only tips above).

### (4) f30 co-lane dedupe table (f30 `f8cef8cb8`, no f30b branch exists)

f30's tail census (21/37-partial/38/41 WIRED, 45 PREP, 31/33/34/35/36/39/
40/43/44/46 CITED) vs this gap list — the overlap is ENTIRELY inside f30's
reserved tail, no double-wire:

| Gap-list row | f30 verdict | Owner after merge |
| --- | --- | --- |
| 31, 33, 34, 35, 36, 39, 44, 46 | CITED (same missing systems) | f30's citation stands; c36 does not re-cite |
| 40 | CITED (vol-2 flag never sets) | **a6 sets the flag** — the armor-tail wire is f30's seam (its 38/41 arms), c36 supplies the re-probe fact |
| 43 | CITED (state 25 skip) | **f47 ports the transform + flag** — the drain wire lands with f47's merge |
| 45 | PREP (`check_research_achievement`) | **c24 ports the consumer** — the click wire joins f30's prep at merge |
| 37 partial (1305/4057/4372) | CITED (4715 wired by f30) | unowned — the instruments use system has no lane |
| 18, 24, "Hit", "Do"×5, 17/26, 29 | outside f30's tail | c36 (this list) |

Sibling-lane reads: **e22 `aae211f16`** adds the 180-condition pin test
(the "S — data test pins every row's condition count" task seed below is
DONE on its branch); **c37 `3e138525e`** owns the provider-enum fold (the
"M — replace quick_unlock with a per-entry provider enum" seed) AND the
`try_grant_bestiary_100` signature move to `WorldBestiary` — c36's
`KillTracker`-based call sites (`netglue.rs:2987`, `netmodules.rs:467`)
will need the one-line rebase at c37's merge, no semantic conflict (the
gate predicate is identical).

## Task seeds

- ~~**S** — Model `NPCKilledCondition.CreateMany` faithfully~~ — **done (S1-E-18)**.
- ~~**S** — Call `achievements::save` from the autosave/F9 pass~~ — **done (S1-E-18)** (save on completion + `save_world`, path-bound bootstrap).
- **M** — Continue wiring notify sites for progression/custom rows as owning systems land (remaining: the 20-row gap list, re-probed below — every owning system is still branch-only; 17 partial, 18; 19/20 + nurse `Pay` landed). Partially done (S1-E-18). Dep: owning systems merging to main.
- ~~**S** — Make `kills_needed_for` fall back to
  `ItemID_Sets_DefaultKillsForBannerNeeded` (50) instead of 0, and pin it with a
  regression test that a fresh tracker leaves a non-quick entry at
  `NotKnownAtAll`. Fixes #4.~~ **DONE (S1-F-15, re-verified W4-c36)** —
  `the_family_banner_chain` + `kills_needed_for_honors_kills_to_banner_overrides`.
- **M** — Replace the `quick_unlock` bool with a per-entry provider enum
  (`CommonEnemy` / `TownNPC` / `Critter` / `HighestOfMultiple` / `GoldCritter` /
  `Salamander`) and record sightings and chats. Fixes #5, #6. Dep: the S task
  above for the denominator. **OWNED by c37 `3e138525e`** (W4-c36 re-probe:
  its `bestiary_data::Gate` enum + the `WorldBestiary` completion move).
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
- **S** — Reconcile `ENTRIES` (249) with `BESTIARY_NPC_SHEET_IDS` (181) and add
  a test that every sheet slot has an entry and vice versa. Fixes #13.
- **DONE (S1-I-5, 2026-11-16)** — `NetBestiaryModule` is ported: the host broadcasts
  `SerializeKillCount` on a kill (`NPCKillsTracker.cs:23-33`) and syncs the counts on
  player join (`:98-107`), with the `97`/`98` achievement wires
  (`MessageBuffer.cs:3558-3569`). Fixes #16. The `Sight`/`Chat` halves remain open.
- **S** — Add a data test that pins every Rust row's condition count against
  `AchievementInitializer.cs` (137 rows / 180 conditions) so the `CreateMany`
  class of bug cannot return. Dep: the first S task. **DONE on e22
  `aae211f16`** (W4-c36 re-probe: its `CONDITION_COUNTS` transcription test).
