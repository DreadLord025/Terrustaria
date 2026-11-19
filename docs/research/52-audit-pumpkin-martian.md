# 52 — Census slice B-4: Pumpkin Moon gaps + Martian family audit (distrust-verbatim)

**Mode:** READ-ONLY audit, code-only round — **no compiler invoked**, no cargo/rustc/
rustfmt/tests/dotnet/graphify; reads + git only. Base: `origin/main` @ `c7fe175d`.
Method: the f1701eb0 defect catalogue applied (verbatim claims re-read against the
C# line-by-line; RNG draw order/count, missing writes, invented constants, inverted
gates, cite-vs-content). Every verdict below carries exact C#/Rust lines.

Scope (census §B-4, `origin/docs/entity-ai-census:docs/research/50-entity-ai-census.md`):
(1) Pumpkin Moon 325/327 live arms + the six W101 merger gaps; (2) the Martian
family — style 72/73 landed modules, R10 defaults branch, the LIVE saucer 392-395
machines vs C# whole; (3) the five citation smells.

**ID-key note (distrust-verbatim applies to the census too):** the census's kind
column uses the port's legacy `vanilla_id` band, not `NPCID.cs`. Actual C# ids:
BrainScrambler **381**, RayGunner **382**, MartianSaucer **392**, SaucerTurret
**393**, SaucerCannon **394**, SaucerCore **395**, MartianProbe **399**,
MartianDrone **388**, MoonLordHead/Hand/Core **396/397/398**, DD2LanePortal
**549**, towers **493/507/422/517** (`ID/NPCID.cs:11439-11479`, `:11659-11711`,
`:11775`). The port's legacy ids (`npc.rs:15276-15279` → 89/90/91/92) are a
known mapping, not a defect.

## Verdict table

