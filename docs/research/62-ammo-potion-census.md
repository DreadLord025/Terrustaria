# 62 — Ammo & Potion census (S1-P-i03, code-only)

Audit base: `aa147ce0` (origin/main). Worktree `TR-i03`, branch
`task/S1-P-i03-ammo-potion-census`. Worker W3-i03. **Code-only mode: no
cargo/rustc/tests invoked** — every claim is file-read evidence (C#
`decompiled/Terraria/Player.cs` / `Item.cs` / `ID/*.cs` vs the Rust tree at the
base commit). Reads were kept to ≤240-line windows.

Scope: (a) the **ammo system** — `Player.PickAmmo`, the pick/consumption order,
the ammo categories (arrow/bullet/rocket/dart/special) and the not-consume rolls;
(b) **potions** — potion use, Potion Sickness, buff durations and the healing /
mana cooldowns. Each row carries a C# line cite, a port status, and a fix slice.

Status legend: **DONE** = 1:1 port present · **PARTIAL** = some halves missing ·
**MISSING** = no port counterpart · **DIVERGENT** = ported but behaviour differs ·
**PARKED** = deliberately deferred (documented).

> **ID note (load-bearing):** in this decompilation **Potion Sickness = buff 21**
> (`BuffID.cs:213`) and **Mana Sickness = buff 94** (`BuffID.cs:359`). The commonly
> quoted 88/89 are *not* the sickness ids here. Every gate/duration below keys on
> 21 / 94.

---

## 0. Verdict table (top findings first)

| # | Target | Verdict | Severity |
| --- | --- | --- | --- |
| P1 | Live heal arm ignores Potion Sickness **and** has no use-cadence gate (`editing.rs:540-565`) | **DIVERGENT** — heals every held tick while sick, drains the stack | **CRITICAL** |
| P2 | Per-item sickness durations missing (Restoration 2700 / Eggnog 2400 / Mushroom 1800 / Strange Brew random) — all hardcoded 3600 | **MISSING** | HIGH |
| P3 | Mana potions (110/189/500/2209) have **no use path**; Mana Sickness (94) never applied | **MISSING** | HIGH |
| P4 | Buff potions (Ironskin/Swiftness/Regen/Archery/…) grant **nothing** on use | **MISSING** | HIGH |
| P5 | QuickHeal / QuickMana / QuickBuff hotkeys bound but **no handler** | **MISSING** | HIGH |
| P6 | `DebugOptions.ManaV2 = true` default (slow-magic at 0 mana, ×2.5) unported; port hard-blocks low-mana use | **MISSING** | HIGH |
| A1 | Nanite bullet (proj 285) ×1.25 applied **twice** to the ammo half | **DIVERGENT** | HIGH |
| A2 | Ale Throwing Glove (3821) 69% not-consume arm dropped as "DEAD" — but `useAmmo = 353` is live | **DIVERGENT** | HIGH |
| P7 | Philosopher's Stone / Charm of Myths (`pStone`) ×0.75 sickness reduction unported | **MISSING** | MED |
| A3 | `HasAmmo` use-start refusal missing — port plays a dry swing with no ammo | **DIVERGENT** | MED |
| A4 | Consumable-weapon ammo-cost save rolls (`Player.cs:43283-43301`) skipped on the generic shoot consume | **DIVERGENT** | MED |
| A5 | Ground pickup never routes ammo into the ammo slots (`FillAmmo`) | **MISSING** | MED |
| A6 | Ammo Cycler +8% ranged damage (`accAmmoCyclerDamage`) and Sharp Barb +8 ranged crit halves missing | **PARTIAL** | MED |
| P8 | Heal lands on press, not at `ItemTimeIsZero` mid-animation; no `HealEffect` text | **DIVERGENT** | LOW |
| A7 | Snowball Cannon remix-world 50% arm | **PARKED** (documented) | — |

---

## A. Ammo system

### A.0 The C# shape (reference)

