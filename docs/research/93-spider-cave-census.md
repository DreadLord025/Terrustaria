# 93 — Spider cave biome gen census (W4-g41)

Code-only census of the spider-cave worldgen against `decompiled/Terraria/WorldGen.cs`,
scoped to the three axes of the task brief: **web placement**, **egg sacs** (the spider
stalactite family + its support validator), **wall coverage** (the wall-62 flood, the
`ILied` sweep, `SpiderWallRemover`). Landed home: `crates/tr-core/src/worldgen/spider_gem_moss.rs`
(M5-STRUCT-5), NOT `mini_biomes.rs` (the micro-biome classes carry no spider content —
`GameContent/Biomes` has no SpiderNest class; the spider cave is a `WorldGen` pass, not a
`Biomes.Place` micro-biome). Every row below was read against the C# source, not the docs.

## Landed (M5-STRUCT-5) — verified verbatim or draw-faithful

| C# | Port | Verdict |
| --- | --- | --- |
| `SpiderCaves` pass `:17356-17427` (pocket search, `500 <= count < 3500`, `maxTilesX/2` retries, `maxTilesX*0.005` attempts) | `pass_spider_caves` `spider_gem_moss.rs:953` | faithful classic path; FTW ×3 (`:17378`) via `spider_attempts`; `noSpiderCavesActuallyNoSpiderCaves` gate (`:17358`, `:269-278`) via `no_spider_caves_actually_no_spider_caves` |
| `countTiles`/`nextCount` `:9546-9635` | `count_tiles` `:214` | iterative stack stand-in for the recursion (no RNG in the fill — traversal order free); the `wall != 0` gate reads the port's pre-laid baseline as the C#'s 0 (`BASELINE_WALLS`, documented M5-STRUCT-5 deviation) |
| `Spread.Spider` `:3682-3789` | `spread_spider` `:600` | **draw-faithful incl. the duplicate-visit quirk**: the C# `hashSet.Add` (`:3716`) ignores its return, so a cell enqueued twice in one wave is processed twice — but the two depth-gate draws (`:3706`, `:3711`) run on EVERY visit before the dedup, and the second visit hits the `wall != 0` branch (no draws, no re-decor). The port draws both gates before `seen.insert` (`:621-628`) and skips the re-process — same draw count, same writes |
| `PlaceTight` `:38303-38325` | `place_tight` `:392` | the shimmer/larva veto + the `Next(2)`/`Next(3)` pair faithful; **the `CheckStalactite` tail `:38320-38323` is MISSING** (below) |
| `PlaceUncheckedStalactite` `:38327-38641` | `place_unchecked_stalactite` `:409` | the spider family (`108 + variation*18`, two tiles, `:38334-38350`) faithful; the hanging ice/stone-moss/hive/sandstone arms + the rising stone-moss/sandstone/granite/marble arms faithful; **MISSING arms below**; `CopyPaintAndCoating` folded into the frame write (documented) |
| `PlacePot` `:54066-54114` | `place_pot` `:507` | the 2×2 seat validation + `Next(3)*36` variant + style frames faithful; **the `halfBrick(false)` write `:54108` and the remix centre veto `:54069-54072` MISSING** |
| `PlaceSmallPile` `:46815-46873` | `place_small_pile` `:541` | the size-1 2-wide seat + the size-0 1-wide seat faithful; **the `InvalidTileForPilesOrSpeleothems` boulder-seat veto `:46847-46850` MISSING**; the `anyLava` gate reads the kind without the `liquid > 0` guard (`Tile.cs:410-415` requires both) |
| `WebsInSpiderCavesAndHoneyPlusSpeleothemsInBeehives` `:19995-20063` | `pass_webs_and_honey` `:1201` | the wall-86 honey + `Next(3)` speleothem arm and the wall-62 `Next(10)!=0` → `Next(2,5)` solid-scan → cobweb-51 arm faithful (the scan's inner-only `break` `:20050` matches the port's `'scan` label); **the DST row-50 start `:20003-20006` and the `ILied` honey arm `:20022-20028` MISSING** |
| `DoNoSpiderCavesILiedMoreSpiderCaves` `:2618-2630` | `micro_finish.rs:442` `no_spider_caves_i_lied` | the 8-wall → 62 sweep faithful, but **UNWIRED**: the C# dispatches it from the SpiderCaves pass tail (`:17422-17425`); the port's `pass_spider_caves` never calls it |

## MISSING — ported verbatim by this task (W4-g41)

