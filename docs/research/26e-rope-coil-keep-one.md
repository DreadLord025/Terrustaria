# 26e — Rope-coil DUPLICATE keep-one (W4-d4 DOC-SWEEP, 2026-11-21)

> **STATUS: the KEEP-d4-machine verdict below is SUPERSEDED — see
> "FINAL-FINAL re-adjudication" at the foot of this doc (RETIRE phase):
> g82 `coil_step` is the canonical machine, d4's death-hook queue +
> `UpdateItems` drain the canonical refund emit; the ONE-branch merge
> instruction + the exact both-side drop-hunks are tabled there.**

> Compared `origin/task/W4-g82-rope-coil-draw` (tip `de797f45`, code
> `67c9a740`) against this branch's landed `ai_rope_coil`
> (`crates/tr-app/src/projectile.rs`, commit `548a9190`) — arm-by-arm over
> `Projectile.cs:27189-27291` (re-read verbatim this sweep). Doc-only:
> no code changed, no cargo/tests invoked.

## The overlap

Both tasks ported the SAME C# arm (`AI()`'s `else if (this.aiStyle == 35)`,
the four coils 171/475/505/506):

| piece | W4-d4 (LANDED, wired) | W4-g82 (`67c9a740`, UNWIRED code-only) |
| --- | --- | --- |
| the aiStyle-35 machine | `projectile.rs::ai_rope_coil` — dispatch `35 =>` live (`:17798`) | `projectile_rope_coil.rs::coil_step` — module never declared in `main.rs` |
| the four `SetDefaults` rows | `projectile/data.rs` `ROPE_COIL \| VINE_ROPE_COIL \| SILK_ROPE_COIL \| WEB_ROPE_COIL` (14×14/35/−1/400) | `coil_proj_row` — a second copy of the same tuple |
| the rope-kind ladder | inline `match self.kind` (475→353, 506→366, 505→365, _→213) | `coil_rope_kind` — same |
| `WorldGen.IsRope` | `render::ropes::is_rope` (the verbatim `:70081-70104` port, S1-G-1) | `is_rope_cell` — g82's own inline copy (the d56 inlined-table convention, `WorldGen.cs:70026-70104` incl. the no-top-end degeneration) |
| wind-list 35 | landed (`uses_wind_physics`, `:13896`) | n/a (wiring deferred) |

**KEEP-ONE VERDICT (SUPERSEDED — see FINAL-FINAL at the foot): keep W4-d4's `ai_rope_coil` + the four data rows.**
The g82 `coil_step`/`coil_proj_row`/`coil_rope_kind`/`is_rope_cell` are the
duplicate half — the merge must NOT land them as a second dispatch arm (the
`unreachable_patterns` deny on `update`'s match, S1-D-4c, would reject a
second `35 =>` anyway). The g82 module's UNWIRED status (its header: the
dispatch/data.rs/assets.rs/main.rs sites "ALL contested by in-flight
siblings") is now resolved by d4's landing: those sites are TAKEN.

## Arm-by-arm comparison (`:27189-27291`)

Both ports are faithful to the same C# body; the deltas:

1. **Head** (`:27191-27201`): identical in both — ai[0]++, >30 → +0.2 vy,
   ×0.985 vx, 14 vy clamp; spin (|vx|+|vy|)·direction·0.02.
   *d4 reads `direction` as the velocity-sign stand-in computed in the arm;
   g82 reads `p.direction` (the slot field). Neither updates the slot's
   `direction` post-move for style 35 — `AutomaticallyChangesDirection`
   (`:17281-17288`) is TRUE for 35 and the pool's post-movement write only
   runs for style 149 (`projectile.rs:20093-20105`), so both carry the same
   one-tick-lag/frozen-direction deviation. Equivalent; recorded in d4's
   ANALYSIS section.*
2. **Owner gate** (`:27202-27205`): identical — foreign coils free-move
   (`pos += vel`, the `HandleMovement` stand-in) and never place.
3. **Collision probe** (`:27206-27222`): identical — `tile_collision
   (fallThrough: true, fall2: true)`, the `!= velocity` contact test, the
   centre-plus-velocity `(int)` truncation BEFORE `/16`, the bottomSlope
   snap to `(tx*16+8, ty*16+24)`.
