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

/// `WorldGen.ConvertTreeAndGround_Branches(x, y, newType)`
/// (`WorldGen.cs:55054-55065`, W4-f25 helper C7): the left neighbour that is
/// a branch/root marker pointing RIGHT (+1, its trunk is this column) and
/// the right neighbour pointing LEFT (-1) re-type to the new tree kind.
fn convert_tree_and_ground_branches(world: &mut World, x: i32, y: i32, new_type: u16) {
    // `:55057-55060` — the left cell's +1 markers.
    if is_tile_a_tree_branch(world, x - 1, y) == Some(1)
        || is_tile_a_tree_root(world, x - 1, y) == Some(1)
    {
        if let Some(t) = world.tile_mut(x - 1, y) {
            t.kind = new_type;
        }
    }
    // `:55061-55064` — the right cell's -1 markers.
    if is_tile_a_tree_branch(world, x + 1, y) == Some(-1)
        || is_tile_a_tree_root(world, x + 1, y) == Some(-1)
    {
        if let Some(t) = world.tile_mut(x + 1, y) {
            t.kind = new_type;
        }
    }
}

/// `WorldGen.ConvertTreeAndGround(x, y, newFloorType, oldTreeType,
/// newTreeType)` (`WorldGen.cs:54902-55052`, W4-f25 helper C7): the tree
/// standing on the converted ground cell re-types to the new tree family
/// (the 596/616 vanity → 5/323 and 5 ↔ 323 conversions of
/// `TryConvertingOrKillingTreesAboveIfTheyWouldBecomeInvalid`
/// `:54855-54896`). `rng` is the caller's `WorldGen.genRand` stream (the
/// g39 explicit-stream precedent) — the frame rolls draw in C# order.
/// WIRED: `spread::try_kill_invalid_tree_above` (the Convert/clentaminator
/// path) calls it on the runtime `frame_rng` stand-in; the growth-tick copy
/// (`growth/trees.rs::kill_invalid_tree_above`) stays kill-only (g17's lane).
/// The `NetMessage.SendTileSquare` tail (`:55051`) is MP-only.
pub fn convert_tree_and_ground(
    world: &mut World,
    rng: &mut crate::UnifiedRandom,
    x: i32,
    y: i32,
    new_floor_type: u16,
    old_tree_type: u16,
    new_tree_type: u16,
) {
    // `:54904-54907` — InWorld(5).
    if x < 5 || y < 5 || x >= world.width - 5 || y >= world.height - 5 {
        return;
    }
    // `:54908-54912` — the ground cell below must be active (the final
    // `tile.type = newFloorType` write targets it).
    if !world.tile(x, y + 1).is_some_and(|t| t.active()) {
        return;
    }
    // `:54913-54929` — the trunk-height walk: num counts down from 100,
    // num2 counts the trunk cells above. The C#'s branch test inside the
    // loop reads the FIXED `y - 1` cell (`:54924`), not `y - 1 - num2` —
    // the quirk kept verbatim.
    let mut num = 100;
    let mut num2 = 1;
    while num > 0 {
        num -= 1;
        let py = y - 1 - num2;
        if py < 5 || py >= world.height - 5 || x < 5 || x >= world.width - 5 {
            break;
        }
        let Some(tile2) = world.tile(x, py) else {
            break;
        };
        if !tile2.active()
            || (tile2.kind != 323 && !is_tree_trunk(tile2.kind))
            || is_tile_a_tree_branch(world, x, y - 1).is_some()
        {
            break;
        }
        num2 += 1;
    }
    // `:54930-54938` — a branch/root marker at the base dies instead.
    if is_tile_a_tree_branch(world, x, y).is_some() || is_tile_a_tree_root(world, x, y).is_some() {
        let _ = crate::edit::kill_tile(world, x, y, false);
        return;
    }
    // `:54939-54957` — converting INTO a palm: the side branch/root markers
    // pointing at this column die along the whole height.
    if new_tree_type == 323 {
        for i in 0..=num2 {
            if world.tile(x, y - i).is_some_and(|t| t.active()) {
                if is_tile_a_tree_branch(world, x - 1, y - i) == Some(1)
                    || is_tile_a_tree_root(world, x - 1, y - i) == Some(1)
                {
                    let _ = crate::edit::kill_tile(world, x - 1, y - i, false);
                }
                if is_tile_a_tree_branch(world, x + 1, y - i) == Some(-1)
                    || is_tile_a_tree_root(world, x + 1, y - i) == Some(-1)
                {
                    let _ = crate::edit::kill_tile(world, x + 1, y - i, false);
                }
            }
        }
    }
    // `:54958-55049` — the column re-type: the base (j == 0), the top
    // (j == num2) and the middle cells each carry their own frame rolls.
    for j in 0..=num2 {
        if !world.tile(x, y - j).is_some_and(|t| t.active()) {
            continue;
        }
        if let Some(t) = world.tile_mut(x, y - j) {
            t.kind = new_tree_type;
        }
        if j == 0 {
            // `:54968-54991` — the base cell.
            match new_tree_type {
                5 | 596 | 616 => {
                    if old_tree_type == 323 {
                        if let Some(t) = world.tile_mut(x, y - j) {
                            t.frame_x = 0;
                            t.frame_y = (22 * rng.next_range(0, 3)) as i16;
                        }
                    } else {
                        convert_tree_and_ground_branches(world, x, y - j, new_tree_type);
                    }
                }
                323 => {
                    if let Some(t) = world.tile_mut(x, y - j) {
                        t.frame_x = 66;
                        t.frame_y = 0;
                    }
                }
                _ => {}
            }
            continue;
        }
        if j == num2 {
            // `:54992-55015` — the top cell.
            match new_tree_type {
                5 | 596 | 616 => {
                    if old_tree_type == 323 {
                        if let Some(t) = world.tile_mut(x, y - j) {
                            t.frame_x = 22;
                            t.frame_y = (22 * rng.next_range(9, 12)) as i16;
                        }
                    } else {
                        convert_tree_and_ground_branches(world, x, y - j, new_tree_type);
                    }
                }
                323 => {
                    if let Some(t) = world.tile_mut(x, y - j) {
                        t.frame_x = (22 * rng.next_range(4, 7)) as i16;
                        t.frame_y = 0;
                    }
                }
                _ => {}
            }
            continue;
        }
        // `:55016-55048` — the middle cells: the palm-origin branch lottery
        // (Next(2) gate, then the second Next(2), then the frame row).
        match new_tree_type {
            5 | 596 | 616 => {
                if old_tree_type == 323 {
                    if rng.next_max(2) == 0 {
                        let row = rng.next_range(3, 6);
                        if let Some(t) = world.tile_mut(x, y - j) {
                            t.frame_x = 44;
                            t.frame_y = (22 * row) as i16;
                        }
                    } else if rng.next_max(2) == 0 {
                        let row = rng.next_range(0, 6);
                        if let Some(t) = world.tile_mut(x, y - j) {
                            t.frame_x = 22;
                            t.frame_y = (22 * row) as i16;
                        }
                    } else {
                        let row = rng.next_range(0, 6);
                        if let Some(t) = world.tile_mut(x, y - j) {
                            t.frame_x = 0;
                            t.frame_y = (22 * row) as i16;
                        }
                    }
                } else {
                    convert_tree_and_ground_branches(world, x, y - j, new_tree_type);
                }
            }
            323 => {
                let col = rng.next_range(0, 3);
                if let Some(t) = world.tile_mut(x, y - j) {
                    t.frame_x = (22 * col) as i16;
                    t.frame_y = 0;
                }
            }
            _ => {}
        }
    }
    // `:55050` — the ground cell takes the new floor type.
    if let Some(t) = world.tile_mut(x, y + 1) {
        t.kind = new_floor_type;
    }
}

