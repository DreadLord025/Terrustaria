# 68 — Mood reconcile: W4-f37 ProcessMood core × W4-g58 happiness UI (merger assist)

Date: 2026-11-21. Code-only reconcile — no cargo/tests run; every claim below is
re-read from the two branch tips and the C# cites they carry.

**Sides examined**

| Side | Ref | Tip | Scope |
| --- | --- | --- | --- |
| f37 (this branch) | `task/W4-f37-shop-price-math` | `7df7b7d8` (phase 2 core `c939c97c`) | faithful `ShopHelper.ProcessMood` + capture-once latch + discount/hair price math |
| g58 | `origin/task/W4-g58-ui-shop-happiness` | `1409a455` (P1 `cbd244e9`, P2 `46a72dd5`, P3 `084e8362`) | full preference DATA tables + report builder + `SetTalkNPC` settings latch + chat Happiness button + shop chrome |

**g58b census finding (up front):** no `g58b` continuation exists anywhere —
`git ls-remote origin` shows only `task/W4-g58-ui-shop-happiness` at `1409a455`;
the g58 worktree is clean at the same tip; local branches, reflog and `fsck`
dangling commits carry no g58 follow-up touching `happiness.rs`. The deviation
census in §5 is therefore against g58's own ANALYSIS "Shop happiness UI (W4-g58)"
list, scored by what f37 already fixes. If a g58b lands later, re-run §5 against it.

Both branches fork from the same base `a725acb0` and both rewrite
`crates/tr-app/src/happiness.rs` (f37 +765 lines, g58 +847 lines) — the merge
WILL conflict there and in `app/mod.rs`, `app/clicks.rs`, `world_sim/npcs.rs`,
`recall_teleport.rs`, `app/frame/draw.rs`. This doc is the resolution plan.

## 1. Side-by-side arm table (`process_mood` + companions)

C# anchor: `ShopHelper.cs:95-181` (+ `AllPersonalitiesModifier.cs:12-506`,
`BiomePreferenceListTrait.cs:38-53`). "faithful" = matches the C# arm.

