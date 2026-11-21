# 237 — W4-g48: the f11 minecart fail-tile tail-fix hunk (upstream publish)

Number 237 per the research-number registry (research/150 on
`origin/task/W4-d54-dust-sites-world`: "Numbers 237+ stay FREE for future
lanes"; 139 is RESERVED for g52c-net-master — see that lane's receipt in
research/17's NET-APPLY-1 declaration, this lane does not take it).

## The defect (f11 branch tail, STILL wrong at its tip)

`origin/task/W4-f11-wiring-arms-4` tip `84ed6342b`,
`crates/tr-core/src/minecart.rs` blob `5efef353`, `frame_track`'s tail
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

## The fix hunk (for the f11e/merger lane — apply on the f11 tip)

File `crates/tr-core/src/minecart.rs`, the tail block only (f11 tip lines
357-362; the g48 landed form is blob `02ee05d98` → post-witness):

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
killing form — this hunk is published so f11e/the merger carries it
upstream instead of re-deriving it; the DEPRECATED-BY-UNION module note
(the e20 superset drop, research/95 §6) does NOT cover this tail: the
`flag4`/`KillTile(fail)` half is the 95 §8.8 salvage note, and any
survivor of the union keeps the no-op semantics.

## The witness (landed on this branch with the hunk)

`crates/tr-core/src/minecart.rs::tests::w4_g48_the_pound_fail_tail_never_removes_the_track`
— the isolated rail (front 0) between two level neighbours (mask 18,
options `[1]`) cycles 0 → 1 under `pound: true, mute: false`, `flag4`
fires, and the tile SURVIVES active with the new frame. Derived from the
connection tables by hand (the options builder's exact-match rule for
popcount-2 masks; frame 1's mask `0x12`); WRITTEN NOT RUN (code-only).

## Verification

Code-only — no cargo/rustc/tests/rustfmt/graphify invoked. The f11 tip
blob re-read at `84ed6342b` (the killing `kill_tile` call present); the
C# fail arm re-read (`WorldGen.cs:63700-63737` — `effectOnly` dust,
`AttemptFossilShattering`, the fail conversions + `SquareTileFrame`
return); the reader default re-read (`MessageBuffer.cs:1229`).