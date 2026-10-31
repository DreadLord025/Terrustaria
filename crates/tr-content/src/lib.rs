//! Runtime readers for Terraria's original `.xnb` content, ported from the
//! original pipeline (`tools/XnbExtract` for the container, `decompiled-libs/ReLogic`
//! for `DynamicSpriteFontReader`).

pub mod adpcm;
pub mod dxt;
pub mod font;
pub mod lzx;
pub mod sound;
pub mod texture;
pub mod xact;
pub mod xnb;

pub use adpcm::decode_msadpcm;
pub use adpcm::AdpcmBlockInfo;
pub use font::{load_font, DynamicSpriteFont, FontPage, Glyph};
pub use sound::{load_sound, Sound};
pub use texture::{load_texture, RgbaImage};
pub use xact::{
    parse_sound_bank, parse_wave_bank_entries, parse_wave_bank_entry, Codec, MiniFormat,
    XactSoundBank, XactWaveBank, XactWaveBankEntry,
};

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

    /// Pins the P4 minimap/boss-bar sheet sizes decoded from the retail
    /// `.xnb`s (the whole-UI P4 research §C/§H): the Default frame 256x264,
    /// the three Default buttons 18x18 and `UI_BossBar` 516x348. The geometry
    /// in `tr-app/src/ui/minimap.rs` hard-codes no sheet size — it reads
    /// `SheetRect::width/height` like `Button.Size` does (`MinimapFrame.cs:24`)
    /// — so this is the runtime pin that the geometry tables and the shipped
    /// assets cannot drift apart.
    #[test]
    fn decodes_minimap_and_boss_bar_sheets_at_the_pinned_sizes() {
        let Some(dir) = std::env::var_os("TR_CONTENT") else { return };
        let dir = std::path::PathBuf::from(dir);
        let frame = crate::load_texture(dir.join("Images/UI/Minimap/Default/MinimapFrame.xnb"))
            .expect("MinimapFrame");
        assert_eq!((frame.width, frame.height), (256, 264));
        for name in ["MinimapButton_Reset", "MinimapButton_ZoomIn", "MinimapButton_ZoomOut"] {
            let img = crate::load_texture(dir.join(format!("Images/UI/Minimap/Default/{name}.xnb")))
                .unwrap_or_else(|e| panic!("{name}: {e}"));
            assert_eq!((img.width, img.height), (18, 18), "{name}");
        }
        let boss = crate::load_texture(dir.join("Images/UI/UI_BossBar.xnb")).expect("UI_BossBar");
        assert_eq!((boss.width, boss.height), (516, 348));
    }

    /// Pins the P6 front-end sheet sizes decoded from the retail `.xnb`s
    /// (the whole-UI P6 plan's Uncertainty-1 table, load-bearing rows). The
    /// card/creation geometry reads `SheetRect::width/height` at runtime like
    /// `UIImageButton.cs:30-33` does, so this is the pin that keeps the
    /// geometry tables and the shipped assets from drifting apart.
    #[test]
    fn decodes_p6_front_end_sheets_at_the_pinned_sizes() {
        let Some(dir) = std::env::var_os("TR_CONTENT") else { return };
        let dir = std::path::PathBuf::from(dir);
        let cases: &[(&str, u32, u32)] = &[
            ("UI/ButtonPlay", 22, 22),
            ("UI/PlayerBackground", 59, 58),
            ("UI/InnerPanelBackground", 24, 27),
            ("UI/Divider", 8, 4),
            ("UI/IconPlayedBefore", 25, 28),
            ("UI/CharCreation/CategoryPanel", 44, 44),
            ("UI/CharCreation/SmallPanel", 32, 32),
            ("UI/CharCreation/PanelGrayscale", 44, 44),
            ("UI/WorldCreation/PreviewBorder", 84, 84),
            ("UI/WorldCreation/PreviewDifficultyNormal1", 76, 76),
            ("UI/VK_Shift", 26, 26),
            ("UI/VK_Backspace", 54, 26),
        ];
        for (name, w, h) in cases {
            let img = crate::load_texture(dir.join(format!("Images/{name}.xnb")))
                .unwrap_or_else(|e| panic!("{name}: {e}"));
            assert_eq!((img.width, img.height), (*w, *h), "{name}");
        }
    }
}
