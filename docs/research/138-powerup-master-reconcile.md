# 138 — POWERUP MASTER RECONCILE (a6 ⨉ g87/g88/g89/g90 ⨉ c36/f33 ⨉ f44/f45)

**Status: SEALED (seal pass, §12 — over the FINAL-FINAL retire pass §11).
The `.trwld` union authority is g71d `f2d8c2c67` research/92 §1-FULL's
107-byte head (§12.1 re-aligns §11.2's 101 and corrects a6's own seats);
the Success-leg activation contract is §12.2; the family bible §12.3.**
Written by W4-a6 reconcile
(branch `task/W4-a6-crystals-powerups`, base `8ed96ee5`); the FINAL pass
(§10) re-adjudicated the pushed g87 `dfbcaf94b` arm-by-arm. The RETIRE
pass (§11, over `115ce655a`) receipts the NOW-LANDED g88 fold 7
(`a9f1c29d8` — the six-line v7 rule IN CODE + the v11 torch line; §6's
re-flag RETIRED), aligns §5a's union to the f44 FINAL master map
(`bdd1cc79a`, research/132 §3-L3's 97-byte-head claim — corrected to
**98 + g87's 3 = 101-byte head + W×H dense**, §11.2's delta; the seal
pass §12.1 re-aligns the 101 to the LANDED **107** — moon five +
underworld one — under g71d's 92 §1-FULL authority), cross-
receipts the g87 closeout (`26620538e`, local over `dfbcaf94b` — the
"g87b-143" the Lead named exists as NO pushed ref and NO research/143
doc; the closeout folds all three a6 deltas, §11.3) and lands a6's
exchange-fail chat fold (§11.3). This doc sits BESIDE the
persist contracts — research/99 + 132 (`.plr`, f44), research/103 v10 + 122
(→ 103 FINAL v11, g88), research/135 (`.trwld`/`.trplayer` fold-input, g90),
research/92 §1 (`.trwld` union, g70×g71) — and adjudicates the **use-arm /
consumer keep-one** per item plus the **persist-layer split** and the exact
**wld-offset collisions**. C# re-read is the tie-breaker (golden rule).
Code-only lane: no cargo/tests/rustfmt/graphify; witnesses written, not run.

This pass ALSO landed five fidelity fixes on a6 where a claimant was more
faithful (§3, each cited). a6's own use arms were never less faithful than a
claimant's for the same C# body — the gaps were in the CONSUMERS and the
achievement dispatch, now closed.

## 0. Inputs read at probe (2026-11-21, fetched tips)

| Ref | Tip read | What it owns |
| --- | --- | --- |
| `task/W4-a6-crystals-powerups` (this) | `8ed96ee5` + this reconcile | ALL use arms `powerups.rs` (29/109/1291/3335/5289/5326/4382/5336/5043/5343/4829-30-4910/5337-42/5451/5738) + consumers + `.wld` book2/satchel/bought-pets bytes; NO `.plr`/`.trplayer` writer |
| `origin/task/W4-g87-demon-heart-use` | `dfbcaf94b` (FINAL probe, §10) | NOW PUSHED (was zero commits at the reconcile probe): 3335/5289/4829-30-4910 use arms (`editing/permanent_upgrades.rs`, 566 lines) + `.trplayer` v7 `dh` line + `.trwld` v18 3-byte pet tail + `.wld` bought-pet flips + `World.bought_*` fields. **No `g87b-143` branch exists on origin** (`git branch -r` census). Re-adjudicated §10 |
| `origin/task/W4-g88-life-fruit-use` | `a9f1c29d8` (fold 7 FINAL, v11 — retire probe §11.1; was `c8985b534` fold 6 at the FINAL probe) | 29/109/1291 use arms (`crystals.rs`/`life_fruit.rs`) + `.trplayer` **v11 SUPERSET** writer/reader + research/103 FINAL + 122; crystal flags tokens 4/5/6 (v10), the f33 torch pair as the LAST post-dye line (v11), the 135 §2 six-line v7 rule + the f33 lone-pair v7 rule LANDED |
| `origin/task/W4-g89-aegis-fruit-use` | `598642563` (g89f) | 5338/5341/5342 use arms + 4382/5336 combat-book arms (g89e) + `.plr` fruit/gummy/ambrosia bools + `.wld` book2 byte + `.trplayer` v9 lineage + research/122; g89f adds Rod of Discord/Harmony (out of scope) |
| `origin/task/W4-g90-artisan-bread-use` | `94d33f2ca` (reconcile §7 addendum over `b3b82d071`; retire probe) | 5326 loaf + 5337/5339/5340/5341/5342 use arms + 4382/5336/5343 + consumers (regen `aegis_crystal`/`arcane_crystal` slots, pearl mirror, skeleton-merchant shop) + `.plr` bools 1/3/4/5/6 + bread + `.trwld` v18 2-byte tail + net sync + research/135; `94d33f2ca`'s §7 cross-note read a6 at `8ed96ee5` (138 NOT yet pushed then — its keep-one proposal predates this doc's §1 verdicts; §11.4 cross-receipts) |
| `origin/task/W4-c36-achievement-notify-save` | `585fa0a84` | 5043 use-credit (`complete_condition` GAIN_TORCH_GODS_FAVOR) + 29/109/1291 `use_permanent_powerup` + the row-targeted `complete_condition` helper + research/30 |
| `origin/task/W4-f33-torch-god-favor` | `333479e0b` | the Torch God EVENT machine (`torch_god.rs`) + the 5043 flag READS (`has_torch_gods_favor` scan, `ctx.unlocked_biome_torches`); completes NO achievement |
| `origin/task/W4-f44-save-plr-2` | `bdd1cc79a` (FINAL docs; retire probe §11.2 — was `05e4020bb` at the reconcile probe, `63204f852` the slot-11 property fix between) | the COMPLETE `.plr` walk (research/99, 71 rows) incl. `used_upgrades[6]` array + `extra_accessory` + torch pair + bread + dd2 + counters; research/132 the persist MASTER INDEX (FINAL) — its §3-L3 publishes the seven-claimant **97-byte-head** `.trwld` union claim (§11.2 corrects the arithmetic to 98 and seats g87's pets → 101) |
| `origin/task/W4-f45-wld-header-2` | `6352d3840` | the COMPLETE `.wld` flags block (pets 2021-23, book2 2036, satchel 2037, nurse, slime green/old, …) + a `.trwld` **v18 44-byte identity block** |
| `origin/task/W4-g70-bg-layers-1` / `g71-bg-layers-2` | (92 §1 at the probe; g71d **`f2d8c2c67`** at the seal, §12) | `.trwld` v18 forest 16B / surface 3B → the 19-byte union at the probe; g71d's 92 §1-FULL now publishes the 13-claimant **107-byte** union + the fired underworld byte (§12.1 — THE merger authority) |
| `origin/task/W4-f43-town-spawn-2` | **`79a037f5e`** (seal probe, §12.2) | the town-spawn census tail (`town_arrival_pets.rs`, UNWIRED): the pet arrival gates + the FULL exchange chain as pure fns (reroll/profiles/names) — the Success-leg activation consumer |
| `origin/task/W4-d43-npc-kinds-631-697` | = main **`a725acb04`** (zero own commits, seal census §12.2) | the town-pet KINDS lane (637/638/656 unclaimed — no `net_id` arm on main); the Success leg's step-1 dependency |

**Census (ItemID.cs, verified):** 29 LifeCrystal / 109 ManaCrystal / 1291
LifeFruit (`:4265`) / 3335 DemonHeart / 5289 MinecartPowerup / 5326
ArtisanLoaf / 4382 CombatBook / 5336 CombatBookVolumeTwo (`:12355`) / 5043
TorchGodsFavor / 5343 PeddlersSatchel / 4829-4830-4910 PetLicenses /
5337 AegisCrystal / 5338 AegisFruit / 5339 ArcaneCrystal / 5340 GalaxyPearl
/ 5341 GummyWorm / 5342 Ambrosia (`:12357-12367`) / 5451 JimsDrone / 5738
RemoteControlCar. **4383 is DesertTorch, NOT a powerup** (g89e/g90 census).

## 1. Per-item adjudication table

The C# call row is `ItemCheck_OwnerOnlyCode` `Player.cs:43592-43608`:
TryDestroyingDrones `:43592` → UseLifeCrystal `:43595` → UseLifeFruit
`:43596` → UseManaCrystal `:43597` → UseDemonHeart `:43598` →
UseMinecartPowerUp `:43599` → UseTorchGodsFavor `:43600` → UseArtisanLoaf
`:43601` → UseEventItems `:43602` → UseBossSpawners `:43603` →
UseCombatBook `:43604` → UsePeddlersSatchel `:43605` → UsePetLicenses
`:43606` → UseShimmerPermanentItems `:43607`. Every consumable arm gates
`itemAnimation > 0 && ItemTimeIsZero` (mid-swing), `ApplyItemTime` re-arms,
the generic tail `:43280-43326` pays ONE only when `itemTime == itemTimeMax`
(a refused use never consumes).

| Item(s) | C# arm | Claimants | a6-vs-claimant fidelity (C# re-read) | KEEP verdict | Persist split |
| --- | --- | --- | --- | --- | --- |
| **29 / 109 / 1291** Life/Mana Crystal, Life Fruit | `:45306-45320` / `:45274-45288` / `:45290-45304` | a6, g88, c36 | All three port the same RAW +20/+20/+20 (or +5) to base/effective/live, HealEffect/ManaEffect, the cap gate on the PERMANENT base, event 0 no-op / 1 STAR_POWER / 2 GET_A_LIFE + TOPPED_OFF. a6's first pass used the `notify_custom("Use")` BROADCAST (less faithful); **fixed §3.1** to row-targeted. g88's `handle_special_event_life_mana` + c36's `complete_condition` are the faithful dispatch. No remaining fidelity gap. | **g88 keep-own** use arms (`crystals.rs`/`life_fruit.rs`, 103 §5a — the `.trplayer` carrier lives there); a6's three dups DROP at merge; c36's `use_permanent_powerup` DROP (its `max_hp`-gate variant is less faithful than the permanent-base gate). The shared `complete_condition` helper survives (§3.1). | `.plr` life/mana maxima: f44 rows 15/17 (pre-existing). `.trplayer` life pair line 9 / mana pair line 10: **g88 v10**. No `.wld`. |
| **3335** Demon Heart | `:45192-45200` + start refusal `:52051-52054` | a6, g87 (`dfbcaf94b`) | Both grant arms faithful (`!extraAccessory && expertMode` → flag once, ApplyItemTime 30, consume). **a6 MORE faithful at the START**: a6 ports the `CheckCanUse` refusal (`:52051-52054` — used/non-expert: no swing, no UseSound); g87 starts the animation unconditionally and lets the grant gate refuse (its own classic-mode witness shows the swing starts). Re-adjudication §10.1. | **a6** (`use_demon_heart` + `use_start_refused`); g87's arm DROP at merge (§10.1). | `.plr` `extraAccessory` `:54776`: **f44** (`extra_accessory`, L1-only per 132 §2a). g87's `.trplayer` v7 `dh` line **DROP** — version collision + L1-only ruling (§10.4). No `.wld`. |
| **5289** Minecart Powerup | `:45202-45214` | a6, g87 (`dfbcaf94b`) | **Both ports are NO-flag-gate ALWAYS-spawn** (`:45204` has no `!unlockedSuperCart` leg — every granted use re-arms ApplyItemTime, raises the flag once, spawns the 3353 Mech). a6's `quick_spawn_item` is the FULL `GetOrDropItem` walk: `GetItemSettings.PickupItemFromWorld` (`GetItemSettings.cs:57`, `CanGoIntoVoidVault: true`) → deposit walk + per-slot chime/notify (`:39491`/`:39505`) + void-vault LAST resort (`:39395-39398`) + overflow world drop with the pinned velocity triplet; the prefix roll rides **UpdatePlayers** (the use runs inside `Player.Update` → `Main.rand`; the Mech is non-prefixable so the `CanHavePrefixes` gate draws NOTHING either way, `Item.cs:515-518`). g87's walk is the bobber-catch simplification: `slot_for_pickup` only (no ammo/favorites/chime/notify/vault) and the roll on **UpdateItems** — the bobber's projectile-pass context misapplied to a player-context use (a recorded stream mis-attribution). Re-adjudication §10.2. | **a6** (`use_minecart_powerup` + `quick_spawn_item`); g87's arm + its `quick_spawn_item` twin DROP (§10.2). | `.plr` super-cart: **f44** (the <253 cart-derive, L1-only). g87's `dh` token 2 DROP (§10.4). No `.wld`. |
| **5326** Artisan Loaf | `:45216-45224` | a6, g90 | Both port `!ateArtisanBread` → flag once, ApplyItemTime, consume. Equal use-arm fidelity. **g90 owns MORE consumers**: the skeleton-merchant stock gate `Chest.cs:2958` (`ArtisanLoafWindow`, shop index 20) + the `.plr` bread slot; a6 wires only the reach flag (pre-existing `craft.rs`). | **g90 keep-own** (`use_artisan_loaf` + the skeleton-merchant shop consumer); a6's dup DROP at merge. The reach consumer (`craft.rs::tile_reach` `:35671`) is shared/pre-existing. | `.plr` `ateArtisanBread` `:54779`/`:55245` gate 256: **f44** (== g90's field, clean dedup). `.trplayer` bread line 11: **g88 v10** (g90 folded). No `.wld`. |
| **4382 / 5336** Combat Book + Vol Two | `:45322-45350` | a6, g89e, g90 | All three port the two independent WORLD-flag once-ever gates, ApplyItemTime 45, the `Misc.CombatBookUsed`/`Misc.CombatBookVolumeTwoUsed` ChatColors.World line (vol-two key is a `{$Misc.CombatBookUsed}` copy command), consume + TurnToAir. MP SendData(61,-11/-17) folds out. Equal fidelity (kinds disjoint from every other arm → press-edge position observation-identical). | **keep ONE** — recommend **a6** (`use_combat_book`, the single cluster seam) or g89e (pushed precedent); dedup the other two. The TRAINING_DAY progression-40 consumer (`:25695-25698`) lands once (§3.5, a6 now carries it; g89e/g90 also do). | `.wld` book2 byte tail **2036** gate 259: **f45** supersedes a6/g89e/g90 (§4). book1 tail 1939 gate 204 pre-existing (i35). `.trwld` book2: g90 v18 union offset **51** (§12.1 SEALED 107; §5a's 22 / §11.2's 55 superseded). NO sidecar token (world global, 122 §5). |
| **5043** Torch God's Favor | `:45226-45243` | a6, c36, f33 | **a6 MORE faithful than c36**: the C# sets `UsingBiomeTorches = true` whose SETTER writes `builderAccStatus[11] = 0` (`:3960-3963`); a6 does BOTH (`unlocked_biome_torches = true` + `builder_acc_status[11] = 0`), c36 folds UsingBiomeTorches into the unlocked flag (its research/30 deviation). a6 also ports HandleSpecialEvent(24) + the SP bestiary TorchGod(664) RegisterKill. **Fixed §3.1**: a6's credit now row-targeted `complete_condition("GAIN_TORCH_GODS_FAVOR","Use")` (== c36). f33 owns the EVENT machine + flag READS, completes no achievement. | **a6** use arm (the faithful setter) — **supersedes c36 research/30's "keep c36's write" note** (C# re-read `:3960-3963`). **c36**'s `complete_condition` helper is the shared dispatch (a6 now carries the identical helper, dedup). **f33** event machine + `has_torch_gods_favor`/`unlocked_biome_torches` reads. Three-way split by responsibility, no overlap to delete. | `.plr` torch pair `:54777-54778` gate 229: **f44** (`unlocked_biome_torches` + `using_biome_torches`, L1-only). NO sidecar. No `.wld`. |
| **5343** Peddler's Satchel | `:45368-45383` | a6, g90 | Both port the WORLD-flag once-ever gate, ApplyItemTime, `Misc.PeddlersSatchelUsed` World line, consume; the travel-shop +1 slot consumer `Chest.cs:1303-1306` (plain `num++`, NO RNG draw). Equal fidelity. | **keep ONE** — recommend **a6** (`use_peddlers_satchel`) or g90; dedup. The shop +1 consumer is identical on both (`shop.rs::setup_travel_shop`). | `.wld` satchel byte tail **2037** gate 260: **f45** supersedes a6/g90 (§4). `.trwld` satchel: g90 v18 union offset **52** (§12.1 SEALED 107; §5a's 23 / §11.2's 56 superseded). NO sidecar token. |
| **4829 / 4830 / 4910** Pet Licenses | `:45352-45365` / `:45438-45445` / `NPC.cs:80894-80918` | a6, g87 (`dfbcaf94b` + closeout `26620538e`) | Both port the mid-swing `ItemTimeIsZero && (!bought \|\| AnyNPCs(npcType))` gate + the SP purchase arm (flag + `Misc.License*Used` World line + consume). **Exchange-fail adjudication (FINAL §10.3, closed at the retire pass §11.3):** the C# exchange legs (`:80910-80917`) always chat — Success on a reroll, **Fail** otherwise — and consume either way. At the FINAL probe a6's live scan had NO chat on the bought-and-resident pass (a future silent-consume leg) and g87 hardcoded `any_npcs = false`. **g87's closeout `26620538e` folded a6's live scan + `!bought` guard AND added the PetExchangeFail line; a6's retire pass adopts the same fail-chat fold (§11.3) — the lanes are now shape-identical on all three families.** The Success leg (RerollVariation + the chat) remains the shared merge debt, landing WITH the town-pet kinds + `TownNPCProfiles`. | **a6** (`use_pet_license`, the single-seam cluster machine) + the §11.3 shared Success-leg debt. The town-arrival gates `Main.cs:65900-65911` are the deferred consumer. g87's arm dedups onto a6's at merge (shape-identical now). | `.wld` bought-pet bytes tail **2021/2022/2023** gate 217: **f45** supersedes a6 **and g87** (§4/§10.5). g87's `.trwld` 3-byte v18 pet tail rides the SEALED 107-byte union at offsets **59-61** (§12.1; §11.2's 51-53 and five-way §5a superseded). NO sidecar (world globals). |
| **5337-5342** six shimmer permanents | `:45385-45423` | a6 (all six), g89 (5338/5341/5342), g90 (5337/5339/5340/5341/5342) | Use arms: all faithful (once-ever `!used*` gate, ApplyItemTime 45, consume + TurnToAir, SendData(4) folds out). a6's mid-swing machine is the most literal (keeps the per-tick re-check for refused arms); g89/g90 fold onto the press tick (i37 convention, observation-identical SP). **Consumers were the gap — fixed §3.2-3.4**: pearl ownership (g90's Player field + mirror), aegis-crystal f32 order (g90's own `RegenInputs.aegis_crystal` slot), booster row order (g89's fruit→worm→ambrosia). a6 now matches every claimant consumer shape. | **a6** `use_shimmer_permanent` machine = the single use-arm owner for all six (the only one covering the full cluster on one seam); **dedup** g89's `use_aegis_fruit`/`use_gummy_worm`/`use_ambrosia` + g90's `use_shimmer_permanent_item`. **g90's Player field declarations + consumer shapes** are the merge anchors (a6 now matches): `used_aegis_crystal`/`used_arcane_crystal`/`used_galaxy_pearl` after `used_ambrosia`, the `update_luck` pearl mirror, the regen slots. g89's `used_aegis_fruit`/`used_ambrosia` decls dedup (same anchor). | `.plr` six unlock bools `:54781-54786` gate 260: **f44** `used_upgrades[6]` array (g89's 3 + g90's 5 named bools dedup onto it, 132 §3 L1). `.trplayer` unlock line 8 tokens 1-6: **g88 v10** (§6). No `.wld`. |
| **5451 / 5738** Jim's Drone / Remote Control Car | `:45245-45272` + refusal `:52067-52074` + shoot suppression `:43519-43526` | a6 only | a6 ports `ItemCheck_TryDestroyingDrones`: the mid-swing `ownedProjectileCounts[proj] > 0 && controlUseItem && ItemTimeIsZero && !mouseInterface` kill of the owner's 1020/1105 projectiles, `releaseUseItem = false`, NO ApplyItemTime (weapons, not consumable), the use-start refusal + shoot suppression. The 1020/1105 AI is unported (dormant until it lands). Sole claimant. | **a6** (`drone_destroy_check`). | NO persist (the remotes are weapons; the projectile counts are live pool state). |