`ItemCheck_Shoot` (`Player.cs:47056-47082`) seeds `projToShoot = sItem.shoot`,
`speed = sItem.shootSpeed`, `Damage = weaponDamage`, `KnockBack = sItem.knockBack`,
then calls `PickAmmo` only `if (sItem.useAmmo > 0)` (`:47069-47072`), passing
`dontConsume = ItemID.Sets.gunProj[sItem.type]`. For the four `gunProj` items the
whole fuse is discarded afterwards (`:47077-47082`).

`PickAmmo` (`Player.cs:53458-53683`) does, in order:
1. pick the ammo item (`PickAmmo_PickAmmoItem`, `:53727-53749`);
2. resolve `projToShoot` (pair table → rocket/solution additive → ammo `shoot` →
   per-weapon overrides, `:53471-53541`);
3. fuse speed/damage/knockback (`:53542-53568`);
4. advance the ammo cycler (`:53569-53577`);
5. run the not-consume flag chain (`:53578-53674`);
6. drain `item.stack--` / `TurnToAir` iff `!flag && item.consumable` (`:53675-53682`).

### A.1 Pick order — **DONE**

| C# | Port |
| --- | --- |
| `AmmoSlotOrder_Default` = coins 50-53 → ammo 54-57 → grid 0-53 (`Player.cs:57141-57144`) | `ammo.rs:118-142` `pick_ordered` (coins → ammo → grid) |
| `AmmoSlotOrder_AmmoOnly` = 54-57, `CoinsOnly` = 50-53 (`:57143-57144`) | `ammo.rs:231-242` (`AmmoSlots` mode) |
| match predicate `item.ammo == sItem.useAmmo && stack > 0` (`:53696`, `:53715`) | `ammo.rs:93-95` `is_ammo_match` |
| `PickAmmo_IterateRange` cycling `offset % count` (`:53689-53724`) | `ammo.rs:160-195` |

The port splits the C#'s single 59-slot array into `App.inventory` (50) +
`equip.coins` (4) + `equip.ammo` (4) (`inv.rs:14-29`); the walk order is preserved.
The grid is 50 not 54 cells, but the coin/ammo/grid *grouping* — the part the pick
order reads — is identical. No action.

### A.2 Projectile resolution + stat fuse — **DONE** (one defect, A1)

`ammo.rs:577-700` `resolve_projectile` ports the whole chain
(`Player.cs:53458-53568`): the 60-cell `SpecificLauncherAmmoProjectileMatches`
table (`AmmoID.cs:22-114` → `ammo.rs:314-380`), the rocket/solution additive arms,
the per-weapon overrides (3019/3052/4953/4381/4058/sand-42/Bees-Knees-2888/Molten),
and the speed/damage/knockback fuse. The two formula arms `:53475-53482` (1946 /
3930) are correctly identified as dead (the table covers both launchers for all
twelve rocket kinds) and omitted (`ammo.rs:24-28`).

- Magic Quiver ×1.1 speed + knockback-on-weapon-only (`:53543-53547` before
  `:53568`): **DONE** (`ammo.rs:669-674`, `editing.rs:1898-1907`).
- Archery ×1.2 cap-20 (`:53560-53567`): **DONE** (`ammo.rs:686-692`).
- Sharp Barb +1 ammo damage (`:53548-53551`): **DONE** (`ammo.rs:675-678`).
- Molten Quiver wooden→flaming +2 (`:53537-53541`): **DONE** (`ammo.rs:663-666`).

**A1 — Nanite double-scale (DIVERGENT, HIGH).** C# applies ×1.25 **once** to the
*fused* `Damage` (`Player.cs:53556-53559`). The port applies it to the ammo half
inside the resolver (`ammo.rs:680-685`, `damage = (num*1.25) as i32`) **and again**
to the fused total in the caller (`editing.rs:1892-1897`, `damage *= 1.25`). The
ammo contribution therefore lands at ≈×1.5625 instead of ×1.25, and the truncation
points differ (C# truncates the fused int once; the port truncates the ammo half,
re-multiplies, then truncates at spawn `editing.rs:1934`).
**Fix slice A1:** drop the ×1.25 from `resolve_projectile` (return the unscaled
ammo contribution) and keep the single caller-side scale on the fused value, OR
keep the resolver scale and remove the caller's — exactly one site, matching
`:53556-53559`. Add a Nanite-bullet damage witness (weapon + ammo → expected int).

