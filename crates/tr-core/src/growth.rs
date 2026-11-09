//! The world-growth ticks — `WorldGen.UpdateWorld` (`WorldGen.cs:71495-71632`),
//! `GrowAlch` (`:45777-45890`) and the crystal-shard arm of `hardUpdateWorld`
//! (`:69600-69640`).
//!
//! Source map (line numbers into `decompiled/Terraria/`):
//! - the rate (`GetWorldUpdateRate` `:72020-72030`): `min(dayRate, 24)`, 0
//!   when the time freezes; `dayRate` = the time rate ×5 when every player
//!   sleeps (`Main.cs:6210-6220`)
//! - the loops (`:71540-71632`): the overground
//!   `maxX·maxY · 3e-5 · rate` (×1.5 raining), the underground
//!   `maxX·maxY · 1.5e-5 · rate`; each overground pick rolls the alch-seed
//!   gate `Next(num7·100) == 0 → skip` with
//!   `num7 = Lerp(151, 151·2.8, clamp(maxTilesX/4200 − 1, 0, 1))`
//!   (`:71606-71614`)
//! - `UpdateWorld_OvergroundTile` (`:72030+`): the port's slice is the herb
//!   branch (`TileID.Sets.tileAlch` = {82, 83, 84} → `GrowAlch`); the other
//!   arms (the trees, the cacti, the shells, the bamboo) land with their
//!   follow-ups
//! - `GrowAlch` (`:45777-45890`): the herb lifecycle — the mature 84
//!   blooms into 83 by the day/night/rain/depth cue (the frameX stages
//!   0/18/36/72/90/108), the 83 flashes stage frames, the sprout 82 matures;
//!   the liquid kill keeps only the matching stage
//! - the crystal shards (`hardUpdateWorld` `:69603-69640`): the
//!   `CanGrowCrystalShards` stones {117, 116, 164, 402, 403} below the rock
//!   layer roll `Next(5)`, pick a 4-dir neighbour, need it empty, keep the
//!   density `< 2` per radius-6 box and place tile 129 with the style
//!   `Next(18)` (the big shard `18 + Next(6)` at 1/50)
//!
//! Deviations (ANALYSIS "Growth ticks (M7-GROWTH-1)"): the vines/moss and
//! the biome spread = M5-SPREAD-2's follow-up; the notTheBees/remix loop
//! variants are skipped (the port generates no such seeds); the NetMessage
//! sides are MP scope.
//!
//! M5-SPREAD-1 adds the tree + cactus arms: the sapling dispatch (`:72430-
//! 72449` — the kind-20 sapling at `Next(20)`, the default arm → `GrowTree`),
//! `IsTileTypeFitForTree` (`:29695-29710`), `GrowTree`'s gates + the frame
//! table (`:29767-29880+`, the same shape the worldgen `plant_trees` ports),
//! and `GrowCactus` (`:56575-56815`: the water-box abort
//! `cactusWaterWidth=50/Height=25/Limit=25` :86930-86932, the sand arm's
//! ≤3-cactus + >10-sand gates, the tall-arm's 11-height budget + the
//! `Next(11,13)` cap, the three-way fork/lean/highten rolls).

use crate::tile::FLAG_ACTIVE;
use crate::tile_light::FastRandom;
use crate::world::World;

/// `TileID.Sets.tileAlch` (`TileID.cs`) — the herb tiles GrowAlch runs on.
pub const TILE_ALCH: [u16; 3] = [82, 83, 84];

/// `TileID.Sets.CanGrowCrystalShards` (`TileID.cs:585`).
pub const CAN_GROW_CRYSTAL_SHARDS: [u16; 5] = [117, 116, 164, 402, 403];

/// `GetWorldUpdateRate` (`WorldGen.cs:72020-72030`): `min(dayRate, 24)`,
/// 0 while the time freezes.
pub fn world_update_rate(day_rate: i32, freeze_time: bool) -> i32 {
    if freeze_time {
        return 0;
    }
    day_rate.min(24)
}

/// The alch-seed gate (`:71606-71614`): `Next(num7·100) == 0` skips the
/// pick; `num7 = Lerp(151, 151·2.8, clamp(maxTilesX/4200 − 1, 0, 1))`.
pub fn alch_seed_skips(world: &World, rng: &mut FastRandom) -> bool {
    let t = ((world.width as f64 / 4200.0 - 1.0).clamp(0.0, 1.0));
    let num7 = 151.0 + (151.0 * 2.8 - 151.0) * t;
    rng.next((num7 * 100.0) as i32) == 0
}

/// `GrowAlch` (`WorldGen.cs:45777-45890`): the herb lifecycle. The stages
/// ride `frameX` (0/18/36/72/90/108) and the bloom target 83; the mature 84
/// blooms back; the liquid kill keeps only the matching stage.
pub fn grow_alch(world: &mut World, x: i32, y: i32, day_time: bool, raining: bool, rng: &mut FastRandom) {
    let Some(t) = world.tile(x, y) else {
        return;
    };
    if !t.active() {
        return;
    }
    // `:45784-45796`: the liquid kill — the wrong liquid for the stage
    // destroys the herb (the lava keeps only the stage-5 cactus-stem shape).
    if t.liquid > 0 {
        let stage = t.frame_x / 18;
        let lava = t.flags & crate::tile::FLAG_LIQUID_LAVA != 0;
        let water = t.liquid_kind() == crate::tile::LiquidKind::Water;
        let keep = (lava && stage == 5) || (water && (stage == 1 || stage == 4));
        if !keep {
            crate::edit::kill_tile(world, x, y, false);
            return;
        }
    }
    if t.kind == 82 {
        // The sprout matures: the special cues bloom straight into 83.
        let frame_x = world.tile(x, y).map(|t| t.frame_x).unwrap_or(0);
        let below_surface = y as f64 > world.world_surface;
        let near_underworld = y > world.height - 200;
        let bloom = if rng.next(50) == 0 && (frame_x == 108 || rng.next(2) == 0) {
            true
        } else if day_time && frame_x == 0 && rng.next(50) == 0 {
            true
        } else if !day_time && frame_x == 18 && rng.next(50) == 0 {
            true
        } else if raining && frame_x == 72 && rng.next(50) == 0 {
            true
        } else if below_surface && frame_x == 36 && rng.next(50) == 0 {
            true
        } else if near_underworld && frame_x == 90 && rng.next(50) == 0 {
            true
        } else {
            false
        };
        if bloom {
            if let Some(t) = world.tile_mut(x, y) {
                t.kind = 83;
            }
        }
        return;
    }
    // `:45862-45883`: the stage-3 flash (`frameX == 36`, 2 of 3 rolls skip).
    if world.tile(x, y).is_some_and(|t| t.frame_x == 36) && rng.next(3) != 0 {
        let kind = world.tile(x, y).map(|t| t.kind).unwrap_or(0);
        if kind == 83 {
            if rng.next(2) == 0 {
                if let Some(t) = world.tile_mut(x, y) {
                    t.kind = 84;
                }
            }
        } else if kind == 84 || rng.next(5) == 0 {
            if let Some(t) = world.tile_mut(x, y) {
                t.kind = 83;
            }
        }
    }
}

/// `UpdateWorld_OvergroundTile`'s port slice (`:72030+`): the herb branch.
/// The other arms (the trees, the cacti, the beach shells, the bamboo) land
/// with M7-GROWTH-2.
/// `WorldGen.IsTileTypeFitForTree` (`:29695-29710`).
pub const TREE_SOILS: [u16; 12] = [2, 23, 60, 70, 109, 147, 199, 477, 492, 633, 661, 662];

/// `Main`'s weather/seed state `IsAlchemyPlantHarvestable` reads
/// (`WorldGen.cs:66148-66175`) beyond the `World`'s saved fields: the
/// app's weather clock passes the live values.
#[derive(Clone, Copy, Default)]
pub struct WeatherCtx {
    /// `Main.raining`.
    pub raining: bool,
    /// `Main.cloudAlpha`.
    pub cloud_alpha: f32,
    /// `Main.remixWorld`.
    pub remix_world: bool,
}

/// `WorldGen.IsAlchemyPlantHarvestable` (`WorldGen.cs:66148-66175`): the
/// per-style condition an herb may be harvested with a seed in hand —
/// style 0 daybloom (day), 1 moonglow (night), 3 deathweed (night +
/// blood moon / full moon), 4 waterleaf (rain or any cloud), 5 fireblossom
/// (dry or below the surface, late afternoon).
pub fn is_alchemy_plant_harvestable(world: &World, style: i32, y: i32, wx: &WeatherCtx) -> bool {
    if style == 0 && world.day_time {
        return true;
    }
    if style == 1 && !world.day_time {
        return true;
    }
    if style == 3 && !world.day_time && (world.blood_moon || world.moon_phase == 0) {
        return true;
    }
    if style == 4 && (wx.raining || wx.cloud_alpha > 0.0) {
        return true;
    }
    if style == 5 {
        let mut flag = y as f64 > world.world_surface;
        if wx.remix_world {
            flag = y < world.height - 350;
        }
        // The C# `(!Main.raining | flag)` is the bitwise bool `|` — both
        // operands are pure, so `||` is equivalent.
        if ((!wx.raining || flag) && world.time > 40500.0) {
            return true;
        }
    }
    false
}

/// `WorldGen.IsHarvestableHerbWithSeed` (`WorldGen.cs:66118-66138`): only
/// the grown herbs 83/84 carry the seed-harvest arm — 84 always is, and
/// 83 when its style's condition holds (the plain herb 82 never is).
pub fn is_harvestable_herb_with_seed(
    world: &World,
    kind: u16,
    style: i32,
    y: i32,
    wx: &WeatherCtx,
) -> bool {
    match kind {
        82 => false,
        83 | 84 => kind == 84 || is_alchemy_plant_harvestable(world, style, y, wx),
        _ => false,
    }
}

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
    rng: &mut FastRandom,
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
pub fn grow_tree(world: &mut World, i: i32, y: i32, rng: &mut FastRandom) -> bool {
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
    let num2 = 5 + rng.next(12);
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
            rng.next(10) as i16
        };
        while ((num6 == 5 || num6 == 7) && left_last)
            || ((num6 == 6 || num6 == 7) && right_last)
        {
            num6 = rng.next(10) as i16;
        }
        let style = rng.next(3) as i16;
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
            markers.push((i - 1, by, 44, 198 + rng.next(3) as i16 * 22));
        }
        if right_last {
            markers.push((i + 1, by, 66, 198 + rng.next(3) as i16 * 22));
        }
    }
    rows[height - 1] = (22, 198 + rng.next(3) as i16 * 22);
    // The forks: the base style follows the grass neighbours (:29980+).
    let grass_at = |dx: i32| {
        world
            .tile(i + dx, top)
            .is_some_and(|t| t.active() && matches!(t.kind, 2 | 23 | 60 | 109))
    };
    let left_grass = grass_at(-1);
    let right_grass = grass_at(1);
    let mut base_style = rng.next(3);
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
        let fy = 132 + rng.next(3) as i16 * 22;
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
            let fy = 132 + rng.next(3) as i16 * 22;
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

