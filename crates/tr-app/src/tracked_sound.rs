//! The tracked-sound layer — `Terraria.Audio`'s `SoundPlayer` /
//! `ActiveSound` / `SoundStyle` / `SoundPlayOverrides` (`SoundPlayer.cs:1-132`,
//! `ActiveSound.cs:1-166`, `SoundStyle.cs:1-51`, `SoundPlayOverrides.cs`,
//! S1-E-23). The pool is `ReLogic`'s `SlotVector<ActiveSound>(4096)`
//! (`SoundPlayer.cs:9`, ported in `tr_core::slot`); every tracked sound lives
//! in a versioned slot the callers hold as a `SlotId` (`SceneState.cs:32-34`'s
//! blizzard ids, the NPC `localAI` float dance via `SlotId::to_float`).
//!
//! What the layer does that the fire-and-forget legacy path does not: the
//! sound stays addressable — `ActiveSound.Update` re-computes the intended
//! volume every frame against the live listener (`ActiveSound.cs:127-140`'s
//! `DetermineIntendedVolume`, `:142-165`), the looped plays carry a
//! `LoopedPlayCondition` that stops them the moment it turns false, and the
//! `MaxTrackedInstances` cap rejects new plays of a full style
//! (`SoundEngine.cs:103-107`'s `GetActiveSoundCount` guard).
//!
//! Engine deviations (ANALYSIS, S1-E-23): the per-frame re-pan is not
//! reproducible on a rodio sink (the pan is baked at creation — the same
//! documented gap as the liquid loops), and `SoundInstanceGarbageCollector`
//! maps to nothing (rodio drops its finished sources).

use tr_core::slot::{SlotId, SlotVector};

use crate::sound::Listener;

/// `Terraria.Audio.SoundType` (`SoundType.cs:3-9`): the slider
/// `DetermineIntendedVolume` multiplies by (`ActiveSound.cs:152-163`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SoundType {
    /// `Main.soundVolume`.
    Sound,
    /// `Main.ambientVolume`.
    Ambient,
    /// `Main.musicVolume`.
    Music,
}

/// The `SoundStyle` stand-in for the tracked plays (`SoundStyle.cs:6-51`'s
/// volume/pitch-variance/type + `LegacySoundStyle.cs`'s trackable identity:
/// the cue family and its variations). The C# compares styles by reference
/// (`SoundPlayer.cs:113`'s `value.Style == style`); the port compares the
/// cue family — the `SoundID` entries the port plays are distinct assets, so
/// the counts land the same.
#[derive(Clone, Debug, PartialEq)]
pub struct TrackedStyle {
    /// The trackable base asset name (`Sounds/Custom/<cue>.xnb`, the
    /// `LegacySoundPlayer.cs:257` path); the variation assets are
    /// `<cue>_<n>` for `variations > 1` (`SoundID.cs:1201-1219`).
    pub cue: &'static str,
    /// `LegacySoundStyle.Variations` (`LegacySoundStyle.cs:14`).
    pub variations: u32,
    /// `SoundStyle.Volume` (`SoundStyle.cs:16`).
    pub volume: f32,
    /// `SoundStyle.PitchVariance` (`SoundStyle.cs:18`) — `GetRandomPitch`
    /// draws `NextFloat() * variance - variance * 0.5` (`:40-43`).
    pub pitch_variance: f32,
    /// `SoundStyle.Type` (`SoundStyle.cs:20`).
    pub sound_type: SoundType,
    /// `SoundStyle.MaxTrackedInstances` (`LegacySoundStyle.cs:26-33`) — 0 is
    /// uncapped.
    pub max_tracked_instances: u32,
}

impl TrackedStyle {
    /// `SoundStyle.GetRandomSound` for the trackable family: the `Style`
    /// getter picks the variation — `LegacySoundStyle.Random.Next(_style,
    /// _style + Variations)` (`LegacySoundStyle.cs:21-28`) — and the port
    /// resolves it to the `<cue>_<n>` asset. `rng` is the engine's
    /// `LegacySoundStyle.Random` stand-in (the C# seeds the static with
    /// `Environment.TickCount`; ANALYSIS, S1-E-23).
    pub fn random_sound(&self, rng: &mut tr_core::unified_random::UnifiedRandom) -> String {
        if self.variations != 1 {
            let n = rng.next_range(0, self.variations as i32).max(0) as u32;
            format!("{}_{}", self.cue, n)
        } else {
            self.cue.to_string()
        }
    }

