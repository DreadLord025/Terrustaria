# 22 — Chests / loot / shops / shopping

> Wave 2 · researched 2026-11-12 · sources: decompiled + crates + ROADMAP/ANALYSIS/PORT_MAP

## Scope
The container half of the object system (`Terraria/Chest.cs`, 3 600 lines), the
world-gen chest stocking block (`WorldGen.cs` `AddBuriedChest` family), the NPC
drop-rule engine (`Terraria/GameContent/ItemDropRules/`, 37 files), the shop
tables (`Chest.SetupShop` / `SetupTravelShop_*`) and the price maths
(`Player.GetItemExpectedPrice`). Rust side: `crates/tr-core/src/loot.rs`
(2 035 lines, the rule engine), `crates/tr-app/src/loot_db_gen.rs`
(1 439 lines, generated) + `loot_db.rs` (154), `crates/tr-app/src/chest_gen.rs`
(2 673 lines, world-gen stocking), `crates/tr-app/src/chest.rs` (3 277 lines, the
`Chest` entity + UI glue), `crates/tr-core/src/wld/sections/chests.rs`
(445 lines, the `.wld` codec), `crates/tr-app/src/shop.rs` (1 535 lines) and
`crates/tr-app/src/ui/chest.rs` (865).

## Key entities
- `Chest` (`Chest.cs:18`) — the container record: `maxItems` (:68), `item[]`
  (:72), readonly `x/y/index` (:74-78), `bankChest` (:80), `name` (:82),
  `frameCounter`/`frame` (:84-87), `eatingAnimationTime` (:88). Constants
  `chestStackRange = 600f` (:50), `maxChestTypes = 52` (:52),
  `maxChestTypes2 = 38` (:56), `maxDresserTypes = 65` (:60),
  `DefaultMaxItems = 40` (:64), `AbsoluteMaxItems = 200` (:66),
  `MaxNameLength = 20` (:70).
- `Chest.Initialize` (`Chest.cs:229-258`) — builds `chestTypeToIcon` /
  `chestTypeToIcon2` / `dresserTypeToIcon` from `WorldGen.GetItemDrop_Chests(
  style, secondType)` / `GetItemDrop_Dressers`, then the hard overrides
  `[2]=327, [4]=329, [23..27]=1533..1537, [36]=[38]=[40]=327`, `array2[13]=4714`
  — i.e. the icon of a chest style *is* that style's main loot item.
- `Chest.cs` lifecycle: `CreateWorldChest` (:104), `Resize` (:118),
  `CreateBank` (:142), `CreateShop` (:147), `CreateOutOfArray` (:152),
  `IsLockedOrInUse` (:178), `Lock`/`Unlock` (:346/:428), `FindChest` (:513),
  `FindEmptyChest` (:522), `AfterPlacement_Hook` (:559), `CreateChest` (:590),
  `DestroyChest(Direct)` (:625/:647), `UpdateChestFrames` (:3603),
  `FixLoadedData` (:3664).
- `WorldGen.AddBuriedChest` (`WorldGen.cs:35877`, overloads :35741/:35871) — the
  world-gen stocking block: up to `15` tries (`×3` under
  `tenthAnniversaryWorldGen`), the depth-band style roll `num9`, the pre-set main
  item `num10`, the `SecretSeed.errorWorld` style override (:35986-36000), the
  `IsChestRigged` gate (:35746) and the ~1 700 lines of per-band/per-style item
  rolls (the flag families `flag2..flag13`, the vanilla fairness flags).
- `WorldGen.GetNextJungleChestItem` (:10170), `GetDungeonLootAndChestStyle`
  (:35800), `GetItemDrop_Chests(style, secondType)` (:41554 — the style→main-item
  switch), `GetItemDrop_FakeChests` (:41841), `PlaceChest(Direct)` (:57813/
  :57848), `CheckChest` (:57923), `GetChestIcon` (:57989).
- Chest persistence — **not in `Chest.cs`**: `Main.SaveChests` (~:1650-1673) and
  `Main.LoadChests` (~:2723-2780); a slot's wire form is a bare `i16 0` when
  empty/banned, else `stack` then `(type, prefix)` only when `stack > 0`.
