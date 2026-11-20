# W4-g27 — orb keep-one prep (the five-way diff + the canonical pick)

Branch `task/W4-g27-orb-keep-one-prep`, base `a725acb0` (= origin/main at
claim time). Code-only worker: **no cargo/rustc/rustfmt/tests/graphify
invoked**; witnesses written, not executed. Audience: **W4-f12
(`task/W4-f12-buff-frame-wiring`) and the wave mergers** — this doc is the
keep-one decision record for the lunar-orb buff arms.

## Scope

The 18 lunar-orb ids of `Player.UpdateBuffs` (`Player.cs:9855-12069`):

| family | ids | C# arm | gate field |
|---|---|---|---|
| Beetle Endurance | 95-97 | `:10342-10376` | `beetleDefense` |
| Solar Shield | 170-172 | `:10377-10407` | `setSolar` |
| Beetle Might | 98-100 | `:10408-10443` | `beetleOffense` |
| Nebula Mana | 176-178 | `:10444-10447` | — |
| Nebula Life | 173-175 | `:10448-10452` | — |
| Nebula Damage | 179-181 | `:10453-10461` | — |

plus the shared helper `UpdateBuffs_NebulaBuffs` (`:12070-12079`).
Neighbouring ids (94 Mana Sickness, 396 Mana Heat, 192 Sugar Rush, 321
Brain of Confusion, 62 Ice Barrier) are NOT in scope — they stay with
their wave modules; the canonical dispatch returns `NotHandled` for them.

## The five implementations (probed in-flight first: all five worktrees
clean, no uncommitted work; f12 had already gathered all five verbatim)

| branch | tip | module | orb shape |
|---|---|---|---|
| `task/W4-c16-buff-effects-wave-b` | `ffffa3b4` | `buff_effects_orbs.rs` (509 ln) | pure accumulator `OrbEffects` + per-slot REQUESTS (`buff_time_set`, `buff_type_decrement`, `delete_buff`, `rescan_slot`, `purge_band`); counters seeded from live fields by the caller |
| `task/W4-c17-buff-effects-wave-c` | `357dabed` | `buff_effects_potions_sets.rs` (611 ln) | slot-ordered driver `update_potion_set_buff_arms` over `&mut BuffSlots`; per-arm `rescan: bool`; counters start 0 per tick (the ResetEffects claim) |
| `task/W4-c18-buff-effects-wave-d` | `cef7e5aa` | `buff_effects_summons.rs` (697 ln) | **NO orb arms** — owns 62 + the 29 summon keep-alives (`:10462-10976`); listed for the census only |
| `task/W4-d73-buff-effects-wave-g` | `33c38000` | `buff_effects_boosters.rs` (703 ln) | direct `BuffSlots` mutation; `BoosterArmResult::NextSlot(i32)` adjusted-j contract; shared `orb_tier_sweep`/`orb_unset_tail` helpers; persistent `BoosterState` |
| `task/W4-d74-buff-effects-wave-h` | `3a80d60f` | `buff_effects_tail.rs` (707 ln) | pure accumulator `TailEffects` + requests (`clear_buff_range`, `buff_time_set`, `buff_type_decrement`, `beetle_orbs_set`…); counters read from a per-tick CONTEXT (caller-seeded) |

f12's gathered copies are byte-identical to the wave tips for the orb
modules (verified: `git diff c16 f12 -- buff_effects_orbs.rs` = 0 lines).

## Per-quirk diff vs the C# (the decision matrix)

The C# facts re-verified against `decompiled/Terraria/Player.cs` (line
numbers are the decompilation's):

- **Q1 — the 5-tick pin.** Endurance `:10344` and Solar `:10379` pin
  `buffTime[j] = 5`; Might has NO pin (`:10408-10443`). All five agree.
- **Q2 — the tier sweep directions.** Stored count > tier → `DelBuff(j);
  j--` (self-delete); stored count < tier → whole-bar walk
  `base..=base+tier-1` with `DelBuff(m); m--`. All five agree on the
  directions and the INCLUSIVE band.
- **Q3 — the upgrade walk deletes the CURRENT slot too, and the outer `j`
  is NOT decremented afterwards** (the shifted-in buff skips its arm this
  tick). c16 documents it explicitly; d73/d74/c17 implement it (the walk
  runs, `j` unchanged); the request-drain mergers must honour it.
