# 69 — Liquid 4-way reconcile (W4-g46 merger assist)

Date: 2026-11-21. Author: W4-g46 (reconcile phase, code-only — no cargo/tests run).

> **STALE-TIP WARNING (W4-g45 LIQUID RETIRE, 2026-11-21).** Everything below
> through A7 is written against the PRE-RE-FOLD tips (union `f57d7bacd`,
> graft-prep). The g46d RE-FOLD LANDED: `origin/task/W4-g46-liquid-evaporation`
> is now **`9bc722e5`** (base-swapped onto g47 `95e9c1ef`). Read **A8** (the
> NOW-LANDED receipt) + **A8-graft** (the exact down-arm hunk for g46e-seal)
> at the tail FIRST; the A1-A7 flags are historical except where A8
> re-marks them. The per-fn owner table of record is research/104 §5-final;
> the family receipt is research/104 §7 (SEALED).

> **UNION-CONTRACT (graft-prep phase, 2026-11-21).** The UNION `liquid.rs` is
> now LANDED on `task/W4-g46-liquid-evaporation` (this branch). **The merger
> keeps THIS branch's `crates/tr-core/src/liquid.rs` verbatim and DROPS the
> g45 (`2f15b7ba`) and g47 (`cfe5f1c9`) versions of that file** — the union
> already contains every KEEP item from both, grafted per the table below.
> The supporting files on this branch are the union too:
> `crates/tr-core/src/tileobject.rs` and
> `crates/tr-app/src/world_sim/gfx_quality.rs` are the g47 versions verbatim
> (the `GetTileData(Tile)`/`CheckWaterDeath`/`CheckLavaDeath` seams + the
> quickSettle quality override); `crates/tr-core/src/edit.rs`,
> `crates/tr-core/src/edit/kill.rs` and
> `crates/tr-app/src/world_sim/editing.rs` are the g46 versions (the
> `check_alch_tile`/`check_lily_pad`/`square_tile_frame_record` seams + the
> `resolve_liquid_retire_kills` app resolver); `crates/tr-app/src/world_sim/liquids.rs`
> drains the union `retire_kills` queue inside the tick and resolves after it.
> The f25 gen pass (`worldgen/settle_liquids.rs`) stays on its OWN branch
> (`task/W4-f25-*`) — it is NOT part of this union and the merger takes it
> separately; its gen-hash pins need re-derivation after the DelWater/buffer
> graft (row 11 warning below). The single kill queue is named `retire_kills`
> (g46's name); the g47 `WaterCheck` liquid-death records ride it too, and the
> g47 `drain_edit_kills` accessor is renamed `drain_retire_kills`. All
> witnesses are written-not-run (code-only mode); the merger runs the full
> union post-graft.

Four lanes touched `crates/tr-core/src/liquid.rs` (and neighbours) from the
same base `a725acb0` (wave-3 landing). This note is the per-function 4-way
comparison against the decompiled C# (`decompiled/Terraria/Liquid.cs`,
`WorldGen.cs`, `IO/WorldFile.cs`, `LiquidBuffer.cs`), the KEEP recommendation
per function, the witness union, and the runtime-inert gen-fields contract.
It changes NO code — the merger applies the recommendations.

## The four tips

| Lane | Tip | Scope |
| --- | --- | --- |
| g45 | `2f15b7ba` (branch W4-g45) | DelWater + keep-alive + stuck band + buffer counts, delay machinery, underworld evap (guarded), Next(30) nudge on `World.frame_rng`, 7/5-neighbour extra wakes |
| g46 | `524faa8b` (this branch) | LATEST-faithful DelWater drain/tail (drained-tail fall-through, lily branch, backwards splice), underworld evap (faithful, fixture sentinel), Next(30) nudge on per-consumer `surface_rng`, `retire_kills` loot seam + app resolver |
| g47 | `cfe5f1c9` (phases `047f37f3`/`e43fa0bc`/`dab3df3d`) | quickSettle/quickFall surface, DelWater + keep-alive + stuck + panic, QuickWater/SettleWaterAt/AttemptToMove* family, ReInit, WaterCheck, `settle_loaded_world` (UNWIRED), LiquidBuffer AddBuffer/DelBuffer counts |
| f25 | `0ed1f3cf` (branch W4-f25) | worldgen `Settle Liquids` pass (`worldgen/settle_liquids.rs`, WorldGen.cs:16132-16210): ocean retype, gen-local QuickWater(3)/SettleWaterAt with the waterLine/lavaLine/dungeon retype, WaterCheck, 10-round quickSettle budget loop, LiquidInteractionsCleanup + ShimmerRemoveWater; liquid.rs quickSettle arms + `gen_ignore_water` |

