//! W4-g28 — the `Draw` pass's quality-light tier thresholds
//! (`TileDrawing.cs:479-495`): the per-channel byte requirements that
//! `DrawSingleTile_SlicedBlock` compares `drawData.tileLight` against —
//! above the HIGH tier a block draws as nine light slices
//! (`TileDrawing.cs:3524`), above the MEDIUM tier as four (`:3605`),
//! below it one flat quad (`:3663`).
//!
//! The C# rebuilds both colours once per `Draw` call (twice per frame,
//! solid + non-solid layers) from `Main.gfxQuality`:
//! * high `num = 255*(1-q) + 30*q`, medium `num2 = 50*(1-q) + 2*q`;
//! * `R = (byte)num`, `G = (byte)((double)num * 1.1)`,
//!   `B = (byte)((double)num * 1.2)` — the G/B factors ride DOUBLE
//!   precision before the truncating byte cast;
//! * `DebugOptions.devLightTilesCheat` forces both tiers to
//!   `byte.MaxValue` (`:487-495`) so every block takes the flat arm.
//!
//! The comparison input is `drawData.tileLight` = `Lighting.GetColor`
//! (`TileDrawing.cs:871`), whose channels are the TRUNCATED
//! `(int)(engineColor * GlobalBrightness * 255)` bytes clamped at 255
//! (`Lighting.cs:224-238`) — so the tier test is integer-on-bytes, not a
//! float threshold (research/09 deviation 11: the port previously
//! compared raw floats against 142/255 and 26/255 constants pinned at
//! gfxQuality 0.5).

/// `TileDrawing._highQualityLightingRequirement` /
/// `_mediumQualityLightingRequirement` (`TileDrawing.cs:146-148`),
/// rebuilt per `Draw` call (`:479-495`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct QualityLightTiers {
    /// High tier channels; above any of them → the 9-slice arm.
    pub high: [u8; 3],
    /// Medium tier channels; above any of them → the 4-slice arm.
    pub medium: [u8; 3],
}

/// `TileDrawing.Draw`'s threshold build (`TileDrawing.cs:479-495`).
/// `gfx_quality` is `Main.gfxQuality` (the port's adaptive loop,
/// `world_sim::gfx_quality`); `dev_light_tiles_cheat` is
/// `DebugOptions.devLightTilesCheat` (`:487`, no port toggle — fed
/// `false`).
pub fn quality_light_tiers(gfx_quality: f32, dev_light_tiles_cheat: bool) -> QualityLightTiers {
    // TileDrawing.cs:479-486 — the two byte formulas. The C# stores
    // `gfxQuality` as a float, so `num`/`num2` are f32 expressions; only
    // the ×1.1 / ×1.2 channel factors are computed in double.
    let num = 255.0f32 * (1.0 - gfx_quality) + 30.0f32 * gfx_quality;
    let num2 = 50.0f32 * (1.0 - gfx_quality) + 2.0f32 * gfx_quality;
    // The `(byte)` cast is UNCHECKED (the decompiled target is net40,
    // `Terraria.csproj:7`): float→byte lowers to conv.i4 (truncate toward
    // zero) then conv.u1 (keep the low byte), so an out-of-range product
    // WRAPS — at q = 0 the high G channel is (byte)280.5 = 24 and B is
    // (byte)306 = 50, NOT 255. Rust's `as u8` saturates instead, so the
    // wrap is spelled out via i32. (Pre-.NET 11 CoreCLR byte casts wrap
    // the same way; the .NET 9/11 saturation change postdates the game.)
    let channels = |n: f32| {
        [
            (n as i32) as u8,
            ((n as f64 * 1.1) as i32) as u8,
            ((n as f64 * 1.2) as i32) as u8,
        ]
    };
    let mut tiers = QualityLightTiers {
        high: channels(num),
        medium: channels(num2),
    };
    // TileDrawing.cs:487-495 — the dev cheat pins both tiers to 255 so
    // no tile light can exceed them (every block draws flat).
    if dev_light_tiles_cheat {
        tiers.high = [u8::MAX; 3];
        tiers.medium = [u8::MAX; 3];
    }
    tiers
}

