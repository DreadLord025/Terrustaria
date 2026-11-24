# 328 — W4-g68 WIND SEAL: the wind-flag owner's close + the weather-family mutual receipts

Status: **SEALED** (2026-11-21, code-only lane; witnesses WRITTEN NOT RUN).
Branch: `task/W4-g68-weather-wind-leaves`. Registry number **328** (FINAL,
registry §9.2 UNIFIED FINAL, d54 tip `b243c7086` — renumbered from the
provisional **247** = max-on-origin **246** + 1 at seal time; **247 = d10's
reservation WINS, EXECUTED** (`72431e61f`), the first-pushed-wins provisional
loses, the g67 134→146 precedent `c38d42d32` applied).

Authority consumed: **d130 research/83 §9.4 WEATHER-FAMILY bible FINAL**
(`b8f655708912431a84da0c90b77fc5d69b1a8303`) — "**g68** owns the wind sim +
leaf cadence — its LIVE `windSpeedCurrent` feeds d130's
`GetRainFallVelocity` (`wind*18,14`), the Eyebrella drip (`wind*3,9`) and
c40's severity, ONE wind-flag owner with read-only consumers". This file
is the g68-side mutual receipt of that declaration plus the g67/c40
receipts and the wind-family bible.

## 1. The git-log census — what landed on g68 (wind-flag owner + leaf cadence)

17 commits `a725acb04..2075c3aaa` (base = the wave-3 land): the 15
pre-seal commits (`a725acb04..d247a1c9f`) touched 13 files, +2052/−355,
then the two final-phase seal commits below. The lane NEVER
edited `tr-core/src/wind.rs` or `tr-app/src/player.rs` in its committed
history (both byte-identical to base at `d247a1c9f` — verified by
`git diff --stat a725acb04 d247a1c9f -- <file>` empty): the wind SIM
(`Wind::update_rain`, the target walk, the gust grid S1-G-8,
`leaf_frequency`) is base-trunk code the lane VERIFIED and CONSUMES; the
lane's own work is the leaf-cadence port + the wiring wave.

