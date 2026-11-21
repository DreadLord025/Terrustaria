# W4-d14 census — projectile aiStyles 131-140

Branch `task/W4-d14-proj-ai-131-140`, base `a725acb0` (wave-3 complete).
Code-only mode: witnesses WRITTEN NOT RUN (no cargo on this branch).
Sources read whole: `Projectile.cs`'s `AI()` else-if chain (`:36310-36526`),
each named `AI_1xx` method, and the `SetDefaults` rows.

## Census table (pre-d14 state → d14 landed)

| style | C# method / block | kinds (SetDefaults) | pre-d14 status | d14 landed |
| --- | --- | --- | --- | --- |
| 131 | `AI_131_FlameBurstShot` `:68456-68508` | 664/666/668 (`:7153-7162` group) | PORTED (`projectile.rs` `131 =>` arm, S1-D-4b-9) | verified — kind guard, latch, 25-fade, tier dust gates, rotation + `kill_step` all match |
| 132 | inline `AI()` block `:36318-36434` | 672 DD2ElderWins (`:7196-7207`) | MISSING — no row, no arm | `b106d9fb` — row + `ai_elder_wins` (4-state machine, gore/dust censuses) + the `NPC.cs:40883` loss-scene spawn |
| 133 | inline `AI()` block `:36435-36498` | 673 DD2SummonSkeleton (`:7208-7220`), 674 DD2DarkMageHeal (same row) | 673 PORTED (`ai_dark_mage_raise`); 674 MISSING (row + heal arm) | `da609513` — 674 row (the 673 twin + ignoreWater overlay) + the 40-tick heal scan |
| 134 | `AI_134_Ballista` `:68510-68651` | 677/678/679 | PORTED (`134 =>` arm) | verified |
| 135 | `AI_135_OgreStomp` `:68671-68787` | 683 DD2OgreSmash (`:7282-7293`), 922 QueenSlimeSmash (`:8745-8756`) | MISSING — no rows, no arm | `e33e111b` — rows + `ai_ogre_stomp` (growing box, tile walk, dust/gore censuses) + the ogre throw aim/jitter fix |
| 136 | `AI_136_BetsyBreath` `:68789-68852` | 687 DD2BetsyBreath (`:7315-7324`) | MISSING arm + WRONG row (f19 F5) | `97c5a984` — row fix + `136 =>` arm + the `NPC.cs:62026` spawn wiring (ai1 = whoAmI, owned) |
| 137 | `AI_137_LightningAura` `:68854-69040` | 688/689/690 (`:7325-7339`) | PORTED (`137 =>` arm) | verified |
| 138 | `AI_138_ExplosiveTrap` `:69042-69388` | 691/692/693 (`:7340-7356`) | PORTED (`138 =>` arm) | verified |
| 139 | `AI_139_ExplosiveTrapExplosion` `:69390-69414` | 694/695/696 | PORTED (`139 =>` arm) | verified |
| 140 | `AI_140_MonkStaffT1` `:69416-69600` | 697 MonkStaffT1 (`:7366-7384`), 707 MonkStaffT3 (`:7474-7490`) | MISSING — no rows, no arm | `6d7b1b9b` — rows + `ai_monk_staff` (both kind arms, the 698 ground-impact spawn, player writes) + the 698 row (aiStyle 141, d15's arm claim) |

## Cross-claim note — the 698 explosion (aiStyle 141)

The 140 arm's ground-impact spawn (`:69500`) creates kind 698
(`MonkStaffT1Explosion`, aiStyle **141**). 141 is d15's range (141-150), so
d14 rows the 698 (so the spawn resolves) but leaves the arm to d15 — the
rowed-but-unarmed deferral (the census §3c pattern, `UNARMED_ROWED` now
`[141, 156, 164, 169, 206]`). d15 lands `AI_141_MonkStaffT1Explosion`
(`:69602+`) on the existing row.

## Finding F5 (f19 verify branch `ec5dec82`) — RESOLVED by d14

`data.rs` rowed 687 as 12×12 aiStyle **1** extraUpdates 1 tileCollide true;
the C# (`Projectile.cs:7315-7324`) is 30×30 aiStyle **136** alpha 255
penetrate −1 hostile, stock tileCollide/extraUpdates/timeLeft. Fixed with
the arm (the F3 pattern — row + arm land together). 686's box was 22×22 vs
C# 30×30 (`:7305-7314`) and its alpha reset 0 vs C# 255 — both fixed. Both
kinds left the `BASELINE_ROWS` reset sweep (alpha 255 overrides).

The 687 spawn site (`npc.rs` `update_betsy`, the ai[1] == 40 dash tick) now
rides `wants_hostile_shot_ai` with `ai1 = self.slot` — the C# passes
`base.whoAmI` (`NPC.cs:62026`) and `Owner: Main.myPlayer`; the drain spawns
it owned + centre-converted (the deathray precedent). The old plain-list
push dropped the payload, which would have left the breath anchored to
NPC 0 or dead on the index guard.

## Deliberate skips (the pool's established conventions)

- Dust/gore BODIES: skipped (no draw layer in the projectile pool); every
  EXPLICIT `Main.rand` draw stays on the `UpdateProjectiles` stream in C#
  order (the `celeb2_explosion` census convention, `projectile_ai_misc`).
  `Gore.NewGore`'s internal kicks stay: `Next(10,31)` then `Next(-20,21)`
  (`Gore.cs:1327-1328`).
- `Filters.Scene` (the 132 crystal-destruction shaders), `PlotTileLine`
  light casts, tracked sounds: no pool layers — cited at each point.
- 136's three `PlotTileLine` CastLight beams (`:68810-68812`) and the
  dust/gore bodies: skipped; the plume/gore-gate/beam-dust draws stay.
- 132's state-1 `velocity.Y` ramp sits INSIDE `!Main.dedServ`
  (`:36367-36371`) — the port keeps the server shape (no velocity write),
  a verbatim source quirk. Rationale: the ramp is a CLIENT-only visual
  rise bundled with the two `Filters.Scene` writes; a dedicated server
  never moves the marker in state 1 (only state 2's unconditional
  `velocity.Y--` integrates). The sim is the authoritative server shape,
  so the whole block skips together; pinned by
  `the_states_advance_and_the_fall_integrates` (`projectile.rs`,
  `vel.y == 0.0` through state 1). See ANALYSIS "W4-d14 doc-sweep — the
  672 state-1 dedServ velocity quirk" and report 119 item 5.
- 135's `KillTile_MakeTileDust` type table feeds only the skipped bodies;
  its exposed-solid path is rand-free (the genRand arms are worldgen-only
  tile kinds, unreachable in the stomp's gate).
- 140's `player.heldProj`/`SetDummyItemTime`/`itemRotation`/`ChangeDir`
  writes ride the `MonkStaffTick` (the `FlailTick` precedent); the
  `reuseDelay` write rides the pool's `pending_monk_reuse` queue drained
  into `app.reuse_delay`; the `DD2_MonkStaffGroundImpact` tracked cue is
  skipped (the pool's sound queue carries only the id-0/2 families, the
  131/134 tracked-sound precedent); `Main.MouseWorld` rides
  `FlailInputs.aim`; `gravDir = 1` (the `ai_158_home_location` precedent).

## Verification (code-only mode)

No cargo/tests/graphify on this branch (the task's code-only constraint).
Each witness is WRITTEN NOT RUN, re-derived from the cited C# block by
hand arithmetic. The dispatch coverage pin
(`s1_d_4c_every_rowed_style_has_exactly_one_live_arm`) gains 132/135/136/
140 as armed and 141 as the documented rowed-but-unarmed deferral.