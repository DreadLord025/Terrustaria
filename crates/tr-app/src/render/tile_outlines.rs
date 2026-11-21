//! The smart-cursor tile-outline overlay — `TileDrawing.GetTileOutlineInfo`
//! (`TileDrawing.cs:4478-4489`) + its draw tail (`:1404-1408`), gated by
//! `TileID.Sets.HasOutlines` (`TileID.cs:551`) and fed by
//! `Main.InSmartCursorHighlightArea` (`Main.cs:3945-3957`).
//!
//! W4-g86 (the demon/crimson altar draw lane), CODE-ONLY MODE — no compiler
//! invoked; the witnesses below are WRITTEN NOT RUN.
//!
//! ## Census (what the altar draw lane verified vs ported)
//!
//! The altar family (26 DemonAltar / 695 DemonAltarReplica; the crimson
//! flavour is tile 26's frame columns 54+, NOT a second id) already draws:
//!
//! * frames — the height-18 group `TileDrawing.cs:4880-4908` is ported at
//!   `tr_core::drawdata::draw_data` (`drawdata.rs:327-347`, the d61/d62
//!   census verdict PORTED); 26 has no `Main.tileFrame` animation (no case
//!   in `AnimateTiles`, no first/second-switch stride) — the static rect is
//!   the whole frame story;
//! * glow — the altar's light is the `TileLightScanner` seed
//!   (`TileLightScanner.cs:2965-2983`), ported at
//!   `tile_light/scanner.rs:1051-1061` (the fx≥54 crimson-lamp arm + the
//!   purple demon arm, one `Next(-5,6)` draw each). There is NO
//!   `tileGlowMask[26]`/`[695]` row (`Main.cs:10024-10068` census) and no
//!   glow-switch case (`TileDrawing.cs:6049-6507`) — the scanner seed IS
//!   the glow;
//! * ambient particles — `DrawTiles_EmitParticles` `:7348-7361` rides the
//!   in-flight W4-d60 `tile_draw_particles` module (keep-one at the merge;
//!   this lane does NOT duplicate it);
//! * hammer damage — the ≥80-power/hardmode gate + the `HitTile` book +
//!   the refusal self-hurt are landed (S1-P-i31, `editing.rs::hammer_tile_hit`,
//!   `Player.cs:45665-45704`); the fail-tick dust type ladder
//!   (`WorldGen.cs:68970-68973`) rides the in-flight W4-g14
//!   `worldgen_dust_tile` module. The altar is NOT `tileSolid`
//!   (`Main.cs` init census — no `tileSolid[26]`), so `DrawTileCracks`
//!   (`Main.cs:61577`) never cracks it: the damage visual is the dust, not
//!   a crack overlay.
//!
//! **The MISSING verbatim piece this lane ports**: the smart-cursor
//! selection outline. 26 IS in `TileID.Sets.HasOutlines` (`TileID.cs:551`,
//! the `…, 16, 17, 18, 26, 77, …` tail); 695 is NOT (replicas never
//! outline). The port had no `GetTileOutlineInfo` counterpart at all
//! (research/09 row "Dangerous-tile outline … MISSING"; the pylon pass
//! feeds `selection_mode = 0` — `pylons.rs:35`). The outline texture
//! `Images/Misc/TileOutlines/Tiles_26` exists in the content dir
//! (`AssetInitializer.cs:356-360`).
//!
//! ## UNWIRED — the two feed seams
//!
//! 1. `Main.SmartInteractTileCoords`/`…Selected` (the highlight-area point
//!    lists, `Main.cs:2035-2037`, filled by the smart-cursor lookup tail,
//!    cleared by `ClearSmartInteract` `:48054-48062`) are unported — the
//!    `world_sim/smart_cursor` lane owns the lookup; [`HighlightArea`]
//!    carries the lists + the interaction rect until then (fed empty =
//!    no outline, exactly the pre-wiring pixels).
//! 2. The `HighlightMask` sheets are not packed into the atlas; the draw
//!    tail returns an [`OutlineQuad`] record (sheet name + rect + tint)
//!    for the atlas-binding wave, the `CrackQuad`/`SpecialSprite` seam
//!    convention (`tile_cracks.rs`, `pylons.rs`).
//!
//! ## RNG census
//!
//! ZERO draws — `GetTileOutlineInfo`, `InSmartCursorHighlightArea` and
//! `GetSelectionGlowColor` are pure reads/integer maths. The outline never
//! touches a stream.

use tr_core::smart_cursor::in_tile_bounds;
use tr_core::tiledata as td;

