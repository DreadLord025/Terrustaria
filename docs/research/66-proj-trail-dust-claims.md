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
  inline-machine kinds (styles 5/8/12/2/21/22/24). The shared-arm
  co-riders ≤400 land with their arm: 96 (the cursed-flame pair),
  253/258 (the caster default), 280/288 (the golden-shower head),
  76/78 (the note trio), 162/240/306 (the thrown bursts/trail),
  7/8/150/151/152 (the thorn fade-cross), 281 (the aiStyle-49 bunny
  burst emitter, UNWIRED — no row). The `update`/`update_dust`
  scratch-pool seam (the `Npc::update_rng` precedent) threads the
  live `app.dust` + `spawn_ctx` from `world_sim/projectiles.rs`.
- batch 2: 114, 115, 116, 132, 173, 293, 295, 296, 297, 309 — the
  rowed caster machines (styles 27/51/50/28). The trident family's
  `base.oldVelocity` latch lands as the slot's `old_velocity` field
  (stamped at the sub-step start, `Projectile.cs:16766`); the 114
  Next(2)+NextFloat trail, the 115 20-step latch trail, the 173
  Next(3) type-pick sparkle, the 132 2×107 pulse, the post-15 114/115
  27 + 116 64 pulse; the 293 9×175 + 297 post-4 5×175 spectre dusts;
  the 295 8×174 bolt trail + the 296 fade-window burst (3 argument +
  2 jitter draws per dust); the 309 3×185 frost trail. The 156/157
  beams ride the shared pulse (no own dust block ≤400).
- batch 3: 101, 188, 356, 385 — the rowed eye-fire / spectre-wrath /
  sharknado machines (styles 23/59/65). The 101/188 launch-ramp dust
  (the 8/9/10 scale ladder, the `num192==6 || Next(2)==0` gate — the
  188's 6 short-circuits, the 101's 75 draws — the `Next(3)!=0 ||
  (75 && Next(3)==0)` boost chain, the ×1.25/×1.5 scale, the 75's
  velocity += v); the 356 per-tick 5×175 ring (the velocity-offset
  points); the 385 post-10 6×172 ring (the NextDouble + Next(3,8)
  argument pair per dust, the rotated box-edge point, noLight).
