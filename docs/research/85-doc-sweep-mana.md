# 85 — DOC-SWEEP lane 12: mana/magic (W4-f49 continuation, 2026-11-21)

Code-only doc-sweep on `task/W4-f49-mana-regen-2` (branched on the f49 second
half, 16415f2f). Mined `docs/ANALYSIS.md` + the regen/potion censuses
(`62-ammo-potion-census.md`, the d117/f49 sections) for unported mana-lane
items. **No compiler invoked; every witness below is WRITTEN, NOT RUN.**

Lane verdicts at a glance:

| # | Item | Verdict | Action |
| --- | --- | --- | --- |
| 1 | Nebula/4143 pickup consume arms | **REAL GAP** (f49 recorded leftover) | PORTED |
| 1b | Nebula tier-writer (`UpdateBuffs_NebulaBuffs`) | **REAL GAP** (d117 landed READ only) | PORTED |
| 2 | Mana sickness drink-while-sick | faithful; census now FULL | witnessed |
| 3 | Star-power mana crystals 20-ladder | g88b owns crystals | SKIPPED per lane |
| 4 | Magic Cuffs damage-to-mana | faithful (S1-C-7 wave A) | census verified |
| 5 | Mana Regeneration Potion buff-6 ladder | faithful (i19/E-7a) | witnessed |

## 1. The Nebula trio + ManaCloak star consume arms — PORTED

**The doc trail.** ANALYSIS "The W4-f49 mana-regen second half" recorded:
"The nebula-trio / 4143 consume arms (`PickupItem`
`:35045-35058`/`:35080-35098`) stay out — their `NebulaLevelup` buff-scan
derivation is the c16/c17 buff-effects lane's (in flight), and the mana-band
tier write (`UpdateBuffs_NebulaBuffs` `:12070-12079`) has no port writer
yet; d117 landed the regen READ side." The i52 deviations list repeated it
("their pickup arms stay unported"), and `world_sim/items.rs` carried a
skip-gate: `is_a_pickup(kind) && !is_heart_or_star(kind) → continue`.

**The f12 census (per lane instruction).** The recorded buff-scan lane is
`task/W4-f12-buff-frame-wiring` (tip 7b9fb486). It is NOT an ancestor of this
branch (`git merge-base --is-ancestor` exit 1) and its tree owns the tier
derivation as `orb_scan.nebula_level_mana`
(`buff_effects_orbs.rs::nebula_buffs(slots, …, 176, j)`), a SEPARATE field
from this branch's `Player.nebula_level_mana` (d117). Neither writer exists
in THIS tree — so the gap is real here and the port lands the writer on the
d117 field the regen engine already reads. **Merge note:** f12's `orb_scan`
twin and this branch's scan arms both port `:12070-12079`; the merge must
keep ONE writer feeding ONE field (the regen read side reads
`Player.nebula_level_mana`). The Life/Damage CONSUMERS (`lifeRegen += 6 ×
tier` `:10451`, the `0.15 × tier` four-class adds `:10456-10460`) are NOT
ported here — they stay f12's, recorded at the scan arms.

**What landed.**