/// `Main.InSmartCursorHighlightArea`'s state (`Main.cs:3945-3957`): the two
/// smart-interact point lists + the tile-interaction rect + `Main.hideUI`.
/// The lists are `Vec` membership tests in the C# (`List<Point>.Contains`);
/// the port keeps slices (order irrelevant, duplicates harmless).
pub(crate) struct HighlightArea<'a> {
    /// `Main.SmartInteractTileCoords` (`Main.cs:2035`).
    pub coords: &'a [(i32, i32)],
    /// `Main.SmartInteractTileCoordsSelected` (`Main.cs:2037`).
    pub selected: &'a [(i32, i32)],
    /// `Main.hideUI` (`:3948`).
    pub hide_ui: bool,
    /// `Main.TileInteractionLX/LY/HX/HY` (`:3952`).
    pub lx: i32,
    pub ly: i32,
    pub hx: i32,
    pub hy: i32,
}

/// `Main.InSmartCursorHighlightArea` (`Main.cs:3945-3957`) verbatim: the
/// selected-membership out-param is computed FIRST (even when the area
/// test fails), `hideUI` refuses, then the interaction-rect gate + the
/// coords membership. Returns `(in_area, actually_selected)`.
pub(crate) fn in_smart_cursor_highlight_area(
    area: &HighlightArea<'_>,
    x: i32,
    y: i32,
) -> (bool, bool) {
    // `:3947` — the out-param precedes every gate.
    let actually_selected = area.selected.contains(&(x, y));
    if area.hide_ui {
        return (false, actually_selected);
    }
    if in_tile_bounds(x, y, area.lx, area.ly, area.hx, area.hy) {
        return (area.coords.contains(&(x, y)), actually_selected);
    }
    (false, actually_selected)
}

/// `Colors.GetSelectionGlowColor` (`Colors.cs:89-92`): the integer
/// halves/thirds of the average tile lighting, as XNA bytes. The pylon
/// pass carries the f32 form (`pylons.rs:134`); this is the byte form the
/// outline tint needs (the C# `Color` ctor takes ints).
pub(crate) fn selection_glow_color_bytes(selected: bool, avg: i32) -> [u8; 4] {
    let a = avg.clamp(0, 255) as u8;
    if selected {
        [a, a, (avg / 3).clamp(0, 255) as u8, a]
    } else {
        let h = (avg / 2).clamp(0, 255) as u8;
        [h, h, h, a]
    }
}

/// The `GetTileOutlineInfo` result + the `:1404-1408` draw arguments: the
/// `HighlightMask[type]` sheet name, the framed source rect and the tint.
/// Position/centring reuse the tile's own `vector` maths (the `:1407`
/// anchor is the plain `tileTop` anchor, NOT the half-brick-shifted one).
pub(crate) struct OutlineQuad {
    /// `TextureAssets.HighlightMask[typeCache]` —
    /// `Images/Misc/TileOutlines/Tiles_{kind}` (`AssetInitializer.cs:360`).
    pub sheet: String,
    /// `new Rectangle(tileFrameX + addFrX, tileFrameY + addFrY,
    /// tileWidth, tileHeight)` (`:1406`) — the FULL height (no
    /// half-brick cut, unlike the base sprite's `rectangle`).
    pub src: [i32; 4],
    /// `Colors.GetSelectionGlowColor(actuallySelected, num)` (`:4486`).
    pub color: [u8; 4],
}

/// `TileDrawing.GetTileOutlineInfo` (`TileDrawing.cs:4478-4489`) + the
/// `HasOutlines` gate (`:893-896`) + the draw tail (`:1404-1408`), one
/// cell. `light` is `Lighting.GetColor(x, y)` as XNA bytes (the caller's
/// `cell_bytes` convention, `tiles.rs:725-730`); `fx/fy` are the RESOLVED
/// `tileFrameX/Y` and `add_fr_x/y`/`width`/`height` the `TileDrawData`
/// fields. `None` when the kind has no outline set, the cell is outside
/// the highlight area, or the average light is ≤ 10 (`:4483`).
pub(crate) fn tile_outline_quad(
    kind: u16,
    fx: i32,
    fy: i32,
    add_fr_x: i32,
    add_fr_y: i32,
    width: i32,
    height: i32,
    light: [u8; 3],
    area: &HighlightArea<'_>,
    x: i32,
    y: i32,
) -> Option<OutlineQuad> {
    // `:893` — the HasOutlines gate runs BEFORE GetTileOutlineInfo.
    if !td::SET_HAS_OUTLINES.has(kind) {
        return None;
    }
    let (in_area, selected) = in_smart_cursor_highlight_area(area, x, y);
    if !in_area {
        return None;
    }
    // `:4482` — the integer average of the three light bytes.
    let num = (light[0] as i32 + light[1] as i32 + light[2] as i32) / 3;
    if num <= 10 {
        return None;
    }
    Some(OutlineQuad {
        sheet: format!("Misc/TileOutlines/Tiles_{kind}"),
        // `:1406` — the framed rect, full height.
        src: [fx + add_fr_x, fy + add_fr_y, width, height],
        color: selection_glow_color_bytes(selected, num),
    })
}