    /// `SoundStyle.GetRandomPitch` (`SoundStyle.cs:40-43`) — the roll rides
    /// the `SoundStyle._random` static's stand-in (a second unobservable
    /// stream, distinct from `LegacySoundStyle.Random`).
    pub fn get_random_pitch(&self, rng: &mut tr_core::unified_random::UnifiedRandom) -> f32 {
        rng.next_float() * self.pitch_variance - self.pitch_variance * 0.5
    }
}

/// `Terraria.Audio.SoundPlayOverrides` (`SoundPlayOverrides.cs`): the
/// caller's `Volume` override (`ActiveSound.cs:37-43`'s `UseOverrides`).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SoundPlayOverrides {
    /// `SoundPlayOverrides.Volume` — `Some` replaces the ActiveSound's
    /// default 1 (`ActiveSound.cs:49`).
    pub volume: Option<f32>,
}

/// `ActiveSound.LoopedPlayCondition` (`ActiveSound.cs:9`): the looped play
/// stops the moment it turns false (`:131-135`).
pub type LoopedPlayCondition = Box<dyn Fn() -> bool + Send>;

/// `Terraria.Audio.ActiveSound` (`ActiveSound.cs:7-166`): one tracked sound —
/// its style, its live position/volume override, the instance pitch and the
/// engine sink. The engine (`audio.rs`) builds it with the initial gain
/// already applied (`:85`'s `Volume = DetermineIntendedVolume()`); the
/// per-frame [`ActiveSound::update`] keeps the volume live.
pub struct ActiveSound {
    /// `ActiveSound.IsGlobal` (`:11`): a global play skips the attenuation
    /// and the pan (`:145-150`).
    pub is_global: bool,
    /// `ActiveSound.Position` (`:13`) — live: the sizzle site re-points it
    /// every update (`Player.cs:28599`).
    pub position: (f32, f32),
    /// `ActiveSound.Volume` (`:15`): the caller's override the per-frame
    /// update multiplies in (`SceneState.cs:229-235`'s split).
    pub volume: f32,
    /// `ActiveSound.Pitch` (`:17`): the instance pitch, baked at play
    /// (`:83-84`'s `Pitch += Style.GetRandomPitch()`).
    pub pitch: f32,
    /// `ActiveSound.Condition` (`:19`) — the looped play's stop condition.
    pub condition: Option<LoopedPlayCondition>,
    /// `ActiveSound.Style` (`:23`) — the port's copied fields (the cue
    /// identity, the style volume, the slider type).
    pub style: TrackedStyle,
    /// `ActiveSound.Sound` (`:21`) — the engine instance: a rodio sink (the
    /// port's `SoundEffectInstance`; `None` headless).
    sound: Option<rodio::Sink>,
    /// The pan baked at creation (the deviation: `DetermineIntendedVolume`
    /// re-pans the live XNA instance every update, `:148`; a rodio sink
    /// cannot).
    pan: f32,
    /// The test seam: a headless run has no output device, so the pool tests
    /// force the playing state to exercise the count/cap/update paths.
    #[cfg(test)]
    test_playing: bool,
}

impl ActiveSound {
    /// The engine constructor — every field the C# ctors set
    /// (`:45-78`), plus the baked pan and the pre-built sink.
    pub(crate) fn new(
        style: TrackedStyle,
        position: (f32, f32),
        is_global: bool,
        overrides: SoundPlayOverrides,
        condition: Option<LoopedPlayCondition>,
        sound: Option<rodio::Sink>,
        pan: f32,
        pitch: f32,
    ) -> Self {
        Self {
            is_global,
            position,
            volume: overrides.volume.unwrap_or(1.0),
            pitch,
            condition,
            style,
            sound,
            pan,
            #[cfg(test)]
            test_playing: false,
        }
    }

    /// `ActiveSound.IsPlaying` (`:25-35`): the instance's `State == Playing`
    /// — a finished rodio sink is empty; the test seam forces the state.
    pub fn is_playing(&self) -> bool {
        #[cfg(test)]
        if self.test_playing {
            return true;
        }
        self.sound.as_ref().is_some_and(|sink| !sink.empty())
    }

    /// `ActiveSound.Stop` (`:103-109`).
    pub fn stop(&mut self) {
        if let Some(sink) = &self.sound {
            sink.stop();
        }
    }

    /// `ActiveSound.Pause`/`Resume` (`:111-125`).
    pub fn pause(&mut self) {
        if let Some(sink) = &self.sound {
            sink.pause();
        }
    }