All four are code-only landings: every witness below is **written, NOT run**.
Line numbers cited as `gNN:LLL` are `liquid.rs` lines at that lane's tip
(f25's gen pass is `settle_liquids.rs`).

## C# ground truth anchors (re-read for this reconcile)

- `DelWater` — Liquid.cs:1470-1599. Order: <2 drain (:1479-1498) → <20
  drain (:1499-1505) → **>=20 keep-alive `kill=0; return`** (:1506-1510,
  gated on `!Liquid.stuck`) → above-wake (:1511-1514) → drained type reset
  (:1515-1518, **falls through**, no return) → unequal-side wakes +
  lava/honey/shimmer reaction + 3×3 grass ring (:1519-1569) → `numLiquid--`,
  `checkingLiquid(false)`, copy-last-into-slot **x/y/kill only, NOT delay**
  (:1575-1579) → alch/lily tail (:1580-1598). Lily: `quickFall` →
  CheckLilyPad; else `below < 255 || above > 0` → SquareTileFrame; else
  CheckLilyPad (:1586-1597).
- Retirement sweep — :1109-1119, **backwards** (`num7 = numLiquid-1; >=0;
  --`), 254→255 promotion before each DelWater (:1113-1116).
- Buffer promotion — :1120-1130: `num8 = numLiquid` clamped to buffer count,
  fixed loop; per promotion `checkingLiquid(false)` FIRST, AddWater,
  `DelBuffer(0)` **swap-remove** (LiquidBuffer.cs:22-27). AddBuffer caps at
  `maxLiquidBuffer - 2` and marks the buffered cell checking
  (LiquidBuffer.cs:11-20); AddWater's overflow threshold is `curMaxLiquid-1`
  (:1167-1171).
- Stuck band — :1131-1149: ±50 of `stuckAmount` for 10 000 cycle-ends →
  `stuck=true`, unconditional backwards DelWater sweep, `stuck=false`.
- Evaporation — :466-474: `y > Main.UnderworldLayer` (Main.cs:2790,
  `maxTilesY-200`), water type only, shed `min(2, liquid)`; the zero-kill
  (:475-479) reads the post-evap amount; the kill bookkeeping uses the
  pre-evap capture (:464).
- Next(30) nudge — :892-895: 3-neighbour average `== 254f` →
  `WorldGen.genRand.Next(30) == 0` → 255. The ONLY genRand draw in Liquid.cs.
- ReInit — :83-102 (counters/cadence/window reset; does NOT clear the buffer
  or the checking bits). QuickWater — :104-148. SettleWaterAt — :150-289
  (gen retype :178-192). AttemptToMove* — :291-433. Panic — :966-1054.
  Load settle — WorldFile.cs:733-764. Gen settle pass — WorldGen.cs:16132-16210.

## Per-function 4-way table

Legend: ✅ faithful · ⚠️ faithful-with-documented-deviation · ❌ wrong/absent ·
— not in lane.

### 1. `DelWater` (Liquid.cs:1470-1599)

| Aspect | g45 (`del_water(l)->bool`, :409) | g46 (`del_water(x,y)`, :426) | g47 (`del_water(l)`, :558) | f25 |
| --- | --- | --- | --- | --- |
| <2 / <20 drain arms | ✅ | ✅ | ✅ | — (no DelWater; `end_pass` :342 just DROPS kill≥8 entries) |
| >=20 keep-alive (:1506) | ✅ (`stuck`-gated) | ⚠️ parked (R-11-7 comment :464 — no stuck machine) | ✅ (`stuck`-gated) | — |
| Drained-tail fall-through (:1515→:1580) | ✅ | ✅ (explicit, witnessed) | ✅ | — |
| Lily branch (:1586-1597) | ✅ `quickFall \|\| (below==255 && above==0)` | ✅ same, inverted-guard form | ✅ same | — |
| Splice (:1575-1579) | caller `retire_sweep` after tail; x/y/kill only ✅ | caller `end_pass` after tail; x/y/kill only ✅ (equivalence documented: tail never AddWaters) | **inside** del_water, BEFORE the alch/lily tail — exact C# order ✅ | — |
| Lava grass ring (:1532-1560) | ✅ converts, but `square_tile_frame_default` DISCARDS the kill/drop records | ✅ converts + `square_tile_frame_record` → `retire_kills` (loot rides the app seam) | ✅ converts, `square_tile_frame_default` discards drops | — |
| Alch/lily kill loot | `check_alch_tile`/`check_lily_pad` return discarded | ✅ `retire_kills` + app `resolve_liquid_retire_kills` (UpdateLiquid RNG swap, WorldGen.cs:71529) | ✅ `edit_kills` + `drain_edit_kills` (same seam idea, different name) | — |
| Edge safety | ✅ `tile()` probe before `index` | ✅ `tile_mut`/`tile` probes | ✅ width guard | — |

**KEEP: g46 body + seams, grafted onto g47's structure.** g46 has the
LATEST-faithful drain/tail fixes and the only loot-carrying grass-ring
records; g47 has the only C#-ordered splice (before the tail) and the
keep-alive. Merger recipe:
1. Take g47's `del_water(&mut self, world, l: usize)` skeleton (keep-alive
   return, splice inside before the tail).
2. Replace its reaction/tail body with g46's (record-producing
   `square_tile_frame_record`, `check_alch_tile`, `check_lily_pad`), keeping
   ONE kill queue — recommend g46's `retire_kills` name + app resolver, or
   rename to g47's `edit_kills` if the WaterCheck kills share it (g47's
   WaterCheck already pushes there; a single queue is simpler — the app
   drains once per pass either way).
3. g46's `end_pass` must then STOP splicing itself (g47's `end_pass` shape
   wins, see row 2). g46's parked keep-alive comment is retired by the graft.
4. `check_lily_pad` signature differs: g46 `(world,x,y)`, g47
   `(world,x,y,|k| self.kind_solid(k))`. Keep g46's (its 518
   TileFrameImportant dispatch + `lily_pad_move` out-of-world decline are
   the newer edit/kill.rs work); verify g47's closure carried no extra
   solidity state the pure form loses (it reads the same static tables).

### 2. `end_pass` retirement sweep + 254 promotion (:1106-1119)

| | g45 (:342, `retire_sweep`) | g46 (:358) | g47 (:493) | f25 (:342) |
| --- | --- | --- | --- | --- |
| Backwards walk | ✅ | ✅ | ✅ | ❌ forward rebuild-and-drop |
| 254→255 before drain | ✅ (skipped on force — matches :1137 having no promotion) | ✅ | ✅ | ❌ none |
| DelWater drain | ✅ | ✅ | ✅ | ❌ entries dropped, no drain/tail |
| Stuck band (:1131-1149) | ✅ | ❌ absent | ✅ | ❌ |

**KEEP: g47** (newest, has the stuck band; g45's is equivalent but
superseded). g46's backwards-sweep fix is already subsumed.

### 3. Buffer (`AddWater` overflow + cycle-end promotion, LiquidBuffer.cs)

| | g45 (:245/:537) | g46 (:266/:404) | g47 (:360/:511) | f25 (:247/:354) |
| --- | --- | --- | --- | --- |
| Overflow threshold `curMaxLiquid-1` | ✅ | ❌ `MAX_ACTIVE` | ✅ | ❌ `MAX_ACTIVE` |
| Buffer cap `maxLiquidBuffer-2` + checking mark | ✅ | ❌ `MAX_BUFFER`, no mark | ✅ | ❌ |
| Promotion count = `numLiquid` (fixed, clamped) | ✅ (re-checks cap per iter — minor) | ⚠️ `.max(1)`, counts only successful adds | ✅ `min(active,buffer)` fixed | ⚠️ same as g46 |
| `checkingLiquid(false)` before AddWater | ✅ | ❌ | ✅ | ❌ |
| `DelBuffer(0)` swap-remove | ✅ | ❌ FIFO `remove(0)` | ✅ | ❌ FIFO |

**KEEP: g47** (g45 equivalent, superseded). g46/f25's FIFO promotion is a
real ordering deviation — do NOT keep it. NOTE: f25's gen-pass hash pins
were derived with the FIFO buffer; swapping in g47's buffer changes gen
settle order → see the hash warning under row 10.

### 4. `QuickWater` (:104-148)

| | g45 | g46 | g47 (:1327) | f25 (`settle_liquids.rs:131`) |
| --- | --- | --- | --- | --- |
| Runtime/load form (minY/maxY, bubble solid, ignoreSolids, bottom-up 4..w-4) | — | — | ✅ | ✅ (gen-local copy, verbose=3) |
| Gen arms: ShimmerRemoveWater before/after, noTraps boulders, LiquidInteractionsCleanup | — | — | ⚠️ deferred to gen lane (documented) | ✅ ShimmerRemoveWater + Cleanup; noTraps deferred to d107/c30 |

**KEEP: g47 for runtime/load; f25's gen-local copy for the gen pass.** They
are complementary, not competing: g47's is the `Liquid.QuickWater` method
on `LiquidSystem`; f25's is the `WorldGen.cs:16157` call-site wrapper with
GenVars. A later unification (one method, `generating`-gated gen arms) is
possible but NOT required for the merge — keep both, cite each other.

### 5. `SettleWaterAt` (:150-289)

| | g45 | g46 | g47 (:1349) | f25 (`settle_liquids.rs:185`) |
| --- | --- | --- | --- | --- |
| Drop / fan / landing mechanics | — | — | ✅ | ✅ |
| Gen retype (:178-192 waterLine/lavaLine/dungeon-wall) | — | — | ⚠️ omitted (runtime keeps origin type — documented) | ✅ full (remix band + dungeon-wall keep) |
| Landing AttemptToMove* tail | — | — | ✅ | ✅ |
| Bounds-safe landing guard | — | — | ✅ | ✅ (degenerate-canvas guard) |