| # | Target | Verdict | One-line |
|---|--------|---------|----------|
| 1 | MourningWood 325 live arm (`npc_ai_pumpkin_boss::mourning_wood_step`) | **FAITHFUL** | whole type-325 path verbatim incl. RNG order; light skipped (documented) |
| 2 | Pumpking 327 live arm (`pumpking_step`) | **FAITHFUL** (gaps documented) | arm verbatim; blade spawns deferred — style-59 module landed UNWIRED on `297fe054` |
| 3 | W101 six merger gaps (FindFrame 327, rotation whitelist, blade/scythe, light, cloak/face ghosts) | **DEVIATION-DOCUMENTED → still open** | all six confirmed present on main; §2 below |
| 4 | Saucer mounts 392/393/394 (`npc.rs::update_saucer_mount` :9187-9440) | **WRONG** | seat offsets unrotated/unscaled, 450 battery collapsed to one emitter with broken velocities, phase bridge MISSING, despawn-restore inverted; R34's style-75 module (`989c9055`, UNWIRED) is the fix vehicle — §2.3a |
| 5 | Saucer core 395 (`npc.rs::update_saucer_core` :12718-12944) | **WRONG on main** (machine otherwise faithful; R35's branch fixes verified) | never invulnerable (`dontTakeDamage` default + phase-1 clear missing), phase-2 deathray born dead (`ai0` dropped), `-1 → 0` restore missing; all three fixed on `89b65c54` (spot-checked C#-correct, §2.3 boundary note) |
| 6 | Deathray proj 447 (`projectile.rs::ai_saucer_deathray` :10117-10208) | **DEVIATION-DOCUMENTED + 1 WRONG** | beam maths faithful; shell-owner window gate skipped (C# `flag38`), core-spawn payload broken upstream (#5) |
| 7 | FindFrame 392/393/394/395 (`npc.rs` :4736-4812) | **FAITHFUL** | all four cases match `NPC.cs:71806-71883` incl. the 5-band aim cascade |
| 8 | MartianProbe 399 style 80 (`update_martian_probe` :8597-8680) | **DEVIATION-DOCUMENTED** | constants faithful; `SolidTile` → `TILE_SOLID` proxy, collideX fold, light skipped, escape keeps living until the sim pass |
| 9 | MartianDrone 388 style 74 (`update_martian_drone` :8700-8859) | **DEVIATION-DOCUMENTED** | machine faithful; per-tick `Next(4)` charge dust gate (:35382) skipped → stream drift; box-resize/difficulty folds documented |
| 10 | BrainScrambler/RayGunner style-3 shooter (`update_martian_shooter` :8890-9052) | **FAITHFUL (spot-check)** | 381/382 rows exact (80 cd, 436/24 @11, 438/30 @7, ±100×0.4 + 85-116 rescale, +6 y); stealth gate documented-off |
| 11 | ForceBubble 384 style 72 (`npc_ai_style72.rs`, main) | **FAITHFUL, UNWIRED** | arm `NPC.cs:35080-35101` RNG-free; no kind on main — R10 branch `5f64226c` carries the defaults (wire ids 213/214), NOT merged |
| 12 | MartianTurret 387 style 73 (`npc_ai_style73.rs`, branch `be1d00d9`) | **FAITHFUL (doc-level), UNWIRED, NOT on main** | arm `NPC.cs:35102-35217`; census note inside the module ("393 … on aiStyle 76") is wrong — 393 rides 75 |
| 13 | Citation smell: moon-lord hand/head cross-swap | **CENSUS WRONG — cites already correct on main** | hand cites AI_078 :41577-42082, head cites AI_079 :42084-42457; C# fns at :41577/:42084 |
| 14 | Citation smell: `update_saucer_core` cites Prime range | **CENSUS WRONG — misattribution** | :27626-27942 belongs to `update_prime`'s doc (:12950/:12956); the core's own cites :36071-36520 are correct |
| 15 | Citation smell: `update_dd2_lane` cites AI_077 | **CENSUS WRONG — cite correct** | doc cites :40970-41047 = the real style-106 arm; behaviour faithful (one missing `dontTakeDamage` clear at death, minor) |
| 16 | Citation smell: towers under a cultist-named fn | **CONFIRMED — and worse: the fn body is NOT the tower machine** | `update_ancient_cultist` :12553-12606 is an invented stand-in; the real CultistBoss 439 rides `update_lunatic_cultist` (style 84) since S1-D-3b |
| 17 | Saucer duplicate dispatch arms | **CONFIRMED** | live :3887-3890, dead :4036-4038 (first match wins); wiring edits must hit the FIRST arm |

## 1. Pumpkin Moon — the live arms are faithful; the gaps are all render/wiring

### 1.1 MourningWood 325 (`NPC.cs:32761-33043` type-325 paths)

Re-read whole against `mourning_wood_step` (`npc_ai_pumpkin_boss.rs:84-295`):
hp bands 2/3/4 (f64 compares kept, `:32771-32778`), dawn `EncourageDespawn(10)` +
speed 8 (`:32787-32791`), idle triple-time counter + `Next(3,5)`/`Next(1,3)` pick
(`:32792-32816`), stomp 15-tick 325 @50 speed-10 TOP-aim ±20×0.01 (`:32846-32865`),
spin 8-tick strict (60,240) window, 10+|dx|·0.004 cap 14, `(dy − Next(50,201))`,
±30×0.01 jitters then `Next(326,329)` kind LAST (`:32893-32920` — the named-arg
textual order is preserved), beam 30-tick @75 ±20×0.001 (`:32922-32941`), barrage
10-tick @50 ±30×0.005 (`:32942-32969`), the |dx|<50 damp + ±0.1 snap and
`(vx*20 ± num836)/21` (`:32970-32992`), the 80×20 probe hover with the exact
−0.025/−0.2/−4 and +0.025/+0.5/10 ramps (`:32993-33042`). **No defect-class hit.**
The `:32785` light is a documented skip (module docs :23-28); the type-344
(Everscream) branches are out of roster (S1-F-13c).

### 1.2 Pumpking 327 (`NPC.cs:33044-33197`)

`pumpking_step` (:302-448) matches line-for-line: the 7-tick/5-row `localAI`
cycle, the >300 `Next(3)` mode pick, the mode-0 lob with the `WorldGen.SolidTile`
origin gate and `(dy + Next(50,201)) * 0.2` flattening (five draws, kind last),
the init retarget, the 2000-px flee re-check, dawn fall, hover `num875` 6/8/10/12
bands on /15, charge 16 on /50 with the 600-tick/mode exit, flee 0.95 decays +
`EncourageDespawn(500)`, `rotation = vx * -0.02`. **No defect-class hit.**

**Blade wiring pending (per task):** the two `NPC.NewNPC(328)` spawns
(`:33093-33103` — left `ai[0]=-1`, right `ai[0]=1, ai[3]=150`) are deferred in the
module (docs :31-35). The blade machine itself landed **UNWIRED** on
`task/S1-D-3-aistyle59` (`297fe054`, `npc_ai_style59.rs`, arm `NPC.cs:33198-33373`);
its module doc re-derives the carrier facts correctly (incl. the `ai[3]==2`
face-mode 91-tick scythe-329 volley and the death-cue RNG note). NOT on main;
the wiring wave must add kind 328 + the style-59 claim + the two spawns together.

### 1.3 The six W101 merger gaps — all still open on main (confirmed)

| Gap | C# | Port state on main |
|-----|----|--------------------|
| FindFrame 327 | `NPC.cs:72296-72356` — the ai[3]-driven bidirectional frame walk (mode 0: rows >5 up / >0 down; mode 1: 4..10 band; mode 2: 2..8 band; ±6 counter wraps over 12 rows) | `npc.rs:6319` routes Pumpking to `walker_frame` (:4312) — a walk-speed strip, wrong machine. FindFrame 325 (`:72491-72520`, idle 5-row / walk rows 5-9 at >4 counter) likewise rides the walker strip |
| Rotation whitelist | `Main.cs` generic draw rotates by `npc.rotation`; the Pumpking writes `vx * -0.02` (`:33196`) | `render/entities.rs:3018-3065` whitelist lacks `Pumpking` — the tilt is computed but never drawn (S1-G-19 class) |
| Blade 328 / scythe 329 | `:33093-33103`, `:33198-33373` | deferred (§1.2); style-59 module on branch only |
| Type-327 light | `:33059` `AddLight(Center, 0.4, 0.4, 0.2)` | documented skip → S1-E-5 |
| Cloak draw | `Main.cs:22662-22668` — PumpkingCloak at Centre+80, row = `localAI[1]`, rotation 0, scale 1 | absent (no cloak overlay in `pumpkin_moon_draw`, `entities.rs:1171-1186`) |
| Face + ghosts | `Main.cs:25669-25688` — PumpkingFace/TreeFace + 9 ghosts each, **2 `Main.rand.Next(-10,11)` per ghost = 18 draws per NPC per FRAME** | parked (documented at `entities.rs:2880-2884`). NOTE: these are draw-pass RNG — the wiring round must decide the stream (draw RNG vs sim RNG) before porting; they do NOT touch the `UpdateNpcs` stream |

## 2. Martian family

### 2.1 What serves 392-395 (census answer)

Dispatch (`npc.rs:3887-3890`, kind-first): core 395 → `update_saucer_core`
(:12718), shell/turret/cannon 392/393/394 → `update_saucer_mount` (:9187). The
C# arms: style **75** `NPC.cs:35501-36070` (mounts; the 392 shell's batteries +
the phase bridge live HERE, not in the core), style **76** `:36071-36519`
(core). The pre-pass (`world_sim/npcs.rs:2472-2530`) mirrors the core's
`ai[3]`/`ai[0]`/rotation/facing + crew counts; the crew spawn resolve
(:4084-4124) matches the C# init (`:36073-36103`: 2×393 + 2×394 at
`Centre.X + seat*300 − 150`, shell at Centre, `ai[1]` seats). The duplicate
arm at :4036-4038 is DEAD (first match wins) — confirmed, cleanup row below.

