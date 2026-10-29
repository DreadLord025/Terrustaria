//! XACT sound bank (`SDBK`) and wave bank (`WBND`) parsing for Terraria's music.
//!
//! Both layouts are the ones documented for XACT from the DirectX SDK (June 2010)
//! and re-derived/verified against the retail `Sound Bank.xsb` / `Wave Bank.xwb`
//! (see `docs/ANALYSIS.md`, "XACT music banks"). The wave bank is opened in two
//! steps — header + entry metadata first, then the compressed bytes of a single
//! entry on demand — because the retail wave bank is ~495 MB and must not be
//! slurped into memory to play one track.
//!
//! Wave bank (`WAVEBANKHEADER`, `xact3wb.h`):
//! ```text
//! 0x00 "WBND"                   0x04 dwVersion        0x08 dwHeaderVersion
//! 0x0C WAVEBANKREGION[5]        { u32 offset; u32 length; }   // BANKDATA, ENTRYMETADATA,
//!                               //   SEEKTABLES, ENTRYNAMES, ENTRYWAVEDATA
//! ```
//! `BANKDATA` sits at `segments[0].offset`: `u32 flags, u32 entryCount, char[64] name,
//! u32 metaDataElementSize, u32 nameElementSize, u32 alignment, u32 compactFormat,
//! FILETIME buildTime`. Each `ENTRY` (24 bytes) is `u32 flagsAndDuration, u32 format,
//! WAVEBANKREGION playRegion, WAVEBANKREGION loopRegion`.
//!
//! Sound bank (`SDBK`): header counts at unaligned offsets, then the tables they
//! point to. `simpleCues` are 5-byte records (`u8 flags; u32 soundOffset`), the cue
//! name entry table is 6 bytes per cue (`u32 nameOffset; u16 hashNext`), and a simple
//! sound stores its wave as `u16 trackIndex` at `+0x09` and `u8 waveBankIndex` at
//! `+0x0B`.

use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

use anyhow::{bail, Context, Result};

/// `ADPCM_BLOCKALIGN_CONVERSION_OFFSET` (`xwbdump.cpp`): the wave bank stores the
/// ADPCM block alignment as `blockAlign / channels - 22`.
const ADPCM_BLOCKALIGN_CONVERSION_OFFSET: u32 = 22;

/// `WAVEBANKMINIWAVEFORMAT.wFormatTag`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Codec {
    Pcm,
    Xma,
    Adpcm,
    Wma,
}

/// A decoded `WAVEBANKMINIWAVEFORMAT` (`xact3wb.h`).
///
/// ```text
/// bits 0..1   wFormatTag      0 PCM, 1 XMA, 2 MS-ADPCM, 3 WMA
/// bits 2..4   nChannels       channel count (1 - 6)
/// bits 5..22  nSamplesPerSec  sampling rate
/// bits 23..30 wBlockAlign     PCM: block alignment; ADPCM: blockAlign/channels - 22
/// bit  31     wBitsPerSample  8 vs 16 (PCM only)
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MiniFormat {
    pub tag: u32,
    pub channels: u16,
    pub sample_rate: u32,
    pub block_align_field: u32,
    pub bits_field: u32,
}

impl MiniFormat {
    pub fn decode(dw: u32) -> Self {
        Self {
            tag: dw & 0b11,
            channels: ((dw >> 2) & 0b111) as u16,
            sample_rate: (dw >> 5) & 0x3_FFFF,
            block_align_field: (dw >> 23) & 0xFF,
            bits_field: (dw >> 31) & 1,
        }
    }

    pub fn codec(&self) -> Codec {
        match self.tag {
            0 => Codec::Pcm,
            1 => Codec::Xma,
            2 => Codec::Adpcm,
            _ => Codec::Wma,
        }
    }

    /// `MINIWAVEFORMAT::BlockAlign()` (`xwbdump.cpp`).
    pub fn block_align(&self) -> u32 {
        match self.tag {
            0 => self.block_align_field,
            1 => u32::from(self.channels) * 16 / 8,
            2 => (self.block_align_field + ADPCM_BLOCKALIGN_CONVERSION_OFFSET) * u32::from(self.channels),
            _ => {
                const WMA: [u32; 17] = [
                    929, 1487, 1280, 2230, 8917, 8192, 4459, 5945, 2304, 1536, 1485, 1008, 2731,
                    4096, 6827, 5462, 1280,
                ];
                WMA[(self.block_align_field & 0x1F) as usize]
            }
        }
    }

