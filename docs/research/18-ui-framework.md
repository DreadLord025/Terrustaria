# 18 — UI framework (UIElement / UIState / UserInterface / layers)

> Wave 2 · researched 2026-11-12 · sources: `decompiled/Terraria/UI/*` (37 files,
> ~11 970 lines) + `decompiled/Terraria/GameContent/UI/*` (ROADMAP:933 — 170 files,
> 34 k lines) + `crates/tr-app/src/ui/` (31 files, ~25 840 lines) + ROADMAP §23.

## Scope

The module is the retained-mode UI *machinery* of vanilla, not the screens (those
are modules 19/20): `UIElement` (621 lines) — the layout pass (`Recalculate` /
`GetDimensionsBasedOnParentDimensions` / `RecalculateChildren`), the draw walk
(`Draw` → `DrawSelf` → `DrawChildren`), the hit-test (`GetElementAt`), the mouse
event chain (`LeftMouseDown`…`MouseOut`, incl. the child→parent bubble), the
`OverflowHidden` scissor, snap points, and `CalculatedStyle` (29 lines) /
`StyleDimension` (44 lines) geometry. `UIState` (13 lines) is the root element
(100% × 100%, `Recalculate` in the ctor). `UserInterface` (362 lines) is the
pointer machine: element stack (`_currentState`, `_history`), `InputPointerCache`
per button (down/up/click/double-click, 500 ms window, 200 ms state-change
lockout), hover tracking (`_lastElementHover`), `MouseCaptured`, `IsElementUnderMouse`,
and the root dimensions `PlayerInput.OriginalScreenSize / Main.UIScale`
(`UserInterface.cs:332-336`). `GameInterfaceLayer` / `LegacyGameInterfaceLayer` /
`InterfaceScaleType` are the named draw-layer stack with three scale spaces.

Rust: `crates/tr-app/src/ui/` is an **immediate-mode** kit — no retained tree, no
`UserInterface` object, no layer registry (ruling documented at
`uistate.rs:6-12`, `pointer.rs:5-16`, `framework.rs:5-17`). The shared machinery
is `uistate.rs` (871: the layout pass + vanilla colours + `DrawSplicedPanel` +
text panel + image buttons), `framework.rs` (177: the flat hit order),
`pointer.rs` (432: the pointer machine), `select_hit.rs` (36: element ids),
`cards.rs`, `panel.rs`, `text.rs`, `sliders.rs`, `color.rs`, `cursor.rs`,
`mod.rs` helpers. Per-screen halves live in the 20 screen files.

## Key entities

- `decompiled/Terraria/UI/UIElement.cs::Recalculate` (:285-304) — outer → margins →
  `_dimensions` → padding → `_innerDimensions`, then `RecalculateChildren()`.
- `UIElement.cs::GetDimensionsBasedOnParentDimensions` (:306-322) — X/Y from
  StyleDimension, W/H clamped *before* the H/VAlign shift (:315-320).
- `UIElement.cs::RecalculateChildren` (:372-378), `GetInnerDimensions` (:380),
  `CopyStyle` (:395-412).
- `UIElement.cs::Draw` (:199-238) — `DrawSelf`, then the `OverflowHidden` scissor
  around `DrawChildren` (:222-237); `GetClippingRectangle` (:252-269).
- `UIElement.cs::GetElementAt` (:324-352) — reverse child walk, first hit wins
  unless `PassThroughMouseInteraction` (:333, never set in 1.4.5.8).
- `UIElement.cs::LeftMouseDown`…`MouseOut`, `ScrollWheel` (:414-547) — each raises
  its own delegate **and then recurses into `Parent`** (bubble, :420-423 etc.).
- `UIElement.cs::SetSnapPoint`/`GetSnapPoint`/`GetSnapPoints` (:124-145, :271-283).
- `decompiled/Terraria/UI/SnapPoint.cs::Calculate` (:26-30) — anchor × dimensions + offset.
- `decompiled/Terraria/UI/UserInterface.cs::Update` (:190-228) — hover swap
  (`MouseOut` then `MouseOver`, :205-213), `HandleClick` left then right (:216-217),
  scroll wheel (:219-223), then `_currentState.Update` (:226).
