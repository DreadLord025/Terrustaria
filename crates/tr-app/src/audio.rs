//! Audio: original PCM SFX via `SoundEffectReader`, MS-ADPCM music from XACT banks.
//!
//! The music engine mirrors `Main.UpdateAudio` + `LegacyAudioSystem.UpdateCommonTrack`
//! (`Main.cs:11017`, `LegacyAudioSystem.cs:292`): a per-track `musicFade` ramp of
//! ±0.005 per frame, one rodio [`Sink`] per track standing in for the XACT
//! `AudioTrack`, and `Main.newMusic`/`Main.curMusic` selection. Positional SFX use
//! [`crate::sound`] for the `LegacySoundPlayer.PlaySound` attenuation.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::Duration;

use rodio::Source;

use crate::sound::{self, Listener};

/// `MusicID.Count` (`MusicID.cs:228`); `musicFade` is `Main.maxMusic` long.
pub const MAX_MUSIC: usize = 105;

/// `TrackIndex` → `LegacySoundPlayer.PlaySound` `type`. The cue assets the game
/// currently triggers by name.
const SFX_CUES: &[(&str, u32)] = &[
    ("Dig_0", 0),
    ("Dig_1", 0),
    ("Dig_2", 0),
    ("Item_1", 2),
    ("Coin_0", 18),
    ("Player_Hit_0", 1),
    ("NPC_Hit_1", 3),
    ("Double_Jump", 16),
];

fn sound_type(cue: &str) -> u32 {
    SFX_CUES
        .iter()
        .find(|(n, _)| *n == cue)
        .map(|(_, t)| *t)
        .unwrap_or(0)
}

pub struct Audio {
    _stream: Option<rodio::OutputStream>,
    handle: Option<rodio::OutputStreamHandle>,
    cues: HashMap<&'static str, (u16, u32, Vec<i16>)>,
    /// Per-frame `Main.Camera.Center` + the volume sliders (see [`crate::sound`]).
    listener: Listener,
}

impl Audio {
    pub fn new(content: Option<&Path>) -> Self {
        let mut cues = HashMap::new();
        let mut stream = None;
        let mut handle = None;
        if let Some(dir) = content.map(|c| c.join("Sounds")) {
            match rodio::OutputStream::try_default() {
                Ok((s, h)) => {
                    stream = Some(s);
                    handle = Some(h.clone());
                    // The music system shares this output stream.
                    MusicSystem::set_handle_from_system(h);
                }
                Err(e) => log::warn!("audio: no output device ({e})"),
            }
            for (name, _) in SFX_CUES {
                let path = dir.join(format!("{name}.xnb"));
                match load_sound(&path) {
                    Ok(s) => {
                        let samples = s
                            .data
                            .chunks_exact(2)
                            .map(|b| i16::from_le_bytes([b[0], b[1]]))
                            .collect();
                        cues.insert(*name, (s.channels, s.sample_rate, samples));
                    }
                    Err(e) => log::debug!("sound {name}: {e:#}"),
                }
            }
        }
        log::info!("audio: {} PCM cues loaded", cues.len());
        Self {
            _stream: stream,
            handle,
            cues,
            listener: Listener {
                camera_center: (0.0, 0.0),
                sound_volume: 1.0,
                ambient_volume: 1.0,
                quiet_ambient_sounds: false,
                game_menu: false,
            },
        }
    }

    /// Update the listener state the attenuation reads (`Main.Camera.Center`,
    /// `Main.soundVolume`/`Main.ambientVolume`, `Main.gameMenu`).
    pub fn set_listener(&mut self, listener: Listener) {
        self.listener = listener;
    }

    /// A 2D sound (`PlaySound(type)`, no position → no attenuation).
    pub fn play(&self, cue: &str) {
        self.play_at(cue, -1.0, -1.0, 1.0);
    }

    /// A positioned sound (`PlaySound(type, x, y, volumeScale)`), attenuated by
    /// [`crate::sound::play_sound`] against the current listener.
    pub fn play_at(&self, cue: &str, x: f32, y: f32, volume_scale: f32) {
        let Some((channels, rate, samples)) = self.cues.get(cue) else {
            return;
        };
        let Some(handle) = &self.handle else { return };
        let Some(playback) = sound::play_sound(
            sound_type(cue),
            x,
            y,
            self.listener,
            volume_scale,
        ) else {
            return;
        };
        let (left, right) = sound::pan_gains(playback.pan);
        let buf = rodio::buffer::SamplesBuffer::new(*channels, *rate, samples.clone());
        let source = Panned {
            inner: buf.convert_samples::<f32>(),
            channels: *channels,
            left: playback.volume * left,
            right: playback.volume * right,
            channel: 0,
        };
        let _ = handle.play_raw(source);
    }
}

