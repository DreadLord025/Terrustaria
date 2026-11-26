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
use crate::worldgen::chest_stock::WorldGenRand;

/// The `UnifiedRandom.Next(max)` / `Next(min, max)` method names the tree
/// bodies were written against, bridged onto [`WorldGenRand`]'s `next` /
/// `range` so the generic ports compile unchanged for both the runtime
/// `UnifiedRandom` and the worldgen xorshift `Rng` (W4-g42 G5). The blanket
/// impl shadows nothing — `UnifiedRandom`'s inherent methods win for a
/// concrete receiver, and both compute the identical `[0, max)` /
/// `[min, max)` draw.
trait TreeRandExt: WorldGenRand {
    fn next_max(&mut self, max: i32) -> i32 {
        self.next(max)
    }
    fn next_range(&mut self, min: i32, max: i32) -> i32 {
        self.range(min, max)
    }
}
impl<R: WorldGenRand + ?Sized> TreeRandExt for R {}

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

/// `WorldGen.GemTreeWallTest` (`WorldGen.cs:30269-30302`): the default
/// plant-wall set OR the gem trees' extra stone/dirt wall family — the gem
/// profiles' `WallTest` delegate (`:3935` etc.). The vanity/ash profiles ride
/// `DefaultTreeWallTest` (`:4005`/`:4025`). Hand-transcribed (the generated
/// tables carry neither wall set).
fn gem_tree_wall_test(wall: u16) -> bool {
    ALLOWS_PLANTS_TO_GROW.contains(&wall)
        || matches!(
            wall,
            2 | 54 | 55 | 56 | 57 | 58 | 59 | 61 | 185 | 196 | 197 | 198 | 199 | 208 | 209 | 210
                | 211 | 212 | 213 | 214 | 215
        )
}

/// The C#'s `GrowTreeSettings.WallTest` delegates (`WorldGen.cs:30260-30302`):
/// `DefaultTreeWallTest` for the vanity/ash profiles, `GemTreeWallTest` for
/// the seven gem profiles.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum WallTestKind {
    /// `WorldGen.DefaultTreeWallTest` (`:30260-30267`).
    Default,
    /// `WorldGen.GemTreeWallTest` (`:30269-30302`).
    Gem,
}

impl WallTestKind {
    /// The delegate's call (the C#'s `profile.WallTest(wallType)`).
    pub fn test(&self, wall: u16) -> bool {
        match self {
            WallTestKind::Default => ALLOWS_PLANTS_TO_GROW.contains(&wall),
            WallTestKind::Gem => gem_tree_wall_test(wall),
        }
    }
}

