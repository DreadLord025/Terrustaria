//! S1-G-1 wave B: the rope back-draw pass — `TileDrawing.DrawTile_BackRope`
//! (`TileDrawing.cs:3766-3784`). When a platform / planter (380) / minecart
//! track (314) cell sits on a rope column, the rope's vertical segment sprite
//! is drawn BEHIND the cell: source rect `(90, (tileY + tileX) % 3 * 18)` of
//! the rope tile's sheet (the three connector rows at X 90 on the 18 px grid),
//! tinted with the CELL's `drawData.tileLight` but textured/painted from the
//! rope tile found at the column's top end (`Main.tile[tileX, topRopeY]`,
//! `:3776`/`:3780`).
//!
//! The rope tiles themselves (213 Rope, 214 Chain, 353 VineRope, 365 SilkRope,
//! 366 WebRope, 449/450/451, 504 MysticSnakeRope — `Main.cs:9313-9321`) draw
//! their vertical segments through the ordinary `DrawSingleTile` pass; this
//! module is only the rope-specific behind-the-cell draw plus its
//! `WorldGen.IsRope`/`GetRopeEnds` probes.
//!
//! WIRING: the tiles pass does not call this yet (wave A owns `tiles.rs`);
//! the hook is [`wants_back_rope`] + [`draw_back_rope`], to be invoked from
//! `DrawBasicTile`'s dispatch site (`TileDrawing.cs:1511-1514`, before the
//! platform sprite) and `DrawTile_MinecartTrack` (`:3792`). Draw order and
//! framing are untouched by this module.
use crate::assets;
use crate::render::entities::flip_uv;
use tr_core::lighting::LightStyle;
use tr_core::tiledata as td;
use tr_core::{TILE_PX, World};
use tr_render::{Atlas, SpriteRenderer};

/// `WorldGen.InWorld(x, y, fluff)` (`WorldGen.cs:9688-9694`).
#[inline]
fn in_world(world: &World, x: i32, y: i32, fluff: i32) -> bool {
    !(x < fluff || x >= world.width - fluff || y < fluff || y >= world.height - fluff)
}

/// `WorldGen.GetRopeEnds` (`WorldGen.cs:70026-70072`), verbatim: scan up to
/// `range_to_check` cells above for the first active rope tile, then scan
/// down from that top end. The second loop's start `j = 1 + (y - topRopeY)`
/// resumes one cell below `y` when a top was found — and when none was
/// (`topRopeY == -1`) it degenerates to absolute rows `j - 1` capped at
/// `range_to_check + 1`, the C#'s own quirk (for `y > 5` it never runs).
/// `treat_empty_as_the_rope_end` records an inactive cell as the end instead
/// of leaving `-1`.
pub(crate) fn rope_ends(
    world: &World,
    x: i32,
    y: i32,
    treat_empty_as_the_rope_end: bool,
    range_to_check: i32,
) -> (i32, i32) {
    let mut top_rope_y = -1;
    let mut bottom_rope_y = -1;
    // WorldGen.cs:70030-70050
    let mut i = 1;
    while i <= range_to_check && in_world(world, x, y - i, 0) {
        let Some(tile) = world.tile(x, y - i) else {
            break;
        };
        if !tile.active() {
            if treat_empty_as_the_rope_end {
                top_rope_y = y - i;
            }
            break;
        }
        if td::TILE_ROPE.has(tile.kind) {
            top_rope_y = y - i;
            break;
        }
        i += 1;
    }
    // WorldGen.cs:70051-70071
    let mut j = 1 + (y - top_rope_y);
    while j <= range_to_check + 1 && in_world(world, x, top_rope_y + j, 0) {
        let Some(tile) = world.tile(x, top_rope_y + j) else {
            break;
        };
        if !tile.active() {
            if treat_empty_as_the_rope_end {
                bottom_rope_y = top_rope_y + j;
            }
            break;
        }
        if td::TILE_ROPE.has(tile.kind) {
            bottom_rope_y = top_rope_y + j;
            break;
        }
        j += 1;
    }
    (top_rope_y, bottom_rope_y)
}

