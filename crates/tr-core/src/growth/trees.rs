//! The tree-growth family of the world-growth ticks (S1-K-3 split out of
//! `growth.rs`; behaviour frozen - verbatim move). `WorldGen.GrowTree`
//! (`:29767-29980`), `AttemptToGrowTreeFromSapling` (`:72810-72925`), the
//! `GrowTreeSettings` profiles (`:3825-4048`) + `TryGrowingTreeByType`
//! (`:30343-30387`) / `GrowTreeWithSettings` (`:30388-30847`),
//! `GrowPalmTree` (`:27116-27202`) and the tree-invalidity kill
//! (`:54802-54860`). The public seams stay re-exported from `growth`.

use crate::tile::FLAG_ACTIVE;
use crate::UnifiedRandom;
use crate::world::World;

/// `UpdateWorld_OvergroundTile`'s port slice (`:72030+`): the herb branch.
/// The other arms (the trees, the cacti, the beach shells, the bamboo) land
/// with M7-GROWTH-2.
/// `WorldGen.IsTileTypeFitForTree` (`:29695-29710`).
pub const TREE_SOILS: [u16; 12] = [2, 23, 60, 70, 109, 147, 199, 477, 492, 633, 661, 662];

pub fn is_tile_type_fit_for_tree(kind: u16) -> bool {
    TREE_SOILS.contains(&kind)
}

/// `TileID.Sets.CommonSapling` (`TileID.cs:493`): the sapling tiles — the
/// ash-tree sapling 20, the gem saplings 590, the sakura 595 and the
/// willow 615. `EmptyTileCheck`'s `ignoreID` arm and the smart-cursor
/// seed steps' proximity gates read it.
pub const COMMON_SAPLINGS: [u16; 4] = [20, 590, 595, 615];

/// Membership in [`COMMON_SAPLINGS`] (`TileID.Sets.CommonSapling`,
/// `TileID.cs:493`).
pub fn is_common_sapling(kind: u16) -> bool {
    COMMON_SAPLINGS.contains(&kind)
}

/// `WorldGen.GemTreeGroundTest` (`WorldGen.cs:30304-30315`): the stone or
/// moss conversion tiles — the gem trees' ground. The C#'s arrays are
/// length `TileID.Count`, so a `u16` id is always in range and its
/// `tileType < 0` guard is vacuous in the port.
fn gem_tree_ground_test(kind: u16) -> bool {
    crate::tiledata::SET_CONVERSION_STONE.has(kind) || crate::tiledata::SET_CONVERSION_MOSS.has(kind)
}

/// `WorldGen.VanityTreeGroundTest` (`WorldGen.cs:30317-30328`): the grass
/// conversion tiles minus the corruption/crimson pair (23/199).
fn vanity_tree_ground_test(kind: u16) -> bool {
    crate::tiledata::SET_GRASS.has(kind) && kind != 23 && kind != 199
}

/// `WorldGen.AshTreeGroundTest` (`WorldGen.cs:30330-30341`): ash grass only.
fn ash_tree_ground_test(kind: u16) -> bool {
    kind == 633
}

/// `WorldGen.GrowTreeSettings` (`WorldGen.cs:3825-4048`'s struct): one
/// gem/vanity/ash tree profile. The C#'s `GroundTest`/`WallTest` delegates
/// are the port's [`GroundTestKind`]; the wall arm stays out — no
/// smart-cursor step reads it (the C#'s other callers do).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct GrowTreeSettings {
    /// `GroundTest` — the delegate the C# stores per profile.
    pub ground: GroundTestKind,
    /// `TreeTileType` — the tree-top tile the profile grows.
    pub tree_tile_type: u16,
    /// `TreeHeightMin`.
    pub tree_height_min: i32,
    /// `TreeHeightMax`.
    pub tree_height_max: i32,
    /// `TreeTopPaddingNeeded`.
    pub tree_top_padding_needed: i32,
    /// `SaplingTileType` — the profile's sapling (`EmptyTileCheck`'s
    /// ignore id and the proximity gate's set).
    pub sapling_tile_type: u16,
}

/// The C#'s `GrowTreeSettings.Profiles`' `GroundTest` delegates
/// (`WorldGen.cs:30304-30341`).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum GroundTestKind {
    /// `WorldGen.GemTreeGroundTest`.
    Gem,
    /// `WorldGen.VanityTreeGroundTest`.
    Vanity,
    /// `WorldGen.AshTreeGroundTest`.
    Ash,
}

impl GroundTestKind {
    /// The delegate's call (the C#'s `profile.GroundTest(tileType)`).
    pub fn test(&self, kind: u16) -> bool {
        match self {
            GroundTestKind::Gem => gem_tree_ground_test(kind),
            GroundTestKind::Vanity => vanity_tree_ground_test(kind),
            GroundTestKind::Ash => ash_tree_ground_test(kind),
        }
    }
}

