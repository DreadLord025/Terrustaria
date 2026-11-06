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
    // `SoundEngine.PlaySound(8/:9)` — `WorldGen.OpenDoor` (`WorldGen.cs:37732`)
    // and `CloseDoor` (`:31906`).
    ("Door_Opened", 8),
    ("Door_Closed", 9),
    // `SoundID.Item4 = new LegacySoundStyle(2, 4)` (`SoundID.cs:1387`) — the
    // dial arm (`Player.cs:32642`/:32652, `SoundEngine.PlaySound(SoundID.Item4,
    // ...)`); the legacy Item track, like `Item_1` above.
    ("Item_4", 2),
    // `SoundEngine.PlaySound(7)` — the craft grant (`CraftingEffects.cs:51`,
    // `SpawnEffects_BeforeGrantingItem` ← `OnCraft`); `SoundID.Grab = 7`
    // (`SoundID.cs`), the `Grab.xnb` cue.
    ("Grab", 7),
    // `SoundID.Item53/55/56` — the minecart cues: the landing thud
    // (`DelegateMethods.cs:248-251`), the rolling rattle
    // (`Player.cs:19870/:19928`) and the bouncy bumper
    // (`DelegateMethods.cs:253-256`); the legacy Item track like `Item_1`.
    ("Item_53", 2),
    ("Item_55", 2),
    ("Item_56", 2),
    // `SoundID.Roar` = 15 (`SoundID.cs`) — the boss-summon roar
    // (`Player.cs:43765`'s `SoundEngine.PlaySound(15, ...)`; the `Roar_0`
    // cue family).
    ("Roar_0", 15),
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
    /// The two `SoundInstanceLiquid[n]` stand-ins (`LegacySoundPlayer.cs:322`):
    /// 0 = the waterfall, 1 = the lavafall. `Some` while the loop plays.
    liquid_loops: [Option<rodio::Sink>; 2],
    /// The decoded `Sounds/Liquid_{0,1}` loops (channels, rate, samples).
    liquid_cues: [Option<(u16, u32, Vec<i16>)>; 2],
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
        // `LegacySoundPlayer.cs:223-225`: `Sounds/Liquid_{0,1}` — the
        // waterfall and lavafall loop assets.
        let mut liquid_cues: [Option<(u16, u32, Vec<i16>)>; 2] = [None, None];
        let sounds_dir = content.map(|c| c.join("Sounds"));
        for (n, slot) in liquid_cues.iter_mut().enumerate() {
            let Some(sounds_dir) = &sounds_dir else { break };
            let path = sounds_dir.join(format!("Sounds/Liquid_{n}.xnb"));
            match load_sound(&path) {
                Ok(snd) => {
                    let samples = snd
                        .data
                        .chunks_exact(2)
                        .map(|b| i16::from_le_bytes([b[0], b[1]]))
                        .collect();
                    *slot = Some((snd.channels, snd.sample_rate, samples));
                }
                Err(e) => log::debug!("liquid loop {n}: {e:#}"),
            }
        }
        log::info!("audio: {} PCM cues loaded", cues.len());
        Self {
            _stream: stream,
            handle,
            cues,
            liquid_loops: [None, None],
            liquid_cues,
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
        let Some(playback) = sound::play_sound(sound_type(cue), x, y, self.listener, volume_scale)
        else {
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

    /// `LegacySoundPlayer`'s case 34/35 (`LegacySoundPlayer.cs:1249-1307`): the
    /// two persistent liquid loops whose Volume/Pan update in place every ¼ s
    /// tick (`Main.Ambience`, `Main.cs:13466-13515`). `volume <= 0` stops the
    /// channel outright (the vanilla stops the instance; no fade). The pan is
    /// baked at creation (a documented deviation — XNA re-pans the live
    /// instance).
    pub fn set_liquid_loop(&mut self, channel: usize, volume: f32, pan: f32) {
        let Some((channels, rate, samples)) = self.liquid_cues.get(channel).cloned().flatten()
        else {
            return;
        };
        let Some(handle) = &self.handle else { return };
        let (left, right) = crate::sound::pan_gains(pan);
        if volume <= 0.0 {
            if let Some(Some(sink)) = self.liquid_loops.get_mut(channel) {
                sink.stop();
                self.liquid_loops[channel] = None;
            }
            return;
        }
        if let Some(Some(sink)) = self.liquid_loops.get_mut(channel) {
            sink.set_volume(volume);
            return;
        }
        // A fresh looped sink: the decoded PCM wrapped in an infinite repeat.
        let buf =
            rodio::buffer::SamplesBuffer::new(channels, rate, samples).convert_samples::<f32>();
        let source = Panned {
            inner: buf.repeat_infinite(),
            channels,
            left: left.max(right).max(0.0),
            right: right.max(left).max(0.0),
            channel: 0,
        };
        if let Ok(sink) = rodio::Sink::try_new(handle) {
            sink.append(source);
            sink.set_volume(volume);
            self.liquid_loops[channel] = Some(sink);
        }
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
    /// `Main.cloudAlpha` (the rain coverage the volume follows).
    pub cloud_alpha: f32,
    pub rain: bool,
    // — the M1.4-BG-20 ambience-track inputs (`Main.cs:11527-11600`) —
    /// `SceneMetrics.ZoneRain`.
    pub zone_rain: bool,
    /// `SceneMetrics.ZoneSnow`.
    pub zone_snow: bool,
    /// `SceneMetrics.SurfaceAtmospherics`.
    pub surface_atmospherics: bool,
    /// `Main.windSpeedCurrent`.
    pub wind_speed: f32,
    /// `Main.shimmerAlpha` (the shimmer mutes both ambience tracks).
    pub shimmer_alpha: f32,
    /// `Main.atmo` (the depth factor, `Main.cs:63185`).
    pub atmo: f32,
    /// `Main.ambientVolume` (the user slider).
    pub ambient_volume: f32,
    /// Music id of the nearest active boss (`NPCID → newMusic`), if any.
    pub boss_music: Option<u32>,
    /// `Main.time` (0..54000).
    pub clock: f64,
    // — the M6-MUSIC-1 chain inputs (`UpdateAudio_DecideOnNewMusic`,
    // `Main.cs:12187-12988`), in ladder order —
    /// `Main.eclipse` with the surface gate (`:12644`).
    pub eclipse: bool,
    /// `SceneMetrics.ZoneShimmer` (`:12668`).
    pub shimmer: bool,
    /// `SceneMetrics.TownNPCCount` (`:12199`) — 0 until M4-TOWN lands.
    pub town_npcs: i32,
    /// `SceneMetrics.ZoneShadowCandle` — the town-music cut (`:12199`).
    pub shadow_candle: bool,
    /// `SceneMetrics.ZoneSandstorm` (`:12680`).
    pub sandstorm: bool,
    /// The scene centre below `Main.UnderworldLayer` (`:12682`).
    pub underworld: bool,
    /// `Main.bloodMoon` — the night tail (`:12968`).
    pub blood_moon: bool,
    /// `Main._shouldUseWindyDayMusic` (`:12961`).
    pub windy_music: bool,
    // — the M6-AUDIO-2 inputs (`Main.cs:11076-11350`) —
    /// `SceneMetrics.ActiveMusicBox`: the box id playing, -1 = none.
    pub music_box: i32,
    /// `SceneMetrics.MusicBoxSilence` — the silent box cuts the music.
    pub music_box_silence: bool,
    // — the M6-MUSIC-biome-tracks inputs (`:12686-12920`), ladder order —
    /// The space band: `num5 < 1f` with
    /// `num5 = (centerTileY - (65 + 10*(maxTilesX/4200)^2)) / (worldSurface/5)`
    /// (`:12533-12534`).
    pub space_band: bool,
    /// The Lihzahd temple wall behind the scene centre (`:12690`, wall 87).
    pub temple_wall: bool,
    /// `SceneMetrics.ZoneDungeon` (`:12694`).
    pub dungeon: bool,
    /// The mushroom background: `bgStyle == 9` on the surface band
    /// (`:12700`; `undergroundBackground` is pinned 0 in the port).
    pub mushroom_bg: bool,
    /// `SceneMetrics.ZoneCorrupt` (`:12702`).
    pub corrupt: bool,
    /// `SceneMetrics.ZoneCrimson` (`:12731`).
    pub crimson: bool,
    /// The corrupt/crimson split's blood count lead (`:12704`).
    pub blood_leads_evil: bool,
    /// `SceneMetrics.ZoneMeteor` (`:12744`).
    pub meteor: bool,
    /// `SceneMetrics.ZoneGraveyard` (`:12746`).
    pub graveyard: bool,
    /// `SceneMetrics.ZoneDesert` (`:12750`).
    pub desert: bool,
    /// `SceneMetrics.ZoneJungle` (`:12752`).
    pub jungle: bool,
    /// `SceneMetrics.ZoneSnow` (`:12792`).
    pub snow: bool,
    /// `SceneMetrics.ZoneHallow` (`:12816/:12875`).
    pub hallow: bool,
    /// `Main._shouldUseStormMusic` (`:12887`).
    pub storm: bool,
    /// `WorldGen.oceanDepths` at the screen centre (`:12896`).
    pub ocean: bool,
    /// The depth bands in pixels: `rockLayer*16 + screenHeight/2`,
    /// `(rockLayer-50)*16 + screenHeight/2` (the jungle's hysteresis),
    /// `worldSurface*16 + screenHeight/2` — plus the scene centre Y.
    pub center_y: f32,
    pub surface_band_px: f32,
    pub rock_band_px: f32,
    pub jungle_deep_band_px: f32,
    /// `Main.cloudAlpha == 0 && !windy` — the town arm's quiet-sky gate.
    pub quiet_sky: bool,
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
        // `Main.cs:11076-11350`: the box chain sits in `UpdateAudio` AFTER
        // `DecideOnNewMusic` returns — the box (or the silence box) wins
        // over every world branch including the bosses.
        self.apply_music_box_override(&env);
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

        // `Main.cs:11527-11600`: the two ambience tracks ride the same pool —
        // the rain loop (28) while ZoneRain and not ZoneSnow, the wind loop
        // (45) while |windSpeed| >= 20/50 and SurfaceAtmospherics; the weather
        // factor scales the system volume and shimmer mutes both.
        if let Some(volume) = ambience_rain_volume(&env) {
            self.update_common_track(true, 28, env.ambient_volume * volume);
        } else {
            self.update_common_track_toward_stopping(
                28,
                env.ambient_volume * ambience_rain_volume(&env).unwrap_or(0.0),
                is_main_track_audible,
            );
        }
        if let Some(volume) = ambience_wind_volume(&env) {
            self.update_common_track(true, 45, env.ambient_volume * volume);
        } else {
            self.update_common_track_toward_stopping(
                45,
                env.ambient_volume * ambience_wind_volume(&env).unwrap_or(0.0),
                is_main_track_audible,
            );
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
            // The boss-flag ladder (`:12560-12620`): the event tracks (DD2
            // 92, the frost/pumpkin moons 97/98, the mech bosses 34/24/57/58,
            // Deerclops 38, the pillars…) sit between King Slime's 93
            // (flag19, `:12596`) and the generic `boss` 5 (flag, `:12611`) —
            // none of those NPCs exist in the port yet, so the effective
            // order is 93 then 5.
            self.new_music = boss;
            return;
        }
        // `:12644`: the eclipse — surface only (`!remixWorld` and the scene
        // centre above `worldSurface * 16 + screenHeight/2`).
        if env.eclipse {
            self.new_music = 27;
            return;
        }
        // `:12668`: the shimmer zone.
        if env.shimmer {
            self.new_music = 91;
            return;
        }
        // `flag23` (`:12199`): `TownNPCCount >= 3 && !ZoneShadowCandle` —
        // the town day 46 / night 47 tracks (`:12670-12678`) with their
        // quiet-sky gates (`cloudAlpha == 0 && !windy`, or the underground
        // view) and the graveyard cut. The port has no town NPCs yet
        // (M4-TOWN).
        let quiet_or_underground = env.quiet_sky || env.underground;
        if env.town_npcs >= 3
            && !env.shadow_candle
            && !env.graveyard
            && quiet_or_underground
            && (!env.blood_moon || env.day)
        {
            self.new_music = if env.day { 46 } else { 47 };
            return;
        }
        // `:12680`: the sandstorm.
        if env.sandstorm {
            self.new_music = 40;
            return;
        }
        // `:12682`: the underworld — `Center.Y > UnderworldLayer * 16`.
        if env.underworld {
            self.new_music = 36;
            return;
        }
        // `:12686`: the space band (`num5 < 1f`).
        if env.space_band {
            self.new_music = if env.day { 42 } else { 15 };
            return;
        }
        // `:12690`: the Lihzahd temple wall (87).
        if env.temple_wall {
            self.new_music = 26;
            return;
        }
        // `:12694`: the dungeon zone.
        if env.dungeon {
            self.new_music = 23;
            return;
        }
        // `:12700`: the mushroom background (the surface band; the port's
        // `undergroundBackground` is pinned 0).
        if env.mushroom_bg {
            self.new_music = 29;
            return;
        }
        // `:12702-12742`: the evil biomes — the corrupt arm splits on the
        // crimson lead, the underground/surface halves pick 33/10/8/16.
        if env.corrupt {
            if env.crimson && env.blood_leads_evil {
                self.new_music = if env.underground { 33 } else { 16 };
            } else if env.underground {
                self.new_music = 10;
            } else {
                self.new_music = 8;
            }
            return;
        }
        if env.crimson {
            self.new_music = if env.underground { 33 } else { 16 };
            return;
        }
        // `:12744-12751`: the meteor and the graveyard.
        if env.meteor {
            self.new_music = 2;
            return;
        }
        if env.graveyard {
            self.new_music = 53;
            return;
        }
        // `:12750` (the UG-desert arm sits constants-off — the sandstone
        // wall sets the port does not carry yet); `:12751`: the desert.
        if env.desert {
            self.new_music = 21;
            return;
        }
        // `:12752-12791`: the jungle — the non-remix arm with the 54 band's
        // 50-tile hysteresis above the rock layer.
        if env.jungle {
            if env.center_y > env.rock_band_px {
                self.new_music = 54;
            } else if self.new_music == 54 && env.center_y > env.jungle_deep_band_px {
                self.new_music = 54;
            } else if env.storm && env.center_y < env.surface_band_px {
                self.new_music = 52;
            } else if env.day {
                self.new_music = 7;
            } else {
                self.new_music = 55;
            }
            return;
        }
        // `:12792-12811`: the snow — underground 20, surface 14.
        if env.snow {
            self.new_music = if env.underground { 20 } else { 14 };
            return;
        }
        if env.underground {
            // `:12816-12821`: the hallow keeps its own underground track.
            if env.hallow {
                self.new_music = 11;
                return;
            }
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
        // `:12875-12894`: the surface fallthrough — the daytime hallow, then
        // the storm pair (blood 2, else the storm track 52).
        if env.day && env.hallow {
            if env.storm {
                self.new_music = 52;
            } else if env.rain {
                self.new_music = 19;
            } else if env.windy_music {
                self.new_music = 44;
            } else {
                self.new_music = 9;
            }
            return;
        }
        if env.storm {
            self.new_music = if env.blood_moon { 2 } else { 52 };
            return;
        }
        // `:12896-12915`: the ocean bands — blood 2, the town pair, else the
        // ocean day 22 / night 43.
        if env.ocean {
            if env.blood_moon {
                self.new_music = 2;
            } else if env.town_npcs >= 3 && !env.shadow_candle {
                self.new_music = if env.day { 46 } else { 47 };
            } else {
                self.new_music = if env.day { 22 } else { 43 };
            }
            return;
        }
        // The surface tail (`:12925-12986`): night — blood moon 2, rain 19,
        // else OverworldNight 3.
        if !env.day {
            self.new_music = if env.blood_moon {
                2
            } else if env.rain {
                19
            } else {
                3 // OverworldNight
            };
            return;
        }
        // Day (`:12936-12966`): the rain morning split, then the 1/18
        // alternates with the windy-day override after the pick.
        if env.rain {
            self.new_music = if env.clock < 10800.0 { 59 } else { 19 };
            return;
        }
        if self.day_music == 0 {
            self.day_music = 1;
        }
        if !self.is_track_playing(1) && !self.is_track_playing(18) {
            self.day_music = if self.next_rand() % 2 == 0 { 1 } else { 18 };
        }
        self.new_music = self.day_music;
        if env.windy_music {
            self.new_music = 44;
        }
    }

    /// The `UpdateAudio` box chain (`Main.cs:11076-11350`) — the override
    /// pass that runs after `decide_new_music` returns; the box (or the
    /// silence box) wins over every world branch including the bosses.
    fn apply_music_box_override(&mut self, env: &MusicEnv) {
        if env.music_box_silence {
            self.new_music = 0;
        } else if env.music_box >= 0 {
            if let Some(track) = music_box_track(env.music_box) {
                self.new_music = track;
            }
        }
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
        let buffer =
            rodio::buffer::SamplesBuffer::new(format.channels, format.sample_rate, samples);
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

/// `UpdateAudio`'s box→track mapping (`Main.cs:11076-11350`) — the 100
/// `ActiveMusicBox == N → newMusic = M` pairs extracted verbatim; `None`
/// for the boxes the source does not map (its chain is exhaustive 0..99,
/// so only out-of-range ids land here).
pub fn music_box_track(box_id: i32) -> Option<u32> {
    const PAIRS: [(i32, u32); 100] = [
        (0, 1),
        (1, 2),
        (2, 3),
        (3, 6),
        (4, 4),
        (5, 5),
        (6, 7),
        (7, 8),
        (8, 10),
        (9, 9),
        (10, 12),
        (11, 11),
        (12, 13),
        (13, 14),
        (14, 15),
        (15, 16),
        (16, 17),
        (17, 18),
        (18, 19),
        (19, 20),
        (20, 21),
        (21, 22),
        (22, 23),
        (23, 24),
        (24, 25),
        (25, 26),
        (26, 27),
        (27, 29),
        (28, 30),
        (29, 31),
        (30, 32),
        (31, 33),
        (32, 38),
        (33, 37),
        (34, 35),
        (35, 36),
        (36, 34),
        (37, 39),
        (38, 40),
        (39, 41),
        (40, 44),
        (41, 48),
        (42, 42),
        (43, 43),
        (44, 46),
        (45, 47),
        (46, 49),
        (47, 51),
        (48, 52),
        (49, 53),
        (50, 54),
        (51, 55),
        (52, 56),
        (53, 57),
        (54, 58),
        (55, 59),
        (56, 60),
        (57, 61),
        (58, 62),
        (59, 63),
        (60, 64),
        (61, 65),
        (62, 66),
        (63, 67),
        (64, 68),
        (65, 69),
        (66, 70),
        (67, 71),
        (68, 72),
        (69, 73),
        (70, 74),
        (71, 75),
        (72, 76),
        (73, 77),
        (74, 78),
        (75, 79),
        (76, 80),
        (77, 81),
        (78, 82),
        (79, 83),
        (80, 84),
        (81, 85),
        (82, 86),
        (83, 87),
        (84, 88),
        (85, 89),
        (86, 90),
        (87, 91),
        (88, 93),
        (89, 96),
        (90, 95),
        (91, 94),
        (92, 97),
        (93, 98),
        (94, 92),
        (95, 99),
        (96, 101),
        (97, 100),
        (98, 104),
        (99, 103),
    ];
    PAIRS
        .iter()
        .find(|(id, _)| *id == box_id)
        .map(|(_, track)| *track)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A headless system (no sinks) — `decide_new_music` only touches the
    /// fade table and the picked ids.
    fn system() -> MusicSystem {
        MusicSystem::new(None)
    }

    fn env(day: bool) -> MusicEnv {
        MusicEnv {
            music_volume: 1.0,
            day,
            cloud_alpha: 0.0,
            zone_rain: false,
            zone_snow: false,
            surface_atmospherics: false,
            wind_speed: 0.0,
            shimmer_alpha: 0.0,
            atmo: 1.0,
            ambient_volume: 1.0,
            underground: false,
            rain: false,
            boss_music: None,
            clock: 27000.0,
            eclipse: false,
            shimmer: false,
            town_npcs: 0,
            shadow_candle: false,
            sandstorm: false,
            underworld: false,
            blood_moon: false,
            windy_music: false,
            music_box: -1,
            music_box_silence: false,
            space_band: false,
            temple_wall: false,
            dungeon: false,
            mushroom_bg: false,
            corrupt: false,
            crimson: false,
            blood_leads_evil: false,
            meteor: false,
            graveyard: false,
            desert: false,
            jungle: false,
            snow: false,
            hallow: false,
            storm: false,
            ocean: false,
            center_y: 0.0,
            surface_band_px: 1000.0,
            rock_band_px: 2000.0,
            jungle_deep_band_px: 1500.0,
            quiet_sky: true,
        }
    }

    #[test]
    fn the_ladder_order_beats_the_surface_tail() {
        // The boss scan outranks everything the port models; the eclipse and
        // the sandstorm cut in front of the underworld and the underground.
        let mut m = system();
        let mut e = env(true);
        e.boss_music = Some(93);
        m.decide_new_music(e);
        assert_eq!(m.new_music, 93);
        let mut m = system();
        let mut e = env(true);
        e.eclipse = true;
        e.sandstorm = true;
        e.underworld = true;
        e.underground = true;
        m.decide_new_music(e);
        assert_eq!(m.new_music, 27);
        let mut m = system();
        let mut e = env(true);
        e.sandstorm = true;
        e.underworld = true;
        e.underground = true;
        m.decide_new_music(e);
        assert_eq!(m.new_music, 40);
        let mut m = system();
        let mut e = env(true);
        e.underworld = true;
        e.underground = true;
        m.decide_new_music(e);
        assert_eq!(m.new_music, 36);
        let mut m = system();
        let mut e = env(true);
        e.underground = true;
        m.decide_new_music(e);
        // The underground pair alternates 4/31 (`Main.cs:12830`'s coin flip).
        assert!(matches!(m.new_music, 4 | 31));
    }

    #[test]
    fn the_surface_tail_blood_rain_and_wind() {
        // Night: blood moon 2 over rain 19 over night 3 (`:12968-12979`).
        let mut m = system();
        let mut e = env(false);
        e.blood_moon = true;
        e.rain = true;
        m.decide_new_music(e);
        assert_eq!(m.new_music, 2);
        let mut m = system();
        let mut e = env(false);
        e.rain = true;
        m.decide_new_music(e);
        assert_eq!(m.new_music, 19);
        let mut m = system();
        m.decide_new_music(env(false));
        assert_eq!(m.new_music, 3);
        // Day rain: the morning split on the clock (`:12940-12947`).
        let mut m = system();
        let mut e = env(true);
        e.rain = true;
        e.clock = 5000.0;
        m.decide_new_music(e);
        assert_eq!(m.new_music, 59);
        e.clock = 20000.0;
        let mut m = system();
        m.decide_new_music(e);
        assert_eq!(m.new_music, 19);
        // The windy-day override rides AFTER the 1/18 pick (`:12961`).
        let mut m = system();
        let mut e = env(true);
        e.windy_music = true;
        m.decide_new_music(e);
        assert_eq!(m.new_music, 44);
    }

    #[test]
    fn the_music_box_overrides_the_whole_ladder() {
        // `Main.cs:11076+`: the box chain sits behind `DecideOnNewMusic`,
        // so a playing box beats the boss/eclipse/biome arms; the silent
        // box cuts the music to 0; the box id 85 → the credits track 89
        // (`SceneMetrics.cs:349`'s CanPlayCreditsRoll trigger).
        let mut m = system();
        let mut e = env(true);
        e.boss_music = Some(93);
        e.music_box = 0;
        m.decide_new_music(e.clone());
        m.apply_music_box_override(&e);
        assert_eq!(m.new_music, 1);
        let mut m = system();
        let mut e = env(true);
        e.eclipse = true;
        e.sandstorm = true;
        e.music_box = 85;
        m.decide_new_music(e.clone());
        m.apply_music_box_override(&e);
        assert_eq!(m.new_music, 89);
        let mut m = system();
        let mut e = env(true);
        e.music_box = 42;
        m.decide_new_music(e.clone());
        m.apply_music_box_override(&e);
        assert_eq!(m.new_music, 42);
        let mut m = system();
        let mut e = env(true);
        e.music_box_silence = true;
        m.decide_new_music(e.clone());
        m.apply_music_box_override(&e);
        assert_eq!(m.new_music, 0);
        // The no-box case leaves the ladder untouched (the day pair
        // alternates 1/18 — `Main.cs:12945`'s coin flip).
        let mut m = system();
        m.decide_new_music(env(true));
        assert!(matches!(m.new_music, 1 | 18));
    }

    #[test]
    fn the_biome_ladder_matches_the_source_order() {
        // The dungeon/mushroom/corrupt/meteor/graveyard/desert run in front
        // of the jungle/snow arms (`:12694-12811`), and the underground gate
        // keeps its hallow arm.
        let mut m = system();
        let mut e = env(true);
        e.dungeon = true;
        e.corrupt = true;
        m.decide_new_music(e);
        assert_eq!(m.new_music, 23);
        let mut m = system();
        let mut e = env(true);
        e.mushroom_bg = true;
        m.decide_new_music(e);
        assert_eq!(m.new_music, 29);
        let mut m = system();
        let mut e = env(true);
        e.corrupt = true;
        e.underground = true;
        m.decide_new_music(e);
        assert_eq!(m.new_music, 10);
        let mut m = system();
        let mut e = env(true);
        e.corrupt = true;
        e.crimson = true;
        e.blood_leads_evil = true;
        m.decide_new_music(e);
        assert_eq!(m.new_music, 16);
        let mut m = system();
        let mut e = env(true);
        e.meteor = true;
        e.graveyard = true;
        m.decide_new_music(e);
        assert_eq!(m.new_music, 2);
        let mut m = system();
        let mut e = env(true);
        e.graveyard = true;
        m.decide_new_music(e);
        assert_eq!(m.new_music, 53);
        let mut m = system();
        let mut e = env(true);
        e.desert = true;
        m.decide_new_music(e);
        assert_eq!(m.new_music, 21);
        // The jungle surface day is 7, the deep band 54; the snow splits
        // 14 surface / 20 underground.
        let mut m = system();
        let mut e = env(true);
        e.jungle = true;
        e.center_y = 500.0;
        m.decide_new_music(e);
        assert_eq!(m.new_music, 7);
        let mut m = system();
        let mut e = env(true);
        e.jungle = true;
        e.center_y = 2500.0;
        m.decide_new_music(e);
        assert_eq!(m.new_music, 54);
        let mut m = system();
        let mut e = env(true);
        e.snow = true;
        m.decide_new_music(e);
        assert_eq!(m.new_music, 14);
        let mut m = system();
        let mut e = env(true);
        e.snow = true;
        e.underground = true;
        m.decide_new_music(e);
        assert_eq!(m.new_music, 20);
        // The hallow keeps the underground gate; the storm pair and the
        // ocean sit behind it (`:12816/:12875-12915`).
        let mut m = system();
        let mut e = env(true);
        e.underground = true;
        e.hallow = true;
        m.decide_new_music(e);
        assert_eq!(m.new_music, 11);
        let mut m = system();
        let mut e = env(true);
        e.storm = true;
        m.decide_new_music(e);
        assert_eq!(m.new_music, 52);
        let mut m = system();
        let mut e = env(true);
        e.ocean = true;
        m.decide_new_music(e);
        assert_eq!(m.new_music, 22);
        let mut m = system();
        let mut e = env(false);
        e.ocean = true;
        m.decide_new_music(e);
        assert_eq!(m.new_music, 43);
    }

    #[test]
    fn the_town_gate_needs_three_npcs_and_no_shadow_candle() {
        // `flag23` (`:12199`): TownNPCCount >= 3 && !ZoneShadowCandle; the
        // blood-moon night keeps the regular tail (`:12674`'s gates).
        let mut m = system();
        let mut e = env(true);
        e.town_npcs = 3;
        m.decide_new_music(e);
        assert_eq!(m.new_music, 46);
        let mut m = system();
        let mut e = env(false);
        e.town_npcs = 3;
        m.decide_new_music(e);
        assert_eq!(m.new_music, 47);
        let mut m = system();
        let mut e = env(true);
        e.town_npcs = 3;
        e.shadow_candle = true;
        m.decide_new_music(e);
        assert_ne!(m.new_music, 46);
    }

    #[test]
    fn dw_2_1_item4_cue_maps_to_the_item_legacy_track() {
        // `SoundID.Item4 = new LegacySoundStyle(2, 4)` (`SoundID.cs:1387`) —
        // the dial arm's cue (`Player.cs:32642`/:32652), the same legacy
        // Item track as `Item_1`.
        assert_eq!(sound_type("Item_4"), 2);
    }

    #[test]
    fn the_cue_table_carries_the_legacy_ids() {
        // `CraftingEffects.cs:51` — `SoundEngine.PlaySound(7)`, `SoundID.Grab`
        // (`SoundID.cs`); the grant sound rides the legacy type 7.
        assert_eq!(sound_type("Grab"), 7);
        // Every cue's asset name is its table name (`Sounds/{name}.xnb`).
        assert!(SFX_CUES.iter().any(|(n, _)| *n == "Grab"));
    }
}

/// `Main.cs:11528-11546`: the rain track's weather factor — `Some` while the
/// rain zone is active and snow isn't.
fn ambience_rain_volume(env: &MusicEnv) -> Option<f32> {
    if !(env.zone_rain && !env.zone_snow) {
        return None;
    }
    let num10 = 0.5;
    let mut num11 = env.cloud_alpha / 9.0 * 10.0 * num10 + (1.0 - num10);
    if env.shimmer_alpha > 0.0 {
        num11 *= 1.0 - env.shimmer_alpha;
    }
    let v = (num11.min(1.0)) * env.atmo.powi(4);
    Some(v)
}

/// `Main.cs:11562-11595`: the wind track's weather factor — `Some` while
/// `|windSpeedCurrent| >= 20/50` and `SurfaceAtmospherics`.
fn ambience_wind_volume(env: &MusicEnv) -> Option<f32> {
    let num8 = 20.0f32;
    if env.wind_speed.abs() < num8 / 50.0 || !env.surface_atmospherics {
        return None;
    }
    let num6 = 0.7;
    let mut num7 = env.wind_speed.abs() * num6 + (1.0 - num6);
    if env.shimmer_alpha > 0.0 {
        num7 *= 1.0 - env.shimmer_alpha;
    }
    let v = (num7.min(1.0)) * 0.9 * env.atmo.powi(4);
    Some(v)
}
