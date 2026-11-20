# 12j — W4-g39: worldgen missing-d → the helper-wrapper lane (claim + census)

> Wave 4 · lane `task/W4-g39-worldgen-missing-d` @ base a725acb0 · claimed 2026-11-20
> Parent audit: `docs/research/12-worldgen.md` (the 52-pass unexecuted list).
> Sibling claim docs read at claim time (pushed branches only): d101
> `12b-d101-passes-1.md`, d102 `12a-d102-passes-2.md`, d103 `12-worldgen.md`
> lane-split section, d104 `12d-worldgen-passes-4.md`, d105
> `12e-worldgen-passes-5.md`, d106 `12f-worldgen-passes-6.md`, g37
> `12h-g37-worldgen-missing-b.md`. f24/f25/g36/g38 had pushed **no branches**
> at claim time (re-checked after `git fetch --prune`).

## The pass census (re-derived on this branch's base a725acb0)

The ordering rule (shared with the d10x docs): the classic (`gate == ""`)
`VANILLA_CHAIN` declarations in `pipeline.rs` that no `EXECUTED_PIPELINE`
step cites as its `vanilla` name, sorted by their `AddGenerationPass` anchor.
Scripted diff on a725acb0: **109 chain declarations, 55 distinct executed
vanilla names, 51 unexecuted classic passes** — the same 51 the d101/d102
docs derived (Granite folded into `"granite and marble"` and Generate Ice
Biome only partially covered by `"Ice"` are two of the 51).

## Coverage: every unexecuted pass is claimed by a pushed sibling