- `UserInterface.cs::HandleClick` (:230-255) — press edge (down + double-click),
  release edge (click only if `lastDown.ContainsPoint`, :246).
- `UserInterface.cs::SetState`/`GoBack`/`AddToHistory`/`ResetState` (:276-322).
- `crates/tr-app/src/ui/uistate.rs::Style` (:26-46) — `StyleDimension`.
- `uistate.rs::xna_clamp` (:50-52) — `Math.Max(min, Math.Min(max, value))`.
- `uistate.rs::resolve` (:83-107) — the layout pass, pure, margins dropped.
- `uistate.rs::inner` (:111-118) — the padding tail of `Recalculate` (:298-303).
- `framework.rs::Element`/`get_element_at` (:36-80) — the flat hit order.
- `pointer.rs::PointerState::{tick,press,release,mouse_captured,reset_on_state_change}`
  (:122-227) — the `HandleClick` edges + the 500/200 ms constants (:24-30).
- `app/flow.rs::FrontScreen` + `FrontScreenBehaviours` (draw/click/key, :304-341)
  ≈ `UIState`; `flow.rs` `next_flow_state` table ≈ `UserInterface.SetState`.
- `cursor.rs::CursorState` — the vanilla cursor (`Main.DrawCursor`), not
  `UserInterface`; hover text is `tooltip.rs::draw_item_tooltip` (`mod.rs:984`).

## Dependencies

- Calls into: `tr_render::{SpriteRenderer, SheetRect}` (all draws), `assets::Lookups`
  (every sheet an `Option` so `Lookups::default()` draws nothing — `uistate.rs:11-12`),
  `lang` (text), `input_map`/`settings` (screens).
- Called by: `app/frame.rs` (HUD order `ui::draw_hud` :1358, chest :1409, death
  screen :1479, invasion :1494, achievements :1513, minimap :1255-1268) and
  `app/loop.rs` (front-end press/release dispatch, `:390-403`).
- Shared data structures: `[f32; 4]` rects everywhere (x, y, w, h) — the
  `CalculatedStyle` analogue; `u32` element ids (`select_hit.rs`) standing in for
  the C#'s object references in `InputPointerCache` (`pointer.rs:31-35`).
- C# side consumed by ~everything: `Main.DrawInterface` builds the layer list;
  every `GameContent/UI/State` is a `UIState`.

## Port status

