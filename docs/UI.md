# UI — the original menu and settings screens (M7)

The menu is Terraria's immediate-mode menu (`Main.DrawMenu`), not the newer `UIState`
system: `Main.menuMode` selects a screen, and every screen is a flat list of strings drawn
with `FontAssets.DeathText` around `Main.screenWidth / 2`. This port follows that code
path so the layout, text sizes and spacing match the original.

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

`Main.SaveSettings` writes `config.json` in the game root; this port writes the same keys
to `Documents/My Games/Terrustaria/config.json` and reads them back on start (unknown keys
ignored, missing keys keep their default):

`AutoSave`, `AutoPause`, `MapEnabled`, `HidePassword`, `SettingPlayWhenUnfocused`,
`PickupText`, `InvasionProgressMode`, `PlacementPreview`, `HighlightNewItems`,
`MouseShowBuildingGrid`, `GamepadInstructions`, `BackgroundParallax`,
`FrameSkipMode`, `LightingMode`, `Quality`, `BackgroundEnabled`, `BloodAndGore`,
`MinersWobble`, `TilesSwayInWind`, `UseStormEffect`, `UseHeatDistortion`,
`ForegroundSunlightEffects`, `WaveQuality`, `FlashyEffectsWorld`, `UseScreenShake`,
`Zoom`, `MusicVolume`, `SoundVolume`, `AmbientVolume`, `MapScale`, `MouseColor`
(R/G/B dict), `MouseBorderColor` (R/G/B/A dict), `SmartCursorToggle`,
`UseSmartAxeAfterSmartPickaxe`, `UseSmartCursorForCommonBlocks`, `LockOnPriority`,
`ReverseUpDownForArmorSetBonuses`, `DisableLeftShiftTrashCan`, `DisableQuickTrash`,
`PlayerResourcesSet`, `PortraitPreference`, `ShowBossBarHealthText`,
`FlashyEffectsInterface`, `MinimapFrame`.

The renderer reads `LightingMode` (applied when the menu closes, through
`LightMode::ALL`), `BackgroundEnabled` (the parallax layer gate in `draw_sky`) and
`MapEnabled` (minimap/full map). The rest is stored and shown in the labels, ready for the
systems that will consume it (volumes for `audio`, `PlacementPreview` for the outline,
`FrameSkipMode` for the loop).

## Menu sub-screens (P5)

