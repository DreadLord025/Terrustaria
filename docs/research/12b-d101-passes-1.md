# 12b — W4-d101: worldgen passes, first slice (lane claim + census)

> Wave 4 · lane `task/W4-d101-worldgen-passes-1` @ base a725acb0 · claimed 2026-11-20
> Parent audit: `docs/research/12-worldgen.md` (the 52-pass unexecuted list).
> Sibling lanes: `12a-d102-passes-2.md` (d102, #10-18); d103-d106 carry #19-27,
> #28-36, #37-45, #46-51 respectively.

## The census (re-derived on this branch's base)

The ordering rule (shared with d102's `12a` doc): the classic (`gate == ""`)
`VANILLA_CHAIN` declarations in `pipeline.rs` that no `EXECUTED_PIPELINE` step
cites as its `vanilla` name, sorted by their `AddGenerationPass` anchor
(`WorldGen.cs`). Scripted diff on a725acb0: **109 chain declarations, 66
executed steps, 51 unexecuted classic passes** — the audit's 52-list predates
S1-K-2 waves A/B/C (`thin ice`, `mining explosives`, `mahogany trees` executed
since). Two of the 51 are not real work: `Granite` is folded into the executed
`"granite and marble"` step (this lane gives it its own executed row), and
`Generate Ice Biome`'s slot is only partially covered by the late `"Ice"`
(FragileIce, `:16667`) step — the actual biome generator `:12312` is missing
and lands in this lane.

## Lane split (six lanes, C#-anchor order — the d102 `12a` table, verified)

| lane | passes (C# anchor order) | anchors |
| --- | --- | --- |
| **d101 (this lane, 1st)** | **Sand Patches, Mount Caves, Dirt Wall Backgrounds, Clay, Small Holes, Dirt Layer Caves, Rock Layer Caves, Generate Ice Biome, Granite** | **11689–12839** |
| d102 (2nd) | Silt, Webs, Slush, Mountain Caves, Gems, Shimmer, Clean Up Dirt, Dirt Rock Wall Runner, Wood Tree Walls | 13143–15717 |
| d103 (3rd) | Settle Liquids, Oasis, Waterfalls, Wall Variety, Life Crystals, Statues, Surface Chests, Water Chests, Cave Walls | 16132–17711 |
| d104 (4th) | Floating Island Houses, Quick Cleanup, Hellforge, Spreading Grass, Surface Ore and Stone, Place Fallen Log, Traps, Piles, Spawn Point | 17859–19460 |
| d105 (5th) | Grass Wall, Guide, Sunflowers, Herbs, Dye Plants, Flowers, Mushrooms, Random Gems, Moss Grass | 19589–20769 |
| d106 (6th) | Larva, Settle Liquids Again, Tile Cleanup, Stalac, Remove Broken Traps, Final Cleanup | 20868–22141 |

In-flight check at claim time: d102 claimed #10-18 (f8c69c72), d103 claimed
#19-27 (8a856d94, Oasis landed 90b7eee6), d104 claimed #28-36 (2d9cfcbb), d106
claimed the remainder (89a96c19); d105 unclaimed. No lane touches #1-9.

## This lane's nine passes (C# bodies read in full)

| # | vanilla name | C# body | Rust seam (planned → landed) |
| --- | --- | --- | --- |
| 1 | Sand Patches | `WorldGen.cs:11689-11717` — `maxTilesX * 0.013` sand-53 TileRunner blobs in `[worldSurface, rockLayer)`, the spawn-middle rejection loop | `worldgen/passes1.rs::pass_sand_patches` → step `"sand patches"` after `ocean sand`, before `cave fields`/Tunnels (`:11718`) |
| 2 | Mount Caves | `:11784-11857` + `Mountinater` `:79486-79560` — `maxTilesX * 0.001` dirt-mountain raisers, the ±90 middle exclusion + the <100 spacing gate, the sand/sandstone/gemstone proximity scan; publishes `GenVars.numMCaves/mCaveX/mCaveY` (`GenVars.cs:228-232`, `[0;30]` `:457-458`) | `passes1.rs::pass_mount_caves` → step `"mount caves"` after `cave fields`, before `stone-in-dirt blobs`/Rocks In Dirt (`:11896`). **Consumed by d102's Mountain Caves (`:14873`) + Webs (`:13607`).** |
| 3 | Dirt Wall Backgrounds | `:11858-11895` — the per-column dirt-2/snow-40 wall sweep down to `worldSurface + Next(-1,2)` (clamped 0..10), the 2×2-solid `flag` latch, the wall-64 (jungle) skip | `passes1.rs::pass_dirt_wall_backgrounds` → step `"dirt wall backgrounds"` after `mount caves`, before `stone-in-dirt blobs` |
| 4 | Clay | `:11959-12007` — three clay-40 TileRunner bands (`2E-05` above surface-low, `5E-05` surface band, `2E-05` surface-high→rock-high) + the surface clay→dirt tidy sweep | `passes1.rs::pass_clay` → step `"clay"` after `dirt-in-stone blobs`, before `small holes` |
| 5 | Small Holes | `:12008-12066` — `area * 0.0015` paired diggers (`Next(5)==0` → type -2 water fill), the beach-avoidance (`smallHolesBeachAvoidance = 340`, `:11204`) + spawn-middle rejection loops, the small (2..5, 2..20) and large (8..15, 7..30) runners | `passes1.rs::pass_small_holes` → step `"small holes"` after `clay`, before `dirt layer caves` |
| 6 | Dirt Layer Caves | `:12067-12106` — `area * 3E-05` big dirt-band worms (5..15, 30..200), `Next(6)==0` water, the same rejection gates (middle inclusive `>=`/`<=` here) | `passes1.rs::pass_dirt_layer_caves` → step `"dirt layer caves"` after `small holes`, before `rock layer caves` |
| 7 | Rock Layer Caves | `:12107-12161` — `area * 0.00013` deep worms (`Next(10)==0` water, 6..20, 50..300) below `rockLayerHigh`; the remix second loop deferred with the remix seeds | `passes1.rs::pass_rock_layer_caves` → step `"rock layer caves"` after `dirt layer caves`, before `surface caves` (`:12162`) |
| 8 | Generate Ice Biome | `:12312-12422` — the snow column walk from `snowOriginLeft/Right` (Reset draws `:11401-11414`), the per-row averaged drift, the dungeon-side bias draw, the solid retype (0/2/23/40/53→147, 1→161, wall 2→40) down to `lavaLine - Next(160,200)`, the fringe drip below it; publishes `GenVars.snowTop/snowBottom/snowMinX/snowMaxX` (`GenVars.cs:78-88`) | `worldgen/ice_biome.rs::pass_ice_biome` → step `"generate ice biome"` after `wavy caves` (`:12271`), before `micro grass`/Grass (`:12423`). **Consumed by d102's Slush (`:14697`).** |
| 9 | Granite | `:12839-12926` — already ported (`biomes.rs::pass_granite`, the `GraniteBiome` pressure sim) and executed inside the `"granite and marble"` step on `pass_rng("Granite")`; this lane splits it into its own executed row (`vanilla: "Granite"`) so the census closes without a single new draw | step `"granite"` after `"granite and marble"` (renamed `"marble"`, `vanilla: "Marble"`) — same call order, same streams |

Every new step draws from its own `pass_rng("<vanilla name>")` stream (the
port's per-pass re-seed model, `pipeline.rs:212-225`), so no existing step's
draw ledger shifts; the world hash moves only by the new tiles/walls/liquid.

Documented deviations (classic-world faithful, recorded per pass):
- The `Skyblock.denyAllGeneration` / `noSurface` / `surfaceIsDesert` outer gates
  are the deferred seed arms (the port has no skyblock runtime gate yet); a
  classic world always enters the body.
- The remix / tenth-anniversary count multipliers and bands port their classic
  arms; the remix-only second loops stay with the remix-seed lane (d107).
- `Generate Ice Biome`'s `snowOriginLeft/Right` are the Reset draws
  (`:11378-11414`): `num7 = Next(maxTilesX)` rejected into the dungeon-opposite
  band (the port pins dungeon Right, so `[0.6, 0.75]·width`), then the
  `Next(50,90)` + two `(int)(Next(20,40) · width/4200)` terms per side. They
  append to `reset_genvars`'s fresh `pass_rng("Reset")` stream AFTER the
  `crimson_left` draw — every existing consumer re-copies the stream and reads
  only the first draw, so no ledger shifts.

## Landing ledger (ONE pass per commit+push, hash re-pin discipline)

Code-only lane (no cargo/tests/graphify on this branch): each pass lands with
its positional test + the `world_hash.rs` re-pin **note** (values deferred to
the merge gate, the S1-K-2-wave-B convention); the `a == b` determinism asserts
stay in place untouched.

| pass | commit | status |
| --- | --- | --- |
| census + range doc | (this commit) | claimed |
| Sand Patches | (next commit) | landed (code-only; hash re-pin deferred to gate) |
| Mount Caves | (this commit) | landed (code-only; 512-pin unchanged, default re-pin deferred) |
| Dirt Wall Backgrounds | — | pending |
| Clay | — | pending |
| Small Holes | — | pending |
| Dirt Layer Caves | — | pending |
| Rock Layer Caves | — | pending |
| Generate Ice Biome | — | pending |
| Granite (own row) | — | pending |
