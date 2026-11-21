# 64 — `RandomizeCaveBackgrounds` worldgen RNG-shift census (W4-g72 lane 3, phase 6)

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

2. **Boot / world load** — `Main.cs:6603` (`Initialize`):
   ```
   6594  WorldGen.RandomizeBackgrounds(Main.rand);
   6595-6602  if (treeBGSet1[0]==173) RandomizeBackgrounds(Main.rand);  // ≤2 retries
   6603  WorldGen.RandomizeCaveBackgrounds();    // <-- re-rolled on EVERY load
   ```
   **Consequence:** the `.wld`-persisted cave arrays (`WorldFile.cs:1260-1269`
   write / `:2058-2066` read) are **overwritten at boot** by a fresh
   `Main.rand`-driven roll. The persisted values are not what the renderer sees
   after load — the boot roll is. (The port skip-reads them,
   `wld/sections/flags.rs:382-383`, and re-rolls only `hell_back_style` in the
   worldgen tail; see §3.)

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

**Boot re-roll caveat (§2.2):** C# re-rolls the cave arrays on every world load
(`Main.cs:6603`), so a faithful port must ALSO re-roll them at load from the
load-time `Main.rand` stand-in — otherwise loaded worlds keep the generation
roll while vanilla shows a fresh one. That is a separate app-lane seam (the
port's load path), flagged here so the worldgen lane does not assume the
generation roll is the final renderer input.

---

## 5. Verdict

- **Do NOT wire in lane 3** (pin-moving; the task forbids it). The style machine
  (`bg_styles.rs`) and the capture seams (`bg_target.rs`) consume the rolls but
  the rolls themselves are unowned.
- The exact draw list, order, rejection loops, width-arm bounds and the
  moon-before/after ordering constraint are documented above for the worldgen
  lane to implement + re-pin in one pass.
- No RNG was drawn, no `world_hash` constant touched, no `World` field added by
  this census. Witnesses: none (documentation only).
