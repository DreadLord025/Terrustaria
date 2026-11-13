# UI — the original menu and settings screens (M7)

Terraria has two menu systems: the legacy immediate-mode `Main.DrawMenu` (`Main.menuMode`
selects a screen; every screen is a flat list of strings drawn with `FontAssets.DeathText`
around `Main.screenWidth / 2`) and the modern `UIState` screens (`Main.menuMode == 888`).
The legacy screens live in `crates/tr-app/src/menu/`; the UIState front-end — character
and world select + create, the virtual keyboard, the flow between them — in
`crates/tr-app/src/ui/` + `app/flow.rs` (see "Front-end screens (P6)" below). Both follow
their source code path so the layout, text sizes and spacing match the original.

## Sources

| Screen | `Main.menuMode` | Items |
| --- | --- | --- |
| Main menu | `0` | `Lang.menu[12..15]`, `[131]`, `UI.ResourcePacks`, `UI.Credits` |
| Settings | `11` | `Lang.menu[114]`, `[210]`, `[63]`, `[65]`, `[218]`, `[219]`, `[103]`, `[5]` |
| General | `112` | `Lang.menu[67/68]`, `[69/70]`, `[112/113]`, `[211/212]`, `UI.PlayWhenUnfocused*`, `[5]` |
| Interface | `1112` | `Lang.menu[71/72]`, `[123]+[124..126]`, `[128/129]`, `Lang.inter[116/117]`, `[229/230]`, `[241/242]`, map border, health style, portraits, boss-bar numbers, `UI.FlashyEffects*`, `[5]` |
| Video | `1111` | `Lang.menu[51]`, `[52]`, `[247..249]`, `UI.LightMode_*` (`Lang.menu[55..58]`), `[59..62]`, `[100/101]`, `[132/133]`, `[250/251]`, `UI.TilesSwayInWind*`, `UI.Effects`, `[5]` |

The strings are the English `LegacyMenu`/`LegacyInterface`/`UI` values from
`Terraria.Localization.Content.en-US*.json`; the indices above are the `Lang.menu[...]`
indices used by the original code, so a future localization pass can swap them for
`Language.GetTextValue` keys without touching the layout.

## Layout (`Main.DrawMenu`)

Every screen sets three constants, reproduced in `menu::Page::layout`:

| Screen | `num2` (first row y) | `num4` (row pitch) | `array7` (text scale) |
| --- | --- | --- | --- |
| Main | 220 | 52 | 1.0 |
| Settings | 210 | 37 | 0.75 |
| General | 250 | 52 | 0.78 |
| Interface | 210 | 28 | 0.6 |
| Video | 186 | 30 | 0.6 (Back row: 1.0) |

* `array4` (per-row y offset): Settings Back `8`, General Back `18`, Interface Back `16`,
  Video Back `8`. The Video screen only assigns `array7 = 0.6` to its first ten rows, so
  its Back row stays at `1.0` — that quirk is reproduced.
* Rows are centred on `Main.screenWidth / 2` with `origin = MeasureString(text) / 2`; the
  row y is `num2 + num4 * i + array4[i]`, and the text is drawn with
  `origin.Y * array7` as the vertical anchor.
* `menuItemScale` eases from `0.8` to `1.0` by `0.02` per frame for the focused row and
  back for the others. The text alpha is `255 * (menuItemScale * 2 - 1)` (so unfocused
  rows are at 60 % alpha), the focused row is lerped to `(255, 215, 0)`, and every row is
  drawn five times: four black copies at `(-2,0)`, `(2,0)`, `(0,-2)`, `(0,2)` plus the
  coloured pass — the original's shadow.
* The logo is drawn at `(screenWidth / 2, 100)` with its centre as origin; `logoScale`
  eases between `1.0` and `1.35` and `logoRotation` between `±0.08` rad with the original
  speed ramp (`±50` and `±20` per frame).
* `Main.DrawVersionNumber`: `v1.4.5.8` in `Mouse_Text` at the bottom-left, 10 px from the
  left edge and 2 px from the bottom.
* Mouse hover selects the row under the cursor (the same `±width/2`, `row_y ..
  row_y + 50 * array7` box as the original) and a left click activates it; the arrow keys
  move the same cursor for keyboard play. Until the first hover/keypress no row is
  focused, which is how the original starts.

## Settings (`settings.rs`)

`Main.SaveSettings` writes `config.json` in the game root; this port writes to
`Documents/My Games/Terrustaria/config.json` and reads it back **once** at start
(`main.rs` loads it and threads it into `App::new` and `Menu::new`; the menu holds
a working clone and the close copies the edited values back into `App.settings`).
Unknown keys are ignored and missing keys keep their default, which is the vanilla
`static Main()` set (volumes `1f`, `Event Progress Bar: On`, `Pause When Unfocused:
On`, `Frame Skip Subtle`, …), so a fresh install shows the original's out-of-box
labels. The writer uses the **vanilla key names** (so a vanilla `config.json` is
readable) and `parse` additionally accepts this port's older spellings
(`PickupText`, `MusicVolume`, `HidePassword`, …) as aliases:

`AutoSave`, `AutoPause`, `Language`, `MapEnabled`, `HidePasswords`,
`PlayWhenUnfocused`, `ShowItemText`, `InvasionBarMode`, `PlacementPreview`,
`HighlightNewItems`, `MouseShowBuildingGrid`,
`GamepadDisableInstructionsDisplay`, `BackgroundParallax`, `FrameSkipMode`,
`LightingMode` (persisted as the C# ordinal: `White 0, Retro 1, Trippy 2, Color 3`),
`GraphicsQuality`, `BackgroundEnabled`, `GoreVisualsAllowed`,
`SettingsEnabled_MinersWobble`, `SettingsEnabled_TilesSwayInWind`,
`UseStormEffect`, `UseHeatDistortion`, `ForegroundSunlightEffects`, `WaveQuality`,
`FlashyEffectsWorld`, `UseScreenShake`, `Zoom`, `VolumeMusic`, `VolumeSound`,
`VolumeAmbient`, `MapScale`, `MouseColor` (R/G/B dict), `MouseBorderColor`
(R/G/B/A dict), `SmartCursorToggle`, `UseSmartAxeAfterSmartPickaxe`,
`UseSmartCursorForCommonBlocks`, `CraftFromNearbyChests`, `LockOnPriority`,
`ReverseUpDownForArmorSetBonuses`, `DisableLeftShiftTrashCan`, `DisableQuickTrash`,
`PlayerResourcesSet`, `PortraitPreference`, `ShowBossBarHealthText`,
`FlashyEffectsInterface`, `MinimapFrame`.

The renderer reads `LightingMode` (applied when the menu closes, through
`LightMode::ALL`), `BackgroundEnabled` (the parallax layer gate in `draw_sky`) and
`MapEnabled` (minimap/full map); the volumes feed `audio`/`sound` (including
M1.4-BG-20's ambience chain), `ShowBossBarHealthText` gates the boss bar's
`current / max` numbers and `MapScale` scales the minimap panel. The rest is stored
and shown in the labels, ready for the systems that will consume it — the flags
whose system the port does not have yet (`blood` without a gore system, the
smart-cursor flags without the outline) stay stored-only and are recorded in
`docs/ANALYSIS.md` → "Menu sub-screens (P5)".

## Menu sub-screens (P5)

Every `Main.DrawMenu` settings branch below menuMode 11 is ported as one `Page` in
`menu/`, keyed by its original `menuMode` — which is also accepted as a `TR_PAGE`
name so each screen can be opened directly. The shared slider chrome is
`ui/sliders.rs`: the `IngameOptions.DrawValueBar` port (ColorBar 178×16 track,
167 ColorBlips 2×8 graded with the truncating `ColorLerp_BlackToWhite`, the 11×25
ColorSlider knob at `rect.x + 5 + 167·value`, the hit box inflated −5 px
horizontally, and the `UI/Slider_Highlight` overlay tinted `OurFavoriteColor
(255, 231, 69)` while hovered) and the HSL colour-slider block (Hue + 168 blips per
row, tops `num9 + 0/26/52/78` from `num9 = 320`, left edge `screenWidth/2 − 89`,
176 px knob travel, hit inflated 4 px; the alpha row's blips are
`hsl_to_rgb(h, s, l, 255) · i/167`).

| Screen | `menuMode` | `num2`/`num4`/`array7` | Rows |
| --- | --- | --- | --- |
| Volume | `26` | 200 / 10 / 1.0 | blank, "Volume" header @220, Back @470 → Settings |
| Parallax | `28` | 240 / 60 / 1.0 | blank, "Parallax" header @310, Back @530 → Video |
| Cursor | `1125` | 232 / 38 / 0.73 | Cursor Color, Border Color, Smart Cursor Mode/Priority/Placement, Lock On Priority, Back @478 |
| Cursor Color | `25` | 200 / 10 / 1.0 | header @220, Back @470; HSL sliders with luminosity floored at `0.15` and no alpha bar |
| Border Color | `252` | 200 / 10 / 1.0 | the 25 twin: full luminosity range and the alpha bar |
| Resolution | `111` | 210 / 55 / 0.85 | Fullscreen Resolution @210, Borderless Window @265, Go Fullscreen @320, Apply @475, Back @530 |
| Effects | `2008` | 210 / 28 / 0.6 (header 0.7) | header @202 (`array4 = −8`), six effect rows, Back @406 — the branch declares 9 rows but leaves row 8 null and the item loop skips it |
| Language | `1213` | 200 / 26 / 0.65 | "Select language" @180 (scale 0.85), 12 languages, Back @547 (scale 0.8) |
| Language Initial | `1212` | 200 / 30 / 0.65 | the first-launch picker: header scale 1.0, 12 languages, no Back |
| Controls | `1127` | 250 / 52 / 0.78 | Set Bonuses @250, Quick Trash @302, Keybindings @354 (notice — the original opens the 888 UIState), Back @424 |
| Keybindings | `27` | 176 / 22 / 0.45 | 14 left-aligned bind rows (`array5 = −80`) + Reset + Back, both scale 0.8 |
| Map Keys | `272727` | 200 / 30 / 0.55 | 6 left-aligned bind rows (`array5 = −140`) + Reset (0.8) + Back (scale **1.0** — the branch never sets `array7[7]`) |
| Multiplayer | `12` | 250 / 80 / 1.0 | Join via IP @250, Join via Steam @330, Host & Play @410, Back @490 — `num41 = 1` (the retail build ships Steam), so `num5 = 3 + num41` = 4 rows; the `num41 == 0` build overwrites row 1 and drops to three |
| Join IP | `13` | 180 / 30 / 1.0 | header @180, input @229 (`array4 = 19`), 7 blank recent rows @280..460 (scale 0.6), Accept @494, Back @544 |
| Join Port | `131` | 250 / 80 / 1.0 | header @250, input @310, Accept @430 (disabled unless the text parses to 1..=65535), Back @490 |
| Host Password | `30` | 250 / 80 / 1.0 | header @250, masked input @310, Accept @430 → Host (fixed port 7777), Back @490 |
| Join Password | `31` | 250 / 80 / 1.0 | header @250, masked input @310, Accept @430 → Join, Back @490 |
| Server Settings | `889` | 200 / 60 / 1.0 | header @200, Steam Multiplayer toggle @290, two blank lobby rows, Start @510, Back @570 |

* **Slider drag is draw-time and sticky.** The bars draw *and* drag in one pass,
  exactly like `DrawValueBar`/`DrawMenu`: the bar renders at the stored value,
  then the drag pass stores `(mouseX − rect.x) / rect.width`, so the knob trails
  the pointer by one frame — the original's order. `Menu::bar_grab` /
  `Menu::color_grab` mirror `IngameOptions.rightLock` / `Main.grabColorSlider`:
  the lock sticks to the bar while the button is held even outside it, and clears
  on release or page change. Volume labels sit at `370 + screenWidth/2 − 400 + 37`
  (x 807 at 1600×900: "Music:" @320, "Sound:" @350, "Ambient:" @380, each value
  90 px right); the bar right edge is `screenWidth/2 − 40 + 37` (797) with centres
  y 332/362/392, and the percentages print `Math.Round(v·100)` with ties-to-even
  (`0.125` → `"12%"`). Parallax draws `"Parallax: {bgScroll}"` at (770, 400) with
  its bar right edge at 760, centre y 412.
* **The colour screens** copy `selColor` into `mouseColor` / `MouseBorderColor`
  every frame, seed the bars from `rgbToHsl` while any is still `−1`, and
  recompose `selColor` from the bars at the end of the pass; the cursor tint reads
  the stored colour per frame, so dragging re-colours the pointer live. A config
  write lands once per drag, on release.
* **The input rows are headers with a blinking cursor**, not boxes: the
  IP / port / password rows are non-clickable, append `|` on a 20-frame blinker
  (nudging the row one pixel right, `array5[1] = 1`) and a space otherwise.
  Passwords render `*`-masked while "Passwords: Hidden" is set, Enter submits from
  any row, and the port keeps separate host/join password strings where vanilla
  shares one.
* **Keybind capture** (27/272727): clicking a bind row shows `"_"`, the next key
  press stores it and closes the capture, Reset restores the vanilla defaults;
  the lists are left-aligned at `array5` with the raw `Lang.menu` values,
  trailing spaces included.

## Front-end screens (P6) — the UIState half (`ui/`, `app/flow.rs`, `menu/`)

The character/world select + create flow is the modern half of the same single-mode
invariant: while `Main.menuMode == 888` is live (`Main.cs:51059-51069`), the screen is a
`UIState`, not a `Main.DrawMenu` branch. The port keeps the same shape:

* `app/flow.rs` owns the `FrontScreen` enum — `CharSelect`, `WorldSelect`,
  `CharCreation(CreationState)`, `WorldCreation(NewWorldDraft)`,
  `Keyboard(Box<KeyboardSession>)` — held in `App.front` while `App.menu` is `None`
  (exactly one side is live, never both), the pure `next_flow_state` click-routing table,
  and the `FrontScreenBehaviours` trait: per-screen `header`/`draw`/`click`/`key`
  overrides — the `UIState` `DrawSelf` + `Click_*` handler-table shape (see
  `docs/ARCHITECTURE.md` → "C# seams"). The keyboard session carries its `back` target,
  so Cancel/Submit tails reopen the screen the rename came from.
* `ui/uistate.rs` is the element kit the screens share: `UIElement.cs:306-322`'s resolve
  order (offset → clamp → align), `UITextPanel` (the MinWidth-sized header panels),
  `DrawSplicedPanel` (the 3-piece horizontal splice of `UICharacterListItem.cs:310-315`),
  the `UIColoredImageButton` pair (border = hovered, highlight = selected) and the gated
  12-size font pin. `ui/cards.rs` holds the fat-button/card pieces the select screens
  share.

### Select screens (`ui/char_select.rs`, `ui/world_select.rs`)

`UIWorldSelect` delegates to `UICharacterSelect`'s constructor, and so does the port.
Geometry at 1600×900 (`UICharacterSelect.cs:38-84`):

* root `(475, 220, 650, 680)` (Width `0.8·screen` → MaxWidth 650, HAlign 0.5; Height
  `−220`, Top 220), panel `(475, 220, 650, 570)` (Height `−110`);
* the card list resolves against the panel's 12 px inner padding — `(487, 232, 626, 546)`,
  detached width `650 − 24`; the 20 px scrollbar attaches at HAlign 1 of the inner
  `(1093, 232, 20, 546)` only from the seventh card on (`UIScrollbar.cs:49-82`);
* the header is **MinWidth-sized**, not the 650 hint rect: `100·0.8 + 30` wide,
  `32·0.8 + 30` high, HAlign 0.5, Top −40 → `(745, 180, 110, 55.6)` at the test's 100 px
  measure;
* Back and New are `0.5·650 − 10 = 315` wide, 50 high at the root's bottom − 45:
  `(475, 805)` and `(810, 805)`;
* cards (`UICharacterListItem`/`UIWorldListItem`): the 3-piece inner panel, name label,
  divider at x 557, the five `Button_*` sheets mapped by `button_sheet` (the ctor ordinal
  is not the sheet index — Rename is 3), the favourite star, and for worlds the
  difficulty/size/created/played panels (divider row `(557, 259, 556, 4)`, size panel
  width 668, the played icon centred at ImageScale 0.75) plus the completion/
  mixed-seed overlay rects.
* Hit order mirrors `UIElement.GetElementAt`'s reverse-child iteration: New → Back win
  over the cards. The listing, sorting and erase behaviour underneath is
  `ui/save_list.rs` (`Main.LoadPlayers`/`LoadWorlds`, `ErasePlayer`/`EraseWorld`, the
  `UICharacterSelect.UpdatePlayersList`/`UIWorldSelect.UpdateWorldsList` orderings).

### Creation screens (`ui/char_creation.rs`, `ui/world_creation.rs`)

* Character creation (`UICharacterCreation`): 500×384 root at `(550, 220)`, categories
  bar inner `(550, 274, 500, 46)` (ten `UIColoredImageButton` pickers), the animated
  80×80 player preview drawn with the real player UV machinery (anchor `(790, 219)`,
  hitbox half, unscaled), the Info / HSL / Hairstyles / Cloth-styles middle menus (the
  hue-0.5 blip is cyan, the hair scrollbar `(1025, 345, 20, 127)`, the hex panel
  `(810, 455, 100, 32)`), Back `(550, 509, 240, 50)` and Create `(810, 509, 240, 50)`.
  The draft is the `Main.PendingPlayer` port (`NewPlayerDraft`: name, difficulty, hair,
  cloth style, the six colours, `dirty`); Back's discard confirm (`mode 40`) parks the
  whole `CreationState` in `App.front_backup` (`BackupConfirmationState`,
  `UICharacterCreation.cs:1215`).
* World creation (`UIWorldCreation` + `UIWorldCreationPreview`): 500×452 root at
  `(550, 152)`, panel `(550, 202, 500, 298)`, the name/seed plates `(604, 210/254, 348, 40)`
  with their randomize icons, the three `GroupOptionButton` rows (size / difficulty /
  evil), the 84×84 preview stack (`PreviewDifficulty*` + evil-icon sheets, idle icon tint
  White), the description panel (y 450) that swaps draw-side from the hover, Back
  `(550, 509, 240, 50)` / Create `(810, 509, 240, 50)`. The draft (`NewWorldDraft`)
  carries name/seed/size/difficulty/evil; Create really generates and saves a `.trwld`.
  A numeric seed parses `int` (i32 + Abs), anything else goes through the
  `ReLogic.Utilities.Crc32` port of the name — the stand-in for `Main.rand`'s composition
  (see `docs/ANALYSIS.md` → "Front-end screens (P6)").

### Virtual keyboard (`ui/keyboard.rs`, `UIVirtualKeyboard.cs`)

The rename flow's keyboard draws the **desktop** layer (no gamepad:
`PlayerInput.SettingsForUI.ShowGamepadHints` is false without one): the label
`(542, 229, 516, 37)`, the textbox `(542, 260, 516, 48.96)` (`MinHeight = 32·0.78 + 24`),
Submit/Cancel, and the 50-key `UITextPanel` grid whose origin `(542, 282)` is an exact
fit — 10·48 + 9·4 = 516 and 5·37 + 4·4 = 201 fill the panel inner exactly. The grid
layer is kept behind `TR_VK_GRID=1`. Rename caps: 20 characters for players, 27 for
worlds (`Click_SetName`, `UIWorldCreation.cs:693`); `allowEmpty` differs per caller
(Create passes `false`).

### Credits and legacy confirms (`menu/`)

The pages the flow parks onto are still legacy `Main.DrawMenu` branches: `menuMode`
5/9/40 confirms ("Delete {name}?" / "Discard your new player?") at the 250/80 layout
with Yes/No rows — `Action::{ConfirmYes, ConfirmNo}` run the erase and re-open the
screen the confirm came from — and `menuMode` 3000 credits, a single Back row at y 500
(scale 0.9) over the world backdrop; the `SkyManager.Activate("CreditsRoll")` cinematic
is not ported (documented deviation). Multiplayer's "Join via IP" and "Host & Play" still
open the input screens directly (the P5 shortcut) — the join/host paths through the
character select stay with a future multiplayer-flow task — and "Join via Steam" reports
the not-ported notice because the port has no `SocialAPI`/Steam session to hand
`SocialAPI.Friends.OpenJoinInterface()` to (`Main.cs:50726-50730`).

### Front-end dev hooks

`TR_UI=<screen>` opens a front screen directly — `charsel`, `worldsel`, `charnew`,
`worldnew`, `vk`, `credits`, `delplayer`, `delworld`, `discard` (the legacy pages the
flow parks onto); `TR_FAKE_CHARS`/`TR_FAKE_WORLDS=<n>` prepend fake list entries so the
cards are screenshotable; `TR_VK_GRID=1` layers the keyboard's 50-key grid.

## UI framework (`ui/pointer.rs`, `ui/framework.rs`, `ui/select_hit.rs`)

The two halves of `UIElement`/`UserInterface` the screens share, besides the layout
pass in `ui/uistate.rs`. Screen-model ruling (unchanged): this is an **immediate**
port — no retained `UIElement` tree and no `UserInterface` object; the screens keep
drawing and hit-testing themselves, and the framework supplies the two things that
cannot live per-frame: the hit *order* and the click *timing*.

* `ui/pointer.rs` — the `UserInterface` pointer machine (`UserInterface.cs:53-91`,
  `:230-255`): the per-button caches (`WasDown`/`LastDown`/`LastClicked`/
  `LastTimeDown`), the 500 ms double-click window (`:41`) with the `LastClicked`
  clear (`:236-240`), the release-outside-the-element rule (`:243-251`, `MouseUp`
  and no click), `ResetState` with `ImmediatelyUpdateInputPointers`' `WasDown`
  re-sync (`:158-163`, `:165-179`), and the 200 ms
  `STATE_CHANGE_CLICK_DISABLE_TIME` (`:43`) armed on a front-screen entry
  (`enter_front`). The clock is `App::anim_time` in ms — deterministic under
  `TR_SHOT`, which pins the frame step to 1/60 (`frame.rs`).
* `ui/framework.rs` — `get_element_at` (`UIElement.cs:324-352`): the back-to-front
  walk over a flat `Element` list, `IgnoresMouseInteraction` per element, and
  `contains_point` reusing `in_rect`'s inclusive/exclusive edges. The port's
  screens are flat, so the C#'s recursion (`:341`) is not modelled (the module doc
  says so); `PassThroughMouseInteraction` is declared but never set in 1.4.5.8, so
  there is no pass-through flag and the first hit always wins.