#[cfg(test)]
mod w4_g86_tests {
    use super::*;

    fn area<'a>(coords: &'a [(i32, i32)], selected: &'a [(i32, i32)]) -> HighlightArea<'a> {
        HighlightArea { coords, selected, hide_ui: false, lx: 0, ly: 0, hx: 100, hy: 100 }
    }

    /// The altar 26 outlines; the replica 695 never does (the HasOutlines
    /// set census, `TileID.cs:551`).
    #[test]
    fn w4_g86_the_altar_outlines_but_the_replica_does_not() {
        let a = area(&[(5, 5)], &[]);
        let q = tile_outline_quad(26, 0, 0, 0, 0, 16, 18, [90, 90, 90], &a, 5, 5);
        assert!(q.is_some(), "26 is in HasOutlines");
        assert_eq!(q.unwrap().sheet, "Misc/TileOutlines/Tiles_26");
        assert!(
            tile_outline_quad(695, 0, 0, 0, 0, 16, 18, [90, 90, 90], &a, 5, 5).is_none(),
            "695 is NOT in HasOutlines"
        );
    }

    /// The crimson columns (fx 54/72/90) keep the outline; the rect rides
    /// the resolved frame + addFr (`:1406`), full 18-px height.
    #[test]
    fn w4_g86_the_crimson_columns_outline_with_the_framed_rect() {
        let a = area(&[(5, 5)], &[(5, 5)]);
        let q = tile_outline_quad(26, 54, 18, 0, 0, 16, 18, [120, 120, 120], &a, 5, 5)
            .expect("the crimson column outlines");
        assert_eq!(q.src, [54, 18, 16, 18]);
        // selected: (avg, avg, avg/3, avg), avg = 120.
        assert_eq!(q.color, [120, 120, 40, 120]);
    }

    /// The ≤10 average-light refusal (`:4483`): 10 refuses, 11 passes;
    /// the average is the INTEGER third (30+30+31)/3 = 30, not 30.33.
    #[test]
    fn w4_g86_the_dark_cell_refuses_at_ten_and_the_average_truncates() {
        let a = area(&[(5, 5)], &[]);
        assert!(tile_outline_quad(26, 0, 0, 0, 0, 16, 18, [10, 10, 10], &a, 5, 5).is_none());
        let q = tile_outline_quad(26, 0, 0, 0, 0, 16, 18, [11, 11, 11], &a, 5, 5)
            .expect("11 passes");
        // unselected: (avg/2, avg/2, avg/2, avg) = (5,5,5,11).
        assert_eq!(q.color, [5, 5, 5, 11]);
        let q2 = tile_outline_quad(26, 0, 0, 0, 0, 16, 18, [30, 30, 31], &a, 5, 5).unwrap();
        assert_eq!(q2.color, [15, 15, 15, 30], "(91)/3 truncates to 30");
    }

    /// `InSmartCursorHighlightArea`'s gate order: the selected out-param
    /// is computed even outside the area; hideUI refuses; the rect gate
    /// precedes the coords membership.
    #[test]
    fn w4_g86_the_highlight_area_gates_match_the_csharp_order() {
        let mut a = area(&[(5, 5)], &[(5, 5)]);
        assert_eq!(in_smart_cursor_highlight_area(&a, 5, 5), (true, true));
        // Outside the interaction rect: not in area, but STILL selected.
        assert_eq!(in_smart_cursor_highlight_area(&a, 200, 200), (false, true));
        // In the rect but not in coords: not in area.
        assert_eq!(in_smart_cursor_highlight_area(&a, 6, 6), (false, false));
        // hideUI refuses even a listed cell (the selected flag survives).
        a.hide_ui = true;
        assert_eq!(in_smart_cursor_highlight_area(&a, 5, 5), (false, true));
    }

    /// The empty-list feed (the pre-wiring state) draws nothing — the
    /// outline lane is pixel-inert until the smart-cursor lists land.
    #[test]
    fn w4_g86_the_empty_feed_is_pixel_inert() {
        let a = area(&[], &[]);
        assert!(tile_outline_quad(26, 0, 0, 0, 0, 16, 18, [255, 255, 255], &a, 5, 5).is_none());
    }

    /// The byte tint's integer maths (`Colors.cs:89-92`): the selected
    /// third and the unselected half both truncate; alpha is the raw avg.
    #[test]
    fn w4_g86_the_selection_tint_uses_integer_division() {
        assert_eq!(selection_glow_color_bytes(false, 101), [50, 50, 50, 101]);
        assert_eq!(selection_glow_color_bytes(true, 101), [101, 101, 33, 101]);
        assert_eq!(selection_glow_color_bytes(true, 255), [255, 255, 85, 255]);
    }
}