Every `Main.DrawMenu` settings branch below menuMode 11 is ported as one `Page` in
`menu.rs`, keyed by its original `menuMode` — which is also accepted as a `TR_PAGE`
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
| Volume | `26` | 200 / 10 / 1.0 | blank, "Volume" header @210, Back @470 → Settings |
| Parallax | `28` | 240 / 60 / 1.0 | blank, "Parallax" header @250, Back @530 → Video |
| Cursor | `1125` | 232 / 38 / 0.73 | Cursor Color, Border Color, Smart Cursor Mode/Priority/Placement, Lock On Priority, Back @478 |
| Cursor Color | `25` | 200 / 10 / 1.0 | header @210, Back @470; HSL sliders with luminosity floored at `0.15` and no alpha bar |
| Border Color | `252` | 200 / 10 / 1.0 | the 25 twin: full luminosity range and the alpha bar |
| Resolution | `111` | 210 / 55 / 0.85 | Fullscreen Resolution @210, Borderless Window @265, Go Fullscreen @320, Apply @475, Back @530 |
| Effects | `2008` | 210 / 28 / 0.6 (header 0.7) | header @202 (`array4 = −8`), six effect rows, Back @406 — the branch declares 9 rows but leaves row 8 null and the item loop skips it |
| Language | `1213` | 200 / 26 / 0.65 | "Select language" @180 (scale 0.85), 12 languages, Back @547 (scale 0.8) |
| Language Initial | `1212` | 200 / 30 / 0.65 | the first-launch picker: header scale 1.0, 12 languages, no Back |
| Controls | `1127` | 250 / 52 / 0.78 | Set Bonuses @250, Quick Trash @302, Keybindings @354 (notice — the original opens the 888 UIState), Back @424 |
| Keybindings | `27` | 176 / 22 / 0.45 | 14 left-aligned bind rows (`array5 = −80`) + Reset + Back, both scale 0.8 |
| Map Keys | `272727` | 200 / 30 / 0.55 | 6 left-aligned bind rows (`array5 = −140`) + Reset (0.8) + Back (scale **1.0** — the branch never sets `array7[7]`) |
| Multiplayer | `12` | 250 / 80 / 1.0 | Join @250, Host & Play @330, Back @410 — "Join via Steam" is overwritten without `SocialAPI.Network`, so Join is first |
| Join IP | `13` | 180 / 30 / 1.0 | header @180, input @199 (`array4 = 19`), 7 blank recent rows @280..460 (scale 0.6), Accept @494, Back @544 |
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
| `ui/tooltip.rs` | `Main.MouseText` + `Utils.DrawInvBG` | the 9-slice tooltip box, rarity-coloured name |
| `ui/slot.rs` | `ItemSlot.Draw` | per-context background, icon, stack text, hotbar digit, empty-equip glyph |
| `ui/inventory.rs` | `Main.DrawInventory` | 10×5 grid, coins/ammo, equipment columns, misc page, trash, labels |
| `ui/chrome.rs` | `DrawPageIcons`/`DrawDefenseCounter`/`DrawGuideCraftText`/`DrawPVPIcons` + `Main.DrawInventory`'s map-style row | page tabs, defence shield, guide text, PVP icons, the four map-style icons |
| `ui/minimap.rs` | `MinimapFrame`/`MinimapFrameManager`/`MinimapFrameTemplate` (+ `SelectionHolder.CycleSelection`) | the minimap frame chrome: 9 style table, backdrop, frame + button hover sheets, zoom buttons |
| `ui/bars.rs` | `BigProgressBarHelper.DrawFancyBar` (+ `DrawHealthText`) | the boss health bar: 516×58 `UI_BossBar` rows, 456×22 bar, boss head icon, `current / max` text |
| `ui/crafting.rs` | `NewCraftingUI.cs` (+ `UIWrappedSearchBar`, `UICreativeItemsInfiniteFilteringOptions`, `CraftingUI.DrawGridToggle`) | the modern crafting panel: frame, title, search bar, filter strip, separator, empty grid, scrollbar, detail area, toggles |

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

* The world behind the menu is our own surface render (the original draws the world at the
  surface with `Main.DrawMenu`'s camera); the logo, items, font and colours are the
  original's.
* Not ported yet, shown as a notice when clicked: Achievements, Resource Packs,
  Credits, and the backing systems behind rows that now render at the original
  geometry — window resolution / borderless / fullscreen (menuMode 111), language
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

## Dev hooks and verification

* `TR_MENU=1` keeps the menu on screen even with `TR_SHOT` (screenshots skip it by
  default), `TR_PAGE=name` opens a screen directly — the word aliases
  (`settings|general|interface|video|multiplayer`) plus the numeric `menuMode` of
  every P5 screen (`26`, `28`, `25`, `252`, `1125`, `111`, `2008`, `1212`, `1213`,
  `1127`, `27`, `272727`, `12`, `13`, `131`, `30`, `31`, `889`), `TR_FOCUS=n`
  focuses row `n` (the yellow hover state).
* `cargo test -p tr-app` covers the per-screen `num2/num4/array7` table, the seven main-menu
  rows, the eight settings rows, the twelve interface rows, the Video Back-row quirk, the
  toggle cycles, Back navigation and the not-ported notices, the P5 sub-screen row tables
  (slider chrome maths and drag locks, the colour-screen floors, the Effects phantom-row
  skip, the language lists, the bind lists and capture machine, the multiplayer row gates
  and the submit chain), plus the `config.json` round-trip.
* Screenshots at 1600×900: `.qoder_tmp/ui_{main,settings,general,interface,video}.png`
  (local; regenerate with the hooks above).