/// `WorldGen.IsRope(x, y, out topRopeY, out bottomRopeY, rangeToCheck = 5)`
/// (`WorldGen.cs:70081-70104`). Returns `(is_rope, top_rope_y, bottom_rope_y)`.
/// A rope tile itself answers `true` with BOTH ends left at `-1` (the early
/// return at `:70090-70093` never runs `GetRopeEnds`) — the draw below then
/// finds no tile at row `-1` and draws nothing, exactly like the C#'s
/// `Main.tile[tileX, -1] == null` arm.
pub(crate) fn is_rope(world: &World, x: i32, y: i32) -> (bool, i32, i32) {
    let Some(tile) = world.tile(x, y) else {
        return (false, -1, -1);
    };
    if !tile.active() {
        return (false, -1, -1);
    }
    if td::TILE_ROPE.has(tile.kind) {
        return (true, -1, -1);
    }
    // WorldGen.cs:70094-70097: only the minecart track (314), the planter
    // (380) and platforms probe the column.
    if tile.kind != 314 && tile.kind != 380 && !td::SET_PLATFORMS.has(tile.kind) {
        return (false, -1, -1);
    }
    let (top, bottom) = rope_ends(world, x, y, false, 5);
    (top != -1 && bottom != -1, top, bottom)
}

/// The `DrawBasicTile` / `DrawTile_MinecartTrack` dispatch gate: the back-rope
/// draw runs for the planter and platforms (`TileDrawing.cs:1511-1514`) and
/// unconditionally for the minecart track (`:3792`, type 314).
#[inline]
pub(crate) fn wants_back_rope(kind: u16) -> bool {
    kind == 380 || kind == 314 || td::SET_PLATFORMS.has(kind)
}

/// The resolved back-rope sprite of one cell: which sheet/paint to draw and
/// which connector row. `kind`/`paint` come from the rope tile at the TOP end
/// (`TileDrawing.cs:3776-3780` — `GetTileDrawTexture(tile, …)` keys on that
/// tile's `type`/`color()`, `:1434-1441`), NOT from the platform cell.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct BackRopeDraw {
    pub kind: u16,
    pub paint: u8,
    /// Always 90 — the connector column of the rope sheets (`:3781`).
    pub src_x: i16,
    /// `(tileY + tileX) % 3 * 18` (`:3779`) — the 18 px-grid row.
    pub src_y: i16,
}

/// `DrawTile_BackRope`'s decision chain (`TileDrawing.cs:3766-3784`):
/// the `InWorld(tileX, tileY, 1)` gate (`:3768`), `IsRope` (`:3774`), the
/// top-end tile lookup (`:3776-3777`) and the source rect (`:3779-3781`).
/// The C#'s `topRopeY = tileX` init (`:3772`) is dead — `IsRope` assigns its
/// out params at entry (`WorldGen.cs:70083-70084`).
pub(crate) fn back_rope_cell(world: &World, tile_x: i32, tile_y: i32) -> Option<BackRopeDraw> {
    if !in_world(world, tile_x, tile_y, 1) {
        return None;
    }
    let (is_rope, top_rope_y, _bottom) = is_rope(world, tile_x, tile_y);
    if !is_rope {
        return None;
    }
    let tile = world.tile(tile_x, top_rope_y)?;
    let y = (tile_y + tile_x) % 3 * 18;
    Some(BackRopeDraw {
        kind: tile.kind,
        paint: tile.paint(),
        src_x: 90,
        src_y: y as i16,
    })
}

