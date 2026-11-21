# 12k — W4-f24: worldgen gated passes (the unclaimed remainder)

> Wave 4 · lane `task/W4-f24-worldgen-passes-7` @ base a725acb0 · claimed 2026-11-21
> Parent audit: `docs/research/12-worldgen.md` (the unexecuted-pass census).
> Code-only mode (no cargo/tests/graphify on this branch); C# read in ≤100-line
> windows; ONE pass per commit+push; hash re-pins deferred to the merge gate
> (the S1-K-2-wave-B / d10x convention), `a == b` determinism halves untouched.

## In-flight probe (all pushed branches + census docs read at claim time)

`git fetch origin --prune` + every pushed `docs/research/12*` doc and the
pushed `pipeline.rs`/`crates/` diffs of all thirteen worldgen lanes:

- **d101** `12b-d101-passes-1.md` @ 4194acf8 — **9/9 landed**: Sand Patches,
  Mount Caves, Dirt Wall Backgrounds, Clay, Small Holes, Dirt Layer Caves,
  Rock Layer Caves, Generate Ice Biome, Granite (own row). Census 51→42.
- **d102** `12a-d102-passes-2.md` @ 28ec5561 — **5 landed**: Silt c746a7da,
  Webs 96e4b92f, Slush d4fd0af0, Mountain Caves 665041f8, Gems 28ec5561;
  Shimmer, Clean Up Dirt, Dirt Rock Wall Runner, Wood Tree Walls pending
  (g36's a-slice reassigns them per g37's `12h` split).
- **d103** lane-split section in `12-worldgen.md` @ 561b3ab2 — **2 landed**:
  Oasis 90b7eee6, Waterfalls 561b3ab2; Wall Variety…Cave Walls + Floating
  Island Houses claimed, pending.
- **d104** `12d-worldgen-passes-4.md` @ 2d9cfcbb — claim only (#28-36,
  Floating Island Houses…Spawn Point).
- **d105** `12e-worldgen-passes-5.md` @ 5bb4116e — claim only (#37-45,
  Guide…Larva).
- **d106** `12f-worldgen-passes-6.md` @ ee0fcbef — **1 landed**: Grass Wall
  (R1, the orphan adoption); Settle Liquids Again, Tile Cleanup, Stalac,
  Remove Broken Traps, Final Cleanup pending.
- **d107** @ 26672912 — special-seed branches (drunk ore re-rolls, remix
  bands, SurfaceCaves gates, dualDungeons pruning name fix). Its pushed
  ANALYSIS **explicitly defers**: "the skyblock generator, … the
  dualDungeons world content itself (the second dungeon)". No classic or
  gated pass row executed.
- **d108** @ 7ac633e9 — temple seed branches + paint; no census rows.
- **f25** `12g-f25-settle-liquids.md` @ fd098528 — adopted the **Settle
  Liquids** orphan (`:16132`), pending.
- **g36** `12g-g36-worldgen-missing-a.md` @ 0b01c170 — a-slice minus d102's
  landed Mountain Caves/Gems: Shimmer, Clean Up Dirt, Dirt Rock Wall Runner,
  Wood Tree Walls, Settle Liquids, Wall Variety, Life Crystals (7), pending.
- **g37** `12h-g37-worldgen-missing-b.md` @ 30f605c7 — b-slice: Statues,
  Surface Chests, Water Chests, Cave Walls, Floating Island Houses, Quick
  Cleanup, Hellforge, Spreading Grass, Surface Ore and Stone (9), pending;
  fixes the deterministic quartered split a=#1-9, b=#10-18, c=#19-27,
  d=#28-37 of the 37-pass remainder.
- **g38** `12i-g38-worldgen-missing-c.md` @ d32b13cc — c-slice minus d106's
  landed Grass Wall: Place Fallen Log, Traps, Piles, Spawn Point, Guide,
  Sunflowers, Herbs, Dye Plants (8), pending.
- **g39** `12j-g39-worldgen-helpers.md` @ b24483c2 — the d-slice is fully
  claimed by d105/d106/g38, so this lane took the **PlaceTile-wrapper helper
  family** (H1-H12 landed, H13-H15 pending). Its "Not claimed here" list
  explicitly leaves `PlaceTorchesAroundSpawn` and the runtime/event helpers
  to their owning lanes.
- **c32** @ d27fb00a — the dungeon biome style **tables** (Snow/Desert/
  Hallow/Crimson + Crystal, unwired). Its ANALYSIS explicitly defers "the
  dual-dungeon list arms (`DungeonCrawler.cs:111-128`…)" and the other style
  statics (Cavern/Corruption/Jungle/Temple).
- **c33** (Water Chests/Oasis, referenced by d104/g37) — no pushed branch.

## Census re-derived on this branch's base (a725acb0)

