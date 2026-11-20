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
| 19 | `UIClothStyleButton` | pending |
| 20 | `UIColoredSlider` | pending |
| 21 | `UIColoredSliderSimple` | pending |

## Conventions (landed style, per research/18 + S1-E-26 wave B)

Code-only lane: ports are **pure helpers + tests-as-code** (written, not run —
no cargo in this lane), one module per family under `crates/tr-app/src/ui/`,
every ported block carries `// File.cs:line` citations, `anchors.txt` rows per
landed family, docs updated (this file + PORT_MAP + research/18 status rows).
Screens keep their immediate-mode draw; the element ports supply the C# class's
*state machine and geometry* as pure functions the screens can adopt later
(the S1-H-13 `panel::scrollbar_*` pattern).
