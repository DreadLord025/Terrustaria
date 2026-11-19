//! Tree composition data (`TileDrawing.DrawTrees`) and the tree family
//! queries the kill cascade needs (`WorldGen.GetTreeType`,
//! `WorldGen.GetTreeBottom`, `WorldGen.TreeTypeDropsAcorns`).
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

/// `TileID.Sets.IsATreeTrunk` (`TileID.cs:487`): the trunk kinds the tree
/// walks count as "still a tree" — 5, 72 (the ash trunk) and the variant
/// kinds 583-589/596/616/634. `is_tree` is the *drawing* set (no 72); this is
/// the C#'s kill-side set.
pub fn is_tree_trunk(kind: u16) -> bool {
    matches!(kind, 5 | 72 | 583..=589 | 596 | 616 | 634)
}

/// `WorldGen.TreeTypes` — the biome family a tree belongs to. The C# enum
/// (`WorldGen.cs`, `TreeTypes`) names the same members.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TreeType {
    None,
    Forest,
    Corrupt,
    Mushroom,
    Crimson,
    Jungle,
    Snow,
    Hallowed,
    Palm,
    PalmCorrupt,
    PalmCrimson,
    PalmHallowed,
    Ash,
}

/// `WorldGen.GetTreeType(int tileType)` (`WorldGen.cs:63075-63110`): the
/// ground tile under a tree names its family.
pub fn tree_type_of(kind: u16) -> TreeType {
    match kind {
        2 | 477 => TreeType::Forest,
        23 | 661 => TreeType::Corrupt,
        70 => TreeType::Mushroom,
        199 | 662 => TreeType::Crimson,
        60 => TreeType::Jungle,
        147 => TreeType::Snow,
        109 | 492 => TreeType::Hallowed,
        53 => TreeType::Palm,
        112 => TreeType::PalmCorrupt,
        234 => TreeType::PalmCrimson,
        116 => TreeType::PalmHallowed,
        633 => TreeType::Ash,
        _ => TreeType::None,
    }
}

/// `WorldGen.TreeTypeDropsAcorns` (`WorldGen.cs:63112-63119`): every family
/// but the mushroom, the jungle and "none" drops acorns.
pub fn tree_type_drops_acorns(tree_type: TreeType) -> bool {
    !matches!(
        tree_type,
        TreeType::None | TreeType::Mushroom | TreeType::Jungle
    )
}

/// The wood item of a family — `KillTile_GetTreeDrops`'s switch
/// (`WorldGen.cs:66224-66254`). The mushroom is the RNG arm (183 or nothing)
/// and the forest keeps the default Wood (9).
pub fn wood_item_for(tree_type: TreeType) -> u16 {
    match tree_type {
        TreeType::Ash => 5215,
        TreeType::Corrupt => 619,
        TreeType::Crimson => 911,
        TreeType::Jungle => 620,
        TreeType::Hallowed => 621,
        TreeType::Snow => 2503,
        TreeType::Mushroom => 183,
        _ => 9,
    }
}

/// `WorldGen.GetTreeBottom` (`WorldGen.cs:63436-63482`): the trunk column's
/// ground cell. The palm walks down over the empty and palm cells; the other
/// trees shift one column by their marker frame and then walk down over the
/// trunk cells.
pub fn tree_bottom(world: &World, x: i32, y: i32) -> (i32, i32) {
    let mut bx = x;
    let mut by = y;
    let Some(cell) = world.tile(x, y) else {
        return (bx, by);
    };
    if cell.kind == 323 {
        // The palm: `while (y < maxTilesY - 1 && (!tile.active() || tile.type == 323))`.
        while by < world.height as i32 - 1 {
            let cont = match world.tile(bx, by) {
                None => true,
                Some(t) => !t.active() || t.kind == 323,
            };
            if !cont {
                break;
            }
            by += 1;
        }
        return (bx, by);
    }
    // The marker frame picks the trunk column: `frameX / 22` and `frameY / 22`.
    let num = i32::from(cell.frame_x) / 22;
    let num2 = i32::from(cell.frame_y) / 22;
    if num == 3 && num2 <= 2 {
        bx += 1;
    } else if num == 4 && (3..=5).contains(&num2) {
        bx -= 1;
    } else if num == 1 && (6..=8).contains(&num2) {
        bx -= 1;
    } else if num == 2 && (6..=8).contains(&num2) {
        bx += 1;
    } else if num == 2 && num2 >= 9 {
        bx += 1;
    } else if num == 3 && num2 >= 9 {
        bx -= 1;
    }
    // The walk down over the trunk cells.
    while by < world.height as i32 - 1 {
        let cont = match world.tile(bx, by) {
            None => true,
            Some(t) => !t.active() || is_tree_trunk(t.kind),
        };
        if !cont {
            break;
        }
        by += 1;
    }
    (bx, by)
}

