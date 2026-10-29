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