- `ItemDropRules/` (37 files) — `IItemDropRule`/`IItemDropRuleChainAttempt`,
  `LeadingConditionRule`, `CommonDrop(+WithRerolls)`, `OneFromOptionsDrop(+…Not
  ScaledWithLuck)`, `ItemDropWithCondition`, `DropBasedOn{Expert,Master,MasterAnd
  Expert}Mode`, `DropLocalPerClientAndResetsNPCMoneyTo0`, `DropOneByOne`,
  `DropBasedOnExtraGel`, `SlimeBodyItemDropRule`, `StatueMimicItemDropRule`,
  `MechBossSpawnersDropRule`, `FromOptionsWithoutRepeatsDropRule`,
  `DropPerPlayerOnThePlayer`, `OneFromRulesRule`, `DropNothing`, `Conditions`,
  `ItemDropDatabase`, `ItemDropResolver`, `DropRateInfo(+ChainFeed)`, `Chains`,
  `CommonDrop(Not)ScalingWithLuck`.
- `Chest.SetupShop(int type)` (`Chest.cs:1360-3544`, ~2 180 lines — the switch
  keys on the *shop index*, not the NPC id) + `AddItemToShop` (:670, the
  `buyOnce` stack merge at :684/:701) + the `SetupTravelShop_*` family
  (:708/:797/:821/:956/:984/:1275).
- `Player.GetItemExpectedPrice` (`Player.cs:35359-35398`) — the price formula
  (store value, the 0.8 discount, the `buyOnce` `/5` and the `PriceAdjustment`).
- Rust: `crates/tr-core/src/loot.rs::{DropRng, DropAttemptInfo, LootFlags,
  LootNpc, DropRequest, DropAttemptResult, ChainKind, ChainRule, resolve_rule,
  ItemDropDatabase, try_dropping, roll_luck..roll_only_bad_luck_extreme,
  drop_money, DropNothing, CommonDrop, CommonDropWithRerolls, OneFromOptionsDrop,
  ItemDropWithCondition, DropBasedOnExpertMode, DropBasedOnMasterMode,
  DropLocalPerClientAndResetsMoney, DropOneByOne, DropBasedOnExtraGel, Cond,
  LeadingConditionRule, DropPerPlayerOnThePlayer, OneFromRulesRule}` (+L14-1 870).
- `crates/tr-app/src/chest_gen.rs::{stock_band, stock_chest, Context,
  DungeonCycle, JungleCycle, shuffle_hell_chest, context_at, roll_prefix_on,
  give_item_good_prefixes, generate_world_chests}` (the load-time seam).
- `crates/tr-app/src/shop.rs::{ShopFlags, ShopEntry, StockWhen, setup_shop,
  roll_int, roll_luck, setup_travel_shop, adjust_slot_rarities,
  can_add_item_to_shop, add_to_shop, get_travel_item, get_travel_painting,
  resolve_stock, resolve_stock_moon, buy_price, sell_price, reforge_cost,
  round_half_to_even}`.

## Dependencies
- Calls into: `World`/`App` state, `tr_core::loot` (rule engine),
  `tile_light::FastRandom` (the seeded RNG), `prefix::roll_prefix`, `ITEMS`
  defaults, `World.chests`/`App.chest` and the `.wld` codec
  (`wld::sections::chests`), `ui/chest.rs` + `ui/slot.rs` (the chest/shop UI),
  `app/clicks.rs` (ctx-3/10 clicks), `world_sim/{npcs,editing,player}.rs`
  (`spawn_loot`, `try_open_shop_at_cursor`, `tick_open`), `town.rs`
  (`happiness::process_mood`), `banners.rs`, `netglue.rs`.
- Called by: worldgen pipeline (chest placement) → `generate_world_chests` at
  `App::new`; the NPC death pass (`spawn_loot`, `spawn_coins`); the chest UI and
  the shop open path; save/load (`saveglue::save_world`).
- Shared data structures: `ItemSlot`, `WorldChest`, `LootFlags`/`LootNpc`
  (the `Conditions` snapshot), `ItemDropDatabase` (net-id → rules).

