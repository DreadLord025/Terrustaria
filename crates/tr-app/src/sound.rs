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
    /// `SoundEffectInstance.Pitch` — the per-family roll's result
    /// (`LegacySoundPlayer.cs:450`/`:613-626` + `SoundStyle.GetRandomPitch`).
    pub pitch: f32,
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
    Some(Playback {
        volume,
        pan,
        pitch: 0.0,
    })
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

/// `SoundStyle.GetRandomPitch` (`SoundStyle.cs:40-43`):
/// `SoundStyle._random.NextFloat() * PitchVariance - PitchVariance * 0.5f`.
/// `next_float` mirrors `NextFloat()`'s 0..1 draw.
pub fn get_random_pitch(variance: f32, next_float: f32) -> f32 {
    next_float * variance - variance * 0.5
}

/// `LegacySoundPlayer.PlaySound`'s per-family pitch rolls (`:450`, `:613-626`
/// — the dig arm's `Next(-10, 11) * 0.01` and the item switch's per-cue
/// scales; S1-E-22). `rng` mirrors `Main.rand`'s `Next(min, max)` draw.
pub fn legacy_family_pitch(family: Family, rng: &mut dyn FnMut(i32, i32) -> i32) -> f32 {
    match family {
        Family::Dig => rng(-10, 11) as f32 * 0.01,
        // Case 17 (`:862`): the run-step roll — the same window as the dig
        // arm, drawn from the same `Main.rand` (S1-E-21 wave D).
        Family::Run => rng(-10, 11) as f32 * 0.01,
        // Case 6 (`:714`): the grass roll — the wider ±0.30 window
        // (S1-E-21i, the KillWall grass arm).
        Family::Grass => rng(-30, 31) as f32 * 0.01,
        // Cases 8/9 (`:736`/`:747`): the door open/close rolls share the
        // ±0.20 window (S1-E-21i, WorldGen.OpenDoor/CloseDoor).
        Family::Door => rng(-20, 21) as f32 * 0.01,
        Family::Item(cue) => match cue {
            53 => rng(-20, -11) as f32 * 0.02,
            55 => -(rng(-20, -11) as f32) * 0.02,
            132 => rng(-20, 21) as f32 * 0.001,
            153 | 156 => rng(-50, 51) as f32 * 0.003,
            _ => rng(-6, 7) as f32 * 0.01,
        },
    }
}

/// The pitch-roll families the legacy player switches over
/// (`LegacySoundPlayer.cs:440-462` the dig case, `:611-630` the item switch).
#[derive(Clone, Copy, Debug)]
pub enum Family {
    /// `type == 0` — the dig/sound-style case (`:440`).
    Dig,
    /// `type == 2` — the item family, keyed by the style id (`:611`).
    Item(i32),
    /// `type == 17` — the run/footstep case (`:854-864`, the `:862` roll).
    Run,
    /// `type == 6` — the grass case (`:706-716`, the `:714` roll).
    Grass,
    /// `type == 8`/`9` — the door open/close cases (`:728-749`, the
    /// `:736`/`:747` rolls — one shared window).
    Door,
}

/// One `LegacySoundPlayer.PlaySound` case-28 (Mech) play's pitch decision
/// (`LegacySoundPlayer.cs:997-1007`, S1-E-21i — the `Wiring.HitSwitch`
/// clicks). `playing_0` is the `SoundInstanceMech[0].State == Playing` read
/// (`:998-1001`): while the click is live a new play returns `null` BEFORE
/// the pitch draw — `None` models that refusal. Otherwise the ONE
/// `Main.rand.Next(-10, 11) * 0.01` draw (`:1005`) rides the caller's
/// stream — the stage swap wrapping the `HitSwitch` call (`Main.cs:17754`
/// the player, `:17763` the weighted plates, `:17767` the NPCs, `:17802`
/// the wiring tick), never the wiring stand-in stream. The caller must
/// gate on audibility BEFORE calling (the C# reaches the switch only past
/// `:390-436`).
pub fn mech_case28(playing_0: bool, rng: &mut dyn FnMut(i32, i32) -> i32) -> Option<f32> {
    // `:998-1001` — the playing guard returns before any draw.
    if playing_0 {
        return None;
    }
    // `:1005` — one `Main.rand.Next(-10, 11) * 0.01` pitch draw.
    Some(rng(-10, 11) as f32 * 0.01)
}

