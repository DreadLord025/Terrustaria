# 127 — DOC-SWEEP lane 38: proj 111-120 spawns (W4-d12)

> Researched 2026-11-21 on branch `task/W4-d12-proj-ai-111-120` (base
> `a725acb0`, the wave-3-complete main). Continues the 26d census
> (`26d-projectile-ai-111-120-census.md`): the aiStyle-113 target-sticker
> family (598/614/636/971/975/1024) and the 117/189 explosion kinds are
> already rowed + armed here; this lane sweeps the SPAWN / ITEM-USE sites
> that feed them. Code-only round — no cargo/rustc/rustfmt/tests/graphify
> invoked; every witness is WRITTEN NOT RUN.

## In-flight claim probe (read before porting)

- `origin/task/W4-a9-minion-four-machines` @ `8c4fded8` — machines
  156/164/169/206 (bat/gem/smolstar/forbidden). Its `projectile.rs` diff
  touches NONE of `ai_flying_minion` / `MinionShot` / 614. **No 614 claim.**
- `origin/task/W4-d32-npc-ai-styles-85-93` @ `ac1d38e7` — NPC aiStyles
  85-93 (Mothron family, dummy, granite flyer, Dutchman). Its MoonLord
  kinds (396/397/398/400/401) are the HEAD/HAND/CORE eyes, NOT a tentacle
  spike. **No 971 claim** — and the brief's "MoonLord core tentacle spawn"
  guess is wrong (see item 3).
- `origin/task/W4-d19-proj-ai-181-190` @ `a4185f89` — landed the aiStyle
  182-186/190 arms + 13 rows. **The 189 Volcano ARM is present (shared with
  this branch's `ai_volcano`), but the 978 ROW is NOT** (grep of its
  `projectile/data.rs` for 978/Volcano returns only the `:1978` BulletSnowman
  cite). So item 7's condition ("if 189-arm+978-row there, cross-note only")
  is NOT met — the row ports here.

## The seven items

### Item 1 — 614 StardustCellMinion shot: the cell→614 target seating (FIXED)