/// The draw hook the tiles pass calls at the dispatch sites: pushes the
/// 16x16 back-rope quad at the cell's world position, tinted with the CELL's
/// raw `drawData.tileLight` (`:3781` — the light is the platform's, before
/// `GetFinalLight`; the actuation ghost never rides this draw) and flipped by
/// the cell's `tileSpriteEffect`. Returns the quad count (0 or 1).
pub(crate) fn draw_back_rope(
    r: &mut SpriteRenderer,
    world: &World,
    lookups: &assets::Lookups,
    atlas: Option<&Atlas>,
    tile_light: [f32; 3],
    style: &LightStyle,
    anim_time: f32,
    tile_x: i32,
    tile_y: i32,
    flip: bool,
) -> u32 {
    let Some(draw) = back_rope_cell(world, tile_x, tile_y) else {
        return 0;
    };
    let Some(a) = atlas else {
        return 0;
    };
    let Some(tex) = lookups.rect_uv(a, draw.kind, draw.src_x, draw.src_y, 16, 16) else {
        return 0;
    };
    let c = style.shade(tile_light, tile_x, tile_y, anim_time);
    r.push_sprite_paint(
        [tile_x as f32 * TILE_PX, tile_y as f32 * TILE_PX],
        [16.0, 16.0],
        flip_uv(tex.uv, flip),
        [c[0], c[1], c[2], 1.0],
        0.0,
        draw.paint,
    );
    1
}

#[cfg(test)]
mod tests {
    use super::*;
    use tr_core::tile::FLAG_ACTIVE;

    /// 40x40 empty world; `place` activates a tile of `kind` at (x, y).
    fn world_with(tiles: &[(i32, i32, u16)]) -> World {
        let mut w = World::new("ropes".into(), 40, 40);
        for &(x, y, kind) in tiles {
            let t = w.tile_mut(x, y).unwrap();
            t.kind = kind;
            t.flags |= FLAG_ACTIVE;
        }
        w
    }

    #[test]
    fn the_rope_set_matches_the_main_init() {
        // Main.cs:9313-9321: exactly these nine ids are `Main.tileRope`.
        for id in [213u16, 214, 353, 365, 366, 449, 450, 451, 504] {
            assert!(td::TILE_ROPE.has(id), "rope id {id}");
        }
        for id in [1u16, 19, 314, 380, 215] {
            assert!(!td::TILE_ROPE.has(id), "non-rope id {id}");
        }
    }

    #[test]
    fn is_rope_answers_true_for_a_rope_tile_with_both_ends_minus_one() {
        // WorldGen.cs:70090-70093: the early return leaves the out params at
        // their entry values (-1, -1) — GetRopeEnds never runs.
        let w = world_with(&[(10, 10, 213)]);
        assert_eq!(is_rope(&w, 10, 10), (true, -1, -1));
        // …and the draw then finds no tile at row -1 (the C# null arm).
        assert_eq!(back_rope_cell(&w, 10, 10), None);
    }

    #[test]
    fn is_rope_is_false_for_plain_blocks_and_inactive_cells() {
        let w = world_with(&[(10, 10, 1)]);
        assert_eq!(is_rope(&w, 10, 10), (false, -1, -1));
        let mut w = world_with(&[(10, 10, 19)]);
        w.tile_mut(10, 10).unwrap().flags &= !FLAG_ACTIVE;
        assert_eq!(is_rope(&w, 10, 10), (false, -1, -1));
        // Out of the world: the null-tile arm (WorldGen.cs:70085-70089).
        assert_eq!(is_rope(&w, -1, 10), (false, -1, -1));
    }

    #[test]
    fn a_platform_between_two_rope_ends_is_rope() {
        // A contiguous column: rope at y-2/y-1, platform at y, rope at y+1.
        // The scans break on inactive cells (WorldGen.cs:70037/:70058), so
        // the cells between the ends must be active.
        let w = world_with(&[(10, 8, 213), (10, 9, 213), (10, 10, 19), (10, 11, 365)]);
        assert_eq!(is_rope(&w, 10, 10), (true, 9, 11));
        // The planter (380) and the minecart track (314) probe the same way.
        for kind in [380u16, 314] {
            let w = world_with(&[(10, 8, 214), (10, 9, 214), (10, 10, kind), (10, 11, 214)]);
            assert_eq!(is_rope(&w, 10, 10), (true, 9, 11), "kind {kind}");
        }
    }