| Arm | f37 (`happiness.rs:407-602`) | g58 (`happiness.rs:305-473`) | Verdict |
| --- | --- | --- | --- |
| loveStruck ×0.9 head BEFORE exclusions (`:99-102`, buff 119) | YES — `npc.buffs.has_buff(119)`, unrounded on the early-return path | NO — comment defers to a "buff 87 hook" (wrong buff id, no code) | f37 |
| Remix-world early return (`:103`) | YES — `SeedFlags::REMIX`, every town NPC out at the raw adjustment | NO — only the TravellingMerchant returns | f37 |
| Exclusion set 37/368/453 (`:103-106`, `IsNotReallyTownNPC :363-371`) | YES — by `net_id` | PARTIAL — TravellingMerchant kind only (37/453 unported, equivalent today) | f37 (superset) |
| Homeless / >120-tile sentinel 1000 (`:107-114`) | YES | YES (same math) | either |
| Evil-biome sentinel (`:115-118`) | YES — but name-less `MoodFlag::HateBiomeEvil` | YES — **`HateBiomeNamed(BiomeKind)`** carries the `_dangerousBiomes` first-hit name for the report (`:28-34`/`:353-356`) | g58 flag, f37 placement (scan once for sentinel + traits) |
| Crowd scan roster (`GetNearbyResidentNPCs :288-325`) | `is_town_npc_like` **+ DD2Bartender** minus 37/368/453 — Tavernkeep (550) COUNTS | `is_town_resident` minus TM/DD2Bartender — Tavernkeep EXCLUDED (recorded deviation) | f37 |
| **Princess-random-loves** (RNG head, `AllPersonalitiesModifier.cs:15-36`) | YES — `rng: &mut UnifiedRandom` param (`app.rngs.global()`), type-deduped id-sorted picks, `next_max` + remove, up to 3 ×0.88; RNG-replay witness | NO — constants-off (recorded deviation); no rng param at all | f37 |
| Princess HateLonely / no-crowd-mult / no-LoveSpace (`:122-131`) | YES — `crowd_step = 1.0`, HateLonely sentinel when <2 house AND <2 village, crowded REPORT still lands | NO — constants-off | f37 |
| Universal LikePrincess ×0.94 (`:37-40`) | YES — `nearby_ids.contains(&663)` | YES — `nearby_types.contains(&NpcKind::Princess)` | equivalent |
| Crowd ×1.05 past 3 / Hate>6 (`:132-146`), LoveSpace gate (`:147-151`) | YES | YES (same thresholds) | either |
| **Zone-judge** — biome traits test the PLAYER (`BiomePreferenceListTrait.cs:44` `IsInBiome(info.player)`) | YES — `shopping_zones` snapshot; Forest/Underground ride the special getters (`Player.cs:3823-3847`: `BelowSurface = tileY > worldSurface`, `Forest = !AnyBiome && !BelowSurface`, Desert counts) | NO — home-anchor scene scan + `classify_biome` first-match (the recorded M4-TOWN-2 deviation); no Mushroom/Underground rows reachable | f37 |
| Single BEST-affection biome row (`AffectionLevel.cs:5-8` ranks) | YES — `affection_rank` compare, one multiply | NO — multiplies every row matching the one classified biome (moot only under its own wrong judge) | f37 |
| **Sentinel-traits** — traits run at adjustment ≥ 1000 (`:152-168` has NO gate) | YES — homeless/evil NPCs still collect LoveSpace + biome + neighbor rows into the report | NO — `if adjustment < 1000.0` gates the whole trait block; the report loses those rows | f37 |
| Neighbor switch rows (`:41-505`) | 20 ported-kind cases keyed `NpcKind`, values `(AffectionLevel, i32 vanilla id)` — unported ids stay verbatim | ALL 25 cases keyed C# NPCID (`neighbor_preferences_by_id`), incl. unported 142/160/178/209/229/550; kind view DROPS unported neighbors (`Box::leak` OnceLock) | g58 table (superset), f37 id-carrying contract |
| Biome DATA rows (`PersonalityDatabasePopulator.cs:27-147`) | ~20 ported kinds, hand-expanded structs | all 25 keyed NPCID incl. the unported six; kind view rides `cs_id` | g58 table (superset) |
| **Banker's-cents** tail (`LimitAndRoundMultiplier :176-181`, `Math.Round` ToEven) | YES — `shop::round_half_to_even(clamped * 100.0) / 100.0` | NO — `.round()` is away-from-zero (92.5 → 93, wrong) | f37 |
| **Report-prepend** (`AddHappinessReportText :188-196`, `text + report + " "`) | NO — f37 latches the multiplier only | YES — `prepend_report`/`happiness_report`, category `TownNPCMood_{name}`, `{BiomeName}`/`{NPCName}` substitutes, `LikeNPC_Princess` dedicated key (`:252-259`), `npc_full_name` (`NPC.cs:7010-7020`), `biome_name_key` (Underground = `NormalUnderground`) | g58 |
| `ShoppingSettings` + `GetShoppingSettings` (`ShoppingSettings.cs:3-14`, `ShopHelper.cs:49-62`) | NO struct — `App::shop_mood: f32` latch | YES — `ShoppingSettings { price_adjustment, happiness_report }` + `shopping_settings()` wrapper (calls its own rng-less `process_mood`) | g58 shape, f37 capture |
| Capture-once per `SetTalkNPC` (`Player.cs:4383-4390`) | YES — talk open latches `shop_mood` (`world_sim/npcs.rs:223-231`); shop open re-captures ONLY when `talk_npc != Some(idx)` (collapsed-entry #9 deviation) — no double Princess draw; 6 close sites reset 1.0 | PARTIAL — `App::set_talk_npc` seam latches the struct at all 6 sites, BUT the base shop-open capture (`world_sim/npcs.rs:992`) still runs unconditionally → two mood passes per talk+shop open (harmless without RNG, a double-draw once f37's Princess head lands) | f37 gating, g58 seam |
| Nurse heal cost reads the latch (`Main.cs:40100`) | YES — `nurse_heal_cost()` reads `self.shop_mood` | NO — keeps the per-call `process_mood` re-scan (`app/mod.rs:2114` g58 side) | f37 |
| Chat Happiness button (`NPCInteractions.cs:289-312`) + `CanShowHomelessText` (`NPC.cs:94828-94841`) | NO | YES — `ui/npc_chat.rs` button row (index 1, wrap-4 rect math), Condition + sound-12 Interact, `can_show_homeless_text` | g58 |
| Shop chrome (`Main.cs:41790-41797`) | NO (feeds `shop_mood` into the shop tuple) | YES — `ui/shop_happiness.rs` icon bands ≤0.82/≤1/≤1.1/else, P0 percent, offsets (26,98)/(12,108) off (504,258), four-way ring | g58 |
| Discount Card / hair-restyle price math | YES — `discount_equipped` writer + shop-latched `discount_available`, `hair_restyle_cost` f32-bit witnesses | NO (out of g58 scope) | f37 |

## 2. KEEP recommendation

**f37's `process_mood` core + g58's report-builder/UI feeding from it.** Concretely:

KEEP from f37 (the whole `process_mood` body and its helpers):
- `process_mood(world, all_npcs, npc, player_center, raining, rng)` — loveStruck
  head, remix return, Princess branches + RNG loves, Tavernkeep-counting crowd
  roster, ungated traits, player-zone judge (`shopping_zones`/`biome_in`),
  `affection_rank` best-row pick, `round_half_to_even` cents tail.
- `MoodFlag::HateLonely`, `MoodFlag::NpcAffection(AffectionLevel, i32)` (vanilla
  id — the Princess picks and the unported-kind rows need it).
- The capture-once plumbing: `App::shop_mood`, the talk-open latch, the
  `talk_npc != Some(idx)`-gated shop-open capture, the six NotInShop resets,
  `nurse_heal_cost()` reading the latch, the discount chain, `hair_restyle_cost`.

KEEP from g58 (data + presentation, adapted per §4):
- `biome_preferences_by_id` / `neighbor_preferences_by_id` — the full 25-row /
  25-case NPCID-keyed tables (strict supersets of f37's kind-keyed ones; row
  content identical where both carry it — spot-checked all 20 shared cases).
  f37's `biome_preferences(kind)` / `neighbor_preferences(kind)` become thin
  wrappers over the id tables keyed on `kind.net_id()`, RETURNING THE VERBATIM
  ROWS (no unported-neighbor drop — see DROP #4).
- `biome_name_key`, `biome_name`, `npc_full_name`, `can_show_homeless_text`,
  `prepend_report`, `happiness_report`, `ShoppingSettings` + `Default`
  (`NotInShop = {1f, ""}`).
- `MoodFlag::HateBiomeNamed(BiomeKind)` — adopt into the merged enum; f37's
  evil arm pushes it instead of the name-less marker.
- `App::set_talk_npc` seam — extended to run the f37 capture (§4).
- `ui/npc_chat.rs` Happiness button + rect math, `ui/shop_happiness.rs` chrome,
  the `UI/NPCHappiness` sheet load (`assets.rs`), the `draw.rs` Condition feed.
- g58's DATA witnesses (`w4_g58_tests` table rows, name keys, report prepend,
  neighbor naming, Princess key, homeless gate) and the 3 chrome witnesses —
  they pin the KEEP side and don't touch the mood math. The two kind-view
  witnesses re-pin per DROP #4.

## 3. DROP list (g58 side unless noted)

1. **g58's `process_mood` body** (`happiness.rs:305-473`): home-anchor judge,
   `classify_biome` first-match, the `adjustment < 1000.0` trait gate, the
   away-from-zero `.round()` tail, the missing loveStruck/remix/Princess arms,
   the Tavernkeep-excluding crowd filter. All replaced by the f37 core.
2. **g58's `classify_biome`** — deleted with the body; f37's
   `shopping_zones`/`biome_in` is the judge.
3. **g58's unconditional shop-open capture** (`world_sim/npcs.rs:992` on its
   side) — replace with f37's `if app.talk_npc != Some(idx)` gate; with the
   Princess RNG head live, an ungated second pass double-draws `Main.rand`.
