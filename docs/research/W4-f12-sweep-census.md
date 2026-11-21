# W4-f12 sweep — the FULL UpdateBuffs stat-arm census (2026-11-21)

Branch `task/W4-f12-buff-frame-wiring` (sweep phase, after 6351c615).
Script-diff of the C# `Player.UpdateBuffs` case-list (`Player.cs:9865-12045`)
against EVERY ported ownership set. Code-only: witnesses written, not run.

## The orb_scan nebula twin — f12-side contract (sweep gap 3)

f49b (`origin/task/W4-f49-mana-regen-2`, tip 5fbbc430) landed the
`:12070-12079` tier writer as `Player::update_buffs_nebula_buffs` on the
d117 `Player.nebula_level_mana` field (the regen engine's read side) and
published the one-writer recipe in `docs/research/85-doc-sweep-mana.md`:
*"the merge must keep ONE writer feeding ONE field (the regen read side
reads `Player.nebula_level_mana`)"*; the Life/Damage consumers
(`lifeRegen += 6 × tier` `:10451`, the `0.15 × tier` four-class adds
`:10456-10460`) stay f12's — confirmed, they are live here in
`buff_effects_orbs.rs::apply_orb_arm` (`:306-322`).

**Writer-semantics diff (verified line-by-line):** the two writers are
SEMANTICALLY IDENTICAL — both ASSIGN `level = 1 + buffType[b] − base`
(`:12072`, no stale-tier survival), both step down on `buffTime == 2 &&
level > 1` (`level--`, `buffType[b]--`, `buffTime = 480`, `:12073-12078`),
both read the POST-tick-head timer. Ours:
`buff_effects_orbs.rs::nebula_buffs` writing `OrbScanState.
nebula_level_mana` (base 176, `:303`), zero-seeded every tick by
`orb_scan.reset_effects()` (the `:18755` chained clear, the c17
correction). Theirs: `player.rs::update_buffs_nebula_buffs` returning the
level onto `Player.nebula_level_mana`, chained ResetEffects clear.

**f12-side contract (published for the merger):** keep ONE writer — the
canonical g27 `apply_orb_arm` (the wiring routes the 18 orb ids to it
EXCLUSIVELY; f49's scan arms are a second dispatch of the same C# block).
At the merge, either (a) the regen read side reads
`p.orb_scan.nebula_level_mana` (the g27 field, already ResetEffects-zeroed
in the same chained statement), or (b) the `OrbScanState.nebula_level_mana`
write is retargeted onto the d117 `Player.nebula_level_mana` field — one
writer, one field, either binding is faithful because the semantics are
identical. Do NOT keep both dispatches: f49's `update_buffs_nebula_buffs`
call sites fold out when the wiring lands (its pickup/consume arms
`:35045-35098` + `NebulaLevelup` `:57030-57059` are NOT duplicated here —
they stay f49's). No code change on this side: the g27 module was adopted
byte-exact per the phase-1 merger contract; this note IS the alignment.

## The buff-321 co-check — keep-one note (sweep gap 4)

d115b (`origin/task/W4-d115-dodge-family`, 6c5a1c57b inside tip 7c4fcb75)
landed an INLINE 321 arm in `player.rs`'s effect match (+10 melee/ranged/
magic crit, +0.1 minionDamage). THIS branch already owns 321 through the
d73 booster lane: `buff_effects_boosters.rs::brain_of_confusion_arm`
(`:160-166`) dispatched by the wiring at the C# chain position
(`buff_frame_wiring.rs` `matches!(buff, 192 | 321 | 396 | 62)`).

**Both cite `Player.cs:9940-9947` — re-verified against the source:**
`int num = 10; meleeCrit += num; rangedCrit += num; magicCrit += num;
minionDamage += (float)num / 100f`. The VALUES ARE IDENTICAL in both
ports (d115 inline `+= 10`/`+= 0.1`; d73 `num = 10`, `num as f32 / 100.0`).

**Keep-one ruling for the merger:** the d73/wiring arm is canonical — it
sits at the exact C# slot-chain position inside the ONE UpdateBuffs pass
(the wiring IS the frame), while d115's inline copy runs in the separate
`update_with_grapple` effect match; keeping both double-grants (+20 crit,
+0.2 minion). At the merge, d115's inline 321 arm folds out (its dodge
GRANT `AddBuff(321,240)` `:38052` and the retaliate census stay d115's —
only the stat arm is duplicated). If the merger prefers the inline home
instead, 321 must move to INLINE_OWNED and drop out of the wiring's
booster match — never both. The existing witness
`w4_f12_brain_buff_crits_and_minion_damage` pins the single-grant values
(4+10 crit ×3, minion +0.1). No code change on this side.

## The C# arm inventory

- **256 `==`-style arm heads + 6 range heads = 262 heads** in the slot loop
  (`:9875-12044`), covering **270 distinct buff ids** (252 explicit `==` ids
  — the fairy trio head `:11268` carries 27/101/102 on one line — plus the
  18 lunar-orb ids in the six range heads `:10342/:10377/:10408/:10444/
  :10448/:10453`).
- The `BuffID.Sets.MountType` head (`:9881`, `BuffID.cs:169`, 66 rows) owns
  every mount buff id not in the explicit list (90/128-132/141-143/162/193/
  212/220-255 odd rows/269/272/275-283/318/338/342/346/370/374/377-381/384/
  387/388/391/392); the port's `mount.rs::MOUNTS` (66 rows) +
  `mount_type_of_buff` covers the set, the tick head re-affirms
  (`player.rs:5892-5903`, `:9881-9884`), and the five swim sub-arms
  (131/168/265/279/305) are inline-owned.