### A.3 Not-consume chain — **DONE** except A2 / A7

`ammo.rs:732-843` `not_consume` ports the flat, order-load-bearing flag chain
(`Player.cs:53578-53674`) with short-circuit draws (a failed gate spends no RNG),
the `dontConsume` seed, and the Acorn Slingshot's final `flag = false`
(`:53671-53674` → `ammo.rs:839-841`). All item-keyed and flag-gated arms are present
in source order; the module doc (`ammo.rs:40-74`) pins the determinism contract.

**A2 — Ale Throwing Glove 69% arm dropped (DIVERGENT, HIGH).** `ammo.rs:60-63` and
`:807-809` omit `sItem.type == 3821 && Main.rand.Next(100) < 69`
(`Player.cs:53639-53641`) as "DEAD — `useAmmo == 0`". **That premise is false:**
`Item.cs:33927` sets `this.useAmmo = 353` for 3821, and the port's own registry
carries it (`itemdata.rs:3987`, `use_ammo: 353`; Ale 353 `ammo: 353`,
`itemdata.rs:519`). So `PickAmmo` *does* run for the glove and the 69% save is live;
the port currently consumes an Ale on **every** throw.
**Fix slice A2:** add the `shooter == 3821 && rng.next_max(100) < 69` arm at its
source position (after the Fairy-Queen arm, before the Huntress arm) in
`not_consume`; correct the module doc; add a rate witness (~69/100 saves).

**A7 — Snowball Cannon remix arm (PARKED).** `Main.remixWorld && sItem.type == 1319`
50% (`Player.cs:53659-53661`) is omitted because the port has no celebration-seed
stand-in (`ammo.rs:64-68`, `:826-827`). Documented; revisit with the seed system.

### A.4 Consumption site & cadence — **DONE** (latent note)

The drain lives inside the pick (`ammo.rs:276-290` `drain`, called at
`editing.rs:1837`), matching C# `:53675-53682` (drain before the spawn). The port's
`drain` does **not** re-check `item.consumable` (C# `:53675`); every ammo kind the
registry carries is `consumable = true`, so this is latent, not reachable today.
Note for any future non-consumable ammo.

### A.5 Ammo categories & damage dispatch — **DONE**

`AmmoID` constants (`ID/AmmoID.cs:159-177`): Gel 23, Arrow 40, Coin 71, FallenStar
75, Bullet 97, Sand 169, Dart 283, Rocket 771, Solution 780, Flare 931, Snowball
949, StyngerBolt 1261, CandyCorn 1783, JackOLantern 1785, Stake 1836, NailFriendly
3108, Acorn 27. Port: `ammo.rs:298-304` + `is_arrow_ammo`/`is_bullet_ammo`/
`is_specialist_ammo` (`ammo.rs:400-419`) match `AmmoID.Sets.IsArrow/IsBullet/
IsSpecialist` (`AmmoID.cs:115-117`). `IS_RANGED_SPECIALIST_WEAPONS`
(`ammo.rs:424`) matches `ItemID.cs:1472`.

`GetWeaponDamageMultiplier`'s ranged dispatch (arrow→bow, bullet→gun,
specialist→rocket, else ranged; `Player.cs:53396-53408`) is ported as
`player.rs:4089-4118` `weapon_damage_multiplier_with_ammo` (S1-C-4), with the
truncation moved before the fuse (`editing.rs:1864-1869`). **DONE.**

