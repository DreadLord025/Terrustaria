//! `Main.SaveSettings`: the options behind the settings screens.
//!
//! The original stores them in `config.json` (`Main.SaveSettings`, `Preferences`), so the
//! same file name and key names are used here; unknown keys are ignored and missing keys
//! keep their default, which makes a vanilla `config.json` readable.

use std::path::PathBuf;

#[derive(Clone, Debug)]
pub struct Settings {
    /// `Language.ActiveCulture.Name` — the saved `GameCulture` name
    /// ("en-US", "ru-RU", …). Vanilla persists it under `"Language"`
    /// (`Put` `Main.cs:4099`, read back by `LoadSettings`'s
    /// `LanguageManager.SetLanguage(text)` `:4345`); the port applies it at
    /// startup (`lang::init`) and on the language-menu click.
    pub language: String,
    /// `Main.autoSave`.
    pub auto_save: bool,
    /// `Main.autoPause`.
    pub auto_pause: bool,
    /// `Main.mapEnabled`.
    pub map_enabled: bool,
    /// `Main.HidePassword`. Vanilla persists it under `"HidePasswords"` — plural
    /// (`Put` `Main.cs:4144`, `Get` `:4422`); the port wrote the singular
    /// `"HidePassword"` before this parity pass, so `parse` reads **both**
    /// spellings (vanilla-last) while `to_json` writes only the vanilla name.
    pub hide_password: bool,
    /// `Main.SettingPlayWhenUnfocused`: default **false** — `static Main()`
    /// (`Main.cs:66512`), persisted under `"PlayWhenUnfocused"` (`Put`
    /// `Main.cs:4217`, `Get` `:4352`). The menuMode 11 row inverts it into the
    /// "Pause When Unfocused" label ("Off" when the flag is true).
    pub play_when_unfocused: bool,
    /// `Main.showItemText`, persisted under `"ShowItemText"` (`Put`
    /// `Main.cs:4135`, `Get` `:4409`) — the port's old key was `"PickupText"`.
    pub pickup_text: bool,
    /// `Main.invasionProgressMode`: 0 off, 1 timed, 2 on; default **2** (On) —
    /// `static Main()` (`Main.cs:67080`). Vanilla persists it under
    /// `"InvasionBarMode"` (`Put` `Main.cs:4096`, `Get` `:4332`) — the port's
    /// old key was `"InvasionProgressMode"`.
    pub event_progress: u8,
    /// `Main.placementPreview`.
    pub placement_preview: bool,
    /// `ItemSlot.Options.HighlightNewItems`.
    pub highlight_new_items: bool,
    /// `Main.MouseShowBuildingGrid`.
    pub tile_grid: bool,
    /// `Main.GamepadDisableInstructionsDisplay`, persisted under the same name
    /// (`Put` `Main.cs:4151`, `Get` `:4427`) — the port's old key was
    /// `"GamepadInstructions"`.
    pub gamepad_instructions: bool,
    /// `Main.bgScroll` (`Main.cs:2429`, a plain static): the "Parallax" slider
    /// value, 0..=100 — the bar stores `(int)(value * 100f)` (`Main.cs:51642`)
    /// and the backdrop scrolls by `caveParallax = 1 - bgScroll / 500`
    /// (`Main.cs:50383`, `:51643`). Vanilla persists it under "BackgroundParallax"
    /// (`:4134`, read back at `:4407`). A fresh config still starts at 60, not 0:
    /// `static Main()` seeds `caveParallax = 0.88f` (`Main.cs:66809`), the
    /// `Get("Parallax", ref caveParallax)` finds no key so 0.88 survives, then
    /// `bgScroll = (int)((1f - 0.88f) * 500f)` = 60 and the trailing
    /// `Get("BackgroundParallax", ref bgScroll)` keeps it (`Main.cs:4405-4408`)
    /// — the screen shows "Parallax: 60" with the 0.88 backdrop factor.
    pub parallax_scroll: u8,
    /// `Main.FrameSkipMode`: 0 off, 1 on, 2 subtle (`FrameSkipMode.cs:3-8`);
    /// default **2** (Subtle) — `static Main()` (`Main.cs:67204`).
    pub frame_skip: u8,
    /// `Lighting.Mode` in the **port's internal** order (`LightMode.cs:3-9`'s
    /// mirror, `tr-core/src/lighting/params.rs:12-17`): 0 Color, 1 White,
    /// 2 Retro, 3 Trippy. The *persisted* value is the C# ordinal instead —
    /// `LightMode` in the original is `{White=0, Retro=1, Trippy=2, Color=3}`
    /// (`Terraria/Graphics/Light/LightMode.cs:3-9`), so `parse`/`to_json`
    /// translate in both directions and every in-memory reader sees this port
    /// order.
    pub light_mode: u8,
    /// `Main.qaStyle`: 0 auto, 1 high, 2 medium, 3 low. Vanilla persists it
    /// under `"GraphicsQuality"` (`Put` `Main.cs:4130`, `Get` `:4371`) — the
    /// port's old key was `"Quality"`.
    pub quality: u8,
    /// `Main.BackgroundEnabled`.
    pub background: bool,
    /// `ChildSafety.Disabled` (blood and gore), persisted under
    /// `"GoreVisualsAllowed"` (`Put` `Main.cs:4101`, `Get` `:4349`) — the port's
    /// old key was `"BloodAndGore"`.
    pub blood: bool,
    /// `Main.SettingsEnabled_MinersWobble`, persisted under the same name (`Put`
    /// `Main.cs:4153`, `Get` `:4540`) — the port's old key was
    /// `"MinersWobble"`.
    pub miners_wobble: bool,
    /// `Main.SettingsEnabled_TilesSwayInWind`, persisted under the same name
    /// (`Put` `Main.cs:4157`, `Get` `:4543`) — the port's old key was
    /// `"TilesSwayInWind"`.
    pub tiles_sway: bool,
    /// `Player.Settings.CraftFromNearbyChests` (`Player.cs:360`), defaulted
    /// true (`Player.cs:406`): the crafting pulls ingredients from the world
    /// chests in range.
    pub craft_from_nearby_chests: bool,
    /// `Main.GameZoomTarget` (`Zoom`): 1.0..=2.0, only changed in the settings UI.
    pub zoom: f32,
    /// `Main._uiScaleWanted` (`Main.cs:251`), the persisted **wanted** UI scale
    /// — the value `Main.SaveSettings` writes under `"UIScale"`
    /// (`Put` `Main.cs:4201`, read back by `LoadSettings` `:4487`). The setter
    /// (`Main.UIScale`, `:2557-2573`) stores the raw `value` here and clamps only
    /// the *used* value to `UIScaleMax` (`:2566-2571`, `:2576-2583`), so the
    /// wanted and the applied scale can differ; [`Settings::applied_ui_scale`]
    /// is the used value. A 100 %-scaling desktop's `GetScaling()` is 1.0, which
    /// is the port's default (the vanilla's `newDefaultUIScale` pick at
    /// `:4257-4265` is version-gated and needs a live display — see
    /// docs/ANALYSIS.md).
    pub ui_scale: f32,
    /// `Main.musicVolume`: default **1f** — `static Main()` (`Main.cs:66857`).
    /// Persisted under `"VolumeMusic"` (`Put` `Main.cs:4107`, `Get` `:4357`) —
    /// the port's old key was `"MusicVolume"`.
    pub music_volume: f32,
    /// `Main.soundVolume`: default **1f** — `static Main()` (`Main.cs:66859`).
    /// Persisted under `"VolumeSound"` (`Put` `Main.cs:4105`, `Get` `:4355`) —
    /// the port's old key was `"SoundVolume"`.
    pub sound_volume: f32,
    /// `Main.ambientVolume`: default **1f** — `static Main()` (`Main.cs:66858`).
    /// Persisted under `"VolumeAmbient"` (`Put` `Main.cs:4106`, `Get` `:4356`) —
    /// the port's old key was `"AmbientVolume"`.
    pub ambient_volume: f32,
    /// `Main.MapScale`: saved as "MapScale" (`Main.cs:4202`), load-clamped
    /// [0.5, 1] (`:4494`), default 1 (`:67298`). P4 draws the reference 1.0
    /// only (the whole-UI P4 plan, Uncertainty 5 ruling).
    pub map_scale: f32,
    /// `MinimapFrameManager.cs:13-18`: the active minimap frame style key
    /// (`"Default"`, `"Golden"`, ...; unknown names fall back to index 0).
    pub minimap_frame: String,
    /// `Main.mouseColor` (`Main.cs:67199`, default `new Color(255, 50, 95)`):
    /// the cursor colour the menuMode 25 screen edits (`Main.cs:50324`).
    /// Vanilla persists R/G/B only (`:4164-4176`, `:4448-4460`), so the alpha
    /// byte stays 255.
    pub mouse_color: [u8; 4],
    /// `Main.MouseBorderColor` (`Main.cs:66514`, default
    /// `new Color(64, 64, 64, 64)`): the smart-cursor border colour the
    /// menuMode 252 screen edits (`Main.cs:50346`). Vanilla persists R/G/B/A
    /// (`:4179-4197`, `:4462-4478`).
    pub mouse_border_color: [u8; 4],
    /// `Main.cSmartCursorModeIsToggleAndNotHold` (`Main.cs:67178`, default
    /// true): the "Smart Cursor Mode" row of menuMode 1125
    /// (`Main.cs:50269-50277`).
    pub smart_cursor_toggle: bool,
    /// `Player.SmartCursorSettings.SmartAxeAfterPickaxe` (`Player.cs:334`, a
    /// plain static: default false): the "Smart Cursor Priority" row
    /// (`Main.cs:50278-50285`).
    pub smart_axe_first: bool,
    /// `Player.SmartCursorSettings.SmartBlocksEnabled` (`Player.cs:331`, a
    /// plain static: default false): the "Smart Block Placement" row
    /// (`Main.cs:50286-50293`).
    pub smart_blocks: bool,
    /// `DoorOpeningHelper.PreferenceSettings` (`DoorOpeningHelper.cs:216`,
    /// default `Disabled` = 0): the "Smart Doors" row of the in-game options
    /// (`IngameOptions.cs:644-657`). Persisted as `"DoorAutoOpeningMode"`
    /// (`Main.cs:4208`/`:4545`).
    pub smart_doors: u8,
    /// `Player.Settings.HoverControl` (`Player.cs:362`, default
    /// `ButtonControlMode.Hold` = 0 at `Player.cs:407`): the "Hover Controls"
    /// row (`IngameOptions.cs:659-675`). Persisted as `"HoverControlMode"`
    /// (`Main.cs:4209`/`:4546`).
    pub hover_control: u8,
    /// `Main.SettingsEnabled_AutoReuseAllItems` (`Main.cs:261`, default false):
    /// the "Autofire" row (`IngameOptions.cs:677-684`). Persisted as
    /// `"SettingsEnabled_AutoReuseAllItems"` (`Main.cs:4158`/`:4544`).
    pub auto_reuse_all: bool,
    /// `LockOnHelper.UseMode` (`LockOnHelper.cs:24`): 0 FocusTarget,
    /// 1 TargetClosest, 2 ThreeDS. Default **2**, not the enum's 0:
    /// `LoadSettings` runs the `"LockOnPriority"` chain on every boot
    /// (`Main.cs:4496-4505`), clamps to 0..=2 and remaps FocusTarget to
    /// ThreeDS, so a fresh config boots into ThreeDS.
    pub lock_on_priority: u8,
    /// `Main.UseStormEffects` (`Main.cs:429`, default true at `:66572`): the
    /// "Storm Effects" row of menuMode 2008 (`Main.cs:50113-50118`). Vanilla
    /// persists it under `"UseStormEffect"` — singular (`Put` `:4159`, `Get`
    /// `:4435`).
    pub use_storm_effects: bool,
    /// `Main.ForegroundSunlightEffects` (`Main.cs:433`, default true at
    /// `:66574`): the "Sunlight Effects" row (`Main.cs:50120-50125`). Vanilla
    /// key `"ForegroundSunlightEffects"` (`Put` `:4161`, `Get` `:4436`).
    pub foreground_sunlight: bool,
    /// `Main.UseHeatDistortion` (`Main.cs:431`, default true at `:66573`): the
    /// "Heat Distortion" row (`Main.cs:50127-50132`). Vanilla key
    /// `"UseHeatDistortion"` (`Put` `:4160`, `Get` `:4437`).
    pub use_heat_distortion: bool,
    /// `Main.WaveQuality` (`Main.cs:427`, default 3 at `:66571`): the "Waves
    /// Quality" row cycles `(q + 1) % 4` (`Main.cs:50144`); the label reads
    /// 1 Low, 2 Medium, 3 High, anything else Off (`Main.cs:50134-50143`).
    /// Vanilla key `"WaveQuality"` (`Put` `:4162`), load-clamped to 0..=3
    /// (`:4438-4445`).
    pub wave_quality: u8,
    /// `Main.UseScreenShake` (`Main.cs:435`, default true at `:66575`): the
    /// "Screen Shake" row (`Main.cs:50150-50155`). Vanilla key
    /// `"UseScreenShake"` (`Put` `:4218`, `Get` `:4552`).
    pub use_screen_shake: bool,
    /// `Main.FlashyEffectsWorld` (`Main.cs:437`, default true at `:66576`): the
    /// "Thunder Effects" row (`Main.cs:50157-50162`). Vanilla key
    /// `"FlashyEffectsWorld"` (`Put` `:4215`, `Get` `:4350`).
    pub flashy_effects_world: bool,
    /// `Main.ReversedUpDownArmorSetBonuses` (`Main.cs:273`, a plain static:
    /// default false): the "Activate Set Bonuses" row of menuMode 1127
    /// (`Main.cs:50647-50656`). Vanilla key `"ReverseUpDownForArmorSetBonuses"`
    /// (`Put` `:4145`, `Get` `:4424`).
    pub reversed_armor_bonuses: bool,
    /// `ItemSlot.Options.DisableLeftShiftTrashCan` (`ItemSlot.cs:28`, default
    /// true in the `static Options()` ctor `:36`): the menuMode 1127
    /// "Quick Trash" row's Left Control state (`Main.cs:50656-50687`). Vanilla
    /// key `"DisableLeftShiftTrashCan"` (`Put` `:4141`, `Get` `:4416`).
    pub disable_left_shift_trash: bool,
    /// `ItemSlot.Options.DisableQuickTrash` (`ItemSlot.cs:30`, default false at
    /// `:37`): the same row's Disabled state. Vanilla key
    /// `"DisableQuickTrash"` (`Put` `:4142`, `Get` `:4420`).
    pub quick_trash: bool,
    /// `PlayerResourceSetsManager._activeSetConfigKey`: the health/mana style
    /// of the menuMode 1112 "Health and Mana Style" row
    /// (`Main.cs:49948-49952`). Vanilla key `"PlayerResourcesSet"`
    /// (`Put` `PlayerResourceSetsManager.cs:37`, `Get` `:28` — default
    /// `"New"`); the port's default is `"Default"` because the port renders
    /// the Classic set, so the row must show "Classic" like the HUD does
    /// (the deviation is recorded for Task 7).
    pub resource_sets: String,
    /// `Main.DialoguePortraitPreference` (`Main.cs:2331`, a plain static:
    /// default 0 Detailed): the `DialoguePortraitDrawOption` of the menuMode
    /// 1112 "Portraits Style" row (`Main.cs:49954-49965`) — 0 Detailed,
    /// 1 CloseUp, 2 FullBodyRetro, 3 Disabled (`Main.cs:186-189`). Vanilla key
    /// `"PortraitPreference"` (`Put` `:4220`, `Get` `:4551`).
    pub portrait_preference: u8,
    /// `BigProgressBarSystem.ShowText` (`BigProgressBarSystem.cs:42`): the
    /// menuMode 1112 "Boss Health Bar Numbers" row (`Main.cs:49967-49971`).
    /// The default is **true** — the static constructor sets it
    /// (`BigProgressBarSystem.cs:259`), so the field's `false` never shows.
    /// Vanilla key `"ShowBossBarHealthText"` (`Put` `:231`, `Get` `:236`).
    /// Stored only: P4's boss bar draws the numbers unconditionally.
    pub boss_bar_text: bool,
    /// `Main.FlashyEffectsInterface` (`Main.cs:439`, default true in the
    /// static block `:66578`): the menuMode 1112 "Inventory Effects" row
    /// (`Main.cs:49973-49979`). Vanilla key `"FlashyEffectsInterface"`
    /// (`Put` `:4216`, `Get` `:4351`).
    pub flashy_effects_interface: bool,
    /// `Main.cUp`..`Main.cMount` (`Main.cs:67163-67176`): the 14 gameplay
    /// binds of the menuMode 27 list, in the vanilla's declaration order
    /// (`cUp, cLeft, cDown, cRight, ...` — **not** the typing order).
    ///
    /// Vanilla keeps them as plain strings and reads them back from
    /// `config.json` under the `Key*` names (`Main.LoadSettings`,
    /// `Main.cs:4582-4595`), handing them to `PlayerInput.CurrentProfile`'s
    /// `KeyStatus` table (`:4602-4625`). The port writes the same names so a
    /// vanilla config round-trips. See [`crate::input_map`] for the lookups.
    pub keybinds: [String; crate::input_map::BIND_COUNT],
    /// `Main.cMapStyle`..`Main.cMapAlphaDown` (`Main.cs:67190-67195`): the 6
    /// map binds of the menuMode 272727 list (`:4596-4600`).
    pub map_binds: [String; crate::input_map::MAP_BIND_COUNT],
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            language: "en-US".to_string(),
            auto_save: true,
            auto_pause: false,
            map_enabled: true,
            hide_password: false,
            // `Main.SettingPlayWhenUnfocused = false` (`Main.cs:66512`).
            play_when_unfocused: false,
            pickup_text: true,
            // `Main.invasionProgressMode = 2` (`Main.cs:67080`) — the
            // "Event Progress Bar: On" row of menuMode 1112.
            event_progress: 2,
            placement_preview: true,
            highlight_new_items: true,
            tile_grid: false,
            gamepad_instructions: true,
            parallax_scroll: 60,
            // `Main.FrameSkipMode = FrameSkipMode.Subtle` (`Main.cs:67204`), i.e.
            // the enum's 2 (`Terraria/Enums/FrameSkipMode.cs:3-8`).
            frame_skip: 2,
            light_mode: 0,
            quality: 0,
            background: true,
            blood: true,
            miners_wobble: true,
            tiles_sway: true,
            craft_from_nearby_chests: true,
            zoom: 1.0,
            // `LoadSettings`' `Get("UIScale", GetScaling())` (`Main.cs:4487`) — a
            // 100 %-scaling desktop returns 1.0.
            ui_scale: 1.0,
            // `Main.musicVolume = 1f` (`Main.cs:66857`), `Main.ambientVolume =
            // 1f` (`:66858`), `Main.soundVolume = 1f` (`:66859`) — the
            // `static Main()` values, not the 0.75 the port used before.
            music_volume: 1.0,
            sound_volume: 1.0,
            ambient_volume: 1.0,
            map_scale: 1.0,
            minimap_frame: "Default".to_string(),
            mouse_color: [255, 50, 95, 255],      // Main.cs:67199
            mouse_border_color: [64, 64, 64, 64], // Main.cs:66514
            smart_cursor_toggle: true,            // Main.cs:67178
            smart_axe_first: false,               // Player.cs:334
            smart_blocks: false,                  // Player.cs:331
            smart_doors: 0,                       // DoorOpeningHelper.cs:216 (Disabled)
            hover_control: 0,                     // Player.cs:407 (Hold)
            auto_reuse_all: false,                // Main.cs:261
            lock_on_priority: 2,                  // Main.cs:4496-4505
            use_storm_effects: true,              // Main.cs:66572
            foreground_sunlight: true,            // Main.cs:66574
            use_heat_distortion: true,            // Main.cs:66573
            wave_quality: 3,                      // Main.cs:66571
            use_screen_shake: true,               // Main.cs:66575
            flashy_effects_world: true,           // Main.cs:66576
            reversed_armor_bonuses: false,        // Main.cs:273
            disable_left_shift_trash: true,       // ItemSlot.cs:36
            quick_trash: false,                   // ItemSlot.cs:37
            resource_sets: "Default".to_string(), // the deviation recorded for Task 7
            portrait_preference: 0,               // Main.cs:2331 (Detailed)
            boss_bar_text: true,                  // BigProgressBarSystem.cs:259
            flashy_effects_interface: true,       // Main.cs:66578
            keybinds: crate::input_map::default_binds(),
            map_binds: crate::input_map::default_map_binds(),
        }
    }
}