- batch 4: 399 — the rowed molotov (style 68). The `alpha == 0` fire
  dust (flag34 = type 399): the (4,−8) spin point (X −4 on direction
  −1) rotated by the rotation, one dust 6 at Center + spin − (5,5),
  scale 1.5, noGravity, velocity×0.25 + Normalize(spin) rotated by
  −π/2·direction. The 669's flag35 two-dust block + the 387/388/533
  twins' SelectRandom(226,228,75) rings are kinds > 400 (W4-g64's —
  the :29804/:30055 blocks sit under the type-533 gate). The rowed
  kinds with NO per-kind dust ≤400 are censused closed: 48 (the
  thrown-knife arm draws nothing), 76/78 (the note trio shares 77's
  arm), 156/157 (the beams ride the shared pulse), 131 (the spore is
  RNG-free), 181/316 (the bee/bat hunt draws nothing), 387/388 (the
  twins' dust is the 533 gate).
- batch 5: 41, 120, 207, 265, 270, 282, 325, 337, 350, 355 — the ROWED
  `AI_001` per-kind arms (the contested slice modules). Each slice gains
  a `step_dust` variant threading `rng`+`dust`+`ctx`; the old signature
  stays a scratch-pool wrapper for its witnesses. The 41 Hellfire 31+6
  pair (`:55483-55489`, special_arrows), the 282 Venom 171 + Next(1,10)
  post-spawn scale (`:55003-55011`, special_arrows), the 120 Frost 67
  trail inside its gravity arm (`:57251-57253`, special_arrows), the 337
  Blizzard 1-in-2 197 sparkle (`:56949-56954`, ai_blizzard), the 350
  Missile alpha==0 window (`:55138-55157`, frost_shots), the 325
  FlamingWood twin window (`:55172-55190`, ai001_a), the 270 Skull
  first-tick 10×5 burst + per-tick 2×5 trail (`:56079-56087`/`:56678-56685`,
  ai001_a), the 265/355 fang alpha==0 163/205 dusts (`:56710-56745`,
  ai_style1_rider — the 265 fade re-armed with its dust), the 207
  Chlorophyte alpha<170 10-point 75 trail (`:56780-56793`,
  special_bullets). The 357 PulseBolt 206 twin emitter written UNWIRED
  (its flight arm is W4-f20 slice D `4bd2f496`, unmerged — the f20b dust
  seam). 11 draw-mirror witnesses WRITTEN NOT RUN.
- batch 6: 12, 87, 93, 118, 119, 128, 129, 177, 196, 229, 261, 291, 298,
  305, 307, 335, 358 — the UNROWED-kind emitters, written + witnessed
  UNWIRED (no `proj_data` row at this tip; the row is the wiring gate).
  The 93 MagicDagger 1-in-5 drip (`:23211`), the 87 fairy 1-in-6 sparkle
  (`:24395`, d2's rows), the 12 natural star gore+dust gates (`:23970`,
  d15's row), the 177 IcewaterSpit 3×137 (`:26602`), the 118 IceBolt 2×92
  (`:26611`) + the 119/128 frost trio 3×92 (`:26620`), the 129 RuneBlast
  6×106 (`:26638`), the 196 SmokeBomb Next(1,3) burst (`:24898`), the 261
  BoulderStaff dual gates (`:25017`), the 229 ChlorophyteOrb 4×157
  (`:27922`), the 307 TinyEater 3×184 (`:27310`), the 335 Ornament
  frame-colour sparkle (`:26722`), the 291 InfernoHostileBolt 10×174
  (`:28325`), the 305/298 heal-orb trails (`:28533`, d6's rows), the 358
  WaterGun 6×211 + 3 alpha gates (`:29158`, d6's row). The
  `NextVector2Square` helper (`Utils.cs:1251-1254`). 14 draw-mirror
  witnesses WRITTEN NOT RUN (the 118/119+128 pair and the 305/298 pair
  share one witness each).
- (following batches recorded in their commit messages + this table)

Kinds with NO ported `proj_data` row keep their dust arm UNWIRED (the
row is the wiring gate — the S1-F-9b convention). Batch 1 landed the
281 bunny emitter cited + witnessed as the unwired precedent; batch 6
landed the remaining unrowed per-kind emitters (12/87/93/118/119/128/
129/177/196/229/261/291/298/305/307/335/358) the same way. The 162/288
emitters were already re-armed in batch 1 (their rows exist).

### Row census (2026-11-21, the d-lane pushed tips)

The unrowed-kind emitters' rows were checked against the pushed d-lane
`data.rs` diffs (`git diff a725acb0..origin/task/W4-d*`):
- **12** NATURAL_FALLING_STAR — rowed on **d15** (`1c373008`, aiStyle 5).
- **87** GREEN_FAIRY (+72/86) — rowed on **d2** (`15e5448a`, aiStyle 11).
- **305** VAMPIRE_HEAL / **298** SPIRIT_HEAL / **358** WATER_GUN — rowed
  on **d6** (`5ae2978c`, aiStyles 52/52/60).
- **93/118/119/128/129/177/196/229/261/291/307/335** — NO row on any
  pushed tip (the 118/119/129 `proj_data` hits are ai_style false
  positives: NEBULA_ARCANUM 617 / NEBULA_ARCANUM_SUBSHOT 618 /
  SPIRIT_FLAME 659, different kinds).
- **281** CANNON_BUNNY — the const exists (`data.rs:710`) but NO match
  arm on ANY branch (`git log --all -S 'CANNON_BUNNY =>'` empty). The
  bunny emitter stays UNWIRED; the aiStyle-49 machine (`:28243-28322`)
  is also unported (no `49 =>` dispatch arm). Activation needs both the
  row AND the machine — a d-lane follow-up, not this dust lane's.

These rows live on UNMERGED d-lane branches; at this tip (main `a725acb0`)
none is present, so every batch-6 emitter is correctly UNWIRED. They
activate when the d-lanes merge (the wiring is a one-line dispatch gate
per kind, cited at each emitter).

### AI_001 remainder census (the 20 kinds ≤400)

Of the 20 `AI_001` per-kind gated kinds ≤400, batch 5 re-armed the 10
ROWED ones with dust (41/120/207/265/270/282/325/337/350/355) + the 357
emitter (UNWIRED, f20b's). The remaining 9 are UNROWED at this tip and
their emitters are the lane's continuation:
- **55** Stinger (`:55490-55494`, dust 18 noGravity) — no row.
- **174** IceSpike (`:55032-55051`, alpha==0 dust 76 at oldPosition−v*3)
  — no row.
- **176** JungleSpike (`:55109-55129`, alpha==0 dust 22 at oldPosition)
  — no row.
- **267** PoisonDart (`:57292-57306`) — ROWED, but its C# block draws NO
  dust (the fade + gravity only; the 479's split dust `:57344-57347` is
  the Ichor Dart, a different kind). Censused closed — no emitter owed.
- **275/276** SeedPlantera (`:55012-55024` frame flutter, `:57197-57231`
  gravity) — ROWED (ai001_c), but their C# blocks draw NO dust ≤400 (the
  expert homing is RNG-free). Censused closed — no emitter owed.
- **299** Shadowflames (`:56631-56656`, localAI[0]==6 40×181 burst +
  post-6 3×181 trail) — no row.
- **374** HornetStinger (`:55544-55557`, 1-in-2 dust 18) — no row.
- **376** ImpFireball (`:55558-55586`, post-3 1/2× dust 6) — no row.

The 267/275/276 closures are the batch-4 no-dust convention (the rowed
kind whose C# arm draws nothing). The 55/174/176/299/374/376 emitters
are unrowed-kind work (batch 6's shape) — they ride the d-lane rows.

## Status (2026-11-21)

The ROWED inline-machine kinds ≤400 are RE-ARMED end-to-end (batches
1-4): every kind with a `proj_data` row whose `AI()` arm carries
per-kind trailing/ambient dust now spawns through the live `DustPool`
on the `UpdateProjectiles` stream, wired from
`world_sim/projectiles.rs` via the `update_dust` seam. The no-dust
rowed kinds are censused closed (batch 4). Batch 5 re-armed the 10 rowed
`AI_001` slice-module kinds (+ the 357 emitter for f20b); batch 6 wrote
the 17 unrowed-kind emitters UNWIRED. 44 draw-mirror witnesses WRITTEN
NOT RUN (code-only, the actual `#[test]` tally): 19 (batches 1-4) + 11
(batch 5) + 14 (batch 6). The
281 bunny stays UNWIRED (no row, no machine). The unrowed `AI_001`
emitters (55/174/176/299/374/376) + the d-lane row activations are the
documented remainder above.