/// `WorldGen.IsTileATreeBranch(x, y, out offsetToTrunk)`
/// (`WorldGen.cs:29717-29740`, W4-f25 helper C6): the cell is a trunk-kind
/// tile (NOT the palm 323) whose marker frame is a BRANCH — frameX 44 at
/// frameY 198/220/242 or frameX 66 at frameY 0/22/44 reads offset +1 (the
/// trunk sits one column right); frameX 66 at frameY 198/220/242 or frameX
/// 88 at frameY 66/88/110 reads offset -1. `None` = not a branch (the C#'s
/// false + offset 0). WIRED through C7 `convert_tree_and_ground` (the
/// trunk-height walk + the side-marker arms); ConsideredSolidTileForAnchor
/// `:53903` (unported) is the other C# caller.
pub fn is_tile_a_tree_branch(world: &World, x: i32, y: i32) -> Option<i32> {
    // `:29720-29723` — InWorld(2).
    if x < 2 || y < 2 || x >= world.width - 2 || y >= world.height - 2 {
        return None;
    }
    // `:29724-29728` — null / inactive / palm / non-trunk refuse.
    let t = world.tile(x, y)?;
    if !t.active() || t.kind == 323 || !is_tree_trunk(t.kind) {
        return None;
    }
    // `:29729-29733` — the +1 markers.
    if (t.frame_x == 44 && matches!(t.frame_y, 198 | 220 | 242))
        || (t.frame_x == 66 && matches!(t.frame_y, 0 | 22 | 44))
    {
        return Some(1);
    }
    // `:29734-29738` — the -1 markers.
    if (t.frame_x == 66 && matches!(t.frame_y, 198 | 220 | 242))
        || (t.frame_x == 88 && matches!(t.frame_y, 66 | 88 | 110))
    {
        return Some(-1);
    }
    None
}