| C# subsystem | Status | Rust location | Evidence / notes |
| --- | --- | --- | --- |
| `StyleDimension` px + percent | PORTED | `uistate.rs::Style` :26-46 | `GetValue` = px + pct·container (:17-20 of StyleDimension.cs); constructors match :24-32 |
| Clamp `MathHelper.Clamp` | PORTED | `uistate.rs::xna_clamp` :50-52 | min-wins when min>max; test `xna_clamp_keeps_the_min_when_min_exceeds_max` |
| Layout math `:308-320` | PORTED | `uistate.rs::resolve` :83-107 | min/max clamp before alignment (:99-101); tests `resolve_*` |
| Margins `:293-297`, `:317-318` | MISSING | `uistate.rs:54-58,105` | dropped: every P6 caller leaves them 0, so they cancel; latent divergence |
| Padding / inner rect `:298-303` | PORTED | `uistate.rs::inner` :111-118 | test `resolve_padding_shrinks_the_inner_rect_only` |
| Parent = `UIList` ⇒ height ∞ `:288-291` | MISSING | `uistate.rs:73-75` | admitted in code; list rows derive arithmetically per screen |
| Root dims `UserInterface.cs:332-336` | PARTIAL | per-screen root rects (`char_select::select_player` etc.) | no `Main.UIScale` field; UI scale is pinned 1.0 (`ingame_options.rs:733-734`, `entities.rs:1539`) |
| `RecalculateChildren` | MISSING (n/a) | — | no retained tree; each screen re-derives its own rects per frame |
| Hit test `GetElementAt` `:324-352` | PARTIAL | `framework.rs::get_element_at` :64-80 | flat only; recursion (:341) a no-op on flat lists; `PassThroughMouseInteraction` (:333) unmodelled (never set) |
| `ContainsPoint` `:354-357` | PORTED | `framework.rs::contains_point` :51-55 → `mod.rs::in_rect` :54 | Rectangle.Contains edges inclusive-left/exclusive-right; tests `dw_2_1_the_edges_*` |
| Press/release/click/double-click, 500 ms, 200 ms | PORTED | `pointer.rs` :122-227 | 12 tests `dw_1_1..dw_1_3`; double-click needs same element, click needs release inside |
| Hover swap `Update` :205-213 | PARTIAL | per-screen `update_hover` (`loop.rs:377`) + `pointer.rs` | no `_lastElementHover` on the machine; no `IsMouseHovering` on ancestors |
| Event bubble child→parent `:414-547` | MISSING | `pointer.rs:154-224` | events raised on the hit element only; no parent walk |
| Scroll wheel `Update` :219-223 | PARTIAL | per-screen counters (`chest.rs`, `emotes.rs`, `achievements.rs`) | not part of `pointer.rs`; no `UIScrollWheelEvent` type |
| `MouseCaptured` :113-124 | PORTED | `pointer.rs::mouse_captured` :226 | `left_held` read by `ingame_options` (`frame.rs:117`) |
| `ResetState`/200 ms lockout :165-179 | PORTED | `pointer.rs::reset_on_state_change` :142 | test `dw_1_2_the_state_change_disable_swallows_the_next_press` |
| `OverflowHidden` scissor :199-238 | MISSING | `char_select.rs:856`, `world_select.rs:818`, `char_creation.rs:1859` | admitted: "the port has no scissor, so they are skipped" — rows break-gated out of draw *and* hit |
| `GetClippingRectangle` :252-269 | MISSING | — | same root cause |
| Snap points `:124-145, :271-283`, `SnapPoint.cs` | MISSING | — | gamepad navigation absent |
| Gamepad / `UILinkPointNavigator` (`UserInterface.cs:184-187`) | MISSING | — | NoGamepadSupport / gamepad branch unported |
| `Activate`/`Deactivate`/`Initialize` :548-605 | MISSING | — | immediate mode; screens carry their own reset |
| State stack `_history`/`GoBack` :305-322 | MISSING | `flow.rs` `next_flow_state` | code comment (`pointer.rs:8-10`) claims zero call sites in 1.4.5.8 — **unverified** here |
| `GameInterfaceLayer` / `InterfaceScaleType` | MISSING | fixed order in `frame.rs` | no named layer list, no per-layer zoom matrix (`Game`/`UI`/`None`) |
| `UIPanel` 9-slice / separators / scrollbar sprite | PORTED | `panel.rs` :1-21 (`CORNER`/`BAR` constants) | draw-only; thumb drag per screen |
| `UITextPanel` / `UIImageButton` / `UIColoredImageButton` | PORTED | `uistate.rs:290-527` | tests `text_panel_rect_*`, `lookups_default_*` |
| `Utils.DrawSplicedPanel` | PORTED | `uistate.rs::spliced_rects` :168-219 | incl. the `:2593` top-right quirk; 4 tests |

## Gap list — C# UI classes with no Rust counterpart

The centrepiece. "Counterpart" means a port of *that class*, not a lookalike.
Screens already covered by immediate ports are listed for completeness at the end.

### Framework-level (no retained equivalents)