- **Q4 — the double-delete quirk.** After a downgrade self-delete `j` was
  decremented; the `!setFlag` tail then deletes the ALREADY-adjusted `j`
  (the slot BEFORE the original) and decrements again. **d73 implements
  it verbatim** (`orb_unset_tail` takes the adjusted `jj`, guards `jj < 0`
  where the C# `DelBuff(-1)` throws). **c17 collapses it** to a single
  `rescan` bool (its doc argues the path is unreachable in retail play —
  true, but the collapse loses the quirk for the general scan). c16/d74
  record `delete_buff`/`rescan_slot` without the adjusted index — a
  request-drain that replays them at the ORIGINAL `j` deletes the wrong
  slot. **d73 wins Q4.**
- **Q5 — Might stats BEFORE the offense gate** (`:10431-10432` precede
  `:10433`): the damage/speed adds land even on the delete tick. All five
  agree (c16/d73/d74 pin it with witnesses).
- **Q6 — the nebula step-down.** `level = 1 + buffType - base`; at
  `buffTime == 2 && level > 1`: `level--`, `buffType[b]--`,
  `buffTime[b] = 480` — an IN-PLACE rewrite, no DelBuff, advance normally.
  All five agree; tier 1 at time 2 does NOT step (the `level > 1` guard).
- **Q7 — the nebula stat adds read the POST-step level** (`:10451` reads
  `nebulaLevelLife` after the helper; `:10456` likewise). All five agree.
  Life +6×level, Damage +0.15×level to all FOUR classes (one `num12`),
  Mana level-only. Levels are last-slot-wins assignments; the adds STACK
  per slot.
- **Q8 — the counter seeding (the decisive correction).** `ResetEffects`
  zeroes `beetleOrbs` (`:18501`), `beetleBuff` (`:18502`), `solarShields`
  (`:18505`) and the nebula trio (`:18755`) EVERY FRAME. So within the
  `UpdateBuffs` scan the counters start at 0 each tick; a kept slot's
  count is what the NEXT slot's sweep compares against.
  - **c17 is RIGHT**: its accumulator starts at 0 per tick and its doc
    cites `:18501-18505`/`:18755` correctly.
  - **d73 is WRONG on the claim**: `BoosterState`'s doc says "ResetEffects
    does NOT clear them in the C# (they persist while the buff family is
    absent)" — contradicted by `:18501`/`:18505`/`:18755`. Its ARM code is
    still correct IF the caller zeroes the state per tick; the claim as
    written would make a kept tier-3 slot self-downgrade on the next tick
    (stored 3 vs tier 3 is equal — actually safe for the equal case, but
    a stale count across armor swaps misfires the sweep). The correction
    is folded into the canonical `OrbScanState::reset_effects()`.
  - c16 seeds from "the live fields" (correct only if the wiring zeroes
    them first — its doc does not say); d74 reads a caller-seeded context
    (same caveat). The canonical module makes the zeroing explicit.
- **Q9 — the Solar arm has no `else` flag** (no `beetleBuff`, no stat
  adds; the shield count IS the state). All five agree.
- **Q10 — the set-flag citations.** c17's `SetBonusFlags` doc SWAPS the
  chestpieces: it says `beetleDefense` = "Beetle Scale Mail set" and
  `beetleOffense` = "Beetle Shell set". The C# is the reverse:
  `ApplySetBonus_BeetleDefense` (`:16499-16501`) is the Shell lane,
  `ApplySetBonus_BeetleDamage` (`:16529-16532`) the Scale Mail lane (c16
  has it right). Behaviourally irrelevant (both flags gate their own
  arms), but the canonical doc uses c16's mapping.
- **Q11 — RNG.** Zero `Main.rand` draws in `:10342-10461` +
  `:12070-12079` — all five census it; the canonical module is
  stream-neutral by construction.
- **Q12 — the timer head.** None of the 18 ids is in
  `TimeLeftDoesNotDecrease` (`BuffID.cs:47` = {28, 334, 29, 159, 150, 93,
  348, 366}), so the caller's timer tick decrements them before the arms
  run; the drivers must NOT decrement again. All five agree.

## The pick

**MOST FAITHFUL: d73 (`buff_effects_boosters.rs`)** — the only
implementation that mutates the bar directly with the adjusted-`j`
contract, preserving Q3/Q4 verbatim (the double-delete quirk + the
negative-index guard) instead of delegating them to a request drain that
can replay them wrong. Its arm bodies (pin → sweep → assign → stats →
gate) are line-for-line the C# order.

**CORRECTIONS ported into the canonical module
(`crates/tr-app/src/buff_effects_orbs.rs` on this branch):**

