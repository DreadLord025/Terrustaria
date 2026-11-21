# 151 — The light-family unification: final hunks for c42c (95 §4 / §8.5)

> Renumbered 134 → 151 (2026-11-21): the INDEX-registry wave on the
> d54-branch reads assigned 134/135/136 to the first-pushed g67/g90/g31
> docs (`134-g67-post-union-folds`, `135-g90-persist-folds`,
> `136-tilepass-d-hunks`); 151-153 were free across all origin tips at
> rename time (research/150 not yet landed — if the registry assigns
> different numbers when it pushes, the merger re-points mechanically).
>
> Merger hand-off · 2026-11-21 · code-only (no cargo/tests/graphify run) ·
> sources: `decompiled/Terraria/Wiring.cs:2800-3072` (the 11 statics),
> `WorldGen.cs:72970-73024` (`TryToggleLight`), the f11 tip
> (`task/W4-f11-wiring-arms-4`, this doc's commit) and d6 `5d81f2de`
> (`task/W4-d6-proj-ai-51-60`). Implements research/95 §4's "unify at merge"
> verdict and closes §8.5 (`skip_wires=true` documented skip) structurally.

## 0. What landed ON f11 (this branch, executable half)

The nine `Wiring` Toggle* methods in `tr-core/src/wiring.rs` gained the C#
`forcedStateWhereTrueIsOn` gate — the **d6 half** of each static — with the
wire-path callers passing `None` (behaviour frozen: `None` short-circuits
every gate, the flips/skips are byte-identical to the pre-unification
bodies):

| Method | Forced-gate anchor (C# line) | OFF flag |
| --- | --- | --- |
| `toggle_holiday_light` | `:2801-2802` | clicked `frameX >= 54` |
| `toggle_hanging_lantern` | `:2828-2829` | clicked `frameX > 0` |
| `toggle_2x2_light` | `:2860-2861` | ANCHOR `frameX > 0` |
| `toggle_lamp_post` | `:2886-2890` | clicked `frameX > 0` |
| `toggle_candle` | `:2925-2926` | clicked `frameX > 0` (NO SkipWire in C#) |
| `toggle_lamp` | `:2946-2947` | clicked `frameX > 0` |
| `toggle_chandelier` | `:2980-2984` | ANCHOR `frameX % 108 > 0` |
| `toggle_fire_place` | `:3049-3053` | ANCHOR `frameX >= 54` |
| `toggle_campfire` | `:3016-3020` | ANCHOR `frameY >= 36` |

Gate form (both C# spellings are the same truth table):
`if forced.is_some() && forced != Some(off) { return; }` — the C#
`!HasValue || !Value != flag` (flip when no forced state OR forced differs
from OFF) and the early-return `HasValue && !Value == flag` are De Morgan
twins. Verified per-static against the cited lines.

`toggle_campfire` additionally gained the **skips out-collector**
(`skips: Option<&mut Vec<(i32,i32)>>`): the wire arm collects into a local
vec and joins `self.skip` (the §1 Option-gate convention); a
`TryToggleLight` caller passes `None` (its `skip_wires=false` today) or
`Some(&mut vec)` and discards/forwards. The C# order is pinned: the
validate-kill (`:3012-3015`) runs BEFORE the forced gate (`:3017`) — a
forced refusal on a broken body STILL kills the matching cells.

Witness (written not run): `w4_f11_the_unified_light_family_honours_the_forced_state`
(forced refusal / forced flip / null flip on the campfire + holiday +
candle + chandelier; the anchor-keyed gates read the ANCHOR, not the
clicked cell; the skips collect only on a flip).

## 1. The merger hunks (c42c — cross-branch, NOT executable on f11)

> **SUPERSEDE NOTE (retire phase, 2026-11-21)**: the c42 union-exec
> item 6 (`6381b52d`, research/95 §10) landed the unification in the
> OPPOSITE shape — ONE free-fn family with `forced` + `skips` (the
> campfire also `drops`), the `Wiring` methods thin wrappers passing
> `None`. §1a/§1b below are therefore SUPERSEDED: do NOT add the
> method-side collectors or drop the c42 free fns. §1c's structural
> closure holds on the c42 shape (the collector feeds `local_skips`,
> discarded — no free-fn caller owns a flood skip set). The surviving
> f11 unique is the forced-gate witness
> `w4_f11_the_unified_light_family_honours_the_forced_state` — the
> re-point contract is research/133 §7.3 (FINAL-FINAL).

### 1a. The eight non-campfire methods: add the skips out-collector

f11's methods now carry `forced` but still insert into `self.skip`
unconditionally (the base SkipWire half — correct for the wired caller,
wrong for a `skip_wires=false` `TryToggleLight` caller). The final shape
per method (the campfire's landed shape is the template):

```rust
fn toggle_lamp(
    &mut self, world: &mut World, i: i32, j: i32,
    frame_x: i16, frame_y: i16,
    forced: Option<bool>,
    skips: Option<&mut Vec<(i32, i32)>>,   // <-- ADD
) {
    // ... the forced gate (LANDED on f11) ...
    let mut skips = skips;
    for y in num..num + 3 {
        world.set_tile_frame_x(i, y, world.tile_frame_x(i, y) + num2);
        if let Some(s) = skips.as_deref_mut() { s.push((i, y)); }  // <-- was self.skip.insert
    }
}
```

Per-method skip cells (the C# `doSkipWires` blocks): lantern 2 (`:2833-2837`),
2×2 four (`:2867-2873`), lamp post 6 (`:2893-2898`), lamp 3 (`:2952-2957`),
chandelier 9 (`:2988-2993`), fireplace 6 (`:3060-3065`), campfire 6
(`:3030-3035`, LANDED). Candle + holiday light have NO SkipWire in C# —
no collector. The wire-arm call sites pass `Some(&mut skips)` +
`self.skip.extend(skips)` (the campfire arm's landed pattern); the
`HitWireSingle` arms are the ONLY wired callers, so the merge may instead
keep `self.skip.insert` behind a `do_skip: bool` param — either shape, ONE
copy. The e20 skip-set witnesses (`the_hanging_lantern_flips_both_cells_and_skips_them`
et al., `2322af90`) pin the SETS and survive both shapes.

### 1b. d6's `try_toggle_light` dispatch: KEEP, re-point at the methods

d6's free-fn family (`toggle_torch` … `toggle_holiday_light`, wiring.rs
`:3132-3336` on its tip) DROPS — each is the forced-half of a static the
unified method now carries whole. The dispatch itself is UNIQUE (no other
lane ports `WorldGen.TryToggleLight`) and stays, re-pointed:

```rust
pub fn try_toggle_light(
    wiring: &mut Wiring,                    // <-- was free fns over &mut World
    world: &mut World, x: i32, y: i32,
    forced: Option<bool>,
    skip_wires: bool,                       // <-- now HONOURed (95 §8.5 closed)
    drops: &mut Vec<crate::edit::ObjectDrop>,
) -> bool {
    let Some(t) = world.tile(x, y) else { return false };
    if !t.active() { return false; }
    let (fx, fy) = (t.frame_x, t.frame_y);
    let mut skips = Vec::new();
    let sk = if skip_wires { Some(&mut skips) } else { None };
    match t.kind {
        4 => { /* the torch ±66 inline flip + the forced gate — d6's
                  toggle_torch body verbatim, it has no Wiring-method twin
                  (the f11/c42 torch arm is inline too); keep ONE inline
                  copy here or extract a shared free fn */ }
        215 => wiring.toggle_campfire(world, x, y, fx, fy, forced, sk),
        33 | 49 | 174 | 372 | 646 => wiring.toggle_candle(world, x, y, fx, forced),
        405 => wiring.toggle_fire_place(world, x, y, fx, fy, forced, sk),
        95 | 100 | 126 | 173 | 564 => wiring.toggle_2x2_light(world, x, y, fx, fy, forced, sk),
        92 => wiring.toggle_lamp_post(world, x, y, fx, fy, forced, sk),
        34 => wiring.toggle_chandelier(world, x, y, fx, fy, forced, sk),
        93 => wiring.toggle_lamp(world, x, y, fx, fy, forced, sk),
        42 => wiring.toggle_hanging_lantern(world, x, y, fx, fy, forced, sk),
        149 => wiring.toggle_holiday_light(world, x, y, fx, forced),
        _ => return false,
    }
    // skip_wires=true: the collected cells join the flood's skip set —
    // the C# SkipWire writes the STATIC _wireSkip the running flood reads.
    // A TryToggleLight caller outside a flood (the aiStyle-60 gun streams)
    // passes false; a future wired caller passes true and the cells must
    // reach wiring.skip (the insert is harmless outside a flood — the set
    // clears per TripWire).
    if skip_wires { wiring.skip_extend(&skips); }   // expose a pub(crate) extend or reuse self.skip via the methods
    true
}
```

Notes for the merger:
- The campfire's validate-kill drops: the method queues them into
  `self.actuation_drops` (the wire convention); d6's free fn took a `drops`
  out-param. Either drain `take_actuation_drops()` after the dispatch (the
  gun-stream caller spawns them on its stage stream) or add a drops
  out-param to the method — ONE choice, documented. The d6 witnesses
  (`w4_d6_the_campfire_flips_framey_and_a_broken_square_kills`) re-point
  their call args only.
- The `&mut Wiring` threading: d6's callers (`aiStyle-60` gun streams,
  `try_toggle_light` call sites in the projectile AI) hold `&mut World`
  only today; the wiring instance lives on `App`. The aiStyle-60 tile walk
  runs app-side, so the caller passes `&mut app.wiring` (the
  `update_logic_sensors(world, wiring, …)` precedent). If a core-side
  caller ever needs it, the fallback is the free-fn core + a thin method
  wrapper — but never three copies.
- The torch: neither c42/f11 nor the base port has a `toggle_torch`
  METHOD (the arm is inline ±66). d6's free fn is the only forced-gated
  copy — keep it as the shared torch body (free fn, both callers use it)
  or inline the gate into the arm; the 95 §4 verdict "one arm" holds
  either way.

### 1c. The `skip_wires=true` path (95 §8.5) — CLOSED structurally

With 1a+1b the parameter is honoured: `true` collects the per-static skip
cells and joins the flood's set; `false` discards. Every CURRENT caller
passes false (the gun streams), so no behaviour change lands with the
unification — the seam closes without a new witness requirement beyond
the re-pointed d6 four.

## 2. Never-ship-both reminder (95's list, this family)

`toggle_camp_fire`/`toggle_campfire` ×3 (c42/f11 methods + d6 free fn) →
ONE method (f11's landed signature is the template); the 8 base Toggle*
methods vs d6's free-fn copies → the methods win, the free fns drop;
`try_toggle_light` stays UNIQUE. The e20 skip-set witnesses are the only
skip-set pins for the 8 non-campfire statics — they MUST survive (1a).

*All C# anchors re-read at the cited lines in this worktree's
`decompiled/Terraria/` checkout. No cargo/tests/graphify run (code-only
lane); the f11 witness is WRITTEN NOT RUN on this branch.*