### 2.2 `update_saucer_mount` — WRONG (5 defects)

**M1 — seat offsets unrotated and unscaled (all mounts).** C# builds
`vector132` then `*= core.scale` then `.RotatedBy(core.rotation)` BEFORE adding
to the core's centre (`:35547-35551` shell (0,2); `:35623-35627` turret
(±60,29); `:35637-35641` cannon (±49,−13); `:35652-35656` Dutchman cannon
((−122+68·seat)·flip, −6)). The port adds the RAW offsets
(`npc.rs:9193-9199`) — no scale, no rotation. The family tilts its sprites
(rotation copied :9217) but the guns stay on unrotated seats: during the core's
phase-1 wobble (rotation up to 2π) and phase-2 the mounts visually detach.
Fix: rotate+scale the offset vector by `carrier_rot`/core scale in the pre-pass
payload (add `carrier_scale`), exactly the C# order.

**M2 — the 450 missile battery is a one-emitter stand-in with broken maths.**
C# (`:35955-36026`): FOUR independent emitters gated per-slot
(`localAI[0..3]` aliveness), each `Main.rand.Next(maxValue4/5) == 0`
(60/90 in the 280-400 window, 90/60 in 440-580, 120/120 in phase 2), then per
shot: velocity `new Vector2(∓NextDouble()*3, ±1).RotatedBy((NextDouble()-0.5)*0.785…)*3`
and position `Centre ∓ UnitX*Next(50,70) + UnitY*Next(30,45)` (turret pair) /
`∓ UnitX*Next(30,60) + UnitY*Next(-30,-10)` (cannon pair). The port
(`npc.rs:9371-9405`) draws ONE `next_float()` for the whole battery, fires at
most ONE missile per tick, reuses the gate roll as the velocity X factor
(`-(roll*3.0)*ang.cos()*3.0` — at roll≈1/60 the X speed is ≈0.05·cos instead of
up to 3·cos; the cannon emitters' C# X factor is `NextDouble()*2`, not 3 —
`:36008`/`:36019` — the port's `roll*2.0` is corrupted the same way),
hardcodes the emitter positions (−55,+35 / −45,−20) and folds
both sides into one `side`-mirrored emitter. The group stand-in is acknowledged
in a comment (:9373-9376) but the velocity corruption is not. RNG count per
window tick: C# 4 gate rolls + up to 4×(2 NextDouble + 2 Next) shot draws; port
1 roll + ≤2. Fix: port the four emitters verbatim (the pre-pass already carries
`crew_turrets`/`crew_cannons`; per-slot aliveness needs the 4 slot flags or the
count-gated approximation must be re-documented honestly).

**M3 — the phase bridge is MISSING.** C# `:35554-35612`: when the core is in
phase 0 and all four guns are dead, the SHELL sets `core.ai[0] = 3` (classic)
or `1` (expert) + zeroes ai[1..3]. The port's mount fn has no bridge at all;
the test `the_dead_crew_opening_the_core_kills_it`
(`world_sim/npcs.rs:7977-8008`) asserts `core.is_some() || invasion_type == 0`
— a tautology that admits the gap ("the bridge lands … its own follow-up
slice"). **Consequence: killing the four guns never opens the core — the boss
fight cannot progress.** Fix: in the shell arm, when `carrier_phase == 0` and
`crew_turrets == 0 && crew_cannons == 0`, request the core phase write (the
pre-pass write-through seam), classic → 3 / expert → 1 per the difficulty flag.

**M4 — the despawn-ride restore is inverted.** C# `:36130-36149`: in −1/−2 the
core climbs (`vy -= 0.4`), `EncourageDespawn(10)`, and when the player is ALIVE
restores `timeLeft = 300`, −2 → 2, ai[1..3] = 0 (−1 stays −1 until the far gate
re-runs; the `ai[0]==0` self-assign at :36141 is a no-op). The port
(`npc.rs:12744-12757`) restores when `player.hp > 0` but maps **−1 → 0**
(invented) and never restores −2 → 2 in the same tick shape; the dead-player
arm forces `ai[0] = -1` even from −2 (C# keeps −2). Also `EncourageDespawn(10)`
and `timeLeft = 300` are absent (the port comments the timeLeft fold). Fix:
mirror :36130-36149 exactly; the despawn timer seam exists
(`encourage_despawn`, `npc.rs:1855`).

**M5 — the shell's phase-0 deathray window is right, but see C2 below for the
core's.** Shell 447 spawn (`npc.rs:9345-9358`) matches `:35907-35920`
(t==20 entry, `ai0 = slot+1`, 50 dmg) — faithful. Turret 449 (280-420, %6,
±0.785/3 jitter, 16 speed, 35 dmg, Y-clamped aim outside the window) matches
`:35851-35893`; cannon 448 (440-580, %20, ±0.785, 8 speed, side·36+8 spawn,
ai1=20) matches `:35823-35849`. The `core_open` gate (`!= 1 && >= 0`,
:9223) matches the C# `flag81/82/84` (`!= 1 && != -1 && != -2`) for the
reachable phase set {−2,−1,0,1,2,3}. Dutchman cannon block matches
`:35754-35822` (240 reload, ×14 −5Y shot, 8-ray aim with the 9−x mirror).
Sound calls (Item12/Item39) are the documented S1-E-21g territory.

### 2.3 `update_saucer_core` — WRONG (2 defects), machine otherwise faithful

The scan cycle (600-tick, six bands, the 19/279/439 skips, the 16/20-speed
lerps, the ground-clearance −4 clamp), the phase-1 wobble (150 ticks, the
cos-not-sin `UnitY.RotatedBy(...).Y` bands ×0.2/0.3/0.4 + the final spin), the
phase-2 gun cycle (3600 ticks, %120≥60 window, rising-edge fire, 8·ai[2] dash,
16-lerp chase) and the off-world clamp-then-kill all match
`NPC.cs:36151-36518` line-for-line (incl. the `ground_clearance` helper
`npc.rs:1832-1850` vs the C# 150-row scan with the `num--` break).

**C1 — the core is never invulnerable.** C# `SetDefaults` type 395 sets
`dontTakeDamage = true` (`NPC.cs:14060`); the style-76 arm clears it ONLY in
phase 1 (`:36383`). The port's `Npc::new` defaults `dont_take_damage: false`
(`npc.rs:2304`) and neither the kind defaults nor `update_saucer_core` ever set
it (grep: no write in :12718-12944). **Consequence: the 10000-hp core can be
shot dead during the scan/gun phases with the guns still up — the entire
"kill the crew first" fight structure is bypassed.** Fix: set
`dont_take_damage = true` in the core's defaults row and
`self.dont_take_damage = false` at the phase-1 entry (`:36383`'s slot).

**C2 — the phase-2 deathray is born dead.** C# `:36437` spawns the 447 with
`ai0 = base.whoAmI + 1` (the CORE's slot). The port pushes through the plain
`wants_hostile_shot` tuple (`npc.rs:12904`) which carries NO ai payload — the
drain (`world_sim/npcs.rs:3884-3920`) spawns it with `ai0 = 0`, and
`ai_saucer_deathray` kills any ray with `ai[0] == 0` on its first tick
(`projectile.rs:10132`). **Consequence: the core's signature phase-2 deathray
never exists for more than the spawn tick.** The shell's phase-0 ray uses
`wants_hostile_shot_ai` correctly (:9349-9358) — the fix is to route the core's
shot through the same `HostileShotAi` seam with `ai0 = self.slot + 1`. The
drain's comment (:3869-3873) claims both sites carry the payload; only one does.

**Branch boundary (LEAD-S76 coordination):** R35 (`task/S1-D-3-aistyle76`,
`89b65c54`, PUSHED, **NOT on main** — `git merge-base --is-ancestor` exit 1)
already fixed the core in place: C1 (defaults `dont_take_damage: kind ==
MartianSaucerCore` + the phase-1 clear), C2 (`wants_hostile_shot_ai` with
`ai0: slot+1`), M4 (the −2→2 / −1-stays restore + `timeLeft 300` +
`encourage_despawn(10)`), plus the `Math.Sign` ties and the ground-clearance
`!solidTop` term. This audit therefore did NOT deep re-audit the core; the
three spot-checks below confirm R35's fixes are C#-correct:

- **Shield drop timing** — branch adds `self.dont_take_damage = false;` at
  the top of the `ai[0]==1` wobble arm before `vx *= 0.96` — exactly
  `NPC.cs:36383` (the clear runs EVERY wobble tick, before the velocity
  decay); the construction shield matches `:14060` ✓.
- **Math.Sign tie rule** — branch `math_sign(v)`: `>0 → 1, <0 → −1, else 0`
  replaces `.signum()` (Rust returns +1 on +0.0). C# `:36229`
  `-600 * Math.Sign((center23 - player19.Center).X)` picks side 0 on the
  exact tie — the branch's `-600.0 * math_sign(cx - pcx)` matches ✓.
- **Deathray payload** — branch pushes `HostileShotAi { kind: 447, …, ai0:
  self.slot as f32 + 1.0 }` at the rising-edge fire — `:36437`'s
  `base.whoAmI + 1` ✓ (witness asserts the ray survives its first tick).

C1/C2/M4 stay in the verdict table as **main's** state; they close when R35
merges. The mount defects M1-M3 are NOT in R35's scope (style 75 ≠ 76).

### 2.3a The style-75 module (R34, `task/S1-D-3-aistyle75`) — UNWIRED, pending

R34's `989c9055` (+ `ac1f45a2`) landed `npc_ai_style75.rs` UNWIRED (only the
`mod` seam; the live `update_scutlix_rider`/`update_saucer_mount` keep
running byte-identical). Its ANALYSIS "The mount machine (S1-D-3 aiStyle-75,
code-only)" already catalogued the live-arm gaps; this audit re-verified
three against the C# and confirms them:

- **(a) grouped battery** — verified above (M2): the C# draws per-slot
  `Next(maxValue4)`/`Next(maxValue5)` gates (`:35978`/`:35989`/`:36004`/
  `:36015`), each hit its own NextDouble×2 + Next×2 chain (`:35982-35984`
  etc.). R34's cite window `:35978-36025` ✓.
- **(c) phase bridge absent** — verified above (M3): `:35552-35613` read
  whole this session (the flag80 crew scan + the classic/expert split).
  R34's module ports it; the live arm has nothing.
- **(h) unrotated/unscaled seats** — verified above (M1): `:35547-35551`
  read whole (scale-multiply THEN RotatedBy THEN rotation copy). R34's
  note "identity at scale 1 / rotation 0, but the core DOES rotate in its
  vulnerable wobble" ✓.

Also confirmed present in R34's catalogue and consistent with this
session's reads: the carrier-gone `Transform(382)`/`Transform(518)` +
orphan-death arms (`:36056-36069` read: 390→382, 416→518, else
`life=0 + HitEffect + active=false`) are absent from the live mount fn
(it early-returns at `carrier_x < 0`, `npc.rs:9188-9190`). The module is
the fix vehicle for M1/M2/M3 — the wiring wave should merge R34's module
+ claim style 75 rather than patching the live stand-in twice.

### 2.3b PORT_MAP overclaim (new finding)

`docs/PORT_MAP.md:376` claims the mounts' batteries + "the phase bridge"
are **done** ("M4-EVENT-4f; the expert-mode bridge arm = constants-off").
M3 proves the bridge is ABSENT from the live shell arm — not merely
"expert constants-off": the classic arm (`:35596-35603`) is missing too.
The row's status must drop to partial when the fix wave lands (or now,
via a docs touch-up). Similarly `:374`'s "the core phase bridge = the
follow-ups" is the honest earlier claim — :376 overwrote it.

### 2.4 Deathray 447 arm — one gate skipped

`ai_saucer_deathray` (`projectile.rs:10117-10208`) vs `Projectile.cs:30791-30910`:
the beam top (+46), the player-Bottom target, the 150-row scan with the
`len--` break, the `(len+1)*16` height + the `%16 − 2` trim, the 5-tick 4-frame
loop — all faithful. **D1:** for a SHELL owner the C# resolves the core through
`shell.ai[0]` and sets `flag38 = true`, which EXEMPTS the core from the
`ai[3]%120 >= 60 && ai[0] == 2` window check (`:30818-30823` — `!flag38 && …`);
the port applies the window gate to core owners unconditionally
(:10138-10143) and checks the shell's link by "any live core exists"
(:10136) instead of the slot link. For the shell-spawned ray (phase 0) the
port's `is_shell` path skips the window check entirely, so the live behaviour
is currently equivalent — but a core-owned ray spawned via the shell path
would diverge. Fix with C2: carry the owner kind through the payload and port
`flag38` literally. The owner-index `.max(0)` clamp (:10125) also masks the
C#'s `num593 >= 0` guard — cosmetic today, fix in the same pass.

### 2.5 Probe 399 / Drone 388 / shooters 381/382

- **Probe (style 80, `:36532-36624`)**: the 3·direction drift, the 30-row scan
  with the <15/<20 bands (−0.05/−3.5, ×0.95, +0.05/1.5), the 352-px below-player
  detect, the 60-tick flash, the escape (−0.1/−10 rise, +ai[3]·0.05/4 drift,
  180-tick or off-world → invasion) all match. Deviations: the scan uses
  `TILE_SOLID.has` without the `!solidTop`/halfbrick/slope exclusions of
  `WorldGen.SolidTile` (the Wood's port has the full probe — reuse it);
  `collideX` is folded into the integration flip (documented); the
  SkyBlue/Red ×0.65 light (:36617-36623) skipped; the C# deactivates the probe
  on the invasion tick (:36613) — the port keeps it alive until the sim pass
  retains it out (`world_sim/npcs.rs:4055`), one tick of extra life.
- **Drone (style 74, `:35218-35500`, 388 params)**: the ±π/2 rotation fold +
  `-Sign(vx)` sprite, the 200-above approach on /30, the ±π/8 cone AFTER the
  ai[3]≥120 override, the 30-tick charge with the ±50×0.04 jitter ×14, the
  20-lerp dash with the 100-px/7-speed exit, the 64-px contact and solid-fold
  explosions, the 3-tick fuse @80 — all match. **RNG deviation:** the C# draws
  `Main.rand.Next(4)` EVERY charge tick for the dust gate (`:35382`); the port
  skips the dust AND the roll (documented "no dust layer") — the charge-phase
  stream drifts vs vanilla. The explosion's 192×192 box resize is folded to
  `damage_override = 80` (documented, kind-fixed size). The 418 (Drakomire
  rider skull) param set is out of roster.
- **Shooters 381/382 (style-3 block)**: spot-check of `shooter_params`
  (:8866-8875) vs `NPC.cs:59103-59105` (cd 80), `:59247-59249` (382 speed 7;
  381 keeps the 11 default), `:59292-59294` (+6 y), `:59350-59355`
  (±100×0.4 then ×Next(85,116)·0.01, X-then-Y order), `:59432-59440`
  (436/24, 438/30) — **exact**. The stealth/itemAnimation engage gate is
  documented constants-off (:8885-8888). The pirate trio's rows ride the same
  fn (census B-5/B-6 territory, not re-audited here).

### 2.6 ForceBubble 384 / MartianTurret 387 — landed modules, wiring pending

- `npc_ai_style72.rs` (main, UNWIRED): the doc's census is correct (sole kind
  384, `NPC.cs:13888`; the arm :35080-35101 is RNG-free — officer-slot snap,
  integer half-extents, gfxOffY copy, constant light, officer-gone pop). No
  `NpcKind::ForceBubble` on main. **R10's defaults branch `5f64226c`
  (`task/S1-F-13d-martian-defaults`) is NOT merged** (`git merge-base
  --is-ancestor` exit 1): it carries the ForceBubble/MartianTurret kind rows
  (wire ids moved 189/190 → **213/214**, render keys ride vanilla 384/387,
  ALL_KINDS 240, styles 72/73, sizes/sounds). The wiring wave merges R10 +
  claims the two styles; until then both kinds are absent from the live game.
- `npc_ai_style73.rs` (branch `be1d00d9`, NOT on main): doc-level read matches
  the C# arm `:35102-35217` (the ×0.93 decay + ±0.1 snap, the 120-tick
  materialize with the alpha ramp + dontTakeDamage + frame pin + early return,
  the 120→121 latch, the −120 re-arm/60-tick cycle/justHit −30, the four-draw
  aim ×14 bolt 435 @ GetAttackDamage_ForProjectiles(35, 28)). **One census
  error inside the module doc:** "the saucer's MartianSaucerTurret 393 is a
  different kind on aiStyle 76" — 393 rides aiStyle **75** (`NPC.cs:13993+`
  SetDefaults; the 76 row is the core 395 only). Correct the doc at merge.

## 3. Citation smells — three of five are census errors, one is real and worse

Verified against the actual C# line content (function boundaries:
`AI_077_MoonLordCore` :41157, `AI_078_MoonLordHands` :41577,
`AI_079_MoonLordHead` :42084, `AI_081_TrueEyeOfCthulhu` :42459; style-106 arm
:40970-41047; style-94 arm :38968-39383; style-32 Prime :27626-27942).

