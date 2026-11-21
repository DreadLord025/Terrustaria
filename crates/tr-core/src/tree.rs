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

/// `WorldGen.GetHollowTreeFoliageStyle` (`WorldGen.cs:63027-63040`): the
/// hallowBG switch — 4 → 19, 2/3 → 20, default 3. The port's worlds carry
/// the `hallow_bg_style` byte (the `.wld` setBG(4) slot).
pub fn hollow_tree_foliage_style(world: &World) -> i32 {
    match world.hallow_bg_style {
        4 => 19,
        2 | 3 => 20,
        _ => 3,
    }
}

/// `WorldGen.GetTreeLeaf` (`WorldGen.cs:29384-29598`): the leaf-gore style
/// of a tree's top marker over its ground tile. Returns
/// `(treeFrame, passStyle)`; `passStyle == -1` = no leaf family. The
/// hallowed arm bumps `treeHeight` by 5 (`:29593`) — the caller's `ref`
/// write, returned as the third element. The `render::foliage`
/// `tree_leaf_pass_style` twin reads the same table for the ambient
/// leaves (its hallow arm is the style-20-unreachable simplification;
/// this port carries both branches).
pub fn tree_leaf(
    world: &World,
    x: i32,
    top_kind: u16,
    top_frame_x: i16,
    top_frame_y: i16,
    ground_kind: u16,
) -> (i32, i32, i32) {
    let mut tree_frame = 0;
    let mut pass_style = -1;
    // `:29388-29407` — the branch/top frame pick (both arms identical).
    if matches!(top_frame_x, 22 | 44 | 66) {
        if top_frame_y == 220 {
            tree_frame = 1;
        } else if top_frame_y == 242 {
            tree_frame = 2;
        }
    }
    // `:29408-29434` — the vanity/gem/ash top-kind overrides.
    match top_kind {
        596 => pass_style = 1248,
        616 => pass_style = 1257,
        634 => pass_style = 1278,
        583..=589 => pass_style = 1249 + (top_kind as i32 - 583),
        _ => {}
    }
    if pass_style > -1 {
        return (tree_frame, pass_style, 0);
    }
    // `:29435-29595` — the ground-kind table.
    let mut height_add = 0;
    match ground_kind {
        2 | 477 => pass_style = 910,
        60 => pass_style = 914,
        70 => pass_style = 912,
        23 | 112 => pass_style = 915,
        199 | 234 => pass_style = 916,
        53 => pass_style = 911,
        116 => pass_style = 919,
        147 => pass_style = 913,
        633 => pass_style = 1278,
        109 | 492 => {
            // `:29467-29592` — the hallowed tables.
            let num = 917;
            if hollow_tree_foliage_style(world) != 20 {
                if x % 3 == 1 {
                    tree_frame += 3;
                }
                if x % 3 == 2 {
                    tree_frame += 6;
                }
                let styles = [2, 1, 7, 4, 5, 6, 3, 8, 0];
                pass_style = styles.get(tree_frame as usize).copied().unwrap_or(0);
            } else {
                let num2 = 196;
                match x % 6 {
                    1 => tree_frame += 3,
                    2 => tree_frame += 6,
                    3 => tree_frame += 9,
                    4 => tree_frame += 12,
                    5 => tree_frame += 15,
                    _ => {}
                }
                let styles = [0, 0, 0, 1, 2, 1, 3, 4, 5, 6, 6, 6, 7, 7, 7, 8, 8, 8];
                pass_style = num2 + styles.get(tree_frame as usize).copied().unwrap_or(0);
            }
            pass_style += num;
            height_add = 5;
        }
        _ => {}
    }
    (tree_frame, pass_style, height_add)
}

