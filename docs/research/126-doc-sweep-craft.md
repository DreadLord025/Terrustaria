# 126 — DOC-SWEEP lane 37: the craft/UI residuals (W4-f23)

> Wave 4 · task/W4-f23-ui-elements-4 · base adf5c908 (the deviation pass) ·
> researched 2026-11-21 · code-only mode: witnesses written, not run.
> Sources: `decompiled/Terraria/Recipe.cs`,
> `GameContent/UI/NewCraftingUI.cs`, `UI/CraftingUI.cs`,
> `GameContent/UI/Elements/{UIList,UIScrollbar,UIDynamicItemCollection}.cs`,
> `GameContent/UI/States/UIWorldLoad.cs`, `Main.cs`.

The lane took the six craft/UI residuals the wiring census (research/18e)
left open. Three were real gaps and landed; three were verify-only and are
recorded here with their evidence.

## 1. Refocus-on-craft — LANDED (`beb2af45`)

`Recipe.TryRefocusingRecipe` (`Recipe.cs:477-495`) is the arm the earlier
census cited but never ported. `UpdateRecipeList` (`:230-259`) captures
`oldRecipe = Main.availableRecipe[Main.focusRecipe]` BEFORE
`ClearAvailableRecipes` (`:232-233`), rebuilds the list, then refocuses
(`:257`): the walk finds the old recipe's NEW position (`:479-486`), the
first clamp pins a focus past the new end to the last row (`:487-490`), the
second pins an empty list to 0 (`:491-494`). So after a craft consumes the
last ingredients, the focus does NOT stay on the stale position — it
follows the recipe while it survives, then rides the last row.

- **Port:** `craft::try_refocusing_recipe(available, old_recipe, focus)`
  (`crates/tr-app/src/craft.rs`), arm for arm.