fn load_sound(path: &Path) -> anyhow::Result<tr_content::Sound> {
    tr_content::load_sound(path)
}

/// `SoundEffectInstance.Pan` + `Volume`: scale each channel by its gain.
///
/// XNA applies `Pan` in XAudio2, which the decompilation does not contain, so the
/// reference pan law from [`crate::sound::pan_gains`] is used.
struct Panned<S> {
    inner: S,
    channels: u16,
    left: f32,
    right: f32,
    channel: u16,
}

impl<S: Iterator<Item = f32>> Iterator for Panned<S> {
    type Item = f32;

    fn next(&mut self) -> Option<f32> {
        let sample = self.inner.next()?;
        let gain = match self.channel {
            0 => self.left,
            1 => self.right,
            _ => 1.0,
        };
        self.channel = if self.channels == 0 {
            0
        } else {
            (self.channel + 1) % self.channels
        };
        Some(sample * gain)
    }
}

impl<S: Source<Item = f32>> Source for Panned<S> {
    fn current_frame_len(&self) -> Option<usize> {
        self.inner.current_frame_len()
    }
    fn channels(&self) -> u16 {
        self.inner.channels()
    }
    fn sample_rate(&self) -> u32 {
        self.inner.sample_rate()
    }
    fn total_duration(&self) -> Option<Duration> {
        self.inner.total_duration()
    }
}

/// Global output handle shared with [`Audio`] (the XACT `audioSystem` singleton).
static MUSIC_HANDLE: std::sync::Mutex<Option<rodio::OutputStreamHandle>> =
    std::sync::Mutex::new(None);

/// The per-frame world state `Main.UpdateAudio_DecideOnNewMusic` reads.
#[derive(Clone, Copy, Debug)]
pub struct MusicEnv {
    /// `Main.musicVolume`.
    pub music_volume: f32,
    /// `Main.dayTime`.
    pub day: bool,
    /// Scene centre below the world surface (`SceneMetrics.Center.Y >= worldSurface*16
    /// + screenHeight/2`), i.e. the `Main.cs:12815` branch.
    pub underground: bool,
    /// `Main.cloudAlpha > 0` (raining/storming).
    pub rain: bool,
    /// Music id of the nearest active boss (`NPCID → newMusic`), if any.
    pub boss_music: Option<u32>,
    /// `Main.time` (0..54000).
    pub clock: f64,
}

/// MS-ADPCM music player using the XACT wave bank.
///
/// This ports the XACT track lifecycle: `AudioTracks[i]` are the rodio [`Sink`]s,
/// `musicFade[i]` is `Main.musicFade`, and `newMusic`/`curMusic` follow
/// `Main.UpdateAudio` (`Main.cs:11017-11606`).
pub struct MusicSystem {
    wavebank_path: Option<PathBuf>,
    /// `Music_{id}` → wave bank entry index, from `Sound Bank.xsb`.
    wave_indices: HashMap<u32, usize>,
    /// `Main.curMusic` / `Main.newMusic`.
    cur_music: u32,
    new_music: u32,
    /// `Main.musicFade[0..MAX_MUSIC]`.
    music_fade: Vec<f32>,
    /// `AudioTracks[0..MAX_MUSIC]`; `Some` while a track is loaded/playing.
    tracks: Vec<Option<rodio::Sink>>,
    handle: Option<rodio::OutputStreamHandle>,
    /// `Main.dayMusic` / `Main.ugMusic`.
    day_music: u32,
    ug_music: u32,
    /// `Main.rand` stand-in for `Main.rand.Next(2)`.
    rng: u64,
    /// `Main.lastMusicPlayed`.
    last_music_played: u32,
}

