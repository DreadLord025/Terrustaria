# 88 — Worldgen master keep-one matrix (the merger playbook)

> Wave 4 reconcile · produced on `task/W4-g36-worldgen-missing-a` @ 7f848edd ·
> 2026-11-21 · code-only (no cargo/tests run; all evidence is commit-message,
> diff-stat and source-read based). Parent census: `docs/research/12-worldgen.md`
> (109 `VANILLA_CHAIN` decls = 106 classic + Skyblock + the dead skyblock-Jungle
> re-registration + Dual Dungeons Dither Snake; 66 executed steps at base
> a725acb0 citing 55 distinct vanilla names).
>
> **Gap-fill re-verification (2026-11-21, this phase):** all pushed tips re-read
> (d105 advanced to `241ffa39`, f25 to `803093b1`) + the local g37 claim
> (`4ced188f9`→`81fd1e0d`). The three gaps flagged at `6d46f6a5` (Random Gems,
> Moss Grass, Final Cleanup) are now ALL filled/claimed elsewhere — §6.1
> rewritten. New residual: g37's Final Cleanup deferred the **ShimmerCleanUp**
> arm (`:22472`) back to g36 (it reads g36 A1's `shimmer_position`); PORTED this
> phase (`shimmer.rs::shimmer_clean_up`, unwired, W:7, a==b). Larva contest
> re-adjudicated (§2.20): d105 #45 landed a slice, c31 stays the faithful owner.
>
> **This doc IS the merger playbook.** Every vanilla pass gets exactly ONE
> owner branch below; every duplicate gets an explicit KEEP (with arm-count /
> witness-count evidence) and DROP list.

## 1. Pushed tips read (all worldgen lanes, fetched 2026-11-21)

| lane | tip | role |
| --- | --- | --- |
| d101 | `4194acf8` | passes 1–9 (Sand Patches..Generate Ice Biome) + Granite row-split |
| d102 | `29e8859b` | Silt/Webs/Slush/Mountain Caves/Gems + Shimmer/Clean Up Dirt/Dirt Runner/Wood Walls (9/9) |
| d103 | `8be04cb0` | Oasis/Waterfalls/Wall Variety/Life Crystals/Statues/Surface Chests/Water Chests/Cave Walls/Island Houses (9/9) |
| d104 | `8a229c3e` | Island Houses/Quick Cleanup/Hellforge/Spreading Grass/Surface Ore/Fallen Log/Traps/Piles/Spawn Point (9/9) |
| d105 | `241ffa39` | Guide/Sunflowers/Herbs/Dye Plants/Flowers/Mushrooms/Random Gems/Moss Grass/Larva (9/9 landed — #43 `9f7b1a24`, #44 `d590d99a`, #45 `22a0026ce`; LANE 5 COMPLETE) |
| d106 | `e2c2609c` | Grass Wall/Settle Liquids Again/Tile Cleanup/Stalac/Remove Broken Traps (R1–R5; R6 Final Cleanup yielded to g37's gap-fill `4ced188f9`) |
| f24 | `9f2276d5` | the two GATED passes: Dither Snake (B1–B6) + Skyblock (A1–A4), review-fix round included |
| f25 | `0ed1f3cf` | Settle Liquids (orphan adoption fd098528, earlier claim than g36's) |
| g36 | `6d46f6a5` + gap-fill | A1 Shimmer, A2 Clean Up Dirt, A3 Dirt Runner, A4 Wood Walls, A6 Wall Variety, A7 Life Crystals (A5 yielded to f25); gap-fill: ShimmerCleanUp arm (g37's Final Cleanup deferral, `shimmer.rs::shimmer_clean_up`, unwired) |
| g37 | `81fd1e0d` (local; pushed tip `53a5b5b5`) | B1–B9 + FINAL CLEANUP gap-fill (`4ced188f9`→`81fd1e0d`, claim-probed against g36 tip `6d46f6a5` row 109 GAP); ShimmerCleanUp arm deferred back to g36 |
| g38 | `8c9b7b60` | C1 Place Fallen Log only (rest of c-quarter claimed, not landed) |
| g39 | `bab25558` | H1–H15 PlaceTile-wrapper helper family (`place_helpers.rs`, unwired) |
| c29 | `d95c32cf` | DeadMansChestBiome + the Micro Biomes dead-man's-chest block |
| c30 | `8f5f1086` | dual-dungeons dither snake (whole family, single file) |
| c31 | `d538536b` | faithful Hive rewrite + Larva step + larva-break→Queen Bee seam |
| c33 | `321c9b66` | faithful Pyramids + UnderwaterChests (Water Chests) + lily/cattail/bamboo arms |
| g40 | `79361f00` | Marble/Granite residual fidelity (SlabStates, SmoothSlope, PlaceTight seams, ore-list fix) |
| g41 | `ee937411` | spider-cave egg-sac validator chain (CheckStalactite family) + stalactite arms + webs/DST residuals |
| g42 | `d4836bbd` | mushroom-biome residuals (GlowingMushroomPatches deltas, GrowTree rewrite, DoSurfaceIsMushrooms, Convert-3) |
| g44 | `a6412835` | verbatim FloatingIslands + FloatingIslandHouses/IslandHouse + skyLakes |

Note: d105/d106 tips moved past the hashes quoted in the dispatch brief
(dd43734a / 29faa099); this matrix uses the **current** tips above. d107/d108
(special-seed arms / temple paint) and c32 (dungeon style tables) carry no
classic-pass rows and are out of the keep-one scope except where noted.

**Evidence convention.** `W:n` = `#[test]` count in the owning file(s) at the
tip (witnesses are written-not-run on every code-only lane; the merge gate runs
them). `arms` = the C# conditional arms the commit messages claim ported.
KEEP picks the most-faithful/complete version: arm coverage first, witness
count second, vanilla-slot correctness third. A KEEP with fewer witnesses but
more arms says so explicitly.

## 2. The keep-one matrix (every contested pass)

### 2.1 Shimmer (`WorldGen.cs:15195` + `ShimmerMakeBiome :34853` + opening `:35059`)

| owner | evidence |
| --- | --- |
| d102 `12154ee7` | `shimmer.rs` 504 lines, W:5 (biome replay, sub-vanilla valve, skyblock skip, evil-box veto, out-of-world veto); stone shelf + PlaceTight, 500-roll gem tail via the shared growth module |
| g36 A1 `702e1e75` | `shimmer.rs` 866 lines, W:4 (veto scan, carve+replay, pass termination+replay, opening budget); **more arms**: num==0 island arm with PlaceTight + a local CheckStalactite/UpdateStalagtiteStyle port, gem speckle on a local GrowTreeWithSettings with GemTreeWallTest + nactive side-ground fidelity repairs, `SHIMMER_SAFETY_DISTANCE` pub const + the 200×200 protected rect, GenVars shimmerPosition |

**KEEP g36 A1** (`702e1e75`, review-hardened through `7f848edd`). Arm coverage
decides it: the gem-tree wall test, the stalactite validator and the protected
rect are load-bearing for downstream Wall Variety / Traps shimmer vetoes.
**DROP d102 `12154ee7`** (`shimmer.rs` + its pipeline row). Merger notes:
port d102's skyblock-seed-skip witness onto g36's body if absent; g36's local
stalactite helpers should be rewired onto the canonical chain (§3.2) unless the
draw schedule differs (verify at the gate); the GenVars `shimmer_position`
slot is added by d102/d104/d106/f25 too — keep ONE field (§3.4).

### 2.2 Clean Up Dirt (`:15246` + `DoNoSurfaceFillTheTop :1055`)

| owner | evidence |
| --- | --- |
| d102 `73e83ebe` | `passes2.rs`, W:4 (sky-wall scrub+replay, sand flag+neighbours, noSurface fill+replay, **dungeon-tile exemption**); review commit raised the test canvas to 120 tall + corrected the first-active-row assert to the C# centre scrub; fixed Shimmer's missed step-count bump |
| g36 A2 `7260a9aa` | `micro_finish.rs`, W:3 (sweeps, sand flag+replay, noSurface arm); same verbatim duplicated `==40` disjunct + 5-row re-arm |

**KEEP d102 `73e83ebe`** (4 witnesses incl. the dungeon exemption; the
review round corrected a fixture the g36 version does not carry).
**DROP g36 A2 `7260a9aa`** (the `pass_clean_up_dirt` + `no_surface_fill_the_top`
+ `clean_dirt_column` + `dirt_wall_neighbor` block in `micro_finish.rs`).

### 2.3 Dirt Rock Wall Runner (`:15461` + `DirtyRockRunner :77145`)

| owner | evidence |
| --- | --- |
| d102 `16aa0c64` | `passes2.rs`, W:3 (wall-2→59 blobs+replay, non-dirt candidate draws-but-does-not-run, **sub-vanilla valve before any draw**); the per-cell jitter draw evaluated before the wall `&&` test, the full-strength threshold quirk |
| g36 A3 `7005940a` | `micro_finish.rs`, W:2 (pocket conversion, wall-2-only sites); same walker quirks cited |

**KEEP d102 `16aa0c64`** (3 witnesses incl. the draw-order and valve proofs).
**DROP g36 A3 `7005940a`** (`pass_dirt_rock_wall_runner` + `dirty_rock_runner`).

### 2.4 Wood Tree Walls (`LivingTreeWalls :15717`)

| owner | evidence |
| --- | --- |
| d102 `29e8859b` | `passes2.rs`, W:4 (trunk interior+replay, missing-diagonal veto, existing-244 corner satisfies flag, 25-margin/surface-line bound); the no-inner-break kept, RNG-free |
| g36 A4 `aa3ee000` | `living_trees.rs`, W:2 (four-corner gate, corner-count + RNG-free) |

**KEEP d102 `29e8859b`** (4 witnesses vs 2; same verbatim body).
**DROP g36 A4 `aa3ee000`** (`wood_tree_walls` in `living_trees.rs`).

### 2.5 Wall Variety (`:16697` + ShapeFloodFill/OuterOutline framework)

| owner | evidence |
| --- | --- |
| d103 `2005402d` | `wall_variety.rs` 415 lines, W:6 (dirt unsafe family, determinism, size guard, **flood-fill mushroom rejection**, outline unsafe-wall skip + one-row shift, jungle unsafe family); framework inline verbatim; shimmer re-draw at the **zero-init** shimmerPosition |
| g36 A6 `8486571f`+`7f848edd` | `micro_finish.rs`, W:3 (flood-fill invalid-touch flag, outline rim+protected walls, pass replay; review strengthened to 30×30 caverns within the 1000-cell cap); shimmer-safety re-roll **reads A1's published centre** |

**KEEP d103 `2005402d`** (6 witnesses vs 3, both biome families covered).
**DROP g36 A6** (`pass_wall_variety` + `flood_fill_wall_variety` +
`place_wall_variety_outline` + `random_world_point`/`is_not_solid`/`is_touching`
if unused elsewhere). **Merger must rewire** d103's zero-init shimmerPosition
read onto g36 A1's published `GenVars.shimmer_position` + safety rect (§2.1) —
otherwise the shimmer-safety re-roll is dead on the merged chain.

### 2.6 Life Crystals (`:16755` + AddLifeCrystal `:31912` + AddManaCrystal `:31981`)

| owner | evidence |
| --- | --- |
| d103 `d1e0d0d9` | `crystals.rs` 269 lines, W:7 (2×2 seat, lava veto, dungeon-wall veto, sloped-support flatten, determinism, size guard, mana 639 family); anniversary ×1.2 both loops, remix y-redraw, 10000 budget, errorWorld/noSurface mana arm; skyblock gated at the adapter (denySomeGeneration) |
| g36 A7 `f3930740`+`87170af3` | `structures.rs`, W:5 (heart seat, lava/dungeon refuse, pass replay, mana stars, collapsed-band guard); same arms + skyblock-10 attempt cap; seed arms documented-deferred |

**KEEP d103 `d1e0d0d9`** (7 witnesses vs 5; the sloped-support flatten witness
is unique). **DROP g36 A7** (`pass_life_crystals` + `add_crystal` +
`is_tile_nearby`/`any_lava`/`any_shimmer` locals in `structures.rs` if unused).
The g36 skyblock-10 retry cap is moot under the adapter gate (d103's own note).

### 2.7 Statues (`:16853` + SetupStatueList `:4449` + PlaceStatueTrap `:4490`)

| owner | evidence |
| --- | --- |
| d103 `bec7d67a` | `statues.rs` 424 lines, W:7; 73-entry cycle with the **jug swap**, style/55 sheet offset, **remix redraw consumed-and-discarded**, oceanDepths budget-100, frameImportant/tileCut/shimmer vetoes, **the 488 peace veto**, {4,7,10,18} trap plates + WireLine, 105/349/135 boot statics |
| g37 B1 `a0bf6174` | `passes_b.rs`, W:1 (pipeline positional); same cycle/traps but remix extra draw deferred, no peace veto; SEED42_512-unchanged beach-band proof |

**KEEP d103 `bec7d67a`** (7 witnesses, two extra arms). **DROP g37 B1**.
Carry g37's 512-wide beach-band rejection proof into the gate's hash notes.
The pass-local Place1x1/Place2xX slices in both versions are superseded by
g39's canonical helpers (§3.1).

### 2.8 Surface Chests (`:17130`) + the AddBuriedChest core (`:35877-37568`)

| owner | evidence |
| --- | --- |
| d103 `f3929904` | `buried_chest.rs` 655 lines, W:7 — the **full** placement core: down-scan floor gate, shimmer/231 aborts, trySlope flatten/restore + style-17 exclusion, remix centre-bottom abort, 5×5 boulder fluff-100, flag15 default + family dispatch, **errorWorld/noTraps placement draws**, PlaceChest boulder veto + NearOtherChests, wall-2/59/244 gate + ±50 living-tree search, **hell-chest Reset shuffle + desertHiveHigh/Low** |
| g37 B2 `1cf9f0f1`(+`809a61a8` slot fix) | `passes_b.rs`, W:1; a smaller `add_buried_chest` placement half (family arms kept but no Reset shuffle / errorWorld draws); stocking rides chest_stock |

**KEEP d103 `f3929904`** (the core is the shared dependency of Water Chests /
Island Houses / Pyramids — the most complete version wins for all of them).
**DROP g37 B2** (pass + its local core). Merger: re-point c33's Pyramids and
g44's IslandHouse chest calls onto d103's `buried_chest.rs` core (c33
generalised its own half from temple; g44 uses the sky-band slice).

### 2.9 Water Chests (`UnderwaterChests :17245`) — THREE-way

| owner | evidence |
| --- | --- |
| d103 `9a935127` | `water_chests.rs` 253 lines, W:3; NextFromList main + widening box + x shift, liquid>250, style-17 trySlope; 9·width/4200 scatter with the 186/4404/277/187 cycle, **1/10 anniversary-1/7 863 override**, the two unbounded water re-draw loops with the 1→50 y-floor quirk + **arm-2 no-remix gate**, beach-margin trySlope, **extraLiquid arms** |
| g37 B3 `4c58c56c` | `passes_b.rs`, W:1; same skeleton, 10000-cap degenerate-world guard, seed arms deferred |
| c33 `321c9b66` | `structures.rs::pass_underwater_chests`, W:1; 1/50 first-draw asymmetry kept, **flag5 4425/4460 stocking pair + style-17 main join the stocking hook**, pyramid probe gains wall 34 |

**KEEP d103 `9a935127`** (most arms: anniversary override + no-remix gate +
extraLiquid). **DROP g37 B3 and c33's `pass_underwater_chests`**. Merger: carry
c33's stocking-hook additions (4425/4460 pair, style-17 main) onto the kept
core's chest_stock seam, and g37's 10000-cap guard if d103's loops are truly
unbounded (hang risk on waterless worlds). **c33's Pyramids stay** (§2.16).

### 2.10 Cave Walls (`CaveWallsInEnclosedSpaces :17711` + Spread.Wall/Wall2)

| owner | evidence |
| --- | --- |
| d103 `463cab95`(+`8be04cb0` fixture fix) | `cave_walls.rs` 463 lines, W:7; three sweeps verbatim incl. the **remix clean-pocket dirt/grass arm**, jungle-wall→mud Wall2 with wallHouse/244 re-draw gate, 162/226/232 solid window, maxWallOut2 5000 budget, protected/stops-at-air sets; fixture corrected to the baseline-wall convention |
| g37 B4 `ef7c9f76` | `passes_b.rs`, W:1; 4% + 2% loops, remix second loop **deferred**, tileSolid toggles unmodelled |

**KEEP d103 `463cab95`** (7 witnesses + the remix arm + the reviewed fixture).
**DROP g37 B4**.

### 2.11 Floating Island Houses (`:17859` + IslandHouse `:79203`) — FOUR-way

| owner | evidence |
| --- | --- |
| d103 `eb018138`(+`85f85683` borrow fix) | `island_houses.rs` 559 lines, W:4; ledger recorded by the islands pass, build deferred to the vanilla slot; wall veto + **drunk crimson split**, sunplate-202/wall-82, **case-10 door dispatch**, chest cycle, fountain + SwitchFountain, windows/table/chairs/banners, desert palms |
| d104 `93b1938e` | `island_houses.rs` 580 lines, W:2; faithful house at a **non-vanilla slot** (between living trees and trees), sky-band chest slice, denyFloatingIslands gate |
| g37 B5 `3d74564e` | `passes_b.rs` + `islands.rs`, W:2; faithful two-pass split (sites ledger → build), style-9 door / style-13 chest, classic islandStyle 0 (fountain/palm no-ops), remix/drunk/getGood variants deferred |
| g44 `ae4472e6`(+`a6412835` ledger fix) | `islands.rs` 1143 lines, W:10; **verbatim BOTH passes**: FloatingIslands count 0.0008·width + **skyLakes tail lakes**, errorWorld triple, spawn re-draw 150/100, anchor spacing, column-scan y-band + surfaceLow clamp, num3·2 cap, GenVars ledger; houses sunplate-202/disc-wall-82 + door/chest-cycle/windows/table/chairs/banners; the disc-top-row ledger fix composes the j-15 ground scan with the grass cap |

**KEEP g44** (`ae4472e6` + `a6412835`) — it owns **both** Floating Islands
(#26) and Floating Island Houses (#73) with 10 witnesses and the skyLakes /
errorWorld / spawn-re-draw arms none of the other three port.
**DROP d103 `eb018138`, d104 `93b1938e`, g37 B5 `3d74564e`** (all three
`island_houses.rs`/`passes_b.rs` house bodies + their islands.rs ledger
edits). Merger: verify g44 carries d103's drunk-crimson wall split + case-10
door dispatch (if absent, port those two arms onto g44's body at the gate);
re-point g44's chest call onto d103's buried_chest core (§2.8).

### 2.12 Quick Cleanup (`:17875`)

| owner | evidence |
| --- | --- |
| d104 `df067118` | `micro_finish.rs`, W:2 (sweep replay+tidy, ocean lava→water); **tileSolid[137]/[130] false window modelled on the predicate**, 225 half-brick exemption + 137-trap veto, dualDungeons-gated liquid arms, skyblock denyAllGeneration gate, save_slopes shared |
| g37 B6 `b48a702b` | `passes_b.rs`, W:1; same sweeps but tileSolid toggles **unmodelled**, notTheBees arm deferred |

**KEEP d104 `df067118`** (the tileSolid window changes which tiles the clear
family removes — behavioural fidelity). **DROP g37 B6**.

### 2.13 Hellforge (`:18187`)

| owner | evidence |
| --- | --- |
| d104 `2ac7c298` | `structures.rs`, W:1 (places+replays); maxTilesX/200, wall-13/14 gate, drop scan bounded height-20, PlaceTile 77 + type-77 success check, 10000 valve, **wall-miss arm redraws without bumping the count**, skyblock denyAllGeneration gate |
| g37 B7 `454e43ba` | `passes_b.rs`, W:1; same body; the catch-arm quirk documented as a deviation (dense array never throws), skyblock deferred |

**KEEP d104 `2ac7c298`** (tie on witnesses; d104 ports the gate and states the
wall-miss no-bump arm as behaviour, not deviation; co-located with Piles in
structures.rs). **DROP g37 B7**.

### 2.14 Spreading Grass (`:18237`, classic arm)

| owner | evidence |
| --- | --- |
| d104 `3a101031` | `surface.rs`, W:2 (surface dirt takes grass, stone takes biome flavour); sweep 1 ±50 retypes with the sand-53 latch + evil cover veto + jungle-band 59/60 split, sweep 2 ±10 SpreadGrass(0,2,repeat:false) with flag2 latch + worldSurfaceHigh break; `spread_evil_grass` gains the C# repeat parameter; remix deferred to d107 |
| g37 B8 `df468d00` | `passes_b.rs`, W:1; 50-margin sweep + per-column spread with grassSpread<1000 recursion; remix deferred |

**KEEP d104 `3a101031`** (2 witnesses; the repeat-parameter plumbing is the
C# signature). **DROP g37 B8**. Both defer remix to d107 — unchanged.

### 2.15 Surface Ore and Stone (`:18447` + OrePatch `:10579` + StonePatch `:10242`)

| owner | evidence |
| --- | --- |
| d104 `19bc132b` | `surface.rs`, W:1 (patches place+replay); ore count Next(5w/4200,10w/4200), w/420 budget, beach band + [0.48,0.52] re-roll, ±200 spacing + maxOrePatch-1 into GenVars.ore_patch_x; OrePatch verbatim (tier roll, Next(2) sink, OreHelper, re-roll-per-check Next(8,13) walk, **1-in-8 double**, velocity clamps); StonePatch (199/23 veto, **186 Place3x2 + two 185 PlaceSmallPile decor**); `place_small_pile` gains the C# type parameter |
| g37 B9 `94c365ee` | `passes_b.rs`, W:1; same counts/spacing/ledger (num_ore_patch + ore_patch_x[50]), OrePatch/StonePatch + place_small_pile local |

**KEEP d104 `19bc132b`** (tie on witnesses; d104's StonePatch decor arm +
the type-parameter generalisation are the more complete C# shape, and the
body sits with the rest of d104's surface family). **DROP g37 B9**.
GenVars ore slots: keep one set (§3.4).

### 2.16 Pyramids (`:15360` + Pyramid `:27669`) — single faithful owner

c33 `524d94a7` replaces the base shape-level 480 hull: numPyr anchor loop
consuming Dunes/Ocean-Sand PyrX/PyrY, 300 margins, dungeon-side 15% exclusion,
220 spacing + drunk halving, 151 hull + wall-34 sweep, entrance shaft, zig-zag
tunnel + chamber (Pharaoh chest style 1, 185 piles, four 91 banners, 28 pot
row), post-chamber surface tunnel. W:3 (hull, drunk spacing, chamber
furniture). **KEEP c33** — no competitor. Its `add_buried_chest` half is
superseded by d103's full core (§2.8): re-point, keep the pass.

### 2.17 Place Fallen Log (`:18513`)

| owner | evidence |
| --- | --- |
| d104 `2fdc8b96`(+`341f4280` witness fixes) | `surface.rs`, W:1 (places+replays); width/2100+Next(-1,2), **30000/5000/2500/1000 four-tier budget ladder**, ±400 beach column + centre re-roll, drop scan, worldSurface-10 veto, ±50 cloud/sand/dungeon/evil scan, ±10 overhead clear, liquid gate, grass triplet + notTheBees/frozen/desert alternatives, 1-in-2 logX/logY anchor, denySomeGeneration + noSurface gates, skyblock /10 |
| g38 C1 `8c9b7b60` | `surface.rs`, W:3 (Place3x2 frames, existing-log veto, gates skip); 30000/5000 two-tier ladder, 100×100 neighbourhood + 20×9 headroom, logX/logY anchor; **slot after cave houses** (vanilla is after Surface Ore and Stone) |

**KEEP d104 `2fdc8b96`** — arm coverage (four-tier ladder, the ±50/±10 scans,
liquid gate) and the correct vanilla slot outweigh g38's extra witnesses.
**DROP g38 C1**. Merger: port g38's three witnesses (frame write, existing-log
veto, gate-skip) onto d104's body — they cover arms d104's single witness
does not assert.

### 2.18 Dual Dungeons Dither Snake (gated, `:14755`)

| owner | evidence |
| --- | --- |
| c30 `8f5f1086` | single `dither_snake.rs` 1295 lines, W:12 (10 module + 2 pipeline a==b); whole DitherSnake/DitherSnakePass/DungeonControlLine + dual style lists (paint quintuples only) + CreatePotentialDungeonBounds + Reset dual block; **registration gate rides all three driver seams** (apply_step, GenerationRunner.advance skip, total_weight); GenVars.dither_snakes seam for c32 |
| f24 B1–B6 `341b6643..993864e1` + fix `9f2276d5` | 5 modules (`snake_math` 237 / `dungeon_bounds` 279 / `dungeon_control_line` 469 / `dither_snake` 588 / `dungeon_genvars` 319), W:16 total (2+3+3+5+3); **full producer chain**: SetupDungeonGenVarVariables Next(3) colour + remix pin + drunk wall re-roll, side-split bounds, the dual style quads carrying the four c32-deferred statics (Cavern/Corruption/Jungle/Temple) + GetCurrentDungeonStyle; Dunes-pass dual setup draws; **review-fix round**: witness tangents match SetTangents, adjust_tangents persists the CacheNormals side-effect on the false arm |

**KEEP f24 B1–B6** (16 witnesses, the complete producer chain incl. the
c32-deferred style statics, and a review round that fixed two real fidelity
bugs). **DROP c30** (whole `dither_snake.rs` + its pipeline gate). **Merger
must verify** f24's wire covers c30's three driver seams (apply_step,
`GenerationRunner.advance` skip loop, `total_weight`) — if f24 only gates
`apply_step`, port c30's `pass_registration_gate_closed` seam coverage; keep
c30's `GenVars.dither_snakes` seam name if the c32 crawler consumes it.

### 2.19 Settle Liquids (`:16132`) — orphan, single owner

f25 `0ed1f3cf` (claim `fd098528` predates g36's; g36 formally yielded A5 in
`f474adc1`). `settle_liquids.rs` 583 lines, W:4; ocean retype, QuickWater(3)
pre-settle, WaterCheck re-registration, 10-round quickSettle, ClearPendingLiquid
tail, LiquidInteractionsCleanup (up-shimmer quirk) + ShimmerRemoveWater;
liquid.rs quick_settle/gen_ignore_water toggles; tileobject resolve_tile_def +
check_water/lava_death. **KEEP f25** — no competitor for the pass itself.
The **machinery** overlaps d106 R2 (§3.3).

### 2.20 Hives / Larva — single faithful owner

c31 `5de48d50..d538536b`: `hive.rs` 844 lines, W:10 (+2 kill +2 app);
HiveBiome.cs verbatim (Place gates, CreateHiveTunnel dither bands, honey-fall
cubes/dents, CreateStandForLarva larvaX/Y seam, drunk extra stand), the faithful
Beehives pass over shared srng, the **Larva executed step** (`:20868`, Place3x3
231 num=-2) at its vanilla slot, the larva-break→Queen Bee seam (gore 300/301/302
+ SpawnOnPlayer 222, never an item), the tileSolid[225] weight-order correction
(Hives slot 50 runs BEFORE the Life Crystals flip — window wrapper removed).
**KEEP c31** for Hives (#51) and Larva (#100). g40's PlaceTight hive-rising arm
composes with c31's dents. The retired shape-level dome stays retired.

**Re-adjudication (gap-fill re-verification, 2026-11-21):** d105 #45 `22a0026ce`
(pushed tip `241ffa39`) landed a Larva step AFTER this matrix's first pass —
`structures.rs::pass_larva` consumes a `GenVars.larva: Vec<(i32,i32)>` ledger its
`pass_beehives` publishes (the ~half-the-hives `Next(2)` anchor roll, the 100-cap
push), step after `muds walls in jungle`, skyblock denyAll gate, W:3. It is the
ledger-consumer slice over the RETIRED shape-level dome (`pass_beehives` still
draws the dome, not the faithful HiveBiome tunnels). c31's `hive.rs` is the
faithful owner: the full `HiveBiome.Place` (CanPlace gate, 15-disc IsSolid scan,
the Next(2,5) tunnel chains with the 0.4/0.75/0.6 dither bands, the honey-fall
cubes/dents, `CreateStandForLarva` at the chain end + the drunk 1000-try extra
stand `:86-99`), the `LarvaAnchors` 100-slot wrap-at-99 ledger (the C#
`GenVars.larvaX/Y` array, not a growable Vec), `place_larva_3x3` with the
Place3x3 clear+floor gates, W:10. **KEEP c31; DROP d105's Larva step +
`structures.rs::pass_larva` + the `pass_beehives` larva-ledger edits + the
`GenVars.larva` Vec slot at the merge** (c31's `LarvaAnchors` is the canonical
slot, §3.4). d105's skyblock denyAll gate on Larva is correct and c31's step
should carry it too (the C# `:20870` gate) — port the gate onto c31's step at the
merge if absent (c31's `step_larva` currently has no skyblock gate; the C#
`BeeLarvaInBeehives` is `!Skyblock.denyAllGeneration`-gated).

**FINAL adjudication (seal phase, 2026-11-21 — C# re-read in ≤100-line windows;
the d104c/86-§6.3 challenge answered):** the challenge was that this §2.20 DROP
might delete a site d104's research/86 §6.3 keeps ("at the merge both survive:
d105's classic ledger-driven pass + g41's flag-gated seed tail… the two routes
write different sites"). **The DROP stands — the two "both-survive" routes are
`:20868` vs `:25608-25629`, NOT c31 vs d105:**

1. **`:20868-20900` `BeeLarvaInBeehives` (the vanilla Larva slot, row 100)** —
   re-read verbatim: the `!Skyblock.denyAllGeneration` gate `:20870`, the
   `tileSolid[229]=true` `:20872` / `[232]/[162]` `:20897-20898` boot statics,
   the per-anchor 3×4 re-clear (`:20878-20894`, three upper rows deactivated,
   the floor row 225 flat) + `PlaceTile(num, num2, 231, mute: true)` `:20895`
   over `GenVars.larvaX/Y[0..numLarva]`. **c31 `hive.rs::pass_larva` +
   `place_larva_3x3` and d105 `structures.rs::pass_larva` + `place_larva` are
   TWO PORTS OF THIS SAME SITE.** One row, one owner: c31 (the faithful
   producer below). d105's slice DROPs — nothing is lost that c31 lacks.
2. **`:25608-25629` (the FinishNotTheBees conversion-tail larva rolls, INSIDE
   the Final Cleanup pass's `NotTheBees(); FinishNotTheBees()` call
   `:22435-22439`)** — re-read verbatim: the continue gate `:25608`
   (`n <= num || m <= 350 || m >= w-350 || (liquid != 0 && !honey) ||
   (anniversary && !remix)`), then `Next(600)==0` (DST‖remix) /
   `Next(150)==0` (noTraps) / `Next(225)==0` (else) → `PlaceTile(m, n, 231)`.
   **g41 `not_the_bees.rs::place_larva` (`78b0cd15`, called from the
   `finish_not_the_bees` per-cell walk) owns THIS site** — flag-gated, never a
   classic row, zero overlap with row 100. **KEEP g41's tail arm** (86 §6.3's
   "both survive" is exactly this pair: c31's classic pass + g41's seed tail;
   an NTB world fires BOTH, the vanilla behaviour — the Hives pass `:15929`
   gates only skyblock).
3. **The d105 producer is unfaithful, independently of the dome retirement** —
   its `pass_beehives` publishes the anchor behind `if rng.range(0, 2) == 0`
   ("~half the hives"). **No such roll exists in the C#:**
   `HiveBiome.cs:85` calls `CreateStandForLarva(vector2D)` UNCONDITIONALLY at
   every placed hive's chain end (the `Next(2)` at `:61` is the honey-fall
   direction pick; the drunk extra stand `:88-99` adds a second anchor).
   c31's `create_stand_for_larva(world, anchors, cx, cy)` runs at the chain
   end unconditionally (the `:85` citation) + the drunk 1000-try extra stand,
   with the `Utils.Clamp(5, max-5)` + wrap-at-99 ledger semantics
   (`HiveBiome.cs:359-365`, `GenVars.cs:446-447` `new int[100]`). d105's Vec
   drops anchors past 100; the C# overwrites slot 99. c31 wins on the
   producer AND the consumer; the contest is not close.
4. **Merge actions (unchanged from the re-adjudication, now FINAL):** DROP
   d105's `pass_larva`/`place_larva` + `step!("larva")` row + the
   `pass_beehives` larva-ledger edits + the `GenVars.larva` Vec; KEEP c31's
   `hive.rs` pair + `LarvaAnchors`; PORT d105's skyblock denyAll step gate
   onto c31's `step_larva` (the `:20870` gate — c31's step has none; d105's
   `skyblock_deny_all_generation(opts)` is the shape, canonical table f24
   §3.5); KEEP g41's NTB tail untouched. The 229/232/162 solidity writes are
   boot statics both ports correctly treat as no-ops (d105's note, verified
   `Main.cs:7145/:9612/:10220` by 86 §6.3).

Verdict ledger: 88 §2.20 (this doc) = c31; 86 §6.3 (d104c) = d105 — written
against d105 `96f65cd6` before it read c31's pushed `d538536b` hive rewrite
(its "g41 does not cover the slot" half is CORRECT and adopted above; its
"d105 does" half is superseded: d105 covers the slot too, but less
faithfully). 75 §8.2/§8.3 carried no Larva verdict. **FINAL: row 100 = c31.**

### 2.21 Micro Biomes dead-man's-chest block (`:20916`) — single owner

c29 `dbc93e9b`(+`d95c32cf` docs): `mini_biomes.rs` +906 lines, W:8 new (29 in
file); scan→roll→pick order, accumulating-band quirk, 467 style-4 retype +
Next(3) sweater DeepClone shift, 137/138/141 traps + red wire, Place2x2
nactive()/tileTable arms, Collision.EmptyTile origin gate. **KEEP c29** — no
competitor. Its step leads the micro group (pipeline count 67 on its branch).

### 2.22 Marble / Granite residuals — complementary, both keep

d101 `4194acf8` gives Granite its own executed row (RNG-neutral split, census
51→42). g40 `2f552d6e..79361f00` ports the residual fidelity INTO the bodies
(SlabStates delegates, SmoothSlope masks, PlaceSlab extent seeds, ResetToType,
drunk 368/180 swap, Tile.SmoothSlope cascade, the Sets.Ore literal fix,
GraniteBiome pressure sim + PlaceDecorations FastRandom ledger), W:20 in
biomes.rs. **KEEP BOTH, reconciled**: g40's bodies on d101's two-row split
(Marble row + Granite row). g40 kept "own pass_rng(Marble) slot unchanged" —
the merger moves its granite half onto d101's Granite row (same stream name
`pass_rng("Granite")`, RNG-neutral). g40's P0 PlaceTight/stalactite-table seams
overlap g41/d106 (§3.2).

### 2.23 Spider-cave residuals / mushroom residuals — single owners

g41 `e5e2d783..ee937411`: egg-sac validator chain + missing stalactite arms +
PlacePot/PlaceSmallPile vetoes + spider shroom veto/ILied/remix y-bands + webs
DST row-50 + SpiderWallRemover (unwired), W:14 new (34 in file). **KEEP g41**
for the Spider Caves residuals; its validator chain is the canonical one
(§3.2). g42 `65009957..d4836bbd`: GlowingMushroomPatches double-bound/desert
veto/py-advance deltas, GrowTree verbatim rewrite (streaming trunk loop,
70 split-box, case-5 dispatch), GrowShroom (dead-code, unwired),
DoSurfaceIsMushrooms per-row Next(3), Convert-3 cell, the plants-pass mushroom
arm wiring. **KEEP g42** for Mushroom Patches / Planting Trees / plants-pass
residuals. g42's `growth/trees.rs` rewrite supersedes d102's 2-line tweak and
is the body d106 R4's gem-trees and g36/d102's shimmer gem-tails should call.

### 2.24 Skyblock (gated, `:11511`) — single owner

f24 A1–A4 (`372a4ec7..9cbbf9de`): the 75/22-name denyAll/denySome pruning
tables, SkyblockIsland 14-style palette, GenerateSkyBlockWorld + the gated
step, seed_tails (DoPooEverywhere / DoErrorWorldShuffleBlocks /
PlaceTorchesAroundSpawn), W:5+3. **KEEP f24** — no competitor. The skyblock
Jungle re-registration (`:11509`) is dead (denyAll always true) — no row.
f24's denySomeGeneration table is the canonical gate for every pass that
grew a local copy (§3.5).

## 3. Helper / machinery reconciliation (cross-cutting dups)

### 3.1 PlaceTile-wrapper helpers: g39 canonical, pass-local slices DROP

g39 `place_helpers.rs` (1744 lines, W:15, H1–H15 complete, unwired): general
Place1x1/1x2/2x1/2x2(+Style)/3x1/3x2/3x3/3x4/5x4/6x3/4x2, Place1xX/2xX (92/93/
105/104/207 arms), the five wall-painting wrappers, PlaceLogicTiles/
PlaceOnTable1x1, PlaceMan/Woman, the five SolidTile floor variants +
PlatformProperTopFrame. **KEEP g39 as the single helper module.**
DROP/rewire the pass-local duplicates:
- g37 `passes_b.rs`: Place1x1/Place2xX/Place3x2/PlaceSmallPile slices (its own
  doc defers this to the merger);
- d103 `statues.rs`: Place2xX/Place1x1 slices;
- d104 `structures.rs`/`surface.rs`: the generalised Place3x2 + PlaceSmallPile
  (g39 H6 carries more arms: picnic/campfire/285-family/186-187 speleothem
  veto/26 boulder vetoes; d104's C# type parameter on place_small_pile must
  survive the rewire — g39 H6's signature check at the gate);
- d105 `surface.rs` Place2x2/PlaceSunflower local arms → g39 H4/H5 where the
  C# body is the general wrapper (d105's PlaceDye/PlaceSuitableHerbHere are
  pass-specific, keep);
- c29's Place2x2 nactive()/tileTable arms are **fixes to the general wrapper**
  — verify g39 H4 carries them (Tile.cs:240-247 + :51371-51378); if not, port
  them onto g39 before dropping c29's local copy.
g39 is unwired by design; the merger wires each kept pass onto it. Where a
kept pass's local slice has a draw-schedule difference (verify against the
C# at the gate), the pass keeps its slice and g39's general version serves
the other callers — record the exception in PORT_MAP.

### 3.2 Stalactite validator chain: ONE chain, g41 body + d106 generalisation

Four copies exist: g36 `shimmer.rs` (local check_stalactite/update/style),
g40 P0 (PlaceTight table arms: granite 368/marble 367 hanging + hive rising),
g41 `spider_gem_moss.rs` (full chain `:38643-39175` incl. the spider
GetDesiredStalagtiteStyle override + tileMoss[627], W:14), d106 R4
`spider_gem_moss.rs` (chain `:38996-39176` + PlaceTight generalised over
WorldGenRand + `Streams::pass_unified`).
**KEEP: g41's chain body** (widest anchor range, the spider override, 14
witnesses) **+ d106's WorldGenRand generalisation + pass_unified stream**
(Stalac's gem-trees need the UnifiedRandom draw semantics; d106 proved spider
output byte-identical under the generalisation). g40's table arms are the
same arms g41 adds — keep one table (g41's, it has the witness coverage).
DROP g36's local chain (rewire shimmer's PlaceTight tail onto the canonical
chain; if the draw schedule differs, keep g36's local copy and record the
exception). Single home: `spider_gem_moss.rs` (or a new `stalactite.rs`
extracted at the gate — cosmetic).

### 3.3 Liquid-settle machinery: f25 core, d106 R2 pass re-based

f25 (`settle_liquids.rs` + `liquid.rs` +225 + `tileobject.rs` +147) and d106
R2 (`liquids.rs` +398 + `liquid.rs` +394 + `tileobject.rs` +108 + `tile.rs`
+18) both port QuickWater/SettleWaterAt/WaterCheck/quickSettle +
LiquidInteractionsCleanup + ShimmerRemoveWater + GetTileData(Tile)/
CheckLavaDeath/CheckWaterDeath. **KEEP f25's machinery as the core** (first
pass owner, the :23755 up-shimmer quirk kept); **KEEP d106 R2's pass**
(Settle Liquids Again, `:21246`) re-based onto f25's core, retaining d106's
additions f25 lacks: the `generating` toggle, `tile.rs` any_shimmer/
left_slope/right_slope, the None-guarded ShimmerRemoveWater stand-in upgrade
(f25's shimmer_position defer → g36 A1's published position). DROP the
duplicate half of whichever module loses the diff at the gate — the two
`liquid.rs`/`tileobject.rs` additions are the same C# bodies and MUST NOT
both land (double definition / double draw).

### 3.4 GenVars slot dedupe

`shimmer_position` is added by d102, d104, d106 and referenced by f25/g36;
g36 A1 publishes it + the 200×200 safety rect. **Keep ONE field + rect**
(g36's, it is the producer). `ore_patch_x`/`num_ore_patch`: d104 and g37 both
add — keep d104's (§2.15). `island_house_sites`/floatingIslandHouseX/Y/Style/
skyLake: d103/d104/g37/g44 all touch — keep g44's ledger (§2.11).
`dual_dungeons`/dungeonGenVars[2]: f24's (§2.18). mCave/snow slots: d101
producer, d102 consumer — already complementary, keep both halves.

### 3.5 denySomeGeneration / denyAllGeneration: f24 A1 canonical

Local copies grew in d104 (`surface.rs::deny_some_generation`), d105 (skyblock
gate helper), d106 (`cleanup.rs::deny_some_generation`), g38
(`surface.rs::deny_some_generation(opts)`). f24 A1 ports the full 75-name
denyAll + 22-name denySome pruning tables + the eight-micro-seed gate +
extraLivingTrees arm (`WorldGen.cs:3063-3281`). **KEEP f24's table**; rewire
every kept pass's local gate onto it (the local copies are subsets; the gate
runs the pruning census test). DROP the locals after rewire.

## 4. Single-owner passes (no contest — KEEP as landed)

| pass(es) | owner | tip evidence |
| --- | --- | --- |
| Sand Patches, Mount Caves, Dirt Wall Backgrounds, Clay, Small Holes, Dirt Layer Caves, Rock Layer Caves, Generate Ice Biome (+Granite row) | d101 | `passes1.rs` W:11 + `ice_biome.rs` W:2; 9/9 ledger |
| Silt, Webs, Slush, Mountain Caves, Gems | d102 | `passes2.rs` W:23 total (per-pass 2–3 each); producer/consumer GenVars seams with d101 |
| Oasis (+awakened cacti block), Waterfalls | d103 | `90b7eee6`/`561b3ab2`; oasis ledger + PlaceOasis; both liquid-edge sweeps |
| Traps, Piles, Spawn Point | d104 | `traps.rs` W:2 (+witness-fix `341f4280`), structures W:2, surface W:1; g38 claimed, never landed |
| Guide, Sunflowers, Herbs, Dye Plants, Flowers, Mushrooms, Random Gems, Moss Grass | d105 | `starter_npcs.rs` W:9, `surface.rs` W:18, `spider_gem_moss.rs` #43 W:3 + #44 W:3; 9/9 ledger (Larva #45 yielded to c31, §2.20) |
| Grass Wall, Settle Liquids Again (pass), Tile Cleanup, Stalac (pass), Remove Broken Traps | d106 | R1–R5 landed (`surface.rs` W:4, `liquids.rs` W:6, `cleanup.rs` W:3, `spider_gem_moss.rs` W:22); R6 Final Cleanup yielded to g37; machinery dedupe per §3.2/§3.3 |
| Final Cleanup (pass body) | g37 gap-fill | `4ced188f9`→`81fd1e0d` (local; pushed tip `53a5b5b5`), `final_cleanup.rs` W:5 + positional; ShimmerCleanUp arm → g36 (§6.1.5) |
| Settle Liquids (pass + core) | f25 | §2.19 |
| Skyblock (gated) | f24 | §2.24 |
| Dither Snake (gated) | f24 | §2.18 (DROP c30) |
| Pyramids (faithful) + lily/cattail/bamboo arms | c33 | §2.16 (DROP its Water Chests only) |
| Hives (faithful) + Larva | c31 | §2.20 (DROP d105 #45's Larva slice) |
| Micro Biomes dead-man's-chest block | c29 | §2.21 |
| Marble/Granite residuals | g40 (+d101 row) | §2.22 |
| Spider Caves residuals + validator chain | g41 | §2.23 |
| Mushroom-biome residuals + GrowTree + plants-pass mushroom arm | g42 | §2.23 |
| Floating Islands + Floating Island Houses + skyLakes | g44 | §2.11 (DROP d103/d104/g37) |
| PlaceTile-wrapper helper family | g39 | §3.1 |
| Shimmer + ShimmerCleanUp tail arm | g36 A1 + gap-fill | §2.1 — Shimmer is g36's surviving pass; `shimmer_clean_up` (the Final Cleanup :22472 arm, unwired) composes into g37's Final Cleanup |

## 5. FINAL pipeline order table (all 109 registrations, single owner each)

`base` = already executed at a725acb0 (no lane owns it; residual owners noted).
Order = `VANILLA_CHAIN` declaration order (the C# `AddPasses` anchors).

| # | vanilla name | C# anchor | KEEP owner | notes |
| --- | --- | --- | --- | --- |
| 1 | Terrain | :11504 (TerrainPass.cs) | base | terrain.rs compact model (documented deviation) |
| 2 | Skyblock (gated) | :11511 | **f24** | A1–A4; denyAll pruning canonical (§3.5) |
| 3 | Jungle (skyblock re-reg) | :11509 | — dead | denyAll always true under skyblock (f24 census); no row |
| 4 | Dunes | :11517 | base | + f24 B6 dual-setup draws at this slot (gated) |
| 5 | Ocean Sand | :11580 | base | |
| 6 | Sand Patches | :11689 | **d101** | |
| 7 | Tunnels | :11718 | base | compact cave field; d101's discrete passes surround it |
| 8 | Mount Caves | :11784 | **d101** | producer of numMCaves/mCaveX/Y |
| 9 | Dirt Wall Backgrounds | :11858 | **d101** | |
| 10 | Rocks In Dirt | :11896 | base | |
| 11 | Dirt In Rocks | :11926 | base | |
| 12 | Clay | :11959 | **d101** | |
| 13 | Small Holes | :12008 | **d101** | |
| 14 | Dirt Layer Caves | :12067 | **d101** | |
| 15 | Rock Layer Caves | :12107 | **d101** | |
| 16 | Surface Caves | :12162 | base | d107 seed gates ride it |
| 17 | Wavy Caves | :12271 | base | |
| 18 | Generate Ice Biome | :12312 | **d101** | producer of snowTop/Bottom/MinX/MaxX |
| 19 | Grass | :12423 | base | micro-grass seed bodies |
| 20 | Jungle | :12466 | base | |
| 21 | Mud Caves To Grass | :12468 | base | |
| 22 | Full Desert | :12500 | base | |
| 23 | Mushroom Patches | :12544 | base + **g42** | g42 deltas (double bound, desert veto, py advance, ShroomPatch guard) |
| 24 | Marble | :12757 | base + **g40** | g40 residual bodies on the Marble row |
| 25 | Granite | :12839 | **d101** row + **g40** body | §2.22 reconciliation |
| 26 | Floating Islands | :12927 | **g44** | verbatim rewrite incl. skyLakes; DROP base shape + d103/d104/g37 ledger edits |
| 27 | Dirt To Mud | :13123 | base | |
| 28 | Silt | :13143 | **d102** | |
| 29 | Shinies | :13182 | base | |
| 30 | Webs | :13607 | **d102** | mCave anchor override (d101 producer) |
| 31 | Underworld | :13645 | base | |
| 32 | Corruption | :13866 | base | d104's spread_evil_grass repeat param lands with §2.14 |
| 33 | Lakes | :14556 | base | |
| 34 | Slush | :14697 | **d102** | snow-rect consumer (d101 producer) |
| 35 | Dual Dungeons Dither Snake (gated) | :14755 | **f24** | DROP c30; verify 3-seam gate (§2.18) |
| 36 | Dungeon | :14757 | base | c32 style tables; f24 B5 dual statics |
| 37 | Mountain Caves | :14873 | **d102** | CaveOpenater/Cavinator |
| 38 | Beaches | :14888 | base | |
| 39 | Gems | :15050 | **d102** | |
| 40 | Gravitating Sand | :15138 | base | |
| 41 | Create Ocean Caves | :15167 | base | treasure anchors consumed by #66 |
| 42 | Shimmer | :15195 | **g36 A1** | DROP d102; publishes shimmer_position + safety rect |
| 43 | Clean Up Dirt | :15246 | **d102** | DROP g36 A2 |
| 44 | Pyramids | :15360 | **c33** | faithful; chest core re-pointed to d103 (§2.8) |
| 45 | Dirt Rock Wall Runner | :15461 | **d102** | DROP g36 A3 |
| 46 | Living Trees | :15476 | base | |
| 47 | Wood Tree Walls | :15717 | **d102** | DROP g36 A4 |
| 48 | Altars | :15750 | base | |
| 49 | Wet Jungle | :15810 | base | |
| 50 | Jungle Temple | :15834 | base | d108 paint/seed arms |
| 51 | Hives | :15927 | **c31** | faithful rewrite; base dome retired |
| 52 | Jungle Chests | :15977 | base | |
| 53 | Settle Liquids | :16132 | **f25** | g36 yielded; machinery core (§3.3) |
| 54 | Remove Water From Sand | :16211 | base | |
| 55 | Oasis | :16243 | **d103** | + awakened cacti-pass oasis block |
| 56 | Shell Piles | :16266 | base | |
| 57 | Smooth World | :16419 | base | |
| 58 | Waterfalls | :16610 | **d103** | |
| 59 | Ice | :16667 | base | fragile ice |
| 60 | Wall Variety | :16697 | **d103** | DROP g36 A6; rewire shimmerPosition (§2.5) |
| 61 | Life Crystals | :16755 | **d103** | DROP g36 A7 |
| 62 | Statues | :16853 | **d103** | DROP g37 B1 |
| 63 | Buried Chests | :16971 | base | cave houses |
| 64 | Surface Chests | :17130 | **d103** | DROP g37 B2; full AddBuriedChest core |
| 65 | Jungle Chests Placement | :17211 | base | |
| 66 | Water Chests | :17245 | **d103** | DROP g37 B3 + c33 underwater; carry c33 stocking pair |
| 67 | Spider Caves | :17356 | base + **g41** | egg-sac validator + web/wall residuals |
| 68 | Gem Caves | :17428 | base | |
| 69 | Moss | :17472 | base | |
| 70 | Temple | :17701 | base | temple part 2 |
| 71 | Cave Walls | :17711 | **d103** | DROP g37 B4 |
| 72 | Jungle Trees | :17840 | base | |
| 73 | Floating Island Houses | :17859 | **g44** | DROP d103/d104/g37 B5; verify drunk door arms (§2.11) |
| 74 | Quick Cleanup | :17875 | **d104** | DROP g37 B6 |
| 75 | Pots | :17996 | base | |
| 76 | Hellforge | :18187 | **d104** | DROP g37 B7 |
| 77 | Spreading Grass | :18237 | **d104** | DROP g37 B8; remix → d107 |
| 78 | Surface Ore and Stone | :18447 | **d104** | DROP g37 B9 |
| 79 | Place Fallen Log | :18513 | **d104** | DROP g38 C1; port g38's 3 witnesses |
| 80 | Traps | :18657 | **d104** | |
| 81 | Piles | :18788 | **d104** | |
| 82 | Spawn Point | :19460 | **d104** | lifts the compact spawn out of step_world_tail |
| 83 | Grass Wall | :19589 | **d106 R1** | orphan adoption |
| 84 | Guide | :19685 | **d105** | gen-time NPC ledger + App materialisation |
| 85 | Sunflowers | :19898 | **d105** | |
| 86 | Planting Trees | :19940 | base + **g42** | GrowTree verbatim rewrite underneath |
| 87 | Herbs | :19964 | **d105** | |
| 88 | Dye Plants | :19981 | **d105** | |
| 89 | Webs And Honey | :19995 | base | |
| 90 | Weeds | :20064 | base | |
| 91 | Glowing Mushrooms and Jungle Plants | :20122 | base + **g42** | G5 mushroom arm wiring (`:20133-20147`: triple GrowTree retry + PlaceTile 71) |
| 92 | Jungle Plants | :20165 | base | |
| 93 | Vines | :20193 | base | |
| 94 | Flowers | :20447 | **d105** | logX/logY seam from #79 |
| 95 | Mushrooms | :20599 | **d105** | |
| 96 | Gems In Ice Biome | :20697 | base | |
| 97 | Random Gems | :20729 | **d105 #43** | landed `9f7b1a24` (pushed): ExposedGemsUnderground scatter + marble/granite diamond patches, PlaceTile-178 multi-directional anchors, pass_rng(Random Gems); W:3 |
| 98 | Moss Grass | :20769 | **d105 #44** | landed `d590d99a` (pushed): LongMoss sweep growing 184 on moss-tile neighbours, PlaceTile-184 two-gate arm, pass_rng(Moss Grass); W:3 |
| 99 | Muds Walls In Jungle | :20813 | base | |
| 100 | Larva | :20868 | **c31** | KEEP c31 (§2.20); d105 #45 `22a0026ce` ALSO landed a Larva step (pushed) — the contest is re-adjudicated in §2.20, c31 stays the faithful owner |
| 101 | Micro Biomes | :20901 | base + **c29** | dead-man's-chest block leads the micro group; thin ice/campsites/sword/mining/mahogany already base |
| 102 | Settle Liquids Again | :21246 | **d106 R2** | pass kept, machinery re-based on f25 (§3.3) |
| 103 | Cactus, Palm Trees, & Coral | :21310 | base | + d103 oasis block + c33 lily/cattail/bamboo arms |
| 104 | Tile Cleanup | :21520 | **d106 R3** | |
| 105 | Lihzahrd Altars | :21943 | base | |
| 106 | Water Plants | :21973 | base | |
| 107 | Stalac | :22038 | **d106 R4** | validator chain unified with g41 (§3.2); gem-trees call g42's trees.rs |
| 108 | Remove Broken Traps | :22121 | **d106 R5** | |
| 109 | Final Cleanup | :22141 | **g37 gap-fill** | landed `4ced188f9`→`81fd1e0d` (local g37, claim-probed vs g36 `6d46f6a5`): wall-hole fill + whole-map sweep + 668 dirt-throw scatter, pass_rng(Final Cleanup), W:5+positional; ShimmerCleanUp arm (:22472) deferred to g36 → `shimmer.rs::shimmer_clean_up` (this gap-fill, unwired); liquid-merge sweep (:22473) = f25/d106 seam (§3.3) |

## 6. Gaps and merger playbook

### 6.1 Unlanded passes — RE-VERIFIED 2026-11-21 (gap-fill phase, all pushed tips + local claims read)

The three gaps flagged at `6d46f6a5` are now ALL filled/claimed elsewhere — no
follow-up lane is needed for the pass bodies:

1. **Random Gems** (`:20729`) — LANDED d105 #43 `9f7b1a24` (pushed tip
   `241ffa39`). ExposedGemsUnderground scatter + marble/granite diamond patches
   (`spider_gem_moss.rs::pass_random_gems`), PlaceTile-178 multi-directional
   anchors, `pass_rng("Random Gems")`, W:3. Row 97 → d105.
2. **Moss Grass** (`:20769`) — LANDED d105 #44 `d590d99a` (pushed). LongMoss
   sweep growing 184 on moss-tile neighbours (`pass_moss_grass`), PlaceTile-184
   two-gate arm, `pass_rng("Moss Grass")`, W:3. Row 98 → d105.
3. **Final Cleanup** (`:22141`) — CLAIMED + LANDED g37 gap-fill `4ced188f9`
   (local g37 worktree, advanced to `81fd1e0d`; pushed g37 tip is still
   `53a5b5b5`). g37's claim probe explicitly read g36's tip `6d46f6a5` row 109
   "GAP — unlanded" + d106 `e2c2609c` R6 unchecked → unclaimed → g37 fills
   (`final_cleanup.rs::pass_final_cleanup`, wall-hole fill + whole-map sweep +
   668 dirt-throw scatter, `pass_rng("Final Cleanup")`, W:5 + positional).
   **KEEP g37** — no duplicate port on g36 (keep-one + never-same-task). Row 109
   → g37. d106 R6 yields.

**New residual found in the re-verification (the g36 gap-fill this phase ports):**

4. **Larva** (`:20868`) — d105 #45 `22a0026ce` (pushed) landed a Larva step
   (`structures.rs::pass_larva`, the hive-pass larva ledger consumer) AFTER this
   matrix assigned row 100 to c31. The contest is re-adjudicated in §2.20: c31
   stays the faithful owner (its `hive.rs` carries the full HiveBiome rewrite +
   the drunk extra-stand arm + W:10; d105's is the ledger-consumer slice over the
   retired dome). **DROP d105's Larva step + its `structures.rs::pass_larva` /
   `pass_beehives` larva-ledger edits at the merge; KEEP c31.**
5. **ShimmerCleanUp** (`WorldGen.cs:34716-34821`, the Final Cleanup `:22472`
   arm) — g37's Final Cleanup port DEFERRED this arm back to g36 ("reads
   GenVars.shimmerPosition, which g36 A1 owns; no slot on this branch"). g36 A1
   publishes `shimmer_position`, so the arm is g36-owned. **PORTED this gap-fill**
   (`shimmer.rs::shimmer_clean_up` + `shimmer_remove_water` + `shimmerator`,
   verbatim, W:7, a==b). UNWIRED by design (the g39/f25 convention): the merger
   composes it into g37's Final Cleanup at `:22472`, after the `Finish*` tails
   and before the `Liquid.LiquidCheck` merge sweep (`:22473-22483`, the f25/d106
   liquid-core seam §3.3). The `Liquid.tilesIgnoreWater` window (`:34767`/`:34810`)
   is a documented no-op for this arm (it writes liquid directly, makes no
   solidity read that the window changes — the Shimmerator flood carries its own
   `solid_tile_ignore_water` predicate). The `Convert(j,i,0,3)` cleanse reuses the
   shared `spread::convert_cleanse` (its pre-existing genRand deviation stands).

### 6.2 Merge order (dependency-first)

1. **Foundations**: g39 helpers (§3.1) → f24 A1 gate tables (§3.5) → f25 liquid
   core (§3.3) → g41+d106 stalactite chain (§3.2) → g42 trees.rs. These are the
   shared bodies every kept pass rewires onto.
2. **Producers**: d101 (mCave/snow anchors, Granite row) → g36 A1 Shimmer
   (shimmer_position) → g44 islands ledger → d104 GenVars ore slots.
3. **Pass waves in vanilla order**: d101/d102 early passes → c33 Pyramids →
   d103 mid passes (Oasis..Cave Walls, chest core first) → g44 houses → d104
   late passes → d106 R1 → d105 plants/NPC/gems/moss (Random Gems #43, Moss
   Grass #44) → c31 Larva (DROP d105 #45's Larva, §2.20) → c29 micro block →
   d106 R2–R5 tail → **g37 Final Cleanup** (`final_cleanup.rs`, the last classic
   pass; compose g36's `shimmer_clean_up` at its `:22472` arm, then the f25/d106
   `Liquid.LiquidCheck` merge sweep at `:22473`). f24's gated steps slot at
   :11511/:14755 (classic pins unchanged by construction).
4. **DROP list execution** (per branch, surgical — never whole-branch reverts
   where the branch also carries KEEP rows):
   - d102: drop shimmer.rs + Shimmer row ONLY (keep Silt/Webs/Slush/Mountain
     Caves/Gems/Clean Up Dirt/Dirt Runner/Wood Walls).
   - g36: drop A2/A3/A4/A6/A7 blocks (micro_finish/living_trees/structures);
     keep shimmer.rs + its GenVars publication + the gap-fill `shimmer_clean_up`
     arm (unwired; the merger composes it into g37's Final Cleanup at :22472).
     g36 contributes ONE pass (Shimmer) + the ShimmerCleanUp tail arm.
   - g37: drop B1–B9 (passes_b.rs + islands.rs edits + pipeline rows) BUT KEEP
     the Final Cleanup gap-fill (`final_cleanup.rs` + its pipeline row + the
     `pass_rng("Final Cleanup")` stream); g37 contributes the Final Cleanup pass
     (the B-row value is the census split + the 512-wide statue proof + the
     witness set the gate may port). Compose g36's `shimmer_clean_up` into it.
   - d105: drop the #45 Larva step + `structures.rs::pass_larva` + the
     `pass_beehives` larva-ledger edits + the `GenVars.larva` Vec slot (§2.20,
     c31's LarvaAnchors is canonical); KEEP #43 Random Gems + #44 Moss Grass.
   - d103: drop island_houses.rs + its islands.rs ledger edits ONLY.
   - d104: drop island_houses.rs + its islands.rs edits ONLY.
   - g38: drop C1 (surface.rs fallen-log block + pipeline row); port its 3
     witnesses onto d104's body first.
   - c30: drop dither_snake.rs + pipeline gate; verify f24 3-seam coverage first.
   - c33: drop pass_underwater_chests ONLY (keep Pyramids + stocking-hook pair
     + lily/cattail/bamboo).
   - d106: dedupe liquid/tileobject machinery against f25 (§3.3); dedupe the
     stalactite chain against g41 (§3.2).
4. **Gate discipline** (every lane's contract): hash re-pins were deferred —
   the merge gate runs ALL witnesses (every lane wrote them NOT run), measures
   the new SEED42_512 / DEFAULT_WORLD constants ONCE on the fully merged chain,
   and keeps the `a == b` determinism halves green throughout. Per-lane
   "pins unchanged" claims (e.g. g37 B1's 512 beach-band proof, d102's
   sub-vanilla valves) are arguments about ISOLATED insertions — they do not
   compose; only the gate's measured re-pin is authoritative.

### 6.3 Residual-arm verification checklist (KEEP versions to re-read at the gate)

- g44 IslandHouse: drunk crimson wall split + case-10 door dispatch (d103 had
  them; g44's commit list does not name them).
- d103 Water Chests: unbounded inner loops — add g37's 10000-cap guard.
- d103 Wall Variety: shimmerPosition rewire (§2.5) — currently zero-init.
- f24 dither wire: GenerationRunner.advance + total_weight seams (§2.18).
- g39 Place2x2: c29's nactive()/tileTable fixes present? (§3.1).
- g39 Place3x2 vs d104's type-parameter generalisation (§3.1).
- d104 Place Fallen Log: g38's frame/veto/gate witnesses ported (§2.17).
- f25 ShimmerRemoveWater: upgrade the None-guard stand-in to g36's published
  shimmer_position (§3.3).

---
*Produced by the g36 reconcile phase. No code changed; this document is the
single authoritative keep-one decision record for the worldgen merger.*
