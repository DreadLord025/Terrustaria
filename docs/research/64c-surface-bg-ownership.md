# 64c — the surface-bg draw ownership (W4-g72 closeout verification)

**Branch:** `task/W4-g72-bg-layers-3` (closeout). **Mode:** code-only.
**Question (the Lead's item 3):** does our live cave-chain's SURFACE arm read
the g71-shape style tables, or does it re-port/pin them? **Verdict: the five
v16-owned variants read the g71-shape `setBG` expansion (LIVE); the
forest/desert/snow/ocean style tables are g70/g71's and stay style-0 PINNED
here — this lane does NOT re-port them.** Cross-note published so the merge
lifts the pins to g71's reads without a round-trip.

## 1. The C# dispatch (the source of truth)

`Main.DrawSurfaceBG`'s front-layer loop (`Main.cs:59529-59597`) dispatches
each biome chain with a `Main.<biome>BG` id array that `WorldGen.setBG`
(`WorldGen.cs:7215-7768`) fills from the world's style byte:

| l | Chain | C# id array | setBG slot | Style table owner |
| --- | --- | --- | --- | --- |
| 0/10/11/12 | `DrawSurfaceBG_Forest` | `treeBGSet1..4` | 0/10/11/12 (`SetForestBGSet`, `:7639-7768`) | **g70** |
| 1 | `DrawSurfaceBG_Corrupt` | `corruptBG` | 1 (`:7235-7274`) | v16-owned (main) |
| 2 | `DrawSurfaceBG_Desert` | `desertBackgroundSet.Pure.Backgrounds` | 6 (`:7501-7546`) | **g71** |
| 3 | `DrawSurfaceBG_Jungle` | `jungleBG` | 2 (`:7276-7316`) | v16-owned (main) |
| 5/13/14 | `DrawSurfaceBG_GoodEvilDesert` | the evil desert sets | 6's evil variants | **g71** |
| 6 | `DrawSurfaceBG_Hallow` | `hallowBG` | 4 (`:7424-7459`) | v16-owned (main) |
| 7 | `DrawSurfaceBG_Snow` | `snowBG` | 3 (`:7317-7423`) | **g71** |
| 8 | `DrawSurfaceBG_Crimson` | `crimsonBG` | 5 (`:7460-7500`) | v16-owned (main) |
| 9 | `DrawSurfaceBG_Mushroom` | `mushroomBG` | 8 (`:7578-7608`) | v16-owned (main) |
| (far) | ocean horizon | `oceanBG` | 7 (`:7548-7576`) | **g71** |

## 2. Our live reads (verified at this probe)

`render/sky.rs`'s front-loop (the `draw_sky` surface arm, live since S1-C-9):

- **The five v16-owned variants READ the g71-shape expansion:**
  `world.corrupt_bg()` / `jungle_bg()` / `hallow_bg()` / `crimson_bg()` /
  `mushroom_bg()` (sky.rs:1268/:1330/:1374/:1439/:1491) →
  `tr_core::world::surface_bg_texture_ids(bg, style)` (world.rs:1400, the
  `setBG` slots 1/2/4/5/8 style→ids tables, v16-owned on main). These are
  NOT style-0 pins — a non-default style byte expands to its variant rows
  (witnessed: `w4_g72_the_surface_variants_read_the_g71_shape_set_bg_tables`).
  g71's own draw-side reads the SAME shape through the SAME function (its
  world.rs keeps `surface_bg_texture_ids` and extends it with slot 3) — so
  the five variants are already g71-compatible; the merge keeps one table.
- **The forest/desert/snow/ocean chains are STYLE-0 PINNED here:**
  forest `[50, 51, 52]` (sky.rs:1233), desert `[21, 20, -1]` (:1303), snow
  `[37, 38, 39]` (:1412), ocean `Background_28` (the `sky.ocean` load,
  :471). Their style tables are g70/g71's:
  - g70's `world.forest_bg(n)` → `forest_bg_set(style)` (its world.rs:1448/
    :1524, `SetForestBGSet` slots 0/10/11/12) returns `(mnt[2], tree[3])`.
  - g71's `world.snow_bg()` / `desert_bg_set()[0]` / `ocean_bg()` (its
    sky.rs:1515/:1343/:1198) read `surface_bg_texture_ids(3, ..)` /
    `desert_bg_set(style)` / `ocean_bg_texture_id(style)` (its world.rs).
  This lane does NOT re-port those tables (the Lead's "SURFACE styles
  g71-owns draw-tables"); the pins lift to g71's/g70's reads at the merge.

## 3. The merge cross-note (mechanical)

- The five v16-owned variants: keep ONE `surface_bg_texture_ids` (g71's
  superset adds slot 3; our slots 1/2/4/5/8 are identical rows — verified
  against g71's world.rs at `f08e8d41e`). No conflict.
- The four pinned chains: replace the style-0 id literals with g70's
  `world.forest_bg(n)` (forest, four sets) and g71's `world.snow_bg()` /
  `desert_bg_set()[variant]` / `ocean_bg()` (snow/desert/ocean) once those
  lanes' World fields + tables land. The `bg_offsets`/`layer_y_offset`
  reduced tables (bg.rs:1302/:1356) already carry the variant ids the five
  style tables reach; g71's fuller tables extend them (its research/92 §3).
- The underground-desert cavern style is OURS (the cave machine, no
  separate desert arm in `PickUndergroundBackgroundStyle`, g71 research/92
  §4 verified) — not part of this surface cross-note.

## 4. Witnesses

1 witness WRITTEN NOT RUN
(`w4_g72_the_surface_variants_read_the_g71_shape_set_bg_tables`, bg.rs):
the five variants' style-0 + one non-default row each, and the slot
0/3/6/7/9/10/11/12 `None` verdicts marking the g70/g71-owned tables as
unported HERE.