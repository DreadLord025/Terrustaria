# 18 — UI framework (UIElement / UIState / UserInterface / layers)

> Wave 2 · researched 2026-11-12 · sources: `decompiled/Terraria/UI/*` (37 files,
> ~11 970 lines) + `decompiled/Terraria/GameContent/UI/*` (ROADMAP:933 — 170 files,
> 34 k lines) + `crates/tr-app/src/ui/` (31 files, ~25 840 lines) + ROADMAP §23.
>
> **ALL-ROWS STATUS REFRESH 2026-11-21 (W4-d85 FINAL phase)** — the
> framework rows re-verified against the branch tips (d87 `38779a1d2`
> margins/snap/state-stack/layers, d88 `d2279e811` UIList/scissor, d85
> `23fe0d4e5`). The **FINAL 28-screen coverage table** is research/112
> §5 (this doc's gap table below keeps the historical rows; the
> refresh marks are inline). f22's research/144 declares the family
> COMPLETE; this refresh independently re-verifies it.

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
| Margins `:293-297`, `:317-318` | ~~MISSING~~ **PORTED (REFRESH: d87 `75009f973`)** | `uistate::resolve_with_margins` | inflate-before-align, shrink-after; `resolve` delegates at 0 margins bit-identically |
| Padding / inner rect `:298-303` | PORTED | `uistate.rs::inner` :111-118 | test `resolve_padding_shrinks_the_inner_rect_only` |
| Parent = `UIList` ⇒ height ∞ `:288-291` | MISSING | `uistate.rs:73-75` | admitted in code; list rows derive arithmetically per screen |
| Root dims `UserInterface.cs:332-336` | PARTIAL | per-screen root rects (`char_select::select_player` etc.) | `Main.UIScale` ported (`settings::ui_scale_max`, `Settings::ui_scale`, the `GameUI.UIScale` bar) but the `GetDimensions()` division is still not applied to the per-screen rects (S1-H-3; docs/ANALYSIS.md) |
| `RecalculateChildren` | MISSING (n/a) | — | no retained tree; each screen re-derives its own rects per frame |
| Hit test `GetElementAt` `:324-352` | PARTIAL | `framework.rs::get_element_at` :64-80 | flat only; recursion (:341) a no-op on flat lists; `PassThroughMouseInteraction` (:333) unmodelled (never set) |
| `ContainsPoint` `:354-357` | PORTED | `framework.rs::contains_point` :51-55 → `mod.rs::in_rect` :54 | Rectangle.Contains edges inclusive-left/exclusive-right; tests `dw_2_1_the_edges_*` |
| Press/release/click/double-click, 500 ms, 200 ms | PORTED | `pointer.rs` :122-227 | 12 tests `dw_1_1..dw_1_3`; double-click needs same element, click needs release inside |
| Hover swap `Update` :205-213 | PARTIAL | per-screen `update_hover` (`loop.rs:377`) + `pointer.rs` | no `_lastElementHover` on the machine; no `IsMouseHovering` on ancestors |
| Event bubble child→parent `:414-547` | PORTED | `framework.rs::bubble_chain` :77-113 + `Element.parent`/`Element::child` :30-84 | the `:420-423` recursion as a parent-chain walk; wired to the select screens' card buttons (`char_select.rs`/`world_select.rs`) |
| Scroll wheel `Update` :219-223 | PORTED | `pointer.rs::queue_scroll` / `scroll_wheel` :238-262 → the per-screen consumers (`chest.rs`, `emotes.rs`, `achievements.rs`) | the machine owns `PlayerInput.ScrollWheelDeltaForUI` (accumulate + consume once); `PointerEvent::ScrollWheel { delta }` is `UIScrollWheelEvent.ScrollWheelValue` (`UIScrollWheelEvent.cs:7`); tests `dw_1_4_*` |
| `MouseCaptured` :113-124 | PORTED | `pointer.rs::mouse_captured` :226 | `left_held` read by `ingame_options` (`frame.rs:117`) |
| `ResetState`/200 ms lockout :165-179 | PORTED | `pointer.rs::reset_on_state_change` :142 | test `dw_1_2_the_state_change_disable_swallows_the_next_press` |
| `OverflowHidden` scissor :199-238 | PORTED | `tr-render/src/clip.rs` + `SpriteRenderer::push_clip`/`pop_clip`; wired in `char_select.rs`, `world_select.rs`, `char_creation.rs` (`draw_hairstyles_menu`) | the clip is a per-instance `clip` vec4 + `discard` in `sprite.wgsl` (one instanced draw); nested pushes intersect as `GetClippingRectangle` does. S1-H-2 |
| `GetClippingRectangle` :252-269 | PORTED | `clip.rs::intersect`/`ClipStack` + `uistate::inner` at the call sites | reads the element's **inner** rect (`:254`), which is what the screens push; S1-H-2 |
| Snap points `:124-145, :271-283`, `SnapPoint.cs` | ~~MISSING~~ **PORTED (REFRESH: d87 `164cb7090`)** | `ui/pointer.rs` (the snap-point maths) | `SnapPoint.Calculate` anchor × dimensions + offset; the gamepad NAVIGATOR stays absent (the row below) |
| Gamepad / `UILinkPointNavigator` (`UserInterface.cs:184-187`) | MISSING | — | NoGamepadSupport / gamepad branch unported (Stage-1 exclusion, unchanged) |
| `Activate`/`Deactivate`/`Initialize` :548-605 | MISSING | — | immediate mode; screens carry their own reset (unchanged, by design) |
| State stack `_history`/`GoBack` :305-322 | ~~MISSING~~ **PORTED (REFRESH: d87 `164cb7090`)** | `ui/pointer.rs` (SetState/GoBack/AddToHistory, the 33→29 history sawtooth) | the dead-code claim **VERIFIED 2026-11-21** (W4-d85 doc-sweep: all 8 `.GoBack()` hits are UIState-own methods; `_history` is write-only in 1.4.5.8) — d87 ports the machinery anyway for the seam's completeness |
| `GameInterfaceLayer` / `InterfaceScaleType` | ~~MISSING~~ **PARTIAL (REFRESH: d87 `164cb7090`)** | the 43-layer `Main.cs:44718-44918` table + the stop-on-false walk | the named layer list + the three scale spaces landed as data; the per-layer zoom matrix application stays the follow-up (d87's own PARTIAL mark) |
| `UIPanel` 9-slice / separators / scrollbar sprite | PORTED | `panel.rs` :1-21 (`CORNER`/`BAR` constants) | draw-only; thumb drag per screen |
| `UITextPanel` / `UIImageButton` / `UIColoredImageButton` | PORTED | `uistate.rs:290-527` | tests `text_panel_rect_*`, `lookups_default_*` |
| `Utils.DrawSplicedPanel` | PORTED | `uistate.rs::spliced_rects` :168-219 | incl. the `:2593` top-right quirk; 4 tests |

## Gap list — C# UI classes with no Rust counterpart

The centrepiece. "Counterpart" means a port of *that class*, not a lookalike.
Screens already covered by immediate ports are listed for completeness at the end.

### Framework-level (no retained equivalents)

| C# class (file) | Provides | Rust counterpart | Status | Impact |
| --- | --- | --- | --- | --- |
| `UIElement` fields: margins, `MaxWidth`/`MaxHeight`/`Min*`, `OverrideSamplerState`, `UseImmediateMode`, `IgnoresMouseInteraction`(partial), `PassThroughMouseInteraction` | layout + draw modifiers | `uistate.rs` px/pct/H/VAlign/padding + d87's margins (`75009f973`) | PARTIAL | margins closed (REFRESH); the sampler/immediate-mode flags stay unmodelled (no caller sets them) |
| `UIElement.Recalculate/RecalculateChildren` as tree ops | whole-tree relayout | per-screen per-frame derivation | MISSING (by design) | the `UIList` ∞-height case is now modelled (`resolve_in_list`, S1-H-2) |
| `UIElement` event bubble (`:414-547`) | ancestor handlers + ancestor `IsMouseHovering` | `framework.rs::bubble_chain` + `Element` parent links | PORTED (S1-H-1) | the `MouseOver`-driven ancestor hovering is the same chain (`:510-521`) |
| `UIElement.Draw` `OverflowHidden` path + `GetClippingRectangle` | clipping | `tr-render/src/clip.rs` + `SpriteRenderer::push_clip`/`pop_clip` + `Element.viewport` for the hit side | PORTED (S1-H-2) | the clip cuts pixels in the fragment stage; hit-testing stops descending through an ancestor that does not contain the point (`UIElement.cs:327-341`) |
| `UIElement.GetSnapPoints`, `SetSnapPoint` ; `SnapPoint.cs` | gamepad nav anchors | d87's snap-point port (`164cb7090`) | ~~MISSING~~ **PORTED (REFRESH: d87)** | the anchor maths; the navigator stays gamepad-only absent |
| `UIElement.Activate/Deactivate/Initialize/ExecuteRecursively/CopyStyle` | lifecycle | none | MISSING | none today (immediate screens) |
| `UserInterface` (object, `_currentState`, `_history`) | element stack, `GoBack`, 32-entry history | `flow.rs` table + d87's state-stack port (`164cb7090`) | ~~MISSING/partial~~ **PORTED (REFRESH: d87)** | the 33→29 sawtooth + `SetState`/`GoBack`/`AddToHistory` landed; `GoBack` has zero callers in 1.4.5.8 (verified `ef9c8954`) |
| `GameInterfaceLayer`, `LegacyGameInterfaceLayer`, `InterfaceScaleType` | named layer stack, 3 scale spaces | d87's 43-layer table + stop-on-false walk (`164cb7090`) | ~~MISSING~~ **PARTIAL (REFRESH: d87)** | the layer list is data now; the per-layer zoom application stays the follow-up |
| `UIState` (as a class you instantiate) | 100% root + `NoGamepadSupport` | `FrontScreen` / per-screen root | PARTIAL | no shared root type (by design, immediate mode) |

### Element classes (`Terraria/UI/*.cs` + `GameContent/UI/Elements/*.cs`)

| C# class (file:line) | What it is | Rust counterpart | Status |
| --- | --- | --- | --- |
| `UIList` + `UIInnerList` (`UIList.cs:10,14`) | scrolling viewport with ∞ height + scrolling | `uistate::resolve_in_list` + `list_total_height` (d88 `d2279e811`: the verbatim `RecalculateChildren` inner-height sum, the raw-counter wheel, the `SetView` clamp, the AABB cull) + per-screen row maths | ~~MISSING as a class~~ **PORTED as machinery (REFRESH: d88)** — the ∞-height case S1-H-2, the scroll/scissor consumers d88; still not a retained class (by design) |
| `UIScrollbar` | thumb track/drag | `panel::draw_scrollbar` + the per-screen drag (`front_scrollbar_press`/`drag`, the reports/seeds-list attach-on-overflow) | PARTIAL (the drag is per-screen, not an element — unchanged) |
| `UISliderBase`, `UIColoredSliderSimple` | slider input element | `sliders.rs` draw-only | PARTIAL (no drag element) |
| `UISearchBar`, `UIWrappedSearchBar` | bestiary/creative filter input | none | MISSING |
| `UIProgressBar`, `UIGenProgressBar` | progress elements | `bars.rs` (boss bar, bespoke) | PARTIAL/MISSING for the generic element |
| `UIHeader`, `UIHorizontalSeparator`, `UIVerticalSeparator` | chrome | `panel.rs` separators (9-slice primitives) | PARTIAL |
| `UISlicedImage`, `UISlicedImage` variants, `UIImageFramed` | sliced images | `uistate::spliced_rects` primitive | PARTIAL (helper, not element) |
| `UIToggleImage`, `UIImage`, `UIText` | basic draw elements | `text::push_font_text`, `panel` | PARTIAL |
| `UISortableElement` | drag-reorder | `ui/sortable.rs` (d83 `17799c642` tip) | ~~MISSING~~ **PORTED on d83 (REFRESH)** |
| `UIDynamicItemCollection` (+subclasses) | item grids | `slot.rs` per-context draw | PARTIAL |
| `UIItemSlot`, `UIItemIcon` | item slot *elements* | `slot.rs::draw` (ports `ItemSlot.Draw` directly, no element) | PARTIAL |
| `UIParticleLayer` | UI particles | none | MISSING |
| `PowerStripUIElement` | power-button strip | none | MISSING |
| `UICreativePowerButton`, `UIDifficultyButton`, `UIClothStyleButton`, `UICharacterNameButton`, `UIIconTextButton`, `UIDebugCommandItem`, `UIDebugLoadoutItem` | composite buttons | d82/d83 element halves (`UICreativePowerButton` empty marker d82 `484c25291`; `UIIconTextButton` d84's `workshop_kit.rs`; the debug row pair d85; `UICharacterNameButton`'s trim d85 `9b8cd40d`) | ~~MISSING~~ **PORTED across d82/d83/d84/d85 (REFRESH)** — the per-element residual pins ride the f23 census |
| `UIKeybindingListItem`, `-Cycle-`, `-Simple-`, `-Slider-`, `-ToggleListItem` | controls screen rows | `ui/keybinding_rows.rs` (d83, per d86's 18c note) | ~~MISSING~~ **PORTED on d83 (REFRESH: 18c's row-40-44 note)** |
| `UIBestiaryEntryButton/Grid/Icon/InfoPage/SortingOptionsGrid/FilteringOptionsGrid/NPCEntryPortrait/InfoItemLine` | bestiary elements | `bestiary.rs` (immediate) | PARTIAL |
| `UIAchievementListItem`, `EmoteButton`, `EmotesGroupListItem`, `UICharacterListItem`, `AWorldListItem`, `UICharacter`, `UIWorldCreationPreview`, `UIResourcePack`, `UIWorkshopPublishResourcePackListItem` | list rows/cards | `achievements.rs`, `emotes.rs`, `cards.rs` + the select screens (immediate) | PARTIAL |
| `IngameFancyUI` (open helpers), `IngameUIWindows`, `InGameNotificationsTracker`, `InGamePopups`, `GameTipsDisplay` | window open/close tracker, notifications | `frame.rs` open flags; `mod.rs::draw_hud` | PARTIAL |
| `FancyErrorPrinter`, `TooltipProcessor`, `ItemTooltip`, `INetDiagnosticsUI`, `NetDiagnosticsUI`, `EmptyDiagnosticsUI`, `IInGameNotification`, `IPipsUI`, `ICraftingUI`, `IHaveBackButtonCommand` | small interfaces/utilities | none / partial | MISSING |

### `UIState` subclasses with no Rust screen at all

**Wave-4 split (2026-11-20, branches `task/W4-d84/d85/d86-ui-screens-*`, table order):**
d84 = first 6 (the Workshop/ResourcePack cluster: `UIWorkshopHub`,
`UIWorkshopSelectResourcePackToPublish`, `UIWorkshopSelectWorldToPublish`,
`UIWorkshopWorldImport`, `UIResourcePackSelectionMenu`, `UIResourcePackInfoMenu`).
**d85 = second 6 (table rows 7-12)**: `UIDebugCommandsList`,
`UIDebugLoadoutsList`, the `UIDebugLoadoutItem`-family row (both debug lists
plus their row elements `UIDebugCommandItem`/`UIDebugLoadoutItem`),
`UIReportsPage`, `UIWorldCreationAdvanced`,
`UIWorldCreationAdvancedSecretSeedsList`.
d86 = the rest (`UIWorldGenDebug`, `UITextWrappingTest`, `UIManageControls`,
the `AWorkshopPublishInfoState` pair if not folded into d84).
`UICreativePowersMenu` is Journey-mode (c24 lane) - skipped by all three.

**REFRESH 2026-11-21 (W4-d85 FINAL)**: the "Lines" column below held
class-declaration line numbers, not lengths — the raw-file counts are
in research/112 §5's final table (f22's 18d established the correction).
Every `none` row in the d84/d86/c24 ranges is now LANDED on its tip;
the rows keep their historical text with the refresh verdict appended.