impl GrowTreeSettings {
    /// The C#'s `Profiles` static table (`WorldGen.cs:3930-4022`): every
    /// gem tree shares the heights 12/7, the padding 4 and the sapling 590;
    /// the sakura's sapling is 595, the willow's 615 and the ash's 20.
    pub const GEM_TREE_RUBY: GrowTreeSettings = GrowTreeSettings {
        ground: GroundTestKind::Gem,
        tree_tile_type: 587,
        tree_height_min: 7,
        tree_height_max: 12,
        tree_top_padding_needed: 4,
        sapling_tile_type: 590,
    };
    /// `Profiles.GemTree_Diamond` (`WorldGen.cs:3942`).
    pub const GEM_TREE_DIAMOND: GrowTreeSettings =
        GrowTreeSettings { tree_tile_type: 588, ..GrowTreeSettings::GEM_TREE_RUBY };
    /// `Profiles.GemTree_Topaz` (`WorldGen.cs:3952`).
    pub const GEM_TREE_TOPAZ: GrowTreeSettings =
        GrowTreeSettings { tree_tile_type: 583, ..GrowTreeSettings::GEM_TREE_RUBY };
    /// `Profiles.GemTree_Amethyst` (`WorldGen.cs:3962`).
    pub const GEM_TREE_AMETHYST: GrowTreeSettings =
        GrowTreeSettings { tree_tile_type: 584, ..GrowTreeSettings::GEM_TREE_RUBY };
    /// `Profiles.GemTree_Sapphire` (`WorldGen.cs:3972`).
    pub const GEM_TREE_SAPPHIRE: GrowTreeSettings =
        GrowTreeSettings { tree_tile_type: 585, ..GrowTreeSettings::GEM_TREE_RUBY };
    /// `Profiles.GemTree_Emerald` (`WorldGen.cs:3982`).
    pub const GEM_TREE_EMERALD: GrowTreeSettings =
        GrowTreeSettings { tree_tile_type: 586, ..GrowTreeSettings::GEM_TREE_RUBY };
    /// `Profiles.GemTree_Amber` (`WorldGen.cs:3992`).
    pub const GEM_TREE_AMBER: GrowTreeSettings =
        GrowTreeSettings { tree_tile_type: 589, ..GrowTreeSettings::GEM_TREE_RUBY };
    /// `Profiles.VanityTree_Sakura` (`WorldGen.cs:4002`).
    pub const VANITY_TREE_SAKURA: GrowTreeSettings = GrowTreeSettings {
        ground: GroundTestKind::Vanity,
        tree_tile_type: 596,
        tree_height_min: 7,
        tree_height_max: 12,
        tree_top_padding_needed: 4,
        sapling_tile_type: 595,
    };
    /// `Profiles.VanityTree_Willow` (`WorldGen.cs:4012`).
    pub const VANITY_TREE_WILLOW: GrowTreeSettings = GrowTreeSettings {
        ground: GroundTestKind::Vanity,
        tree_tile_type: 616,
        tree_height_min: 7,
        tree_height_max: 12,
        tree_top_padding_needed: 4,
        sapling_tile_type: 615,
    };
    /// `Profiles.Tree_Ash` (`WorldGen.cs:4022`).
    pub const TREE_ASH: GrowTreeSettings = GrowTreeSettings {
        ground: GroundTestKind::Ash,
        tree_tile_type: 634,
        tree_height_min: 7,
        tree_height_max: 12,
        tree_top_padding_needed: 4,
        sapling_tile_type: 20,
    };
}

/// `WorldGen.GrowTreeSettings.Profiles.TryGetFromItemId`
/// (`WorldGen.cs:3853-3884`): the seed items that carry a profile — the
/// seven gem seeds 4851-4857, the willow 4907 and the sakura 4871. (The
/// ash profile's only entry is `TryGetFromTreeId`'s tree tile 634 — the
/// C# has no ash SEED item, so no item id reaches it here.)
pub fn grow_tree_profile_from_item(item: u16) -> Option<GrowTreeSettings> {
    match item {
        4857 => Some(GrowTreeSettings::GEM_TREE_AMBER),
        4852 => Some(GrowTreeSettings::GEM_TREE_AMETHYST),
        4856 => Some(GrowTreeSettings::GEM_TREE_DIAMOND),
        4854 => Some(GrowTreeSettings::GEM_TREE_EMERALD),
        4855 => Some(GrowTreeSettings::GEM_TREE_RUBY),
        4853 => Some(GrowTreeSettings::GEM_TREE_SAPPHIRE),
        4851 => Some(GrowTreeSettings::GEM_TREE_TOPAZ),
        4907 => Some(GrowTreeSettings::VANITY_TREE_WILLOW),
        4871 => Some(GrowTreeSettings::VANITY_TREE_SAKURA),
        _ => None,
    }
}

/// `WorldGen.EmptyTileCheck(startX, endX, startY, endY, ignoreID = 20)`
/// (`:31453-31520`): every cell in the box is inactive — or a
/// `CommonSapling` when the passed `ignoreID` is itself a sapling (the
/// C#'s two-part flag arm, `:31471-31475` + `:31501`).
pub fn empty_tile_check(
    world: &World,
    start_x: i32,
    end_x: i32,
    start_y: i32,
    end_y: i32,
    ignore_id: Option<u16>,
) -> bool {
    if start_x < 0 || end_x >= world.width as i32 || start_y < 0 || end_y >= world.height as i32 {
        return false;
    }
    let flag = ignore_id.is_some_and(is_common_sapling);
    for x in start_x..=end_x {
        for y in start_y..=end_y {
            let t = world.tile(x, y);
            if let Some(t) = t {
                if t.active() && !(flag && is_common_sapling(t.kind)) {
                    return false;
                }
            }
        }
    }
    true
}

