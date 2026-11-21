# 138 — POWERUP 4-WAY MASTER RECONCILE (a6 ⨉ g87/g88/g89/g90 ⨉ c36/f33 ⨉ f44/f45)

**Status: FINAL.** Written by W4-a6 reconcile (branch
`task/W4-a6-crystals-powerups`, base `8ed96ee5`); the FINAL pass (§10)
re-adjudicates the NOW-PUSHED g87 `dfbcaf94b` arm-by-arm, re-flags the g88
v7 recognition gap, publishes the exact 68-byte v18 union byte table
(§5a) and receipts a6's five fidelity fixes (§10.7). This doc sits BESIDE the
persist contracts — research/99 + 132 (`.plr`, f44), research/103 v10 + 122
(`.trplayer`, g88/g89), research/135 (`.trwld`/`.trplayer` fold-input, g90),
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
| `origin/task/W4-g88-life-fruit-use` | `c8985b534` (fold 6, v10) | 29/109/1291 use arms (`crystals.rs`/`life_fruit.rs`) + `.trplayer` **v10 SUPERSET** writer/reader + research/103 v10 + 122; carries the crystal flags as tokens 4/5/6 |
| `origin/task/W4-g89-aegis-fruit-use` | `598642563` (g89f) | 5338/5341/5342 use arms + 4382/5336 combat-book arms (g89e) + `.plr` fruit/gummy/ambrosia bools + `.wld` book2 byte + `.trplayer` v9 lineage + research/122; g89f adds Rod of Discord/Harmony (out of scope) |
| `origin/task/W4-g90-artisan-bread-use` | `b3b82d071` (reconcile-input) | 5326 loaf + 5337/5339/5340/5341/5342 use arms + 4382/5336/5343 + consumers (regen `aegis_crystal`/`arcane_crystal` slots, pearl mirror, skeleton-merchant shop) + `.plr` bools 1/3/4/5/6 + bread + `.trwld` v18 2-byte tail + net sync + research/135 |
| `origin/task/W4-c36-achievement-notify-save` | `585fa0a84` | 5043 use-credit (`complete_condition` GAIN_TORCH_GODS_FAVOR) + 29/109/1291 `use_permanent_powerup` + the row-targeted `complete_condition` helper + research/30 |
| `origin/task/W4-f33-torch-god-favor` | `333479e0b` | the Torch God EVENT machine (`torch_god.rs`) + the 5043 flag READS (`has_torch_gods_favor` scan, `ctx.unlocked_biome_torches`); completes NO achievement |
| `origin/task/W4-f44-save-plr-2` | `05e4020bb` (over `d331ea7c`) | the COMPLETE `.plr` walk (research/99, 71 rows) incl. `used_upgrades[6]` array + `extra_accessory` + torch pair + bread + dd2 + counters; research/132 master map |
| `origin/task/W4-f45-wld-header-2` | `6352d3840` | the COMPLETE `.wld` flags block (pets 2021-23, book2 2036, satchel 2037, nurse, slime green/old, …) + a `.trwld` **v18 44-byte identity block** |
| `origin/task/W4-g70-bg-layers-1` / `g71-bg-layers-2` | (92 §1) | `.trwld` v18 forest 16B / surface 3B → the 19-byte union |

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
| **4382 / 5336** Combat Book + Vol Two | `:45322-45350` | a6, g89e, g90 | All three port the two independent WORLD-flag once-ever gates, ApplyItemTime 45, the `Misc.CombatBookUsed`/`Misc.CombatBookVolumeTwoUsed` ChatColors.World line (vol-two key is a `{$Misc.CombatBookUsed}` copy command), consume + TurnToAir. MP SendData(61,-11/-17) folds out. Equal fidelity (kinds disjoint from every other arm → press-edge position observation-identical). | **keep ONE** — recommend **a6** (`use_combat_book`, the single cluster seam) or g89e (pushed precedent); dedup the other two. The TRAINING_DAY progression-40 consumer (`:25695-25698`) lands once (§3.5, a6 now carries it; g89e/g90 also do). | `.wld` book2 byte tail **2036** gate 259: **f45** supersedes a6/g89e/g90 (§4). book1 tail 1939 gate 204 pre-existing (i35). `.trwld` book2: g90 v18 union offset **22** (§5a — moved from 19 by g87's pet trio). NO sidecar token (world global, 122 §5). |
| **5043** Torch God's Favor | `:45226-45243` | a6, c36, f33 | **a6 MORE faithful than c36**: the C# sets `UsingBiomeTorches = true` whose SETTER writes `builderAccStatus[11] = 0` (`:3960-3963`); a6 does BOTH (`unlocked_biome_torches = true` + `builder_acc_status[11] = 0`), c36 folds UsingBiomeTorches into the unlocked flag (its research/30 deviation). a6 also ports HandleSpecialEvent(24) + the SP bestiary TorchGod(664) RegisterKill. **Fixed §3.1**: a6's credit now row-targeted `complete_condition("GAIN_TORCH_GODS_FAVOR","Use")` (== c36). f33 owns the EVENT machine + flag READS, completes no achievement. | **a6** use arm (the faithful setter) — **supersedes c36 research/30's "keep c36's write" note** (C# re-read `:3960-3963`). **c36**'s `complete_condition` helper is the shared dispatch (a6 now carries the identical helper, dedup). **f33** event machine + `has_torch_gods_favor`/`unlocked_biome_torches` reads. Three-way split by responsibility, no overlap to delete. | `.plr` torch pair `:54777-54778` gate 229: **f44** (`unlocked_biome_torches` + `using_biome_torches`, L1-only). NO sidecar. No `.wld`. |
| **5343** Peddler's Satchel | `:45368-45383` | a6, g90 | Both port the WORLD-flag once-ever gate, ApplyItemTime, `Misc.PeddlersSatchelUsed` World line, consume; the travel-shop +1 slot consumer `Chest.cs:1303-1306` (plain `num++`, NO RNG draw). Equal fidelity. | **keep ONE** — recommend **a6** (`use_peddlers_satchel`) or g90; dedup. The shop +1 consumer is identical on both (`shop.rs::setup_travel_shop`). | `.wld` satchel byte tail **2037** gate 260: **f45** supersedes a6/g90 (§4). `.trwld` satchel: g90 v18 union offset **23** (§5a — moved from 20). NO sidecar token. |
| **4829 / 4830 / 4910** Pet Licenses | `:45352-45365` / `:45438-45445` / `NPC.cs:80894-80918` | a6, g87 (`dfbcaf94b`) | Both port the mid-swing `ItemTimeIsZero && (!bought \|\| AnyNPCs(npcType))` gate + the SP purchase arm (flag + `Misc.License*Used` World line + consume). **Exchange-fail adjudication:** the C# exchange legs (`:80910-80917`) are unreachable on BOTH ports today (no TownCat/Dog/Bunny kind, AnyNPCs always false → a bought license refuses the gate). BUT a6 keeps a LIVE `AnyNPCs` scan (`n.hp > 0 && kind.net_id() == npc_type`) and, if a resident pet ever lands, its gate passes → `ApplyItemTime` + consume with NO chat — the C# ALWAYS posts PetExchangeSuccess (`RerollVariation` true) or **PetExchangeFail** (`:80916`). g87 hardcodes `any_npcs = false` (documented deviation), so its bought-license ALWAYS refuses — the C# verdict with no resident pet, no future silent-consume leg. a6's shape is closer to the C# scan but incomplete on the exchange-fail chat; g87's is the safer closed world. **Verdict: a6 keeps the arm** (the live scan is the faithful shape); the merge must add the exchange legs (RollVariation + the Success/Fail chats) when the town pets land — flagged §10.3. | **a6** (`use_pet_license`) + the §10.3 exchange-fail merge debt. The town-arrival gates `Main.cs:65900-65911` are the deferred consumer. g87's arm DROP. | `.wld` bought-pet bytes tail **2021/2022/2023** gate 217: **f45** supersedes a6 **and g87** (§4/§10.5). g87's `.trwld` 3-byte v18 pet tail rides the union (§5a). NO sidecar (world globals). |
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
- **L2 `.trplayer`** (`persist.rs`, port text sidecar): **g88 v10** is the
  base — the strict superset writer/reader (research/103 v10). a6 contributes
  NO sidecar writer. The unlock line 8 carries tokens 1-6 (§6); life pair
  line 9, mana pair line 10, bread line 11, loadout block lines 12-15. The
  live `Player.used_*` fields restore at world load (`app/mod.rs`); the
  pearl mirrors into `LuckState` via `update_luck` (g90 shape, a6 now matches).
- **L3 world** (`.wld` flags + `.trwld`): **f45** is the `.wld` flags base
  (the complete block); a6/g89/g90/g87's five contested bytes dedup onto it
  (§4). The `.trwld` v18 tail is the g70×g71×g90×g87×f45 **68-byte union**
  (92 §1 + 135 §4 + f45's 44-byte identity block + g87's 3-byte pet block)
  — the FIVE-way collision this doc surfaces (§5/§5a). World globals NEVER
  ride L2 (122 §5).

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

## 5. The `.trwld` v18 union collision (FIVE-way — f45 identity + g87 pets)

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

### 5a. THE EXACT BYTE-LAYOUT TABLE (the g71c/f44e/f45 merger input)

All offsets are from the START of the v18 block, i.e. the byte immediately
after the v17 `downedFishron` u8. Every multi-byte field is little-endian
(the port's `binio` convention, matching `BinaryWriter`). **v18 UNION tail =
68 bytes** (16 forest + 3 surface + 3 pets + 2 books + 44 identity):

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

**VERSION stays 18.** Every truncation constant moves to **+68** (was +65
in this doc's first pass, before g87's 3 bytes surfaced). The exact sites,
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

Every lane's round-trip witness re-pins 16/3/3/2/44 → 68. **Merge action:**
the g71c union extension (135 §4's owner) folds g87's pets at offsets 19-21
and f45's identity block at offsets 24-67; g90's pair MOVES from 19/20 to
22/23 (its writer/reader + witnesses re-pin); f45's `stamp_created_identity`
+ WorldId roll stay its scope; g87's `World.bought_*` fields dedup onto
f45/a6's identical fields (same names — a clean field-level dedup). This
doc surfaces BOTH the f45 and the g87 collisions — flag to the
g71c/f44e/f45/g87 merger.

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
arcaneCrystal galaxyPearl` (tokens 1-6), token 7 DD2 RESERVED→v11. This is
g88 v10's actual writer/reader; the merger keeps it verbatim. 135 §3's
mapping table column "UNION token (recommended)" must be corrected to
crystal=4/arcane=5/pearl=6 at the fold (its "103 §2a wins" recommendation is
overridden by the pushed writer).

**The six-line v7 recognition rule (135 §2) is NOT yet in g88 v10 —
RE-CONFIRMED at the FINAL probe.** `origin/task/W4-g88-life-fruit-use` tip
is STILL `c8985b534` (fold 6; no g88f commit landed after it — `git log`
census). The v7 walk (`persist.rs:409-418` at that tip) still collapses
g90b's six 1-token lines into the aegis/bread slots: the `match toks.len()`
`1 =>` arm reads the FIRST one-token line as `used_aegis_fruit`
(`flag_seen` latch) and EVERY later one-token line as `ate_artisan_bread`
(the last wins). No `post.len() == 6 && all(t.len() == 1)` guard exists
anywhere in the file. The witness
`g90bs_bread_plus_flags_v7_loads_with_the_bounded_misread` (`:1064`) still
ACCEPTS the misread (bread→aegis, last line→bread, crystal/arcane/pearl/
gummy/ambrosia lost — its own assert `!s.used_aegis_crystal, "no positional
v7 crystal read"`). **The flag STANDS.** 135 §2 demands the unambiguous
rule:

```
if version == "7" && post.len() == 6 && post.iter().all(|t| t.len() == 1) {
    bread=post[0]; crystal=post[1]; arcane=post[2]; pearl=post[3];
    gummy=post[4]; ambrosia=post[5];   // positional, .plr write order
}
```

**Merge action:** g88's v7 legacy arm MUST gain 135 §2's six-line rule (the
bounded-misread witness is replaced by a positional-read witness). Until
then, a g90b lane-7 v7 file loads lossily on g88 v10. This is the one
remaining `.trplayer` reader gap; flag to the g88 merger.

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
  `g90bs_bread_plus_flags_v7_loads_with_the_bounded_misread` witness is
  REPLACED by 135 §2's positional-read witness (§6 merge action).
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
2. **L2 `.trplayer`** — base **g88 v10** (`c8985b534`, the superset). Add
   135 §2's six-line v7 recognition rule (§6 merge action). Tokens 4/5/6 are
   ALREADY live in the written crystal/arcane/pearl order (§6) — no
   activation needed, just keep g88's writer/reader. Dedup g90's
   `SavedPlayer` flag fields + App restores onto the superset's. a6
   contributes nothing (no writer).
3. **L3 `.wld` flags** — base **f45** (`6352d384`, the complete block). Dedup
   a6/g89e/g90/g87's book2 (2036) / satchel (2037) / bought-pets (2021-23)
   flips onto f45's fields (§4). a6's use arms write the same World fields.
4. **L3 `.trwld`** — the **68-byte v18 union** (§5a's exact table): g70
   forest 16 + g71 surface 3 + g87 pets 3 + g90 book2/satchel 2 + f45
   identity 44. Move every truncation constant to +68. VERSION stays 18,
   forward-only. The g71c union owner folds g87's pets at offsets 19-21
   and f45's identity block at offsets 24-67; g90's pair moves 19/20→22/23.
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
   (19→68 union), and INDEX rows 103/122/132/135/138.

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
`dh` reader arm, both round-trip witnesses) DROP at merge; the g88 v10
superset stays the sole L2 writer.** The App restore
(`p.extra_accessory = saved.extra_accessory` etc.) dedups onto the f44
hydration at the save funnel (§2 L1). If the merger ever wants a live-
session carrier before f44 lands, the fields ride g88's NEXT version
(v11+) as new tokens — never a parallel v7.

### 10.5 g87's `.wld`/`.trwld` halves — dedup onto f45 / ride the union

- **`.wld` bought-pets:** g87 flips `flags.rs`'s three `w.boolean(false)`
  → `w.boolean(world.bought_*)` at gate 217 and reads them back — the
  SAME bytes/offsets/gates as a6 and f45 (§4's table now five-lane).
  **KEEP f45**; g87's flips dedup (same field names — g87's
  `World.bought_cat/dog/bunny` declarations dedup onto a6/f45's identical
  fields, ctor-false). g87's `wld/mod.rs` tail pins `tail[2021-23]` are
  byte-identical to a6/f45's — the witnesses merge cleanly.
- **`.trwld` pets:** g87's 3-byte v18 tail is the FIFTH union member
  (§5/§5a): offsets 19-21, ahead of g90's book pair (vanilla write order
  `:1392-1394` < `:1407-1408`). Its round-trip witness re-pins 3→68.

### 10.6 g88 recognition-rule re-check (the Lead's item 2) — **RE-FLAGGED**

`origin/task/W4-g88-life-fruit-use` tip is STILL `c8985b534` at this probe
(no g88f commit; `git log -3` census: fold 6 / fold-5 docs / fold 5). The
135 §2 six-line v7 recognition rule is STILL MISSING — §6's flag stands
verbatim, now with the reader-line receipts (`persist.rs:409-418`'s
`flag_seen` latch walk, no `post.len() == 6` guard, the bounded-misread
witness still asserting the lossy read). **Merge action unchanged:** g88's
v7 legacy arm gains the positional six-line rule; the bounded-misread
witness is replaced.

### 10.7 a6's five fidelity fixes — LANDED, witnesses present (the Lead's item 4)

Re-verified on this branch (`d7de8bd53` + this pass), each fix's code AND
witness present:

1. **Row-targeted dispatch:** `AchievementManager::complete_condition`
   (`achievements.rs:445`) + the four call sites (`powerups.rs:177`/`:179`/
   `:362`/`:516`/`:518` — GET_A_LIFE/TOPPED_OFF/GAIN_TORCH_GODS_FAVOR/
   STAR_POWER). Witness `complete_condition_targets_one_row_not_the_flag_family`
   (`achievements.rs:634`). ✓
2. **Pearl Player-field ownership:** `Player.used_galaxy_pearl`
   (`player.rs:2171`, ctor `:2978` false), the 5340 arm sets the Player
   field (`powerups.rs:474`), `update_luck` mirrors
   (`player.rs:10651`). Witness assertions `powerups.rs:1267-1269` (the
   Player field owner + the mirror seeds the state). ✓
3. **Aegis-crystal regen slot:** `RegenInputs.aegis_crystal`
   (`regen.rs:117`), applied BETWEEN `+= 1.0` (`:190`) and `+= extra_time`
   (`:197`) at `:194-196`. Witness
   `the_aegis_crystal_adds_between_the_base_and_the_crimson`
   (`regen.rs:688`). ✓
4. **Booster order:** fruit (`player.rs:6916`) → worm (`:6919`) → ambrosia
   (`:6922`) — the C# `UpdatePermanentBoosters` order. ✓
5. **TRAINING_DAY hook:** `notify_progression_net(40)` when both world book
   flags stand (`world_sim/player.rs:1621-1626`). Witness
   `both_combat_books_fire_training_day` (`powerups.rs:1277`). ✓

No cargo/tests/rustfmt/graphify per the lane constraint — the witnesses
are WRITTEN, NOT RUN; presence verified by direct file reads.

### 10.8 FINAL verdict summary

The §1 table's keep-owns are UNCHANGED by g87's push: **a6 keeps
3335/5289/4829-30-4910** (g87's three arms DROP — §10.1 start-refusal gap,
§10.2 spawn-walk gaps, §10.3 the live-scan shape + the exchange debt),
**g87's sidecar `dh` line DROPS** (§10.4), **g87's `.wld` flips dedup onto
f45** (§10.5), **g87's `.trwld` pets ride the 68-byte union at offsets
19-21** (§5a). The `.trplayer` reader gap (§6/§10.6) and the five-way v18
collision (§5) remain the two open merge actions, both flagged to their
owners. This doc is FINAL for the a6 lane.