4. **Walk anchor** (`:27240-27242`): identical for the 14×14 rows — d4
   `self.w / 2.0` (f32), g82 `p.w as i32 / 2` (the C# integer division);
   14/2 = 7 either way. The `pos += adjusted` step is in both.
5. **Null-tile abort** (`:27244-27247`): both return WITHOUT Kill. **d4
   deviation (pre-existing, unchanged this sweep): d4's abort path adds
   `pos += vel` (the HandleMovement stand-in); g82's returns with no
   integrate. The C# returns from AI only — `HandleMovement` still runs
   (`:17015`), so d4's extra integrate is the faithful one.**
6. **IsRope skip** (`:27248-27250`): identical walk-down; d4 folds to
   `render::ropes::is_rope`, g82 to its inline twin. Both verbatim.
7. **Placement walk** (`:27251-27289`): identical — the 314/platform
   alternator (no budget spend), the tileCut/165 clear, the place+ai[1]++
   (alternator reset), the solid-stop `budget = 0`, the 10-link budget.
   **Delta: g82 collects the `KillTile` results into `Vec<TileKill>` for
   the editing resolver; d4 drops them (`let _ =`) — the pool-wide
   convention (every `tr_core::edit::kill_tile` call site in
   `projectile.rs` drops the record: the ice block `:7597`, the boulder
   crater `:20614`, etc.). The cut-tile drops are a shared pool-level gap,
   not a coil-specific one — see the union recommendation.**
8. **Kill** (`:27290`): d4 `self.kill()` + the dead-tick integrate (the
   C# `UpdatePosition` moves the dead slot too, `:20445` — the
   `ai_trap_spear` precedent); g82 sets the kill fields with no dead-tick
   move. d4's is the faithful shape.
9. **`NetMessage.SendData(17)` pairs** (`:27269`/`:27277`): skipped in
   both (single-player; the MP relay's).

## The g82 EXTRAS d4 does not carry (union recommendation)

The g82 module ports four pieces OUTSIDE the `:27189-27291` arm that d4
never claimed. They are NOT duplicates — the merge should KEEP them
(their own module, minus the four duplicate fns above):

1. **The trailing-chain placement preview** — `Main.DrawProjDirect`
   `Main.cs:28499-28669` (`coil_chain_sheets`/`coil_chain_segments`/
   `draw_coil_chain`): the coil items have NO `createTile`, so the chain
   of rope links IS the placement preview (the 17/30-tick count/sag
   ramps, the 5.6-px normalize with the manhattan-<1 shrink quirk incl.
   the kept 0/0 NaN pin, reverse draw order, the Chain16/38/39/
   Chains_4..7 sheet parity, per-link light + 0.8 scale). Draw layer —
   d4's slot arms carry no draw; the render wiring is the draw task's.
2. **The `Kill()` refund ladder** — `Projectile.cs:77572-77615`
   (`coil_death_drop`): ai[1]==0 → one coil item back (985/3005/3079/
   3080); ai[1]<10 → `10−ai[1]` rope items (965/2996/3077/3078). d4's
   `ai_rope_coil` kills with NO refund — the refund is a real gap. The
   union: keep `coil_death_drop` as the pure ladder and drain it at the
   coil's death (the `drain_catches` seam, `world_sim/projectiles.rs:589`
   — inventory-first, overflow drop), keyed on the coil kinds.
3. **`PlaceThing_Tiles_CheckRopeUsability`** — `Player.cs:41506-41545`
   (`check_rope_usability`): the hand-placement down-column walk (the
   5-consecutive-platform cap, the 214 chain lava exemption, the
   header-bit lava test). d4 never touched the hand-placement route;
   keep for the `use_tile_item` rope seam.
4. **The rope FRAMING arm** — `render/ropes.rs::rope_frame_arm`
   (`WorldGen.cs:81894-81931`, g82 phase 1 `bfbf661c`): unrelated to the
   coil AI, no overlap with d4 — keep (its `framing.rs` seam was
   contested by d109/d110, not by d4).

**Union recommendation (for the merge):** keep d4's `ai_rope_coil` +
rows + wind-list as the single style-35 machine; from g82 keep the chain
preview, the refund ladder (wire it to the coil death drain — the one
behavioural gap this comparison found), the usability walk and the
framing arm; DROP g82's `coil_step`/`coil_proj_row`/`coil_rope_kind`/
`is_rope_cell` (the duplicate half). The cut-tile `TileKill` drop gap is
pool-wide (every projectile kill_tile site drops the record) — a
follow-up for the projectile-loot seam, not a coil-only fix.

## Witnesses

Code-only sweep: the C# arm re-read (`Projectile.cs:27189-27291` via the
decompiled junction), both Rust arms read in full (d4
`projectile.rs:11413-11522` + its four witnesses `:26794-26919`; g82
`projectile_rope_coil.rs` at `origin/task/W4-g82-rope-coil-draw`, 757
lines incl. its 11 witnesses). No cargo/tests/graphify invoked; no code
changed by this item.