/// `WorldGen.GrowCactus` (:56575-56815). The water box
/// (`cactusWaterWidth=50/Height=25/Limit=25`, :86930-86932) aborts; the
/// sand arm plants a fresh cactus (the neighbours clear, ≤3 cacti in the
/// i±6 × j-3..j+1 box, >10 sand in it); the cactus arm grows the stack by
/// one with the 11-height budget and the `Next(11,13)` cap (the fork/lean
/// arms ride the same rolls).
pub fn grow_cactus(world: &mut World, i: i32, j: i32, rng: &mut FastRandom) {
    if j <= 0 {
        return;
    }
    let Some(tile) = world.tile(i, j) else {
        return;
    };
    if !tile.active() || tile.half_brick() || tile.slope() != 0 {
        return;
    }
    let is_sand = matches!(tile.kind, 53 | 234 | 116 | 112)
        || tile.kind == 234
        || tile.kind == 116
        || tile.kind == 112;
    if tile.kind != 80 && !is_sand {
        return;
    }
    if world.tile(i, j - 1).is_some_and(|t| t.liquid > 0) {
        return;
    }
    // The water box abort (:56588-56595).
    let mut num3 = 0u32;
    for k in (i - 50)..(i + 50) {
        for l in (j - 25)..(j + 25) {
            if let Some(t) = world.tile(k, l) {
                num3 += t.liquid as u32;
            }
        }
    }
    if num3 / 255 > 25 {
        return;
    }
    if is_sand {
        // The sand arm (:56598-56660): the neighbours above must be clear,
        // the box scan gates, then the plant.
        if world.tile(i, j - 1).is_some_and(|t| t.active())
            || world.tile(i - 1, j - 1).is_some_and(|t| t.active())
            || world.tile(i + 1, j - 1).is_some_and(|t| t.active())
        {
            return;
        }
        let mut num4 = 0;
        let mut num5 = 0;
        for m in (i - 6)..=(i + 6) {
            for n in (j - 3)..=(j + 1) {
                if let Some(t) = world.tile(m, n) {
                    if t.active() {
                        if t.kind == 80 {
                            num4 += 1;
                        }
                        if matches!(t.kind, 53 | 234 | 116 | 112)
                            || matches!(t.kind, 234 | 116 | 112)
                        {
                            num5 += 1;
                        }
                    }
                }
            }
        }
        if num4 >= 4 || num5 <= 10 {
            return;
        }
        let idx = world.index(i, j - 1);
        world.tiles[idx] = crate::tile::Tile {
            kind: 80,
            frame_x: 0,
            frame_y: 0,
            flags: FLAG_ACTIVE,
            ..Default::default()
        };
        if world.frame_important.len() <= 80 {
            world.frame_important.resize(crate::tiledata::TILE_COUNT, false);
        }
        world.frame_important[80] = true;
        return;
    }
    // The cactus arm (:56668-56799): the stack walk DOWN from the top with the
    // lean corrections, the height/budget, the ±2-column density cap, and the
    // fork — the side shoots (left/right) and the tip.
    if world.tile(i, j).is_none_or(|t| t.kind != 80) {
        return;
    }
    let mut num = i;
    let mut num2 = j;
    while world
        .tile(num, num2)
        .is_some_and(|t| t.active() && t.kind == 80)
    {
        num2 += 1;
        if !world
            .tile(num, num2)
            .is_some_and(|t| t.active() && t.kind == 80)
        {
            if world
                .tile(num - 1, num2)
                .is_some_and(|t| t.active() && t.kind == 80)
                && world
                    .tile(num - 1, num2 - 1)
                    .is_some_and(|t| t.active() && t.kind == 80)
                && num >= i
            {
                num -= 1;
            }
            if world
                .tile(num + 1, num2)
                .is_some_and(|t| t.active() && t.kind == 80)
                && world
                    .tile(num + 1, num2 - 1)
                    .is_some_and(|t| t.active() && t.kind == 80)
                && num <= i
            {
                num += 1;
            }
        }
    }
    let num6 = num2 - 1 - j;
    let num7 = i - num;
    num = i - num7;
    num2 = j;
    let num8 = 11 - num6;
    let mut num9 = 0;
    for num10 in (num - 2)..=(num + 2) {
        for num11 in (num2 - num8)..=(num2 + num6) {
            if let Some(t) = world.tile(num10, num11) {
                if t.active() && t.kind == 80 {
                    num9 += 1;
                }
            }
        }
    }
    // The drunk arm's cap is the 11-20 range (`:56690-56698`); the classic
    // worlds take the 11-13 cap the port already carried.
    if num9 >= 11 + rng.next(2) {
        return;
    }
    num = i;
    num2 = j;
    if num7 != 0 {
        // The lean arm (:56788-56797): a free cell above with the neighbour
        // shoot standing — the cactus leans into it.
        if !world.tile(num, num2 - 1).is_some_and(|t| t.active())
            && !world.tile(num, num2 - 2).is_some_and(|t| t.active())
            && !world.tile(num + num7, num2 - 1).is_some_and(|t| t.active())
            && world
                .tile(num - num7, num2 - 1)
                .is_some_and(|t| t.active() && t.kind == 80)
        {
            let idx = world.index(num, num2 - 1);
            world.tiles[idx] = crate::tile::Tile {
                kind: 80,
                frame_x: 0,
                frame_y: 0,
                flags: FLAG_ACTIVE,
                ..Default::default()
            };
        }
        return;
    }
    if num6 == 0 {
        if world.tile(num, num2 - 1).is_none_or(|t| !t.active()) {
            let idx = world.index(num, num2 - 1);
            world.tiles[idx] = crate::tile::Tile {
                kind: 80,
                frame_x: 0,
                frame_y: 0,
                flags: FLAG_ACTIVE,
                ..Default::default()
            };
        }
        return;
    }
    // The fork arms (:56739-56783): the side shoots and the tip.
    let mut flag = false;
    let mut flag2 = false;
    if world
        .tile(num, num2 - 1)
        .is_some_and(|t| t.active() && t.kind == 80)
    {
        if !world.tile(num - 1, num2).is_some_and(|t| t.active())
            && !world.tile(num - 2, num2 + 1).is_some_and(|t| t.active())
            && !world.tile(num - 1, num2 - 1).is_some_and(|t| t.active())
            && !world.tile(num - 1, num2 + 1).is_some_and(|t| t.active())
            && !world.tile(num - 2, num2).is_some_and(|t| t.active())
        {
            flag = true;
        }
        if !world.tile(num + 1, num2).is_some_and(|t| t.active())
            && !world.tile(num + 2, num2 + 1).is_some_and(|t| t.active())
            && !world.tile(num + 1, num2 - 1).is_some_and(|t| t.active())
            && !world.tile(num + 1, num2 + 1).is_some_and(|t| t.active())
            && !world.tile(num + 2, num2).is_some_and(|t| t.active())
        {
            flag2 = true;
        }
    }
    let num12 = rng.next(3);
    if num12 == 0 && flag {
        let idx = world.index(num - 1, num2);
        world.tiles[idx] = crate::tile::Tile {
            kind: 80,
            frame_x: 0,
            frame_y: 0,
            flags: FLAG_ACTIVE,
            ..Default::default()
        };
    } else if num12 == 1 && flag2 {
        let idx = world.index(num + 1, num2);
        world.tiles[idx] = crate::tile::Tile {
            kind: 80,
            frame_x: 0,
            frame_y: 0,
            flags: FLAG_ACTIVE,
            ..Default::default()
        };
    } else if num6 < rng.next_range(2, 8)
        && world
            .tile(num - 1, num2 - 1)
            .is_none_or(|t| !t.active() || t.kind != 80)
        && world
            .tile(num + 1, num2 - 1)
            .is_none_or(|t| !t.active() || t.kind != 80)
        && world.tile(num, num2 - 1).is_none_or(|t| !t.active())
    {
        let idx = world.index(num, num2 - 1);
        world.tiles[idx] = crate::tile::Tile {
            kind: 80,
            frame_x: 0,
            frame_y: 0,
            flags: FLAG_ACTIVE,
            ..Default::default()
        };
    }
}

/// `WorldGen.UpdateWorld_GrassGrowth` (:74269-74424), the slice: the
/// per-type table (:74284-74356 - the infection rows gated by
/// `AllowedToSpreadInfections`, the jungle/mushroom/plant rows maxValue 10),
/// the plant-above roll (:74368-74371), the 3x3 soil scan converting the
/// neighbours via [`spread_grass`] (:74381-74404).
pub fn update_world_grass_growth(
    world: &mut World,
    i: i32,
    j: i32,
    min_i: i32,
    max_i: i32,
    min_j: i32,
    max_j: i32,
    allowed_to_spread: bool,
    rng: &mut FastRandom,
) {
    if i < 10 || j < 10 || i >= world.width as i32 - 10 || j >= world.height as i32 - 10 {
        return;
    }
    let Some(t) = world.tile(i, j) else {
        return;
    };
    let (soil, soil2, grass, grass2, plant, max_value): (i16, i16, i16, i16, i16, i32) = match t.kind {
        2 => (0, -1, 2, -1, -1, 1),
        23 => (0, 59, 23, 661, 24, 2),
        199 => (0, 59, 199, 662, 201, 2),
        661 => (59, 0, 661, 23, 24, 2),
        662 => (59, 0, 662, 199, 201, 2),
        60 => (59, -1, 60, -1, 61, 10),
        70 => (59, -1, 70, -1, 71, 10),
        633 => (57, -1, 633, -1, 637, 2),
        226 => (-1, -1, 226, -1, 61, 10),
        _ => return,
    };
    if matches!(t.kind, 23 | 199 | 661 | 662) && !allowed_to_spread {
        return;
    }
    // The plant-above roll (:74368-74371).
    if plant != -1
        && min_j < world.height as i32
        && world.tile(i, min_j).is_none_or(|t| !t.active())
        && rng.next(max_value) == 0
    {
        let idx = world.index(i, min_j);
        world.tiles[idx] = crate::tile::Tile {
            kind: plant as u16,
            flags: FLAG_ACTIVE,
            ..Default::default()
        };
        if world.frame_important.len() <= plant as usize {
            world.frame_important.resize(crate::tiledata::TILE_COUNT, false);
        }
        world.frame_important[plant as usize] = true;
    }
    // The 3x3 soil scan (:74381-74404).
    if soil != -1 {
        for k in min_i..max_i {
            for l in min_j..max_j {
                if k < 10
                    || l < 10
                    || k >= world.width as i32 - 10
                    || l >= world.height as i32 - 10
                {
                    continue;
                }
                if k == i && l == j {
                    continue;
                }
                let Some(n) = world.tile(k, l) else {
                    continue;
                };
                if !n.active() {
                    continue;
                }
                if n.kind == soil as u16 {
                    spread_grass(world, k, l, soil as u16, grass as u16);
                } else if soil2 > -1 && grass2 > -1 && n.kind == soil2 as u16 {
                    spread_grass(world, k, l, soil2 as u16, grass2 as u16);
                }
            }
        }
    }
}

/// `WorldGen.SpreadGrass` (:75146-75270), the slice: the guards (the dirt
/// type match, the surface rule `j >= worldSurface && dirt == 0`, the +-1
/// solid-neighbour scan with the lava exception, the flower-27 exclusion)
/// + the write. The infection Convert()/wall arms ride the next slice.
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
fn kill_invalid_tree_above(world: &mut World, i: i32, j: i32, new_floor: u16) {
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

pub fn spread_grass(world: &mut World, i: i32, j: i32, dirt: u16, grass: u16) {
    let Some(t) = world.tile(i, j) else {
        return;
    };
    if !t.active() || t.kind != dirt {
        return;
    }
    // The surface rule (:75166-75169).
    if dirt == 0 && j as f64 >= world.world_surface {
        return;
    }
    // The +-1 enclosure scan with the lava exception (:75186-75201): the C#'s
    // `flag` starts true, ANY non-solid tile in the 3x3 clears it, and a lava
    // tile re-arms it and breaks the scan. The spread proceeds only while
    // `flag` is cleared — a fully-solid 3x3 refuses. (The earlier slice had
    // this inverted — it refused when the box was all-empty instead of
    // all-solid, so the growth tick refused exactly the conversions the C#
    // allows. `worldgen/biomes.rs`'s `spread_jungle_grass` carries the C#
    // semantics already.)
    let mut enclosed = true;
    'enclosure: for k in (i - 1)..=(i + 1) {
        for l in (j - 1)..=(j + 1) {
            let Some(n) = world.tile(k, l) else {
                continue;
            };
            if !(n.active() && crate::tiledata::TILE_SOLID.has(n.kind)) {
                enclosed = false;
            }
            if n.liquid > 0 && n.liquid_kind() == crate::tile::LiquidKind::Lava {
                enclosed = true;
                break 'enclosure;
            }
        }
    }
    if enclosed {
        return;
    }
    // The flower-27 exclusions (:75204-75206).
    if matches!(grass, 23 | 661 | 199 | 662 | 109)
        && world.tile(i, j - 1).is_some_and(|t| t.kind == 27)
    {
        return;
    }
    // `flag2 = dirt > 0 && TileID.Sets.Conversion.Grass[dirt]` (`:75207`) and
    // the conversion dispatch (`:75208-75221`): spreading a conversion grass
    // over a conversion-grass dirt converts the cell through the flavor's
    // map instead of the plain type write — corruption for 23/661, crimson
    // for 199/662, hallow for 109. Everything else takes the plain write,
    // preceded by the tree-invalidity kill.
    let conversion_dirt = matches!(dirt, 2 | 23 | 199 | 109 | 477 | 492);
    if conversion_dirt && matches!(grass, 23 | 661) {
        crate::spread::convert_cell_flavor(
            world,
            i,
            j,
            crate::spread::ConversionFlavor::Corruption,
            false,
        );
    } else if conversion_dirt && matches!(grass, 199 | 662) {
        crate::spread::convert_cell_flavor(
            world,
            i,
            j,
            crate::spread::ConversionFlavor::Crimson,
            false,
        );
    } else if conversion_dirt && grass == 109 {
        crate::spread::convert_cell_flavor(
            world,
            i,
            j,
            crate::spread::ConversionFlavor::Hallow,
            false,
        );
    } else {
        kill_invalid_tree_above(world, i, j, grass);
        let idx = world.index(i, j);
        world.tiles[idx] = crate::tile::Tile {
            kind: grass,
            flags: FLAG_ACTIVE,
            ..Default::default()
        };
        if world.frame_important.len() <= grass as usize {
            world.frame_important.resize(crate::tiledata::TILE_COUNT, false);
        }
        world.frame_important[grass as usize] = true;
    }
}

/// `WorldGen.SolidTile` (`:70111-70131`) — the growth module's own copy of the
/// tile-only solidity test (the shared `worldgen` module is crate-private).
fn growth_solid(world: &World, x: i32, y: i32) -> bool {
    match world.tile(x, y) {
        Some(t) => {
            t.active()
                && (t.kind as usize) < crate::tiledata::TILE_COUNT
                && crate::tiledata::TILE_SOLID.has(t.kind)
                && !crate::tiledata::TILE_SOLID_TOP.has(t.kind)
                && !t.half_brick()
                && t.slope() == 0
        }
        None => false,
    }
}

/// `WorldGen.GetWaterDepth` (`:58355-58373`): the liquid column above the
/// floor under `(x, y)` — the solid walk down, then the liquid count back up.
fn water_depth(world: &World, x: i32, y: i32) -> i32 {
    let mut num = y;
    while !growth_solid(world, x, num) {
        num += 1;
        if num > world.height - 1 {
            return 0;
        }
    }
    num -= 1;
    let mut num2 = num;
    while world.tile(x, num2).is_some_and(|t| t.liquid > 0)
        && !growth_solid(world, x, num2)
    {
        num2 -= 1;
    }
    num - num2
}

/// `WorldGen.CountGrowingPlantTiles` (`:58375-58390`): the tiles of `kind` in
/// the `[x-range, x+range] × [y-range*3, y+range*3]` box.
fn count_growing_plant_tiles(world: &World, x: i32, y: i32, range: i32, kind: u16) -> i32 {
    let mut num = 0;
    for i in (x - range)..=(x + range) {
        for j in (y - range * 3)..=(y + range * 3) {
            if world.tile(i, j).is_some_and(|t| t.active() && t.kind == kind) {
                num += 1;
            }
        }
    }
    num
}

/// `WorldGen.PlaceBamboo` (`:58391-58445`): one bamboo stalk on the jungle
/// grass / bamboo under `(x, y)` — the wall/surface guard, the water-depth
/// window (2..5), the density gates and the write. Returns whether a stalk
/// landed.
fn place_bamboo(world: &mut World, x: i32, y: i32, rng: &mut FastRandom) -> bool {
    let min_depth = 2;
    let max_depth = 5;
    let num3 = rng.next_range(1, 21);
    let Some(tile) = world.tile(x, y) else {
        return false;
    };
    if tile.wall > 0 && (y as f64) <= world.world_surface {
        return false;
    }
    if tile.active() && tile.kind == 314 {
        return false;
    }
    let below = world.tile(x, y + 1).map(|t| (t.active(), t.kind));
    let Some((below_active, below_kind)) = below else {
        return false;
    };
    if below_active && matches!(below_kind, 571 | 60) {
        let depth = water_depth(world, x, y);
        if depth < min_depth || depth > max_depth {
            return false;
        }
        let mut num4 = count_growing_plant_tiles(world, x, y, 5, 571);
        let mut i = 1;
        if below_kind == 571 {
            loop {
                if growth_solid(world, x, y + i) {
                    break;
                }
                i += 1;
            }
            if i + num4 / rng.next_range(1, 21) > num3 {
                return false;
            }
        } else {
            num4 += 25;
        }
        if num4 + i * 2 > rng.next_range(40, 61) {
            return false;
        }
        if let Some(t) = world.tile_mut(x, y) {
            t.kind = 571;
            t.set_slope(0);
            t.flags = FLAG_ACTIVE;
        }
        return true;
    }
    false
}

