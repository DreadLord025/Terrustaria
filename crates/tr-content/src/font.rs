//! ReLogic `DynamicSpriteFontReader` — the encoding Terraria 1.4.5.8 actually
//! ships, verified byte-exact against all five retail fonts (`Fonts/*.xnb`).
//!
//! The decompiled `decompiled-libs/ReLogic/ReLogic.Graphics/DynamicSpriteFontReader.cs`
//! documents the logical structure (spacing, line spacing, default char, pages of
//! texture + glyphs + padding + characters + kerning), but the shipped writer
//! differs from it in three ways, which the byte walk pins down:
//!
//! 1. the default character is written as a 7-bit encoded int (`'*'` = 42 in all
//!    five retail fonts), and the page count follows as a raw `Int32`;
//! 2. every table (texture, glyph rects, padding, characters, kerning) carries a
//!    7-bit `ReadObject` reader-index prefix — 2 = `Texture2DReader`, 3 =
//!    `ListReader<Rectangle>`, 5 = `ListReader<Char>`, 7 = `ListReader<Vector3>`
//!    (the order in the file's reader table, `XnbFile::read`) — but the *list*
//!    lengths are raw `Int32`s, not XNA's 7-bit lengths;
//! 3. the character list is a UTF-8 string, not UTF-16 (`ListReader<Char>` would
//!    store 2 bytes per char).
//!
//! The parser therefore walks the payload deterministically and accepts a font
//! only when the walk consumes it exactly: every page must carry four tables of
//! the same length, every glyph rect must lie inside the page texture, and the
//! final kerning table must end precisely at the payload end. A 1.4.5.8 font is
//! fully CJK+Cyrillic-capable: `Mouse_Text` ships 159 pages / 38 636 glyphs.
//!
//! Page surfaces stay in their shipped compressed form (`SurfaceFormat.Dxt3` =
//! BC2 in every retail font) so the renderer uploads them exactly like XNA does —
//! GPU BC2 sampling reproduces the original pixels without a 4x decompression
//! blow-up (~230 MB of font textures instead of ~900 MB).

use std::path::Path;

use anyhow::{bail, Result};

use crate::xnb::XnbFile;

/// `SurfaceFormat.Dxt3` — the compressed form of every retail font page.
pub const SURFACE_DXT3: i32 = 5;

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