The 614 row + the `AI_062` flying-minion machine + the `MinionShot` pool
spawn were already landed (F1 round + M4-PROJ-8). The REAL gap was a
mis-seated target: `AI_062`'s Cell fire (`Projectile.cs:66088`) passes
`ai0 = 0f, ai1 = num13` (the scan's NPC index), but the dispatch's shared
spawn passed `ai0 = target, ai1 = 0`. The 614 chase reads `(int)ai[1]`
(`:40594`) and the stick-pin / debuff gates read `ai[1] == victim`, so the
old wiring ALWAYS chased NPC 0. Fixed to seat the target in `ai1` for the
614 only (the UFO `:66072` / generic `:66122` fires pass no ai args). The
stale "614 rowless" comments corrected (the row landed in F1). The other
shot families (374/376/408/433) remain rowless — not this lane.
**Commit `de878f6ea`**, 1 witness.

### Item 2 — 636 Daybreak ITEM use (item 3543): VERIFY-ONLY, already ported

The brief's hint ("useStyle throw + stack-consume") is WRONG vs source.
`Item.cs:31871-31888` (case 3543): `useStyle = 1` (swing, NOT throw),
`noMelee = true`, `noUseGraphic = true`, `autoReuse = true`, shoot 636,
shootSpeed 10, damage 150, **NO `consumable`** (the 3094 javelin IS
consumable, `:28621`, but the Daybreak is not). The only 3543-specific
`ItemCheck_Shoot` arm is the upper-third spawn point
(`Player.cs:47225-47228`, shared with 3094/3378) — already ported
(`editing.rs:3114`). The 636 row (`data.rs:1868`), the aiStyle-113 fall
(0.995/0.15), the stick (5·MaxUpdates), the Daybroken 189/300 grant and the
Kill→953 spawn are all landed (F1). **No gap — no code change.**

### Item 3 — 971 TentacleSpike: the item-5094 melee on-hit proc (PORTED)

The brief's "MoonLord core tentacle spawn / d32 lane" guess is WRONG. The
SOLE C# spawn site for 971 is `Player.TentacleSpike_TrySpiking`
(`Player.cs:44160-44172`), called from `ApplyNPCOnHitEffects` (`:44477-44480`)
when the held item is **5094 (Tentacle Spike**, a plain melee sword,
`Item.cs:40805-40818`, no shoot). The latch `_spawnTentacleSpikes` arms on
the use start (`ItemCheck_OwnerOnlyCode` `:43468-43471`,
`ItemAnimationJustStarted`). Ported: the Player field, the use-start arming
(`start_swing_extras`), the post-strike consumption in `resolve_melee_strike`
(the `ApplyNPCOnHitEffects` position, `:44404`), and
`spawn_melee_sticker_proc` — the pre-pinned sticker (ai0=1/ai1=victim) at
`Hitbox.ClosestPointInRect(MountedCenter) + aim`, velocity
`(npc.Center - point) * 0.8`, `StatusNPC` at spawn (TentacleSpike 337/540,
`Projectile.cs:11343-11346`), `KillOldestJavelin` cap 5
(`_tentacleSpikesMax5`, `Player.cs:57136`). The `CanBeChasedBy` gate reads
post-strike liveness (a killing blow refuses the proc), the latch consumes
INSIDE the gate — verbatim. The 971 draws NOTHING. **Commit `3dbdcaef3`**
(shared with item 4), 1 witness.

### Item 4 — 975 BloodButcherer ITEM (item 795, NOT 3548): PORTED

The brief's "3548?" guess is WRONG — 3548 is `PartyGirlGrenade`
(`ItemID.cs:8779`); the Blood Butcherer is **795** (`ItemID.cs:3273`,
`Item.cs:10675-10688`, plain melee sword). Same proc shape as item 3:
`BloodButcherer_TryButchering` (`Player.cs:44174-44186`), latch
`_spawnBloodButcherer` (`:43472-43475`), the one difference is the velocity
rotates by `NextFloatDirection() * π/4` (`:44180`, ONE `NextDouble` draw,
`Utils.cs:1203-1206`) and the StatusNPC grants BloodButcherer 344/540
(`Projectile.cs:11347-11350`). Cap 5 (`_bloodButchererMax5`, `:57137`).
**Commit `3dbdcaef3`**, 1 witness.

### Item 5 — 1024 KillingDeckCard ITEM (item 5461, NOT 5069): PORTED

The brief's "5069?" guess is WRONG — 5069 is `FlinxStaff`
(`ItemID.cs:11821`, a summon staff whose `SpawnMinionOnCursor` arm is
already ported); the Killing Deck is **5461** (`ItemID.cs:12605`,
`Item.cs:43061-43078`, magic, shoot 1024, shootSpeed 12, mana 8). The
`ItemCheck_Shoot` 5461 arm (`Player.cs:49190-49261`) is the four-mode
card-draw cycle `killingCardFireType` 0..3:
- state 0: single card, ±0.4 spin (`Next(2)*2-1`);
- state 1: forward + back, two spin draws;
- state 2: six-card fan, the aim rotated by `(π/4/5)*(-2.5+i)`, the spin
  sign flips on `Next(2)==0` (one draw per card, after its spawn);
- state 3: recall-all — every active 1024 of the owner takes velocity 0 /
  ai0=2 / `netUpdate2` / `ResetLocalNPCHitImmunity` + inline `Damage()`
  (the ×1.5 return multiplier + the +6 hitbox ride the boxed walk).
The state-3 pre-check (`:49192-49208`) resets the cycle to 0 when no card
is still outbound (ai[0]!=2). The cycle advances after every use, wraps
>3→0; `UpdateDead` zeroes it (`:16992`). Ported: the Player field + reset,
the `fire_non_ammo_shot` 5461 arm, `recall_killing_deck_cards` +
`killing_deck_recall_damage` (the boxed-walk seam). The card seats its spin
in ai2 (the aiStyle-113 `FinalizeProjectile` rotation latch cancels the
+π/2 for the card, `:22049-22056`). **Commit `f0efe07e7`**, 1 witness.

### Item 6 — 953 Daybreak-explosion row: VERIFY-ONLY, present

The 953 row IS on this branch (`data.rs:3742`, aiStyle 117, 8×8,
penetrate −1, timeLeft 60, local-immunity 80, alpha 255, ignoreWater,
tileCollide false — `Projectile.cs:8988-9001` verbatim). The Kill→953 spawn
target resolves through it (`projectile.rs:20889`, `sticker_death` →
`spawn_with_data_ai_owned(DAYBREAK_EXPLOSION, …)`). The explosion-row audit
witness (`w4_d12_f1_the_explosion_kinds_all_have_rows`) already pins it.
**No gap — no code change.**

### Item 7 — 978 Volcano: the ROW ports here (d19 landed the arm, not the row)

d19's `a4185f89` landed the aiStyle-189 `ai_volcano` arm (shared with this
branch) + the 182-186/190 rows, but NOT the 978 row (claim probe above).
The row ports verbatim (`Projectile.cs:9253-9268`): 160×160, aiStyle 189,
friendly, melee, alpha 255, ignoreWater, tileCollide false, ownerHitCheck
(no port field — the spear family is the only ported ownerHitCheck
consumer), penetrate 3, local-immunity −1. The `timeLeft = 60` (`:9261`) is
OVERWRITTEN by the second `timeLeft = 30` (`:9268`) — the row keeps 30.
`proj_frames[978] = 5` (`Main.cs:8493`). The 189 dispatch arm already serves
it; the census witness upgraded from the conditional `if let Some` to a
hard `expect`. The item-121 (Fiery Greatsword/Volcano) melee spawn proc
(`Volcano_TrySpawningVolcano` `Player.cs:44150-44158`, the
`_spawnVolcanoExplosion` latch `:43476-43479`) is the player-lane
follow-up — NOT this lane (the row is the spawn's only gate).
**Commit `9e668d0f0`**, census witness upgraded.

## RNG contract

Every explicit `Main.rand` draw at the ported call sites stays in C# order
on the `UpdatePlayers` stream (`Main.cs:17754`, the melee/shoot pass):
- item 1: the cell fire draws nothing new (the target seating is a write);
- items 3/4: the 971 draws ZERO, the 975 ONE `NextDouble` (the rotation);
- item 5: state 0/1 one/two `Next(2)` spins, state 2 six `Next(2)` flips,
  state 3 the recall's boxed `Damage()` per-hit rolls (the existing
  `damage_pve_boxed` stream order).
