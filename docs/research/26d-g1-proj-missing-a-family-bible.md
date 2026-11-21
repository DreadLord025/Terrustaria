# 26d-g1 — the proj-missing-a family bible (W4-g1, lane a) — FINAL

> Written 2026-11-21 at the g1 SEAL round (branch
> `task/W4-g1-proj-ai-missing-a`, worktree `TR-W4-g1`, code-only mode —
> no cargo/rustc/tests/rustfmt/graphify invoked). This is the lane's
> single family ledger: what the lane claimed, what landed, what the
> cross-lane adjudications moved, and the exact merge instructions.
> Companion docs: `26d-projectile-kill-hooks-census.md` (the census,
> SEALED), d78's `26f-d78-final-kill-dedupe.md` (the adjudication,
> tip `fd59c51a`), f20's `26f-projectile-ai001-overlays-round2.md` F3
> (the 709 half, tip `01e0b1083`), g65's `26i-g65-kill-keep-one.md`
> (the dust-body keep-one, tip `b0842907c`).

## 1. The lane's history (hashes)

| commit | what |
| --- | --- |
| `5bd68ce2` | the census push — research/26d only; claimed 452/454/459/521/448/80 (the six functional Kill hooks missing for rowed+armed kinds) |
| `e43c62879` | the DONE tip — the six hooks + the 522 row/arm/bounce landed whole; the 459 stream-misorder FIXED (resize → Damage → dust-rings, the C# `:73351` order — the WIP's draws-before-Damage was a per-death stream desync); the 522 Kill census ADDED (`:73271-73290`, 10 `Next(3)` gates + branch draws — the WIP skipped it); the 522 step kill uses `kill_time_left` + the `active` gate (the `:17027-17030` movement skip); the pool-burst witnesses spawn OFFSET from the player (centre spawns let the standing contact fire first). 18 witnesses WRITTEN NOT RUN (its message says 22 — the count was wrong; 10 module + 8 pool at that tip) |
| this SEAL round | the d78-fold + the f20 gate widening + this bible (see §2-§4) |

The adjudication inputs (read-only probes of the sibling tips):

- **d78 FINAL `fd59c51a`** (`task/W4-d78-ui-sounds-menus`), 26f §1:
  d78 KEEPS the 452/454/448 EVP trio (`hostile_death_blast` +
  `damage_evp_boxed`, the verbatim `Damage_EVP` `:14773-14900` port with
  DamageVar/hostileDamageScaling/×2/BossNoCheese through
  `hurt_inner_cooldown`); g1's port was probed LOCAL-ONLY at that time
  (it has since been committed as `e43c62879` — the adjudication's
  arm-by-arm reasoning is unchanged: g1's EVP hit was RAW damage and its
  General immune gate would refuse the boss blasts the C# `:14799`
  lands). g1 KEEPS its unique 459/521/522/80.
- **f20 F3 `01e0b1083`** (`task/W4-f20-proj-ai-verify-2`): the 709-only
  half `:73372-73388` + the 1110 row/arm (`sky_dragons_fury_kill`).
  The 459/709 SHARED burst `:73338-73371` stays g1's claim.
- **g65 keep-one (26i @ `b0842907c`)**: the dust/gore BODIES for
  452/454/448 (and 459|709 / 521 / 522 / 80) are pushed; g1's standalone
  trio census DROPS (26f §1 — keeping both double-rolls the stream).

## 2. FINAL ownership — the family ledger

| piece | C# site | owner | g1 state |
| --- | --- | --- | --- |
| 459/709 SHARED burst: scale≥1 Resize-144 → `Damage()` → 7+30 ring dusts + 2 gores; below 1: 3+10 dusts, 0 gores | `Projectile.cs:73338-73371` | **g1** | KEPT — `charged_blaster_orb_kill` (resize+flag) / `charged_blaster_orb_kill_draws` (census) split so the pool's `damage_pve_boxed` sits BETWEEN (the `:73351` order). Pool gate `CHARGED_BLASTER_ORB \| MONK_STAFF_T3_ALT_SHOT_KIND` |
| 709-only half: the 80×80 sibling 1110 pull-in + centre spawn + `30·Next(2,6)` life; the 1110 row | `:73372-73388` / `:10340-10352` | **f20** (`01e0b1083`) | NOT g1's — kind-disjoint `sky_dragons_fury_kill` |
| the 709 ROW | `ProjectileID.MonkStaffT3AltShot` | **d15** | row-gated inert on g1/f20 (`proj_data(709)` None; both carry a byte-identical local const) |
| 521 Kill: 20-dust census + owner-gated `Next(3,6)` 522 split (state-bearing velocity draws feed the spawns inline) | `:73292-73327` | **g1** | KEPT — `crystal_pulse_kill` + the pool spawn loop (corpse-flag lift, centre→top-left −4) |
| the 521 arm's pre-move `oldPosition` latch | `Update` `:17013` | **g1** | KEPT — the port's standing latch is golf-only (`:19798`), so the 521 arm carries its own pre-move latch at the C# `Update` position (between the AI and HandleMovement), which the split's `:73324` spawn centre reads |
| 522 row (8×8 aiStyle29 alpha255 penetrate1 magic) + AI_029 arm (ai1>40 kill, +0.2 gravity clamp 18, ×0.98 drag) + the full ±1.0 reflection bounce (no penetrate spend) | `:5919-5928` / `:37742-37773` / `:18796-18806` | **g1** | KEPT — the F3 row+arm+hook-together landing; `crystal_pulse2_step` + `crystal_pulse2_move` (gated on `active`, the `:17027-17030` kill-tick skip) |
| 522 Kill: the 10-dust census (10 `Next(3)` gates + branch draws) | `:73271-73290` | **g1** | KEPT — `crystal_pulse2_kill_draws` (fires for every 522 corpse) |
| 80 Kill: ai0≥0 census + the tile-127 break | `:75533-75553` | **g1** | KEPT — `ice_block_kill_draws` (rolls nothing) + the pool's `kill_tile` seam (active-127 gate) |
| 452 Kill: Resize 144→gore→Resize 30 + EVP `Damage()` | `:72987-73013` | **d78** state+EVP; **g65** body; **d77** cue | DROPPED from g1 (`phantasmal_eye_kill` + its census + the pool arm + its 2 witnesses) |
| 454 Kill: recentre 208 + rings/gore + EVP `Damage()` | `:73014-73045` | **d78** / **g65** / **d77** | DROPPED from g1 (`phantasmal_sphere_kill` + …) |
| 448 Kill: recentre 112 + gore + EVP `Damage()` | `:73603-73631` | **d78** / **g65** / **d77** | DROPPED from g1 (`saucer_missile_kill` + …) |
| the raw-EVP player-hit machinery | `Damage_EVP` stand-in | **d78** (`damage_evp_boxed` verbatim) | DROPPED — `death_burst_player_hit` + `hostile_burst_colliding` + the 3 dispatcher arms + the gate witness |
| the deferred unrowed set (467/468, 483/484, 117→1124, 502, 510, 358/406/1042, 171/475/505/506) | various | the F3/F5 row+arm lanes | unchanged — census §"Deferred" (117's 1124 child row landed on d78 2b; the hook still defers with its rider) |
| the ~150 dust/sound-only Kill arms | various | g63-65 dust / g16 gore / g21-25+d75-77 sound | unchanged — never g1's |

## 3. The SEAL-round diff (what changed on this branch)

1. **The d78-fold DROP** (`projectile_kill_hooks_a.rs` +
   `projectile.rs`): the three hostile hooks + their censuses,
   `hostile_burst_colliding`, the pool's `death_burst_player_hit`, the
   three dispatcher arms and the 7 hostile witnesses (4 module + 3 pool)
   are REMOVED. Drop-marker comments cite 26f §1 @ `fd59c51a` at both
   former sites. The `PHANTASMAL_*`/`SAUCER_MISSILE` consts stay (other
   main callers). No raw-damage EVP code remains in g1.
2. **The f20 gate widening EXECUTED** (was "at the merge"): the pool arm
   is `CHARGED_BLASTER_ORB | MONK_STAFF_T3_ALT_SHOT_KIND`; the local
   `const MONK_STAFF_T3_ALT_SHOT_KIND: u16 = 709;` sits at the SAME file
   position as f20's (after `AI_STYLE_1_RIDERS`), same name, same value —
   keep-one at the g1×f20 merge is a trivial dedupe. Row-gated inert
   (d15 owns the row).
3. **One NEW witness**: `w4_g1a_the_shared_burst_gate_widens_to_709`
   (pool) — hand-built 709 corpses (the f20 precedent) drive
   `kill_hooks_a` directly: scale 1.6 → 144×144 centre-kept resize + 74
   doubles + 14 ranges; scale 0.85 → box unchanged + 26 doubles + 0
   ranges. WRITTEN NOT RUN.
4. **The g65 census keep-one note** (module header + census doc): g65's
   pushed bodies carry the same explicit draws for 459|709/521/522/80 —
   at the g1×g65 merge the g1 census CALLS drop per-kind (26i §3
   pattern); g1's STATE stays. Not executed here (g65's module is not on
   this branch; dropping the censuses now would desync this branch's
   stream).
5. **Docs**: 26d SEALED (banner + final-owner table + the drop
   deviation), this bible, the INDEX rows, the anchors row corrected.

**Witness ledger (final): 12 WRITTEN NOT RUN** — 6 module
(`projectile_kill_hooks_a.rs`: the 459 scale gate, the 521 split
draw-order, the 521 foreign-owner refusal, the 522 census, the 522
step kill+integrate, the 80 zero-roll) + 6 pool (`projectile.rs`
`w4_g1a_kill_hook_tests`: the 522 row, the 521 death split, the 522
count-and-die, the 459 death resize, the 80 tile break, the 709 gate
widening). The DONE tip's "22 witnesses" message overcounted (18
actual); 7 dropped + 1 added = 12.

