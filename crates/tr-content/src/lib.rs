//! Runtime readers for Terraria's original `.xnb` content, ported from the
//! original pipeline (`tools/XnbExtract` for the container, `decompiled-libs/ReLogic`
//! for `DynamicSpriteFontReader`).

pub mod dxt;
pub mod font;
pub mod lzx;
pub mod texture;
pub mod xnb;

pub use font::{load_font, DynamicSpriteFont, FontPage, Glyph};
pub use texture::{load_texture, RgbaImage};

#[cfg(test)]
mod tests {
    /// The runtime decoder must match the already-verified extracted PNGs.
    #[test]
    fn decodes_tiles_pixel_identically_to_the_png() {
        let Some(dir) = std::env::var_os("TR_CONTENT") else { return };
        let img = match crate::load_texture(std::path::PathBuf::from(dir).join("Images/Tiles_0.xnb")) {
            Ok(v) => v,
            Err(e) => panic!("Tiles_0 decode failed: {e:#}"),
        };
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let png = image::open(root.join("assets/Images/Tiles_0.png")).unwrap().to_rgba8();
        assert_eq!((img.width, img.height), (png.width(), png.height()));
        let mut diff = 0usize;
        let mut swapped = 0usize;
        for (a, b) in img.pixels.chunks_exact(4).zip(png.pixels()) {
            if a[0..3] != b.0[0..3] {
                diff += 1;
                if a[2] == b.0[0] && a[0] == b.0[2] {
                    swapped += 1;
                }
            }
        }
        println!("Tiles_0: {diff} differing pixels, {swapped} look channel-swapped");
        assert_eq!(diff, 0, "decoded texture differs from the PNG");
    }
}