## Union landing status (2026-11-21, same branch, code-only)

The union recommendation above is now LANDED on this branch (the merge
keeps d4's machine untouched; the g82 duplicate half stays dropped):

1. **Refund ladder — WIRED** (`bb2f8d2c`): g82's `coil_death_drop` re-homed
   in `crates/tr-app/src/projectile_rope_coil.rs`, queued at
   `projectile.rs`'s once-per-death hook (`pending_death_drops`, behind
   `!died_out_of_world` — the `:16696` guard never calls `Kill()`) and
   drained by `world_sim/projectiles.rs` as `Item.NewItem` WORLD drops on
   the `UpdateItems` stream. NOT the `drain_catches` inventory hand-off:
   the C# `RequestNewItem` (`Item.cs:49639-49646`) spawns a world item —
   the 26e recommendation's "inventory-first" reading is corrected here
   (that shape is the bobber's `GiveItemToPlayer`, a different C# path).
   The beach-ball 859 return (`:77620-77623`) rides the same queue.
2. **Usability walk — WIRED** (`1cae02fd`): `check_rope_usability` in
   `world_sim/editing.rs`, called at `use_tile_item`'s head (the `:39726`
   site — before the `:39727` tile re-read and the `:39743` lava gate).
3. **Chain preview — geometry landed, EMIT deferred to g82b** (`bb2f8d2c`):
   `is_coil`/`coil_chain_sheets`/`coil_chain_segments`/`draw_coil_chain`
   re-homed in the same module. The C# (`Main.cs:28499-28669`) was re-read
   verbatim this round: the three per-kind bodies are term-for-term
   identical bar the sheet pick (171 Chain16; 475 Chain38/39; 505/506
   `Chains[4|6 + drawIndex%2]`), g82's port matches every constant and the
   write order (count ramp `:28505-28509`, sag ramp `:28510-28517`, the
   5.6 normalize + manhattan-<1 shrink `:28522-28527`, record-before-
   advance `:28531-28534`, reset+sag `:28535-28540`, reverse emit
   `:28542`/`:28593`/`:28654`). The emit half is draw-layer work — d4's
   slot arms carry no draw and `assets.rs`' sheet list is contested — so
   it stays `#[allow(dead_code)]` with the wiring contract in ANALYSIS
   "The rope-coil lane extras (W4-d4 union)": `draw_with_hand` gates on
   `is_coil`, per-link `Lighting.GetColor`, scale 0.8, half-sheet origin,
   the Chain16/38/39/Chains_4..7 sheets join `assets.rs` (the FishingLine
   precedent). The 0/0 NaN pin is kept and witness-pinned (vel = (0, 0.4)
   at ai[0] ≥ 30 cancels the first sag step exactly).
4. **Framing arm** — untouched (g82 phase 1's own lane, no d4 overlap).

## FINAL-FINAL re-adjudication (RETIRE phase, 2026-11-21, d4 tip `cafec722` → retire fixes)

> Input: g82 `origin/task/W4-g82-rope-coil-draw` tip `59139eb19` PLUS its
> four unpushed local commits `c2b7507a0` (the two integrate fixes) →
> `e23c69da7` (the f3 compose witness) → `38fd7dcf1` (26e-f) → `187557f8e`
> (the R-26-13 register seed) — read at `187557f8e`, the g82c-active tip.
> d4 side re-read at `cafec722` + this retire phase's fixes (`954824d0`:
> the refund gates + the Dig cue). The g82 26e-f verdict (KEEP-g82) was
> made against d4 tip `4fa74b3c7`, which carried NONE of the extras; d4
> then landed refund + usability + preview-geometry, so BOTH branches are
> now near-identical supersets. This pass re-runs the comparison per-fn
> with byte-level diffs (python fn-extraction + unified diff, comments
> stripped) instead of arm-by-arm prose.

### Per-fn byte-compare verdicts

