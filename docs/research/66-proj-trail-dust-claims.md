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