    /// `MINIWAVEFORMAT::AdpcmSamplesPerBlock()`.
    pub fn samples_per_block(&self) -> u32 {
        if self.channels == 0 {
            return 0;
        }
        self.block_align() * 2 / u32::from(self.channels) - 12
    }

    /// `MINIWAVEFORMAT::BitsPerSample()`.
    pub fn bits_per_sample(&self) -> u16 {
        match self.tag {
            1 | 3 => 16,
            2 => 4, // MSADPCM_BITS_PER_SAMPLE
            _ => {
                if self.bits_field == 1 {
                    16
                } else {
                    8
                }
            }
        }
    }
}

/// A single `WAVEBANKENTRY` (24 bytes, non-compact banks).
#[derive(Clone, Debug)]
pub struct XactWaveBankEntry {
    /// `dwFlagsAndDuration` low nibble.
    pub flags: u32,
    /// Decoded `Format`.
    pub format: MiniFormat,
    /// Duration of the wave, in samples (`dwFlagsAndDuration >> 4`).
    pub duration: u32,
    /// `PlayRegion.dwOffset`, relative to the wave-data segment.
    pub play_offset: u32,
    /// `PlayRegion.dwLength`, in bytes.
    pub play_length: u32,
    /// `LoopRegion.dwStartSample`.
    pub loop_start: u32,
    /// `LoopRegion.dwTotalSamples`.
    pub loop_length: u32,
}

impl XactWaveBankEntry {
    /// Number of PCM samples this entry decodes to, from the play region length.
    /// Mirrors `GetDuration()` in `xwbdump.cpp`.
    pub fn sample_count(&self) -> u64 {
        let len = u64::from(self.play_length);
        match self.format.codec() {
            Codec::Adpcm => {
                let ba = u64::from(self.format.block_align());
                let spb = u64::from(self.format.samples_per_block());
                if ba == 0 {
                    return 0;
                }
                let mut duration = (len / ba) * spb;
                let partial = len % ba;
                if partial >= 7 * u64::from(self.format.channels) {
                    duration += partial * 2 / u64::from(self.format.channels) - 12;
                }
                duration
            }
            Codec::Pcm => {
                let bits = u64::from(self.format.bits_per_sample().max(8));
                len * 8 / (bits * u64::from(self.format.channels.max(1)))
            }
            _ => 0,
        }
    }
}

/// A wave bank opened for metadata; entry bytes are read on demand.
#[derive(Clone, Debug)]
pub struct XactWaveBank {
    pub entries: Vec<XactWaveBankEntry>,
    /// Absolute file offset of the wave-data segment (`segments[4].offset`).
    pub data_offset: u64,
}