**KEEP: g47 runtime + f25 gen** (same split as row 4). The retype arms are
gen-only by C# construction (`flag4 && !honey && !shimmer` inside the
`isGeneratingOrLoadingWorld` world); the runtime omission is faithful, not
a gap.

### 6. `AttemptToMoveLava/Honey/Shimmer` (:291-433)

g45 —, g46 —. g47: three explicit methods (:1474/:1489/:1504). f25: one
generic `attempt_to_move(kind)` (`settle_liquids.rs:336`) dispatching through
`LiquidSystem::liquid_reaction_check`. Both faithful (first different-type
neighbour in L/R/U/D order, check at origin-or-neighbour).

**KEEP: g47's three methods** for the runtime family; f25's generic helper
stays inside the gen pass (it calls the public `liquid_reaction_check`
which the merged liquid.rs must keep exporting).

### 7. Panic mode (`StartPanic` + 5-row drain, :966-1054)

g45 — (comment only), g46 —, f25 —. **g47 only** (:208 `start_panic`, :403-429
the gate + drain in `update`): ≥90% buffer for 3600 updates → drop lists,
`panic_y = height-3`, five QuickWater rows per update, WaterCheck on finish.
The `GenVars.waterLine = maxTilesY` pin is documented runtime-inert.

**KEEP: g47.**

### 8. `ReInit` (:83-102) + `settle_loaded_world` (WorldFile.cs:733-764)

g45 —, g46 —, f25 —. **g47 only** (:184 `re_init`, :229
`settle_loaded_world`). Deviations documented in g47: ReInit also clears
buffer/checking/skip (C# leaves them; the load tail's WaterCheck wipes them,
so the combined sequence matches); `skipCount` has no port counter; the
`Setting_UseReducedMaxLiquids` 5000 cap is unported (SP stock).
`settle_loaded_world` is ported + witnessed but **UNWIRED** — the save/load
round-trip pins need a deliberate re-derivation first.

**KEEP: g47, load hook stays UNWIRED** (do not wire it in this merge).

### 9. Underworld evaporation (:466-474)

| | g45 (:576-602) | g46 (:575-593) | g47 | f25 |
| --- | --- | --- | --- | --- |
| Shed `min(2,liquid)`, water-only, pre-flow | ✅ | ✅ | ❌ NOT ported (documented deferral) | ❌ |
| Depth gate | ⚠️ `underworld_layer >= 0` guard — deviates from C# on sub-200 worlds | ✅ `y > height-200` unguarded = C# semantics | — | — |
| Small-fixture isolation | guard (behaviour change) | ✅ `new_flow_fixture` pins `underworld_layer = i32::MAX` (test-only, tool.rs convention) | — | — |
| Post-evap zero kill / pre-evap capture | ✅ | ✅ (explicit :464 capture note) | — | — |

**KEEP: g46.** It is the faithful one (g45's `>=0` guard is a behaviour
deviation the C# does not have); the fixture sentinel keeps the ~20 flow
pins and `mass_conserved` out of the band without touching production
semantics. g47's deferral note is resolved by this landing.

### 10. Next(30) surface nudge (:892-895)

| | g45 (:783-793) | g46 (:758-770) | g47 | f25 |
| --- | --- | --- | --- | --- |
| Arm (`avg==254 && Next(30)==0 → 255`, draw only on 254) | ✅ | ✅ | ❌ deferred (documented) | ❌ |
| Stream | `World.frame_rng` (shared with SquareTileFrame re-rolls) | per-consumer `LiquidSystem::surface_rng`, world-seeded (RNG census + stream-pin witness) | — | — |

**KEEP: g46's arm + `surface_rng` stream.** Vanilla shares one `genRand`
across ALL consumers; neither stand-in is fully faithful, both are
documented deviations — but g46's is the one with the census
(`docs/research/11-liquid.md` deviation 5) and the determinism pin
(`surface_rng_is_the_world_seeded_unified_stream`). The merger must pick
ONE stream; picking g45's `frame_rng` instead would entangle the liquid
settle with every SquareTileFrame consumer and invalidate g46's pin. Carry
g45's three nudge witnesses (below) re-pinned to `surface_rng`.

### 11. Gen settle pass driver (WorldGen.cs:16132-16210) — f25 only

`worldgen/settle_liquids.rs`: ocean retype sweep, QuickWater(3), WaterCheck,
10-round `num3*5` budget loop, ClearPendingLiquid tail,
LiquidInteractionsCleanup (the :23755 up-shimmer→water-counter quirk kept),
ShimmerRemoveWater (d102 `shimmer_position` seam), dungeon-bounds retype
guard. No RNG draws — ledger-neutral insertion. Pipeline step before
`remove water` (66→67).

**KEEP: f25 wholesale.** No other lane has it.

> **⚠️ Gen-hash warning for the merger.** f25's `world_hash.rs` pins and
> `the_pass_is_deterministic` were derived against f25's OWN liquid.rs
> base: discard-only `end_pass` (no DelWater drain/tail), FIFO buffer,
> `MAX_ACTIVE` overflow. Grafting g46/g47's real DelWater + g47's
> swap-remove buffer into the shared `LiquidSystem` changes the gen
> settle's retirement order and tail side-effects (grass burns, alch/lily
> kills now fire during gen). The f25 hash pins MUST be re-derived after
> the graft — expect a delta, do not silence it. The `retire_kills`/
> `edit_kills` queue also needs a gen-side drain (or an explicit
> gen-discard note) so records don't leak across the per-pass instance.

### 12. Supporting surfaces (quickSettle arms, WaterCheck, extra wakes)

- **quickSettle/quickFall** (:588-591 clamp, :944-948 settle-step,
  :1056-1068 derivation): f25 and g47 both ported; g46/g45 have only the
  `quick_fall` field + delay-10 loop. **KEEP g47's** (phase 1 `047f37f3`,
  witnessed); f25's identical arms dedupe onto it.
- **WaterCheck** (WorldGen.cs:80415-80494): f25 (liquid.rs, raw-solidity
  form, `kill_tile` drops discarded) and g47 (:1523, `kind_solid` form,
  kills ride `edit_kills`). **KEEP g47's** (loot-carrying); f25's gen pass
  calls `sys.water_check` so it inherits the upgrade — the gen-hash warning
  applies (liquid-death kills now produce records during gen).
- **7/5-neighbour extra wakes** (:748-771/:821-836): **g45 ONLY**
  (:740/:760). Not superseded by anything — the one g45 item that must be
  carried forward (or explicitly deferred with an ANALYSIS row). Recommend
  carrying it: it is small, witnessed, and RNG-free.
- **Delay machinery** (lava 5 / honey 10 / quickFall 10): g45/g46/f25 all
  have `flow_delay`; g47 too. Equivalent — keep the merged base's.
- **`gen_ignore_water`** (doors 10/192, living-wood/leaf 191/190,
  Liquid.cs:75-81): f25 only. KEEP (gen-fields contract below).

## Witness union (all written-not-run; merger runs them post-graft)

