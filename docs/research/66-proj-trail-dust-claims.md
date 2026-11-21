# 66 — Projectile.cs per-kind TRAIL dust lanes (W4-g63 / W4-g64 / W4-g65): claims + census

Status: CLAIM doc (W4-g64, lane 2). Written early per the lane's coordination
rule — the branch list is the claim record; this doc pins the kind-order split
so the three batches never overlap.

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
| 503 | 5 | 23959 | the :23963 Next(48) GORE gate (d55/g16) |
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
| 596 | 107 | 33623 (the Next(3)!=0 EARLY-RETURN ring) | 33644 (timeLeft<4 death burst → g65) |
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
- **813 — still deferred.** No row on any pushed branch (the d-lane probe:
  d12/d13/f19/f20 carry no 813 const). The 13 sites span four `ai[0]` phase
  windows of the style-108 machine (`:33760/33858/33894/33951/34018`) — a
  whole-machine port, and the port's `ai_vortex_ritual` covers 578/579 only.
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