/// One `LegacySoundPlayer.PlaySound` case-2 (item) play's decisions
/// (`LegacySoundPlayer.cs:516-644`, S1-E-21 wave A): the final style number
/// after the style-1 variation remap, the volume tweak the case applies to
/// the attenuated gain, and the instance pitch BEFORE the caller's
/// `pitchOffset` (the final `Clamp(pitch + pitchOffset, -1, 1)` is `:1410`).
///
/// `rng` mirrors `Main.rand` — the item-use wave passes the `UpdatePlayers`
/// stream (`Player.ItemCheck` runs under the `Main.cs:17754` swap). The C#
/// reaches this switch only after the attenuation walk passed (`:390-436`),
/// so the caller must gate on audibility BEFORE drawing.
///
/// Not modelled (the port's standing D4 deviation): the per-style instance
/// stop/re-create (`:580-606`) and the 53/55/159 playing guards — the port
/// spawns a fresh source per play.
pub fn item_use_case2(style: i32, rng: &mut dyn FnMut(i32, i32) -> i32) -> (i32, f32, f32) {
    let mut num = style;
    let mut tweak = 1.0f32; // the num2 multipliers
    // `:517-535` — the per-style volume ifs.
    match num {
        176 => tweak *= 0.9,
        129 => tweak *= 0.6,
        123 => tweak *= 0.5,
        124 | 125 => tweak *= 0.65,
        116 => tweak *= 0.5,
        _ => {}
    }
    // `:537-578` — the first switch: the style-1 variation remap and the
    // 53/55/37/52/157/158/210 volume arms.
    match num {
        1 => {
            // `int num11 = Main.rand.Next(3);` (`:541`) — 1 → 18, 2 → 19.
            let roll = rng(0, 3);
            if roll == 1 {
                num = 18;
            }
            if roll == 2 {
                num = 19;
            }
        }
        53 | 55 => {
            tweak *= 0.75;
            if num == 55 {
                tweak *= 0.75;
            }
        }
        37 => tweak *= 0.5,
        52 => tweak *= 0.35,
        157 => tweak *= 0.7,
        158 => tweak *= 0.8,
        210 => tweak *= 0.35,
        _ => {}
    }
    // `:610-634` — the pitch switch (one `Main.rand` draw per play, except
    // 192's `Projectile.kiteSoundPitch` read — no item-use style is 192).
    let mut pitch = match num {
        53 => rng(-20, -11) as f32 * 0.02,
        55 => -(rng(-20, -11) as f32) * 0.02,
        132 => rng(-20, 21) as f32 * 0.001,
        153 => rng(-50, 51) as f32 * 0.003,
        156 => {
            tweak *= 0.6; // `:626` — Volume *= 0.6 rides the pitch arm
            rng(-50, 51) as f32 * 0.002
        }
        _ => rng(-6, 7) as f32 * 0.01,
    };
    // `:635-639` — the music-box styles play at the music pitch; `:640-643`
    // — style 169 drops 0.8. (No item-use row names 26/35/47/169 today; the
    // arms are ported so the table cannot drift silently.)
    if num == 26 || num == 35 || num == 47 {
        tweak *= 0.75;
        pitch = 0.0; // Main.musicPitch — the port's music pitch stand-in
    }
    if num == 169 {
        pitch -= 0.8;
    }
    (num, tweak, pitch)
}