    /// The `Resume` half (`:117-125`).
    pub fn resume(&mut self) {
        if let Some(sink) = &self.sound {
            sink.play();
        }
    }

    /// `ActiveSound.Update` (`:127-140`): the loop condition stops the sound
    /// outright, then the intended volume re-applies (`:136-138`'s
    /// `Sound.Volume = volume; Sound.Pitch = this.Pitch` — the pitch is
    /// baked here, the volume is live).
    pub fn update(&mut self, listener: &Listener) {
        if let Some(condition) = &self.condition {
            if !condition() {
                self.stop();
                return;
            }
        }
        let gain = self.determine_intended_volume(listener);
        if let Some(sink) = &self.sound {
            sink.set_volume(gain);
        }
    }

    /// `ActiveSound.DetermineIntendedVolume` (`:142-165`): the attenuation
    /// (unless global), × `Style.Volume × Volume`, × the `SoundType`'s
    /// slider, clamped to 0..1. The pan leg (`:148`) is the baked one.
    pub fn determine_intended_volume(&self, listener: &Listener) -> f32 {
        let mut num = 1.0;
        if !self.is_global {
            // `:147-150`.
            let dx = self.position.0 - listener.camera_center.0;
            let dy = self.position.1 - listener.camera_center.1;
            let distance = (dx * dx + dy * dy).sqrt();
            num = (1.0 - distance / crate::sound::SOUND_ATTENUATION_DISTANCE).clamp(0.0, 1.0);
        }
        num *= self.style.volume * self.volume;
        // `:152-163`: the slider per `SoundType`.
        num *= match self.style.sound_type {
            SoundType::Sound => listener.sound_volume,
            SoundType::Ambient => listener.ambient_volume,
            SoundType::Music => 1.0,
        };
        num.clamp(0.0, 1.0)
    }

    /// The baked pan (read by the engine at creation).
    pub(crate) fn pan(&self) -> f32 {
        self.pan
    }

    /// The test seam: force the playing state.
    #[cfg(test)]
    pub(crate) fn set_test_playing(&mut self, playing: bool) {
        self.test_playing = playing;
    }

    /// The engine attaches the built instance after computing the initial
    /// gain (`ActiveSound.cs:85`'s order — `DetermineIntendedVolume` at
    /// creation, then the instance starts).
    pub(crate) fn attach_sink(&mut self, sink: rodio::Sink) {
        self.sound = Some(sink);
    }
}

/// `Terraria.Audio.SoundPlayer` (`SoundPlayer.cs:7-132`): the 4096-slot
/// pool. `Update` runs per frame (`SoundEngine.cs:64`) and reaps the dead;
/// `GetActiveSoundCount` feeds the `MaxTrackedInstances` cap
/// (`SoundEngine.cs:103-107`).
pub struct SoundPlayer {
    tracked_sounds: SlotVector<Option<ActiveSound>>,
    /// `SoundEngine.AreSoundsPaused` (`SoundEngine.cs:66`): the pause flip's
    /// edge state.
    are_sounds_paused: bool,
}

impl Default for SoundPlayer {
    fn default() -> Self {
        Self {
            tracked_sounds: SlotVector::new(4096),
            are_sounds_paused: false,
        }
    }
}

impl SoundPlayer {
    /// `SoundEngine.PlayTrackedSound`'s `MaxTrackedInstances` guard
    /// (`SoundEngine.cs:104-107`): a full style rejects the play. The
    /// distance guard (`SoundPlayer.cs:17-20`) and the sink construction
    /// stay at the [`crate::audio::Audio`] seam (they need the decoded cue
    /// and the live listener).
    pub fn style_is_at_cap(&self, style: &TrackedStyle) -> bool {
        style.max_tracked_instances > 0
            && self.get_active_sound_count(style) >= style.max_tracked_instances as usize
    }

    /// `SoundPlayer.Play/PlayLooped`'s `Add` (`SoundPlayer.cs:22`/`:35`/
    /// `:51`): the engine hands over the built [`ActiveSound`].
    pub(crate) fn add(&mut self, sound: ActiveSound) -> SlotId {
        self.tracked_sounds.add(Some(sound))
    }

    /// `SoundPlayer.GetActiveSound` (`SoundPlayer.cs:54-61`).
    pub fn get_active_sound(&self, id: SlotId) -> Option<&ActiveSound> {
        self.tracked_sounds.get(id)?.as_ref()
    }

