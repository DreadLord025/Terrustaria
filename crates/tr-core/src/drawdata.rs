//! `TileDrawing.GetTileDrawData` subset: per-type sprite size/top/flip.
//!
//! Ported from `Terraria.GameContent.Drawing.TileDrawing.GetTileDrawData`
//! (`decompiled/Terraria/GameContent/Drawing/TileDrawing.cs:4527`, type switch
//! at 4540-6032). Only the per-type metrics are carried here; the global
//! `addFrY = Main.tileFrame[type] * 38` default (line 4533) and every branch
//! that needs the tile frame row / world state stay at their defaults and are
//! marked `TODO(app)`.

/// `GetTileDrawData` metrics needed by the tile pass.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TileDrawData {
    pub width: u16,
    pub height: u16,
    /// Vertical offset of the sprite above the cell origin (`tileTop`).
    pub top: i16,
    /// Added to the source rectangle X/Y (`addFrX`/`addFrY`).
    pub add_fr_x: i16,
    pub add_fr_y: i16,
    /// `SpriteEffects.FlipHorizontally`.
    pub flip: bool,
}

impl Default for TileDrawData {
    /// `GetTileDrawData` starts from `tileWidth = tileHeight = 16`,
    /// `tileTop = 0`, `addFrX = addFrY = 0`, `SpriteEffects.None`
    /// (`TileDrawing.cs:4529-4535`).
    fn default() -> Self {
        Self {
            width: 16,
            height: 16,
            top: 0,
            add_fr_x: 0,
            add_fr_y: 0,
            flip: false,
        }
    }
}

