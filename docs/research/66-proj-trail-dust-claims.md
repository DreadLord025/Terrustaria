# 66 — Projectile TRAIL dust per-kind lane (W4-g63 / W4-g64 / W4-g65): claims + census

Status: CLAIM doc (W4-g63, lane 1 of the per-kind trail split). Written early
per the lane's coordination rule — the branch list is the claim record; this
doc pins the kind-range split so the lanes never overlap.

## Scope

The C# `Projectile.AI()` per-kind **trailing / ambient dust** arms — the
`Dust.NewDust*` sites inside the AI machines (NOT `Kill()`, NOT the
draw-path, NOT the site-batch lanes). Co-lanes read the pushed claims:

- **W4-d53 / W4-e3 / W4-f5** own the projectile dust SITE batches (the
  per-file line-order census). At claim time their branches carry no
  commits beyond `a725acb0` / d50's `2e243a2d` (f5 = d50 tip, in-flight
  probe 2026-11-21) — no site claim collides with the per-kind arms below.
- **W4-g63 `task/W4-g63-proj-trail-dust-1`** — kinds **1-400**, this doc.
- **W4-g64 `task/W4-g64-proj-trail-dust-2`** — kinds **401+**.
- **W4-g65 `task/W4-g65-proj-death-dust`** — the `Kill()` death-dust arms
  (all kinds).

Boundary kinds are owned whole by range: 400 is g63's, 401 is g64's. A
shared style-level block (e.g. aiStyle 8's default 2×dust-6 arm,
`Projectile.cs:24326-24336`) belongs to the lane whose kinds reach it;
where a block serves kinds on BOTH sides of 400 (aiStyle 5's
`ai[1]==1||type==92` arm serves 9/92/955), g63 ports the block gated to
its own kinds and cites the g64 remainder.

## Pool + RNG contract (the co-lane rule)

- Pool: `origin/task/W4-d50-dust-type-table` —
  `tr_core::particles::DustPool::new_dust` (`Dust.cs:75-256` verbatim),
  `new_dust_perfect` (`:54-63`), `new_dust_direct` (`:65-73`). d50's tip
  `2e243a2d` leaves `particles.rs` byte-identical to main (verified
  `git diff` empty), so this lane consumes the contract from main.
- RNG: one slot per C# pass — every draw rides the **UpdateProjectiles**
  stream (`Main.cs:17996-18022` `UpdateWorld_Projectiles`, the
  `SwapRandom("UpdateProjectiles")` wrap at `Main.cs:17775-17777`), in
  C# statement order: the explicit `Main.rand` argument draws FIRST
  (C# evaluates arguments before the call), then the `NewDust` internals
  inside `DustPool::new_dust`, then the post-spawn writes (which land on
  the sentinel slot 6000 when the pool refuses — the d52/e2 convention).
- Gore siblings between dust draws: their ARGUMENT draws are consumed in
  order here (the `Next(61,64)` type rolls etc.); the `NewGore` bodies
  stay the gore lane's (W4-d55 precedent, the e2 split).
- The port's earlier "dust SKIPPED — the roll skips with it" entries are
  RE-ARMED by this lane: the gate rolls return to the stream at their
  C# positions (the S1-F-13c/d52 precedent). Every re-arm is cited at
  its machine.

## Census (kinds 1-400, AI()-region sites)

`decompiled/Terraria/Projectile.cs` carries 1195 `Dust.NewDust*` sites;
233 sit inside `AI()` (`:23199-36807`) and 121 inside `AI_001`
(`:54002-57825`). Kind attribution = the C# `SetDefaults` aiStyle row
(`:697-10920`) + the nearest enclosing `type ==` gate. The per-kind
gated kinds ≤400 in `AI()` proper:

| style | kinds (≤400, per-kind gated dust) | sites |
| ----- | --------------------------------- | ----- |
| 2  | 48/54/93 (`:23687` shared gate), 162 (`:23296/23302/23306`), 240 (`:23386/23392/23396`), 306 (`:23618`) | thrown family |
| 4  | 151 (`:23790`, kind-gated head) | vilethorn tip |
| 5  | 9 (`:24041-24048`), 12 (`:23985`), 92 (the `:24051-24061` shared arm) | falling stars |
| 8  | 27 (`:24261-24296`), 95/96 (`:24311`), 253 (`:24318/24328`) + the default 2×6 arm (`:24328`, kinds 15/258 ≤400) | bouncing casters |
| 11 | 87 (`:24406`, the fairy dust-type picker 56/73/74) | fairies |
| 12 | 22 (`:24577/24601/24615/24625`), 288 (`:24527/24537`, the 280/288 shared head) | aqua/golden streams |
| 14 | 196 (`:24903`), 261 (`:25032/25038`) | darts/spears |
| 21 | 77 (the style arm `:25596`, kinds 76/77/78 all ≤400) | music notes |
| 22 | 80 (the style arm `:25665/25676/25683/25756`) | ice block |
| 23 | 101 (`:25900`, the 101/188 shared arm) | eye fire |
| 24 | 94 (`:25960`) | crystal shard |
| 27 | 114 (`:26451/26465*`/26472*/26482*/26484*/26496), 115 (`:26465/26496`), 116 (`:26501`), 132/173 (`:26472-26486`, ≤400) | trident/beam family |
| 28 | 118 (`:26615`), 119/128 (`:26624`), 129 (`:26642`), 177 (`:26606`), 309 (`:26633`), 359 (the `:26624` triple gate) | frost bolt family |
| 49 | 281 (`:23341/23347/23351` — the C# aiStyle-2 `type==281` twin arm; SetDefaults row 49 is the 1.4.5 remap, the AI arm is the type-gated 2-block) | cannon bunny |
| 50 | 291 (`:28343`), 295 (`:28354`), 296 (`:28392`) | inferno fork |
| 51 | 297 (`:28411`), 293 (the else arm `:28426`) | spectre staff |
| 52 | 298/305 (`:28539/28552`) | rainbow |
| 30 | 335 (`:26754`), 907 (`:26689`, >400 → g64) | gem sparkles |
| 60 | 358 (`:29167/29191/29205/29229/29245/29284`) | last prism |
| 36 | 307 (`:27327`), 181/189/316 (style-shared) | bees/bats |
| 65/66/68 | 385/387/388/399 | hostile shots |

`AI_001` per-kind gated kinds ≤400 (20): 41, 55, 120, 174, 176, 207,
265, 267, 270, 275, 276, 282, 299, 325, 337, 350, 355, 357, 374, 376.

Named-fn machines with per-kind dust ≤400 (`AI_003` boomerangs,
`AI_015_Flails_Dust`, `AI_016_Bombs`, `AI_019_Spears`, `AI_026`,
`AI_029_GemStaffs`, `AI_061` bobber splash, `AI_062` minions, `AI_075`,
`AI_099` yoyos, `AI_113`, `AI_130/131`, `AI_135-137`, `AI_140-145`,
`AI_147`, `AI_155-157`, `AI_165` whips, `AI_190/191/192/193`, …) ride
the same kind-range rule; the batch ledger below records each as it
lands.

## Batch ledger (g63, kinds ascending, 10 kinds per commit)

- batch 1: 9, 15, 22, 27, 54, 77, 80, 92, 94, 95 — the rowed
  inline-machine kinds (styles 5/8/12/2/21/22/24).
- (following batches recorded in their commit messages + this table)

Kinds with NO ported `proj_data` row keep their dust arm UNWIRED (the
row is the wiring gate — the S1-F-9b convention); their emitters land
cited + witnessed, the row/dispatch stays the AI lane's remainder.
