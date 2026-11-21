# 12k — W4-c31 jungle residuals census + gap-fill

Branch `task/W4-c31-hive-rewrite` (continues the landed hive rewrite, DONE
d538536b). Code-only: no cargo / tests / graphify run. The hive module landed;
this pass censuses the **rest of the C# `WorldGen` jungle family** against the
port and ports the missing pieces verbatim.

## Census — C# jungle-family passes vs landed

Line numbers into `decompiled/Terraria/WorldGen.cs` unless noted. "Landed" =
present on this branch at d538536b.

| C# pass / fn | Anchor | Port | Verdict |
| --- | --- | --- | --- |
| `JunglePass.ApplyPass` | `JunglePass.cs:19-63` | `biomes.rs::pass_jungle` | faithful (M5-BIOME p3) |
| `MudCavesToJungleGrass` | `:12468-12499` | `biomes.rs::pass_mud_caves_to_jungle_grass` | faithful |
| `DirtToMud` | `:13123-13142` | `biomes.rs` (`dirt to mud` step) | faithful |
| `UndergroundJungleTrees` + `GrowUndergroundTree` | `:17840-17858` / `:30849-31299` | `biome_detail.rs::pass_underground_jungle_trees` | faithful (`pass_rng("Jungle Trees")`) |
| `SurfaceWaterInJungle` (Wet Jungle) | `:15810-15833` | `biome_detail.rs::pass_wet_jungle` | faithful |
| `GlowingMushroomPlantsUndergroundAndJunglePlants` (jungle arms) | `:20122-20164` | `biome_detail.rs::pass_mushroom_and_jungle_plants` | faithful (jungle arms) |
| `JunglePlantsPart2` | `:20165-20192` | `biome_detail.rs::pass_jungle_plants_part2` | faithful |
| `DirtWallsIntoMudWallsInJungleAndJungleMinMax` | `:20813-20866` | `biome_detail.rs::pass_muds_walls_in_jungle` | faithful |
| `LihzahrdTemple` + `makeTemple` | `:15834-15926` / `:33435-34237` | `temple.rs::pass_lihzahrd_temple` | faithful (M5-TEMPLE) |
| `LihzahrdTemplePart2` + `templePart2` | `:17701-17709` / `:34464` | `temple.rs::pass_temple_part2` | faithful |
| `LihzahrdAltar` | `:21943-21972` | `temple.rs::pass_lihzahrd_altar` | faithful |
| `Beehives` + `HiveBiome` | `:15927-15976` / `HiveBiome.cs` | `hive.rs` | faithful (W4-c31, landed) |
| `BeeLarvaInBeehives` | `:20868-20900` | `hive.rs::pass_larva` | faithful (W4-c31, landed) |
| **`JungleShrines`** | **`:15977-16131`** | `structures.rs::pass_jungle_shrines` | **SHAPE-LEVEL** ("the C#'s exact brick patterns are not 1:1", module doc) → **ported faithful here** |
| `ChestsInJungleShrines` | `:17211-17244` | `structures.rs::pass_chests` (shrine arm) | faithful item/jitter draws (S1-A-4); the `AddBuriedChest` failure-retry (`:17222-17240`) is the storage-task follow-up |
| **`GeneratePlanteraBulbOnAllMechsDefeated` + `AttemptToGeneratePlanteraBulbAt`** | **`:74127-74267`** | — | **MISSING** → **ported here** (`growth/plantera.rs`) |
| **`UpdateWorld_GrassGrowth` case-60 hardmode tail (life fruit + bulb + 233)** | **`:74414-74496`** | `growth/grass.rs::update_world_grass_growth` stops at `:74424` | **MISSING** → **ported here** |
| `UpdateWorld_GrassGrowth` case-70 tail (cattail + mushroom tree) | `:74497-74506` | — | residual, same dispatcher; **ported here** (cattail + tree) |
| `Piles` jungle arm (`tile2.type == 60` → 187/`Next(6)`) | `:18856-18860` | W4-d104 co-lane (`pass_piles`) | **claimed by W4-d104**, not this lane |

### Co-lane claims read (not duplicated)

* **g43 `task/W4-g43-biome-jungle-temple`** — 0 commits ahead of `origin/main`
  (claim-only branch). The temple is already faithful on this branch via
  M5-TEMPLE (`temple.rs`), so nothing to port for the temple; the census
  confirms it. The ee937411-adjacent claim is W4-g41 (spider cave witness
  fixes), already merged — read, no overlap.
