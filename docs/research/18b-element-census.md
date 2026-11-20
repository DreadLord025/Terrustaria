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

**Re-audit finding (2026-11-20):** research/18's gap table predates S1-E-17 and
the crafting-filter landing. Five band classes already have faithful immediate
ports — the lane keeps them (no re-split) and ports only their *missing halves*:

- `UIGenProgressBar` (row 31) — **LANDED** S1-E-17: `ui/world_load.rs`
  `GenProgressBar`/`GenBarFrame`/`filling_even_width`/`lerp_color_packed` +
  the draw (`UIGenProgressBar.cs:8-126` whole; the drunk-world `Main.rand`
  flicker is a documented RNG-order deviation). Nothing left.
- `UIHeader` (row 33) — **LANDED** S1-E-17: `ui/world_load.rs::draw_header`
  (`UIHeader.cs:46-62` whole, incl. the HotPink arm). The `Text` setter's
  change-gate + measure pin now in `ui/chrome_elements.rs::HeaderText`.
- `UIHorizontalSeparator` (row 34) — **LANDED**: `ui/panel.rs::separator_rects`/
  `separator_srcs`/`draw_horizontal_separator` (`Utils.DrawPanel` 2/0). Missing
  halves: the ctor's `highlightSideUp` texture pick + size (`:16-29`) and the
  `ContainsPoint => false` override (`:37-40`).
- `UIDifficultyButton` (row 29) — draw/click/title **LANDED**
  (`char_creation.rs:1981-2018`, `Click::Difficulty`). Missing halves: the
  `_hovered`/`_soundedHover` one-shot hover sound (`:49-60`) and the
  `MouseOver`/`MouseOut` flags (`:88-98`).
- `UIHairStyleButton` (row 32) — draw/click **LANDED**
  (`char_creation.rs::draw_hair_button`, `Click::Hair`; the head-preview
  deviation documented there). Missing halves: `SkipRenderingContent`/
  `_framesToSkip` (`:36-39`, `:65-69`) and the hover-sound latch.

| row | class | status |
| --- | --- | --- |
| 22 | `UICreativeInfiniteItemsDisplay` | LANDED (`ui/creative_display.rs` state machines) |
| 23 | `UICreativeItemGrid` | LANDED (`ui/creative_items.rs`; grid base row 30) |
| 24 | `UICreativeItemsInfiniteFilteringOptions` | LANDED (board `craft_filter` + geometry `crafting` pre-landed; theme/sizing/tint `ui/creative_items.rs`) |
| 25 | `UICreativePowerButton` | LANDED (empty marker, `ui/creative_powers.rs`) |
| 26 | `UICyclingImage` | LANDED (`ui/images.rs::CyclingImage`) |
| 27 | `UIDebugCommandItem` | LANDED (`ui/debug_items.rs`) |
| 28 | `UIDebugLoadoutItem` | LANDED (`ui/debug_items.rs`) |
| 29 | `UIDifficultyButton` | LANDED (hover latch `ui/buttons.rs`; draw/click pre-landed) |
| 30 | `UIDynamicItemCollection` | LANDED (`ui/item_collection.rs`) |
| 31 | `UIGenProgressBar` | LANDED (S1-E-17, re-verified) |
| 32 | `UIHairStyleButton` | LANDED (skip + hover latch `ui/buttons.rs`; draw/click pre-landed) |
| 33 | `UIHeader` | LANDED (S1-E-17 draw + `ui/chrome_elements.rs::HeaderText` setter) |
| 34 | `UIHorizontalSeparator` | LANDED (ctor pick + hit override `ui/chrome_elements.rs`; draw pre-landed) |

## d83 progress (`task/W4-d83-ui-elements-3`, base = d82 tip `53912fa0`)

d81's batch 1 (`0b815a62`) landed shared small classes into `ui/elements.rs`
(GroupOptionButtonState, PowerStrip, traits, UICyclingImage cycler,
UICreativePowerButton marker) ahead of its row band; d83 keeps the census rows
as the claim record and ports the *remaining halves* without re-splitting.
Where d81/d82 already landed a class's state, the row cites both homes.

