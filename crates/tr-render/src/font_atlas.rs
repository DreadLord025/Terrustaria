//! The font atlas: the retail `Fonts/*.xnb` page surfaces packed into a BC2
//! texture array.
//!
//! The game's five `DynamicSpriteFont`s ship ~160 pages each (38 636 glyphs
//! covering Latin, Cyrillic, Greek, the CJK ideographs, Kana and full Hangul);
//! as mip-0 DXT3/BC2 blocks they total ~110 MB — 4x that as decompressed RGBA,
//! which does not fit a 8192-wide RGBA atlas. XNA uploads the same compressed
//! surfaces verbatim (`Texture2D` `SurfaceFormat.Dxt3`), so the port does too:
//! the pages are shelf-packed into 8192x8192 array layers and sampled as
//! `Bc2RgbaUnormSrgb`, reproducing the vanilla pixels exactly.
//!
//! Pages land at PAD-aligned shelf positions like [`crate::atlas::Atlas`]'s
//! shelf builder, so the layout is reproducible across runs (the glyph UVs
//! derive from it).

/// The shelf padding between packed sheets, matching [`crate::atlas::Atlas`]'s
/// `PAD` so glyph rects keep the same 1 px guard band.
const PAD: u32 = 1;
/// One packed layer's edge (`Atlas::build`'s `max_width`).
const LAYER_SIZE: u32 = 8192;

/// One font page as uploaded: the shipped mip-0 BC2 blocks.
#[derive(Clone, Debug)]
pub struct FontPageData {
    pub width: u32,
    pub height: u32,
    /// BC2 blocks, `ceil(w/4) * ceil(h/4) * 16` bytes.
    pub blocks: Vec<u8>,
}

/// Where one page landed inside the array.
#[derive(Clone, Copy, Debug)]
pub struct PageRect {
    /// 1-based instance layer (0 is reserved for the main atlas); the shader
    /// subtracts 1 before indexing the array.
    pub layer: u32,
    /// Normalized uv rect inside the layer.
    pub uv: [f32; 4],
}

/// One layer's shelf cursor plus its packed BC2 bytes. The cursor and the
/// page positions work in 4x4 block units — a compressed texture can only be
/// written at block-aligned origins, so the packer rounds every position to
/// a block boundary (the `PAD` guard band is one block, 4 px).
struct Layer {
    x_blocks: u32,
    y_blocks: u32,
    row_height_blocks: u32,
    data: Vec<u8>,
}

/// Blocks per layer edge (8192 / 4).
const BLOCKS: u32 = LAYER_SIZE / 4;

impl Layer {
    fn new() -> Self {
        // BC2 stores 16 bytes per 4x4 block = 1 byte per pixel.
        Layer {
            x_blocks: 1,
            y_blocks: 1,
            row_height_blocks: 0,
            data: vec![0u8; (LAYER_SIZE * LAYER_SIZE) as usize],
        }
    }

    /// Places the page if it fits below the current shelf row, wrapping to a
    /// new row first (`Atlas::build`'s cursor walk). The page dimensions must
    /// be block multiples (every retail font page is).
    fn place(&mut self, page: &FontPageData) -> Option<PageRect> {
        if page.width % 4 != 0 || page.height % 4 != 0 {
            return None;
        }
        let bw = page.width / 4;
        let bh = page.height / 4;
        if self.x_blocks + bw + 1 > BLOCKS {
            self.x_blocks = 1;
            self.y_blocks += self.row_height_blocks + 1;
            self.row_height_blocks = 0;
        }
        if self.y_blocks + bh + 1 > BLOCKS {
            return None;
        }
        let (x, y) = (self.x_blocks, self.y_blocks);
        let stride = (BLOCKS * 16) as usize;
        let src_stride = (bw * 16) as usize;
        for row in 0..bh as usize {
            let src = row * src_stride;
            let dst = (y as usize + row) * stride + x as usize * 16;
            self.data[dst..dst + src_stride].copy_from_slice(&page.blocks[src..src + src_stride]);
        }
        self.x_blocks += bw + 1;
        self.row_height_blocks = self.row_height_blocks.max(bh);
        Some(PageRect {
            layer: 0,
            uv: [
                (x * 4) as f32 / LAYER_SIZE as f32,
                (y * 4) as f32 / LAYER_SIZE as f32,
                ((x + bw) * 4) as f32 / LAYER_SIZE as f32,
                ((y + bh) * 4) as f32 / LAYER_SIZE as f32,
            ],
        })
    }
}