Common base (all four, identical): grate/bubble/boulder solidity ×3, merge
family ×11 (obsidian/honey/crisp-honey/shimmer/24-gate/down-merge/plant-burn/
type-priority/change-type/container/live-sound), flow ×4 (pit/pool/dam/walls),
`add_water_deduplicates_tiles`, stride ×2, actuated floor, `liquid_sim` ×4
(`water_falls_per_csharp`, `water_spreads_to_csharp_thickness`,
`water_plus_lava_is_obsidian`, `mass_conserved`). Keep one copy.

| Witness | Lane | Disposition |
| --- | --- | --- |
| `underworld_water_evaporates_two_per_update`, `a_sub_two_film_evaporates_whole`, `water_above_the_underworld_layer_does_not_evaporate`, `lava_honey_and_shimmer_never_evaporate` | g46 :1837-1905 | KEEP (g46 evap) |
| `the_retire_sweep_splices_backwards_and_the_lily_tail_runs_on_drained_cells` | g46 :1908 | KEEP — re-run against the g47-structured splice (tail-before/after equivalence) |
| `surface_rng_is_the_world_seeded_unified_stream` | g46 :1937 | KEEP (stream pin) |
| `water_evaporates_two_per_visit_below_the_underworld_layer`, `water_at_the_underworld_layer_row_does_not_evaporate`, `lava_honey_shimmer_never_evaporate` | g45 :2183-2231 | DROP the first (g45 guard semantics) or re-derive on g46; the layer-row boundary witness is unique — RE-PIN onto g46 (`y > layer` strict) and keep |
| `lava_falls_once_every_six_visits`, `honey_falls_once_every_eleven_visits`, `quick_fall_bypasses_the_delay_gate`, `the_delay_gate_never_throttles_the_merge` | g45 :2112-2181 | KEEP (delay pins; g46 has the machinery, no pins) |
| `the_down_arm_drains_a_sub_24_cell_into_the_liquid_below`, `the_water_cell_down_arm_burns_no_plant`, `shimmer_merges_with_lava_and_honey_into_659` | g45 :2233-2324 | KEEP (g46 has the sub-24 arm at :931; pins are lane-independent) |
| `the_swap_remove_inherits_the_retired_slots_delay`, `the_sweep_promotes_a_254_cell_to_255`, `the_keep_alive_branch_rescues_a_draining_cell` | g45 :2326-2380 | KEEP — overlap g47's `delwater_swap_remove_rides_kill_but_not_delay` / `the_retirement_sweep_promotes_254_before_delwater` / `delwater_keeps_a_full_cell_over_an_open_below_alive`; keep ONE of each pair (g47's names, g45's delay-inheritance assertion is the sharper one — merge the assertions) |
| `lava_retirement_burns_the_grass_ring` (mud-60 ring fix), `the_alch_tail_kills_an_unsupported_herb`, `the_lily_pad_tail_sinks_a_dry_pad_into_the_cell_below` (grass-family ground fix) | g45 :2382-2435 | KEEP — re-point at g46's record seams (`retire_kills` instead of discarded returns); g47's `delwater_retires_a_submerged_herb_through_check_alch` / `delwater_sinks_a_dry_lily_pad_onto_the_full_water_below` overlap — keep one of each, g45's fixtures carry the 2f15b7ba witness corrections |
| `the_buffer_promotion_clears_checking_and_resets_delay` (survivor fix), `the_stuck_window_force_clears_after_10000_cycles` (kill-0 survivor fix) | g45 :2439-2483 | KEEP vs g47's `the_buffer_overflow_marks_checking_and_the_promotion_swap_removes` / `the_stuck_band_self_clears_after_10000_cycles` + `the_stuck_band_repins_when_the_count_leaves_it` — union: g47's repin witness is unique, g45's delay-0 assertion is unique |
| `the_254_nudge_promotes_on_a_zero_genrand_draw`, `the_254_nudge_keeps_254_on_a_nonzero_draw`, `the_nudge_draw_only_advances_the_stream_on_a_254` | g45 :2517-2564 | RE-PIN onto g46 `surface_rng` (seed-35/seed-0 draw values change) and KEEP |
| `the_extra_wake_requeues_an_unqueued_neighbour_on_a_centre_change` | g45 :2566 | KEEP with the extra-wake carry (row 12) |
| quickSettle ×3 (`quick_settle_clamps_the_fall_remainder_to_255`, `quick_settle_promotes_the_254_settle_step`, `the_pass_head_derives_quick_fall_from_quick_settle`) | g47 :2472-2587 | KEEP |
| `delwater_drains_a_thin_film_into_the_open_side` | g47 :2615 | KEEP |
| QuickWater family ×5 (`settle_water_at_drops_the_blob_to_the_floor`, `..._tops_up_the_partial_cell_below`, `..._keeps_the_bubble_and_skips_empty`, `quick_water_settles_the_whole_column`, `the_landing_merge_check_fires_on_the_different_type_side`) | g47 :2865-2941 | KEEP |
| WaterCheck ×2, `re_init_resets_the_cadence_and_lists`, panic ×2, `the_load_settle_drains_a_suspended_blob_to_the_floor` | g47 :2943-3046 | KEEP (load-settle witness stays unwired-path) |
| `the_pass_settles_a_suspended_pool_and_clears_the_queue`, `the_ocean_retype_sweep_turns_lava_to_water`, `the_pass_is_deterministic`, `the_cleanup_refills_a_lone_obsidian_block` | f25 `settle_liquids.rs:537-` | KEEP — `the_pass_is_deterministic` + the world_hash pins need RE-DERIVATION after the DelWater/buffer graft (row 11 warning) |

Fixture conventions to preserve: g46 `new_flow_fixture`
(`underworld_layer = i32::MAX`) for every flow witness; g45's `floor_world`
/ `put_as` helpers (height ≥ 200 worlds) for the DelWater witnesses; g47's
9-wide fixtures are incompatible with the deferred `AddWater` 5-tile border
guard (g47 ANALYSIS) — that guard stays deferred in this merge.

## Runtime-inert gen-fields contract

The merged `LiquidSystem` carries generation state that MUST be inert on
the runtime path. This is the contract the merger preserves:

1. **Fields, default-false in `new`**: `generating`, `remix_world_gen`,
   `quick_settle`, `gen_ignore_water` (f25), plus `quick_fall` (derived,
   never set directly at runtime). Only the gen pass
   (`settle_liquids.rs`) and the load driver (`settle_loaded_world`, g47)
   flip them, via `set_generating` / `set_quick_settle` /
   `world_gen_tiles_ignore_water` / `tiles_ignore_water` / `set_bubble_solid`.
2. **No GenVars reads in liquid.rs.** The waterLine/lavaLine/dungeon retype
   (:178-192), ShimmerRemoveWater, noTraps boulders and
   LiquidInteractionsCleanup live ONLY in the f25 gen module, which takes
   `&GenVars` explicitly. g47's runtime `settle_water_at` keeps the origin
   type — that omission is the contract, not a gap.
3. **Per-pass instance boundary**: the gen pass constructs its own
   `LiquidSystem::new(world)` (f25 module doc; `ClearPendingLiquid` +
   `numLiquid==0` exit, WorldGenerator.cs:508). Gen state never leaks into
   the runtime system. The `retire_kills`/`edit_kills` queue must be drained
   or discarded at the pass tail (row 11 warning).
