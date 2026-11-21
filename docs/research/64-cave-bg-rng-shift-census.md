# 64 — `RandomizeCaveBackgrounds` worldgen RNG-shift census (W4-g72 lane 3, phase 6)

> **SEALED (W4-g72 retire, 2026-11-21).** The census verdict below was
> superseded in-lane (phases 7-8 wired the roll); §6's order-contract was
> CONSUMED by g71 (`fc76a48b3` composed the tail); §7 is the keep-one
> receipt — OUR `cave_bg.rs` is canonical, g71 dropped its duplicate. No
> further edits owed here; the BG-FAMILY bible lives in docs/ANALYSIS.md
> "The BG-FAMILY bible".

**Code-only census. NO wiring** — inserting these draws moves the frozen
`world_hash` pins (`worldgen/world_hash.rs` `SEED42_512` / `DEFAULT_WORLD`), so
the roll is a **worldgen lane's call**, not this one (the lane-3 style machine
`render/bg_styles.rs` consumes the rolls but does not own them). This note
documents the **exact stream-delta** so the worldgen lane can re-pin in one pass.

Source of truth: `decompiled/Terraria/WorldGen.cs:8192-8249`
(`RandomizeCaveBackgrounds`), its two call sites, and the port's
`crates/tr-core/src/worldgen/pipeline.rs:2685-2731` (`step_world_tail`).

---

## 1. The C# function — every draw, in order

`WorldGen.genRand => Main.rand` (`WorldGen.cs:4445`) — ONE shared stream.
`maxValue = 8` (`:8194`). Three width arms (`maxTilesX == 4200` / `== 6400` /
else), then a common tail.

### Arm A — small (`maxTilesX == 4200`, `:8195-8206`)
| # | draw | line | notes |
|---|------|------|-------|
| 1 | `caveBackX[0] = Next(1050, 3150)` | 8197 | `(int)(4200*0.5 - 4200*0.25)=1050`, `(int)(4200*0.5 + 4200*0.25)=3150` |
| — | `caveBackX[1] = caveBackX[2] = maxTilesX` | 8198-8199 | NO draw |
| 2 | `caveBackStyle[0] = Next(8)` | 8200 | |
| 3 | `caveBackStyle[1] = Next(8)` | 8201 | |
| 3+k | `while (style[1]==style[0]) style[1]=Next(8)` | 8202-8205 | rejection, k≥0 extra draws |

**Arm A total: 3 + k draws** (k = the style[1] rejection count).

### Arm B — medium (`maxTilesX == 6400`, `:8207-8223`)
| # | draw | line | notes |
|---|------|------|-------|
| 1 | `caveBackX[0] = Next(857, 3417)` | 8209 | `(int)(6400*0.334 ∓ 6400*0.2)`: 2137.6∓1280 → 857 / 3417 (truncate) |
| 2 | `caveBackX[1] = Next(2988, 5548)` | 8210 | `(int)(6400*0.667 ∓ 1280)`: 4268.8∓1280 → 2988 / 5548 |
| — | `caveBackX[2] = maxTilesX` | 8211 | NO draw |
| 3 | `caveBackStyle[0] = Next(8)` | 8212 | |
| 4 | `caveBackStyle[1] = Next(8)` | 8213 | |
| 5 | `caveBackStyle[2] = Next(8)` | 8214 | |
| +k1 | `while (style[1]==style[0]) …` | 8215-8218 | |
| +k2 | `while (style[2]==style[0]‖style[1]) …` | 8219-8222 | |

**Arm B total: 5 + k1 + k2 draws.**

### Arm C — large / else (`:8224-8245`)
| # | draw | line | notes |
|---|------|------|-------|
| 1 | `caveBackX[0] = Next((int)(W*0.25 - W*0.15), (int)(W*0.25 + W*0.15))` | 8226 | W = maxTilesX |
| 2 | `caveBackX[1] = Next((int)(W*0.5 - W*0.15), (int)(W*0.5 + W*0.15))` | 8227 | |
| 3 | `caveBackX[2] = Next((int)(W*0.75 - W*0.15), (int)(W*0.75 + W*0.15))` | 8228 | |
| 4 | `caveBackStyle[0] = Next(8)` | 8229 | |
| 5 | `caveBackStyle[1] = Next(8)` | 8230 | |
| 6 | `caveBackStyle[2] = Next(8)` | 8231 | |
| 7 | `caveBackStyle[3] = Next(8)` | 8232 | |
| +k1 | `while (style[1]==style[0]) …` | 8233-8236 | |
| +k2 | `while (style[2]==style[0]‖style[1]) …` | 8237-8240 | |
| +k3 | `while (style[3]==style[0]‖style[1]‖style[2]) …` | 8241-8244 | |