impl Settings {
    /// `Documents/My Games/Terrustaria/config.json` — the game root, not `Worlds/`,
    /// matching `Main.SaveSettings`' location in the original.
    pub fn path() -> Option<PathBuf> {
        tr_core::save::default_world_dir().and_then(|d| {
            d.parent().map(|root| root.join("config.json"))
        })
    }

    /// `Main.UIScale`'s *used* value: the setter clamps the wanted value down to
    /// `UIScaleMax` and never below it (`Main.cs:2563-2572`). See
    /// [`ui_scale_max`].
    pub fn applied_ui_scale(&self, real_screen_w: f32, real_screen_h: f32) -> f32 {
        self.ui_scale.min(ui_scale_max(real_screen_w, real_screen_h))
    }

    pub fn load() -> Self {
        let Some(path) = Self::path() else {
            return Self::default();
        };
        let Ok(text) = std::fs::read_to_string(path) else {
            return Self::default();
        };
        Self::parse(&text)
    }

    /// Reads the `config.json` subset this port stores. Unknown keys are ignored and
    /// missing keys keep their default.
    ///
    /// Two spellings are accepted for the eleven keys this port renamed during the
    /// parity pass (`HidePassword`→`HidePasswords` `Main.cs:4144` plus the ten
    /// `SaveSettings` renames `:4096-4157`): the vanilla name comes first and the
    /// port's pre-parity name second, so a file carrying both spellings still
    /// resolves the vanilla line (the parser's "last line wins"). None of the
    /// eleven legacy spellings exists in the decompiled source (grep-verified),
    /// so the alias set is purely the restore path for the port's own configs.
    ///
    /// KNOWN DEVIATION, deliberately **not** changed by the rename pass (recorded
    /// here so docs own the ANALYSIS entry): the boolean arms coerce every token
    /// that is not the bare literal `"true"` — quoted `"True"`, `7`, `null`
    /// included — to `false`, where vanilla's `Preferences.Get` would keep the
    /// field default on a `Convert.ChangeType` failure
    /// (`Terraria/IO/Preferences.cs:176-190`). That coercion is port-wide and
    /// pre-existing, so each alias arm behaves exactly like its vanilla sibling.
    pub fn parse(text: &str) -> Self {
        let mut me = Self::default();
        for line in text.lines() {
            let line = line.trim().trim_end_matches(',');
            let Some((key, value)) = line.split_once(':') else {
                continue;
            };
            let key = key.trim().trim_matches('"');
            let value = value.trim();
            let boolean = |v: &str| v.eq_ignore_ascii_case("true");
            match key {
                "AutoSave" => me.auto_save = boolean(value),
                "AutoPause" => me.auto_pause = boolean(value),
                // `configuration.Put("Language", Language.ActiveCulture.Name)`
                // (`Main.cs:4099`) writes a quoted string.
                "Language" => {
                    let v = value.trim();
                    me.language = v
                        .strip_prefix('"')
                        .and_then(|rest| rest.split_once('"').map(|(s, _)| s))
                        .unwrap_or(v)
                        .to_string();
                }
                "MapEnabled" => me.map_enabled = boolean(value),
                // `configuration.Put("HidePasswords", Main.HidePassword)`
                // (`Main.cs:4144`) and `Get` `:4422`. The port wrote the
                // singular `"HidePassword"` before this pass, so both spellings
                // are read and the vanilla name comes last: a file carrying
                // both still resolves vanilla (the file's "last line wins").
                "HidePassword" => me.hide_password = boolean(value),
                "HidePasswords" => me.hide_password = boolean(value),
                // `configuration.Put("PlayWhenUnfocused",
                // Main.SettingPlayWhenUnfocused)` (`Main.cs:4217`, read back at
                // `:4352`). The port's old `"SettingPlayWhenUnfocused"` was its
                // own invention and stays as the alias, with the port's name
                // first so the vanilla spelling wins a file carrying both
                // (the parser's "last line wins").
                "SettingPlayWhenUnfocused" => me.play_when_unfocused = boolean(value),
                "PlayWhenUnfocused" => me.play_when_unfocused = boolean(value),
                // `configuration.Put("ShowItemText", Main.showItemText)`
                // (`Main.cs:4135`), read back at `:4409`; the port's old key
                // `"PickupText"` was its own invention (vanilla has no such key).
                "ShowItemText" => me.pickup_text = boolean(value),
                "PickupText" => me.pickup_text = boolean(value),
                // `configuration.Put("InvasionBarMode", Main.invasionProgressMode)`
                // (`Main.cs:4096`), read back at `:4332`; the port's old key was
                // `"InvasionProgressMode"`.
                "InvasionBarMode" => {
                    me.event_progress = value.parse().unwrap_or(me.event_progress).min(2)
                }
                "InvasionProgressMode" => {
                    me.event_progress = value.parse().unwrap_or(me.event_progress).min(2)
                }
                "PlacementPreview" => me.placement_preview = boolean(value),
                "HighlightNewItems" => me.highlight_new_items = boolean(value),
                "MouseShowBuildingGrid" => me.tile_grid = boolean(value),
                // `configuration.Put("GamepadDisableInstructionsDisplay",
                // Main.GamepadDisableInstructionsDisplay)` (`Main.cs:4151`),
                // read back at `:4427`; the port's old key `"GamepadInstructions"`
                // appears nowhere in the original — like every other legacy
                // spelling paired here, it was the port's own invention.
                "GamepadDisableInstructionsDisplay" => me.gamepad_instructions = boolean(value),
                "GamepadInstructions" => me.gamepad_instructions = boolean(value),
                // `configuration.Get("BackgroundParallax", ref Main.bgScroll)`
                // (`Main.cs:4407`); the slider only writes 0..=100 (`:51642`).
                "BackgroundParallax" => {
                    me.parallax_scroll =
                        value.parse().unwrap_or(me.parallax_scroll).min(100)
                }
                "FrameSkipMode" => me.frame_skip = value.parse().unwrap_or(me.frame_skip).min(2),
                // `LoadSettings` casts the stored int to the enum unchecked —
                // `Lighting.Mode = (LightMode)currentValue6` (`Main.cs:4403`),
                // which would make a 9 an undefined mode. The port instead
                // falls back to the default for anything outside the four C#
                // members; a deliberate robustness deviation, recorded here
                // (`LightMode.cs:3-9`).
                "LightingMode" => {
                    if let Some(v) = value.parse::<u8>().ok().and_then(light_mode_from_vanilla) {
                        me.light_mode = v;
                    }
                }
                // `configuration.Put("GraphicsQuality", Main.qaStyle)`
                // (`Main.cs:4130`), read back at `:4371`; the port's old key was
                // `"Quality"`.
                "GraphicsQuality" => me.quality = value.parse().unwrap_or(me.quality).min(3),
                "Quality" => me.quality = value.parse().unwrap_or(me.quality).min(3),
                "BackgroundEnabled" => me.background = boolean(value),
                // `configuration.Put("GoreVisualsAllowed", ChildSafety.Disabled)`
                // (`Main.cs:4101`), read back at `:4349`; the port's old key was
                // `"BloodAndGore"`.
                "GoreVisualsAllowed" => me.blood = boolean(value),
                "BloodAndGore" => me.blood = boolean(value),
                // `configuration.Put("SettingsEnabled_MinersWobble",
                // Main.SettingsEnabled_MinersWobble)` (`Main.cs:4153`), read back
                // at `:4540`; the port's old key was `"MinersWobble"`.
                "SettingsEnabled_MinersWobble" => me.miners_wobble = boolean(value),
                "MinersWobble" => me.miners_wobble = boolean(value),
                // `configuration.Put("SettingsEnabled_TilesSwayInWind",
                // Main.SettingsEnabled_TilesSwayInWind)` (`Main.cs:4157`), read
                // back at `:4543`; the port's old key was `"TilesSwayInWind"`.
                "SettingsEnabled_TilesSwayInWind" => me.tiles_sway = boolean(value),
                "TilesSwayInWind" => me.tiles_sway = boolean(value),
                "Zoom" => {
                    me.zoom = value.parse::<f32>().unwrap_or(me.zoom).clamp(1.0, 2.0)
                }
                // `configuration.Put("UIScale", Main._uiScaleWanted)`
                // (`Main.cs:4201`), read back by `Main.UIScale =
                // configuration.Get("UIScale", GetScaling())` (`:4487`). The
                // stored value is the **wanted** scale (the setter clamps only
                // the used value, `:2563-2572`), so it is not clamped here.
                "UIScale" => me.ui_scale = value.parse().unwrap_or(me.ui_scale),
                // The audio trio: `configuration.Put("VolumeSound", ...)`,
                // `"VolumeAmbient"` / `"VolumeMusic"` (`Main.cs:4105-4107`), read
                // back at `:4355-4357`; the port's old keys were
                // `"SoundVolume"`, `"AmbientVolume"`, `"MusicVolume"` (note the
                // swapped order of the noun and "Volume").
                "VolumeSound" => me.sound_volume = value.parse().unwrap_or(me.sound_volume),
                "SoundVolume" => me.sound_volume = value.parse().unwrap_or(me.sound_volume),
                "VolumeAmbient" => me.ambient_volume = value.parse().unwrap_or(me.ambient_volume),
                "AmbientVolume" => me.ambient_volume = value.parse().unwrap_or(me.ambient_volume),
                "VolumeMusic" => me.music_volume = value.parse().unwrap_or(me.music_volume),
                "MusicVolume" => me.music_volume = value.parse().unwrap_or(me.music_volume),
                "MapScale" => {
                    me.map_scale = value.parse::<f32>().unwrap_or(1.0).clamp(0.5, 1.0)
                }
                "MinimapFrame" => {
                    // `Preferences.Put` writes `"MinimapFrame": "Default"`
                    // (`MinimapFrameManager.cs:18`); the string runs between
                    // the quotes.
                    let v = value.trim();
                    me.minimap_frame = v
                        .strip_prefix('"')
                        .and_then(|rest| rest.split_once('"').map(|(s, _)| s))
                        .unwrap_or(v)
                        .to_string();
                }
                // `configuration.Put("MouseColor", ...)` (`Main.cs:4164-4176`)
                // is an R/G/B dict; `LoadSettings` reads the entries
                // individually and leaves the alpha alone (`:4448-4460`).
                "MouseColor" => {
                    if let Some(v) = dict_byte(value, "R") {
                        me.mouse_color[0] = v;
                    }
                    if let Some(v) = dict_byte(value, "G") {
                        me.mouse_color[1] = v;
                    }
                    if let Some(v) = dict_byte(value, "B") {
                        me.mouse_color[2] = v;
                    }
                }
                // The `"MouseBorderColor"` twin, with an `A` byte
                // (`Main.cs:4179-4197`, `:4462-4478`).
                "MouseBorderColor" => {
                    if let Some(v) = dict_byte(value, "R") {
                        me.mouse_border_color[0] = v;
                    }
                    if let Some(v) = dict_byte(value, "G") {
                        me.mouse_border_color[1] = v;
                    }
                    if let Some(v) = dict_byte(value, "B") {
                        me.mouse_border_color[2] = v;
                    }
                    if let Some(v) = dict_byte(value, "A") {
                        me.mouse_border_color[3] = v;
                    }
                }
                // `configuration.Put("SmartCursorToggle", ...)` (`Main.cs:4094`,
                // read back at `:4330`).
                "SmartCursorToggle" => me.smart_cursor_toggle = boolean(value),
                // The menuMode 2008 effect toggles, in `Main.SaveSettings`'s
                // `Put` order. `configuration.Put("UseStormEffect",
                // Main.UseStormEffects)` — singular (`Main.cs:4159`), read back
                // at `:4435`.
                "UseStormEffect" => me.use_storm_effects = boolean(value),
                // `configuration.Put("UseHeatDistortion", ...)` (`Main.cs:4160`,
                // read back at `:4437`).
                "UseHeatDistortion" => me.use_heat_distortion = boolean(value),
                // `configuration.Put("ForegroundSunlightEffects", ...)`
                // (`Main.cs:4161`, read back at `:4436`).
                "ForegroundSunlightEffects" => me.foreground_sunlight = boolean(value),
                // `configuration.Put("WaveQuality", ...)` (`Main.cs:4162`);
                // `LoadSettings` clamps the stored value to 0..=3 — values
                // above 3 to 3, negatives to 0 (`:4438-4445`).
                "WaveQuality" => {
                    if let Ok(v) = value.parse::<i32>() {
                        me.wave_quality = v.clamp(0, 3) as u8;
                    }
                }
                // `configuration.Put("FlashyEffectsWorld", ...)`
                // (`Main.cs:4215`, read back at `:4350`).
                "FlashyEffectsWorld" => me.flashy_effects_world = boolean(value),
                // `configuration.Put("UseScreenShake", ...)` (`Main.cs:4218`,
                // read back at `:4552`).
                "UseScreenShake" => me.use_screen_shake = boolean(value),
                // `configuration.Put("UseSmartAxeAfterSmartPickaxe",
                // Player.SmartCursorSettings.SmartAxeAfterPickaxe)`
                // (`Main.cs:4139`, read back at `:4412`).
                "UseSmartAxeAfterSmartPickaxe" => me.smart_axe_first = boolean(value),
                // `configuration.Put("UseSmartCursorForCommonBlocks",
                // Player.SmartCursorSettings.SmartBlocksEnabled)`
                // (`Main.cs:4138`, read back at `:4411`).
                "UseSmartCursorForCommonBlocks" => me.smart_blocks = boolean(value),
                // `configuration.Put("DoorAutoOpeningMode",
                // DoorOpeningHelper.PreferenceSettings)` (`Main.cs:4208`, read
                // back at `:4545`); the enum is Disabled/GamepadOnly/Everything.
                "DoorAutoOpeningMode" => {
                    if let Ok(v) = value.parse::<i32>() {
                        me.smart_doors = v.clamp(0, 2) as u8;
                    }
                }
                // `configuration.Put("HoverControlMode",
                // Player.Settings.HoverControl)` (`Main.cs:4209`/`:4546`).
                "HoverControlMode" => {
                    if let Ok(v) = value.parse::<i32>() {
                        me.hover_control = v.clamp(0, 3) as u8;
                    }
                }
                // `configuration.Put("SettingsEnabled_AutoReuseAllItems",
                // Main.SettingsEnabled_AutoReuseAllItems)` (`Main.cs:4158`).
                "SettingsEnabled_AutoReuseAllItems" => me.auto_reuse_all = boolean(value),
                // `configuration.Put("CraftFromNearbyChests",
                // Player.Settings.CraftFromNearbyChests)` (`Main.cs:4213`), the
                // `Player.Settings` control pair read back through the same
                // group; the port wrote no key for it before.
                "CraftFromNearbyChests" => me.craft_from_nearby_chests = boolean(value),
                // `configuration.Get("LockOnPriority", ...)` (`Main.cs:4496-4505`):
                // the stored value clamps to 0..=2 and FocusTarget (0) reloads
                // as ThreeDS (2), so 0 never survives a boot.
                "LockOnPriority" => {
                    if let Ok(v) = value.parse::<u8>() {
                        me.lock_on_priority = match v.min(2) {
                            0 => 2,
                            other => other,
                        };
                    }
                }
                // `configuration.Put("ReverseUpDownForArmorSetBonuses",
                // Main.ReversedUpDownArmorSetBonuses)` (`Main.cs:4145`, read
                // back at `:4424`).
                "ReverseUpDownForArmorSetBonuses" => me.reversed_armor_bonuses = boolean(value),
                // `configuration.Put("DisableLeftShiftTrashCan",
                // ItemSlot.Options.DisableLeftShiftTrashCan)` (`Main.cs:4141`,
                // read back at `:4416`) and the `"DisableQuickTrash"` twin
                // (`:4142`, `:4420`).
                "DisableLeftShiftTrashCan" => me.disable_left_shift_trash = boolean(value),
                "DisableQuickTrash" => me.quick_trash = boolean(value),
                // `PlayerResourceSetsManager.Configuration_OnSave` writes
                // `"PlayerResourcesSet"` (`PlayerResourceSetsManager.cs:37`);
                // `Configuration_OnLoad` defaults the missing key to `"New"`
                // (`:28`). The raw string is kept — an unknown name
                // normalizes to the first set where it is used
                // (`SetActive`, `:74-83`).
                "PlayerResourcesSet" => {
                    // `"PlayerResourcesSet": "Default"` — the string runs
                    // between the quotes.
                    let v = value.trim();
                    me.resource_sets = v
                        .strip_prefix('"')
                        .and_then(|rest| rest.split_once('"').map(|(s, _)| s))
                        .unwrap_or(v)
                        .to_string();
                }
                // `configuration.Put("PortraitPreference",
                // Main.DialoguePortraitPreference)` (`Main.cs:4220`, read back
                // at `:4551`); the byte clamps to the four enum values, whose
                // `_` arms render anything above as Detailed anyway.
                "PortraitPreference" => {
                    me.portrait_preference =
                        value.parse().unwrap_or(me.portrait_preference).min(3)
                }
                // `BigProgressBarSystem.Configuration_Save` writes
                // `"ShowBossBarHealthText"` (`BigProgressBarSystem.cs:231`),
                // read back at `:236` — a missing key keeps the static-ctor
                // `true` (`:259`).
                "ShowBossBarHealthText" => me.boss_bar_text = boolean(value),
                // `configuration.Put("FlashyEffectsInterface",
                // Main.FlashyEffectsInterface)` (`Main.cs:4216`, read back at
                // `:4351`).
                "FlashyEffectsInterface" => me.flashy_effects_interface = boolean(value),
                // The 14 gameplay binds (`Main.cs:4582-4595`) and the 6 map
                // binds (`:4596-4600`). A name the reader does not know keeps
                // the row's default, like the vanilla `Get` leaving the field
                // untouched when the key is absent.
                "KeyUp" | "KeyLeft" | "KeyDown" | "KeyRight" | "KeyJump"
                | "KeyThrowItem" | "KeyQuickHeal" | "KeyQuickMana" | "KeyQuickBuff"
                | "KeyUseHook" | "KeyAutoSelect" | "KeyInventory" | "KeySmartCursor"
                | "KeyMount" => me.set_keybind(key, json_bare_string(value)),
                "KeyMapStyle" | "KeyFullscreenMap" | "KeyMapZoomIn" | "KeyMapZoomOut"
                | "KeyMapAlphaUp" | "KeyMapAlphaDown" => {
                    me.set_map_bind(key, json_bare_string(value))
                }
                _ => {}
            }
        }
        me
    }

