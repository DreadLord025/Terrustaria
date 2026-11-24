# 09 — TileDrawing / DrawData / paint+colour

> Wave 1 · researched 2026-11-12 · sources: decompiled + crates + ROADMAP/ANALYSIS/PORT_MAP

## Scope

| Side | File | Lines |
| --- | --- | --- |
| C# | `Terraria/GameContent/Drawing/TileDrawing.cs` | 9 656 |
| C# | `Terraria/GameContent/TilePaintSystemV2.cs` | 552 |
| C# | `Terraria/GameContent/Drawing/TileDrawInfo.cs` | (the per-tile record) |
| C# | `Terraria/GameContent/Drawing/WindGrid.cs` | (gust push) |
| Rust | `crates/tr-app/src/render/tiles.rs` | 1 769 |
| Rust | `crates/tr-app/src/render/foliage.rs` | 482 |
| Rust | `crates/tr-core/src/drawdata.rs` | 2 742 |
| Rust | `crates/tr-core/src/wind.rs` | 830 |
| Rust | `crates/tr-render/src/sprite.wgsl` | (paint recolor) |

Responsibility: turn the tile array into sprite batches. The C# has a **two-layer split**
(`Draw(solidLayer: true/false)`), a **deferred special-point registry** (`_specialTilesX/Y`,
`TileCounterType`) for anything that cannot be drawn from one cell (trees, grass, vines,
chests, tile entities), and a **paint render-target cache** (`TilePaintSystemV2`). The Rust
port has **one row-major pass** that draws everything inline; there is no registry and no
render-target cache.

## Key entities

### C# — `Terraria/GameContent/Drawing/TileDrawing.cs`
- `Draw` (:465-742) — per-layer entry point, called twice by the caller.
- `DrawSingleTile` (:864-1411) — the per-cell pipeline (see pass list below).
- `GetTileDrawData` (:4527-6510) — the ~490-`case` static table (≈376 distinct tile ids,
  switch body :4540-6044) producing `tileWidth/tileHeight/tileTop/addFrX/addFrY/flip/glow`.
- `DrawBasicTile` (:1507-1725) — rope, slope, sliced, glow arms.
- `DrawSingleTile_Flames` (:1723/2684-3491) + `GetTileFlameData` (:1847) — flame/glow overlays.
- `DrawSpecialTilesLegacy` (:7893-8024) — 237 SunOrb, 334 in-tile item, 395 item frame,
  520 food platter, 471 weapon rack, 620 critter cage.
- Deferred passes: `DrawEntities_DisplayDolls` (:8026), `DrawEntities_HatRacks` (:8039),
  `DrawTrees` (:8052), `DrawGrass` (:8326), `DrawAnyDirectionalGrass` (:8383),
  `DrawTeleportationPylons` (:8639), `DrawMasterTrophies` (:8597), `DrawVoidLenses` (:8708),
  `DrawMultiTileGrass` (:8738), `DrawMultiTileVines` (:8829), `DrawVines` (:8890),
  `DrawReverseVines` (:8903), `DrawMultiTileGrassInWind` (:8916), `DrawVineStrip` (:9034),
  `DrawRisingVineStrip` (:9134), `DrawMultiTileVinesInWind` (:9238).
- Other: `PostDrawTiles` (:414), `DrawLiquidBehindTiles` (:444), `CacheSpecialDraws_Part1`
  (:4152-4286), `CacheSpecialDraws_Part2` (:4287-4357), `GetFinalLight` (:4358-4402),
  `ShouldTileShine` (:4404-4433), `IsTileDrawLayerSolid` (:4469), `IsVisible` (:1411),
  `GetTileDrawTexture` (:1434-1506), `IsWindBlocked` (:6510), `GetWindGridPush` (:8554),
  `GetWindGridPush2Axis` (:8564), `GetWindGridPushComplex` (:8579),
  `GetAverageWindGridPush` (:9195), `GetHighestWindGridPushComplex` (:9218),
  `Update` (:367-388), `GetWindCycle` (:7676), `ShouldSwayInWind` (:7702),
  `EmitTreeLeaves` (:7779), `ClimbCatTail` (:8807).

### Rust
- `crates/tr-app/src/render/tiles.rs::draw_tiles` (:256-1077) — the one row-major pass:
  waterfall lighting feed → foreground liquid → wall layer → per-tile → tiles' flames →
  item drops/entities → foreground water.
- `crates/tr-app/src/render/tiles.rs::resolve_tile_frame` (:147-192) — source-rect lookup
  (`frame_x_override`/`frame_y_override`/`add_fr_*`/`width`/`height`, tree 20×20 grid).
- `crates/tr-app/src/render/tiles.rs::push_lit` (:1214), `push_slope` (:1271), `push_half`
  (:1315), `push_sliced` (:1340), `draw_tile_flames` (:1409), `flat_src_uv` (:1189),
  `merge_dark_row` (:89), `sunflower_sway` (:216).