4. **Runtime consumers of the gated arms**: `quick_settle`'s :588-591 clamp
   and :944-948 promotion are `false`-gated at runtime; `gen_ignore_water`'s
   door/leaf solidity carve-out is `false`-gated; the panic `waterLine` pin
   has no runtime consumer (g47 doc). `ReInit` clears `quick_settle` /
   `quick_fall` (Liquid.cs:92-93).
5. **`underworld_layer` is NOT a gen field** — it is runtime-faithful
   (g46), seeded `height-200` in `new`; only the `#[cfg(test)]`
   `new_flow_fixture` sentinel moves it. Do not gate it on `generating`.
6. **`surface_rng`** (g46) is per-consumer and world-seeded; it advances
   ONLY on a 254 average (:892 guard) — runtime and gen alike. It is not
   part of the gen ledger (f25's pass is ledger-neutral; the nudge draw is
   a documented per-consumer deviation, ANALYSIS W4-g46).

## KEEP summary (one line per function)

| Function | KEEP |
| --- | --- |
| DelWater body/tail + loot records | **g46**, grafted into g47's `del_water(l)` skeleton (keep-alive + splice-before-tail) |
| end_pass sweep / 254 promotion / stuck band | **g47** |
| Buffer AddBuffer/DelBuffer/promotion | **g47** |
| QuickWater / SettleWaterAt / AttemptToMove* (runtime) | **g47** |
| QuickWater / SettleWaterAt / AttemptToMove (gen, with retype) | **f25** (gen-local, keep) |
| Panic / ReInit / WaterCheck / settle_loaded_world | **g47** (load hook UNWIRED) |
| Underworld evaporation | **g46** |
| Next(30) nudge | **g46** arm + `surface_rng` (g45 witnesses re-pinned) |
| Gen settle pass driver + Cleanup + ShimmerRemoveWater | **f25** |
| quickSettle arms / gen_ignore_water / num_liquid_total / liquid_reaction_check | **g47 + f25** (dedupe quickSettle onto g47; keep f25's exports) |
| 7/5-neighbour extra wakes + delay pins | **g45 carry-forward** (the only non-superseded g45 items) |
| Everything else g45 | **superseded** by g46 (DelWater/evap) + g47 (sweep/buffer/stuck) |

Merger order suggestion: g47 liquid.rs as the structural base → graft g46
evap/nudge/record-seams/underworld_layer → dedupe f25's quickSettle arms and
keep its gen module + exports → carry g45 extra-wakes + witness union →
re-derive f25 gen-hash pins and the re-pinned nudge witnesses → run the full
union (no lane ran any of them).

## Graft-prep landing record (what THIS branch now carries)

The union `liquid.rs` on this branch is the g47 skeleton (`cfe5f1c9`) with the
g46 grafts applied and the g45 carry-forward appended. Concretely:

- **Structural base = g47**: `del_water(&mut self, world, l: usize)` with the
  keep-alive return (`:1506-1510`, `stuck`-gated) and the swap-remove INSIDE
  `del_water` before the alch/lily tail (`:1575-1579`); `end_pass` is g47's
  (backwards sweep + 254→255 promotion + `min(active,buffer)` swap-remove
  promotion + the stuck band); `add_water` is g47's (`curMaxLiquid-1` overflow,
  `maxLiquidBuffer-2` cap + checking mark); the QuickWater/SettleWaterAt/
  AttemptToMove* family, `start_panic`/panic drain, `re_init`,
  `settle_loaded_world` (UNWIRED), `water_check`, the quickSettle arms and the
  `quick_fall = quick_settle` pass-head derivation are all g47's.
- **g46 grafts**: the `underworld_layer` field (seeded `height-200` in `new`,
  `i32::MAX` in the `#[cfg(test)] new_flow_fixture`), the underworld
  evaporation block in `update_tile` (`y > underworld_layer`, water-only,
  `min(2,liquid)`, pre-flow, zero-kill `KILL_DEAD`), the live post-evap
  `own_now` read in the fall arm, the `surface_rng` field + the Next(30) nudge
  in the 3-cell arm, the record-producing lava grass ring
  (`square_tile_frame_record` → `retire_kills`), the `check_alch_tile` /
  `check_lily_pad` (pure g46 form, no closure) alch/lily tail with the
  `square_tile_frame_record` reframe arm, and the drained-type-reset
  `mark_chunk`. The single kill queue is `retire_kills` (g46 name); the g47
  `WaterCheck` liquid-death kills push to it; `drain_edit_kills` →
  `drain_retire_kills`.
