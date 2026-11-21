# 12g (helpers) — W4-f25: the WorldGen helper census, round 2

> Wave 4 · lane `task/W4-f25-worldgen-passes-8` @ 0ed1f3cf (Settle Liquids landed)
> · claimed 2026-11-21. The pass ledger lives in `12g-f25-settle-liquids.md`;
> with the classic-pass census exhausted this lane takes the **missing
> WorldGen.cs helper functions**, the g39 H1-H15 precedent (its `12j` doc).
> Code-only mode: C# read in ≤100-line windows, ONE helper per commit+push,
> witnesses written NOT run, helpers UNWIRED (no executed step calls them —
> both world-hash pins unchanged, `a == b` halves untouched).

## In-flight probe (pushed siblings re-read at claim time)

- **g39** `12j` (bab25558 tip): H1-H15 landed — the Place-wrapper family
  (Place1x1/1x2/2x1/2x2/3x1/3x2/3x3/3x4/5x4/4x2/6x3, wall paintings,
  PlaceOnTable1x1/PlaceLogicTiles, Place1xX/2xX, PlaceMan/Woman, the five
  SolidTile placement-floor variants). Its census also pins:
  **`ConvertGrassToDirt` does not exist in this decompilation (1.4.5.8)** —
  re-verified here (`Select-String WorldGen.cs ConvertGrassToDirt` = 0 hits);
  the grass→dirt conversion rides `Convert`/`Convert_ActuallyConvertTile`.
- **d1** `task/W4-d1-proj-ai-1-10` (660e111f + 78dc5272): the aiStyle-6
  powder arm + **Convert types 1/4/11** (`spread::convert_sized` /
  `convert_powder_cell`, the type-11 wall no-op gates, the torch style set).
  NOT on this branch — the types below are the COMPLEMENT, no overlap.
- **g42** (e2707f2d): **Convert type 3** as `spread::convert_cell_mushroom`
  + DoSurfaceIsMushrooms — claimed there, excluded here.
- **g37/d103/d106** (df468d00 / 463cab95 / ee0fcbef): `Spread.Wall` /
  `Spread.Wall2` as pass-local `spread_wall`/`spread_wall2` copies + the
  generation-time `SpreadGrass(0, 2)` slices — claimed, excluded.
- **g41**: `SpiderWallRemover` (:77279) + the egg-sac/stalactite arms —
  claimed, excluded. **g38/d104**: `PlaceTile(488)` slice — excluded.
- **c32**: dungeon style TABLES only (3d0f3c19) — the biome-dungeon room
  bodies (`BiomeDungeonRoom.cs` etc., the `TileIsExposedToAir` /
  `BlockBelowMakesSandConvertIntoHardenedSand` consumers) are NOT on it.
- **d4** (proj-ai-31-40): no pushed Convert work — the aiStyle-31
  Clentaminator's spray types 5/6/7 (`Projectile.cs:26815-26823`) have no
  ported cell arm; this lane ports them UNWIRED for that consumer.

## The helper census (C# `WorldGen.cs` vs `crates/` on this branch)

