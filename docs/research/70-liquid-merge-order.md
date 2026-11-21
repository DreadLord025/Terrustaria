# 70 — FINAL liquid merge order (W4-g47, the closing verdict)

Date: 2026-11-21. Author: W4-g47 (final phase, code-only — no cargo/tests run).
Supersedes research/69's "Merger order suggestion" where they disagree; the
per-function KEEP table in 69 stays valid EXCEPT the rows this note re-marks.

## The lane tips this order is written against

| Lane | Branch | Tip at writing | Role |
| --- | --- | --- | --- |
| base | `main` | `a725acb0` (wave-3 landing) | the common ancestor of every lane below |
| g45 | `task/W4-g45-liquid-lava-honey-delays` | `8e81814e` | DelWater v1 + delays + nudge/extra-wakes (`ccfe0b42`) + round-2 AddWater guards (`daa1cdb0`) |
| g46 | `task/W4-g46-liquid-evaporation` | `f3b33b92` (research/69; **g46c graft in flight on this branch**) | evap + `surface_rng` nudge + `square_tile_frame_record` loot seam |
| g47 | `task/W4-g47-liquid-skip-settle` | `d222af69` (this branch: `95e9c1ef` wiring + `d3a30ebf` final code + `d222af69` verdict docs) | quickSettle/settle/panic/stuck/buffer skeleton, DelWater canonical, load hook WIRED, reduced-liquids + dungeon re-type + ShimmerRemoveWater |
| f25 | `task/W4-f25-worldgen-passes-8` | `803093b1` | the gen `Settle Liquids` driver (`worldgen/settle_liquids.rs`) + gen-local liquid copies |
| f24 | `task/W4-f24-worldgen-passes-7` | `9f2276d5` | `DungeonBounds`/`create_potential_dungeon_bounds`/`DualDungeonSetup` (the dungeon-retype producer) |
| c30 | `task/W4-c30-dither-snake` | `8f5f1086` | the dither-snake variant of the same producer chain (dedupe onto f24's at the worldgen merge) |
| g36 | `task/W4-g36-worldgen-missing-a` | `6d46f6a5` | `gv.shimmer_position` (GenVars slot + the Shimmer-step write, `WorldGen.cs:15241`) — the ShimmerRemoveWater producer |

All lanes are code-only: every witness named below is **written, NOT run**.

## Which branch is the liquid base

**g47 (`task/W4-g47-liquid-skip-settle`) is the structural base for
`crates/tr-core/src/liquid.rs` and its app consumers.** It carries the only
complete runtime surface (quickSettle arms, DelWater + keep-alive + splice,
buffer swap-remove counts, stuck band, panic, QuickWater/SettleWaterAt/
AttemptToMove* family, ReInit, WaterCheck, the WIRED load hook) plus the
final-phase deferral closures. research/69 row 1's "graft g46's body into
g47's skeleton" recipe is **superseded**: g47's `del_water` at `d3a30ebf`
already contains every g45 detail (adopted `551c1e7b`) AND g46's last delta
(the `square_tile_frame_record` loot seam, one `edit_kills` queue). The
ANALYSIS section "The g46c graft-union verdict" is the authoritative
adoption list; g46c must take g47's `del_water`/`end_pass`/`add_water`
bodies wholesale.

## Graft order (each step a commit on the g46 branch — g46c)

1. **Base swap**: take g47's `liquid.rs` whole (tip `d222af69`) as the new
   file base, plus its `edit.rs`/`edit/kill.rs` exports
   (`square_tile_frame_record`), `settings.rs` (`reduced_max_liquids` +
   `apply_launch_args`), `main.rs` (launch-arg application), `app/mod.rs`
   (the `App::new` settle hook + reduced-flag seed), `app/netglue.rs` (the
   flag re-seed) and `world_sim/gfx_quality.rs` (the reduced caps). g46's
   own DelWater/`end_pass` bodies are dropped, NOT merged arm-by-arm.
2. **g46 evap graft** (research/69 row 9, unchanged): the `:466-474`
   unguarded `y > underworld_layer` drain + the `underworld_layer` field
   (`height-200`, runtime-faithful) + the `#[cfg(test)] new_flow_fixture`
   sentinel (`i32::MAX`). Re-apply onto g47's `update_tile` head — g47's
   file has no evap block, so this is a clean insertion, no conflict.
3. **g46 nudge graft** (row 10, unchanged): the `:892-895` arm + the
   per-consumer `surface_rng` (world-seeded `UnifiedRandom`). g47's
   `update_tile` 3-neighbour arm has no nudge — clean insertion. Keep
   g46's stream pick (NOT g45's `frame_rng`); g45's three nudge witnesses
   re-pin onto `surface_rng` (row 10's note).
4. **g45 carries** (row 12): the 7/5-neighbour extra wakes (`ccfe0b42`,
   `:748-771`/`:821-836`) onto g47's averaging arms — the one
   non-superseded g45 sim item. The delay pins (`lava_falls_once_every_six_visits`
   etc.) ride along; g47 has the machinery.
5. **g45b AddWater guards** (the verdict's item 3 — 69's "stays deferred"
   is stale): port `daa1cdb0`'s guards onto g47's `add_water`: the `:1163`
   border exclusion + `type != 546` carve-out, the `:1183-1206` inline
   death check (g47's `generating` field IS `isGeneratingOrLoadingWorld` —
   g45b's gate is already right), the pre-loop num8 promotion pin, and the
   (+6,+6) fixture embeds. **Upgrade**: g45b's inline `kill_tile` discards
   the record — push the `TileKill` onto `edit_kills` instead (the same
   loot-seam argument as the DelWater keep-one). The `&mut World` ripple
   (`wake_around`/promotion callers + the app callers) is g45b's own; g47's
   `add_water(&World)` signature changes to `&mut World` in this step —
   expect mechanical ripples in `liquid.rs` internals + `world_sim` callers.
6. **f25 gen module** (rows 4/5/11): keep `worldgen/settle_liquids.rs`
   wholesale BUT dedupe its private copies onto the g47 surfaces:
   - its `shimmer_remove_water(world, gv)` → `sys.set_gen_shimmer_position(gv.shimmer_position)`
     + the LiquidSystem arm (delete the private fn);
   - its `in_any_potential_dungeon_bounds` + landing-wall read →
     `sys.set_gen_dungeon_context(...)` built from the f24 producer (see
     step 7); **the wall read must become the ORIGIN cell's** (`Liquid.cs:152`
     binds `tile` at the origin; f25 reads the landing cell — a real
     deviation, fix at this graft);
   - its gen-local `settle_water_at`/`quick_water` may stay gen-local
     (69 rows 4/5 "keep both") OR unify onto `LiquidSystem::quick_water`
     now that the gen arms are ported — unification is RECOMMENDED (the
     g47 method + `set_gen_settle_context` covers the whole C# surface);
     if kept gen-local, cite each other and keep the retype semantics
     identical (origin-wall fix applies either way);
   - its quickSettle arms in liquid.rs dedupe onto g47's (row 12);
   - keep the exports `tiles_ignore_water`/`world_gen_tiles_ignore_water`/
     `num_liquid_total`/`clear_pending_liquid`/`liquid_reaction_check`/
     `gen_ignore_water` (f25's; g47's file lacks them — carry them over).
7. **Producer wiring** (the gen driver's context arming, one place):
   - dungeon: convert f24's `DualDungeonSetup.dungeons[i].outer` hitbox to
     `(left, top, right, bottom)` rows + the style quads' wall extensions
     (`DungeonCrawler.cs:155-181`; classic world = empty extension) into
     `GenDungeonContext`; on a single-dungeon world use f25's
     `create_potential_dungeon_bounds`-equivalent single entry. c30's
     duplicate producer dedupes onto f24's at the worldgen master merge
     (g36 research/88 owns that matrix).
   - shimmer: `gv.shimmer_position` (g36-A1) → `sys.set_gen_shimmer_position`.
   - the driver arms `set_gen_settle_context(water_line, lava_line, remix)`
     (already g47's seam) and drains/discards `edit_kills` at the pass tail
     (69's row-11 warning — records must not leak across the per-pass
     instance).

## Witness re-pins (the union run happens AFTER the graft, not before)

| Pin / witness | Why it moves | Action |
| --- | --- | --- |
| f25 `world_hash.rs` SEED42_512 / DEFAULT_WORLD + `the_pass_is_deterministic` | 69 row 11: derived against f25's discard-only `end_pass` + FIFO buffer; the graft swaps in the real DelWater drain/tail (grass burns, alch/lily kills now fire during gen) + the swap-remove buffer + the origin-wall dungeon fix | RE-DERIVE, expect a delta, do not silence. The gen-side `edit_kills` drain choice (resolve vs discard) changes the pins too — pick one and record it |
| g45 nudge ×3 (`the_254_nudge_promotes_on_a_zero_genrand_draw` etc.) | stream switch `frame_rng` → `surface_rng` (69 row 10) | RE-PIN the seed-35/seed-0 draw values onto `surface_rng` |
| g45 evap witnesses | g45's `>=0` guard semantics dropped for g46's faithful form | DROP `water_evaporates_two_per_visit_below_the_underworld_layer`; RE-PIN the layer-row boundary witness onto g46 (`y > layer` strict); keep g46's four |
| g46 `the_retire_sweep_splices_backwards_and_the_lily_tail_runs_on_drained_cells` | g47's splice is INSIDE `del_water` before the tail | RE-RUN against the g47 shape (tail-before/after equivalence documented in the keep-one table) |
| g45 DelWater fixture witnesses (`lava_retirement_burns_the_grass_ring`, alch/lily tails, buffer/stuck pairs) | 69's witness-union table: keep ONE of each overlapping pair (g47 names), merge g45's sharper assertions (delay-inheritance, delay-0, kill-0 survivor, mud-60 ring, grass-family ground) | UNION per 69's table; re-point the loot assertions at `edit_kills` (was `retire_kills` on g46, discarded returns on g45) — now also the grass-ring RECORDS exist on g47 (`d3a30ebf`), so g45's ring witness can assert the record, not just the conversion |
| g45b AddWater guard witnesses ×10 | the `&mut World` signature + the `edit_kills` loot upgrade | RE-RUN as-is except the inline-death witness: assert the queued `TileKill` instead of the discard |
| save/load round-trip pins (`saveglue.rs`) | the load hook is WIRED (g47 `551c1e7b`) — every fixture is DRY, dry settle is a byte/RNG no-op (witnessed) | HOLD; any future WET-world pin must re-derive against the settled state (the ANALYSIS re-pin obligation) |
| `liquid_sim::mass_conserved` + the ~20 flow pins | g46 evap drains water below `height-200`; small fixtures would evaporate everywhere | the `new_flow_fixture` sentinel (`underworld_layer = i32::MAX`) keeps them; every flow witness must use the sentinel fixture (g46 convention) |
| g47 final-phase witnesses ×8 (`d3a30ebf`) | none — they test the grafted-in surfaces directly | RUN as written |

## Gate checklist for the g46c landing

1. `cargo test --workspace -- --skip migrates_version_2_kinds_to_item_ids`
   green AFTER the re-pins (judge by `test result: ok` lines).
2. The anchors canary: g47's rows (`fn del_water`, `fn quick_water`,
   `settle_loaded_world`, `fn shimmer_remove_water`, `struct GenDungeonContext`,
   `reduced_max_liquids`, `pub fn square_tile_frame_record`, …) must all
   survive the graft — a failed marker means the base swap dropped a g47
   surface; recover the arm, never delete the row.
3. The runtime-inert gen-fields contract (69 §contract) holds: `generating`/
   `generating_world`/`gen_ignore_water`/`quick_settle` default false; the
   gen context setters are called ONLY by the gen driver; the load driver
   arms `generating` but never `generating_world`; `underworld_layer` is
   runtime-faithful (NOT gen-gated); `surface_rng` advances only on a 254
   average.
4. `graphify update .` on the merged branch (code-shape change), then the
   main merge per MAIN-RULES.

## R-12-5 settle drivers addendum (g47 continuation, 2026-11-21)

The recorded remainder ("full Settle Liquids drivers remain R-12-5") landed
on this branch in four per-driver commits, all code-only (witnesses written,
NOT run):

| Driver | Commit | Scope |
| --- | --- | --- |
| runtime pump | `3dbbe3ef` | `LiquidSystem::update_world_tick` = `WorldGen.cs:71526-71534` verbatim (skipCount on the Liquid statics' home, `ReInit` :85 resets); `world_sim/liquids.rs` consumes it, the app `liquid_tick` divider REMOVED (never reset on world swap — a phase deviation the port fixes); the `SwapRandom("UpdateLiquid")` seam stays the caller's `edit_kills` drain |
| buffer arm | `d9445fc4` | census re-verify of the g45b-landed promotion loop (`Liquid.cs:1120-1130`): budget = min(post-sweep numLiquid, buffer), unconditional `DelBuffer(0)` even on an `AddWater` refusal; 2 new witnesses |
| f25 gen split | `4288657c` | f25's export surface carried onto the g47 machinery (step 6's carry-over list: `tiles_ignore_water`/`world_gen_tiles_ignore_water` + the `gen_ignore_water` `kind_solid` fold/`set_bubble_solid`/`num_liquid_total`/`clear_pending_liquid`/`liquid_reaction_check`) + the shared `gen_settle_loop` (`:16158-16206` verbatim, Part-2 twin classic-identical); f25's `set_generating` maps onto `set_gen_settle_context`; its "no RNG draws" claim holds only for its discard-only `end_pass` — the g47 kills ride `edit_kills`, the world-hash RE-DERIVE obligation above stands |
| e21 pumps | `ae688401` | the `XferWater` `SquareTileFrame` wakes (`Wiring.cs:512/:516/:521` → `WorldGen.cs:81344-81346` `AddWater`) were unported — pump-teleported liquid never joined the settle lists; `Wiring::take_pump_wakes` + the app `wake_around(r=1)` drain compose the pump with the settle pump |

The g46c graft order above is unchanged: step 6's dedupe now finds the
export surface ALREADY on the g47 base (delete f25's private copies, call
these); step 7's producer wiring still owns the context arming. The
witness stream-pins (the g45/g46 `Next(30)` nudge contract) are untouched —
no nudge code lives on this branch and none of the four drivers draws RNG.

## §8 — g46d re-fold obligation (the union tip post-dates NOTHING; the drivers post-date IT)

Status re-read 2026-11-21 against `origin/task/W4-g46-liquid-evaporation`
tip **`f57d7bacd`** ("W4-g46 graft-prep: UNION liquid.rs", 07:06 +0200).
The union's own landing record (research/69 §"Graft-prep landing record")
names its structural base as **g47 `cfe5f1c9`** — the phase-4 tip, BEFORE
the settle wiring, the final-phase surfaces and the four R-12-5 drivers.
The g47 branch has since advanced to **`dd66fa3a`**; the union is therefore
**stale by eleven commits** and g46d (the active re-fold on that branch)
MUST carry the delta. Verified by symbol grep on `f57d7bacd`'s blobs —
every surface below has **zero hits** in the union:

| Missing surface (g47 `dd66fa3a`) | Union evidence | Carrying commit |
| --- | --- | --- |
| `LiquidSystem::update_world_tick` + `skip_count` (the `WorldGen.cs:71526-71534` runtime pump) | union `liquid.rs` has no `skip_count`; union `world_sim/liquids.rs` still walks `app.liquid_tick % 2` and union `app/mod.rs` still declares `liquid_tick` (the divider g47 REMOVED — its C# home is the Liquid statics, `ReInit` :85 resets it) | `3dbbe3ef1` |
| the buffer-arm witnesses (budget growth + unconditional DelBuffer) | union `liquid_settle` lacks both names | `d9445fc4f` |
| `gen_settle_loop` + the f25 export surface: `tiles_ignore_water`, `world_gen_tiles_ignore_water` (+ the `gen_ignore_water` `kind_solid` fold), `set_bubble_solid`, `num_liquid_total`, `clear_pending_liquid`, `liquid_reaction_check` | zero grep hits in union `liquid.rs`; the union `kind_solid` has no door/leaf carve-out | `4288657ce` |
| `Wiring::take_pump_wakes` + `pump_wakes` (the `XferWater` `SquareTileFrame` wakes, `Wiring.cs:512/:516/:521`) + the app `wake_around(r=1)` drain | zero hits in union `wiring.rs` / `world_sim/wiring.rs` — pump-teleported liquid never joins the settle lists on the union | `ae6884017` |
| `set_gen_settle_context`/`set_gen_shimmer_position`/`set_gen_dungeon_context` + `GenDungeonContext` + the `settle_water_at` gen re-type arm (`Liquid.cs:178-192`, ORIGIN-wall read) | union `settle_water_at` has NO re-type block (its `remix_world_gen` reads are the `lava_check` desert gate only) | `551c1e7ba` + `d3a30ebf2` |
| `shimmer_remove_water` (both `QuickWater` arms, `WorldGen.cs:34823-34851`) | zero hits in union `liquid.rs` | `d3a30ebf2` |
| `reduced_max_liquids` (settings field + `slowliquids=1` launch arm + `ReInit` 5000 cap + gfx 2500+2500q/flat-5000 override + the pass-head `numLiquid > 2000` quickFall OR-arm) | zero hits in union `settings.rs`/`liquid.rs`/`gfx_quality.rs` (the union pass head is `quick_fall = quick_settle` only, its own comment admits the OR-arm "lands with the reduced-liquids field") | `d3a30ebf2` |
| the WIRED load hook (`App::new` settle + `TR_NO_SETTLE`/`TR_POOL`/`TR_SPOUT` staging, `saveglue`) | union `app/mod.rs` has no `settle_loaded_world` call; union `saveglue.rs` unchanged from base — the union's `settle_loaded_world` is the UNWIRED phase-3 form | `551c1e7ba` (+ `95e9c1eff` witness shaft wall) |

**Exact commit list g46d must re-fold** (code, in order): `551c1e7ba`,
`95e9c1eff`, `d3a30ebf2`, `3dbbe3ef1`, `d9445fc4f`, `4288657ce`,
`ae6884017`. Doc re-fold alongside: `ba6946530`, `d222af697`, `fdeb897ca`,
`dd66fa3aa` (ANALYSIS/PORT_MAP/research-70/anchors — research/70 never
existed on the union's tree, its base predates `fdeb897ca`; carry it over,
this file is the merge order of record).

**Shape of the re-fold**: do NOT merge arm-by-arm. Re-base onto g47
`dd66fa3a`'s `liquid.rs` + `edit.rs` + `edit/kill.rs` +
`world_sim/editing.rs` + the app consumers (`app/mod.rs`, `settings.rs`,
`main.rs`, `env.rs`, `app/netglue.rs`, `app/saveglue.rs`,
`world_sim/liquids.rs`, `world_sim/gfx_quality.rs`, `world_sim/wiring.rs`,
`wiring.rs`) and re-apply ONLY the union's own deltas: the g46 evap block
+ `underworld_layer` + `new_flow_fixture` sentinel, the `surface_rng`
Next(30) nudge, the g45 7/5-neighbour extra wakes, and the witness union.
Naming keep-one: the queue is **`edit_kills`** (g47 name; the union's
`retire_kills`/`drain_retire_kills`/`resolve_liquid_retire_kills` rename
back), and `edit/kill.rs` keeps the g47 `check_alch`/`check_lily_pad`
(closure `kind_solid`) form — the union took the g46 `check_alch_tile`
form whose `del_water` tail callers do not exist on the g47 body. The
union's `del_water` is NOT the canonical one: research/70's base verdict
(g47 `del_water` wholesale, `d3a30ebf` loot seam) still stands; the union
re-derived the same seam under its own queue name, so the re-fold is a
rename + re-base, not a behaviour merge.

**Witness carry**: the union's 72 pins are 49 shared + 23 union-only; the
g47 tip carries 20 pins the union lacks (the driver/final-phase/wiring
suites — `the_update_world_driver_runs_the_pass_every_second_tick`,
`re_init_resets_the_driver_divider`, the two promotion-budget pins, the
two load-settle wiring pins, the four gen re-type/dungeon pins, the two
shimmer-remove pins, the two reduced-liquids pins, the two gen-driver
export pins, the two `gen_settle_loop` pins,
`world_gen_tiles_ignore_water_lifts_the_door_family`,
`clear_pending_liquid_drops_both_lists_and_their_checking_bits`,
`delwater_sub2_dry_at_the_world_edge_does_not_panic`,
`delwater_lava_grass_ring_converts_and_dirties_the_pass`). The re-fold
takes BOTH sets (keep-one dedupe per §11's final table); the wiring-side
compose pins (`wiring::tests::the_pump_moves_water_and_adopts_the_type`
extended + `world_sim::wiring::tests::r_12_5_the_pump_transfer_wakes_the_settle_and_the_water_flows`)
ride `ae6884017` and must survive too.