    /// `Main.SaveSettings`: written to `config.json` next to the worlds.
    ///
    /// The key names are the **vanilla** `SaveSettings` `Put` names
    /// (`Main.cs:4089-4231`) so a vanilla `config.json` round-trips; the port's
    /// own pre-parity names are only accepted as aliases by [`Self::parse`].
    /// The order stays as written here (this port's long-standing Put order).
    pub fn save(&self) -> std::io::Result<()> {
        let Some(path) = Self::path() else {
            return Ok(());
        };
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir)?;
        }
        std::fs::write(path, self.to_json())
    }

    pub fn to_json(&self) -> String {
        format!(
            "{{\n  \"AutoSave\": {},\n  \"AutoPause\": {},\n  \"Language\": \"{}\",\n  \
             \"MapEnabled\": {},\n  \
             \"HidePasswords\": {},\n  \"PlayWhenUnfocused\": {},\n  \"ShowItemText\": {},\n  \
             \"InvasionBarMode\": {},\n  \"PlacementPreview\": {},\n  \
             \"HighlightNewItems\": {},\n  \"MouseShowBuildingGrid\": {},\n  \
             \"GamepadDisableInstructionsDisplay\": {},\n  \"BackgroundParallax\": {},\n  \
             \"FrameSkipMode\": {},\n  \
             \"LightingMode\": {},\n  \"GraphicsQuality\": {},\n  \"BackgroundEnabled\": {},\n  \
             \"GoreVisualsAllowed\": {},\n  \"SettingsEnabled_MinersWobble\": {},\n  \
             \"SettingsEnabled_TilesSwayInWind\": {},\n  \
             \"UseStormEffect\": {},\n  \"UseHeatDistortion\": {},\n  \
             \"ForegroundSunlightEffects\": {},\n  \"WaveQuality\": {},\n  \
             \"FlashyEffectsWorld\": {},\n  \"UseScreenShake\": {},\n  \
             \"Zoom\": {:.3},\n  \"UIScale\": {:.3},\n  \"VolumeMusic\": {:.3},\n  \"VolumeSound\": {:.3},\n  \
             \"VolumeAmbient\": {:.3},\n  \"MapScale\": {:.3},\n  \
             \"MouseColor\": {{\"R\": {}, \"G\": {}, \"B\": {}}},\n  \
             \"MouseBorderColor\": {{\"R\": {}, \"G\": {}, \"B\": {}, \"A\": {}}},\n  \
             \"SmartCursorToggle\": {},\n  \"UseSmartAxeAfterSmartPickaxe\": {},\n  \
             \"UseSmartCursorForCommonBlocks\": {},\n  \"DoorAutoOpeningMode\": {},\n  \
             \"HoverControlMode\": {},\n  \"SettingsEnabled_AutoReuseAllItems\": {},\n  \
             \"CraftFromNearbyChests\": {},\n  \
             \"LockOnPriority\": {},\n  \
             \"ReverseUpDownForArmorSetBonuses\": {},\n  \
             \"DisableLeftShiftTrashCan\": {},\n  \"DisableQuickTrash\": {},\n  \
             \"PlayerResourcesSet\": \"{}\",\n  \"PortraitPreference\": {},\n  \
             \"ShowBossBarHealthText\": {},\n  \"FlashyEffectsInterface\": {},\n  \
             \"MinimapFrame\": \"{}\"{}\n}}\n",
            self.auto_save,
            self.auto_pause,
            self.language,
            self.map_enabled,
            self.hide_password,
            self.play_when_unfocused,
            self.pickup_text,
            self.event_progress,
            self.placement_preview,
            self.highlight_new_items,
            self.tile_grid,
            self.gamepad_instructions,
            self.parallax_scroll,
            self.frame_skip,
            // The stored byte is the C# ordinal, so Color (the port's 0) goes
            // out as 3 (`LightMode.cs:3-9`).
            light_mode_to_vanilla(self.light_mode),
            self.quality,
            self.background,
            self.blood,
            self.miners_wobble,
            self.tiles_sway,
            // The menuMode 2008 toggles, written under the vanilla keys in the
            // `Main.SaveSettings` `Put` order (`Main.cs:4159-4162`, `:4215`,
            // `:4218`).
            self.use_storm_effects,
            self.use_heat_distortion,
            self.foreground_sunlight,
            self.wave_quality,
            self.flashy_effects_world,
            self.use_screen_shake,
            self.zoom,
            self.ui_scale,
            self.music_volume,
            self.sound_volume,
            self.ambient_volume,
            self.map_scale,
            // `configuration.Put("MouseColor", new Dictionary<string, byte> ...)`
            // (`Main.cs:4164-4176`) — one-line R/G/B dict, `A` only on the
            // border colour (`:4179-4197`).
            self.mouse_color[0],
            self.mouse_color[1],
            self.mouse_color[2],
            self.mouse_border_color[0],
            self.mouse_border_color[1],
            self.mouse_border_color[2],
            self.mouse_border_color[3],
            self.smart_cursor_toggle,
            self.smart_axe_first,
            self.smart_blocks,
            // `DoorOpeningHelper.PreferenceSettings`, `Player.Settings.HoverControl`
            // and `Main.SettingsEnabled_AutoReuseAllItems` (`Main.cs:4208`,
            // `:4209`, `:4158`) — the three rows the options window drew as
            // "Parked" because the port carried no field for them.
            self.smart_doors,
            self.hover_control,
            self.auto_reuse_all,
            // `configuration.Put("CraftFromNearbyChests",
            // Player.Settings.CraftFromNearbyChests)` (`Main.cs:4213`); the port
            // keeps its own position in this list (Phase 2 only renames keys, it
            // does not reorder the writer).
            self.craft_from_nearby_chests,
            self.lock_on_priority,
            // The Task 5 keys, written under the vanilla names (`Main.cs:4145`,
            // `:4141`, `:4142`, `:4220`; `BigProgressBarSystem.cs:231`;
            // `Main.cs:4216`; `PlayerResourceSetsManager.cs:37`).
            self.reversed_armor_bonuses,
            self.disable_left_shift_trash,
            self.quick_trash,
            self.resource_sets,
            self.portrait_preference,
            self.boss_bar_text,
            self.flashy_effects_interface,
            self.minimap_frame,
            self.bind_block(),
        )
    }

    /// The `Key*` rows of `config.json`, as a comma-prefixed JSON fragment.
    ///
    /// Vanilla reads these back (`Main.cs:4582-4600`) but never writes them
    /// from `SaveSettings`; the port writes them because its bind table lives
    /// in `Settings` (the vanilla keeps the authority in
    /// `PlayerInput.CurrentProfile`, a file the port does not have). The names
    /// are the vanilla's, so a vanilla `config.json` still parses.
    fn bind_block(&self) -> String {
        let mut out = String::new();
        for (i, action) in crate::input_map::Action::ALL.iter().enumerate() {
            out.push_str(&format!(
                ",
  \"{}\": \"{}\"",
                crate::input_map::Action::CONFIG_KEYS[i],
                self.keybinds[action.index()]
            ));
        }
        for (i, action) in crate::input_map::MapAction::ALL.iter().enumerate() {
            out.push_str(&format!(
                ",
  \"{}\": \"{}\"",
                crate::input_map::MapAction::CONFIG_KEYS[i],
                self.map_binds[action.index()]
            ));
        }
        out
    }

    /// One `Key*` row of [`Settings::parse`]. The row index comes from the
    /// `config.json` name, so the two tables cannot drift.
    fn set_keybind(&mut self, key: &str, value: String) {
        for (i, name) in crate::input_map::Action::CONFIG_KEYS.iter().enumerate() {
            if *name == key {
                // An unknown key name (a hand edit) would bind nothing, so the
                // row keeps its default instead of going dead.
                if crate::input_map::parse_key(&value.trim()).is_some() {
                    self.keybinds[i] = value.trim().to_string();
                }
                return;
            }
        }
    }

    fn set_map_bind(&mut self, key: &str, value: String) {
        for (i, name) in crate::input_map::MapAction::CONFIG_KEYS.iter().enumerate() {
            if *name == key {
                if crate::input_map::parse_key(&value.trim()).is_some() {
                    self.map_binds[i] = value.trim().to_string();
                }
                return;
            }
        }
    }
}

