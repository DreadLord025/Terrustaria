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

## BUG-009 — Player hopped up full-tile ledges ("jumps at tile seams")

**Status:** fixed — commit `434efb4` ("Vanilla HUD, screen-snapped sprites and movement fixes")
**Area:** `tr-app/src/player.rs` (`move_x`, `slope_floor`)

**Report:** walking along the surface, the player appeared to jump by itself at tile
junctions instead of following the slope smoothly.

**Cause:** `move_x` auto-stepped a full 16 px tile whenever a horizontal collision had free
space above, so every 1-tile ledge or grass step produced a one-tick vertical teleport. That
is not Terraria behaviour: full blocks require a jump; only slopes and half bricks step.

**Fix:** removed the full-tile auto-step (slopes/half bricks keep their own collision), and
widened the slope snap from 10 px to the full 16 px cell so the feet cannot fall through a
slope triangle into the solid tile below. Added two tests: walking down a slope never moves
more than 4 px per tick; a full-tile step blocks the player instead of lifting them.

## BUG-010 — Slimes (and other NPCs) floated a few pixels above the ground

**Status:** fixed — commit `434efb4`
**Area:** `tr-app/src/main.rs` (`draw_npcs`)

**Cause:** the NPC draw offsets aligned the *frame* bottom with the hitbox, but the sprites
have transparent padding rows at the bottom (slime 2 px, zombie/king slime 3 px), so the art
sat above the floor.

**Fix:** the sprite offsets now align the last opaque art row with the hitbox bottom
(slime `y+1`, zombie `y-5`, king slime `y-25`; the demon eye is centred in its hitbox).

## BUG-011 — Clouds only moved with the player

**Status:** fixed — commit `434efb4`
**Area:** `tr-app/src/main.rs` (`draw_sky`)

**Cause:** cloud positions combined only the static layout and `camera.x * 0.5`; nothing
advanced with time, so clouds stood still until the camera moved.

**Fix:** clouds drift horizontally with the animation clock (4 px/s) on top of the existing
0.5x parallax, so they move independently of the player like `Cloud.Update`.

## BUG-012 — Player lost the torso after the arm change

**Status:** fixed — commit `434efb4`
**Area:** `tr-app/src/main.rs` (`draw_player`)

**Cause:** torso and shirt were switched to the animated body-row frame; their 9x4 sheets
have empty art in the walking rows. In vanilla the composite torso/undershirt frames are
always `(0,0)`; only the head and arms follow the animation frame.

**Fix:** torso/shirt use composite cell `(0,0)` again; head/eyes/hair keep the animation row
and the composite front/back arms use the `PlayerDrawSet` frame table.

## BUG-013 — Player texture grid crawled while moving (face changed height)

**Status:** fixed — commit `434efb4`
**Area:** `tr-app/src/main.rs` (`draw_player`, `draw_npcs`, `snap_to_screen`)

**Cause:** player and NPC sprites were drawn at interpolated sub-pixel world positions, so
nearest sampling picked different source rows/columns every frame; at fractional zoom this
made a face row appear/disappear while walking.

**Fix:** entity sprites (player, NPCs) snap their top-left to an integer screen pixel before
uploading the instance. The camera still moves smoothly; the sprite keeps a constant texel
grid.

## BUG-014 — HUD was custom, not the vanilla layout

**Status:** fixed — commit `434efb4`
**Area:** `tr-app/src/main.rs` (`draw_hud`), `tr-app/src/assets.rs`

**Report:** the HUD was a custom bottom-left hotbar, red health bar and top-centre boss bar.

**Fix:** ported the classic layout from `GUIHotbarDrawInner`,
`ClassicPlayerResourcesDisplaySet` and `BigProgressBarHelper` using the original textures:
ten `Inventory_Back` slots at the top-left (selected slot animates to 1.0 and the item name
is centred at x=236), hearts at the top-right with the "Life" label, breath bubbles above
the player's head, and the `UI_BossBar` bar with the boss head icon at the bottom centre.
The coin counter and custom bars were removed from the HUD (coins still accumulate).

## BUG-015 — Crash when pressing PrintScreen and moving

**Status:** diagnostic added — commit `434efb4`
**Area:** `tr-app/src/main.rs` (`main`, edit target bounds)

