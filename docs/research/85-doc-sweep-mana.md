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

---

# ADDENDUM — MANA RESIDUALS (W4-f49 continuation, 2026-11-21, commits 7aef18b6..onward)

The five residual items after the DOC-SWEEP. **Code-only; every new witness
WRITTEN, NOT RUN.**

## R1. Mana Flower auto-consume — census COMPLETE, ONE real gap PORTED

**The flag chain (all faithful, no changes).** C# `manaFlower` has exactly
EIGHT sites: the field (`Player.cs:2408`), the grants 3991 Arcane Flower
(`:14405-14410`), 4000 Magnet Flower (`:14455-14458`), 4001 Mana Cloak
(`:14459-14463`), 555 Mana Flower (`:15591-15594`), the ONE post-loop
`manaCost -= 0.08f` fold (`:12962-12965`), the `ResetEffects` clear
(`:18721`) and the `CheckMana` gate (`:52588`). NO `ItemCheck` or
`UpdateManaRegen` read exists — the auto-consume is ONLY `CheckMana`'s
flower arm. The port arms all four kinds (`player.rs` equip match), folds
once post-loop, clears in the reset block. Witnesses
`s1_p_i19_the_mana_flower_fold_applies_once` + the 3991/4000/4001 twins
already pin it.

**The use-chain census (C# `CheckMana` callers, ALL nine):**

| C# site | Weapon | Port status |
| --- | --- | --- |
| `Player.cs:52650` `ItemCheck_ActuallyPayMana` | every mana item | LIVE — `editing.rs` → `check_mana_with_flower` (f49) |
| `Projectile.cs:66613` `AI_075` type 633 | Last Prism | LIVE — `ai_last_prism` + the `wants_prism_flower_drink` intent drain (f49) |
| `Projectile.cs:66479` `AI_075` type 460 | Charged Blaster Cannon | **WAS A REAL GAP — PORTED (7aef18b6)** |
| `Projectile.cs:66204` `AI_075` type 439 | Laser Machinegun | AI body unported (the i22 OUT list, `projectile.rs` documented skips) — the CheckMana site rides that lane |
| `Projectile.cs:36943` `AI_208` | Glacier Fang (1115) | AI body unported — same |
| `Projectile.cs:42662` `AI_185` | Life Drain | AI body unported — same |
| `Projectile.cs:44567` `AI_100` | Medusa Head (535) | AI body unported — same |
| `Player.cs:56870` `CommandForbiddenStorm` | Forbidden set bonus | deferred (ANALYSIS, the storm command's lane — `key_double_tap_down` records it) |

**The blaster gap (the port's `charged_blaster_step`, S1-P-i22):** the
`flag3` pay was a hand-rolled `if p.mana >= item.mana { pay } else { false
}` — THREE deviations from `CheckMana` (`Player.cs:52576-52617`): (a) no
`manaCost` scale (`:52579`), (b) the pre-V2 refusal killed the holder on a
short bar (the shipped ManaV2 never refuses — `:52610-52614` drains to 0,
arms `slowMagicUse`, returns true), (c) no flower lane. Fixed: the pay now
rides `regen::check_mana` (the prism's pool-side convention) and the
short+flower tick records the shared `wants_prism_flower_drink` intent —
`drain_prism_flower_drinks` walks ALL slots, so the existing
`world_sim/projectiles.rs` pass-tail re-runs the FULL `CheckMana`
(`check_mana_with_flower_on`, `UpdateProjectiles` stream) with no new
machinery. ALSO fixed: the charge clock hard-coded `ai[0] += 1.0` ("no
ManaV2") — `:66362` reads `GetSlowMagicUseRate()` (`:47028-47035`): 1/2.5
while the slow cast is armed. Mana-sickness applies through the drink
itself (`QuickMana` → `ApplyLifeAndOrMana` `:51938` grants 94; the
drink-while-sick path is the §2 census — no gate). 3 witnesses
(`w4_f49_the_blaster_pay_scales_and_never_kills_under_v2`,
`…_short_pay_owes_a_flower_drink`, `…_charge_slows_under_the_slow_cast`).

## R2. Magic Cuffs vs Celestial Cuffs — manaMagnet VERIFIED, stale notes fixed

The 2221 `manaMagnet` "unported" note was stale in THREE places: the
PORT_MAP cuffs row (`:202`), `item.rs`'s `GrabFlags` doc ("has not landed
the Mana Magnet / Treasure Magnet grants"), and `player.rs`' 4000 comment
("its manaMagnet half unported"). i52 landed everything:

- Grants: 2219 Celestial Magnet (`:14940-14943`), 2220 Celestial Emblem
  (`:14944-14948`), 2221 Celestial Cuffs (`:14949-14953`, + `magicCuffs`),
  4000 Magnet Flower (`:14455-14458`, + `manaFlower`); `ResetEffects`
  clear `:18491`. Port: `player.rs` 2219/2220/2221/4000 arms + reset;
  witness `s1_p_i52_the_magnet_accessories_arm_mana_magnet`.
- Range: `GetItemGrabRange` `:35137-35140` adds `Item.manaGrabRange = 300`
  (`Item.cs:49963`) for the star trio {184, 1735, 1868}. Port:
  `item.rs::get_item_grab_range` + the `GrabFlags.mana_magnet` feed
  (`world_sim/items.rs:177`).
- Pull: `GrabItems` `:34942-34946` — `flag2` = the star trio;
  `manaMagnet & flag2` → `PullItem_Pickup(12f, 5)` FIRST in the arm chain,
  BEFORE the `:34947` gate (`!flag2 || HeldItem.magic || statMana !=
  statManaMax2`) that stops star pulls at full mana without the magnet.
  Port: `items.rs:675-703` verbatim arm order.
- The 1595 Magic Cuffs carry NO magnet (the C# `:14936-14939` arms only
  `magicCuffs`) — the upgrade's delta is exactly the magnet + the shared
  +40 mana row (`:14925-14928`).

The two code docs corrected (f02ba1b8); the PORT_MAP row corrected in this
addendum's doc pass. New end-to-end witness
`w4_f49_the_cuffs_arm_the_star_magnet_end_to_end`: the real equip tick arms
the flag, the grab pass feeds it, a star 100 px out rides the −2.4 blend,
full-mana-without-magnet refuses, magnet-at-full-mana pulls.

## R3. The Nebula f12 reconcile — ONE-WRITER MERGE RECIPE (the contract)

**The two trees** (merge-base `a725acb0`, neither is an ancestor of the
other):

- THIS branch (`task/W4-f49-mana-regen-2`): `Player.nebula_level_mana/
  life/damage` fields (d117/f49), the three scan arms in `player.rs`'
  inline buff loop (`176..=178`/`173..=175`/`179..=181`, each re-finding
  its slot), `update_buffs_nebula_buffs` (the `:12070-12079` helper),
  `nebula_levelup` (the `:57030-57059` pickup grant), the chained
  `ResetEffects` zero (`:18755`), the regen READ side
  (`regen.rs::apply_nebula_buff_mana` via `RegenInputs.nebula_level_mana`,
  fed at `update_survival`), and the pickup consume arms
  (`world_sim/items.rs`, the skip-gate removed).
- f12 (`task/W4-f12-buff-frame-wiring`, tip d62ef0bc at census): the g27
  canonical `buff_effects_orbs.rs::apply_orb_arm` owns ALL 18 orb ids
  (95-100/170-181) EXCLUSIVELY inside `buff_frame_wiring::update_buff_frame`
  — the tier derivation writes `Player.orb_scan.nebula_level_mana/life/
  damage` (`OrbScanState`), the Life/Damage CONSUMERS fold there
  (`fold_orb`: `buff_life_regen += 6 × tier`, the `0.15 × tier` four-class
  adds). f12's tree has NO `Player.nebula_level_mana` field, NO regen
  nebula read (its `regen.rs` predates d117), NO `nebula_levelup`, and its
  `items.rs` still carries the `IsAPickup` skip-gate.

**Which field survives: `Player.nebula_level_mana` (this branch's).** The
regen engine's read side (`RegenInputs.nebula_level_mana` →
`apply_nebula_buff_mana`, `Player.cs:19421`/`:19565-19585`) is the consumer
that matters and it reads the Player field. f12's `orb_scan` twin is a
scan-local accumulator whose mana arm has NO consumer in either tree
("the mana-regen consumer reads nebulaLevelMana in its own lane" — the g27
comment). Keeping `orb_scan.nebula_level_mana` would strand the regen
engine; keeping the Player field costs one bridge line.

**The recipe (apply at the merge, in order):**

1. **Keep f12's writer, delete this branch's three inline scan arms.**
   `apply_orb_arm` is the keep-one canonical (the g27 contract, the c16
   anchor row); the inline `176..=178`/`173..=175`/`179..=181` arms in
   `player.rs`' buff loop and the `update_buffs_nebula_buffs` helper are
   the SAME `:12070-12079` port — delete them (the helper's logic lives in
   `buff_effects_orbs.rs::nebula_buffs`). Two writers on one buff bar
   would double-step the `buffTime == 2` edge (each writer re-arms 480,
   so the second writer's `buffTime == 2` test would never fire — but the
   tier ASSIGN would run twice and the slot's `buffType--` could
   double-decrement if both saw time 2; keep ONE). Canary housekeeping:
   the `anchors.txt:904` row's marker is `nebula_levelup` (KEPT), but its
   prose names the deleted helper — reword the row to cite
   `buff_effects_orbs.rs::nebula_buffs` as the tier-writer's new home
   (never delete the row). This branch's four lane-12 scan witnesses
   (`w4_f49_the_nebula_scan_derives_the_tier_and_steps_down_at_time_two`,
   `…_the_mana_pickup_feeds_the_regen_engine_through_the_scan`, the
   levelup + reset twins) re-target: the levelup/reset twins keep their
   Player-field assertions (the bridge keeps the fields live); the two
   scan-driving witnesses move to the wiring call (the f12 gap-3
   convention — drive `update_buff_frame`, assert `orb_scan` + the
   bridged field).
2. **Bridge the mana tier in `fold_orb`** (`buff_frame_wiring.rs`): add
   `p.nebula_level_mana = s.nebula_level_mana;` (an ASSIGN — the C#
   `:12072` is assign-derivation, and `OrbScanState` starts at 0 each tick
   via `reset_effects`, so a stale field cannot survive a buff-less tick).
   The Life/Damage consumers STAY f12's (`buff_life_regen` + the
   four-class adds) — do NOT port this branch's Life/Damage stat writes
   (they were never landed here; the scan arms only wrote the fields).
   `Player.nebula_level_life/damage` fields may stay as the bridge twins
   (`fold_orb` assigns all three) or be deleted if nothing reads them —
   the regen engine reads ONLY the mana one.
3. **Keep this branch's `ResetEffects` chained zero** (`:18755`) AND
   f12's `orb_scan.reset_effects()` — they clear different objects (the
   Player fields vs the scan state); both are the C#'s per-frame zero.
   `nebula_levelup` (the pickup grant) writes the Player fields directly
   (`:57047-57056`) — KEEP it; its tier climb reads the field the bridge
   keeps warm. (In the C# the pickup's `NebulaLevelup` runs in
   `GrabItems` AFTER `UpdateBuffs`, so the field it increments is the
   scan-derived tier — the bridge must run BEFORE the items phase, which
   the frame order already guarantees: player → projectiles → items.)
4. **Move the regen READ after the wiring.** The C# order inside
   `Player.Update` is `ResetEffects` (`:25491`) → `UpdateBuffs` (`:25566`,
   the scan) → `UpdateEquips` (`:25633`) → `UpdateLifeRegen` (`:25977`) →
   `UpdateManaRegen` (`:25995`) → `GrabItems` (`:27428`). The port's
   `update_survival` (the regen engine) currently runs INSIDE
   `update_with_grapple` — BEFORE f12's wiring seam
   (`world_sim/player.rs`: `update_with_grapple` :1113 → summon arms
   :1191 → `update_buff_frame` :1230 → accessory arms :1293). Merged, the
   mana tier the regen reads would be the PREVIOUS tick's (the reset
   zeroes it at the head). FIX: lift the `update_survival` call out of
   `update_with_grapple` to the world-sim seam AFTER `update_buff_frame`
   (before the accessory pass is fine — the C# regen reads sit after
   UpdateEquips too). This ALSO fixes f12's own `buff_life_regen` feed:
   on f12 alone the fold (`:1230`) lands after the regen read (`:11454`
   inside `update_with_grapple`) and the next tick's reset (`:6256`)
   wipes it before the read — the Nebula Life regen add NEVER reaches
   `update_life_regen` on f12's tree; the lifted order repairs both.
   Record the lift in ANALYSIS as the merge's ordering fix. The
   standalone `Player::update` wrapper (the no-wiring callers: equip
   tests, the potion rigs) re-adds the `update_survival` call AFTER its
   `update_with_grapple` so those paths keep their regen tick (their
   inline scan arms are gone, so their nebula tier stays 0 — the same
   "the wiring owns the scan" shape the f12 gap-3 werewolf witnesses
   already record). The wiring's own witnesses drive `update_buff_frame`
   directly (the f12 convention).
5. **Keep this branch's `items.rs` wholesale** (the skip-gate removal +
   the nebula/4143 consume arms + the nebula pull): f12's `items.rs`
   changes are the i52-era arms this branch already contains; the
   conflict resolves to THIS branch's version plus any f12-only rows
   (none in the pickup block at census).
6. **The 4143 producer + the Life/Damage consumers stay their lanes'**
   (the projectile-Kill lane's 723 arm; f12's `fold_orb`) — unchanged
   from the §1 merge note.

**Scan-position C# order (the merged tick, verbatim):**
`buffs.update_buffs_tick` (the `:9871-9873` timer head — f12 keeps it in
`update_with_grapple`) → `ResetEffects` (both zeros) → `UpdateEquips` →
`update_buff_frame` (the `:9857-9864` frame head, the arm chain with
`apply_orb_arm` at the `:10444-10461` position — Mana → Life → Damage per
slot, the `:12046-12067` tails, `fold_orb` + the mana bridge) →
`update_survival` (the `:25977`/`:25995` regen reads, NOW post-scan) →
projectiles phase → items phase (`GrabItems` → `NebulaLevelup` increments
the bridged field; the NEXT tick's scan re-derives it from the buff bar —
the assign-derivation makes the pickup's field write a same-tick hint
only, exactly the C# shape).

## R4. Star pickup mana-restock sound — VERIFIED landed (f29b2086)

`PickupItem`'s star arm plays `PlaySound(7)` (`Player.cs:35067`) — the
UNPOSITIONED Grab chime, legacy id 7 (`SoundID.Grab`), NOT Item style
23/25 (those are `sound_id 2` use cues; the "case-25/Item23" guess in the
lane brief is wrong — the pickup family is id 7). ONE `Next(-10, 11) *
0.01` pitch draw (`LegacySoundPlayer.cs:725`) on `UpdatePlayers` (the
`GrabItems` swap, `Main.cs:17754`). `ManaEffect` (`:4531-4538`) is
TEXT-ONLY — no second cue. The heart (`:35061`) and 4143 (`:35082`) ride
the same unpositioned play; only the nebula trio is POSITIONED (`:35047`,
the f49 `play_grab_sound_at`). The i52 arm's
`audio.play_pickup_sound(false, UpdatePlayers)` is verbatim. Witness
`w4_f49_the_star_pickup_plays_the_unpositioned_grab_chime` pins the
count+1 play, the pitch draw spent, and the silent-listener refusal (the
restore is not sound-gated; the CombatText jitter still draws —
`CombatText.cs:88-89` is not sound-gated).

## R5. Super Mana Potion 2209 + the tier rows — census MATCH (37ed7d2f)

| id | name | C# healMana | Item.cs | generated heal_mana |
| --- | --- | --- | --- | --- |
| 110 | Lesser Mana Potion | 50 | `:2680-2691` | 50 ✓ |
| 189 | Mana Potion | 100 | `:3701-3713` | 100 ✓ |
| 500 | Greater Mana Potion | 200 | `:7336-7348` | 200 ✓ |
| 2209 | Super Mana Potion | 400 | `:22346-22358` (the early-return arm) | 400 ✓ |

All four: `useStyle 9`, `useTurn`, `consumable`, `useTime/useAnimation
17`, `UseSound Item3`, NONE `potion`-flagged (no Potion Sickness 21 — only
Mana Sickness 94 via `:51938`). The drink restores RAW capped at
`statManaMax2` (`:51920-51928`), queues the RAW `ManaEffect` (`:51950`),
`manaPotionDelay = 0` (`:51937`). The `QuickMana` chooser is FIRST-fit
(`:5629-5649`) — a Super behind a Lesser does not win. Witness
`w4_f49_the_mana_potion_tiers_match_the_csharp_rows` drives all four rows
through `apply_life_and_or_mana` + the chooser order.

## Addendum witnesses (WRITTEN, NOT RUN — code-only)

- `projectile.rs` ×3: the blaster pay/scale/V2, the flower intent, the
  slow charge clock (+ the `blaster_rig` helper).
- `world_sim/items.rs` ×2: the cuffs→magnet end-to-end, the star chime.
- `potions.rs` ×1: the four-tier rows + the first-fit chooser.

## Addendum files touched

| File | Change |
| --- | --- |
| `crates/tr-app/src/projectile.rs` | the blaster CheckMana pay + the slow charge rate + the shared-intent docs, ×3 witnesses |
| `crates/tr-app/src/world_sim/projectiles.rs` | the drain comment widened to both holders |
| `crates/tr-app/src/item.rs` | the GrabFlags stale doc corrected |
| `crates/tr-app/src/player.rs` | the 4000 stale comment corrected |
| `crates/tr-app/src/world_sim/items.rs` | ×2 witnesses |
| `crates/tr-app/src/potions.rs` | ×1 witness |
| `docs/research/85-doc-sweep-mana.md` | this addendum (the R3 merge recipe) |
| `docs/PORT_MAP.md` | the cuffs row's stale note + the mana row's blaster gap update |
| `anchors.txt` | the blaster canary row |

---

# FINAL-STATUS — MANA FINAL (W4-f49 continuation, 2026-11-21, commits 8eb20d8b..onward)

The lane's final phase. **Code-only; every new witness WRITTEN, NOT RUN.**

## F1. The R3 contract EXECUTED on this branch (8eb20d8b)

The R3 recipe above is no longer a merge note — this branch's half is
DONE, so the f12 merge only has to land its own writer + wiring:

1. **The three inline scan arms (`176..=178`/`173..=175`/`179..=181`) and
   `update_buffs_nebula_buffs` are DELETED** from `player.rs`. The g27
   canonical `buff_effects_orbs.rs::apply_orb_arm` (f12, adopted
   byte-exact per its phase-1 contract, cross-verified byte-identical at
   its final item 5, tip 61724db38) is the ONE tier writer. The f12-side
   contract (`W4-f12-sweep-census.md` "the orb_scan nebula twin")
   verified the two writers SEMANTICALLY IDENTICAL line-by-line — this
   deletion is the keep-one resolution, binding (b): the bridge retargets
   the scan's write onto the d117 Player field.
2. **The `fold_orb` bridge ASSIGN stub landed**:
   `Player::fold_orb_nebula_bridge(mana, life, damage)` — the three
   `p.nebula_level_* = s.nebula_level_*` lines f12's `fold_orb` carries
   verbatim at the merge (ASSIGNs, not adds: the `:12072` assign-
   derivation + the `OrbScanState` zero-seed make a stale field
   impossible). The Life/Damage consumers stay f12's `fold_orb`; the
   regen read keeps `Player.nebula_level_mana`.
3. **`nebula_levelup`, both ResetEffects zeros, and `items.rs` wholesale
   KEPT** — unchanged.
4. **`update_survival` LIFTED** out of `update_with_grapple` to the
   world-sim seam in `world_sim/player.rs`, AFTER
   `update_summon_buff_arms` (the `:25566` scan position where f12's
   `update_buff_frame` hooks per its phase-2 contract) and BEFORE the
   accessory pass — the C# `:25977`/`:25995` regen-read order. The
   standalone `Player::update` wrapper re-adds the call for the
   no-wiring callers (equip tests, potion rigs). The lift also repairs
   f12's own `buff_life_regen` feed (its fold landed after the regen
   read; the next reset wiped it before any read).
5. **Witness re-targets per the recorded notes**: the levelup/reset
   twins keep their Player-field assertions verbatim; the two
   scan-driving witnesses
   (`w4_f49_the_nebula_scan_derives_the_tier_and_steps_down_at_time_two`,
   `…_the_mana_pickup_feeds_the_regen_engine_through_the_scan`) now drive
   the bridge + the lifted read (the f12 gap-3 convention — the wiring
   owns the scan; the no-wiring wrapper's tier stays 0). One NEW seam
   witness (`world_sim/player.rs::w4_f49_the_lifted_regen_tick_runs_after_the_wiring_seam`)
   pins the lift's presence through the real pass. The `anchors.txt:904`
   row reworded to cite `buff_effects_orbs.rs::nebula_buffs` as the
   tier-writer's home (the `nebula_levelup` marker KEPT).

**The f12d/f49-contract receipt (for the merger):** f49's half of the
one-writer contract is EXECUTED at 8eb20d8b. What remains for the f12
merge: (a) land `buff_frame_wiring::update_buff_frame` at the marked seam
(the comment block in `world_sim/player.rs` after
`update_summon_buff_arms`), (b) add the three bridge ASSIGNs to its
`fold_orb` (or call `fold_orb_nebula_bridge`), (c) resolve `items.rs` to
THIS branch's version (R3 step 5). No f49-side scan arm survives to
conflict; the `Player.nebula_level_*` fields + the chained reset are the
shared surface. The 4143 producer + the Life/Damage consumers stay their
lanes' (R3 step 6, unchanged).

## F2. The five CheckMana sites — the d-lane census UPDATED (0e91e70e)

The R1 census table's "AI body unported" column is STALE for three of
the five — the d-lanes landed since (tips read at this phase):

| C# site | Weapon | d-lane | Status NOW |
| --- | --- | --- | --- |
| `Projectile.cs:66204` AI_075 type 439 | Laser Machinegun | — | STILL unported (no d-lane covers AI_075's 439 holder arm; the i22 OUT list stands) |
| `Projectile.cs:36943` AI_208 | Glacier Fang 1115 | **d21** (tip 9fa9e9ba0) | LANDED — `projectile_ai_201_210.rs::ai_glacier_fang_shot`, the every-20-tick `ai[0] <= 60` charge pay ALREADY rides `regen::check_mana` on the held 6153 row's mana 10 |
| `Projectile.cs:42662` AI_185 | Life Drain (Soul Drain 476) | **d19** (tip 1054c396a) | LANDED — `projectile_ai_sword_styles.rs::ai_life_drain`, the first-tick overlap pay rides `check_mana` on the 3006 row's mana 10, then latches `localAI[0] = 1` |
| `Projectile.cs:44567` AI_100 | Medusa Head 535 | **d10** (tip 1f0b60e96) | LANDED — `projectile_ai_91_100.rs::ai_medusa_head`, the fire-cadence pay rides `check_mana` on the 3269 row's mana 15 (the `!flag \|\|` short-circuit never skips it) |
| `Player.cs:56870` CommandForbiddenStorm | Forbidden set bonus | — | STILL deferred (the storm command's lane; the flat 20 pay + `ApplyManaRegenerationDelay` `:56873`) |

The three landed bodies call the SHARED `regen::check_mana` — the same
machine the blaster/prism pays ride (the manaCost scale + the shipped
ManaV2 empty arm). Their flower lanes owe the same
`wants_prism_flower_drink` intent when they merge (the intent/drain docs
widened to name them; the drain walks ALL slots, no new machinery).

**The constants-copy** (`potions.rs`): `LASER_MACHINEGUN_MANA = 6`
(itemdata 2795), `GLACIER_FANG_MANA = 10` (6153), `SOUL_DRAIN_MANA = 10`
(3006), `MEDUSA_HEAD_MANA = 15` (3269), `FORBIDDEN_STORM_MANA = 20` (the
flat `:56870` pay, NOT a held-row read). These are WITNESS PINS only —
the AI lanes read `registry::item(...).mana`, never a hand-typed number.
One witness
(`w4_f49_the_five_checkmana_sites_pay_through_the_shared_machine`) arms
all five pay shapes through the shared machine: the row-vs-pin match,
the manaCost scale, the V2 empty arm, the flower re-check on
`UpdateProjectiles`, and the storm's delay reset.

## F3. The Last Prism flower-lane — FINAL witness coverage (9ee912e71)

The lane was verified faithful at R1 (the flag chain: 555/3991/4000/4001
grants, the ONE −0.08 fold, the `:18721` clear; the use chain:
ActuallyPayMana + the prism interval pay wired). The final coverage
check found ONE missing link: the pool-side witnesses pinned the intent
record/drain and the potions-side witnesses pinned the flower re-check,
but NO witness drove the **end-to-end pass-tail** — a real 633 holder's
short-pay intent drained by `update_projectiles` itself. Added:
`world_sim/projectiles.rs::w4_f49_the_prism_flower_drink_drains_end_to_end_through_the_pass`
(the holder at `ai[0]` 29→30, the first interval tick; short+flower →
the V2 empty arm + the intent → the pass-tail drink: 50 capped at 20,
the 12 re-pay → 8, the slow cast cleared, 94 granted, one potion
consumed, the intent taken, both streams pristine under the silent
listener; the full-bar no-drink + flowerless-V2 twins). Coverage is now
COMPLETE: grant → fold → pay → intent → drain → drink → re-pay → sickness.

## F4. Lane final verdicts

| Lane | Verdict |
| --- | --- |
| Nebula/4143 pickups + tier writer | PORTED; R3 EXECUTED (one writer = f12's g27, the bridge stub + the lift landed here) |
| Mana flower auto-consume | faithful; the blaster gap PORTED; the prism pass-tail NOW witnessed end to end |
| Magic/Celestial Cuffs | faithful (i52 complete; the stale notes fixed) |
| Star pickup cue | faithful (id-7 unpositioned Grab) |
| Mana-potion tiers | MATCH (all four rows) |
| Drink-while-sick / buff-6 ladder | faithful, witnessed |
| The five CheckMana AI sites | 3 of 5 LANDED on d10/d19/d21 (shared machine); 439 + ForbiddenStorm stay their lanes' |
| Mana crystals 20-ladder | g88b's (skipped per lane) |

**Remaining recorded seams (NOT gaps of this lane):** the 4143 producer
(723 Kill), the Life/Damage consumers (f12 `fold_orb`), the 439 holder
arm, the CommandForbiddenStorm lane, the f12 wiring merge itself (the
receipt above).

## FINAL witnesses (WRITTEN, NOT RUN — code-only)

- `player.rs`: the two re-targeted scan witnesses (bridge-driven).
- `world_sim/player.rs` ×1: the lifted-seam presence.
- `potions.rs` ×1: the five-site constants-copy pay shapes.
- `world_sim/projectiles.rs` ×1: the prism flower end-to-end pass-tail.

## FINAL files touched

| File | Change |
| --- | --- |
| `crates/tr-app/src/player.rs` | the scan arms + helper deleted, the bridge stub, the lift + the wrapper re-add, ×2 re-targeted witnesses |
| `crates/tr-app/src/world_sim/player.rs` | the lifted `update_survival` at the wiring seam + the seam marker, ×1 witness |
| `crates/tr-app/src/potions.rs` | the five-site constants-copy block, ×1 witness |
| `crates/tr-app/src/projectile.rs` | the intent docs widened to the d-lane holders |
| `crates/tr-app/src/world_sim/projectiles.rs` | the drain comment widened, ×1 end-to-end witness |
| `anchors.txt` | the :904 row reworded (the tier-writer's new home) |
| `docs/research/85-doc-sweep-mana.md` | this FINAL-STATUS + the f12d/f49-contract receipt |
| `docs/ANALYSIS.md` | the MANA FINAL section (the R3 execution + the lift as the ordering fix) |
| `docs/PORT_MAP.md` | the nebula row's R3-executed update + the mana row's d-lane census |
