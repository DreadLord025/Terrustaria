# 92 — DOC-SWEEP: background variants (lane 16, W4-g71)

**Branch:** `task/W4-g71-bg-layers-2` (continues the DONE `2b57b362` surface-variant
work). **Mode:** code-only — no cargo/tests/graphify invoked; witnesses WRITTEN NOT
RUN. **Scope:** the four bg-variant follow-ups the Lead flagged after the g70/g71/g72
surface lanes landed. Companion lanes: `task/W4-g70-bg-layers-1` (forest sets, tip
`b86c8d69`), `task/W4-g72-bg-layers-3` (cavern/hell style machine, tip `f956d3ba`).

Four items, four verdicts:

| # | Item | Verdict | Evidence |
| --- | --- | --- | --- |
| 1 | g70/g71/g69/d125/d109 v18 tails | **CONFLICT — five-branch UNION contract below** | all five append a *different* v18 block after the v17 fishron byte |
| 2 | `RandomizeBackgrounds`/Tree/Cave rolls | **WIRED** (`worldgen/bg_rolls.rs` → `step_world_tail`) + boot seam + atlas budget guard | g72c "ACTIVE" claim STALE (census only); tile pins unmoved; hell value stale-by-design |
| 3 | ocean wave ANIMATION | **NO-OP — the C# ocean bg is STATIC** | ocean ids absent from every animation site |
| 4 | desert oasis row-250 + underground-desert bg | **row-250 VERIFIED live; underground-desert = g72's, no separate bg** | `sky.rs:1735` witness; `Main.cs:54033-54095` has no desert arm |

---

## 1. The TRWLD v18 UNION layout contract (five-branch extension)

> **Extended by the W4-g71 wiring phase** (this branch, `task/W4-g71-bg-layers-2`).
> The original §1 reconciled only g70×g71. Three more branches have since bumped
> `save.rs` `VERSION` to **18** with their own tail block — **g69** (crimson byte),
> **d125** (downedEmpress byte) and **d109** (the dense framing-variants block).
> The union below now carries **all five**. Every tail was read off the branch's own
> `save.rs` (writer + reader + back-compat fixtures); the exact per-branch truncation
> constants are tabulated in §1.5.

### The conflict (five incompatible v18 tails)

Each lane independently bumped `VERSION` 17→**18** and appended a *different* block
immediately after the v17 `downedFishron` byte. The version byte alone cannot
disambiguate them — a v18 file from one branch mis-parses on every other:

| Branch | Tip | Block | Size | C# citation | Pre-v18 default |
| --- | --- | --- | --- | --- | --- |
| **g70** | `bbe47378` | forest-set: `tree_x[0..3]` 3×i32 LE + `forest_bg_styles[0..4]` 4×u8 | **16 B** | `WorldFile.cs:1253-1259` (treeX/treeStyle), `:1318` (setBG0), `:1376-1379` (treeBG2-4) | `tree_x=[width;3]`, `forest_bg_styles=[0;4]` |
| **g71** | `edc40654` | surface: `snow/desert/ocean_bg_style` 3×u8 | **3 B** | `WorldFile.cs:1321`/`:1324-1325` (setBG 3/6/7) | all three 0 |
| **g69** | `9717c0bb` | `WorldGen.crimson` 1×u8 (bool) | **1 B** | `WorldFile.cs:1281` (flags byte); `:3507` corruption fallback | `false` (corruption) |
| **d125** | `d8047a10` | `NPC.downedEmpressOfLight` 1×u8 (bool) | **1 B** | `NPC.cs:6393`, case-636 death flag `:79351-79353` | `false` |
| **d109** | `81894095` | packed framing variants `World::variants`, dense `width*height`×u8, row-major | **W×H B** (dense) | port-only (`Tile.bTileHeader2` bits 4-7, `Tile.cs:464-482`; the C# `.wld` never writes them, `SaveWorldTiles` `WorldFile.cs:1432-1604`) | `World::new` zeros, `variants_from_save=false` (re-roll) |

All five writers end their block at `Ok(())`; all five readers gate on
`if version >= 18`. g69/d125 append their byte to the **flags tail** (the same
`write_flags` stream g70/g71 extend); d109 appends its dense block at the very end of
`write_world` (after the flags tail) and its reader sizes it from `world.tiles.len()`.

### The UNION contract (one tail, all five blocks, fixed order)

The merge keeps **one** v18 tail carrying **all five** blocks in a fixed order:
**forest-first** (the C# `.wld` field order — treeX/treeStyle precede the setBG bytes,
which precede the crimson flag), then the g71 surface bytes, then the two new flag
bytes (g69 crimson, d125 empress), then **d109's dense variants block LAST** (it is
variable-length and must sit at the end so the fixed head stays at a known offset):