/// One `LegacySoundPlayer.PlaySound` case-3 (NPC hit) play's decisions
/// (`LegacySoundPlayer.cs:646-672`, S1-E-21 wave C): the volume tweak the
/// case applies to the attenuated gain and the instance pitch BEFORE the
/// caller's `pitchOffset` (the final `Clamp(pitch + pitchOffset, -1, 1)` is
/// `:1410`). `rng` mirrors `Main.rand` — the strike sites pass the stage's
/// named stream (`Player.ItemCheck`/`Projectile.Damage` run under the
/// `Main.cs:17754`/`:17775` swaps). The C# reaches this switch only after
/// the attenuation walk passed (`:390-436`), so the caller must gate on
/// audibility BEFORE drawing.
///
/// `playing_57` is the `SoundInstanceNpcHit[57].State == Playing` read
/// (`:651-654`): while the Moon Lord's hit voice is live, a new 57 play
/// returns `null` BEFORE the pitch draw. `None` models that refusal. The
/// stop/re-create pool itself (`:663-666`) is the standing D4 deviation.
pub fn npc_hit_case3(
    style: i32,
    playing_57: bool,
    rng: &mut dyn FnMut(i32, i32) -> i32,
) -> Option<(f32, f32)> {
    let mut tweak = 1.0f32;
    // `:647-650` — styles 20..54 play at half volume.
    if (20..=54).contains(&style) {
        tweak *= 0.5;
    }
    // `:651-654` — the 57 playing guard returns before any draw.
    if style == 57 && playing_57 {
        return None;
    }
    // `:655-662` — 57 plays at ×0.6, 55/56 at ×0.5.
    if style == 57 {
        tweak *= 0.6;
    }
    if style == 55 || style == 56 {
        tweak *= 0.5;
    }
    // `:670` — one `Main.rand.Next(-10, 11) * 0.01` pitch draw.
    let pitch = rng(-10, 11) as f32 * 0.01;
    Some((tweak, pitch))
}