/// `WorldGen.AttemptToGrowTreeFromSapling` (:72810-72925)'s port slice: the
/// kind-20 sapling's default arm → `GrowTree` (the gem/ash/vanity saplings
/// 595/615/590+ and the palm-arm styles ride the settings profiles — the
/// port's surface gen spawns the common saplings only).
pub fn attempt_grow_tree_from_sapling(
    world: &mut World,
    x: i32,
    y: i32,
    underground: bool,
    rng: &mut UnifiedRandom,
) -> bool {
    // `InWorld(x, y, 2)` (`:72813-72816`).
    if x < 2 || y < 2 || x >= world.width - 2 || y >= world.height - 2 {
        return false;
    }
    let Some(t) = world.tile(x, y) else {
        return false;
    };
    if !t.active() {
        return false;
    }
    // The `frameX / 54` style the sapling carries (`TileObject`'s sapling
    // styles — the ash/gem/palm families).
    let style = t.frame_x.max(0) / 54;
    match t.kind {
        20 => {
            // The kind-20 switch (`:72825-72849`): the style-10 sapling is
            // the ash tree; the styles 6-9 are the palms (their
            // `GrowPalmTree` port rides the worldgen palms — parked); the
            // default arm is the vanilla `GrowTree`.
            if style == 10 {
                try_growing_tree_by_type(world, 634, x, y, rng);
                true
            } else if (6..=9).contains(&style) {
                // The C#'s underground gate (`:72844-72847`): the
                // underground call only proceeds on the notTheBees/remix
                // seeds; the port's overground call always proceeds.
                if underground
                    && !world.seeds.contains(crate::seeds::SeedFlags::NOT_THE_BEES)
                    && !world.seeds.contains(crate::seeds::SeedFlags::REMIX)
                {
                    return false;
                }
                try_growing_tree_by_type(world, 323, x, y, rng)
            } else {
                if underground {
                    false
                } else {
                    grow_tree(world, x, y, rng)
                }
            }
        }
        // The sakura (`:72850-72861`) and willow (`:72862-72873`) saplings.
        595 => try_growing_tree_by_type(world, 596, x, y, rng),
        615 => try_growing_tree_by_type(world, 616, x, y, rng),
        // The gem saplings (`:72874-72905`): underground only, the profile
        // by the style.
        590 => {
            if !underground {
                return false;
            }
            let tree = match style {
                0 => 583,
                1 => 584,
                2 => 585,
                3 => 586,
                4 => 587,
                5 => 588,
                _ => 589,
            };
            try_growing_tree_by_type(world, tree, x, y, rng)
        }
        _ => false,
    }
}