impl XactWaveBank {
    /// Read the header + entry metadata of a `WBND` file. Does not touch the (huge)
    /// wave-data segment.
    pub fn open(path: &Path) -> Result<Self> {
        let mut file = File::open(path).with_context(|| format!("opening {:?}", path))?;

        let mut head = [0u8; 52];
        file.read_exact(&mut head)
            .context("reading the wave bank header")?;
        if &head[0..4] != b"WBND" {
            bail!("not a wave bank");
        }
        let segments: [(u32, u32); 5] = std::array::from_fn(|i| {
            (
                u32::from_le_bytes(head[12 + i * 8..16 + i * 8].try_into().unwrap()),
                u32::from_le_bytes(head[16 + i * 8..20 + i * 8].try_into().unwrap()),
            )
        });
        let data_offset = u64::from(segments[4].0);

        // BANKDATA.
        let bank_off = u64::from(segments[0].0);
        let mut bank = [0u8; 96];
        file.seek(SeekFrom::Start(bank_off))?;
        file.read_exact(&mut bank)
            .context("reading the wave bank BANKDATA")?;
        let flags = u32::from_le_bytes(bank[0..4].try_into().unwrap());
        let entry_count = u32::from_le_bytes(bank[4..8].try_into().unwrap()) as usize;
        let meta_elem = u32::from_le_bytes(bank[72..76].try_into().unwrap()) as usize;
        if flags & 0x0002_0000 != 0 {
            bail!("compact wave banks are not supported");
        }
        if meta_elem != 24 {
            bail!("unexpected entry metadata element size {meta_elem}");
        }

        // ENTRYMETADATA.
        let meta_off = u64::from(segments[1].0);
        let mut meta = vec![0u8; entry_count * meta_elem];
        file.seek(SeekFrom::Start(meta_off))?;
        file.read_exact(&mut meta)
            .context("reading the wave bank entry metadata")?;

        let mut entries = Vec::with_capacity(entry_count);
        for i in 0..entry_count {
            let e = &meta[i * meta_elem..(i + 1) * meta_elem];
            let flags_and_duration = u32::from_le_bytes(e[0..4].try_into().unwrap());
            let format = MiniFormat::decode(u32::from_le_bytes(e[4..8].try_into().unwrap()));
            let play_offset = u32::from_le_bytes(e[8..12].try_into().unwrap());
            let play_length = u32::from_le_bytes(e[12..16].try_into().unwrap());
            let loop_start = u32::from_le_bytes(e[16..20].try_into().unwrap());
            let loop_length = u32::from_le_bytes(e[20..24].try_into().unwrap());
            entries.push(XactWaveBankEntry {
                flags: flags_and_duration & 0xF,
                format,
                duration: flags_and_duration >> 4,
                play_offset,
                play_length,
                loop_start,
                loop_length,
            });
        }

        Ok(Self {
            entries,
            data_offset,
        })
    }

    /// Read one entry's compressed bytes by seeking to `dataOffset + playOffset`.
    /// Returns an empty vector for an out-of-range index.
    pub fn read_data(&self, path: &Path, index: usize) -> Result<Vec<u8>> {
        let Some(entry) = self.entries.get(index) else {
            return Ok(Vec::new());
        };
        let start = self.data_offset + u64::from(entry.play_offset);
        let mut file = File::open(path).with_context(|| format!("opening {:?}", path))?;
        file.seek(SeekFrom::Start(start))?;
        let mut data = vec![0u8; entry.play_length as usize];
        file.read_exact(&mut data)
            .context("reading the wave bank entry data")?;
        Ok(data)
    }
}

/// A parsed `SDBK`: one row per cue, `(name, wave_bank_index, track_index)`.
#[derive(Clone, Debug, Default)]
pub struct XactSoundBank {
    pub cues: Vec<(String, u16, u32)>,
}

