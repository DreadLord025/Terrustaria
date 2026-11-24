# 83 — Weather reconcile (W4-d130 merger assist + conflict resolution)

**STATUS: SEALED — §9 (2026-11-21), on top of RETIRED FINAL-FINAL (§8).**
The seal closes the last three co-lane ledgers: the g67 mutual receipt
(146-rename, BOTH-sides drop-wholesale agreement, the R5/R6/R7/A2 gifts
folded — §9.1), the c39 EXACT 2-hunk graft verified-OPEN-then-FOLDED
(§9.2), the f39 season/SlimeSky keep-one receipt (§9.3), and the
WEATHER-FAMILY bible FINAL + gate command (§9.4). All folds executed:
R1–R4 via `16a792d5`, R5 via `10ee539da`, R6/R7 via `df28ea1ac`, the f39
co-lane compose via `22371b4fe`; g67d's final-phase censuses composed
(the `rain_light_tile` extract + witness). research/134 EXISTS on g67 now
(tip `e777b3e2c`) — §7's absence note is superseded, the per-item receipt
is §8.2. **The merger keeps THIS branch wholesale for rain + snow + the
weather tick** (§8.5, the 134-FINAL KEEP-d130-CANONICAL instruction).
Earlier status: EXECUTED + FINAL PHASE LANDED (union-exec commit `88cfbf38`;
final-phase commits `85250712` g11 snow fold §4, `5289d055` rain ambience,
`85b2a09c` intensity ramps, `703370e6` sandstorm cross-note §5, `cf4574e9`
f39 slime-rain fold §6 — all on this branch). §1's gate
fix landed in `50c845e5`; §3's splash union is now transplanted IN THIS BRANCH —
`spawn_splash` (the DustPool recipe), `water_splash_arm`, `draw_splashes` and the
`Option<(DustPool, SpawnCtx)>` drain seam are live in `rain.rs`/`minecart.rs`/
`weather.rs`/`app/frame/draw.rs`, the `Splash` vec stand-in is retired. **The merger
keeps THIS branch for BOTH rain AND snow** — g67's rain.rs drops wholesale (its
`update_drops`/compacted Vec/stand-in-adjacent rig keeps NOTHING; see the tip census
below), d54's `world_dust_main.rs::snowing` and g11's `main_dust.rs::snowing` retire
per §2. g67 tip census (`006bc04d`, re-fetched at union-exec): the tip is
`40128e5d` (code) + `006bc04d` (docs-only: ANALYSIS + PORT_MAP + anchors row) — NO
g67b code residuals beyond the two known commits; the doc-unique content (the
QueueRipple no-rain-caller finding, the 4×4 draw-cull widening, the RNG census, the
retired stand-in constants) is folded into ANALYSIS "The rain collision splash (g67,
transplanted W4-d130)" and the PORT_MAP splash row; the anchors row re-lands adapted
below.

Code-only pass on `task/W4-d130-weather-ambience` (base `393000e6`), reconciling the
three weather lanes that touch the same systems: **W4-d130** (this branch — the
verbatim rain fall/draw core + `Main.snowing()`), **W4-g67**
(`task/W4-g67-weather-rain-splash`, `40128e5d` + `006bc04d` — the real rain splash
lane), **W4-g11** (`task/W4-g11-dust-main-sites-1`, `94273f44`/`549021cc`/`25e180d4`
bodies + `73545f0b` wiring) and **W4-d54 batch-4** (`48343a4c`, latent
`world_dust_main.rs::snowing`). Every claim below was re-derived from the C# at the
cited lines, not from commit messages.

## 1. CRITICAL — the Rain.Update impact gate: g67 is RIGHT, d130 was WRONG (fixed)

### The C# truth (`decompiled/Terraria/Rain.cs:96-133`, re-read line-by-line)

```
:98-101  if (Main.gamePaused) return;
:102     position += velocity;
:103-109 if (Main.gameMenu)      { if (Y > screenBottom + 2000f) active = false; }
:110-116 else if (Main.remixWorld){ if (Y > screenBottom + 100f)  active = false; }
:117     else if (Collision.SolidCollision(pos,2,2)
              || pos.Y > screenBottom + 100f
              || Collision.WetCollision(pos,2,2)) {
:119         active = false;
:120         if ((float)Main.rand.Next(100) < Main.gfxQuality * 100f) {
:122-130         Dust.NewDust(pos - velocity, 2, 2, Dust.dustWater()) + 8 recipe writes
             }
         }
```

