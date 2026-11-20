//! `Main.tileGlowMask` — the per-tile-type glow-mask sheet table
//! (`Main.cs:1435` decl, `:8437` the −1 reset loop, `:66897` the
//! `new short[TileID.Count]` allocation, `:10024-10068` the assignments).
//!
//! Consumer census (W4-g35): `TileDrawing.DrawSingleTile`
//! (`TileDrawing.cs:1178-1290`) reads the table AFTER `DrawBasicTile`:
//! `!= -1` opens the glow block, `TextureAssets.GlowMask.IndexInRange(num8)`
//! swaps `drawTexture` to the glow sheet (the −2 sentinel on tile 129 fails
//! the range check and keeps the tile's own texture), then a per-type colour
//! switch (the moss colours `_lavaMossGlow`/`_kryptonMossGlow`/`_xenonMossGlow`/
//! `_argonMossGlow`/`_violetMossGlow`, the disco 627/628/692, the shimmer
//! glitter 659/667/708, the portal 209, the meteor 370/390, the lamp 350
//! cosine, the lava 717 `LavaLightA`, the 129 four-copy hsl ring, the 725
//! Noir-filter arm) and the slope/half-brick draw ladder (`:1291-1339`).
//! The `:971` flag gate (`glowTexture != null || tileGlowMask != -1 ||
//! tileFlame`) keeps glow tiles out of the black-cell merge.
//!
//! The TEXTURE BINDING + colour switch + draw ladder are TILE-DRAW-2
//! (task W4-f15); this module is the record table that binding reads.
//! Two dead colour arms are recorded for the census: `case 391`
//! (`TileDrawing.cs:1247`) and `case 718` (`:1190`) have colour arms but NO
//! `tileGlowMask` assignment anywhere in `Main.cs` — the `!= -1` gate never
//! opens for them (verbatim source state, not a port gap).
//!
//! The `DrawSingleTile_Flames` per-type glow branches (`TileDrawing.cs:
//! 2684-2825`) are a SEPARATE mechanism already ported in `flame.rs`
//! (`glow_quads`); they do not read this table.

/// One `Main.tileGlowMask[type] = mask` assignment, in source order
/// (`Main.cs:10024-10068`). The value is the C# `short` verbatim: a
/// `TextureAssets.GlowMask` index (`Images/Glow_{id}`,
/// `AssetInitializer.cs:352-354`), or the −2 sentinel (tile 129, `:10048` —
/// `IndexInRange` fails, the glow block keeps the painted tile texture).
pub const TILE_GLOW_RECORDS: &[(u16, i16)] = &[
    (350, 94),   // :10024 — MartianConduitPlating
    (390, 130),  // :10025 — LavaLamp
    (381, 126),  // :10026 — LavaMoss
    (517, 258),  // :10027 — LavaMossBrick
    (687, 336),  // :10028 — LavaMossBlock
    (534, 259),  // :10029 — KryptonMoss
    (535, 260),  // :10030 — KryptonMossBrick
    (689, 338),  // :10031 — KryptonMossBlock
    (536, 261),  // :10032 — XenonMoss
    (537, 262),  // :10033 — XenonMossBrick
    (690, 339),  // :10034 — XenonMossBlock
    (539, 263),  // :10035 — ArgonMoss
    (540, 264),  // :10036 — ArgonMossBrick
    (688, 337),  // :10037 — ArgonMossBlock
    (625, 311),  // :10038 — VioletMoss
    (626, 312),  // :10039 — VioletMossBrick
    (691, 340),  // :10040 — VioletMossBlock
    (627, 313),  // :10041 — RainbowMoss (disco colour, TileDrawing.cs:1235)
    (628, 314),  // :10042 — RainbowMossBrick (disco colour)
    (692, 341),  // :10043 — RainbowMossBlock (disco colour)
];

/// `Main.tileGlowMask[type]` (`Main.cs:10024-10068`): the glow-mask sheet
/// index, or −1 (the `:8437` reset stock) for every unlisted type. Tile
/// ids beyond the table keep −1 like the C# array's untouched cells.
pub fn tile_glow_mask(kind: u16) -> i16 {
    for &(t, m) in TILE_GLOW_RECORDS {
        if t == kind {
            return m;
        }
    }
    -1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn w4_g35_the_reset_stock_is_minus_one() {
        // `Main.cs:8437` — every cell starts −1; unlisted types stay dark.
        assert_eq!(tile_glow_mask(0), -1);
        assert_eq!(tile_glow_mask(1), -1);
        // 391/718 have colour arms in TileDrawing but NO table assignment.
        assert_eq!(tile_glow_mask(391), -1);
        assert_eq!(tile_glow_mask(718), -1);
    }

    #[test]
    fn w4_g35_the_landed_records_match_the_source() {
        assert_eq!(tile_glow_mask(350), 94); // :10024
        assert_eq!(tile_glow_mask(537), 262); // :10033
        assert_eq!(tile_glow_mask(692), 341); // :10043
        assert_eq!(tile_glow_mask(627), 313); // :10041
    }
}