Scripted diff of `pipeline.rs` (sha256
`5f430aa7a119dc58929280f138cc6094ac32f8a0010a51d06599430e651a0335`):
**109 `VANILLA_CHAIN` decls (106 classic `gate == ""` + 3 gated), 66 executed
steps citing 55 distinct vanilla names → 51 classic unexecuted + 2 gated
unexecuted** (the skyblock "Jungle" `:11509` is a duplicate registration of
the already-executed "Jungle" step, not new work).

**Union of every pushed lane's OWN ledger covers all 51 classic names**
(d101 #1-9 ∪ d102 #10-18 ∪ d103 #19-27 ∪ d104 #28-36 ∪ d105 #37-45 ∪
d106 {Grass Wall, #46-51}, with the g36/g37/g38 mop-up reassigning the
pending overlaps and f25 adopting the Settle Liquids orphan). **No classic
pass is unclaimed.** A scripted probe of every pushed branch's `crates/`
diff for `GenerateSkyBlockWorld|SkyblockIsland|DitherSnake|DungeonControlLine|
SetupDungeonGenVarVariables` returns **zero hits** — neither gated pass is
ported anywhere.

## The gap this lane takes: the two GATED passes + their exclusive dependencies

The only unexecuted, unclaimed registrations are the special-seed-gated
pair. d107 landed the *pruning* (`pass_disabled_by_special_seeds`) and the
seed branches inside already-ported passes, but explicitly deferred the two
gated pass **bodies**. This lane ports them verbatim, plus the helper
functions they alone need that no pushed ledger claims.

### Lane B — Dual Dungeons Dither Snake (`WorldGen.cs:14755`, gate `SecretSeed.dualDungeons`)

| # | unit | C# anchor | Rust home (planned) |
| --- | --- | --- | --- |
| B1 | `Vector2D` math + `Utils.Remap`/`Lerp` (f64) + `DungeonBounds` | ReLogic `Vector2D.cs`; `Utils.cs`; `DungeonBounds.cs:8-337` | new `worldgen/snake_math.rs` + `worldgen/dungeon_bounds.rs` |
| B2 | `DungeonControlLine` | `DungeonControlLine.cs:9-285` | new `worldgen/dungeon_control_line.rs` |
| B3 | `DitherSnake` | `DitherSnake.cs:10-199` | new `worldgen/dither_snake.rs` |
| B4 | `DitherSnakePass` (Calculate/Generate/Apply + bayer) | `DitherSnakePass.cs:13-145` | `worldgen/dither_snake.rs` |
| B5 | producer chain: `SetupDungeonGenVarVariables` dual arms + `CreatePotentialDungeonBounds` + the Cavern/Corruption/Jungle/Temple style statics c32 deferred | `DungeonCrawler.cs:54-213`, `DungeonUtils.cs:2067-2087`, `DungeonGenerationStyles.cs:189-351/:434-473/:594-797` | `worldgen/dungeon_genvars.rs` (+ style rows) |
| B6 | pipeline wire: the gated step at the `:14755` slot + `GenVars.dungeonGenVars[2]` + `CurrentDungeon` + the Dunes dual setup draws (`:11520-11525`) + `NormalizedDistanceSafeFromDither` (`:11465`) | `WorldGen.cs:11465/:11520-11525/:14753-14756` | `pipeline.rs` |

### Lane A — Skyblock (`WorldGen.cs:11511`, gate `WorldGen.skyblockWorldGen`)

| # | unit | C# anchor | Rust home (planned) |
| --- | --- | --- | --- |
| A1 | `Skyblock` statics + gate properties + the `denyAllGeneration` pruning census | `WorldGen.cs:3063-3281` | new `worldgen/skyblock.rs` + `pipeline.rs` gate table |
| A2 | `SkyblockIsland` | `WorldGen.cs:22818-23556` | `worldgen/skyblock.rs` |
| A3 | shared tail helpers (unported, unclaimed): `DoPooEverywhere`, `DoErrorWorldShuffleBlocks` (+`DoErrorWorldGetRandomBlock`/`ErrorWorldSwapTiles`/`errorWorldAdjustment`), `DoAdditionalChangesAroundSpawnIfNeeded` (+`PlaceTorchesAroundSpawn`/`RemoveSpecificTilesUnderSpawn`) | `WorldGen.cs:2966-2993/:1084-1295/:329-336/:24936-25087` | `worldgen/skyblock.rs` / `worldgen/seed_tails.rs` |
| A4 | `GenerateSkyBlockWorld` + pipeline wire (the gated step + the skyblock Jungle registration gate + the denyAll pruning) | `WorldGen.cs:22526-22816/:11505-11516` | `worldgen/skyblock.rs` + `pipeline.rs` |

Neighbours NOT this lane: every classic pass (claimed above); the
`PlaceTile`-wrapper family (g39); the dungeon style tables Snow/Desert/
Hallow/Crimson/Crystal (c32); `GrowLivingTree` (already ported,
`living_trees.rs:327`); `DoErrorWorldFinish`/`DoPortalGunInChests` (chest/
runtime-side, not the skyblock generator's tail).

## Discipline

- Verbatim ports (golden rule): same structure, maths, draw order; citation
  comment per block; readable Rust names; C# read in ≤100-line windows.
- Every RNG-taking unit draws from an explicit `&mut Rng` (the caller's pass
  stream = `Main.rand`/`genRand` at the C# call site); each gated step uses
  its own `pass_rng("<vanilla name>")` so the insertion shifts no classic
  step's ledger.
- **Both classic hash pins STAY** (`SEED42_512`/`DEFAULT_WORLD`): every new
  draw is gated behind `SeedFlags::SKYBLOCK` / `MicroSeedFlags::DUAL_DUNGEONS`
  — a classic world runs the byte-identical ledger. The affected seeds'
  hashes move by design and ride `a == b` witnesses (written NOT run); the
  `a == b` determinism halves of both pin tests stay untouched.
- ONE pass per commit+push; docs + `anchors.txt` row per pass.

## Landing ledger

| unit | commit | status |
| --- | --- | --- |
| census + claim (this doc) | f353e13f | claimed |
| B1 snake_math + dungeon_bounds | 341b6643 | landed (code-only; unwired, both pins unchanged) |
| B2 DungeonControlLine | 3abf580d | landed (code-only; unwired, both pins unchanged) |
| B3 DitherSnake | 37c0cdc8 | landed (code-only; unwired, both pins unchanged) |
| B4 DitherSnakePass | fd816665 | landed (code-only; unwired, both pins unchanged) |
| B5 producer chain + style statics | 126e9ab6 | landed (code-only; unwired, both pins unchanged) |
| B6 pipeline wire (dither snake) | 993864e1 | landed (code-only; gated, both classic pins unchanged) |
| A1 Skyblock statics + gate census | 372a4ec7 | landed (code-only; gated, both classic pins unchanged) |
| A2 SkyblockIsland | a9eb3fc3 | landed (code-only; unwired, both classic pins unchanged) |
| A3 shared tail helpers | b37de294 | landed (code-only; unwired, both classic pins unchanged) |
| A4 GenerateSkyBlockWorld + wire | 9cbbf9de | landed (code-only; gated, both classic pins unchanged) — LANE A COMPLETE |

## Lane result

Both gated passes executed: `EXECUTED_PIPELINE` grows 66 → 68 steps
(`dither snake` at the `:14755` slot, `skyblock` at the `:11511` slot). The
unexecuted census closes from 53 (51 classic + 2 gated) to 51 — every one of
the 51 classic names is claimed by a pushed sibling ledger (d101-d106 +
g36-g39 + f25), and the two gated names are now executed here. The skyblock
"Jungle" registration (`:11509`) is dead code (`denyAllGeneration` is always
true under skyblock, so the `!denyAllGeneration` guard never adds it) — no
row needed.

Both classic hash pins (`SEED42_512`/`DEFAULT_WORLD`) are UNCHANGED: every
new draw is gated behind `SeedFlags::SKYBLOCK` /
`MicroSeedFlags::DUAL_DUNGEONS`, and the A1 denyAll/denySome pruning + the
B6/A4 step bodies are no-ops on a classic world. The affected seeds' hashes
move by design and ride the `a == b` witnesses
(`w4_f24_the_dither_snake_is_gated_and_deterministic`,
`w4_f24_the_skyblock_world_is_gated_and_deterministic`, plus the per-unit
witnesses) — all written NOT run (code-only lane); the merge gate measures
the per-seed constants. The `a == b` determinism halves of both pin tests
are untouched.

New modules: `snake_math.rs`, `dungeon_bounds.rs`,
`dungeon_control_line.rs`, `dither_snake.rs`, `dungeon_genvars.rs`,
`skyblock.rs`, `seed_tails.rs` (7 files, ~4300 lines incl. witnesses).
`anchors.txt` gains 11 rows (one per unit). The merger may dedupe B5's
paint-field style rows onto c32's `dungeon_styles.rs` tables and A3's tail
helpers onto the sibling passes' call sites (Beaches `:14894`, Life Crystals
`:16761`).

## Retire round (2026-11-21) — the room-content FINAL

The lane's room-content scope is CLOSED: R1 (e0733c86) + R2 (dc9fff21) +
R3 (9ee1477b, the drunk/noSurface spike scan-row arms) + R3b (7e02e61e,
the legacy drunk wall re-roll). The FINAL + the room-content-family
declaration live in **`243-f24-room-content-final.md`** (number 243 per
registry-150: 237–239 multi-claimed, 240–242 g58-local, 243 the first free
at every tip). Key verdicts: the skyblock room-content scope is NONE
(denyAllGeneration prunes the whole dungeon chain); the SetTangents
witness tangents verified 1:1; g43's temple work DISJOINT (keep both);
**c30's dither-snake lane is a full duplicate — f24 B1–B6 is the keep-one
survivor (claim 00:09:38 vs c30 00:45:18), the merger must NOT
double-register the `:14755` slot**. Both classic pins unchanged
throughout; all witnesses written NOT run.
