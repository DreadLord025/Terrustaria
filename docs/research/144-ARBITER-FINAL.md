# 144 ARBITER FINAL — the splash keep-one, definitive (merger-blocker resolved)

> **SUPERSEDED by `379-ARBITER-FINAL-v2.md`** (renumbered
> 144→379 per registry §10; the e19 SHIMMER SEAL
> `0fb0ac4ad` landed the verbatim entry gate, mooting this doc's
> tiebreak; v2 re-rules on witness superset coverage at both new tips
> and re-pins the drop-hunks at `0fb0ac4ad`). The keeper pick is
> UNCHANGED (f47); 141/master reads v2. Kept as the lane record.

> Rendered 2026-11-21 on `task/W4-f47-liquid-shimmer-2` (tip `1997be092`,
> the arbitration commit follows). Code-only: no cargo/tests/rustfmt/
> graphify. Both tips read directly: f47 `1997be092` (this branch) and
> e19 `c7a38bea5` (`origin/task/W4-e19-liquid-quicksell-shimmer`, read via
> `git show` — the remote-tracking ref already held the exact tip; the
> fetch retry hit a connection reset, the cached tip IS the specified
> `c7a38bea`). The C# re-read: `WorldItem.cs:128-145` (the ctor 16-box),
> `:355-453` (UpdateItem head + the grabbed clear), `:620-637` /
> `:1830-1854` (the two `wet = true` tail writers), `:772-861`
> (MoveInWorld head — no early return), `:855-1049` (the full splash
> region), `:1046-1084` (the collision + ApplyMovement tail);
> `Dust.cs:402-418` (`dustWater()` — RNG-free).

## The two divergent verdicts

| | f47c (`research/144-item-splash-keep-one.md`, this branch) | e19d (`research/144-splash-keep-one.md`, e19 tip) |
| --- | --- | --- |
| keeper | f47 `Splash`/`splash_due`/`ItemDrop::splash` | e19 `ItemSplash`/`pending_splash`/`take_splash`/`item_splash` |
| rationale | verbatim LOCAL entry gate (`:886` flag2), once-per-pass water style, method-local latch spend | the C#-correct 16-box (f47 read at the STALE pre-fold tip — SIZE 14); sticky-vs-local "behaviourally void" |
| box | 16 (e19's fix FOLDED at `f80490bcf`) | 16 (`e2003946b`) |
| water style | once-per-pass `TR_WATER_STYLE` pin | per-burst recompute at verdict time; the f47 pin ADOPTED in the same receipt (`items.rs:126-130` on its tip) |

Both verdicts agree on every body term (the four families, the
post-writes, the Next(6) palette, the style-4/5 cues, the exit snapshot,
the drain slot, the 16-box). They diverge ONLY on which shape survives.
e19d's decisive argument — the f47 half's 14-box — is **stale**: it
censused the f47 tip BEFORE `f80490bcf` folded the 16. At the arbitration
tips **both halves are 16-box and body-identical**; the box argument no
longer selects a keeper.

## DEFINITIVE KEEPER: the f47 half

**KEEP `Splash` / `splash_due` / `ItemDrop::splash` (f47,
`crates/tr-app/src/item.rs`). DROP the e19 `ItemSplash` /
`pending_splash` / `take_splash` / free-fn `item_splash` at merge.**

Grounds, in decision order:

1. **C#-verbatim fidelity (the tiebreak rule).** The entry gate C#
   `:886` reads the LOCAL `bool flag2` (`:865`). f47 latches
   `lava: lava_now` (`item.rs:1356`) — verbatim. e19 latches
   `lava: self.lava_wet` (its `item.rs:1404`) — the sticky flag. The exit
   gate `:965` reads the STICKY `base.lavaWet`; BOTH lanes read
   `self.lava_wet` there (f47 `:1372`, e19 `:1418`) — faithful both.
2. **The sticky-vs-local difference: behaviourally void on BOTH current
   tips — CONFIRMED, with the exact reachable-state proof below — but
   not void by construction.** The equivalence is a lemma over the
   port's write set, not a structural identity. The verbatim read is
   robust to any future writer of `wet`/`lava_wet`; the sticky read is
   not (see the divergence edge). Fidelity wins where behaviour ties.
3. **Water style.** C# `Dust.dustWater()` (`Dust.cs:402-418`) is an
   RNG-free switch over `Main.waterStyle` — a per-frame static. f47's
   once-per-pass snapshot is the closer read; e19d ADOPTED the same pin
   on its tip, so this is now a tie — recorded, not decisive.
4. **Fold completeness.** The f47 tip already carries every e19 win:
   the 16-box + all re-pins (`f80490bcf`), `box_is_wet` restored
   (`item.rs:1063`, the `player::wet_collision` bool half), the wiring
   literals, the e19 pass-level quartet folded as `w4_f47k1_*`, the
   flush pair, the surface-offset witness. The e19 tip does NOT carry
   the f47 unique wins (the latch-lifecycle unit twins
   `w4_f47ds_the_entry_splash_latches_with_the_sticky_flags` /
   `..._exit_splash_snapshots_the_pre_clear_flags`, the 71/31 draw-count
   twins with the palette/vy pins). Keeping f47 preserves the strict
   superset; keeping e19 would orphan them.
5. **Anchors.** The keeper row already exists on this branch:
   `crates/tr-app/src/item.rs|pub fn splash|W4-f47 doc-sweep 36 …`. The
   e19 row `crates/tr-app/src/world_sim/items.rs|fn item_splash|W4-e19
   round 2 …` retires WITH its function (the row and the code drop
   together — no canary failure).

## The sticky-vs-local proof (e19d's claim, verified at the edges)

C# writers, full census (`Select-String lavaWet|wet =|wetCount =` over
`WorldItem.cs`):

