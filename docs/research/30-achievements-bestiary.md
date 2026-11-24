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
- `crates/tr-app/src/ui/achievements.rs::tests`: the panel/list/back/filter/
  sort/row-hit/scroll/icon-frame layout tests (M6-ACH-2), d81's five pure-half
  witnesses (row height/icon offsets/icon frame/description clamp/tracker
  geometry+colours, `6d700ed7`), and W4-g53's three new ones
  (`the_scrollbar_rides_the_lists_right_edge`,
  `the_content_height_and_scroll_range_are_the_ui_list_walk`,
  `the_scrollbar_press_grabs_the_handle_or_jumps_the_track`) plus the two
  re-derived onto the padded layout (`the_list_and_rows_follow_the_layout_constants`,
  `the_scroll_range_is_the_list_overflow`). App-level W4-g53 witnesses in
  `app/clicks.rs::tests`: `g53_the_achievements_scrollbar_grabs_drags_and_jumps`,
  `g53_the_filter_toggle_sounds_and_clamps_the_scroll`,
  `g53_the_wheel_walks_120_px_per_detent_on_the_list`,
  `g53_the_back_hover_sounds_once_per_edge` — all written not run (code-only).
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
- `crates/tr-app/src/achievements_data.rs::tests` (2, W4-e22):
  `every_rows_conditions_match_the_initializer_verbatim` (a second verbatim
  transcription of `AchievementInitializer.cs:16-443` compared entry-by-entry:
  row order, per-row count, kind + arguments; 137 rows / 180 conditions) and
  `the_create_many_rows_stay_expanded` (the 3/4/24/15 shapes, no empty row).
- No test covers: the `notify` sites' reachability, the achievement save being
  called, or the bestiary's drop lines.

## Deviations & risks

| # | Deviation | Severity | Admitted? |
| --- | --- | --- | --- |
| 1 | `CreateMany` rows expanded to 1:1 (S1-E-18): all 180 conditions present. | behavioural | Resolved (S1-E-18) |
| 2 | Save is now wired (S1-E-18): path bound at bootstrap/world swap, saved on completion and from `save_world`; headless apps have no bound path. Format remains the port's plain-text `achievements.trdat` (not vanilla Rijndael+BSON). | behavioural / interop | Partial — call sites wired, format differs by design |
| 3 | Notify wrappers and additional sites landed (S1-E-18), but many progression/custom rows remain unreachable because their owning systems/paths are not ported (e.g. 17 partial, 18, and the 20-row gap list below). Progression 29's hook FIRES (S1-E-18p29); the old S1-F-15 denominator blocker is fixed (see #4). | behavioural | Partly — expanded, not complete |
| 4 | ~~`kills_needed_for` returns 0 for net ids outside the 11-pair banner table; 0 means "≥ full", so 164 of 172 entries report tier 4 with zero kills.~~ **FIXED (S1-F-15, re-verified W4-c36)** — the default arm returns 50 (`ItemID_Sets_DefaultKillsForBannerNeeded`), pinned by `the_family_banner_chain` + the override test. | behavioural | Resolved (S1-F-15) |
| 5 | `quick_unlock` on 24 entries (W4-c36 re-probe count) instead of every boss + the override rows; the town `Guide` no longer carries it. The salamander/HighestOfMultiple overrides ride c37's provider enum. | behavioural | Partly — c37 owns the provider fold |
| 6 | The town-NPC (chat) and critter (sight) unlock providers are absent, and the `sights`/`chats` sets are never recorded. | behavioural | Yes — `tr-core/src/bestiary.rs:124-130`, ANALYSIS "the sightings/chats stay empty until the recording behaviour lands" |
| 3 | Notify wrappers and additional sites landed (S1-E-18), but many progression/custom rows remain unreachable because their owning systems/paths are not ported (e.g. 17 partial, 18, 19/20, nurse `Pay`, and most other progression/custom flags). Progression 29 is explicitly blocked on S1-F-15. | behavioural | Partly — expanded, not complete |
| 4 | ~~`kills_needed_for` returns 0 for net ids outside the 11-pair banner table; 0 means "≥ full", so 164 of 172 entries report tier 4 (`CanShowDropsWithDropRates`) with zero kills. The C# default is 50.~~ **FIXED (S1-F-15 + W4-c37)** — the full banner chain (`npc_to_banner`/`banner_to_item`/`kills_to_banner_item`) with the 50 default; the fresh-world witness pins every roster entry at `NotKnownAtAll`. | behavioural | Resolved |
| 5 | ~~`quick_unlock` on 5 entries instead of every boss + the override rows; the town NPC `Guide` wrongly carries it (the C# uses the chat provider).~~ **FIXED (W4-c37)** — the flag census matches the `SetDefaults` boss rows + `ShouldBeCountedAsBossForBestiary`; the per-entry providers (`bestiary_data::gate`) carry the town chat / critter sight / DD2 quick rows / boss-credit folds / Skeletron Highest-of-two. | behavioural | Resolved |
| 6 | The town-NPC (chat) and critter (sight) unlock providers are absent, and the `sights`/`chats` sets are never recorded. **Providers DONE (W4-c37)** — `Gate::Town`/`Gate::Critter` read the trackers; the RECORDING (`ScanWorldForFinds:102-130` + the chat hook) stays open. | behavioural | Partly — providers landed, recording still state-only (`tr-core/src/bestiary.rs:124-130`) |
| 7 | `ReportDroprates` / the drop-rate tables are absent: the bestiary's drop list is a 4-row hand-written table, so the tier-3/tier-4 drop lines are empty for 168 of 172 entries. | behavioural / pixel | Yes — ANALYSIS.md:2969 "not ported — M4-BEST-1 territory" |
| 8 | ~~`TryGrantingBestiary100PercentAchievement` (→ progression event 29 → `BOOK_WORM`) never fires.~~ **FIRES (S1-E-18p29)** — the death-seam hook (`world_sim/npcs.rs:5274` → `try_grant_bestiary_100_net`) + the `98` MP wire; the gate is the completion fraction (see #12). | behavioural | Resolved (S1-E-18p29) |
| 9 | The achievement save format is plain text (`achievements.trdat`), not the vanilla Rijndael-CBC+BSON `"RELOGIC-TERRARIA"`. | behavioural (interop) | Yes — `achievements.rs:390-394` |
| 10 | ~~The unlock popup, `AchievementAdvisor` and the chat tag handler are missing.~~ **Popup DONE (S1-E-26 wave A)** — `ui/notifications.rs` + the drain seam; the advisor and the chat tag's snippet rendering remain open. | cosmetic / behavioural | Yes — ANALYSIS "The achievement unlock popup (S1-E-26 wave A)" |
| 11 | ~~`notify` returns only the last completed achievement; the C# fires `OnCompleted` (and a save) for every one.~~ **FIXED (S1-E-26 wave A)** — `notify_with` queues every completing row into `completed_events` (save + event per row); the return value keeps the last-name shape for the existing callers. Pinned by `one_event_completing_two_rows_queues_both`. | behavioural (minor) | Resolved |
| 12 | `completion_percent` divides by the port's 249-entry roster (W4-c36 re-probe), not the C#'s full roster; the #4 tier-4 inflation is FIXED so a fresh world reads 0% (pinned). The residual early-100% risk is the flat kill-ladder fold for town/critter entries (one kill = portrait tier where the C# needs a chat/sight) — c37's provider enum owns it. The Zoologist shop (`Chest.cs:3265`) reads the fraction. | behavioural | Partly — denominator documented, provider fold c37's |
| 13 | `ENTRIES` (249) vs `BESTIARY_NPC_SHEET_IDS` (181) — W4-c36 re-probe: the entries now EXCEED the sheet (the DD2/moon families carry rows without sheet slots); the old "2 sheet slots have no entry" skew inverted. | cosmetic | Partly — BUG-15 removed the literal drift, not the skew |
| 12 | `completion_percent` divides by the port's 172-entry roster, not the C#'s full roster; ~~combined with #4 it reads ~95% at start~~ — with #4 fixed it reads 0 at start (W4-c37's fresh-world witness); the roster-denominator half stays until the roster waves land. | behavioural | Partly — the start-value bug resolved (W4-c37), the denominator is the port roster |
| 13 | `ENTRIES` (172) vs `BESTIARY_NPC_SHEET_IDS` (174) — 2 sheet slots have no entry row. | cosmetic | Partly — BUG-15 removed the literal drift, not the skew |
| 14 | The kill-count and sight/chat sets iterate in sorted order vs the C#'s unspecified `Dictionary`/`HashSet` order — deterministic, but a different byte order in the save. | determinism | Yes — documented in `tr-core/src/bestiary.rs:96-100, 132-136` |
| 15 | ~~`banner_to_item` keeps only 3 bands (276+/88+/default).~~ **FIXED (re-verified W4-c36)** — the port now carries the 292/291/290/289 specials, the 276+/274-275/273/267+/257+/252+/251/249+/186+/88+ bands and the default (`tr-core/src/bestiary.rs:1043-1087`, the full `BannerSystem.cs:341-401` shape). | behavioural (latent) | Resolved |
| 16 | ~~No MP sync of the bestiary kill counts (NetBestiaryModule).~~ **PORTED (S1-I-5)** - module 4 carries the kill count, the 97/98 wires carry the credit and the progression id; the Sight/Chat halves are still open. | behavioural (MP) | Yes - pp/netmodules.rs::BestiaryKill, pp/netglue.rs |
| 17 | `Player.HasAchievement`, `Main.achievementTracker` and a `BossAchievement` type **do not exist** in the 1.4.5.8 decompile. The boss kill requirement is the bestiary's banner→`KillsToBanner` chain, and the achievement query is `Main.Achievements.GetAchievement(name).IsCompleted`. | n/a | n/a |
| 18 | ~~`Achievement.completed` (flag) vs `is_completed()` (derived) disagree for a zero-condition row before and after a load.~~ Moot after S1-E-18/W4-e22: no shipped row has zero conditions (`the_create_many_rows_stay_expanded` pins it); the flag/derived skew is unreachable through the registry. | behavioural (minor) | Resolved (W4-e22 re-verification) |
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
  rainbow-slime release (`WorldItem.cs:1742`). UNREACHABLE in play ON
  THIS BASE: Truffle 160, Steampunker 178, Cyborg 209, Pirate 229 (FOUR
  kinds — the W4-c36 re-probe: Bartender 550 IS ported, `DD2Bartender`)
  and the eight town slimes 670/678-684 have no NpcKind rows.
  **NOTIFY-SYNC (4): the four town kinds are on d120 `4b17a2cf1`
  (160/178/209) + d37 `08f7d25cf` (229) — the roster arm activates at
  their merge; the slimes stay unported on every tip.**
- Progression 18 (Truffle arrival, `WorldGen.cs:5580`): the Truffle kind
  is on d120 `4b17a2cf1` (NOTIFY-SYNC (4)) — the notify hunk is published
  there; the arrival-ladder entry is d120/f43's lane.
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

> **SUPERSEDED by the RETIRE (3) FINAL verdict below** — f33's tip moved
> past `333479e0` (the probe this table read): `8a000407` landed the FULL
> `ItemCheck_UseTorchGodsFavor` body + the `HandleSpecialEvent(24)` named
> dispatcher, and f33's retire closeout (research/93a §3b/§8, tip
> `83387628b`) adjudicates **keep-f33-full-body, DROP c36's reduced copy
> and a6's arm**. The "no keep-one deletion needed" verdict below is
> stale; the exact c36 drop-hunk is published in RETIRE (3).

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
gap list, after c36+f30; SUPERSEDED by the RETIRE (2) FINAL census below —
35 retired on c46, the 24/22 kinds landed on d42):**