/// Per-type overrides of `TileDrawing.GetTileDrawData`.
///
/// `kind` is `Tile.type`, `x` the tile column (used for the `x % 2 == 0`
/// sprite flip), and `solid_above` is `WorldGen.SolidTile(x, y - 1)` (used by
/// torches). Frame- and world-dependent fields stay at [`TileDrawData::default`].
pub fn draw_data(kind: u16, x: i32, solid_above: bool) -> TileDrawData {
    let mut data = TileDrawData::default();
    // C#: `if (x % 2 == 0) tileSpriteEffect = SpriteEffects.FlipHorizontally;`
    let flip_x = x % 2 == 0;

    match kind {
        // Torches: 20x20, tileTop 4 when the tile above is solid
        // (`TileDrawing.cs:4672-4679`).
        4 => {
            data.width = 20;
            data.height = 20;
            if solid_above {
                data.top = 4;
            }
        }
        // Plants / corruption / jungle / mushroom / hallowed / crimson /
        // ash / echo plants: 20 px tall, flip on even columns
        // (`TileDrawing.cs:4850-4863`).
        3 | 24 | 61 | 71 | 110 | 201 | 637 | 703 => {
            data.height = 20;
            data.flip = flip_x;
        }
        // Corrupt thorns / hellforge: 18 px tall
        // (`TileDrawing.cs:4880-4908`, 4886-4887).
        32 | 77 => {
            data.height = 18;
        }
        // Plants2 / JunglePlants2: 32 px tall, raised 12 px, flip on even
        // columns (`TileDrawing.cs:5027-5036`).
        73 | 74 => {
            data.height = 32;
            data.top = -12;
            data.flip = flip_x;
        }
        // Immature / mature / blooming herbs: 20 px tall, raised 2 px,
        // flip on even columns (`TileDrawing.cs:4978-4987`).
        82 | 83 | 84 => {
            data.height = 20;
            data.top = -2;
            data.flip = flip_x;
        }
        // Vines / jungle vines: raised 2 px, flip on even columns
        // (`TileDrawing.cs:5000-5013`).
        52 | 62 => {
            data.top = -2;
            data.flip = flip_x;
        }
        // Candles / water candle / platinum candle / peace candle / shadow
        // candle: 20 px tall, raised 4 px (`TileDrawing.cs:4819-4826`).
        33 | 49 | 174 | 372 | 646 => {
            data.height = 20;
            data.top = -4;
        }
        // Palm tree: 20x20. C# also replaces the frame row with
        // `tileFrameY = 22 * GetPalmTreeBiome(x, y)` (line 4670), which is a
        // frame replacement rather than an additive offset.
        // TODO(app): source-rect Y = 22 * palm tree biome.
        323 => {
            data.width = 20;
            data.height = 20;
        }
        // Cactus: raised 2 px (`TileDrawing.cs:5014-5018`). `DrawSingleTile`
        // then adds 54/108/162 to the source Y for corrupt/hallowed/crimson
        // cacti (`TileDrawing.cs:1132-1147`, `WorldGen.GetCactusType`).
        // TODO(app): add the biome variant offset 54/108/162.
        80 => {
            data.top = 2;
        }
        // Dye plants: 32x38, flip on even columns. C# sets `tileTop` to -6
        // when `tileFrameX == 238` and -20 otherwise (lines 5052-5062), and
        // rewrites frame X for biome variants; frame X is not in this API.
        // TODO(app): top = -6 for tileFrameX == 238, else -20.
        227 => {
            data.width = 32;
            data.height = 38;
            data.flip = flip_x;
        }
        // Plantera bulb: raised 2 px; `addFrY = Main.tileFrame[238] * 36`
        // (lines 5441-5443) is the animation offset applied by app code.
        // TODO(app): add_fr_y = 36 * animation frame.
        238 => {
            data.top = 2;
        }
        // Beds: 18 px tall. The style variants (`tileFrameY / 2016`) also
        // shift addFrX/addFrY (lines 4909-4916) and need the frame row.
        // TODO(app): apply the bed style frame offset.
        79 => {
            data.height = 18;
        }
        // Tables / containers / detonators: C# raises tileHeight to 18 only
        // on the `tileFrameY == 18` row (lines 4943-4952); frame Y is not
        // part of this API, so every row keeps the 16 px default here.
        // TODO(app): tileFrameY == 18 -> height 18.
        14 | 21 | 411 | 467 | 469 => {}
        // Chairs / toilets: C# raises tileHeight to 18 only when
        // `tileFrameY % 40 == 18` (lines 4953-4959).
        // TODO(app): tileFrameY % 40 == 18 -> height 18.
        15 | 497 => {}
        // 19 (platforms), 25, 40, 50, 51, 75 and 76 have no branch in
        // GetTileDrawData and use the generic 16x16 block metrics.
        _ => {}
    }

    data
}

#[cfg(test)]
mod tests {
    use super::*;

    const DEFAULT: TileDrawData = TileDrawData {
        width: 16,
        height: 16,
        top: 0,
        add_fr_x: 0,
        add_fr_y: 0,
        flip: false,
    };

    #[test]
    fn default_block_is_16x16() {
        assert_eq!(TileDrawData::default(), DEFAULT);
        // Generic blocks and the listed types with no switch case.
        for kind in [0, 1, 19, 25, 40, 50, 51, 75, 76, 255, u16::MAX] {
            assert_eq!(draw_data(kind, 0, true), DEFAULT, "kind {kind}");
            assert_eq!(draw_data(kind, 7, false), DEFAULT, "kind {kind}");
        }
    }

    #[test]
    fn torch_uses_solid_above() {
        let hanging = draw_data(4, 10, true);
        assert_eq!(
            hanging,
            TileDrawData {
                width: 20,
                height: 20,
                top: 4,
                ..DEFAULT
            }
        );
        let standing = draw_data(4, 10, false);
        assert_eq!(
            standing,
            TileDrawData {
                width: 20,
                height: 20,
                top: 0,
                ..DEFAULT
            }
        );
        // Torches do not care about column parity.
        assert_eq!(draw_data(4, 11, true), hanging);
        assert_eq!(draw_data(4, 11, false), standing);
    }

