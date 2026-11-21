# 243 — W4-f24 room-content RETIRE: the FINAL + the room-content-family declaration

> Lane `task/W4-f24-worldgen-passes-7` @ base a725acb0 · retire round 2026-11-21.
> Number **243** per the d54 research/150 registry (151–236 consumed; 237
> double-claimed e23/g48, 238 triple-claimed c8/d42/g61, 239 d42; 240/241/242
> claimed by g58's LOCAL retire commits cdfe3d108/30bb6e2b1/d0f6275a5 — a
> fresh sweep of every remote tip AND local branch finds 243–249 free, f24
> takes 243, the first free number). Parent ledger: `12k-f24-worldgen-gated.md`.
> Code-only: no cargo/tests/rustfmt/graphify; witnesses WRITTEN NOT RUN.
> C# read in ≤100-line windows; all cites `decompiled/Terraria/…` (1.4.5.8).

## 1. Item 1 — the f24b-skyblock-content scope: adjudicated NONE

The dispatch framed this round as "skyblock room-content". The census
verdict: **there is no skyblock-specific dungeon room-content in the C#** —
the room-content slice this lane ported (R1/R2) is DUNGEON/dual-dungeon
content, and skyblock prunes the whole dungeon chain:

- `Skyblock.denyAllGeneration => WorldGen.skyblockWorldGen`
  (`WorldGen.cs:3131`) — a skyblock world denies ALL generation passes.
- The Dungeon pass body is gated `if (!Skyblock.denyAllGeneration)`
  (`:14759`); the DitherSnakePass registration too (`:14753`). The port
  carries both gates (`skyblock.rs`'s denyAll table lists `"Dungeon"` at
  the `:14757` row; `pipeline.rs:1214` `pass_disabled_by_special_seeds`).
- `GenerateSkyBlockWorld` ends with `Main.dungeonX = -1; Main.dungeonY = -1`
  (`:22792-22793`) — ported verbatim (`skyblock.rs:1483-1485`). No dungeon
  anchor, no crawler, no room content on a skyblock world.
- The skyblock "Jungle" registration (`:11507-11509`) is dead code
  (`!denyAllGeneration` is always false under skyblock) — the 12k lane
  result already records it; no row needed.
- The only skyblock×dungeon intersection in the whole decompile is
  `DoPooEverywhere`'s dungeon-tile veto (`:2988`, the `skyblockWorldGen ||`
  conjunct short-circuits the dungeon test) — ported in A3
  (`seed_tails.rs`), not room content.

**GetGood / drunk-world dungeon-variant census** (every seed arm in the
dungeon family, re-read this round):