| Row | Cond | C# site | Missing owning system |
| --- | --- | --- | --- |
| IT_CAN_TALK | 18 | `WorldGen.cs:5580` | ~~Truffle kind 160~~ **kind LANDED on d120 `4b17a2cf1`** (proto 242, bestiary net_id 160) — the notify-18 wire + the arrival-ladder entry stay open (NOTIFY-SYNC (4)) |
| TURN_GNOME_TO_STATUE | 24 | `NPC.cs:55896` | ~~gnome AI~~ **kind LANDED on d42 `2d7ac08c3`** (Gnome 624, `npc.rs:1250` + bestiary `:2398`) — the AI rides `update_zombie` (stand-in, `npc.rs:6715`), the statue-conversion + notify stay unowned (RETIRE FINAL) |
| QUEEN_MACHINE | 31 | `NPC.cs:78866` | Mechdusa summon 5334 + combined-boss death (f30 cited; the seed gate landed) |
| FEAR_THE_SUN | 33 | `Player.cs:28647` | vampire sun-scorch (f30 cited) |
| ITS_SHALING_OUTSIDE | 34 | `WorldGen.cs:71694` | boulder rain (f30 cited) |
| EXTRA_LIFE | 35 | `Player.cs:38782` | ~~hardcore death penalty (f30 cited)~~ **RETIRED — c46 WIRES it live** (`4445a1459`, tip `2802ed5b9`: `death.rs:527` `notify_progression_net(35)` on the survive arm, wired at the KillMe difficulty-2 gate `:437-438`) — RETIRE FINAL |
| GRAVE_MISTAKE | 36 | `Player.cs:20473`/`:38836` | striking gravestones + MP teammate deaths (f30 cited) |
| FORTUNE_FAVORS_THE_BOULD | 39 | `Main.cs:61828` | rainbow boulder draw (f30 cited) |
| TRAINING_DAY | 40 | `Player.cs:25697` | ~~combat book vol. 2 use 5336 (f30 cited)~~ **g90b WIRES it live** (`0316b4446`, tip `b3b82d071`) — see the NOTIFY-SYNC keep-one below |
| NEW_DIGS | 43 | `NPC.cs:53273` | ~~town shimmer transform (f30 cited)~~ **f47 WIRES it live** (`6f126bd9e`, tip `a2561d6d9`) — see NOTIFY-SYNC (2) |
| MY_PEOPLE_NEED_ME | 44 | `Player.cs:24076` | remix top-world death (f30 cited) |
| TRASH_COMPACTOR | 45 | `AchievementsHelper.cs:396` | Journey sacrifice UI (f30 PREP) |
| INTERDIMENSIONAL_RECYCLING | 46 | `Main.cs:40294` | Dryad Joja-cola gift (f30 cited) |
| SPICY_LICKS (partial) | 37 | `Player.cs:46574`/`:46588` | The Axe 1305 / Carbon+Ivy 4057/4372 chord arms (the 4715 arm wired by f30) |
| VEHICULAR_MANSLAUGHTER | "Hit" | `Player.cs:27602` | minecart NPC-collision damage (`GetMinecartDamage` block) |
| PET_THE_PET | "Do" | `Player.cs:33350`/`:33397` | petting system |
| FIND_A_FAIRY | "Do" | `NPC.cs:48647` | ~~fairy kind~~ **kinds LANDED on d42 `2d7ac08c3`** (FairyCritterPink/Green/Blue 583-585, bestiary `:2203-2224`) — the sight seam (HandleSpecialEvent 22) stays unowned on every tip (RETIRE FINAL) |
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
| IT_CAN_TALK | 18 | ~~d36 `65e481f96` (CLAIM only)~~ **NOTIFY-SYNC (4): the Truffle kind IS ported — on d120 `4b17a2cf1`, NOT d36** (kinds A `2891b0bd8`: SantaClaus 142/Truffle 160/Steampunker 178, proto 241-243; Cyborg 209 proto 244 in kinds B; bestiary net ids vanilla). d36 stays CLAIM-only; d119 `04d7f9fd2`'s `unlocked_truffle_spawn` housing gate reads the f45 World latch (`6352d3840`, `world.rs:797`). STILL NO notify 18 on any tip (d120's arrival comment `npcs.rs:5038` still says "waits on a ported Truffle kind" — stale, the kind is on its own tip) | no | the arrival switch (`world_sim/npcs.rs:4949`'s cited comment) — `net_id() == 160` arm → 18 + `unlockedTruffleSpawn` (`WorldGen.cs:5579-5583`); the exact hunk is published in NOTIFY-SYNC (4); the arrival LADDER entry (Truffle's `condition_holds` arm + LADDER row) is d120/f43's lane — the kind alone never arrives |
| TURN_GNOME_TO_STATUE | 24 | ~~none — d40's 473 row is BigMimicCorruption, NOT the gnome~~ **RETIRE FINAL: the Gnome kind 624 IS on d42 `2d7ac08c3`** (`npc.rs:1247-1250`, bestiary `:2398-2403`; e26 `048cca6cf` ports the Garden Gnome ITEM 4609 → tile 567, NOT the NPC conversion) | no | the statue-conversion AI (`NPC.cs:55896`) unowned — d42's Gnome rides `update_zombie` (`npc.rs:6715`, a stand-in); the conversion + notify 24 stay the wiring lane's |
| QUEEN_MACHINE | 31 | f30 `f8cef8cb8` (seed gate + mecha_mayhem machine; 31 CITED) | no | Mechdusa summon 5334 + combined-boss death (`NPC.cs:78857-78869`) unowned |
| FEAR_THE_SUN | 33 | f50 `ff56e6e51` does NOT port `UpdateSunScorch` (grep: no sun_scorch writer); the `vampire_burning_in_sunlight` flag still has no play writer (`armor_sets.rs:345`) | no | `Player.cs:28647` unowned |
| ITS_SHALING_OUTSIDE | 34 | d15 ports `SpawnFallingObjects`' starfall loop ONLY — the boulder-rain arm folds false (its own comment); f45 carries the header fields | no | `WorldGen.cs:71694` boulder-rain start unowned |
| EXTRA_LIFE | 35 | ~~none (all-tip grep: no `surviveHardcoreDeath`/`HardcoreDeathPenalty`)~~ **SUPERSEDED — c46 `2802ed5b9` WIRES 35 LIVE**: final-phase-1 `4445a1459` ports `HardcoreDeathPenalty` (`Player.cs:38775-38787`, the −60 stat half `player.rs:3851`) + `KillMeForGood` (`:38789-38805`), wired at KillMe's `:38864-38866` difficulty-2 gate AFTER DropItems (`death.rs:437-438`); the survive arm fires `notify_progression_net(35)` (`death.rs:525-528`, `:38782`) | no | NONE — c46 owns it; f30's citation retires at merge; c36 owes nothing |
| GRAVE_MISTAKE | 36 | none — `death.rs` tombstones are the falling visual, not striking projectiles | no | `Player.cs:20473`/`:38836` unowned |
| FORTUNE_FAVORS_THE_BOULD | 39 | none (all-tip grep: no rainbow-boulder music counter) | no | `Main.cs:61828` unowned |
| TRAINING_DAY | 40 | ~~a6 `8ed96ee5d` … fires NO notify 40~~ **SUPERSEDED — g90b `b3b82d071` WIRES 40 LIVE**: `world_sim/player.rs:1625-1627` (`drop_backlog` tail) reads `combat_book_was_used && combat_book_volume_two_was_used` → `notify_progression_net(40)` every tick the pair stands (`Player.cs:25695-25698`, idempotent — `notify_with` skips completed rows). g90's lane-7 `0316b4446` ports `use_combat_book` 4382/5336 itself (`editing.rs`, both world flags). **f12 `61724db38` has NO 40 wire** (grep: only the `achievements_data.rs` row + the `npc_ai_town` ladder's flag READS — no notify). **c36's owed armor-tail wire is RETIRED** (keep-one: g90b-live) | no | NONE — g90b owns it; at the a6×g90 merge keep ONE `use_combat_book` (both write the same flags; g90's carries the notify's producers) and ONE notify site (g90's `:1625`); f30's cited comment (`world_sim/player.rs:1631-1635`) retires with it |
| NEW_DIGS | 43 | ~~f47 `507a0da5c` … NO consumer fires 43~~ **SUPERSEDED — f47 `a2561d6d9` WIRES 43 LIVE**: doc-sweep `6f126bd9e` adds `npc_ai_town::town_shimmer_step` (the state-25 emergence machine at the NPC's slot turn, INSTEAD of the dry walk — the C# `:53277` return) + the post-pass latch drain `if shimmer_progression43 { let _ = app.notify_progression_net(43); }` in `world_sim/npcs.rs` (`update_npcs` tail, ahead of the GetShimmered drain). The `TownOutputs.shimmer_town_progression` field (`npc_ai_town.rs:735`, set at `:1831` on the flipped-variation-1 completion, returned by `town_shimmer_step` at `:1966`) IS consumed on f47's own tip | no | NONE on f47's seam — the b3 merge folds it into the adapter's `TownOutputs` drain (f47's research/121 MERGE CONTRACT: keep ONE driver); the exact 2-hunk fold for b3 is published in NOTIFY-SYNC (2) below |
| MY_PEOPLE_NEED_ME | 44 | none — no lane ports `BordersMovement`'s remix top-world kill | no | `Player.cs:24076` unowned |
| TRASH_COMPACTOR | 45 | **c24 `e2e50a427` ports the Journey UI + the `register_sacrifice` consumer** (`journey_clicks.rs`) — no notify 45; f30 `f8cef8cb8` has the PREP (`LastEditId` + `count_fully_researched` + `check_research_achievement`) | no | after c24+f30 merge: the sacrifice click → `check_research_achievement` → 45 (`AchievementsHelper.cs:387-399`); the enter-world forced call (`Player.cs:325`) + the per-tick `:25703` |
| INTERDIMENSIONAL_RECYCLING | 46 | none — `npc_chat.rs` has the cola REMINDER line only; no gift path (all-tip grep negative) | no | `Main.cs:40294` unowned |
| SPICY_LICKS (partial) | 37 | f30 wires the 4715 arm; NO lane ports `ItemCheck_PlayInstruments` (a1 grep negative) — the 1305/4057/4372 arms stay cited | no | `Player.cs:46574`/`:46588` unowned (the instruments use system) |
| VEHICULAR_MANSLAUGHTER | "Hit" | none (all-tip grep: no `GetMinecartDamage`); the prerequisites ARE in main (`mount::is_cart`, `damage_var`, `minecart.rs`) | no | `Player.cs:27585-27604` unowned |
| PET_THE_PET | "Do" | none — the petting stop is cited-unported in `player.rs:9178` | no | `Player.cs:33350`/`:33397` unowned |
| FIND_A_FAIRY | "Do" | ~~d42 `88df58090` WIP adds the FairyCritter enum variants (non-compiling, superseded)~~ **RETIRE FINAL: d42's roster join COMPILES the kinds** — `2d7ac08c3` carries FairyCritterPink/Green/Blue 583-585 (`npc/data.rs:601-603` cs ids 583-585, bestiary `:2203-2224` net ids 583/584/585) | no | the sight seam (`NPC.cs:48647`, HandleSpecialEvent 22 → FIND_A_FAIRY "Do") unowned on every tip — no `handle_special_event(22)`/`complete_condition("FIND_A_FAIRY"` anywhere (all-tip grep); the kinds alone never sight |
| PURIFY_ENTIRE_WORLD | "Do" | f22 `e7756fa70` does NOT port the Dryad status dialog (grep negative) | no | `Main.cs:40276` unowned |
| TO_INFINITY_AND_BEYOND | "Do" | **d20 `13de9ccf6` ports AI_195 Jim's Drone + the altitude/held-5451 kill gates** (`projectile_ai_misc.rs:1092-1110` — the `pos.y - h <= 640` kill at `:1105-1109`) — but NO achievement wire; the 1020 `proj_data` row IS on d20 (`data.rs:3633`), NOT on main (the spawn drops there) | no | the exact kill-arm hunk for d20b is PUBLISHED in NOTIFY-SYNC (3) below — the `Kill()` 1020 arm (`Projectile.cs:70857-70862`) latches a `wants_to_infinity` intent at the death hook, the world-sim drain fires `complete_condition("TO_INFINITY_AND_BEYOND", "Do")` (the c36 lava-fishing pattern) |
| GOING_OLDSCHOOL | "Do" | none — the classic title screen is unported on every tip | no | `Main.cs:63018` → `AchievementsHelper.cs:382-385` unowned |

**Net: 0 of 20 wireable in-tree today; 4 rows (40, 43, 45, TO_INFINITY) have
their owning system + the exact notify seam identified on pushed tips** —
c36 wires them the moment those lanes land in main (the next sync's
re-probe). The other 16 stay cited.

**NOTIFY-SYNC refresh (2026-11-21, second pass — the tips moved):**
40 and 43 are now WIRED LIVE on their owning tips (g90b `b3b82d071`,
f47 `a2561d6d9`) — c36's owed rows RETIRED, nothing to wire at merge
(the keep-one contracts: NOTIFY-SYNC (1)/(2)). TO_INFINITY's exact
4-hunk kill-arm is published for d20b (NOTIFY-SYNC (3)); 18's Truffle
kind landed on d120 and its arrival hunk is published (NOTIFY-SYNC (4)).
45 stays the c24+f30 merge join. **Revised net: 0 of 20 wireable
in-tree; 2 rows (40/43) fully covered on sibling tips, 3 rows
(TO_INFINITY/18/45) carry exact published hunks or merge joins; the
roster 17 activates at the d120/d37 merge (4 kinds landed), 26 waits
for the slime kinds (unported on every tip).** (The RETIRE (2) FINAL
census below re-sweeps all 357 tips: 35 joins 40/43 as retired — the
final no-path list is 17 rows.)

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
`net_id() == 550`. ~~The bands are not porting yet (claim-only tips
above).~~ **NOTIFY-SYNC (4) UPDATE: all four town kinds ARE on pushed
tips now** — see the NOTIFY-SYNC (4) section below (d120 `4b17a2cf1`
lands 160/178/209, d37 `08f7d25cf` lands 229; the eight slimes stay
unported on every tip).

### (4) f30 co-lane dedupe table (f30 `f8cef8cb8`, no f30b branch exists)

f30's tail census (21/37-partial/38/41 WIRED, 45 PREP, 31/33/34/35/36/39/
40/43/44/46 CITED) vs this gap list — the overlap is ENTIRELY inside f30's
reserved tail, no double-wire:

| Gap-list row | f30 verdict | Owner after merge |
| --- | --- | --- |
| 31, 33, 34, 36, 39, 44, 46 | CITED (same missing systems) | f30's citation stands; c36 does not re-cite |
| 35 | CITED (hardcore death penalty) | **RETIRED — c46-live** (`4445a1459`, tip `2802ed5b9`: `death.rs:527` fires 35 on the survive arm). f30's citation retires at merge; c36 owes nothing (RETIRE (2)) |
| 40 | CITED (vol-2 flag never sets) | **RETIRED — g90b-live** (`b3b82d071`: g90 ports BOTH book uses + the `world_sim/player.rs:1625` notify itself). f30's cited comment retires at merge; c36 owes nothing (the earlier "armor-tail wire is f30's seam" note is superseded — f30's tip `f8cef8cb8` still CITES 40, g90's tip WIRES it) |
| 43 | CITED (state 25 skip) | **RETIRED — f47-live** (`a2561d6d9`: `6f126bd9e` wires the emergence seam + the post-pass `notify_progression_net(43)` itself). f30's citation retires at merge; c36 owes nothing — the b3 adapter fold is f47's published contract (NOTIFY-SYNC (2)) |
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

## W4-c36 NOTIFY-SYNC — the keep-one refresh (2026-11-21, second pass)

Witnesses (WRITTEN NOT RUN, code-only — the consumer halves of the four
published contracts, `app/achievement_hooks.rs`):
`w4_c36_training_day_keeps_one_notify_forty_site`,
`w4_c36_new_digs_fires_on_progression_forty_three`,
`w4_c36_to_infinity_completes_on_the_do_flag`,
`w4_c36_it_can_talk_fires_on_progression_eighteen`.

Tips read this pass: g90 `b3b82d071`, f12 `61724db38`, f47 `a2561d6d9`
(moved past the re-probe's `507a0da5c`), d20 `13de9ccf6`, d120 `4b17a2cf1`,
d37 `08f7d25cf`, f43 `79a037f5e`, b3 `96a5d88f0`, f30 `f8cef8cb8`,
a6 `8ed96ee5d`, d43 `a725acb04` (= main, zero commits). Main unchanged
(`a725acb04`) — everything below is still branch-only.

### (1) TRAINING_DAY 40 — WHO wires it: g90b. c36's owed-row RETIRED

- **g90b-live**: lane-7 `0316b4446` ports `use_combat_book` (4382/5336,
  `Player.cs:45322-45350`, both world flags) in `editing.rs` AND the
  notify: `world_sim/player.rs:1625-1627` (tip `b3b82d071`), the
  `drop_backlog` tail — `if app.world.combat_book_was_used &&
  app.world.combat_book_volume_two_was_used { let _ =
  app.notify_progression_net(40); }`, cited `Player.cs:25695-25698`,
  every tick the pair stands (idempotent: `notify_with` skips completed
  rows). g90's research/135 consumer census pins it (`:282`).
- **f12 latest (`61724db38`)**: NO 40 wire — grep over its crates finds
  only the `achievements_data.rs:1148` row and the `npc_ai_town.rs`
  ladder's flag READS (`TownEnv.combat_book`/`combat_book_two`, the
  NPC.cs:53025-53031 stat ladder — not an achievement site).
- **a6 (`8ed96ee5d`)**: ports its own `use_combat_book`
  (`powerups.rs:316`, both flags) but NO notify — the re-probe's a6 fact
  stands; the merge keep-one is between a6's and g90's USE arms (same
  flags, same once-ever gates — keep ONE; g90's is the notify's
  producer, so keep g90's and drop a6's `powerups.rs:103` dispatch arm).
- **f30 (`f8cef8cb8`)**: still CITES 40 (`world_sim/player.rs:1631-1635`
  — "the volume-two use is unported"); that comment is stale against
  g90/a6 and retires at merge.
- **c36 verdict**: the re-probe's owed armor-tail wire (row 40's
  post-merge seam) is **RETIRED — covered by g90b-live**. c36 adds NO
  40 site. Witness: `w4_c36_training_day_keeps_one_notify_forty_site`
  (the registry row is one `Progression(40)`; one notify completes;
  the re-notify returns None — the every-tick shape is safe).

