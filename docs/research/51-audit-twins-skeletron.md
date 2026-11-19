# 51 — Twins / Skeletron audit (census slice B-1, distrust-verbatim)

Code-only audit round, 2026-11-19 (`docs/audit-twins-skeletron`, base `c7fe175d`).
**No compiler invoked** — file reads + git only; no cargo/rustc/rustfmt/tests run.
Scope: the census §B-1 targets — Retinazer (kind row 52 / vanilla 125, aiStyle 30),
Spazmatism (53 / 126, aiStyle 31, spot-check only), Skeletron head (44 / 35,
aiStyle 11) + hands (79 / 36, aiStyle 12), Skeletron Prime head (51 / 127,
aiStyle 32) + the shared `update_prime_limb` stand-in. The f1701eb0 defect
catalogue is the lens: missing gates, f32 op order, int-vs-float division,
strict bounds, draw-before-early-return, invented constants, RNG draw
order/count.

C# source: the decompiled `Terraria/NPC.cs` at the junction (all `NPC.cs:`
cites below are that file). Rust cites are `crates/tr-app/src/…` at the same
commit. **Vanilla ids**: `NPCID.cs:10927` Retinazer = **125**, `:10929`
Spazmatism = **126**, `:10747` SkeletronHead = 35, `:10749` SkeletronHand = 36,
`:10931` SkeletronPrime = 127, `:10813` DungeonGuardian = 68.

## Verdict table

