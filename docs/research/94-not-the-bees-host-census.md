# 94 — NOT-THE-BEES host census (W4-g41 follow-up)

Code-only census of the not-the-bees world-conversion machinery against
`decompiled/Terraria/WorldGen.cs`, taken because W4-g41's `spider_wall_remover`
(`spider_gem_moss.rs`, `:77279-77344`) landed UNWIRED — its only C# callers are
`FinishNotTheBees`' two search loops, and that host was unported (the c32
constants-off precedent). Co-lane claim check FIRST, per the brief.

## Co-lane claim check — c31 owns NOTHING here

`git ls-remote origin` + `git log origin/main..origin/task/W4-c31-hive-rewrite`:
the c31 hive-rewrite branch tip is **a725acb0 — identical to origin/main, zero
unique commits pushed**. No census, no host port, no seam contract exists on
that branch. The whole NTB machinery below is UNCLAIMED; this task takes it.
(If c31 later pushes a hive rewrite, the seam is the `step_seed_finish` slot
and the `pass_rng` names recorded here — the host dispatch order is the C#'s
own, so a hive-pass rewrite lands upstream of it without touching the host.)

## The C# machinery (read verbatim, not from docs)

| C# | Lines | Role |
| --- | --- | --- |
| `WorldGen.notTheBees` static | `:4387`, set `:11116` | the seed flag (`Main.notTheBeesWorld` alias) |
| `GenVars.notTheBeesAndForTheWorthyNoCelebration` | `:11125` | `notTheBees && getGoodWorld && !tenthAnniversary` — the spider-cave attempt-count branch (`:17367`), already read by `spider_attempts` |
| Finish dispatch | `:22423-22447` | `FinishNoTraps` → `FinishTenthAnniversaryWorld` → `FinishDrunkGen` → **`NotTheBees()` + `FinishNotTheBees()`** (`:22435-22439`) → `FinishGetGoodWorld` → `FinishRemixWorld` |
| `NotTheBees()` | `:25107-25360` | the hive-cluster world conversion: the ocean/dungeon-side honey + paint-14 arm (`:25132-25149`), the drunk/anniversary/DST centre-strip mud arm (`:25150-25181`), vines 52/382 → hive 62 (`:25187-25190`), the SolidOrSloped conversion cascade (`:25195-25267` — living wood 191/383 → 383, leaf 192/384 → 384, shimmer 224 → honey block 229, beach sand 53 → 229, grass → 60/70 by the lava line, dirt 0/mud 59 → 59, the deep 230 / shallow 225 band at `num2 = (lavaLine + maxTilesY - 180)/2`), the exposed-hive grass skirts (`:25268-25296` — 225 → 60 + 8×`Next(-2,3)`² mud pocks, 230 → 70 + the same), the wall conversion (`:25301-25338` — 63 → 64, the rest → 86 hive wall below `num2`), the liquid honey arm (`:25339-25357`) |
| `FinishNotTheBees()` | `:25388-25797` | the host: `tileSolid[379] = false` (`:25394`), the cloud-row scan (`:25396-25406`, keeps the LAST cloud row — the inner `break` only leaves the column loop), **the FTW+NTB `SpiderWallRemover` search loops** (`:25407-25435`: `maxTilesX/4` deep-band rolls `Next(UnderworldLayer+10, maxTilesY-20)` + `maxTilesX/3` mid-band rolls `Next(worldSurface, UnderworldLayer-30)`, each a ≤10000-draw wall-62 search then `SpiderWallRemover(x, y)` — called even when the search fails), the main sweep (`:25436-25630`: FTW altar 76 → 230, above-cloud honey block 229 → water, DST/noTraps spikes 48 → 232, bamboo 571 `TileFrame`, the evil-radius conversions 25/23 → corrupt jungle 661 and 203/199 → crimson jungle 662 with the bee-hive 444 kill + the 61/74 → 24/201 plant swap, vines 382/52 on hive seats → 62, `SpreadGrass(59, 70/60)` by the lava line, the beach-band wall-86 clear + honey/lava clear + 59/225/230 → ice 161/snow 147 by the midpoint, the deep ore demotion 6/7/166/167 → 0 and 40/123 → 1, the larva 231 rolls 1/600 DST-remix / 1/150 noTraps / 1/225 else), the DST spider-web arm (`:25632-25752` — the surface `Spread.Wall2(…, 62)` seeds + the rock-layer cobweb-51 placement with the `Next(10)` veto), the surface strip (`:25754-25791` — honey clear + honey drip 375 → water drip 373, jungle grass 60 → dirt 2 + the `Next(2)` vanity-tree 616/596 roll + grass-3 placement, mud 59 → dirt 0; remix: `TryGrowingTreeByType(5)`), `tileSolid[379] = true` (`:25792`), `OverGrownDungeon()` (`:25793-25796`, `!remix`) |
| `SpiderWallRemover` | `:77279-77344` | the drifting diamond wall-62 eraser — ALREADY ported (`spider_gem_moss.rs::spider_wall_remover`, W4-g41), unwired |
| `Spread.Wall2` | `:3416-3524` | the BFS wall flood: `maxWallOut2 = 5000` (`:86922`), `WallID.Sets.CannotBeReplacedByWallSpread` = {4, 40, 3, 83, 87, 244, 34} (`WallID.cs:98`), `WallSpreadStopsAtAir` = {63, 62} (`:99`) — wall 62 stops at air and spreads 8-way + the ±2 x arms; solid cells take the wall without spreading |
| `OverGrownDungeon` | `:23971-24035` | per-column `Next(50, maxTilesX/2)` (Left) / `Next(maxTilesX/2, maxTilesX-50)` (Right) + `Next(50, maxTilesY-50)` anchor, dungeon-wall gate, `Next(5, 16)` radius, the enclosed-brick test → jungle grass 60 + paint 0 + `GrowTree` / jungle plant 61, the 1/3 vine-62 cascade with the 1/4, 1/3, 1/2 tail rolls |
| `SkipOnDualDungeons` | `:25089-25105` | the dual-dungeon bounds veto — INACTIVE in the port (no `InAnyPotentialDungeonBounds` model; the `ocean.rs:620` / `mini_biomes.rs:1902` precedent) |
| `oceanDepths` | `:10963-10974` | `y <= oceanLevel && (x < 380 || x > maxTilesX - 380)` — ported (`scene.rs:971`) |
| `beachDistance` | `:86914` | 380, constant |
| `DungeonSide` | `WorldBuilding/DungeonSide.cs:13-15` | Left = −1, None = 0, Right = 1; the port's dungeon side is fixed Right (M5-BIOME, `pipeline.rs:294`) |

