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
| C1 | `TileIsExposedToAir` | `WorldGen.cs:54782-54800` | `spread::tile_is_exposed_to_air` | Convert 7/15/18/19 (C4), the c32 biome-dungeon rooms | **landed 59fb188b** |
| C2 | `BlockBelowMakesSandConvertIntoHardenedSand` | `:86562-86583` | `spread::block_below_makes_sand_convert_into_hardened_sand` | Convert 5/16 (C4), BiomeDungeonRoom | **landed 8e3eb894** |
| C3 | `Convert` type 2 (hallow cell) | `:55176-55260` | `spread::convert_hallow_cell` + `convert_sized_hallow` | SpreadGrass :75218, DoHallowOnSurface :1967-2011, ConvertSkyIslands :24701, initializeHardMode :31630-31651, the 146 spray / ChlorophyteBolt :77635 | **landed 6c5dcea1** |
| C4 | `Convert` types 5/6/7 (desert/snow/purify spray cells) | `:55363-55408` / `:55409-55445` / `:55446-55526` | `spread::convert_spray_cell` + `convert_sized_spray` (+ the two `:55465` union seams in `growth/grass.rs`) | the aiStyle-31 Clentaminator 1015/1016/1017 (`Projectile.cs:26813-26823`, d4's dispatch) | **landed 0e6bee9e** |
| C5 | `Convert` types 8/9/10 (chlorophyte/jungle-mud cells) | `:55665-55670` / `:55671-55720` / `:55721-55763` | `spread::convert_chlorophyte_cell` + `convert_sized_chlorophyte` | hardUpdateWorld :69658/:69686/:69697 (the ported `convert_cell` approximation stays — unwired), ChlorophyteDefense :69460 (remix) | **landed a6b21b72** |
| C6 | `IsTileATreeBranch` / `IsTileATreeRoot` | `:29717-29740` / `:29742-29765` | `tree::is_tile_a_tree_branch` / `is_tile_a_tree_root` (`Option<i32>` = the C# `out offsetToTrunk`) | C7, ConsideredSolidTileForAnchor :53903 | **landed b53ebbc0** |
| C7 | `ConvertTreeAndGround` (+ `_Branches`) | `:54902-55052` (+ `:55054-55065`) | `tree::convert_tree_and_ground` (explicit `&mut UnifiedRandom` = the caller's genRand) | the 596/616/5/323 arms of TryConvertingOrKillingTreesAbove :54855-54896 (the port's kill-only copies stay — unwired) | **landed f9c3dba7** |
| C8 | `GetTileMossColor` | `:81215-81255` | `framing::get_tile_moss_color` | TileFrame case 184 (:85176-85202, the loose-moss colour pick) | **landed 6e9ae957** |

Claim commit: 057d9ce1. **C1-C8 fully landed — the round is closed.**

## Landing notes / deliberate deviations

- **The wall halves** ride the shared `growth::convert_wall_by_case`
  (types 2/5/6/7 verbatim). Type 7's `:55465` dirt arm there carries only
  the Dirt set — C4 pre-gates the HardenedSand/Snow union half through two
  `pub(crate)` membership seams (`conversion_hardened_sand_walls_contains` /
  `conversion_snow_walls_contains`), order-equivalent: both unions are
  disjoint from the natural Stone/Ice/Sandstone arm, the 262/274 switch and
  the NewWall sets, so the pre-gate cannot reorder a conversion.
- **genRand**: the wall ladders take `None` (the `convert_cleanse_cell` / d1
  precedent — no runtime genRand stream; types 2/5/6/8/9/10 draw nothing
  anyway; type 7's ladder has no roll). C7 takes the explicit caller stream
  (the g39 H-family precedent) and draws in C# order — the base/top/middle
  frame rolls, the `Next(2)`-then-`Next(2)` branch lottery.
- **The C7 trunk-walk quirk kept**: the loop's branch test reads the FIXED
  `y - 1` cell (`:54924`), not `y - 1 - num2` — verbatim.
- **The re-frame** is the port's `reframe_area` superset (the
  `convert_cleanse_cell` documented deviation); the
  `RainbowSandAndBlackSandWalls` paint arms and the `NetMessage` tails are
  the d1-documented skips (a mode the port does not generate / MP-only).
- **The thorn kills** call `edit::kill_tile(world, x, y, false)` — the C#
  `KillTile(i2, j2)` default arm (fail/effectOnly/noItem all false).
- **The type-2 mud arm** (`:55256-55259`) reads the SNAPSHOT type 59 against
  the LIVE 109 4-neighbours, independent of the else-if ladder — kept.
- **Unwired**: no executed step calls any C1-C8 fn; both world-hash pins
  unchanged by this round (the comment-only `world_hash.rs` note), the
  `a == b` determinism halves untouched. 19 witnesses WRITTEN NOT RUN
  (code-only; no cargo/rustc/rustfmt/tests/dotnet/graphify invoked).
  Verified by re-reading the cited C# blocks in ≤100-line windows and hand
  arithmetic against the ladders.

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

## Wiring phase (consumers census + landed seams)

The helper round closed unwired; this phase wires the C1-C8 cells at their
**unclaimed** C# consumer positions (the pushed sibling claims re-read after
`git fetch --prune`). One commit per consumer-family. All seams are
**runtime-only** (no executed worldgen step calls them), so both world-hash
pins stay unchanged; the `a == b` halves stay untouched.

### The consumer census (C# call-sites → port seam → owner)

| helper | C# consumers | port seam wired | owner verdict |
| --- | --- | --- | --- |
| C1 `TileIsExposedToAir` | Convert 7 `:55512`; 15/18/19 `:56031/:56171/:56222`; BiomeDungeonRoom | the C4 type-7 cell's sand arm (transitive) | types 15/18/19 unported; BiomeDungeonRoom = c32's lane |
| C2 `BlockBelow…HardenedSand` | Convert 5 `:55386`; 16 `:56064`; BiomeDungeonRoom `:264/:280` | the C4 type-5 cell's sand arm (transitive) | type 16 unported; BiomeDungeonRoom = c32 |
| C3 `Convert` type 2 | aiStyle-31 146 `:26803`; holy water `:77635`; SpreadGrass `:75218`; DoHallowOnSurface `:1967`; ConvertSkyIslands `:24701`; initializeHardMode `:31630`; GERunner | `ai_spray` + `water_death` + `spread_grass` (tiles-only) | GERunner/initializeHardMode = **d103 P2/P4 (pushed)**; ConvertSkyIslands unported; DoHallowOnSurface keeps its map approximation |
| C4 `Convert` 5/6/7 | aiStyle-31 1015/1016/1017 `:26813-26823` | `ai_spray` per-kind dispatch | d4 marks style 31 "P" (ported), does not touch the file |
| C5 `Convert` 8/9/10 | hardUpdateWorld `:69658/:69686/:69697`; ChlorophyteDefense `:69460` | **none — stays unwired** | **d103 P8 (pushed 014cf2937)** owns the verbatim hardUpdateWorld + its own `convert_purge_cell` |
| C6 `IsTileATreeBranch/Root` | ConvertTreeAndGround `:54924+`; ConsideredSolidTileForAnchor `:53903` | C7 (transitive) | ConsideredSolidTileForAnchor/ValidateOrAttemptReorientingAnchor unported |
| C7 `ConvertTreeAndGround` | TryConvertingOrKillingTreesAbove 596/616/5/323 `:54855-54896` | `try_kill_invalid_tree_above` (the Convert/clentaminator path) | unclaimed (d103 keeps the "documented skip"); growth/trees.rs copy = g17's lane |
| C8 `GetTileMossColor` | TileFrameImportant case 184 `:85176-85202` (the ONLY caller — no Main lighting arm) | `edit/kill.rs::check_long_moss` (the Check* dispatch) | unclaimed in the pushed record (d109/d110 don't carry it; d105 owns PlaceTile-184 growth, not TileFrame) |

### Landed wiring slices

1. **Spray family (C3+C4, transitively C1+C2)** — `ai_spray`'s owner-gated
   `WorldGen.Convert` dispatches type 2 → `convert_sized_hallow`, types 5/6/7
   → `convert_sized_spray` beside the type-0 cleanse; `water_death`'s holy
   arm → `convert_sized_hallow`; `spread_grass`'s 109 arm →
   `convert_hallow_cell_halves(tiles, walls=false)` (the C3 cell gained the
   C# single-cell overload's `tiles`/`walls` flags). The evil arms keep
   `convert_cell_flavor` until d1's type-1/4 cells land.
2. **Tree family (C7, transitively C6)** — `try_kill_invalid_tree_above`
   grows the 596/616/5/323 `ConvertTreeAndGround` arms on the explicit
   `frame_rng` stream (the C# `genRand == Main.rand`, the port's runtime
   stand-in).
3. **Moss family (C8)** — `edit/kill.rs::check_long_moss` ports
   TileFrameImportant case 184 verbatim (the four-neighbour colour pick via
   `get_tile_moss_color`, the `Next(3)*18` frameY roll on `frame_rng`, the
   no-colour KillTile), dispatched from `square_tile_frame`; the 184 boot
   static lands in `boot_statics`.

**C5 stays unwired** — d103 P8 owns the verbatim hardUpdateWorld chlorophyte
arms with its own `convert_purge_cell`; wiring the C5 cells there would
collide. The C5 helpers remain available for that lane / the merger.

## Closeout phase (2026-11-21) — the Convert-family census gaps

The wiring round left four Convert-family consumers unported/unrowed; this
phase closes them (one commit per item, witnesses written not run).

### 1. ConvertSkyIslands — LANDED f0fe0f388

`WorldGen.cs:24674-24714` as `finish_seeds.rs::convert_sky_islands`, wired
at the `:24243` anniversary call (the `!getGood && !drunk` gate inside the
reduced `:24239` guard). g44's islands lane (read at tip 32e478d34) marks
the call "not ported" in its `finish_seeds.rs` module note — the
Convert-family port lands HERE (cross-note in ANALYSIS/PORT_MAP). The C3
`convert_hallow_cell` single-cell overload is the Convert arm; the vanity
596/616 + plant 82/83/185-187/227 kills ride `kill_tile(no_item: true)`
(the gen-time `:63546-63549` flag, the `finish_get_good_world` precedent);
the 1-in-3 `TryGrowingTreeByType(5)` = `GrowTree` roll rides
`growth::grow_tree` on a fresh `UnifiedRandom(opts.seed as i32)` (the
`micro_surface.rs::start_in_hardmode` precedent — the per-pass xorshift
contract cannot feed the `UnifiedRandom` growth seam). Classic worlds
never run the anniversary pass: both pins unchanged.

### 2. The clentaminator SOLUTIONS census — LANDED 5e83d3ecb

The full `ItemID` solution list vs the `ai_spray` dispatch
(`Projectile.cs:26791-26824`):

| item | name | `Item.shoot` | + 145 → proj | Convert type | cell owner |
| --- | --- | --- | --- | --- | --- |
| 780 | Green Solution | 0 (`Item.cs:10517`) | 145 PureSpray | 0 | `convert_cleanse` (landed) |
| 781 | Blue Solution | 1 (`:10526`) | 146 HallowSpray | 2 | C3 (landed, wired) |
| 782 | Purple Solution | 2 (`:10535`) | 147 CorruptSpray | 1 | d1 `convert_sized` (unmerged) |
| 783 | Dark Blue Solution | 3 (`:10544`) | 148 MushroomSpray | 3 | g42 `convert_cell_mushroom` (unmerged) |
| 784 | Red Solution | 4 (`:10553`) | 149 CrimsonSpray | 4 | d1 `convert_sized` (unmerged) |
| 5392 | Sand Solution | 870 (`DefaultToSolution(1015)`, `:42772`+`:47799`) | 1015 SandSpray | 5 | C4 (landed, wired) |
| 5393 | Snow Solution | 871 (`:42775`) | 1016 SnowSpray | 6 | C4 (landed, wired) |
| 5394 | Dirt Solution | 872 (`:42778`) | 1017 DirtSpray | 7 | C4 (landed, wired) |

- The dispatch itself was already verbatim-complete (the wiring round);
  the GAP was the seven missing `proj_data` rows (146-149 line-identical
  `:2350-2397` copies of the 145 arm; 1015-1017 `DefaultToSpray`
  `:9633-9636` → `:10691-10702`) — rowed this phase. All eight item rows
  were already in the registry; both guns (779 speed 7 / 5134 speed 11,
  `shoot 145` + `useAmmo Solution`) resolve through `ammo.rs`'s additive
  arm (`Player.cs:53487-53490`).
- **5495 is NOT a Clentaminator**: it is the **Shimmer Gun**
  (`ItemID.cs:12673`, `Item.cs:43313-43325` — `shoot 1042`
  ShimmerGunStream, aiStyle 60 `Projectile.cs:9848-9857`, NO useAmmo).
  Its `Player.cs:49124-49128` spawn passes `ai1 = 1`, but that is the
  shimmer stream's own flag — no solution arm belongs to it. The 1042
  row is the shimmer lane's (unrowed here, out of scope).
- The 147/148/149 arms stay no-ops in `ai_spray` until d1/g42 merge —
  their cells are the COMPLEMENT this lane documented at claim time.

### 3. The powder items — census, d1 owns the machine (no port here)

The task prompt's "194/195" are NOT powders: `ItemID.cs:2071/:2073` —
194 MushroomGrassSeeds, 195 JungleGrassSeeds. The powders are:

| item | name | `Item.shoot` | proj | Convert call (`Projectile.cs:24191-24202`) |
| --- | --- | --- | --- | --- |
| 66 | Purification Powder (`Item.cs:2168-2184`) | 10 | 10 (`:897-912`) | `Convert(x, y, 11, 1)` (`:24193`) |
| 67 | Vile Powder (`:2185-2198`) | 11 | 11 (`:913-923`) | `Convert(x, y, 1, 1)` (`:24197`) |
| 2886 | Vicious Powder (`:27373-27386`) | 463 | 463 (`:5340-5350`) | `Convert(x, y, 4, 1)` (`:24201`) |

The aiStyle-6 machine (`Projectile.cs:24100-24228`) is **d1's claim,
verified at tip e271b5bc8**: `projectile_ai_powder.rs::step` (660e111fc)
ports the whole arm — the 0.95 decay / 180-tick kill / one-shot dust burst
(the 1051's 50 `Next(3)` draws stay), the owner-gated tile scan calling
`spread::convert_sized(world, tx, ty, 11|1|4, 1)` (78dc52720's
`convert_powder_cell` — the type-1/4/11 ladders), the fertilizer
(1019/1051) sapling growth deferring to the `UpdateWorldGen` pass via
`wants_sapling_grow` → `growth::attempt_grow_tree_from_sapling`, the five
`SetDefaults` rows (7e70c3c91) + the dispatch 6-arm. The three item rows
(66/67/2886) are already registry-rowed on THIS branch; the generic
throw path (`editing.rs`'s `held_use_ammo == 0` fire block) spawns them.

**Are f25's Convert cells called by the powder chain? NO — and correctly
so.** The powders call types 11/1/4 (d1's cells); f25's C3/C4/C5 cells
are types 2/5/6/7/8/9/10 — disjoint. The only shared seams are
`growth::convert_wall_by_case` (d1's types 1/4/11 ride the same flat
ladder) and the `CONV_THORN`/`in_world`/`CONV_JUNGLE_GRASS` constants —
d1's `CONV_THORN` duplicate carries the merger-dedupe note (the C3
precedent). No f25 wiring is missing on the powder chain; the merge
lands d1's machine whole.
