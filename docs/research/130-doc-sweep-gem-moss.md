# 130 — DOC-SWEEP lane 40: the gem/moss residuals (W4-c29 sweep phase)

Code-only doc-sweep of the five gem/moss residual items the lane brief
named, against `decompiled/Terraria/` (read in ≤100-line windows) and the
pushed co-lane tips (`git fetch --prune` at sweep time). One REAL gap
found and ported (the per-cell break-drop slice, commit `55555fb9b`);
the other four items are claimed-and-faithful, refuted premises, or
deferred-with-owner. Witnesses WRITTEN NOT RUN (no compiler on this
branch — the lane contract).

## Co-lane claim check (read FIRST)

| Co-lane | Tip read | Claim relevant to this lane |
| --- | --- | --- |
| `W4-g18-addlight-main-1` | `22d529c5` | Main.cs AddLight batch 1: exactly FIVE sites, all hook-chain (`:29016-29028`/`:29186`), all e8's. **Negative census**: "Main.cs has NO update-side or ambient AddLight: tile ambience … is scanner-seeded (`TileLightScanner`, ported in `tr-core/tile_light`)". No gem claim — the gem tiles never had a Main.cs AddLight site. |
| `W4-f25-worldgen-passes-8` | `84b90360` | C8 `GetTileMossColor` (`6e9ae957`, helper round closed `803093b1`). The C8 commit note says UNWIRED, but a LATER f25 commit wired it: `edit/kill.rs::check_long_moss` (the `TileFrameImportant` case-184 arm, `WorldGen.cs:85149-85213`) dispatches at the tree-arm slot and calls `framing::get_tile_moss_color` for the four-neighbour colour pick. **The brief's "UNWIRED" cross-note is stale — f25 owns the consumer, wired.** |
| `W4-e19-liquid-quicksell-shimmer` / `W4-f47-liquid-shimmer-2` | `3627ce22` / `507a0da5` | The extractinator residuals (chlorophyte consumable set, the `CanUseItem` gate, the smart-select trader arm, the place-path routing). The base roll (`extract::roll_extractinator_drop`, M7-EXTRACT-1) is NOT theirs to re-port — it is the landed base this sweep verifies. |
| `W4-d102-worldgen-passes-2` / `W4-d106-worldgen-passes-6` | `29e8859b` / `e2c2609c` | Classic unexecuted worldgen PASSES (#10-18 Silt..WoodTreeWalls / GrassWall..RemoveBrokenTraps). Neither touches the growth-tick moss arms. **d103** (`1c844cbb`, tail P9) owns the UNDERGROUND moss arm (`grow_moss_arm_underground`, `WorldGen.cs:73699-73755`, no NextDouble gate) — read, taken, not re-ported. |
| `W4-e12-tileobjectdata-6` | `75e10fb3` | TileObjectData remainder kinds 605-753 (emission 501-604). Its census lists the def-owning kinds — **627 is NOT among them** (605-614, 619-623, 629, …): the C# has no `addTile(627)`. |
| `W4-d60-tile-draw-particles` | `77429fe1` | `TileDrawing.DrawTiles_EmitParticles` (`TileDrawing.cs:6688-7547`) as the UNWIRED `tile_draw_particles` module — the `tileShine` table + the shine arm incl. the 178 gemspark special and the 63-68 colours. |
| `W4-g35-glow-records-wire` / `W4-f15-glow-bind-tiledraw2` / `W4-g31-tile-draw-pass-4` | `fd0a99f6` / `8ca19097` / `766a5a5a` | g35: the `Main.tileGlowMask` record table COMPLETE (37 rows incl. the moss rows `Main.cs:10026-10043`). f15: the SECOND `GetTileDrawData` switch's glow-record bind (`TileDrawing.cs:1343-1403`), the `:1178-1342` table pass **explicitly out of scope**. g31 research/136 row 14: the `Main.tileGlowMask` TABLE pass `:1178-1342` is **UNPORTED — a bind contract for the table-pass lane** (the 717 arm binds g31's `tile_ctor::lava_light_a`). |
| `W4-d105-worldgen-passes-5` | (read at `9f7b1a24`/`d590d99a`) | #43 Random Gems + #44 Moss Grass PASSES with their own `PlaceTile`-178/-184 slices (`place_long_moss`, the two-gate arm `:59703-59720`). Whole passes — not the break-drop tables. |

