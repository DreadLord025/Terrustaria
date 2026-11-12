# 04 — Player combat

> Wave 1 · researched 2026-11-12 · sources: decompiled (Terraria 1.4.5.8) + crates
> (Terraria-Rust) + ROADMAP rows M2-COMBAT-1/2, M3-ITEM-4, M3-EQUIP-2/5, M4-DAMAGE

## Scope
Player-side combat: `Player.Hurt` (the whole inbound damage pipeline),
`Player.KillMe`'s death-text half, the player-dealt damage/crit math
(`GetWeaponDamage`/`GetWeaponDamageMultiplier`/`GetWeaponCrit`, `DamageVar`,
`CalculateDamageNPCsTake`/`CalculateDamagePlayersTake`, `StrikeNPC_Inner`), and the
item-use cadence (`ItemCheck`, `ItemCheck_Shoot`, `PickAmmo`, `CheckMana`, the mining
tools). C# anchors: `decompiled/Terraria/Player.cs:38197-38567` (Hurt),
`:38807-38863` (KillMe), `:42595-45584` (ItemCheck family), `:53371-53429`
(weapon damage/crit), `:52576-52595` (CheckMana); `Main.cs:66118-66160`
(DamageVar / CalculateDamageNPCsTake / CalculateDamagePlayersTake);
`Projectile.cs:12938` (DamageVar inside `Projectile.Damage`). Rust side:
`crates/tr-app/src/player.rs` (10 275 lines), `crates/tr-app/src/world_sim/editing.rs`
(7 738), `crates/tr-app/src/ammo.rs` (2 180), `crates/tr-app/src/npc.rs` (17 134),
`crates/tr-app/src/projectile.rs` (17 411), `crates/tr-core/src/damage.rs` (279),
`crates/tr-core/src/armor_sets.rs` (525), `crates/tr-app/src/death.rs` (435).

## Key entities
- `Player.cs::Hurt` (38197-38567) — the inbound-damage pipeline; the single funnel
  every melee contact, projectile, tile-contact and self-damage source goes through.
- `Player.cs::KillMe` (38807-38863) — death: the coin/gem drop half (ported);
  `PlayerDeathReason` + the death message (`:38807` argument) are the text half.
- `Player.cs::GetWeaponDamageMultiplier` (53388-53419) / `GetWeaponDamage`
  (53421-53429) / `GetWeaponCrit` (53371-53386) — the player-dealt class multipliers
  (`(int)(damage * mult + 5E-06f)`).
- `Player.cs::ItemCheck` (42595+) — the per-tick item dispatch; `ItemCheck_Shoot`
  (`:47056`), `PickAmmo` (53458-53683), `CheckMana` (52576-52595),
  `ItemCheck_UseMiningTools` (45584-45678) are its arms.
- `Main.cs::DamageVar` (66118-66134) / `CalculateDamageNPCsTake` (66147-66154) /
  `CalculateDamagePlayersTake` (:66160, cited by the port) / `CalculateDamagePlayersTakeInPVP`
  (`:38364` caller).
- `crates/tr-core/src/damage.rs::damage_var` (:30) — `DamageVar` + the luck reroll,
  banker's rounding (`round_bankers` :13).
- `damage.rs::calculate_damage_npcs_take` (:50) / `strike_npc_damage` (:88) /
  `check_armor_penetration` (:59) / `knock_back_npc` (:98) / `iframes_for` (:172) /
  `crit_roll` (:186) — the pure combat math the callers own the entities for.
- `crates/tr-core/src/armor_sets.rs::reduce_damage` (:62) — the player-side defense
  step (`Main.CalculateDamagePlayersTake`).
- `crates/tr-app/src/player.rs::Player::hurt` (:2247) / `hurt_inner` (:2255-2295) /
  `hurt_fall` (:2300) — the port's whole inbound pipeline (55 lines vs 370).
- `player.rs::weapon_damage_multiplier` (:2371) / `class_crit` (:2383) — the class
  stat accessors; `reduce_damage` + `endurance` are folded into `hurt_inner`.
