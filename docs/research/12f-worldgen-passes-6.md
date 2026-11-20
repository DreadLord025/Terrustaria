# 12f — WorldGen passes, lane 6 (task/W4-d106-worldgen-passes-6)

> Wave 4 · 2026-11-20 · code-only mode (no cargo/tests on this branch) ·
> source: `docs/research/12-worldgen.md` (the 52-pass gap list) re-derived
> against the current `pipeline.rs` at base `a725acb0` (109 chain decls,
> 66 executed steps, 55 distinct vanilla names → 53 unmapped, of which
> Skyblock + the skyblock Jungle re-registration + Dual Dungeons Dither
> Snake are special-seed-gated (d107/c30 lanes), `Granite` is executed
> inside the `"granite and marble"` step (`step_granite_marble`,
> `biomes::pass_granite` on `pass_rng("Granite")`) and `Surface Caves` is
> executed (WORLDGEN-CAVES) → **50 classic unexecuted passes**).

## Range claim (documented early, before any port)

This lane takes the **REMAINDER after the d101–d105 ranges**. The sibling
claims (all @ base `a725acb0`, read from their pushed range docs /
dispatches):

- **d101** #1–9: Sand Patches :11689, Mount Caves :11784, Dirt Wall
  Backgrounds :11858, Clay :11959, Small Holes :12008, Dirt Layer Caves
  :12067, Rock Layer Caves :12107, Generate Ice Biome :12312, Silt :13143
  (d103's published lane-split doc, `8a856d94`, numbers d101 1–9 ending
  at Silt).
- **d102** #10–18: Webs :13607, Slush :14697, Mountain Caves :14873,
  Gems :15050, Shimmer :15195, Clean Up Dirt :15246, Dirt Rock Wall
  Runner :15461, Wood Tree Walls :15717, Settle Liquids :16132.
- **d103** #19–27 (pushed `8a856d94`): Oasis :16243, Waterfalls :16610,
  Wall Variety :16697, Life Crystals :16755, Statues :16853, Surface
  Chests :17130, Water Chests :17245, Cave Walls :17711, Floating Island
  Houses :17859.
- **d104** (pushed `2d9cfcbb`): Floating Island Houses :17859 (overlap
  with d103 — the two lanes/merger reconcile, NOT this lane), Quick
  Cleanup :17875, Hellforge :18187, Spreading Grass :18237, Surface Ore
  and Stone :18447, Place Fallen Log :18513, Traps :18657, Piles :18788,
  Spawn Point :19460. d104's doc explicitly hands **Grass Wall :19589 to
  lane 5** ("#37 Grass Wall :19589 (lane 5)").
- **d105** (its own dispatch statement): Guide :19685, Sunflowers :19898,
  Herbs :19964, Dye Plants :19981, Flowers :20447, Mushrooms :20599,
  Random Gems :20729, Moss Grass :20769, Larva :20868 — it starts at
  Guide, so it does NOT take Grass Wall either.

**Grass Wall :19589 is orphaned** between d104's and d105's actual claims
(d104 says lane 5, d105 starts at Guide). The remainder after the union
of d101–d105's claimed ranges is therefore **6 passes**:

| # | Pass (`GenPassNameID`) | C# anchor | Rust home (planned) |
| --- | --- | --- | --- |
| R1 | Grass Wall | `WorldGen.cs:19589` | `surface.rs` / new arm |
| R2 | Settle Liquids Again | `WorldGen.cs:21246` | `liquids.rs` (rides d102's Settle Liquids machinery; stand-in documented until it lands) |
| R3 | Tile Cleanup | `WorldGen.cs:21520` | `micro_finish.rs` / new arm |
| R4 | Stalac | `WorldGen.cs:22038` | `spider_gem_moss.rs` / new arm |
| R5 | Remove Broken Traps | `WorldGen.cs:22121` | new arm |
| R6 | Final Cleanup | `WorldGen.cs:22141` | new arm |

If a sibling's final landed range differs from the claims above, the
merger reconciles the overlap; this lane's six passes are disjoint from
every sibling's *stated* range except the Grass Wall orphan it adopts.

## Discipline

- Verbatim ports (golden rule): same structure, maths, draw order; citation
  comment per block; readable Rust names; C# read in ≤120-line windows.
- Each pass gets its `EXECUTED_PIPELINE` row at the vanilla slot + a
  positional witness; per-pass `pass_rng("<vanilla name>")` stream so the
  insertion shifts no other step's ledger (the genRand current convention).
- **Hash re-pins are DEFERRED to the merge gate** (code-only mode — no
  compiler on this branch, same discipline as S1-K-2 wave B / S1-P-i33):
  `world_hash.rs` gains a per-pass stale-by-design note; the `a == b`
  determinism halves of both pin tests stay untouched and must stay green
  at the gate.
- Commit + push per pass; docs + `anchors.txt` row per pass.

## Status

- [x] Range claim documented (this doc)
- [x] R1 Grass Wall — `surface.rs::grass_wall` + `spread_wall`/`count_dirt_tiles`/`spread_dirt_grass`, `pipeline.rs::step_grass_wall` (between `living trees` and `trees`), `pass_rng("Grass Wall")`; witnesses written NOT run; hash pins expected unchanged (wall-0 grass tops), re-pin deferred
- [x] R2 Settle Liquids Again — `liquids.rs::settle_liquids_again` + the shared `quick_water`/`settle_liquids_body`/`liquid_interactions_cleanup`/`shimmer_remove_water` machinery, `liquid.rs::settle_water_at`/`water_check`/`attempt_to_move` + the `quick_settle`/`gen_ignore_water`/`generating` toggles, `tileobject.rs::get_tile_data_for_tile`/`check_lava_death`/`check_water_death`, `tile.rs::any_shimmer`/`left_slope`/`right_slope`, `pipeline.rs::step_settle_liquids_again` (before `cacti`), `GenVars.shimmer_position`; draws no RNG; witnesses written NOT run; pins move by design (liquid settles), re-pin deferred
- [ ] R3 Tile Cleanup
- [ ] R4 Stalac
- [ ] R5 Remove Broken Traps
- [ ] R6 Final Cleanup
