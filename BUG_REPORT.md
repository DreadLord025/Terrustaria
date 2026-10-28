# Bug report

Tracked issues reported from in-game screenshots. The current performance plan is in
`docs/ROADMAP.md` (M1.5); this file is only for correctness/visual defects. Keep this file up to date: when a bug is fixed,
record the commit and the verification (tests / screenshots).

## BUG-001 — Liquids are too dark and do not light the water / underwater terrain

**Status:** fixed — commit `fc0c9bb` ("Port the vanilla liquid draw cache and two-pass water")
**Area:** `tr-core/src/liquid_draw.rs`, `tr-core/src/lighting.rs`, `tr-app/src/main.rs`

**Report:** the generated lake looked almost black under the surface, sunlight did not
reach the underwater body and the surface showed hard repeated stripes.

**Cause:** the liquid pass was a hand-written approximation, not the original's:
- every tile sampled the 16 px wave rows of `Misc/water_*` (`32 + frame*80`), so open
  water repeated white squiggles in a grid; `LiquidRenderer` uses the plain interior row
  (`FrameOffset` (16,48)) and only surface/edge cells use the crest frames;
- light was a flat per-tile sample instead of `Lighting.GetCornerColors`' 3x3 corner
  averages, which is what makes the water surface bright and the body smooth;
- an invented depth tint darkened the already attenuated water, and a `+1` px quad
  overlap double-blended a dark 1 px line at every tile row; a per-tile sine offset
  misaligned neighbouring columns;
- water was drawn once at 0.6 alpha, skipping `Main.DrawLiquid(bg: true)`, the opaque
  background pass. The parallax showed through the lake and the body composited over a
  dark background, so it looked black instead of the original's solid lit blue.

**Fix:** ported `LiquidRenderer.InternalPrepareDraw` (visible levels, edge walls,
`FrameOffset`, source rectangle, waterfall-fade opacity, `IsSurfaceLiquid`, half-brick
visibility), added `LightMap::corner4` (`Lighting.GetCornerColors`) and split each visible
rect into 3x3 sub-quads that sample the bilinear corner light. The draw runs twice like
`Main.DrawLiquid`: opaque before walls/tiles, `DEFAULT_OPACITY` (water 0.6, lava/honey
0.95, shimmer 0.75) after the entities. Removed the depth tint, the per-tile wave offset,
the overlap and the unused shader UV-ripple flag.

**Verification:** unit tests in `liquid_draw.rs` (open surface uses frame 1280, interior
(16,48), shore column picks the x=0 edge frame); lake screenshots at zoom 1.0, 1.5 and 2.0
plus a zoom-4 shore shot; `TR_BENCH=240` at the lake: 1.44 ms average (697 fps),
0.01 ms draw on cache hits.

## BUG-002 — Tree bases / forks detached from the trunk

**Status:** fixed — commit `318f128` ("Port the GrowTree base forks …")
**Area:** `tr-core/src/worldgen.rs` (`plant_trees`), `TileDrawing.DrawTrees`

**Report:** at the foot of common trees a small stump/block was visibly detached from the
trunk and offset to one side (circled in the report screenshots).

**Cause:** the generator used its own fork rule: the right fork used `frameX = 66` (which is
a branch marker frame) and the lowest trunk row used a random straight frame instead of the
root frames. `WorldGen.GrowTree` instead chooses a base style (`num5`), sets the lowest trunk
tile to `frameX 88/0/66`, `frameY 132/154/176`, and adds a right fork with `frameX 22` or a
left fork with `frameX 44` (`num5` decides which sides), depending on grass on both sides.

**Fix:** ported the `GrowTree` base-style selection, root frames, side-fork frames and the
both/left/right grass conditions. Verified by regenerating the default world and a zoom-3
screenshot: trunk, roots and ground are connected.

## BUG-003 — Empty seams between blocks at some zoom levels

**Status:** fixed — commit `e389936` ("Close tile seams with a half-texel sprite UV inset")
**Area:** `tr-render/src/sprite.wgsl`

**Report:** underground, at several zoom levels, a regular grid of slightly lighter 1 px
lines appeared between the otherwise black tiles, as if the cave-fade background showed
through gaps.

**Cause:** when a camera/zoom alignment put a sprite edge exactly on a pixel centre, the
edge pixel interpolated `uv = 1.0`; nearest sampling then picked the transparent 2 px gap
between the 18 px frames of the tile sheet. Every tile row (or column, depending on the
phase) lost its outermost pixel, exposing the background in a grid. That is why the
artefact only appeared at some zoom levels and camera positions and why it vanished when
the view moved.

**Fix:** the vertex shader now insets the interpolated UV by half a texel toward the
sprite centre (`mix(uv0 + 0.5*texel, uv1 - 0.5*texel, corner)`), so nearest sampling
always lands on a texel centre inside the source rect. This applies to tiles, walls,
liquids, entities and raw sheets alike.

