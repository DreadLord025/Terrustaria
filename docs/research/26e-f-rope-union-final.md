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

## FINAL VERDICT: g82's `coil_step` is the CANONICAL style-35 machine

**Keep `task/W4-g82-rope-coil-draw` whole** — `projectile_rope_coil.rs`
(`coil_step`/`coil_proj_row`/`coil_rope_kind`/`is_rope_cell`/`is_coil` +
the chain preview + `coil_death_drop` + `check_rope_usability`), the
`35 =>` dispatch arm, the `data.rs` literal rows, the `coil_kill` death
hook + `pending_coil_drops` drain, the usability gate in `use_tile_item`,
the chain sheets in `assets.rs`, the friendly pin, the PVP exemption,
the style-35 `AutomaticallyChangesDirection` write, the wind-list 35.

**Drop from `task/W4-d4-proj-ai-31-40`** (exact hunks, UPDATED to d4 tip
`cafec7222` line numbers — the union push added the extras module and
its wiring; every row grep-verified at that tip):

| # | file | hunk (`cafec7222`) | what |
| --- | --- | --- | --- |
| 1 | `crates/tr-app/src/projectile.rs` | `:11394-11522` | the `ai_rope_coil` fn + its doc comment (unchanged by the union push) |
| 2 | `crates/tr-app/src/projectile.rs` | `:17812-17818` | the `35 => self.slots[i].ai_rope_coil(world, local_owner)` dispatch arm + its comment (was `:17792-17798`; a second `35 =>` would trip the `unreachable_patterns` deny anyway — S1-D-4c) |
| 3 | `crates/tr-app/src/projectile/data.rs` | `:604-607` | the `ROPE_COIL`/`VINE_ROPE_COIL`/`SILK_ROPE_COIL`/`WEB_ROPE_COIL` consts — **consumers UPDATED**: after dropping rows 1/2/4/5/6/7 below the only survivors would be d4's extras module (`projectile_rope_coil.rs:49` import) which row 6 drops wholesale; g82's module owns its own `pub(crate)` consts (`projectile_rope_coil.rs:92-95`) |
| 4 | `crates/tr-app/src/projectile/data.rs` | `:2326-2341` | the named-const row arm (the row-dedupe, below — unchanged) |
| 5 | `crates/tr-app/src/projectile.rs` | `:26861-27111` | the four `the_rope_coil_*` witnesses (was `:26810-26939`; the placement witness now also carries the union refund-queue arms `:26965-27048` — drop those too, g82's death-hook witness covers cue+refund+owner-gate; the per-kind placement ladder salvage note below) |
| 6 | `crates/tr-app/src/projectile_rope_coil.rs` | whole file (`:1-331`) | **NEW at cafec722** — d4's extras module: `is_coil`/`coil_death_drop`/`CoilLink`/`coil_chain_sheets`/`coil_chain_segments`/`draw_coil_chain` are byte-compare-identical copies of g82's (modulo doc/visibility, see the re-read below); its 4 witnesses duplicate g82's minus the NaN pin + plus the is_coil gate rows (salvage below) |
| 7 | `crates/tr-app/src/projectile.rs` | `:14751`, `:15166`, `:15266-15272`, `:20389-20419` | **NEW** — the `pending_death_drops` field + init + `drain_death_drops` + the death-hook refund arm. Drop the COIL half (`:20389-20409`); the beach-ball half (`:20410-20418`) is d4's own 155 machine — KEEP the hook arm; its queue/drain half keeps d4's field + `drain_death_drops` + the `UpdateItems` two-integer-draw drain loop (`world_sim/projectiles.rs:633-658` minus the coil comment, witness-pinned by `w4_d4_union_the_beach_ball_return_drains_on_the_items_stream`) — do NOT fold the ball onto g82's `spawn_item_drop_px` shape, that would silently change its pinned stream draws |
| 8 | `crates/tr-app/src/world_sim/projectiles.rs` | `:620-632` (coil comment), `:775-846` (the coil-drain witness + `coil_app`) | **NEW** — d4's drain loop serves BOTH kinds: keep the loop for the ball (row 7), drop only the coil-ladder comment lines and the coil witness `w4_d4_union_the_coil_expiry_drains_the_refund_as_a_world_drop` (`:802-846`) — it pins the WRONG stream + gates for coils; g82's coil drain (`:600-618`) is the canonical one. Keep the ball witness (`:848-868`) |
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
   this lane.) g82's drain survives.
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
added copies + a mis-gated wire, not a better machine.