- `lavaWet`: SET only at `:868` (inside MoveInWorld, AFTER the `:865`
  probe, BEFORE the `:886` gate); CLEARED only at `:429` (the grabbed
  arm, which also zeroes `wetCount` `:428` and `wet` `:427`) and `:1038`
  (the `!wet` tail, EVERY MoveInWorld pass).
- `wet`: SET at `:635` (SpawnShimmeredItem callback), `:956` (the entry
  arm), `:1844` (the Shimmer transform tail); cleared at `:427`, `:961`.
- `wetCount`: set 20 at `:885`/`:964`, zeroed at `:428`, decremented at
  `:1042-1045` EVERY MoveInWorld pass.
- `MoveInWorld` (`:772-861`) has NO early return before the wet block;
  the gravity head runs unconditionally when `!beingGrabbed`.

**Lemma.** At the entry edge (`flag3 && !wet && wetCount == 0`,
`:879-885`) the sticky `lavaWet` equals the local `flag2`:

- The previous pass ended with `wet == false` (nothing between
  MoveInWorld's tail and this pass's `:881` sets `wet` except the two
  transform writers — excluded next), so the `:1036-1041` tail cleared
  `lavaWet`; OR the drop was grabbed, and `:427-429` cleared it (and
  zeroed the cooldown, making the edge reachable the very next tick).
- Either way `lavaWet` is FALSE at this tick's `:866`; if `flag2` is
  true, `:868` re-sets it before `:886`. Sticky == local. ∎

**The transform edge (the one both receipts hand-waved).** `:635` /
`:1844` set `wet = true` mid-life, so a drop can arrive at MoveInWorld
with `wet` true and a stale sticky set. But then `!wet` is FALSE — the
entry arm cannot fire on that tick, and the `:956`/tail path keeps
`wet` true while in liquid or clears the sticky flags on the dry exit
pass. By the next ENTRY edge the lemma's precondition holds again.
SpawnShimmeredItem also leaves `wetCount == 0`, but its first
MoveInWorld pass has `wet == true` — no burst; the eventual exit burst
reads the sticky flags, which both lanes do. **Void on every reachable
state of the CURRENT write set — e19d's claim CONFIRMED.**

**Why it still decides the keeper:** the lemma is one future writer
away from breaking. If any port lands a `wet = true` write without the
C#-paired flag context (exactly the class of bug the f47 doc-sweep's
missing `shimmer_tail` wet write was), the sticky read can fire a LAVA
burst on a shimmer/water entry where C# fires the shimmer/water family
— a wrong dust family AND a wrong RNG stream length (5×6+1 vs 10×7+1),
desyncing every later draw. The verbatim local read cannot diverge by
construction. Under "C#-verbatim-fidelity wins", f47 keeps the gate.

**Exit gate: no difference at all** — C# itself reads the sticky flag
at `:965`; both lanes match. The arbitration is ONLY the entry read +
the shape.

## Exact drop-hunks for the loser (e19, at merge — 141/master)

All line numbers are the e19 tip `c7a38bea5`:

