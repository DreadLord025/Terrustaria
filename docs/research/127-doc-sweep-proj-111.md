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
