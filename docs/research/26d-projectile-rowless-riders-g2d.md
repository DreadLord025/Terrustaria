# 26d — the g2d rowless riders: the 981 row + the phaseblade/491 task-seeds + the Shimmer drop-hunk

> Coded 2026-11-21 on `task/W4-g2-proj-ai-missing-b` (resume of the frozen
> 81c4ab85 tip; the g2 census `d0708e3b` + the reconciliation round
> `d5cdd23e`/`4674c364`/`63a6db26`/`81c4ab85` all landed and pushed).
> Code-only contract: no cargo/tests/rustfmt/graphify; witnesses WRITTEN
> NOT RUN. C# reads are direct-file windows on `decompiled/Terraria/`.

## 1. The rider census (unrowed kinds whose AI arms are LIVE)

The g2 lane's rowless set after the nine-row round (`af472214`/`ee857a36`):

| kind(s) | aiStyle | AI arm status | disposition this round |
| --- | --- | --- | --- |
| 981 SilverBullet | 1 | the GENERIC bullet flight is its whole AI (no per-type `AI_001` state arm) | **ROWED — batch A `8bab64583`** |
| 1059-1072, 1074-1076, 1101, 1102 (the 19 phaseblades) | 3 | `ai_boomerang` is armed but EXPLICITLY excludes the phaseblades (their outbound/return/swing machine is unported) | **TASK-SEED §3** |
| 491 FlyingKnife | 9 | `ai_magic_missile` is armed but its head routes 491 to the UNPORTED `_Old` variant | **TASK-SEED §4** |
| 23 Harpoon / 24 SpikyBall | 13 / 14 | style 13 has NO dispatch arm; style 14's generic `ai_spear` tail exists but the 24's platform `flag6=false` + bounce row are owed | out of the named scope — noted, unclaimed |

The 27 WaterBolt is ALREADY rowed (`data.rs` WATER_BOLT, the aiStyle-8
`ai_ball_of_fire` arm carries its lava-bounce block) — the e3e "no row"
flag was stale for it; no work owed.

## 2. Batch A — the 981 row (LANDED `8bab64583`, pushed)