Registry coverage (count of items per `use_ammo`): arrow 40, bullet 24, rocket 7,
dart 4, solution 2, gel 2, star 2, and one each for sand/coin/flare/snowball/
stynger/candycorn/jack/stake/nail/acorn/ale — **all 18 categories present**.

### A.6 gunProj four — **DONE**

`ItemID.Sets.gunProj = {3475, 3540, 3854, 3930}` (`ItemID.cs:1327`). Port
`editing.rs:82-135` `GUN_PROG` carries the id-keyed vanilla rows; `dontConsume`
seeds the roll chain and the fuse is discarded (`editing.rs:1745-1748`,
`:1873-1884`), matching `Player.cs:47077-47082`. **DONE.**

### A.7 Ammo cycling accessory — **DONE** (damage half missing, A6)

Modes (`Player.cs:14529-14542`), `Update_AmmoCycler` (`:28420-28426`), and the
per-pick offset/cooldown write (`:53569-53577`) are ported: `ammo.rs:494-540`
(`AmmoCycling`/`tick`), `world_sim/player.rs:127-131` (grant), `editing.rs:1719-1807`
(per-pick write, Random draw on the `UpdatePlayers` stream), `world_sim/player.rs:831`
(per-tick tick). The Clentaminator forced-index-0 (`:53706-53709`) is applied at
`editing.rs:1730-1732`. **DONE.**

**A6 (part 1) — `accAmmoCyclerDamage` +8% ranged damage (MISSING).**
`Player.cs:12958-12961` adds `rangedDamage += 0.08f` when the cycler accessory is
equipped (`:14530`/`:14534`/`:14538`/`:14542` set `accAmmoCyclerDamage = true`).
The port grants the cycling *mode* but has no `acc_ammo_cycler_damage` flag and no
+0.08 ranged row. **Fix slice A6a:** add the flag to the cycler grant
(`world_sim/player.rs:127-131`) and the `ranged_damage += 0.08` fold in the equip
stat pass.

### A.8 Ammo-cost armor/buff flags — **DONE**

Huntress 1/10 (`:13179`/`:13634`), Chlorophyte 1/5 (`:13649`), Shroomite/Huntress-alt
1/5 (`:13715`/`:13222`), Vortex 1/4 (`:13908`), Ammo Box buff 93 1/5 (`:10080`),
Ammo Reservation buff 112 1/5 (`:10195`), Magic Quiver 1/5 (`:53595`). Port:
`world_sim/player.rs:57-104` (buff + armor grants), `armor_sets.rs:386-387` (set
bonuses), `ammo.rs:761-825` (roll arms). **DONE.**

**A6 (part 2) — Sharp Barb +8 ranged crit (MISSING).** `Player.cs:12966-12969`
adds `rangedCrit += 8` for `accSharpBarb`. The port grants `acc_sharp_barb`
(`world_sim/player.rs:124`) and the +1 ammo-damage half (`ammo.rs:675`) but no
+8 ranged-crit row was found. **Fix slice A6b:** add the crit fold beside the
existing Sharp Barb grant.

### A.9 `HasAmmo` use-start refusal — **DIVERGENT (A3, MED)**