pub struct FontAtlas {
    /// Packed BC2 bytes per 8192x8192 layer, in layer order.
    pub layers: Vec<Vec<u8>>,
    /// One `PageRect` per input page, in input order.
    pub rects: Vec<PageRect>,
}

impl FontAtlas {
    /// Shelf-packs `pages` (tallest first) into `LAYER_SIZE` layers.
    pub fn build(mut pages: Vec<FontPageData>) -> Self {
        // Tallest first, then widest, then the input order — a deterministic
        // tiebreak so the layout is reproducible across runs.
        let mut order: Vec<usize> = (0..pages.len()).collect();
        order.sort_by(|&a, &b| {
            pages[b]
                .height
                .cmp(&pages[a].height)
                .then(pages[b].width.cmp(&pages[a].width))
                .then(a.cmp(&b))
        });

        let mut layers: Vec<Layer> = Vec::new();
        let mut rects = vec![
            PageRect {
                layer: 0,
                uv: [0.0; 4]
            };
            pages.len()
        ];
        for pi in order {
            let page = &pages[pi];
            let mut rect = None;
            for (li, layer) in layers.iter_mut().enumerate() {
                if let Some(r) = layer.place(page) {
                    rect = Some((li, r));
                    break;
                }
            }
            let Some((li, mut r)) = rect else {
                let mut layer = Layer::new();
                let mut r = layer
                    .place(page)
                    .expect("a fresh 8192x8192 layer fits any font page");
                layers.push(layer);
                r.layer = (layers.len()) as u32;
                rects[pi] = r;
                continue;
            };
            r.layer = (li + 1) as u32;
            rects[pi] = r;
        }

        let data = layers.into_iter().map(|l| l.data).collect();
        Self {
            layers: data,
            rects,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A 64x64 page (16x16 blocks = 4 KB) packed with a 32x32 page next to it
    /// in one layer, tallest first.
    #[test]
    fn packs_shelf_layers() {
        let big = FontPageData {
            width: 64,
            height: 64,
            blocks: vec![1u8; 64 * 64],
        };
        let small = FontPageData {
            width: 32,
            height: 32,
            blocks: vec![2u8; 32 * 32],
        };
        let atlas = FontAtlas::build(vec![small, big]);
        assert_eq!(atlas.layers.len(), 1);
        let uv_big = atlas.rects[1].uv;
        assert_eq!(uv_big[0], 4.0 / LAYER_SIZE as f32);
        assert_eq!(uv_big[1], 4.0 / LAYER_SIZE as f32);
        assert_eq!(atlas.rects[1].layer, 1);
        let uv_small = atlas.rects[0].uv;
        assert_eq!(uv_small[0], (4.0 + 64.0 + 4.0) / LAYER_SIZE as f32);
        assert_eq!(atlas.rects[0].layer, 1);
        // The small page's first BC2 block lands at its shelf position.
        let stride = (BLOCKS * 16) as usize;
        let x = ((uv_small[0] * LAYER_SIZE as f32) as usize / 4) * 16;
        let y = (uv_small[1] * LAYER_SIZE as f32) as usize / 4;
        assert_eq!(atlas.layers[0][y * stride + x], 2);
    }

    /// Pages wider than the remaining row wrap to a new row; a full layer
    /// starts the next one.
    #[test]
    fn wraps_rows_then_layers() {
        let pages = (0..3)
            .map(|_| FontPageData {
                width: 4096,
                height: 4096,
                blocks: vec![0u8; 4096 * 4096],
            })
            .collect();
        let atlas = FontAtlas::build(pages);
        assert_eq!(atlas.layers.len(), 3);
        // Layer 1 holds page 0; layer 2 wraps page 1; layer 3 wraps page 2
        // (each page is 4096 tall, so one row per layer).
        assert_eq!(atlas.rects[0].layer, 1);
        assert_eq!(atlas.rects[1].layer, 2);
        assert_eq!(atlas.rects[2].layer, 3);
    }
}
