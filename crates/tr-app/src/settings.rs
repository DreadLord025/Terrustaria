//! `Main.SaveSettings`: the options behind the settings screens.
//!
//! The original stores them in `config.json` (`Main.SaveSettings`, `Preferences`), so the
//! same file name and key names are used here; unknown keys are ignored and missing keys
//! keep their default, which makes a vanilla `config.json` readable.

use std::path::PathBuf;

#[derive(Clone, Debug)]
pub struct Settings {
    /// `Main.autoSave`.
    pub auto_save: bool,
    /// `Main.autoPause`.
    pub auto_pause: bool,
    /// `Main.mapEnabled`.
    pub map_enabled: bool,
    /// `Main.HidePassword`.
    pub hide_password: bool,
    /// `Main.SettingPlayWhenUnfocused`.
    pub play_when_unfocused: bool,
    /// `Main.showItemText`.
    pub pickup_text: bool,
    /// `Main.invasionProgressMode`: 0 off, 1 timed, 2 on.
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
    /// `Main.FrameSkipMode`: 0 off, 1 on, 2 subtle.
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
    /// `Main.GameZoomTarget` (`Zoom`): 1.0..=2.0, only changed in the settings UI.
    pub zoom: f32,
    /// `Main.musicVolume`.
    pub music_volume: f32,
    /// `Main.soundVolume`.
    pub sound_volume: f32,
    /// `Main.ambientVolume`.
    pub ambient_volume: f32,
    /// `Main.MapScale`: saved as "MapScale" (`Main.cs:4202`), load-clamped
    /// [0.5, 1] (`:4494`), default 1 (`:67298`). P4 draws the reference 1.0
    /// only (the whole-UI P4 plan, Uncertainty 5 ruling).
    pub map_scale: f32,
    /// `MinimapFrameManager.cs:13-18`: the active minimap frame style key
    /// (`"Default"`, `"Golden"`, ...; unknown names fall back to index 0).
    pub minimap_frame: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            auto_save: true,
            auto_pause: false,
            map_enabled: true,
            hide_password: false,
            play_when_unfocused: true,
            pickup_text: true,
            event_progress: 1,
            placement_preview: true,
            highlight_new_items: true,
            tile_grid: false,
            gamepad_instructions: true,
            parallax_scroll: 60,
            frame_skip: 0,
            light_mode: 0,
            quality: 0,
            background: true,
            blood: true,
            miners_wobble: true,
            tiles_sway: true,
            zoom: 1.0,
            music_volume: 0.75,
            sound_volume: 0.75,
            ambient_volume: 0.75,
            map_scale: 1.0,
            minimap_frame: "Default".to_string(),
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
            "{{\n  \"AutoSave\": {},\n  \"AutoPause\": {},\n  \"MapEnabled\": {},\n  \
             \"HidePassword\": {},\n  \"SettingPlayWhenUnfocused\": {},\n  \"PickupText\": {},\n  \
             \"InvasionProgressMode\": {},\n  \"PlacementPreview\": {},\n  \
             \"HighlightNewItems\": {},\n  \"MouseShowBuildingGrid\": {},\n  \
             \"GamepadInstructions\": {},\n  \"BackgroundParallax\": {},\n  \
             \"FrameSkipMode\": {},\n  \
             \"LightingMode\": {},\n  \"Quality\": {},\n  \"BackgroundEnabled\": {},\n  \
             \"BloodAndGore\": {},\n  \"MinersWobble\": {},\n  \"TilesSwayInWind\": {},\n  \
             \"Zoom\": {:.3},\n  \"MusicVolume\": {:.3},\n  \"SoundVolume\": {:.3},\n  \
             \"AmbientVolume\": {:.3},\n  \"MapScale\": {:.3},\n  \
             \"MinimapFrame\": \"{}\"\n}}\n",
            self.auto_save,
            self.auto_pause,
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
            self.zoom,
            self.music_volume,
            self.sound_volume,
            self.ambient_volume,
            self.map_scale,
            self.minimap_frame,
        )
    }
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
        assert!(s.play_when_unfocused);
        assert!(s.placement_preview);
        assert!(!s.tile_grid);
        // `Main.bgScroll` is a plain static (`Main.cs:2429`) that a fresh config
        // seeds to 60: `caveParallax = 0.88f` (`Main.cs:66809`) survives the
        // keyless `Get("Parallax", ...)`, then `(int)((1f - 0.88f) * 500f)` = 60
        // and the keyless `Get("BackgroundParallax", ...)` keeps it
        // (`Main.cs:4405-4408`).
        assert_eq!(s.parallax_scroll, 60);
        assert_eq!(s.frame_skip, 0);
        assert_eq!(s.light_mode, 0);
        assert_eq!(s.quality, 0);
        assert!(s.background);
        assert!(s.blood);
        assert_eq!(s.event_progress, 1);
        assert!((s.music_volume - 0.75).abs() < 1e-6);
        assert_eq!(s.map_scale, 1.0);
        assert_eq!(s.minimap_frame, "Default");
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
}
