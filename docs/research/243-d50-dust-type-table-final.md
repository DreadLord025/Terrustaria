# 243 — W4-d50 dust-type-table RETIRE FINAL (the wiring census, the G3/G4 keep-one drop-hunks, the flag-b/flag-c receipts + the dust-type-table family declaration)

Lane: `task/W4-d50-dust-type-table`, worktree `TR-W4-d50`. Code-only mode:
**no cargo/rustc/rustfmt/tests/graphify invoked** (user directive). Witnesses
WRITTEN NOT RUN — the compile window rides the merge-verify wave
(d54 research/131 §8.3 step 13).

**Number 243 per the d54 registry-150** (`38bfd1d22`,
`docs/research/150-research-number-registry.md` §2: free block 151–236
consumed, "Numbers 237+ stay FREE for future lanes", late pushers take the
first free number). Fresh scan at this retire (2026-11-21, after
`git fetch origin --prune`): every `origin/task/*` tip's `docs/research/`
ls-tree — 237 taken ×2 (e23 `237-bestiary-ui-final`, g48 `237-f11-tail-fix`),
238 taken ×3 (c8 / d42 / g61), 239 taken ×2 (d42 / g56), 240/241/242 taken
(g58 ×3), **max-on-origin = 242, nothing numbered ≥243 at any tip** → 243 is
the first free number. This doc is d50's FINAL receipt; it claims 243.

## 1. Census — what landed on this branch (git log)

Base `a725acb04` (wave-3 magic+summon landing = the d54×g13 merge-base).
Six lane commits, in order:

| Commit | Scope | Files |
| ------ | ----- | ----- |
| `cb07d2b66` | phase 1 — port `Dust.UpdateDust` per-type physics chain (`Dust.cs:467-2334`), UNWIRED module + 34 `w4_d50_*` witnesses | `crates/tr-core/src/dust_physics.rs` (new, 2154 lines), `lib.rs` |
| `2e243a2d2` | phase 2 — record the chain (anchor + PORT_MAP + ANALYSIS) | `anchors.txt`, `docs/PORT_MAP.md`, `docs/ANALYSIS.md` |
| `ea749c5ae` | wiring phase 1 (tr-core) — the pre_move-35 STRIP + the `update_dust` head wiring + the snowDust union | `dust_physics.rs`, `particles.rs` |
| `807350de0` | wiring phase 2 (tr-app) — `type_chain` into the live tick + the `dust_lights` decrement retire | `app/frame/world.rs`, `dust_lights.rs`, `minecart.rs`, `sand.rs` |
| `e84e6bd76` | wiring phase 3 — re-derive the phase-1 arithmetic witnesses for the in-chain generic tails + the research/69 §8 receipt docs | `dust_physics.rs`, `anchors.txt`, ANALYSIS/PORT_MAP/ROADMAP |
| (this commit) | RETIRE FINAL — research/243 + INDEX row + ANALYSIS/PORT_MAP/ROADMAP tails | docs only |

`2e243a2d2` is the tip the g13 §8 probe receipted dead-clean (`f9566e5b4`)
and the tip d54 research/131 §7.2's gap-table read. The wiring phases
(`ea749c5ae`/`807350de0`/`e84e6bd76`, 2026-11-21 up to 13:07:13+02) are the
mechanical apply of research/69 §4-6+8 that probe left owed. **The ledger
items (2) pre_move-strip and (3) type_chain-wire are CONFIRMED-LANDED, not
missing — §2/§3 are the receipts.** The d54-bible §8.3.12 "d50 wiring wave"
row flips OWED → EXECUTED at `e84e6bd76`.

## 2. pre_move-strip — CONFIRMED-LANDED (`ea749c5ae`, research/69 §4 contract)

The strip-contract (g13 research/69 §4: g13's inline increment CANONICAL,
d50's `pre_move` 35 arm to strip, the unused `lava_bubbles` parameter d50's
to drop) applied verbatim at `ea749c5ae`. Receipt at HEAD `e84e6bd76`:

* `dust_physics.rs:179` — `pub fn pre_move(d: &mut Dust)` — the
  `lava_bubbles: &mut i32` parameter is GONE; the body (`:180-185`) is
  exactly the 299/300/301/305 shrink/lift (`(uint)(type-299) <= 2 || 305` →
  `scale *= 0.96; velocity.y -= 0.01`, `Dust.cs:468-472`). **No type-35 arm
  anywhere in the file** (grep `dust_type == 35` in `dust_physics.rs`: only
  the `:1140` doc-comment citation of the strip).
