# W4-g26 — buff coverage census + FIX: the one id no wave ported (Bleeding 30)

> Worker W4-g26, branch `task/W4-g26-buff-wiring-verify` @ main `a725acb0`.
> Code-only wave: tests WRITTEN NOT RUN (no cargo/rustc per the worker
> contract); compile-by-inspection. Census scripts + extracts live in the
> session temp dir (`g26_precise.py` / `g26_verify.py`), not the repo.

## Method (re-derived, not trusted from the in-flight claims)

1. **C# side**: re-extracted every arm head of `Player.UpdateBuffs`
   (`decompiled/Terraria/Player.cs:9855-12069`, fresh window) — the literal
   `this.buffType[j] == N` tests plus the six range arms (`95-97`, `170-172`,
   `98-100`, `176-178`, `173-175`, `179-181`). **270 distinct ids.** The
   `BuffID.Sets.MountType` arm (`:9881`) is set-membership, not literal ids;
   its membership rides `mount.rs::MOUNTS` (35/66 rows — the mount system's
   own documented partial table, not an `UpdateBuffs` arm gap). The
   `:11273` switch re-reads 101/102 (already counted).
2. **Branch side**: `git show` of each wave module at its pushed tip, then
   the ids transcribed from the ACTUAL dispatch (match arms + const tables),
   not loose regexes:

| branch | tip | module | ids |
| --- | --- | --- | --- |
| c16 | `ffffa3b4` | (0 commits past main — waves A/B landed) | via main |
| c17 | `357dabed` | `buff_effects_potions_sets.rs` | 20: 94, 396, 95-100, 170-181 |
| c18 | `cef7e5aa` | `buff_effects_summons.rs` | 30: 62 + the 29 `SUMMON_ARMS` rows |
| d71 | `315dec64` | `buff_effects_status.rs` | 37: the 20 `STATUS_PET_ARMS` + 60/80/320/353/163/164/194-198/145/43/28/333/334/336 |
| d72 | `cf96c8c1` | `buff_effects_frame.rs` | frame only: 151 head + 332/333/334 hunger ladder (no new arm ids) |
| d73 | `33c38000` | `buff_effects_boosters.rs` | 22: 396, 95-100, 170-181, 62, 192, 321 |
| d74 | `3a80d60f` | `buff_effects_tail.rs` | 20: 396, 95-100, 170-181, 62 (94 + summons witness-only) |
| g27 | `1cd7ac36` | `buff_effects_orbs.rs` (canonical keep-one) | 18: 95-100, 170-181 |

3. **Main side**: the live dispatch sites, read in full —
   `player.rs:6429-6727` inline wave-A match (61 ids) + `:6941` regen match
   (2/165) + `SUMMON_BUFF_ARMS` (29) + `buff_effects.rs::apply_debuff_effect_arm`
   (47) + `buff_effects_misc.rs` (PET_ARMS 82 + match + mount sub-arms) +
   `buff_effects_accessory.rs::update_accessory_buff_arms` (79, wired at
   `world_sim/player.rs:1190`) + `world_sim/player.rs:58` flags (16/93/112) +
   `entity_lights.rs:140` Shine (11) + `potions.rs` hunger ladder (332-334).
4. **Cross-check**: every transcribed set re-extracted mechanically from the
   code (brace-balanced match-arm scan + const-table scan) and diffed against
   the hand sets — **0 diffs across all 12 sources** (`g26_verify.py`).

## Result

- C# ids: **270**. Branch union (c17/c18/d71/d73/d74/g27): 89. Main union: 248.
- Branch-only ids (the in-flight waves' payload): 95-100, 170-181, 192, 321, 396.
- **MISSING FROM ALL BRANCHES AND MAIN: `[30]` — Bleeding.**

The earlier loose-regex census (`census3/4`) reported zero missing because it
counted ANY numeric read of the id — including `player.rs:8608`'s
`self.holding(30)` in `derive_dot_inputs`. That is the buff's *consumer*
(the `lifeRegenTime = 0f` ramp reset, `Player.cs:19178-19181`), folded into
the DoT derivation per the port's convention — NOT the `UpdateBuffs` arm
(`Player.cs:10029-10032`, `this.bleed = true`). The arm — the `bleed` field
grant — existed nowhere: wave A's 18 arms skip it (it sits between 18 and 31
in the chain), wave B's block starts at 46, and no later wave's range covers
`:10029`. The C# field's full consumer census (`Player.cs` grep `\bbleed\b`):
declaration `:2362`, writers `:10031` (buff 30) + `:11829` (Obstructed 163),
clear `:18694`, sole reader `:19178`. The 163 half is already in the misc
table (`e.bleed`) and d71's status table; only the 30 arm was unported.

## The FIX (verbatim port, wave-A seam)

`crates/tr-app/src/player.rs` (+45/−2):
- `pub bleed: bool` field beside the wave-A flag family (citation
  `Player.cs:2362`), `bleed: false` in `spawn`'s literal (the only full
  `Player` construction; every other site uses `spawn`).
- `30 => self.bleed = true` in the inline buff match, in C# chain order
  between 18 (`:10025-10028`) and 31 (`:10033-10036`) — citation
  `:10029-10032`.
- `self.bleed = false` in the ResetEffects seam beside the confused/silence
  clears (`:18694`), the citation comment extended.
- The ramp-reset consumer stays in `derive_dot_inputs`'s `holding(30)` fold
  (the port's DoT convention — the same shape as 20/70/24/…; switching it to
  the field would break the isolated `update_survival` drain rigs, which do
  not run the buff scan). No behaviour change: the fold already produced the
  exact C# effect; the arm adds the C# field grant the 163 writer shares.
- Witness `w4_g26_bleeding_30_arms_the_flag_and_resets_the_regen_ramp`:
  spawn-clear → `add_buff(30, 600)` + one tick → flag set; `resets_time`
  true / `regen == 0` from the derivation; expiry (time 2, four ticks) →
  flag down. Written not run (worker contract).
- `anchors.txt` row: `crates/tr-app/src/player.rs|30 => self.bleed = true|…`.

RNG census of the arm: ZERO draws (`:10029-10032` is a bool write). No
stream, fixed-step-order or `WorldFrameKey` change. f12 owns the wave
wiring; this fix rides the LIVE wave-A scan, so it needs no wiring step.

## Overlap notes for the mergers (not this wave's problem)

- 95-100/170-181 are ported FOUR times (c17 driver, d73 boosters, d74 tail,
  g27 orbs keep-one). g27 `1cd7ac36` is the canonical orb survivor; the
  mergers keep one (c17's `update_potion_set_buff_arms` is the driver shape).
- 396 likewise in c17/d73/d74; 62 in c18/d73/d74 + the live inline endurance
  half (`player.rs:6526`, the d74 `SplitIceBarrier` note).
- 192/321 are d73-only (Sugar Rush / Brain of Confusion dodge gate); the
  live 321 consumer is `player.rs:4341`'s `find_buff_index(321)` hurt gate.