/// `WorldGen.GrowTreeSettings` (`WorldGen.cs:3825-4048`'s struct): one
/// gem/vanity/ash tree profile. The C#'s `GroundTest`/`WallTest` delegates
/// are the port's [`GroundTestKind`]; the wall arm stays out — no
/// smart-cursor step reads it (the C#'s other callers do).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct GrowTreeSettings {
    /// `GroundTest` — the delegate the C# stores per profile.
    pub ground: GroundTestKind,
    /// `WallTest` — the delegate the C# stores per profile (W4-g42 tail:
    /// the gem profiles carry `GemTreeWallTest`, the old port ran the
    /// default set for every profile).
    pub wall: WallTestKind,
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
        wall: WallTestKind::Gem,
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
        wall: WallTestKind::Default,
        tree_tile_type: 596,
        tree_height_min: 7,
        tree_height_max: 12,
        tree_top_padding_needed: 4,
        sapling_tile_type: 595,
    };
    /// `Profiles.VanityTree_Willow` (`WorldGen.cs:4012`).
    pub const VANITY_TREE_WILLOW: GrowTreeSettings = GrowTreeSettings {
        ground: GroundTestKind::Vanity,
        wall: WallTestKind::Default,
        tree_tile_type: 616,
        tree_height_min: 7,
        tree_height_max: 12,
        tree_top_padding_needed: 4,
        sapling_tile_type: 615,
    };
    /// `Profiles.Tree_Ash` (`WorldGen.cs:4022`).
    pub const TREE_ASH: GrowTreeSettings = GrowTreeSettings {
        ground: GroundTestKind::Ash,
        wall: WallTestKind::Default,
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
/// (`:31453-31535`), the full port (W4-g42 G5): bounds fail first; per tile
/// only active cells matter; `ignoreID −1` (None) rejects everything,
/// `11`/`71` tolerate only that kind, and a `CommonSapling` ignore-id
/// (`TileID.cs:493` — {20, 590, 595, 615}) tolerates the sapling set plus the
/// hardcoded plant list (`:31503-31528`). A non-sapling ignore-id other than
/// 11/71 leaves `flag` false and the body falls through per tile — the
/// decompile's own (lax) shape, ported verbatim (the `biome_detail`
/// `empty_tile_check` twin; the old trees.rs slice only exempted the sapling
/// set, missing the 11/71 arms `grow_shroom` needs).
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
    // `flag = ignoreID != -1 && CommonSapling[ignoreID]` (`:31471-31475`).
    let flag = ignore_id.is_some_and(is_common_sapling);
    for x in start_x..=end_x {
        for y in start_y..=end_y {
            let Some(t) = world.tile(x, y) else {
                continue;
            };
            if !t.active() {
                continue;
            }
            match ignore_id {
                None => return false,
                Some(11) | Some(71) => {
                    if t.kind != ignore_id.unwrap() {
                        return false;
                    }
                    continue;
                }
                _ => {}
            }
            if flag && !is_common_sapling(t.kind) {
                // The plant exemption switch (`:31503-31528`).
                if matches!(
                    t.kind,
                    3 | 24 | 32 | 61 | 62 | 69 | 71 | 73 | 74 | 82 | 83 | 84 | 110 | 113 | 184
                        | 201 | 233 | 352 | 485 | 529 | 530 | 637 | 655
                ) {
                    continue;
                }
                return false;
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
    // The kind-20 switch (`:72830-72853`): the style-10 sapling is the ash
    // tree; the styles 6-9 are the palms; the default arm is the vanilla
    // `GrowTree`. Both non-ash arms carry the underground gate
    // (`underground && !notTheBeesWorld && !remixWorld` → false).
    let bees = world.seeds.contains(crate::seeds::SeedFlags::NOT_THE_BEES);
    let remix = world.seeds.contains(crate::seeds::SeedFlags::REMIX);
    match t.kind {
        20 => {
            if style == 10 {
                // `:72833-72835` — `flag = TryGrowingTreeByType(634, …);
                // return flag` (the result rides the grow, not a flat true).
                try_growing_tree_by_type(world, 634, x, y, rng)
            } else if (6..=9).contains(&style) {
                // `:72836-72845`.
                if underground && !bees && !remix {
                    return false;
                }
                try_growing_tree_by_type(world, 323, x, y, rng)
            } else {
                // `:72846-72852` — the default arm's underground gate was
                // missing the bees/remix exemption in the old port.
                if underground && !bees && !remix {
                    return false;
                }
                try_growing_tree_by_type(world, 5, x, y, rng)
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
            // `:72890-72915` — `num2 = 587` BEFORE the switch; styles 0-6
            // remap, any style ≥ 7 keeps the 587 default (the old port's
            // catch-all sent it to the amber 589).
            let tree = match style {
                0 => 583,
                1 => 584,
                2 => 585,
                3 => 586,
                4 => 587,
                5 => 588,
                6 => 589,
                _ => 587,
            };
            try_growing_tree_by_type(world, tree, x, y, rng)
        }
        _ => false,
    }
}

/// The trunk/branch/fork field write (`Main.tile[x, y].active(true); .type =
/// 5; .UseBlockColors(cache); .frameX = …; .frameY = …`): a field mutation,
/// not a replace — the wall/liquid behind the tree survive, as in vanilla
/// (the `biome_detail::place_tree_tile` precedent). The bounds guard is
/// defensive (the C# indexes the padded tile array).
fn put_tree_tile(world: &mut World, x: i32, y: i32, fx: i16, fy: i16, paint: u8) {
    if x < 0 || y < 0 || x >= world.width || y >= world.height {
        return;
    }
    let idx = world.index(x, y);
    let t = &mut world.tiles[idx];
    t.flags |= FLAG_ACTIVE;
    t.kind = 5;
    t.set_paint(paint);
    t.frame_x = fx;
    t.frame_y = fy;
}

/// `WorldGen.GrowTree` (`:29767-29980`), verbatim (W4-g42 G5): the sapling
/// stack walk, the liquid gate (`!notTheBees`), the ground checks (nactive,
/// flat, soil, the wall arm `ignoreWalls || DefaultTreeWallTest` OR'd with
/// the remix/bees/wall-0 disjunction, a tree-fitting neighbour), the height
/// roll (+ the errorWorld/extraLivingTrees seed arms), the clearance (the
/// tile-70 split-box variant `:29803-29806` then the full box), then the
/// STREAMING trunk loop (top→bottom, `num4 = Next(3)` BEFORE `num5 =
/// Next(10)`, both drawn even on the forced-0 end rows, the branch else-arms
/// with their second `Next(3)`), the neighbour-driven base/forks and the top
/// `Next(13)` canopy gate. Generic over [`WorldGenRand`] so the worldgen
/// plants pass draws it on the port's xorshift `Rng` and the runtime growth
/// on `UnifiedRandom` — the same draw sequence either way.
///
/// `treeHeightAddon`/`ignoreWalls` default to 0/false (every port caller
/// passes the defaults — the `grow_tree_with_settings` precedent); the
/// tenthAnniversary paint roll (`:29783-29786`) is gated on
/// `!isGeneratingOrLoadingWorld` and the port has no runtime genRand here, so
/// it parks (paint = the ground's `BlockColorAndCoating`). `RangeFrame`
/// (`:29974`) is the framer's job (the worldgen convention); the
/// `frame_important[5]` boot bit is kept for the runtime path.
pub fn grow_tree<R: WorldGenRand>(world: &mut World, i: i32, y: i32, rng: &mut R) -> bool {
    let not_the_bees = world.seeds.contains(crate::seeds::SeedFlags::NOT_THE_BEES);
    let remix = world.seeds.contains(crate::seeds::SeedFlags::REMIX);
    // The sapling stack walk (:29769).
    let mut j = y;
    while world.tile(i, j).is_some_and(|t| t.active() && t.kind == 20) {
        j += 1;
    }
    // The liquid gate (:29772-29776) — `&& !WorldGen.notTheBees`.
    if !not_the_bees {
        for dx in [-1i32, 0, 1] {
            if world.tile(i + dx, j - 1).is_some_and(|t| t.liquid > 0) {
                return false;
            }
        }
    }
    // The ground tile (:29778): nactive, flat.
    let Some(ground) = world.tile(i, j) else {
        return false;
    };
    if !ground.active() || ground.inactive() || ground.half_brick() || ground.slope() != 0 {
        return false;
    }
    let ground_kind = ground.kind;
    let paint = ground.paint();
    // The wall arm (:29779-29780): `flag = ignoreWalls ||
    // DefaultTreeWallTest(wall)`; the gate is `soil && (((remix && j >
    // surface) || notTheBees || wall == 0) | flag) && neighbour`.
    let above_wall = world.tile(i, j - 1).map_or(0, |t| t.wall);
    let flag = ALLOWS_PLANTS_TO_GROW.contains(&above_wall); // ignoreWalls = false
    let wall_ok = (remix && (j as f64) > world.world_surface) || not_the_bees || above_wall == 0 || flag;
    if !is_tile_type_fit_for_tree(ground_kind) || !wall_ok {
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
    // The height roll (:29788-29795): `Next(5,17) + addon`, the errorWorld
    // re-roll and the extraLivingTrees bonus (both flag-gated; a classic
    // world draws neither).
    let mut num2 = rng.range(5, 17);
    let micro = world.micro_seeds;
    if micro.contains(crate::seeds::MicroSeedFlags::ERROR_WORLD) {
        let active = crate::seeds::active_secret_seed_count(micro, world.seeds);
        // `Variations.errorWorldAdjustment(3.0)` (`:329-336`).
        let adj = if active < 1 { 4 } else { (3.0 * ((active + 3) / 4) as f64) as i32 };
        if rng.next(adj) == 0 {
            num2 = rng.range(2, 101);
        }
    } else if micro.contains(crate::seeds::MicroSeedFlags::EXTRA_LIVING_TREES) {
        num2 += rng.range(0, 11);
    }
    let mut num3 = num2 + 4;
    if ground_kind == 60 {
        num3 += 5;
    }
    // The clearance (:29802-29810): the tile-70 split-box variant, then the
    // full box (both evaluated — EmptyTileCheck draws nothing).
    let mut flag2 = false;
    if ground_kind == 70
        && empty_tile_check(world, i - 2, i + 2, j - num3, j - 3, Some(20))
        && empty_tile_check(world, i - 1, i + 1, j - 2, j - 1, Some(20))
    {
        flag2 = true;
    }
    if empty_tile_check(world, i - 2, i + 2, j - num3, j - 1, Some(20)) {
        flag2 = true;
    }
    if !flag2 {
        return false;
    }
    // `flag3 = remix && j < worldSurface` (:29813) — the branch-frame variant.
    let flag3 = remix && (j as f64) < world.world_surface;
    let mut flag4 = false; // left branch last row
    let mut flag5 = false; // right branch last row
    // The streaming trunk loop (:29815-30077): k = j-num2 (top) ..< j.
    for k in (j - num2)..j {
        let mut num4 = rng.next(3);
        let mut num5 = rng.next(10);
        if k == j - 1 || k == j - num2 {
            num5 = 0;
        }
        while ((num5 == 5 || num5 == 7) && flag4) || ((num5 == 6 || num5 == 7) && flag5) {
            num5 = rng.next(10);
        }
        flag4 = num5 == 5 || num5 == 7;
        flag5 = num5 == 6 || num5 == 7;
        let dy = num4 as i16 * 22;
        let (fx, fy) = match num5 {
            1 => (0, 66 + dy),
            2 => (22, dy),
            3 => (44, 66 + dy),
            4 => (22, 66 + dy),
            5 => (88, dy),
            6 => (66, 66 + dy),
            7 => (110, 66 + dy),
            _ => (0, dy),
        };
        put_tree_tile(world, i, k, fx, fy, paint);
        // The left branch (:29981-30021).
        if num5 == 5 || num5 == 7 {
            let num4 = rng.next(3) as i16;
            if rng.next(3) < 2 && !flag3 {
                put_tree_tile(world, i - 1, k, 44, 198 + num4 * 22, paint);
            } else {
                put_tree_tile(world, i - 1, k, 66, num4 * 22, paint);
            }
        }
        // The right branch (:30022-30077).
        if num5 == 6 || num5 == 7 {
            let num4 = rng.next(3) as i16;
            if rng.next(3) < 2 && !flag3 {
                put_tree_tile(world, i + 1, k, 66, 198 + num4 * 22, paint);
            } else {
                put_tree_tile(world, i + 1, k, 88, 66 + num4 * 22, paint);
            }
        }
    }
    // The base/fork orientation (:30078-30110): `num6 = Next(3)` remapped by
    // the flat tree-soil neighbours (IsTileTypeFitForTree, all 12 soils).
    let mut num6 = rng.next(3);
    let soil_at = |dx: i32| {
        world.tile(i + dx, j).is_some_and(|t| {
            t.active() && !t.inactive() && !t.half_brick() && t.slope() == 0 && is_tile_type_fit_for_tree(t.kind)
        })
    };
    let flag6 = soil_at(-1);
    let flag7 = soil_at(1);
    if !flag6 {
        if num6 == 0 { num6 = 2; }
        if num6 == 1 { num6 = 3; }
    }
    if !flag7 {
        if num6 == 0 { num6 = 1; }
        if num6 == 2 { num6 = 3; }
    }
    if flag6 && !flag7 { num6 = 2; }
    if flag7 && !flag6 { num6 = 1; }
    // The forks (:30111-30156) — unconditional writes (the C# overwrites).
    if num6 == 0 || num6 == 1 {
        let num4 = rng.next(3) as i16;
        put_tree_tile(world, i + 1, j - 1, 22, 132 + num4 * 22, paint);
    }
    if num6 == 0 || num6 == 2 {
        let num4 = rng.next(3) as i16;
        put_tree_tile(world, i - 1, j - 1, 44, 132 + num4 * 22, paint);
    }
    // The base row frame (:30157-30200).
    let num4 = rng.next(3) as i16;
    match num6 {
        0 => put_tree_tile(world, i, j - 1, 88, 132 + num4 * 22, paint),
        1 => put_tree_tile(world, i, j - 1, 0, 132 + num4 * 22, paint),
        2 => put_tree_tile(world, i, j - 1, 66, 132 + num4 * 22, paint),
        _ => {} // num6 3: no soil neighbour — the base keeps its trunk frame
    }
    // The top-row rewrite (:30201-30240): `Next(13) != 0 && !flag3` keeps the
    // canopy marker (frameX 22), else the plain trunk art (frameX 0).
    let canopy = rng.next(13) != 0 && !flag3;
    let num4 = rng.next(3) as i16;
    put_tree_tile(world, i, j - num2, if canopy { 22 } else { 0 }, 198 + num4 * 22, paint);
    // `RangeFrame(i-2, j-num2-1, i+2, j+1)` (:30241) is the framer's job; the
    // frame-important boot bit serves the runtime path.
    if world.frame_important.len() <= 5 {
        world.frame_important.resize(crate::tiledata::TILE_COUNT, false);
    }
    world.frame_important[5] = true;
    true
}

/// `WorldGen.GrowTreeSettings.Profiles.TryGetFromTreeId` (`:3853-3884`): the
/// tree tile kind to its profile (the gem trees 583-589, the sakura 596, the
/// willow 616, the ash 634).
fn profile_for_tree(kind: u16) -> Option<GrowTreeSettings> {
    Some(match kind {
        // `TryGetFromTreeId` (`:3897-3917`) — the per-kind gem profile (the
        // ground/wall delegates are shared, but the tree tile differs; the
        // old port returned RUBY for all seven).
        589 => GrowTreeSettings::GEM_TREE_AMBER,
        584 => GrowTreeSettings::GEM_TREE_AMETHYST,
        588 => GrowTreeSettings::GEM_TREE_DIAMOND,
        586 => GrowTreeSettings::GEM_TREE_EMERALD,
        587 => GrowTreeSettings::GEM_TREE_RUBY,
        585 => GrowTreeSettings::GEM_TREE_SAPPHIRE,
        583 => GrowTreeSettings::GEM_TREE_TOPAZ,
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
pub(crate) fn kill_invalid_tree_above(world: &mut World, i: i32, j: i32, new_floor: u16) {
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
/// neither set). The overworld flower arm reads it too (`:59543`).
pub(super) fn allows_plants_to_grow(wall: u16) -> bool {
    ALLOWS_PLANTS_TO_GROW.contains(&wall)
}

pub(crate) const ALLOWS_PLANTS_TO_GROW: [u16; 27] = [
    0, 150, 138, 145, 107, 152, 140, 139, 141, 106, 245, 315, 317, 63, 64, 65, 66, 67, 68, 69, 81, 70,
    264, 268, 265, 74, 80,
];

/// `WorldGen.TryGrowingTreeByType` (`WorldGen.cs:30343-30387`): the
/// tree-tile → profile dispatch. The kind-5 arm is `GrowTree` (`:30348-
/// 30350`, W4-g42 G5 — the mushroom-grass trees the plants pass grows), the
/// kind-323 palm rides `grow_palm_tree`, the rest the settings profiles.
/// Generic over [`WorldGenRand`] so the worldgen plants pass dispatches on
/// the port's xorshift `Rng` and the runtime growth on `UnifiedRandom`.
pub fn try_growing_tree_by_type<R: WorldGenRand>(
    world: &mut World,
    tree_tile_type: u16,
    x: i32,
    y: i32,
    rng: &mut R,
) -> bool {
    let profile = match tree_tile_type {
        // `case 5: result = GrowTree(...)` (`:30348-30350`).
        5 => return grow_tree(world, x, y, rng),
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
/// `TreeHeightAddon`/`ignoreWalls` args park (the callers pass 0/false);
/// the errorWorld height arm (`:30420-30423`) and the profile `WallTest`
/// delegates are verbatim (W4-g42 tail). The tenthAnniversary paint roll
/// (`:30414-30417`) parks — runtime-only + seed-gated (M5-SEED-1).
pub fn grow_tree_with_settings<R: WorldGenRand>(
    world: &mut World,
    checked_x: i32,
    checked_y: i32,
    settings: GrowTreeSettings,
    rng: &mut R,
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
    // The ground tile (`:30399-30403`): `nactive()` (the active bit set, the
    // inActive bit clear), unsloped, no half-brick.
    let ground = match world.tile(checked_x, num) {
        Some(t) if t.active() && !t.inactive() && !t.half_brick() && t.slope() == 0 => t,
        _ => return false,
    };
    // The wall test (`:30404-30408`) — the profile's `WallTest` delegate
    // (`GemTreeWallTest` for the gem profiles, `DefaultTreeWallTest` for the
    // vanity/ash ones; W4-g42 tail — the old port ran the default set for
    // every profile, refusing gem saplings against the stone-wall family).
    let wall_ok = world
        .tile(checked_x, num - 1)
        .is_some_and(|t| settings.wall.test(t.wall));
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
    // `BlockColorAndCoating` (`:30413`): the ground's paint rides onto every
    // written tile. The tenthAnniversary colour roll (`:30414-30417`) is
    // seed-gated AND runtime-only (`!isGeneratingOrLoadingWorld`) and parks
    // (M5-SEED-1).
    let paint = ground.paint();
    // The height roll (`:30418-30423`): `Next(min, max+1) + addon`, then the
    // errorWorld re-roll — `Next(errorWorldAdjustment(2.0)) == 0` picks
    // `Next(3, 15)` below the surface line or `Next(3, 50)` above it
    // (W4-g42 tail; the addon parks at 0 — every ported caller's default).
    let mut num3 = rng.next_range(settings.tree_height_min, settings.tree_height_max + 1);
    if world
        .micro_seeds
        .contains(crate::seeds::MicroSeedFlags::ERROR_WORLD)
    {
        let active = crate::seeds::active_secret_seed_count(world.micro_seeds, world.seeds);
        // `Variations.errorWorldAdjustment(2.0)` (`WorldGen.cs:329-336`).
        let adj = if active < 1 { 4 } else { (2.0 * ((active + 3) / 4) as f64) as i32 };
        if rng.next(adj) == 0 {
            num3 = if (checked_y as f64) < world.world_surface {
                rng.next_range(3, 50)
            } else {
                rng.next_range(3, 15)
            };
        }
    }
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
    // The side grounds (`:30686-30703`): `nactive()` + flat + tree-soil —
    // the ACTIVE neighbours seed the side caps (W4-g42 tail: the old port
    // read `!t.active()`, the inverted polarity, so the caps landed on the
    // wrong side of the roll).
    let side_ground = |dx: i32| {
        world.tile(checked_x + dx, num).is_some_and(|t| {
            t.active() && !t.inactive() && !t.half_brick() && t.slope() == 0 && is_tile_type_fit_for_tree(t.kind)
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
    // The top cap (`:30803-30840`): the `Next(13)` gate draws FIRST, then the
    // taken branch's `Next(3)` column pick (W4-g42 tail: the old port drew
    // the column before the gate — one draw of the wrong modulus early).
    let canopy = rng.next_max(13) != 0;
    let num4 = rng.next_max(3);
    let (fx, fy) = if canopy {
        (22, 198 + 22 * num4)
    } else {
        (0, 198 + 22 * num4)
    };
    write_tree_frame(world, checked_x, num - num3, settings.tree_tile_type, paint, fx, fy);
    // The frame-important boot bit for the grown tree kind (the `grow_tree`
    // precedent — the runtime kill cascade / framer must recognise the tile).
    if world.frame_important.len() <= settings.tree_tile_type as usize {
        world.frame_important.resize(crate::tiledata::TILE_COUNT, false);
    }
    world.frame_important[settings.tree_tile_type as usize] = true;
    // `RangeFrame(checkedX - 2, num - num3 - 1, checkedX + 2, num + 1)`
    // (`:30841`) — the port's radius-clamped superset (reset: false, no draw).
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
/// false); the `RangeFrame`/`NetMessage` closing arms are the framer's /
/// MP's job (the `grow_tree` convention). The lean walk's `num5 != num4`
/// gate (`:27185`) is verbatim (W4-g42 tail).
pub fn grow_palm_tree<R: WorldGenRand>(world: &mut World, i: i32, y: i32, rng: &mut R) -> bool {
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
        // The middle (`:27185-27198`): the lean advance first, then the
        // 22·Next(0, 3) column — the vanilla draw order. The whole lean walk
        // sits behind `num5 != num4` (`:27185`, W4-g42 tail — the old port
        // omitted the gate, so a palm that reached its target lean kept
        // drawing the Next(13)/Next(9)/Next(5) rolls every middle row).
        if num5 != num4 as i16 {
            // The lean walk (`:27187-27192`): the decompiled guard is an
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

/// `WorldGen.GrowShroom` (`WorldGen.cs:31301-31362`), verbatim (W4-g42 G5):
/// the giant glowing-mushroom tree — tile-72 trunk on the 18 px grid, the
/// `Next(4, 11)` height, the three-cell lava gate (the C# reads `i-1` TWICE
/// — the middle-cell check is a decompile typo, kept), the mushroom-grass
/// ground + both-side-70 + wall-0 gate, the two `EmptyTileCheck(…, 71)`
/// boxes, the gen-time `Next(3) != 0` slope flatten, then the trunk column
/// (`Next(3)` frameY per cell) and the cap (`Next(3)` frameY at frameX 36).
///
/// Census fact (research/12j-g42): `GrowShroom` has ZERO callers in the whole
/// decompile — a dead-code vanilla helper in 1.4.5.8. The gen-time mushroom
/// "trees" are tile-5 trunks on grass 70 (`grow_tree`); tile 72 arises from
/// the runtime framer (`TileFrame case 72`, `:85837-85874`). Ported verbatim
/// and wired into nothing (the faithful state); the witnesses pin its draws.
/// `isGeneratingOrLoadingWorld` is true at the port's worldgen call sites;
/// the runtime path (no caller today) would pass false — the parameter keeps
/// the C# shape. `RangeFrame`/`NetMessage` close arms are the framer/MP-only.
pub fn grow_shroom<R: WorldGenRand>(
    world: &mut World,
    i: i32,
    y: i32,
    rng: &mut R,
    generating: bool,
) -> bool {
    // The lava gate (`:31303-31306`) — `anyLava()` on (i-1, y-1) twice and
    // (i+1, y-1); the middle cell is never read (the decompile typo).
    let lava_at = |dx: i32| {
        world
            .tile(i + dx, y - 1)
            .is_some_and(|t| t.liquid > 0 && t.liquid_kind() == crate::tile::LiquidKind::Lava)
    };
    if lava_at(-1) || lava_at(-1) || lava_at(1) {
        return false;
    }
    // The ground gate (`:31307`): nactive 70, wall-0 above, both sides
    // active 70, the two 71-ignore clear boxes.
    let ground_ok = world.tile(i, y).is_some_and(|t| {
        t.active() && !t.inactive() && t.kind == 70
    });
    let wall0 = world.tile(i, y - 1).is_some_and(|t| t.wall == 0);
    let side70 = |dx: i32| {
        world.tile(i + dx, y).is_some_and(|t| t.active() && t.kind == 70)
    };
    if !ground_ok
        || !wall0
        || !side70(-1)
        || !side70(1)
        || !empty_tile_check(world, i - 2, i + 2, y - 13, y - 3, Some(71))
        || !empty_tile_check(world, i - 1, i + 1, y - 3, y - 1, Some(71))
    {
        return false;
    }
    // The gen-time slope flatten (`:31309-31313`): `Next(3) != 0` clears the
    // ground's half-brick/slope (the draw happens only when generating).
    if generating && rng.next(3) != 0 {
        if let Some(t) = world.tile_mut(i, y) {
            t.flags &= !crate::tile::FLAG_HALF_BRICK;
            t.set_slope(0);
        }
    }
    // The flat re-check (`:31314`).
    if world.tile(i, y).is_some_and(|t| t.half_brick() || t.slope() != 0) {
        return false;
    }
    // The trunk (`:31316-31338`): `num = Next(4, 11)`, j = y-num ..< y.
    let num = rng.range(4, 11);
    for j in (y - num)..y {
        let num2 = rng.next(3);
        let fy = (num2 * 18) as i16;
        if i < 0 || j < 0 || i >= world.width || j >= world.height {
            continue;
        }
        let idx = world.index(i, j);
        let t = &mut world.tiles[idx];
        t.flags |= FLAG_ACTIVE;
        t.kind = 72;
        t.frame_x = 0;
        t.frame_y = fy;
    }
    // The cap (`:31339-31353`): frameX 36, frameY = Next(3) * 18.
    let num3 = rng.next(3);
    let cap_y = y - num;
    if i >= 0 && cap_y >= 0 && i < world.width && cap_y < world.height {
        let idx = world.index(i, cap_y);
        let t = &mut world.tiles[idx];
        t.frame_x = 36;
        t.frame_y = (num3 * 18) as i16;
    }
    // `RangeFrame(i-2, y-num-1, i+2, y+1)` (`:31354`) is the framer's job;
    // the frame-important boot bit serves the runtime path.
    if world.frame_important.len() <= 72 {
        world.frame_important.resize(crate::tiledata::TILE_COUNT, false);
    }
    world.frame_important[72] = true;
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

    /// W4-g42 G5: `TryGrowingTreeByType(5)` dispatches to `GrowTree`
    /// (`:30348-30350`) — the case the old port lacked (it returned false).
    /// A sapling on forest grass grows a tile-5 trunk through the dispatch.
    #[test]
    fn w4_g42_try_growing_tree_by_type_5_grows_the_vanilla_tree() {
        let mut w = world();
        put(&mut w, 100, 200, 2, 0); // the grass
        put(&mut w, 101, 200, 2, 0); // the neighbour soil
        put(&mut w, 100, 199, 20, 0); // the sapling
        let mut rng = UnifiedRandom::new(7);
        assert!(
            try_growing_tree_by_type(&mut w, 5, 100, 199, &mut rng),
            "the case-5 arm grows the tree"
        );
        assert!(
            w.tile(100, 199).is_some_and(|t| t.active() && t.kind == 5),
            "the trunk over the sapling"
        );
        // The worldgen xorshift stream drives the same dispatch (the generic
        // seam the plants pass uses).
        let mut w2 = world();
        put(&mut w2, 100, 200, 2, 0);
        put(&mut w2, 101, 200, 2, 0);
        put(&mut w2, 100, 199, 20, 0);
        let mut xrng = crate::rng::Xorshift64(0x5EED);
        assert!(
            try_growing_tree_by_type(&mut w2, 5, 100, 199, &mut xrng),
            "the xorshift stream grows the tree too"
        );
    }

    /// W4-g42 G5: the tile-70 split-box clearance (`:29803-29806`) — a
    /// mushroom-grass ground with a blocker in the `j-2..j-1` strip's wide
    /// columns but a clear narrow strip + upper box still grows (the 70
    /// variant), where a forest grass with the same blocker refuses.
    #[test]
    fn w4_g42_the_mushroom_grass_split_box_clearance() {
        // The blocker at (i+2, j-2): inside the FULL box (i±2, j-num3..j-1)
        // but OUTSIDE the 70 variant's upper box (ends j-3) and narrow strip
        // (i±1). The 70 ground passes via the variant; grass 2 fails the full
        // box and has no variant.
        let grow_with = |ground: u16| -> bool {
            let mut w = world();
            for dx in -2i32..=2 {
                put(&mut w, 100 + dx, 200, ground, 0);
            }
            put(&mut w, 100, 199, 20, 0); // the sapling
            put(&mut w, 102, 198, 1, 0); // the blocker at (i+2, j-2)
            let mut rng = UnifiedRandom::new(7);
            grow_tree(&mut w, 100, 199, &mut rng)
        };
        assert!(grow_with(70), "the 70 split-box variant clears");
        assert!(!grow_with(2), "the forest grass full box refuses");
    }

    /// W4-g42 G5: `GrowShroom` (`:31301-31362`) — the tile-72 trunk on the
    /// 18 px grid, the `Next(4,11)` height, the cap at frameX 36. The gates:
    /// both-side-70 + wall-0 + the 71-ignore clear boxes.
    #[test]
    fn w4_g42_grow_shroom_grows_the_giant_mushroom() {
        let mut w = world();
        for dx in -2i32..=2 {
            put(&mut w, 100 + dx, 200, 70, 0); // the mushroom-grass ground row
        }
        let mut rng = UnifiedRandom::new(7);
        assert!(
            grow_shroom(&mut w, 100, 200, &mut rng, true),
            "the shroom grows on the 70 ground"
        );
        // The trunk: tile 72, frameX 0, frameY ∈ {0,18,36}, height 4..10.
        let trunk: Vec<(i16, i16)> = (185..200)
            .filter(|&row| w.tile(100, row).is_some_and(|t| t.active() && t.kind == 72))
            .map(|row| {
                let t = w.tile(100, row).unwrap();
                (t.frame_x, t.frame_y)
            })
            .collect();
        assert!((4..=10).contains(&trunk.len()), "the Next(4,11) height ({})", trunk.len());
        // The cap is the topmost trunk cell rewritten to frameX 36.
        let cap_row = 200 - trunk.len() as i32;
        let cap = w.tile(100, cap_row).unwrap();
        assert_eq!(cap.frame_x, 36, "the cap column");
        assert!(matches!(cap.frame_y, 0 | 18 | 36), "the cap frameY");
        // The cells below the cap keep frameX 0 and a valid frameY.
        for &(fx, fy) in trunk.iter().skip(1) {
            assert_eq!(fx, 0, "the trunk column");
            assert!(matches!(fy, 0 | 18 | 36), "the trunk frameY");
        }
    }

    /// W4-g42 G5: `GrowShroom`'s gates — a non-70 ground, a missing side-70,
    /// a wall above, or lava in the gate cells all refuse (no draws past the
    /// gate). The lava gate reads (i-1, y-1) TWICE (the decompile typo).
    #[test]
    fn w4_g42_grow_shroom_gates_refuse() {
        // Non-70 ground.
        let mut w = world();
        for dx in -2i32..=2 {
            put(&mut w, 100 + dx, 200, 2, 0);
        }
        let mut rng = UnifiedRandom::new(7);
        assert!(!grow_shroom(&mut w, 100, 200, &mut rng, true), "the grass ground refuses");
        // A missing side-70 (the right NEIGHBOUR i+1 is dirt; i+2 is not
        // read by the gate).
        let mut w2 = world();
        for dx in -2i32..=1 {
            put(&mut w2, 100 + dx, 200, 70, 0);
        }
        put(&mut w2, 101, 200, 2, 0);
        let mut rng2 = UnifiedRandom::new(7);
        assert!(!grow_shroom(&mut w2, 100, 200, &mut rng2, true), "the missing side-70 refuses");
        // Lava in the (i-1, y-1) gate cell.
        let mut w3 = world();
        for dx in -2i32..=2 {
            put(&mut w3, 100 + dx, 200, 70, 0);
        }
        let i = w3.index(99, 199);
        w3.tiles[i].liquid = 255;
        w3.tiles[i].set_liquid_kind(crate::tile::LiquidKind::Lava);
        let mut rng3 = UnifiedRandom::new(7);
        assert!(!grow_shroom(&mut w3, 100, 200, &mut rng3, true), "the lava gate refuses");
    }

    /// W4-g42 G5: the `EmptyTileCheck` 71-ignore arm tolerates 71-cells and
    /// rejects others (`:31487-31493`); the sapling-ignore plant exemption
    /// tolerates the hardcoded plant list (`:31503-31528`).
    #[test]
    fn w4_g42_empty_tile_check_ignore_arms() {
        let mut w = world();
        // A 71-cell inside the box: the 71-ignore passes, the 20-ignore
        // (sapling) also passes via the plant exemption (71 is in the list).
        put(&mut w, 100, 190, 71, 0);
        assert!(empty_tile_check(&w, 99, 101, 189, 191, Some(71)), "71-ignore tolerates 71");
        assert!(empty_tile_check(&w, 99, 101, 189, 191, Some(20)), "the plant exemption tolerates 71");
        // A stone cell: both refuse.
        put(&mut w, 100, 190, 1, 0);
        assert!(!empty_tile_check(&w, 99, 101, 189, 191, Some(71)), "71-ignore rejects stone");
        assert!(!empty_tile_check(&w, 99, 101, 189, 191, Some(20)), "the sapling-ignore rejects stone");
        // The None (-1) ignore rejects everything active.
        assert!(!empty_tile_check(&w, 99, 101, 189, 191, None), "the -1 ignore rejects");
    }

    /// W4-g42 tail T1: the gem profiles' `WallTest` is `GemTreeWallTest`
    /// (`:30269-30302`, `:3935`), not the default set — a gem sapling with the
    /// dirt wall 2 behind it (in the gem set, NOT in `AllowsPlantsToGrow`)
    /// grows; a wall in neither set refuses. The old port ran the default set
    /// for every profile and refused the gem sapling against wall 2.
    #[test]
    fn w4_g42_tail_gem_wall_test_accepts_the_stone_wall_family() {
        let grow_with_wall = |wall: u16| -> bool {
            let mut w = world();
            for dx in -2i32..=2 {
                put(&mut w, 100 + dx, 210, 1, 0); // the stone ground (gem soil)
            }
            put(&mut w, 100, 209, 590, 0); // the gem sapling, style 0 (topaz)
            let i = w.index(100, 209);
            w.tiles[i].wall = wall; // the wall behind the tree (num-1)
            let mut rng = UnifiedRandom::new(7);
            attempt_grow_tree_from_sapling(&mut w, 100, 209, true, &mut rng)
        };
        assert!(grow_with_wall(2), "the dirt wall 2 is in GemTreeWallTest");
        assert!(grow_with_wall(54), "the stone wall 54 is in GemTreeWallTest");
        assert!(!grow_with_wall(1), "a wall in neither set refuses");
    }

    /// W4-g42 tail T10: the gem sapling's style switch (`:72890-72915`) keeps
    /// `num2 = 587` (ruby) for any style ≥ 7 — the old port's catch-all sent it
    /// to the amber 589. Style 6 → 589 (amber), style 7 → 587 (ruby).
    #[test]
    fn w4_g42_tail_gem_sapling_style_default_keeps_ruby() {
        let trunk_kind = |style: i16| -> u16 {
            let mut w = world();
            for dx in -2i32..=2 {
                put(&mut w, 100 + dx, 210, 1, 0);
            }
            put(&mut w, 100, 209, 590, style * 54);
            let mut rng = UnifiedRandom::new(7);
            attempt_grow_tree_from_sapling(&mut w, 100, 209, true, &mut rng);
            // The trunk base over the sapling carries the grown tree kind.
            w.tile(100, 209).map(|t| t.kind).unwrap_or(0)
        };
        assert_eq!(trunk_kind(6), 589, "style 6 → amber");
        assert_eq!(trunk_kind(7), 587, "style 7 keeps the ruby default");
        assert_eq!(trunk_kind(9), 587, "style 9 keeps the ruby default");
    }

    /// W4-g42 tail T8: the style-10 ash sapling arm returns the grow RESULT
    /// (`:72833-72835` `return flag`), not a flat true — on ash grass 633 it
    /// grows the ash tree 634 and returns true; on a non-ash ground it returns
    /// false (the old port returned true regardless).
    #[test]
    fn w4_g42_tail_ash_sapling_returns_the_grow_result() {
        // Ash grass 633 ground → grows 634, returns true.
        let mut w = world();
        for dx in -2i32..=2 {
            put(&mut w, 100 + dx, 210, 633, 0);
        }
        put(&mut w, 100, 209, 20, 10 * 54); // the style-10 ash sapling
        let mut rng = UnifiedRandom::new(7);
        assert!(
            attempt_grow_tree_from_sapling(&mut w, 100, 209, false, &mut rng),
            "the ash sapling grows on ash grass"
        );
        assert!(w.tile(100, 209).is_some_and(|t| t.kind == 634), "the ash tree 634");
        // A non-ash ground (forest grass) → the ash ground test refuses, false.
        let mut w2 = world();
        for dx in -2i32..=2 {
            put(&mut w2, 100 + dx, 210, 2, 0);
        }
        put(&mut w2, 100, 209, 20, 10 * 54);
        let mut rng2 = UnifiedRandom::new(7);
        assert!(
            !attempt_grow_tree_from_sapling(&mut w2, 100, 209, false, &mut rng2),
            "the ash sapling on forest grass returns false (not flat true)"
        );
    }

    /// W4-g42 tail T2/T3/T6: a gem sapling on stone with ACTIVE stone
    /// neighbours grows the gem tree (the side-ground flags read `nactive()`,
    /// the ground gate checks `!inactive()`) and boots the tree kind's
    /// frame-important bit so the runtime kill cascade recognises it.
    #[test]
    fn w4_g42_tail_gem_tree_grows_and_boots_frame_important() {
        let mut w = world();
        for dx in -2i32..=2 {
            put(&mut w, 100 + dx, 210, 1, 0); // active stone ground + neighbours
        }
        put(&mut w, 100, 209, 590, 4 * 54); // style 4 → ruby 587
        let mut rng = UnifiedRandom::new(7);
        assert!(
            attempt_grow_tree_from_sapling(&mut w, 100, 209, true, &mut rng),
            "the gem sapling grows on active stone"
        );
        assert!(w.tile(100, 209).is_some_and(|t| t.kind == 587), "the ruby tree 587");
        assert!(
            w.frame_important.get(587).copied().unwrap_or(false),
            "the grown tree kind's frame-important bit is booted"
        );
        // An actuated (inactive) ground refuses (`nactive()` is false).
        let mut w2 = world();
        for dx in -2i32..=2 {
            put(&mut w2, 100 + dx, 210, 1, 0);
        }
        let gi = w2.index(100, 210);
        w2.tiles[gi].flags |= crate::tile::FLAG_INACTIVE;
        put(&mut w2, 100, 209, 590, 4 * 54);
        let mut rng2 = UnifiedRandom::new(7);
        assert!(
            !attempt_grow_tree_from_sapling(&mut w2, 100, 209, true, &mut rng2),
            "the actuated ground refuses (nactive is false)"
        );
    }
}