**Report:** the game closed when PrintScreen was pressed and the player moved sharply.

**Changes:** the release build now installs a panic hook that writes the message and a
backtrace to `crash.log` next to the executable, so the next crash leaves evidence; the
tile-edit path no longer panics when the cursor maps outside the world (the only
unchecked `expect("in bounds")` on the input path was replaced with a bounds check). The
exact crash could not be reproduced (PrintScreen opens the Windows snipping overlay and
changes focus), so the next run with `crash.log` will identify it.

## BUG-016 — Water corners/seams on sloped shorelines (BUG-005 reopened)

**Status:** fixed — commit `WATERCORNERS`
**Area:** `tr-app/src/main.rs` (`draw_tile_liquid_behind`), `tr-render/src/sprite.rs` (`probe`)

**Report:** green background slivers ran along every underwater slope step and the
shoreline corner still showed the parallax instead of water.

**Cause:** three divergences from `TileDrawing.DrawTile_LiquidBehindTile`:
1. the water-surrounded case keeps the original's default `(0, 4, 16, 16)` source rect;
   feeding `src_y = 4` into the `LiquidSlope_*` mask shifted the diagonal four pixels down
   and opened a seam along every hypotenuse (the `y = 4` skip only applies to `Liquid_*`,
   whose first rows are the surface highlight);
2. the right-only 4 px strip lost the original's `vector.x += 12` shift, and the
   `flag_left && !flag_right` width override also clipped slope masks to 4 px;
3. `sub_uv` subdivided corner-lit quads over a hardcoded 16 px, mis-sampling 4 px strips.

**Fix:** ported the branch selection verbatim (slope masks always full width, `+12` shift,
surrounded default rect with `src_y = 0` for masks), added the `flag5` own-liquid and
`tileSolid`/`BlocksWaterDrawingBehindSelf` early-outs, `WorldGen.SolidTile` for the
neighbour tests, and `sub_uv_frac` for non-16 px destinations. Verified with lake-bank
screenshots at zoom 1/2/3 (`TR_CAM=2240,362`): the seams are gone. `TR_PROBE=x,y` now logs
every sprite covering a world point, which is how the mis-sampled mask was found.

## BUG-017 — Player could not enter slope/step junctions

**Status:** fixed — commit `STEPCOLLISION`
**Area:** `tr-app/src/collision.rs` (new), `tr-app/src/player.rs`

**Report:** walking into sloped or stepped ground the player either bumped against it
when climbing or was thrown back a tile when descending; slopes felt solid from the
open side.

**Cause:** the movement code was an ad-hoc axis-separated AABB that skipped ground
slopes entirely and snapped the feet to a slope surface with a 16 px tolerance, while
the earlier BUG-009 fix had removed auto-stepping altogether. None of that is how the
original moves: `Collision.StepUp`/`StepDown` lift the hitbox over one-tile ledges
before the AABB pass, `SlopeCollision` slides the feet along ramps after it, and
`gfxOffY` hides the lift so the climb looks smooth.

**Fix:** ported `TileCollision`, `SlopeCollision`, `StepUp` and `StepDown` verbatim
(including the slope pass-through flags, the half-brick heights, the `num3 == num`
rule that keeps `velocity.Y` at gravity on the contact tick, and the underworld
scan margin), and drove them from `Player::update` in the original order with the
`gfxOffY`/`stepSpeed` ease and rendering at `position + gfxOffY`. Verified by unit
tests (step up/down smoothness, slope standing at the left-edge diagonal) and by
walking the lake shore in game: the player descends the slope staircase into the
lake and climbs the far bank without sticking.

## BUG-018 — Background tied to the player's height and moving in jerks

**Status:** fixed — commit `BGANCHOR`
**Area:** `tr-app/src/main.rs` (`draw_bg_layer`, cloud pass)

**Report:** the skyline followed the player's altitude and snapped whenever the camera
crossed a hill; compared with the original the whole backdrop sat in the wrong place.