/// `Main.UIScaleMax` (`Main.cs:2576-2583`): `Math.Max(1f, Math.Min(h / 600f,
/// w / 800f))` over the *real* (pre-zoom) screen size
/// (`PlayerInput.RealScreenWidth`/`RealScreenHeight`). The `UIScale` setter
/// clamps a wanted scale down to this and never below the `1f` floor.
pub fn ui_scale_max(real_screen_w: f32, real_screen_h: f32) -> f32 {
    1.0f32.max((real_screen_h / 600.0).min(real_screen_w / 800.0))
}

/// The bare text of a JSON string value, as the `parse` pass sees it (the
/// value arrives including its quotes, e.g. `\"W\"` -> `W`).
fn json_bare_string(value: &str) -> String {
    let v = value.trim();
    v.strip_prefix('"')
        .and_then(|rest| rest.split_once('"').map(|(s, _)| s))
        .unwrap_or(v)
        .to_string()
}

/// The stored `LightingMode` byte (`Terraria.Graphics.Light.LightMode`:
/// White 0, Retro 1, Trippy 2, Color 3 — `LightMode.cs:3-9`) → the port's
/// internal `LightMode` order (`Color` 0, `White` 1, `Retro` 2, `Trippy` 3,
/// `tr-core/src/lighting/params.rs:12-17`). `None` for anything outside the
/// four enum members, which is the robustness deviation noted on the
/// `"LightingMode"` arm of `parse`.
fn light_mode_from_vanilla(stored: u8) -> Option<u8> {
    match stored {
        0 => Some(1), // White
        1 => Some(2), // Retro
        2 => Some(3), // Trippy
        3 => Some(0), // Color
        _ => None,
    }
}

