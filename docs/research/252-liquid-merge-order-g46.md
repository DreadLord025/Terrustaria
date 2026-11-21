# 252 — FINAL liquid merge order (W4-g47, the closing verdict; g46 fork copy)

> Chain-banner: 70-fork → 252 per research-number-registry §6.3 (round-1
> owed rename, listed in §10.4): `70-liquid-merge-order.md` (the W4-g46
> fork copy, first add `9bc722e5f`) → `252-liquid-merge-order-g46.md` —
> 70 belongs to d53's proj-dust-site-batches-claims (round-1 winner);
> g47's copy keeps its **209**; g46's 69 KEEP unchanged. The §8-15
> docs-tail extension (`789bdf8aa` + `acd7c84b5`) is preserved whole.
> Cite THIS registry §6.3/§10.4.

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

> **SEAL STATUS (W4-g46 seal phase, 2026-11-21).** This section was written
> against the stale graft-prep union `f57d7bacd`. The g46d re-fold it demands
> has since EXECUTED on this branch in research/252's order — the RE-FOLD
> landing record (research/69 §"UNION-CONTRACT — FINAL", ANALYSIS "The g46c
> RE-FOLD landing record") lists the four steps `e7626b72`/`f588d53c`/
> `16fd6ad7`/`d95911ce` that re-based onto g47 `95e9c1ef`→`d3a30ebf`→`8e81814e`
> guards→the R-12-5 drivers. Every "missing surface" row below is now ON the
> union; the §8 obligation is discharged. The g47 closeout (`5c09a68b`) then
> added the three per-liquid witnesses + the §10 census + the render
> gaps-list, carried by this seal. Kept verbatim below as the merge order of
> record.

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
`dd66fa3aa` (ANALYSIS/PORT_MAP/this note (then key 70)/anchors — this note
never existed on the union's tree, its base predates `fdeb897ca`; carry it
over, this file is the merge order of record).

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
union's `del_water` is NOT the canonical one: research/252's base verdict
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

## §9 — The drivers addendum (what the four R-12-5 commits changed for the merger)

The §"R-12-5 settle drivers addendum" table above stands; its merger
consequences, restated as obligations:

1. **Step 1's base swap now targets `dd66fa3a`**, not `d222af69` — the
   base file list grows by `world_sim/liquids.rs` (the driver consumer),
   `world_sim/wiring.rs` (the wake drain), `wiring.rs` (the wake queue)
   and the `liquid.rs` driver/export surfaces. `app.liquid_tick` is GONE
   on the base — a graft that re-introduces it re-creates the phase
   deviation the driver port fixed (the divider never reset on swap).
2. **Step 6's dedupe is now a DELETE, not a carry**: the export surface
   (`tiles_ignore_water`/`world_gen_tiles_ignore_water`/`set_bubble_solid`/
   `num_liquid_total`/`clear_pending_liquid`/`liquid_reaction_check`) and
   the shared `gen_settle_loop` are ON the base. f25's private copies
   delete; its module calls the LiquidSystem methods. f25's
   `set_generating(remix)` maps onto `set_gen_settle_context` (the fuller
   strict-flag context — same classic result).
3. **Step 7's producer wiring is unchanged** (the context arming stays
   the gen driver's job) with one addition: the gen driver drains or
   discards `edit_kills` at the pass tail — on the g47 machinery the
   DelWater grass-ring/lily kills FIRE during gen (f25's discard-only
   `end_pass` never produced them), so the world-hash RE-DERIVE
   obligation is real, not hypothetical.
4. **The e21 compose is base machinery now**: any lane that ports pump
   behaviour must route through `pump_wakes` → `wake_around(r=1)`; a
   direct grid mutation without the wake leaves the liquid inert (the
   sim walks its ACTIVE list). The three wake sites fire in the C# call
   order (`:512` out, `:516` emptied-in, `:521` in-tail — the emptied arm
   queues twice).
5. **No driver draws RNG** — the `Next(30)` nudge stream-pins are
   untouched by the driver carry; the only new RNG seam is the caller's
   `RngKey::UpdateLiquid` kill resolution (already the g47 shape).

## §10 — Liquid.cs full-fn census verdict (the gaps-list is EMPTY for the sim)

The closeout census (ANALYSIS "The Liquid.cs FULL-FN final census")
script-diffed all 24 `Liquid.cs` members + both `LiquidBuffer.cs` fns
against the port at this tip. **Every sim fn is ported**; the only
non-ported members are MP (`NetSendLiquid`, the broadcast tails, the MP
cycle scaling), UI (`UpdateProgressDisplay`) and dev tooling
(`AddGameplaySnapshotComponents`) — all cited out-of-scope. "LiquidEdge"
does not exist in the decompile (the edge surface is
`LiquidRenderer.InternalPrepareDraw` Pass-4, ported in `liquid_draw.rs`);
the waterfall-source family is ported in `waterfall/`. The two g46-owned
`Update` arms (evap `:466-474`, nudge `:892-895`) and the g45b `AddWater`
guards (`:1163`/`:1183-1206`) are the ONLY sim deltas left, and both are
graft steps (2/3/5), not gaps. The per-liquid settle arms (honey 1-in-11,
shimmer ungated, lava 1-in-6, the exclusive wake cascade, the quickFall
bypass) are verified arm-for-arm — the drivers are kind-agnostic by
design, exactly like the C#. Render-side follow-ups (HasFullWater /
GetVisibleLiquid / SetWaveMaskData / the wind-driven frameState) are
cited in ANALYSIS and do NOT block this merger.

