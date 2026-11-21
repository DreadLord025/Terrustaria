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
| 5 | G5 plants-pass mushroom arm verbatim + PlaceTile-71 (biome_detail.rs) + witnesses + hash notes + docs/anchors | d4836bbd | landed |
| 6 | TAIL census: the other tree types riding the generic rewrite (gem 583-589 / palm 323 / vanity / ash) + the mushroom runtime growth + the glowing-mushroom drops | d13d05a1 / 73a4e267 / a964730f | landed |

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

## The TAIL census (phase 6, 2026-11-21) — the other riders + runtime + drops

The G5 `GrowTree` rewrite is generic; the tail censuses every OTHER tree type
riding `TryGrowingTreeByType` (`WorldGen.cs:30343-30386`) and the mushroom
runtime/drop consumers against the C#, reading the sibling claims (i30 axe
arm landed on main `0c888e89`; c29 gem-moss co-lane = `spider_gem_moss.rs`
`count_tiles` shared, no Convert(3) overlap; g14/d54 dust own the dust half).

### The dispatch table (`:30348-30384`) — verdict: COMPLETE at G5

Every C# case has a port arm in `try_growing_tree_by_type`: 5→`grow_tree`,
323→`grow_palm_tree`, 587/588/583/584/589/586/585→the gem profiles,
596/616→vanity, 634→ash. No missing case. The deltas are in the BODIES:

| # | unit | C# anchor | port state at d4836bbd | verdict |
| --- | --- | --- | --- | --- |
| T1 | `GrowTreeWithSettings` wall test | `:30404` + `GemTreeWallTest :30269-30302` + profiles `:3935` | ran `DefaultTreeWallTest` for EVERY profile (the gem profiles' `WallTest = GemTreeWallTest` was dropped) | **MISSING** — gem saplings against the stone/dirt wall family {2,54-59,61,185,196-199,208-215} refused. Added `WallTestKind` + the gem set. |
| T2 | `GrowTreeWithSettings` side-ground flags | `:30688-30694` `nactive()` | port read `!t.active()` — INVERTED polarity (the caps landed on the wrong side of the `Next(3)` rolls) | **MISSING** — fixed to `active() && !inactive()`. |
| T3 | `GrowTreeWithSettings` ground gate | `:30400` `nactive()` | port checked `active()` only, missing `!inactive()` | **MISSING** — added. |
| T4 | `GrowTreeWithSettings` top-cap draw order | `:30803` `Next(13)` THEN `:30805` `Next(3)` | port drew `Next(3)` before `Next(13)` | **MISSING** — one draw of the wrong modulus early; reordered. |
| T5 | `GrowTreeWithSettings` errorWorld height arm | `:30420-30423` (`errorWorldAdjustment(2.0)`, `Next(3,15)`/`Next(3,50)` by surface) | parked ("the getGoodWorld callers park") | **MISSING** — ported (the `grow_tree` errorWorld precedent, adjustment 2.0). |
| T6 | `GrowTreeWithSettings` frame-important boot bit | `grow_tree :30248` precedent | absent (the runtime kill cascade could not recognise a freshly-grown gem/vanity/ash tile) | **MISSING** — added. |
| T7 | `profile_for_tree` (`TryGetFromTreeId`) | `:3890-3928` | mapped ALL gems 583-589 to `GEM_TREE_RUBY` | **WRONG (benign for the ground test, fixed for verbatim)** — per-kind profiles. |
| T8 | `AttemptToGrowTreeFromSapling` style-10 return | `:72833-72835` `return flag` | port returned flat `true` after the ash grow | **MISSING** — returns the grow result. |
| T9 | `AttemptToGrowTreeFromSapling` default-arm underground gate | `:72846-72852` (`underground && !bees && !remix`) | port refused ALL underground default grows (missing the bees/remix exemption) | **MISSING** — added. |
| T10 | `AttemptToGrowTreeFromSapling` gem style ≥7 | `:72891` `num2 = 587` default | port's catch-all sent style ≥7 to amber 589 | **MISSING** — keeps the 587 default. |
| T11 | `GrowPalmTree` lean gate | `:27185` `if (num5 != num4)` | port omitted the gate — a palm at its target lean kept drawing Next(13)/Next(9)/Next(5) every middle row | **MISSING** — added (RNG-stream delta on palm growth). |

### The mushroom runtime growth — verdict: the Convert(3) cell is gen-only

`Convert(i, j, 3, …)` has exactly THREE call sites in the whole decompile
(`:2101`, `:2125`, `:2129` — all inside `DoSurfaceIsMushrooms`); there is NO
runtime mushroom-spread Convert(3) (the runtime spread is `SpreadGrass(59,
70)` `:74737`, already faithful in `growth/grass.rs`). The MushroomSpray
solution (ammo 113) routes through the aiStyle-31 Clentaminator arm, which
the port documents as type-0-only (`spread.rs:391-395`, S1-D-4) — the other
spray kinds have no `proj_data` rows. **No missing Convert(3) runtime arm.**

**GROWTH FINAL correction (phase 8)**: the literal-`3` grep missed the
VARIABLE-typed fourth site — `Projectile.cs:26834`
`WorldGen.Convert(point2.X, point2.Y, num245, size)` where the aiStyle-31
ladder (`:26791-26825`) maps the DarkBlueSolution projectile 148 →
`num245 = 3`. So Convert(3) has FOUR call sites: three gen-only
(`DoSurfaceIsMushrooms`) + one runtime (the Clentaminator spray). The
verdict stands for the UpdateWorld side (no mushroom-spread Convert arm
there — the runtime grass-70 spread is the `SpreadGrass(59, 70)` mud scan,
covered by T-3's overworld case-70 + D1's underground soil scan + D4's
tail); the spray side is the projectile lane's documented park
(`projectile_ai_misc.rs::ai_spray` ports the ladder, runs only the type-0
cleanse cell — the 146-149/1015-1017 solutions are unrowed;
`spread::convert_cell_mushroom` (G7) stands ready for that wiring).

The OVERWORLD grass-growth case arms (`UpdateWorld_GrassGrowth`'s
`switch (num10)` `:74510-74868`, incl. the mushroom case-70 arm `:74704-74750`:
the `Next(10)` plant-71 roll + `CopyPaintAndCoating`, the `Next(300)`
`TryGrowingTreeByType(5)` roll, the 59→70 spread scan) are a SEPARATE branch
from the underground slice the port implements (`update_world_grass_growth`
runs the `:74275-74413` underground body only). The overworld case-70 arm
needs `PlaceCatTail` (c33's helper, `:74500`) for its liquid-above branch and
`PlayerLOS`/`TreeGrowFXCheck` (unported) — **documented follow-up** (the
overworld grass-growth dispatcher tail, shared with the jungle case-60 arm
`:74752` and the hallow/ash arms; not mushroom-exclusive). The underground
mushroom arm (`:74497-74506`: the `Next(250)` tree roll + the cattail
liquid branch) shares the same dispatcher gap.

### The glowing-mushroom drops (KillTile arms) — verdict: THREE missing

The dust half is g14/d54's (the `KillTile_MakeTileDust` ladder, landed); the
`KillTile` body (`:63517-63921`) carries ZERO gore calls (the pot shards
`:56878-56953` are the documented deferred gore, `edit/kill.rs:2363`). The
gore/drop-side census of the mushroom arms:

| # | unit | C# anchor | port state at d4836bbd | verdict |
| --- | --- | --- | --- | --- |
| D1 | `KillTile_GetItemDrops` case 71 (the glowing-mushroom PLANT) | `:65642-65652` (`case 71: case 72:`) | `tree_drop_record` records ONLY 72; `by_tile(71)` = None (item 183 places tile 190, the potted mushroom) → a broken 71 plant drops NOTHING | **MISSING** — record 71 too (the `Mushroom` event: Next(40)→194 seeds, else Next(2)→183 mushroom). |
| D2 | `KillTile_GetItemDrops` case 70 (mushroom GRASS) + 60/661/662 | `:64702-64704` (70→176 mud), `:64936-64941` (59/60/661/662→176) | `by_tile(70)` = 194 (the seeds item places 70) — WRONG, the grass drops the MUD block 176; 60/661/662 likewise leak their seed items | **MISSING** — `tile_drop` 70/60/661/662 → 176. |
| D3 | `KillTile_GetItemDrops` case 227 (dye plants — the teal mushroom grows in mushroom biomes) | `:65048-65055` (`1107 + frameX/34`, styles 8-11 → `3385 + num - 8`) | `by_tile(227)` = 1107 FLAT — every style drops the teal mushroom | **MISSING** — the frameX/34 style ladder. |

The mushroom-TREE (72) drops + support check + canopy re-roll landed with
i30 (`check_mushroom_tree`, `TreeDropEvent::Mushroom`, `mushroom_frame_rolls`);
the trunk `TreeType::Mushroom` arm (`KillTile_GetTreeDrops :66241-66250`,
Next(2)→183) is faithful (`trunk_tree_drops`). The gem-tree drops
(`SetGemTreeDrops :66094-66108`) are faithful (`gem_tree_drops`, i30).

### The mushroom runtime growth — the overworld case-70 arm (landed)

`UpdateWorld_GrassGrowth` has TWO branches: the underground slice
(`:74275-74508`, returns at `:74508`) and the overworld `switch (num10)`
(`:74510-74868`). The port's `update_world_grass_growth` carries the
underground slice; the overground dispatcher (`update_world_overground_tile`)
routed kind 70 through it (passing `true`), so the OVERWORLD case-70 arm
(`:74704-74750`) never ran on the surface. **Landed** (`growth.rs::
overworld_mushroom_growth`): the `!inActive()` gate, the `Next(10)` plant-71
roll + `CopyPaintAndCoating`, the `Next(300)` `TryGrowingTreeByType(5)` giant-
tree roll (the `& flag10` PlayerLOS conjunction evaluates the grow regardless
— the FX is unported), the 3×3 mud→mushroom-grass spread. The overground
dispatcher routes kind 70 to this arm INSTEAD of the underground slice
(routing through both would double the plant roll). The underground case-70
(`:74497-74506`, the `Next(250)` tree roll + the `PlaceCatTail` liquid branch
— c33's helper) and the other overworld case arms (the forest/evil/hallow/ash
grasses `:74513`, the jungle case-60 `:74752`) stay the shared dispatcher
follow-up (not mushroom-exclusive).

### TAIL landing ledger (one phase per commit+push)

| phase | scope | commit | status |
| --- | --- | --- | --- |
| T-1 | the tree-type riders: GrowTreeWithSettings WallTest/nactive/side-flags/draw-order/errorWorld/boot-bit, profile_for_tree per-kind, the sapling dispatch (ash return / bees-remix gate / gem style default), the palm lean gate (growth/trees.rs) | d13d05a1 | landed |
| T-2 | the glowing-mushroom drops: 71 plant record, grass→mud 176, dye-plant style ladder (edit/kill.rs + registry.rs + editing.rs) | 73a4e267 | landed |
| T-3 | the overworld mushroom runtime growth arm (growth.rs) | a964730f | landed |
| T-4 | docs/anchors/ledger (this pass) | (this commit) | landed |

All TAIL phases code-only (no cargo/tests/graphify). 10 witnesses WRITTEN NOT
RUN: growth/trees.rs (4 — gem wall, gem style default, ash return, gem grow +
boot bit), edit/kill.rs (1 — the 71 plant record), registry.rs (1 — grass→mud
+ 71/227 gates), editing.rs (2 — dye style ladder, grass→mud), growth.rs (2 —
overworld tree grow, plant + mud spread). The runtime-only changes (the tree
growth draw order, the drops, the overworld arm) touch NO worldgen pass stream
— both world-hash pins stay as G5 left them (stale-by-design, re-pin deferred
to the gate); the `a == b` determinism halves are untouched.

## Phase 7 — the shared-dispatcher follow-up (the full case census)

The TAIL's case-70 find generalised: the overground dispatcher
(`update_world_overground_tile`) routed EVERY `SpreadOverground` kind but 70
through `update_world_grass_growth` — the UNDERGROUND slice (`:74275-74508`,
returns at `:74508` BEFORE the overworld `switch (num10)` at `:74510`). The
C# overground caller passes `underground: false` (`:72357`); the underground
caller passes `true` (`:73282`). The full census of every case kind:

| kind | overworld arm (`:74510-74866`) | underground slice (`:74275-74508`) | port state at 8d367094 | verdict |
| --- | --- | --- | --- | --- |
| 2 | shared arm `:74513` (plant 3 Next(12), the dirt spread ladder) | NOT in the table (`TileID.cs:622`) | routed to the underground slice's phantom kind-2 row (maxValue 1, no plant) | WRONG both ways — D1/D2 |
| 23/199/661/662 | shared arm (plant 24/201 Next(10), the infection ladder + the mud twins) | table rows (maxValue 2, infection-gated) + the soil scan | overworld ran the underground rows (wrong plant odds, no cross-infection ladder) | WRONG overworld — D2 |
| 32/352 | shared arm via the thorn remap 32→23/352→199 (`:74594-74607`, frozen-spread early return) | not in the table (the caller's GrowSpike arm `:72359` still runs) | overworld ran the slice (no-op for 32/352) — the remap spread never ran | WRONG overworld — D2 |
| 477/492 | shared arm via the golf remap 477→2/492→109 (`:74608-74613`) | not in the table | overworld no-op | WRONG overworld — D2 |
| 109 | shared arm (plant 110 Next(10), the hallow ladder) | table row? NO — 109 IS in `SpreadUnderground` but has NO table row (the C# falls through with num=-1/num3=-1: no plant, no scan) | overworld ran the slice's no-op | WRONG overworld — D2 |
| 60 | case-60 `:74752` (Next(7) plant 61 / Next(500) tree / Next(25) rose + the mud scan) | table row (plant 61 maxValue 10) + the tail switch `:74416` (bulb 238 / fruit 236 / rose 233) | overworld ran the underground row; the tail switch missing everywhere | WRONG overworld + MISSING tail — D2/D4 |
| 70 | case-70 `:74704` (T-3, landed) | table row (plant 71 maxValue 10) + the tail `:74497` (cattail + Next(250) tree) | overworld fixed (T-3); the underground tail missing | MISSING tail — D4 |
| 226 | case-226 `:74815` (Next(7) + TooManyJunglePlantsNearby) | table row (plant 61 maxValue 10, the `:74359` density veto) | overworld ran the underground row (no veto); the veto missing underground too | WRONG overworld + MISSING veto — D1/D2 |
| 633 | case-633 `:74829` (Next(10) plant 637 + the ash-57 scan) | table row (plant 637 maxValue 2, the ash scan) | overworld ran the underground row (wrong odds) | WRONG overworld — D2 |
| 69 | not in SpreadOverground (the old port's overground kind-69 arm was DEAD CODE) | the caller's `:73658` GrowSpike(69,60) — g83's claimed case (fe5b5b8c, UNWIRED) | dead overworld arm removed; the underground arm stays g83's | g83 lane |

The underground slice's own gaps (D1): the phantom kind-2 row; the flat
plant write (the C# `PlaceTile` carries the flower ladder `:59507-59581`,
the jungle-61 arm `:59582-59629`, the mushroom-71 arm `:59651-59678` with the
below-surface cattail climb — each with its own draws); the missing
`CopyPaintAndCoating` (`:74367-74369`); the missing tail switch
(`:74414-74507`). The underground DISPATCHER's own gaps (D4/D5): the `return`
at the grass arm skipped the sapling arms (`:73756-73780` — the gemcorn 590's
runtime growth) and the wall arms (`:73809-73813`) for every grass/lily/vine
cell; the overground dispatcher lacked the wall arms entirely (`:72499-72503`).

**The acorn PLANT side (item 27) — censused, ALREADY LANDED**: `Item.cs:1697-
1710` (createTile 20, useStyle 1, consumable) rides the generated
`itemdata.rs` row (`tile: Some(20)`) through `use_tile_item` →
`place_item_tile` → the `TileObject.CanPlace`/`Place` registry route (the
D_20 def: 1×2, origin (0,1), the 10 soil alternates, RandomStyleRange 3 —
`TileObjectData.cs:4860-4909`). The `Place1x2` legacy arm (`:46652-46679`,
the Next(3) frameX) is the NON-registry path the C# `PlaceTile` case-20
(`:60013-60026`) uses for the smart-cursor/axe replant — the port's
`try_replanting_tree` (editing.rs) already rides the registry route too.
**The gemcorns are items 4851-4857** (`ItemID.cs:11385-11397`,
`GemTree*Seed`; NOT the 4490-range — those are the gem ECHO walls,
`Item.cs:39596+` neighbours), createTile 590 styles 0-6 (`Item.cs:39599-
39626`), the D_590 def (the stone-family anchors {1,25,117,203,182,180,179,
381,183,181,534,536,539,625,627}, StyleMultiplier 3 — `TileObjectData.cs:
4910-4930`). All seven rows landed in `itemdata.rs`; the placement route is
shared with the acorn. The gem sapling's GROWTH side (the underground
Next(5) arm) was the missing half — D5. **The sunflower item 63** (tile 27,
the d105 co-lane boundary) rides the same registry route (the D_27 def,
2×4) — landed, no port needed.

### Phase-7 landing ledger (one commit per case-family)

| phase | scope | commit | status |
| --- | --- | --- | --- |
| D1 | the underground slice: the kind-2 row removed, the PlaceTile per-kind plant arms + the 226 veto + paint copy + the tail call (growth/grass.rs) | (this pass) | landed |
| D2 | the overworld arms (growth/overworld.rs new) + the per-kind routing + the overground wall arms (growth.rs) | (this pass) | landed |
| D3 | the jungle helpers (growth/jungle.rs new) | (this pass) | landed |
| D4 | the underground tail + the runtime PlaceCatTail (growth/underground_tail.rs new) + the dispatcher no-return chain (growth.rs) | (this pass) | landed |
| D5 | the underground sapling arms — the gemcorn 590 runtime growth (growth.rs) | (this pass) | landed |
| D6 | docs/anchors/ledger (this pass) | (this commit) | landed |

All phase-7 changes code-only (no cargo/tests/graphify). 14 witnesses WRITTEN
NOT RUN: growth/overworld.rs (5 — the plain-grass plant roll + frame chain,
the evil ladder + frozen thorn, the jungle draw order, the ash roll + spread,
the 226 density veto), growth/underground_tail.rs (4 — the jungle tail draw
order, the mushroom Next(250), the cattail rise + gates, the life-fruit gate
+ break), growth/grass.rs (3 — the kind-2 skip, the 226 roll-before-veto, the
71 cattail-branch frame-draw skip), growth.rs (2 — the gemcorn underground
grow + surface control, the grass-cell wall-arm reach). The runtime-only
changes touch NO worldgen pass stream — both world-hash pins stay; the
`a == b` halves untouched. Co-lane boundaries honoured: g83's kind-69 thorn
+ vine grow-tops (fe5b5b8c) stay UNWIRED g83 lane; c33's worldgen
PlaceCatTail (fd7fae0e) untouched (the runtime copy is a separate fn over
UnifiedRandom); d102/d106's growth.rs re-export lines untouched; d105's
sunflower item row untouched.

## Phase 8 — GROWTH FINAL (the last-pass census + the tiny closes)

The full-fn final verify of `UpdateWorld_GrassGrowth` (`WorldGen.cs:74269-
74867`) every-case against the three port modules (`growth/grass.rs`
underground slice, `growth/overworld.rs` overworld arms,
`growth/underground_tail.rs` tail switch) after D1-D5. The case bodies are
1:1 (the plant rolls' odds/order, the `&` non-short-circuit draws, the
spread ladders' call order, the tail's flag/break semantics — re-read line
by line). The residuals found and closed:

| # | residual | C# anchor | close |
| --- | --- | --- | --- |
| F1 | the D6 wall-arm residual: both dispatchers returned early on the inactive path; the actuated (`nactive()` false) path ran the growth chain | `:72499-72503` / `:73809-73813` sit OUTSIDE the if-else; `:72216`/`:73252` gate on `nactive()` | the inactive path runs the tail-else web + the wall arms; the actuated path skips the chain, runs web + walls; the underground `nactive()` body split into `underground_nactive_arms` so the alch arm falls through to the walls; the overground alch/vanity-kill/cactus/sand/moss arms no longer return |
| F2 | the overground window unclamped — at the pick j=10 the port read row 9 and spent the plant roll; the C# minJ is the source cell itself | `:72032-72051` (the clamp before any arm) | the clamped min/max window threaded through every grass call + the moss arm |
| F3 | the overworld arms lacked the entry guard | `:74271` `InWorld(i,j,10)` sits before the `if (underground)` split | all four overworld arms + T-3's mushroom arm carry it, plus the callers' `nactive()` gate folded in |
| F4 | the vanity-kill below-gate checked `active()` | `:72146` reads the TYPE only | the extra active check removed |
| F5 | `SpreadGrass`'s plain write replaced the whole Tile (wall/liquid zeroed) | `:75223` `tile.type = grass` — a field write | the kind-only write (the vine-arm precedent) |
| F6 | `SpreadDesertWalls`: an extra HardenedSand-TILE entry check; the flavour source gated on active; the num6 neighbour-family gate missing | `:74986` (Sandstone wall/tile + HardenedSand WALL only), `:74993-75001` (no active check), `:75088-75122` (the Sandstone/HardenedSand neighbour gate + the same-wall skip) | all three closed (RNG-neutral — the gates sit after the ±2 draws) |

**The actuated-grass census verdict (F1)**: actuated grass grows NO grass
(the `nactive()` gates) but its evil WALL still spreads (the wall arms'
source test reads `tile.wall` when `tile.active()` is off, and an actuated
tile keeps both bits). The inactive cell's type-keyed head arms
(`:72146-72215` vanity-kill/bamboo/lily/cattail/liquid — they run on `.type`
regardless of the active bit) stay parked (ANALYSIS); the web arm
(`:72491`/`:73803`) now reaches them.

**The mushroom-CONVERT runtime census (corrected)**: Convert(3) has FOUR
call sites — the three gen-only `DoSurfaceIsMushrooms` arms (`:2101/:2125/
:2129`, G6/G7 landed) AND `Projectile.cs:26834` (the aiStyle-31 Clentaminator
spray: the ladder `:26791-26825` maps proj 148 → type 3; the shoot math is
`Player.cs:53487-53490` `projToShoot += item.shoot` — Clentaminator 779
shoot 145 + DarkBlueSolution 783 shoot 3 = 148). The phase-6 "three call
sites" verdict missed the variable-typed site. The UpdateWorld side has NO
Convert(3) arm — the runtime grass-70 spread is the `SpreadGrass(59, 70)`
mud scan, covered by T-3 (overworld case-70 `:74731-74745`) + D1
(underground soil scan `:74377-74413`) + D4 (the tail). The spray side is
the projectile lane's park: `ai_spray` ports the ladder, runs only the
type-0 cleanse (the 146-149/1015-1017 solutions unrowed);
`spread::convert_cell_mushroom` (G7) stands ready for that wiring.

**The tree-GROWTH chain verified complete (sapling→tree tick)**: the acorn
item 27 / gemcorns 4851-4857 place through the TileObject registry
(`itemdata.rs` rows + the D_20/D_590 defs) → the overground Next(20) arm
(`:72431-72436`) / the underground Next(5) arms (`:73756-73780`, D5 — the
gemcorn 590 included) → `attempt_grow_tree_from_sapling`
(`:72810-72924` verbatim: the style switch, the underground gates with the
bees/remix exemption, the gem style≥7 → 587 default) →
`try_growing_tree_by_type` (`:30343-30384`: case-5 → `grow_tree`
(`:29767-29980`, G5 verbatim), 323 → `grow_palm_tree`, the gem/vanity/ash
profiles → `grow_tree_with_settings` (`:30388-30847`) with the T1 WallTest
delegates). The d105/g17 tree-FX co-lanes (`PlayerLOS`/`TreeGrowFXCheck`)
stay unported — the `&`/`&&` conjunctions evaluate the grow regardless.

### Phase-8 landing ledger

| phase | scope | commit | status |
| --- | --- | --- | --- |
| F1-F6 | the tiny closes (growth.rs both dispatchers + the split body, growth/overworld.rs entry guards, growth/grass.rs field write + the desert-wall gates) + 5 witnesses + ANALYSIS/12j | (this commit) | landed |

All phase-8 changes code-only (no cargo/tests/graphify). 5 witnesses WRITTEN
NOT RUN: growth.rs (3 — the actuated/inactive wall-arm reach + no-grass
control, the margin clamp draw-skip, the acorn end-to-end grow),
growth/overworld.rs (1 — the entry guards skip margin + actuated),
growth/grass.rs (1 — the plain write keeps wall/liquid). Runtime-only — no
worldgen pass stream touched, both world-hash pins stay; the `a == b`
halves untouched. Co-lane boundaries unchanged (g83/c33/d102/d106/d105).
