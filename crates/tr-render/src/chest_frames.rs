//! W4-d58 (R2 chest slice): the chest draw-frame selection —
//! `TileDrawing.CacheSpecialDraws_Part2`'s `TileID.Sets.BasicChest` block
//! (`TileDrawing.cs:4287-4330`), which `DrawSingleTile` runs AFTER
//! `GetTileDrawData` (`:993`). Pure source-rect decision maths, no engine
//! state: the caller resolves the chest's lid frame (`Main.chest[i].frame`,
//! `Chest.cs:86` — 0 closed / 1 half-open / 2 open, advanced by
//! `Chest.UpdateChestFrames`, `Chest.cs:3603-3662`, the S1-P-i43 sim-side
//! animator) and feeds it in. The draw side never rolls RNG.
//!
//! Moved here from the UNWIRED `tr-app/src/render/special_tiles.rs` slice
//! (S1-G-1e) so the frame selection lives in the render crate; the app's
//! `render/tiles.rs` wires it after the drawdata dispatch (W4-d58). The
//! `typeCache == 378` training-dummy half (`:4331-4355`) needs
//! `TETrainingDummy` + `Main.npc[npc].frame` and stays unported.

/// The chest block's writes to the draw data: `addFrX`/`addFrY`, the
/// bottom-row height clamp and the optional glow rect (golden-chest styles
/// 48/49).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ChestFrameAdjust {
    /// `drawData.addFrX = num * 18 - tileFrameX` (`TileDrawing.cs:4307`).
    pub add_fr_x: i16,
    /// `drawData.addFrY = num4 - tileFrameY` (`:4321`), `num4` carrying the
    /// open-frame row shift +38 / +76.
    pub add_fr_y: i16,
    /// `drawData.tileHeight = 18` when `frameY / 18 != 0` (`:4322-4325`);
    /// `None` keeps the incoming height.
    pub height_override: Option<i16>,
    /// `drawData.glowSourceRect` for tile 21 styles 48/49 (`:4326-4329`):
    /// `(16 * (num % 2), tileFrameY + addFrY, tileWidth, tileHeight)` —
    /// the height AFTER the clamp above. The port's glow pass is unbound
    /// (TILE-DRAW-2), so the wiring records this without drawing it.
    pub glow_src: Option<[i32; 4]>,
}

/// The `_chestPositions` key: the chest's top-left cell
/// (`TileDrawing.cs:4291-4299`) — step left/up when the frame is not on the
/// 36 px anchor grid. The caller resolves it through `Chest.FindChest`
/// (`Chest.cs:513-520`, the port's `chest::find_chest`), caching the result
/// per draw like the C# dict (`:4300-4303`, cleared by
/// `ClearLegacyCachedDraws`, `:6613-6615`).
pub fn chest_anchor(x: i32, y: i32, frame_x: i16, frame_y: i16) -> (i32, i32) {
    let mut key = (x, y);
    if frame_x % 36 != 0 {
        key.0 -= 1;
    }
    if frame_y % 36 != 0 {
        key.1 -= 1;
    }
    key
}

