# 69 — Liquid 4-way reconcile (W4-g46 merger assist)

Date: 2026-11-21. Author: W4-g46 (reconcile phase, code-only — no cargo/tests run).

> **SUPERSEDED-BY-252 (was 70; RE-FOLD phase, 2026-11-21).** This note was written
> against g47 `cfe5f1c9` — BEFORE the settle wiring (`551c1e7b`/`95e9c1ef`),
> the final phase (`d3a30ebf`) and the R-12-5 drivers (`3dbbe3ef`..`ae688401`).
> **`docs/research/252-liquid-merge-order-g46.md` (g47 `fdeb897c` + the driver
> addendum `dd66fa3a`) is the authoritative merge order**; where this note
> disagrees, 252 wins. Specifically superseded: row 1's graft recipe (g47's
> `del_water` is canonical WHOLESALE — the g45 details AND g46's
> `square_tile_frame_record` loot seam are already adopted at `d3a30ebf`;
> do NOT graft g46's body over it), row 8's "load hook stays UNWIRED"
> (WIRED at `App::new` since `551c1e7b`, dry-world no-op witnessed), the
> fixture note's "AddWater border guard stays deferred" (g45b landed the
> guards WITH the (+6,+6) widening — carried per 252 step 5), and the
> `retire_kills` queue-name recommendation (the union keeps g47's
> `edit_kills`/`drain_edit_kills`/`resolve_liquid_edit_kills` names). The
> per-function KEEP table below stays valid EXCEPT those re-marks; the
> witness-union dispositions were executed per 252's re-pin table.
>
> **UNION-CONTRACT — FINAL (RE-FOLD landing, 2026-11-21).** The union on
> `task/W4-g46-liquid-evaporation` was REBUILT in research/252's 7-step
> order (the stale `f57d7bac` graft-prep union was replaced, not patched):
>
> | Step | Commit | What landed |
> | --- | --- | --- |
> | 1 | `e7626b72` | g47 `95e9c1ef` WHOLESALE base (`liquid.rs`, `edit.rs`, `edit/kill.rs`, `app/mod.rs` WIRED settle hook, `saveglue.rs`, `env.rs`, `world_sim/editing.rs`/`gfx_quality.rs`/`liquids.rs`); g46 evap (`underworld_layer` height−200, unguarded `y > layer`, live `own_now` fall read, `new_flow_fixture` i32::MAX sentinel) + `surface_rng` nudge + g45 7/5-extra-wakes re-grafted; `edit_kills` canonical (the `retire_kills` name and the `check_alch_tile` wrapper DROPPED; `check_lily_pad` is g47's `raw_solid`-closure form) |
> | 2 | `f588d53c` | g47 `d3a30ebf`: `reduced_max_liquids` (settings + `apply_launch_args` + launch-path skip + App/netglue seeds + ReInit 5000 cap + gfx 2500+2500q/flat-5000 + `numLiquid>2000` quickFall OR-arm), `GenDungeonContext` + the `:188-191` dungeon re-type with the ORIGIN wall read (`:152` — f25's landing-wall read is its deviation), `ShimmerRemoveWater` both QuickWater arms (g36-A1 `set_gen_shimmer_position`), the g46 `square_tile_frame_record` adoption (`edit.rs` export + `kill.rs` whole-record split); 8 witnesses |
> | 3 | `16fd6ad7` | g45b `8e81814e`: the `:1163` border + 546 carve-out + `:1183-1206` inline death on `add_water(&mut World)` with the **`edit_kills` loot upgrade**; the `&mut World` ripple (5 app callers + every test); ~40 fixtures (+6,+6)-widened with containment walls; the g45 witness union (`liquid_delwater` ×5 sharper incl. the grass-ring RECORD assertion, `liquid_nudge_wakes` ×3 RE-PINNED onto `surface_rng` + the 7-arm extra-wake RE-DERIVE, `liquid_addwater_guards` ×10 with the queued-`TileKill` assertion); g46's `TileFrameImportant` case-518 dispatch restored into `square_tile_frame` (`WorldGen.cs:84966-84968`) |
> | 4 | `d95911ce` | g47d R-12-5 ×4: `update_world_tick` (skipCount verbatim, ReInit reset) + the app `liquid_tick` divider REMOVED; the buffer-arm witnesses ×2; the f25 export surface (`tiles_ignore_water`/`world_gen_tiles_ignore_water` + `gen_ignore_water` fold/`set_bubble_solid`/`num_liquid_total`/`clear_pending_liquid`/`liquid_reaction_check`) + `gen_settle_loop` + 4 witnesses; the e21 `Wiring::pump_wakes`/`take_pump_wakes` compose + the app `wake_around(r=1)` drain |
>
> **The merger keeps THIS branch's `liquid.rs` + the supporting files above;
> drops the g45/g47 lane versions.** Steps 6/7 (the f25 gen-module dedupe
> onto the export surface + context setters — INCLUDING the origin-wall fix
> — and the f24/c30/g36 producer wiring) execute at the worldgen master
> merge; the surfaces they call are ON this branch. Witness re-pins per 252's
> table: the f25 `world_hash.rs` SEED42_512/DEFAULT_WORLD +
> `the_pass_is_deterministic` **RE-DERIVE** (the real DelWater drain/tail +
> swap-remove buffer + origin-wall fix change the gen settle; the gen-side
> `edit_kills` drain choice — resolve vs discard — must be picked and
> recorded there); the g45 nudge ×3 are RE-PINNED (done, step 3); the g45
> evap duplicate DROPPED + the layer-row boundary RE-PINNED onto g46 (done);
> the g46 retire-sweep/lily witness RE-RUN against the g47 splice shape
> (done, note in the witness); the DelWater fixture union done (g47 names +
> g45 sharper assertions, loot on `edit_kills`); the g45b guard witnesses
> RE-RUN as-is except the inline-death one (asserts the queued record);
> the save/load round-trip pins **HOLD** (every fixture DRY, the dry settle
> is a byte/RNG no-op — witnessed); every flow witness uses the
> `new_flow_fixture` sentinel. **102 `#[test]` fns** in the union
> `liquid.rs`, all written-NOT-run (code-only); the merger runs the full
> union post-graft (`cargo test --workspace -- --skip
> migrates_version_2_kinds_to_item_ids`, judge by `test result: ok`) and
> the anchors canary (252 gate 2 — every g47 row survives on this branch).

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

## Graft-prep landing record (SUPERSEDED — historical, the `f57d7bac` union)

> **SUPERSEDED-BY-252 (was 70) / the FINAL UNION-CONTRACT above.** The record below
> describes the STALE graft-prep union (`f57d7bac`, built against g47
> `cfe5f1c9`): its `retire_kills` queue name, `check_alch_tile` wrapper,
> pure-form `check_lily_pad`, UNWIRED load hook and arm-by-arm DelWater
> graft were all replaced by the RE-FOLD (`e7626b72`..`d95911ce`). Kept for
> the audit trail only — do NOT merge against this section.

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