/// `TileID.Sets.IsVine` (`TileID.cs:531`) — the vine kinds the density count
/// reads.
const IS_VINE: &[u16] = &[52, 382, 62, 115, 205, 528, 636, 638];

/// `Collision.CanHitLine`'s LOS over the two tile centres: the port's own
/// Bresenham walk (the shared `smart_cursor` helper is module-private; the C#
/// call passes 1x1 boxes, so the walk runs centre-to-centre).
fn can_hit_line(world: &World, x1: i32, y1: i32, x2: i32, y2: i32) -> bool {
    let steep = (y2 - y1).abs() > (x2 - x1).abs();
    let (mut ax, mut ay, mut bx, mut by) = (x1, y1, x2, y2);
    if steep {
        std::mem::swap(&mut ax, &mut ay);
        std::mem::swap(&mut bx, &mut by);
    }
    let dx = (bx - ax).abs();
    let dy = (by - ay).abs();
    let sx = if ax < bx { 1 } else { -1 };
    let sy = if ay < by { 1 } else { -1 };
    let mut err = dx / 2;
    let mut i = ax;
    loop {
        let (px, py) = if steep { (ay, i) } else { (i, ay) };
        if world
            .tile(px, py)
            .is_some_and(|t| t.active() && growth_solid_kind(t.kind))
        {
            return false;
        }
        if i == bx {
            break;
        }
        err -= dy;
        if err < 0 {
            ay += sy;
            err += dx;
        }
        i += sx;
    }
    true
}

fn growth_solid_kind(kind: u16) -> bool {
    (kind as usize) < crate::tiledata::TILE_COUNT
        && crate::tiledata::TILE_SOLID.has(kind)
        && !crate::tiledata::TILE_SOLID_TOP.has(kind)
}

/// `WorldGen.GrowMoreVines` (`:45586-45621`): the vine-density cap around
/// `(x, y)` — the ±4/±6/±10 box's vine count, the LOS-weighted accumulation
/// below the source and the 60 budget (÷5 on the mushroom vine 528).
fn grow_more_vines(world: &World, x: i32, y: i32) -> bool {
    if x < 30 || y < 30 || x >= world.width - 30 || y >= world.height - 30 {
        return false;
    }
    let (lo_x, lo_y, hi_y, mut num4) = (4, 6, 10, 60);
    if world.tile(x, y).is_some_and(|t| t.kind == 528) {
        num4 /= 5;
    }
    let mut num5 = 0;
    for i in (x - lo_x)..=(x + lo_x) {
        for j in (y - lo_y)..=(y + hi_y) {
            if world.tile(i, j).is_some_and(|t| t.active() && IS_VINE.contains(&t.kind)) {
                num5 += 1;
                if j > y && can_hit_line(world, x, y, i, j) {
                    let weight = if world.tile(i, j).is_some_and(|t| t.kind == 528) {
                        20
                    } else {
                        2
                    };
                    num5 += (j - y) * weight;
                }
                if num5 > num4 {
                    return false;
                }
            }
        }
    }
    true
}

/// `UpdateWorld_UndergroundTile`'s vine arms (`:73310-73470`): the jungle,
/// mushroom, corrupt, crimson and ash families grow their vine one row below
/// at the vanilla odds (the 1/5 jungle arm, the 70/7 odds for the rest,
/// 7 when the tile is already the vine kind), behind the free-below gate, the
/// 10-row look-up through the family grass and `GrowMoreVines`' density.
pub(crate) fn grow_vine_arm(world: &mut World, i: i32, j: i32, rng: &mut FastRandom) {
    let kind = world.tile(i, j).map(|t| (t.active(), t.kind, t.bottom_slope()));
    let Some((active, kind, bottom_slope)) = kind else {
        return;
    };
    if !active || bottom_slope {
        return;
    }
    // The free-below gate: the cell below the source must be inactive and dry
    // (the C#'s `!active() && !anyLava()` `:73313`).
    let below_clear = world
        .tile(i, j + 1)
        .is_some_and(|t| !t.active() && t.liquid == 0);
    if !below_clear {
        return;
    }
    // `(grass, vine, odds, gate)` per family (`:73311-73499`).
    let (grass, vine, odds, gated) = match kind {
        60 | 62 => (60, 62, 5, true),
        633 | 638 => (633, 638, 70, false),
        70 | 528 => (70, 528, 70, false),
        661 | 636 => (661, 636, 70, false),
        662 | 205 => (662, 205, 70, false),
        _ => return,
    };
    let odds = if kind == vine {
        if gated {
            odds
        } else {
            7
        }
    } else {
        odds
    };
    if rng.next(odds) != 0 {
        return;
    }
    if !grow_more_vines(world, i, j) {
        return;
    }
    // The 10-row look-up: the family grass below (a bottom slope aborts).
    let mut flag = false;
    let mut n = j;
    while n > j - 10 {
        let t = world.tile(i, n);
        if t.is_some_and(|t| t.bottom_slope()) {
            flag = false;
            break;
        }
        if t.is_some_and(|t| t.active() && t.kind == grass && !t.bottom_slope()) {
            flag = true;
            break;
        }
        n -= 1;
    }
    if !flag {
        return;
    }
    if let Some(t) = world.tile_mut(i, j + 1) {
        t.kind = vine;
        t.flags = FLAG_ACTIVE;
        t.liquid = 0;
    }
    if world.frame_important.len() <= vine as usize {
        world.frame_important.resize(crate::tiledata::TILE_COUNT, false);
    }
    world.frame_important[vine as usize] = true;
}

/// `WorldGen.GrowSpike` (:45481-45570): the thorn bushes on the evil/jungle
/// grass. The 4-neighbour density gate, the Next(4) directional walk, the
/// landing cell empty with <=2 spike neighbours and a spike within the
/// diamond radius 7 standing on the land type, then the plant.
pub fn grow_spike(
    world: &mut World,
    i: i32,
    j: i32,
    spike_type: u16,
    land_type: u16,
    rng: &mut FastRandom,
) {
    let converted = match land_type {
        23 => 661,
        199 => 662,
        _ => 0,
    };
    let spike_neighbours = |x: i32, y: i32| -> i32 {
        [(1i32, 0), (-1, 0), (0, 1), (0, -1)]
            .iter()
            .filter(|&&(dx, dy)| {
                world
                    .tile(x + dx, y + dy)
                    .is_some_and(|t| t.active() && t.kind == spike_type)
            })
            .count() as i32
    };
    let Some(t) = world.tile(i, j) else {
        return;
    };
    let num4 = spike_neighbours(i, j);
    if num4 >= 3 && t.kind != land_type && t.kind != converted {
        return;
    }
    let (num2, num3) = match rng.next(4) {
        0 => (i, j - 1),
        1 => (i, j + 1),
        2 => (i - 1, j),
        _ => (i + 1, j),
    };
    if world.tile(num2, num3).is_some_and(|t| t.active()) {
        return;
    }
    if spike_neighbours(num2, num3) >= 2 {
        return;
    }
    let num5 = 7;
    let mut flag = false;
    'outer: for k in (num2 - num5)..(num2 + num5) {
        for l in (num3 - num5)..(num3 + num5) {
            if (k - num2).abs() * 2 + (l - num3).abs() < 9
                && world.tile(k, l).is_some_and(|t| {
                    t.active() && (t.kind == land_type || t.kind == converted)
                })
                && world
                    .tile(k, l - 1)
                    .is_some_and(|t| t.active() && t.kind == spike_type && t.liquid == 0)
            {
                flag = true;
                break 'outer;
            }
        }
    }
    if !flag {
        return;
    }
    let idx = world.index(num2, num3);
    world.tiles[idx] = crate::tile::Tile {
        kind: spike_type,
        flags: FLAG_ACTIVE,
        ..Default::default()
    };
    if world.frame_important.len() <= spike_type as usize {
        world.frame_important.resize(crate::tiledata::TILE_COUNT, false);
    }
    world.frame_important[spike_type as usize] = true;
}

pub fn update_world_overground_tile(
    world: &mut World,
    x: i32,
    y: i32,
    day_time: bool,
    raining: bool,
    max_raining: f32,
    wind_speed: f32,
    rng: &mut FastRandom,
) {
    let kind = world.tile(x, y).map(|t| (t.active(), t.kind));
    let Some((active, kind)) = kind else {
        return;
    };
    if !active {
        // The beach-shells arm (`:72073`): an inactive beach cell under the
        // surface grows a shell pile behind the wind/rain odds.
        grow_shell_pile(world, x, y, wind_speed, max_raining, raining, rng);
        return;
    }
    if TILE_ALCH.contains(&kind) {
        grow_alch(world, x, y, day_time, raining, rng);
        return;
    }
    // The sapling arm (:72430-72434): the kind-20 sapling at `Next(20)`,
    // the beach palm saplings 595/615 at `Next(5)` (the settings profiles
    // land with the vanity trees).
    if kind == 20 && rng.next(20) == 0 {
        attempt_grow_tree_from_sapling(world, x, y, false, rng);
        return;
    }
    if (kind == 595 || kind == 615) && rng.next(5) == 0 {
        // `AttemptToGrowTreeFromSapling`'s 595/615 arms (`:72435-72444`)
        // — the Sakura/Willow settings profiles.
        attempt_grow_tree_from_sapling(world, x, y, false, rng);
        return;
    }
    // The vanilla/KillTile arm (:72146-72152): the sakura/willow sapling or
    // its grown tile standing on the evil grass is killed (the trees do not
    // take root in the corruption/crimson).
    if matches!(kind, 595 | 615 | 596 | 616)
        && world
            .tile(x, y + 1)
            .is_some_and(|t| t.active() && matches!(t.kind, 199 | 23))
    {
        if let Some(t) = world.tile_mut(x, y) {
            t.flags &= !FLAG_ACTIVE;
        }
        return;
    }
    // The bamboo arm (:72154-72156): the bamboo 571 or the jungle grass 60
    // with liquid above, at `Next(5)`, when the cell above is free and — for
    // the jungle grass — at `Next(30)`.
    let above_kind = world.tile(x, y - 1).map(|t| (t.active(), t.kind, t.liquid));
    let bamboo = match above_kind {
        Some((_, _, liquid)) if kind == 60 && liquid > 0 => true,
        Some(_) if kind == 571 => true,
        _ => false,
    };
    if bamboo && rng.next(5) == 0 {
        let above_free = world
            .tile(x, y - 1)
            .is_some_and(|t| !t.active() || matches!(t.kind, 61 | 74 | 518));
        let gate = if kind == 60 { rng.next(30) == 0 } else { true };
        if above_free && gate {
            place_bamboo(world, x, y - 1, rng);
        }
    }
    // The cactus arm (:72229-72231): `Next(15)`.
    if kind == 80 && rng.next(15) == 0 {
        grow_cactus(world, x, y, rng);
        return;
    }
    // The bare-sand arm (:72282-72284) rolls `Next(300)` inside the
    // beach-distance bounds — the port has no beachDistanceYet; the
    // water-box abort inside `grow_cactus` keeps the shore in check.
    if matches!(kind, 53 | 234 | 116 | 112) && rng.next(300) == 0 {
        grow_cactus(world, x, y, rng);
        return;
    }
    // The grass-growth pass (:72357 + :73282): the spread-overground grasses
    // with the plant windows (minI/maxI = i±1, minJ/maxJ = j±1 per :72350-
    // 72356's window).
    let grass_pass = matches!(
        kind,
        2 | 23 | 199 | 60 | 70 | 109 | 226 | 633 | 661 | 662
    );
    if grass_pass {
        update_world_grass_growth(
            world,
            x,
            y,
            x - 1,
            x + 2,
            y - 1,
            y + 2,
            true,
            rng,
        );
        // The thorn arms (:72362-72370): the jungle 32 at Next(3), the evil
        // 352 at Next(3), on their grass types.
        if (kind == 23 && rng.next(3) == 0) || (kind == 60 && rng.next(3) == 0) {
            grow_spike(world, x, y, 32, 23, rng);
        }
        if kind == 199 && rng.next(3) == 0 {
            grow_spike(world, x, y, 352, 199, rng);
        }
    }
}

/// The crystal-shard arm of `hardUpdateWorld` (`:69603-69640`): the shard
/// stones below the rock layer roll `Next(5)`, the 4-dir neighbour must be
/// empty, the radius-6 density stays under 2, the style `Next(18)` (the big
/// shard `18 + Next(6)` at 1/50).
pub fn grow_crystal_shards(world: &mut World, x: i32, y: i32, rng: &mut FastRandom) {
    let Some(t) = world.tile(x, y) else {
        return;
    };
    if !t.active() || !CAN_GROW_CRYSTAL_SHARDS.contains(&t.kind) {
        return;
    }
    // `((double)j > Main.rockLayer || remixWorld)`.
    if (y as f64) <= world.rock_layer {
        return;
    }
    if rng.next(5) != 0 {
        return;
    }
    // `:69609-69625`: the 4-dir pick — 0 left, 1 right, 2/3 down (the C#'s
    // `default: num3 = ((num != 0) ? 1 : -1)` resolves to +1 for both —
    // the shards never grow upward).
    let (dx, dy) = match rng.next(4) {
        0 => (-1, 0),
        1 => (1, 0),
        _ => (0, 1),
    };
    let (nx, ny) = (x + dx, y + dy);
    if world.tile(nx, ny).is_some_and(|t| t.active()) {
        return;
    }
    // `:69629-69638`: the radius-6 density scan for the existing shards
    // (tile 129).
    let mut count = 0;
    for k in (x - 6)..=(x + 6) {
        for l in (y - 6)..=(y + 6) {
            if world
                .tile(k, l)
                .is_some_and(|t| t.active() && t.kind == 129)
            {
                count += 1;
            }
        }
    }
    if count >= 2 {
        return;
    }
    // `:69639-69643`: the style — the big shards 18..=23 at 1/50.
    let mut style = rng.next(18);
    if rng.next(50) == 0 {
        style = 18 + rng.next(6);
    }
    if let Some(t) = world.tile_mut(nx, ny) {
        t.kind = 129;
        t.frame_x = (style * 18) as i16;
        t.flags |= FLAG_ACTIVE;
    }
}