**Arm C total: 7 + k1 + k2 + k3 draws.**

### Common tail (all arms, `:8246-8248`)
| # | draw | line |
|---|------|------|
| +1 | `iceBackStyle = Next(4)` | 8246 |
| +1 | `hellBackStyle = Next(3)` | 8247 |
| +1 | `jungleBackStyle = Next(2)` | 8248 |

**Grand totals (base, before rejection): A = 6, B = 8, C = 10.** With rejection
the counts are data-dependent (the `while` loops re-draw until distinct).

> The port's sub-vanilla test worlds (512×512, `GenOptions` default 4200×1200)
> take **Arm C** (512 ≠ 4200/6400) and **Arm A** (4200) respectively. The
> `worldgen_rolls_hell_back_style` witness (`render/bg.rs:1751`) uses 512×512 →
> Arm C.

---

## 2. The two call sites — and the boot re-roll

1. **Worldgen tail** — `WorldGen.cs:11339`, inside `GenerateWorld`'s setup:
   ```
   11337  Main.ActiveWorldFileData.WorldId = genRand.Next(int.MaxValue);
   11338  WorldGen.RandomizeTreeStyle();          // genRand, width-arm draws
   11339  WorldGen.RandomizeCaveBackgrounds();    // <-- this census
   11340  WorldGen.RandomizeBackgrounds(genRand); // surface BG rolls
   11341  WorldGen.RandomizeMoonState(genRand);   // moonType = Next(9)
   11342  WorldGen.TreeTops.CopyExistingWorldInfoForWorldGeneration();
   11343  WorldGen.GenerateWorld_SetupDungeonGenVars(0, clearOld: true);
   11344  dungeonSide = genRand.Next(2) …         // then jungleOriginX etc.
   ```
   So in C# the cave rolls sit **between** `RandomizeTreeStyle` and
   `RandomizeBackgrounds`, and **before** `RandomizeMoonState`'s `Next(9)`.

2. **Boot / world load** — `Main.cs:6603` (`Initialize_AlmostEverything`):
   ```
   6594  WorldGen.RandomizeBackgrounds(Main.rand);
   6595-6602  if (treeBGSet1[0]==173) RandomizeBackgrounds(Main.rand);  // ≤2 retries
   6603  WorldGen.RandomizeCaveBackgrounds();    // <-- rolled ONCE at process boot
   ```
   **CORRECTION (W4-g72 phase 8, 2026-11-21):** the original census read this
   as "re-rolled on EVERY load, overwriting the persisted arrays". That is
   WRONG. `Initialize_AlmostEverything` fires ONCE per process — from the
   splash tail (`Main.cs:52572`) or dedServ `Initialize` (`:6499`) — BEFORE
   any world file is loaded. At that point `Main.maxTilesX` is still its
   static 0 (`Main.cs:1011`; the first writes are `autoCreate` `:4886-4896` /
   the dedServ size prompt `:5301` / `LoadWorldHeader` `WorldFile.cs:1976`,
   all later), so the boot roll ALWAYS takes the else arm with width 0
   (three `Next(0,0)` band draws — Knuth still samples — + four distinct
   styles + the ice/hell/jungle tail). The subsequent world load's flags
   read (`WorldFile.cs:2058-2067`) then OVERWRITES the rolled statics with
   the persisted values — the load wins, not the boot roll. The persisted
   arrays ARE what the renderer sees after load. The port's flags reader
   now owns them (`wld/sections/flags.rs`, W4-g72 phase 8); the boot roll
   executes in `App::new` on a dedicated `UnifiedRandom(world_seed)` as a
   dead store (the draw-order contract), leaving `rngs.global()` untouched.

---

## 3. The port's current tail — what is drawn vs missing