* `ui/select_hit.rs` — the element ids the two select screens' hit order needs
  (the C#'s `UIElement` references that `UserInterface`'s `InputPointerCache`
  holds). Card *i*'s body and its four buttons take consecutive ids, and the
  screens list them in the source's **append** order
  (`UICharacterListItem.cs:71-121`, then `Back`, then `New`) so the reverse walk
  gives `New`/`Back` a click over an overlapping card row.
* **Double-click play**: a card's `OnLeftDoubleClick` → `PlayGame`
  (`UICharacterListItem.cs:68-69`, `UIWorldListItem.cs:69`/`:99`) is the only
  double-click user in 1.4.5.8. It needs the same element hit twice inside 500 ms,
  which a per-frame recompute cannot answer — hence the ids. `card_hit` takes a
  `double_click` flag and resolves it to `CardAction::DoubleClickPlay(i)`, routed
  through new indexed `FlowClick::CharPlay(usize)`/`WorldPlay(usize)` arms.
  **Routing only**: the port has no per-card selection state and its play path
  closes the front flow instead of loading the entry (spec §9.4), so the index is
  information, not a load. The world card's `PlayGame` acceptance gate (`IsValid`
  + `TryMovingToRejectionMenuIfNeeded`, `UIWorldListItem.cs:362`/`:387-410`)
  lands in `world_select::rejection_key` + `flow.rs::world_play_click`
  (S1-H-10): a refused click sets `App::front_hint` instead of launching.

