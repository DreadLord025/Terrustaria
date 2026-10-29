//! MS-ADPCM decoder for XACT-compatible music tracks.
//!
//! Port of the Microsoft ADPCM (MS-ADPCM) block format used by XACT. The layout
//! matches libsndfile / FFmpeg's `adpcm_ms_expand_nibble` and the DirectX
//! reference decoder:
//!
//! * A block starts with a **field-interleaved** header: every channel's
//!   predictor byte, then every channel's `idelta` (i16), then `sample1` (i16),
//!   then `sample2` (i16). For stereo the header is
//!   `pL pR | dL dR | s1L s1R | s2L s2R` (14 bytes), *not* 7 contiguous bytes
//!   per channel.
//! * The first two output frames are `sample2` then `sample1` (oldest first).
//! * The rest of the block is 4-bit nibbles: for stereo each byte carries the
//!   left channel in the high nibble and the right in the low nibble; for mono
//!   the high then low nibble are two consecutive samples.
//! * `sample = (sample1*coeff1 + sample2*coeff2) >> 8 + nibble*delta`, then
//!   `delta = max(16, delta*AdaptationTable[nibble]/256)`.

pub struct AdpcmBlockInfo {
    /// Bytes per block.
    pub block_size: u32,
    /// Number of channels: 1=mono, 2=stereo.
    pub channels: u16,
}

/// Decode MS-ADPCM data into interleaved signed 16-bit PCM samples.
///
/// `sample_rate` is not needed to decode (it is carried so callers can hand the
/// same format through to the audio backend). `block_align` is the full block
/// size in bytes, including the header.
pub fn decode_msadpcm(
    adpcm_data: &[u8],
    _sample_rate: u32,
    num_channels: u16,
    block_align: u16,
) -> Result<Vec<i16>, AdpcmError> {
    let block = block_align as usize;
    if block == 0 || adpcm_data.len() < block {
        return Err(AdpcmError::TooShort);
    }

    let mut out = Vec::new();
    match num_channels {
        1 => decode_mono(adpcm_data, block, &mut out)?,
        2 => decode_stereo(adpcm_data, block, &mut out)?,
        _ => return Err(AdpcmError::UnsupportedChannels(num_channels)),
    }
    Ok(out)
}

fn decode_mono(data: &[u8], block: usize, out: &mut Vec<i16>) -> Result<(), AdpcmError> {
    // predictor (1) + idelta (2) + sample1 (2) + sample2 (2)
    if block < 7 {
        return Err(AdpcmError::TooShort);
    }
    let mut p = 0usize;
    while p + block <= data.len() {
        let predictor = data[p] as usize;
        if predictor > 6 {
            return Err(AdpcmError::BadPredictor(data[p]));
        }
        let mut delta = i16::from_le_bytes([data[p + 1], data[p + 2]]);
        let mut s1 = i16::from_le_bytes([data[p + 3], data[p + 4]]);
        let mut s2 = i16::from_le_bytes([data[p + 5], data[p + 6]]);
        p += 7;

        out.push(s2);
        out.push(s1);

        for _ in 7..block {
            let byte = data[p];
            p += 1;
            for nibble in [byte >> 4, byte & 0x0f] {
                let sample = calc_single(nibble, predictor, s1, s2, delta);
                s2 = s1;
                s1 = sample;
                delta = update_delta(nibble, delta);
                out.push(sample);
            }
        }
    }
    Ok(())
}

fn decode_stereo(data: &[u8], block: usize, out: &mut Vec<i16>) -> Result<(), AdpcmError> {
    // predictors (2) + ideltas (4) + sample1s (4) + sample2s (4)
    if block < 14 {
        return Err(AdpcmError::TooShort);
    }
    let mut p = 0usize;
    while p + block <= data.len() {
        let l_pred = data[p] as usize;
        let r_pred = data[p + 1] as usize;
        if l_pred > 6 {
            return Err(AdpcmError::BadPredictor(data[p]));
        }
        if r_pred > 6 {
            return Err(AdpcmError::BadPredictor(data[p + 1]));
        }
        let mut l_delta = i16::from_le_bytes([data[p + 2], data[p + 3]]);
        let mut r_delta = i16::from_le_bytes([data[p + 4], data[p + 5]]);
        let mut l_s1 = i16::from_le_bytes([data[p + 6], data[p + 7]]);
        let mut r_s1 = i16::from_le_bytes([data[p + 8], data[p + 9]]);
        let mut l_s2 = i16::from_le_bytes([data[p + 10], data[p + 11]]);
        let mut r_s2 = i16::from_le_bytes([data[p + 12], data[p + 13]]);
        p += 14;

        // The two seed frames, oldest first.
        out.push(l_s2);
        out.push(r_s2);
        out.push(l_s1);
        out.push(r_s1);

        for _ in 14..block {
            let byte = data[p];
            p += 1;

            let l_nibble = byte >> 4;
            let r_nibble = byte & 0x0f;

            let l_sample = calc_single(l_nibble, l_pred, l_s1, l_s2, l_delta);
            l_s2 = l_s1;
            l_s1 = l_sample;
            l_delta = update_delta(l_nibble, l_delta);

            let r_sample = calc_single(r_nibble, r_pred, r_s1, r_s2, r_delta);
            r_s2 = r_s1;
            r_s1 = r_sample;
            r_delta = update_delta(r_nibble, r_delta);

            out.push(l_sample);
            out.push(r_sample);
        }
    }
    Ok(())
}