/// `CacheSpecialDraws_Part2`'s `TileID.Sets.BasicChest` block
/// (`TileDrawing.cs:4289-4330`). `chest_frame` is `Main.chest[i].frame`
/// (`Chest.cs:86`): 0 closed, 1 half-open (+38), 2 open (+76); `None` is the
/// `FindChest == -1` miss (no row shift, `:4309`).
pub fn chest_frame_adjust(
    kind: u16,
    frame_x: i16,
    frame_y: i16,
    tile_width: i16,
    tile_height: i16,
    chest_frame: Option<i32>,
) -> ChestFrameAdjust {
    // TileDrawing.cs:4304-4308
    let num = frame_x as i32 / 18;
    let num2 = frame_y as i32 / 18;
    let num3 = frame_x as i32 / 36;
    let add_fr_x = num * 18 - frame_x as i32;
    let mut num4 = num2 * 18;
    // TileDrawing.cs:4309-4320 — the open-frame row shift.
    if let Some(frame) = chest_frame {
        if frame == 1 {
            num4 += 38;
        }
        if frame == 2 {
            num4 += 76;
        }
    }
    let add_fr_y = num4 - frame_y as i32;
    // TileDrawing.cs:4322-4325 — the bottom row draws 18 px tall.
    let height_override = if num2 != 0 { Some(18) } else { None };
    let height = height_override.unwrap_or(tile_height);
    // TileDrawing.cs:4326-4329 — the two golden-chest styles glow.
    let glow_src = if kind == 21 && (num3 == 48 || num3 == 49) {
        Some([
            16 * (num % 2),
            frame_y as i32 + add_fr_y,
            tile_width as i32,
            height as i32,
        ])
    } else {
        None
    };
    ChestFrameAdjust {
        add_fr_x: add_fr_x as i16,
        add_fr_y: add_fr_y as i16,
        height_override,
        glow_src,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chest_anchor_steps_to_the_top_left() {
        assert_eq!(chest_anchor(10, 20, 0, 0), (10, 20));
        assert_eq!(chest_anchor(11, 20, 18, 0), (10, 20));
        assert_eq!(chest_anchor(10, 21, 0, 18), (10, 20));
        assert_eq!(chest_anchor(11, 21, 18, 18), (10, 20));
    }

    #[test]
    fn closed_chest_keeps_the_frame_row() {
        // Top-left cell: addFr 0/0, height untouched, no glow.
        let a = chest_frame_adjust(21, 0, 0, 16, 16, Some(0));
        assert_eq!(a.add_fr_x, 0);
        assert_eq!(a.add_fr_y, 0);
        assert_eq!(a.height_override, None);
        assert_eq!(a.glow_src, None);
        // Bottom-right cell of style 0: frame (18, 18) → addFrX 0, row 18.
        let b = chest_frame_adjust(21, 18, 18, 16, 16, Some(0));
        assert_eq!(b.add_fr_x, 0);
        assert_eq!(b.add_fr_y, 0);
        assert_eq!(b.height_override, Some(18));
    }

    #[test]
    fn open_frames_shift_38_and_76() {
        // frame == 1 → num4 = row*18 + 38 (TileDrawing.cs:4312-4315).
        let half = chest_frame_adjust(21, 0, 0, 16, 16, Some(1));
        assert_eq!(half.add_fr_y, 38);
        let half_bottom = chest_frame_adjust(21, 18, 18, 16, 16, Some(1));
        assert_eq!(half_bottom.add_fr_y, 38); // 18 + 38 - 18
        // frame == 2 → +76 (:4316-4319).
        let open = chest_frame_adjust(21, 0, 0, 16, 16, Some(2));
        assert_eq!(open.add_fr_y, 76);
        let open_bottom = chest_frame_adjust(21, 18, 18, 16, 16, Some(2));
        assert_eq!(open_bottom.add_fr_y, 76);
        assert_eq!(open_bottom.height_override, Some(18));
        // FindChest miss → no shift (:4309).
        let miss = chest_frame_adjust(21, 0, 0, 16, 16, None);
        assert_eq!(miss.add_fr_y, 0);
    }

    #[test]
    fn style_columns_shift_x_by_the_18px_grid() {
        // Style 1 top-left: frameX 36 → num = 2, addFrX = 36 - 36 = 0;
        // its right column frameX 54 → num = 3, addFrX = 54 - 54 = 0.
        // A style whose sheet column differs (the C# num*18 rewrite) is
        // exercised by the golden styles below.
        let s1 = chest_frame_adjust(21, 36, 0, 16, 16, Some(0));
        assert_eq!(s1.add_fr_x, 0);
        let s1r = chest_frame_adjust(21, 54, 0, 16, 16, Some(0));
        assert_eq!(s1r.add_fr_x, 0);
    }

    #[test]
    fn golden_chest_styles_48_49_glow() {
        // Style 48: frameX = 48*36 = 1728, num = 96 (even → glow x 0).
        let g = chest_frame_adjust(21, 1728, 0, 16, 16, Some(1));
        assert_eq!(g.glow_src, Some([0, 38, 16, 16]));
        // Right column frameX 1746: num = 97 (odd → glow x 16), bottom row
        // clamps the glow height to 18.
        let g2 = chest_frame_adjust(21, 1746, 18, 16, 16, Some(1));
        assert_eq!(g2.glow_src, Some([16, 56, 16, 18]));
        // Style 49 also glows; style 47 does not.
        assert!(chest_frame_adjust(21, 49 * 36, 0, 16, 16, Some(0)).glow_src.is_some());
        assert!(chest_frame_adjust(21, 47 * 36, 0, 16, 16, Some(0)).glow_src.is_none());
        // Containers2 (467) never glows even at style 48.
        assert!(chest_frame_adjust(467, 48 * 36, 0, 16, 16, Some(0)).glow_src.is_none());
    }

    /// W4-d58 frame-state witness: the three lid states the i43 animator
    /// produces (`Chest.cs:3648-3659`'s 0/1/2 map) select exactly the
    /// +0/+38/+76 sheet rows on BOTH sub-cell rows, and the selection is a
    /// pure function of the frame — no counter, timer or RNG input.
    #[test]
    fn w4_d58_the_lid_states_select_the_three_sheet_rows() {
        for (frame, shift) in [(0, 0), (1, 38), (2, 76)] {
            for fy in [0_i16, 18] {
                let a = chest_frame_adjust(21, 0, fy, 16, 16, Some(frame));
                // The final source row = tileFrameY + addFrY.
                assert_eq!(fy as i32 + a.add_fr_y as i32, fy as i32 + shift);
            }
        }
        // Deterministic: the same inputs always give the same rect.
        assert_eq!(
            chest_frame_adjust(21, 18, 18, 16, 16, Some(1)),
            chest_frame_adjust(21, 18, 18, 16, 16, Some(1))
        );
    }
}
