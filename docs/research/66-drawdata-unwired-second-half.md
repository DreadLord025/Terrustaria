# 66 — DrawData unwired-arm census, SECOND half + wiring (W4-g34)

> Wave 4 · researched 2026-11-21 · branch `task/W4-g34-drawdata-wire-2` @ base a725acb0 ·
> code-only mode (no builds/tests run; witnesses WRITTEN NOT RUN) · sources:
> `decompiled/Terraria/Main.cs`, `decompiled/Terraria/GameContent/Drawing/TileDrawing.cs`,
> `decompiled/Terraria/WorldGen.cs`, `crates/tr-core/src/critter_cage_anim.rs`,
> `crates/tr-core/src/drawdata_furniture.rs`, `crates/tr-core/src/drawdata.rs`,
> `crates/tr-app/src/drawdata_cases_a.rs`, `crates/tr-app/src/drawdata_cases_b.rs`,
> `crates/tr-app/src/render/tiles.rs`.

## Scope — the lane split

The d61–d64 DrawData case censuses (`61-drawdata-cases-1.md` … `64-drawdata-cases-4.md`,
on the `task/W4-d61…d64` branches) found **0 missing case arms** across all 371 ids; the
gaps are **wiring**, not arms. The PORTED-UNWIRED list is the 92 critter-cage ids whose
faithful `drawdata_furniture::cage_table` arms had no live caller (frozen at row 0 until
the cage animator + a `furniture_post_pass`-style call land).

The two wiring lanes split that list:

| Lane | Branch | Half | Ids |
| --- | --- | --- | --- |
| g33 | `task/W4-g33-drawdata-wire-1` | FIRST | the 41 big-cage ids + snail 285 (42) |
| **g34 (this doc)** | `task/W4-g34-drawdata-wire-2` | **SECOND** | **the 50 small-animal + water/jellyfish ids** |

The split is exactly the d64 census's batch boundary: **batch 2** (small-animal cage tail,
21 ids) + **batch 3** (water-animal + jellyfish cages, 29 ids) = 50. The big-cage group
(d63 batches 5–8 / d64 R16–R19) and snail 285 are g33's.

## The SECOND-half unwired-arm list (50 ids, now WIRED)

C# groups: small-animal `TileDrawing.cs:5756-5831` (probe `GetSmallAnimalCageFrame`
`:6535-6540`), water-animal `:5833-5897` (probe `GetWaterAnimalCageFrame` `:6528-6533`),
jellyfish `:5898-5907` (reads the SMALL probe). Port arms: `drawdata_furniture.rs::cage_table`
(the `SMALL`/`WATER` rows), applied by `cage_draw_data`.

### Batch 2 — small-animal cage tail (21 ids, stride 36, SMALL probe)

| Id(s) | C# inner arm (verbatim key line) | C# line | Rust arm (`cage_table`) | Was | Now |
| --- | --- | --- | --- | --- | --- |
| 286, 582 | `addFrY = Main.snail2CageFrame[smallAnimalCageFrame2] * 36;` | :5788-5791 | `286 \| 582 => (Snail2, 0, 36, SMALL)` | UNWIRED | WIRED |
| 298, 361 | `addFrY = Main.frogCageFrame[smallAnimalCageFrame2] * 36;` | :5792-5795 | `298 \| 361 => (Frog, 0, 36, SMALL)` | UNWIRED | WIRED |
| 339, 362 | `addFrY = Main.grasshopperCageFrame[smallAnimalCageFrame2] * 36;` | :5796-5799 | `339 \| 362 => (Grasshopper, 0, 36, SMALL)` | UNWIRED | WIRED |
| 299, 363 | `addFrY = Main.mouseCageFrame[smallAnimalCageFrame2] * 36;` | :5800-5803 | `299 \| 363 => (Mouse, 0, 36, SMALL)` | UNWIRED | WIRED |
| 310, 364, 391, 619 | `addFrY = Main.wormCageFrame[smallAnimalCageFrame2] * 36;` | :5804-5809 | `310 \| 364 \| 391 \| 619 => (Worm, 0, 36, SMALL)` | UNWIRED | WIRED |
| 392, 393, 394 | `addFrY = Main.slugCageFrame[typeCache - 392, smallAnimalCageFrame2] * 36;` | :5810-5814 | `392..=394 => (Slug, kind-392, 36, SMALL)` | UNWIRED | WIRED |
| 532 | `addFrY = Main.maggotCageFrame[smallAnimalCageFrame2] * 36;` | :5815-5817 | `532 => (Maggot, 0, 36, SMALL)` | UNWIRED | WIRED |
| 533 | `addFrY = Main.ratCageFrame[smallAnimalCageFrame2] * 36;` | :5818-5820 | `533 => (Rat, 0, 36, SMALL)` | UNWIRED | WIRED |
| 538, 544, 629 | `addFrY = Main.ladybugCageFrame[smallAnimalCageFrame2] * 36;` | :5821-5825 | `538 \| 544 \| 629 => (Ladybug, 0, 36, SMALL)` | UNWIRED | WIRED |
| 555, 556 | `addFrY = Main.waterStriderCageFrame[smallAnimalCageFrame2] * 36;` | :5826-5829 | `555 \| 556 => (WaterStrider, 0, 36, SMALL)` | UNWIRED | WIRED |

