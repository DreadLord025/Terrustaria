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
> **SEAL (2026-11-21, FINAL-FINAL):** all peer tips re-fetched and every
> addendum incorporated — g37 research/75 (+§8) & 124, d104 research/86
> (+§6 addendum: the repeat:true fix `74103ae4` + the Hellforge graft
> `574f1d58` LANDED), g44 research/87 (FINAL-KEEP + biome closeout), c33
> research/120 (Water Chests arm-by-arm + the chest_gen superset), g41
> 93/94/95 (NTB/FinishNoTraps/FinishRemixWorld hosts WIRED + spawn pool +
> PlacePot veto), g40 94 (desert-entrance family), g42 12j (tails +
> dispatcher), c29 95 (gem-moss fidelity), f25 closeouts (Convert C1–C7 +
> ConvertSkyIslands). Larva FINAL (§2.20): the DROP-d105 stands — the
> "both-survive" pair is `:20868` (c31) vs `:25608-25629` (g41 NTB tail),
> NOT c31 vs d105. §5a adds the 120-row EXECUTED-order union (75 §4 +
> Final Cleanup), single owner each. §6.4 publishes the exact
> ShimmerCleanUp compose-hunk; §7 is the one-paragraph merger bible.
>
> **SEAL 3/4 + 4/4 (2026-11-21, resume):** the g41 RETIRE tips re-read
> (`2254f2e1`→`3c6da7a3f`, research/96) — the **stalactite-6-point-FINAL**
> receipt `b8ee829b8` consumed into §3.2 (d106 `e2c2609c` owns its half:
> `pass_stalac:774` + `pass_unified("Stalac")`; KEEP g41's internal
> validator, the d106 Stalac sites DROP their explicit `check_stalactite`
> calls — the double-validation trap); §2.23 gains the RETIRE delta
> (QuickWater activation → g46 RE-FOLD union `9bc722e5f`, the id-agnostic
> spawn-pool hunk); §6.3 gains the four g41 gate items; §6.4 re-verified
> against the retire tips (gv already named, union seam line numbers).
> §7 lands the merger bible + the **g36 family declaration**: ONE pass
> (Shimmer) + ONE tail arm (ShimmerCleanUp) + the GenVars publications;
> everything else DROP-listed with a named keeper. Matrix FINAL-FINAL.
>
> **This doc IS the merger playbook.** Every vanilla pass gets exactly ONE
> owner branch below; every duplicate gets an explicit KEEP (with arm-count /
> witness-count evidence) and DROP list.

## 1. Pushed tips read (all worldgen lanes, re-fetched at the SEAL 2026-11-21)

| lane | tip | role |
| --- | --- | --- |
| d101 | `4194acf8` | passes 1–9 (Sand Patches..Generate Ice Biome) + Granite row-split |
| d102 | `29e8859b` | Silt/Webs/Slush/Mountain Caves/Gems + Shimmer/Clean Up Dirt/Dirt Runner/Wood Walls (9/9) |
| d103 | `12318e50` | Oasis/Waterfalls/Wall Variety/Life Crystals/Statues/Surface Chests/Water Chests/Cave Walls/Island Houses (9/9); `8be04cb0`→tip = island-house borrow-fix only (87 closeout ledger, no arm change) |
| d104 | `242692e1` | Island Houses/Quick Cleanup/Hellforge/Spreading Grass/Surface Ore/Fallen Log/Traps/Piles/Spawn Point (9/9) + research/86 reconcile `0c7a3483` + §6 addendum: Spreading Grass repeat:true FIX `74103ae4`, Hellforge PlaceTile-77 graft `574f1d58` (both LANDED — §2.13/§2.14 updated) |
| d105 | `241ffa39` | Guide/Sunflowers/Herbs/Dye Plants/Flowers/Mushrooms/Random Gems/Moss Grass/Larva (9/9 landed — #43 `9f7b1a24`, #44 `d590d99a`, #45 `22a0026ce`; LANE 5 COMPLETE; Larva #45 yields to c31, §2.20 FINAL) |
| d106 | `e2c2609c` | Grass Wall/Settle Liquids Again/Tile Cleanup/Stalac/Remove Broken Traps (R1–R5; R6 Final Cleanup yielded to g37's gap-fill `4ced188f9`) |
| f24 | `9f2276d5` | the two GATED passes: Dither Snake (B1–B6) + Skyblock (A1–A4), review-fix round included |
| f25 | `84b90360` | Settle Liquids (orphan adoption fd098528) + the Convert C1–C7 helper family + wiring closeouts (spray/tree/moss), ConvertSkyIslands `f0fe0f38`, solution-spray rows + powder census (§2.19/§3.6) |
| g36 | `bf7b6bf5` + this seal | A1 Shimmer, A2 Clean Up Dirt, A3 Dirt Runner, A4 Wood Walls, A6 Wall Variety, A7 Life Crystals (A5 yielded to f25); gap-fill: ShimmerCleanUp arm (`shimmer.rs::shimmer_clean_up`, unwired); seal: matrix FINAL-FINAL + compose-hunk + g41e retire receipts + merger bible + family declaration (§7) |
| g37 | `81fd1e0d` (PUSHED at the seal) | B1–B9 + FINAL CLEANUP gap-fill (`4ced188f9`→`81fd1e0d`, three witness fixes) + research/75 (§8 addendum) + research/124 hellforge graft-patch (superseded by d104's landed `574f1d58`); ShimmerCleanUp arm deferred back to g36 → compose-hunk §6.4 |
| g38 | `8c9b7b60` | C1 Place Fallen Log only (rest of c-quarter claimed, not landed) |
| g39 | `bab25558` | H1–H15 PlaceTile-wrapper helper family (`place_helpers.rs`, unwired) |
| c29 | `bf90f6e7` | DeadMansChestBiome + the Micro Biomes dead-man's-chest block + gem-moss fidelity `12a1fc97` (Spread.Gem duplicate-visit redraw, neonMossBiome ×4 fixes, moss arm A reject ledger + tileDungeon + arm F live tileMoss sweep — §2.21/§4) |
| c30 | `8f5f1086` | dual-dungeons dither snake (whole family, single file) |
| c31 | `d538536b` | faithful Hive rewrite + Larva step + larva-break→Queen Bee seam (§2.20 FINAL owner of row 100) |
| c33 | `5479468c` | faithful Pyramids + UnderwaterChests + lily/cattail/bamboo `fd7fae0e` + underground-desert music `c7393225` + the chest_gen stocking superset `3ecb8b08`/`5479468c` + research/120 keep-one addendum (§2.9/§2.16/§3.7) |
| g40 | `e367fb56` | Marble/Granite residual fidelity + the DesertBiome entrance family (`7bb5332e`/`a8150064`/`e367fb56`, research/94: 17-deviation residual closed verbatim — §2.22/§4) |
| g41 | `3c6da7a3f` | spider-cave egg-sac validator chain + stalactite arms + webs/DST residuals + **the NTB host (`78b0cd15`, incl. the `:25608-25629` larva tail) + FinishNoTraps (`d267d43f`) + FinishRemixWorld (`6300aab6`) hosts WIRED at step_seed_finish** + spider spawn pool `c5523972` + PlacePot remix veto `d1892354` (research/93/94/95 — §2.23/§3.8); RETIRE phase `27ba76593`→`3c6da7a3f` (research/96: g47f liquid receipt + d36e hand-off + **stalactite-6-point-FINAL `b8ee829b8`** + FINAL declaration — §3.2/§7) |
| g42 | `92f7eced` | mushroom-biome residuals + GrowTree rewrite + the tree-type tail deltas (`d13d05a1`), mushroom drops/runtime arm (`73a4e267`/`a964730f`), the grass-growth DISPATCHER D1–D6 (`dd659587`..`92f7eced`, research/12j — §2.23) |
| g44 | `e1047c31` | verbatim FloatingIslands + IslandHouse + skyLakes + drunk variants (`cb5985c9`) + the FULL AddBuriedChest graft (`e172bc76`) + sky loot/spawn/housing closeouts (`0898f352`..`e1047c31`, research/87 FINAL-KEEP — §2.11 FINAL) |

Note: d105/d106 tips moved past the hashes quoted in the dispatch brief
(dd43734a / 29faa099); this matrix uses the **current** tips above. d107/d108
(special-seed arms / temple paint) and c32 (dungeon style tables) carry no
classic-pass rows and are out of the keep-one scope except where noted.
The liquid-sim lanes (g45/g46/g47/c34/f47) are governed by research/70
(g47 `5c09a68b`, the FINAL liquid merge order) — §3.3 cross-refs it; this
matrix owns the worldgen-pass side of that seam.

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
c33's stocking-hook additions onto the kept core's chest_stock seam, and g37's
10000-cap guard if d103's loops are truly unbounded (hang risk on waterless
worlds). **c33's Pyramids stay** (§2.16).

**SEAL refinement (research/120 @ c33 `5479468c`, arm-by-arm 18-row table —
INCORPORATED, supersedes the two merger clauses above):**
1. **The 10000-cap graft is STALE — do NOT graft it.** d103's OUTER attempt
   loops already carry `num7 > 10000` exactly like the C# (`:17330-17333`/
   `:17348-17351`, 120 row A13); only the INNER water re-draw hunt is
   unbounded and **that is the C#'s own shape** — both ports keep it
   verbatim. Capping the inner loop would change the stream on sparse-water
   worlds (120 row A14: keep verbatim). §6.3's checklist row is corrected.
2. **NEW d103 deviation the matrix missed (120 row A2) — fix at the merge:**
   the treasure-main extraLiquid arm. The C# `:17252-17256` draws the
   5-list `{863,186,277,187,4404}` FIRST, then (extraLiquid) draws the
   6-list `{…,268}` and OVERWRITES — BOTH draws consume the stream. d103
   ports `if extra_liquid {6-list} else {5-list}` (ONE draw). Restore the
   draw-then-overwrite shape; add the stream-position witness (120 §5).
3. **The stocking carry is now c33's WHOLE `chest_gen.rs` block** (§3.7),
   not just the 4425/4460 pair: after `3ecb8b08`/`5479468c` it is the
   superset (flag5 two-independent-halves, flag11 main-based + surface-at-
   depth, preset-main weapon skips, the re-derived mid/deep bands,
   CreativeWings, every-slot-0 Prefix); d103's copy still carries the
   phantom rows. The shared `ChestSiteEvent` seam composes with no
   signature change (120 row A17).
4. 120 confirms KEEP d103 (wins 9 arms, c33 wins 0 in the pass body) and
   adds four gate witnesses: the A2 double-draw, the A7 anniversary
   `Next(7)`, the A11 arm-2 no-remix asymmetry, the A8 268 cycle slot.
   The 1/50 first-draw asymmetry is NOT a c33 exclusive (both keep it,
   row A9) — the §2.9 evidence line above is corrected by this note.

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

**SEAL — research/87 FINAL-KEEP (@ g44 `e1047c31`) INCORPORATED; the two
merger clauses above are CLOSED:**
- drunk-crimson wall split + case-10 door dispatch: **both verified PRESENT**
  (islands.rs:1199-1202 `:79217-79220`; :1011-1027 `place_tile_door` up/down
  `:59843-59883` + the remix 5/38 split :1293-1297). No gate-port needed;
  §6.3's checklist row CLOSED. Variant A `cb5985c9` added the drunk
  snow/desert islands + fountain/palm + the full remix/getGood style tables.
- chest-core re-point: **DONE in-lane** (`e172bc76` — g44 carries the full
  AddBuriedChest core as its own `buried_chest.rs`, kind-aware
  `place_chest_kind`, the remix 467 retype arm). The merge **dedupes the two
  cores onto ONE — the kind-aware survivor** (g44's graft IS d103's core
  re-pointed); keep d103's Surface Chests half (§2.8). §2.8's "re-point g44"
  clause is superseded by this dedupe.
- 87's other FINAL-KEEP items: ONE deny helper (f24's
  `skyblock_gates::deny_floating_islands`, re-point g44's two call sites);
  g44's ledger-y convention (num11−radius/2 compact discs, num11 verbatim
  variants) stays; the five g44 anchors.txt rows stay, peer island rows
  repoint (never delete); **NEW housing dedupe** — g44's
  `housing::is_house_wall` → `edit::wall_house` full 279-entry table
  (`8e791809`, disc-wall 82 admits the sky room); d118/d119's narrow copies
  dedupe onto it (predicate-only, no RNG, their witnesses stay valid).