/// `WorldGen.TreeGrowFXCheck` (`WorldGen.cs:29345-29382`): the height/style
/// scan over a grown tree's trunk column. Returns
/// `Some((height, pass_style))` when the FX should fire (`treeHeight > 0 &&
/// passStyle > 0`), `None` otherwise. The netMode-2 SendData arm drops
/// (single-player port); the caller runs `TreeGrowFX` with the pair.
/// RNG-free — every gate reads tiles only.
pub fn tree_grow_fx_check(world: &World, x: i32, y: i32) -> Option<(i32, i32)> {
    let mut tree_height = 1;
    let mut top: Option<(u16, i16, i16)> = None;
    // `:29349-29358` — the upward walk over the GetsCheckedForLeaves cells.
    for num in 1..100 {
        let Some(t) = world.tile(x, y - num) else { break };
        if !t.active() || !crate::tiledata::SET_GETS_CHECKED_FOR_LEAVES.has(t.kind) {
            break;
        }
        top = Some((t.kind, t.frame_x, t.frame_y));
        tree_height += 1;
    }
    // `:29359-29370` — the downward walk (max 4 cells), then GetTreeLeaf.
    let mut pass_style = -1;
    for i in 1..5 {
        let Some(t) = world.tile(x, y + i) else { break };
        if t.active() && crate::tiledata::SET_GETS_CHECKED_FOR_LEAVES.has(t.kind) {
            tree_height += 1;
            continue;
        }
        if let Some((top_kind, fx, fy)) = top {
            let (_, style, height_add) = tree_leaf(world, x, top_kind, fx, fy, t.kind);
            tree_height += height_add;
            pass_style = style;
        }
        break;
    }
    // `:29371-29381`.
    if tree_height > 0 && pass_style > 0 {
        Some((tree_height, pass_style))
    } else {
        None
    }
}

/// `WorldGen.PlayerLOS` (`WorldGen.cs:69443-69454`): the 16×16 tile rect
/// intersects the player's 1.2×sWidth/sHeight screen box (the
/// `NPC.sWidth/sHeight` 1920/1200 constants). `players` is the roster
/// snapshot `(active, x, y, width, height)` per slot — the single-player
/// port passes one. RNG-free.
pub fn player_los(x: i32, y: i32, players: &[(bool, f32, f32, f32, f32)]) -> bool {
    // `NPC.sWidth`/`sHeight` (`NPC.cs:6767-6769`).
    let (s_width, s_height) = (1920.0f64, 1200.0f64);
    let rx = x * 16;
    let ry = y * 16;
    for &(active, px, py, w, h) in players {
        if !active {
            continue;
        }
        // The C#'s double-then-int truncation (`:69449`).
        let bx = (px as f64 + w as f64 * 0.5 - s_width * 0.6) as i32;
        let by = (py as f64 + h as f64 * 0.5 - s_height * 0.6) as i32;
        let bw = (s_width * 1.2) as i32;
        let bh = (s_height * 1.2) as i32;
        // XNA `Rectangle.Intersects`: strict overlap on both axes.
        if rx < bx + bw && rx + 16 > bx && ry < by + bh && ry + 16 > by {
            return true;
        }
    }
    false
}

/// `WorldGen.IsTileALeafyTreeTop` (`WorldGen.cs:29674-29691`): the palm
/// canopy (323, frameX >= 88) or a top marker (frameX 22, frameY 198-242)
/// of a GetsCheckedForLeaves kind.
pub fn is_tile_a_leafy_tree_top(world: &World, x: i32, y: i32) -> bool {
    let Some(t) = world.tile(x, y) else { return false };
    if !t.active() || !crate::tiledata::SET_GETS_CHECKED_FOR_LEAVES.has(t.kind) {
        return false;
    }
    if t.kind == 323 && t.frame_x >= 88 {
        return true;
    }
    t.frame_x == 22 && (198..=242).contains(&t.frame_y)
}

/// `Collision.SolidTiles(startX, endX, startY, endY)` (`Collision.cs:
/// 3455-3485`): the bounds refusals (start < 0, endX >= width, endY >=
/// height - 40 → true), then the inclusive scan for an active,
/// non-actuated, solid, non-solid-top cell. A null tile answers false
/// (the scan continues — the C#'s own shape).
pub fn solid_tiles(world: &World, start_x: i32, end_x: i32, start_y: i32, end_y: i32) -> bool {
    if start_x < 0 || end_x >= world.width || start_y < 0 || end_y >= world.height - 40 {
        return true;
    }
    for i in start_x..=end_x {
        for j in start_y..=end_y {
            let Some(t) = world.tile(i, j) else { continue };
            if t.active()
                && !t.inactive()
                && crate::tiledata::TILE_SOLID.has(t.kind)
                && !crate::tiledata::TILE_SOLID_TOP.has(t.kind)
            {
                return true;
            }
        }
    }
    false
}