1. **The egg-sac validator chain** (the spider stalactite's support check — the "egg sac"
   is tile 165 frameX `108..144`, the two-tall spider family):
   - `CheckStalactite` `:38996-39175` — the four frameY arms (72 small-hanging, 90
     small-rising, ≥36 tall-rising, else tall-hanging with the 18→top-cell step), each
     validating the support + the pair's type/frameX agreement and killing the object
     through `destroyObject = true` + `KillTile` on failure.
   - `UpdateStalagtiteStyle` `:38643-38718` — the `netMode == 1` early-true, the
     current/desired style pair, and on mismatch the **`Next(3)*18` draw** + the
     13-case desired-style frame base switch (0→54, 1→216, 2→270, 3→324, 4→378, 5→432,
     6→486, 7→+0, 8→540, 9→594, 10→648, 11→108, 12→162) rewritten over the object's
     height. This is the chain's ONLY RNG draw.
   - `GetStalagtiteStyle` `:38823-38872` — the `frameX / 54` → style decode
     (0→7, 1→0, 2→11, 3→12, 4→1, 5→2, 6→3, 7→4, 8→5, 9→6, 10→8, 11→9, 12→10, else fail).
   - `GetDesiredStalagtiteStyle` `:38720-38821` — the frameY → (support-side, height, y)
     decode, the stone/moss → style 0 with the **spider override** (`num == 3 &&
     tile.wall == 62` → style 11, `:38766-38769`), the support-kind switch
     (200→10, 164→8, 163→9, 117/402/403→1, 25/398/400→2, 203/399/401→3, 396/397→4,
     367→6, 368→5, 147/161→7) and the hive small-only arm (225 → 12 for sides 0/1, else fail).
   - `InvalidTileForPilesOrSpeleothems` `:38979-38994` — the `InWorld(x,y,2)` + active +
     `TileID.Sets.Boulders` test (`TileID.cs:504`: 138, 484, 664, 665, 711-716).
   - Wiring: `PlaceTight`'s tail (`:38320-38323`) — after the placement, if the cell is
     active kind 165, run `CheckStalactite`.
2. **`PlaceUncheckedStalactite` missing arms**: the hanging granite (`368` → `432+v*18`,
   `:38448-38476`) and marble (`367` → `486+v*18`, `:38477-38505`) arms; the rising hive
   (`225` → `162+v*18` frameY 90, `:38543-38552`); `Main.tileMoss[627]` (`Main.cs:7031`,
   the rainbow moss) in the moss-support test.
3. **`PlaceSmallPile`** boulder-seat veto (`:46847-46850`) + the `anyLava` `liquid > 0` guard.
4. **`PlacePot`** `halfBrick(false)` write (`:54108`).
5. **Webs pass**: the DST `num = 50` start row (`:20003-20006`); the `ILied` honey arm
   (`:20022-20028` — wall-62 liquid converts to honey instead of clearing).
6. **Spider pass**: the `shroomCount > 1` retry veto (`:17407-17410` — sets the count to
   0 INSIDE the retry loop only, forcing another retry); the `ILied` tail dispatch
   (`:17422-17425` → `micro_finish::no_spider_caves_i_lied`); the remix y-band draws
   (`:17392-17395`, `:17402-17405` — `Next(worldSurface, rockLayer)` instead of the deep
   band, gated on `SeedFlags::REMIX`, classic unchanged).
7. **`SpiderWallRemover` `:77279-77344`** — the drifting diamond wall-62 eraser: the
   `Next(8,21)` budget / `Next(8,33)` life pair, the `Next(-10,11)*0.1` velocity pair, the
   per-step `num*(life/total)` radius, the `|dx|+|dy| < radius*0.5*(1+Next(-10,11)*0.015)`
   diamond test clearing wall 62 → 0, and the `Next(-10,11)*0.05` velocity jitter with the
   ±1 clamps. **WIRED by the W4-g41 follow-up**: its only callers are `FinishNotTheBees`'
   two search loops (`:25409-25434`, the FTW+NTB combo), ported in
   `not_the_bees.rs::spider_wall_search` and dispatched from `step_seed_finish`'s NTB slot
   (census `docs/research/94-not-the-bees-host-census.md`).

## Census deviations recorded, NOT changed (out of this task's scope)

- **`Spread.Gem`'s duplicate-visit redraw** (`:3617`): unlike Spider, the gem BFS's
  second visit of a cell re-runs the `randGemTile` conversions (the C# `hashSet.Add`
  ignores its return and the gem branch draws). The port's `spread_gem` dedups
  (`spider_gem_moss.rs:328`) — a real RNG deviation for the Gem Caves pass, left for
  the gem-fidelity row. `Spread.Moss`'s duplicates are draw-free (idempotent writes),
  so its dedup is faithful.
- The NTB+FTW combo branch (`:17367-17373`), `roundLandmasses` (`:17374-17377`, the
  micro seed is deferred M5-SEED-2), the `notTheBees` `tileSolid[225]` toggles
  (`:17382-17385`/`:17417-17420`) and the `tileSolid[162] = true` pass tail (`:17421`,
  the boot static already true `Main.cs:7145`) stay with the seed-fidelity rows.
- `AddBuriedChest`'s full search (`:35877+`) stays the port's `place_chest` +
  `chest_stock` seam (M5-CHEST-gen); the `notNearOtherChests` scan is not modelled.
- The `SquareWallFrame` (`:3728`) / `TileFrame` (`:20057`) calls ride the port's
  post-pipeline framing seam (the M5-STRUCT-5 idiom).