## Cross-receipt: the d4d retire lane reads THIS reversal

The d4d (d4-retire) lane's keep-one input is this doc's FINAL verdict,
not d4's superseded `26e-rope-coil-keep-one.md` (whose KEEP-d4 ruling
was premised on g82 being unwired at `de797f45`). Receipt for that
lane: retire d4's `ai_rope_coil` + named-const rows + witnesses per
the UPDATED drop-hunks table above (rows 1-10, `cafec7222` lines);
keep d4's wind-32 + styles 31-40-less-35 + the item-row verifies +
the beach-ball refund half (whole on its own queue/drain, rows 7/8).
If d4d's retire
already ran against `4fa74b3c7` lines, re-derive against `cafec7222`
— the union push shifted the dispatch arm (+20 lines) and added the
extras module/wiring this table now covers. No code on THIS branch
changes for the retire: g82's lane is self-contained (own module,
own consts, own queue names).

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
   `drain_coil_drops` → `spawn_item_drop_px` (g82, LIVE) — the ride
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

## ROPE-FAMILY merger bible (one paragraph, the lane's whole truth)

The rope family merges as: **g82 tip (`3173cc689` + this seal) is the
CANONICAL coil lane** — keep `projectile_rope_coil.rs` whole (machine
`coil_step` + rows reference + chain preview + `coil_death_drop` +
`check_rope_usability`), the `35 =>` dispatch, the four literal
`data.rs` rows (`:3979-3980`), `coil_kill`/`pending_coil_drops`/
`drain_coil_drops` (the `:77267` owner + `:77367` noDropItem gates
KEPT, the cue ungated, the `UpdateProjectiles` drain), the usability
call in `use_tile_item`, the 7 chain sheets + `draw_with_hand` emit,
the friendly pin + PVP exemption + style-35 direction write +
`framing_rope`; **drop d4's machine half** (`ai_rope_coil`, the
`35 =>` arm, the 4 consts + named row arm, the coil witnesses) **and
d4's union-extras duplicates** (its `projectile_rope_coil.rs` copies,
its private usability fn + call, its mis-gated refund hook + wrong-
stream drain — the UPDATED ten-row table above, `cafec7222` lines;
the dedupe is trivial because the pure fns are byte-identical copies
of g82 `67c9a740`); **keep from d4** the wind-list **32** (merged
list `2|8|24|28|29|32|35` — d4's beach-ball machine is its own), the
beach-ball refund half (kept whole on d4's own queue + `UpdateItems`
drain — rows 7/8, never folded onto g82's coil drain shape), the
859/1156/190 row verifies, styles 31-40 less 35, and the death-centre
timing deviation paragraph (pool-wide, both ports share it);
**f3-pulley is disjoint** (player.rs vs projectile*/editing.rs/
assets.rs — the place→attach→ride→coil-refund chain composes on the
merged tree, witness `w4_g82_f3_compose_…` pins f3's exact predicates
at `0b11548f`); **g83 vines stay out** (the pulley gate is
`is_rope`+`tileRope`, never `IsVine`); salvage d4's four-kind
placement loop + is_coil gate rows + NaN-quirk witness onto g82's
module (rename `w4_d4_union_` → `w4_g82_`); the pool-wide cut-tile
`TileKill` gap (7 census sites) is **R-26-13's seed**, NOT a merge
blocker and NOT a coil-only fix; row-dedupe 171/475/505/506 keeps
g82's literal arm (term-identical both-ported); after the merge run
the deferred gate `cargo test --workspace -- --skip
migrates_version_2_kinds_to_item_ids` — every lane witness was
WRITTEN NOT RUN under the code-only constraint.

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