**Verification:** reproduced with the logged spawn camera (tile 2063.657, 361.688, zoom
1.5): 1 px background rows every 16 world px. After the fix the same camera and zooms 1.0,
1.25, 1.5, 1.75 and 2.0 show no periodic rows in the underground; surface/lake screenshots
are unchanged apart from the seams.

## BUG-004 — Underwater terrain stayed at daylight brightness

**Status:** fixed — commit `84e7c20` ("Darken underwater terrain, fill shore slopes and fix mobs/player")
**Area:** `tr-core/src/lighting.rs` (`seed`/`surface_cutoff`), `tr-app/src/main.rs`

**Report:** the lake bed and the part of the bank below the waterline were bright brown, as
if they were above water. In the original everything under a liquid is attenuated/darkened.

**Cause:** surface light seeds every cell up to each column's first solid tile. For a lake
column that tile is the lake bed, so it received a full sky seed even though several water
cells sit above it; the water cells themselves were dark (their seed is blocked by
`liquid >= 200`), which made the bed brighter than the water.

**Fix:** the per-column surface cutoff is now the top of the terrain **or** the topmost
liquid cell, whichever is higher, so a lake bed is lit only by the light that propagates
through the water (`LightDecayThroughWater = (0.88,0.96,1.015)*0.91`). `TR_LIGHT_AT` at the
lake bed went from `0.996` to `0.46/0.59/0.70` (blue-tinted), below the water column.

## BUG-005 — Background showed through the gap between water and sloped ground

**Status:** fixed — commit `WATERLIGHT`
**Area:** `tr-app/src/main.rs` (`draw_tile_liquid_behind`), `tr-app/src/assets.rs`

**Report:** along lake shores a regular row of background-coloured wedges sat between the
water quads and the diagonal dirt slopes, looking like holes.

**Cause:** the liquid cache only draws liquid cells; a sloped block's open corner is not
covered by the neighbouring water cell. The original fills it with
`DrawTile_LiquidBehindTile`, which our renderer did not port.

**Fix:** ported the partial-liquid branch: solid tiles with adjacent liquid draw a
`LiquidSlope_*` mask (or a `Liquid_*` 4 px strip for blocks) behind their open part, with
the original's `num14` water-level offset, corner lighting and wall/surface opacity rules.
The original skips the quad when water is both above and beside a slope and keeps 0.5
opacity below `worldSurface`; that is exactly where the visible vanilla shoreline gaps come
from, so for submerged cells we fill the corner and use full opacity (intentional deviation,
documented in `docs/ANALYSIS.md`).

## BUG-006 — Slimes sink instead of floating

**Status:** fixed — commit `84e7c20` ("Darken underwater terrain, fill shore slopes and fix mobs/player")
**Area:** `tr-app/src/npc.rs` (`update_slime`)

**Report:** slimes dropped into the lake sank and stayed under water; in the original they
bob on the surface.

**Fix:** ported the `AI_001_Slimes` water tail: any liquid overlapping the body sets `wet`,
gravity becomes `0.2` with max fall `7`, draining `vy` by `0.5` per tick (floor `-4`), a
`-2` bounce when touching the floor/surface and the original direction flip when rising
along the same x. Verified with `TR_SLIME_AT`: two slimes float at the lake surface.

## BUG-007 — Player had no arms and a fixed jump height

**Status:** fixed — commit `84e7c20`
**Area:** `tr-app/src/player.rs`, `tr-app/src/main.rs` (`draw_player`)

**Report:** the player sprite showed no arms; the jump always reached the same height and
did not depend on holding the space bar.

**Fix:** arms now use the composite layout from `PlayerDrawSet.CreateCompositeData`
(`Player_0_7` 9x4 cell sheet; back arm two rows lower, front arm on the near side). The jump
ports `Player.JumpMovement`: jump speed `5.01`, `jumpHeight 15`; holding the key keeps
resetting the vertical speed for up to 15 ticks (tap ≈ 31 px, full hold ≈ 95 px), and a new
jump requires a fresh press edge. Added a tap-vs-hold unit test.

## BUG-008 — Old save still showed detached tree stump tiles

**Status:** fixed by regenerating the world (no code change; generator fix was `318f128`)
**Area:** `Documents/My Games/Terrustaria/Worlds/Terrustaria.trwld`

**Report:** trees with a 16x16 bark block detached beside the trunk at ground level, even
after the `GrowTree` fork port.

**Cause:** tree trunk/fork frames are stored in the save; the world was generated before the
fork fix, and the load path only re-frames autotiled blocks, not tree markers.

**Fix:** the world was backed up to `.qoder_tmp/world_backup/` and regenerated (seed 1) with
`TR_REGEN=1`; the new trees show connected roots/forks. Existing worlds written by the old
generator keep their stored tree frames until regenerated.