4. **g58's filtered kind view** (`neighbor_preferences(NpcKind)` with the
   `OnceLock`/`Box::leak` unported-neighbor drop, `happiness.rs:220-269`) —
   contradicts f37's "ids stay verbatim, they never match until the kind lands"
   contract and hides the Princess's whole-roster picks. Replace with a direct
   `neighbor_preferences_by_id(kind.net_id())` wrapper. Re-pin
   `the_kind_view_filters_the_unported_neighbors` to the verbatim rows (Guide:
   4 rows incl. the 178 Dislike; TaxCollector: 5 incl. the 142 Hate).
5. **g58's `MoodFlag::NpcAffection(AffectionLevel, NpcKind)`** — the merged enum
   carries f37's `i32`; `happiness_report` adapts (§4).
6. **g58's per-call nurse re-scan** (`nurse_heal_cost(nurse_idx)` calling
   `process_mood`) — f37's latch-reading `nurse_heal_cost()` wins; the g58
   call-site signature must follow.
7. **f37 side, one item:** the name-less `MoodFlag::HateBiomeEvil` push in the
   evil arm — replaced by g58's `HateBiomeNamed(evil)` (first-hit
   Corruption→Crimson→Dungeon order is already identical). Keep the variant
   only if a legacy price-side caller needs it; re-pin f37's
   `the_evil_biome_prices_at_the_ceiling` to `HateBiomeNamed(Corruption)`.