/// `WorldGen.ShakeTree` (`WorldGen.cs:63131-63418`)'s FX slice: the
/// shake dedupe bookkeeping (`numTreeShakes`/`treeShakeX/Y`, the 500 cap,
/// the dawn reset), the `GetTreeTypeAndTreeBottom` family probe, the
/// `IsShakeable` trunk walk up to the leafy top, the
/// `IsTileALeafyTreeTop` + `SolidTiles` refusal, then the
/// `GetTreeLeaf` → `TreeGrowFX(bottomX, bottomY, 1, passStyle,
/// hitTree: true)` tail (`:63404-63417`). Returns the FX request
/// `Some((bottom_x, bottom_y, pass_style))` — the app runs the gore
/// burst on its stream. The loot ladder (`:63160-63403`, the genRand
/// item/NPC/projectile arms) is the loot lane's — documented drop; the
/// FX tail's position is independent of it (no RNG before it).
pub fn shake_tree_fx(world: &mut World, i: i32, j: i32) -> Option<(i32, i32, i32)> {
    // `:63133-63136` — the 500-shake cap (`maxTreeShakes`, `:86960`).
    if world.tree_shakes.len() >= 500 {
        return None;
    }
    // `:63137-63142` — the family + the ground cell. `num` keeps the
    // ORIGINAL ground row (`:63138`) — the FX tail's GetTreeLeaf reads
    // it as the ground tile (`:63406`).
    let (tree_type, bottom_x, mut bottom_y) = tree_type_and_bottom(world, i, j);
    let num = bottom_y;
    if tree_type == TreeType::None {
        return None;
    }
    // `:63143-63149` — the per-day dedupe.
    if world.tree_shakes.contains(&(bottom_x, bottom_y)) {
        return None;
    }
    // `:63150-63152` — the record.
    world.tree_shakes.push((bottom_x, bottom_y));
    // `:63153-63158` — the trunk walk up to the leafy top.
    bottom_y -= 1;
    while bottom_y > 10
        && world
            .tile(bottom_x, bottom_y)
            .is_some_and(|t| t.active() && crate::tiledata::SET_IS_SHAKEABLE.has(t.kind))
    {
        bottom_y -= 1;
    }
    bottom_y += 1;
    // `:63159-63162` — the leafy-top + the 5x5 solid refusal.
    if !is_tile_a_leafy_tree_top(world, bottom_x, bottom_y)
        || solid_tiles(world, bottom_x - 2, bottom_x + 2, bottom_y - 2, bottom_y + 2)
    {
        return None;
    }
    // `:63404-63417` — the GetTreeLeaf probe: the TOP tile is the walked
    // leafy cell (`bottomY`), the GROUND tile is `Main.tile[bottomX, num]`
    // — the ORIGINAL ground row. The loot ladder between
    // (`:63163-63403`) is the loot lane's — it draws genRand/Main.rand
    // but never moves the FX position.
    let Some(top) = world.tile(bottom_x, bottom_y).copied() else {
        return None;
    };
    let Some(ground) = world.tile(bottom_x, num).copied() else {
        return None;
    };
    let mut tree_height = 0;
    let (_, pass_style, height_add) = tree_leaf(
        world,
        bottom_x,
        top.kind,
        top.frame_x,
        top.frame_y,
        ground.kind,
    );
    tree_height += height_add;
    let _ = tree_height;
    // `:63407` — `passStyle != -1` fires the FX (the C# height arg is 1).
    if pass_style != -1 {
        Some((bottom_x, bottom_y, pass_style))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---- W4-g17 wiring: the TreeGrowFXCheck / ShakeTree FX ports ----

    use crate::tile::Tile;

    /// A height-`h` forest tree at x=10 on dirt ground (row 20), the
    /// canopy marker (22, 198) on top — the `edit::kill` tree_world shape.
    fn fx_tree(h: i32) -> World {
        let mut w = World::new("fx".into(), 64, 64);
        for x in 8..14 {
            let i = w.index(x, 20);
            w.tiles[i] = Tile { kind: 2, flags: crate::tile::FLAG_ACTIVE, ..Default::default() };
        }
        for dy in 0..h {
            let i = w.index(10, 19 - dy);
            let (fx, fy) = if dy == h - 1 { (22, 198) } else { (0, 0) };
            w.tiles[i] = Tile {
                kind: 5,
                frame_x: fx,
                frame_y: fy,
                flags: crate::tile::FLAG_ACTIVE,
                ..Default::default()
            };
        }
        w
    }

    /// `WorldGen.cs:29345-29382` — the check scans the trunk column up
    /// from the base, counts the GetsCheckedForLeaves cells, and the
    /// ground tile under the base names the leaf family (dirt → 910).
    #[test]
    fn w4_g17_the_tree_grow_fx_check_resolves_height_and_style() {
        let w = fx_tree(5);
        // The check cell is the sapling's old base row (the growth call
        // passes the sapling cell; the trunk grew upward from it).
        let got = tree_grow_fx_check(&w, 10, 19);
        assert_eq!(got, Some((5, 910)), "height 5, the forest leaf 910");
        // No tree → no FX.
        let empty = World::new("e".into(), 64, 64);
        assert_eq!(tree_grow_fx_check(&empty, 10, 19), None);
    }

    /// `WorldGen.cs:29408-29434` — the vanity/gem/ash top-kind overrides
    /// outrank the ground table; the hallowed ground adds 5 to the height.
    #[test]
    fn w4_g17_the_tree_leaf_overrides_and_hallow_height() {
        let w = World::new("t".into(), 64, 64);
        // The ash top 634 → 1278 regardless of ground.
        assert_eq!(tree_leaf(&w, 3, 634, 22, 198, 2).1, 1278);
        // The gem top 583 → 1249.
        assert_eq!(tree_leaf(&w, 3, 583, 22, 198, 1).1, 1249);
        // The hallowed ground 109: style 917+ and height +5.
        let (frame, style, add) = tree_leaf(&w, 3, 5, 22, 198, 109);
        assert_eq!(add, 5);
        assert!((917..=925).contains(&style), "style {style}");
        assert_eq!(frame, 0);
        // x % 3 == 1 bumps the frame by 3 (the style table's row).
        let (frame1, style1, _) = tree_leaf(&w, 4, 5, 22, 198, 109);
        assert_eq!(frame1, 3);
        assert_ne!(style1, style);
    }

    /// `WorldGen.cs:63131-63162` + `:63404-63417` — the shake records the
    /// ground cell, dedupes per day, and the FX tail resolves the leafy
    /// top's style (hitTree: true, height 1).
    #[test]
    fn w4_g17_the_shake_fx_records_and_dedupes() {
        let mut w = fx_tree(5);
        let first = shake_tree_fx(&mut w, 10, 17);
        assert_eq!(first, Some((10, 20, 910)), "the ground cell + the forest leaf");
        assert_eq!(w.tree_shakes, vec![(10, 20)]);
        // The same tree shakes once per day.
        assert_eq!(shake_tree_fx(&mut w, 10, 15), None);
        // The dawn reset clears the book.
        w.tree_shakes.clear();
        assert!(shake_tree_fx(&mut w, 10, 17).is_some());
    }

    /// `WorldGen.cs:69443-69454` — the LOS box is the 1.2×screen rect
    /// centred on the player; a far cell refuses.
    #[test]
    fn w4_g17_the_player_los_box() {
        let players = [(true, 1000.0f32, 1000.0f32, 20.0f32, 42.0f32)];
        // The player's own cell is on screen.
        assert!(player_los(62, 62, &players));
        // 200 tiles away is off screen.
        assert!(!player_los(262, 62, &players));
        // No players → no LOS.
        assert!(!player_los(62, 62, &[]));
    }

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