| C# state (file) | Lines | Rust counterpart | Notes |
| --- | --- | --- | --- |
| `UIWorkshopHub` | 13 | none | Steam Workshop hub — d84 claim; EXCLUDED-BY-DESIGN fallback (research/18c, ANALYSIS "Not ported and not planned") |
| `UIWorkshopSelectResourcePackToPublish` | 17 | none | d84 claim; same Workshop exclusion fallback |
| `UIWorkshopSelectWorldToPublish` | 16 | none | d84 claim; same Workshop exclusion fallback |
| `UIWorkshopWorldImport` | 18 | none | d84 claim; same Workshop exclusion fallback |
| `UIResourcePackSelectionMenu` | 17 | none | d84 claim; MISSING-with-dependency fallback (no AssetSourceController/ResourcePackList port — research/18c) |
| `UIResourcePackInfoMenu` | 15 | none | same |
| `UIDebugCommandsList` | 15 | none | debug — d85 claim (research/18c reconciled split) |
| `UIDebugLoadoutsList` | 14 | none | debug — d85 claim |
| `UIDebugLoadoutItem`-family screens | — | none | debug — d85 claim (row elements: d82 `ui/debug_items.rs`) |
| `UIReportsPage` | 16 | none | d85 claim (+ the IProvideReports data family) |
| `UIWorldCreationAdvanced` | 16 | none | ROADMAP:953 lists it; world_creation.rs covers the base screen only — d85 claim |
| `UIWorldCreationAdvancedSecretSeedsList` | 17 | none | same — d85 claim |
| `UIWorldGenDebug` | 27 | none | debug — EXCLUDED (dev-only worldgen inspector, Testing/** subsystem; research/18c scope-outs) |
| `UICreativePowersMenu` | 13 | none | journey-mode powers — c24 lane |
| `UITextWrappingTest` | 14 | none | test-only screen — EXCLUDED (dev text-wrap harness; closes research/38-7's undocumented absence, research/18c) |
| `UIManageControls` | 20 | `ui/manage_controls.rs` + `input_profiles.rs` (d86) | PORTED (d86) — screen state machine + the PlayerInputProfile data; rows: d83 `ui/keybinding_rows.rs`; `input_map.rs` keeps the 14+6 bind DATA for the input pass |
| `AWorkshopPublishInfoState` (+ForResourcePack/ForWorld) | — | `ui/workshop_publish_info.rs` (d86) | data/state half PORTED (d86); Steam I/O excluded with Social/** (research/18c pair decision) |
| `UIWorldLoad` | 16 | `ui/world_load.rs` (first-class `FrontScreen::WorldLoad`, S1-E-17) | PORTED — re-verified against UIWorldLoad.cs:16-137 (research/18c); nothing was folded into world_select/save_list; deferrals recorded (drunk Main.rand arms, gameTips, F5 debug scope-out, gamepad squiggle) |
| `UIWorkshopHub` | 13 | `ui/workshop_hub.rs` (d84 `ba7e8f94a`) | **PORTED (REFRESH)** — the no-Steam hub view; the Steam I/O stays excluded |
| `UIWorkshopSelectResourcePackToPublish` | 17 | `ui/workshop_publish.rs` (d84 `5db83131e`) | **PORTED (REFRESH)** |
| `UIWorkshopSelectWorldToPublish` | 16 | `ui/workshop_publish.rs` (d84 `05d2fe8d8`) | **PORTED (REFRESH)** |
| `UIWorkshopWorldImport` | 18 | `ui/workshop_publish.rs` (d84 `0607f5f8f`) | **PORTED (REFRESH)** |
| `UIResourcePackSelectionMenu` | 17 | `ui/resource_packs.rs` + `resource_pack.rs` (d84 `856a1c74d`/`2d178bcb4`) | **PORTED (REFRESH)** — the data layer landed, d86's dependency fallback moot |
| `UIResourcePackInfoMenu` | 15 | `ui/resource_pack_info.rs` (d84 `ff68eb9d9`) | **PORTED (REFRESH)** |
| `UIDebugCommandsList` | 165 | `ui/debug_commands.rs` (W4-d85) | PORTED (immediate overlay; `/hh` opener, 62-row registry, search + scroll; fake-item hover tooltip not drawn — ANALYSIS "Debug lists (W4-d85)") |
| `UIDebugLoadoutsList` | 146 | `ui/debug_loadouts.rs` + `loadout.rs` (W4-d85) | PORTED (immediate overlay; `/loadouts` opener, ScenarioTestLoadout JSON reader + Apply, static `_lastSearch`; ~~minion respawn deferred~~ **the respawn landed the doc-sweep `6220169d` + the FINAL dragon-grow fix `d3f720cf1`** — ANALYSIS "Debug lists (W4-d85)") |
| `UIDebugLoadoutItem`-family screens | — | `ui/debug_commands.rs` + `ui/debug_loadouts.rs` row halves (W4-d85) | PORTED (the two row elements; hover tooltips not drawn) |
| `UIReportsPage` | 248 | `ui/reports.rs` (W4-d85) | PORTED (immediate front screen; `TR_UI=reports`, empty-log initial message, attach-on-overflow scrollbar + GoToBottom, parked Back target; ~~no issue-report pipeline~~ **the reporter data layer landed `ecbde4c4` + the IssueReportsIndicator consumer wire `389dd8134` (FINAL)** — ANALYSIS "Reports page (W4-d85)") |
| `UIWorldCreationAdvanced` | 532 | `ui/world_creation_advanced.rs` (W4-d85) | PORTED (immediate front screen; the seed icon opens it, the 10-option grid with the Normal/Everything cascades on the draft, the secret cell, the joined-seed description; ring/particles not drawn — ANALYSIS "Advanced world creation (W4-d85)") |
| `UIWorldCreationAdvancedSecretSeedsList` | 459 | `ui/secret_seeds_list.rs` (W4-d85) | PORTED (the unlocked-seed rows from the draft's lists, CustomSort by unlock text, toggle + glow ring, attach-on-overflow scrollbar; particles + settings tracker not ported — same ANALYSIS section) |
| `UIWorldGenDebug` | 27 | none | **SCOPE-OUT (REFRESH: d86 `19641e25f` recorded it in `world_load.rs` + ANALYSIS)** — dev-only worldgen inspector, the `UIWorldLoad.cs:50-52` F5 arm; excluded with `Testing/**` |
| `UICreativePowersMenu` | 13 | `ui/journey.rs` (c24 `ff99f1486`) | **PORTED (REFRESH)** — the Journey strips/categories/sliders/search/duplication |
| `UITextWrappingTest` | 14 | none | **SCOPE-OUT (REFRESH: d86 18c)** — test-only harness, research/38's UNDOCUMENTED-ABSENCE now recorded |
| `UIManageControls` | 20 | `ui/manage_controls.rs` + `input_profiles.rs` (d86 `21fab7274`) | **PORTED (REFRESH)** — the screen state machine + the profile data; rows cite d83's `keybinding_rows.rs` |
| `UIWorldLoad` | 16 | `ui/world_load.rs` (S1-E-17, first-class `FrontScreen::WorldLoad`) | ~~PARTIAL — verify no screen is lost~~ **PORTED (REFRESH: d86 `19641e25f` re-verified in depth)** — nothing was folded into world_select/save_list; the row was stale |
| `NewCraftingUI` | 21 | `crafting.rs` | PORTED (immediate) |
| `UICharacterSelect/UICharacterCreation/UIWorldSelect/UIWorldCreation/UIVirtualKeyboard/UIAchievementsMenu/UIBestiaryTest/UIEmotesMenu` | — | `char_*`, `world_*`, `keyboard.rs`, `achievements.rs`, `bestiary.rs`, `emotes.rs` | PORTED |

## Tests

`grep "#\[test\]"` in `crates/tr-app/src/ui/` — 325 attributes, 291 named test
fns across 31 files (framework-relevant subset):

- `ui/framework.rs` (5): `dw_2_1_back_to_front_the_last_appended_element_wins`
  (`GetElementAt` :327-333), `dw_2_1_ignores_mouse_is_invisible_to_the_hit_but_still_addressable`
  (:330), `dw_2_1_a_zero_sized_rect_is_never_a_hit`, `dw_2_1_the_edges_follow_rectangle_contains`.
- `ui/pointer.rs` (15): `dw_1_1_press_fires_mouse_down_and_a_release_inside_fires_click`,
  `dw_1_1_a_second_press_within_500ms_is_a_double_click_and_clears_last_clicked`,
  `dw_1_1_the_same_pair_past_500ms_is_click_then_down`, `dw_1_1_double_click_requires_the_same_element`,
  `dw_1_2_release_outside_the_down_element_fires_mouse_up_and_no_click`,
  `dw_1_2_the_state_change_disable_swallows_the_next_press`,
  `dw_1_2_the_right_button_behaves_like_the_left`, `dw_1_3_reset_state_clears_a_stale_was_down`,
  `dw_1_3_the_disable_countdown_decays_to_zero_and_clamps`,
  `dw_1_4_the_scroll_wheel_fires_once_on_the_element_under_the_pointer`,
  `dw_1_4_a_scroll_with_no_element_still_consumes_the_delta`, `dw_1_4_a_zero_delta_fires_nothing`.
- `ui/uistate.rs` (12): `resolve_lands_the_select_screen_children`,
  `resolve_clamps_min_width_before_aligning`, `resolve_padding_shrinks_the_inner_rect_only`,
  `spliced_rects_*` (4, incl. the :2593 quirk and the 44×44 sheet map),
  `colour_table_matches_the_vanilla_literals`, `text_panel_rect_applies_the_settext_minimums_to_the_hint_box`,
  `xna_clamp_keeps_the_min_when_min_exceeds_max`, `lookups_default_keeps_every_p6_sheet_none`.
- Screens: `char_select` 17 (`dw_2_3_a_double_click_on_a_card_body_yields_double_click_play`,
  `dw_2_3_the_fat_buttons_do_not_double_click_into_play`), `world_select` 17 (same pair),
  `chest` 13 (`dw_3_2_*` scroll math + `dw_3_2_scroll_wheel_gate_matches_doscrollingininventory`),
  `emotes` 6, `ingame_options` 27, `char_creation` 23, `keyboard` 13.
- No test covers: margins, snap points, or the layer stack. (The `UIList`
  ∞-height case is covered by `resolve_in_list_gives_the_child_an_infinite_parent_height`;
  `OverflowHidden` clipping by `clip.rs`'s stack/edge-rule tests,
  `dw_2_1_a_row_outside_its_list_viewport_is_unreachable` and
  `the_hair_list_clip_rect_bounds_the_grid_content`. The child→parent bubble is
  covered by `framework.rs`'s `the_event_bubble_*` tests, S1-H-1.)

## Deviations & risks

1. ~~**Margins silently dropped**~~ **CLOSED (REFRESH 2026-11-21): d87
   `75009f973`** — `resolve_with_margins` ports `:293-297`/`:317-318`;
   `resolve` delegates at 0 margins bit-identically, so every existing
   caller is unchanged and a future margin-bearing element resolves
   faithfully.
2. **`MaxHeight` clamp uses `f32::MAX`** (`uistate.rs:101`) instead of
   `MaxHeight.GetValue(parentH)` (`UIElement.cs:314,316`). Same as the `Fill`
   default; a percent-valued `MaxHeight` cannot be expressed. Behavioural, latent.
3. ~~**`UIList` infinite-height parent special case not modelled**~~
   **FIXED 2026-11-13 (`S1-H-2-uilst-scissor`)**: `uistate::resolve_with_max_h` /
   `resolve_in_list` port `UIElement.cs:288-291` (`if (Parent is UIList) num3 =
   float.PositiveInfinity`), so percent-height children of a list resolve against
   ∞. `resolve` delegates with `list_parent = false`, which keeps every non-list
   caller byte-identical. Test
   `resolve_in_list_gives_the_child_an_infinite_parent_height`; the hair grid and
   the select-screen rows use it. The earlier text read: percent-height children
   of a list resolve against the list rect instead of ∞.
4. ~~**No child→parent event bubble** (`UIElement.cs:414-547` vs `pointer.rs:154-224`).~~
   **FIXED 2026-11-13 (`S1-H-1-event-bubbling`)**: `framework.rs::bubble_chain`
   ports the `:420-423` recursion as a parent-chain walk over `Element.parent`
   (`Element::child`), so a leaf control's event reaches its container's
   handlers and the same chain is the ancestor-`IsMouseHovering` set
   (`MouseOver` sets the flag then recurses, `:510-521`). Wired to the
   `UICharacterListItem`/`UIWorldListItem` button rows (they link to their card
   panel; `card_hit` resolves the card by walking the chain). The earlier text
   read: ancestor `On_*` handlers never fire and ancestor `IsMouseHovering` is
   never set (ROADMAP:938-939's "ported" claim was wrong for the Rust side).
   Also still unmodelled: the C#'s Out-then-Over double-toggle when the pointer
   moves child→parent inside one frame (`UserInterface.cs:205-213`).
5. ~~**No scissor / `OverflowHidden`**~~ — **FIXED 2026-11-13
   (`S1-H-2-uilst-scissor`)**: `tr-render/src/clip.rs` carries `NO_CLIP`,
   `intersect` and a `ClipStack`; `SpriteInstance.clip` is stamped on every
   instance and `sprite.wgsl` discards fragments outside it, so a row straddling
   a list edge is half-painted exactly as the C# device scissor cuts it. The
   break-gates in `char_select.rs`/`world_select.rs`/`char_creation.rs` are gone.
   The hit side is `Element.viewport` (`Element::row`/`row_child`): `get_element_at`
   now descends only through an ancestor that contains the point
   (`UIElement.cs:327-341`), which reproduces the C# *without* the old
   approximation (a row fully past the edge is unreachable, which is what the C#
   does because the list parent rejects the point first). Tests
   `dw_2_1_a_row_outside_its_list_viewport_is_unreachable`,
   `the_hair_list_clip_rect_bounds_the_grid_content`, `clip.rs`'s 7.
   Screenshot: the hair grid loses 140 949 px at `y >= 484` with the scissor on,
   two runs SHA256-equal. **Deviation:** the clip is a per-instance attribute +
   fragment `discard`, not `set_scissor_rect` device state — the renderer batches
   all UI into one instanced draw, so a device scissor would need one pass per
   rect. Pixel-equivalent at the C# rule (`left <= x < right`), recorded in
   `docs/ANALYSIS.md`.
6. **UI scale applied only at two sites** — `Main.UIScale`/`GameUI.UIScale` are
   now ported (`settings::ui_scale_max`, `Settings::ui_scale`, the ingame-options
   bar, S1-H-3), but the `UserInterface.GetDimensions()` division
   (`:332-336`) and `GameInterfaceLayer`'s three scale spaces are still not
   applied to the per-screen rects, and only the fullscreen-map head icon reads
   the applied scale. Behavioural; partially admitted (S1-H-3 follow-up).
7. ~~**Scroll wheel not in the pointer machine**~~ — **FIXED 2026-11-13**
   (`S1-H-4-scroll-wheel`): `UserInterface.cs:219-223`'s arm now lives in
   `pointer.rs` (`queue_scroll` + `scroll_wheel`, `PointerEvent::ScrollWheel`
   ≈ `UIScrollWheelEvent`); the delta is consumed once against the element under
   the pointer. The per-screen consumers still own their own rect/clamp maths,
   and `loop.rs`'s hotbar/`map` fallbacks stay raw (they are outside
   `UserInterface`).
8. **Frame ordering** — C# `Update` is hover → left click → right click → scroll →
   state update (`UserInterface.cs:190-228`); the front-end screens are
   event-driven from `loop.rs` (`:390-403`), so the press dispatch may run before
   or after a hover refresh in the same frame. Determinism-relevant for
   double-click edges. Not verified; no test pins it.
9. ~~**`GoBack`/history "dead code" claim unverified**~~ **VERIFIED
   2026-11-21 (W4-d85 doc-sweep `ef9c8954`)** — the full `decompiled/` +
   `decompiled-libs/` sweep finds 8 `.GoBack()` hits, ALL UIState-own
   private/static methods (`UICharacterCreation.cs:1207/:2378`,
   `UIWorldCreation.cs:667/:1375`, `UIWorldCreationAdvanced.cs:472/:530`,
   `UIWorldCreationAdvancedSecretSeedsList.cs:383/:410`); ZERO callers of
   `UserInterface.GoBack` — `_history` is write-only in 1.4.5.8. The
   original deviation's "is called from the back-button paths" claim was
   FALSE (the `IHaveBackButtonCommand` screens call their OWN GoBack
   methods, not the UserInterface one). d87 ports the state-stack
   machinery anyway (`164cb7090`).
10. **Snap points / gamepad** — **PARTIAL (REFRESH 2026-11-21): d87
    `164cb7090` ports the anchor maths** (`SnapPoint.cs:26-30` +
    `UIElement.cs:124-145/:271-283`); the `UILinkPointNavigator` and the
    `NoGamepadSupport` handling (`UIState.cs:5`) stay absent — gamepad
    input unsupported, expected for Stage 1 (unchanged).

## Task seeds

- **S — Pin the event order.** Add a `ui/pointer.rs` test that reproduces
  `UserInterface.cs:190-228`'s order (hover swap before clicks, left before right)
  against the front-end dispatch in `app/loop.rs:390-403`. Depends on nothing.
- ~~**S — Verify the `GoBack` dead-code claim.**~~ **DONE 2026-11-21
  (W4-d85 doc-sweep lane 30)**: the full `decompiled/` + `decompiled-libs/`
  sweep for `.GoBack()` finds 8 call sites, all UIState-own methods
  (`UICharacterCreation.cs:1207/:2378`, `UIWorldCreation.cs:667/:1375`,
  `UIWorldCreationAdvanced.cs:472/:530`,
  `UIWorldCreationAdvancedSecretSeedsList.cs:383/:410`); zero callers of
  `UserInterface.GoBack` (`UserInterface.cs:305-313`) — the `_history`
  list is write-only in 1.4.5.8. Evidence recorded in `pointer.rs:6-15`.
- ~~**M — Port margins into `uistate::resolve`.**~~ **DONE (REFRESH
  2026-11-21): d87 `75009f973`** — `resolve_with_margins` inflates before
  align, shrinks after; `resolve` delegates at 0 margins bit-identically.
  Deviation 1 closed for future callers.
- ✅ **M — Model the `UIList` ∞-height parent.** DONE 2026-11-13
  (`S1-H-2-uilst-scissor`): `uistate::resolve_in_list` sets
  `parent[3] = f32::POSITIVE_INFINITY` (`UIElement.cs:288-291`), tested by
  `resolve_in_list_gives_the_child_an_infinite_parent_height`. The scrolling
  screens' rows use it.
- ✅ **M — Ancestor bubble.** DONE 2026-11-13 (`S1-H-1-event-bubbling`): the
  parent chain lives in `framework.rs` (`Element.parent`/`Element::child` +
  `bubble_chain`, `UIElement.cs:414-547`), not `pointer.rs` — the C#'s recursion
  is in `UIElement`, so `UIElement` (the framework module) is its faithful home.
  The chain is the event's raise list *and* the ancestor-`IsMouseHovering` set;
  it is wired to the select screens' card buttons. Deviation 4 closed for hover.
- ✅ **L — Clipping.** DONE 2026-11-13 (`S1-H-2-uilst-scissor`): the scissor is a
  per-instance `clip` vec4 in `tr-render/src/clip.rs` + `SpriteInstance.clip`,
  discarded in `sprite.wgsl` (one instanced draw ⇒ no device scissor state),
  wired into `char_select.rs`/`world_select.rs`/`char_creation.rs` in place of the
  break-gates. Pixel-verified with `TR_SHOT` (140 949 px removed below the hair
  list's inner bottom; two runs SHA256-equal) and the hit side by
  `Element.viewport`. Remaining: the *scrollbar* is still static on those screens
  (the `UIScrollbar` drag element is untouched) and `UIInnerList.RecalculateChildren`
  is still per-screen arithmetic.
- ~~**M — `UIWorldCreationAdvanced` + secret-seeds list** (ROADMAP:953)~~
  **DONE 2026-11-20 (W4-d85 rows 11-12)**: `ui/world_creation_advanced.rs`
  + `ui/secret_seeds_list.rs` immediate ports; the option rows derive
  arithmetically (the `UIList` ∞-height seed landed). The Everything
  icon's glitch animation landed 2026-11-21 (doc-sweep lane 30).
- ~~**L — Screens with no counterpart:** `UIManageControls` …
  `UICreativePowersMenu`. `UIWorkshop*` / `UIResourcePack*` should be
  scoped out (Steam-only)~~ **DONE (REFRESH 2026-11-21)**: UIManageControls
  → d86 `21fab7274`, UICreativePowersMenu → c24 `ff99f1486`, the
  Workshop/ResourcePack cluster → d84 `ba7e8f94a` (the no-Steam VIEWS
  ported; the Steam I/O stays excluded, research/38's rows updated).
  The only unported screens left are the two d86 scope-outs
  (UIWorldGenDebug/UITextWrappingTest) — research/112 §5's final table.
- ~~**L — Layer stack.**~~ **PARTIAL (REFRESH 2026-11-21): d87
  `164cb7090`** — the named 43-layer list (`Main.cs:44718-44918`) + the
  stop-on-false walk + the `InterfaceScaleType` spaces landed as data;
  the per-layer zoom-matrix application to the draw passes stays the
  follow-up (d87's own PARTIAL mark).
