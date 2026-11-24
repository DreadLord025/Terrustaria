# 79 — Doc-sweep lane 8: background/sky census & port (W4-g70, code-only)

Branch `task/W4-g70-bg-layers-1`, worktree `TR-W4-g70`, base `b86c8d69`
(the forest-sets fix round). **Code-only mode: no cargo/rustc/tests/graphify
invoked** — every claim is file-read evidence (C# `decompiled/Terraria/
Main.cs` / `Star.cs` / `Projectile.cs` / `WorldGen.cs` /
`GameContent/NextHorizonRenderer.cs` vs the Rust tree). C# reads kept to
≤100-line windows, direct paths only (the `decompiled/` junction is never
recursed). Witnesses WRITTEN NOT RUN, expectations hand-derived from the
cited C#.

Commits: phase 1 `6b3049e19` (SetBackColor chain), phase 2 `752655914`
(DrawSunAndMoon event bodies + flares), phase 3 `9d8478278` (live star pool
+ starfall chain), phase 4 (this doc + PORT_MAP/ANALYSIS/anchors).

Scope: the **surface background / sky** machinery — `Main.DrawSurfaceBG`'s
celestial half (`SetBackColor`, `DrawSunAndMoon`, `DrawStarsInBackground`/
`DrawStar`, `ApplyColorOfTheSkiesToTiles`, the atmo chain), `Star.cs` whole,
and the starfall projectile chain (`WorldGen.SpawnFallingObjects`' tail,
aiStyle 148, the natural star 12). The terrain layers (forest/dirt/rock/
magma) were the lane's earlier rounds; this sweep is the sky above them.

Status legend: **FAITHFUL** = 1:1 port present · **WRONG** = ported but
behaviour differs · **MISSING** = no port counterpart · **CLAIMED** =
another lane's branch owns it (not touched here).

---

## 0. Census table

| C# region | Cite | Verdict at base | Now |
| --- | --- | --- | --- |
| Ocean background (draw + offsets + setBG) | `Main.cs:60412-60520`, `WorldGen.cs` ocean setBG | **CLAIMED** by g71 (`8f38c508`, snow/desert/ocean) | skipped — g71 owns it |
| Desert background (draw + offsets + setBG) | `Main.cs:60268-60410` | **CLAIMED** by g71 | skipped |
| Snow background | `Main.cs` snow chain | **CLAIMED** by g71 | skipped |
| Underworld / cavern backdrops | `Main.cs:52844-53460` | **CLAIMED** by g72 (UNWIRED modules) | skipped |
| Clouds (`Cloud.cs`, `UpdateClouds`, `DrawClouds`) | `Cloud.cs` whole | **CLAIMED** by g69 (`CloudField`) | skipped |
| Sun/moon arc placement | `Main.cs:62848-62928` | FAITHFUL already (`tr-core/sky.rs::celestial_body`, verbatim) | kept |
| `SetBackColor` full modifier chain | `Main.cs:63452-63924` | **WRONG** — f32 approximations (`render/sky.rs::color_of_the_skies`/`tile_color`, `tr-core/sky.rs::sun_color`/`moon_color`) skipped the blood-moon double-write, the crimson G-read, the mushroom cross-reads, the eclipse step, the moon-phase floor | **ported** — `tr-core/src/sky_color.rs` byte-space chain (phase 1) |
| `ApplyColorOfTheSkiesToTiles` + atmo dim order | `Main.cs:61982-61983`, `:63175-63209` | **WRONG** — the tile colour took the atmo-dimmed sky | **ported** — integer 7/10 BEFORE the atmo dim; the sky colour dims after (phase 1) |
| `SmoothedMushroomLightInfluence` walk | `Main.cs:61945-61970` | **MISSING** | **ported** — `sky_color::smooth_mushroom_influence`, per-draw walk ahead of SetBackColor (phase 1) |
| Horizon blend (`ModifyHorizonLight` / `GetVisibilities`) | `Main.cs:63921`, `NextHorizonRenderer.cs:196-234` | **WRONG** — f32 space, no mushroom factor / eclipse kill | **ported** — byte space with the mushroom factor + eclipse kill (phase 1) |
| `DrawSunAndMoon` event bodies | `Main.cs:62930-62978` | **MISSING** — eclipse Sun3, Sunglasses Sun2/Extra_280, Smiley/Pumpkin/Snow moons | **ported** (phase 2) |
| `DrawSunAndMoon` num12/num13 colour chain | `Main.cs:62910-62962` | **WRONG** — `cloudBGAlpha` stood in for `cloudAlpha`, mushroom influence pinned | **ported** — byte-exact chain with the real `cloudAlpha` + `SmoothedMushroomLightInfluence` (phase 2) |
| `NextHorizonRenderer.DrawSun` flares | `NextHorizonRenderer.cs:168-181`, `:311-321` | **MISSING** | **ported** — 5 flare quads + `GetCelestialEffectPower` (phase 2) |
| `Star.cs` whole (spawn/update/NightSetup/StarFall) | `Star.cs:1-325` | **MISSING** — static `make_stars`/`StarSprite` stand-in, no state | **ported** — `tr-app/src/star_field.rs` live pool (phase 3) |
| `DrawStarsInBackground` / `DrawStar` | `Main.cs:63030-63173` | **WRONG** — static positions, no graveyard/shimmer/fadeIn/trail | **ported** — live pool draw (phase 3) |
| Starfall roll (`SpawnFallingObjects` tail) | `WorldGen.cs:71828-71856` (called `:71624`) | **MISSING** (PORT_MAP said "documented out") | **ported** — `world_sim/growth.rs::spawn_starfall` (phase 3) |
| aiStyle 148 spawner (720) | `Projectile.cs:53094-53128`, `:7617-7625` | **MISSING** | **ported** — `projectile_ai_misc::ai_star_spawner` + `star_spawner_convert`, dispatch arm 148 (phase 3) |
| Natural falling star (12) | `Projectile.cs:924-938`, `:23830-24065`, `:75681-75712` | **MISSING** | **ported** — row + `ai_natural_falling_star` + `natural_star_kill` (phase 3) |
| `NightSetup` dusk wiring | `Main.cs:65223` | **MISSING** | **ported** — `clock.rs` dusk arm, ahead of the event pair (phase 3) |
| Anniversary blood-moon flag | `Main.cs:65334-65337` | **WRONG** — `roll_blood_moon` got `false` | **fixed** — the TENTH_ANNIVERSARY seed flag now rides the call (phase 3) |

---

## 1. Phase 1 — the SetBackColor chain (`6b3049e19`)

`crates/tr-core/src/sky_color.rs` (new): `set_back_color` ports
`Main.SetBackColor` (`Main.cs:63452-63924`) in XNA **byte space** — every
`Color * float` multiplies all four channels and truncates per channel,
every `Color.Lerp` truncates. The chain, in order:

- the day/night ramps (the `num`/`num2`/`num3` time remaps),
- the **blood-moon night** — including the `:63513-63515` double-write
  (the R channel is written twice, the second write wins),
- the rain dim (`cloudAlpha`) and the graveyard dim,
- the corruption/crimson/jungle/mushroom influences — the crimson arm
  reads the **G** channel at `:63627` (not R), the mushroom moon arm
  cross-reads R→G/B at `:63773-63774` (kept verbatim),
- the **unconditional moon-phase floor** + the blood floor 25,
- the **eclipse light step** (`:63845-63898`): `time/1242` ramp, the
  0.01 decay, the `NotRetro` floors,
- the remix flatten, the lightning lerp, the white-moon reset
  (`:63917-63920`).

`apply_sky_to_tiles` ports `ApplyColorOfTheSkiesToTiles` (`:63175`): the
integer 7/10 blend. `atmo_dim_sky` ports the colour half of
`UpdateAtmosphereTransparencyToSkyColor` (`:63201-63209`): the SKY colour
fades with depth while the tile light keeps the undimmed bytes — the
`:61982-61983` order (tile colour first, atmo dim after) is load-bearing
and was wrong at base.

`smooth_mushroom_influence` ports `Main.SmoothedMushroomLightInfluence`
(`:61945-61970`): the per-draw ±0.01 walk toward the
`mushroom_tile_count / 160` target (the TileMax, NOT the zone threshold
140/100). `App.mushroom_light_influence` + `App.eclipse_light` carry the
state across draws.

The horizon blend moved to byte space (`render/sky.rs::modify_horizon_light`,
`NextHorizonRenderer.cs:196-234`): the sunrise/sunset gradient entries
with the mushroom factor (`:203`) and the eclipse kill (`:219-223`). The
`ShouldDrawSurfaceBackground` gate reads remix only — it does NOT read
`BackgroundEnabled`, so the tint survives the backgrounds-off toggle.

The DontStarve hooks (`DontStarveSeed.cs:9-59`) ride the chain behind the
world's seed flag.

Retired: `render/sky.rs::color_of_the_skies`/`tile_color` and
`tr-core/sky.rs::sun_color`/`moon_color` (the f32 approximations).
`draw.rs` + `scene_scan.rs` rewired onto the byte-space chain; the scene
scan reads the stored (last-scan) metrics exactly like `Main.Draw` reads
`Main.SceneMetrics` before the next scan.

16 witnesses WRITTEN NOT RUN (`sky_color.rs`), expectations hand-derived
from the cited lines.

## 2. Phase 2 — DrawSunAndMoon event bodies + flares (`752655914`)

`render/sky.rs`'s `DrawSunAndMoon` block (`Main.cs:62848-63028`):

- the **eclipse Sun3** swap (`:62930`),
- the **Sunglasses** head==12 Sun2 body + the Extra_280 overlay with the
  flag-gated `(B-60)*num12` alpha (`:62926-62953`) — the head slot reads
  `self.equip.armor.first()` → `registry::get(kind).head_slot` (the
  Sunglasses item 237 resolves to 12),
- the **Smiley** moon (drunk seed, rotation `num9/2 + π`, `:62964-62967`),
  the **PumpkinMoon** / **SnowMoon** phase-row draws (`:62968-62978`, the
  same 8-row slice as `Moon_0..8`),
- the remix no-sun gate (`:62910`),
- the byte-exact **num12/num13 colour chain** with the real mushroom
  influence + `cloudAlpha*atmo` (the base used `cloudBGAlpha` — wrong
  field), the white-moon reset threaded from SetBackColor's out-colour.

`NextHorizonRenderer.DrawSun`'s **five flare quads**
(`NextHorizonRenderer.cs:168-181`): flare1 ×3/×2 at sunset ×0.75/0.35,
flare2 ×2/×1.5/×1 at sunrise ×0.35/0.15/0.1, colour
`(255,255,255,0) × strength`. `GetCelestialEffectPower` (`:311-321`): the
remap in 0..3600, out over the last 1800 ticks of each half-day.

New sheets in `STATIC_SHEET_NAMES`: Sun2, Sun3, Extra_280, Moon_Smiley,
Moon_Pumpkin, Moon_Snow, Sunflare/flare1, Sunflare/flare2.

3 witnesses WRITTEN NOT RUN.

## 3. Phase 3 — the live star pool + starfall chain (`9d8478278`)

`crates/tr-app/src/star_field.rs` (new): the whole `Star.cs` (325 lines).

- `spawn_stars` (`Star.cs:86-158`): the 200..400 count roll, the
  10-candidate farthest-from-neighbours X pick for a single-slot respawn
  (the `num5 = -2000` start makes the first candidate always win slot 0),
  the attribute recipe in its exact RNG order (Y, rotation, scale, tex,
  twinkle, twinkleSpeed ±, rotationSpeed ±, the 1/40 ×2-scale quirk).
  The C# installs the fresh default Star into the slot FIRST so the
  neighbour scan sees a (0,0) star at slot i — the port mirrors the
  install order.
- `update_star` / `update_stars` (`:171-317`): the falling branch
  integrates `fallSpeed * (dayRate+99)/100`, retires below y 1500, runs
  the twinkle at 3× speed and the rotation at a flat 0.5/tick; the idle
  branch decays `fadeIn` at `6.1728395E-05 * dayRate * 10`, oscillates
  the twinkle (0.6..1 rails, speed sign-flip) and the rotation (6.28
  wrap). The **`dayCheck` never-true quirk** is kept verbatim: the
  decompile's only writes are the two `false` resets (`:245`/`:321`), so
  every day tick past 27000 respawns the hidden slots.
- `night_setup` (`:41-60`): the 1-in-10 → 3..5× boost window, else
  1-in-3 → 1..1.5×; the anniversary narrows to 1-in-5 / 1-in-2. Wired
  into `UpdateTime_StartNight` (`Main.cs:65223`) on the UpdateTime
  stream, ahead of the BirthdayParty/LanternNight event pair.
- `star_fall` (`:62-84`): the nearest live non-falling star to the
  spawner's world X (mapped into the 1920 design space through
  `rightWorld`) starts falling; `Fall`'s two rolls (Y 7..10 first, then
  X −4..4) ride the caller's stage stream. The secret-seed gate
  (`:162`) skips the Fall body but still bumps the count.

The draw (`render/sky.rs`, `Main.cs:63030-63173`): the graveyard opacity
(`1 − graveyard*1.4`), the shimmer dim/recolour (one channel per `i%3`
class scales by twinkle², lerped by the shimmer alpha, then ×`1−alpha`),
the fadeIn fade, the `(255−skyCh−100)×twinkle×(1−fadeIn)` brightness
averaged ×1.4 capped 255, the non-falling-only wrap, the falling trail
(j=1..min(fallTime,30), offset `fallSpeed×j×0.4`, fade `1−j/30`), the
`bgTopY=0` pin when `worldSurface ≤ 30`. The `shimmerAlpha == 1` gate
skips the natural pass (`:62086`).

The starfall chain, end to end:

- `world_sim/growth.rs::spawn_starfall` — `WorldGen.SpawnFallingObjects`'
  tail (`WorldGen.cs:71828-71856`, called `:71624`): the night-only
  `Next(8000) < 10 * maxTilesX/4200 * starfallBoost` roll per dayRate
  iteration on the UpdateWorldGen stream, the spawn position (X in
  100..width−50+100 tiles, Y in the top 5%), the `RollLuck(15)` target
  scan (the remix world rolls BAD luck), the ±Next(1,640) X jitter, the
  solid-collision gate, the velocity normalised to speed 12.
- `projectile_ai_misc::ai_star_spawner` + `star_spawner_convert`
  (`Projectile.cs:53094-53128`): the day kill (remix keeps it), the
  `ai[0] += dayRate` age, the one-shot `localAI[0]` latch that requests
  `Star.StarFall(position.X)` while the local player is above the
  surface, the owner + 180-tick conversion gate, the homing nudge
  (X ×0.35 toward the locked player's centre). The dispatch arm 148
  spawns the 12 (damage 1000, kb 10) and kills the spawner.
- The natural star 12: the `SetDefaults` row (`:924-938` — 18×18,
  aiStyle 5, penetrate −1, alpha 50, light 1, trap), the aiStyle-5
  generic slice (`:23830-24065` — the day kill at damage 1000, the
  generic ai[1] latch, the alpha bounce, the spin, the light 0.9, the
  1/20 dust gate + the anniversary 1/15 gate, the screen-gated gore
  draws), the `Kill()` burst (`:75681-75712` — the 8-star cornflower
  ring + the 4-star gold ring, one NextFloat each, then the screen-gated
  7-gore burst: NextVector2CircularEdge is ONE NextFloat, SelectRandom
  is Next(8), NewGore's Y kick Next(10,31) + X kick Next(−20,21)).
- The pool's `pending_starfall` / `view_rect` channels carry the requests
  (the golf-context precedent): `world_sim/projectiles.rs` drains the
  StarFall request after the AI walk (the Fall's rolls ride the same
  UpdateProjectiles stream) and sets the view rect for the gore gates.
- `world_sim/weather.rs` ticks `Star.UpdateStars` once per tick in the
  (2c) slot (`Main.cs:17249-17254`), gated on the view being above the
  surface line, ahead of `Cloud.UpdateClouds`.

The `UNARMED_ROWED` census pin passes: 148 was never in the unarmed list
(the rows are new), and the dispatch has a single 148 arm under
`deny(unreachable_patterns)`.

8 witnesses in `star_field.rs` + 4 in `projectile_ai_misc.rs` + the
rewired spawn-recipe witness in `render/sky.rs`, all WRITTEN NOT RUN.

---

## 4. Deviations (recorded in ANALYSIS.md)

1. **Star spawn RNG pin.** The C# `SpawnStars` seeds a fresh OS-random
   `FastRandom` per call (`Star.cs:88`, `CreateWithRandomSeed`); the port
   pins each call's stream to `world.seed ^ counter × 0xA076_1D64_78BD_642F`
   so screenshots stay deterministic (the make_stars deviation, kept).
2. **Smiley moon gate.** The C# reads `WorldGen.drunkWorldGen`, which is
   gen-time-only (set `WorldGen.cs:11115`, cleared in the finally
   `:11097`) — in-game it is always false, so the Smiley moon never
   draws in vanilla after generation. The port reads the in-game DRUNK
   seed flag instead, so a drunk-seed world actually shows the Smiley
   moon. Deliberate divergence: the port's seed flags are the persistent
   in-game representation of the gen-time flags.
3. **AFK gate always open.** The starfall target scan's
   `afkCounter < 10800` gate (`Player.cs:57103`) has no port AFK counter,
   so the gate is always open.
4. **Flare blend.** The DrawSun flares draw at colour
   `(255,255,255,0) × strength` — alpha 0. Under the port's premultiplied
   blend an alpha-0 tint leaves the destination alpha untouched while
   the tinted rgb ADDS, which matches vanilla's additive glow.
5. **Star draw alpha.** The decompile builds the star colour from
   `default` and assigns only R/G/B (`Main.cs:63083-63086`) — a zero
   alpha would make every star invisible under the port's blend, so the
   port draws them at the computed brightness with the byte alpha the
   `color *=` chain produces.
6. **Sound/dust/gore bodies skipped.** The starfall chain's dust/gore
   BODIES are the pool-wide skip (the draws stay on the stream); the
   `SoundID.Item10` cue is the sound skip. The gore types 16/17 carry no
   SpecialAI (`GoreID.cs:20`), so no further draws.
7. **The menu gravity block.** `Star.UpdateStars`' secret-seed menu
   gravity block (`Star.cs:261-316`) needs `gameMenu && generatingWorld`
   — unreachable in the port's frame loop, documented out.

## 5. Remainders (not ported, with cites)

- The **artificial shimmer star pass** (`Main.cs:62203`): the shimmer
  overlay's own star draw at `shimmerAlpha == 1`. The natural pass skips
  at that alpha; the artificial pass has no port caller yet.
- The **starGame minigame** (`Star.cs`'s `starGame` flag + the menu
  gravity block): the secret-seed star-catching minigame. The port has
  no starGame, so `StarFall`'s Fall body skips on any secret seed.
- The **AFK counter** (`Player.cs:57103`): see deviation 3.
- The **menu flat-35 night** (`Main.cs`'s menu sky colour): the main
  menu's flat night sky is a separate draw path the port's menu does
  not run.
- The **AuroraSky tint** (`AuroraSky.cs`): the aurora borealis sky
  overlay — a filter-family remainder (the ANALYSIS "animated screen
  effects" block).
- The **Smiley gen-time nuance**: see deviation 2 — the port's in-game
  flag is a deliberate divergence, not a remainder.

## 6. Verification

Code-only lane: no cargo/tests/graphify runs. The witnesses are WRITTEN
NOT RUN with hand-derived expectations (the cited C# lines). The
consistency sweep verified: the `draw_sky` signature ↔ call site (38
params), the `back.sun`/`back.moon` out-colours threaded from
SetBackColor, the `sky_tint` derivation (atmo-dimmed bytes / 255), the
`pending_starfall`/`view_rect` pool channels + the `Default` impl, the
`proj_data` rows' tuple shape + the `deny(unreachable_patterns)` single
148 arm, the `UNARMED_ROWED` pin (148 not in the list), the
`spawn_with_data_ai`/`spawn_with_data_ai_owned` signatures, the
`UnifiedRandom::new(i32)` / `FastRandom::new(u64)` seed types, the
`Player::solid_collision`/`luck`/`center` signatures, the
`roll_luck`/`roll_bad_luck` `DropRng` bound (the `UnifiedRandom` impl),
the `head_slot: i16` field type, the `SkyAssets.stars` field + the
`push_screen_rot` signature, the `App.stars` type swap (mod.rs + the
flow.rs test fixture), the `quantised`/`is_day`/`surface_px` bindings
ahead of their uses, and the atmo formula alignment
(`camera.y − screen.y/(2·world_zoom) + screen.y/2`, the `:61983` input)
in draw.rs + scene_scan.rs.