## Item 1 — gemsparkle DRAW (tile 178 ambient dust / light emission)

**Census.** The "gemsparkle" is the dust-43 sparkle of
`TileDrawing.DrawTiles_EmitParticles`' `Main.tileShine` arm
(`TileDrawing.cs:7413-7529`), NOT a light:

- `Main.tileShine[178] = 500` (`Main.cs:7516`), `[63..68] = 900`
  (`:7523-7528`), `[566] = 900` (`:7529`), `[211] = 500` (`:7549`).
- The 178 special (`:7453-7482`): the colour by `tileFrameX / 18` —
  0 magenta (255,0,255), 1|6 yellow (255,255,0), 2 blue (0,0,255),
  3 green (0,255,0), 4 red (255,0,0), 5 white — then
  `Dust.NewDust(…, 43, 0, 0, 254, colour, 0.5f)`, `velocity *= 0`.
- The gem-stone colours (`:7483-7506`): 63 blue, 64 red, 65 green,
  66 yellow, 67 magenta, 68 white (566 yellow, 12/665/204 red,
  639 blue, 211 (50,255,100)).
- **Light emission: NONE.** `Main.tileLighted[63..68]`/`[178]` have no
  writes anywhere in Main.cs; `TileLightScanner.ApplyTileLight` has no
  top-level case for them; `Lighting.cs` has no gem AddLight. The
  brief's "Lighting AddLight gem tiles 63-68?" premise is **REFUTED** —
  the gem tiles sparkle (dust) but do not emit light. (The `case 63-68`
  hits inside `ApplyTileLight` are the chandelier-34 / lantern-42
  frame-index sub-switches, not tile kinds.)

**Verdict: CLAIMED (d60), faithful.** d60's `tile_draw_particles.rs`
`tile_shine` table (`178 | 211 => 500`, `63..68 | 566 => 900`) and
`shine_arm` (the 178 style-colour match incl. the `1 | 6` fold, the
63-68 override chain, the dark-light early return, the
`Next(tileShine)`-before-frame-windows order) match the C# line for
line. UNWIRED on d60 (the wiring wave owns the `DrawSingleTile`
`:965-968` entry gate — g85's `3be55964` ports that gate for the orb
family). NOT re-ported here. The port's EMISSIVE/TILE_LIGHTED tables
correctly carry no gem entries (verified against the generated sets).

## Item 2 — moss PAINT colors (GetTileMossColor consumer)

**Census.** `WorldGen.GetTileMossColor` (`WorldGen.cs:81215-81255`)
maps the eleven moss/moss-brick pairs to the colour index 0-10. A
whole-tree grep finds **exactly ONE consumer**: `TileFrameImportant`
case 184 (`:85176-85202`) — the loose-moss frame-column pick
(`frameX = 22 * colour`, the frameY band re-roll). It is NOT a draw
tint: `TileDrawing.cs` never calls it; the moss TILES (179/…/627) are
solid blocks drawn with the ordinary paint colour
(`render/tiles.rs:911` etc., `tile.paint()`).

**Verdict: CLAIMED (f25 C8), faithful, WIRED at tip.** f25's
`framing::get_tile_moss_color` matches the C# map verbatim
(Option<i32> for the -1 default), and the case-184 consumer
(`edit/kill.rs::check_long_moss`) is wired at the f25 tip: the four
neighbour captures with their slope gates (above !bottomSlope, below
!halfBrick && !topSlope, left/right plain active), the `Next(3)*18`
draw BEFORE the colour tests (`:85175`), the below→above→left→right
else-if ladder with the 0/54/108/162 bands, the KillTile on no colour
neighbour. The brief's "UNWIRED" cross-note is stale. NOT re-ported.

## Item 3 — gem tile EXTRACTION (extractinator gem-stone loot)

