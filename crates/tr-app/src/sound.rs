//! `LegacySoundPlayer.PlaySound` attenuation, the 3D-audio half of M6-3DAUDIO.
//!
//! Everything here is pure maths so it can be unit-tested without an audio device;
//! `audio.rs` turns the result into a rodio source. The port follows
//! `decompiled/Terraria/Audio/LegacySoundPlayer.cs` line by line — the C# line
//! numbers are quoted on every rule that is not self-evident.

/// `LegacySoundPlayer.SoundAttenuationDistance`, set in the static constructor
/// (`LegacySoundPlayer.cs:1466`).
pub const SOUND_ATTENUATION_DISTANCE: f32 = 2500.0;

/// `Main.MaxWorldViewSize.X` (`Main.cs:677`); the pan denominator is half of it
/// (`LegacySoundPlayer.cs:395`).
pub const MAX_WORLD_VIEW_WIDTH: f32 = 1920.0;

/// `LegacySoundPlayer.DoesSoundScaleWithAmbientVolume` (`LegacySoundPlayer.cs:1440`).
///
/// These sounds are ambience (wind, rain, critters, thunder, …) and follow
/// `Main.ambientVolume` instead of `Main.soundVolume`.
pub fn does_sound_scale_with_ambient_volume(sound_type: u32) -> bool {
    matches!(
        sound_type,
        30 | 31 | 32 | 33 | 34 | 35 | 39 | 43 | 44 | 45 | 46 | 67 | 68 | 69
    )
}

/// The audible result of `LegacySoundPlayer.PlaySound`: `volume` is the final gain
/// (already scaled by the volume slider and `volumeScale`) and `pan` is the XNA
/// `-1` (left) .. `1` (right) stereo placement.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Playback {
    pub volume: f32,
    pub pan: f32,
}

/// Parameters the caller supplies for one sound; mirrors the `Main` state the C#
/// reads while attenuating.
#[derive(Clone, Copy, Debug)]
pub struct Listener {
    /// `Main.Camera.Center` in world pixels.
    pub camera_center: (f32, f32),
    /// `Main.soundVolume` (0..1).
    pub sound_volume: f32,
    /// `Main.ambientVolume` (0..1).
    pub ambient_volume: f32,
    /// `FocusHelper.QuietAmbientSounds`.
    pub quiet_ambient_sounds: bool,
    /// `Main.gameMenu`.
    pub game_menu: bool,
    /// `WorldGen.isGeneratingOrLoadingWorld` — the `LegacySoundPlayer`
    /// position-walk's first guard (`LegacySoundPlayer.cs:382`): a sound at
    /// world coordinates during generation/loading returns `null`. The port
    /// is always `false` outside the worldgen worker, which never plays.
    pub generating_or_loading: bool,
    /// `Main.netMode == 2` — the server's second guard
    /// (`LegacySoundPlayer.cs:386`).
    pub server: bool,
}

/// `LegacySoundPlayer.PlaySound(int type, int x, int y, int Style, float volumeScale)`
/// (`LegacySoundPlayer.cs:360`), reduced to what decides whether and how loud a cue
/// plays. The per-type instance bookkeeping, pitch randomisation and the remaining
/// `num2 *=` tweaks live at the call site.
///
/// `x`/`y` are world pixels; the sentinel `-1` in either matches the C#
/// `x == -1 || y == -1` early branch (no attenuation, pan 0).
///
/// Returns `None` when the original would `return null` (silent/ineligible),
/// `Some` with the final volume and pan otherwise.
#[allow(clippy::too_many_arguments)]
pub fn play_sound(
    sound_type: u32,
    x: f32,
    y: f32,
    listener: Listener,
    volume_scale: f32,
) -> Option<Playback> {
    // `if (Main.soundVolume == 0f && (type < 30 || type > 35)) return null;`
    if listener.sound_volume == 0.0 && !(30..=35).contains(&sound_type) {
        return None;
    }
    // `if (WorldGen.isGeneratingOrLoadingWorld) return null;` +
    // `if (Main.netMode == 2) return null;` (`LegacySoundPlayer.cs:382-388`):
    // the position-walk's worldgen/server guards — S1-E-24. The unpositioned
    // `x == -1` path in the C# checks them AFTER the `flag = true` arm, so a
    // generation-time unpositioned call would still pass; the port's rig
    // never reaches a worldgen play, so the guards gate the positioned walk
    // verbatim.
    if (x != -1.0 || y != -1.0) && (listener.generating_or_loading || listener.server) {
        return None;
    }

    let mut audible = false; // flag
    let mut volume = 1.0f32; // num2
    let mut pan = 0.0f32; // num3
    if x == -1.0 || y == -1.0 {
        audible = true;
    } else {
        let dx = x - listener.camera_center.0;
        let dy = y - listener.camera_center.1;
        let distance = (dx * dx + dy * dy).sqrt();
        if distance < SOUND_ATTENUATION_DISTANCE {
            audible = true;
            pan = (dx / (MAX_WORLD_VIEW_WIDTH * 0.5)).clamp(-1.0, 1.0);
            volume = 1.0 - distance / SOUND_ATTENUATION_DISTANCE;
        }
    }
    pan = pan.clamp(-1.0, 1.0);
    if volume > 1.0 {
        volume = 1.0;
    }
    // `if (num2 <= 0f && (type < 34 || type > 35 || type > 39)) return null;`
    // (`type > 39` is subsumed by `type > 35`.)
    if volume <= 0.0 && !(34..=35).contains(&sound_type) {
        return None;
    }
    if !audible {
        // Not audible at all: the C# skips the whole `if (flag)` block and returns
        // `soundEffectInstance` which is still null.
        return None;
    }

    if does_sound_scale_with_ambient_volume(sound_type) {
        volume *= listener.ambient_volume * if listener.quiet_ambient_sounds { 0.0 } else { 1.0 };
        if listener.game_menu {
            volume = 0.0;
        }
    } else {
        volume *= listener.sound_volume;
    }
    if volume > 1.0 {
        volume = 1.0;
    }
    // `if (num2 <= 0f && (type < 30 || type > 35) && type != 39) return null;`
    if volume <= 0.0 && !(30..=35).contains(&sound_type) && sound_type != 39 {
        return None;
    }

    // `soundEffectInstance.Volume *= volumeScale` (`LegacySoundPlayer.cs:1411`).
    volume *= volume_scale;
    if volume > 1.0 {
        volume = 1.0;
    }
    Some(Playback { volume, pan })
}

