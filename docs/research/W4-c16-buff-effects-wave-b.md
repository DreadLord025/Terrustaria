# W4-c16 — buff-effects wave B (the first missing block)

Branch `task/W4-c16-buff-effects-wave-b`, base `a725acb0`. Code-only worker
(no cargo/tests run; witnesses written, not executed). Sibling lanes: c17
(second block), c18 (third block).

## Range claimed

`Player.cs:9935-9947` (the two potion-range stragglers) **+**
`Player.cs:10328-10461` (the ManaHeat / lunar-orb tail). This is the FIRST
contiguous region of `Player.UpdateBuffs`' chain (`:9855-12069`) whose arms
have **no live effect anywhere in the port**.

## Census method

Extracted every `buffType[j] == N` head and every `>= A && <= B` band in
`Player.cs:9855-12069` (256 distinct ids incl. the beetle/solar/nebula bands),
then diffed against the union of the LIVE effect paths:

* `player.rs` inline buff match (`:6428-6728`, wave A + the partial wave-B
  rows) and the `accessory_life_regen` fold (`:6940-6946`, buffs 2/165);
* `world_sim/player.rs::grant_ammo_mastery` (16/93/112) and
  `Player::update_summon_buff_arms` (110/150/348 + the 29 `SUMMON_BUFF_ARMS`);
* `buff_effects_accessory.rs::update_accessory_buff_arms` (WIRED: the pet
  family, 37/38/146/148, the imbues 71/73-79);
* the DoT derivation `Player::derive_dot_inputs` / `holding()` (20/70/397/24/
  323/44/324/39/67/68/144/38/30/48/137), `refresh_wet_state` (103), `cced`
  (156);
* the light feed `entity_lights::shine_light` / `player_lights` (Shine 11,
  Inferno 116, Electrified 144, Ice Barrier 62 — RNG-free AddLight halves).

The `buff_effects.rs` (wave B debuff table) and `buff_effects_misc.rs` (wave D
mount/pet/misc table) accumulators are PURE and **not wired** — their ids are
claimed by their own waves and are out of scope here (re-porting them would
duplicate B/D). The wired accessory table (C) is likewise out of scope.

## The 21 missing ids (this block)

| id | name | C# arm | effect |
|----|------|--------|--------|
| 192 | SugarRush | `:9935-9939` | `pickSpeed -= 0.2f; moveSpeed += 0.2f` |
| 321 | BrainOfConfusionBuff | `:9940-9947` | `meleeCrit/rangedCrit/magicCrit += 10; minionDamage += 0.1f` |
| 396 | ManaHeat | `:10328-10341` | `manaHeat += latestBonus * (buffTime / latestDuration)`; duration 0 → 1 |
| 95/96/97 | BeetleEndurance 1-3 | `:10342-10376` | pin time 5; orb reconcile; `beetleOrbs = n`; gate `beetleDefense` → `beetleBuff` |
| 170/171/172 | SolarShield 1-3 | `:10377-10407` | pin time 5; shield reconcile; `solarShields = n`; gate `setSolar` |
| 98/99/100 | BeetleMight 1-3 | `:10408-10443` | orb reconcile; `meleeDamage/meleeSpeed += 0.1f * orbs`; gate `beetleOffense` → `beetleBuff` |
| 176/177/178 | NebulaUpMana 1-3 | `:10444-10447` | `UpdateBuffs_NebulaBuffs(ref nebulaLevelMana, 176, j)` |
| 173/174/175 | NebulaUpLife 1-3 | `:10448-10452` | nebula level + `lifeRegen += 6 * nebulaLevelLife` |
| 179/180/181 | NebulaUpDmg 1-3 | `:10453-10461` | nebula level + `0.15f * level` to all four class damages |

`UpdateBuffs_NebulaBuffs` (`Player.cs:12070-12079`): `level = 1 + buffType[b] -
base`; if `buffTime[b] == 2 && level > 1` → `level--; buffType[b]--;
buffTime[b] = 480` (the tier-downgrade at expiry).

## Why "~40" overcounts

A naive grep of `buff_effects*.rs` alone (ignoring the player.rs wave-A match,
the summon table, the DoT/regen folds and the light feed) reports ~69 absent
ids; the first ~40 of those would re-include the wave-A potions (6-18, 30) and
the summon keep-alives (49/64/83/…), which ARE live. The honest live census
leaves only these 21 truly-missing ids, and they form a single block — there is
no second/third truly-missing block in `:9855-12069` (the remainder is claimed
by the B/C/D waves). c17/c18 should therefore target the unwired B/D
accumulator arms (wiring them onto the live fields) rather than new ids.

## RNG census

NONE of the 21 arms draws `Main.rand` (verified against each C# body: SugarRush,
BrainOfConfusion, ManaHeat, the beetle/solar orb reconciles and the nebula
level updates are all arithmetic + slot bookkeeping). The block is
stream-neutral: porting it cannot move the `UpdatePlayers` RNG stream. The
beetle/solar `DelBuff` reconciles and the nebula `buffType--` downgrade are
slot mutations, not draws.

## Seam

Pure accumulator `buff_effects_orbs.rs::apply_orb_effect_arm` (the B/D pattern):
folds one slot's writes into an `OrbEffects` accumulator; the cross-slot
`DelBuff` purges and the nebula downgrade ride as recorded requests for the
wiring step (the merge step copies the accumulator onto the live `Player`
fields and applies the slot requests). Nothing here touches `BuffSlots` timers
beyond the recorded `buff_time_set` / `buff_type_decrement` requests.

The beetle/solar reconcile reads the CURRENT `beetleOrbs`/`solarShields` and the
`beetleDefense`/`beetleOffense`/`setSolar` set-bonus flags (granted by
`ApplySetBonus_BeetleDefense`/`_BeetleDamage`/`_Solar`, `:16499-16535`/`:16366`,
not yet ported) — they ride in `OrbEffectContext`. With the flags false (the
current port state) the arms zero the orbs/shields and `DelBuff` the slot,
exactly as the C# does without the set bonus; the stat adds still land first
(`:10431-10432`), matching the source order. The ManaHeat arm reads
`latestManaPotionDamageBonus`/`latestManaPotionDuration` (written by
`TryApplyManaHeat`, `Player.cs:52006-52014`, already ported as
`Player::try_apply_mana_heat`).

## Witnesses

One `#[test]` per id (21), each naming its C# lines, plus the orb-reconcile
branches (higher-tier self-delete vs lower-tier purge band), the nebula
tier-downgrade at `buffTime == 2`, and the ManaHeat duration-0 guard. Written,
**not run** (code-only mode).