impl MusicSystem {
    pub fn new(content_dir: Option<&Path>) -> Self {
        let mut wave_indices = HashMap::new();
        let wavebank_path = content_dir.map(|c| c.join("Wave Bank.xwb"));
        if let Some(dir) = content_dir {
            match tr_content::XactSoundBank::load(&dir.join("Sound Bank.xsb")) {
                Ok(bank) => {
                    for (name, _, track) in &bank.cues {
                        if let Some(id) = name
                            .strip_prefix("Music_")
                            .and_then(|n| n.parse::<u32>().ok())
                        {
                            wave_indices.insert(id, *track as usize);
                        }
                    }
                    log::info!("music: {} cue indices loaded", wave_indices.len());
                }
                Err(e) => log::warn!("music: cannot read the sound bank ({e:#})"),
            }
        }
        Self {
            wavebank_path,
            wave_indices,
            cur_music: 0,
            new_music: 0,
            music_fade: vec![0.0; MAX_MUSIC],
            tracks: (0..MAX_MUSIC).map(|_| None).collect(),
            handle: None,
            day_music: 0,
            ug_music: 0,
            rng: 0x9E37_79B9_7F4A_7C15,
            last_music_played: 0,
        }
    }

    /// Set the global audio handle after initialization.
    pub fn set_handle_from_system(handle: rodio::OutputStreamHandle) {
        *MUSIC_HANDLE.lock().unwrap() = Some(handle);
    }

    fn next_rand(&mut self) -> u32 {
        // xorshift64*; only the parity of `Main.rand.Next(2)` is observable.
        let mut x = self.rng;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.rng = x;
        (x.wrapping_mul(0x2545_F491_4F6C_DD1D) >> 33) as u32
    }

    /// `LegacyAudioSystem.IsTrackPlaying` — a loaded sink is playing (music loops).
    fn is_track_playing(&self, i: usize) -> bool {
        self.tracks.get(i).is_some_and(Option::is_some)
    }

    /// `Main.UpdateAudio`, minus the focus/moon-lord/ambient specials the port has
    /// no state for. Call once per frame.
    pub fn update(&mut self, env: MusicEnv) {
        if self.handle.is_none() {
            self.handle = MUSIC_HANDLE.lock().unwrap().clone();
            if self.handle.is_none() {
                return;
            }
        }

        self.decide_new_music(env);
        if env.music_volume == 0.0 {
            self.new_music = 0;
        }
        self.cur_music = self.new_music;

        let is_main_track_audible = self
            .music_fade
            .get(self.cur_music as usize)
            .is_some_and(|f| *f > 0.25);

        for i in 1..MAX_MUSIC {
            // Only loaded cues have an `AudioTrack`.
            if !self.wave_indices.contains_key(&(i as u32)) {
                continue;
            }
            let mut volume = self.music_fade[i] * env.music_volume;
            if (62..=88).contains(&i) {
                volume *= 0.9;
            } else if i == 52 {
                volume *= 1.15;
                if volume > 1.0 {
                    volume = 1.0;
                }
            }
            if i == self.cur_music as usize {
                self.update_common_track(true, i, volume);
            } else {
                self.update_common_track_toward_stopping(i, volume, is_main_track_audible);
            }
        }
    }

    /// `Main.UpdateAudio_DecideOnNewMusic`, reduced to the states this port models:
    /// boss, underground (`Main.ugMusic`), rain, surface day (`Main.dayMusic`) and
    /// night. The biome branches (`Main.cs:12690-12925`) need `SceneMetrics` and are
    /// left for when those zones exist.
    fn decide_new_music(&mut self, env: MusicEnv) {
        // `if (Main.musicVolume == 0f) Main.newMusic = 0;`
        if env.music_volume == 0.0 {
            self.new_music = 0;
            return;
        }
        self.last_music_played = self
            .music_fade
            .iter()
            .enumerate()
            .filter(|(_, f)| **f > 0.0)
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
            .filter(|(_, f)| **f == 1.0)
            .map(|(i, _)| i as u32)
            .unwrap_or(self.last_music_played);

        if let Some(boss) = env.boss_music {
            // `if (flag) Main.newMusic = 5;` / King Slime → 93 (`Main.cs:12598`).
            self.new_music = boss;
            return;
        }
        if env.underground {
            // `Main.cs:12830`: alternate Underground (4) / UndergroundAlt (31).
            if self.ug_music == 0 {
                self.ug_music = 4;
            }
            if !self.is_track_playing(4) && !self.is_track_playing(31) {
                if self.music_fade[4] == 1.0 {
                    self.music_fade[31] = 1.0;
                }
                if self.music_fade[31] == 1.0 {
                    self.music_fade[4] = 1.0;
                }
                if self.next_rand() % 2 == 0 {
                    self.ug_music = 4;
                    self.music_fade[31] = 0.0;
                } else {
                    self.ug_music = 31;
                    self.music_fade[4] = 0.0;
                }
            }
            self.new_music = self.ug_music;
            return;
        }
        if env.rain {
            // `Main.cs:12932` (day) / `12973` (night).
            self.new_music = if env.day {
                if env.clock < 10800.0 {
                    59 // MorningRain
                } else {
                    19 // Rain
                }
            } else {
                19
            };
            return;
        }
        if env.day {
            // `Main.cs:12945`: alternate OverworldDay (1) / OverworldDayAlt (18).
            if self.day_music == 0 {
                self.day_music = 1;
            }
            if !self.is_track_playing(1) && !self.is_track_playing(18) {
                self.day_music = if self.next_rand() % 2 == 0 { 1 } else { 18 };
            }
            self.new_music = self.day_music;
            return;
        }
        // `Main.cs:12979`.
        self.new_music = 3; // OverworldNight
    }

