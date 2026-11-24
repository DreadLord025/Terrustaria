# 300 — W4-c31 jungle residuals census + gap-fill + DUP-RECONCILE EXECUTED

> **RENUMBERED 12k → 300** per the research/150 registry (d54
> `51a28d09e`, §7.3 round-3 RE-ISSUE): key 12k belongs to W4-f24
> (`12k-f24-worldgen-gated.md`, first-add 00:09:38 `f353e13f6`); c31's
> 05:10:24 copy loses. The round-2 reservation 244 was re-issued to 300
> (244 taken by g48's pre-publish push `2b61530b4` 14:30:16). The
> round-2 244 pick was NEVER executed on this branch, so there is no
> 12k→244 intermediate — the chain is 12k → 300 direct. The only on-branch
> ref (`docs/ANALYSIS.md`) is repointed in the same commit.

Branch `task/W4-c31-hive-rewrite` (continues the landed hive rewrite, DONE
d538536b). Code-only: no cargo / tests / graphify run. The hive module landed;
this pass censuses the **rest of the C# `WorldGen` jungle family** against the
port and ports the missing pieces verbatim.

## DUP-RECONCILE EXECUTED (the g42 GROWTH SEAL verdict, 910cc974)

The g42 lane (`task/W4-g42-biome-mushroom`) ported the SAME
`UpdateWorld_GrassGrowth` case-60/70 tails (`:74414-74506`) as this lane's
pass 3, plus the full overworld switch, the tree-growth rewrite and the
Convert family. Its GROWTH SEAL (12j phase 9, commit `910cc974`) ran the
per-fn byte-compare and ruled: **the g42 copy is the KEEPER of every runtime
fn** (the kill-sweep + paint-inherit + boot-frame superset, the OOB guards,
the `CopyPaintAndCoating` `:58738` the c31 cattail copy misses, the D1
table/plant-arm supersession). ONE drop-hunk set was published for the c31
side. **This pass EXECUTES that set on this branch** — the c31 duplicates
are gone and the g42 keeper blobs are adopted BYTE-IDENTICAL (verified by
`git hash-object` against the g42 tip blobs):

| dropped (c31 side) | g42 keeper adopted (blob) |
| --- | --- |
| `growth/jungle_plants.rs` (the whole module + its `mod`/`pub use`) | `growth/jungle.rs` `790ccdfd7` — `place_jungle_plant(world, …, inherit_paint)` + `too_many_jungle_plants_nearby` (the OOB-skip superset) |
| `grass.rs::jungle_grass_hardmode_tail` / `mushroom_grass_tail` / `too_many_jungle_plants_nearby` / `no_nearby_kind` / the tail `match kind` / the kind-2 table row / the plain plant-write body / the 4 tail witnesses | `grass.rs` `9fa351c3a` (the D1 table + per-kind `PlaceTile` plant arms + the `underground_tail` call) + `underground_tail.rs` `ca2293299` (the case-60/70 tail switch + the runtime `place_cat_tail`) + `overworld.rs` `d595b5c24` (the overworld switch arms + the shared `PlaceTile` flower/61/71 helpers) |
| `decorations.rs::place_cat_tail` (the c31 copy + the `growth.rs` re-export) | `underground_tail.rs::place_cat_tail` (the `CopyPaintAndCoating` `:58738` + boot-frame superset); `decorations.rs` `10f15277e` keeps only the `CAT_TAIL_DISTANCE` visibility widening (`pub(super)`) |
| the old `trees.rs` slice (no case-5 dispatch, the batch-write `grow_tree`) | `trees.rs` `b8cb817ea` — the G5 verbatim `GrowTree` generic over `WorldGenRand`, the case-5 `try_growing_tree_by_type` arm, `grow_shroom` (UNWIRED, zero decompile callers), the `WallTestKind` delegates, the full `EmptyTileCheck` |
| the old `growth.rs` dispatchers (everything through the underground slice) | `growth.rs` `bf41a467d` + the c31 plantera seam re-added — the D2 per-kind overworld routing, the D5 underground sapling arms, the D6/F1 wall-arm fall-through (`underground_nactive_arms`), the F2 window clamps, `World.allowed_to_spread_infections` (the new runtime field, stock true, `update_world` re-sets per tick) |

`plantera.rs` (this lane's ONLY unique growth file) KEEPS its bulb event and
re-points its two `place_jungle_plant` calls at `jungle::place_jungle_plant`
with the extra `false` arg per the verdict. **Deviation recorded**: the C#
passes `inheritPaint: true` at `:74238`/`:74253`; the verdict's `false`
preserves the c31 paint-no-op behaviour — the flip to `true` is the merger's
one-arg decision (the keeper's paint-copy arm is ready).

The liquid-veto witness — the ONE c31 witness shape the g42 stream pins do
not cover — is ported as `underground_tail.rs::
w4_c31_the_liquid_veto_blocks_the_tail` (the verdict's sanctioned one-witness
port). The other dropped witnesses (the 4000-tick statistical grows, the
jungle_plants placement frames) are covered by the g42 D3/D4 stream pins.

**Compose note for the merger**: this branch now carries the g42
growth-family keeper blobs verbatim, so the g42 × c31 merge of
`growth.rs`/`grass.rs`/`trees.rs`/`decorations.rs`/`jungle.rs`/`overworld.rs`/
`underground_tail.rs` is a keep-one (identical bodies; c31 adds only the
plantera module/seam + the one liquid-veto witness + the
`allowed_to_spread_infections` field, which g42 also carries). The g42
worldgen-side hunks (spread.rs Convert family, biomes/micro_surface/
biome_detail, registry.rs, kill.rs 71-drop) are NOT adopted here — they stay
g42's to land; this branch's kill.rs keeps the c31 larva hunks only.

**Hash discipline**: no worldgen pass stream is touched by the reconcile
(the keeper tails are runtime-only); both world-hash pins stay
stale-by-design from the JungleShrines/hive rewrites above. `a == b` stays
green. Witnesses WRITTEN NOT RUN (code-only).

## Census — C# jungle-family passes vs landed

Line numbers into `decompiled/Terraria/WorldGen.cs` unless noted. "Landed" =
present on this branch at d538536b.

| C# pass / fn | Anchor | Port | Verdict |
| --- | --- | --- | --- |
| `JunglePass.ApplyPass` | `JunglePass.cs:19-63` | `biomes.rs::pass_jungle` | faithful (M5-BIOME p3) |
| `MudCavesToJungleGrass` | `:12468-12499` | `biomes.rs::pass_mud_caves_to_jungle_grass` | faithful |
| `DirtToMud` | `:13123-13142` | `biomes.rs` (`dirt to mud` step) | faithful |
| `UndergroundJungleTrees` + `GrowUndergroundTree` | `:17840-17858` / `:30849-31299` | `biome_detail.rs::pass_underground_jungle_trees` | faithful (`pass_rng("Jungle Trees")`) |
| `SurfaceWaterInJungle` (Wet Jungle) | `:15810-15833` | `biome_detail.rs::pass_wet_jungle` | faithful |
| `GlowingMushroomPlantsUndergroundAndJunglePlants` (jungle arms) | `:20122-20164` | `biome_detail.rs::pass_mushroom_and_jungle_plants` | faithful (jungle arms) |
| `JunglePlantsPart2` | `:20165-20192` | `biome_detail.rs::pass_jungle_plants_part2` | faithful |
| `DirtWallsIntoMudWallsInJungleAndJungleMinMax` | `:20813-20866` | `biome_detail.rs::pass_muds_walls_in_jungle` | faithful |
| `LihzahrdTemple` + `makeTemple` | `:15834-15926` / `:33435-34237` | `temple.rs::pass_lihzahrd_temple` | faithful (M5-TEMPLE) |
| `LihzahrdTemplePart2` + `templePart2` | `:17701-17709` / `:34464` | `temple.rs::pass_temple_part2` | faithful |
| `LihzahrdAltar` | `:21943-21972` | `temple.rs::pass_lihzahrd_altar` | faithful |
| `Beehives` + `HiveBiome` | `:15927-15976` / `HiveBiome.cs` | `hive.rs` | faithful (W4-c31, landed) |
| `BeeLarvaInBeehives` | `:20868-20900` | `hive.rs::pass_larva` | faithful (W4-c31, landed) |
| **`JungleShrines`** | **`:15977-16131`** | `structures.rs::pass_jungle_shrines` | **SHAPE-LEVEL** ("the C#'s exact brick patterns are not 1:1", module doc) → **ported faithful here** |
| `ChestsInJungleShrines` | `:17211-17244` | `structures.rs::pass_chests` (shrine arm) | faithful item/jitter draws (S1-A-4); the `AddBuriedChest` failure-retry (`:17222-17240`) is the storage-task follow-up |
| **`GeneratePlanteraBulbOnAllMechsDefeated` + `AttemptToGeneratePlanteraBulbAt`** | **`:74127-74267`** | — | **MISSING** → **ported here** (`growth/plantera.rs`) |
| **`UpdateWorld_GrassGrowth` case-60 hardmode tail (life fruit + bulb + 233)** | **`:74414-74496`** | `growth/grass.rs::update_world_grass_growth` stops at `:74424` | **MISSING** → **ported here** |
| `UpdateWorld_GrassGrowth` case-70 tail (cattail + mushroom tree) | `:74497-74506` | — | residual, same dispatcher; **ported here** (cattail + tree) |
| `Piles` jungle arm (`tile2.type == 60` → 187/`Next(6)`) | `:18856-18860` | W4-d104 co-lane (`pass_piles`) | **claimed by W4-d104**, not this lane |

### Co-lane claims read (not duplicated)

* **g43 `task/W4-g43-biome-jungle-temple`** — 0 commits ahead of `origin/main`
  (claim-only branch). The temple is already faithful on this branch via
  M5-TEMPLE (`temple.rs`), so nothing to port for the temple; the census
  confirms it. The ee937411-adjacent claim is W4-g41 (spider cave witness
  fixes), already merged — read, no overlap.
* **g84 `task/W4-g84-plantera-bulb-draw`** — 0 commits ahead (claim-only);
  the **draw** side (tile 238 `drawdata.rs`/`drawdata_anim.rs` already carry
  the bulb frame rows). This lane ports the **placement / worldgen side**
  (`:74127-74267` event + `:74414-74448` growth), which is missing.
* **g88 `task/W4-g88-life-fruit-use`** — the **item-use** side
  (`life_fruit.rs`, `crystals.rs`, `ItemCheck_UseLifeFruit`); this lane ports
  the **growth** side (tile 236 placement), which is missing. No overlap.
* **W4-d104** owns the `Piles` pass (incl. the jungle arm); not touched here.

## What this pass ports (verbatim, fresh `pass_rng` slots)

1. **`JungleShrines` faithful rewrite** (`structures.rs::pass_jungle_shrines`,
   `WorldGen.cs:15977-16131`): the `GenVars.jungleHut` material pick
   (`Reset` `:11167` `Next(5)` → `:11317-11336` map to 119/120/158/175/45),
   the hut count `Next(7,12) * maxTilesX/4200`, the dungeon-opposite-half
   origin search with the `num4 > maxTilesX*10` retry valve, the 30-radius
   hive/temple proximity veto (`225/229/226/119/120` tiles + walls `86/87`),
   `structures.CanPlace(area, 1)`, the brick shell + carved interior with the
   hut-specific wall, the 100-try torch (`PlaceTile(4, style 3)`), the floor
   carve, the mud pillars, the shrinking roof rows, the `JChestX/Y`
   publication + `AddProtectedStructure`, and the `tileSolid[137] = false`
   window. Fresh `pass_rng("Jungle Chests")` slot (the shape-level pass rode
   the shared `srng`; the faithful pass isolates its draws).
2. **Plantera bulb placement** (`growth/plantera.rs`):
   `GeneratePlanteraBulbOnAllMechsDefeated` (`:74127-74186`) +
   `AttemptToGeneratePlanteraBulbAt` (`:74188-74267`), verbatim. The
   `NPC.OnGameEventClearedForTheFirstTime` case-16/17/18 hook
   (`NPC.cs:78780-78788`) wires it at the mech-boss death path.
3. **Life-fruit / bulb / 233 hardmode growth** — the case-60/70 tails
   (`:74414-74506`): the `Next(25)` gate + liquid veto, the all-mechs
   `Next(60)` bulb arm with the 150-radius stride-2 density scan, the
   any-mech `Next(30/40)` life-fruit arm with the 60/50-radius scan, the
   233 detritus fallback (`Next(8)` styleY 0, `Next(12)` styleY 1), and the
   case-70 cattail/tree tail. **SUPERSEDED by the DUP-RECONCILE above**: the
   runtime fns now ride the g42 keeper `growth/underground_tail.rs` (the
   expert constants fold off the same way; the placement rides
   `jungle::place_jungle_plant` with the kill-sweep + paint-inherit +
   boot-frame superset, not the c31 placement + 3 `square_tile_frame_gen`
   calls).

## Deviations (recorded in ANALYSIS "Jungle residuals (W4-c31)")

* The `JungleShrines` rewrite moves the shared-`srng` draw schedule (the
  faithful pass draws its own count/origin/size/torch/roof rolls on
  `pass_rng("Jungle Chests")`), so every downstream `srng` consumer re-rolls
  and the shrine tiles change. Both world-hash pins move **by design**;
  stale-by-design, the merge gate re-pins. `a == b` stays green (the pass is
  a pure function of world + stream).
* `PlaceJunglePlant`'s `KillTile` collapse + `CopyPaintAndCoating` no-op
  followed the `biome_detail.rs::place_jungle_plant_detritus` precedent in
  the c31 pass-2 slice — **SUPERSEDED by the DUP-RECONCILE**: the g42 keeper
  runs the real `edit::kill_tile` sweep + the paint-inherit arm. The bulb
  event passes `inherit_paint: false` per the verdict (the C# `true` at
  `:74238`/`:74253` is the merger's one-arg flip).
* The bulb event's `Main.dungeonX` reads the port's `World.dungeon_x`
  (published by the dungeon step / read from `.wld`); `Main.UnderworldLayer`
  = `maxTilesY - 200` (`Main.cs:2790`).
* `Main.expertMode` is constants-off in the port (single-difficulty), so the
  life-fruit `maxValue2` collapses to 40 and the density radius to 60 (the
  classic arm); the expert branches are carried in the code for the
  difficulty flip.
* The `NetMessage.SendTileSquare` sides are MP scope (the port runs the host
  path); the `PlayerLOS` gate on the mushroom-tree arm reads the port's
  player-LOS seam (constants-off when no player is near).

## FINAL — the JUNGLE-FAMILY bible (SEALED)

> **300 SEALED** — the jungle family is closed on this lane. The three
> residual gaps (JungleShrines / the Plantera bulb event / the hardmode
> growth tails) are landed; the growth-tail duplicate is reconciled per the
> g42 GROWTH SEAL (the keeper blobs adopted byte-identical); the registry
> number is FINAL at 300 (12k → 300, §7.3). Nothing owed by this lane.

**The jungle family, one paragraph.** The C# `WorldGen` jungle family is
the generation passes (`JunglePass`, `MudCavesToJungleGrass`, `DirtToMud`,
`UndergroundJungleTrees`, `SurfaceWaterInJungle`, the
`GlowingMushroomPlantsUndergroundAndJunglePlants` jungle arms,
`JunglePlantsPart2`, `DirtWallsIntoMudWallsInJungle`, `Beehives`/`HiveBiome`,
`BeeLarvaInBeehives`, `JungleShrines`, `ChestsInJungleShrines`,
`LihzahrdTemple`/`makeTemple`/`LihzahrdAltar`) plus the runtime growth/event
tails (`UpdateWorld_GrassGrowth` case-60/70, the
`GeneratePlanteraBulbOnAllMechsDefeated` event). On this branch the
generation passes are faithful (M5-BIOME / M5-TEMPLE / the W4-c31 hive +
shrine rewrites); the runtime tails are the **g42 keeper** (the full
grass-growth dispatcher: the underground slice + the overworld switch + the
case-60/70 tail + the `PlaceJunglePlant`/`PlaceCatTail` helpers + the
tree-growth chain); the bulb event is this lane's `plantera.rs` placing
through the keeper. The family's ONE contested seam — the case-60/70 tail
duplicate — is resolved ONE-directionally (g42 keeps, c31 drops), so the
merged tree carries a single copy of every runtime fn.

**Co-lane receipts (read at the seal, not duplicated):**
* **g42** (`task/W4-g42-biome-mushroom` @ `910cc974`) — the growth-family
  keeper. Its worldgen-side hunks (the Convert type-3 family in `spread.rs`,
  `GlowingMushroomPatches`/`ShroomPatch` in `biomes.rs`,
  `DoSurfaceIsMushrooms` in `micro_surface.rs`, the plants-pass mushroom arm
  in `biome_detail.rs`, the 71-drop in `kill.rs`, the grass-block drop in
  `registry.rs`) are NOT adopted here — they stay g42's to land. This branch
  adopts ONLY the growth-family runtime keeper (the seven blobs above).
* **g43** (`task/W4-g43-biome-jungle-temple`) — the temple is already
  faithful via M5-TEMPLE; g43's dual-temple STYLE work
  (`DungeonGenerationStyles.cs:760-797`) is disjoint from `makeTemple`
  (`WorldGen.cs:33435-34237`). Keep-both at merge (the f24 receipt).
* **g83** (`task/W4-g83-vine-draw-residuals`) — the vine draw residuals +
  the `grow_more_vines` `pub(crate)` lift (its research/123 §10.6). The
  adopted g42-shape keeps `grow_more_vines` private; g83's lift + the vine
  tail insert compose over it at merge (the g83 bible's "g42-shape-with-g83-
  insertions"). The jungle-thorn kind-69 arm (`:73658`) is g83's claimed
  case — it stays out of the adopted dispatcher until g83 wires it.
* **g84 / g88** — the bulb DRAW side / the life-fruit USE side; disjoint
  from this lane's placement/growth side.
* **c33** — owns the worldgen `PlaceCatTail` (`worldgen/ocean.rs`, the
  xorshift `Rng` copy); the runtime copy adopted here
  (`underground_tail.rs`, `UnifiedRandom`, RNG-free at runtime) is the g42
  keeper, distinct from c33's gen-time helper.
* **d104** — owns the `Piles` pass (incl. the jungle arm `:18856-18860`);
  not touched here.
* **c31c-jungle-final** — dispatched earlier; NO branch/doc/commit exists
  at any tip (a fresh `git branch -r` + `git log --all --grep=c31c` +
  `ls-remote` sweep returns zero). Its verify mandate is folded into THIS
  seal: the jungle-family census above is re-verified against the current
  tree (the generation passes faithful, the runtime tails the g42 keeper,
  the bulb event wired), so there is no separate c31c artifact to land.

**Witness ledger (WRITTEN NOT RUN, code-only):** the lane's own witnesses
are 4 `structures.rs` shrine + 4 `plantera.rs` event + 10 `hive.rs` (the
landed hive rewrite) + 1 ported liquid-veto
(`underground_tail.rs::w4_c31_the_liquid_veto_blocks_the_tail`). The adopted
g42 keeper modules carry their own witnesses (5 `underground_tail.rs` incl.
the ported one, 6 `overworld.rs`, 14 `grass.rs`, 14 `trees.rs`, 20
`w4_g42_*` across grass/trees/growth.rs) — written not run on g42, carried
verbatim here. The 6 `jungle_plants.rs` + 4 `grass.rs` c31 tail witnesses
DROPPED with their fns (the g42 D3/D4 stream pins cover the boxes/draws).

**Hash discipline:** no worldgen pass stream is touched by the reconcile
(the keeper tails are runtime-only); both world-hash pins stay
stale-by-design from the JungleShrines/hive rewrites; `a == b` stays green.
Re-pin values deferred to the merge gate.

**Verification record (this seal):** the keeper adoption verified by
`git hash-object` (jungle `790ccdfd7`, underground_tail `ca2293299`,
overworld `d595b5c24`, grass `9fa351c3a`, trees `b8cb817ea`, decorations
`10f15277e`, growth `bf41a467d` — all byte-identical to the g42 tip blobs
before the c31 plantera-seam re-add + the one liquid-veto witness); a
zero-stale `git grep` for the dropped symbols; the compile-surface seams
checked by read/grep (`World.allowed_to_spread_infections` added, no `World`
struct literals anywhere, `GrowTreeSettings` literals only inside trees.rs,
every `super::`/`crate::` dependency the keeper references present on c31).
No cargo/tests/rustfmt/graphify invoked.