1. **Hand/head cross-swap — NOT PRESENT on main.** `update_moon_lord_hand`
   doc (`npc.rs:10923`) cites `AI_078_MoonLordHands (NPC.cs:41577-42082)` ✓;
   `update_moon_lord_head` (:11080) cites `AI_079_MoonLordHead
   (NPC.cs:42084-42457)` ✓; the core (:10778) cites :41157-41575 ✓ and the
   free eye (:11166) cites :42459-42880 ✓. The census's smell was stale
   (pre-repair). **Action: strike the smell from the census; no code change.**
2. **`update_saucer_core` "cites :27626-27942" — MISATTRIBUTION.** The only
   :27626-27942 cites in npc.rs are :12950/:12956 (and test :18515) — they
   belong to `update_prime`'s doc block, which physically follows the core fn
   (the core fn ends :12944). The core's own cites (:36071-36520 etc.) are
   correct. **Action: strike the smell; the Prime cites are right where they
   sit.** (The core's REAL defects are C1/C2 above, not citations.)
3. **`update_dd2_lane` "cites AI_077" — NOT PRESENT.** The doc (:10720) cites
   `NPC.cs:40970-41047` = the actual style-106 arm ✓. Behaviour spot-check:
   the LaneSpawnRate counter/reset-then-modulo order, the 180-idle clamp, the
   AnyNPCs(548) crystal guard, the 500→600 scale lerp and the 550 death all
   match. One minor: the C# clears `dontTakeDamage` at the 550 death
   (:41040); the port does too (:10772) ✓. The `leftGate` polarity: C#
   `leftGate = ai[2] == 0` (:40972); the port stores
   `wants_lane_right = ai[2] == 1` (:10733) — equivalent naming, verify the
   sim's spawn-side consumer reads it as RIGHT (it does per the field name;
   flagged for the B-6 lane audit, not a defect here).
