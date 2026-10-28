//! ReLogic `DynamicSpriteFontReader`: glyph pages plus kerning tables.
//!
//! The reader in `decompiled-libs/ReLogic` documents the logical structure
//! (spacing, line spacing, default char, pages of texture + glyphs + padding +
//! characters + kerning), but the ReLogic shipped with Terraria 1.4.5.8 writes a
//! slightly different encoding: the character list is a UTF-8 string and the
//! page header carries extra fields, so the primitive offsets differ from the
//! decompiled build. The parser therefore locates the page texture and every
//! table structurally and accepts a layout only when the whole page validates:
//! the mip-0 byte count must match the surface size, every glyph rect must lie
//! inside the page, there must be exactly one kerning entry per character, and
//! the character string must decode as UTF-8.

use std::path::Path;

use anyhow::{bail, Result};

use crate::dxt;
use crate::xnb::XnbFile;

pub use crate::texture::RgbaImage;

#[derive(Clone, Debug)]
pub struct Glyph {
    pub ch: char,
    /// `Rectangle` in the page texture (Glyphs).
    pub src: [i32; 4],
    /// Cropping/padding rectangle (Padding).
    pub pad: [i32; 4],
    /// `Vector3`: horizontal bearing, vertical offset and advance width.
    pub kerning: [f32; 3],
}

pub struct FontPage {
    pub texture: RgbaImage,
    pub glyphs: Vec<Glyph>,
}

pub struct DynamicSpriteFont {
    pub spacing: f32,
    pub line_spacing: i32,
    pub default_char: char,
    pub pages: Vec<FontPage>,
}

pub fn load_font(path: impl AsRef<Path>) -> Result<DynamicSpriteFont> {
    let raw = std::fs::read(path.as_ref())?;
    let xnb = XnbFile::read(&raw)?;
    if !xnb.asset_reader()?.contains("DynamicSpriteFontReader") {
        bail!("{}: not a DynamicSpriteFont asset", path.as_ref().display());
    }
    read_font(&xnb.payload).map_err(|e| anyhow::anyhow!("{}: {e}", path.as_ref().display()))
}

pub(crate) fn read_font(d: &[u8]) -> Result<DynamicSpriteFont> {
    let spacing = read_f32_at(d, 0)?;
    let line_spacing = read_i32_at(d, 4)?;
    if !(0.0..=10.0).contains(&spacing) || !(-10..=400).contains(&line_spacing) {
        bail!("implausible font metrics: spacing {spacing}, line {line_spacing}");
    }

    // Page texture: first offset whose `Texture2D` header validates exactly.
    let mut found = None;
    for start in 8..=96 {
        if let Ok((tex, end)) = texture_at(d, start) {
            found = Some((tex, end, start));
            break;
        }
    }
    let (texture, mut pos, tex_start) = found.ok_or_else(|| anyhow::anyhow!("no page texture"))?;

    let (glyphs, end) = find_rects(d, pos, 20, Some(&texture))?;
    let count = glyphs.len();
    pos = end;
    let (padding, end) = find_rects_exact(d, pos, count)?;
    pos = end;
    let (characters, end) = find_chars(d, pos, count)?;
    pos = end;
    let (kerning, _end) = find_kerning(d, pos, count)?;

    // The default character sits in the small header block before the texture.
    let default_char = d
        .get(8..tex_start)
        .and_then(|b| {
            b.windows(2)
                .find_map(|w| char::from_u32(u16::from_le_bytes([w[0], w[1]]) as u32))
        })
        .filter(|c| !c.is_control())
        .unwrap_or('?');

    let page = FontPage {
        texture,
        glyphs: characters
            .into_iter()
            .enumerate()
            .map(|(i, ch)| Glyph {
                ch,
                src: glyphs[i],
                pad: padding[i],
                kerning: kerning[i],
            })
            .collect(),
    };
    Ok(DynamicSpriteFont {
        spacing,
        line_spacing,
        default_char,
        pages: vec![page],
    })
}

fn read_i32_at(d: &[u8], p: usize) -> Result<i32> {
    d.get(p..p + 4)
        .map(|b| i32::from_le_bytes(b.try_into().unwrap()))
        .ok_or_else(|| anyhow::anyhow!("short read at {p}"))
}

fn read_f32_at(d: &[u8], p: usize) -> Result<f32> {
    Ok(f32::from_bits(read_i32_at(d, p)? as u32))
}

