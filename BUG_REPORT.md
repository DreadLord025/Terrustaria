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