/// `Lighting.GetColor(x, y)`'s channel bytes (`Lighting.cs:222-238`):
/// `(int)(engineColor * GlobalBrightness * 255)` per channel, clamped at
/// 255. This is `drawData.tileLight` (`TileDrawing.cs:871`) — the tier
/// test's input; the actuation ghost (`GetFinalLight`) lands on
/// `finalColor`, never on `tileLight`.
pub fn tile_light_bytes(raw: [f32; 3], global_brightness: f32) -> [u8; 3] {
    // Lighting.cs:224 — one float product, then the truncating cast.
    let num = global_brightness * 255.0;
    let ch = |v: f32| ((v * num) as i32).clamp(0, 255) as u8;
    [ch(raw[0]), ch(raw[1]), ch(raw[2])]
}

impl QualityLightTiers {
    /// `DrawSingleTile_SlicedBlock`'s arm pick (`TileDrawing.cs:3524` /
    /// `:3605` / `:3661`): per-channel `tileLight.X > requirement.X` —
    /// ANY channel above the tier takes the finer arm.
    pub fn slice_arm(&self, tile_light: [u8; 3]) -> SliceArm {
        let above = |req: [u8; 3]| {
            tile_light[0] > req[0] || tile_light[1] > req[1] || tile_light[2] > req[2]
        };
        if above(self.high) {
            SliceArm::Nine
        } else if above(self.medium) {
            SliceArm::Four
        } else {
            SliceArm::Flat
        }
    }
}