1. **c17's zero-seeding** (Q8): `OrbScanState::reset_effects()` — the
   `ResetEffects` writes `:18501`/`:18505`/`:18755`, called once per frame
   before the scan. d73's "persists" claim is dropped.
2. **c16's request vocabulary + band documentation** (Q2/Q3):
   `OrbArmRequests` + `orb_arm_requests()` — derived from the SAME arm
   core on a one-slot scratch bar (no second implementation), for mergers
   that keep the accumulator-drain wiring. `next_index` is the
   authoritative quirk encoding; `purge_band` is inclusive-of-self with
   no outer `j--`; `delete_buff` excludes the band's self-inclusion.
3. **d74's per-slot request names** (`buff_time_set`,
   `buff_type_decrement`) kept as the view's field names so a d74-shaped
   merger maps 1:1.
4. **c16's set-flag chestpiece mapping** (Q10) in the context doc.
5. **d73's re-derived solar equal-tier witness** (set absent + equal tier
   → delete at `j` itself, not `j-1`) and the ice-frame-free scope (62
   stays out).

**What the canonical module does NOT take:** c17's single-rescan collapse
(Q4 loss), d73's persistence claim (Q8), c16/d74's unadjusted delete
requests as the PRIMARY path (they survive only as the derived view),
c17's swapped chestpiece doc (Q10).

## Merger contract (f12 / keep-one step)

1. **Keep `buff_effects_orbs.rs` (this branch's) as the ONE orb module.**
   Delete the orb arms from the gathered `buff_effects_potions_sets.rs`
   (c17), `buff_effects_boosters.rs` (d73), `buff_effects_tail.rs` (d74)
   and the old `buff_effects_orbs.rs` (c16) — or drop those modules'
   orb dispatch entirely and route 95-100/170-181 here. The non-orb ids
   (94/396/192/321/62 + the summon/pet/status tails) stay with their
   modules; this module returns `NotHandled` for them.
2. **Drive it directly** (the faithful path): timer tick first
   (`buffs.rs::update_buffs_tick`), `OrbScanState::reset_effects()` once
   per frame, then per active slot (`buffType > 0 && buffTime > 0`) call
   `apply_orb_arm`, advance by `NextSlot` clamped `>= 0`. Fold
   `OrbEffects` after the scan: stats `+=`, `beetle_buff` OR; copy the
   `OrbScanState` counters onto the player fields.
3. **If a merger keeps the request-drain shape instead**, use
   `orb_arm_requests` and honour `next_index` (NOT a replay of
   `delete_buff` at the original `j`), the inclusive `purge_band` with no
   outer `j--`, and the in-place nebula writes. The direct driver is
   preferred — the scratch view cannot see cross-slot sweep deletes.
4. **Set flags**: `beetleDefense`/`beetleOffense`/`setSolar` come from the
   armor-set lane (`ApplySetBonus_BeetleDefense` `:16499`,
   `ApplySetBonus_BeetleDamage` `:16529`, `ApplySetBonus_Solar` `:16366` —
   all unported; false until they land, so the arms zero + delete exactly
   like the C# without the set).
5. **Consumers**: `nebulaLevelMana` feeds the mana-regen lane; the
   damage/speed/regen adds fold onto the live stat fields; `beetleBuff`
   feeds the beetle-orb visual lane (unported).

## Witnesses

22 `w4_g27_*` tests in the canonical module (written, NOT run —
code-only): reset-zeroing, per-tier endurance/solar keeps, the upgrade
band inclusive-of-self + no-`j--`, the downgrade rescan, the gate-failure
delete, the double-delete quirk (both slots die) + the negative-index
guard, Might no-pin + stats-before-gate + shared-counter sweep, the
nebula step-down / tier-1 guard / last-slot-wins + stacked adds / mana
level-only / damage four-class, the request-view parity, the
out-of-block pass-through, and the across-tick zero-seeding end-to-end.

## Verification

Code-only: no cargo/tests/graphify invoked (per the worker contract).
The C# facts above were re-read from `decompiled/Terraria/Player.cs`
(`:10342-10461`, `:12070-12079`, `:18501-18505`, `:18755`,
`:16366-16369`, `:16499-16532`, `:9865-9874`) and `BuffID.cs:47`; the
five implementations were diffed from their branch tips via `git show`.
The `BuffSlots` seam (`buffs.rs:163-186`, `del_buff` `:348-363` — the
`maxBuffs-1` compaction quirk) is the same one c17/d73 already use.
