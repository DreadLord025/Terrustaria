# 26d — Projectile aiStyles 111-120 COVERAGE census (W4-d12)

> Researched 2026-11-20 against `a725acb0` (wave-3 complete). C# cites are
> `decompiled/Terraria/Projectile.cs` (stable); Rust cites are
> `crates/tr-app/src/projectile.rs` / `projectile/data.rs` at this revision.
> Task contract: census the decade, then port the MISSING arms verbatim with
> witnesses (one style per commit). In-flight probe first: d11/d12 local-only
> at base, remote d13/g1 pushed but empty, f19 census covers 1-105 only — no
> live commit claims 111-120.

## Method

- Dispatch census: the `match data.ai_style` block in `ProjectilePool::update`
  (`projectile.rs:16731+`). An aiStyle is PORTED iff it has a live top-level
  arm (the `s1_d_4c_every_rowed_style_has_exactly_one_live_arm` pin's shape).
- C# arm spans: `AI()`'s if/else chain (`:23199-36815`); the 111-120 run sits
  at `:34110-34891`. Kind lists parsed from `SetDefaults`' `else if (this.type
  == N)` chain. `Main.projFrames` from `Main.cs:8439-8727` (init 1, per-kind
  overrides).

## Coverage table (111-120) at base `a725acb0`

P = ported (dispatch arm), M = missing, part = family partial.

| style | state | C# body | kinds (SetDefaults) | Rust dispatch / arm |
| --- | --- | --- | --- | --- |
| 111 | M | `AI_111_DryadsWard` :41134-41240 (dispatch :34110-34113) | 586 (:6358-6366) | — |
| 112 | part | inline :34114-34363 | 836 (:8060-8065), 590 (:6385-6396), 644 (:6941-6952) | :18803 (644 only, `ai_rainbow_crystal_shard`) |
| 113 | P | `AI_113_TargetSticker` :40562-40899 (dispatch :34364-34367) | 598 (:6470-6480); 614/636/971/975/1024 unrowed | :17584 (`projectile_ai_thrown::bone_javelin_step`) |
| 114 | M | inline :34368-34413 | 602 (:6511-6520) | — |
| 115 | P | inline :34414-34437 | 604 (:6071-6083) | :17807 (`ai_yoyo_beam`) |
| 116 | M | inline :34438-34458 | 607 (:6542-6553) | — |
| 117 | M | inline :34459-34637 | 608 (:6554-6566), 612 (:6584-6597), 624 (:6717-6727), 953 (:8988-9000), 1044 (:9875-9889) | — |
| 118 | P | inline :34638-34808 | 617 (:6644-6657) | :17679 (`ai_nebula_arcanum`) |
| 119 | P | inline :34809-34887 | 618 (:6658-6670) | :17699 (`ai_nebula_arm`) |
| 120 | P | `AI_120_StardustGuardian` :44634-44963 (dispatch :34888-34891) | 623 (:6699-6716) | :17354 (`ai_stardust_guardian`) |

**Verdict at base: 5/10 armed (113/115/118/119/120); 112 partial (644 only).
MISSING: 111, 112's 836+590 sub-arms, 114, 116, 117.** All five sit in this
task's claimed decade (branch `task/W4-d12-proj-ai-111-120`), so they port here.

## Findings

- **F1 (113 partial kinds).** `AI_113_TargetSticker` serves 598/614/636/971/
  975/1024, but only 598 has a `proj_data` row + the ported arm. 614
  (StardustCellMinionShot), 636 (Daybreak), 971 (TentacleSpike), 975
  (BloodButcherer), 1024 (DeadCellsKillingDeckCard) are UNROWED — their
  per-type sub-arms (the 614 homing, the 636/971/975 stick-life ladders, the
  1024 return-to-owner state) are out of this decade's arm scope (they are
  minion/butcher/deck-card lanes). Recorded, not ported here.
- **F2 (117's 978 is style 189).** The AI's `flag58 = type==612 || type==953
  || type==978` (:34461) names 978 (Volcano), but 978's SetDefaults row is
  aiStyle **189** (:9253-9267), so it rides the 189 arm (`ai_volcano`), NOT
  117. The 117 arm's flag58 keeps the 978 branch verbatim for source fidelity
  but no 117-rowed kind is 978. The 117 rows are 608/612/624/953/1044.
- **F3 (117 hide / 1044 hide).** 608 and 1044 set `hide = true` at SetDefaults
  (:6565/:9885); the 117 AI sets `hide = true` on the frame wrap (:34477).
  The port's `Projectile` has no `hide` field (draw-layer flag) — documented
  skip, the state machine (counter/scale/frame/alpha/Damage) is the port.
- **F4 (116 setSolar).** The SolarFlareRay reads `player.setSolar`
  (:34446) to hold `timeLeft = 2`. The port had no `set_solar` field; this
  task adds it + wires the Solar armor triple (2763/2764/2765,
  `ArmorSetBonuses.cs:479` → `ApplySetBonus_Solar` `Player.cs:16365-16369`)
  the same flag-only way the Stardust triple landed (`set_stardust`).
- **F5 (114 portal support).** The gate's kill probe
  `PortalHelper.SupportedTilesAreFine` (`PortalHelper.cs:470-551`) + the
  `SupportedSlope`/`SupportedHalfbrick`/`SupportedNormal` helpers
  (`:554-583`) port with the arm — the tile slope/halfbrick/solid membership
  reads `tr_core::tiledata` + `tileobject::tile_cut`.

## Port plan (one style per commit)

1. 111 DryadsWard 586 — row + `AI_111_DryadsWard` (buff 165/186 grants, the
   scale/alpha/rotation ramps, the dust-gate RNG census).
2. 112 836+590 — rows + kind-gated sub-arms (dandelion wind steering, truffle
   spore bob), the 644 path unchanged.
3. 114 PortalGunGate 602 — row + `SupportedTilesAreFine` + the gate arm.
4. 116 SolarFlareRay 607 — row + `set_solar` field/wiring + the orbit arm.
5. 117 explosions 608/612/624/953/1044 — rows + the shared explosion arm +
   the tick-1 resized-box Damage (the `damage_pve_boxed` seam).

RNG contract: the `UpdateProjectiles` stream — every EXPLICIT `Main.rand`
draw at the call sites stays in C# order; the dust/gore BODIES and their
internal `NewDust` rolls go with the dust layer (the `projectile_ai_misc`
precedent, ANALYSIS "Dust & gore"). `Lighting.AddLight` / `hide` /
`ParticleOrchestrator` are the render/particle layer's documented skips.
Witnesses are WRITTEN NOT RUN (code-only round; no cargo per task contract).