/// `UpdateWorld`'s growth loops (`:71540-71632`): the overground
/// `3e-5 · rate` picks (×1.5 raining) with the alch-seed gate, then the
/// underground `1.5e-5 · rate` picks (the port's slice runs the same herb
/// pass — the underground arms are GROWTH-2). Returns the picks made (the
/// diagnostics).
pub fn update_world(
    world: &mut World,
    day_rate: i32,
    raining: bool,
    freeze_time: bool,
    day_time: bool,
    max_raining: f32,
    wind_speed: f32,
    rng: &mut FastRandom,
) -> u32 {
    let rate = world_update_rate(day_rate, freeze_time);
    if rate == 0 {
        return 0;
    }
    // `WorldGen.UpdateWorld`'s setup (`:71503`, `:71508`): the infection
    // spread is allowed (the creative-power override parks with M6-CREATIVE)
    // and the wall-arm box radius is the constant 3.
    let allowed_to_spread = true;
    let wall_dist = 3;
    let mut picks = 0u32;
    let mut overground_count =
        (world.width as f64 * world.height as f64) * 3e-5 * rate as f64;
    if raining {
        overground_count *= 1.5;
    }
    for _ in 0..overground_count as i32 {
        if alch_seed_skips(world, rng) {
            continue;
        }
        let x = rng.next_range(10, world.width - 10);
        let y = rng.next_range(10, world.world_surface as i32 - 1);
        update_world_overground_tile(
            world,
            x,
            y,
            day_time,
            raining,
            max_raining,
            wind_speed,
            rng,
        );
        picks += 1;
    }
    let underground_count = (world.width as f64 * world.height as f64) * 1.5e-5 * rate as f64;
    for _ in 0..underground_count as i32 {
        let x = rng.next_range(10, world.width - 10);
        let y = rng.next_range(world.world_surface as i32 - 1, world.height - 20);
        // The underground dispatcher (`UpdateWorld_UndergroundTile`
        // `:73214-73614`) replaces the overground parity call; the dyed/
        // moss/oyster arms inside it park (ANALYSIS).
        update_world_underground_tile(
            world,
            x,
            y,
            wall_dist,
            allowed_to_spread,
            day_time,
            raining,
            rng,
        );
        picks += 1;
    }
    picks
}

/// `WorldGen.beachDistance` = 380 (`WorldGen.cs:86914`) — the shell arm's
/// beach margins.
const BEACH_DISTANCE: i32 = 380;

/// `WorldGen.RollRandomSeaShellStyle` (`WorldGen.cs:45170-45187`) over the
/// growth `FastRandom` stream: the 5 seashell styles on tile 324 — `Next(2)`,
/// then `Next(10)==0 → 2`, `Next(10)==0 → 3`, `Next(50)==0 → 4` (nested `if`s,
/// so the later rolls re-roll earlier).
fn roll_sea_shell_style(rng: &mut FastRandom) -> i32 {
    let mut result = rng.next_range(0, 2);
    if rng.next(10) == 0 {
        result = 2;
    }
    if rng.next(10) == 0 {
        result = 3;
    }
    if rng.next(50) == 0 {
        result = 4;
    }
    result
}

/// The beach-shells arm of `UpdateWorld_OvergroundTile`
/// (`WorldGen.cs:72073-72135`): a dry empty cell in the beach margin under
/// the surface grows a shell pile — the calmer and drier the weather the
/// higher the odds (`num5 = 3000 - |wind| · 1250 - maxRaining · 1250`, floor
/// 300), then the sand scan, the pile-count cap `Next(2, 5)` with the
/// ±`Next(8, 11)` box check and the coral (81, on deep water) / seashell
/// (324, the rolled style) write.
pub fn grow_shell_pile(
    world: &mut World,
    i: i32,
    j: i32,
    wind_speed: f32,
    max_raining: f32,
    raining: bool,
    rng: &mut FastRandom,
) -> bool {
    let surface = world.world_surface;
    let width = world.width;
    // The gate: an inactive cell under the surface inside the beach margins
    // (`:72073`).
    if !((j as f64) < surface + 10.0) {
        return false;
    }
    if !(i < BEACH_DISTANCE || i > width - BEACH_DISTANCE) {
        return false;
    }
    if world.tile(i, j).is_some_and(|t| t.active()) {
        return false;
    }
    // The odds (`:72074-72080`).
    let mut num5 = 3000 - (wind_speed.abs() * 1250.0) as i32;
    if raining {
        num5 -= (1250.0 * max_raining) as i32;
    }
    if num5 < 300 {
        num5 = 300;
    }
    if rng.next(num5.max(1)) != 0 {
        return false;
    }
    // The sand scan (`:72083-72090`): down to the first solid within 15 rows.
    let mut k = j;
    while (k as f64) < surface + 10.0 && !world.tile(i, k).is_some_and(|t| t.active()) && k - j < 15
    {
        k += 1;
    }
    let sand_ok = world
        .tile(i, k)
        .is_some_and(|t| t.active() && t.kind == 53)
        && crate::edit::solid_tile_allow_bottom_slope(world, i, k);
    if !sand_ok {
        return false;
    }
    // The pile above the sand (`:72091-72096`): an existing pile stands down.
    if world.tile(i, k - 1).is_some_and(|t| t.active() && t.kind == 324) {
        return false;
    }
    // The density cap and the box count (`:72097-72110`).
    let num6 = rng.next_range(2, 5);
    let num7 = rng.next_range(8, 11);
    let mut num8 = 0;
    for l in (i - num7)..=(i + num7) {
        for m in (k - num7)..=(k + num7) {
            if world.tile(l, m).is_some_and(|t| t.active() && matches!(t.kind, 324 | 81)) {
                num8 += 1;
            }
        }
    }
    if num8 >= num6 {
        return false;
    }
    // The coral on the deep water, the seashell otherwise (`:72111-72130`).
    let coral = rng.next(2) == 0 && world.tile(i, k).is_some_and(|t| t.liquid >= 230);
    let style = if coral { 0 } else { roll_sea_shell_style(rng) };
    let idx = world.index(i, k - 1);
    let t = &mut world.tiles[idx];
    t.flags |= crate::tile::FLAG_ACTIVE;
    t.kind = if coral { 81 } else { 324 };
    if !coral {
        t.frame_x = (22 * style) as i16;
    }
    true
}

// --- M5-SEED-2: the `halloweenGen` / `endlessHalloween` pumpkins ------------

/// `WorldGen.PlacePumpkin` (`WorldGen.cs:52438-52488`): the 2x2 pumpkin
/// (tile 254) at `(x-1..x, superY-1..superY)`, the style row
/// `genRand.Next(6) * 36` in `frameY` and the 0/18 columns in `frameX`.
///
/// The style roll is drawn BEFORE the bounds guard (`:52440-52445`), so an
/// out-of-bounds call still spends it. The 2x2 must be clear (any active tile
/// except the 3/73/113/110 set — and 185 while its `frameY == 0` — blocks, as
/// does any liquid) and stand on solid grass/mushroom
/// ([`crate::edit::solid_tile`], types 2/109).
pub fn place_pumpkin(world: &mut World, x: i32, super_y: i32, rng: &mut FastRandom) {
    // `:52440-52441`: `ushort type = 254; int num = WorldGen.genRand.Next(6) * 36;`
    let num = rng.next(6) * 36;
    // `:52442-52445`: the bounds guard — after the roll.
    if x < 5
        || x > world.width as i32 - 5
        || super_y < 5
        || super_y > world.height as i32 - 5
    {
        return;
    }
    let mut flag = true;
    for i in (x - 1)..(x + 1) {
        for j in (super_y - 1)..(super_y + 1) {
            if let Some(t) = world.tile(i, j) {
                if t.active()
                    && !matches!(t.kind, 3 | 73 | 113 | 110)
                    && !(t.kind == 185 && t.frame_y == 0)
                {
                    flag = false;
                }
                if t.liquid > 0 {
                    flag = false;
                }
            }
        }
        // `:52464-52467`: the ground under BOTH columns.
        let ground = world.tile(i, super_y + 1).map(|t| t.kind);
        if !crate::edit::solid_tile(world, i, super_y + 1)
            || !matches!(ground, Some(2 | 109))
        {
            flag = false;
        }
    }
    if !flag {
        return;
    }
    // `:52471-52486` — the write: the two columns (`frameX` 0 / 18) over the
    // two rows (`frameY` = the style row, `+18` on the bottom half), anchored
    // at `(x-1, superY-1)`.
    for (dx, dy, frame_x, frame_y) in [
        (0, 0, 0i16, num as i16),
        (1, 0, 18, num as i16),
        (0, 1, 0, num as i16 + 18),
        (1, 1, 18, num as i16 + 18),
    ] {
        let idx = world.index(x - 1 + dx, super_y - 1 + dy);
        world.tiles[idx].kind = 254;
        world.tiles[idx].frame_x = frame_x;
        world.tiles[idx].frame_y = frame_y;
        world.tiles[idx].flags |= crate::tile::FLAG_ACTIVE;
    }
}

/// `WorldGen.GrowPumpkin` (`WorldGen.cs:52490-52552`): widen the pumpkin the
/// tile `(i, j)` belongs to by one 2-tile patch — every one of its 4 tiles
/// shifts `frameX += 36`, so the object moves one style column right in the
/// tilesheet. Only the leftmost 2x2 patch of a ≤4-patch pumpkin grows: the
/// frames must already spell a consistent 2x2 patch (`frameX ==
/// (k - num3) * 18 + num4 * 36`) standing on solid grass (types 2/477/492/109),
/// and a pumpkin already 4 patches wide returns.
///
/// `destroyObject` (`:52492-52495`) gates the KillTile-driven call, which the
/// port never routes here (the worldgen sweep is its only caller), so the port
/// omits the flag.
pub fn grow_pumpkin(world: &mut World, i: i32, j: i32) {
    let Some(t) = world.tile(i, j) else {
        return;
    };
    // `:52497-52502`: the patch's top row — `num2` (the tile's `frameY / 18`
    // folded down to 0/1 by the `-2` walk) is subtracted from `j`, so both a
    // top-row and a bottom-row tile resolve to the pumpkin's top row.
    let mut num2 = (t.frame_y / 18) as i32;
    while num2 > 1 {
        num2 -= 2;
    }
    let num = j - num2;
    // `:52503-52510`: the same fold across the columns gives the left edge
    // and the patch index (each patch is 36 px = 2 tiles wide).
    let mut num3 = (t.frame_x / 18) as i32;
    let mut num4 = 0i32;
    while num3 > 1 {
        num3 -= 2;
        num4 += 1;
    }
    let num3 = i - num3;
    let num5 = num4 * 36;
    // `:52512-52515`: the four-patch cap.
    if num4 >= 4 {
        return;
    }
    // `:52516-52533`: the 2x2 must be this patch of the pumpkin and the row
    // below it must be solid soil.
    let mut flag = false;
    for k in num3..(num3 + 2) {
        for l in num..(num + 2) {
            // `:52524`: the cell must be this patch of the pumpkin — the
            // `frameX` the patch index predicts (`num5` = 36 px per patch).
            let want_x = ((k - num3) * 18 + num5) as i16;
            let matches_patch = world
                .tile(k, l)
                .is_some_and(|t| t.active() && t.kind == 254 && t.frame_x == want_x);
            if !matches_patch {
                flag = true;
            }
        }
        let soil = world.tile(k, num + 2).map(|t| t.kind);
        // `:52529`: the row below must be solid soil.
        let solid = crate::edit::solid_tile(world, k, num + 2);
        if !solid || !matches!(soil, Some(2 | 477 | 492 | 109)) {
            flag = true;
        }
    }
    if flag {
        return;
    }
    // `:52534-52550`: the widen.
    for m in num3..(num3 + 2) {
        for n in num..(num + 2) {
            if let Some(t) = world.tile_mut(m, n) {
                if t.active() && t.kind == 254 {
                    t.frame_x += 36;
                }
            }
        }
    }
}