## In-game interface (`crates/tr-app/src/ui/`)

The in-game HUD is immediate mode in the original too: `Main.DrawInterface` calls
`DrawInterface_25_ResourceBars` → … → `DrawInterface_36_Cursor` in one pass, with the
constants hard-coded inside `ClassicPlayerResourcesDisplaySet` / `Main.DrawInventory` /
`Main.MouseText`. The port mirrors that: each panel is a function named after the
original, in its own `ui/` module, called by `ui::draw_hud` in the vanilla pass order
(resources 25 → inventory 27 → hotbar 30 → mouse-text 33 → cursor 36).

| Module | Original | What it draws |
| --- | --- | --- |
| `ui/resources.rs` | `ClassicPlayerResourcesDisplaySet.DrawLife/DrawMana` | hearts, mana stars, "Life x/y" / "Mana" text, breath bubbles |
| `ui/cursor.rs` | `Main.DrawCursor` | the vanilla `Cursor_0` at its pulse scale |
| `ui/tooltip.rs` | `Main.MouseText` + `Utils.DrawInvBG` | the 9-slice tooltip box, rarity-coloured name; `draw_mouse_text` is the single-line `MouseTextHackZoom` shape (the PVP/team hover) |
| `ui/slot.rs` | `ItemSlot.Draw` | per-context background, icon, stack text, hotbar digit, empty-equip glyph — one port shared by the open grid (context 0), the closed hotbar (context 13, `GUIHotbarDrawInner`, `Main.cs:43600-43639`) and the cursor stack (context 21, `Main.cs:45173-45195`) |
| `ui/inventory.rs` | `Main.DrawInventory` | 10×5 grid, coins/ammo, equipment columns, misc page, trash, labels |
| `ui/chrome.rs` | `DrawPageIcons`/`DrawDefenseCounter`/`DrawGuideCraftText`/`DrawPVPIcons` + `Main.DrawInventory`'s map-style row | page tabs (**drawn and clickable**: per-tab mouse-item gates, the same-tab→page-0 toggle, `EquipPageSelected` + `ItemSlot.SelectEquipPage`), defence shield, guide text, the PVP toggle + six team-select cells (drawn **and** hit-tested: hover text, `teamCooldown` dim, `PVP_2` hover ring, the click chain), the four map-style icons |
| `ui/minimap.rs` | `MinimapFrame`/`MinimapFrameManager`/`MinimapFrameTemplate` (+ `SelectionHolder.CycleSelection`) | the minimap frame chrome: 9 style table, backdrop, frame + button hover sheets, zoom buttons |
| `ui/bars.rs` | `BigProgressBarHelper.DrawFancyBar` (+ `DrawHealthText`) | the boss health bar: 516×58 `UI_BossBar` rows, 456×22 bar, boss head icon, `current / max` text |
| `ui/crafting.rs` | `NewCraftingUI.cs` (+ `UIWrappedSearchBar`, `UICreativeItemsInfiniteFilteringOptions`, `CraftingUI.DrawGridToggle`) | the modern crafting panel: frame, title, search bar, filter strip, separator, empty grid, scrollbar, detail area, toggles |
| `ui/chest.rs` | `Terraria/UI/ChestUI.cs` | the world-chest panel: name line, rename buttons, the Loot All/Deposit All/Quick Stack/Restock button row, the 10×4 chest slot window, context-3 slot backgrounds |

