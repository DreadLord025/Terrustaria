# 12a — W4-d102: worldgen passes, second slice (lane claim + range)

> Wave 4 · lane `task/W4-d102-worldgen-passes-2` @ base a725acb0 · claimed 2026-11-20
> Parent audit: `docs/research/12-worldgen.md` (the 52-pass unexecuted list).
> This file is the lane's **range declaration** (documented early per the task
> brief) and its per-pass landing ledger.

## The ordering rule

"Unexecuted passes in C# order" = the classic (`gate == ""`) `VANILLA_CHAIN`
declarations in `pipeline.rs` that no `EXECUTED_PIPELINE` step cites as its
`vanilla` name, sorted by their `AddGenerationPass` anchor (`WorldGen.cs`).
Re-derived on this branch's base (the audit's 52-list predates S1-K-2 waves
A/B/C — `thin ice`, `mining explosives`, `mahogany trees` executed since; the
current diff is **51 names**, two of which are not real work: `Granite` is
folded into the executed `"Marble"` step and `Generate Ice Biome`'s slot is
partially covered by the executed `"Ice"` step — both stay on the list until
their own lanes reconcile them).

## Lane split (six lanes, nine passes each, C#-anchor order)

| lane | passes (C# anchor order) | anchors |
| --- | --- | --- |
| d101 (1st) | Sand Patches, Mount Caves, Dirt Wall Backgrounds, Clay, Small Holes, Dirt Layer Caves, Rock Layer Caves, Generate Ice Biome, Granite | 11689–12839 |
| **d102 (this lane, 2nd)** | **Silt, Webs, Slush, Mountain Caves, Gems, Shimmer, Clean Up Dirt, Dirt Rock Wall Runner, Wood Tree Walls** | **13143–15717** |
| d103 (3rd) | Settle Liquids, Oasis, Waterfalls, Wall Variety, Life Crystals, Statues, Surface Chests, Water Chests, Cave Walls | 16132–17711 |
| d104 (4th) | Floating Island Houses, Quick Cleanup, Hellforge, Spreading Grass, Surface Ore and Stone, Place Fallen Log, Traps, Piles, Spawn Point | 17859–19460 |
| d105 (5th) | Grass Wall, Guide, Sunflowers, Herbs, Dye Plants, Flowers, Mushrooms, Random Gems, Moss Grass | 19589–20769 |
| d106 (6th) | Larva, Settle Liquids Again, Tile Cleanup, Stalac, Remove Broken Traps, Final Cleanup | 20868–22141 |

## This lane's nine passes (C# bodies read in full)

| # | vanilla name | C# body | Rust seam (planned → landed) |
| --- | --- | --- | --- |
| 1 | Silt | `WorldGen.cs:13143-13181` — two TileRunner loops (kind 123), `rockLayerHigh..maxTilesY` (remix: `worldSurface..rockLayer`), wall-187/216 skip | `worldgen/passes2.rs::pass_silt` → step `"silt"` after `dirt to mud`, before `ores and gems` |
| 2 | Webs | `:13607-13644` — `0.0006·area` ceiling-hugging cobweb 51 runners, `mCaveX/Y` first-`numMCaves` override | `passes2.rs::pass_webs` → step `"webs"` after `ores and gems`, before `ash blobs` |
| 3 | Slush (`StoneToIceAndSiltPlusMudIntoSlush`) | `:14697-14752` — the snow-rect retype sweep (123→224, 59→224 gated on the 7×7 jungle-grass scan, 1→161) | `passes2.rs::pass_slush` → step `"slush"` after `liquids`, before `dungeon`. **Blocked producer:** `Generate Ice Biome` (d101) owns `GenVars.snowTop/Bottom/MinX/MaxX`; until it lands the sweep is a no-op on an empty rect (documented below). |
| 4 | Mountain Caves (`MountainCaveOpenings`) | `:14873-14887` — `CaveOpenater` (`:80085-80166`) + recursive `Cavinator` (`:79984-80083`) per `mCaveX/Y` anchor | `passes2.rs::pass_mountain_caves` → step `"mountain caves"` after `dungeon`/`constant lava layer`, before `beaches`. **Blocked producer:** `Mount Caves` (d101) owns `numMCaves/mCaveX/mCaveY`; the port adds the GenVars slots (reset 0 / `[0;30]`, `GenVars.cs:228-232`, `:457-458`) so the pass is a no-op until the producer lands. |
| 5 | Gems | `:15050-15137` — the six gem-kind TileRunner bands (63..68 densities), `tileSolid[484]=false`, the two-direction sand-edge slide | `passes2.rs::pass_gems` → step `"gems"` after `gravitating sand`, before `ocean caves` |
| 6 | Shimmer | `:15195-15245` + `ShimmerMakeBiome` `:34853-35057` + `ShimmerMakeBiomeOpening` `:35059-35102` | `worldgen/shimmer.rs` → step `"shimmer"` after `ocean caves`, before `pyramids`. Publishes `GenVars.shimmer_position` (`GenVars.cs:282`). |
| 7 | Clean Up Dirt (`DirtWallCleanup`) | `:15246-15359` — the two surface wall-scrub sweeps (the C#'s duplicated `==40` terms kept verbatim) | `passes2.rs::pass_clean_up_dirt` → step `"clean up dirt"` after `shimmer`, before `pyramids` |
| 8 | Dirt Rock Wall Runner | `:15461-15475` + `DirtyRockRunner` `:77145-77210` — wall-2→59 blobs | `passes2.rs::pass_dirt_rock_wall_runner` → step `"dirt rock wall runner"` after `pyramids`, before `pots` (the vanilla Pyramids `:15360` < Dirt Rock Wall Runner `:15461` < Living Trees `:15476` pair) |
| 9 | Wood Tree Walls (`LivingTreeWalls`) | `:15717-15749` — the 191-neighbour wall-244 sweep | `passes2.rs::pass_wood_tree_walls` → step `"wood tree walls"` after `living trees`, before `trees` (the vanilla Living Trees `:15476` < Wood Tree Walls `:15717` < Altars `:15750` order; the port's `altars` step sits earlier in its grouped chain) |

Every new step draws from its own `pass_rng("<vanilla name>")` stream (the
port's per-pass re-seed model, `pipeline.rs:212-225`), so no existing step's
draw ledger shifts; the world hash moves only by the new tiles/walls/liquid.

## Landing ledger (commit per pass, hash re-pin discipline)

Code-only lane (no cargo/tests on this branch): each pass lands with its
positional test + the `world_hash.rs` re-pin **note** (values deferred to the
merge gate, the S1-K-2-wave-B convention); the `a == b` determinism asserts
stay in place untouched.

| pass | commit | status |
| --- | --- | --- |
| range doc | f8c69c72 | claimed |
| Silt | c746a7da | landed (code-only; hash re-pin deferred to the gate) |
| Webs | 96e4b92f | landed (code-only; GenVars mCave slots added — producer d101; hash re-pin deferred) |
| Slush | d4fd0af0 | landed (code-only; GenVars snow slots added — producer d101, empty-rect no-op; pins unchanged by this pass) |
| Mountain Caves | 665041f8 | landed (code-only; no-op while numMCaves==0 — producer d101; pins unchanged by this pass) |
| Gems | 28ec5561 | landed (code-only; hash re-pin deferred to the gate) |
| Shimmer | 12154ee7 | landed (code-only; `GenVars.shimmer_position` published; SEED42_512 unchanged — sub-vanilla valve; DEFAULT_WORLD re-pin deferred) |
| Clean Up Dirt | 73e83ebe | landed (code-only; hash re-pin deferred to the gate; the Shimmer landing's missed step-count bump fixed here — 73) |
| Dirt Rock Wall Runner | 16aa0c64 | landed (code-only; hash re-pin deferred to the gate) |
| Wood Tree Walls | (this commit) | landed (code-only; draws nothing; hash re-pin deferred to the gate) — **lane complete 9/9** |
