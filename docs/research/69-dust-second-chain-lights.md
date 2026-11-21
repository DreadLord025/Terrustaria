# 69 — The 52 orphaned second-chain dust lights (W4-g13, flag-a closure): census, collector port, wiring, the d50 strip-contract (flag-b) and the decrement retire (flag-c)

Status: DONE + FINAL (W4-g13, code-only — `dust_lights_b.rs` ported +
WIRED in `app/frame/draw.rs`; 23 witnesses WRITTEN NOT RUN; no cargo/
rustc/rustfmt/tests/graphify invoked). Base `5ea44967` (research/68
sweep tip). Sibling probes (read-only): d50 `2e243a2d` (`dust_physics.rs`,
2154 ln — re-verified UNCHANGED at the final phase, §6 receipt), c19
`4eedcb32` (`proj_lights_a.rs`), e6 `c8b072b6` (`proj_ai_lights.rs`).

research/68 §3 flag-a found the 52 second-chain `AddLight` arms
(`Dust.cs:1061-2312`) ORPHANED: `dust_lights.rs:41-46` deferred them to
"S1-E-3's chain port" while d50's `dust_physics` skipped them ALL as
"S1-E-5 (`dust_lights`)". This lane closes the circular deferral: it OWNS
the 52, ports them as a dust-light collector (the e6/c19 `EntityLights`
pattern), and wires the feed post-move.

## 1. The census — all 52 arms (type / colour / intensity, C# exact)

`Dust.UpdateDust` carries 94 `Lighting.AddLight` sites: 42 in the FIRST
ladder (≤`:1046`, ported in `dust_lights.rs`) and 52 from `:1061` to
`:2312`. Every arm anchors `(int)(position.X/16f)`, `(int)(position.Y/16f)`
(the `XAnchor::Center` zero-box form) EXCEPT the 35 noGravity arm (`:1896`,
`y+1`). "num" is the arm's intensity scalar; the colour triples multiply it
per channel. Gates are the C#'s verbatim.