## 2. The persist-layer split (one owner per field, no dup-writes)

Per research/132 §2's routing rule, three layers, one writer per file:

- **L1 `.plr`** (`player_file.rs`, vanilla binary release 326): **f44** is
  the base — the complete 71-row walk (research/99). a6 contributes NO
  `.plr` code (its use arms set live `Player` fields; the file slots stay
  hardcoded false on a6). The merger hydrates a6's live `Player.used_*` /
  `extra_accessory` / `unlocked_biome_torches` / `ate_artisan_bread` /
  `unlocked_super_cart` into f44's carriers at the save funnel:
  `used_upgrades[6]` indices **0=aegisCrystal, 1=aegisFruit, 2=arcaneCrystal,
  3=galaxyPearl, 4=gummyWorm, 5=ambrosia** (file order `:54781-54786`).
  g89's three named bools + g90's five named bools + bread DEDUP onto f44's
  array/fields (132 §3 L1 step 3). L1-only fields (no sidecar):
  `extraAccessory` `:54776`, torch pair `:54777-54778`, `downedDD2` `:54787`,
  super-cart, taxMoney, death counters.
- **L2 `.trplayer`** (`persist.rs`, port text sidecar): **g88 v11** is the
  base — the strict superset writer/reader (research/103 FINAL; was v10
  at this doc's FINAL probe, fold 7 `a9f1c29d8` bumped it, §11.1). a6
  contributes NO sidecar writer. The unlock line 8 carries tokens 1-6
  (§6); life pair line 9, mana pair line 10, bread line 11, loadout block
  lines 12-15; the **v11 torch pair rides line 16** — the LAST post-dye
  line AFTER the loadout block (`unlockedBiomeTorches UsingBiomeTorches`,
  writer `persist.rs:300-304`, reader `:500-524`), NOT an unlock token
  (token 7 stays DD2-RESERVED, a DD2 lane bumps v12). The live
  `Player.used_*` fields restore at world load (`app/mod.rs`); the
  pearl mirrors into `LuckState` via `update_luck` (g90 shape, a6 now
  matches); the torch restore rides `set_using_biome_torches` (the
  slot-11 property setter, f33's exact spot after `luck()`).
- **L3 world** (`.wld` flags + `.trwld`): **f45** is the `.wld` flags base
  (the complete block); a6/g89/g90/g87's five contested bytes dedup onto it
  (§4). The `.trwld` v18 tail: this doc's §5a published the **68-byte
  five-way union** (g70×g71×g90×g87×f45); the retire probe aligns it to
  the f44 FINAL master map's wider claimant set — the retire probe's
  UNIFIED input was the **101-byte fixed head + W×H dense** (§11.2: f44's
  97-byte claim corrected to 98 by its own component sum, + g87's 3 pets;
  g72 cave 31 + g69 1 + d125 1 + d109 dense seated); the SEAL pass seats
  the moon five + the fired underworld byte → the **107-byte head** under
  g71d's 92 §1-FULL authority (§12.1). World globals NEVER ride L2 (122 §5).

**a6's net persist contribution:** the `.wld` book2/satchel/bought-pet bytes
(now superseded by f45's superset, §4) and the live `Player`/`World` fields
the use arms write. a6 owns NO `.plr`/`.trplayer` writer — by design, the
carriers belong to f44/g88/g89/g90.

## 3. Fidelity fixes landed on a6 this pass (claimant more faithful → cited)

Each fix moves a6 TOWARD the C# / the more-faithful claimant. No claimant's
code was edited (out of scope); only a6's arm.

1. **Achievement dispatch → row-targeted** (`achievements.rs`,
   `powerups.rs`). a6's first pass called `notify_custom("Use")`, which
   completes EVERY `Flag("Use")` row (STAR_POWER + GET_A_LIFE + TOPPED_OFF +
   GAIN_TORCH_GODS_FAVOR + … at once). The C# `HandleSpecialEvent`
   (`AchievementsHelper.cs:288-379`) completes ONE row's ONE condition via
   `GetCondition(row,"Use").Complete()`. g88's `complete_named` and c36's
   `complete_condition` are the faithful shape. **Fix:** added
   `AchievementManager::complete_condition` (c36's verbatim shape, the
   keep-one at merge) and re-pointed the crystal/fruit/torch arms:
   `STAR_POWER`/`GET_A_LIFE`/`TOPPED_OFF`/`GAIN_TORCH_GODS_FAVOR` each
   complete only their own row. Witness
   `complete_condition_targets_one_row_not_the_flag_family`.
2. **Galaxy Pearl ownership → Player field + mirror** (`player.rs`,
   `luck.rs`, `powerups.rs`). a6's first pass latched
   `LuckState.used_galaxy_pearl` directly — no Player field to persist. The
   C# `RecalculateLuck` (`:29142-29145`) reads the PLAYER field
   `usedGalaxyPearl` (`.plr` `:54784`/`:55258`); g90 declares
   `Player.used_galaxy_pearl` as the persistence owner and mirrors it into
   the LuckState consumer in `update_luck`. **Fix:** added
   `Player.used_galaxy_pearl` (ctor `:56081` false), the 5340 arm sets it,
   `update_luck` mirrors before every recalculation. g88 v10's token 6
   carrier needs this Player-owned field (103 §5a flagged a6's direct write).
3. **Aegis Crystal f32 order → own regen slot** (`regen.rs`, `player.rs`).
   a6's first pass pre-summed `0.2 + 1.0` into `extra_time`, reordering the
   C#'s f32 accumulation. The C# adds `lifeRegenTime++` (`:19112`), THEN
   `+= 0.2f` if usedAegisCrystal (`:19113-19116`), THEN `++` if crimsonRegen
   (`:19117-19120`). g90's `RegenInputs.aegis_crystal` bool slot applies the
   +0.2 BETWEEN the base +1 and the crimson fold. **Fix:** added the
   `aegis_crystal` slot, applied between `+= 1.0` and `+= extra_time`;
   `extra_time` reverts to crimson-only. Witness
   `the_aegis_crystal_adds_between_the_base_and_the_crimson`.
4. **Booster row order → C# order** (`player.rs`). a6's first pass ran
   worm → fruit → ambrosia; the C# `UpdatePermanentBoosters`
   (`:12851-12867`) runs fruit (+4 def) → worm (+3 fish) → ambrosia (speed
   triple) — g89's verbatim order. **Fix:** re-ordered to fruit → worm →
   ambrosia. (Integer adds commute, but the order now matches the source
   for the future f32/defense-floor interactions.)
5. **TRAINING_DAY progression-40 consumer** (`world_sim/player.rs`). a6
   lacked the both-books hook. The C# `:25695-25698` fires
   `NotifyProgressionEvent(40)` every tick both world book flags stand
   (idempotent). g89e/g90 carry it. **Fix:** added the
   `notify_progression_net(40)` hook after `drop_backlog`. Witness
   `both_combat_books_fire_training_day`.

**Not a gap (kept):** a6's mid-swing `powerup_item_check` machine is the most
literal port of the `itemAnimation > 0 && ItemTimeIsZero` gate (it keeps the
per-tick re-check for refused arms); g88/g89/g90's press-tick folds are
observation-identical in SP. a6's Demon Heart double gate, Minecart
QuickSpawnItem walk, Pet License exchange gate, and Drone destroy arm have
no claimant equal or better — sole or most-faithful.

## 4. The `.wld` flags byte-compare (a6 tails 2036/2037 vs g90b/g89e/f45/g87)

All five lanes write the SAME five contested bytes at the SAME tail offsets
behind the SAME version gates. Verified line-by-line (`flags.rs` write +
read, `wld/mod.rs` tail pins; g87 added at the FINAL probe §10.5).
**Byte-identical** for the contested fields:

| Field | `.wld` write | tail offset | gate | a6 | g90b | g89e | f45 | g87 | C# anchor |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| boughtCat | `w.boolean(world.bought_cat)` | **2021** | 217 | ✓ | `false` | `false` | ✓ | ✓ | `WorldFile.cs:1392`/`:2358` |
| boughtDog | `w.boolean(world.bought_dog)` | **2022** | 217 | ✓ | `false` | `false` | ✓ | ✓ | `:1393`/`:2359` |
| boughtBunny | `w.boolean(world.bought_bunny)` | **2023** | 217 | ✓ | `false` | `false` | ✓ | ✓ | `:1394`/`:2360` |
| combatBookVolumeTwoWasUsed | `w.boolean(world.combat_book_volume_two_was_used)` | **2036** | 259 | ✓ | ✓ | ✓ | ✓ | `false` | `:1407`/`:2416-2423` |
| peddlersSatchelWasUsed | `w.boolean(world.peddlers_satchel_was_used)` | **2037** | 260 | ✓ | ✓ | `false` | ✓ | `false` | `:1408`/`:2424-2429` |

Read sides identical: `world.X = r.bool()?` at the gate (a6 `:608-610`/
`:631`/`:634`; g90 `:629`/`:632`; g89 `:631`; f45 `:704-706`/`:731`/`:734`;
g87 replaces the `r.skip(3)` with three `r.bool()?` reads at gate 217).
Tail pins identical: `tail[2036]`/`tail[2037]` (a6/g90/g89), `tail[2021-23]`
(a6/f45/g87). The golden fixture's bytes there are all 0 → the pinned
divergence set is unchanged.

**Offset collisions: NONE among the five** — every lane agrees 2036 =
princess@2035 + 1, 2037 = +1, 2021-23 = the post-ore pet trio. The collision
is DUPLICATE WRITERS of the same byte, not different offsets:

- **book2 (2036):** a6 ⨉ g90b ⨉ g89e ⨉ f45 all flip `false`→`world.X`.
  **KEEP f45** (the complete flags block superset); a6/g90/g89's flips dedup
  (same field, same gate, same byte). 132 §3 L3 step 2 already pins this
  ("g89e, g90 lane 7 and a6 ALL write combatBookVolumeTwoWasUsed — keep ONE").
- **satchel (2037):** a6 ⨉ g90b ⨉ f45 (g89e leaves `false`). **KEEP f45**;
  a6/g90 dedup.
- **bought-pets (2021-23):** a6 ⨉ f45 ⨉ g87 (g90/g89 leave `false`/skip).
  **KEEP f45**; a6/g87 dedup (g87's `World.bought_*` field names identical).
  f45 also models nurse/slime-green/old/etc. that a6/g90/g89/g87 leave
  hardcoded — f45 wins those unconditionally.

**Verdict:** f45's `flags.rs` is the `.wld` merge base. a6's use arms
(`use_combat_book`/`use_peddlers_satchel`/`use_pet_license`) write the same
`World` fields f45 persists — no semantic conflict, a clean field-level
dedup. a6's `wld/mod.rs` gate-test extensions (v258/259/260, v216/217) and
tail pins survive as witnesses re-pinned onto f45's block.

## 5. The `.trwld` v18 union collision (five-way at the FINAL probe — NINE claimants at the retire probe, §11.2)

> **Retire-pass banner:** §5/§5a below are the FINAL-probe five-way table
> (g70/g71/g90/g87/f45, 68 bytes). It EXCLUDED g72's 31-byte cave block,
> g69's 1-byte crimson, d125's 1-byte downedEmpress and d109's dense
> variants — all seated by the f44 FINAL master map (research/132 §3-L3,
> `bdd1cc79a`). The retire probe's UNIFIED input was §11.2's **101-byte
> head + W×H dense** table; the SEAL pass re-aligns it to the LANDED
> **107-byte head** (§12.1 — g71d 92 §1-FULL authority, pets 59-61, books
> 51/52, identity 63..107). §5a's offsets survive as the
> g70/g71/g90/g87/f45-relative sub-blocks with the moves §11.2/§12.1 list.

research/92 §1 fixes the g70×g71 union at **19 bytes** (forest 16 + surface
3); research/135 §4 extends it to **21** (book2/satchel at offsets 19/20).
**Neither knows f45.** f45 (`6352d384`) ALSO bumps `save.rs` VERSION 17→18
and appends a **44-byte world-identity block** at the SAME insertion point
(after the v17 `downedFishron` byte):

```
f45 v18 identity block (44 bytes), save.rs write_world:442-446:
  generator_version  u64  (8)   WorldFile.cs:1225
  unique_id          [u8;16] (16)  :1226
  world_id           i32  (4)   :1227
  creation_time      i64  (8)   :1250  (DateTime.ToBinary bit pattern)
  last_played        i64  (8)   :1251  (DateTime.UtcNow.ToBinary)
```

**And the FINAL probe adds a FIFTH writer: g87 (`dfbcaf94b`)** bumps VERSION
17→18 too and appends a **3-byte bought-pet block** at the same point
(`save.rs` writer: `bought_cat`/`bought_dog`/`bought_bunny` u8×3,
`WorldFile.cs:1392-1394`). Every one of the five v18 writers mis-parses on
every other's reader — same class 92 §1 documents for g70×g71, now
**five-way**. **Resolution (forward-only, no released v18 exists):** union
ALL FIVE blocks at v18 in a fixed order, the 92 §1 "least-surprising C#
field order" rationale extended.

### 5a. THE EXACT BYTE-LAYOUT TABLE (the FINAL-probe five-way sub-block — SUPERSEDED by §11.2's unified 101-byte head)

> **Retire-pass banner (seal-updated):** the offsets below are the
> five-way (68-byte) layout. In the retire probe's UNIFIED 101 head
> (§11.2) g70's forest stayed 0..16; g72's cave inserted at 16..47; g71's
> surface moved 16..19 → 47..50; g69's crimson took 50; g87's pets moved
> 19-21 → 51-53; d125's empress took 54; g90's books moved 22/23 → 55/56;
> identity moved 24..68 → 57..101. **In the SEALED 107 head (§12.1, g71d
> 92 §1-FULL authority) the final seats are: books 51/52 (64a prefix),
> moon five 53-57, underworld 58, pets 59-61 (vanilla write order: pets
> `:1392-1394` precede downedEmpress `:1395`), empress 62, identity
> 63..107, d109 dense LAST at 107..107+W×H.** The field ORDER inside
> every block is unchanged; only the positions move.

All offsets are from the START of the v18 block, i.e. the byte immediately
after the v17 `downedFishron` u8. Every multi-byte field is little-endian
(the port's `binio` convention, matching `BinaryWriter`). **v18 five-way
tail = 68 bytes** (16 forest + 3 surface + 3 pets + 2 books + 44 identity):

| Offset | Len | Field | Rust type | Owner block | C# anchor | Writer witness |
| --- | --- | --- | --- | --- | --- | --- |
| 0..4 | 4 | `tree_x[0]` | i32 LE | g70 forest | `WorldFile.cs:1253` | `save.rs:435-437` (g70) |
| 4..8 | 4 | `tree_x[1]` | i32 LE | g70 forest | `:1254` | 〃 |
| 8..12 | 4 | `tree_x[2]` | i32 LE | g70 forest | `:1255` | 〃 |
| 12 | 1 | `forest_bg_styles[0]` (treeBG1) | u8 | g70 forest | `:1318` | `save.rs:438` (g70) |
| 13 | 1 | `forest_bg_styles[1]` (treeBG2) | u8 | g70 forest | `:1377` | 〃 |
| 14 | 1 | `forest_bg_styles[2]` (treeBG3) | u8 | g70 forest | `:1378` | 〃 |
| 15 | 1 | `forest_bg_styles[3]` (treeBG4) | u8 | g70 forest | `:1379` | 〃 |
| 16 | 1 | `snow_bg_style` | u8 | g71 surface | `:1321` | `save.rs:438-440` (g71) |
| 17 | 1 | `desert_bg_style` | u8 | g71 surface | `:1324` | 〃 |
| 18 | 1 | `ocean_bg_style` | u8 | g71 surface | `:1325` | 〃 |
| 19 | 1 | `bought_cat` | u8 bool | **g87 pets (NEW)** | `:1392` | g87 `save.rs` v18 writer |
| 20 | 1 | `bought_dog` | u8 bool | **g87 pets (NEW)** | `:1393` | 〃 |
| 21 | 1 | `bought_bunny` | u8 bool | **g87 pets (NEW)** | `:1394` | 〃 |
| 22 | 1 | `combat_book_volume_two_was_used` | u8 bool | g90 books | `:1407` | g90 `save.rs:443` |
| 23 | 1 | `peddlers_satchel_was_used` | u8 bool | g90 books | `:1408` | g90 `save.rs:444` |
| 24..32 | 8 | `generator_version` | u64 LE | f45 identity | `:1225` | f45 `save.rs:442` |
| 32..48 | 16 | `unique_id` (GUID bytes) | [u8;16] | f45 identity | `:1226` | f45 `save.rs:443` |
| 48..52 | 4 | `world_id` | i32 LE | f45 identity | `:1227` | f45 `save.rs:444` |
| 52..60 | 8 | `creation_time` | i64 LE (ToBinary) | f45 identity | `:1250` | f45 `save.rs:445` |
| 60..68 | 8 | `last_played` | i64 LE (ToBinary) | f45 identity | `:1251` | f45 `save.rs:446` |

**Ordering rationale** (92 §1's rule extended): the bg-style cluster first
(g70's forest bytes are the C#'s treeX dividers `:1253-1255` + the four
tree-set styles treeBG1 `:1318` / treeBG2-4 `:1377-1379` — g70's
`forest_bg_styles[0..4]` maps to treeBG1..4, NOT the corrupt/jungle/hallow
biome bytes; g71's three surface styles snowBG `:1321` / desertBG `:1324`
/ oceanBG `:1325` follow), then the world-flag bytes in vanilla write
order (pets `:1392-1394` BEFORE books `:1407-1408` — g87's pets slot in
ahead of g90's pair, which moves +3), then f45's identity block last (its
fields live in the HEADER `:1225-1227` + flags-head `:1250-1251`, far from
this tail — a late-append is the least-surprising spot and keeps the
g70/g71 blocks' offsets 0..19 untouched; only g90's pair (+3) and f45's
block (+24) re-pin).

**f45's identity-block internal order is FIXED** (its writer
`:442-446` writes generator_version → unique_id → world_id → creation_time
→ last_played; the reader `:833-839` mirrors it, `creation_time`/
`last_played` read `r.u64()? as i64` — the `DateTime.ToBinary` bit pattern,
NOT a Unix timestamp). The merger must keep the five fields in this order
inside offsets 24..68; only the block's POSITION in the union moves
(f45's own file has it at 0..44; the union shifts it +24).

**VERSION stays 18.** In the five-way layout every truncation constant
moves to **+68**; in the retire probe's UNIFIED head (§11.2) every
constant moved to **+101 plus the dense-block term**; in the SEALED head
(§12.1) every constant moves to **+107 plus the dense-block term** (the
fixed head alone is 107; the end-relative probes subtract
`107 + width*height` — 92 §1-FULL.4's per-site table). The exact sites,
per lane (line numbers at each lane's own tip):

- **g71 (`save.rs`)**: `:1188` v6-downgrade truncate `+16+3`→`+68`… the v6
  site's tail addend, `:1238` `- 10 - EMPTY_V16_TAIL` probe, `:1300` v12
  `let tail … + 3`→`+ 68`, `:1329` `EMPTY_V16_TAIL = … + 1 + 3`→`+ 68`,
  `:1332`/`:1339` back_flags ends, `:1518` v14 probe `- 1 - 5 - 1 - 3`→
  `- 1 - 5 - 1 - 68`, `:2210` pre-v18 probe `- 3`→`- 68`.
- **g90**: the same six sites at its `+2` constants → `+68` (135 §4's list).
- **f45 (`save.rs`)**: `:1103` `- 3 - 44`→`- 3 - 68`, `:1199` v6 tail
  `+ 44`→`+ 68`, `:1250` `- EMPTY_V18_TAIL` (44→68 via the const), `:1312`
  v12 `let tail … + EMPTY_V18_TAIL`, `:1343` `EMPTY_V18_TAIL = 8+16+4+8+8`
  → `68` (rename `EMPTY_V18_UNION_TAIL`), `:1346`/`:1353` back_flags ends,
  `:1532` v14 probe, `:1561` pre-v18 probe `- EMPTY_V18_TAIL`, `:1608` v5
  downgrade `+ EMPTY_V18_TAIL`.
- **g87 (`save.rs`)**: the v12 `let tail … + 3`, `EMPTY_V16_TAIL … + 3`,
  the v14 `- 1 - 5 - 1 - 3` probe, the pre-v18 `- 3` probe → all `68`.
- **g70**: the `+16` sites (92 §1's list) → `+68`.

Every lane's round-trip witness re-pins 16/3/3/2/44 → 68 (five-way) or →
101 + W×H (unified, §11.2) or → **107 + W×H (SEALED, §12.1)**. **Merge
action:** the g71d union owner lands 92 §1-FULL.2's writer/reader pair
(the 107 authority, §12.1); g90's pair MOVES (19/20 → 22/23 five-way →
55/56 unified-101 → **51/52 sealed-107**); f45's `stamp_created_identity`
+ WorldId roll stay its scope; g87's `World.bought_*` fields dedup onto
f45/a6's identical fields (same names — a clean field-level dedup). This
doc surfaces BOTH the f45 and the g87 collisions — flag to the
g71c/f44e/f45/g87 merger. (Retire note: f45's 44-byte identity block and
c43 `076970809`'s are the SAME block — c43 is in f45's lineage, verified
byte-order-identical at both tips; the f44 FINAL map names it c43's. ONE
44-byte seat, not two.)

## 6. The `.trplayer` token-order adjudication (g88 v10 WRITTEN order wins)

Three coordinate systems met on line 8's tokens 4/5/6:
- **103 §2a (pre-v10 reservation):** pearl=4 / arcane=5 / crystal=6.
- **122 §3 (append order):** crystal=4 / arcane=5 / pearl=6 ("vanilla .plr
  order" `:54781`/`:54783`/`:54784`).
- **135 §3 recommendation:** "103 §2a wins" (pearl/arcane/crystal), arguing
  no pushed writer ever emitted token 4+ → zero back-compat obligation.

**g88 fold 6 (`c8985b534`) SUPERSEDES this: it WROTE tokens 4/5/6 in the
vanilla relative order crystal=4 / arcane=5 / pearl=6** (verified
`persist.rs:229-234` writer + `:390-392` reader:
`used_aegis_crystal`/`used_arcane_crystal`/`used_galaxy_pearl`). 135 §3's
"no pushed writer emitted token 4+" premise is now FALSE — g88 v10 is pushed
and emits them. **The written order is binding** (it matches 122 §3, not
103 §2a's old reservation). 132 §2b's table (pearl=4/arcane=5/crystal=6,
written 06:56 before fold 6 at 07:15) is STALE on this point.

**Verdict:** line 8 = `aegisFruit gummyWorm ambrosia aegisCrystal
arcaneCrystal galaxyPearl` (tokens 1-6), token 7 DD2 RESERVED (a DD2 lane
bumps **v12** — v11 is TAKEN by the torch-pair LINE, not a token; retire
correction, §11.1). This is g88's actual writer/reader (v10 fold 6, kept
verbatim by v11 fold 7); the merger keeps it. 135 §3's
mapping table column "UNION token (recommended)" was corrected by g88's
own 103 §8 alignment note (the v10 written order WINS; 135's
no-pushed-writer premise means zero back-compat obligation either way).

**The six-line v7 recognition rule (135 §2) LANDED in g88 fold 7
(`a9f1c29d8`) — the FINAL-probe re-flag below is RETIRED (§11.1).** At the
FINAL probe the tip was still `c8985b534` (fold 6) and the rule was missing;
fold 7 landed it IN CODE: `persist.rs:463-465` computes
`v7_six_line = v7_flags && nonempty.len() == 6 &&
nonempty.iter().all(|t| t.len() == 1)` and `:470-479` reads the block
POSITIONALLY — `ate_artisan_bread = nonempty[0][0]`,
`used_aegis_crystal = nonempty[1][0]`, `used_arcane_crystal =
nonempty[2][0]`, `used_galaxy_pearl = nonempty[3][0]`, `used_gummy_worm =
nonempty[4][0]`, `used_ambrosia = nonempty[5][0]` — exactly 135 §2's
`.plr` write order. The bounded-misread witness is RETIRED (zero
`bounded_misread` matches in the tip's `persist.rs`), replaced by
`g90bs_six_line_v7_loads_positionally` (`:1198`). Fold 7 ALSO lands the
f33 lone-all-0/1-pair rule (`:466-469` + `:485-488`: a lone post-dye
2-token line whose tokens are all "0"/"1" is the torch pair — no maxima
pair can be all-0/1, the permanent halves start 100/20; witness
`f33s_torch_only_v7_loads_positionally` `:1233`). The historical
FINAL-probe flag text follows, kept as the record:

<details>
<summary>FINAL-probe flag (STALE — the rule landed in a9f1c29d8)</summary>

The v7 walk (`persist.rs:409-418` at `c8985b534`) collapsed
g90b's six 1-token lines into the aegis/bread slots: the `match toks.len()`
`1 =>` arm read the FIRST one-token line as `used_aegis_fruit`
(`flag_seen` latch) and EVERY later one-token line as `ate_artisan_bread`
(the last wins). No `post.len() == 6 && all(t.len() == 1)` guard existed.
The witness `g90bs_bread_plus_flags_v7_loads_with_the_bounded_misread`
ACCEPTED the misread. 135 §2 demanded the unambiguous rule:

```
if version == "7" && post.len() == 6 && post.iter().all(|t| t.len() == 1) {
    bread=post[0]; crystal=post[1]; arcane=post[2]; pearl=post[3];
    gummy=post[4]; ambrosia=post[5];   // positional, .plr write order
}
```

</details>

**Merge action: SATISFIED by the pushed writer — no reader debt remains.**
g88's v7 legacy arm carries 135 §2's positional six-line rule; a g90b
lane-7 v7 file now loads LOSSLESSLY on the superset. The one remaining
`.trplayer` reader gap this doc flagged is CLOSED (§11.1's receipt).

## 7. Witness union (every lane's witnesses survive the merge)

The merger keeps ALL witnesses, re-pinned onto the surviving carriers:

- **a6 `powerups.rs`** (this pass): the cluster witnesses (cap refusals,
  holding-LMB one-per-press, RAW adds, demon-heart expert gate, minecart
  overflow, pet-license exchange refusal, drone kill) + NEW
  `both_combat_books_fire_training_day`. The crystal/fruit/torch achievement
  assertions now read row-targeted completions.
- **a6 `achievements.rs`**: NEW
  `complete_condition_targets_one_row_not_the_flag_family` (the dispatch
  keep-one; dedups c36's identical helper witness).
- **a6 `regen.rs`**: `the_arcane_crystal_shaves_the_delay_and_adds_regen` +
  NEW `the_aegis_crystal_adds_between_the_base_and_the_crimson` (the f32
  order; supersedes a6's old extra_time fold).
- **g88 `crystals.rs`/`life_fruit.rs`/`persist.rs`**: the RAW-add + 20-fruit
  ladder + sidecar round-trip + the v10 six-token unlock witnesses
  (`the_six_token_unlock_line_reads_under_v10`,
  `a_three_token_unlock_line_reads_under_v10`,
  `the_v9_superset_still_loads_without_the_crystals`). The
  bounded-misread witness was REPLACED by 135 §2's positional-read witness
  — LANDED in fold 7 as `g90bs_six_line_v7_loads_positionally` (§11.1);
  the v11 torch-line witnesses (`f33s_torch_only_v7_loads_positionally`,
  `the_v10_superset_still_loads_without_the_torch_pair`, the v11
  round-trip with the `1 1` torch line LAST) ride the same file.
- **g89 `player_file.rs`/`editing.rs`/`npc_ai_town.rs`**: the `.plr`
  fruit/gummy/ambrosia round-trips (re-pin onto f44's `used_upgrades[6]`),
  the use-arm witnesses (dedup onto a6's machine), the vol-two town-ladder
  witness (`w4_g89e_the_vol_two_book_stacks_the_same_bonus_and_heals_on_growth`).
- **g90 `player_file.rs`/`persist.rs`/`save.rs`/`shop.rs`/`player.rs`**: the
  `.plr` five-bool + bread round-trips (re-pin onto f44), the `.trwld` v18
  round-trip (re-pin 2→68 + the offset move 19/20→22/23, §5a), the
  skeleton-merchant loaf gate, the ambrosia
  speed triple, the pearl mirror, the crystal regen witnesses.
- **c36**: the `complete_condition` helper witness (dedups a6's), the
  GAIN_TORCH_GODS_FAVOR credit (a6's torch arm now carries it).
- **f33**: the Torch God event witnesses (untouched — the event machine is
  its sole scope).
- **f44 `player_file.rs`**: the complete-walk + G1-G7 + timestamp witnesses
  (the `.plr` base; g89/g90's slot assertions re-pin onto the array).
- **f45 `wld/`/`save.rs`**: the complete-flags + identity-block witnesses
  (the `.wld` base + the `.trwld` identity tail, re-pin 44→68 + the block
  move 0..44→24..68, §5a).
- **g87 `permanent_upgrades.rs`/`persist.rs`/`save.rs`/`flags.rs`**: the
  use-arm witnesses DEDUP onto a6's machine (§10.1-10.3); the `.trplayer`
  `dh` round-trip witnesses DROP with the line (§10.4); the `.trwld` pet
  round-trip re-pins 3→68 at offsets 19-21 (§5a); the `.wld` bought-pet
  round-trip assertions re-pin onto f45's block (§4).

## 8. Merge order (the recipe)

Per-layer, the surviving base first, then the folds:

1. **L1 `.plr`** — base **f44** (`05e4020bb`, the complete walk). Fold g55's
   loadout semantics. Dedup g89's 3 + g90's 5 named unlock bools + bread onto
   `used_upgrades[6]` / `ate_artisan_bread` (132 §3 L1). Hydrate a6's live
   `Player.used_*` at the save funnel (indices §2). Keep f44's timestamp fix.
2. **L2 `.trplayer`** — base **g88 v11** (`a9f1c29d8`, the superset; was
   `c8985b534` v10 at the FINAL probe). 135 §2's six-line v7 recognition
   rule is ALREADY LANDED in code (§6/§11.1) — no reader action remains.
   Tokens 4/5/6 are ALREADY live in the written crystal/arcane/pearl order
   (§6) — no activation needed, just keep g88's writer/reader. The v11
   torch pair (line 16, LAST) folds f33's transport — keep g88's
   setter-restore (`set_using_biome_torches`, the slot-11 property; f44's
   `63204f852` L1 byte rides the same property). Dedup g90's
   `SavedPlayer` flag fields + App restores onto the superset's. a6
   contributes nothing (no writer). The g88 103 §2b CONDITIONAL DROP of
   the torch sidecar line applies only when character-load hydration AND
   the exit-save both land (132 §2c precedence: character first, sidecar
   second, sidecar wins as newer state).
3. **L3 `.wld` flags** — base **f45** (`6352d384`, the complete block). Dedup
   a6/g89e/g90/g87's book2 (2036) / satchel (2037) / bought-pets (2021-23)
   flips onto f45's fields (§4). a6's use arms write the same World fields.
4. **L3 `.trwld`** — the **107-byte-head + W×H dense v18 union** (§12.1's
   SEALED alignment of §11.2's 101, g71d 92 §1-FULL.2 authority;
   superseding §5a's five-way 68): g70 forest 16 + g72 cave 31 + g71
   surface 3 + g69 crimson 1 + g90 book2/satchel 2 + c39/f39 moon 5 +
   g71 underworld 1 + g87 pets 3 + d125 empress 1 + f45/c43 identity 44
   + d109 dense LAST. Move every truncation constant to +107 (+ the
   dense term at the end-relative sites). VERSION stays 18,
   forward-only. The g71d union owner seats every block per 92
   §1-FULL.2; g90's pair lands at 51/52, g87's pets at 59-61, the
   identity block at 63..107.
5. **Use arms / consumers** — keep **a6's `powerups.rs` machine** for the
   cluster EXCEPT 29/109/1291 (g88 keep-own, 103 §5a) and 5326 (g90 keep-own,
   the skeleton-merchant consumer). Dedup g89's/g90's/c36's parallel shimmer
   / combat-book / torch / loaf arms onto a6's machine + the surviving
   keep-owns. **Dedup g87's three arms** (`permanent_upgrades.rs` —
   3335/5289/licenses, §10.1-10.3) onto a6's machine; DROP g87's `.trplayer`
   `dh` line + SavedPlayer fields (§10.4). Carry the §10.3 exchange-fail
   debt (the Success/Fail chats must land WITH the town-pet kinds). Keep **g90's Player field declarations + consumer shapes**
   (a6 now matches: pearl mirror, regen slots, booster order). Keep the
   shared `complete_condition` helper (a6 == c36, dedup). Keep **f33's** Torch
   God event machine + flag reads. Keep **a6's** torch write (the faithful
   `builder_acc_status[11]=0` setter, supersedes c36's fold).
6. **Docs** — update 103 §2a (tokens 4/5/6 LIVE, crystal/arcane/pearl order),
   135 §3 (correct the UNION-token column to the written order), 132 §2b
   (correct the stale token order + add the f45 identity collision), 92 §1
   (superseded by g71d's own §1-FULL — the 107 union landed there, §12.1),
   and INDEX rows 103/122/132/135/138.

**No code change to any other lane** (out of scope). a6's five fidelity
fixes (§3) are the only code edits this pass; they make a6's consumers match
the more-faithful claimants so the merge dedups without semantic conflict.

## 9. Verification performed

- **C# re-reads** (direct file paths, ≤100-line windows, no junction
  recursion): `Player.cs:45180-45479` (all use arms), `:12830-12889`
  (UpdatePermanentBoosters), `:19100-19129`/`:19420-19464` (regen rows),
  `:25685-25709` (TRAINING_DAY), `:29135-29149` (RecalculateLuck),
  `:3948-3967` (UsingBiomeTorches setter), `:42590-42614`/`:42720-42759`/
  `:43280-43309`/`:43580-43619` (ItemCheck gates + call row + consume tail),
  `:51660-51704`/`:52048-52077` (StartActualUse + CheckCanUse refusals),
  `:54765-54799`/`:55220-55264` (Serialize/Deserialize unlock block);
  `AchievementsHelper.cs:282-381` (HandleSpecialEvent switch);
  `AchievementCondition.cs:1-62` (Complete()); `NPC.cs:80880-80954`
  (UnlockOrExchangePet); `Chest.cs:1280-1314` (travel shop);
  `IO/WorldFile.cs:1385-1419`/`:2350-2429` (flags write/read gates).
- **Claimant diffs at fetched tips**: g88 `c8985b534` (v10 writer/reader
  verbatim), g89 `598642563`, g90 `b3b82d071` (+ research/135), c36
  `585fa0a84` (+ research/30), f33 `333479e0b`, f44 `05e4020bb` (+
  research/99/132), f45 `6352d3840`, g71 92 §1. g87 was ZERO commits at
  this probe; the FINAL probe (§10) read its pushed `dfbcaf94b`.
- **Byte-compare**: the five `.wld` contested bytes extracted line-by-line
  from a6/g90/g89/f45 `flags.rs` (§4) — identical fields/offsets/gates.
- **a6 edits**: `git diff --stat` = achievements.rs +70, luck.rs ±9,
  player.rs ±59, powerups.rs ±65, regen.rs +42, world_sim/player.rs +8,
  ANALYSIS.md ±35. No cargo/tests/rustfmt/graphify per the lane constraint;
  witnesses WRITTEN, NOT RUN.

## 10. FINAL re-adjudication — g87 `dfbcaf94b` pushed (arm-by-arm C# re-read)

The reconcile probe (§0) read g87 as zero commits. **g87 is NOW pushed**
(`origin/task/W4-g87-demon-heart-use` = `dfbcaf94b`, one commit over main
`a725acb04`, 11 files +876/-27). **No `g87b-143` branch exists** — the
full `git branch -r` census shows only `task/W4-g87-demon-heart-use`; the
143 lane never pushed a g87b ref. g87's scope: 3335/5289/4829-30-4910 use
arms in a new `world_sim/editing/permanent_upgrades.rs` (566 lines, its own
inlined `permanent_upgrade_item_check` machine — g88's `life_fruit` shape),
a `.trplayer` **v7** writer/reader with a self-identifying `dh <extra>
<cart>` line, a `.trwld` **v18** 3-byte bought-pet tail, `.wld` bought-pet
flips at gate 217, and `World.bought_*` field declarations. Every arm
re-adjudicated against the C# (direct reads `Player.cs:45192-45214`,
`:45352-45366`, `:45438-45445`, `:52051-52054`, `:7170-7200`,
`:39385-39399`; `NPC.cs:80894-80929`; `GetItemSettings.cs:57`;
`Item.cs:504-528`; `IO/WorldFile.cs:1221-1259`):

### 10.1 Demon Heart 3335 — **a6 KEEPS** (g87 less faithful at the start)

Grant arms equal: both gate `!extra_accessory && expert_mode`, set the flag
once, ApplyItemTime (useTime 30), consume one; SendData(4) folds out on
both. **The difference is the use START.** The C# refuses the START in
`ItemCheck_CheckCanUse_Inner` (`:52051-52054`: `type == 3335 &&
(extraAccessory || !Main.expertMode)` → `flag = false` — no
`ApplyItemAnimation`, no UseSound, no swing). a6 ports this
(`use_start_refused`, `powerups.rs:80-88` — the machine returns before the
swing starts). g87 does NOT: its machine starts the animation
unconditionally on the press edge and lets the grant gate refuse — its own
witness `w4_g87_the_demon_heart_refuses_in_classic_mode` documents "the
machine starts the swing, but the grant gate refuses". In classic mode the
C# player never swings the heart; g87's does (a visible 30-tick HoldUp
animation + the UseSound). **Verdict: a6's arm is the more faithful port;
g87's `try_use_demon_heart`/`demon_heart_item_check` DROP at merge.** g87's
two grant witnesses dedup onto a6's `demon_heart_unlocks_the_extra_slot_in_expert`
/ `a_second_demon_heart_refuses`; its classic witness RE-PINS onto a6's
`demon_heart_outside_expert_refuses` (which asserts no swing).

### 10.2 Minecart Powerup 5289 — **a6 KEEPS** (both always-spawn; a6's spawn walk full)

**The gate question is settled: BOTH ports are NO-flag-gate ALWAYS-spawn.**
The C# `:45204` is `type == 5289 && itemAnimation > 0 && ItemTimeIsZero` —
NO `!unlockedSuperCart` leg (contrast the Demon Heart `:45194`). Every
granted use re-arms ApplyItemTime, raises the flag once (`:45207-45211`)
and `QuickSpawnItem(source, 3353)` (`:45212`) — a second powerup ALWAYS
spawns another Mech. a6 (`use_minecart_powerup`, `powerups.rs:206-214`) and
g87 (`try_use_minecart_powerup`) both port exactly this; a6's witness
`a_second_minecart_powerup_grants_another_mech` and g87's
`…unlocks_the_cart_and_spawns_the_mech` both pin the re-use spawn. No
fidelity gap on the gate.

**The spawn walk differs.** The C# `QuickSpawnItem` (`:7170-7199`) makes a
fresh `Item`, `Prefix(-1)` (`:7176`), then `GetOrDropItem(item,
GetItemSettings.PickupItemFromWorld)` — and `PickupItemFromWorld` is
`CanGoIntoVoidVault: true` (`GetItemSettings.cs:57`). `GetItem` runs the
full deposit walk (occupied-stack scan → ammo slots for ammo kinds →
empty-slot scan, chime + `NotifyItemPickup` per accepted slot
`:39491`/`:39505`), the **void vault as the LAST resort**
(`:39395-39398`), and the remainder drops at `base.Center` with the prefix
preserved (`:7196-7199`). a6's `quick_spawn_item` (`powerups.rs:230-294`)
ports ALL of it: `deposit_pickup` (the shared walk incl. ammo/favorites),
the per-slot chime/achievement notify, `void_vault_take` gated on
`void_vault_enabled`, the overflow drop with the pinned velocity triplet.
g87's twin is the **bobber-catch simplification** (`slot_for_pickup` loop +
`spawn_item_drop_px`, copied from `world_sim/projectiles.rs:569`'s catch
drain): no ammo/favorites handling, no chime/notify, **no void vault** (the
C# settings demand it), and the overflow drop goes through the shared
`spawn_item_drop_px` (a different velocity/phase pin than the
reforge-return precedent a6 uses). **The RNG stream differs too:** the C#
roll rides `Main.rand` inside `Player.Update` (the use arm runs in
`ItemCheck` under the player pass) — a6 pins **UpdatePlayers**; g87 pins
**UpdateItems**, importing the bobber's projectile-pass convention into a
player-context use. For the Mech (3353) the difference is dormant —
`CanHavePrefixes` false → `Item.Prefix` returns before any draw
(`Item.cs:515-518`), both streams untouched — but g87's walk would diverge
the moment the helper is reused for a prefixable kind, and its missing
vault leg is a real behaviour gap TODAY (a full-inventory use with the void
vault open drops the Mech in the world instead of banking it). **Verdict:
a6's arm + `quick_spawn_item` KEEP; g87's twin DROP.** g87's spawn witness
dedups onto a6's `minecart_powerup_unlocks_and_grants_the_mech`.

### 10.3 Pet Licenses 4829/4830/4910 — **a6 KEEPS** + the exchange-fail merge debt

> **Retire-pass banner:** the debt below is HALF-CLOSED on both lanes —
> g87's closeout `26620538e` added the Fail chat, a6's retire pass folds
> the same (§11.3). The Success leg is the one remaining shared debt.

Both port `LicenseOrExchangePet`'s gate (`:45440`: `ItemTimeIsZero &&
(!petBoughtFlag || NPC.AnyNPCs(npcType))`), the SP purchase arm
(`NPC.cs:80904-80909`: flag + `Misc.License*Used` World chat), ApplyItemTime
+ consume; the MP legs (`:80897-80902`, `TrySendData(7)`) fold out on both.
Today both refuse a bought license: no TownCat(637)/TownDog(638)/
TownBunny(656) kind is ported, so `AnyNPCs` never answers.

**The exchange-fail difference (the Lead's question):** the C# exchange
legs (`NPC.cs:80910-80917`) post `Misc.PetExchangeSuccess` when
`RerollVariationForNPCType` finds the resident pet and rerolls, else
**`Misc.PetExchangeFail`** (`:80916`) — and the use CONSUMES either way
(`:45442` ApplyItemTime ran before the dispatch). a6 keeps the LIVE scan
(`app.npcs.iter().any(|n| n.hp > 0 && n.kind.net_id() == npc_type)`,
`powerups.rs:418-421`): today it never answers (identical observed verdict
to g87), but when a town pet lands, a6's gate will pass and its arm will
`ApplyItemTime` + consume with **no chat and no reroll** — a silent consume
where the C# always chats. g87 hardcodes `let any_npcs = false` (its module
header records the deviation): its bought license ALWAYS refuses, forever
— no silent-consume leg, but also no path to the C# exchange behaviour
without a rewrite. **Neither port is complete on the exchange; a6's live
scan is the more faithful SHAPE (the C# gate really does scan), g87's
hardcode is the safer CLOSED world.** Verdict: **a6 keeps the arm**; the
merge carries an explicit debt — when the town-pet kinds land
(`Main.cs:65900-65911` arrival + `TownNPCProfiles.RollVariation`), the arm
must gain the `RerollVariationForNPCType` dispatch + the Success/Fail chats
(`NPC.cs:80910-80917`) BEFORE the pets can spawn, else the silent-consume
leg goes live. g87's three license witnesses dedup onto a6's
`cat_license_sets_bought_cat_and_chats` / `dog_and_bunny_licenses_set_their_own_flags`
/ `a_bought_license_without_a_resident_pet_refuses`.

### 10.4 g87's `.trplayer` v7 `dh` line — **DROP** (version collision + L1-only)

g87 bumps the sidecar writer `"6\n"` → `"7\n"` and appends a
self-identifying `dh <extraAccessory> <unlockedSuperCart>` line after the
dye line; its reader tolerates a missing/non-`dh` line. **Two problems:**

1. **Version collision:** g88's superset owns v7 (the aegis/bread flag
   lines) through v10 (the six-token unlock line). g88's v7 reader walks
   post-dye lines by TOKEN COUNT — a `dh 0 0` line is a THREE-token line,
   which its walk does not recognize as a flag/pair/loadout shape (it falls
   through the `match toks.len()` arms). A g87-written v7 file loaded by
   g88's superset silently loses both flags; a g88 v10 file loaded by g87's
   reader mis-parses (g87 only knows "3"-"7"). The `dh` sentinel's
   self-identification protects g87's OWN reader, not the superset's.
2. **L1-only ruling:** 132 §2a routes `extraAccessory` (`:54776`) and the
   super-cart BitsByte (`:54914-54918`) to **f44's `.plr` walk, L1-only** —
   g87's own module header concedes "the `.plr` halves are the f44
   verify-sweep's lane". 122 §4.4 records NO sidecar line for these fields.
   The sidecar carrier is redundant with f44's complete walk.

**Verdict: g87's `persist.rs` changes (SavedPlayer fields, v7 writer bump,
`dh` reader arm, both round-trip witnesses) DROP at merge; the g88 v11
superset stays the sole L2 writer.** The App restore
(`p.extra_accessory = saved.extra_accessory` etc.) dedups onto the f44
hydration at the save funnel (§2 L1). If the merger ever wants a live-
session carrier before f44 lands, the fields ride g88's NEXT version as
new tokens — never a parallel v7. (Retire-probe note: **v11 is TAKEN by
the f33 torch pair** — such a carrier rides **v12+**, §11.1.)

### 10.5 g87's `.wld`/`.trwld` halves — dedup onto f45 / ride the union

- **`.wld` bought-pets:** g87 flips `flags.rs`'s three `w.boolean(false)`
  → `w.boolean(world.bought_*)` at gate 217 and reads them back — the
  SAME bytes/offsets/gates as a6 and f45 (§4's table now five-lane).
  **KEEP f45**; g87's flips dedup (same field names — g87's
  `World.bought_cat/dog/bunny` declarations dedup onto a6/f45's identical
  fields, ctor-false). g87's `wld/mod.rs` tail pins `tail[2021-23]` are
  byte-identical to a6/f45's — the witnesses merge cleanly.
- **`.trwld` pets:** g87's 3-byte v18 tail is the FIFTH union member
  (§5/§5a): offsets 19-21 in the five-way, ahead of g90's book pair
  there (vanilla write order `:1392-1394` < `:1407-1408`). In the SEALED
  107 union the pets seat **59-61** and the books ride the 64a prefix at
  51/52 (§12.1 — 64a's published order keeps its offsets stable). Its
  round-trip witness re-pins 3→68→107.

### 10.6 g88 recognition-rule re-check — **RETIRED at the retire probe (§11.1)**

At the FINAL probe the tip was `c8985b534` and the 135 §2 six-line v7 rule
was missing (re-flagged then). **g88 fold 7 (`a9f1c29d8`) has since landed
the rule IN CODE** — `persist.rs:463-479`'s positional six-line read +
`:466-488`'s f33 lone-pair read, witnesses
`g90bs_six_line_v7_loads_positionally` (`:1198`) /
`f33s_torch_only_v7_loads_positionally` (`:1233`); the bounded-misread
witness is retired. §6's flag is closed; the receipt is §11.1.

### 10.7 a6's five fidelity fixes — LANDED, witnesses present (re-receipted §11.4)

Re-verified on this branch (`d7de8bd53` + the FINAL pass; the retire pass
re-pins the shifted line cites in §11.4), each fix's code AND witness
present:

1. **Row-targeted dispatch:** `AchievementManager::complete_condition`
   (`achievements.rs:445`) + the call sites (`powerups.rs:177`/`:179`/
   `:362`/`:531`/`:533` — GET_A_LIFE/TOPPED_OFF/GAIN_TORCH_GODS_FAVOR/
   STAR_POWER). Witness `complete_condition_targets_one_row_not_the_flag_family`
   (`achievements.rs:634`). ✓
2. **Pearl Player-field ownership:** `Player.used_galaxy_pearl`
   (`player.rs:2171`, ctor `:2978` false), the 5340 arm sets the Player
   field (`powerups.rs:489`), `update_luck` mirrors
   (`player.rs:10651`). Witness assertions `powerups.rs:1314-1316` (the
   Player field owner + the mirror seeds the state). ✓
3. **Aegis-crystal regen slot:** `RegenInputs.aegis_crystal`
   (`regen.rs:117`), applied BETWEEN `+= 1.0` (`:190`) and `+= extra_time`
   (`:197`) at `:194-196`. Witness
   `the_aegis_crystal_adds_between_the_base_and_the_crimson`
   (`regen.rs:688`). ✓
4. **Booster order:** fruit (`player.rs:6916`) → worm (`:6919`) → ambrosia
   (`:6922`) — the C# `UpdatePermanentBoosters` order. ✓
5. **TRAINING_DAY hook:** `notify_progression_net(40)` when both world book
   flags stand (`world_sim/player.rs:1625-1627`). Witness
   `both_combat_books_fire_training_day` (`powerups.rs:1324`). ✓

No cargo/tests/rustfmt/graphify per the lane constraint — the witnesses
are WRITTEN, NOT RUN; presence verified by direct file reads.

### 10.8 FINAL verdict summary (as of the FINAL probe — the retire pass updates it in §11.5)

The §1 table's keep-owns are UNCHANGED by g87's push: **a6 keeps
3335/5289/4829-30-4910** (g87's three arms DROP — §10.1 start-refusal gap,
§10.2 spawn-walk gaps, §10.3 the live-scan shape + the exchange debt),
**g87's sidecar `dh` line DROPS** (§10.4), **g87's `.wld` flips dedup onto
f45** (§10.5), **g87's `.trwld` pets ride the union** (§5a five-way /
§11.2 unified). At the FINAL probe the `.trplayer` reader gap (§6/§10.6)
and the five-way v18 collision (§5) were the two open merge actions; the
retire pass CLOSES the reader gap (g88 fold 7 landed the rule, §11.1) and
WIDENS the union to the unified 101-byte head (§11.2; the seal pass seats
the final 107, §12.1).

## 11. RETIRE pass (2026-11-21, over `115ce655a`) — the landed-claimant receipts

Probe inputs (fetched tips, `git fetch origin --prune`): g88
**`a9f1c29d8`** (fold 7 FINAL — NEW since the FINAL probe), f44
**`bdd1cc79a`** (FINAL docs — NEW), g90 **`94d33f2ca`** (reconcile §7 —
NEW), g87 origin tip still `dfbcaf94b` but its worktree carries the
closeout commit **`26620538e`** (local, unpushed at this probe). f45
origin tip `6352d3840` unchanged. No `g87b`/`f44f` ref exists on origin
(full `git branch -r` census) and no `research/143` doc exists on any
read tip — the Lead's "g87b-143" and "f44f-active" names map to the g87
closeout commit and the f44 FINAL docs respectively (§11.3/§11.2).

### 11.1 g88-NOW-landed — the six-line rule IN CODE + the v11 torch line (receipts)

**The FINAL-probe re-flag (§6/§10.6) is RETIRED.** g88 fold 7
(`a9f1c29d8`, "persist fold 7 FINAL: the v11 TORCH FOLD + the 135/132
reconciliation") landed BOTH reader rules this doc demanded, verified by
direct reads of `crates/tr-app/src/persist.rs` at that tip:

1. **135 §2's six-line v7 recognition — LANDED.** `:462-465`:
   `let nonempty: Vec<&Vec<&str>> = post.iter().filter(|t| !t.is_empty())…;
   let v7_six_line = v7_flags && nonempty.len() == 6 &&
   nonempty.iter().all(|t| t.len() == 1);` — `:470-479` reads positionally
   bread/crystal/arcane/pearl/gummy/ambrosia (the `.plr` `:54779` +
   `:54781-54786` order minus g89's fruit), then `post.clear()`. Exactly
   §6's demanded rule, on the non-empty projection (blank lines skipped —
   strictly more tolerant than the raw `post.len() == 6` spelling, same
   verdict on every pushed writer shape; the header comment enumerates
   the disjointness: g89 flag-only 1×1, g90 bread-only 1×1, g88 life-only
   1×2 with halves ≥100, the 3-line superset 1+2+2, g55 loadout-only
   2+41+41+41).
2. **The bounded-misread witness RETIRED.** Zero `bounded_misread`
   matches in the tip's `persist.rs` (grep census); replaced by
   `g90bs_six_line_v7_loads_positionally` (`:1198`, asserts the six flags
   read + `!s.unlocked_biome_torches` "no torch pair either" `:1223`).
3. **The f33 lone-all-0/1-pair rule — LANDED (a second recognition this
   doc had not demanded).** `:466-469` + `:485-488`: a lone post-dye
   2-token line with all-"0"/"1" tokens is the torch pair (no maxima pair
   can be all-0/1 — the permanent halves start 100/20). Witness
   `f33s_torch_only_v7_loads_positionally` (`:1233`, asserts both halves
   read `:1247-1248`). This closes f44's 132 §2d v7-fork collision from
   the READER side.
4. **The v11 torch line — LANDED.** Writer `:290-304`: header `"11\n"`
   (`:243`), the pair appended AFTER the loadout block as the LAST
   post-dye line — `u8::from(player.unlocked_biome_torches)` +
   `u8::from(player.using_biome_torches())` (the GETTER — the slot-11
   property, f44 `63204f852`'s L1 semantics mirrored). Reader `:500-514`:
   v11 takes the LAST non-empty 2-token line AFTER the last set line;
   `:522-524` reads the pair. NOT unlock tokens — token 7 stays
   DD2-RESERVED (a DD2 lane bumps v12). Back-read witness
   `the_v10_superset_still_loads_without_the_torch_pair` (`:1261`,
   `:1288-1289` assert the v10 block ends at the sets). The restore rides
   `Player.using_biome_torches()`/`set_using_biome_torches` at f33's
   exact spot after `luck()` (`player.rs`, +21 lines) + `app/mod.rs`
   (+11 lines) — no second field.
5. **103 marked FINAL** with §9 the full merger persist-checklist; its
   §8 alignment note records the 135 §3 token-order reconciliation (the
   v10 written crystal=4/arcane=5/pearl=6 order WINS — §6's verdict
   cross-receipted from g88's side). 122 gains the v11 post-receipt note.
   The f44c-G7 redundancy is adjudicated as a DELIBERATE dual-carrier
   with a CONDITIONAL DROP (103 §2b: drop the sidecar torch line when
   character-load hydration AND the exit-save both land; restore
   precedence per 132 §2c — character first, sidecar second, sidecar
   wins as newer state).

**a6's §6 merge action is SATISFIED — no `.trplayer` reader debt remains
on the powerup cluster.** The persist-split (§2 L2) and merge-order
(§8 step 2) rows are re-pinned v10 → v11 above. g88's fold-7 commit
message still records "a6b research/138 STILL unpushed (a6 tip
8ed96ee5)" — true at ITS probe; 138 is pushed now (`d7de8bd53` +
`115ce655a`), and this §11 is the cross-receipt g88's 103 §9 checklist
can point at.

### 11.2 The 68-vs-97-byte union — the UNIFIED input (f44 FINAL aligned, delta published)

**The Lead's question:** this doc's §5a published a 68-byte five-way union
that EXCLUDED g72's 31-byte cave block, g69's 1-byte crimson and d125's
1-byte downedEmpress (and d109's dense variants); the f44 FINAL
(`bdd1cc79a`, research/132 §3-L3 — the "f44f-active consolidating" doc;
no `f44f` ref exists, the FINAL docs ARE `bdd1cc79a` on
`origin/task/W4-f44-save-plr-2`) seats them and publishes a **97-byte
head**. Is the unified input 97 + g87's 3 = 100? **No — 101, and f44's
own 97 is off by one.** The delta, with the arithmetic receipt:

1. **f44's component sum is 98, not 97.** Its §3-L3 lists
   `16+31+3+1+1+2+44` and calls it 97; the sum is **98**. Its own layout
   table shows the slip: `[54..97] c43 identity block (… = 44 B)` seats a
   44-byte block in a 43-byte span — the identity block actually ends at
   **98**, and the dense variants start at 98, not 97. Every "97" in 132
   §3-L3 (the head total, `v18_tail_len`, the truncation-constant moves,
   the §5 back-compat row) carries the same off-by-one.
2. **g87's 3-byte pet block is NOT in f44's claimant set.** 132 §3-L3
   seats seven (g70/g72/g71/g69/d125/g90/c43) + d109 dense; its §3-L3
   step 3 knows g87's `.wld` pet flips only as the a6/f45 dedup — the
   `.trwld` pets never got a seat. This doc's §5a owns that claim
   (g87 `dfbcaf94b`'s pushed 3-byte v18 tail). Seat: **between g69's
   crimson and d125's empress** — the vanilla write order
   `WorldFile.cs:1392-1394` (boughtCat/Dog/Bunny) precedes `:1395`
   (downedEmpressOfLight), the same 92 §1 "least-surprising C# field
   order" rule §5a already applied pets-before-books.
3. **f45's identity block IS c43's** — one seat, not two. Verified at
   both tips: c43 `076970809` `save.rs` writes
   `generator_version u64 (:1225) → unique_id [u8;16] (:1226) →
   world_id i32 (:1227) → creation_time i64 (:1250) → last_played i64
   (:1251)`; f45 `6352d3840`'s block is byte-order-identical (c43 is in
   f45's lineage, 132 §0 records it). 44 bytes once.

**THE UNIFIED v18 INPUT — fixed head 101 bytes + W×H dense (the merger
table, superseding §5a and correcting 132 §3-L3's FINAL 97; itself
SUPERSEDED by §12's 107-authority alignment — kept as the retire-probe
receipt):**

| Offset | Len | Field | Rust type | Owner block | C# anchor |
| --- | --- | --- | --- | --- | --- |
| 0..12 | 12 | `tree_x[0..3]` | 3×i32 LE | g70 forest | `WorldFile.cs:1253-1255` |
| 12..16 | 4 | `forest_bg_styles[0..4]` (treeBG1..4) | 4×u8 | g70 forest | `:1318`/`:1377-1379` |
| 16..28 | 12 | `cave_back_x[0..3]` | 3×i32 LE | g72 cave | `:1260-1262` |
| 28..44 | 16 | `cave_back_style[0..4]` | 4×i32 LE | g72 cave | `:1263-1266` |
| 44..47 | 3 | ice/jungle/hell back styles | 3×u8 | g72 cave | `:1267-1269` |
| 47..50 | 3 | snow/desert/ocean bg styles | 3×u8 | g71 surface | `:1321`/`:1324`/`:1325` |
| 50 | 1 | `crimson` | u8 | g69 | `:1281` |
| 51 | 1 | `bought_cat` | u8 bool | **g87 pets** | `:1392` |
| 52 | 1 | `bought_dog` | u8 bool | **g87 pets** | `:1393` |
| 53 | 1 | `bought_bunny` | u8 bool | **g87 pets** | `:1394` |
| 54 | 1 | `downed_empress_of_light` | u8 bool | d125 | `:1395` |
| 55 | 1 | `combat_book_volume_two_was_used` | u8 bool | g90 books | `:1407` |
| 56 | 1 | `peddlers_satchel_was_used` | u8 bool | g90 books | `:1408` |
| 57..65 | 8 | `generator_version` | u64 LE | f45/c43 identity | `:1225` |
| 65..81 | 16 | `unique_id` (GUID bytes) | [u8;16] | f45/c43 identity | `:1226` |
| 81..85 | 4 | `world_id` | i32 LE | f45/c43 identity | `:1227` |
| 85..93 | 8 | `creation_time` | i64 LE (ToBinary) | f45/c43 identity | `:1250` |
| 93..101 | 8 | `last_played` | i64 LE (ToBinary) | f45/c43 identity | `:1251` |
| 101..101+W×H | W×H | framing variants (dense) | u8/tile | d109, LAST | port-only (`Tile.cs:464-482` header2 bits) |

**Head = 16+31+3+1+3+1+2+44 = 101. Total tail = 101 + width×height.**
VERSION stays 18, forward-only (no released v18 — every claimant is
unmerged task work; the nine single-claimant v18 shapes are EOF/garbage
on the union reader, 132 §5's row stands with 97→101).

**Claimant witnesses verified at the fetched origin tips** (direct
`git show <tip>:crates/tr-core/src/save.rs` reads): g72 `4a75715ef`
writes `cave_back_x[0..3]` i32×3 + `cave_back_style` i32×4 + the
ice/jungle/hell u8 trio (its round-trip witness pins `cave_back_x[3]`
NOT persisted — the port's array is 4-wide, the file carries 3); g69
`20d0380f9` writes `u8::from(world.crimson)`; d125 `d8047a109` writes
`u8::from(world.downed_empress_of_light)`; d109 `81894095e` writes the
dense `width*height` variants block LAST; c43 `076970809` the identity
quintet above. g70/g71/g90/g87/f45's blocks are §5a's rows verbatim.

**Truncation-constant moves (the unified re-pin):** every lane's fixed
addend goes to **101** (g70/g71's 16/3, g69/d125's 1, g90's 2, g87's 3,
g72's 31, c43/f45's 44 — the per-lane site lists in §5a + 132 §3-L3 +
64-add §3 + 135 §4 stay the site inventories, only the target number
changes 97→101); the END-RELATIVE probes subtract `101 + width*height`
(d109's dense term rides `v18_tail_len(width, height)` — the helper f44's
map already names, with the corrected return). Every round-trip witness
keeps its field assertions; only the tail lengths move.

**The underworldBG seat stays OPEN** (132 §3-L3's flag, C# `:1376`): if
g71c's roll-wire persists it, insert 1 byte in the surface group →
**102-byte head**, re-pin once. The g71c/g72c roll keep-one adjudication
(132 §3-L3 step 2) is untouched by this delta — it is a worldgen-roll
question, not a layout one; the persistence trio stays g72c's pushed
shape either way. *(Seal receipt §12: the seat FIRED — g71's pushed
`f08e8d41e` wiring rolls + `.wld`-persists it; the final head is 107 with
the moon-boss five also seated. Both re-pins landed in §12.)*

**Merge action (retire-probe wording, seal-superseded):** the g71c union
owner takes THIS table (not §5a's 68, not 132's 97); f44's 132 §3-L3
needs the one-byte correction receipted here (flag to the f44 lane — a6
edits no foreign doc); g87's pets seat 51-53, g90's pair 55/56, the
identity block 57..101, d109 dense LAST. *(§12.1: the seats moved again
in the SEALED 107 — pets 59-61, books 51/52, identity 63..107; the
correction flag to f44 now covers its divergent 107 too.)*

### 11.3 g87b-143 — the closeout cross-receipt + a6's exchange-fail fold

**Ref census:** no `g87b` branch exists on origin (full `git branch -r`),
no `research/143` doc exists on any read tip (g87's tree carries only the
base research set; the sole `143-*` file in the whole ref database is
c4's d40-465 retire hunk on `task/W4-c4-style44-flying-fish`, unrelated).
The Lead's "g87b-143 / g87b active closeout" is the **g87 lane's closeout
commit `26620538e`** — local in `TR-W4-g87` over the pushed `dfbcaf94b`
at this probe (its `permanent_upgrades.rs` cites "research/143" as the
fold-input doc it intends to publish; the doc is not yet written). a6
receipts the COMMIT, not the phantom ref.

**The closeout folds ALL THREE a6 deltas** (verified by the full diff
read + the committed file at `26620538e`, +243/-63 on
`permanent_upgrades.rs`):

1. **Demon Heart start refusal** — `use_start_refused` added
   (`Player.cs:52051-52054` via `TryStartUse :52021-52023`, citing a6
   `ea478936`); the press edge returns before the swing; the
   `ApplyItemAnimation` reuseDelay tail (`:4496`) added (a6's machine
   already carried it, `powerups.rs:606-607`). Witnesses re-pinned: the
   classic + used-heart refusals now assert NO swing
   (`w4_g87_the_demon_heart_refuses_in_classic_mode` /
   `w4_g87_a_used_demon_heart_refuses_the_start`). **§10.1's fidelity gap
   is CLOSED on g87's side.**
2. **QuickSpawnItem full walk** — `deposit_pickup` + per-slot
   chime/`NotifyItemPickup` + void-vault LAST resort + prefix-preserved
   overflow with the pinned velocity triplet; `Prefix(-1)` re-pinned
   UpdateItems → **UpdatePlayers** (the use's swap, citing a6
   `bb36e6ee`); the Mech non-prefixable zero-draw census witness added.
   **§10.2's gaps CLOSED.** The re-use witness now pins the SECOND Mech
   (`:45204` no flag gate — both lanes agree, the census's "flag-gate"
   reading is wrong).
3. **Pet licenses** — the hardcoded `any_npcs = false` replaced by a6's
   LIVE net-id scan (`d46221b2`'s shape), the `!bought` guard added, and
   the bought-and-resident pass posts **`Misc.PetExchangeFail`**
   (`NPC.cs:80914-80917`) — the fail-chat leg a6 lacked. Witness
   `w4_g87_a_resident_non_pet_does_not_open_the_exchange` (a slime does
   not open the cat gate).

**a6's reciprocal fold (LANDED this pass, `powerups.rs`):** the
`use_pet_license` else-arm now posts `Misc.PetExchangeFail` in the World
colour on the bought-and-resident pass (the C# always chats; the generic
tail consumes one either way, `:45442`). The doc comment's deviation
paragraph re-written: the silent-consume leg is GONE; the remaining debt
is the **Success leg only** (`RerollVariationForNPCType` →
`RollVariation` + `Misc.PetExchangeSuccess`, `:80910-80912`), shared with
g87, landing WITH the town-pet kinds + `TownNPCProfiles`. New witness
`a_resident_non_pet_does_not_open_the_exchange` (g87's twin). The
positive fail-chat path stays unwitnessable end-to-end until a ported
kind carries net id 637/638/656 — the same boundary g87's closeout
records. ANALYSIS's pet-license + g87 bullets updated to match.

**Cross-receipt verdict:** the §10.1-10.3 keep-owns STAND (a6's
single-seam `powerups.rs` machine owns the cluster; g87's module dedups
onto it at the merge) — but the dedup is now **shape-identical on all
three families**, no semantic conflict remains. The exchange-fail debt
is SHARED and half-closed: both lanes post the Fail line today; the
Success dispatch is the one remaining leg, flagged to the town-pet lane
(`Main.cs:65900-65911` arrival + `TownNPCProfiles.RollVariation`). g87's
`dh` sidecar DROP (§10.4) is untouched by the closeout (its
`persist.rs` still writes v7 + `dh` at `26620538e` — verified).

### 11.4 The FINAL-FINAL receipts — a6's five fidelity fixes + the g90 cross-note

**The five §3 fixes re-verified at the retire tip** (direct file reads,
the shifted line cites re-pinned in §10.7 above): row-targeted dispatch
(`achievements.rs:445` + `powerups.rs:177/179/362/531/533`, witness
`:634`), pearl Player-field ownership (`player.rs:2171/:2978/:10651`,
the 5340 arm `powerups.rs:489`, witness `:1314-1316`), the aegis-crystal
regen slot (`regen.rs:117/:194-196` between `:190` and `:197`, witness
`:688`), the booster order (`player.rs:6916/6919/6922`), the TRAINING_DAY
hook (`world_sim/player.rs:1625-1627`, witness `powerups.rs:1324`). All
five PRESENT, code + witness. The retire pass's own code delta is the
§11.3 exchange-fail fold + its twin witness — no other a6 behaviour
changed.

**g90's reconcile cross-note (`94d33f2ca`, research/135 §7) receipted:**
it read a6 at `8ed96ee5` (138 not yet pushed) and proposed a keep-one
split — "a6 keeps its exclusives 29/109/1291/3335/5289/5043/licenses/
drones, g90 keeps the carriers + consumers + the town feeder". This
doc's §1 verdicts SUPERSEDE that proposal on two rows: **29/109/1291 go
to g88 keep-own** (103 §5a — the `.trplayer` carrier lives there) and
**5326 goes to g90 keep-own** (the skeleton-merchant consumer); a6 keeps
3335/5289/5043/licenses/drones + the shimmer machine + the combat-book/
satchel cluster seam. g90's carrier/consumer/feeder ownership is
UNCHANGED by §1 (the persist split §2 already routes every carrier to
f44/g88/f45/g90). No conflict — the proposal predates the published
adjudication; the merger takes §1 + §8.

### 11.5 FINAL-FINAL verdict summary + verification performed

**Verdicts (the a6 lane's final state):**
- **Use arms:** a6's `powerups.rs` machine owns the cluster EXCEPT
  29/109/1291 (g88) and 5326 (g90). a6 KEEPS 3335/5289/4829-30-4910 —
  g87's closeout made its three arms shape-identical, so the merge dedup
  is mechanical (§11.3). The exchange **Success** leg is the one shared
  use-arm debt (town-pet lane).
- **L1 `.plr`:** f44 base (`bdd1cc79a` lineage, the complete walk + the
  slot-11 property fix `63204f852`); a6 contributes no writer.
- **L2 `.trplayer`:** g88 **v11** base (`a9f1c29d8`); the six-line v7 rule
  + the f33 lone-pair rule + the torch line all LANDED — **no reader debt
  remains** (§11.1). g87's `dh` line DROPS (§10.4).
- **L3 `.wld`:** f45 base; the five contested bytes dedup (§4).
- **L3 `.trwld`:** the retire probe's **UNIFIED 101-byte head + W×H
  dense** (§11.2) — f44's 97 corrected to 98 by its own sum, + g87's 3
  pets seated 51-53; the underworldBG seat OPEN (102 if seated).
  *(Seal §12.1: the seat FIRED and the moon five landed — the SEALED
  head is **107**, g71d 92 §1-FULL authority, pets 59-61 / books 51/52 /
  identity 63..107. The g71c/g72c roll keep-one is ADJUDICATED on both
  new tips: g72's `cave_bg.rs` keeps the cave roll, g71 `fc76a48b`
  dropped its duplicate — 92 §2.1-FINAL + 132 §3-L3 rule 2 agree.)*
- **Open merge actions remaining (seal-updated):** the 107-union landing
  (g71d owner, 92 §1-FULL.2's table per §12.1), the f44 divergent-107
  re-pin receipt (§12.1 — its 97→98 correction already landed in
  `d35876af9`), the exchange Success leg (§12.2's four-step activation
  contract, d43/f43 lanes). The `.trplayer` reader gap is CLOSED; the
  roll keep-one is CLOSED.

**Verification performed (retire pass):**
- `git fetch origin --prune` + full `git branch -r` census (no g87b/f44f
  refs); `git worktree list` + direct worktree status reads (g87's
  closeout found local, then committed `26620538e` mid-probe).
- Claimant reads at fetched tips: g88 `a9f1c29d8` `persist.rs` (the
  v7/v11 reader + writer + witnesses, line-cited §11.1), f44 `bdd1cc79a`
  research/132 §3-L3 (the 97-byte claim + its component sum), g90
  `94d33f2ca` (the §7 cross-note), g87 `26620538e` (the full closeout
  diff + committed file), g72 `4a75715ef` / g69 `20d0380f9` / d125
  `d8047a109` / d109 `81894095e` / c43 `076970809` `save.rs` writers
  (§11.2's claimant receipts), f45 `6352d3840` (unchanged).
- C# re-reads (direct file paths, ≤100-line windows, no junction
  recursion): `NPC.cs:80880-80979` (UnlockOrExchangePet +
  RerollVariationForNPCType + RerollVariation), `Player.cs:45428-45452`
  (LicenseOrExchangePet), `IO/WorldFile.cs:1200-1284` + `:1320-1419`
  (the header/flags write order — the pets-before-empress seat).
- a6 edits: `powerups.rs` (the fail-chat fold + the twin witness + the
  deviation comment), ANALYSIS.md (the two bullets), this doc. No
  cargo/tests/rustfmt/graphify per the lane constraint — witnesses
  WRITTEN, NOT RUN; presence verified by direct file reads.

## 12. SEAL pass (2026-11-21, over `6568aeeb6`) — the 107-authority alignment + the Success-leg activation contract

The seal probe fetched the LATEST origin tips (the retire probe's
`bdd1cc79a`/`94d33f2ca` reads are now superseded on the union question):
g71 **`f2d8c2c67`** (research/92 §1-FULL — the 13-branch claimant registry
+ the 107-byte union, 10:03), f44 **`d35876af9`** (research/132
FINAL-FINAL §3-L3 — ALSO 107, 10:17), g88 **`6a829a0e8`** (receipts phase,
v11 code unchanged at `a9f1c29d8`), g90 **`740650d75`** (135 FINAL, v18
pair unchanged), f43 **`79a037f5e`** (the town-spawn census), d43 tip =
main `a725acb04` (zero commits of its own). g87 origin tip still
`dfbcaf94b`.

### 12.1 The 107-byte landed receipt — §11.2's 101 re-aligned, a6's seats corrected

**The arithmetic bridge:** a6's retire-probe 101 (§11.2) + c39/f39's
moon-boss five (5 B, `WorldFile.cs:1347-1351`, ONE deduped block —
byte-identical claims, 92 §1-FULL.1 #7/#8) + g71's underworld byte (1 B,
`:1376` — §11.2's OPEN seat FIRED: the pushed `f08e8d41e` wiring rolls +
`.wld`-persists it) = **107 EXACTLY**. 16+31+3+1+2+5+1+3+1+44 = 107.
Total tail = **107 + width×height** (d109 dense LAST, unchanged).

**The two published 107 layouts DIVERGE on offsets — adjudicated:**

| Seat | a6 §11.2 (101) | g71d 92 §1-FULL.2 (107, 10:03) | f44d 132 §3-L3 (107, 10:17) |
| --- | --- | --- | --- |
| forest / cave / surface | 0..16 / 16..47 / 47..50 | SAME | SAME |
| g90 books | 55/56 | **51/52** (inside the kept-verbatim 64a 53-byte prefix) | **61/62** (strict C# order) |
| crimson | 50 | 50 | **51** |
| underworld | OPEN | **58** | **50** (surface-group extension) |
| moon bosses | absent | 53..58 | 52..57 |
| g87 pets | 51-53 | **59-61** | **57-59** |
| empress | 54 | 62 | 60 |
| identity | 57..101 | 63..107 | 63..107 |

**Verdict: g71d's 92 §1-FULL.2 table is THE merger input.** Reasons, in
order: (1) the Lead's seal directive names 92 §1-FULL the landed
107-authority; (2) g71d's tip is the UNION OWNER lane (its §1-FULL.4
carries the full per-branch truncation-move table + the writer/reader
pair ready to land, and its own `save.rs` at `f2d8c2c67` still writes
only the 3-byte surface tail `:434-441` / reads `:826-830` — the 107 is
published as the merge instruction, exactly the shape §11.2 published
its 101); (3) keeping g72's 64a 53-byte prefix VERBATIM at [0..53] holds
every offset 64a already published stable — f44d's strict-C#-order
re-shuffle moves crimson/books for zero fidelity gain (both orders are
forward-only-legal; no released v18 exists). f44d's own doc claims "THE
authoritative union" — a6 edits no foreign doc; this receipt flags the
divergence to the f44 lane: **132 §3-L3's [50]/[51]/[52..57]/[57..60]/
[61..63] seats re-pin to 92 §1-FULL.2's [58]/[50]/[53..58]/[59..62]/
[51..53]** (the identity seat 63..107 and the dense term already agree).

**a6's own seats re-checked against 92 §1-FULL — DIVERGENT, corrected:**
§11.2 seated pets 51-53 / books 55/56 / empress 54 / identity 57..101
(the moon five + underworld were absent/open). In the 107-authority:
**g87 pets 59-61, g90 books 51/52, empress 62, moon 53-57, underworld 58,
identity 63..107, dense 107..107+W×H.** The pets-before-empress C# order
(`:1392-1394` < `:1395`) survives inside the extension group; the
books-before-pets file order is 64a's published choice, NOT a fidelity
error (the `.trwld` tail is port-only — only the FIELD SET must be
C#-complete, and §1-FULL.1's 13-branch sweep proves it is). Every a6
mention of 51-53/55/56/57..101/101/102 outside §11.2's own audit-trail
table is superseded by THIS section (the §1/§2/§5/§5a/§8/§10/§11
banners now point here). Truncation constants: every lane's fixed
addend → **107** (92 §1-FULL.4's per-site table is the inventory; a6's
§5a/§11.2 site lists stay valid as site lists, only the target moves
101→107); end-relative probes subtract `107 + width*height`.

### 12.2 The Success-leg debt — the activation contract (SHARED a6 × g87, consumers d43/f43)

**Census (the town-pet-kind question the Lead asked):** the Success chat
(`Misc.PetExchangeSuccess`, `NPC.cs:80910-80912`) fires only when
`RerollVariationForNPCType(npcType)` finds a resident of net id
637/638/656 AND its `RerollVariation()` succeeds (`:80920-80961` —
`TownNPCProfiles.GetProfile` gate `:80935-80938`, the ≤100-draw
`profile.RollVariation()` loop `:80941-80944`, `GetNameForVariant` +
`life = lifeMax` + the PetExchange particle + `TrySendData(56)`
`:80949-80959`). Direct C# window re-read this pass:
`NPC.cs:80880-80979`. **No lane can witness the Success leg alone:**

- **main `a725acb04`** (== d43's tip — `task/W4-d43-npc-kinds-631-697`
  carries ZERO commits of its own): `npc/data.rs`'s `net_id` map has NO
  arm for 637/638/656 (or any town slime 670/678-684) — the highest town
  ids mapped are Zoologist 633 / Princess 663 (`:3023-3024`); the
  town-pet AI helpers exist (`npc_ai_town.rs:137-145` `is_town_pet` /
  `is_town_slime`, id-keyed pure fns) but no `NpcKind` carries those
  ids, so a6/g87's LIVE `AnyNPCs` scans never answer (both lanes'
  recorded boundary, re-verified).
- **f43 `79a037f5e`** (`town_arrival_pets.rs`, 819 lines, UNWIRED
  `#![allow(dead_code)]`): ports the WHOLE exchange chain as pure fns —
  `unlock_or_exchange_pet` `:87-113` (the `PetUseOutcome::ExchangeSuccess`
  arm `:74`/`:108-109`), `reroll_variation` `:126-148` (the 100-draw
  cap), `reroll_variation_for_npc_type` `:150-184` (first-resident-only,
  the no-profile false `:158-161`, the legacy-0 roll `:162-171`, the
  heal `:178-179`), the six-variant rosters + head ids 27-44
  (`:196-238`), `has_town_profile`'s 40-key set (`:255-298`), the
  genRand name categories (`:307-330`), and the arrival gates
  `tail_can_spawn` (`:398-414`: 637/638/656 on the bought flags,
  `Main.cs:65900-65911`). It even records the load-side gap: the port's
  `.wld` reader SKIPS the bought trio (`flags.rs:608` `r.skip(3)` per
  its census note `:332-337`) — f45's flags superset closes that.

**THE ACTIVATION CONTRACT (published to d43/f43 + the g87 merge):** the
Success leg activates when ALL FOUR land, in this dependency order:

1. **d43 (kinds):** `NpcKind::TownCat/TownDog/TownBunny` with `net_id`
   637/638/656 (+ the town slimes 670/678-684 for the arrival ladder) —
   the `npc/data.rs` map arms + `from_u16` seats. Until this lands the
   live scans stay dead and BOTH lanes correctly refuse/chat-Fail only.
2. **f43 (arrival):** splice `tail_can_spawn`'s pet arms into
   `town_arrival.rs`'s machine at the cited seats (`Main.cs:65900-65911`)
   so a bought flag actually spawns the resident; the `.wld` bought-trio
   load rides f45's flags base (§4 — NOT a new writer).
3. **f43 (profiles):** map `PetVariationSlot` onto the live `Npc`
   (`variation_index`/`hp`/`max_hp`/`given_name` per its own `:186-194`
   note) and wire `reroll_variation_for_npc_type` behind the use arm,
   the roll on **`Main.rand`** (`Profiles.cs:216` `Next(6)` — the ONLY
   Main.rand slot in the chain) and the name draw on **`WorldGen.genRand`**
   (`LanguageManager.cs:419` — a DIFFERENT stream; f43's census pins both).
4. **a6/g87 (chat):** the merged `use_pet_license` (a6's machine, §10.3/
   §11.3 keep-own) swaps its hardcoded Fail else-arm for the reroll
   result: true → `Misc.PetExchangeSuccess`, false → `Misc.PetExchangeFail`
   (both already post Fail today — the Success dispatch is a ONE-LINE
   conditional at activation, `powerups.rs:439-448`'s else-arm). The
   consume-either-way tail (`:45442`) and the `!bought` guard are
   already shape-identical on both lanes; nothing else moves.

**Witness boundary (unchanged, now with the contract):** a6's
`a_resident_non_pet_does_not_open_the_exchange` + g87's twin stay the
negative-path pins; the positive Success witness lands WITH step 1
(a kind carrying 637/638/656) — f43's pure-fn witnesses
(`reroll_variation_for_npc_type` success/fail/legacy, `:656-672`) already
pin the chain's semantics pre-activation. **No a6 code change this
phase** — the else-arm's deviation comment already names the debt; this
section is the published contract the town-pet lanes consume.

### 12.3 The powerup-family bible (one paragraph)

The powerup family seals as a five-lane machine with one owner per seam:
**a6** owns the single-seam use-arm cluster (`crates/tr-app/src/powerups.rs`
— 3335 Demon Heart with the `CheckCanUse` start refusal, 5289 Minecart
with the full QuickSpawnItem walk, 4382/5336 combat books, 5043 Torch
God's Favor with the faithful slot-11 setter, 5343 satchel, the six
shimmer permanents 5337-5342, the pet licenses 4829/4830/4910 with the
live scan + exchange-Fail chat, and the 5451/5738 drone-destroy checks)
plus the five landed fidelity fixes (§3/§10.7/§11.4) and the shared
`complete_condition` row-targeted dispatch; **g90** keeps the 5326
Artisan Loaf arm + the skeleton-merchant consumer and owns the carrier/
consumer/feeder shapes a6 matches (pearl mirror, regen slots, booster
order, TRAINING_DAY); **g88** keeps 29/109/1291 (the `.trplayer` carrier
lives there) and owns the L2 sidecar at **v11** (`a9f1c29d8` code, the
six-line v7 rule + f33 lone-pair rule + torch line 16 all landed — zero
reader debt, g87's `dh` line DROPS per §10.4); **f44** owns the L1 `.plr`
complete walk (`used_upgrades[6]` + the L1-only fields) and **f45** the
L3 `.wld` flags superset (the five contested bytes dedup, §4); the L3
`.trwld` rides the **107-byte-head + W×H dense v18 union** under g71d's
92 §1-FULL.2 authority (§12.1 — 13 claimants, 11 blocks, three dup pairs
at zero layout cost, forward-only); the ONE remaining use-arm debt is
the exchange **Success** leg, activation-contracted to d43's kinds +
f43's arrival/profiles wiring (§12.2). Every keep-own stands; the merge
is mechanical everywhere except the four-step activation contract.

**Seal verification:** fetched-tip reads g71 `f2d8c2c67` (92 §1-FULL
full + `save.rs:428-447`/`:822-833` — the 107 is published-not-coded,
the surface tail still 3 B), f44 `d35876af9` (132 §3-L3's divergent
107 table, `:557-599`), g88 `6a829a0e8` + g90 `740650d75` (tips moved,
code unchanged), f43 `79a037f5e` (`town_arrival_pets.rs` full read),
d43 = main `a725acb04` (`npc/data.rs` net-id census, `town_arrival.rs`
gate census); C# window `NPC.cs:80880-80979` (≤100 lines, direct path).
a6 edits this pass: this doc (§11.2 banners + §12) + the INDEX row.
Docs-only — no cargo/tests/rustfmt/graphify per the lane constraint.

This doc is SEALED for the a6 lane (FINAL-FINAL + the §12 seal).
