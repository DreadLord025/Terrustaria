# 196 — Projectile.cs per-kind TRAIL dust lanes (W4-g64 fork): claims + census

Status: CLAIM doc (W4-g64, lane 2) — **RENUMBERED 66 → 196** per the d54
collision registry (`docs/research/150-research-number-registry.md` on
`task/W4-d54-dust-sites-world`, tip `38bfd1d22`): the number 66 belongs to
W4-g63's `66-proj-trail-dust-claims.md` (first add 2026-11-21T00:30:45,
`34bf98b39`); this lane's copy is a diverged same-path fork (first add
01:53:25) and takes the assigned free number **196** with the `-g64` suffix.
Content preserved; refs updated in `docs/ANALYSIS.md`, `docs/PORT_MAP.md`,
`anchors.txt` and the two Rust doc comments per the registry's §3 row.
Written early per the lane's coordination rule — the branch list is the
claim record; this doc pins the kind-order split so the three batches never
overlap.

## In-flight probe (2026-11-21)

`git fetch origin --prune` at claim time: **no pushed claims exist for the
co-lanes** — `task/W4-d53-*` and `task/W4-e3-*` have no remote branch, and
`task/W4-f5-dust-proj-3`'s tip is exactly `origin/task/W4-d50-dust-type-table`
(`2e243a2d`, the DustPool foundation — a bare claim, zero site work).
`task/W4-g63-*` / `task/W4-g65-*` likewise unpushed. The only pushed dust-site
claim doc is `64-dust-main-sites-claims.md` (g11/g12/g13, **Main.cs** — a
different file, no overlap). This doc therefore pins the Projectile.cs split
from the directive: **g63 = kinds ≤ 400, g64 = kinds ≥ 401 (this lane),
g65 = death effects** (`Kill()` `:70642-77263`'s 399 sites +
`Kill_Bombs_DoUsualKillCode` `:77648+`'s 52 + `Kill_ExplodeTiles` `:77374` +
`DoRainbowCrystalStaffExplosion` `:78583/78599`).

## Census

`decompiled/Terraria/Projectile.cs` carries **1195** `Dust.NewDust*`-family
call sites. By enclosing method (line-ordered): `Update` 16 (the wet-entry
splash family — movement lane), `EmitEnchantmentVisualsAt` 12 (enchant lane),
`HandleMovement` 8 (bounce/collision lane), `LightDisc_Bounce` 1,
`MakeBoulderOfEarthExplosion` 1, the named `AI_*` machines ~250, `AI()`'s
inline style arms 233, `AI_001` 121, `AI_010` 13, `AI_026` 24, `AI_062` 9,
`AI_075` 13, the late named machines ~90, and the Kill family 454.

**This lane's scope rule:** the REPEATING in-flight trail sites inside
`AI()`'s per-kind arms for kinds ≥ 401 — the dust a kind sheds while it flies.
One-shot spawn-latch bursts (`localAI[0] == 0` / `ai[1] == 0` first-tick
blocks) are NOT trails; they stay with the general proj-site co-lanes
(d53/e3/f5) and are cited per kind below. Death bursts are g65's.

## W4-g64 kind ledger (41 kinds, 5 commits of ≤ 10)