fn expected_surface_bytes(format: i32, w: u32, h: u32) -> Option<usize> {
    let px = w as usize * h as usize;
    match format {
        0 => Some(px * 4),
        4 => Some(px / 2),
        5 | 6 => Some(px),
        _ => None,
    }
}

/// Validates a `Texture2DReader` header at `start` and decodes mip 0.
fn texture_at(d: &[u8], start: usize) -> Result<(RgbaImage, usize)> {
    let format = read_i32_at(d, start)?;
    let width = read_i32_at(d, start + 4)?;
    let height = read_i32_at(d, start + 8)?;
    let mips = read_i32_at(d, start + 12)?;
    if !(8..=4096).contains(&width) || !(8..=4096).contains(&height) {
        bail!("bad size");
    }
    if !(1..=8).contains(&mips) {
        bail!("bad mip count");
    }
    let (width, height) = (width as u32, height as u32);
    let expect = expected_surface_bytes(format, width, height)
        .ok_or_else(|| anyhow::anyhow!("bad format"))?;
    let size = read_i32_at(d, start + 16)?;
    if size as usize != expect {
        bail!("mip size {size} != {expect}");
    }
    let data = d
        .get(start + 20..start + 20 + expect)
        .ok_or_else(|| anyhow::anyhow!("short surface"))?;
    let pixels = match format {
        0 => data.to_vec(),
        4 => dxt::decompress_dxt1(data, width as i32, height as i32)?,
        5 => dxt::decompress_dxt3(data, width as i32, height as i32)?,
        6 => dxt::decompress_dxt5(data, width as i32, height as i32)?,
        _ => unreachable!(),
    };
    Ok((
        RgbaImage {
            width,
            height,
            pixels,
        },
        start + 20 + expect,
    ))
}

fn rects_at(d: &[u8], count_pos: usize, count: usize) -> Result<Vec<[i32; 4]>> {
    let mut out = Vec::with_capacity(count);
    for i in 0..count {
        let p = count_pos + i * 16;
        out.push([
            read_i32_at(d, p)?,
            read_i32_at(d, p + 4)?,
            read_i32_at(d, p + 8)?,
            read_i32_at(d, p + 12)?,
        ]);
    }
    Ok(out)
}

/// Finds `count + rects` where every rect lies inside `page`.
fn find_rects(
    d: &[u8],
    start: usize,
    min_count: usize,
    page: Option<&RgbaImage>,
) -> Result<(Vec<[i32; 4]>, usize)> {
    for gap in 0..=12 {
        for seven in [false, true] {
            let count_pos = start + gap + if seven { 0 } else { 4 };
            let count = if seven {
                match read_7bit_at(d, start + gap) {
                    Some(v) => v as usize,
                    None => continue,
                }
            } else {
                match read_i32_at(d, start + gap) {
                    Ok(v) if v > 0 => v as usize,
                    _ => continue,
                }
            };
            if count < min_count || count > 4096 {
                continue;
            }
            let Ok(rects) = rects_at(d, count_pos, count) else {
                continue;
            };
            let valid = rects.iter().all(|r| {
                let [x, y, w, h] = *r;
                if let Some(page) = page {
                    x >= 0
                        && y >= 0
                        && w > 0
                        && h > 0
                        && x + w <= page.width as i32
                        && y + h <= page.height as i32
                } else {
                    x.abs() <= 4096
                        && y.abs() <= 4096
                        && (0..=512).contains(&w)
                        && (0..=512).contains(&h)
                }
            });
            if valid {
                return Ok((rects, count_pos + count * 16));
            }
        }
    }
    bail!("no rect table at {start}")
}

/// Padding table with a known entry count.
fn find_rects_exact(d: &[u8], start: usize, count: usize) -> Result<(Vec<[i32; 4]>, usize)> {
    for gap in 0..=12 {
        for seven in [false, true] {
            let (stored, after_count) = if seven {
                match read_7bit_at(d, start + gap) {
                    Some(v) => (v as usize, start + gap + 1),
                    None => continue,
                }
            } else {
                match read_i32_at(d, start + gap) {
                    Ok(v) if v > 0 => (v as usize, start + gap + 4),
                    _ => continue,
                }
            };
            if stored != count {
                continue;
            }
            let Ok(rects) = rects_at(d, after_count, count) else {
                continue;
            };
            let valid = rects.iter().all(|r| {
                let [x, y, w, h] = *r;
                x.abs() <= 512 && y.abs() <= 512 && (0..=512).contains(&w) && (0..=512).contains(&h)
            });
            if valid {
                return Ok((rects, after_count + count * 16));
            }
        }
    }
    bail!("no padding table at {start}")
}