```
… v17 downedFishron byte (1)
v18 UNION tail — fixed head (21 bytes) + dense variants (W×H bytes):
  offsets relative to the start of the v18 block:
  [0..4]   tree_x[0]            i32 LE   ┐
  [4..8]   tree_x[1]            i32 LE   │ g70 forest-set block (16 bytes)
  [8..12]  tree_x[2]            i32 LE   │
  [12]     forest_bg_styles[0]  u8       │  (setBG0  / treeBG1)
  [13]     forest_bg_styles[1]  u8       │  (setBG10 / treeBG2)
  [14]     forest_bg_styles[2]  u8       │  (setBG11 / treeBG3)
  [15]     forest_bg_styles[3]  u8       ┘  (setBG12 / treeBG4)
  [16]     snow_bg_style        u8       ┐ g71 surface bytes (3 bytes)
  [17]     desert_bg_style      u8       │  (setBG3 / setBG6 / setBG7)
  [18]     ocean_bg_style       u8       ┘
  [19]     crimson              u8 bool    g69 world evil type (WorldFile.cs:1281)
  [20]     downed_empress_of_light u8 bool d125 Empress flag (NPC.cs:6393)
  [21..21+W*H]  variants        W*H × u8   d109 dense framing variants (LAST)
```

**Fixed head = 21 bytes: 16 (forest) + 3 (surface) + 1 (crimson) + 1 (empress).**
**Total tail = 21 + width*height bytes.**

**Writer** (`save.rs`, after the v17 fishron write; the variants block is the last
thing `write_world` emits):
```rust
// v18 UNION fixed head: g70 forest, g71 surface, g69 crimson, d125 empress.
for x in world.tree_x { w.write_all(&x.to_le_bytes())?; }   // 3×i32  [0..12]
w.write_all(&world.forest_bg_styles)?;                       // 4×u8   [12..16]
w.write_all(&[world.snow_bg_style, world.desert_bg_style, world.ocean_bg_style])?; // [16..19]
w.write_all(&[u8::from(world.crimson)])?;                    //        [19]
w.write_all(&[u8::from(world.downed_empress_of_light)])?;    //        [20]
// … then, at the very end of write_world (d109):
w.write_all(&world.variants)?;                               // dense W*H  [21..]
```

**Reader** (`save.rs`, the `if version >= 18` arm):
```rust
if version >= 18 {
    for x in &mut world.tree_x { *x = r.i32()?; }            // [0..12]
    for s in &mut world.forest_bg_styles { *s = r.u8()?; }   // [12..16]
    world.snow_bg_style = r.u8()?;                           // [16]
    world.desert_bg_style = r.u8()?;                         // [17]
    world.ocean_bg_style = r.u8()?;                          // [18]
    world.crimson = r.u8()? != 0;                            // [19]
    world.downed_empress_of_light = r.u8()? != 0;            // [20]
    let n = world.tiles.len();                               // d109 dense, LAST
    world.variants = r.take(n)?.to_vec();
    world.variants_from_save = true;
}
```

### Back-compat

- **Pre-v18 files** (version ≤ 17): the whole tail sits unread. The World keeps its
  `new`/Reset defaults — `tree_x=[width;3]`, `forest_bg_styles=[0;4]`,
  `snow/desert/ocean_bg_style=0`, `crimson=false`, `downed_empress_of_light=false`,
  `variants=zeros`, `variants_from_save=false`. Each branch's own pre-v18 arm already
  does exactly this; the union preserves all five.