/// `WorldGen.GetTreeTypeAndTreeBottom` (`WorldGen.cs:63055-63066`): the
/// family's ground-derived type plus the trunk column's ground cell.
pub fn tree_type_and_bottom(world: &World, x: i32, y: i32) -> (TreeType, i32, i32) {
    let (bx, by) = tree_bottom(world, x, y);
    let tree_type = match world.tile(bx, by) {
        Some(t) if t.active() => tree_type_of(t.kind),
        _ => TreeType::None,
    };
    (tree_type, bx, by)
}

/// `TileFrameImportant`'s tree dispatch (`WorldGen.cs:85876-85911`): the
/// forest tree 5 runs `CheckTree`, the variant kinds run
/// `CheckTreeWithSettings` with their ground test, and the palm 323 runs
/// `CheckPalmTree`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TreeVariant {
    /// Tile 5 — `WorldGen.CheckTree` (`WorldGen.cs:54570-54780`).
    Forest,
    /// Tiles 583-589 — `CheckTreeWithSettings` + `GemTreeGroundTest`.
    Gem,
    /// Tiles 596/616 — `CheckTreeWithSettings` + `VanityTreeGroundTest`.
    Vanity,
    /// Tile 634 — `CheckTreeWithSettings` + `AshTreeGroundTest`.
    Ash,
    /// Tile 323 — `WorldGen.CheckPalmTree` (`WorldGen.cs:54346-54400`).
    Palm,
}

impl TreeVariant {
    /// The variant of a tree tile kind, or `None` for a non-tree kind.
    pub fn of(kind: u16) -> Option<TreeVariant> {
        match kind {
            5 => Some(TreeVariant::Forest),
            583..=589 => Some(TreeVariant::Gem),
            596 | 616 => Some(TreeVariant::Vanity),
            634 => Some(TreeVariant::Ash),
            323 => Some(TreeVariant::Palm),
            _ => None,
        }
    }

    /// The ground type the check compares against. The forest tree folds the
    /// grass family onto plain dirt (`WorldGen.cs:54611-54642`); the variants
    /// read the raw ground kind.
    pub fn ground_type(self, ground: i32) -> i32 {
        match self {
            TreeVariant::Forest => match ground {
                23 | 661 | 477 | 60 | 70 | 109 | 147 | 199 | 662 | 492 => 2,
                _ => ground,
            },
            _ => ground,
        }
    }

