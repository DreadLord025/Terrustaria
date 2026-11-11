//! Shelf-packed texture atlas.

use std::collections::HashMap;

use font8x8::UnicodeFonts;

/// Pixels of transparent padding around each sheet (avoids bilinear bleeding;
/// we also use nearest filtering, but padding keeps UV math forgiving).
const PAD: u32 = 2;

/// How a sheet lays out its 16x16 frames.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum SheetLayout {
    /// Block tilesets: 16x16 art, 18px stride, no inset (`Tiles_*.png`).
    #[default]
    Tile,
    /// Wall tilesets: 16x16 art centred in a 36px cell with an 8px inset
    /// (`Wall_*.png`). The original draws walls with a 32x32 source rect
    /// offset by -8, which maps this inner 16x16 region 1:1 onto the tile.
    Wall,
    /// Free-form art (sky gradients, clouds...) that is not split into frames.
    Raw,
}

impl SheetLayout {
    /// (stride between frames, inset of the art inside the cell)
    pub const fn geometry(self) -> (u32, u32) {
        match self {
            SheetLayout::Tile => (18, 0),
            SheetLayout::Wall => (36, 8),
            SheetLayout::Raw => (0, 0),
        }
    }
}

pub struct AtlasImage {
    pub name: String,
    pub width: u32,
    pub height: u32,
    /// RGBA8, tightly packed.
    pub rgba: Vec<u8>,
    pub layout: SheetLayout,
}

#[derive(Clone, Copy, Debug)]
pub struct SheetRect {
    /// Position inside the atlas in pixels.
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
    /// Frame chosen for tiles that need automatic framing.
    pub plain_frame: (i16, i16),
}

pub struct Atlas {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>,
    pub sheets: HashMap<String, SheetRect>,
}

impl Atlas {
    pub fn build(mut images: Vec<AtlasImage>, max_width: u32) -> Self {
        if !images.iter().any(|i| i.name == "__white") {
            images.push(AtlasImage {
                name: "__white".into(),
                width: 4,
                height: 4,
                rgba: vec![255; 4 * 4 * 4],
                layout: SheetLayout::Tile,
            });
        }
        // Tallest first keeps the shelf count low.
        images.sort_by(|a, b| b.height.cmp(&a.height).then(b.width.cmp(&a.width)));

        let mut x = PAD;
        let mut y = PAD;
        let mut shelf_height = 0;
        let mut layout = Vec::with_capacity(images.len());

        for img in &images {
            if x + img.width + PAD > max_width {
                x = PAD;
                y += shelf_height + PAD;
                shelf_height = 0;
            }
            layout.push((x, y));
            x += img.width + PAD;
            shelf_height = shelf_height.max(img.height);
        }
        let height = y + shelf_height + PAD;
        let width = max_width;

        let mut pixels = vec![0u8; (width as usize) * (height as usize) * 4];
        let mut sheets = HashMap::with_capacity(images.len());

        for (img, (px, py)) in images.iter().zip(layout) {
            let plain = pick_plain_frame(&img.name, &img.rgba, img.width, img.height, img.layout);
            log::debug!(
                "sheet {} {}x{} {:?} plain {:?}",
                img.name,
                img.width,
                img.height,
                img.layout,
                plain
            );
            for row in 0..img.height as usize {
                let src = row * img.width as usize * 4;
                let dst = ((py as usize + row) * width as usize + px as usize) * 4;
                pixels[dst..dst + img.width as usize * 4]
                    .copy_from_slice(&img.rgba[src..src + img.width as usize * 4]);
            }
            sheets.insert(
                img.name.clone(),
                SheetRect {
                    x: px,
                    y: py,
                    width: img.width,
                    height: img.height,
                    plain_frame: plain,
                },
            );
        }

        Self {
            width,
            height,
            pixels,
            sheets,
        }
    }

    pub fn sheet(&self, name: &str) -> Option<&SheetRect> {
        self.sheets.get(name)
    }

    pub fn memory_bytes(&self) -> usize {
        self.pixels.len()
    }

    /// True when every pixel of the 16x16 frame at `(fx, fy)` inside `sheet` is fully
    /// opaque. Used to skip walls/background that a block covers completely and to
    /// emit merged black rectangles for fully dark, fully covered cells.
    pub fn frame_is_opaque(&self, sheet: &SheetRect, fx: i16, fy: i16) -> bool {
        if fx < 0 || fy < 0 {
            return false;
        }
        self.rect_is_opaque(sheet, fx as u32, fy as u32, 16, 16)
    }

