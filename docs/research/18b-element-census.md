# 18b — UIElement class census + the d81/d82/d83 range split

> Wave 4 · published 2026-11-20 by `task/W4-d82-ui-elements-2` (base `a725acb0`).
> d81 and d83 were unstarted at publication time (both at the same base, clean).
> This census is the shared claim record for the three element lanes; siblings
> read it before taking classes. If a sibling re-splits, edit THIS table (never
> silently take another lane's rows) and say so in the commit message.

## Scope ruling

The lane task ("missing UIElement classes", research/18 gap list) covers the
**concrete `UIElement` subclasses** under
`decompiled/Terraria/GameContent/UI/Elements/` (74 files) plus
`UI/States/UISortableElement.cs` — 75 class files. Of these:

- **5 are interfaces, not elements** — `IColorable` (8 ln), `IGroupOptionButton`
  (10), `IManuallyOrderedUIElement` (6), `ExtraBestiaryInfoPageInformation` (8,
  a data struct) — excluded from the class count; ported alongside their users.
- **8 are already ported** (directly or as faithful immediate halves), excluded
  from the split: `UIList`+`UIInnerList` (S1-H-13, select-screen `Layout`),
  `UIScrollbar` (S1-H-13 `panel::scrollbar_*`), `UIPanel` (`panel.rs`),
  `UITextPanel`/`UIImageButton`/`UIColoredImageButton` (`uistate.rs:290-527`),
  `UIItemSlot` (`slot.rs`, ports `ItemSlot.Draw` directly — the *element* wrapper
  stays a documented deviation).

That leaves **62 missing concrete classes**, split by sorted-file order into
three lanes of ~21/13/28 (d82's lane is the small middle band by task charter:
"SECOND ~13"). Sorted order is the tie-breaker: a class belongs to the lane its
row says, nothing else.

## The split (sorted by C# file name)

| # | C# class (Elements/ unless noted) | ln | lane | Rust home (planned) |
| --- | --- | --- | --- | --- |
| 1 | `AWorldListItem` | 201 | d81 | `ui/world_select.rs` (element half) |
| 2 | `EmoteButton` | 96 | d81 | `ui/emotes.rs` (element half) |
| 3 | `EmotesGroupListItem` | 95 | d81 | `ui/emotes.rs` |
| 4 | `GroupOptionButton` | 290 | d81 | `ui/group_option.rs` |
| 5 | `PowerStripUIElement` | 44 | d81 | `ui/power_strip.rs` |
| 6 | `UIAchievementListItem` | 275 | d81 | `ui/achievements.rs` (element half) |
| 7 | `UIBestiaryEntryButton` | 170 | d81 | `ui/bestiary.rs` (element half) |
| 8 | `UIBestiaryEntryGrid` | 124 | d81 | `ui/bestiary.rs` |
| 9 | `UIBestiaryEntryIcon` | 73 | d81 | `ui/bestiary.rs` |
| 10 | `UIBestiaryEntryInfoPage` | 202 | d81 | `ui/bestiary.rs` |
| 11 | `UIBestiaryFilteringOptionsGrid` | 236 | d81 | `ui/bestiary.rs` |
| 12 | `UIBestiaryInfoItemLine` | 168 | d81 | `ui/bestiary.rs` |
| 13 | `UIBestiaryInfoLine` | 100 | d81 | `ui/bestiary.rs` |
| 14 | `UIBestiaryNPCEntryPortrait` | 65 | d81 | `ui/bestiary.rs` |
| 15 | `UIBestiarySortingOptionsGrid` | 123 | d81 | `ui/bestiary.rs` |
| 16 | `UICharacter` | 163 | d81 | `ui/char_select.rs` (element half) |
| 17 | `UICharacterListItem` | 381 | d81 | `ui/char_select.rs` |
| 18 | `UICharacterNameButton` | 149 | d81 | `ui/char_creation.rs` |
| 19 | `UIClothStyleButton` | 109 | d81 | `ui/char_creation.rs` |
| 20 | `UIColoredSlider` | 160 | d81 | `ui/sliders.rs` |
| 21 | `UIColoredSliderSimple` | 38 | d81 | `ui/sliders.rs` |
| 22 | `UICreativeInfiniteItemsDisplay` | 578 | **d82** | `ui/creative_items.rs` |
| 23 | `UICreativeItemGrid` | 35 | **d82** | `ui/creative_items.rs` |
| 24 | `UICreativeItemsInfiniteFilteringOptions` | 133 | **d82** | `ui/creative_items.rs` |
| 25 | `UICreativePowerButton` | 7 | **d82** | `ui/creative_powers.rs` |
| 26 | `UICyclingImage` | 38 | **d82** | `ui/images.rs` |
| 27 | `UIDebugCommandItem` | 120 | **d82** | `ui/debug_items.rs` |
| 28 | `UIDebugLoadoutItem` | 149 | **d82** | `ui/debug_items.rs` |
| 29 | `UIDifficultyButton` | 99 | **d82** | `ui/buttons.rs` |
| 30 | `UIDynamicItemCollection` | 166 | **d82** | `ui/item_collection.rs` |
| 31 | `UIGenProgressBar` | 126 | **d82** | `ui/progress.rs` |
| 32 | `UIHairStyleButton` | 97 | **d82** | `ui/buttons.rs` |
| 33 | `UIHeader` | 63 | **d82** | `ui/chrome_elements.rs` |
| 34 | `UIHorizontalSeparator` | 41 | **d82** | `ui/chrome_elements.rs` |
| 35 | `UIIconTextButton` | 132 | d83 | `ui/buttons.rs` |
| 36 | `UIImage` | 100 | d83 | `ui/images.rs` |
| 37 | `UIImageFramed` | 50 | d83 | `ui/images.rs` |
| 38 | `UIImageWithBorder` | 68 | d83 | `ui/images.rs` |
| 39 | `UIItemIcon` | 26 | d83 | `ui/item_collection.rs` |
| 40 | `UIKeybindingCycleListItem` | 48 | d83 | `ui/keybinding_rows.rs` |
| 41 | `UIKeybindingListItem` | 177 | d83 | `ui/keybinding_rows.rs` |
| 42 | `UIKeybindingSimpleListItem` | 40 | d83 | `ui/keybinding_rows.rs` |
| 43 | `UIKeybindingSliderItem` | 101 | d83 | `ui/keybinding_rows.rs` |
| 44 | `UIKeybindingToggleListItem` | 48 | d83 | `ui/keybinding_rows.rs` |
| 45 | `UIParticleLayer` | 49 | d83 | `ui/particles.rs` |
| 46 | `UIProgressBar` | 46 | d83 | `ui/progress.rs` |
| 47 | `UIResourcePack` | 142 | d83 | `ui/resources.rs` (element half) |
| 48 | `UIResourcePackInfoButton` | 51 | d83 | `ui/resources.rs` |
| 49 | `UISearchBar` | 197 | d83 | `ui/search_bar.rs` |
| 50 | `UISelectableTextPanel` | 56 | d83 | `ui/text_panels.rs` |
| 51 | `UISlicedImage` | 67 | d83 | `ui/images.rs` |
| 52 | `UISliderBase` | 36 | d83 | `ui/sliders.rs` |
| 53 | `UIText` | 195 | d83 | `ui/text.rs` (element half) |
| 54 | `UITextBox` | 111 | d83 | `ui/text_box.rs` |
| 55 | `UIToggleImage` | 71 | d83 | `ui/images.rs` |
| 56 | `UIVerticalSeparator` | 34 | d83 | `ui/chrome_elements.rs` |
| 57 | `UIVerticalSlider` | 140 | d83 | `ui/sliders.rs` |
| 58 | `UIWorkshopImportWorldListItem` | 199 | d83 | `ui/workshop_rows.rs` |
| 59 | `UIWorkshopPublishResourcePackListItem` | 216 | d83 | `ui/workshop_rows.rs` |
| 60 | `UIWorkshopPublishWorldListItem` | 140 | d83 | `ui/workshop_rows.rs` |
| 61 | `UIWorldCreationPreview` | 133 | d83 | `ui/world_creation.rs` (element half) |
| 62 | `UIWorldListItem` | 533 | d83 | `ui/world_select.rs` (element half) |
| 63 | `UIWrappedSearchBar` | 247 | d83 | `ui/search_bar.rs` |
| 64 | `UISortableElement` (States/) | 25 | d83 | `ui/sortable.rs` |

**Lane totals:** d81 = rows 1–21 (21 classes), **d82 = rows 22–34 (13 classes)**,
d83 = rows 35–64 (30 classes, the "rest" per charter).

## d82 progress (this branch)

| row | class | status |
| --- | --- | --- |
| 22 | `UICreativeInfiniteItemsDisplay` | pending |
| 23 | `UICreativeItemGrid` | pending |
| 24 | `UICreativeItemsInfiniteFilteringOptions` | pending |
| 25 | `UICreativePowerButton` | pending |
| 26 | `UICyclingImage` | pending |
| 27 | `UIDebugCommandItem` | pending |
| 28 | `UIDebugLoadoutItem` | pending |
| 29 | `UIDifficultyButton` | pending |
| 30 | `UIDynamicItemCollection` | pending |
| 31 | `UIGenProgressBar` | pending |
| 32 | `UIHairStyleButton` | pending |
| 33 | `UIHeader` | pending |
| 34 | `UIHorizontalSeparator` | pending |

## Conventions (landed style, per research/18 + S1-E-26 wave B)

Code-only lane: ports are **pure helpers + tests-as-code** (written, not run —
no cargo in this lane), one module per family under `crates/tr-app/src/ui/`,
every ported block carries `// File.cs:line` citations, `anchors.txt` rows per
landed family, docs updated (this file + PORT_MAP + research/18 status rows).
Screens keep their immediate-mode draw; the element ports supply the C# class's
*state machine and geometry* as pure functions the screens can adopt later
(the S1-H-13 `panel::scrollbar_*` pattern).