* `particles.rs:370-383` — `update_dust` calls `pre_move` at the C#'s exact
  `:468-472` position, immediately before the CANONICAL inline `:473-476`
  increment (`if dust.dust_type == 35 { self.lava_bubbles += 1; }`,
  transplanted from g13 `150e7204a`) and the `:477` move (`:385`). Exactly
  ONE increment per live 35 per pass — the double-count research/69 §4
  predicted is closed by construction.
* Head resets `particles.rs:334-336` (`lava_bubbles`/`sand_storm_count`/
  `snow_dust` = 0, `Dust.cs:427-429`).
* Witnesses: `dust_physics.rs:1124`
  `w4_d50_pre_move_the_299_family_shrink_lift_and_the_stripped_35_arm`
  (the 35 passes through untouched); `particles.rs:2136`
  `w4_g13_lava_bubbles_counts_the_live_type35_population` (verbatim g13
  transplant: 3 live 35s → 3, second pass re-counts not accumulates, kill
  drops to 2, the increment draws nothing); the golden skeleton witness
  pins the head resets incl. `snow_dust` (`particles.rs:2119-2125`).

## 3. type_chain-wire — CONFIRMED-LANDED (`807350de0` + `ea749c5ae`)

The live dust-update path census: ONE non-test `update_dust` caller —
`tr-app::minecart::tick`, driven by `app/frame/world.rs::dust_phase`
(`Main.cs:18047`'s slot, `world.rs:305-366`: the riding player's
`spark_rng` or `rngs.global()`). Receipt at HEAD:

* `minecart.rs:1091-1147` — the tick's hook is
  `tr_core::dust_physics::type_chain(d, rng, &mut env)` (`:1140`) over a
  per-slot `DustPhysicsEnv` (`:1126-1139`): `Player::solid_collision` /
  `player::wet_collision` probes, the `DustPlayerSnap` player snapshot
  (vx/vy/wet/center; no-player = the (10,21) default-ctor fallback,
  `:1060-1067`), the bright-default `light_color` (43 never spuriously
  dies until the S1-E-5 sample seam), the child-spawn sink, the
  `sand_storm_count`/`snow_dust` env deltas.
* Hook signature: `particles.rs:323` `FnMut(&mut Dust, &mut R) -> bool` —
  the chain's census draws ride the SAME stream the head rolls; one
  stream, no new instances. The bool is the SECOND-chain match
  (`dust_physics.rs:928-1018`), suppressing the skeleton's `:2335` gravity
  tail (`particles.rs:391-398`) exactly as the C#'s else-if does.
* Post-pass: `pool.sand_storm_count += sand; pool.snow_dust = snow`
  (`minecart.rs:1146-1147`); the child-spawn sink drains through the
  verbatim `new_dust` + `no_gravity = true` + folded scale
  (`minecart.rs:1152+`, the `wants_*` sink precedent).
* RETIRED into the chain: the 213 hand-dispatch and `sand::type_268_arm`
  (+ its `solid_at` box stand-in) — `sand.rs:266-272` carries the retire
  note; the 268 arm regains the C#'s `SandStormCount++` (`:1791`),
  `rotation = 0f` (`:1798`) and the `!noLight` push gate (`:1804`).
