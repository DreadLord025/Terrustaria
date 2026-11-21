# 18e — UIElement census lane 4 (W4-f23): the nested classes the file census missed

> Wave 4 · task/W4-f23-ui-elements-4 · base a725acb0 (= d81/d82/d83 base) ·
> researched 2026-11-20/21 · code-only mode: witnesses written, not run.
> Sources: the pushed claims `origin/task/W4-d81-ui-elements-1` @ `1a1a5df3`
> (rows 1-21), `origin/task/W4-d82-ui-elements-2` @ `b5d41e4a` (rows 22-34),
> `origin/task/W4-d83-ui-elements-3` @ `17799c64` (rows 35-64), the screens
> waves' phase-0 claims (d84 `e0f69499`, d85 `b0ef93b0`, d86 `c9855a4f`) and
> f22's verify-only screens census (`d9f9b60a`, research/18d).

## Method — the transitive closure, not the file list

The d82-published census (research/18b) enumerated **files** under
`GameContent/UI/Elements/` (74) + `States/UISortableElement.cs`. This lane
re-censused by **class declaration**: every `class X : B` in all 1 551
decompiled `.cs` files (junction-safe walk, no recursion into reparse
points), then the transitive closure over `UIElement` — **107 classes**
(including `UIState` and its 28 screen descendants).

Cross-checking the closure against every pushed claim:

- All 74 `Elements/` files are accounted for by 18b (63 rows + the 11
  interfaces/already-ported exclusions). **No missing top-level element.**
- All 28 `UIState` screens are claimed (10 PORTED re-verified by f22, 18 by
  d84/d85/d86/c24 — research/18d). Screens are NOT this lane's scope.
