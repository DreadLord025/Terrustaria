# 164 — Projectile aiStyles 101-110 — census + port (W4-d11)

Renumbered `26d` → **164** per the d54 collision registry (`150`, `38bfd1d22`,
§1 key `26d`: W4-d12's 111-120 census first-add 00:06:06 wins the key, this
lane's 04:17:27 copy takes the assigned free 164; the 164 slot re-verified
unclaimed at every pushed tip at the RETIRE-round scan).

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
  `HealEffect` combat-text (`:34089`) were cited skips — both landed in the
  DOC-SWEEP phase (`af760b3b4` the pool cache + the `:53166-53179` re-derivation
  + the `:55214-55217` second-throw suppression; `c1e686f26` the
  `wants_heal_text` → HEAL_LIFE drain on the `UpdateProjectiles` stream).
- The 813's 619 BloodSquid spawn carried the raw C# NPC id (the kind was
  unported) — `eae7b5097` landed the 619 kind row (`NPC.cs:16934-16949`) +
  AI_005's 619 branches, so the 90th-tick spawn intent now resolves the real
  kind instead of the `from_u16` Slime fold.
- The single-player fold: every `Main.player[target]` / `Main.player[owner]`
  read maps to the local player; `Main.netMode != 1` is always true.
- The 102 host drag rides `old_x/old_y` (the one-tick latch) vs the C#'s
  `oldPos[1]` (two-tick) — the recorded stand-in the `ai_nebula_eye` precedent
  carried, kept.

## Coverage after this branch

All ten aiStyles 101-110 have exactly one live dispatch arm (the
`s1_d_4c_every_rowed_style_has_exactly_one_live_arm` pin passes: 103 arms,
no duplicates). Every rowed kind in the range is armed.

## DOC-SWEEP phase (the nebula-sweep continuation)

After the census, the branch carried a DOC-SWEEP wave (the 619 BloodSquid kind
row `eae7b5097`, the 582 wrench latch `af760b3b4`, the 584 heal-text seam
`c1e686f26`) and the Nebula set's magic on-hit pickup proc — the producer
(`121672908`: `setNebula`/`nebulaCD` + the `(2760,2761,2762)` Benefits.Nebula
arm, the `canGhostHeal` gate, the `Projectile.cs:13826-13830` proc with its
four rolls, the buff lifecycle `NebulaLevelup`/`UpdateBuffs_NebulaBuffs`/
`ApplyNebulaBuffMana`) and the completion (`ab42de2d7`: the
`wants_nebula_pickup` drain + the world-sim drop spawn, the `aiStyle != 59`
gate fix). The proc's pickup drop is DRAW-FREE (the explicit-velocity arm);
the four hit-site rolls ride the `UpdateProjectiles` stream in C# order.

### Sibling-lane dedupe notes (merge keep-one guidance)

- **The nebula buff lifecycle overlaps d79 (`b78b02022`) / f49 (`a597ce2c5`) /
  d117 (`0dbcb0136`).** Those unmerged siblings port the SAME
  `nebulaLevelLife/Mana/Damage` + `nebulaManaCounter` fields, `nebula_levelup`,
  `update_buffs_nebula`, `apply_nebula_buff_mana` and the `:10444-10461` buff
  arms. d11's copies are functionally identical (verified line-by-line against
  the C#); the merge keeps ONE writer. d11 adds the pieces the siblings lack:
  `setNebula`/`nebulaCD` + the `(2760,2761,2762)` set arm + the
  `Projectile.cs:13826-13830` PRODUCER proc + its drain. d79/f49 own the
  CONSUME arm (`PickupItem` `Player.cs:35045-35058`, the positioned case-7 Grab
  via `play_grab_sound`) and the 12/5 pull (`:34953-34956`) — d11 does NOT port
  those (the consume side stays the sibling lanes'). The producer (d11) and the
  consumer (d79/f49) are complementary, not duplicates; only the lifecycle
  fields/methods need keep-one.
- **d77c armed cue 539 (id-4 style-7) + 576 on d11.** The d77 Kill-host wave
  (`0573b70f2`) arms the 539 StardustJellyfishSmall's derived centre-40
  `NPC_Killed_7` cry and the 576 NebulaLaser cue on d11's kinds via
  `kill_tail_cue`. d11's nebula-sweep WIP touches the 539/573/574/576 AI rows
  (`ai_tower_orb`, `2ba79d55`) and the nebula pickup proc — DISJOINT from the
  Kill-tail sound cues (the proc is a Damage_PVE_Inner on-hit effect, the cues
  are Kill()-tail). No overlap: d11 carries no `kill_tail_cue` map at all
  (verified — the seam is d77's, `0573b70f2`), so it adds no competing 539/576
  cue. Merge keeps d77's cue arms; d11's AI-row + proc work is disjoint.

## RETIRE round (the five-item closeout — research/245 is the FINAL)

1. **Git-log census, band 101-110.** `origin/main` (`051600da8` at the scan)
   carries NONE of the band: no `projectile_ai_101_110.rs` (ls-tree empty),
   zero `ai_medusa_ray`/`ai_tower_orb`/`ai_nurse_syringe` hits (grep exit 1),
   no nebula-sweep commit in `git log origin/main --grep=nebula` (the hits
   are the i40 wings / ManaV2 lanes). The nebula-sweep landed ON THIS BRANCH
   ONLY: producer `121672908` + completion `ab42de2d7` + census `2894f6f7d`.
   NOT landed on main; the branch is the sole carrier.
2. **d77c cues 539-id4-style7 / 576 compose CONFIRMED.** d77 tip `5f10560b2`'s
   `kill_tail_cue` (projectile.rs:20211) arms `539 => (4, 7, centre−40)` —
   the C# `:72565-72571` re-read: the 80-box recentre (`position=Center`,
   `width=height=80`, `Center=position`) then `Damage()` then the id-4
   style-7 play, so centre−40 is the derived read; the `NPC_Killed_7` bank
   row + int-facade style-7 arm live in d77's audio.rs (:154-155/:1443).
   The 576 rides the common-bullet list (`:76038` re-read: 576 IS on the
   24-kind arm) `=> (2, 10, pos)` at :20440. d11 carries NO `kill_tail_cue`
   and NO competing 539/576 cue push (grep census: the band module's only
   sound-adjacent text is the module-doc pointer); d11's rows (539
   `data.rs:2508`, 576 `:2517`) are exactly what d77's comment cites
   (`e0395afc`). f20 `7d6e2b72` ports the 576 AI_001 overlay ROW-GATED INERT
   until d11 merges — the three-lane chain (d11 row → f20 overlay → d77
   Kill cue) composes with zero overlap.
