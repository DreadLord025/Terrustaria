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
//! W4-g82 WIRED (2026-11-21): the framing arm below is now LIVE through its
//! tr-core twin `tr_core::framing_rope::rope_frame_arm`, called from
//! `framing::frame_of_variant`'s merge-chain tail (the append-safe zone —
//! d109/d110 own the head/wall halves). This render-side copy stays as the
//! witness-pinned reference incl. the pure reframe-request output (the d56
//! inlined-table convention, the coil module's IsRope twin precedent); the
//! production framing runs the core copy, whose square-reframe callers
//! subsume the :81923-81930 neighbour requests.
//!
//! WIRED (S1-G-1f, 2026-11-19): `render/tiles.rs::draw_tiles` calls
//! [`wants_back_rope`] + [`draw_back_rope`] after the visibility gates and
//! before the cell's own sprite — the `DrawBasicTile` dispatch site
//! (`TileDrawing.cs:1511-1514`, before the platform sprite) and
//! `DrawTile_MinecartTrack` (`:3792`) share the one gated hook. Draw order
//! and framing are untouched by this module; the draw reads only world
//! state (already covered by the G6 key's `world_revision`).
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

// ---------------------------------------------------------------- W4-g82
// The rope variant draw residual: `WorldGen.TileFrameCosmetic`'s rope arm
// (`WorldGen.cs:81894-81931`) — the neighbour merges that pick which rope
// sprite a cell draws. The port's generic framing (`tr-core/framing.rs`)
// carries only the `!tileRope` blend-all gate (`:81731`, framing.rs:928);
// the rope arm itself was never ported, so every rope cell framed through
// the generic RULES as if it were a plain block. This is the DRAW-side
// decision maths, verbatim; WIRED W4-g82 (2026-11-21) via the tr-core twin
// `tr_core::framing_rope::rope_frame_arm` — this copy stays the
// witness-pinned reference (the module doc's wiring note).
//
// Call site (LIVE in the core twin): `frame_of_variant` (tr-core/framing.rs), as an
// `else if td::TILE_ROPE.has(me)` arm AFTER the dirt branch (`:81743`) and
// BEFORE the generic per-type switch (`:81934`), replacing the four side
// slots of `n` before the mask build (framing.rs:1009). The two recursive
// `WorldGen.TileFrame(i, j±1)` calls (`:81923-81930`) are returned as
// [`RopeFrameResult::reframes`] requests — the pure probe never mutates the
// world; the caller runs `reframe_area`/`frame_of` on them (the C# runs
// them BEFORE the mask is read, so the caller must re-probe after).

/// The rope arm's decision output (`WorldGen.cs:81894-81931`): the four
/// side slots after the merges, plus the two neighbour re-frame requests
/// (`:81923-81930`, only when the matching side ended `-1`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct RopeFrameResult {
    pub up: i32,
    pub down: i32,
    pub left: i32,
    pub right: i32,
    /// `(x, y-1)` when `up == -1` (`:81923-81926`).
    pub reframe_up: bool,
    /// `(x, y+1)` when `down == -1` (`:81927-81930`).
    pub reframe_down: bool,
}

/// `WorldGen.IsRope(i, j)` — the 2-arg overload (`WorldGen.cs:70074-70079`)
/// the rope arm calls: a rope cell answers true, a platform/track/planter
/// answers true only with rope on BOTH ends (`:70099-70103`).
#[inline]
fn is_rope_cell(world: &World, x: i32, y: i32) -> bool {
    is_rope(world, x, y).0
}