`step_world_tail` (`pipeline.rs:2685-2731`) is the **last rng-consuming step**
(`pipeline.rs:1380` `step!("world tail", …)`, the comment `:2693-2695` pins the
order). It draws from the shared `streams.rng` (the xorshift64 core,
`rng.rs:26-98`):

| port draw | line | C# analogue |
|-----------|------|-------------|
| `moon_type = next_u32() % 9` | 2726 | `RandomizeMoonState` `Next(9)` (`WorldGen.cs:7787`) — but the port cites `Main.cs:10873` (the BOOT moon roll), not the worldgen one |
| `hell_back_style = next_u32() % 3` | 2730 | `RandomizeCaveBackgrounds` `:8247` `Next(3)` |

**MISSING from the port tail** (everything else `RandomizeCaveBackgrounds`
draws): `caveBackX[0..3]`, `caveBackStyle[0..4]` (+ rejection loops),
`iceBackStyle` (`:8246`), `jungleBackStyle` (`:8248`). Also missing the
neighbouring `RandomizeTreeStyle` (`:11338`), `RandomizeBackgrounds` (`:11340`)
and the `WorldId` draw (`:11337`) — the port models none of them in the tail.

The port's `World` carries only `hell_back_style` (`world.rs:113`); there are
**no** `cave_back_x` / `cave_back_style` / `ice_back_style` / `jungle_back_style`
fields. `bg_styles.rs`'s `BgStylePickInput` therefore has no live source — the
style machine is unwired for exactly this reason (plus the atlas overflow).

### Draw semantics (port vs C#)
The port's `Xorshift64` (`rng.rs`) is **not** `UnifiedRandom`: `next_u32` is the
high half of a xorshift64 word (`:63-65`), `range(lo,hi) = lo + next_u32() %
(hi-lo)` (`:74-80`), `chance(n) = next_u32()%n==0` (`:83-85`). C#
`UnifiedRandom.Next` is a different generator. The port's streams are already
xorshift (the `world_hash` pins are xorshift-derived), so wiring the cave rolls
means drawing them **in C# order on the xorshift stream** and **re-pinning**
`SEED42_512` / `DEFAULT_WORLD` — the values change by design (the established
M5-BIOME / M5-ORES re-pin pattern, `world_hash.rs:76-128`).

---

## 4. The exact stream-delta for the worldgen lane

To wire `RandomizeCaveBackgrounds` faithfully the worldgen lane must, **in
`step_world_tail` (or a new step immediately before it), on `streams.rng`**:

1. Insert the width-arm `caveBackX` draws (Arm A/B/C per `world.width`), then
   the `caveBackStyle` base draws **in index order 0,1,2,3**, then the three
   rejection `while` loops **in C# order** (style[1] vs [0]; style[2] vs [0],[1];
   style[3] vs [0],[1],[2]) — each re-draws `Next(8)` until distinct.
2. Then `iceBackStyle = Next(4)`, `hellBackStyle = Next(3)`,
   `jungleBackStyle = Next(2)` (`:8246-8248`).
3. **Ordering constraint:** the C# puts the cave rolls BEFORE
   `RandomizeMoonState`'s `Next(9)`. The port currently draws `moon_type` FIRST
   (`:2726`) then `hell_back_style` (`:2730`). Faithful wiring must move the
   whole cave block **ahead of** the moon draw (or re-derive the moon roll's
   position) — otherwise the moon-type stream shifts relative to C#.
4. The `hell_back_style` draw already present (`:2730`) is the SAME `Next(3)`
   as `:8247` — it must not be drawn twice; fold it into the cave block's tail.
5. Add `World` fields `cave_back_x: [i32;4]`, `cave_back_style: [i32;4]`,
   `ice_back_style: u8`, `jungle_back_style: u8` (+ `.wld` read/write to replace
   the skip-reads at `flags.rs:382-383` / the zero-writes at `:48-51`), and feed
   `bg_styles.rs::BgStylePickInput`.
6. **Re-pin** `SEED42_512` and `DEFAULT_WORLD` (`world_hash.rs`) — every world
   shifts because the shared `rng` tail advances by the new draw count before
   the (already-last) step ends. The `worldgen_rolls_hell_back_style` witness
   (`bg.rs:1751`) only asserts determinism + `<3`, so it survives; the
   `direct.moon_type == via_pipeline.moon_type` / `hell_back_style` parity
   asserts (`pipeline.rs:3491-3493`) survive too (both paths draw identically).

