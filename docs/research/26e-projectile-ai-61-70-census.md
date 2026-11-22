# 26e — Projectile aiStyles 61-70 dispatch census (W4-d7)

> Researched 2026-11-20/21 against `a725acb0` (wave-3 complete). C# cites are
> `decompiled/Terraria/Projectile.cs` (stable); Rust cites are
> `crates/tr-app/src/projectile.rs` / `projectile/data.rs` at this revision.
> Cross-checks: f19's coverage census (`26c`, `origin/task/W4-f19-proj-ai-verify-1`)
> rows 61-70 agree — this lane (d7) owns the decade and the two MISSING styles
> (63/69); d3/d5/d6 probed in-flight (d3 = census-only 21-30, d5 = the 41-50
> unwired-machine pattern this task mirrors, d6 zero commits beyond main at
> probe time).

## Census

| style | C# arm | kinds (SetDefaults) | Rust arm | Verdict |
| --- | --- | --- | --- | --- |
| 61 | `AI_061_FishingBobber` 53658-53861 (dispatch 29310-29313) | bobbers 360-366/381/382/760/775 (`:4508-4516`) + 986-993 (`:9426`) | `projectile_bobber::bobber_step` (:153), dispatch 19608 | PORTED — verify-only (F1) |
| 62 | `AI_062` (dispatch 29314-29317) | 373 Hornet (`:4563`), 375 FlyingImp (`:4587`), 407 Tempest (`:4883`), 423 UFOMinion (`:4965`), 613 StardustCell (`:6603`), 963 Abigail (`:9112`) | `ai_flying_minion` (:1066) + `minion_separation` (:1015), dispatch 19361 | PORTED — verify-only (F2) |
| 63 | inline 29318-29418 (owner-inactive kill + closest-LOS anchor machine + StepUp + walk frames) + HandleMovement 18453-18463 + StatusNPC 11490-11493 + Kill 75522-75532 + the 378 egg's hatch 77240-77253 | 379 BabySpider (`:4635-4644`: 14×10, friendly, timeLeft 300, penetrate 1, decidesManualFallThrough) | **none** — no arm, no `proj_data` row (the DW-1.1 pin :36585 asserts 379 rowless; `SENTRY_SHOT` lists it, data.rs:4505) | **MISSING — ported by this task** |
| 64 | inline 29419-29533 (sharknado clouds) | 384 ForceBubble (`:4664`), 386 Cthulunado (`:4686`) | `ai_sharknado_cloud` (:4232), dispatch 19561 | PORTED — verify-only |
| 65 | inline 29534-29603 (sharknado bolt) | 385 SharknadoBolt (`:4676`) | `ai_sharknado_bolt` (:4353), dispatch 19568 | PORTED — verify-only |
| 66 | inline 29604-30161 (Twins + Deadly Sphere) | 387 Retanimini (`:4698`), 388 Spazmamini (`:4719`), 533 DeadlySphere (`:6002`) | `ai_twins_minion` (:1791) + `twins_separation` (:1750), dispatch 19419 | PORTED — verify-only (F2) |
| 67 | `AI_067_FreakingPirates` 49121-51201 (dispatch 30162-30165) | 393-395 pirates (`:4755`), 500 CrimsonHeart (`:5417`), 653 CompanionCube (`:7016`), 758 VampireFrog (`:7818`), 833-835 StormTigers (`:8086-8114`), 951 Flinx (`:8973`), 1018 DirtiestBlock (`:9641`), 1022 MushroomBoi (`:9671`), 1036 CobWhipSpider (`:9771`), 1056 BoulderPet (`:9991`), 1090 RainbowBoulderPet (`:10167`), 1093/1112 Cattivas (`:10199`/`:10366`), 1118 ClayPot (`:10454`) | `ai_freaking_pirates` (:11524) + `pirate_movement` (:12305) + `projectile_ai067_b::ai_freaking_pirates_pets`, dispatch 17632 (kind-gated) | PORTED — verify-only, gated (F3) |
| 68 | inline 30166-30239 (spin/fade/gravity/prime) + HandleMovement 19780-19786 | 399 MolotovCocktail (`:4801`), 669 Ale (`:7163-7173`) | `projectile_ai_thrown::molotov_step` (:138), dispatch 17574 — 399 only | PORTED — verify-only (F4: the 669 half is rowless) |
| 69 | inline 30240-30303 (owner-facing rotation, dead kill, SetDummyItemTime(10), the 400/600/1500 recall ladder, the every-4th-tick bubble spawn) + the hit latches 14166-14170 / 14709-14713 + HandleMovement 19512-19571 | 404 Flairon (`:4847-4856`: 26×26, friendly, penetrate −1, alpha 255, melee) | **none** — no arm, no row. NOT the port's `FLAIRON` const (1058 = FlaironFlail, the aiStyle-15 flail body, data.rs:613); item 2611 Flairon shoots **404** (`Item.cs:25528`, the port's itemdata row already carries `projectile: Some(404)`), item 5526 FlaironFlail shoots 1058 (`Item.cs:43534`) | **MISSING — ported by this task** |
| 70 | inline 30304-30354 (Flairon bubble) | 405 FlaironBubble (`:4857-4867`) | `ai_flairon_bubble` (:10500), dispatch 17546 | PORTED — verify-only (F5) |