    #[test]
    fn one_missing_end_means_not_rope() {
        // Only a top end: bottom stays -1 → false (WorldGen.cs:70099-70103).
        let w = world_with(&[(10, 8, 213), (10, 9, 213), (10, 10, 19)]);
        assert_eq!(is_rope(&w, 10, 10), (false, 9, -1));
        // Only a bottom end: the up scan breaks on the empty cell above,
        // and the degenerate down loop never runs at y=10 (see the quirk
        // test) — both ends stay -1.
        let w = world_with(&[(10, 10, 19), (10, 11, 213)]);
        assert_eq!(is_rope(&w, 10, 10), (false, -1, -1));
    }

    #[test]
    fn the_up_scan_stops_at_five_cells() {
        // rangeToCheck = 5: active non-rope cells keep the scan going
        // (WorldGen.cs:70030-70050 has no break for them), but a rope six
        // above is out of reach.
        let w = world_with(&[
            (10, 4, 213),
            (10, 5, 1),
            (10, 6, 1),
            (10, 7, 1),
            (10, 8, 1),
            (10, 9, 1),
            (10, 10, 19),
            (10, 11, 213),
        ]);
        assert_eq!(is_rope(&w, 10, 10), (false, -1, -1));
        // Five above is the last reachable row.
        let w = world_with(&[
            (10, 5, 213),
            (10, 6, 1),
            (10, 7, 1),
            (10, 8, 1),
            (10, 9, 1),
            (10, 10, 19),
            (10, 11, 213),
        ]);
        assert_eq!(is_rope(&w, 10, 10), (true, 5, 11));
    }

    #[test]
    fn the_down_scan_resumes_below_the_cell_and_caps_at_range_plus_one() {
        // Top found at y-1 → j starts at 2, absolute row top+j = y+1; the
        // cap j <= 6 reaches top+6 = y+5. A rope at y+5 is the last hit.
        let w = world_with(&[
            (10, 8, 213),
            (10, 9, 213),
            (10, 10, 19),
            (10, 11, 1),
            (10, 12, 1),
            (10, 13, 1),
            (10, 14, 1),
            (10, 15, 213),
        ]);
        assert_eq!(is_rope(&w, 10, 10), (true, 9, 15));
        let w = world_with(&[
            (10, 8, 213),
            (10, 9, 213),
            (10, 10, 19),
            (10, 11, 1),
            (10, 12, 1),
            (10, 13, 1),
            (10, 14, 1),
            (10, 15, 1),
            (10, 16, 213),
        ]);
        assert_eq!(is_rope(&w, 10, 10), (false, 9, -1));
    }

    #[test]
    fn the_no_top_end_down_scan_keeps_the_c_sharp_quirk() {
        // WorldGen.cs:70051: with top = -1 the loop walks absolute rows j-1
        // for j in (y+2)..=6 — it only runs near the world's top rows.
        let w = world_with(&[(10, 2, 19), (10, 3, 213)]);
        assert_eq!(rope_ends(&w, 10, 2, false, 5), (-1, 3));
        // IsRope still demands a top end, so the draw stays off.
        assert_eq!(is_rope(&w, 10, 2), (false, -1, 3));
        // At y > 5 the degenerate loop never runs at all.
        let w = world_with(&[(10, 10, 19), (10, 11, 213)]);
        assert_eq!(rope_ends(&w, 10, 10, false, 5), (-1, -1));
    }