/// `WorldGen.IsTileATreeRoot(x, y, out offsetToTrunk)`
/// (`WorldGen.cs:29742-29765`, W4-f25 helper C6): the same trunk-kind gate,
/// the ROOT markers — frameX 44 at frameY 132/154/176 reads offset +1,
/// frameX 22 at the same rows reads offset -1. UNWIRED (C7's consumer).
pub fn is_tile_a_tree_root(world: &World, x: i32, y: i32) -> Option<i32> {
    // `:29745-29748` — InWorld(2).
    if x < 2 || y < 2 || x >= world.width - 2 || y >= world.height - 2 {
        return None;
    }
    // `:29749-29753`.
    let t = world.tile(x, y)?;
    if !t.active() || t.kind == 323 || !is_tree_trunk(t.kind) {
        return None;
    }
    // `:29754-29758` — the +1 root markers.
    if t.frame_x == 44 && matches!(t.frame_y, 132 | 154 | 176) {
        return Some(1);
    }
    // `:29759-29763` — the -1 root markers.
    if t.frame_x == 22 && matches!(t.frame_y, 132 | 154 | 176) {
        return Some(-1);
    }
    None
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

    // W4-f25 helper C6 — IsTileATreeBranch / IsTileATreeRoot
    // (`WorldGen.cs:29717-29765`). WRITTEN NOT RUN (code-only round).

    #[test]
    fn w4_f25_c6_the_branch_markers_and_offsets() {
        let mut w = World::new("t".into(), 64, 64);
        // The +1 markers (`:29729-29733`).
        for (fx, fy) in [(44i16, 198i16), (44, 220), (44, 242), (66, 0), (66, 22), (66, 44)] {
            put(&mut w, 20, 10, 5);
            let i = w.index(20, 10);
            w.tiles[i].frame_x = fx;
            w.tiles[i].frame_y = fy;
            assert_eq!(is_tile_a_tree_branch(&w, 20, 10), Some(1), "{fx},{fy}");
        }
        // The -1 markers (`:29734-29738`).
        for (fx, fy) in [(66i16, 198i16), (66, 220), (66, 242), (88, 66), (88, 88), (88, 110)] {
            let i = w.index(20, 10);
            w.tiles[i].frame_x = fx;
            w.tiles[i].frame_y = fy;
            assert_eq!(is_tile_a_tree_branch(&w, 20, 10), Some(-1), "{fx},{fy}");
        }
        // A plain trunk frame is not a branch; the palm 323 never is.
        let i = w.index(20, 10);
        w.tiles[i].frame_x = 0;
        w.tiles[i].frame_y = 0;
        assert_eq!(is_tile_a_tree_branch(&w, 20, 10), None);
        put(&mut w, 21, 10, 323);
        let i = w.index(21, 10);
        w.tiles[i].frame_x = 66;
        w.tiles[i].frame_y = 0;
        assert_eq!(is_tile_a_tree_branch(&w, 21, 10), None, "the palm refuses");
        // The variant trunk kinds count (IsATreeTrunk).
        put(&mut w, 22, 10, 596);
        let i = w.index(22, 10);
        w.tiles[i].frame_x = 44;
        w.tiles[i].frame_y = 198;
        assert_eq!(is_tile_a_tree_branch(&w, 22, 10), Some(1));
        // The InWorld(2) border band refuses.
        put(&mut w, 1, 10, 5);
        let i = w.index(1, 10);
        w.tiles[i].frame_x = 44;
        w.tiles[i].frame_y = 198;
        assert_eq!(is_tile_a_tree_branch(&w, 1, 10), None);
    }

    #[test]
    fn w4_f25_c6_the_root_markers_and_offsets() {
        let mut w = World::new("t".into(), 64, 64);
        for fy in [132i16, 154, 176] {
            put(&mut w, 20, 10, 5);
            let i = w.index(20, 10);
            w.tiles[i].frame_x = 44;
            w.tiles[i].frame_y = fy;
            assert_eq!(is_tile_a_tree_root(&w, 20, 10), Some(1), "44,{fy}");
            w.tiles[i].frame_x = 22;
            assert_eq!(is_tile_a_tree_root(&w, 20, 10), Some(-1), "22,{fy}");
            // A branch marker is not a root.
            w.tiles[i].frame_x = 44;
            w.tiles[i].frame_y = 198;
            assert_eq!(is_tile_a_tree_root(&w, 20, 10), None);
        }
        // Inactive refuses.
        put(&mut w, 21, 10, 5);
        let i = w.index(21, 10);
        w.tiles[i].flags &= !FLAG_ACTIVE;
        w.tiles[i].frame_x = 44;
        w.tiles[i].frame_y = 132;
        assert_eq!(is_tile_a_tree_root(&w, 21, 10), None);
    }

    // W4-f25 helper C7 — ConvertTreeAndGround (`WorldGen.cs:54902-55065`).
    // WRITTEN NOT RUN (code-only round).

    #[test]
    fn w4_f25_c7_the_forest_to_palm_conversion() {
        let mut w = World::new("t".into(), 64, 64);
        // A 3-tall trunk column y=18..20 over grass ground y=21.
        for y in 18..=20 {
            put(&mut w, 30, y, 5);
        }
        put(&mut w, 30, 21, 2);
        let mut rng = crate::UnifiedRandom::new(7);
        convert_tree_and_ground(&mut w, &mut rng, 30, 20, 53, 5, 323);
        // The walk: y-2=18 is a trunk (num2 → 2), y-3=17 inactive breaks.
        // The base takes frameX 66 / frameY 0 (`:54985-54988`).
        let base = w.tile(30, 20).unwrap();
        assert_eq!(base.kind, 323);
        assert_eq!((base.frame_x, base.frame_y), (66, 0));
        // The middle cell's column roll Next(0,3) (`:55045`).
        let mid = w.tile(30, 19).unwrap();
        assert_eq!(mid.kind, 323);
        assert!(matches!(mid.frame_x, 0 | 22 | 44) && mid.frame_y == 0);
        // The top's Next(4,7) (`:55010`).
        let top = w.tile(30, 18).unwrap();
        assert_eq!(top.kind, 323);
        assert!(matches!(top.frame_x, 88 | 110 | 132) && top.frame_y == 0);
        // The ground cell takes the new floor (`:55050`).
        assert_eq!(w.tile(30, 21).unwrap().kind, 53);
    }

    #[test]
    fn w4_f25_c7_the_palm_to_forest_frame_lottery() {
        let mut w = World::new("t".into(), 64, 64);
        for y in 17..=20 {
            put(&mut w, 30, y, 323);
        }
        put(&mut w, 30, 21, 53);
        let mut rng = crate::UnifiedRandom::new(11);
        convert_tree_and_ground(&mut w, &mut rng, 30, 20, 2, 323, 5);
        // The base: frameX 0, frameY 22*Next(0,3) (`:54977-54978`).
        let base = w.tile(30, 20).unwrap();
        assert_eq!(base.kind, 5);
        assert_eq!(base.frame_x, 0);
        assert!(matches!(base.frame_y, 0 | 22 | 44));
        // The top: frameX 22, frameY 22*Next(9,12) (`:55001-55002`).
        let top = w.tile(30, 17).unwrap();
        assert_eq!(top.kind, 5);
        assert_eq!(top.frame_x, 22);
        assert!(matches!(top.frame_y, 198 | 220 | 242));
        // The middle cells: the branch lottery rows (`:55023-55037`).
        for y in [18, 19] {
            let t = w.tile(30, y).unwrap();
            assert_eq!(t.kind, 5);
            assert!(matches!(t.frame_x, 0 | 22 | 44));
            assert!(t.frame_y % 22 == 0 && (0..6).contains(&(t.frame_y / 22)));
        }
        assert_eq!(w.tile(30, 21).unwrap().kind, 2);
    }

    #[test]
    fn w4_f25_c7_the_vanity_branch_retype_and_the_base_marker_kill() {
        let mut w = World::new("t".into(), 64, 64);
        for y in 18..=20 {
            put(&mut w, 30, y, 5);
        }
        put(&mut w, 30, 21, 2);
        // Side markers pointing at the trunk column: left +1 (44@198),
        // right -1 (88@66) at the middle row.
        put(&mut w, 29, 19, 5);
        let i = w.index(29, 19);
        w.tiles[i].frame_x = 44;
        w.tiles[i].frame_y = 198;
        put(&mut w, 31, 19, 5);
        let i = w.index(31, 19);
        w.tiles[i].frame_x = 88;
        w.tiles[i].frame_y = 66;
        let mut rng = crate::UnifiedRandom::new(3);
        // old != 323: the _Branches path, RNG-free.
        convert_tree_and_ground(&mut w, &mut rng, 30, 20, 109, 5, 596);
        assert_eq!(w.tile(29, 19).unwrap().kind, 596, "the left marker retypes");
        assert_eq!(w.tile(31, 19).unwrap().kind, 596, "the right marker retypes");
        assert_eq!(w.tile(30, 20).unwrap().kind, 596);
        assert_eq!(w.tile(30, 21).unwrap().kind, 109);
        // A base cell that IS a branch marker dies instead (`:54930-54938`).
        let mut w2 = World::new("t".into(), 64, 64);
        put(&mut w2, 30, 20, 5);
        let i = w2.index(30, 20);
        w2.tiles[i].frame_x = 44;
        w2.tiles[i].frame_y = 198;
        put(&mut w2, 30, 21, 2);
        convert_tree_and_ground(&mut w2, &mut rng, 30, 20, 53, 5, 323);
        assert!(
            !w2.tile(30, 20).unwrap().active(),
            "the marker base is killed"
        );
        assert_eq!(w2.tile(30, 21).unwrap().kind, 2, "no ground write on the kill arm");
    }
}