## Verify-only notes (ported arms, no behaviour change)

- **61** — the flight/bite machine rides `projectile_bobber.rs` (S1-P-i35) on
  the per-tick `bobber_inputs` snapshot; the dispatch integrates via
  `kill_step` and drains `wants_splash`. The movement-61 bounce is F1.
- **62** — the same-type separation is pre-computed pool-side
  (`minion_separation`, the 0.05/manhattan pair), the attack-shot request
  rides back through the pool (`spawn_with_data_ai_owned` + the 300-tick
  override, `:66124`); `UpdatePosition`'s shared integrate runs outside the
  AI (aiStyle 62 not in the `:20435` skip list). The wet-velocity branch and
  the slope correction stay unported (F2, the dispatch comment's ANALYSIS).
- **64** — the scale ladder, the alpha ramp, the segment re-spawn and the
  shark spawn (372/373 NewNPC) ride `wants_shark_spawn` out to the world sim
  (S1-D-3 W-9); verbatim.
- **65** — the aimed chase (`ai[1] = target+1`, the single-player slot-0
  fold documented in the arm) and the free cosine bob are verbatim; the wet
  kill (`pos.y -= 16` + Kill) and the death-cloud request
  (`wants_bolt_death_cloud`) are wired.
- **66** — the Twins' dash arms (the `get_chase_results` interception), the
  533's orbit and the 387's bolt (damage × 1.15 truncation + the 300-tick
  override, `:30105-30106`) are verbatim; the slope correction + the 533
  bounce stay unported (F2).
- **67** — the pirate trio + the Vampire Frog (slice A, S1-D-4b-7) and the
  three RNG-light pets 500/1018/1056 (slice B, S1-D-4b-12) are verbatim incl.
  `pirate_movement`'s manual fall-through + hoik loop; the rest of the
  16-kind family stays gated (F3).
- **68** — the 399's spin/3-tick fade/15-tick gravity/80×80 prime and the
  stop-and-prime collision are verbatim (`molotov_step`); the `alpha == 0`
  fire-dust window (`:30191-30207`) is the documented dust skip (zero kept
  draws — the 399 branch draws nothing at the call site); the wet prime
  folds dry (the pool's no-wet-flag convention, recorded in the arm doc).
  The 669 Ale half is F4.
- **70** — the one-shot closest-LOS acquire (650 px), the
  `Next(35,75)/30` + two `Next(-100,101)` jitter first-steer, the
  `(v*40+d*12)/41` homing, the −15 no-target fade and the `Next(80,121)/100`
  scale roll are verbatim in the C#'s draw order (S1-D-4b-1); the bubble's
  tile response is F5.

## Findings (owed to the wiring/row rounds, NOT fixed here)

- **F1 (style 61)** — `HandleMovement`'s aiStyle-61 arm (`:19615-19625`:
  `vx = lastVx * -0.3` on an X-block, `vy = lastVy * -0.3` on a Y-block with
  `lastVy > 1`) is folded into the dispatch's `kill_step` kill-on-tile
  stand-in — the recorded M4-PROJ movement deviation shape. The bobber's
  reel state (`tileCollide = false`) integrates freely, faithful.
  **LANDED (W4-d7 F1)**: `bobber_movement` + `bobber_bounce` replace the
  stand-in (the dry TileCollision + the hoik loop + the `Next(3)` prologue
  draw + the 0.3 bounce, NO Kill); ANALYSIS "The fishing rod use chain".
- **F2 (styles 62/66)** — `UpdatePosition`'s slope-correction block
  (`:20447-20474`, the 62 fall case + the 317/387/388/533/759/1098 type
  list) and the 533's blocked-move bounce (`:18173-18205`) stay unported —
  the dispatch comments' existing ANALYSIS entries, unchanged.
- **F3 (style 67)** — the dispatch gate (:17632-17671) admits the pirate
  trio + frog + the 500/1018/1056 pets only. The 653 CompanionCube and the
  1090 RainbowBoulderPet stay deferred (their draws steer observable state,
  `projectile_ai067_b` module doc); the Storm Tigers 833-835 (the
  special-attack chain), the 1036 CobWhipSpider (rowless), the smart-cursor
  miners and the remaining pets keep the no-op fall-through — the gate
  comment's documented deferrals. The 951/1022/1093/1112/1118 rows exist
  (S1-P-i25) but are AI-INERT behind the gate until their arms land.
- **F4 (style 68)** — the **669 Ale** (the Ale Throwing Glove item 3821's
  shot, `use_ammo = 353` Ale) has NO `proj_data` row, so the whole kind is
  inert. Owed to the row round, verbatim census: the row (`:7163-7173`:
  14×14, aiStyle 68, friendly, alpha 255, melee, noEnchantmentVisuals,
  penetrate −1); the AI's `flag35` dust branch (`:30209-30220`: 2× dust 4,
  each ONE `NextFloat` lerp + ONE `NextFloat` scale + the `SelectRandom`
  colour pick — three draws per dust); `StatusNPC`'s Tipsy roll
  (`:11314-11317`: `Next(20) == 0` gate then buff 31 `60 * Next(2,6)`);
  `StatusPlayer`'s two arms (`:11798-11801` / `:12142-12145`, the same
  roll); `Damage_PVE_Inner`'s 669 arm (`:13135-13148`:
  `damage = (int)((double)damage * 0.8)`, `timeLeft = 1`, direction from
  the OWNER centre vs the victim — note it sits in the `:13051` else-if
  chain AFTER the aiStyle-16 arm, so the generic 68 fuse (`:13149-13163`)
  never runs for 669); `GetCollisionParams`' 10×10 box (`:19947-19951`);
  the shimmer group membership (`:22424`); and `Kill()`'s 669 arm
  (`:71952-71985`: sound 13, 12× dust 4 (no call-site draws) + 80×2 dust 4
  (each ONE `Next(10,71)` / `Next(10,41)` velocity draw), `Resize(144,144)`,
  Tipsy 25/3659 to the local player + every intersecting town NPC, then
  `Damage()`).