    /// True when every pixel of an arbitrary source rect inside `sheet` is opaque.
    pub fn rect_is_opaque(&self, sheet: &SheetRect, fx: u32, fy: u32, w: u32, h: u32) -> bool {
        if fx + w > sheet.width || fy + h > sheet.height {
            return false;
        }
        let stride = self.width as usize * 4;
        let mut i = ((sheet.y + fy) as usize) * stride + ((sheet.x + fx) as usize) * 4 + 3;
        for _ in 0..h {
            for x in 0..w {
                if self.pixels[i + x as usize * 4] != 255 {
                    return false;
                }
            }
            i += stride;
        }
        true
    }
}

/// Chooses a "plain" (fully interior) 16x16 frame from a tileset sheet.
///
/// Heuristic: prefer fully opaque frames whose border ring is bright and close to
/// the frame's average colour (edge/corner frames are outlined with dark pixels).
fn pick_plain_frame(
    name: &str,
    rgba: &[u8],
    width: u32,
    height: u32,
    layout: SheetLayout,
) -> (i16, i16) {
    const ART: u32 = 16;
    let (stride, inset) = layout.geometry();
    if layout == SheetLayout::Raw || width < stride || height < stride {
        return (inset as i16, inset as i16);
    }
    let cols = width / stride;
    let rows = height / stride;
    if cols == 0 || rows == 0 {
        return (0, 0);
    }

    let mut best = ((inset as i16, inset as i16), f32::MIN);
    for row in 0..rows {
        for col in 0..cols {
            let ox = col * stride + inset;
            let oy = row * stride + inset;
            let mut border_sum = 0f32;
            let mut border_n = 0f32;
            let mut min_alpha = 255u8;
            let mut lum_sum = 0f32;

            for py in 0..ART {
                for px in 0..ART {
                    let i = (((oy + py) * width + ox + px) * 4) as usize;
                    let (r, g, b, a) = (rgba[i], rgba[i + 1], rgba[i + 2], rgba[i + 3]);
                    min_alpha = min_alpha.min(a);
                    let lum = 0.299 * r as f32 + 0.587 * g as f32 + 0.114 * b as f32;
                    lum_sum += lum;
                    if px < 2 || py < 2 || px > 13 || py > 13 {
                        border_sum += lum;
                        border_n += 1.0;
                    }
                }
            }
            if min_alpha < 250 {
                continue;
            }
            let border = border_sum / border_n;
            let center = lum_sum / 256.0;
            // Bright border, close to the middle colour -> interior tile.
            let score = border - (border - center).abs() * 2.0;
            if score > best.1 {
                best = ((ox as i16, oy as i16), score);
            }
        }
    }
    if best.1 == f32::MIN {
        log::debug!("sheet {name} {width}x{height} {layout:?}: no fully opaque 16x16 frame");
    }
    best.0
}

/// Builds an 8x8 bitmap-font sheet (ASCII 32..127) as an atlas image.
/// Returns (image, glyph_width_px).
pub fn font_sheet() -> (AtlasImage, u32) {
    let chars = 96u32;
    let gw = 8u32;
    let width = chars * gw;
    let height = 8;
    let mut rgba = vec![0u8; (width * height * 4) as usize];

    for (ci, ch) in (32u8..128).enumerate() {
        let glyph = font8x8::BASIC_FONTS.get(ch as char).unwrap_or([0; 8]);
        for (row, bits) in glyph.iter().enumerate() {
            // bit 0 is the leftmost pixel in font8x8's LSB-first layout
            for col in 0..8usize {
                if bits & (1 << col) != 0 {
                    let x = ci as u32 * gw + col as u32;
                    let y = row as u32;
                    let i = ((y * width + x) * 4) as usize;
                    rgba[i] = 255;
                    rgba[i + 1] = 255;
                    rgba[i + 2] = 255;
                    rgba[i + 3] = 255;
                }
            }
        }
    }

    (
        AtlasImage {
            name: "__font".into(),
            width,
            height,
            rgba,
            layout: SheetLayout::Tile,
        },
        gw,
    )
}