    /// Whether the ground below the tree supports it — the `flag` of
    /// `CheckTreeWithSettings` (`WorldGen.cs:54428`).
    ///
    /// The forest tree's own test is the folded dirt: `num4 == 2`
    /// (`WorldGen.cs:54643-54651`). The variants use their own ground test:
    /// `GemTreeGroundTest` (`:30304`), `VanityTreeGroundTest` (`:30317`) and
    /// `AshTreeGroundTest` (`:30330`).
    pub fn ground_valid(self, ground: i32) -> bool {
        match self {
            TreeVariant::Forest => self.ground_type(ground) == 2,
            TreeVariant::Gem => {
                ground >= 0
                    && (crate::tiledata::SET_CONVERSION_STONE.has(ground as u16)
                        || crate::tiledata::SET_CONVERSION_MOSS.has(ground as u16))
            }
            TreeVariant::Vanity => {
                ground >= 0
                    && crate::tiledata::SET_GRASS.has(ground as u16)
                    && ground != 23
                    && ground != 199
            }
            TreeVariant::Ash => ground == 633,
            TreeVariant::Palm => false,
        }
    }
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

/// Biome variant table of `TileDrawing.GetTreeVariant`
/// (TileDrawing.cs:1778-1808) applied to a GROUND kind, kind-only: the
/// jungle's surface/underground split needs the ground row and lives in
/// [`crate::drawdata::tree_variant`] (and the foliage probe
/// `GetCommonTreeFoliageData`, WorldGen.cs:62930-62944). -1 = forest.
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
    pub fn for_ground(kind: u16, ground_kind: u16, frame_y: i16) -> Option<Self> {
        let frame = tree_frame(frame_y);
        match TreeVariant::of(kind) {
            Some(variant @ (TreeVariant::Gem | TreeVariant::Vanity | TreeVariant::Ash)) => {
                if !variant.ground_valid(ground_kind as i32) {
                    return None;
                }
                return Some(match variant {
                    TreeVariant::Gem => Self {
                        style: 22 + (kind - 583) as usize,
                        top_w: 116.0,
                        top_h: 96.0,
                        frame,
                    },
                    TreeVariant::Vanity => Self {
                        style: if kind == 596 { 29 } else { 30 },
                        top_w: 118.0,
                        top_h: 96.0,
                        frame,
                    },
                    _ => Self {
                        style: 31,
                        top_w: 116.0,
                        top_h: 96.0,
                        frame,
                    },
                });
            }
            _ => {}
        }
        Some(match biome(ground_kind) {
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
        })
    }
}

/// Finds the ground tile below a tree marker and returns its foliage data.
/// Mirrors `WorldGen.GetCommonTreeFoliageData`'s downward search (100 tiles).
pub fn foliage_at(
    world: &World,
    x: i32,
    y: i32,
    part: TreePart,
    frame_y: i16,
    kind: u16,
) -> Option<Foliage> {
    let tx = x + part.trunk_offset();
    if let Some(variant) = TreeVariant::of(kind) {
        for dy in 0..100 {
            let Some(t) = world.tile(tx, y + dy) else {
                break;
            };
            if t.active() && !is_tree(t.kind) && variant.ground_valid(t.kind as i32) {
                return Some(Foliage::for_ground(kind, t.kind, frame_y).unwrap());
            }
        }
        return None;
    }
    for dy in 0..100 {
        let Some(t) = world.tile(tx, y + dy) else {
            break;
        };
        if t.active() && !is_tree(t.kind) {
            return Some(Foliage::for_ground(kind, t.kind, frame_y).unwrap());
        }
    }
    return Some(Foliage::for_ground(kind, 2, frame_y).unwrap());
}