| C# class (file) | Provides | Rust counterpart | Status | Impact |
| --- | --- | --- | --- | --- |
| `UIElement` fields: margins, `MaxWidth`/`MaxHeight`/`Min*`, `OverrideSamplerState`, `UseImmediateMode`, `IgnoresMouseInteraction`(partial), `PassThroughMouseInteraction` | layout + draw modifiers | `uistate.rs` covers px/pct/H/VAlign/padding only | PARTIAL | behavioural if a future element sets them |
| `UIElement.Recalculate/RecalculateChildren` as tree ops | whole-tree relayout | per-screen per-frame derivation | MISSING (by design) | the `UIList` ∞-height case diverges |
| `UIElement` event bubble (`:414-547`) | ancestor handlers + ancestor `IsMouseHovering` | none | MISSING | behavioural (containers that react to descendant clicks/hover) |
| `UIElement.Draw` `OverflowHidden` path + `GetClippingRectangle` | clipping | none (break-gate) | MISSING | pixel: a straddling row is fully drawn or absent instead of half-clipped |
| `UIElement.GetSnapPoints`, `SetSnapPoint` ; `SnapPoint.cs` | gamepad nav anchors | none | MISSING | gamepad only |
| `UIElement.Activate/Deactivate/Initialize/ExecuteRecursively/CopyStyle` | lifecycle | none | MISSING | none today (immediate screens) |
| `UserInterface` (object, `_currentState`, `_history`) | element stack, `GoBack`, 32-entry history | `flow.rs` table | MISSING/partial | `GoBack` semantics (2-state pop) not modelled |
| `GameInterfaceLayer`, `LegacyGameInterfaceLayer`, `InterfaceScaleType` | named layer stack, 3 scale spaces | fixed order in `frame.rs` | MISSING | ordering is hard-coded; no per-layer zoom |
| `UIState` (as a class you instantiate) | 100% root + `NoGamepadSupport` | `FrontScreen` / per-screen root | PARTIAL | no shared root type |

### Element classes (`Terraria/UI/*.cs` + `GameContent/UI/Elements/*.cs`)

| C# class (file:line) | What it is | Rust counterpart | Status |
| --- | --- | --- | --- |
| `UIList` + `UIInnerList` (`UIList.cs:10,14`) | scrolling viewport with ∞ height + scrolling | per-screen row maths (`char_select`, `achievements`, `emotes`, `chest`) | MISSING as a class |
| `UIScrollbar` | thumb track/drag | static sprite only (`char_creation.rs:1860` "never scrolls") | PARTIAL |
| `UISliderBase`, `UIColoredSliderSimple` | slider input element | `sliders.rs` draw-only | PARTIAL (no drag element) |
| `UISearchBar`, `UIWrappedSearchBar` | bestiary/creative filter input | none | MISSING |
| `UIProgressBar`, `UIGenProgressBar` | progress elements | `bars.rs` (boss bar, bespoke) | PARTIAL/MISSING for the generic element |
| `UIHeader`, `UIHorizontalSeparator`, `UIVerticalSeparator` | chrome | `panel.rs` separators (9-slice primitives) | PARTIAL |
| `UISlicedImage`, `UISlicedImage` variants, `UIImageFramed` | sliced images | `uistate::spliced_rects` primitive | PARTIAL (helper, not element) |
| `UIToggleImage`, `UIImage`, `UIText` | basic draw elements | `text::push_font_text`, `panel` | PARTIAL |
| `UISortableElement` | drag-reorder | none | MISSING |
| `UIDynamicItemCollection` (+subclasses) | item grids | `slot.rs` per-context draw | PARTIAL |
| `UIItemSlot`, `UIItemIcon` | item slot *elements* | `slot.rs::draw` (ports `ItemSlot.Draw` directly, no element) | PARTIAL |
| `UIParticleLayer` | UI particles | none | MISSING |
| `PowerStripUIElement` | power-button strip | none | MISSING |
| `UICreativePowerButton`, `UIDifficultyButton`, `UIClothStyleButton`, `UICharacterNameButton`, `UIIconTextButton`, `UIDebugCommandItem`, `UIDebugLoadoutItem` | composite buttons | none | MISSING |
| `UIKeybindingListItem`, `-Cycle-`, `-Simple-`, `-Slider-`, `-ToggleListItem` | controls screen rows | none | MISSING |
| `UIBestiaryEntryButton/Grid/Icon/InfoPage/SortingOptionsGrid/FilteringOptionsGrid/NPCEntryPortrait/InfoItemLine` | bestiary elements | `bestiary.rs` (immediate) | PARTIAL |
| `UIAchievementListItem`, `EmoteButton`, `EmotesGroupListItem`, `UICharacterListItem`, `AWorldListItem`, `UICharacter`, `UIWorldCreationPreview`, `UIResourcePack`, `UIWorkshopPublishResourcePackListItem` | list rows/cards | `achievements.rs`, `emotes.rs`, `cards.rs` + the select screens (immediate) | PARTIAL |
| `IngameFancyUI` (open helpers), `IngameUIWindows`, `InGameNotificationsTracker`, `InGamePopups`, `GameTipsDisplay` | window open/close tracker, notifications | `frame.rs` open flags; `mod.rs::draw_hud` | PARTIAL |
| `FancyErrorPrinter`, `TooltipProcessor`, `ItemTooltip`, `INetDiagnosticsUI`, `NetDiagnosticsUI`, `EmptyDiagnosticsUI`, `IInGameNotification`, `IPipsUI`, `ICraftingUI`, `IHaveBackButtonCommand` | small interfaces/utilities | none / partial | MISSING |

