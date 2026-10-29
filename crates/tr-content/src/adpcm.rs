//! MS-ADPCM decoder for XACT-compatible music tracks.
//!
//! Port of QuickWaveBank's `ADPCMConverter` (MIT/public domain):
//! - 48-byte blocks with predictor/delta/2-sample preamble
//! - Stereo interleaving: L/R nibbles in a single byte
//! - Adaptation table and coefficient vectors from Windows SDK

pub struct AdpcmBlockInfo {
    /// Left channel block size (bytes per stereo block).
    pub block_size: u32,
    /// Number of channels: 1=mono, 2=stereo.
    pub channels: u16,
}

/// Decode MS-ADPCM data into signed 16-bit PCM samples.
///
/// Returns `(samples_per_channel, sample_rate, channels)`.
pub fn decode_msadpcm(
    adpcm_data: &[u8],
    sample_rate: u32,
    num_channels: u16,
    block_align: u16,
) -> Result<Vec<i16>, AdpcmError> {
    if adpcm_data.len() < (block_align as usize) * 2 {
        return Err(AdpcmError::TooShort);
    }

    let mut out = Vec::new();

    match num_channels {
        1 => decode_mono(adpcm_data, block_align, &mut out),
        2 => decode_stereo(adpcm_data, block_align, &mut out),
        _ => Err(AdpcmError::UnsupportedChannels(num_channels)),
    }?;

    Ok(out)
}

fn decode_mono(data: &[u8], block_align: u16, out: &mut Vec<i16>) -> Result<(), AdpcmError> {
    let mut p = 0usize;
    let lim = data.len() - block_align as usize;
    let mut s2: i16;
    let mut s1: i16;
    let mut delta: i16;
    let mut predictor: usize;

    while p <= lim {
        predictor = data[p] as usize;
        delta = i16::from_le_bytes([data[p + 1], data[p + 2]]);
        s2 = i16::from_le_bytes([data[p + 3], data[p + 4]]);
        s1 = i16::from_le_bytes([data[p + 5], data[p + 6]]);
        p += 7;

        out.push(s2);
        out.push(s1);

        // `WAVEFORMATEX` MS-ADPCM: 7-byte preamble, the rest of the block is
        // nibble pairs (two samples per byte).
        let n_bytes = (block_align as usize).saturating_sub(7);
        for _ in 0..n_bytes {
            if p >= data.len() {
                break;
            }
            let byte = data[p];
            p += 1;

            for nibble in [(byte >> 4) as u8, byte & 0x0f] {
                let result = calc_single(mono_byte_to_nibble(nibble), predictor, s1, s2, delta)?;
                // Update state after each nibble
                s2 = s1;
                s1 = result;
                delta = update_delta(nibble, delta);
            }
        }
    }

    Ok(())
}

fn decode_stereo(data: &[u8], block_align: u16, out: &mut Vec<i16>) -> Result<(), AdpcmError> {
    let mut p = 0usize;
    let lim = data.len() - block_align as usize;
    let mut l_s2: i16;
    let mut r_s2: i16;
    let mut l_s1: i16;
    let mut r_s1: i16;
    let mut l_delta: i16;
    let mut r_delta: i16;
    let mut l_predictor: usize;
    let mut r_predictor: usize;

    while p <= lim {
        l_predictor = data[p] as usize;
        r_predictor = data[p + 1] as usize;
        l_delta = i16::from_le_bytes([data[p + 2], data[p + 3]]);
        r_delta = i16::from_le_bytes([data[p + 4], data[p + 5]]);
        l_s2 = i16::from_le_bytes([data[p + 6], data[p + 7]]);
        r_s2 = i16::from_le_bytes([data[p + 8], data[p + 9]]);
        l_s1 = i16::from_le_bytes([data[p + 10], data[p + 11]]);
        r_s1 = i16::from_le_bytes([data[p + 12], data[p + 13]]);
        p += 14;

        out.push(l_s2);
        out.push(r_s2);
        out.push(l_s1);
        out.push(r_s1);

        // Stereo: 14-byte preamble; each remaining byte carries one left and one
        // right nibble (one sample per channel per byte).
        let n_bytes = (block_align as usize).saturating_sub(14);
        for _ in 0..n_bytes {
            if p >= data.len() {
                break;
            }
            let byte = data[p];
            p += 1;

            let l_nibble = (byte >> 4) as u8;
            let r_nibble = byte & 0x0f;

            // Process left channel
            let result_l = calc_single(mono_byte_to_nibble(l_nibble), l_predictor, l_s1, l_s2, l_delta)?;
            l_s2 = l_s1;
            l_s1 = result_l;
            l_delta = update_delta(l_nibble, l_delta);

            // Process right channel
            let result_r = calc_single(mono_byte_to_nibble(r_nibble), r_predictor, r_s1, r_s2, r_delta)?;
            r_s2 = r_s1;
            r_s1 = result_r;
            r_delta = update_delta(r_nibble, r_delta);
        }
    }

    Ok(())
}