- `crates/tr-core/src/drawdata.rs::draw_data` (:109-1040) — the `GetTileDrawData` port.
- `crates/tr-core/src/drawdata.rs::TileDrawData::apply_world_corrections` (:1200-1357) —
  world-probe rewrites + the `DrawSingleTile` positional switch.
- `crates/tr-core/src/wind.rs::Wind::cycle` (:284), `in_a_place_with_wind` (:335),
  `advance_counter` (:358), `advance_tree_counter` (:365), `visual_wind_amount` (:377),
  `advance_sunflower_counter` (:768), `advance_vine_counter` (:775).
- `crates/tr-app/src/render/foliage.rs::draw_tree_foliage` — canopy/branches + leaf gores.

## Dependencies
- **Calls into (Rust):** `tr_core::drawdata`, `tr_core::wind`, `tr_core::tree`,
  `tr_core::lighting` (`LightMap`/`LightStyle`), `tr_core::tiledata` (`TILE_SOLID`,
  `SET_PLATFORMS` — the exists set, but no case consumes `SET_PLATFORMS`), `assets`
  (`Lookups::rect_uv`/`frame_tex`/`plain_tex`), `tr_render::SpriteRenderer::push_sprite*`.
- **Called by:** `crates/tr-app/src/app/frame.rs` (one `draw_tiles` per frame, both layers
  fused), `crates/tr-app/src/render/mod.rs`.
