# 12i — W4-c33: UnderwaterChests + PyrX consumption + the Oasis family remainder

> Wave 4 · lane `task/W4-c33-underwater-chests-oasis` @ base a725acb0 · claimed 2026-11-20
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