#[derive(Debug)]
pub enum AdpcmError {
    TooShort,
    UnsupportedChannels(u16),
    ClampFailure(i32),
}

impl std::fmt::Display for AdpcmError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AdpcmError::TooShort => write!(f, "adpcm data too short"),
            AdpcmError::UnsupportedChannels(ch) => write!(f, "unsupported channels: {}", ch),
            AdpcmError::ClampFailure(v) => write!(f, "clamping failed: {}", v),
        }
    }
}

impl std::error::Error for AdpcmError {}

const ADAPTATION_TABLE: [i32; 16] = [
    230, 230, 230, 230, 307, 409, 512, 614, 768, 614, 512, 409, 307, 230, 230, 230,
];

const COEFF1: [i32; 7] = [256, 512, 0, 192, 240, 460, 392];
const COEFF2: [i32; 7] = [0, -256, 0, 64, 0, -208, -232];

fn calc_single(
    nibble: u8,
    predictor: usize,
    s1: i16,
    s2: i16,
    delta: i16,
) -> Result<i16, AdpcmError> {
    let sb = nibble as i8;
    let signed = if (sb & 0x8) != 0 {
        i32::from(sb.wrapping_sub(0x10))
    } else {
        i32::from(nibble)
    };
    let sample = (i32::from(s1) * COEFF1[predictor] + i32::from(s2) * COEFF2[predictor]) / 256
        + signed * i32::from(delta);
    Ok(clamp_i16(sample))
}

fn clamp_i16(v: i32) -> i16 {
    if v < i16::MIN as i32 {
        i16::MIN
    } else if v > i16::MAX as i32 {
        i16::MAX
    } else {
        v as i16
    }
}

fn mono_byte_to_nibble(byte: u8) -> u8 {
    byte & 0x0f
}

fn update_delta(nibble: u8, delta: i16) -> i16 {
    let idx = nibble as usize;
    if idx > 15 {
        return delta;
    }
    let new_delta = (delta as i32 * ADAPTATION_TABLE[idx] / 256) as i16;
    if new_delta < 16 {
        16
    } else {
        new_delta
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::xact::{XactSoundBank, XactWaveBank};

    /// The retail music is local-only, but when `TR_CONTENT` points at it this
    /// decodes `Music_1` and checks the result is audio rather than byte noise.
    #[test]
    fn decodes_retail_music_to_audio() {
        let Some(dir) = std::env::var_os("TR_CONTENT").map(std::path::PathBuf::from) else {
            eprintln!("TR_CONTENT unset; skipping");
            return;
        };
        let wb_path = dir.join("Wave Bank.xwb");
        if !wb_path.exists() {
            return;
        }
        let bank = XactWaveBank::open(&wb_path).expect("wave bank");
        let sound = XactSoundBank::load(&dir.join("Sound Bank.xsb")).expect("sound bank");
        let track = sound.music_track(1).expect("Music_1") as usize;
        let entry = bank.entries[track].clone();
        let data = bank.read_data(&wb_path, track).expect("entry data");

        let samples = decode_msadpcm(
            &data,
            entry.format.sample_rate,
            entry.format.channels,
            entry.format.block_align() as u16,
        )
        .expect("decode");

        // Stereo interleaved, so one frame is two samples.
        let frames = samples.len() / 2;
        assert!(frames > entry.format.sample_rate as usize, "over a second");
        let rms = (samples
            .iter()
            .map(|s| (f64::from(*s)).powi(2))
            .sum::<f64>()
            / samples.len() as f64)
            .sqrt();
        assert!(rms > 200.0 && rms < 25000.0, "rms {rms}");
        assert!(samples.iter().any(|&s| s != samples[0]), "constant output");
        // A garbage decode would clip constantly; real music rarely does.
        let clipped = samples.iter().filter(|s| s.abs() == i16::MAX).count();
        assert!(clipped * 20 < samples.len(), "clipped {clipped}/{}", samples.len());
    }
}

#[allow(dead_code)]
fn calculate_sample(
    nibble: u8,
    predictor: usize,
    sample_1: &[i16],
    sample_2: &[i16],
    delta: &[i16],
) -> i16 {
    let mut signed = nibble as i8;
    if (signed & 0x8) == 0x8 {
        signed -= 0x10;
    }
    let sample_int =
        (i32::from(sample_1[0]) * COEFF1[predictor] + i32::from(sample_2[0]) * COEFF2[predictor])
            / 256
            + i32::from(signed) * i32::from(delta[0]);
    clamp_i16(sample_int)
}