## 4. Merge instructions (the family's cross-lane seams)

- **g1×d78**: NO conflict surface remains — g1's dropped pieces are
  gone; d78's `hostile_death_blast`/`damage_evp_boxed`/`hurt_inner_cooldown`
  land unopposed. The death-hook chains are kind-disjoint (g1's
  `kill_hooks_a` seats at the chain tail; d78's hostile call sits ahead —
  both fire once per death, order unobservable across disjoint kinds).
- **g1×f20**: keep-one `MONK_STAFF_T3_ALT_SHOT_KIND` (byte-identical
  name/value/position). g1's shared-burst gate + f20's
  `sky_dragons_fury_kill` are kind-disjoint halves of the same C# arm;
  BOTH fire for a 709 corpse, in the C# order (shared burst `:73338`
  first, the 709-only tail `:73372` second — f20's call seats EARLIER in
  the pool chain than g1's tail call, so at the merge the f20 call must
  move AFTER `kill_hooks_a` for a 709, or the shared-burst resize must
  precede it: the C# runs the shared half first. Pin with a stream
  witness at the merge).
- **g1×d15**: the 709 row activates the widened gate; no g1 edit owed.
- **g1×g65**: drop the g1 census CALLS per-kind (459|709 / 521 / 522 /
  80) as g65's bodies drain them (26i §3 pattern); keep g1's state +
  spawns + tile break. The 521 split's STATE-BEARING velocity draws
  (`:73314-73323`) are NOT census — they stay in `crystal_pulse_kill`
  regardless (g65's 521 body explicitly leaves the spawn burst to the
  spawn lane, `death_dust.rs:2567`).
- **g1×d77**: the 521 Item110 cue (`:73294`) + the 80 Item27 gate
  (`:75537`) are armed cue-first on d77's `kill_tail_cue` and read
  pre-hook state — compatible, no g1 edit owed (26f §5).

## 5. Standing deviations (g1's kept set)

- The 459 burst's `damage_pve_boxed` ride: the port's boxed seam has no
  player reference (class crits read 0) — the standing boxed-walk
  convention, not new.
- The 522 bounce is the port's per-axis probe (the `gem_bolt_move`
  model) of the C# full ±1.0 reflection — same blocked-axis outcome for
  axis-aligned tiles.
- Sound/dust bodies: the lane skips (d77 cues armed; g65 bodies pushed).
- Everything else is verbatim with citations (the module header's RNG
  contract).

**Verification**: code-only. No cargo/rustc/tests/rustfmt/graphify
invoked (the Lead's constraint). All C# cites re-checked against
`decompiled/Terraria/Projectile.cs` in ≤100-line windows during the
DONE round; the SEAL round re-read 26f §1/§5, 26i §2-3 and f20's
witness pattern. The sibling worktrees/tips were probed READ-ONLY.