## Port status
| C# subsystem | Status | Rust location | Evidence / notes |
| --- | --- | --- | --- |
| `Chest` struct + lifecycles | PORTED | `chest.rs` (3 277) | Store over `App.world_chests`, `IsLockedOrInUse`, `Lock`/`Unlock`, `FindChest`, rename flow — cited in the M3-CHEST-UI row (`Chest.cs:522-533`, `:178-188`, `:346-495`); lazy entries, no placement hook (documented). |
| `.wld` chest section | PORTED | `wld/sections/chests.rs` | `read`/`payload` mirror `SaveChests:1650-1673` / `LoadChests:2723-2780`; the 200-item `Resize` ceiling and the 8 000-chest cap are *refused* rather than trusted. |
| load fix-ups (`FixLoadedData`, `FixAgainstExploit`, dressers) | PORTED | `chest.rs::from_wld`, `fix_dresser_chests` | `chest.rs:383-388` (`FixDresserChests:3269-3282`, `<115` only), `:264`. |
| world-gen chest stocking | PARTIAL | `chest_gen.rs` | `AddBuriedChest` :35871-37580 ported with verbatim odds, but as a **load-time** pass (below), so the RNG order differs from vanilla. |
| chest icons (`chestTypeToIcon/2/dresserTypeToIcon`) | MISSING | — | No Rust symbol (`Chest.cs:54-62`, `Initialize:229-258`); the UI needs a style→icon-item table. |
| drop-rule engine | PORTED | `tr-core/src/loot.rs` | All 22 rule-like classes above; 18 engine tests on seeded `FastRandom` streams. |
| drop database population | PARTIAL | `loot_db_gen.rs` | 1 421 registrations over the 643 C# register sites (generated, C# order, net-id families); 4 rule classes + 2 conditions skipped (below). |
| money roll chain | PORTED | `loot.rs::drop_money` | `NPC.cs:79619-80106`: midas, the +20-75 % spread, the six bonus chains, the blood-moon bump, the two-roll best/worst, the coin split. |
| `SetupShop` + `SetupTravelShop` | PARTIAL | `shop.rs` | Merchant/travel/dye/golf arms ported with the moon-phase gates; see the shop table below. |
| price maths | PARTIAL | `shop.rs::buy_price/sell_price` | The 0.8 discount, the `buyOnce` `/5` and `shopSpecialCurrency` are out (see risks). |

### Chest / container coverage
| C# container family | C# location | Rust | Status |
| --- | --- | --- | --- |
| tile 21 wooden chest, 52 styles | `Chest.cs:52` | `chest.rs`, `chest_gen.rs` (kind 21 gate) | PORTED (style decoded off `frame_x / 18`) |
| tile 467 mahogany (chest2, 38 styles) | `Chest.cs:56` | kind 467/341 arms in `chest_gen.rs` | PARTIAL (only the rows the vanilla 467 block repeats) |
| tile 341 golden / underwater chests | `Chest.cs:52` | `chest_gen.rs` anchors | PARTIAL (wave-wall/skyisland dispatchers out) |
| tile 88 dressers (65 styles) | `Chest.cs:60` | `chest.rs::fix_dresser_chests`, `default_chest_name` | PORTED (fix-up + name) |
| banks (piggy/safe/defender/void) | `Chest.cs:142` | `bank.rs`, `player.chest` | PORTED (M3-STORE-1) |
| pseudo-chest (shop) | `Chest.cs:147` + `SetupShop` | `shop.rs::setup_shop` | PARTIAL |
| rigid/dungeon/locked chests | `Unlock`/`Lock` :346/:428 | `chest.rs` lock decode; rigged arm out | PARTIAL |

### Drop-rule coverage
Ported in `loot.rs`: `DropNothing` (L554), `CommonDrop` (L582),
`CommonDropWithRerolls` (L669), `OneFromOptionsDrop` (L723),
`ItemDropWithCondition` (L775), `DropBasedOnExpertMode` (L843),
`DropBasedOnMasterMode` (L897), `DropLocalPerClientAndResetsMoney` (L952),
`DropOneByOne` (L1039), `DropBasedOnExtraGel` (L1091), `LeadingConditionRule`
(L1590), `DropPerPlayerOnThePlayer` (L1631), `OneFromRulesRule` (L1692), the
chain sugar `chain_on_{success,failed_roll,failed_conditions}` (L1184-1210),
`Cond` (L1350, ~50 `Conditions.*` gates) and the luck variants
`NotScalingWithLuck` / `ScalingWithOnlyBadLuck` (L1797-1856).
**Skipped (13 rows, warned in the M4-LOOT-2 row)**: `SlimeBodyItemDropRule`,
`StatueMimicItemDropRule`, `MechBossSpawnersDropRule`,
`FromOptionsWithoutRepeatsDropRule` and the `Conditions` entries
`NamedNPC`/`RedHatSkeletron`. Also out: `DropRateInfo` +
`ItemDropResolver.ReportDroprates` / `DropRateInfoChainFeed` (bestiary side,
waits M4-BEST-1) and `CustomCurrencyManager` (special shop currency).

