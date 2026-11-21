# 26d — W4-d10 parked-seam cross-checks (1128 spore row, 536 Medusa rays)

> Branch `task/W4-d10-proj-ai-91-100`, 2026-11-21. Code-only round: the
> witnesses are WRITTEN NOT RUN. These are the keep-one / composition
> verdicts for the two seams the d10 style ports parked against sibling
> lanes (g2's rowless-child census, d11's aiStyle-101 arm).

## 1. The 1128 spore row — KEEP-ONE (d10 owns it, g2 declines)

**Verdict: no conflict, no duplicate row. d10's row is the one row.**

- g2 (`task/W4-g2-proj-ai-missing-b`) censused 1128
  (ChlorophyteClaymoreSporeCloud) as a **rowless child** in `8ac7d95e` /
  `docs/research/26d-projectile-rowless-kinds-w4-g2.md`: its 229
  (Chlorophyte orb) Kill burst spawns 32 children of kind 1128 with all 64
  `Main.rand` draws KEPT in order (X `Next(-25,26)` then Y `Next(-45,46)`
  per child), but the spawn **declines at the `proj_data` guard** until the
  row lands. g2's reconciliation phase 4 (`81c4ab85`) records the child as
  **"SKIPPED, owned by W4-d10"** — it explicitly names d10's `b34ad73c` as
  the row's owner and keeps its own draws stream-neutral.
- d10's 92-arm (`b34ad73c`) **rowed 1128**: `projectile/data.rs`
  `SPORE_CLOUD = 1128` (`:727`), the row `:1746-1749`
  (`Projectile.cs:10565-10579`: 30×30, aiStyle 92, melee, penetrate −1,
  scale 1.1, tileCollide false, the idStatic-30 pair), `proj_frames(1128)
  = 5` (`:4786`), the melee-class flag (`:4464`), and the aiStyle-92 cloud
  family arm (`projectile_ai_91_100.rs`, the spore width-falloff separation
  + the `IsWorldPointSolid` reflection ladder + the first-tick `Next(5)`
  frame roll).
- **Distinct kinds, no collision:** g2's `CHLOROPHYTE_SPORE_CLOUD = 228`
  (`data.rs:696`, aiStyle **44**, the orb) is a DIFFERENT kind from d10's
  `SPORE_CLOUD = 1128` (aiStyle **92**, the claymore's spore cloud). The two
  consts coexist; neither branch rows the other's kind.
- **At merge:** g2's 229 Kill-burst spawn (`spawn_with_data(1128, …)`)
  resolves against d10's row and the 32 children spawn; the 64 draws g2
  kept stay aligned (d10's 1128 arm draws only the first-tick `Next(5)`
  frame roll per child, the C# `:32565` — the child's own stream position,
  not the parent burst's). No re-port, no keep-one deletion needed.

## 2. The 536 Medusa rays — COMPOSITION VERIFIED (d10 spawn ⨉ d11 row)

**Verdict: d10's 535 spawn-request composes with d11's 536 row + 101 arm.
One merge conflict to reconcile: the shared `MEDUSA_HEAD_RAY` const.**

- d11 (`task/W4-d11-proj-ai-101-110`) landed the 101 arm in `6aca9b7f`
  (census `e0395afc`): the 536/591 rows + `ai_medusa_ray`
  (`projectile_ai_101_110.rs`) + the dispatch arm. The 536 row
  (`Projectile.cs:6107-6119`: 10×10, aiStyle 101, friendly, penetrate −1,
  tileCollide false, magic, ignoreWater; drawLayer 7 / usesOwnerLight
  render-side dropped) matches the C# verbatim.
- d10's 100-arm (`d448013c`, `ai_medusa_head` + the dispatch) spawns the
  rays through `spawn_with_data_ai_owned(MEDUSA_HEAD_RAY, owner, hx−5,
  hy−5, v, damage, kb, f32::from_bits(key), 0)`:
  - **The key link:** `ai0 = this.key` (`Projectile.cs:44591`/`:44604`)
    rides as the raw bit pattern (`key_of(i)` → `from_bits`); d11's arm
    probes `key_parent(ai[0], 535)` (`Projectile.cs:33211`'s
    `((ProjectileKey)ai[0]).TryGetActive(535)`). d10's 535 row (aiStyle 100)
    is the expected parent kind — the probe resolves once BOTH branches
    land. Until then d11's probe reads None and the ray dies exactly as the
    C#'s failed `TryGetActive` (d11's census records this dependency).
  - **The strike walk:** d10 spawns the ray at the head centre, jumps its
    Centre onto the NPC, sets `damage = num3`, runs `damage_pve_boxed`,
    zeroes the damage, jumps back (`:44592-44596`). d11's arm then anchors
    `Center = parent.Center − velocity` each tick (`:33216`) + the generic
    `kill_step` add — the ray hangs off the head, the 20-tick
    `(int)Lerp(0,255,t/20)` fade (`:33169`) then the `localAI[0] >= 20` kill
    (`:33211`). The C# ray's `timeLeft` is the row default; the fade kill
    collects it at 20 ticks. Composition holds.
  - **The scatter rays:** d10 rolls the 4 velocities (`Next(4)` +
    `RotatedByRandom` pair + `NextFloat`, `:44603`) and spawns them at
    damage 0 (`:44604` passes 0); d11's arm anchors them to the head the
    same way. The rolls stay in d10's arm (the C# evaluates the velocity
    expression BEFORE `NewProjectile`), so the stream position holds
    regardless of the row's presence.
- **The merge conflict (keep-one) — EXECUTED on d10:** BOTH branches
  added `pub const MEDUSA_HEAD_RAY: u16 = 536` to `projectile/data.rs` —
  d10 (the dependency note, no row), d11 in its 101 commit (with the
  row). Neither base (`a725acb0`) had the const. The drop-hunk has now
  RUN on d10: the duplicate const is REMOVED, the 535 dispatch spawns
  through the **literal 536** (`spawn_with_data_ai_owned(536, …)` — both
  the strike and the scatter arms), so d10 compiles standalone (the
  no-row spawn still declines) and the merge takes d11's const as the
  one definition with NO conflict. The s100 row witness
  (`w4_d10_s100_the_medusa_row_is_verbatim`) is RE-PINNED to d11's landed
  row shape via the literal: 10×10, aiStyle 101, penetrate −1,
  tileCollide false, magic, ignoreWater, timeLeft 3600 — a POST-MERGE
  pin (WRITTEN NOT RUN in code-only mode; it reads `is_none()`-shaped
  failure on this branch until d11's row lands, which is the documented
  state).
- **Cross-note:** d10's module doc + the `MEDUSA_HEAD_RAY` const doc record
  the dependency ("the row + AI are d11's claim — until they land the
  spawns decline"). After the merge the spawns resolve and the 535 deals
  damage through the strike rays; the scatter rays stay damage-0 visuals.

## Witnesses

These are census/composition verdicts, not new behaviour — no new witness
tests. The d10 s100 row witness
(`w4_d10_s100_the_medusa_row_is_verbatim`) is re-pinned to d11's landed
536 row shape via the literal (the drop-hunk above); d11's 101 witnesses
cover the ray arm. The pin is WRITTEN NOT RUN (code-only) and asserts
the post-merge state.