| fn | d4 home | g82 home | byte verdict | canonical |
| --- | --- | --- | --- | --- |
| `is_coil` | `projectile_rope_coil.rs:56` | same module `:90` | **IDENTICAL** (124 bytes, both `matches!` over the four consts; d4's consts ride `crate::projectile`, g82's are module-local — same values 171/475/505/506) | either; g82's module-local consts win with the module |
| `coil_death_drop` | `:65` | `:426` | **IDENTICAL** (551 bytes — the same four-arm ladder, the same `placed == 0.0` / `< 10.0` gates, the same `(10.0 - placed) as i32`) | either |
| `coil_chain_sheets` | `:99` | `:310` | **IDENTICAL** (279 bytes — Chain16 / Chain38+39 / Chains_4+5 / Chains_6+7) | either |
| `coil_chain_segments` | `:126` | `:334` | **IDENTICAL** (1657 bytes — the 17/30 ramps, the 5.6 normalize + manhattan-<1 shrink, record-before-advance, the reverse emit, the draw-index parity; the 0/0 NaN quirk lives in BOTH) | either |
| `draw_coil_chain` | `:177` | `:382` | **IDENTICAL** (1259 bytes — per-link `Lighting.GetColor`, 0.8 scale, half-sheet origin, `snap_to_screen_pub`) | either; **g82 WIRES it** (`draw_with_hand:10286-10287` + the 7 packed sheets in `assets.rs`), d4 keeps it `#[allow(dead_code)]` for g82b |
| `check_rope_usability` | `world_sim/editing.rs:7207` | module `:474` | **code-IDENTICAL** (comments stripped; the same gate, the same 5-cap walk, the same 214 exemption, the same header-bit lava test, the same landing gate). Call sites identical in effect: both shadow `ty` at `use_tile_item`'s head after the registry resolve, before the lava gate | either; g82's module home wins (one module owns the lane) |
| the style-35 MACHINE | `projectile.rs::ai_rope_coil:11413-11522` | module `coil_step:180-292` (+ the `c2b7507a0` fixes) | **NOT identical — g82 is the strict superset** (the full diff below) | **g82 `coil_step`** |
| the four data rows | `data.rs:2330-2341` (named-const arm) | `data.rs:3979-3980` (two literal arms) | **term-IDENTICAL** (the same ten terms `(14.0, 14.0, 35, -1, 400, 0, false, false, false, RESET_STATE)`; the two C# arms `:2570-2578`/`:5496-5504` are term-identical) | g82's literal arms (the row-dedupe) |
| the death hook | `projectile.rs:20389-20446` (inline, retire-fixed) | `projectile.rs::coil_kill:21195-21213` + `pending_coil_drops` | **behaviourally IDENTICAL after `954824d0`** (both: the ungated ai1==0 `(0,0,trunc)` Dig cue, the `:77267` owner + `:77367` noDropItem gates on the refund, `!died_out_of_world`; d4 additionally carries the beach-ball 859 arm g82 lacks) | **d4's hook shape + queue** (the ball arm is d4-only; see the merge instruction) |
| the world-sim drain | `world_sim/projectiles.rs:633-658` | `:607-618` | **NOT identical — d4 is the faithful one** (below) | **d4's drain** |

### The machine diff (the only behavioural deltas left)

`ai_rope_coil` vs `coil_step` (post-`c2b7507a0`), comment-stripped diff:

1. **friendly pin** — g82 `p.friendly = false` every tick (the `:788`
   SetDefaults reset; the port Default carries true). d4 NEVER pins it:
   d4's coils ride `friendly = true` — wrong for the PVE pass AND (with
   d4's PVP proxy) would hurt the thrower if the exemption were absent.
   **g82 wins.**
2. **spin sign** — g82 reads `p.direction` (the slot field) AND lands the
   style-35 post-AI `AutomaticallyChangesDirection` write
   (`projectile.rs:19866-19869`, the `:17016-17026`/`:17281-17288` shape —
   35 is not excluded). d4 computes the velocity sign inline at spin time
   and never writes the slot (its ANALYSIS records the frozen-direction
   deviation). **g82 wins** (the exact C# one-tick-lag shape).
3. **walk anchor** — g82 `(p.w as i32 / 2)` = the C# INTEGER `width / 2`;
   d4 `self.w / 2.0` f32 division. Equal for 14, divergent for any odd
   future width. **g82 wins.**
4. **null-tile abort** — BOTH integrate now (g82 `c2b7507a0` took d4's
   delta 5). Tie.
5. **kill tick** — BOTH carry the dead-tick integrate (g82 `c2b7507a0`
   took d4's delta 8; d4 `self.kill()` + `pos += vel`). Tie —
   observationally identical (`kill()` = the three field writes).
6. **TileKill records** — g82 collects into `Vec<TileKill>`, the dispatch
   discards (`let _ =`, `:17632`); d4 drops inline (`let _ =`, `:11502`).
   Observationally identical — the pool-wide loot gap either way (the
   R-26-13 seed, g82 26e-f's 7-site census). Tie.
7. **IsRope probe** — g82's inline `is_rope_cell` (the d56 inlined-table
   convention, `WorldGen.cs:70026-70104` incl. the no-top-end degeneration)
   vs d4's fold to `render::ropes::is_rope`. Both verbatim; the render fn
   stays (S1-G-1's) either way. Tie.
8. **PVP exemption** — g82 adds `!is_coil(kind)` to the `damage_player`
   gate (`:19920`, the debris/slime precedent); d4's list has the flame
   jet but NOT the coils (`:20167-20177`) — with d4's friendly=true the
   coils never reach the proxy anyway, but the merged tree (g82's
   friendly=false pin) NEEDS the exemption or the coils hurt the thrower.
   **g82 wins** (the pair is inseparable).

### The drain diff (d4's emit contract wins)

* **d4** (`world_sim/projectiles.rs:633-658`): two INTEGER draws
  `Next(-30,31)*0.1` / `Next(-40,-15)*0.1` on the `UpdateItems` stream,
  `prefix: 0` (draws nothing), the 14×14 box centred on the death centre —
  the EXACT `Item.NewItem` velocity pair (`Item.cs:49749-49750`, re-read
  this pass: the `velocity.HasValue` arm `:49738-49741` is skipped — the
  COIL calls pass no velocity; the `ItemNoGravity` arm `:49742-49746`
  is skipped — none of 985/3005/3079/3080/965/2996/3077/3078/859 is on the
  set). **SEAL-VERIFY correction:** the BALL call DOES pass a velocity —
  `Vector2.Zero` (`:77622`, the 861 arm's `:77626` pattern) — so its
  `HasValue` arm sets (0,0) and draws NOTHING; the original drain drew
  the pair for the ball too (this doc's "the coil/ball calls pass no
  velocity" misread), fixed at the SEAL-VERIFY pass: the queue carries
  `zero_velocity`, the ball drains draw-free. The stream deviation (C#
  draws under `UpdateProjectiles` `Main.cs:17775`, the port defers to
  `UpdateItems` `:17779`) is recorded.
* **g82** (`:607-618`): `spawn_item_drop_px` — the TILE-LOOT spawn shape:
  three `next_float` draws + a `roll_prefix(kind, -1)` (a natural prefix
  roll the C# `prefix: 0` never makes) + the `(r1*2-1)*1.4` / `-2.2-r2*1.2`
  velocity + a rotation draw, on `UpdateProjectiles`. WRONG velocity
  distribution, WRONG draw count, a prefix the C# never rolls.
* **Verdict: d4's drain is canonical.** g82's `spawn_item_drop_px` call
  is the drop-hunk on the g82 side (the one place g82's wiring is LESS
  faithful than d4's).

### FINAL VERDICT + the ONE-branch merge instruction

**Canonical machine: g82 `coil_step`** (the 26e-f verdict STANDS — the
d4 union did not touch the machine, and the per-fn byte-compare confirms
g82's three faithfulness wins: the friendly pin, the slot-direction write,
the integer anchor). **Canonical refund emit: d4's death-hook queue +
`UpdateItems` two-integer-draw drain** (g82's `spawn_item_drop_px` drain is
the less-faithful half). The shared pure fns are byte-identical — keep ONE
copy, g82's module (it owns the machine).

**Merge into ONE branch (the merge pass's instruction, in order):**

1. Take g82's `projectile_rope_coil.rs` WHOLE (machine + rows-reference +
   probe + the five shared pure fns + usability + its witnesses).
2. Take g82's `35 =>` dispatch (`:17632`), the literal data rows
   (`:3979-3980`), the friendly pin, the PVP exemption (`:19920`), the
   style-35 direction write (`:19866-19869`), the chain-draw wiring
   (`:10286-10287` + the `assets.rs` sheets), the framing arm
   (`tr_core::framing_rope` + `render/ropes.rs`).
3. **Replace g82's `coil_kill` refund queue + `drain_coil_drops` +
   `spawn_item_drop_px` drain with d4's `pending_death_drops` /
   `drain_death_drops` / the two-integer-draw drain** — keep
   g82's `coil_kill` cue + gate shape (identical to d4's retire-fixed
   hook), but route the refund through d4's queue and ADD d4's beach-ball
   859 arm (g82 has none). The queue tuples carry `zero_velocity` (the
   SEAL-VERIFY fix): the coil ladder `false` (the scatter pair draws),
   the ball `true` (`Vector2.Zero`, draw-free). The `UpdateItems` stream
   deviation stays recorded (ANALYSIS).
4. Take d4's `check_rope_usability` call-site wiring as-is if g82's
   module-home version is kept (both identical); ONE copy only.
5. Wind list: the merged `uses_wind_physics` = d4's
   `2 | 8 | 24 | 28 | 29 | 32 | 35` (g82's list + 32 — the beach ball is
   d4's machine).

**Exact d4 drop-hunks (current line numbers, d4 tip `ec3c6a88`):**

| # | file | hunk | what |
| --- | --- | --- | --- |
| 1 | `crates/tr-app/src/projectile.rs` | `:11394-11522` | the `ai_rope_coil` fn + its doc comment |
| 2 | `crates/tr-app/src/projectile.rs` | `:17812-17818` | the `35 => ai_rope_coil` dispatch arm + comment (a second `35 =>` trips the `unreachable_patterns` deny, S1-D-4c) |
| 3 | `crates/tr-app/src/projectile/data.rs` | `:604-607` | the four consts — **NOT a free drop**: `projectile_rope_coil.rs:57` imports them and the surviving witnesses use them; the merge either keeps the consts or repoints the module to g82's module-local consts (g82's module already defines them — take g82's, drop d4's, repoint the `use` at `:57` and the witness imports) |
| 4 | `crates/tr-app/src/projectile/data.rs` | `:2326-2341` | the named-const row arm (g82's literal arms replace it) |
| 5 | `crates/tr-app/src/projectile.rs` | `:26888-26990` | the three machine witnesses (`the_rope_coil_rows_carry_style_35` / `…falls_after_30_ticks` / `…places_its_rope_and_dies`) — g82's module witnesses cover the same ground; **salvage**: the four-kind placement loop may port onto `coil_step` as a bonus witness (the 26e-f note) |
| 6 | `crates/tr-app/src/projectile.rs` | `:27184-27206` | `the_rope_coil_placement_is_owner_gated` (g82's `w4_g82_the_owner_gate` covers it) |

**d4 KEEPS (not rope-duplicates):** the death-hook block `:20396-20460`
(the retire-fixed gates + cue + the ball arm + the SEAL-VERIFY
`zero_velocity` flag — step 3 routes g82's refund through it),
`pending_death_drops`/`drain_death_drops`, the
`world_sim/projectiles.rs` drain (`:636`, the conditional scatter pair) +
its two stream-contract witnesses (`w4_d4_union_the_coil_expiry_drains_the_refund_as_a_world_drop`
/ `…the_beach_ball_return_drains_on_the_items_stream`), the retire witness
`w4_d4_retire_the_refund_gates_and_the_dig_cue` (`:27118-27179`), the
refund-queue witness `w4_d4_union_the_coil_death_queues_the_refund_ladder`
(`:26998-27075` — it drives `pool.update`, so the dispatch swap to
`coil_step` needs NO witness change; the spawn API is unchanged), the
ball-return witness
(`:27081-27111`), the usability witnesses (editing.rs), the module's four
geometry/ladder witnesses (byte-identical fns — keep ONE set; d4's NaN-pin
witness `w4_d4_union_the_cancelled_sag_keeps_the_nan_quirk` is d4-ONLY
coverage — g82's reverse-order witness deliberately avoids the degenerate
axis, so KEEP the d4 NaN witness), wind-32, everything else d4 landed
(styles 31-40 less 35, the DOC-SWEEP items).

**Exact g82 drop-hunks (g82 tip `187557f8e`):**

| # | file | hunk | what |
| --- | --- | --- | --- |
| 1 | `crates/tr-app/src/world_sim/projectiles.rs` | `:600-618` | the `drain_coil_drops` → `spawn_item_drop_px` drain (the tile-loot spawn shape — wrong velocity/draw-count/prefix for `RequestNewItem`; d4's drain replaces it) |
| 2 | `crates/tr-app/src/projectile.rs` | `:14548` + `:14948` + `:21211` + `:21215-21222` | `pending_coil_drops` field/init/push + `drain_coil_drops` (d4's `pending_death_drops` queue replaces them; `coil_kill`'s push at `:21211` repoints to `pending_death_drops`) |

(The g82 `coil_kill` cue + gates STAY — identical to d4's retire-fixed
hook; only the queue name + the drain change.)

### Beach-ball 859 return — the same-queue witness coverage

The 859 return is **d4-only** (g82 never ported `:77620-77623`). It rides
the SAME `pending_death_drops` queue as the coil ladder, behind the SAME
`:77267`/`:77367` gates (the retire fix — the C# type-155 arm sits inside
the same `!noDropItem` block, after the same owner refusal). Coverage:

* `w4_d4_union_the_beach_ball_death_returns_the_item`
  (`projectile.rs:27081-27111`) — the pool hook: the zero-velocity kill
  queues `(859, 1, 1622, 422.1)` (the post-integrate centre, the +0.1
  dry-gravity dead-tick move, the pool-wide hook timing deviation).
* `w4_d4_union_the_beach_ball_return_drains_on_the_items_stream`
  (`world_sim/projectiles.rs:852-868`) — the drain: one world drop
  `(859, 1)` at `(54, 54.1)`, velocity (0,0), ZERO draws on EVERY stream
  (the `Vector2.Zero` `HasValue` arm — the SEAL-VERIFY fix; the earlier
  two-integer-draw pin was the `:77622` misread).
* `w4_d4_retire_the_refund_gates_and_the_dig_cue`
  (`projectile.rs:27118-27179`, fourth arm) — the FOREIGN ball never
  returns (the `:77267` refusal precedes the whole block).
* `w4_d4_sweep_the_beach_ball_item_throws_the_155_and_consumes` +
  `w4_d4_verify_the_beach_ball_and_piranha_gun_item_rows` (editing.rs) —
  the item-use + row pins (the 859 → 155 spawn at speed 6, the consume).

All WRITTEN NOT RUN (code-only). The merge keeps every one (step 3 keeps
d4's queue + drain + the ball arm).

### Verification (this pass)

Per-fn byte-compare via python fn-extraction + unified diff (comments
stripped): the five shared pure fns IDENTICAL byte-for-byte;
`check_rope_usability` code-identical; the machine diff enumerated above
(8 deltas, g82 wins 4, ties 4); the drain diff read against
`Item.cs:49639-49769` verbatim (the `RequestNewItem` → `NewItem` chain,
the velocity arms, the ownership arms). C# re-reads:
`Projectile.cs:77255-77384` (the `Kill_ExplodeTiles` gates),
`:77550-77639` (the ladder + the ball + the 861 arm),
`:70642-70701` (the `Kill()` head), `:74595-74639` (the cue site),
`Main.cs:17768-17787` (the stream swaps). No cargo/tests/rustfmt/graphify
invoked; the deferred gate stays the merge pass's
`cargo test --workspace -- --skip migrates_version_2_kinds_to_item_ids`.

## SEAL-VERIFY receipt (2026-11-21, d4 tip `f1480dc0b` × g82 rope-seal `6e5f6b748`)

Both seal tips re-read CLEAN (`git status --porcelain` empty on both
worktrees): g82 `6e5f6b748` (the 26e-f SEALED receipt + the ROPE-FAMILY
bible, on top of `3173cc689`/`187557f8e`) and d4 `f1480dc0b` (this doc's
FINAL-FINAL). Every claim of the four-item seal re-verified against the
code + the C# source directly (not just the receipts):

1. **The machine verdict STANDS (g82 canonical).** g82's four wins
   grep-pinned at its tip: the friendly pin (`projectile_rope_coil.rs:196`
   `p.friendly = false`), the slot-direction read (`:207`) + the style-35
   post-AI write (`projectile.rs:19866-19869`), the integer walk anchor
   (`:247` `p.w as i32 / 2`), the PVP exemption (`projectile.rs:19920`
   `!is_coil`). d4's machine carries none of them (`ai_rope_coil` — no
   friendly write in its span). The two integrate fixes (`c2b7507a0`)
   present; the chain-draw emit LIVE (`projectile.rs:10286`).
2. **The drain verdict STANDS (d4 canonical) — with ONE correction, FIXED
   this pass.** g82's `spawn_item_drop_px` (`world_sim/items.rs:757-778`)
   is the tile-loot shape: three `next_float` draws + `roll_prefix(kind,
   -1)` + the `(r1*2-1)*1.4` / `-2.2-r2*1.2` pair — wrong count, wrong
   distribution, a prefix the C# `prefix: 0` never rolls. d4's drain
   matches `Item.cs:49749-49750` exactly FOR THE COILS. **The correction:**
   the ball call `:77622` passes `Vector2.Zero` (7th arg; the 861 arm
   `:77626` shows the same pattern) → `velocity.HasValue` TRUE
   (`Item.cs:49738-49741`) → velocity (0,0), ZERO scatter draws. The
   pre-fix drain drew the pair for the ball too and its witness pinned
   "exactly two draws" — both wrong. FIXED: `pending_death_drops` /
   `drain_death_drops` tuples carry `zero_velocity: bool` (coil `false`,
   ball `true`), the drain draws the pair only when `false`, the ball
   witness now pins (0,0) + EVERY stream pristine, the queue witness
   destructures the flag, the retire witness's exact-tuple assert carries
   `false`. The g82-seal's "g82 drain survives" paragraph is superseded
   on the SHAPE axis by this doc's FINAL-FINAL (its stream point stands —
   the `UpdateItems` deferral stays a recorded deviation); the g82-seal's
   ball-keep instruction (rows 7/8: never fold the ball onto
   `spawn_item_drop_px`) is honoured and strengthened — the ball is
   draw-free, not merely d4-shaped.
3. **The gates + cue verified.** d4's hook (`projectile.rs`, the
   `!died_out_of_world` block): the ungated ai1==0 `(0,0,trunc)` Dig cue
   ahead of the `owner == local_owner && !no_drop_item` refund gate —
   matches the C# chain re-read verbatim (`:70656-70661` flag2 true for
   35, `:77258-77261` the `Kill_ExplodeTiles` call, `:77267-77270` the
   owner refusal, `:77367` the `!noDropItem` head through `:77628`,
   `:74617-74620`/`:74681-74684` the int-overload cue). The drain order
   holds: `drain_proj_sounds` (`world_sim/projectiles.rs:460-473`, the
   id-0 `play_place_sound` facade) runs ahead of the death-drop drain
   (`:636`).
4. **31-40 band 10/10 ARMED re-pinned at the CURRENT tip** (the fix's
   field-doc edit shifted the dispatch +7): 40 `:17252` / 33 `:17432` /
   31 `:17575` / 34 `:17801` / 37 `:17811` / 38 `:17818` / 35 `:17825` /
   32 `:17832` / 39 `:17840` / 36 `:17920` — all grep-live, style 35
   RETIRE-PENDING the canonical flip (unchanged).
5. **Beach-ball 859 d4-only confirmed:** ZERO hits for
   `beach_ball|BEACH_BALL` in g82's `projectile.rs`; d4 carries the
   machine + the same-queue return + the four witnesses (the pool hook
   `w4_d4_union_the_beach_ball_death_returns_the_item` now `:27095`, the
   drain witness `world_sim/projectiles.rs:864` — the corrected draw-free
   pin, the retire witness's foreign-ball arm `:27133`, the item-use +
   row pins in editing.rs). The merge keeps all.
6. **The bible's disjoint pins hold:** wind list `2|8|24|28|29|32|35`
   (`uses_wind_physics`, unchanged); f3 `0b11548f` touches ONLY
   `player.rs` (git stat — no projectile*/editing.rs/assets.rs overlap);
   the `is_coil` gate no longer carries `#[allow(dead_code)]` (the cue
   consumes it, `projectile_rope_coil.rs:64`); the NaN witness
   `w4_d4_union_the_cancelled_sag_keeps_the_nan_quirk` still d4-only
   coverage (g82 pins no NaN witness).

Code changed this pass (the fix only): `projectile.rs` (the queue field
+ drain fn + the two pushes + 2 witness touches), `world_sim/
projectiles.rs` (the drain loop + the ball witness + the module doc),
`projectile_rope_coil.rs` (the module doc), ANALYSIS (the refund/stream/
witness paragraphs), this doc (the drain diff + step 3 + the ball
coverage row). The d4 drop-hunks table above was pinned at `ec3c6a88`;
the fix shifts everything below the queue field by +5 lines and the
hook by +10 — the merge re-greps by marker (`ai_rope_coil`, `35 =>`,
`the_rope_coil_`), not by the stale numbers. All witnesses WRITTEN NOT
RUN (code-only); the deferred gate stays the merge pass's
`cargo test --workspace -- --skip migrates_version_2_kinds_to_item_ids`.