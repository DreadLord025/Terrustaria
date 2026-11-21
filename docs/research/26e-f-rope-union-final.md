# 26e-f — Rope-coil FINAL union (W4-g82 final phase, 2026-11-21)

> Re-adjudication of `26e-rope-coil-keep-one.md` (W4-d4 DOC-SWEEP item 1,
> `0290de417`, d4 tip `4fa74b3c7`). That sweep compared d4's LANDED
> `ai_rope_coil` against g82 tip `de797f45` — then **UNWIRED code-only** —
> and ruled KEEP-d4-machine + g82-extras. Since then g82 wired the FULL
> lane (tip `59139eb19`: dispatch-35 + the four rows + the Kill-refund
> ladder + the ai1==0 cue + CheckRopeUsability + the chain placement
> preview, all live), and d4's latest tip still carries NONE of the
> extras (re-verified this pass: zero hits for `77572`/`coil_kill`/
> `coil_death`/`check_rope_usability`/`draw_coil` anywhere under d4's
> `crates/`). The comparison's premise flipped: g82 is now the wired
> superset, d4 the bare machine.
>
> This pass also FIXED the two faithfulness deltas 26e credited to d4
> (g82 `c2b7507a0`), so the verdict reverses cleanly. Code-only: no
> cargo/tests/rustfmt/graphify invoked.
>
> **SEAL addendum (2026-11-21, this commit):** the "d4 carries NONE of
> the extras" probe above pre-dated d4's union push. d4 tip `cafec7222`
> NOW carries the extras (`bb2f8d2cf` refund wire, `1cae02fd8` usability
> wire, `ee950e510` preview census, `b09b75bd0`/`94a51fb79` item/row
> verifies, `21f46f65d`+`cafec7222` death-centre timing). The full
> re-read + byte-compare lives in "The d4-cafec722 union re-read" below;
> the verdict STANDS (the pure extras are copies of g82 `67c9a740`, the
> wiring deltas all favour g82 against the re-read C#), the drop-hunks
> table is UPDATED to `cafec7222` lines, and the ROPE-FAMILY merger
> bible closes the lane.
>
> **DRAIN-RECEIPT addendum (2026-11-21, g82e — this commit):** the
> g82d seal's drain delta above ("g82's drain survives") is REVERSED
> on the SHAPE axis: d4's two-integer-draw drain is the faithful
> `Item.NewItem` emit, g82's `spawn_item_drop_px` is the wrong
> tile-loot shape — **d4's drain CANONICAL** per the d4d retire
> re-adjudication (`26e-rope-coil-keep-one.md` FINAL-FINAL), with the
> ball's zero-velocity fix LANDED at d4 tip `d7c3fbc02` (the
> SEAL-VERIFY commit: the queue tuples carry `zero_velocity`, coil
> `false` / ball `true` — `Projectile.cs:77622` passes `Vector2.Zero`,
> the `Item.cs:49738-49741` HasValue arm draws NOTHING). The machine
> verdict STANDS unchanged (g82 `coil_step`). The full re-read + the
> corrected rows 7/8 + the g82-side drop-hunks + the FINAL merger
> bible live in "The g82e drain receipt" below; the mutual receipt
> pins the canonical drain source at `d7c3fbc02`.

## FINAL VERDICT: g82's `coil_step` is the CANONICAL style-35 machine

**Keep `task/W4-g82-rope-coil-draw` whole** — `projectile_rope_coil.rs`
(`coil_step`/`coil_proj_row`/`coil_rope_kind`/`is_rope_cell`/`is_coil` +
the chain preview + `coil_death_drop` + `check_rope_usability`), the
`35 =>` dispatch arm, the `data.rs` literal rows, the `coil_kill` death
hook's cue + gate shape (its queue/drain repointed to d4's — the g82e
drain receipt below), the usability gate in `use_tile_item`,
the chain sheets in `assets.rs`, the friendly pin, the PVP exemption,
the style-35 `AutomaticallyChangesDirection` write, the wind-list 35.

**Drop from `task/W4-d4-proj-ai-31-40`** (exact hunks, UPDATED to d4 tip
`cafec7222` line numbers — the union push added the extras module and
its wiring; every row grep-verified at that tip):