- **F5 (style 70)** — the bubble's `HandleMovement` type-405 arm
  (`:18317-18335`: `localAI[0]++`, the first bounce reflects both changed
  axes, the second `Kill()`s) is folded into `ai_flairon_bubble`'s
  `kill_step` (kill-on-tile) — a stand-in deviation the arm doc records as
  the terminal default. The `Kill()` 405 hook (`:72849-72873`: sound Item54
  + 20× dust 212, each ONE `NextDouble` hue draw — the `num286 < 30` branch
  is always taken so the `Next(45,91)` arm is dead) has no port hook; the
  20 kept draws + the cue are owed to the death-hook round.
- **F6 (styles 63/69)** — the two MISSING arms, ported by this task below.

## The missing styles — ported by this task (UNWIRED, code-only)

Per the d5 (`26d`) / `npc_ai_style25` pre-wiring precedent: one pure module
per style, keyed on the vanilla type, with the `SetDefaults` rows as module
consts (NOT `proj_data` rows — adding rows would arm the dispatch's
`_ => {}` fall-through for kinds whose wiring is a later round, and the
DW-1.1 pin :36585 asserts 379 rowless). The wiring step adds the rows + the
dispatch arms + the movement/death-hook seams.

- `projectile_ai_style63.rs` — BabySpider 379: the verbatim
  `Projectile.cs:29318-29418` arm (the owner-inactive kill, the 2000-px
  closest-`CanHit` anchor scan, the no-target 0.95 drag + fall-through
  clear, the `shouldFallThrough = target.Y > Bottom.Y` latch, the grounded
  5/0.08 vs airborne 8/0.12 steering pair + the −6 hop when the target sits
  50 px above a grounded spider, the `Collision.StepUp` with a FRESH local
  `stepSpeed = 0f` (`:29379-29380` — NOT `this.stepSpeed`), the 4-frame
  walk machine (airborne frame 3; the 9-wrap counter's 0/1/2 bands), the
  direction/spriteDirection flip and the +0.2/16 gravity tail); the
  HandleMovement bounce `:18453-18463` (X × −0.6 on any X-block, Y × −0.6
  only when `lastVy > 2`); `StatusNPC`'s Venom roll `:11490-11493`
  (buff 70, `60 * Next(2,5)` — kept); the Kill dust census `:75522-75532`
  (5× dust 171, each ONE `Next(1,10)` scale draw — kept, bodies skipped);
  and the 378 SpiderEgg's hatch producer `:77240-77253` (the owner-gated
  `Next(2,4)` count + the `Next(5) == 0` bonus, per-spawn X-then-Y
  `Next(-20,21) * 0.01` velocity jitter — the named-argument evaluation
  order). RNG-free AI; the kept draws ride the `UpdateProjectiles` stream.
  379 is NOT `Main.projPet` (no `projPet[379]` write) — the fall-through is
  the arm's own `shouldFallThrough` (`decidesManualFallThrough`, `:4643`),
  and `CanCutTiles` excludes it (`:15329`).
