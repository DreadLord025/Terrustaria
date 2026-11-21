# 239 — W4-g48: the f11 minecart fail-tile tail-fix hunk (upstream publish)

Number **239** — RENAMED from the initial 237 (correction phase,
2026-11-21). The 237 pick per the research/150 free block
(`origin/task/W4-d54-dust-sites-world` tip `38bfd1d22`: "Numbers 237+ stay
FREE for future lanes"; 139 RESERVED for g52c-net-master) TRIPLE-collided:
e23 `237-bestiary-ui-final.md` (first-add `7a7779690`
2026-11-21T10:53:03+02:00), g61 `237-g61-spawn-family-final.md`
(`fd406ecba` 12:12:59), this doc (`34029437a` 12:15:09). First-pushed
timestamp wins per the registry rules → **e23 keeps 237**; the losers
renumber in first-add order: **g61 → 238** (that lane's own rename),
**g48 → 239** (this doc). A full 358-tip `task/W4-*` scan found exactly
these three 237 claims; 238/239 were unclaimed (g58's 240–243 happiness
family is the only other 237+ claim at scan time — the lane's new
correction publishes took 244/245). All refs on this branch re-pointed
(INDEX row, ANALYSIS final-phase note, PORT_MAP minecart row,
research/17 witness counts + the g52c receipt note).

## The defect (f11 branch tail, STILL wrong at its tip)

`origin/task/W4-f11-wiring-arms-4` tip `14ca0a8d5` (correction-phase
re-probe 2026-11-21; the f11e-retire phase 1b landed the 150-registry
renames 233/234/235 but did NOT touch the minecart body),
`crates/tr-core/src/minecart.rs` blob `5efef353` — UNCHANGED since the
initial publish at `84ed6342b`; the killing call sits at tip lines
360-362, `frame_track`'s tail
(`Minecart.cs:1163-1166` — `(pound & flag4) && !mute`):

```rust
    if (pound & flag4) && !mute {
        let _ = crate::edit::kill_tile(world, i, j, true);
    }
```

The port's kill seam has NO fail flavour — the `true` maps to `no_item`
and the call KILLS the track tile. The C# `KillTile(i, j, fail: true)`
never removes the tile (`WorldGen.cs:63709-63737`: the shakeable-tree
gate, the grass/moss type conversions — 314 matches none — then
`SquareTileFrame` and return; dust/sound only).

f11's own callers (the wire arm `Wiring.cs:1747` → `FlipSwitchTrack` →
`FrameTrack(pound, mute: true)`, the hammer path) all pass `mute: true`,
so the branch never fires there — the defect is INERT on f11 today.
It becomes live for ANY future caller passing the C# default
`mute: false` — the net-17 kind-15 reader already does
(`MessageBuffer.cs:1229`: `Minecart.FrameTrack(x, y, pound: true)`).

## The fix hunk (for the f11f-seal/merger lane — apply on the f11 tip)

File `crates/tr-core/src/minecart.rs`, the tail block only (f11 tip
`14ca0a8d5` lines 357-362 — re-pinned at the correction phase; the g48
landed form is blob `02ee05d98` → post-witness):

```diff
     // `:1163-1166` — `(pound & flag4) && !mute` → KillTile(fail: true). The
-    // wire/hammer callers pass mute: true, so this never fires here; the
-    // fail-tile sound is the app's Dig cue (edit/hammer.rs precedent).
+    // wire/hammer callers pass mute: true, so this never fires there. The
+    // net-17 reader (`MessageBuffer.cs:1229`) passes the DEFAULT mute: false
+    // — the branch fires, but the C# fail-tile KillTile never removes the
+    // tile (`WorldGen.cs:63709-63737`: the grass/moss conversions + the
+    // SquareTileFrame wake, dust/sound only). W4-g48 fix over the f11 body:
+    // the old `kill_tile(world, i, j, true)` mapped `true` to no_item and
+    // KILLED the track — the port's kill seam has no fail flavour, so the
+    // fail tick skips the break (the kind-0 `net_apply` precedent; the
+    // fail-dust/sound is the visual seam, ANALYSIS). The framing wake the
+    // C# fail arm runs is the caller's (`reframe_area` after the call).
     if (pound & flag4) && !mute {
-        let _ = crate::edit::kill_tile(world, i, j, true);
+        // KillTile(fail: true) — no world-state change (see above).
     }
     true
```

The g48 lane landed this fix with its kind-15 wiring (`3a310ccce`,
ANALYSIS "The g48 cited-skip closures"). The f11 branch tail is STILL the
killing form — this hunk is published so f11f-seal/the merger carries it
upstream instead of re-deriving it; the DEPRECATED-BY-UNION module note
(the e20 superset drop, research/95 §6) does NOT cover this tail: the
`flag4`/`KillTile(fail)` half is the 95 §8.8 salvage note, and any
survivor of the union keeps the no-op semantics.

## The f11e-retire consumption receipt (correction phase, 2026-11-21)

The f11e-retire wave (`b03d53f1d` phase 1 → `3da456c3c` phase 2 →
`07e39eb79` phases 3+4 → `14ca0a8d5` phase 1b) consumed the 150-registry
renames (134/135/136 → 233/234/235) and published the c42e fold-lists —
it did NOT consume this hunk: the minecart.rs blob is byte-identical
(`5efef353`) at both `84ed6342b` and the new tip `14ca0a8d5`; the last
commit touching the file is `e052a69f5` (the DEPRECATED-BY-UNION
comment-only banner). **The f11 branch still carries the defect.** The
hunk above is re-published EXACT (unchanged bytes, re-pinned line
numbers) for the f11f-seal/merger.

Merger direction note (re-probed at the e20 tip `46fdcfd8e`): the e20
superset's fail tail is ALREADY the correct no-op shape — its
`frame_track_hammer` returns the `fail_kill` CONDITION
(`crates/tr-app/src/minecart.rs`, blob `442cf21b9`, the `:1398-1400`
contract) and the app caller runs the 314 fail slice WITHOUT a tile kill
(`world_sim/editing.rs:1452-1476`, blob `a9d3d9087`: the Item52 cue +
`square_tile_frame_default` + `refit_tracks_around`, no `kill_tile`).
So the union merge that drops the f11 module per research/95 §6 loses
nothing — the defect dies with the module; this hunk matters only if
the f11 body SURVIVES any merge path (the f11f-seal keep-one fallback),
and for the g48 net-17 kind-15 reader which calls the tr-core
`frame_track(..., pound: true, mute: false)` directly
(`net_apply.rs:284`/:492 — the g48 landed no-op form is what keeps the
track alive there).

## The witness (landed on this branch with the hunk)

`crates/tr-core/src/minecart.rs::tests::w4_g48_the_pound_fail_tail_never_removes_the_track`
— the isolated rail (front 0) between two level neighbours (mask 18,
options `[1]`) cycles 0 → 1 under `pound: true, mute: false`, `flag4`
fires, and the tile SURVIVES active with the new frame. Derived from the
connection tables by hand (the options builder's exact-match rule for
popcount-2 masks; frame 1's mask `0x12`); WRITTEN NOT RUN (code-only).

## Verification

Code-only — no cargo/rustc/tests/rustfmt/graphify invoked. The f11 tip
blob re-read at `84ed6342b` AND re-pinned at the correction-phase tip
`14ca0a8d5` (blob `5efef353` both times — the killing `kill_tile` call
present at tip lines 360-362); the e20 union-survivor fail slice re-read
at tip `46fdcfd8e` (the condition-return + the app-side no-kill 314
slice); the C# fail arm re-read (`WorldGen.cs:63700-63737` —
`effectOnly` dust, `AttemptFossilShattering`, the fail conversions +
`SquareTileFrame` return); the reader default re-read
(`MessageBuffer.cs:1229` — `Minecart.FrameTrack(num150, num151, pound:
true)`, mute defaults false).