/// `LegacySoundStyle.WithVolumePerceived` (`LegacySoundStyle.cs:70-75`): the
/// style's stored volume becomes `volume ^ 1.6608999967575073` — the
/// "perceived loudness" power the C# bakes into the three quiet styles
/// (`SoundID.cs:1497`'s `Item113_Quiet` = `2/3`, `:1584`'s `MysticBloom` =
/// `0.65`, `:1587`'s `ArcSurge` = `0.4`; S1-E-24).
pub fn perceived_volume(volume: f32) -> f32 {
    volume.powf(1.660_899_996_757_507_3)
}

/// The three `WithVolumePerceived` styles' baked volumes
/// (`SoundID.cs:1497`/`:1584`/`:1587`) — the display table the styles read.
pub fn perceived_volume_styles() -> Vec<(&'static str, f32, f32)> {
    vec![
        // (const name, the perceived argument, the baked style volume)
        ("Item113_Quiet", 2.0 / 3.0, perceived_volume(2.0 / 3.0)),
        ("MysticBloom", 0.65, perceived_volume(0.65)),
        ("ArcSurge", 0.4, perceived_volume(0.4)),
    ]
}

/// Left/right amplitude for an XNA `Pan` in `-1..1`.
///
/// XNA applies `Pan` inside XAudio2, which the decompilation does not contain, so
/// this is the standard linear pan law used by the reference implementation: the
/// far channel is attenuated, the near channel is untouched.
pub fn pan_gains(pan: f32) -> (f32, f32) {
    let pan = pan.clamp(-1.0, 1.0);
    if pan <= 0.0 {
        (1.0, 1.0 + pan)
    } else {
        (1.0 - pan, 1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn listener() -> Listener {
        Listener {
            camera_center: (0.0, 0.0),
            sound_volume: 1.0,
            ambient_volume: 1.0,
            quiet_ambient_sounds: false,
            game_menu: false,
            generating_or_loading: false,
            server: false,
        }
    }

    #[test]
    fn ambient_volume_set_matches_the_switch() {
        for t in [30, 31, 32, 33, 34, 35, 39, 43, 44, 45, 46, 67, 68, 69] {
            assert!(does_sound_scale_with_ambient_volume(t), "{t}");
        }
        for t in [0, 1, 2, 3, 16, 18, 29, 36, 38, 40, 66, 70] {
            assert!(!does_sound_scale_with_ambient_volume(t), "{t}");
        }
    }

    #[test]
    fn silent_sound_volume_drops_non_ambient_sounds() {
        let mut l = listener();
        l.sound_volume = 0.0;
        // Type 0 (Dig) is not in the 30..=35 exception, so it is dropped outright.
        assert_eq!(play_sound(0, -1.0, -1.0, l, 1.0), None);
        // Type 34 (Waterfall) is exempt from the early-out; it follows the ambient
        // slider (still 1.0 here) rather than the muted `soundVolume`.
        assert_eq!(
            play_sound(34, -1.0, -1.0, l, 1.0),
            Some(Playback { volume: 1.0, pan: 0.0 })
        );
    }

    #[test]
    fn volume_falls_off_linearly_over_the_attenuation_distance() {
        let l = listener();
        // 1250 px is half of 2500, so the gain is exactly 1 - 0.5 = 0.5.
        let half = play_sound(3, 1250.0, 0.0, l, 1.0).unwrap();
        assert!((half.volume - 0.5).abs() < 1e-6);
        // Just inside the radius stays audible; at/after it is dropped.
        assert!(play_sound(3, 2499.0, 0.0, l, 1.0).is_some());
        assert_eq!(play_sound(3, 2500.0, 0.0, l, 1.0), None);
        assert_eq!(play_sound(3, 5000.0, 0.0, l, 1.0), None);
    }

    #[test]
    fn pan_is_clamped_to_the_view_half_width() {
        let l = listener();
        // dx / 960, clamped.
        assert_eq!(play_sound(3, 960.0, 0.0, l, 1.0).unwrap().pan, 1.0);
        assert_eq!(play_sound(3, -960.0, 0.0, l, 1.0).unwrap().pan, -1.0);
        assert_eq!(play_sound(3, 480.0, 0.0, l, 1.0).unwrap().pan, 0.5);
        // Beyond the half-width is clamped, and volume keeps falling with distance.
        assert_eq!(play_sound(3, 2000.0, 0.0, l, 1.0).unwrap().pan, 1.0);
    }

    #[test]
    fn negative_coordinates_are_two_dimensional() {
        let mut l = listener();
        l.sound_volume = 0.5;
        // x == -1 -> flag = true, full 1.0 * soundVolume.
        let p = play_sound(3, -1.0, -1.0, l, 1.0).unwrap();
        assert_eq!(p, Playback { volume: 0.5, pan: 0.0 });
    }

    #[test]
    fn ambient_sounds_use_the_ambient_slider() {
        let mut l = listener();
        l.sound_volume = 0.5;
        l.ambient_volume = 0.25;
        // Type 45 (Dolphin) scales with ambient, not sound.
        let p = play_sound(45, -1.0, -1.0, l, 1.0).unwrap();
        assert!((p.volume - 0.25).abs() < 1e-6);
    }

    #[test]
    fn ambient_sounds_are_silent_in_the_menu() {
        let mut l = listener();
        l.game_menu = true;
        // Ambient sounds are forced to 0 in the menu...
        assert_eq!(play_sound(45, -1.0, -1.0, l, 1.0), None);
        // ... but a non-ambient sound still plays.
        assert!(play_sound(3, -1.0, -1.0, l, 1.0).is_some());
    }

    #[test]
    fn volume_scale_is_applied_last_and_clamped() {
        let l = listener();
        let p = play_sound(3, -1.0, -1.0, l, 0.5).unwrap();
        assert!((p.volume - 0.5).abs() < 1e-6);
        let p = play_sound(3, -1.0, -1.0, l, 4.0).unwrap();
        assert_eq!(p.volume, 1.0);
    }

    #[test]
    fn pan_gains_attenuate_the_far_channel() {
        assert_eq!(pan_gains(0.0), (1.0, 1.0));
        assert_eq!(pan_gains(-1.0), (1.0, 0.0));
        assert_eq!(pan_gains(1.0), (0.0, 1.0));
        assert_eq!(pan_gains(0.5), (0.5, 1.0));
        assert_eq!(pan_gains(-0.25), (1.0, 0.75));
        // Clamped beyond the legal range.
        assert_eq!(pan_gains(9.0), (0.0, 1.0));
    }
#[test]
fn s1_e_24_the_worldgen_and_server_guards_mute_the_positioned_walk() {
    // `LegacySoundPlayer.cs:382-388`: the position-walk returns null during
    // generation/loading and on the server (`netMode == 2`).
    let mut l = listener();
    l.generating_or_loading = true;
    assert_eq!(
        play_sound(12, 100.0, 100.0, l.clone(), 1.0),
        None,
        "the generation guard"
    );
    l.generating_or_loading = false;
    l.server = true;
    assert_eq!(
        play_sound(12, 100.0, 100.0, l.clone(), 1.0),
        None,
        "the server guard"
    );
    // The unpositioned arm (`x == -1`) passes the guards in the C#'s order —
    // it sets `flag` before reaching them.
    l.generating_or_loading = true;
    l.server = true;
    assert!(
        play_sound(12, -1.0, -1.0, l.clone(), 1.0).is_some(),
        "the unpositioned arm precedes the guards"
    );
}

#[test]
fn s1_e_24_the_perceived_volume_is_the_16609_power() {
    // `Math.Pow(volume, 1.6608999967575073)` (`LegacySoundStyle.cs:75`).
    assert!((perceived_volume(2.0 / 3.0) - (2.0f32 / 3.0).powf(1.660_899_996_757_507_3)).abs() < 1e-7);
    assert!((perceived_volume(0.65) - 0.65f32.powf(1.660_899_996_757_507_3)).abs() < 1e-7);
    assert!((perceived_volume(0.4) - 0.4f32.powf(1.660_899_996_757_507_3)).abs() < 1e-7);
    // The styles table carries the three quiet styles with their baked volumes.
    for (name, arg, baked) in perceived_volume_styles() {
        assert!((baked - perceived_volume(arg)).abs() < 1e-7, "{name}");
    }
    // The perceived power compresses: for every 0..1 argument the baked
    // volume sits below it (`0.5^1.66 = 0.316` — the exponent is above 1).
    assert!(perceived_volume(0.5) < 0.5);
}
}