* **G2 ONE-EDIT hunk (d54 `84eb739b`, research/131 §7.2a) applied:**
  `DustPhysicsEnv.snow_dust: &'a mut i32` (`dust_physics.rs:156`, after
  `sand_storm_count` `:152`); the 76 arm feeds `*env.snow_dust += 1`
  (`:666`, `Dust.cs:1747`); the d130 plumbing union rides —
  `DustPool::snow_dust` field (`particles.rs:235`) + ctor (`:260`) + the
  `:428` head reset inside `update_dust` (`:336`) + the tick landing
  (d130's :1069/:1126 shape). d130's `snow::type_76_arm` is NOT on this
  tree; the merge keeps d50's chain arm (the §7.2a verdict).
* Witness: `minecart.rs:2751`
  `w4_d50_the_wired_tick_lands_the_snow_sand_deltas_and_the_pre_move`
  (head reset 9→0, one live 76 → snow 1, the 299 pre_move ran BEFORE the
  move).

## 4. d54×g13 G3/G4 keep-one — the exact drop-hunks (d54i-bible §8.3.11)

**Both lanes ported the SAME GetAlpha 157/327 arms mid-flight** (d54
research/131 §8.1 row: "CLOSED ×2 — KEEP-ONE NOW OWED"): d54 `641561c2`
(`DustDrawGlobals` + `get_alpha_with_globals`, 2 witnesses) and g13
`daf7b7804` (`get_alpha_mouse_text` + `get_alpha_disco`, 3 witnesses incl.
the tint-census pin). Same verbatim maths (both pin m=255→94 + the 327
zero-quirk). **The keep-one is ALREADY EXECUTED on g13** — `8fd62bd7d`
(LIGHTS RETIRE phase 1) retired g13's split seams into the d54-canonical
blob; research/70 §6 carries the per-fn dedupe table. d50's role is the
third-party receipt + the exact drop-hunk publication for the d54i merger.

**d50's position: NO CONTEST.** `dust_table.rs` on this branch is
byte-identical to the merge-base (`git diff a725acb04 HEAD --
crates/tr-core/src/dust_table.rs` EMPTY; blob `a0c5ce4d5` at both). d50
never touched the file — the 157/327 drops stay documented drops here
(`dust_table.rs:255-256` comment) and the keep-one lands via the d54×g13
merge, not via d50.

**Blob identities (re-verified at this retire, origin tips re-fetched):**

| Item | Blob |
| ---- | ---- |
| d54 tip `38bfd1d22` `dust_table.rs` (= `1d16ef4e`'s file, unchanged since) | `58305d0a8325fb3e631c5afdb6334fc90142b09e` (779 lines) |
| g13 retire tip `eef0202cc` `dust_table.rs` | `300c2647ad724c506051ce26e6c8e39fcbced040` (874 lines) |
| g13 pre-retire `daf7b7804` `dust_table.rs` (the loser shape) | (889 lines; the drop-hunk source below) |
| d50 HEAD `dust_table.rs` (= merge-base `a725acb04`) | `a0c5ce4d5feec43087996d442109065c1b327f3b` |

`git diff 58305d0a8 300c2647a --stat` = **98 insertions, 0 deletions** —
the g13 retire file is a STRICT SUPERSET of d54's (three add-hunks:
`@@ -29,6 +29,17 @@` the module keep-one note, `@@ -770,6 +781,28 @@` the
`FloatLog` rig, `@@ -797,4 +830,69 @@` the two census witnesses). The
merger takes the g13 blob `300c2647a` VERBATIM; d54's file needs no edit.

**The exact keep-one DROP-hunks** (what `8fd62bd7d` deleted from g13's
`daf7b7804` file — the merger must NOT resurrect these; line numbers are
1-based in the `daf7b7804` blob):

1. **`get_alpha_mouse_text`** — lines **314-334** (the 5-line doc comment
   `:314-318` + `pub fn get_alpha_mouse_text(d: &Dust, new_color: Color,
   mouse_text_color: u8) -> Color` `:319` through the closing `}` `:334`).
   The 157 arm (`Dust.cs:2627-2651`): `(int)mouseTextColor/100f − 1.6f`
   scale, +50, cap 255 per channel, uncapped `(int)(100*num7)` alpha.
   SUPERSEDED by `get_alpha_with_globals`'s 157 arm (d54 blob `:344+`,
   g13 retire blob same line — byte-identical).
2. **`get_alpha_disco`** — lines **336-382** (the 13-line doc comment
   `:336-348` + `pub fn get_alpha_disco<R: DustRng>(` `:349` through `}`
   `:382`). The 327 arm (`Dust.cs:2658-2676`): `!dayTime` gate + THREE
   `NextFloat` draws + the dead-raise quirk (`(int)(sum)*255`, sum < 1
   always → every channel 0, colour unchanged, draws still advance the
   stream). SUPERSEDED by the same canonical seam's 327 arm.
3. **`w4_g13_get_alpha_157_the_mouse_text_pulse`** — lines **743-776**
   (doc `:743-745`, `#[test]` `:746`, fn `:747-776`). SUPERSEDED by
   `w4_d54_g3_get_alpha_157_mouse_text_pulse` (kept, byte-identical on
   both lanes; g13 retire blob `:757`).
4. **`w4_g13_get_alpha_327_night_draws_three_and_never_raises`** — lines
   **797-848** (doc `:797-799`, `#[test]` `:800`, fn `:801-848`).
   SUPERSEDED by `w4_d54_g4_get_alpha_327_night_draws_and_zero_quirk`
   (kept; g13 retire blob `:811`).

