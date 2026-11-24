# 86 — WorldGen passes lane-4 reconcile (d104 vs the g37 mop-up + siblings)

> Wave 4 · 2026-11-21 · code-only reconcile (no cargo/tests/graphify on this
> branch) · lane `task/W4-d104-worldgen-passes-4` @ tip `8a229c3e` (9/9).
> Read against the PUSHED tips of every worldgen lane (no code touched on any
> sibling). Parent census: `docs/research/12-worldgen.md`; this lane's port
> notes: `docs/research/12d-worldgen-passes-4.md`.
>
> Scope (per the dispatch addendum): (1) d104 passes **28-32** vs g37 **B5-B9**
> arm-by-arm — which port is faithful; (2) the KEEP recommendation; (3) d104
> **33-36** uniqueness vs every other pushed lane; (4) the pipeline final-order
> union table (all worldgen lanes, full ordered list, owner-branch per step).

## 0. Pushed tips read (the reconcile baseline)

| lane | branch | tip | lane doc | landed classic rows |
| --- | --- | --- | --- | --- |
| d101 | `task/W4-d101-worldgen-passes-1` | `4194acf8` | `12b` | 9 (#1-9 + Granite row-split) |
| d102 | `task/W4-d102-worldgen-passes-2` | `29e8859b` | `12a` | 9 (Silt…Wood Tree Walls) |
| d103 | `task/W4-d103-worldgen-passes-3` | `8be04cb0` | `12-worldgen` split | 9 (Oasis…Floating Island Houses) |
| **d104** | `task/W4-d104-worldgen-passes-4` | **`8a229c3e`** | `12d` | **9 (#28-36)** |
| d105 | `task/W4-d105-worldgen-passes-5` | `dd43734a` | `12e` | 4 (Guide/Sunflowers/Herbs/Dye Plants) |
| d106 | `task/W4-d106-worldgen-passes-6` | `29faa099` | `12f` | 4 (Grass Wall/Settle Again/Tile Cleanup/Stalac) |
| f24 | `task/W4-f24-worldgen-passes-7` | `9f2276d5` | `12k` | 2 GATED (Skyblock/Dither Snake) |
| f25 | `task/W4-f25-worldgen-passes-8` | `0ed1f3cf` | `12g-f25` | 1 (Settle Liquids orphan) |
| g36 | `task/W4-g36-worldgen-missing-a` | `7f848edd` | `12g-g36` | 6 (Shimmer…Life Crystals) |
| **g37** | `task/W4-g37-worldgen-missing-b` | **`1975f559`** | `12h` | **9 (B1-B9)** |
| g38 | `task/W4-g38-worldgen-missing-c` | `8c9b7b60` | `12i` | 1 (C1 Place Fallen Log) |
| g39 | `task/W4-g39-worldgen-missing-d` | `bab25558` | `12j` | 0 passes (H1-H15 helpers, unwired) |
| d107 | `task/W4-d107-worldgen-special-seeds` | `26672912` | — | seed arms only (no classic rows) |

All lanes share base `a725acb0`. The 51-classic-pass census is identical across
every lane doc; the mop-up quartering (g36/g37/g38/g39) reassigns the *pending*
d10x overlaps, and every mop-up doc defers the overlap to the merger.

### The overlap chain (why this reconcile exists)

- **d103 #23-27 = g37 B1-B5**: Statues `:16853`, Surface Chests `:17130`,
  Water Chests `:17245`, Cave Walls `:17711`, Floating Island Houses `:17859`.
  Both lanes LANDED all five (duplicate ports).
- **d104 #28-32 = g37 B5-B9**: Floating Island Houses `:17859` (B5), Quick
  Cleanup `:17875` (B6), Hellforge `:18187` (B7), Spreading Grass `:18237`
  (B8), Surface Ore and Stone `:18447` (B9). Both lanes LANDED all five.
- **Floating Island Houses `:17859` is TRIPLE-claimed** (d103, d104, g37).
- **d104 #33 Place Fallen Log `:18513` = g38 C1** (both landed). d104 #34-36
  (Traps/Piles/Spawn Point) are g38 C2-C4 *pending* — g38 never landed them.

## 1. Arm-by-arm: d104 28-32 vs g37 B5-B9

Legend: **d104** = this lane's dedicated module; **g37** = `passes_b.rs`
monolith (one 2634-line file holding B1-B9 + all their helpers). C# anchors
re-read in ≤100-line windows from `decompiled/Terraria/WorldGen.cs`.

### Pass 28 / B5 — Floating Island Houses (`:17859-17874` + `IslandHouse :79203-79484`)

| arm | C# | d104 (`island_houses.rs`) | g37 (`passes_b.rs::island_house`) | faithful? |
| --- | --- | --- | --- | --- |
| two-pass split (islands records sites, houses builds later) | `:13108-13111` ledger | `GenVars.island_house_slots` (x,y,style,sky_lake) | `GenVars.island_house_sites` | both ✓ |
| ±30 dungeon/87/3/83 wall veto | `:79205-79216` | ✓ | ✓ | tie |
| drunk crimson half-split | `:79217-79220` | ✓ (`drunk`/`crimson_left`) | **absent** (classic-only) | d104 |
| direction/width/height rolls | `:79224-79230` | ✓ | ✓ | tie |
| floor scan (192/191/384 excl) | `:79231-79247` | ✓ | ✓ | tie |
| sunplate-202 shell + skyware-82 interior | `:79249-79314` | ✓ | ✓ | tie |
| door (remix 5/38, else 9) | `:79323-79337` | ✓ remix+getGood arms | classic 9 only | d104 |
| chest main-item cycle + style (getGood 2, else 13) | `:79348-79376` | ✓ getGood arm | classic 13 only | d104 |
| fountain (islandStyle>0) + SwitchFountain | `:79377-79393` | ✓ | ✓ | tie |
| windows / table+chairs / banners / palms | `:79395-79483` | ✓ remix furniture arms | classic only | d104 |
| placement helpers | `Place3x2 :52066`, `Place2xX :39663`, door | **shared** `evil::place_3x2`, `temple::place_temple_door`, `structures::place_chest` | **local** `passes_b::place_tile`/`place_2xx`/`add_buried_chest` | d104 (house style) |

**Verdict:** the classic house body is byte-equivalent across d104/d103/g37.
d104 (and d103) additionally carry the drunk/remix/getGood seed arms; g37 B5 is
classic-only. d104 routes through the **shared** placement helpers (the
2026-11-01 house style); g37 re-implements them inside the monolith. **d104
more complete + better-seamed.** (d103's `island_houses.rs` is a near-identical
third copy — the merge keeps ONE, see §2.)

### Pass 29 / B6 — Quick Cleanup (`:17875-17993`)

| arm | C# | d104 (`micro_finish.rs::pass_quick_cleanup`) | g37 (`passes_b.rs::pass_quick_cleanup`) | faithful? |
| --- | --- | --- | --- | --- |
| `tileSolid[137]/[130]` false window | `:17885-17886`→`:17992-17993` | **modelled** via `solid_tile_qc` (137/130 read non-solid) | **unmodelled** (static `td::TILE_SOLID`) | **d104** |
| ocean non-water→water | `:17893-17896` | ✓ | ✓ | tie |
| beach sand drip `Next(4,7)` bound | `:17903` (bound inside the for-condition) | **re-rolls per check** (`limit = j + rng.range(4,7)` in-loop) | **draws once** (`fill = rng.range(4,7)` pre-loop) | **d104** |
| 397/53/495 stop conditions | `:17903` | ✓ | ✓ | tie |
| desert-wall 187/216 retype + liquid arms | `:17911-17933` | ✓ | ✓ | tie |
| surface-sand ±3 wall bleed | `:17934-17957` (`break` exits INNER loop only → last walled column wins) | **inner break only** (last-wins) | **`break 'bleed`** (exits both → first-wins) | **d104** |
| slope/half-brick clear family (225 exemption, 137 veto) | `:17958-17989` | ✓ (`save_slopes` shared from `tile_runner`) | ✓ (`td::SET_SAVE_SLOPES`) | tie |

**Verdict:** three concrete fidelity edges to d104 — the `tileSolid` window, the
per-check `Next(4,7)` re-roll, and the last-wins wall bleed. g37 B6 deviates on
all three (documented in its own note as "unmodelled"/once-draw, but the bleed
`break 'bleed` is an unflagged first-wins divergence). **d104 more faithful.**

### Pass 30 / B7 — Hellforge (`:18187-18234`)

| arm | C# | d104 (`structures.rs::pass_hellforge`) | g37 (`passes_b.rs::pass_hellforge`) | faithful? |
| --- | --- | --- | --- | --- |
| `maxTilesX/200` count + retry loop | `:18192-18233` | ✓ | ✓ | tie |
| wall 13/14 gate + drop scan + `j--` | `:18204-18209` | ✓ | ✓ | tie |
| `PlaceTile(77)` → `Place3x2` | `:18210`, `:60079-60083` | `evil::place_3x2` | `passes_b::place_tile` | tie |
| 10000-failure valve; wall-miss no-bump | `:18211-18232` | ✓ | ✓ | tie |
| **EXECUTED SLOT vs the wall-14 producer** | Underworld `:13645` calls `AddHellHouses()` `:13840` (paints wall 13/14) BEFORE Hellforges `:18187` | `hell forts` step **40** ≪ `hellforge` step **62** → wall 14 exists, gate fires | `hellforge` step **41** BEFORE `hell forts` step **45** → **no wall 13/14 yet, gate never fires** | **d104** |

**Verdict — the decisive finding.** The port splits the C# Underworld pass into
`ash blobs` (the ash/stone blob carve, `caves::blob_pass`) and `hell forts`
(`structures::pass_hell_forts`, the `AddHellHouses` wall-14 painter). Hellforge's
`wall == 13 || wall == 14` gate can only fire AFTER `hell forts` has run.
- **d104** orders `hell forts` (40) → … → `living trees` (59) → `floating island
  houses` (60) → `quick cleanup` (61) → `hellforge` (62). Wall 14 is present;
  forges place. ✓ matches vanilla (Underworld ≪ Hellforges).
- **g37** orders `pots` (40) → `hellforge` (41) → `spreading grass` (42) →
  `surface ore and stone` (43) → `beehives` (44) → `hell forts` (45). At
  hellforge time NO hell-house wall exists, so every forge spins the retry to
  the 10000 valve and places nothing. **g37 B7 is a no-op as ordered** — an
  ordering bug, not a body bug (the body itself is faithful).

**d104 correct; g37 B7 must move after `hell forts` at the merge.**

### Pass 31 / B8 — Spreading Grass (`:18237-18353` classic arm)

| arm | C# | d104 (`surface.rs` + `evil::spread_evil_grass`) | g37 (`passes_b.rs` + `spread_surface_grass`) | faithful? |
| --- | --- | --- | --- | --- |
| sweep 1: jungle-60 spread + ±3 flavour scan (53 latch, infection/jungle overrides) | `:18242-18320` | ✓ | ✓ | tie |
| sweep 2: flag2 open-sky latch, worldSurfaceHigh break | `:18321-18353` | ✓ | ✓ | tie |
| **the SpreadGrass call** | `:18334` `SpreadGrass(num3, num4)` = **default `repeat:true`**; the `repeat:false` at `:18339` is ONLY the dead `catch` arm (SpreadGrass has its own try/catch `:75148/:75254`, never throws) | calls `spread_evil_grass(…, repeat:false)` → **no recursion** (single cell) | `spread_surface_grass` recurses `grassSpread<1000` → **repeat:true behaviour** | **g37** |
| RNG | none (tile-only) | none | none | tie |

**Verdict — the reverse edge.** d104's port note cites `:18334-18338` and passes
`repeat:false`, but `:18334` is the LIVE call (default `repeat:true`); `:18339`
is the unreachable catch fallback. So **d104 pass 31 under-spreads** (writes one
grass cell, no 3×3 recursion), while **g37 B8 is faithful** (bounded recursion).
This is the one arm where g37 beats d104. **The merge must adopt g37's recursive
spread for pass 31** (flip d104's call to `repeat:true`, or reuse
`spread_surface_grass`). d104's `spread_evil_grass` already takes the `repeat`
parameter, so the fix is a one-argument change at the call site.

### Pass 32 / B9 — Surface Ore and Stone (`:18447-18511` + `OrePatch :10579-10705` + `StonePatch :10242-10330`)

| arm | C# | d104 (`surface.rs`) | g37 (`passes_b.rs`) | faithful? |
| --- | --- | --- | --- | --- |
| ore count `Next(w*5/4200, w*10/4200)`, ±200 ledger, [0.48,0.52] centre | `:18452-18483` | ✓ | ✓ | tie |
| stone count `Next(1, w*7/4200)`, ±100, [0.47,0.53] | `:18484-18510` | ✓ | ✓ | tie |
| OrePatch tier roll | `:10581-10585` `SavedOreTiers.Copper/Iron` | **tier params** (`copper`,`iron`) | hardcoded 6/7 | d104 (minor) |
| OrePatch floor/grass/wall/body vetoes, Next(2) sink, Next(8,13) walk, drifting blob | `:10586-10703` | ✓ | ✓ | tie |
| StonePatch 199/23 veto, blob retype, 186+185 decor | `:10242-10330` | ✓ | ✓ | tie |
| 186 pile / 185 small-pile helpers | `Place3x2 :52066`, `PlaceSmallPile :46815` | **shared** `evil::place_3x2`, `spider_gem_moss::place_small_pile` (type param) | **local** `passes_b::place_tile`/`place_small_pile` | d104 (house style) |
| GenVars ledger | `GenVars.cs:240-244`, `:461` (maxOrePatch 50) | `ore_patch_x: Vec` | `ore_patch_x: [i32;50]` + `num_ore_patch` | tie |
| sub-vanilla width guard | C# `Next(1, w*7/4200)` throws ≤600 | `w*7/4200 <= 1` (≤600) | `w <= 760` + y-band guard | tie (both skip) |

**Verdict:** behaviourally equivalent on a classic world; d104 is tier-aware and
uses the shared helpers. **d104 marginally better-seamed; tie on fidelity.**

### Arm-by-arm summary

| pass | more faithful | why |
| --- | --- | --- |
| 28 / B5 | **d104** | seed arms present + shared helpers (g37 classic-only monolith) |
| 29 / B6 | **d104** | tileSolid window + per-check Next(4,7) + last-wins bleed |
| 30 / B7 | **d104** | correct slot after `hell forts`; g37 hellforge is a no-op as ordered |
| 31 / B8 | **g37** | recursive SpreadGrass (repeat:true); d104 ports the dead catch arm |
| 32 / B9 | tie (d104 seamed) | equivalent; d104 tier-aware + shared helpers |

## 2. KEEP recommendation

**KEEP d104's dedicated modules + GenVars slots as the merge base for passes
28-36**, with ONE targeted correction:

1. **Keep** `island_houses.rs`, `micro_finish.rs::pass_quick_cleanup`,
   `structures.rs::pass_hellforge`/`pass_piles`, `surface.rs::pass_spreading_grass`/
   `pass_surface_ore_and_stone`/`pass_place_fallen_log`/`pass_spawn_point`,
   `traps.rs`, and the `pipeline.rs` GenVars slots (`island_house_slots`,
   `ore_patch_x`, `log_anchor`, `shimmer_position`, `boulder_pets_placed`).
   Rationale: correct Hellforge ordering (§1 B7), the three Quick-Cleanup
   fidelity edges (§1 B6), the seed arms in the island house (§1 B5), and the
   shared-helper seams (`evil::place_3x2` generalised, `spider_gem_moss::
   place_small_pile` type-param, `temple::place_temple_door`, `structures::
   place_chest`) that match the house style.
2. **Fix pass 31 at the merge:** flip d104's `spread_evil_grass(…, false)` call
   to `repeat:true` (or adopt g37's `spread_surface_grass` recursion). This is
   the single arm where g37 is faithful and d104 is not (§1 B8). One-argument
   change; the `repeat` parameter already exists.
3. **Dedupe the g37 monolith onto d104's modules.** g37 `passes_b.rs` B5-B9
   duplicate d104 28-32; drop them in favour of d104's. g37 B1-B4 (Statues,
   Surface Chests, Water Chests, Cave Walls) duplicate **d103**'s
   `statues.rs`/`buried_chest.rs`/`water_chests.rs`/`cave_walls.rs` — keep d103's
   dedicated modules (same house-style argument), drop g37's. The triple-claimed
   Floating Island Houses keeps ONE copy (d104's, which carries the seed arms).
4. **General-helper dedupe target = g39 `place_helpers.rs`** (H1-H15, unwired).
   d104's shared `evil::place_3x2` / `spider_gem_moss::place_small_pile` and
   g37/g38's local copies all converge onto g39's general `Place3x2`/`Place2x2`/
   `Place1x1`/`PlaceSmallPile` wrappers at the merge (g39's doc explicitly
   leaves the per-module narrow copies for the merger to dedupe).
5. **g38 C1 Place Fallen Log** duplicates d104 pass 33 (§3). Keep d104's; g38's
   `place_tile_fallen_log` PlaceTile preamble (`:59345` existing-488 veto +
   `:59473` inactive-cell clear) is structurally more complete but behaviourally
   inert on the always-air seat — optionally fold it in, not required.

**Do NOT keep g37 as the base:** its Hellforge ordering bug (§1 B7) and three
Quick-Cleanup divergences (§1 B6) outweigh its single Spreading-Grass edge,
which is a one-line fix on d104.

## 3. d104 33-36 uniqueness (verified against every pushed tip)

Scripted probe of every lane's `EXECUTED_PIPELINE` (the `steps-*.txt` extracts)
for the four vanilla names:

| d104 pass | vanilla / anchor | other landed lanes | unique? |
| --- | --- | --- | --- |
| 33 Place Fallen Log | `FallenLogsAndWaterFeatures` `:18513` | **g38 C1** (`8c9b7b60`, `surface.rs::place_fallen_log`) | **NO — duplicated by g38** |
| 34 Traps | `Traps` `:18657` | none (g38 C2 pending, never landed) | **YES** |
| 35 Piles | `Piles` `:18788` | none (g38 C3 pending) | **YES** |
| 36 Spawn Point | `SpawnPoint` `:19460` | none (g38 C4 pending) | **YES** |

Cross-checks (no Traps/Piles/Spawn Point row on any of these pushed tips):
- **d105** `dd43734a`: guide/sunflowers/herbs/dye plants only.
- **d106** `29faa099`: grass wall/settle liquids again/tile cleanup/stalac only.
- **f24** `9f2276d5`: skyblock/dither snake (gated) only.
- **g36** `7f848edd`: shimmer/clean up dirt/dirt rock wall runner/wood tree
  walls/wall variety/life crystals only.
- **g37** `1975f559`: B1-B9 ends at Surface Ore and Stone `:18447` — does NOT
  reach Fallen Log/Traps/Piles/Spawn Point.
- **g38** `8c9b7b60`: C1 Place Fallen Log ONLY (C2-C8 pending in its ledger).
- **g39** `bab25558`: helper lane, zero executed pass rows.

**Correction to the addendum's "33-36 UNIQUE":** pass **33 is NOT unique** — g38
C1 landed the same Place Fallen Log. Passes **34/35/36 (Traps/Piles/Spawn Point)
ARE unique to d104** across all pushed lanes. The merge keeps d104's 33 (dedupe
g38 C1) and d104's 34-36 outright.

### Pass 33 d104 vs g38 C1 (the one landed duplicate)

| arm | C# | d104 (`surface.rs::pass_place_fallen_log`) | g38 (`surface.rs::place_fallen_log`) |
| --- | --- | --- | --- |
| count `w/2100 + Next(-1,2)`, 30000/5000 budgets, skyblock /10 | `:18525-18537` | ✓ | ✓ |
| ±400 column, Next(10,worldSurface) row, centre re-roll | `:18541-18558` | ✓ | ✓ |
| drop scan + worldSurface-10 veto | `:18562-18577` | ✓ | ✓ |
| ±50 cloud/sand/dungeon/evil scan + ±10 overhead clear | `:18578-18635` | ✓ | ✓ |
| grass triplet (bees 60/frozen 147/desert 53) + liquid gate | `:18637` | ✓ | ✓ |
| `PlaceTile(488)` | `:18640`, `:59888-59891` | `evil::place_3x2` direct | `place_tile_fallen_log` (PlaceTile preamble `:59345`/`:59473` + Place3x2) |
| 1-in-2 logX/logY record | `:18643-18647` | `Option<(i32,i32)>` | `(i32,i32)` |
| width guard draw schedule | count roll `:18525` first | guard ≤800 BEFORE count roll | count roll THEN margin guard (matches C# order) |

Both faithful on a vanilla-width world (the seat is always air, so g38's extra
PlaceTile preamble is inert). g38 matches the C# count-roll-before-guard draw
order; d104 guards first (sub-vanilla only). **Keep d104's** for lane
consistency; g38's preamble is an optional fold-in.

## 4. Pipeline final-order union table (all worldgen lanes)

The canonical vanilla order = the 109 `VANILLA_CHAIN` declarations sorted by
`AddGenerationPass` anchor (`WorldGen.cs`). Every lane's `EXECUTED_PIPELINE`
groups some families early (e.g. the chest family, the micro-biome family) and
therefore deviates from this anchor order; **the merge target is this vanilla
anchor order**, reconciling each lane's grouped deviations. Owner-branch column
lists every lane whose pushed tip executes that vanilla name (`base` = present
in the unmodified 66-step base on all lanes; `gated` = special-seed-gated).

Counts (updated by the §6 addendum, 2026-11-21): **55 base** + **50
landed-new** + **3 gated** (Skyblock, Dither Snake, and the dead
skyblock-Jungle re-registration) + **1 UNLANDED** (Final Cleanup) = 109
decls. The original reconcile counted 44 landed-new + 7 unlanded; d105
tip `96f65cd6` landed Flowers/Mushrooms/Random Gems/Moss Grass/Larva and
d106 tip `e2c2609c` landed Remove Broken Traps (§6.3/§6.4). The 50
landed-new + 55 base = 105 classic executed names; + 2 real gated = 107;
the 51-pass census closes as Final Cleanup lands.

| # | anchor | vanilla pass | status | owner-branch(es) | executed step |
| --- | --- | --- | --- | --- | --- |
| 1 | :11504 | Terrain | base | base | `terrain height map` |
| 2 | :11509 | Jungle | gated | all (dead re-reg) | `jungle` |
| 3 | :11511 | Skyblock | gated | f24 | `skyblock` |
| 4 | :11517 | Dunes | base | base | `dunes` |
| 5 | :11580 | Ocean Sand | base | base | `ocean sand` |
| 6 | :11689 | Sand Patches | landed | d101 | `sand patches` |
| 7 | :11718 | Tunnels | base | base | `cave fields` |
| 8 | :11784 | Mount Caves | landed | d101 | `mount caves` |
| 9 | :11858 | Dirt Wall Backgrounds | landed | d101 | `dirt wall backgrounds` |
| 10 | :11896 | Rocks In Dirt | base | base | `stone-in-dirt blobs` |
| 11 | :11926 | Dirt In Rocks | base | base | `dirt-in-stone blobs` |
| 12 | :11959 | Clay | landed | d101 | `clay` |
| 13 | :12008 | Small Holes | landed | d101 | `small holes` |
| 14 | :12067 | Dirt Layer Caves | landed | d101 | `dirt layer caves` |
| 15 | :12107 | Rock Layer Caves | landed | d101 | `rock layer caves` |
| 16 | :12162 | Surface Caves | base | base | `surface caves` |
| 17 | :12271 | Wavy Caves | base | base | `wavy caves` |
| 18 | :12312 | Generate Ice Biome | landed | d101 | `generate ice biome` |
| 19 | :12423 | Grass | base | base | `micro grass` |
| 20 | :12466 | Jungle | base | base | `jungle` |
| 21 | :12468 | Mud Caves To Grass | base | base | `mud caves to grass` |
| 22 | :12500 | Full Desert | base | base | `full desert` |
| 23 | :12544 | Mushroom Patches | base | base | `mushroom patches` |
| 24 | :12757 | Marble | base | base | `marble` (d101 splits Granite out) |
| 25 | :12839 | Granite | landed | d101 | `granite` |
| 26 | :12927 | Floating Islands | base | base | `floating islands` |
| 27 | :13123 | Dirt To Mud | base | base | `dirt to mud` |
| 28 | :13143 | Silt | landed | d102 | `silt` |
| 29 | :13182 | Shinies | base | base | `ores and gems` |
| 30 | :13607 | Webs | landed | d102 | `webs` |
| 31 | :13645 | Underworld | base | base | `ash blobs` (+ `hell forts` = AddHellHouses) |
| 32 | :13866 | Corruption | base | base | `evil` |
| 33 | :14556 | Lakes | base | base | `liquids` |
| 34 | :14697 | Slush | landed | d102 | `slush` |
| 35 | :14755 | Dual Dungeons Dither Snake | gated | f24 | `dither snake` |
| 36 | :14757 | Dungeon | base | base | `dungeon` |
| 37 | :14873 | Mountain Caves | landed | d102 | `mountain caves` |
| 38 | :14888 | Beaches | base | base | `beaches` |
| 39 | :15050 | Gems | landed | d102 | `gems` |
| 40 | :15138 | Gravitating Sand | base | base | `gravitating sand` |
| 41 | :15167 | Create Ocean Caves | base | base | `ocean caves` |
| 42 | :15195 | Shimmer | landed | d102,g36 | `shimmer` |
| 43 | :15246 | Clean Up Dirt | landed | d102,g36 | `clean up dirt` |
| 44 | :15360 | Pyramids | base | base | `pyramids` |
| 45 | :15461 | Dirt Rock Wall Runner | landed | d102,g36 | `dirt rock wall runner` |
| 46 | :15476 | Living Trees | base | base | `living trees` |
| 47 | :15717 | Wood Tree Walls | landed | d102,g36 | `wood tree walls` |
| 48 | :15750 | Altars | base | base | `altars` |
| 49 | :15810 | Wet Jungle | base | base | `wet jungle` |
| 50 | :15834 | Jungle Temple | base | base | `lihzahrd temple` |
| 51 | :15927 | Hives | base | base | `beehives` |
| 52 | :15977 | Jungle Chests | base | base | `jungle shrines` |
| 53 | :16132 | Settle Liquids | landed | f25 | `settle liquids` |
| 54 | :16211 | Remove Water From Sand | base | base | `remove water` |
| 55 | :16243 | Oasis | landed | d103 | `oasis` |
| 56 | :16266 | Shell Piles | base | base | `spike pits` / `shell piles` |
| 57 | :16419 | Smooth World | base | base | `slopes` |
| 58 | :16610 | Waterfalls | landed | d103 | `waterfalls` |
| 59 | :16667 | Ice | base | base | `fragile ice` |
| 60 | :16697 | Wall Variety | landed | d103,g36 | `wall variety` |
| 61 | :16755 | Life Crystals | landed | d103,g36 | `life crystals` |
| 62 | :16853 | Statues | landed | d103,g37 | `statues` |
| 63 | :16971 | Buried Chests | base | base | `cave houses` |
| 64 | :17130 | Surface Chests | landed | d103,g37 | `surface chests` |
| 65 | :17211 | Jungle Chests Placement | base | base | `chests` |
| 66 | :17245 | Water Chests | landed | d103,g37 | `water chests` |
| 67 | :17356 | Spider Caves | base | base | `spider caves` |
| 68 | :17428 | Gem Caves | base | base | `gem caves` |
| 69 | :17472 | Moss | base | base | `moss` |
| 70 | :17701 | Temple | base | base | `temple part 2` |
| 71 | :17711 | Cave Walls | landed | d103,g37 | `cave walls` |
| 72 | :17840 | Jungle Trees | base | base | `underground jungle trees` |
| 73 | :17859 | Floating Island Houses | landed | d103,d104,g37 | `floating island houses` |
| 74 | :17875 | Quick Cleanup | landed | d104,g37 | `quick cleanup` |
| 75 | :17996 | Pots | base | base | `pots` |
| 76 | :18187 | Hellforge | landed | d104,g37 | `hellforge` (must follow `hell forts`) |
| 77 | :18237 | Spreading Grass | landed | d104,g37 | `spreading grass` |
| 78 | :18447 | Surface Ore and Stone | landed | d104,g37 | `surface ore and stone` |
| 79 | :18513 | Place Fallen Log | landed | d104,g38 | `place fallen log` |
| 80 | :18657 | Traps | landed | d104 | `traps` |
| 81 | :18788 | Piles | landed | d104 | `piles` |
| 82 | :19460 | Spawn Point | landed | d104 | `spawn point` |
| 83 | :19589 | Grass Wall | landed | d106 | `grass wall` |
| 84 | :19685 | Guide | landed | d105 | `guide` |
| 85 | :19898 | Sunflowers | landed | d105 | `sunflowers` |
| 86 | :19940 | Planting Trees | base | base | `trees` |
| 87 | :19964 | Herbs | landed | d105 | `herbs` |
| 88 | :19981 | Dye Plants | landed | d105 | `dye plants` |
| 89 | :19995 | Webs And Honey | base | base | `webs and honey` |
| 90 | :20064 | Weeds | base | base | `surface flora` |
| 91 | :20122 | Glowing Mushrooms and Jungle Plants | base | base | `mushroom and jungle plants` |
| 92 | :20165 | Jungle Plants | base | base | `jungle plants` |
| 93 | :20193 | Vines | base | base | `vines` |
| 94 | :20447 | Flowers | landed | d105 | `flowers` (§6.4) |
| 95 | :20599 | Mushrooms | landed | d105 | `mushrooms` (§6.4) |
| 96 | :20697 | Gems In Ice Biome | base | base | `exposed gems` |
| 97 | :20729 | Random Gems | landed | d105 | `random gems` (§6.4) |
| 98 | :20769 | Moss Grass | landed | d105 | `moss grass` (§6.4) |
| 99 | :20813 | Muds Walls In Jungle | base | base | `muds walls in jungle` |
| 100 | :20868 | Larva | landed | d105 | `larva` (§6.3 — g41's is the NTB seed arm, NOT this slot) |
| 101 | :20901 | Micro Biomes | base | base | `thin ice`/`campsites`/`sword shrines`/`mining explosives`/`mahogany trees` |
| 102 | :21246 | Settle Liquids Again | landed | d106 | `settle liquids again` |
| 103 | :21310 | Cactus, Palm Trees, & Coral | base | base | `cacti` |
| 104 | :21520 | Tile Cleanup | landed | d106 | `tile cleanup` |
| 105 | :21943 | Lihzahrd Altars | base | base | `lihzahrd altar` |
| 106 | :21973 | Water Plants | base | base | `water plants` |
| 107 | :22038 | Stalac | landed | d106 | `stalac` |
| 108 | :22121 | Remove Broken Traps | landed | d106 | `remove broken traps` (§6.4) |
| 109 | :22141 | Final Cleanup | UNLANDED | d106 claim | — |

### Ordering reconciliations the merge must apply

1. **Hellforge after `hell forts`** (§1 B7): d104 already correct; g37's row
   must move from before `hell forts` to after it (vanilla Underworld `:13645`
   ≪ Hellforges `:18187`).
2. **Chest family grouping**: g37 moved Surface Chests/Water Chests into the
   port's early chest group (after `chests`, before `spider caves`, commit
   `809a61a8`); d103 keeps them at the vanilla slot (after Statues/cave houses).
   Vanilla order is Statues `:16853` → Buried Chests `:16971` → Surface Chests
   `:17130` → Jungle Chests Placement `:17211` → Water Chests `:17245` → Spider
   Caves `:17356`. The merge restores this anchor order (the isolated per-pass
   streams make either grouping RNG-neutral, but the anchor order is the
   faithful target).
3. **Spawn Point lifts out of `step_world_tail`** (d104 pass 36): the tail keeps
   only the moon/hell draws; the spawn search rides its own row after Piles.
4. **The UNLANDED rows** — originally 7 (Flowers/Mushrooms/Random Gems/
   Moss Grass/Larva/Remove Broken Traps/Final Cleanup); the §6 addendum
   closes six of them (d105 ×5, d106 ×1). Only **Final Cleanup** remains
   (d106 claim) to close the census.

## 5. Verification performed

Code-only reconcile — **no cargo/tests/graphify run** (per the lane discipline).
Evidence gathered by direct reads (no junction recursion):
- `git show origin/task/W4-<lane>:docs/research/<doc>` for all 12 lane docs +
  `git log` tips (the §0 table).
- `git show <tip>:crates/tr-core/src/worldgen/pipeline.rs` → scripted
  `EXECUTED_PIPELINE` extraction (`steps-*.txt`, 66-75 steps/lane) and
  `VANILLA_CHAIN` anchor sort (`vanilla-sorted.txt`, 109 decls) → the §4 union
  table (`union-table.txt`, 55 base / 44 landed / 3 gated / 7 unlanded).
- C# re-read in ≤100-line windows: `WorldGen.cs` `:17875-17995` (Quick Cleanup),
  `:18187-18236` (Hellforge), `:18237-18356` + `:75146-75268` (Spreading Grass /
  SpreadGrass), `:18447-18512` + `:10242-10330` + `:10579-10705` (Surface Ore /
  StonePatch / OrePatch), `:18513-18656` (Fallen Log), `:79203-79400`
  (IslandHouse), `:52066-52165` + `:59860-60090` (Place3x2 / PlaceTile dispatch),
  `:70111-70133` (SolidTile), `:13820-13866` (Underworld→AddHellHouses);
  `Main.cs:7992-7997` (tileDungeon); `TileID.cs:80-100/:468-496`
  (Conversion.Grass / SaveSlopes).
- d104 port bodies read in full (`island_houses.rs`, `micro_finish.rs`,
  `structures.rs`, `surface.rs`, `evil.rs::spread_evil_grass`/`place_3x2`);
  g37 `passes_b.rs` B5-B9 + helpers; g38 `surface.rs::place_fallen_log`;
  d103 `island_houses.rs` (the third island-house copy).

No code changed on any sibling; this branch adds only this document.

## 6. Addendum (2026-11-21, post-reconcile dispatch — DONE)

> The four dispatch items, executed on THIS branch. Tips re-read:
> g37 `53a5b5b5` (its research/75 reconcile — **no research/124 exists**
> on the pushed tip; the "hellforge-graft-patch" the dispatch mentions
> was never pushed, so the graft was re-derived from g37's landed
> `passes_b.rs::place_tile` + a fresh C# read), g41 `b5c8e694`, d105
> `96f65cd6`, d106 `e2c2609c`. Code-only: witnesses written NOT run;
> commit-per-phase.

### 6.1 PASS-31 repeat:true FIX — landed `74103ae4`

The §1 B8 finding, executed: `surface.rs::pass_spreading_grass`'s sweep-2
call flipped `spread_evil_grass(…, false)` → `true` (the one-argument fix
§2 item 2 predicted). C# re-verified in ≤100-line windows: `:18333-18334`
`grassSpread = 0; SpreadGrass(num3, num4)` is the LIVE call; the
signature `:75146` defaults `dirt = 0, grass = 2, repeat = true`; the
`:18339` `repeat: false` call sits ONLY in the `catch` block, and
SpreadGrass wraps its whole body in its own try/catch (`:75148` /
`:75254-75256`) — it never throws, so the catch arm is unreachable. The
recursion `:75241-75245` (`repeat && grassSpread < 1000`, increment
before / decrement after) now fires: a DEPTH cap, and the outer column
loop re-seeds any un-grassed first-surface dirt with a fresh
`grassSpread = 0`.

**Witness re-derivation (the pins that moved):** the classic arm draws
NO RNG, so there are no pinned grass-spread DRAWS to re-derive — only
tile-state witnesses. The old `w4_d104_31_the_surface_dirt_takes_grass`
asserted the buried row-70 cell STAYS dirt — true only under the buggy
single-cell write. Under the faithful recursion the open-flanked column
grasses down to the surface rule's refusal (`j >= worldSurface &&
dirt == 0`, `:75162`): rows 60..99 grass, row 100 stays dirt. The
witness was re-derived to assert exactly that; added
`w4_d104_31_the_grass_spread_recurses` (a 100-cell shelf grasses
end-to-end from ONE seed call) and
`w4_d104_31_the_long_shelf_grasses_past_the_depth_cap` (a 1100-cell
shelf: the depth cap saturates near 1000, the column loop re-seeds —
the C# behaviour). Both world-hash pins were ALREADY stale-by-design
from the pass landing; the fix folds into the same deferred gate
re-pin (stale-by-design notes added to `world_hash.rs`). a == b halves
untouched. `evil.rs`'s doc comment corrected (every generation call
takes the default `true`).

### 6.2 B7 HELLFORGE keep-one RE-ADJUDICATION — graft landed `574f1d58`

**Question:** d104's ORDER correct + g37's PLACEMENT body more faithful?
**Answer: YES to both — graft executed.**

- **ORDER stays d104's.** Re-confirmed against the C#: Underworld
  `:13645` calls `AddHellHouses` (the wall-13/14 painter) long before
  Hellforges `:18187`. d104 orders `hell forts` (40) → … → `hellforge`
  (62); g37's tip still orders hellforge (41) BEFORE hell forts (45) —
  its B7 remains a no-op as ordered. g37's own research/75 §2.7
  CONCEDES the ordering (§4's merge table moves its B6-B9 onto d104's
  slots) and recommends exactly this graft: "KEEP d104's pass shell
  (gate, slot, witness) but GRAFT g37's `place_tile` route for the 77
  write". No research/124 / hellforge-graft-patch exists on `53a5b5b5`.
- **PLACEMENT body: g37 more faithful — verified against a fresh C#
  read, with one refinement.** `PlaceTile(77)` is NOT a bare Place3x2.
  Re-read `:59342-59482` + `:60040-60099` + `:60230-60294`:
  - `:59345` the gen-time existing-488 veto — d104's bare route skipped it.
  - `:59373` the entry gate `forced || EmptyTile || !tileSolid[num]` —
    **`Main.tileSolid[77]` is FALSE** (no `Main.cs` assignment exists;
    verified the port's generated `TILE_SOLID` word 1 bit 13 = 0), so
    the entry ALWAYS admits for 77. g37 research/75's claim that the
    gate "refuses an occupied cell" is overstated for THIS kind — the
    refusal comes from Place3x2's own body scan, which d104 already
    ran. The gate is ported verbatim anyway.
  - `:59473-59476` the inactive-seat `Clear(Tile|TilePaint|Slope)` —
    REAL behavioural arm d104 skipped: a residue seat (stale
    frames/paint/coatings/slope/halfBrick on an inactive cell) kept its
    residue under the bare write. Ported with the full
    `ClearBlockPaintAndCoating` semantics (`Tile.cs:904-909` — paint +
    Glow + Echo), the g38 `place_tile_fallen_log` idiom (g37's own
    slice missed the coatings).
  - `:59477-59482` the `ResetsHalfBrickPlacementAttempt[77]` arm — the
    set is DEFAULT-TRUE (`TileID.cs:497`, 77 not in the false list): an
    active non-frame-important seat loses halfBrick + frames EVEN WHEN
    the placement then fails. Real arm d104 skipped; ported.
  - `:60079-60083` the dispatch: `Place3x2` + `SquareTileFrame`;
    `:60255-60262` the tail: `TruncatesWalls[77]` false
    (`TileID.cs:595`), a second `SquareTileFrame`, `result` = centre
    active. Both re-frames ride `edit::square_tile_frame_gen` (the
    temple precedent — g37's slice does the same).
  - The sub-vanilla `height <= 250 || num <= 0` guard adopted from g37
    (the C# `Next(maxY-250, maxY-30)` band goes negative below 250).
- **Body home:** `structures.rs::place_tile_hellforge` (a kind-77
  PlaceTile slice beside the pass, the g38 fallen-log pattern — NOT
  g37's monolith). The retry loop / draw schedule / wall-miss no-bump /
  10000 valve / skyblock adapter gate are UNCHANGED — the route draws
  no RNG, so no ledger shifts. 3 new witnesses: the Clear-arm residue
  clean, the halfBrick reset on a refused seat (+ the C#
  result-is-centre-active semantics — TRUE even when the forge is
  refused; the pass's own `type == 77` check counts the failure), the
  488 veto. The existing a==b placement witness still holds (the
  route's writes on a clean seat are identical).
- **Merge guidance updated:** §2's "optionally fold in" for g38 C1's
  PlaceTile preamble stays optional (488's seat is always air); the
  B7 graft is now DONE on d104 — the merger takes d104's
  `pass_hellforge` + `place_tile_hellforge` wholesale and drops g37 B7
  entirely. At the g39-helper convergence, `place_tile_hellforge`
  folds into the canonical PlaceTile the same way.

### 6.3 Larva UNLANDED claim — ADJUDICATED: g41 does NOT cover the slot; d105 does

The dispatch's claim ("g41 `b5c8e694` `not_the_bees.rs` Place3x3 larva
landed — verify it covers vanilla Larva pass slot") is **FALSE for the
vanilla slot**:

- g41's `place_larva` (`not_the_bees.rs:818`) is called from the
  `FinishNotTheBees` per-cell walk (`:771-791`) — that is the C#
  **seed-conversion tail** `WorldGen.cs:25608-25629` (the
  `Next(600)/Next(150)/Next(225) == 0` → `PlaceTile(m, n, 231)` rolls
  inside `FinishNotTheBees`), dispatched from `Finish()` `:22435-22439`
  — NOT `GenPassNameID.BeeLarvaInBeehives` `:20868`. g41's own pipeline
  proves it: its `VANILLA_CHAIN` comment says "Two of the twelve have
  no executed step in the port yet (`Larva` — `GenPassNameID.
  BeeLarvaInBeehives` — and `Shimmer`)" and there is NO `step!("larva",
  …)` in its EXECUTED_PIPELINE; the NTB host runs only under the
  NOT_THE_BEES flag (a classic world never enters it).
- **d105 `22a0026c` landed the vanilla slot**: `pass_beehives` publishes
  the `GenVars.larva` ledger (`HiveBiome.CreateStandForLarva`
  `HiveBiome.cs:357-365` — the clamp + the 100-entry cap) instead of
  placing inline; `structures.rs::pass_larva` consumes it at
  `step!("larva", "Larva", …)` after `muds walls in jungle` — the
  `:20874-20896` body (the 3×4 clear: three upper rows deactivated, the
  floor row hive-225 flat; then `PlaceTile(x, y, 231, mute: true)` →
  the Place3x3 `:53534-53633` slice). The `tileSolid[229/232/162]`
  writes (`:20872/:20897-20898`) are boot statics the port's generated
  table already carries (verified `Main.cs:7145/:9612/:10220`).
- **Union-table row 100 updated: landed, owner d105.** g41's NTB larva
  is a seed-arm placement (like d107's seed arms), not a classic row —
  no double-count. At the merge both survive: d105's classic
  ledger-driven pass + g41's flag-gated seed tail. They are NOT
  disjoint by flag (the C# Hives pass `:15927-15929` gates only on
  skyblock, so an NTB world still runs the hive ledger and BOTH larva
  placements fire — the vanilla behaviour); the two routes write
  different sites (the hive centres vs the conversion-tail rolls).

### 6.4 Flowers/Mushrooms/Gems/MossGrass — d105 CLOSED; union 7 → 1

d105 tip `96f65cd6` (read past the `d590d99a` the dispatch cites — two
more commits landed: the lane-5 witness re-pins `93cdd325` + the
`dye_nearby` faithfulness fix `96f65cd6`):

| vanilla / anchor | d105 commit | evidence (tip read) |
| --- | --- | --- |
| Flowers `:20447` | `3980b799` | `step!("flowers", …)` after vines; `pass_rng("Flowers")`; the `:59507-59580` PlaceTile-3 arm + `IsFitToPlaceFlowerIn :58231-58251`; GenVars.logX/logY seam |
| Mushrooms `:20599` | `03efd7bc` | `step!("mushrooms", …)` after flowers; frameX-144 tuft conversion + the tall-270 glowing arm |
| Random Gems `:20729` | `9f7b1a24` | `step!("random gems", …)` after mushrooms; the 178 multi-directional anchors + marble/granite diamond patches |
| Moss Grass `:20769` | `d590d99a` | `step!("moss grass", …)` after random gems; the LongMoss 184 sweep via the PlaceTile-184 two-gate arm |
| Larva `:20868` | `22a0026c` | §6.3 — LANE 5 COMPLETE 9/9 |
| Remove Broken Traps `:22121` | d106 `e2c2609c` | `step!("remove broken traps", …)` after stalac; the `ClearBrokenTraps :26840-26958` two-list BFS |

**Union-table status: 7 UNLANDED → 1** (rows 94/95/97/98/100 → d105,
row 108 → d106; only row 109 **Final Cleanup** `:22141` remains, d106
claim — its pipeline's `VANILLA_CHAIN` still lists it unexecuted).
Counts updated in §4. All six landings carry their own witnesses +
stale-by-design pin notes on their branches (read, not re-run — this
lane stays code-only).

### 6.5 Verification performed (addendum)

Code-only — **no cargo/tests/rustfmt/graphify run** (lane discipline).
Evidence: C# re-reads in ≤100-line windows (`WorldGen.cs:18185-18239`,
`:18300-18359`, `:20860-20959`, `:25595-25639`, `:52066-52165`,
`:59342-59516`, `:59960-60099`, `:60230-60294`, `:70111-70200`,
`:75140-75269`, `:80168-80199`; `Tile.cs:744-803/:900-919`;
`TileID.cs:497/:595`; `Main.cs` tileSolid probes; `HiveBiome.cs:340-374`);
sibling reads via `git show origin/<branch>:<path>` (g37 `passes_b.rs`
place_tile + pass_hellforge + research/75 §2.7; g41 `not_the_bees.rs` +
pipeline; d105 pipeline + structures + 12e; d106 pipeline); the
`TILE_SOLID` bit-77 check computed from the generated table word.
Commits: `74103ae4` (§6.1), `574f1d58` (§6.2), this doc (§6.3/§6.4).
