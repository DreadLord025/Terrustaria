# 70 — Projectile dust SITE batches (W4-d53 / W4-e3 / W4-f5): claims + census

Status: CLAIM + completion record (W4-d53, the FIRST ~40 lane). Written per
the lane's coordination rule — the branch list is the claim record; this doc
pins the site-range split so the three site-batch lanes never overlap each
other or the per-kind trail lanes (g63/g64) or the death lane (g65).

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
| 15 | 17113 | stationary 466 trail (`:17108-17116`) | CITED SKIP — CultistBossLightningOrbArc has no ported row / TrailingMode entry |
| 16 | 17122 | stationary Vortex-bolt 580 puff (`:17117-17125`) | **batch 1 WIRED** |
| 17-28 | 17325/17332/17346/17368/17377/17387/17397/17408/17418/17428/17443/17462 | enchantment visuals (`:17290-17467`, frostBurn/magmaStone/bee-mount/imbue 1-8) | **batch 1 WIRED** (incl the Party gore sibling's draws) |
| 29 | 18231 | Meowmere 502 bounce ring (`:18226-18237`) | CITED SKIP — no `proj_data` row |
| 30-31 | 18949/18952 | LavaBoulder 1053 impact (`:18947-18956`) | CITED SKIP — no row |
| 32 | 19245 | Poo 1048 bounce (`:19242-19246`) | CITED SKIP — no row |
| 33-35 | 19392/19401/19468 | phaseblades / Keybrand 1074 (`:19375-19508`) | CITED SKIP — no rows |
| 36 | 19811 | Sword Beam 116 impact burst (`:19809-19814`) | **batch 2 WIRED** |
| 37 | 20322 | Light Disc 106 bounce sparks (`LightDisc_Bounce` `:20317-20329`, both callers `:13074`/`:19518`) | **batch 2 WIRED** |
| 38 | 20423 | BoulderOfEarth 261 explosion (`:20421-20430`) | CITED SKIP — no row |
| 39 | 22161 | SuperStar 728 trail (`AI_151` `:22157-22167`) | CITED SKIP — no row |
| 40 | 22297 | SuperStarSlash 977 trail (`AI_152` `:22292-22301`) | CITED SKIP — no row |

**Totals (after batch 4): 29 wired (batch 1 = 19, batch 2 = 2, batch 4 =
the 8 wet sites) + 11 remaining cited unported-host skips = 40.** (The
batch-3 re-census of the 11 skips — the 466 trail, the 190/191 ring, the
728 emitter, the e3 keep-one — is the co-lane's in-flight work, recorded
in its own commit.) The batch-4 wet machine is ANALYSIS "Batch 4".

### Commits

- `5308208b` — batch 1: the 19 drain-lane sites (arrow trail 6, stationary
  580 puff 1, enchantment visuals 12). `projectile_dust.rs` + the
  `pending_dust_bursts` queue + the tick snapshots.
- `e5893283` — batch 2: Sword Beam 116 + Light Disc 106 (both callers), the
  remaining first-40 unported hosts cited skips.
- batch 4 — the WET-SPLASH HOST: `projectile_wet.rs` (the generic wet
  machine `:16786-17007` verbatim — wetCount, the liquid-flag latching, the
  IsDestroyedByWater/2-transform/34-dryout/80-kill arms, the entry/exit
  edges + the 155 carve-out) + the `WetSplash` drain (the 8 sites) + the
  `wet_count`/`wants_wet_splash` slot fields + the `water_style` pool feed;
  the dispatch runs the machine per sub-step at the C# `:16790` slot.

### Witnesses

17 draw-mirror witnesses in `projectile_dust.rs` (the d52 `burst_block`
oracle shape + the post-field writes + the queue half; batch 4 adds the
wet-splash per-kind census, the post-fields + `dustWater` style feed, the
exit-water box asymmetry and the wet queue half) + 9 machine witnesses in
`projectile_wet.rs` (the entry no-cooldown asymmetry, the exit 10-tick
cooldown, the four liquid-kind transitions, the water-kill + 2→1
transform, the 34/80 arms, the 155 carve-out, the preamble + ignoreWater
gate, the ≤40 dry fold, the dry-clear tail), **WRITTEN NOT RUN**
(code-only mode — no cargo/rustc/tests invoked). The stationary-580 witness
pins the 0×0-box `+4` inset (`Dust.cs:155-171`: the box clamps to 5×5, so the
default position is `anchor + Next(1) + 4`, NOT the exact anchor).