#[derive(Debug)]
pub enum AdpcmError {
    TooShort,
    UnsupportedChannels(u16),
    BadPredictor(u8),
}

impl std::fmt::Display for AdpcmError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AdpcmError::TooShort => write!(f, "adpcm data too short"),
            AdpcmError::UnsupportedChannels(ch) => write!(f, "unsupported channels: {}", ch),
            AdpcmError::BadPredictor(p) => write!(f, "invalid block predictor: {}", p),
        }
    }
}

impl std::error::Error for AdpcmError {}

const ADAPTATION_TABLE: [i32; 16] = [
    230, 230, 230, 230, 307, 409, 512, 614, 768, 614, 512, 409, 307, 230, 230, 230,
];

const COEFF1: [i32; 7] = [256, 512, 0, 192, 240, 460, 392];
const COEFF2: [i32; 7] = [0, -256, 0, 64, 0, -208, -232];

/// `adpcm_ms_expand_nibble`: predict the next sample from the two previous ones,
/// add the signed 4-bit nibble scaled by the current delta, and clamp to i16.
fn calc_single(nibble: u8, predictor: usize, s1: i16, s2: i16, delta: i16) -> i16 {
    let signed = if nibble & 0x08 != 0 {
        i32::from(nibble) - 0x10
    } else {
        i32::from(nibble)
    };
    // Arithmetic shift, like `msadpcm_decode_block`: `/ 256` would round toward
    // zero and differ from the reference decoder by 1 LSB on negative predictions.
    let sample = ((i32::from(s1) * COEFF1[predictor] + i32::from(s2) * COEFF2[predictor]) >> 8)
        + signed * i32::from(delta);
    clamp_i16(sample)
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

fn update_delta(nibble: u8, delta: i16) -> i16 {
    let idx = nibble as usize;
    if idx > 15 {
        return delta;
    }
    let new_delta = (delta as i32 * ADAPTATION_TABLE[idx] / 256) as i16;
    if new_delta < 16 { 16 } else { new_delta }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::xact::{XactSoundBank, XactWaveBank};

    fn retail_track(id: u32) -> Option<(Vec<i16>, u32, u16)> {
        let dir = std::env::var_os("TR_CONTENT").map(std::path::PathBuf::from)?;
        let wb_path = dir.join("Wave Bank.xwb");
        if !wb_path.exists() {
            return None;
        }
        let bank = XactWaveBank::open(&wb_path).expect("wave bank");
        let sound = XactSoundBank::load(&dir.join("Sound Bank.xsb")).expect("sound bank");
        let track = sound.music_track(id).expect("cue") as usize;
        let entry = bank.entries[track].clone();
        let data = bank.read_data(&wb_path, track).expect("entry data");
        let samples = decode_msadpcm(
            &data,
            entry.format.sample_rate,
            entry.format.channels,
            entry.format.block_align() as u16,
        )
        .expect("decode");
        Some((samples, entry.format.sample_rate, entry.format.channels))
    }

    /// Builds one 140-byte stereo block. The header is field-interleaved:
    /// `pL pR | dL dR | s1L s1R | s2L s2R`; the rest is 126 packed nibble bytes.
    fn stereo_block(p: (u8, u8), d: (i16, i16), s1: (i16, i16), s2: (i16, i16)) -> Vec<u8> {
        let mut block = vec![0u8; 140];
        block[0] = p.0;
        block[1] = p.1;
        block[2..4].copy_from_slice(&d.0.to_le_bytes());
        block[4..6].copy_from_slice(&d.1.to_le_bytes());
        block[6..8].copy_from_slice(&s1.0.to_le_bytes());
        block[8..10].copy_from_slice(&s1.1.to_le_bytes());
        block[10..12].copy_from_slice(&s2.0.to_le_bytes());
        block[12..14].copy_from_slice(&s2.1.to_le_bytes());
        block
    }

    /// With predictor 0 and zero nibbles every decoded sample repeats `sample1`.
    /// The distinct seeds make the header field order and the `sample2, sample1`
    /// output order visible; reading seven bytes per channel (the old layout) gives
    /// a completely different sequence.
    #[test]
    fn stereo_block_header_is_field_interleaved() {
        let data = stereo_block((0, 0), (16, 16), (3000, 4000), (1000, 2000));
        let samples = decode_msadpcm(&data, 44100, 2, 140).expect("decode");
        assert_eq!(samples.len(), 128 * 2);
        assert_eq!(
            &samples[..8],
            &[1000, 2000, 3000, 4000, 3000, 4000, 3000, 4000]
        );
        assert_eq!(&samples[samples.len() - 2..], &[3000, 4000]);
    }

    /// The retail music is local-only; when `TR_CONTENT` points at it this decodes
    /// `Music_1` and checks the result is smooth audio rather than byte noise.
    #[test]
    fn decodes_retail_music_to_audio() {
        let Some((samples, sample_rate, channels)) = retail_track(1) else {
            eprintln!("TR_CONTENT unset; skipping");
            return;
        };
        assert_eq!(channels, 2);

        // Stereo interleaved, so one frame is two samples.
        let frames = samples.len() / 2;
        assert!(frames > sample_rate as usize, "over a second");
        let rms = (samples.iter().map(|s| (f64::from(*s)).powi(2)).sum::<f64>()
            / samples.len() as f64)
            .sqrt();
        assert!(rms > 200.0 && rms < 25000.0, "rms {rms}");
        assert!(samples.iter().any(|&s| s != samples[0]), "constant output");
        // A garbage decode would clip constantly; real music rarely does.
        let clipped = samples.iter().filter(|s| s.abs() == i16::MAX).count();
        assert!(
            clipped * 20 < samples.len(),
            "clipped {clipped}/{}",
            samples.len()
        );
    }

    /// Music is heavily oversampled, so consecutive samples *within one channel*
    /// are close. A wrong block header (e.g. reading the predictor fields
    /// per-channel instead of field-interleaved) seeds every 128-frame block with
    /// garbage and makes the differences across each boundary explode.
    #[test]
    fn decoded_music_is_smooth_across_block_boundaries() {
        let Some((samples, _, channels)) = retail_track(1) else {
            eprintln!("TR_CONTENT unset; skipping");
            return;
        };
        assert_eq!(channels, 2);

        // Deinterleave first: an L→R step is not a sample-to-sample step and makes
        // even a perfect stereo decode look noisy.
        let left: Vec<i16> = samples.iter().step_by(2).copied().collect();
        let right: Vec<i16> = samples.iter().skip(1).step_by(2).copied().collect();
        assert!(left.len() >= 128 * 8, "several blocks");

        for (name, x) in [("left", &left), ("right", &right)] {
            let rms =
                (x.iter().map(|s| f64::from(*s).powi(2)).sum::<f64>() / x.len() as f64).sqrt();
            let diff_rms = (x
                .windows(2)
                .map(|w| f64::from(i32::from(w[1]) - i32::from(w[0])).powi(2))
                .sum::<f64>()
                / (x.len() - 1) as f64)
                .sqrt();
            // Healthy 44.1 kHz music: the sample-to-sample change is a small
            // fraction of the signal level. A corrupt decode is close to the RMS.
            assert!(
                diff_rms < 0.5 * rms,
                "{name} channel not smooth: diff_rms {diff_rms}, rms {rms}"
            );
        }

        // Each block header carries the two samples *preceding* its nibble data, so
        // `x[128k + 1]` (`sample1`) must continue from the previous block's last
        // sample. With the fields read in the wrong order every boundary is a jump.
        for (name, x) in [("left", &left), ("right", &right)] {
            let blocks = x.len() / 128;
            let boundary_mean = (1..blocks)
                .map(|k| f64::from(i32::from(x[128 * k + 1]) - i32::from(x[128 * k - 1])).abs())
                .sum::<f64>()
                / (blocks - 1) as f64;
            let interior_steps: Vec<f64> = (1..x.len() - 1)
                .filter(|i| i % 128 > 1)
                .map(|i| f64::from(i32::from(x[i + 1]) - i32::from(x[i])).abs())
                .collect();
            let interior_mean = interior_steps.iter().sum::<f64>() / interior_steps.len() as f64;
            assert!(
                boundary_mean < 4.0 * interior_mean + 16.0,
                "{name} block boundaries do not line up: boundary {boundary_mean}, \
                 interior {interior_mean}"
            );
        }
    }
}
