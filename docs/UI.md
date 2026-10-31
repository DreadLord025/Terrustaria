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
`MouseShowBuildingGrid`, `GamepadInstructions`, `CaveParallax`, `FrameSkipMode`,
`LightingMode`, `Quality`, `BackgroundEnabled`, `BloodAndGore`, `MinersWobble`,
`TilesSwayInWind`, `MusicVolume`, `SoundVolume`, `AmbientVolume`.

The renderer reads `LightingMode` (applied when the menu closes, through
`LightMode::ALL`), `BackgroundEnabled` (the parallax layer gate in `draw_sky`) and
`MapEnabled` (minimap/full map). The rest is stored and shown in the labels, ready for the
systems that will consume it (volumes for `audio`, `PlacementPreview` for the outline,
`FrameSkipMode` for the loop).

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
| `ui/chrome.rs` | `DrawPageIcons`/`DrawDefenseCounter`/`DrawGuideCraftText`/`DrawPVPIcons` | page tabs, defence shield, guide text, PVP icons |
| `ui/crafting.rs` | `NewCraftingUI.cs` (+ `UIWrappedSearchBar`, `UICreativeItemsInfiniteFilteringOptions`, `CraftingUI.DrawGridToggle`) | the modern crafting panel: frame, title, search bar, filter strip, separator, empty grid, scrollbar, detail area, toggles |

Key geometry (1600×900, `UIScale = 1.0`, `Main.inventoryScale = 0.85`):

* Hearts start at `screenWidth − 300` and step 26 px; mana stars at `775 + anchor`.
* Grid slot `(i, j)` is at `(20 + i*56*0.85, 20 + j*56*0.85)` with a hard-coded
  56 px pitch; the drawn cell is the sheet's own `Size()*scale` (52 px sheet).
* Coins and ammo use `inventoryScale = 0.6` at x `497` / `534`, rows at
  `(int)(85 + k*56*0.6 + 20)`.
* Equipment columns are right-anchored at `screenWidth − 92 / − 139 / − 186`
  (armour, vanity, dye) from `num20 = 174`, `+4` on the rows past the third.
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

## Deviations

* The world behind the menu is our own surface render (the original draws the world at the
  surface with `Main.DrawMenu`'s camera); the logo, items, font and colours are the
  original's.
* Not ported yet, shown as a notice when clicked: Achievements, Resource Packs, Credits,
  Volume (the three `IngameOptions.DrawValueBar` sliders), Cursor colour, Controls/key
  rebinding, Language selection, Resolution, Effects, map borders, health/mana styles,
  portraits, boss-bar numbers and inventory effects.
* The social-media buttons under the logo are not drawn (no icon assets loaded).
* `Main.UIScale` is assumed to be 1.0 (the DPI scale of the reference machine); the
  original multiplies the whole menu by it.
* In-game HUD/inventory deviations are collected in
  `docs/ANALYSIS.md` → "In-game interface (M7-UI)" (breath-bubble offsets, the
  `Main.invAlpha` pulse, `Main.ThickMouse` tooltip offsets, the missing
  bestiary/emote/achievement shortcut buttons, `Player.statDefense` not modelled …).

## Dev hooks and verification

* `TR_MENU=1` keeps the menu on screen even with `TR_SHOT` (screenshots skip it by
  default), `TR_PAGE=settings|general|interface|video|multiplayer` opens a screen directly,
  `TR_FOCUS=n` focuses row `n` (the yellow hover state).
* `cargo test -p tr-app` covers the per-screen `num2/num4/array7` table, the seven main-menu
  rows, the eight settings rows, the twelve interface rows, the Video Back-row quirk, the
  toggle cycles, Back navigation and the not-ported notices, plus the `config.json`
  round-trip.
* Screenshots at 1600×900: `.qoder_tmp/ui_{main,settings,general,interface,video}.png`
  (local; regenerate with the hooks above).