| row | class | status |
| --- | --- | --- |
| 35 | `UIIconTextButton` | LANDED (`ui/buttons.rs::IconTextButton`, colour/opacity machine + SetText resize + icon geometry `UIIconTextButton.cs:36-132`; hover latch shared with rows 29/32; dead ctor params + `_hoverColor` documented) |
| 36 | `UIImage` | LANDED (`ui/images.rs::ImageElement`/`ImageDraw`, draw geometry `UIImage.cs:70-99` + SetImage resize) |
| 37 | `UIImageFramed` | LANDED (`ui/images.rs::FramedImage`, ctor/SetFrame/grid slice `:16-43` via `Utils.cs:1344/1328`) |
| 38 | `UIImageWithBorder` | LANDED (`ui/images.rs::BorderedImage`, SetBorder resize + border draw `:26-67`) |
| 39 | `UIItemIcon` | LANDED (`ui/item_collection.rs::ItemIconElement`, 32×32 ctor + context-31 centre draw + blackout tint `UIItemIcon.cs:13-25`; icon draw pre-landed `slot.rs::draw_item_icon`) |
| 40 | `UIKeybindingCycleListItem` | LANDED (`ui/keybinding_rows.rs::KeybindingCycleRow`, shared row chrome + "???" fallbacks + value GlyphsScale 0.85 window `:18-46`; dead `X -= 17` documented) |
| 41 | `UIKeybindingListItem` | LANDED (`ui/keybinding_rows.rs::KeybindingRow`, click listen-latch `:28-41`, GenInput join `:80-112`, GetFriendlyName table `:114-176`, listening Gold / unbound (80,80,80) arms + gamepad -3 shift `:43-78`; GlyphTagHandler.GenerateTag table `GlyphTagHandler.cs:169-289`) |
| 42 | `UIKeybindingSimpleListItem` | LANDED (`ui/keybinding_rows.rs::KeybindingSimpleRow`, centred text X+W/2-strW/2 + shared chrome `:21-39`; DrawSettings2Panel = DrawSettingsPanel documented) |
| 43 | `UIKeybindingSliderItem` | LANDED (`ui/keybinding_rows.rs::KeybindingSliderRow`, rightHover/rightLock grab machine + num2 bar state + flag overrides + keyboard/gamepad action gates `:42-99`; valuePosition +10; DrawValueBar pre-landed `sliders.rs`) |
| 44 | `UIKeybindingToggleListItem` | LANDED (`ui/keybinding_rows.rs::KeybindingToggleRow`, (W-2)/2+2 integer toggle frame + right-edge White scale-1 draw `:28-47`; dead Gold arm + X-=17 documented) |
| 45 | `UIParticleLayer` | LANDED (`ui/particles.rs::ParticleLayer`, ToRectangle-truncated anchor + percents/pixels offsets `:28-33`, IgnoresMouseInteraction pin `:18`; ParticleRenderer unported, caller's seam) |
| 46 | `UIProgressBar` | LANDED (`ui/progress.rs::ProgressBar`, 0.95/0.05 ease + snap-down SetProgress + Blue MagicPixel fill `UIProgressBar.cs:31-45`; distinct from row 31's UIGenProgressBar) |
| 47 | `UIResourcePack` | LANDED (`ui/resource_packs.rs::ResourcePackCard` + `pack_child_layout`, static colour pairs *0.8 truncation `:135-141`, hover swap, icon/border stack + workshop badge geometry `:98-114`, BuildChildren top walk `:57-96`; planned home `ui/resources.rs` taken by the HUD module — renamed, no re-split) |
| 48 | `UIResourcePackInfoButton` | LANDED (`ui/text_panels.rs::PackInfoButton`, PanelGrayscale splice Lerp(Black,_color,0.8)*0.5 + White CategoryPanelBorder hover `:36-50`) |
| 49 | `UISearchBar` | LANDED (`ui/search_bar.rs::SearchBar`, SetContents placeholder/cap/read-back `:67-89`, int-edge display-trim loop `:91-114`, input latch `:169-196`; shared Utils.TrimUserString/TrimLastCharacter UTF-16 helpers `Utils.cs:2839-2855`; IME/GetInputText caller's seam) |
| 50 | `UISelectableTextPanel` | LANDED (`ui/text_panels.rs::SelectableTextPanel`, base splice + 4px-inset selected splice Lerp(_color,White,0.7)*0.5 + hover border stack `:36-55`) |
| 51 | `UISlicedImage` | LANDED (`ui/images.rs::SlicedImage`, ctor-only resize + transparent default(Color) + top/bottom-first SetSliceDepths + int-truncated spliced draw `UISlicedImage.cs:34-66`; pieces ride `uistate::spliced_rects`) |
| 52 | `UISliderBase` | LANDED (`ui/sliders.rs::SliderLocks`, id-keyed CurrentLocked/AimedSlider statics + GetUsageLevel 0/1/2 + EscapeElements `UISliderBase.cs:13-35`; HSL-bar mirror pre-landed `char_creation.rs`) |
| 53 | `UIText` | LANDED (`ui/text.rs::TextElement`, InternalSetText metrics (16/32*scale vs wrapped +20*scale, MinWidth 0 for wrapped/dynamic) + DrawSelf geometry (-2/-10*scale shift, origin centring, dynamic shrink, shadow*textAlpha) `UIText.cs:82-194`; snippet layout caller's, wrap = `bestiary.rs::wrap_text`) |
| 54 | `UITextBox` | pending |
| 55 | `UIToggleImage` | pending |
| 56 | `UIVerticalSeparator` | pending |
| 57 | `UIVerticalSlider` | pending |
| 58 | `UIWorkshopImportWorldListItem` | pending |
| 59 | `UIWorkshopPublishResourcePackListItem` | pending |
| 60 | `UIWorkshopPublishWorldListItem` | pending |
| 61 | `UIWorldCreationPreview` | pending |
| 62 | `UIWorldListItem` | pending |
| 63 | `UIWrappedSearchBar` | pending |
| 64 | `UISortableElement` | pending |

## Conventions (landed style, per research/18 + S1-E-26 wave B)

Code-only lane: ports are **pure helpers + tests-as-code** (written, not run —
no cargo in this lane), one module per family under `crates/tr-app/src/ui/`,
every ported block carries `// File.cs:line` citations, `anchors.txt` rows per
landed family, docs updated (this file + PORT_MAP + research/18 status rows).
Screens keep their immediate-mode draw; the element ports supply the C# class's
*state machine and geometry* as pure functions the screens can adopt later
(the S1-H-13 `panel::scrollbar_*` pattern).