3. **813 three-way keep-one (g64 × d11 × d13).** g64 `3cdc64236`'s dust-wired
   `ai_vortex_ritual` (5-arg, `trail_813_windup/_charge/_full/_dissolve` +
   the opacity tail on the live DustPool) is the STRICT SUPERSET and
   SURVIVES: it already absorbed d11's `be58df9e` dissolve-no-return fix and
   d13's `b56db900` wind-up scale fix (`scale = self.scale` — the C#
   `:33714-33783` writes NO scale; re-read confirms) + the `a2588cd2`
   once-per-death Kill head + the Item8 tick-90 cue + the `(int)Center`
   truncation (`:33884` re-read: `(int)base.Center.X/Y` — g64 keeps the C#
   verbatim). d11's gates-only machine + d13's machine DROP at merge. THE
   ROW: d11's `-1`/`RESET_STATE` (alpha 0) was stale — the C# `:6282-6293`
   block sets alpha 255 + friendly + hostile + ignoreWater + drawLayer 1
   and NEVER sets penetrate (stock 1, `:774` re-read). This round corrects
   d11's row + the `:6290` ignoreWater flag-list entry to the g64/d13 shape;
   the tuple is now BYTE-EQUAL across all three tips (string-compare
   verified) — the `w4_d11_retire_the_marker_row_is_byte_equal_across_the_three_way`
   witness pins it (WRITTEN NOT RUN). The 619 spawn resolves through d11's
   `NpcKind::BloodSquid` row (`eae7b5097`) at merge — g64's raw-619 intent
   comment ("kind unported") is stale against d11's landed row; g64's own
   FINAL 1 already records this resolution.
4. **All-armed/rowed census + remaining gaps.** All ten styles 101-110 have
   exactly one live dispatch arm (projectile.rs:18050-18237: 101/102/103/
   104/105/106/107/108/109/110 — one each). All 20 rowed kinds resolve:
   536/591→101, 539/573/574→102, 540→103, 565→104, 567/568→105,
   569/570/571→106, 575/596→107, 578/579/813→108, 582→109, 584→110, plus
   the 576 laser row→1 (f20's overlay). Remaining gaps (all cited, none
   owed by this lane): the 535 head anchor waits on d10's merge (d10 tip
   `72431e61f` HAS the 535 row :1806 + the 100 arm :18339 — the probe
   resolves at merge); the 578/579 charge/full/dissolve dust arms + the
   579 wind-up bodies are g64's axis (its machine wires them); the 813
   dust BODIES ride g64's superset; the 596 keep-one is d11's
   `ai_nebula_sphere` + g64's `trail_596` bool seam (g64 FINAL pins it);
   the 582/584/619 cited skips all landed in the DOC-SWEEP wave.
5. **research FINAL + declaration** → `245-w4-d11-retire-final.md` (the
   number per the registry-150 first-free scan at the RETIRE tip: 244
   taken f39/g48, 245 free at all 361 tips; the 246/247 seats were claimed
   by d10/g40/g43 mid-round — the race the registry documents). The
   declaration: the band-101-110 family is d11-owned wholesale; the nebula
   PROC producer is d11-unique (360-tip grep census: only d11 carries
   `set_nebula`/`nebula_cd`/`:13826`/`wants_nebula_pickup`; d6's hit is the
   `can_ghost_heal` FIELD shape, see the FINAL's keep-one row); the nebula
   LIFECYCLE half folds into the mana-family declaration (`fb771bfb8`):
   f49's `nebula_levelup`/tier fields/zeros, f12's g27 `apply_orb_arm` tier
   writer, d117/f49's `regen.rs` mana cadence — d11's player.rs twins DROP
   at merge; the `can_ghost_heal` keep-one is d6's FIELD (same C# gate,
   ctor-derived; d11's method retires, the call site reads the field).