- `world_sim/editing.rs::update_player` (:212) — the item-use tick: consumables
  (:465-527), the shooter use start (:562-599), `CheckMana` (:627-647), the generic
  shoot (:697-703, :897-936), the melee hit (:1080-1135), the ammo fire
  (:1496-1666), the `itemTime` re-arm (:1666).
- `ammo.rs::pick`/`drain`/`not_consume`/`resolve_projectile` (:219/:276/:711/:556) —
  the `PickAmmo` port, including the not-consume roll chain and the RNG draw count.
- `npc.rs::Npc::hurt` (:4027) / `hurt_from_projectile` (:4062) — the NPC-side damage
  entry points; `contact`/`contact_damage` → `player.hurt` at `npc.rs:12333`.
- `projectile.rs::ProjectilePool::damage_player` (:9730-9750) — the hostile-shot →
  `Player::hurt` bridge.
- `death.rs::death_message` (:266) / `death_text_key` (:250) — the `DotDeath` death
  text and `on_player_death` (:289).

## Dependencies
- Calls into: `tr_core::damage` (all combat math), `tr_core::armor_sets`
  (`reduce_damage`, `SetStats`), `tr_core::whip` (tag rows), `crate::regen`
  (mana-regen delay, life ramp), `crate::prefix::prefixed_stats`, `crate::ammo`,
  `crate::projectile` (pool), `crate::item::use_animation_secs`, `crate::registry`
  (`DamageClass`, item defs).
- Called by: `world_sim/player.rs::update_player` (fixed step) → `editing::update_player`;
  NPC contact (`npc.rs:12333`) and hostile projectiles (`projectile.rs:9747`) → `hurt`;
  `app/netglue.rs:379` for a client's NPC strikes.
- Shared data structures: `Player` fields (`stat_defense`, `endurance`, `hurt_cd`,
  `mana_cost`, `melee_crit`/`melee_damage`, `creative_god_mode`, `no_knockback`),
  `App` (`swing`, `swing_max`, `item_time`, `attack_cd`, `reuse_delay`, `item_rng`),
  `ItemSlot` + `prefix::PrefixedStats`, `EquipSlots`.

