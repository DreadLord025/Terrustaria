//! Skyline-packed texture atlas.

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

#[derive(Clone)]
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
    pub fn build(images: Vec<AtlasImage>, max_width: u32) -> Self {
        let mut images = images;
        if !images.iter().any(|i| i.name == "__white") {
            images.push(AtlasImage {
                name: "__white".into(),
                width: 4,
                height: 4,
                rgba: vec![255; 4 * 4 * 4],
                layout: SheetLayout::Tile,
            });
        }
        Self::build_ref(&images, max_width)
    }

    /// [`Atlas::build`] over borrowed sheets: the pixels are copied out of
    /// `images` and the list stays owned by the caller. The world swap uses
    /// this to pack the same decoded static sheets into every new world's
    /// atlas without a second decode (the layout is identical to `build`'s —
    /// the same dedup, sort and skyline pass over the same input).
    pub fn build_ref(images: &[AtlasImage], max_width: u32) -> Self {
        // Name dedup (S1-G-12b): the enumeration decodes some sheets twice
        // (41 of the 48 `Projectile_*` names in `STATIC_SHEET_NAMES` are also
        // packed by the `proj_data` loop), but the `sheets` HashMap keeps one
        // rect per name — the other copy's pixels were dead weight in the
        // atlas. Keep the FIRST occurrence of each name: both copies are the
        // same file decoded twice, so the pixels are identical and every
        // lookup samples the same texels as before (the old last-write-wins
        // rect pointed at an identical copy).
        let mut seen = std::collections::HashSet::with_capacity(images.len());
        let mut order: Vec<&AtlasImage> = images
            .iter()
            .filter(|i| seen.insert(i.name.as_str()))
            .collect();
        let white = AtlasImage {
            name: "__white".into(),
            width: 4,
            height: 4,
            rgba: vec![255; 4 * 4 * 4],
            layout: SheetLayout::Tile,
        };
        if !order.iter().any(|i| i.name == "__white") {
            order.push(&white);
        }
        // Tallest first keeps the skyline low; the stable sort breaks size
        // ties by input order, so the layout is a pure function of the list.
        order.sort_by(|a, b| b.height.cmp(&a.height).then(b.width.cmp(&a.width)));

        // Skyline bottom-left packing (S1-G-12b, replacing the best-fit
        // decreasing-height shelves of S1-F-13c slice B): the shelf pass
        // strands every row's width tail, and the ~1400 gore sheets of
        // S1-E-4b pushed it past the 8192 strict-GPU bound (8493 px with
        // gore, 8664 px with gore + the S1-L-8b mount sheets). The skyline
        // backfills those tails with the short sheets — the same full set
        // packs in 7932 px (measured by tools/atlas_sim.js over the real
        // content dims). Deterministic: the same sort + the same fixed
        // tie-break scan over the same input; every consumer reads its UVs
        // through `sheets`.
        let rects: Vec<(u32, u32)> = order.iter().map(|i| (i.width + PAD, i.height + PAD)).collect();
        let (positions, used_height) = skyline_pack(&rects, max_width - PAD);
        let height = used_height + PAD;
        let width = max_width;

        let mut pixels = vec![0u8; (width as usize) * (height as usize) * 4];
        let mut sheets = HashMap::with_capacity(order.len());

        for (img, &(px, py)) in order.iter().zip(&positions) {
            // skyline_pack works in the PAD-inset bin; shift back out.
            let (px, py) = (px + PAD, py + PAD);
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

    /// The packed height [`build_ref`] would produce for `images`, WITHOUT
    /// copying any pixels — the same name-dedup, tallest-first stable sort and
    /// skyline pass, returning only `used_height + PAD`. The atlas budget guard
    /// (`assets::fit_bg_styles_to_atlas`) trials candidate background-sheet sets
    /// through this so it can measure overflow cheaply (the full `build_ref`
    /// allocates the whole atlas pixel buffer per trial).
    pub fn packed_height(images: &[AtlasImage], max_width: u32) -> u32 {
        let mut seen = std::collections::HashSet::with_capacity(images.len());
        let mut order: Vec<&AtlasImage> = images
            .iter()
            .filter(|i| seen.insert(i.name.as_str()))
            .collect();
        // `build_ref` appends a synthetic 4x4 `__white` when absent.
        let white = AtlasImage {
            name: "__white".into(),
            width: 4,
            height: 4,
            rgba: Vec::new(),
            layout: SheetLayout::Tile,
        };
        if !order.iter().any(|i| i.name == "__white") {
            order.push(&white);
        }
        order.sort_by(|a, b| b.height.cmp(&a.height).then(b.width.cmp(&a.width)));
        let rects: Vec<(u32, u32)> = order.iter().map(|i| (i.width + PAD, i.height + PAD)).collect();
        let (_, used_height) = skyline_pack(&rects, max_width - PAD);
        used_height + PAD
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

/// Skyline bottom-left rectangle packing (S1-G-12b).
///
/// `rects` are the PAD-inflated sheet sizes `(w + PAD, h + PAD)`, already in
/// placement order (tallest first, stable); the bin is `bin_width` wide and
/// unbounded in height. Returns each rect's top-left position inside the bin
/// and the used height. The caller offsets everything by `PAD` so the atlas
/// keeps its `PAD` border on every side and at least `PAD` between sheets.
///
/// The skyline is a list of `(x, y, width)` nodes sorted by `x`, covering
/// `[0, bin_width)` with no gaps; `y` is the height of the packed column
/// below the node. Each rect tries every node's left edge: the placement `y`
/// is the max skyline height over the span the rect covers, and the winner
/// minimises `(y, waste, x)` — bottom-left first, then the least area buried
/// under the rect (the skyline's valleys it fills), then leftmost. All three
/// keys are integers and the scan order is fixed, so the layout is a pure
/// function of the input (the world-swap `build_ref` contract).
///
/// After placement the covered nodes are trimmed/split around the new node
/// `(x, y + h, w)` and adjacent equal-height nodes merge, keeping the node
/// count near the number of distinct column heights.
///
/// Port-infrastructure, not game logic: the C# draws per texture
/// (`TileBatch.cs:485-536`) and has no atlas; the packing algorithm is the
/// port's own (research/41 D-3). Mirrored by `tools/atlas_sim.js` for the
/// measured-height evidence in ANALYSIS "The atlas skyline packer".
fn skyline_pack(rects: &[(u32, u32)], bin_width: u32) -> (Vec<(u32, u32)>, u32) {
    // (x, y, width), sorted by x, covering [0, bin_width).
    let mut nodes: Vec<(u32, u32, u32)> = vec![(0, 0, bin_width)];
    let mut positions = Vec::with_capacity(rects.len());
    for &(w, h) in rects {
        assert!(w <= bin_width, "sheet wider than the atlas: {w} > {bin_width}");
        let mut best: Option<(u32, u32, u64)> = None; // (x, y, waste)
        for i in 0..nodes.len() {
            let x = nodes[i].0;
            if x + w > bin_width {
                continue;
            }
            // y = max skyline height over [x, x + w); waste = the area
            // between that height and the skyline under the span.
            let mut y = 0u32;
            let mut waste = 0u64;
            let mut acc = 0u32;
            let mut j = i;
            while acc < w && j < nodes.len() {
                let (_, ny, nw) = nodes[j];
                y = y.max(ny);
                let seg = nw.min(w - acc);
                // second pass below needs y final; accumulate heights here
                acc += seg;
                j += 1;
            }
            if acc < w {
                continue; // the span runs off the bin's right edge
            }
            acc = 0;
            for k in i..j {
                let (_, ny, nw) = nodes[k];
                let seg = nw.min(w - acc);
                waste += seg as u64 * (y - ny) as u64;
                acc += seg;
            }
            let better = match best {
                None => true,
                Some((bx, by, bw)) => {
                    y < by || (y == by && (waste < bw || (waste == bw && x < bx)))
                }
            };
            if better {
                best = Some((x, y, waste));
            }
        }
        let (x, y, _) = best.expect("a rect that fits the bin width always finds a skyline slot");
        positions.push((x, y));
        // Insert (x, y + h, w), trimming the nodes the rect covers.
        let mut next: Vec<(u32, u32, u32)> = Vec::with_capacity(nodes.len() + 1);
        let mut inserted = false;
        for &(nx, ny, nw) in &nodes {
            let nend = nx + nw;
            if nend <= x || nx >= x + w {
                // untouched; keep x order when splicing the new node in
                if !inserted && nx > x {
                    next.push((x, y + h, w));
                    inserted = true;
                }
                next.push((nx, ny, nw));
                continue;
            }
            if !inserted {
                if nx < x {
                    next.push((nx, ny, x - nx));
                }
                next.push((x, y + h, w));
                inserted = true;
            }
            if nend > x + w {
                next.push((x + w, ny, nend - (x + w)));
            }
        }
        if !inserted {
            next.push((x, y + h, w));
        }
        // Merge adjacent equal-height nodes.
        let mut merged: Vec<(u32, u32, u32)> = Vec::with_capacity(next.len());
        for n in next {
            if let Some(last) = merged.last_mut() {
                if last.1 == n.1 && last.0 + last.2 == n.0 {
                    last.2 += n.2;
                    continue;
                }
            }
            merged.push(n);
        }
        nodes = merged;
    }
    let height = nodes.iter().map(|n| n.1).max().unwrap_or(0);
    (positions, height)
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

#[cfg(test)]
mod tests {
    use super::*;

    fn sheet(name: &str, w: u32, h: u32, seed: u8) -> AtlasImage {
        AtlasImage {
            name: name.into(),
            width: w,
            height: h,
            rgba: (0..(w * h * 4))
                .map(|i| (i as u8).wrapping_add(seed))
                .collect(),
            layout: SheetLayout::Raw,
        }
    }

    /// `build_ref` is the world swap's packing path (it borrows the sheets so the
    /// decoded static half stays cached): it must place every sheet exactly where
    /// `build` would, pixel for pixel — otherwise a swapped world would draw with
    /// the boot world's UVs.
    #[test]
    fn build_ref_matches_build_bit_for_bit() {
        // `AtlasImage` is not `Clone` (the pixels are tens of MB), so the list
        // is built twice from the same recipe.
        let recipe = || {
            vec![
                sheet("Tiles_1", 16, 16, 1),
                sheet("Tiles_2", 32, 48, 2),
                sheet("Wall_3", 36, 36, 3),
                sheet("Item_4", 20, 20, 4),
                sheet("Tall_5", 8, 200, 5),
            ]
        };
        let sheets = recipe();
        let owned = Atlas::build(recipe(), 512);
        let borrowed = Atlas::build_ref(&sheets, 512);

        assert_eq!(owned.width, borrowed.width);
        assert_eq!(owned.height, borrowed.height);
        assert_eq!(owned.pixels, borrowed.pixels, "the packed pixels must match");
        assert_eq!(owned.sheets.len(), borrowed.sheets.len());
        for (name, rect) in &owned.sheets {
            let other = borrowed.sheets.get(name).expect("same sheet set");
            assert_eq!((rect.x, rect.y), (other.x, other.y), "{name} placement");
            assert_eq!(rect.plain_frame, other.plain_frame, "{name} plain frame");
        }
        // The synthetic white sheet both paths add is not in the caller's list.
        assert!(!sheets.iter().any(|s| s.name == "__white"));
        assert!(borrowed.sheets.contains_key("__white"));
    }

    // --- S1-G-12b: the skyline packer's witnesses (written, not run —
    // code-only mode; no compiler invoked on this branch). ---

    /// Determinism: two `build_ref` runs over the same input must produce the
    /// identical layout AND the identical pixels (the world-swap contract —
    /// a swapped world draws with the boot world's UVs).
    #[test]
    fn skyline_layout_is_deterministic_across_runs() {
        let recipe = || {
            vec![
                sheet("Tiles_1", 288, 270, 1),
                sheet("Background_0", 48, 1400, 2),
                sheet("Item_71", 20, 20, 3),
                sheet("Gore_1", 16, 16, 4),
                sheet("Gore_2", 14, 12, 5),
                sheet("water_0", 48, 1360, 6),
                sheet("Cloud_3", 200, 90, 7),
                sheet("NPC_1", 40, 560, 8),
            ]
        };
        let a = Atlas::build(recipe(), 8192);
        let b = Atlas::build(recipe(), 8192);
        assert_eq!((a.width, a.height), (b.width, b.height));
        assert_eq!(a.pixels, b.pixels, "two runs must pack bit-identically");
        for (name, rect) in &a.sheets {
            let other = b.sheets.get(name).expect("same sheet set");
            assert_eq!(
                (rect.x, rect.y, rect.width, rect.height),
                (other.x, other.y, other.width, other.height),
                "{name} placement"
            );
        }
    }

    /// UV correctness: every sheet's pixels must land exactly at its
    /// `SheetRect` (the draw code samples through those rects), and the rect
    /// must stay inside the atlas with the PAD border.
    #[test]
    fn skyline_copies_every_sheet_to_its_rect() {
        let images = vec![
            sheet("Tiles_1", 288, 270, 11),
            sheet("Background_0", 48, 1400, 22),
            sheet("Item_71", 20, 20, 33),
            sheet("Gore_910", 512, 128, 44),
            sheet("water_0", 48, 1360, 55),
        ];
        let atlas = Atlas::build_ref(&images, 1024);
        for img in &images {
            let rect = atlas.sheet(&img.name).expect("sheet registered");
            assert_eq!((rect.width, rect.height), (img.width, img.height));
            assert!(rect.x >= PAD && rect.y >= PAD, "{} keeps the border", img.name);
            assert!(rect.x + rect.width <= atlas.width - PAD, "{} width", img.name);
            assert!(rect.y + rect.height <= atlas.height - PAD, "{} height", img.name);
            for row in 0..img.height as usize {
                let src = row * img.width as usize * 4;
                let dst = ((rect.y as usize + row) * atlas.width as usize + rect.x as usize) * 4;
                assert_eq!(
                    &atlas.pixels[dst..dst + img.width as usize * 4],
                    &img.rgba[src..src + img.width as usize * 4],
                    "{} row {row} must match the source",
                    img.name
                );
            }
        }
    }

    /// No two sheets may overlap — checked on the PAD-inflated rects, so the
    /// padding reservation itself is part of the proof (bilinear bleeding
    /// must stay impossible).
    #[test]
    fn skyline_sheets_never_overlap_including_padding() {
        let images: Vec<AtlasImage> = (0..40u32)
            .map(|i| {
                sheet(
                    &format!("S_{i}"),
                    8 + (i * 7) % 60,
                    8 + (i * 13) % 120,
                    i as u8,
                )
            })
            .collect();
        let atlas = Atlas::build_ref(&images, 512);
        let rects: Vec<(u32, u32, u32, u32)> = images
            .iter()
            .map(|img| {
                let r = atlas.sheet(&img.name).unwrap();
                (r.x, r.y, r.width + PAD, r.height + PAD)
            })
            .collect();
        for i in 0..rects.len() {
            for j in (i + 1)..rects.len() {
                let (ax, ay, aw, ah) = rects[i];
                let (bx, by, bw, bh) = rects[j];
                assert!(
                    ax + aw <= bx || bx + bw <= ax || ay + ah <= by || by + bh <= ay,
                    "sheets {} and {} overlap (padding included)",
                    images[i].name,
                    images[j].name
                );
            }
        }
    }

    /// The backfill witness: a short sheet must land ON the shorter sheet's
    /// column (in the space beside the tall one) instead of opening a new
    /// row — the exact behaviour the shelf packer lacked (its stranded tails
    /// overflowed 8192 with the gore set). Hand-computable: bin 100, PAD 2,
    /// usable width 98.
    ///   Tall 40x80 → rect (42,82) at bin (0,0)   → atlas (2,2);
    ///   Wide 54x20 → rect (56,22) at bin (42,0)  → atlas (44,2), filling
    ///                the row exactly (42+56 = 98);
    ///   Small 30x10 → rect (32,12): x=0 gives y=82, x=42 gives y=22 —
    ///                bottom-left picks (42,22) → atlas (44,24).
    /// Height = 82 + PAD = 84; the shelf pass opens a third row (96 px).
    #[test]
    fn skyline_backfills_the_tall_sheets_tail() {
        let images = vec![
            sheet("Tall", 40, 80, 1),
            sheet("Wide", 54, 20, 2),
            sheet("Small", 30, 10, 3),
        ];
        let atlas = Atlas::build_ref(&images, 100);
        let tall = atlas.sheet("Tall").unwrap();
        let wide = atlas.sheet("Wide").unwrap();
        let small = atlas.sheet("Small").unwrap();
        assert_eq!((tall.x, tall.y), (PAD, PAD));
        assert_eq!((wide.x, wide.y), (PAD + 40 + PAD, PAD));
        assert_eq!(
            (small.x, small.y),
            (PAD + 40 + PAD, PAD + 20 + PAD),
            "the short sheet backfills beside the tall one, on the wide one"
        );
        assert_eq!(atlas.height, 84, "no third row: 2 + 80 + 2");
    }

    /// Name dedup: a sheet decoded twice (the 41 `Projectile_*` names that
    /// ride both `STATIC_SHEET_NAMES` and the `proj_data` loop) packs once —
    /// the FIRST occurrence wins. Both copies are the same file decoded
    /// twice, so the pixels are identical; the old packer's last-write-wins
    /// rect pointed at an identical copy, so every lookup samples the same
    /// texels. (The test uses different seeds to prove WHICH copy is kept.)
    #[test]
    fn duplicate_names_pack_once_first_wins() {
        let images = vec![
            sheet("Projectile_3", 22, 22, 7),
            sheet("Item_2", 16, 16, 1),
            sheet("Projectile_3", 22, 22, 9), // the second decode, different pixels
        ];
        let atlas = Atlas::build_ref(&images, 512);
        assert_eq!(atlas.sheets.len(), 3, "two names + the synthetic __white");
        let rect = atlas.sheet("Projectile_3").unwrap();
        // the FIRST copy's pixels (seed 7) must be at the rect
        let first = &images[0];
        for row in 0..first.height as usize {
            let src = row * first.width as usize * 4;
            let dst = ((rect.y as usize + row) * atlas.width as usize + rect.x as usize) * 4;
            assert_eq!(
                &atlas.pixels[dst..dst + first.width as usize * 4],
                &first.rgba[src..src + first.width as usize * 4],
                "the first decode's pixels win"
            );
        }
    }

    /// The stress shape of the real failure: one tall band + ~1400 small
    /// same-band sheets (the gore set's shape) must fit a width-8192 bin well
    /// under the 8192 height bound. The shelf packer stranded a width tail
    /// per row here; the skyline backfills them. (The REAL-content proof with
    /// the actual gore+mount dims lives in assets.rs's
    /// `the_full_gore_and_mount_sheet_set_fits_the_strict_gpu_bound`.)
    #[test]
    fn skyline_fits_the_gore_shaped_stress_set() {
        let mut images = vec![
            sheet("Background_0", 48, 1400, 1),
            sheet("water_0", 48, 1360, 2),
            sheet("Tiles_0", 288, 270, 3),
        ];
        for i in 0..1400u32 {
            images.push(sheet(&format!("Gore_{i}"), 16 + i % 8, 16 + (i / 8) % 8, i as u8));
        }
        let atlas = Atlas::build_ref(&images, 8192);
        assert!(
            atlas.height <= 8192,
            "the gore-shaped set must fit: {} px",
            atlas.height
        );
        assert_eq!(atlas.sheets.len(), images.len() + 1, "+ __white");
    }
}