/// Biome section of the trunk sheet (`Tiles_5.png`), the `GetTileDrawData`
/// case-5 rewrite `tileFrameX += 176 * (GetTreeBiome(x, y, fx, fy) + 1)`
/// (TileDrawing.cs:4645-4649). The probe shifts the column by the
/// trunk/branch marker frame and walks down cells of the START kind
/// (`GetTreeBiome`, TileDrawing.cs:1735-1776); forest trees (-1) keep the
/// first 176 px section (offset 0). The variant trunks (583-589/596/616/634)
/// have their own sheets and no shift (TileDrawing.cs:4650-4662).
pub fn trunk_sheet_offset_at(
    world: &World,
    x: i32,
    y: i32,
    kind: u16,
    frame_x: i16,
    frame_y: i16,
) -> i32 {
    if kind != TREE {
        return 0;
    }
    crate::drawdata::tree_frame_x_offset(world, x, y, frame_x, frame_y)
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
        assert_eq!(biome(2), -1);
        assert_eq!(biome(147), 3);
        assert_eq!(Foliage::for_ground(5, 2, 198).unwrap().style, 0);
        assert_eq!(Foliage::for_ground(5, 60, 198).unwrap().top_h, 96.0);
    }

    // ---- S1-F-9 wave A: the case-5 trunk sheet probe (GetTreeBiome) ----

    use crate::tile::FLAG_ACTIVE;

    fn put(w: &mut World, x: i32, y: i32, kind: u16) {
        let i = w.index(x, y);
        let mut t = crate::tile::Tile::default();
        t.kind = kind;
        t.flags |= FLAG_ACTIVE;
        w.tiles[i] = t;
    }

    #[test]
    fn trunk_probe_walks_the_start_kind_and_shifts_by_marker() {
        // Three trunk columns x=19/20/21, y=10..13, on snow/corrupt/crimson
        // ground. GetTreeBiome shifts the probe column by the marker frame
        // (TileDrawing.cs:1740-1771) and then walks down cells of the START
        // kind (kind 5 here).
        let mut w = World::new("t".into(), 64, 64);
        for x in 19..=21 {
            for y in 10..14 {
                put(&mut w, x, y, 5);
            }
        }
        put(&mut w, 19, 14, 147); // snow
        put(&mut w, 20, 14, 23); // corruption
        put(&mut w, 21, 14, 199); // crimson
        // Straight trunk cell (frame 0,0): walks down x=20 -> variant 0.
        assert_eq!(trunk_sheet_offset_at(&w, 20, 10, 5, 0, 0), 176 * 1);
        // Branch-right marker (66, >=198): column x-1 -> snow -> variant 3.
        assert_eq!(trunk_sheet_offset_at(&w, 20, 10, 5, 66, 242), 176 * 4);
        // Branch-left marker (44, >=198): column x+1 -> crimson -> variant 4.
        assert_eq!(trunk_sheet_offset_at(&w, 20, 10, 5, 44, 220), 176 * 5);
        // Trunk-region shifts: (66, fy<=45) -> x+1; (88, 66..110) -> x-1.
        assert_eq!(trunk_sheet_offset_at(&w, 20, 10, 5, 66, 44), 176 * 5);
        assert_eq!(trunk_sheet_offset_at(&w, 20, 10, 5, 88, 66), 176 * 4);
        assert_eq!(trunk_sheet_offset_at(&w, 20, 10, 5, 88, 110), 176 * 4);
        // Band 132..197: (22) -> x-1, (44) -> x+1.
        assert_eq!(trunk_sheet_offset_at(&w, 20, 10, 5, 22, 132), 176 * 4);
        assert_eq!(trunk_sheet_offset_at(&w, 20, 10, 5, 44, 176), 176 * 5);
        // Outside every shift window the column stays put.
        assert_eq!(trunk_sheet_offset_at(&w, 20, 10, 5, 66, 66), 176 * 1);
        assert_eq!(trunk_sheet_offset_at(&w, 20, 10, 5, 88, 132), 176 * 1);
        // The variant trunks keep their own sheets — no shift
        // (TileDrawing.cs:4650-4662).
        for kind in [583u16, 584, 585, 586, 587, 588, 589, 596, 616, 634] {
            assert_eq!(trunk_sheet_offset_at(&w, 20, 10, kind, 0, 0), 0);
        }
    }

    #[test]
    fn trunk_probe_stops_at_a_foreign_kind_not_at_any_non_tree() {
        // GetTreeBiome walks while type == START type (TileDrawing.cs:1772):
        // a foreign active tile mid-column stops the walk and IS the ground.
        let mut w = World::new("t".into(), 64, 64);
        put(&mut w, 20, 10, 5);
        put(&mut w, 20, 11, 60); // jungle mid-column: the C# stops here
        put(&mut w, 20, 12, 5);
        put(&mut w, 20, 13, 23);
        w.world_surface = 30.0;
        assert_eq!(trunk_sheet_offset_at(&w, 20, 10, 5, 0, 0), 176 * 2);
        // An inactive cell also stops the walk and classifies -1 (forest).
        let mut w = World::new("t".into(), 64, 64);
        put(&mut w, 20, 10, 5);
        let mut t = crate::tile::Tile::default();
        t.kind = 23;
        let i = w.index(20, 11);
        w.tiles[i] = t; // inactive corruption
        assert_eq!(trunk_sheet_offset_at(&w, 20, 10, 5, 0, 0), 0);
    }
}