| range (C# anchor order) | claimant (pushed doc) | landed at claim time |
| --- | --- | --- |
| #1-9 Sand Patches :11689 … Granite :12839 | d101 `12b` | **9/9 landed** (4194acf8) |
| #10-18 Silt :13143 … Wood Tree Walls :15717 | d102 `12a` | Silt/Webs/Slush/Mountain Caves/Gems landed (28ec5561); Shimmer/Clean Up Dirt/Dirt Rock Wall Runner/Wood Tree Walls pending |
| Settle Liquids :16132 | d102 per d106's read (`12f`), d103 per the published split (`8a856d94`) — claimed either way, merger reconciles | pending |
| #19-27 Oasis :16243 … Cave Walls :17711 (+ Floating Island Houses :17859 per `8a856d94`) | d103 | Oasis/Waterfalls landed (561b3ab2); rest pending |
| #28-36 Floating Island Houses :17859 … Spawn Point :19460 | d104 `12d` | claim only (2d9cfcbb) |
| #37-45 Guide :19685 … Larva :20868 | d105 `12e` | claim only (5bb4116e) |
| Grass Wall :19589 (the d104/d105 orphan) | d106 `12f` adopts it | **landed** (ee0fcbef) |
| #46-51 Settle Liquids Again :21246 … Final Cleanup :22141 | d106 `12f` | claim only (89a96c19) |
| g37's quarter (remainder #10-18: Statues … Surface Ore and Stone) | g37 `12h` — overlaps d103/d104 stated ranges, its doc defers to the merger | claim only (8ef3e111) |

**No unclaimed pass remains.** The special-seed-gated registrations
(Skyblock :11511, the skyblock Jungle re-registration, Dual Dungeons Dither
Snake :14755) belong to d107 (pushed, P1-P4 landed) / c30. Per the dispatch's
exhaustion rule this lane takes the **WorldGen helper functions missing vs
C#**, ported verbatim.

## The helper census (C# `WorldGen.cs` vs `crates/tr-core/src/`)

The `PlaceTile`-wrapper family: the placement helpers the worldgen passes
call. Already ported (NOT this lane): `PlaceTile`'s torch/platform/planter
arms + `PlaceAlch` (`edit/place.rs`), `PlaceWall` (`edit/wall.rs` +
`tile_runner.rs`), `PoundTile` (`edit.rs`/`edit/hammer.rs`), the `Convert`
family (`spread.rs`), `PlaceActuator` (`edit.rs`), `PlaceDye`
(`growth/decorations.rs`), `PlacePumpkin` (`growth/pumpkins.rs`),
`PlaceSunflower`'s layout (`tileobject.rs`), `PlaceOasis`/`PlaceOasisPlant`
(`ocean.rs`, d103), `PlaceSmallPile`/`PlacePot`/`PlaceTight`/
`PlaceUncheckedStalactite` (`spider_gem_moss.rs`), `PlaceBanner`
(`dungeon_features.rs`), `RollRandomSeaShellStyle` (`ocean.rs:1240`),
`TileRunner`/`OreRunner`-shape walkers (`tile_runner.rs`), the
`SolidTile`/`SolidTile2`/`SolidTile3`/`SolidTileAllowBottomSlope`/
`SolidTileNoAttach` family (per-module copies + `edit.rs:1012`).
`ConvertGrassToDirt` does **not exist** in this decompilation (1.4.5.8) —
the grass→dirt conversion rides `Convert`/`Convert_ActuallyConvertTile`
(`WorldGen.cs:55067`/`:56312`, ported in `spread.rs`); nothing to port under
that name.

### Missing — this lane's claim (verbatim ports, shared module)

New module `crates/tr-core/src/worldgen/place_helpers.rs` — the shared
worldgen-side wrappers (the per-module narrow copies stay where they are;
the merger may dedupe onto these later). Every RNG-taking helper draws from
an explicit `&mut Rng` (the caller's pass stream = `Main.rand`/`genRand` at
the C# call site), so no stream is invented.

| # | C# helper | anchor | consumers (pending sibling passes) | commit |
| --- | --- | --- | --- | --- |
| H1 | `Place1x2` | `WorldGen.cs:46652-46679` | Herbs/Flowers (d105), temple-chair shape | landed 578b94ef |
| H2 | `Place2x1` | `:47368-47406` | Hellforge furniture (d104/g37), Guide | landed 578b94ef |
| H3 | `Place3x1` | `:52022-52065` | furniture passes | (this slice) |
| H4 | `Place2x2` (general) | `:51330-51423` | Piles/Traps (d104) — the 411-only slice exists (`mini_biomes.rs:1673`) | (this slice) |
| H5 | `Place2x2Style` | `:52198-52256` | furniture passes | (this slice) |
| H6 | `Place3x2` (general) | `:52066-52196` | the 26-altar slice exists (`evil.rs:1925`) | (this slice) |
| H7 | `Place3x3` | `:53534-53635` | Statues (g37), Hellforge | pending |
| H8 | `Place1x1` (general) | `:45188-45235` | the 141-only slice exists (`mini_biomes.rs:1653`); 324 seashell arm | (this slice) |
| H9 | `Place3x4` / `Place5x4` | `:51912-51962` / `:51963-52021` | Statues/large furniture | pending |
| H10 | `Place4x2` / `Place6x3` | `:50514-50873` / `:50465-50513` | beds/benches (Floating Island Houses d103/d104) | pending |
| H11 | `Place3x2Wall` / `Place3x3Wall` / `Place2x3Wall` / `Place4x3Wall` / `Place6x4Wall` | `:44725-44853` / `:44254-44567` / `:44568-44724` / `:44854-44945` / `:44946-45187` | paintings (Floating Island Houses, temple) | pending |
| H12 | `PlaceOnTable1x1` / `PlaceLogicTiles` | `:45340-45379` / `:45309-45339` | Guide/house furniture | pending |
| H13 | `Place1xX` / `Place2xX` | `:39177-39522` / `:39663-39935` | the temple slices exist (`temple.rs:1880`); general banners/statues | pending |
| H14 | `PlaceMan` / `PlaceWoman` | `:46302-46347` / `:46348-46393` | Guide (d105) | pending |
| H15 | `SolidTileNoPlatforms` / `SolidTileAllowPlatformTopFrame` / `SolidTileAllowTopSlope` / `SolidTileAllowLeftSlope` / `SolidTileAllowRightSlope` | `:70191-70219` / `:70220-70319` / `:70320-70339` / `:70340-70359` / `:70360-70459` | placement floors for the pending passes | pending |

Not claimed here (runtime-side or already covered): `PlaceObject`
(`:51425`, the player-facing dispatcher — `edit/place.rs` owns its arms),
`PlaceTile` proper (`:59342`, same), `KillTile` (`:63517`, `edit/kill.rs`),
`PlaceChest`/`PlaceChestDirect` (`:57813`/`:57848`, `structures.rs` +
`chest_stock.rs`), `PlaceSign`/`Place2x2Horizontal` (tombstones — no
worldgen consumer), `PlaceXmasTree`/`PlaceTorchesAroundSpawn`/`PlaceGnome`/
`PlaceCannon`/`PlaceMB`/`PlaceSuitableHerbHere` (runtime growth/event
helpers — their lanes own them).

## Discipline

- Verbatim ports (golden rule): same structure, maths, draw order; citation
  comment per block; readable Rust names; C# read in ≤100-line windows.
- Helpers are unwired until their consumer pass lands on a sibling branch —
  **both world-hash pins stay unchanged** (no executed step calls them);
  a comment-only note records that in `world_hash.rs`. The `a == b`
  determinism halves stay untouched.
- ONE slice per commit+push; docs + `anchors.txt` row per slice.
- Code-only lane: no cargo/tests/graphify on this branch; witnesses written
  NOT run, per the S1-K-2-wave-B / d10x convention.