- **Any single-branch v18 file** read by the union loader mis-parses (the fixed head
  expects 21 bytes + the dense block; a 16/3/1/1-byte or variants-only tail runs off
  the end or reads garbage). **Not back-compat** — but no such file ships: all five
  branches are unmerged task work, and the union is the *first* v18 to land on main.
  The contract is forward-only from the merge; there is no released v18 to grandfather.
- **d109's dense block is self-delimiting by `tiles.len()`**, so a v18 payload cut
  mid-variants-block is a structural EOF error (d109's
  `w4_d109_a_truncated_variants_block_bails` witness) — the union keeps that property.

### 1.5 Per-branch truncation-constant moves (the merge arithmetic)

Every back-compat fixture that counts the v18 tail from the end moves to the union
length. **Fixed-head fixtures → 21; end-relative fixtures that also peel d109's dense
block → `21 + v18_tail_len(buf)`** (d109's `v18_tail_len` helper reads width×height
from the header, `save.rs:1351` on its branch). Per branch (line numbers are that
branch's own `save.rs`):

| Site | g70 | g71 | g69 | d125 | d109 | **UNION** |
| --- | --- | --- | --- | --- | --- | --- |
| `EMPTY_V16_TAIL` | `…+1+16` (`:1331`) | `…+1+3` (`:1329`) | `…+1+1` (`:1325`) | `…+1+1` (`:1319`) | `…+1` (no v18 in const; dense peeled separately) | `EMPTY_V13_TAIL + 9 + 1 + 5 + 1 + 21` |
| v12 `let tail = …` | `…+1+16` (`:1302`) | `…+1+3` (`:1300`) | `…+1+1` (`:1296`) | `…+1+1` (`:1290`) | `v18_tail_len(&buf)+…` (`:1201`) | `…+1+21` (+ dense if end-relative) |
| v14 `truncate(len-1-5-1-…)` | `-16` (`:1520`) | `-3` (`:1518`) | `-1` (`:1514`) | `-1` (`:1508`) | `v18_tail_len+1+5+1` (`:1552`) | `-21` |
| v18 round-trip truncation | `-16` (`:2212`) | `-3` (`:2210`) | `-1` (`:1541`) | `-1` (`:1526`) | `-v18_tail_len` (`:2280`) | `-21` (fixed head) / `-v18_tail_len` (dense) |
| v5/v6 `+1` fishron comment | `+16` (`:1189`) | `+1` (`:1187`) | `+1` (`:1183`) | `+2` (`:1176`) | `v18_tail_len+…` (`:1104`) | `+21` (+ dense) |

The `expected_flags_block` byte-table test (g71 phase 4b, `2b57b362`) and g70's
forest round-trip keep their assertions; only the tail length moves. **d109's
end-relative fixtures** (`:1104`, `:1201`, `:1253`, `:1319`, `:1366`, `:1373`,
`:1552`, `:1574`, `:1596`, `:1713`, `:1834`, `:1950`, `:2078`, `:2180`) already peel
`v18_tail_len(buf)` first — in the union that helper returns **`21 + width*height`**
(the fixed head rides ahead of the dense block), so those sites need no structural
change, only the helper's new return value.

**Merge instruction:** when the five branches reconcile on main, take the union
writer + reader above (NOT `--ours`/`--theirs`), keep `VERSION = 18`, move every
fixed-head truncation constant to **21**, and make `v18_tail_len` return
`21 + width*height`. The five round-trip witnesses (g70 forest, g71 surface, g69
crimson, d125 empress, d109 variants) all keep their assertions; only the tail
lengths move.

**Why forest-first + variants-last:** the order is arbitrary but must be *fixed and
shared*. Forest first matches the C# `.wld` field order for the two style groups
(treeX/treeStyle `WorldFile.cs:1253-1259` precede the setBG bytes `:1318-1325`) and
keeps g70's 16-byte block readable as a unit at offset 0 — the original §1 contract,
unchanged. The two new flag bytes (crimson, empress) follow the style bytes as a
group; they do NOT sit at their C# relative positions (crimson `:1281` precedes setBG
in the vanilla block) because the union tail is the port's own appended block, not a
replay of the vanilla flags layout. The dense variants block is variable-length and
port-only, so it sits LAST — the fixed head stays at a known offset from the v17
byte, and d109's end-relative peel arithmetic keeps working. This is the
least-surprising choice for the next lane that extends the tail.

