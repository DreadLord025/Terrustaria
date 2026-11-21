# 312 — W4-d106 WORLDGEN-6 RETIRE FINAL: overlap census + keep-one drop-hunks + the e20 fidelity receipts + the pin schedule + the family declaration

> **STATUS: RETIRE FINAL — 2026-11-21, branch
> `task/W4-d106-worldgen-passes-6` (retire phase, base COMPLETE `4612ffc57`,
> R1–R6 all landed, retire commit `87ff33322`).**
>
> **RENUMBERED 281 → 312 (registry-150 §9 UNIFIED FINAL, d54 tip
> `b243c7086`, §9.2 row 312 — chain 240 → 281 → 312).** The round-2 281 was
> a self-pick into f26's §7.3-assigned block (281 = f26's
> `68-findframe-census-addendum-7h.md`); §9.2 assigns d106 **312** (the
> §8.7 note confirms: d106 KEEPS 312; c9's 312 self-pick → 349). Executed
> per dispatch: `git mv` + this header + all branch refs (INDEX / ANALYSIS
> / PORT_MAP / 12f + the cleanup.rs / spider_gem_moss.rs comments) repoint
> to **research-312**; the content is unchanged. The round-2 history below
> is kept as the chain record.
>
> **RENUMBERED 240 → 281 (registry-150 round-2 self-adjudication, the SEAL
> verify pass).** The original pick took **240** as "the first free number
> at this scan" — but registry-150 round 2 (d54 tip `72226e500`, §4 winner
> table + §6 instruction rows) had already awarded **240–243 to g58**
> (executed self-rename 143→240 at 12:16:37, pushed `c9af393dd` 14:08:18 —
> BEFORE this doc's 14:50 write), with d53's fresh 240 → **275** and c42's
> fresh 240 → **280** already instructed. The round-2 rule: numbers
> **237–243 are taken, 244–280 are assignments, 281+ are FREE**; new lanes
> must take the next number ≥281 after a fresh registry read AND an
> all-tip sweep (the post-sweep cutoff note makes this lane self-adjudicate).
> Executed here: a fresh `git fetch --prune` + a **360-tip `git ls-tree`
> sweep confirms 281–289 unclaimed at every tip** (the only ≥244 docs at
> tips are g48's 244/245 winners and the executed renames 271/273); this
> doc takes **281**, the first free number. All `research-240` refs on this
> branch (cleanup.rs / spider_gem_moss.rs comments + witnesses, PORT_MAP
> R4/R5/R6 rows, ANALYSIS retire notes, the 12f retire row) repoint to
> **research-281** in the same commit; the content is unchanged.
> Cross-lane number note: e20's research-**148** is instructed to rename
> → **269** (registry round 2), g41's research-**96** → **255**, research-
> **95** → **225** — this doc's citations name the docs by their CURRENT
> pushed filenames and add the assigned numbers here once; after those
> lanes execute their renames the citations resolve by slug. Code-only:
> no cargo/tests/rustfmt/graphify; witnesses WRITTEN NOT RUN; C# read in
> ≤100-line windows via direct junction paths. Tips read at this retire:
> d102 `29e8859bd`, d107 `266729126`, g36 `a91bf92b7` (matrix-88
> FINAL-FINAL), g37 `81fd1e0d0`, g41 `3c6da7a3f` (research-96 §3/§4),
> e20 `46fdcfd8e` (research-148/149), g46 union `9bc722e5f` (via the
> research-96 §1 receipt).

## 1. d102/d107 overlap census — which passes overlap, keep-one drop-hunks

The 12f range claim said "disjoint from every sibling's *stated* range
except the Grass Wall orphan". The LANDED tips diverge from the claims in
five places; the census below is per-file and per-pass, decided against
matrix-88 (the FINAL-FINAL keep-one authority) where it speaks.

### 1.1 d102 (`task/W4-d102-worldgen-passes-2`, tip `29e8859bd`, 9/9 landed)

d102's landed range is Silt :13143 … Wood Tree Walls :15717 — it did NOT
land Settle Liquids :16132 (its 12a doc's own range table hands :16132 to
d103/f25; matrix §2.19 confirms f25 owns it). **No pass-body overlap with
d106's six.** The overlaps are machinery/slot-shaped:

| # | Overlap | d102 side | d106 side | KEEP / DROP |
| --- | --- | --- | --- | --- |
| O1 | `GenVars.shimmer_position` field | `pipeline.rs` field + `gv.shimmer_position = shimmer::pass_shimmer(...)` producer | `pipeline.rs:388` field + None-init; consumers `liquids::shimmer_remove_water`, `cleanup::shimmer_clean_up`, `finish_remix_world` | **KEEP ONE field — g36 A1's publication** (matrix §3.4: g36 is the producer, the ONE field every consumer rewires onto). DROP d102's field AND d106's field at the graft; both consumers read g36's. d102's Shimmer PASS drops per matrix §2.1 (g36's A1 wins) — its producer write drops with it |
| O2 | `place_tight` signature | `spider_gem_moss.rs` `fn → pub(crate) fn` visibility-only edit, still `rng: &mut Rng` | `spider_gem_moss.rs:411` `pub(crate)` generic over `chest_stock::WorldGenRand` | **KEEP d106's generalisation grafted over g41's body** (matrix §3.2 point 2 / research-96 §3). d102's visibility hunk is subsumed — DROP the d102 hunk, the g41+d106 graft already exports `pub(crate)` |
| O3 | `growth.rs` re-exports | `pub(crate) use trees::ALLOWS_PLANTS_TO_GROW;` (:68) | `pub(crate) use trees::kill_invalid_tree_above;` (:69-72) | **KEEP BOTH** — different symbols, adjacent lines; a textual merge conflict only, resolve by keeping both re-exports |
| O4 | `worldgen/mod.rs` module decls | `mod passes2; mod shimmer;` | `mod cleanup;` (+ R2's `liquids` already at base) | **KEEP BOTH** — disjoint module lists; d102's `mod shimmer;` drops with its Shimmer pass (O1/matrix §2.1) iff g36's module carries the same name — the merger keeps ONE `shimmer` module (g36's `worldgen/shimmer.rs`); the two files are same-path forks, g36's wins wholesale |
| O5 | `progress.rs` message rows | silt/webs/mountain caves/gems/clean up dirt/wood tree walls rows | settle liquids again/tile cleanup/stalac/remove broken traps/final cleanup rows | **KEEP BOTH** — disjoint match arms, textual adjacency only. d102's Shimmer row (if any) drops with O1 |
| O6 | `world_hash.rs` stale notes | d102's per-pass deferred notes | d106 R1–R6 notes (§4 below) | **KEEP BOTH note sets** — they are comments; the pins re-pin ONCE at the gate (§4), never per-lane |
| O7 | `spider_gem_moss.rs` spider/webs `place_tight` callers | d102 did not touch the call sites | d106 R4 generalised the calls (`rng.next(n)` semantics) | No conflict — d106's callers ride the g41 graft (research-96 §3 point 3 keeps `pass_stalac` wholesale; the spider callers keep their pre-validator behaviour through g41's internal-validator body, byte-identical per d106's proof) |

### 1.2 d107 (`task/W4-d107-worldgen-special-seeds`, tip `266729126`)

d107's landed diff touches `caves.rs` (SurfaceCaves seed gates),
`worldgen/mod.rs` (tests only: the 6-seed determinism witness),
`pipeline.rs` (the dualDungeons pruning name fix `"Ice"` →
`"Generate Ice Biome"`), `world_hash.rs` (notes). **Zero pass overlap with
d106.** Two seam notes:

- **O8 — the pruning-name fix vs d106's gates.** d107's fix keeps
  `fragile ice` enabled under dualDungeons; d106's R3 `deny_some_generation`
  local gate is a SUBSET copy that matrix §3.5 already DROPs onto f24 A1's
  canonical tables — after that drop the d107 fix and the d106 passes read
  the same f24 truth, no interaction. KEEP d107's fix; DROP d106's local
  `cleanup.rs::deny_some_generation` at the f24 graft (matrix §3.5, named
  there).
- **O9 — the `:22470` anniversary paint tail.** d107's seed scope may land
  `FinishTenthAnniversaryWorldPaintTreesAndMushrooms`; matrix §6.4 places
  it immediately BEFORE the compose-hunk at `step_seed_finish`'s tail.
  d106's `final_cleanup` documents the tail as riding `step_seed_finish`
  (the M5-SEED-1 seam) — consistent; nothing to drop on d106.

### 1.3 The matrix-owned overlaps (context for the merger — not d102/d107)

For completeness, the contested d106 pieces OUTSIDE the d102/d107 axis,
already decided by matrix-88 FINAL-FINAL (this lane executes its own half
in §2, the rest is the gate's):

- **Final Cleanup pass body** — g37 `4ced188f9` landed the same C# range
  (`:22141-22422` body) at 05:47, d106 R6 `4612ffc57` at 12:43. Matrix §4
  row 109 names **g37 the owner** (its claim probe read d106's then-tip
  `e2c2609c` with R6 unchecked). d106 R6 landed later and is the SUPERSET
  (head gate + sweep + scatter + Finish* dispatch + shimmer cleanup +
  merge sweep). **Merger instruction: keep ONE `final cleanup` step.**
  The recommended resolution is d106's `cleanup::final_cleanup` body with
  g37's witnesses re-pointed (they assert the same C# arms; g37's
  `final_cleanup.rs` DROPs wholesale), OR g37's body + d106's tail arms —
  either way ONE step, ONE `pass_rng("Final Cleanup")` thread (§2.2), and
  the anchors canary keeps ONE row (both rows' markers must survive on the
  kept body; the dropped row is REMOVED BY THE MERGER as a duplicate, not
  to silence a failure — the surviving row carries both citations).
  This doc does not re-adjudicate the matrix; it publishes the d106-side
  facts the decision needs.
- **Settle machinery** — f25 core wins (matrix §3.3), d106 R2's PASS
  stays, re-based onto the g47→g46-union surface; the duplicate
  `liquid.rs`/`tileobject.rs` halves DROP per §3.3.
- **Stalactite chain** — g41 body + d106 generalisation (research-96 §3);
  d106's local moss lists DELETED this retire (§2.3), the explicit
  validator calls DROP at the graft (§2.1).
- **ShimmerCleanUp arm** — g36's `shimmer.rs::shimmer_clean_up` wins
  (matrix §7); d106's local copy DROPs (§2.2).

## 2. g36-matrix compose + the g41 stalactite keep-one — the d106-side hunks

### 2.1 The double-validation trap — DROP-explicit-check_stalactite (published, applied at the graft)

research-96 §3 FINAL INSTRUCTION point 2: g41's `place_tight` runs the
validator INTERNALLY; d106's Stalac sites were written against d106's
unvalidated base and call `check_stalactite` explicitly. Keeping both
doubles the `Next(3)` mismatch draws. **The four DROP hunks** (d106
`spider_gem_moss.rs`, current line numbers; each block is the `place_tight`
call's explicit-check tail — delete the `if` block, keep the `place_tight`
call):

```rust
// :843-845 (the underground stalactite arm, after place_tight :842) — DROP:
if world.tile(i, j).is_some_and(|t| t.active() && t.kind == TILE_STALACTITE) {
    check_stalactite(world, rng, i, j);
}
// :861-863 (the snow/ice ceiling arm, after place_tight :860) — DROP:
if world.tile(i, k).is_some_and(|t| t.active() && t.kind == TILE_STALACTITE) {
    check_stalactite(world, rng, i, k);
}
// :876-878 (the evil ceiling arm, after place_tight :875) — DROP: same shape at (i, k).
// :891-893 (the evil floor arm, after place_tight :890) — DROP: same shape at (i, k).
```

After the graft the validator fires inside `place_tight` exactly where
the C# `PlaceTight` tail (`WorldGen.cs:38320-38323`) runs it — one
validation, one `Next(3)` schedule. The `check_stalactite` fn itself
STAYS `pub(crate)` (g41's body keeps its name; the standalone d106
witnesses `r4_check_stalactite_kills_the_unsupported` /
`r4_place_tight_hangs_a_stalactite_under_stone` re-point onto the g41
body at the graft — the latter's explicit check call drops too, the
former calls the validator directly and stays). The caller-runs doc
comment on `place_tight` carries the keep-one note (added this retire).

### 2.2 The ShimmerCleanUp keep-one + the §6.4 compose — d106's half

Matrix §6.4/§7: g36's `shimmer.rs::shimmer_clean_up` is the ONE arm; it
composes at the TAIL of g41's `step_seed_finish` (after
`micro_surface::run`), with the merge sweep behind it on the g47/g46-union
`LiquidSystem` (`set_gen_settle_context(gv.water_line, gv.lava_line,
remix)` + `set_gen_shimmer_position` + the pub `gen_merge_sweep` wrapper
the 70-graft adds). **d106-side DROP hunks at the graft:**

