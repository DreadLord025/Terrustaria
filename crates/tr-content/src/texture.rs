//! `Texture2DReader`: decodes the embedded surface of a `.xnb` texture to RGBA8.
//! Port of the texture half of `tools/XnbExtract/Program.cs`.

use std::path::Path;

use anyhow::{bail, Result};

use crate::dxt;
use crate::xnb::{ByteReader, XnbFile};

#[derive(Clone)]
pub struct RgbaImage {
    pub width: u32,
    pub height: u32,
    /// RGBA8, row-major.
    pub pixels: Vec<u8>,
}

/// Field order of the texture header. `FormatFirst` matches `Texture2DWriter`
/// (format, width, height, mipCount), which the extracted `Images/*.xnb` use.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextureLayout {
    FormatFirst,
    /// format, mipCount, width, height.
    FormatMipsFirst,
    /// width, height, format, mipCount.
    SizeFirst,
    /// format, width, height, mipCount with the pixel bytes written directly,
    /// no per-mip byte count.
    FormatFirstBare,
}

pub fn load_texture(path: impl AsRef<Path>) -> Result<RgbaImage> {
    let raw = std::fs::read(path.as_ref())?;
    let xnb = XnbFile::read(&raw)?;
    if !xnb.asset_reader()?.contains("Texture2DReader") {
        bail!("{}: not a Texture2D asset", path.as_ref().display());
    }
    let mut br = ByteReader::new(&xnb.payload);
    read_texture_with(&mut br, TextureLayout::FormatFirst)
}

/// Reads one `Texture2D` object (`Texture2DReader.Read`). Each mip is preceded by
/// its byte count; only mip 0 is converted.
pub(crate) fn read_texture_with(br: &mut ByteReader, layout: TextureLayout) -> Result<RgbaImage> {
    let (format, width, height, mip_count) = match layout {
        TextureLayout::FormatFirst => {
            let f = br.read_i32()?;
            let w = br.read_i32()?;
            let h = br.read_i32()?;
            let m = br.read_i32()?;
            (f, w, h, m)
        }
        TextureLayout::FormatMipsFirst => {
            let f = br.read_i32()?;
            let m = br.read_i32()?;
            let w = br.read_i32()?;
            let h = br.read_i32()?;
            (f, w, h, m)
        }
        TextureLayout::SizeFirst => {
            let w = br.read_i32()?;
            let h = br.read_i32()?;
            let f = br.read_i32()?;
            let m = br.read_i32()?;
            (f, w, h, m)
        }
        TextureLayout::FormatFirstBare => {
            let f = br.read_i32()?;
            let w = br.read_i32()?;
            let h = br.read_i32()?;
            let m = br.read_i32()?;
            (f, w, h, m)
        }
    };
    let bare = layout == TextureLayout::FormatFirstBare;
    if !(1..=4096).contains(&width) || !(1..=4096).contains(&height) {
        bail!("bad texture size {width}x{height}");
    }
    if !(1..=8).contains(&mip_count) {
        bail!("bad mip count {mip_count}");
    }
    let width = width as u32;
    let height = height as u32;
    let mut pixels: Option<Vec<u8>> = None;
    for mip in 0..mip_count {
        let (size, data) = if bare {
            let n = (width * height * 4) as usize;
            (n as i32, br.read_bytes(n)?)
        } else {
            let size = br.read_i32()?;
            if size < 0 {
                bail!("negative mip size {size}");
            }
            (size, br.read_bytes(size as usize)?)
        };
        let _ = size;
        if mip == 0 {
            pixels = Some(match format {
                // `SurfaceFormat.Color` stores `XNA Color`, i.e. RGBA8 in memory.
                0 => data.to_vec(),
                4 => dxt::decompress_dxt1(data, width as i32, height as i32)?,
                5 => dxt::decompress_dxt3(data, width as i32, height as i32)?,
                6 => dxt::decompress_dxt5(data, width as i32, height as i32)?,
                other => bail!("unsupported surface format {other}"),
            });
        }
    }
    let pixels = pixels.ok_or_else(|| anyhow::anyhow!("no mip 0"))?;
    if pixels.len() < (width * height * 4) as usize {
        bail!("short pixel buffer: {} bytes for {width}x{height}", pixels.len());
    }
    Ok(RgbaImage { width, height, pixels })
}