/// `TileFrameCosmetic`'s rope arm (`WorldGen.cs:81894-81931`), verbatim.
/// `num` is the (framing-normalised) rope kind; `up/down/left/right` are
/// the four side slots AFTER the slope adjustments (the C# reads them at
/// `:81488-81507`). Order of the seven writes:
/// 1. `num != 504 && up != num && IsRope(i, j-1)` → `up = num` (`:81896`)
///    — the mystic snake rope 504 never merges upward through a rope.
/// 2. `down != num && (IsRope(i, j+1) || down == 698)` → `down = num`
///    (`:81900`) — the Dead Cells display jar 698 anchors a rope below it.
/// 3. `num != 504 && up > -1 && tileSolid[up] && !tileSolidTop[up]` →
///    `up = num` (`:81904`) — a full block above anchors the top.
/// 4. `down > -1 && tileSolid[down]` → `down = num` (`:81908`) — ANY solid
///    below anchors the bottom (solid-top platforms included).
/// 5. `num != 504 && up != num` → the left/right slots merge into `num`
///    when their neighbour is solid (`:81912-81922`) — a rope hanging from
///    a ceiling draws the side-attached sprites.
/// 6/7. the `-1` re-frame requests (`:81923-81930`).
pub(crate) fn rope_frame_arm(
    world: &World,
    x: i32,
    y: i32,
    num: i32,
    up: i32,
    down: i32,
    left: i32,
    right: i32,
) -> RopeFrameResult {
    let (mut up, mut down, mut left, mut right) = (up, down, left, right);
    // WorldGen.cs:81896-81899
    if num != 504 && up != num && is_rope_cell(world, x, y - 1) {
        up = num;
    }
    // WorldGen.cs:81900-81903
    if down != num && (is_rope_cell(world, x, y + 1) || down == 698) {
        down = num;
    }
    // WorldGen.cs:81904-81907
    if num != 504 && up > -1 && td::TILE_SOLID.has(up as u16) && !td::TILE_SOLID_TOP.has(up as u16) {
        up = num;
    }
    // WorldGen.cs:81908-81911
    if down > -1 && td::TILE_SOLID.has(down as u16) {
        down = num;
    }
    // WorldGen.cs:81912-81922
    if num != 504 && up != num {
        if left > -1 && td::TILE_SOLID.has(left as u16) {
            left = num;
        }
        if right > -1 && td::TILE_SOLID.has(right as u16) {
            right = num;
        }
    }
    RopeFrameResult {
        up,
        down,
        left,
        right,
        // WorldGen.cs:81923-81930
        reframe_up: up == -1,
        reframe_down: down == -1,
    }
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

    // ------------------------------------------------------------ W4-g82
    // The rope framing arm witnesses (`WorldGen.cs:81894-81931`).
    // WRITTEN NOT RUN (code-only mode, no compiler invoked on this branch).

    /// A rope column merges both vertical neighbours into `num` and leaves
    /// the sides open — the mask `UP|DOWN` picks the (90, 0/18/36) vertical
    /// segment row (the same column the back-rope draw reads at X 90).
    #[test]
    fn w4_g82_a_rope_column_merges_up_and_down() {
        let w = world_with(&[(10, 9, 213), (10, 10, 213), (10, 11, 213)]);
        // Slots as the generic pass sees them: same-kind neighbours already
        // merged to `num` by the same-type fold — the arm's `!= num` guards
        // make those writes no-ops, exactly like the C#.
        let r = rope_frame_arm(&w, 10, 10, 213, 213, 213, -1, -1);
        assert_eq!((r.up, r.down, r.left, r.right), (213, 213, -1, -1));
        assert!(!r.reframe_up && !r.reframe_down);
        // A rope over a DIFFERENT rope kind (silk over vine): the IsRope
        // probes (`:81896`/`:81900`) still merge — the column is contiguous.
        let w = world_with(&[(10, 9, 365), (10, 10, 213), (10, 11, 353)]);
        let r = rope_frame_arm(&w, 10, 10, 213, 365, 353, -1, -1);
        assert_eq!((r.up, r.down), (213, 213));
    }

    /// The mystic snake rope 504 skips the upward IsRope merge, the
    /// solid-above merge AND the side merges (`num != 504` on `:81896`,
    /// `:81904`, `:81912`) — but keeps the downward merge (`:81900` has no
    /// 504 guard).
    #[test]
    fn w4_g82_the_mystic_snake_rope_never_merges_up_or_sideways() {
        let w = world_with(&[
            (10, 9, 504),
            (10, 10, 504),
            (10, 11, 504),
            (9, 10, 1),
            (11, 10, 1),
        ]);
        let r = rope_frame_arm(&w, 10, 10, 504, 504, 504, 1, 1);
        assert_eq!((r.up, r.down, r.left, r.right), (504, 504, 1, 1));
        // With a rope above but slots NOT pre-merged, up stays unmerged.
        let r = rope_frame_arm(&w, 10, 10, 504, 213, 213, 1, 1);
        assert_eq!(r.up, 213, "the :81896 504 guard");
        assert_eq!(r.down, 504, "the :81900 merge has no 504 guard");
        assert_eq!((r.left, r.right), (1, 1), "the :81912 504 guard");
    }

    /// The Dead Cells display jar 698 anchors a rope's bottom end even
    /// though it is not a rope and not solid (`down == 698`, `:81900`).
    #[test]
    fn w4_g82_the_display_jar_anchors_the_bottom() {
        let w = world_with(&[(10, 10, 213), (10, 11, 698)]);
        let r = rope_frame_arm(&w, 10, 10, 213, -1, 698, -1, -1);
        assert_eq!(r.down, 213);
        assert!(!r.reframe_down);
        // Any other non-rope non-solid neighbour does NOT anchor.
        let w = world_with(&[(10, 10, 213), (10, 11, 4)]);
        let r = rope_frame_arm(&w, 10, 10, 213, -1, 4, -1, -1);
        assert_eq!(r.down, 4);
    }

    /// A full block above anchors the top (`tileSolid && !tileSolidTop`,
    /// `:81904`); a platform above does NOT (solid-top excluded) — but ANY
    /// solid below anchors the bottom (`:81908`, solid-top included).
    #[test]
    fn w4_g82_solid_anchors_top_full_bottom_any() {
        let w = world_with(&[(10, 9, 1), (10, 10, 213), (10, 11, 19)]);
        assert!(td::TILE_SOLID.has(1) && !td::TILE_SOLID_TOP.has(1));
        assert!(td::TILE_SOLID.has(19) && td::TILE_SOLID_TOP.has(19));
        let r = rope_frame_arm(&w, 10, 10, 213, 1, 19, -1, -1);
        assert_eq!((r.up, r.down), (213, 213));
        // The platform ABOVE stays unmerged (the !solidTop guard).
        let w = world_with(&[(10, 9, 19), (10, 10, 213)]);
        let r = rope_frame_arm(&w, 10, 10, 213, 19, -1, -1, -1);
        assert_eq!(r.up, 19);
        assert!(r.reframe_down, "down ended -1 (:81927)");
    }

    /// The side merges fire only when the top did NOT merge (`up != num`,
    /// `:81912`): a rope hanging beside a wall with open air above draws
    /// the side-attached sprite; a rope in a column never does.
    #[test]
    fn w4_g82_sides_merge_only_when_the_top_is_open() {
        let w = world_with(&[(10, 10, 213), (9, 10, 1), (11, 10, 1)]);
        let r = rope_frame_arm(&w, 10, 10, 213, -1, -1, 1, 1);
        assert_eq!((r.left, r.right), (213, 213));
        assert!(r.reframe_up && r.reframe_down);
        // With the top merged the sides stay as they were.
        let w = world_with(&[(10, 9, 1), (10, 10, 213), (9, 10, 1), (11, 10, 1)]);
        let r = rope_frame_arm(&w, 10, 10, 213, 1, -1, 1, 1);
        assert_eq!(r.up, 213);
        assert_eq!((r.left, r.right), (1, 1), "the :81912 up != num gate");
    }

    /// The IsRope probes see through platforms: a platform with rope on
    /// both ends is a rope end (`WorldGen.cs:70094-70103`), so the column
    /// merge continues through it.
    #[test]
    fn w4_g82_the_probes_see_through_anchored_platforms() {
        let w = world_with(&[
            (10, 8, 213),
            (10, 9, 19),
            (10, 10, 213),
            (10, 11, 19),
            (10, 12, 213),
        ]);
        // The platform at y-1 has rope above (y-2) and below (y) — IsRope
        // true; the arm merges up through it.
        let r = rope_frame_arm(&w, 10, 10, 213, 19, 19, -1, -1);
        assert_eq!((r.up, r.down), (213, 213));
        // A platform with NO rope below is not a rope end — but it is
        // solid, so the bottom still anchors via :81908.
        let w = world_with(&[(10, 8, 213), (10, 9, 19), (10, 10, 213), (10, 11, 19)]);
        let r = rope_frame_arm(&w, 10, 10, 213, 19, 19, -1, -1);
        assert_eq!(r.up, 213, "the platform above is a rope end");
        assert_eq!(r.down, 213, "the solid-bottom anchor, not IsRope");
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