Already ported (NOT this lane): `Convert` type 0 (`spread::convert_cleanse`,
the S1-D-4 slice), types 1/4/11 on d1, type 3 on g42; the spread runtime
(`hard_update_world` :69645-69860, `convert_cell`/`convert_cell_flavor`);
`SpreadGrass` runtime slice (`growth::spread_grass` :75146-75257 repeat:false)
+ the gen-time copies (`evil.rs::spread_evil_grass`,
`biomes.rs::spread_jungle_grass`); `SpreadGrassWalls`/`SpreadDesertWalls`
(:74884/:74984, `growth/grass.rs`); `Spread.Moss`/`Gem`/`Spider`
(:3526/:3594/:3682, `spider_gem_moss.rs`); `MossConversion` (:71457,
`growth/grass.rs`) + `randMoss`/`setMoss`/`neonMossBiome`; `PlaceTile`'s
torch/platform/planter/alch arms (`edit/place.rs`), `KillTile` + the
`KillTile_GetItemDrops` per-cell records + `KillTile_GetTreeDrops` +
`KillTile_PlaySounds` (`edit/kill.rs` — the `no_item` flag IS the C#
noItem gate, no separate variant exists in the C#); the g39 H-family.

### Missing — this lane's claim (verbatim ports, unwired)

| # | C# helper | anchor | port seam | consumers | commit |
| --- | --- | --- | --- | --- | --- |
| C1 | `TileIsExposedToAir` | `WorldGen.cs:54782-54800` | `spread::tile_is_exposed_to_air` | Convert 7/15/18/19 (C4/C5), the c32 biome-dungeon rooms | (pending) |
| C2 | `BlockBelowMakesSandConvertIntoHardenedSand` | `:86562-86583` | `spread::block_below_makes_sand_convert_into_hardened_sand` | Convert 5/16 (C4), BiomeDungeonRoom | (pending) |
| C3 | `Convert` type 2 (hallow cell) | `:55176-55260` | `spread::convert_hallow_cell` + `convert_sized_hallow` | SpreadGrass :75218, DoHallowOnSurface :1967-2011, ConvertSkyIslands :24701, initializeHardMode :31630-31651, the 146 spray / ChlorophyteBolt :77635 | (pending) |
| C4 | `Convert` types 5/6/7 (desert/snow/purify spray cells) | `:55363-55408` / `:55409-55445` / `:55446-55526` | `spread::convert_spray_cell` + `convert_sized_spray` | the aiStyle-31 Clentaminator 1015/1016/1017 (`Projectile.cs:26813-26823`, d4's dispatch) | (pending) |
| C5 | `Convert` types 8/9/10 (chlorophyte/jungle-mud cells) | `:55665-55670` / `:55671-55720` / `:55721-55763` | `spread::convert_chlorophyte_cell` | hardUpdateWorld :69658/:69686/:69697 (the ported `convert_cell` approximation stays — unwired), ChlorophyteDefense :69460 (remix) | (pending) |
| C6 | `IsTileATreeBranch` / `IsTileATreeRoot` | `:29717-29740` / `:29742-29765` | `tree::is_tile_a_tree_branch` / `is_tile_a_tree_root` | C7, ConsideredSolidTileForAnchor :53903 | (pending) |
| C7 | `ConvertTreeAndGround` (+ `_Branches`) | `:54902-55052` (+ `:55054-55065`) | `tree::convert_tree_and_ground` | the 596/616/5/323 arms of TryConvertingOrKillingTreesAbove :54855-54896 (the port's kill-only copies stay — unwired) | (pending) |
| C8 | `GetTileMossColor` | `:81215-81255` | `framing::get_tile_moss_color` | TileFrame case 184 (:85176-85202, the loose-moss colour pick) | (pending) |

Not claimed (owner lanes / no worldgen consumer): `Convert` types 12-21
(the paint-spray/evil-II family — zero call sites in the whole decompile
outside the switch itself; the 21 dungeon arm reads
`GenVars.CurrentDungeonGenVars`, c32/d107's seam); `Spread.Wall`/`Wall2`
(g37/d103/d106 own copies); `SpiderWallRemover` (g41);
`OverGrownDungeonWithRainbowMoss` (:24037, the celebrationmk10 arm — d107/c30
special-seed lane); `KillTile_ShouldDropSeeds`/`KillTile_DropBait`
(:66176/:66263 — player-state runtime halves, the kill lane's phase-3 drop
resolution owns them); `PlaceTile` proper (:59342, the player-facing
dispatcher — g37/g38/d104 carry per-kind slices, `edit/place.rs` the arms).

## Discipline

- Verbatim ports (golden rule): same structure, maths, draw order; citation
  comment per block; readable Rust names; C# read in ≤100-line windows.
- The Convert wall halves route through the shared
  `growth::convert_wall_by_case` where its arm exists (types 2/5/6/7/12 —
  the M5-SPREAD-2c chains); the genRand rolls take `None` (the
  `convert_cleanse_cell` / d1 precedent — no runtime genRand stream).
- Helpers are UNWIRED until their consumer lands — **both world-hash pins
  stay unchanged** (a comment-only note records that in `world_hash.rs`);
  the `a == b` determinism halves stay untouched.
- ONE helper per commit+push; this ledger + an `anchors.txt` row per slice.
- Code-only lane: no cargo/tests/graphify on this branch; witnesses written
  NOT run, per the S1-K-2-wave-B / d10x convention.