### `UIState` subclasses with no Rust screen at all

| C# state (file) | Lines | Rust counterpart | Notes |
| --- | --- | --- | --- |
| `UIWorkshopHub` | 13 | none | Steam Workshop hub |
| `UIWorkshopSelectResourcePackToPublish` | 17 | none | |
| `UIWorkshopSelectWorldToPublish` | 16 | none | |
| `UIWorkshopWorldImport` | 18 | none | |
| `UIResourcePackSelectionMenu` | 17 | none | |
| `UIResourcePackInfoMenu` | 15 | none | |
| `UIDebugCommandsList` | 15 | none | debug |
| `UIDebugLoadoutsList` | 14 | none | debug |
| `UIDebugLoadoutItem`-family screens | — | none | debug |
| `UIReportsPage` | 16 | none | |
| `UIWorldCreationAdvanced` | 16 | none | ROADMAP:953 lists it; world_creation.rs covers the base screen only |
| `UIWorldCreationAdvancedSecretSeedsList` | 17 | none | same |
| `UIWorldGenDebug` | 27 | none | debug |
| `UICreativePowersMenu` | 13 | none | journey-mode powers |
| `UITextWrappingTest` | 14 | none | test-only screen |
| `UIManageControls` | 20 | none | keybinding screen; `input_map.rs` has the data, not the screen |
| `UIWorldLoad` | 16 | `world_select.rs`/`save_list.rs` (folded in) | PARTIAL — verify no screen is lost |
| `NewCraftingUI` | 21 | `crafting.rs` | PORTED (immediate) |
| `UICharacterSelect/UICharacterCreation/UIWorldSelect/UIWorldCreation/UIVirtualKeyboard/UIAchievementsMenu/UIBestiaryTest/UIEmotesMenu` | — | `char_*`, `world_*`, `keyboard.rs`, `achievements.rs`, `bestiary.rs`, `emotes.rs` | PORTED |

## Tests

`grep "#\[test\]"` in `crates/tr-app/src/ui/` — 325 attributes, 291 named test
fns across 31 files (framework-relevant subset):

- `ui/framework.rs` (5): `dw_2_1_back_to_front_the_last_appended_element_wins`
  (`GetElementAt` :327-333), `dw_2_1_ignores_mouse_is_invisible_to_the_hit_but_still_addressable`
  (:330), `dw_2_1_a_zero_sized_rect_is_never_a_hit`, `dw_2_1_the_edges_follow_rectangle_contains`.
