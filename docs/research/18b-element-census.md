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

**Lane closed 2026-11-20:** all 13 rows LANDED across batches 1-3
(`484c2529`/`eefc3b05`/`53912fa0`), 8 anchors + 21 witness tests written not
run; docs landed (ANALYSIS "The UIElement band d82", PORT_MAP row, ROADMAP
module-18 row, research/18 annotations). **Cross-lane note:** d81's batch 1
(`0b815a62`) independently ports `UICyclingImage` + `UICreativePowerButton`
in `ui/elements.rs` (pushed after this census); the merge owner keeps one
copy — see ANALYSIS.
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
| 54 | `UITextBox` | LANDED (`ui/text_box.rs::TextBox`, 20-unit cap + cursor clamp/Write-insert/Backspace-tail quirk + draw cursor-to-end pin + 21-of-40 ticker + caret offsets `UITextBox.cs:38-109`; keyboard write/caret halves pre-landed `keyboard.rs`; IME caller's seam) |
| 55 | `UIToggleImage` | LANDED (`ui/images.rs::ToggleImage`, one-sheet on/off offsets + fixed draw size + White/Silver hover tint + LeftClick toggle `UIToggleImage.cs:26-70`; achievements filter draw pre-landed) |
| 56 | `UIVerticalSeparator` | LANDED (`ui/chrome_elements.rs::VerticalSeparator`, OnePixel stretch ToRectangle + White ctor + never-hit + dead EdgeWidth `UIVerticalSeparator.cs:16-33`) |
| 57 | `UIVerticalSlider` | LANDED (`ui/sliders.rs::VerticalSlider`, 5/4/4/4 spliced frame + Right/Bottom-edge insets (-9/-8) + bottom-up fill + CenteredRectangle knob + hit Inflate(4,0) + aim/lock machine + two independent sound one-shots + clamped bottom-up GetLerpValue `UIVerticalSlider.cs:39-138`) |
| 58 | `UIWorkshopImportWorldListItem` | LANDED (`ui/workshop_rows.rs::WorkshopImportWorldRow`, shared rest/hover pairs, Import label latch, valid+target click gate + name trim + invalid Gray status prefix, 100/150(+40)/remainder panel walk + divider /8 scale `:66-198`; SocialAPI/keyboard caller's seams) |
| 59 | `UIWorkshopPublishResourcePackListItem` | LANDED (`ui/workshop_rows.rs::WorkshopPublishPackRow`, zip grey arms (Lerp hover→120 *1 / Lerp rest→80 *0.7), rejection-vs-publish click routing, (W-22-icon-btn)/2 two-panel split `:95-215`; spliced 10/10/10/10 panels) |
| 60 | `UIWorkshopPublishWorldListItem` | LANDED (`ui/workshop_rows.rs::WorkshopPublishWorldRow`, plain shared pairs (no grey arms), target-only click, +6 panel origin + second-panel non-English +40 `:53-139`) |
| 61 | `UIWorldCreationPreview` | LANDED (re-audit: whole class pre-landed `ui/world_creation.rs::preview_layers`/`draw_preview` — UpdateOption mapping, 0/3→Normal bg, DarkGray Expert/Master tint, Master bunny *1.2 XNA wrap, +4 layer offset + border `UIWorldCreationPreview.cs:44-132`; nothing left) |
| 62 | `UIWorldListItem` | LANDED (draw/click/colours/labels pre-landed S1-H-10 `ui/world_select.rs`; missing halves added: conditional cloud/seed slot num-walk + their hover keys + status-icon label keys + SeedCopied + WorldFileData.GetFullSeedText/GetSerializedSeedsSum `UIWorldListItem.cs:113-162/:271-318/:460-464`, `WorldFileData.cs:123-197`; slots parked until SocialAPI/generator-version data lands) |
| 63 | `UIWrappedSearchBar` | LANDED (`ui/search_bar.rs::WrappedSearchBar`, theme box colour (35,40,83)/cyan shift, OurFavoriteColor focus border, HideSearchButton -3 width, cancel 11/12 sounds, right-click/outside-click/search-area gates, virtual submit trim; draw pre-landed `crafting.rs:756-838`) |
| 64 | `UISortableElement` | LANDED (`ui/sortable.rs::SortableElement`, OrderIndex + ascending CompareTo with the base-arm fallback `UISortableElement.cs:5-22`) |

**d83 complete (2026-11-20):** all 30 rows (35-64) LANDED. Batches 1-11 on
`task/W4-d83-ui-elements-3`; tests written not run (code-only lane). The
planned `ui/resources.rs` home (row 47) was taken by the HUD module — the
pack card lives in `ui/resource_packs.rs` (no re-split). Row 61 re-audited
as wholly pre-landed (`world_creation.rs`).
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

## d81 progress (task/W4-d81-ui-elements-1)

**Keep-one dedupe (2026-11-20):** d81's batch 1 (`0b815a62`, written before this
census was published) ported `UICyclingImage` (row 26) and
`UICreativePowerButton` (row 25) into `ui/elements.rs`. Both are **d82 rows**;
d82 landed them independently (`ui/images.rs::CyclingImage`,
`ui/creative_powers.rs::CreativePowerButton`, `484c2529`). The d81 copies are
**removed** — the d82 modules are the survivors. No behaviour change: the two
ports were byte-equivalent state machines and neither had a consumer yet.

| row | class | status |
| --- | --- | --- |
| 1 | `AWorldListItem` | LANDED (missing halves `ui/world_select.rs`: `difficulty_text_with_ftw` full `:43-85` chain, `WorldIconFlags`/`seed_icon_name`/`first_icon_name`/`icon_name_list` `:87-200`, `GlitchAnimation` `:21-41` with injected `Main.rand` rolls; card geometry/difficulty draw pre-landed) |
| 2 | `EmoteButton` | LANDED (re-audit: whole class pre-landed — blink counter `emotes.rs::button_frame` `:38-44`, frame pick + border tint + MouseText `ui/emotes.rs::draw` `:32-75`, hover sound-12 edge `app/loop.rs:600-619` `:77-88`, click+close `app/clicks.rs:1170-1197` `:90-95`) |
| 3 | `EmotesGroupListItem` | LANDED (geometry `ui/emotes.rs`: `group_row_count`/`group_height`/`group_header_rect`/`group_separator_color`/`group_button_top_left`/`compare_group_order` `:11-94`; the flat-roster wheel draw pre-landed, group titles/separators a documented deviation) |
| 4 | `GroupOptionButton` | LANDED (colour state batch 1; missing halves `ui/elements.rs`: `InnerHighlightRim`/`highlight_rect` `:32`/`:236`, `hover_border_color` `:240`, `icon_tint` `:242-249`, `hover_sound_tick` `:212-223`, `GROUP_OPTION_SIZE`/`group_option_title_rect` `:124-125`/`:136-142`; the world-creation draw pre-landed `option_button_colors`) |
| 5 | `PowerStripUIElement` | LANDED (`ui/elements.rs` strip layout/colours, batch 1; re-audit: ctor `:13-43` fully covered minus the unported gamepad snap points) |
| 6 | `UIAchievementListItem` | LANDED (draw/geometry/hover pair/sort pre-landed `ui/achievements.rs::draw_row` + `sorted_indices`; missing halves: `row_height`/`icon_offsets` small-language branch `:49-53`, `icon_frame_rect` `:57-61`/`:155-169`, `description_scale_y` `:101-112`, `tracker_bar_geometry`/`tracker_bar_colors` `:139-149`/`:241-256` — the flat-bar draw + horizontal shrink stay documented stand-ins) |
| 7 | `UIBestiaryEntryButton` | LANDED (card draw/hover text pre-landed `ui/bestiary.rs::draw_card`; missing halves: `entry_button_border_layers` `:150-169`/`:93-96`, `pick_background_provider` `:104-129`, `first_display_index` `:131-140`; the always-overlay hover stack stays a documented deviation until adoption; the `MouseOver` sound-12 edge `:152` defers to wiring — the shared `elements.rs::hover_sound_tick` latch is the mechanism) |
| 8 | `UIBestiaryEntryGrid` | LANDED (metrics/range/clamp/draw pre-landed; missing halves `ui/bestiary.rs`: `offset_library_by_pages` `:103-114`, `grid_cell_rect` fractional placement `:50-66` — the raw-lattice draw stays the documented simplification) |
| 9 | `UIBestiaryEntryIcon` | LANDED (locked draw/hover text/portrait layout pre-landed; missing halves `ui/bestiary.rs`: `icon_is_hovered` `:38`/`:52` ForceHover gate, `icon_walks` `UnlockableNPCEntryIcon.cs:100-109` walk gate — the frame-0 rest stays the documented no-cache deviation) |
| 10 | `UIBestiaryEntryInfoPage` | LANDED (stat rows/drop lines pre-landed `draw_info_page` + `drop_rate_text`; missing halves `ui/bestiary.rs`: `info_page_colors`/`INFO_PAGE_LIST_PADDING`/`info_page_scrollbar_rect`/`info_page_scroll`/`info_page_check_scrollbar` `:28-109`, `BestiaryInfoCategory` + `bestiary_info_category` + `info_page_row_order` `:11-20`/`:128-201` — the immediate row order stays the documented stand-in) |
| 11 | `UIBestiaryFilteringOptionsGrid` | LANDED (`ui/bestiary.rs`: `filter_grid_display_settings`/`filter_grid_container_size`/`filter_button_top_left` `:43-60`/`:159-173` with the dead sqrt/centring walks pinned, `filter_button_selection` `:175-190`, `filter_is_available` `:192-207`, `unavailable_filter_button` `:118-146`; the filter UI itself lands with the filters/sort work) |
| 12 | `UIBestiaryInfoItemLine` | LANDED (`drop_rate_text` pre-landed `GetDropInfo` `:78-110`; missing halves `ui/bestiary.rs`: `info_item_line_rect` `:26-31`, `info_item_locked_icon_rect` `:36-55`, `info_item_line_border` `:34`/`:158-167`, `bestiary_notes` `:137-156`, `info_item_shows_tooltip` `:112-126`; CompareTo = the batch-1 `compare_manual_order`) |
| 13 | `UIBestiaryInfoLine` | LANDED (`compare_manual_order` batch 1 = `CompareTo` `:92-99`; missing halves `ui/bestiary.rs`: `info_line_text_size` `SetText` `:73-81`, `info_line_text_pos` `DrawSelf` `:83-90`) |
| 14 | `UIBestiaryNPCEntryPortrait` | LANDED (portrait draw pre-landed `draw_portrait`/`portrait_layout`; missing halves `ui/bestiary.rs`: `NPC_PORTRAIT_SIZE`/`npc_portrait_inner` `:17-28`, `npc_portrait_overlay_color` `:53` — the 216 px info-page card stays the documented stand-in) |
| 15 | `UIBestiarySortingOptionsGrid` | LANDED (`ui/bestiary.rs`: `sort_grid_container_size`/`sort_button_rect` `BuildGrid` `:37-74`, `sort_click_option` re-click fallback `:89-95`, `sort_button_colors` shares `filter_button_selection` `:100-107`, `sort_grid_entries_to_show` `:117-122`; the sort UI lands with the filters/sort work) |
| 16 | `UICharacter` | LANDED (59x58 box/anchor/animation row pre-landed `ui/char_creation.rs`; missing halves: `preview_pet_shoot` `PreparePetProjectiles` `:55-66`, `preview_anchor_with_pet` `:122-130`, `preview_pet_position` `DrawPets` `:139-142`, `preview_held_item_swap` `:116-119` — the card preview draws no pets/held item yet, documented) |
| 17 | `UICharacterListItem` | LANDED (card geometry/colours/labels/hit/draw pre-landed `ui/char_select.rs`; missing halves: `compare_card_order` `CompareTo` `:285-292`, `renamed_player_name` `:208-214`, `load_status_name`/`card_name_text` `:324-330` + `ID/StatusID.cs:25-32`, `playtime_text` `:376-378` — the draw pins `00:00:00` until play-time tracking lands) |
| 18 | `UICharacterNameButton` | LANDED (spliced DrawSelf + title/value draw pre-landed `draw_info_menu`/`world_creation::draw_plate`; missing halves `ui/char_creation.rs`: `NAME_BUTTON_SIZE`/`NAME_VALUE_LEFT_INITIAL` `:36-37`/`:56`, `name_value_left` `SetContents` `:98`, `name_value_display` `:88-97`, `trim_last_character` + `trim_display_to_fit` `:106-131` (`Utils.cs:2839-2855` surrogate-safe cut) — the plates draw raw strings today, documented deviation) |
| 19 | `UIClothStyleButton` | LANDED (44x80 lattice + spliced base/inset-3 rim/hover border pre-landed `draw_cloth_styles_menu`; missing halves `ui/char_creation.rs`: `CLOTH_STYLE_SIZE` `:37-38`, `cloth_style_preview_rect` the centred 59x58 child `:42-46`, `cloth_style_draw_variant` the Draw swap `:50-63`, `cloth_style_is_selected` `:81`, `cloth_style_preview_animated` `:96-108` — the hover sound latch is the shared `elements::hover_sound_tick`; the menu draws no per-variant bodies, documented deviation) |
| 20 | `UIColoredSlider` | LANDED (bar/knob/value geometry + the aimed/locked latch pre-landed `char_creation::hsl_slider_*`; missing halves `ui/sliders.rs`: `colored_slider_bar_anchor` `:54-76`, `colored_slider_hover` usage override `:60-68`, `colored_slider_hit_rect` `:112-121`, `colored_slider_highlight` gate `:122-131`, `colored_slider_read` wasInBar+clamp `:144-158`, `colored_slider_usage_sound` latch `:80-92`) |
| 21 | `UIColoredSliderSimple` | LANDED (`ui/sliders.rs`: `SIMPLE_SLIDER_FILLED_DEFAULT`/`SIMPLE_SLIDER_EMPTY_DEFAULT` `:11-13`, `simple_slider_fill_rects` the (5,4,4,4) inset + truncating fill `:23-36` — the bestiary completion bar's flat fill stays the documented stand-in until adoption) |

## Conventions (landed style, per research/18 + S1-E-26 wave B)

Code-only lane: ports are **pure helpers + tests-as-code** (written, not run —
no cargo in this lane), one module per family under `crates/tr-app/src/ui/`,
every ported block carries `// File.cs:line` citations, `anchors.txt` rows per
landed family, docs updated (this file + PORT_MAP + research/18 status rows).
Screens keep their immediate-mode draw; the element ports supply the C# class's
*state machine and geometry* as pure functions the screens can adopt later
(the S1-H-13 `panel::scrollbar_*` pattern).
