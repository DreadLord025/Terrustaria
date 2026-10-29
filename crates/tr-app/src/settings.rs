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
    /// `Main.caveParallax` (the "Parallax" slider, on/off in this port).
    pub parallax: bool,
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
    /// `Main.musicVolume`.
    pub music_volume: f32,
    /// `Main.soundVolume`.
    pub sound_volume: f32,
    /// `Main.ambientVolume`.
    pub ambient_volume: f32,
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
            parallax: true,
            frame_skip: 0,
            light_mode: 0,
            quality: 0,
            background: true,
            blood: true,
            miners_wobble: true,
            tiles_sway: true,
            music_volume: 0.75,
            sound_volume: 0.75,
            ambient_volume: 0.75,
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
                "CaveParallax" => me.parallax = boolean(value),
                "FrameSkipMode" => me.frame_skip = value.parse().unwrap_or(me.frame_skip).min(2),
                "LightingMode" => me.light_mode = value.parse().unwrap_or(me.light_mode).min(3),
                "Quality" => me.quality = value.parse().unwrap_or(me.quality).min(3),
                "BackgroundEnabled" => me.background = boolean(value),
                "BloodAndGore" => me.blood = boolean(value),
                "MinersWobble" => me.miners_wobble = boolean(value),
                "TilesSwayInWind" => me.tiles_sway = boolean(value),
                "MusicVolume" => me.music_volume = value.parse().unwrap_or(me.music_volume),
                "SoundVolume" => me.sound_volume = value.parse().unwrap_or(me.sound_volume),
                "AmbientVolume" => me.ambient_volume = value.parse().unwrap_or(me.ambient_volume),
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
             \"GamepadInstructions\": {},\n  \"CaveParallax\": {},\n  \"FrameSkipMode\": {},\n  \
             \"LightingMode\": {},\n  \"Quality\": {},\n  \"BackgroundEnabled\": {},\n  \
             \"BloodAndGore\": {},\n  \"MinersWobble\": {},\n  \"TilesSwayInWind\": {},\n  \
             \"MusicVolume\": {:.3},\n  \"SoundVolume\": {:.3},\n  \"AmbientVolume\": {:.3}\n}}\n",
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
            self.parallax,
            self.frame_skip,
            self.light_mode,
            self.quality,
            self.background,
            self.blood,
            self.miners_wobble,
            self.tiles_sway,
            self.music_volume,
            self.sound_volume,
            self.ambient_volume,
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
        assert!(s.parallax);
        assert_eq!(s.frame_skip, 0);
        assert_eq!(s.light_mode, 0);
        assert_eq!(s.quality, 0);
        assert!(s.background);
        assert!(s.blood);
        assert_eq!(s.event_progress, 1);
        assert!((s.music_volume - 0.75).abs() < 1e-6);
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
    }

    #[test]
    fn unknown_keys_and_garbage_are_ignored() {
        let s = Settings::parse("{\n \"Nope\": true,\n \"AutoSave\": false,\n \"LightingMode\": 9\n}");
        assert!(!s.auto_save);
        assert_eq!(s.light_mode, 3, "clamped to the four modes");
    }
}