## The ownership sets diffed (the port side)

| set | home | ids |
| --- | --- | --- |
| INLINE_OWNED | `player.rs` effect match (`:6683-7000` region) + `buff_frame_wiring.rs` registry | 61 → **62** (sweep adds 30) |
| ACCESSORY_OWNED | `buff_effects_accessory.rs` (the live seam pass) | 79 |
| SUMMON_OWNED | `player.rs::SUMMON_BUFF_ARMS` + cap rows + 216 respawn | 32 |
| DEBUFF_DISPATCH | wave B `buff_effects.rs` via the wiring | 23 → **24** (sweep adds 156) |
| MISC_DISPATCH | wave D `buff_effects_misc.rs` DoT flags | 11 |
| wave-E status | `buff_effects_status.rs` (20 STATUS_PET_ARMS + 17 match arms) | 37 |
| d73 non-orb | `buff_effects_boosters.rs` 192/321/396/62 | 4 |
| canonical orbs | g27 `buff_effects_orbs.rs::apply_orb_arm` (keep-one) | 18 |
| other lanes | Wet 103 (`refresh_wet_state`), Shine 11 (`entity_lights::shine_light`), Archery 16 (`world_sim::grant_ammo_mastery`), Regen 2 + Dryad-regen (`accessory_life_regen` lane) | 4 |

## The census result — exactly 10 gaps (ALL closed by this sweep)

**Whole ids with NO owner anywhere (2):**

1. **Bleeding 30** (`:10029-10032`, `bleed = true`) — no inline arm, no
   dispatch set, no module arm. The `derive_dot_inputs` `holding(30)` twin
   read the slot directly, but `Player.bleed` never armed (Obstructed 163
   was its only writer). **Fix: inline arm + INLINE_OWNED 30.**
2. **Stoned 156** (`:10057-10060`, `stoned = true`) — the wave-B module arm
   existed (`buff_effects.rs:127`) but 156 was in NO dispatch set, so it
   never fired; `CCed` only saw the raw slot. **Fix: DEBUFF_DISPATCH 156.**

**Partial arms — the id is owned but stat halves were missing (8):**

3. **Builder 107** (`:10158-10163`) — `blockRange++` (`:10162`) missing;
   tile/wall speed halves were live. **Fix: inline `block_range += 1`.**
4. **Dryad's Ward 165** (`:10303-10312`) — `dryadWard = true` (`:10307`)
   missing; defense/thorns inline + regen in the accessory lane were live.
   **Fix: inline flag.**
5. **Weak 33** (`:11911-11917`) — `meleeSpeed -= 0.051` (`:11914`) missing.
   **Fix: inline `melee_speed_bonus -= 0.051`.**
6. **WellFed 26** (`:11931-11946`) — `minionDamage += 0.05` (`:11942`) +
   `minionKB += 0.5` (`:11943`) missing ("waits for M4-MINION" — the fields
   are live now). **Fix: inline pair.**
7. **Plenty 206** (`:11947-11962`) — `minionDamage += 0.075` (`:11958`) +
   `minionKB += 0.75` (`:11959`). **Fix: inline pair.**
8. **Exquisitely 207** (`:11963-11978`) — `minionDamage += 0.1` (`:11974`)
   + `minionKB++` (`:11975`). **Fix: inline pair.**
9. **Kite 383** (`:9914-9926`) — the whole stat block missing: `moveSpeed
   += 0.15` (`:9916`), `pickSpeed -= 0.1` (`:9917`), `tileSpeed += 0.15`
   (`:9918`), `wallSpeed += 0.15` (`:9919`); and the tier assign fired
   every tick instead of the C# ON-CHANGE gate with `luckNeedsSync = true`
   (`:9921-9925`). **Fix: inline stats + change-gated sync.**
10. **Ichor 69** (`:10061-10065`) — `ichor = true` (`:10063`) missing: the
    inline arm carried only the −15 defense, and 69 is inline-owned so the
    wave-B module's flag copy never dispatches — `Player.ichor` had NO
    live writer. **Fix: inline flag beside the defense.**

After the sweep the set-diff is **empty**: every one of the 270 ids has
exactly one live grant path (the mount set rides the mount head). The
INLINE_OWNED registry now records 30 explicitly (the Tipsy-25 pattern).

**Verified NOT gaps** (checked against the C# line-by-line): Tipsy 25
(gap-1 pass complete), Sugar Rush 192 / Brain 321 / Mana Heat 396 / Ice
Barrier 62 (d73 wiring), Magic Power 29 (`mana_max` IS `statManaMax2`),
the fairy trio 27/101/102 (one head), Rabid 148 (accessory-owned RNG
triple), Webbed 149 (inline flag + hook removal; the movement lockdown
halves ride the control-lock + velocity-zero seams, the misc module's
full arm is not dispatched), Hungry/Starving 333/334 (wave E), the DoT
flag family (MISC_DISPATCH), Shine 11 / Archery 16 / Wet 103 / Regen 2
(other lanes).

## Witnesses (written, not run — code-only)

`player.rs`: `w4_f12_sweep_bleeding_arms_the_flag_inline`,
`w4_f12_sweep_builder_potion_adds_block_range`,
`w4_f12_sweep_dryads_ward_arms_the_flag`,
`w4_f12_sweep_weak_subtracts_melee_speed`,
`w4_f12_sweep_ichor_arms_the_flag_inline`,
`w4_f12_sweep_fed_tiers_grant_the_minion_halves`,
`w4_f12_sweep_kite_stats_and_change_gated_sync`.
`buff_frame_wiring.rs`: `w4_f12_sweep_stoned_dispatches_through_the_debuff_lane`.