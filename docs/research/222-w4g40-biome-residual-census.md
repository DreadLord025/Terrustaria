# 222 — Biome-residual census (W4-g40 continuation, 2026-11-21, code-only)

RENUMBERED per the d54 research/150 collision registry §1 key-94 row:
`94` belongs to W4-g41's `94-not-the-bees-host-census.md` (first-add
2026-11-21T03:24:48 `4317e2ba5`); this doc (first-add 04:15:49) takes the
registry-assigned free number **222**. Content-preserving rename — the
body below is unchanged; refs updated on this branch (INDEX row, ANALYSIS
citation, desert_entrances.rs/world_hash.rs comments). The lane FINAL +
desert/marble-family declaration lives in `248-w4g40-desert-marble-final.md` (renumbered 246-248 at the batch-B merge).

Scope: after the marble/granite residual closed (79361f00), census the
remaining C# biome classes against the landed port, mark every co-lane
claim, and port the FIRST unclaimed residual verbatim. Base a725acb0
(origin/main); claims read from pushed + local branch tips at 03:54
(2×90 s mtime probes clean — no live writer in this worktree).

## Gap table — C# biome units vs landed

| C# unit (anchor) | Landed port | Residual | Claim / verdict |
| --- | --- | --- | --- |
| `MarbleBiome.cs` / `GraniteBiome.cs` | `biomes.rs` (M5-BIOME-2 + W4-g40 P0-P4) | none — closed 79361f00 | **g40 (this lane), DONE** |
| `DesertBiome.cs:13-46` + `Desert/DesertDescription/SandMound/SurfaceMap/DesertHive` | `desert.rs` / `desert_dunes.rs` (M5-DESERT-structures) | (a) entrance gate `!tenthAnniversary && NextDouble()<=ChanceOfEntrance && !extraLiquid` (`DesertBiome.cs:23`) — port draws unconditionally at the JSON 0.5, no anniversary/extraLiquid gates, no config read; (b) `structures.AddStructure(area,10)` (`:43-44`) — NON-protected list, no ported `CanPlace` reads it (`StructureMap.cs:19-52` reads `_protectedStructures` only; the thin-ice precedent) — documented no-op; (c) `DesertHive` `PostPlacementEffect.Smooth` (`DesertHive.cs:197,216-218,284,315,331,358`) — the carve's per-cell `Tile.SmoothSlope(x,y,false)` is skipped (desert.rs header deviation); (d) remix sand-drip arm (`WorldGen.cs:12525-12541`) — deferred with the seed | (a) **UNCLAIMED → this task P2**; (c) **UNCLAIMED → next slice** (the `tile_smooth_slope_one` seam now exists, g40 P1); (b)/(d) documented |
| `Desert/AnthillEntrance.cs` / `LarvaHoleEntrance.cs` / `ChambersEntrance.cs` / `PitEntrance.cs` (the antlion nests / larva holes) | `desert_entrances.rs` (M5-DESERT-structures, shape-level) | **the residual this task ports (P1)** — full list below | **UNCLAIMED** (no branch touches `desert_entrances.rs` since ab6c11f7; g44's census is FloatingIslands/IslandHouse-only) |
| `FloatingIslands` `WorldGen.cs:12927-13122` + `CloudIsland`/`CloudLake` + `IslandHouse` `:79203-79484` | `islands.rs` (M1.4-WGEN-4 compact stand-in) | 13 residuals censused + verbatim port in flight | **g44 co-lane** (local d32816ce census + phase-2 tip; READ, not touched) |
| `Generate Ice Biome` `WorldGen.cs:12312-12422` | `ice_biome.rs` on d101 (e28463f6, pushed; lane complete 9/9 incl. the Granite row split 4194acf8) | none in the pass itself; the Reset snow-origin draws `:11401-11414` landed with it; `GenVars.snowTop/Bottom/MinX/MaxX` published for d102 Slush (d4fd0af0) | **d101 co-lane, READ** — the d107 Ice-name fix (8dce1711: `GenPassNameID.IceBiome` = "Generate Ice Biome", `GenPassNameID.cs:239`, NOT "Ice" = FragileIce `:280`) is consistent on both branches. `ThinIceBiome`'s dead snow-dominance gate (S1-K-2 wave C) unblocks when d101 merges — no action here |
| `FragileIceOverIceBiomeWater` `:16667` / `ExposedGemsInIceBiome` `:20697` | `biome_detail.rs` (M1.4-WGEN-7) | none found | landed |
| Corruption/Crimson **worldgen** spread (`CorruptionAndCrimson` `:13866-14554` + ChasmRunner/Crim* family) | `evil.rs` (M5-BIOME faithful + M5-SEED-1 drunk half) | remix/tenthAnniversary/dontStarve/notTheBees/noSurface/surfaceIsInSpace/skyblock arms — documented deviations | landed; seed arms = d107 lane |
| Corruption/Crimson **runtime** spread (`Convert` `:55085-55673`, `hardUpdateWorld` `:69645`) | `spread.rs` + `growth/grass.rs` (M5-SPREAD-2, S1-K-3b) | thorn bushes + brick-wall tables (documented follow-ups); the type-3 mushroom Convert cell landed g42 (e2707f2d) | landed; follow-ups unclaimed but not biome classes |
| `ConvertGrassToDirt` (Lead's census name) | — | **does not exist in the 1.4.5.8 decompilation** — 0 hits over the whole `decompiled/Terraria` tree (`findstr /s /m`). The Lead's name is stale; the real units are `Convert` (:55085) and `SpreadGrass` (:75146), both landed | census correction, no port |
| `Spreading Grass` worldgen pass `:18237-18446` | compact `micro_grass.rs` stand-in at base | verbatim port in flight | **g37 B8 (df468d00) + d104 (3a101031) co-lanes, pushed** — not touched |
| `Grass Wall` pass `:19589` + `Spread.Wall2` | — | verbatim port landed | **d106 R1 (ee0fcbef), pushed** |
| c29 dead-man traps (`DeadMansChestBiome.cs` + Micro Biomes block `:20916-20938`) | — | ported | **c29 (dbc93e9b + d95c32cf), pushed** — the "dead-man traps?" question resolves: the biome landed on c29; `MiningExplosivesBiome` landed S1-K-2 wave B |
| `JunglePass.cs:19-257` | `biomes.rs::pass_jungle` (faithful, M5-BIOME; the test transcribes the whole class) | none in the pass | landed |
| Jungle temple (`LihzahrdTemple`/Part2/Altar) | `temple.rs` (faithful placement hook) | paint + residuals | **g43 co-lane** (branch at base, no commits yet — claim stands) + d108 temple-paint |
| Jungle detail family (`WetJungle`, plants, `MudsWallsInJungle`, `GrowUndergroundTree`) | `biome_detail.rs` (M5-BIOME faithful) | hallowed/crimson vines + the mushroom half (documented follow-ups) | landed; follow-ups unclaimed, not biome classes |
| `HiveBiome`/`HoneyPatchBiome` | `structures.rs`/`mini_biomes.rs` (M5-BIOME-2) | none | landed |
| `CorruptionPitBiome.cs` | — | audited DEAD CODE (PORT_MAP row 92) | no port needed |
| `CampsiteBiome`/`CaveHouseBiome`/`SpikePitBiome`/`EnchantedSwordBiome` | `mini_biomes.rs` (M1.4-WGEN-8) | `CaveHouse/HouseBuilder` variants (Ice/Jungle/Granite/Marble/Mushroom/Desert/Wood) — the chest-chance variants resolve to the plain gold chest | follow-up, unclaimed (not this slice) |
| `ThinIceBiome`/`MahoganyTreeBiome`/`MiningExplosivesBiome` | `mini_biomes.rs` (S1-K-2 waves A/B/C) | thin-ice gate dead until d101 merges (above) | landed |
| `DunesBiome` | `desert_dunes.rs` (M5-DESERT-structures) | special-seed arms deferred | landed |
| `TerrainPass`/`DitherSnake`/`DungeonControlLine` | `terrain.rs`/`dungeon.rs` + d10x lanes | dual-dungeons dither = d101 lane | co-lane |

**First unclaimed residual in C# call order:** the `DesertBiome.Place`
entrance family (`DesertBiome.cs:25-40` runs BEFORE the hive `:41` and
before FloatingIslands `:12924`). Ported verbatim by this task.

## The entrance residual — line-by-line (P1 scope)

The landed `desert_entrances.rs` (97a7db29) inlined the shapes but
skipped the stochastic modifiers and approximated three chains. The
re-read of all four entrance classes + the framework (`Shapes.cs`,
`Modifiers.cs`, `Actions.cs`, `ModShapes.cs`, `ShapeData.cs`,
`GenAction.cs`, `Utils.cs:2238-2379`, `Tile.cs`) found:

1. **`Shapes.Circle` is the wrong shape AND the wrong order**
   (`Shapes.cs:33-49`): C# is **y-major**, rows `oy-vr..=oy+vr`, each
   row half-width `min(hr, (int)sqrt((hr+1)² - (hr/vr·dy)²))` — the
   `(hr+1)²` inflation admits cells the port's normalised
   `(dx/hr)²+(dy/vr)² <= 1` ellipse rejects, and the port walked
   x-major. With `Blotches` now drawing per cell, the visit order is
   load-bearing for the RNG ledger.
2. **`Modifiers.Blotches` skipped everywhere** (`Modifiers.cs:126-146`):
   two `NextDouble` draws per cell (the first discarded), then on
   `< 0.3` four more draws (`Next(-1,1)`, `Next(0,2)` ×2 axes) spread
   the chain over a box. Sites: anthill `:26` (below-surface rect),
   larva `:27`, chambers mouth `:29`, chambers rooms `:44`, tunnels
   `:49`. The shared `mini_biomes::blotched_gen` (S1-K-2 wave C,
   already faithful) is exposed `pub(crate)` and reused.
3. **`Actions.Smooth` skipped** (`Actions.cs:608-622` →
   `Tile.SmoothSlope`, `Tile.cs:792-865`): anthill `:49` smooths every
   recorded tail cell; larva `:26` runs `Offset(0,1)+Expand(1)+IsSolid
   +Smooth(applyToNeighbors:true)` over the recorded clear-shape. The
   g40-P1 `biomes::tile_smooth_slope`/`_one` seams are exposed
   `pub(crate)` and reused (the five-arm mask map + the flag2 no-write
   veto already landed).
4. **`Actions.SetTile` ≠ `Tile.ResetToType`** (`Actions.cs:203-236`):
   SetTile runs `Clear(~(Wiring|Actuator))` (`Tile.cs:744-790`) —
   wall, wall-frame, liquid, slope, paint and frames all CLEAR (only
   wiring/actuator/inActive survive) — then sets kind+active. The
   port used `reset_to_type` (wall SURVIVES). Every `SetTile(53/397)`
   site changes: the tail sand, the shaft rings, the chamber outline.
5. **`Actions.ClearTile` ≠ kind=0** (`Actions.cs:155-169` →
   `WorldUtils.ClearTile` → `Tile.ClearTile`, `Tile.cs:123-128`):
   slope/half-brick cleared, active+inActive off, **kind/wall/liquid
   survive**. The anthill top blowout `:48` used kind=0+active-off;
   the g40-P0 `mini_biomes::clear_tile_keep` seam is reused.
6. **`Modifiers.IsSolid` ≠ `SolidTile`** (`Modifiers.cs:561-571`):
   `active && SolidOrSlopedTile` (`WorldGen.cs:69985-70008`) — slopes
   and half-bricks PASS (no slope==0/half-brick veto), `!inActive`.
   The port's flat-only `solid()` rejected them.
7. **Anthill: the per-row 1×1 rect is missing above AND below the
   surface** (`AnthillEntrance.cs:34`): C# runs the rect chain on
   EVERY row (`i < position.Y` → the plain IsSolid+Clear+PlaceWall
   variant, else the Blotches variant); the port only ran it inside
   the `i%3==0` arm. The `pair` Circle(2,3) ring also runs every row
   (port had this) and the `i%3` arm re-runs it after the drift
   (`:45`, port had this).
8. **Anthill blowout Expand** (`:48`): `ClearTile` per circle cell,
   then `Expand(1)` places wall 0 on the 3×3 box around EACH cleared
   cell — the port approximated with a second larger ellipse (wrong
   cell set: the union of 3×3 boxes is not an ellipse).
9. **LarvaHole: the wall-0 reset is a SHAPE, not one tile**
   (`LarvaHoleEntrance.cs:41`): `Gen(position+(0,2), All(data),
   PlaceWall(0))` clears the wall on EVERY recorded rect+tail cell
   shifted down 2. The port wrote the single tile
   `(position.X, position.Y+2)`.
10. **Chambers: `OuterOutline` was an ellipse guess**
    (`ChambersEntrance.cs:48`, `ModShapes.cs:28-71`): the real outline
    walks the RECORDED clear-shape (`ShapeData` relative to
    `position`, accumulated across rooms via `Clear().Output(data)`
    inside `Offset+Blotches`), and for each datum applies the chain
    to every 8-neighbour offset NOT in the data (`useDiagonals:true`,
    the 16-entry POINT_OFFSETS table), then `Expand(1)+OnlyTiles(53)+
    SetTile(397)+PlaceWall(187)`. The chain is RNG-free, and a cell
    converted 53→397 fails `OnlyTiles(53)` on any later visit, so the
    HashSet iteration order does not change the outcome (documented;
    the port walks insertion order).
11. **Chambers tunnels: wrong rect origin + missing chain semantics**
    (`:49-62`): `Shapes.Rectangle(2,4)` is `(0,0,2,4)` — NOT centred
    (the port used (-1,-2)); the chain is `IsSolid → Blotches → Clear
    → Expand(1) → PlaceWall(187) → OnlyTiles(53) → SetTile(397)` per
    spread cell: the 3×3 ring gets wall 187, and ring cells that are
    active sand 53 convert to 397 — whose SetTile `Clear` WIPES the
    wall just placed (chain order is load-bearing). The port's three
    separate rects approximated this with wrong offsets and no draws.
12. **Chambers final wall-clear re-filtered the expansion**
    (`:64`): `NotInShape(shapeData)` filters the RECT cell only; the
    `Expand(1)` 3×3 wall-0 writes are NOT re-filtered (the port
    re-checked the mouth set per expansion cell).
13. **Pit `SmootherStep` is missing π** (`PitEntrance.cs:67-71`):
    C# `1.0 - Math.Cos(delta * 3.1415927410125732) * 0.5 - 0.5` (the
    f32-π literal); the port had `1.0 - delta.cos()*0.5 - 0.5` —
    at delta 0.5 the C# gives ≈0.5, the port gave ≈0.061, so the
    taper closed far too early.
14. **Pit bowl depth** (`:49-50`): C# `num6 = holeRadius-|k|;
    num6 = Math.Min(10, num6*num6)`; the port computed
    `min(num6, min(10, num6²))` — at num6=3 the C# digs 9 rows, the
    port dug 3.
15. **Pit `ResetToType(397)` missed liquid=0** (`:34`,
    `Tile.cs:249-259`): the shared `ocean::reset_to_type` seam is
    reused (wall survives — correct, `ResetToType` never touches
    `wall`).
16. **`Shapes.Tail` tile rounding** (`Utils.cs:1812-1815`):
    `ToTileCoordinates` is `(int)v >> 4` (arithmetic shift = floor),
    the port used `(v/16.0) as i32` (truncation) — differs on negative
    coordinates; fixed to the shift.
17. **Chambers direction draw is INVERTED** (`ChambersEntrance.cs:33`):
    C# `num2 = (genRand.Next(2) != 0) ? 1 : -1` — a `Next(2)==0` draw
    yields **-1**. The port had `if rng.chance(2) { 1 } else { -1 }`
    and `chance(n)` is `Next(n)==0`, so it returned **+1** on the same
    draw — the whole chamber/tunnel layout mirrored. Fixed to
    `if rng.range(0,2) != 0 { 1 } else { -1 }`.

**Kept deviations (documented, unchanged):** `SquareWallFrame` /
`TileFrame` framing rides the post-pipeline seam (`desert.rs::
cleanup_area` + `framing::reframe_area`); the C# `ShapeData` HashSet
iteration order is .NET-unspecified — the port uses insertion order
(the RNG-free chains above are order-independent in outcome; the
Smooth chains are order-dependent in C# itself, so vanilla is not
deterministic there either — the port pins insertion order).

## RNG / hash impact

The entrance chains now draw `Blotches` on the `Full Desert` per-pass
stream (2 draws per visited cell + 4 per spread). Every world whose
entrance roll fires changes BY DESIGN. The anniversary gate (P2)
removes the entrance draws entirely on celebrationmk10 worlds (C#
short-circuits before the `NextDouble`). Both `DEFAULT_WORLD` and
`SEED42_512` may move (the 512-wide desert band is 36 tiles wide and
the entrance roll can fire); no compiler on this branch — the re-pin
is DEFERRED to the merge gate, the a==b witnesses pin the ledgers
(the d101/g40-P4 pattern). No other pass's stream is touched.