/// `WorldGen.GrowTree` (`:29767-29980`): walk through the sapling stack,
/// the liquid gate, the ground checks, the clear-box roll, then plant the
/// trunk + the branches + the forks (the frame table the worldgen
/// `plant_trees` shares — the same vanilla tables). Returns the success.
pub fn grow_tree(world: &mut World, i: i32, y: i32, rng: &mut UnifiedRandom) -> bool {
    // The sapling stack walk (:29769).
    let mut j = y;
    while world
        .tile(i, j)
        .is_some_and(|t| t.active() && t.kind == 20)
    {
        j += 1;
    }
    // The liquid gate (:29772-29776).
    for dx in [-1i32, 0, 1] {
        if world
            .tile(i + dx, j - 1)
            .is_some_and(|t| t.liquid > 0)
        {
            return false;
        }
    }
    // The ground tile checks (:29778-29789): nactive, flat, the soil fits,
    // the wall arm (`wall == 0` — the port's surface trees stand on bare
    // ground; the wall-sets arm rides the M5-SPREAD-2 continuation), and a
    // tree-fitting soil neighbour.
    let Some(ground) = world.tile(i, j) else {
        return false;
    };
    if !ground.active() || ground.half_brick() || ground.slope() != 0 {
        return false;
    }
    if !is_tile_type_fit_for_tree(ground.kind) {
        return false;
    }
    if world.tile(i, j - 1).is_some_and(|t| t.wall != 0) {
        return false;
    }
    let neighbour_soil = [-1i32, 1].iter().any(|&dx| {
        world
            .tile(i + dx, j)
            .is_some_and(|t| t.active() && is_tile_type_fit_for_tree(t.kind))
    });
    if !neighbour_soil {
        return false;
    }
    // The height + the clearance (:29796-29816): `Next(5,17)`, the jungle
    // grass adds 5, the clear box `EmptyTileCheck(i±2, j-num3..j-1, 20)`.
    let num2 = 5 + rng.next_max(12);
    let mut num3 = num2 + 4;
    if ground.kind == 60 {
        num3 += 5;
    }
    if !empty_tile_check(world, i - 2, i + 2, j - num3, j - 1, Some(20)) {
        return false;
    }
    // The frame table — the same vanilla shape `plant_trees` ports
    // (surface.rs): the trunk rows num6/num5, the branch markers, the forks.
    let height = num2 as usize;
    let top = j; // the ground row; the trunk runs j-num2..j-1
    let mut rows: Vec<(i16, i16)> = vec![(0, 0); height];
    let mut markers: Vec<(i32, i32, i16, i16)> = Vec::new();
    let mut left_last = false;
    let mut right_last = false;
    for (row_i, row) in rows.iter_mut().enumerate() {
        let mut num6 = if row_i == 0 || row_i + 1 == height {
            0
        } else {
            rng.next_max(10) as i16
        };
        while ((num6 == 5 || num6 == 7) && left_last)
            || ((num6 == 6 || num6 == 7) && right_last)
        {
            num6 = rng.next_max(10) as i16;
        }
        let style = rng.next_max(3) as i16;
        let dy = style * 22;
        *row = match num6 {
            1 => (0, 66 + dy),
            2 => (22, dy),
            3 => (44, 66 + dy),
            4 => (22, 66 + dy),
            5 => (88, dy),
            6 => (66, 66 + dy),
            7 => (110, 66 + dy),
            _ => (0, dy),
        };
        left_last = num6 == 5 || num6 == 7;
        right_last = num6 == 6 || num6 == 7;
        let by = top - 1 - row_i as i32;
        if left_last {
            markers.push((i - 1, by, 44, 198 + rng.next_max(3) as i16 * 22));
        }
        if right_last {
            markers.push((i + 1, by, 66, 198 + rng.next_max(3) as i16 * 22));
        }
    }
    rows[height - 1] = (22, 198 + rng.next_max(3) as i16 * 22);
    // The forks: the base style follows the grass neighbours (:29980+).
    let grass_at = |dx: i32| {
        world
            .tile(i + dx, top)
            .is_some_and(|t| t.active() && matches!(t.kind, 2 | 23 | 60 | 109))
    };
    let left_grass = grass_at(-1);
    let right_grass = grass_at(1);
    let mut base_style = rng.next_max(3);
    if !left_grass && base_style == 0 {
        base_style = 2;
    }
    if !left_grass && base_style == 1 {
        base_style = 3;
    }
    if !right_grass && base_style == 0 {
        base_style = 1;
    }
    if !right_grass && base_style == 2 {
        base_style = 3;
    }
    if left_grass && !right_grass {
        base_style = 2;
    }
    if right_grass && !left_grass {
        base_style = 1;
    }
    if base_style < 3 {
        let fy = 132 + rng.next_max(3) as i16 * 22;
        rows[0] = match base_style {
            0 => (88, fy),
            1 => (0, fy),
            _ => (66, fy),
        };
        let mut add_fork = |side: i32, fx: i16| {
            let bx = i + side;
            if bx < 1 || bx >= world.width - 1 {
                return;
            }
            if world.tile(bx, top - 1).is_some_and(|t| t.active()) {
                return;
            }
            let fy = 132 + rng.next_max(3) as i16 * 22;
            let idx = world.index(bx, top - 1);
            world.tiles[idx] = crate::tile::Tile {
                kind: 5,
                frame_x: fx,
                frame_y: fy,
                flags: FLAG_ACTIVE,
                ..Default::default()
            };
        };
        if base_style == 0 || base_style == 1 {
            add_fork(1, 22);
        }
        if base_style == 0 || base_style == 2 {
            add_fork(-1, 44);
        }
    }
    // Write the trunk.
    for (row_i, (fx, fy)) in rows.iter().enumerate() {
        let ty = top - 1 - row_i as i32;
        let idx = world.index(i, ty);
        world.tiles[idx] = crate::tile::Tile {
            kind: 5,
            frame_x: *fx,
            frame_y: *fy,
            flags: FLAG_ACTIVE,
            ..Default::default()
        };
        if world.frame_important.len() <= 5 {
            world.frame_important.resize(crate::tiledata::TILE_COUNT, false);
        }
        world.frame_important[5] = true;
    }
    // The branch markers sit beside the branch pieces.
    for (bx, by, fx, fy) in markers {
        if bx < 1 || bx >= world.width - 1 || by <= top - height as i32 || by >= top {
            continue;
        }
        if world.tile(bx, by).is_some_and(|t| t.active()) {
            continue;
        }
        let idx = world.index(bx, by);
        world.tiles[idx] = crate::tile::Tile {
            kind: 5,
            frame_x: fx,
            frame_y: fy,
            flags: FLAG_ACTIVE,
            ..Default::default()
        };
    }
    true
}

/// `WorldGen.GrowTreeSettings.Profiles.TryGetFromTreeId` (`:3853-3884`): the
/// tree tile kind to its profile (the gem trees 583-589, the sakura 596, the
/// willow 616, the ash 634).
fn profile_for_tree(kind: u16) -> Option<GrowTreeSettings> {
    Some(match kind {
        583..=589 => GrowTreeSettings::GEM_TREE_RUBY,
        596 => GrowTreeSettings::VANITY_TREE_SAKURA,
        616 => GrowTreeSettings::VANITY_TREE_WILLOW,
        634 => GrowTreeSettings::TREE_ASH,
        _ => return None,
    })
}

/// `WorldGen.TryConvertingOrKillingTreesAboveIfTheyWouldBecomeInvalid`
/// (`:54802-54860`): the tree standing on the spread cell is killed when the
/// new floor type fails the tree's profile ground test — a forest tree 5
/// survives only a grass/jungle/snow/mushroom ground, a palm 323 only sand,
/// and the profile trees their `GroundTest`'s kind.
///
/// The 596/616 `ConvertTreeAndGround` arm (the sakura/willow re-grounding to
/// a vanilla tree on a grass conversion) is the port's Out — the tree-
/// conversion machinery is not ported.
pub(super) fn kill_invalid_tree_above(world: &mut World, i: i32, j: i32, new_floor: u16) {
    if new_floor as usize >= crate::tiledata::TILE_COUNT {
        return;
    }
    let Some(t) = world.tile(i, j) else {
        return;
    };
    if !t.active() {
        return;
    }
    let Some(above) = world.tile(i, j - 1) else {
        return;
    };
    if !above.active() {
        return;
    }
    let tree_kind = above.kind;
    if !crate::tree::is_tree(tree_kind) && tree_kind != 323 {
        return;
    }
    let mut ground_ok = true;
    match tree_kind {
        // The C#'s `Conversion.Grass` (2/23/199/109/477/492) + `JungleGrass`
        // (60/661/662) + `Snow` (147) + `MushroomGrass` (70) sets — the port's
        // `SET_GRASS` is the first; the rest are the literal members (no port
        // sets carry them).
        5 => {
            ground_ok = crate::tiledata::SET_GRASS.has(new_floor)
                || matches!(new_floor, 60 | 661 | 662 | 147 | 70);
        }
        323 => {
            ground_ok = crate::tiledata::SET_CONVERSION_SAND.has(new_floor);
        }
        _ => {}
    }
    if let Some(profile) = profile_for_tree(tree_kind) {
        ground_ok = profile.ground.test(new_floor);
    }
    if !ground_ok {
        let idx = world.index(i, j - 1);
        world.tiles[idx] = crate::tile::Tile::default();
    }
}