/// One `LegacySoundPlayer.PlaySound` case-4 (NPC killed) play's decisions
/// (`LegacySoundPlayer.cs:673-695`, S1-E-21 wave C) — the death-side peer
/// of [`npc_hit_case3`]. `playing_10` is the
/// `SoundInstanceNpcKilled[10].State == Playing` read (`:686-689`): while
/// the Wall of Flesh's death voice is live a new 10 play returns `null`
/// BEFORE the pitch draw. Unlike case 3 the pool entry is overwritten
/// without a Stop (`:690`), the standing D4 deviation either way.
pub fn npc_killed_case4(
    style: i32,
    playing_10: bool,
    rng: &mut dyn FnMut(i32, i32) -> i32,
) -> Option<(f32, f32)> {
    let mut tweak = 1.0f32;
    // `:674-677` — styles 23..57 play at half volume.
    if (23..=57).contains(&style) {
        tweak *= 0.5;
    }
    // `:678-685` — 61 and 62 play at ×0.6.
    if style == 61 || style == 62 {
        tweak *= 0.6;
    }
    // `:686-689` — the 10 playing guard returns before any draw.
    if style == 10 && playing_10 {
        return None;
    }
    // `:693` — one `Main.rand.Next(-10, 11) * 0.01` pitch draw.
    let pitch = rng(-10, 11) as f32 * 0.01;
    Some((tweak, pitch))
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
            Some(Playback {
            volume: 1.0,
            pan: 0.0,
            pitch: 0.0,
        })
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
        assert_eq!(p, Playback {
        volume: 0.5,
        pan: 0.0,
        pitch: 0.0,
    });
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

#[test]
fn s1_e_22_the_legacy_family_pitch_rolls_follow_the_switches() {
    // The dig arm (`LegacySoundPlayer.cs:450`): `Next(-10, 11) * 0.01`.
    let mut draw = |lo: i32, hi: i32| {
        assert_eq!((lo, hi), (-10, 11), "the dig roll's window");
        5
    };
    assert!((legacy_family_pitch(Family::Dig, &mut draw) - 0.05).abs() < 1e-6, "5 * 0.01 in f32");
    // The item switch's per-cue windows (`:612-626`): 53 (down 0.02),
    // 55 (mirrored up), 132 (0.001), 153/156 (0.003), the default 0.01.
    let seq: [(i32, i32); 5] = [(-20, -11), (-20, -11), (-20, 21), (-50, 51), (-6, 7)];
    let mut i = 0;
    let mut draw2 = move |lo: i32, hi: i32| {
        assert_eq!((lo, hi), seq[i], "the item roll {}'s window", i);
        let r = seq[i].0 + 1;
        i += 1;
        r
    };
    let p53 = legacy_family_pitch(Family::Item(53), &mut draw2);
    assert!((p53 - (-19 as f32 * 0.02)).abs() < 1e-6);
    let p55 = legacy_family_pitch(Family::Item(55), &mut draw2);
    assert!((p55 - 19.0 * 0.02).abs() < 1e-6, "the mirrored 55 arm negates the draw");
    let p132 = legacy_family_pitch(Family::Item(132), &mut draw2);
    assert!((p132 - (-19 as f32 * 0.001)).abs() < 1e-6);
    let p153 = legacy_family_pitch(Family::Item(153), &mut draw2);
    assert!((p153 - (-49 as f32 * 0.003)).abs() < 1e-6);
    let pd = legacy_family_pitch(Family::Item(999), &mut draw2);
    assert!((pd - (-5 as f32 * 0.01)).abs() < 1e-6, "the default -6..7 window");
}

#[test]
fn s1_e_22_get_random_pitch_is_the_variance_walk() {
    // `SoundStyle.GetRandomPitch` (`SoundStyle.cs:40-43`):
    // `NextFloat() * PitchVariance - PitchVariance * 0.5`.
    assert_eq!(get_random_pitch(0.2, 0.0), -0.1, "the draw-0 floor");
    assert_eq!(get_random_pitch(0.2, 1.0), 0.1, "the draw-1 ceiling");
    assert_eq!(get_random_pitch(0.2, 0.5), 0.0, "the draw-0.5 centre");
}

#[test]
fn s1_e_21c_npc_hit_case3_tweaks_the_volume_and_guards_57() {
    // `LegacySoundPlayer.cs:646-672`: styles 20..54 ×0.5, 55/56 ×0.5,
    // 57 ×0.6 — one Next(-10, 11) * 0.01 pitch draw each (`:670`).
    for style in [1, 19, 20, 54, 55, 56, 58] {
        let mut roll = |lo: i32, hi: i32| {
            assert_eq!((lo, hi), (-10, 11), "style {style}'s pitch window");
            5
        };
        let (tweak, pitch) = npc_hit_case3(style, false, &mut roll).unwrap();
        let want = if (20..=54).contains(&style) || style == 55 || style == 56 { 0.5 } else { 1.0 };
        assert!((tweak - want).abs() < 1e-6, "style {style}");
        assert!((pitch - 0.05).abs() < 1e-6);
    }
    let mut roll = |lo: i32, hi: i32| {
        assert_eq!((lo, hi), (-10, 11));
        -10
    };
    let (tweak, pitch) = npc_hit_case3(57, false, &mut roll).unwrap();
    assert!((tweak - 0.6).abs() < 1e-6, "the 57 arm's ×0.6");
    assert!((pitch - (-0.1)).abs() < 1e-6);
    // The 57 playing guard (`:651-654`) returns BEFORE the pitch draw.
    let mut roll = |_: i32, _: i32| panic!("the guarded play draws nothing");
    assert_eq!(npc_hit_case3(57, true, &mut roll), None);
    // Other styles ignore the 57 slot's state.
    let mut roll = |_: i32, _: i32| 0;
    assert!(npc_hit_case3(1, true, &mut roll).is_some());
}

#[test]
fn s1_e_21c_npc_killed_case4_tweaks_the_volume_and_guards_10() {
    // `LegacySoundPlayer.cs:673-695`: styles 23..57 ×0.5, 61/62 ×0.6 —
    // one Next(-10, 11) * 0.01 pitch draw each (`:693`).
    for style in [1, 22, 23, 57, 58, 61, 62] {
        let mut roll = |lo: i32, hi: i32| {
            assert_eq!((lo, hi), (-10, 11), "style {style}'s pitch window");
            0
        };
        let (tweak, pitch) = npc_killed_case4(style, false, &mut roll).unwrap();
        let want = if (23..=57).contains(&style) {
            0.5
        } else if style == 61 || style == 62 {
            0.6
        } else {
            1.0
        };
        assert!((tweak - want).abs() < 1e-6, "style {style}");
        assert_eq!(pitch, 0.0);
    }
    // The 10 playing guard (`:686-689`) returns BEFORE the pitch draw.
    let mut roll = |_: i32, _: i32| panic!("the guarded play draws nothing");
    assert_eq!(npc_killed_case4(10, true, &mut roll), None);
    let mut roll = |_: i32, _: i32| 0;
    assert!(npc_killed_case4(10, false, &mut roll).is_some());
}

#[test]
fn s1_e_21a_item_use_case2_remaps_style1_and_tweaks_the_volume() {
    // `LegacySoundPlayer.cs:539-550`: style 1 draws Next(3); 1 → 18, 2 → 19,
    // then the default pitch arm draws Next(-6, 7) (`:632`).
    let seq: [(i32, i32); 2] = [(0, 3), (-6, 7)];
    let mut i = 0;
    let mut roll = |lo: i32, hi: i32| {
        assert_eq!((lo, hi), seq[i], "the draw {i}'s window");
        i += 1;
        1 // remap roll 1 → style 18
    };
    let (num, tweak, pitch) = item_use_case2(1, &mut roll);
    assert_eq!(num, 18);
    assert_eq!(tweak, 1.0);
    assert!((pitch - 1.0 * 0.01).abs() < 1e-6);
    // The per-style volume arms (`:564-578`): 37 → 0.5, 52 → 0.35,
    // 157 → 0.7, 158 → 0.8, 210 → 0.35 — one default pitch draw each.
    for (style, want) in [(37, 0.5f32), (52, 0.35), (157, 0.7), (158, 0.8), (210, 0.35)] {
        let mut roll = |lo: i32, hi: i32| {
            assert_eq!((lo, hi), (-6, 7), "style {style}'s default pitch window");
            0
        };
        let (num, tweak, pitch) = item_use_case2(style, &mut roll);
        assert_eq!(num, style);
        assert!((tweak - want).abs() < 1e-6, "style {style}");
        assert_eq!(pitch, 0.0);
    }
    // Style 156 (`:624-627`): Next(-50, 51) * 0.002 and Volume *= 0.6.
    let mut roll = |lo: i32, hi: i32| {
        assert_eq!((lo, hi), (-50, 51));
        -50
    };
    let (num, tweak, pitch) = item_use_case2(156, &mut roll);
    assert_eq!(num, 156);
    assert!((tweak - 0.6).abs() < 1e-6);
    assert!((pitch - (-50.0 * 0.002)).abs() < 1e-6);
    // The ifs at `:517-535`: 123/116 → 0.5, 129 → 0.6, 124/125 → 0.65,
    // 176 → 0.9.
    for (style, want) in [(123, 0.5f32), (116, 0.5), (129, 0.6), (124, 0.65), (125, 0.65), (176, 0.9)] {
        let mut roll = |_: i32, _: i32| 0;
        let (_, tweak, _) = item_use_case2(style, &mut roll);
        assert!((tweak - want).abs() < 1e-6, "style {style}");
    }
}

/// S1-E-21i — the grass/door pitch windows (`LegacySoundPlayer.cs:714`
/// case 6, `:736`/`:747` cases 8/9): one draw per play, the exact window
/// and the 0.01 scale.
#[test]
fn s1_e_21i_the_grass_and_door_pitch_windows() {
    // Case 6 (`:714`): Next(-30, 31) * 0.01.
    let mut draw = |lo: i32, hi: i32| {
        assert_eq!((lo, hi), (-30, 31), "the grass roll's window");
        12
    };
    assert!(
        (legacy_family_pitch(Family::Grass, &mut draw) - 0.12).abs() < 1e-6,
        "12 * 0.01"
    );
    // Cases 8 and 9 (`:736`/`:747`): the shared Next(-20, 21) * 0.01.
    let mut draw = |lo: i32, hi: i32| {
        assert_eq!((lo, hi), (-20, 21), "the door roll's window");
        -20
    };
    assert!(
        (legacy_family_pitch(Family::Door, &mut draw) - (-0.2)).abs() < 1e-6,
        "-20 * 0.01 — the window floor"
    );
}

/// S1-E-21i — case 28's playing guard (`LegacySoundPlayer.cs:998-1001`)
/// returns BEFORE the pitch draw (`:1005`); a fresh slot draws exactly one
/// Next(-10, 11) * 0.01.
#[test]
fn s1_e_21i_mech_case28_guards_before_the_pitch_draw() {
    // The live-instance refusal draws nothing.
    let mut roll = |_: i32, _: i32| panic!("the guarded play draws nothing");
    assert_eq!(mech_case28(true, &mut roll), None);
    // A fresh slot draws one pitch in the ±0.10 window.
    let mut roll = |lo: i32, hi: i32| {
        assert_eq!((lo, hi), (-10, 11), "the mech roll's window");
        7
    };
    assert!(
        (mech_case28(false, &mut roll).unwrap() - 0.07).abs() < 1e-6,
        "7 * 0.01"
    );
}