### Batch 3 — water-animal + jellyfish cages (29 ids, stride 36)

| Id(s) | C# inner arm (verbatim key line) | C# line | Rust arm (`cage_table`) | Probe | Was | Now |
| --- | --- | --- | --- | --- | --- | --- |
| 282, 505, 543 | `addFrY = Main.fishBowlFrame[waterAnimalCageFrame5] * 36;` | :5837-5840 | `282 \| 505 \| 543 => (FishBowl, 0, 36, WATER)` | WATER | UNWIRED | WIRED |
| 598 | `addFrY = Main.lavaFishBowlFrame[waterAnimalCageFrame4] * 36;` | :5845-5848 | `598 => (LavaFishBowl, 0, 36, WATER)` | WATER | UNWIRED | WIRED |
| 568, 569, 570 | `addFrY = Main.fairyJarFrame[waterAnimalCageFrame3] * 36;` | :5855-5858 | `568..=570 => (FairyJar, 0, 36, WATER)` | WATER | UNWIRED | WIRED |
| 288-295 | `addFrY = Main.butterflyCageFrame[num8, waterAnimalCageFrame2] * 36;` (`num8 = typeCache - 288`) | :5873-5881 | `288..=295 => (Butterfly, kind-288, 36, WATER)` | WATER | UNWIRED | WIRED |
| 360, 580, 620 | same butterfly body, `num8 = 8` override | :5877-5880 | `360 \| 580 \| 620 => (Butterfly, 8, 36, WATER)` | WATER | UNWIRED | WIRED |
| 521-527 | `addFrY = Main.dragonflyJarFrame[typeCache - 521, waterAnimalCageFrame] * 36;` | :5892-5895 | `521..=527 => (Dragonfly, kind-521, 36, WATER)` | WATER | UNWIRED | WIRED |
| 316, 317, 318 | `addFrY = Main.jellyfishCageFrame[typeCache - 316, smallAnimalCageFrame] * 36;` | :5902-5905 | `316..=318 => (Jellyfish, kind-316, 36, SMALL)` | **SMALL** | UNWIRED | WIRED |

The jellyfish group reads the **SMALL** probe (`/3 * /3`), not the water probe — the port
carries that (`SMALL` flag). The 360/580/620 → row 8 override is the "jarred butterfly
variants share the eighth row" quirk, ported as the constant row.

## What was missing (the wiring, not the arms)

The d64 batch 2/3 rationale (verbatim from the census): `furniture_post_pass` had **zero
callers** outside its own tests; the static fallback arm in `drawdata.rs` set only `top = 2`
and left `add_fr_y = tile_frame * 38`. Cage kinds are absent from `Main.AnimateTiles`
(they animate through `AnimateTiles_CritterCages`, `Main.cs:13511`, which moves the
`*CageFrame` tables, not `tileFrame`), so `tile_frame ≡ 0` and the fallback drew row 0 —
pixel-identical to the C# t=0 state but **frozen**. Two pieces were missing:

1. **The animator** — `Main.AnimateTiles_CritterCages` (`Main.cs:13511-16596`) + the
   `UpdateOwlCageFrames`/`UpdateDragonflyJarFrames` helpers (`:16598-16727`).
