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

## Delivered this phase

- `crates/tr-core/src/dust_table.rs` — `get_alpha_mouse_text` +
  `get_alpha_disco` (the G3/G4 closures, staged pure seams) + the module
  doc re-write + 3 witnesses (`w4_g13_get_alpha_157_the_mouse_text_pulse`,
  `w4_g13_get_alpha_327_night_draws_three_and_never_raises`,
  `w4_g13_draw_tint_census_pins`) WRITTEN NOT RUN.
- This doc + the ANALYSIS section + the PORT_MAP/ROADMAP/anchors rows.
- No sibling file touched (`dust_physics.rs` read-only; the seams live in
  main's `dust_table.rs`, empty-diff on every probed sibling).