**Cause:** every layer was clamped so its painted ground band started at the horizon of
the camera's *own column* (`world.surface_of`), which changes per column, and the clamp
dominated the vanilla `bgTopY`, screen-locking the layers vertically. Clouds were
anchored to the camera instead of the world. The layer scales had been tuned around that
clamp (0.85-1.85 instead of the original's 2.0-2.74).

**Fix:** removed the clamp and the per-column horizon: layer tops now come only from
`backgroundTopMagicNumber` and `ComputeScAdj`, exactly as `DrawSurfaceBG` computes them,
so they parallax at ~20-30 % of the camera's vertical speed and never jump horizontally.
Restored the original `bgScale` values and the zoom-1 background matrix, and anchored
clouds at a fixed world altitude above `worldSurface`. Verified with screenshots at the
same camera Y over a hill and over the lake (identical skyline) and 12 tiles higher
(skyline shifts ~31 px, i.e. ~0.16 parallax), at zoom 1 and 2.

## BUG-019 — Escape quit the game

**Status:** fixed — commit `ESCINV`
**Area:** `tr-app/src/main.rs` (input, `draw_hud`)

**Report:** pressing Escape saved and closed the window; the original opens the
inventory instead.

**Fix:** Escape now toggles `inventory_open`; the world is still saved on window close
and with F9. The open inventory is the original's 10x5 grid of 56 px cells at (20,20)
at `inventoryScale = 0.85` (`Main.DrawInventory`, `ItemSlot.Draw` context 0: grey
`InventoryBack`, hotbar row `InventoryBack9`), row 0 showing the hotbar stacks. The
closed hotbar now follows context 13 exactly: grey `InventoryBack` at 200/255, the
selected slot in yellow `InventoryBack14` at full white, `ItemStack`-style digits with
slot 10 drawn as "0", and item icons from the original `Item_<id>` sprites
(`item::item_id` maps slot kinds to `ItemID`), falling back to the tile frame for
kinds without a sprite yet. Slot interaction (click to move stacks) remains M3 work.

## BUG-020 — HUD and inventory text used a built-in bitmap font

**Status:** fixed — commit `FONTS`
**Area:** `tr-content` (`font.rs`), `tr-app/src/assets.rs`, `tr-app/src/main.rs`

**Report:** the fonts did not match the original in the HUD, item names, stack counts or
the inventory.

**Cause:** the HUD drew text with the debug overlay's 8x8 bitmap font; the original uses
ReLogic `DynamicSpriteFont`s from `Fonts/Mouse_Text.xnb` and `Fonts/Item_Stack.xnb`.

**Fix:** `tr-content` now decodes `DynamicSpriteFont` at runtime (see the content-pipeline
notes in ANALYSIS): one 256x256 DXT3 page each, 212 glyphs, Mouse_Text line spacing 29,
Item_Stack 24. `assets::UiFont` binds the glyph rects, padding and kerning to the atlas,
and `push_font_text`/`font_width` follow `DynamicSpriteFont.DrawString`/`MeasureString`
exactly (pen += spacing + kerning.X, draw at pen + padding, pen += kerning.Y + kerning.Z;
the first glyph clamps a negative bearing). HUD labels, item names and the Life readout
use Mouse_Text; stack counts and slot numbers use Item_Stack. Textures now load from the
retail `.xnb` files at runtime (`TR_CONTENT`, default Steam install) with the extracted
PNGs as a fallback; `Images/Tiles_0.xnb` decodes pixel-identically to the PNG.

## BUG-021 — NPC and item behaviour gaps (deferred to the NPC/AI milestone)

**Status:** recorded — not fixed yet (the user asked to defer this)
**Area:** `tr-app/src/npc.rs`, placing logic in `tr-app/src/main.rs`

**Report:**
- The Demon Eye flies through water freely; in the original most flying enemies avoid or
  behave differently in liquid.
- Torches can be placed under water; in the original a torch placed in water breaks
  immediately and drops as an item (`Item.CheckTorch`/`WorldGen` torch rules), and
  non-torch items placed in liquid follow `Item.PlaceThing`'s checks.

**Plan:** fix together with the full NPC AI port (all enemies/bosses, their AI, behaviour,
attacks and patterns), where ambient liquid rules and item placement checks are ported
from `NPC.cs`/`Item.cs` rather than special-cased. Until then the current behaviour is an
accepted approximation and must be listed in ROADMAP M4.