- `Player::nebula_levelup` (`player.rs`) — `NebulaLevelup`
  (`Player.cs:57030-57059`): the `whoAmI != Main.myPlayer` gate is always
  false single-player; the `[type, type+3)` delete scan keeps the C#
  `DelBuff(i--)` rescan (the port's `del_buff` compacts identically, so the
  loop re-reads the same index); the family tier climbs ONE clamped 0..3
  (`MathHelper.Clamp`), `AddBuff(base + tier − 1, 480)`. Unknown base ids
  fall through the switch with the deletes done (the C# shape).
- `Player::update_buffs_nebula_buffs` (`player.rs`) —
  `UpdateBuffs_NebulaBuffs` (`:12070-12079`): `tier = 1 + buffType[b] −
  base` is an ASSIGN (the C# `ref` param's first statement overwrites it —
  a stale field cannot survive a tick with the buff up); the `buffTime ==
  2 && tier > 1` edge steps the returned tier AND `buffType[b]` down one
  and re-arms `buffTime = 480`. The timer read is POST-tick-head (the head
  decrements inside the same scan loop, `:9871-9873`), so a fresh 480
  tier-2+ buff steps down 478 ticks later.
- The three scan arms 176-178 / 173-175 / 179-181 (`:10444-10461`, C# chain
  order Mana → Life → Damage) in `player.rs`'s existing buff loop, each
  re-finding its slot like the neighbouring 94/257/383 arms. New fields
  `nebula_level_life` (`Player.cs:924`) / `nebula_level_damage` (`:930`)
  join `nebula_level_mana` (`:926`); `ResetEffects`' CHAINED zero
  (`:18755`: `nebulaLevelDamage = (nebulaLevelLife = (nebulaLevelMana =
  0))`) clears all three.
- The consume arms (`world_sim/items.rs`): the skip-gate is GONE. The
  nebula trio (`ItemID.Sets.NebulaPickup` = {3453, 3454, 3455},
  `ItemID.cs:1324`) plays the POSITIONED `PlaySound(7, (int)position.X,
  (int)position.Y)` (`:35047` — the player's top-left, NOT the unpositioned
  heart/star chime) via the new shared `audio::play_grab_sound_at` (split
  out of the chest-eat chime, same attenuation gate + Grab pitch draw),
  reads the item's `buffType` (`Item.cs:31008-31016`: 3453→179 Damage,
  3454→173 Life, 3455→176 Mana — the port's generated rows already carry
  them), consumes the drop and calls `nebula_levelup`. The netMode-1
  `SendData(102)` arm (`:35050-35053`) is MP-only, out.
- The 4143 arm (`:35080-35098`): the UNPOSITIONED `PlaySound(7)`,
  `statMana += 50`, `TryApplyManaHeat(0.2f, 2f)` + `ManaEffect(50)` (the
  RAW 50 prints BEFORE the cap; the `num2 != 0` gate is always true), cap,
  `TurnToAir` — the i52 100-star arm's shape at 50.
