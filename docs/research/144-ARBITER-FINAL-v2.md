# 144 ARBITER FINAL v2 — the splash keep-one, THE definitive verdict

> Rendered 2026-11-21 on `task/W4-f47-liquid-shimmer-2` (tip `18fe75f2e`,
> the v2 arbitration commit follows). Code-only: no cargo/tests/rustfmt/
> graphify. Both tips read directly: f47 `18fe75f2e` (this branch) and
> e19 `0fb0ac4ad` (`origin/task/W4-e19-liquid-quicksell-shimmer`, the
> SHIMMER SEAL tip, read via `git show`). C# re-read this round:
> `WorldItem.cs:862-906` (the `:865` local `flag2` probe, the `:866-869`
> sticky arm, the `:879-885` entry edge, the `:886` `!flag2` gate).

## Supersession

**BOTH prior verdicts are SUPERSEDED by this one:**

- `docs/research/144-ARBITER-FINAL.md` (f47 tip `18fe75f2e`) — its keeper
  pick STANDS but its decisive ground (#1 the verbatim-vs-sticky entry
  gate) is MOOT: the e19 SHIMMER SEAL landed the identical verbatim read.
  Its e19 drop-hunk line numbers were pinned at `c7a38bea5` — stale.
- e19's `docs/research/144-splash-keep-one.md` (its tip `0fb0ac4ad`, the
  SHIMMER-seal §arbitration) — its keep-e19 ruling is OVERRULED again,
  now on the verified witness-superset ground below. Its 8-row
  f47-drop table was rendered against the STALE f47 tip `1997be092`
  (rows 2/3/4 misclassify f47 witnesses it claims f47 lacks — the
  corrections below).