/// One font page: the mip-0 surface in its shipped form plus the glyph tables.
#[derive(Clone, Debug)]
pub struct FontPage {
    /// `Texture2D.format` (`SurfaceFormat.Dxt3` for the retail fonts).
    pub format: i32,
    pub width: u32,
    pub height: u32,
    /// Mip 0 in the shipped compressed form (BC2 blocks, `w * h / 2` bytes).
    pub mip0: Vec<u8>,
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

/// The `ReadObject` reader-indexes the shipped writer emits for each table, in
/// the file's reader-table order (`XnbFile.readers`): 2 = `Texture2DReader`,
/// 3 = `ListReader<Rectangle>`, 5 = `ListReader<Char>`, 7 = `ListReader<Vector3>`.
const RID_TEXTURE: u32 = 2;
const RID_RECT_LIST: u32 = 3;
const RID_CHAR_LIST: u32 = 5;
const RID_VECTOR3_LIST: u32 = 7;

pub(crate) fn read_font(d: &[u8]) -> Result<DynamicSpriteFont> {
    let mut p = 0usize;
    let spacing = f32_at(d, &mut p)?;
    let line_spacing = i32_at(d, &mut p)?;
    if !(-10.0..=10.0).contains(&spacing) || !(-10..=400).contains(&line_spacing) {
        bail!("implausible font metrics: spacing {spacing}, line {line_spacing}");
    }
    // `input.ReadChar()` in the decompiled reader; the shipped writer emits a
    // 7-bit encoded codepoint (always a single ASCII byte in the retail fonts).
    let default_char = u32::from(seven_at(d, &mut p)?)
        .try_into()
        .map_err(|_| anyhow::anyhow!("default char out of range"))?;
    let page_count = i32_at(d, &mut p)?;
    if !(1..=4096).contains(&page_count) {
        bail!("implausible page count {page_count}");
    }

    let mut pages = Vec::with_capacity(page_count as usize);
    for _ in 0..page_count {
        pages.push(read_page(d, &mut p)?);
    }
    if p != d.len() {
        bail!("{} trailing bytes after the last page", d.len() - p);
    }
    Ok(DynamicSpriteFont {
        spacing,
        line_spacing,
        default_char,
        pages,
    })
}

/// One page: `Texture2D` + glyph rects + padding + UTF-8 characters + kerning,
/// each list carrying its 7-bit reader index and a raw `Int32` length.
fn read_page(d: &[u8], p: &mut usize) -> Result<FontPage> {
    let rid = seven_at(d, p)?;
    if rid != RID_TEXTURE {
        bail!("page texture: reader index {rid}, expected {RID_TEXTURE}");
    }
    let format = i32_at(d, p)?;
    let width = i32_at(d, p)?;
    let height = i32_at(d, p)?;
    let mips = i32_at(d, p)?;
    if !(8..=4096).contains(&width) || !(8..=4096).contains(&height) {
        bail!("bad page size {width}x{height}");
    }
    if !(1..=8).contains(&mips) {
        bail!("bad mip count {mips}");
    }
    let (width, height) = (width as u32, height as u32);
    let expect = expected_surface_bytes(format, width, height)
        .ok_or_else(|| anyhow::anyhow!("bad surface format {format}"))?;
    // Mip 0 first; the smaller mips are parsed and discarded (the renderer
    // never samples them — the sprite pipeline draws straight 1:1 quads).
    let mip0_size = i32_at(d, p)?;
    if mip0_size as usize != expect {
        bail!("mip 0 size {mip0_size} != {expect}");
    }
    let mip0 = bytes_at(d, p, expect)?.to_vec();
    for _ in 1..mips {
        let size = i32_at(d, p)?;
        if size < 0 {
            bail!("negative mip size");
        }
        bytes_at(d, p, size as usize)?;
    }

    let (glyphs, end) = read_rect_list(d, p, Some((width, height)))?;
    let count = glyphs.len();
    *p = end;
    let (padding, end) = read_rect_list(d, p, None)?;
    if padding.len() != count {
        bail!("padding table has {} entries, expected {count}", padding.len());
    }
    *p = end;
    let characters = read_char_list(d, p, count)?;
    let characters: Vec<char> = characters.chars().collect();

    let (kerning, end) = read_vector3_list(d, p, count)?;
    *p = end;

    Ok(FontPage {
        format,
        width,
        height,
        mip0,
        glyphs: (0..count)
            .map(|i| Glyph {
                ch: characters[i],
                src: glyphs[i],
                pad: padding[i],
                kerning: kerning[i],
            })
            .collect(),
    })
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

fn i32_at(d: &[u8], p: &mut usize) -> Result<i32> {
    let b = d
        .get(*p..*p + 4)
        .ok_or_else(|| anyhow::anyhow!("short read at {}", p))?;
    *p += 4;
    Ok(i32::from_le_bytes(b.try_into().unwrap()))
}

fn f32_at(d: &[u8], p: &mut usize) -> Result<f32> {
    Ok(f32::from_bits(i32_at(d, p)? as u32))
}

fn bytes_at<'a>(d: &'a [u8], p: &mut usize, n: usize) -> Result<&'a [u8]> {
    let end = (*p)
        .checked_add(n)
        .ok_or_else(|| anyhow::anyhow!("byte count overflow at {}", p))?;
    let b = d
        .get(*p..end)
        .ok_or_else(|| anyhow::anyhow!("short surface read at {}", p))?;
    *p = end;
    Ok(b)
}