    /// The `GetActiveSound(...)` reference the C# mutates through
    /// (`SceneState.cs:233`'s `activeSound3.Volume = ...`).
    pub fn get_active_sound_mut(&mut self, id: SlotId) -> Option<&mut ActiveSound> {
        self.tracked_sounds.get_mut(id)?.as_mut()
    }

    /// `SoundEngine.Update`'s pause half (`SoundEngine.cs:56-63`): the
    /// `FocusHelper.PauseSounds` edge flips the whole pool
    /// (`SoundPlayer.cs:63-77`).
    pub fn update_pause_state(&mut self, pause_sounds: bool) {
        if !self.are_sounds_paused && pause_sounds {
            let ids: Vec<SlotId> = self.tracked_sounds.pairs().map(|(id, _)| id).collect();
            for id in ids {
                if let Some(sound) = self.tracked_sounds.get_mut(id).and_then(|o| o.as_mut()) {
                    sound.pause();
                }
            }
            self.are_sounds_paused = true;
        } else if self.are_sounds_paused && !pause_sounds {
            let ids: Vec<SlotId> = self.tracked_sounds.pairs().map(|(id, _)| id).collect();
            for id in ids {
                if let Some(sound) = self.tracked_sounds.get_mut(id).and_then(|o| o.as_mut()) {
                    sound.resume();
                }
            }
            self.are_sounds_paused = false;
        }
    }

    /// `SoundPlayer.StopAll` (`SoundPlayer.cs:79-86`): the Reload path
    /// (`SoundEngine.cs:76-80`) — the port has no hot-reload, the API stays
    /// for completeness.
    pub fn stop_all(&mut self) {
        let ids: Vec<SlotId> = self.tracked_sounds.pairs().map(|(id, _)| id).collect();
        for id in ids {
            if let Some(sound) = self.tracked_sounds.get_mut(id).and_then(|o| o.as_mut()) {
                sound.stop();
            }
        }
        self.tracked_sounds.clear();
    }

    /// `SoundPlayer.Update` (`SoundPlayer.cs:88-105`): every sound updates
    /// against the live listener and the dead ones leave the pool.
    pub fn update(&mut self, listener: &Listener) {
        let mut dead: Vec<SlotId> = Vec::new();
        for (id, _) in self.tracked_sounds.pairs() {
            dead.push(id);
        }
        for id in dead {
            if let Some(sound) = self.tracked_sounds.get_mut(id).and_then(|o| o.as_mut()) {
                sound.update(listener);
                if !sound.is_playing() {
                    self.tracked_sounds.remove(id);
                }
            }
        }
    }

    /// `SoundPlayer.GetActiveSoundCount` (`SoundPlayer.cs:107-119`): the
    /// playing instances of one style (the cue family is the identity
    /// stand-in).
    pub fn get_active_sound_count(&self, style: &TrackedStyle) -> usize {
        self.tracked_sounds
            .pairs()
            .filter_map(|(_, sound)| sound.as_ref())
            .filter(|sound| sound.style.cue == style.cue && sound.is_playing())
            .count()
    }