---

## 2. `RandomizeBackgrounds` worldgen rolls — PORTED UNWIRED

**Census.** The vanilla rolls the surface/cave background styles mid-`WorldGen.Reset`
on the shared `genRand`, in this order (`WorldGen.cs:11337-11341`):

```
:11337  Main.ActiveWorldFileData.WorldId = genRand.Next(int.MaxValue)
:11338  RandomizeTreeStyle()          // treeX dividers + treeStyle (8117-8190)
:11339  RandomizeCaveBackgrounds()    // caveBackX/Style + ice/hell/jungle (8192-8249)
:11340  RandomizeBackgrounds(genRand) // the 11 setBG styles (7947-8012)
:11341  RandomizeMoonState(genRand)   // moonType Next(9) (7782-7790)
:11344+ dungeon-side Next(2), jungle-origin Next(15,30)/Next(25,35) …
```

`RandomizeBackgrounds` (`:7947-8012`) draws, in order: four **pairwise-distinct**
forest styles (`RollRandomForestBGStyle` `:8034-8059` with `while` re-rolls), then
corrupt (`RandomizeBackgrounds_Corruption` `:8014-8022`, 5→51/52), jungle `Next(7)`,
snow (`:7971-8004`, 2→21/22, 3→31/32, 4→41/42 promotions), hallow `Next(6)`, crimson
`Next(7)`, desert (`RandomizeBackgrounds_Desert` `:8024-8032`, 5→51+Next(5)/2), ocean
`Next(8)`, mushroom `Next(5)`, underworld `Next(3)`. `SetForestBGSet` (`:7639-7768`,
the 19-arm style→texture table) is **already ported** by g70 as
`World::forest_bg_set` (`world.rs`) — this lane ports the *rolls*, not the table.