fn seven_at(d: &[u8], p: &mut usize) -> Result<u32> {
    let mut v = 0u32;
    let mut sh = 0;
    loop {
        let b = *d
            .get(*p)
            .ok_or_else(|| anyhow::anyhow!("unexpected end at {}", p))?;
        *p += 1;
        v |= ((b & 0x7F) as u32) << sh;
        if b & 0x80 == 0 {
            return Ok(v);
        }
        sh += 7;
        if sh > 28 {
            bail!("malformed 7-bit int at {}", p);
        }
    }
}

/// `ListReader<Rectangle>`: reader index + `Int32` count + `count` rects.
/// With `page` set, every rect must lie inside it (the glyph table check).
fn read_rect_list(
    d: &[u8],
    p: &mut usize,
    page: Option<(u32, u32)>,
) -> Result<(Vec<[i32; 4]>, usize)> {
    let rid = seven_at(d, p)?;
    if rid != RID_RECT_LIST {
        bail!("rect list: reader index {rid}, expected {RID_RECT_LIST}");
    }
    let count = i32_at(d, p)?;
    if !(0..=65536).contains(&count) {
        bail!("implausible rect count {count}");
    }
    let mut out = Vec::with_capacity(count as usize);
    for _ in 0..count {
        let x = i32_at(d, p)?;
        let y = i32_at(d, p)?;
        let w = i32_at(d, p)?;
        let h = i32_at(d, p)?;
        if let Some((pw, ph)) = page {
            if x < 0 || y < 0 || w <= 0 || h <= 0 || x + w > pw as i32 || y + h > ph as i32 {
                bail!("glyph rect {x},{y} {w}x{h} outside the {pw}x{ph} page");
            }
        }
        out.push([x, y, w, h]);
    }
    Ok((out, *p))
}

/// `ListReader<Char>` in the shipped form: reader index + `Int32` char count +
/// a UTF-8 string of exactly `count` characters.
fn read_char_list(d: &[u8], p: &mut usize, count: usize) -> Result<String> {
    let rid = seven_at(d, p)?;
    if rid != RID_CHAR_LIST {
        bail!("char list: reader index {rid}, expected {RID_CHAR_LIST}");
    }
    let stored = i32_at(d, p)?;
    if stored as usize != count {
        bail!("char list has {stored} entries, expected {count}");
    }
    let mut s = String::with_capacity(count);
    for _ in 0..count {
        let b = *d
            .get(*p)
            .ok_or_else(|| anyhow::anyhow!("unexpected end in the char list at {}", p))?;
        let len = match b {
            0x00..=0x7F => 1,
            0xC2..=0xDF => 2,
            0xE0..=0xEF => 3,
            0xF0..=0xF4 => 4,
            _ => bail!("invalid UTF-8 lead byte {b:#x} in the char list at {}", p),
        };
        let slice = d
            .get(*p..*p + len)
            .ok_or_else(|| anyhow::anyhow!("short char list at {}", p))?;
        let ch = std::str::from_utf8(slice)
            .map_err(|_| anyhow::anyhow!("bad UTF-8 in the char list at {}", p))?
            .chars()
            .next()
            .unwrap_or('?');
        s.push(ch);
        *p += len;
    }
    Ok(s)
}