    /// `SoundPlayer.FindActiveSound` (`SoundPlayer.cs:121-131`) — note the
    /// C# has no `IsPlaying` filter here.
    pub fn find_active_sound(&self, style: &TrackedStyle) -> Option<&ActiveSound> {
        self.tracked_sounds
            .pairs()
            .find(|(_, sound)| sound.as_ref().is_some_and(|s| s.style.cue == style.cue))
            .and_then(|(_, sound)| sound.as_ref())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The blizzard outside loop's style (`SoundID.cs:1599`'s
    /// `CreateTrackable("blizzard_strong_loop", SoundType.Ambient)
    /// .WithVolume(0.5f)`).
    fn strong_style() -> TrackedStyle {
        TrackedStyle {
            cue: "blizzard_strong_loop",
            variations: 1,
            volume: 0.5,
            pitch_variance: 0.0,
            sound_type: SoundType::Ambient,
            max_tracked_instances: 0,
        }
    }

    fn capped_style(cap: u32) -> TrackedStyle {
        TrackedStyle {
            max_tracked_instances: cap,
            ..strong_style()
        }
    }

    fn sound(style: TrackedStyle, overrides: SoundPlayOverrides) -> ActiveSound {
        ActiveSound::new(style, (0.0, 0.0), true, overrides, None, None, 0.0, 0.0)
    }

    #[test]
    fn the_pool_tracks_addresses_and_reaps_the_dead() {
        // `SoundPlayer.cs:22`'s Add + `:88-105`'s Update: a live sound stays
        // addressable through its SlotId, a dead one leaves the pool.
        let mut player = SoundPlayer::default();
        let mut active = sound(strong_style(), SoundPlayOverrides::default());
        active.set_test_playing(true);
        let id = player.add(active);
        assert!(player.get_active_sound(id).is_some());
        assert_eq!(player.get_active_sound_count(&strong_style()), 1);

        // The instance finishes: the update reaps the slot.
        let stale = player.get_active_sound_mut(id).unwrap();
        stale.set_test_playing(false);
        player.update(& Listener {
            camera_center: (0.0, 0.0),
            sound_volume: 1.0,
            ambient_volume: 1.0,
            quiet_ambient_sounds: false,
            game_menu: false,
            generating_or_loading: false,
            server: false,
        });
        assert!(
            player.get_active_sound(id).is_none(),
            "Update removed the finished sound"
        );
        assert_eq!(player.get_active_sound_count(&strong_style()), 0);
    }

    #[test]
    fn the_max_tracked_instances_cap_rejects_a_full_style() {
        // `SoundEngine.cs:104-107`: `GetActiveSoundCount(style) >=
        // style.MaxTrackedInstances` → `SlotId.Invalid` (the play seam's
        // rejection is asserted through `style_is_at_cap`).
        let mut player = SoundPlayer::default();
        let mut first = sound(capped_style(1), SoundPlayOverrides::default());
        first.set_test_playing(true);
        let id = player.add(first);
        assert!(player.style_is_at_cap(&capped_style(1)));
        assert!(!player.style_is_at_cap(&strong_style()), "0 is uncapped");

        // A finished instance frees the cap (the count filters on
        // IsPlaying, `SoundPlayer.cs:113`).
        player
            .get_active_sound_mut(id)
            .unwrap()
            .set_test_playing(false);
        assert!(!player.style_is_at_cap(&capped_style(1)));
    }

    #[test]
    fn the_blizzard_split_rides_the_volume_override() {
        // `SceneState.cs:229-235`: the split lands on the ActiveSound.Volume
        // override and DetermineIntendedVolume multiplies the style's 0.5
        // and the ambient slider in (`ActiveSound.cs:151-158`).
        let mut player = SoundPlayer::default();
        let id = player.add(sound(strong_style(), SoundPlayOverrides::default()));
        let listener = Listener {
            camera_center: (0.0, 0.0),
            sound_volume: 1.0,
            ambient_volume: 0.8,
            quiet_ambient_sounds: false,
            game_menu: false,
            generating_or_loading: false,
            server: false,
        };
        let gain = {
            let sound_ref = player
                .get_active_sound_mut(id)
                .expect("the added sound is live");
            sound_ref.volume = 0.9;
            sound_ref.determine_intended_volume(&listener)
        };
        assert!((gain - 0.5 * 0.9 * 0.8).abs() < 1e-6, "gain {gain}");
    }

    #[test]
    fn the_variation_and_pitch_picks_follow_the_style() {
        // `LegacySoundStyle.cs:21-28`'s variation pick + `SoundStyle.cs:40-43`'s
        // pitch roll — deterministic over the fixed-seed stand-ins.
        let style = TrackedStyle {
            cue: "dd2_betsy_wind_attack",
            variations: 3,
            volume: 1.0,
            pitch_variance: 0.1,
            sound_type: SoundType::Sound,
            max_tracked_instances: 0,
        };
        let mut legacy_rng = tr_core::unified_random::UnifiedRandom::new(42);
        let mut pitch_rng = tr_core::unified_random::UnifiedRandom::new(42);
        let asset = style.random_sound(&mut legacy_rng);
        assert!(
            asset.starts_with("dd2_betsy_wind_attack_"),
            "asset {asset}"
        );
        let pitch = style.get_random_pitch(&mut pitch_rng);
        assert!(
            pitch >= -0.05 && pitch <= 0.05,
            "the roll sits inside ±variance/2: {pitch}"
        );
        let single = TrackedStyle {
            variations: 1,
            ..style
        };
        let mut rng = tr_core::unified_random::UnifiedRandom::new(1);
        assert_eq!(
            single.random_sound(&mut rng),
            "dd2_betsy_wind_attack",
            "one variation keeps the base name"
        );
    }
}
