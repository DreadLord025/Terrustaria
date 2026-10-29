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
    /// FNV-1a over the decoded RGBA of `Images/Tiles_0.xnb` from the retail 1.4.5.8
    /// copy. This pins the runtime decoder to the pixels verified against the
    /// original extraction (0-diff before the PNGs were removed).
    #[test]
    fn decodes_tiles_deterministically() {
        let Some(dir) = std::env::var_os("TR_CONTENT") else { return };
        let img = crate::load_texture(std::path::PathBuf::from(dir).join("Images/Tiles_0.xnb"))
            .expect("Tiles_0");
        assert_eq!((img.width, img.height), (288, 270));
        let mut h: u64 = 0xcbf2_9ce4_8422_2325;
        for b in &img.pixels {
            h ^= *b as u64;
            h = h.wrapping_mul(0x100_0000_01b3);
        }
        println!("Tiles_0 fnv {h:016x}");
        assert_eq!(h, HASH);
    }

    const HASH: u64 = 0xdb76_2213_37e7_e205;
}
