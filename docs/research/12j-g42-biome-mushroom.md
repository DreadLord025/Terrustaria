# 12j — W4-g42: the glowing mushroom biome lane (census + claim)

> Wave 4 · lane `task/W4-g42-biome-mushroom` @ base a725acb0 · claimed 2026-11-21
> Code-only mode (no cargo/tests/graphify on this branch). C# read in ≤100-line
> windows against `decompiled/Terraria/WorldGen.cs` (direct paths only — no
> junction recursion). Parent audit: `docs/research/12-worldgen.md`.

## In-flight probe (before claiming)

All pushed worldgen siblings read at claim time:

| lane | tip | overlap with this lane |
| --- | --- | --- |
| d101 passes-1 | 4194acf8 (9/9 landed, Sand Patches..Granite) | none — `Mushroom Patches` already executed at base (M5-BIOME-2), not in the unexecuted list |
| d102 passes-2 | 28ec5561 (Silt/Webs/Slush/Mountain Caves/Gems landed) | none |
| d103 passes-3 | 561b3ab2 (Oasis/Waterfalls landed) | its `water_plants` defers the cattail arms to **c33** |
| d104/d105/d106 | claims only (d106 landed Grass Wall) | **d105 lane-5 claims the `Mushrooms` surface-decoration pass `:20599`** — NOT this lane (this lane never touches the `:20599` body or its step slot) |
| d107 special-seeds | 26672912 (ores/SurfaceCaves/dual-dungeon Ice-name) | no mushroom arm landed; the remix mushroom-layer arms stay deferred (d107's row defers remix content) |
| d108 temple-paint | 7ac633e9 | none |
| g36/g37/g38/g39 | claims/helpers (g39 H1-H15 landed) | g37's B-lane covers `:16853-18447` only; g38's C-lane ends at Dye Plants `:19981`; neither touches `:20122` or the mushroom helpers |
| c33 underwater-chests-oasis | 512c4cff (claim) | **c33 pass 4 claims `PlaceCatTail :58629-58741` + `GrowCatTail` consumers** for the `:21988` lily/cattail pass. This lane therefore does NOT port `PlaceCatTail`; the `:59652` PlaceTile-71 cattail consumer stays documented-out here (c33 owns the helper) |
| g40/g41/g43/g44 | base pins, no pushes | biome siblings — marble/spider/jungle-temple/sky-lakes are disjoint families |

No other branch touches `GlowingMushroomPatches`, `GlowingMushroomPlantsUndergroundAndJunglePlants`,
`GrowShroom`, `ShroomPatch` or `DoSurfaceIsMushrooms`. Claim is clean.

## The census (re-derived on a725acb0, C# anchors verified)

The glowing-mushroom biome generation is **five C# units**; the port's state:

| # | C# unit | anchor | port state at base | verdict |
| --- | --- | --- | --- | --- |
| 1 | `GlowingMushroomPatches` pass (cluster loop + `SpreadGrass(59,70)` sweep + tidy sweep) | `WorldGen.cs:12544-12756` | EXECUTED, `biomes.rs::pass_mushroom` on `pass_rng("GlowingMushroomPatches")` (M5-BIOME-2) | **MISSING verbatim** — four deltas below (G1-G4) |
| 2 | `ShroomPatch` (the elliptical mud dome + 2 mud runners/step) | `:79862-79982` | `biomes.rs::shroom_patch` | **MISSING verbatim** — G3: the first-ring activation guard absent (`num2--` runs at loop top, so `num2 == num4` holds on the FIRST iteration only; C# writes `type=59` unconditionally but `active(true)` only when `num2 == num4 && l > vector2D.Y` — the port activates EVERY mud write) |
| 3 | `GlowingMushroomPlantsUndergroundAndJunglePlants` mushroom arm (triple `TryGrowingTreeByType(5)` retry then `PlaceTile(71)`) | `:20122-20164` (mushroom half `:20133-20147`) | EXECUTED, `biome_detail.rs::pass_mushroom_and_jungle_plants` — **shape-level stand-in**: writes 71 directly, the triple tree retry "collapses to the plant placement" (its doc) | **MISSING verbatim** — the giant mushroom trees never grow at gen time. The jungle arms (`:20149-20160`) are already faithful — untouched here |
| 4 | `GrowTree` (tile-5 trunks; the mushroom grass 70 clearance variant) + `TryGrowingTreeByType(5)` dispatch | `:29767-29980` + `:30343-30352` | `growth/trees.rs::grow_tree` — ported, but **two missing pieces**: (a) the `type == 70` split-box clearance arm (`:29803-29806`: the 70 ground passes when the box to `j-3` AND the `j-2..j-1` strip are clear — the port only runs the full-box check, so mushroom-ground trees refuse more often than vanilla); (b) `try_growing_tree_by_type` has NO case 5 (`:30348-30350`) — returns false instead of `GrowTree` | **MISSING verbatim** (both halves) |
| 5 | `GrowShroom` (the giant glowing-mushroom tree: tile-72 trunk, 18 px grid, `Next(4,11)` height) | `:31301-31362` | **ABSENT** — no `grow_shroom` anywhere in crates/ | **MISSING whole**. Census fact: `GrowShroom` has ZERO callers in the whole decompile (grep of every `Terraria/*.cs` top level: only its own definition) — a dead-code vanilla helper in 1.4.5.8; the gen-time mushroom "trees" are tile-5 trunks on grass 70 (`GrowTree`), and tile 72 arises from the runtime framer (`TileFrame case 72 :85837-85874`, already ported in `edit/kill.rs`) and drops. Ported verbatim + wired into nothing (the faithful state), witnesses pin its draws |
| 6 | `DoSurfaceIsMushrooms` (the surfaceIsMushrooms seed conversion, normal + noSurface arms) | `:2087-2173` | EXECUTED flag-gated, `micro_surface.rs::surface_is_mushrooms` — normal arm only, and **wrong**: `Convert(k,l,3,0)` on tile 60 and wall 15 both stand in as `set_kind(59)`; the C# mushroom Convert writes **70** (jungle grass→mushroom grass) and **wall 80** | **MISSING verbatim** — G6 |
| 7 | `Convert` type-3 flavour (the mushroom conversion machine cell) | `:55336-55359` | `spread.rs` has type-0 cleanse + evil/hallow maps only; no mushroom flavour | **MISSING whole** — the `WallID.Sets.CanBeConvertedToGlowingMushroom` {64,67,15,247}→80 wall arm (`WallID.cs:88`), the torch style-22 arm (`TileID.Sets.Conversion.Torch(tile,3)` — the default-switch {0,9,16,18,19,20,21,22} set), tile 60→70, the thorn kill (`Conversion.Thorn` {32,352,69,655}, `TileID.cs:95`) |

Out of scope (owned elsewhere / documented):
- `Mushrooms` surface-decoration pass `:20599-20695` (grass frameX 144 / ice 270 speckle) — **d105 lane 5**.
- `PlaceCatTail`/`GrowCatTail` + the PlaceTile-71 consumer `:59652-59676` — **c33 pass 4** (GrowCatTail itself already landed in `growth/decorations.rs`; PlaceCatTail is c33's claim).
- The remix arms of `GlowingMushroomPatches` (`:12552-12564` dirt→mud sweep on `remixMushroomLayerLow`, `:12586` remix Y-band) — the port has no `remixMushroomLayer*` producer (Reset `:11157-11158` zeroes them; the remix seed lane d107 defers remix content). Documented-out, flag-gated skip kept.
- `errorWorld`/`dualDungeons` cluster arms (`:12588-12610`, `:12618-12627`) — `DungeonUtils.IntersectsAnyPotentialDungeonBounds` has no port model (the dual-dungeon second-dungeon bounds are unported); the errorWorld whole-world SpreadGrass sweep variant (`:12662-12677`) reads `InWorld(50)` from y=0 instead of worldSurface. Both stay flag-gated documented skips (the flags exist; the arms need the unported machinery).
- `Skyblock.denyAllGeneration`/`denySomeGeneration` outer gates — the port has no skyblock runtime gate (the standing sibling-lane deviation); a classic world always enters the body.

## The deltas this lane lands (G1-G7)

- **G1** `pass_mushroom` cluster loop verbatim (`:12570-12660`): the `num` DOUBLE loop bound (`(double)k < num` — C# runs ceil(num) clusters when num is fractional: width 6400 → 9.142… → **10** clusters, the port's `(w/700.0) as i32` truncates to **9** — a real medium-world delta; fix to the double comparison), the attempt cap `num2 > maxTilesX/2` checked AFTER the accept (the port checks `attempts <= w/2` continue — same semantics, verified), the `numMushroomBiomes < maxMushroomBiomes` accept gate (the port's `positions.len() >= 50` reject-equivalent), the underground-desert rect veto (`GenVars.UndergroundDesertLocation.Contains`, `:12630-12634` — the port HAS `gv.underground_desert` since d103/desert.rs; `pass_mushroom` never reads it: **missing**), the 500-separation via `Vector2D.Distance` (sqrt — the port's squared compare is equivalent for `<`, kept), the accept writes the position AFTER the 6 ShroomPatch calls (port pushes BEFORE — order-only, no draw difference; fixed for verbatim).
- **G2** `shroom_site_rejected` (`:12611-12635`): add the underground-desert rect veto (needs the rect passed in — signature change through `step_mushroom`).
- **G3** `shroom_patch` (`:79862-79982`): the last-ring activation guard (`num2 == num4 && l > py` activates; otherwise only `type = 59` writes on possibly-inactive cells) + the `Next(-200,-100)` vy draw (port has `range(-200,-99)` — C# `Next(-200, -100)` is exclusive-hi, `-99` is WRONG, one-off), + the mud-runner re-roll condition verbatim (`while (!active && type != 59)` — the port's `hit = active || kind==59` break is the same predicate, kept).
- **G4** the tidy sweep (`:12694-12754`): already faithful (`shroom_tidy`) — verify-only, no change expected. The `SpreadGrass(59,70)` sweep band (`worldSurface..maxTilesY`, `InWorld(50)`) already faithful.
- **G5** the plants-pass mushroom arm (`:20133-20147`) verbatim: `j >= (int)worldSurface && type==70 && !above.active` → `TryGrowingTreeByType(5, i, j)` THREE times, each gated on the above-cell still free, then `PlaceTile(i, j-1, 71, mute: true)`. The PlaceTile-71 body (`:59652-59676`) needs the below-gate (active, no slope/half-brick, type 70) and the plain arm `active/type/frameX = Next(5)*18` — the cattail branch (`j > worldSurface` → PlaceCatTail) is c33's helper, documented-out with the plain-arm fallback (the port's current direct-71 write becomes the gated PlaceTile port). RNG: `grow_tree` draws on the pass stream; the port's `Rng` is xorshift while `growth::grow_tree` takes `UnifiedRandom` — the seam uses the `chest_stock::WorldGenRand` precedent: a new `grow_tree_worldgen` wrapper in `growth/trees.rs` generic over the draw trait, OR the worldgen-local tree writer. **Decision**: extend `growth/trees.rs::try_growing_tree_by_type` with case 5 → `grow_tree`, and add a worldgen-side adapter that runs the same `GrowTree` logic on the port's `Rng` via the `WorldGenRand` trait (the trait already abstracts exactly `next(max)`/`range(lo,hi)`; `grow_tree` needs only those two draws — refactor `grow_tree` to be generic over `WorldGenRand`, keeping the `UnifiedRandom` callers byte-identical).
- **G6** `DoSurfaceIsMushrooms` normal arm (`:2118-2172`): the band draw is in the **inner `for` CONDITION** — `for (l = 0; l < worldSurface + Next(3); l++)` re-evaluates every iteration, so C# draws `Next(3)` **once per row visited plus one failing draw per column** (~worldSurface+1 draws per column), NOT one per column and NOT the port's one global draw. The port's single pre-loop band is a real stream + coverage delta on the seed world; the verbatim port moves the draw into the loop condition. The Convert(3) calls on tile 60 AND wall 15 (both → the type-3 machine, G7), the direct writes 0→59 / 2→60 / walls {63,65,66,68,64,205,206,207}→80 (the port's wall set matches; the tile writes match; only the two Convert arms are wrong). The C# tile reads are UNGATED by active (`Main.tile[k,l].type == 60` reads the stored type even on inactive cells) — the port matches (`kind_at`). The noSurface arm (`:2094-2116`) stays documented-out (no noSurface world state — the standing M5-SEED-2 deviation) but the DOUBLE-call dispatch (`:607-613`: `if !noSurface { Do() } Do()` — the second call always runs, and `MudCavesToJungleGrass :12474-12477` runs a THIRD conditional call mid-pipeline) is recorded; the port's `micro_surface::run` calls it once — the mid-pipeline call site is a documented follow-up (it needs the `mud caves to grass` step seam, and its draws ride the same pass stream).
- **G7** the `Convert` type-3 cell (`:55336-55359`) into `spread.rs` as `convert_cell_mushroom` (the `convert_cleanse_cell` sibling): walls {64,67,15,247}→80 (`Convert_ActuallyConvertWall` non-zero+different gate), torch style→22 (the default-switch set), 60→70 (`Convert_ActuallyConvertTile` active+different + tree-above kill), thorn {32,352,69,655} kill. The `InWorld(1)` + `!active && wall<=0` early-out verbatim. No RNG (the type-3 arm draws nothing — the wall-80 depth rolls live in the type-0 cleanse arm only).

## RNG world-pass slots (the stream contract)

- `pass_mushroom` keeps `pass_rng("GlowingMushroomPatches")` — the G1-G3 fixes change the DRAW SEQUENCE on that stream only (the vy one-off changes values not counts; the desert veto changes accept/reject cadence). No other step's ledger shifts (per-pass re-seed model).
- The plants pass keeps `pass_rng("Glowing Mushrooms and Jungle Plants")` — G5 ADDS draws (the triple GrowTree attempts + the PlaceTile-71 Next(5)) on that stream; the jungle arms after each mushroom anchor shift along the stream (the accepted world change, documented).
- `DoSurfaceIsMushrooms` keeps `pass_rng("DoSurfaceIsMushrooms")` — G6 changes the draw count (per-column Next(3) instead of one global) on the seed world only; classic worlds draw nothing (flag-gated).
- `GrowShroom` (G5b) is caller-less — its witness seeds are local `UnifiedRandom`/`Rng` fixtures, no world stream.

## Hash discipline

Code-only lane: `world_hash.rs` gains stale-by-design notes for BOTH pins
(the mushroom-arm draws + the ShroomPatch activation guard move every world
with a mushroom patch by design); the `a == b` determinism halves of both pin
tests stay untouched. Re-pin values DEFERRED to the merge gate (the d10x /
S1-K-2-wave-B convention). Affected-seed witnesses (surfaceIsMushrooms a==b)
written not run.

## Landing ledger (one phase per commit+push)

| phase | scope | commit | status |
| --- | --- | --- | --- |
| 1 | census + claim (this doc) | 65009957 | landed |
| 2 | G7 Convert type-3 cell + G6 DoSurfaceIsMushrooms verbatim (spread.rs + micro_surface.rs) | e2707f2d | landed |
| 3 | G1-G4 pass_mushroom/ShroomPatch verbatim + desert veto + tidy re-mud (biomes.rs + pipeline.rs) | 67040975 | landed |
| 4 | G5 GrowTree verbatim rewrite + case-5 dispatch + WorldGenRand generic + GrowShroom + EmptyTileCheck (growth/trees.rs) | b44db327 | landed |
| 5 | G5 plants-pass mushroom arm verbatim + PlaceTile-71 (biome_detail.rs) + witnesses + hash notes + docs/anchors | (this commit) | landed |

All phases code-only (no cargo/tests/graphify invoked). 18 witnesses WRITTEN
NOT RUN across spread.rs (2), micro_surface.rs (3 incl. the seed-world a==b),
biomes.rs (4), growth/trees.rs (5), biome_detail.rs (4 incl. the re-derived
DW-5.2 ledger). Both world-hash pins carry stale-by-design notes; the `a == b`
determinism halves are untouched; re-pin values deferred to the merge gate.

Extra deltas found during the port (beyond the census): the ShroomPatch `py`
advance was missing entirely (G3), the tidy vertical re-mud read the wrong
column/row (G4), the cluster bound truncation differs at width 6400 (G1),
the cap-gate accept retried instead of exiting (G1), and `GrowTree`'s port
was a batch-write refactor with reversed trunk draw order + missing branch
else-arms + missing top `Next(13)` gate (G5 — full verbatim rewrite).
