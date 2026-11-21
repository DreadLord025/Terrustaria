# 70 — Dust DRAW-tint census FINAL + the GetAlpha 157/327 drop closure (W4-g13 final phase; d54 research/131 §7.2 G3/G4)

Status: DONE (W4-g13 final phase, code-only — two seam fns + 3 witnesses
WRITTEN NOT RUN in `dust_table.rs`; no cargo/rustc/rustfmt/tests/graphify
invoked). Base `6a2f6830` (the research/69 §6 strip-contract receipt).
Probed tips: d54 `c89aea9f` (research/131 §7.2 FINAL gap-table), d50
`2e243a2d` (UNCHANGED, research/69 §6), g67 `30b39fceb` (the DrawDust
cull `2c452edd` is an ancestor — `merge-base --is-ancestor` exit 0).

The task brief's "d54g-G3/G4-157/327-drops co-lane": **no `task/W4-d54g`
branch exists on origin** (`git ls-remote --heads` census) and both
`git diff origin/main origin/task/W4-d50-dust-type-table -- dust_table.rs`
and the same vs d54 are EMPTY — the drops were UNCLAIMED. This lane
claims and closes them on ITS OWN reachable file (`dust_table.rs` is
main's S1-E-2 file, untouched by every in-flight sibling).

## 1. The dust-DRAW-tint census — every consumer of GetColor/GetAlpha

C# `Dust.GetColor` (`Dust.cs:2729-2772`) and `GetAlpha` (`:2425-2727`)
have exactly these draw-side consumers (full `Main.cs` `dust.GetAlpha`
grep, 8 hits, all inside `DrawDust` `:38828-39027`; `GetColor` likewise):

| Consumer | C# | Tint shape | Port status |
| -------- | -- | ---------- | ----------- |
| 130-134 trail | `:38864-38878` | per-echo `GetAlpha(light@(pos+4)/16)`, scale ×(1−j/10), j < min((|vx|+|vy|)*3, 10) | dust DRAW lane remainder (research/68 §2) — the pure tint arms are table-complete (25-alpha family `:2584`, 131's lerp) |
| 278 trail | `:38880-38895` | same shape, `(int)(pos+4f)/16` float cast (NOT the double form) | same lane; the `mul_rgb` arm `:2588` table-complete |
| 219-223 trail | `:38897-38911` | `fadeIn == 0` gate, then the same echo loop | same lane; the halfway-white lerp `:2594` table-complete |
| 264 trail | `:38913-38929` | ×10 speed (not ×3), origin (5,5), base `GetAlpha(light)*0.3f` THEN a second sprite `GetColor(base)` | same lane; the 263/264 arm `:2486` + GetColor table-complete, the composition pinned by witness `w4_g13_draw_tint_census_pins` |
| 226/272 trail | `:38931-38950` | `fadeIn == 0` gate; `GetColor` ONLY when `customData is Color` (`:38944`) | same lane; customData dropped port-wide (research/131 G5) → the tint is GetAlpha alone, cited |
| 331 trail + cross | `:38952-39001` | trail tint `GetColor(GetAlpha(WHITE))` ONCE outside the loop (`:38959`); the 4-way ±2 cross draws `GetAlpha(dust.color)` (NOT light) + a WHITE centre | same lane; composition pinned by the census witness |
| generic base | `:38968-39005` | light@(pos+4)/16, WHITE override for 6/15/59-64 (`:38969-38972`), `GetAlpha(newColor)`, 213 scale-force 1 (`:38974`) | `minecart::draw` (213, WIRED) + `sand::draw` (268, WIRED) cover the live types; the WHITE override pinned by the census witness |
| generic overlay | `:39006-39013` | `color.PackedValue != 0` gate → `GetColor(newColor)` → second sprite unless packed 0 | `sand::draw`'s overlay + packed-0 skip ported (`channel_overlay`); the packed-0 gate pinned |
| black death | `:39015-39017` | `newColor == Color.Black` (POST-GetAlpha) deactivates | `sand::draw` kills pre-tint on zero light — equivalent for 268 (GetAlpha at alpha 0 is the identity); the all-type form is the DRAW lane's |
| cull + else-kill | `:38831-38847`, `:39020-39022` | the wide ±1000/±1050 box (the `:38832` overwrite), 4×4 strict-< test, trail types restore the SAME wide rect, miss ⇒ deactivate | **g67 PORTED** `2c452edd` (`rain.rs::splash_cull_rect`/`dust_in_cull_rect` + the else-kill, ancestor of tip `30b39fceb`; research/134 R3 carries the d130 fold) |

**Verdict: the SPRITE-tint column is table-complete port-wide.** Every
per-type arm of GetAlpha/GetColor is in `dust_table.rs` (S1-E-2's 123-type
mechanical diff, research/131 §7.2 — only 327 was absent, 157 documented);
the two drops close BELOW; the remaining work is the generic all-type
draw WALK (the trail loops + slot order + shader batches), which is the
dust DRAW lane's remainder (research/68 §2), not a tint gap.

## 2. G3 — GetAlpha type 157 (`Dust.cs:2627-2651`) CLOSED

The drop's blocker was "`Main.mouseTextColor` has no port equivalent".
It now does: `App.mouse_text_color` (`app/mod.rs`, the
`DoUpdate_AnimateCursorColors` pulse ported in `app/frame/draw.rs:328-337`,
190..255 band, init 225 `flow.rs:2403`).

Ported as **`dust_table::get_alpha_mouse_text(d, new_color, mouse_text_color: u8)`**
— the pure fn stays global-free; the DRAW caller passes the pulse byte:

- `:2630` `num7 = (float)(int)mouseTextColor / 100f - 1.6f` — the f32 op.
- `:2631-2649` the three channels: `(int)(255*num7) + 50`, cap 255 each.
- `:2634` alpha `(int)(100f*num7)` **UNCAPPED** — at the pulse floor 190,
  f32 `1.9−1.6 = 0.29999995` truncates the alpha to **29** (not 30) while
  the channel is `(int)76.49999+50 = 126`; at the ceiling 255, f32
  `2.55−1.6 = 0.94999993` gives alpha **94** (not 95). Both boundaries
  pinned by `w4_g13_get_alpha_157_the_mouse_text_pulse` (f32 simulation
  cross-checked with Python struct-pack).
- fullBright precedes the arm (`:2427`); non-157 delegates to `get_alpha`.

## 3. G4 — GetAlpha type 327 (`Dust.cs:2658-2676`) CLOSED, the dead-raise quirk

The drop's blockers were `Main.dayTime`, `Main.Disco*` and the DRAW-pass
`Main.rand` placement. All three now have port equivalents:
`world.day_time`, `App.disco` (`[u8; 3]`, the `DoUpdate_AnimateDiscoRGB`
cycle ported `app/mod.rs:1498`/`frame.rs:205`; C# ints clamped 0..255,
`Main.cs:19591`), and the `DustRng` trait's `next_float` (`particles.rs:45`).

Ported as **`dust_table::get_alpha_disco(d, new_color, day_time, disco: [u8; 3], rng)`**:

- `:2658` gate `type == 327 && !Main.dayTime` — day/non-327 delegate to
  `get_alpha` with **ZERO draws** (the short-circuit precedes the first
  NextFloat). fullBright precedes the gate.
- `:2660-2662` THREE `NextFloat()` draws, R/G/B order.
- **The dead-raise quirk (the census's key finding):** the C# reads
  `(int)(NextFloat()*0.04f + 0.1f + Disco/800f) * 255` — the truncation is
  INSIDE, the `*255` OUTSIDE. The inner f ∈ [0.1, 0.45875) is always < 1,
  so every `(int)` is 0, every raise is 0, and no `>` comparison ever
  fires: **the arm can never change the colour**. Its only observable
  effect is the three stream draws per night draw of a live 327. The port
  keeps the expression verbatim (not folded to a constant), so a future
  C# re-read can diff it arm-for-arm.
- Witness `w4_g13_get_alpha_327_night_draws_three_and_never_raises` pins:
  3 draws at night (max disco + max sample), colour passthrough, 0 draws
  on day / non-327 / fullBright, 3 draws at zero disco.

**Draw-pass RNG placement (the wiring contract, research/131 G4's "needs
its own census"):** the 327 draws fire in `DrawDust`'s slot walk
(`Main.cs:38836`, `i < maxDustToDraw`) at the GENERIC tint site `:38973`
— 327 has no trail arm, so exactly one GetAlpha call per live 327 per
draw, AFTER the gore draw (`:62282`) and BEFORE the `DrawnThisUpdate`
end-of-Draw set (`:62478`, g13's relocation). The stream is the bare
`Main.rand` slot (`rngs.global()`), the same slot g13's draw feeds ride;
relative order vs the item sparkle (`:38664`, inside DrawItems BEFORE
DrawDust) and the retro liquid pair (`:57789`, DrawWaters before both) is
already fixed by the C# call chain — the DRAW lane wires
`get_alpha_disco` into its generic walk at the `:38973` position and the
stream order holds by construction. Until that lane wires, no live code
calls the seam (staged, `get_alpha`'s fall-through unchanged — zero
behavioural delta on this branch).

## 4. Co-lane close-out for d54f-131 master

research/131 §7.2's gap-table rows G3/G4 flip to CLOSED (this doc +
`dust_table.rs` at this branch's tip). G1 (the 52 lights) closed by
`a234107a` (research/69). G2 (the 76-arm snowDust++ seam) stays d50×d130's
resolve. G5/G6 are closure rows, unchanged. **The Dust.cs port-wide
coverage now has NO open tint/physics/light gap — only wiring seams.**

## 5. FINAL cross-references (the three-doc closure set)

- research/68 §5 — the Main.cs dust-family COMPLETE declaration (the
  spawn census + non-spawn machinery + the sibling re-probe at the final
  tips g11 `9a9ac1fd` / g12 `f24dd76a`).
- research/69 §6-7 — the d50 strip-contract receipt (tip `2e243a2d`
  UNCHANGED, §4 stands) + the 94/94 light-family closure.
- This doc — the tint column table-complete + G3/G4 CLOSED.
- **For the d54f-131 master (research/131 §7.2): G1 CLOSED (`a234107a`),
  G3 CLOSED + G4 CLOSED (`daf7b780`, this doc); G2 stays the d50×d130
  resolve; G5/G6 closure rows unchanged. No open port gap remains in the
  Dust.cs / Main.cs dust family — only wiring seams.**

## Delivered this phase

- `crates/tr-core/src/dust_table.rs` — `get_alpha_mouse_text` +
  `get_alpha_disco` (the G3/G4 closures, staged pure seams) + the module
  doc re-write + 3 witnesses (`w4_g13_get_alpha_157_the_mouse_text_pulse`,
  `w4_g13_get_alpha_327_night_draws_three_and_never_raises`,
  `w4_g13_draw_tint_census_pins`) WRITTEN NOT RUN.
- This doc + the ANALYSIS section + the PORT_MAP/ROADMAP/anchors rows.
- No sibling file touched (`dust_physics.rs` read-only; the seams live in
  main's `dust_table.rs`, empty-diff on every probed sibling).

## 6. RETIRE keep-one — the G3/G4 SAME-FILE collision with d54 (dedupe table for the d54i-bible)

**Post-FINAL re-probe (2026-11-21, retire phase).** d54's own gap-closure
landed the SAME two arms AFTER this branch's FINAL declaration: tip
`1d16ef4e` (was `c89aea9f` at the final phase) carries
`DustDrawGlobals` + `get_alpha_with_globals` in the SAME file
`crates/tr-core/src/dust_table.rs` — a textual-merge-collision on BOTH
branches. `daf7b780` (this branch's two-seam closure) is NOT an ancestor
of d54's tip (`merge-base --is-ancestor` exit 1); the merge-base of the
two branches is `a725acb04`.

**Keep-one decision: d54-canonical.** d54's single-seam shape folds both
arms into ONE entry at their exact C# positions (one call site for the
DRAW lane instead of two type-routed seams) and owns the gap-table rows
(research/131 §7.2). This branch RETIRES `get_alpha_mouse_text` +
`get_alpha_disco` and takes the d54 blob `58305d0a8` VERBATIM. Verified:
`git diff --no-index <d54-file> <this-file>` has **ZERO deletions** — every
d54-canonical item is byte-identical; the only adds are the module
keep-one note and the g13 census witnesses below. Behaviour identical:
same 157 maths (`/100−1.6`, +50, cap 255, uncapped alpha; 190→a29 /
255→a94), same 327 gate + THREE R/G/B draws + dead-raise quirk
(`(int)(sum)*255`, sum < 1); the disco field widens `[u8;3]`→`[i32;3]`
(the C# `Main.DiscoR/G/B` ARE ints, `Main.cs:959-963` — the port's
`App.disco` bytes feed via `as i32`).

**Per-fn dedupe table (the d54i-bible merger contract):**

| Item | d54 `1d16ef4e` | g13 retire tip | Merger resolution |
| ---- | -------------- | -------------- | ----------------- |
| module doc | canonical | canonical + keep-one note | take g13 (superset) |
| `DustDrawGlobals` | canonical | byte-identical | keep one |
| `get_alpha` (+ the G3/G4 comment) | canonical | byte-identical | keep one |
| `get_alpha_with_globals` | canonical | byte-identical | keep one |
| `get_alpha_mouse_text` / `get_alpha_disco` | absent | RETIRED (absent) | keep absence |
| `w4_d54_g3_get_alpha_157_mouse_text_pulse` | canonical | byte-identical | keep one |
| `w4_d54_g4_get_alpha_327_night_draws_and_zero_quirk` | canonical | byte-identical | keep one |
| `w4_g13_get_alpha_157_the_mouse_text_pulse` / `w4_g13_get_alpha_327_night_draws_three_and_never_raises` | absent | RETIRED (superseded by the d54 witnesses) | keep absence |
| `FloatLog` rig | absent | g13-only add (feeds the census witness) | take g13 |
| `w4_g13_globals_seam_gate_census` | absent | g13-only add (non-327/fullBright 0-draw + zero-disco 3-draw pins over the canonical seam) | take g13 |
| `w4_g13_draw_tint_census_pins` | absent | g13-only add (§1 composition pins) | take g13 |

The anchors row re-points at `pub fn get_alpha_with_globals` (the retired
`pub fn get_alpha_disco` marker would fail the canary). §2-3 above stay as
the HISTORICAL closure record; the live seam is the canonical one.

## 7. RETIRE — the DrawDust-tint wiring-contract (exact wire-hunks for the draw lane)

Probed at g67 tip `30b39fce` (the cull+else-kill `2c452edd` is an ancestor,
`merge-base --is-ancestor` exit 0; research/134 R3 carries the d130 fold).
The draw lane's live generic-path consumers today are g67's
`rain::draw_splashes` (the `dustWater()` family), `sand::draw` (268) and
`minecart::draw` (213). The 8-consumer census (§1) maps onto them as
follows; every hunk below is the EXACT edit the draw lane applies.

### 7.1 The tint-feed compose (the `:38973` wire)

The C# `:38968-38973` sequence per live slot: light sample at
`(int)((double)pos + 4.0) / 16` → the 6/15/59-64 WHITE override →
`GetAlpha(newColor)`. The canonical seam (§6) folds the 157/327 arms in at
their exact positions, so ONE call replaces `get_alpha` at every generic
tint site — no type routing at the caller.

**Hunk W1 — the globals sample, once per draw pass** (`app/frame/draw.rs`,
immediately before the world-effect dust block — after the gore draw
`:1369-1373`, before `sand::draw` `:1376`; the g67 union inserts
`draw_splashes` between `:1380` and the foliage draw):

```rust
// `Main.DrawDust`'s globals (research/70 §7): sampled once per pass.
// mouseTextColor is the DoUpdate_AnimateCursorColors pulse already
// ticked in this frame's update; dayTime is the world clock; Disco are
// the DoUpdate_AnimateDiscoRGB bytes (C# ints clamped 0..=255).
let dust_globals = tr_core::dust_table::DustDrawGlobals {
    mouse_text_color: self.mouse_text_color,
    day_time: world.day_time,
    disco_rgb: [
        self.disco[0] as i32,
        self.disco[1] as i32,
        self.disco[2] as i32,
    ],
};
```

**Hunk W2 — `rain::draw_splashes` signature + tint call** (g67
`rain.rs:1463-1494`): add two params after `screen: Vec2`:

```rust
    globals: &tr_core::dust_table::DustDrawGlobals,
    rng: &mut impl tr_core::particles::DustRng,
```

and replace `:1494`

```rust
        let tint = tr_core::dust_table::get_alpha(&d, new_color);
```

with

```rust
        // `:38969-38973`: the WHITE override for 6/15/59-64, then the
        // globals seam (research/70 §7 — the 157/327 arms ride here).
        let new_color = if matches!(d.dust_type, 6 | 15 | 59..=64) {
            tr_core::particles::Color::WHITE
        } else {
            new_color
        };
        let tint =
            tr_core::dust_table::get_alpha_with_globals(&d, new_color, globals, rng);
```

(the splash family is 33/98-105/123/288 — the override and the 157/327
arms never fire for it today; the wire is behaviour-identical and keeps
the entry exact for the family's future spawners, the `:39006` overlay
precedent).

**Hunk W3 — the call site** (`app/frame/draw.rs`, g67's `:1295-1297`):

```rust
                    crate::rain::draw_splashes(
                        dust, renderer, lookups, light, draw_cam, world_zoom, screen,
                        &dust_globals, self.rngs.global(),
                    );
```

`rngs.global()` is the bare `Main.rand` slot — the same slot the g13
spawn feeds ride (`:1249`), so the 327 draws land in the C# stream
position by construction: AFTER the item sparkle (`:38664`, DrawItems
before DrawDust) and the retro liquid pair (`:57789`, DrawWaters before
both), BEFORE the end-of-Draw `DrawnThisUpdate` set (`:62478`, g13's
relocation — the mark reads clear through the draw, `:1240`).

**Hunk W4 — the generic all-type walk** (the DRAW lane's remainder,
research/68 §2): the same two-line tint compose at the `:38973` position
of its slot loop, `globals` sampled once (W1) and `rng` threaded as
`&mut` through the walk. The trail arms (§1 rows 1-6) call `get_alpha`
pure — 157/327 match no trail type, so no trail hunk needs the seam; the
331 cross (`:38999`) reads `GetAlpha(dust.color)` — also pure for every
type that reaches it. The 213 scale-force (`:38974`) and the black-death
(`:39015`) stay as ported.

### 7.2 The cull+else-kill compose (g67-owns, already landed)

`splash_cull_rect`/`dust_in_cull_rect` (`rain.rs:1561/1573`) + the
`:1481-1484` else-kill ARE the `:38832/:38847/:39020-39022` port — the
draw lane does NOT re-port them; the generic walk reuses the two helpers
(one wide rect for the whole family, the `:38845` restore is a no-op).
The tint-feed (W1-W4) composes INSIDE the cull-survivor branch, exactly
where `draw_splashes` already sits.

### 7.3 The per-family stand-ins (no wire owed)

`sand::draw` (268) and `minecart::draw` (213) read the table through
identity-equivalent short forms (268 at alpha 0: GetAlpha is the light
itself; 213's arm ignores the light) — neither type is 157/327 nor
6/15/59-64, so the seam swap is a no-op for them; they fold into the
generic walk (W4) when the draw lane lands it, not before. Until W1-W4
land, no live code calls the seam — staged, zero behavioural delta.