2. **A live draw call** — the cage rewrite applied after the drawdata dispatch, reading the
   animator's tables.

## What landed (W4-g34)

### 1. The animator — `crates/tr-core/src/critter_cage_anim.rs` (NEW)

`CritterCageAnimator` ports `AnimateTiles_CritterCages` **whole** — all 34 table machines in
C# source order (`:13517-16595`), the `critterCage` gate (`:13513-13516`), the owl/dragonfly
helpers, the mode bytes (`fishBowlFrameMode`, `butterflyCageMode`, `jellyfishCageMode`), and
the 2-D tables (scorpion 2, slug 3, butterfly 9, dragonfly 7, jellyfish 3). Every machine
draws from the caller's `UnifiedRandom` (the exact `Main.rand` Knuth subtractive stream) in
source order.

**Why whole, not halved:** the machines share ONE RNG stream. g33's first-half tables
(bunny … snail) draw RNG *before* g34's second-half tables (snail2 … jellyfish); porting only
the second half would desynchronise every downstream draw. The animator is therefore shared
infrastructure; the **draw wiring** is what's halved (g34 wires only `is_lane2_cage_kind`).

- `frame_of(table, row, idx)` — the lookup `cage_draw_data` consumes (out-of-range probe → 0).
- `anim_key()` — FNV-1a over every frame/counter/mode value (the G6 convention).
- `warm_up()` — `Initialize_TileAndNPCData2`'s 3600 forced ticks (`Main.cs:6802-6807`),
  exposed but **NOT wired into App init** (it needs the full `Initialize_TileAndNPCData2`
  RNG-order census; documented deviation below).
- 12 witnesses WRITTEN NOT RUN (zero-state, gate, determinism, fairy-jar derivation,
  lava-bowl wrap, slug wraps, jellyfish/butterfly/dragonfly/fish-bowl bands, `frame_of`
  table map, warm-up equivalence).

### 2. The draw seam — `render/tiles.rs::draw_tiles`

- New params `cage_anim: &CritterCageAnimator`, `critter_cage: &mut bool`.
- After `apply_cases_a`, for `is_lane2_cage_kind(tile.kind)` the rewrite
  `cage_draw_data(..., |t,r,i| cage_anim.frame_of(t,r,i))` replaces the static fallback
  (`top = 2` + `addFrY = table[idx] * stride`). The big-cage ids + snail 285 stay g33's;
  the chest rewrite stays with the chest lane (W4-d58).
- `Main.critterCage` (`Main.cs:1445`): cleared at the pass head (`TileDrawing.cs:474`),
  raised for **every** cage cell (`is_cage_kind`, `:5672` etc.) — the flag gates the whole
  animator, not just this half.

### 3. The tick + G6 key

- `App::animate_tiles_phase` (`app/frame/world.rs`) ticks `cage_anim.animate(critter_cage, rng)`
  beside `tile_anim.animate_tiles` on the shared 60 Hz accumulator + `Main.rand` stream
  (the C# calls `AnimateTiles_CritterCages` at the tail of `AnimateTiles`, `Main.cs:19303`).
- `App` gains `cage_anim` + `critter_cage` fields (both constructors + the world-load reset).
- The G6 `WorldFrameKey::tile_anim` (`app/frame/draw.rs`) gains a cage-gated contribution:
  none of the 50 lane-2 ids is in `TileAnimator::is_animated_kind`'s gate set (they animate
  through the separate cage tables), so a lane-2 cage active in the view rect mixes
  `cage_anim.anim_key()` into the key; a cage-free view keeps the pre-wiring key bit-for-bit.

### 4. Predicates — `drawdata_furniture.rs`

- `is_cage_kind(kind)` — any of the 92 cage ids (the `critterCage` flag).
- `is_lane2_cage_kind(kind)` — exactly the 50 second-half ids (the draw-wiring gate).
- Witness `lane2_cage_split_is_disjoint_and_exhaustive`: 92 = 50 (g34) + 42 (g33), disjoint.

## DEVIATIONS fixed (the census-flagged draw-math gaps)

