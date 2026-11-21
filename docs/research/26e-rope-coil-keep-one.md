# 26e — Rope-coil DUPLICATE keep-one (W4-d4 DOC-SWEEP, 2026-11-21)

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

**KEEP-ONE VERDICT: keep W4-d4's `ai_rope_coil` + the four data rows.**
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