### (2) NEW_DIGS 43 — f47's tip MOVED: the drain is LIVE on f47, not owed

The re-probe read f47 at `507a0da5c` (field set, no consumer). The tip is
now **`a2561d6d9`** — doc-sweep 36 `6f126bd9e` wired the whole seam:

- `npc_ai_town::town_shimmer_step` (`npc_ai_town.rs:1865-1967`): the
  state-25 emergence machine for a living town-like NPC at `ai[0]==25`,
  on the caller's UpdateNpcs stream; drains `ShimmerFx` (EntityLights +
  NewDustPerfect 309) and RETURNS `out.shimmer_town_progression`.
- `world_sim/npcs.rs`: the slot loop runs it INSTEAD of the dry walk
  (`shimmer25` gate at `:4640`), latches `shimmer_progression43`
  (`:4438`/`:4652`), and the post-pass drain fires
  `notify_progression_net(43)` (`:6367-6369`, ahead of the GetShimmered
  drain) — `NPC.cs:53273`'s flipped-variation-1 completion.
- The field shape (constants-copy contract): `TownOutputs`
  (`npc_ai_town.rs:711`) gains `pub shimmer_fx: Vec<ShimmerFx>` +
  `pub shimmer_town_progression: bool` (`:735`); `TownEnv` gains
  `pub time_for_visual_effects: f64` (`:639`); the completion writer is
  `out.shimmer_town_progression = true; // :53273` (`:1831`, inside the
  `ai[1] >= 75 && shimmerTransparency <= 0 && netMode != 1` arm, gated
  on the flipped `townNpcVariationIndex == 1`).