- `ui/pointer.rs` (12): `dw_1_1_press_fires_mouse_down_and_a_release_inside_fires_click`,
  `dw_1_1_a_second_press_within_500ms_is_a_double_click_and_clears_last_clicked`,
  `dw_1_1_the_same_pair_past_500ms_is_click_then_down`, `dw_1_1_double_click_requires_the_same_element`,
  `dw_1_2_release_outside_the_down_element_fires_mouse_up_and_no_click`,
  `dw_1_2_the_state_change_disable_swallows_the_next_press`,
  `dw_1_2_the_right_button_behaves_like_the_left`, `dw_1_3_reset_state_clears_a_stale_was_down`,
  `dw_1_3_the_disable_countdown_decays_to_zero_and_clamps`.
- `ui/uistate.rs` (12): `resolve_lands_the_select_screen_children`,
  `resolve_clamps_min_width_before_aligning`, `resolve_padding_shrinks_the_inner_rect_only`,
  `spliced_rects_*` (4, incl. the :2593 quirk and the 44×44 sheet map),
  `colour_table_matches_the_vanilla_literals`, `text_panel_rect_applies_the_settext_minimums_to_the_hint_box`,
  `xna_clamp_keeps_the_min_when_min_exceeds_max`, `lookups_default_keeps_every_p6_sheet_none`.
- Screens: `char_select` 17 (`dw_2_3_a_double_click_on_a_card_body_yields_double_click_play`,
  `dw_2_3_the_fat_buttons_do_not_double_click_into_play`), `world_select` 17 (same pair),
  `chest` 13 (`dw_3_2_*` scroll math + `dw_3_2_scroll_wheel_gate_matches_doscrollingininventory`),
  `emotes` 6, `ingame_options` 27, `char_creation` 23, `keyboard` 13.
- No test covers: margins, the `UIList` ∞-height case, ancestor bubbling,
  `OverflowHidden` clipping, snap points, or the layer stack.

## Deviations & risks

1. **Margins silently dropped** — `uistate.rs:54-58,105` documents that margins
   cancel at 0 and omits them. All current callers pass 0, so behaviour matches
   today, but any new element with a margin diverges invisibly. Behavioural,
   latent; admitted in code, not in ROADMAP.
2. **`MaxHeight` clamp uses `f32::MAX`** (`uistate.rs:101`) instead of
   `MaxHeight.GetValue(parentH)` (`UIElement.cs:314,316`). Same as the `Fill`
   default; a percent-valued `MaxHeight` cannot be expressed. Behavioural, latent.