## Rust seam census (what exists, what this task adds)

| Need | Port state |
| --- | --- |
| Finish dispatch slot | `pipeline.rs::step_seed_finish` (`:1427`) — drunk → FTW → anniversary → micro; the NTB slot between drunk and FTW is EMPTY (this task fills it) |
| `pass_rng` convention | `Streams::pass_rng(name)` = `Rng(seed ^ fnv1a64(name))` (`pipeline.rs:139`); the Finish family uses the C# method name (`"FinishDrunkGen"` etc.) — this task adds `"NotTheBees"` + `"FinishNotTheBees"` |
| `spider_wall_remover` | `spider_gem_moss.rs:1609` — landed W4-g41, UNWIRED; this task wires it at the C# slot |
| `spread_grass` | `growth::spread_grass` (`grass.rs:113`, `:75146-75270`) — one-shot, no recursion (the documented M5-STRUCT-5 deviation); draw-free |
| `ocean_depths` | `scene::ocean_depths` (`scene.rs:971`) — verbatim |
| tree growth | `growth::{grow_tree, try_growing_tree_by_type}` (`trees.rs:286/:554`) — take `UnifiedRandom`; the port's worldgen passes are xorshift, so the host feeds them a `UnifiedRandom` seeded from its own pass stream (the `micro_surface.rs:419` `start_in_hardmode` precedent — a documented bridge, NOT the shared `genRand`) |
| `kill_tile` | `edit::kill_tile(world, x, y, no_item)` (`kill.rs:193`) |
| tile sets | `td::{SET_CLOUDS, SET_CRACKED_BRICKS, SET_BOULDERS, SET_ORE, TILE_SOLID, TILE_SOLID_TOP}`; `TileID.Sets.Conversion.Grass` = {2, 23, 199, 109, 477, 492} (`TileID.cs:86`) has no generated set — hand-transcribed like `trees.rs`' `ALLOWS_PLANTS_TO_GROW`; `WallID.Sets.Glass` = {21, 88, 89, 90, 91, 92, 241, 93, 243, 242} (`WallID.cs:91`) likewise |
| `Main.tileDungeon` / `wallDungeon` | `finish_seeds.rs`' local `TILE_DUNGEON` / `WALL_DUNGEON` consts (the six bricks / nine walls) — the host re-declares its own (module-private) |
| `Spread.Wall2` | MISSING — ported by this task (`not_the_bees.rs::spread_wall2`) |
| `OverGrownDungeon` | MISSING — ported by this task (`not_the_bees.rs::over_grown_dungeon`) |
| `PlaceTile` | the port's single-tile idiom (`spider_gem_moss.rs:54` note): the host's plant/larva/cobweb placements write kind + active + frames directly; the C# `PlaceTile` plant-arm RNG schedule (`:59507-59620`) is NOT modelled (documented deviation — the port has no `IsFitToPlaceFlowerIn`/`NextFromList` plant engine); the larva rides a verbatim `Place3x3` (`:53534-53634`, draw-free) |
| `TileFrame(571)` / `SquareTileFrame` | the post-pipeline `frame_world` seam (the M5-STRUCT-5 idiom) — no in-pass framing |
| `tileSolid[379]` toggle | the port's `TILE_SOLID` boot table already carries 379; the host's solid tests veto 379 explicitly for the pass window (the `spider_gem_moss.rs:193` precedent), restoring nothing after (the table is immutable) |

## ILied-family / micro_finish wiring census (the brief's OTHER-hosts axis)

Every `micro_finish.rs` arm is WIRED: `run()` (`:34`, dispatched from
`step_seed_finish`) covers `actually_no_traps`, `coat_everything_echo`,
`coat_everything_illuminant`, `paint_everything_gray`, `paint_everything_negative`,
`rainbow_stuff`; `no_spider_caves_i_lied` (`:442`) is dispatched from the spider
pass tail (`spider_gem_moss.rs:1309`, W4-g41). `micro_surface::run` covers the
nine surface arms. **No ILied-family host is unwired.** The still-unwired
micro-finish-adjacent pieces are the NTB host itself (this task) and the
`FinishNoTraps` (`:25986`) / `FinishRemixWorld` (`:26350`) hosts — both unported,
both out of this task's scope (no spider-wall dependency; recorded here as the
remaining Finish-family gaps).

## RNG / hash contract

- The host draws ONLY on its two fresh streams (`pass_rng("NotTheBees")`,
  `pass_rng("FinishNotTheBees")`); the tree-growth bridge seeds a per-call
  `UnifiedRandom` from the host stream (documented). A classic world reads no
  flag and never enters either function — **both world-hash pins are UNCHANGED
  by this task** (the g41 egg-sac re-pin deferral stands as-is).
- The `a == b` determinism witnesses are written NOT run (code-only).