- **Shared data:** `TileDrawData` (Rust) ↔ `TileDrawInfo` (C#); `Wind` ↔ `Main.windCounter`/
  `WindForVisuals`/`WindGrid`; `World.extras` (paint/coating, sparse) ↔ `Tile.color()`/
  `Tile.wallColor()`.

## Port status

### A. Pass structure

| C# pass (order) | Status | Rust location | Evidence / notes |
| --- | --- | --- | --- |
| 1. `Draw` quality-light thresholds (:479-495) | PARTIAL | `tiles.rs:362-366` | The three tiers exist as `HIGH_LIGHT`/`MEDIUM_LIGHT`/flat; the `_highQualityLightingRequirement` byte formula (255*(1-q)+30q) is not reproduced — a different constant set. |
| 2. `Draw` screen-area loop (+4/-2 margin) (:504-506) | PARTIAL | `tiles.rs::draw_tiles` + `render/tile_pass.rs` (W4-g29, UNWIRED) | One fused loop instead of two layer passes; margin differs (G6 cached range). The MISSING orchestration (prologue/epilogue order, `GetScreenDrawArea`, black strips, `PostDrawTiles` order, the frame-level layer order) landed as structure in `tile_pass.rs` — ANALYSIS "The Draw orchestration (W4-g29 pass 2)". |
| 3. `DrawTile_LiquidBehindTile` on solid layer (:524) | PORTED | `render/liquids.rs::draw_tile_liquid_behind` | |
| 4. Special-point classification switch (:534-723) — vines / MultiTileVine / MultiTileGrass / VoidLens / Pylon / MasterTrophy / AnyDirectionalGrass / WindyGrass | PARTIAL | none (registry) | Inlined: grass+sunflower sway (`tiles.rs:671-751`, `838-851`), trees (`foliage.rs`). The **deferred ordering** is lost — `tiles.rs:671-678` admits the "z-order approximation". |
| 5. `DrawSingleTile` (:864-1411) | PARTIAL | `tiles.rs::draw_tiles` body | See coverage table; `GetTileDrawData`, the positional switch, flames, half/slice/slope are ported. |
| 6. `DrawTileCracks` (solid only, :732-733) | PORTED | `render/tile_cracks.rs::draw_tile_cracks` | S1-G-1 wave C: `Main.cs:61533-61626` verbatim (the damage ladder, the tree/mushroom shape arms, the 18 px rects, the `Lighting.GetColor` tint); the port's mining accumulator projects onto the `HitTile` damage units. The wall sites (`WallDrawing.cs:203-204`) draw when the wall book fills — the projection already emits `CRACK_WALL` records. Deviations: ANALYSIS "The tile-crack overlay (S1-G-1 wave C)". |
| 7. `DrawSpecialTilesLegacy` (:736, :7893-8024) | MISSING | — | No SunOrb / in-tile item / item frame / food platter / weapon rack / critter cage. |
| 8. `DrawEntities_DisplayDolls`/`HatRacks` (:8026/:8039) | MISSING | — | No tile-entity sprite pass. |
| 9. `DrawTrees` (:8052) | PORTED | `foliage.rs::draw_tree_foliage` | Canopy + branch + leaf gores; sway per `DrawGrass`. |
| 10. `DrawGrass`/`DrawAnyDirectionalGrass` (:8326/:8383) | PARTIAL | `tiles.rs:786-799`, `838-851` | WindyGrass inlined; `AnyDirectionalGrass` (tile 184) has no separate pass. |
| 11. `DrawMultiTileGrass(…InWind)` (:8738/:8916) | PARTIAL | `tiles.rs:721-751` | Sunflower ported per-cell; the general 2×n multi-tile grass pass is not. |
| 12. `DrawMultiTileVines(…InWind)`/`DrawVines`/`DrawReverseVines`/`DrawVineStrip`/`DrawRisingVineStrip` (:8829-9238) | PARTIAL | `render/vines.rs` (S1-G-1 wave A, 2026-11-18; W4-g83 2026-11-21) | `DrawVines`/`DrawVineStrip` + the root crawl ported term-for-term (the sway ladder, the grid push, the anchor walk, the 638 glowmask); the strips draw after the foliage pass. **W4-g83 (rising pass WIRED into the fused pass, wire phase)**: `DrawReverseVines`/`DrawRisingVineStrip` (549) landed (`draw_reverse_vines`, the +16+2 anchor / num7>10 walk / no-remix gate / 40,-0.004 push / negated ladder / (rot-PI/2) walk / bottom-centre origin / no glowmask, 10 witnesses) and wired at the C# positions — the `tiles.rs` case-549 crawl arm (`:546-548`), the inline skip, the `draw_vines` tail (`:429-430`); the g31-union rebind + the checks/grow-tops attach seams are `research/123-vine-wire-contract.md`. The lifecycle checks `tr-core::vine_checks` (CheckVines + the seaweed tip-frame variants) + the grow-tops/jungle-thorn residuals `tr-core::vine_growth` are **WIRED (wire phase 2, contract §9)**: the checks ride `edit/kill.rs::square_tile_frame`'s dispatch on the caller's ACTIVE genRand (threaded, never `frame_rng`), the grow arms ride the `growth.rs` tick tails (the dead overground-69 arm deleted), the thorn down-merge is the 4-line `frame_of_variant` insertion, and the g31 keep-one `show_invisible`/full-IsVisible delta landed (the final hunk-pair: contract §9.1). `DrawMultiTileVines(…InWind)` is the furniture lane (d57), still out. **SEALED (g83d vine-retire, contract §10)**: the latest-tip re-verify + the g42 dispatcher-merge script + the hunk-B re-pin + the vine-family merger bible + the 74-witness acceptance checklist; the `grow_more_vines` duplicate retired onto `growth.rs`. Deviations: ANALYSIS "The vine strip pass" + "The vine draw residuals (W4-g83)" (LitNature shader, biomeSight dust). |
| 12. `DrawMultiTileVines(…InWind)`/`DrawVines`/`DrawReverseVines`/`DrawVineStrip`/`DrawRisingVineStrip` (:8829-9238) | MISSING | — | No vine strip passing. |
| 13. `DrawTeleportationPylons` (:8639) | PORTED | `render/pylons.rs::pylon_sprites` | S1-G-1 wave D (`task/S1-G-1d`): the crystal + 6-copy ring, byte colour math, IsVisible gate, selection arm (fed 0). Deviations: fx-range z-order, dust skipped, counter stand-in — ANALYSIS "The pylon/trophy/void-lens passes". |
| 14. `DrawMasterTrophies` (:8597) | PORTED | `render/pylons.rs::trophy_sprites` | S1-G-1 wave D: the 1×28 row, the frameY/72 flip, the light-tinted ring, Glow-coating white. |
| 15. `DrawVoidLenses` (:8708) | PORTED | `render/pylons.rs::void_lens_sprites` + `void_lens_light` | S1-G-1 wave D: `VoidLensHelper.cs:11-105` — the -2px anchor, float-mod frame, 3-copy purple ring, the AddLight(0.4,0.2,0.9) into EntityLights; dust skipped. |
| 16. `PostDrawTiles` (:414) | MISSING | — | |
| 17. `Update` counters (:367-388) | PORTED | `wind.rs::advance_counter`/`…_tree`/`…_sunflower`/`…_vine` | |
| 18. Waterfall overlay (`Main.DoDraw_Waterfalls` :63281 → `WaterfallManager.Draw`, the falls between the non-solid and solid tile passes) | PORTED (W4-g73/g74/g75 FINAL UNION + final sweep + closeout, 2026-11-21) | `tr-core/src/waterfall/{mod,draw}.rs` + `render/tiles.rs` (the quad drain) + `render/liquids.rs::draw_shimmer_fall` | The full-file `WaterfallManager.cs` census: all 4 `FindWaterfalls` spawn arms (half-brick spout with the lava/honey/shimmer precedence :182-197, cloud fountains 196→11 :204-219 / 460→22 :220-235 / 717→26 :236-251), every draw arm (typed 1/14/25 slow-frame, rain 11/22/26 two-sheet pairs + frame tables, disco 160→2 + gemspark 262-268→15-21 conversions, the slope/bend walk, the :749 white foam), `UpdateFrame`'s 6 counters, `StylizeColor`/`GetAlpha`/`TrySparkling`/`AddLight` all cases, both ambience scans + the live two-tier `liquid_loop_update`→`liquid_ambience` consumer. Final-sweep fix (folded): the disco fall reads the SHARED `Main.DiscoR/G/B` via `set_disco_rgb(params.disco)` (the manager-local cycle copy removed — the C# manager has no disco state). Deviations: the row-major fusion z-order (deviation 1), the one-frame-late AddLight (matches vanilla). `docs/UNION-CONTRACT-waterfalls.md`, ANALYSIS "Waterfall overlay" + "W4-g73 FALLS CLOSEOUT". |

### B. DrawData struct & ordering

| Aspect | C# | Rust | Status |
| --- | --- | --- | --- |
| Record | `TileDrawInfo` class, out-params filled by `GetTileDrawData` :4529-4535 | `TileDrawData` struct (`drawdata.rs:24-68`) | PORTED (shape parity) |
| Default | 16×16, `tileTop=0`, `addFrX/Y=0`, effects none; `addFrY = Main.tileFrame[type]*38` :4533 | `DrawData::default()` :70-104, `add_fr_y = tile_frame*38` :122 | PORTED |
| Static table | ~490 `case` labels, ≈376 tile ids (:4540-6044) | ~119 match arms, ≈113 tile ids (`draw_data.rs:129-1021`) | PARTIAL — ~30 % of ids; the rest fall to the 16×16 default by design (`drawdata.rs:1005-1007`, `:2072`). |
| Animated rows | `Main.tileFrame[type]` advanced by the animator (`Main.AnimateTiles`, `Main.cs:18516-19304`) | `TileAnimator` (`drawdata_anim.rs`, S1-F-9b); production `tile_frame` still 0 until wired | PARTIAL — animator ported 2026-11-18 (16 tests), renderer wiring deferred. |
| World probes in-table | palm biome 323 (:4670), dye plants 227 (:5063-5078), `GetTreeBiome`/`GetTreeVariant` (:1735/:1778) | `apply_world_corrections` :1200-1243 + `palm_tree_biome`/`palm_frame_y`/`cactus_type`/`dye_frame_x_offset`; **S1-F-9 wave A**: `tree_variant`/`tree_biome`/`tree_frame_x_offset` + case 5's static arm and correction (2026-11-18) | PARTIAL — tree probes PORTED (incl. the jungle underground variant 5). |
| `DrawSingleTile` positional switch (:1020-1159) | `vector` nudges, `rectangle.Y` mutation, `finalColor` revisions, `skip` arms | `draw_dx/dy`, `src_y_shift`, `light_scale`, `src_alpha`, `skip_draw`, `no_width_adjust` + renderer arms `tiles.rs:800-867` | PORTED except parking list below. |
| Glow textures (`glowTexture`/`glowSourceRect`/`glowColor` out params) | per-case in `GetTileDrawData` :6049+ | `draw_tile_flames` :1409 | PARTIAL — the `Main.tileGlowMask` table arm is the only consumer; the per-case glow rects are gone. |

**Parked inside the otherwise-ported positional switch** (ROADMAP TILE-DRAW-6, `done`):
disco arms 160/692 (`Main.DiscoR/G/B`), tile 83's texture override, 323's frond-column
`return` (no palm canopy pass — deviation, `drawdata.rs` deviation comment).

### C. Special-tile coverage — THE CENTREPIECE

| Family (C# path) | C# behaviour | Rust | Verdict |
| --- | --- | --- | --- |
| **Platforms** — `GetTileDrawData` case 19 (:6157-6170, martian conduit glow rows 26/27), `DrawBasicTile` :1511-1545 (slope 1/2 → behind-tile quads at X 198/324/162/306), `DrawSingleTile` :1361-1400, `DrawTile_BackRope` :3766 dispatch :1513 | 16×16 cell + glow on conduit rows + two "side fill" quads when a sloped platform abuts a solid block | `drawdata.rs` has **no case 19** (documented :1019-1020); `SET_PLATFORMS` (`tiledata/sets.rs:164`) is never read by the render path; `tiles.rs:1156` only uses it inside `solid_full` | **PARTIAL** — base sprite correct, glow + slope side-fills + the `BackRope` dispatch MISSING |
| **Torches** — case 4 (:4540-4546), `Main.tileFlame` → `DrawSingleTile_Flames` :1723-3491, `GetTileFlameData` :1847 | 20×20 sprite, `tileTop 4`, plus a flame overlay whose colour/source come from the flame table | `drawdata.rs` case 4 (`torch_uses_solid_above`, test :1410); `tiles.rs::draw_tile_flames` :1409 | **PARTIAL** — metrics ported; the flame table is a subset of C#'s 760-line arm |
| **Chests** — `CacheSpecialDraws_Part2` :4287-4330 (`TileID.Sets.BasicChest`) | rewrites `addFrX/addFrY` per sub-cell, `tileHeight = 18` on the bottom row, **+38/+76 row by `Main.chest[i].frame`** (opening animation), Demon/Chest glow rect for 21 columns 48/49 | no chest arm anywhere in `render/tiles.rs` (no `chest` symbol in the pass) | **MISSING** |
| **Plants / herbs (SwaysInWindBasic, 83, 227, 529/530, 485/489/490...)** — `DrawGrass` :8326, `ShouldSwayInWind` :7702 | sway transform `pos + (cycle, |cycle|)`, rotation `cycle*0.1` about `(tileWidth/2, 16-tileTop)` | `tiles.rs:786-799` (default-size), `:800-824` (non-16×16), `wind.rs::cycle` | **PORTED inline** (z-order deviation) |
| **Sunflower 113 / multi-tile grass** — `DrawMultiTileGrassInWind` :8916-9041 | 2×4 object swayed as one about the bottom-centre anchor, `_sunflowerWindCounter` cadence `1/420 + num*5/420` (:374, :7719 area) | `tiles.rs::sunflower_sway` :216-253 + tests :1552/:1605/:1624; `wind.rs::advance_sunflower_counter` :768 | **PORTED (per-cell approximation)** — general `DrawMultiTileGrass` not |
| **Vines / reverse vines / vine strips** — :8829-9238, `CrawlToTopOfVineAndAddSpecialPoint` :744 | strip passes with `ClimbCatTail`, average/highest grid push, `GetAverageWindGridPush` :9195 | `render/vines.rs` (S1-G-1 wave A): `DrawVines`/`DrawVineStrip` + the crawl + `BottomEdgeCanBeAttachedTo` | **PARTIAL** — the hanging-vine strip pass ported (10 `s1_g_1a_*` tests); reverse vines (549) + multi-tile vines still missing |
| **Ropes (BackRope 149?/`DrawTile_BackRope` :3766)** — dispatched for `typeCache == 380 \|\| Platforms[type]` :1511 + `DrawTile_MinecartTrack` :3792 | rope sprite drawn behind the platform/planter/track cell: rect `(90, (tileY+tileX)%3*18)` of the TOP rope tile's sheet (`Main.tile[tileX, topRopeY]` :3776-3781), cell's raw `tileLight`, `WorldGen.IsRope`/`GetRopeEnds` :70026-70104 | `render/ropes.rs` (S1-G-1 wave B): `back_rope_cell`/`draw_back_rope` + `is_rope`/`rope_ends` verbatim incl. the no-top-end down-scan quirk; the rope tiles' own vertical segments already ride `DrawSingleTile` | **PORTED, UNWIRED** — the tiles pass call sites land with the wave-A merge (no pixels change until then) |
| **Cacti 80/227** — `rectangle.Y += 54/108/162` :1132-1145 | biome row on the plain flat rect only | `drawdata.rs::cactus_source_y_offset` :1184 + `apply_world_corrections` :1231-1234, consumed in `flat_src_uv` :1189 (BUG-19b) | **PORTED** |
| **Trees (5, 583-589, 596, 616, 634, 72 shroom cap, 323 palm)** — `DrawTrees` :8052, `GetTreeBiome` :1735, `GetTreeVariant` :1778, `GetPalmTreeVariant` :3491, shroom-cap sprite :994-1006 | 20×20 trunks on a 22 px grid, canopy/branch sheets by biome+variant+`tileColor`, shroom cap overlay | `foliage.rs::draw_tree_foliage` + `resolve_tile_frame` is_tree arm; the trunk biome probe is `drawdata::tree_biome`/`tree_variant` via `tree::trunk_sheet_offset_at` (S1-F-9 wave A, 2026-11-18 — was a 100-row any-non-tree approximation); shroom cap MISSING | **PARTIAL** — trunk biome/variant selection PORTED; the canopy style's per-region `TreeTops.GetTreeStyle` randomisation stays a documented deviation (`tree.rs::Foliage::for_ground`) |
| **Animated tiles generally** — `Main.tileFrame[type]` + animator (`Update` :367) | every `case` row keyed on `tileFrame` | `tile_frame ≡ 0` (`tiles.rs:543-545`, `drawdata.rs:5-8`) | **MISSING** — biggest single-row gap |
| **Tile-entity-bearing furniture** — 395/698/520/471/470/475/378/334/620/237 | special-point registry + entity item sprites | none | **MISSING** |
| **Particles/dust from tiles** — `DrawTiles_EmitParticles` :6688-7548, `EmitLivingTreeLeaf` :7549, `EmitAlchemyHerbParticles` :9605 | per-tile sparkle/dust rolls on `Main.TileFrameSeed` | none (leaves only: `tiles.rs:868-914`) | **MISSING** (leaves partial) |
| **Dangerous-tile outline / `GetTileOutlineInfo` :4478** | highlight overlay | none | **MISSING** (debug feature) |
| **`Draw_LiquidBehindTile` / submerged vertex tints :3824-4152, :4145** | water mask behind sloped cells + shimmer tint | `render/liquids.rs::draw_tile_liquid_behind` + `liquid_behind_spout_refused` | **PARTIAL** — the shimmer vertex tint is called out in ROADMAP :188-189 as open. **W4-g75 final-sweep census + retire-contract wiring (2026-11-21, folded into g73):** the `:3858` refusal chain's `CheckForWaterfall` arm — `tileCache.halfBrick() && (num3 > 160 || num > 160) && waterfallManager.CheckForWaterfall(tileX, tileY)` (a spouting half-brick suppresses its own behind-water so the fall reads clean) — is now WIRED as `liquid_behind_spout_refused` (the `:3835-3844` type-379 bubble exclusion on `num3`/`num` verbatim; the manager handle rides `draw_tiles`' existing `waterfall` seam). The `inActive()` half of the same refusal is owned by the g80 actuator lane (`liquid_behind_refused`, af6cd44f — that commit leaves the spout clause as a recorded deviation; the halves compose by OR at the family merge). 3 witnesses written not run. |
| **Master trophy / pylon / void lens / critter cage** | dedicated passes | `render/pylons.rs` (S1-G-1 wave D) | **PORTED** (trophy/pylon/lens; critter cage stays MISSING — `DrawSpecialTilesLegacy`) |

## Tests

- `tr-core/src/drawdata.rs`: `default_block_is_16x16` :1385, `torch_uses_solid_above` :1410,
  `plants_flip_on_even_columns` :1437, `tall_plants_are_raised_and_flipped` :1455,
  `herbs_are_raised_and_flipped` :1473, `candles_are_20px_high_and_raised` :1491,
  `vines_family_is_raised_and_flipped` :1503, `thorns_beds_and_palms_use_their_extra_size`
  :1516, `cactus_family_and_plantera_bulb_sit_low` :1535, `dye_plants_are_32x38_and_flip`
  :1552, `test_DW_1_4_conditional_heights` :1566, `test_DW_1_4_stride_2016` :1596,
  `test_DW_1_4_banner_parity` :1651, `test_DW_1_4_frame_overrides` :1685,
  `test_DW_1_2_campfire_epilogue` :1739, `test_DW_1_4_tile_frame_passthrough` :1759,
  `test_DW_1_1_group_lists_complete` :1783, `test_DW_1_1_starred_static_skeletons` :1842,
  `test_DW_1_1_static_switch_sweep` :1885, `test_DW_1_1_deviation_kinds_stay_default`
  :2067, `i16_domain_extremes_do_not_panic` :2078, `test_DW_2_1_palm_biome` :2117,
  `test_DW_2_1_palm_oasis_offset` :2148, `the_positional_switch_nudges_and_hides_the_sprites`
  :2185, `bug19_the_cactus_shift_lands_on_the_flat_draw_rect` :2313,
  `test_DW_2_2_cactus_type` :2355, `test_DW_2_2_cactus_frame_columns` :2387,
  `test_DW_2_2_cactus_walk_limit` :2424, `test_DW_2_3_wiring_arithmetic` :2443,
  `test_DW_2_3_apply_world_corrections` :2473, `test_DW_2_4_world_boundaries` :2530,
  `test_DW_2_4_cactus_no_ground` :2575, `corrections_i16_extremes_do_not_panic` :2589,
  `test_DW_4_1_s3b_static_parts` :2617, `test_DW_4_2_solid_above_scope` :2651,
  `test_DW_4_3_direct_asserts` :2680.
- `tr-core/src/wind.rs`: `advance_counter_matches_grass_formula` :484,
  `advance_tree_counter_matches_tree_formula` :512, `test_DW_4_1_sunflower_sway_counter`
  :522, `cycle_stays_within_source_bounds` :553, `cycle_varies_with_x_y_and_counter` :579,
  `cycle_respects_surface_and_sway_setting` :591, `too_windy_for_butterflies_…` :694,
  `sways_in_wind_basic_matches_the_csharp_list` :786, `leaf_frequency_table_matches_the_source`
  :800, `probe_roll_distribution` :814.
- `tr-app/src/render/tiles.rs`: `test_DW_4_1_sunflower_sway_transform_per_row` :1552,
  `test_DW_4_1_sunflower_sway_tracks_the_counter` :1605,
  `sunflower_sway_gates_freeze_the_flower` :1624,
  `test_BUG19b_the_cactus_flat_draw_carries_the_biome_row_shift` :1671,
  `test_DW_4_3_unframed_tile_draws_nothing` :1719.
- **No test covers**: platform glow/slope side-fills, chests, back-rope,
  the animator, `DrawSpecialTilesLegacy`, the paint recolor. The vine strip
  pass now has the `s1_g_1a_*` suite (`render/vines.rs`, S1-G-1 wave A).
  There is no renderer-level
  golden-image test for `draw_tiles` (only the `TR_SHOT` SHA256 double-run practice).

## Deviations & risks

1. **Row-major fusion of the two layer passes** — `behavioural`. C# runs `Draw(solid:true)`
   then `Draw(solid:false)`, each with its own `SetLayer` stream; the port interleaves.
   `tiles.rs:427-435` documents the waterfall-order approximation. ROADMAP acknowledges
   nothing for the tile layer split. Severity: **behavioural / z-order**.
2. **Deferred special points → inline drawing** — `behavioural / z-order`. Vine strips,
   multi-tile vines, grass and trees lose C#'s "collect first, draw after" order.
   Acknowledged at `tiles.rs:671-678`, `:727-730`.
3. **`GetTileDrawData` covers ≈113 of ≈376 case ids** — `pixel`. Every uncovered kind with a
   non-16×16 frame or an animation-row stride silently draws the default rect (e.g. tiles
   whose `addFrY = tileFrame*38`), so a lamp/candle/banner family can be panned or squashed.
   Documented per-arm in `drawdata.rs`, but the *set* of uncovered ids is not enumerated in
   docs.
4. **Animator absent (`tile_frame ≡ 0`)** — `behavioural`. Affects every animated tile
   (furniture, torches, campfires, jellyfish blocks, starry glass). ROADMAP admits it
   ("animator not ported"). **Update 2026-11-18 (S1-F-9b):** `Main.AnimateTiles`
   (`Main.cs:18516-19304`) + `AnimateTiles_WeatherVane` (`:19306-19329`) ported into
   `crates/tr-core/src/drawdata_anim.rs` (`TileAnimator`, 16 per-case tests); the
   production feed into `draw_data` is still 0 — wiring is the remaining half.
5. **Chest frame rewrite missing** — `behavioural`. `Main.chest[i].frame` (+0/+38/+76,
   `tileHeight=18`) never reaches the source rect, so opening chests do not animate and the
   bottom row is 16 px tall instead of 18. **Not admitted in ROADMAP.**
   **S1-F-9 wave C update (2026-11-18)**: the rewrite itself is ported
   (`drawdata_furniture.rs::chest_draw_rewrite`, per-case tests); what remains is the
   wiring — an open-frame field on the chest state and the `tiles.rs` call after the
   campfire epilogue (deferred to the S1-F-9 merge; waves A/B own `drawdata.rs`/`tiles.rs`).
   The cage-frame draw rewrites (deviation 4's furniture slice) are likewise ported in
   `drawdata_furniture.rs::cage_draw_data` with the `Main.*CageFrame` tables as a lookup;
   only the cage animator (`Main.AnimateTiles_CritterCages`) is still missing.
6. **Platforms: martian-conduit glow rows (frameY/18 == 26/27) and slope side-fill quads
   missing** — `pixel`. `SET_PLATFORMS` is generated but unused by the renderer. **Not
   admitted.**
7. **Wind math is faithful** — `Wind::cycle` (`wind.rs:284-311`) reproduces `GetWindCycle`
   (:7676-7700) term for term (cos, `y/100` integer division, surface/remix gate,
   `+WindForVisuals`, `×GetLerpValue(0.08,0.18)`). One risk: Rust guards with
   `world_surface.is_finite()` and otherwise treats the surface as "above" — the C# always
   has a surface, so a degenerate world changes the gate. Severity: **cosmetic** (test
   worlds only).
8. **`WindGrid` gust push entirely missing** — `behavioural`. `GetWindGridPush` (:8554),
   `GetWindGridPush2Axis` (:8564), `GetWindGridPushComplex` (:8579),
   `GetAverageWindGridPush` (:9195), `GetHighestWindGridPushComplex` (:9218) and
   `EnsureWindGridSize` (:7773) have no counterpart. RoADMAP M1.4-WEATHER-1 lists it as
   "Out: the WindGrid push wave (the tile-draw seam)".
9. **Paint is shader-side, C#-side is a baked render-target cache** — `pixel`.
   `TilePaintSystemV2.RequestTile`/`PrepareAllRequests`/`TryGetTileAndRequestIfNotReady`
   (:369-541) produce a recolored *copy* per `(type, style, paint)`; the port instead
   recolors per pixel in `sprite.wgsl::paint_recolor` :158-169 from the `PaintID` sRGB
   table :222-226. The C#'s bake is compiled HLSL and absent from the decompile, so the
   match is **unverifiable**; `sprite.wgsl:151-158` admits "this port shades in the sRGB
   domain". TILE-DRAW-4 is marked DONE on an unprovable equivalence. Severity:
   **pixel / determinism of screenshots**, and it defeats the C#'s one-draw-per-tile
   caching (per-tile cost now scales with painted pixels).
10. **Tile colour lookup** — C# `GetFinalLight` (:4358-4402) = `(tileLight*tint)/255`
   truncating multiply, then `fullbrightBlock` → White, else `inActive` → `actColor`
   (RGB × 0.4), else `ShouldTileShine` (:4404-4433) → `Main.shine`. The port folds this
   into `push_lit` :1214 with `TileDrawData.light_scale`/`src_alpha`; the **`ShouldTileShine`
   shimmer branch and `Main.shine` are not ported** (`tiles.rs:526-528` covers only
   `actColor`). Severity: **pixel** on shimmer-affected tiles.
11. **Quality-light tier constants differ** — `behavioural / pixel`. C# `_highQualityLighting
    Requirement = (255*(1-q)+30q, ×1.1, ×1.2)` and `_medium… = (50*(1-q)+2q, …)` (:479-486)
    vs the port's `HIGH_LIGHT`/`MEDIUM_LIGHT` (`tiles.rs:362-366`). **Not admitted.**
12. **`DrawTileCracks` missing** — `cosmetic` (the mining/hit overlay).
    **RESOLVED 2026-11-18 (S1-G-1 wave C)**: `render/tile_cracks.rs` ports the
    pass; the single-accumulator projection + the world-seeded crack-style
    roll are the recorded deviations (ANALYSIS "The tile-crack overlay").

## Task seeds

- ~~**S** — Wire `WaterfallManager.CheckForWaterfall` into `draw_tile_liquid_behind`'s refusal
  (`TileDrawing.cs:3858`)~~ **DONE (W4-g75 retire-contract, 2026-11-21, folded into g73):**
  `liquid_behind_spout_refused` (`render/liquids.rs`), the manager handle fed from the
  `draw_tiles` seam; the g80 `inActive()` half (af6cd44f) composes by OR at the family
  merge — complementary, not contested. *[was the W4-g75 final-sweep residual]*
- **S** — Port the platform arms: `SET_PLATFORMS` cases in `drawdata.rs` (martian glow rows
  26/27 from `TileDrawing.cs:6157-6170`) + the slope side-fill quads (`:1517-1543`) in
  `tiles.rs`. Depends on nothing; high pixel payoff. *[covers deviation 6]*
- **M** — Port `CacheSpecialDraws_Part2`'s chest rewrite (`:4287-4330`) into
  `apply_world_corrections` (needs `Main.chest[i].frame` — a chest-open state field on
  `App`), plus a test pinning addFrY 0/38/76 and `tileHeight=18`. *[deviation 5]*
- **M** — `WindGrid` push wave (`:8554-8596`, `:9195-9237`) + `EnsureWindGridSize`
  (:7773) as a `tr_core::wind` extension consumed by the grass/vine sway. Depends on the
  M1.4-WEATHER-1 row; unblocks TILE-DRAW-3's remaining half. *[deviation 8]*
- **L** — Tile animator: `Main.tileFrame[type]` advance (`TileDrawing.Update` :367-388 +
  the per-type `Main.tileFrameCounter` rules) so `draw_data`'s many `tile_frame` strides
  become live. Prerequisite for a truthful TILE-DRAW-1/2. *[deviation 4]*
- **M** — Vine/rope family: `DrawVines`/`DrawReverseVines`/`DrawVineStrip`/
  `DrawRisingVineStrip`/`DrawMultiTileVinesInWind` (:8829-9238) + `DrawTile_BackRope`
  (:3766) as a deferred special pass beside `foliage.rs`. Needs the special-point registry
  of seed L2 below. *[deviation 2]*
- **L** — Reintroduce the deferred `_specialTilesX/Y` registry (`TileCounterType`, :183 /
  AddSpecialPoint) so trees, grass, vines, pylons, trophies and void lenses draw in C#'s
  order; measure the `TR_SHOT` SHA256 diff before/after. Depends on nothing but touches
  the whole pass. *[deviations 1, 2]*
- **S** — `ShouldTileShine`/`Main.shine` (:4404-4433) into `push_lit`. *[deviation 10]*
- **M** — Verify or replace the paint recolor: build a pixel A/B of painted tiles against a
  reference frame, or document a measured deviation in ANALYSIS. Until then TILE-DRAW-4
  should read PARTIAL. *[deviation 9]*
- **S** — Quality-tier constants reconciliation (`:479-486` vs `tiles.rs:362-366`).
  *[deviation 11]*
- **S** — Sweep `draw_data`'s uncovered ids: emit the C#-side id set from
  `TileDrawing.cs:4540-6044` and diff against the Rust arms, then triage which uncovered
  ids have a non-default static part (the `test_DW_1_1_static_switch_sweep` shape).
  *[deviation 3]*