## Port status
| C# subsystem | Status | Rust location | Evidence / notes |
| --- | --- | --- | --- |
| `Hurt`'s creative-godmode arm (`:38203-38206`) | PORTED | `player.rs:2259-2261` | The only pre-iframe arm ported; returns false (0 damage). `the_godmode_power_swallows_hits_death_and_refills` (`player.rs:7222`) pins it. |
| `Hurt`'s iframe gate (`:38208-38213`, `ImmunityCooldownID`) | PARTIAL | `player.rs:2262-2264` | Only the General counter exists (`hurt_cd`); the `Sets.ImmuneTimerOnlyLimitsEffects` / non-General counters and the `flag2` remote-player bypass are absent. The 4-tick EoC dash immunity is hand-set (`npc.rs:12288`). |
| Shimmer dodge (`:38199`), mystic sash / blackBelt / brain-of-confusion / shadow dodge (`:38214-38236`) | MISSING | — | No `dodge`, `black_belt`, `shadow_dodge`, `mystic_sash`, `brain_of_confusion` field exists anywhere in `crates/` (grep). No RNG draw, no accessory wiring (`ApplyEquipFunctional` arms), no `ShadowDodge`/`NinjaDodge`/`BrainOfConfusionDodge`. |
| `panic` buff 63 / `setSquireT2` buff 205 (`:38237-38244`) | MISSING | — | No counter buff on hurt. |
| `stealth = 1f` (`:38245`) | MISSING | `player.rs` has no stealth field | cosmetic/behavioural (Dye/rogue stealth). |
| Crit ×2 on inbound damage (`:38248-38251`) | NOT-FOUND (by design) | `hurt_inner` has no `crit` parameter | Note the C# doubles only `num` (the raw `Damage` int), which feeds `magicCuffs` (`:38293`) and the PVP return (`:38364`); the amount subtracted from `statLife` is `num2` (`:38383`), NOT the doubled value. The port's callers pass no crit flag at all. |
| `magicCuffs` mana restore (`:38291-38303`) | MISSING | — | `Player.magic_cuffs` flag not ported; no `ManaEffect`. |
| Endurance step (`:38304`) | PORTED | `player.rs:2274` | `(int)((1 - endurance) * num2)`; `endurance` is folded from sets/accessories (`player.rs:2617-2755`, `:3069`), tests at `player.rs:6067`. |
| Solar Flare `ConsumeSolarFlare` (`:38305-38320`), beetle orbs (`:38321-38337`) | MISSING | — | No `beetle_orbs`, no 95/96/97 buff handling in `hurt`. |
| Paladin team share (`:38338-38368`) | MISSING | — | No `defendedByPaladin`; single-player only, but the 25/75 % split and the `TeamDamageShare` cooldown are absent. |
| Net sends / combat text / `DamageTracker` / dismount (`:38369-38377`, `:38382-38384`, `:38396-38400`) | MISSING | — | No `CombatText`, no damage tracker; `mount::TryDismount` exists for mounts but not on hurt. |
| The `num2 < 1 → 1` floor (`:38378-38381`) | PORTED (S1-C-3, 2026-11-12) | `player.rs::hurt_inner` | The Rust applies no floor after the endurance truncation. A 1-damage hit past 25 % endurance lands for 0 in the port (asserted as intended by `the_endurance_rows_cut_the_incoming_damage`, `player.rs:6093-6097`), where `Player.cs:38378-38381` floors it to 1 and the iframes are then 20, not 40. `iframes_for(0, …)` returns 40 (damage != 1) — both the HP loss and the window diverge. **Fixed (S1-C-3, 2026-11-12):** `hurt_inner` now floors `amount.max(1)` after `:38304`; the hit lands for 1 and `iframes_for(1, ...)` returns 20. Test: `a_sub_one_hit_floors_to_one_and_uses_the_short_iframe`. |
| Immunity window (`:38385-38394`) | PORTED (partial) | `player.rs:2279` + `damage.rs:172` | `iframes_for` transcribes 40/80/20/PVP-8 exactly; the caller always passes `pvp=false, long_invince=false`, so 80/8 are unreachable and the General `immune`/`immuneTime` pair is the port's `hurt_cd` (`player.rs:2494` decay). |
| `lifeRegenTime = 0` (`:38395`) | PORTED | `player.rs:2281` | |
| Retaliate set bonuses — brain of confusion, star cloak / bee / honeycomb (`:38401-38516`) | MISSING | — | No retaliate arm. |
| Fixed contact knockback (`:38519-38524`) | PORTED | `player.rs:2282-2293` | `4.5 * hitDirection` / `-3.5` / `on_ground=false`; the port derives direction from `from_x` and gates on `no_knockback`. It drops `fallStart` reset and the mount-cart gate, both documented at `player.rs:2251-2254`. `the_horseshoe_and_the_shield_grant_their_flags` (`player.rs:7262`) pins the `noKnockback` gate. |
| Hurt sound / eye blink / hit dust (`:38525-38553`) | MISSING | — | Cosmetic; no per-hit dust loop keyed to `num2/statLifeMax2`. |
| Death branch (`:38554-38561`) → `KillMe` | PARTIAL | `death.rs:289`, `death.rs:250-287` | `on_player_death` does the softcore gem/coin drop and the tombstone; the message exists only for `DotDeath` sources (`death_text_key_matches_the_by_other_table`), not for a `PlayerDeathReason` from `Hurt`. |
| PVP recompute (`:38562-38565`) | MISSING | — | `Loadout`/PVP damage unmodelled. |
| `GetWeaponDamageMultiplier` bow/gun/specialist splits (`:53396-53407`) | PARTIAL | `player.rs:2371-2379` | Only melee/ranged/magic/summon; `bowEffectiveDamage`, `gunEffectiveDamage`, `specialistEffectiveDamage` and the AmmoID `IsArrow/IsBullet/IsSpecialist` routing are absent — every ranged shot rolls the plain `rangedDamage`. |
| `GetWeaponDamage` (`:53421-53429`, `(int)(d*mult + 5E-06f)`) | PARTIAL | `editing.rs:697`, `editing.rs:1587-1608` | The port keeps the product as `f32` and truncates later (`as i32` at `editing.rs:916`, `damage as i32` at `:1650`), so the truncation point moved after the ammo fuse / `raw as i32` — a rounding-order difference against `:53428`. |
| `GetWeaponCrit` + `Next(1,101) <= crit` (`:53371-53386`) | PORTED | `player.rs:2383` + `damage.rs:186` | Rolled at throw time from the item stream (`editing.rs:701-703`, `editing.rs:1633-1636`). `crit_roll_is_the_1_to_100_le_crit_gate` pins it. |
| `DamageVar` on the player-dealt melee hit (`:44342`) | PORTED | `editing.rs:1091-1101` | `damage_var(raw, luck)` after `strike_npc_damage`'s defence, per `ProcessHitAgainstNPC`. Pinned by `test_DW_3_2_melee_hit_passes_the_player_luck_to_damage_var` (`editing.rs:4060`). |
| `StrikeNPC_Inner` (defence ×0.5, crit ×2, floor 1) + armour pen | PORTED | `damage.rs:50/59/88`, used at `editing.rs:1124` | 8 tests in `damage.rs:190-297`. |
| `Projectile.Damage`'s `DamageVar` + defence (`Projectile.cs:12938`) | FAIL | `projectile.rs:9703` → `npc.rs:4062` | `hurt_from_projectile` takes the raw `damage` with no `damage_var` roll and no defence subtraction (`grep damage_var|strike_npc_damage|defense()` in `projectile.rs` → no matches). The ROADMAP's M4-DAMAGE admission covers only the contact hit, not this path. Owner/luck is also not threaded (the `owned_by_someone` arm, `Projectile.cs:12938`). |
| NPC → player contact damage (`:31387`, `Update_NPCCollision` `:31423`) | PARTIAL | `npc.rs:12326-12334` | Flat `contact_damage()`, no `DamageVar(npcdmg, -luck)` roll and no `damageMultiplier`; the town-NPC exclusion is a hard-coded kind list (`npc.rs:12297-12325`) rather than the friendly-AI check. |
| `ItemCheck`'s frozen/webbed gate (`:3381`) | PORTED | `editing.rs:331-338` | |
| Consumables: food (`:42756` wellFed gate), healing potions (`:42756` + `ApplyPotionDelay`/`ApplyLifeAndMana`) | PORTED | `editing.rs:465-527` | Buff 26/206/207 gate + 14400-ticks fallback; buff 21 at 3600 ticks gates the re-drink. M3-ITEM-4 verified. |
| `CheckMana` gate + pay (`:52576-52595`) | PORTED | `editing.rs:633-647` | `(int)(mana * mana_cost)`, a failed check blocks the use, the pay re-arms `apply_mana_regen_delay`. The manaFlower quick-mana arm is absent (also admitted by the row). |
| Use-time cadence: `useAnimation`/`useTime`, the volley re-arm | PORTED | `item.rs:30`, `editing.rs:581-597`, `editing.rs:1666`, `prefix.rs:389` | Swing in seconds, `item_time` re-armed per shot (`SetItemTime(useTime)`), the first shot of a swing on the next tick (`editing.rs:584-598`); the prefix `spd` scaling uses `round_bankers` (`prefix.rs:389-390`). |
| Ammo pick / drain / resolve chain / not-consume rolls | PORTED | `ammo.rs:219/276/556/711` | Documented order-preserving port with the golden-stream RNG test (`ammo.rs:1488`). The two dead arms (`:53639`, `:53659`) are deliberately omitted. |
| Tool usage hooks (`ItemCheck_UseMiningTools`, `PickTile`) | PARTIAL | `editing.rs:980+`, `tr-core/src/edit.rs`, `tr-core/src/tool.rs` | The tile-first order is ported; the mining path lives beside the combat arm (module 13's scope). |

## Tests
- `tr-core/src/damage.rs:190-297` (8): `damage_var_stays_inside_the_15_percent_band`,
  `damage_var_is_deterministic_per_seed`, `positive_luck_rerolls_keep_the_higher`,
  `bankers_rounding_matches_the_c_sharp_midpoints`,
  `npc_damage_takes_defense_halved_with_a_floor_of_one`,
  `strike_npc_line_applies_the_crit_after_defense`,
  `armor_penetration_caps_at_half_defense`, `crit_roll_is_the_1_to_100_le_crit_gate`.
- `player.rs`: `the_defense_reduces_the_hurt_damage` (:6293),
  `the_endurance_rows_cut_the_incoming_damage` (:6067 — encodes the missing floor-1),
  `the_godmode_power_swallows_hits_death_and_refills` (:7222),
  `the_horseshoe_and_the_shield_grant_their_flags` (:7262, the no-knockback gate),
  `fall_damage_ignores_iframes` (:7843), `the_mana_cost_multiplier_folds_every_row`
  (:6131).
- `world_sim/editing.rs`: `test_DW_3_2_melee_hit_passes_the_player_luck_to_damage_var`
  (:4060) — the only test covering the melee damage chain.
- `ammo.rs`: `test_the_rng_golden_stream_spends_the_vanilla_draw_count` (:1488),
  `test_DW_3_2_the_fairy_queen_saves_only_inside_the_eight_tick_window` (:1360),
  `test_DW_3_2_the_acorn_slingshot_clears_the_ammo_box_save` (:1258), the cycling
  group (:865-1013) and the pair-table group (:1787-2223).
- `projectile.rs`: `the_hostile_shot_hurts_the_player_and_dies` (:9851),
  `default_penetrating_hits_use_the_owner_immunity_of_10` (:10113),
  `id_static_immunity_is_shared_across_slots_of_the_same_kind` (:10151).
- `death.rs`: `death_text_key_matches_the_by_other_table` (:314),
  `death_message_picks_the_variant_and_formats_the_name` (:331),
  `the_death_sets_the_lost_coins_state` (:420).
- Gaps: no test covers iframe duration after the endurance truncation, no test for
  the projectile-hit damage chain, no test for dodge (nothing to test), no test for
  the ranged specialist split.

## Deviations & risks
Behavioural, medium-high:
1. **The projectile→NPC hit skips `DamageVar` and defence** (`Projectile.cs:12938` vs
   `projectile.rs:9703`/`npc.rs:4062`). Every bow/gun/magic shot does its raw fused
   damage with no ±15 % variance and no defence halving; the crit is carried but the
   `strike_npc_damage` multiply happens nowhere on this path. Not admitted by the
   M4-DAMAGE row (which excuses only the *contact* hit). Severity: behavioural + a
   different RNG draw count per hit (determinism risk).
2. **The `num2 < 1 → 1` floor is missing** (`Player.cs:38378-38381`). 0-damage hits
   (high endurance / high defence) are legal in the port; the iframe window then uses
   `damage != 1` on an un-floored value. The existing test asserts the divergent
   behaviour, so the port's own "docs" disagree with the source. Severity:
   behavioural.
3. **`GetWeaponDamageMultiplier`'s ranged split is missing** — arrows never roll
   `bowEffectiveDamage`, bullets never `gunEffectiveDamage`, specialists never
   `specialistEffectiveDamage` (`Player.cs:53396-53407`). With the shot damage
   truncation moved after the ammo fuse, all ranged damage is off. Severity:
   behavioural.
4. **No dodge at all** (`:38214-38236`) — blackBelt/shadowDodge/brain/mystic sash are
   a 10 %-or-always miss that never happens, and the whole accessory wiring is absent.
   Severity: behavioural (rogue/assassin gear is inert).
5. **No `PlayerDeathReason`** — the death text only exists for `DotDeath`
   (`death.rs:250`), so `Hurt`-caused deaths lose their cause string. Severity:
   cosmetic (UI text).
Behavioural, low/medium:
6. **No retaliate / paladin / beetle / solar arms** in `Hurt` — all six defensive
   set families are inert on the inbound path.
7. **No combat text, no damage tracker, no hurt sound/eye blink/dust** — visual.
8. **No net/PVP path** — `netMode == 1` sends and `CalculateDamagePlayersTakeInPVP`
   are absent; single-player only. Admitted implicitly (MP transport design).
9. **NPC→player contact has no `DamageVar(npcdmg, -luck)`** (`Player.cs:31387`) and
   no difficulty `damageMultiplier`; the town-NPC gate is a kind list, not the
   friendly-AI check. Severity: behavioural (luck and the master/expert scaling).
10. **The `Crit` parameter is dropped end-to-end** — because the C# only doubles the
    raw `num` (fed to `magicCuffs` and PVP), the impact is limited to those two
    consumers; but the port cannot express it. Severity: behavioural, narrow.
Determinism: items 1, 2, 9 and 10 all change RNG draw counts / results on the shared
`item_rng` stream; any fix must re-run the golden-stream tests (`ammo.rs:1488`,
`editing.rs:4060`).

## Task seeds
- **M5-COMBAT-1 — floor and iframe consistency (S).** Port `if (num2 < 1.0) num2 =
  1` (`Player.cs:38378-38381`) before the `statLife` subtraction, feed the floored
  value to `iframes_for`, and update `the_endurance_rows_cut_the_incoming_damage`
  (`player.rs:6067`). Fixes deviation 2.
- **M5-COMBAT-2 — projectile hit chain (L).** Thread `Projectile.Damage`'s
  `DamageVar` + owner luck (`Projectile.cs:12938`) plus `strike_npc_damage`
  (defence floor-1, crit ×2) into `ProjectilePool`'s NPC hit, using the item stream
  in the same draw order as the vanilla. Depends on a decision about which RNG the
  combat rolls draw from (the same question the M4-DAMAGE contact admission parks).
  Fixes deviation 1; re-verify `ammo.rs:1488`.
- **M5-COMBAT-3 — dodge family (M).** Port `Player.Hurt`'s dodge block
  (`:38214-38236`) and the four accessory flags (`blackBelt`, `shadowDodge`,
  `mysticSashDodge`, `brainOfConfusionItem`) plus their `ApplyEquipFunctional` grants.
  Depends on M3-EQUIP-3's flag table. `crit_roll`-style draws must be ordered before
  the defence step.
- **M5-COMBAT-4 — ranged damage split (S).** Add `bow_effective_damage` /
  `gun_effective_damage` / `specialist_effective_damage` and the AmmoID
  `IsArrow/IsBullet/IsSpecialist` routing to `weapon_damage_multiplier`
  (`Player.cs:53396-53407`), then move the `GetWeaponDamage` truncation to the
  `:53428` point. Fixes deviation 3.
- **M5-COMBAT-5 — `PlayerDeathReason` (M).** Introduce a compact death-cause enum and
  route `hurt`/`kill` through it so `death.rs`'s message table covers contact,
  projectile, DoT and fall causes (`Player.cs:38807`). Fixes deviation 5.
- **M5-COMBAT-6 — contact DamageVar + difficulty scaling (S).** Apply
  `DamageVar(npc.damage * multiplier, -luck)` at `Player.cs:31387` in
  `npc.rs::contact`, with the difficulty multiplier. Fixes deviation 9.
- **M5-COMBAT-7 — remaining `Hurt` arms (M).** `magicCuffs`, retaliate (brain/star
  cloak/honeycomb), paladin share, beetle orbs, solar flare, combat text. Grouped
  because they share the same insertion point between the endurance step and the
  iframe set (`Player.cs:38305-38516`).
- **DONE re-verification verdicts:** M2-COMBAT-1 — PARTIAL (the math is exact, the
  plumbing is not: no cooldown counters, no floor-1, no dodge). M2-COMBAT-2 —
  PARTIAL (crit/class accessors and the roll are ported and used; the ranged split
  and bow/gun/specialist multipliers are not). M3-ITEM-4 — PASS (`CheckMana` gate+pay
  and the potion-sickness gate match; only manaFlower quick-mana is open, as the row
  itself says). M4-DAMAGE — PASS for the pure functions, FAIL for the wiring
  (projectile hits). M3-EQUIP-5 — PASS (`stat_defense` is derived per tick from the
  pieces + sets and feeds `reduce_damage`).