### Shop coverage
| C# shop block | C# location | Rust | Status |
| --- | --- | --- | --- |
| Merchant base rows (88/87/35/1991/3509/3506/8/28/110/40/42/965/1786) | `Chest.cs:1371-1483` | `shop.rs` (test-pinned) | PORTED |
| stock condition gates (`StockWhen`, 25 kinds) | `Chest.cs:1360-3544` | `shop.rs:1176+43-1250` | PORTED, two gates hard-false |
| travel shop | `Chest.cs:708-1275` | `setup_travel_shop`, `get_travel_item/painting`, `adjust_slot_rarities` | PORTED |
| prices | `Player.cs:35359-35398` | `buy_price`/`sell_price`/`reforge_cost` | PARTIAL |
| happiness multiplier | `ShopHelper.ProcessMood` | `town.rs::process_mood` | PORTED (M4-TOWN-2) |
| skeleton-merchant curve / sellback helper | `ItemShopSellbackHelper.cs` | — | MISSING |

## Tests
Rust unit tests (`#[test]`) in this module — all seeded, no cargo run for this
research:
- `chest.rs` — 61 tests (1586-3417): `default_chest_name`/`dresser` names
  (1808-1810), the `<115` dresser stream (`chest.rs:3352`, DW-3.4), the lock
  decode, transfer/favorite flows.
- `wld/sections/chests.rs` — 9 tests (257-441): the slot wire shapes, the
  per-chest `maxItems`, the pre-294 shared-count path, the banned-arm, the
  8 000 cap, the round trips.
- `chest_gen.rs` — 28 tests (1486-2721): surface/deep stock, the anchor-only
  gate, the determinism pair, the pyramid/desert/ice/hell/skyisland dispatchers,
  the style probes.
- `loot.rs` — 18 tests (1885-2161): `common_drop_gate_and_stack_range`,
  `always_drop_rule_drops_the_exact_stack`, `failed_roll_fires_the_on_failed
  _roll_chain`, `success_fires_the_on_success_chain`, `expert_mode_routes_to_the
  _expert_rule`, `condition_gate_splits_fail_states`, `rerolls_zero_matches_the
  _plain_common_drop_on_the_same_seed`, `drop_one_by_one_emits_the_chunk_shape`,
  `gel_factory_doubles_stacks_under_the_extra_gel_seed`,
  `boss_bag_requests_the_money_reset`, `database_registers_and_removes_per_net_id`,
  `roll_luck_luck_zero_is_the_plain_roll`, the four `drop_money` cases,
  `try_dropping_walks_every_registered_rule`.
- `shop.rs` — 11 tests (1303-1535): `the_merchant_stock_carries_the_base_rows`,
  `the_prices_ride_the_happiness_adjustment` (buy 1.0/0.75/1.5/0.89 →
  40000/30000/60000/35600 for item 88; sell 8000/10667),
  `the_buy_price_matches_the_happiness_multiplier`, `the_stock_conditions_cover_
  the_merchant_gates`, the travel-shop index-19 and vampirism rows.
- `ui/chest.rs` — 13 tests (619-887): button anchors, slot window, scroll clamp.

## Deviations & risks
1. **Chest loot is a load-time pass, not a worldgen pass** (determinism, and a
   *content* divergence, not just an ordering one). The C# stocks a chest inside
   `AddBuriedChest` on `WorldGen.genRand`, interleaved with every other worldgen
   consumer; `chest_gen.rs` re-derives the same rows at `App::new` from a
   world-seeded splitmix stream, one stream per chest. Determinism *within the
   port* holds (the M5-CHEST-gen determinism test), but the port's chest contents
   cannot equal vanilla's for the same seed, and the vanilla's cross-chest
   fairness (`generatedShadowKey`, `generatedRamRune`, `gennedLivingMahoganyWands`)
   is approximated per-chest. Admitted in the M5-CHEST-gen row.
2. **Port's worldgen chest rows out of scope** (behavioural): the seed-flavour
   rows (vampire/error/rainbow/tenthAnniversary), the remix variants (desert
   `num9` style rows, ice `1319 → 725`, tenthAnniversary `0 → 1`), the
   wave-wall/skyisland/pyramid dispatchers and the `GenVars` residual state
   ("the hive bounds die with the worldgen"). Admitted in the same row.
3. **Buy price formula is incomplete** (behavioural). `GetItemExpectedPrice`
   `:35380-35397`: a `buyOnce` item divides *both* prices by `PriceAdjustment`
   and then `/5` with a `storeValue != 0 && < 1 → 1` clamp. `shop.rs::buy_price`
   (`L1257-1259`) only multiplies by the adjustment, so every `buyOnce` row in
   the Rust costs **5× the vanilla**. `sell_price` (`L1282-1284`) conversely
   divides every item by 5, while the vanilla's `calcForSelling` is `storeValue`
   (only divided by `PriceAdjustment`); the vanilla `/5` appears in the tooltip
   path `Main.cs:20454-20471`. Severity behavioural (shop economy). Not admitted
   anywhere in ROADMAP as a gap — only the "no discount accessory yet" note is.