| # | C# | type | num (intensity) | colour × num | gate |
| - | -- | ---- | --------------- | ------------ | ---- |
| 1 | `:1061` | 159 | scale*1.3 cap 1 | (1, 1, 0.1) | — |
| 2 | `:1099` | 164 | scale cap 1 | (1, 0.1, 0.8) | — |
| 3 | `:1137` | 173 | scale cap 1 | (0.4, 0.1, 1) | — |
| 4 | `:1208` | 162 | scale*1.3 cap 1 | (1, 0.7, 0.1) | — |
| 5 | `:1212` | 160 | scale*1.3 cap 1 | (0.1, 1, 1) | — |
| 6 | `:1241` | 168 | scale*0.8 cap **0.55** | (1, 0, 0.8) | — |
| 7 | `:1399` | 244-247 | ((b/270)+1)/2 * scale*0.9 (b = ambient mean byte) | per-type tint × num47 (below) | `!noLightEmittance` |
| 8 | `:1441` | 43 | GetColor/270 * scale*1.07 * color/255 | (1, 1, 1) per channel | dark-kill else `!noLightEmittance` |
| 9 | `:1470` | 15 | scale cap 1 | (0.45, 0.55, 1) | `!noLightEmittance` |
| 10 | `:1474` | 57 | scale*0.8 cap 1 | (0.95, 0.95, 0.45) | `!noLightEmittance` |
| 11 | `:1478` | 58 | scale*0.8 cap 1 | (1, 0.55, 0.75) | `!noLightEmittance` |
| 12 | `:1501` | 110 | scale*0.1 cap 1 | (0.2, 1, 0.5) | — |
| 13 | `:1510` | 111 | scale*0.125 cap 1 | (0.2, 0.7, 1) | — |
| 14 | `:1519` | 112 | scale*0.1 cap 1 | (0.8, 0.2, 0.8) | — |
| 15 | `:1528` | 113 | scale*0.1 cap 1 | (0.2, 0.3, 1.3) | — |
| 16 | `:1537` | 114 | scale*0.1 cap 1 | (1.2, 0.5, 0.4) | — |
| 17 | `:1546` | 311 | scale*0.1 cap 1 | torch 16 (Desert) | — |
| 18 | `:1555` | 312 | scale*0.1 cap 1 | torch 9 (Ice) | — |
| 19 | `:1564` | 313 | scale*0.25 cap 1 | (1, 0.8, 0.6) | — |
| 20 | `:1588` | 66 | (scale+0.02)*0.8 cap 1 | color/255 | — |
| 21 | `:1619` | 267 | (scale+0.02)*0.8 cap 1 | color/255 | `!noLightEmittance` (noLight lifted `:1613`) |
| 22 | `:1635` | 21 | (scale+0.005)*0.4 **UNCAPPED** | (0.8, 0.3, 1) | `!noLightEmittance` (else → 20 arm) |
| 23 | `:1640` | 231 | (scale+0.005)*0.4 **UNCAPPED** | (1, 0.5, 0.3) | — |
| 24 | `:1644` | 20 | (scale+0.005)*0.8 cap 1 | (0.3, 0.6, 1) | — |
| 25 | `:1692` | 27/45 | post-burn scale (×0.1 if noLight) cap 1 | (0.6, 0.2, 1) | — |
| 26 | `:1706` | 55 | scale*0.8 cap 1 | (1, 1, 0.6) | `!noLightEmittance` |
| 27 | `:1714` | 73 | scale*0.8 cap 1 | (1, 0.35, 0.5) | `!noLightEmittance` |
| 28 | `:1722` | 74 | scale*0.8 cap 1 | (0.35, 1, 0.5) | `!noLightEmittance` |
| 29 | `:1731` | 56 | scale*1.2 cap 1 | (0.35, 0.5, 1) | `!noLightEmittance` |
| 30 | `:1743` | 71/72 | scale cap 1 | (0.2, 0, 0.1) | — |
| 31 | `:1896` | 35 ng | (scale+0.03)*0.6 cap 1 | (1, 0.3, 0.1) | `noGravity`, **y+1 tile** |
| 32 | `:1926` | 67/92 | scale cap 1 (×0.1 if noLight) | (0, 0.8, 1) | — |
| 33 | `:1939` | 185 | scale cap 1 (×0.1 if noLight) | (0.1, 0.7, 1) | — |
| 34 | `:1950` | 107 | scale*0.5 cap 1 | (0.1, 1, 0.4) | `!noLightEmittance` |
| 35 | `:1994` | 35 wet | post-wet scale*0.3+0.4 cap 1 | (1, 0.5, 0.3) | gravity 35 |
| 36 | `:2004` | 68 | scale*0.3 cap 1 | (0.1, 0.2, 1) | — |
| 37 | `:2013` | 70 | scale*0.3 cap 1 | (0.5, 0, 1) | — |
| 38 | `:2041` | 41 | (scale+0.007)*0.7 cap 1 | (0.4, 0.9, 1) | — |
| 39 | `:2069` | 44 | (scale+0.0085)*0.7 cap 1 | (0.7, 1, 0.8) | — |
| 40 | `:2139` | 130 | post-ramp scale cap 1 | (1, 0.5, 0.4) | — |
| 41 | `:2143` | 131 | post-ramp scale cap 1 | (0.4, 1, 0.6) | — |
| 42 | `:2147` | 132 | post-ramp scale cap 1 | (0.3, 0.5, 1) | — |
| 43 | `:2151` | 133 | post-ramp scale cap 1 | (0.9, 0.9, 0.3) | — |
| 44 | `:2182` | 278 | post-ramp scale cap 1 | color/255 (Vector2 overload) | `!noLight && !noLightEmittance` |
| 45 | `:2214` | 219 | post-ramp scale cap 1 | (1, 0.5, 0.4) | `!noLight` |
| 46 | `:2218` | 220 | post-ramp scale cap 1 | (0.4, 1, 0.6) | `!noLight` |
| 47 | `:2222` | 221 | post-ramp scale cap 1 | (0.3, 0.5, 1) | `!noLight` |
| 48 | `:2226` | 222 | post-ramp scale cap 1 | (0.9, 0.9, 0.3) | `!noLight` |
| 49 | `:2254` | 226 | post-ramp scale cap 1 | (0.2, 0.7, 1) | `!noLight` |
| 50 | `:2275` | 226 | post-ramp scale cap 1 | customData Color/255 | `customData is Color` — **ABSENT** (field dropped) |
| 51 | `:2299` | 331 | post-ramp scale cap 1 | color/255 | `!noLightEmittance` |
| 52 | `:2312` | 272 | post-ramp scale cap 1 | (0.5, 0.2, 0.8) | `!noLight` |