| # | file | hunk (`cafec7222`) | what |
| --- | --- | --- | --- |
| 1 | `crates/tr-app/src/projectile.rs` | `:11394-11522` | the `ai_rope_coil` fn + its doc comment (unchanged by the union push) |
| 2 | `crates/tr-app/src/projectile.rs` | `:17812-17818` | the `35 => self.slots[i].ai_rope_coil(world, local_owner)` dispatch arm + its comment (was `:17792-17798`; a second `35 =>` would trip the `unreachable_patterns` deny anyway — S1-D-4c) |
| 3 | `crates/tr-app/src/projectile/data.rs` | `:604-607` | the `ROPE_COIL`/`VINE_ROPE_COIL`/`SILK_ROPE_COIL`/`WEB_ROPE_COIL` consts — **consumers UPDATED**: after dropping rows 1/2/4/5/6 below the only survivors would be d4's extras module (`projectile_rope_coil.rs:49` import) which row 6 drops wholesale; g82's module owns its own `pub(crate)` consts (`projectile_rope_coil.rs:92-95`) |
| 4 | `crates/tr-app/src/projectile/data.rs` | `:2326-2341` | the named-const row arm (the row-dedupe, below — unchanged) |
| 5 | `crates/tr-app/src/projectile.rs` | `:26888-26990` + `:27181-27206` *(f1480dc0 lines; at `d7c3fbc02` head +14 / tail +15 — the ball-witness assert grew a line between them: `:26902-27004` + `:27196-27221`)* | the four `the_rope_coil_*` MACHINE witnesses (was `:26810-26939`, then the whole `:26861-27111` span — that span's head is the FLAME-JET witness, a mis-pin; **CORRECTED by the g82e drain receipt**: the union refund-queue witness `w4_d4_union_the_coil_death_queues_the_refund_ladder` (`:26998-27075` at `f1480dc0` → `:27006-27089` at `d7c3fbc02`) and the ball-return witness (`:27081-27111` → `:27091-27126`) are KEEPS — the refund witness drives `pool.update` so the dispatch swap to `coil_step` needs no change, and it pins the canonical queue (its exact-tuple assert carries the `zero_velocity` flag `false` since `d7c3fbc02`); the per-kind placement ladder salvage note below) |
| 6 | `crates/tr-app/src/projectile_rope_coil.rs` | whole file (`:1-331`) | **NEW at cafec722** — d4's extras module: `is_coil`/`coil_death_drop`/`CoilLink`/`coil_chain_sheets`/`coil_chain_segments`/`draw_coil_chain` are byte-compare-identical copies of g82's (modulo doc/visibility, see the re-read below); its 4 witnesses duplicate g82's minus the NaN pin + plus the is_coil gate rows (salvage below) |
| 7 | `crates/tr-app/src/projectile.rs` | `:14751`, `:15166`, `:15266-15272`, `:20389-20419` *(cafec7222 lines)* | **NEW** — the `pending_death_drops` field + init + `drain_death_drops` + the death-hook refund arm. **CORRECTED by the g82e drain receipt: KEEP THE WHOLE HOOK BLOCK** (coil ladder + beach-ball arm + the retire-fixed `954824d0` gates/cue — behaviourally identical to g82's `coil_kill`, which is the g82-side drop now); its queue/drain half keeps d4's field + `drain_death_drops` + the two-integer-draw drain loop — the CANONICAL `Item.NewItem` emit for BOTH kinds (d4 tip `d7c3fbc02` lines: field `:14756`, init `:15171`, drain fn `:15277`, hook `:20396-20460`; the queue is the 5-tuple `(u16, i32, f32, f32, bool)` — the `zero_velocity` flag, coil push `:20445` `false`, ball push `:20457` `true`, the SEAL-VERIFY ball fix LANDED) |
| 8 | `crates/tr-app/src/world_sim/projectiles.rs` | `:620-658` (drain + comment), `:775-868` (the two witnesses + `coil_app`) *(cafec7222 spans, corrected)* | **NEW** — **CORRECTED by the g82e drain receipt: KEEP THE WHOLE DRAIN + BOTH WITNESSES.** The seal dropped the coil witness as "WRONG stream + gates"; its gates were FIXED by `954824d0` and its stream pin (`UpdateItems`, two integer draws) is the canonical emit's recorded deviation — g82's `spawn_item_drop_px` drain is the wrong-shape half (the g82-side drop-hunks below). d4 tip `d7c3fbc02` lines: drain `:620-667` (the conditional scatter pair `:640-648` — `zero_velocity` true → (0,0), ZERO draws), witness module `:785-889`, coil witness `:812-856`, ball witness `:858-888` (pins (0,0) + EVERY stream pristine — the SEAL-VERIFY fix, the ball sub-delta CLOSED), `coil_app` `:802` |
| 9 | `crates/tr-app/src/world_sim/editing.rs` | `:7193-7265`, `:7330-7337`, `:8289-8376` | **NEW** — d4's private `check_rope_usability` copy + its `use_tile_item` call + its 3 witnesses. The fn is byte-compare-identical to g82's `projectile_rope_coil.rs::check_rope_usability` modulo `pub(crate)`; g82's module copy + call site (`editing.rs:7204`) survive, d4's inline copy drops |
| 10 | `docs/ANALYSIS.md` | `:39367-39453` ("The rope-coil lane extras (W4-d4 union)") | **NEW** — superseded by g82's "The rope-coil lane (W4-g82)" section + this doc; the item-row verifies `:39455-39480` (union items 4/5) are d4's OWN census — KEEP those paragraphs |

**Keep from d4 (NOT rope-duplicates):** the wind-list **32** (the beach
ball, d4's own machine — the merged `uses_wind_physics` list is d4's
`2 | 8 | 24 | 28 | 29 | 32 | 35` verbatim, g82's list + 32); everything
else d4 landed (styles 31-40 less 35, the DOC-SWEEP items 2-5). d4's
`26e`/`26d` follow-up note stay as history — **superseded by this doc**.

### Row-dedupe (171/475/505/506 — both ported, term-identical)

* g82 `projectile/data.rs:3979-3980`: `171 | 505 | 506 => (14.0, 14.0,
  35, -1, 400, 0, false, false, false, RESET_STATE)` + the same tuple for
  `475` (the two C# arms `:2570-2578`/`:5496-5504` are term-identical).
* d4 `projectile/data.rs:2330-2341`: `ROPE_COIL | VINE_ROPE_COIL |
  SILK_ROPE_COIL | WEB_ROPE_COIL => (…)` — the SAME ten terms.
* Merge resolution: keep g82's literal arm (hunk 4 drops d4's). g82's
  `coil_proj_row` stays the witness-pinned reference (the module-doc
  convention), NOT a second data source.

## Arm-by-arm re-comparison (`Projectile.cs:27189-27291`, re-read verbatim)

The 26e table's nine rows re-checked against g82 tip + the two fixes:

1. **Head** (`:27191-27201`): identical maths. **g82 now MORE faithful on
   the spin sign**: the C# reads `base.direction` (the slot field the
   generic `AutomaticallyChangesDirection` write maintains, `:17016-17026`
   — 35 is not excluded, `:17281-17288` re-read). g82 reads `p.direction`
   AND landed the style-35 post-AI write (`projectile.rs:19866-19869`) —
   the exact C# one-tick-lag shape. d4 computes the velocity sign inline
   at spin time and never writes the slot (its own ANALYSIS records the
   frozen-direction deviation) — a same-tick deviation g82 does not carry.
2. **Owner gate** (`:27202-27205`): identical (both free-move + return).
3. **Collision probe** (`:27206-27222`): identical (fallThrough/fall2,
   the `!= velocity` test, the truncation-before-/16, the bottomSlope
   snap `(tx*16+8, ty*16+24)`).
4. **Walk anchor** (`:27240-27242`): **g82 more faithful** — `(p.w as i32
   / 2)` is the C#'s INTEGER `base.width / 2`; d4's `self.w / 2.0` is f32
   division (equal for 14, divergent for any odd future width).
5. **Null-tile abort** (`:27244-27247`): 26e's d4-winning delta — **FIXED
   in g82 `c2b7507a0`**: the abort now integrates (`p.pos += p.vel`), the
   C# returns from AI only, `HandleMovement` still runs (`:17015` →
   `UpdatePosition` `:20445`; 35 not on the `:20435` skip list).
6. **IsRope skip** (`:27248-27250`): both verbatim `WorldGen.cs:70026-
   70104`. g82's inline `is_rope_cell` (the d56 inlined-table convention)
   stays — the canonical module owns its probe; d4's fold to
   `render::ropes::is_rope` dies with its arm (the render fn itself stays,
   S1-G-1's).
7. **Placement walk** (`:27251-27289`): identical (the 314/platform
   alternator with no budget spend, the tileCut/165 clear, the
   place+ai[1]++, the solid-stop, the 10-link cap). The cut-tile
   `TileKill` records: g82 collects + the dispatch discards; d4 drops
   inline (`let _ =`). Observationally IDENTICAL — the pool-wide gap
   either way (census below).
8. **Kill** (`:27290`): 26e's second d4-winning delta — **FIXED in g82
   `c2b7507a0`**: the kill fields (`kill_time_left` capture + active off
   + timeLeft 0, the pool's `kill()` shape) PLUS the dead-tick integrate
   (`UpdatePosition` moves the dead slot too; the `!active` return sits
   AFTER HandleMovement, `:17027`).
9. **SendData(17) pairs** (`:27269`/`:27277`): skipped in both (MP).

**g82-only pieces d4 never carried (all WIRED, re-verified at tip):**
the friendly=false pin (`:788` reset — d4's arm leaves the port Default's
true, so d4's coils would trip the port's `!friendly` PVP proxy the
wrong way AND read friendly=true in the PVE pass), the PVP exemption
(`projectile.rs:19916-19923` — d4's exemption list has the flame jet but
NOT the coils), the style-35 direction write, the wind tail (both have
35; d4 adds 32), the Kill-refund ladder + cue (`coil_kill`,
`projectile.rs:21195` — the `:77267`/`:77367` gates kept, the cue
ungated), `check_rope_usability` in `use_tile_item`
(`world_sim/editing.rs:7204`), the chain placement preview
(`draw_with_hand`, `projectile.rs:10287` + the 7 packed sheets), the
framing arm (`tr_core::framing_rope`, no d4 overlap).

**Witness salvage note:** d4's `the_rope_coil_places_its_rope_and_dies`
loops the placement over all FOUR kinds; g82's placement witnesses rig
171 + 475 and pin the four-kind rope map separately
(`w4_g82_the_four_rows_and_rope_kinds`). Coverage-equivalent through the
map + the shared `coil_rope_kind` call path; the merge MAY port d4's
four-kind loop onto `coil_step` as a bonus witness, nothing else.
Two more d4-cafec722 salvage rows: the refund-ladder witness's `is_coil`
gate asserts (`projectile_rope_coil.rs:247-248` — g82's ladder witness
lacks them) and the NaN-quirk witness
(`w4_d4_union_the_cancelled_sag_keeps_the_nan_quirk`, `:311-330` — g82
documents the quirk in a comment but pins no witness; d4 rigs vel
(0, 0.4) at ai0=30 so −vel cancels the first sag step exactly). Both
port onto g82's module verbatim (rename `w4_d4_union_` → `w4_g82_`).

## The d4-cafec722 union re-read (SEAL pass, 2026-11-21)

d4 tip `cafec7222` (six commits past `4fa74b3c7`: `bb2f8d2cf`,
`1cae02fd8`, `ee950e510`, `b09b75bd0`, `94a51fb79`, `21f46f65d`,
`cafec7222`) read in FULL. The extras are copies of g82 `67c9a740`
(d4's own module doc says so); a mechanical normalized byte-compare
(comments/whitespace stripped, brace-balanced fn extraction) confirms:

| piece | d4 (`cafec7222`) | g82 (this branch) | verdict |
| --- | --- | --- | --- |
| `is_coil` | `projectile_rope_coil.rs:56` | `:99` | IDENTICAL (3 norm-lines; d4 adds `#[allow(dead_code)]`) |
| `coil_death_drop` | `:65` | `:447` | IDENTICAL (16 norm-lines) |
| `CoilLink` struct | `:85` | `:318` | IDENTICAL |
| `coil_chain_sheets` | `:99` | `:331` | IDENTICAL (8 norm-lines) |
| `coil_chain_segments` | `:126` | `:355` | IDENTICAL (36 norm-lines) |
| `draw_coil_chain` | `:177` | `:403` | IDENTICAL (38 norm-lines; d4 unwired `#[allow(dead_code)]`, g82 LIVE at `draw_with_hand`) |
| `check_rope_usability` | `world_sim/editing.rs:7207` | `projectile_rope_coil.rs:474` | IDENTICAL modulo `pub(crate)` (57 norm-lines) |
| ladder witness | module tests `:235` | `:822` | d4 = g82 + the is_coil gate rows (salvage above) |
| reverse-order + sheet-parity witnesses | `:253`/`:286` | `:767`/`:800` | IDENTICAL bodies |
| usability call site | `editing.rs:7337` | `editing.rs:7204` | same seam position (after registry resolve, before the style/lava reads) |

So the PURE extras dedupe is trivial: g82's module survives whole,
d4's copies drop (table rows 6/9). The WIRING is where d4's union
diverges — three deltas, each re-checked against the C# this pass:

1. **The refund gates.** d4's hook comment claims "NO owner gate —
   `ReserveForLocalPlayer` is a grab reservation, not a spawn gate"
   (`projectile.rs:20397-20401`). **Misread**: the ladder
   `Projectile.cs:77572-77615` sits INSIDE `Kill_ExplodeTiles`
   (`:77265`), behind the `owner != Main.myPlayer → return` head
   (`:77267-77270`) AND the `if (!this.noDropItem)` block (`:77367`)
   that encloses it through `:77628`. Kill() calls it at `:77258-77261`
   under `flag2` (true for every non-bomb style, `:70660`; only the
   aiStyle-16 arm can clear it, `:70682` — 35 never does). The
   `ReserveForLocalPlayer` argument is indeed only a grab reservation
   (`Item.cs:49757-49760`) — but that is ORTHOGONAL to the enclosing
   gates. g82's `coil_kill` keeps both (`projectile.rs:21206`, the
   cue correctly ungated ahead of them); d4's hook refunds foreign
   coils and `noDropItem` coils the C# never drops. g82 wins.
   *(The gates half FIXED on d4 by the retire commit `954824d0` —
   its hook now behaviourally identical to `coil_kill`; the g82e
   receipt keeps d4's whole hook block for the canonical emit.)*
2. **The drain stream + shape.** d4 drains on `UpdateItems`
   (`world_sim/projectiles.rs:637`) with a hand-built `ItemDrop`
   (two integer draws `:638-639`, prefix 0). The C# `RequestNewItem`
   → `NewItem` velocity draws (`Item.cs:49749-49750`) run INSIDE
   `Kill()` — under the `UpdateProjectiles` swap (`Main.cs:17775-
   17778`, re-read; `UpdateItems` is the NEXT using-block `:17779`).
   d4's own ANALYSIS records this as a deviation ("the bobber
   overflow's recorded stream deviation, extended"); g82 drains on
   `UpdateProjectiles` (`world_sim/projectiles.rs:613`) through
   `spawn_item_drop_px` — the closer stream. (Both ports still draw
   g82's triplet-vs-pair shape through their helpers; the exact
   NewItem draw-shape audit rides the R-26-13 loot-seam task, NOT
   this lane.) g82's drain survives. *(REVERSED on the shape axis by
   the g82e drain receipt below — the seal weighed the STREAM (g82
   closer) but not the DRAW SHAPE: `spawn_item_drop_px` draws the
   tile-loot triplet + natural-prefix roll the ladder's `prefix: 0`
   calls never make; d4's pair is the exact emit. d4's drain is
   CANONICAL — `Item.cs:49749-49750` re-read.)*
3. **The death-centre timing.** d4 `21f46f65d`+`cafec7222` pin the
   refund centre AFTER the dead-tick integrate (the pool's death hook
   reads `base.Center` post-`UpdatePosition`; the C# Kill captures it
   mid-AI) and record it as a POOL-WIDE deviation shared by every
   Kill arm (the beach ball's +0.1 dry-gravity Y). g82's hook reads
   the same post-integrate slot — SAME deviation, same seam, nothing
   to reconcile: the deviation belongs to the pool's hook placement,
   not to either coil port. d4's ANALYSIS paragraph
   (`:39402-39406`) is the better write-up; the merger keeps that
   paragraph's text under g82's section (the beach-ball witnesses
   stay d4's own — style 32 is d4's machine).

Also re-verified at `cafec7222`: the wind list is `2|8|24|28|29|32|35`
(`projectile.rs:13896`) = g82's + 32 (the merger-bible keep); the
data.rs row arm is the same ten terms (`:2330-2341` vs g82
`:3979-3980` literal arms); the machine `ai_rope_coil` is unchanged
(`:11394-11522` — still no friendly pin, no direction write, the f32
`w/2.0` anchor, the inline velocity-sign spin); d4's PVP exemption
list still lacks the coils; the item-row verifies (`b09b75bd0`/
`94a51fb79`: 859/1156 itemdata + the 190 proj row) touch no g82 file —
d4's own, KEEP.

**FINAL verdict UNCHANGED: g82 canonical, stands.** The union push
added copies + a mis-gated wire, not a better machine. *(The gates
half was then FIXED on d4 by the retire commit `954824d0`; the drain
delta was REVERSED by the g82e receipt below — machine verdict
unchanged, emit verdict d4.)*

## The g82e drain receipt (2026-11-21, g82 tip `6e5f6b74` → this commit)

> Input: the d4d retire lane's FINAL-FINAL re-adjudication (d4 tip
> `f1480dc0b`, `26e-rope-coil-keep-one.md` at that tip — the per-fn
> byte-compare + the drain diff + the ONE-branch merge instruction),
> re-verified HERE against the C# and both ports' code, then UPDATED
> against d4's SEAL-VERIFY commit `d7c3fbc02` (the ball-arm fix below —
> the canonical drain source is now `d7c3fbc02`, not `f1480dc0`). The g82d seal
> above ruled "g82's drain survives" on the STREAM axis alone (g82
> drains on `UpdateProjectiles`, the closer stream); it explicitly
> deferred the draw-shape audit ("Both ports still draw g82's
> triplet-vs-pair shape through their helpers"). That premise is
> FALSE — d4's drain does NOT draw the triplet: it hand-builds the
> `ItemDrop` with the exact two integer draws. The shape axis decides
> the emit, and it favours d4.

**The C# re-read (verbatim, this pass):**

* The ladder's eight calls (`Projectile.cs:77576/:77580/:77587/
  :77591/:77598/:77602/:77609/:77613`) all pass `prefix: 0` and NO
  velocity argument; the beach-ball return (`:77622`) passes
  `prefix: 0` AND `Vector2.Zero`.
* `Item.NewItem` (`Item.cs:49727-49751`): `item.Prefix(0)` returns
  before ANY draw (`:511-513` — the `prefixWeWant == 0` head, ahead of
  even `CanHavePrefixes`); the `velocity.HasValue` arm (`:49738-49741`)
  fires ONLY for the ball (sets (0,0), draws nothing); none of
  985/3005/3079/3080/965/2996/3077/3078/859 is in `ItemNoGravity`
  (`ItemID.cs:1320` re-read: the set is 520/521/575/549/548/547/3453-
  3459/3580/3581/4143), so the coil kinds take the scatter pair
  `X = Next(-30,31)*0.1f`, `Y = Next(-40,-15)*0.1f` (`:49749-49750`) —
  exactly TWO integer draws, no rotation, no prefix roll.
* The draws run inside `Kill()` under the `UpdateProjectiles` swap
  (`Main.cs:17775-17778`; `UpdateItems` is the NEXT using-block
  `:17779-17782`).

**The port re-read (both tips):**

* d4 `world_sim/projectiles.rs:636-667` (tip `d7c3fbc02`, the
  SEAL-VERIFY fix; the pre-fix drain sat at `:633-658` at `f1480dc0`):
  `next_range(-30,31) as f32 * 0.1` / `next_range(-40,-15) as f32 *
  0.1` on `UpdateItems`, `prefix: 0`, rotation 0, the 14×14 box
  centred on the death centre — the EXACT coil emit, with the stream
  deferral (`UpdateProjectiles` → `UpdateItems`, the bobber overflow's
  recorded deviation) as its only departure. The ball sub-delta this
  receipt first recorded as uncommitted is now LANDED: the queue is
  the 5-tuple `(u16, i32, f32, f32, bool)` (`projectile.rs:14756`, the
  `zero_velocity` flag — coil `false` at `:20445`, ball `true` at
  `:20457`), the drain draws the pair ONLY when false (`:640-648`) —
  the ball's `Vector2.Zero` (`Projectile.cs:77622`) takes the
  `velocity.HasValue` arm (`Item.cs:49738-49741`), sets (0,0) and
  draws NOTHING; the ball witness now pins (0,0) + every stream
  pristine. The canonical drain source is `d7c3fbc02`.
* g82 `world_sim/projectiles.rs:607-618` → `world_sim/items.rs:757-
  785` `spawn_item_drop_px`: THREE `next_float` draws + a
  `roll_prefix(kind, -1)` natural roll + `(r1*2-1)*1.4` /
  `-2.2-r2*1.2` velocity + `r3*TAU` rotation, on `UpdateProjectiles`.
  For these non-prefixable kinds the roll returns 0 without drawing
  (`prefix.rs:514-517` — the `rollable_prefixes` None gate), so the
  live defect is THREE float draws vs TWO integer draws, the wrong
  velocity distribution (±1.4 uniform vs −3.0..3.0 tenths; −2.2..−3.4
  vs −4.0..−1.6 tenths) and a rotation draw the C# never makes — the
  tile-loot shape is wrong for a `RequestNewItem` refund regardless
  of the prefix gate outcome.

**Verdict: d4's death-hook queue + drain CANONICAL** (the d4d
FINAL-FINAL ruling ADOPTED; the g82d seal's delta 2 REVERSED on the
shape axis, its stream note kept as the recorded deviation). The
machine verdict is untouched: g82 `coil_step` stays CANONICAL (the
friendly pin + slot-direction write + integer anchor + PVP pair).

**The g82-side drop-hunks (g82 tip `6e5f6b74`, grep-pinned this
pass — the merge replaces these with d4's queue/drain):**

| # | file | hunk | what |
| --- | --- | --- | --- |
| 1 | `crates/tr-app/src/world_sim/projectiles.rs` | `:600-618` | the `drain_coil_drops` → `spawn_item_drop_px` drain + its comment (the tile-loot shape — wrong draws/velocity/rotation for `RequestNewItem`); d4's `:620-667` drain at `d7c3fbc02` replaces it |
| 2 | `crates/tr-app/src/projectile.rs` | `:14548` + `:14948` + `:21211` + `:21215-21222` | the `pending_coil_drops` field/init/push + `drain_coil_drops`; d4's `pending_death_drops` replaces them — the 5-tuple `(u16, i32, f32, f32, bool)` with `zero_velocity` at `d7c3fbc02` (`projectile.rs:14756`; g82's coil push repoints with `false` — the coil ladder never passes a velocity), `coil_kill`'s push repoints |
| 3 | `crates/tr-app/src/projectile.rs` | `:25816-25857` (`w4_g82_the_kill_hook_queues_the_cue_and_the_refund`) | reads `drain_coil_drops` at `:25828/:25837/:25845/:25856` — repoint to `drain_death_drops` (assert the 5-tuple with `false` at `d7c3fbc02`) or retire in favour of d4's `w4_d4_retire_the_refund_gates_and_the_dig_cue` (`:27128-27194` at `d7c3fbc02`, the local/foreign/noDropItem/foreign-ball arms — g82's witness lacks the noDropItem arm) + the refund-queue witness (`:27006-27089` at `d7c3fbc02`, the ai1 ladder arms, the exact-tuple assert carries `false`) |
| 4 | `crates/tr-app/src/projectile_rope_coil.rs` | `:45-49` (module doc) + `projectile.rs:21187-21194/:21215-21219` (the `coil_kill`/drain doc comments) | the "UpdateProjectiles-stream spawn" prose flips to d4's drain (the recorded `UpdateItems` deviation) |

(g82's `coil_kill` cue + gate LOGIC stays — behaviourally identical
to d4's retire-fixed hook; only the queue name + drain + doc prose
change. The d4d FINAL-FINAL's own g82 drop table listed rows 1/2 at
tip `187557f8e`; rows 3/4 are this receipt's completion — the seal
commit `6e5f6b74` touched docs only, so the code lines are unmoved.)

**The one-branch merge instruction** is the d4d FINAL-FINAL's five
ordered steps (d4 `26e-rope-coil-keep-one.md`, FINAL-FINAL at
`f1480dc0` + the SEAL-VERIFY corrections at `d7c3fbc02`), with
this receipt's correction folded in: step 3 routes g82's refund
through d4's `pending_death_drops` + the conditional
two-integer-draw drain (the `zero_velocity` 5-tuples at `d7c3fbc02`:
coil `false` → the scatter pair, ball `true` → (0,0) draw-free;
KEEP d4's whole hook block — rows 7/8 above corrected — not just
the ball half). The beach-ball 859 arm + its four witnesses stay
d4-only, whole.

**The g82-side ball arm: NONE (receipt-only).** g82's lane never
ported the `:77620-77623` ball return — re-confirmed this pass: zero
`beach_ball|BEACH_BALL` hits across g82's `crates/` (the 155 machine,
the 859 item row, the return arm and all four ball witnesses are
d4-only). The ball-arm defect (`Vector2.Zero` → zero scatter draws)
has NO g82-side code counterpart — nothing to fix on this branch;
the merge inherits the fixed ball arm wholesale from `d7c3fbc02`.

## Cross-receipt: the d4d retire lane reads THIS reversal

The d4d (d4-retire) lane's keep-one input is this doc's FINAL verdict,
not d4's superseded `26e-rope-coil-keep-one.md` (whose KEEP-d4 ruling
was premised on g82 being unwired at `de797f45`). Receipt for that
lane: retire d4's `ai_rope_coil` + named-const rows + witnesses per
the UPDATED drop-hunks table above (rows 1-10, `cafec7222` lines,
the g82e corrections folded in);
keep d4's wind-32 + styles 31-40-less-35 + the item-row verifies +
the WHOLE retire-fixed death-hook block + its queue/drain (the
canonical emit for the coil ladder AND the beach ball, rows 7/8).
If d4d's retire
already ran against `4fa74b3c7` lines, re-derive against `cafec7222`
— the union push shifted the dispatch arm (+20 lines) and added the
extras module/wiring this table now covers. No code on THIS branch
changes for the retire: g82's lane is self-contained (own module,
own consts, own queue names). **CLOSED (g82e)**: the d4d retire ran
and pushed (`954824d0` gates/cue fix → `ec3c6a88` band table →
`f1480dc0` FINAL-FINAL re-adjudication); its machine verdict matched
this doc, its drain verdict reversed this doc's seal delta 2 — the
g82e drain receipt above ADOPTS that reversal, so the two lanes now
read ONE verdict (g82 machine + d4 emit).

## Mutual receipt: 26e ↔ 26e-f at `d7c3fbc02` (the ROPE bible FINAL pin)

The two lanes' receipts now cross-read each other and agree on every
axis:

* **d4's `26e-rope-coil-keep-one.md`** (FINAL-FINAL `f1480dc0` +
  SEAL-VERIFY receipt `d7c3fbc02`) reads THIS doc's g82-machine
  verdict and confirms it in code (the friendly pin
  `projectile_rope_coil.rs:196`, the slot-direction write
  `projectile.rs:19866-19869`, the integer anchor `:247`, the PVP
  exemption `:19920` — all g82-tip pins). Its drain verdict (d4
  canonical) is the reversal THIS doc's g82e receipt adopted; its
  SEAL-VERIFY correction (the ball's `Vector2.Zero` → `zero_velocity`
  tuples, draw-free ball drain) is folded into the g82e receipt
  above — no residual disagreement.
* **THIS doc's g82e receipt** reads d4's SEAL-VERIFY commit
  `d7c3fbc02` diff in full (the queue field/drain-fn/pushes/witness
  touches in `projectile.rs`, the conditional drain + ball witness
  in `world_sim/projectiles.rs`, the module-doc prose in
  `projectile_rope_coil.rs`) and re-verified the fix against the C#
  verbatim: `Projectile.cs:77622` passes `Vector2.Zero` as the 7th
  arg (the 861 arm's `:77626` pattern), `Item.cs:49738-49741`'s
  `velocity.HasValue` arm sets (0,0) and SKIPS the `:49749-49750`
  scatter pair — the pre-fix drain's two-draw ball pin was the
  FINAL-FINAL "coil/ball pass no velocity" misread. The fix is
  correct; the drain source is pinned at `d7c3fbc02`.
* **ROPE bible FINAL pin: the canonical drain source is
  `d7c3fbc02`** (`origin/task/W4-d4-proj-ai-31-40`) — the merge takes
  d4's `pending_death_drops`/`drain_death_drops` 5-tuples + the
  conditional drain + the whole retire-fixed hook + all queue/drain/
  ball witnesses FROM THAT COMMIT, not from `f1480dc0` (whose ball
  arm still drew the pair). The machine source stays g82
  (`6e5f6b74` + this receipt). The bible below carries the same pin.

## f3-pulley compose (origin/task/W4-f3-dust-player-3 `0b11548f`)

f3's pulley machine (`find_pulley`/`attach_pulley`/
`can_move_forward_on_rope`/`update_pulley_direction`/`pulley_ride`,
`Player.cs:19644-19749`/`:24247-24266`/`:26116-26493`) attaches to the
SAME `td::TILE_ROPE` set g82's placement writes — the attach gate is
`render::ropes::is_rope` (`:19652`) + the raw `tileRope` neighbour probe
(`:19665`), never `IsVine` (the g83 disjoint pin). The end-to-end chain:

1. **PLACE** — the thrown coil (985 → proj 171) contacts, `coil_step`
   writes 213 down the column (g82, LIVE).
2. **ATTACH** — up/down held over the placed cell: `find_pulley`'s
   `is_rope` gate passes (a TILE_ROPE kind early-returns true), the
   lone-cell arm snaps `row*16+22` when no rope neighbour sits above/
   below (f3, LIVE on its branch).
3. **RIDE** — the climb/descend machine steps the column; the rope kinds
   are non-solid so the hang box never collides (f3 `0b11548f` verified).
4. **COIL-REFUND** — the coil's own death already paid `10-ai1` rope
   items back (or the coil item at ai1==0) through `coil_kill` →
   the death-drop queue → the two-integer-draw drain (g82's hook
   shape LIVE; the merged emit is d4's drain per the g82e receipt) — the ride
   consumes nothing, the refund is placement-count-keyed only.

Witness: `w4_g82_f3_compose_the_placed_column_attaches_and_refunds`
(g82 `e23c69da7`) — LEG1/LEG4 run the live g82 code; LEG2/LEG3 pin the
EXACT predicates f3's machine reads (the f3 player fns live on the f3
branch; the merged tree runs the full chain). No seam conflict: f3
touches `player.rs`, g82 touches `projectile*`/`editing.rs`/`assets.rs` —
disjoint files, both merge clean.

## Cut-tile TileKill drops — the POOL-WIDE gap (census + task seed)

The g82 dispatch discards `coil_step`'s returned `TileKill` records
(`projectile.rs:17632` `let _ =`). This is NOT coil-specific — **every
projectile-pool `kill_tile` call site drops the record** (census at g82
tip, `crates/tr-app/src/projectile.rs`):

| site | fn | C# source |
| --- | --- | --- |
| `:7598` | `ai_ice_block` | the 900-tick owner-block mine (`:25757-25760`) |
| `:12786` | `ai_dungeon_debris` | the debris landing carve |
| `:20393` | `debris_death_crack` | the cracked-brick roll (`:77376-77378`) |
| `:20472` | `celeb2_death` | the 3×3 celebration carve |
| `:22064`/`:22109` | `explode_bomb` | `Kill_ExplodeTiles` (`:77265-77365`) |
| `:22256` | `explode_rocket` | `ExplodeTiles` (`:78707-78756`) |
| `:17632` (dispatch) | `coil_step` | the placement walk's cut clears (`:27265-27270`) |

(d4's arm dropped them inline too — `let _ =` at its `:11502` — so the
gap never distinguished the two ports.)

The C# `WorldGen.KillTile` spawns the per-cell + cascade drops via
`Item.NewItem` inside the call; the port's `tr_core::edit::kill_tile`
RUNS the cascade but returns the drops as data (`TileKill.object_drops/
events/herb_events/pile_events/tree_events` + the frame-roll requests).
The player-mining path resolves them (`world_sim/editing.rs:5887`
`break_drop_events`, called at `:5668` under the `UpdatePlayers` rng
stream); **the projectile pool has no equivalent drain** — the cleared
plants/herbs/pots vanish silently.

**TASK SEED (for the Lead's catalog): "projectile-loot seam" — a
`TileKill` drain for the projectile pool.** Scope: the seven census
sites above. Shape: the pool queues the records (the `pending_coil_drops`
precedent), `world_sim/projectiles.rs` drains them at the pass tail
through `break_drop_events` under the `UpdateProjectiles` rng stream
(the C# `Kill()`/AI tile writes run under the `Main.cs:17775` swap —
the stream key needs the per-site C# audit: explosion kills run inside
`Kill()` = UpdateProjectiles, the ice-block mine inside AI = same).
Blockers: `break_drop_events` takes the mining ctx (staff bonus/axe
power/HitCell) — the projectile sites pass defaults (no player tool);
the palm/mushroom frame rolls need the genRand resolution the editing
path already does. NOT a coil-only fix; do not wire it inside
`coil_step`.

## ROPE-FAMILY merger bible (one paragraph, the lane's whole truth — FINAL, g82e drain receipt folded in)

The rope family merges as: **g82 tip (`6e5f6b74` + this receipt) is
the CANONICAL coil lane MACHINE** — keep `projectile_rope_coil.rs`
whole (machine `coil_step` + rows reference + chain preview +
`coil_death_drop` + `check_rope_usability`), the `35 =>` dispatch,
the four literal `data.rs` rows (`:3979-3980`), the `coil_kill`
death-hook cue + gate shape (the `:77267` owner + `:77367`
noDropItem gates KEPT, the cue ungated), the usability call in
`use_tile_item`, the 7 chain sheets + `draw_with_hand` emit, the
friendly pin + PVP exemption + style-35 direction write +
`framing_rope`; **the refund EMIT is d4's** — replace g82's
`pending_coil_drops`/`drain_coil_drops`/`spawn_item_drop_px` drain
with d4's `pending_death_drops`/`drain_death_drops`/the
two-integer-draw `UpdateItems` drain at `d7c3fbc02` (the exact
`Item.NewItem` pair, `Item.cs:49749-49750`, `prefix: 0` draws
nothing, the ball's `Vector2.Zero` → (0,0) zero draws via the
`zero_velocity` flag; the stream
deferral recorded; the g82-side drop-hunks in the drain receipt
above — g82's tile-loot shape draws the wrong velocity/count/
rotation), keep d4's WHOLE retire-fixed hook block (the coil ladder
+ the beach-ball 859 arm + its queue/drain + all its witnesses,
rows 7/8 corrected); **drop d4's machine half** (`ai_rope_coil`,
the `35 =>` arm, the 4 consts + named row arm, the four machine
witnesses at `:26902-27004`/`:27196-27221` at `d7c3fbc02`) **and d4's union-extras
duplicates** (its `projectile_rope_coil.rs` copies, its private
usability fn + call — the UPDATED ten-row table above, `cafec7222`
lines with the g82e corrections; the dedupe is trivial because the
pure fns are byte-identical copies of g82 `67c9a740`); **keep from
d4** the wind-list **32** (merged list `2|8|24|28|29|32|35` — d4's
beach-ball machine is its own), the beach-ball refund half + its
four witnesses (d4-only, whole), the 859/1156/190 row verifies,
styles 31-40 less 35, and the death-centre timing deviation
paragraph (pool-wide, both ports share it); **f3-pulley is
disjoint** (player.rs vs projectile*/editing.rs/assets.rs — the
place→attach→ride→coil-refund chain composes on the merged tree,
witness `w4_g82_f3_compose_…` pins f3's exact predicates at
`0b11548f`); **g83 vines stay out** (the pulley gate is
`is_rope`+`tileRope`, never `IsVine`); salvage d4's four-kind
placement loop + is_coil gate rows + NaN-quirk witness onto g82's
module (rename `w4_d4_union_` → `w4_g82_`); the pool-wide cut-tile
`TileKill` gap (7 census sites) is **R-26-13's seed**, NOT a merge
blocker and NOT a coil-only fix; row-dedupe 171/475/505/506 keeps
g82's literal arm (term-identical both-ported); the merge order is
the d4d FINAL-FINAL's five steps (d4 `26e-rope-coil-keep-one.md`,
FINAL-FINAL `f1480dc0` + the SEAL-VERIFY drain-source pin
`d7c3fbc02`); after the merge run the deferred gate `cargo test
--workspace -- --skip migrates_version_2_kinds_to_item_ids` — every
lane witness was WRITTEN NOT RUN under the code-only constraint.

## Witnesses (this final phase)

* `w4_g82_the_null_abort_and_the_kill_tick_still_integrate` (`c2b7507a0`)
  — the two faithfulness fixes: the kill tick's adjusted+raw double step,
  the null abort's raw integrate + no-kill.
* `w4_g82_f3_compose_the_placed_column_attaches_and_refunds` (`e23c69da7`)
  — the four-leg compose chain above.

Both WRITTEN NOT RUN (code-only). Running total for the lane: 29
witnesses (17 port + 10 wiring + 2 final). Verification: the C# arm
re-read verbatim (`decompiled/Terraria/Projectile.cs:27189-27291` +
`:17010-17030` + `:17281-17288` + `:20435-20446` + `:788`), d4 tip
`4fa74b3c7` read in full for the arm/rows/witnesses/exemption-list/
wind-list, f3 tip `0b11548f` read for the pulley predicates, every
line cite in the drop-hunk table grep-verified. No
cargo/rustc/tests/graphify invoked; the deferred gate stays the merge
pass's `cargo test --workspace -- --skip migrates_version_2_kinds_to_item_ids`.

**SEAL-pass verification (this commit):** d4 tip `cafec7222` read in
FULL (`projectile_rope_coil.rs` all 331 lines; `projectile.rs` hook
`:20389-20419` + field/init/drain + the machine/dispatch/witness
spans; `world_sim/projectiles.rs` drain `:620-658` + the 2-witness
module `:775-869`; `world_sim/editing.rs` fn `:7193-7265` + call
`:7337` + 3 witnesses; ANALYSIS `:39367-39480`); the mechanical
normalized byte-compare (python, comments/whitespace stripped,
brace-balanced extraction) over all 7 shared fns/struct + 3 shared
witness bodies; the C# gate chain re-read (`Projectile.cs:70642-70687`
flag2, `:77258-77270` Kill_ExplodeTiles head, `:77367`/`:77628`
noDropItem block bounds, `:77572-77623` ladder + ball;
`Main.cs:17775-17782` swap order; `Item.cs:49726-49766` NewItem
draws/ownership). Every UPDATED table line grep-pinned at `cafec7222`.
No cargo/rustc/tests/graphify invoked (code-only seal).

**G82e DRAIN-RECEIPT verification (this commit):** the d4d FINAL-FINAL
(d4 tip `f1480dc0b`, `26e-rope-coil-keep-one.md:167-362`) read in full
and its drain diff RE-VERIFIED against the C# verbatim:
`Projectile.cs:77565-77629` (the eight ladder calls' `prefix: 0` + no
velocity, the ball's `Vector2.Zero`), `Item.cs:49700-49769` (the
NewItem spawn body: `Prefix(prefix)` at `:49730`, the three velocity
arms `:49738-49751`) + `Item.cs:490-534` (`Prefix(0)` returns at
`:511-513` before any draw), `ItemID.cs:1320` (the ItemNoGravity set —
none of the nine kinds on it), `Main.cs:17765-17789` (the swap order).
Both ports' drains read at their tips: d4
`world_sim/projectiles.rs:620-667` + the 5-tuple `zero_velocity` queue
(`projectile.rs:14756/:15171/:15277`) + the retire-fixed hook
(`:20396-20460`, coil push `:20445` `false` / ball push `:20457`
`true`) + the retire witness (`:27128-27194`) + the
refund-queue/ball witnesses (`:27006-27089`/`:27091-27126`) + the
drain-contract witnesses (`world_sim/projectiles.rs:812-856`/
`:858-888`, the ball pin (0,0) + every stream pristine) — all
`git show d7c3fbc02:` pinned (the SEAL-VERIFY commit landed the ball
`Vector2.Zero` fix this receipt first saw uncommitted; the pre-fix
4-tuple spans at `f1480dc0` — drain `:633-658`, queue
`:14751/:15166/:15270`, hook `:20389-20446` — were the first-pass
pins); g82 `world_sim/projectiles.rs:600-618` +
`world_sim/items.rs:749-785` (`spawn_item_drop_px`'s triplet +
`roll_prefix(kind,-1)`) + `prefix.rs:509-520` (the None-gate no-draw
return) + the `coil_kill` hook (`projectile.rs:21195-21213`) + its
witness (`:25816-25857`). The g82-side drop-hunks table's every line
grep-pinned at `6e5f6b74` (docs-only seal commit — code unmoved from
`187557f8e`). Verdict: **g82 `coil_step` CANONICAL machine (unchanged),
d4 death-hook queue + drain CANONICAL emit (the seal's delta 2
reversed on the shape axis)**; rows 3/5/7/8 of the drop table
corrected, the ROPE-FAMILY bible FINAL. No cargo/rustc/tests/
rustfmt/graphify invoked (code-only receipt); the deferred gate stays
the merge pass's `cargo test --workspace -- --skip
migrates_version_2_kinds_to_item_ids`.