impl XactSoundBank {
    pub fn load(path: &Path) -> Result<Self> {
        let raw = std::fs::read(path).with_context(|| format!("reading {:?}", path))?;
        if raw.len() < 0x4A + 64 || &raw[0..4] != b"SDBK" {
            bail!("not a sound bank");
        }
        let u16_at = |o: usize| -> Result<u16> {
            raw.get(o..o + 2)
                .map(|b| u16::from_le_bytes([b[0], b[1]]))
                .context("sound bank truncated")
        };
        let u32_at = |o: usize| -> Result<u32> {
            raw.get(o..o + 4)
                .map(|b| u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
                .context("sound bank truncated")
        };

        let simple_count = u16_at(0x13)? as usize;
        let simple_off = u32_at(0x22)? as usize;
        let name_entries_off = u32_at(0x42)? as usize;

        let read_cstr = |o: usize| -> String {
            let end = raw[o..].iter().position(|&b| b == 0).unwrap_or(0) + o;
            String::from_utf8_lossy(&raw[o..end]).into_owned()
        };

        let mut cues = Vec::with_capacity(simple_count);
        for i in 0..simple_count {
            let cue = simple_off + i * 5;
            let Some(sound_off) = raw
                .get(cue + 1..cue + 5)
                .map(|b| u32::from_le_bytes([b[0], b[1], b[2], b[3]]) as usize)
            else {
                break;
            };
            let name_off = u32_at(name_entries_off + i * 6)? as usize;
            if name_off >= raw.len() || sound_off + 12 > raw.len() {
                continue;
            }
            let name = read_cstr(name_off);
            let track = u16::from_le_bytes([raw[sound_off + 9], raw[sound_off + 10]]);
            let wave_bank = raw[sound_off + 11];
            cues.push((name, u16::from(wave_bank), u32::from(track)));
        }

        Ok(Self { cues })
    }

    pub fn get_cue(&self, name: &str) -> Option<(u16, u32)> {
        self.cues
            .iter()
            .find(|(n, _, _)| n == name)
            .map(|(_, wb, t)| (*wb, *t))
    }

    /// Wave-bank entry index for `Music_{id}`, the cue `LegacyAudioSystem` loads
    /// (`LoadCue(i, "Music_" + i)`).
    pub fn music_track(&self, id: u32) -> Option<u32> {
        self.get_cue(&format!("Music_{id}")).map(|(_, track)| track)
    }
}

pub fn parse_sound_bank(path: &Path) -> Result<XactSoundBank> {
    XactSoundBank::load(path).with_context(|| format!("parsing {:?}", path))
}

/// Entry metadata of a wave bank (compatibility helper).
pub fn parse_wave_bank_entries(path: &Path) -> Result<Vec<XactWaveBankEntry>> {
    Ok(XactWaveBank::open(path)?.entries)
}

/// One entry's metadata (compatibility helper).
pub fn parse_wave_bank_entry(path: &Path, index: usize) -> Result<Option<XactWaveBankEntry>> {
    Ok(XactWaveBank::open(path)?.entries.get(index).cloned())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn content_dir() -> Option<std::path::PathBuf> {
        let dir = std::env::var_os("TR_CONTENT").map(std::path::PathBuf::from)?;
        let wb = dir.join("Wave Bank.xwb");
        wb.exists().then_some(dir)
    }

    #[test]
    fn mini_format_decodes_the_msadpcm_music_format() {
        // The retail music entries all carry this `format` dword.
        let f = MiniFormat::decode(0x1815_888a);
        assert_eq!(f.codec(), Codec::Adpcm);
        assert_eq!(f.channels, 2);
        assert_eq!(f.sample_rate, 44100);
        // blockAlign = (48 + 22) * 2 = 140, which divides every play length.
        assert_eq!(f.block_align(), 140);
        assert_eq!(f.samples_per_block(), 128);
    }

    /// The retail bank is local-only; this pins the parser against it when present.
    #[test]
    fn reads_the_retail_wave_bank_metadata() {
        let Some(dir) = content_dir() else {
            eprintln!("TR_CONTENT unset; skipping");
            return;
        };
        let bank = XactWaveBank::open(&dir.join("Wave Bank.xwb")).expect("wave bank");
        assert_eq!(bank.entries.len(), 104);
        let e0 = &bank.entries[0];
        assert_eq!(e0.format.block_align(), 140);
        assert_eq!(e0.format.sample_rate, 44100);
        assert_eq!(e0.play_offset, 0);
        // Every play length is a whole number of 140-byte ADPCM blocks.
        for (i, e) in bank.entries.iter().enumerate() {
            assert_eq!(e.play_length % 140, 0, "entry {i} length not block aligned");
        }
        // Entry durations are ~30 s..4 min.
        let secs = e0.sample_count() as f64 / 44100.0;
        assert!(secs > 1.0 && secs < 600.0, "entry 0 is {secs}s");
    }

    #[test]
    fn sound_bank_maps_music_cues_to_wave_entries() {
        let Some(dir) = content_dir() else {
            eprintln!("TR_CONTENT unset; skipping");
            return;
        };
        let bank = XactSoundBank::load(&dir.join("Sound Bank.xsb")).expect("sound bank");
        assert_eq!(bank.cues.len(), 104);
        // The retail cue order is not the track order: Music_1 -> entry 2,
        // Music_2 -> entry 1, Music_3 -> entry 0, Music_4 -> entry 11.
        assert_eq!(bank.music_track(1), Some(2));
        assert_eq!(bank.music_track(2), Some(1));
        assert_eq!(bank.music_track(3), Some(0));
        assert_eq!(bank.music_track(4), Some(11));
        assert_eq!(bank.music_track(104), Some(103));
    }
}
