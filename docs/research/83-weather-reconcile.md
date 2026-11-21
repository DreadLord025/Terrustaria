# 83 — Weather reconcile (W4-d130 merger assist + conflict resolution)

**STATUS: EXECUTED + FINAL PHASE LANDED** (union-exec commit `88cfbf38`;
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
