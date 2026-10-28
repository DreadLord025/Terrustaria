//! Tree composition data (`TileDrawing.DrawTrees`).
//!
//! A tree is a column of tile id 5. The trunk sheet uses a **22 px** frame grid and
//! draws 20x20 sprites (art is inset 2 px), unlike the 18/16 px block tilesets. The
//! leafy canopy and branches are not tiles at all: the top and the side tiles are
//! transparent *markers* whose frames trigger `Tree_Tops_*` / `Tree_Branches_*`
//! sprites, positioned relative to the marker tile.

use crate::world::World;

pub const TREE: u16 = 5;
/// Frame stride of `Tiles_5.png` and the other tree trunk sheets.
pub const TRUNK_GRID: i32 = 22;
/// Trunk sprite size in the sheet and on screen.
pub const TRUNK_SIZE: f32 = 20.0;

/// Tile ids that `TileDrawing.DrawTrees` handles as trees.
pub fn is_tree(kind: u16) -> bool {
    matches!(kind, 5 | 583..=589 | 596 | 616 | 634)
}

/// Transparent marker part stored in a tree tile.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TreePart {
    /// `frameX = 22`, `frameY >= 198`: canopy anchor.
    Top,
    /// `frameX = 44`: marker one tile left of the trunk; branch extends left.
    BranchLeft,
    /// `frameX = 66`: marker one tile right of the trunk; branch extends right.
    BranchRight,
}

impl TreePart {
    #[inline]
    pub fn marker(frame_x: i16, frame_y: i16) -> Option<TreePart> {
        if frame_y < 198 {
            return None;
        }
        match frame_x {
            22 => Some(TreePart::Top),
            44 => Some(TreePart::BranchLeft),
            66 => Some(TreePart::BranchRight),
            _ => None,
        }
    }

    /// Offset from this marker to the trunk column (`WorldGen.GetTreeBiome`).
    #[inline]
    pub fn trunk_offset(self) -> i32 {
        match self {
            TreePart::Top => 0,
            TreePart::BranchLeft => 1,
            TreePart::BranchRight => -1,
        }
    }
}

/// `WorldGen.GetTreeFrame`: frameY 198/220/242 pick the top/branch variant.
#[inline]
pub fn tree_frame(frame_y: i16) -> usize {
    match frame_y {
        220 => 1,
        242 => 2,
        _ => 0,
    }
}

/// Biome variant from `TileDrawing.GetTreeVariant` (-1 = forest). Selects the
/// 176 px section of `Tiles_5.png` and the foliage texture set.
#[inline]
pub fn biome(kind: u16) -> i32 {
    match kind {
        23 | 661 => 0,  // corruption
        60 => 1,        // jungle (surface)
        109 | 492 => 2, // hallow
        147 => 3,       // snow
        199 | 662 => 4, // crimson
        70 => 6,        // glowing mushroom
        _ => -1,        // forest
    }
}

/// `tileFrameX += 176 * (treeBiome + 1)` in `GetTileDrawData`.
#[inline]
pub fn trunk_sheet_offset(kind: u16) -> i32 {
    let v = biome(kind);
    if v < 0 { 0 } else { 176 * (v + 1) }
}

/// Texture set used for canopy and branch sprites.
#[derive(Clone, Copy, Debug)]
pub struct Foliage {
    /// `Tree_Tops_{style}` / `Tree_Branches_{style}`.
    pub style: usize,
    /// Width/height of one canopy frame (`topTextureFrameWidth/Height`).
    pub top_w: f32,
    pub top_h: f32,
    /// `treeFrame`: 0..3, indexes canopies and branch rows.
    pub frame: usize,
}

impl Foliage {
    /// Chooses the texture set from the ground tile the tree grows on. The original
    /// also randomises forest styles per world region (`TreeTops.GetTreeStyle`); our
    /// generated world uses style 0, so loaded forest trees do too.
    pub fn for_ground(ground_kind: u16, frame_y: i16) -> Self {
        let frame = tree_frame(frame_y);
        match biome(ground_kind) {
            1 => Self {
                style: 2,
                top_w: 114.0,
                top_h: 96.0,
                frame,
            },
            2 => Self {
                style: 3,
                top_w: 80.0,
                top_h: 140.0,
                frame,
            },
            3 => Self {
                style: 4,
                top_w: 80.0,
                top_h: 80.0,
                frame,
            },
            4 => Self {
                style: 5,
                top_w: 80.0,
                top_h: 80.0,
                frame,
            },
            6 => Self {
                style: 14,
                top_w: 80.0,
                top_h: 80.0,
                frame,
            },
            0 => Self {
                style: 1,
                top_w: 80.0,
                top_h: 80.0,
                frame,
            },
            _ => Self {
                style: 0,
                top_w: 80.0,
                top_h: 80.0,
                frame,
            },
        }
    }
}

/// Finds the ground tile below a tree marker and returns its foliage data.
/// Mirrors `WorldGen.GetCommonTreeFoliageData`'s downward search (100 tiles).
pub fn foliage_at(world: &World, x: i32, y: i32, part: TreePart, frame_y: i16) -> Foliage {
    let tx = x + part.trunk_offset();
    for dy in 0..100 {
        let Some(t) = world.tile(tx, y + dy) else {
            break;
        };
        if t.active() && !is_tree(t.kind) {
            return Foliage::for_ground(t.kind, frame_y);
        }
    }
    Foliage::for_ground(2, frame_y)
}

/// Biome section of `Tiles_5.png` for a trunk tile, found by following the trunk
/// column down to the ground (`TileDrawing.GetTreeBiome`). Forest trees use the
/// first 176 px section (offset 0).
pub fn trunk_sheet_offset_at(world: &World, x: i32, y: i32) -> i32 {
    for dy in 0..100 {
        let Some(t) = world.tile(x, y + dy) else {
            break;
        };
        if t.active() && !is_tree(t.kind) {
            return trunk_sheet_offset(t.kind);
        }
    }
    0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn markers_and_frames_match_vanilla() {
        assert_eq!(TreePart::marker(22, 198), Some(TreePart::Top));
        assert_eq!(TreePart::marker(44, 220), Some(TreePart::BranchLeft));
        assert_eq!(TreePart::marker(66, 242), Some(TreePart::BranchRight));
        assert_eq!(TreePart::marker(22, 176), None);
        assert_eq!(tree_frame(220), 1);
        assert_eq!(tree_frame(242), 2);
        assert_eq!(tree_frame(198), 0);
    }

    #[test]
    fn forest_uses_first_sheet_section() {
        assert_eq!(trunk_sheet_offset(2), 0);
        assert_eq!(trunk_sheet_offset(147), 176 * 4);
        assert_eq!(Foliage::for_ground(2, 198).style, 0);
        assert_eq!(Foliage::for_ground(60, 198).top_h, 96.0);
    }
}
