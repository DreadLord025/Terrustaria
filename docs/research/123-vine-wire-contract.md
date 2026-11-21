# 123 — the vine wire contract (W4-g83 wiring phase)

**Scope:** the per-function attach seams for W4-g83's four residual groups
(rising-vine draw pass, vine lifecycle checks, grow-tops/jungle-thorn, thorn
down-merge), written on `task/W4-g83-vine-draw-residuals` after the four
code-only phases (`8dbd449f`/`0c380c31`/`d889e3c6`/`fe5b5b8c`). The
UNCONTESTED small wires landed on this branch in the wiring phase (§5); the
contested seams below are the wiring wave's script. All C# claims were
re-read from `decompiled/Terraria/*.cs` at the cited lines; all sibling-lane
claims from `git show`/`git diff` of the pushed tips (base `a725acb0`).

Sibling tips read for this contract:

| Lane | Tip | Files that matter here |
|---|---|---|
| g31 union (EXECUTED) | `c3027a38` | `render/tile_pass.rs` (g29 base), `render/special_points.rs` (g30), `render/tile_drains.rs`, `render/tile_draw_pass.rs` (g32 reduced), `torch.rs` keep-one; `docs/UNION-CONTRACT-tilepass.md` §1–§8 |
| d102 worldgen-passes-2 | `29e8859b` | `growth.rs` +1 (`pub(crate) use trees::ALLOWS_PLANTS_TO_GROW`), `growth/trees.rs` ±1 (pub lift); worldgen passes only |
| d106 worldgen-passes-6 | `e2c2609c` | `growth.rs` +4 (`pub(crate) use trees::kill_invalid_tree_above`); worldgen passes only |
| g42 biome-mushroom | `d4836bbd` | `growth.rs` ±2 (re-export `grow_shroom`/`WallTestKind`), `growth/trees.rs` +624 (GrowTree rewrite); gen-time plants-pass mushroom arm |
| d109 framing-genrand-a3 | `81894095` | `framing.rs` +134 — `frame_world`/`frame_walls` variant-reuse gates (`variants_from_save`), save v18; NO `frame_of_variant` body edit |
| d110 framing-selframe8way | `054e3dc2` | `framing.rs` +438 — `self_frame_8way` family inserted at `frame_of_variant`'s HEAD (base `:818` hunk +13), `wall_frame_head`, `reframe_area` wall-arm rework (`:1257`/`:1277` hunks) |
| g48 net-apply-arms-1 | `9e037b7c` | `framing.rs` +75 — NEW `section_tile_frame` fn appended AFTER `reframe_area` (base `:1313` hunk); net apply arms; NO existing-framing-body edit |
| d58 chest-frames (WIRED) | `a13635dc` | `tiles.rs` hunks at `:325` (signature), `:586` (chest map), `:731` (chest rewrite), `:1873` (tests) |
| d59 platform-slopes (WIRED) | `f70498be` | `tiles.rs` hunks at `:1017`/`:1575`/`:2225` (slope family) |
| d56/d57/d60 | `5d80b38a`/`70f43aa7`/`77429fe1` | EMPTY diffs on `tiles.rs`/`vines.rs`/`torch.rs` (verified) — their contest is the g31-union rewrite, not textual hunks |

**Key correction to the phase-2/3 UNWIRED notes:** `CheckVines` and
`CheckUnderwaterPlant` are NOT growth-tick arms. Their only C# call sites are
in the **framing dispatch** (§2). The growth tick reaches them indirectly
through `SquareTileFrame` after a vine write. The d102/d106/g42 "contest" on
`growth.rs` is three one-line re-exports — the tick-driver functions
(`update_world_overground_tile`/`update_world_underground_tile`) and
`world_sim/growth.rs` are untouched by all three lanes (verified empty diffs
on those functions), so §3's insertions are add-only and low-conflict.

---

## 1. `draw_reverse_vines` → the tile draw pass

### 1.1 The C# position

* Registration: `TileDrawing.Draw`'s per-cell switch, `case 549:` →
  `CrawlToBottomOfReverseVineAndAddSpecialPoint(i, j)` + `continue`
  (`TileDrawing.cs:546-548`), immediately after the eight vine cases
  (`:536-545`). The crawl runs BEFORE any light/visibility/dark-cell gate.
* Draw: `PostDrawTiles(solidLayer: false)` step 10 — `DrawVines()` (`:429`)
  then `DrawReverseVines()` (`:430`), then `spriteBatch.End()` (`:431`) and
  `_natureRenderer.DrawAfterAllObjects` (`:433`). The solid layer never runs
  it. `DrawReverseVines` (`:8903-8914`) walks the `ReverseVine` bucket
  (`int num = 7`, `:8907`) in registration order.