/// `GrassPlantsEvilPlantsAndPumpkinsOnSurface`'s holiday arm
/// (`WorldGen.cs:20067-20086`): the `SecretSeed.halloweenGen.Enabled ||
/// SecretSeed.endlessHalloween.Enabled` sweep over the surface grass
/// (`Main.tile[i, j].type == 2`, `j` from 50 while `j < Main.worldSurface`,
/// `i` from 40 to `maxTilesX - 40`). Each active grass cell rolls
/// `genRand.Next(15)`; a hit calls [`place_pumpkin`] at `(i, j - 1)` and then
/// [`grow_pumpkin`] `genRand.Next(5)` times.
///
/// Seam: the port's worldgen runs no `GrassPlantsEvilPlantsAndPumpkinsOnSurface`
/// step yet — the M5-SEED-2 pipeline hooks landed the Grass slot
/// (`worldgen/micro_grass.rs`, the `DoDigExtraHoles` host) and the
/// `FinalizeSecretSeeds` tail, not this pass — so this is the pass body the
/// step will call when it lands, flag-gated so a classic world draws nothing.
/// `rng` is the pass's `genRand` stream; the C# reads the worldgen's global
/// `WorldGen.genRand`, so the step supplies the pipeline's stream for this
/// pass name.
pub fn halloween_pumpkins(world: &mut World, rng: &mut FastRandom) {
    // `:20069`: `halloweenGen || endlessHalloween` (the two micro seeds share
    // the arm — `endlessHalloween` also sets `Main.forceHalloweenForever`,
    // `WorldGen.cs:572-574`).
    if !(world
        .micro_seeds
        .contains(crate::seeds::MicroSeedFlags::HALLOWEEN_GEN)
        || world
            .micro_seeds
            .contains(crate::seeds::MicroSeedFlags::ENDLESS_HALLOWEEN))
    {
        return;
    }
    let world_surface = world.world_surface;
    for i in 40..(world.width as i32 - 40) {
        // `:20075`: `for (int j = 50; (double)j < Main.worldSurface; j++)`.
        let mut j = 50;
        while (j as f64) < world_surface {
            // `:20076-20077`: the active grass gate, then the 1-in-15 roll.
            if world
                .tile(i, j)
                .is_some_and(|t| t.active() && t.kind == 2)
                && rng.next(15) == 0
            {
                // `:20078-20082`: the pumpkin plus `Next(5)` growth patches.
                place_pumpkin(world, i, j - 1, rng);
                let num = rng.next(5);
                for _ in 0..num {
                    grow_pumpkin(world, i, j - 1);
                }
            }
            j += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::seeds::MicroSeedFlags;
    /// A grass world for the pumpkin sweep (`WorldGen.cs:20067-20086`): one
    /// grass row (type 2, solid) at y 55 with air above it and
    /// `world_surface = 60`, so the sweep's `j in 50..worldSurface` band
    /// covers the row.
    fn pumpkin_world() -> World {
        let mut w = World::new("p".into(), 200, 120);
        for x in 0..200 {
            let i = w.index(x, 55);
            w.tiles[i] = Tile {
                kind: 2,
                ..Default::default()
            };
            w.tiles[i].flags |= FLAG_ACTIVE;
        }
        w.world_surface = 60.0;
        w
    }

    // M5-SEED-2: `halloweenGen` / `endlessHalloween`
    // (`WorldGen.cs:20069-20084`) — the surface pumpkin sweep and its two
    // helpers (`PlacePumpkin` `:52438-52488`, `GrowPumpkin` `:52490-52552`).
    #[test]
    fn place_pumpkin_lays_the_two_by_two_over_the_grass_row() {
        // `WorldGen.cs:52471-52486`: the 2x2 of tile 254 at
        // `(x-1..x, superY-1..superY)`, `frameX` 0/18 across the columns and
        // `frameY` = the `Next(6) * 36` style row (+18 on the bottom half).
        let mut w = pumpkin_world();
        let mut rng = FastRandom::new(3);
        place_pumpkin(&mut w, 60, 54, &mut rng);
        let top = w.tile(59, 53).unwrap();
        let right = w.tile(60, 53).unwrap();
        let bot_left = w.tile(59, 54).unwrap();
        let bot_right = w.tile(60, 54).unwrap();
        assert_eq!((top.kind, top.frame_x, top.frame_y), (254, 0, top.frame_y));
        assert_eq!(right.frame_x, 18);
        assert_eq!(bot_left.frame_x, 0);
        assert_eq!(bot_right.frame_x, 18);
        // The style row is one of the six 36-px rows, the bottom half 18 lower.
        assert_eq!(top.frame_y % 36, 0, "a 36-px style row");
        assert_eq!(bot_left.frame_y, top.frame_y + 18);
        assert_eq!(bot_right.frame_y, top.frame_y + 18);
        assert!(top.frame_y < 216, "six style rows: 0..180");
        // The roll is drawn even when the placement is refused (the C# draws
        // `Next(6)` before its bounds guard, `:52441-52445`).
        let mut r1 = FastRandom::new(9);
        place_pumpkin(&mut w, 1, 2, &mut r1); // out of bounds: the guard fires
        let mut r2 = FastRandom::new(9);
        let _ = r2.next(6);
        assert_eq!(r1.seed, r2.seed, "the style roll still spends its draw");
    }

    #[test]
    fn grow_pumpkin_widens_one_patch_and_caps_at_four() {
        // `WorldGen.cs:52534-52550`: each accepted call shifts the pumpkin's
        // four tiles `frameX += 36` (one 2-tile patch right in the tilesheet),
        // and `:52512-52515` refuses the fifth patch.
        let mut w = pumpkin_world();
        let mut rng = FastRandom::new(11);
        place_pumpkin(&mut w, 60, 54, &mut rng);
        // The call passes the grass column's own cell (the pumpkin's
        // bottom-right tile), like the C#'s `GrowPumpkin(i, j - 1, 254)`.
        grow_pumpkin(&mut w, 60, 54);
        assert_eq!(w.tile(59, 53).unwrap().frame_x, 36);
        assert_eq!(w.tile(60, 53).unwrap().frame_x, 54);
        assert_eq!(w.tile(59, 54).unwrap().frame_x, 36);
        assert_eq!(w.tile(60, 54).unwrap().frame_x, 54);
        // Three more accepted patches, then the cap: the frames stay put and
        // the tiles are still the pumpkin.
        for _ in 0..2 {
            grow_pumpkin(&mut w, 60, 54);
        }
        assert_eq!(w.tile(59, 53).unwrap().frame_x, 108);
        grow_pumpkin(&mut w, 60, 54);
        assert_eq!(w.tile(59, 53).unwrap().frame_x, 144, "the fourth patch");
        grow_pumpkin(&mut w, 60, 54);
        assert_eq!(w.tile(59, 53).unwrap().frame_x, 144, "the cap: no fifth");
        // The soil gate: dig the row out from under the pumpkin and the widen
        // refuses (`:52529`).
        let mut w = pumpkin_world();
        let mut rng = FastRandom::new(11);
        place_pumpkin(&mut w, 60, 54, &mut rng);
        let i = w.index(59, 55);
        w.tiles[i].flags &= !FLAG_ACTIVE;
        grow_pumpkin(&mut w, 60, 54);
        assert_eq!(w.tile(59, 53).unwrap().frame_x, 0, "no soil, no widen");
    }

    #[test]
    fn the_pumpkin_sweep_runs_only_under_the_holiday_flags() {
        // `WorldGen.cs:20069`: `halloweenGen || endlessHalloween` — the sweep
        // is the ONLY writer of tile 254 here, so its census is the flag's
        // observable, and a classic world draws nothing at all (the stream is
        // untouched).
        let mut classic = pumpkin_world();
        let mut rng = FastRandom::new(5);
        let before = rng.seed;
        halloween_pumpkins(&mut classic, &mut rng);
        assert_eq!(rng.seed, before, "a classic world spends no draw");
        assert!(classic.tiles.iter().all(|t| t.kind != 254), "no pumpkins");
        // The two seeds share the arm (`:20069`).
        for bit in [
            MicroSeedFlags::HALLOWEEN_GEN,
            MicroSeedFlags::ENDLESS_HALLOWEEN,
        ] {
            let mut w = pumpkin_world();
            w.micro_seeds.insert(bit);
            halloween_pumpkins(&mut w, &mut FastRandom::new(5));
            let planted = w.tiles.iter().filter(|t| t.kind == 254).count();
            assert!(planted >= 4, "the flag plants pumpkins: {planted}");
            // Every pumpkin stands directly on the grass row it was rolled
            // from: the top row's soil is 2 below it, the bottom row's 1.
            let mut checked = 0;
            for x in 40..160 {
                for y in 53..55 {
                    if let Some(t) = w.tile(x, y) {
                        if t.kind == 254 {
                            // The top row's `frameY / 18` is even (0/2/4/…),
                            // the bottom row's odd (1/3/5/…) — the fold
                            // `GrowPumpkin` runs (`:52499-52502`).
                            let soil_row = if (t.frame_y / 18) % 2 == 0 { y + 2 } else { y + 1 };
                            assert!(
                                w.tile(x, soil_row)
                                    .is_some_and(|s| s.active() && s.kind == 2),
                                "the pumpkin at ({x}, {y}) stands on grass"
                            );
                            checked += 1;
                        }
                    }
                }
            }
            assert!(checked > 0, "a pumpkin cell was inspected");
        }
    }

    use super::*;

    /// The underground vine arms (`:73310-73470`): the jungle grass with the
    /// vine below it grows the vine one row down at the 1/5 arm, behind the
    /// free-below gate, the 10-row look-up and `GrowMoreVines`' density.
    #[test]
    fn the_vine_arms_grow_below_their_grass() {
        // A jungle-grass column 30..32 with the free cell below (33), far
        // enough from the world edges for GrowMoreVines' InWorld(30).
        let mut w = World::new("t".into(), 200, 200);
        w.world_surface = 100.0;
        for y in 30..=32 {
            let i = w.index(100, y);
            w.tiles[i] = crate::tile::Tile {
                kind: 60,
                ..Default::default()
            };
            w.tiles[i].flags |= FLAG_ACTIVE;
        }
        let mut grew = 0;
        let mut runs = 0;
        for seed in 1u64..600 {
            let mut w = w.clone();
            let mut rng = FastRandom::new(seed);
            grow_vine_arm(&mut w, 100, 32, &mut rng);
            runs += 1;
            grew += w
                .tiles
                .iter()
                .filter(|t| t.active() && t.kind == 62)
                .count();
        }
        assert!(grew > 0, "the jungle arm grows the vine 62 ({grew} over {runs})");

        // The control: the same column with the vine already one below takes
        // the 70-odds arm of the family (the vine kind's own odds).
        let mut w = World::new("t".into(), 200, 200);
        w.world_surface = 100.0;
        for (y, kind) in [(30, 60), (31, 60), (32, 62), (33, 60)] {
            let i = w.index(100, y);
            w.tiles[i] = crate::tile::Tile {
                kind,
                ..Default::default()
            };
            w.tiles[i].flags |= FLAG_ACTIVE;
        }
        let mut grew2 = 0;
        for seed in 1u64..600 {
            let mut w = w.clone();
            let mut rng = FastRandom::new(seed);
            grow_vine_arm(&mut w, 100, 33, &mut rng);
            grew2 += w
                .tiles
                .iter()
                .filter(|t| t.active() && t.kind == 62)
                .count();
        }
        assert!(grew2 > 0, "the vine's own arm extends it ({grew2})");
    }

    /// The bamboo arm (`:72154-72156` + `PlaceBamboo` `:58391-58445`): a
    /// bamboo 571 with a free cell above grows a stalk at the vanilla gates —
    /// the wall/surface refusal, the water-depth window (2..5), the density
    /// caps and the write.
    #[test]
    fn the_bamboo_arm_grows_a_stalk_over_the_water() {
        // A pool of water over a floor: the bamboo 571 on the floor with a
        // water column 3 deep and the free cell above it.
        let mut w = World::new("t".into(), 60, 80);
        w.world_surface = 40.0;
        // The floor at y = 60.
        for x in 0..60 {
            let i = w.index(x, 60);
            w.tiles[i] = crate::tile::Tile {
                kind: 1,
                ..Default::default()
            };
            w.tiles[i].flags |= FLAG_ACTIVE;
        }
        // The water column y = 57..59 and the bamboo at y = 56.
        for x in 0..60 {
            for y in 57..=59 {
                let i = w.index(x, y);
                w.tiles[i].liquid = 255;
            }
            let i = w.index(30, 56);
            w.tiles[i] = crate::tile::Tile {
                kind: 571,
                ..Default::default()
            };
            w.tiles[i].flags |= FLAG_ACTIVE;
        }
        // Sweep seeds: the arm's Next(5)/Next(30) gates and PlaceBamboo's
        // own rolls land across enough streams.
        let mut grew = 0;
        let mut runs = 0;
        for seed in 1u64..600 {
            let mut w = w.clone();
            let mut rng = FastRandom::new(seed);
            update_world_overground_tile(&mut w, 30, 56, true, false, 0.0, 0.0, &mut rng);
            runs += 1;
            grew += w
                .tiles
                .iter()
                .filter(|t| t.active() && t.kind == 571)
                .count();
        }
        assert!(grew > 0, "the bamboo arm grows a stalk ({grew} over {runs} picks)");
    }

    /// The vanity-KillTile arm (`:72146-72152`): a sakura/willow sapling or
    /// its grown tile standing on the evil grass is killed, not left rooted.
    #[test]
    fn the_vanity_saplings_die_on_the_evil_grass() {
        let mut w = World::new("t".into(), 40, 60);
        w.world_surface = 40.0;
        // The evil grass at y = 30, the sapling on it at y = 29.
        let i = w.index(10, 30);
        w.tiles[i] = crate::tile::Tile {
            kind: 23,
            ..Default::default()
        };
        w.tiles[i].flags |= FLAG_ACTIVE;
        let i = w.index(10, 29);
        w.tiles[i] = crate::tile::Tile {
            kind: 595,
            ..Default::default()
        };
        w.tiles[i].flags |= FLAG_ACTIVE;
        let mut rng = FastRandom::new(5);
        update_world_overground_tile(&mut w, 10, 29, true, false, 0.0, 0.0, &mut rng);
        assert!(
            !w.tile(10, 29).is_some_and(|t| t.active()),
            "the sapling on the evil grass is killed"
        );
        // The control: on plain grass it survives the arm.
        let mut w = World::new("t".into(), 40, 60);
        w.world_surface = 40.0;
        let i = w.index(10, 30);
        w.tiles[i] = crate::tile::Tile {
            kind: 2,
            ..Default::default()
        };
        w.tiles[i].flags |= FLAG_ACTIVE;
        let i = w.index(10, 29);
        w.tiles[i] = crate::tile::Tile {
            kind: 595,
            ..Default::default()
        };
        w.tiles[i].flags |= FLAG_ACTIVE;
        let mut rng = FastRandom::new(5);
        update_world_overground_tile(&mut w, 10, 29, true, false, 0.0, 0.0, &mut rng);
        assert!(
            w.tile(10, 29).is_some_and(|t| t.active()),
            "the sapling on the plain grass survives"
        );
    }

    use super::*;

    /// `SpreadGrass`'s enclosure gate (`WorldGen.cs:75186-75201`): the C#'s
    /// `flag` starts true, any non-solid 3x3 tile clears it, and lava re-arms
    /// it. The spread proceeds when the box is OPEN (some non-solid tile) and
    /// refuses when it is fully solid — the earlier slice had this inverted.
    #[test]
    fn spread_grass_proceeds_on_an_open_box_and_refuses_when_enclosed() {
        // An open box: the dirt cell has air above it. The surface rule
        // (:75162) keeps dirt-0 below `worldSurface`, so the fixtures sit the
        // cell under it.
        let mut w = World::new("t".into(), 40, 40);
        w.world_surface = 35.0;
        let i = w.index(10, 20);
        w.tiles[i] = crate::tile::Tile {
            kind: 0,
            ..Default::default()
        };
        w.tiles[i].flags |= FLAG_ACTIVE;
        let mut rng = FastRandom::new(5);
        spread_grass(&mut w, 10, 20, 0, 2);
        assert!(w.tile(10, 20).is_some_and(|t| t.kind == 2), "the open box converted");

        // An enclosed box: every 3x3 neighbour solid.
        let mut w = World::new("t".into(), 40, 40);
        w.world_surface = 35.0;
        for dx in -1i32..=1 {
            for dy in -1i32..=1 {
                let i = w.index(10 + dx, 20 + dy);
                w.tiles[i] = crate::tile::Tile {
                    kind: 1,
                    ..Default::default()
                };
                w.tiles[i].flags |= FLAG_ACTIVE;
            }
        }
        let i = w.index(10, 20);
        w.tiles[i] = crate::tile::Tile {
            kind: 0,
            ..Default::default()
        };
        w.tiles[i].flags |= FLAG_ACTIVE;
        let mut rng = FastRandom::new(5);
        spread_grass(&mut w, 10, 20, 0, 2);
        assert!(
            w.tile(10, 20).is_some_and(|t| t.kind == 0),
            "the enclosed box refused"
        );

        // The lava exception: a lava tile in the box re-arms the flag and the
        // conversion refuses even though the box is open.
        let mut w = World::new("t".into(), 40, 40);
        w.world_surface = 35.0;
        let i = w.index(10, 20);
        w.tiles[i] = crate::tile::Tile {
            kind: 0,
            ..Default::default()
        };
        w.tiles[i].flags |= FLAG_ACTIVE;
        let i = w.index(11, 20);
        w.tiles[i] = crate::tile::Tile {
            kind: 0,
            liquid: 255,
            ..Default::default()
        };
        w.tiles[i].flags |= FLAG_ACTIVE;
        w.tiles[i].set_liquid_kind(crate::tile::LiquidKind::Lava);
        let mut rng = FastRandom::new(5);
        spread_grass(&mut w, 10, 20, 0, 2);
        assert!(
            w.tile(10, 20).is_some_and(|t| t.kind == 0),
            "the lava box refused"
        );
    }

    use super::*;
    use crate::tile::Tile;

    fn world() -> World {
        let mut w = World::new("t".into(), 4200, 400);
        w.rock_layer = 160.0;
        w
    }

    /// Sets a cell's wall (the wall-arm tests' natural/evil walls).
    fn set_wall(w: &mut World, x: i32, y: i32, wall: u16) {
        let i = w.index(x, y);
        w.tiles[i].wall = wall;
    }

    fn put(w: &mut World, x: i32, y: i32, kind: u16, frame_x: i16) {
        let i = w.index(x, y);
        w.tiles[i] = Tile {
            kind,
            frame_x,
            ..Default::default()
        };
        w.tiles[i].flags |= FLAG_ACTIVE;
    }

    #[test]
    fn the_rate_caps_and_freezes() {
        // `:72020-72030`: `min(dayRate, 24)`, the freeze → 0.
        assert_eq!(world_update_rate(1, false), 1);
        assert_eq!(world_update_rate(30, false), 24);
        assert_eq!(world_update_rate(30, true), 0);
    }

    #[test]
    fn the_alch_gate_uses_the_world_size_lerp() {
        // `:71606-71614`: `num7 = Lerp(151, 422.8, maxTilesX/4200 − 1)` —
        // the 4200-wide world sits at t = 0 → 151; a 8400-wide world at
        // t = 1 → 422.8. The gate skips on `Next(num7·100) == 0`.
        let w = world();
        let mut rng = FastRandom::new(5);
        let skips = (0..1000).filter(|_| alch_seed_skips(&w, &mut rng)).count();
        // The skip chance = 1/(151·100) ≈ 1/15100 → ~0.07/1000 rolls; the
        // 4200-world roll is nearly never skipped.
        assert!(skips <= 2, "skips {skips}");
    }

    #[test]
    fn the_herb_lifecycle_blooms_by_the_cues() {
        // The sprout 82 (the stage-0 frame) blooms in the daylight.
        let mut w = world();
        put(&mut w, 30, 90, 82, 0);
        let mut rng = FastRandom::new(1);
        let mut bloomed = false;
        for _ in 0..4000 {
            grow_alch(&mut w, 30, 90, true, false, &mut rng);
            if w.tile(30, 90).unwrap().kind == 83 {
                bloomed = true;
                break;
            }
        }
        assert!(bloomed, "the daylight cue blooms the stage-0 sprout");
        // The rain cue drives the stage-72 frame.
        let mut w = world();
        put(&mut w, 30, 90, 82, 72);
        let mut rng = FastRandom::new(2);
        let mut bloomed = false;
        for _ in 0..4000 {
            grow_alch(&mut w, 30, 90, false, true, &mut rng);
            if w.tile(30, 90).unwrap().kind == 83 {
                bloomed = true;
                break;
            }
        }
        assert!(bloomed, "the rain cue blooms the stage-72 frame");
        // The generic arm (`:45785-45800`): every frame except 108 blooms on
        // `Next(50) == 0` regardless of the cue — the cue arms are the
        // chain's fallbacks.
        let mut w = world();
        put(&mut w, 30, 90, 82, 0);
        let mut rng = FastRandom::new(3);
        let mut bloomed = false;
        for _ in 0..400 {
            grow_alch(&mut w, 30, 90, false, false, &mut rng);
            if w.tile(30, 90).unwrap().kind == 83 {
                bloomed = true;
                break;
            }
        }
        assert!(bloomed, "the generic 1/50 arm blooms any frame");
    }

    #[test]
    fn the_wrong_liquid_kills_the_herb() {
        let mut w = world();
        put(&mut w, 30, 90, 82, 0);
        let i = w.index(30, 90);
        w.tiles[i].liquid = 200;
        let mut rng = FastRandom::new(1);
        grow_alch(&mut w, 30, 90, true, false, &mut rng);
        assert!(
            w.tile(30, 90).unwrap().flags & FLAG_ACTIVE == 0,
            "the water on the stage-0 herb kills it"
        );
    }

    #[test]
    fn the_shards_grow_sparse_and_big_rarely() {
        let mut w = world();
        // The stone below the rock layer with an air pocket BELOW (the
        // shards grow down/sideways only).
        for x in 100..120 {
            for y in 200..210 {
                if !(x == 110 && y == 206) {
                    put(&mut w, x, y, 117, 0);
                }
            }
        }
        let mut rng = FastRandom::new(4);
        let mut grew = 0;
        for _ in 0..200 {
            let before = w
                .tiles
                .iter()
                .filter(|t| t.active() && t.kind == 129)
                .count();
            grow_crystal_shards(&mut w, 110, 205, &mut rng);
            let after = w
                .tiles
                .iter()
                .filter(|t| t.active() && t.kind == 129)
                .count();
            grew += (after - before) as i32;
        }
        assert!(grew > 0, "the shards grow on the stones");
        // The density cap: no 6-radius box holds 3 shards.
        let mut max_density = 0;
        for x in 100..120 {
            for y in 200..210 {
                let mut count = 0;
                for k in (x - 6)..=(x + 6) {
                    for l in (y - 6)..=(y + 6) {
                        if w.tile(k, l).is_some_and(|t| t.active() && t.kind == 129) {
                            count += 1;
                        }
                    }
                }
                max_density = max_density.max(count);
            }
        }
        assert!(max_density <= 2, "the density cap holds (max {max_density})");
    }

    #[test]
    fn test_spread_1_sapling_grows_a_tree_with_the_gates() {
        // `GrowTree` (:29767-29980): the sapling stack walk, the liquid
        // gate, the soil + neighbour checks, the clear box, then the trunk
        // plants over the sapling.
        let mut w = world();
        put(&mut w, 100, 200, 2, 0); // the grass
        put(&mut w, 101, 200, 2, 0); // the neighbour soil
        put(&mut w, 100, 199, 20, 0); // the sapling
        let mut rng = FastRandom::new(7);
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
        let mut rng = FastRandom::new(7);
        assert!(!attempt_grow_tree_from_sapling(&mut w, 100, 199, false, &mut rng));
    }




    #[test]
    fn the_underground_dispatcher_runs_the_passed_arms() {
        // `UpdateWorld_UndergroundTile`'s ported arms (`:73214-73614`): the
        // alch plant grows, the grass family converts (underground), the lily
        // becomes 74 and the grass growth re-runs the spikes. The parked arms
        // (the 655 kill, the dye/cattail/seaweed, the 444/396 moss/oyster, the
        // web/town-NPC) stay out.
        // The lily conversion: a lily (61) at Next(3)==0 over a non-226 tile
        // converts to 74.
        let mut w = world();
        put(&mut w, 100, 200, 61, 0);
        let mut grew = false;
        for seed in 1..64u64 {
            let mut rng = FastRandom::new(seed);
            update_world_underground_tile(&mut w, 100, 200, 3, true, true, false, &mut rng);
            if w.tile(100, 200).is_some_and(|t| t.kind == 74) {
                grew = true;
                break;
            }
        }
        assert!(grew, "the lily converts to 74 on some stream");

        // The grass family: the jungle grass 60 converts in the underground
        // direction (the conversion arm runs through the dispatcher).
        let mut w2 = world();
        put(&mut w2, 100, 200, 0, 0);
        put(&mut w2, 100, 201, 60, 0);
        put(&mut w2, 99, 201, 60, 0);
        let mut converted = false;
        for seed in 1..64u64 {
            let mut rng = FastRandom::new(seed);
            update_world_underground_tile(&mut w2, 100, 201, 3, true, true, false, &mut rng);
            if w2.tile(100, 200).is_some_and(|t| t.active()) {
                converted = true;
                break;
            }
        }
        assert!(converted, "the grass arm runs underground");
    }

    #[test]
    fn the_wall_arms_convert_the_neighbour_walls() {
        // `SpreadGrassWalls`/`SpreadDesertWalls` (`:74884`/`:74984`): an evil
        // cell with its source in the wallDist box converts a neighbouring
        // natural wall through Convert's wall chain; the gated arms stand
        // down without the source or with a wrong neighbour family.
        // The setup: a corrupt grass tile (23) beside a natural grass wall
        // (63) at ±2, the corruption source inside the box.
        let mut w = world();
        for dx in 0..10 {
            for dy in 200..205 {
                put(&mut w, 100 + dx, dy, 1, 0); // the stone floor
                set_wall(&mut w, 100 + dx, dy, 63); // the natural grass walls
            }
        }
        // The corrupt grass in the box (a `SpreadsCorruption` tile).
        put(&mut w, 102, 203, 23, 0);
        let mut rng = FastRandom::new(3);
        // The evil cell: the wall itself is a corruption-spreading wall (69)
        // at the source cell, so the arm fires from (102, 203).
        set_wall(&mut w, 102, 203, 69);
        let mut converted = false;
        for seed in 1..64u64 {
            let mut rng = FastRandom::new(seed);
            spread_grass_walls(&mut w, 3, 102, 203, &mut rng);
            // Any wall that left the natural family counts.
            converted = (100..110).any(|x| {
                (200..205).any(|y| w.tile(x, y).is_some_and(|t| t.wall != 63))
            });
            if converted {
                break;
            }
        }
        assert!(converted, "the corruption arm converts a natural wall");

        // Without the source in the box the arm stands down.
        let mut w2 = world();
        for dx in 0..10 {
            for dy in 200..205 {
                put(&mut w2, 100 + dx, dy, 1, 0);
                set_wall(&mut w2, 100 + dx, dy, 63);
            }
        }
        set_wall(&mut w2, 102, 203, 69); // the spreading wall, but no evil tile
        let mut rng2 = FastRandom::new(3);
        spread_grass_walls(&mut w2, 3, 102, 203, &mut rng2);
        assert!(
            (100..110).all(|x| (200..205).all(|y| {
                // The source cell keeps its own spreading wall by construction.
                (x, y) == (102, 203) || w2.tile(x, y).is_some_and(|t| t.wall == 63)
            })),
            "no source in the box - no conversion"
        );
    }


    #[test]
    fn the_beach_shells_arm_places_the_pile() {
        // The beach-shells arm (`:72073-72135`): a dry empty beach cell under
        // the surface grows a seashell (324) behind the calm/dry odds — the
        // gate first, then the wind/rain-adjusted `num5` roll, the sand scan,
        // the density cap and the write. With the odds at their easiest
        // (calm, dry: `num5 = 3000`) the 1-in-3000 roll still needs luck, so
        // the test seeds a hit through a tight loop and checks the contract.
        let mut w = world();
        w.world_surface = 100.0;
        // The beach margin: a sand floor at y = 101 near the world edge.
        for dx in -30i32..=30 {
            let i = w.index(5 + dx.max(0) as i32, 101);
            w.tiles[i] = crate::tile::Tile {
                kind: 53,
                flags: crate::tile::FLAG_ACTIVE,
                ..Default::default()
            };
        }
        // The pick sits above the sand, inside the margin (i < 380).
        let grew = (1..4096u64).any(|seed| {
            let mut rng = FastRandom::new(seed);
            grow_shell_pile(&mut w, 5, 100, 0.0, 0.0, false, &mut rng)
        });
        assert!(grew, "a calm dry day grows a shell pile on the beach");
        let pile = w.tile(5, 100).is_some_and(|t| t.active() && t.kind == 324)
            || w.tile(5, 100)
                .is_some_and(|t| t.active() && t.kind == 81);
        assert!(pile, "the pile is a seashell or a coral");
    }

    #[test]
    fn the_beach_shells_arm_refuses_the_gates() {
        // The arm's refusals: the weather odds (a storm makes `num5` bottom
        // at 300 but the roll still runs), a cell outside the beach margin,
        // and a cell with no sand below.
        let mut w = world();
        w.world_surface = 100.0;
        for dx in -30i32..=30 {
            let i = w.index(4000 + dx.max(0) as i32, 101);
            w.tiles[i] = crate::tile::Tile {
                kind: 53,
                flags: crate::tile::FLAG_ACTIVE,
                ..Default::default()
            };
        }
        let mut rng = FastRandom::new(3);
        // Mid-world (outside the beach margins) never grows.
        assert!(
            !grow_shell_pile(&mut w, 2000, 100, 0.0, 0.0, false, &mut rng),
            "the inland cell is refused"
        );
        // The beach cell with no sand under it (the floor is stone).
        let mut w2 = world();
        w2.world_surface = 100.0;
        for dx in -30i32..=30 {
            let i = w2.index(5 + dx.max(0) as i32, 101);
            w2.tiles[i] = crate::tile::Tile {
                kind: 1,
                flags: crate::tile::FLAG_ACTIVE,
                ..Default::default()
            };
        }
        for seed in 1..256u64 {
            let mut rng2 = FastRandom::new(seed);
            if grow_shell_pile(&mut w2, 5, 100, 0.0, 0.0, false, &mut rng2) {
                panic!("no sand under the cell yet the pile grew");
            }
        }
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
        let mut rng = FastRandom::new(11);
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
        let mut rng = FastRandom::new(11);
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
        let mut rng = FastRandom::new(11);
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
        let mut rng = FastRandom::new(11);
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
        let mut rng2 = FastRandom::new(11);
        assert!(
            !attempt_grow_tree_from_sapling(&mut w2, 100, 209, false, &mut rng2),
            "the occupied clear box refuses"
        );
    }

    /// The cactus arm's fork (`:56739-56783`): a standing column forks into the
    /// side shoots (the left/right clearance checks behind `Next(3)`) or the
    /// tip (`num6 < Next(2,8)` with the diagonals clear), and the lean arm
    /// (`:56788-56797`) fills the free cell when the column leans.
    #[test]
    fn the_cactus_forks_into_the_side_shoots_and_the_tip() {
        // A 3-tall column on sand: num6 = 2, so the tip gate's range and the
        // side shoots are both live over a seed sweep.
        let mut base = World::new("t".into(), 60, 80);
        base.world_surface = 40.0;
        for x in 0..60 {
            for y in 30..80 {
                let i = base.index(x, y);
                base.tiles[i] = crate::tile::Tile {
                    kind: 53,
                    ..Default::default()
                };
                base.tiles[i].flags |= FLAG_ACTIVE;
            }
        }
        for y in 28..30 {
            let i = base.index(30, y);
            base.tiles[i] = crate::tile::Tile {
                kind: 80,
                ..Default::default()
            };
            base.tiles[i].flags |= FLAG_ACTIVE;
        }
        let mut fork = 0;
        let mut runs = 0;
        for seed in 1u64..800 {
            let mut w = base.clone();
            let mut rng = FastRandom::new(seed);
            // j is the column's TOP cell (the C# walks the stack downward).
            grow_cactus(&mut w, 30, 28, &mut rng);
            runs += 1;
            // The side shoot or the tip adds a 80 outside the original column.
            fork += w
                .tiles
                .iter()
                .enumerate()
                .filter(|(idx, t)| t.active() && t.kind == 80 && {
                    let x = (*idx % 60) as i32;
                    let y = (*idx / 60) as i32;
                    // Outside the original column (28..=29): the tip above or
                    // the side shoots.
                    !(x == 30 && (28..=29).contains(&y))
                })
                .count();
        }
        assert!(fork > 0, "the column forks ({fork} over {runs} picks)");
    }

    #[test]
    fn test_spread_1_cactus_plants_on_sand_with_the_box_gates() {
        // `GrowCactus` (:56575-56660): the sand arm needs the neighbours
        // above clear, <=3 cacti in the i±6 box and >10 sand in it.
        let mut w = world();
        for dx in -8..=8 {
            put(&mut w, 100 + dx, 200, 53, 0); // the sand floor
        }
        let mut rng = FastRandom::new(7);
        grow_cactus(&mut w, 100, 200, &mut rng);
        assert!(w.tile(100, 199).is_some_and(|t| t.active() && t.kind == 80), "the cactus planted");
        // A cactus crowding (4 in the box) refuses.
        let mut w = world();
        for dx in -8..=8 {
            put(&mut w, 100 + dx, 200, 53, 0);
        }
        for dx in [-3i32, 3, -5, 5] {
            put(&mut w, 100 + dx, 199, 80, 0); // 4 existing cacti
        }
        let mut rng = FastRandom::new(7);
        grow_cactus(&mut w, 100, 200, &mut rng);
        assert!(w.tile(100, 199).is_none_or(|t| !t.active() || t.kind != 80), "the crowd gate");
    }

    #[test]
    fn test_spread_2_grass_grows_over_dirt_and_the_spikes_grow() {
        // `UpdateWorld_GrassGrowth` (:74269+): the 3x3 scan converts the
        // dirt neighbours to grass; the flower-27 exclusion; the spike arms
        // grow on the matching grass with the density gates.
        let mut w = world();
        w.world_surface = 250.0; // the surface below the cells
        put(&mut w, 100, 200, 2, 0); // the grass seed cell
        put(&mut w, 101, 200, 0, 0); // the dirt neighbour
        let mut rng = FastRandom::new(7);
        update_world_grass_growth(&mut w, 100, 200, 99, 102, 199, 202, true, &mut rng);
        assert!(
            w.tile(101, 200).is_some_and(|t| t.kind == 2),
            "the dirt converted to grass"
        );
        // The spike: the evil grass grows a 352 thorn onto the empty cell.
        let mut w = world();
        w.world_surface = 250.0; // the surface below the cells
        put(&mut w, 100, 200, 199, 0);
        put(&mut w, 101, 200, 199, 0);
        put(&mut w, 100, 199, 352, 0); // a spike on the land type
        let mut rng = FastRandom::new(7);
        let mut grew = false;
        for tick in 0..200 {
            update_world_overground_tile(&mut w, 100, 200, true, false, 0.0, 0.0, &mut rng);
            let grew_cell = [
                (99i32, 199),
                (100, 198),
                (101, 199),
                (100, 201), // the down walk lands too (the diamond covers it)
            ]
            .iter()
            .any(|&(cx, cy)| {
                w.tile(cx, cy).is_some_and(|t| t.active() && t.kind == 352)
            });
            if grew_cell {
                grew = true;
                break;
            }
        }
        assert!(grew, "the thorn grew eventually");
    }

    #[test]
    fn the_driver_picks_at_the_verbatim_rates() {
        // 4200×400 tiles at rate 1: the overground 3e-5·tiles = 50 picks,
        // ×1.5 raining = 75; the underground 1.5e-5·tiles = 25.
        let mut w = world();
        let mut rng = FastRandom::new(9);
        let picks = update_world(&mut w, 1, false, false, true, 0.0, 0.0, &mut rng);
        assert_eq!(picks, 50 + 25);
        let picks = update_world(&mut w, 1, true, false, true, 0.0, 0.0, &mut rng);
        assert_eq!(picks, 75 + 25);
        // The frozen time picks nothing.
        let picks = update_world(&mut w, 1, false, true, true, 0.0, 0.0, &mut rng);
        assert_eq!(picks, 0);
    }

    #[test]
    fn the_alch_harvest_matrix_runs_the_style_conditions() {
        // `IsAlchemyPlantHarvestable` (`WorldGen.cs:66148-66175`): the
        // per-style gates, against a daytime world under the surface
        // (world_surface = 100 in the 4200×400 fixture).
        let mut w = world();
        let dry = WeatherCtx::default();
        let rainy = WeatherCtx {
            raining: true,
            ..WeatherCtx::default()
        };
        // style 0 (daybloom): day only.
        w.day_time = true;
        assert!(is_alchemy_plant_harvestable(&w, 0, 120, &dry));
        w.day_time = false;
        assert!(!is_alchemy_plant_harvestable(&w, 0, 120, &dry));
        // style 1 (moonglow): night only.
        assert!(is_alchemy_plant_harvestable(&w, 1, 120, &dry));
        // style 3 (deathweed): night + (blood moon or full moon) — the
        // fixture's moon_phase 0 is the full moon.
        assert!(is_alchemy_plant_harvestable(&w, 3, 120, &dry));
        w.moon_phase = 4;
        assert!(!is_alchemy_plant_harvestable(&w, 3, 120, &dry));
        w.blood_moon = true;
        assert!(is_alchemy_plant_harvestable(&w, 3, 120, &dry));
        w.blood_moon = false;
        // style 4 (waterleaf): rain or any cloud.
        assert!(!is_alchemy_plant_harvestable(&w, 4, 120, &dry));
        assert!(is_alchemy_plant_harvestable(&w, 4, 120, &rainy));
        let cloudy = WeatherCtx {
            cloud_alpha: 0.1,
            ..WeatherCtx::default()
        };
        assert!(is_alchemy_plant_harvestable(&w, 4, 120, &cloudy));
        // style 5 (fireblossom): late afternoon (the C#'s
        // `Main.time > 40500.0` is strict) + the rain/height arm — the
        // herb harvests when it is not raining, or when it sits BELOW the
        // surface (`y > worldSurface`, the rows the rain does not reach).
        w.time = 40501.0;
        assert!(is_alchemy_plant_harvestable(&w, 5, 90, &dry));
        assert!(is_alchemy_plant_harvestable(&w, 5, 120, &rainy));
        assert!(!is_alchemy_plant_harvestable(&w, 5, 90, &rainy));
        // An unknown style never harvests.
        assert!(!is_alchemy_plant_harvestable(&w, 2, 90, &dry));
    }

    #[test]
    fn the_seed_harvest_arm_covers_the_grown_herbs() {
        // `IsHarvestableHerbWithSeed` (`WorldGen.cs:66118-66138`): only
        // 83/84 — 84 always, 83 through its style, 82 never.
        let mut w = world();
        w.day_time = true;
        let dry = WeatherCtx::default();
        assert!(!is_harvestable_herb_with_seed(&w, 82, 0, 90, &dry));
        assert!(is_harvestable_herb_with_seed(&w, 84, 0, 90, &dry));
        assert!(is_harvestable_herb_with_seed(&w, 83, 0, 90, &dry));
        w.day_time = false;
        assert!(!is_harvestable_herb_with_seed(&w, 83, 0, 90, &dry));
        assert!(is_harvestable_herb_with_seed(&w, 83, 1, 90, &dry));
        // A non-herb kind never harvests.
        assert!(!is_harvestable_herb_with_seed(&w, 1, 0, 90, &dry));
    }

    /// DW-2.8: the SpreadGrass conversion dispatch (WorldGen.cs:75207-75221):
    /// a conversion-grass dirt (the Conversion.Grass set) spreads through the
    /// flavor's map, a plain write otherwise, and the tree-invalidity kill
    /// rides the plain arm.
    #[test]
    fn spread_grass_2_the_conversion_dispatch_and_the_tree_kill() {
        let mut w = World::new("t".into(), 64, 64);
        w.world_surface = 30.0;
        let idx = w.index(10, 20);
        w.tiles[idx] = Tile { kind: 2, flags: FLAG_ACTIVE, ..Default::default() };
        let tree_idx = w.index(10, 19);
        w.tiles[tree_idx] = Tile { kind: 5, flags: FLAG_ACTIVE, ..Default::default() };
        // Jungle grass over the wood grass: the plain write (60 is not a
        // conversion dirt), and the forest tree survives a jungle ground.
        spread_grass(&mut w, 10, 20, 2, 60);
        assert_eq!(w.tile(10, 20).unwrap().kind, 60);
        assert!(w.tile(10, 19).is_some_and(|t| t.active()), "jungle keeps the tree");
        // Sand under the same tree kills it — the tree arm.
        let mut w2 = World::new("t".into(), 64, 64);
        w2.world_surface = 30.0;
        let idx = w2.index(10, 20);
        w2.tiles[idx] = Tile { kind: 2, flags: FLAG_ACTIVE, ..Default::default() };
        let tree_idx = w2.index(10, 19);
        w2.tiles[tree_idx] = Tile { kind: 5, flags: FLAG_ACTIVE, ..Default::default() };
        spread_grass(&mut w2, 10, 20, 2, 112);
        assert_eq!(w2.tile(10, 20).unwrap().kind, 112);
        assert!(!w2.tile(10, 19).is_some_and(|t| t.active()), "sand kills the forest tree");
    }

    /// DW-2.8: the conversion arm — the corruption spread over a
    /// conversion-grass dirt takes the flavor's Convert (whose map turns the
    /// wood grass 2 into the corruption grass 23), and the walls are left
    /// alone (the SpreadGrass call passes walls: false).
    #[test]
    fn spread_grass_2_the_conversion_arm_converts_through_the_flavor() {
        let mut w = World::new("t".into(), 64, 64);
        w.world_surface = 30.0;
        let idx = w.index(10, 20);
        w.tiles[idx] = Tile { kind: 2, flags: FLAG_ACTIVE, ..Default::default() };
        spread_grass(&mut w, 10, 20, 2, 23);
        assert_eq!(w.tile(10, 20).unwrap().kind, 23, "the corruption map lands");
        let wall_idx = w.index(11, 21);
        w.tiles[wall_idx].wall = 1;
        spread_grass(&mut w, 10, 20, 2, 23);
        assert_eq!(w.tile(11, 21).unwrap().wall, 1, "the walls stay (walls: false)");
    }
}

/// `WallID.Sets.AllowsPlantsToGrow` (`WallID.cs:97`): the wall set
/// `DefaultTreeWallTest` (`WorldGen.cs:30260-30267`) reads — the vanity and
/// ash profiles' `WallTest`. Hand-transcribed (the generated tables carry
/// neither set).
const ALLOWS_PLANTS_TO_GROW: [u16; 27] = [
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
    rng: &mut FastRandom,
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
    rng: &mut FastRandom,
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
        let num4 = rng.next(3);
        let mut num5 = rng.next(10);
        if i == num - 1 || i == num - num3 {
            num5 = 0;
        }
        // The re-roll guard (`:30454-30458`): a shape that repeats the same
        // side re-rolls until it picks another.
        while (num5 == 5 || num5 == 7) && left_branch || (num5 == 6 || num5 == 7) && right_branch {
            num5 = rng.next(10);
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
            let num4 = rng.next(3);
            let (fx, fy) = if rng.next(3) < 2 {
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
            let num4 = rng.next(3);
            let (fx, fy) = if rng.next(3) < 2 {
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
    if rng.next(3) == 0 {
        flag4 = false;
    }
    if rng.next(3) == 0 {
        flag5 = false;
    }
    // The side caps (`:30704-30746`).
    if flag5 {
        let num4 = rng.next(3);
        write_tree_frame(world, checked_x + 1, num - 1, settings.tree_tile_type, paint, 22, 132 + 22 * num4);
    }
    if flag4 {
        let num4 = rng.next(3);
        write_tree_frame(world, checked_x - 1, num - 1, settings.tree_tile_type, paint, 44, 132 + 22 * num4);
    }
    // The centre cap (`:30748-30783`).
    let num4 = rng.next(3);
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
    let num4 = rng.next(3);
    let (fx, fy) = if rng.next(13) != 0 {
        (22, 198 + 22 * num4)
    } else {
        (0, 198 + 22 * num4)
    };
    write_tree_frame(world, checked_x, num - num3, settings.tree_tile_type, paint, fx, fy);
    // `RangeFrame` (`:30820`).
    crate::framing::reframe_area(world, checked_x, num - num3, 3);
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
/// (`oceancave`/`AddTrees`); the growth pass keeps its own `FastRandom`
/// stream, so the arm is ported locally here (the `growth::spread_grass` /
/// `evil::spread_grass` precedent) — same maths, same frame table.
///
/// `treeHeightAddon`/`ignoreWalls` park (every ported caller passes 0 /
/// false) and the `RangeFrame`/`NetMessage` closing arms do not exist in
/// the port (documented in ANALYSIS).
pub fn grow_palm_tree(world: &mut World, i: i32, y: i32, rng: &mut FastRandom) -> bool {
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
                if num6 < 0.5 && rng.next(13) == 0 {
                    done = true;
                }
                if !done && num6 < 0.7 && rng.next(9) == 0 {
                    done = true;
                }
                if !done && !(num6 < 0.95) {
                    done = true;
                }
                if !done && rng.next(5) != 0 {
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

// --- M5-SPREAD-1: the underground dispatcher + its wall arms -----------------

/// `WallID.Sets.SpreadsCorruption` (`WallID.cs:93`) — the walls that carry the
/// corruption into `SpreadGrassWalls`/`SpreadDesertWalls`.
const SPREADS_CORRUPTION_WALLS: [u16; 4] = [69, 217, 220, 3];
/// `WallID.Sets.SpreadsCrimson` (`WallID.cs:94`).
const SPREADS_CRIMSON_WALLS: [u16; 4] = [83, 81, 218, 221];
/// `WallID.Sets.SpreadsHallow` (`WallID.cs:95`).
const SPREADS_HALLOW_WALLS: [u16; 4] = [70, 219, 222, 28];
/// `WallID.Sets.Conversion.Grass` (`WallID.cs:37`).
const CONVERSION_GRASS_WALLS: [u16; 12] = [63, 64, 65, 66, 67, 68, 69, 70, 81, 264, 265, 268];
/// `WallID.Sets.Conversion.Stone` (`WallID.cs:38`).
const CONVERSION_STONE_WALLS: [u16; 12] = [1, 61, 185, 3, 28, 83, 262, 274, 246, 248, 269, 349];
/// `WallID.Sets.Conversion.HardenedSand` (`WallID.cs:43`).
const CONVERSION_HARDENED_SAND_WALLS: [u16; 8] = [216, 217, 219, 218, 304, 305, 307, 306];
/// `WallID.Sets.Conversion.Sandstone` (`WallID.cs:42`).
const CONVERSION_SANDSTONE_WALLS: [u16; 8] = [187, 220, 222, 221, 275, 308, 310, 309];
/// `WallID.Sets.Conversion.PureSand` (`WallID.cs:44`).
const CONVERSION_PURE_SAND_WALLS: [u16; 4] = [216, 187, 304, 275];
/// `WallID.Sets.Conversion.NewWall1..4` (`WallID.cs:45-48`).
const CONVERSION_NEW_WALLS: [[u16; 10]; 4] = [
    [188, 192, 200, 204, 212, 276, 280, 288, 292, 300],
    [189, 193, 201, 205, 213, 277, 281, 289, 293, 301],
    [190, 194, 202, 206, 214, 278, 282, 290, 294, 302],
    [191, 195, 203, 207, 215, 279, 283, 291, 295, 303],
];
/// `TileID.Sets.SpreadsHallow` (`TileID.cs:584`): the hallow-source tiles.
const SPREADS_HALLOW: [u16; 10] = [109, 492, 117, 116, 402, 403, 164, 115, 110, 113];
/// `TileID.Sets.SpreadUnderground` (`TileID.cs:622`) — the underground tiles
/// the growth pass runs the grass arm over.
const SPREAD_UNDERGROUND: [u16; 9] = [23, 661, 109, 199, 662, 60, 70, 633, 226];

/// `WorldGen.Convert`'s wall chains for the three conversion types the wall
/// arms use (`WorldGen.cs:55261+` corruption, `:55176+` hallow, `:55100+`
/// crimson): the cell's wall set picks the new wall and the chain skips a cell
/// that already carries it.
fn convert_wall_by_case(wall: u16, conversion_type: u8) -> Option<u16> {
    // `(guard set, target)` in the C#'s else-if order.
    let cases: [(usize, u16); 8] = match conversion_type {
        1 => [
            (0, 69),
            (1, 3),
            (2, 217),
            (3, 220),
            (4, 188),
            (5, 189),
            (6, 190),
            (7, 191),
        ],
        2 => [
            (0, 70),
            (1, 28),
            (2, 219),
            (3, 222),
            (4, 200),
            (5, 201),
            (6, 202),
            (7, 203),
        ],
        // 4 — the crimson.
        _ => [
            (0, 81),
            (1, 83),
            (2, 218),
            (3, 221),
            (4, 192),
            (5, 193),
            (6, 194),
            (7, 195),
        ],
    };
    for (set, target) in cases {
        let in_set = match set {
            0 => CONVERSION_GRASS_WALLS.contains(&wall),
            1 => CONVERSION_STONE_WALLS.contains(&wall),
            2 => CONVERSION_HARDENED_SAND_WALLS.contains(&wall),
            3 => CONVERSION_SANDSTONE_WALLS.contains(&wall),
            n => CONVERSION_NEW_WALLS[n - 4].contains(&wall),
        };
        if in_set && wall != target {
            return Some(target);
        }
    }
    None
}

/// The crimson source tile set (`TileID.Sets.SpreadsCrimson`, `TileID.cs:589`).
fn crimson_set_slice() -> [u16; 10] {
    crate::spread::SPREADS_CRIMSON
}
/// The corruption source tile set (`TileID.Sets.SpreadsCorruption`, `TileID.cs:580`).
fn corrupt_set_slice() -> [u16; 10] {
    crate::spread::SPREADS_CORRUPTION
}
/// The hallow source tile set (`TileID.Sets.SpreadsHallow`).
fn hallow_set_slice() -> [u16; 10] {
    SPREADS_HALLOW
}

/// The `wallDist` box scan for the source tile kind (the wall arms'
/// `:74895-74906` shape).
fn box_has_source(
    world: &World,
    wall_dist: i32,
    i: i32,
    j: i32,
    is_source: &[u16],
) -> bool {
    for k in (i - wall_dist)..=(i + wall_dist) {
        for l in (j - wall_dist)..=(j + wall_dist) {
            if world
                .tile(k, l)
                .is_some_and(|t| t.active() && is_source.contains(&t.kind))
            {
                return true;
            }
        }
    }
    false
}

/// `WorldGen.SpreadGrassWalls` (`WorldGen.cs:74884-74982`): an evil cell
/// spreads into the neighbouring natural walls (the 63-68 family) when its own
/// source sits in the `wallDist` box — the crimson branch checks the crimson
/// source, the corruption branch the corruption one, the hallow branch the
/// hallow one. The neighbour wall then converts through `Convert`'s wall chain
/// (`tiles: false`).
pub fn spread_grass_walls(
    world: &mut World,
    wall_dist: i32,
    i: i32,
    j: i32,
    rng: &mut FastRandom,
) {
    if i < 10 || j < 10 || i >= world.width - 10 || j >= world.height - 10 {
        return;
    }
    let Some(tile) = world.tile(i, j) else {
        return;
    };
    let (wall, active, kind) = (tile.wall, tile.active(), tile.kind);
    let crimson = SPREADS_CRIMSON_WALLS.contains(&wall)
        || (active && crate::spread::SPREADS_CRIMSON.contains(&kind));
    let corrupt = SPREADS_CORRUPTION_WALLS.contains(&wall)
        || (active && crate::spread::SPREADS_CORRUPTION.contains(&kind));
    let hallow =
        SPREADS_HALLOW_WALLS.contains(&wall) || (active && SPREADS_HALLOW.contains(&kind));
    // `(conversion type, the neighbour wall gate, the source set)`: the
    // crimson arm's family is the 63-68 band, the corruption arm's is
    // 63/65/66/68 (`:74959-74962`) and the hallow arm's the 63-68 band.
    let arm: Option<(u8, &[u16], &[u16])> = if crimson {
        Some((4, &[63, 64, 65, 66, 67, 68], &crimson_set_slice()))
    } else if corrupt {
        Some((1, &[63, 65, 66, 68], &corrupt_set_slice()))
    } else if hallow {
        Some((2, &[63, 64, 65, 66, 67, 68], &hallow_set_slice()))
    } else {
        None
    };
    let Some((conversion_type, gate, source)) = arm else {
        return;
    };
    let (x, y) = (i + rng.next_range(-2, 3), j + rng.next_range(-2, 3));
    if x < 10 || y < 10 || x >= world.width - 10 || y >= world.height - 10 {
        return;
    }
    let Some(neighbour) = world.tile(x, y) else {
        return;
    };
    let neighbour = neighbour.wall;
    if !gate.contains(&neighbour) {
        return;
    }
    if !box_has_source(world, wall_dist, i, j, source) {
        return;
    }
    let Some(new) = convert_wall_by_case(neighbour, conversion_type) else {
        return;
    };
    let idx = world.index(x, y);
    world.tiles[idx].wall = new;
}

/// `WorldGen.SpreadDesertWalls` (`WorldGen.cs:74984-75123`): the desert walls
/// (the sandstone/hardened-sand family) convert to their evil variant when an
/// evil source sits in the `wallDist` box and the neighbour cell is still pure
/// sand.
pub fn spread_desert_walls(
    world: &mut World,
    wall_dist: i32,
    i: i32,
    j: i32,
    rng: &mut FastRandom,
) {
    if i < 10 || j < 10 || i >= world.width - 10 || j >= world.height - 10 {
        return;
    }
    let Some(tile) = world.tile(i, j) else {
        return;
    };
    let (wall, active, kind) = (tile.wall, tile.active(), tile.kind);
    let sandstone_wall = CONVERSION_SANDSTONE_WALLS.contains(&wall);
    let hardened_wall = CONVERSION_HARDENED_SAND_WALLS.contains(&wall);
    let sandstone_tile = active && crate::tiledata::SET_CONVERSION_SANDSTONE.has(kind);
    let hardened_tile = active && crate::tiledata::SET_CONVERSION_SAND.has(kind);
    if !sandstone_wall && !hardened_wall && !sandstone_tile && !hardened_tile {
        return;
    }
    let mut flavour = 0u8;
    if SPREADS_CORRUPTION_WALLS.contains(&wall)
        || (active && crate::spread::SPREADS_CORRUPTION.contains(&kind))
    {
        flavour = 1;
    }
    if SPREADS_HALLOW_WALLS.contains(&wall) || (active && SPREADS_HALLOW.contains(&kind)) {
        flavour = 2;
    }
    if SPREADS_CRIMSON_WALLS.contains(&wall)
        || (active && crate::spread::SPREADS_CRIMSON.contains(&kind))
    {
        flavour = 3;
    }
    if flavour == 0 {
        return;
    }
    let conversion_type = match flavour {
        2 => 2,
        3 => 4,
        _ => 1,
    };
    let (x, y) = (i + rng.next_range(-2, 3), j + rng.next_range(-2, 3));
    let Some(neighbour) = world.tile(x, y) else {
        return;
    };
    if !CONVERSION_PURE_SAND_WALLS.contains(&neighbour.wall) {
        return;
    }
    let source: &[u16] = match conversion_type {
        4 => &crimson_set_slice(),
        2 => &hallow_set_slice(),
        _ => &corrupt_set_slice(),
    };
    if !box_has_source(world, wall_dist, i, j, source) {
        return;
    }
    let Some(new) = convert_wall_by_case(neighbour.wall, conversion_type) else {
        return;
    };
    let idx = world.index(x, y);
    world.tiles[idx].wall = new;
}

/// `WorldGen.UpdateWorld_UndergroundTile` (`WorldGen.cs:73214-73614`): the
/// underground growth dispatcher — the clamps, the alch/hard-mode arms, the
/// grass growth (underground) + the spike arms, the lily conversion, the five
/// vine families and, behind `allowed_to_spread`, the wall arms.
///
/// Parked (documented in ANALYSIS): the 655 life-fruit kill (the NPC store),
/// `plantDye`, the 519 cattail and the 549/53 seaweed, the 444/396 moss and
/// oyster arms (`PlayerLOS`/`AnchorValid`/`CountNearBlocksTypes` have no
/// ports), `GrowWeb`, `TrySpawningTownNPC`, the net sides, and the C#'s dual
/// `genRand`/`Main.rand` streams (the port's per-consumer stream carries every
/// draw).
pub fn update_world_underground_tile(
    world: &mut World,
    i: i32,
    j: i32,
    wall_dist: i32,
    allowed_to_spread: bool,
    day_time: bool,
    raining: bool,
    rng: &mut FastRandom,
) {
    // The clamped box (`:73216-73231`).
    let min_i = (i - 1).max(10);
    let max_i = (i + 2).min(world.width - 10);
    let min_j = (j - 1).max(10);
    let max_j = (j + 2).min(world.height - 10);
    let Some(tile) = world.tile(i, j) else {
        return;
    };
    let (kind, active) = (tile.kind, tile.active());
    // The 655 life-fruit kill parks (`:73239-73246`).
    if TILE_ALCH.contains(&kind) {
        grow_alch(world, i, j, day_time, raining, rng);
        return;
    }
    if !active {
        // The hard-mode infection arm (`:73252`); the dye/cattail/seaweed
        // arms park.
        crate::spread::hard_update_world(world, i, j, false, true, rng);
        return;
    }
    // The grass family + the spikes (`:73281-73295`).
    if SPREAD_UNDERGROUND.contains(&kind) {
        update_world_grass_growth(
            world,
            i,
            j,
            min_i,
            max_i,
            min_j,
            max_j,
            allowed_to_spread,
            rng,
        );
        if matches!(kind, 32 | 352) && rng.next(3) == 0 {
            if kind == 32 {
                grow_spike(world, i, j, 32, 23, rng);
            } else {
                grow_spike(world, i, j, 352, 199, rng);
            }
        }
        return;
    }
    // The lily → 74 conversion (`:73296-73310`).
    if kind == 61 && rng.next(3) == 0 && world.tile(i, j + 1).map(|t| t.kind).unwrap_or(0) != 226 {
        let frame_x = world.tile(i, j).map(|t| t.frame_x).unwrap_or(0);
        if frame_x < 144 {
            if rng.next(4) == 0 {
                if let Some(t) = world.tile_mut(i, j) {
                    t.frame_x = (162 + rng.next_range(0, 8) * 18) as i16;
                }
            }
            if let Some(t) = world.tile_mut(i, j) {
                t.kind = 74;
            }
        }
        return;
    }
    // The five vine families (`:73311-73482`) — the port's one-fn arm. The
    // 444/396 moss and oyster arms park.
    grow_vine_arm(world, i, j, rng);
    // The wall arms (`:72499-72502`, behind `AllowedToSpreadInfections`); the
    // web and town-NPC arms park.
    if allowed_to_spread {
        spread_grass_walls(world, wall_dist, i, j, rng);
        spread_desert_walls(world, wall_dist, i, j, rng);
    }
}
