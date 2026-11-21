# Projectile aiStyles 101-110 — census + port (W4-d11)

Branch `task/W4-d11-proj-ai-101-110`, base `a725acb0` (main). Code-only round:
the witness tests are WRITTEN NOT RUN (no compiler invoked on this branch);
each was verified by re-reading the cited C# block and hand arithmetic.

Source of truth: `decompiled/Terraria/Projectile.cs`'s `AI()` if/else chain.
The dispatch is `ProjectilePool::update`'s `match data.ai_style`
(`crates/tr-app/src/projectile.rs`); the rows are `projectile/data.rs::proj_data`.

## Census (the state on the base, before this branch)

| aiStyle | C# block | kinds | base state |
| --- | --- | --- | --- |
| 101 | `:33165-33221` | 536 MedusaHeadRay, 591 MinecartMechLaser | MISSING (no rows, no arm) |
| 102 | `:33222-33447` | 539 StardustJellyfishSmall, 573 NebulaBolt, 574 NebulaEye | PARTIAL — the 574-only `ai_nebula_eye` stand-in (no release volley, no dust census, no frames, no 539/573) |
| 103 | `:33448-33477` | 540 StardustTowerMark | PORTED (`ai_stardust_mark`) — row alpha 0 vs C# 255, spawn anchor top-left vs Center.X/Bottom.Y, missing the negative-counter early return |
| 104 | `:33478-33511` | 565 BrainOfConfusion | MISSING |
| 105 | `:33512-33515` → `AI_105_SporeSac` `:40043-40192` | 567 SporeTrap, 568 SporeTrap2 | MISSING |
| 106 | `:33516-33552` | 569/570/571 SporeGas | MISSING |
| 107 | `:33553-33678` | 575 NebulaSphere, 596 DesertDjinnCurse | MISSING |
| 108 | `:33679-34025` | 578 VortexLightning, 579 VortexPortal, 813 BloodNautilusTears | PARTIAL — the 578/579 core (`ai_vortex_ritual`); the 813 kind + its dust-gate census missing |
| 109 | `:34026-34065` | 582 MechanicWrench | MISSING |
| 110 | `:34066-34109` | 584 NurseSyringeHeal | MISSING |

