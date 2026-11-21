# 70 — Projectile dust SITE batches (W4-d53 / W4-e3 / W4-f5): claims + census

Status: CLAIM + completion record (W4-d53, the FIRST ~40 lane). Written per
the lane's coordination rule — the branch list is the claim record; this doc
pins the site-range split so the three site-batch lanes never overlap each
other or the per-kind trail lanes (g63/g64) or the death lane (g65).

**RETIRE (2026-11-21):** the lane's close-out record is **research/240**
(the wetVelocity movement-gap census — 96 unclaimed integrate
expressions, 0 covered by the wet block, seed S1-P-i57; the g2
staged-copy drop-hunk consuming f20's F5 cross-note; the d53×e3 fn-set
DISJOINT confirmation at the current tips; the insurance-patch
supersede receipt; the proj-dust-family declaration). This doc's ledger
+ totals stand as the site-range authority; 240 is the retire layer.

## Scope + the split

`decompiled/Terraria/Projectile.cs` carries **1195** `Dust.NewDust*` sites.
The three SITE-batch lanes cut them by **file line order** (the per-file
census convention), NOT by kind:

- **W4-d53 `task/W4-d53-dust-sites-projectile`** — sites **1-40** (this doc,
  COMPLETE). C# lines **:16851-22297**.