- **g45 carry-forward**: the 7/5-neighbour extra-wake passes (`:748-771` /
  `:821-836`, centre-diff reduction, before the conditional centre write) in
  the row arms; the `liquid_merge_cadence`, `liquid_delwater` and
  `liquid_nudge_wakes` witness modules (re-pinned per the disposition table:
  the nudge witnesses on `surface_rng` with `w.seed = 35/0`, the extra-wake
  witness re-derived onto the 7-arm fixed-point fixture, the layer-row evap
  boundary on the g46 unguarded gate, the delay-inheritance / grass-ring /
  dry-pad / buffer-delay-0 / kill-0-stuck witnesses kept, the g45 duplicates
  dropped in favour of g47's named equivalents).
- **Flow witnesses** use `new_flow_fixture` (the evaporation sentinel) so the
  ~20 flow pins and `mass_conserved` stay out of the band; the evaporation
  witnesses and the layer-row boundary use `new`.
- **Supporting files**: `tileobject.rs` + `gfx_quality.rs` = g47 verbatim;
  `edit.rs` + `edit/kill.rs` + `world_sim/editing.rs` = g46 verbatim (the
  `resolve_liquid_retire_kills` resolver, doc updated to name the WaterCheck
  records); `world_sim/liquids.rs` = the g47 inside-tick drain shape on the
  union `retire_kills` queue.
- **f25 gen pass**: NOT on this branch — stays on `task/W4-f25-*`; the merger
  takes it separately and re-derives its gen-hash pins (row 11 warning).

Witness count: 72 `#[test]` fns in the union `liquid.rs` (g47's 49 + g46's 6
evap/stream/lily-tail pins + g45's 17 carried cadence/delwater/nudge pins,
minus the 5 g45 duplicates dropped per the disposition table). All
written-not-run; compile-by-inspection only (no cargo/tests in this phase).

---

## ADDENDUM — W4-g45 DOC-SWEEP lane 24 (postdates this reconcile + the graft-prep UNION)

Date: 2026-11-21. Author: W4-g45 (doc-sweep phase, code-only — no cargo/tests
run). This addendum postdates BOTH the original 4-way table (g45 tip
`2f15b7ba`) AND the graft-prep UNION landing record above (g46 tip
`f57d7bacd`). The g45 branch has since landed the **AddWater death-guards**
(`daa1cdb0` phase 1 + `8e81814e` phase 2 docs, the round-2 work) and a
doc-sweep pass (`c8908a55d`/`fa96cf32a`/`99a4f8ccf`). **The UNION dropped the
g45 AddWater guards and the widened fixtures** — the graft-union MUST carry
them. The g46c active reads (the graft-prep UNION `liquid.rs`) are cited below.

### A1. The UNION's `add_water` is g47's pre-guard form — the g45 guards are MISSING

The UNION `add_water` (`f57d7bacd:crates/tr-core/src/liquid.rs:381-409`,
`&World` signature) is g47's queue-only form: bounds + checking dedupe +
`liquid==0 || tile_is_solid` refusal + the `curMaxLiquid-1` overflow. It does
NOT carry the g45 round-2 guards (`daa1cdb0`, g45 `liquid.rs:236-300`):

1. **The 5-tile border exclusion** (`Liquid.cs:1163`): `x < 5 || y < 5 ||
   x >= width-5 || y >= height-5` → return. The UNION has only the `x<0 /
   x>=width` bounds, not the 5-tile margin.
2. **The grate-546 carve-out** (`Liquid.cs:1163`'s `type != 546`): the UNION's
   `tile_is_solid(t)` refuses the grate outright; the g45 form exempts
   `t.kind != 546` EVEN outside the pass (the stock `Main.tileSolid[546]=true`).
3. **The inline death check** (`Liquid.cs:1183-1206`): after the queue, an
   active non-generating cell reads `TileObjectData.CheckLavaDeath` (lava) /
   `CheckWaterDeath` (every other kind — honey AND shimmer ride the water arm,
   no shimmer AddWater exists) and kills the tile inline (`kill_tile`). The
   UNION's `add_water` returns right after the queue — NO death check. This is
   the **W4-e11 merge-order seam** (`tileobject::check_water_death` /
   `check_lava_death`, the Tile overloads resolving the def from the cell's
   frames, `TileObjectData.cs:5083-5119`); e11 is not in main yet.
4. **The `&mut World` ripple**: the g45 `add_water` takes `&mut World` (the
   inline `kill_tile` mutates); `wake_around`/`refill_from_buffer` + the 5 app
   callers carry it. The UNION's `add_water`/`wake_around` are `&World`
   (`f57d7bacd:381/:413`) — grafting the death check forces the `&mut World`
   signature through the whole wake chain.

**Graft instruction**: take the g45 `add_water` body (guards + death check +
`&mut World`) IN PLACE OF the UNION's g47 form, keeping the UNION's
`curMaxLiquid-1` overflow + `maxLiquidBuffer-2` cap (the two agree there). The
g45 `liquid_addwater_guards` witness module (10 pins: border both edges, grate
carve-out, inline torch death, torch-style/water-torch-subtile + plant-3
global + pile-372 def-beats-global discriminators, honey/shimmer arm,
open-cell/generation early returns, promotion death check, dry-cell promotion
swap, RNG-free census) MUST travel with it.

### A2. The UNION's fixtures are NOT widened — they break under the border guard

The g45 round-2 work embedded EVERY liquid fixture at a (+6,+6) offset in a
world 12 wider / 12 taller, with containment walls, so every liquid cell
clears the 5-tile border (the g47 co-lane's documented precondition). The
UNION kept g47's un-widened fixtures:

- `basin(width, wall_y)` = `World::new(width, wall_y+3)` (`f57d7bacd:1859-1875`)
  — the scene sits at the world edge, x from 0. Under the border guard every
  cell with x<5 / y<5 refuses to queue → the ~11 merge-family witnesses and
  the flow pins go inert.
- The g45 widened `basin` (`g45 liquid.rs:1418-1451`) = `World::new(width+12,
  wall_y+12)`, floor rows `wall_y+6..=wall_y+8`, containment walls at x=5 and
  x=width+6, callers shift (+6,+6). `floor_world`/`shaft`/`put_as` likewise
  (+6,+6), height ≥ 200 for the evap band.

**Graft instruction**: carry the g45 widened fixture helpers + the (+6,+6)
caller shifts. The UNION's `new_flow_fixture` evaporation sentinel
(`underworld_layer = i32::MAX`, `f57d7bacd:303-305`) is the g46 mechanism and
stays for the flow pins; the g45 `floor_world` height≥200 convention is the
DelWater/evap-witness mechanism — the two coexist (the sentinel moves the
layer, the widening moves the border). Do NOT drop either.

### A3. The UNION carries the SAME down-arm stale-read bug g45 just fixed

The UNION's `liquid_check` down arm (`f57d7bacd:1130-1145`) reads `down` ONCE
before the `tileCut` burn (`:1131`) and re-uses that copy for the merge gate
(`:1143`) — identical to the pre-fix g45 form. The C# `tile4` is a LIVE
reference: the gate (`Liquid.cs:1286`) and the merge-type reads (`:1303`) run
AFTER the burn (`:1280`), so a `tileCut` kind the kill just cleared passes on
`!tile4.active()` even when it is not `tileObsidianKill`. The seven cut kinds
OUTSIDE the set (444 BeeHive / 518 LilyPad / 519 Cattail / 549 Seaweed /
637 AshPlants / 638 AshVines / 711 — generated-bitset membership hand-computed)
are wrongly refused by the pre-kill copy.

**g45 fix `c8908a55d`**: re-read `down` after `kill_tile` (the `flag` container
read stays PRE-kill — `:1274` precedes `:1278`); `remove_tile` keeps the
liquid + type bits (`WorldGen.cs:63885`), so the amount/type reads see the same
liquid. 4 discriminating witnesses (`liquid_downarm_live`: sub-24 drain over
seaweed-549, full merge over lily-pad-518, honey arm over cattail-519, water
arm burns nothing). The existing plant-3 witness cannot discriminate (3 IS
obsidianKill). **Graft instruction**: carry the g45 re-read + the
`liquid_downarm_live` module into the UNION's `liquid_check`.

### A4. The g45 doc-sweep additions the UNION must also carry

- **The fall-arm post-evap live read** (`8d581953b`):
  the g45 fall arm read the PRE-evaporation `start.liquid` for the down
  transfer (`Liquid.cs:570-575` reads the LIVE `tile5.liquid`), resurrecting
  the shed units on every fall inside the underworld band. The UNION ALREADY
  carries the fix (`own_now`, `f57d7bacd:832`); g45 caught up to it +
  added the discriminating witness `the_fall_arm_moves_the_post_evaporation_level`
  (a 100-water cell in the band falls 98, not 100). **Graft instruction**:
  keep the UNION's `own_now` form; carry the g45 witness.
- **The bucket/sponge family** (`fa96cf32a`): `Player.ItemCheck_UseBuckets`
  (`Player.cs:46318-46504`) was entirely unported on EVERY lane (research/11
  open ROADMAP item; S1-E-21f deferred the five splashes). g45 ported it into
  `crates/tr-app/src/world_sim/editing/buckets.rs` (the twelve kinds, the
  pickup/placement arms, `PutItemInInventoryFromItemUsage`, the five
  `PlaySound(19)` splashes) + wired it before the melee arm + 10 witnesses.
  This is an `editing.rs`-side addition (the UNION's `editing.rs` is g46
  verbatim) — the merger takes the g45 `buckets.rs` module + the
  `use_buckets` call site + the `bucket_use_tests` module. It calls
  `liquids.add_water(&mut world, …)` / `wake_around`, so it depends on the A1
  `&mut World` signature.
- **The liquid-side shimmer flow witness** (`99a4f8ccf`):
  `shimmer_falls_every_visit_like_water` in `liquid_merge_cadence` — the
  shimmer arm (`Liquid.cs:542-545`) has NO delay gate (only lava/honey
  throttle), so shimmer falls like water at the one-tile-per-two-passes
  skipLiquid cadence. e19/f47 own the item-side shimmer physics; this pins the
  engine half. Carry it into the UNION's cadence module.

### A5. Census verdicts (the doc-sweep's remaining Liquid.cs questions)

- **QuickShimmer**: does NOT exist in the 1.4.5.8 decompilation (a grep for
  `QuickShimmer` over `decompiled/Terraria/**.cs` returns nothing). The
  shimmer bulk-settle is `AttemptToMoveShimmer` (`Liquid.cs:387-433`) +
  `ShimmerCheck` (`:1465-1468`), both already in the g47 family the UNION
  keeps. No gap.
- **Bucket water placement / wet-bucket fill**: the `ItemCheck_UseBuckets`
  arms — ported by g45 (A4). The EmptyBucket/LavaBucket/HoneyBucket use arms
  live in `Player.cs:46318-46504`, NOT in Liquid.cs; the i-items census
  (research/60) listed buckets at `:43572-43575` as an open use-arm.
- **Liquid pressure/flow-rate constants**: there is NO "pressure" constant in
  Liquid.cs (a grep for `pressure|Pressure` returns nothing). The flow rates
  are the per-entry `delay` gates (lava 5 → 1-in-6, honey 10 → 1-in-11,
  `:484-490`/`:513-522`), the `curMaxLiquid/cycles` stride (`:1070`), the
  `kill >= 8` retirement (`:1111`, MP `10+players/3`), and the statics
  `maxLiquid 25000 / maxLiquidBuffer 50000 / cycles 10` (`:1671-1673`). All
  verified matching the port (`MAX_ACTIVE`/`MAX_BUFFER`/`CYCLES`/`KILL_PASSES`,
  the gfx-quality runtime pair `17-10q` / `maxLiquid*0.25+0.75q`). The
  `wetCounter` window (`:1069-1108`) and the `num2=-1` sub-3 bias (`:644-648`)
  match. No gap.
- **Honey-block/crispy-honey reaction (c34 owns the merge)**: c34
  (`a7f63d95`/`37b0b702`) landed the cadence/merge witness suite (delay 5/10,
  quickFall bypass, sub-24 down-arm drain, shimmer 659 arms) + the underworld
  evaporation. The g45 delay counters (`Entry.delay`, the reaction-block
  gates) feed c34's merge exactly as the C# does — the reactions run EVERY
  visit, the delay throttles only the flow (`Liquid.cs:480-490`). Verified:
  the g45 `the_delay_gate_never_throttles_the_merge` pin and c34's cadence
  pins agree. The A3 down-arm fix is the one merge-side correction c34's
  suite did not catch (its plant-3 witness is obsidianKill).

### A6. Updated graft order (supersedes the "Merger order suggestion" above)

g47 liquid.rs structural base (the UNION already is this) → graft g46
evap/nudge/record-seams/underworld_layer (the UNION already has these) →
**REPLACE the UNION's g47 `add_water` with the g45 guarded form + carry the
widened fixtures (A1/A2)** → **carry the g45 down-arm re-read (A3)** → carry
g45 extra-wakes + the bucket module + the shimmer-flow witness (A4) → dedupe
f25's quickSettle arms + keep its gen module → re-derive f25 gen-hash pins +
the re-pinned nudge witnesses → run the full union (no lane ran any of them).
The e11 `tileobject::check_water_death/check_lava_death` seam is a merge-order
dependency for A1's death check (e11 not in main yet).

### A7. g46d RE-FOLD RECEIPT (W4-g45 LIQUID FINAL, 2026-11-21) — the guards are STILL missing; A6 is SUPERSEDED by research/70 §8

Re-read of `origin/task/W4-g46-liquid-evaporation` at the LIQUID-FINAL phase:
the tip is STILL **`f57d7bacd`** (no commit since the graft-prep UNION; g46d
has not landed). Every A1-A3 flag re-verified against the union blob, line
numbers fresh:

| Flag | Union evidence (`f57d7bacd:crates/tr-core/src/liquid.rs`) | Status |
| --- | --- | --- |
| A1 border/grate/death guards | `add_water` :381-409 is the `&World` queue-only form — bounds `x<0/y<0/x>=width/y>=height` only, `tile_is_solid(t)` with NO `kind != 546` carve-out, returns right after the queue (no `check_water_death`/`check_lava_death`, no `kill_tile`) | **RE-FLAGGED — still missing** |
| A1 `&mut World` ripple | `add_water(&World)` :381, `wake_around(&World)` :413 | **RE-FLAGGED** |
| A2 widened fixtures | `basin` :1859 = `World::new(width, wall_y+3)`, scene at x=0, no containment walls; `floor_world` :3321 un-widened | **RE-FLAGGED** |
| A3 down-arm stale read | `let down = …copied()` :1131 BEFORE `kill_tile` :1139; the gate :1143 reads the pre-kill copy | **RE-FLAGGED** |
| A4 fall-arm `own_now` | :832 `let own_now = world.tiles[i].liquid` after the evap block | present (keep) |
| A4 bucket module / shimmer-flow / fall-arm witnesses | `editing/` tree has doors/sentry/wiring only (no `buckets.rs`); zero hits for `shimmer_falls_every_visit_like_water` / `the_fall_arm_moves_the_post_evaporation_level` / `liquid_downarm_live` / `liquid_addwater_guards` | **RE-FLAGGED — all four g45 carries absent** |

**The graft base moved.** g47 advanced past `dd66fa3a` to **`5c09a68b8`**
(R-12-5 closeout 1-4: `e420e351e` §8 re-fold obligation, `4c81731bc` the
full-fn census + the per-liquid settle trio, `5c09a68b8` §9-12 the FINAL
merger checklist). **research/70 §8 now owns the re-fold shape**: g46d must
RE-BASE onto the g47 tip (not arm-merge the union), re-apply only the g46
evap/nudge + g45 extra-wake deltas + the witness union, and rename
`retire_kills` → `edit_kills`. A6's graft order above is superseded where
they disagree — the A1/A2/A3 g45 carries ride research/70's step 5 (guards,
with the `edit_kills` loot upgrade) and the §11 re-pin table; the bucket
module + the three g45 witness modules (A4) are NOT in research/70's tables
(they postdate its census scope) and travel per THIS addendum. The g47 tip
itself still carries BOTH stale reads (down-arm pre-kill copy at
`5c09a68b8:liquid.rs:1274`, fall-arm `amount > start.liquid` at :1015) — the
g45 fixes (`c8908a55d` re-read, `8d581953b` `own_now`) are graft obligations
on the NEW base too, not just on the union. The per-fn owner table of record
is research/104 §5-final.

### A8. g46d-NOW-LANDED receipt (W4-g45 LIQUID RETIRE, 2026-11-21) — A7 is STALE: the guards ARE restored; ONE stale read survives

Re-read of `origin/task/W4-g46-liquid-evaporation` at the LIQUID-RETIRE
phase: the tip is **`9bc722e5f`** — the g46d RE-FOLD LANDED as the 4-step
chain `e7626b72d` (base swap onto g47 `95e9c1ef` wholesale) → `f588d53c5`
(g47c `d3a30ebf` folded) → `16fd6ad78` (g45b `8e81814e` AddWater guards +
the union carries) → `d95911ce1` (the g47d R-12-5 settle drivers) →
`9bc722e5f` (docs). Line numbers below are fresh against
`9bc722e5:crates/tr-core/src/liquid.rs` (5323 lines):

| A7 flag | Re-fold evidence (`9bc722e5:liquid.rs`) | Status |
| --- | --- | --- |
| A1 border/grate/death guards | `add_water(&mut World)` :565; the 5-tile border :572; the `kind != 546` carve-out :582; the inline `check_lava_death`/`check_water_death` :624-628 → `kill_tile` :630 with the **`edit_kills` LOOT UPGRADE** :631 (`drain_edit_kills` :510) | **RESTORED** |
| A1 `&mut World` ripple | `add_water` :565 + `wake_around` :640 both `&mut World` | **RESTORED** |
| A2 widened fixtures | `basin` :2420 = (+6,+6) embed in a +12/+12 world WITH containment walls at x=5/x=width+6 (:2443-2448); the ~40 module fixtures widened (cadence :4404, evap pocket :4254, guards :4906, downarm-adjacent scenes); the `new_flow_fixture` `i32::MAX` evap sentinel :481-483 | **RESTORED** |
| A3 down-arm stale read | `let down = …copied()` **:1414** BEFORE `kill_tile` :1422; the gate **:1426** still reads the pre-kill copy (`!down.active()` on the stale cell); no post-kill re-read anywhere in the arm | **STILL PRE-KILL — the ONE surviving stale read** |
| A4 fall-arm `own_now` | :1112 `let own_now = world.tiles[i].liquid` after the evap block :1043-1051; the fall reads it :1119/:1123/:1126 | **IN the re-fold (keep)** |
| A4 guards witness module | `liquid_addwater_guards` :4900 — all ×10 fns present, the inline-death witness asserts the queued `TileKill` on `edit_kills` (the research/70 re-pin) | **RESTORED** |
| A4 bucket module | `editing/` tree = doors/sentry/wiring only — NO `buckets.rs`, no `use_buckets` in `editing.rs` | **ABSENT (by owner-table design — g45-unique, travels at the merge, NOT a g46e gap)** |
| A4 downarm/fall-arm/shimmer-flow witness modules | zero hits for `liquid_downarm_live` / `the_fall_arm_moves_the_post_evaporation_level` / `shimmer_falls_every_visit_like_water` / `shimmer_falls_every_visit_ungated` | **ABSENT (g45-unique fold-source, same rule)** |

Extra re-fold carries verified in passing: the g45 extra wakes ARE in
(:1200-1212 7-arm / :1222-1229 5-arm, the centre-diff reduction intact);
the `surface_rng` nudge :248/:470/:1259 (world-seeded, NOT g45's
`frame_rng`); the evap block's immediate `KILL_DEAD` retire :1048-1050
(g46 faithful form — g45's post-block zero-check is the dropped form);
`update_world_tick` :662 (skipCount driver); the g47 machinery symbols
(`reduced_max_liquids`, `gen_settle_loop`, `quick_water`,
`settle_water_at`, `set_gen_dungeon_context`, `shimmer_remove_water`)
all present.

#### A8-graft. The EXACT graft-hunk for g46e-seal (the down-arm LIVE re-read)

Target: `9bc722e5:crates/tr-core/src/liquid.rs`, `liquid_check`'s down arm
(:1413-1428). The gate at :1426 and the merge-type reads at :1436-1441
run AFTER the burn in C# (`tile4` is a live reference — `Liquid.cs:1286`/
`:1303` follow `:1280`'s `KillTile`), so the seven cut kinds OUTSIDE
`tileObsidianKill` (444/518/519/549/637/638/711) pass on `!tile4.active()`
once the kill cleared them. The pre-kill copy wrongly refuses. Apply
verbatim (the g45 `c8908a55d` form; `flag` stays the PRE-kill read — the
C# computes it at `:1274`, before `:1278`'s burn; `remove_tile` keeps the
liquid + type bits, `WorldGen.cs:63885`, so the amount/type reads below
see the same liquid the C# does):

```rust
             if this_liquid_type != 0 && crate::tileobject::tile_cut(down.kind) {
                 crate::edit::kill_tile(world, x, y + 1, false);
                 // `Main.netMode == 2` → `NetMessage.SendData(17, …)` (`:1309`)
                 // — no multiplayer in the port.
             }
+            // W4-g45 doc-sweep: the C# `tile4` is a LIVE reference — the gate
+            // (`:1286`) and the merge-type reads (`:1303`) run AFTER the burn,
+            // so a `tileCut` kind the kill just cleared passes on
+            // `!tile4.active()` even when it is not `tileObsidianKill` (the
+            // seven cut kinds outside the set: 444/518/519/549/637/638/711).
+            // The pre-kill copy refused those; re-read the cell. The kill
+            // keeps the liquid + type bits (`remove_tile`, `WorldGen.cs:63885`
+            // clears everything but wall/liquid), so the amount/type reads
+            // below see the same liquid the C# does. `flag` stays the PRE-kill
+            // read — the C# computes it at `:1274`, before `:1278`'s burn.
+            let down = world.tile(x, y + 1).copied().unwrap_or_default();
             if !((!down.active() || td::TILE_OBSIDIAN_KILL.has(down.kind)) || flag) {
                 return;
             }
```

Companion carries for the same seal (g45-unique, per the owner table
research/104 §5-final — they postdate research/70's census scope):
1. the `liquid_downarm_live` ×4 witness module (g45 tip `326808019:liquid.rs:3061-3165`
   — `the_burned_non_obsidian_kill_plant_lets_the_sub_24_drain_run` /
   `the_burned_lily_pad_lets_the_full_merge_land_below` /
   `the_honey_arm_burns_and_merges_the_same_way` /
   `the_water_arm_never_burns_the_non_obsidian_kill_plant`). The module's
   `floor_world(9,120,8)` (+6,+6) fixture and the direct
   `sys.liquid_check(&mut w, …)` calls are signature-identical on the
   re-fold base (:1347) — graft as-is.
2. `the_fall_arm_moves_the_post_evaporation_level` (g45 :2342-2361): the
   re-fold base HAS the `own_now` behaviour but NO pin; the witness rides
   the widened `shaft`/`floor_world` helpers the base already carries
   (:4404/:4420) and `LiquidSystem::new` (the band is live there — no
   sentinel wanted). Append into the base's `liquid_merge_cadence`.
3. the shimmer-flow pin: keep ONE of g45 `shimmer_falls_every_visit_like_water`
   (:2424-2441, the cadence-trace assertions) and g47 `shimmer_falls_every_visit_ungated`
   (`4c81731bc:liquid.rs:3002-3021`, the shelf fixture + the `entry.delay == 0`
   assert). NOTE the re-fold dropped BOTH (the g47 base `95e9c1ef` predates
   the closeout commit that wrote the twin) — the merged pin is a g46e
   obligation, not a dedupe of something already there. The g47 fixture
   calls `sys.add_water(&w, …)` (pre-guard signature) — re-pin to
   `&mut w` at the graft.