**Port.** `crates/tr-core/src/worldgen/bg_rolls.rs` (`pub mod`): `roll_random_forest_bg_style`,
`randomize_backgrounds_corruption`, `randomize_backgrounds_desert`, `roll_snow_bg_style`,
`randomize_backgrounds` (returns a `BackgroundStyles` struct), `randomize_tree_style`,
`randomize_cave_backgrounds` (returns `CaveBackgrounds`), `is_background_considered_the_same`
(`:8061-8095`, the 3≡31/5≡51/7≡71-73 equivalence the player-driven re-roll uses), and
`boot_reroll_backgrounds` (the `Main.cs:6594-6603` boot seam, §2.3). All are **pure
functions over a caller-supplied stream** through the [`RollRng`] trait — no `World`
writes inside the rolls, no global state. The trait is implemented for BOTH the
worldgen xorshift64 `Rng` (the tail's `pass_rng` slots) and `UnifiedRandom` (the boot
re-roll's bare-`Main.rand` stand-in), so the SAME canonical code serves both callers.
10 roll witnesses + 4 boot witnesses WRITTEN NOT RUN, expectations derived from
independent Python references (xorshift64 for the tail; a Knuth-subtractive
`UnifiedRandom.cs` port — golden `Next()` vector verified — for the boot).

### 2.1 WIRING (this phase) — the keep-one adjudication + the seam contract

**Adjudication (g72c "ACTIVE cave wiring" is STALE).** The brief claimed g72c ports
`RandomizeCaveBackgrounds` into `step_world_tail` and that `bg_rolls.rs` has the SAME
fn (keep-one). **Read off g72's tip `1f5c2b1e`:** its `step_world_tail`
(`pipeline.rs:2685`) still draws ONLY moon type + hell back style; commit `345c334a`
is explicitly a *census* ("NO wiring, no RNG drawn, no pin touched", research/64). g72
has **zero roll code** — only the `render/bg_styles.rs` *consumer*
(`BgStylePickInput`/`background_style_sheets`). So there is **no duplicate to
keep-one**: this lane's `bg_rolls.rs` is the sole canonical copy.

**Seam contract (published):** g72's cave-style machine consumes the rolls; it does
NOT re-port them. The wired tail (`step_world_tail`) calls
`bg_rolls::randomize_cave_backgrounds` and writes `World.cave_back_x` /
`cave_back_style` / `ice_back_style` / `hell_back_style` / `jungle_back_style`; g72's
`BgStylePickInput` reads those `World` fields when its draw side wires. If a future
g72 commit lands its own roll copy, THIS fn is canonical — delete the copy and call
`bg_rolls::randomize_cave_backgrounds` (the `RollRng` seam accepts either stream).

**The wiring.** `step_world_tail` now runs the three rolls in C# order
(`WorldGen.cs:11338-11340`), each on its OWN `pass_rng` slot
(`"RandomizeTreeStyle"` / `"RandomizeCaveBackgrounds"` / `"RandomizeBackgrounds"`),
writing the `World` fields through `set_surface_bg_style` (the `setBG` calls). The
`hell_back_style` `Next(3)` (`:8247`) folds into the cave block — the former
standalone tail draw is gone. The moon draw keeps its historical `streams.rng` slot.

**Why the tile-hash pins do NOT move (correcting the earlier UNWIRED rationale).**
The earlier note feared a mid-`Reset` shared-stream insertion would shift the
dungeon-side/jungle-origin/seed-finish draws. But the port's tail is the **last**
`streams.rng` consumer (the seed-finish/micro passes all ride isolated `pass_rng`
slots), and the rolls write **only style/divider fields, never tiles**. Wiring them
on isolated `pass_rng` slots therefore shifts no other pass's draws and moves no
`world_hash` TILE pin — `SEED42_512`/`DEFAULT_WORLD` stay byte-identical (the
`w4_g71_the_wired_tail_rolls_styles_without_moving_tiles` witness asserts
`a.tiles == b.tiles`). Only the folded `hell_back_style` VALUE moves
(stale-by-design; the `worldgen_rolls_hell_back_style` determinism + `<3` witness
stays green, as does the direct-vs-pipeline parity).

### 2.2 The atlas overflow — the pack-time budget guard

The one real blocker remains: a rolled non-default surface style needs its
`Background_<id>` sheets packed, and the single 8192 px atlas packs the style-0 set
to **7847 px** (measured, `tools/atlas_sim.js` skyline twin). Non-default styles add
336–1503 px:

| style | extra sheets | atlas height | delta |
| --- | --- | --- | --- |
| baseline (style 0) | — | 7847 | — |
| desert 4 (oasis 248/249/250) | +3 | 8186 | +339 (fits) |
| mushroom 4 (321/320/319) | +3 | 8183 | +336 (fits) |
| ocean 5 (283) | +1 | 8008 | +161 (fits) |
| corrupt 4 (240/241/242) | +3 | **8193** | +346 **OVERFLOW** |
| snow 7 (267/266/268+269/270) | +5 | **8323** | +476 **OVERFLOW** |
| hallow 5 (327/326/325) | +3 | **8370** | +523 **OVERFLOW** |
| snow 5 (258/259/260) | +3 | **8410** | +563 **OVERFLOW** |
| jungle 6 (302/301/300) | +3 | **8478** | +631 **OVERFLOW** |
| crimson 6 (339/338/337) | +3 | **8729** | +882 **OVERFLOW** |
| desert 51 (8 variant sheets) | +8 | **9350** | +1503 **OVERFLOW** |
| one-each rolled (8 biomes) | +29 | **12345** | +4498 **OVERFLOW** |

**The guard (`assets::fit_bg_styles_to_atlas` + `build_atlas_inner`).** Rather than
overflow, the atlas build reverts the offending biome styles to slot-0 (the classic
look) until the set fits: after the first pack, if `atlas.height > 8192`, the guard
trials candidate bg-id sets through the cheap `Atlas::packed_height` (the same
dedup/sort/skyline as `build_ref`, no pixel copies) over `bg trial ++ static ++ font`,
reverting slots in the fixed `setBG` order 1..8 until one fits, then applies the
reverts to the `World` and rebuilds. Deterministic, greedy, no RNG. The forest tree
sets (0/10/11/12) and underworld (9) are NOT reverted — g70/g72 own their draw sides
and their sheets are not in the surface-bg world half. **This is a deliberate
deviation** (the C# draws surface bgs outside the tile atlas via `Main.spriteBatch`
under the sky matrix — the S1-G-13 background-target family is the long-term fix);
until that lands, the revert keeps the atlas in bounds at the cost of the classic
look for overflowing biomes. Recorded in ANALYSIS.

### 2.3 The boot re-roll (Main.cs:6594-6603) — the menu seam

`Main.Initialize` (the XNA `Game.Initialize` override, `Main.cs:6447`) runs
`RandomizeBackgrounds(Main.rand)` + the two `treeBGSet1[0]==173` retries +
`RandomizeCaveBackgrounds()` ONCE per process at boot, **before any world loads**.
`LoadWorldFlags` then overwrites every field it writes with the file's persisted
values (`WorldFile.cs:2050-2067`, `:2211-2233`). So the boot roll drives the **menu**
background only — a loaded world keeps its saved styles. (This corrects g72's
research/64 §2.2 claim that the boot roll "overwrites the .wld-persisted arrays" at
load: the load read comes AFTER the boot roll and wins.)

**The seam (`bg_rolls::boot_reroll_backgrounds`).** The canonical roll-and-apply the
menu caller invokes on the bare-`Main.rand` stand-in (`UnifiedRandom`, the same
`RollRng` the tail uses). The 173 retry: `SetForestBGSet(6)` sets `treeSet[0]=173`
(`WorldGen.cs:7691`), so a forest style 6 re-rolls the whole set, up to two extra
times (the C#'s two `if` guards = at most three rolls). The port reads the rolled
`forest[0]` and re-rolls while it is 6. Witnesses pin seed 42 (forest `[9,71,10,73]`,
cave `[6,4,0,5]`/hell 2 at width 512) and the seed-11 retry (first 6 → final 3).
**Placement:** the port's menu world is the `App`'s boot world; the faithful caller
is the menu-background seam, NOT the world-load path. This phase ports + witnesses
the fn; wiring it into the live menu draw is the menu lane's call (the menu currently
renders a static sky — the roll would feed a future menu-bg consumer).

**Ownership note (updated).** The cave/ice/jungle back-style fields + the tree
arrays + the forest/underworld setBG bytes are **owned** now (the wired tail rolls
them; `wld/sections/flags.rs` writes + reads them, the `.trwld` v18 union carries the
forest block). g72's `render/bg_styles.rs` consumes them as `World` fields when its
draw side wires. `hell_back_style` was already owned; its draw folds into the cave
block.

---

## 3. Ocean wave ANIMATION — NO-OP (the C# ocean bg is STATIC)

**Census.** The Lead flagged "your ocean static — census Main ocean bg wave frame
tick". The census finds **no such tick exists**. The ocean background is a single
far-layer sheet (`Main.oceanBG`, one of 28/110/111/209/210/283/332/340 per
`WorldGen.setBG` case 7, `WorldGen.cs:7548-7576`) drawn by
`DrawSurfaceBG_BackMountainsStep2` → `DrawSurfaceBG_DrawBackMountainsLayer`
(`Main.cs:59916-59920`, `:59725-59738`). That draw calls `GetBackgroundRect`
(`:61220-61262`), which returns `null` (whole sheet, no animation) for **every**
ocean id — none appears in its `case` list (the animated ids are 235, 219-221,
271-273, 302/303/305/307/309/311/313/315/317, 326, 337/338, 341, 281). None appears
in `LoadBackground`'s width/height-halving switch (`:59046-59072`) either, and
`DoUpdate_AnimateBackgrounds` (`:19650-19735`) animates **only** the four forest
hill families (`UpdateForest`), never the ocean.

The only ocean-specific per-frame work is `UpdateOceanWaterLineForAmbience`
(`:59965-59984`): it computes the water-line Y (`bgTopY + GetLayerYOffset + num2 *
bgScale`, num2 = 102 for 28/110/111/209/210, 124 for 283, 0 for 332/340) and writes
it + the far-layer-3 alpha into `AmbientSkyDrawCache.OceanLineInfo`. That is a
**geometry/opacity cache for the ambience entities**, not a sheet animation — the
ocean sheet itself never changes frame. The water *surface* waves (the liquid
shader's `WaterDistortion`/`WaveQuality`, `LiquidRenderer._waveMask`) are a separate
liquid-rendering system, not the background.

**Verdict.** The port's static ocean (`sky.rs:1183-1204`, `draw_bg_layer` with no
`frame_step`) is **faithful** — there is nothing to port. The ocean-line *geometry*
is already ported as `ocean_water_line_offset` (`render/bg.rs:1327-1334`, g71 phase
2); the ocean-line *opacity* term (`AmbientSky.cs:72-79`
`Helper_GetOpacityWithAccountingForOceanWaterLine`) remains the documented **Q4
constant-1 deviation** (`ANALYSIS.md:1616-1620`, `ambience.rs:2346-2347`). That
deviation is about entity fade at the water line, not bg animation, and is unchanged
by this sweep. **No code change.**

---

## 4. Desert oasis row-250 + underground-desert bg

### Oasis row-250 — VERIFIED live

`DrawSurfaceBG_Desert` (`Main.cs:61019-61096`) draws `Pure.Backgrounds`' three rows;
the oasis style 4 adds the third row `Background_250`. Verified end-to-end on this
branch:

- **Style table:** `desert_bg_set(4) = [[248,249,250], …]` (`world.rs:1475`,
  `WorldGen.cs:7501-7546`).
- **Offsets:** `bg_offsets(250) = (Some(1.70), Some(0.49), 2000, 1000)`
  (`render/bg.rs:1234`, `Main.cs:60456-60461`) — the third row's scale 1.7, parallax
  0.49, top `magic*2000 + 1000`.
- **Draw:** `sky.rs:1343-1367` builds the three Pure rows incl `sky.desert_third`
  (`one(desert[0][2])`, `sky.rs:498`); the witness
  `desert_pure_rows_follow_the_style_including_the_oasis_third_row`
  (`sky.rs:1735-1772`) pins style 4's third row at scale `1.70*2`, parallax 0.49,
  top `2000/1000-150`.
- **Sheet:** `Background_250.xnb` is **1024×1000** (measured via the `tr-content`
  `dims` probe) — a static sheet (250 is not in `GetBackgroundRect`'s animated list),
  packed by `load_world_sheets` when the world rolls desert style 4.

The Step1/2 desert mountains 23/22 are correctly **skipped** for `desertBG == 4`
(`sky.rs:1166`, `Main.cs:59850-59853` Step1 / `:59906-59909` Step2). **No code change** — the
oasis row is already live and witnessed (g71 phase 2/3).

### Underground-desert bg — g72 owns it; there is NO separate desert cavern style

**Census.** `DrawBackground_PickUndergroundBackgroundStyle` (`Main.cs:54033-54095`)
has **no underground-desert arm**. Its style picks are: the cave-band ternary
(`caveBackX`/`caveBackStyle` + 3), snow (1) / jungle (11) count overrides, the beach
ocean caverns (18/19/20/21 by evil), the deep-cavern evil/snow-evil rows
(12/13/14/15/16/17), and the glowshroom trump (2). `ZoneUndergroundDesert`
(`SceneMetrics.cs:692` — desert + below-surface + sandstone/hardened-sand wall)
drives **music** (`Main.cs:12064/12089`, track 66) and spawning, **not** a background
style. An underground desert therefore shows whatever the cave-band/evil pick
resolves to — there is no dedicated desert cavern sheet set to port.

**g72's claim (read, not touched).** `task/W4-g72-bg-layers-3` ports the whole cavern
style machine UNWIRED in `render/bg_styles.rs`:
`pick_underground_background_style` (`:54033-54095`), the 22-case
`background_style_sheets` table (`:53812-54008`), and `ug_back_step`
(`:52823-52841`). Its module docs cite the same two blockers this lane hit (atlas
overflow + the `RandomizeCaveBackgrounds` tail RNG-shift) and explicitly name
`caveBackX`/`caveBackStyle`/`iceBackStyle`/`jungleBackStyle` as **unowned, a worldgen
lane's call**. This lane's `bg_rolls.rs::randomize_cave_backgrounds` is exactly that
worldgen-side roll, ported to feed g72's machine when a future lane wires both. The
underworld stack (`render/underworld_bg.rs`, `setBG(9)` `WorldGen.cs:7609-7635`) is
likewise g72's; this lane's `randomize_backgrounds` returns the `underworld` style
(`Next(3)`) for completeness but does not own its draw side. **No overlap, no code
change to g72's files.**

---

## Files touched this sweep

DOC-SWEEP phase (commit `6b4f9d27` + `e2d085ae`):
- `crates/tr-core/src/worldgen/bg_rolls.rs` — NEW, the style rolls + 10 witnesses.
- `crates/tr-core/src/worldgen/mod.rs` — `mod bg_rolls;` registration.
- `crates/tr-core/src/worldgen/pipeline.rs` — the pin-move note at `step_world_tail`.
- `docs/research/92-doc-sweep-bg.md` — this report.

WIRING phase (this branch, continuing `e2d085ae`):
- `crates/tr-core/src/worldgen/bg_rolls.rs` — the `RollRng` seam (xorshift `Rng` +
  `UnifiedRandom`), `pub` fns, `boot_reroll_backgrounds` + 4 boot witnesses.
- `crates/tr-core/src/worldgen/mod.rs` — `pub mod bg_rolls;`.
- `crates/tr-core/src/worldgen/pipeline.rs` — the wired `step_world_tail` (three
  rolls in C# order on `pass_rng` slots, hell folded into the cave block) + the
  `w4_g71_the_wired_tail_rolls_styles_without_moving_tiles` witness + parity extends.
- `crates/tr-core/src/world.rs` — the new fields (`cave_back_x/style`,
  `ice/jungle_back_style`, `tree_x`, `tree_style`, `forest_bg_styles`,
  `underworld_bg_style`), `World::new` defaults, `set_surface_bg_style` slots
  0/9/10/11/12 + witness.
- `crates/tr-core/src/wld/sections/flags.rs` — `.wld` write + read ownership of the
  tree/cave arrays, ice/jungle/hell back styles, setBG 0/9/10-12.
- `crates/tr-core/src/wld/mod.rs` — the byte-table fixtures (`flags_tail_prefix`,
  `expected_flags_block`), `owned_world` + `assert_owned_fields` extends.
- `crates/tr-app/src/assets.rs` — `world_bg_sheet_ids`, `BgStyleBytes`,
  `fit_bg_styles_to_atlas` (the budget guard) + 4 guard witnesses.
- `crates/tr-app/src/app/mod.rs` — `build_atlas_inner` revert-and-rebuild guard;
  `build_atlas_reusing_static`/`swap_world` take `&mut World`.
- `crates/tr-app/src/main.rs` — boot caller passes `&mut world`.
- `crates/tr-render/src/atlas.rs` — `Atlas::packed_height` (the cheap probe).
- `crates/tr-app/src/render/bg.rs` — `worldgen_rolls_hell_back_style` witness extends.
- `docs/research/92-doc-sweep-bg.md` — §1 five-branch union, §2 wiring/seam/guard/boot.

**Verification:** code-only mode — no cargo/tests/rustfmt/graphify run. The roll +
boot + guard witnesses are WRITTEN NOT RUN; their expectations come from independent
Python references (xorshift64 for the tail; a Knuth-subtractive `UnifiedRandom.cs`
port — golden `Next()` vector verified — for the boot) fed through the verbatim C#
roll logic, not from executing the Rust. The atlas heights come from the
`tools/atlas_sim.js` skyline twin (node, not cargo). The UNION v18 contract is a
*merge instruction* — the five branches' `save.rs` tails are not reconciled here
(each stays on its own branch); the reconciliation happens when they land on main
per §1. The `.wld` ownership changes ARE landed on this branch (the wired tail rolls
the fields, so the save layer must carry them).