- **W4-e3** (the SECOND ~40 lane; branch not yet pushed at d53's completion)
  — sites **41-80**. C# lines **:22308-24318**.
- **W4-f5 `task/W4-f5-dust-proj-3`** — sites **81-120**. C# lines
  **:24328-26472**.

Boundary sites are owned whole by range: site 40 (`:22297`) is d53's, site
41 (`:22308`) is e3's, site 80 (`:24318`) is e3's, site 81 (`:24328`) is
f5's. Sites 121+ stay unclaimed by the site lanes until the Lead extends
them.

**In-flight probe (2026-11-21):** at d53's claim time the e3 lane had no
pushed branch and f5 (`task/W4-f5-dust-proj-3`) carried no commits beyond
d50's tip `2e243a2d` — no site claim collided. d53's lane is now landed (see
the ledger below).

### Overlap with the per-kind trail lanes (g63/g64) and the death lane (g65)

The site-range cut and the per-kind cut are DIFFERENT axes, so they are
disjoint by construction for d53's range:

- d53's sites 1-40 all sit **before `AI()`** (`Projectile.cs:23199`) — they
  are the `Update` body (wet splash, arrow trail, enchantment visuals,
  `HandleMovement` impact arms, the named `AI_151/152` machines at
  `:22138`/`:22230`). g63/g64 own the `AI()`-region per-kind trail arms
  (`:23199+`) and `AI_001` (`:54002+`); g65 owns `Kill()`. **No d53 site is
  inside any of those regions**, so d53 never touches a g63/g64/g65 arm.
- e3's range (`:22308-24318`) and f5's (`:24328-26472`) DO reach into `AI()`.
  Where a site-batch site and a per-kind trail arm coincide, the **site
  batch owns the site** (it is the per-file census); the per-kind lane cites
  it as already-landed. The g63 claim doc (66) already records the
  d53/e3/f5 co-lane probe and the kind-range split; this doc is its
  site-range mirror.

## Pool + RNG contract (the co-lane rule)

- Pool: `origin/task/W4-d50-dust-type-table` —
  `tr_core::particles::DustPool::new_dust` (`Dust.cs:75-256` verbatim),
  `new_dust_perfect` (`:54-63`), `new_dust_direct` (`:65-73`), `clone_dust`
  (`:263-286`). d50's tip `2e243a2d` leaves `particles.rs` byte-identical to
  main, so the contract is consumed from main. (W4-g61 later generalised the
  `perfect/direct/clone` stream param to the `DustRng` seam — behaviour
  identical; d53 calls them with `&mut UnifiedRandom`, which implements it.)
- RNG: every draw rides the **UpdateProjectiles** stream
  (`Main.cs:17996-18022` `UpdateWorld_Projectiles`, the
  `SwapRandom("UpdateProjectiles")` wrap at `Main.cs:17775-17777`), in C#
  statement order: the explicit `Main.rand` argument draws FIRST (C#
  evaluates arguments before the call), then the `NewDust` internals inside
  `DustPool::new_dust`, then the post-spawn writes (which land on the
  sentinel slot 6000 when the pool refuses — the d52/e2 convention).
- The projectile pass rolls a `&mut dyn DropRng` and cannot borrow the App's
  live `DustPool`, so d53 uses the **queue→drain** convention (the d51
  `player_dust` / `pending_proj_sounds` precedent): the tick SNAPSHOTS the
  box/velocity/owner flags at the C# read moment into `pending_dust_bursts`,
  and `projectile_dust::drain` replays the chain on the `UpdateProjectiles`
  stream at the pass tail. The C# draws mid-walk; the port at the tail — the
  recorded S1-E-21c deviation.
- Gore siblings between dust draws: their ARGUMENT draws are consumed in
  order here (the `Next(276,283)` type roll etc.); the `NewGore` bodies stay
  the gore lane's (W4-d55, the e2 split). d53's Party imbue-7 sibling
  (`:17450-17458`) keeps every draw, cites the visual skip.

## Census + batch ledger (d53, sites 1-40, file line order)

| # | C# line | host | disposition |
| - | ------- | ---- | ----------- |
| 1-8 | 16851/16878/16891/16905/16933/16960/16973/16987 | `Update` wet entry/exit splash (`:16843-16996`) | **batch 4 WIRED** (ANALYSIS "Batch 4") — the generic wet machine (`projectile_wet::wet_block`, `:16786-17007` verbatim) + the 8-site `WetSplash` drain, the g15 WorldItem precedent |
| 9-14 | 17038/17042/17046/17055/17066/17071 | per-tick arrow trail (`:17036-17077`, kinds 2/82/172/103/278/4/5) | **batch 1 WIRED** |
| 15 | 17113 | stationary 466 trail (`:17108-17116`) | **batch 3 ARMED** — the TrailingMode-1 row landed (`CULTIST_LIGHTNING_ORB_ARC`); the 580 twin's body verbatim; the AI_088 466 branch stays unported + kind-gated |
| 16 | 17122 | stationary Vortex-bolt 580 puff (`:17117-17125`) | **batch 1 WIRED** |
| 17-28 | 17325/17332/17346/17368/17377/17387/17397/17408/17418/17428/17443/17462 | enchantment visuals (`:17290-17467`, frostBurn/magmaStone/bee-mount/imbue 1-8) | **batch 1 WIRED** (incl the Party gore sibling's draws) |
| 29 | 18231 | Meowmere 502 bounce ring (`:18226-18237`) | CITED SKIP (FINAL re-probe 2026-11-21: aiStyle 8, d21 covers 201-209 NOT 8; no row on d1/d2/d16/g1/g2/g3 tips — the d75 sound side `:18209` is STILL DEFERRED too, no split conflict) |
| 30-31 | 18949/18952 | LavaBoulder 1053 impact (`:18947-18956`) | **FINAL STAGED** — d3 `2d01693ca` rowed the boulder family AFTER the batch-3 census; `lava_boulder_impact_dust` (10×2 type-6 pair, no argument draws) + the d3 `bounce_ladder` drop-in contract (the ladder's dust skip draws nothing today). The `:18958` HitTiles + `:18959` Dig stay the tile-dust/sound lanes' (d75's `:18959` row re-classifies SIBLING-CUED at the d3 merge) |
| 32 | 19245 | Poo 1048 bounce (`:19242-19246`) | **FINAL STAGED** — d3 `2d01693ca` rowed it; `poulder_bounce_dust` (10 type-322 at NextVector2FromRectangle(Hitbox)−One, the X-then-Y float pair) + the REPLACE contract (the ladder's count-only pairs must be DELETED at the swap — keeping them DOUBLE-DRAWS; the PooFly particle draws stay d3's). The `:19241` Item177 cue is ALREADY d3's (`wants_sound (2,177)`) — dust-vs-sound split clean |
| 33-35 | 19392/19401/19468 | phaseblades / Keybrand 1074 (`:19375-19508`) | CITED SKIP (FINAL re-probe: aiStyle-3 rows for 1059-1072/1074-1076/1101/1102 unported on every pushed tip; `is_a_phaseblade` serves the hitbox narrowing only; the d75 `:19531` sound row stays STILL DEFERRED) |
| 36 | 19811 | Sword Beam 116 impact burst (`:19809-19814`) | **batch 2 WIRED** |
| 37 | 20322 | Light Disc 106 bounce sparks (`LightDisc_Bounce` `:20317-20329`, both callers `:13074`/`:19518`) | **batch 2 WIRED** |
| 38 | 20423 | BoulderOfEarth 261 explosion (`:20421-20430`) | CITED SKIP (FINAL re-probe: aiStyle 14, no row on d2/d16/g1/g3; d54 does not own it; the d75 `:19649` sound row stays STILL DEFERRED — the `MakeBoulderOfEarthExplosion` host `:20391-20431` is the aiStyle-14 HandleMovement arm `:19643-19651`, unported) |
| 39 | 22161 | SuperStar 728 trail (`AI_151` `:22157-22167`) | **batch 3 STAGED** — d16's `ai_super_star` (e122a6b2) keeps count-only gates; `super_star_dust` replays them (the e3 drop-in contract) |
| 40 | 22297 | SuperStarSlash 977 trail (`AI_152` `:22292-22301`) | **KEEP-ONE e3** — the pushed W4-e3 lane STAGED `star_slash_dust` (7a25cd61) for :22297/:22308; this lane cites it |

**Totals (after batch 4): 29 wired (batch 1 = 19, batch 2 = 2, batch 4 =
the 8 wet sites) + 1 armed (the 466 trail) / 1 staged (the 728 emitter) +
the 190/191 ring builder ARMED (the :17299-17306 geometry, not a numbered
NewDust site of its own) + 1 keep-one-e3 (:22297, with :22308 in e3's
window) + 8 unported-host skips (502 + the 1053 pair + 1048 + the 3
phaseblade sites + 261) = 40.** (The batch-4 commit message's "7 skips"
under-counted the phaseblade group as one — the ledger rows are the
authority: 33-35 are THREE sites.) The batch-3 re-census verdicts are
recorded in ANALYSIS "Batch 3" (the pushed-branch host checks: d16
e122a6b2, d19 a4185f89, d21 9fa9e9ba, d54 8cb5d5a4, f32 c05ef86f); the
batch-4 wet machine in ANALYSIS "Batch 4".

**FINAL totals (after the FINAL phase, 2026-11-21): 29 wired + 1 armed
(the 466 trail) + 4 staged (the 728 emitter + the FINAL 1053 pair + the
FINAL 1048 site) + the ring builder armed + 1 keep-one-e3 + 5 remaining
unported-host skips (502 + the 3 phaseblade sites + 261 — the 1053/1048
skips SUPERSEDED by d3 2d01693ca) = 40.** The FINAL re-probe read the
pushed sibling tips (d3 2d01693ca, d1 e271b5bc8, d2 f430ccf83, d16
d414afbca, g1/g2 81c4ab85a/g3, e19 939e45dc8, f47 507a0da5c, d75
0e700d9ab, e3 8023aa4d5); the verdicts are in the ledger above +
ANALYSIS "The FINAL phase".

### Commits

- `5308208b` — batch 1: the 19 drain-lane sites (arrow trail 6, stationary
  580 puff 1, enchantment visuals 12). `projectile_dust.rs` + the
  `pending_dust_bursts` queue + the tick snapshots.
- `e5893283` — batch 2: Sword Beam 116 + Light Disc 106 (both callers), the
  remaining first-40 unported hosts cited skips.
- batch 3 — the NOW-UNBLOCKED arms: the 466 TrailingMode-1 row + shared
  stationary trail, the 190/191 two-box enchantment ring, the STAGED 728
  `super_star_dust`; the e3 name-collision merge note (ANALYSIS).
- batch 4 — the WET-SPLASH HOST: `projectile_wet.rs` (the generic wet
  machine `:16786-17007` verbatim — wetCount, the liquid-flag latching, the
  IsDestroyedByWater/2-transform/34-dryout/80-kill arms, the entry/exit
  edges + the 155 carve-out) + the `WetSplash` drain (the 8 sites) + the
  `wet_count`/`wants_wet_splash` slot fields + the `water_style` pool feed;
  the dispatch runs the machine per sub-step at the C# `:16790` slot.
- `f266c6fb` — FINAL item 1: the 11-skips ledger re-probe — the STAGED
  `lava_boulder_impact_dust` (1053) + `poulder_bounce_dust` (1048) emitters
  with the d3 `bounce_ladder` drop-in contract; the 502/phaseblades/261
  skips re-probed and kept.
- `804ad571` — FINAL item 2: the Shimmer() ACTIVATION — g2 `81c4ab85`'s
  STAGED 13-arm helper adopted verbatim (`Projectile::shimmer` +
  `shimmer_two_cell_kill`), the `:17011` caller-gate WIRED in the dispatch
  right after `wet_block` (this lane's wet machine is the authoritative
  `shimmer_wet` writer g2 waited for; the g2d-active probe found NO pushed
  activation branch, so the wiring lands here — keep-one g2's staged copy
  at merge). The `CHLOROPHYTE_ORB` const added (g2's row unmerged here).
- `ab40c8a1` — FINAL item 3: the proj-side wetVelocity census (ANALYSIS
  "The wetVelocity census") — the flail + d3 boulder arms carry the drag;
  the generic `kill_step`/free-move integrate is a movement-lane gap, the
  follow-up arm's input (the wet flags) now authoritative.

### The dust-vs-sound split (the d75 co-lane, FINAL re-probe)

d75 (`0e700d9ab`, the first-120 sound census) re-classified its deferred
rows against the pushed AI tips; the dust lane's FINAL verdicts align:

- **1053/1048** — d75's `:18959`/`:19241` Dig/Item177 rows were STILL
  DEFERRED at its census (no rows); d3 `2d01693ca` now rows the family AND
  queues the cues itself (`bounce_ladder`'s `dig` closure → `wants_sound
  (0,0)` / `(2,177)`). The DUST half is this lane's STAGED emitters; the
  SOUND half is d3's already-queued cues — no overlap, the merge keeps
  both (d75's rows re-classify SIBLING-CUED at the d3 merge).
- **502** — d75's `:18209` Item37 row stays STILL DEFERRED (no row); this
  lane's `:18231` dust skip stays too. Same host, same verdict.
- **261** — d75's `:19649` Dig row stays STILL DEFERRED; the dust skip
  stays. The `MakeBoulderOfEarthExplosion` host is the aiStyle-14
  HandleMovement arm, unported on every tip.
- **The wet splash family** (`:16871-16994`) — d75's row was STILL
  DEFERRED ("the generic wet block, d53's skip"); batch 4 landed the
  machine AND the case-19 plays (`pending_splashes`, shimmer entry 4 /
  exit 5, the rest default 1) — d75's row re-classifies SIBLING-CUED at
  this lane's merge. The 9 guard-refused shimmer siblings are cited in
  `projectile_wet.rs`'s module doc.

### Witnesses

20 draw-mirror witnesses in `projectile_dust.rs` (the d52 `burst_block`
oracle shape + the post-field writes + the queue half; batch 3 adds the
466/580 twin census, the ring two-box geometry and the staged SuperStar
census; batch 4 adds the wet-splash per-kind census, the post-fields +
`dustWater` style feed, the exit-water box asymmetry and the wet queue
half) + 9 machine witnesses in `projectile_wet.rs` (the entry no-cooldown
asymmetry, the exit 10-tick cooldown, the four liquid-kind transitions,
the water-kill + 2→1 transform, the 34/80 arms, the 155 carve-out, the
preamble + ignoreWater gate, the ≤40 dry fold, the dry-clear tail),
**WRITTEN NOT RUN** (code-only mode — no cargo/rustc/tests
invoked). The stationary-580 witness
pins the 0×0-box `+4` inset (`Dust.cs:155-171`: the box clamps to 5×5, so the
default position is `anchor + Next(1) + 4`, NOT the exact anchor).

FINAL adds 3 drain witnesses (`projectile_dust.rs`: the 1053 10×2 type-6
census + post-fields, the 1048 pair-then-block census + the truncating
anchor band, the pairs-then-blocks split contract) + 7 Shimmer witnesses
(`projectile.rs` `w4_d53_shimmer_tests`: the arrow float, the style-1
bounce cap/age, the two-cell kill probe, the hostile refusal, the star
bounce keeping life, the thrown -0.7/-20 arm, the dispatch gate firing
after the wet machine on a shimmer-pond shuriken) — all WRITTEN NOT RUN
(code-only).