- `projectile_ai_style69.rs` — Flairon 404 (the recall head, NOT the 1058
  flail body): the verbatim `Projectile.cs:30240-30303` arm (the
  owner-facing `rotation = ToRotation() − 1.57f` BEFORE the dead check, the
  dead `Kill()` + return, `SetDummyItemTime(10)` + `ChangeDir` +
  `itemRotation = (−v * direction).ToRotation()` as player-write intents,
  the `spriteDirection` sign, the 400-px arm latch, the 1500-px cut / the
  600-px state-2 escalation / the 20→40 recall speed + `tileCollide = false`
  / the arrival kill, the `ai[1]++` counter + the >5 alpha-0, and the
  local-owner every-4th-tick 405 spawn — `Next(45,65) * 0.1` speed then ONE
  `NextDouble` jitter, `RotatedBy((d − 0.5) * 1.5707963705062866)` via the
  `bow_launch::rotated_by` double seam, `ai0 = −10f`); the hit latches
  `:14166-14170` / `:14709-14713` (`ai[0] = 1` + netUpdate) and the
  HandleMovement `:19512-19571` group's 69 half (the `ai[0] = 1` latch +
  netUpdate, NO velocity flip — the flip is the 3/109-only arm
  `:19564-19568`; the HitTiles dust + sound 0 are the documented skips).
  `AutomaticallyChangesDirection` excludes 69 (`:17283`) — the arm owns the
  direction. The item side is already faithful: itemdata 2611 →
  `projectile: Some(404)`; only the row + the arm are missing.

Witnesses: per-module tests WRITTEN NOT RUN (no compiler/tests on this
branch, the task contract); verified by re-reading the cited C# blocks and
hand arithmetic.