- `UIList.UIInnerList` — covered by S1-H-13 + d88's scissor lane
  (`achievements.rs`'s `aabb_overlap` ports `UIList.cs:21-32` verbatim).
- `UIProgressBar.UIInnerProgressBar` — covered by d83 row 46
  (`ui/progress.rs::ProgressBar::fill_draw`, `UIProgressBar.cs:11-15`).
- `UIState` itself (the base, `Terraria/UI/UIState.cs:3-13`) — research/18's
  PARTIAL row "no shared root type"; d87 ports the state *stack*, not the
  root element. **MISSING → landed here.**
- The **nested classes outside `Elements/`** the file census never
  enumerated: `NewCraftingUI.RecipeEntry` + `NewCraftingUI.ItemGrid`
  (`NewCraftingUI.cs:23-76`), `UIWorldGenDebug.TooltipElement` /
  `.UIImageButtonWithExtraIcon` / `.GenPassElement`
  (`UIWorldGenDebug.cs:29-735`), `UITextWrappingTest.TestElement`
  (`UITextWrappingTest.cs:26-108`). **MISSING from all lanes → landed here.**
  (The two dev screens are d86 scope-outs; their *element halves* are pure
  state/geometry and port cleanly without the controller/manifest plumbing.)

No re-split of any 18b row; no screen ports; the d81 batch-1 keep-one note
(UICyclingImage/UICreativePowerButton → d82 homes) stands.

## Landed (3 commits, 3 classes each)

| C# class (file:lines) | Rust home | What landed |
| --- | --- | --- |
| `UIState` (`UI/UIState.cs:3-13`) | `ui/state_root.rs::StateRoot`/`root_rect` | the 100%×100% ctor rect through the shared `uistate::resolve` + `NoGamepadSupport`'s only consumer arm (`UILinkPointNavigator.cs:280-283`: gameMenu ∧ visible ∧ non-null state ∧ flag) |
| `NewCraftingUI.RecipeEntry` (`NewCraftingUI.cs:23-39`) | `ui/crafting_grid.rs::RecipeEntry` | the readonly recipe index + the -1 `availableIndex`/`gridIndex` + `Available >= 0` + the `Main.recipe[index]` read |
| `NewCraftingUI.ItemGrid` (`:41-76`) | `ui/crafting_grid.rs::draw_slot`/`slot_hovering`/`is_grid_point` | the DrawSlot machine: the mouse/gamepad highlight-static swap (`:59-68`, `DrawSelectionHighlightForGridSlot` = selected-index match vs the gamepad shortcut = hover), the hover latch (`:69-73`), the White-vs-`DisabledSlotColor(160,160,160,255)` tint (`:74`, `:994`), `IsGridPoint`'s `[20100,21000)` window (`:754-761`), the base walk's hover gate (`UIDynamicItemCollection.cs:52`, int-truncated mouse + exclusive edges + `!IgnoreMouseInterface`) |
| `UIWorldGenDebug.TooltipElement` (`UIWorldGenDebug.cs:29-57`) | `ui/worldgen_debug_elements.rs::tooltip_publish` | parent-hover gate, null-description→`""`, `IgnoresMouseInteraction=true` pin, StrongRed10 (=10) fake-item rarity |
| `UIWorldGenDebug.UIImageButtonWithExtraIcon` (`:93-170`) | `ui/worldgen_debug_elements.rs::extra_icon_rect`/`extra_icon_tint`/`set_button_state` | ToRectangle truncation + `Inflate(-2,-2)` inset, the non-square aspect fit (short-side ratio shrinks the long side, int truncation + integer-halved centre shift, frame-size-over-texture-size pick), `IconColor * visibility` XNA truncation, `SetButtonState` 1/0.4 vs 0/0 + `IgnoresMouse` (`:840-852`) |
| `UIWorldGenDebug.GenPassElement` (`:172-735`) | `ui/worldgen_debug_elements.rs` (PassIconEntry factories, `pass_text_color`, `pass_panel_colors`, `pass_title`/`pass_description`, `matches_search`) + `ui/worldgen_pass_icons.rs` (the 108-row `passIcons` table + bestiary-64 fallback) | the icon factories incl. the `regionW > regionH ? regionW : ASSET-height` fit-denominator quirk + 1.2 cap, `AddIcon`'s integer-halved 26 px centring, padding-2/height-96 pins, the running>skipped>completed>disabled>white text colour + 0.6 search dim, the hover/rest panel pairs with the highlight→target→disabled override order (target border NOT dimmed; disabled lerps the CURRENT bg then re-multiplies), the title/description line machines, `MatchesSearch`'s trim+lowercase contains |
| `UITextWrappingTest.TestElement` (+`Mode`) (`UITextWrappingTest.cs:16-108`) | `ui/text_wrapping_test.rs` | the ≤0 inner-width→1000f wrap fallback + `(int)(num/scale)` truncation, the 30·scale pitch + row anchors, the per-mode MinHeight source (lineCount·pitch vs measured Y vs the `Mode.UIText` no-case no-op), the six-arm draw dispatch incl. the legacy Black-shadow-then-White two-pass |

**Witnesses:** 16 tests written not run (code-only lane): state_root 2,
crafting_grid 4, worldgen_debug_elements 6, text_wrapping_test 3,
worldgen_pass_icons 1. The 108-row icon table was mechanically diffed
against `InitPassIcons` + the `GenPassNameID` string values (0 mismatches).

## Deliberate deviations (recorded per the golden rule)

- **Caller's seams stay caller's:** `ItemSlot.Draw` (context 41) is
  `ui::slot`'s; the wrap/measure functions (`Utils.WordwrapString*`,
  `ChatManager.GetStringSize`) feed `text_wrapping_test` as data; the
  worldgen controller/manifest/snapshot state feeds `PassState` as data;
  `Main.DisplayAndGetFakeItem` is the tooltip renderer's.
- **Gamepad as data:** `IsGridPoint`/the shortcut highlight/`NoGamepadSupport`
  port as pure predicates; gamepad navigation itself stays unported
  (research/18 deviation 10, Stage-1 exclusion).
- **Dev screens stay scoped out** (d86): the element halves land as parked
  pure helpers (`#![allow(dead_code)]`), no screen wiring.
- `crafting.rs`'s existing `recipe_cell` geometry is untouched —
  `crafting_grid` supplies the C# class's state machine, adopted by the
  live grid in the wiring pass below (the S1-H-13 pattern), incl. the
  selected-recipe `InventoryBack24` overlay (`ItemSlot.cs:2371-2374`).

## Wiring census (W4-f23 wiring pass, 2026-11-21)

The five lane-4 modules were parked unwired; this pass censused their live
consumers and wired what has them:

| Module | Verdict | Evidence |
| --- | --- | --- |
| `state_root.rs` | **WIRED (keep-one)** — the nine landed full-screen root rects adopt `root_rect`: achievements/bestiary/emotes `panel_rect`, keyboard/char_creation/world_creation `layout`, world_load's bar+header resolves, char_select's `root_rect` parent (world_select delegates to it). Identity proven by `the_root_fills_its_parent_exactly`; zero pixel change. `StateRoot`/`no_gamepad_blocks_navigation` stay parked: the only `NoGamepadSupport` setter is `UIWorldGenDebug`'s ctor (`:883`, d86 scope-out) and the navigator is unported. | commit `fadad2be` |
| `crafting_grid.rs` | **WIRED** — `NewCraftingUI` IS landed (`ui/crafting.rs`, live draw `ui/mod.rs`, clicks `app/clicks.rs`, `App.focus_recipe`): the grid loop builds `RecipeEntry` per shown recipe, `draw_slot` picks the overlay+tint (mouse arm), `slot_hovering` gates hover AND `hit_test` (exclusive edges — the old inclusive `<=` accepted the exact edge pixel), `slot_tint` rides `Available`, `grid_selection_highlight` draws the new `ui_back24` sheet in `Main.inventoryBack`. Selection = `available[focus_recipe]` (`OnActivate` `:904`). Deviation pass (same lane): `available_position` ports `:610`'s RAW-list mapping (the click path stores the raw position into `focus_recipe`, `:510` — filter-active divergence fixed), the cells are the C# 44×44 hitboxes centred on the UIList rect (`GetGridParameters` `:85-86`) with the `Inventory_Back18` background at `inventoryScale = 3465/4095` (exactly 44 px, no stretch), and `ui/inv_pulse.rs` ports the exact `invAlpha` curve (`Main.cs:45697-45708`) — the 210 midpoint stand-in retired. Gamepad arm still parked (ANALYSIS "Crafting panel"). | commits `92a05687` + deviation pass |
| `worldgen_debug_elements.rs` | **DEBUG-DEFERRED** — no driver: `UIWorldGenDebug` unported (the `UIWorldLoad.cs:50-53` F5 arm parked at `app/flow.rs`, research-19 L1); the F3 workbench (`debug_panel.rs`, the e18 unlockMap precedent) has no pass-list tab; the loading screen draws only bar+message. | module-header census note |
| `worldgen_pass_icons.rs` | **DEBUG-DEFERRED** with its element half — the name-mapping half IS ported (deviation pass): `tr_core::worldgen::pipeline::pass_name_id` maps `EXECUTED_PIPELINE` display names → `Step::vanilla` (the `GenPassNameID` keys), and `get_step_icon` composes it with `get_pass_icon` — the loading screen CAN look the table up now; the screen itself stays deferred (no driver). | module-header census note + `get_step_icon` |
| `text_wrapping_test.rs` | **DEBUG-DEFERRED** — the `TR_UI` preset was evaluated and is not cheap: `UITextWrappingTest` is a whole screen (UIList + mode/scale/lang labels + cycling, `:110-336`), needing a new `FrontScreen` variant with exhaustive draw/click/key arms — d86 scope, not a wiring pass. | module-header census note |

## Closure verdict

After this lane the transitive `UIElement` closure is fully accounted for:
62 missing concrete classes → d81/d82/d83 (18b), the nested remainder +
the `UIState` root → f23 (this doc), `UIInnerList`/`UIInnerProgressBar` →
S1-H-13/d88/d83, screens → d84/d85/d86/c24/f22, framework base
(`UIElement` itself) → research/18's P6 rows. No UNCLAIMED element remains.

## DOC-SWEEP lane 37 addendum (2026-11-21, research/126)

The craft/UI residuals this lane's wiring census left open were taken by
the doc-sweep: the refocus-on-craft arm (`Recipe.TryRefocusingRecipe`
`:477-495`) and the craft-grid SCROLL (the `UIList`/`UIScrollbar` machine)
landed live, right-click craft parity landed (no craft-max exists in the
source), the eleven filter tabs verified complete (no favorite-recipe
star exists in `NewCraftingUI`), the loading-screen draw confirmed already
wired (S1-E-17 — the deferred half is the `UIWorldGenDebug` DRIVER,
contract in research/126 §5), and the closure verdict re-verified: no
small unclaimed row remained. Commits `beb2af45` / `d2bbd976` / `3720a75b`
+ the research/126 doc; 10 witnesses written not run (code-only).
