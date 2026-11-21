# 144 — the item entry/exit splash KEEP-ONE (W4-f47 × W4-e19, 2-way)

> **ARBITER FINAL v2: `144-ARBITER-FINAL-v2.md`** — the e19 SHIMMER
> SEAL (`0fb0ac4ad`) re-opened the dispute (keep e19) after fixing its
> entry gate verbatim; the v2 arbitration re-ruled at both new tips on
> witness SUPERSET COVERAGE and UPHELD this doc's keeper (the f47
> `Splash` shape). Both prior FINALs are superseded; the merger reads
> v2, not this receipt. (v1 record: `144-ARBITER-FINAL.md`.)
>
> **SEALED (research/151)**: the family-wide cross-lane compositions
> (the lights mutual-receipt, the 676 HitEffect census, the b1
> activation-contract, the d6 ShimmerImmunity compose, the d43 keep-one)
> live in `151-shimmer-family-seal.md`; its §7 family bible is the
> merge checklist this receipt feeds.

> Coded 2026-11-21 on `task/W4-f47-liquid-shimmer-2` (base `a2561d6d`).
> Code-only: no cargo/tests/rustfmt/graphify invoked; every verdict is a
> re-read of the cited C# window (≤100 lines) against both lanes' pushed
> code. The e19 lane read at `origin/task/W4-e19-liquid-quicksell-shimmer`
> tip `939e45dc8` (the full splash diff: `7d3cb5e3` deferred 4,
> `3ea49d98b` deferred 5 wet flags, `e2003946b` round-3 box fix,
> `939e45dc8` round-3 flush fix). The f47 lane's splash landed in
> `d7eb4e913` (doc-sweep 36 items 1+2, research/121).

## The region

`WorldItem.MoveInWorld`'s wet block (`WorldItem.cs:865-1045`): the two
probes (`:865` LavaCollision full-box, `:870` WetCollision centre-inset),
the sticky kind-flag sets (`:866-878`), the entry edge (`:879-958`), the
exit edge (`:959-1035`), the dry clear (`:1036-1041`), the wetCount
decrement (`:1042-1045`). Both lanes ported it; the keep-one picks ONE
shape per arm and folds the other lane's unique wins.

## Arm-by-arm verdict (C# vs e19 `7d3cb5e3`+ vs f47 `d7eb4e913`)

### 1. Cooldown semantics — BOTH faithful, f47 shape KEPT

C# (`:883-885` entry / `:962-964` exit): the burst gate is
`wetCount == 0`; on a burst `wetCount = 20`; the decrement runs at the
block's tail (`:1042-1045`) EVERY update while > 0 — so the entry tick
reads 19 afterwards. The grabbed arm zeroes it (`:428`).

- **e19**: `wet_count: i32` on `ItemDrop`, the same 20-arm / tail-decrement
  / grabbed-zero. Faithful.
- **f47**: identical (`item.rs` `wet_count`, `:1325-1364` region, the
  grabbed clear at `:1239`). The "latch" in the f47 commit message is the
  SPLASH REQUEST latch (`splash_due`), NOT a different cooldown — the
  cooldown semantics are the C# wetCount-20 in both lanes. **No
  semantic difference.** The f47 field names + the inline arm structure
  stay (the e19 `wet_count` is the same field; the merge dedupes trivially).

### 2. Position-snapshot timing — BOTH faithful, f47 KEPT

C#: the dust anchor reads `base.position` AT the edge (inside
`MoveInWorld`, after the conveyor bend `:864`, BEFORE the tile collision
`:1051` and `ApplyMovement` `:1074-1084`). Both lanes snapshot `self.pos`
at the arm site (post-conveyor, pre-collision) — the same tick, the same
position. e19's `ItemSplash.pos` and f47's `Splash.pos` are identical
reads. **f47 KEPT** (the struct carries the gates + pos together).

### 3. The entry lava gate — f47 VERBATIM, e19 equivalent-but-not-verbatim

C# `:886`: `if (!flag2)` — the LOCAL `LavaCollision` result THIS tick
(the `bool flag2` at `:865`), NOT the sticky `base.lavaWet`.

- **f47**: `lava: lava_now` — the local probe result, verbatim (`:1333`).
- **e19**: `lava: self.lava_wet` — the sticky flag. At the ENTRY edge the
  two are equivalent in practice (the `:866-869` set ran lines earlier,
  and the sticky flag can only be true if some earlier tick's probe hit
  lava — but the dry clear `:1036-1041` resets it whenever `!wet`, and a
  dry drop entering liquid had `wet == false` last tick, so the sticky
  flag was cleared UNLESS the drop was grabbed... the grabbed arm also
  clears it `:429`). The equivalence holds for every reachable state,
  but the f47 read is the verbatim one. **f47 KEPT.**