**244-247 per-type tints** (`:1366-1393`): 244 (226/255, 118/255, 76/255)
×0.9; 245 (131/255, 172/255, 173/255) ×1.0; 246 (0.8, 181/255, 72/255)
×1.1; 247 (0.6, 172/255, 185/255) ×1.2. The ambient `b` is
`(GetColor.R + GetColor.G + GetColor.B) / 3` as a byte (`:1338`).

**Dark sharers** (share an arm but have NO AddLight branch): 274/292 (the
15/57/58 arm, `:1468-1479` covers 15/57/58 only), 134 (the 130-133 arm),
223 (the 219-222 arm). **Site 50** (226's customData Color second light) is
absent wholesale — `customData` is dropped (`particles.rs`'s `Dust` record).

**Post-mutation reads** (the C# computes the colour AFTER a scale/position
mutation in the same arm): 66/267 (+0.02 grow), 20/21/231 (+0.005), 41
(+0.007), 44 (+0.0085), 27/45 (the burn-down + noLight double −0.06 + the
player push), 35 ng (+0.03), 35 wet (the wet-arm shrink/kill), and the
whole second chain 130-134/278/219-223/226/331/272 (the fadeIn ramp
`:2083-2129` precedes the chain). The collector replays these on a local
copy (§2).

## 2. The port — `dust_lights_b.rs` (the e6/c19 EntityLights pattern)

`crates/tr-app/src/dust_lights_b.rs`:
- `dust_chain_light(d, p, env) -> Option<(i32, i32, [f32; 3])>` — the pure
  per-slot collector (the c19 `proj_lights_a` / e6 `proj_ai_lights` shape).
  No RNG parameter exists, so the feed cannot move any stream.
- `light_scale` / `light_position` — replay the owning arm's pre-light
  mutations from the POST-move record (the `dust_lights.rs` 206/174
  compensation precedent, extended to the whole chain). Each cites its d50
  `type_chain` arm. `arm_27_45_scale` (the burn-down ladder + noLight
  shrink), `arm_35_wet_scale` (the wet probe + the draw-independent alpha
  kill), `ramp_scale` (the generic fadeIn +0.03 / burn-out −0.01).
- `ChainLightEnv` — the d50 `DustPhysicsEnv` probe shape narrowed to the
  LIGHT column: `player_wet`/`player_velocity` (the 27/45 push),
  `light_color` (the 244-247/43 ambient sample, bound to
  `main_dust::light_color_bytes` over the light map + `style.brightness`),
  `wet_collision` (the 35 wet probe, bound to `player::wet_collision`).
- `feed_dust_chain_lights(pool, p, env, lights)` — walks `Main.dust` slot
  order (`Dust.cs:431`), appends to the shared `EntityLights` pair.

**Two fidelity subtleties pinned by witnesses:**
1. **Type 21 with `noLightEmittance` does NOT go dark** — the C# else-if
   `if (type==21 && !noLightEmittance)` / `else if (type==231)` / `else`
   (`:1632-1644`) falls through to the ELSE arm (the 20 colour, scale*0.8
   cap 1). The collector mirrors the fall-through.
2. **The 35 alpha kill is draw-dependent at `alpha == 255`** — `alpha +=
   Next(2)` then `if (alpha > 255) scale = 0` (`:1961-1965`). For
   `alpha >= 256` it fires on every draw, for `alpha <= 254` never; only
   `alpha == 255` splits on the `Next(2)` (d50's physics draw). The pure
   collector mirrors the draw-independent kill (`alpha > 255`) and leaves
   the boundary to the physics wiring (§4).

## 3. The wiring — post-move, after the FIRST-ladder feed

`app/frame/draw.rs::draw_region`: the chain feed runs immediately after
`feed_dust_lights` (the FIRST ladder), in the same `Main.dust` slot order,
reading the POST-move pool (d50 owns the move — `update_dust`'s `:477`
`position += velocity`). It is placed AFTER the `style.brightness`
(Darkness) block so the 244-247/43 ambient sample reads the adjusted
GlobalBrightness. The `ChainLightEnv` probes bind the existing seams
(`main_dust::light_color_bytes`, `player::wet_collision`, the player's
`vx`/`vy`/`wet`). No arm draws `Main.rand`, so the placement leaves the
fixed-step order and every stream exact (the `dust_lights_b.rs` RNG witness
pins it). The feed is OUTSIDE the G6-cached world pass (the entity-light
precedent — it mutates `entity_lights`, not the pool).

## 4. flag-b — the d50 pre_move strip-contract (lava_bubbles double-count)

research/68 §3 flag-b: d50's `pre_move` (`dust_physics.rs:162-173`)
increments `lava_bubbles` for type 35 (`:170-172`), AND g13's landed inline
increment (`particles.rs:359-361`, witnessed
`w4_g13_lava_bubbles_counts_the_live_type35_population`) does the same
inside `update_dust`. d50's ANALYSIS wiring note ("the generic loop calls
pre_move ... then update_dust") would count every 35 TWICE per pass,
halving the `< 200` bubble budget (`Main.cs:57785` / `LiquidRenderer.cs:556`).

**The strip-contract (g13's inline increment is CANONICAL):**
- **KEEP** the inline increment in `DustPool::update_dust`
  (`particles.rs:359-361`) — it is landed + witnessed, sits at the C#'s
  exact `:473-476` position (before the `:477` move), and resets at the
  head (`:427`, `particles.rs:319`).
- **STRIP** the type-35 arm from d50's `pre_move` (`dust_physics.rs:170-172`,
  the `if d.dust_type == 35 { *lava_bubbles += 1; }` block). `pre_move`
  KEEPS the 299/300/301/305 shrink/lift (`:163-168`, `scale *= 0.96`,
  `velocity.y -= 0.01`) — the skeleton LACKS it and the wiring must add it.
- After the strip, `pre_move`'s `lava_bubbles: &mut i32` parameter is
  unused; d50's wiring wave may drop it (the signature is d50's to change).
  The contract is behavioural: exactly ONE increment per live 35 per pass,
  at the inline site.

This is a CONTRACT for d50's wiring wave (g13 does not edit d50's staged
`dust_physics.rs` — it is UNWIRED on main, d50's lane owns the strip). The
g13 inline increment stays untouched.

## 5. flag-c — the dust_lights local-decrement retire (206/174/230 + 309/261)

research/68 §3 flag-c: `dust_lights.rs`'s FIRST-ladder feed compensates for
the per-type scale mutations it does not run — 206 (`scale -= 0.1` then
light, `:766-768`), 174 (`scale -= 0.01` then light, `:807-813`), 230
(light reads PRE-increment scale, `:632-640`), 309 (`scale *= 0.97`,
`:486`), 261 (the noGravity drift + shrink, `:695-704`). d50's `type_chain`
runs ALL these mutations (`dust_physics.rs:192-237`, `:259-275`).

**The retire note:** once d50's `type_chain` is wired into the `type_arm`
hook, the pool record carries the POST-mutation scale, so the feed's
compensating local decrements DOUBLE-COUNT. Retire them in the same wiring
wave:
- 206: `dust_lights.rs:381` `(d.scale - 0.1) * 0.4` → `d.scale * 0.4`.
- 174: `dust_lights.rs:419` `cap(d.scale - 0.01, 0.6)` → `cap(d.scale, 0.6)`.
- 230: `dust_lights.rs:309` reads `d.scale` (PRE-increment) — after wiring
  the record is POST-`+0.02`, so recover `d.scale - 0.02` (or read the
  pre-hook scale). d50's `:208-213` cites this exact recovery.
- 309/261: the post-mutation reads d50's ANALYSIS flags — reconcile at wiring.

**The same retire applies to THIS module's replays** (`light_scale` /
`light_position` / `arm_27_45_scale` / `arm_35_wet_scale` / `ramp_scale`):
each replays a d50 `type_chain` mutation on a local copy. When d50 wires,
the record carries the post-mutation value and the replay retires arm-by-arm
(the cited d50 arm is the retire key). The 35 `alpha == 255` boundary (§2)
resolves then too — d50's chain writes the post-kill scale.

## 6. FINAL-phase receipt — the d50 strip-contract re-verified UNCHANGED

Re-probe at the final phase (this branch `441f6120` base): origin
`task/W4-d50-dust-type-table` tip is STILL `2e243a2d` (phase-2 docs,
2026-11-20 21:10 +0200 — the same tip §1-5 probed; the d50 session has
pushed NOTHING since). `git show` of its `dust_physics.rs` re-read:

- `pub fn pre_move(d: &mut Dust, lava_bubbles: &mut i32)` at `:162` —
  the type-35 arm `if d.dust_type == 35 { *lava_bubbles += 1; }` is
  STILL PRESENT at `:170-172`. **No strip landed.**
- The 299/300/301/305 shrink/lift (`:163-168`) is present, as §4 requires
  it to stay.
- d50's own integration witness helper `tick_physics` (`:1999-2020`)
  drives `pre_move` per-slot with `pool.lava_bubbles` — i.e. the d50
  lane's own test harness DOUBLE-COUNTS against the skeleton's landed
  inline increment (`particles.rs:359-361`, re-read present at this
  phase) exactly as §4 predicts.

**Receipt verdict: research/69 §4 STANDS for the merger, verbatim.**
KEEP the inline `update_dust` increment (canonical, landed + witnessed);
STRIP the `pre_move` 35 arm at the d50 wiring wave (the 299-305
shrink/lift stays and must be wired — the skeleton lacks it); the
`lava_bubbles: &mut i32` parameter then goes unused and is d50's to
drop. §5's decrement retire (206/174/230 + 309/261, and this module's
replays arm-by-arm) is likewise UNCHANGED — `dust_lights.rs` on main
still carries the compensating decrements (`:381` re-read
`(d.scale - 0.1) * 0.4`), and `dust_physics` is still UNWIRED on main
(no `dust_physics` reference in any live crate file — only this lane's
doc comments cite it).

## Delivered this lane

- `crates/tr-app/src/dust_lights_b.rs` — the 52-arm collector + feed +
  23 per-arm colour-pin witnesses (WRITTEN NOT RUN).
- `app/frame/draw.rs` — the post-move chain feed (after the FIRST-ladder
  feed, after the brightness block).
- `main.rs` — `mod dust_lights_b;`.
- This doc + the ANALYSIS section + the PORT_MAP/ROADMAP/anchors rows.
- d50's `dust_physics.rs` is a READ-ONLY probe (the strip-contract §4 and
  the retire note §5 are its wiring wave's work, not edited here).