| # | Deviation (census cite) | C# | Fix |
| --- | --- | --- | --- |
| 1 | case-4 `solid_above` probe dropped halfBrick/slope/inactive/null clauses (d61 probe note / d64 R3) | `WorldGen.SolidTile` `WorldGen.cs:70111-70132`, case 4 `TileDrawing.cs:4672-4679` | `render/tiles.rs` now feeds the full predicate via `drawdata_cases_a::solid_tile` (made `pub`); the old three-clause subset is gone. Witness `solid_tile_is_the_full_worldgen_predicate`. |
| 2 | 564 glow rect read post-clear `addFrY` (0) instead of pre-clear `tileFrame*36` (d63 batch 4 / d64 R15) | `TileDrawing.cs:6105-6113` (rect :6109 reads pre-clear, :6112 clears) | `drawdata.rs`'s 564 arm keeps `tileFrame*36`; the `addFrY = 0` clear rides `drawdata_cases_a`'s `add_fr_y_delta` AFTER the rect is built (the C# order). Witnesses updated in both modules. |
| 3 | 441/468 first-switch `GetTemporaryFrame` Y rewrite unscaffolded (d63 summary / d64 R13) | `TileDrawing.cs:5342-5356` | `drawdata_cases_b` gains the `441 \| 468` probe arm (`38 * f + num44` `frame_y_override`, the 349 pattern); behaviour-neutral while the registry is empty. `OWNED_KINDS` 16 → 18. Slice A keeps their second-switch glow rows. |
| 4 | stale pre-wiring comments: 455 "always false", 428 "not pressed" (d63 summary item 4) | — | `drawdata.rs` comments swept: the static table carries the party-down / not-pressed state, the live `drawdata_cases_b` arm feeds the real flags. |

## Residual deviations (NOT fixed here — other lanes)

- **Glow bind** — every glow record still rides `_glow` (`tiles.rs`, TILE-DRAW-2 / g35).
  The 564 rect fix above is correct *data*; the bind is g35's.
- **711 RainbowBoulder first-switch glow** — fixed on the `task/W4-d62` branch
  (`TILE_SHEET_GLOW` sentinel), not yet on main; not re-litigated here.
- **637 painted-sheet / 184 col-220 disco colour** — engine state unrouted (d61/d62), the
  rects port; the sheet/colour bind is the glow lane's.
- **185/186 `Main.tileShine2`** — sparkle lane (d64 R10), not DrawData.
- **`martianLight` fed 0 / `mouseTextColor` fed 255** — event/UI state not simulated
  (documented `cases_a.rs`); the glow colours take the values so the events can feed them.
- **Cage animator warm-up** — `warm_up` is ported but not wired into App init: the C#
  `Initialize_TileAndNPCData2` (`Main.cs:6800-6807`) runs 3600 forced cage ticks at startup
  amid other init RNG draws; wiring it needs that whole init's RNG-order census. Until then
  the cages start at the zero state (the vanilla t=0 draw) and animate from the first tick —
  a startup-phase deviation, not a steady-state one.
- **Temporary-animation registry** — `Animation._temporaryAnimations` (`Animation.cs:186-196`)
  still unported; `temporary_frame` returns `None`. The 349/593/594/441/468 probe arms are
  scaffolded around it (the 441/468 half is new here).

## Verification

Code-only mode: **no cargo/tests run**. Witnesses are WRITTEN NOT RUN (the repo's code-only
convention, the `npc_ai_*` precedent). The animator's machines were ported against the C#
verbatim in ≤100-line windows; the tricky quirks are pinned in comments and witnesses:
- penguin `:15166` compares `> 3` (not `> 11`) in the 10-11 band — faithful.
- frog `:15711` compares `> 17` (not `> 23`) in the 18-23 band — faithful.
- fish-bowl mode-0 arm `:16332` rolls `Next(12)` (not `Next(20)`) and does NOT clear the mode.
- owl `:16623` `(counter+1)%40==39` reseed quirk — faithful.
- snail/snail2 double `== 1 && Next(3)` clauses (`:15490-15497` / `:15563-15570`) — both kept.
- butterfly `>= 10` mode walks rows backward with the ±8 landmark hops — faithful.

## Commits

1. `Port AnimateTiles_CritterCages whole …` — the animator module + 12 witnesses.
2. `Wire lane-2 cage draws (50 ids …)` — the draw seam + tick + G6 key + predicates + 3 witnesses.
3. `Fix census-flagged draw-math deviations …` — the 4 deviation fixes + witness updates.