- 87-vs-86 scope adjudication (g44 `32e478d3`): 86 §1 B5's d104-wins verdict
  is d104-vs-g37 ONLY (g44 absent from 86); with variant A + the graft, g44
  is a superset of all three peers on every arm. d104's house module is
  UNCHANGED `93b1938e`→`242692e1` (86's addendum commits touch other files).
  No conflict; **rows 26/73 = g44 FINAL**.
- Boundary receipts: Sand Patches = d101 (`d0042f85e` verified line-for-line);
  the DesertBiome entrance family = g40 (§2.22); sky-lake water SURVIVES
  settle (87 §2, the WaterCheck/SettleWaterAt census + g47's gen_settle_loop
  read — the enclosed-bowl invariant, witness written); f25's
  ConvertSkyIslands (`f0fe0f38`) owns the anniversary island conversion
  `:24674-24714` (87 marked the call unported — cross-note, no dup).

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

**SEAL refinement (75 §2.7 → 124 → 86 §6.2, graft LANDED d104 `574f1d58`):**
the KEEP is now **d104's pass shell + the grafted `place_tile_hellforge`
route** — the merger takes d104's `pass_hellforge` + `place_tile_hellforge`
WHOLESALE and drops g37 B7 entirely; research/124's graft-patch is superseded
by the landed graft (no action). What the graft added over the bare
`evil::place_3x2(77)`: the gen-time 488 veto (`:59345`), the entry gate
(`:59373` — `tileSolid[77]` is FALSE so it always admits for this kind;
75's "refuses an occupied cell" claim was overstated, 86 §6.2 corrected it
against the generated TILE_SOLID bit), the inactive-seat
`Clear(Tile|TilePaint|Slope)` incl. Glow/Echo coatings (`:59473-59476`,
`Tile.cs:904-909`), the `ResetsHalfBrickPlacementAttempt[77]` reset
(`:59477-59482`, default-true set `TileID.cs:497` — fires even on a refused
seat), Place3x2 + the two SquareFrame tails (`:60079-60083`/`:60255-60262`),
the sub-vanilla `height<=250` guard. The ORDER stays d104's (hellforge AFTER
`hell forts` — the wall-13/14 producer `:13645` ≪ `:18187`; g37's
before-hell-forts slot never fires). Draw schedule UNCHANGED (the route
draws no RNG); +3 witnesses (Clear residue, halfBrick reset + result-is-
centre-active, 488 veto). At the g39 convergence `place_tile_hellforge`
folds into the canonical PlaceTile (§3.1).

### 2.14 Spreading Grass (`:18237`, classic arm)

| owner | evidence |
| --- | --- |
| d104 `3a101031` | `surface.rs`, W:2 (surface dirt takes grass, stone takes biome flavour); sweep 1 ±50 retypes with the sand-53 latch + evil cover veto + jungle-band 59/60 split, sweep 2 ±10 SpreadGrass(0,2,repeat:false) with flag2 latch + worldSurfaceHigh break; `spread_evil_grass` gains the C# repeat parameter; remix deferred to d107 |
| g37 B8 `df468d00` | `passes_b.rs`, W:1; 50-margin sweep + per-column spread with grassSpread<1000 recursion; remix deferred |

