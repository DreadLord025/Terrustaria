# 67 — Cage-animator triple-dup reconcile (merger assist)

> Wave 4 · 2026-11-21 · branch `task/W4-g34-drawdata-wire-2` @ 16f88fb7 ·
> code-only (no cargo/tests/graphify; witnesses WRITTEN NOT RUN everywhere) ·
> merger-assist doc: NO code changes here, other lanes untouched.
>
> **STATUS: union EXECUTED on the g34 branch (2026-11-21).** The KEEP
> recommendation (§2) and this branch's DROP items (§3) are LANDED here —
> see §6 for the per-phase commit record. **The merger keeps THIS branch's
> cage wiring and drops the f14/g33 wiring wholesale** (their
> `drawdata_cages.rs` modules, f14's dispatcher/key slot, g33's live-stream
> warm-up); their surviving contributions (f14's composite order + warm-up
> clone semantics, g33's 92-id gate + witness set) are already folded in.
>
> Three branches independently ported `Main.AnimateTiles_CritterCages`
> (`Main.cs:13511-16596` + `UpdateOwlCageFrame` `:16617-16666` +
> `UpdateDragonflyJarFrame` `:16668-16727`) and wired cage draws. This doc
> is the arm-by-arm 3-way comparison, the KEEP/DROP recommendation, the
> witness-supplement union, and the warm-up init RNG-order census (g34's
> documented residual) with the exact C# init position for the merger.

## The three implementations

| Branch | Tip | Animator module | Loops | Draw ids wired | Warm-up |
| --- | --- | --- | --- | --- | --- |
| `task/W4-f14-cage-furniture-wire` | 50046fe1 | `tr-core/src/drawdata_cages.rs` — `CritterCageAnimator` (2489 lines, 10 witnesses) | "31 loop bodies" = 31 distinct bodies; byte-identical variants share helpers (squirrel≡orange, mallard≡duck, bird≡blue≡red, snail/snail2 one parameterised body), so the driver still runs **34 tables** in C# order | all 92, via `furniture_post_pass` (cage half; chest `None`, W4-d58 seam) | **wired**: `new(world_seed)` replays 3600 ticks on a FRESH world-seed `UnifiedRandom` clone; live stream untouched |
| `task/W4-g33-drawdata-wire-1` | 97704d01 | `tr-core/src/drawdata_cages.rs` — `CageAnimator` (3140 lines, 40 witnesses) | 34 tables in C# order (same keep-one folds) | all 92, via `cage_draw_data` | **wired**: `App` construction calls `warm_up(app.rngs.global())` — consumes the LIVE world-seeded stand-in stream (documented deviation) |
| `task/W4-g34-drawdata-wire-2` (this) | 16f88fb7 | `tr-core/src/critter_cage_anim.rs` — `CritterCageAnimator` (2525 lines, 12 witnesses) | 34 tables in C# order, whole-port (one `animate(critter_cage, rand)` driver) | 50 (lane 2: d64 batches 2-3), via `cage_draw_data` + `is_lane2_cage_kind`; flag raised for ALL 92 (`is_cage_kind`) | **NOT wired**: `warm_up()` exposed, App init untouched (residual — see §4) |

All three share the base `a725acb0` (g33/g34) / the f14 base; all three
tick at the `AnimateTiles` tail (`Main.cs:19303`) inside the SAME
`tile_anim_step` closure on the SAME `Main.rand` stand-in
(`app/frame/world.rs` g33/g34, `app/frame.rs` f14), gate on
`Main.critterCage` (`:13513-13516`), clear the flag at the walk head
(`TileDrawing.cs:474` fused-layer stand-in) and set it per cage cell
(`:5672` etc.), and mix `anim_key()` into the G6 key view-gated on cage
kinds. None resets the cage tables per world EXCEPT g34 (see DROP list).

Line references below are to each branch's own tip file
(`git show <branch>:crates/tr-core/src/...`), C# refs to
`decompiled/Terraria/Main.cs` / `.../Drawing/TileDrawing.cs`.

## 1. Arm-by-arm 3-way quirk table

All six census quirks are **PRESENT in all three branches** — each port
was written verbatim against the C#. The table records where, and the
structural differences that matter to the merger.

| Quirk (C# cite) | Behaviour | f14 `drawdata_cages.rs` | g33 `drawdata_cages.rs` | g34 `critter_cage_anim.rs` |
| --- | --- | --- | --- | --- |
| **penguin `> 3`** (`:15166`, 10/11 band) | `frame++` then `> 3` (copy-paste, not `> 11`) → the `Next(3)` roll fires EVERY 5-tick pass from 10/11; frames never reach 12 via this band | PRESENT — `step_penguin` :1179-1183 (`if t.frame[i] > 3` + comment) | PRESENT — `step_penguin` :1240-1247 + witness `penguin_10_11_band_keeps_the_gt3_quirk` | PRESENT — `step_penguin` :1158-1166 (`:15166` comment) |
| **frog `> 17`** (`:15711`, 18-23 band) | the `> 17` wrap sits OUTSIDE the 5-tick gate → frames 18-23 re-roll `Next(3)` (→6/18) EVERY tick | PRESENT — `step_frog` :1466+, 18-23 arm ends `if t.frame[i] > 17 { …6/18 }` outside the gate | PRESENT — `step_frog` :1519+ with `:15711` comments + witness `frog_18_23_band_rerolls_the_wrap_every_tick` | PRESENT — `step_frog` :1531-1540 (`:15711` comment, check outside the `>= 5` gate) |
| **fishbowl `Next(12)`** (`:16332`, mode-0 frame-10 arm) | bubble jump rolls `Next(12)` (NOT `Next(20)`), and the 11-jump does NOT clear the mode | PRESENT — `step_fish_bowl` :1969 (`next_max(12)`) | PRESENT — :2031 + witness `fish_bowl_mode0_frame10_bubble_uses_next12_not_next20` | PRESENT — :1995-2003 (`:16332` comment: Next(12) not Next(20), no mode clear) |
| **owl reseed** (`:16619-16625`) | `counter%40 != 0 \|\| Next(80)==0` short-circuit (roll only on 40-boundaries) + `(counter+1)%40==39 → counter = 40*Next(4)` reseed; frame is a pure counter function | PRESENT — `update_owl_cage_frame` :1356-1365 + witness `w4_f14_the_owl_frame_is_the_counter_function` | PRESENT — `step_owl` :1412-1420 + witness `owl_counter_cycle_maps_the_four_phases` | PRESENT — `step_owl` :1323-1331 |
| **snail double-`Next(3)`** (`:15490-15497` snail / `:15563-15570` snail2) | TWO identical `else if (frame==1 && Next(3)==0)` clauses → 19 then 20; the second roll draws ONLY when the first missed | PRESENT — parameterised `step_snail` :1422-1426, both clauses, snail2 rides the same body (30,2700)/(45,5400) | PRESENT — :1475-1479 + witness `snail_hide_band_swaps_19_20_and_can_exit_to_2` | PRESENT — `step_snail` :1376-1380 AND `step_snail2` :1431-1435 (two explicit bodies, same clauses) |
| **butterfly backward** (`:16398-16525`, mode `>= 10`) | mode≥10 walks rows BACKWARD (`frame--`, wraps 7/14) with ±8 landmark hops at 1/4/6 & 9/12/14, `Next(20)` hop / `Next(6)` mode-flip chain; PLUS the unconditional per-tick base `Next(3,16)` (`:16411`) drawn even when the mode overrides it | PRESENT — `step_butterfly` :2020-2117 (`m >= 10` reverse branch :2045+, base roll :2031, `:16411` doc note) | PRESENT — :2086-2200 (base roll :2101, reverse :2115+) + witnesses `butterfly_base_speed_draw_is_unconditional`, `butterfly_forward_flap_wraps_and_can_jump_bands` | PRESENT — `step_butterfly` :2052-2151 (base roll :2066, reverse `mode[i] >= 10` :2079+, wraps 7/14, ±8 hops, `Next(20)`/`Next(6)` chain) |

Secondary verbatim quirks also checked, present in all three:

| Quirk | f14 | g33 | g34 |
| --- | --- | --- | --- |
| duck/mallard frame-1 `Next(3)` short-circuit (`:14397`) | ✓ `step_duck` :614+ | ✓ `step_duck` :641+ | ✓ `step_mallard_duck` :560+ (one body, two tables) |
| ladybug UNCONDITIONAL base `Next(10,12)` (`:15990`) | ✓ `step_ladybug` | ✓ + witness `ladybug_base_limit_draw_is_unconditional` | ✓ `step_ladybug` (base `next_range(10,12)` before the state arms) |
| jellyfish separate-ifs mode chain (1→2 promotion re-gates in-tick) | ✓ `step_jellyfish` | ✓ + witness `jellyfish_mode_arms_are_separate_ifs` | ✓ `step_jellyfish` + witness `jellyfish_mode_and_frame_bands_hold` |
| orange-squirrel stream-only table (ticks though no tile reads it) | ✓ separate table + driver call | ✓ | ✓ `step_squirrel(&mut self.squirrel_orange, …)` |
| fairy-jar slot-0-only cycle, slots 1.. derive RNG-free (`:16197-16227`) | ✓ `step_fairy_jar(_rand)` + witness | ✓ + witness | ✓ + witness `fairy_jar_slot0_cycles_and_slots_derive_without_rng` |
| dragonfly weighted picks `NextFromList(1,5,9)`/`(0,5,5,9,9)` + counter/4%4 triangle (`:16668-16727`) | ✓ `update_dragonfly_jar_frames` | ✓ `step_dragonfly` + witness | ✓ `step_dragonfly` (`next_from_slice`) |
| slug per-row wraps 24/31/34 (`:15859-15885`) | ✓ `[24,31,34][r]` | ✓ inline + witness | ✓ `step_slug(wrap)` + witness `slug_rows_wrap_at_24_31_34` |

Driver order: all three run the 34 tables in the exact C# source order
bunny → squirrel → squirrelOrange → mallard → duck → grebe → seagull →
bird → blueBird → redBird → macaw → scorpion(2) → penguin → turtle →
seahorse → pufferfish → owl(:15476) → snail → snail2 → frog → mouse →
worm → slug(3) → grasshopper → maggot → ladybug → rat → waterStrider →
fairyJar → fishBowl → lavaFishBowl → butterfly(9) → dragonfly(7,:16526)
→ jellyfish(3). Row-major within each 2-D table. **No RNG-order
divergence between the three animators for the same seed + gate
sequence** (f14/g33 use per-slot bodies with `return`, g34 per-table
loops with `continue` — equivalent control flow).

Structural (non-quirk) differences the merger must choose between:

| Aspect | f14 | g33 | g34 |
| --- | --- | --- | --- |
| Gate flag ownership | `critter_cage` field INSIDE the animator | field inside `CageAnimator` | separate `App::critter_cage` bool + `animate(critter_cage, rand)` param |
| `anim_key` coverage | FRAME tables only (counters/modes excluded as draw-irrelevant) | frames + counters + modes | frames + counters + modes |
| G6 key slot | dedicated `WorldFrameKey::cage_anim`, gated `is_cage_kind` | FNV-mixed into `tile_anim`, gated `is_cage_kind` (92) | FNV-mixed into `tile_anim`, gated `is_lane2_cage_kind` (50) |
| Draw call | `furniture_post_pass` (cage+chest dispatcher, chest `None`) | `cage_draw_data` direct | `cage_draw_data` direct, lane-2 gate |
| **Composite order** | **AFTER `draw_data` + slice B, BEFORE slice A** (`apply_cases_a`) — the exact FIRST-switch tail (`TileDrawing.cs:5629-5907`); the 568/569/570/580 glow rects read `tileFrameY + addFrY` POST-cage (`:6085-6104`) | AFTER `draw_data`, BEFORE `apply_world_corrections`/`cases_b`/`cases_a` | **AFTER `apply_cases_a`** — tiles.rs :748 glow, :762 cage rewrite; witness `the_lane2_cage_rewrite_feeds_from_the_cage_animator` even PINS `cage > glow` (:1957-1959). The glow rects for 568-570/580 therefore read the PRE-cage addFrY — a real deviation vs the C# second switch |
| World-entry lifetime | PROCESS state, NO reset (correct: `Main.cs:8393-8394` zeroes `tileFrame` only; cage arrays allocated once in the STATIC ctor `:66901-66972`) | PROCESS state, NO reset (correct) | **RESETS**: `enter_loaded_world` calls `cage_anim.reset()` (app/mod.rs :2329-2330) citing ":66902-66972 re-allocates" — WRONG: that allocation is `static Main()` (`:66502`), runs once per process, not per world |
| Warm-up | wired in `new(world_seed)` on a fresh clone | wired at App construction on the LIVE global stream | exposed, NOT wired (residual, §4) |

## 2. KEEP recommendation

**KEEP g34's `critter_cage_anim.rs` whole-port as the surviving animator
module, union the 92-id draw wiring, and take f14's slice-order fix.**

1. **Animator: g34 `crates/tr-core/src/critter_cage_anim.rs` (whole).**
   Rationale: (a) it is the only one whose module name/anchors/PORT_MAP
   rows on THIS branch already point at it (the anchors canary reads
   `critter_cage_anim.rs|pub fn animate`); (b) the whole-port argument is
   load-bearing — the 34 machines share ONE `Main.rand` stream, g33-half
   tables draw before g34-half tables, so a halved animator desynchronises
   every downstream draw (research/66 §1); (c) the per-table `for i in
   0..N` loop shape matches the C# driver most directly and the
   `(critter_cage, rand)` parameter form keeps the gate flag in App state
   where the fused-layer draw can raise it for all 92 kinds. All six
   quirks are verbatim (§1). f14's and g33's `drawdata_cages.rs` are
   behaviourally equivalent animators — dropping them loses NO machine.
2. **Draw wiring: the UNION — all 92 ids, one `cage_draw_data` call site.**
   g34's `is_lane2_cage_kind` gate must be widened to `is_cage_kind`
   (both predicates already live in `drawdata_furniture.rs` on this
   branch; `lane2_cage_split_is_disjoint_and_exhaustive` pins 92 = 50+42).
   g33's `wired_cage_rewrite_matches_the_static_arms_at_t0` covers all
   92 ids and is the parity witness to keep. Keep `cage_draw_data` (not
   f14's `furniture_post_pass` dispatcher) — the chest half belongs to
   the W4-d58 `CacheSpecialDraws_Part2` seam (`TileDrawing.cs:993`), not
   this site; f14 itself passes `chest_frame: None` there.
3. **Composite order: f14's slice-B/A fix (the load-bearing seam).**
   Move the cage rewrite from its current post-`apply_cases_a` position
   (g34 tiles.rs :762) to the FIRST-switch tail: after `draw_data` +
   slice B's `case_draw_data`, BEFORE `apply_cases_a` — f14's exact
   placement (`w4_f14_the_cage_post_pass_rides_the_first_switch_tail`).
   C# evidence: the cage groups are arms of the FIRST `GetTileDrawData`
   switch (`TileDrawing.cs:5629-5907`); the SECOND switch's glow rects
   for the firefly jars 568/569/570 and butterfly 580 read
   `tileFrameY + addFrY` POST-cage (`:6085-6104`), and the `:1007`
   source-rect resolve reads the final `addFrX/addFrY`. g33's placement
   (before `apply_world_corrections`) also precedes slice A but runs
   before slice B; prefer f14's after-slice-B position — slice B owns
   first-switch cases (349/441/468 probes) that the C# runs before the
   cage groups. g34's current witness assertion `cage > glow` must be
   INVERTED when the move lands (it currently pins the deviation).
   Interaction to re-check at merge: g34's a80da26a 564 fix (pre-clear
   `tileFrame*36` rect, clear riding `cases_a`'s `add_fr_y_delta`) is
   independent of the cage position (564 is not a cage kind) — keep it.
4. **G6 key: one cage contribution, gated on `is_cage_kind` (all 92).**
   g34's lane-2-only gate would leave g33-half cages (e.g. bunny 275,
   penguin, owl) animating with a stale cache key once the union wiring
   lands. Keep the FNV-mix-into-`tile_anim` form (g33/g34) rather than
   f14's dedicated `cage_anim` slot — fewer `WorldFrameKey` fields, same
   coverage; keep g34's frames+counters+modes `anim_key` (superset of
   f14's frames-only; over-keying is safe, under-keying risks stale
   pixels when a mode byte changes the next frame's cadence).
5. **Lifetime: PROCESS state, NO world-entry reset.** Drop g34's
   `enter_loaded_world` reset (see DROP). The C# `tileFrame`/
   `tileFrameCounter` zero loop at `:8387-8395` is ctor-time init (the
   tail of `Initialize_TileAndNPCData2`, called once from the ctor at
   `:6610`) and covers `tileFrame`/`tileFrameCounter` ONLY; the cage
   arrays are allocated once in `static Main()` (`:66502`,
   `:66901-66972`) and never re-zeroed per world.
6. **Warm-up: wire g34's `warm_up` with f14's fresh-clone semantics.**
   f14's `new(world_seed)` clone keeps the live stream bit-for-bit until
   a cage enters the view; g33's live-stream warm-up shifts EVERY
   downstream `Main.rand` consumer (a much larger determinism blast
   radius). Take the clone form; §4 pins the exact C# init position and
   the residual RNG-order caveat for the merger.

## 3. DROP list

| Drop | From | Why |
| --- | --- | --- |
| `crates/tr-core/src/drawdata_cages.rs` (both variants) + its `lib.rs` registration | f14 AND g33 | behaviourally equivalent to the kept g34 animator (§1: all quirks present in all three); keeping two modules doubles the anchors/maintenance surface. Port f14's `w4_f14_*` and g33's per-machine witnesses over first (§5) |
| f14's `furniture_post_pass` draw call site (cage+chest dispatcher) | f14 tiles.rs | replace with the direct `cage_draw_data` call (g33/g34 form); the chest half stays with W4-d58's `:993` seam. `furniture_post_pass` itself may stay as a library fn but must have exactly ONE live caller shape after the merge |
| f14's dedicated `WorldFrameKey::cage_anim` field | f14 app/mod.rs :1141 | superseded by the FNV-mix into `tile_anim` (KEEP §2.4); two key slots for one input invites double-counting |
| g33's lane split (`is_cage_kind` static probe on `CageAnimator`) | g33 drawdata_cages.rs | the predicates live in `drawdata_furniture.rs` on g34 (canary rows exist); the animator-local copy goes with the module |
| g34's `enter_loaded_world` cage reset (`self.cage_anim.reset(); self.critter_cage = false;`, app/mod.rs :2329-2330) | g34 (THIS branch — merger must remove) | NOT C# behaviour: cage tables are process state (§2.5). The comment's ":66902-66972 re-allocates in Initialize_TileAndNPCData" mis-cites the STATIC ctor. Keep the `tile_anim.reset()` beside it (that one IS `:8393-8394` + `:67242`) |
| g34's lane-2-only draw gate + lane-2-only G6 gate | g34 tiles.rs :762, frame/draw.rs :1023-1033 | widen both to `is_cage_kind` (92) per §2.2/§2.4; keep `is_lane2_cage_kind` + its split witness as documentation of the census boundary |
| g34's post-slice-A cage position + the `cage > glow` witness assertion | g34 tiles.rs :762, :1957-1959 | replaced by f14's slice-order fix (§2.3); the assertion must flip to `cage < cases_a` |
| g33's live-stream warm-up call (`app.cage_anim.warm_up(app.rngs.global())`, app/mod.rs :4095) | g33 | replaced by the fresh-clone form (§2.6) — consuming the live global stream at construction shifts every later `Main.rand` draw vs the pre-wiring baseline |

NOT dropped (explicitly kept from the "losing" branches): f14's
composite-order placement + its order witness; g33's 40 per-machine
witnesses and the all-92 t0 parity witness; f14's warm-up-clone witness;
g33's documented limitations register (research/65 — 620 legacy lid pass
unwired, negative-probe clamp, fused-layer flag-clear note) — all still
true of the merged result.

## 4. Warm-up init RNG-order census (g34 residual → exact C# position)

g34 exposes `warm_up()` but does not wire it (research/66 residual:
"needs the whole init's RNG-order census"). The census, from the
decompiled source:

### The exact C# init position

* `Main.rand` is created in the **instance ctor** `public Main()`
  (`Main.cs:6348`): `:6461-6463`
  `if (Main.rand == null) Main.rand = new UnifiedRandom((int)DateTime.Now.Ticks);`
* First draw: `:6467` `Main.lo = Main.rand.Next(6);`
* Then `WorldGen.RandomizeBackgrounds(Main.rand)` at `:6594`, re-rolled
  at `:6597` and `:6601` while `treeBGSet1[0] == 173` (0-2 extra calls;
  `WorldGen.cs:7947` — the forest-style `Next(14)` ladder + corruption/
  jungle/hallow/snow/desert/ocean/mushroom rolls).
  `RandomizeCaveBackgrounds` (`:6603`) draws `WorldGen.genRand`, NOT
  `Main.rand`.
* `Main.Initialize_TileAndNPCData1()` at `:6609` (body `:8433-10608`) —
  **zero `Main.rand` draws** (grep-verified across the whole body).
* **`Main.Initialize_TileAndNPCData2()` at `:6610`, body `:6800-8424`.
  The warm-up is the FIRST statement of the body:**

  ```csharp
  // Main.cs:6800-6807
  private static void Initialize_TileAndNPCData2()
  {
      Main.critterCage = true;
      for (int i = 0; i < 3600; i++)
      {
          Main.AnimateTiles_CritterCages();
      }
      Main.critterCage = false;
      // :6808+ — pure tile-flag tables (tileBlockLight/tileSolid/
      // tileSolidTop/tileMerge/…); the tileFrame/tileFrameCounter zero
      // loop rides at :8387-8395; NO further Main.rand draws anywhere
      // in the body (grep-verified :6808-8424).
  }
  ```

* After it: `Main.Initialize_Items()` (`:6611`, body `:6732-6798`) —
  zero `Main.rand` draws; the ctor tail to `:6731` — zero draws. The
  next `Main.rand` consumers are `PostContentLoadInitialize`
  (`:10864`, called from `LoadContent` at `:52573`): `moonType Next(9)`
  `:10873`, `windSpeedCurrent Next(-800,801)` `:10874`,
  `numClouds Next(200)` `:10876`, … — i.e. **the warm-up's 3600 ticks
  sit BETWEEN the background randomisation and the sky/cloud init on
  the one process stream**.
* The cage tables themselves: allocated zeroed ONCE in `static Main()`
  (`:66502`), `cageFrames = 25` at `:66901`, arrays `:66902-66972`.
  Never re-allocated per world.

### Merger instruction

1. Wire `CritterCageAnimator::warm_up` at **App construction** (both
   constructors — `app/mod.rs` `App::new`/the flow.rs swap-world twin
   that already initialise `cage_anim`), BEFORE any world-load or
   ambience draw touches the global stream — the port's stand-in for
   the ctor position `:6610`.
2. Use **f14's fresh-clone semantics**: `warm_up(&mut UnifiedRandom::new(world_seed))`
   — NOT g33's live `rngs.global()` call. The C# stream is
   `DateTime.Now.Ticks`-seeded process state (already a documented
   stand-in deviation, `app/rng.rs`); replaying on a clone keeps the
   port's live-stream interleave bit-for-bit with the pre-wiring
   baseline, at the cost of the cage tables' post-warm-up state not
   matching a hypothetical exact-stream replay. Record that as the
   deviation (f14's module doc already words it).
3. **Residual caveat for the verify wave**: an EXACT-stream port would
   need the ctor draws before `:6610` replayed in order — `Next(6)`
   (`:6467`) + 1-3 `RandomizeBackgrounds` passes (`:6594-6601`, the
   173-reroll condition) — and would then shift `moonType`/`wind`/
   `numClouds` (`:10873-10876`) and every later `Main.rand` consumer.
   That is a whole-init determinism project, NOT a cage-lane change;
   until then the clone form is the documented approximation. The
   3600-tick count, the forced-true gate and the closing `false`
   (`:6802`/`:6807`) are exact in all three ports' `warm_up`.
4. Do NOT add a world-entry warm-up or reset (§2.5/§3) — the C# runs it
   once per process, before any world exists.

## 5. Witness-supplement union (all WRITTEN NOT RUN, code-only)

The kept g34 module has 12 witnesses; the dropped modules carry 50 more
plus 9 app-side wiring witnesses. Port these over (renaming
`w4_f14_*` → the kept module's convention, retargeting
`drawdata_cages` → `critter_cage_anim`, `CageAnimator` →
`CritterCageAnimator`, and the gate-field accesses to the
`animate(bool, rand)` parameter form):

### From g33 `drawdata_cages.rs` (40) — the per-machine quirk pins

`frame_of_reads_the_right_table_and_row`,
`cage_kind_set_matches_the_furniture_table` (retarget to
`drawdata_furniture::is_cage_kind`),
`macaw_walk_band_uses_gt5_not_ge5`,
`scorpion_frame7_snaps_to_zero_without_rolls`,
`penguin_10_11_band_keeps_the_gt3_quirk`,
`turtle_29_resets_and_15_can_jump_to_30`,
`seahorse_swim_band_limit_6_and_24_reset`,
`pufferfish_swim_wraps_40_to_0_and_inflate_reads_post_step`,
`pufferfish_inflate_cycle_returns_to_the_swim_entry`,
`owl_counter_cycle_maps_the_four_phases`,
`snail_hide_band_swaps_19_20_and_can_exit_to_2`,
`frog_18_23_band_rerolls_the_wrap_every_tick`,
`mouse_idle_band_can_hop_to_4_after_the_wrap`,
`worm_frame9_coin_and_18_wrap`,
`slug_rows_wrap_at_24_31_34` (g34 already has an equivalent — keep one),
`grasshopper_hop_bands_mirror_the_mouse_shape`,
`maggot_idle_0_picks_13_or_1_and_12_resets`,
`ladybug_base_limit_draw_is_unconditional`,
`rat_sleep_band_ladder_and_groom_mod10`,
`water_strider_15_18_reset_and_21_wrap`,
`fairy_jar_slots_derive_from_the_master_with_offsets` (g34 equivalent exists),
`fish_bowl_mode0_frame10_bubble_uses_next12_not_next20`,
`lava_fish_bowl_gate_draws_every_tick_and_wraps_26` (g34 equivalent exists),
`butterfly_base_speed_draw_is_unconditional`,
`butterfly_forward_flap_wraps_and_can_jump_bands`,
`dragonfly_flap_ladder_and_early_exit`,
`jellyfish_mode_arms_are_separate_ifs`,
`warm_up_runs_3600_gated_calls_and_clears_the_flag` (adapt: g34's gate is
a parameter — assert the caller-side flag dance instead),
`full_animate_is_order_stable_and_seed_reproducible` (g34's
`same_seed_replays_bit_equal_and_the_key_tracks_state` overlaps — keep
the stronger),
`zero_tables_stay_the_static_arms_until_animated`.

### From f14 `drawdata_cages.rs` (10)

`w4_f14_the_gate_returns_before_any_draw` (g34 equivalent exists),
`w4_f14_one_tick_is_deterministic_and_consumes_the_caller_stream`,
`w4_f14_the_warmup_replays_on_a_clone_not_the_live_stream` (**the §2.6
semantics witness — must ride the merged warm-up wiring**),
`w4_f14_anim_key_tracks_the_frame_tables` (adapt to the
frames+counters+modes key),
`w4_f14_frame_of_routes_table_and_row` (g34 equivalent exists),
`w4_f14_the_tables_feed_the_census_cage_rewrite`,
`w4_f14_the_owl_frame_is_the_counter_function`,
`w4_f14_the_fairy_jar_slots_are_slot0_offsets` (g34 equivalent exists),
`w4_f14_the_pufferfish_machine_stays_in_range`,
`w4_f14_the_driver_order_matches_the_csharp` (source-scan order pin —
keep; retarget to `pub fn animate`).

### App-side wiring witnesses

* f14 `render/tiles.rs`: `w4_f14_the_cage_post_pass_rides_the_first_switch_tail`
  (**the §2.3 order witness — adapt the call name to `cage_draw_data`
  and the gate to `is_cage_kind`; it REPLACES g34's inverted
  `cage > glow` assertion**), `w4_f14_the_walk_start_clears_the_cage_flag`.
* f14 `app/frame.rs`: `w4_f14_the_cage_tick_rides_the_animate_tiles_tail`,
  `w4_f14_the_cage_key_is_view_gated_and_the_draw_takes_the_animator`
  (retarget `frame.rs` → `frame/world.rs` + `frame/draw.rs`, the g33/g34
  file layout).
* f14 `app/mod.rs`: `cage_animator_state_changes_the_frame_key` (adapt
  from the dedicated `cage_anim` slot to the `tile_anim` FNV-mix).
* g33 `render/tiles.rs`: `cage_rewrite_rides_the_first_switch_before_the_glow_switch`
  (overlaps f14's order witness — keep f14's, it also pins slice B),
  `wired_cage_rewrite_matches_the_static_arms_at_t0` (**the all-92 t0
  parity witness — the union wiring's acceptance pin**).
* g34 (already on this branch, keep): `the_lane2_cage_rewrite_feeds_from_the_cage_animator`
  (after the §2.3 move + §2.2 widening: flip the order assertion, widen
  the gate assertion), `lane2_cage_split_is_disjoint_and_exhaustive`,
  the 12 animator witnesses, plus a80da26a's deviation-fix witnesses
  (`solid_tile_is_the_full_worldgen_predicate` et al. — unaffected by
  this reconcile).

### Merger acceptance checklist (verify wave, not this doc)

1. `cargo test --workspace -- --skip migrates_version_2_kinds_to_item_ids`
   green with the union witness set; anchors canary passes with the
   `critter_cage_anim.rs` rows (the `drawdata_cages.rs` rows from the
   other branches must NOT be merged in — they would fail the canary
   against the dropped module).
2. t0 static parity over all 92 ids (g33 witness) + the slice-order pin
   (f14 witness, adapted) both green.
3. Screenshot recipe (f14 ANALYSIS): terrarium row (275/620/316) in
   view, `TR_SHOT` f0 vs f600 — addFrY rows advance; `TR_NOCACHE=1` vs
   cached pixel-equal; cage-free view stays cache-hit; two runs
   SHA256-equal at zoom 1.0/1.5/2.0.
4. Determinism: cage-free play must keep the pre-wiring `Main.rand`
   interleave bit-for-bit (the gate + the clone warm-up are the two
   pieces that guarantee it).

## 6. Execution record (union EXECUTED on `task/W4-g34-drawdata-wire-2`)

All six phases landed on this branch, code-only (witnesses WRITTEN NOT
RUN), one commit per phase:

| Phase | Commit | What landed |
| --- | --- | --- |
| 1 — composite order (§2.3) | `f4da0697` | the `cage_draw_data` call moved from post-`apply_cases_a` to the FIRST-switch tail: after `draw_data` + slice B's `case_draw_data`, before slice A — the 568-570/580 glow rects now read the POST-cage `addFrY` (`:6085-6104`) and the `:1007` source-rect resolve reads the final `addFr`. New order witness `the_cage_rewrite_rides_the_first_switch_tail` (f14's pin adapted to the direct call); the old `cage > glow` assertion INVERTED (glow reads post-cage). |
| 2 — lifetime (§2.5/§3) | `d9b6219b` | `enter_loaded_world`'s `cage_anim.reset()` + `critter_cage = false` dropped — PROCESS state; the mis-cited "`:66902-66972` re-allocates in Initialize_TileAndNPCData" comment replaced with the static-ctor fact (`:66502`). `tile_anim.reset()` stays. |
| 3 — 92-id union gates (§2.2/§2.4) | `a5ba9493` | the tiles.rs draw gate + the G6 `tile_anim` cage contribution widened from `is_lane2_cage_kind` (50) to `is_cage_kind` (92; g33's 42 verified against 97704d01's id list + `cage_table` map — byte-identical arms). `is_lane2_cage_kind` kept as the census-split documentation predicate + its split witness. Feed witness renamed `the_cage_rewrite_feeds_from_the_cage_animator`; f14's tick-order + key-gate app-side witnesses retargeted to `frame/world.rs`/`frame/draw.rs`. |
| 4 — warm-up (§2.6/§4) | `da5c8176` | `warm_up` wired at BOTH constructors (`App::new` world-seed, `flow_app` seed-1 twin) on a FRESH `UnifiedRandom` clone — NOT g33's live `rngs.global()` form; the live `Main.rand` interleave stays bit-for-bit. The §4 exact-stream residual (ctor draws `:6467`/`:6594-6601` before `:6610`) stays recorded. f14 key-coverage witness adapted to the `tile_anim` FNV-mix + a warm-up/lifetime seam witness. |
| 5 — witness union (§5) | `23054bfb` (+ seam-witness self-match fix `3412f999`) | 42 supplement witnesses ported into `critter_cage_anim.rs` (54 total), retargeted to the per-table loop bodies + `animate(bool, rand)`; keep-one folds applied (g33 slug/fairy-jar/full-animate, f14 gate/frame_of/fairy-jar dropped as equivalents exist). Two g33 written-not-run assertions RE-DERIVED against the C#: the butterfly forward hop reads the POST-increment frame (6→7 never hops; pinned 3→4→12 + the 7→0 wrap instead of g33's unreachable 6→15), the pufferfish inflate cycle pinned to num25 = 183 (g33's 175 comment shipped the same code). The all-92 t0 acceptance pin `wired_cage_rewrite_matches_the_static_arms_at_t0` landed in tiles.rs (phase 1 commit). App-side union: 7 NEW wiring witnesses — tiles.rs order + t0 pins (phase 1), frame.rs tick-order + key-gate (phase 3), mod.rs key-coverage + warm-up/lifetime (phase 4; the negative assertions use runtime-built needles / statement forms so they cannot match their own source — `3412f999`), plus the renamed/updated tiles.rs feed witness (phase 3). |
| 6 — docs | `43ff6e93` | this status + record; research/66 residual + wiring notes repointed; ANALYSIS union section; PORT_MAP cage rows; anchors rows updated per phase. |

Merger instruction (supersedes the §2/§3 future tense): take THIS branch's
`render/tiles.rs`, `app/mod.rs`, `app/flow.rs`, `app/frame.rs`,
`app/frame/{world,draw}.rs`, `drawdata_furniture.rs`,
`critter_cage_anim.rs` and `anchors.txt` rows as-is; drop BOTH
`drawdata_cages.rs` variants + their `lib.rs` registrations, f14's
`furniture_post_pass` call site + `WorldFrameKey::cage_anim` slot, g33's
live-stream warm-up call. The §5 acceptance checklist (verify wave) is
unchanged and still owed: cargo green, t0 parity + order pin, screenshots,
cage-free determinism.

## Sources

* `git show origin/task/W4-f14-cage-furniture-wire:crates/tr-core/src/drawdata_cages.rs` (50046fe1 tree), `docs/ANALYSIS.md` "W4-f14 cage furniture wiring", `docs/research/09-tile-drawing.md` update.
* `git show origin/task/W4-g33-drawdata-wire-1:crates/tr-core/src/drawdata_cages.rs` (97704d01 tree), `docs/research/65-drawdata-wiring-lane1.md`.
* This branch @ 16f88fb7: `crates/tr-core/src/critter_cage_anim.rs`, `docs/research/66-drawdata-unwired-second-half.md`, `crates/tr-app/src/render/tiles.rs`, `app/mod.rs`, `app/frame/{world,draw}.rs`.
* C#: `decompiled/Terraria/Main.cs` `:6348/:6461-6467/:6594-6611/:6732-6798/:6800-6807/:8387-8395/:66502/:66901-66972/:10864-10876/:13511-16727/:19303`; `decompiled/Terraria/GameContent/Drawing/TileDrawing.cs` `:474/:5629-5907/:6085-6104/:6528-6547`; `decompiled/Terraria/WorldGen.cs` `:7947/:8192`.
