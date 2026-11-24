//! W4-g31 TILEPASS final — the `IsVisible` keep-one survivor
//! (`TileDrawing.cs:1411-1432`), promoted per research/136 §4's correction:
//! d56's tip has NO standalone fn (the logic is inlined in `tile_draw_gate`,
//! `animated_tiles.rs:489-503`); the only standalone pub copy is d60's
//! `tile_draw_particles.rs:442`. This module IS that promotion — d60's
//! shape, adopted with ONE correction: the `invisibleBlock()` bit is a
//! PARAMETER, not the hardcoded `false` d60 reads (the bit is unported in
//! d60's module; the union's callers own `World::invisible_block`, so the
//! promoted copy takes the real bit and stays tr-core-free).
//!
//! Shape note (recorded deviation from research/136 §4's "takes `&Tile`"):
//! tr-render has no tr-core dependency (Cargo.toml), and d56's
//! `tile_draw_gate` already takes `kind`/`frame_y`/`invisible_block` as
//! primitives — the promoted fn takes the same primitives so d56's gate,
//! d60's battery and the g31/pylons wrappers all bind without a crate
//! dependency change. At the family merge d60's copy DROPs onto this one
//! (passing `false` for the bit keeps its exact behaviour) and d56's
//! inlined block calls it.
//!
//! Consumers on this branch: `render/tile_drains.rs::is_visible` and
//! `render/pylons.rs::is_visible` are thin world-reading wrappers over
//! this fn (the g31/pylons rebind of UNION-CONTRACT §1.4, executed).

/// `TileDrawing.IsVisible` (`TileDrawing.cs:1411-1432`): an Echo-coated
/// block hides unless sight is granted; the martian-conduit platform row
/// 48 (`frameY / 18 == 48`) and the two invisible wiring tiles (541/631)
/// ALWAYS ride the sight flag regardless of the coating bit.
pub fn is_visible(
    kind: u16,
    frame_y: i16,
    invisible_block: bool,
    show_invisible_blocks: bool,
) -> bool {
    let mut flag = invisible_block;
    match kind {
        19 => {
            if frame_y as i32 / 18 == 48 {
                flag = true;
            }
        }
        541 | 631 => flag = true,
        _ => {}
    }
    if flag {
        return show_invisible_blocks;
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The coating bit gates every kind; sight grants it back (`:1413`,
    /// `:1427-1430`). An uncoated ordinary block is always visible.
    #[test]
    fn w4_g31_is_visible_echo_bit_rides_the_sight_flag() {
        assert!(is_visible(1, 0, false, false));
        assert!(!is_visible(1, 0, true, false));
        assert!(is_visible(1, 0, true, true));
    }

    /// The platform row-48 arm (`:1416-1421`): ONLY `frameY / 18 == 48`
    /// (864..881) forces the flag on an uncoated platform; every other
    /// row draws normally. The division is the C# int division.
    #[test]
    fn w4_g31_is_visible_platform_row48_only() {
        assert!(!is_visible(19, 48 * 18, false, false));
        assert!(!is_visible(19, 48 * 18 + 17, false, false));
        assert!(is_visible(19, 48 * 18, false, true));
        assert!(is_visible(19, 47 * 18, false, false));
        assert!(is_visible(19, 0, false, false));
        // A non-platform kind on the same row is NOT gated.
        assert!(is_visible(1, 48 * 18, false, false));
    }

    /// 541/631 (`:1422-1425`) are sight-gated on EVERY row, coating or
    /// not — d60's shape, verbatim.
    #[test]
    fn w4_g31_is_visible_wiring_tiles_always_gated() {
        for kind in [541u16, 631] {
            assert!(!is_visible(kind, 0, false, false));
            assert!(!is_visible(kind, 36, false, false));
            assert!(is_visible(kind, 0, false, true));
        }
    }
}