The dust/gore BODIES and their internal rolls go with the dust layer (the
`projectile_ai_thrown` module contract); `Lighting.AddLight` / `hide` /
`netUpdate2` are the render/net layer's documented skips (netUpdate2 folds
to the one `net_update` flag).

## Deliberate skips / follow-ups (recorded for ANALYSIS)

- The 978 `ownerHitCheck` gate has no port field (the spear family's
  `SPEAR_KINDS` gate is the only ported consumer) — the Volcano's
  owner-line-of-sight refusal stays unmodelled until the field lands.
- The item-121 Volcano / item-155 Muramasa / item-5097 BatBat use-start
  latches (`ItemCheck_OwnerOnlyCode` `:43476-43483`) are NOT armed here —
  only the 5094/795 pair this lane ports. Their procs are the player-lane
  follow-ups.
- The 374/376/408/433 minion-shot rows remain rowless (other minions'
  shots, not this lane) — their `AI_062` spawns still drop.

## Commits (this lane, on `task/W4-d12-proj-ai-111-120`)

| item | hash | scope |
| --- | --- | --- |
| 1 | `de878f6ea` | 614 target seating ai1 + stale-comment fix |
| 3+4 | `3dbdcaef3` | 971/975 melee on-hit sticker procs |
| 5 | `f0efe07e7` | 1024 Killing Deck four-mode card cycle |
| 7 | `9e668d0f0` | 978 Volcano proj_data row |
| 2, 6 | — | verify-only (already ported / rowed) |

Witnesses: 5 written not run (1 item-1, 2 items-3/4, 1 item-5, 1 item-7
census upgrade). Code-only round — no build/test/lint invoked per the task
contract.

---

## FINAL phase (2026-11-21, same branch, base `4ba56e660`)