**Census.** `ExtractinatorHelper.RollExtractinatorDrop`
(`ExtractinatorHelper.cs:5-525`): the gems arm (`:297-344`) rolls
`Next(6)` → 181/180/177/179/178/182 (Amethyst/Topaz/Sapphire/Emerald/
Ruby/Diamond) behind the five stack-adder gates (20/30/40/50/60). The
outputs are gem ITEMS (which place the 178 tile via
`DefaultToPlaceableTile(178, style)`), not the tile directly. The
mode map (`ItemID.cs:1321`): silt 424 / slush 1103 → 0, fossil 3347 →
1 (num2 *= 2), junk trio → 2, mosses → 3, poop → 4, hive → 5,
shell/obsidian → 6.

**Verdict: FAITHFUL at base (M7-EXTRACT-1), not a gap.**
`extract.rs::roll_extractinator_drop`'s gems arm (`:230-254`) matches
the C# draw-for-draw (the `Next(6)` map, the five adders, the branch
order); `extractinator_mode` matches the C# set incl. 5128; the
fossil-mode `num /= 3` integer division and the 642-hardmode ore
re-roll are verbatim. e19/f47 own the residuals around it (the
chlorophyte consumable gate, the trader arm) — read, disjoint. NOT
re-ported.

## Item 4 — neon moss CONVERT (growth-tick side)

**Census.** The runtime conversion is `UpdateWorld_OvergroundTile`'s
moss branch (`WorldGen.cs:72371-72429`): the `NextDouble() < 0.5`
gate, the clamped-box dirt-1/pearlstone-38 conversion through
`MossConversion` (`:71457-71493`) + `SpreadGrass(repeat:false)` + the
per-cell `SquareTileFrame`, then the `Next(6)`/`Next(4)` loose-moss-184
spray through `PlaceTile(184)`. The underground twin (`:73699-73755`)
has NO NextDouble gate.

**Verdict: FAITHFUL at base + d103's underground arm; the worldgen
clamps are MY DONE commit.** `growth/grass.rs::moss_conversion`
(`:225-312`) matches the C# map pair-for-pair (the moss-brick-over-38
and moss-over-1 guards, all eleven swap pairs incl. 627↔628, the
0 default); `grow_moss_arm` (`:323-371`) carries the NextDouble gate,
the box walk, the reframe, the spray. d103 (`1c844cbb`) extracted the
shared `moss_arm_body` and wired the underground twin — claimed, not
re-ported. The `neonMossBiome` WORLDGEN clamps (the num6 top-edge
clamp, the ±5.0 SET drift, the scaled-double step budget, the
exclusive paint loops) are my DONE `12a1fc97` — already landed on
this branch. **Known documented deviation (both slices)**: the 184
spray writes bare (no `PlaceTile(184)` support gate / frameY draw) —
d103's commit note records it; the runtime stream contract keeps it
until the growth-wiring wave. NOT re-ported here.

## Item 5 — rainbow moss 627 tile (TileObjectData row / framing)

**Census.** The C# `TileObjectData.Initialize` has **no
`addTile(627)`** (nor 179-183/381/534/536/539/625/626/628) — the moss
tiles are SOLID BLOCKS (`Main.tileSolid[627] = true`, `Main.cs:7029`),
not multi-tile objects. The only 627 mention in TileObjectData.cs is
the sapling-20 `AnchorValidTiles` list (`:4920-4921`) — and the port's
generated registry carries it verbatim
(`registry.rs:20842`, the 15-id list incl. 627). e12's kind census
(605-753) correctly skips 627 (no def owns it).

The 627 FRAMING is `TileFrameCosmetic`'s grass family: the moss/brick
vs dirt merge (`WorldGen.cs:82131-82142`) and the base-material
override (moss → 1, moss brick → 38, `:82194-82201`). The port's
`framing.rs::grass_frame_of` (`:233-256`) matches both, keyed on the
generated `TILE_MOSS` set (which carries 627, `Main.cs:7031`). The
627 light emission (`TileLightScanner.cs:604-610`, disco × 0.25) is
ported (`scanner.rs:811-817`); the 627 map colour
(`MapHelper` `(x+y)%9`) is ported (`map/option.rs:130`); the 627
glow-mask RECORD (313) is g35's table. The arm-F live sweep incl. 627
is my DONE `12a1fc97`.

