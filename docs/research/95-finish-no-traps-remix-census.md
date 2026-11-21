# 95 — FinishNoTraps + FinishRemixWorld census (W4-g41 finish gaps)

Code-only census of the two remaining classic-seed `Finish*` hosts against
`decompiled/Terraria/WorldGen.cs`, taken to close W4-g41's recorded finish
gaps after the NOT-THE-BEES host landed (b5c8e694). Co-lane claim check
FIRST, per the brief.

## Co-lane claim check — d107 owns NEITHER host

`git fetch origin` + the pushed `origin/task/W4-d107-worldgen-special-seeds`
tree (tip 26672912): its ANALYSIS "The worldgen arms" / "Deviations +
deferred" section says VERBATIM:

> The remix inverted-terrain family (~211 `remixWorldGen` sites: the
> inverted bands of `TerrainPass`, the doubled cave counts, the NPC swaps,
> **`FinishRemixWorld`'s cloud/wall conversions) is deferred** — it is a
> shape-level rewrite of the compact terrain model, not a branch add. The
> row's remix coverage is the flags + the two `Reset`-range arms + the
> spawn override.

d107's pushed code touches `caves.rs`/`mod.rs`/`pipeline.rs`/`world_hash.rs`
only (SurfaceCaves seed gates + OresAndShinies secret-seed branches); its
`finish_seeds.rs` is byte-identical to this branch's base — no
`FinishRemixWorld` body exists there. The remix host is UNCLAIMED; this task
takes the `FinishRemixWorld` body itself (the ~211-site inverted-terrain
family stays deferred with d107's note — this port is the Finish tail only).

d107's no-traps coverage is the MICRO-seed side only: the
`actuallyNoTrapsForRealIMeanIt` spike sweeps (`WorldGen.cs:2632-2639` +
`Variations`) already live in this branch's `micro_finish.rs`
(`actually_no_traps`, M5-SEED-2) and the `ExplosiveTrapCount` ×1.5
`(getGoodWorld || noTraps) && !notTheBees` multiplier (S1-K-2 wave B,
`mini_biomes.rs`). The CLASSIC `noTrapsWorldGen` seed's own tail
`FinishNoTraps` (`:25986-26224`, dispatched `:22423-22426`) appears NOWHERE
in d107's pushed claims or code. UNCLAIMED; taken.

## The C# machinery (read verbatim, ≤100-line windows)

| C# | Lines | Role |
| --- | --- | --- |
| Finish dispatch | `:22423-22447` | **`FinishNoTraps`** (`:22423-22426`, FIRST — before anniversary) → `FinishTenthAnniversaryWorld` → `FinishDrunkGen` → `NotTheBees`+`FinishNotTheBees` → `FinishGetGoodWorld` → **`FinishRemixWorld`** (`:22444-22447`, LAST of the classic tails) |
| `FinishNoTraps()` | `:25986-26224` | `SetBoulderSolidity(false)` (`:25988`, body `:11009-11020`: `tileSolid[138/484/664/711-716]` — 665 NOT in the list); `maxValue = 5`, FTW-&&!anniversary → 2 (`:25989-25993`); the 50-border column-major sweep (`:25994-26184`): the per-cell `Next(maxValue)==0` heart-12 frame-0,0 → 2×2 life-crystal-boulder 665 (`:25999-26005`, draw FIRST, `&&` short-circuit), the even-cell statue/plate-chest gate (`:26006`: `i%2==0 && j%2==0 && active && (type==105 || (type==467 && (frameX==144 || frameX==162)))`), the 3×3 red-wire veto (`:26010-26020`), the 625-draw ±25 wire search + L-shaped red-wire run (`:26023-26066`), the statue-only 225-attempt ±15 pressure-plate-135 placement (`PlaceTile(…, forced, style Next(2,4))` — the style draw runs EVERY attempt; `:26067-26112`) + its own L-run, the 467-only (`!tenth || remix`) 64-attempt ±8 explosives-141 placement on `{0,1,Ore,59,151}` cells with a fully-solid 3×3 (`:26114-26178`, `TileFrame(resetFrame:true)` + the kind-141 recheck) + L-run + `break`, and the **`else { j++; }`** row-skip (`:26180-26183` — fires for statue cells AND for 467 cells under `tenth && !remix`); `SetBoulderSolidity(true)` (`:26186`); the 20-border unwired-plate-chest sweep (`:26187-26203`: 467 frameX 144 frameY 0 with all four 2×2 cells wire-free → 2×2 chest 21 frameX 36/54); the 8000-chest 1-in-15 item-5346 add-on (`:26204-26223`) |
| `FinishRemixWorld()` | `:26350-26770` | the 25-border cloud/sunplate sweep (`:26352-26432`: `Clouds[type] || type==202` records the deepest row `num` and — under `!tenth || ftw` — converts to flesh 195 / lesion 474 by drunk-`crimsonLeft` half-split `Next(-2,3)` / crimson / corruption; the cloud wall 73/82 arm converts to flesh wall 77 / lesion wall 233 the same way, skipped under `tenth && !ftw`); the 25-border conversion sweep (`:26433-26585`: the `l > num || errorWorld || surfaceIsInSpace` inland band above `worldSurface` between the `beachDistance` (380, `:86914`) margins — wall-87 liquid → water, else FTW lava + drip 375/373/709 → 374 + honey/shimmer/obsidian kills, NTB honey + 374/373/709 → 375, drunk+anniversary shimmer + 374/373/375 → 709, each with the 230/229/659/56 kills + the 82/83/84 `TileFrame`; the per-cell `Next(3)` surface band — anniversary-&&!FTW `Convert(2)` + the 661/662 → 109 replacement with `PlaceTile(110)` + the 7×7 mud-59 → 0 kill, drunk half-split `Convert(1/4)`, else `Convert(crimson?4:1)`; the per-cell `Next(19,22)` marble-178/granite-180 wall clear above the band; the obsidian-56 kill; the below-rock-layer active-dirt-0 → 2 fill with the 8-neighbour veto); `Liquid.QuickWater(-2)` (`:26586`); the spawn-box liquid clear loop (`:26587-26632`: `randomSpawn` micro-seed veto, the 0.38-0.62 × `maxTilesY-200..-125` box, hellstone 58 → ash 57, 230/56/659 deactivates, `QuickWater(-2)` + the spawn-column ±10/15 liquid recheck); `AddTrees(undergroundOnly: true)` (`:26633`, body `:31365-31447` — the two head `NextDouble` draws run even under `undergroundOnly`; the remix underground loop `worldSurface..maxTilesY-300` grows vanity 616/596 on grass-2 at 1/7 (anniversary: always) else `TryGrowingTreeByType(5)`); the `!tenth || ftw` dungeon repaint (`:26634-26717`: per-column colour 22, drunk half-split / crimson → 13; dungeon bricks + cracked → colour, dungeon walls → wallColour 25 + platform-19 non-frameY-180 → colour, anniversary living-wood-191 (`!noSurface`) / 226 / 61-on-226 / trap-137 frames 1-4 → colour2, sandstone-151 → colour, `!noSurface` wall-244 / wall-34 / wall-87 → wallColour 25 (+87 tile colour2)); the ash-plants restyle (`:26718-26733`: `maxTilesX/4200.0 * Next(2,5)` clusters, x `Next(0.39w, 0.61w)` ± `Next(10,31)`, rows `maxTilesY-250..-25`, kind 637 → frameX `18 * Main.rand.Next(6,11)` — `Main.rand == genRand`, `:4445`); the NTB hive-wall strip (`:26734-26758`: per-column above-surface wall-86 clear until the first active tile, the 5-down/2-side wall+active re-arm); `QuickWater(-2)` (`:26759`); the full-map lily-pad-518 `CheckLilyPad` sweep (`:26760-26769`, body `:59203-59309`) |
| `SetBoulderSolidity` | `:11009-11020` | the nine-kind `tileSolid` window toggle (665 excluded) |
| `Place1x1` | `:45188-45235` | the plate write: `InWorld(x,y,2)`, the mechanism-on-boulder veto (`IsAMechanism[135] && Boulders[floor]`), `SolidTile2` floor + empty cell, `frameY = 18*style` (default branch) |
| `PlaceTile` outer | `:59342-59476` | `forced` bypasses the occupancy gate only; the `!active` → `Clear(Tile|TilePaint|Slope)` pre-write; 135 is neither torch nor plant-excluded so liquid does not veto |
| `IsFitToPlaceFlowerIn` | `:58231-58251` | the 110 floor test: active, slope 0, !halfBrick, floor kind ∈ {109, 492} |
| `PlaceTile(110)` flower arm | `:59507-59580` | the wall-gated plant schedule: `AllowsPlantsToGrow` on BOTH walls → `Next(50)==0` → frameX 144; else `Next(35)==0 || wall ∈ 63..=70` → `NextFromList(6,7,9,10,…,20)` ×18; else `Next(6)*18` |
| `Convert` single-cell | `:55085-55260` | types 1/4/2 — the port's `spread::convert_cell_flavor` seam. **Case 2 has NO `Conversion.JungleGrass` arm** (`:55215-55260`): 661/662 survive `Convert(2)` — the port's shared hallow map is GERunner's superset (661/662 → 60), so the remix host restores the jungle grass after the shared call to keep the `:26524` replacement arm reading the C# state (documented reconciliation) |
| `AddTrees` | `:31365-31447` | the two head `NextDouble` draws + the remix-only underground loop |
| `CheckLilyPad` | `:59203-59309` | non-water → kill; the down-scan to the first solid non-top floor (`maxTilesY-50` cap); grass 2/477 → frameY 0, hallow 109/116 → 18, jungle 60 → 36 (the C#'s duplicated `type==109` clause is dead); the pad moves UP into an inactive liquid cell above, else DOWN into an inactive floor-below cell (liquid-0 gate), else a non-platform non-solid floor kills it; unknown floor → kill |
| `beachDistance` | `:86914` | 380, constant |
| `TileID.Sets.Boulders` | `TileID.cs:504` | {138, 484, 664, 665, 711-716} — the plate floor veto set (665 INCLUDED, unlike the solidity window) |

## Rust seam census (what exists, what this task adds)

| Need | Port state |
| --- | --- |
| Finish dispatch slot | `pipeline.rs::step_seed_finish` — drunk → NTB → FTW → anniversary → micro today. This task adds `finish_no_traps` FIRST (the C# `:22423` slot) and `finish_remix_world` right after FTW (the C# `:22444` slot). NOTE: the port's anniversary tail sits AFTER FTW (M5-SEED-1 ordering, C# has it BEFORE drunk) — pre-existing, left untouched (each tail draws its own stream; the reorder is not this task's scope) |
| `pass_rng` convention | `Streams::pass_rng(name)` = `Rng(seed ^ fnv1a64(name))`; the Finish family uses the C# method name — this task adds `"FinishNoTraps"` + `"FinishRemixWorld"` |
| red wires | `World.wires` (`wiring::WireState`, sparse `HashSet`s) — `get`/`set` with `WireColor::Red`; the L-run shape is the `mini_biomes.rs::wire_line` precedent (horizontal at the START row, vertical at the END column) |
| `SolidTile` window | the boot `TILE_SOLID` table is immutable — the boulder window is a local solid helper vetoing the nine `SetBoulderSolidity` kinds (the `not_the_bees.rs` 379-window precedent); nothing is restored after |
| `SolidTile2` floor | `edit::solid_tile2` (the `mini_biomes.rs::place_explosives_tile` precedent) |
| `KillTile` | `edit::kill_tile(world, x, y, true)` |
| `Convert` | `spread::convert_cell_flavor(world, x, y, flavor, walls=true)` — the M5-SPREAD-2c seam; the case-2 jungle-grass reconciliation above |
| tree growth | `growth::{grow_tree, try_growing_tree_by_type}` take `UnifiedRandom` — the host feeds ONE `UnifiedRandom` seeded from its own pass stream (the NTB tree-bridge precedent, a documented deviation) |
| `TileFrame`/`SquareTileFrame` | the post-pipeline `frame_world` seam (the M5-STRUCT-5 idiom) — no in-pass framing; the explosives kind-141 recheck reads the written kind |
| tile sets | `td::{SET_CLOUDS, SET_CRACKED_BRICKS, SET_ORE, SET_BOULDERS, TILE_SOLID, TILE_SOLID_TOP, TILE_LAVA_DEATH}`; `WallID.Sets.AllowsPlantsToGrow` hand-transcribed (the `trees.rs` precedent) |
| `MicroSeedFlags` | `ERROR_WORLD` / `SURFACE_IS_IN_SPACE` / `RANDOM_SPAWN` / `NO_SURFACE` all exist (`seeds.rs:575-603`) — the remix gates read them directly |
| `Liquid.QuickWater(-2)` | MISSING — the port has no worldgen liquid settle (d107's `extraLiquid` deferral precedent). Documented no-op: the spawn-box loop's re-flood recheck reads the cleared box and exits after one iteration |
| chest inventories | the port's chests carry no item stacks — the `FinishNoTraps` 8000-chest 5346 loop iterates nothing: no RNG draws, no writes (the `finish_seeds.rs` module-note precedent) |

## Deviations pinned by this task (all in the module notes + ANALYSIS)

1. `QuickWater(-2)` ×3 in `FinishRemixWorld` — no-op (no settle engine on
   this branch). The g47d ACTIVATION contract is pinned in the addendum
   below (the `-2` is the verbose param; the call is
   `quick_water(world, -1, -1)`).
2. The `UnifiedRandom` tree bridge (one `next_u32` draw off the host stream).
3. In-pass `TileFrame`/`SquareTileFrame` ride the post-pipeline framer; the
   explosives recheck reads the written kind (the C# could pop an
   unsupported 141 inside `TileFrame`; the port pops it at the framer).
4. The case-2 `Convert` jungle-grass restore (the shared hallow map is a
   superset; the host keeps the C# state for the 661/662 → 109 arm).
5. The chest 5346 add-on — inventory-free chests, no draws.
6. The boulder solidity window as a local helper (immutable boot table).

Both hosts are flag-gated (`NO_TRAPS` / `REMIX`): a classic world enters
neither, draws nothing — BOTH world-hash pins UNCHANGED (the W4-g41 egg-sac
re-pin deferral stands as-is). Witnesses written NOT run (code-only).

## QuickWater activation addendum (W4-g41 FINAL — the g47d wire contract)

Read against `origin/task/W4-g47-liquid-skip-settle` tip **fdeb897c** (the
FINAL merge-order doc `docs/research/70`) + its code base **d222af69**.
The three `QuickWater(-2)` calls in `finish_remix.rs` (`:26586`/`:26622`/
`:26759`) are no-ops on THIS branch; they ACTIVATE when the g47d settle
drivers land. The exact contract:

1. **The call shape.** C# `Liquid.QuickWater(int verbose = 0, int minY = -1,
   int maxY = -1)` (`Liquid.cs:104`) — the `-2` is the **verbose** progress
   parameter, NOT a band. The activation call at all three seams is
   `sys.quick_water(world, -1, -1)` — g47's signature
   `pub fn quick_water(&mut self, world: &mut World, min_y: i32, max_y: i32)`
   (`liquid.rs:1581` at d222af69); the `-1`s take the method's own defaults
   (`min_y = 3`, `max_y = height - 3`, `:116-123`). No port consumer for
   verbose (g47's doc note) — nothing else to pass.
2. **The context arming (ONCE, at the gen settle driver head — NOT per
   call).** g47's setters (`liquid.rs` at d222af69):
   - `pub fn set_gen_settle_context(&mut self, water_line: i32, lava_line:
     i32, remix: bool)` (`:1536`) — arm with the port GenVars'
     `water_line`/`lava_line` (`pipeline.rs:288-290`, the TerrainPass
     products) and `remix = true` for this host (`Liquid.cs:182`'s remix
     band; a shared driver arms per-world flags).
   - `pub fn set_gen_shimmer_position(&mut self, pos: Option<(i32, i32)>)`
     (`:1549`) — g36-A1's `gv.shimmer_position` slot (the `WorldGen.cs:15241`
     Shimmer-step write); the ShimmerRemoveWater arms (`Liquid.cs:108`/
     `:138`) run inside `quick_water` pre/post sweep.
   - `pub fn set_gen_dungeon_context(&mut self, ctx:
     Option<GenDungeonContext>)` (`:1560`) — the f24/c30 producer
     (research/70 step 7), ORIGIN-wall read (`Liquid.cs:152`).
   - The noTraps `SetBoulderSolidity` swap (`Liquid.cs:109-112`/`:139-142`)
     is subsumed by g47's pass-scoped `tilesIgnoreWater(true)` (same nine
     boulder kinds, g47's `BOULDER_SOLIDS`); `LiquidInteractionsCleanup`
     (`:144-147`) rides the gen settle driver (research/70 step 6 keeps it
     in f25's `settle_liquids.rs` module).
3. **Which LiquidSystem instance.** research/70 step 6 RECOMMENDS unifying
   f25's gen-local `quick_water` onto `LiquidSystem::quick_water`; if the
   g46c graft keeps the gen-local copies instead, this host calls the
   gen-local fn with the SAME `(world, -1, -1)` shape and the identical
   retype semantics (the origin-wall fix applies either way). The three
   seam comments in `finish_remix.rs` name the call site, not the owner.
4. **The spawn-box loop regains its `while flag` shape** (`:26587-26632`):
   after seam 2's settle, the recheck loop (`for num7 = -10; num7 < 15` →
   `Main.tile[spawnTileX, spawnTileY - num7].liquid > 0`, i.e. the spawn
   column rows `spawn_y - 14 ..= spawn_y + 10`) re-arms `flag` and the box
   clear + settle repeats. The port's `World.spawn_x`/`spawn_y`
   (`world.rs:76-81`) are the C# `Main.spawnTileX/Y` stand-ins.
5. **Hash impact at activation.** REMIX worlds change BY DESIGN (the
   settles move liquid + the retype arms fire); the classic pins stay
   UNCHANGED (flag gate). The activation commit re-derives the REMIX
   a==b witnesses' pinned values — it must NOT touch the classic
   `SEED42_512`/`DEFAULT_WORLD` pins beyond the egg-sac deferral that
   already stands. research/70's witness re-pin table (f25 gen-hash
   RE-DERIVE) governs the driver side.

**RETIRE-PHASE RECEIPT (research/96 §1).** Re-verified 2026-11-21 against
g47 tip `5c09a68b8` (closeouts 1-4 touch only research/70 + three
`liquid_sim` witnesses — the API is unchanged) and the g46 RE-FOLD union
tip `9bc722e5f` (the g46d re-base is DONE: `e7626b72d`/`f588d53c5`/
`16fd6ad78`/`d95911ce1` carried every §8 surface). Items 1-2 hold
verbatim on both (`quick_water` g47 `liquid.rs:1742` / union `:1882`;
the setters `:1582`/`:1597`/`:1606` / `:1722`/`:1737`/`:1746`); item 3
resolves at the f25 dedupe (research/70 §9 item 2 — DELETE the gen-local
copies, call the LiquidSystem methods). The activation now targets the
UNION + the f25 gen-driver graft, not a separate "g47d" landing.