`SetDefaults` `Projectile.cs:9340-9353` verbatim: 4x4, aiStyle 1,
friendly, penetrate 1, light 0.5, alpha 255, scale 1.2, timeLeft 600,
ranged, extraUpdates 1 (NOT the hardmode bullets' 2). Stock tileCollide.

- The whole AI is the generic aiStyle-1 flight (`AI_001`'s tail
  `:57572-57583` + the ±16 clamp) — the only per-type 981 site inside
  `AI_001` is the `Main.rand.Next(6) == 0` dust-279 gate (`:56746-56752`),
  dust-only, skipped with the layer (every other bullet's trail precedent).
- `ProjLight` `:16343-16347`: its OWN colour arm `num2 *= 0.7; num3 *= 0.7`
  — `(l, 0.7l, 0.7l)`, NOT the common bullet arm's 0.1 blue. Landed in
  `proj_lights.rs` between the 14/110/180/242 group and the 15 arm.
- `Kill` `:76040-76049`: the same `HitTiles` + `SoundID.Item10` pair as the
  common bullet arm (`:76035-76039`) plus the `SilverBulletSparkle`
  ParticleOrchestrator request — the particle layer is unported; the kind
  folds onto the common group in `special_bullet_kill`.
- `Damage_PVE_Inner` `:13385-13388`: `type == 981 && targetNPC.type == 104`
  (NPCID.Werewolf, `NPCID.cs:10885`) → dmg x3. **INERT SKIP**: the Werewolf
  is an unported NPC kind. NOT gated on the port's internal id 104 — that
  is DD2SkeletonT1 (`npc/data.rs:288`, the internal enum space, NOT the
  vanilla id); a false triple there would corrupt the DD2 lane. The cited
  skip comment sits at the C# order site in `damage_pve` (above the
  `:13389` yoyo doublings).
- `GetAlpha` `:78927-78930` returns White — draw layer, no port field.
- 3 witnesses WRITTEN NOT RUN: `data.rs::the_w4_g2d_silver_bullet_row_is_verbatim`,
  `projectile.rs::w4_g2d_silver_bullet_tests` (the spawn+flight sub-step
  shape, the ProjLight own-arm pin).

## 3. TASK-SEED — the phaseblade 19-set (d75e's cited sites)

**Why seeded, not rowed:** `ai_boomerang` (S1-P-i08) hard-excludes the
phaseblades at every generic site (the whoosh head, the speed table, the
rotation tail). Rowing the 19 kinds without their machine would fly them
as generic 9/0.4 boomerangs — wrong speed (40/3), no swing state, no
`ai[0] == 2` hit narrowing consumer. The rows + the machine must land
TOGETHER (the f19-F4 arm+row-together precedent).

The set (`ProjectileID.cs:192`): 1059-1072, 1074-1076, 1101, 1102 — 19
kinds; **1073 is NOT a phaseblade** (the hostile trap dart, 10x10 aiStyle 1,
`:10038-10046`). The Rust predicate `is_a_phaseblade` (`projectile.rs:14435`)
already carries the exact set.

### The rows (three SetDefaults groups, verbatim)

- `:10006-10021` — 1059-1064, 1071, 1075, 1101: **38x38**, aiStyle 3,
  friendly, penetrate -1, melee, netImportant, usesLocalNPCImmunity +
  localNPCHitCooldown 10. The 1101 adds glowMask 372 (draw seam).
- `:10022-10037` — 1065-1070, 1072, 1076, 1102: **46x46**, the same flags.
  The 1102 adds glowMask 373.
- `:10047-10058` — 1074 (the Orange Phasesaber's brand twin): **46x46**,
  the same flags, no glowMask.
- No `Main.projFrames` overrides (the stock 1); `GetAlpha` `:78862-78865`
  White; DisplayDoll `:22730-22734` refuses the draw (draw layer).

### The machine (AI_003's phaseblade arms — the port owed)

- Head `:38985-39023`: the out-of-world return latch (`InWorld(Center,10)`
  fail → ai0=1, local-owner), the `timeLeft < 5` floor + return latch, the
  ±2π rotation wrap, the held-phaseblade `controlUseItem` return latch
  (`HeldItem.shoot` in the set), the `Item.GetPhaseColor(type) * 0.5`
  two-cell AddLight (NOT the 1074).
- Outbound `:39041-39108`: `ai[1] >= 30` → vy++, vx x0.96, vy cap 16; the
  falling rotation lerps toward 2.335 (dir>0) / -3.948 (dir<0) with the
  `(rot*9 + target ± 2π)/10` wrap trick; the rising/pre-30 spin
  `rot += 0.4 * direction`. Sounds: Item15 cooldown 14; the 1074 Item7
  cooldown 9 (rising) / 11 pitch -0.3 (pre-30) — d75's 0e700d9ab census
  rows, SIBLING-CUED at merge.
- Swing `ai[0] == 2` `:39342-39364`: vy<0 → x0.5; rot = 2.335; vx = 0;
  vy++; the localAI[1] one-shot sound latch (Item15; the 1074 Item182 +
  localAI[0] = -1).
- Return `:39367-39397` + `:39398-39491`: direction faces the owner,
  `rot = velocity.ToRotation() - 2.335`, the localAI[0] sound latch
  (Item15; 1074 Item182 pitch -0.2 / Item7 pitch -0.5); tileCollide=false;
  **speed 40 / accel 3, flag3 = false** (NO meleeSpeed scale — the
  `:39462` gate); `extraUpdates = (int)(dist/3000)` (+1 for the 1074),
  cap 20 (`:39480-39491`) — NO 3000px kill (the generic arm's kill does
  not apply); the owner-box catch `:39551-39554` stands.
- HandleMovement tile-hit `:19375-19511`: `ai[0] = 2`; when X unchanged
  && lastY > 0 the impact dust fires — the 1074's 15x type-57/43 loop
  (`Next(2)` type pick, `Next(10)` variant, `Next(4)` noGravity,
  `Next(-4,6)` X jitter, `Next(300)` Y spread) and the others' 5x
  NewDustPerfect with the per-kind colour table (1059/1065→88,
  1060/1066→90, 1061/1067→89, 1062/1068→86, 1064/1070→87, 1063/1069→91,
  1071/1072→138, 1075/1076→325, 1101/1102→267+GetPhaseColor) — the RNG
  draws must be KEPT in order when the dust lane wires them (the g2
  64-draw precedent).
- `Damage_PVE_Inner` `:13306-13333`: the `ai[0] == 2` swing damage x0.6
  (default) / **x0.5** (1065-1070, 1072, 1076), knockback x0.6, direction
  faces the owner-vs-target side. UNPORTED — lands with the machine.
- `:12943-12946`: the 1074's `targetNPC.KeyBrandStrike` — the NPC-side
  Brand of the Inferno debuff machine, an unported dependency (cited skip
  until the NPC lane lands it).
- `GetCollisionParams` `:19957-19961`: the phaseblades collide on a
  **10x10** box, resizeAnchor (0.5, 0.5) — `collision_params`
  (`projectile.rs:10147`) owes the arm (the g2 −20 shrink precedent; the
  doc there already records the phaseblade arm as out-of-scope).
- `Damage_GetHitbox` `:14962-14965` inflate(-14, 0) at ai0==2 — ALREADY
  PORTED (`damage_hitbox`, `projectile.rs:22873`).
- Shimmer `:22384-22402` (the swing refusal + the ai reset/float) —
  ALREADY PORTED (the g2 staged helper, d53-activated; §5).

## 4. TASK-SEED — the 491 FlyingKnife (e3e's LIVE `_Old` machine)

**Why seeded, not rowed:** `ai_magic_missile`'s doc (`projectile.rs:7948`)
records the skip: the C# head `:43197-43201` routes 491 to
`AI_009_MagicMissiles_Old` and RETURNS — the ported modern machine
(32-cruise homing) is the WRONG body for it. Row + `_Old` must land
together.

- Row `:5404-5412`: **26x26**, aiStyle 9, friendly, **melee**, penetrate
  -1. Stock life 3600 / tileCollide / scale.
- `_Old` `:43427-43583`: the channel block (local owner, ai0 <= 0): speed
  num = **20** for the 491; the cursor aim (`LimitPointToPlayerReachableArea`);
  ai0 < 0 increments; the **<100f close-range boost** (vel x1.1 capped to
  20, the ai0==0 → **-10** latch); the far steer `(vel*4 + aimUnit*20)/5`
  (the 491's blend — the other kinds SET the velocity); the x1000
  int-truncated netUpdate compares; the release arm sets ai0=1 + netUpdate
  (the 491 SKIPS the away-boost block, `:43509`).
- The return hunt `:43540-43554`: localAI[0]++; ai0>0 && localAI[0]>15 →
  tileCollide=false, the owner seek (`<20f` kill), `(vel*5 + dir*25)/6`.
- The rotation block `:43556-43567`: ai0<0 or zero vel → +0.3; ai0>0 →
  +0.3 x direction; else `atan2(vy,vx) + 1.57`. The 2000px owner-distance
  kill `:43568-43571`. The `Next(2)` dust gate → `Next(3)` type 15/57/58
  (`:43572-43582`, the draws stay when wired). The shared tail vy cap 16
  (`:43649-43652`); the 491 sets `flag = true` so the generic
  `!flag` rotation (`:43645-43648`) does NOT apply.
- HandleMovement `:18464-18477` (inside the `!correctSlopeCollision`
  chain, `:18398`): the tile hit sets ai0 = **-10** when ai0 <= 0, then
  reverses each changed axis x-1 gated `|last| > 0` — the knife BOUNCES
  while aimed, and the aiStyle-9 generic kill (`:19325-19347`, the
  local-owner gate; the 79 carve-out) collects it after the release.
  Nesting re-verify owed at port time (the depth scan: the 491 arm at
  brace depth 5, the style-9 kill at depth 3 — different chains).
- `GetCollisionParams` `:19957-19961`: the 491 collides on a **10x10**
  box (the same group as the phaseblades).
- `GetAlpha` `:79204-79207` White (draw). No Kill spawn arm.

## 5. The Shimmer() drop-hunk (f20d-F5: g2's staged copy DROPS at merge)

d53 batch 4 `fa7b331d3` landed the generic wet machine
(`projectile_wet::wet_block`, the authoritative `shimmer_wet` writer) and
FINAL item 2 **`804ad5715` ACTIVATED** the g2 staged helper on
`origin/task/W4-d53-dust-sites-projectile` (tip `88094c4ce`): the `:17011`
caller-gate wired in the dispatch right after `wet_block`, OUTSIDE the
ignoreWater block, no active guard (verbatim). f20d's RETIRE `0883a80bc`
re-verified the keep-one by line-diff: the two copies differ on **6 lines**
(fn → pub(crate) visibility, the two STAGED dead_code markers, the doc
header, the −8-comment typo fix). **The surviving copy is d53's; g2's
staged copy DROPS.**

### The exact drop-hunk (this branch's tree, tip 8bab64583)

`crates/tr-app/src/projectile.rs`:

1. **DELETE lines 10189-10376** — the whole staged block: the `///`
   doc header starting `/// \`Projectile.Shimmer()\` (\`Projectile.cs:22361-22516\`)`
   (:10189) through `shimmer_two_cell_kill`'s closing brace (:10376).
   That is: the doc + `#[allow(dead_code)] // STAGED: the :17011 caller
   lands with the generic wet block` + `fn shimmer(&mut self, world: &World)`
   (:10208-10357) and the probe's doc + `#[allow(dead_code)] // STAGED:
   only \`shimmer\` calls it (the wet block is unported)` +
   `fn shimmer_two_cell_kill(&self, world: &World) -> bool` (:10359-10376).
   d53's adopted pair (its `pub(crate) fn shimmer` + the probe, byte-identical
   bodies bar the -10 comment fix) survives the merge.
2. **KEEP** the five `w4_g2_shimmer_*` witnesses (:45192-45293 pre-batch-A;
   the module comment at :45185-45187 saying "STAGED, unwired" should be
   re-worded to "driven directly; the :17011 gate is d53's") — they call
   `p.shimmer(&w)` and compile against d53's surviving pub(crate) copy.
   d53 carries its own 7 witnesses (`w4_d53_shimmer_tests`); the two sets
   are disjoint fn names — keep BOTH (the union precedent).
3. The `anchors.txt:900` marker `fn shimmer(&mut self, world: &World)` is
   a SUBSTRING of d53's `pub(crate) fn shimmer(&mut self, world: &World)`
   — the canary survives the drop; update the row's name text from
   "STAGED/unwired" to "ACTIVATED by d53 804ad5715" (this round, below).
4. The g2 comment typo ("-8 clamp" in the 25/17 arm doc, :10346-10347)
   dies with the block — d53's copy already reads -10 (`:22511-22514`).

No code deletion lands on THIS branch (the staged copy is what this
branch's witnesses compile against pre-merge); the hunk above is the
merger's instruction, per the f20d-F5 contract.

## Commits (this round)

1. `8bab64583` — batch A: the 981 row + ProjLight arm + Kill fold + the
   inert Werewolf cite + 3 witnesses (PUSHED).
2. (this doc + the ANALYSIS/PORT_MAP/anchors rows + the staged-copy
   superseded notes) — the seeds + the drop-hunk publication.