/// `WallID.Sets.AllowsPlantsToGrow` (`WallID.cs:97`): the wall set
/// `DefaultTreeWallTest` (`WorldGen.cs:30260-30267`) reads — the vanity and
/// ash profiles' `WallTest`. Hand-transcribed (the generated tables carry
/// neither set).
pub(crate) const ALLOWS_PLANTS_TO_GROW: [u16; 27] = [
    0, 150, 138, 145, 107, 152, 140, 139, 141, 106, 245, 315, 317, 63, 64, 65, 66, 67, 68, 69, 81, 70,
    264, 268, 265, 74, 80,
];

/// `WorldGen.TryGrowingTreeByType` (`WorldGen.cs:30343-30387`): the
/// tree-tile → profile dispatch. The kind-5 `GrowTree` and kind-323 palm
/// arms ride their own ports (`grow_tree` / the worldgen palms); this is the
/// settings arm the sapling growth needs.
pub fn try_growing_tree_by_type(
    world: &mut World,
    tree_tile_type: u16,
    x: i32,
    y: i32,
    rng: &mut UnifiedRandom,
) -> bool {
    let profile = match tree_tile_type {
        587 => GrowTreeSettings::GEM_TREE_RUBY,
        588 => GrowTreeSettings::GEM_TREE_DIAMOND,
        583 => GrowTreeSettings::GEM_TREE_TOPAZ,
        584 => GrowTreeSettings::GEM_TREE_AMETHYST,
        589 => GrowTreeSettings::GEM_TREE_AMBER,
        586 => GrowTreeSettings::GEM_TREE_EMERALD,
        585 => GrowTreeSettings::GEM_TREE_SAPPHIRE,
        596 => GrowTreeSettings::VANITY_TREE_SAKURA,
        616 => GrowTreeSettings::VANITY_TREE_WILLOW,
        634 => GrowTreeSettings::TREE_ASH,
        // The palms ride their own port (`grow_palm_tree` below), not a
        // settings profile — the kind-20 styles 6-9 arm's target.
        323 => {
            return grow_palm_tree(world, x, y, rng);
        }
        _ => return false,
    };
    grow_tree_with_settings(world, x, y, profile, rng)
}