The `:117` condition is ONE combined `else if` — solid OR **below-screen** OR wet.
The `:120` `Next(100) < gfxQuality*100` gate sits INSIDE that branch, AFTER
`active = false`, so it rolls for **ALL THREE kills**, the below-screen kill
included. There is no collision-only sub-gate anywhere in the method. The only
kills that never roll are the `gameMenu` (`:103-109`, +2000f margin) and
`remixWorld` (`:110-116`, +100f margin) arms, which merely clear `active`.
The `+100f` margin is added to `screenPosition.Y + screenHeight` in RAW world
pixels — no zoom division (the C# has no zoom term here).

### Verdicts

* **g67 (`006bc04d`, code `40128e5d`) — CORRECT on the gate.** Its `update_drops`
  rolls `range(0,100) < gfx_quality*100` for `solid || below || wet` (its comment:
  "the ONE combined branch rolls the gfx gate and spawns the splash for ALL THREE
  kills (:119-131), the below-screen one included"). Matches Rain.cs:117-131
  exactly, including the raw `+ 100.0` bottom margin.
* **d130 (this branch, `ea4863d9`) — WRONG on the gate.** It re-probed
  solid/wet after the kill and rolled only when a collision probe hit, with the
  comment "the impact dust fires only on the collision arms … the off-screen half
  of `:117` spawns nothing". That comment misreads the C# — `:120` is not gated on
  which disjunct fired. Observable consequences of the old code: (a) below-screen
  kills consumed NO `Next(100)` draw (a stream divergence on Rain's RNG); (b) no
  splash on below-screen kills at any quality; (c) a redundant double probe.
  Its `100.0 / zoom` bottom margin was also a deviation from the raw `+100f`.
* **g67's one omission (kept from d130):** g67 folded out the `remixWorld` /
  `gameMenu` kill-only arms ("no port seam: the drop sim runs in-world only",
  its `update_drops` doc). `remixWorld` DOES hold in-world (the REMIX seed) —
  folding it out would make remix drops die on solid tiles AND roll the gate,
  where the C# kills them only below-screen and never rolls. d130 keeps the
  remix arm; the union must too. `gameMenu` never holds in-world — that fold is
  safe. g67 also uses `drops.swap_remove(i)` on a compacted Vec; d130's fixed
  750-slot pool keeps the C#'s slot order (`Main.rain[i]`, `Main.cs:66801`) —
  the union keeps d130's pool.

### The fix (this branch, this reconcile commit)

`crates/tr-app/src/rain.rs::update_drops`:
* the `:117` branch now evaluates `solid || below || wet` once (C# `||`
  short-circuit shape — the wet probe runs only when dry AND on-screen) and
  rolls `range(0,100) < gfx_quality*100` for every one of the three kills;
* the remix arm stays kill-only — no roll, no splash (`:110-116`);
* the bottom margin is the raw `+ 100.0` world px (g67's shape);
* the stand-in `spawn_splash` stays (g67's real DustPool spawn supersedes it at
  merge — §3), but the GATE around it is now verbatim.

Witnesses (WRITTEN NOT RUN — code-only mode):
* `the_impact_gate_rolls_for_all_three_kills` — below-screen / solid / wet kills
  each consume exactly ONE gate draw at quality 0 (hand-derived stream replay,
  `StreamReplay`), never splash at quality 0, and the below-screen kill DOES
  splash at quality 1. The solid/wet fixtures are 128-tall worlds with the
  floor/liquid at row 80 — the probes clamp their scan at `height - 40`
  (`Collision.cs:2708`/`:1594`), so a 64-tall world's row-30 floor is invisible
  (g67's `tall_ground` lesson, adopted).
* `the_remix_kill_arm_never_rolls` — the remix bottom kill consumes NO draw and
  leaves no splash.
* `forced_rain_spawns_drops_that_die_on_the_ground` comment re-pinned: on the
  64-tall fixture the drops die BELOW-SCREEN (the clamped probes never see the
  floor) — with the fixed gate that kill still splashes at quality 1, so the
  steady-state assertion stands for the right reason now.

## 2. The snowing() 3-way — KEEP d130's WIRED version

All three arms port `Main.snowing()` (`Main.cs:13032-13116`). Comparison against
the C# (re-read at `:13032-13116`):

| aspect | C# | d54 batch-4 `48343a4c` (`world_dust_main.rs`) | g11 `73545f0b` (`main_dust.rs`) | d130 `5ae7f39a` (`snow.rs`) |
| --- | --- | --- | --- | --- |
| wiring | `Main.Update` `:17238`, before `EmitDust` `:17247` | **LATENT** — body + witnesses only, "rain.rs stand-in untouched" (its own commit note) | WIRED — `scene_scan::update_scene` before `emit_dust` | WIRED — same slot |
| RNG slot | bare `Main.rand` (outside every `SwapRandom`) | takes the stream as a parameter (caller picks) | `NamedRngs::global()` — the port's bare-`Main.rand` stand-in | own splitmix stream (the WEATHER-2 per-consumer convention, documented) |
| budget `:13057` | `Main.snowDust` static (reset `Dust.cs:428`, ++ `:1747`) | `env.snow_dust` param (caller must supply the census) | **live-census stand-in** — `minecart::live_type(pool,76)` per attempt (documented SandStormCount pattern) | **the real counter** — `DustPool.snow_dust` (head reset + the 76 arm's ++), read once per call |
| tile gate `:13086` | `!t.nactive() && wall == 0`; `nactive()` = active && !actuated (`Tile.cs:240-247`) | **WRONG** — `!t.active()` misses the actuated arm (an actuated tile passes the C# gate, fails d54's) | correct — `!(t.active() && !t.inactive())` | correct — `(t.active() && !t.inactive()) \|\| wall != 0 → continue` |
| `Math.Sign(0)` `:13099` | three-way sign, 0 at 0 | `signum()` — observation-equal (sqrt(0)=0 factor) but not the C# shape | explicit three-way match | explicit three-way match |
| type-76 AI (`Dust.cs:1745-1763`) + tails (`:2073/:2081/:2127`) | shared `UpdateDust` | none (d50 lane deferred) | none — "until it lands the flakes ride the generic move/kills" | **ported** — `type_76_arm` + `type_76_tails` on the shared tick |
| draw (`Main.DrawDust` generic 76) | `Main.cs:38968-39018` | none (draw lane deferred) | none (draw lane deferred) | **ported** — `(4,4)` origin, full-dark kill |

The intensity/budget/attempt-loop/recipe maths are identical in all three
(the `pow(count/6000, 4-3α)` f64 pow, the inner-`(int)` budget, the `1+50α`
float loop, the BREAK-not-skip budget gate, the player-fall pull, the two
edge `Next(5)` pushes, the off-screen y spread, the double `+0.2α` scale,
the Deerclops fades). The differences are the four rows above.

**KEEP recommendation: d130's `snow.rs`.** It is the only arm that is wired AND
carries the real `Main.snowDust` counter AND the flake AI AND the draw; g11's
census-per-attempt is an O(6000) scan inside the attempt loop standing in for a
counter the port now has, and d54's body is latent with a tile-gate defect. At
merge, keep-one: `snow.rs` survives; `main_dust.rs::snowing` + `SnowingEnv` and
`world_dust_main.rs::snowing` + its `SnowingEnv` retire (g11's `TeleportEffect`
and d54's other 9 styles are OUTSIDE this reconcile — they stay with their lanes).
The one d130 deviation to carry into ANALYSIS (already recorded there): the
per-consumer splitmix stream instead of the bare global — the WEATHER-2
convention, same as the rain rig; if the merger prefers g11's `NamedRngs::global`
slot instead, that is a stream-convention decision, not a maths one, and either
way the d54/g11 bodies must not survive beside `snow.rs`.

## 3. The splash union — g67's real splash supersedes d130's stand-in — EXECUTED

d130's `spawn_splash` is an explicit stand-in (its doc: "the verbatim pool spawn
+ the splash dust's AI/draw are g67's scope"). g67 (`40128e5d`) landed the real
lane: `Dust.NewDust(position - velocity, 2, 2, Dust.dustWater())` through the d50
`DustPool` contract on Rain's own stream, the eight recipe writes
(`Rain.cs:123-130`: X−2, Y+2, alpha 38, vel×0.1, vel += −velocity×0.025,
vel.Y −2, scale 0.6, noGravity), the refused-sentinel post-writes, the
`Dust.cs:1834-1874` water-splash physics arm on the shared tick, and the
`Main.DrawDust` generic draw (GetAlpha tint, (4,4)-origin rotated quad, GetColor
overlay, Black kill). **The union at merge:**

* **d130 keeps:** the verbatim fall/draw core — `MakeRain`'s float attempt loop +
  AnyDanger throttle, `NewRain`'s first-inactive-slot budget (gfxQuality/snow²/
  atmo⁹), the LIVE `windSpeedCurrent*18` fall velocity, the fixed 750-slot pool
  in slot order, the remix kill-only arm, the raw `+100f` bottom margin, the
  XNA byte tint chain, the `Main.snowing()` lane (§2).
* **g67 supersedes:** the `Splash` vec + `SPLASH_LIFE` + `spawn_splash` stand-in
  and its draw pass → the real DustPool spawn (`water_splash_arm` +
  `draw_splashes` + the `update_rain` pool/spawn_ctx hand-off in
  `world_sim/weather.rs` and `app/frame/draw.rs`). g67's gate reading is already
  the truth (§1) — after this reconcile fix the two branches' `update_drops`
  gates are semantically identical, so the conflict reduces to: d130's pool
  shape + remix arm + inputs struct vs g67's spawn body + physics/draw lane.
  Transplant g67's `spawn_splash(pool, ctx, …)` body and its `water_splash_arm`/
  `draw_splashes` into d130's core; do NOT take g67's `update_drops` wholesale
  (it lacks the remix arm and uses the compacted Vec).
* **g67's witnesses port over** (recipe replay, gate census, below-screen cull,
  wet kill, the 1834 chain, shared-loop tick, same-seed byte-equality) — they
  were derived against the same gate this branch now has. Its `tall_ground`
  fixture convention is already adopted by §1's witnesses.
* d130's stand-in-only witnesses that assert `splashes` contents retire with the
  vec; the gate/remix witnesses (§1) survive unchanged (they pin the roll, not
  the puff).

### Execution record (union-exec, `88cfbf38` + the docs commit on this branch)

Everything above landed as written, with these adaptations recorded:

* `spawn_splash(pool, ctx, world, camera, pos, vel)` — g67's body verbatim
  (`new_dust` on `SplitmixRand(&mut self.rng)`, `dust_water(water_style)`, the
  eight writes, the sentinel post-writes). Called from d130's `update_drops`
  behind the (already-fixed) gate; `i.camera` feeds the water-style read.
* `water_splash_arm` + `is_splash_type` + `draw_splashes` — g67's bodies
  verbatim. Hook dispatch: the `minecart::tick` else fall-through AFTER the
  d130 type-76 arm (g67's branch had no 76 arm; the union keeps both — 76 is an
  else-if of the first ladder, the splash arm is a standalone statement, the
  C# order holds). Draw call site: after `minecart::draw`, before `snow::draw`.
* The drain seam rides d130's `update(world, &RainInputs, dt, pool)` — NOT
  g67's positional-args `update`; `tick`/`update_drops` thread
  `Option<(&mut DustPool, &SpawnCtx)>` the same way g67 did.
* g67's 7 witnesses adapted: `one_falling_drop` → d130's `pin_drop` (fixed
  slot 0..4), `rain.gfx_quality` field → `RainInputs.gfx_quality`,
  `drops.len()` → `drop_count()`, `full_quality()` helper dropped (the
  `inputs()` helper already pins quality 1), `impact_ctx()` → `splash_ctx()`
  directly. d130's `the_impact_gate_rolls_for_all_three_kills` (b) arm and
  `the_remix_kill_arm_never_rolls` now hand a real pool and assert
  `splash_count`; the (c)/(d) quality-0 arms keep `None` (no spawn possible)
  and a tail census pins that only the quality-1 kill left dust.
* `forced_rain_spawns_drops_that_die_on_the_ground` runs with a pool now —
  the below-screen kills' dust survives the spawn cull (591 < 900 margin
  box), so the steady-state assertion holds against the REAL lane.
* g67 tip census: `006bc04d` = `40128e5d` + docs only (anchors/ANALYSIS/
  PORT_MAP). No unique code residuals to fold. The doc-unique content is
  folded: ANALYSIS "The rain collision splash (g67, transplanted W4-d130)"
  (the full lane + the QueueRipple finding + the census + the deviations,
  re-pinned to the union's shapes), the PORT_MAP splash row, and the
  anchors.txt row (the marker `fn spawn_splash` survives the transplant).

## 4. Final-phase g11 snowing fold (arm-by-arm vs Main.cs:13032-13116)

Re-read g11 `13a1c48e`'s `main_dust.rs::snowing` + its `scene_scan` wiring
against this branch's `snow.rs`, arm by arm against the C#. The maths rows of
§2's table hold (identical intensity/budget/attempt/recipe bodies). The g11
details MISSING from d130, now folded:

* **The in-body `gamePaused` disjunct** (`:13040`): g11's `SnowingEnv` carries
  `game_paused` and checks it inside `snowing`; d130 folded it into the
  caller's pause gate only. `SnowInputs.game_paused` lands, the check is the
  C#'s own first disjunct, `update_scene` feeds `app.game_paused`; the gate
  witness gains the pause arm.
* **The HiDef `maxScreenW` pin** (`Main.cs:3709` — 4096): g11 budgets
  `500*view/4096`; d130 pinned Reach 1920. The port runs HiDef
  (`app/frame/draw.rs`), so g11 is right — `MAX_SCREEN_W` 4096, the budget
  witnesses re-derived (1600-px dry num2 = 195, budget 214.5; q0.5 → 165.75;
  Deerclops → 65/71.5). `sand.rs`'s 1920 emit-budget pin is a DIFFERENT
  static's deviation and stays (ANALYSIS re-pinned).
* **The Deerclops census wording**: `IsADeerclopsNearScreen`
  (`NPC.cs:44534-44537`) reads `npcsFoundForCheckActive[668]`, which
  `UpdateFoundActiveNPCs` (`NPC.cs:7229-7239`) sets for any ACTIVE type-668
  npc — no near-screen filter despite the name. Both ports' `hp > 0` scans
  are observation-equal; the citation now names the table walk (g11's
  wording).

KEPT from d130 (g11 lacks or stands in): the real `DustPool.snow_dust` counter
(g11 re-scans `live_type(pool,76)` per attempt — O(6000) inside the loop), the
type-76 AI + tails + draw (g11 defers them to the dust lanes), the stored
`SceneMetrics` parameter shape, the per-consumer splitmix stream (g11 rides
`NamedRngs::global()` — the WEATHER-2 convention decision of §2 stands). The
raw-cloudAlpha read (`rain.cloud_alpha()`, unclamped) was already d130's —
g11's `intensity_value()` is the same raw field.

## 5. Final-phase sandstorm interplay (c40 co-lane census + cross-note)

The gate census: **the interplay is ONE-WAY — the sandstorm suppresses the
rain ZONE, rain never gates the sandstorm.** `SceneMetrics.cs:698-703`:
`ZoneRain = Main.raining && SurfaceAtmospherics`; `ZoneSandstorm = ZoneDesert
&& SurfaceAtmospherics && Sandstorm.Happening`; then `if (ZoneSandstorm)
ZoneRain = false`. `Sandstorm.UpdateTime` (`Sandstorm.cs:32-77`, c40's port in
`tr-core/scene.rs::Sandstorm::update_time`) reads ONLY wind — no rain term
anywhere in the storm's start/stop/severity machine. The task brief's "rain
blocks sandstorm gate" phrasing resolves to this zone suppression, in the
storm→rain direction.

Already landed in this branch (`scene.rs::calculate_zones`, the
`zone_sandstorm_suppresses_zone_rain` witness); the final phase re-pins the
citations (the earlier `:701/:703/:704-707` numbers were the pre-fold
numbering — the 1.4.5.8 lines are `:698-703`) and adds the one-way arm to the
witness (a happening storm with `raining` false still sets ZoneSandstorm).

The downstream chain, verified end-to-end:
* `Rain.MakeRain`'s `!Main.SceneMetrics.ZoneRain` early-out (`Rain.cs:32`,
  `rain.rs::spawn`) — no rain DROP spawns inside the storm zone; a new
  witness pins the suppressed-zone arm (no drops, the event flag + intensity
  ramp survive, the spawns resume when the zone returns).
* The rain ambience cue's `ZoneRain && !ZoneSnow` gate (`Main.cs:11554`, §the
  ambience commit) — track 28 fades out under the storm (the 200-frame
  unconditional stop-fade).
* The blizzard pair needs ZoneRain (`SceneState.cs:78`) — a desert storm
  never co-zones with snow anyway.
* `Main.raining` itself stays up: the event drains `rainTime` through the
  storm (`Main.cs:64874`) and `cloudAlpha` keeps ramping — only the zone
  consumers pause.

c40 co-lane boundary: c40's `update_time` gains the `hard_mode` third
argument (the `21600 * (hardMode ? 2 : 3)` start denominator) — THIS branch
keeps the two-arg pinned-`*3` shape; the merger takes c40's sim half
wholesale (its five gaps), the zone gate here is already identical in both.

## 6. Final-phase f39 slime-rain interplay fold (the rain→slime-rain chain)

f39 (`task/W4-f39-weather-slimerain`, tip `f1a16877`) re-ported the
`UpdateTime` slots around the rain section. Its UpdateTime-side fixes,
verified against `Main.cs:64833-64951` and folded into THIS branch's tick
(the f39 branch keeps its own sky-fall/kill-counter lanes — outside this
fold):

* **The `:64846` surface gate wraps the slime clock + roll.**
  `isThereAWorldSurface` = `worldSurface > 50.0` (`Main.cs:2788`),
  EXCLUSIVE. The pre-fold port decayed the slime clock in a surface-less
  world; the gate now wraps the clock and the morning roll
  (`Rain::update_time` carries its own copy).
* **The clock decays BEFORE the rain section** (`:64848-64863` ahead of
  `:64864`) — the expiry's `StopSlimeRain` draw precedes the rain
  section's draws on `UpdateTime`. The pre-fold port ran the clock after
  the sandstorm walk.
* **The ladder samples its arm ONCE.** `was_raining`/`slime_rain`/
  `lanterns_up`/`next_night_lanterns` sampled at the branch entry — a
  rain that expires INSIDE `update_time` does not open the start arms or
  the slime roll until the next tick; the lantern statics are read before
  `LanternNight.UpdateTime` (`:64965`) can move them. The roll's live
  `!Main.raining` term still closes it when a start arm fired.
* **The lantern arms.** `:64868-64871`'s `LanternsUp → StopRain()` (no
  drain, no strength roll) inside the raining branch; `:64890`'s
  `!slimeRain && !LanternsUp && !NextNightIsLanternNight` gate on the
  start arms — the pre-fold `update_time` folded all three to false. The
  signature gains the three sampled flags (f39's shape).
* **`StartSlimeRain`'s `:64744` gate**: the REMIX seed refuses outright
  (`World.seeds`), and the surface check moves from `<= 0.0` to the
  exclusive `<= 50.0`.

The mutual-exclusion chain, verified end-to-end: a live `Main.raining`
closes the slime morning roll (`:64925`'s first term + the `:64890` arm
never runs while raining); a live `slimeRain` closes the rain start arms
(`:64890`'s first term). `slimeRainTime != 0` additionally closes the roll
through `BusyWithAnyInvasionOfSorts` (`NPC.cs:7024`). The roll's position
after the event walks is the port's recorded within-stream deviation (the
entry samples keep the GATE values pre-update either way; the C# runs the
roll before the walks — the `UpdateTime` draws of the cultist/party/
lantern walks are RNG-free in the port, so the stream order is unchanged).

NOT folded (f39's own lanes, no overlap with this branch): the sky-fall
`SlimeRainSpawns` re-port (`NPC.cs:5892-5955`), the per-death kill-counter
gate (`NPC.cs:79082`), the menu-return `StopSlimeRain` (`Main.cs:48216`).
The merger keeps f39's bodies for those; this branch's tick now matches
f39's UpdateTime shape so the two merge without a semantic conflict.

Witnesses (WRITTEN NOT RUN): the lanterns-up stop arm (draw-free), the
`:64890` three-flag gate (fire-seeded control + each flag draw-free), the
remix/exclusive-surface `StartSlimeRain` gate, the surface-gated clock
(frozen surface-less, decays with a surface), the expiring-rain roll
closure (the fire-seeded UpdateTime stream stays put this tick, fires the
next), the mutual-exclusion pair (live rain blocks the slime roll; live
slime rain blocks the fire-seeded start arms).

## 7. Snow-stream conflict adjudication + the g67b-post-union fold (EXECUTED)

The Lead's adjudication round: g11's research/83-addendum (`d892d8d4` on
`task/W4-g11-dust-main-sites-1`) F1 claims `snowing()` rides the GLOBAL
bare-`Main.rand`; this branch's §4 fold had kept the PRIVATE splitmix.
**Verdict: g11 is RIGHT — the private stream is retired (F1 applied).**

### The call-chain proof (Main.cs re-read in full, :16774-17292)

* `Main.Update` (`:16774`) calls `Main.snowing()` at **`:17238`** — inside
  the Update body itself, after `UpdateWeather` (`:17228-17231`) and
  `Main.Ambience()` (`:17233`, waterfall/lava only — no swap), before
  `Sandstorm.EmitDust` (`:17247`).
* The world simulation runs LATER: `:17278-17281` gates
  `this.DoUpdateInWorld()` (`:17726`), whose FIRST statement is
  `using (Main.SwapRandom("DoUpdateInWorld"))` (`:17732`). Every other
  swap scope in the game — the 13 nested wrappers of
  `DoUpdateInWorld_Inner` (`:17749-17809`), `SpawnNPC` (`:17887`),
  `UpdateLiquid` (`WorldGen.cs:71529`) — is nested INSIDE that block.
  There is **no `SwapRandom("UpdateWorld")` key** (the 17-key census,
  `app/rng.rs`, transcribed from those lines).
* `RandomSwap.Swap`/`Dispose` (`Main.cs:103-120`) restore the previous
  instance on scope exit, so at `:17238` NO swap is live: the instance is
  the bare `Main.rand` seeded once at `Initialize` (`:6463`,
  `DateTime.Now.Ticks` — the port's recorded world-seed deviation,
  `NamedRngs::global()`).
* Therefore snowing's draws interleave with the global stream's other
  pre-swap consumers (the ambience scheduler `:17214-17216`, the
  chest-eat cues, chat/menu rolls). The WEATHER-2 per-consumer convention
  covers the rain rig's OWN consumers, not this site.

### F1/F5/F7 applied (`099df89e`)

* **F1**: `Snow` struct + its splitmix (`new`/`next_u64`/`rand01`/
  `next_int`) + the `App.snow` field (both init sites) RETIRED.
  `snow::snowing` is a free fn taking `rng: &mut UnifiedRandom`;
  `scene_scan::update_scene` passes `app.rngs.global()`; the pool's
  `NewDust` rides `DustRng for UnifiedRandom` (no `SplitmixRand` adapter).
* **F5**: the int attempt-loop shape (`(attempt as f32) < num3`, the C#
  `for (int i…)`), replacing the float accumulator.
* **F7**: the 8 d130 snow witnesses re-pinned to `UnifiedRandom` (the
  gates + budget breaks now also pin ZERO stream draws); g11's 3
  draw-sequence mirrors ported (`w4_g11_snowing_mirrors_the_dry_arm_draw_sequence`,
  `…_the_storm_arm_with_deerclops` — re-derived on the FROZEN F4 counter,
  `w4_g11_snowing_breaks_on_the_counter_budget` — the 4096-pin cap 31.2).
* F2 (4096 HiDef), F3 (in-body `gamePaused`), F4 (counter once per call),
  F6 (env sourcing) were already folded in §4 — verified unchanged.

### The g67b-post-union fold (`16a792d5`)

The d130d union-exec report (`ec1d73c7`) censused g67 at `006bc04d`
(docs-only tip claim) — but the g67 branch has FOUR MORE code commits
(`d2dd8989` kill-arms, `5c9398ea` MakeRain/NewRain census, `2c452eddb`
cull-kill, `112d1b20` docs+census). **research/134 does NOT exist** on
`origin/task/W4-g67-weather-rain-splash` (tip `112d1b203`, no 134 file in
`docs/research/` — re-fetched and verified); the fold-list is the commits
themselves. Per-item verdicts:

| g67b unique | d130 state at `560a02ea` | action |
| --- | --- | --- |
| menu +2000 kill-only arm (`d2dd8989`, `Rain.cs:103-109`) | MISSING ("gameMenu never holds in-world" fold) | FOLDED — `RainInputs.game_menu`, move-first/no-roll/no-probe arm |
| remix +100 kill-only arm | present (d130 core) | verified, no change |
| MakeRain AnyDanger 0.05 throttle + floor (`5c9398ea`) | present | verified, no change |
| MakeRain rising-player window / wind+player drift / clamps / clamped probe | present | verified, no change |
| MakeRain `:84` menu spawn-through-solids disjunct | MISSING | FOLDED |
| NewRain gfx-cut `(1+q)/2 < 0.9` | present (real adaptive quality) | verified, no change |
| NewRain live `GetRainFallVelocity` / scale / frame rolls | present | verified, no change |
| `NewRainForced` Eyebrella seam (`Rain.cs:135-164`) | MISSING | FOLDED — fixed-slot scan, `maxRain` sentinel |
| skyblock `SnowTileThreshold` gap note (`SceneMetrics.cs:154-163`) | MISSING | FOLDED (note; the port tracks no skyblock field) |
| IsItRaining fix (`112d1b20`, `Main.cs:2597`) | MISSING — 5 feeds read the event flag | FOLDED — `Rain::is_it_raining()` feeds the ambience scheduler (frame + netmodules ctx), the music rain branches, the town quiet-sky gate, the storm-lightning relaxation; the fountain scanner (`TileLightScanner.cs:3030`) + `Lang.cs:141` KEEP `Main.raining` (verified) |
| DrawDust cull-rect + else-kill (`2c452eddb`, `:38832`/`:38847`/`:39020-39022`) | MISSING ("documented widening") | FOLDED — `splash_cull_rect`/`dust_in_cull_rect` in `draw_splashes` AND `snow::draw` (the same loop covers 76) |
| `ClearRain` (`5c9398ea`, `WorldGen.cs:6672`) | MISSING (cited visual-only, never called) | FOLDED — `Rain::clear_rain` wired into `save_and_quit` |
| `:64890` start gate | present (§6 f39 fold) | verified, no change |

Witnesses (WRITTEN NOT RUN): 4 g67b arms adapted to the fixed-slot pool +
`RainInputs` shape (menu-arm census, forced-budget 375 cap,
clear+IsItRaining semantics, cull-rect geometry) + the 3 g11 snow mirrors
above. The g67 `spawn_acc` fractional-rate cadence is NOT taken — d130's
per-tick float attempt loop is the C# `for (i < num)` shape (ceil
attempts), the closer port.

## Verification

Code-only mode (per the task brief): no cargo/tests/graphify run. The fix was
verified by re-reading `Rain.cs:96-133`, `Main.cs:13032-13116`, `Tile.cs:240-247`
and diffing all three snowing arms + both update_drops bodies line-by-line
against them. The new witnesses are WRITTEN NOT RUN, hand-derived from the C#
with the port's `StreamReplay` splitmix replay. ANALYSIS "Rain and weather" +
PORT_MAP's Rain.cs row re-pinned to the corrected gate; this doc + INDEX row
record the reconcile.

Commits: the fix + this doc land on `task/W4-d130-weather-ambience` on top of
`393000e6` (hashes in the branch log / the report back to the Lead).

## 8. FINAL-FINAL — the retire phase (ALL FOLDS EXECUTED, 2026-11-21; SEALED by §9)

**STATUS: RETIRED.** Every fold-list item from g67 (research/134 R1–R7),
g67d (the g67 final-phase render/blizzard censuses) and the f39 co-lane is
either EXECUTED on this branch or verified-present. This section is the
per-item receipt; the d130-canonical declaration is restated at the end.

### 8.1 The g67 tip moved — research/134 EXISTS now

§7's "research/134 does NOT exist on the g67 branch" was true at the g67b
fold (`16a792d5`, g67 tip `112d1b203`). g67 has since pushed (re-fetched
2026-11-21, origin tip **`e777b3e2c`**):

| commit | content |
| --- | --- |
| `f8c4b659c` | skyblock low-tiles: `World.skyblock_low_tiles` + the snow-threshold arm (R5) |
| `356a1ac0e` | Eyebrella: the `UpdateEquips` drip caller + `eyebrellaCloud` (R6) |
| `01ba6c7a9` | umbrella held-item arm: the 2 px/tick slow-fall + overhead placement (R7) |
| `30b39fceb` | research/134 published — the post-union fold-list for d130d/e |
| `d0ebe7f07` | final phase 1: fold-monitor re-check (d130 tip was still `560a02ea`, R1–R7 all OPEN then) |
| `a7f927464` | final phase 2: Rain.cs FINAL fn-census — 6/6 methods + 7/7 fields; the `alpha` field is dead in the C# (no port gap) |
| `7aaf66f00` | final phase 3: DrawRain render census — `rain_light_tile` + `rain_drop_tint` (the XNA byte chain + shimmer) |
| `0dfb19229` | final phase 4: blizzard/sandstorm visual-overlap + wind-slant census (docs-only) |
| `e777b3e2c` | final phase 5: research/134 FINAL + the KEEP-d130-CANONICAL refresh |

### 8.2 Per-item receipt vs the 134 fold-list (R1–R7)

| item | 134 says | d130 state | receipt |
| --- | --- | --- | --- |
| R1 `IsItRaining` consumers (5 feeds) | fold `is_it_raining()` + the ambience/music/quiet-sky/lightning feeds | **ALREADY FOLDED** by `16a792d5` (the g67b fold pre-empted the 134 list — same semantics, derived from `5c9398ea`/`112d1b20` directly) | verified at the retire tip: `rain.rs:316` `is_it_raining`, `frame/world.rs:192/:259/:503`, `netmodules.rs:486`, `weather.rs:116` — all read the cloud, not the event flag. The quiet-sky feed reads `!is_it_raining()` (≡ `cloudAlpha == 0`, the 134 `visual_alpha() == 0` form is observation-equal for the non-negative ramp) |
| R2 `ClearRain` on save-and-quit | deactivate all 750 slots at `WorldGen.cs:6672` | **ALREADY FOLDED** by `16a792d5` | `rain.rs:325` `clear_rain` (fixed-slot deactivation), `saveglue.rs:366` — before `save_world` (the C# order: ClearRain `:6672` precedes SaveWorld `:6675`; the event triple is untouched so the save still snapshots the live event) |
| R3 DrawDust cull rect + else-kill | the ±1000/±1050 wide box, strict-< 4×4, else-DEACTIVATE | **ALREADY FOLDED** by `16a792d5` | `rain.rs` `splash_cull_rect`/`dust_in_cull_rect` + the kill in `draw_splashes` AND `snow.rs::draw` (the same loop covers 76); the `w4_g67_drawdust_cull_rect_is_the_wide_box_with_an_else_kill` witness pins the edges |
| R4 `NewRainForced` seam | fixed-slot scan, gfx-cut budget, `maxRain` sentinel | **ALREADY FOLDED** by `16a792d5` (as `#[allow(dead_code)]` — the caller was R6) | `rain.rs` `new_rain_forced(&RainInputs, pos, vel)`; the retire phase lifted the allow (R6's drain is the production writer); the 375-cap witness present |
| R5 skyblock snow-threshold | `World.skyblock_low_tiles` + scan + `snow_tile_threshold` + the NewRain arm | **FOLDED THIS PHASE** — `10ee539da` | `world.rs` field + `scan_skyblock_low_tiles` (40-margin census, f32 fraction < 0.1 AND SKYBLOCK), load (`app/mod.rs`) + both gen entries; `scene.rs` 300-constants + getters + `_at` variants + the zone arms; `scene_scan.rs` copies the flag; **d130 adaptation**: `new_rain` reads `RainInputs.skyblock_low_tiles` (the gathered-inputs seam) instead of g67's direct `world` read. 3 witnesses adapted |
| R6 Eyebrella caller + `eyebrellaCloud` | `do_eyebrella_rain_effect` + the UpdateEquips arm + the drain (needs R4) | **FOLDED THIS PHASE** — `df28ea1ac` | `player.rs` field + pending queue + the verbatim `:14308-14331` port + the `:13083-13087` arm + the `:18461` reset; `update_with_grapple` gains `allow_rain` (12 test sites re-pinned true, production derives `window_focused && !game_paused && !map_open`); `world_sim/player.rs` drains into `new_rain_forced` with live `wind*3, 9` through a gathered `RainInputs`. 3 witnesses adapted |
| R7 umbrella held-item arm | `umbrella_hold_slow_fall` + the render overhead override | **FOLDED THIS PHASE** — `df28ea1ac` | `player.rs::umbrella_hold_slow_fall` at the ItemCheck position (after the border clamps), `render/player_layers.rs` style-2 override (946/4707 → overhead, rotation 0; the generic quarter-turn kept). 2 witnesses adapted |

The 134 "Deliberate divergences" all stand: the float attempt-loop cadence
(d130's, NOT g67's `spawn_acc`), the fixed-750-slot storage, and — updated —
the `gameMenu` kill-only arm is NOW PORTED on d130 (`16a792d5` folded it IN,
superseding the 134's "d130 folds it out" note; the residual-drop menu tick
is carried for shape behind `RainInputs.game_menu`).

### 8.3 g67d (the g67 final phase) — fold-or-compose receipts

* **`a7f92746` fn-census (docs-only)**: 6/6 methods + 7/7 fields — d130's
  rain.rs covers MakeRain/NewRain/NewRainForced/ClearRain/Update/
  GetRainFallVelocity; the dead `alpha` field needs no port. Compose-note
  only, no code.
* **`7aaf66f0` DrawRain render census**: the two residuals it closed on g67
  (the `(pos+4)>>4` light tile, the XNA byte chain + shimmer) were ALREADY
  canonical on d130 since `ea4863d9` (the inline `light_tile` closure +
  `rain_tint`). **COMPOSED**: `rain_light_tile` extracted as a named helper
  (the draw closure and the new witness share it) + the
  `w4_g67d_draw_rain_light_tile_is_pos_plus_four_shifted` witness; the tint
  keeps d130's `rain_tint` name (the 134-FINAL keep-one instruction — g67's
  `rain_drop_tint` copy drops). The draw caller already passes
  `scene_state.shimmer_alpha`. DrawRain has NO cull rect / NO black-kill —
  d130's drop draw has neither (verified). The per-drop in-draw `Update`
  cadence fold is the recorded 60 Hz deviation (unchanged).
* **`0dfb1922` blizzard/sandstorm/wind-slant census (docs-only)**:
  CONFIRMS d130 canonical — vanilla MakeRain has NO biome gate (the snow
  suppression shrinks the budget, `Main.snowing()` runs its own gates and
  branches on `Main.raining` `:13092`); d130's `crate::snow` carries that
  branch verbatim, g67's `snow_mode` replaces-drops gate stays retired
  (§2). The one-way ZoneSandstorm→ZoneRain gate matches §5. The wind-slant
  IS `atan2(vx,-vy)` over the live wind at spawn — both ports faithful, no
  draw-angle term. Compose-note only, no code.
* **`e777b3e2` 134-FINAL + KEEP-d130-CANONICAL**: the instruction restated
  below (§8.5).

### 8.4 f39 co-lane compose (origin tip `bd5a5a24a`)

Executed in `22371b4fe` — the full receipt in ANALYSIS "Rain and weather"
(retire-phase co-lane compose paragraph): the music track-48 arm
(`Main.cs:12654-12657`), the anniversary `StartRain` range 5
(`:64663-64667`), the `ResetSpawnTime` `/2` (`AmbienceServer.cs:133-136`),
the menu-return `StopSlimeRain` (`:48216-48219`) and the `slime_rain_roll`
lowTiles `/5` arm (`:64931-64934`, activated by the R5 census). The
UpdateTime tick-shapes compose-verified EQUAL (the sampled-flag ladder,
the surface gate, the clock-before-rain order, the StartSlimeRain gates —
d130's `cf4574e9` fold and f39's `c4d9937a5` are the same port). f39's
SlimeSky spawn bodies (`npcs_w4_f39*.rs`), kill-counter lane and season/date
module (`tr-core/season.rs`, Phase B) do not overlap d130's files — the
merger keeps f39's bodies for those; the four gates above are the only
cross-boundary hunks and they are now on BOTH branches with identical
semantics.

### 8.5 The d130-canonical declaration (FINAL)

**The merger keeps `task/W4-d130-weather-ambience` wholesale for the rain
core, the snow lane and the weather tick.** g67's `rain.rs` drops wholesale
(134-FINAL's own instruction, `e777b3e2`): its compacted-Vec storage, its
`spawn_acc` cadence, its `snow_mode` gate, its `rain_light_tile`/
`rain_drop_tint` copies (d130 carries the same semantics under
`rain_light_tile`/`rain_tint`), its `DropSimInputs` seam (d130's
`RainInputs` superset covers it). Every semantic residual R1–R7 is now ON
this branch (R1–R4 via `16a792d5`, R5 via `10ee539da`, R6/R7 via
`df28ea1ac`); g67d's censuses are compose-notes with zero open code gaps;
the f39 boundary hunks are composed (`22371b4fe`). The snow-stream
adjudication (§7, g11 F1) stands: `crate::snow` on the GLOBAL bare stream.
**No weather-lane fold remains open.** (The seal §9.2 adds the two c39
graft hunks — the `:64825-64831` head clamp + the full `LanternsCanPersist`
gate — the only ledger items the co-lanes still owed at this declaration;
both are now folded, and §9.1 records g67's matching drop-wholesale FINAL
under the 146 rename.)

Retire-phase commits (this branch, on top of `97ed44f5d`):
`10ee539da` (R5) → `df28ea1ac` (R6+R7) → `22371b4fe` (f39 compose) →
`af3ebc1a` (the retire docs). All witnesses WRITTEN NOT RUN (code-only mode — no
cargo/tests/rustfmt/graphify invoked, per the task directive).

---

## 9. SEAL — the weather-family close (2026-11-21, on top of `af3ebc1a`)

The seal re-reads the three co-lane tips (`git fetch`, full hashes):
**g67 `8b9a1ff46c4a4e795c51ea96f9c13568665ca16b`** (seal phase 4 — the
146-rename + drop-wholesale FINAL + the family bible), **c39
`7dd785dc57ebf793f197378451e277a636c162f5`** (retire FINAL-FINAL, research/114
§16 — code behaviour exactly `4f72847e`'s), **f39
`bd5a5a24adfef95fafb76f7705b5d7e3dd333b8f`** (Phase B). Four seal items:

### 9.1 The g67 mutual receipt — BOTH sides agree, ONE fn owner

g67's seal renamed research/134 → **research/146** (`c38d42d32` — the
registry sweep: 134 collides f11's light-family file, 145 double-claimed,
146 = max-on-origin + 1 provisional; renumber at merge if the registry
assigns differently). The g67 seal's d130f-monitor was written against tip
`97ed44f5d` (BEFORE this branch's retire phase) — its OPEN rows are now
all closed, receipts per item:

| g67 146 row (seal-state) | d130 receipt |
| --- | --- |
| R5 skyblock — OPEN → d130f | **FOLDED `10ee539da`** (§8.2): `rain.rs` reads `RainInputs.skyblock_low_tiles` → `scene::snow_tile_threshold` (300/1500), the unconditional `SNOW_TILE_NORMAL` retired |
| R6 Eyebrella caller — OPEN → d130f | **FOLDED `df28ea1ac`** (§8.2): `do_eyebrella_rain_effect` + the 5101 arm + the `new_rain_forced` drain, the `dead_code` allow lifted |
| R7 umbrella — OPEN → d130f | **FOLDED `df28ea1ac`** (§8.2): `umbrella_hold_slow_fall` + the render overhead override |
| A1 tint alpha — NO-OP keep d130's | **KEPT verbatim**: d130's `rain_tint` dims alpha (XNA `Color * float` scales A); g67's alpha-255 witness assertions drop with the wholesale copy |
| A2 light-tile witness — OPEN → d130f | **FOLDED `af3ebc1a`** (§8.3): `rain_light_tile` extracted (`rain.rs:1309`), `w4_g67d_draw_rain_light_tile_is_pos_plus_four_shifted` (`rain.rs:3428` region) — the 12..15-band crossing, the 63.9 edge, the negative-truncation arms |

**The drop-wholesale agreement is MUTUAL**: g67's 146 FINAL instruction
(`8b9a1ff46`/`c38d42d32`) says DROP g67's `rain.rs` + `draw.rs` census
wiring wholesale, ONLY the four-commit hash-list crosses (R5
`f8c4b659c30dcd6378c8ca5f4a1caff9e84639c4`, R6
`356a1ac0ef42848a8e5ee0a58abd6c197cc5b38c`, R7
`01ba6c7a9494fb7f348cc4fc2cb393e1e0c005c8`, A2-witness-only
`7aaf66f00e00e5e2ca2ffd3c1349e3888c098181`; code tip
`e777b3e2c2fab2a917fe645df3a4a469b99fad8f`); this branch's §8.5
declaration says the SAME from the keep side. The R5/R6/R7 player/world/
scene-lane hunks are the folds themselves, NOT part of the rain.rs drop —
they are ON this branch already. **ONE fn owner per body**: d130 owns every
`Rain.cs`/`Main.snowing` fn; g67's copies (`update_drops` compacted-Vec,
`spawn_acc`, `snow_mode`, `rain_drop_tint`, `DropSimInputs`, the
KNOWN-WRONG tint witness) all drop. The g67f-4-fold-gifts are folded —
receipt closed, no g67 content remains uncrossed.

### 9.2 The c39 2-hunk graft — verified OPEN, then FOLDED (this seal)

g67's seal said BOTH c39 hunks still missing at `97ed44f5d`; this seal
re-verified at `af3ebc1a`: STILL missing — the f39-compose (`22371b4fe`)
covered the `:64868-64871` lanterns-up StopRain arm + the `:64890` gate
(`cf4574e9`), NOT the `:64825-64831` head clamp nor the persistence gate.
The graft spec is **c39 research/114 §13** (`8e0946ff0`, re-read at c39
tip `7dd785dc5` — §16 seals it unchanged). Both hunks folded EXACTLY as
specified:

* **Hunk 1** — `rain.rs::update_time`: the `lanterns_up → cloud_bg = 0.0`
  head clamp inserted between the forced/calm early return and the
  ladybug drain (`Main.cs:64825-64831`; the `numClouds` 30 cap is
  outcome-neutral at the 100 stand-in — recorded). The `lanterns_up`
  param already existed (the f39-order signature) — compiles unchanged.
  c39's witness `w4_c39_the_lantern_head_clamps_the_storm_deck_before_
  the_drain` ports VERBATIM (d130's rig names are identical:
  `rain_world`/`with_forced`/`rng`).
* **Hunk 2** — `world_sim/weather.rs`: `lantern_can_persist` is now
  `!world.day_time && lanterns_can_start(&LanternGates::from_world(world,
  &app.npcs))` — the full `LanternsCanPersist` (`LanternNight.cs:49-56`).
  The one mechanical rider applied: `world_events.rs::lanterns_can_start`
  bumped `fn` → `pub(crate) fn` (c39's module exports it with the
  identical seven-conjunct body — no logic choice). A new tick-level
  witness `w4_c39_the_lantern_persistence_gate_is_the_full_lanterns_can_
  persist` pins the calm-night persist, the blood-moon mid-night clear,
  the manual-lanterns ride-through and the dawn clear.
* d130's KEEP regions untouched (§10c rows 3/4): the slime-clock-first
  restructure, the `:64846` surface gate, the if/else StopRain shape,
  `StartSlimeRain`'s `:64744` fix. The wind rename (`lantern_night` ↔
  `lanterns_up`) stays f40/105 §4's mechanical verdict — d130 keeps
  `lanterns_up`.

### 9.3 The f39 season/SlimeSky keep-one receipt

Re-verified against f39 tip `bd5a5a24a` (merge-base `a725acb04`): d130 has
**zero edits** to `world_sim/npcs.rs` and `render/sky.rs` since the base —
f39's SlimeSky spawn bodies (`npcs_w4_f39.rs`/`npcs_w4_f39_spawns.rs`),
the kill-counter refinement (the per-death `slime_rain`/King recheck,
f39 `npcs.rs:5441` region vs d130's base-inherited `:5202` census-gate
form), the season/date module (`tr-core/season.rs` — ABSENT on d130,
grep-verified), the `enter_loaded_world` season re-derivation and the
`weather_w4_f39.rs`/`saveglue_w4_f39.rs` witness modules do not overlap
any d130-changed body. **The merger keeps f39's bodies for those lanes.**
The six shared files (`ambience.rs`, `app/mod.rs`, `app/saveglue.rs`,
`rain.rs`, `world_sim/weather.rs`, `tr-core/world.rs`) carry the five
boundary gates already composed BOTH-ways (`22371b4fe`, §8.4 — music
track 48, the anniversary range-5 + `/2`, the menu-return StopSlimeRain,
the lowTiles `/5`): d130's `saveglue.rs:373-384` arm and f39's
`saveglue.rs:366+` arm are the SAME hunk (same comment intent, same
`rngs.global()` stream); `ambience.rs::reset_spawn_time`'s `/2` is
byte-identical on both tips. At merge those hunks dedupe; the rain.rs/
weather.rs core bodies are d130's wholesale (f39's copies are the
pre-d130 base shapes, superseded).

### 9.4 The WEATHER-FAMILY bible (FINAL — one paragraph, for the merger)

The weather family unions as ONE composed lane, **d130 canonical for the
rain/snow core**: this branch owns `Rain.cs` verbatim (fall/draw/splash,
the fixed-750-slot pool, `RainInputs`, the two-arm cloudAlpha ramp, the
ambient tracks 28/45 lifecycle, the `update_time` roll ladders INCLUDING
the c39-grafted `:64825-64831` lantern head clamp) and `Main.snowing()` on
the GLOBAL bare-`Main.rand` stream (F1, §7), with g67's four unique folds
grafted (R5/R6/R7/A2, the 146 hash-list) and **g67's rain.rs + draw.rs
census wiring dropped wholesale**; **f39** owns the slime-rain event half
(`start_slime_rain`/`stop_slime_rain`/the slime clock live on d130's
f39-shaped tick — `cf4574e9`/`22371b4fe`, mutual exclusion witnessed both
ways — while the SlimeSky spawn bodies, the kill-counter refinement, the
season/date module and the v18 seasonal downeds stay f39's, §9.3);
**c40** owns `Sandstorm.cs` (the wind-only severity machine + the
flying-sand 268 emit) and the gate is ONE-WAY — `ZoneSandstorm` clears
`ZoneRain` (`SceneMetrics.cs:698-703`), rain NEVER gates the storm
(`Sandstorm.cs:32-77`, §5); **g69** owns the cloud draw/parallax lane
including `rain_dim_sky` (the XNA byte-truncate-FIRST convention) — the
`cloudAlpha`/`cloudBG` VALUES come from d130's rain state (the head clamp
included), g69 keeps its draw bodies; **g68** owns the wind sim + leaf
cadence — its LIVE `windSpeedCurrent` feeds d130's `GetRainFallVelocity`
(`wind*18,14`), the Eyebrella drip (`wind*3,9`) and c40's severity, ONE
wind-flag owner with read-only consumers; **c39** touches the family ONLY
through the two graft hunks — NOW FOLDED (§9.2: the storm-deck head clamp
+ the full `LanternsCanPersist` gate), its lantern/party/event-sky bodies
stay c39's. No two family branches own the same fn body; the only
cross-branch writes were the enumerated grafts and they are all landed.
**Gate command (the merger's verify):** `cargo test --workspace -- --skip
migrates_version_2_kinds_to_item_ids` — judge only by the `test result:
ok` lines (cargo may exit 1 on success in this repo); every witness on
every family lane is WRITTEN NOT RUN (code-only), so the merged tree's
first full run is the gate.

Seal-phase commits (this branch, on top of `af3ebc1a`): the c39-graft
code commit → this docs commit. No cargo/tests/rustfmt/graphify invoked.
# 83 — Weather reconcile: the g11-side addendum (snowing keep-one)

**STATUS: FINAL (g11 final phase, d130 tip `560a02ea` re-read).** d130
executed the keep-one on ITS branch (union-exec `88cfbf38`/`ec1d73c7`,
final-phase `85250712`/`5289d055`/`85b2a09c`/`703370e6`/`cf4574e9`, docs
`560a02ea`): it folded the g11 details F2 (HiDef 4096) + F3 (in-body pause
gate) + the census wording (`85250712`, its §4), kept F4's real counter
and the 76 AI/tails/draw, and **KEPT its own splitmix stream — F1 NOT
APPLIED** (its §4: "the per-consumer splitmix stream … the WEATHER-2
convention decision of §2 stands"; F5's int loop also not taken — its
float loop stays). The §-patch F1 hunks below are therefore
**REFUTED-AS-APPLIED**, not refuted-as-maths: the §-adjudication section
(below the patch) re-derives the `:17238` scope from the C# and confirms
the g11 pre-swap/global claim is the faithful reading; d130's §2
explicitly delegates the slot choice to the merger ("a stream-convention
decision, not a maths one"), so F1 stays the published RECOMMENDATION
the merger may still take. g11's own `main_dust.rs::snowing` +
`SnowingEnv` + consts are **marked DEPRECATED in-code** (the module doc
RETIRE-NOTE + the fn doc) — the merger drops them for d130's `snow.rs`;
this branch keeps the body + its four witnesses compiling until then
(the witnesses migrate per F7). The teleport half is research/116 —
EXECUTED + FINAL on this branch.

W4-g11 reconcile phase, code-only. This is the **g11 branch's addendum** to the
d130 reconcile doc (`docs/research/83-weather-reconcile.md` at `50c845e5` on
`task/W4-d130-weather-ambience` — not present on this branch; the merger unions
the two files). d130 picked **KEEP d130** for the `Main.snowing()` 3-way; this
addendum **agrees with the d130 base** and re-derives the arm-by-arm comparison
from the C# (`Main.cs:13032-13116`, re-read in full), correcting one claim in
d130's table and listing the **exact folds** the union needs. No code changes
in this commit — the folds are the merger's worklist.

Arms compared:

| arm | commit | body |
| --- | --- | --- |
| g11 (this branch) | `73545f0b` | `main_dust.rs::snowing` + `SnowingEnv`, wired at `scene_scan.rs:399-434` |
| d130 | `50c845e5` (body `5ae7f39a`) | `snow.rs::Snow::snowing` + `SnowInputs`, wired at its `scene_scan.rs:399-431` |
| d54 batch-4 | `48343a4c` | `world_dust_main.rs::snowing` + `SnowingEnv`, LATENT |

## 1. Arm-by-arm (every row re-derived against `Main.cs:13032-13116`)

| aspect | C# | g11 `73545f0b` | d130 `50c845e5` | d54 `48343a4c` |
| --- | --- | --- | --- | --- |
| wiring slot | `Main.Update:17238`, before `Sandstorm.EmitDust:17247`, outside every `SwapRandom` | WIRED — `scene_scan::update_scene`, before `emit_dust`, stored metrics (last-scan persistence) | WIRED — same slot, same order, stored metrics clone | LATENT (body + witnesses only) |
| RNG stream | bare `Main.rand` (pre-swap) | `NamedRngs::global()` — the port's bare-`Main.rand` stand-in; the pre-swap chest-eat cues already ride it (`frame/world.rs:580`) | own splitmix `Snow` stream (the WEATHER-2 per-consumer convention, `sand.rs` pattern) | stream parameter (caller picks; doc says `global`) |
| `Main.maxScreenW` (`:13045`) | static: 1920 at init (`Main.cs:66715`), **4096 under HiDef** (`SetGraphicsProfileInternal:3709`); the non-XNA default selects HiDef (`:67047`) | **4096** (`main_dust.rs:56`, cites `:3709`; the port runs HiDef — `frame/draw.rs:65`, `ambience.rs:184-187` "the Reach fallback is not used by modern builds") | **1920** Reach pin (`snow.rs:43`, "HiDef's 4096 arm is a documented drop", the `sand.rs:35`/`rain.rs:889` stand-in family) | `env.max_screen_w` parameter (caller picks) |
| budget `:13057` | `Main.snowDust` static — reset at the `UpdateDust` head (`Dust.cs:428`), `++` per live 76 slot (`:1747`); read once per attempt, constant within the call | **live-census stand-in** — `minecart::live_type(pool,76)` per attempt (O(6000) scan; the `SandStormCount` pattern) | **the real counter** — `DustPool.snow_dust` (`particles.rs:233/:329`), read once per call, fed by the 76 arm | `env.snow_dust` parameter (caller must supply the census) |
| `gamePaused` gate `:13040` | in-body early return BEFORE any draw | in-body (`env.game_paused`, `main_dust.rs:118`) | **folded out** — "folds into the caller's pause gate"; the `scene_scan` caller does NOT gate on pause (the fused per-frame scene update runs paused) | in-body (`env.game_paused`) |
| tile gate `:13086` | `InWorld && tile != null && !nactive() && wall == 0`; `nactive()` = active && !actuated (`Tile.cs:240-247`) | correct — `!(t.active() && !t.inactive()) && t.wall == 0`, OOB → no spawn | correct — `(active && !inactive) \|\| wall != 0 → continue`, OOB → continue | **WRONG** — `!t.active()` misses the actuated arm (d130's finding, confirmed) |
| attempt loop `:13053` | `for (int i = 0; (float)i < num3; i++)` | int counter, `(i as f32) < num3` — the C# shape | float `attempt += 1.0` — observation-equal (exact ≤ 2^24) | float `i += 1.0` — observation-equal |
| `Math.Sign(0)` `:13099` | three-way sign, 0 at 0 | explicit three-way match | explicit three-way match | `signum()` — observation-equal (sqrt(0)=0 factor), not the C# shape |
| intensity/recipe maths `:13044-13108` | f64 `Pow`, inner-`(int)` budget, `1+50*α` float bound, BREAK-not-skip budget gate, player-fall pull, else-if edge `Next(5)` pushes, off-screen y spread, double `+0.2*α` scale, Deerclops `/20` `/3` `-0.5` fades | identical | identical | identical |
| type-76 AI (`Dust.cs:1745-1763`) + tails (`:2071-2074`/`:2079-2082`/`:2119-2128`) | shared `UpdateDust` | none — "the 76 physics arm stays d50's dust_physics lane" (flakes ride the generic move/kills until it lands) | **ported** — `type_76_arm` (counter `++`, `+0.009` growth, the 0.04 lerp pull, the `!noLight` position push, the 10×10 solid damp) + `type_76_tails` (0.99 drag, rotation, `-0.01` scale), hooked in `minecart::tick:1106-1107` | none (d50 lane deferred) |
| draw (`Main.DrawDust` generic 76, `Main.cs:38968-39018`) | light at `(pos+4)/16`, (4,4)-origin rotated quad, full-dark kill | none (draw lane deferred) | **ported** — `snow::draw`, hooked at `frame/draw.rs:1292` | none (draw lane deferred) |
| `Snowflake` rain-rig stand-in | n/a | retired (keep-one, `73545f0b`) | retired (no `Snowflake` in its `rain.rs`) | untouched (latent note) |

**Correction to d130's §2 table:** its closing claim "the intensity/budget/
attempt-loop/recipe maths are identical in all three" is **wrong on the
budget** — the `maxScreenW` pins differ (4096 vs 1920), so `num2` differs by
~2.13× (at a 1600-px view, dry: `500*1600/4096 = 195` vs `500*1600/1920 =
416`), and the Deerclops `/3` arm compounds it. Every downstream budget
assertion (d130's snow witnesses, g11's
`w4_g11_snowing_breaks_on_the_live_76_budget` cap comment `25e180d4`) is
pin-dependent. The port runs HiDef (`frame/draw.rs:65`, `ambience.rs:184-187`
— both cite `Main.cs:67047`/`:3709`), so **4096 is the faithful pin**; d130's
1920 is a stand-in inherited from `sand.rs` (whose own 1920 pin is the same
deviation family — flagged for the sand/rain lanes, NOT changed here).

## 2. KEEP-ONE recommendation: d130 base + the g11 details folded

**Keep d130's `snow.rs` as the surviving module** — it is the only arm that is
wired AND carries the real `Main.snowDust` counter AND the flake AI AND the
draw (d130 §2's reasoning stands). **Fold the g11 body details into it:**

* **F1 — the body + stream.** Replace `Snow::snowing`'s body with g11's
  `main_dust.rs::snowing` body (they are line-for-line the same maths; g11's
  carries the corrections below). Re-sign to a free function taking
  `rng: &mut UnifiedRandom` (the caller passes `app.rngs.global()`) +
  `snow_dust: i32` (d130's counter param) + d130's `SnowInputs`/metrics/ctx/
  world/pool. Retire the `Snow` struct's private splitmix (`new`/`next_u64`/
  `rand01`/`next_int`) and the `App.snow` field (`app/mod.rs:890`,
  `flow.rs:2427`, `app/mod.rs:3550` on d130). Rationale: `:17238` sits before
  every `SwapRandom` — the bare `Main.rand` is the port's `global` stream
  (S1-A-10 one-stream-per-pass; the pre-swap chest-eat cues at
  `frame/world.rs:580` already ride it). The WEATHER-2 per-consumer convention
  is a documented deviation for the rain rig's own consumers; snowing is not
  one of them — its draws must be visible to the global stream's other
  consumers. (d130 §2 explicitly leaves this slot decision to the merger;
  this addendum recommends `global`.)
* **F2 — `MAX_SCREEN_W = 4096.0`** (g11 `main_dust.rs:54-56`, the HiDef pin
  `Main.cs:3709` + `:67047`) replaces d130's 1920 Reach pin (`snow.rs:42-43`).
  Re-derive the budget expectations in every snow witness under 4096.
* **F3 — restore the in-body `gamePaused` gate** (`:13040`): add
  `game_paused: bool` to `SnowInputs` (g11's `SnowingEnv` has it) and the
  early return. d130's "caller's pause gate" fold is unverified — the fused
  `scene_scan::update_scene` runs per-frame (`frame/world.rs:596`) and its
  pause gating is not established; the C# gates in-body before any draw.
* **F4 — the budget reads the counter, once per call** (d130's shape):
  `snow_dust` param, NOT g11's per-attempt `live_type(pool,76)` census. g11's
  census stand-in retires; its budget-break witness re-pins to the counter.
* **F5 — the int attempt-loop shape** (g11's `let mut i = 0; while (i as f32)
  < num3 { i += 1; … }`, the C# `for (int i…)`): optional, observation-equal
  to d130's float loop; take g11's for shape fidelity.
* **F6 — env sourcing stays d130's caller** (`wind.remix_world`,
  `rain.cloud_alpha()` — the raw unclamped intensity, identical to g11's
  `intensity_value()`; the live-Deerclops `hp > 0 && kind == Deerclops` scan
  is identical in both). g11's `scene_scan.rs:407-434` `SnowingEnv` block
  re-points at `crate::snow::SnowInputs` (+ `game_paused: app.game_paused`).
* **F7 — witness union.** g11's four snow witnesses port over re-pinned:
  `w4_g11_snowing_gates_before_any_draw` (add the paused case, F3),
  `w4_g11_snowing_mirrors_the_dry_arm_draw_sequence` and
  `…_the_storm_arm_with_deerclops` (re-derive the draw sequences on the
  `global` `UnifiedRandom` stream, F1), `…_breaks_on_the_live_76_budget`
  (counter input, F4; the `25e180d4` cap comment re-derives under F2 — at
  cloudAlpha 0 the 1280-px fixture gives `num2 = 156`, Deerclops `156/3 = 52`,
  cap `52*0.6 = 31.2` — unchanged, g11 already pinned 4096). d130's snow
  witnesses survive except budget-count expectations (F2).

**Retire at merge (keep-one):** g11 `main_dust.rs::snowing` + `SnowingEnv` +
its `MAX_SCREEN_W`/`SNOW_TILE_MAX` consts (the teleport half of `main_dust.rs`
is OUTSIDE this reconcile — see `116-teleport-reconcile.md`); d54
`world_dust_main.rs::snowing` + its `SnowingEnv` (latent + the `!t.active()`
tile-gate defect); d130's `Snow` RNG struct (F1). **Keep:** d130's
`type_76_arm`/`type_76_tails`/`draw` + `DustPool.snow_dust` (field, head reset
`particles.rs:329`, the `minecart::tick` hook) — g11's branch does not have
the counter yet, so the d130 `particles.rs`/`minecart.rs` hunks land with the
union; g11's `rain.rs` `Snowflake` retirement + the debug row's
`live_type(pool,76)` + the `gfx_quality.rs` `snow_cap` retirement comments
(re-point at `crate::snow`). The `sand.rs:35`/`rain.rs:889` 1920 pins are
flagged as the same HiDef deviation family for their own lanes — no change
here.

The rain/splash halves of d130's doc (its §1 impact-gate fix, §3 g67 splash
union) are untouched by this addendum — g11 has no rain-gate or splash code.

## Verification

Code-only mode per the task brief: no cargo/tests/graphify. Every row above
was re-derived by reading `Main.cs:13032-13116`, `:17225-17248`,
`:3702-3719`, `:66708-66716`, `:67040-67048`, `Dust.cs:1743-1764`,
`:2069-2084`, `Tile.cs:240-247` (via d130's cited re-read) and diffing the
three port bodies (`git show 73545f0b`/`50c845e5`/`48343a4c`) line-by-line.
The folds are a merger worklist, WRITTEN NOT APPLIED.

## §-patch — F1-F7 as EXACT patch-hunks (union-exec phase, W4-g11)

**d130 owns execution on ITS branch** (`task/W4-d130-weather-ambience`,
tip `ec1d73c7` at this writing — its `snow.rs` is byte-identical to
`50c845e5`'s, verified `git diff 50c845e5 ec1d73c7 -- crates/tr-app/src/snow.rs`
empty). Every hunk below quotes the OLD text exactly as it stands at that
tip (file + line anchors) and the NEW text to replace it with, so d130
applies mechanically. Line numbers are 1-based at `ec1d73c7`. The g11
`main_dust.rs::snowing` + `SnowingEnv` retirement is the MERGER's drop
(keep-one, §2) — not a d130 hunk. Body indentation inside the retired
`impl Snow` wrapper stays as-is after F1c (valid Rust; the merge-verify
wave's rustfmt normalises — no rustfmt in the code-only phase).

### F1 — the body + stream (global `UnifiedRandom`, the `Snow` splitmix retires)

**F1a — `crates/tr-app/src/snow.rs:9-11`** (module doc, the RNG bullet):

OLD:
```rust
//! * the RNG is this module's own splitmix stream (the per-consumer-stream
//!   convention — vanilla rolls `Main.rand`); the pool's `NewDust` rolls ride
//!   the same stream via `SplitmixRand`, exactly like `sand.rs`;
```
NEW:
```rust
//! * the RNG is the bare `Main.rand` — the caller passes
//!   `app.rngs.global()` (`:17238` sits BEFORE every `SwapRandom`, the
//!   S1-A-10 one-stream-per-pass contract; research/83 F1). The pool's
//!   `NewDust` rolls ride the same `UnifiedRandom`;
```

**F1b — `snow.rs:33`** (the use block; add the `UnifiedRandom` import):

OLD:
```rust
use tr_core::World;
```
NEW:
```rust
use tr_core::{UnifiedRandom, World};
```

**F1c — `snow.rs:71-113`** (retire the `Snow` struct + its splitmix; the
free-fn signature). OLD (the whole span, struct through the old signature):
```rust
pub(crate) struct Snow {
    rng: u64,
}

impl Snow {
    pub(crate) fn new(seed: u64) -> Self {
        Self {
            rng: (seed ^ 0x2545_F491_4F6C_DD1D).wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1,
        }
    }

    fn next_u64(&mut self) -> u64 {
        self.rng = self.rng.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.rng;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    fn rand01(&mut self) -> f32 {
        (self.next_u64() >> 40) as f32 / (1u32 << 24) as f32
    }

    /// `Main.rand.Next(max)` — `Next(0)` rolls 0 (XNA), the sand.rs convention.
    fn next_int(&mut self, max: u32) -> u32 {
        (self.next_u64() % max.max(1) as u64) as u32
    }

    /// `Main.snowing()` (`Main.cs:13032-13116`). `metrics` is the stored
    /// `Main.SceneMetrics` (the last scan), `snow_dust` the live type-76
    /// population the previous `UpdateDust` head counted (`Main.snowDust`),
    /// `ctx` the pool's spawn statics. The `gamePaused` early-out
    /// (`:13040`) folds into the caller's pause gate; `netMode != 2`
    /// (`:17234`) is the single-player fold.
    pub(crate) fn snowing(
        &mut self,
        metrics: &SceneMetrics,
        i: &SnowInputs,
        snow_dust: i32,
        ctx: &SpawnCtx,
        world: &World,
        pool: &mut DustPool,
    ) {
```
NEW:
```rust
/// `Main.snowing()` (`Main.cs:13032-13116`). `rng` is the bare
/// `Main.rand` (the caller passes `app.rngs.global()` — `:17238` pre-swap,
/// research/83 F1); `metrics` is the stored `Main.SceneMetrics` (the last
/// scan), `snow_dust` the live type-76 population the previous
/// `UpdateDust` head counted (`Main.snowDust`, read ONCE per call — F4),
/// `ctx` the pool's spawn statics. The `gamePaused` early-out (`:13040`)
/// is IN-BODY (F3); `netMode != 2` (`:17234`) is the single-player fold.
pub(crate) fn snowing(
    rng: &mut UnifiedRandom,
    metrics: &SceneMetrics,
    i: &SnowInputs,
    snow_dust: i32,
    ctx: &SpawnCtx,
    world: &World,
    pool: &mut DustPool,
) {
```

**F1c-tail — `snow.rs:239-241`** (drop the retired `impl`'s closing brace;
the loop's `}` and the fn's `}` stay). OLD:
```rust
        }
    }
}
```
NEW:
```rust
        }
}
```
(The body between F1c and this tail keeps its current indentation —
mechanical; rustfmt at merge-verify normalises.)

**F1d — the draw sites inside the body** (each OLD → NEW, exact):

`snow.rs:149`:
```rust
            if !(self.rand01() < num) {
```
→
```rust
            if !(rng.next_float() < num) {
```

`snow.rs:154-155`:
```rust
            let mut num4 = self.next_int(i.scaled_size.x as u32 + 1500) as i32 - 750;
            let mut num5 = i.scaled_position.y as i32 - self.next_int(50) as i32;
```
→
```rust
            let mut num4 = rng.next_max(i.scaled_size.x as i32 + 1500) - 750;
            let mut num5 = i.scaled_position.y as i32 - rng.next_max(50);
```

`snow.rs:162-166`:
```rust
            if self.next_int(5) == 0 {
                num4 = self.next_int(500) as i32 - 500;
            } else if self.next_int(5) == 0 {
                num4 = self.next_int(500) as i32 + i.scaled_size.x as i32;
            }
```
→
```rust
            if rng.next_max(5) == 0 {
                num4 = rng.next_max(500) - 500;
            } else if rng.next_max(5) == 0 {
                num4 = rng.next_max(500) + i.scaled_size.x as i32;
            }
```

`snow.rs:169-170`:
```rust
                num5 += self.next_int((i.scaled_size.y as f64 * 0.8) as u32) as i32
                    + (i.scaled_size.y as f64 * 0.1) as i32;
```
→
```rust
                num5 += rng.next_max((i.scaled_size.y as f64 * 0.8) as i32)
                    + (i.scaled_size.y as f64 * 0.1) as i32;
```

`snow.rs:189`:
```rust
                &mut crate::app::SplitmixRand(&mut self.rng),
```
→
```rust
                rng,
```

`snow.rs:205`:
```rust
            d.velocity.y = 3.0 + self.next_int(30) as f32 * 0.1;
```
→
```rust
            d.velocity.y = 3.0 + rng.next_max(30) as f32 * 0.1;
```

`snow.rs:210-211` (the dry drift takes the C# `Next(-10,10)` shape, not
the `Next(20)-10` fold — same values, the g11/C# form):
```rust
                d.velocity.x = i.wind_speed_current
                    + (self.next_int(20) as i32 - 10) as f32 * 0.1;
```
→
```rust
                d.velocity.x = i.wind_speed_current
                    + rng.next_range(-10, 10) as f32 * 0.1;
```

`snow.rs:229`:
```rust
                    + self.rand01() * 0.2
```
→
```rust
                    + rng.next_float() * 0.2
```

**F1e — the `App.snow` field retires** (three sites):

`crates/tr-app/src/app/mod.rs:889-890` — OLD:
```rust
    /// The ambient snowfall (`Main.snowing()`, W4-d130).
    pub(crate) snow: crate::snow::Snow,
```
NEW: (delete both lines)

`crates/tr-app/src/app/mod.rs:3550` — OLD:
```rust
            snow: crate::snow::Snow::new(1),
```
NEW: (delete the line)

`crates/tr-app/src/app/flow.rs:2427` — OLD:
```rust
            snow: crate::snow::Snow::new(1),
```
NEW: (delete the line)

### F2 — `MAX_SCREEN_W = 4096.0` (the HiDef pin)

`snow.rs:15-18` (module doc) — OLD:
```rust
//! * `Main.maxScreenW` is pinned at its Reach value 1920
//!   (`Main.cs:66715`/`:3715` — the port's `MaxWorldViewSize.X` convention,
//!   `rain.rs`'s spawn rate uses the same pin; HiDef's 4096 arm is a
//!   documented drop);
```
NEW:
```rust
//! * `Main.maxScreenW` is pinned at its HiDef value 4096
//!   (`Main.cs:3709` + `:67047` — the non-XNA default selects HiDef; the
//!   port renders HiDef, `frame/draw.rs:65` / `ambience.rs:184-187`;
//!   research/83 F2 — the `sand.rs`/`rain.rs` 1920 pins are their own
//!   lanes' recorded deviation family);
```

`snow.rs:42-43` — OLD:
```rust
/// `Main.maxScreenW` — the Reach pin (see the module doc).
const MAX_SCREEN_W: f32 = 1920.0;
```
NEW:
```rust
/// `Main.maxScreenW` under the HiDef profile (`Main.cs:3709` — the port
/// renders HiDef; research/83 F2).
const MAX_SCREEN_W: f32 = 4096.0;
```

### F3 — the in-body `gamePaused` gate

`snow.rs:67-68` (the `SnowInputs` tail) — OLD:
```rust
    /// `Main.remixWorld` (`:13034-13037`).
    pub remix_world: bool,
```
NEW:
```rust
    /// `Main.remixWorld` (`:13034-13037`).
    pub remix_world: bool,
    /// `Main.gamePaused` (`:13040`) — the IN-BODY early return (F3: the
    /// fused per-frame `scene_scan::update_scene` runs paused; the C#
    /// gates before any draw).
    pub game_paused: bool,
```

`snow.rs:118-121` — OLD:
```rust
        // `:13040-13043`: no snow tiles / below the surface — nothing.
        if metrics.snow_tile_count <= 0 || !metrics.surface_atmospherics {
            return;
        }
```
NEW:
```rust
        // `:13040-13043`: paused / no snow tiles / below the surface —
        // nothing (the pause gate is IN-BODY, research/83 F3).
        if i.game_paused || metrics.snow_tile_count <= 0 || !metrics.surface_atmospherics {
            return;
        }
```

### F4 — the budget reads the counter ONCE per call

**No body hunk** — d130's `snow_dust: i32` param + the constant-within-
call gate (`snow.rs:143-147`) IS the F4 shape; it survives verbatim. The
g11 per-attempt `live_type(pool,76)` census retires with the g11 body at
merge (the merger's drop). The F7 budget witnesses re-pin to the param.

### F5 — the int attempt-loop shape

`snow.rs:138-140` — OLD:
```rust
        let mut attempt = 0.0_f32;
        while attempt < num3 {
            attempt += 1.0;
```
NEW:
```rust
        let mut attempt = 0i32;
        while (attempt as f32) < num3 {
            attempt += 1;
```
(The C# `for (int i = 0; (float)i < num3; i++)` shape; the counter keeps
the `attempt` name — `i` is the `SnowInputs` param. Observation-equal,
exact ≤ 2^24.)

### F6 — env sourcing stays d130's caller (+ the pause field, the free call)

`crates/tr-app/src/world_sim/scene_scan.rs:399-429` — OLD:
```rust
    // `Main.snowing()` (`Main.cs:17238`, body `:13032-13116`): the ambient
    // snowfall spawns right BEFORE `Sandstorm.EmitDust` in the C# — the port
    // runs both in this slot (the spawn order between the two dust families
    // only matters for the shared pool's slot scan; the snow's own stream is
    // separate). The budget reads the previous tick's `Main.snowDust`
    // (`pool.snow_dust`, the shared tick's count). W4-d130.
    let snow_inputs = crate::snow::SnowInputs {
        scaled_position: *camera - screen * 0.5 / *zoom,
        scaled_size: screen / *zoom,
        cloud_alpha: rain.cloud_alpha(),
        raining: rain.is_raining(),
        wind_speed_current: wind.speed_current,
        player_vy,
        gfx_quality: app.gfx_quality.quality,
        // `NPC.IsADeerclopsNearScreen()` (`NPC.cs:44534-44537`): the active
        // 668 scan (`npcsFoundForCheckActive[668]`) — the destructure's npcs.
        deerclops_near: npcs
            .iter()
            .any(|n| n.hp > 0 && n.kind == crate::npc::NpcKind::Deerclops),
        remix_world: wind.remix_world,
    };
    let snow_dust = app.dust.snow_dust;
    let snow_metrics = app.scene_metrics.clone();
    app.snow.snowing(
        &snow_metrics,
        &snow_inputs,
        snow_dust,
        &ctx,
        world,
        &mut app.dust,
    );
```
NEW:
```rust
    // `Main.snowing()` (`Main.cs:17238`, body `:13032-13116`): the ambient
    // snowfall spawns right BEFORE `Sandstorm.EmitDust` in the C# — the port
    // runs both in this slot. The stream is the BARE `Main.rand` (the port's
    // `NamedRngs::global()` — `:17238` sits before every `SwapRandom`,
    // research/83 F1; the pre-swap chest-eat cues already ride it). The
    // budget reads the previous tick's `Main.snowDust` (`pool.snow_dust`,
    // the shared tick's count, ONCE per call — F4). W4-d130 + g11 F1-F6.
    let snow_inputs = crate::snow::SnowInputs {
        scaled_position: *camera - screen * 0.5 / *zoom,
        scaled_size: screen / *zoom,
        cloud_alpha: rain.cloud_alpha(),
        raining: rain.is_raining(),
        wind_speed_current: wind.speed_current,
        player_vy,
        gfx_quality: app.gfx_quality.quality,
        // `NPC.IsADeerclopsNearScreen()` (`NPC.cs:44534-44537`): the active
        // 668 scan (`npcsFoundForCheckActive[668]`) — the destructure's npcs.
        deerclops_near: npcs
            .iter()
            .any(|n| n.hp > 0 && n.kind == crate::npc::NpcKind::Deerclops),
        remix_world: wind.remix_world,
        // F3: the in-body pause gate reads the live flag.
        game_paused: app.game_paused,
    };
    let snow_dust = app.dust.snow_dust;
    let snow_metrics = app.scene_metrics.clone();
    crate::snow::snowing(
        app.rngs.global(),
        &snow_metrics,
        &snow_inputs,
        snow_dust,
        &ctx,
        world,
        &mut app.dust,
    );
```
(The g11 `scene_scan.rs:407-434` `SnowingEnv` block is the MERGER's drop —
keep-one takes this d130 block; the env values are identical per §1:
`rain.cloud_alpha()` = g11's `intensity_value()` raw intensity, the
Deerclops scan and the view-rect fold match.)

### F7 — the witness union (re-pins + the four g11 mirrors)

**F7a — `snow.rs:386-398`** (`inputs()` gains the pause field) — OLD:
```rust
    fn inputs() -> SnowInputs {
        SnowInputs {
            scaled_position: CAMERA - SCREEN * 0.5,
            scaled_size: SCREEN,
            cloud_alpha: 0.0,
            raining: false,
            wind_speed_current: 0.0,
            player_vy: 0.0,
            gfx_quality: 1.0,
            deerclops_near: false,
            remix_world: false,
        }
    }
```
NEW:
```rust
    fn inputs() -> SnowInputs {
        SnowInputs {
            scaled_position: CAMERA - SCREEN * 0.5,
            scaled_size: SCREEN,
            cloud_alpha: 0.0,
            raining: false,
            wind_speed_current: 0.0,
            player_vy: 0.0,
            gfx_quality: 1.0,
            deerclops_near: false,
            remix_world: false,
            game_paused: false,
        }
    }
```

**F7b — the mechanical call-site re-shape** (every remaining witness;
`Snow::new(N)` → a pinned `UnifiedRandom`, `x.snowing(A, …)` →
`snowing(&mut x, A, …)`). The exact pairs, by line at `ec1d73c7`:

| line | OLD | NEW |
| --- | --- | --- |
| 434 | `let mut dry = Snow::new(11);` | `let mut dry = UnifiedRandom::new(11);` |
| 438 | `dry.snowing(&m, &i, 0, &ctx(), &w, &mut dry_pool);` | `snowing(&mut dry, &m, &i, 0, &ctx(), &w, &mut dry_pool);` |
| 440 | `let mut storm = Snow::new(11);` | `let mut storm = UnifiedRandom::new(11);` |
| 445 | `storm.snowing(&m, &i, 0, &ctx(), &w, &mut storm_pool);` | `snowing(&mut storm, &m, &i, 0, &ctx(), &w, &mut storm_pool);` |
| 489 | `let mut near = Snow::new(17);` | `let mut near = UnifiedRandom::new(17);` |
| 494 | `near.snowing(&m, &i, 0, &ctx(), &w, &mut near_pool);` | `snowing(&mut near, &m, &i, 0, &ctx(), &w, &mut near_pool);` |
| 496 | `let mut far = Snow::new(17);` | `let mut far = UnifiedRandom::new(17);` |
| 500 | `far.snowing(&m, &i, 0, &ctx(), &w, &mut far_pool);` | `snowing(&mut far, &m, &i, 0, &ctx(), &w, &mut far_pool);` |
| 527 | `let mut dry = Snow::new(19);` | `let mut dry = UnifiedRandom::new(19);` |
| 530 | `dry.snowing(&m, &i, 0, &ctx(), &w, &mut dry_pool);` | `snowing(&mut dry, &m, &i, 0, &ctx(), &w, &mut dry_pool);` |
| 533 | `let mut storm = Snow::new(19);` | `let mut storm = UnifiedRandom::new(19);` |
| 536 | `storm.snowing(&m, &i, 0, &ctx(), &w, &mut storm_pool);` | `snowing(&mut storm, &m, &i, 0, &ctx(), &w, &mut storm_pool);` |
| 572 | `let mut s = Snow::new(23);` | `let mut s = UnifiedRandom::new(23);` |
| 575 | `s.snowing(&m, &i, 0, &ctx(), &w, &mut pool);` | `snowing(&mut s, &m, &i, 0, &ctx(), &w, &mut pool);` |
| 639 | `let mut s = Snow::new(29);` | `let mut s = UnifiedRandom::new(29);` |
| 642 | `s.snowing(&m, &i, 0, &ctx(), &w, &mut pool);` | `snowing(&mut s, &m, &i, 0, &ctx(), &w, &mut pool);` |

(The gates witness at 411-425 and the budget witness at 456-480 are
replaced WHOLESALE by F7d/F7c below — their `Snow::new` sites are not in
this table. The statistical expectations of the intensity/velocity/
determinism/recipe witnesses are stream-family-agnostic — they survive
the splitmix→UnifiedRandom swap unchanged.)

**F7c — `snow.rs:456-480`** (the budget witness re-derives under F2's
4096: `(int)(500*1600/4096) = 195`; quality-1 cap `195*1.1 = 214.5`;
quality-0.5 cap `195*0.75 + 19.5 = 165.75`) — OLD:
```rust
    #[test]
    fn the_budget_gate_breaks_the_loop() {
        // `Main.cs:13057`: at quality 1 the budget is num2 * 1.1; the dry
        // 1600-px num2 is `(int)(500*1600/1920) = 416` → 457.6. A snowDust
        // already at 458 spawns NOTHING (the break, not a skip).
        let w = snow_world();
        let m = metrics(6000, true); // intensity 1: every attempt passes
        let mut s = Snow::new(13);
        let mut pool = DustPool::new();
        let i = inputs();
        s.snowing(&m, &i, 458, &ctx(), &w, &mut pool);
        assert_eq!(live_type(&pool, 76), 0, "the budget break");
        // One under the budget the loop runs (and the pool fills).
        for _ in 0..30 {
            s.snowing(&m, &i, 457, &ctx(), &w, &mut pool);
        }
        assert!(live_type(&pool, 76) > 0);
        // The gfxQuality scale: at 0.5 the budget is 416*0.75 + 41.6 = 353.6.
        let mut q = Snow::new(13);
        let mut q_pool = DustPool::new();
        let mut qi = inputs();
        qi.gfx_quality = 0.5;
        q.snowing(&m, &qi, 354, &ctx(), &w, &mut q_pool);
        assert_eq!(live_type(&q_pool, 76), 0, "the quality-scaled budget");
    }
```
NEW:
```rust
    #[test]
    fn the_budget_gate_breaks_the_loop() {
        // `Main.cs:13057`: at quality 1 the budget is num2 * 1.1; the dry
        // 1600-px num2 under the HiDef 4096 pin (F2) is
        // `(int)(500*1600/4096) = 195` → 214.5. A snowDust already at 215
        // spawns NOTHING (the break, not a skip; F4 — the param is read
        // once per call).
        let w = snow_world();
        let m = metrics(6000, true); // intensity 1: every attempt passes
        let mut s = UnifiedRandom::new(13);
        let mut pool = DustPool::new();
        let i = inputs();
        snowing(&mut s, &m, &i, 215, &ctx(), &w, &mut pool);
        assert_eq!(live_type(&pool, 76), 0, "the budget break");
        assert_eq!(s, UnifiedRandom::new(13), "the break precedes any draw");
        // One under the budget the loop runs (and the pool fills).
        for _ in 0..30 {
            snowing(&mut s, &m, &i, 214, &ctx(), &w, &mut pool);
        }
        assert!(live_type(&pool, 76) > 0);
        // The gfxQuality scale: at 0.5 the budget is 195*0.75 + 19.5 = 165.75.
        let mut q = UnifiedRandom::new(13);
        let mut q_pool = DustPool::new();
        let mut qi = inputs();
        qi.gfx_quality = 0.5;
        snowing(&mut q, &m, &qi, 166, &ctx(), &w, &mut q_pool);
        assert_eq!(live_type(&q_pool, 76), 0, "the quality-scaled budget");
    }
```

`snow.rs:484-486` (the Deerclops comment re-derives) — OLD:
```rust
        // `Main.cs:13047-13052`: num /= 20, num2 /= 3 — at count 6000 the
        // dry intensity 1 → 0.05 and the budget 457.6 → 152*1.1 = 167.2.
        // The per-flake scale also drops 0.5 (`:13104-13107`).
```
NEW:
```rust
        // `Main.cs:13047-13052`: num /= 20, num2 /= 3 — at count 6000 the
        // dry intensity 1 → 0.05 and the budget 214.5 → 65*1.1 = 71.5
        // (the 4096 pin, F2). The per-flake scale also drops 0.5
        // (`:13104-13107`).
```

**F7d — `snow.rs:411-425`** (the gates witness gains the F3 pause case +
the zero-draw stream pin — g11 `w4_g11_snowing_gates_before_any_draw`
folded in) — OLD:
```rust
    #[test]
    fn the_gates_need_snow_tiles_at_the_surface_and_no_remix() {
        // `Main.cs:13034-13043`: the three early returns.
        let w = snow_world();
        let mut s = Snow::new(7);
        let mut pool = DustPool::new();
        let mut i = inputs();
        s.snowing(&metrics(0, true), &i, 0, &ctx(), &w, &mut pool);
        assert_eq!(live_type(&pool, 76), 0, "no snow tiles");
        s.snowing(&metrics(3000, false), &i, 0, &ctx(), &w, &mut pool);
        assert_eq!(live_type(&pool, 76), 0, "below the surface");
        i.remix_world = true;
        s.snowing(&metrics(3000, true), &i, 0, &ctx(), &w, &mut pool);
        assert_eq!(live_type(&pool, 76), 0, "the remix world");
    }
```
NEW:
```rust
    #[test]
    fn the_gates_need_snow_tiles_at_the_surface_and_no_remix() {
        // `Main.cs:13034-13043`: the FOUR early returns (F3 adds pause);
        // none draws (the g11 gates witness folded in, F7).
        let w = snow_world();
        let mut s = UnifiedRandom::new(7);
        let mut pool = DustPool::new();
        let mut i = inputs();
        snowing(&mut s, &metrics(0, true), &i, 0, &ctx(), &w, &mut pool);
        assert_eq!(live_type(&pool, 76), 0, "no snow tiles");
        snowing(&mut s, &metrics(3000, false), &i, 0, &ctx(), &w, &mut pool);
        assert_eq!(live_type(&pool, 76), 0, "below the surface");
        i.remix_world = true;
        snowing(&mut s, &metrics(3000, true), &i, 0, &ctx(), &w, &mut pool);
        assert_eq!(live_type(&pool, 76), 0, "the remix world");
        i.remix_world = false;
        i.game_paused = true;
        snowing(&mut s, &metrics(3000, true), &i, 0, &ctx(), &w, &mut pool);
        assert_eq!(live_type(&pool, 76), 0, "the pause gate (F3)");
        assert_eq!(s, UnifiedRandom::new(7), "no gate draws");
    }
```

**F7e — the three g11 draw-mirror witnesses port over** (INSERT before the
test module's closing brace, after `the_spawn_recipe_matches_the_csharp_order`).
Fixture constants re-derived: view top-left `CAMERA - SCREEN*0.5 =
(800, 1070)`, size 1600×900; the `NewDust` cull rect (`Dust.cs:97-101`,
dCount 0 → margin 400) is `(400, 670, 2400, 1700)`, XNA `Intersects`
strict `<`; the snow rows 95+ are ACTIVE (the tile gate refuses
`num5/16 >= 95`); every rolled x lands in-world (base `[50, 3149]`, the
edge pushes `[300, 799]`/`[2400, 2899]` — all < 3200). NEW (whole):
```rust
    /// g11 `w4_g11_snowing_mirrors_the_dry_arm_draw_sequence` (F7,
    /// re-pinned to the d130 fixture + the global UnifiedRandom stream):
    /// cloudAlpha 0 → ONE attempt, intensity 1 → the gate always passes;
    /// the mirror walks every draw in C# order and pins the flake —
    /// a culled spawn draws NO internals but the recipe rolls still ride
    /// the stream (the sentinel takes the writes, `Dust.cs:79`).
    #[test]
    fn the_dry_arm_mirrors_the_draw_sequence() {
        let w = snow_world();
        let m = metrics(6000, true);
        let mut i = inputs();
        i.player_vy = 5.0;
        i.wind_speed_current = 0.3;
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(0x5EED);
        snowing(&mut rng, &m, &i, 0, &ctx(), &w, &mut pool);
        let mut mirror = UnifiedRandom::new(0x5EED);
        let _ = mirror.next_float(); // the spawn gate (`:13061`)
        let mut num4 = mirror.next_max(1600 + 1500) - 750;
        let mut num5 = 1070 - mirror.next_max(50);
        num5 -= 5; // player_vy > 0 (`:13067-13070`)
        if mirror.next_max(5) == 0 {
            num4 = mirror.next_max(500) - 500;
        } else if mirror.next_max(5) == 0 {
            num4 = mirror.next_max(500) + 1600;
        }
        if num4 < 0 || num4 as f32 > 1600.0 {
            num5 += mirror.next_max((900.0f64 * 0.8) as i32) + (900.0f64 * 0.1) as i32;
        }
        num4 += 800;
        let open = (num5 / 16) < 95; // rows 95+ are active snow
        let passes =
            400 < num4 + 10 && num4 < 400 + 2400 && 670 < num5 + 10 && num5 < 670 + 1700;
        let mut v = Vec2::ZERO;
        let mut s = 0.0;
        if open {
            if passes {
                // The six default-block draws (`Dust.cs:153-193`, 10×10).
                let px = num4 as f32 + mirror.next_max(6) as f32 + 4.0;
                let py = num5 as f32 + mirror.next_max(6) as f32 + 4.0;
                let vx = mirror.next_range(-20, 21) as f32 * 0.1;
                let vy = mirror.next_range(-20, 21) as f32 * 0.1;
                let _ = mirror.next_max(3);
                let scale = 1.0 + mirror.next_range(-20, 21) as f32 * 0.01;
                let d = pool.slots().iter().find(|d| d.active).expect("the flake");
                assert_eq!(d.dust_type, 76);
                assert_eq!(d.position, Vec2::new(px, py));
                v = Vec2::new(vx, vy);
                s = scale;
            } else {
                assert_eq!(live_type(&pool, 76), 0, "the cull refuses the spawn");
            }
            // `:13089-13095` + the shared tail at cloudAlpha 0.
            v.y = 3.0 + mirror.next_max(30) as f32 * 0.1;
            v.y *= s;
            v.x = 0.3 + mirror.next_range(-10, 10) as f32 * 0.1;
            v.x += 0.3 * 15.0;
            v.y *= 1.0;
            if passes {
                let d = pool.slots().iter().find(|d| d.active).unwrap();
                assert_eq!(d.velocity, v);
                assert_eq!(d.scale, s);
            } else {
                assert_eq!(
                    pool.slot(DUST_SENTINEL).velocity.x,
                    v.x,
                    "the sentinel took the writes"
                );
            }
        }
        assert_eq!(rng, mirror);
    }

    /// g11 `w4_g11_snowing_mirrors_the_storm_arm_with_deerclops` (F7):
    /// cloudAlpha 0.5 → 26 attempts, the 0.05 gate, the sqrt/sign storm
    /// drift, the double scale bump + the Deerclops −0.5 cut. F4: the
    /// budget reads the CONSTANT snow_dust param (0 here — num2 =
    /// (int)(500*1600/4096)*2/3 = 130, quality-1 cap 143, never reached
    /// in 26 attempts), NOT a growing live census.
    #[test]
    fn the_storm_arm_mirrors_with_deerclops() {
        let mut any_spawn = false;
        for seed in [99i32, 7, 0x5EED, 12345] {
            let w = snow_world();
            let m = metrics(6000, true);
            let mut i = inputs();
            i.cloud_alpha = 0.5;
            i.raining = true;
            i.wind_speed_current = -0.4;
            i.deerclops_near = true;
            i.player_vy = 5.0;
            let mut pool = DustPool::new();
            let mut rng = UnifiedRandom::new(seed);
            snowing(&mut rng, &m, &i, 0, &ctx(), &w, &mut pool);
            let mut mirror = UnifiedRandom::new(seed);
            let num = 1.0f32 / 20.0;
            let mut spawned = 0usize;
            let mut attempt = 0i32;
            while (attempt as f32) < 1.0 + 50.0 * 0.5 {
                attempt += 1;
                // F4: the constant param (0) — the gate never breaks here.
                if !(0.0f32 < 130.0 * 1.0 + 13.0) {
                    break;
                }
                if !(mirror.next_float() < num) {
                    continue;
                }
                let mut num4 = mirror.next_max(1600 + 1500) - 750;
                let mut num5 = 1070 - mirror.next_max(50) - 5;
                if mirror.next_max(5) == 0 {
                    num4 = mirror.next_max(500) - 500;
                } else if mirror.next_max(5) == 0 {
                    num4 = mirror.next_max(500) + 1600;
                }
                if num4 < 0 || num4 as f32 > 1600.0 {
                    num5 += mirror.next_max((900.0f64 * 0.8) as i32)
                        + (900.0f64 * 0.1) as i32;
                }
                num4 += 800;
                let open = (num5 / 16) < 95;
                let passes = open
                    && 400 < num4 + 10
                    && num4 < 400 + 2400
                    && 670 < num5 + 10
                    && num5 < 670 + 1700;
                let mut v = Vec2::ZERO;
                let mut s = 0.0;
                if open {
                    if passes {
                        let px = num4 as f32 + mirror.next_max(6) as f32 + 4.0;
                        let py = num5 as f32 + mirror.next_max(6) as f32 + 4.0;
                        let vx = mirror.next_range(-20, 21) as f32 * 0.1;
                        let vy = mirror.next_range(-20, 21) as f32 * 0.1;
                        let _ = mirror.next_max(3);
                        let scale = 1.0 + mirror.next_range(-20, 21) as f32 * 0.01;
                        let d = pool.slot(spawned);
                        assert!(d.active);
                        assert_eq!(d.dust_type, 76);
                        assert_eq!(d.position, Vec2::new(px, py));
                        v = Vec2::new(vx, vy);
                        s = scale;
                        spawned += 1;
                    }
                    s += 0.5 * 0.2;
                    v.y = 3.0 + mirror.next_max(30) as f32 * 0.1;
                    v.y *= s;
                    v.x = (0.4f64.sqrt() as f32) * -1.0 * (0.5 + 0.5) * 10.0
                        + mirror.next_float() * 0.2
                        - 0.1;
                    v.y *= 0.5;
                    v.y *= 1.0 + 0.3 * 0.5;
                    s += 0.5 * 0.2;
                    s -= 0.5; // the Deerclops cut
                    v *= 1.0 + 0.5 * 0.5;
                    if passes {
                        let d = pool.slot(spawned - 1);
                        assert_eq!(d.velocity, v);
                        assert_eq!(d.scale, s);
                    }
                }
            }
            assert_eq!(live_type(&pool, 76), spawned);
            assert_eq!(rng, mirror);
            any_spawn |= spawned > 0;
        }
        assert!(any_spawn, "the seed sweep must pass the 0.05 gate");
    }

    /// g11 `w4_g11_snowing_breaks_on_the_live_76_budget` (F7, re-pinned
    /// to the F4 counter input + the F2 4096 budget): Deerclops at
    /// cloudAlpha 0 → num2 = 195/3 = 65, quality-1 cap 65*1.1 = 71.5; a
    /// counter already at 72 spends nothing.
    #[test]
    fn the_budget_breaks_on_the_counter() {
        let w = snow_world();
        let m = metrics(6000, true);
        let mut i = inputs();
        i.deerclops_near = true;
        let mut pool = DustPool::new();
        let mut rng = UnifiedRandom::new(1);
        snowing(&mut rng, &m, &i, 72, &ctx(), &w, &mut pool);
        assert_eq!(live_type(&pool, 76), 0);
        assert_eq!(rng, UnifiedRandom::new(1));
    }
```

### §-patch verification

Code-only (no cargo/tests/graphify per the lane directive). Every OLD
block was copied verbatim from `git show
origin/task/W4-d130-weather-ambience:crates/tr-app/src/snow.rs` (and
`scene_scan.rs`/`app/mod.rs`/`app/flow.rs`) at tip `ec1d73c7`; the NEW
blocks re-derive the g11 body (`73545f0b`, this branch's
`main_dust.rs:104-231` pre-union) against `Main.cs:13032-13116` and the
d130 fixture constants. The F7e mirrors' cull/tile-gate branches were
walked by hand for the fixture's roll ranges (no execution — WRITTEN NOT
RUN, the merge-verify wave executes).

## §-adjudication — the `:17238` SwapRandom scope (g11 final phase, d130 tip `560a02ea`)

> **SUPERSEDED-IN-STATUS by the §-receipt below (dust-retire phase):** d130e
> (`099df89e2`) APPLIED F1+F5+F7 on its own branch — the splitmix is retired
> there. The scope walk below STANDS unchanged; the REFUTED-AS-APPLIED
> disposition rows describe the `560a02ea` tip and are historical.

The task brief flagged the snow-stream question as an ACTIVE
adjudication: the g11 F1 claim (the bare `Main.rand` / `NamedRngs::global`
slot) vs d130's kept per-consumer splitmix. Re-derived from the C# at the
1.4.5.8 lines, call-chain first:

1. `Main.Update(GameTime)` (`Main.cs:16774`) calls `this.DoUpdate(ref
   gameTime)` (`:16788`) — no `SwapRandom` anywhere in `Update`.
2. `DoUpdate` (`:16828-17292`) runs `Main.snowing()` at **`:17238`**
   (inside the `netMode != 2` try/catch `:17234-17246`), then
   `Sandstorm.EmitDust()` `:17247`. The FIRST `SwapRandom` in the whole
   `DoUpdate` body is at **`:17732`** — inside `DoUpdateInWorld`
   (`:17726`), which `DoUpdate` calls LATER at `:17280` (behind
   `ShouldUpdateEntities() && WorldUpdateStepper.ShouldUpdateWorld()`).
   Every other swap (`:17749-17809`, `:17887`) sits inside that method
   or its callees. `grep SwapRandom` over `:16828-17725` returns NOTHING.
3. `Main.SwapRandom(key)` (`:3204-3215`) returns `RandomSwap.Swap(value)`
   — an `IDisposable` that RESTORES the previous `Main.rand` on dispose.
   So even the `UpdateTime` swap (`:17795`, the `UpdateWeather` chain
   `:17230` runs BEFORE snowing but inside the earlier per-pass scopes)
   cannot leak: at `:17238` `Main.rand` is the bare static
   (`:1292`, seeded `:5194`/`:6463`), the same stream the draw path and
   every pre-swap consumer rolls.

**Verdict: the g11 pre-swap claim STANDS as the faithful C# reading.**
`:17238` draws on the bare `Main.rand`; the port's bare-stream slot is
`NamedRngs::global()` (the S1-A-10 contract; g13's draw-path sites and
the pre-swap chest-eat cues already ride it). d130's kept splitmix is a
DOCUMENTED DEVIATION (its WEATHER-2 per-consumer convention), not a
scope correction — its own §2 says "if the merger prefers g11's
`NamedRngs::global` slot instead, that is a stream-convention decision,
not a maths one". d130's final-phase commits did NOT re-trace the scope
(its §4 keeps the stream with the §2 delegation intact); nothing in its
tip refutes the `:17238` pre-swap maths.

**Disposition (aligned, not re-litigated):**
* F1 stays PUBLISHED as the recommendation (the hunks above apply
  mechanically if the merger takes the global slot). It is marked
  REFUTED-AS-APPLIED in the status header only in the sense that d130
  chose not to apply it — the merger owns the decision per d130 §2.
* F2/F3 landed on d130 (`85250712`): `snow.rs:43` `MAX_SCREEN_W = 4096.0`,
  `SnowInputs.game_paused` + the in-body disjunct (`snow.rs:125`),
  `update_scene` feeds `app.game_paused` — verified by reading the tip.
* F4 was already d130's shape (the `snow_dust: i32` param read once per
  call, `snow.rs:112`/`:149`); F5 (the int loop) NOT taken — d130 keeps
  its float `attempt` loop (`snow.rs:144-146`), observation-equal per
  §1's table (exact ≤ 2^24), no action owed.
* F6/F7 (env re-point + witness migration) are the MERGER's work at the
  keep-one drop — g11's `scene_scan.rs:407-434` block re-points at
  `crate::snow::SnowInputs`, the four g11 snow witnesses port per F7.
* g11's body is marked DEPRECATED in-code this phase (`main_dust.rs`
  module doc RETIRE-NOTE + the `snowing` fn doc); the branch keeps
  compiling — no removal, no witness loss before the merger's drop.
* The g13 sweep's "still-owed g11 snowing budget-hoist" (research/68 §1,
  d54 research/66 §4.3) resolves the same way: the hoist is F4, d130's
  surviving body already reads the counter once per call; g11's
  per-attempt `live_type` census retires WITH the deprecated body at the
  merge — no separate g11b fix is owed on this branch.

Verification: code-only. The scope walk re-read `Main.cs:16774-16800`,
`:16828-16887`, `:17170-17292`, `:17726-17810`, `:3204-3215`, `:1292`,
`:5194`, `:6463` directly; the d130 tip's `snow.rs` re-read at
`560a02ea` (the splitmix struct `:75-101` alive, `rand01`/`next_int`
draw sites `:155-235`, `MAX_SCREEN_W` `:43`, `game_paused` `:53`/`:125`).

## §-receipt — d130e APPLIED the global-stream fold (dust-retire phase, W4-g11)

**F1-stands → F1-APPLIED.** The adjudication above read d130 at
`560a02ea` (origin tip) where the splitmix was kept by §2's
merger-delegation. The d130e session then applied the fold on ITS branch:

* **`099df89e2` "adjudication F1+F5+F7"** — the `Snow` splitmix struct +
  `App.snow` field + both init sites RETIRED (`app/mod.rs`: "W4-d130 F1:
  no `snow` field"; `app/flow.rs` init dropped); `snowing` is a free fn
  `pub(crate) fn snowing(…, rng: &mut UnifiedRandom, …)` and
  `world_sim/scene_scan.rs` passes **`app.rngs.global()`** at the `:17238`
  slot with the pre-swap scope proof cited in-line; the pool `NewDust`
  rides `DustRng for UnifiedRandom`. **F5** landed too:
  `while (attempt as f32) < num3` (the C# `for (int i…)` shape — the
  earlier "float loop kept observation-equal" disposition is retired).
  **F7** applied: the d130 witnesses re-pinned to `UnifiedRandom`
  (gate/budget arms pin ZERO stream draws) and g11's three draw-sequence
  mirrors PORTED by name — `w4_g11_snowing_mirrors_the_dry_arm_draw_sequence`,
  `w4_g11_snowing_mirrors_the_storm_arm_with_deerclops` (the 26-attempt
  frozen-counter rig), `w4_g11_snowing_breaks_on_the_counter_budget`
  (the 4096-pin cap 31.2).
* **`16a792d56` (d130 local tip)** — the g67b-post-union fold; for THIS
  lane it adds the cull-kill into `snow::draw` (`splash_cull_rect` /
  `dust_in_cull_rect`, `Main.cs:38832`/`:38847`/`:39020-39022` — the same
  DrawDust loop covers type 76). F2 `MAX_SCREEN_W = 4096.0`, F3
  `SnowInputs.game_paused` + the in-body disjunct, F4 the counter-once
  param all re-verified alive at this tip.
* **Counter feed verified faithful:** `Dust.UpdateDust` resets
  `Main.snowDust = 0` (`Dust.cs:428`) then counts the live 76 arm
  (`:1747`); the port's `particles.rs` mirrors it (`snow_dust = 0` at
  the reset `:329`, `minecart.rs` recomputes per tick → `pool.snow_dust`),
  and `scene_scan` reads the previous tick's count — same ordering as
  the C# (`UpdateDust` `:18055` runs after snowing `:17238`).

**Push-rescue flag (d130):** the fold commits `099df89e2` + `16a792d56`
are LOCAL-ONLY on `TR-W4-d130` — origin tip is still `560a02eac`. The
worktree is ALIVE (dirty: `docs/ANALYSIS.md`, `docs/PORT_MAP.md`,
`docs/research/83-weather-reconcile.md` modified, mtime 2026-11-21
08:57:45 vs probe 09:05). The merger must read the LOCAL tip (or the
rescue push) or it silently re-merges the splitmix shape.

**Merger delta shrinks to the DROP only:** g11's deprecated
`main_dust.rs::snowing` + `SnowingEnv` + consts + the four snow witnesses
+ the `scene_scan` SnowingEnv block drop for d130's already-folded
`snow.rs` — no patch-hunks to apply, no witness migration owed (F7 is
done on d130). The `main_dust.rs` RETIRE-NOTE + fn doc are re-worded to
this receipt in the same commit.

Verification: code-only — `git show 099df89e2/16a792d56` full diffs read;
`snow.rs`/`scene_scan.rs`/`app/mod.rs` grepped at `16a792d56` (no splitmix
draw sites survive; `UnifiedRandom` witnesses present); `Dust.cs:419-441`
+ `Main.cs:66700` (init-only zero) read directly; d130 worktree
`git status --porcelain` + file mtimes probed read-only.