/// `ListReader<Vector3>`: reader index + `Int32` count + `count` 3-float rows.
fn read_vector3_list(d: &[u8], p: &mut usize, count: usize) -> Result<(Vec<[f32; 3]>, usize)> {
    let rid = seven_at(d, p)?;
    if rid != RID_VECTOR3_LIST {
        bail!("kerning list: reader index {rid}, expected {RID_VECTOR3_LIST}");
    }
    let stored = i32_at(d, p)?;
    if stored as usize != count {
        bail!("kerning list has {stored} entries, expected {count}");
    }
    let mut out = Vec::with_capacity(count);
    for _ in 0..count {
        let x = f32_at(d, p)?;
        let y = f32_at(d, p)?;
        let z = f32_at(d, p)?;
        if !(x.abs() < 1000.0 && y.abs() < 1000.0 && z.abs() < 1000.0) {
            bail!("implausible kerning {x},{y},{z}");
        }
        out.push([x, y, z]);
    }
    Ok((out, *p))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn content_dir() -> Option<std::path::PathBuf> {
        std::env::var_os("TR_CONTENT").map(std::path::PathBuf::from)
    }

    /// The shipped-format pins for all five retail fonts (page counts, glyph
    /// totals, the `'*'` default char, full-payload consumption — a parse error
    /// fails the walk before the pin checks run).
    #[test]
    fn decodes_the_ui_fonts() {
        let Some(dir) = content_dir() else { return };
        for (name, pages, glyphs, line) in [
            ("Mouse_Text", 159usize, 38636usize, 29i32),
            ("Item_Stack", 159, 38636, 24),
            ("Death_Text", 159, 38636, 72),
            ("Combat_Text", 160, 38636, 30),
            ("Combat_Crit", 160, 38636, 38),
        ] {
            let font = load_font(dir.join(format!("Fonts/{name}.xnb")))
                .unwrap_or_else(|e| panic!("{name}: {e:#}"));
            assert_eq!(font.default_char, '*', "{name} default char");
            assert_eq!(font.pages.len(), pages, "{name} pages");
            let total: usize = font.pages.iter().map(|p| p.glyphs.len()).sum();
            assert_eq!(total, glyphs, "{name} glyphs");
            assert_eq!(font.line_spacing, line, "{name} line spacing");
            // Every page is the shipped DXT3 surface of the right byte size.
            for page in &font.pages {
                assert_eq!(page.format, SURFACE_DXT3, "{name} page format");
                assert_eq!(
                    page.mip0.len(),
                    (page.width as usize) * (page.height as usize),
                    "{name} page {}x{} mip size",
                    page.width,
                    page.height
                );
            }
        }
    }

    /// The retail `Mouse_Text` covers the scripts the shipped localizations
    /// need: Latin + Greek on page 0, Cyrillic and the CJK ideographs on later
    /// pages, full Hangul at the end.
    #[test]
    fn covers_the_scripts_the_localizations_use() {
        let Some(dir) = content_dir() else { return };
        let font = load_font(dir.join("Fonts/Mouse_Text.xnb")).expect("Mouse_Text");
        let mut all = String::new();
        for page in &font.pages {
            for g in &page.glyphs {
                all.push(g.ch);
            }
        }
        for probe in ["A", "АБВГДабвгдёЁ", "你好世界中文", "日本語テスト", "한국어"] {
            assert!(
                probe.chars().all(|c| all.contains(c)),
                "glyph coverage missing {probe:?}"
            );
        }
        // Page 0 is the Latin/Greek page; Cyrillic lives on its own page.
        let page0: String = font.pages[0].glyphs.iter().map(|g| g.ch).collect();
        assert!(page0.contains('A') && page0.contains('Ω'));
        assert!(!page0.contains('А'));
        let cyr: String = font
            .pages
            .iter()
            .flat_map(|p| p.glyphs.iter())
            .filter(|g| ('\u{0400}'..='\u{04FF}').contains(&g.ch))
            .map(|g| g.ch)
            .collect();
        assert!(cyr.contains('А') && cyr.contains('я'), "Cyrillic page");
    }

    /// Page-0 glyph metrics for a few known chars, pinned from the retail
    /// `Mouse_Text` (the exact `Rectangle`/`Padding`/`Kerning` rows).
    #[test]
    fn pins_page0_metrics() {
        let Some(dir) = content_dir() else { return };
        let font = load_font(dir.join("Fonts/Mouse_Text.xnb")).expect("Mouse_Text");
        let page = &font.pages[0];
        for ch in ['A', 'a', '1', ' ', '.'] {
            let g = page
                .glyphs
                .iter()
                .find(|g| g.ch == ch)
                .unwrap_or_else(|| panic!("{ch:?} glyph"));
            println!("{ch:?} src {:?} pad {:?} kern {:?}", g.src, g.pad, g.kerning);
        }
        let a = page.glyphs.iter().find(|g| g.ch == 'A').unwrap();
        assert_eq!(a.src, [242, 40, 13, 15]);
        assert_eq!(a.pad, [0, 2, 13, 27]);
    }
}