/// `WorldGen.GrowTreeWithSettings` (`WorldGen.cs:30388-30847`): the
/// settings-driven tree grow — the sapling-stack walk, the liquid/ground/
/// wall gates, the height roll, the trunk frame table (the `num5` branch
/// shapes with the `num4` column pick, the left/right branch arms), the
/// side-ground caps and the top cap, closing with `RangeFrame`. The
/// `TreeHeightAddon`/`ignoreWalls` args and the errorWorld height arm park
/// (the callers pass 0/false; M5-SEED-1).
pub fn grow_tree_with_settings(
    world: &mut World,
    checked_x: i32,
    checked_y: i32,
    settings: GrowTreeSettings,
    rng: &mut UnifiedRandom,
) -> bool {
    // The sapling stack walk (`:30390-30393`).
    let mut num = checked_y;
    while world.tile(checked_x, checked_y).is_some_and(|t| t.active())
        && world
            .tile(checked_x, num)
            .is_some_and(|t| t.kind == settings.sapling_tile_type)
    {
        num += 1;
    }
    // The liquid gate (`:30394-30398`): the three cells above the ground.
    for dx in [-1i32, 0, 1] {
        if world
            .tile(checked_x + dx, num - 1)
            .is_some_and(|t| t.liquid != 0)
        {
            return false;
        }
    }
    // The ground tile (`:30399-30404`): active, unsloped, no half-brick.
    let ground = match world.tile(checked_x, num) {
        Some(t) if t.active() && !t.half_brick() && t.slope() == 0 => t,
        _ => return false,
    };
    // The wall test (`:30405-30411`) — `DefaultTreeWallTest`'s set.
    let wall_ok = world
        .tile(checked_x, num - 1)
        .is_some_and(|t| ALLOWS_PLANTS_TO_GROW.contains(&t.wall));
    if !settings.ground.test(ground.kind) || !wall_ok {
        return false;
    }
    // The side grounds (`:30412-30417`): at least one must be tree soil.
    let side_soil = |dx: i32| {
        world
            .tile(checked_x + dx, num)
            .is_some_and(|t| t.active() && settings.ground.test(t.kind))
    };
    if !side_soil(-1) && !side_soil(1) {
        return false;
    }
    // `BlockColorAndCoating` (`:30418`): the ground's paint rides onto every
    // written tile. The tenthAnniversary colour roll (`:30419-30423`) is
    // seed-gated (M5-SEED-1); the errorWorld height arm (`:30425`) likewise
    // — the vanilla `getGoodWorld` callers park.
    let paint = ground.paint();
    // The height roll (`:30424`).
    let num3 = rng.next_range(settings.tree_height_min, settings.tree_height_max + 1);
    // The clear box (`:30426-30430`).
    if !empty_tile_check(
        world,
        checked_x - 2,
        checked_x + 2,
        num - (num3 + settings.tree_top_padding_needed),
        num - 1,
        Some(20),
    ) {
        return false;
    }
    // The trunk loop (`:30431-30678`).
    let mut left_branch = false;
    let mut right_branch = false;
    for i in num - num3..num {
        let num4 = rng.next_max(3);
        let mut num5 = rng.next_max(10);
        if i == num - 1 || i == num - num3 {
            num5 = 0;
        }
        // The re-roll guard (`:30454-30458`): a shape that repeats the same
        // side re-rolls until it picks another.
        while (num5 == 5 || num5 == 7) && left_branch || (num5 == 6 || num5 == 7) && right_branch {
            num5 = rng.next_max(10);
        }
        left_branch = num5 == 5 || num5 == 7;
        right_branch = num5 == 6 || num5 == 7;
        // The trunk cell's frames (`:30459-30592`): the shape table.
        let (fx, fy) = match num5 {
            1 => (0, 66 + 22 * num4),
            2 => (22, 22 * num4),
            3 => (44, 66 + 22 * num4),
            4 => (22, 66 + 22 * num4),
            5 => (88, 22 * num4),
            6 => (66, 66 + 22 * num4),
            7 => (110, 66 + 22 * num4),
            _ => (0, 22 * num4),
        };
        let idx = world.index(checked_x, i);
        let t = &mut world.tiles[idx];
        t.flags |= FLAG_ACTIVE;
        t.kind = settings.tree_tile_type;
        t.set_paint(paint);
        t.frame_x = fx as i16;
        t.frame_y = fy as i16;
        // The left branch arm (`:30596-30637`).
        if left_branch {
            let num4 = rng.next_max(3);
            let (fx, fy) = if rng.next_max(3) < 2 {
                (44, 198 + 22 * num4)
            } else {
                (66, 22 * num4)
            };
            let idx = world.index(checked_x - 1, i);
            let t = &mut world.tiles[idx];
            t.flags |= FLAG_ACTIVE;
            t.kind = settings.tree_tile_type;
            t.set_paint(paint);
            t.frame_x = fx as i16;
            t.frame_y = fy as i16;
        }
        // The right branch arm (`:30640-30678`).
        if right_branch {
            let num4 = rng.next_max(3);
            let (fx, fy) = if rng.next_max(3) < 2 {
                (66, 198 + 22 * num4)
            } else {
                (88, 66 + 22 * num4)
            };
            let idx = world.index(checked_x + 1, i);
            let t = &mut world.tiles[idx];
            t.flags |= FLAG_ACTIVE;
            t.kind = settings.tree_tile_type;
            t.set_paint(paint);
            t.frame_x = fx as i16;
            t.frame_y = fy as i16;
        }
    }
    // The side grounds (`:30680-30703`).
    let side_ground = |dx: i32| {
        world.tile(checked_x + dx, num).is_some_and(|t| {
            !t.active() && !t.half_brick() && t.slope() == 0 && is_tile_type_fit_for_tree(t.kind)
        })
    };
    let mut flag4 = side_ground(-1);
    let mut flag5 = side_ground(1);
    if rng.next_max(3) == 0 {
        flag4 = false;
    }
    if rng.next_max(3) == 0 {
        flag5 = false;
    }
    // The side caps (`:30704-30746`).
    if flag5 {
        let num4 = rng.next_max(3);
        write_tree_frame(world, checked_x + 1, num - 1, settings.tree_tile_type, paint, 22, 132 + 22 * num4);
    }
    if flag4 {
        let num4 = rng.next_max(3);
        write_tree_frame(world, checked_x - 1, num - 1, settings.tree_tile_type, paint, 44, 132 + 22 * num4);
    }
    // The centre cap (`:30748-30783`).
    let num4 = rng.next_max(3);
    let (fx, fy) = if flag4 && flag5 {
        (88, 132 + 22 * num4)
    } else if flag4 {
        (0, 132 + 22 * num4)
    } else if flag5 {
        (66, 132 + 22 * num4)
    } else {
        (0, 132 + 22 * num4) // the vanilla else-if chain leaves the cell untouched — the port writes nothing
    };
    if flag4 || flag5 {
        write_tree_frame(world, checked_x, num - 1, settings.tree_tile_type, paint, fx, fy);
    }
    // The top cap (`:30785-30818`): the 1/13 branch keeps the 22-column.
    let num4 = rng.next_max(3);
    let (fx, fy) = if rng.next_max(13) != 0 {
        (22, 198 + 22 * num4)
    } else {
        (0, 198 + 22 * num4)
    };
    write_tree_frame(world, checked_x, num - num3, settings.tree_tile_type, paint, fx, fy);
    // `RangeFrame` (`:30820`).
    crate::framing::reframe_area(world, checked_x, num - num3, 3, false);
    true
}

/// One tree cell's write — `UseBlockColors(cache)` + the frame pair.
fn write_tree_frame(
    world: &mut World,
    x: i32,
    y: i32,
    kind: u16,
    paint: u8,
    fx: i32,
    fy: i32,
) {
    let idx = world.index(x, y);
    let t = &mut world.tiles[idx];
    t.flags |= FLAG_ACTIVE;
    t.kind = kind;
    t.set_paint(paint);
    t.frame_x = fx as i16;
    t.frame_y = fy as i16;
}