**KEPT additive (g13-only, no d54 counterpart — take with the blob):**
the `FloatLog` rig (`daf7b7804:778-795` → retire blob `:788-809`, feeds
the census witness), `w4_g13_globals_seam_gate_census` (retire blob
`:839`, non-327/fullBright 0-draw + zero-disco 3-draw pins over the
canonical seam), `w4_g13_draw_tint_census_pins` (retire blob `:864`, the
research/70 §1 8-consumer composition pins), the module keep-one note.
The anchors row re-points at `pub fn get_alpha_with_globals` (the retired
`pub fn get_alpha_disco` marker would fail the canary) — `8fd62bd7d`
already did this on g13.

**Witness arithmetic (research/131 §8.4):** d54 +2 vs g13 3 = 5 gross →
3 net (the −2 deduction stands; the two retired g13 witnesses are the
drop). The draw-pass wiring contract is g13 research/70 §7 (the
`Main.cs:38973` tint-feed compose) — the draw lane's, not d50's.

## 5. flag-b strip / flag-c retire — the d54-bible owed rows, RECEIPTED

d54 research/131 §8.1 rows "g13 flag-b pre_move 35 strip — OWED (d50
wiring wave)" and "g13 flag-c dust_lights decrement retire — OWED (same
wave)" both flip **EXECUTED** at this branch:

* **flag-b** = §2 above (`ea749c5ae`). The g13 §6 receipt's three probes
  (the `:170-172` arm present, the `lava_bubbles` parameter, the
  `tick_physics` harness double-count) are all resolved: arm gone,
  parameter dropped, the full-pool harness's outer pre_move loop dropped
  (the skeleton owns it now — ANALYSIS wiring-resume witness deltas).
