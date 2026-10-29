//! XACT sound bank (`SDBK`) and wave bank parsing for Terraria's audio system.
//!
//! Layout (MonoGame/QuickWaveBank sources):
//! - Sound Bank: `SDBK` magic, cue tables + names, references to track index in wave bank
//! - Wave Bank: `WBND` container with segment table, metadata entries + PCM data
//! - Music uses MS-ADPCM compression at 48-byte block alignment, stereo, 44100 Hz

use std::path::Path;
use anyhow::{Context, Result, bail};

#[derive(Debug)]
pub struct XactSoundBank {
    /// Cue name → (wave_bank_index, track_index) mapping.
    pub cues: Vec<(String, u16, u32)>, // name, wb_idx, track_idx
}

#[derive(Debug)]
pub struct XactWaveBankEntry {
    /// Flags (bit 0 = compact, etc.).
    pub flags: u32,
    /// Format field encoding codec, channels, sample rate, block align.
    pub format: u32,
    /// Play region start offset from segment base.
    pub play_offset: u32,
    /// Play region length in bytes.
    pub play_length: u32,
    /// Duration (milliseconds).
    pub duration: u32,
    /// Raw PCM/ADPCM data.
    pub data: Vec<u8>,
}

impl XactSoundBank {
    pub fn load(path: &Path) -> Result<Self> {
        let raw = std::fs::read(path)?;
        if &raw[0..4] != b"SDBK" {
            bail!("not a sound bank");
        }

        let mut o = 4usize;
        // skip toolVersion(2), formatVersion(2), crc(2), lastModified(8), platform(1)
        o += 4 + 8 + 1;
        let simple_count = u16::from_le_bytes([raw[o], raw[o + 1]]) as usize;
        let complex_count = u16::from_le_bytes([raw[o + 2], raw[o + 3]]) as usize;
        o += 4; // total cues + unk
        let _unk = u16::from_le_bytes([raw[o], raw[o + 1]]) as usize;
        o += 2;
        let num_wbanks = raw[o];
        o += 1;
        let num_sounds = u16::from_le_bytes([raw[o], raw[o + 1]]);
        o += 2;
        let cue_name_len = u16::from_le_bytes([raw[o], raw[o + 1]]) as usize;
        o += 2;

        // offsets
        let [simple_off, complex_off, names_off, _, var_off, _, wb_names_off, _, _, sounds_off]:
            [u32; 10] = raw[o..o + 40].try_into().unwrap();
        o += 40;

        let names_start = names_off as usize;
        let names_end = names_start + cue_name_len;
        let names_raw: &[u8] = &raw[names_start..names_end.min(raw.len())];
        let cue_names: Vec<&[u8]> = names_raw.split(|&b| b == 0).collect();
        let cue_names_str: Vec<String> = cue_names.iter()
            .filter_map(|n| if n.is_empty() { None } else { String::from_utf8_lossy(n).into() })
            .take(simple_count + complex_count).collect();

        let mut cues = Vec::with_capacity(cue_names_str.len());
        let wb_base = wb_names_off as usize;

        if simple_count > 0 {
            let base = simple_off as usize;
            for i in 0..simple_count {
                if base + 5 > raw.len() { break; }
                let so = u32::from_le_bytes([raw[base + i*5 + 1], raw[base+i*5+2], raw[base+i*5+3], raw[base+i*5+4]]);
                let so = so as usize;
                if so + 18 > raw.len() { continue; }
                let track = u16::from_le_bytes([raw[so + 9], raw[so + 10]]);
                let wb = raw[so + 11];
                let name = cue_names_str.get(i).map(|s| s.clone()).unwrap_or_else(|| format!("Music_{i}"));
                cues.push((name, wb, track.into()));
            }
        }

        Ok(Self { cues })
    }

    pub fn get_cue(&self, name: &str) -> Option<(u16, u32)> {
        self.cues.iter().find(|(n,_,_)| n == name).map(|(_,wb,t)| (*wb,*t))
    }
}