- **Wiring:** the draw-path rebuild (`app/frame/draw.rs`, the
  `craft_available` computation) runs it every frame the panel draws —
  exactly where `NewCraftingUI.Draw` → `UpdateContents` (`:585`) →
  `UpdateRecipeList` runs it in the C#. `App.craft_last_available` snapshots
  the RAW list so the `:232-233` pre-rebuild capture survives the frame
  boundary (the C# keeps the array across the clear; the port recomputes).
- **Deviation (recorded in ANALYSIS "Crafting panel"):** the C# also runs
  `UpdateRecipeList` while the panel is CLOSED (`Main.cs:41712` — the
  classic pip list's arm, unported). The port's snapshot keeps the last
  open frame's list across a close/reopen instead of tracking every
  inventory change while closed.
- **`VisuallyRepositionRecipes` (`CraftingUI.cs:132-139`, called at
  `Recipe.cs:239`/`:258`)** shifts the CLASSIC pip list's
  `availableRecipeY` offsets (`65·i`, `:126-129`) — it has no effect on the
  modern grid. Unported with that list; NOT a gap in the modern panel.
- **Witnesses (written, not run):** `craft.rs` 3 (follow-the-recipe,
  last-row clamp, empty-list pin) + `clicks.rs` 1 (the craft → rebuild →
  refocus sequence end to end).

## 2. Craft-grid SCROLL — LANDED (`d2bbd976`)

The panel drew a static 1-of-1 scrollbar (`crafting.rs`'s old
`draw_scrollbar(…, 1.0, 1.0, 0.0, true)`); the C# grid scrolls. The full
machine:

| C# | Port |
| --- | --- |
| `UIDynamicItemCollection.UpdateSize` (`:160-165`): `cols = Width/44`, `MinHeight = 44·ceil(n/cols)` | `crafting::grid_cols` / `grid_content_height` (the fill height floors it — a short grid never scrolls) |
| `UIList.DrawSelf` (`:181`): `_innerList.Top = -ViewPosition` | `crafting::grid_start`'s lifted rect top |
| `GetGridParameters` (`:81-97`): the int-toward-zero start-row snap `num = (min(cullTop, rectTop) − cullTop)/44`, the `+cols` cull slack (`:92-96`) | `grid_start` / `grid_cull_range` — the SAME window bounds the draw loop and `hit_test` (the C# `DrawSlot` — and so `HandleCraftSlot` — never fires outside it) |
| `UIList`'s `OverflowHidden` (`:59`) scissor | `r.push_clip(l.list)` around the grid loop (the d88 scissor machinery) |
| `UIScrollbar` AutoHide/Cyan (`NewCraftingUI.cs:376-383`), `SetView` clamp (`:72-78`), handle alpha 1-vs-0.85 (`:122`), hover-enter sound 12 (`:119`) | `panel::draw_scrollbar_handle_alpha` (the shared bar renderer grew the live-alpha variant; the select screens keep 0.85), the App-side latch + sound in `frame/draw.rs` |
| `UIList.ScrollWheel` (`:107-114`): `ViewPosition -= ScrollWheelValue` (120/notch) | `crafting::scroll_wheel` over `char_select`'s shared `WHEEL_STEP`/`clamp_scroll`; `App::craft_wheel` in the loop's wheel arm |
| `Main.cs:17453-17456`: `NewCraftingUI.Visible` sets `flag = false` — the classic list never also scrolls, and the accumulator is consumed whatever the hit | the wheel arm returns early while `crafting_open` |
| `UIScrollbar.LeftMouseDown` (`:126-143`): handle grab (`_dragYOffset`) / track jump; `LeftMouseUp` (`:148`) clears the drag; `DrawSelf` (`:109-112`) drags | `crafting::scrollbar_press`/`scrollbar_drag`, `App::craft_scrollbar_press`/`_release`, the drag frame in `frame/draw.rs` |

**Hover containment:** `base.IsMouseHovering`
(`UIDynamicItemCollection.cs:52`) walks up through the `UIList`'s
`ContainsPoint` (`UIElement.cs:354-357`) — the grid does not override it
(the always-true `ContainsPoint` is `UIInnerList`'s, one level BELOW the
grid). So a row lifted above the list top draws clipped but is NOT
hoverable/clickable; `hit_test` and the draw's hover gate both require the
pointer inside the list rect. The straddling row's visible sliver IS
clickable (the hitbox spans the full 44 px).

**State:** `App.craft_scroll` (ViewPosition), `craft_scroll_drag`
(`_isDragging`'s offset), `craft_scroll_hover` (`_isHoveringOverHandle`).
`SetView`'s clamp re-runs every rebuild, so a shrinking list pulls the view
back exactly like the C#.

**Witnesses (written, not run):** `crafting.rs` 4 — content height +
culling window at 1600×900 (list 404 tall, 10 cols; 110 recipes → 80 px
travel), the wheel's 120 px/notch + rails + target, the scrolled hit-test's
list containment (the straddling-row quirk), the bar grab/jump/drag.

## 3. Recipe FILTER tabs — VERIFIED COMPLETE (no gap)

All eleven categories landed in `craft_filter.rs` (S1-H-7), verified arm
for arm against `ItemFilters.cs` this sweep: Weapon (`:191-212`), Armor
(`:226-247`), Vanity (`:249-270`), BuildingBlock (`:71-104`, the witness
tab), Furniture (`:106-132`), Accessories (`:304-325`), MiscAccessories
(`:327-348`), Consumables (`:350-381`), Tools (`:134-189`, the 58-id list),
Materials (`:443-464`), MiscFallback (`:406-441`, the ten-filter
complement). Construction order = `NewCraftingUI.cs:282-298`; the exclusive
toggle = `UICreativeItemsInfiniteFilteringOptions.cs:80-100`. The recorded
deviations stand (tileWand uncarried, BySearch unwired, the memo cache
omitted — ANALYSIS "Crafting panel").

**Favorite-recipe star: DOES NOT EXIST.** `NewCraftingUI.cs` (996 lines)
and `Recipe.cs` carry no favorite/star state — grep for
`favorite|star` returns only `AddStardustFurniture`/`AddFallenStarFurniture`
recipe builders. The inventory's `Item.favorited` is a different system.
Nothing to port; the census question is closed.

## 4. Craft-all / craft-stack right-click — PARITY LANDED (`3720a75b`)

**There is no craft-max in 1.4.5.8.** `HandleCraftSlot` (`:499-528`) and
`HoverOverCraftingItemButton` (`Main.cs:42250-42306`) treat the buttons
identically: the select edge is `(mouseLeft && mouseLeftRelease) ||
(mouseRight && mouseRightRelease)` (`:505`, `:42254`), the craft gate is
`flag2 = mouseLeft || mouseRight` (`:42255`), and `num = superFastStack + 1`
with the Shift ×10 (`:42271-42274`) — no right-click-specific quantity.
The port was left-only; the sweep landed the parity:

- either button's press edge selects (consumes both edges),
- either HELD button keeps `prevent_craft_click` + the recipe pin
  (`:42301-42305`'s `!flag2` reset) and the held-repeat pacing,
- the craft itself is button-agnostic.

The filter tabs and the nearby-chests toggle stay left-only — their C#
reads `mouseLeft` only (`UICreativeItemsInfiniteFilteringOptions.cs:80-100`,
`CraftingUI.cs:346`).

**Witnesses (written, not run):** `clicks.rs` 2 — the right-button
select→craft sequence (one recipe, NOT the max) and the held-right lock
retention.

## 5. Loading-screen DEBUG-DEFERRED — contract documented (no new port)

The loading-screen DRAW is already landed and wired (S1-E-17):
`ui/world_load.rs::draw` paints the `UIGenProgressBar` + `UIHeader` from
the live job's snapshot (`FrontScreen::WorldLoad`, `app/flow.rs:478-491`),
including the secret-seed message overrides and the abort latch. The
`pass_name_id` + `get_step_icon` composition (deviation pass, `09dbb9cc`)
means the screen CAN look the 108-row icon table up.

**What stays deferred is the DRIVER, not the draw:** `UIWorldGenDebug`
(the F5 screen, `UIWorldLoad.cs:50-53`) is unported (d86 scope-out), so the
pass-list elements (`worldgen_debug_elements.rs`) and the icon table have
no consumer. The contract for whoever ports the driver:

1. `UIWorldLoad.Update`'s F5 arm (`DebugOptions.enableDebugCommands &&
   keyState.IsKeyDown(F5)` → `UIWorldGenDebug.Open()`) is parked at
   `app/flow.rs` (research-19 L1).
2. The pass list feeds `PassState` per executed step; the display name →
   `GenPassNameID` key mapping is `tr_core::worldgen::pipeline::pass_name_id`,
   the icon lookup is `ui::worldgen_pass_icons::get_step_icon`
   (bestiary-64 fallback for unmapped/port-only steps).
3. The element halves (`tooltip_publish`, `extra_icon_rect`/`tint`,
   `pass_text_color`/`pass_panel_colors`/`pass_title`/`pass_description`,
   `matches_search`) are pure and parked `#![allow(dead_code)]` — the
   driver wires them, no re-port needed.
4. The drunk-world `Main.rand` arms (`UIGenProgressBar.cs:63-67`,
   `UIWorldLoad.cs:93-103`) stay deferred with the RNG-order rule
   (ANALYSIS "The loading screen") — wiring them perturbs the stream.

## 6. research/18e remaining rows — VERIFIED CLOSED (nothing unclaimed)

The 18e closure verdict stands after this sweep's re-check: every
transitive `UIElement` class is accounted for (62 → d81/d82/d83, the
nested remainder + `UIState` root → f23, `UIInnerList`/`UIInnerProgressBar`
→ S1-H-13/d88/d83, screens → d84/d85/d86/c24/f22, the framework base →
research/18 P6). The two DEBUG-DEFERRED dev screens
(`UIWorldGenDebug`, `UITextWrappingTest`) are d86 scope-outs with their
element halves already landed here; the `text_wrapping_test` TR_UI preset
was evaluated in the wiring pass and is a whole screen (a new
`FrontScreen` variant with exhaustive arms), not a small row. No small
unclaimed element remained to take.

## Commits

| Hash | Mechanic |
| --- | --- |
| `beb2af45` | 1 — refocus-on-craft (`try_refocusing_recipe` + the draw-path wiring + 4 witnesses) |
| `d2bbd976` | 2 — craft-grid scroll (the UIList/UIScrollbar machine + 4 witnesses) |
| `3720a75b` | 4 — right-click craft parity (+ 2 witnesses) |
| (this doc) | 3/5/6 — the verify-only census rows + the deferred-driver contract |

10 witnesses written, not run (code-only lane). Docs updated: ANALYSIS
"Crafting panel" (the scrollbar bullet retired, three new bullets),
UI.md "Crafting panel" (the scroll + click paragraphs), PORT_MAP lane-4
row, anchors.txt (3 rows).