| C# arm | anchor | port status |
| --- | --- | --- |
| drunk wall re-roll `Next(3)` → 7/8/9 (wall ONLY) | `DungeonCrawler.cs:94-108` | dual producer: B5 `dungeon_genvars.rs:114-137`; **legacy path: R3b this round** (`dungeon.rs`, commit 7e02e61e) |
| drunk start row `(int)worldSurface + 70` | `WorldGen.cs:14804-14807` + dual twin `:14854-14857` | landed M5-SEED-1 (`dungeon.rs:628`) |
| GetGood spike budget ×3 | `DungeonGlobalSpikes.cs:32-35` | landed M5-DUNGEON (`place_spikes` `good_world`) |
| drunk‖noSurface spike scan-row redraw (floor) | `DungeonGlobalSpikes.cs:41-44` | **R3 this round** (commit 9ee1477b) |
| noSurface-ONLY spike scan-row redraw (ceiling) | `DungeonGlobalSpikes.cs:138-141` | **R3 this round** — the C# asymmetry kept verbatim |
| drunk‖noSurface → `createdDungeonEntranceOnSurface = true` | `DungeonCrawler.cs:411-414` | cited skip — the port's crawler has no entrance-object state (the shape-level entrance tower stands in) |
| drunk‖getGood entrance direction flip `num2 *= -1` | `LegacyDungeonEntrance.cs:67-70` | task-seed — needs the entrance object; the port's precalculated walk has no `num2` (`dungeon.rs:517-519` cited skip) |
| drunk entrance-shell rewrite (the ±0.5·strength 3-wide brick band) | `LegacyDungeonEntrance.cs:584-614` | task-seed — same entrance-object seam |
| drunk‖noSurface hall direction flip | `LegacyEntranceDungeonHall.cs:84-88` | DEAD on the port's path: the `base.calculated` arm (`:89-96`) overwrites `num3`/`vector2D2` after the flip; the port walks only the precalculated path — cited, no port |
| drunk BiomeChests: +opposite-evil style / `num = 6` | `DungeonGlobalBiomeChests.cs:39-42/:108-111` | task-seed — needs the `BiomeDungeonRoom` list (the wiring round's seam) |
| GroundFurniture / Bookshelves drunk/getGood arms | — | NONE exist (grep zero in both files) |

**Verdict: no unclaimed skyblock-specific room-content exists — nothing to
port under that label.** The two genuinely unclaimed drunk variants (the
spike scan-row pair + the legacy wall re-roll) were small and are ported
this round as R3/R3b; the entrance-object and room-list arms are task-seeded
to the wiring round (they need the `DungeonRoom`/`DungeonHall` object lists
the port's shape-level crawler does not build).

### R3 — the spike scan-row arms (commit 9ee1477b)

`place_spikes` gains `drunk`/`no_surface`/`gen_pos_y`. The C# pair is
ASYMMETRIC: the floor half redraws under `drunkWorldGen || noSurface`
(`:41`), the ceiling half under `noSurface` ONLY (`:138`) — kept verbatim,
not "fixed". Both redraws consume a SECOND `Next` (the first
`worldSurface + 25` draw is burned either way — the double-draw quirk).
`gen_pos_y` is the port's start-row proxy for
`genVars.generatingDungeonPositionY` (the C# moves it to the topmost room
at `DungeonCrawler.cs:399-400`; no room list in the port — cited skip).
The redraw's empty-range guard is draw-free (the port's `Next(lo, hi)`
convention). Flag-gated: a classic world burns the same single draw per
attempt — both classic pins unchanged. 3 witnesses written not run (the
flag-set divergence incl. the ceiling asymmetry, the mirrored-oracle
stream-position count `1001 × (2 + redraw)` per half on a wall-less
fixture, the draw-free guard).

### R3b — the legacy drunk wall re-roll (commit 7e02e61e)

`generate_dungeon` now spends one extra `Next(3)` after the colour switch
under DRUNK and overrides ONLY the wall to 7/8/9 (brick/cracked keep the
rolled colour) — `DungeonCrawler.cs:94-108`. B5 landed the twin in the dual
producer (`roll_dungeon_color`); this is the legacy path's copy on its own
HallSeed stream, drawn before the draw-free `CreatePotentialDungeonBounds`
(`:109`). The drunk setup stream shifts by exactly one `Next(3)`; classic
burns nothing extra. The drunk shell-pin witness is re-derived structurally
(the +70 start-row shift dominates the one-draw setup jitter; the exact
drunk pin rides the merge gate). 1 stream-position + a==b witness written
not run.

## 2. Item 2 — DungeonControlLine witness tangents: VERIFIED

The fix round (9f2276d5) is correct; re-verified against the C# this round:

- `SetTangents` (`DitherSnake.cs:141-152`): first line's start tangent =
  its direction; interior joints average the two directions
  (`SafeNormalize(default)` = Zero) and share the negated tangent; the
  LAST line's end tangent = `-direction` (`:151`). The port
  (`dither_snake.rs:221-235`) is 1:1, incl. the zero-length fallback.
- The witnesses (`dungeon_control_line.rs:456-457/:487-488`) set
  `start_tangent = UNIT_X`, `end_tangent = -UNIT_X` on a single
  (0,0)→(10,0) line — exactly the single-line arm (`:144` + `:151`, the
  while loop never runs). `CanPaint`/`IsInsideBorder` project correctly
  with those tangents (the mid-point/end-cap/border pins hold by
  construction).
- The pass-level end-cap override (`DitherSnakePass.cs:127-131`:
  `First().StartTangent = Last().EndTangent = UnitX * num25`, POSITIVE on
  both ends, `num25 = side == Left ? 1 : -1`) is ported at
  `dither_snake.rs:451-455` after `set_tangents()` — the witness tangents
  model the pre-override single-line state, which is what the
  `CanPaint`/`IsInsideBorder` unit tests exercise. No conflict.
- `adjust_tangents` (`dungeon_control_line.rs:417-438`) persists the
  `CacheNormals` side-effect on the FALSE arm (the C#
  `IsSelfIntersecting` caches on the shared object at `:237` either way;
  `AdjustTangentsToPreventSelfIntersection` `:249-268` returns false
  AFTER the cache) — the port writes `lines[index] = line` before the
  early return. Verified 1:1.

No code change needed; the fix-round landed state stands.

## 3. Item 3 — g43-temple compose: DISJOINT, keep both

Probe of `task/W4-g43-biome-jungle-temple` (the named tip **fc5bdd806**
"temple FINAL 3: the temple-CHARGER keep-one receipt" is the LOCAL branch
tip, one commit ahead of the pushed origin tip 9f53f18a0 — ancestor check
exit 0; the receipt itself drops the c36 `try_unlock_temple_door` pair at
the merge, g43's `unlock_temple_door` + `edit::unlock_door` the superset):

- g43's full crates diff vs base: `worldgen/temple.rs` (+815/−143),
  `tr-app/projectile.rs` + `projectile/data.rs` (the flame-trap jet 187),
  `world_sim/editing.rs` + `editing/doors.rs` (the altar summon / Temple
  Key / Solar Tablet use arms), `world_sim/wiring.rs` + `core/wiring.rs` +
  `core/edit.rs` (the unlock seam). **Zero overlap with f24's files**
  (`dungeon.rs`, `dungeon_features.rs`, `dungeon_room_content.rs`,
  `dungeon_genvars.rs`, `dither_snake.rs`, `skyblock.rs`, …).
- The two "temple" systems are DIFFERENT C# structures: the jungle temple
  is `makeTemple` (`WorldGen.cs:33435-34237`, the `LihzahrdTemple` /
  `LihzahrdTemplePart2` passes at `:15834/:17701`) — g43/d108's scope; the
  dual-dungeon "Temple" is a biome STYLE (`DungeonGenerationStyles.Temple`
  `:760-797`, brick 226 / wall 87, a `TempleStyleData`) painted by the
  dither snake into dungeon 1's quad — f24 B5's `style_temple()`
  (`dungeon_genvars.rs:87`). Keep-one verdict: **keep both, disjoint.**
- The shared constants are data, not code: wall 87 / tile 226 / the
  spike-kind swap 48→232. g43's `mayanTrap` dual arm (`:8442-8446`,
  `num17 = 48 → 232` under dualDungeons) is the TEMPLE trap pass; f24's
  87→232 swap is `DungeonGlobalSpikes` (`:52-56/:149-153`) — different
  passes, both keep. g43's temple.rs explicitly defers the dual-dungeon
  trap pass's reuse to the crawler lane ("the dual-dungeon trap pass's
  future reuse — ported verbatim" comment) — no double-port.
- d108 (temple seed branches + paint) and c33/d104/g83's temple.rs touches
  are likewise temple.rs-only — the merger composes them on that file;
  f24 has no temple.rs diff at all.
- g43's `101-doc-sweep-temple.md` collides with g80's 101 — registry-150
  assigns g43 → **227** (its own lane's rename, not this lane's job).

### 3b. The c30 dither-snake collision (found this round) — f24 WINS

`task/W4-c30-dither-snake` (8f5f10861, first-add **00:45:18**) ports the
WHOLE dither-snake lane into a single `dither_snake.rs` (1369 lines) +
the same `:14755` pipeline wire. f24 claimed the gated pair at
**00:09:38** (f353e13f, the pushed 12k census doc) and landed B6's wire at
**00:42:59** (993864e1) — both BEFORE c30's single commit. Per the
registry-150 first-pushed rule and the branch-is-the-claim rule, **f24's
B1–B6 is the keep-one survivor**; c30's module is the fold-source:

- c30's `Vec2D`/`ControlLine`/`DitherSnake`/pass bodies duplicate f24's
  `snake_math.rs` + `dungeon_control_line.rs` + `dither_snake.rs` +
  `dungeon_bounds.rs` (f24's split is the architecture-map shape).
- c30's `roll_dungeon_location` (the `GenVars.cs:452` beach-padding roll)
  is its ONE unique helper — f24 consumes the span planner's
  `gv.spans().dungeon_x` instead (the port's accepted dungeon-anchor
  model), so the helper is a cited fold-source, not a required port.
- c30's `GenVars.dither_snakes` seam duplicates f24's B6
  `gv.dual_dungeons` publication — the merger keeps f24's.
- **Merger instruction: do NOT double-register the `:14755` slot.** Both
  lanes add a "Dual Dungeons Dither Snake" step; keep f24's row + step
  body, drop c30's (its 12 module + 2 pipeline witnesses fold onto f24's
  per-unit witnesses where they pin the same C# arms).

## 4. Item 4 — the room-content-family declaration

The `DungeonGlobal*` family (`DungeonCrawler.MakeDungeon`'s feature tail,
`DungeonCrawler.cs:476-505`, 14 passes in the C# order) + the room-object
features. Status at this lane's tip:

| # | C# pass (progress slot) | lines | port | home |
| --- | --- | --- | --- | --- |
| 1 | EarlyDualDungeonFeatures (0.7) | 661 | UNPORTED — style-4/5 pit/drop-trap scatter, needs `outerProgressionBounds` + room objects | task-seed: wiring round |
| 2 | Spikes (0.75) | 246 | PORTED + WIRED (M5-DUNGEON) + R3 drunk/noSurface arms | `dungeon_features.rs::place_spikes` |
| 3 | Doors (0.8) | 366 | UNPORTED — needs `CalculatePlatformsAndDoors` lists (`:460-475`) | task-seed: wiring round |
| 4 | WallVariants (0.825) | 128 | PORTED + WIRED | `dungeon_features.rs::wall_variants`/`spread_wall_dungeon` |
| 5 | Platforms (0.85) | 219 | UNPORTED — same door/platform-list seam | task-seed: wiring round |
| 6 | BiomeChests (0.875) | 181 | UNPORTED — needs `BiomeDungeonRoom` list; the drunk arms (`:39-42/:108-111`) ride it; `GenerateDungeonBiomeChest` (`DungeonUtils.cs:281-313`) is the body | task-seed: wiring round |
| 7 | Bookshelves (0.9) | 208 | PORTED R1, UNWIRED (the dual content runs only under dualDungeons) | `dungeon_room_content.rs::bookshelves` |
| 8 | BasicChests (0.92) | 35 | UNPORTED — per-room `TryGenerateChestInRoom` (`LegacyDungeonRoom.cs:73-78` → `GenerateDungeonRegularChest` `DungeonUtils.cs:320-371`); the style-0 loot cycle is ALREADY ported (`tr-app/chest_gen.rs:145` `GetDungeonLootAndChestStyle`) | task-seed: wiring round + M5-CHEST-gen |
| 9 | Lights (0.94) | 212 | PORTED + WIRED | `dungeon_features.rs::place_lights` |
| 10 | Traps (0.95) | 156 | PORTED + WIRED (the legacy dart trap); the dual mayan traps need room objects (`dungeon.rs:748-749` cited skip; g43's `mayanTrap` port is the TEMPLE's, reusable) | `dungeon_features.rs::place_traps` |
| 11 | GroundFurniture (0.96) | 874 | SELECTION LAYER PORTED R2, UNWIRED; the placement tail (`:555-857`) needs the room/hall object lists | `dungeon_room_content.rs` |
| 12 | Paintings (0.97) | 455 | UNPORTED — the wall-variant scan + `lihzahrdPaintingsMax` budget (`:31-44`); no room objects needed for the legacy half — the smallest remaining global pass | task-seed |
| 13 | Banners (0.98) | 103 | PORTED + WIRED | `dungeon_features.rs::place_banners` |
| 14 | LateDualDungeonFeatures (0.99) | 197 | UNPORTED — the style-7/8 grass-brick conversion + style-0 door-53 rewrite + the BiomeDungeonRoom liquid cleanup + per-room late features | task-seed: wiring round |

Room-object features (the `DungeonRoom.GenerateLateDungeonFeaturesInRoom`
/ pit-trap / window families, all UNPORTED, all need the room objects):
`DungeonDropTrap` (358), `DungeonPitTrap` (407), `DungeonPillar` (226),
`DungeonTileClump` (104), `DungeonWindow`/`Basic`/`Mosaic` (19/122/495).
The R1 slice's supporting cast (the dual set clones + `IsConsidered*` +
`FirstSolid` + the room-object helpers + `PlacePot` + the shelf/platform/
wall style lookups + `CrackedSolidity`) is ported in
`dungeon_room_content.rs` — the wiring round consumes it directly.

**Family verdict:** 5 of the 14 global passes are ported and wired
(Spikes/WallVariants/Lights/Traps-legacy/Banners), plus the R1/R2 unwired
content layer (Bookshelves + the GroundFurniture selection layer); the 7
unported passes ALL block on the room/hall object lists (the wiring
round's seam) except Paintings' legacy half (a wall-variant scan — the
smallest standalone task-seed). **No unclaimed skyblock-specific
room-content exists** (§1). The f24 room-content scope is CLOSED: R1
(e0733c86) + R2 (dc9fff21) + R3 (9ee1477b) + R3b (7e02e61e); the remaining
family work is the wiring round's, seeded above.

## 5. Verification record

- C# re-reads this round (≤100-line windows): `WorldGen.cs:3131/:11505-11534/
  :14750-14879/:22526-22605/:22720-22814`, `DungeonCrawler.cs:50-114/:340-414/
  :415-514`, `DungeonGlobalSpikes.cs:25-84/:128-177`,
  `DungeonGlobalBiomeChests.cs:28-112`, `DungeonGlobalBasicChests.cs` (whole,
  35 lines), `DungeonGlobalPaintings.cs:1-45`, `DungeonGlobalEarly/Late…`
  heads, `DungeonUtils.cs:276-375`, `LegacyDungeonRoom.cs:60-154`,
  `LegacyDungeonEntrance.cs:55-94/:575-614`, `LegacyEntranceDungeonHall.cs:70-109`,
  `DitherSnake.cs:130-189`, `DitherSnakePass.cs:115-144`,
  `DungeonControlLine.cs:1-100/:216-285`.
- The R3/R3b ports re-read against those windows; the witness tangents
  re-derived from `SetTangents`' single-line arm; the g43/c30 tips probed
  from origin (fc5bdd806 is g43's LOCAL tip, 9f53f18a0 the pushed one).
- The number-243 pick: a fresh `git ls-tree docs/research/` sweep of every
  remote tip AND local branch (243–249 free everywhere; 240–242 held by
  g58's local retire commits).
- No cargo/rustc/tests/rustfmt/graphify invoked (code-only mandate). All
  witnesses WRITTEN NOT RUN; the merge gate measures the drunk/noSurface
  seed constants and re-pins the drunk shell row. Both classic hash pins
  (`SEED42_512`/`DEFAULT_WORLD`) unchanged by construction — every new
  draw is flag-gated (DRUNK / NO_SURFACE), the classic ledger is
  byte-identical; the `a == b` halves of both pin tests untouched.

## 6. Commit ledger (this retire round)

| unit | commit | status |
| --- | --- | --- |
| R3 spike scan-row arms + 3 witnesses + anchor | 9ee1477b | landed (flag-gated, both pins unchanged) |
| R3b legacy drunk wall re-roll + witness re-pin + anchor | 7e02e61e | landed (flag-gated, both pins unchanged) |
| this FINAL (243) + the family declaration + doc rows | (this commit) | landed |

Prior lane ledger: `12k-f24-worldgen-gated.md` (B1–B6 + A1–A4, 11 commits)
+ R1 e0733c86 + R2 dc9fff21 + the fix round 9f2276d5.