### 1.2 Placement in the g31-union step map — COMPATIBLE

The union's map already carries this pass end-to-end; the g83 bodies bind
without changing any order:

| Union seam (UNION-CONTRACT-tilepass §4/§7) | g83 bind |
|---|---|
| §4 step 4 classify: g30 `classify_tile` `549 => ReverseVineCrawl` (`special_points.rs:209`) + `apply_cell_class`'s arm (`:389-392`) | g30's registry-side `collect_reverse_vine_root` (`:333-353`) and g83's `vines::collect_reverse_vine_root` are the SAME `:764-782` port (start AT `j`, root `k-1`, `top_edge_can_be_attached_to`, dedup). **Keep-one: mirror the forward pattern** — g30's `VineCrawl` arm already delegates to `vines::collect_vine_root` (`:379-387`); rebind the `ReverseVineCrawl` arm to delegate to `vines::collect_reverse_vine_root(world, x, y, &mut reg.reverse_vine_roots_seen, &mut roots)` and copy new roots into bucket 7, dropping g30's inline crawl body (its 3 crawl witnesses rebind to the vines.rs fn). |
| §4 step 6 post: g29 `post_draw_tiles` → `PostDrawStep::DrawReverseVines` → g31 `drain_pass_for_post_step` → `DrainPass::ReverseVines` → `drain_one_pass` → `drain_reverse_vines` over `BUCKET_REVERSE_VINE` (= `TileCounterType::ReverseVine` = 7) | `drain_reverse_vines` (`tile_drains.rs:625-636`) calls g31's `rising_vine_strip` per root. **Keep-one on the strip maths: g83's `vines::rising_vine_strip_segments`** (the two are line-for-line the same `:9134-9193` port — same +16+2 anchor, `num7 > 10` walk, no-remix gate, `(40, -0.004)` push, negated ladder, `(rot − PI/2)` step; g31's `drain_draw_data` == g83's inline `draw_data(kind, x, y, fx, fy, 0, solid_above)` + half-brick probe). Two REAL deltas to reconcile, both in g31's favour: (a) g31 takes `show_invisible` and reads the full `IsVisible` (`:1411-1432` — platform row 48 + 541/631 arms); g83 reads only `world.invisible_block`. At wiring, g83's `visible` field binds to the d56 `tr_render` `is_visible` keep-one (union §1.4) with the caller's `show_invisible`; (b) g31 emits `RisingVineSegment` into a Vec for the light seam (`DrainCellLight` → d56), g83 pushes sprites directly. Resolution: keep g83's `draw_reverse_vines` as the SPRITE tail (it owns `resolve_tile_frame` + the bottom-centre origin + the no-glowmask rule), fed by g31's drain segments; g31's `rising_vine_strip` drops (its 2 strip witnesses rebind to `rising_vine_strip_segments`). |
| §4 step 6 `EndBatch`/`NatureAfterAllObjects` | unchanged — the rising pass is the LAST nature step before them; no `_rand` draw exists in either rising-strip port (the C# `DrawRisingVineStrip` rolls none), so the shared-stream rule is trivially satisfied. |

### 1.3 The interim fused-pass wiring (LANDED, §5)

Until the union's `draw_layer` replaces the fused pass, this branch wires the
pass into `render/tiles.rs`'s own seams at the same C# positions:

* root lists declared beside `vine_roots` (`tiles.rs:589-597` — the
  `_reverseVineRootsPositions` per-draw rebuild, `ClearSpecialBlockCounts`
  `:391`);
* the crawl arm right after the forward `collect_vine_root` call
  (`tiles.rs:648-661`), before the draw-data dispatch and every light gate;
* the inline skip widened to `if vine_cell || reverse_vine_cell`
  (`tiles.rs:867-872`, the `:545`/`:548` `continue`s);
* `draw_reverse_vines` called right after `draw_vines` at the foliage tail
  (`tiles.rs:1218-1231`, the `:429-430` order), same `vine_wind_counter`.

At the union merge these four hunks are REPLACED by the §1.2 binds (the
crawl → g30 classify, the tail → the post-step drain); the two source-order
witnesses (`w4_g83_the_reverse_crawl_rides_the_switch_and_skips_the_inline_draw`,
`w4_g83_the_rising_pass_draws_after_the_hanging_pass`) retire with the fused
pass, their assertions covered by g29's `post_draw_orders` + g31's bucket
witness.

**Merge-conflict census for the interim hunks:** d58's `:586` chest-map hunk
is ADJACENT to the root-list declaration (context overlap, trivial union —
keep both blocks); d58's `:731` and d59's `:1017` hunks sit between the crawl
and the skip but do not touch either; the `:1173` tail region is untouched
by d58/d59 (their nearest hunks are `:1017` above and `:1575` below).
d56/d57/d60 have EMPTY `tiles.rs` diffs. No semantic conflict: the 549 cell
previously drew inline through `DrawSingleTile` (wrong — the C# `continue`s),
now it draws as a strip; no other lane changes 549's draw.

## 2. `check_vines` / `check_underwater_plant` → the framing dispatch (NOT a tick arm)

### 2.1 The exact C# call sites (re-read, this phase)

* `CheckVines` — ONE call site: `WorldGen.TileFrame(i, j, resetFrame,
  noBreak)` (`WorldGen.cs:81328`), non-solid arm: after the `tileSolid`
  gate (`:81361`) and the 49/80 switch (`:81363-81374`),
  `if (TileID.Sets.IsVine[type] && !WorldGen.CheckVines(i, j, type, tile))
  return;` (`:81375-81378`), then `TileFrameCosmetic` (`:81380-81383`).
  None of the 8 vine kinds is `tileFrameImportant` (no
  `Main.tileFrameImportant[52/62/115/205/382/528/636/638]` write exists in
  `Main.cs`), so they always ride this arm. `CheckVines` draws NO RNG.
* `CheckUnderwaterPlant` — ONE call site: `TileFrameImportant`'s `case 549:`
  (`WorldGen.cs:84972-84974`), reached from `TileFrame`'s frame-important
  dispatch (`:81353-81359`). 549 IS frame-important (`Main.cs:7373`).
  Draws `genRand.Next(1,8)` / `Next(7,13)` ONLY when a tip column disagrees
  with the neighbours (`:58601`/`:58612`).
* The growth tick reaches both only THROUGH framing: every vine write calls
  `SquareTileFrame(i, j+1)` (`:72555`, `:72610`, …), whose 3×3 `TileFrame`
  walk runs the checks on the touched cells. `GrowCheckSeaweed`
  (`:58917-58973`) does NOT call `CheckUnderwaterPlant` directly — its
  `PlaceTile(x, y-1, 549)` (`:58965`) frames via the placement path.

### 2.2 The Rust attach seams

The port splits `TileFrame` into two halves: the CHECK dispatch lives in
`edit/kill.rs::square_tile_frame` (`:384-509` — the tree/72/cactus/object
arms), the cosmetic framing in `framing.rs::frame_of_variant` via
`reframe_area`. The checks belong in the FIRST half:

* **`check_vines`** → `edit/kill.rs::square_tile_frame`, immediately after
  the cactus-80 arm (`:445-448`), as the non-solid `IsVine` arm:
  `if crate::tileobject::is_vine(cell.kind) { if
  !crate::vine_checks::check_vines(world, cx, cy, cell.kind) { continue; } }`.
  The C# `return` on `false` maps to the loop's `continue` (skip the object
  check + cosmetic frame for this cell). Prerequisite: the entry gate
  (`:411-418`) must admit vine kinds — today it skips `!def && !important &&
  kind != 80`; add `&& !is_vine(kind)` to the skip (the same shape as the
  cactus exception, same C# justification `:81361-81374`). NOTE the
  frame-important deviation: `vine_growth::plant_vine` and
  `growth.rs::grow_vine_arm` SET `frame_important[vine] = true` (C# does
  not) — so placed vines already pass the gate, but GENERATED vines (the
  worldgen Vines pass) may not; the explicit `is_vine` exception covers both
  and the flag write should be dropped at wiring (it also mis-routes vines
  away from `reframe_area`'s cosmetic pass — with the flag cleared,
  `frame_of_variant` frames them and the thorn-style `SquareTileFrame`
  cosmetic tail applies).
* **`check_underwater_plant`** → the SAME loop, as the `case 549` arm
  BEFORE the vine arm (the C# dispatches 549 in `TileFrameImportant`,
  which precedes the non-solid arm):
  `if cell.kind == 549 { vine_checks::check_underwater_plant(world, 549,
  cx, cy, rng); continue; }` (the C# `return`s after the case). The gate
  needs the 549 exception too (549 has no `TileObjectDef`; the seaweed
  growth arm does not set `frame_important[549]` — set it at the
  `grow_check_seaweed` placement, `decorations.rs:341-347`, mirroring
  `Main.cs:7373`, OR add the kind exception; pick ONE — recommendation:
  set the flag at placement + worldgen, matching the C# table, and keep the
  gate reading `important` alone).
* **The RNG seam (the real wiring cost):** `square_tile_frame` has NO rng
  parameter today. `CheckUnderwaterPlant` consumes the ACTIVE `genRand`
  (= `Main.rand` property, `WorldGen.cs:4445`): the growth tick's
  `UpdateWorldGen` named stream when reached via `plant_vine`/
  `grow_check_seaweed`, the edit stream when reached via a player edit.
  The port's convention for framing-side rolls is `world.frame_rng`
  (`reframe_area`'s "genRand stand-in", `framing.rs:1261-1263`) — but that
  is a DIFFERENT stream from the growth phase's `UpdateWorldGen` and wiring
  the tip re-rolls onto it would change both streams' draw order. The
  faithful bind: thread `&mut UnifiedRandom` through
  `square_tile_frame`/`square_tile_frame_default`/`square_tile_frame_gen`
  from each caller (growth callers pass the SAME `rng` they handed
  `update_world_*`; edit callers pass the edit stream; worldgen callers
  pass the pass stream), with `world.frame_rng` only for the variant
  re-rolls it already owns. Until that threading lands, do NOT wire
  `check_underwater_plant` with a fresh/fixed-seed rng — the tip columns
  would consume a foreign stream (determinism rule).
* `framing.rs::frame_world` (the full-frame pass) frames non-important
  tiles without the check dispatch; once vines lose their port-only
  frame-important flag they enter it. The C# `FrameEverything` runs
  `TileFrame` per cell (checks included); the port's full pass is the
  documented cosmetic-only approximation — record the check-skip as a
  standing deviation in ANALYSIS at wiring, or route `frame_world`'s vine
  cells through the same arm (no RNG involved for `CheckVines`, so that
  half is safe to add immediately at the wiring wave).

### 2.3 Contest census

`edit/kill.rs` is touched by NONE of d102/d106/g42/d109/d110/g48 (verified
empty diffs) — the check-dispatch insertion is textually uncontested; the
contest is SEMANTIC: the rng threading changes `square_tile_frame`'s
signature, which every edit/growth/worldgen caller shares. That signature
change is the wiring wave's single coordinated step, not a per-lane hunk.
d110's `reframe_area` wall-arm rework and g48's `section_tile_frame` both
call the cosmetic half only — neither runs the checks today; g48's
`section_tile_frame` (the net `TileFrameSection` apply) SHOULD also run the
check arms per cell at wiring (the C# `Reframe` → `TileFrame` runs them),
noted here so the g48 merger picks it up.

## 3. grow-tops / jungle-thorn → the growth tick arms

### 3.1 The C# tick order (re-read)

`UpdateWorld` (`WorldGen.cs:71495-71625`): rate loops pick
`(genRand.Next(10, maxX-10), genRand.Next(10, surface-1))` →
`UpdateWorld_OvergroundTile` (`:71601`); underground loop
`Next(10, maxX-10) × Next(surface-1, maxY-20)` →
`UpdateWorld_UndergroundTile` (`:71612`/`:71621`); remix runs BOTH per pick
(`:71612-71613`). All draws ride `genRand` == the swapped `Main.rand`
(`Main.SwapRandom("UpdateWorldGen")` — the port's `RngKey::UpdateWorldGen`
named stream, `world_sim/growth.rs:6-12`).

`UpdateWorld_OvergroundTile` (`:72030-72808`): the big per-type else-if
chain ends at `:72498`; then `SpreadGrassWalls`/`SpreadDesertWalls` behind
`AllowedToSpreadInfections` (`:72499-72503`); then a NEW
`if (tile.nactive())` (`:72504`) holding the vine chain: first arm
2/52/382/192 (`:72506-72562`), `else if` lily-61 (`:72563-72574`), then a
SECOND `if`/`else if` chain — jungle 60/62/384/226 (`:72575`), mushroom
70/528 (`:72618`), hallow 109/115 (`:72655`), corrupt 23/661/636 (`:72692`),
crimson 199/662/205 (`:72729`), ash 633/638 (`:72766`). The DST tail
(`:72804-72807`) follows.

`UpdateWorld_UndergroundTile` (`:73214-…`): the else-if chain — alch
(`:73239`), inactive web arm, dye (`:73255`), cattail 519 (`:73263`),
seaweed 549/53 (`:73272-73279`), `SpreadUnderground` grass + 32/352 spikes
(`:73280-73295`), lily-61 (`:73296`), the five vine families
(`:73311-73482`), the 444/396 arms, **jungle-thorn `else if (type == 69)
GrowSpike(i, j, 69, 60)` (`:73658-73661`, NO roll)**, snow/gemcave
(`:73662+`).

### 3.2 The Rust attach points (exact)

* **`grow_overground_vines(world, x, y, rng)`** → ONE call at the END of
  `growth.rs::update_world_overground_tile` (after the `grass_pass` block,
  base `:846`, before the closing brace `:847`). The fn internally encodes
  BOTH chains + the fresh per-arm type re-reads, matching `:72504-72802`.
  Order notes: (a) the port's early `return`s (alch `:732`, saplings
  `:739/:745`, cactus `:781/:788`, moss `:816`) skip the tail — faithful
  for the vine arms because none of those kinds is a vine-chain source
  (the only shared kind would be 192, which takes no early return); (b)
  the port runs the moss arm BEFORE `grass_pass` (the C# order is
  grass `:72355` then moss `:72371`) — a pre-existing deviation, NOT this
  wire's to fix; (c) the lily-61 `else if` between the chains is the
  jungle-plant lane's (documented in `vine_growth.rs:225-227`); when it
  lands it must sit BETWEEN the two chains — the merged fn keeps its
  two-chain structure so the insertion point exists; (d) the wall-spread
  arms (`:72499-72503`) run BEFORE the vine chain in C#; the port's
  overground fn has no wall-spread call today (the underground fn does,
  `:1772-1775`) — when the overground wall-spread lands it goes BEFORE the
  `grow_overground_vines` call.
* **`grow_underground_jungle_thorn(world, i, j, rng)`** → ONE call in
  `growth.rs::update_world_underground_tile` immediately AFTER
  `grow_vine_arm(world, i, j, rng)` (base `:1769`), BEFORE the wall-spread
  block (`:1772`). Faithful despite the C# `else if`: 69 is disjoint from
  every earlier arm's kind set (`SPREAD_UNDERGROUND` =
  {23,661,109,199,662,60,70,633,226}, the vine-arm families, 61, 519, 549,
  53), and the fn self-gates on `kind == 69`. The wall-spread position is
  itself a port deviation (C# runs it in the OVERGROUND fn only,
  `:72499`); keep the existing call where it is — this wire adds no
  reorder.
* **Retire the dead overground 69 arm** (`growth.rs:841-845`): it sits
  inside `if grass_pass`, 69 is not a member, it never fires
  (`vine_growth.rs:13-17` documents this). DELETE it in the same commit
  that wires the underground arm — keeping both would double-fire if the
  grass set ever grows.
* **Stream:** both fns take the caller's `rng` — the growth phase already
  threads `RngKey::UpdateWorldGen` (`world_sim/growth.rs:64-97`); NO new
  stream, no reseed. Draw-order deltas introduced by wiring (the arms
  were previously absent): the plain arm's `Next(1)` always-draw, the
  192 `Next(10)` gate, the per-family odds rolls, `GrowMoreVines`'
  draw-free density walk — all INSIDE the ported fns, already
  witness-pinned; the world-hash pins are generation-only and unaffected
  (the tick never runs during `generate()`).

### 3.3 Contest census

d102/d106/g42 touch `growth.rs` ONLY at the re-export block (`:55-70`) —
zero overlap with the `:846`/`:1769` insertions or the `:841-845` deletion.
`growth/decorations.rs` (the seaweed arm) is untouched by all three. The
phase-2/3 "contested" labels overestimated the conflict: these are add-only
hunks in functions no sibling edits. They stay UNWIRED on this branch per
the task scope (the growth tick is a behaviour change; the wiring wave
lands it with the §2 rng threading so the vine writes' `SquareTileFrame`
checks consume the same stream in the same commit).

## 4. thorn down-merge → the GenTileData/framing seam

`thorn_down_merge_frame` (`vine_growth.rs:393-509`) computes the thorn's
frame with the `:82043-82066` down-merge applied. The wiring is NOT a call
to this fn — it is a 4-line insertion into `framing.rs::frame_of_variant`
(the fn re-ports the mask/RULES tail only because `framing.rs` was
contested at port time):

* **Insertion point:** after the tile's own-slope match (base
  `framing.rs:861-879`), before the neighbour-slot mask build (`:881`).
  Insert:
  `let grass_below: u16 = match me { 32 => 23, 352 => 199, 69 | 655 => 60, _ => 0 };`
  `if grass_below != 0 && down == grass_below as i32 { down = num; }`
  — the direct `down = num` assignments the GenTileData merge parser never
  emits (they are not `TileMergeAttempt` calls), so the generated
  `td::merges` tables cannot carry them and the hand insertion is the
  correct home (do NOT "fix" GenTileData for these).
* **Order pin:** the slope adjustments (`:839-879`) run BEFORE the merge —
  a sloped grass-below cell reads `down = -1` and does not merge
  (`vine_growth.rs:423-471` pins this).
* **Keep-one:** after the insertion, `thorn_down_merge_frame` becomes a
  duplicate of `frame_of_variant` for thorn kinds. KEEP it as the witness
  fixture (its 4 down-merge witnesses run against it unchanged) or rebind
  them to `frame_of_variant` and drop the fn — recommendation: rebind (the
  witnesses assert frames, not internals) and drop, with the anchors row
  repointed if it names the fn (it names `grow_overground_vines` — no
  canary impact).
* **Contest census:** d110 inserts at the fn HEAD (`:818` hunk — the
  gemspark bypass, kinds 255-268/385/446-448, disjoint from the thorns);
  d109 edits `frame_world`/`frame_walls` only; g48 appends a new fn after
  `reframe_area`. The `:880` insertion is textually uncontested by all
  three. It stays UNWIRED here per scope (framing behaviour change).

## 5. UNCONTESTED wires LANDED on this branch (wiring phase)

| Commit | Wire | Evidence |
|---|---|---|
| `11a226a4` | The §1.3 interim fused-pass wiring: `tiles.rs` case-549 crawl + inline skip + `draw_reverse_vines` tail; `vines.rs` `dead_code` gates lifted, doc flips WIRED; `torch.rs` doc keep-one (§6) | 2 source-order witnesses added (`tiles.rs` tests, 14 total in file) |
| `1d2d46f4` | The §7 ROPE-interplay census pins | 2 witnesses added (`vine_checks.rs`, 17 total in file — the phase-2 commit message's "16" counted one fixture twice; actual baseline 15) |

The forward `collect_vine_root` caller (`tiles.rs:633`) was verified LIVE
(the S1-G-1a wiring, `draw_vines` at the foliage tail) — the reverse crawl
joins it at the same seam rather than inventing a second collection site.

## 6. `torch::top_edge_can_be_attached_to` dedupe note

g28/g30/g83 all lifted this fn to `pub` for the reverse crawl
(`TileDrawing.cs:770`). The g31 union-exec (`c3027a38`) landed the keep-one
doc ("W4-g28 pass 1 / g30 pass 3 — identical visibility change, keep-one
landed by the union-exec"). This branch's `torch.rs` is now **byte-identical
to the union's blob** (`git hash-object` = `737d4cf488896ffbc2ca2354b615c0a482a093dc`
on both) — the merge is a zero-conflict same-blob resolution. The body was
never changed by any lane; the only other g83 delta (`pub(crate)` →
`pub` doc-line) is absorbed. No further action at merge.

## 7. Vine × ROPE interplay census (climbing movement)

**Finding: natural vines are NOT climbable in vanilla — the "vine climb" is
the ROPE system, and the two tile sets are disjoint.**

* `TileID.Sets.IsVine` = {52, 382, 62, 115, 205, 528, 636, 638}
  (`TileID.cs:531`); `ReverseVineThreads` = {549} (`:545`).
* `Main.tileRope` = {213, 214, 353, 365, 366, 504, 449, 450, 451}
  (`Main.cs:9313-9321`) — rope 213/214, **VineRope 353** (the crafted
  placeable item, `TileID.cs:1335`), silk 365, web 366, mystic snake 504,
  chains 449-451. Disjoint from both vine sets (witness
  `w4_g83_vines_and_ropes_are_disjoint_sets`).
* `Player.cs`'s ONLY `IsVine` read is the placement-kill gate
  (`PlaceThing_Tiles_BlockPlacementForAssortedThings`, `:41309` — an
  alch-seed placement may kill a vine); already ported
  (`world_sim/editing.rs:7404` via `tileobject::is_vine`, S1-P-i32).
* The climbing movement reads `WorldGen.IsRope` / `Main.tileRope` only:
  `FindPulley` (`Player.cs:19644-19749`, ~105 lines — the `controlUp/Down`
  gate, the `IsRope` probe at the body centre, the ±6 px three-candidate
  snap with `SolidCollision` retries, `pulley/pulleyDir/direction` writes,
  `stepSpeed = 2.5`, `gfxOffY`, the `cameraX` shift); `Update`'s rope block
  (`:26116-26470`, ~355 lines — the `ropeCount` decay, the `FindPulley`
  trigger gate `!pulley && !CCed && !controlJump && gravDir == 1 &&
  ropeCount == 0 && grappling[0] == -1 && !tongued && !mount.Active`, the
  pulleyDir 1↔2 side-switch ladder, the climb velocity ladder
  `-0.2/-0.02` capped `-8`, the `*0.7` decays, the `controlDown` dismount
  `ropeCount = 10; velocity.Y = 1`, the idle snap `y = num35*16 + 22`);
  `CanMoveForwardOnRope` (`:24247-24260`).
* **Port verdict: NOT self-contained small.** ~460 lines entangled with
  `Player.Update`'s movement core (the port's `player.rs::update_with_grapple`
  owns that region; `pulley`/`pulley_dir` exist as POSE fields only,
  `player.rs:1208-1211`, no producer). `ropeCount`, `stepSpeed`,
  `releaseLeft/rightTimer` inputs are unported. The draw half
  (`render/ropes.rs`, S1-G-1f WIRED) already ports `IsRope`/`GetRopeEnds`
  (`:70026-70104`) — the movement port should REUSE `ropes::is_rope`
  (keep-one; lift it out of `render/` into a shared module at that time).
  Attach seam when a movement lane takes it: `player.rs::update_with_grapple`
  at the C# `:26116` position — after the grappling/statics rebase, before
  the mount stat override (`:26623`), i.e. between the port's grapple gate
  (`player.rs:7076-7091`) and the cart block (`:7092+`). This is a catalog
  row for the movement family, NOT a g83 wire.
* Frame-important asymmetry pin (routes §2's two checks):
  `w4_g83_only_the_seaweed_is_frame_important`.

## 8. Verification (for the wiring wave; NOT run in this code-only phase)

```
cargo test -p tr-core vine_checks
cargo test -p tr-core vine_growth
cargo test -p tr-app --lib render::vines
cargo test -p tr-app --lib render::tiles
# after the §1.2 union binds:
cargo test -p tr-app --lib render::tile_drains
cargo test -p tr-app --lib render::special_points
# full gate
cargo test --workspace -- --skip migrates_version_2_kinds_to_item_ids
```

Witness census on this branch after the wiring phase (actual `#[test]`
counts, not the commit-message claims): vines.rs 20 (10 S1-G-1a + 10 g83)
+ vine_checks.rs 17 (15 + 2 census) + vine_growth.rs 16 + tiles.rs 14
(12 + 2 wiring) = **67 written not run** (g83's share: 10 + 17 + 16 + 2 =
45). World-hash pins unchanged (the landed wires are
draw-side only; no tick/framing behaviour changed).

*Produced code-only: no cargo/rustc/tests/graphify invoked; every claim
cites a direct file read or a `git show`/`git diff` of the named tip.*

---

## 9. Wire phase 2 — EXECUTED (the remaining seams, this branch)

The four remaining seams landed on `task/W4-g83-vine-draw-residuals`
(code-only, witnesses written not run):

| Seam | Commit | What landed |
|---|---|---|
| §2 checks + rng threading | `885514e1d` | `check_underwater_plant` as the case-549 arm BEFORE the tree arms + `check_vines` as the non-solid `IsVine` arm after cactus-80 in `edit/kill.rs::square_tile_frame`; the gate admits vine kinds + 549 explicitly; the whole `kill_tile`/`square_tile_frame_*`/cascade family threads `&mut UnifiedRandom` (the caller's ACTIVE genRand — growth `UpdateWorldGen`, liquid `UpdateLiquid`, wiring `self.rand`, edit `UpdatePlayers`, npc `UpdateNpcs`, projectile `UpdateProjectiles`, net the bare global, worldgen a fresh `UnifiedRandom(world.seed)` stand-in); the port-only `frame_important[vine]` writes dropped (`plant_vine` + `grow_vine_arm`); `frame_world`'s cosmetic-only check-skip recorded (fn doc + ANALYSIS row below) |
| §3 grow-tops + jungle-thorn | `2325528ce` | `grow_overground_vines` at `update_world_overground_tile`'s tail + `grow_underground_jungle_thorn` after `grow_vine_arm` + the dead overground-69 arm DELETED; both on the caller's `UpdateWorldGen` stream |
| §4 thorn down-merge | `6b9a1d9fa` | the 4-line `grass_below`/`down = num` insertion in `framing.rs::frame_of_variant` after the own-slope match; `thorn_down_merge_frame` kept as the thin witness fixture delegating to `frame_of_variant` (the standalone re-port + private RULES copy dropped) |
| §1.2 g31 keep-one deltas | (this commit) | `rising_vine_strip_segments` takes `show_invisible` and reads the FULL `IsVisible` (the `is_visible` helper added to `vines.rs`, byte-shape identical to the union's); `draw_reverse_vines` threads it; the fused-pass caller passes its `show_invisible_blocks` |

### 9.1 The g31 keep-one FINAL hunk-pair (g31e d-hunk-prep reads this)

The keep-one binds g31's `tile_drains.rs` onto g83's
`vines.rs::rising_vine_strip_segments` (the strip maths is line-for-line the
same `:9134-9193` port). After seam 4 the two REAL deltas of §1.2 are
resolved as follows — **hunk A** lands on the g83 side (DONE, this branch),
**hunk B** is the g31-side rebind script (apply at the union merge):

**Hunk A — `crates/tr-app/src/render/vines.rs` (LANDED):**

```rust
// the signature gains the sight override (the g31 form):
pub(crate) fn rising_vine_strip_segments(
    world: &World,
    wind: &Wind,
    vine_counter: f32,
    x: i32,
    start_y: i32,
    show_invisible: bool,           // <- NEW (g31's 6th param)
) -> Vec<RisingVineSegment> {
    ...
    // the per-segment push reads the FULL IsVisible, not the bare probe:
    visible: is_visible(world, x, y, show_invisible),   // was: !world.invisible_block(x, y)
    ...
}

// the helper (byte-shape identical to g31's tile_drains::is_visible):
#[inline]
pub(crate) fn is_visible(world: &World, x: i32, y: i32, show_invisible: bool) -> bool {
    let Some(tile) = world.tile(x, y) else { return true; };
    let mut flag = world.invisible_block(x, y);
    match tile.kind {
        19 if tile.frame_y / 18 == 48 => flag = true,
        541 | 631 => flag = true,
        _ => {}
    }
    !flag || show_invisible
}

// draw_reverse_vines gains the same param and threads it:
pub(crate) fn draw_reverse_vines(..., roots: &[(i32, i32)], show_invisible: bool) -> u32 {
    ...
    let segments =
        rising_vine_strip_segments(world, wind, vine_counter, x, start_y, show_invisible);
    ...
}
```

The `RisingVineSegment` struct is UNCHANGED (g31's struct is field-identical;
keep g83's — it carries the `seam_top` doc). The interim fused-pass caller
(`tiles.rs`) passes its `show_invisible_blocks`; the 10 strip witnesses pass
`false` (the pre-wire behaviour) + 1 new witness pins the full-IsVisible
arms (`w4_g83_the_rising_visible_reads_the_full_is_visible`).

**Hunk B — `crates/tr-app/src/render/tile_drains.rs` (the g31-side rebind,
apply at the union merge):**

```rust
// 1. DELETE g31's rising_vine_strip (the whole fn, base :550-622) and its
//    RisingVineSegment struct (:525-547) — keep-one: import g83's instead:
use crate::render::vines::{rising_vine_strip_segments, RisingVineSegment};

// 2. drain_reverse_vines keeps its signature, the body rebinds:
pub(crate) fn drain_reverse_vines(
    world: &World,
    wind: &Wind,
    vine_counter: f32,
    show_invisible: bool,
    points: &[(i32, i32)],
    out: &mut Vec<RisingVineSegment>,
) {
    for &(x, y) in points {
        out.extend(rising_vine_strip_segments(world, wind, vine_counter, x, y, show_invisible));
    }
}

// 3. g31's is_visible (:178-191) stays for the OTHER drains (grass/pylons/
//    trophies); the vines.rs copy is the same byte-shape — the family merge
//    keep-ones one (the d56 tr_render bind, union §1.4). Until then both
//    copies are identical, zero-conflict.
// 4. g31's 2 strip witnesses (w4_g31_rising_vine_strip_walks_up /
//    _breaks_on_non_thread) rebind to rising_vine_strip_segments (same
//    args + the trailing `false`/`show_invisible`); their assertions are
//    unchanged (the maths is the same port).
// 5. drain_draw_data == g83's inline draw_data + half-brick probe (verified
//    line-for-line) — no delta, the segments carry dd/half already.
```

The light seam (`DrainCellLight` → d56) is UNAFFECTED: g83's
`draw_reverse_vines` stays the SPRITE tail (it owns `resolve_tile_frame` +
the bottom-centre origin + the no-glowmask rule), fed by the drain segments;
the `visible` field now carries the full-IsVisible result the d56 bind
expects.

### 9.2 Witness census after wire phase 2

vines.rs 21 (20 + 1 full-IsVisible) + vine_checks.rs 17 + vine_growth.rs 16
+ tiles.rs 14 + kill.rs +2 (the dispatch conversion/kill + the seaweed
caller-stream twin) + growth.rs +2 (the overground tail + the underground
thorn / dead-arm pair) + framing.rs +1 (the down-merge fold + slope order +
the four pairs) = **73 written not run** (g83's share: 10 + 17 + 16 + 2 + 2
+ 2 + 1 + 1 = 51). World-hash pins unchanged: the worldgen callers' new
`UnifiedRandom(world.seed)` stand-ins are drawn ONLY by a seaweed tip
re-roll (none exists in the pinned passes' framing rings — the temple
furniture/altar, the FTW plant kills and the no-traps spikes frame no 549),
and the xorshift pass streams are untouched.
