# 154 — W4-c33: UnderwaterChests + PyrX consumption + the Oasis family remainder

> Wave 4 · lane `task/W4-c33-underwater-chests-oasis` @ base a725acb0 · claimed 2026-11-20
> **Number 154** per the research/150 collision registry (d54 `38bfd1d22`
> round-1 §3: `12i-c33…` → 154; g38 wins 12i). The reservation was honored
> by g50's self-rename 154→271 (`e2fd0637d`, registry §6.3). Was `12i`.
> Parent audit: `docs/research/12-worldgen.md` (the unexecuted-pass census).
> Sibling lanes probed in-flight at claim time: d101 `12b-d101-passes-1.md`
> (9/9 landed, incl. **Sand Patches**), d103 (Oasis + Waterfalls landed:
> 90b7eee6 / 561b3ab2), g37 `12h-g37-worldgen-missing-b.md` (claim only —
> its B3 row explicitly defers **Water Chests :17245** to this lane:
> "c33 claimed but pushed nothing — merger reconciles"; this push closes
> that note).

## The census (remaining work of the M5-OCEAN follow-up trio)

The M5-OCEAN ROADMAP row names three follow-ups: `Sand Patches`, the Oasis
block, `UnderwaterChests` (M5-CHEST-gen) and the `PyrX` consumption
(M5-DESERT-structures). Re-derived on a725acb0:

| follow-up | state at claim | owner |
| --- | --- | --- |
| Sand Patches (`:11689`) | LANDED on d101 (`passes1.rs::pass_sand_patches`, census 51→42) | d101 — census only here |
| Oasis pass (`:16243` + `PlaceOasis` `:10707`) + the cacti-pass oasis block (`:21326-21363`) | LANDED on d103 (90b7eee6) | d103 — census only here |
| the lily/cattail/bamboo arms of `LilypadsCattailsBambooAndSeaweed` (`:21988-22017`) | UNPORTED — d103's `ocean.rs::water_plants` doc defers them whole ("out of this task's ocean scope") | **this lane, pass 4** |
| `UnderwaterChests` (`GenPassNameID.UnderwaterChests` = **"Water Chests"**, `:17245-17355`) | UNEXECUTED — the `VANILLA_CHAIN` row exists (`pipeline.rs` "Water Chests"), no executed step cites it; the port's `"chests"` step is `Jungle Chests Placement`, not this pass | **this lane, pass 3** |
| the `PyrX/PyrY/numPyr` consumption — the `Pyramids` pass (`:15360-15460` + `Pyramid` `:27669-27967`) | EXECUTED but shape-level: `structures.rs::pass_pyramids` ignores the Dunes/Ocean-Sand-published anchors, invents 1-2 hulls of tile **480** (BloodMoonMonolith — the C# hull is **151** SandstoneBrick, wall **34**), no tunnel crawl, no chamber furniture | **this lane, pass 2** (faithful rewrite in place) |

## This lane's passes (C#-anchor order)

| # | pass | C# body | Rust seam (planned) |
| --- | --- | --- | --- |
| 2 | Pyramids (faithful) | pass `WorldGen.cs:15360-15460` (the `numPyr` loop: the >300/<width-300 gate, the dungeon-side 15 % exclusion, the sand-column descent, the nearest-slot 220 spacing, `j--`), `Pyramid` `:27669-27967` (the 151 hull, the wall-34 3×3 interior sweep + `SquareWallFrame`, the entrance shaft, the zig-zag tunnel with the chamber — the Pharaoh double-roll chest style 1, `PlaceSmallPile` 185, four banners 91 via `PlaceTile`, the `PlacePot` 28 row — and the post-chamber surface tunnel) | `structures.rs::pass_pyramids` rewritten on `GenVars.pyr_x/pyr_y/num_pyr` (the Dunes `:11564-11579` + Ocean Sand `:11580+` producers already publish them); step keeps its executed row, moves to the vanilla slot after `ocean caves`/Shimmer (`:15360` sits between `DirtWallCleanup` `:15246` and `DirtRockWallRunner` `:15461`) |
| 3 | Water Chests (UnderwaterChests) | `:17245-17355` — the `numOceanCaveTreasure` loop (the `NextFromList` {863,186,277,187,4404} main, the 2.0→50.0 +0.1 widening jitter box, the `liquid > 250` gate, `AddBuriedChest(..., 17, trySlope: true)`), then the `9 · width/4200` scatter (the `Next(10)` 863 override + the num4 1→186 / 2→4404 / 3→277 / else 187 cycle, two water hunts per iteration — `Next(1, UnderworldLayer)` then `Next(worldSurface, UnderworldLayer)` — each with the 10000-attempt cap, `trySlope` = the beach-band test) | `structures.rs::pass_underwater_chests` + the shared `add_buried_chest` placement half (the `temple.rs` private one generalised); executed step `"water chests"` (vanilla "Water Chests") at the vanilla slot after `"chests"` (`:17211`), before `"spider caves"` (`:17356`) |
| 4 | Lily/cattail/bamboo arms | the j-loop `:21988-22017` (`Next(5)` gate + `liquid > 0`, the `Next(2)` lily-vs-cattail pick, `PlaceLilyPad` `:59086-59197`, `PlaceCatTail` `:58629-58741` + `Next(14)` × `GrowCatTail` `:58974-59082`, `PlaceBamboo` `:58391-58440` + the `Next(10,20)` climb) | `ocean.rs::water_plants` gains the j-loop ahead of the seaweed walk (same step, same stream — the deferred draws join in vanilla order) |

Pass 1 is this claim + census doc.

## Discipline (shared with the d10x / g37 lanes)

- Verbatim ports (golden rule): same structure, maths, draw order; citation
  comment per block; C# read in ≤100-line windows.
- Per-pass `pass_rng("<vanilla name>")` streams (the port's per-pass re-seed
  model) — the new/changed draws shift no other step's ledger.
- **Hash re-pins DEFERRED to the merge gate** (code-only mode — no compiler
  on this branch, the S1-K-2-wave-B / d10x convention): `world_hash.rs`
  gains stale-by-design notes; the `a == b` determinism halves of both pin
  tests stay untouched.
- ONE pass per commit+push; docs + `anchors.txt` row per pass.
- Documented classic-world deviations (per the sibling lanes): the
  Skyblock/`noSurfaceNoPyramids`/remix/anniversary/dualDungeons/extraLiquid/
  surfaceIsDesert/errorWorld arms ride their seed flags where the port has
  them; the rest stay with the seed lanes (d107). The `SquareWallFrame` /
  `SquareTileFrame` visual-frame calls run through the port's framing
  convention (worldgen writes walls/tiles only — the existing
  `desert_entrances.rs` / `tile_runner.rs` precedent).

## Landing ledger

| pass | commit | status |
| --- | --- | --- |
| census + claim (this doc) | 512c4cff | claimed |
| Pyramids faithful rewrite | 524d94a7 | landed (code-only; pins deferred) |
| Water Chests (UnderwaterChests) | 321c9b66 | landed (code-only; pins deferred) |
| lily/cattail/bamboo arms | fd7fae0e | landed (code-only; pins deferred) |

## Sweep + reconcile phase (2026-11-21, on fd7fae0e)

| item | commit | outcome |
| --- | --- | --- |
| (1) Water Chests matrix conflict — arm-by-arm d103 vs c33 vs C# `:17245-17355` | (docs commit) | **ADDENDUM `120-water-chests-keep-one.md`** — confirms matrix §2.9 KEEP d103 (9 arm wins, 0 c33 wins); flags d103's extraLiquid single-draw deviation the matrix missed; re-scopes the stocking carry to c33's WHOLE `chest_gen.rs` block (post-3ecb8b08 superset); witness union + 4 new witnesses for the merge gate; matrix line 616's "unbounded loops" note corrected (d103's outer caps are present) |
| (2) pyramid loot table | 3ecb8b08 | flag11 is main-based (`:36172`) — the wall-34 chamber now stocks the Pharaoh main + Prefix(-1) + robe; flag11 forces the surface band at depth; preset mains skip the num10==0 weapon ladders; the phantom ctx.pyramid 4346/4066 gate removed (flag3 = desert/mahogany); the flag12 1/40 CreativeWings 4978 row lands; Pharaoh set + carpet + bottle table-complete in itemdata |
| (3) oasis remainder census | c7393225 + docs | no unclaimed oasis worldgen arm (d103 owns Oasis+PlaceOasis+cacti block; c33's plant deferral closed fd7fae0e); "mirage bg" = HeatDistortion (ported), oasis falls = waterfall 23 (ported); the real gap was the **UndergroundDesert 61 music arm** (constants-off since S1-L-1) — landed c7393225 |
| (4) flag5 4425/4460 verify | 3ecb8b08 | the pair is TWO INDEPENDENT 1/2 rolls (`:36515/:36845`), style-17 gated — the phantom ctx.mahogany/ctx.sky either-or rows deleted; the mid band's common rows re-derived to `:36668-36801`; all six water mains (863/186/277/187/4404/268 — trident 277 + breathing reed 186 present) stock with the pair; 5 new witnesses |

## Chest-gen FINAL phase (2026-11-21, c33d — on 5479468c)

The dispatch census: **is the oasis-chest / underwater-chest stocking
complete, and is any WorldGen chest-loot arm unclaimed?** Read at the
pushed tips — g44 `f688ab142`, f35 `5ca49d78e`, d103 `12318e505`, g37
`81fd1e0d0`, d54 registry `0426245f2`. C# re-read in ≤100-line windows.

### The chest-site census (every `AddBuriedChest` caller, `WorldGen.cs`)

| site (C# anchor) | style / main | stocking owner | status |
| --- | --- | --- | --- |
| Surface Chests scatter (`:17024/:17050/:17195`) | -1 / 0, gold-1 | d103 `buried_chest.rs` (the Surface Chests half) + c33 hook | COMPLETE |
| Jungle shrines (`:17220/:17240`) | 10 / cycle | c33 `structures.rs` site + hook (`ctx.jungle`) | COMPLETE |
| **Water Chests** (`:17267/:17328/:17346`) | 17 / 863…268 | d103 `water_chests.rs` + c33 hook (flag5) | COMPLETE (120 keep-one) |
| Pyramid chamber (`:27873`) | 1 / Pharaoh | c33 `structures.rs::pyramid` + hook (flag11) | COMPLETE |
| Living-tree room (`:29326/:29332/:29336/:29341`) | 12/14/7 / 832·4281 | base `living_trees.rs` (room carved, **chest deferred** — `:31-33` "needs TileObjectData") | DEFERRED (base, named) |
| Temple (`:34507`) | 16 / 1293 | g43 `temple.rs` + c33 hook (flag13) | COMPLETE |
| Spider-cave web (`:3737`) | 15 / 939 | base `spider_gem_moss.rs` + hook (939 override) | COMPLETE |
| **IslandHouse** (`:79371/:79375`) | 13/2 / sky cycle | g44 `islands.rs` + hook (flag12) | COMPLETE (this phase fixes the gate) |
| Dungeon regular (`DungeonUtils.cs:365`) | 2 / cycle | hook `DungeonCycle` (flag9) | COMPLETE (hook); site = M5-CHEST-gen |
| Dungeon locked biome (`DungeonGlobalBiomeChests.cs:177`) | 23-27/467-13 / 1156…4607 | hook (flag10 + the style-pottery row) | COMPLETE (hook); site = M5-CHEST-gen |
| Cave-house variants (`HouseBuilder.cs:666`) | 0 / 0 | base `mini_biomes.rs` (mushroom style-32 **not yet placed** — gold stand-in) | flag7 arm lands here; site = g42 |

**No unclaimed oasis-chest arm**: the Oasis pass (`:16243` + `PlaceOasis`
`:10707-10932`) places **no chest** — the oasis water is Desert-zone, its
"loot" is the underground-desert cave chests (flag3, COMPLETE) and the
61 music arm (c7393225). The underwater-chest stocking is COMPLETE (the
flag5 pair + all six mains, 120 §6). The only chest-loot arms NOT in the
hook are the **seed-flavour finish sweeps** — `ImproveAllChestContents`
(`:24716`, anniversary, ported `improve_all_chest_contents`), `PutMonsterVanityInChests`
(`:24782`, vampire), `DoPortalGunInChests` (`:2995`, portalGun seed),
`DoErrorWorldFinish`'s swap (`:1297`) — all named seed-lane (d107) scope,
not c33's classic-world block.

### The fixes this phase lands (the census found 4 real gate defects)

1. **flag12 is STYLE/MAIN based** (`:36176-36178`: kind 21 && (num9==13 ||
   num10 ∈ {159,65,158,2219})), NOT the wall-73 probe. The classic
   IslandHouse chest sits on **sunplate walls** (`IslandHouse :79222`
   writes disc wall 82 / sunplate 144), so the old `ctx.sky` gate missed
   EVERY faithful house chest and fired on non-sky chests near cloud
   walls. `stock_chest` now computes flag12 and threads it through
   `stock_band`; the SkyMill/pottery/Cloud/wings rows read it. The flag
   survives the remix corrupt retype (`:36179-36194`, kind→467) — the
   main disjunct carries no kind gate (the sky mains survive).
2. **The style-pottery rows sit OUTSIDE flag12** (`:37453-37456` kind-21
   styles 23-27, `:37528-37531` kind-467 style 13 — plain `num9` tests).
   The LOCKED BIOME chests (flag10, never flag12: mains 1156/1571/1569/
   1260/1572/4607) roll the 1/2 RemnantsofDevotion 5234 row on their own
   style; the old flag12 nest dropped every locked-chest draw.
3. **The flag7 mushroom arms land** (`:36151-36153` kind 21 style 32):
   the MID band's 1/2 ShroomMinecart 4450 + an INDEPENDENT 1/3
   MushroomVanity set 4779/4780/4781 (`:36653-36666`); the DEEP band's
   EITHER-OR (`:36977-36993`). Both in the `num10 == 0` branch. The old
   copy had no flag7 arm at all (the g42 mushroom-house site is the
   future producer; the arm is ready).
4. **The flag5/flag6 extras carry the C# `num10 > 0` gates** (`:36505`/
   `:36805`/`:36229`): a main-0 style-17 chest rolls NO 4425/4460 pair;
   a main-0 surface chest rolls NO flag6 living-tree extras. The flag9
   dungeon ladder keeps its family gate (the port's `DungeonCycle`
   supplies the main internally — `ev.main_item` is 0 for cycle chests).

### g44 / f35 compose (the sky family, disjoint streams)

- **Water-chest vs sky-chest DISJOINT**: the Water Chests (style 17,
  flag5) and the IslandHouse sky chests (style 13, flag12) are different
  sites, different flags, different mains — no shared arm. The sky-LAKE
  (`CloudLake :78539-78912`) places **zero chests** (verified: 0 `chest`
  hits in the body); the houses pass gates `if (!GenVars.skyLake[i])`
  (`:17868`) — a lake island gets NO house chest. g44's `9ce512b4`
  sky-lake-chest claim is the FISHING crate (3206/3985), not a placed
  chest; c33's water-chest stocking never touches it.
- **f35 26-crate-open COMPOSES, zero port work**: the sky CRATE tails
  (`fishing_crates.rs:610-622`, f35 `5ca49d78e`) share the pottery/cloud/
  wings item family with the sky CHEST by C# design (`Player.cs:9117-9157`
  vs `WorldGen.cs:37418-37527`) — two landings, NOT a dedupe target
  (g44 87 FINAL-FINAL "chest-side sibling" note). Different streams,
  different gates: the chest's Next(6) pottery is unconditional inside
  flag12; the crate's rides its own 1/2. The crate's SkyMill 2197 is
  `skyblock_low_tiles`-seam gated (`:621`), the chest's is flag12 1/3.
  c33 touches no fishing file; f35 touches no `chest_gen.rs` — clean union.

### Witnesses (WRITTEN NOT RUN — code-only lane)

3 new (`the_flag7_mushroom_rows_roll_the_two_band_shapes`,
`a_mainless_style_17_chest_rolls_no_flag5_pair`,
`the_surface_flag6_extras_need_a_preset_main`) + 5 re-derived
(`the_sky_mid_band_rolls_the_creative_wings` sunplate-82 fixture + main-0
negative; `the_sky_chests_walk_the_island_house_cycle` disc-82;
`the_21_block_rows_gate_on_the_chest_kind` flag12 style/main + wall-73
negative; `the_remaining_tail_rows_roll_at_the_vanilla_odds` style-26
locked-chest pottery; `the_dungeon_fairness_grants_once_per_world`
locked-chest negative; `the_sky_probe_reads_the_cloud_wall` disc-82 arm).