/// `WorldGen.GrowPalmTree` (`WorldGen.cs:27116-27202`) — the palm trunk on
/// tile 323 (the kind-20 styles 6-9 sapling arm's target). The twin port
/// `worldgen::ocean::grow_palm_tree` rides the worldgen `Rng` stream
/// (`oceancave`/`AddTrees`); the growth pass rides its own `UnifiedRandom`
/// (the C# `genRand`), so the arm is ported locally here (the
/// `growth::spread_grass` / `evil::spread_grass` precedent) — same maths,
/// same frame table.
///
/// `treeHeightAddon`/`ignoreWalls` park (every ported caller passes 0 /
/// false) and the `RangeFrame`/`NetMessage` closing arms do not exist in
/// the port (documented in ANALYSIS).
pub fn grow_palm_tree(world: &mut World, i: i32, y: i32, rng: &mut UnifiedRandom) -> bool {
    if i < 0 || y < 0 || i >= world.width || y >= world.height {
        return false;
    }
    let mut num = y;
    // The 20-walk (`:27124-27129`).
    while world
        .tile(i, num)
        .is_some_and(|t| t.active() && t.kind == 20)
    {
        num += 1;
        if world.tile(i, num).is_none() {
            return false;
        }
    }
    let ground = match world.tile(i, num) {
        Some(t) if t.active() && !t.half_brick() && t.slope() == 0 => t,
        _ => return false,
    };
    // `BlockColorAndCoating` (`:27134`): the ground's paint rides the trunk.
    // The tenthAnniversary colour roll (`:27135-27138`) parks with M5-SEED-1.
    let paint = ground.paint();
    // The wall test + the liquid gate (`:27142-27146`).
    let wall_ok = world
        .tile(i, num - 1)
        .is_some_and(|t| ALLOWS_PLANTS_TO_GROW.contains(&t.wall));
    if world.tile(i, num - 1).is_some_and(|t| t.liquid != 0) || !wall_ok {
        return false;
    }
    // The sand ground (`:27147-27149`).
    if !matches!(ground.kind, 53 | 234 | 116 | 112) {
        return false;
    }
    // The clear boxes (`:27150-27157`).
    if !empty_tile_check(world, i, i, num - 2, num - 1, Some(20)) {
        return false;
    }
    let num2 = 20;
    if !empty_tile_check(world, i - 1, i + 1, num - num2, num - 3, Some(20)) {
        return false;
    }
    // The height roll and the lean (`:27158-27159`).
    let num3 = rng.next_range(10, 21);
    let num4 = rng.next_range(-8, 9) * 2;
    let mut num5: i16 = 0;
    for j in 0..num3 {
        if j == 0 {
            // The base (`:27168-27176`): the 66-column, the straight frameY.
            let idx = world.index(i, num - 1);
            let t = &mut world.tiles[idx];
            t.flags |= FLAG_ACTIVE;
            t.kind = 323;
            t.set_paint(paint);
            t.frame_x = 66;
            t.frame_y = 0;
            continue;
        }
        if j == num3 - 1 {
            // The top (`:27177-27185`): the 22·Next(4, 7) column.
            let fx = 22 * rng.next_range(4, 7);
            let idx = world.index(i, num - 1 - j);
            let t = &mut world.tiles[idx];
            t.flags |= FLAG_ACTIVE;
            t.kind = 323;
            t.set_paint(paint);
            t.frame_x = fx as i16;
            t.frame_y = num5;
            continue;
        }
        // The middle (`:27186-27200`): the lean advance first, then the
        // 22·Next(0, 3) column — the vanilla draw order.
        {
            // The lean walk (`:27175-27183`): the decompiled guard is an
            // always-true chain (`… || true`), so the advance fires whenever
            // the fraction `j / num3` is at or above 0.25 — the `Next(13)` /
            // `Next(9)` / `Next(5)` draws stay in their short-circuit order
            // (they advance while the branch's rolls happen in vanilla).
            let num6 = j as f64 / num3 as f64;
            if num6 >= 0.25 {
                let mut done = false;
                if num6 < 0.5 && rng.next_max(13) == 0 {
                    done = true;
                }
                if !done && num6 < 0.7 && rng.next_max(9) == 0 {
                    done = true;
                }
                if !done && !(num6 < 0.95) {
                    done = true;
                }
                if !done && rng.next_max(5) != 0 {
                    done = true;
                }
                // the trailing `|| true` — the group is always true.
                num5 += (num4.signum() * 2) as i16;
            }
        }
            let fx = 22 * rng.next_range(0, 3);
            let idx = world.index(i, num - 1 - j);
            let t = &mut world.tiles[idx];
            t.flags |= FLAG_ACTIVE;
            t.kind = 323;
            t.set_paint(paint);
            t.frame_x = fx as i16;
            t.frame_y = num5;
        }
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::growth::test_fixtures::{put, world};

    #[test]
    fn test_spread_1_sapling_grows_a_tree_with_the_gates() {
        // `GrowTree` (:29767-29980): the sapling stack walk, the liquid
        // gate, the soil + neighbour checks, the clear box, then the trunk
        // plants over the sapling.
        let mut w = world();
        put(&mut w, 100, 200, 2, 0); // the grass
        put(&mut w, 101, 200, 2, 0); // the neighbour soil
        put(&mut w, 100, 199, 20, 0); // the sapling
        let mut rng = UnifiedRandom::new(7);
        assert!(attempt_grow_tree_from_sapling(&mut w, 100, 199, false, &mut rng));
        assert!(w.tile(100, 199).is_some_and(|t| t.kind == 5), "the trunk over the sapling");
        let trunk = (0..20).filter(|&dy| {
            w.tile(100, 199 - dy).is_some_and(|t| t.active() && t.kind == 5)
        }).count();
        assert!(trunk >= 5, "the trunk height in the Next(5,17) band: {trunk}");
        // The liquid gate: a flooded neighbour refuses.
        let mut w = world();
        put(&mut w, 100, 200, 2, 0);
        put(&mut w, 101, 200, 2, 0);
        put(&mut w, 100, 199, 20, 0);
        let i = w.index(101, 199);
        w.tiles[i].liquid = 255;
        w.tiles[i].flags |= FLAG_ACTIVE;
        let mut rng = UnifiedRandom::new(7);
        assert!(!attempt_grow_tree_from_sapling(&mut w, 100, 199, false, &mut rng));
    }

    #[test]
    fn the_palm_sapling_grows_the_leaning_trunk() {
        // `AttemptToGrowTreeFromSapling`'s style-6-9 arm (`:72838-72848`) →
        // `TryGrowingTreeByType(323)` → `GrowPalmTree` (`:27116-27202`): the
        // sand ground, the 10-20 height roll, the base 66-column, the top
        // 22·Next(4,7) column and the lean walk's frameY offsets.
        let mut w = world();
        for dx in -3i32..=3 {
            put(&mut w, 100 + dx, 220, 53, 0);
        }
        put(&mut w, 100, 219, 20, 6 * 54); // the style-6 sapling frame
        let mut rng = UnifiedRandom::new(11);
        let grew = attempt_grow_tree_from_sapling(&mut w, 100, 219, false, &mut rng);
        assert!(grew, "the palm sapling grows on sand");
        assert!(
            w.tile(100, 219).is_some_and(|t| t.active() && t.kind == 323 && t.frame_x == 66),
            "the base wrote the 66-column palm"
        );
        let trunk: Vec<(i16, i16)> = (195..=219)
            .filter(|&row| w.tile(100, row).is_some_and(|t| t.active() && t.kind == 323))
            .map(|row| {
                let t = w.tile(100, row).unwrap();
                (t.frame_x, t.frame_y)
            })
            .collect();
        assert!(
            (10..=20).contains(&trunk.len()),
            "the trunk spans the height roll ({})",
            trunk.len()
        );
        let top = trunk[0];
        assert!(matches!(top.0, 88 | 110 | 132), "the top column {top:?}");
    }

    #[test]
    fn the_palm_sapling_refuses_the_wrong_ground() {
        // A sapling over non-sand ground never grows a palm (the ground
        // test at `:27147-27149`).
        let mut w = world();
        for dx in -3i32..=3 {
            put(&mut w, 100 + dx, 220, 2, 0); // forest grass, not sand
        }
        put(&mut w, 100, 219, 20, 6 * 54);
        let mut rng = UnifiedRandom::new(11);
        assert!(
            !attempt_grow_tree_from_sapling(&mut w, 100, 219, false, &mut rng),
            "the non-sand ground refuses the palm"
        );
    }

    #[test]
    fn the_sakura_sapling_grows_the_vanity_tree() {
        // `AttemptToGrowTreeFromSapling`'s 595 arm (`:72850-72861`) →
        // `GrowTreeWithSettings` with the Sakura profile (`:4002`): the
        // trunk grows the tree tile 596 with the profile's frame table,
        // gated by the ground test (the Conversion.Grass set minus the
        // 23/199 pair), the wall test, the side soil, the clear box and
        // the height roll.
        let mut w = world();
        for dx in -2i32..=2 {
            put(&mut w, 100 + dx, 210, 2, 0); // the forest grass
        }
        put(&mut w, 100, 209, 595, 0); // the sapling stack's top
        let mut rng = UnifiedRandom::new(11);
        let grew = attempt_grow_tree_from_sapling(&mut w, 100, 209, false, &mut rng);
        assert!(grew, "the sakura grows (the gates pass)");
        assert!(
            w.tile(100, 209).is_some_and(|t| t.active() && t.kind == 596),
            "the trunk wrote the sakura tile 596"
        );
        // The trunk spans the 7-12 height roll upward from the ground row.
        let trunk = (195..=210)
            .filter(|&row| w.tile(100, row).is_some_and(|t| t.active() && t.kind == 596))
            .count();
        assert!((7..=12).contains(&trunk), "the trunk spans the height roll ({trunk})");
    }

    #[test]
    fn the_sapling_gates_refuse_the_wrong_ground() {
        // A sapling on non-soil ground never grows (the ground test).
        let mut w = world();
        for dx in -2i32..=2 {
            put(&mut w, 100 + dx, 210, 1, 0); // plain dirt — not in TREE_SOILS
        }
        put(&mut w, 100, 209, 615, 0); // the willow sapling
        let mut rng = UnifiedRandom::new(11);
        assert!(
            !attempt_grow_tree_from_sapling(&mut w, 100, 209, false, &mut rng),
            "the non-soil ground refuses"
        );
        // A blocked clear box (a tile inside the height window) refuses too.
        let mut w2 = world();
        for dx in -2i32..=2 {
            put(&mut w2, 100 + dx, 210, 60, 0);
        }
        put(&mut w2, 100, 209, 615, 0);
        put(&mut w2, 100, 205, 1, 0); // the blocker inside the box
        let mut rng2 = UnifiedRandom::new(11);
        assert!(
            !attempt_grow_tree_from_sapling(&mut w2, 100, 209, false, &mut rng2),
            "the occupied clear box refuses"
        );
    }
}