C# `ItemCheck_CheckCanUse_Inner` calls `HasAmmo(sItem, flag)` (`Player.cs:52318`),
which scans `inventory[0..58]` for a matching stack (`:53431-53446`) and refuses the
**use start** when none exists — no animation plays. The port's shooter use-start
(`editing.rs:615-638`) does not check ammo presence; it arms the swing, then the
per-shot pick (`editing.rs:1734-1742`) returns dry. Net effect: the port plays a dry
swing animation with no ammo where vanilla refuses the use outright.
**Fix slice A3:** gate the shooter use-start on a `has_ammo` scan over
coins/ammo/grid (reuse `ammo::pick`'s predicate) before arming the swing.

### A.10 Consumable-weapon own-stack consumption — **DIVERGENT (A4, MED)**

C# `ItemCheck` consumes a consumable **weapon's own** stack at
`itemTime == itemTimeMax` and, for `item.ranged`, rolls the four ammo-cost saves
first (`Player.cs:43280-43326`, rolls at `:43283-43301`). The port's generic shoot
path consumes `def.consumable` unconditionally (`editing.rs:1067-1070`) with no
save rolls — so e.g. the Snowball (949, `ranged`, `consumable`, `use_ammo = 0`)
ignores the Huntress/Chloro/Shroomite/Vortex saves on its own stack.
**Fix slice A4:** route the generic consumable-weapon consume through the same four
flag rolls (a small shared helper with `not_consume`'s flag arms) before decrementing.

### A.11 Ground pickup → ammo slots (`FillAmmo`) — **MISSING (A5, MED)**

C# `GetItem` routes `newItem.ammo > 0 && !notAmmo` into the ammo slots first
(`Player.cs:39175-39184`, `FillAmmo` `:39264-39343`, `CanGoIntoAmmoOnPickup`
`:39237-39262`). The port's pickup (`world_sim/items.rs:249-265` →
`item.rs:133` `slot_for_pickup`) walks only the grid; `chest.rs:1183-1185` explicitly
records "the `ammo > 0` FillAmmo arm … have no port counterparts yet". Manual
chest→ammo merge exists (`chest.rs:1198-1201`) but ground pickup never fills ammo
slots. Note `notAmmo` (sand/coin/ale/star) is also absent from `ItemDef`.
**Fix slice A5:** add a `FillAmmo` pre-pass to `slot_for_pickup`/`update_items`
(ammo slots first for `ammo > 0 && !notAmmo`), and add a `not_ammo` field to the
generated item table.

---

## B. Potions

### B.0 The C# shape (reference)

- Use start: `item.potion && flag2` → `ApplyPotionDelay(item)`
  (`Player.cs:42756-42759`).
- Mid-animation at `ItemTimeIsZero && itemAnimation > 0`: `healLife/healMana > 0` →
  `ApplyLifeAndOrMana(item)` (`:42867-42875`); `buffType > 0` (≠90, ≠27) →
  `AddBuff(buffType, buffTime)` (`:42876-42883`).
- `ApplyPotionDelay` (`:51853-51886`) sets `potionDelay` and `AddBuff(21, …)`.
- `ApplyLifeAndOrMana` (`:51888-51952`) clamps life/mana, `HealEffect`, and for
  `healMana > 0` adds buff 94 (`manaSickTime = 300`, `:57101`).
- Sickness gate: `sItem.potion && potionDelay > 0` → cannot use
  (`:52217-52220`); `potionDelay` is synced from buff 21's `buffTime`
  (`:11719-11722`) and decremented per tick (`:24718-24721`).

### B.1 Potion Sickness durations — **MISSING (P2, HIGH)**

`ApplyPotionDelay` (`Player.cs:51853-51886`) is **per-item**:

| Item | C# delay | C# cite |
| --- | --- | --- |
| 3001 Strange Brew | `rand.Next(2400, 4201)`, ×0.75 if `pStone` | `:51855-51863` |
| 227 Restoration / 126 Bottled Water | `restorationDelayTime` = 2700 | `:51864-51868`, `Item.cs:49969` |
| 1912 Eggnog | `eggnogDelayTime` = 2400 (+hunger reset) | `:51869-51874`, `Item.cs:49970` |
| 5 Mushroom | `mushroomDelayTime` = 1800 (+hunger reset) | `:51875-51880`, `Item.cs:49971` |
| all other potions | `potionDelayTime` = 3600 | `:51881-51885`, `Item.cs:49968` |

The port hardcodes `add_buff(21, 3600)` for **every** heal potion
(`editing.rs:552`, `:1385`). Restoration (the common early-game potion) gets 60 s
instead of 45 s; Eggnog/Mushroom/Strange Brew are all wrong.
**Fix slice P2:** port `ApplyPotionDelay` as a per-item function (the four named
delays + the Strange Brew random draw on the `UpdatePlayers` stream) and call it
from both heal arms; carry `Item.potionDelay/restorationDelay/eggnogDelay/
mushroomDelay` as constants (`Item.cs:49968-49971`).

### B.2 The live heal arm — **DIVERGENT (P1, CRITICAL)**

`editing.rs:540-565` (the first, pre-weapon heal arm) computes
`sick = find_buff_index(21).is_some()` (`:542-546`) **but never reads it** — it heals
and re-applies buff 21 unconditionally (`:548-553`). Worse, the enclosing block
(`:504`, `if app.mine_down && held.count > 0 && !app.inventory_open`) has **no
`attack_cd`/`reuse_delay`/press-edge gate**, so holding LMB with a heal potion heals
**every tick**, drains the whole stack in ~1 s, and keeps refreshing sickness to
3600. The second arm (`editing.rs:1372-1400`) *does* gate `if !sick` (`:1379`) but is
shadowed by the first (which returns early). C# refuses the use entirely while
`potionDelay > 0` (`:52217-52220`) and paces it by `useAnimation`.
**Fix slice P1 (do first):** in the live arm, (a) gate on `!sick` (or a real
`potionDelay` field), and (b) add the use-cadence gate (`attack_cd <= 0 &&
reuse_delay <= 0` + press edge) so one drink per `useAnimation`. Delete or merge the
duplicate second arm so there is a single heal path. Witness: holding LMB while sick
consumes nothing and heals nothing; one press = one drink.

### B.3 `potionDelay` field / buff-21 sync — **PARTIAL**

C# keeps `Player.potionDelay` (`:1650`), decrements it (`:24718-24721`), and
re-derives it from buff 21 each tick (`:11719-11722`). The port derives
`e.potion_delay = Some(buff_time)` in the buff-effect table
(`buff_effects_misc.rs:276`, `:624`) but **nothing consumes it** — the heal gate reads
buff presence directly. There is no `Player.potionDelay` field. This is why P1/P2
gate on the buff rather than the C# field.
**Fix slice P1/P2 (shared):** add a `potion_delay` field synced from buff 21 (or
consume the derived `e.potion_delay`), and gate potion use on it to match
`:52217-52220` exactly (including the `potion` flag — mana potions are `potion =
false`, see B.4).

### B.4 Mana potions & Mana Sickness — **MISSING (P3, HIGH)**

Mana potions 110/189/500/2209 carry `healMana` 50/100/200/400 and **`potion =
false`** (verified `Item.cs` case 189 has no `potion = true`; port
`itemdata.rs:276/355/665/2510` all `potion: false`). C# `ApplyLifeAndOrMana`
(`:51888-51952`) adds `statMana`, clamps, and for `healMana > 0` adds **buff 94**
(`manaSickTime = 300`, `:51938`, `:57101`); the buff-94 arm sets `manaSick` and
`manaSickReduction = 0.25 * buffTime / 300` (`:10318-10327`), which scales
`magicDamage *= 1 - reduction` (`:25900-25902`). Buff 94 stacks additively to a 600
cap (`BuffID.Sets.AddBuffTimeAdditivelyToCap`, ported `buffs.rs:147-153`).

The port has **no mana-potion use path** (`heal_mana` is read only by
itemdata/tooltip/sorting; no `add_buff(94, …)` outside `buffs.rs` tests) and **no
buff-94 effect arm**. Mana potions are unusable and Mana Sickness never applies.
**Fix slice P3:** add a `healMana > 0` use arm (mana clamp + `add_buff(94, 300)`),
and the buff-94 effect arm (`mana_sick_reduction = 0.25 * t / 300`) folded into
`magic_damage`. The additive-cap machinery already exists (`buffs.rs:147-153`).

### B.5 ManaV2 / slow-magic & Mana Flower — **MISSING (P6, HIGH)**

`DebugOptions.ManaV2 = true` and `ManaPotionDelay = false` are the **defaults**
(`Testing/DebugOptions.cs:85-86`). Consequences in 1.4.5.8:
- `CheckMana` at 0 mana does **not** block; it sets `slowMagicUse` and returns true
  (`Player.cs:52610-52616`), slowing the weapon ×2.5
  (`GetSlowMagicMultiplier`, `:47045-47048`; the animation arm `:4478-4482`,
  `GetSlowMagicUseTime` `:47019-47026`).
- A mana potion **undoes** the slow (`UndoSlowMagicUse`, `:51946-51965`).
- `manaPotionDelay` stays 0 (the 8 s arm `:51939-51943` is gated off), so QuickMana
  is limited only by `statMana == statManaMax2` (`:5600`).

The port's mana pay (`editing.rs:698-705`) **hard-blocks** the use when
`mana < cost` — the pre-ManaV2 behaviour. No `slow_magic_use`, no ×2.5, no
Mana-Flower auto-QuickMana (`CheckMana` `:52588-52609`; the port grants only the
555 `mana_cost -= 0.08` half, `player.rs:3951`).
**Fix slice P6:** port the ManaV2 branch of `CheckMana` (allow at 0 mana, set
`slow_magic_use`, ×2.5 use-time via `GetSlowMagicUseTime`), the undo on mana-potion
use, and the Mana Flower auto-drink. This is systemic for every magic weapon.

### B.6 Buff potions on use — **MISSING (P4, HIGH)**

C# grants `item.buffType` (≠90/27) at `ItemTimeIsZero` for **any** item
(`Player.cs:42876-42883`), plus a use-start grant for `mana > 0` items
(`:42760-42763`). Buff potions carry correct rows in the port
(`itemdata.rs`: 289 Regen buff 2/28800, 290 Swiftness 3/28800, 292 Ironskin
5/28800, 294 MagicPower 7/14400, 303 Archery 16/28800, 2344 AmmoReservation
112/28800), but the only use-path grant is inside the **projectile** branch
(`editing.rs:721-722`), which buff potions (`projectile: None`) never reach. The
food arm handles only 26/206/207 (`editing.rs:506`). So drinking an Ironskin /
Swiftness / Archery potion grants **nothing**.
**Fix slice P4:** add a generic `buffType > 0 && buffType != 90 && buffType != 27`
use arm (grant `buff_time`, consume the stack) reachable by non-projectile
consumables, mirroring `:42876-42883`. The buff *effect* arms for these ids already
exist (`buff_effects.rs`, `world_sim/player.rs:62-69`); only the *grant on use* is
missing.

### B.7 QuickHeal / QuickMana / QuickBuff — **MISSING (P5, HIGH)**

C# `QuickHeal` (`Player.cs:5501-5547`) + best-fit picker
(`QuickHeal_GetItemToUse`, `:5549-5596`, Restoration +30 bias `:5575-5582`, void-bag
scan), `QuickMana` (`:5598-5649`), `QuickBuff` (`:5766-5835`). The port binds the
actions and keys (`input_map.rs:50-55`, `:79-81`, H/J/B at `:461-463`) but **no
handler** consumes `Action::QuickHeal/QuickMana/QuickBuff` (grep across
`app/`, `world_sim/` returns nothing outside input_map/lang/settings). The hotkeys
are dead.
**Fix slice P5:** wire the three actions to `quick_heal`/`quick_mana`/`quick_buff`
functions porting the pickers + the shared use tail (sound, `ApplyPotionDelay`,
`ApplyLifeAndOrMana`, buff grant, consume). Depends on P1-P6 for the shared tail.

### B.8 Philosopher's Stone / Charm of Myths — **MISSING (P7, MED)**

`pStone` is set by 535 Philosopher's Stone (`Player.cs:15576`), 860 Charm of Myths
(`:15152`), 6185 (`:14617`); `PhilosopherStoneDurationMultiplier = 0.75f`
(`:57112`). It scales the three delay times per tick (`:25477-25486`) and the
Strange Brew roll (`:51858-51861`), and `AdjustRemainingPotionSickness`
(`:28939-28952`) rescales an *active* buff 21 when the accessory is added/removed
(`:25640-25642`). The port has no `p_stone` flag and no ×0.75 anywhere.
**Fix slice P7:** grant `p_stone` from 535/860/6185 in the equip pass, apply ×0.75
in `ApplyPotionDelay` (P2) and the per-tick delay recompute, and port
`AdjustRemainingPotionSickness` on the equip-change edge.

### B.9 AddBuff machinery & buff tick — **DONE**

`AddBuff` (`Player.cs:5263-5298`) + `DetermineBuffTimeToAdd` (`:5428-5449`) +
`TryUpdatingExistingBuffTime` (`:5402-5426`, additive cap / max) +
`ActuallyTryToAddTheBuff` (`:5325-5359`) are ported in `buffs.rs:192-234` (fed-state
clear, difficulty extension, resistCold, additive cap 94→600 / 383→43200). The
per-tick decrement + `TimeLeftDoesNotDecrease` (`Player.cs:9865-9874`) is
`buffs.rs:388-397` `update_buffs_tick`, called from `player.rs:4222`
(`update_with_grapple`). **DONE.** (The net/PvP branch `:5269-5275` is unreachable
single-player — recorded in ANALYSIS.)

### B.10 Heal timing & feedback — **DIVERGENT (P8, LOW)**

C# heals at `ItemTimeIsZero` mid-animation (`Player.cs:42856-42875`) and spawns
`HealEffect` green combat text (`:51929-51932`). The port heals on press
(`editing.rs:549`) with no `HealEffect` text. Strange Brew's random heal +
i-frame roll (`:51892-51917`) is also unported (fixed 70).
**Fix slice P8 (with P1/P2):** move the heal to the mid-animation `item_time == 0`
window, add the `HealEffect` text, and port the Strange Brew random heal/immunity
draw. Hunger reset (`TryToResetHungerToNeutral`, `:12081`) is out of scope (no
Don't-Starve hunger system) — PARKED.

---

## C. Fix-slice ordering (dependencies)

1. **P1** — gate the live heal arm on sickness + use cadence; collapse the duplicate
   arm. (Stops the per-tick drain; independent, do first.)
2. **P2 + P3** — per-item `ApplyPotionDelay` + a real `potionDelay` gate. (Depends P1.)
3. **P3** — mana-potion use arm + buff-94 Mana Sickness effect. (Independent of P1.)
4. **P4** — generic buff-potion grant-on-use. (Independent.)
5. **P6** — ManaV2 slow-magic + Mana Flower. (Systemic; pairs with P3.)
6. **P5** — QuickHeal/QuickMana/QuickBuff handlers. (Depends P1-P4/P6 shared tail.)
7. **P7** — `pStone` ×0.75. (Depends P2.)
8. **A1** — Nanite single ×1.25. (Independent, small.)
9. **A2** — Ale glove 69% arm + doc correction. (Independent, small.)
10. **A3 / A4 / A5 / A6** — HasAmmo refusal, consumable-weapon saves, FillAmmo
    pickup, cycler/barb stat halves. (Independent medium slices.)
11. **P8 / A7** — heal timing/text, remix arm. (Low / parked.)

Each slice ships with a C#-cited witness test (written, not run, per the code-only
mandate) and updates `docs/PORT_MAP.md` + `docs/ROADMAP.md` on landing.

## D. Cross-references

- `docs/research/04-player-combat.md` — item use / hurt / buff baseline.
- `docs/research/07-item-core.md` — item defaults / `DefaultToHealingPotion`
  (`Item.cs:48570-48582`).
- `docs/research/28-buffs-debuffs.md` — buff effect table (S1-E-7).
- ROADMAP rows: S1-C-4 (damage dispatch, done), S1-C-12 (equip modifiers),
  S1-E-7 (buff effect table). The potion *use/grant* gaps (P1-P8) and ammo gaps
  (A1-A6) are **new** seeds not yet catalogued.