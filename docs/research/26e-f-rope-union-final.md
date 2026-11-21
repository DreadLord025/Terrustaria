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

## FINAL VERDICT: g82's `coil_step` is the CANONICAL style-35 machine

**Keep `task/W4-g82-rope-coil-draw` whole** — `projectile_rope_coil.rs`
(`coil_step`/`coil_proj_row`/`coil_rope_kind`/`is_rope_cell`/`is_coil` +
the chain preview + `coil_death_drop` + `check_rope_usability`), the
`35 =>` dispatch arm, the `data.rs` literal rows, the `coil_kill` death
hook + `pending_coil_drops` drain, the usability gate in `use_tile_item`,
the chain sheets in `assets.rs`, the friendly pin, the PVP exemption,
the style-35 `AutomaticallyChangesDirection` write, the wind-list 35.

**Drop from `task/W4-d4-proj-ai-31-40`** (exact hunks, d4 tip `4fa74b3c7`
line numbers):

| # | file | hunk | what |
| --- | --- | --- | --- |
| 1 | `crates/tr-app/src/projectile.rs` | `:11394-11522` | the `ai_rope_coil` fn + its doc comment |
| 2 | `crates/tr-app/src/projectile.rs` | `:17792-17798` | the `35 => self.slots[i].ai_rope_coil(world, local_owner)` dispatch arm + its comment (a second `35 =>` would trip the `unreachable_patterns` deny anyway — S1-D-4c) |
| 3 | `crates/tr-app/src/projectile/data.rs` | `:604-607` | the `ROPE_COIL`/`VINE_ROPE_COIL`/`SILK_ROPE_COIL`/`WEB_ROPE_COIL` consts (used ONLY by the dropped arm/rows/witnesses — re-grepped, no other consumer on d4) |
| 4 | `crates/tr-app/src/projectile/data.rs` | `:2326-2341` | the named-const row arm (the row-dedupe, below) |
| 5 | `crates/tr-app/src/projectile.rs` | `:26810-26939` | the four `the_rope_coil_*` witnesses (g82's module + dispatch witnesses cover the same ground; the per-kind placement ladder salvage note below) |

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