**Boot re-roll caveat (§2.2, CORRECTED in phase 8):** C# rolls the cave
arrays ONCE at process boot (`Main.cs:6603`, before any load, width 0), and
the world load's flags read then overwrites them — the persisted values are
the final renderer input. The port's load path owns the arrays (phase 8);
no per-load re-roll exists in C# and none is ported.

---

## 5. Verdict

- **Original census verdict (phase 6):** do NOT wire in lane 3 (pin-moving).
- **SUPERSEDED (phases 7-8, 2026-11-21):** the Lead's port-gap assignment
  moved the roll into this lane. `worldgen/cave_bg.rs` ports the function
  over a `CaveRoll` trait (xorshift gen-side in `step_world_tail` ahead of
  the moon draw; Knuth boot-side in `App::new` as a dead store — see the §2
  correction), `World` carries the four fields, both save formats persist
  them (`.wld` flags + `.trwld` v18) and the `7 WorldInfo` header syncs them.
  The `world_hash` pins carry stale-by-design notes (the tail is the last
  `streams.rng` consumer and writes no tiles — the tile hash is expected
  unchanged; the gate re-pins if measured otherwise, `a == b` stays green).
- The exact draw list, order, rejection loops, width-arm bounds and the
  moon-before/after ordering constraint above were implemented verbatim.
- Witnesses: 5 in `cave_bg.rs` + 2 in `pipeline.rs` (+ the runner parity
  test extended) + 1 extended in `bg.rs` + 1 in `save.rs` + the `wld`
  owned-field sweep extended + 1 in `netglue.rs` — all WRITTEN NOT RUN
  (code-only lane).

---

## 6. The `:11340` ORDER-CONTRACT (W4-g72 final, input to the g71c bg-roll-wire)

The g71c lane wires `RandomizeBackgrounds` (`WorldGen.cs:7947-8012`, called
at `:11340`) on the gen side. **Answer to the boundary question: YES — g71's
forest/surface rolls insert BETWEEN our cave block and the moon draw.** The
exact C# tail order (read verbatim, `WorldGen.cs:11337-11344`):

```
:11337  WorldId = genRand.Next(int.MaxValue)     — UNPORTED (no World.world_id draw)
:11338  RandomizeTreeStyle()                     — UNPORTED (g70's roll, g71c candidate)
:11339  RandomizeCaveBackgrounds()               — PORTED (our cave block, phases 7-8)
:11340  RandomizeBackgrounds(genRand)            — UNPORTED (g71c's insert seat)
:11341  RandomizeMoonState(genRand)              — PORTED (moon_type Next(9))
:11344  dungeonSide = Next(2) …                  — later passes
```

The port's `step_world_tail` (`pipeline.rs:2685-2752`) draws, in order:

```
1. the cave block   — randomize_cave_backgrounds(&mut streams.rng, width, styles)
                      (:2741-2746; bands + styles + rejections + ice/hell/jungle)
2. moon_type        — streams.rng.next_u32() % 9 (:2750)
```

**The contract for g71c:**

1. **Insert seat:** g71c's `RandomizeBackgrounds` draws go AFTER
   `world.apply_cave_backgrounds(&roll)` (`pipeline.rs:2746`) and BEFORE the
   `moon_type` draw (`:2750`). Nothing else may draw between them — the C#
   `:11339→:11340→:11341` sequence is contiguous on `genRand`.
2. **Draw order inside the insert** (g71's `bg_rolls.rs::randomize_backgrounds`
   already ports it): four pairwise-distinct forest rolls
   (`RollRandomForestBGStyle` with its while re-rolls), corrupt
   (`Next(6)` + the 5→51/52 promotion), jungle `Next(7)`, snow `Next(9)` +
   the three 2/3/4 promotion ladders, hallow `Next(6)`, crimson `Next(7)`,
   desert (`Next(6)` + 5→51+`Next(5)/2`), ocean `Next(8)`, mushroom `Next(5)`,
   underworld `Next(3)` — all on `streams.rng`, in that order.
3. **The moon draw shifts again** — the same re-pin pattern our phases 7-8
   documented: `SEED42_512`/`DEFAULT_WORLD` carry stale-by-design notes; the
   tail writes no tiles so the tile hash is expected unchanged, the gate
   re-pins if measured otherwise, `a == b` stays green.