4. **`discountAvailable` and `shopSpecialCurrency` are missing** (behavioural):
   the 0.8 Discount Card factor (`:35369-35374`) and the `calcForSelling =
   calcForBuying` aliasing for `isAShopItem` are not modelled at all.
5. **Two stock conditions hard-false** (behavioural, small): **CLOSED
   2026-11-13** (`S1-F-16-chest-icons-stock-when`): the gates read the World
   flags through `ShopFlags` (`downed_slime_king`/`downed_queen_slime`/
   `downed_moon_lord`; the Queen Slime flag landed on `World` + the `.trwld`
   v15 bit per `WorldFile.cs:1396`); the Princess's 5266/5044 rows resolve.
6. **Four rule classes + two conditions missing** (behavioural): the boss-side
   rules `SlimeBody`/`StatueMimic`/`MechBossSpawners`/`FromOptionsWithoutRepeats`
   and `NamedNPC`/`RedHatSkeletron` — 13 rows, warned in M4-LOOT-2. Unknown item
   ids also only warn.
7. **Bestiary drop-rate reporting** (`ReportDroprates`,
   `DropRateInfoChainFeed`) is unported — cosmetic/UI, waits on M4-BEST-1.
8. **Chest icon tables unported** (cosmetic): `chestTypeToIcon`,
   `chestTypeToIcon2`, `dresserTypeToIcon` have no Rust twin; any UI that shows a
   chest style's item icon must build its own table.
9. **Known chest-side approximations from M3-CHEST-UI** (mostly cosmetic): no lid
   animation (`UpdateChestFrames`/`eatingAnimationTime`), the rigged-chest arm
   out (a chest-trap never triggers), favourites are stateless, one global idle
   open state, and the `plant_boss`-gated `Lock/Unlock` call sites always pass
   `true`. Admitted in ANALYSIS "Chest UI + world chests (M3-CHEST-UI)".
10. **Price rounding** uses `round_half_to_even` (`shop.rs:1289`) for the C#
    banker's `Math.Round` — correct for the f32 values tested, but the C# casts
    through `(int)Math.Round(float)` on *longs* in some branches; a `f64` audit
    is cheap insurance. Cosmetic unless a price lands on a half.

## Task seeds
- **L — chest-gen determinism study**: decide whether `AddBuriedChest` must move
  onto the worldgen stream (per-pass `pass_rng`) or keep the load-time seam, then
  pin a golden fixture (the worldgen FNV pattern). Blocks any claim of
  vanilla-identical chest contents. Depends on nothing; gates 5-CHEST-gen's
  closure row.
- **M — port `GetItemExpectedPrice` fully**: the 0.8 discount, the `buyOnce`
  `/5` + min-1 clamp, `GetStoreValue`, `isAShopItem` aliasing,
  `shopSpecialCurrency`; wire the real buy/sell click path, not only tooltips;
  test the buyOnce rows against vanilla numbers. Depends on the accessory/
  currency work being far enough along.
- **M — the 13 skipped loot rows**: `SlimeBody`, `StatueMimic`,
  `MechBossSpawners`, `FromOptionsWithoutRepeats` rules + `NamedNPC` /
  `RedHatSkeletron` conditions; regenerate through `tools/gen_loot.py`.
  Depends on the rule-engine work only.
- **M — shop-index coverage audit**: enumerate every `case` of
  `Chest.SetupShop` (`Chest.cs:1360-3544`, ~2 180 lines) against
  `shop.rs::setup_shop` (`:205-611`) and list the missing shops; the row counts
  alone (`setup_shop` ≈ 406 lines) suggest partial coverage beyond the rows the
  tests pin.
- **S — chest icon tables**: `chestTypeToIcon` / `chestTypeToIcon2` /
  `dresserTypeToIcon` (+ `Initialize`'s overrides) for the chest UI. Depends on
  `GetItemDrop_Dressers` being ported.
- **S — the two dead stock gates**: `HardModeMoonlord` / `DownedBothSlimes`
  (`shop.rs:1268-1270`) once `downedMoonlord`/`downedQueenSlime` reach `World`. —
  **DONE 2026-11-13** (`S1-F-16-chest-icons-stock-while` typo guard: the branch is
  `S1-F-16-chest-icons-stock-when`).
- **S — bestiary drop rates**: `DropRateInfo` + `ReportDroprates`
  (`DropRateInfoChainFeed`) behind M4-BEST-1.
- **S — price rounding audit** (f64 vs f32) if a half-value price shows up in
  play.