1. `crates/tr-app/src/item.rs`
   - `:884-904`: the `ItemSplash` doc block + derive + struct (the
     struct at `:891`, fields through `:903`, close `:904`).
   - `:942-947`: the `pending_splash: Option<ItemSplash>` field + doc.
   - `:1003`: the `pending_splash: None` initializer.
   - `:1016-1020`: `take_splash` + doc.
   - `:1401-1407` / `:1415-1421`: the two `self.pending_splash =
     Some(ItemSplash { … })` latch sites — replaced by the f47
     `splash_due = Some(Splash { … })` sites (entry `lava: lava_now`,
     exit `lava: self.lava_wet`). The surrounding gate/cooldown/clear
     lines are identical on both tips and stay.
2. `crates/tr-app/src/world_sim/items.rs`
   - `:700-711` (+ the `:693-699` comment): the `drop.take_splash()`
     drain block — replaced by the
     f47 `if drop.splash_due.is_some() { drop.splash(…) }` block (same
     slot: post-`drop.update`, pre-flicker head).
   - the free fn `item_splash` (`:777`) + its doc block, through the
     closing `play_splash_sound` + brace at `:850-851`.
   - `:126-130`: the pass-head `splash_water_style` pin STAYS (the f47
     drain consumes the identical head; dedupe to one copy).
   - The seven splash witnesses: the quartet `:1615`/`:1654`/`:1678`/
     `:1696` (folded as `w4_f47k1_*` on f47) and the census trio
     `:1734`/`:1791`/`:1835` (folded into the f47 twins + the new
     `w4_f47k1_the_water_exit_anchor_drops_the_minus_8`, this commit).
   - KEEP: `w4_e19_the_wet_flags_latch_and_clear` (`item.rs:2670`),
     `w4_e19_the_grabbed_arm_clears_every_wet_flag` (`:2700`), the
     gravity/integration/repulsion family — f47 has no pass-level
     twins for the latch/clear/grabbed pair.
   - `w4_e19_the_wet_probe_uses_the_liquid_surface_offset` (`:2414`)
     drops (folded as `w4_f47ds_the_box_is_wet_probe_uses_the_liquid_
     surface_offset`); `w4_e19_the_wet_landing_keeps_the_flush_residual`
     (`:2620`) and the landing/rest halves of
     `w4_e19_the_dry_drop_integrates_at_full_velocity` (`:2594`) drop
     ONLY where the f47 `w4_f47k1_*` flush pair covers the same pins —
     the e19 dry-integration witness's full-velocity half has no f47
     twin: KEEP that half (split, don't delete wholesale).
3. `crates/tr-app/src/world_sim/wiring.rs`: the e19 `ItemDrop` literals
   (`pending_splash: None` at `:202`/`:795`) lose `pending_splash`,
   gain `splash_due` (the f47 literals at `:190`/`:783` are the
   canonical forms — both carry all five phase-3 fields already).
4. `anchors.txt`: DROP the e19 row
   `crates/tr-app/src/world_sim/items.rs|fn item_splash|W4-e19 round 2
   …` in the SAME merge hunk that deletes the fn; KEEP the f47 row
   `crates/tr-app/src/item.rs|pub fn splash|W4-f47 doc-sweep 36 …`.
5. e19's research receipts (`144-splash-keep-one.md`, `148`'s contract
   1) are SUPERSEDED by this doc — keep the files as the lane record,
   the merge reads THIS verdict.

**e19-owned hunks that still survive the merge** (unchanged from f47c's
list, re-confirmed at the e19 tip): the `shimmer_tail`/`apply_decraft`
`wet = true` writes (`WorldItem.cs:1844`/`:635` — the f47 Transform arm
still lacks them), the `rise_over_shimmer` two-row centre scan
(`:493-504`), and the collision-engine reconciliation (f47's
`tile_collision_opts` wrappers are the shared seam; the residual
semantics agree — `Collision.cs:2394`/`:2444`, both keep the flush
residual as velocity).

## The 16-box repin reconciliation (e19d-145 vs the f47c witnesses)