**c36 verdict: no c36 wire owed** — f47's seam is self-contained. The
only open edge is the **b3 co-lane merge** (`96a5d88f0`'s adapter still
folds `shimmering: false` at `npc.rs:10186` and drops the outputs at
`npc.rs:10209` `let _ = out;`). f47's research/121 MERGE CONTRACT keeps
ONE driver. The exact 2-hunk drain for the b3 lane (apply on b3's tip
AFTER f47's `npc_ai_town.rs` lands — the field is NOT on b3's base, so
these hunks are constants-copy, not a c36 commit):

**Hunk 1 — `crates/tr-app/src/npc.rs`, the adapter's dropped drain
(b3 `:10208-10209`), replace:**

```rust
        // The out drain is phase 2's (see the fn doc); the draws ran.
        let _ = out;
```

**with:**

```rust
        // W4-f47 doc-sweep 36 (NOTIFY-SYNC keep-one) — the TownOutputs
        // drain: the state-25 completion's progression 43
        // (`AchievementsHelper.NotifyProgressionEvent(43)`,
        // `NPC.cs:53273`) latches on the NPC (the adapter cannot borrow
        // the App); the npcs-pass tail fires the notify. The shimmer_fx
        // visuals stay f47's town_shimmer_step seam (ONE driver — the
        // pre-pass owns the dust/lights; this drain only carries the
        // achievement flag, so no double-draw).
        if out.shimmer_town_progression {
            self.wants_shimmer_town_progression = true;
        }
```

(plus the `Npc` field `pub wants_shimmer_town_progression: bool` beside
`town_door_close` (`npc.rs:1334`) + the ctor `false` (`:2993`) — the
`wants_door_bash` intent pattern.)