fn read_7bit_at(d: &[u8], p: usize) -> Option<u32> {
    let mut v = 0u32;
    let mut sh = 0;
    let mut i = p;
    loop {
        let b = *d.get(i)?;
        i += 1;
        v |= ((b & 0x7F) as u32) << sh;
        if b & 0x80 == 0 {
            return Some(v);
        }
        sh += 7;
        if sh > 28 {
            return None;
        }
    }
}

/// Finds `count + a UTF-8 string of exactly `count` characters`.
fn find_chars(d: &[u8], start: usize, count: usize) -> Result<(Vec<char>, usize)> {
    for gap in 0..=12 {
        let Some(stored) = read_i32_at(d, start + gap).ok().filter(|v| *v > 0) else {
            continue;
        };
        if stored as usize != count {
            continue;
        }
        let mut p = start + gap + 4;
        let mut chars = Vec::with_capacity(count);
        let mut ok = true;
        while chars.len() < count {
            let Some(&b) = d.get(p) else {
                ok = false;
                break;
            };
            let len = match b {
                0x00..=0x7F => 1,
                0xC2..=0xDF => 2,
                0xE0..=0xEF => 3,
                0xF0..=0xF4 => 4,
                _ => {
                    ok = false;
                    break;
                }
            };
            let Some(slice) = d.get(p..p + len) else {
                ok = false;
                break;
            };
            let Ok(s) = std::str::from_utf8(slice) else {
                ok = false;
                break;
            };
            let Some(c) = s.chars().next() else {
                ok = false;
                break;
            };
            chars.push(c);
            p += len;
        }
        if ok && chars.len() == count {
            return Ok((chars, p));
        }
    }
    bail!("no character string at {start}")
}

/// Finds `count + 3 floats per entry`.
fn find_kerning(d: &[u8], start: usize, count: usize) -> Result<(Vec<[f32; 3]>, usize)> {
    for gap in 0..=12 {
        let Some(stored) = read_i32_at(d, start + gap).ok().filter(|v| *v > 0) else {
            continue;
        };
        if stored as usize != count {
            continue;
        }
        let mut p = start + gap + 4;
        let mut out = Vec::with_capacity(count);
        let mut ok = true;
        for _ in 0..count {
            let Ok(x) = read_f32_at(d, p) else {
                ok = false;
                break;
            };
            let Ok(y) = read_f32_at(d, p + 4) else {
                ok = false;
                break;
            };
            let Ok(z) = read_f32_at(d, p + 8) else {
                ok = false;
                break;
            };
            if !(x.abs() < 1000.0 && y.abs() < 1000.0 && z.abs() < 1000.0) {
                ok = false;
                break;
            }
            out.push([x, y, z]);
            p += 12;
        }
        if ok {
            return Ok((out, p));
        }
    }
    bail!("no kerning table at {start}")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn content_dir() -> Option<std::path::PathBuf> {
        std::env::var_os("TR_CONTENT").map(std::path::PathBuf::from)
    }

    #[test]
    fn decodes_the_ui_fonts() {
        let Some(dir) = content_dir() else { return };
        let font = load_font(dir.join("Fonts/Mouse_Text.xnb")).expect("Mouse_Text");
        println!(
            "Mouse_Text: spacing {} line {} default {:?} pages {} glyphs {}",
            font.spacing,
            font.line_spacing,
            font.default_char,
            font.pages.len(),
            font.pages[0].glyphs.len()
        );
        let page = &font.pages[0];
        println!("page {}x{}", page.texture.width, page.texture.height);
        for g in page
            .glyphs
            .iter()
            .filter(|g| matches!(g.ch, 'A' | 'a' | '1' | ' ' | '.'))
        {
            println!(
                "{:?} src {:?} pad {:?} kern {:?}",
                g.ch, g.src, g.pad, g.kerning
            );
        }
        assert!(page.glyphs.len() > 80);
        assert!(page.glyphs.iter().any(|g| g.ch == 'A'));
        let item = load_font(dir.join("Fonts/Item_Stack.xnb")).expect("Item_Stack");
        println!(
            "Item_Stack: line {} glyphs {}",
            item.line_spacing,
            item.pages[0].glyphs.len()
        );
    }
}