/// The port `LightMode` byte → the stored `LightingMode` ordinal, the inverse
/// of [`light_mode_from_vanilla`]. Anything outside the four port variants
/// writes `Color` (3), the vanilla default (`Lighting.cs:96`).
fn light_mode_to_vanilla(port: u8) -> u8 {
    match port {
        1 => 0, // White
        2 => 1, // Retro
        3 => 2, // Trippy
        _ => 3, // Color
    }
}

/// One byte of a vanilla one-line colour dict (`"MouseColor": {"R": 255, ...}`):
/// finds `"key"` and reads the decimal number that follows it. `LoadSettings`
/// reads the same entries individually out of the deserialized dictionary
/// (`Main.cs:4448-4478`).
fn dict_byte(dict: &str, key: &str) -> Option<u8> {
    let rest = dict.split_once(&format!("\"{key}\""))?.1;
    let digits: String = rest
        .chars()
        .skip_while(|c| !c.is_ascii_digit())
        .take_while(|c| c.is_ascii_digit())
        .collect();
    digits.parse().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_match_the_vanilla_options() {
        let s = Settings::default();
        assert!(s.auto_save);
        assert!(!s.auto_pause);
        assert!(s.map_enabled);
        assert!(!s.hide_password);
        // `Main.SettingPlayWhenUnfocused = false` (`Main.cs:66512`) — the
        // port's `true` was a plain parity break.
        assert!(!s.play_when_unfocused);
        assert!(s.placement_preview);
        assert!(!s.tile_grid);
        // `Main.bgScroll` is a plain static (`Main.cs:2429`) that a fresh config
        // seeds to 60: `caveParallax = 0.88f` (`Main.cs:66809`) survives the
        // keyless `Get("Parallax", ...)`, then `(int)((1f - 0.88f) * 500f)` = 60
        // and the keyless `Get("BackgroundParallax", ...)` keeps it
        // (`Main.cs:4405-4408`).
        assert_eq!(s.parallax_scroll, 60);
        // `Main.FrameSkipMode = FrameSkipMode.Subtle` (`Main.cs:67204`), the
        // enum's 2 (`FrameSkipMode.cs:3-8`) — the port's 0 (Off) was wrong.
        assert_eq!(s.frame_skip, 2);
        assert_eq!(s.light_mode, 0);
        assert_eq!(s.quality, 0);
        assert!(s.background);
        assert!(s.blood);
        // `Main.invasionProgressMode = 2` (`Main.cs:67080`).
        assert_eq!(s.event_progress, 2);
        // `Main.musicVolume = 1f` (`Main.cs:66857`), `Main.soundVolume = 1f`
        // (`:66859`), `Main.ambientVolume = 1f` (`:66858`).
        assert_eq!(s.music_volume, 1.0);
        assert_eq!(s.sound_volume, 1.0);
        assert_eq!(s.ambient_volume, 1.0);
        assert_eq!(s.map_scale, 1.0);
        assert_eq!(s.minimap_frame, "Default");
        // `Main.mouseColor = new Color(255, 50, 95)` (`Main.cs:67199`) and
        // `Main.MouseBorderColor = new Color(64, 64, 64, 64)` (`:66514`): the
        // static-initializer defaults, not field declarations.
        assert_eq!(s.mouse_color, [255, 50, 95, 255]);
        assert_eq!(s.mouse_border_color, [64, 64, 64, 64]);
        // `Main.cSmartCursorModeIsToggleAndNotHold = true` (`:67178`); the two
        // `SmartCursorSettings` statics are plain bools (`Player.cs:331-334`).
        assert!(s.smart_cursor_toggle);
        assert!(!s.smart_axe_first);
        assert!(!s.smart_blocks);
        // `LockOnHelper.UseMode` boots into ThreeDS: `LoadSettings` clamps the
        // keyless -1 to 0 and remaps FocusTarget to ThreeDS
        // (`Main.cs:4496-4505`).
        assert_eq!(s.lock_on_priority, 2);
        // The menuMode 2008 statics (`Main.cs:66571-66576`), not the field
        // declarations (`Main.cs:427-437`).
        assert!(s.use_storm_effects);
        assert!(s.foreground_sunlight);
        assert!(s.use_heat_distortion);
        assert_eq!(s.wave_quality, 3);
        assert!(s.use_screen_shake);
        assert!(s.flashy_effects_world);
        // The Task 5 statics: `Main.ReversedUpDownArmorSetBonuses` is a plain
        // static (`Main.cs:273`), the `ItemSlot.Options` defaults live in the
        // `static Options()` ctor (`ItemSlot.cs:36-37`), the resource-set key
        // deviates from vanilla's `"New"` (recorded for Task 7), the portrait
        // enum's 0 is Detailed (`Main.cs:186-189`), `ShowText` is set true by
        // the static ctor (`BigProgressBarSystem.cs:259`) and
        // `FlashyEffectsInterface` closes the `:66571-66578` block.
        assert!(!s.reversed_armor_bonuses);
        assert!(s.disable_left_shift_trash);
        assert!(!s.quick_trash);
        assert_eq!(s.resource_sets, "Default");
        assert_eq!(s.portrait_preference, 0);
        assert!(s.boss_bar_text, "the static ctor sets ShowText = true");
        assert!(s.flashy_effects_interface);
    }

    #[test]
    fn json_round_trips() {
        let mut s = Settings::default();
        s.auto_save = false;
        s.auto_pause = true;
        s.map_enabled = false;
        s.hide_password = true;
        s.event_progress = 2;
        s.frame_skip = 2;
        // The port's Trippy, written as the C# 2 and read back as 3 — the
        // ordinal translation (`LightMode.cs:3-9` vs `params.rs:12-17`).
        s.light_mode = 3;
        s.quality = 1;
        s.background = false;
        s.blood = false;
        s.miners_wobble = false;
        s.tiles_sway = false;
        s.music_volume = 0.4;
        s.sound_volume = 0.2;
        s.ambient_volume = 0.9;
        s.parallax_scroll = 37;
        s.map_scale = 0.5;
        s.ui_scale = 0.8;
        s.minimap_frame = "Golden".to_string();
        s.mouse_color = [1, 2, 3, 255];
        s.mouse_border_color = [4, 5, 6, 7];
        s.smart_cursor_toggle = false;
        s.smart_axe_first = true;
        s.smart_blocks = true;
        s.craft_from_nearby_chests = false;
        s.lock_on_priority = 1;
        s.use_storm_effects = false;
        s.foreground_sunlight = false;
        s.use_heat_distortion = false;
        s.wave_quality = 1;
        s.use_screen_shake = false;
        s.flashy_effects_world = false;
        s.reversed_armor_bonuses = true;
        s.disable_left_shift_trash = false;
        s.quick_trash = true;
        s.resource_sets = "HorizontalBars".to_string();
        s.portrait_preference = 2;
        s.boss_bar_text = false;
        s.flashy_effects_interface = false;
        let back = Settings::parse(&s.to_json());
        assert_eq!(back.auto_save, s.auto_save);
        assert_eq!(back.auto_pause, s.auto_pause);
        assert_eq!(back.map_enabled, s.map_enabled);
        assert_eq!(back.hide_password, s.hide_password);
        assert_eq!(back.event_progress, s.event_progress);
        assert_eq!(back.frame_skip, s.frame_skip);
        assert_eq!(back.light_mode, s.light_mode);
        assert_eq!(back.quality, s.quality);
        assert_eq!(back.background, s.background);
        assert_eq!(back.blood, s.blood);
        assert_eq!(back.miners_wobble, s.miners_wobble);
        assert_eq!(back.tiles_sway, s.tiles_sway);
        assert!((back.music_volume - s.music_volume).abs() < 1e-3);
        assert!((back.sound_volume - s.sound_volume).abs() < 1e-3);
        assert!((back.ambient_volume - s.ambient_volume).abs() < 1e-3);
        assert_eq!(back.parallax_scroll, 37);
        assert!((back.map_scale - s.map_scale).abs() < 1e-3);
        assert!((back.ui_scale - s.ui_scale).abs() < 1e-3);
        assert_eq!(back.minimap_frame, s.minimap_frame);
        assert_eq!(back.mouse_color, [1, 2, 3, 255]);
        assert_eq!(back.mouse_border_color, [4, 5, 6, 7]);
        assert!(!back.smart_cursor_toggle);
        assert!(back.smart_axe_first);
        assert!(back.smart_blocks);
        assert!(!back.craft_from_nearby_chests);
        assert_eq!(back.lock_on_priority, 1);
        assert!(!back.use_storm_effects);
        assert!(!back.foreground_sunlight);
        assert!(!back.use_heat_distortion);
        assert_eq!(back.wave_quality, 1);
        assert!(!back.use_screen_shake);
        assert!(!back.flashy_effects_world);
        assert!(back.reversed_armor_bonuses);
        assert!(!back.disable_left_shift_trash);
        assert!(back.quick_trash);
        assert_eq!(back.resource_sets, "HorizontalBars");
        assert_eq!(back.portrait_preference, 2);
        assert!(!back.boss_bar_text);
        assert!(!back.flashy_effects_interface);
    }

    #[test]
    fn effect_keys_read_the_vanilla_names() {
        // The vanilla keys verified against `Main.SaveSettings`/`LoadSettings`:
        // "UseStormEffect" is singular (`Main.cs:4159`, `:4435`),
        // "UseHeatDistortion" (`:4160`, `:4437`),
        // "ForegroundSunlightEffects" (`:4161`, `:4436`), "WaveQuality"
        // (`:4162`) load-clamped to 0..=3 (`:4438-4445`),
        // "FlashyEffectsWorld" (`:4215`, `:4350`), "UseScreenShake" (`:4218`,
        // `:4552`).
        let s = Settings::parse(
            "{\n \"UseStormEffect\": false,\n \"UseHeatDistortion\": false,\n \
             \"ForegroundSunlightEffects\": false,\n \"WaveQuality\": 2,\n \
             \"FlashyEffectsWorld\": false,\n \"UseScreenShake\": false\n}",
        );
        assert!(!s.use_storm_effects);
        assert!(!s.use_heat_distortion);
        assert!(!s.foreground_sunlight);
        assert_eq!(s.wave_quality, 2);
        assert!(!s.flashy_effects_world);
        assert!(!s.use_screen_shake);
        // The load clamp (`Main.cs:4439-4445`).
        let s = Settings::parse("{\n \"WaveQuality\": 9\n}");
        assert_eq!(s.wave_quality, 3);
        let s = Settings::parse("{}");
        assert_eq!(s.wave_quality, 3, "a missing key keeps the default");
    }

    #[test]
    fn unknown_keys_and_garbage_are_ignored() {
        let s = Settings::parse("{\n \"Nope\": true,\n \"AutoSave\": false,\n \"LightingMode\": 9\n}");
        assert!(!s.auto_save);
        // Vanilla would cast 9 to an undefined mode (`Main.cs:4403`); the port
        // keeps the default instead (see the `"LightingMode"` arm).
        assert_eq!(s.light_mode, 0, "out of range keeps the default");
    }

    #[test]
    fn lighting_mode_uses_the_vanilla_ordinal() {
        // The C# enum is White 0, Retro 1, Trippy 2, Color 3
        // (`Terraria/Graphics/Light/LightMode.cs:3-9`) while the port's
        // `LightMode` is Color 0, White 1, Retro 2, Trippy 3
        // (`tr-core/src/lighting/params.rs:12-17`). The persisted byte is the
        // C# ordinal, so the two directions below are the whole translation.
        let cases = [(0u8, 1u8), (1, 2), (2, 3), (3, 0)];
        for (stored, port) in cases {
            let text = format!("{{\n \"LightingMode\": {stored}\n}}");
            let s = Settings::parse(&text);
            assert_eq!(s.light_mode, port, "stored {stored} reads as port {port}");

            let mut w = Settings::default();
            w.light_mode = port;
            let json = w.to_json();
            assert!(
                json.contains(&format!("\"LightingMode\": {stored},")),
                "port {port} writes back as {stored}: {json}"
            );
        }

        // DW-1.2's fixture: a vanilla-shaped config's Color.
        let s = Settings::parse("{\n \"LightingMode\": 3\n}");
        assert_eq!(s.light_mode, 0, "vanilla 3 is Color, the port's 0");
        let json = Settings::default().to_json();
        assert!(
            json.contains("\"LightingMode\": 3,"),
            "the default Color writes vanilla's 3: {json}"
        );

        // Out of range (above the enum) keeps the default — vanilla casts
        // unchecked (`Main.cs:4398-4403`), which is the deviation this arm
        // avoids. Only u8-parseable values reach the translation; anything the
        // `-1` byte range cannot hold fails `parse::<u8>()` instead.
        for bad in ["4", "255", "-1", "\"x\"", "null"] {
            let text = format!("{{\n \"LightingMode\": {bad}\n}}");
            let s = Settings::parse(&text);
            assert_eq!(s.light_mode, 0, "{bad} falls back to the default");
        }
    }

    #[test]
    fn craft_from_nearby_chests_round_trips_the_vanilla_key() {
        // `configuration.Put("CraftFromNearbyChests",
        // Player.Settings.CraftFromNearbyChests)` (`Main.cs:4213`) — the field
        // existed with no key, so it never survived a config round-trip.
        assert!(Settings::default().craft_from_nearby_chests, "Player.cs:406");
        for stored in [true, false] {
            let text = format!("{{\n \"CraftFromNearbyChests\": {stored}\n}}");
            let s = Settings::parse(&text);
            assert_eq!(s.craft_from_nearby_chests, stored);
            let mut w = Settings::default();
            w.craft_from_nearby_chests = stored;
            let json = w.to_json();
            assert!(
                json.contains(&format!("\"CraftFromNearbyChests\": {stored},")),
                "the writer persists the key: {json}"
            );
        }
        // Garbage coerces to false, exactly like the port's other fifteen
        // boolean arms (`boolean` in `parse`): the whole value token — quotes
        // included — must equal "true". Vanilla's `Preferences.Get` would
        // instead keep the field default, its `try`/`catch` swallowing the
        // `Convert.ChangeType` failure
        // (`Terraria/IO/Preferences.cs:176-190`); that coercion is port-wide
        // and pre-existing, so this key follows its siblings rather than
        // standing alone — Phase 2 owns the boolean-arm semantics.
        for bad in ["\"maybe\"", "7", "null", "\"True\"", "TRUE!"] {
            let text = format!("{{\n \"CraftFromNearbyChests\": {bad}\n}}");
            let s = Settings::parse(&text);
            assert!(!s.craft_from_nearby_chests, "{bad} coerces to false");
        }
        // The comparison is case-insensitive; only the bare literal counts.
        for good in ["true", "True", "TRUE"] {
            let text = format!("{{\n \"CraftFromNearbyChests\": {good}\n}}");
            let s = Settings::parse(&text);
            assert!(s.craft_from_nearby_chests, "{good} is truthy");
        }
        // A key absent from the file leaves the default alone.
        assert!(Settings::parse("{}").craft_from_nearby_chests, "Player.cs:406");
    }

    #[test]
    fn parallax_scroll_reads_the_vanilla_key() {
        // `Main.SaveSettings` writes "BackgroundParallax" (`Main.cs:4134`) and
        // `LoadSettings` reads it back (`:4407`), so a vanilla config.json
        // round-trips. The `.min(100)` only clamps 101..=255 (values the u8
        // can still parse); anything >= 256 fails the parse and falls back to
        // the default untouched.
        let s = Settings::parse("{\n \"BackgroundParallax\": 50\n}");
        assert_eq!(s.parallax_scroll, 50);
        let s = Settings::parse("{\n \"BackgroundParallax\": 200\n}");
        assert_eq!(s.parallax_scroll, 100, "101..=255 clamp to the slider max");
        let s = Settings::parse("{\n \"BackgroundParallax\": 500\n}");
        assert_eq!(s.parallax_scroll, 60, "u8 parse fails, the default stays");
        let s = Settings::parse("{\n \"BackgroundParallax\": 90\n}");
        assert_eq!(s.parallax_scroll, 90);
    }

    #[test]
    fn colour_keys_read_the_vanilla_dictionaries() {
        // `Main.SaveSettings` writes one-line `"MouseColor": {"R": .., "G": ..,
        // "B": ..}` (`Main.cs:4164-4176`) and `"MouseBorderColor"` with an `A`
        // byte (`:4179-4197`); `LoadSettings` reads the entries individually
        // and leaves the cursor colour's alpha alone (`:4448-4478`).
        let s = Settings::parse(
            "{\n \"MouseColor\": {\"R\": 10, \"G\": 20, \"B\": 30},\n \
             \"MouseBorderColor\": {\"R\": 40, \"G\": 50, \"B\": 60, \"A\": 70}\n}",
        );
        assert_eq!(s.mouse_color, [10, 20, 30, 255]);
        assert_eq!(s.mouse_border_color, [40, 50, 60, 70]);
        // A partial dict keeps the untouched channels, like the C#
        // `TryGetValue` guards.
        let s = Settings::parse("{\n \"MouseColor\": {\"G\": 90}\n}");
        assert_eq!(s.mouse_color, [255, 90, 95, 255]);
    }

    #[test]
    fn lock_on_priority_loads_through_the_vanilla_chain() {
        // `Main.cs:4496-4505`: the stored value clamps to 0..=2 and FocusTarget
        // (0) reloads as ThreeDS (2); a missing key falls through -1 -> 0 -> 2.
        let s = Settings::parse("{}");
        assert_eq!(s.lock_on_priority, 2, "a fresh config boots into ThreeDS");
        let s = Settings::parse("{\n \"LockOnPriority\": 0\n}");
        assert_eq!(s.lock_on_priority, 2);
        let s = Settings::parse("{\n \"LockOnPriority\": 1\n}");
        assert_eq!(s.lock_on_priority, 1);
        let s = Settings::parse("{\n \"LockOnPriority\": 9\n}");
        assert_eq!(s.lock_on_priority, 2, "clamped to 0..=2");
    }

    #[test]
    fn task_5_keys_read_the_vanilla_names() {
        // Verified against the savers/loaders: "ReverseUpDownForArmorSetBonuses"
        // (`Main.cs:4145`/`:4424`), "DisableLeftShiftTrashCan" (`:4141`/
        // `:4416`), "DisableQuickTrash" (`:4142`/`:4420`),
        // "PlayerResourcesSet" (`PlayerResourceSetsManager.cs:37`/`:28`),
        // "PortraitPreference" (`Main.cs:4220`/`:4551`),
        // "ShowBossBarHealthText" (`BigProgressBarSystem.cs:231`/`:236`),
        // "FlashyEffectsInterface" (`Main.cs:4216`/`:4351`).
        let s = Settings::parse(
            "{\n \"ReverseUpDownForArmorSetBonuses\": true,\n \
             \"DisableLeftShiftTrashCan\": false,\n \"DisableQuickTrash\": true,\n \
             \"PlayerResourcesSet\": \"New\",\n \"PortraitPreference\": 3,\n \
             \"ShowBossBarHealthText\": false,\n \"FlashyEffectsInterface\": false\n}",
        );
        assert!(s.reversed_armor_bonuses);
        assert!(!s.disable_left_shift_trash);
        assert!(s.quick_trash);
        assert_eq!(s.resource_sets, "New", "a vanilla config keeps its own set");
        assert_eq!(s.portrait_preference, 3);
        assert!(!s.boss_bar_text);
        assert!(!s.flashy_effects_interface);
        // The portrait byte clamps to the four enum values
        // (`DialoguePortraitDrawOption`, `Main.cs:186-189`).
        let s = Settings::parse("{\n \"PortraitPreference\": 9\n}");
        assert_eq!(s.portrait_preference, 3);
        let s = Settings::parse("{\n \"PlayerResourcesSet\": \"Nope\"\n}");
        assert_eq!(
            s.resource_sets, "Nope",
            "the raw key is kept; the label normalizes on use"
        );
    }

    // ------------------------------------------------------------------
    // Phase 2 — the vanilla key names (writer) and the alias arms (reader).
    // ------------------------------------------------------------------

    /// The key names of a `to_json()` document, in file order. The writer's one
    /// `"key": value` per line makes a line split enough: the one-line
    /// `"MouseColor"` / `"MouseBorderColor"` dicts yield a single key each,
    /// and their inner `"R"`/`"G"`/`"B"`/`"A"` entries never start a line.
    /// (Only the bare `{` / `}` lines are skipped.)
    fn json_keys(json: &str) -> Vec<&str> {
        json.lines()
            .map(|line| line.trim().trim_end_matches(','))
            .filter_map(|line| line.strip_prefix('"'))
            .filter_map(|line| line.split_once("\":"))
            .map(|(key, _)| key)
            .collect()
    }

    /// A vanilla-shaped `config.json`: the ten renames (`Main.cs:4096-4157`),
    /// the `"HidePasswords"` spelling (`:4144`) and the already-verified keys.
    fn vanilla_renamed_config() -> &'static str {
        "{\n  \"AutoSave\": false,\n  \"AutoPause\": true,\n  \"Language\": \"ru-RU\",\n \
         \"MapEnabled\": false,\n  \"HidePasswords\": true,\n \
         \"PlayWhenUnfocused\": true,\n  \"ShowItemText\": false,\n \
         \"InvasionBarMode\": 1,\n  \"PlacementPreview\": false,\n \
         \"HighlightNewItems\": false,\n  \"MouseShowBuildingGrid\": true,\n \
         \"GamepadDisableInstructionsDisplay\": false,\n \
         \"BackgroundParallax\": 25,\n  \"FrameSkipMode\": 1,\n  \"LightingMode\": 1,\n \
         \"GraphicsQuality\": 2,\n  \"BackgroundEnabled\": false,\n \
         \"GoreVisualsAllowed\": false,\n  \"SettingsEnabled_MinersWobble\": false,\n \
         \"SettingsEnabled_TilesSwayInWind\": false,\n  \"UseStormEffect\": true,\n \
         \"UseHeatDistortion\": true,\n  \"ForegroundSunlightEffects\": true,\n \
         \"WaveQuality\": 1,\n  \"FlashyEffectsWorld\": false,\n  \"UseScreenShake\": false,\n \
         \"Zoom\": 1.5,\n  \"VolumeMusic\": 0.25,\n  \"VolumeSound\": 0.5,\n \
         \"VolumeAmbient\": 0.75,\n  \"MapScale\": 0.75,\n \
         \"MouseColor\": {\"R\": 11, \"G\": 12, \"B\": 13},\n \
         \"MouseBorderColor\": {\"R\": 14, \"G\": 15, \"B\": 16, \"A\": 17},\n \
         \"SmartCursorToggle\": false,\n  \"UseSmartAxeAfterSmartPickaxe\": true,\n \
         \"UseSmartCursorForCommonBlocks\": true,\n  \"CraftFromNearbyChests\": false,\n \
         \"LockOnPriority\": 1,\n  \"ReverseUpDownForArmorSetBonuses\": true,\n \
         \"DisableLeftShiftTrashCan\": false,\n  \"DisableQuickTrash\": true,\n \
         \"PlayerResourcesSet\": \"HorizontalBars\",\n  \"PortraitPreference\": 3,\n \
         \"ShowBossBarHealthText\": false,\n  \"FlashyEffectsInterface\": false,\n \
         \"MinimapFrame\": \"Golden\"\n}\n"
    }

    #[test]
    fn test_DW_2_1_vanilla_fixture_reads_the_renamed_keys() {
        // A real `Main.SaveSettings` file: the port must read the renamed keys
        // without the aliases. Every non-default value below has to survive.
        let s = Settings::parse(vanilla_renamed_config());
        assert!(!s.auto_save);
        assert!(s.auto_pause);
        assert_eq!(s.language, "ru-RU");
        assert!(!s.map_enabled);
        // `HidePasswords` (`Main.cs:4144`, `Get` `:4422`).
        assert!(s.hide_password);
        assert!(s.play_when_unfocused);
        // `ShowItemText` (`:4135`, `Get` `:4409`).
        assert!(!s.pickup_text);
        // `InvasionBarMode` (`:4096`, `Get` `:4332`).
        assert_eq!(s.event_progress, 1);
        assert!(!s.placement_preview);
        assert!(!s.highlight_new_items);
        assert!(s.tile_grid);
        // `GamepadDisableInstructionsDisplay` (`:4151`, `Get` `:4427`).
        assert!(!s.gamepad_instructions);
        assert_eq!(s.parallax_scroll, 25);
        assert_eq!(s.frame_skip, 1);
        // The Phase 1 ordinal translation still rides on top of the renames:
        // vanilla's 1 is Retro, the port's 2.
        assert_eq!(s.light_mode, 2);
        // `GraphicsQuality` (`:4130`, `Get` `:4371`).
        assert_eq!(s.quality, 2);
        assert!(!s.background);
        // `GoreVisualsAllowed` (`:4101`, `Get` `:4349`).
        assert!(!s.blood);
        // `SettingsEnabled_MinersWobble` (`:4153`, `Get` `:4540`) and
        // `SettingsEnabled_TilesSwayInWind` (`:4157`, `Get` `:4543`).
        assert!(!s.miners_wobble);
        assert!(!s.tiles_sway);
        // The audio trio (`:4105-4107`, `Get` `:4355-4357`) — note the noun
        // comes first, so the port's old names do not appear at all.
        assert_eq!(s.music_volume, 0.25);
        assert_eq!(s.sound_volume, 0.5);
        assert_eq!(s.ambient_volume, 0.75);
        assert!((s.zoom - 1.5).abs() < 1e-3);
        assert!((s.map_scale - 0.75).abs() < 1e-3);
        assert_eq!(s.mouse_color, [11, 12, 13, 255]);
        assert_eq!(s.mouse_border_color, [14, 15, 16, 17]);
        assert!(!s.smart_cursor_toggle);
        assert!(s.smart_axe_first);
        assert!(s.smart_blocks);
        // Phase 1's key (`Main.cs:4213`) reads through the same file.
        assert!(!s.craft_from_nearby_chests);
        assert_eq!(s.lock_on_priority, 1);
        assert!(s.reversed_armor_bonuses);
        assert!(!s.disable_left_shift_trash);
        assert!(s.quick_trash);
        assert_eq!(s.resource_sets, "HorizontalBars");
        assert_eq!(s.portrait_preference, 3);
        assert!(!s.boss_bar_text);
        assert!(!s.flashy_effects_interface);
        assert_eq!(s.minimap_frame, "Golden");
    }

    #[test]
    fn tr_zoom_clamp_stays_in_vanilla_range_s1_b_5() {
        // S1-B-5 / Main.cs:16801-16815: GameZoomTarget clamps to 1.0..2.0 and
        // TR_ZOOM must not widen it.
        assert_eq!(0.25f32.clamp(1.0, 2.0), 1.0);
        assert_eq!(8.0f32.clamp(1.0, 2.0), 2.0);
        assert_eq!(1.0f32.clamp(1.0, 2.0), 1.0);
        assert_eq!(2.0f32.clamp(1.0, 2.0), 2.0);
        assert_eq!(1.5f32.clamp(1.0, 2.0), 1.5);
        let s = Settings::parse("{\n  \"Zoom\": 9.0\n}\n");
        assert_eq!(s.zoom, 2.0, "persisted zoom must clamp to vanilla range");
        let s = Settings::parse("{\n  \"Zoom\": 0.1\n}\n");
        assert_eq!(s.zoom, 1.0, "persisted zoom must clamp to vanilla range");
    }

    #[test]
    fn test_DW_2_2_legacy_port_names_load_through_the_aliases() {
        // The port's own pre-parity config (all twelve old spellings) must keep
        // every value it stored; the alias arms exist for exactly this file.
        let s = Settings::parse(
            "{\n  \"PickupText\": false,\n  \"MusicVolume\": 0.3,\n  \
             \"SoundVolume\": 0.4,\n  \"AmbientVolume\": 0.5,\n  \"Quality\": 3,\n \
             \"BloodAndGore\": false,\n  \"MinersWobble\": false,\n \
             \"TilesSwayInWind\": false,\n  \"GamepadInstructions\": false,\n \
             \"InvasionProgressMode\": 0,\n  \"SettingPlayWhenUnfocused\": true,\n  \
             \"HidePassword\": true\n}\n",
        );
        assert!(!s.pickup_text);
        assert!((s.music_volume - 0.3).abs() < 1e-3);
        assert!((s.sound_volume - 0.4).abs() < 1e-3);
        assert!((s.ambient_volume - 0.5).abs() < 1e-3);
        assert_eq!(s.quality, 3);
        assert!(!s.blood);
        assert!(!s.miners_wobble);
        assert!(!s.tiles_sway);
        assert!(!s.gamepad_instructions);
        assert_eq!(s.event_progress, 0);
        assert!(s.play_when_unfocused, "the old spelling still loads");
        assert!(s.hide_password);
    }

    #[test]
    fn test_DW_2_2_hide_password_reads_the_vanilla_and_port_names() {
        // The one alias the task spells out: `Main.HidePassword` is persisted as
        // `"HidePasswords"` (`Main.cs:4144`, `Get` `:4422`); the port wrote the
        // singular before the parity pass, so both spellings load.
        for spelling in ["HidePasswords", "HidePassword"] {
            let text = format!("{{\n  \"{spelling}\": true\n}}");
            assert!(Settings::parse(&text).hide_password, "{spelling}: true reads");
            let text = format!("{{\n  \"{spelling}\": false\n}}");
            assert!(!Settings::parse(&text).hide_password, "{spelling}: false reads");
        }
    }

    #[test]
    fn test_DW_2_2_both_spellings_in_one_file_last_line_wins() {
        // The parser's existing semantics — the last line for a key wins —
        // carries over to the alias pairs: whatever comes last is the value the
        // field keeps, so the ordering must be spelled out in a comment.
        let vanilla_last = "{\n  \"HidePassword\": false,\n  \"HidePasswords\": true\n}";
        assert!(Settings::parse(vanilla_last).hide_password);
        let port_last = "{\n  \"HidePasswords\": false,\n  \"HidePassword\": true\n}";
        assert!(Settings::parse(port_last).hide_password);

        let vanilla_last = "{\n  \"MusicVolume\": 0.9,\n  \"VolumeMusic\": 0.1\n}";
        assert_eq!(Settings::parse(vanilla_last).music_volume, 0.1);
        let port_last = "{\n  \"VolumeMusic\": 0.9,\n  \"MusicVolume\": 0.1\n}";
        assert_eq!(Settings::parse(port_last).music_volume, 0.1);

        let vanilla_last = "{\n  \"BloodAndGore\": true,\n  \"GoreVisualsAllowed\": false\n}";
        assert!(!Settings::parse(vanilla_last).blood);
        let port_last = "{\n  \"GoreVisualsAllowed\": false,\n  \"BloodAndGore\": true\n}";
        assert!(Settings::parse(port_last).blood);
    }

    #[test]
    /// M7-UI-5: the bind table round-trips through `config.json` under the
    /// vanilla `Key*` names, and a vanilla file's rows win over the defaults
    /// (`Main.LoadSettings`, `Main.cs:4582-4600`).
    #[test]
    fn m7_ui_5_the_binds_round_trip_through_the_config() {
        let mut s = Settings::default();
        s.keybinds[crate::input_map::Action::Jump.index()] = "F".into();
        s.keybinds[crate::input_map::Action::Smart.index()] = "Q".into();
        s.map_binds[crate::input_map::MapAction::Full.index()] = "Tab".into();
        let back = Settings::parse(&s.to_json());
        assert_eq!(back.keybinds, s.keybinds, "the 14 gameplay rows");
        assert_eq!(back.map_binds, s.map_binds, "the 6 map rows");
        // A vanilla config that only carries a couple of rows keeps the rest.
        let back = Settings::parse(
            "{
  \"KeyQuickHeal\": \"Tab\",
  \"KeyFullscreenMap\": \"F1\"
}",
        );
        assert_eq!(
            back.keybinds[crate::input_map::Action::QuickHeal.index()],
            "Tab"
        );
        assert_eq!(
            back.map_binds[crate::input_map::MapAction::Full.index()],
            "F1"
        );
        assert_eq!(
            back.keybinds[crate::input_map::Action::Mount.index()],
            "R",
            "an absent row keeps the default"
        );
        // An unreadable name keeps the default rather than going dead.
        let back = Settings::parse("{
  \"KeyJump\": \"NotAKey\"
}");
        assert_eq!(back.keybinds[crate::input_map::Action::Jump.index()], "Space");
    }

    fn test_DW_2_3_to_json_writes_the_vanilla_key_names() {
        let json = Settings::default().to_json();
        // The exact key sequence, in the writer's own order (the order the port
        // has always written; only the names changed). The brace-aware reader
        // keeps the two colour dicts as single pairs.
        assert_eq!(
            json_keys(&json),
            [
                "AutoSave",
                "AutoPause",
                "Language",
                "MapEnabled",
                "HidePasswords",
                "PlayWhenUnfocused",
                "ShowItemText",
                "InvasionBarMode",
                "PlacementPreview",
                "HighlightNewItems",
                "MouseShowBuildingGrid",
                "GamepadDisableInstructionsDisplay",
                "BackgroundParallax",
                "FrameSkipMode",
                "LightingMode",
                "GraphicsQuality",
                "BackgroundEnabled",
                "GoreVisualsAllowed",
                "SettingsEnabled_MinersWobble",
                "SettingsEnabled_TilesSwayInWind",
                "UseStormEffect",
                "UseHeatDistortion",
                "ForegroundSunlightEffects",
                "WaveQuality",
                "FlashyEffectsWorld",
                "UseScreenShake",
                "Zoom",
                "VolumeMusic",
                "VolumeSound",
                "VolumeAmbient",
                "MapScale",
                "MouseColor",
                "MouseBorderColor",
                "SmartCursorToggle",
                "UseSmartAxeAfterSmartPickaxe",
                "UseSmartCursorForCommonBlocks",
                "CraftFromNearbyChests",
                "LockOnPriority",
                "ReverseUpDownForArmorSetBonuses",
                "DisableLeftShiftTrashCan",
                "DisableQuickTrash",
                "PlayerResourcesSet",
                "PortraitPreference",
                "ShowBossBarHealthText",
                "FlashyEffectsInterface",
                "MinimapFrame",
                // `Main.cUp`..`Main.cMount` (`Main.cs:67163-67176`) and the 6
                // map binds (`:67190-67195`), written under the names
                // `Main.LoadSettings` reads back (`:4582-4600`).
                "KeyUp",
                "KeyLeft",
                "KeyDown",
                "KeyRight",
                "KeyJump",
                "KeyThrowItem",
                "KeyQuickHeal",
                "KeyQuickMana",
                "KeyQuickBuff",
                "KeyUseHook",
                "KeyAutoSelect",
                "KeyInventory",
                "KeySmartCursor",
                "KeyMount",
                "KeyMapStyle",
                "KeyFullscreenMap",
                "KeyMapZoomIn",
                "KeyMapZoomOut",
                "KeyMapAlphaUp",
                "KeyMapAlphaDown",
            ]
        );
        // Vanilla names only: the port's twelve pre-parity spellings never
        // appear in the document, not even as a substring of a value.
        for old in [
            "\"HidePassword\"",
            "\"PickupText\"",
            "\"InvasionProgressMode\"",
            "\"GamepadInstructions\"",
            "\"Quality\"",
            "\"BloodAndGore\"",
            "\"MinersWobble\"",
            "\"TilesSwayInWind\"",
            "\"MusicVolume\"",
            "\"SoundVolume\"",
            "\"AmbientVolume\"",
            "\"SettingPlayWhenUnfocused\"",
        ] {
            assert!(!json.contains(old), "{old} must not be written: {json}");
        }
        // The eleven renames as whole `"key":` pairs plus `HidePasswords`, so the
        // alias check above cannot pass on a new name containing an old one.
        for new in [
            "\"HidePasswords\":",
            "\"PlayWhenUnfocused\":",
            "\"ShowItemText\":",
            "\"InvasionBarMode\":",
            "\"GamepadDisableInstructionsDisplay\":",
            "\"GraphicsQuality\":",
            "\"GoreVisualsAllowed\":",
            "\"SettingsEnabled_MinersWobble\":",
            "\"SettingsEnabled_TilesSwayInWind\":",
            "\"VolumeMusic\":",
            "\"VolumeSound\":",
            "\"VolumeAmbient\":",
        ] {
            assert!(json.contains(new), "{new} is the vanilla spelling: {json}");
        }
        // The values sit on the new names, so a vanilla `config.json` reads back
        // what the port wrote (the round-trip is the point of the rename).
        let s = Settings::parse(vanilla_renamed_config());
        let back = Settings::parse(&s.to_json());
        assert_eq!(back.hide_password, s.hide_password);
        assert_eq!(back.pickup_text, s.pickup_text);
        assert_eq!(back.event_progress, s.event_progress);
        assert_eq!(back.gamepad_instructions, s.gamepad_instructions);
        assert_eq!(back.quality, s.quality);
        assert_eq!(back.blood, s.blood);
        assert_eq!(back.miners_wobble, s.miners_wobble);
        assert_eq!(back.tiles_sway, s.tiles_sway);
        assert_eq!(back.music_volume, s.music_volume);
        assert_eq!(back.sound_volume, s.sound_volume);
        assert_eq!(back.ambient_volume, s.ambient_volume);
    }

    #[test]
    fn renamed_keys_keep_the_port_clamps_and_coercions() {
        // Boundary/dirty data on the renamed arms — the semantics are the
        // port's own (documented, not a vanilla `Get`), and they must survive
        // the rename unchanged for both spellings.
        for key in ["GraphicsQuality", "Quality"] {
            assert_eq!(Settings::parse(&format!("{{\n \"{key}\": 9\n}}")).quality, 3);
            assert_eq!(Settings::parse(&format!("{{\n \"{key}\": 255\n}}")).quality, 3);
            assert_eq!(
                Settings::parse(&format!("{{\n \"{key}\": \"x\"\n}}")).quality,
                0,
                "an unparseable byte keeps the default"
            );
        }
        for key in ["InvasionBarMode", "InvasionProgressMode"] {
            assert_eq!(Settings::parse(&format!("{{\n \"{key}\": 3\n}}")).event_progress, 2);
            assert_eq!(
                Settings::parse(&format!("{{\n \"{key}\": \"x\"\n}}")).event_progress,
                2,
                "the default (On) survives"
            );
        }
        for key in ["VolumeMusic", "MusicVolume"] {
            // Garbage falls back to the previous/default value, as before.
            assert_eq!(
                Settings::parse(&format!("{{\n \"{key}\": \"loud\"\n}}")).music_volume, 1.0
            );
            assert_eq!(Settings::parse(&format!("{{\n \"{key}\": 0\n}}")).music_volume, 0.0);
        }
        // The boolean arms coerce any non-"true" token to false (the port-wide,
        // pre-existing semantics this phase deliberately does not change).
        for bad in ["\"True\"", "1", "null", "TRUE!"] {
            let text = format!("{{\n \"GoreVisualsAllowed\": {bad}\n}}");
            assert!(!Settings::parse(&text).blood, "{bad} coerces to false");
        }
        // A key absent from the file keeps the default.
        assert!(Settings::parse("{}").blood);
        assert!(Settings::parse("{}").pickup_text);
        assert!(!Settings::parse("{}").hide_password, "`Main.HidePassword` default");
    }

    #[test]
    fn empty_and_blank_files_and_unknown_keys_keep_the_defaults() {
        // Dirty-input robustness of the whole parser: an empty file, a blank
        // file, a file of only braces and a file of unknown keys all yield the
        // defaults (a missing key never moves a field).
        let d = Settings::default();
        for text in ["", "\n\n", "{}", "{ }", "{\n}", "garbage", "{\n \"Nope\": 1\n}"] {
            let s = Settings::parse(text);
            assert_eq!(s.auto_save, d.auto_save, "{text:?}");
            assert_eq!(s.auto_pause, d.auto_pause, "{text:?}");
            assert_eq!(s.quality, d.quality, "{text:?}");
            assert_eq!(s.event_progress, d.event_progress, "{text:?}");
            assert_eq!(s.light_mode, d.light_mode, "{text:?}");
            assert_eq!(s.music_volume, d.music_volume, "{text:?}");
            assert_eq!(s.pickup_text, d.pickup_text, "{text:?}");
            assert_eq!(s.blood, d.blood, "{text:?}");
            assert_eq!(s.language, d.language, "{text:?}");
        }
        // Blank-ish whitespace around a real key still parses (the parser
        // trims each line).
        assert!(!Settings::parse("   \"GoreVisualsAllowed\" : false   ").blood);
    }

    /// S1-H-3: the vanilla UI-scale factor (`Main.UIScaleMax`, `Main.cs:2576-2583`
    /// — `Math.Max(1f, Math.Min(h / 600f, w / 800f))`) and the setter's clamp
    /// (`Main.cs:2563-2572`). The three probe values are the verify row's
    /// 0.8 / 1.0 / 1.2.
    #[test]
    fn ui_scale_max_and_the_applied_clamp_match_the_c_sharp() {
        // `Math.Max(1f, Math.Min(h / 600f, w / 800f))`, `Main.cs:2581`.
        assert!((ui_scale_max(800.0, 600.0) - 1.0).abs() < 1e-6, "600/600 = 800/800 = 1");
        assert!(
            (ui_scale_max(1280.0, 720.0) - 1.2).abs() < 1e-6,
            "min(1.2, 1.6) = 1.2"
        );
        assert!(
            (ui_scale_max(1920.0, 1080.0) - 1.8).abs() < 1e-6,
            "min(1.8, 2.4) = 1.8"
        );
        // The `1f` floor: a tiny screen cannot push the max below 1.
        assert!((ui_scale_max(640.0, 480.0) - 1.0).abs() < 1e-6, "the 1f floor wins");

        // `Main.UIScale = value` clamps `value > UIScaleMax` down, but never up
        // (`:2566-2571`). On the 1280x720 max of 1.2 the three probe values pass
        // through unchanged; on a 1.0-capped screen 1.2 drops to 1.0.
        let mut s = Settings::default();
        for v in [0.8f32, 1.0, 1.2] {
            s.ui_scale = v;
            assert!((s.applied_ui_scale(1280.0, 720.0) - v).abs() < 1e-6, "{v} <= 1.2");
        }
        s.ui_scale = 1.2;
        assert!((s.applied_ui_scale(800.0, 600.0) - 1.0).abs() < 1e-6, "1.2 clamps to 1.0");
        // The wanted value is what persists, unclamped (`Main.cs:4201`).
        assert!((s.ui_scale - 1.2).abs() < 1e-6);
    }
}
