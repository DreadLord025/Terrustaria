//! Audio: original PCM SFX via `SoundEffectReader`, MS-ADPCM music from XACT banks.

use std::collections::HashMap;
use std::path::Path;

use rodio::Source;
use crate::content::{decode_msadpcm};

pub struct Audio {
    _stream: Option<rodio::OutputStream>,
    handle: Option<rodio::OutputStreamHandle>,
    cues: HashMap<&'static str, (u16, u32, Vec<i16>)>,
}

/// Cues used by gameplay events (`SoundID` names).
const SFX_CUES: &[&str] = &[
    "Dig_0",
    "Dig_1",
    "Dig_2",
    "Item_1",
    "Coin_0",
    "Player_Hit_0",
    "NPC_Hit_1",
    "Double_Jump",
];

impl Audio {
    pub fn new(content: Option<&Path>) -> Self {
        let mut cues = HashMap::new();
        let mut stream = None;
        let mut handle = None;
        if let Some(dir) = content.map(|c| c.join("Sounds")) {
            match rodio::OutputStream::try_default() {
                Ok((s, h)) => {
                    stream = Some(s);
                    handle = Some(h);
                    // Initialize music system with the same handle
                    audio::MusicSystem::set_handle_from_system(h);
                }
                Err(e) => log::warn!("audio: no output device ({e})"),
            }
            for name in SFX_CUES {
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
        }
    }

    pub fn play(&self, cue: &str) {
        let Some((channels, rate, samples)) = self.cues.get(cue) else {
            return;
        };
        let Some(handle) = &self.handle else { return };
        let buf = rodio::buffer::SamplesBuffer::new(*channels, *rate, samples.clone());
        let _ = handle.play_raw(buf.convert_samples());
    }
}

fn load_sound(path: &Path) -> anyhow::Result<(u16, u32, Vec<u8>)> {
    use crate::content::load_sound as tr_load_sound;
    let sound = tr_load_sound(path)?;
    Ok((sound.channels, sound.sample_rate, sound.data))
}

/// Track index mapper for Terraria music selection logic.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TrackIndex {
    None = 0,
    OverworldDay = 1,
    EerieNight = 2,
    Underground = 4,
    Boss1KingSlime = 5,
}

impl TrackIndex {
    pub fn from_env(day: bool, underground: bool, boss: bool) -> Self {
        if boss {
            Self::Boss1KingSlime
        } else if underground {
            Self::Underground
        } else if day {
            Self::OverworldDay
        } else {
            Self::EerieNight
        }
    }
}

/// Global state for music playback handle initialization
struct MusicState {
    current_handle: Option<rodio::OutputStreamHandle>,
}
static mut MUSIC_STATE: MusicState = MusicState {
    current_handle: None,
};

/// MS-ADPCM music player using XACT wave bank.
pub struct MusicSystem {
    next: Option<u32>,
    last_track: Option<u32>,
    timer: f32,
    wavebank_path: Option<std::path::PathBuf>,
}

impl MusicSystem {
    pub fn new(content_dir: Option<&std::path::Path>) -> Self {
        let wb = content_dir.map(|c| c.join("Wave Bank.xwb"));
        Self {
            next: None,
            last_track: None,
            timer: 0.0,
            wavebank_path: wb,
        }
    }

    /// Set the global audio handle after initialization
    pub fn set_handle_from_system(handle: rodio::OutputStreamHandle) {
        unsafe {
            MUSIC_STATE.current_handle = Some(handle);
        }
    }

    pub fn update(&mut self, dt: f32, day: bool, underground: bool, boss: bool) {
        let want = TrackIndex::from_env(day, underground, boss);
        self.next = Some(want as u32);
        
        self.timer += dt;
        if self.timer < 1.0 { return; }
        self.timer = 0.0;
        
        if self.last_track != self.next {
            let idx = self.next.unwrap_or(0);
            self.play_track(idx);
            self.last_track = self.next;
        }
    }

    fn play_track(&mut self, track_index: u32) {
        use crate::content::{parse_wave_bank_entries, decode_msadpcm};

        let Some(wb_path) = &self.wavebank_path else { return; };
        let Some(handle) = unsafe { MUSIC_STATE.current_handle.as_ref() } else { return };

        let Ok(entries) = parse_wave_bank_entries(wb_path) else {
            log::debug!("music: cannot load wave bank");
            return;
        };

        let Some(entry) = entries.get(track_index as usize) else {
            log::debug!("music: no entry {}", track_index);
            return;
        };

        if entry.data.is_empty() {
            log::debug!("music: empty data for entry {}", track_index);
            return;
        }

        let adpcm_block_align = 48u16;
        let sample_rate = 44100u32;
        let channels = 2u16;
        
        let pcm_data = decode_msadpcm(
            &entry.data[entry.play_offset as usize..(entry.play_offset + entry.play_length) as usize],
            sample_rate,
            channels,
            adpcm_block_align,
        );

        match pcm_data {
            Ok(samples) => {
                let buf = rodio::buffer::SamplesBuffer::new(channels, sample_rate, samples);
                let source = rodio::Source::taking(buf, move || drop(buf));
                let _id = handle.play_raw(source);
                log::info!("music playing: track {}", track_index);
            }
            Err(e) => log::warn!("music decode failed: {}", e),
        }
    }
}
