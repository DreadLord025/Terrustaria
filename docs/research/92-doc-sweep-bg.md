# 92 — DOC-SWEEP: background variants (lane 16, W4-g71)

**Branch:** `task/W4-g71-bg-layers-2` (continues the DONE `2b57b362` surface-variant
work). **Mode:** code-only — no cargo/tests/graphify invoked; witnesses WRITTEN NOT
RUN. **Scope:** the four bg-variant follow-ups the Lead flagged after the g70/g71/g72
surface lanes landed. Companion lanes: `task/W4-g70-bg-layers-1` (forest sets, tip
`b86c8d69`), `task/W4-g72-bg-layers-3` (cavern/hell style machine, tip `f956d3ba`).

Four items, four verdicts:

| # | Item | Verdict | Evidence |
| --- | --- | --- | --- |
| 1 | g70 v18 tail vs g71 v18 tail | **CONFLICT — UNION contract below** | both branches append a *different* v18 block after the v17 fishron byte |
| 2 | `RandomizeBackgrounds` worldgen rolls | **PORTED UNWIRED** (`worldgen/bg_rolls.rs`) | RNG-shift + atlas overflow; pin-move documented |
| 3 | ocean wave ANIMATION | **NO-OP — the C# ocean bg is STATIC** | ocean ids absent from every animation site |
| 4 | desert oasis row-250 + underground-desert bg | **row-250 VERIFIED live; underground-desert = g72's, no separate bg** | `sky.rs:1735` witness; `Main.cs:54033-54095` has no desert arm |

---

## 1. The TRWLD v18 UNION layout contract

### The conflict

Both lanes independently bumped `save.rs` `VERSION` from 17 to **18** and appended a
*different* tail block immediately after the v17 `downedFishron` byte. They are
mutually incompatible as written — a v18 file from one branch mis-parses on the other:

- **g70** (`bbe47378`, `save.rs` writer ~`:431-438`): the **forest-set block** —
  `tree_x[0..3]` as 3×i32 LE, then `forest_bg_styles[0..4]` as 4×u8. **16 bytes.**
  Cites `WorldFile.cs:1255-1258` (treeX/treeStyle), `:1319` (setBG0), `:1376-1379`
  (treeBG2-4). Pre-v18 loads the `WorldGen.Reset` defaults (`tree_x = [maxTilesX; 3]`,
  `forest_bg_styles = [0; 4]`).
- **g71** (`edc40654`, `save.rs:434-441`): the **three surface style bytes** —
  `snow_bg_style`, `desert_bg_style`, `ocean_bg_style` as 3×u8. **3 bytes.**
  Cites `WorldFile.cs:1321`/`:1324-1325` (setBG 3/6/7). Pre-v18 loads the slot-0
  defaults (all three 0).

Both writers end their block at `Ok(())`; both readers gate on `if version >= 18`.
The version byte alone cannot disambiguate them.

### The UNION contract (one tail, both blocks)

The merge keeps **one** v18 tail carrying **both** blocks, in a fixed order. The
forest block goes first (it is the larger, structurally-rigid one and g70's loader
already reads it as a unit); the three surface bytes follow:

```
… v17 downedFishron byte (1)
v18 UNION tail (19 bytes), offsets relative to the start of the v18 block:
  [0..4]   tree_x[0]            i32 LE   ┐
  [4..8]   tree_x[1]            i32 LE   │ g70 forest-set block (16 bytes)
  [8..12]  tree_x[2]            i32 LE   │
  [12]     forest_bg_styles[0]  u8       │  (setBG0  / treeBG1)
  [13]     forest_bg_styles[1]  u8       │  (setBG10 / treeBG2)
  [14]     forest_bg_styles[2]  u8       │  (setBG11 / treeBG3)
  [15]     forest_bg_styles[3]  u8       ┘  (setBG12 / treeBG4)
  [16]     snow_bg_style        u8         ┐ g71 surface bytes (3 bytes)
  [17]     desert_bg_style      u8         │  (setBG3 / setBG6 / setBG7)
  [18]     ocean_bg_style       u8         ┘
```

