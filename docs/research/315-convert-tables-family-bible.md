# 315 — The CONVERT-TABLES FAMILY BIBLE (W4-c32-convert, FINAL, code-only)

> **Renumber chain: 281 → 315.** This seal originally took 281 (self-pick per
> §6.3 round 2); research/150 §9 UNIFIED FINAL (W4-d54 `b243c7086`) adjudicated
> the 281-block self-pick race — §7.2's owner keeps 281, the self-pickers take
> 308–317 in claim order — and assigns c32 **315** (§9.2 row 315; claim 9 of
> 10). Executed per §9: `git mv` + header + all refs (INDEX/ANALYSIS/PORT_MAP/
> ROADMAP/anchors/dungeon_styles.rs + world_hash.rs comments); `git grep`
> zero-stale outside this chain-history banner at the pushed tip.

Branch `task/W4-c32-convert` @ base `d27fb00ae` (the W4-c32 tables-only tip),
2026-11-21. Research number **315** (chain 281→315 per registry 150 §9 UNIFIED
FINAL, d54 `b243c7086`; the original 281 pick below is the seal-time history).
Originally picked as 281: registry 150 (d54 tip `72226e500`, round-2
refresh §6.3) declares **281+ FREE** after the 151–236 + 244–280 assignment
blocks; a fresh all-tip sweep (`git ls-tree docs/research/` over all **362**
`task/W4-*` remote tips, post-fetch) found **no 281/282/283 doc at any tip** —
281 was taken per the registry's "next number ≥281 after a fresh sweep" rule
(NOT max-on-origin+1). This doc is the family's FINAL cross-lane declaration:
what landed on THIS branch, the whole `WorldGen.Convert` switch census, the
f25/g42/d1/d103 ownership map with tip hashes, the c32 room-body seam, and the
merger contracts. No game-logic file is touched by this round — the witnesses
are table pins WRITTEN NOT RUN (code-only mandate: no cargo/tests/rustfmt/
graphify; C# read in ≤100-line windows).

## 1. SEAL-1 — the c32 census: what landed (git log, tables ONLY)

`git log` on `task/W4-c32-dungeon-styles` (read at tip `d27fb00ae`, both
commits' `--stat` re-read):

| commit | scope |
| --- | --- |
| `3d0f3c19c` (2026-11-20T18:44:27+02:00) | Port the dungeon biome style tables — `worldgen/dungeon_styles.rs` (824 lines: `DungeonStyleData` + SNOW/DESERT/CRIMSON/HALLOW/CRYSTAL statics, `current_dungeon_style`/`dungeon_style_list`, `style_for_tile`/`style_for_wall`, `wall_tiers`, `DungeonRoomType`, `DungeonFeature`), `dungeon.rs` (+48: the `DungeonColor` window accessors, `DungeonCrawler.cs:63-92`), `mod.rs` (+1), anchors (+2), 10 witnesses |
| `d27fb00ae` (2026-11-20T18:47:17+02:00) | Docs + the hash-pin note (both pins unchanged) + the lookup test's clone fix (clone the data, not the reference) |

**What did NOT land:** no Convert cells, no `BiomeDungeonRoom` bodies, no
`DungeonUtils.ChangeTileType` port, no room/feature passes. The module note
says "UNWIRED: data tables + pure lookups"; both world-hash pins
(`SEED42_512`/`DEFAULT_WORLD`) unchanged (`world_hash.rs`'s W4-c32 note).

**The BiomeDungeonRoom fact (re-verified against the C# this round,
`BiomeDungeonRoom.cs:139-310`):** the room bodies
call **`DungeonUtils.ChangeTileType` — NOT `WorldGen.Convert`**:

- `BiomeRoom_FinishRoom` (`:139-222`): the per-style `switch (StyleData.Style)`
  — default liquid-type 0; case 4 corruption `(25, 23)`, case 5 crimson
  `(203, 199)`, case 6 hallow `(117, 109)` → `BiomeRoom_GrassySurface`;
  case 2 snow → `BiomeRoom_SnowySurface`; case 3 desert →
  `BiomeRoom_SandySurface` (its liquid arm sets `liquidType(1)` — the ONLY
  style that re-types liquid instead of clearing to water-type 0).
- `BiomeRoom_GrassySurface` (`:224-256`): the above-active/inactive/wrong-kind
  refusal, `WorldGen.TileIsExposedToAir` (`:230`/:246 — f25's C1) picks
  grass-vs-dirt-0, `DungeonUtils.ChangeTileType(tile, …, resetTile: false, -1)`
  at `:232/:236/:248/:252`, the `5 + genRand.Next(4)` column walk with the
  `heightsToSkip` list + `liquid = 0`.
- `BiomeRoom_SandySurface` (`:258-290`): kind-396 gate,
  `WorldGen.BlockBelowMakesSandConvertIntoHardenedSand` (`:264`/:280 — f25's
  C2) picks 397-vs-53, `ChangeTileType` at `:266/:270/:282/:286`, same walk.
- `BiomeRoom_SnowySurface` (`:292-310`): kind-161 gate, unconditional 147,
  `ChangeTileType` at `:298/:307`, same walk.
- `ChangeTileType` itself (`DungeonUtils.cs:1577-1590`) is a FIELD write —
  `active(true)`, `Clear(Slope)`, `type =`, optional paint — NO tree kill, NO
  re-frame, NO RNG; `ChangeWallType` (`:1592-1603`) likewise. That is a
  different machine from `Convert_ActuallyConvertTile` (`WorldGen.cs:56312-
  56327`: active+different gate + `TryConvertingOrKillingTreesAbove…` +
  `SquareTileFrame`). The room lane must port the DungeonUtils pair, not
  route through the Convert cells.

The room bodies' only Convert-family dependencies are the two f25 helpers
C1/C2 (landed `59fb188b6`/`8e3eb894b`, wired through f25's own cells). The
room bodies themselves stay c32's wiring round (§5).

## 2. SEAL-2 — f25's Convert 15/16/18/19: zero C# call sites, UNWIRED

f25 closeout 4/5 `9fc704a15` (tip `d587b128d`) ports the four dungeon-biome
cells as `spread::convert_biome_cell` + `convert_sized_biome`
(`WorldGen.cs:56008-56040` type 15 / `:56041-56086` type 16 /
`:56124-56189` type 18 / `:56190-56231` type 19; the sized walk
`:55067-55083` square + Manhattan `< 6`). Re-verified this round:

- **Zero C# call sites.** A whole-decompile grep for
  `Convert\([^)]*,\s*(15|16|18|19)\s*[,)]` returns nothing; every
  `conversionType =` assignment in the decompile is 1/2/4
  (`WorldGen.cs:2045-2059/:18379-18382/:26437-26440/:75009-75016`); the only
  variable-typed Convert sites are `:2064`/`:18416`/`:26569`/`:75121` (those
  1/2/4 feeds), `ConvertSkyIslands`' `convertType` (`:24674`, called once
  with 2 at `:24243`) and the aiStyle-31 `num245` ladder
  (`Projectile.cs:26791-26825`, values 0-7). The four switch arms are dead
  code in 1.4.5.8 — the conversion the biome-dungeon rooms WOULD use if they
  routed through Convert; they don't (§1). f25 ports them UNWIRED as part of
  the family (the C1-C8 precedent); the c32 tables are NOT their consumer.
- **The cell shapes** (f25 body re-read at tip, windows cited above): 15
  wall→80 unconditionally, torch→22, thorn KILL, then the INDEPENDENT
  `!tileFrameImportant[type]` catch-all →70/59 by C1 exposure; 16 the
  HardenedSand/Dirt/Snow wall union→216 else 187, torch→16,
  grass/sand/snow/dirt→53-or-397 via C2, hardened→397, thorn kill (69
  exempt), non-396 catch-all→396; 18 union wall→2 / NewWall→212-215 / else
  349 with the LOAD-BEARING else-if fall-through (a wall already at its
  union/NewWall target falls to 349), torch→0, golf→477 before grass→2,
  sand-family→0-or-2 via C1, thorn kill, non-1 catch-all→1; 19 the wall>0
  switch (63/69/70/81→64, 15 no-op, default 15), torch→21, thorn CONVERTS
  to 69 (not killed), catch-all→60/59 by C1. The 15/19 catch-alls re-read
  the LIVE tile (the C# `ref Tile` — a thorn kill deactivates it;
  `Convert_ActuallyConvertTile`'s `active()` gate refuses the write).
- **c32-side pins:** the landed tables' palettes are the same ids these
  cells produce (15→70/59/80 = GlowingMushroom style 7's brick-grass/brick/
  wall `:598-601`; 16→396/397/53 + walls 187/216 = DESERT `:397-401`;
  18→1/2/0 + walls 2/212-215/349 = the pure baseline; 19→60/59/64/15 =
  Jungle style 8 `:718-722`). 3 witnesses pin the pairs on THIS branch
  (§6); the cell bodies stay f25's — NO copy lands here.

## 3. SEAL-3 — g42's Convert(3): FOUR sites, the fourth is the spray

g42 GROWTH FINAL `fdedf9776` corrected the census to FOUR sites; GROWTH SEAL
`910cc9747` (tip) completed the fourth site's sized walk. Re-verified:

| # | C# site | context | port |
| --- | --- | --- | --- |
| 1 | `WorldGen.cs:2101` | `DoSurfaceIsMushrooms` noSurface arm (kind 60) | g42 G6/G7 (`e2707f2d0`), gen-only |
| 2 | `WorldGen.cs:2125` | `DoSurfaceIsMushrooms` normal arm (kind 60) | same |
| 3 | `WorldGen.cs:2129` | same arm (wall 15) | same |
| 4 | `Projectile.cs:26834` | aiStyle 31 (`:26786-26886`): the `num245` ladder `:26805-26808` maps proj **148 → type 3**; owner-gated `Convert(point2.X, point2.Y, num245, size)`, size 2 (3 when `ai[1] == 1`) | `spread::convert_sized_mushroom` (`910cc9747`) — the same square + Manhattan-<6 walk, per-cell `convert_cell_mushroom` (RNG-free) |

Proj 148 = Clentaminator 779 shoot 145 + DarkBlueSolution 783 shoot 3
(`Player.cs:53487-53490`, `Item.cs:10544`); Terraformer 5134 resolves the
same additive arm. The UpdateWorld side has NO Convert(3) — the runtime
grass-70 spread is `SpreadGrass(59, 70)` (g42 T-3/D1/D4). **Merger
contract (g42's, quoted at tip):** ONE arm into f25's `ai_spray` match —
`3 => tr_core::spread::convert_sized_mushroom(world, tx, ty, size)` beside
f25's `0/2/5/6/7` arms; types 1/4 stay d1's `convert_sized`. THIS branch's
`ai_spray` (`projectile_ai_misc.rs:78-102`) still carries only the type-0
cleanse (the S1-D-4 slice) — the f25/d1/g42 arms all arrive at their merges;
no c32 edit belongs in that match.

## 4. The Convert-switch family bible (the whole switch, :55098-56284)

The single-cell head `:55085-55097` (InWorld(1), the `!active && wall <= 0`
empty-cell refusal, the type/wall snapshot) feeds every arm; the writers are
`Convert_ActuallyConvertTorch` `:56287-56310` (the type-0 style-22→21 quirk
`:56294-56297`), `Convert_ActuallyConvertTile` `:56312-56327` (active +
different, the tree-invalidity cross-kill, `SquareTileFrame`) and
`Convert_ActuallyConvertWall` `:56329-56343` (non-zero + different,
`SquareWallFrame`). The `RainbowSandAndBlackSandWalls` paint arms
(`:56302-56306`/`:56319-56323`/`:56335-56339`, types 1/4/2 only) are the
d1-documented skip; the `NetMessage` tails are MP-only. The port's re-frame
is the `reframe_area` superset (the `convert_cleanse_cell` precedent).

| type | C# window | flavour | owner (tip) | state |
| --- | --- | --- | --- | --- |
| 0 | `:55527-55664` + head | cleanse (the wall-80 depth rolls live here) | f25's base S1-D-4 slice — ON THIS BRANCH (`spread.rs::convert_cleanse` :413) | landed, wired (`ai_spray` 145) |
| 1 | `:55261-55336` | corruption | d1 `convert_powder_cell` (`78dc52720`, tip `212516a82`) | landed on d1, unmerged |
| 2 | `:55176-55260` | hallow (the snapshot-59 mud arm reads LIVE 109 neighbours `:55256`) | f25 C3 `convert_hallow_cell` (`6c5dcea11`) | landed, wired (spray 146 / holy water / spread_grass / ConvertSkyIslands) |
| 3 | `:55337-55362` | glowing mushroom | g42 `convert_cell_mushroom` (`e2707f2d0`) + `convert_sized_mushroom` (`910cc9747`, tip) | landed on g42, unmerged — §3 |
| 4 | `:55100-55175` | crimson | d1 `convert_powder_cell` | landed on d1, unmerged |
| 5 | `:55363-55408` | desertify (C2 hardening floor) | f25 C4 `convert_spray_cell` (`0e6bee9ef`) | landed, wired (spray 1015) |
| 6 | `:55409-55445` | snowify | f25 C4 | landed, wired (spray 1016) |
| 7 | `:55446-55526` | purify (C1 exposure split) | f25 C4 | landed, wired (spray 1017) |
| 8 | `:55665-55670` | chlorophyte seeding (59/60→211) | f25 C5 `convert_chlorophyte_cell` (`a6b21b728`) | landed, UNWIRED (d103 P8 owns hardUpdateWorld) |
| 9 | `:55671-55720` | jungle-ification (the full grass switch) | f25 C5 / d103 P8 `convert_purge_cell` (`014cf2937`, tip `12318e505`) | d103 POST FINAL `12318e505` PF-3: KEEP f25 C5, DROP d103's body at the merge (missing the diff gate/cross-kill/reframe) |
| 10 | `:55721-55763` | evil purify (no pure-grass/dirt rows) | same pair | same verdict |
| 11 | `:55764-55882` | powder purify (the wall RNG roll `:55783`) | d1 `convert_powder_cell` | landed on d1, unmerged |
| 12 | `:55883-55922` | corruption-II (catch-all→25) | unclaimed | ZERO C# call sites |
| 13 | `:55923-55962` | crimson-II (catch-all→203) | unclaimed | ZERO C# call sites |
| 14 | `:55963-56007` | hallow-II (catch-all→117) | unclaimed | ZERO C# call sites |
| 15 | `:56008-56040` | dungeon mushroom | f25 closeout `convert_biome_cell` (`9fc704a15`) | landed, UNWIRED — §2 |
| 16 | `:56041-56086` | dungeon desert | f25 closeout | landed, UNWIRED — §2 |
| 17 | `:56087-56123` | snow-II | unclaimed | ZERO C# call sites |
| 18 | `:56124-56189` | dungeon pure | f25 closeout | landed, UNWIRED — §2 |
| 19 | `:56190-56231` | dungeon jungle | f25 closeout | landed, UNWIRED — §2 |
| 20 | `:56232-56252` | temple (catch-all→226) | unclaimed | ZERO C# call sites |
| 21 | `:56253-56283` | dungeon repaint — reads `GenVars.CurrentDungeonGenVars` (brick ?? 41, wall ?? 17) | **c32/d107 GenVars seam** (d107 tip `266729126`) | unported; the c32 tables are the palette source when the seam lands |

The shared conversion sets (`TileID.cs:83-97`): Grass {2,23,199,109,477,492},
JungleGrass {60,661,662}, GolfGrass {477,492}, Dirt {0}, Snow {147}, Stone
{1,25,117,203}, Ice {161,163,164,200}, Sand {53,112,116,234}, HardenedSand
{397,398,402,399}, Sandstone {396,400,403,401}, Thorn {32,352,69,655}; the
torch default style set {0,9,16,18,19,20,21,22} (`TileID.cs:63-77`).

**The style-ordinal vs conversion-type collision (the c32-specific trap):**
`DungeonGenerationStyleData.Style` ordinals (Shimmer 11, Spider 12,
LivingWood 13, Cavern 1, Snow 2, Desert 3, Corruption 4, Crimson 5, Crystal
15, Hallow 6, GlowingMushroom 7, Beehive 9, LivingMahogany 14, Jungle 8,
Temple 10 — `DungeonGenerationStyles.cs:191/:231/:270/:309/:354/:395/:436/
:476/:516/:555/:596/:636/:676/:716/:762`) are a DIFFERENT numbering from
Convert's `conversionType`. Crystal's Style **15** is NOT Convert type 15
(glowing mushroom); Snow's Style 2 is NOT Convert type 2 (hallow). The
`BiomeRoom_FinishRoom` switch (§1) reads STYLE ordinals; nothing in the room
family ever passes a style ordinal to Convert. A 2nd witness pins the
disjoint pairs on this branch's tables.

## 5. The c32 room-body seam (the wiring round's contract)

When c32's wiring round ports the room bodies it must:

1. Port `DungeonUtils.ChangeTileType`/`ChangeWallType`
   (`DungeonUtils.cs:1577-1603`) as the field-write pair (active-true +
   slope-clear + kind/paint; wall + wall-paint) — NOT route through
   `Convert_ActuallyConvertTile` (different semantics: no tree kill, no
   re-frame, no active/different gate — ChangeTileType writes even when
   inactive or same-kind).
2. Call f25's C1 `tile_is_exposed_to_air` (`59fb188b6`) at the
   GrassySurface `:230`/:246 slots and C2
   `block_below_makes_sand_convert_into_hardened_sand` (`8e3eb894b`) at the
   SandySurface `:264`/:280 slots — both landed on f25, no c32 copy.
3. Keep the `5 + genRand.Next(4)` column walk + `heightsToSkip` +
   `liquid = 0` verbatim (the walk draws ONE Next(4) per surfaced column;
   the snow arm has no exposure/hardening branch — unconditional 147).
4. Read the STYLE ordinals from the landed tables (SNOW.style 2 / DESERT 3
   / CRIMSON 5 / HALLOW 6; corruption style 4's static is a cited skip —
   `DungeonGenerationStyles.cs:434-473` — and rides the wiring round).
5. Convert type 21 (the dungeon repaint) is the c32/d107 GenVars seam:
   `CurrentDungeonGenVars.brickTileType/brickWallType` (the `DungeonColor`
   triple on this branch is the classic-world stand-in), NOT a fixed
   palette. Types 15/16/18/19 stay UNWIRED forever unless a future C#
   version grows call sites — they are family completeness, not room
   machinery.

## 6. This round's witnesses (WRITTEN NOT RUN)

3 table-pin witnesses appended to `dungeon_styles.rs`'s test module (the
existing 10 stay untouched; 13 total on the branch):

- `w4_c32_convert_seal_the_room_styles_match_the_finish_room_switch` — the
  four in-scope style ordinals are exactly `BiomeRoom_FinishRoom`'s case
  labels (`BiomeDungeonRoom.cs:168-217`: 2 snow / 3 desert / 5 crimson /
  6 hallow; case 4 corruption's static is the cited skip), and the tables'
  brick ids are the surface walkers' SOURCE kinds: SnowySurface gates on
  161 = SNOW.brick_tile (`:294`) and writes 147 ∈ SNOW.extra_tiles
  (`:298`); SandySurface gates on 396 = DESERT.brick_tile (`:260`) and
  writes 53 = DESERT.brick_cracked_tile / 397 ∈ DESERT.extra_tiles
  (`:266/:270`); GrassySurface's tileTypeToReplace args are 203 =
  CRIMSON.brick_tile (`:181`) and 117 = HALLOW.brick_tile (`:191`).
- `w4_c32_convert_seal_style_ordinals_are_not_convert_types` — the
  collision pin: Crystal.style 15 but its palette (385/116/186) shares no
  id with Convert-15's outputs (70/59/80); Snow.style 2 but 161/224/71 ≠
  Convert-2's hallow targets (117/109/28); Desert.style 3 ≠ Convert-3
  mushroom (70/80), Crimson.style 5 ≠ Convert-5 desertify (53/397/396),
  Hallow.style 6 ≠ Convert-6 snowify (147/161) — the two numberings are
  disjoint namespaces (§4).
- `w4_c32_convert_seal_the_biome_cell_outputs_match_the_table_palettes` —
  the f25-cell↔c32-table cross-pin: Convert-16's tile outputs {396,53,397}
  ⊆ DESERT's {brick, cracked, extras} and its wall outputs {187,216} ⊆
  DESERT's {brick_wall} ∪ extra_walls; Convert-2's stone 117 / sand 116 /
  wall 28 are HALLOW's brick / cracked / brick_wall; the GlowingMushroom
  (style 7, `:596-601`: 70/59/80) and Jungle (style 8, `:716-722`:
  60/59/64 + extra wall 15) C# rows are Convert-15/19's catch-all outputs —
  pinned as cited literal tuples (those two statics are out-of-scope skips
  on this branch; the witness documents the seam without porting them).

No cargo/rustc/rustfmt/tests/dotnet/graphify invoked (code-only mandate).
Both world-hash pins unchanged — this round adds tests + docs only.

## 7. Tip-hash ledger (the family, read this round)

| lane | tip | role |
| --- | --- | --- |
| W4-c32 (base) | `d27fb00ae` | tables-only (§1) |
| W4-f25-worldgen-passes-8 | `d587b128d` | C1-C8 + types 0/2/5/6/7/8/9/10/15/16/18/19, the spray wiring, ConvertSkyIslands, the 12g FINAL ledger |
| W4-g42-biome-mushroom | `910cc9747` | type 3 (4 sites), GROWTH SEAL |
| W4-d1-proj-ai-1-10 | `212516a82` | types 1/4/11 (the powder machine) |
| W4-d103-worldgen-passes-3 | `12318e505` | hardUpdateWorld verbatim; PF-3 keep-f25-C5 verdict |
| W4-d107-worldgen-special-seeds | `266729126` | the type-21 GenVars seam co-owner |
| W4-d54-dust-sites-world | `72226e500` | registry 150 round-2 (the number authority) |

Merger contracts (unchanged, restated for the family): d1's CONV_THORN
dedupes onto f25's constant; d103's `convert_purge_cell` body DROPS onto f25
C5 (PF-3) with its two consumers rewired; g42's ONE `3 =>` arm into f25's
`ai_spray` match; g44's finish_seeds module note superseded by f25
`f0fe0f388`; the c32 room bodies call f25 C1/C2 + the DungeonUtils pair
(§5). The 12g ledger's coverage map and this bible agree cell-for-cell.