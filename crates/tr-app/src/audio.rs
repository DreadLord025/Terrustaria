//! Audio: sound effects from the original `Sounds/*.xnb` PCM cues and the music
//! selector (`Main.newMusic`). Music itself lives inside the XACT wave bank as WMA;
//! no WMA decoder is available here yet, so the selector tracks and logs the desired
//! track (see docs/ROADMAP.md).

use std::collections::HashMap;
use std::path::Path;

use rodio::Source;

pub struct Audio {
    _stream: Option<rodio::OutputStream>,
    handle: Option<rodio::OutputStreamHandle>,
    cues: HashMap<&'static str, (u16, u32, Vec<i16>)>,
}

/// Cues used by the current gameplay events (`SoundID` names).
const CUES: &[&str] = &[
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
                }
                Err(e) => log::warn!("audio: no output device ({e})"),
            }
            for name in CUES {
                let path = dir.join(format!("{name}.xnb"));
                match tr_content::load_sound(&path) {
                    Ok(s) if s.format_tag == 1 && s.bits_per_sample == 16 => {
                        let samples = s
                            .data
                            .chunks_exact(2)
                            .map(|b| i16::from_le_bytes([b[0], b[1]]))
                            .collect();
                        cues.insert(*name, (s.channels, s.sample_rate, samples));
                    }
                    Ok(s) => log::warn!(
                        "sound {name}: unsupported format {} ({} bit)",
                        s.format_tag,
                        s.bits_per_sample
                    ),
                    Err(e) => log::debug!("sound {name}: {e:#}"),
                }
            }
        }
        log::info!("audio: {} cues loaded", cues.len());
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

/// `Main.newMusic` selection: boss > underground > day/night overworld.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Track {
    OverworldDay,
    OverworldNight,
    Underground,
    Boss,
}

pub struct Music {
    current: Option<Track>,
    timer: f32,
}

impl Music {
    pub fn new() -> Self {
        Self {
            current: None,
            timer: 0.0,
        }
    }

    pub fn update(&mut self, dt: f32, day: bool, underground: bool, boss: bool) {
        self.timer += dt;
        if self.timer < 1.0 {
            return;
        }
        self.timer = 0.0;
        let want = if boss {
            Track::Boss
        } else if underground {
            Track::Underground
        } else if day {
            Track::OverworldDay
        } else {
            Track::OverworldNight
        };
        if self.current != Some(want) {
            self.current = Some(want);
            log::info!(
                "music: {want:?} (the XACT wave bank is WMA; playback lands with the WMA decoder)"
            );
        }
    }
}