Key geometry (1600×900, `UIScale = 1.0`, `Main.inventoryScale = 0.85`):

* Hearts start at `screenWidth − 300` and step 26 px; mana stars at `775 + anchor`.
* Grid slot `(i, j)` is at `(20 + i*56*0.85, 20 + j*56*0.85)` with a hard-coded
  56 px pitch; the drawn cell is the sheet's own `Size()*scale` (52 px sheet).
* Coins and ammo use `inventoryScale = 0.6` at x `497` / `534`, rows at
  `(int)(85 + k*56*0.6 + 20)`.
* Equipment columns are right-anchored at `screenWidth − 92 / − 139 / − 186`
  (armour, vanity, dye) from `num20 = 174 + Main.mH` (`Main.cs:41146`) — `mH` is
  `256` while the minimap HUD is live (see the minimap section), so the block
  sits at `num20 = 430` at the reference layout, `+4` on the rows past the third.
* The misc-equipment page uses pitch `47`; trash is at `(448, 258)`; the page tabs
  sit at `screenWidth − 80 / − 128 / − 176`.
* Closed hotbar: x starts at 20 and advances `(int)(52*scale) + 4` per slot, row
  `y = (int)(20 + 22*(1 − scale))` with `hotbarScale` lerping `0.75 ↔ 1.0` at
  `±0.05`/frame (`Main.cs:43604-43614,43639`); every element is tinted
  `Color(255, 255, 255, (int)(75 + 150*scale))` (`Main.cs:43615`). Digits —
  `(8, 4)*scale` on non-empty slots only (`ItemSlot.cs:2693-2701`) — versus the open
  grid's always-drawn `(6, 4+num29)*scale` digits (`ItemSlot.cs:2753-2771`). The
  cursor stack rides context 21 at `inventoryScale = Main.cursorScale`
  (`Main.cs:45190-45194`).

### Crafting panel (`ui/crafting.rs`, `NewCraftingUI.cs`)

The panel opens with the inventory: a 490 px-wide cyan `UIPanel` frame at
`(20, 312 + 38)` that fills down to `screenHeight − 350` (`NewCraftingUI.cs:300-325`),
everything laid out against the panel's 12/8 px padding. Draw order follows the
source's child order:

* the white "Crafting" title at the panel's top-left, 4-direction shadow at
  spread 1.5 (`UIText.cs:138`);
* the right-aligned search bar, its width inset by the title's measured width
  (`:424`): the 22 px magnifier (`Button_Search_2`), the 24 px `Name:` text field
  and the 24 px cancel button (`UIWrappedSearchBar.cs`);
* the 4 px `UIHorizontalSeparator`, then the empty recipe grid
  (`Recipe.maxRecipes == 0`) with its auto-hidden scrollbar;
* the 11-tab filter strip (40×46 tabs cut from the 84×192 `Infinite_Tabs_B_2`
  sheet, 28×30 icons from the 330×30 `Infinite_Icons` sheet; strip at
  `(45, 316)`, above the panel body) with the non-sequential `ItemFilters.GetImage`
  frame table;
* the detail area — craft slot (`Inventory_Back18` context 42) plus the
  `Images/UI/Craft` arrow at 0.8 scale — and the grid/nearby-chests toggles at
  `(42, 285)` / `(82, 285)` with the `DrawSelectedCraftingBarIndicator` bars.

Deviations: draw-only chrome (no filter/search/scrollbar interaction), the detail
area renders without a selected recipe, the cancel button never hides — the full
list is in `docs/ANALYSIS.md` → "Crafting panel".

### Chest panel (`ui/chest.rs`, `Terraria/UI/ChestUI.cs`)

The panel draws while a world chest is open (`player.chest >= 0` — the unified
open-container index, `App.chest` is gone; `OpenChest` also raises
`playerInventory` — `Player.cs:33517`, so the chest slots render in front of the open
inventory grid). There is no panel background texture — the panel is text plus
`Inventory_Back*` slot frames (`ChestUI.cs` draws no `UIPanel`). Key geometry
(1600×900, `UIScale = 1.0`, `Main.inventoryScale = 0.755` for the whole panel —
`ChestUI.cs:88`/`:363`):

* the name line at `(504, 258)` (`ChestUI.cs:163-164`) — the stored chest name, else
  the tile-style default (`ChestUI.cs:124-158`; the port derives the strings from
  `itemdata::ITEMS` — ANALYSIS for the localization deviation); while renaming, the
  caret blinker rides the same line (`:165-184`) and the `inter[47]` "Save" label sits
  after the text;