| kind | style | trail sites (Projectile.cs) | one-shots EXCLUDED (co-lane) |
| ---- | ----- | --------------------------- | ---------------------------- |
| 406 | 60 | 29205, 29229 | — |
| 419 | 73 | 30501 | — |
| 420 | 73 | 30526 | — |
| 421 | 73 | 30547, 30567 (the %10 ring repeats) | — |
| 422 | 73 | 30590 | — |
| 447 | 79 | 30888, 30896 | — |
| 450 | 14 | 25197, 25206 | — |
| 455 | 84 | 31627, 31633 | — |
| 461 | 84 | 31669, 31675 | — |
| 465 | 88 | 32023, 32040 | — |
| 466 | 88 | 32076, 32083 (stationary sparkle) | — |
| 497 | 2 | 23413 | — |
| 503 | 5 | 23959 (WIRED — W4-g64 FINAL, the e3 `80d47a56` row constants-copy) | the :23963 Next(48) GORE gate (d55/g16 — the gate draw rides AFTER the spark in `ai_falling_star`) |
| 504 | 2 | 23543 | — |
| 509 | 20 | 25578 | — |
| 533 | 65 | 29804 (dash ring), 30055 (spin ring) | — |
| 537 | 84 | 31694, 31701 | — |
| 538 | 14 | 25172, 25180 | — |
| 539 | 102 | 33305 (host), 33420, 33428 (flight) | — |
| 573 | 102 | 33324 (host), 33439 (flight) | — |
| 574 | 102 | 33336 (host) | — |
| 578 | 108 | 33746, 33754 (wind-up) | 33688 (ai[1]==1 25-burst) |
| 579 | 108 | 33721, 33731 (wind-up) | 33688 (shared burst) |
| 596 | 107 | 33623 (the Next(3)!=0 EARLY-RETURN ring) | 33644 (timeLeft<4 death burst → g65; the d11 census keeps the box/damage re-arm + the ten Next(2) picks) |
| 618 | 119 | 34871 | — |
| 642 | 84 | 31648, 31654 | — |
| 656 | 127 | 36074 | — |
| 659 | 129 | 36268 | — |
| 669 | 68 | 30214 (the flag35 pair; 30202 is 399's — g63) | — |
| 723 | 5 | 24023, 24026 (the 3×2 orbit) | the :24016 Next(6) GORE gate (d55/g16) |
| 732 | 44 | 27915 | — |
| 895 | 124 | 35231 | — |
| 898 | 124 | 35295 | — |
| 907 | 30 | 26689 | — |
| 967 | 28 | 26588 | 26657 (ai[1] latch 10-burst) |
| 1026 | 28 | 26563 | 26665 is ParticleOrchestrator (no dust) |
| 1048 | 25 | 26398 | — |
| 1053 | 25 | 26035, 26041, 26046 | 18949/18952 (HandleMovement bounce — movement lane) |
| 1092 | 18 | 25449 | 25402 (ai[0]==0 6-burst) |
| 1114 | 24 | 25985 | — |
| 1133 | 5 | 24003, 24008 | the :23993 Next(24) GORE gate (d55/g16) |

**Deferred within ≥ 401 (cited, unclaimed by this doc's batches):** 813
(style 108's 13 sites span four ai[0] phase windows — a whole-machine port,
left for a follow-up batch), 957 (style 124's spelunker tile-scan draws are
behaviour-coupled `IsTileSpelunkable` world reads), 670 (style 126's sites are
style-GENERIC, not `type == 670`-gated — the style lane's), 644 (style 112's
267 orbit needs `GetLastPrismHue`/`rgbToHsl` + the Disco cycle — the prism
lane's), 493/494 (style 4's alpha-crossing one-shot), 836 (style 112's
localAI[0] latch burst), 968 (style 2's one-shot 8-burst), 1134 (style 24's
latch burst), 1044/624 (style 117: 1044 has NO dust of its own — the flag60
arm is ParticleOrchestrator only; 624's flag59 block is the ai[0]==1
explosion, a burst not a trail), 590 (the style-112 census rows at
34272/34285/34317 belong to **644** — the `type != 644` return at :34235
preceded them; 590's own block :34194-34234 is light-only).

## Contract

Pool: `origin/task/W4-d50-dust-type-table`
(`tr_core::particles::DustPool::new_dust` / `new_dust_perfect` /
`new_dust_direct`, `Dust.cs:75-256` verbatim; the branch is fast-forwarded
into this lane's base). RNG: every draw rides the **UpdateProjectiles**
stream (`Main.cs:17775`'s `SwapRandom`), per-slot C# statement order —
explicit argument draws FIRST, then the `NewDust` internals, then the
post-spawn writes (the sentinel slot takes the writes too). Lights
(`Lighting.AddLight` / `PlotTileLine`) stay the e6/f7 addlight-proj lanes;
gore bodies + their gate draws stay d55/g16; sounds stay d76.
`customData` is the pool's documented dropped field. WRITTEN NOT RUN
(code-only mode): the witnesses are compiled by the merge-verify wave.

## W4-g64 wiring wave (2026-11-21, code-only)

The g63 pushed claims (`origin/task/W4-g63-proj-trail-dust-1`, `34bf98b3` +
`bdeedac0` + `708a4358`) were read: lane 1 wires its kinds-≤400 arms through
the `update`/`update_dust` scratch-pool seam (`Npc::update_rng` precedent),
`DustDraw` adapting the machines' `DropRng` to the pool's `DustRng`,
`TrailSlot` snapshots at the call sites. This lane adopts the SAME seam
shape (`ProjectilePool::update_dust` + the scratch `update` entry +
`world_sim/projectiles.rs` threading `app.dust` + `app.spawn_ctx()`); the
emitters keep their scalar-arg form and take `rng: &mut dyn DropRng`
directly (the `DustDraw` wrap lives inside each pool call). The g63 branch
is NOT merged into this one — the two `update_dust` seams reconcile at the
merge wave (the co-lane rule).

**Row census (the wiring gate — S1-F-9b):** of the 41 ported kinds, **15
have `proj_data` rows** and are WIRED at their C# sites: 447 (79), 450 (14),
455 (84), 461 (84), 509 (20, the behaviour-feeding jitter REPLACES the
`ai_drill` skip), 533 (66 — the port's row; the dash ring at the ladder's
`ai[0]==8` + the spin ring at the frame table's `ai[0]==2`, the C# 65
machine rides the port's `ai_twins_minion`), 574 (102 host), 578/579 (108
wind-up), 618 (119), 642 (84, the draws-kept skip REPLACED), 659 (129), 732
(44), 967/1026 (28, the `localAI[0]` post-increment + cadence pins landed).
The **954 SPARK rider** (rowed, aiStyle 2) shares the 504 arm — its spark
window is wired kind-gated inside `ai_style2` (the 504/979 co-riders are
unrowed, their draws stay off the stream). The other **26 kinds are
UNROWED** (406/419/420/421/422/465/466/497/503/504/537/538/539/573/596/656/
669/723/895/898/907/1048/1053/1092/1114/1133 + 979): their emitters stay
cited + witnessed, the wiring waits on the AI lane's rows. The **596 bool
seam** is pinned by a witness
(`w4_g64_wire_596_bool_seam_gates_the_ai_continuation`): `false` = the C#
`return` from AI() (`:33619-33622`) — the caller must abort the machine's
continuation; 659's twin CONTINUES (wired).

**Remainder-wave row updates (2026-11-21, the d11/d13 reads):** the row
census above is the wiring wave's snapshot; the remainder wave rows + wires
four more kinds — **596** (the d11 row: 8x8/107/pen 1/180/alpha 255/
ignoreWater, `projFrames` 4; `ai_djinn_curse` ACTIVATES the pinned bool
seam — false returns from the machine, the homing tail + the orphan cap
skipped that tick; the timeLeft<4 flash keeps the d11 census, the bodies
g65's), **895/898/957** (the d13 rows, the style-124 machine — remainder 1)
and **813** (the d11 shared 108 row — remainder 2). The unrowed list
shrinks to 22 (596/895/898 drop off; 957/813 were deferred-ledger entries,
not ledger kinds).

**Deferred-kind census (the wiring wave's re-probe):**
- **644 — UNBLOCKED, PORTED + WIRED.** The original deferral cited
  `GetLastPrismHue`/Disco; the actual C# (`:34239`) is
  `Main.hslToRgb(ai[0], 1, 0.5)` — the byte-quantised palette port
  (`tr_core::tile_light::hsl_to_rgb`) already exists, the `Opacity` is the
  machine's `opacity_of(alpha)`, and the sparkle's `CloneDust` rides the
  pool's verbatim `clone_dust` (`Dust.cs:263-286`). `trail_644` lands the
  two orbit gates + the sparkle (the RandomVector2 pair is TWO
  NextDouble-cast draws, `Utils.cs:2431-2434`), wired into
  `ai_rainbow_crystal_shard` replacing the draws-kept skip. The pool's
  `new_dust_perfect`/`new_dust_direct`/`clone_dust` became generic over
  `DustRng` (the `new_dust` shape; concrete callers unchanged).
- **813 — UNBLOCKED, PORTED + WIRED (remainder wave, the d11 read).** The
  d11 branch (`task/W4-d11-proj-ai-101-110`, tip `e0395afc`) landed the 813
  completion: the shared `:6282-6293` row (read via `git show`, landed here
  on the 578/579 arm) + the four-phase machine with the dust GATES modelled
  and the bodies skipped (its census: wind-up two Next(4), charge two
  Next(2), full ONE Next(2) if/else, dissolve two Next(3), the per-tick
  opacity NextFloat gate; the dissolve `Kill()` does NOT return —
  `be58df9e`). This wave REPLACES the census skips with the live bodies:
  `trail_813_windup`/`_charge`/`_full`/`_dissolve` (the shared ring body —
  the 5-arm's Next(10,21) position re-roll vs the 240's ×30 keep, the ×4/×2
  wind-up vs ×6/×3 charge speeds, the dissolve's scale-premultiplied spin
  vector, `customData = Center` dropped) + `trail_813_opacity_tail` (the
  NextVector2Circular(3,20) pair + the type-90 Perfect, UnitY velocity,
  scale = this.scale). The 578/579 charge/full/dissolve arms stay the
  co-lane's unmodelled skips (the d11 census — their witnesses pin the
  current stream). The 619 spawn point takes the C# `(int)Center`
  truncation (`:33884` — the d11 port carried the top-left, fixed).
- **957 — UNBLOCKED, PORTED + WIRED (remainder wave, the d13 read).** The
  d13 branch (`task/W4-d13-proj-ai-121-130`, tip `314fa6cd`, UNMERGED) rows
  were read via `git show`: the `GLOMMER_PET` row (20x20, aiStyle 124,
  penetrate -1, 18000, ignoreWater/netImportant) + `projFrames` 12 landed
  here for the three trail kinds (895/898/957; the 650/882/888/896 rows stay
  d13's — the merge keep-one). The whole `ai_pets_style124` machine
  (`projectile_ai_pets124.rs`, the d13 port) landed dust-wired: the 957's
  behaviour-coupled half is the `SolidTileAllowBottomSlope` 5-cell column
  scan (`:35483-35497`, the shared `projectile_ai026_c` helper) gating the
  grounded bounce cycle — the 957 draws NO dust itself; its stream draw is
  the cycle-end `200 + Next(150)` rest roll (`:35519`), witnessed in scan
  order. The style-124 dust sites (`:35231`/`:35295` the 895/898 flight
  sparkles, `:35601`/`:35681` the 650's 204 pair — d13 skipped the bodies
  whole) are RE-ARMED: `trail_895`/`trail_898` on the frame-reset tick,
  `trail_650_scan_sparkle`/`trail_650_point_sparkle` new. The `screen_width`
  seam (`set_screen_width`, the d13 `set_disco` pattern) feeds the 650 scan
  gate. The buff-spawn seam (`spawn_pet_if_needed`) takes d13's
  rowed-kind `spawn_with_data_ai_owned` drop-in.
- **670 — still deferred.** The d13 branch carries the `OGRE_STOMP` row
  (aiStyle 126), but the style-126 dust sites (`:35852-35859` the 3-dust
  rise, `:35909-35927` the 60-dust latch burst, `:35928-35944` the 9-dust
  arm, `:35952+` the 6-gate column) are style-GENERIC (no `type == 670`
  gate; the 670-gated blocks at `:35870/35889` are the height trim + the
  stomp spawn, no dust) — the style lane's, not a per-kind trail.
- **590 — confirmed NOT a trail kind.** The `:34272+` census rows belong to
  644 (the `type != 644` return at `:34235` precedes them); 590's own block
  (`:34194-34234`) is the frame cycle + the alpha-gated `Lighting.AddLight`
  (the `Next(28,42)` at `:34221` is a LIGHT-intensity draw, e6/f7's) + the
  sine bob — zero dust. The d12 branch's `TRUFFLE_SPORE` row (unmerged)
  changes nothing for this lane.

**Witnesses:** the 40 port witnesses stand (the RNG-seam conversion kept
their concrete `UnifiedRandom` streams); the wiring wave adds 5
stream-parity witnesses (`w4_g64_wire_*`: the DropRng/DustDraw seam parity,
the 509 jitter-replacement full-order mirror, the 644 orbit/sparkle/clone
mirror, the 596 bool-seam contract, the 642 + 509 machine-side parity in
`projectile.rs`). WRITTEN NOT RUN (code-only).

## The remainder wave (2026-11-21, code-only) — the keep-one refresh

The wiring-wave matrix above is a SNAPSHOT (the d54e reads): the remainder
wave re-probed the PUSHED co-lane tips (`git show`, never merged) and landed
the deferred kinds. The updated keep-one table lives in ANALYSIS "The
remainder wave → The keep-one table" (the merge contracts, per contested
site). Summary of the deltas against this doc's stale rows:

- **957/895/898 — ROWED + WIRED** (remainder 1, the d13 `314fa6cd` →
  `a2588cd2` read): the style-124 machine ported dust-wired; the 957's
  column scan is behaviour-coupled (no dust body — the Next(150) rest roll
  is its stream draw). The d13 residual's `tentacle_pointer_pose` (the draw
  consumer) is additive, not ported here.
- **813 — ROWED + WIRED** (remainder 2, the d11 `e0395afc` + the d13
  `a2588cd2` read): the four-phase machine's bodies live; the corrected 108
  row (the stock pen 1 + alpha 255), the ignoreWater flag-list entry, the
  Item8 cue + the once-per-death Kill head adopted from the d13 residual.
- **596 — ROWED + WIRED** (remainder 3, the d11 read): the bool seam
  ACTIVATED in `ai_djinn_curse`; the unrowed list shrinks to 22.
- **The e3/g63 keep-one contracts** (remainder 4): the 497/504/503/723/1133
  twins are stream-exact (5 new `w4_g64_keep_one_*` parity witnesses); the
  survivors are e3's `ai_style2`/`ai_falling_star_family` (the 497/504/503),
  f48's `ai_cloak_star` (the 723/1133 cloak half), the g64 dust-wired
  `ai_pets_style124`/`ai_vortex_ritual` (the 895/898/957/813), and d11's
  `ai_nebula_sphere` 575 half (the 596 seam feeds it). The g63 `update_dust`
  seam is the SAME shape (the kind gates disjoint: ≤400 / ≥401).

The remainder witnesses (6 style-124 + 3 813 + 1 596 + 5 keep-one = 15) are
WRITTEN NOT RUN (code-only).

## The FINAL phase cross-activation census (2026-11-21, the g63d/d19c reads)

The g63 latest tip (`2ea3e711`, the f20b 357-seam cross-note over
`036901a5`'s research/67 activation checklist) re-probes the owed
AI_001 unrowed kinds. This lane's kinds are all ≥401 — **none of the
281/174/176 rows is a g64 kind**, so no g64 emitter is owed for them.
The cross-activation notes (the merge wave's inputs):

**Re-probe receipt (the g63d-active read, this FINAL round):** a fresh
`git fetch origin task/W4-g63-proj-trail-dust-1` confirms the tip is still
`2ea3e7110` (no activation-phase row commit landed). The `281 =>` /
`174 =>` / `176 =>` / `CANNON_BUNNY =>` `proj_data` arms are ZERO on both
the g63 tip and this branch's HEAD (the grep exit-1 census) — the rows
stay UNCLAIMED, the emitters stay UNWIRED. The "g63d-active" phase is the
research/67 activation CHECKLIST (which merge unlocks which emitter), not
a row-landing commit; nothing changed for the 281/174/176 activation
state since the batch-7 re-probe.

- **281 (CannonBunny, aiStyle 49)** — the row stays UNCLAIMED on every
  pushed tip (the g63 `eea7156f` census correction: the machine IS
  claimed + ported — d5 `5f7390a3`'s `projectile_ai_style49.rs`,
  UNWIRED; the `:23321-23364` aiStyle-2 twin is DEAD code, 281 rides
  style 49 per `:3680`). The g63 `bunny_burst` emitter (the `ai[0]==3`
  tick-3 block, sites `:28292/28298/28302`) stays UNWIRED — activation
  = the row + d5's merge + the one-line call into d5's tick-3 block.
  This branch's `CANNON_BUNNY` const (`data.rs:721`) is the
  `BunnyCannonCanFire` scan anchor ONLY — it carries no `proj_data`
  row and no dispatch arm; it does NOT activate the emitter (the
  S1-F-9b row gate reads `proj_data`). The g16 dead-twin landing gore
  + the g65 kill-census one-consumer rules are the merger's (g63's
  research/67 rows).
- **174/176 (IceSpike/JungleSpike, AI_001)** — unrowed on every pushed
  tip (the g63 batch-7 re-probe). The g63 `ice_spike_dust`/
  `jungle_spike_dust` emitters stay UNWIRED. **The `old_position`
  latch debt:** the C# `Projectile.cs:17013` latches
  `base.oldPosition = base.position` for EVERY projectile between the
  AI and HandleMovement; this branch's latch (`projectile.rs`'s
  `old_position = pos` seam) is golf-gated (`ai_style == 149 || 150`,
  the S1-P-i55 slice) — the 174's `oldPosition − velocity×3` spawn
  point (`:55036`) reads a stale zero latch until the activating merge
  widens it to the C# every-projectile form. NOT widened here (the
  widening changes every kind's `old_pos` reads — the golf pad probe
  + the trident `old_vel` precedent ride the same field; the merge
  wave owns the re-derivation).
- **The d19c 92-clouds** — ZERO trail-dust sites (the `:32558-32820`
  arm re-read whole: the only draw is the 1128 first-tick frame roll
  `:32565`; the 1007 GasTrap particle broadcast is the particle
  layer's skip). The d19 machine (`projectile_ai_clouds.rs`,
  `c5a7e5d3`) is the sole owner of the 511/512/513/1007/1128 rows; no
  g64 keep-one owed (the g64 ledger's "92" mentions are the 1092
  LunarHook kind — a different id). Adjudicated in ANALYSIS "The
  FINAL phase".

## The FINAL phase activation (2026-11-21) — trail_503 NOW-WIRED

The e3 `80d47a56` coverage-row commit rowed the 503 StarWrath
(`Projectile.cs:5743-5756`) — this lane's `trail_503` was the unwired
twin. W4-g64 FINAL copies the row CONSTANTS verbatim (the
`STAR_WRATH` const + the `proj_data` 10-tuple + the melee flag-list
entry — byte-identical to e3's port; the 1037/1049 twins stay e3's,
no g64 emitter rides them) and ACTIVATES the emitter:
`ai_falling_star` gains the 503 half (the `:23853-23856` star-family
`Center.Y > ai[1]` head — NOT the 9's Bottom.Y latch; the shared
`:23900-23913` alpha ramp; the `:23946-23948` velocity aim −π/2 with
NO direction write; `trail_503` then the `:23963` Next(48) gore GATE
draw — the e3 keep-one order). The dispatch's aiStyle-5 arm kind-gate
widens to `STARFURY | STAR_CANNON_STAR | STAR_WRATH`. The 9/955
halves keep their pre-existing arms + cited dust skips (the e3/f48
keep-one lanes); the soundDelay hum (`:23881-23885`) stays d75's owed
503 extension (the e3 cross-note). Merge keep-one: e3's
`ai_falling_star_family` survives (the fuller family arm), this
lane's 503 half + `trail_503` retire into it — the row is
byte-identical, the draws stream-exact
(`w4_g64_keep_one_503_gore_gate_rides_after_the_emitter` + the new
`w4_g64_final_503_rowed_and_wired_gore_gate_after_the_spark`
activation witness, WRITTEN NOT RUN). The unrowed list shrinks to 21
(503 drops off).

## The TRAIL-DUST RETIRE round — FINAL (2026-11-21, code-only)

The lane's closing adjudication round. All co-lane reads via `git show`
at the exact pushed tips (never merged); no cargo/tests run — every
witness stays WRITTEN NOT RUN per the code-only mode.

### The five retire items

1. **`clone_dust` genericisation keep-one (e3 `ac6392cf2` vs g64
   `853039bfc`) — CONFIRMED: either lands.** The four genericised blocks
   (`trait DustRng`, `new_dust_perfect`, `new_dust_direct`, `clone_dust`)
   are byte-identical CODE between the two `particles.rs` copies (a
   comment-stripped span diff); the deltas are doc-comment wording only —
   the same verdict as the d54 bible §8.3 step 2 (g61 × g64). The f50
   `BlastRng` (`6fc87fcbc`, `projectile_ai_misc.rs:599`) adapts the SAME
   pool traits over `&mut dyn DropRng` with `DustDraw`'s exact three
   delegations (`Next(max) = Next(0, max)`, one `Sample()` step) plus
   `GoreRng::next_float_direction` (`(float)NextDouble() * 2f − 1f`,
   `Utils.cs:1203-1206`): the adapters are per-lane glue, no keep-one
   between them. New witness
   `w4_g64_retire_clone_dust_generic_seam_matches_the_concrete_stream`.
2. **The g63 357-seam — DISJOINT.** g63's `pulse_bolt_trail`
   (`2ea3e7110`, site `:56761`) waits on f20 slice D (`4bd2f4966`, in the
   f20 tip `31386266f`): the `PULSE_BOLT` arm fades 25, skips the
   dust-only trail, takes NO gravity (the chain head). 357 ≤ 400 — g63's
   axis. Nothing owed by this lane; the contract is g63's research/67
   (one `step_dust` variant, the call BEFORE the fade-25, the PRE-fade
   alpha copied).
3. **The 608 row — f50 owns it (receipt).** f50 `6fc87fcbc` ported
   `solar_flare_blast_bodies` (`Projectile.cs:72183-72224`) citing this
   ledger's `77616a7d6` no-608-row state; re-verified ZERO 608/SOLAR_FLARE
   hits in `proj_trail_dust.rs` + `projectile.rs`. The 608 burst is a
   hurt-reaction Kill-family burst — outside the ≥401 in-flight trail
   scope by construction. No g64 keep-one owed.
4. **The d54-bible §8.3 checklist (`59fe75547`, registry tip
   `38bfd1d22`) — CONFIRMED.** The bible's step-9 g64 row lists exactly
   this lane's remainder keep-ones: pets124 (`ai_pets_style124` g64
   dust-wired survives), 813 (`ai_vortex_ritual` g64 survives), 596
   (d11's 575 half + g64's bool seam), 497/504/503 (e3's arms survive),
   723/1133 (f48's `ai_cloak_star` survives + takes the dust half). The
   FINAL 2 503 activation executes the bible's "trail_503 wireable" row.
5. **This renumber + the family declaration below.**

### The trail-dust-family declaration (FINAL)

The Projectile.cs in-flight trail-dust family is COMPLETE across three
DISJOINT axes, all KEEP at merge (the d54 bible §8.3 step 9):

| axis | lane | file | scope | tip |
| --- | --- | --- | --- | --- |
| per-kind ≤ 400 | W4-g63 | `projectile_trail_dust.rs` | the rowed kinds' arms + the unrowed emitters (research/67 activation checklist) | `2ea3e7110` |
| per-kind ≥ 401 | W4-g64 (this lane) | `proj_trail_dust.rs` + the remainder machines (`projectile_ai_pets124.rs`, `ai_vortex_ritual`, `ai_djinn_curse`) | 41 ledger kinds + 644 + the remainder wave; WIRED via `ProjectilePool::update_dust` | this branch |
| aiStyle-generic window | W4-e3 | `projectile_dust.rs` | `:22297-24318` styles 2/4/5/6/8/152 (a different axis, no body overlap; the d53×e3 filename union is the merger's naming call) | `ac6392cf2` |

The keep-one contracts between the axes (the twins that retire) are the
ANALYSIS keep-one table + the bible §8.3 — pinned by the
`w4_g64_keep_one_*` parity witnesses. The Kill-family death sites stay
g65's; the one-shot spawn-latch bursts stay d53/e3/f5's; the wet-entry /
HandleMovement / enchant sites stay the movement/enchant lanes'.

### FINAL census

- **Ledger:** 41 kinds (5 batches) + the deferred 644 unblocked + the
  remainder wave (813/957/895/898 rowed + wired, 596 activated, 503
  activated FINAL 2). **21 unrowed kinds** stay cited + witnessed
  (S1-F-9b) — the FINAL 2 census: the wiring-wave unrowed list (the
  ledger table above) minus the 596/895/898/503 remainder-wave rows.
- **Witnesses: 63** (46 `proj_trail_dust.rs` incl. the new retire witness
  + 11 `projectile.rs` machine-side + 6 `projectile_ai_pets124.rs` — the
  `fn w4_g64_*` census at this tip). ALL WRITTEN NOT RUN (code-only).
- **Deferred residue (NOT this lane's):** 670 style-generic (the style
  lane's), 493/494/836/968/1134 one-shot latches (co-lanes'), 1044/624
  orchestrator/explosion (no trail), 590 light-only (e6/f7's).

The lane is RETIRED: no further g64 trail-dust work is owed. The merge
wave owns the keep-one executions + the witness compile/run.

## The FINAL union-table (2026-11-21, the research/66 + research/125 input)

The per-kind TRAIL-dust coverage union across the three disjoint axes
(the family declaration above) — the research/66 (this ledger, g64 ≥401)
+ research/125 (e3's coverage census) + g63's ≤400 ledger merged into one
per-kind table for the merge wave. "Wired" = the emitter is called at its
C# site on the `UpdateProjectiles` stream at THIS tip (the S1-F-9b row
gate); "unwired" = the emitter is written + witnessed but no `proj_data`
row activates it. The keep-one column names the surviving machine/arm at
the merge (the ANALYSIS keep-one table + the bible §8.3 step 9).

### g64 ≥401 axis (this lane) — 24 wired kinds, 21 unrowed

| kind | style | emitter | wired? | keep-one at merge |
| --- | --- | --- | --- | --- |
| 406 | 60 | `trail_406` | UNWIRED (no row) | — (unrowed, cited) |
| 419/420/421/422 | 73 | `trail_419`/`_420`/`_421`/`_422` | UNWIRED | — |
| 447 | 79 | `trail_447` | WIRED (`ai_saucer_deathray`) | g64 (sole) |
| 450 | 14 | `trail_450` | WIRED (`ai_spear`) | g64 (sole) |
| 455 | 84 | `trail_455` | WIRED (`ai_moonlord_turret_laser` 455 arm) | g64 (sole) |
| 461 | 84 | `trail_461` | WIRED (`ai_charged_blaster_beam`) | g64 (sole) |
| 465/466 | 88 | `trail_465`/`_466` | UNWIRED | — |
| 497 | 2 | `trail_497` | UNWIRED | e3 `ai_style2` 497 arm survives; `trail_497` retires (stream-exact) |
| 503 | 5 | `trail_503` | **WIRED (FINAL 2)** — `ai_falling_star` 503 half | e3 `ai_falling_star_family` survives; g64 503 half retires (row byte-identical) |
| 504/954/979 | 2 | `trail_504` | WIRED kind-gated to 954 (504/979 unrowed) | e3 `ai_style2` split survives; g64 954 call retires |
| 509 | 20 | `trail_509` | WIRED (`ai_drill` tail, jitter-replacement) | g64 (sole) |
| 533 | 66 | `trail_533_dash_ring`/`_spin_ring` | WIRED (`ai_twins_minion`) | g64 (sole) |
| 537/538/539/573 | 84/14/102 | `trail_537`/`_538`/`_539_*`/`_573_*` | UNWIRED | — |
| 574 | 102 | `trail_574_host` | WIRED (`ai_nebula_eye` host) | g64 (sole) |
| 578/579 | 108 | `trail_578`/`_579` | WIRED (`ai_vortex_ritual` wind-up) | g64 wind-up survives; d11/d13 machines take it |
| 596 | 107 | `trail_596` | WIRED (`ai_djinn_curse`, bool seam) | d11 `ai_nebula_sphere` 575 half + g64 seam; ONE machine |
| 618 | 119 | `trail_618` | WIRED (`ai_nebula_arm`) | g64 (sole) |
| 642 | 84 | `trail_642` | WIRED (`ai_moonlord_turret_laser`, draws-kept-replacement) | g64 (sole) |
| 644 | 112 | `trail_644` | WIRED (`ai_rainbow_crystal_shard`, prism-hue orbit) | g64 (sole) |
| 650 | 124 | `trail_650_scan_sparkle`/`_point_sparkle` | WIRED (`ai_pets_style124`) | g64 dust-wired pets124 survives (d13 machine + dust) |
| 656/669 | 127/68 | `trail_656`/`_669` | UNWIRED | — |
| 659 | 129 | `trail_659` | WIRED (`ai_spirit_flame`) | g64 (sole) |
| 723 | 5 | `trail_723` | UNWIRED | f48 `ai_cloak_star` survives + takes the cloak dust half |
| 732 | 44 | `trail_732` | WIRED (`projectile_ai_spear`) | g64 (sole) |
| 813 | 108 | `trail_813_windup`/`_charge`/`_full`/`_dissolve`/`_opacity_tail` | WIRED (`ai_vortex_ritual`) | g64 dust-wired survives; d11/d13 gates-only retire |
| 895/898 | 124 | `trail_895`/`_898` | WIRED (`ai_pets_style124`) | g64 dust-wired pets124 survives |
| 907 | 30 | `trail_907` | UNWIRED | — |
| 957 | 124 | (no dust body — the column scan + Next(150) rest roll) | WIRED (`ai_pets_style124`) | g64 dust-wired pets124 survives (d13 row superset) |
| 967/1026 | 28 | `trail_967`/`_1026` | WIRED (`ai_frost_bolt`) | g64 (sole) |
| 1048/1053 | 25 | `trail_1048`/`_1053` | UNWIRED | — |
| 1092 | 18 | `trail_1092` | UNWIRED | — |
| 1114 | 24 | `trail_1114` | UNWIRED | — |
| 1133 | 5 | `trail_1133` | UNWIRED | f48 `ai_cloak_star` survives + takes the 1133 dust half |

**Wired: 24 kinds** (447/450/455/461/503/509/533/574/578/579/596/618/642/
644/650/659/732/813/895/898/954/957/967/1026 — the 650 light pet's scan +
point sparkles ride the same wired `ai_pets_style124` machine). **Unrowed:
21 kinds with their own unwired emitter** (406/419/420/421/422/465/466/
497/537/538/539/573/656/669/723/907/1048/1053/1092/1114/1133) **+ the 2
spark riders 504/979** (they share the 954-gated `trail_504` arm — no
emitter of their own; the FINAL 2 census's "21 unrowed" counts the
own-emitter kinds). The 957 draws NO dust body (its stream draw is the
cycle-end Next(150) rest roll); it is wired for the behaviour-coupled
column scan.

### g63 ≤400 axis (the g63 ledger, `projectile_trail_dust.rs`)

The g63 tip (`2ea3e7110`) wires its rowed ≤400 kinds (batches 1-5) + the
17 unrowed emitters (batch 6) + the 6 owed AI_001 emitters (batch 7:
55/174/176/299/374/376). The 281/174/176 rows stay UNCLAIMED on every
pushed tip (the FINAL cross-activation census above) — their emitters
stay UNWIRED pending the row + machine merges. The g63 axis is DISJOINT
from g64 (the kind gates ≤400 / ≥401 never overlap); no keep-one between
the two per-kind axes. The 357 pulse-bolt seam waits on f20 slice D (the
g63 research/67 contract, nothing owed by g64).

### e3 aiStyle-generic axis (`projectile_dust.rs`, research/125)

The e3 tip (`ac6392cf2`) owns the `:22297-24318` aiStyle-generic window
(styles 2/4/5/6/8/152) + the `:78227-78508` Kill_Bombs dust tail (15
sites). This is a DIFFERENT axis (aiStyle-generic, not per-kind) — no body
overlap with the g63/g64 per-kind arms. The keep-one twins (497/504/503/
723/1133) are the per-kind arms that retire into the e3/f48 machines (the
table above); the e3 generic window itself is KEPT whole. The 503/1037/
1049 rows landed on e3 `80d47a568` (the FINAL 2 constants-copy source).

### The union verdict

The Projectile.cs in-flight trail-dust family is COMPLETE: every per-kind
≥401 trail site is either WIRED (24 kinds) or cited-unrowed (21 own-emitter
kinds + the 2 spark riders), every ≤400 site is g63's (wired or
cited-unrowed), and the aiStyle-generic window is e3's. The three axes are
DISJOINT and all KEEP at merge; the only retirements are the per-kind twins
that feed the surviving e3/f48/d11 machines (the keep-one column). The
Kill-family death sites stay g65's; the one-shot spawn-latch bursts stay
d53/e3/f5's; the wet-entry / HandleMovement / enchant sites stay the
movement/enchant lanes'. NO trail-dust site is unclaimed across the union.
