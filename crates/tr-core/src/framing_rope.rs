//! W4-g82 — the rope framing arm of `WorldGen.TileFrameCosmetic`
//! (`WorldGen.cs:81894-81931`), the tr-core home of the decision maths
//! `tr-app/render/ropes.rs::rope_frame_arm` ports verbatim (the d56
//! inlined-table convention: the framing pass lives in tr-core, the render
//! twin stays its own copy with its own witnesses). Wired into
//! [`crate::framing::frame_of_variant`] after the dirt/moss merge families
//! and BEFORE the mask build — the C#'s `else if (Main.tileRope[num])` arm
//! sits between the dirt branch (`:81743-81893`) and the per-type switch
//! (`:81934`). For rope kinds the generic families above are all no-ops
//! (no rope id is in BLEND_ALL / the dirt / mud / moss sets), so the late
//! call site is observation-identical to the C#'s arm position.
//!
//! The two neighbour re-frame requests (`:81923-81930`) only WRITE the
//! neighbours' frames — they never change this cell's neighbour KINDS, so
//! this cell's mask is independent of them; the port's square reframe
//! (`framing::reframe_area` after every edit, and the full `frame_world`
//! pass) covers the neighbour writes (recorded deviation, ANALYSIS "The
//! rope-coil lane (W4-g82)").

use crate::tiledata as td;
use crate::world::World;

/// `WorldGen.IsRope(x, y)` — the 2-arg overload (`WorldGen.cs:70074-70079`):
/// a rope cell answers true; the minecart track (314), the planter (380)
/// and platforms answer true only with rope on BOTH ends
/// (`GetRopeEnds`, `:70026-70072`, rangeToCheck 5, treatEmptyAsTheRopeEnd
/// false — incl. the no-top-end down-scan degeneration).
fn is_rope_cell(world: &World, x: i32, y: i32) -> bool {
    let Some(t) = world.tile(x, y) else {
        return false;
    };
    if !t.active() {
        return false;
    }
    if td::TILE_ROPE.has(t.kind) {
        return true;
    }
    if t.kind != 314 && t.kind != 380 && !td::SET_PLATFORMS.has(t.kind) {
        return false;
    }
    let mut top = -1;
    let mut i = 1;
    while i <= 5 {
        let Some(u) = world.tile(x, y - i) else { break };
        if !u.active() {
            break;
        }
        if td::TILE_ROPE.has(u.kind) {
            top = y - i;
            break;
        }
        i += 1;
    }
    if top == -1 {
        return false;
    }
    let mut j = 1 + (y - top);
    while j <= 6 {
        let Some(d) = world.tile(x, top + j) else { break };
        if !d.active() {
            break;
        }
        if td::TILE_ROPE.has(d.kind) {
            return true;
        }
        j += 1;
    }
    false
}

/// `TileFrameCosmetic`'s rope arm (`WorldGen.cs:81894-81931`), verbatim —
/// the seven-write order the render twin documents. `num` is the framing
/// kind; `up/down/left/right` are the four side slots AFTER the slope
/// adjustments and the generic merges. Returns the merged side slots (the
/// `-1` re-frame requests are the caller's square-reframe concern, see the
/// module doc).
pub fn rope_frame_arm(
    world: &World,
    x: i32,
    y: i32,
    num: i32,
    up: i32,
    down: i32,
    left: i32,
    right: i32,
) -> (i32, i32, i32, i32) {
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
    (up, down, left, right)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tile::FLAG_ACTIVE;

    fn world_with(tiles: &[(i32, i32, u16)]) -> World {
        let mut w = World::new("rope-frame".into(), 40, 40);
        for &(x, y, kind) in tiles {
            let t = w.tile_mut(x, y).unwrap();
            t.kind = kind;
            t.flags |= FLAG_ACTIVE;
        }
        w
    }

    // WorldGen.cs:81896/:81900 — a contiguous column merges both ends,
    // even across rope KINDS (the IsRope probe is kind-blind).
    #[test]
    fn w4_g82_the_core_arm_merges_the_column_across_kinds() {
        let w = world_with(&[(10, 9, 365), (10, 10, 213), (10, 11, 353)]);
        assert_eq!(rope_frame_arm(&w, 10, 10, 213, 365, 353, -1, -1), (213, 213, -1, -1));
    }

    // :81896/:81904/:81912 — the mystic snake 504 guards; :81900 has none.
    #[test]
    fn w4_g82_the_core_arm_keeps_the_504_guards() {
        let w = world_with(&[(10, 9, 504), (10, 10, 504), (10, 11, 504), (9, 10, 1), (11, 10, 1)]);
        assert_eq!(rope_frame_arm(&w, 10, 10, 504, 213, 213, 1, 1), (213, 504, 1, 1));
    }

    // :81900 — the Dead Cells display jar 698 anchors the bottom; :81904
    // excludes solid-top above; :81908 includes any solid below.
    #[test]
    fn w4_g82_the_core_arm_anchors_jar_bottom_and_solid_sides() {
        let w = world_with(&[(10, 10, 213), (10, 11, 698)]);
        assert_eq!(rope_frame_arm(&w, 10, 10, 213, -1, 698, -1, -1).1, 213);
        let w = world_with(&[(10, 9, 19), (10, 10, 213), (10, 11, 19)]);
        assert_eq!(rope_frame_arm(&w, 10, 10, 213, 19, 19, -1, -1), (19, 213, -1, -1));
        // :81912 — the side merges fire only while the top stayed open.
        let w = world_with(&[(10, 10, 213), (9, 10, 1), (11, 10, 1)]);
        assert_eq!(rope_frame_arm(&w, 10, 10, 213, -1, -1, 1, 1), (-1, -1, 213, 213));
    }

    // The wired frame: a rope column frames the UP|DOWN vertical-segment
    // row (90, variant*18) through `frame_of_variant` — the same column
    // the back-rope draw reads (RULES' UP|DOWN row, framing.rs:114).
    #[test]
    fn w4_g82_frame_of_variant_frames_a_rope_column_vertically() {
        let w = world_with(&[(10, 9, 213), (10, 10, 213), (10, 11, 213)]);
        for v in 0..3 {
            let (fx, fy) = crate::framing::frame_of_variant(&w, 10, 10, v).unwrap();
            assert_eq!((fx, fy), (90, 18 * v as i16), "variant {v}");
        }
        // A rope hanging beside walls with open air above/below takes the
        // side-merged LEFT|RIGHT row (108/126/144, 72) — the :81912 arm.
        let w = world_with(&[(10, 10, 213), (9, 10, 1), (11, 10, 1)]);
        let (fx, fy) = crate::framing::frame_of_variant(&w, 10, 10, 0).unwrap();
        assert_eq!((fx, fy), (108, 72));
    }
}