**Hunk 2 — `crates/tr-app/src/world_sim/npcs.rs`, the post-pass tail
(b3 `:4684`'s `switch_sends` block, before it), insert:**

```rust
    // W4-f47 doc-sweep 36 (NOTIFY-SYNC keep-one) — the state-25
    // completion's progression 43 (`NPC.cs:53273`): the b3 adapter
    // latched it on the NPC; the notify needs the app wholesale, so it
    // credits here (RNG-free). ONE site: if f47's town_shimmer_step
    // pre-pass survives the merge instead, this drain reads ITS latch
    // (`shimmer_progression43`) and the adapter hunk is dropped — never
    // both (the notify is idempotent, but the contract is keep-one).
    let mut shimmer43 = false;
    for n in app.npcs.iter_mut() {
        if n.wants_shimmer_town_progression {
            n.wants_shimmer_town_progression = false;
            shimmer43 = true;
        }
    }
    if shimmer43 {
        let _ = app.notify_progression_net(43);
    }
```

The adapter's `shimmering: false` fold (`npc.rs:10186`) MUST become the
live buff-353 flag (`npc.shimmering`) in the same merge — f47's seam
already reads it. Witness (this branch, the consumer half):
`w4_c36_new_digs_fires_on_progression_forty_three`.

### (3) TO_INFINITY_AND_BEYOND — the exact kill-arm hunk for d20b

d20's tip `13de9ccf6` ports `ai_jims_drone`
(`projectile_ai_misc.rs:1092`): the altitude kill is at `:1105-1109`
(`p.pos.y - p.h <= 640.0` → the kill-flag triple + return, inside the
`p.owner == local_owner` block — `Projectile.cs:40933-40937`). The C#
achievement fires in **`Kill()`**, not the AI: `Projectile.cs:70855-70862`
— the `type == 1020 || type == 1105` arm's 1020 half completes
`GetCondition("TO_INFINITY_AND_BEYOND", "Do")` when
`position.Y - height <= 640f && owner == Main.myPlayer`. So ANY 1020
death at altitude credits (the AI's altitude kill is the normal path;
a tile/expire death below 640 px from the top also reads the same
position gate). The port's death-hook seam is the pool's
`if !self.slots[i].active` block (`projectile.rs:20141`, d20 tip) — the
`moonlord_arrow_kill(i)` neighbourhood (`:20205`), kind-gated,
once-per-death. The 1020 row exists on d20 (`data.rs:3633`) — the hunk
is constants-copy for d20b (its sweep is active); NOT applied on c36
(the row + the AI are not on this base).

**Hunk A — `crates/tr-app/src/projectile.rs`, the `Projectile` struct
tail (d20 `:785`'s `pub owner_hit_check: bool,` — after it, before the
closing brace), add:**

```rust
    /// W4-c36 NOTIFY-SYNC (3) — the TO_INFINITY_AND_BEYOND intent
    /// (`Projectile.cs:70857-70862`, `Kill()`'s 1020 arm): the death hook
    /// latches it when a Jim's Drone dies at `pos.y - h <= 640` under the
    /// local owner; the world sim drains it into
    /// `complete_condition("TO_INFINITY_AND_BEYOND", "Do")` (the
    /// `wants_lava_fishing_achievement` pattern — the pool cannot borrow
    /// the App's manager mid-iteration).
    pub wants_to_infinity: bool,
```

**plus the three literal tails** (the `Default` impl d20 `:873` and the
two test fixtures d20 `:30655`/`:31065` — after each
`owner_hit_check: false,`):

```rust
            wants_to_infinity: false,
```

**Hunk B — `crates/tr-app/src/projectile.rs`, the death-hook chain
(d20 `:20205`'s `self.moonlord_arrow_kill(i);` — after it), insert:**

```rust
                // W4-c36 NOTIFY-SYNC (3) — `Kill()`'s 1020 arm
                // (`Projectile.cs:70855-70862`): Jim's Drone's death at
                // `position.Y - height <= 640f` under `Main.myPlayer`
                // completes TO_INFINITY_AND_BEYOND's "Do". The altitude
                // gate re-reads the DEAD slot's position (the AI's kill
                // at `:40933-40937` returns before any move; a
                // tile/expire death reads its own last position — the
                // C# reads the same field in Kill()). Kind-gated,
                // once-per-death (a dead slot never returns to active).
                // The 1105 half of the C# arm carries no achievement.
                if self.slots[i].kind == 1020
                    && self.slots[i].owner == local_owner
                    && self.slots[i].pos.y - self.slots[i].h <= 640.0
                {
                    self.slots[i].wants_to_infinity = true;
                }
```

**Hunk C — `crates/tr-app/src/projectile.rs`, the drain (beside
`drain_lava_fishing_achievement`, c36 `:15026` — the same shape on d20's
`drain_catches` neighbourhood `:15031`), add:**

```rust
    /// W4-c36 NOTIFY-SYNC (3) — the TO_INFINITY drain
    /// (`Projectile.cs:70857-70862`): true when any 1020 died at altitude
    /// under the local owner this pass; the world sim completes
    /// TO_INFINITY_AND_BEYOND's "Do".
    pub fn drain_to_infinity(&mut self) -> bool {
        let mut fired = false;
        for s in self.slots.iter_mut() {
            if s.wants_to_infinity {
                s.wants_to_infinity = false;
                fired = true;
            }
        }
        fired
    }
```

**Hunk D — `crates/tr-app/src/world_sim/projectiles.rs`, the pass-tail
drain (d20 `:562`'s splash drain — after the `for` loop, before the
catch drain), insert:**

```rust
    // W4-c36 NOTIFY-SYNC (3) — the TO_INFINITY drain
    // (`Projectile.cs:70857-70862`): the 1020's altitude death →
    // `GetCondition("TO_INFINITY_AND_BEYOND", "Do").Complete()`.
    if app.projectiles.drain_to_infinity() {
        let _ = app
            .achievements
            .complete_condition("TO_INFINITY_AND_BEYOND", "Do");
    }
```

Order note: the C# fires inside `Kill()` mid-pass; the port drains at
the pass tail (the lava-fishing/splash convention — the completion is
RNG-free, no stream impact). The `SoundID.Item62` cue + the 15-dust
burst (`:70865-70869`) stay d20's cited visual seams. Witness (this
branch, the consumer half): `w4_c36_to_infinity_completes_on_the_do_flag`.

### (4) The 17/26 roster — the missing-4 UPDATE: all four town kinds on pushed tips

The re-probe's missing list was Truffle 160 / Steampunker 178 / Cyborg 209 /
Pirate 229 + the eight slimes. **NOTIFY-SYNC re-read (tips `4b17a2cf1` /
`08f7d25cf`): ALL FOUR town kinds are now ported on pushed tips:**

| id | kind | lane / commit | proto id | bestiary net_id |
| --- | --- | --- | --- | --- |
| 160 | Truffle | d120 kinds A `2891b0bd8` | 242 | 160 (`bestiary.rs:2186`) |
| 178 | Steampunker | d120 kinds A `2891b0bd8` | 243 | 178 (`:2194`) |
| 209 | Cyborg | d120 kinds B `e4e400629` | 244 | 209 (`:2202`) |
| 229 | Pirate | d37 `750d8e446` (tip `08f7d25cf`) | 361 | 229 (`bestiary.rs:2187`) |

(d120 also lands SantaClaus 142 proto 241 — NOT a roster id; the
BartenderUnconscious 579 folds onto DD2Bartender's Hide row.) The
roster scan reads `kind.net_id()` through `bestiary::ENTRIES` — **zero
c36 edits at their merge**; the witness's missing-4 comments are updated
to name the owning tips. The 25-id roster's other rows are all on main
(107/550 corrected by the re-probe; 353/441/588/633/663 verified in
`bestiary.rs:813-845`).

**The eight town slimes (670/678-684) remain UNPORTED on every tip** —
the all-branch grep for `net_id: 670|678|…|684` is empty; d43's tip
equals main (zero commits — the lead's "slime-8 d43r-band" has no pushed
port; no d43r branch exists). f43 `79a037f5e` ports the SPAWN ladder
(`town_arrival_pets.rs`, id-keyed pure fns, UNWIRED — its own census
note) and f45 Ph13 `7a65b6800` claims the slime-colour unlock latches
(the kinds stay unported, its own message). ALL_TOWN_SLIMES 26 stays
unreachable until d43/f43's kind lane lands the eight NpcKind rows.

**The residual 17-gate after the four kinds merge: the ARRIVAL ladder.**
d120's `LADDER` (`town_arrival.rs:31`) is still the 21-kind main list —
Truffle/Steampunker/Cyborg have NO ladder entry and `condition_holds`
folds `_ => false` (`:219`); d37's Pirate is `is_town_npc_like` but also
outside the ladder. The kinds can be spawned (netglue seeds / debug) and
the roster check counts them, but the vanilla arrival path needs the
ladder arms (`Main.cs:65852-65956` — f43's census owns the Santa/truffle
gates). That is d120/f43's wiring lane, not c36's.

**IT_CAN_TALK 18 — the exact notify hunk (publish for the d120 merge):**
the C# fires in the arrival switch (`WorldGen.cs:5577-5583`), case 160:
notify 18 + `NPC.unlockedTruffleSpawn = true` + `SendData(7)`. The port's
arrival arm (`world_sim/npcs.rs:4949`, the `notify_progression_net(8)`
site) gains, right after the NO_HOBO notify:

```rust
                        // `WorldGen.cs:5579-5583` — the Truffle's arrival
                        // fires IT_CAN_TALK (18) and sets
                        // `NPC.unlockedTruffleSpawn` (the f45 World latch,
                        // `world.rs:797`); the `SendData(7)` world-flag
                        // sync rides the netglue flag push.
                        if n.kind.net_id() == 160 {
                            let _ = app.notify_progression_net(18);
                            app.world.unlocked_truffle_spawn = true;
                        }
```

(constants-copy: the Truffle kind is d120's, the World latch is f45's —
neither is on this base; the hunk lands with their merge. The stale
"waits on a ported Truffle kind" comments on c36 `:4947` and d120
`:5038` retire then.)

## W4-c36 RETIRE (1) — the d43r-session probe: DEAD pre-work (2026-11-21)

Read-only probe of the d43 lane (no writes to its worktree):

- `origin/task/W4-d43-npc-kinds-631-697` tip **`a725acb04` = origin/main**
  (`git rev-list --count origin/main..` = 0). No `d43r` branch exists —
  the full 357-ref `for-each-ref` census finds only the d43 band ref.
- The `TR-W4-d43` worktree's git dir
  (`Terraria-Rust/.git/worktrees/TR-W4-d43`): `logs/HEAD` holds exactly
  TWO entries — the checkout `0000…→a725acb04` at epoch 1795188713
  (2026-11-20 17:31:53 +0200) and a `reset: moving to HEAD` one second
  later. `index` mtime 17:38:33; every tracked file's mtime is 17:31:54
  (the checkout write itself). **Zero commits, zero edits — the d43r
  session died before any work.** Verdict: DEAD pre-work, not a stalled
  port; the band needs a fresh dispatch.

**Consequence — the eight town slimes 670/678-684 are UNPORTED on every
pushed tip** (the all-357-tip `net_id: (670|678|…|684)` bestiary scan is
EMPTY; d37's `NpcKind::RainbowSlime` is the WILD rainbow slime, bestiary
`net_id: 244` at `bestiary.rs:2199` on `a341daa12` — NOT the town 681).

**Task-seed priority: HIGH — three published consumers already wait on
the kinds** (the seed is no longer speculative):

| dependent | what waits | cite |
| --- | --- | --- |
| **roster-26-gate** | ALL_TOWN_SLIMES 26 — c36's `check_real_estate_and_town_slimes` is wired at all three C# call sites; the 8-slime half of the roster can never pass until the kinds land | this doc, DOC-SWEEP + NOTIFY-SYNC (4) |
| **f43-ladder** | `town_arrival_pets.rs`'s license/arrival arms are id-keyed pure fns over `cs_id` 670/678-684 (`:289-296` the `IsTownSlime` set, `:313-317` the `SlimeNames_*` keys) — UNWIRED by its own census until the kinds exist | f43 `79a037f5e` |
| **c39-slime-colors** | `party_hat.rs:16-19`'s recorded deviation — the town-slime party-hat special cases (`Main.cs:26945-27028`, types 670-684) are dropped until the kinds ship; d120's `slime_chat_colour` is ALREADY type-keyed (zero d120 edits at activation, its RETIRE note §2) | c39 `7dd785dc5`, d120 `2408233da` |

Plus c39 §14's **QueenSlime 657** seed targets the same dead band (its
READY death-arm hunk, event id 24, waits on the kind; d33's
`npc_ai_style121.rs` machine is landed UNWIRED). The f45 Ph13 slime-colour
unlock latches (`7a65b6800`) fold onto the same rows.

**Seed scope (fresh d43r dispatch):** the 8 `NpcKind` rows + bestiary
`ENTRIES` (vanilla net ids 670/678-684) + sheets/portraits + the f43
ladder wiring + the f45 colour-latch fold + the c39 party-hat special
cases + QueenSlime 657 (c39 §14's hunk is ready). Zero c36 edits at
activation — the roster scan reads `kind.net_id()` through `ENTRIES`.

## W4-c36 RETIRE (2) — the 180-condition FINAL census (2026-11-21)

Method: `git fetch origin --prune`, then a scripted `git grep` of EVERY
one of the 357 `origin/task/W4-*` tip trees (not diffs) for each gap row's
notify symbols — `notify_progression_net(N)` for 18/24/31/33-36/39/44-46,
the row names for the flag rows, and the `complete_condition("ROW"` /
`handle_special_event(9/21/22)` dispatch shapes — excluding the registry
(`achievements_data.rs`), the manager (`achievements.rs`), the menu and
c36's own hooks. f30 re-read at its LATEST tip: **`f8cef8cb8` UNCHANGED
(no f30b branch exists on origin)** — its tail census stands: 21/37-partial
(4715 arm)/38/41 WIRED, 45 PREP (`research.rs:111`
`check_research_achievement`, the notify rides the caller — the doc at
`:103`, no live call site), 31/33/34/35/36/39/40/43/44/46 CITED.

**Deltas vs the NOTIFY-SYNC net (the tips moved):**

| Row | Delta | Evidence |
| --- | --- | --- |
| EXTRA_LIFE 35 | **RETIRED — c46 wires it LIVE** | c46 `4445a1459` (tip `2802ed5b9`): `death.rs:527` `notify_progression_net(35)` on the `HardcoreDeathPenalty` survive arm, wired at the KillMe difficulty-2 gate `:437-438` (`Player.cs:38782`/`:38864-38866`); the stat half `player.rs:3851`, `KillMeForGood` `:38789-38805` |
| TURN_GNOME_TO_STATUE 24 | the KIND landed; the notify stays unowned | d42 `2d7ac08c3` (the roster join now compiles — the old non-compiling-WIP note retires): Gnome 624 `npc.rs:1250` + bestiary `:2398`; the AI rides `update_zombie` (`:6715`, stand-in) — no statue conversion, no notify 24 on any tip |
| FIND_A_FAIRY "Do" | the KINDS landed; the sight seam stays unowned | d42 `2d7ac08c3`: FairyCritterPink/Green/Blue, bestiary net ids 583/584/585 (`:2203-2224`); no `handle_special_event(22)` / FIND_A_FAIRY completion on any tip (g58's hits are NEGATIVE assertions in a witness, `app/mod.rs:5824-5825`) |
| TRAINING_DAY 40 / NEW_DIGS 43 | re-verified LIVE at the moved tips | g90 `740650d75`: `world_sim/player.rs:1625-1627` intact; f47 `a2561d6d9` unchanged: `world_sim/npcs.rs:6368` intact |
| TO_INFINITY "Do" | the d20 hunks' line pins still VALID | d20 `13de9ccf6→243f81672` is DOCS-ONLY (`git diff --stat -- crates` EMPTY); the 1020 row `data.rs:3633` + `ai_jims_drone` `projectile_ai_misc.rs:1067` intact; still NO achievement wire |
| IT_CAN_TALK 18 | the d120 arrival site MOVED | d120 `2408233da`: the NO_HOBO notify is now `npcs.rs:5042` (was `:4949`/`:4951` on c36's base); the stale "waits on a ported Truffle kind" comment `:5038-5039` unchanged; the published hunk lands after `:5042` (NOTIFY-SYNC (4)'s shape, re-pinned) |
| 5043 GAIN_TORCH_GODS_FAVOR | f33's FINAL verdict supersedes the DOC-SWEEP keep-both note | f33 `83387628b` research/93a §3b/§8: keep the f33 full body, DROP c36's reduced arm — the exact c36 drop-hunk is published in RETIRE (3) below |

**THE FINAL LIST — conditions with NO notify path ANYWHERE (17 rows,
after the 35 retirement; every one re-grepped across all 357 tips):**

| # | Row | Cond | C# site | Missing owning system (final state) |
| --- | --- | --- | --- | --- |
| 1 | IT_CAN_TALK | 18 | `WorldGen.cs:5580` | the arrival-ladder entry (d120/f43's lane); the kind is on d120, the hunk published |
| 2 | TURN_GNOME_TO_STATUE | 24 | `NPC.cs:55896` | the gnome statue-conversion AI (kind on d42, zombie stand-in) |
| 3 | QUEEN_MACHINE | 31 | `NPC.cs:78866` | Mechdusa summon 5334 + combined-boss death (f30's seed gate landed; no 5334 item/AI on any tip) |
| 4 | FEAR_THE_SUN | 33 | `Player.cs:28647` | vampire sun-scorch (`UpdateSunScorch`) |
| 5 | ITS_SHALING_OUTSIDE | 34 | `WorldGen.cs:71694` | boulder-rain start |
| 6 | GRAVE_MISTAKE | 36 | `Player.cs:20473`/`:38836` | striking gravestones + MP teammate deaths |
| 7 | FORTUNE_FAVORS_THE_BOULD | 39 | `Main.cs:61828` | rainbow-boulder draw counter |
| 8 | MY_PEOPLE_NEED_ME | 44 | `Player.cs:24076` | remix top-world death (`BordersMovement`) |
| 9 | TRASH_COMPACTOR | 45 | `AchievementsHelper.cs:396` | the c24+f30 merge join (consumer + prep both pushed, no live call) |
| 10 | INTERDIMENSIONAL_RECYCLING | 46 | `Main.cs:40294` | Dryad Joja-cola gift path |
| 11 | SPICY_LICKS (partial) | 37 | `Player.cs:46574`/`:46588` | the 1305/4057/4372 instrument arms (`ItemCheck_PlayInstruments` unported on every tip; f30's 4715 arm live) |
| 12 | VEHICULAR_MANSLAUGHTER | "Hit" | `Player.cs:27602` | minecart NPC-collision damage (`GetMinecartDamage` — all-tip grep empty) |
| 13 | PET_THE_PET | "Do" | `Player.cs:33350`/`:33397` | petting system |
| 14 | FIND_A_FAIRY | "Do" | `NPC.cs:48647` | the fairy SIGHT seam (kinds on d42; HandleSpecialEvent 22 unwired) |
| 15 | PURIFY_ENTIRE_WORLD | "Do" | `Main.cs:40276` | Dryad world-status dialog + button |
| 16 | TO_INFINITY_AND_BEYOND | "Do" | `Projectile.cs:70861` | the kill-arm wire (AI + row on d20; the 4-hunk patch published, NOTIFY-SYNC (3)) |
| 17 | GOING_OLDSCHOOL | "Do" | `Main.cs:63018` | classic title-screen sun/moon grab |

**Retired since the DOC-SWEEP census (3):** TRAINING_DAY 40 (g90b-live),
NEW_DIGS 43 (f47-live), EXTRA_LIFE 35 (c46-live). **Reachable-but-gated
(the wire exists, the gate cannot pass in play):** REAL_ESTATE_AGENT 17
(the 4 town kinds on d120/d37 — activates at their merge; the arrival
LADDER is the residual gate), ALL_TOWN_SLIMES 26 (the 8 kinds unported —
the DEAD d43 lane, RETIRE (1)'s HIGH-priority seed), BOOK_WORM 29 (the
hook fires; the residual gate is c37's provider fold). Everything else
in the 86 mapped conditions has a live notify site on this branch, f30's
tip, or a published merge join; the 94 Npc/Item/Tile/Craft conditions all
keep their live dispatch (per-id reachability is content coverage).

## W4-c36 RETIRE (3) — 5043 FINAL: keep-f33-full-body, the c36 drop-hunk (2026-11-21)

**The verdict is f33's** (research/93a §3b + §8, retire tip `83387628b`,
read in full): the f33 tip `8a000407` landed the WHOLE
`ItemCheck_UseTorchGodsFavor` body (`torch_god_favor_use.rs` —
`run_favor_arm` `:98`, the `handle_special_event(24)` credit `:117`, the
press-edge machine `torch_gods_favor_item_check` `:182`, the
`editing.rs:2007` seam) + the named dispatcher (`achievements.rs:482-487`).
a6b `d7de8bd5` fixed its broadcast (row-targeted now) but still lacks the
ApplyItemTime order, the `UsingBiomeTorches` property accessor, the net
split, the msg-51 request and the `:43323` re-arm. **KEEP f33's full body;
DROP a6's arm and c36's reduced copy.** c36's enter-world re-check
(`achievement_hooks.rs:173-176`, the `OnPlayerEnteredWorld` `:192-195`
arm) is a SEPARATE wire and SURVIVES — it re-completes the row for a
character whose `.plr` carries the unlock (idempotent).

**Re-verified at the moved tips (this pass):** a6 `c039e4448` (retire
1/4+2/4 are docs — its 5043 arm is UNCHANGED from `d7de8bd5`, the §3b
drop pins still valid: `powerups.rs:347-366` the fn, `:104` the dispatch
entry, `:71` the `is_powerup` leg, `:1071`/`:1093` the two witnesses);
f33 `83387628b` (the seam `editing.rs:2007` + the dispatcher intact);
the `GAIN_TORCH_GODS_FAVOR` row is blob-identical c36↔f33
(`e048692067a9…`). **g53 `04013f53d`'s row-category correction rides its
own tip** (`Category::Slayer`, blob `b9f8c4f8…` — the vanilla
`(AchievementCategory)0` default; c36's `Category::None` row dedups onto
g53's at merge, no c36 edit — the registry is e22/g53's lane).

### The exact c36 drop-hunk (apply at the f33×c36 merge, THIS tip's lines)

All four hunks are in `crates/tr-app/src/world_sim/editing.rs` (tip
`5be9d3438` line numbers — the RETIRE phases are doc-only, the code lines
are `3a64a1b96`'s):

**Hunk 1 — the `is_summon` gate list, `:2017`, replace:**

```rust
                | 4271 | 2767 | 29 | 109 | 1291 | 5043
```

**with:**

```rust
                | 4271 | 2767 | 29 | 109 | 1291
```

(the 5043 leg leaves the summon gate — f33's favor arm rides its OWN
`:43600` permanent-item seam, not the mid-swing summon list.)

**Hunk 2 — the dispatch arm, `:2088-2094`, DELETE (7 lines):**

```rust
        // W4-c36 — the Torch God's Favor (`ItemCheck_UseTorchGodsFavor`,
        // `Player.cs:45226-45242`): the 5043 use while locked unlocks the
        // biome torches and fires HandleSpecialEvent(24) →
        // GAIN_TORCH_GODS_FAVOR ("Use").
        if held.kind == 5043 && held.count > 0 && use_torch_gods_favor(app) {
            return;
        }
```

(f33's `editing.rs:2007` seam replaces it — the C# `ItemCheck` order puts
the favor check at the permanent-item seam BEFORE the summon gate.)

**Hunk 3 — the reduced fn, `:16325-16353`, DELETE (29 lines: the doc
`:16325-16331` + `pub(crate) fn use_torch_gods_favor` `:16332-16353`).**
The reduced copy's gaps vs the C# body (f33 93a §3b's table): no
`UsingBiomeTorches` write (the port folds it into `unlocked_biome_torches`,
`player_lights.rs:97`'s recorded deviation — f33 carries the real
accessor pair), no net split / msg-51 request, the stack pay inline
instead of the `:43280-43326` consumable tail with the `:43323` re-arm.

**Hunk 4 — the witness, `:16534-16559`, DELETE (26 lines: the doc
`:16534-16536` + `#[test]` `:16537` +
`w4_c36_the_torch_gods_favor_unlocks_and_completes` `:16538-16559`).**
Its assertions re-pin onto f33's six witnesses (93a §3's list: the
Use-flag row pin `achievements.rs:839`, the event-24 isolation `:868`,
the use-arm grant + credit). The enter-world sweep witness
(`achievement_hooks.rs:516-544`, `the_enter_world_sweep_completes_the_loaded_rows`
— its `GAIN_TORCH_GODS_FAVOR` assertion `:534`) SURVIVES untouched.

**Keep on c36 (NOT dropped):** the `GAIN_TORCH_GODS_FAVOR` registry row
(`achievements_data.rs:1004-1010` — e22/g53's lane owns its category
correction), the enter-world re-check (`achievement_hooks.rs:173-176`),
the `complete_condition` primitive (`achievements.rs` — ONE helper at the
merge, a6b==c36==f33; a6b's `frac`-clear line rides the survivor if the
shared `Condition` carries `frac`, 93a §8 step 3). The
`player_lights.rs:94-98` fold comment retires at the merge (f33's accessor
pair replaces the folded read — the biome-torch hold-style then reads the
real `UsingBiomeTorches` property).

**Anchors:** `anchors.txt:901`'s feature text names
`use_torch_gods_favor` beside the power-up fn — the MARKER
(`fn use_permanent_powerup`) survives the drop; the merger repoints the
feature-text clause to f33's `torch_god_favor_use.rs` (the canary rule —
never delete the row).

## W4-c36 RETIRE (4) — FINAL-FINAL + the achievement-family merger-bible (2026-11-21)

> **The M task-seed update (this phase):** the notify-site M seed's
> remaining list is the RETIRE (2) FINAL 17-row register (was 20 — 40/43
> retired at NOTIFY-SYNC, 35 retired here onto c46); the d43r slime-8
> seed is NEW at HIGH priority (three published consumers + the ready
> QueenSlime-657 hunk). No other seed changes state this pass: the c37
> provider fold, the ReportDroprates L, the bestiary-UI M and the
> ENTRIES/sheet reconciliation S stay open with their owners named.

**Status: FINAL-FINAL.** The lane retires at this commit. Code behaviour
is exactly `3a64a1b96`'s (the NOTIFY-SYNC (5) tip) — the four RETIRE
phases are doc-only; all witnesses WRITTEN NOT RUN (no cargo/tests/
rustfmt/graphify in any phase). Every obligation is either landed on this
branch, retired onto a sibling's live wire, or published as an exact
evidence-cited hunk/seed for its owner.

### The final state (one paragraph)

137 rows / 180 conditions, all modelled; 86 mapped conditions carry live
notify sites on this branch + f30's tip + the sibling live wires
(40 g90b, 43 f47, 35 c46); the FINAL no-path list is **17 rows** (RETIRE
(2)), each with its missing owning system named; the roster 17 activates
at the d120/d37 merge (zero c36 edits), 26 waits on the DEAD d43 lane's
fresh dispatch (RETIRE (1), HIGH-priority seed); the save wire matches the
C#'s single external call site; the popup/event-queue landed via S1-E-26.

### The achievement-family merger-bible (the keeper's ordered recipe)

Read at the fetched tips (2026-11-21): c36 `deaffcfda` (this), f30
`f8cef8cb8`, e22 `aae211f16`, g53 `04013f53d`, f33 `83387628b`, a6
`c039e4448`, g90 `740650d75`, g88 `a9f1c29d8`, f47 `a2561d6d9`, b3
`96a5d88f0`, c46 `2802ed5b9`, d20 `243f81672`, d120 `2408233da`, d37
`a341daa12`, c24 `e2e50a427`, c37 `3e138525e`, d42 `2d7ac08c3`, f43
`79a037f5e`, f45 `6352d3840`, c39 `7dd785dc5`, main `a725acb04`.

1. **Registry + condition table (e22 + g53 win the data):** keep ONE
   `achievements_data.rs` — e22's `CONDITION_COUNTS` transcription test
   (`aae211f16`) + g53's `Category::Slayer` correction on
   GAIN_TORCH_GODS_FAVOR (`04013f53d`, blob `b9f8c4f8…`; the vanilla
   `(AchievementCategory)0` default — c36/f33's `Category::None` blob
   `e0486920…` dedups onto it). c36/f30 never touch the table (the lane
   contract); no semantic conflict — the rows are identical otherwise.
2. **The notify-site union (c36 + f30 + the live siblings):** c36's site
   table (the lane-claim + DOC-SWEEP tables) and f30's tail
   (21/37-partial/38/41) are DISJOINT by the co-lane reservation — take
   both wholesale. The three live sibling wires join with ZERO c36/f30
   edits: 40 (g90 `world_sim/player.rs:1625-1627` — f30's cited comment
   `:1631-1635` retires), 43 (f47 `npcs.rs:6368` — f30's citation
   retires), 35 (c46 `death.rs:527` — f30's citation retires). At the
   a6×g90 merge keep ONE `use_combat_book` (g90's — the notify's
   producer) and drop a6's `powerups.rs` dispatch arm (NOTIFY-SYNC (1)).
3. **5043 (RETIRE (3)):** keep f33's full body + named dispatcher; apply
   the published c36 4-hunk drop (`editing.rs:2017/:2088-2094/:16325-16353/:16534-16559`)
   and a6's §3b drop (`powerups.rs:347-366/:104/:71` + the two witnesses);
   keep c36's enter-world re-check; ONE `complete_condition` helper
   (a6b's `frac`-clear rides the survivor). The persist halves ride
   g88's v11 superset per 93a §8 step 4 / 103 §9.
4. **The b3 town-AI merge (NOTIFY-SYNC (2)):** f47's `town_shimmer_step`
   keeps ONE driver — if b3's adapter survives, apply the published
   2-hunk drain (`npc.rs:10208-10209` replace + the `npcs.rs:4684`
   insert, the `wants_shimmer_town_progression` field/ctor riders) and
   lift `shimmering: false` (`npc.rs:10186`) to the live buff-353 flag;
   if f47's pre-pass survives, drop the adapter hunk. Never both.
5. **The d20 merge (NOTIFY-SYNC (3)):** apply the published 4-hunk
   TO_INFINITY kill-arm (the `wants_to_infinity` field + 3 literal tails,
   the death-hook latch after `moonlord_arrow_kill` `:20205`, the
   `drain_to_infinity` pool drain, the pass-tail `complete_condition`) —
   the line pins re-verified VALID at `243f81672` (crates diff empty).
6. **The d120/d37 merge (NOTIFY-SYNC (4)):** the roster-17 scan activates
   with zero edits; apply the published IT_CAN_TALK arrival hunk after
   the NO_HOBO notify — **re-pinned to d120 `2408233da`'s
   `npcs.rs:5042`** (was `:4949`); the `unlocked_truffle_spawn` latch is
   f45's (`world.rs:797`); the stale comments (c36 `:4947`, d120
   `:5038-5039`) retire. The arrival LADDER arms are d120/f43's lane.
7. **The c24+f30 merge (45):** the sacrifice click (`journey_clicks.rs`)
   → f30's `check_research_achievement` (`research.rs:111`) →
   `notify_progression_net(45)`; the enter-world forced call
   (`Player.cs:325`) + the per-tick `:25703` ride the same join.
8. **The c37 provider fold (29's residual gate):** c37's `Gate` enum +
   the `WorldBestiary` completion move (`3e138525e`) — c36's two
   `KillTracker`-based `try_grant_bestiary_100` call sites
   (`netglue.rs:2987`, `netmodules.rs:467`) take the one-line rebase;
   the gate predicate is identical, no semantic conflict.
9. **The d43r fresh dispatch (RETIRE (1), HIGH):** the 8 town-slime kinds
   + QueenSlime 657 — the roster-26 gate, f43's ladder arms, c39's
   party-hat colour cases and the f45 colour latches all activate/fold
   with zero c36 edits.
10. **Post-merge verify:** the anchors canary (`anchors.txt:901`'s marker
    survives the 5043 drop — repoint the feature text), the four
    NOTIFY-SYNC consumer witnesses + the DOC-SWEEP four + the lane-claim
    witnesses run green, and the FINAL census's 17-row list is the
    residual gap register for the next wave's task seeds.

### The retire commit table

| Phase | Commit | Scope |
| --- | --- | --- |
| RETIRE (1) | `40006c825` | the d43r probe (DEAD pre-work) + the HIGH-priority slime-8 seed |
| RETIRE (2) | `5be9d3438` | the 180-condition FINAL census (17-row final list, 35 retired, the 24/22 kinds noted) |
| RETIRE (3) | `deaffcfda` | 5043 keep-f33 verdict + the exact c36 drop-hunk |
| RETIRE (4) | this commit | FINAL-FINAL + the merger-bible + the ANALYSIS/ROADMAP/PORT_MAP/INDEX receipts |

**The merger's entry point: this section + RETIRE (2)'s final list +
RETIRE (3)'s drop-hunk. Nothing on this branch needs a follow-up pass.**
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

### W4-f30 RETIRE phase (2026-11-21) — `335-f30-systems-retire.md` (renamed 240→335)

The sibling lanes landed their overlapping halves (c46 hardcore `7330cd4e`,
d70 sun-scorch `a7620eec`, c24 Journey sacrifice `ff99f148`, f47 shimmer
`1997be09`, d2 tombstones `f430ccf8` — all UNMERGED at this writing). The
retire verdicts, the exact keep-one drop-hunks and the compose meeting points
are published in **`335-f30-systems-retire.md`** (renamed 240→335 per the d54
collision registry 150 §9.2 — g58's earlier rename wins 240): 35 → KEEP c46 (the verbatim two-file delete + the no-draft-clear
ruling + the seeded `update_dead`), 33 → KEEP f30's scan/33 wire, DROP d70's
method copies (its lens-flare/shadow/HeatDistortion halves survive on the kept
field), 45 → COMPOSE with c24 (its clamped `register_sacrifice` + f30's
`LastEditId` bump at the `MarkContentsDirty` slot; the forced EnterWorld check
is the load-path 45 site — C# `Load` does not dirty), 43 → KEEP f47's wired
machine, DROP f30's core (+ add the `netMode != 1` completion gate f47 omits),
36 → KEEP d2's verbatim flight + f30's 36 machinery/date/FTW-hostile folds.
The census table above stays the band's record; the FINAL systems-family
declaration (the lane is census-complete for its reserved band) is 335 §6.

## Task seeds

- ~~**S** — Model `NPCKilledCondition.CreateMany` faithfully~~ — **done (S1-E-18)**.
- ~~**S** — Call `achievements::save` from the autosave/F9 pass~~ — **done (S1-E-18)** (save on completion + `save_world`, path-bound bootstrap).
- **M** — Continue wiring notify sites for progression/custom rows as owning systems land (remaining: the gap list, FINAL census below — 17 rows with no notify path on any tip; 17 partial, 18; 19/20 + nurse `Pay` landed). Partially done (S1-E-18). Dep: owning systems merging to main. **NOTIFY-SYNC: 40/43 retired (g90b/f47 wire them live), TO_INFINITY + 18 hunks published, the roster's 4 town kinds landed on d120/d37. RETIRE FINAL: 35 retired (c46 `2802ed5b9` wires EXTRA_LIFE live); the 24/22 seams gained their kinds on d42 `2d7ac08c3` (Gnome 624 / Fairy 583-585) — the AI/sight halves stay unowned.**
- **M (HIGH priority)** — **d43r fresh dispatch: the 8 town slimes 670/678-684** (+ QueenSlime 657). The d43 session is DEAD pre-work (RETIRE (1) probe: checkout-only reflog, zero commits, no d43r ref); three published consumers wait (the roster-26 gate, f43's id-keyed ladder, c39's party-hat slime-colour cases) + c39 §14's ready 657 death-arm hunk. Zero c36 edits at activation.
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
- **M** — Continue wiring notify sites for progression/custom rows as owning systems land (remaining: 17 partial, 18, 19/20, nurse `Pay`, most progression/custom flags; 29 blocked on S1-F-15). Partially done (S1-E-18). Dep: owning systems.
- ~~**S** — Make `kills_needed_for` fall back to
  `ItemID_Sets_DefaultKillsForBannerNeeded` (50) instead of 0, and pin it with a
  regression test that a fresh tracker leaves a non-quick entry at
  `NotKnownAtAll`. Fixes #4.~~ **DONE (S1-F-15 + W4-c37)** — the full banner
  chain + the fresh-world census witness.
- ~~**M** — Replace the `quick_unlock` bool with a per-entry provider enum
  (`CommonEnemy` / `TownNPC` / `Critter` / `HighestOfMultiple` / `GoldCritter` /
  `Salamander`) and record sightings and chats. Fixes #5, #6.~~ **Providers DONE
  (W4-c37)** — `bestiary_data::Gate` (Common/Town/Critter/Highest) over the
  populator's override walk; the GoldCritter/Salamander/TorturedSoul gates have
  no port entries yet (recorded in `gate`'s doc). The sight/chat RECORDING stays
  open (#6's second half).
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
- ~~**S** — Add a data test that pins every Rust row's condition count against
  `AchievementInitializer.cs` (137 rows / 180 conditions) so the `CreateMany`
  class of bug cannot return. Dep: the first S task. **DONE on e22
  `aae211f16`** (W4-c36 re-probe: its `CONDITION_COUNTS` transcription test).
  class of bug cannot return. Dep: the first S task.~~ **DONE (W4-e22,
  2026-11-20)** — `achievements_data.rs::tests::every_rows_conditions_match_the_initializer_verbatim`
  compares `ROWS` against a second verbatim transcription
  (`CONDITION_COUNTS`) of `AchievementInitializer.cs:16-443`: row order,
  per-row count, and every condition's kind + arguments (137 rows / 180
  conditions). An independent C#-vs-Rust diff (kinds, ids, condition names,
  trackers, icons, categories) found **0 mismatches** — the table itself
  needed no porting; S1-E-18's `b93459de` had already landed the expansion.
  `the_create_many_rows_stay_expanded` pins the four `CreateMany` shapes
  (3/4/24/15) and that no row is born complete.