* **g84 `task/W4-g84-plantera-bulb-draw`** — 0 commits ahead (claim-only);
  the **draw** side (tile 238 `drawdata.rs`/`drawdata_anim.rs` already carry
  the bulb frame rows). This lane ports the **placement / worldgen side**
  (`:74127-74267` event + `:74414-74448` growth), which is missing.
* **g88 `task/W4-g88-life-fruit-use`** — the **item-use** side
  (`life_fruit.rs`, `crystals.rs`, `ItemCheck_UseLifeFruit`); this lane ports
  the **growth** side (tile 236 placement), which is missing. No overlap.
* **W4-d104** owns the `Piles` pass (incl. the jungle arm); not touched here.

## What this pass ports (verbatim, fresh `pass_rng` slots)

1. **`JungleShrines` faithful rewrite** (`structures.rs::pass_jungle_shrines`,
   `WorldGen.cs:15977-16131`): the `GenVars.jungleHut` material pick
   (`Reset` `:11167` `Next(5)` → `:11317-11336` map to 119/120/158/175/45),
   the hut count `Next(7,12) * maxTilesX/4200`, the dungeon-opposite-half
   origin search with the `num4 > maxTilesX*10` retry valve, the 30-radius
   hive/temple proximity veto (`225/229/226/119/120` tiles + walls `86/87`),
   `structures.CanPlace(area, 1)`, the brick shell + carved interior with the
   hut-specific wall, the 100-try torch (`PlaceTile(4, style 3)`), the floor
   carve, the mud pillars, the shrinking roof rows, the `JChestX/Y`
   publication + `AddProtectedStructure`, and the `tileSolid[137] = false`
   window. Fresh `pass_rng("Jungle Chests")` slot (the shape-level pass rode
   the shared `srng`; the faithful pass isolates its draws).
2. **Plantera bulb placement** (`growth/plantera.rs`):
   `GeneratePlanteraBulbOnAllMechsDefeated` (`:74127-74186`) +
   `AttemptToGeneratePlanteraBulbAt` (`:74188-74267`), verbatim. The
   `NPC.OnGameEventClearedForTheFirstTime` case-16/17/18 hook
   (`NPC.cs:78780-78788`) wires it at the mech-boss death path.
3. **Life-fruit / bulb / 233 hardmode growth** (`growth/grass.rs` case-60 tail,
   `:74414-74496`): the `Next(25)` gate + liquid veto, the all-mechs
   `Next(60)` bulb arm with the 150-radius stride-2 density scan, the
   any-mech `Next(30/40)` life-fruit arm with the 60/50-radius scan, and the
   233 detritus fallback (`Next(8)` styleY 0, `Next(12)` styleY 1). Plus the
   case-70 cattail/tree tail (`:74497-74506`).

## Deviations (recorded in ANALYSIS "Jungle residuals (W4-c31)")

* The `JungleShrines` rewrite moves the shared-`srng` draw schedule (the
  faithful pass draws its own count/origin/size/torch/roof rolls on
  `pass_rng("Jungle Chests")`), so every downstream `srng` consumer re-rolls
  and the shrine tiles change. Both world-hash pins move **by design**;
  stale-by-design, the merge gate re-pins. `a == b` stays green (the pass is
  a pure function of world + stream).
* `PlaceJunglePlant`'s `KillTile` collapse + `CopyPaintAndCoating` no-op follow
  the existing `biome_detail.rs::place_jungle_plant_detritus` precedent (the
  box is fully rewritten on success; paint is 0 during generation / the
  growth tick carries no paint source here).
* The bulb event's `Main.dungeonX` reads the port's `World.dungeon_x`
  (published by the dungeon step / read from `.wld`); `Main.UnderworldLayer`
  = `maxTilesY - 200` (`Main.cs:2790`).
* `Main.expertMode` is constants-off in the port (single-difficulty), so the
  life-fruit `maxValue2` collapses to 40 and the density radius to 60 (the
  classic arm); the expert branches are carried in the code for the
  difficulty flip.
* The `NetMessage.SendTileSquare` sides are MP scope (the port runs the host
  path); the `PlayerLOS` gate on the mushroom-tree arm reads the port's
  player-LOS seam (constants-off when no player is near).