pub fn parse_sound_bank(path: &Path) -> Result<XactSoundBank> {
    XactSoundBank::load(path).with_context(|| format!("parsing {:?}", path))
}

/// Load and parse wave bank entry count + basic metadata + data.
pub fn parse_wave_bank_entries(path: &Path) -> Result<Vec<XactWaveBankEntry>> {
    let raw = std::fs::read(path)?;
    if &raw[0..4] != b"WBND" {
        bail!("not a wave bank");
    }
    
    // Read segment table (10 integers after header)
    let seg_off_start = 16usize;
    let seg_lens_start = 32usize;
    
    // Segment 0: header/metadata, Segment 4: play region (PCM data)
    let play_segment = ((raw[seg_off_start+8] as u32) |
                       ((raw[seg_off_start+9] as u32)<<8) |
                       ((raw[seg_off_start+10] as u32)<<16) |
                       ((raw[seg_off_start+11] as u32)<<24)) as usize;
    
    // Entry metadata segment (segment 1 or 0 depending on layout)
    let meta_segment = ((raw[seg_off_start+0] as u32) |
                        ((raw[seg_off_start+1] as u32)<<8) |
                        ((raw[seg_off_start+2] as u32)<<16) |
                        ((raw[seg_off_start+3] as u32)<<24)) as usize;
    
    // Header: flags(4) + entryCount(4)
    let hdr_start = meta_segment;
    let flags = u32::from_le_bytes([raw[hdr_start+0], raw[hdr_start+1], raw[hdr_start+2], raw[hdr_start+3]]);
    let entry_count = u32::from_le_bytes([raw[hdr_start+4], raw[hdr_start+5], raw[hdr_start+6], raw[hdr_start+7]]) as usize;
    
    // Entry metadata element size (after "Wave Bank" string at offset 64)
    let entry_info = hdr_start + 68;
    let entry_size = u32::from_le_bytes([raw[entry_info], raw[entry_info+1], raw[entry_info+2], raw[entry_info+3]]);
    
    // Metadata entries start after entry_info
    let meta_base = entry_info + 16; // skip alignment padding
    
    let mut entries = Vec::with_capacity(entry_count);
    for i in 0..entry_count {
        let pos = meta_base + i * (entry_size as usize);
        if pos + 24 > raw.len() { break; }
        
        let mut flags_entry = 0u32;
        let mut format = 0u32;
        let mut po = 0u32;
        let mut pl = 0u32;
        
        if entry_size >= 4 {
            flags_entry = u32::from_le_bytes([raw[pos], raw[pos+1], raw[pos+2], raw[pos+3]]);
        }
        if entry_size >= 8 {
            format = u32::from_le_bytes([raw[pos+4], raw[pos+5], raw[pos+6], raw[pos+7]]);
        }
        if entry_size >= 12 {
            po = u32::from_le_bytes([raw[pos+8], raw[pos+9], raw[pos+10], raw[pos+11]]);
        }
        if entry_size >= 16 {
            pl = u32::from_le_bytes([raw[pos+12], raw[pos+13], raw[pos+14], raw[pos+15]]);
        }
        
        // Duration is top 16 bits of flags
        let duration = (flags_entry >> 4) & 0xffff;
        
        entries.push(XactWaveBankEntry {
            flags: flags_entry,
            format,
            play_offset: po,
            play_length: pl,
            duration: duration as u32,
            data: vec![], // filled later
        });
    }
    
    // Fill data by reading from play segment
    for (i, entry) in entries.iter_mut().enumerate() {
        let data_start = play_segment + entry.play_offset as usize;
        let data_len = entry.play_length as usize;
        if data_start + data_len <= raw.len() {
            entry.data = raw[data_start..data_start+data_len].to_vec();
        } else {
            eprintln!("wavebank entry {} truncated", i);
            entry.data = vec![];
        }
    }
    
    Ok(entries)
}

pub fn parse_wave_bank_entry(path: &Path, index: usize) -> Result<Option<XactWaveBankEntry>> {
    let entries = parse_wave_bank_entries(path)?;
    Ok(entries.get(index).cloned())
}