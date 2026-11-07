//! Runtime readers for Terraria's original `.xnb` content, ported from the
//! original pipeline (`tools/XnbExtract` for the container, `decompiled-libs/ReLogic`
//! for `DynamicSpriteFontReader`).

pub mod adpcm;
pub mod dxt;
pub mod effect;
pub mod font;
pub mod lzx;
pub mod sound;
pub mod texture;
pub mod xact;
pub mod xnb;

pub use adpcm::decode_msadpcm;
pub use adpcm::AdpcmBlockInfo;
pub use effect::load_effect_pass_names;
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

    /// FNV of the decoded `Images/Misc/noise` RGBA (pinned by
    /// `test_DW_2_3_noise_xnb_decodes` against the retail `.xnb`).
    const NOISE_HASH: u64 = 0xf3ed_720d_4478_1edd;

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

    /// Pins the tile-flame and tile-glow sheet sizes decoded from the retail
    /// `.xnb`s (`TILE-DRAW-2`): `TextureAssets.Flames` (`Images/Flame_0..17`,
    /// `AssetInitializer.cs:382-384`) and the glow sheets
    /// `DrawSingleTile_Flames` draws (`Images/Glow_{id}` per
    /// `AssetInitializer.cs:352-354`, plus the named jar/timer sheets). The
    /// flame pass reads frame coordinates straight off the tile into these
    /// sheets (`TileDrawing.cs:2722`, `:2832`), so the decoded sizes are the
    /// pin that keeps the port's frame math and the shipped assets from
    /// drifting apart — e.g. the torch sheet 132x528 holds the 22 px frame
    /// grid six columns wide, and the moondial `Glow_335` 36x432 holds eight
    /// 54 px moon-phase rows.
    /// DW-2.3: `Images/Misc/noise` — the noise image every M1 screen filter binds
    /// (`ScreenEffectInitializer.cs:30-40`) — decodes at its retail size with the
    /// pixels pinned by FNV (the `decodes_tiles_deterministically` pattern).
    /// `ScreenShaderData.cs:175-177` gives it LinearWrap because 256x256 is POT.
    #[test]
    fn test_DW_2_3_noise_xnb_decodes() {
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../assets/Content/Images/Misc/noise.xnb"
        );
        let img = match crate::load_texture(std::path::PathBuf::from(path)) {
            Ok(img) => img,
            Err(_) => {
                eprintln!("noise.xnb not present; skipping");
                return;
            }
        };
        assert_eq!((img.width, img.height), (256, 256));
        let mut h: u64 = 0xcbf2_9ce4_8422_2325;
        for b in &img.pixels {
            h ^= *b as u64;
            h = h.wrapping_mul(0x100_0000_01b3);
        }
        println!("noise fnv {h:016x}");
        assert_eq!(h, NOISE_HASH);
    }

    #[test]
    fn decodes_flame_and_glow_sheets_at_the_pinned_sizes() {
        let Some(dir) = std::env::var_os("TR_CONTENT") else { return };
        let dir = std::path::PathBuf::from(dir);
        let cases: &[(&str, u32, u32)] = &[
            ("Flame_0", 132, 528),
            ("Flame_1", 36, 1412),
            ("Flame_2", 142, 2016),
            ("Flame_3", 214, 2000),
            ("Flame_4", 70, 2048),
            ("Flame_5", 18, 22),
            ("Flame_6", 34, 34),
            ("Flame_7", 72, 324),
            ("Flame_8", 86, 248),
            ("Flame_9", 54, 378),
            ("Flame_10", 54, 216),
            ("Flame_11", 54, 270),
            ("Flame_12", 54, 648),
            ("Flame_13", 70, 2014),
            ("Flame_14", 106, 72),
            ("Flame_15", 864, 324),
            ("Flame_16", 18, 22),
            ("Flame_17", 18, 22),
            ("Glow_131", 54, 684),
            ("Glow_291", 108, 36),
            ("Glow_293", 54, 756),
            ("Glow_295", 18, 234),
            ("Glow_296", 36, 468),
            ("Glow_297", 486, 108),
            ("Glow_298", 108, 90),
            ("Glow_299", 324, 108),
            ("Glow_300", 54, 684),
            ("Glow_325", 36, 54),
            ("Glow_334", 48, 58),
            ("Glow_335", 36, 432),
            ("FireflyJar", 108, 36),
            ("LightningBugJar", 108, 36),
            ("XmasLight", 108, 72),
            ("Timer", 90, 36),
            ("SunAltar", 54, 36),
            ("jellyfishBowl1", 36, 360),
            ("jellyfishBowl2", 36, 360),
            ("jellyfishBowl3", 36, 360),
        ];
        for (name, w, h) in cases {
            let img = crate::load_texture(dir.join(format!("Images/{name}.xnb")))
                .unwrap_or_else(|e| panic!("{name}: {e}"));
            assert_eq!((img.width, img.height), (*w, *h), "{name}");
        }
    }
}

#[cfg(test)]
mod tmp_dump {
    #[test]
    fn dump_flame0_tiles4_rgba() {
        let Some(dir) = std::env::var_os("TR_CONTENT") else { return };
        let dir = std::path::PathBuf::from(dir);
        // The scratch directory this debug dump targets is not part of the
        // repo; skip (instead of failing) wherever it has not been created.
        let out_dir = std::path::Path::new(r"C:\Users\DreadLord\Desktop\TR-tile2-tmp");
        if !out_dir.is_dir() {
            eprintln!("skipping: {} does not exist", out_dir.display());
            return;
        }
        for (name, out) in [("Flame_0", "flame0.rgba"), ("Tiles_4", "tiles4.rgba")] {
            let img = crate::load_texture(dir.join(format!("Images/{name}.xnb"))).unwrap();
            std::fs::write(out_dir.join(out), &img.pixels).unwrap();
            println!("{name} {}x{}", img.width, img.height);
        }
    }
}