- The nebula PULL arm (`:34953-34956`): `PullItem_Pickup(12f, 5)` between
  the lifeMagnet-heart and void-vault arms — previously deferred with the
  consume arms. The +100 grab-range row (`:35157-35160`) was already live
  (i52's `get_item_grab_range`).

**Recorded seams (not gaps of this lane).**

- The 4143 PRODUCER: the ManaCloak star spawns from the 723 projectile's
  Kill (`Projectile.cs:75159-75177`: `owner == Main.myPlayer && ai[2] ==
  1f` → `Item.RequestNewItem(…, 4143)`). The port's projectile Kill lane
  has no 723 arm (the star-cloak SPAWN side landed with S1-C-7 wave B,
  `hurt_modifiers.rs`, but the Kind rows/Kill effects are the projectile
  lane's). The consume arm is complete for ANY 4143 drop.
- The `is_heart_or_star` predicate lost its only caller (the skip-gate);
  kept with `#[allow(dead_code)]` as the documented heart/star subset.

## 2. Mana sickness drink-while-sick — census COMPLETE

**The doc trail.** `62-ammo-potion-census.md` P3 flagged "Mana potions have
no use path; Mana Sickness never applied" — landed with i19/i37. The f49
first half witnessed the STACKING (additive cap 600) but through direct
`add_buff` calls, not the drink path. This sweep verifies the full
interaction against the C#:

- **No sick-drink gate exists.** `ItemCheck_CheckCanUse_Inner`'s potion
  gate (`:52217-52220`) reads `sItem.potion && potionDelay > 0`; mana
  potions are NOT `potion`-flagged (`Item.cs`'s 110 row sets `healMana`,
  `useStyle 9`, `consumable` — no `potion`). Mana Sickness (94) itself has
  NO use gate anywhere in `ItemCheck`. The port's `potion_use_blocked`
  matches (witnessed by the i37 buff-potion test).
- **The drink stacks.** `ApplyLifeAndOrMana` (`:51888-51952`): healMana
  identity-multiply truncation (`:51918`), the restore + max clamps, the
  `healMana <= 0` early return, `manaPotionDelay = 0` (`:51937`),
  `AddBuff(94, 300)` (`:51938`). `AddBuff_TryUpdatingExistingBuffTime`
  (`:5402-5426`) reads `AddBuffTimeAdditivelyToCap[94] = 600`
  (`BuffID.cs:53`): drink 2 inside the window lifts 300 → 600, drink 3
  caps. The `DebugOptions.ManaPotionDelay` 8×60 re-grant (`:51939-51943`)
  is constants-off (false retail static, `DebugOptions.cs:86`).
- **The scan assigns, the tail cuts once.** Buff-94 arm (`:10318-10327`):
  `manaSick = true`, `manaSickReduction = 0.25 × buffTime/300` — one ASSIGN
  per scan tick (up to 0.5 at the cap), never compounding. The recalc tail
  (`:25900-25903`) multiplies `magicDamage *= 1 − reduction` once per tick
  after every grant. 94 is absent from `BuffTimeIsExtendedWithGameDifficulty`
  (`BuffID.cs:50`) — no expert extension. All ported (i19/i37/f49); the new
  witness drives the WHOLE path: drink → drink-while-sick → cap → scan →
  damage cut → no compounding.
- **QuickMana drinks while sick too.** `QuickMana` (`:5598-5627`): the
  gates are pending-actions/cursed/CCed/dead/`manaPotionDelay > 0` (the
  full-mana arm is `!ManaV2`-gated, constants-off); the chooser
  (`:5629-5649`) skips only `potion`-flagged rows while `potionDelay > 0`.
  No mana-sick gate; the drink's `:51937` zero keeps `manaPotionDelay`
  open. Ported (`potions.rs::quick_mana_on`); witnessed.

## 3. Star-power mana crystals 20-ladder — SKIPPED

Per lane assignment: g88b owns the crystals lane. Not censused, not
touched. (The Arcane Crystal 5339 USE arm + the two regen read arms are
already on this branch — f49/d117 — and are NOT the 20-crystal ladder.)

## 4. Magic Cuffs damage-to-mana — census verified, NO port needed

**The doc trail.** ANALYSIS "The magic cuffs (S1-C-7 wave A)" records the
`Hurt` arm (`Player.cs:38291-38303`) landed. Re-censused against the C#:

- `num3 = (int)(num × 1f)` is the RAW incoming damage (crit-doubled at
  `:38248-38251`; every single-player caller passes `Crit: false`), NOT the
  defense-reduced `num2`. The arm sits AFTER the invis strip
  (`:38281-38290`) and BEFORE the endurance step (`:38304`).
- `statMana += num3`, cap at `statManaMax2`, `ManaEffect(num3)` on the
  local player (the NetMessage 43 send is MP-only).
- The port (`player.rs` `hurt_inner`): `raw` kept pre-reduction, crit ×2
  parity, restore + cap + `pending_mana_effect` — verbatim, at the same
  position (before the endurance multiply).
- Grants: 1595 Magic Cuffs (`:14936-14939`) + 2221 Celestial Cuffs
  (`:14949-14953`) arm `magic_cuffs`; `ResetEffects` clears (`:18599`); the
  +40 `statManaMax2` rows landed with i19. Witnesses
  `s1_c_7_magic_cuffs_restore_the_raw_damage_and_queue_the_text` /
  `…cap_the_restore_at_the_mana_max` / `…without_the_cuffs…` /
  `s1_c_7_magic_cuffs_arm_from_the_two_cuffs_and_clear` already pin every
  arm. No new code, no new witness needed.
- Note: the PORT_MAP cuffs row's "2221's `manaMagnet` half unported" is
  STALE — i52 landed the mana-magnet grants (the row predates i52); left
  for the cuffs lane's own doc pass, recorded here.

## 5. Mana Regeneration Potion buff-6 ladder — census verified, witnessed

**The doc trail.** ANALYSIS `:6743` records "ManaRegeneration 6 →
`RegenInputs::mana_regen_buff`". Re-censused end to end:

- Item 293 (`Item.cs:4872-4885`): `buffType = 6`, `buffTime = 28800`
  (8 min), `useStyle 9`, NOT `potion`-flagged — drinks through Potion
  Sickness like every buff potion (the i37 `drinkable` gate covers it:
  `buff_type > 0 && use_style 2|9`). The generated row matches.
- Buff 6 scan arm (`:9964-9967`): `manaRegenBuff = true` — a BOOL assign,
  one slot's worth of effect; there is NO per-tier ladder (unlike the
  nebula trios). `ResetEffects` clears (`player.rs:6089` region).
- The engine's four read sites (`regen.rs`, all live since i19/E-7a):
  1. `ApplyManaRegenerationDelay`'s cap 20 (`:19399-19402`) — runs BEFORE
     the shipped ManaV2 cap-4/pin, so both leave 4 (the i19 witness pins
     the ordering);
  2. the extra delay decrement (`:19426-19429`, shared with
     standing-still/grapple);
  3. the regen-line doubling (`:19449-19452`, same shared gate);
  4. `num9 = 1` (`:19462-19465`) — the mana-fraction scaler
     `num9 + (1−num9)·frac` pins at 1.0 instead of ramping 0.5→1.0.
- The existing `s1_e_7a_mana_regeneration_buff_feeds_the_regen_engine`
  witness pins sites 3+4 at frac 0.5 through the full player tick; the new
  `regen.rs` witness pins all four sites against no-buff twins at equal
  fractions (including the empty-bar scaler pin and the countdown).

## The witnesses (WRITTEN, NOT RUN — code-only)

- `player.rs` ×4:
  `w4_f49_nebula_levelup_climbs_the_ladder_and_replaces_the_family_buff`,
  `w4_f49_the_nebula_scan_derives_the_tier_and_steps_down_at_time_two`,
  `w4_f49_the_mana_pickup_feeds_the_regen_engine_through_the_scan`,
  `w4_f49_reset_effects_zeroes_all_three_nebula_tiers`.
- `world_sim/items.rs` ×3:
  `w4_f49_the_nebula_pickups_grant_their_family_buff_and_vanish`,
  `w4_f49_the_mana_cloak_star_restores_fifty_and_arms_mana_heat`,
  `w4_f49_the_nebula_pull_rides_the_twelve_five_blend`.
- `potions.rs` ×1:
  `w4_f49_drinking_while_mana_sick_stacks_the_timer_and_deepens_the_cut`.
- `regen.rs` ×1:
  `w4_f49_the_buff_six_ladder_doubles_the_line_and_pins_the_scaler`.

## Files touched

| File | Change |
| --- | --- |
| `crates/tr-app/src/player.rs` | the two nebula fields + chained reset, `nebula_levelup`, `update_buffs_nebula_buffs`, the three scan arms, ×4 witnesses |
| `crates/tr-app/src/world_sim/items.rs` | the skip-gate removal, the nebula + 4143 consume arms, the nebula 12/5 pull, ×3 witnesses |
| `crates/tr-app/src/audio.rs` | `play_grab_sound_at` (the shared positioned case-7; the chest-eat chime delegates) |
| `crates/tr-app/src/item.rs` | the `is_heart_or_star` doc/allow update (its skip-gate caller landed out) |
| `crates/tr-app/src/potions.rs` | ×1 drink-while-sick witness |
| `crates/tr-app/src/regen.rs` | ×1 buff-6 ladder witness |
| `docs/ANALYSIS.md` | the lane-12 section + three stale-claim supersede notes |
| `docs/PORT_MAP.md` | the nebula/4143 row + the mana-machinery row's gap list update |
| `anchors.txt` | the lane-12 canary rows |