    /// `LegacyAudioSystem.UpdateCommonTrack` (`LegacyAudioSystem.cs:292`).
    fn update_common_track(&mut self, active: bool, i: usize, total_volume: f32) {
        self.music_fade[i] += 0.005;
        if self.music_fade[i] > 1.0 {
            self.music_fade[i] = 1.0;
        }
        if !self.is_track_playing(i) && active {
            self.start_track(i, total_volume);
        } else if let Some(Some(sink)) = self.tracks.get(i) {
            sink.set_volume(total_volume);
        }
    }

    /// `LegacyAudioSystem.UpdateCommonTrackTowardStopping` (`LegacyAudioSystem.cs:320`).
    fn update_common_track_toward_stopping(
        &mut self,
        i: usize,
        total_volume: f32,
        is_main_track_audible: bool,
    ) {
        if !self.is_track_playing(i) {
            self.music_fade[i] = 0.0;
            return;
        }
        if is_main_track_audible {
            self.music_fade[i] -= 0.005;
        } else if self.cur_music == 0 {
            self.music_fade[i] = 0.0;
        }
        if self.music_fade[i] <= 0.0 {
            self.music_fade[i] = 0.0;
            self.stop_track(i);
        } else if let Some(Some(sink)) = self.tracks.get(i) {
            sink.set_volume(total_volume);
        }
    }

    /// `AudioTracks[i].Reuse(); SetVariable("Volume"); Play()`.
    fn start_track(&mut self, i: usize, volume: f32) {
        let Some(handle) = self.handle.as_ref() else {
            return;
        };
        let Some(&entry_index) = self.wave_indices.get(&(i as u32)) else {
            return;
        };
        let Some(wb_path) = self.wavebank_path.as_ref() else {
            return;
        };
        let Ok(bank) = tr_content::XactWaveBank::open(wb_path) else {
            return;
        };
        let Some(entry) = bank.entries.get(entry_index) else {
            return;
        };
        let format = entry.format;
        let block_align = format.block_align();
        if format.codec() != tr_content::Codec::Adpcm || block_align == 0 {
            log::debug!("music {i}: unsupported codec {:?}", format.codec());
            return;
        }
        let Ok(data) = bank.read_data(wb_path, entry_index) else {
            return;
        };
        let Ok(samples) = tr_content::decode_msadpcm(
            &data,
            format.sample_rate,
            format.channels,
            block_align as u16,
        ) else {
            return;
        };
        let Ok(sink) = rodio::Sink::try_new(handle) else {
            return;
        };
        let buffer = rodio::buffer::SamplesBuffer::new(format.channels, format.sample_rate, samples);
        sink.append(buffer.repeat_infinite());
        sink.set_volume(volume);
        self.tracks[i] = Some(sink);
        log::info!("music: track {i} started (entry {entry_index})");
    }

    /// `AudioTracks[i].SetVariable("Volume", 0); Stop()`.
    fn stop_track(&mut self, i: usize) {
        if let Some(Some(sink)) = self.tracks.get(i) {
            sink.stop();
        }
        self.tracks[i] = None;
    }
}