* **flag-c** (`807350de0`, research/69 §5 arm-by-arm in `dust_lights.rs`):
  - 206: `(d.scale - 0.1) * 0.4` → `d.scale * 0.4` (`:408`).
  - 174: `cap(d.scale - 0.01, 0.6)` → `cap(d.scale, 0.6)` (`:447`).
  - 230: recovers the PRE-increment `(d.scale - 0.02) * 0.6` (`:320` —
    the C# light precedes `:639`).
  - 309: recovers the PRE-shrink `d.scale / 0.97` (`:218` — the light
    precedes `:486`).
  - 261: recovers the PRE-drift scale (`:352-356` — net 0 when
    `noGravity && fadeIn == 0`, else `+0.0025`; the light precedes
    `:695-704`). **Residual boundary (documented, ANALYSIS):** 261's
    fadeIn gate reads the PRE-arm fadeIn in C#; the chain's `:2114` ramp
    may zero it after the arm — a POST record at `fadeIn == 0` that took
    the ramp with non-zero PRE fadeIn is indistinguishable; the recovery
    treats it as the net-0 case (the dominant shape; 261 spawns fadeIn 0).
  - The 5-key witness set re-pinned on POST-update records:
    `the_post_decrement_arms_read_the_decremented_scale` (`:710`,
    206+174), `w4_d50_the_230_recovers_the_pre_increment_scale` (`:736`),
    `w4_d50_the_261_recovers_the_pre_drift_scale` (`:748`),
    `the_shimmer_spark_rides_its_own_colour` (`:778`, the 309 key).
* **The g13 `dust_lights_b.rs` replays are NOT on this tree** (verified:
  `Test-Path crates/tr-app/src/dust_lights_b.rs` = False; `main.rs` has
  only `mod dust_lights;`). Their arm-by-arm retire rides the g13×d50
  merge (research/69 §5 second paragraph) — §6 publishes the exact keys.

## 6. The lights_b replay-retire keys (the g13×d50 merge contract)

g13's `dust_lights_b.rs` (blob `da47b7d6d` at tip `eef0202cc`) replays d50
`type_chain` mutations on local copies. Its cites point at the OLD d50 tip
`2e243a2d` line numbers; the file moved at the wiring commits. **The
retire keys re-pinned at d50 HEAD `e84e6bd76`** (`dust_physics.rs` blob
`60bc9112e`):

| g13 replay (lights_b line) | What it replays | d50 HEAD arm (the retire key) | POST-record read after retire |
| -------------------------- | --------------- | ----------------------------- | ----------------------------- |
| `light_scale` 66/267 `+0.02` (`:254`) | the spin-arm grow BEFORE the light | `:603`/`:615` (`Dust.cs:1578`/`:1607`) | `d.scale` direct (post-grow) |
| `light_scale` 20/21/231 `+0.005` (`:255`) | the grow before the light | `:622` (`:1624`) | `d.scale` direct |
| `light_scale` 41 `+0.007` (`:256`) | the jitter-arm grow | `:854` (`:2035`) | `d.scale` direct |
| `light_scale` 44 `+0.0085` (`:257`) | the jitter-arm grow | `:871` (`:2063`) | `d.scale` direct |
| `arm_27_45_scale` (`:192-216`) | the 27 burn-down ladder + `+0.002` + noLight double `−0.06` | `:628-648` (`:1647-1678`) | `d.scale` direct (the chain ran all of it) |
| `arm_35_wet_scale` (`:234-244`) | the dry `scale=0` kill + the `alpha>255` kill + `−0.01` | `:805-831` (`:1953-1996`) | `d.scale` direct — **the `alpha == 255` boundary RESOLVES**: the chain's `rng.next_max(2)` (`:810`) wrote the post-kill scale; the collector's draw-independent mirror retires wholesale |
| `light_scale` 35 noGravity `+0.03` (`:262-264`) | the noGravity grow | `:777` (`:1877`) | `d.scale` direct |
| `ramp_scale` (`:177-183`, second-chain types) | the fadeIn ramp `+0.03` / burn-out `−0.01` | `:888-926` (`:2083-2129`) | `d.scale` direct (post-ramp) |
| `light_position` 27/45 push (`:283-294`) | the noLight player-push BEFORE the light | `:649-654` (`:1679-1686`) | `d.position` direct (post-push) |
| the 267 `noLight` lift gate (`:434` "d50 `:603-605`") | the `noLight = false` lift | `:616-618` (`:1613-1616`) | the POST record already carries `no_light == false` — the gate replay retires |

After the retire, `light_scale`/`light_position` collapse to identity
reads (`d.scale`/`d.position`) for every arm — the collector becomes a
pure POST-record reader exactly like `dust_lights.rs` (§5). The 23
per-arm colour-pin witnesses re-pin on POST-update records (the same
re-pin shape as §5's 5-key set); the ramp special-cases (235/46/213/260/
304) never reach the second-chain types, so the generic ±keys above are
complete. **No RNG order changes** — the replays drew nothing.

## 7. The dust-type-table FAMILY declaration (d50's slice of the Dust.cs family)

d54 research/131 §8.2 declared the Dust.cs SPAWN family complete. This
section declares d50's slice — the **UpdateDust per-type physics +
type-table** family — RETIRED at `e84e6bd76` (+ this doc):

| C# region | Port | Status at HEAD |
| --------- | ---- | -------------- |
| `Dust.cs:420-466` head (resets, budget split, dCount, scale>10 kill, ChildSafety) | `particles.rs::update_dust` | LANDED (S1-E-3) + the `:428` snow reset union (`ea749c5ae`) |
| `:468-472` pre-move shrink/lift | `dust_physics::pre_move`, called from the skeleton | LANDED + WIRED (`ea749c5ae`) |
| `:473-476` lavaBubbles++ | the skeleton's canonical inline increment | LANDED (g13 transplant, strip §2) |
| `:477` move | skeleton | LANDED (S1-E-3) |
| `:478-2334` the per-type chain (FIRST ladder + standalone ifs + ramp + SECOND chain) | `dust_physics::type_chain` | LANDED (`cb07d2b66`) + WIRED (`807350de0`); every arm present incl. the dead 235 vector3 draws, the 159/164/173/160/162 child-spawn sink, 304 sine/ladder, 43 pre-ramp dark-kill, 27 burn-down, 76/268/270/271, 41/44 clamps, the second chain 130-134/278/219-223/226/331/272 with the tail-suppression bool |
| `:2335-2422` generic tails + post-loop recompute | skeleton | LANDED (S1-E-3); the `:2335` tail suppression via the hook bool |
| `GetAlpha`/`GetColor`/visual accessors | `dust_table.rs` | LANDED (S1-E-2); the 157/327 drops close via the d54×g13 keep-one (§4 — NOT d50's file) |
| the 94 AddLight sites | `dust_lights.rs` (42, S1-E-5) + g13 `dust_lights_b.rs` (52) | 94/94 ported (g13 §7); d50's flag-c retire EXECUTED (§5), the lights_b replay retire keys published (§6) |

**Deliberate drops (each cited at its arm, unchanged from the phase-1
declaration):** `customData` (the field is gone — the 31 `else if
(noGravity)` arm at `:1297` becomes reachable BECAUSE the NPC branch
`:1277` dropped); the 43 `light_color` probe reads the bright default
until the S1-E-5 sample seam (never spuriously dies); the 226
customData-Color light site stays absent wholesale (the G5 field drop).
The 235 homing arm is EXACT single-player (d54 G6 CLOSED-EXACT: one
producer, `NPC.cs:90977`, active-player index 0 only).

**What d50 does NOT own (cross-lane, for the d54i merger):** the
`main_dust.rs` three-way (g11×g13×d54, research/116 §6), the d130 snow/rain
folds, the g73 waterfall family, the trail trio, the G3/G4 keep-one blob
take (§4), the lights_b replay retire (§6), the draw-pass tint wire
(research/70 §7). d50's merge surface is exactly: `dust_physics.rs` (new),
`particles.rs` (head wiring + snow union), `minecart.rs` (tick hook),
`sand.rs` (268 retire), `dust_lights.rs` (flag-c), `app/frame/world.rs`
(dust_phase snapshot), `lib.rs` (mod), docs + anchors.

**Witness census at HEAD (machine-checked `#[test]` greps):**
`dust_physics.rs` 34 `w4_d50_*` (the phase-1 ANALYSIS "38" figure is
SUPERSEDED — the actual count at every commit `cb07d2b66`→`e84e6bd76` is
34; the phase-3 re-derivation corrected pins in place, no count change);
`minecart.rs` +1 (`w4_d50_the_wired_tick_…`); `dust_lights.rs` +2
(`w4_d50_the_230_…`/`w4_d50_the_261_…`) + 2 re-pinned
(`the_post_decrement_arms_…`, `the_shimmer_spark_…`); `particles.rs` +1
transplanted (`w4_g13_lava_bubbles_…`) + the golden witness's snow-reset
pin. **d50 lane total: 37 `w4_d50_*` + 1 `w4_g13_*` transplant = 38
witnesses, ALL WRITTEN NOT RUN.** The research/131 §8.4 union recount
(492 gross / ≈461 net) does not include d50's 38 — they ride the
merge-verify wave's compile+run like every other lane's.

## 8. Verification record (this retire)

* `git fetch origin --prune` — tips re-read: d50 origin `2e243a2d2` (the
  three wiring commits are LOCAL, pushed by this retire), d54
  `38bfd1d22`, g13 `eef0202cc`.
* `git status --porcelain` EMPTY at resume; `git log origin..HEAD` = the
  three wiring commits (the dead-clean phase-2 tip + the 12:31→13:07
  wiring window the brief's mt-frozen probe saw).
* C# re-reads: `Dust.cs:420-437` (head resets), `:460-484` (pre-move +
  move + chain head), `:1743-1752` (the 76 snowDust++), `:2083-2157` (the
  ramp + second-chain head), `:2620-2681` (the 157/327 GetAlpha arms).
* Rust re-reads at HEAD: `dust_physics.rs:160-289/583-1107` (pre_move,
  the chain arms, the second chain), `particles.rs:319-398/2095-2169`
  (the head wiring + the transplanted witness), `minecart.rs:1060-1159`
  (the tick), `dust_lights.rs:1-100/200-459/700-799` (the retire + the
  witnesses), `sand.rs:260-299` (the 268 retire note),
  `app/frame/world.rs:295-374` (dust_phase).
* Blob diffs: `a725acb04..HEAD -- dust_table.rs` EMPTY;
  `58305d0a8..300c2647a` = 98 insertions / 0 deletions (the strict
  superset, §4); the `daf7b7804` drop-hunk spans re-read line-by-line.
* Registry scan: all `origin/task/*` tips' `docs/research/` ls-trees —
  237-242 taken, ≥243 free (§0).
* Code-only: **no cargo/tests/rustfmt/graphify invoked.** The witness
  compile window is the merge-verify wave's (research/131 §8.3.13).

**RETIRED.** The d50 ledger is closed: pre_move-strip LANDED (§2),
type_chain-wire LANDED (§3), the G3/G4 keep-one drop-hunks PUBLISHED
(§4 — the take is g13's blob, d50 uncontested), flag-b/flag-c RECEIPTED
EXECUTED (§5), the lights_b replay keys PUBLISHED for the merge (§6),
the family declaration made (§7). Nothing owed by d50 except the push.