The five-item FINAL round: the 978 item-121 spawn proc, the 111-120 band
census table, the 1024 recall census, the 614 ai1 witness coverage, and
this report. Code-only — no cargo/rustc/rustfmt/tests/graphify invoked;
every witness WRITTEN NOT RUN.

### FINAL item 1 — the 978 item-121 Volcano spawn proc (PORTED)

The player-lane follow-up the DOC-SWEEP item 7 deferred. The SOLE C# spawn
site for 978 is `Player.Volcano_TrySpawningVolcano` (`Player.cs:44150-44158`),
called from `ApplyNPCOnHitEffects` (`:44485-44488`) when the held item is
**121 (Fiery Greatsword / Volcano**, `Item.cs` row: melee, damage 40,
useStyle 1, no shoot). The latch `_spawnVolcanoExplosion` (`:2900`) arms on
the use start (`ItemCheck_OwnerOnlyCode` `:43476-43479`). Ported:
- the Player field + init (`spawn_volcano_explosion`),
- the use-start arming (`start_swing_extras`, the 121 arm),
- the post-strike consumption in `resolve_melee_strike` (the
  `ApplyNPCOnHitEffects` position, damage `(int)(dmg * 0.75)` `:44487`),
- `spawn_volcano_proc` — the 978 at the victim's CENTRE, `ai0=0 / ai1=2`,
  velocity `(0, -1·gravDir)` (gravDir folded to 1), the centre→top-left
  recentre. **NO `StatusNPC` at spawn** (the OnFire 24/180 rides the
  explosion's own damage walks, `Projectile.cs:11359-11362`) and **NO
  `KillOldestJavelin` cap** — the two differences from the 971/975 pair.
- the gate `HittableForOnHitRewards` (`NPC.cs:90043-90050`) +
  `CountsAsACritter` (`:6887-6897`, the 594 WindyBalloon / 686
  BoundTownSlimePurple exclusions), ported as `Npc` methods. **NO liveness
  term** — unlike the 971/975's `CanBeChasedBy`, a KILLING blow still fires
  the Volcano. The latch consumes INSIDE the gate (`:44156`). ZERO RNG.

**Latent-bug fix (the DOC-SWEEP items 3/4):** `resolve_melee_strike` read
`let proc_damage = dmg;` BEFORE `dmg` was declared (an E0425 — the first
pass never compiled). The C# value at that point is the class-multiplied
`raw` (`ApplyRapidAttackBonus` is a no-op for 5094/795/121 — the set is
{2608, 3349, 672}, `ItemID.cs:344`), so the fix reads `raw as i32`. The
971/975 witness re-press rigs were also fixed (the items are `autoReuse:
false` and the rig never decays `attack_cd`/`hurt_cd` — the release/press
edge + timer clears + parked life, the established i05 convention).
**Commit `7cf1281c5`**, 4 witnesses (the gate truth-table, the centre/ai/
damage proc, the killing-blow contrast, + the 971/975 rig repairs).

### FINAL item 2 — the 111-120 band FINAL census table (ALL ARMED+ROWED)

A source+table witness (`w4_d12_final_the_111_120_band_is_all_armed_rowed`)
re-derives BOTH sides: the dispatch's top-level arm labels (the §3b scan)
and the `proj_data` rows. Every style 111..120 is ARMED and ROWED with the
expected kind membership pinned:

| style | kinds | arm |
| --- | --- | --- |
| 111 | 586 DryadsWardCircle | `ai_dryads_ward` |
| 112 | 644/590/836 (shard/spore/seed) | the kind-split arm |
| 113 | 598/614/636/971/975/1024 | `target_sticker_step` |
| 114 | 602 PortalGunGate | `ai_portal_gate` |
| 115 | 604 TerrarianBeam | `ai_yoyo_beam` |
| 116 | 607 SolarFlareRay | `ai_solar_flare_ray` |
| 117 | 608/612/624/953/1044 | `ai_explosion` |
| 118 | 617 NebulaArcanum | `ai_nebula_arcanum` |
| 119 | 618 NebulaArcanumSubshot | `ai_nebula_arm` |
| 120 | 623 StardustGuardian | `ai_stardust_guardian` |

**NONE is UNARMED_ROWED** — the band's wiring completed across F1 + the
tail rounds (111/112/114/116/117 here, 113 F1, 115 S1-D-5, 118/119 the
Arcanum pair, 120 S1-P-i56). The `UNARMED_ROWED` pin still holds only the
four a9 minion styles (156/164/169/206), none in this band. **Commit
`c839e294a`**, 1 witness.

### FINAL item 3 — the 1024 recall-all census (FIXED + cap interplay)

The DOC-SWEEP item 5 port added `p.active` gates to the recall scan and the
no-outbound pre-check. **The C# has NEITHER** (`Player.cs:49243` reads only
type + owner; `:49198` the same) and `Kill()` never clears
type/owner/ai/damage (`Projectile.cs:77262` only flips `active`). So a DEAD
stuck card (the victim died under it, `:40770-40773`) is recalled too: it
takes `ai0=2` + `ResetLocalNPCHitImmunity` + the inline `Damage()`, landing
ONE more ×1.5 hit on whatever sits in its box, then the return-state kill
check re-kills it next tick (`:40866`). Both `p.active` gates dropped
verbatim (the port's dead slots keep kind/owner/ai/damage until respawn).

**The `KillOldestJavelin` cap-5 interplay (the census question):** recalled
cards (ai0=2) **NEVER count toward a cap** — the scan's predicate is
`ai[0]==1` (`:15079`), so the return state is invisible to it. And the 1024
has **no cap arm at all**: `:13979` names 598/636/614 only (caps 6/8/10);
the 971/975 cap-5 lives at the SPAWN PROC (`Player.cs:44169/:44183`), not
the on-hit arm. So the recall and the javelin caps are orthogonal.
**Commit `e9eb06619`**, 2 witnesses (the pool-level dead-card recall + cap
census, the editing-level pre-check no-active-gate).

### FINAL item 4 — the 614 ai1-fix witness coverage + a9 compose-contract

The DOC-SWEEP item 1 fix seats the Cell fire's target in `ai1`
(`Projectile.cs:66088` ai0=0/ai1=num13). A chase-machine witness
(`w4_d12_final_the_614_chase_reads_ai1_not_ai0`) pins the CONSEQUENCE: with
the live target at index 1 (−X) and a dead NPC at index 0 (+X), the
correctly-seated shot (ai1=1) chases −X and lives; the OLD mis-seating
(ai0=target/ai1=0) reads the dead NPC 0 and kills (`:40594` reads ai[1]).

**a9 compose-contract (read `origin/task/W4-a9-minion-four-machines` @
`8c4fded81`):** a9's four machines (156 bat / 164 gem / 169 smolstar / 206
forbidden) spawn their OWN shots — 755/946 (bat), 831/970 (gem), 864
(smolstar), 1119/1120 (forbidden) — each through `spawn_with_data_ai_owned`
with `ai0 = features.ai_state()` / `ai1 = 0` (the 1120's `:37592`
`gemStaffFeatures.Bits`, verbatim). NONE touches the 62-arm / `MinionShot` /
614: the cell→614 target seating is the 62 dispatch's per-kind branch, and
a9's machines are separate aiStyles. The two lanes compose — no shared
spawn site, no ai-arg conflict. **Commit `39507c133`**, 1 witness.

### FINAL item 5 — this report

research/127 gains this FINAL section; ANALYSIS gains the recall no-active-
gate deviation note + the Volcano proc; PORT_MAP gains the two new seams.

## FINAL commits (this phase, on `task/W4-d12-proj-ai-111-120`)

| item | hash | scope |
| --- | --- | --- |
| 1 | `7cf1281c5` | 978 item-121 Volcano proc + the items-3/4 E0425/rig fixes |
| 2 | `c839e294a` | the 111-120 band FINAL census-table witness |
| 3 | `e9eb06619` | the 1024 recall no-active-gate fix + cap-interplay census |
| 4 | `39507c133` | the 614 ai1 chase witness + a9 compose-contract |
| 5 | (this) | research/127 FINAL + ANALYSIS + PORT_MAP |

Witnesses: 8 written not run this phase (4 item-1 incl. the 971/975 rig
repairs, 1 item-2, 2 item-3, 1 item-4). Code-only round — no build/test/lint
invoked per the task contract.
