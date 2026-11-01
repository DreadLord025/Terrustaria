//! Pure composite tables for the player draw: the frame cells, the shoulder
//! order, the head-gear bob and the weapon slot of `PlayerDrawSet` /
//! `PlayerDrawLayers`. No render types — the builders (phase 2) turn these
//! tables into `LayerSprite` lists and the render passes only push them.

/// One drawable sprite produced by a layer builder: position and size in world
/// pixels, an atlas UV quad, a pre-multiplied tint and a top-left rotation.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct LayerSprite {
    pub(crate) pos: [f32; 2],
    pub(crate) size: [f32; 2],
    pub(crate) uv: [f32; 4],
    pub(crate) color: [f32; 4],
    pub(crate) rotation: f32,
}

/// The per-frame inputs every layer builder reads (parameter object — the
/// cc-routine-and-class-design 7±2 cap; fields grow with the builders).
#[derive(Clone, Copy, Debug)]
pub(crate) struct PlayerCompositeState {
    pub(crate) leg_frame: u32,
    pub(crate) body_frame: u32,
    pub(crate) facing: f32,
}

/// `WeaponDrawOrder` (`PlayerDrawSet.cs:358`): which of the three
/// `DrawPlayer_27_HeldItem` call slots the held item occupies.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum WeaponDrawOrder {
    BehindBackArm,
    BehindFrontArm,
    OverFrontArm,
}

/// The front-arm composite cell per body frame — the `frameIndex2` switch of
/// `PlayerDrawSet.CreateCompositeData` (`PlayerDrawSet.cs:1952-2023`), with the
/// out-of-range clamp falling back to the standing frame like the original's
/// `bodyFrame` layout.
pub(crate) fn front_arm_cell(frame: u32) -> (u32, u32) {
    match frame {
        0 => (2, 0),
        1 => (3, 0),
        2 => (4, 0),
        3 => (5, 0),
        4 => (6, 0),
        5 => (2, 1),
        6 => (3, 1),
        7..=10 => (4, 1),
        11..=13 => (3, 1),
        14 => (5, 1),
        15..=16 => (6, 1),
        17 => (5, 1),
        18..=19 => (3, 1),
        _ => (2, 0),
    }
}

/// The back arm reuses the front cell two rows down
/// (`frameIndex.Y = frameIndex2.Y + 2`, `PlayerDrawSet.cs:2034`).
pub(crate) fn back_arm_cell(frame: u32) -> (u32, u32) {
    let (col, row) = front_arm_cell(frame);
    (col, row + 2)
}

/// Front shoulder cell — `pt2 = (0, 1)` (`PlayerDrawSet.cs:1939`).
pub(crate) const FRONT_SHOULDER_CELL: (u32, u32) = (0, 1);

/// Back shoulder cell — `pt = (1, 1)` (`PlayerDrawSet.cs:1938`).
pub(crate) const BACK_SHOULDER_CELL: (u32, u32) = (1, 1);

/// Torso skin cell — `pt3 = default` with the case-5 override to column 1
/// (`PlayerDrawSet.cs:1940,1975`).
pub(crate) fn torso_cell(frame: u32) -> (u32, u32) {
    if frame == 5 {
        (1, 0)
    } else {
        (0, 0)
    }
}

/// `compShoulderOverFrontArm`: when true the shoulder stack draws after (over)
/// the arm stack. Default true; rows 1, 2 and 5 clear it
/// (`PlayerDrawSet.cs:1956,1961,1966,1971,1976`).
pub(crate) fn shoulder_over_front_arm(frame: u32) -> bool {
    !matches!(frame, 1 | 2 | 5)
}

/// The raw `OffsetsPlayerHeadgear` table row — Y is 0 exactly at rows 7-9 and
/// 14-16, 2 elsewhere (`Main.cs:66638-66662`).
pub(crate) fn headgear_offset(frame: u32) -> f32 {
    match frame {
        7..=9 | 14..=16 => 0.0,
        _ => 2.0,
    }
}

/// The shift the composite consumers apply: `vector2.Y -= 2; vector += vector2
/// * -1` for the un-flipped case (`PlayerDrawLayers.cs:1217-1219`) — so the
/// torso and both arm stacks sit 2 px lower on the zero rows, legs never shift.
pub(crate) fn headgear_composite_shift(frame: u32) -> f32 {
    2.0 - headgear_offset(frame)
}

/// The held-item slot for the starter player: `weaponDrawOrder` defaults to
/// `BehindFrontArm` (`PlayerDrawSet.cs:358`); the 4952 pet and golf-club
/// overrides (`359-365`) are out of scope.
pub(crate) fn weapon_draw_order() -> WeaponDrawOrder {
    WeaponDrawOrder::BehindFrontArm
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dw_1_2_front_arm_cell_matches_c_switch() {
        // `PlayerDrawSet.cs:1952-2023`, one case per row.
        let expected: [(u32, u32); 20] = [
            (2, 0),
            (3, 0),
            (4, 0),
            (5, 0),
            (6, 0),
            (2, 1),
            (3, 1),
            (4, 1),
            (4, 1),
            (4, 1),
            (4, 1),
            (3, 1),
            (3, 1),
            (3, 1),
            (5, 1),
            (6, 1),
            (6, 1),
            (5, 1),
            (3, 1),
            (3, 1),
        ];
        for (frame, cell) in expected.iter().enumerate() {
            assert_eq!(front_arm_cell(frame as u32), *cell, "frame {frame}");
        }
        // Out-of-range clamp: standing frame.
        assert_eq!(front_arm_cell(20), (2, 0));
        // The back arm is the same cell two rows down (`:2034`).
        assert_eq!(back_arm_cell(0), (2, 2));
        assert_eq!(back_arm_cell(7), (4, 3));
    }

    #[test]
    fn test_dw_1_3_headgear_raw_table_and_composite_shift() {
        // Raw `OffsetsPlayerHeadgear` (`Main.cs:66638-66662`): zero rows.
        for frame in [7u32, 8, 9, 14, 15, 16] {
            assert_eq!(headgear_offset(frame), 0.0, "raw frame {frame}");
        }
        // Every other row is 2.
        for frame in [0u32, 6, 10, 13, 17, 19] {
            assert_eq!(headgear_offset(frame), 2.0, "raw frame {frame}");
        }
        // Composite consumers shift by `2 − y` (`PlayerDrawLayers.cs:1217-1219`).
        assert_eq!(headgear_composite_shift(7), 2.0);
        assert_eq!(headgear_composite_shift(0), 0.0);
    }

    #[test]
    fn test_dw_1_4_shoulder_order_and_weapon_slot() {
        // Rows 1, 2, 5 clear the flag; 3, 4 set it; the rest default true.
        assert!(!shoulder_over_front_arm(1));
        assert!(!shoulder_over_front_arm(2));
        assert!(!shoulder_over_front_arm(5));
        assert!(shoulder_over_front_arm(3));
        assert!(shoulder_over_front_arm(4));
        assert!(shoulder_over_front_arm(0));
        assert!(shoulder_over_front_arm(19));
        // The starter's weapon slot is the default (`PlayerDrawSet.cs:358`).
        assert_eq!(weapon_draw_order(), WeaponDrawOrder::BehindFrontArm);
    }

    #[test]
    fn torso_cell_overrides_frame_5() {
        assert_eq!(torso_cell(0), (0, 0));
        assert_eq!(torso_cell(5), (1, 0));
        assert_eq!(torso_cell(19), (0, 0));
        assert_eq!(FRONT_SHOULDER_CELL, (0, 1));
        assert_eq!(BACK_SHOULDER_CELL, (1, 1));
    }
}