- `cleanup.rs::shimmer_clean_up` (:2164 at the retire tip) — DELETE the
  fn; its call in `final_cleanup` (:2383, on the threaded `fc_rng` seeded
  at :2349) deletes with it.
- `cleanup.rs::shimmerator` (:2103 at the retire tip) — DELETE (g36's arm
  carries its own Shimmerator/Shimminate port, W:7).
- `cleanup.rs`'s final LiquidCheck sweep block (the `LiquidSystem::new` +
  `set_generating`/`set_remix_world_gen` + y-outer/x-inner loop) — DELETE
  from `final_cleanup`; the §6.4 hunk re-lands it at `step_seed_finish`'s
  tail on the union surface. d106's sweep witness
  (`r6_the_merge_sweep_matches_the_y_outer_replay`, this retire) re-points
  onto the composed hunk.
- The `liquids::shimmer_remove_water` shared helper STAYS (matrix §3.3
  names it as d106 R2's addition f25 lacks — the None-guard upgrade onto
  g36 A1's published position).

**The stream trap — FIXED on this branch this retire.** Matrix §6.4's
stream note: `pass_rng(name)` returns a FRESH stream per call; the C#
genRand is ONE per-pass instance, so the ShimmerCleanUp jitter must
CONTINUE the poo scatter's sequence. d106 R6 as landed called
`streams.pass_rng("ShimmerCleanUp")` — a fresh, wrongly-named stream (the
replay trap in its worst form). The retire fix threads ONE
`pass_rng("Final Cleanup")` instance through `poo_scatter` →
`shimmer_clean_up` (`cleanup.rs::final_cleanup`), the minimal-edit form
(a) the matrix prescribes. When the arm moves to `step_seed_finish`, the
merger stashes the threaded Rng (`gv.final_cleanup_rng = Some(rng)` per
§6.4) — the witness
`r6_the_shimmer_cleanup_continues_the_scatter_stream` (this retire,
WRITTEN NOT RUN) pins the continuation and FAILS under any fresh-stream
form. The flag-gated Finish* hosts between the two calls draw their own
name-seeded streams (the recorded M5-SEED-1 deviation); on a classic world
nothing draws between scatter and cleanup.

### 2.3 ONE moss set — EXECUTED this retire

research-96 §3 point 5: the moss support test reads the GENERATED
`tiledata::TILE_MOSS` (`sets.rs:44`, `Main.cs:7031` — 11 members
179/381/534/536/539/625/627/180/181/182/183). d106's local
`MOSS_TILES`+`NEON_MOSS`+381 union in `place_unchecked_stalactite` covered
only 10 — **627 (KryptonMoss Brown) was missing**, a real fidelity gap
(the stalactite refused to seat under a 627 ceiling). Both `is_moss`
reads now call `td::TILE_MOSS.has(support)`; the local `MOSS_TILES` /
`NEON_MOSS` consts STAY for their other consumers (`set_moss`'s index
table, the neon-blob scatter — those are `randMoss` draw tables, NOT the
`tileMoss` membership set; the generated-set dedupe applies to membership
reads only). Witness `r4_place_tight_hangs_under_the_generated_moss_627`
(this retire, WRITTEN NOT RUN) pins the 627 seat. The g41 graft keeps its
own generated-set reads (already correct on that branch).

## 3. e20 research-148 — the two fidelity items VERIFIED (fix + receipt)

### 3a. `kill_tile` no_item:true vs C# default false — RECEIPT: the `true` IS vanilla at gen

C# re-read (direct windows): `WorldGen.cs:63517` —
`KillTile(int i, int j, bool fail = false, bool effectOnly = false, bool
noItem = false)`; the pass calls the 2-arg form (`:26946`/`:26953`), so
the ARGUMENT is false — but `:63546-63549`:

```csharp
if (WorldGen.isGeneratingOrLoadingWorld)
{
    noItem = true;
}
```

and `isGeneratingOrLoadingWorld` is set true for the whole generation
(`:6612`, the `GenerateWorld` entry; declared `:4196`). **At gen time the
C# KillTile ALWAYS suppresses items** — d106's `kill_tile(world, x, y,
true)` is behaviourally identical, not a deviation. The port's `kill_tile`
(`edit/kill.rs:193`) takes `no_item` explicitly (no global gen flag in
tr-core), so passing `true` at every gen call site is the port's faithful
encoding of the `:63546` override. **Verdict: NO FIX NEEDED; receipt
recorded** (the citation now sits at the R5 strip site). The same receipt
covers every other gen-time `kill_tile(..., true)` in `cleanup.rs`
(R3/R6) and `spider_gem_moss.rs` (R4). No RNG either way — the drop path
is event-queued, not drawn, under `no_item`.

### 3b. The `Utils.Swap` leftover-cell BFS edge — FIXED this retire

C# re-read `:26840-26914`: `Utils.Swap(ref t, ref t2)` at `:26856` does
NOT clear the swapped-in list. When the inner loop breaks on `num2 <= 0`
with `t` non-empty, the leftovers survive in `t2` and the next wave
processes them BEFORE its own appended neighbours; the outer `num`
exhaustion path breaks before another swap (leftovers never processed —
identical either way). d106 R5 had `t2.clear()` after the swap — the
leftovers were DROPPED (a >2000-inner-pop wired blob kept un-stripped
cells). **FIX: the `t2.clear()` is removed** (`cleanup.rs`, the 1:1 swap);
the two caps are exposed through a test-only seam
(`clear_broken_traps_capped`, the vanilla caller passes 2000/2000) so the
edge is witnessable on a runnable fixture. Witnesses (WRITTEN NOT RUN):
`r5_the_bfs_carries_capped_leftovers_like_the_cs_swap` (a 4-cell wired
plus at a 2-pop cap — the old code leaves 3 wires alive, the fix strips
all 4) and `r5_the_outer_cap_leaves_the_queue_unprocessed` (the outer-cap
path). Reachability note: only networks needing >2000 pops in ONE wave
(the vanilla cap) hit this — huge wired blobs; the fix is draw-free.

### 3c/3d/3e — the remaining research-148 items, resolved

- **3c IsPressurePlate seam:** KEEP d106's inline
  `matches!(t.frame_x, 20 | 21)` in `is_it_a_trigger` (tr-core cannot call
  tr-app's `pub(crate)` fn). e20's `minecart::is_pressure_plate`
  (`6d1754719`) stays the app-side mirror; its `dead_code` attribute is
  justified (its named consumer ports the predicate inline). ONE truth
  per crate, both pinned by witnesses (e20's `6d1754719` witness + d106's
  R5 witnesses). No code change.
- **3d ClearAllBrokenTraps:** zero callers in the decompile (research-148
  §1 grep) — dead C# API, NO PORT, recorded as discarded.
- **3e pointers:** research-71 §7c's residual line is superseded by d106
  R5 (e20's own research-149 already points here); d106's PORT_MAP row
  :103 names the pass. The pins move by design — §4.

## 4. world-hash pins — the sequential re-pin schedule at the merge gate

Both classic pins (`SEED42_512 = 124297949`, `DEFAULT_WORLD =
3647283075`) are stale-by-design on this branch; the per-pass notes in
`world_hash.rs` (R1–R6, both pins) stay as landed. **No lane's isolated
"pins unchanged" argument composes** (matrix §7) — the gate re-pins ONCE
per pin on the fully merged chain, sequentially per landed lane, with the
failing run recorded first and the a==b half green on it. The d106
schedule (the gate walks it in pass order after the §6.2 merge order):

| Step | Pass (slot) | SEED42_512 | DEFAULT_WORLD | Why |
| --- | --- | --- | --- | --- |
| 1 | R1 Grass Wall (living trees → trees) | expected UNCHANGED | expected UNCHANGED | wall-0 grass tops (`terrain.rs:343` walls start top+3); no wall-63 producer earlier; the Next(4) lattice draws on its own stream. If a grass-with-wall cell exists on some seed the pin moves BY DESIGN — measure, don't assume |
| 2 | R2 Settle Liquids Again (before cacti) | MOVES | MOVES | the settle engine re-types/drops/merges liquid cells (lake, ocean shelves, lava, honey); draws no RNG. Re-based onto the g47/g46-union surface first (matrix §3.3) — the pin measures the POST-graft body |
| 3 | R3 Tile Cleanup (cacti → lihzahrd altar) | MOVES | MOVES | slope clear + drip + object normalisation; own `pass_rng("Tile Cleanup")` stream |
| 4 | R4 Stalac (after water plants) | MOVES | MOVES | gem trees + speleothems on `pass_unified("Stalac")`; measure AFTER the g41 stalactite graft (§2.1) + the moss-627 fix (§2.3) — the 627 seat adds tiles the pre-retire body missed |
| 5 | R5 Remove Broken Traps (after stalac) | MOVES | MOVES | the broken-circuit strip (dungeon/temple wires); draws no RNG; the §3b leftover fix changes only >2000-pop blobs (neither pin world has one — but measure, don't assume) |
| 6 | R6 Final Cleanup (remove broken traps → world tail) | MOVES | MOVES | wall-hole fill + sand columns + boulder/liquid/painting sweeps + poo scatter + the LiquidCheck merge sweep; the 512 world's scatter ranges stay valid (512 > 250). Measure AFTER the §2.2 keep-one (g36 arm at seed-finish tail, ONE threaded Rng) — the classic chain's jitter draws are identical either way (None-guard), the shimmer-seed witnesses carry the stream pin |

Rules (the g41 research-93 §FINAL STATUS step-2 discipline, matrix §7):
re-pin each step ONLY after its a==b witness is green on the failing run;
record old → new in the `world_hash.rs` comment with the gate log name;
never re-pin two lanes in one measurement; the anchors canary vetoes any
lossy resolution (a missing marker = recover the arm, never delete the
row). The seed-world pins (d107's 6-seed witness, g41's flag-world
witnesses) ride their own lanes' gates — d106's flag-gated arms
(FinishNoTraps/FinishRemixWorld/NotTheBees) move NO classic pin.

## 5. FINAL DECLARATION — the worldgen-6 family

**ALL SIX PASSES LANDED** on `task/W4-d106-worldgen-passes-6` (R1
`ee0fcbefa`, R2 `08aa8289f`, R3 `7c5913737`, R4 `29faa0995`, R5
`e2c2609c5`, R6 `4612ffc57`; retire phase = this doc + the §2.3/§3b/§2.2
fixes). The lane's contribution to the merged chain is exactly:

- **FIVE passes KEEP as landed** (matrix §4 row 108): Grass Wall (R1),
  Settle Liquids Again (R2, the PASS — re-based onto f25-core/g47-union
  machinery per §3.3), Tile Cleanup (R3, the local denySome DROPs onto
  f24), Stalac (R4, `pass_stalac` + slot + `pass_unified` wholesale per
  research-96 §3 point 3), Remove Broken Traps (R5, + the §3b swap fix).
- **ONE pass CONTESTED** — Final Cleanup (R6): g37 owns the matrix row;
  §1.3 publishes the keep-one facts, §2.2 the d106-side drop hunks. The
  R6 superset body is the recommended survivor; either resolution keeps
  ONE step + ONE threaded stream + both anchor citations.
- **The machinery d106 produced that survives the dedupes:** the
  `WorldGenRand`-generic `place_tight`/`check_stalactite` signatures (the
  g41 graft donor), `Streams::pass_unified`, the `generating` toggle +
  `tile.rs` any_shimmer/left_slope/right_slope + the None-guarded
  `shimmer_remove_water` upgrade (f25 lacks them, §3.3), the
  `WireState::wire` red-bit accessor (R5), `tile_runner::save_slopes`
  shared (R3), `micro_surface::surface_is_desert` exposed +
  ws-parameterized (the Final Cleanup head gate), `ocean_depths` shared,
  the generated-TILE_MOSS membership reads (§2.3).
- **DROP-listed with named keepers:** the local stalactite validator
  calls at the Stalac sites → g41's internal validator (§2.1); the local
  `shimmer_clean_up`/`shimmerator` + the in-pass LiquidCheck sweep →
  g36's arm + the §6.4 compose at seed-finish (§2.2); the duplicate
  settle halves → f25/g47-union (§3.3); `deny_some_generation` local →
  f24 A1 (§3.5); `GenVars.shimmer_position` local field → g36 A1 (§3.4).
- **Witnesses:** R1–R6's written-not-run sets as landed + the retire's
  five new ones (`r5_the_bfs_carries_capped_leftovers_like_the_cs_swap`,
  `r5_the_outer_cap_leaves_the_queue_unprocessed`,
  `r6_the_shimmer_cleanup_continues_the_scatter_stream`,
  `r6_the_merge_sweep_matches_the_y_outer_replay`,
  `r4_place_tight_hangs_under_the_generated_moss_627`) — ALL WRITTEN NOT
  RUN (code-only); the gate runs them with the full workspace command.
- **Pins:** both classic pins stale-by-design, the §4 sequential schedule
  governs; nothing in the retire phase re-pins.

**Retire verdict: no unclaimed d106 work remains.** Every residual is
either executed on this branch (§2.3, §3b, the §2.2 stream fix) or
published as an exact graft-time hunk with a named keeper (§1, §2.1,
§2.2, §4). The gate executes; it does not re-adjudicate.

## 6. SEAL-VERIFY receipt (the dispatched d106b verify pass, independent re-read)

The dispatched verify pass re-derived the retire claims from source and
tip objects AFTER `87ff33322` landed+pushed; findings:

1. **§3b swap fix VERIFIED against C#** — `WorldGen.cs:26840-26914` re-read
   whole: `Utils.Swap(ref t, ref t2)` at `:26856`, no `t2.Clear()` anywhere
   in the loop; neighbours APPEND to `t2` (`:26891`/`:26896`/`:26901`/
   `:26906`); the inner `num2 <= 0` break (`:26861-26864`) leaves `t`
   non-empty; the outer `num <= 0` break (`:26852-26855`) precedes the
   swap. The port's `clear_broken_traps_capped` body mirrors this exactly
   (the removed `t2.clear()` was the only delta); the two cap witnesses
   drive the same code path the vanilla 2000/2000 caps take.
2. **§3a noItem receipt VERIFIED** — `:63517` signature re-read (defaults
   `fail=false, effectOnly=false, noItem=false`); `:63546-63549` re-read
   (`if (WorldGen.isGeneratingOrLoadingWorld) { noItem = true; }`). The
   gen-time `kill_tile(..., true)` is vanilla; no fix needed, receipt at
   the strip site stands.
3. **§2.3 moss set VERIFIED by bit-decode** — the generated
   `tiledata/sets.rs:44-48` `TILE_MOSS` bitmask decodes to exactly
   {179, 381, 534, 536, 539, 625, 627, 180, 181, 182, 183} (word2 bits
   51/52/53/54/55 = 179–183, word5 bit61 = 381, word8 bits 22/24/27 =
   534/536/539, word9 bits 49/51 = 625/627) — the 11 `Main.cs:7031`
   members incl. 627. The local `MOSS_TILES`/`NEON_MOSS` consts correctly
   STAY (they are `set_moss` index tables + the neon scatter draw tables,
   not membership sets; the remaining union read at the moss-biome Arm F
   filter is c29/g41-owned residual code, identical on g41's tip — NOT
   d106's to touch).
4. **§2.1 drop-hunk line numbers VERIFIED** at the retire tip: the four
   explicit-check blocks sit at `spider_gem_moss.rs:843-845`/`:861-863`/
   `:876-878`/`:891-893` exactly as published; the caller-runs keep-one
   note sits on `place_tight`'s doc.
5. **§2.2 stream fix VERIFIED against the C# + the witness replay** —
   `ShimmerCleanUp` `:34716-34763` re-read: the ellipse jitter at `:34753`
   draws TWO `genRand.NextDouble()` per cell over the FULL 241×241 rect
   (the jitter gate is OUTSIDE the radius-120 inner-disc gate), stretch
   1.4 above / 1.2 below, radius 90 — the witness's `gates` replay matches
   this shape cell-for-cell; `poo_scatter` draws exactly 2 range rolls per
   successful pick on the all-dirt fixture (the 3000 valve never trips).
   ONE `fc_rng` threads scatter → cleanup in `final_cleanup` (`:2349-2383`).
6. **R1–R6 landing VERIFIED** — pipeline slots `:1409` grass wall (living
   trees → trees), `:1418` settle liquids again (before cacti), `:1424`
   tile cleanup (cacti → lihzahrd altar), `:1435` stalac (after water
   plants), `:1440` remove broken traps, `:1446` final cleanup (before
   world tail); the anchors rows 899–904 all present; both pins untouched
   (`SEED42_512 = 124297949` `world_hash.rs:291`, `DEFAULT_WORLD =
   3647283075` `:360`) with the per-pass stale-by-design notes on both.
7. **Registry correction EXECUTED** — the 240→281 rename (the header
   block): registry-150 round 2 (`72226e500`) awards 240 to g58; the
   fresh 360-tip sweep found 281 free; every branch ref repointed.
8. **Cross-lane facts re-confirmed at the verify fetch** — g36 tip
   `a91bf92b7` (matrix-88 §6.4 compose-hunk + §3.2 six-point + §7 family
   declaration read WHOLE), g41 tip `3c6da7a3f` (research-96 §3/§4 read
   WHOLE — the 6-point FINAL matches §2.1/§2.3 here), e20 tip `46fdcfd8e`
   (research-148 read WHOLE — items 3a/3b/3c/3d/3e all closed by §3),
   g37 tip `81fd1e0d0` (the Final Cleanup contest, §1.3 — unchanged),
   d102 tip `29e8859bd` / d107 tip `266729126` (the O1–O9 census inputs
   unchanged). No sibling tip moved in a way that changes any §1–§5
   verdict.

**SEAL verdict: the retire is COMPLETE and CORRECT modulo the executed
240→281 renumber. The lane is ready for the merge gate; the gate executes
§1–§4, it does not re-adjudicate.**

*Sources: C# direct reads `WorldGen.cs`
:22121-22140/:22423-22492/:26840-26958/:34716-34815/:38303-38362/:63517-63576,
`Tile.cs:249-259` (via the R5 port citations); sibling tips via
`git show origin/task/W4-…` (no junction recursion, no other branch
touched). Registry: research-150 @ d54 `72226e500` (round 2). Matrix:
research-88 @ g36 `a91bf92b7`. Receipts: research-96 @ g41 `3c6da7a3f`,
research-148/149 @ e20 `46fdcfd8e`.*