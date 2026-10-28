# Bug report

Tracked issues reported from in-game screenshots. The current performance plan is in
`docs/ROADMAP.md` (M1.5); this file is only for correctness/visual defects. Keep this file up to date: when a bug is fixed,
record the commit and the verification (tests / screenshots).

## BUG-001 — Liquids are too dark and do not light the water / underwater terrain

**Status:** open
**Area:** `tr-core/src/lighting.rs`, `tr-app/src/main.rs` (`draw_liquids`), `Liquid`

**Report:** the generated lake looks almost black under the surface. Sunlight does not
seem to reach the underwater ground and the water body is much darker than in the original.
The water surface shows hard repeated stripes instead of a continuous lit surface.

**Observed:** lake screenshots at zoom 1.5–2.0; the surface row is visible but the body
darkens within a few tiles and the lake bed is effectively black.

**Reference:** `LightMap.cs` uses `LightDecayThroughWater = (0.88, 0.96, 1.015) * 0.91`
and `TileLightScanner.ApplySurfaceLight` refuses to seed cells with `liquid >= 200`; light
must still carry into the water from the adjacent air. `LiquidRenderer.DrawNormalLiquids`
also uses a per-corner visible height and a depth/tint pass that our single-quad pass does
not reproduce.

**Next steps:** verify the water mask decay and the sunlight seed near the shoreline with
`TR_LIGHT_AT`; compare a lake cross-section with the original at the same camera. Check
whether the liquid pass's depth tint and opacity double-darken the already attenuated light.

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

**Status:** open
**Area:** tile / wall instance emission, sliced-block lighting, dark-rect merging

**Report:** underground, at several zoom levels, a regular grid of slightly lighter lines
is visible between the otherwise black tiles ("empty spaces between blocks"). It is visible
at fractional camera/zoom positions and looks like the cave-fade background showing through
1 px seams.

**Observed:** wide underground screenshot below the surface; the seams repeat every tile in
both axes.

**Next steps:** reproduce at zoom 1.0, 1.5 and 2.0 with `TR_CAM` in a cave; dump the edge
pixels with `TR_SHOT` and compare the tile/wall/dark-rect quads. Check the 9-slice block
quad edges, the `TILE_PX + 0.5` overlaps, and whether walls are skipped under fully dark
cells while the background strips are still visible. Any fix must keep the render
pixel-identical at the reference camera.