## §11 — FINAL witness re-pin table (supersedes the table above where they differ)

| Pin / witness | Final disposition |
| --- | --- |
| f25 `world_hash.rs` SEED42_512 / DEFAULT_WORLD + `the_pass_is_deterministic` | RE-DERIVE after the graft (the DelWater kills now fire during gen + the swap-remove buffer + the origin-wall fix). Pick the gen-side `edit_kills` drain choice (resolve vs discard), RECORD it, re-derive both pins. Do not silence |
| g45 nudge ×3 | RE-PIN onto `surface_rng` (seed-35/seed-0 values change); the union already carries the re-pinned forms (`liquid_nudge_wakes`) — keep the union's |
| g45 evap ×3 | DROP `water_evaporates_two_per_visit_below_the_underworld_layer`; the layer-row boundary RE-PIN + g46's four are IN the union — keep |
| g46 `the_retire_sweep_splices_backwards_and_the_lily_tail_runs_on_drained_cells` | RE-RUN against the g47 splice-inside-`del_water` shape (in the union; rename its queue reads to `edit_kills`) |
| g45 DelWater fixture pairs (ring/alch/lily/buffer/stuck) | UNION per 69's table (g47 names, g45's sharper assertions merged); loot assertions re-point at `edit_kills`; the union carries them — the re-fold renames, not re-derives |
| g45b AddWater guard ×10 | RE-RUN as-is except the inline-death witness (assert the queued `TileKill`); the `&mut World` ripple is g45b's own |
| save/load round-trip pins | HOLD (dry fixtures, dry settle is a byte/RNG no-op — witnessed); WET-world pins re-derive against the settled state |
| `liquid_sim::mass_conserved` + the flow pins | the `new_flow_fixture` sentinel convention (g46) — every flow witness uses it after the evap graft. The closeout's three new per-liquid witnesses (`honey_falls_once_every_eleven_visits`, `shimmer_falls_every_visit_ungated`, `quick_fall_bypasses_both_delay_gates`) use `LiquidSystem::new` on 120-tall worlds — SAFE pre-graft (no evap block on this lane) and safe for the honey/shimmer/lava kinds post-graft (the evap arm is water-only, `:466` `liquidType() == 0`); still SWITCH them to `new_flow_fixture` at the graft for convention uniformity (g46's rule: every flow witness rides the sentinel) |
| g47 final-phase ×8 (`d3a30ebf`) + settle-wiring ×2 (`551c1e7b`/`95e9c1ef`) | RUN as written — they test the base surfaces directly |
| g47 driver pins ×8 (`3dbbe3ef` ×2, `d9445fc4` ×2, `4288657c` ×4) + the 2 edge/grass-ring pins | RUN as written — the union LACKS all of these (§8's list); the re-fold carries them |
| e21 compose pins ×2 (`wiring.rs` extended + `world_sim/wiring.rs`) | RUN as written — the union lacks the wake queue entirely |
| union-only pins (evap ×5, stream ×1, cadence ×4, down-arm ×3, delwater ×5, buffer/stuck ×2, nudge ×3, extra-wake ×1 = 23) | KEEP the union's forms; the cadence four overlap the closeout's honey/quickFall twins — keep ONE of each pair (the union's `lava_falls_once_every_six_visits` is unique, keep; `honey_falls_once_every_eleven_visits` + `quick_fall_bypasses_the_delay_gate` dedupe onto the closeout's fixture forms or vice versa — same assertions, pick one name) |

Post-graft expected count: 72 (union) + 20 (g47-only) + 3 (closeout) −
the honey/quickFall pair dedupes (2) − the g45 evap drop (already out of
the union) = **93 pins** in `liquid.rs`, plus the 2 wiring-side compose
pins. All written-not-run until the gate.

> **SEAL execution note (W4-g46 seal phase).** The §11 dispositions above
> are now carried on this branch: the closeout trio landed with the
> sentinel switch (`shimmer_falls_every_visit_ungated` on
> `new_flow_fixture`, re-pinned +6 past the AddWater border) and the
> honey/quickFall pair dedupes (the union's
> `honey_falls_once_every_eleven_visits` kept; `quick_fall_bypasses_the_delay_gate`
> widened to the lava+honey twins under the closeout name
> `quick_fall_bypasses_both_delay_gates`). The g45 doc-sweep delta
> (`6f5bd2a2`) added the down-arm LIVE post-burn re-read (`c8908a55`,
> `liquid_downarm_live` ×4), the fall-arm post-evap witness
> (`the_fall_arm_moves_the_post_evaporation_level`), the shimmer-flow pin
> (`shimmer_falls_every_visit_like_water`) and the bucket module — all
> carried by this seal. The union `liquid.rs` now holds **109** `#[test]`
> fns (the 102 RE-FOLD pins + the 3 closeout + the 4 g45 doc-sweep).

## §12 — FINAL gate command + landing checklist (the merger runs this, in order)

1. **Re-fold** (§8): g46d re-bases the union onto g47 `dd66fa3a` (or its
   successor tip — re-fetch first), re-applies the evap/nudge/extra-wake
   deltas + the witness union, renames `retire_kills` → `edit_kills`
   throughout, keeps the g47 `check_alch`/`check_lily_pad` closure form.
2. **Graft steps 5-7** (§"Graft order"): the g45b AddWater guards with
   the `edit_kills` loot upgrade + the `&mut World` ripple; the f25 gen
   module dedupe (DELETE the private copies, call the base exports);
   the f24/c30/g36 producer wiring.
3. **Re-pins** (§11): the f25 gen-hash RE-DERIVE (record the drain
   choice), the sentinel switch on the closeout witnesses, the pair
   dedupes.
4. **The gate command** (judge ONLY by the `test result: ok` lines —
   cargo may exit 1 on success in this project):

   ```
   cargo test --workspace -- --skip migrates_version_2_kinds_to_item_ids
   ```

5. **The anchors canary** — every g47 row survives: `fn del_water`,
   `fn quick_water`, `settle_loaded_world` (app/mod.rs), `fn
   shimmer_remove_water`, `struct GenDungeonContext`,
   `reduced_max_liquids` (settings.rs), `pub fn square_tile_frame_record`,
   `fn update_world_tick`, `pub fn gen_settle_loop`, `pub fn
   take_pump_wakes`, `the_promotion_budget_is_the_post_sweep_active_count_clamped_to_the_buffer`,
   `fn honey_falls_once_every_eleven_visits`, the retire rows
   (`pub struct LiquidAnimClock`, `pub fn wave_mask`, `pub fn
   has_full_water` — liquid_draw.rs; `fn
   the_fall_arm_reads_the_live_post_reaction_level`, `mod
   liquid_downarm_live` — liquid.rs), plus the union's own rows
   (`underworld_layer: i32`, `surface_rng`). A failed marker = a dropped
   surface: recover the arm, NEVER delete the row.
6. **The runtime-inert gen-fields contract** (69 §contract, extended):
   `generating`/`generating_world`/`gen_ignore_water`/`quick_settle`
   default false; the gen context setters are called ONLY by the gen
   driver; the load driver arms `generating` but never `generating_world`;
   `underworld_layer` is runtime-faithful (NOT gen-gated); `surface_rng`
   advances only on a 254 average; `skip_count` resets in `re_init`;
   `app.liquid_tick` stays DEAD (no re-introduction).
7. **`graphify update .`** on the merged branch, then the main merge per
   MAIN-RULES (ref-update, post-merge ancestor + symbol verification).

## §13 — g45d co-lane FINAL reconcile (W4-g47 retire, 2026-11-21)

> **SEAL STATUS (W4-g46 seal phase, 2026-11-21).** §13–§15 carried verbatim
> from the g47 seal tip `ca76d68ee` per §15 Seal 2's docs-wholesale row.
> The g46e delta fold they demand has EXECUTED on this branch: the code
> rows landed as `a4d49b355` (the A9.2 rows 1-8 fold — the down-arm hunk
> verbatim `2bbadd9bd`, the fall-arm comment-only upgrade, the shimmer
> keep-ONE MERGED pin, the quickFall swap + honey dedupe, the render-gaps
> ALL-CODE `a9dba7cf1` + the `bf5fc4ffb` flow.rs compile-rider, the buckets
> module + wire + x15), the steps-6/7 hunk record as `db2322656`
> (research/374, was 71 — registry §10.2 rename `4bcbdd645`). Kept verbatim
> below as the reconcile/seal record of the g47 lane.

Re-read against `origin/task/W4-g45-liquid-lava-honey-delays` tip
**`6f5bd2a2d`** (the doc-sweep landing). The four post-`8e81814e` commits
censused fn-by-fn against this branch's §10 owner table:

| g45 commit | Surface | Owner verdict (FINAL) |
| --- | --- | --- |
| `fa96cf32a` | `Player.ItemCheck_UseBuckets` (`Player.cs:46318-46504`) + `editing/buckets.rs` (455 lines) + `PutItemInInventoryFromItemUsage` + the 5 splashes + 10 witnesses | **g45 owns it whole** — a Player/editing surface, NOT liquid.rs; zero overlap with the g47 base files. The graft takes the module + its `world_sim/editing.rs` wiring as-is; its `add_water(&mut World)` calls ride g45b's own signature ripple (§"Graft order" step 5). No g47 action. |
| `c8908a55d` | the `LiquidCheck` down arm's LIVE post-burn reads (`:1286` gate + `:1303` merge-type reads AFTER the `:1280` burn) + 4 witnesses (`liquid_downarm_live`) | **ADOPTED onto the g47 base this retire phase** — the g47 canonical `liquid_check` had the same pre-kill-copy bug (the down snapshot taken at `:1274` before the burn). The re-read + `flag`-stays-pre-kill form landed verbatim; the 4 witnesses carried into g47's `liquid_downarm_live` module (fixture shape kept — the +6 embed is inert on the guard-free base, becomes the g45b precondition at the graft). **The g46 UNION has the SAME stale read** (`f57d7bacd`'s `liquid_check`: `down` copied before its `kill_tile`, gate reads the copy) — g46d's re-fold takes the g47 base form, which now carries the fix; the union's own copy is superseded, NOT re-merged. Dedupe at the graft: ONE `liquid_downarm_live` module (the g47 copy; g45's is byte-identical bar the module doc). |
| `99a4f8ccf` | `shimmer_falls_every_visit_like_water` (the shimmer no-gate flow witness) | **DROPS at the union** — the g47 closeout's `shimmer_falls_every_visit_ungated` pins the same arm with the sharper zero-delay assertion (§11's pair-dedupe rule, one name kept: the g47 form). §11's count adjusts: the g45 shimmer-flow pin is a THIRD twin of the same pair, not a new pin. |
| `8d581953b` | the fall arm's LIVE post-evaporation read (`own_now`, `:570-575`) + `the_fall_arm_moves_the_post_evaporation_level` | **ADOPTED onto the g47 base this retire phase** — the same `start.liquid` pre-read existed here; on this evap-free lane the live read is a no-op for pure falls but fixes the REACTION-consumption resurrection (the side merge zeroes the cell mid-visit, the fall must move the live 0). New g47 witness `the_fall_arm_reads_the_live_post_reaction_level` discriminates it without evap (lava+water side merge under quickSettle). The union ALREADY carries `own_now` (`f57d7bacd:832`) — the re-fold's base now matches it, zero delta. The g45 evap-form witness stays g45/union-owned (it needs the evap block; rides the graft, §11 row 1's convention). |
| `6f5bd2a2d` | docs (research/104, the 69 ADDENDUM A1-A6, research/11, PORT_MAP, ANALYSIS, INDEX, anchors ×4) + the bucket witness rig fixes | **g45 owns its docs**; the 69 ADDENDUM's A1/A2 (the union dropped the g45b guards + fixtures) and A3 (the union's stale down-arm read — CONFIRMED still present at `f57d7bacd` this pass) fold into §8's re-fold obligations: the re-base onto the g47 tip now carries the down-arm fix natively, and step 5 still owns the guards. A6's graft order matches §12's. |