/// The `DrawSingleTile_SlicedBlock` arm (`TileDrawing.cs:3517-3665`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SliceArm {
    /// The 9-slice 4/8/4 grid (`:3524-3604`).
    Nine,
    /// The 4-slice 8×8 grid (`:3605-3660`).
    Four,
    /// One flat quad with `finalColor` (`:3661-3664`).
    Flat,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Witness: the byte formulas at the quality endpoints and the old
    /// pinned default (`TileDrawing.cs:479-486`). q = 0.5 reproduces the
    /// pre-port constants' inputs: num = 142.5 → R 142, G (double)156.75
    /// → 156, B 171.0 → 171; num2 = 26 → 26 / 28 / 31. The old port
    /// compared only R-tier floats (142/255, 26/255) — the G/B channels
    /// were silently ignored.
    #[test]
    fn tier_bytes_at_quality_zero_half_one() {
        let q0 = quality_light_tiers(0.0, false);
        // q = 0: num = 255 exactly, but G = (double)255 * 1.1 = 280.5 and
        // B = 306 overflow the byte — the net40 unchecked cast WRAPS
        // (conv.i4 → conv.u1): 280 & 0xFF = 24, 306 & 0xFF = 50. So at
        // the lowest quality the high tier's G/B gates are LOWER than R
        // (the vanilla quirk the saturating `as u8` would have erased).
        assert_eq!(q0.high, [255, 24, 50]);
        assert_eq!(q0.medium, [50, 55, 60]);
        let qh = quality_light_tiers(0.5, false);
        assert_eq!(qh.high, [142, 156, 171]);
        assert_eq!(qh.medium, [26, 28, 31]);
        let q1 = quality_light_tiers(1.0, false);
        assert_eq!(q1.high, [30, 33, 36]);
        assert_eq!(q1.medium, [2, 2, 2]); // 2.2 → 2, 2.4 → 2
    }

    /// Witness: the G/B factors ride DOUBLE precision (`:481-482` /
    /// `:485-486`). At q = 0.5 the high R byte is 142 (from 142.5f);
    /// G = 142.5 * 1.1 = 156.75 → 156 either way, but the medium tier's
    /// B at q = 1 is 2 * 1.2 = 2.4 → 2 while a naive `(byte)(n * 1.2f)`
    /// with n already truncated stays equal — the pinned pair below
    /// distinguishes the (double)num path from truncating num first:
    /// num = 142.5 → (byte)(142.5 * 1.1) = 156, but (byte)142 * 1.1 =
    /// 156.2 → 156 too; B differs at other q — pin q = 0.25:
    /// num = 198.75 → B = 238.5 → 238; truncated-first 198 * 1.2 =
    /// 237.6 → 237.
    #[test]
    fn gb_factors_use_the_untruncated_double_product() {
        let q = quality_light_tiers(0.25, false);
        // num = 255*0.75 + 30*0.25 = 198.75
        assert_eq!(q.high, [198, 218, 238]);
        // (double)198.75 * 1.1 = 218.625 → 218; * 1.2 = 238.5 → 238.
        // Truncating num to 198 first would give 217 / 237.
        assert_ne!(q.high[2], ((198.0f64) * 1.2) as u8);
    }

    /// Witness: `DebugOptions.devLightTilesCheat` pins both tiers to
    /// byte.MaxValue (`TileDrawing.cs:487-495`) — every light draws flat.
    #[test]
    fn the_dev_cheat_pins_both_tiers_to_255() {
        let t = quality_light_tiers(0.5, true);
        assert_eq!(t.high, [255, 255, 255]);
        assert_eq!(t.medium, [255, 255, 255]);
        assert_eq!(t.slice_arm([255, 255, 255]), SliceArm::Flat);
    }

    /// Witness: `Lighting.GetColor`'s byte quantisation
    /// (`Lighting.cs:224-238`) — GlobalBrightness 1.2, truncation toward
    /// zero, clamp at 255. raw 0.5 → (int)(0.5 * 306) = 153; the f32
    /// product 0.1f * 306f = 30.6000004 truncates to 30 (the cast never
    /// rounds up).
    #[test]
    fn light_bytes_truncate_and_clamp() {
        assert_eq!(tile_light_bytes([0.5, 0.5, 0.5], 1.2), [153, 153, 153]);
        assert_eq!(tile_light_bytes([1.0, 2.0, 0.0], 1.2), [255, 255, 0]);
        let b = tile_light_bytes([0.1, 0.1, 0.1], 1.2);
        assert_eq!(b, [30, 30, 30]);
        // GlobalBrightness 1.0 (the blind debuff, Lighting.cs:114).
        assert_eq!(tile_light_bytes([0.5, 0.5, 0.5], 1.0), [127, 127, 127]);
    }

    /// Witness: the per-channel OR comparison (`TileDrawing.cs:3524` /
    /// `:3605`) — ANY channel above the tier picks the finer arm; equal
    /// bytes do NOT (strict `>`).
    #[test]
    fn the_arm_pick_is_a_strict_per_channel_or() {
        let t = quality_light_tiers(0.5, false); // high 142/156/171, medium 26/28/31
        // All channels at the tier: flat/4 boundary is strict.
        assert_eq!(t.slice_arm([26, 28, 31]), SliceArm::Flat);
        assert_eq!(t.slice_arm([27, 28, 31]), SliceArm::Four);
        // B alone above the medium tier (the old R-only float gate
        // missed this): 32 > 31.
        assert_eq!(t.slice_arm([0, 0, 32]), SliceArm::Four);
        // At the high tier exactly: still four.
        assert_eq!(t.slice_arm([142, 156, 171]), SliceArm::Four);
        // G alone above high: nine (156 → 157).
        assert_eq!(t.slice_arm([0, 157, 0]), SliceArm::Nine);
        // B alone above high: nine.
        assert_eq!(t.slice_arm([0, 0, 172]), SliceArm::Nine);
    }

    /// Witness: the adaptive quality moves the tiers monotonically —
    /// higher gfxQuality lowers the requirements (more sliced blocks).
    #[test]
    fn higher_quality_lowers_the_requirements() {
        let lo = quality_light_tiers(0.2, false); // high 210/231/252, medium 40/44/48
        let hi = quality_light_tiers(0.8, false); // high 75/82/90, medium 11/12/13
        for k in 0..3 {
            assert!(hi.high[k] < lo.high[k]);
            assert!(hi.medium[k] < lo.medium[k]);
        }
        // A light that draws four slices at low quality slices nine at
        // high quality: 80 > lo.medium (40/44/48) but ≤ lo.high
        // (210/231/252); 80 > hi.high.R (75).
        let l = [80, 80, 80];
        assert_eq!(lo.slice_arm(l), SliceArm::Four);
        assert_eq!(hi.slice_arm(l), SliceArm::Nine);
    }
}