8. **g58's `shopping_settings()` wrapper as a second entry point** — it calls
   `process_mood` rng-less; after the merge the ONLY capture sites are
   `set_talk_npc` and the gated shop-open (§4). Keep the function only as the
   struct-building helper the capture sites call WITH the rng.

## 4. Integration contract (g58 UI calls the f37 captured mood)

One capture, two consumers, zero re-scans:

```text
App::set_talk_npc(Some(i))                     // Player.cs:4383-4390
    mood = happiness::process_mood(world, npcs, &npcs[i],
                                   player.center(), world.raining,
                                   app.rngs.global())   // f37 core, ONLY RNG draw site
    app.shop_mood            = mood.multiplier          // f37 latch (prices, nurse, chrome)
    app.shopping_settings    = ShoppingSettings {
        price_adjustment: mood.multiplier,
        happiness_report: happiness_report(npcs[i].kind, &mood.flags, npcs),
    }                                                    // g58 report off the SAME flags
App::set_talk_npc(None)  → shop_mood = 1.0, shopping_settings = NotInShop
```

- **Capture sites (exhaustive):** `set_talk_npc` (all six latch sites route
  through it — g58's repointing stands) and `try_open_town_shop_at_cursor`'s
  collapsed-entry arm, gated `if app.talk_npc != Some(idx)` (f37). The gated
  arm runs the SAME capture block (multiplier + report). No other caller may
  invoke `process_mood` — the Princess's loves consume the global stream once
  per talk (`Main.OpenShop` `Main.cs:40459-40467` does not recompute).
- **Report builder adaptation:** `happiness_report(kind, flags, all_npcs)`
  keeps its signature; the `NpcAffection(level, id: i32)` arm resolves the
  category name through `lang_tables::NPC_ID` by id (as `prepend_report`
  already does for the subject) and the `{NPCName}` substitute through
  `npc_full_name` via `kind_from_net_id(id)` (unported id → the plain
  `Lang.GetNPCNameValue` type name, which `npc_full_name`'s no-resident arm
  already answers). The `LikeNPC_Princess` dedicated key gates on
  `level == Like && id == 663`. ADD a `MoodFlag::HateLonely` arm → the
  `HateLonely` key (`ShopHelper.cs:128`'s report call). The Princess's random
  loves are `NpcAffection(Love, id)` → the ordinary `LoveNPC` key — no special
  case needed.
- **Flag order = report order:** f37's push order (sentinels → crowd/LoveSpace
  → biome row → Princess loves / LikePrincess → switch rows) folds through
  g58's prepend unchanged — the LAST pushed row reads FIRST, matching the C#
  call order. f37's order witness
  (`the_guide_likes_a_nearby_princess_and_clothier`) pins it.
- **Shop chrome:** `ui::shop_happiness::draw(r, lookups, app.shop_mood, pulse)`
  — unchanged; it already rides the shop tuple's `shop_mood`. f37's banker's
  cents make the P0 ×100 integral, which g58's `price_percent_text` comment
  assumes — the assumption becomes true only with the f37 tail.
- **Chat button:** Condition = `!can_show_homeless_text(talk_npc) &&
  !shopping_settings.happiness_report.is_empty()`; Interact = sound 12 +
  `npc_chat_text = shopping_settings.happiness_report.clone()` — g58's code
  stands verbatim, now fed by the f37-derived flags.
- **Nurse:** `nurse_heal_cost()` reads `app.shop_mood` (f37) — never the report
  struct, never a re-scan.
- **Test fixtures:** the merged `forest_world()` keeps f37's
  `world_surface = 120.0` (the player-zone judge needs the player above the
  surface for the Forest row); g58's report/DATA witnesses don't scan zones and
  are fixture-independent. g58's two mood-chain witnesses
  (`the_forest_guide_alone_is_content`, `the_crowd_multiplies_past_three`) exist
  on both sides with the same expectations — keep f37's rng-passing copies.

## 5. g58b deviation census (against g58's recorded list; no g58b ref exists)

| Deviation (g58 ANALYSIS) | g58 status | f37 status | After the §2 merge |
| --- | --- | --- | --- |
| Princess RNG head (`AllPersonalitiesModifier.cs:15-36`) + HateLonely gate (`ShopHelper.cs:122-130`) | constants-off | **DONE** (`c939c97c`): rng param, type-dedup, remove-per-pick, stream-replay + HateLonely/no-crowd/no-LoveSpace witnesses | CLOSED by the f37 core |
| Zoologist shapeshift report suffix (`ShopHelper.cs:191-194`, `altTexture == 2`) | constants-off (no alt-texture state) | not addressed (f37 has no report builder) | **STILL OPEN** — needs the NPC alt-texture state (shapeshift lane); the merged `prepend_report` is the hook point: pass the subject NPC's alt flag and switch the category suffix there |
| Tavernkeep town twin (550 ports as the DD2 event Bartender) | excluded from the crowd roster; DATA rows present but the kind view can't reach case 550 | **mood-side DONE**: the crowd scan counts `DD2Bartender` explicitly and `net_id()` = 550 (bestiary row), so his rows match; f37's kind-keyed tables carry his biome + case-550 rows | CLOSED for the mood math via the id-keyed wrapper (§2); a true town twin (housing/shop/spawn) remains its own lane |
| Home-anchor biome judge (M4-TOWN-2 convention) | kept (recorded deviation) | **DONE**: player-zone judge + single best-affection row + the Forest/BelowSurface getters (`Player.cs:3823-3847`), witnesses `the_biome_traits_judge_the_player_not_the_npc_home` / `the_best_affection_row_wins_over_the_lower_ones` | CLOSED by the f37 core; g58's `classify_biome` dropped |
| `ItemSlot.DrawSavings` coin pile (`Main.cs:41791`) | deferred to the coin-slot lane | out of scope | STILL OPEN (coin-slot lane) |
| ≤0.82 achievement (`Player.cs:4391-4394`), portrait hop, corner item | deferred (achievement / S1-E-10b lanes) | out of scope | STILL OPEN (their lanes) |
| Pets' `housingCategory` filter (`TownRoomManager.cs:153-161`) | inert (no pets ported) | same note | unchanged — no-op until a category-1 kind ships |

Net: of g58's four mood-math deviations, **three are already fixed on f37**
(Princess head, Tavernkeep crowd count, home-anchor judge) and the fourth
(Zoologist suffix) is report-side and stays open with a named hook point. The
merge is therefore "f37 core wins wholesale, g58 contributes data + report +
UI", not a line-by-line interleave.

## 6. Conflict-file guidance for the merger

| File | Resolution |
| --- | --- |
| `crates/tr-app/src/happiness.rs` | f37 core + g58 id-tables/report per §2-4; merged enum = f37's + `HateBiomeNamed` (− name-less push); both witness sets kept minus the re-pins named in §3 |
| `crates/tr-app/src/app/mod.rs` | BOTH fields: f37 `shop_mood` + g58 `shopping_settings`; g58 `set_talk_npc` extended per §4; f37 `nurse_heal_cost()` signature wins |
| `crates/tr-app/src/world_sim/npcs.rs` | f37's gated captures, extended to also build the report (§4); g58's `set_talk_npc` repointing of the close arm kept |
| `crates/tr-app/src/app/clicks.rs` | g58's Happiness-button block + f37's `shop_mood = 1.0` close reset (or route the close through `set_talk_npc(None)` which does both) |
| `crates/tr-app/src/recall_teleport.rs`, `chest.rs` | g58's `set_talk_npc(None)` repointing wins (it subsumes f37's manual `shop_mood = 1.0` once the seam resets both) |
| `crates/tr-app/src/app/frame/draw.rs` | g58's chat Condition argument + f37's shop tuple — disjoint hunks, keep both |
| `crates/tr-app/src/ui/npc_chat.rs`, `ui/shop_happiness.rs`, `ui/mod.rs`, `assets.rs` | g58-only, take as-is |
| `crates/tr-app/src/player.rs`, `shop.rs`, `world_sim/player.rs` | f37-only, take as-is |
| `anchors.txt` | keep ALL six rows (3 f37 + 3 g58); the g58 `process_mood`-adjacent markers (`pub fn biome_preferences_by_id`, `pub(crate) fn icon_frame`, `pub(crate) fn set_talk_npc`) and f37's (`pub fn process_mood`, `pub discount_equipped: bool`, `pub fn hair_restyle_cost`) all survive §2 |
| `docs/ANALYSIS.md`, `docs/PORT_MAP.md` | keep both sections/rows; add one line to each noting this reconcile (the g58 "constants-off" Princess/home-anchor entries are retired by the f37 core) |

Verification for the merger (this lane ran none — code-only): the catalog
command `cargo test --workspace -- --skip migrates_version_2_kinds_to_item_ids`
after the resolution, with attention to `happiness.rs` both witness modules,
`npc.rs`'s six `biome_preferences` callers (unchanged API), and the nurse-cost
tests on both sides.