**KEEP d104 `3a101031`** (2 witnesses; the repeat-parameter plumbing is the
C# signature). **DROP g37 B8**. Both defer remix to d107 — unchanged.

**SEAL refinement (86 §1 B8 finding → §6.1 fix LANDED d104 `74103ae4`):**
d104's sweep-2 call originally passed `repeat:false` — the DEAD `:18339`
catch arm (SpreadGrass wraps its own try/catch `:75148`/`:75254-75256`, never
throws) instead of the LIVE `:18334` default `repeat:true` (`:75146`). The
one-argument flip is LANDED on d104: the bounded 3×3 recursion
(`grassSpread<1000` depth cap, outer column loop re-seeds) now fires;
witnesses re-derived (the old single-cell pin was stale-by-design WRONG;
+the 100-cell shelf recursion + the 1100-cell depth-cap/re-seed witnesses).
No RNG draws in the classic arm — no pinned draws move. **The merger takes
d104's tip body (`242692e1`), NOT `3a101031` bare**; 75 §2.8's "g37 wins only
Spreading Grass" is resolved (the fix landed d104-side; g37 B8 still DROPs).

### 2.15 Surface Ore and Stone (`:18447` + OrePatch `:10579` + StonePatch `:10242`)

| owner | evidence |
| --- | --- |
| d104 `19bc132b` | `surface.rs`, W:1 (patches place+replay); ore count Next(5w/4200,10w/4200), w/420 budget, beach band + [0.48,0.52] re-roll, ±200 spacing + maxOrePatch-1 into GenVars.ore_patch_x; OrePatch verbatim (tier roll, Next(2) sink, OreHelper, re-roll-per-check Next(8,13) walk, **1-in-8 double**, velocity clamps); StonePatch (199/23 veto, **186 Place3x2 + two 185 PlaceSmallPile decor**); `place_small_pile` gains the C# type parameter |
| g37 B9 `94c365ee` | `passes_b.rs`, W:1; same counts/spacing/ledger (num_ore_patch + ore_patch_x[50]), OrePatch/StonePatch + place_small_pile local |

**KEEP d104 `19bc132b`** (tie on witnesses; d104's StonePatch decor arm +
the type-parameter generalisation are the more complete C# shape, and the
body sits with the rest of d104's surface family). **DROP g37 B9**.
GenVars ore slots: keep one set (§3.4).

**SEAL refinement (75 §2.9 + §8.1 — INCORPORATED):** the stronger KEEP
evidence is the **ore-tier parameterisation**: d104's `ore_patch` takes
`copper`/`iron` from `world.ore_tiers` (the Shinies-rolled `SavedOreTiers`,
`WorldGen.cs:10581-10585`; `surface.rs:522` + `pipeline.rs:2780` feed +
the saveglue round-trip) — on a tin/lead world (166/167) d104 writes the
alternate metals; g37 B9 hardcodes 6/7 (`passes_b.rs:2361`). Confirmed
fully d104-side (75 §8.1: nothing for d104 to add). g37's `[i32;50]`
fixed-array shape argument noted; §3.4 keeps d104's Vec — no action.

### 2.16 Pyramids (`:15360` + Pyramid `:27669`) — single faithful owner

c33 `524d94a7` replaces the base shape-level 480 hull: numPyr anchor loop
consuming Dunes/Ocean-Sand PyrX/PyrY, 300 margins, dungeon-side 15% exclusion,
220 spacing + drunk halving, 151 hull + wall-34 sweep, entrance shaft, zig-zag
tunnel + chamber (Pharaoh chest style 1, 185 piles, four 91 banners, 28 pot
row), post-chamber surface tunnel. W:3 (hull, drunk spacing, chamber
furniture). **KEEP c33** — no competitor. Its `add_buried_chest` half is
superseded by d103's full core (§2.8): re-point, keep the pass.

**SEAL delta (c33 tip `5479468c`):** `3ecb8b08` fixed the pyramid loot to the
C# flag11 shape (`:36172`/`:36222-36257` — MAIN-based kind-21 style≠0
main∈{848,857,934}, the `|flag11` disjunct forces the SURFACE band at any
depth; the chamber's wall-34 chest stocks Pharaoh main + Prefix(-1) + robe;
the phantom ctx.pyramid wall-187 gate dropped). The fix rides c33's
`chest_gen.rs` — which §3.7 keeps WHOLESALE, so it lands with the stocking
carry automatically. `c7393225` added the underground-desert music arm
(Main.cs:12748-12751, runtime-only, no worldgen row). `fd7fae0e`'s
lily/cattail/bamboo arms join `ocean.rs::water_plants` on the same stream
(row 106 residual, §4).

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

**SEAL delta (f25 tip `84b90360`, research/70 graft steps 6–7):** the pass
driver stays f25's `settle_liquids.rs` WHOLESALE, but its private machinery
copies dedupe onto the g47 liquid base per research/70 (the FINAL liquid
merge order): the private `shimmer_remove_water` → the LiquidSystem arm
armed with g36 A1's `gv.shimmer_position`; the dungeon-bounds read →
`set_gen_dungeon_context` from f24's producer with the ORIGIN-cell wall fix
(`Liquid.cs:152` — f25 reads the landing cell, a real deviation 70 fixes at
the graft); the gen-local settle fns unify onto `LiquidSystem::quick_water`
(RECOMMENDED) or stay gen-local citing each other; f25's exports
(`tiles_ignore_water`/`num_liquid_total`/`clear_pending_liquid`/
`liquid_reaction_check`/`gen_ignore_water`) carry over — g47's file lacks
them. The gen-hash pins RE-DERIVE after that graft (70 §11 — record the
`edit_kills` drain choice). f25's Convert C1–C7 family + wiring closeouts +
ConvertSkyIslands are a DIFFERENT helper lane (§3.6), no overlap here.

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

**SEAL delta (c29 tip `bf90f6e7`, research/95 gem-moss):** `12a1fc97` landed
the gem-moss fidelity residuals in `spider_gem_moss.rs` — `Spread.Gem`'s
duplicate-visit redraw (`:3617` the `hashSet.Add` return IGNORED; the port's
dedup dropped real re-draws), the neonMossBiome ×4 classic fixes (TOP-edge
`num6` clamp `:9843`, SET ±5.0 drift, scaled-double step budget, exclusive
paint loops), moss arm A's reject ledger (`:17554-17566`) + the tileDungeon
exclusion (`:17545`) + arm F's LIVE column-major tileMoss sweep incl. 627
(`:17659-17698`). These are residuals of rows 68/69 (Gem Caves / Moss, base
passes) — **c29 owns them**; g41 left `spread_gem`/`pass_moss` byte-identical
to base (its research/93 handoff, verified by c29's co-lane read), d105's
#43/#44 are the DIFFERENT Random Gems/Moss Grass passes (rows 97/98) — no
overlap. The `pass_rng("Gem Caves")` stream shifts by design (the redraw
fix); pins fold into the gate re-pin.

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

**SEAL delta (g40 tip `e367fb56`, research/94):** g40 ALSO owns the
**DesertBiome entrance family** (`desert_entrances.rs`, `7bb5332e`+`a8150064`+
`e367fb56`) — the AnthillEntrance/LarvaHoleEntrance/ChambersEntrance/PitEntrance
residual closed verbatim (17 deviations: the y-major Shapes.Circle walk,
Blotches at all five sites via the shared `mini_biomes::blotched_gen_shared`
seam, SetTile=Clear(~(Wiring|Actuator)) wall-wipe ≠ ResetToType, the chambers
OuterOutline/ShapeData + tunnel chain order, the INVERTED Next(2) direction
fix, the pit SmootherStep f32-π literal) + the `DesertBiome.cs:23` entrance
gate (anniversary short-circuits BEFORE the fraction draw; extraLiquid draws
then skips; ChanceOfEntrance reads the embedded JSON 0.5 through CreateBiome).
This is a **Full Desert (row 22) residual — g40 owns it** (87 §1 receipt
confirms; g44's desert files are untouched base). The entrance Blotches
ledger lands on the Full Desert stream — SEED42_512 may move (g40's hash
notes, re-pin deferred to the gate). The DesertHive PostPlacementEffect.Smooth
residual stays g40's next slice (unclaimed beyond g40). Census correction
(g40): `ConvertGrassToDirt` does NOT exist in 1.4.5.8 — stale name, no row.

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

**SEAL delta — g41 tip `2254f2e1` (research/93/94/95, the hosts are WIRED):**
- **NotTheBees host** (`78b0cd15`, `:25107-25360` + FinishNotTheBees
  `:25388-25797`): wired at step_seed_finish's C# slot `:22435-22439`
  (between drunk and FTW); `spider_wall_remover` WIRED at the two FTW+NTB
  search loops (`:25409-25434` — the "unwired" note above is stale);
  Spread.Wall2 + OverGrownDungeon ported; **the `:25608-25629` larva tail
  rolls are g41's** (§2.20 FINAL — KEEP, flag-gated, coexists with c31's
  classic row 100).
- **FinishNoTraps host** (`d267d43f`, `:25986-26224`): wired FIRST of the
  classic tails (`:22423-22426`); heart 12→665, statue/plate-chest wire
  search + L-runs; chest 5346 add-on deferred (inventory-free).
- **FinishRemixWorld host** (`6300aab6`, `:26350-26770`): wired LAST of the
  classic tails (`:22444-22447`); d107 explicitly deferred this host —
  census-verified unclaimed, g41 took it.
- Spider-cave SPAWN pool (`c5523972`, NPC.cs:1148-1176/:1660-1678 — the
  wall-62 arm; 163/164 kinds consume-and-None until d36) + PlacePot remix
  centre veto (`d1892354`, `:54069-54072`) + QuickWater wire-contract
  activation markers (`3b626b83`, the g47 context-arming read).
- All hosts flag-gated: classic pins unchanged on the lane; the egg-sac
  re-pin deferral stands (g41's closeout `2254f2e1` — 6-step merger
  checklist, the §3.2 stalactite keep-one restated).
- **RETIRE phase (`27ba76593`→`3c6da7a3f`, research/96 — seal-3 receipt):**
  the FINAL declaration lands — ALL g41 passes + hosts + pools LANDED
  (9-piece table, 59 written-NOT-run witnesses = 14+17+7+11+1+9; both
  classic pins stale-by-design untouched). The stalactite keep-one is
  published EXACT (§3.2's 6-point FINAL INSTRUCTION, `b8ee829b8`); the
  QuickWater activation owner MOVES to the g46 RE-FOLD union `9bc722e5f`
  + the f25 dedupe (research/96 §1 — the API holds verbatim on both g47
  `5c09a68b8` and the union; the g47-only closeout shimmer witness
  `shimmer_falls_every_visit_ungated` must be carried per research/70
  §11); the spawn-pool activation hunk is published id-agnostic
  (research/96 §2 — `spawn_spider_walker` Bottom-placement alias on
  `spawn_caster` math, NOT research/93's stale `spawn_cave_walker`
  recipe; d36 163 unported, g8 164 landed `82c93cecf`, d37 238 landed
  `a341daa12`; band re-pin R7 767-816 vs d36's own 762-811 note — the
  hunk keys on NpcKind names and survives either). Retire verdict: no
  unclaimed g41 work; every residual is a named cross-lane obligation.

**SEAL delta — g42 tip `92f7eced` (research/12j tails + dispatcher):**
- Tree-type tail (`d13d05a1`): GrowTreeWithSettings gains the profile
  WallTest delegate (GemTreeWallTest for the seven gem profiles), the
  nactive() ground gate, the INVERTED side-ground flags fix, the
  Next(13)-before-Next(3) top-cap order, the errorWorld height arm;
  per-kind gem profiles (was all-RUBY); GrowPalmTree lean gate.
- Mushroom tails (`73a4e267`/`a964730f`): the 71 plant drop record, the
  grass→176 mud drops, the 227 dye-style ladder; the OVERWORLD case-70
  runtime arm (`:74704-74750`).
- **The grass-growth DISPATCHER D1–D6** (`dd659587`..`92f7eced`): the
  overworld/underground UpdateWorld_GrassGrowth routing fixed (every grass
  kind to its overworld arm — the `:74510` switch never ran before), the
  underground tail switch verbatim (Plantera-bulb/life-fruit/rose/cattail
  arms), PlaceJunglePlant + TooManyJunglePlantsNearby, the wall arms, the
  gemcorn sapling arm. Runtime-only — both hash pins stay. Residual noted:
  the inactive-cell wall arms (`:72499`/`:73809`) run for actuated cells in
  the C#; both port dispatchers return early on the inactive path
  (pre-existing structure, documented `92f7eced`, out of scope).
- Boundary: c33's worldgen PlaceCatTail copy (xorshift) stays untouched by
  g42's runtime copy (UnifiedRandom) — two sites, two streams, both keep.

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

**SEAL delta (75 §5 resolution table — INCORPORATED):** Place2xX — all three
slices (g37/d103/d104) DROP, g39 carries every arm (105/34→349, 55-wrap,
104-5tall, 207-4tall). Place1x1 — g37/d103 DROP → g39. Place3x2 — g39 H6
(d104's type-param must survive). PlaceSmallPile — keep the TYPE-
PARAMETERISED version (d104's, in spider_gem_moss.rs), g37's DROP. Place1x2
— g37/d104 DROP → g39 H1. PlaceDoor — NOT in g39's H-list: keep d103's
`place_tile_door` up/down dispatch (g44 has it too — reconcile at the gate).
PlaceChest — **graft g37's TileObject route** under d103's core (75 §2.2;
g44's `place_chest_kind` is the kind-aware form — dedupe per §2.11 SEAL).
SolidTile2 floor variants → g39 H15; the Tile-overload `solid_tile2_of`
stays with its caller. d104's `place_tile_hellforge` (§2.13) folds into the
canonical PlaceTile at this convergence.

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

**SEAL-3 receipt consumed — g41e `b8ee829b8` research/96 §3 (the
stalactite-6-point-FINAL).** The keep-one above is VERIFIED against both
owner tips and made EXACT; the gate applies the 6 points, not a fresh
adjudication:

- **d106 `e2c2609c5` still owns its half** (g41e re-read, symbol-level):
  `pass_stalac` (`spider_gem_moss.rs:774`) + the `Stalac` slot
  (`pipeline.rs:1152`/`:1428`, vanilla `:22038` after Water Plants) on ONE
  name-seeded `Streams::pass_unified("Stalac")` (`pipeline.rs:235`, called
  `:2800`); `place_tight` `pub(crate)` generic over
  `chest_stock::WorldGenRand` (`:402-404`), `check_stalactite` `:528` /
  `update_stalagtite_style` `:703` same bound. d106's `place_tight` does
  NOT validate internally (caller-runs, `:396-401` doc): its Stalac sites
  call `check_stalactite` explicitly (`:826-876`), its spider/webs sites
  don't (`:1062`/`:1592`).
- **The g41 body receipt:** chain at `spider_gem_moss.rs:441-660`
  (`invalid_tile_for_piles_or_speleothems` `:441` … `check_stalactite`
  `:566`), validator wired INSIDE `place_tight`'s tail (`:422-424`); moss
  reads the GENERATED `tiledata::TILE_MOSS` (`sets.rs:44`, 627 included —
  `Main.cs:7031` re-read, the 11 members 179/381/534/536/539/625/627/180/
  181/182/183) vs d106's local `MOSS_TILES`+`NEON_MOSS`+381 lists (same
  members — dedupe onto the generated set).

The 6-point FINAL INSTRUCTION (research/96 §3, verbatim scope):
1. KEEP g41's chain body + frame table + the 14 egg-sac witnesses + the
   `place_tight`-INTERNAL validator call.
2. GRAFT d106 R4's signatures over it: `rng: &mut impl
   super::chest_stock::WorldGenRand` on `place_tight`/`check_stalactite`/
   `update_stalagtite_style` (the trait + both impls already exist on g41,
   `chest_stock.rs:185-213` — Xorshift64 AND UnifiedRandom); swap the two
   `rng.range(0, n)` draws in `place_tight` for `rng.next(n)`; make
   `place_tight`/`check_stalactite` `pub(crate)`. **Do NOT adopt d106's
   caller-runs convention — the d106-double-validation-trap: the d106
   Stalac sites DROP their explicit `check_stalactite` calls when g41's
   internal validator is kept, or the `Next(3)` mismatch draws double**
   (d106's sites were written against ITS unvalidated base).
3. KEEP d106's `pass_stalac` + slot + `pass_unified("Stalac")` stream
   wholesale; its gem-tree scatter calls g42's trees.rs (row 107).
4. DROP g36's local `shimmer.rs` chain (`check_stalactite` `:699` + the
   island-arm tail calls `:330`/`:335`); rewire onto the canonical
   `place_tight` (draw-schedule exception clause stands).
5. ONE moss set — `tiledata::TILE_MOSS` (generated, never hand-edit);
   delete d106's local lists at the graft.
6. Re-pin AFTER the graft, sequentially (research/93 §FINAL STATUS step 2;
   d106 R1–R5 pins move by design, g41's egg-sac pins move at the same
   gate). RNG-neutral for the spider pass by construction — same draws,
   same order (d106 proved byte-identity).

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

**SEAL cross-ref (research/70 @ g47 `5c09a68b` — the FINAL liquid merge
order governs the SIM side):** g47 is the structural base for `liquid.rs`
(the only complete runtime surface + the WIRED load hook); the g46c/g46d
graft union takes g47's bodies wholesale, then g46's evap/nudge + g45's
extra wakes + g45b's AddWater guards graft on. **This matrix's f25-core
verdict composes with 70's graft step 6:** f25's gen driver stays wholesale,
its private copies DELETE onto the g47 surfaces (`set_gen_shimmer_position`
/ `set_gen_dungeon_context` / `LiquidSystem::quick_water`), the ORIGIN-cell
wall fix applies, f25's exports carry over (§2.19 SEAL delta). The
`Liquid.LiquidCheck` merge sweep at Final Cleanup `:22473-22483` rides
`LiquidSystem::liquid_check(..., create_merge_tiles_during_gen: true)` on
the g47 base — the compose-hunk §6.4 names the seam; the merger makes the
sweep callable (a pub wrapper) at the graft. d106 R2's pass re-bases onto
the SAME post-graft surface (not onto f25's pre-graft private copies).

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

### 3.6 Convert / growth helper family: f25 C1–C7 + wiring — no g39 overlap

f25's `f9c3dba7` family (Convert types 2/5/6/7/8/9/10, TileIsExposedToAir,
BlockBelowMakesSandConvertIntoHardenedSand, IsTileATreeBranch/Root,
ConvertTreeAndGround) is the Convert/growth side, NOT the PlaceTile side —
**both keep** (75 §5 verdict). SEAL deltas at f25 tip `84b90360`: the spray
wiring (`cffdc1c7` — ai_spray type 2→C3, 5/6/7→C4; water_death holy arm;
spread_grass 109 dispatch), the tree wiring (`e76c077d` — C7 at
try_kill_invalid_tree_above), the moss wiring (`b004f4f1` — case-184
check_long_moss), the seven solution-spray proj rows (`5e83d3ec`),
ConvertSkyIslands (`f0fe0f38`, the anniversary `:24243` call — g44's islands
lane marked it unported, f25 owns it), the powder census (`84b90360` — d1's
aiStyle-6 machine is the owner, f25's cells 2/5/6/7/8/9/10 disjoint from
d1's 11/1/4; the merger dedupes d1's CONV_THORN constant copy). d104's
spread_evil_grass repeat-param (§2.14) composes with these — the grass
conversion rides `spread::convert_*`. Types 1/3/4 stay no-ops until d1/g42
cells land (f25's own note).

### 3.7 Chest stocking: c33's chest_gen.rs WHOLESALE (the superset)

Per research/120 §4 (c33 `3ecb8b08`/`5479468c`): d103's `chest_gen.rs` copy
(the pre-sweep base) carries phantom rows — the either-or 4425/4460
mahogany/sky rows (nowhere in the C#), the wall-probe ctx.pyramid gate (the
real chamber is wall 34), weapons rolled for preset-main chests, the
phantom mid/deep rows. c33's sweep re-derived the whole block to
`WorldGen.cs:35871-37597`: flag5 TWO independent 1/2 rolls (`:36515-36525`/
`:36845-36855`), flag11 MAIN-based + the surface-at-depth disjunct, the
preset-main weapon skips (the num10==0 else-branch shape), every slot-0
main Prefix(-1) (`:36232/:36513/:36808/:37167`), the deep ladder re-derivation
(Next(7) pick FIRST, lavaLine AND, FlareGun case 6, prefixes ONCE), the
flag9 shadow-key/ram-rune once-per-world fairness + flag10 locked-biome
skip, the mid flare unconditional, the 3093/4345 bonus stacks, the flag8
extras + tungsten 4915, the recall/torch/life deep rows, the Cloud stack in
BOTH kind blocks + the flag13 temple gate, the rolled-ore-tier bar rows,
the 832 LeafWand companion; lava_line + ore_tiers thread through
ChestSiteEvent (World.lava_line published by the terrain step).
**Merger: take c33's `chest_gen.rs` block WHOLESALE; take d103's
`water_chests.rs` + `buried_chest.rs` + wiring; the shared ChestSiteEvent
seam composes with no signature change** (120 A17). g44's sky loot fixes
(`0898f352` — the sky main Prefix + CreativeWings rider) are the same arms
c33's sweep carries — verify once at the gate, keep one copy. The 10
census witnesses + 4 new gate witnesses per 120 §5.

### 3.8 Seed-finish hosts: g41 owns the three, the Final Cleanup tail composes

The Final Cleanup pass's C# tail (`:22423-22488`) splits across lanes:
**g41** owns FinishNoTraps (`:22423-22426`), NotTheBees+FinishNotTheBees
(`:22435-22439`), FinishRemixWorld (`:22444-22447`) — all WIRED at
`step_seed_finish` (§2.23 SEAL delta); base/d107 own finish_drunk_gen /
finish_get_good_world / finish_tenth_anniversary_world + the
FinalizeSecretSeeds arms (micro_finish/micro_surface, `:22467`); **g36**
owns ShimmerCleanUp (`:22472`, §6.4 compose-hunk); **f25/d106-on-g47** own
the LiquidCheck merge sweep (`:22473-22483`, §3.3). Still-unported tail
arms (census at the seal): ShimmerifySideOfWorld (`:22448-22452`),
PaintTheSandRainbowAndSandWallsBlack (`:22453-22456`), the vampire-seed
block (`:22457-22466`), FinishTenthAnniversaryWorldPaintTreesAndMushrooms
(`:22468-22471` — no port on any read tip), ExtraSpawnPointManager /
TownNPCPositionsCleanup / AnglerQuestSwap (`:22484-22488`). These are
g37's cited deferrals + d107's seed scope — flag them at the gate; none
blocks the classic chain. **Ordering obligation:** the C# runs the whole
tail INSIDE Final Cleanup AFTER the dirt-throw scatter; the port splits it
into `final cleanup` → `world tail` → `seed finish` steps. The compose-hunk
(§6.4) places shimmer_clean_up + the merge sweep at the END of
`step_seed_finish` (after the hosts + FinalizeSecretSeeds) so the relative
order matches the C#; `world tail`'s moon/hell draws are RNG-isolated
(own streams) — the split is stream-neutral, recorded as the executed-order
deviation.

## 4. Single-owner passes (no contest — KEEP as landed)

| pass(es) | owner | tip evidence |
| --- | --- | --- |
| Sand Patches, Mount Caves, Dirt Wall Backgrounds, Clay, Small Holes, Dirt Layer Caves, Rock Layer Caves, Generate Ice Biome (+Granite row) | d101 | `passes1.rs` W:11 + `ice_biome.rs` W:2; 9/9 ledger |
| Silt, Webs, Slush, Mountain Caves, Gems | d102 | `passes2.rs` W:23 total (per-pass 2–3 each); producer/consumer GenVars seams with d101 |
| Oasis (+awakened cacti block), Waterfalls | d103 | `90b7eee6`/`561b3ab2`; oasis ledger + PlaceOasis; both liquid-edge sweeps |
| Traps, Piles, Spawn Point | d104 | `traps.rs` W:2 (+witness-fix `341f4280`), structures W:2, surface W:1; g38 claimed, never landed |
| Guide, Sunflowers, Herbs, Dye Plants, Flowers, Mushrooms, Random Gems, Moss Grass | d105 | `starter_npcs.rs` W:9, `surface.rs` W:18, `spider_gem_moss.rs` #43 W:3 + #44 W:3; 9/9 ledger (Larva #45 yielded to c31, §2.20) |
| Grass Wall, Settle Liquids Again (pass), Tile Cleanup, Stalac (pass), Remove Broken Traps | d106 | R1–R5 landed (`surface.rs` W:4, `liquids.rs` W:6, `cleanup.rs` W:3, `spider_gem_moss.rs` W:22); R6 Final Cleanup yielded to g37; machinery dedupe per §3.2/§3.3 |
| Final Cleanup (pass body) | g37 gap-fill | `4ced188f9`→`81fd1e0d` (PUSHED at the seal), `final_cleanup.rs` W:5 + positional; ShimmerCleanUp arm → g36 (§6.4 compose-hunk) |
| Settle Liquids (pass + core) | f25 | §2.19 (+ the Convert C1–C7 family §3.6; machinery dedupes onto g47 per research/70) |
| Skyblock (gated) | f24 | §2.24 |
| Dither Snake (gated) | f24 | §2.18 (DROP c30) |
| Pyramids (faithful) + lily/cattail/bamboo arms + chest_gen stocking superset | c33 | §2.16/§3.7 (DROP its Water Chests pass only) |
| Hives (faithful) + Larva | c31 | §2.20 FINAL (DROP d105 #45's Larva slice; g41's NTB tail is a DIFFERENT site, keeps) |
| Micro Biomes dead-man's-chest block + gem-moss fidelity | c29 | §2.21 (+ Spread.Gem/neonMoss/moss-arm residuals, research/95) |
| Marble/Granite residuals + DesertBiome entrance family | g40 (+d101 row) | §2.22 (research/94) |
| Spider Caves residuals + validator chain + NTB/FinishNoTraps/FinishRemixWorld hosts + spider spawn pool | g41 | §2.23/§3.8 (research/93/94/95) |
| Mushroom-biome residuals + GrowTree + tree-type tails + grass-growth dispatcher | g42 | §2.23 (research/12j) |
| Floating Islands + Floating Island Houses + skyLakes + buried-chest graft + housing wall-table | g44 | §2.11 FINAL (DROP d103/d104/g37; research/87 FINAL-KEEP) |
| PlaceTile-wrapper helper family | g39 | §3.1 |
| Shimmer + ShimmerCleanUp tail arm | g36 A1 + gap-fill | §2.1 — Shimmer is g36's surviving pass; `shimmer_clean_up` (the Final Cleanup :22472 arm, unwired) composes per §6.4 |

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
| 22 | Full Desert | :12500 | base + **g40** | g40's DesertBiome entrance family residual (§2.22 SEAL delta, research/94) |
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
| 66 | Water Chests | :17245 | **d103** | DROP g37 B3 + c33 underwater; carry c33's WHOLE chest_gen block (§3.7); fix the extraLiquid double-draw (120 A2); NO inner-loop cap (120 A13/A14 — the outer 10000 caps already present) |
| 67 | Spider Caves | :17356 | base + **g41** | egg-sac validator + web/wall residuals |
| 68 | Gem Caves | :17428 | base + **c29** | Spread.Gem duplicate-visit redraw fix (§2.21 SEAL delta) |
| 69 | Moss | :17472 | base + **c29** | neonMossBiome ×4 + arm A ledger/tileDungeon + arm F live tileMoss sweep (§2.21 SEAL delta) |
| 70 | Temple | :17701 | base | temple part 2 |
| 71 | Cave Walls | :17711 | **d103** | DROP g37 B4 |
| 72 | Jungle Trees | :17840 | base | |
| 73 | Floating Island Houses | :17859 | **g44** | DROP d103/d104/g37 B5; drunk door arms VERIFIED PRESENT (87 receipt — §2.11 FINAL); chest-core dedupe onto the kind-aware survivor |
| 74 | Quick Cleanup | :17875 | **d104** | DROP g37 B6 |
| 75 | Pots | :17996 | base | |
| 76 | Hellforge | :18187 | **d104** | DROP g37 B7; take d104's tip body incl. the LANDED PlaceTile-77 graft `574f1d58` (§2.13 SEAL) |
| 77 | Spreading Grass | :18237 | **d104** | DROP g37 B8; take d104's tip body incl. the LANDED repeat:true fix `74103ae4` (§2.14 SEAL); remix → d107 |
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
| 100 | Larva | :20868 | **c31** | §2.20 FINAL: DROP d105 #45's same-site slice; PORT its skyblock gate onto c31's step; g41's `:25608-25629` NTB tail is a DIFFERENT site — KEEP both routes (vanilla fires both on NTB) |
| 101 | Micro Biomes | :20901 | base + **c29** | dead-man's-chest block leads the micro group; thin ice/campsites/sword/mining/mahogany already base |
| 102 | Settle Liquids Again | :21246 | **d106 R2** | pass kept, machinery re-based on f25 (§3.3) |
| 103 | Cactus, Palm Trees, & Coral | :21310 | base | + d103 oasis block + c33 lily/cattail/bamboo arms |
| 104 | Tile Cleanup | :21520 | **d106 R3** | |
| 105 | Lihzahrd Altars | :21943 | base | |
| 106 | Water Plants | :21973 | base + **c33** | lily/cattail/bamboo arms `fd7fae0e` (PlaceLilyPad/PlaceCatTail/GrowCatTail/PlaceBamboo, the d103 deferral closed); g42's runtime dispatcher copies are the OTHER site (§2.23) |
| 107 | Stalac | :22038 | **d106 R4** | validator chain unified with g41 (§3.2); gem-trees call g42's trees.rs |
| 108 | Remove Broken Traps | :22121 | **d106 R5** | |
| 109 | Final Cleanup | :22141 | **g37 gap-fill** | landed `4ced188f9`→`81fd1e0d` (PUSHED at the seal): wall-hole fill + whole-map sweep + 668 dirt-throw scatter, pass_rng(Final Cleanup), W:5+positional; ShimmerCleanUp arm (:22472) → g36 `shimmer.rs::shimmer_clean_up` — compose-hunk §6.4; liquid-merge sweep (:22473-22483) = the g47 LiquidSystem seam (§3.3); seed-tail hosts = g41 (§3.8) |

## 5a. FINAL EXECUTED-order union (all 120 steps, single owner each)

§5 is the vanilla REGISTRATION order (the 109 `VANILLA_CHAIN` decls). This
table is the **merged EXECUTED order** — the port's grouped chain the
count/positional witnesses assert: research/75 §4's 119-step union + Final
Cleanup (75 §8.4's "→120") = **120 steps**, re-verified at the seal against
g37's pushed `81fd1e0d` pipeline (68 single-line + 8 multi-line `step!`s =
76 steps on that branch — its own count witness; the union folds in every
other lane's inserts). Every step has exactly ONE owner; `base` = executed
at a725acb0.
The load-bearing group constraints (75 §4 ordering notes) are restated
below the table — they survive the seal unchanged, with the two d104-side
corrections (hellforge slot, larva owner) applied.

| # | step | owner | # | step | owner |
| --- | --- | --- | --- | --- | --- |
| 1 | terrain height map | base | 61 | hell forts | base |
| 2 | terrain fill | base | 62 | hellforge | d104 |
| 3 | plan biome spans | base | 63 | spreading grass | d104 |
| 4 | skyblock (gated) | f24 | 64 | surface ore and stone | d104 |
| 5 | dunes | base | 65 | place fallen log | d104 |
| 6 | ocean sand | base | 66 | traps | d104 |
| 7 | micro grass | base | 67 | piles | d104 |
| 8 | jungle | base | 68 | spawn point | d104 |
| 9 | mud caves to grass | base | 69 | lihzahrd temple | base |
| 10 | full desert | base+g40 | 70 | temple part 2 | base |
| 11 | mushroom patches | base+g42 | 71 | cave walls | d103 |
| 12 | sand patches | d101 | 72 | statues | d103 |
| 13 | cave fields | base | 73 | cave houses | base |
| 14 | mount caves | d101 | 74 | dead man's chests | c29 |
| 15 | dirt wall backgrounds | d101 | 75 | thin ice | base |
| 16 | marble | d101+g40 | 76 | campsites | base |
| 17 | granite | d101+g40 | 77 | spike pits | base |
| 18 | stone-in-dirt blobs | base | 78 | sword shrines | base |
| 19 | dirt-in-stone blobs | base | 79 | mining explosives | base |
| 20 | clay | d101 | 80 | mahogany trees | base |
| 21 | small holes | d101 | 81 | webs and honey | base |
| 22 | dirt layer caves | d101 | 82 | mushroom and jungle plants | base+g42 |
| 23 | rock layer caves | d101 | 83 | jungle plants | base |
| 24 | surface caves | base | 84 | vines | base |
| 25 | wavy caves | base | 85 | flowers | d105 |
| 26 | generate ice biome | d101 | 86 | mushrooms | d105 |
| 27 | dirt to mud | base | 87 | random gems | d105 |
| 28 | silt | d102 | 88 | moss grass | d105 |
| 29 | ash blobs | base | 89 | muds walls in jungle | base |
| 30 | ores and gems | base | 90 | larva | c31 |
| 31 | webs | d102 | 91 | floating islands | g44 |
| 32 | evil | base | 92 | settle liquids | f25 |
| 33 | liquids | base | 93 | floating island houses | g44 |
| 34 | dither snake (gated) | f24 | 94 | quick cleanup | d104 |
| 35 | slush | d102 | 95 | remove water | base |
| 36 | dungeon | base | 96 | oasis | d103 |
| 37 | constant lava layer | base | 97 | shell piles | base |
| 38 | mountain caves | d102 | 98 | slopes | base |
| 39 | beaches | base | 99 | living trees | base |
| 40 | gravitating sand | base | 100 | grass wall | d106 |
| 41 | gems | d102 | 101 | guide | d105 |
| 42 | ocean caves | base | 102 | sunflowers | d105 |
| 43 | shimmer | g36 A1 | 103 | waterfalls | d103 |
| 44 | clean up dirt | d102 | 104 | wall variety | d103 |
| 45 | fragile ice | base | 105 | life crystals | d103 |
| 46 | underground jungle trees | base | 106 | wood tree walls | d102 |
| 47 | exposed gems | base | 107 | trees | base+g42 |
| 48 | jungle shrines | base | 108 | herbs | d105 |
| 49 | chests | base | 109 | dye plants | d105 |
| 50 | surface chests | d103 | 110 | surface flora | base |
| 51 | water chests | d103 | 111 | settle liquids again | d106 |
| 52 | spider caves | base+g41 | 112 | cacti | base+d103+c33 |
| 53 | gem caves | base+c29 | 113 | tile cleanup | d106 |
| 54 | moss | base+c29 | 114 | lihzahrd altar | base |
| 55 | altars | base | 115 | water plants | base+c33 |
| 56 | wet jungle | base | 116 | stalac | d106 |
| 57 | pyramids | c33 | 117 | remove broken traps | d106 |
| 58 | dirt rock wall runner | d102 | 118 | final cleanup | g37 (+g36 arm §6.4) |
| 59 | pots | base | 119 | world tail | base |
| 60 | beehives | c31 | 120 | seed finish | base+g41 hosts (§3.8) |

**Seal corrections vs 75 §4's table:** the hellforge slot is FIXED in-place
(75 had hellforge at 60 BEFORE hell forts at 68 — the never-fires order 86
§6.2 rejected; this table moves beehives to 60, hell forts to 61, hellforge
to 62, d104's own pipeline order); row 10 full desert gains g40's
entrance residual (§2.22); rows 53/54 gain c29's gem-moss fidelity (§2.21);
row 90 larva = c31 (75 already had c31 — the 86 §6.3 d105 verdict is
superseded, §2.20 FINAL); row 112 cacti gains the d103 oasis block + c33
lily/cattail/bamboo arms (row 103/106 residuals); row 118 final cleanup
inserted before world tail (75 §8.4); row 120 seed finish carries g41's
three WIRED hosts + the §6.4 compose-hunk at its tail.

**Load-bearing constraints (the positional witnesses the gate keeps):**
- Chest family group: `chests → surface chests → water chests → spider
  caves` (g37 `809a61a8`; `test_DW_5_6` reads spider == water+1). 86 §4's
  "restore the raw anchor order" alternative is RNG-equivalent (isolated
  streams) — the merged chain keeps the grouping; the Lead may flip to
  anchor order only with the positional witnesses re-derived.
- Statues/cave-walls group after `temple part 2`, before `cave houses`
  (`statues + 1 == cave houses`).
- **Hellforge AFTER `hell forts`** (86 §6.2 — the wall-13/14 producer
  `:13645` ≪ `:18187`; g37's before-hell-forts slot never fires). APPLIED
  in this table: beehives 60 → hell forts 61 → hellforge 62 → spreading
  grass 63 (d104's own pipeline order). The pots-adjacent grouping of
  spreading grass/surface ore is otherwise RNG-neutral (own streams).
- Island houses + quick cleanup after `floating islands` (g44 producer),
  before `remove water`.
- `trees` after `wood tree walls`; `herbs`/`dye plants` after `trees`;
  `larva` after `muds walls in jungle` (c31's own slot witness).
- `final cleanup` after `water plants`, before `world tail`; the §6.4
  shimmer/liquid-merge arms compose at `seed finish`'s tail (§3.8's
  ordering obligation).
- Count witness: `the_executed_pipeline_is_59_named_steps` → **120**.
- Hash pins: SEED42_512 + DEFAULT_WORLD re-measured ONCE on the merged
  120-step chain; every lane's isolated-insertion "pins unchanged" argument
  does NOT compose (75 §6 discipline); a==b halves stay green throughout.

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
   (advanced to `81fd1e0d`, **PUSHED at the seal** — three witness fixes
   `9ea5db8a`/`7db8900f`/`81fd1e0d`). g37's claim probe explicitly read g36's
   tip `6d46f6a5` row 109 "GAP — unlanded" + d106 `e2c2609c` R6 unchecked →
   unclaimed → g37 fills
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
   `pass_beehives` larva-ledger edits at the merge; KEEP c31.** §2.20's FINAL
   adjudication (seal) answers the 86-§6.3 challenge: the both-survive pair is
   `:20868` (c31) vs `:25608-25629` (g41's NTB tail) — NOT c31 vs d105; d105's
   ~half-the-hives anchor roll has no C# counterpart (HiveBiome.cs:85 is
   unconditional). Row 100 = c31 FINAL.
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

1. **Foundations**: g39 helpers (§3.1) → f24 A1 gate tables (§3.5) → the g47
   liquid base + the research/70 graft (the g46 RE-FOLD union `9bc722e5f` —
   the re-base is DONE per research/96 §1, supersedes the stale-union read →
   g45b guards already folded → f25 gen-module dedupe, §3.3) → g41+d106
   stalactite chain (§3.2's 6-point FINAL INSTRUCTION — g41 body + internal
   validator KEPT, d106 signatures grafted, the d106 Stalac sites DROP their
   explicit check calls) → g42 trees.rs.
   These are the shared bodies every kept pass rewires onto.
2. **Producers**: d101 (mCave/snow anchors, Granite row) → g36 A1 Shimmer
   (shimmer_position) → g44 islands ledger → d104 GenVars ore slots.
3. **Pass waves in vanilla order**: d101/d102 early passes → c33 Pyramids →
   d103 mid passes (Oasis..Cave Walls, chest core first) → g44 houses → d104
   late passes → d106 R1 → d105 plants/NPC/gems/moss (Random Gems #43, Moss
   Grass #44) → c31 Larva (DROP d105 #45's Larva, §2.20 FINAL) → c29 micro
   block + gem-moss residuals → d106 R2–R5 tail → **g37 Final Cleanup**
   (`final_cleanup.rs`, the last classic pass) → g41's seed-finish hosts →
   **the §6.4 compose-hunk** (g36's `shimmer_clean_up` at `:22472`, then the
   `Liquid.LiquidCheck` merge sweep at `:22473-22483` on the g47 seam). f24's
   gated steps slot at :11511/:14755 (classic pins unchanged by construction).
   The full 120-step executed order is §5a.
4. **DROP list execution** (per branch, surgical — never whole-branch reverts
   where the branch also carries KEEP rows):
   - d102: drop shimmer.rs + Shimmer row ONLY (keep Silt/Webs/Slush/Mountain
     Caves/Gems/Clean Up Dirt/Dirt Runner/Wood Walls).
   - g36: drop A2/A3/A4/A6/A7 blocks (micro_finish/living_trees/structures);
     keep shimmer.rs + its GenVars publication + the gap-fill `shimmer_clean_up`
     arm (unwired; the merger composes it per §6.4).
     g36 contributes ONE pass (Shimmer) + the ShimmerCleanUp tail arm.
   - g37: drop B1–B9 (passes_b.rs + islands.rs edits + pipeline rows) BUT KEEP
     the Final Cleanup gap-fill (`final_cleanup.rs` + its pipeline row + the
     `pass_rng("Final Cleanup")` stream); g37 contributes the Final Cleanup pass
     (the B-row value is the census split + the 512-wide statue proof + the
     witness set the gate may port). Compose g36's `shimmer_clean_up` per §6.4.
     The B7 PlaceTile graft already landed d104-side (`574f1d58`) — research/124
     is superseded, no graft action remains.
   - d105: drop the #45 Larva step + `structures.rs::pass_larva`/`place_larva`
     + the `pass_beehives` larva-ledger edits + the `GenVars.larva` Vec slot
     (§2.20 FINAL, c31's LarvaAnchors is canonical); PORT its skyblock denyAll
     step gate onto c31's `step_larva` first; KEEP #43 Random Gems + #44 Moss
     Grass.
   - d103: drop island_houses.rs + its islands.rs ledger edits ONLY; fix the
     Water Chests extraLiquid double-draw (§2.9 SEAL); its chest_gen.rs copy
     is REPLACED by c33's superset (§3.7); the buried_chest core dedupes onto
     the kind-aware survivor (§2.11 SEAL).
   - d104: drop island_houses.rs + its islands.rs edits ONLY; take the tip
     bodies for Hellforge (`574f1d58` graft) + Spreading Grass (`74103ae4`
     fix).
   - g38: drop C1 (surface.rs fallen-log block + pipeline row); port its 3
     witnesses onto d104's body first.
   - c30: drop dither_snake.rs + pipeline gate; verify f24 3-seam coverage first.
   - c33: drop pass_underwater_chests ONLY (keep Pyramids + the WHOLE
     chest_gen stocking block §3.7 + lily/cattail/bamboo + the music arm).
   - d106: dedupe liquid/tileobject machinery against the post-graft g47/f25
     surface (§3.3 + research/70); dedupe the stalactite chain against g41
     (§3.2).
   - d118/d119: dedupe `housing::is_house_wall` onto g44's full-table read
     (§2.11 SEAL item 6 — predicate-only, their witnesses stay valid).
5. **Gate discipline** (every lane's contract): hash re-pins were deferred —
   the merge gate runs ALL witnesses (every lane wrote them NOT run), measures
   the new SEED42_512 / DEFAULT_WORLD constants ONCE on the fully merged chain,
   and keeps the `a == b` determinism halves green throughout. Per-lane
   "pins unchanged" claims (e.g. g37 B1's 512 beach-band proof, d102's
   sub-vanilla valves) are arguments about ISOLATED insertions — they do not
   compose; only the gate's measured re-pin is authoritative. The liquid-side
   re-pin table is research/70 §11 (the f25 gen-hash RE-DERIVES after the
   graft — record the edit_kills drain choice).

### 6.3 Residual-arm verification checklist (KEEP versions to re-read at the gate)

- ~~g44 IslandHouse: drunk crimson wall split + case-10 door dispatch~~
  **CLOSED** (87 receipt: both present, islands.rs:1199-1202/:1011-1027).
- ~~d103 Water Chests: unbounded inner loops — add g37's 10000-cap guard~~
  **CORRECTED** (120 A13/A14: the outer caps are present; the inner hunt is
  the C#'s own unbounded shape — do NOT cap). **NEW**: fix the extraLiquid
  treasure-main double-draw (120 A2) + the four new gate witnesses (120 §5).
- d103 Wall Variety: shimmerPosition rewire (§2.5) — currently zero-init.
- f24 dither wire: GenerationRunner.advance + total_weight seams (§2.18).
- g39 Place2x2: c29's nactive()/tileTable fixes present? (§3.1).
- g39 Place3x2 vs d104's type-parameter generalisation (§3.1); PlaceDoor
  dispatch reconcile (75 §5 — not in g39's H-list).
- d104 Place Fallen Log: g38's frame/veto/gate witnesses ported (§2.17).
- f25 ShimmerRemoveWater: upgrade the None-guard stand-in to g36's published
  shimmer_position (§3.3 — research/70 graft step 6 does this via
  `set_gen_shimmer_position`; the ORIGIN-cell wall fix rides the same graft).
- c31 step_larva: the skyblock denyAll gate ported (§2.20 FINAL).
- g44 buried_chest vs d103 buried_chest: ONE kind-aware core (§2.11 SEAL);
  the sky loot Prefix/CreativeWings arms verified once against c33's
  chest_gen superset (§3.7).
- The §6.4 compose-hunk applied + its witnesses run.
- The §3.2 stalactite graft per the 6-point FINAL INSTRUCTION (research/96
  §3): d106 Stalac sites' explicit `check_stalactite` calls DROPPED (the
  double-validation trap — double `Next(3)` mismatch draws), one generated
  `tiledata::TILE_MOSS` set, g36 local chain dropped, re-pins sequential.
- The g41 QuickWater activation (research/96 §1): against the g46 RE-FOLD
  union `9bc722e5f`+, arm the gen context ONCE at the driver head, swap the
  three `finish_remix.rs` seams (`:379`/`:405`/`:445`) for
  `quick_water(world, -1, -1)`, restore the spawn-box while-flag shape,
  re-derive the REMIX pins ONLY; carry the g47-only
  `shimmer_falls_every_visit_ungated` witness + the research/70 §11 dedupes.
- The g41 spawn-pool hunk (research/96 §2) applied when NpcKind 163 (d36) +
  164 (g8) both exist on main — `spawn_spider_walker` Bottom alias, NOT
  `spawn_cave_walker`; the two consume-and-None pins flip to kind/position
  assertions, the draw-order pins hold.
- The 5-row g41 anchors canary survives the merge (`anchors.txt:899-903`
  region per research/96 §4 step 6: `fn check_stalactite`,
  `finish_not_the_bees`, `finish_no_traps`, `finish_remix_world`,
  `fn set_spawn_spider_flag`) — a failed marker = lossy merge, recover the
  arm, never delete the row.
- The unported Final Cleanup tail arms censused (§3.8): ShimmerifySide /
  RainbowSand / vampire / anniversary-paint-trees / ExtraSpawns / TownNPC
  cleanup / AnglerQuestSwap — flag at the gate, none blocks the classic chain.

### 6.4 The ShimmerCleanUp compose-hunk (EXACT — for the g37 branch / the merger)

The C# anchor (`WorldGen.cs:22468-22483`, re-read at the seal):

```csharp
if (Main.tenthAnniversaryWorld)
{
    WorldGen.FinishTenthAnniversaryWorldPaintTreesAndMushrooms();   // :22470 — UNPORTED (§3.8)
}
WorldGen.ShimmerCleanUp();                                          // :22472 — g36's arm
for (int num21 = 0; num21 < Main.maxTilesY; num21++)                // :22473 — y OUTER
{
    for (int num22 = 0; num22 < Main.maxTilesX; num22++)            // :22475 — x INNER
    {
        Tile tile8 = Main.tile[num22, num21];
        if (tile8.liquid > 0)
        {
            Liquid.LiquidCheck(num22, num21, tile8.liquidType(),
                               createMergeTilesDuringGen: true);    // :22480
        }
    }
}
```

**Where it lands.** The port splits the Final Cleanup tail across steps
(§3.8): g37's `final cleanup` step carries `:22143-22422` (wall-hole fill +
sweep + scatter); g41's `step_seed_finish` carries the `Finish*` hosts +
the FinalizeSecretSeeds arms (`micro_finish::run` + `micro_surface::run`,
`:22423-22467`). The `:22472` arm therefore composes at the **TAIL of
`step_seed_finish`** — immediately after `micro_surface::run(world, streams,
opts);` (g41 `pipeline.rs`, the last statement of the fn) — NOT inside
`pass_final_cleanup` (whose `:22423-22488` deferral comment stays accurate
for the pass body; update it to point here). The relative order then
matches the C#: hosts → FinalizeSecretSeeds → ShimmerCleanUp → merge sweep.
(`world tail` sits between the two steps in the port's grouped chain; its
moon/hell draws are RNG-isolated — the recorded executed-order deviation,
§3.8.) The unported `:22470` anniversary-paint call would sit immediately
BEFORE this hunk if/when d107 lands it.

**The hunk (append to `step_seed_finish`, g41's body — the tip `3c6da7a3f`
signature already names `gv: &mut GenVars`, no rename needed; the tail
statement is `micro_surface::run(world, streams, opts);` at `pipeline.rs:1472`):**

```rust
    // WorldGen.cs:22472 — ShimmerCleanUp (g36's gap-fill arm, composed here).
    // Reads g36 A1's published GenVars.shimmerPosition (the zero-vector
    // sentinel is handled inside: None / (0,0) → no-op). Draws on the Final
    // Cleanup stream — the C# genRand at this point IS the Final Cleanup
    // pass's re-seed (WorldGenerator.cs:497), the same stream the scatter
    // drew on; pass_rng is name-mixed so the two calls share one sequence
    // ONLY if the merger threads ONE Rng through — see the stream note.
    shimmer::shimmer_clean_up(
        world,
        gv.shimmer_position,
        opts.flags.contains(crate::seeds::SeedFlags::TENTH_ANNIVERSARY),
        &mut streams.pass_rng("Final Cleanup"),
    );
    // WorldGen.cs:22473-22483 — the Liquid.LiquidCheck merge sweep, y-outer
    // x-inner, liquid>0 cells only, createMergeTilesDuringGen: true. Rides
    // the g47 LiquidSystem (research/70 base): liquid_check is private on
    // LiquidSystem — the merger adds the pub gen wrapper at the 70-graft:
    //     pub fn gen_merge_sweep(&mut self, world: &mut World) {
    //         for y in 0..self.height { for x in 0..self.width {
    //             let t = world.tile(x, y).copied().unwrap_or_default();
    //             if t.liquid > 0 {
    //                 self.liquid_check(world, x, y, t.liquid_type(), true);
    //             }
    //         }}
    //     }
    // armed with set_gen_settle_context(water_line, lava_line, remix) +
    // set_gen_shimmer_position(gv.shimmer_position) (the 70 graft step 7).
    let mut sys = crate::liquid::LiquidSystem::new(world);
    sys.set_gen_settle_context(
        gv.water_line,
        gv.lava_line,
        opts.flags.contains(crate::seeds::SeedFlags::REMIX),
    );
    sys.set_gen_shimmer_position(gv.shimmer_position);
    sys.gen_merge_sweep(world);
    // edit_kills drain choice: RECORD it (research/70 §11) — the gen-time
    // kills queue TileKill records; resolve-or-discard must be pinned once.
```

**Stream note (the one fidelity trap).** `Streams::pass_rng(name)` returns
a FRESH `Rng(seed ^ fnv1a64(name))` per call (g41 tip `3c6da7a3f`
pipeline.rs:226-228) — calling it
twice for "Final Cleanup" (scatter in `step_final_cleanup`, jitter here)
would REPLAY the scatter's draws instead of continuing the sequence. The
C# `genRand` is ONE UnifiedRandom re-seeded per pass; the scatter
(`:22379-22422`) and the ShimmerCleanUp outer-ellipse jitter (`:34753`)
draw on the SAME pass instance. The merger must either (a) thread the
scatter's `Rng` out of `pass_final_cleanup` (return it / stash it in
GenVars) and hand the SAME instance to `shimmer_clean_up`, or (b) extend
Streams with a checked-out per-pass stream (`pass_rng_continue`) — (a) is
the minimal edit: `pass_final_cleanup` already owns `rng: &mut Rng`; change
`step_final_cleanup` to stash it (`gv.final_cleanup_rng = Some(rng)`) and
`step_seed_finish` to take it back. The hunk above shows the naive form
for readability; **(a) is REQUIRED for stream fidelity** — flag it in the
gate witness (below). On a classic world with no shimmer placed
(`gv.shimmer_position == None`) the arm draws NOTHING (the sentinel
returns before any draw), so the trap only bites shimmer worlds — the
witness must place a shimmer.

**Witnesses the hunk adds (write at the merge, run at the gate):**
1. `the_seed_finish_tail_cleans_the_shimmer_pool` — a world with
   `gv.shimmer_position = Some((cx,cy))` + evil ore 22/204 inside the
   120-disc: after `step_seed_finish`, the cells read kind 1; the
   `None`-position control is byte-identical to the pre-hunk chain.
2. `the_shimmer_cleanup_continues_the_scatter_stream` — seed-pin: run
   `step_final_cleanup` (scatter draws N rolls) then the hunk; assert the
   first jitter draw equals the (N+1)-th draw of a single
   `Rng(seed ^ fnv1a64("Final Cleanup"))` replay — FAILS under the naive
   fresh-stream form, passes under (a).
3. `the_merge_sweep_visits_y_outer_x_inner` — a two-type liquid adjacency
   fixture; the merge result matches a manual y-outer/x-inner
   `liquid_check(..., true)` replay (the visit order is load-bearing:
   the side arm zeroes neighbours even when the merge gate refuses).
4. The a==b halves: the full chain with the hunk replays byte-equal.

**Composition preconditions (all already KEEP-owned, re-verified at the
seal-3 tips):** g36's `shimmer.rs::shimmer_clean_up` (bf7b6bf5, `pub`
`:965`, W:7 — drop its `#[allow(dead_code)]` when wired); g36 A1's
`gv.shimmer_position` publication (pipeline.rs:2093); g41's
`step_seed_finish` host body (`6300aab6`/`d267d43f`/`78b0cd15`, tail
`micro_surface::run` at pipeline.rs:1472 on tip `3c6da7a3f`); the g47
`LiquidSystem` + `set_gen_*` seams — now RE-FOLDED on the g46 union
`9bc722e5f` (research/96 §1 receipt: `set_gen_settle_context` :1722,
`set_gen_shimmer_position` :1737, `liquid_check` private :1347,
`gen_settle_loop` :1838; `gv.water_line`/`gv.lava_line` published on that
tip's pipeline.rs:285/:287). The `Liquid.tilesIgnoreWater`
window (`:34767`/`:34810`) is a documented no-op for the arm (g36's
gap-fill note — the Shimmerator flood carries its own
`solid_tile_ignore_water` predicate); the `Convert(j,i,0,3)` cleanse
reuses `spread::convert_cleanse` (its pre-existing genRand deviation
stands, recorded).

---

## 7. The merger bible (one paragraph) + the g36 family declaration

**The bible.** Merge dependency-first (§6.2): foundations (g39 helpers, f24
gate tables, the g47→g46-union liquid graft, the §3.2 six-point stalactite
keep-one, g42 trees) → producers (d101 anchors, g36 A1 `shimmer_position`,
g44 islands ledger, d104 ore slots) → the pass waves in the §5a 120-step
executed order, single owner each (§5 registration table is the vanilla
anchor map, §5a is what the count/positional witnesses assert) → the §6.4
compose-hunk at `seed finish`'s tail with the ONE threaded Final Cleanup
Rng → execute the §6.2 DROP list surgically, per branch, never whole-branch
reverts → at the gate run EVERY lane's written-not-run witnesses
(`cargo test --workspace -- --skip migrates_version_2_kinds_to_item_ids`,
judge by `test result:` lines), re-pin SEED42_512 + DEFAULT_WORLD ONCE on
the merged chain (no lane's isolated "pins unchanged" argument composes),
keep every a==b half green, re-pin sequentially per lane with the failing
run recorded first, and let the anchors canary veto any lossy resolution —
a missing marker means recover the arm, never delete the row.

**g36 family declaration (seal 4/4, FINAL).** The g36 lane's contribution
to the merged chain is exactly: **ONE pass — Shimmer** (A1 `702e1e75`,
review-hardened through `7f848edd`; the surviving contested-pass verdict
§2.1, DROP d102's copy) **+ ONE tail arm — `shimmer.rs::shimmer_clean_up`**
(gap-fill `bf7b6bf5`, the Final Cleanup `:22472` arm, W:7, unwired by
design, composed per §6.4) **+ the GenVars publications** it produces
(`shimmer_position` + the 200×200 `SHIMMER_SAFETY_DISTANCE` rect — the ONE
field every consumer rewires onto: d103 Wall Variety §2.5, f25/d106
ShimmerRemoveWater §3.3, the §6.4 hunk). Everything else g36 landed is
DROP-listed with a named keeper: A2 Clean Up Dirt → d102 (§2.2), A3 Dirt
Runner → d102 (§2.3), A4 Wood Walls → d102 (§2.4), A6 Wall Variety → d103
(§2.5, shimmer rewire owed), A7 Life Crystals → d103 (§2.6); A5 Settle
Liquids yielded to f25's earlier claim (`f474adc1`); the local stalactite
chain DROPs onto g41's canonical body (§3.2 point 4, the draw-schedule
exception clause stands). The lane's 11 `shimmer.rs` witnesses (A1 W:4 +
cleanup W:7) + the dropped blocks' witnesses travel per the §6.2 port-first
rules; both hash pins are unchanged on the lane (the 512 width-guard keeps
`shimmer_position` None; the unwired arm draws nothing) and re-pin at the
gate with the whole chain. **This matrix is FINAL-FINAL: no g36 work
remains unclaimed, every KEEP/DROP is decided, every cross-lane residual
has a named owner and a published contract. The gate executes; it does not
re-adjudicate.**

---
*Produced by the g36 reconcile phase. No code changed; this document is the
single authoritative keep-one decision record for the worldgen merger.*