* the rename text buttons at `x = 506`, `y = invBottom(258) + 40 + ID·26`
  (`ChestUI.cs:191`/`:203`) — IDs 5 RenameChest and 6 RenameChestCancel render
  (`ChestUI.cs:20-42`; the port's `ButtonID` table matches all of 0-6); IDs 0-3
  render too — Loot All / Deposit All / Quick Stack / Restock
  (`ChestUI.cs:206-228`) — and work (see the interactions below); anchor 4
  (Sort) stays reserved and label-less;
  cancel is hidden unless the rename keyboard is editing (`:198-202`);
* the 10×4 slot window from `(73, 258)` with the 56 px layout pitch: slot `(i, j)` at
  `(int)(73 + i·56·0.755), (int)(258 + j·56·0.755)` (`ChestUI.cs:379-380`), the drawn
  cell is the 52 px `Inventory_Back` sheet × scale. The row scroll clamps to
  `0..ceil(maxItems/10) − 4` (`ChestUI.cs:311-327`, `Main.cs:17436-17452`) — at the
  default 40 slots that is `0..0` and the wheel gate is dead, 1:1 with vanilla; when
  rows exist, the wheel scrolls only while the mouse is inside the display rectangle
  `(73, 258, 422, 169)` (`ChestUI.cs:364`, `Main.cs:17446`);
* chest slots draw with the context-3 background — the `Inventory_Back5` frame
  (`ItemSlot.cs:2215-2221`), tinted `Main.inventoryBack` like the inventory slots.

Interactions (`app/clicks.rs`): chest slots use the vanilla context-3 semantics — left
is action 0 (take/place/swap/stack, the shared `left_click_movement` extraction the
inventory uses), right is the stack-split (first delay 30, repeat 7, superFast after
~2 s — `Main.cs:61785-61803`, `ItemSlot.cs:1700-1772`); with shift held, the hover
evaluates the transfer cursors — 8 (chest→inventory, `ItemSlot.cs:692-699`/`:800-812`)
and 9 (inventory→chest, `TryPlacingInChest` `ChestUI.cs:1209-1289`: a stack-merge pass
then an empty-slot pass). Re-clicking the open chest's tile toggles the panel closed
(or switches chests) through the UI click path — the editing interception is gated
while the inventory is open.

The transfer buttons: the button dispatch runs before the slot-grid clicks
(`ChestUI.cs:281-308`) and the whole panel (labels, hit, dispatch) gates to
`chest >= 0` — banks keep the STORE-1 panel instead of drawing a second button row.
MoveCoins is hoisted to the dispatch layer: the bank call site runs it before Deposit
All and Quick Stack (`ChestUI.cs:543-562`/`:670-685`), the world chest's Quick Stack
gets none; Loot All routes its overflow into the void vault when it is enabled
(`GetItemSettings.cs:55-56`, `Player.cs:39395-39398`).

Favorites and trash: hovering an inventory slot with LeftAlt held turns left-click
into the favorite toggle (the `ItemSlot.cs:602-625` cursorOverride-3 arm, sound 12,
the `can_favorite_at` gate — the C# binding; the row sketch's right-click wording was
wrong); favorited slots swap their background to the gold `Inventory_Back10`
(`ItemSlot.cs:2149-2156`, packed as `lookups.ui_back10`, tint still
`Main.inventoryBack`; `Inventory_Back17`/`_19` are the gamepad-highlight and
loadout variants — neither surface is ported) and shift-transfer refuses them.
The coin (ctx 1) and ammo (ctx 2) rows are click surfaces too
(`Main.cs:41752-41782` handle every cell they draw): left runs the shared
action-0 move behind `PickItemMovementAction`'s gate (coins 71-74 only; the
ammo row takes `Item.FitsAmmoSlot`'s `ammo`/`bait` set), right splits the stack,
and `ui::inventory::coin_ammo_hit` gives the draw pass and the click pass one
geometry (x 497/534, `(int)(85 + row·56·0.6 + 20)`). Shift+click transfers either
row into the open chest (`GetAlternateClickAction`'s ctx-1/ctx-2 arms reach the
same `TryPlacingInChest` check, `ItemSlot.cs:657-665`) through
`chest::try_placing_from`/`chest::Source`, which reads the row's own array where
the C# reads one `Player.inventory[0..58]`. Alt+left-click favorites either row
(`canFavoriteAt`'s contexts 1/2) in the vanilla key space — grid `0..50`, coins
`50..54`, ammo `54..58` (`inv::INV_SLOTS`) — so a row favorite draws the gold
sheet, refuses the shift transfer and rides the `.plr` bitfield like a grid one
(`:631-634`). The trash slot at `(448, 258)` is clickable: left-click swaps held ↔
trash (`MouseItemSwap` `ItemSlot.cs:1255-1263`, the Grab cue).

Rename: the RenameChest button opens the existing virtual keyboard as
`FrontScreen::Keyboard` mid-game with a `KeyboardTail::ChestRename` session — prefilled
from the stored name or the tile default, submit caps at 20 characters and writes only
when different (an unchanged default name clears to `""`), the panel-cancel closes the
keyboard with the edit discarded (`ChestUI.cs:827-869`). Deviation: the keyboard is a
front screen, so the world sim pauses and the HUD hides while it is open (vanilla draws
the game UI under `InGameUI`) — the full list is in `docs/ANALYSIS.md` →
"Chest UI + world chests (M3-CHEST-UI)".

Screenshots: `TR_CHEST` stages a stocked style-0 chest on the spawn plateau and opens
the panel deterministically (the `TR_INV`/`TR_CRAFT` presence-flag pattern); double
runs are SHA256-equal.

### Minimap frame (`ui/minimap.rs`, `MinimapFrame.cs`)

The frame around the already-ported map tiles is the `MinimapFrame`/
`MinimapFrameManager` system: nine frame styles (`MinimapFrameManager.cs:23-33`,
portrayed element-for-element by `FRAME_STYLES`), the active one picked by the
`MinimapFrame` config key with the `Default` fallback, and the two draw passes
around `MapRenderer.DrawMap`. The `MapScale`/`MinimapFrame` config keys live in
`settings.rs` like the vanilla ones (`Main.cs:4202`, `MinimapFrameManager.cs:13-18`).
Key geometry (1600×900, `MapScale = 1`):

* The top-right anchor is `(52 + (int)(240·MapScale), 90)` = `(292, 90)`
  (`Main.UpdateMinimapAnchors`, `Main.cs:55275-55276`), so the tile panel origin
  is `(1308, 90)` and the `DrawTo` position (what `MinimapPosition` is set to)
  is `(1312, 94)` — origin `− 6 + 10` on both axes (`Main.cs:55546`).
* The black backdrop is `Rectangle((int)MinimapPosition − 6, 244, 244)` =
  `(1306, 88)` 244×244 — the 240 tile panel (drawn at `(1308, 90)`) plus a 2 px
  skirt on each side (`MinimapFrame.cs:181-182`) — tinted
  `Color.Black · Main.mapMinimapAlpha` (alpha 1). It is pushed into the
  `map_back` pass so it lands **under** the tiles (the renderer's fixed order
  is `back → world → map_back → map → ui`).
* The frame sheet draws whole (no 9-slice) at `FramePosition` = `MinimapPosition
  + _frameOffset` (`MinimapFrame.cs:81-91`); Default's offset `(-8, -15)` puts
  the 256×264 frame at `(1304, 79)`. The hovered button's hover sheet is drawn
  over it at `_position + parentPosition` (`Button.Draw`, `:38-44`).
* The three zoom buttons sit at frame-relative raw positions — Default: reset
  `(150, 240)`, zoom-out `(176, 240)`, zoom-in `(202, 240)` (`:23-24` with
  `num = 2, num2 = 6`), i.e. absolute top-lefts `(1454, 319)`, `(1480, 319)`,
  `(1506, 319)`. The hitbox is `Max(button size, (22, 22)) · 0.5` around the
  button centre (`:46-56`) — an 11 px half-extent with a **strict `<`** on both
  axes, so the centres are `(1463, 328)`, `(1489, 328)`, `(1515, 328)` and the
  nominal rectangle edges are outside. `GetButtonUnderMouse` tests zoom-in,
  zoom-out, reset in that order and returns the first hit (`:194-210`).
* Clicks mirror `MinimapFrame.Update` (`:138-177`): the action re-fires every
  frame while the button is held (`Main.mouseLeft`, `:169-171`). Reset pins the
  scale to `1.05`, zoom-in multiplies by `1.025`, zoom-out by `0.975`
  (`:59-63`, `:107-136`), and the draw side clamps to `0.2..3`
  (`Main.cs:55515-55522`), so a held zoom-in saturates at exactly `3.0`.
* `Main.DrawInterface_16_MapOrMinimap` (`Main.cs:45771-45798`) sets
  `Main.mH = 256` while the minimap HUD is live (`mapEnabled && !mapFullscreen
  && mapStyle == 1`), clamped so `mH + 600 ≤ screenHeight`
  (`RecommendedEquipmentAreaPushUp`, `Main.cs:2967-2977`); the equipment block
  reads `num20 = 174 + mH` (`Main.cs:41146`).
* The frame draws before the HUD sprites — vanilla renders interface layer 16
  (the minimap) before layers 25/27/30 (`Main.cs:44796`, `:44831/:44841/:44861`) —
  so hearts and the inventory paint over the frame where they overlap.
* Hovering a button sets the vanilla `mouseInterface` behaviour: mine/place
  inputs are suppressed behind the frame (`MinimapFrame.cs:164`).

Deviations: `MapScale` is `1.0` only (no scale slider — the `CreateScale(MapScale)`
transforms of `Main.cs:55308-55329,55768` cite the single reference value), the
map content icons (NPC/player heads, death markers, pings) are not ported, and
no UI click sounds play — the full list is in `docs/ANALYSIS.md` →
"Minimap frame and boss bar (P4)".

### Boss bar (`ui/bars.rs`, `BigProgressBarHelper.cs`)

The boss health bar is `BigProgressBarHelper.DrawFancyBar`'s 4-arg overload, fed
by `CommonBossBigProgressBar` (`CommonBossBigProgressBar.cs:32-36`) — in this
port King Slime is the only boss, head index 7, drawn from the whole
`NPC_Head_Boss_7` sheet (`NPC.cs:7557-7585`, `NPCID.cs:10479`). All six 516×58
rows of the `UI_BossBar` sheet are used through `Utils.Frame(1, 6, …)`.
Key geometry (1600×900):

* The bar area is `CenteredRectangle(Main.ScreenSize.ToVector2() *
  (0.5, 1) + (0, −50), (456, 22))` (`BigProgressBarHelper.cs:46`,
  `Utils.cs:947-950`) = `(572, 839, 456, 22)`, each component `(int)`-truncated.
* The draw origin is the bar's top-left `− p2 = (32, 24)` = `(540, 815)`
  (`:47`); the boss head icon is centred on `origin + (4, 20) + (26, 28)/2` =
  `(557, 849)` — `(26, 28)` is the source's literal centre, **not**
  `barIconFrame.Size()/2` (`:52`).
* The fill width is `(int)(456 · current/max)`, then `num2 −= num2 % 2`
  (`:33-35`) — always an even `0..=456`. The 2 px fill body
  (`(32, 140, 2, 22)` on the sheet) stretches to that width, the 2 px cap
  (`(32, 82, 2, 22)`) sits at `bar.x + fill − 2`, the frame row
  (`(0, 0, 516, 58)`) draws over both, and the backdrop row
  (`(0, 174, 516, 58)`) is tinted `Color.White · 0.2` — XNA byte truncation
  makes that `(51, 51, 51, 51)`, not white at 20 % alpha.
* The `current / max` text is `Fonts/Item_Stack` at scale 1, drawn with
  `Utils.DrawBorderStringFourWay` (`Utils.cs:2464-2495`): four black passes at
  `(±2, 0)`/`(0, ±2)` and the white centre pass, around
  `area.Center + (0, 1)`; `current` is right-aligned at `anchor.x − 5`, the
  slash centred, `max` left-aligned at `anchor.x + 5` (`:59-70`).

`ShowText` is always on (`BigProgressBarSystem.cs:259`); the port has no
settings toggle. Deviations: the shielded `DrawFancyBar` overload
(`:72-126`) and `DrawBareBonesBar` (`:11-23`) are not ported (no reachable
caller) — the full list is in `docs/ANALYSIS.md` →
"Minimap frame and boss bar (P4)".

## Deviations

* The framework ports the **flat/event halves** of `UIElement`/`UserInterface`:
  the retained tree, `OverflowHidden` scissoring (`:47`, 6 C# users),
  `IsElementUnderMouse`/`MouseCaptured` as a general predicate, `UIList`'s scroll
  mechanics and `UIScrollbar`'s drag/handle are deliberately out (the
  immediate-mode ruling + no ported consumer). The child→parent **event bubble**
  (`UIElement.cs:414-546`) *is* ported (`framework.rs::bubble_chain` + `Element`
  parent links, S1-H-1) and drives the select screens' card-button resolution;
  `MouseOver`/`MouseOut` *edges* still have no ported counterpart because hover is
  draw-derived and the colour swaps are `ui::hover_pair` (sound 12 is already a
  recorded deviation). See `docs/ANALYSIS.md` → "UIElement framework (M7-UI-1)"
  for the full gap list.
* The world behind the menu is our own surface render (the original draws the world at the
  surface with `Main.DrawMenu`'s camera); the logo, items, font and colours are the
  original's.
* Not ported yet, shown as a notice when clicked: Resource Packs, Credits, and the
  backing systems behind rows that now render at the original geometry. The
  Achievements row opens the `UIAchievementsMenu` overlay (M6-ACH-2: the 137-row
  registry, the category filters, `TR_ACHIEVEMENTS`/`TR_ACHIEVEMENTS_CATS`). — window resolution / borderless / fullscreen (menuMode 111), language
  switching (1213/1212), and the menuMode-888 `ManageControlsMenu` UIState (the
  1127 "Keybindings" row). The P5 sub-screens themselves are ported (see
  "Menu sub-screens (P5)"); their remaining gaps are collected in
  `docs/ANALYSIS.md` → "Menu sub-screens (P5)".
* The social-media buttons under the logo are not drawn (no icon assets loaded).
* `Main.UIScale` is assumed to be 1.0 (the DPI scale of the reference machine); the
  original multiplies the whole menu by it.
* In-game HUD/inventory deviations are collected in
  `docs/ANALYSIS.md` → "In-game interface (M7-UI)" (breath-bubble offsets, the
  `Main.invAlpha` pulse, `Main.ThickMouse` tooltip offsets, the missing
  bestiary/emote/achievement shortcut buttons, `Player.statDefense` not modelled …).
* Front-end (P6) deviations are collected in `docs/ANALYSIS.md` →
  "Front-end screens (P6)" (no `.plr` writer, no cloud buttons, fixed world
  metadata, no gamepad snap points, no UI click sounds, no credits cinematic …).
* The housing page (`EquipPage == 1`) is `ui::housing`: `Main.DrawNPCHousesInUI`
  (`Main.cs:41993-42107`) with `NPCHeadID.Sets.HeadListOrder`, the vanilla
  `NPC.TypeToDefaultHeadIndex` table and the column wrap at `screenHeight - 80`.
  The hover prints the cell's name (`:42064`; `Inter.8` for the query cell,
  `NPCName.<Kind>` for a head — the port has no per-NPC given names). Not ported
  yet: the `SetMouseNPC` click (`:42065-42071`, the port has no `Main.mouseNPC`
  housing query), the `Inventory_Back14` white gamepad highlight and the
  `hidePVPAndTeamIcons` third-column suppression.
* Chest panel (M3-CHEST-UI) deviations are collected in `docs/ANALYSIS.md` →
  "Chest UI + world chests (M3-CHEST-UI)" (lazy store entries, no lid animation,
  the rename keyboard pauses the sim, the cursor-override textures deferred …);
  the chest/bank integration has its own entry → "Chest/bank integration
  (M3-CHEST-INT)" (the two-panel ownership, the Alt+left-click favorite binding,
  the retain semantics …).

## Dev hooks and verification

* **`TR_SHOT` reaches the world but not (always) the interface — re-check
  2026-11-12.** Four captures taken on a clean `main`: `TR_SHOT_FRAME=20/60/300`
  with `TR_INV`, `TR_CHEST` and `TR_NOMENU` set in turn. Every one came back
  byte-identical (254 538 B) and pixel-checked as world-only — the inventory
  slot, the coin/ammo cells and the chest panel regions all sample the sky
  behind them (e.g. grid cell 0 mean RGB `(72, 92, 127)`), i.e. no HUD layer was
  drawn at all, while the log line `shot camera px …` shows a camera inside the
  world. `no_hud` was never set. So the HUD-screenshot recipes below are
  **unverified in that state**: before trusting a visual check, stage one shot
  that you can prove contains the panel (a known label pixel), and if the HUD
  still misses, treat the capture path itself as broken rather than the UI.
* `TR_MENU=1` keeps the menu on screen even with `TR_SHOT` (screenshots skip it by
  default), `TR_PAGE=name` opens a screen directly — the word aliases
  (`settings|general|interface|video|multiplayer`) plus the numeric `menuMode` of
  every P5 screen (`26`, `28`, `25`, `252`, `1125`, `111`, `2008`, `1212`, `1213`,
  `1127`, `27`, `272727`, `12`, `13`, `131`, `30`, `31`, `889`), `TR_FOCUS=n`
  focuses row `n` (the yellow hover state). `TR_CHEST` stages and opens the chest
  panel for its screenshot (the rename flow is covered by unit tests, not a shot).
* `cargo test -p tr-app` covers the per-screen `num2/num4/array7` table, the seven main-menu
  rows, the eight settings rows, the twelve interface rows, the Video Back-row quirk, the
  toggle cycles, Back navigation and the not-ported notices, the P5 sub-screen row tables
  (slider chrome maths and drag locks, the colour-screen floors, the Effects phantom-row
  skip, the language lists, the bind lists and capture machine, the multiplayer row gates
  and the submit chain), plus the `config.json` round-trip. The P6 front-end suites cover
  the `uistate` resolve/panel maths, the select/creation/keyboard layout tables and hit
  orders, the save listing/sorting rules, the `next_flow_state` table and the
  single-mode invariant.
* Screenshots at 1600×900: `.qoder_tmp/ui_{main,settings,general,interface,video}.png`
  (local; regenerate with the hooks above).

## The invasion progress bar (`Main.DrawInvasionProgress`)

`ui::draw_invasion_progress` is a full port of `Main.cs:47696-47845`.

* **Gates.** The function returns before drawing anything when
  `invasion_progress == -1` (no invasion tracked, `:47698`) or when the fade
  alpha clamps to `0` (`:47729`). Those two returns are
  `ui::invasion_bar_alpha`, unit-tested without a renderer — they are what
  keeps the panel off screen outside an event (BUG-08).
* **The fade.** `world_sim::npcs::update_invasion_state` is the head: the
  160-tick display timer decays, the alpha ramps ±0.05 per tick, the
  `EventProgress` setting's Off mode zeroes both, and the near-invasion flag
  holds the timer at 160. `num = 0.5 + alpha * 0.5` is the panel scale.
* **Two arms.** `invasion_progress_wave > 0` (the moon events and the Old
  One's Army) draws `Game.WaveMessage` — "Wave {0}: {1}"; the other arm draws
  `Game.WaveCleared` — "Cleared {0}", where `{0}` is the percent (or the raw
  points when the max is 0). Both arms draw the same 200x45
  `Inventory_Back13` panel at `(screenWidth - 120, screenHeight - 40)` with the
  `(63,65,151,255) * 0.785` tint, the 178x16 `ColorBar` track, and the
  yellow `(255,241,51)` / orange `(255,165,0,127)` / black fills; every
  colour is multiplied by the fade alpha with XNA's byte truncation.
* **The group plate.** A second, group-tinted `Inventory_Back13` box centred
  at `(screenWidth - num13, screenHeight - 80)` carries the `Extra` icon
  (`num * 0.8`, its centre on the plate's top edge) and the right-aligned
  label (`num * 0.9`).
* **The icon table** (`Main.cs:47734-47780`), all localised:

  | icon | `Extra` | label | plate colour |
  | --- | --- | --- | --- |
  | 1 | `Extra_8` | `Lang.inter[83]` Frost Moon | (64,109,164) |
  | 2 | `Extra_12` | `Lang.inter[84]` Pumpkin Moon | (112,86,114) |
  | 3 | `Extra_79` | `DungeonDefenders2.InvasionProgressTitle` | (88,0,160) |
  | 7 | `Extra_10` | `Lang.inter[85]` Martian Madness | (165,160,155) |
  | 6 | `Extra_11` | `Lang.inter[86]` Pirate Invasion | (148,122,72) |
  | 5 | `Extra_7` | `Lang.inter[87]` Frost Legion | (173,135,140) |
  | 4 | `Extra_9` | `Lang.inter[88]` Goblin Army | (94,72,131) |
  | other | `Extra_9` | "" | White (no halving) |