**Net effect on §8/§12**: the g46d re-fold list is UNCHANGED (the two sim
fixes now live on the g47 base the re-fold takes wholesale — they ride
`dd66fa3a`'s successor tip automatically). The witness arithmetic against
§11's 93 (which predates the four g45 doc-sweep commits — the union
`f57d7bacd` 07:06 pre-dates `c8908a55d` 07:18 / `99a4f8ccf` 07:39 /
`8d581953b` 07:51): +4 (the down-arm module — ONE copy, the g47 carry, the
g45 original dedupes onto it), +1 (g45's evap-form fall witness, rides the
graft), +1 (the g47 live-fall reaction pin), +0 (the g45 shimmer-flow twin
DROPS onto the closeout's ungated pin) = **99 pins** in `liquid.rs`
post-graft, plus the 2 wiring compose pins. All written-not-run.

## §14 — FINAL-FINAL retire receipts (W4-g47, 2026-11-21, the branch retires here)

The retire pass's four receipts, in order. Everything code-only: no
cargo/tests/rustfmt run; every witness named is WRITTEN, NOT RUN.

**Receipt 1 — the g46d re-base monitor (§8 re-verified).** Re-fetched
`origin/task/W4-g46-liquid-evaporation`: tip is STILL **`f57d7bacd`**
(2026-11-21 07:06:27 +0200, the graft-prep UNION — g46d has NOT landed).
Symbol re-grep on the union blobs, zero hits confirmed per surface:
`skip_count|update_world_tick|gen_settle_loop|tiles_ignore_water|shimmer_remove_water|reduced_max_liquids|take_pump_wakes`
in union `liquid.rs` = 0; union `world_sim/liquids.rs` still walks
`app.liquid_tick % 2` (`let liquid_tick = &mut app.liquid_tick; … if
*liquid_tick % 2 == 0`); union `wiring.rs` `pump_wakes` = 0; union
`app/mod.rs` still declares `liquid_tick: u32` and has no
`settle_loaded_world` call. **§8 STANDS as written** — the re-fold is
still g46d's obligation, now against this retire tip (below). One NEW
finding folded into §8's shape note via §13: the union's `liquid_check`
down arm carries the same pre-burn stale read the g45 doc-sweep fixed —
the re-base onto the fixed g47 body resolves it without an arm merge.

**Receipt 2 — the render-gaps closure (commit `a9dba7cf1`).** The four
cited draw-side follow-ups each censused to a verdict (ANALYSIS "The
liquid render-gaps closure"): `HasFullWater` ported unwired (its sole
caller `Main.DrawBlack` `:58008` is dead under the default
`DrawOwnBlacks=true`, `TileDrawingBase.cs:54` — the port's merged
wall+black path IS that configuration); `GetVisibleLiquid` dead (zero
callers, re-grepped both decompile trees); `SetWaveMaskData`'s data half
ported (`wave_mask` + `VISCOSITY_MASK`, the strength table proven
never-assigned — the GPU viscosity filter chain stays shader-infra
scope, cited); the wind clock ported AND WIRED (`LiquidAnimClock` →
`App::liquid_clock` → `draw_waters` + the G6 anim slot; the
`anim_time*8`/`*0.5` derivations retired — the deliberate render-cadence
fix, determinism held: pure (dt, wind) fn, `TR_WIND_FORCE` pins it).
3 witnesses + 3 anchors rows. The sim gaps-list stays EMPTY (§10).

**Receipt 3 — the g45d co-lane FINAL reconcile (commit `2bbadd9bd`).**
§13's owner table is the FINAL per-fn verdict against g45 tip
`6f5bd2a2d`: the two sim fixes ADOPTED onto this base (the fall arm's
`own_now` + the down arm's post-burn re-read — both were base-body
fidelity gaps, the union's stale down-arm copy superseded), the bucket
module g45-owned whole, the shimmer-flow twin dropped, the docs
g45-owned. 5 witnesses + 2 anchors rows. The §8 commit list is
UNCHANGED; the re-fold base tip moves to this branch's retire tip.

**Receipt 4 — the landing state.** The g47 branch retires at
**`bf5fc4ffb`** (the §14 receipts commit `0823ec3dd` + the wiring
completion: the `liquid_clock` field on the flow.rs front-end-boot App
literal, the second full-App construction site — the compile-by-
inspection census found no others). The merger's contract,
FINAL: (a) g46d re-bases onto THIS tip per §8+§12 step 1 (re-fetch
first — if g47's tip moved again, the successor wins); (b) graft steps
5-7 per §12 (the g45b guards + the f25 DELETE-dedupe + the producer
wiring); (c) re-pins per §11 as amended by §13 (99 liquid.rs pins + 2
compose pins expected); (d) the gate command + the anchors canary per
§12 steps 4-5 (the retire rows included); (e) the runtime-inert
contract per §12 step 6 — plus the render-side addition: the
`liquid_clock` is a pure (dt, wind) accumulator, draws no RNG, and
liquid-free worlds keep the pre-wiring G6 cache behaviour bit-for-bit.
No sim witness on this lane draws RNG beyond the pinned streams; the
render-gaps closure adds none (the clock and the mask are RNG-free).

## §15 — LIQUID SEAL (W4-g47, 2026-11-21: the g46e monitor, the delta-fold list, the down-arm receipt, the family bible)

**Seal 1 — the g46e monitor (re-fold LANDED; §14 receipt 1 SUPERSEDED).**
Fresh fetch: `origin/task/W4-g46-liquid-evaporation` tip is now
**`9bc722e5f`** (2026-11-21 10:00:03 +0200). The re-fold landed as five
commits: `e7626b72d` (step 1, base swap to g47 `95e9c1ef` wholesale),
`f588d53c5` (step 2, g47c `d3a30ebf`), `16fd6ad78` (step 3, g45b
`8e81814e` guards + the ~40 fixture widenings), `d95911ce1` (step 4, the
four R-12-5 drivers), `9bc722e5f` (docs: 69 SUPERSEDED-BY-252 (was 70) + the
UNION-CONTRACT). **§8's driver-surface obligation is DISCHARGED** —
symbol re-grep on the `9bc722e5` blobs: union `liquid.rs` carries
`skip_count` (:114/:323/:440) + `update_world_tick` (:662), union
`world_sim/liquids.rs` consumes it (:27, the `liquid_tick % 2` walk is
GONE), union `wiring.rs` carries `pump_wakes`/`take_pump_wakes`
(:261/:807), union `app/mod.rs` calls `settle_loaded_world` (:4116),
`gen_settle_loop`/`reduced_max_liquids`/`shimmer_remove_water`/the
guard module all present. BUT the re-fold's base snapshot is
`dd66fa3a` (07:26:06) — it **PRE-DATES all seven g47 post-`dd66fa3a`
commits** (`e420e351e` 08:36 → `776e64671` 10:20), and its carried
copy of this note (then key 70) is the 153-line `dd66fa3a`
snapshot: **§8–§14 are absent
from the union's doc**. The remaining obligation is the delta fold
below, NOT a re-base.

**Seal 2 — the delta-fold list for g46e (onto this seal tip; re-fetch
first, the successor wins per §12 step 1).** Seven commits, code/doc
split, hunks verified against the `9bc722e5` blobs this pass:

| g47 commit | Kind | What g46e must fold |
| --- | --- | --- |
| `e420e351e` | docs | §8 (historical record — its re-fold list is now discharged; carry for the audit trail) |
| `4c81731bc` | code+docs | `liquid.rs` +94: the 3 closeout witnesses. `honey_falls_once_every_eleven_visits` DEDUPES onto the union's `liquid_merge_cadence` form (:4470, already sentinel-riding — keep ONE, §11's rule); `shimmer_falls_every_visit_ungated` FOLD (+1, union zero hits); `quick_fall_bypasses_both_delay_gates` FOLD (replaces the union's lava-only `quick_fall_bypasses_the_delay_gate` :4489 — sharper both-kinds form, net 0). ANALYSIS full-fn census + the per-liquid verdicts |
| `5c09a68b8` | docs+anchors | §9–§12 + the `honey_falls` anchors row (union anchors LACKS it) |
| `a9dba7cf1` | **CODE** | the render-gaps closure, ALL of it: `liquid_draw.rs` +244 whole-file (`LiquidAnimClock`/`wave_mask`/`has_full_water`/`VISCOSITY_MASK` — union grep = 0 hits); `app/mod.rs` +7 (the `liquid_clock` field :856 + the `App::new` literal :3999 + the per-update tick); `render/liquids.rs` clock wiring (union STILL derives `anim_time * 8.0` :99/:206 + `* 0.5` :100 — the retired cadence); `render/tiles.rs` +6; `frame/draw.rs` +18/−3; anchors ×3; ANALYSIS + PORT_MAP rows |
| `2bbadd9bd` | **CODE** | the two sim fixes: the down-arm post-burn re-read hunk (`@@ -1283,6 +1292,18 @@` — the `let down = world.tile(x, y + 1)…` re-read after `kill_tile`, `flag` stays the pre-kill read) — **the union is STALE, see Seal 3**; the fall arm's `own_now` is a ZERO code delta (union :1112 already reads live post-reaction; only the comment differs — take the base cite `:570-575` + the reaction case). `liquid_downarm_live` ×4 + `the_fall_arm_reads_the_live_post_reaction_level` ×1 FOLD (+5, ONE module copy); §13; anchors ×2 |
| `bf5fc4ffb` | **CODE** | `flow.rs` +1: the `liquid_clock` field on the front-end-boot App literal (:2414; the union's literal :2410 lacks it). Compile-critical rider of `a9dba7cf1`'s `app/mod.rs` field — fold them together |
| `0823ec3dd` + `776e64671` + this seal | docs | §14 + §15 (replace the union's 153-line pre-rename snapshot with this file wholesale) |

**Seal 3 — the down-arm-live receipt (2bbadd9bd IS the fix-source).**
Re-verified on `9bc722e5`'s `liquid_check`: the down arm copies
`down` at :1414 BEFORE the `tileCut` burn (`kill_tile` :1422), then
the merge gate (:1426) AND the merge-type reads (:1438-1441) all read
the STALE pre-kill copy — the live post-burn re-read is **ABSENT**.
The seven cut kinds outside `tileObsidianKill` (444/518/519/549/637/
638/711) wrongly refuse the drain/merge on the union. g47 `2bbadd9bd`
(the g45 `c8908a55` carry) is the fix-source: the re-read hunk + the
4-pin `liquid_downarm_live` module fold verbatim; the union's copy is
superseded, NOT arm-merged (§13's verdict stands, now against the
re-folded tip). The fall arm needs NO code fold — union :1112's
`own_now` matches the base behaviourally (its comment cites only the
evap case; the base cite adds the reaction-consumption case).

**Seal 4 — the pin matrix (99+2 projected → 109+2 sealed).** Measured
this pass: old union `f57d7bacd` = 72 pins; re-folded union `9bc722e5`
= **102** (72 + 30 new, ZERO dropped). Delta fold: +1 shimmer-ungated,
+1 fall-reaction, +4 downarm-live, quickFall swap net 0, honey dedupe
net 0 → **108**; +1 g45 evap-form fall witness
(`the_fall_arm_moves_the_post_evaporation_level`, rides the graft —
union zero hits) → **109 `liquid.rs` pins + 2 compose pins**
(`wiring.rs` wake-order asserts inside
`the_pump_moves_water_and_adopts_the_type` :3945-3949 +
`world_sim/wiring.rs` `r_12_5_the_pump_transfer_wakes_the_settle_and_the_water_flows`
:670 — BOTH verified present on the union). Reconciliation vs §13's
99: **+10 = the g45b AddWater-guard ×10 module**, which landed at
re-fold step 3 (`16fd6ad78`) AHEAD of §12's graft step 5 — the 93/99
projection counted those at the graft; the union carries them NOW.
99 + 10 = 109. The anchors delta: the union LACKS all 6 g47
closeout/retire rows (`honey_falls…`, `LiquidAnimClock`, `wave_mask`,
`has_full_water`, `the_fall_arm_reads…`, `liquid_downarm_live`) —
fold them with their commits; the §12 step-5 canary then runs whole.

**Seal 5 — the LIQUID-FAMILY bible.** The liquid family lands as ONE
union: **g46-union-canonical** (`9bc722e5f` + the Seal-2 delta fold
onto this seal tip — the union `liquid.rs` is the merger's file, g45/
g47 copies DROP) carrying **g47-machinery-source** (`776e64671` +
seal: the skip/settle drivers, the force-settle family, the DelWater
tail, the render-gaps closure + wind clock, the two sim fixes — all
folded per Seal 2, none re-derived), **g45-buckets/deltas**
(`fa96cf32a` buckets module g45-owned whole with its `world_sim/
editing.rs` wiring; the guards + cadence + nudge + extra-wakes already
union-side; the evap-form fall witness + the shimmer-flow twin drop
per §13), and **f25-gen** (`84b90360`: step 6 DELETE-dedupes its
private copies onto the base exports, step 7 arms the producer
contexts; the gen-hash RE-DERIVE + the `edit_kills` drain-choice
record stand, §11 row 1). Gate: `cargo test --workspace -- --skip
migrates_version_2_kinds_to_item_ids` (judge ONLY by `test result: ok`
lines), then the §12 step-5 anchors canary + the step-6 runtime-inert
contract (+ the render-side addition: `liquid_clock` pure dt+wind,
RNG-free, liquid-free worlds bit-identical). Expected: **109 liquid.rs
pins + 2 compose pins** (§13's 99+2 as projected, +10 guards arrived
early — Seal 4), all written-not-run until the gate. The g47 branch
seals here; the successor-wins rule (§12 step 1) governs any later tip.

> **SEAL execution receipt (W4-g46 seal phase, this docs-tail).** Seal 2's
> code rows landed as `a4d49b355` (the g45 research/69 A9.2 rows 1-8 fold
> onto the union — down-arm hunk verbatim + `liquid_downarm_live` ×4
> sentinel-switched at :5491, the fall-arm comment-only upgrade +
> `the_fall_arm_reads_the_live_post_reaction_level` at :3169, the shimmer
> keep-ONE MERGED pin per A9.2 row 4 (the g47 section-15 ungated-only fold
> superseded — BOTH assert families ride
> `shimmer_falls_every_visit_ungated`), the quickFall swap + honey dedupe,
> the render-gaps ALL-CODE + the flow.rs `liquid_clock` compile-rider
> (:2414), the buckets module + x15). Seal 4's matrix RE-MEASURED on this
> tip: **109** `#[test]` in `liquid.rs` + the 2 compose pins
> (`wiring.rs:3922` / `world_sim/wiring.rs:670`) — the sealed expectation
> holds. Seal 4's 6 anchors rows now ALL present (the closeout
> `honey_falls…` row + the 5 retire rows, markers verified in the cited
> files). The steps-6/7 hunk record + the family bible live in
> research/374 (was 71 — the registry §10.2 rename `4bcbdd645`); the
> `edit_kills` gen-tail drain choice DISCARD is recorded there (§7.3). The
> f25 gen-hash RE-DERIVE obligation stands for the worldgen master merge.
> All witnesses written-not-run until the gate.