    #[test]
    fn treat_empty_as_the_rope_end_records_inactive_cells() {
        // WorldGen.cs:70037-70044 / :70058-70065 (the flag IsRope passes
        // false, but GetRopeEnds' other callers pass true).
        let mut w = world_with(&[(10, 8, 213), (10, 10, 19), (10, 12, 213)]);
        w.tile_mut(10, 9).unwrap().flags &= !FLAG_ACTIVE; // empty above
        // The empty cell at y-1 becomes the top end; the down scan resumes
        // at top+2 = y+1, whose empty cell becomes the bottom end.
        assert_eq!(rope_ends(&w, 10, 10, true, 5), (9, 11));
        assert_eq!(rope_ends(&w, 10, 10, false, 5), (-1, -1));
    }

    #[test]
    fn the_back_rope_rect_rides_the_connector_column_and_the_mod_three_row() {
        // TileDrawing.cs:3779-3781: Rectangle(90, (tileY + tileX) % 3 * 18, 16, 16).
        for (x, y, row) in [
            (10i32, 10i32, 36i16),
            (10, 11, 0),
            (10, 12, 18),
            (11, 12, 36),
        ] {
            let w = world_with(&[
                (x, y - 2, 213),
                (x, y - 1, 213),
                (x, y, 19),
                (x, y + 1, 213),
            ]);
            let d = back_rope_cell(&w, x, y).unwrap();
            assert_eq!((d.src_x, d.src_y), (90, row), "({x},{y})");
        }
    }

    #[test]
    fn the_back_rope_texture_and_paint_come_from_the_top_rope_tile() {
        // TileDrawing.cs:3776-3780: the sheet key is Main.tile[tileX, topRopeY]
        // — a painted silk rope above a painted platform draws the SILK
        // sheet with the ROPE's paint, not the platform's.
        // The nearest rope above the platform (topRopeY = y-1) supplies the
        // sheet key — paint that one, not the farther rope cell.
        let mut w = world_with(&[(10, 8, 365), (10, 9, 365), (10, 10, 19), (10, 11, 213)]);
        w.tile_mut(10, 9).unwrap().set_paint(13);
        w.tile_mut(10, 10).unwrap().set_paint(7);
        let d = back_rope_cell(&w, 10, 10).unwrap();
        assert_eq!((d.kind, d.paint), (365, 13));
    }

    #[test]
    fn the_back_rope_draw_gates_on_in_world_fluff_one() {
        // TileDrawing.cs:3768: InWorld(tileX, tileY, 1) — the outermost
        // row/column never draws even with a valid rope column.
        let w = world_with(&[(0, 3, 213), (0, 4, 213), (0, 5, 19), (0, 6, 213)]);
        assert_eq!(back_rope_cell(&w, 0, 5), None);
        let w = world_with(&[(1, 3, 213), (1, 4, 213), (1, 5, 19), (1, 6, 213)]);
        assert!(back_rope_cell(&w, 1, 5).is_some());
        // The bottom edge: y = height - 2 is the last in-world row
        // (InWorld's `y >= maxTilesY - fluff` bound).
        let w = world_with(&[(5, 36, 213), (5, 37, 213), (5, 38, 19), (5, 39, 213)]);
        assert!(back_rope_cell(&w, 5, 38).is_some());
        let w = world_with(&[(5, 37, 213), (5, 38, 213), (5, 39, 19)]);
        assert_eq!(back_rope_cell(&w, 5, 39), None);
    }

    #[test]
    fn the_dispatch_gate_matches_the_two_call_sites() {
        // TileDrawing.cs:1511 (380 || Platforms) and :3792 (314, unconditional).
        for kind in [19u16, 380, 314, 427, 435, 436, 437, 438, 439] {
            assert!(wants_back_rope(kind), "kind {kind}");
        }
        // Rope tiles draw their own segments through DrawSingleTile — the
        // back-rope dispatch never fires for them; nor for plain blocks.
        for kind in [1u16, 213, 214, 353, 365, 366, 504] {
            assert!(!wants_back_rope(kind), "kind {kind}");
        }
    }
}
