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
| 113 | P | `AI_113_TargetSticker` :40562-40899 (dispatch :34364-34367) | 598 (:6470-6480), 614 (:6611-6619), 636 (:6838-6849), 971 (:9197-9206), 975 (:9287-9296), 1024 (:9693-9706) — ALL ROWED (F1 round) | `projectile_ai_thrown::target_sticker_step` (the whole census) |
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

**LANDED (this branch, one style per commit):** 111 (`d159cdfe`), 112's
836+590 (`cf31db11`), 114 (`0cbdd909`), 116 (`f4f41e48`), 117 (`f0150a69`).
The decade is now 10/10 armed (112's three kind sub-arms all live). The
608 Solar Counter shares 117's row but its spawn sites `Kill()` it
immediately, so its burst rides the Kill path (`:72177-72225`), ported
with the 117 commit. Witnesses WRITTEN NOT RUN (code-only round).

## Findings

- **F1 (113 partial kinds) — RESOLVED, the whole census ported.** The EXACT
  SetDefaults census (`this.aiStyle = 113`, six writes): 598 BoneJavelin
  (:6474), 614 StardustCellMinionShot (:6615), 636 Daybreak (:6842), 971
  TentacleSpike (:9201), 975 BloodButcherer (:9291), 1024
  DeadCellsKillingDeckCard (:9697). The shadowflame-family guess is WRONG:
  495 ShadowFlameArrow is aiStyle **1** (:5681, rowed), 496 ShadowFlame is
  aiStyle **91** (:5692, W4-d10's decade), 659 SpiritFlame is aiStyle **129**
  (:7083, rowed) — none rides 113. The five missing rows ported verbatim
  (`928e6bbf`); `bone_javelin_step` generalized into `target_sticker_step`
  serving all six (`719ffb76`): the 614 chase lerp + 63-fade, the 636
  0.995/0.15 fall + 5·MaxUpdates stick, the 971/975 9·MaxUpdates sticks +
  pulse censuses + rotation latches, the 1024 three-state machine (frames,
  sine flutter, the 4×4 tile-stick probe, the return at 16 with the inline
  ResetLocalNPCHitImmunity+Damage, the +6 hitbox + ×1.5 return multiplier).
  Pool: the pin/caps (6/8/10)/debuffs (189/337/344/183) generalize, the
  `Damage_CanDealDamage` stuck gate (:12448), the 614/636 Colliding 1/8
  shrink. Kill paths (`85054785`): the 636 spawns the 953 (:72126-72145),
  the 1024 Item8 + ring (:74817-74829); the spawn rotation latch
  (:22049-22056) landed (`9a493123`). The 612/953 Daybroken grants
  (:11355-11366) ride both damage walks. Witnesses WRITTEN NOT RUN.
- **F2 (117's 978 is style 189).** The AI's `flag58 = type==612 || type==953
  || type==978` (:34461) names 978 (Volcano), but 978's SetDefaults row is
  aiStyle **189** (:9253-9268), so it rides the 189 arm (`AI_189_Volcano`
  :42264), NOT 117. The 117 arm's flag58 keeps the 978 branch verbatim for
  source fidelity but no 117-rowed kind is 978. The 117 rows are
  608/612/624/953/1044. **F1-round census: 978 has NO `proj_data` row on
  this branch.** Style 189 is W4-d19's decade (181-190); at the census both
  `task/W4-d19-proj-ai-181-190` and `task/W4-d20-proj-ai-191-200` sat at
  `a725acb0` (= origin/main, ZERO pushed commits — no claims read), so the
  978 row + the Volcano arm stay d19's to land. The 978 StatusNPC term
  (OnFire 24/180, :11359-11362) ports verbatim with the explosion grants.
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

## F1 follow-up round — the explosion-row audit

All six explosion kinds the decade armed carry `proj_data` rows (the
`spawn_with_data` rowless-decline audit, pinned by
`w4_d12_f1_the_explosion_kinds_all_have_rows`): 612/624/953/1044 (aiStyle
117, data.rs), 608 (117, the Kill-path counter), 644 (112, the crystal
shard). NONE is rowless. The 978 flag58 term is the only style-117-adjacent
kind without a row — deliberately, it is d19's 189 (see F2).