| commit | phase | what landed |
| --- | --- | --- |
| `0f24c565e` | phase 1 | the d60-flagged 192-leaf deviation reconciled; every leaf spawn routed through `GorePool` (the private 400-cap `LeafGore` Vec + draw-time `tick_leaves` retired — `Gore_UpdateLeaf` in the gores phase owns physics, `Main.DrawGore` owns the draw); `EmitTreeLeaves` hallowed height pick + `NewGoreDirect` argument rolls onto `Main.rand` (two-stream census); 8 witnesses |
| `791a240fd` | phase 2+3 | 8 `Gore_UpdateLeaf` physics pins (π/180 sway + wind*4 surface term, underground drop, collision rest, water ×0.94, honey freeze, lava burn-up, open-ground resume, 600+255 fade) + the `sand.rs` desert-debris wind-coupling bound (num3 sign*Lerp(0.9,1,\|wind\|), num13*20 push, dead-calm refusal) |
| `204457142` | docs | ANALYSIS "The wind leaf particles" + PORT_MAP + anchors |
| `fd9415f3f` | self-review | liquid=100 fixtures, alpha-21 resume pin, 200k gate-hit seed search |
| `2a0c48559` | deferred-1 | the lava leaf five-dust puff WIRED (`GorePool::update` DustPool+SpawnCtx seam, `DustFromGore` adapter on the UpdateGores stream, `Gore.cs:1119-1122`) |
| `76b418ac3` | deferred-3 | the Width/Height asset seam (`set_texture_sizes` from `Lookups.gore_sheets`, `SpriteFrame::source_size`, the sheet-half fall probe `:1180`) |
| `07396db1c` | deferred-4 | `EmitLiquidDrops` ported UNWIRED (`TileDrawing.cs:7594-7674`, the one unclaimed body — `:7779-7891` is exactly `EmitTreeLeaves`, petals d60's, vision dust d56's) |
| `373645c3e` | deferred docs | the G6 emission-cadence contract for the d60 wiring wave (three C# call sites, DrawnThisUpdate vs `!game_paused`) |
| `7305944bd` | self-review | the `gore_texture_sizes` snapshot-before-move fix + the 1x1-frame fall probe |
| `882229a21` | self-review | the probe re-derivation (pos.x 170.1, the −sin(π/180) drift) |
| `b20480d72` | self-review | the explicit `DustFromGore(&mut *rng)` reborrow |
| `ea9e784f6` | wiring-1 | `EmitLiquidDrops` WIRED at the live per-frame draw seam (`emit_liquid_drops_area` beside tile_cracks, the persistent `App.tile_draw_rand` gate stream — the `TileDrawing._rand` `:104/:191` deviation class); the g30 `liquid_drop_cells` attach-contract documented |
| `5504acc5d` | wiring-2 | the spawn-position-adjust arms behind the loaded-texture gate (1225 / 411-430 / SpecialAI-2 825-827, the exact 0/4(+1)/1 stream deltas, the `Next(0,0)` UnifiedRandom twin) |
| `3ccd49e60` | wiring-3 | **the leaf-cadence verify**: `leaf_frequency` matches `UpdateLeafFrequency` `:7719-7771` band-for-band incl the ×7; the hallowed /2 → height-10000 → branch ×3 order matches `:7816-7833`; `WindForVisuals == windSpeedCurrent` (`Main.cs:2823`); the FIX — the inline 192/384 arm gains site-1's throwaway quarter-roll on the `flame::tile_frame_seed_at` 5-draw-cycle schedule (the port shed 4× too often; the throwaway roll never steps `leaf_rng`) |
| `d247a1c9f` | wiring-4 | the d60 G6-contract cross-note (tip `77429fe1`, 2086 ln UNWIRED): site-1 FOLLOWS, site-2 SILENT-legitimate (g31 `can_do_dust` already-consumed), site-3 FOLLOWS (dedupe at merge), the per-frame move SILENT-deferred; ONE factual flag — d60's `:693` cite sits in `TileDrawing.Draw` (`:465`), not `CacheSpecialDraws` |
| `32ee28ca5` | **final-phase (this seal)** | the wind-flag owner's three missing arms in `wind.rs` (see §2) + the player.rs push-copy retirement (see §5) |
| `2075c3aaa` | **final-phase (this seal)** | the kite-drop feed — `spawn_loot`'s live `LootFlags.windy` reads `app.wind.windy_enough_for_kite_drops()` (was `Default::default()` false; the 8 kite rows `loot_db_gen.rs` now gate on the live target) |

**Witness census: 31 WRITTEN NOT RUN** (grep-verified `fn w4_g68_*` at
the seal tip; CORRECTED from the seal's own 33 — the per-file rows
below always summed to 31, the 33 was an arithmetic slip) —
`render/foliage.rs` 12, `tr-core/gore.rs` 14, `render/tiles.rs` 1 (the
site-1 quarter-roll gate order), `sand.rs` 1, `tr-core/wind.rs` 3
(this seal). The wiring-4 commit message said 28; the file-level
census finds 28 pre-seal (the wiring-2 A/B witnesses counted as 3 in
the message but landed as the per-arm pins) — the grep count is
authoritative. No cargo/rustc invoked on this lane (code-only).

## 2. The final-phase missing arms (g68-unique, `32ee28ca5` + `2075c3aaa`)

The in-flight dirty state found at resume was g68's OWN final-phase work
(mtimes ~4.5 h stale, no live cargo/rustc process, every hunk cites
W4-g68 — the co-lane probe verdict, §6). Adjudicated hunk-by-hunk:

**KEPT + LANDED (`wind.rs`, g68-unique, hunk-disjoint from d130's
`lanterns_up` graft — §3):**
1. **The remix branch of `UpdateWindyDayState`** (`Main.cs:13021-13027`,
   re-read verbatim): a remix world clears the STORM flag and leaves
   windy-day alone; non-remix clears windy-day. The old port
   unconditionally cleared windy-day (the remix arm was a comment).
   `remix_world` rides the seed flag (`app/mod.rs:3407`, M5-SEED-1).
2. **`Wind::windy_enough_for_kite_drops`** (`Main.cs:2907` re-read:
   `Math.Abs(windSpeedTarget) >= _maxWind`, `_maxWind = 0.4f`
   `Main.cs:67226`) — a DISTINCT property from the butterfly gate
   (`NPC.cs:6885`, same value today) kept separate so a future
   `_maxWind` change moves only this one. **FED** at `2075c3aaa`:
   `spawn_loot` (`world_sim/npcs.rs:6811` snapshot) `windy:` reads the
   getter — the 8 kite rows (`Cond::WindyEnoughForKiteDrops`,
   `loot.rs:1719`; Wyvern 87 / Harpy 48-family / Sand Elemental 133 /
   the 170/180/171/98/86/63 rows, items 4613/4379/4611/4684/4683/4649)
   were dead-false before. The holiday-zombie snapshot (`:8051`) keeps
   its minimal flags (no kite rows on kind 3).
3. **`InAPlaceWithWind`'s `InWorld` skip** (`WorldGen.cs:86858-86879`
   re-read): an OUT-OF-BOUNDS cell is SKIPPED (`:86868`, fluff 0), only
   an in-world null tile / liquid / wind-blocking wall refuses
   (`:86871-86877`). The old `return false` on `None` refused every
   probe touching the map edge — the sunflower's 2×5 window
   (`advance_sunflower_counter`'s caller) and the player's `wind_here`
   at the border. The port's dense grid has no interior nulls, so
   `World::tile` `None` == out-of-bounds exactly.

3 witnesses WRITTEN NOT RUN: the remix both-ways flag pin, the kite
target-threshold (sign / current-vs-target), the edge-skip + liquid /
blocking-wall / AllowsWind-150 / height-gate refusals.

**RETIRED (`player.rs`, the push-chain copy — a c40-duplicate):** see §5
for the keep-one adjudication and the exact graft ledger. The retired
state is fully reconstructible from c40's pushed tip + the three graft
hunks below (the `b46556b2d` keep-one precedent — no stash needed, the
ledger IS the preservation).

## 3. The d130 mutual receipt (research/83 §9 SEALED, `b8f655708`)

Read FULL at the fetched tip `b8f655708912431a84da0c90b77fc5d69b1a8303`:

* **The bible row CONSUMED** (§9.4 quoted in the header): g68 = the
  SINGLE wind-flag owner. The g68-side confirmation: `Wind`
  (`tr-core/src/wind.rs`) is the only integrator of
  `windSpeedCurrent`/`windSpeedTarget` (`update_rain`, the 60 Hz
  `dayRate` tick via `world_sim::weather::update_world_tick`); every
  family consumer is READ-ONLY — d130's `GetRainFallVelocity`
  (`wind*18,14`) and the folded Eyebrella drip (`wind*3,9`, R6
  `df28ea1ac`) read the live value through their gathered `RainInputs`;
  c40's `Sandstorm::update_time` reads only wind (`Sandstorm.cs:32-77`,
  no rain term); g68's own leaf cadence reads `WindForVisuals ==
  windSpeedCurrent` (`Main.cs:2823`, wiring-3). No second writer exists
  on any family branch (the §4 file census).
* **The wind.rs compose: HUNK-DISJOINT, both keep.** d130's wind.rs
  delta (`git diff a725acb04 b8f655708 -- crates/tr-core/src/wind.rs`,
  +75/−5) is exactly the `lanterns_up` graft: the module-doc reword
  (`:15-24`), the field (`:74-80`), the ctor init (`:118`), the
  `update_rain` early-out AFTER the creative-freeze gate
  (`:200-210`, `Main.cs:58856-58859`), and the
  `lanterns_up_skips_the_target_walk_but_not_the_drift` witness
  (`:851+`). g68's final-phase delta touches the
  `update_windy_day_state` doc+body (`:120-163`), the NEW kite getter
  (`:171-180`), `in_a_place_with_wind` (`:356-380`), and 3 witnesses at
  the test-module tail (`:1088+`). ZERO shared lines — the merge is a
  clean union, no dedupe owed. (d130's `lanterns_up` is the c39-graft
  sibling — the wind-target walk skips under Lantern Night while the
  drift still eases; g68's leaf cadence reads the CURRENT speed, so the
  lantern skip is behaviour-correct for leaves too.)
* **The four-file overlap census** (`git diff --stat a725acb04 <tip> --
  wind.rs foliage.rs gore.rs sand.rs`): d130 touches ONLY wind.rs (the
  graft above); g67 (`8b9a1ff46`) touches NONE of the four; c40
  (`5552a3266`) touches NONE of the four (its player.rs overlap is §5).
  The leaf/gore/sand bodies are g68's wholesale.
* **The gate command ADOPTED**: `cargo test --workspace -- --skip
  migrates_version_2_kinds_to_item_ids` — judge only by the `test
  result: ok` lines; every family witness is WRITTEN NOT RUN, the
  merged tree's first full run is the gate.

## 4. The g67 mutual receipt (research/146, `8b9a1ff46`)

Read at the fetched tip `8b9a1ff46c4a4e795c51ea96f9c13568665ca16b`
(the 134→146 rename `c38d42d32`, the registry-150 first-pushed-wins
sweep):

* **The g68 row CONSUMED** (146's WEATHER-FAMILY paragraph, quoted):
  "**g68** (`task/W4-g68-weather-wind-leaves`) owns the wind sim + leaf
  cadence (`UpdateLeafFrequency` band-for-band, `3ccd49e6`) — the LIVE
  `windSpeedCurrent` it integrates feeds d130's `GetRainFallVelocity`
  (`wind*18,14`) and g67's Eyebrella drip (`wind*3,9`) and c40's storm
  severity, so the wind flag has ONE owner (g68) with read-only
  consumers". The `3ccd49e6` cite is the wiring-3 cadence verify (§1) —
  g67's seal-monitor read the correct commit.
* **g68-wind DISJOINT confirmed both ways**: g67's drop-wholesale FINAL
  (rain.rs + draw.rs census wiring drop, ONLY the R5/R6/R7/A2 four-commit
  hash-list crosses into d130) carries NO wind.rs / foliage.rs / gore.rs
  / sand.rs hunks (the §3 four-file census at `8b9a1ff46` is empty). The
  Eyebrella drip's `wind*3,9` feed is a READ of g68's flag through d130's
  gathered `RainInputs` — g67's R6 fold (`df28ea1ac`, now d130's) never
  writes the wind state. No compose owed either direction; the g68 leaf
  bodies and the g67→d130 rain core merge whole, side by side.

## 5. The c40 receipt — sandstorm wind-only, the c40c-resume, and the push keep-one

c40 tip `5552a32669fd536d1b8c140ea78ac8aca5f38a27` = the **c40c
checkpoint** ("freeze-resume the in-flight storm-bolt geometry + spawn
lane — `lightning_generator.rs` 507 L, `storm_lightning.rs` 595 L,
unregistered modules, verification pending"). The bolt lane is c40's
remainder; everything below is the LANDED c40 core the g68 seal composes
against.

* **Wind-only severity CONFIRMED** (c40 research/78, read at the tip):
  `Sandstorm.cs`'s whole sim machine is `tr-core/src/scene.rs::Sandstorm`
  — `update_time` (`:32-77` → the `TimeLeft` reset/decay, the
  `15*dayRate` dead-wind penalty `HasSufficientWind = |wind| >= 0.6`,
  the `wind == 0 → TimeLeft = 0` instant stop, the
  `Next(21600 * (hardMode ? 2 : 3))` start roll, the `Next(18000)`
  intention), `update_severity` (`:99-115` → the sign-step 0.003/tick,
  NOT a proportional lerp), `change_severity_intentions` (`:79-97`).
  **No rain term anywhere** — the storm reads ONLY g68's wind flag
  (the bible's read-only-consumer contract). The severity consumers are
  all accounted (EmitDust → sand.rs f42/dust lane, SandstormSky → the
  f42 visual gap, SceneState filter params landed, the save quad
  landed).
* **The ONE-WAY gate CONFIRMED** (`SceneMetrics.cs:698-703` re-read
  verbatim): `ZoneRain = Main.raining && SurfaceAtmospherics`;
  `ZoneSandstorm = ZoneDesert && SurfaceAtmospherics &&
  Sandstorm.Happening`; `if (ZoneSandstorm) ZoneRain = false;` — rain
  NEVER gates the storm. The port already carries it
  (`scene.rs:786-790`, base-trunk, the
  `zone_sandstorm_suppresses_zone_rain` witness `:2645`). **One cite
  correction for c40**: the port comment says `SceneMetrics.cs:704-707`
  — the actual lines are `:698-703` (`:704-707` is the shimmer window);
  d130's bible cites `:698-703` correctly. Comment-only, no behaviour.
* **The player.rs push KEEP-ONE: c40 CANONICAL.** The in-flight g68
  dirty player.rs (+193/−22) ports the SAME `windPushed` chain c40
  pushed FIRST (`fa3717be1` 00:32:47 the push + grant, `bdd039802`
  00:42:40 the i12 sandStorm reuse, `7fc862c91` 02:44:02 the witness
  hardening — all BEFORE the g68 dirty mtimes 09:36/10:02). c40's copy
  is the STRICT SUPERSET: the `Auras.sandstorm_wind` → `AddBuff(194, 2)`
  grant (`Player.cs:25556-25559` re-read), the bottle-jump lifecycle
  (`:26573-26577` run boost, `:26743` clear, `DoubleJumpVisuals`
  `:21905-21919` re-arm, `:26753-26756` cancel), and 8 witnesses incl
  the zero-draw RNG census. The g68 copy has NO grant (the flag could
  only be set by hand) — it is the consumer half alone. **Fidelity
  clincher**: the g68 copy used `wind_speed.signum()` — Rust's
  `f32::signum` returns ±1.0 for ±0.0, NEVER 0, so dead-calm wind 0
  would push +0.06 where C# `Math.Sign(0f) = 0` pushes nothing; c40's
  explicit `(w > 0) as i32 - (w < 0) as i32` sign is the faithful form.
  The g68 copy is RETIRED wholesale (the `b46556b2d` keep-one
  precedent); its three g68-UNIQUE refinements survive as the exact
  graft ledger below — the retired state is reconstructible from c40's
  tip + these hunks, nothing is lost.

### 5.1 The graft ledger — three g68-unique refinements for c40/the merger

All three re-verified against the C# at seal time; c40's tip lacks them
(its `can_be_pushed_by_wind` says "isLockedToATile — unported (constant
false)", its two frame sites say "the windPushed term has no port
counterpart"). Line numbers are c40-tip (`5552a3266`) player.rs:

* **G1 — the `isLockedToATile` gate** (c40 `:10370-10372`,
  `Player.cs:19754-19757` + `:4020-4030` re-read: the getter is
  `sitting.isSitting || sleeping.isSleeping`). The port HAS both fields
  (`player.rs:1139 pub sitting`, `:1143 pub sleeping`). Graft: replace
  the "unported (constant false)" comment with the head guard
  ```rust
  // `:19754-19757`: isLockedToATile (`:4020-4030`) = sitting || sleeping.
  if self.sitting || self.sleeping {
      return false;
  }
  ```
  before the mount block. (The g68 dirty copy's `holding_dir` local is
  cosmetic — c40's `flag` local is the same predicate.)
* **G2 — the mount grounded-frame rest row** (`Mount.cs:4550` re-read:
  `slippy || slippy2 || windPushed`). c40 has TWO
  `GetIntendedGroundedFrame` copies — the `:36216` Update site
  (`:7877`) AND `try_landing` (`:7754`, `Mount.cs:4562-4568`'s touchdown
  re-probe). **The g68 dirty diff patched only the first — the merger
  must apply BOTH** (the C# has one fn, both port copies read the same
  predicate):
  ```rust
  || ((self.slippy || self.slippy2 || self.wind_pushed)
      && !self.control_left && !self.control_right)
  ```
  and drop the "no port counterpart" comment at `:7876`.
* **G3 — the walk-cycle rest row** (c40 `:7927`, `Player.cs:36250`
  re-read: `(slippy || slippy2 || windPushed) | flag8`):
  ```rust
  if ((self.slippy || self.slippy2 || self.wind_pushed) | roller_skates)
  ```
* **Cite correction (comment-only)**: c40's field/reset docs cite
  `Player.cs:16970` for the `ResetEffects` clear — `:16970` is inside
  `UpdateDead` (`:16921`); the `ResetEffects` (`:18275`) clear is
  `:18739`. BOTH clears are real C# (the dead player also drops the
  flag); the port's `reset_effects` clear should cite `:18739`, and the
  `UpdateDead` `:16970` clear is a c40-lane remainder to census (the
  port's dead-player path may already fold it — not g68's scope).

The g68 dirty copy's 2 witnesses (the push accel + the gates/tiers) are
SUBSUMED by c40's 8 (its `w4_c40_the_push_accelerates_to_the_wind_speed`
/ `_tiers_airborne_and_steering` / `_can_be_pushed_by_wind_gates` pin
the same maths with the exact-f32 replay); the G1 sitting refusal gains
one assertion in c40's gates witness at the graft (the g68 dirty
witness's sitting arm is the spec: `p.sitting = true` → vx stays 0).

## 6. The co-lane probe verdict (dirty-2)

`git status` at resume: `player.rs` + `wind.rs` modified, branch
`task/W4-g68-weather-wind-leaves` (the task brief's
`task/W4-g68-weather-wind` slug is the same lane — the leaves suffix is
the pushed branch name). Verdict: **NOT an alive-advancing foreign
co-lane** —
* no live cargo/rustc/rust-analyzer process (`Get-Process` empty),
* the dirty mtimes (09:36 / 10:02) are ~4.5 h stale at probe (14:30),
  the last commit 06:49 — no advancing writer,
* every dirty hunk cites **W4-g68** ("final-phase missing-arm port") —
  it is THIS lane's own in-flight work, not a FOREIGN writer (contrast
  the d36-inflight stashes, which were explicitly foreign),
* the stash list is untouched (the 6 preserved FOREIGN stashes stay).

Disposition: the g68-unique wind.rs half LANDED (`32ee28ca5` +
`2075c3aaa`); the c40-duplicate player.rs half RETIRED with the §5.1
graft ledger as its preservation (reconstructible, nothing lost). No
clobber, no foreign state committed.

## 7. The WIND-FAMILY bible (FINAL — one paragraph, for the merger)

The wind family composes under the d130 WEATHER-FAMILY bible (§9.4) with
**g68 the SINGLE wind-flag owner**: `tr-core/src/wind.rs`'s `Wind` is the
only integrator of `windSpeedCurrent`/`windSpeedTarget` (the 60 Hz
`update_rain` dayRate tick, the target walk + ±0.8 clamp, the gust grid
S1-G-8, `leaf_frequency` band-for-band `UpdateLeafFrequency`
`:7719-7771`), and g68's final phase closes its three missing arms (the
remix `UpdateWindyDayState` branch `:13021-13027`, the
`WindyEnoughForKiteDrops` getter `:2907` FED into `spawn_loot`'s
`LootFlags.windy`, the `InAPlaceWithWind` `InWorld` edge-skip
`WorldGen.cs:86868`) — d130's `lanterns_up` graft composes HUNK-DISJOINT
(the target-walk skip, both keep, zero shared lines); **every consumer
is read-only**: d130's rain fall velocity (`wind*18,14`) + the folded
g67 Eyebrella drip (`wind*3,9`) through the gathered `RainInputs`, c40's
wind-only `Sandstorm` severity machine (`scene.rs`, the ONE-WAY
`ZoneSandstorm`→clears-`ZoneRain` gate `SceneMetrics.cs:698-703` already
base-landed + witnessed, rain never gates the storm), g68's own leaf
cadence (the GorePool-routed `EmitTreeLeaves`/`EmitLivingTreeLeaf` trio +
the site-1 quarter-roll on the 5-draw-cycle `tile_frame_seed_at`
schedule, the per-frame `EmitLiquidDrops` seam on the persistent
`App.tile_draw_rand`), and the player Strong-Wind push — **c40 CANONICAL**
for the push chain (`fa3717be1`/`bdd039802`/`7fc862c91`, the grant +
bottle lifecycle + 8 witnesses, the faithful `Math.Sign` form), g68's
duplicate RETIRED with the three refinement grafts (G1 the
sitting/sleeping `isLockedToATile` gate, G2 the TWO
`GetIntendedGroundedFrame` rest-row copies, G3 the walk-cycle rest row —
§5.1, apply at the c40 merge); the c40c bolt-geometry checkpoint
(`5552a3266`, unregistered modules) is c40's own remainder, orthogonal to
the wind flag. No two family branches own the same fn body; the only
cross-branch writes are the enumerated grafts (d130's `lanterns_up`,
g68's G1-G3) and they are all published. **Gate command**: `cargo test
--workspace -- --skip migrates_version_2_kinds_to_item_ids` — judge only
by the `test result: ok` lines; 32 g68 witnesses (grep-verified, +1 the
§8 dead-player gate pin) + c40's 8 push witnesses + the family's seals
are WRITTEN NOT RUN (code-only), the merged tree's first full run is
the gate.

## 8. FINAL-PHASE ADDENDUM (post-seal, this lane) — the wind-system census close, the particle interplay, the leaf-gore chain, the shake cross-note, the §8.5 receipt

Seven commits on top of the seal + rename (`97a6fd083`): `08a4c3bb0`
(the census-count correction 33→31), `f5a65c7da` (the WindGrid
dead-player gate, +1 witness → 32), and the five per-item docs
commits below (§8.1 the item-1 census close, §8.2 item 2, §8.3 item 3,
§8.4 item 4, §8.5 item 5). All findings re-verified against the C# at
the post-seal tips: g67 `534ba9a35`, c40 `8b7eb0e1e`, g17 `2c73354f1`
(the docs-only research renumber 304→339 on top of `3dec710dc` — the
code cites below are the `3dec710dc` bodies, unchanged), g31
`36b5f2d78`, d130 `b8f655708`, d60 `77429fe1d`.

### 8.1 The wind SYSTEM census (item 1 close) — C# `Main.UpdateWeather` :58737-59037 vs the port

There is NO `Main.UpdateWind` in the 1.4.5.8 decompile — the brief's
":59000-region" is `UpdateWeather`'s cloud-count tail; the wind machine
is `:58830-58950` inside it. Full arm-by-arm verify (the port =
`tr-core/wind.rs::Wind::update_rain` + `world_sim/weather.rs::
update_world_tick`):

| C# arm | lines | port | verdict |
| --- | --- | --- | --- |
| the drift `num7 = 0.0003 + |num8-current|*0.0015`, `num8 = target*(1+5/9*maxRaining)`, move-towards | :58830-58849 | `wind.rs:195-211` | 1:1 (the f32 op order kept) |
| netMode/gameMenu early-out | :58850-58853 | `frozen` fold (`wind.rs:215-217`) | documented (module doc) |
| creative FreezeWind gate | :58854 | `frozen` fold | documented — the creative power id 10 is NOT ported (`creative.rs:427` `test_DW_6_3_the_unported_vanilla_ids_are_never_written` pins ids 9/10/12 never written); the `frozen` FIELD exists and the world tick could feed it, a creative-power-wave obligation |
| LanternNight.LanternsUp early-out | :58856-58859 | d130's `lanterns_up` graft (`b8f655708`, §3) | composing hunk-disjoint, both keep |
| `windCounter--`, the `Next(4)/Next(2)/±100` walk | :58860-58888 | `wind.rs:220-234` | 1:1 incl the draw order |
| the <120-life 0.35 cap (BOTH sites) | :58889-58892, :58917-58920 | `wind.rs:235-237, :256-258` | 1:1; the 255-player scan folds to the one local player (`weather.rs:135`, the S1-L-2 witness) |
| `extremeWindCounter--` + the 13/30 → (1/2 → calm 7200..28801 | ±200) / 13/20 → ±400 / else ±850 tiers | :58893-58916 | `wind.rs:240-255` | 1:1 incl the `Next(30)<13`/`Next(20)<13` gate order |
| the >0.3/>0.5/>0.7 counter extensions (5..11/10..21/15..31) | :58921-58932 | `wind.rs:259-267` | 1:1, all three thresholds CUMULATIVE (not else-if) — kept |
| `ResetWindCounter` (900..2701, extreme 10..31) | :58698-58706 | `wind.rs:292-297` | 1:1; the C#'s fresh `FastRandom.CreateWithRandomSeed()` per call vs the port's shared xorshift64 is the documented deviation (module doc) |
| the 2/3 direction-flip roll | :58938-58941 | `wind.rs:273-278` | 1:1 (the sign captured BEFORE the walk) |
| the ±0.8 target clamp OUTSIDE the counter gate | :58943-58950 | `wind.rs:282-287` | 1:1 (runs even when the counter did not fire) |
| the cloud-count walk + weatherCounter | :58952-59036 | UNPORTED — `rain.rs:57-60` documents the `numClouds = 100` stand-in (the `numClouds > 150`/`> 100` ChangeRain arms read false), `sky.rs:276-280` the clear-weather clamp equilibrium | the g69 cloud-parallax lane's (`task/W4-g69-weather-clouds-parallax` exists); NOT a wind-arm gap — the wind machine ends at :58950 |
| `PostContentLoadInitialize` wind init ±0.8 | :10874-10875 | `Wind::new` (`wind.rs:98-118`) | 1:1 ranges; the C# rolls on Main.rand between moonType and numClouds, the port on its own seeded stream — the documented determinism stand-in |
| `UpdateWindyDayState` | :12992-13030 | `wind.rs:128-162` | 1:1 incl the remix branch (the seal's arm 1) |
| `TileDrawing.Update` counters (tree 1/240+2, grass 1/180+4, sunflower 1/420+5, vine 1/120+0.4) | TileDrawing.cs:367-386 | `advance_*` (`wind.rs`), driven `draw.rs:827-847` | 1:1 formulas; CADENCE deviation documented: tree/grass advance per DRAW, sunflower/vine per 60 Hz step — the C# advances ALL FOUR per world update (`TilesRenderer.Update()` Main.cs:17812). The per-frame pair is the S1-G-8 convention (the sway counters ride the G6 key); the wiring wave's per-frame-move contract covers the reconciliation |
| `EnsureWindGridSize` + `_windGrid.Update()` + ScanPlayers | :377-378, WindGrid.cs:25-101 | `draw.rs:858-882` + `WindGrid` (`wind.rs:482+`) | 1:1 EXCEPT the dead-player gate — **FIXED `f5a65c7da`**: the C# `ScanPlayer` head refuses `!active || dead` (WindGrid.cs:91); the port fed every `Some(player)` incl the drifting corpse. The `.filter(|p| !p.dead)` now rides the borrow (+1 source-pin witness). The `velocity == 0` / camera-intersect / NaN gates were already inside `scan_player` (`wind.rs:618-633`) |
| the push family (GetWindGridPush/2Axis/Complex/Average/Highest) | :8554-8607, :9195-9233 | `wind.rs:645-787` | 1:1 verbatim incl the `for i in 0..1` quirk (:9222) and the `(int)` truncations; the CONSUMERS: vines (:9076 → `vines.rs:160`) wired; the grass passes (:8368, :8425) NOT wired — `tiles.rs:879/887` documents "the WindGrid gust push is not ported" (the g31/g84 drain lane owns the grass passes, §8.5); the multi-tile-vine Highest (:9242) rides the unported DrawMultiTileVinesInWind pass (g84's mapping answer, g31 §8.6) |

**Missing-arm verdict: ONE found + fixed** (the dead-player gate). The
creative-freeze fold and the counter-cadence split are pre-existing
documented deviations, not new gaps. The `TR_WIND_FORCE` clamp ±1.24
(`draw.rs:825`) matches the rain-scaled current's max `0.8*(1+5/9)` ≈
1.244 — the screenshot pin, not a sim arm.

### 8.2 The wind PARTICLE interplay census (item 2) — complete

Every wind-consuming particle family, its stream, and its owner:

| family | C# | port | wind input | owner |
| --- | --- | --- | --- | --- |
| wind leaves (tree sheds) | TileDrawing.cs:7779-7891 + :7549-7592 | `foliage.rs::emit_tree_leaves` + the `emit_living_tree_leaf` trio, the `tiles.rs:1107-1131` 192/384 arm | `speed_current` via `leaf_frequency` + the ±0.2 two-cell gates | **g68** (wired, the §1 census) |
| leaf PHYSICS drift | Gore.cs:1126 (water rest `vector3.X = WindForVisuals`), :1166 (fall `+= wind*4` above surface) | `gore.rs:1605/:1647` (`update_leaf`, the gores phase `world.rs:273-292` on the UpdateGores stream) | `wind_for_visuals` = `speed_current` (`world.rs:277`) | **g68** (S1-E-4 + the g68 pins) |
| drips (vine/stalactite/candle) | TileDrawing.cs:7594-7674 | `foliage.rs::emit_liquid_drops(_area)`, `draw.rs:1250-1269` | none (the gate is `Next(num*2)`; the wind lane owns the SEAM) | **g68** (wiring-1; the §8.5 receipt below) |
| sandstorm debris | Sandstorm.cs:144-192 | `sand.rs::emit_dust` (`scene_scan.rs:403` feeds `speed_current`) | `sign(wind)*Lerp(0.9,1,|wind|)` drift + `num13*20` push, dead-calm <0.01 refusal | **c40** severity machine (`scene.rs::Sandstorm`, wind-only) + the S1-E-1/E-3 dust lane; the g68 wind-coupling witness (`sand.rs`) |
| rain drift | Rain.cs:56-57 (spawn `wind*15*40`), :91-94 (`GetRainFallVelocity` = `(wind*18, 14)`) | g67 `534ba9a35` rain.rs:1183/:1201 through the gathered `RainInputs.wind_current`; the FOLD into d130 (`b8f655708`) per the g67 seal | `speed_current` read-only | **g67→d130** (the mutual receipt §4; g67's drop-wholesale carries no wind hunks) |
| Eyebrella drip | Player.cs:14328 (`Rain.NewRainForced(wind*3, 9)`) | d130's R6 fold (`df28ea1ac`) | `speed_current` read-only | **d130** (the bible row) |
| lava leaf puff | Gore.cs:1119-1122 | `gore.rs:1578-1604` (the DustFromGore seam, UpdateGores stream) | none directly (the leaf's drift carried it) | **g68** (deferred-1, wired) |
| vision dust / petals / smoke | TileDrawing.cs:897-962, :6796-6847 etc. | d56 `animated_tiles.rs` / d60 `tile_draw_particles.rs` | none (the `_rand` gates only) | **d56/d60** (the deferred-4 census; the wiring-4 cross-note verified the stream orders) |

**Completeness verdict: YES** — every `Main.windSpeedCurrent` /
`WindForVisuals` particle consumer in the C# (the grep census over
Cloud/Gore/Main/Mount/NPC/Player/Projectile/Rain/SceneState/WorldGen/
TileDrawing/AmbientWind) maps to a ported family above or to a
non-particle consumer (the vane/flag tile counters
`drawdata_anim.rs:440-448`, the water-distortion `scene.rs:2400`, the
bubble NPC jitter `npc_ai_style70.rs:262`, the kite/leaf loot gates,
the mount pose `Mount.cs:3572` — all read-only, all accounted). The
ONE interplay seam between families: the leaf's water-rest drift reads
the SAME live flag the rain spawn does — no stream coupling (the leaf
physics rolls UpdateGores, the rain spawn rolls its own), so the
families compose without ordering constraints.

### 8.3 The leaf GORE types chain verify (item 3) — emit → gore-pool → draw

The full chain from g68's cadence, verified end-to-end at this tip:

1. **EMIT** (`foliage.rs:499-597`): the `draw_tree_foliage` top/branch
   sites (`:443/:462/:479` — the C# :8121/:8166/:8214 order) roll
   `emit_tree_leaves`; the 192/384 living-wood arm (`tiles.rs:1107-
   1131`) carries the site-1 quarter-roll (wiring-3). The passStyle
   comes from `tree_leaf_pass_style` (`foliage.rs:719-759`).
2. **POOL** (`gore.rs:433-443` `new_gore_direct` → `new_gore`): the
   SpecialAI-3 arm (`gore.rs:585-606`, Gore.cs:1391-1402) rolls the
   four draws (vel X/Y, row, counter) and frames the leaf 32×8 (the
   910-925/1113-1121/1248-1255/1257/1278 set, `gore_id.rs:30`); the
   paint column rides `frame.current_column` (`foliage.rs:595`).
3. **TICK** (`world.rs:642` gores phase → `gore.rs:1035` →
   `update_leaf` :1541-1702): the Gore_UpdateLeaf port (the 14 g68
   physics witnesses).
4. **DRAW** (`draw.rs:1316` → `render/gore.rs:220`): the
   `draws_in_front` gate (:133-142 — leaves are not IsDrip, always
   front), the `gore_sheets` lookup (:239 — every Gore_<id> packed,
   910/911 via STATIC_SHEET_NAMES, the rest via the enumeration
   `assets.rs:80-97`), the 32-column `source_rect` (:39-44), the
   `GetAlpha` tint. **The chain is CLOSED — no private leaf pool
   remains** (the phase-1 retirement).

**The g17 hallow-tables relationship (the keep-one flag):** g17
(`3dec710dc`) landed `tr-core/tree.rs::tree_leaf` — the FULL
`GetTreeLeaf` port incl BOTH hallow tables (the style-3 x%3 nine-entry
table AND the style-20 x%6 eighteen-entry 1113-1121 table) + the
`height_add = 5` ref-write + the `hollow_tree_foliage_style` reader
(the `.wld` setBG(4) byte, `world.rs:121`). g68's `tree_leaf_pass_style`
is a SIMPLIFIED TWIN: it lacks the style-20 arm (its comment says
"unreachable today" — WRONG since the `.wld` load carries
`hallow_bg_style` 2/3, `wld/sections/flags.rs:456`, and g17's
`hollow_tree_foliage_style` maps 2|3 → 20 — a loaded vanilla hallow
world DOES reach the style-20 table), lacks the
`frameX in {22,44,66}` guard on the frame pick (harmless — the caller
only feeds marker frames), and drops the height_add (the ambient shed
never reads treeHeight — the C# `out var _` discards it too,
TileDrawing.cs:7791). **Keep-one recommendation: g17's `tree_leaf`
WINS at the merge** (the strict superset, the C#-complete tables, the
world-aware hallow style; g17's own doc comment already flags the g68
twin as "the style-20-unreachable simplification"); g68's
`emit_tree_leaves` rebinds its one call site (`foliage.rs:526`) to
`tree::tree_leaf(world, grass_x, marker.kind, marker.frame_x,
marker.frame_y, ground).1` and the local twin DELETES. The g68
`hallowed` range check (`:530`, 917..=925 | 1113..=1121) already
covers BOTH tables' outputs — the rebind changes no gate. This is the
C2-class leaf dedupe's FOURTH copy (the three `emit_living_tree_leaf`
copies keep-one onto g68's foliage body per g31 §9.1; the passStyle
TABLE keep-one is this new row — g17 canonical).

### 8.4 The tree-shake interplay cross-note (item 4)

g17's ShakeTree lane is ACTIVE at `2c73354f1`: the axe fail tick
(`editing.rs`, Player.cs:45758 → WorldGen.cs:63710-63714) runs
`shake_tree_resolve` (`tr-core/tree.rs:663`, :63133-63162
dedupe/walk) → `shake_loot::shake_tree_loot`
(`world_sim/editing/shake_loot.rs:120`, :63163-63403, the genRand
ladder, ONE UpdatePlayers throw seeds the per-shake FastRandom) →
`shake_tree_fx_at` (`tr-core/tree.rs:707`, :63404-63417) →
`world_gore_a::tree_grow_fx(hitTree: true)` (`world_gore_a.rs:126`)
— the C# order, the FX tail RNG-free in its position.

**Composition with g68's wind leaves — NO conflict, three axes:**
* **Cadence**: the shake burst is EVENT-driven (one 5-leaf top-row
  burst per shake, `TreeGrowFX` :29626-29634 `num3 = 5` when hitTree
  (20 for a grow), height 1 → 3 trunk gores + 5 leaves = 8, the g17
  witness); the wind shed is CADENCE-driven (the frequency gates per
  draw). Both spawn into the SAME 600-slot GorePool through the same
  `NewGore` allocator — the pool pressure composes (the first-inactive
  scan refuses identically for both).
* **Streams**: the shake's kicks roll the UpdatePlayers stream (the
  ItemCheck swap); the wind shed's spawn internals roll the bare
  global Main.rand (outside every SwapRandom, the deferred-2 census);
  the leaf PHYSICS of both rolls UpdateGores. Three distinct streams,
  no interleaving constraint — a shake tick and a shed draw never
  share a stream.
* **Types**: both use the same passStyle family (the shake reads
  `tree_leaf` at the walked leafy top over the ORIGINAL ground row
  `num`; the shed reads the marker's own ground walk). After the §8.3
  keep-one rebind both read the SAME g17 table — the style outputs
  agree by construction. The ash shake (1278) delegates to
  `TreeGrowFX_AshShake` (dust + 61+Next(3) gores, NOT the leaf family)
  — the wind shed refuses 1278 (`foliage.rs:531`), so the two never
  double-spawn the same kind from the same tree.

**Cross-note for g17**: the shake's `tree_grow_fx` leaf gores carry
`Frame.CurrentColumn` from the TRUNK tile's paint (:29614), the wind
shed's from the MARKER tile's paint (:7889) — both faithful, different
C# sites, no reconciliation owed. The shake burst's leaves ride the
same `update_leaf` physics (the wind drift applies to them too — the
C# does not distinguish spawn source in Gore_UpdateLeaf).

### 8.5 The g31 §8.5 receipt (item 5) — the keep-one recommendation

Read at the g31 FINAL tip `36b5f2d78` (the §8.5 FINAL-phase update +
§9 execution record; the drip order plan EXECUTED `10530fd88`,
`tile_attach.rs::classify_and_draw_cell` plans `drip_plan` row-major,
witnessed equal to `reg.liquid_drop_cells` —
`w4_g31_attach_drip_plan_preserves_the_switch_arm_order`).

**The two candidates:**
* **g68's live seam** (wiring-1, `ea9e784f6`): `emit_liquid_drops_area`
  rescans the draw view rect per frame beside tile_cracks
  (`draw.rs:1250-1269`), gate stream the persistent
  `App.tile_draw_rand`, spawn internals the bare global Main.rand.
  WIRED on this branch; the rescan is g68's own documented stand-in.
* **The union direct-bind** (g31 §8.5's published 2-line hunk): the
  CAD LiquidDrops arm calls `foliage::emit_liquid_drops(world, y, x,
  kind, water_style, zone_corrupt, zone_crimson, zone_hallow,
  &mut self.rand, self.main_rand, self.gore)` DIRECTLY per classified
  cell, the record push stays as the census input, and **g68's rescan
  call DROPS**.

**KEEP-ONE RECOMMENDATION: the union direct-bind WINS.** Reasons, in
the contract's own terms:
1. **Stream order is the pixel contract** (research/136 C5): the C#
   emits INSIDE the `Draw` switch (:688-694) BEFORE the `continue`, so
   the `Next(num*2)` gate draws on the ONE `_rand` interleaved with
   every other switch-arm draw in row-major cell order. The rescan
   rolls the same gates in the same row-major order but in a SEPARATE
   pass — the drip gates land after ALL the tile-pass draws instead of
   interleaved. Once the union's CAD owns the other `_rand` consumers
   (the d60 battery, the g31 drains), the rescan's separate-pass order
   diverges from the C# stream; the direct bind cannot.
2. **The rescan is a stand-in by its own doc** — g68's wiring-1
   comment and the ANALYSIS deferred-4 row both say "when g30 lands,
   the wiring wave swaps the rescan for the recorded cells"; g31 §8.5
   sharpens this to the DIRECT bind (not a drain — draining
   `liquid_drop_cells` after the walk would re-order the stream the
   same way). The g68 lane ACCEPTS the sharpening: the per-cell emit
   is the C# shape, the rescan was always the approximation.
3. **No double-roll**: keeping both would roll every drip gate twice
   (the contract's own warning). The bind drops the rescan call —
   ONE emission site.
4. **The body is g68's wholesale either way** — the union binds
   g68's `emit_liquid_drops` (the exact signature re-verified at this
   tip, `foliage.rs:165-176`, matches g31's published signature
   verbatim), no body dedupe owed; only the DRIVER keep-one is decided
   here.

**The g68-side obligations at the bind** (for the merger): drop the
`emit_liquid_drops_area` call block (`draw.rs:1225-1269` incl the
comment); the `emit_liquid_drops_area` fn (`foliage.rs:273`) + its
area-driver witness
(`w4_g68_the_area_driver_scans_drip_kinds_row_major`,
`foliage.rs:1310`) retire with it (the row-major order it pins is
preserved by the CAD walk + g31's `drip_plan` witness); the per-drip
witnesses (the 706 vine, the type overrides, the wet/anti-cluster
refusals) survive UNCHANGED — they test the body, not the driver. The
persistent `App.tile_draw_rand` stays (the CAD's `self.rand` binds
it). The `DrawnThisUpdate` gate reconciliation stays the d60 wiring
wave's contract (the deferred-2 row) — the direct bind inherits it,
the rescan's `!game_paused` stand-in retires with the call.

**Final status (the research close):** the g68 lane is COMPLETE —
the wind sim verified + its missing arms landed (the seal §2 + the
§8.1 dead-player fix), the leaf cadence + physics + draw chain closed
(§8.3), the emission seams wired with the keep-one recommendations
published (the §8.3 passStyle table → g17, the §8.5 drip driver →
the union direct-bind, the push chain → c40 per the seal §5). Every
witness WRITTEN NOT RUN (32 on the lane, grep-verified); the merged
tree's first full `cargo test --workspace -- --skip
migrates_version_2_kinds_to_item_ids` run is the gate.