- The EXIT gate (`:965`) reads the STICKY `base.lavaWet` in C# — both
  lanes read `self.lava_wet` there. Faithful both.

### 4. The exit pre-clear snapshot — BOTH faithful, f47 KEPT

C#: the exit arm (`:965-1033`) runs BEFORE the `:1036-1041` dry clear, so
it sees the sticky flags pre-clear. Both lanes snapshot at arm time
(f47's `Splash` doc cites this explicitly; e19's queue comment says "the
exit arm the OLD ones before the `!wet` tail clears them"). **f47 KEPT.**

### 5. Drain slot / stream order — BOTH faithful, f47 KEPT

C#: the dust + sound run INLINE inside `MoveInWorld` (`UpdateItem` `:413`),
before `UpdateItem_VisualEffects` (`:436`), all under the `UpdateItems`
swap (`Main.cs:18024`). Both lanes drain same-tick in the per-drop loop
right after `drop.update`, before the flicker head:

- **e19**: `drop.take_splash()` → the free fn `item_splash(...)` in
  `world_sim/items.rs` (the g67 rain-splash contract).
- **f47**: `drop.splash(...)` — the method on `ItemDrop` (the latch spent
  inside), same call position, same stream.

The draw ORDER inside the burst is identical (the per-dust `NewDust`
internals, then the `Next(6)` colour for 308, then the case-19 play after
the loop). **f47 KEPT** — the method shape keeps the latch spend local;
the e19 free fn + `take_splash` are the dedupe losers.

One real difference: e19 computes the water style INSIDE the drain
(`calculate_water_style(world, camera)` per burst); f47 snapshots it once
per items pass (`splash_water_style`, the `TR_WATER_STYLE` pin capped at
14 like the draw pass). The C# reads `Main.waterStyle` — a per-FRAME
static, not a per-burst recompute — so the f47 once-per-pass read is
closer (and the pin matches the draw pass). **f47 KEPT.**

### 6. The four dust bodies — BOTH verbatim, f47 KEPT

| arm | C# | e19 | f47 |
| --- | --- | --- | --- |
| shimmer | 10x308, vy−4, vx×2.5, scale SET 0.8, noGravity, NO alpha, Next(6) colour (`:890-913`) | same | same |
| honey | 5x152, vy−1, vx×2.5, scale 1.3, alpha 100, noGravity (`:917-926`) | same | same |
| water | 10x dustWater, vy−4, vx×2.5, scale ×0.8 (MULTIPLY), alpha 100 (`:930-939`); exit drops the −8 Y (`:1011`) | same | same |
| lava | 5x35, vy−1.5, vx×2.5, scale 1.3, alpha 100 (`:944-953`) | same | same |

The style-4/5 shimmer cues (`:913`/`:992`) vs the 3-arg default style 1
(`SoundEngine.cs:102`): both lanes identical. The anchor
`(pos.X−6, pos.Y + height/2 − 8)` box `(width+12, 24)`: both lanes
identical (the integer half — 7 at the old 14 box, 8 at the C# 16).
**f47 KEPT** (the tuple-driven body table vs e19's if-chain — same
writes, same order).

### 7. The drop box — e19's 16 FIX FOLDED (the f47 lane was WRONG)

C# `WorldItem.cs:138`: `base.width = (base.height = 16)`. The f47 lane
carried the old 14 px stand-in (`SIZE = 14.0`); e19's round-3 `e2003946b`
fixed it to 16 across every probe, keeping the 14 px DRAW visual centred
(`pos + 1`, `SIZE - 2`). **FOLDED into f47** — the const, the draw quad,
every geometry-pinned witness re-derived (the probe 10x7 → 10x8 at
+3/+4, the splash `Next(22)` → `Next(24)`, the grab-rect 42/43 edge, the
magnet centre-8, the light anchors 168/184, the resting fixtures flush at
19*16, the entry-latch park 184, the aetherium normal-platform residual
0). The e19 witness re-pins were re-derived independently on the f47
fixtures (the two lanes' fixture sets differ — the f47 `shimmer_world`
pool is rows 18-19 over a row-20 floor, e19's is the same shape; the
`resting_drop` park moved 19*16+2 → 19*16 in both).

### 8. The dry flush residual — e19's round-3 fix ALREADY f47 semantics

e19's pre-round-3 code had a dry snap-and-zero stand-in; `939e45dc8`
retired it (the `Collision.TileCollision` flush residual is the velocity
in BOTH arms, `Collision.cs:2394`/`:2444`). The f47 lane NEVER had the
stand-in — `move_in_world` runs the real `tile_collision_opts` and keeps
`tc.vel` verbatim, then `ApplyMovement` integrates it. **Nothing to fold
code-wise**; the e19 landing/rest witness pair folded as `w4_f47k1_*`
(the dry 303→304 residual 1.0, the rest residual 0; the wet twin).

### 9. `box_is_wet` — e19's NAME kept, the f47 deletion was a COMPILE BREAK

The f47 doc-sweep 36 (`d7eb4e913`) deleted `ItemDrop::box_is_wet` (the old
cell-grid stand-in) but `app/frame/draw.rs:419` still calls it — the
branch did not compile. e19 kept the name, re-pointed at the faithful
`player::wet_collision` bool half. **FOLDED**: the fn restored on f47
with the e19 body (the same inset maths as `wet_probe`'s `npc_ai_slime`
engine, minus the kind statics). The e19 surface-offset witness folded
as `w4_f47ds_the_box_is_wet_probe_uses_the_liquid_surface_offset`.

### 10. The wiring.rs literals — a SECOND f47 compile break, fixed

The phase-3 `ItemDrop` field additions (`honey_wet`/`lava_wet`/
`wet_count`/`repulsion_brake`) + the doc-sweep `splash_due` never updated
the two struct literals in `world_sim/wiring.rs` (`:190`/`:783`). e19's
`7d3cb5e3` updated its own literals. **FIXED on f47** (all five fields
added to both literals).

## The keep-one result

**KEEP the f47 `Splash`/`splash_due`/`ItemDrop::splash` shape** (the
verbatim entry gate, the once-per-pass water style, the method-local
latch spend). **DROP the e19 `ItemSplash`/`pending_splash`/`take_splash`/
free-fn `item_splash`** at merge (the same behaviour, the loser shape).
**FOLD the e19 wins**: the 16 box (+ every witness re-pin), the
`box_is_wet` name/body, the four pass-level splash witnesses, the
flush landing/rest pair.

### Fold-hunks applied on this branch (the commit map)

1. `item.rs`: SIZE 16 + the draw VIS quad + `box_is_wet` restored + the
   probe/splash/repulsion/grab/magnet/aetherium witness re-pins + the
   `w4_f47k1_*` flush pair + the folded `box_is_wet` surface witness.
2. `entity_lights.rs` / `item_lights.rs`: the 14.0 → 16.0 fixture args +
   the 167/183 → 168/184 and −13 → −12 anchor comments.
3. `world_sim/items.rs`: the resting fixtures 19*16 flush + the four
   `w4_f47k1_*` pass-level splash witnesses (the e19 quartet, re-pinned).
4. `world_sim/editing.rs`: the rest fixture `floor_top - SIZE`.
5. `bug_net.rs` / `shimmer.rs`: the 16-box doc comments.
6. `world_sim/wiring.rs`: the two literals' missing fields.

### e19-owned hunks NOT folded (their lane keeps them at merge)

- The `shimmer_tail`/`apply_decraft` `wet = true` writes (`3ea49d98b`,
  `WorldItem.cs:1844`/`:635`). The f47 Transform arm still misses the
  shared-tail `wet` write — e19's chain-apply half carries it; the merge
  keeps e19's `shimmer.rs` apply (the two lanes' halves are disjoint:
  e19 edits the chain's apply, f47 the NPC drain — research/121 §5).
- The `rise_over_shimmer` two-row centre scan (e19's `UpdateShimmer`
  rise, `:493-504`) vs f47's `over_shimmer` single-tile-above read
  (`CanShimmerAtPosition` `:756-768` — the RAMP gate's probe reused for
  the rise). The C# rise scan is the two-row centre one; e19 is
  verbatim there. The merge reconciles onto e19's rise (a one-line
  swap in f47's `update_shimmer`), NOT duplicated here (the f47
  `update_shimmer` is the phase-2 shape the 91→92 re-pin derives on).
- The wetVelocity blocked-axis refresh (e19's hand-rolled TileCollision
  vs f47's `tile_collision_opts`): the residual semantics agree (both
  keep the flush residual as velocity); the merge keeps ONE collision
  engine (f47's opts wrappers are the shared seam).

## Witness dedupe (the e19c active reads — its brief item-4 overlap)

The e19 lane's splash witnesses (`7d3cb5e3`'s four in
`world_sim/items.rs::shimmer_tests`, re-pinned by `e2003946b`) overlap
this lane's brief item 4 (the 676 row) only in the FILE, not the
behaviour: the e19 quartet tests the ITEM splash pass; the f47 676 work
is the NPC roster + AI. The dedupe:

| e19 witness | f47 fate |
| --- | --- |
| `w4_e19_the_shimmer_entry_splash_bursts_10_dust_308` | folded → `w4_f47k1_the_shimmer_entry_splash_bursts_10_dust_308` |
| `w4_e19_the_shimmer_exit_splash_after_the_cooldown` | folded → `w4_f47k1_the_shimmer_exit_splash_after_the_cooldown` |
| `w4_e19_the_wet_count_suppresses_the_second_burst` | folded → `w4_f47k1_the_wet_count_suppresses_the_second_burst` |
| `w4_e19_the_water_entry_splash_uses_the_water_style_dust` | folded → `w4_f47k1_the_water_entry_splash_uses_the_water_style_dust` (the water-style read re-pointed at the f47 once-per-pass source) |
| `w4_e19_the_wet_probe_uses_the_liquid_surface_offset` | folded → `w4_f47ds_the_box_is_wet_probe_uses_the_liquid_surface_offset` |
| `w4_e19_the_dry_drop_integrates_at_full_velocity`'s landing/rest pair | folded → `w4_f47k1_the_dry_flush_landing_carries_the_residual_then_rests` + the wet twin |
| the f47 unit-level 71/31-draw twins (`w4_f47ds_the_shimmer_entry_splash_draws_71...`, `..._lava_splash_wins_the_gate_and_draws_31`) | KEPT (e19 has no draw-count twin — the f47 unique win, re-pinned at `Next(24)`) |

At merge: DELETE the e19 `w4_e19_*` splash quartet + the surface-offset
witness (the f47 folds cover them); KEEP the e19 gravity/latch/grabbed
witnesses (`3ea49d98b`'s three — the f47 lane has the ladder-order twin
but not the latch/clear/grabbed pass-level ones).

## The wet-probe 2-way (brief item 2)

- **e19c active**: `box_is_wet` → `player::wet_collision` (the bool
  half), the move-half probe → `npc_ai_slime::wet_collision` (the
  3-tuple), both at SIZE 16 → the inset **10x8 at (pos+3, pos+4)**.
- **f47 (this branch, post-fold)**: `wet_probe` →
  `npc_ai_slime::wet_collision`, `box_is_wet` → `player::wet_collision`,
  both at SIZE 16 → the SAME 10x8 box.
- **C# census** (`Collision.cs:1572-1651`): `num = min(10, Width)`,
  `num2 = min(Height/2, Height)` — INTEGER halves; the probe top-left
  `Center − (num/2, num2/2)` (integer halves again): for 16x16,
  `num = 10`, `num2 = 8`, probe at `(x + 8 − 5, y + 8 − 4)` =
  `(x+3, y+4)`, span 10x8. The scan range `(int)(pos/16) − 1 ..
  (int)((pos+size)/16) + 2` clamped to `maxTilesY − 40`; the surface
  drop `(256 − liquid)/32 * 2` px; the FIRST overlapping cell wins
  (early return) and sets the honey/shimmer statics from THAT cell;
  the liquid-less SLOPED tile under liquid counts via the tile above
  (`:1628-1646`, honey ELSE-IF shimmer there). Both port engines
  (`npc_ai_slime::wet_collision` `:328-396`, `player::wet_collision`
  `:11478-11527`) match every term — the only difference is the kind
  statics (the player port drops them, no consumer). **ALIGNED: both
  lanes now read the identical C# box; the 14px→16px fix is the e19
  win folded, the inset maths was already faithful on both.**
- `LavaCollision` (`Collision.cs:1653-1684`): the FULL box (no inset),
  the same surface drop, lava-only cells. Both lanes →
  `player::lava_collision` at SIZE 16. Aligned.

## The 676 ShimmerSlime AI (brief item 3)

**aiStyle census**: `NPC.cs:17570` — `aiStyle = 1` (the slime family).
The machine EXISTS: `npc_ai_slime::ai_001_slimes` (the S1-D-3 A1-4
re-port, `NPC.cs:60341-61723`), wired by W4-b1's `ai_001_slime_family`
adapter (`f010b85b`, pushed — the `update_slime` stand-in retired on
THAT branch). The f47 676 roster row (`e8bacc2f`) routes the kind
through the shared stand-in until b1 lands.

**The 676-specific arm** (`NPC.cs:61128-61150`, inside `AI_001_Slimes`):
the torch-23 centre light + the conditional 306 dust. This was the
censused skip on BOTH lanes (the b1 tip still carries "unported kind"
at its `:530`). **WIRED on f47** (the keep-one round):

- The gate `(velocity.Length() > 1 && Next(3) == 0) || Next(5) == 0`
  (`:61131`) — the exact short-circuit (speed ≤ 1 draws ONLY Next(5);
  speed > 1 draws Next(3) first, the `||` skips Next(5) on its hit).
- On a hit: `NextVector2FromRectangle(base.Hitbox)` (2 NextFloat over
  the INTEGER-truncated top-left, `Utils.cs:1256-1258`), the hue
  `NextFloat() * 0.1`, the scale `NextFloat() * 0.3 + 0.5` — 4 draws
  in-arm; the velocity `(pos − Center) * 0.1`, X ×0.25, positive Y
  flipped (`:61139-61145`).
- The `NewDustPerfect(pos, 306)` + the post-writes (noGravity,
  noLightEmittance, alpha 127, the `hslToRgb((tve/300 + jitter) % 1,
  1, 0.65)` colour with A=0, the scale, fadeIn 0.9, the netOffset on
  the position) ride `wants_shimmer_slime_dust` to the sim drain (the
  gallop-dust seam — the allocator's 6 draws land post-tick, the
  recorded pool-pass deviation).
- The light: `Lighting.AddLight(base.Center, 23)` (`:61130`) — the
  `(Vector2, torchID)` overload truncates BEFORE dividing
  (`Lighting.cs:154`, `XAnchor::CenterTruncated`), the colour is
  `TorchID.TorchColor(23)` = the animated `ShimmerTorchLight`
  (`TorchID.cs:131`/`:161`, the mouseTextColor pulse —
  `tile_light::torch_color`'s id-23 arm). RNG-free → the draw-side
  `npc_lights::shimmer_slime_light` feed (the aiStyle-8 precedent).
- **The shimmer-wet flag**: the 676 arm does NOT read `shimmerWet` —
  it is unconditional per-tick (the slime's shimmer LOOK, not a
  liquid-contact gate). The wet tail (`:61463-61510`) is the generic
  slime buoyancy (the `ai_001_slimes` port already carries it, the
  676 rides the else arm `:61494-61505`). No shimmer-wet-flag wiring
  needed beyond the existing `SlimeCtx.shimmer_wet` (the gravity
  merge 0.15/5.5, `slime_gravity`).
- **The stand-in gap**: the live f47 route (`update_slime`) draws
  nothing for the 676 until b1's adapter lands — the faithful core +
  the drain are wired, the stand-in keeps its byte-identical behaviour
  (the documented merge seam; the b1 adapter reaches `ai_001_slimes`
  which now rolls the 676 gate live).
- **Cited-unwired residuals (SUPERSEDED by the seal census,
  research/151 §2)**: the `HitEffect` 676 arm
  (`NPC.cs:82058-82078`: 8 dust 310 alive / 40 on death, scale
  1.1/1.5, the per-dust `Next(3) != 0` noGravity gate, vel ×1.5 +
  base.vel ×0.1) — the HitEffect dust pass IS ported: g62 owns the
  LIVE half (`npc_hit_dust.rs`), g60 the death halves (the 676 row a
  g60 census gap, the 40-count death half seeded). f47 ports nothing
  there;
  the `FindFrame` case-676 (`:70823`, the 8-tick hop strip) IS landed
  (the f47 roster row); the gore 676 (`:89622`) is the GORE id 676,
  not the NPC (a different namespace — no action).

## Verification

Code-only: no cargo/tests/rustfmt/graphify invoked (the lane contract).
Every verdict re-read against the cited C# window; the witness re-pins
hand-derived (the 16-box probe spans, the `Next(24)` splash roll, the
flush residuals, the 676 gate draw counts). 12 new witnesses WRITTEN
NOT RUN (3 gate + 1 light + 1 drain + 4 pass-level splash + 2 flush +
1 box_is_wet surface). The two f47 compile breaks the census caught
(`box_is_wet` missing, the wiring literals) are fixed in the same round.