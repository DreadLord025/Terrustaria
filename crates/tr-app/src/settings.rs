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
    /// `Main.HidePassword`.
    pub hide_password: bool,
    /// `Main.SettingPlayWhenUnfocused`: default **false** — `static Main()`
    /// (`Main.cs:66512`). The menuMode 11 row inverts it into the
    /// "Pause When Unfocused" label ("Off" when the flag is true).
    pub play_when_unfocused: bool,
    /// `Main.showItemText`.
    pub pickup_text: bool,
    /// `Main.invasionProgressMode`: 0 off, 1 timed, 2 on; default **2** (On) —
    /// `static Main()` (`Main.cs:67080`).
    pub event_progress: u8,
    /// `Main.placementPreview`.
    pub placement_preview: bool,
    /// `ItemSlot.Options.HighlightNewItems`.
    pub highlight_new_items: bool,
    /// `Main.MouseShowBuildingGrid`.
    pub tile_grid: bool,
    /// `Main.GamepadDisableInstructionsDisplay`.
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
    /// `Lighting.Mode`: 0 Color, 1 White, 2 Retro, 3 Trippy.
    pub light_mode: u8,
    /// `Main.qaStyle`: 0 auto, 1 high, 2 medium, 3 low.
    pub quality: u8,
    /// `Main.BackgroundEnabled`.
    pub background: bool,
    /// `ChildSafety.Disabled` (blood and gore).
    pub blood: bool,
    /// `Main.SettingsEnabled_MinersWobble`.
    pub miners_wobble: bool,
    /// `Main.SettingsEnabled_TilesSwayInWind`.
    pub tiles_sway: bool,
    /// `Player.Settings.CraftFromNearbyChests` (`Player.cs:360`), defaulted
    /// true (`Player.cs:406`): the crafting pulls ingredients from the world
    /// chests in range.
    pub craft_from_nearby_chests: bool,
    /// `Main.GameZoomTarget` (`Zoom`): 1.0..=2.0, only changed in the settings UI.
    pub zoom: f32,
    /// `Main.musicVolume`: default **1f** — `static Main()` (`Main.cs:66857`).
    pub music_volume: f32,
    /// `Main.soundVolume`: default **1f** — `static Main()` (`Main.cs:66859`).
    pub sound_volume: f32,
    /// `Main.ambientVolume`: default **1f** — `static Main()` (`Main.cs:66858`).
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
                "HidePassword" => me.hide_password = boolean(value),
                "SettingPlayWhenUnfocused" => me.play_when_unfocused = boolean(value),
                "PickupText" => me.pickup_text = boolean(value),
                "InvasionProgressMode" => {
                    me.event_progress = value.parse().unwrap_or(me.event_progress).min(2)
                }
                "PlacementPreview" => me.placement_preview = boolean(value),
                "HighlightNewItems" => me.highlight_new_items = boolean(value),
                "MouseShowBuildingGrid" => me.tile_grid = boolean(value),
                "GamepadInstructions" => me.gamepad_instructions = boolean(value),
                // `configuration.Get("BackgroundParallax", ref Main.bgScroll)`
                // (`Main.cs:4407`); the slider only writes 0..=100 (`:51642`).
                "BackgroundParallax" => {
                    me.parallax_scroll =
                        value.parse().unwrap_or(me.parallax_scroll).min(100)
                }
                "FrameSkipMode" => me.frame_skip = value.parse().unwrap_or(me.frame_skip).min(2),
                "LightingMode" => me.light_mode = value.parse().unwrap_or(me.light_mode).min(3),
                "Quality" => me.quality = value.parse().unwrap_or(me.quality).min(3),
                "BackgroundEnabled" => me.background = boolean(value),
                "BloodAndGore" => me.blood = boolean(value),
                "MinersWobble" => me.miners_wobble = boolean(value),
                "TilesSwayInWind" => me.tiles_sway = boolean(value),
                "Zoom" => {
                    me.zoom = value.parse::<f32>().unwrap_or(me.zoom).clamp(1.0, 2.0)
                }
                "MusicVolume" => me.music_volume = value.parse().unwrap_or(me.music_volume),
                "SoundVolume" => me.sound_volume = value.parse().unwrap_or(me.sound_volume),
                "AmbientVolume" => me.ambient_volume = value.parse().unwrap_or(me.ambient_volume),
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
                _ => {}
            }
        }
        me
    }

    /// `Main.SaveSettings`: written to `config.json` next to the worlds.
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
             \"HidePassword\": {},\n  \"SettingPlayWhenUnfocused\": {},\n  \"PickupText\": {},\n  \
             \"InvasionProgressMode\": {},\n  \"PlacementPreview\": {},\n  \
             \"HighlightNewItems\": {},\n  \"MouseShowBuildingGrid\": {},\n  \
             \"GamepadInstructions\": {},\n  \"BackgroundParallax\": {},\n  \
             \"FrameSkipMode\": {},\n  \
             \"LightingMode\": {},\n  \"Quality\": {},\n  \"BackgroundEnabled\": {},\n  \
             \"BloodAndGore\": {},\n  \"MinersWobble\": {},\n  \"TilesSwayInWind\": {},\n  \
             \"UseStormEffect\": {},\n  \"UseHeatDistortion\": {},\n  \
             \"ForegroundSunlightEffects\": {},\n  \"WaveQuality\": {},\n  \
             \"FlashyEffectsWorld\": {},\n  \"UseScreenShake\": {},\n  \
             \"Zoom\": {:.3},\n  \"MusicVolume\": {:.3},\n  \"SoundVolume\": {:.3},\n  \
             \"AmbientVolume\": {:.3},\n  \"MapScale\": {:.3},\n  \
             \"MouseColor\": {{\"R\": {}, \"G\": {}, \"B\": {}}},\n  \
             \"MouseBorderColor\": {{\"R\": {}, \"G\": {}, \"B\": {}, \"A\": {}}},\n  \
             \"SmartCursorToggle\": {},\n  \"UseSmartAxeAfterSmartPickaxe\": {},\n  \
             \"UseSmartCursorForCommonBlocks\": {},\n  \"LockOnPriority\": {},\n  \
             \"ReverseUpDownForArmorSetBonuses\": {},\n  \
             \"DisableLeftShiftTrashCan\": {},\n  \"DisableQuickTrash\": {},\n  \
             \"PlayerResourcesSet\": \"{}\",\n  \"PortraitPreference\": {},\n  \
             \"ShowBossBarHealthText\": {},\n  \"FlashyEffectsInterface\": {},\n  \
             \"MinimapFrame\": \"{}\"\n}}\n",
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
            self.light_mode,
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
        )
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
        s.minimap_frame = "Golden".to_string();
        s.mouse_color = [1, 2, 3, 255];
        s.mouse_border_color = [4, 5, 6, 7];
        s.smart_cursor_toggle = false;
        s.smart_axe_first = true;
        s.smart_blocks = true;
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
        assert_eq!(back.minimap_frame, s.minimap_frame);
        assert_eq!(back.mouse_color, [1, 2, 3, 255]);
        assert_eq!(back.mouse_border_color, [4, 5, 6, 7]);
        assert!(!back.smart_cursor_toggle);
        assert!(back.smart_axe_first);
        assert!(back.smart_blocks);
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
        assert_eq!(s.light_mode, 3, "clamped to the four modes");
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
}