    #[test]
    fn plants_flip_on_even_columns() {
        for kind in [3, 24, 61, 71, 110, 201, 637, 703] {
            let even = draw_data(kind, 2, true);
            assert_eq!(
                (even.height, even.top, even.flip),
                (20, 0, true),
                "kind {kind}"
            );
            let odd = draw_data(kind, 3, true);
            assert_eq!(
                (odd.height, odd.top, odd.flip),
                (20, 0, false),
                "kind {kind}"
            );
        }
    }

    #[test]
    fn tall_plants_are_raised_and_flipped() {
        for kind in [73, 74] {
            let even = draw_data(kind, 42, false);
            assert_eq!(
                (even.width, even.height, even.top, even.flip),
                (16, 32, -12, true),
                "kind {kind}"
            );
            let odd = draw_data(kind, 43, false);
            assert_eq!(
                (odd.width, odd.height, odd.top, odd.flip),
                (16, 32, -12, false),
                "kind {kind}"
            );
        }
    }

    #[test]
    fn herbs_are_raised_and_flipped() {
        for kind in [82, 83, 84] {
            let even = draw_data(kind, 0, true);
            assert_eq!(
                (even.height, even.top, even.flip),
                (20, -2, true),
                "kind {kind}"
            );
            let odd = draw_data(kind, 1, true);
            assert_eq!(
                (odd.height, odd.top, odd.flip),
                (20, -2, false),
                "kind {kind}"
            );
        }
    }

    #[test]
    fn candles_are_20px_high_and_raised() {
        for kind in [33, 49, 174, 372, 646] {
            let data = draw_data(kind, 0, false);
            assert_eq!(
                (data.width, data.height, data.top, data.flip),
                (16, 20, -4, false),
                "kind {kind}"
            );
        }
    }

    #[test]
    fn tables_and_chairs_need_frame_y() {
        // C# only raises the height on specific frame rows (`tileFrameY == 18`
        // for tables, `tileFrameY % 40 == 18` for chairs/toilets). The frame
        // row is not part of the signature, so these stay 16 px (TODO(app)).
        for kind in [14, 21, 411, 467, 469, 15, 497] {
            assert_eq!(draw_data(kind, 0, true), DEFAULT, "kind {kind}");
        }
    }

    #[test]
    fn vines_are_raised_and_flipped() {
        for kind in [52, 62] {
            let even = draw_data(kind, 4, true);
            assert_eq!(
                (even.height, even.top, even.flip),
                (16, -2, true),
                "kind {kind}"
            );
            assert_eq!(draw_data(kind, 5, true).flip, false, "kind {kind}");
        }
    }

    #[test]
    fn thorns_beds_and_palms_use_their_extra_size() {
        for kind in [32, 77] {
            assert_eq!(
                (
                    draw_data(kind, 0, true).width,
                    draw_data(kind, 0, true).height
                ),
                (16, 18),
                "kind {kind}"
            );
        }
        let bed = draw_data(79, 0, true);
        assert_eq!(
            (bed.width, bed.height, bed.top, bed.flip),
            (16, 18, 0, false)
        );
        let palm = draw_data(323, 0, true);
        assert_eq!(
            (palm.width, palm.height, palm.top, palm.flip),
            (20, 20, 0, false)
        );
    }

    #[test]
    fn cactus_and_plantera_bulb_sit_low() {
        for kind in [80, 238] {
            let data = draw_data(kind, 0, true);
            assert_eq!(
                (data.width, data.height, data.top, data.flip),
                (16, 16, 2, false),
                "kind {kind}"
            );
        }
    }

    #[test]
    fn dye_plants_are_32x38_and_flip() {
        let even = draw_data(227, 6, false);
        assert_eq!(
            (even.width, even.height, even.top, even.flip),
            (32, 38, 0, true)
        );
        assert_eq!(draw_data(227, 7, false).flip, false);
    }
}