In-flight probe: no other live branch claims 101-110 (d10 = 91-100,
d12 = 111-120, g1-g3 at base, f19 verify-only). The 535 MedusaHead (aiStyle
100) is d10's claim — the 536 ray's `key_parent(ai[0], 535)` probe reads
None until it lands (the ray dies like the C#'s failed `TryGetActive`).

## What landed (one style per commit)

- **101** `6aca9b7f` — `ai_medusa_ray` (the 536/591 rows, the 20-tick
  `(int)Lerp(0,255,t/20)` fade, the key-linked head anchor / the player-delta
  rider + opposing-travel kill). ZERO rand draws.
- **102** `2ba79d55` — `ai_tower_orb` replaces `ai_nebula_eye` (the 539/573/576
  rows, the 574 penetrate 1 fix, the per-kind param table, the host drag +
  sign-homing 6px clamp, the dust-gate census + frames 539=4/574=2, the
  release jitter `Next(-50,51)`×2 + `Next(80,121)`×2, the LOS-gated 576 volley
  + ai[0] reset).
- **103** `df96ce48` — `ai_stardust_mark` fixes (the 540 row alpha 255, the
  Center.X/Bottom.Y spawn anchor, the `!(ai[0]>=0f)` early return). The 540
  `UpdatePosition` skip (`:20435`) already held (no kill_step).
- **104** `0f4f5938` — `ai_brain_of_confusion` (the 565 row + 4-frame sheet,
  the grow 1.01/fade-7 → shrink 0.995/fade+3 legs, the 250 kill w/ the tail
  running on the kill tick, the 7-tick frame cycle, the vy-=0.03/vx*=0.97
  drift). ZERO rand draws.
- **105** `f1128752` — `ai_spore_sac` (the 567/568 rows +
  `UseGroupStaticImmunity(567)`, the `Player.sporeSac` field + ResetEffects
  clear, the ±90-tick breathing scale, the IndexForVisuals%6 drift table +
  240-tick cycle, the 5400 age + distance ladder + unequipped +100, the
  Manhattan-340 chaseable homing 4/41 blend / 0.98 drag). ZERO rand draws.
- **106** `2b9204a3` — `ai_spore_gas` (the 569-571 rows +
  `UseGroupStaticImmunity(569)`, the vel.X*0.02 ±|vel.Y|*0.02 spin, the 0.98
  drag, the 60-tick alpha gate -30→80 / +5→255 + owner kill). ZERO rand draws.
- **107** `e098fb42` — `ai_nebula_sphere` (the 575/596 rows + 4-frame sheets,
  the player-index homing (v*9+dir*5)/10 (596 accel 7.5) past 40px, the orphan
  cap tl30 + ai[0]=-1 netUpdate latch, the solid-collision alpha-128 pin, the
  596 three Next(3) gates w/ early RETURN + the tl<4 60x60 flash dmg 30/22 +
  ten SelectRandom Next(2) picks).
- **108** `f05eae46` + `be58df9e` — the 813 completion (the shared 32x32 row,
  the kind-gated 619 BloodSquid spawn at 90 w/ raw-id intent cited-skip, the
  per-stage dust-gate census Next(4)×2/Next(2)×2/Next(2) if-else/Next(3)×2 +
  the opacity NextFloat gate + NextVector2Circular pair; flag53 excludes the
  813). The 578/579 core stays byte-identical (their dust gates stay the
  recorded skip); the dissolve `Kill()` no longer returns (the 813 death-tick
  gates roll; a 578/579 no-op).
- **109** `96654865` — `ai_mechanic_wrench` (the 582 row MaxUpdates2, the
  localAI[1] speed latch, the 30-tick outbound wait, the Mechanic-124 host
  return leg, the pi/10 spin + the HandleMovement aiStyle-109 tile bounce
  `:19512-19571` reverse-both + ai[0]=1 + netUpdate + Dig thud, own movement
  no kill_step). `lazyNPCOwnedProjectileSearchArray` cited skip. ZERO rand.
- **110** `28d3b2ff` — `ai_nurse_syringe` (the 584 row pen3 scale0.8, the
  localAI[1] speed latch, the town-NPC host gate `is_town_npc_like`, the catch
  heal min(lifeMax-life,20) positive-only, the seek 0.04 lerp w/ Y floor
  max(value.Y,vel.Y) then ++ + vel.X*0.05 spin). HealEffect combat-text cited
  skip. ZERO rand draws.

Witness fixes: `cdd2b937` (the 105 homing blend expects the computed
zombie-centre direction, self-consistent with the Manhattan scan).

## Deliberate deviations (recorded, not bugs)

- The dust BODIES skip everywhere (no draw layer in the pool); the explicit
  `Main.rand` GATE rolls stay on the `UpdateProjectiles` stream in C# order
  (the `projectile_ai_misc` precedent). The 578/579 dust gates stay the
  pre-existing recorded skip (their witnesses pin the current stream).
- `Lighting.AddLight` feeds the render light layer only — skipped.
- The 582's `lazyNPCOwnedProjectileSearchArray` latch (`:34050`) and the 584's
  `HealEffect` combat-text (`:34089`) are cited skips (the seams are unported).
- The 813's 619 BloodSquid spawn carries the raw C# NPC id (the kind is
  unported — the `drain_mark_spawns` `from_u16` folds it).
- The single-player fold: every `Main.player[target]` / `Main.player[owner]`
  read maps to the local player; `Main.netMode != 1` is always true.
- The 102 host drag rides `old_x/old_y` (the one-tick latch) vs the C#'s
  `oldPos[1]` (two-tick) — the recorded stand-in the `ai_nebula_eye` precedent
  carried, kept.

## Coverage after this branch

All ten aiStyles 101-110 have exactly one live dispatch arm (the
`s1_d_4c_every_rowed_style_has_exactly_one_live_arm` pin passes: 103 arms,
no duplicates). Every rowed kind in the range is armed.