The files stay as the lane record; **141/master reads THIS doc** for the
splash keep-one. The e19 144 filename fork (`144-splash-keep-one.md` vs
f47's `144-item-splash-keep-one.md` vs this `144-ARBITER-FINAL-v2.md`)
remains noted for the d54 registry sweep.

## The NEW FACT, verified at both tips

The entry gate is **verbatim in BOTH lanes**:

| | C# | f47 `18fe75f2e` | e19 `0fb0ac4ad` |
| --- | --- | --- | --- |
| local probe | `:865` `bool flag2 = Collision.LavaCollision(...)` | `item.rs:1336` `let lava_now = self.lava_probe(world)` | `item.rs:1378-1379` `let lava_now = player::lava_collision(...)` |
| sticky arm | `:866-869` `if (flag2) base.lavaWet = true` | `:1337-1339` | `:1380-1382` |
| ENTRY gate | `:886` `if (!flag2)` — the LOCAL | `:1356` `lava: lava_now` | `:1415` `lava: lava_now` (the seal's fix, comment `:1408-1414` cites it) |
| EXIT gate | `:965` reads STICKY `base.lavaWet` | `:1372` `lava: self.lava_wet` | `:1429` `lava: self.lava_wet` |

The prior arbiter's ONE semantic objection is therefore MOOT on both
halves. Both halves are 16-box, body-identical (the four families, the
post-writes, the Next(6) palette, the style-4/5 cues, the exit snapshot,
the drain slot, the once-per-pass `TR_WATER_STYLE` pin). **The keep-one
reduces to SUPERSET COVERAGE of the witness sets + merge-hunk economy.**

## The witness census at BOTH tips (the decisive evidence)

`Select-String "fn w4_"` over both lanes' `item.rs` + `world_sim/items.rs`:

### What f47 carries that e19 LACKS (verified absent at `0fb0ac4ad`)

| f47 witness (tip `18fe75f2e`) | pin | e19 twin? |
| --- | --- | --- |
| `w4_f47ds_the_entry_splash_latches_with_the_sticky_flags` (`item.rs:2065`) | the latch LIFECYCLE: entry arms `splash_due`, wetCount 19, once-per-entry (the second tick fires none) | NONE — e19's `w4_e19_the_wet_flags_latch_and_clear` (`:2681`) pins the FLAGS (honey probe + dry clear), never reads the queued splash |
| `w4_f47ds_the_exit_splash_snapshots_the_pre_clear_flags` (`:2093`) | the exit snapshot READS the pre-clear flags while the drop's flags read false after | NONE at the unit level — e19's pass-level exit twin asserts only `after_exit > after_entry` (a count), not the snapshot semantics |
| `w4_f47ds_the_shimmer_entry_splash_draws_71_and_colours_verbatim` (`:2112`) | the RNG-STREAM twin: 71 draws exactly in C# order (`assert_eq!(rng, twin)`), scale SET 0.8, alpha 0, palette, + the folded 16-box anchor bands 298..321 / 304..323 | e19's `..._draws_71_at_the_16_box` (`items.rs:1734`) is the SAME assertion set through the free fn — a true twin, but see the quartet row: the e19 originals lose nothing f47 lacks |
| `w4_f47ds_the_lava_splash_wins_the_gate_and_draws_31` (`:2174`) | 31 draws, vy −2.0, scale 1.3, alpha 100, + the folded noGravity + 5-active-dust-35 count | e19 `..._draws_31_and_wins_the_gate` (`:1791`) — same pins, twin |
| `w4_f47k1_the_water_exit_anchor_drops_the_minus_8` (`:2227`) | same-seed entry/exit Y delta exactly 8 | e19 `..._the_minus_8` (`:1835`) — twin |
| `w4_f47k1_*` pass-level quartet (`items.rs:1452/:1494/:1519/:1538`) | the entry burst / exit-after-cooldown / cooldown-suppression / water-style family through the FULL `update_items` pass, **with the `max_dust_to_draw = 6000` budget fix** (the flow_app default 0 would trip the >90% pressure band's Next(4) refusal — the e19 quartet at `:1615/:1654/:1678/:1696` LACKS this line and would fail under its own fixture when run) | the e19 originals exist but are the BUDGET-BROKEN copies; the f47 folds are the strict improvement |

### What e19 carries that f47 LACKED (now resolved)

| e19 witness (tip `0fb0ac4ad`) | disposition |
| --- | --- |
| `w4_e19_the_entry_splash_gate_reads_the_local_lava_probe` (`item.rs:2734`) — the seal's verbatim-gate witness | **FOLDED this commit**: NEW `w4_f47k2_the_entry_splash_gate_reads_the_local_lava_probe` (`item.rs`, after `w4_f47k1_the_water_exit_anchor_drops_the_minus_8`), same fixture (40×80, row-12 lava pool, park y 184), driven through the keeper `splash_due` latch. WRITTEN NOT RUN. After this fold the keeper has NO uncovered e19 splash pin. |
| `w4_e19_the_wet_flags_latch_and_clear` (`:2681`) + `w4_e19_the_grabbed_arm_clears_every_wet_flag` (`:2711`) | KEEP on e19's side either way — flags-only witnesses, reference NO splash type, unaffected by the keeper pick. Not an e19-shape advantage. |
| the gravity/integration/repulsion family (`:2605` dry-integration full-velocity half, `:2648` honey/water arms, the repulsion quartet, `rise_over_shimmer`, etc.) | KEEP on e19's side either way — lane-owned hunks BOTH receipts already agree stay e19's (the f47 gravity ladder is a pure-fn witness, not an integration twin; the e19 full-velocity half survives under either keeper). Not a keeper discriminator. |
| the two probe twins (`w4_e19_the_wet_probe_is_the_centre_inset_box` `:2766`, `..._respects_the_liquid_surface_drop` `:2790`) | the e19 seal FOLDED f47's originals — byte-equivalent pins on both tips (`w4_f47ds_*` `:1990/:2013`). Tie; dedupe at merge. |
| `w4_e19_the_wet_probe_uses_the_liquid_surface_offset` (`:2425`) | f47's `w4_f47ds_the_box_is_wet_probe_uses_the_liquid_surface_offset` (`:2043`) is its fold — tie, dedupe. |

**Result: after this commit's one fold, the f47 witness set is the STRICT
SUPERSET of the e19 splash witness set.** The e19-unique remainder
(flags/gravity/repulsion family) is keeper-agnostic — it survives on the
e19 lane hunks under either shape. The e19 seal's claim "the e19
pass-level witness quartet ... have no f47 pass-level twin" is FALSE at
the arbitration tips: `w4_f47k1_*` at `items.rs:1452/:1494/:1519/:1538`
are exactly those four, plus the dust-budget fix e19's copies lack.

## FINAL RULING: THE KEEPER = f47

**KEEP `Splash` / `splash_due` / `ItemDrop::splash` (f47,
`crates/tr-app/src/item.rs`). DROP the e19 `ItemSplash` /
`pending_splash` / `take_splash` / free-fn `item_splash` at merge.**

Grounds, in decision order (the v2 rules: superset coverage first,
merge-hunk economy second):

1. **Superset coverage.** With both gates verbatim and both bodies
   identical, the keeper is the side whose witness set covers the
   other's. Census above: f47 covers every e19 splash pin (after the
   gate-witness fold this commit); e19 lacks the latch-lifecycle twins,
   the exit-snapshot unit twin, and carries the budget-broken quartet.
   Keeping e19 orphans strictly more coverage than keeping f47.
2. **Merge-hunk economy.** Keeping f47 retires e19's shape (struct
   `:884-903`, field `:944-949`, init `:1005`, `take_splash`
   `:1018-1022`, two latch sites, the drain block, the free fn, seven
   witnesses, two wiring literals, one anchors row). Keeping e19 would
   retire MORE: the f47 shape PLUS the five f47-unique witnesses with
   no e19 home (they would need re-derivation onto the free fn — new
   hunks, not drops), PLUS the quartet re-fold. The e19 keep is
   strictly more merge work for strictly less coverage.
3. **The prior grounds, re-scored.** Verbatim gate: TIE (both). Water
   style: TIE (both pin once-per-pass). 16-box: TIE. The g67
   rain-splash "contract" argument (e19's free fn matches the g67
   drain pattern): a STYLE preference, not a dependency — verified at
   `origin/task/W4-g67-weather-rain-splash`: its splash drain is
   `projectiles.rs:557` `drain_splashes()` (the bobber `DoASplash`),
   it references NEITHER `item_splash` NOR `ItemSplash` NOR
   `splash_due`; g67's items.rs/item.rs contain no splash symbols at
   all. Nothing compiles against the e19 shape. The keeper's method
   drain sits in the same slot (post-`drop.update`, pre-flicker,
   `UpdateItems` stream) — the seam is positional, not nominal.

## Exact loser drop-hunks (e19, at ITS tip `0fb0ac4ad` — re-pinned)

All line numbers verified this round at `0fb0ac4ad` (they drifted from
the v1 doc's `c7a38bea5` pins):

1. `crates/tr-app/src/item.rs`
   - `:884-903`: the `ItemSplash` doc block + derive + struct (doc
     `:884-889`, derive `:890`, struct `:891`, fields through `:902`,
     close `:903`).
   - `:944-949`: the `pending_splash: Option<ItemSplash>` field + doc.
   - `:1005`: the `pending_splash: None` initializer.
   - `:1018-1022`: `take_splash` + doc.
   - `:1405-1418` / `:1426-1432`: the two `self.pending_splash =
     Some(ItemSplash { … })` latch sites — replaced by the f47
     `splash_due = Some(Splash { … })` sites (entry `lava: lava_now`
     — the e19 seal's own verbatim comment block `:1408-1414` drops
     with the site; the f47 entry site carries the same read). The
     surrounding gate/cooldown/clear lines are identical on both
     tips and stay.
   - `w4_e19_the_entry_splash_gate_reads_the_local_lava_probe`
     (`:2726-2755`, doc from `:2726`): DROP — FOLDED as
     `w4_f47k2_the_entry_splash_gate_reads_the_local_lava_probe`
     (this commit).
   - `w4_e19_the_wet_probe_is_the_centre_inset_box` (`:2757` doc,
     fn `:2766`) + `..._respects_the_liquid_surface_drop` (fn
     `:2790`): DROP — the f47 `w4_f47ds_*` originals carry the same
     pins (the e19 seal's fold of them).
   - `w4_e19_the_wet_probe_uses_the_liquid_surface_offset` (fn
     `:2425`): DROP — f47's `w4_f47ds_the_box_is_wet_probe_uses_the_
     liquid_surface_offset` (`:2043`) is the same body.
   - `w4_e19_the_wet_landing_keeps_the_flush_residual` (fn `:2631`):
     DROP — the f47 `w4_f47k1_the_wet_landing_keeps_the_flush_
     residual` (`:1961`) covers the same pins (flush landing + the
     residual velocity), driven through the probe latch.
   - `w4_e19_the_dry_drop_integrates_at_full_velocity` (fn `:2605`):
     SPLIT — the landing/rest half (`:2612-2623`) drops (the f47
     `w4_f47k1_the_dry_flush_landing_carries_the_residual_then_
     rests` `:1943` covers it); the full-velocity half
     (`:2606-2611`) has NO f47 twin: KEEP that half.
   - KEEP unchanged: `w4_e19_the_wet_flags_latch_and_clear`
     (`:2681`), `w4_e19_the_grabbed_arm_clears_every_wet_flag`
     (`:2711`), the gravity/repulsion/rise/transform family — none
     reference the splash shape.
2. `crates/tr-app/src/world_sim/items.rs`
   - `:693-710` (comment `:693-699` + the `drop.take_splash()` drain
     block `:700-710`): replaced by the f47
     `if drop.splash_due.is_some() { drop.splash(…) }` block — same
     slot (post-`drop.update` `:692`, pre-flicker head `:711`).
   - the free fn `item_splash` (`:777`) + its doc block, through the
     closing brace `:851`.
   - `:126-130`: the pass-head `splash_water_style` pin STAYS (the
     f47 drain consumes the identical head; dedupe to one copy).
   - the seven splash witnesses DROP: the quartet `:1615/:1654/
     :1678/:1696` (the f47 `w4_f47k1_*` folds carry the same pins +
     the dust-budget fix) and the census trio `:1734/:1791/:1835`
     (folded into the f47 twins + `w4_f47k1_the_water_exit_anchor_
     drops_the_minus_8`).
3. `crates/tr-app/src/world_sim/wiring.rs`: the e19 `ItemDrop`
   literals (`pending_splash: None` at `:202`/`:795`) lose
   `pending_splash`, gain `splash_due` (the f47 literals are the
   canonical forms — both lanes carry all five phase-3 fields).
4. `anchors.txt`: DROP the e19 row
   `crates/tr-app/src/world_sim/items.rs|fn item_splash|W4-e19 round 2
   …` in the SAME merge hunk that deletes the fn; KEEP the f47 row
   `crates/tr-app/src/item.rs|pub fn splash|W4-f47 doc-sweep 36 …`.
5. e19's research receipts (`144-splash-keep-one.md`, `148`'s splash
   contract) are SUPERSEDED by this doc — the files stay as the lane
   record, the merge reads THIS verdict.

**e19-owned hunks that survive the merge** (unchanged, re-confirmed at
`0fb0ac4ad`): the `shimmer_tail`/`apply_decraft` `wet = true` writes
(`WorldItem.cs:1844`/`:635`), the `rise_over_shimmer` two-row centre
scan (`:493-504`), the chain-apply half, the collision-engine
reconciliation onto f47's `tile_collision_opts` wrappers, the
flags/gravity/repulsion witness family, and the dry-integration
full-velocity half (the split above).

**f47-keep-hunks NOT dropped** (the e19 seal's own KEEP list, agreed):
the 676 ShimmerSlime light/dust arm, the NPC-side shimmer transforms/
town state-25, the collision-opts wrappers, the research/121 arc.

## The e19 seal's 8-row drop table — corrections at the true tips

Its table (rendered against f47 `1997be092`) misclassifies three rows:

- its row 2 claims the five f47ds/f47k1 item witnesses are
  "superseded by e19's latch/clear pair + the census trio" — FALSE for
  the two latch-lifecycle twins (`:2065/:2093`): the e19 latch/clear
  witness never reads the queued splash, and no e19 witness pins the
  exit-snapshot semantics at the unit level. Those two are f47-unique
  coverage, not droppable.
- its row 3 claims the f47 pass-level quartet is a re-drive of "the
  e19 originals" that can drop — direction REVERSED: the f47 copies
  carry the `max_dust_to_draw = 6000` fix the e19 originals lack; at
  merge the e19 copies are the ones that drop.
- its row 4 claims the f47 flush pair drops to e19's landing pair —
  the wet twin and the dry landing/rest half are equal-coverage
  either way; the full-velocity half survives on e19's side under
  THIS verdict too (the split above). The row is direction-reversed,
  not content-wrong.

Rows 1/5/6/7/8 stand with the direction reversed (the e19 shape is the
loser now): the struct/latch/drain hunks drop on the E19 side per the
re-pinned list above; the probe twins + surface-offset dedupe either
way; the anchors row retires with its fn; the wiring/16-box values are
identical on both tips (trivial dedupe).

## The fold applied on this branch (this commit)

1. NEW `w4_f47k2_the_entry_splash_gate_reads_the_local_lava_probe`
   (`crates/tr-app/src/item.rs`, the `w4_f47ds`/`w4_f47k1` test block,
   after the −8 twin): the e19 seal's gate witness re-driven through
   the keeper `splash_due` latch — the sticky flag false at the tick
   head, the local probe hits, the latch reads `lava: true`, the
   `:866-869` arm sets the sticky flag the same tick, wetCount 19.
   WRITTEN NOT RUN (code-only). Compile surface checked by reading:
   `World::new`/`index`/`compute_column_surface`/`compute_chunks` (the
   same fixture calls as the neighbouring `w4_f47ds_the_entry_splash_
   latches_with_the_sticky_flags`), `ItemDrop::new` 8-arg form,
   `splash_due` field (`item.rs:941`), `Splash` fields
   (`exit/lava/shimmer/honey/pos`), `LiquidKind::Lava` — all match the
   in-file uses.

After this fold the keeper's witness set covers EVERY e19 splash pin;
no further arbitration is needed on the splash region.

## Cross-receipt — the e19 lane

**e19** (`task/W4-e19-liquid-quicksell-shimmer`, tip `0fb0ac4ad`): its
SHIMMER-seal keep-e19 ruling is OVERRULED (this doc). Its seal's real
wins STAND and are acknowledged: the verbatim entry-gate fix (verified
`:1415` — the fact that made this re-arbitration necessary and mooting
the v1 tiebreak), the two probe-twin folds, the local-gate witness
(folded here), the 145 repin execution instructions, the aether g69d
boundary, the quicksell-cursor compose, the 148 seal. Its lane keeps
every keeper-agnostic hunk listed above. Its 144 seal section's
f47-drop table is superseded by the corrected table here. The merger
reads: keeper f47 `Splash`/`splash_due`/`ItemDrop::splash`; the exact
e19 drop-hunks above (re-pinned at `0fb0ac4ad`); the e19 KEEP list;
the one split (dry-integration full-velocity half).

## Verification

Code-only per the lane contract: no cargo/tests/rustfmt/graphify.
Evidence gathered this round: both verdict docs read in full; the C#
`:862-906` window re-read (≤100 lines); both tips' `item.rs` +
`world_sim/items.rs` witness censuses (`Select-String "fn w4_"`); the
disputed witness BODIES read at both tips (the latch-lifecycle twins,
the exit-snapshot twin, the 71/31 twins, the −8 twins, the quartets,
the flush pairs, the e19 latch/clear/grabbed/gate witnesses); the e19
latch sites `:1378-1444` read verbatim; the g67 tip grepped for splash
symbols (none in items/item — the "contract" is positional); the
anchors rows on both tips; the e19 `shimmer_world` fixture (no dust
budget — the quartet gap). 1 witness WRITTEN NOT RUN (the gate fold).