3. **`UIList` infinite-height parent special case not modelled**
   (`uistate.rs:73-75`, C# `UIElement.cs:288-291`). Percent-height children of a
   list resolve against the list rect instead of ∞. The screens derive rows
   arithmetically, so it is untested by the current callers. Behavioural; admitted
   in code, absent from ROADMAP §23 (which *does* list it at :937-938 — so ROADMAP
   is ahead of the port here).
4. **No child→parent event bubble** (`UIElement.cs:414-547` vs `pointer.rs:154-224`).
   Ancestor `On_*` handlers never fire; ancestor `IsMouseHovering` is never set.
   Behavioural. ROADMAP:938-939 claims the bubble as a ported fact — the claim is
   wrong for the Rust side; the framework-completion task (M7-UI-1, ROADMAP:964)
   is still open. Also unmodelled: the C#'s Out-then-Over double-toggle when the
   pointer moves child→parent inside one frame (`UserInterface.cs:205-213`).
5. **No scissor / `OverflowHidden`** — rows past a list bottom are *skipped*
   rather than clipped (`char_select.rs:856`, `world_select.rs:818`,
   `char_creation.rs:1859`). A row straddling the edge draws whole (C#: half);
   a fully past-edge row that C# still hit-tests (`GetElementAt` does not cull
   clipped children, `char_creation.rs:1861-1863` notes this) is unhittable in the
   port because the same gate prunes the element list. Pixel + behavioural;
   admitted per-screen, not in ROADMAP.
6. **UI scale pinned to 1.0** — `Main.UIScale`/`GameUI.UIScale` have no port field
   (`ingame_options.rs:733-734` read-only bar, `entities.rs:1539` passes 1.0), so
   `UserInterface.GetDimensions()` (`:332-336`) and `GameInterfaceLayer`'s three
   scale spaces cannot be exercised. Behavioural; admitted.
7. **Scroll wheel not in the pointer machine** — `UserInterface.cs:219-223` runs
   scroll after the two buttons with the delta consumed once; the port has
   per-screen counters and no shared `UIScrollWheelEvent`. Order across screens
   differs. Behavioural, low.
8. **Frame ordering** — C# `Update` is hover → left click → right click → scroll →
   state update (`UserInterface.cs:190-228`); the front-end screens are
   event-driven from `loop.rs` (`:390-403`), so the press dispatch may run before
   or after a hover refresh in the same frame. Determinism-relevant for
   double-click edges. Not verified; no test pins it.
9. **`GoBack`/history "dead code" claim unverified** — `pointer.rs:8-10` states
   zero call sites in 1.4.5.8. `UserInterface.GoBack` (:305-313) *is* called from
   the back-button paths of `IHaveBackButtonCommand` screens; the port reimplements
   back as a `next_flow_state` table entry per screen instead. Behavioural if a
   screen relies on the 2-deep history pop.
10. **Snap points / gamepad** absent — no `UILinkPointNavigator`, no anchor maths
    (`SnapPoint.cs:26-30`), no `NoGamepadSupport` handling (`UIState.cs:5`).
    Behavioural (gamepad input unsupported), expected for Stage 1.

## Task seeds

- **S — Pin the event order.** Add a `ui/pointer.rs` test that reproduces
  `UserInterface.cs:190-228`'s order (hover swap before clicks, left before right)
  against the front-end dispatch in `app/loop.rs:390-403`. Depends on nothing.
- **S — Verify the `GoBack` dead-code claim.** Grep `decompiled` for
  `UserInterface.GoBack` / `.GoBack()` call sites and record the result in
  `pointer.rs:8-10`; if non-empty, add a `flow.rs` history seed. (S)
- **M — Port margins into `uistate::resolve`.** Add `margin: [f32;4]` to the
  signature and reproduce `:293-297`/`:317-318` (add then subtract), with a test
  against a C# hand-computed case. Closes deviation 1 for future callers.
- **M — Model the `UIList` ∞-height parent.** A `list_parent: bool` flag on
  `resolve` that sets `parent[3] = f32::MAX` (`UIElement.cs:288-291`), plus a test
  on a percent-height child. Unblocks the scrolling screens of module 20.
- **M — Ancestor bubble.** Give `pointer.rs` an optional parent chain (or a
  screen-level "container handler" table) so a hit element's event also raises on
  its ancestors, and add ancestor `IsMouseHovering`. Closes deviation 4 for
  hover; confirm against a real C# handler before generalising.
- **L — Clipping.** Implement a scissor/rect-clip in `tr_render` and wire
  `OverflowHidden` into the list screens, replacing the break-gates in
  `char_select.rs:856` / `world_select.rs:818` / `char_creation.rs:1859`.
  Depends on a renderer change; pixel-verify with `TR_SHOT`.
- **M — `UIWorldCreationAdvanced` + secret-seeds list** (ROADMAP:953): a
  `ui/world_creation_advanced.rs` immediate port; unblocks seed selection.
  Depends on the `UIList` seed above for its option rows.
- **L — Screens with no counterpart:** `UIManageControls` (20 lines C#, has
  `input_map.rs` data behind it) and `UICreativePowersMenu`. `UIWorkshop*` /
  `UIResourcePack*` should be scoped out (Steam-only) and recorded in module 38.
- **L — Layer stack.** Introduce a named ordered layer list
  (`GameInterfaceLayer` analogue) in `app/frame.rs` so draw order is data, not
  code, with `InterfaceScaleType` spaces. Prerequisite for any screen that must
  draw between existing HUD layers.