**Total 19 bytes: 16 (forest) + 3 (surface).**

**Writer** (`save.rs`, after the v17 fishron write):
```rust
// v18 UNION: the g70 forest-set block then the g71 surface style bytes.
for x in world.tree_x { w.write_all(&x.to_le_bytes())?; }   // 3×i32
w.write_all(&world.forest_bg_styles)?;                       // 4×u8
w.write_all(&[world.snow_bg_style, world.desert_bg_style, world.ocean_bg_style])?; // 3×u8
```

**Reader** (`save.rs`, the `if version >= 18` arm):
```rust
if version >= 18 {
    for x in &mut world.tree_x { *x = r.i32()?; }
    for s in &mut world.forest_bg_styles { *s = r.u8()?; }
    world.snow_bg_style = r.u8()?;
    world.desert_bg_style = r.u8()?;
    world.ocean_bg_style = r.u8()?;
}
```

### Back-compat

- **Pre-v18 files** (version ≤ 17): the whole 19-byte tail sits unread. The World
  keeps its `new`/Reset defaults — `tree_x = [width; 3]`, `forest_bg_styles = [0; 4]`,
  `snow/desert/ocean_bg_style = 0`. This is exactly what each branch's own pre-v18
  arm already does; the union preserves both.
- **A g70-only v18 file** (16-byte tail) read by the union loader: the forest block
  parses, then the three surface reads run off the end → `Reader` EOF error. **Not
  back-compat** — but no such file ships: both branches are unmerged task work, and
  the union is the *first* v18 to land on main. The contract is forward-only from the
  merge; there is no released v18 to grandfather.
- **A g71-only v18 file** (3-byte tail): same — the union loader reads 12 i32 bytes
  of forest data from the 3 surface bytes + whatever follows → garbage/EOF. Again,
  no released file exists.

**Merge instruction:** when g70 and g71 reconcile on main, take the union writer +
reader above (NOT `--ours`/`--theirs`), keep `VERSION = 18`, and update the two
back-compat tests' tail-length constants: g70's `w4_g70_the_forest_set_block_round_trips_in_v18`
truncates **16** bytes (g70 `save.rs:2212`) and g71's
`w4_g71_snow_desert_ocean_styles_round_trip_in_v18` truncates **3** (`save.rs:2210`)
— both become **19**. The other truncation-arithmetic sites that count the v18 tail
also move from 3 (g71) / 16 (g70) to 19: the v12 `let tail = … + 3` (`save.rs:1300`),
the v14 `truncate(len - 1 - 5 - 1 - 3)` (`:1518`), and the `EMPTY_V16_TAIL`-family
constants (`:1238`, `:1328`). The `expected_flags_block` byte-table test (g71 phase
4b, `2b57b362`) and g70's forest round-trip both keep their assertions; only the tail
length moves.

**Why forest-first:** the order is arbitrary but must be *fixed and shared*. Forest
first matches the C# `.wld` field order (treeX/treeStyle at `WorldFile.cs:1253-1259`
precede the setBG bytes at `:1318-1325`), so the union tail reads in the same
relative order as the vanilla flags block — the least-surprising choice for the
next lane that extends it.

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