4. **`RandomizeTreeStyle` (`:11338`) and the `WorldId` draw (`:11337`)** are
   still unported. If g71c wires them too, they insert AHEAD of our cave
   block (the C# order) — our block's position relative to the moon is
   unchanged, but every cave value shifts. Our witnesses
   (`w4_g72_the_cave_block_draws_before_the_moon_roll`,
   `pipeline.rs:3529`) assert ranges/distinctness/determinism, NOT absolute
   values, so they survive any ahead-of-cave insert; the runner parity test
   survives because both paths draw identically.
5. **The boot side is NOT part of this contract** — `Main.cs:6603`'s boot
   roll is a once-per-process dead store (§2 correction); g71c's insert is
   gen-side only. The `.wld`/`.trwld`/net persistence of the surface styles
   is g70/g71's owned territory (the v18 union, research/64a); our cave
   fields are already persisted and the draw side reads them live
   (W4-g72 final item 3).
6. **The underworld `Next(3)`** (`:8011`, the last draw of
   `RandomizeBackgrounds`) feeds `underworldBG` (`WorldFile.cs:1376`,
   still skip-read at `flags.rs:573`). **SEAT FIRED (retire, §7):** g71
   owns that byte — rolled (`bg_rolls.rs:168`/`:311`) + `.wld`-persisted
   (`flags.rs:186`/`:577`) at tip `f2d8c2c67`; its v18 seat is [58] in the
   FINAL 107-byte union (research/64b §0/§2, 92 §1-FULL.2); the underworld
   DRAW stack stays ours (UNWIRED, the d35 §9.1
   screenTarget escape).

## 7. The keep-one RECEIPT (W4-g72 retire — g71 dropped ITS copy, kept OURS)

**Executed on g71's branch, NOT ours.** `origin/task/W4-g71-bg-layers-2`
commit **`fc76a48b3`** ("W4-g71 FINAL keep-one: adopt g72 cave_bg.rs
VERBATIM, drop the bg_rolls cave duplicate") + docs **`f2d8c2c67`**
(research/92 §2.1-FINAL) adjudicated the double-draw risk in OUR favour:

- **KEPT (canonical): OUR `worldgen/cave_bg.rs`.** g71's copy of the file
  differs from ours ONLY by the added keep-one header comment (verified:
  the diff carries no non-comment line). Deciding deltas (92 §2.1-FINAL's
  table): ours carries the PREVIOUS `caveBackStyle` values through the
  un-rolled arm slots (C# `:8195-8223` keeps them; the g71 copy zeroed
  them), returns the C# `new int[4]` shape (`[3]` never
  rolled/persisted/sent), emits u8 ice/hell/jungle directly, and our LIVE
  draw chain + `.wld`/`.trwld`/net persistence trio are typed against ITS
  outcome.
- **DROPPED on g71:** `bg_rolls.rs`'s `struct CaveBackgrounds` +
  `fn randomize_cave_backgrounds` (verified absent at the tip); its 3 cave
  witnesses re-point at the canonical driver; its
  `boot_reroll_backgrounds` now calls
  `cave_bg::randomize_cave_backgrounds` (`bg_rolls.rs:317`).
- **Composed tail order (g71's pipeline, matching our §6 contract):**
  `:11338` tree (pass_rng) → `:11339` OUR cave seat (`streams.rng`,
  `pipeline.rs:2760`) → `:11340` surface (pass_rng) → `:11341` moon
  (`streams.rng`). The tree/surface rolls ride isolated pass_rng slots, so
  the shared-stream sequence stays cave→moon exactly — ONE cave draw,
  never both.
- **NO DEPRECATED-BY-KEEP-ONE banners on OUR branch:** the drop happened
  on g71's side; our `cave_bg.rs` IS the canonical copy — nothing here is
  deprecated, nothing to banner. This §7 is the g71-drop-receipt only.
- **Merge-side residue (flagged, small):** our `step_world_tail` comment
  block updates to the composed order (92 §2.1-FINAL drop-hunk 6 — our
  code hunks land UNCHANGED); at merge keep ONE boot call site per process
  (our `App::new` dead-store vs g71's menu-seam boot fn — complementary
  today, must not both survive as live rolls on the same boot path).