4. **Towers under `update_ancient_cultist` — CONFIRMED, and the body is a
   stand-in.** Dispatch `npc.rs:3937-3940` routes all four towers
   (493/507/422/517) to `update_ancient_cultist` (:12553-12606). The real
   CultistBoss 439 rides `update_lunatic_cultist` (style 84, S1-D-3b) — so the
   fn name is a pure misnomer. Worse, the body is NOT the style-94 machine:
   - C# `:38970-39067`: the `ai[2] == 1` DEATH arm — velocity pinned to
     +Y·|v| eased toward 0.25, `dontTakeDamage = true`, opacity fade after
     120, death at `ai[1] >= 180`. The port instead treats `ai[2]` as a
     plain 180-tick lifetime counter for EVERY tick (:12555-12558) — a
     tower dies 180 ticks after spawn regardless of state.
   - C# `:39101-39115`: `dontTakeDamage = ShieldStrengthTower* != 0` — the
     shield mechanic. **Absent** (no shield read anywhere in the fn).
   - C# `:39131-39195`: the sine bob ✓ (port matches) BUT the ground-hover
     position corrections (the 10/20/30-tile searches, ±1.5 px), the
     world-edge clamps (60·16 + 80) and the worldSurface pin are absent.
   - C# `:39221-39224` (Stardust): `SpawnStardustMark_StardustTower` — absent.
   - C# `:39275-39299` (Vortex): the 3240-px no-LOS ritual — the port fires
     579 at the PLAYER CENTRE every qualifying tick with no `ai[1]` cooldown
     (C# re-arms `60 + Next(120)`), no 14-minion cap
     (`CountNPCS(428)+427+426 < 14`), no 10-step tile-up scan, and never the
     578 fallback when the cap is hit.
   - C# `:39301-39344` (Vortex close): the 420+Next(360) rearm — the port
     hardcodes 420 (:12599), no Next(360) draw, no 20-tile window pick (the
     100-attempt scan with the ±8/±3 exclusion boxes, lava/solid/LOS gates),
     fires at the player centre instead of the picked tile, and reuses
     `caster_counter` (the regen timer) as the shot cooldown — the two C#
     counters (`localAI[0]` regen vs `ai[1]` shot) are conflated.
   - C# `:39374-39382` (Solar): the 519 SolarFlare spawn with the
     −UnitY·(7+NextFloat·5) velocity and the 60-tick rearm — absent.
   **Verdict: WRONG.** Fix spec: rename to `update_lunar_tower` and re-port
   `NPC.cs:38968-39383` whole (the shield statics need a World seam like the
   golem's `downed_golem`; the per-type dust arms are the documented skip BUT
   their `Main.rand.Next(5)`/`Next(4)` gates DRAW — the stream decision must
   be made explicitly, the f1701eb0 lesson). This is census slice B-6's
   territory; recorded here because the smell verification exposed it.
5. **Saucer duplicate arms — CONFIRMED.** `npc.rs:3887-3890` (live) vs
   :4036-4038 (dead, same callee). First-match-wins; a wiring edit to the
   second arm silently no-ops. Cleanup: delete :4036-4038 (the
   S1-D-4c dead-arm-purge precedent).

## 4. Fix specs (priority order)

| P | Fix | Where | Spec |
|---|-----|-------|------|
| P0 | C1 core invulnerability | `npc/data.rs` defaults + `npc.rs:12868` | `dont_take_damage = true` at spawn; clear at phase-1 entry (`NPC.cs:14060`/`:36383`) — **fixed on R35 `89b65c54`, spot-checked ✓; merge closes it** |
| P0 | M3 phase bridge | `npc.rs` shell arm + pre-pass | crew dead + phase 0 → core `ai[0] = 3` (classic) / `1` (expert), zero ai[1..3] (`:35594-35612`); replace the tautology test with a real bridge witness — R34's module carries it; merge + wire |
| P0 | C2 core deathray payload | `npc.rs:12904` | route through `HostileShotAi { ai0: slot+1 }` (`:36437`); witness: the ray survives its first tick in phase 2 — **fixed on R35 `89b65c54`, spot-checked ✓** |
| P1 | M2 four-emitter battery | `npc.rs:9371-9405` | port `:35955-36026` verbatim: 4 gate rolls, per-shot NextDouble×2 + Next×2, the ±side positions (turret X×3, cannon X×2); per-slot aliveness via the pre-pass — R34's module ports it |
| P1 | M1 rotated/scaled seats | pre-pass + `npc.rs:9193-9199` | `offset *= core.scale; offset = offset.RotatedBy(core.rotation)` (`:35547-35551` etc.) — R34's module ports it |
| P1 | M4 despawn restore | `npc.rs:12744-12757` | mirror `:36130-36149`: −2→2, −1 stays, timeLeft 300, EncourageDespawn(10) — **fixed on R35 `89b65c54`, spot-checked ✓** |
| P1 | Towers re-port | `npc.rs:12553` | rename + whole-arm re-port of `:38968-39383` (B-6); shield seam + RNG-gate decision first |
| P2 | D1 deathray flag38 | `projectile.rs:10132-10148` | port `:30797-30833` literally with the owner-kind payload |
| P2 | Probe solid probe | `npc.rs:8613-8624` | reuse the full `WorldGen.SolidTile` port (the pumpkin module's `solid_tile`) |
| P2 | Drone charge roll | `npc.rs:8778-8795` | consume the `Next(4)` gate even with dust skipped, or document the stream drift in ANALYSIS |
| P2 | FindFrame 325/327 + rotation whitelist + cloak/face | `npc.rs:6317-6320`, `entities.rs:3018` | the W101 six-gap wave (§1.3); the face ghosts' 18 draws/frame need the draw-RNG decision |
| P2 | Blade wiring | merge `297fe054` + kind 328 + the two spawns | style-59 claim + `:33093-33103` in `pumpking_step`'s init |
| P3 | Dead arm purge | `npc.rs:4036-4038` | delete the duplicate saucer arm |
| P3 | PORT_MAP :376 overclaim | `docs/PORT_MAP.md:376` | the row claims batteries + phase bridge "done (M4-EVENT-4f)" — M3 proves the bridge absent (classic arm too, not just expert constants-off); drop to partial (§2.3b) |
| P3 | Census corrections | census doc | strike smells 1/2/3 (already correct on main); fix the style73 module's "393 on 76" note at merge; note the legacy-id band |

## 5. Verification performed

- `git status --porcelain` in the primary: empty (precondition).
- Worktree `TR-audit2` on `docs/audit-pumpkin-martian` from `origin/main`
  (`c7fe175d`); junctions assets/decompiled/decompiled-libs absolute.
- Every C# range quoted above was read whole in this session (NPC.cs
  :32761-33198, :35080-35217 via module docs + :35218-36626, :38968-39387,
  :40970-41049, :14045-14063, :71804-71964, :72280-72363, :72485-72524,
  :59096-59474; Projectile.cs :30789-30910; Main.cs :22655-22679,
  :25655-25694; NPCID.cs rows).
- Branch containment checked with `git merge-base --is-ancestor` /
  `git branch -a --contains` (5f64226c NOT in main; f1701eb0 in main;
  297fe054/be1d00d9 branch-only).
- **Retry session (400-death resume, same base `c7fe175d`):** re-verified
  the draft's load-bearing claims by targeted reads — M1 seats
  (`NPC.cs:35540-35551` vs `npc.rs:9193-9199`), M2 battery
  (`:35925-36029` vs `:9362-9405`, incl. the cannon ×2 X-factor
  correction), M3 bridge (`:35552-35613` whole vs the shell arm), M4
  restore (`:36130-36150` vs `:12744-12757`), C1 shield
  (`:14060`/`:36383` vs the `dont_take_damage` write census), C2 payload
  (`:36437` vs `:12904`), the shell 447 entry (`:35895-35921` vs
  `:9340-9358`), the towers stand-in (`npc.rs:12553-12606` whole), the
  dispatch arms (`npc.rs:3887-3890`/`:3937-3940`/`:4036-4038`), the
  arm-start census (`aiStyle == 57` :32761, `== 58` :33044, `== 94`
  :38968, `== 106` :40970, `== 32` :27626; `AI_077/078/079/081` at
  :41157/:41577/:42084/:42459), the moon-lord/dd2/prime cite lines
  (`npc.rs:10923`/`:11080`/`:10720`/`:12950`), the Pumpking blade spawns
  + lob draws (`NPC.cs:33060-33109` vs `npc_ai_pumpkin_boss.rs:320-348`),
  and PORT_MAP :374/:376/:394/:405. R35 boundary spot-checks:
  `git show origin/task/S1-D-3-aistyle76 -- crates/tr-app/src/npc.rs`
  (shield-drop placement, `math_sign` body, `HostileShotAi ai0`); R34
  catalogue: `git show origin/task/S1-D-3-aistyle75:docs/ANALYSIS.md`
  ("The mount machine" section) + `:36052-36070` orphan arms read.
- **No compiler invoked** — no cargo/rustc/rustfmt/tests/dotnet/graphify.
  All verdicts are reads + hand-arithmetic against the C#.