**Port.** `crates/tr-core/src/worldgen/bg_rolls.rs` (new, `#![allow(dead_code)]`,
UNWIRED): `roll_random_forest_bg_style`, `randomize_backgrounds_corruption`,
`randomize_backgrounds_desert`, `roll_snow_bg_style`, `randomize_backgrounds`
(returns a `BackgroundStyles` struct), `randomize_tree_style`,
`randomize_cave_backgrounds` (returns `CaveBackgrounds`), and
`is_background_considered_the_same` (`:8061-8095`, the 3≡31/5≡51/7≡71-73
equivalence the player-driven re-roll uses). All are **pure functions over a
caller-supplied `Rng`** — no `World` writes, no global state — so a future worldgen
lane wires them with its own `pass_rng` slot and owns the `setBG`/`Main.treeX`/…
writes. 10 witnesses WRITTEN NOT RUN, expectations derived from an independent
Python xorshift64 reference (seed 42: forest `[2,7,51,11]`, corrupt 51, jungle 5,
snow 5, hallow 4, crimson 4, desert 4, ocean 2, mushroom 2, underworld 2; treeX
`[1546,5165,6282]`/treeStyle `[2,3,0,5]` at width 8400; caveBackX
`[846,3324,5930]`/caveBackStyle `[1,4,3,0]`, ice 3/hell 2/jungle 0).

**Why UNWIRED (two blockers, both measured/cited):**

1. **RNG-shift (the g72 flag).** The port's tail (`step_world_tail`,
   `pipeline.rs:2685`) draws only moon type + hell back style from `streams.rng`,
   pinned by `worldgen_rolls_hell_back_style` (`render/bg.rs:1715`) and the frozen
   `world_hash` constants (`SEED42_512 = 124297949`, `DEFAULT_WORLD = 3647283075`).
   Inserting the full rolls mid-tail shifts every later `streams.rng` draw
   (dungeon-side, jungle-origin, the seed-finish passes) and moves both pins. The
   pin-move note now lives at `step_world_tail` (`pipeline.rs`): wiring is a
   worldgen lane's call — re-pin the tail in the same move.
2. **Atlas overflow (measured this lane).** A rolled non-default surface style needs
   its `Background_<id>` sheets packed (`assets.rs::load_world_sheets`). The single
   8192 px atlas already packs the style-0 set to **7847 px** (8097 sheets, measured
   with the `tools/atlas_sim.js` skyline twin on this branch's asset set). Non-default
   styles add far more than the 345 px headroom:

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

   So wiring the rolls live would overflow the atlas for most worlds. Needs the
   S1-G-13 background-target family (the C# draws surface bgs through
   `Main.spriteBatch` under the sky matrix, outside the tile atlas) or an atlas
   re-budget — the same blocker g72 hit for the cavern/hell sheets.

**Ownership note.** The cave/ice/jungle back-style fields stay **unowned** in the
save layer (`wld/sections/flags.rs:48-54` writes zeros, `:382-384` skip-reads) —
this lane does not own them, because the rolls are unwired and g72's
`render/bg_styles.rs` consumes them as `BgStylePickInput` parameters, not `World`
fields. The future wiring lane owns the fields + the `.wld`/`.trwld` persistence
together with the tail re-pin. `hell_back_style` is the one exception: already
owned (the tail draws it), and the port's `randomize_cave_backgrounds` reproduces
its `Next(3)` (`:8247`) for when the full roll lands.

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

- `crates/tr-core/src/worldgen/bg_rolls.rs` — NEW, the UNWIRED style rolls + 10
  witnesses (commit `6b4f9d27`).
- `crates/tr-core/src/worldgen/mod.rs` — `mod bg_rolls;` registration.
- `crates/tr-core/src/worldgen/pipeline.rs` — the pin-move note at `step_world_tail`.
- `docs/research/92-doc-sweep-bg.md` — this report.

**Verification:** code-only mode — no cargo/tests/graphify run. The 10 `bg_rolls`
witnesses are WRITTEN NOT RUN; their expectations come from an independent Python
xorshift64 reference (`Rng(seed ^ fnv1a64(name))`, high-32 word, `range = lo + word %
(hi-lo)`) fed through the verbatim C# roll logic, not from executing the Rust. The
atlas heights come from the `tools/atlas_sim.js` skyline twin run on this branch's
asset set (node, not cargo). The sheet dims come from the prebuilt `tr-content`
`dims` example probe. The UNION v18 contract is a *merge instruction* — the two
branches' `save.rs` tails are not reconciled here (each stays on its own branch);
the reconciliation happens when they land on main per §1.
