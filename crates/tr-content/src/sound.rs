//! `SoundEffectReader`: decodes the PCM payload of a `Sounds/*.xnb` cue.
//! Layout (XNA `SoundEffectWriter`): format size + `WAVEFORMATEX`, data size + PCM,
//! loop start/length and duration in milliseconds.

use std::path::Path;

use anyhow::{bail, Result};

use crate::xnb::{ByteReader, XnbFile};

pub struct Sound {
    pub format_tag: u16,
    pub channels: u16,
    pub sample_rate: u32,
    pub bits_per_sample: u16,
    /// Raw PCM bytes (`SoundEffect` data).
    pub data: Vec<u8>,
}

pub fn load_sound(path: impl AsRef<Path>) -> Result<Sound> {
    let raw = std::fs::read(path.as_ref())?;
    let xnb = XnbFile::read(&raw)?;
    if !xnb.asset_reader()?.contains("SoundEffectReader") {
        bail!("{}: not a SoundEffect asset", path.as_ref().display());
    }
    let mut r = ByteReader::new(&xnb.payload);
    let format_size = r.read_i32()?;
    if !(16..=64).contains(&format_size) {
        bail!("bad format size {format_size}");
    }
    let format = r.read_bytes(format_size as usize)?.to_vec();
    let data_size = r.read_i32()?;
    if data_size < 0 {
        bail!("bad data size {data_size}");
    }
    let data = r.read_bytes(data_size as usize)?.to_vec();
    let format_tag = u16::from_le_bytes([format[0], format[1]]);
    let channels = u16::from_le_bytes([format[2], format[3]]);
    let sample_rate = u32::from_le_bytes([format[4], format[5], format[6], format[7]]);
    let bits_per_sample = u16::from_le_bytes([format[14], format[15]]);
    Ok(Sound {
        format_tag,
        channels,
        sample_rate,
        bits_per_sample,
        data,
    })
}