| target | live machine | C# arm | verdict | headline |
| --- | --- | --- | --- | --- |
| Retinazer 125 (style 30) | `npc.rs::update_retinazer` :13118-13334 via kind arm :3812 | `NPC.cs:26405-27014` | **WRONG** | ports the **aiStyle-57 MourningWood** machine (`:32761-33043`), not style 30 — wrong projectiles (325/326-328 vs 83/100), no laser phases, no 40%-hp transform, no despawn, RNG stream mismatch |
| Spazmatism 126 (style 31) | `npc_ai_style31.rs::spazmatism_ai` via axis :3665-3671 | `NPC.cs:27015-27625` | **FAITHFUL** (spot-check) | aim/turn/dust-gate/shot-RNG/dash all match; CLASSIC_SOLO env parked as documented |
| Skeletron head 35 (style 11) | `npc.rs::update_skeletron` :8100-8287 via kind arm :3795 | `NPC.cs:21919-22299` | **WRONG** | the 800-tick hover→spin cycle shape survives but: expert block keyed on `hard_mode`, hands-count via `ai[3]` (collides with the red-hat latch), **no +25/hand defense**, no damage-lerp, no `EncourageDespawn(50)`, spin-shot gate/order/velocity/origin defects |
| Skeletron hands 36 (style 12) | `npc.rs::update_skeletron_hand` :13735-13805 via kind arm :3796 | `NPC.cs:22300-22614` | **WRONG** | 6-state machine collapsed to 3; strike exits one tick early (invented ×0.95 decel); **no vy damp in the rise** (rise ~2× too fast); no horizontal-slam states 4/5 (unreachable in C# too — but state 2's real exit conditions are gone); hand never dies when the head dies |
| Skeletron Prime head 127 (style 32) | `npc.rs::update_prime` :12963-13103 via kind arm :3805 | `NPC.cs:27626-27942` | **DEVIATION-DOCUMENTED → mostly FAITHFUL core, gaps below** | hover/charge/rage constants and bands match line-for-line; expert keyed on `hard_mode`; no despawn timer in state 3; limb spawn via sim drain (faithful seats) |
| Prime limbs 128-131 (styles 33-36) | `npc.rs::update_prime_limb` :13456-13609 | `NPC.cs:27943-28990` | **WRONG** (already catalogued W82/W109) | shared invented v1 — re-confirmed, details §6 |

Also in scope of the brief: **"guardian spawn at dawn" is a false premise** —
the C# head has no guardian transform; the DungeonGuardian (68) is a *dungeon
spawner* NPC (`NPC.SpawnNPC`'s ZoneDungeon arm, `NPC.cs:2643-2651`: while
`!NPC.downedBoss3`, dungeon spawns become type 68). The dawn behaviour is the
head's own `ai[1] = 2` enraged 8-px rush (`NPC.cs:21958-21962`, `:22256-22272`).
The port has **no DungeonGuardian kind and no dungeon-spawn gate on
`downed_skeletron`** (only `lang_tables.rs:145` names it) — recorded as a
missing-feature gap, not an AI-arm defect.

## 1. Retinazer — the mislabel, confirmed

**What the live 125 rides.** `NpcKind::Retinazer` maps to vanilla id 52 in the
port's protocol table (`npc/data.rs:374`) — the port id, not vanilla — and to
aiStyle **30** (`npc/data.rs:1944`, matching `SetDefaults` type 125,
`NPC.cs:10522-10537`: 100×110, def 10, dmg 45, 20000 hp, noGravity +
noTileCollide). The style axis (`npc.rs:3651-3689`) has no `30 =>` arm, so it
falls to `update_kind_ai`'s `NpcKind::Retinazer => self.update_retinazer(...)`
(`npc.rs:3812`).

**What `update_retinazer` actually is.** Its own doc comment (`npc.rs:13110-13113`)
says it: *"`AI()`'s aiStyle-57 block (`NPC.cs:32761-33043`) — the Retinazer's
machine: the idle (0), the stomp (1), the spin (2), the beam (3) and the charge
(4)"* — that is **MourningWood** (type 325). Every in-body citation is a
`:327xx-330xx` line. State-by-state it is a near-copy of the sibling
`npc_ai_pumpkin_boss.rs::mourning_wood_step` (W98): hp-band speeds 2/3/4 +
day 8 (:13125-13134 ≈ C# `:32771-32790`), 300-tick idle with `Next(1,3)`/`Next(3,5)`
(:13149-13155), FLAMING_WOOD 325 stomp every 15 (:13163-13181), Greek fires
`Next(326,329)` spin (:13193-13213), 30-tick beam (:13224-13242), 10-tick
barrage (:13254-13274), the ×0.9 damp + `(v*20±speed)/21` drift (:13287-13298)
and the solid-tile probe hover (:13302-13333). The dispatch comment
(`npc.rs:3806-3808`) even admits *"the S1-D-3i port of the shared aiStyle-57
block shape"*. The census's MISLABEL flag is correct.

**What the REAL aiStyle-30 arm is** (`NPC.cs:26405-27014`, read whole):

* Preamble: retarget gate `:26407-26410`; bottom-anchored aim
  `Atan2(x: npcCX − playerCX, y: npcBottom − 59 − playerCY) + 1.57f` with the
  6.283 wrap (`:26412-26420`), 0.1 shortest-path turn with the twice-applied
  snap band (`:26421-26459`) — the same shape as style 31 but **0.1 not 0.15**;
  every-tick `Next(5)` dust gate (`:26460-26465`); the twin-alive
  `DiscourageDespawn(timeLeft−1)` loop over active 125/126 (`:26466-26475`);
  `reflectsProjectiles = false` (`:26485`); **day/dead flee `vy −= 0.04` +
  `EncourageDespawn(10)` + return** (`:26486-26491`).
* Phase 0 (`ai[0]==0`, `:26492-26711`): hover `ai[1]==0` — 7-speed/0.1-accel
  (expert 8.25/0.115), side ±300 x / −300 y station, 600-tick timer → dash,
  the below-player `dist<400` shot window with `ai[3]` cadence 60 (expert
  +0.3×4 hp bands), projectile **83** (the hostile eye laser, shared with the
  WoF eye's arm `:26183`) at 9-speed (expert 10.5), `Next(-40,41)*0.08` X-then-Y jitters,
  spawn offset ×15 (`:26618-26641`); dash `ai[1]==1` — rotation snap + 12-speed
  (expert 15) direct velocity, →2 (`:26645-26664`); decay `ai[1]==2` — 25-tick
  hold then ×0.96 damp with the ±0.1 dead-band, 70-tick cycle, **4 dashes then
  back to hover** (`:26665-26700`); the **40%-hp transform** `ai[0]=1` +
  reset + netUpdate (`:26702-26709`), then `return`.
* Transform (`ai[0]==1|2`, `:26712-26771`): spin-up ±0.005/tick to 0.5,
  `rotation += ai[2]`, 100-tick per stage, gore/dust burst + sounds at the
  1→2 edge, ×0.98 damp + dead-band, `return`.
* Phase 3 (`ai[0]==3`, `:26773-27013`): `damage = defDamage*1.5`,
  `defense = defDefense+10`, `HitSound = NPCHit4`; hover — 8-speed/0.15-accel
  station −300 above, 300-tick timer, **localAI[1] laser counter** with hp
  bands +1/+1/+1/+2 (<75/50/25/10%), `>180` + `Collision.CanHit` → projectile
  **100 (DeathLaser)** at 8.5-speed (expert 10), dmg `GetAttackDamage_ForProjectiles(25,23)`,
  offset ×15 (`:26873-26906`); side-strafe `ai[1]==1` — ±340 station, 8/0.2,
  laser counter with **+0.5/+0.75/+1/+1.5 bands + expert +1.5**, `>60` + CanHit
  → 100 at 9-speed dmg (18,17) (`:26909-27003`), 180-tick cycle (`:27005-27013`).

**Defect list (live vs the real arm):**

| # | class | C# | live Rust | fix spec |
| --- | --- | --- | --- | --- |
| R1 | wrong machine | `:26405-27014` whole | :13118-13334 = the style-57 MourningWood port | port style 30 as `npc_ai_style30.rs` (the style31 module is the template — same aim/turn/wrap skeleton, rate 0.1); claim `30 =>` on the axis; delete the kind arm :3812 |
| R2 | wrong projectiles | 83 phase-0 (`:26628`), 100 phase-3 (`:26895`/`:26996`) | FLAMING_WOOD 325 + Greek fires 326-328 (:13175/:13207/:13236/:13268) | the two laser kinds + the `GetAttackDamage_ForProjectiles` classic/expert pairs (20/19, 25/23, 18/17) |
| R3 | missing gates | day/dead flee + `EncourageDespawn(10)` `:26486-26491`; 40%-hp transform `:26702-26709`; `Collision.CanHit` shot gates `:26890`/`:26991`; twin-alive discourage `:26466-26475` | none (day only raises speed to 8, :13132) | all four; the transform is the boss's second phase — without it the fight never escalates |
| R4 | missing stats writes | phase-3 `damage×1.5`, `defense+10`, `HitSound NPCHit4` `:26773-26775` | none | the phase-3 preamble every tick |
| R5 | RNG draw order/count | `Next(5)` every tick before the day gate (`:26460`); phase-0 shot X-then-Y `Next(-40,41)*0.08` (`:26636-26637`); spin burst 2×(3 gores×2) + 20 dust×2 = 52 draws (`:26747-26756`) + per-tick spin dust 2 (`:26760`); no draws on plain hover ticks besides the gate | idle `Next(1,3)`/`Next(3,5)` every 300 (:13152-13154), stomp/spin/beam jitters `Next(-20,21)`/`Next(-50,51)`/`Next(50,201)`/`Next(326,329)`/`Next(-30,31)` on firing ticks (:13171-13274) | the style31 header's draw-order contract applies verbatim to style 30 (same skeleton); the live stream desyncs the shared `UpdateNPCs` RNG on every Retinazer tick |
| R6 | integration | `noTileCollide` — position += velocity, no tile probe | solid-tile probe hover (:13309-13320) + gravity-shaped vy ramps | the raw integration tail (style31 :197-202 precedent) |

The Spazmatism's faithful style-31 module proves the house pattern for this
exact machine family; style 30 is its sibling with different constants and
projectiles. **Fix size: M** (one module + axis arm + kind-table cleanup).

## 2. Spazmatism — spot-check (V-W1 FAITHFUL re-confirmed)

Per the brief, no deep re-audit. Spot-checked against `NPC.cs:27015-27625`:

* Aim `:27022` (bottom −59, named-arg Atan2 = `b.atan2(a)`, integer halves) —
  module :140-152 ✓; 0.15 turn + 3.1415 half-turn + twice-applied snap band
  `:27031-27073` — module :226-257 ✓ (incl. the MechQueen ×0.25 quarter-rate).
* `Next(5)` dust gate every tick before the day gate `:27074` — module :260 ✓.
* Twin-alive discourage `:27080-27089` — module :266-273 ✓ (twin list is the
  documented `&[]` seam; despawn sentinel gate documented).
* Day/dead flee `:27100-27105` — module :291-295 ✓; hover 12/0.4 + side ±400
  `:27111-27124` — module :301-305 ✓; shot 96 at 12-speed (expert 14), jitters
  `Next(-40,41)*0.05` X-then-Y, offset ×4 `:27221-27235` — module :377-387 ✓.
* Phase-3 laser jitter **Y-first** `:27525-27526` — module :636-637 ✓ (the
  reversal is pinned in the header).
* The `noTileCollide` integration tail — module :195-202 ✓.

CLASSIC_SOLO env (expert/getGood/MechQueen parked false/None) is the documented
stand-in (module header + ROADMAP row 315). **Verdict FAITHFUL** — no new
findings; the wiring of `twin_time_lefts` from the sim remains the known
follow-up (it becomes live once Retinazer's real machine lands).

## 3. Skeletron head — the "verbatim" claim fails

`update_skeletron` (`npc.rs:8100-8287`) vs `NPC.cs:21919-22299` (read whole).
The 4-state skeleton (hover 0 / spin 1 / enrage 2 / despawn 3), the 800/400-tick
cycles, the hover accel tables (0.02/2/0.05/8 classic, 0.03/4/0.07/9.5 expert),
the ±250 station, the ×0.98 reversal damps, the spin's `direction*0.3` roll,
the enrage 8-px rush and the despawn sink+damp all match. The defects:

| # | class | C# | live Rust | fix spec |
| --- | --- | --- | --- | --- |
| S1 | wrong difficulty flag | `Main.expertMode` gates the hands scan, +defense, spin shots, charge ladder (`:21964`, `:22046`, `:22191`) | `world.hard_mode` (:8126/:8168/:8222-8223) — hardmode ≠ expert; pre-hardmode expert worlds get classic stats, post-WoF classic worlds get expert stats | a real difficulty flag (the style31 `Style31Env::expert` seam precedent); until then document the fold, don't silently proxy |
| S2 | missing defense ladder | `defense = defDefense` reset each tick (`:21923`) then **`+= num168*25` per live hand in expert** (`:21973`), spin `−= 10` (`:22165`) | reset :8106 + spin −10 :8212 only — **no +25/hand anywhere** | the per-tick hands scan + `defense_override = def + hands*25` (expert), spin subtracts 10 from THAT |
| S3 | hands-count slot collision | `num168` = live count of type+1 (`:21966-21972`), a LOCAL; the head's `ai[3]` is the **red-hat latch** (`RedHatSkeletronAdjustmentsEnabled` `:66673` reads `type==35 && ai[3]==1`) | the sim writes `head.ai[3] = hands_alive` every tick (`world_sim/npcs.rs:3739`) and `skeletron_red_hat = ai[3]==1` (:3745) — **one hand alive ⇒ the head reads RED-HAT MODE**: `red_hat_skeletron_adjustments_enabled` (`npc.rs:7981-7987`) returns true, the Dark Caster latch + hand `local_ai[3]` mirror activate spuriously | keep `ai[3]` for the (unported) red-hat trigger only; carry the hands count in a separate field/World mirror. ANALYSIS:5522 documents the stand-in but not this cross-contamination — with 2 hands spawned the count is 2→1→0, so **every Skeletron fight spends its last-hand phase in fake red-hat mode** |
| S4 | spin-shot gate dropped | expert shot runs when `(num168 < 2 \|\| life < lifeMax*0.75) && ai[1]==0` (`:21974`) — with 2 hands and hp ≥75% **no shots** | fires whenever `hard_mode` (:8168-8171) — no hands/hp gate | add the gate |
| S5 | spin-shot f32 op order + velocity | direction = normalize(playerΔ + `Next(-20,21)` each axis) × speed, THEN + `Next(-50,51)*0.01` jitters, THEN `Normalize()` × speed again, **THEN `+= base.velocity`** (`:21995-22008`); spawn at `center + vector17*5` (post-velocity) (`:22011`) | jitter added after the first normalize but the re-normalize (:8195-8197) matches; **velocity added only to the shot's velocity tuple (:8202-8203), NOT to the spawn origin (:8200-8201)** — origin misses the velocity term | fold `vx/vy` into (dx,dy) before both the origin ×5 and the shot velocity |
| S6 | strict bound / zero guard | `num174 <= 0 → 1` (`:21998-22001`) — a floor on the raw distance only | `.max(1.0)` on the sqrt (:8189) ≈ same; second normalize uses `.max(1.0)` (:8195) where C# `Normalize()` has no floor — near-zero post-jitter vectors diverge | mirror `Vector2.Normalize` semantics (zero → zero) |
| S7 | missing damage lerp | spin damage = `GetAttackDamage_LerpBetweenFinalValues(defDamage, defDamage*1.3f)` (`:22186`, red-hat ×1.3 `:22189`); hover damage = `defDamage` (red-hat ×1.3 `:22020-22024`) | spin damage = flat `contact_damage()*1.3` if hard_mode (:8274-8275) — no lerp over difficulty; hover reset ✓ (:8270-8271) | the lerp helper (classic endpoint = defDamage, so classic-only impact is nil until expert lands — document) |
| S8 | missing despawn timer | state 3 `EncourageDespawn(50)` (`:22281`) | none (:8259-8266) — the head drifts forever; only the off-bottom kill (:8284-8286, a port invention — C# relies on timeLeft) | `encourage_despawn(50)` on the port's despawn seam |
| S9 | missing retarget | `TargetClosest()` at init (`:21926`), at the 2000-px/dead check (`:21951`) and at the 800-tick spin transition (`:22034`) | none — single-player target is implicit; the dead/far check sets `ai[1]=3` directly (:8108-8111) without the second-chance retarget | `npc_target::target_closest` at the three sites (the second-chance shape matters for MP later) |
| S10 | missing getGood/red-hat arms | `:21981-21984` cadence ×0.8, `:22053-22067` accel ×1.35 / +0.01 etc., `:22119-22163` Dark-Caster spawns during spin (cap 6/4, `CountNPCS(32)`), taunts `:22038` | none | getGood parked (house precedent); the red-hat spawn block is a real gap once S3 is fixed — record as follow-up |
| S11 | RNG count | expert spin shot draws 2×`Next(-20,21)` + 2×`Next(-50,51)` per firing tick (`:21995-22003`); red-hat taunt `Next(2,6)` (`:22038`); caster spawn `Next(-50,51)`×2 per attempt up to 1000 (`:22141-22142`) | shot draws match (:8186-8194) **but fire on ticks the C# gate (S4) would skip** — stream drift in expert; no other draws | fixing S1/S4 restores the count |
| S12 | missing dust tail | `:22283-22297` — 1+2 dusts per tick outside states 2/3, gated `(num168 != 0 \|\| !expertMode)` | skipped | dust is a documented port-wide skip, but these carry **no rand draws** (fixed velocities) — zero stream impact, cosmetic only |

The head's init hand-spawn lives in the sim (`world_sim/npcs.rs:3701-3725`):
two hands, sides −1/+1, `ai[3]=150` on the +1 hand — matches `:21930-21940`
(`ai[0]=−1` then `+1`, `ai[3]=150` on the second) ✓, except the spawn position:
C# `NewNPC(..., X: headCX, Y: headTop + height/2)` places the hands' **Bottom**
at the head's vertical centre (`NewNPC` sets `Bottom`, `NPC.cs:80770`); the
port centres a 52×52 hand box on `(headCX−75, headCY−75)` (:3714-3715) — an
invented −75 offset on both axes (the hands start up-left of the C# spot; the
tether pulls them to the dock within a second, cosmetic-class but cited).

**Verdict WRONG** — the shape is right, the numbers/gates are not. Fix size M
(the arm is 200 lines; the hands-count/red-hat split touches the sim mirror +
the caster latch + ANALYSIS:5500-5534's documented model).

## 4. Skeletron hands — the slam cadence is broken

`update_skeletron_hand` (`npc.rs:13735-13805`) vs `NPC.cs:22300-22614` (read
whole). The C# is a **6-state** machine on `ai[2]`: 0/3 dock-return, 1 rise,
2 slam-flight, 4 horizontal wind-out, 5 horizontal strike. The port has 3
arms (0|3, 1, `_`).

| # | class | C# | live Rust | fix spec |
| --- | --- | --- | --- | --- |
| H1 | missing head-death despawn | head inactive/not-style-11 → `ai[2] += 10`, `>50 \|\| netMode != 2` → die (`:22303-22312`); single-player dies immediately | none — the port's hands **never die when the head dies** (the sim only stops updating `skeletron_x/y`; the hands keep slamming the last anchor forever) | the head-gone check via the sim's head-present mirror (the prime limb's `prime_present` pattern) |
| H2 | missing head-state gate | dock-return steering only when `head.ai[1] != 0` (head spinning/enraged); while the head HOVERS (`ai[1]==0`) the hands run the **300-tick attack timer** (`:22328-22395`) | the timer runs unconditionally in state 0|3 (:13752-13756) — hands attack during the head's hover AND spin | gate on the head's `ai[1]` (World mirror) |
| H3 | f32 op order — the rise | state 1: `vx *= 0.95` **every tick**, `vy −= 0.1` then expert −0.06 more / classic cap −8 (`:22509-22531`) | `vy −= 0.1` + cap −8 (:13783-13786) but **no `vx *= 0.95`** — the hand keeps its horizontal drift through the rise | add the damp |
| H4 | strict bounds — the dock Y | C# compares `position.Y` (TOP) vs `head.Y + 230` and `head.Y − 100/−200` (`:22447`, `:22532`) | compares `cy` (CENTRE, :13744/:13757/:13787) — 26 px off on every threshold | use `self.y` |
| H5 | invented state collapse | state 2 exits ONLY on `position.Y > player.Y \|\| dot(v, playerΔ) <= 0 \|\| dist > 2000 \|\| vy < 0` (`:22548`) → state 3; the flight keeps its 18-speed vector until then | `_ =>` arm (:13796-13801) damps ×0.95 both axes and forces `ai[2]=3` **on the first tick of state 2** — the slam never flies; it decays in place | port state 2's real exit test; the ×0.95 decel is invented (no such line in the arm) |
| H6 | missing states 4/5 | the horizontal wind-out (`vx += 0.1*(−ai[0])`, cap ±8, ±500 window → 17-speed strike, `:22553-22611`) | absent | NOTE: `ai[2]=4` is never SET inside this C# arm (grep: the only writers are other styles) — states 4/5 are dead code in vanilla 1.4.5.8 too; port them verbatim for completeness but they are not a behavioural gap |
| H7 | missing strike speed tiers | classic 18 / expert 21 / red-hat 24 (`:22540`) | flat 18 (:13791-13792) ✓ classic; expert/red-hat parked | env seam (S1's difficulty flag) |
| H8 | rotation dropped | dock aim `Atan2(dy,dx)+1.57` written every tick in 0/3, 1, 4 (`:22500`, `:22508`, `:22559`) | comment-only fold (:13777-13778) — the hand draws with whatever rotation it had | write `self.rotation` (the render reads it — the prime-limb v2 fixed the same class, :13593-13606 precedent) |
| H9 | dock-return X asymmetry + f32 op order | two separate ifs with the −120·side (head spinning, `:22354-22377`) / −200·side (hover, `:22471-22494`) stations; Y ±0.04 cap 3, X ±0.07 cap 8; the damp runs BEFORE the accel (`if vy>0 vy*=0.96; vy−=0.04`, `:22449-22453`); expert runs BOTH steering blocks (`:22396-22446` then `:22447-22494`) | one if/else on `cx > slot_x` (:13764-13776), single station; the accel runs BEFORE the damp (:13758-13760 — reversed); the two-sided `clamp(-3,3)`/`clamp(-8,8)` vs the C#'s one-sided caps; **no else-if on Y** — when `cy < slot_y` the C# ADDS 0.04 (`:22459-22470`); the port only subtracts (:13757-13763) — **the hand can never descend to the dock** | port both Y branches + both X stations in the C#'s damp-then-accel order with the one-sided caps |
| H10 | timer cadence | `ai[3]++` (+1 red-hat, +0.5 expert) → 300 (`:22381-22395`); the +1 hand starts at `ai[3]=150` (spawn) | +1 → 300 (:13752-13753) ✓ classic; init 150 ✓ (sim :3711) | env seam for the +0.5/+1 |
| H11 | localAI[3] mirror | `localAI[3] = head.ai[3]` — the RAW slot copy (`:22315`), red-hat predicate reads `==1` | `local_ai[3] = if world.skeletron_red_hat {1} else {0}` (:13740) — a bool-ised copy of the hands-count stand-in (S3's contamination flows through here) | copy the raw slot once S3 splits the fields |

**Verdict WRONG.** H5 is the headline: the hands' signature slam — rise above
the head, then an 18-px/tick dive at the player — **never happens**; the port
rises, then decays in place. Combined with H9 (can't descend) the hands hover
above their dock permanently after the first attack cycle. Fix size M.

## 5. Skeletron Prime head — mostly faithful, three gaps

`update_prime` (`npc.rs:12963-13103`) vs `NPC.cs:27626-27942` (read whole).
Line-by-line: damage/defense reset ✓ (:12969-12970 ≈ `:27628-27629`),
6000-px/dead → state 3 ✓, day → state 2 ✓, hover 600-tick ✓, rotation vx/15 ✓,
accel table (0.1/2/0.1/8 classic — note this differs from Skeletron's
0.02/0.05!) ✓ (:12996-13000 ≈ `:27695-27714`), the TOP-edge −200/−500 Y band ✓
(:13004/:13012 ≈ `:27716/:27728`), the ±100 X band ✓, charge ×2 damage/defense
✓ (:13044-13045 ≈ `:27767-27768`), 400-tick ✓, roll ✓, 2/6-speed + the
150→600 expert ladder ✓, rage 9999/9999 + `(10 + d/100).clamp(8,32)` ✓
(:13076-13082 ≈ `:27862-27888`), despawn sink ✓. Gaps:

| # | class | C# | live Rust | fix spec |
| --- | --- | --- | --- | --- |
| P1 | wrong difficulty flag | `Main.expertMode` (`:27709`, `:27796`) | `world.hard_mode` (:12996/:13055-13056) | same as S1 |
| P2 | missing despawn timer | state 3 `EncourageDespawn(500)` (`:27933`) | none (:13087-13094) — drifts forever (off-bottom kill only) | `encourage_despawn(500)` |
| P3 | missing charge sound tick | `ai[2]==2` → roar (`:27770-27773`) | none | sound layer follow-up (no rand draws — stream-safe) |
| P4 | init latch shape | `ai[0]==0` → TargetClosest + `ai[0]=1` + the four limb spawns (`:27635-27661`) | `caster_counter/cast_windup` mirrors + `wants_limbs` (:12982-12985), sim drain `world_sim/npcs.rs:3628-3653` | the drain's seats (cannon −1, saw +1, vice −1 ai3=150, laser +1 ai3=150) match `:27639-27660` ✓; position: C# hands' Bottom at head centre vs port limb-box centred (:3644) — same class as §3's spawn note; TargetClosest missing (single-player implicit) |
| P5 | MechQueen arms | `ai[3]!=0` → `NPC.mechQueen` (:27630-27633), the orbit/AngleLerp/0.6-0.75 rows | absent | the port has no Mech Queen (Twins+Prime+Destroyer simultaneous) — documented fold, the style31 `mech_queen: None` precedent |

**Verdict DEVIATION-DOCUMENTED** (the classic solo core is faithful; P1/P2 are
real defects to fix with the difficulty-flag + despawn-seam wave; P4/P5 are
the documented stand-ins). The limbs it spawns ride §6.

## 6. Prime limbs — the shared v1 stand-in (re-confirmed, W82/W109 territory)

`update_prime_limb` (:13456-13609) serves all four kinds (dispatch :3798-3800).
Against the four C# arms (33: `:27943-28244`, 34: `:28245-28521`, 35:
`:28522-28756`, 36: `:28757-28990`):

* The dock drift ±0.07/±0.04 with the ±8/±3 dead-bands (:13472-13483) is
  **invented** — no C# arm steers to the dock that way (the saw's faithful
  module `npc_ai_style33.rs` ports the real 7-speed chase + 320/260 band +
  800/400 hysteresis; UNWIRED).
* Cannon: fires every 40 ticks at the PLAYER (:13490-13503) — C# fires in
  state 0 at the **dock-reflected** direction `(0−num523, 0−num524)` every
  **140** ticks (`localAI[0] > 140`, `:28659-28673`) and in state 1 at the
  player every 40 (`:28738-28752`); the port has no state gate, no 140-tick
  idle volley, no `Next(-40,41)*0.01` jitters, no ×4 spawn offset. Bomb
  damage 0 ✓ (`:28664`/`:28743`).
* Laser: fires every 80 ticks (:13564-13577) — C# only in state 1
  (`ai[2]==1`, `:28914-28917`), `localAI[0] > 80`, 10-speed dmg 25, jitters
  `Next(-40,41)*0.05`, offset ×8 (`:28972-28986`); state 0 fires every
  **200** ticks at **8**-speed dmg 25 with the same jitters/offset
  (`:28894-28908`) — the port fires in every state at 10-speed flat 25 with
  no jitter/offset and no 200-tick idle volley.
* Saw/vice dash shapes (:13506-13559) approximate the rise thresholds
  (−200/−280) but miss the state-2 exit tests, the 600-tick resets' gates and
  the head-state coupling — the faithful style33 module documents all of it.
* Instant `hp = 0` when the head is gone (:13465-13468) vs the C#
  `ai[2] += 10; > 50` grace (`:27961-27970` — single-player also instant, so
  this one is behaviourally equal in solo).
* The v2 rotation writes (:13585-13606) match the C# baselines ✓.

**Verdict WRONG (known)** — this is the catalogued W82/W109 work: wire
`npc_ai_style33` (landed, unwired) and port 34/35/36 (W109 died; check its
worktree for partials before restarting). No new findings beyond the cannon's
dock-reflected idle volley and the laser's state-0 arm, which the fix wave
must include. RNG: the live limb fn draws **zero** rand values; the C# arms
draw 2 per shot (`Next(-40,41)` pairs) — stream drift on every limb shot.

## 7. Fix-wave ordering (proposed)

1. **Retinazer style-30 module + axis claim** (R1-R6) — biggest behaviour
   delta, template exists (style31), unlocks the twin-despawn coupling.
2. **Skeletron hands state machine** (H1-H5, H8-H9) — the slam never flies
   and the hands outlive the head; both are player-visible every fight.
3. **Skeletron head gates** (S2-S5, S8-S9) + the **hands-count/red-hat slot
   split** (S3) — the split touches `world_sim/npcs.rs:3739-3745`, the hand
   mirror (H11), the caster latch and ANALYSIS:5500-5534; do it in one round.
4. **Prime despawn timers** (P2, S8) + the difficulty-flag seam (S1/P1) —
   small, cross-cutting; the despawn seam already exists (`encourage_despawn`).
5. W82/W109 limb wave (unchanged priority, census §6 queue).

DungeonGuardian kind + the `downed_skeletron` dungeon-spawn gate
(`NPC.cs:2643-2651`) is a separate roster/spawner task, not this slice.

## Verification

Docs-only: no compiler invoked, no tests run, no code touched. Every verdict
above cites both sides at `c7fe175d`; the C# arms were read whole (style 30
`:26405-27014`, style 11 `:21919-22299`, style 12 `:22300-22614`, style 32
`:27626-27942`, styles 35/36 shot blocks `:28640-28759`/`:28900-28991`).