e19's `research/145-16box-cross-lane-repins.md` censused the pushed
tips BEFORE the f47 fold; its f47 row ("`item.rs:548` SIZE = 14.0 —
retire") is stale. Reconciled state at the arbitration tips:

| 145 rule | e19 tip pin | f47 tip pin (post-fold) | verdict |
| --- | --- | --- | --- |
| SIZE itself | 16 (`e2003946b`) | `item.rs:555` `SIZE: f32 = 16.0` | ALIGNED — the 145 f47 row is retired by the fold |
| grab edge 42/43 | `item.rs:2137-2138` | `item.rs:2548-2549` (identical asserts) | ALIGNED — dedupe to one copy |
| explicit 14×14 helper probes | `:2140-2141` stay | same policy | ALIGNED (rule 1 tail) |
| magnet 292/192 | `:2325`/`:2339`, y==192 `:2334` | `:2736`/`:2750` heart+star at 292/192 | ALIGNED |
| resting flush `floor_top − SIZE` | `editing.rs:17856/:17863` | `editing.rs:17872/:17879` | ALIGNED |
| splash anchor pos.Y+0 / roll 24 | census trio `:1734-1863` | twins `:2137-2145`/`:2189-2196` + the folded bands (this commit) | ALIGNED |
| light anchors 168/184, −12 trunc | `e2003946b` | `entity_lights.rs:444-446` + the 16.0 fixture args | ALIGNED |
| entry-latch park 184, probe 10×8 at +3/+4 | wet-probe parking −8 | `item.rs:2075` comment + `:2081` park | ALIGNED |
| `pc−4` pickup spawns | NO repin (rule 4) | untouched | ALIGNED |
| mount 14×14 player override | NO repin (Rat hitbox, `DelegateMethods.cs:190-194`) | untouched | ALIGNED |
| f49 nebula `pc+93/−7` → `pc+92/−8` | lane-unique, listed | not an f47 file | stands as written — f49's merge applies it |
| g66/e1/f6/f49/d79/g25/g79/g56/f40/d98/g48/g52 i47/i52 blocks | per-lane list | the same main-inherited block, already 16-pinned on f47 | 145's merge-order note stands: first 16-box merge repins once, later merges take the 16-pinned side; the f47 tip is now ALSO a canonical 16-pinned side |

No 145 rule needs re-derivation; the only edit is striking its f47
"retire SIZE 14" row (superseded — the fold landed the 16 on f47
independently, hand-derived to the same pins).

## The arbitration folds applied on this branch (this commit)

The e19 census trio's UNIQUE pins folded into the keeper witnesses
(`crates/tr-app/src/item.rs`, WRITTEN NOT RUN — code-only):

1. `w4_f47ds_the_shimmer_entry_splash_draws_71_and_colours_verbatim`:
   + the 16-box anchor bands (x 298..321, y 304..323 at pos 300/300 —
   the e19 `..._draws_71_at_the_16_box` bands).
2. `w4_f47ds_the_lava_splash_wins_the_gate_and_draws_31`: + the
   `no_gravity` pin + the 5-active-dust-35 family count (the e19
   `..._draws_31_and_wins_the_gate` asserts).
3. NEW `w4_f47k1_the_water_exit_anchor_drops_the_minus_8`: the e19
   same-seed entry/exit pair re-driven through `ItemDrop::splash`
   (the Y delta exactly 8, the entry band at pos.Y+0).

After these folds the keeper's witness set covers EVERY pin of the
e19 seven; the e19 splash witnesses drop wholesale at merge per the
hunk list above (with the two KEEP/split exceptions noted).

## Cross-receipt

- **e19** (`task/W4-e19-liquid-quicksell-shimmer`, tip `c7a38bea5`):
  its 144 verdict is OVERRULED on the keeper pick (its 16-box argument
  was stale at rendering time); its "behaviourally void" lemma is
  CONFIRMED and its `TR_WATER_STYLE` adopt is acknowledged (now a tie).
  Its 145 repin list stands minus the stale f47 row. Its lane keeps the
  chain-apply/rise/collision hunks + the latch/clear/grabbed witnesses.
- **f47** (this branch): the f47c verdict is UPHELD; this commit adds
  the trio folds so the keeper superset is complete. The keeper row in
  `anchors.txt` is unchanged.
- **141/master**: the merger-blocker is RESOLVED — one keeper (f47
  `Splash`/`splash_due`/`ItemDrop::splash`), the exact e19 drop-hunks
  above, the reconciled 145 table, the e19 KEEP list. No further
  arbitration needed on the splash region.

## Verification

Code-only per the lane contract: no cargo/tests/rustfmt/graphify.
Every verdict re-read against the cited C# windows (≤100-line reads)
and both tips' pushed code (`git show` for e19, direct reads for f47).
3 witness edits WRITTEN NOT RUN (2 folds + 1 new). The compile-surface
of the folds checked by reading: `DustPool::slots()` (`particles.rs:258`),
`Dust.active/dust_type/position` (`:118/:120/:109`), `SpawnCtx` derives
`Default` (`:180-181`), `Splash` field set (`item.rs:890`), the
`ItemDrop::splash` signature (`:1457-1465`) — all match the folded
calls.

## § FINAL v2 — the SPLASH seal consumed (e19 `0fb0ac4ad`), verdict STANDS

Cross-receipt to the e19 seal (`docs/research/144-splash-keep-one.md`
at e19 tip `0fb0ac4ad`, read via `git show`, seal section + its 8-row
f47-drop table). This f47 lane acknowledges the seal and answers it:

1. **e19 entry-gate fix ACKNOWLEDGED — the objection is MOOT.** The
   e19 tip now parks `let lava_now = player::lava_collision(...)` and
   the ENTRY `ItemSplash` latches `lava: lava_now` (the C# `:886`
   `!flag2` local), exit keeps the sticky `lava_wet` (`:965`), pinned
   by `w4_e19_the_entry_splash_gate_reads_the_local_lava_probe`. The
   grounds-1 fidelity gap cited above is closed on the e19 half. Both
   halves now run the identical 16-box and behaviourally identical
   gates — the tie is total on behaviour.
2. **KEEP f47 — the `18fe75f2e` verdict STANDS on the superset
   rationale.** With behaviour tied, the keeper is the strict
   superset: the f47 half carries the latch-lifecycle unit twins
   (`w4_f47ds_the_entry_splash_latches_with_the_sticky_flags` /
   `..._exit_splash_snapshots_the_pre_clear_flags`) and the 71/31
   draw-count twins with no e19 twin; the e19 half carries no unique
   behaviour the f47 half lacks. The verbatim-local entry read is now
   on both, so grounds 1-3 are ties; grounds 4 (fold completeness)
   decides, unchanged.
3. **The e19 8-row drop table is REVERSED.** The e19 seal published
   f47-drop hunks; the keeper verdict inverts them — **e19 drops its
   half, not f47**. The e19-side drop hunks, from its own table
   inverted (the KEEP/split exceptions in the drop-hunk list above
   still apply):

   | # | e19 hunk (tip `0fb0ac4ad`) | Fate at merge |
   | --- | --- | --- |
   | 1 | `ItemSplash` + `pending_splash` field + the two `Some(ItemSplash{..})` sites + `take_splash` + free-fn `item_splash` | DROP — f47 `Splash`/`splash_due`/`ItemDrop::splash` survive |
   | 2 | census trio (`..._draws_71_at_the_16_box`, `..._draws_31_and_wins_the_gate`, `..._the_minus_8`) + the new local-probe gate witness | DROP — folded/covered by the f47 71/31 draw twins, the −8 twin and the entry-latch twin. EXCEPTION (per the list above): `w4_e19_the_wet_flags_latch_and_clear` + `w4_e19_the_grabbed_arm_clears_every_wet_flag` KEEP — no f47 pass-level twin |
   | 3 | e19 pass-level splash quartet (`world_sim/items.rs:1615/:1654/:1678/:1696`) | DROP — the f47 `w4_f47k1_*` re-drives are the same four assertions on the keeper drain |
   | 4 | `w4_e19_the_wet_landing_keeps_the_flush_residual` + the landing/rest halves of `w4_e19_the_dry_drop_integrates_at_full_velocity` | DROP only where the f47 flush pair covers the pins — the full-velocity half KEEP (split, per the list above) |
   | 5 | `w4_e19_the_wet_probe_uses_the_liquid_surface_offset` | DROP — byte-identical body to the f47 `w4_f47ds_*` fold |
   | 6 | the two folded probe twins `w4_e19_the_wet_probe_is_the_centre_inset_box` / `..._respects_the_liquid_surface_drop` | DROP the e19 copies — the f47 originals (`w4_f47ds_*`, `item.rs:1990/:2013`) **KEEP as f47 witnesses**; they are the f47-unique probe geometry the e19 seal folded, and the keeper half retains its own |
   | 7 | anchors row `crates/tr-app/src/world_sim/items.rs\|fn item_splash\|W4-e19 round 2 …` | RETIRE with hunk 1 — the f47 row `crates/tr-app/src/item.rs\|pub fn splash` survives |
   | 8 | wiring literals + `box_is_wet` + SIZE-16 const/quad/light re-pins | KEEP whichever side lands first — identical values both lanes; trivial dedupe (unchanged, NOT a drop) |

4. **Cross-receipt e19.** The e19 seal doc is received and its
   entry-gate/water-style/16-box parity claims verified against its
   tip; its keep-e19 verdict and its f47-drop table are OVERRULED by
   this FINAL (v1 keeper pick + this v2 inversion). The e19 doc stays
   as the lane record; the merge reads THIS verdict. No code changes
   this round — docs only, code-only lane contract honoured (no
   cargo/tests/rustfmt/graphify).