**Verdict: premise REFUTED — no row to port.** 627 correctly has no
TileObjectData def; its framing/light/map/sweep halves are all landed
or claimed. The one UNPORTED 627 draw piece is the tileGlowMask TABLE
pass colour switch (`TileDrawing.cs:1235-1239`, disco) — part of the
`:1178-1342` pass g31's research/136 explicitly defers to the
table-pass lane (f15 out-of-scope note `d5d550c6`). NOT this lane's
port; recorded for the table-pass lane.

## The REAL gap — the gem/moss per-cell break-drop slice (PORTED)

`WorldGen.KillTile_GetItemDrops` (`WorldGen.cs:63964-66092`) carries
four gem/moss arms the port's `registry::tile_drop` + `cell_drops`
lacked on ALL 354 pushed W4 branches (the all-branch grep for the
C# anchors `:63999`/`:64009`/`:64023`/`:64945`/`:64951`/`:65493`
returned nothing):

| C# arm | Anchor | Pre-fix port behaviour | Fix |
| --- | --- | --- | --- |
| Moss tiles 179/180/181/182/183/381/534/536/539/625/627 → `dropItem = 3` | `:63999-64011` | `by_tile` leaked the MOSS item (4349-5128) — breaking moss refunded the moss, not stone | `tile_drop` arm → Some(3) |
| Moss bricks 512-517/535/537/540/626/628 → `dropItem = 129` | `:64012-64024` | `by_tile` = None (no item places a moss brick) — the gray-brick drop was LOST entirely | `tile_drop` arm → Some(129) |
| Gem-stone blocks 63-68 → `dropItem = type - 63 + 177` | `:64945-64952` | `by_tile` leaked the placing stone-block item (4640-4645) | `tile_drop` arm → Some(tile-63+177) |
| Amber stone block 566 → `dropItem = 999` | `:64953-64955` | `by_tile` leaked the Amber Stone Block 4646 | `tile_drop` arm → Some(999) |
| Exposed gems 178 → the gem by `frameX / 18` (0→181, 1→180, 2→177, 3→179, 4→178, 5→182, 6→999) | `:65493-65518` | `by_tile` leaked Sapphire 177 for EVERY style | `cell_drops` 178 arm reads `hit.frame_x` (the C# `tileCache`); `tile_drop(178)` refuses (frame_y-only accessor — the style-0 cascade/record paths drop nothing rather than the wrong gem, matching the C# `dropItem > 0` gate `:63953` for an unset column) |

The moss-block items 687-692 → 5439-5444 (`:64219-64236`) were
already correct through `by_tile` (the items place those tiles) —
verified, no arm needed.

**RNG contract**: every arm is a pure table lookup — ZERO draws, no
stream order change, no world-hash pin movement (runtime-only). The
`a == b` determinism halves are untouched.

**Witnesses (6, WRITTEN NOT RUN — code-only)**:
`registry.rs::test_w4_c29_tile_drop_gem_stone_blocks_drop_the_gem`,
`…_moss_tiles_drop_stone`, `…_moss_bricks_drop_gray_brick`,
`…_exposed_gems_refuses_the_frame_y_only_path`;
`editing.rs::test_w4_c29_break_drop_events_exposed_gems_drop_the_style_gem`
(all 7 columns + the column-7 nothing),
`…_gem_stone_and_moss_drop_their_items` (63/179/512/627 through the
`drop_events` harness). Each pins the C# line it restores and the
`by_tile` fallback it displaces.

## Status

- [x] Co-lane claims read (g18/f25/e19/f47/d102/d103/d106/e12/d60/g35/f15/g31/d105)
- [x] Item 1 gemsparkle DRAW: d60 claimed + faithful; the AddLight premise refuted
- [x] Item 2 moss PAINT: f25 C8 claimed + faithful + WIRED at tip (stale cross-note corrected)
- [x] Item 3 gem EXTRACTION: base faithful, e19/f47 residuals disjoint
- [x] Item 4 neon CONVERT: base + d103 faithful; worldgen clamps = my DONE
- [x] Item 5 627 row: premise refuted (no def); framing/light/map/sweep landed; the glow table pass deferred-with-owner (g31)
- [x] The REAL gap ported: the gem/moss per-cell break-drop slice (`55555fb9b`)
- [x] 6 witnesses written (NOT run — code-only)
- [ ] The tileGlowMask table pass (`TileDrawing.cs:1178-1342`) — the table-pass lane's bind contract (g31 research/136), NOT this lane