## RNG / hash contract

- All new draws ride the EXISTING world-pass slots: the egg-sac validator's `Next(3)*18`
  draws on `pass_rng("Spider Caves")` (via `Spread.Spider` → `PlaceTight`) and
  `pass_rng("Webs And Honey")` (via the hive speleothem arm); the remix y-bands and the
  shroom veto add draws only on REMIX / shroom-bearing worlds. No new stream slot.
- The validator KILLS unsupported sacs and RESTYLES mismatched ones, the missing arms
  PLACE new stalactites, and the pile veto REMOVES boulder-seated piles — so classic
  worlds change by design. `SEED42_512` / `DEFAULT_WORLD` move; the re-pin is DEFERRED
  to the merge gate (no compiler on this branch, the S1-K-2 wave B precedent). The
  `a == b` determinism witnesses are written NOT run.

## AMBIENCE census (W4-g41 FINAL) — the spider biome has NO music/wall-ambience gate in 1.4.5.8

The brief's ambience axis was censused against the decompile before any port; the
verdict is **NOTHING TO PORT** — the C# has no spider-music gate, and the port's
landed generic machinery already covers everything the spider cave actually uses.
Co-lane claim check: `origin/task/W4-g24-sounds-worldgen` (tip `4c410bd1c`, the
WorldGen.cs PlaySound census — 83 sites, 3 batches) claims the WorldGen sound
sites; the g24b sweep window overlaps NONE of the rows below because the spider
passes contain zero sound sites. No claim conflict; nothing unclaimed to take.

| Axis | C# evidence (read, not remembered) | Port state |
| --- | --- | --- |
| Spider music track | `ID/MusicID.cs` full read (229 lines, `Count = 105`): NO Spider/SpiderCave/Nest row exists. The only cave-biome tracks are Temple 26, UndergroundDesert 61, UndergroundJungle 54, Mushroom 29 — no wall-62 track | nothing to port |
| Music gate | `Main.UpdateAudio_DecideOnNewMusic` (`Main.cs:12175-12930`): the biome chain (`:12666-12815`) tests ZoneShimmer / temple wall 87 (`:12690`) / ZoneDungeon / bgStyle 9 / evil / meteor / graveyard / underground-desert / desert / jungle / snow — NO spider arm. `Main.cs` has ZERO `wall == 62` reads and zero case-insensitive "spider" mentions | the spider cave falls to the generic underground arm (`:12830-12856`, the ugMusic 4/31 alternation) — ALREADY PORTED (`audio.rs:2814-2883`, the `MusicEnv` ladder + `ug_music` state) |
| SceneMetrics spider count | `SceneMetrics.cs` full spider grep: ZERO members (no SpiderTileCount, no ZoneSpider). The wall reads (`:685-692`) are dungeon/lihzahrd/granite/marble/hive/gem/desert only; wall 62 appears NOWHERE in the class | `scene.rs` mirrors the class (`:816-817` the wall-zone arms) — faithful as landed |
| Wall ambience | `SceneMetrics.BehindBackwall` (`:664`, `tileSafely.wall > 0`) is the generic backwall flag the ambience fades read — wall 62 rides it like every wall, no spider-specific arm | `ambient_sound.rs:89` (the behind-backwall fade) — ALREADY PORTED, generic |
| Ambient cue tracks | `Main.cs:11539-11605`'s per-track ambient switch has exactly TWO cases: 28 (RainAmbience) and 45 (WindAmbience); everything else rides `UpdateCommonTrack`. No spider cue | `audio.rs`'s ambience inputs (the M1.4-BG-20 rows) cover both cases — nothing missing |
| NPC idle sounds in the cave | `NPC.cs:57340-57367`'s fighter-moan type sets (the `DespawnEncouragement_AIStyle3` idle arms): the Wall Creeper 164 / Black Recluse 163 are in NO set (the zombie/skeleton 1000-roll, the mummy 500-roll, the vampire/Frankenstein/face-monster arms, the armored-bones range — none names 163/164). The wall forms 165/238 are aiStyle 40, not the fighter family | `ambient_sound.rs::fighter_ambient_moans` carries the full C# type sets verbatim — 163/164 correctly absent |
| Worldgen sound sites | `WorldGen.cs` PlaySound grep over the four spider windows (`Spread.Spider` `:3682-3900`, `SpiderCaves` `:17356-17430`, `Webs…` `:19995-20063`, the stalactite family `:38303-39175`): **0 sites** | g24's census owns the WorldGen sound lane; no spider row exists to claim |

The spider cave's audible identity in the vanilla is the generic underground
track pair + the spiders' own hit/death cues (`NPCHit29`/`NPCDeath32`,
`NPC.cs:11038-11039`/`:11067-11068`) — the cues ride the SPAWN pool row below
(the kind registration), not a music gate. This section closes the ambience
axis: no code change, no claim, no witness needed (there is no behaviour).
