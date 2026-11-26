//! The dual-dungeon room-content slice — the unclaimed remainder of the
//! `DungeonGlobal*` furniture family the W4 lanes left open:
//!
//! - the cracked-brick solidity toggle + chance roll
//!   (`WorldGen.SetCrackedBrickSolidity`, `WorldGen.cs:11022-11027`;
//!   `DungeonHallSettings.CrackedBrickChance`, `DungeonHallSettings.cs:15`;
//!   the `NextDouble() <= chance` roll, `LegacyDungeonHall.cs:79-82`),
//! - the dual `isDungeonTile`/`isCrackedBrick`/… set clones + the
//!   `IsConsidered*` family (`DungeonCrawler.cs:133-183`,
//!   `DungeonUtils.cs:1809-1957`),
//! - the room-object helpers `GenerateDungeonWaterCandle`/`PotionBottle`/
//!   `Pot`/`Book` + `FirstSolid` (`DungeonUtils.cs:373-413/:514-543`),
//! - the shelf-style rolls (`DungeonCrawler.cs:256-267`) and the
//!   `DungeonGlobalBookshelves` pass (`DungeonGlobalBookshelves.cs:23-207`),
//! - the GroundFurniture selection layer (R2): the wall-keyed default
//!   item rows + the style substitution chain + the `Next(13)` kind
//!   ladder + the area tables + the budget scalars
//!   (`DungeonGlobalGroundFurniture.cs:37-63/:191-218/:275-554/:862-873`).
//!   The placement tail (`:555-857`) needs the room/hall object lists —
//!   the wiring round's seam.
//!
//! Claim census (this lane's UNCLAIMED verification): no pushed W4 branch
//! diff touches these symbols; c32 owns the style TABLES (its
//! `dungeon_styles.rs` defers the `:111-128` dual list arms, the `:139-183`
//! set clones and `GetPlatformStyle`/`GetWindowPlatformStyle` explicitly),
//! d107 owns the drunk re-rolls/remix bands/SurfaceCaves gates and the
//! `dualDungeons` PRUNING fix (its P1), not the world content.
//!
//! The style rows below carry ONLY the fields this slice consumes (the
//! set-clone walk + `GetStyleForWall` + `GetPlatformStyle` +
//! `GetBookshelfMinMaxSizes`) — the full furniture/chest tables are c32's
//! `DungeonStyleData`; the merger dedupes these rows onto that struct.
//! `GetPlatformStyle`'s `ItemID.Sets.DerivedPlacementDetails` read
//! (`DungeonGenerationStyleData.cs:150-158`) is resolved here as a cited
//! static table of the platform items' `placeStyle` values (Item.cs rows
//! cited per entry) — c32 deferred the lookup to the tr-app `itemdata`
//! seam; the values are boot-static, so the pure port needs no item store.
//!
//! UNWIRED: no pipeline step calls this module — the dual-dungeon content
//! passes run only under `dualDungeons` (d107's pruning lane owns the
//! wiring), so no RNG draw and no tile write happens in a classic world:
//! both world-hash pins stay, the `a == b` halves are untouched.

use crate::tile::FLAG_ACTIVE;
use crate::world::World;

use super::dungeon::HallSeed;
use super::dungeon_bounds::DungeonBounds;
use super::dungeon_features::{DungeonFootprint, DUNGEON_WALLS};
use super::noise::Rng;

// --- the cracked-brick family ----------------------------------------------

/// `WorldGen.SetCrackedBrickSolidity` (`WorldGen.cs:11022-11027`): the C#
/// mutates `Main.tileSolid[481/482/483]` — false while the crawl runs
/// (`DungeonCrawler.MakeDungeon`, `:350`), true after `SmoothWorld`
/// (`WorldGen.cs:16422`), false again at its tail (`:16608`).
///
/// The port's `tiledata::TILE_SOLID` is a GENERATED static table (never
/// hand-edited), so there is no mutable solidity table to poke. This port
/// keeps the toggle as EXPLICIT STATE the solidity queries read:
/// [`CrackedSolidity`] is threaded through [`first_solid`] (the only
/// solidity consumer in this slice); a caller modelling the crawl window
/// passes `false`, the post-SmoothWorld passes `true`. The static table's
/// boot value for 481-483 is `true` (the post-smooth state). Deviation
/// recorded in the lane doc (12k-f24-worldgen-gated.md).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CrackedSolidity(pub bool);

impl CrackedSolidity {
    /// `Main.tileSolid[kind]` with the `SetCrackedBrickSolidity` override
    /// applied to the cracked triple (`TileID.Sets.CrackedBricks`,
    /// `TileID.cs:498`: 481/482/483).
    pub fn solid_kind(&self, kind: u16) -> bool {
        if crate::tiledata::SET_CRACKED_BRICKS.has(kind) {
            return self.0;
        }
        crate::tiledata::TILE_SOLID.has(kind)
    }
}

/// `DungeonHallSettings.CrackedBrickChance` (`DungeonHallSettings.cs:15`)
/// — the default 0.166 the legacy halls roll against. The dual layout
/// overrides it to 1.0 (`DualDungeonLayoutProvider.cs:137`) and the
/// entrance/dual pins to 0.0 (`DungeonCrawler.cs:538/:758/:768`).
pub const CRACKED_BRICK_CHANCE: f64 = 0.166;

/// `LegacyDungeonHall.cs:79-82` — `flag = unifiedRandom.NextDouble() <=
/// CrackedBrickChance`, gated on `chance > 0.0` (the 0.0 pins skip the
/// draw entirely — the stream must NOT advance). `r` is the per-hall
/// `UnifiedRandom(RandomSeed)` stream; [`HallSeed`] is the port's accepted
/// per-hall stand-in (`dungeon.rs:105-107`), its `next_double` the same
/// splitmix draw its `next`/`range` consume — one stream step, no extra
/// state (the existing `next`/`range` outputs are unchanged).
pub fn roll_cracked_brick(r: &mut HallSeed, chance: f64) -> bool {
    if chance > 0.0 {
        r.next_double() <= chance
    } else {
        false
    }
}

// --- the dual style rows (the set-clone / bookshelf fields only) ------------

/// The subset of `DungeonGenerationStyleData`
/// (`DungeonGenerationStyleData.cs:11-95`) this slice reads. c32's
/// `DungeonStyleData` carries the full row; the merger folds these fields
/// onto it (cited overlap — the values here are verbatim from the same
/// statics).
#[derive(Clone, Copy, Debug)]
pub(crate) struct RoomStyle {
    /// `Style` — the style id (0 = the rolled colour's wrapper).
    pub style: u8,
    pub brick_tile: u16,
    pub brick_grass_tile: Option<u16>,
    pub brick_cracked_tile: u16,
    pub brick_wall: u16,
    pub extra_tiles: &'static [u16],
    pub extra_walls: &'static [u16],
    pub pit_trap_tile: u16,
    pub window_glass_wall: u16,
    pub window_closed_glass_wall: u16,
    pub window_edge_wall: u16,
    /// `PlatformItemTypes` — the bookshelf platform roll's item list.
    pub platform_items: &'static [i32],
    /// `GetBookshelfMinMaxSizes` override (`DungeonGenerationStyles.cs:
    /// 34-38`): only `LivingWoodStyleData` overrides (3, 7); every other
    /// style keeps the caller's defaults (the base, `StyleData.cs:102-106`).
    pub bookshelf_min_max: Option<(i32, i32)>,
    /// `SubStyles` (`:345-350/:592/:754-758`) — the set-clone walk and
    /// `GetStyleForWall` descend into these.
    pub sub_styles: &'static [RoomStyle],
}

/// `DungeonGenerationStyles.Shimmer` (`DungeonGenerationStyles.cs:189-228`).
pub(crate) const SHIMMER: RoomStyle = RoomStyle {
    style: 11,
    brick_tile: 667,
    brick_grass_tile: None,
    brick_cracked_tile: 123,
    brick_wall: 322,
    extra_tiles: &[],
    extra_walls: &[],
    pit_trap_tile: 123,
    window_glass_wall: 93,
    window_closed_glass_wall: 149,
    window_edge_wall: 37,
    platform_items: &[5562],
    bookshelf_min_max: None,
    sub_styles: &[],
};

/// `DungeonGenerationStyles.Spider` (`:229-267`).
pub(crate) const SPIDER: RoomStyle = RoomStyle {
    style: 12,
    brick_tile: 156,
    brick_grass_tile: None,
    brick_cracked_tile: 123,
    brick_wall: 62,
    extra_tiles: &[],
    extra_walls: &[],
    pit_trap_tile: 123,
    window_glass_wall: 21,
    window_closed_glass_wall: 4,
    window_edge_wall: 36,
    platform_items: &[4416],
    bookshelf_min_max: None,
    sub_styles: &[],
};

/// `DungeonGenerationStyles.LivingWood` (`:268-306`) — the
/// `LivingWoodStyleData` bookshelf override (3, 7) (`:34-38`).
pub(crate) const LIVING_WOOD: RoomStyle = RoomStyle {
    style: 13,
    brick_tile: 191,
    brick_grass_tile: None,
    brick_cracked_tile: 192,
    brick_wall: 244,
    extra_tiles: &[],
    extra_walls: &[],
    pit_trap_tile: 123,
    window_glass_wall: 21,
    window_closed_glass_wall: 4,
    window_edge_wall: 196,
    platform_items: &[2629],
    bookshelf_min_max: Some((3, 7)),
    sub_styles: &[],
};

/// `DungeonGenerationStyles.Cavern` (`:307-351`) — SubStyles
/// [Shimmer, Spider, LivingWood] (`:345-350`).
pub(crate) const CAVERN: RoomStyle = RoomStyle {
    style: 1,
    brick_tile: 38,
    brick_grass_tile: None,
    brick_cracked_tile: 123,
    brick_wall: 349,
    extra_tiles: &[],
    extra_walls: &[],
    pit_trap_tile: 123,
    window_glass_wall: 21,
    window_closed_glass_wall: 4,
    window_edge_wall: 5,
    platform_items: &[94, 4416],
    bookshelf_min_max: None,
    sub_styles: &[SHIMMER, SPIDER, LIVING_WOOD],
};

/// `DungeonGenerationStyles.Snow` (`:352-392`).
pub(crate) const SNOW: RoomStyle = RoomStyle {
    style: 2,
    brick_tile: 161,
    brick_grass_tile: None,
    brick_cracked_tile: 224,
    brick_wall: 71,
    extra_tiles: &[147],
    extra_walls: &[40],
    pit_trap_tile: 224,
    window_glass_wall: 90,
    window_closed_glass_wall: 149,
    window_edge_wall: 31,
    platform_items: &[3908, 5812],
    bookshelf_min_max: None,
    sub_styles: &[],
};

/// `DungeonGenerationStyles.Desert` (`:393-433`).
pub(crate) const DESERT: RoomStyle = RoomStyle {
    style: 3,
    brick_tile: 396,
    brick_grass_tile: None,
    brick_cracked_tile: 53,
    brick_wall: 187,
    extra_tiles: &[397, 404],
    extra_walls: &[216, 223],
    pit_trap_tile: 53,
    window_glass_wall: 89,
    window_closed_glass_wall: 151,
    window_edge_wall: 34,
    platform_items: &[4311],
    bookshelf_min_max: None,
    sub_styles: &[],
};

/// `DungeonGenerationStyles.Corruption` (`:434-473`).
pub(crate) const CORRUPTION: RoomStyle = RoomStyle {
    style: 4,
    brick_tile: 25,
    brick_grass_tile: None,
    brick_cracked_tile: 112,
    brick_wall: 3,
    extra_tiles: &[],
    extra_walls: &[188, 189, 190, 191],
    pit_trap_tile: 112,
    window_glass_wall: 88,
    window_closed_glass_wall: 41,
    window_edge_wall: 33,
    platform_items: &[631, 3957, 5770],
    bookshelf_min_max: None,
    sub_styles: &[],
};

/// `DungeonGenerationStyles.Crimson` (`:474-513`).
pub(crate) const CRIMSON: RoomStyle = RoomStyle {
    style: 5,
    brick_tile: 203,
    brick_grass_tile: None,
    brick_cracked_tile: 234,
    brick_wall: 83,
    extra_tiles: &[],
    extra_walls: &[192, 193, 194, 195],
    pit_trap_tile: 234,
    window_glass_wall: 92,
    window_closed_glass_wall: 85,
    window_edge_wall: 174,
    platform_items: &[913, 3907, 5791],
    bookshelf_min_max: None,
    sub_styles: &[],
};

/// `DungeonGenerationStyles.Crystal` (`:514-552`).
pub(crate) const CRYSTAL: RoomStyle = RoomStyle {
    style: 15,
    brick_tile: 385,
    brick_grass_tile: None,
    brick_cracked_tile: 116,
    brick_wall: 186,
    extra_tiles: &[],
    extra_walls: &[],
    pit_trap_tile: 116,
    window_glass_wall: 88,
    window_closed_glass_wall: 43,
    window_edge_wall: 22,
    platform_items: &[3903],
    bookshelf_min_max: None,
    sub_styles: &[],
};

/// `DungeonGenerationStyles.Hallow` (`:553-593`) — SubStyles [Crystal]
/// (`:592`).
pub(crate) const HALLOW: RoomStyle = RoomStyle {
    style: 6,
    brick_tile: 117,
    brick_grass_tile: None,
    brick_cracked_tile: 116,
    brick_wall: 28,
    extra_tiles: &[],
    extra_walls: &[200, 201, 202, 203],
    pit_trap_tile: 116,
    window_glass_wall: 91,
    window_closed_glass_wall: 43,
    window_edge_wall: 22,
    platform_items: &[633, 3903],
    bookshelf_min_max: None,
    sub_styles: &[CRYSTAL],
};

/// `DungeonGenerationStyles.Beehive` (`:634-673`).
pub(crate) const BEEHIVE: RoomStyle = RoomStyle {
    style: 9,
    brick_tile: 225,
    brick_grass_tile: None,
    brick_cracked_tile: 123,
    brick_wall: 86,
    extra_tiles: &[],
    extra_walls: &[],
    pit_trap_tile: 123,
    window_glass_wall: 89,
    window_closed_glass_wall: 172,
    window_edge_wall: 151,
    platform_items: &[2630],
    bookshelf_min_max: None,
    sub_styles: &[],
};

/// `DungeonGenerationStyles.LivingMahogany` (`:674-713`) — a
/// `LivingWoodStyleData`, so the (3, 7) bookshelf override applies.
pub(crate) const LIVING_MAHOGANY: RoomStyle = RoomStyle {
    style: 14,
    brick_tile: 383,
    brick_grass_tile: None,
    brick_cracked_tile: 384,
    brick_wall: 244,
    extra_tiles: &[],
    extra_walls: &[],
    pit_trap_tile: 123,
    window_glass_wall: 21,
    window_closed_glass_wall: 42,
    window_edge_wall: 196,
    platform_items: &[2629],
    bookshelf_min_max: Some((3, 7)),
    sub_styles: &[],
};

/// `DungeonGenerationStyles.Jungle` (`:714-759`) — SubStyles
/// [Beehive, LivingMahogany] (`:754-758`).
pub(crate) const JUNGLE: RoomStyle = RoomStyle {
    style: 8,
    brick_tile: 59,
    brick_grass_tile: Some(60),
    brick_cracked_tile: 123,
    brick_wall: 64,
    extra_tiles: &[],
    extra_walls: &[204, 205, 206, 207, 15],
    pit_trap_tile: 123,
    window_glass_wall: 91,
    window_closed_glass_wall: 42,
    window_edge_wall: 24,
    platform_items: &[632],
    bookshelf_min_max: None,
    sub_styles: &[BEEHIVE, LIVING_MAHOGANY],
};

/// `DungeonGenerationStyles.Temple` (`:760-797`).
pub(crate) const TEMPLE: RoomStyle = RoomStyle {
    style: 10,
    brick_tile: 226,
    brick_grass_tile: None,
    brick_cracked_tile: 123,
    brick_wall: 87,
    extra_tiles: &[],
    extra_walls: &[],
    pit_trap_tile: 123,
    window_glass_wall: 92,
    window_closed_glass_wall: 42,
    window_edge_wall: 24,
    platform_items: &[3906],
    bookshelf_min_max: None,
    sub_styles: &[],
};

/// `DungeonGenerationStyles.GetCurrentDungeonStyle`
/// (`DungeonGenerationStyles.cs:95-135`) — the style-0 wrapper over the
/// rolled colour's genvars fields. `PlatformItemTypes` is `new int[0]`
/// (`:114`), so `GetPlatformStyle` returns −1 WITHOUT a draw. The window
/// triple follows the COLOUR roll (`DungeonCrawler.cs:68-91`), not the
/// drunk wall re-roll (the C# `genVars.brickWallType` IS re-rolled at
/// `:94-108`, so the wrapper's `BrickWallType` uses the effective wall).
pub(crate) fn current_dungeon_style(
    brick_tile: u16,
    brick_cracked_tile: u16,
    brick_wall: u16,
    window_glass_wall: u16,
    window_closed_glass_wall: u16,
    window_edge_wall: u16,
) -> RoomStyle {
    RoomStyle {
        style: 0,
        brick_tile,
        brick_grass_tile: None,
        brick_cracked_tile,
        brick_wall,
        extra_tiles: &[],
        extra_walls: &[],
        // `:107`: PitTrapTileType = brickCrackedTileType.
        pit_trap_tile: brick_cracked_tile,
        window_glass_wall,
        window_closed_glass_wall,
        window_edge_wall,
        platform_items: &[],
        bookshelf_min_max: None,
        sub_styles: &[],
    }
}

/// `DungeonGenerationStyleData.TileIsInStyle`
/// (`DungeonGenerationStyleData.cs:108-129`).
pub(crate) fn tile_is_in_style(
    s: &RoomStyle,
    tile_type: u16,
    include_cracked: bool,
    include_extras: bool,
) -> bool {
    if let Some(grass) = s.brick_grass_tile {
        if tile_type == grass {
            return true;
        }
    }
    if include_cracked && tile_type == s.brick_cracked_tile {
        return true;
    }
    if include_extras && s.extra_tiles.contains(&tile_type) {
        return true;
    }
    tile_type == s.brick_tile
}

/// `DungeonGenerationStyleData.WallIsInStyle` (`:131-148`).
pub(crate) fn wall_is_in_style(
    s: &RoomStyle,
    wall_type: u16,
    include_windows: bool,
    include_extras: bool,
) -> bool {
    if include_windows
        && (wall_type == s.window_glass_wall
            || wall_type == s.window_edge_wall
            || wall_type == s.window_closed_glass_wall)
    {
        return true;
    }
    if include_extras && s.extra_walls.contains(&wall_type) {
        return true;
    }
    wall_type == s.brick_wall
}

/// `DungeonGenerationStyles.GetStyleForWall`
/// (`DungeonGenerationStyles.cs:160-185`) — the style-0 `Main.wallDungeon`
/// arm first, then `WallIsInStyle(includeWindows: false, includeExtras:
/// true)`, then the sub-styles. Returns the index into `styles` (the port
/// has no heap style objects; the caller indexes its own slice).
pub(crate) fn style_for_wall(styles: &[RoomStyle], wall_type: u16) -> Option<usize> {
    for (i, s) in styles.iter().enumerate() {
        if s.style == 0
            && (wall_type as usize) < crate::edit::WALL_COUNT as usize
            && DUNGEON_WALLS.contains(&wall_type)
        {
            return Some(i);
        }
        if wall_is_in_style(s, wall_type, false, true) {
            return Some(i);
        }
        for sub in s.sub_styles {
            if wall_is_in_style(sub, wall_type, false, true) {
                // The sub-style is not in the top-level slice; the C#
                // returns the sub object. The bookshelf caller only reads
                // Style/GetPlatformStyle/GetBookshelfMinMaxSizes, so the
                // port returns a synthetic index past the slice end and
                // `style_at` resolves it (see below).
                return Some(styles.len() + sub_index(s, sub));
            }
        }
    }
    None
}

/// The sub-style's position in its parent's `sub_styles` — the synthetic
/// index `style_for_wall` encodes as `styles.len() + parent_offset + sub`.
fn sub_index(parent: &RoomStyle, sub: &RoomStyle) -> usize {
    // Parents have at most 3 sub-styles; encode parent slot * 4 + sub slot
    // so the synthetic index never collides (max 2 parents per quad).
    let parent_slot = match parent.style {
        1 => 0, // Cavern
        6 => 1, // Hallow
        8 => 2, // Jungle
        _ => 3,
    };
    let sub_slot = parent
        .sub_styles
        .iter()
        .position(|s| s.style == sub.style)
        .unwrap_or(0);
    parent_slot * 4 + sub_slot
}

/// Resolve a `style_for_wall` index (top-level or synthetic sub-style).
pub(crate) fn style_at<'a>(styles: &'a [RoomStyle], idx: usize) -> &'a RoomStyle {
    if idx < styles.len() {
        return &styles[idx];
    }
    let enc = idx - styles.len();
    let (parent_slot, sub_slot) = (enc / 4, enc % 4);
    let parent_style = match parent_slot {
        0 => 1u8,
        1 => 6,
        2 => 8,
        _ => 0,
    };
    for s in styles {
        if s.style == parent_style {
            return &s.sub_styles[sub_slot];
        }
    }
    unreachable!("style_for_wall only encodes sub-styles of listed parents")
}

/// `ItemID.Sets.DerivedPlacementDetails[item].tileStyle` for the platform
/// items the style rows name — the boot-static `placeStyle` values
/// (`ItemID.cs:304-325` derives them from `Item.SetDefaults`):
/// 94 → 0 (Item.cs:2466-2476, no placeStyle), 631 → 1 (:8722), 632 → 2
/// (:8734), 633 → 3 (:8746), 913 → 5 (:11917), 2549 → 18 (:24710),
/// 2629 → 23 / 2630 → 24 (`21 + type - 2627`, :25628-25642),
/// 3903 → 30 / 3906 → 33 / 3907 → 34 / 3908 → 35 (`30 + type - 3903`,
/// :34540-34552), 3957 → 37 (:35086), 4311 → 42 (:36940), 4416 → 43
/// (:37437), 5562 → 50 (:43840), 5770 → 55 (:45185), 5791 → 56 (:45311),
/// 5812 → 57 (:45437). Every one places tile 19 (platform).
fn platform_item_style(item: i32) -> i32 {
    match item {
        94 => 0,
        631 => 1,
        632 => 2,
        633 => 3,
        913 => 5,
        2549 => 18,
        2629 => 23,
        2630 => 24,
        3903 => 30,
        3906 => 33,
        3907 => 34,
        3908 => 35,
        3957 => 37,
        4311 => 42,
        4416 => 43,
        5562 => 50,
        5770 => 55,
        5791 => 56,
        5812 => 57,
        _ => -1,
    }
}

/// `DungeonGenerationStyleData.GetPlatformStyle`
/// (`DungeonGenerationStyleData.cs:150-158`): empty/null list → −1 with
/// NO draw; else `PlatformItemTypes[genRand.Next(len)]` → its
/// `DerivedPlacementDetails.tileStyle`.
pub(crate) fn get_platform_style(s: &RoomStyle, r: &mut Rng) -> i32 {
    if s.platform_items.is_empty() {
        return -1;
    }
    let item = s.platform_items[r.range(0, s.platform_items.len() as i32) as usize];
    if item >= 0 {
        platform_item_style(item)
    } else {
        -1
    }
}

// --- the dual set clones + IsConsidered* ------------------------------------

/// `WallID.Sets.Glass` (`WallID.cs:91`).
const WALL_GLASS: &[u16] = &[21, 88, 89, 90, 91, 92, 241, 93, 243, 242];

/// `Main.tileDungeon` (`Main.cs:7992-7997`) — the six dungeon bricks.
/// NOTE: `dungeon_features::DUNGEON_TILES` ([41,43,44,481,482,483]) is the
/// spikes lane's brick+cracked union, NOT this set; the IsConsidered*
/// family reads the C# statics verbatim here.
const MAIN_TILE_DUNGEON: &[u16] = &[41, 43, 44, 677, 678, 679];

/// One dungeon's `DungeonGenVars` membership sets — the
/// `SetupDungeonGenVarVariables` clones (`DungeonCrawler.cs:133-183`).
/// The C# clones the six static bool arrays under `dualDungeons.Enabled`
/// and ORs in every style's (and sub-style's) brick/wall rows; the port
/// builds the same sets as sorted vectors (the arrays are indexed by
/// tile/wall id — membership is all any consumer reads).
#[derive(Clone, Debug, Default)]
pub(crate) struct DungeonTileSets {
    pub is_dungeon_tile: Vec<u16>,
    pub is_cracked_brick: Vec<u16>,
    pub is_pit_trap_tile: Vec<u16>,
    pub is_dungeon_wall: Vec<u16>,
    pub is_dungeon_wall_edge: Vec<u16>,
    pub is_dungeon_wall_glass: Vec<u16>,
}

fn push_set(set: &mut Vec<u16>, id: u16) {
    if !set.contains(&id) {
        set.push(id);
    }
}

impl DungeonTileSets {
    /// `DungeonCrawler.cs:133-183` — the base sets (the C# assigns the
    /// statics by reference, then clones them under dualDungeons; the
    /// port starts from the same values either way) plus the dual style
    /// walk when `dual` is set. `styles` is the dungeon's
    /// `dungeonGenerationStyles` list (the quad); the walk appends each
    /// style's SubStyles to the iteration list FIRST (`:147-154`), then
    /// ORs every row (`:155-182`).
    pub fn build(styles: &[RoomStyle], dual: bool) -> Self {
        let mut s = DungeonTileSets {
            is_dungeon_tile: MAIN_TILE_DUNGEON.to_vec(),
            is_cracked_brick: vec![481, 482, 483], // TileID.Sets.CrackedBricks (TileID.cs:498)
            is_pit_trap_tile: vec![481, 482, 483],
            is_dungeon_wall: DUNGEON_WALLS.to_vec(),
            is_dungeon_wall_edge: DUNGEON_WALLS.to_vec(),
            is_dungeon_wall_glass: WALL_GLASS.to_vec(),
        };
        if !dual {
            return s;
        }
        // `:147-154`: list = styles + every style's SubStyles.
        let mut list: Vec<&RoomStyle> = styles.iter().collect();
        for style in styles {
            for sub in style.sub_styles {
                list.push(sub);
            }
        }
        // `:155-182`: the per-row OR walk, in list order.
        for item in list {
            push_set(&mut s.is_dungeon_tile, item.brick_tile);
            if let Some(grass) = item.brick_grass_tile {
                push_set(&mut s.is_dungeon_tile, grass);
            }
            push_set(&mut s.is_cracked_brick, item.brick_cracked_tile);
            push_set(&mut s.is_pit_trap_tile, item.pit_trap_tile);
            push_set(&mut s.is_dungeon_wall, item.brick_wall);
            push_set(&mut s.is_dungeon_wall_edge, item.window_edge_wall);
            push_set(&mut s.is_dungeon_wall_glass, item.window_glass_wall);
            push_set(&mut s.is_dungeon_wall_glass, item.window_closed_glass_wall);
            for &t in item.extra_tiles {
                push_set(&mut s.is_dungeon_tile, t);
            }
            for &w in item.extra_walls {
                push_set(&mut s.is_dungeon_wall, w);
            }
        }
        s
    }
}

/// `DungeonUtils.IsConsideredDungeonTile` (`DungeonUtils.cs:1809-1830`).
/// `all` walks every dungeon's sets (`GenVars.dungeonGenVars`); otherwise
/// only `current` (`GenVars.CurrentDungeonGenVars`). The `tileType > 0`
/// guard is the C#'s (id 0 is dirt — never a dungeon tile).
pub(crate) fn is_considered_dungeon_tile(
    sets: &[DungeonTileSets],
    current: usize,
    tile_type: u16,
    all: bool,
) -> bool {
    if tile_type > 0 && MAIN_TILE_DUNGEON.contains(&tile_type) {
        return true;
    }
    if all {
        sets.iter().any(|s| s.is_dungeon_tile.contains(&tile_type))
    } else {
        sets[current].is_dungeon_tile.contains(&tile_type)
    }
}

/// `IsConsideredCrackedDungeonTile` (`:1832-1849`) — NO Main static arm.
pub(crate) fn is_considered_cracked_dungeon_tile(
    sets: &[DungeonTileSets],
    current: usize,
    tile_type: u16,
    all: bool,
) -> bool {
    if all {
        sets.iter().any(|s| s.is_cracked_brick.contains(&tile_type))
    } else {
        sets[current].is_cracked_brick.contains(&tile_type)
    }
}

/// `IsConsideredPitTrapTile` (`:1851-1868`).
pub(crate) fn is_considered_pit_trap_tile(
    sets: &[DungeonTileSets],
    current: usize,
    tile_type: u16,
    all: bool,
) -> bool {
    if all {
        sets.iter().any(|s| s.is_pit_trap_tile.contains(&tile_type))
    } else {
        sets[current].is_pit_trap_tile.contains(&tile_type)
    }
}

/// `IsConsideredAnyDungeonWall` (`:1870-1892`) — wall/edge/glass union.
pub(crate) fn is_considered_any_dungeon_wall(
    sets: &[DungeonTileSets],
    current: usize,
    wall_type: u16,
    all: bool,
) -> bool {
    if wall_type > 0 && DUNGEON_WALLS.contains(&wall_type) {
        return true;
    }
    let hit = |s: &DungeonTileSets| {
        s.is_dungeon_wall.contains(&wall_type)
            || s.is_dungeon_wall_edge.contains(&wall_type)
            || s.is_dungeon_wall_glass.contains(&wall_type)
    };
    if all {
        sets.iter().any(hit)
    } else {
        hit(&sets[current])
    }
}

/// `IsConsideredDungeonWall` (`:1894-1915`).
pub(crate) fn is_considered_dungeon_wall(
    sets: &[DungeonTileSets],
    current: usize,
    wall_type: u16,
    all: bool,
) -> bool {
    if wall_type > 0 && DUNGEON_WALLS.contains(&wall_type) {
        return true;
    }
    if all {
        sets.iter().any(|s| s.is_dungeon_wall.contains(&wall_type))
    } else {
        sets[current].is_dungeon_wall.contains(&wall_type)
    }
}

/// `IsConsideredDungeonWallEdge` (`:1917-1938`).
pub(crate) fn is_considered_dungeon_wall_edge(
    sets: &[DungeonTileSets],
    current: usize,
    wall_type: u16,
    all: bool,
) -> bool {
    if wall_type > 0 && DUNGEON_WALLS.contains(&wall_type) {
        return true;
    }
    if all {
        sets.iter().any(|s| s.is_dungeon_wall_edge.contains(&wall_type))
    } else {
        sets[current].is_dungeon_wall_edge.contains(&wall_type)
    }
}

/// `IsConsideredDungeonWallGlass` (`:1940-1957`) — NO Main static arm.
pub(crate) fn is_considered_dungeon_wall_glass(
    sets: &[DungeonTileSets],
    current: usize,
    wall_type: u16,
    all: bool,
) -> bool {
    if all {
        sets.iter().any(|s| s.is_dungeon_wall_glass.contains(&wall_type))
    } else {
        sets[current].is_dungeon_wall_glass.contains(&wall_type)
    }
}

// --- FirstSolid + the room-object helpers -----------------------------------

/// `WorldGen.SolidTileAllowTopSlope` (`WorldGen.cs:70320-70338`): null →
/// true; `active && (solid || type == 380) && ((!platform &&
/// !bottomSlope) || (platform && halfBrick)) && !inActive`. NOTE the OOB
/// arm differs from the bottom-slope twin: there is no `InWorld` guard,
/// so an out-of-range index throws into the catch → FALSE (the port's
/// `tile()` None is exactly that arm; in-world cells are never null).
/// The port's `edit::kill` has no top-slope twin, so this is the verbatim
/// port; `cracked` applies the `SetCrackedBrickSolidity` override.
fn solid_allow_top_slope(world: &World, x: i32, y: i32, cracked: CrackedSolidity) -> bool {
    let Some(t) = world.tile(x, y) else {
        return false;
    };
    let platform = crate::tiledata::SET_PLATFORMS.has(t.kind);
    t.active()
        && (cracked.solid_kind(t.kind) || t.kind == 380)
        && ((!platform && !t.bottom_slope()) || (platform && t.half_brick()))
        && !t.inactive()
}

/// `WorldGen.SolidTileAllowBottomSlope` (`WorldGen.cs:70167-70189`) with
/// the cracked-solidity override: `!InWorld`/null → true; `active &&
/// (solid || solidTop) && (!topSlope || (platform &&
/// PlatformProperTopFrame)) && !halfBrick && !inActive`. (The
/// `edit::kill::solid_tile_allow_bottom_slope` twin drops the
/// PlatformProperTopFrame refinement and the inactive test; this slice
/// keeps both verbatim.)
fn solid_allow_bottom_slope(world: &World, x: i32, y: i32, cracked: CrackedSolidity) -> bool {
    let Some(t) = world.tile(x, y) else {
        return true;
    };
    t.active()
        && (cracked.solid_kind(t.kind) || crate::tiledata::TILE_SOLID_TOP.has(t.kind))
        && (!t.top_slope()
            || (crate::tiledata::SET_PLATFORMS.has(t.kind)
                && crate::torch::platform_proper_top_frame(t.frame_x)))
        && !t.half_brick()
        && !t.inactive()
}

/// `DungeonUtils.FirstSolid` (`DungeonUtils.cs:514-543`): the
/// `maxTilesX * 5` budget walk from `current` toward the ceiling/floor,
/// stopping at the first solid-allow-slope tile; the loop condition keeps
/// `y` in `(10, maxTilesY - 10)` and (when bounds are given) inside them.
/// Returns the walk's final point either way (the C# returns
/// `currentPoint` on budget exhaustion too).
pub(crate) fn first_solid(
    world: &World,
    ceiling: bool,
    mut point: (i32, i32),
    bounds: Option<&DungeonBounds>,
    cracked: CrackedSolidity,
) -> (i32, i32) {
    let mut budget = world.width * 5;
    loop {
        budget -= 1;
        if budget <= 0 {
            break;
        }
        if ceiling && solid_allow_top_slope(world, point.0, point.1, cracked) {
            return point;
        }
        if !ceiling && solid_allow_bottom_slope(world, point.0, point.1, cracked) {
            return point;
        }
        if ceiling {
            point.1 -= 1;
        } else {
            point.1 += 1;
        }
        if !(point.1 > 10 && point.1 < world.height - 10) {
            break;
        }
        if let Some(b) = bounds {
            if !b.contains(point.0, point.1) {
                break;
            }
        }
    }
    point
}

/// `WorldGen.PlaceTile(x, y, kind, mute: true)`'s object arm for the 1x1
/// dungeon objects (13/49/50) — the `tileobject` registry seam
/// (`skyblock.rs:1089`'s `place_object` pattern): `CanPlace`/`Place` draw
/// on `world.place_rng` (the C# `Main.rand`), NOT the pass stream.
fn place_object_1x1(world: &mut World, x: i32, y: i32, kind: u16) {
    let Some(def) = crate::tileobject::def_for_tile(kind) else {
        return;
    };
    let mut rand = std::mem::take(&mut world.place_rng);
    let instance = crate::tileobject::can_place(
        world,
        x,
        y,
        def,
        0,
        1,
        None,
        &mut rand,
        world.tile_object_random_cache,
    );
    world.place_rng = rand;
    if let Some(instance) = instance {
        crate::tileobject::place(world, &instance);
    }
}

/// `DungeonUtils.GenerateDungeonWaterCandle` (`DungeonUtils.cs:373-376`):
/// `PlaceTile(placeX, placeY, 49, mute: true)`.
pub(crate) fn generate_dungeon_water_candle(world: &mut World, x: i32, y: i32) {
    place_object_1x1(world, x, y, 49);
}

/// `DungeonUtils.GenerateDungeonPotionBottle` (`:378-392`): PlaceTile 13,
/// then (when the cell took type 13) a `genRand.Next(2)` frameX pick
/// 18/36. The frameX write happens even if the tile is somehow inactive
/// (the C# checks `.type == 13`, not `.active()` — verbatim).
pub(crate) fn generate_dungeon_potion_bottle(world: &mut World, r: &mut Rng, x: i32, y: i32) {
    place_object_1x1(world, x, y, 13);
    let Some(t) = world.tile(x, y).copied() else {
        return;
    };
    if t.kind == 13 {
        let fx = if r.range(0, 2) == 0 { 18 } else { 36 };
        if let Some(cell) = world.tile_mut(x, y) {
            cell.frame_x = fx;
        }
    }
}

/// `WorldGen.PlacePot` (`WorldGen.cs:54066-54114`) — the faithful port
/// (the `spider_gem_moss.rs:507` twin misses the remix centre veto and
/// the `halfBrick(false)` write; this one keeps both, cited there for the
/// merger to dedupe). `r` is `WorldGen.genRand` (the `Next(3) * 36`
/// variant draw, `:54097`).
pub(crate) fn place_pot(world: &mut World, r: &mut Rng, x: i32, y: i32, kind: u16, style: i32, remix: bool) -> bool {
    let mut flag = true;
    // `:54069-54072`: the remix centre band veto.
    if remix
        && (x as f64) > (world.width as f64) * 0.48
        && (x as f64) < (world.width as f64) * 0.52
        && y > world.height - 220
    {
        flag = false;
    }
    // `:54073-54094`: the 2x2 seat — the four object cells inactive, the
    // two floor cells nactive/solid/slopeless/not-half.
    for i in x..x + 2 {
        for j in (y - 1)..(y + 1) {
            if world.tile(i, j).is_some_and(|t| t.active()) {
                flag = false;
            }
        }
        let floor_ok = world.tile(i, y + 1).is_some_and(|t| {
            t.active()
                && !t.inactive()
                && !t.half_brick()
                && t.slope() == 0
                && crate::tiledata::TILE_SOLID.has(t.kind)
        });
        if !floor_ok {
            flag = false;
        }
    }
    if !flag {
        return false;
    }
    let num = r.range(0, 3) * 36;
    for k in 0..2 {
        for l in -1..1 {
            if let Some(t) = world.tile_mut(x + k, y + l) {
                t.flags |= FLAG_ACTIVE;
                t.frame_x = (k * 18 + num) as i16;
                t.frame_y = ((l + 1) * 18 + style * 36) as i16;
                t.kind = kind;
                t.flags &= !crate::tile::FLAG_HALF_BRICK; // `:54108` halfBrick(false)
            }
        }
    }
    true
}

/// `DungeonUtils.GenerateDungeonPot` (`DungeonUtils.cs:394-398`):
/// `PlacePot(x, y, 28, genRand.Next(10, 13))` + `SquareTileFrame`.
pub(crate) fn generate_dungeon_pot(world: &mut World, r: &mut Rng, x: i32, y: i32, remix: bool) {
    let style = r.range(10, 13);
    place_pot(world, r, x, y, 28, style, remix);
    crate::edit::square_tile_frame_gen_legacy(world, x, y);
}

/// `DungeonUtils.GenerateDungeonBook(placeX, placeY)` (`:400-403`) — the
/// `genRand.Next(50) == 0` waterbolt roll, then the two-arg form.
pub(crate) fn generate_dungeon_book(world: &mut World, r: &mut Rng, x: i32, y: i32) {
    let waterbolt = r.range(0, 50) == 0;
    generate_dungeon_book_waterbolt(world, x, y, waterbolt);
}

/// `DungeonUtils.GenerateDungeonBook(placeX, placeY, waterbolt)`
/// (`:405-413`): PlaceTile 50, then the waterbolt frameX 90 when
/// `placeY > (worldSurface + rockLayer) / 2` AND the cell took type 50.
///
/// VERBATIM C# QUIRK (`:409`): the type check reads
/// `Main.tile[placeY, placeY].type` — the Y coordinate indexed as BOTH
/// axes (a decompiled-source bug that ships in 1.4.5.8). The port keeps
/// it 1:1: the check reads the cell at `(y, y)`, while the frameX write
/// targets `(x, y)`. When `(y, y)` is out of world the C# would throw
/// inside `PlaceTile`'s caller context; the port's `tile()` returns None
/// → the check fails → no frameX write (the safe reading of the same
/// arm; cited deviation).
pub(crate) fn generate_dungeon_book_waterbolt(world: &mut World, x: i32, y: i32, waterbolt: bool) {
    place_object_1x1(world, x, y, 50);
    let depth_ok = (y as f64) > (world.world_surface + world.rock_layer) / 2.0;
    // The `Main.tile[placeY, placeY]` quirk — verbatim.
    let quirk_took_book = world.tile(y, y).is_some_and(|t| t.kind == 50);
    if waterbolt && depth_ok && quirk_took_book {
        if let Some(cell) = world.tile_mut(x, y) {
            cell.frame_x = 90;
        }
    }
}

// --- the shelf styles + the Bookshelves pass --------------------------------

/// `DungeonCrawler.SetupDungeonDataVariables`'s shelfStyles rolls
/// (`DungeonCrawler.cs:256-267`): three distinct `Next(9, 13)` picks with
/// the C#'s dedupe while-loops (slot 1 ≠ slot 0; slot 2 ≠ both).
pub(crate) fn roll_shelf_styles(r: &mut Rng) -> [i32; 3] {
    let mut shelf = [0i32; 3];
    shelf[0] = r.range(9, 13);
    shelf[1] = r.range(9, 13);
    while shelf[1] == shelf[0] {
        shelf[1] = r.range(9, 13);
    }
    shelf[2] = r.range(9, 13);
    while shelf[2] == shelf[0] || shelf[2] == shelf[1] {
        shelf[2] = r.range(9, 13);
    }
    shelf
}

/// `WorldGen.GetWorldSize` (`WorldGen.cs:6559-6570`): ≤4200 → 0,
/// ≤6400 → 1, else 2.
fn world_size(max_x: i32) -> i32 {
    if max_x <= 4200 {
        0
    } else if max_x <= 6400 {
        1
    } else {
        2
    }
}

/// The Bookshelves pass's inputs — the `DungeonData` fields it reads.
/// `bounds` is `data.dungeonBounds`; `wall_variants`/`shelf_styles` are
/// the `SetupDungeonDataVariables` rolls; `styles` is the dungeon's
/// `dungeonGenerationStyles` quad; `sets`/`current` back the
/// `IsConsidered*` calls (the pass uses the default `allDungeons: false`).
pub(crate) struct BookshelfData<'a> {
    pub bounds: &'a DungeonBounds,
    pub wall_variants: [u16; 3],
    pub shelf_styles: [i32; 3],
    pub styles: &'a [RoomStyle],
    pub sets: &'a [DungeonTileSets],
    pub current: usize,
    /// `data.Type == DungeonType.DualDungeon` — the water-candle minimum
    /// arm (`DungeonGlobalBookshelves.cs:31-39`).
    pub dual: bool,
    /// `data.CanGenerateFeatureAt`'s ported terms (the footprint's
    /// world-bounds + wall-350 gate; the entrance/protected-bounds terms
    /// have no port objects — the `dungeon_features` convention).
    pub footprint: &'a DungeonFootprint,
}

/// `DungeonGlobalBookshelves.Bookshelves` (`DungeonGlobalBookshelves.cs:
/// 23-207`). Returns the number of shelf runs placed (the C#'s `num3`),
/// for tests.
///
/// Ported arms: the 1000-attempt budget (`num2`), the `maxTilesX / 20`
/// run target, the horizontal walk to a dungeon-brick wall column, the
/// 7x7 platform-proximity veto, the 3-tall headroom test, the run-length
/// walk (`k`), the style-0 waterbolt-book flag (`flag3`), the
/// `GetPlatformStyle`/`GetBookshelfMinMaxSizes` overrides, the
/// wall-variant shelf-style pick, the per-shelf `TileFrame`, the inline
/// waterbolt book (`:131-139` — its `Next(50)` roll runs AFTER the
/// PlaceTile, unlike the helper's), and the bottle/candle tail
/// (`:144-196`) with the dual water-candle minimum (5/10/15 by world
/// size, `:31-38`).
///
/// Cited skips: `WorldGen.TileFrame(num5, num6)` (`:130`) — the port's
/// platform framing has no `TileFrame` twin for tile 19 (the
/// `edit::place_platform` legacy write keeps `frameY = 18 * style` and
/// `frameX` as-is; the C#'s platform merge-frame chain
/// `WorldGen.cs:85368-85471` is unported), so the shelf keeps its written
/// frame; the `data.dungeonEntrance` discard (`:26`) is a no-op.
pub(crate) fn bookshelves(world: &mut World, r: &mut Rng, data: &BookshelfData<'_>) -> i32 {
    let mut num = 0; // attempts since the last placed run
    let num2 = 1000; // the attempt budget
    let mut num3 = 0; // runs placed
    // `:30-39`: the dual water-candle minimum by world size.
    let mut num4 = if data.dual {
        match world_size(world.width) {
            1 => 10,
            2 => 15,
            _ => 5,
        }
    } else {
        -1
    };
    let is_wall = |w: u16| is_considered_dungeon_wall(data.sets, data.current, w, false);
    let is_tile = |t: u16| is_considered_dungeon_tile(data.sets, data.current, t, false);
    while num3 < world.width / 20 {
        num += 1;
        let mut num5 = r.range(data.bounds.left(), data.bounds.right());
        let num6 = r.range(data.bounds.top(), data.bounds.bottom());
        let mut flag = true;
        let wall0 = world.tile(num5, num6).map(|t| t.wall).unwrap_or(0);
        let active0 = world.tile(num5, num6).is_some_and(|t| t.active());
        if is_wall(wall0) && !active0 {
            // `:48-52`: the walk direction.
            let num7 = if r.range(0, 2) == 0 { -1 } else { 1 };
            // `:53-64`: walk until an active cell (or the world margin).
            while flag && !world.tile(num5, num6).is_some_and(|t| t.active()) {
                num5 -= num7;
                if num5 < 5 || num5 > world.width - 5 {
                    flag = false;
                } else {
                    let t = world.tile(num5, num6).copied();
                    if t.is_some_and(|t| t.active()) && !is_tile(t.unwrap().kind) {
                        flag = false;
                    }
                }
            }
            // `:65`: the wall column must be dungeon brick at y-1/y/y+1.
            let col_ok = |y: i32| {
                world
                    .tile(num5, y)
                    .is_some_and(|t| t.active() && is_tile(t.kind))
            };
            if flag && col_ok(num6) && col_ok(num6 - 1) && col_ok(num6 + 1) {
                num5 += num7;
                // `:68-78`: the 7x7 platform veto.
                'veto: for i in (num5 - 3)..=(num5 + 3) {
                    for j in (num6 - 3)..=(num6 + 3) {
                        if world
                            .tile(i, j)
                            .is_some_and(|t| t.active() && t.kind == 19)
                        {
                            flag = false;
                            break 'veto;
                        }
                    }
                }
                // `:79`: the 3-tall headroom (the C#'s non-short-circuit
                // `&` — all three reads happen; no side effects, so the
                // port's `&&` chain is observationally identical).
                let headroom = !world.tile(num5, num6 - 1).is_some_and(|t| t.active())
                    && !world.tile(num5, num6 - 2).is_some_and(|t| t.active())
                    && !world.tile(num5, num6 - 3).is_some_and(|t| t.active());
                if flag && headroom {
                    // `:81-85`: the CanGenerateFeatureAt veto — the C#
                    // sets flag = false (never read again; re-initialised
                    // next iteration) and `continue`s, SKIPPING the
                    // budget check at `:201`. The port keeps the
                    // continue verbatim.
                    if data.footprint.can_generate_at(world, num5, num6) {
                        // `:86-91`: the run-length walk.
                        let mut k = num5;
                        let num8 = num5;
                        while k > data.bounds.left()
                            && k < data.bounds.right()
                            && !world.tile(k, num6).is_some_and(|t| t.active())
                            && !world.tile(k, num6 - 1).is_some_and(|t| t.active())
                            && !world.tile(k, num6 + 1).is_some_and(|t| t.active())
                        {
                            k += num7;
                        }
                        let k = (num5 - k).abs();
                        let mut flag2 = true;
                        let mut flag3 = r.range(0, 2) == 0;
                        if k > 5 {
                            let mut num9 = -1i32;
                            let mut min = 1;
                            let mut max = 4;
                            let wall_here =
                                world.tile(num5, num6).map(|t| t.wall).unwrap_or(0);
                            if let Some(idx) = style_for_wall(data.styles, wall_here) {
                                let s = style_at(data.styles, idx);
                                flag2 = s.style == 0;
                                if !flag2 {
                                    flag3 = false;
                                }
                                num9 = get_platform_style(s, r);
                                if let Some((mn, mx)) = s.bookshelf_min_max {
                                    min = mn;
                                    max = mx;
                                }
                            }
                            // `:110-141`: the shelf run.
                            let mut num10 = r.range(min, max);
                            while num10 > 0 {
                                num10 -= 1;
                                let wall_here =
                                    world.tile(num5, num6).map(|t| t.wall).unwrap_or(0);
                                if let Some(cell) = world.tile_mut(num5, num6) {
                                    cell.flags |= FLAG_ACTIVE;
                                    // Clear(TileDataType.Slope) (Tile.cs:773-777):
                                    // slope(0) AND halfBrick(false).
                                    cell.set_slope(0);
                                    cell.flags &= !crate::tile::FLAG_HALF_BRICK;
                                    cell.kind = 19;
                                }
                                let mut num11 = data.shelf_styles[0];
                                if wall_here == data.wall_variants[1] {
                                    num11 = data.shelf_styles[1];
                                }
                                if wall_here == data.wall_variants[2] {
                                    num11 = data.shelf_styles[2];
                                }
                                if num9 > -1 {
                                    num11 = num9;
                                }
                                if let Some(cell) = world.tile_mut(num5, num6) {
                                    cell.frame_y = (18 * num11) as i16;
                                }
                                // `Main.tileFrameImportant[19]` (Main.cs:7654)
                                // — the platform keeps its style frame through
                                // the world re-frame (edit/place.rs:376's twin).
                                if world.frame_important.len() > 19 {
                                    world.frame_important[19] = true;
                                }
                                // `:130` WorldGen.TileFrame — cited skip (doc).
                                if flag3 {
                                    // `:131-139`: the inline waterbolt book —
                                    // PlaceTile FIRST, then the Next(50) roll
                                    // (the helper's order is roll-first; this
                                    // arm is verbatim its own).
                                    place_object_1x1(world, num5, num6 - 1, 50);
                                    let roll = r.range(0, 50) == 0;
                                    let depth_ok = (num6 as f64)
                                        > (world.world_surface + world.rock_layer) / 2.0;
                                    let took = world
                                        .tile(num5, num6 - 1)
                                        .is_some_and(|t| t.kind == 50);
                                    if roll && depth_ok && took {
                                        if let Some(cell) = world.tile_mut(num5, num6 - 1) {
                                            cell.frame_x = 90;
                                        }
                                    }
                                }
                                num5 += num7;
                            }
                            num = 0;
                            num3 += 1;
                            // `:144-196`: the bottle/candle tail.
                            if !flag3 && r.range(0, 2) == 0 {
                                num5 = num8;
                                let ty = num6 - 1;
                                if flag2 {
                                    let mut num12 = if r.range(0, 4) == 0 { 1 } else { 0 };
                                    if num4 > 0 {
                                        num12 = 1;
                                    }
                                    let kind = match num12 {
                                        0 => 13,
                                        _ => 49,
                                    };
                                    place_object_1x1(world, num5, ty, kind);
                                    let t = world.tile(num5, ty).copied();
                                    if t.is_some_and(|t| t.kind == 13) {
                                        let fx = if r.range(0, 2) == 0 { 18 } else { 36 };
                                        if let Some(cell) = world.tile_mut(num5, ty) {
                                            cell.frame_x = fx;
                                        }
                                    }
                                    if t.is_some_and(|t| t.active() && t.kind == 49) {
                                        num4 -= 1;
                                    }
                                } else {
                                    place_object_1x1(world, num5, ty, 13);
                                    if world.tile(num5, ty).is_some_and(|t| t.kind == 13) {
                                        let fx = if r.range(0, 2) == 0 { 18 } else { 36 };
                                        if let Some(cell) = world.tile_mut(num5, ty) {
                                            cell.frame_x = fx;
                                        }
                                    }
                                }
                            }
                        }
                    } else {
                        // `:83-84`: flag = false; continue — the budget
                        // check below is skipped on this arm.
                        continue;
                    }
                }
            }
        }
        // `:201-205`: the budget exhaustion counts as a run.
        if num > num2 {
            num = 0;
            num3 += 1;
        }
    }
    num3
}

// --- the GroundFurniture selection layer (R2) --------------------------------
//
// `DungeonGlobalGroundFurniture` (`DungeonGlobalGroundFurniture.cs:34-874`)
// splits into the room/hall placement loops (they need the port's missing
// `DungeonRoom`/`DungeonHall` object lists — the wiring round's seam) and
// the PURE selection layer ported here: the wall-keyed default item rows
// (`:275-340`), the style item-list substitution
// (`GroundFurniture_GetFurnitureItem`, `:862-873`), the `Next(13)` kind
// ladder with its re-roll arms (`:363-371`), the area-size tables
// (`:372-506`), and the budget scalars (`:37-63/:191-218`). The placement
// tail (`:555-857`, the PlaceTile/Place4x2 arms + the candle/book/pot
// scatter) consumes these products; it rides the room/hall wiring.

/// The furniture item rows `GroundFurniture_ActuallyGenerateFurniture`
/// selects from (`DungeonGlobalGroundFurniture.cs:275-340`) — the C#'s
/// num4..num17 locals, named by their ladder slot:
/// num18=0 table(+chair), 1 workbench(+chair), 2 vase/statue, 3 bookcase,
/// 4 chair, 5 bed, 6 piano, 7 dresser, 8 sofa, 9 bathtub, 10 lamp,
/// 11 candelabra, 12 clock. `candle` (num7) also feeds the on-table
/// scatter (`:612-647/:692-728`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct FurnitureItems {
    pub chair: i32,
    pub table: i32,
    pub workbench: i32,
    pub candle: i32,
    pub vase_or_statue: i32,
    pub bookcase: i32,
    pub bed: i32,
    pub piano: i32,
    pub dresser: i32,
    pub sofa: i32,
    pub bathtub: i32,
    pub lamp: i32,
    pub candelabra: i32,
    pub clock: i32,
}

/// The default rows (`:275-288`) — the blue-dungeon items.
const FURNITURE_BLUE: FurnitureItems = FurnitureItems {
    chair: 1396,
    table: 1397,
    workbench: 1398,
    candle: 1405,
    vase_or_statue: 1408,
    bookcase: 1414,
    bed: 1470,
    piano: 2376,
    dresser: 2386,
    sofa: 2402,
    bathtub: 2658,
    lamp: 2664,
    candelabra: 2645,
    clock: 3900,
};

/// `switch (wallType)` (`:289-323`) + the 94..105 painted-wall override
/// (`:324-340`). The override reads the CELL's wall (`Main.tile[num3,
/// j].wall`), not the passed `wallType` — the port takes both verbatim.
pub(crate) fn furniture_defaults(wall_type: u16, cell_wall: u16) -> FurnitureItems {
    let mut items = match wall_type {
        // `case 8` (`:291-306`) — green.
        8 => FurnitureItems {
            chair: 1399,
            table: 1400,
            workbench: 1401,
            candle: 1406,
            vase_or_statue: 1409,
            bookcase: 1415,
            bed: 1471,
            piano: 2377,
            dresser: 2387,
            sofa: 2403,
            bathtub: 2659,
            lamp: 2665,
            candelabra: 2646,
            clock: 3901,
        },
        // `case 9` (`:307-322`) — pink.
        9 => FurnitureItems {
            chair: 1402,
            table: 1403,
            workbench: 1404,
            candle: 1407,
            vase_or_statue: 1410,
            bookcase: 1416,
            bed: 1472,
            piano: 2378,
            dresser: 2388,
            sofa: 2404,
            bathtub: 2660,
            lamp: 2666,
            candelabra: 2647,
            clock: 3902,
        },
        _ => FURNITURE_BLUE,
    };
    // `:324-340`: the painted-wall band wins over the colour rows.
    if (94..=105).contains(&cell_wall) {
        items = FurnitureItems {
            chair: 1509,
            table: 1510,
            workbench: 1511,
            candle: 5743,
            vase_or_statue: -1,
            bookcase: 1512,
            bed: 5740,
            piano: 5750,
            dresser: 5741,
            sofa: 5753,
            bathtub: 5739,
            lamp: 5742,
            candelabra: 5748,
            clock: 5746,
        };
    }
    items
}

/// The style item lists `GroundFurniture_ActuallyGenerateFurniture`
/// substitutes through `GetFurnitureItem` (`:348-361`) — the
/// `DungeonGenerationStyleData` rows, in the C#'s substitution order
/// (table, workbench, candle, vase, bookcase, chair, bed, piano, dresser,
/// sofa, bathtub, lamp, candelabra, clock — NOTE lamp BEFORE candelabra,
/// `:359-360`). `None` is the C# null (the item becomes −1); `Some(&[])`
/// is `new int[0]` (the default item survives).
#[derive(Clone, Copy, Debug)]
pub(crate) struct FurnitureStyleItems {
    pub table: Option<&'static [i32]>,
    pub workbench: Option<&'static [i32]>,
    pub candle: Option<&'static [i32]>,
    pub vase_or_statue: Option<&'static [i32]>,
    pub bookcase: Option<&'static [i32]>,
    pub chair: Option<&'static [i32]>,
    pub bed: Option<&'static [i32]>,
    pub piano: Option<&'static [i32]>,
    pub dresser: Option<&'static [i32]>,
    pub sofa: Option<&'static [i32]>,
    pub bathtub: Option<&'static [i32]>,
    pub lamp: Option<&'static [i32]>,
    pub candelabra: Option<&'static [i32]>,
    pub clock: Option<&'static [i32]>,
}

/// `GroundFurniture_GetFurnitureItem` (`DungeonGlobalGroundFurniture.cs:
/// 862-873`): null → −1 (NO draw); empty or style 0 → the default (NO
/// draw); else `items[genRand.Next(len)]`.
pub(crate) fn get_furniture_item(
    style_id: u8,
    r: &mut Rng,
    default_item: i32,
    items: Option<&'static [i32]>,
) -> i32 {
    let Some(items) = items else {
        return -1;
    };
    if items.is_empty() || style_id == 0 {
        return default_item;
    }
    items[r.range(0, items.len() as i32) as usize]
}

/// The full substitution chain (`:344-362`): when `GetStyleForWall`
/// resolved a style, `flag`/`flag2` become `Style == 0` and every item
/// row rides `GetFurnitureItem` in the C#'s order. Returns the items and
/// the style-0 flag (the placement tail's `flag`/`flag2` — both are the
/// same value here, `:346-347`).
pub(crate) fn furniture_items_for_style(
    defaults: FurnitureItems,
    style_id: u8,
    lists: &FurnitureStyleItems,
    r: &mut Rng,
) -> (FurnitureItems, bool) {
    let style0 = style_id == 0;
    let mut g = |d: i32, l: Option<&'static [i32]>| get_furniture_item(style_id, r, d, l);
    let items = FurnitureItems {
        table: g(defaults.table, lists.table),
        workbench: g(defaults.workbench, lists.workbench),
        candle: g(defaults.candle, lists.candle),
        vase_or_statue: g(defaults.vase_or_statue, lists.vase_or_statue),
        bookcase: g(defaults.bookcase, lists.bookcase),
        chair: g(defaults.chair, lists.chair),
        bed: g(defaults.bed, lists.bed),
        piano: g(defaults.piano, lists.piano),
        dresser: g(defaults.dresser, lists.dresser),
        sofa: g(defaults.sofa, lists.sofa),
        bathtub: g(defaults.bathtub, lists.bathtub),
        // `:359-360`: num16 (lamp) substitutes BEFORE num15 (candelabra).
        lamp: g(defaults.lamp, lists.lamp),
        candelabra: g(defaults.candelabra, lists.candelabra),
        clock: g(defaults.clock, lists.clock),
    };
    (items, style0)
}

/// The style item lists of the dual quads' statics
/// (`DungeonGenerationStyles.cs` — the same rows c32's tables carry; the
/// merger dedupes). Only the styles the dual dungeons generate are
/// listed; the lookup falls back to no substitution for anything else.
pub(crate) fn furniture_style_items(style_id: u8) -> Option<FurnitureStyleItems> {
    let lists = match style_id {
        // Cavern (`:326-342`).
        1 => FurnitureStyleItems {
            table: Some(&[32, 5894]),
            workbench: Some(&[36, 5896]),
            candle: Some(&[105, 713, 5883]),
            vase_or_statue: None,
            bookcase: Some(&[354, 5881]),
            chair: Some(&[34, 5884]),
            bed: Some(&[224, 5880]),
            piano: Some(&[333, 5891]),
            dresser: Some(&[334, 5888]),
            sofa: Some(&[2397, 5893]),
            bathtub: Some(&[336, 5879]),
            lamp: Some(&[342, 5889]),
            candelabra: Some(&[349, 714, 5882]),
            clock: Some(&[359, 5887]),
        },
        // Snow (`:373-389`).
        2 => FurnitureStyleItems {
            table: Some(&[2248, 5815]),
            workbench: Some(&[2252, 5817]),
            candle: Some(&[2049, 5802]),
            vase_or_statue: None,
            bookcase: Some(&[2031, 5800]),
            chair: Some(&[2288, 5803]),
            bed: Some(&[2068, 5799]),
            piano: Some(&[2247, 5811]),
            dresser: Some(&[3913, 5808]),
            sofa: Some(&[2635, 5814]),
            bathtub: Some(&[2076, 5798]),
            lamp: Some(&[2086, 5809]),
            candelabra: Some(&[2100, 5801]),
            clock: Some(&[2594, 5806]),
        },
        // Desert (`:414-430`).
        3 => FurnitureStyleItems {
            table: Some(&[4314]),
            workbench: Some(&[4315]),
            candle: Some(&[4303]),
            vase_or_statue: None,
            bookcase: Some(&[4300]),
            chair: Some(&[4304]),
            bed: Some(&[4299]),
            piano: Some(&[4310]),
            dresser: Some(&[4301]),
            sofa: Some(&[4313]),
            bathtub: Some(&[4298]),
            lamp: Some(&[4308]),
            candelabra: Some(&[4302]),
            clock: Some(&[4306]),
        },
        // Corruption (`:455-470`).
        4 => FurnitureStyleItems {
            table: Some(&[638, 3974, 5773]),
            workbench: Some(&[635, 3975, 5775]),
            candle: Some(&[2046, 3962, 5760]),
            vase_or_statue: None,
            bookcase: Some(&[2021, 3960, 5758]),
            chair: Some(&[628, 3963, 5761]),
            bed: Some(&[644, 3959, 5757]),
            piano: Some(&[641, 3971, 5769]),
            dresser: Some(&[647, 3968, 5766]),
            sofa: Some(&[2398, 3973, 5772]),
            bathtub: Some(&[2073, 3958, 5756]),
            lamp: Some(&[2083, 3969, 5767]),
            candelabra: Some(&[2093, 3961, 5759]),
            clock: Some(&[2593, 3966, 5764]),
        },
        // Crimson (`:495-510`).
        5 => FurnitureStyleItems {
            table: Some(&[917, 828, 5794]),
            workbench: Some(&[916, 813, 5796]),
            candle: Some(&[2154, 2047, 5781]),
            vase_or_statue: None,
            bookcase: Some(&[2136, 2022, 5779]),
            chair: Some(&[915, 809, 5782]),
            bed: Some(&[920, 2067, 5778]),
            piano: Some(&[919, 2246, 5790]),
            dresser: Some(&[918, 2640, 5787]),
            sofa: Some(&[2401, 2634, 5793]),
            bathtub: Some(&[2127, 2074, 5777]),
            lamp: Some(&[2132, 2084, 5788]),
            candelabra: Some(&[2150, 2094, 5780]),
            clock: Some(&[2604, 2598, 5785]),
        },
        // Hallow (`:574-589`).
        6 => FurnitureStyleItems {
            table: Some(&[640, 3920]),
            workbench: Some(&[637, 3909]),
            candle: Some(&[2051, 3890]),
            vase_or_statue: None,
            bookcase: Some(&[2027, 3917]),
            chair: Some(&[630, 3889]),
            bed: Some(&[646, 3897]),
            piano: Some(&[643, 3915]),
            dresser: Some(&[649, 3911]),
            sofa: Some(&[2400, 3918]),
            bathtub: Some(&[2078, 3895]),
            lamp: Some(&[2088, 3892]),
            candelabra: Some(&[2099, 3893]),
            clock: Some(&[2602, 3898]),
        },
        // Jungle (`:736-751`).
        8 => FurnitureStyleItems {
            table: Some(&[639]),
            workbench: Some(&[636]),
            candle: Some(&[2050]),
            vase_or_statue: None,
            bookcase: Some(&[2026]),
            chair: Some(&[629]),
            bed: Some(&[645]),
            piano: Some(&[642]),
            dresser: Some(&[648]),
            sofa: Some(&[2399]),
            bathtub: Some(&[2077]),
            lamp: Some(&[2087]),
            candelabra: Some(&[2098]),
            clock: Some(&[2597]),
        },
        // Beehive (`:655-670`).
        9 => FurnitureStyleItems {
            table: Some(&[1717]),
            workbench: Some(&[2251]),
            candle: Some(&[2648]),
            vase_or_statue: None,
            bookcase: Some(&[2023]),
            chair: Some(&[1707]),
            bed: Some(&[1721]),
            piano: Some(&[2255]),
            dresser: Some(&[2395]),
            sofa: Some(&[2411]),
            bathtub: Some(&[2124]),
            lamp: Some(&[2129]),
            candelabra: Some(&[2095]),
            clock: Some(&[2240]),
        },
        // Temple (`:779-794`) — the only style with a vase/statue row.
        10 => FurnitureStyleItems {
            table: Some(&[1144]),
            workbench: Some(&[1145]),
            candle: Some(&[2052]),
            vase_or_statue: Some(&[1152, 1153, 1154]),
            bookcase: Some(&[2030]),
            chair: Some(&[1143]),
            bed: Some(&[2069]),
            piano: Some(&[2385]),
            dresser: Some(&[2396]),
            sofa: Some(&[2416]),
            bathtub: Some(&[2079]),
            lamp: Some(&[2089]),
            candelabra: Some(&[2101]),
            clock: Some(&[2595]),
        },
        // Shimmer (`:210-225`).
        11 => FurnitureStyleItems {
            table: Some(&[5565]),
            workbench: Some(&[5566]),
            candle: Some(&[5553]),
            vase_or_statue: None,
            bookcase: Some(&[5550]),
            chair: Some(&[5554]),
            bed: Some(&[5549]),
            piano: Some(&[5561]),
            dresser: Some(&[5551]),
            sofa: Some(&[5564]),
            bathtub: Some(&[5548]),
            lamp: Some(&[5559]),
            candelabra: Some(&[5552]),
            clock: Some(&[5557]),
        },
        // Spider (`:249-264`).
        12 => FurnitureStyleItems {
            table: Some(&[32]),
            workbench: Some(&[36]),
            candle: Some(&[105, 713]),
            vase_or_statue: None,
            bookcase: Some(&[354]),
            chair: Some(&[34]),
            bed: Some(&[224]),
            piano: Some(&[333]),
            dresser: Some(&[334]),
            sofa: Some(&[2397]),
            bathtub: Some(&[336]),
            lamp: Some(&[342]),
            candelabra: Some(&[349, 714]),
            clock: Some(&[359]),
        },
        // LivingWood (`:288-303`) and LivingMahogany (`:695-710`) — the
        // same furniture rows (both LivingWoodStyleData).
        13 | 14 => FurnitureStyleItems {
            table: Some(&[829]),
            workbench: Some(&[2633]),
            candle: Some(&[2153]),
            vase_or_statue: None,
            bookcase: Some(&[2135]),
            chair: Some(&[806]),
            bed: Some(&[2139]),
            piano: Some(&[2245]),
            dresser: Some(&[3914]),
            sofa: Some(&[2636]),
            bathtub: Some(&[2126]),
            lamp: Some(&[2131]),
            candelabra: Some(&[2149]),
            clock: Some(&[2596]),
        },
        // Crystal (`:534-549`).
        15 => FurnitureStyleItems {
            table: Some(&[3920]),
            workbench: Some(&[3909]),
            candle: Some(&[3890]),
            vase_or_statue: None,
            bookcase: Some(&[3917]),
            chair: Some(&[3889]),
            bed: Some(&[3897]),
            piano: Some(&[3915]),
            dresser: Some(&[3911]),
            sofa: Some(&[3918]),
            bathtub: Some(&[3895]),
            lamp: Some(&[3892]),
            candelabra: Some(&[3893]),
            clock: Some(&[3898]),
        },
        _ => return None,
    };
    Some(lists)
}

/// The `Next(13)` furniture-kind ladder (`:363-371`): the initial draw,
/// the 10/11/12 re-roll (`Next(4) != 0` → redraw), then the while-loop
/// that rejects kinds whose item is −1 (2 vase, 5 bed, 6 piano,
/// 7 dresser, 8 sofa, 9 bathtub, 10/11/12 the lamp/candelabra/clock
/// trio). Kinds 0/1/3/4 are never rejected (their items are always
/// present in the default rows).
///
/// VERBATIM C# QUIRK (`:368` vs `:831/:840`): the rejection loop checks
/// kind 10 against num15 (the CANDELABRA) and kind 11 against num16 (the
/// LAMP), while the placement arms place kind 10 → num16 (lamp) and kind
/// 11 → num15 (candelabra) — the loop's two checks are SWAPPED relative
/// to the placement. The port keeps the swap 1:1.
pub(crate) fn roll_furniture_kind(items: &FurnitureItems, r: &mut Rng) -> i32 {
    let mut num18 = r.range(0, 13);
    if (num18 == 10 || num18 == 11 || num18 == 12) && r.range(0, 4) != 0 {
        num18 = r.range(0, 13);
    }
    while (num18 == 2 && items.vase_or_statue == -1)
        || (num18 == 5 && items.bed == -1)
        || (num18 == 6 && items.piano == -1)
        || (num18 == 7 && items.dresser == -1)
        || (num18 == 8 && items.sofa == -1)
        || (num18 == 9 && items.bathtub == -1)
        // `:368`: kind 10 checks num15 (candelabra), kind 11 num16 (lamp)
        // — the swapped pair, verbatim (the placement arms are the other
        // way round, `:831-848`).
        || (num18 == 10 && items.candelabra == -1)
        || (num18 == 11 && items.lamp == -1)
        || (num18 == 12 && items.clock == -1)
    {
        num18 = r.range(0, 13);
    }
    num18
}

/// The furniture area half-sizes (`:372-506`): `(num19, num20)` per kind
/// — the regular table, then the `noRegularFurnitureAreaChecks` override
/// table (the hall fallback arm, `:163/:171`). Index = the ladder kind.
const FURNITURE_AREA: [(i32, i32); 13] = [
    (5, 4), // 0 table
    (4, 3), // 1 workbench
    (3, 5), // 2 vase/statue
    (4, 6), // 3 bookcase
    (3, 3), // 4 chair
    (5, 3), // 5 bed
    (5, 4), // 6 piano
    (5, 4), // 7 dresser
    (5, 4), // 8 sofa
    (5, 3), // 9 bathtub
    (2, 4), // 10 lamp
    (3, 3), // 11 candelabra
    (2, 5), // 12 clock
];

/// The `noRegularFurnitureAreaChecks` table (`:439-506`).
const FURNITURE_AREA_NO_CHECKS: [(i32, i32); 13] = [
    (3, 4),
    (2, 3),
    (3, 5),
    (3, 6),
    (1, 3),
    (4, 3),
    (4, 4),
    (4, 4),
    (4, 4),
    (4, 3),
    (1, 4),
    (2, 3),
    (2, 5),
];

/// The area lookup — kind −1 (the area-check veto, `:532/:553`) has no
/// row; the C# leaves num19/num20 at 0, so the port returns (0, 0).
pub(crate) fn furniture_area(kind: i32, no_regular_checks: bool) -> (i32, i32) {
    if !(0..13).contains(&kind) {
        return (0, 0);
    }
    if no_regular_checks {
        FURNITURE_AREA_NO_CHECKS[kind as usize]
    } else {
        FURNITURE_AREA[kind as usize]
    }
}

/// The area-width gate (`:546-554`): `num22 = num19 * 1.75f` (the
/// regular arm) or `num19` (no-checks); a run narrower than that vetoes
/// the kind (`num18 = -1`). f32 arithmetic verbatim.
pub(crate) fn furniture_width_gate(run_width: i32, half_width: i32, no_regular_checks: bool) -> bool {
    let num22 = if no_regular_checks {
        half_width as f32
    } else {
        half_width as f32 * 1.75f32
    };
    (run_width as f32) >= num22
}

/// The dual room/hall budgets (`GroundFurniture_DualDungeons`,
/// `:37-63`): `num = maxTilesX / 4200f`; alch/bewitch =
/// `(1 + (int)num) * globalFeatureScalar` (the f32→int truncation inside
/// the sum is verbatim — `(float)(int)num`, not `num`); both zero when
/// no style-0 is in the quad (`:40-53`); the water-candle minimum
/// 5/10/15 by world size (`:54-63`). `defaultCount` 4 (rooms, `:64`) /
/// 6 (halls, `:65`) — `GetFurnitureCount`'s base returns the default
/// (`DungeonRoom.cs:261-264`, `DungeonHall.cs:54-57`).
pub(crate) fn dual_furniture_budgets(
    max_x: i32,
    global_feature_scalar: f64,
    has_style0: bool,
) -> (i32, i32, i32) {
    let num = max_x as f32 / 4200f32;
    let mut alch = ((1f32 + (num as i32) as f32) as f64 * global_feature_scalar) as i32;
    let mut bewitch = alch;
    if !has_style0 {
        alch = 0;
        bewitch = 0;
    }
    let minimum_water_candles = match world_size(max_x) {
        1 => 10,
        2 => 15,
        _ => 5,
    };
    (alch, bewitch, minimum_water_candles)
}

/// The legacy `GroundFurniture` budgets (`:191-218`): the attempt count
/// `(2000 * num) * scalar`, the same alch/bewitch/style-0 gate, and the
/// water-candle minimum (−1 for a non-dual dungeon — the `:209-218` arm
/// only sets it under DualDungeon).
pub(crate) fn legacy_furniture_budgets(
    max_x: i32,
    global_feature_scalar: f64,
    has_style0: bool,
    dual: bool,
) -> (i32, i32, i32, i32) {
    let num = max_x as f32 / 4200f32;
    let attempts = ((2000f32 * num) as f64 * global_feature_scalar) as i32;
    let (alch, bewitch, mut candles) = dual_furniture_budgets(max_x, global_feature_scalar, has_style0);
    if !dual {
        candles = -1;
    }
    (attempts, alch, bewitch, candles)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The dual set clones OR in every style + sub-style row
    /// (`DungeonCrawler.cs:139-183`); the classic build keeps the statics.
    #[test]
    fn w4_f24_dual_set_clones_are_verbatim() {
        let quad0 = vec![
            CAVERN,
            CORRUPTION,
            JUNGLE,
            current_dungeon_style(41, 481, 7, 91, 96, 8),
        ];
        let classic = DungeonTileSets::build(&quad0, false);
        assert_eq!(classic.is_dungeon_tile, vec![41, 43, 44, 677, 678, 679]);
        assert_eq!(classic.is_cracked_brick, vec![481, 482, 483]);
        assert_eq!(classic.is_dungeon_wall_glass, WALL_GLASS.to_vec());
        let dual = DungeonTileSets::build(&quad0, true);
        // Cavern's sub-styles ride the walk: Shimmer 667/322, Spider
        // 156/62, LivingWood 191/244 (+ cracked 192).
        for t in [38, 667, 156, 191, 25, 59, 60, 41] {
            assert!(dual.is_dungeon_tile.contains(&t), "tile {t}");
        }
        assert!(dual.is_cracked_brick.contains(&192)); // LivingWood
        assert!(dual.is_cracked_brick.contains(&112)); // Corruption
        // Jungle's extras walls + Beehive/LivingMahogany rows.
        for w in [349, 322, 62, 244, 3, 188, 64, 204, 15, 86, 7] {
            assert!(dual.is_dungeon_wall.contains(&w), "wall {w}");
        }
        // Window glass: Cavern 21/4, Shimmer 93/149, Jungle 91/42,
        // Beehive 89/172, current 91/96.
        for w in [21, 4, 93, 149, 91, 42, 89, 172, 96] {
            assert!(dual.is_dungeon_wall_glass.contains(&w), "glass {w}");
        }
        // Edge walls: Cavern 5, Spider 36, LivingWood 196, Corruption 33,
        // Jungle 24, Beehive 151, current 8.
        for w in [5, 36, 196, 33, 24, 151, 8] {
            assert!(dual.is_dungeon_wall_edge.contains(&w), "edge {w}");
        }
        // Pit traps: the cracked triple + 112 (Corruption) + 123 rows.
        assert!(dual.is_pit_trap_tile.contains(&112));
        assert!(dual.is_pit_trap_tile.contains(&481));
    }

    /// The IsConsidered* family's Main-static arms and allDungeons walks
    /// (`DungeonUtils.cs:1809-1957`).
    #[test]
    fn w4_f24_is_considered_family_matches_the_statics() {
        let quad1 = vec![SNOW, DESERT, HALLOW, TEMPLE];
        let sets = vec![
            DungeonTileSets::build(&[CAVERN], true),
            DungeonTileSets::build(&quad1, true),
        ];
        // Main.tileDungeon arm: 677 is a dungeon tile even with no sets.
        assert!(is_considered_dungeon_tile(&sets, 0, 677, false));
        assert!(!is_considered_dungeon_tile(&sets, 0, 0, false)); // the >0 guard
        // Snow's 161 is only in dungeon 1's sets.
        assert!(!is_considered_dungeon_tile(&sets, 0, 161, false));
        assert!(is_considered_dungeon_tile(&sets, 1, 161, false));
        assert!(is_considered_dungeon_tile(&sets, 0, 161, true));
        // Cracked: no Main arm — 481 only via the cloned sets.
        assert!(is_considered_cracked_dungeon_tile(&sets, 0, 481, false));
        assert!(is_considered_cracked_dungeon_tile(&sets, 0, 224, true)); // Snow
        assert!(!is_considered_cracked_dungeon_tile(&sets, 0, 224, false));
        // Wall glass: Hallow's Crystal sub-style adds 88/43.
        assert!(is_considered_dungeon_wall_glass(&sets, 1, 88, false));
        // Any-wall union: edge 31 (Snow) is not isDungeonWall but IS any.
        assert!(!is_considered_dungeon_wall(&sets, 1, 31, false));
        assert!(is_considered_any_dungeon_wall(&sets, 1, 31, false));
        // Main.wallDungeon arm: 94 passes every wall predicate.
        assert!(is_considered_dungeon_wall(&sets, 0, 94, false));
        assert!(is_considered_dungeon_wall_edge(&sets, 0, 94, false));
    }

    /// The cracked-chance roll: the 0.0 pins draw NOTHING; the default
    /// 0.166 and the dual 1.0 draw one NextDouble each
    /// (`LegacyDungeonHall.cs:79-82`, `DungeonHallSettings.cs:15`).
    #[test]
    fn w4_f24_cracked_chance_roll_pins_and_stream() {
        let mut a = HallSeed::new(7);
        let mut b = HallSeed::new(7);
        assert!(!roll_cracked_brick(&mut a, 0.0));
        // The 0.0 pin advanced no stream: the next draw matches untouched.
        assert_eq!(a.next(1000), b.next(1000));
        // chance 1.0 always cracks (NextDouble <= 1.0), one draw.
        let mut c = HallSeed::new(7);
        assert!(roll_cracked_brick(&mut c, 1.0));
        // The default chance is the C# constant.
        assert_eq!(CRACKED_BRICK_CHANCE, 0.166);
        // Determinism: same seed, same roll sequence.
        let roll_seq = || {
            let mut r = HallSeed::new(99);
            (0..8).map(|_| roll_cracked_brick(&mut r, CRACKED_BRICK_CHANCE)).collect::<Vec<_>>()
        };
        assert_eq!(roll_seq(), roll_seq());
    }

    /// The solidity override flips only the cracked triple
    /// (`WorldGen.cs:11022-11027`).
    #[test]
    fn w4_f24_cracked_solidity_flips_only_the_triple() {
        let off = CrackedSolidity(false);
        let on = CrackedSolidity(true);
        for t in [481u16, 482, 483] {
            assert!(!off.solid_kind(t));
            assert!(on.solid_kind(t));
        }
        // Non-cracked kinds read the static table either way.
        assert_eq!(off.solid_kind(1), on.solid_kind(1));
        assert!(on.solid_kind(1)); // dirt is solid
        // (Platforms ARE in TILE_SOLID — Main.cs:7893 sets tileSolid[19];
        // a bottle is not.)
        assert!(!on.solid_kind(13));
    }

    /// FirstSolid walks to the floor/ceiling within the budget and bounds
    /// (`DungeonUtils.cs:514-543`).
    #[test]
    fn w4_f24_first_solid_walks_to_the_floor() {
        let mut w = World::new("first-solid".into(), 200, 200);
        // A dirt floor at y=100 under an empty column.
        for x in 0..200 {
            if let Some(t) = w.tile_mut(x, 100) {
                t.kind = 1;
                t.flags |= FLAG_ACTIVE;
            }
        }
        let got = first_solid(&w, false, (50, 90), None, CrackedSolidity(true));
        assert_eq!(got, (50, 100));
        // Ceiling walk finds the same slab from below… no — from above:
        let got = first_solid(&w, true, (50, 110), None, CrackedSolidity(true));
        assert_eq!(got, (50, 100));
        // A cracked-brick floor with solidity OFF is walked through.
        if let Some(t) = w.tile_mut(60, 100) {
            t.kind = 481;
        }
        let got = first_solid(&w, false, (60, 90), None, CrackedSolidity(false));
        assert_ne!(got.1, 100); // walked past the non-solid cracked brick
        let got = first_solid(&w, false, (60, 90), None, CrackedSolidity(true));
        assert_eq!(got, (60, 100));
    }

    /// The shelf-style rolls are three distinct Next(9,13) picks
    /// (`DungeonCrawler.cs:256-267`); a == b replay.
    #[test]
    fn w4_f24_shelf_styles_are_distinct_and_deterministic() {
        let roll = || {
            let mut r = Rng::from_seed(7);
            roll_shelf_styles(&mut r)
        };
        let a = roll();
        assert_eq!(a, roll());
        assert!(a.iter().all(|&s| (9..13).contains(&s)));
        assert_ne!(a[0], a[1]);
        assert_ne!(a[2], a[0]);
        assert_ne!(a[2], a[1]);
    }

    /// GetPlatformStyle: the empty style-0 list draws nothing and returns
    /// −1; a non-empty list draws one Next(len) and maps the item's
    /// placeStyle (`DungeonGenerationStyleData.cs:150-158`).
    #[test]
    fn w4_f24_platform_style_maps_the_item_rows() {
        let style0 = current_dungeon_style(41, 481, 7, 91, 96, 8);
        let mut a = Rng::from_seed(7);
        let mut b = Rng::from_seed(7);
        assert_eq!(get_platform_style(&style0, &mut a), -1);
        assert_eq!(a.range(0, 1000), b.range(0, 1000)); // no draw consumed
        // Cavern's [94, 4416] → styles {0, 43}.
        let mut r = Rng::from_seed(3);
        for _ in 0..16 {
            let s = get_platform_style(&CAVERN, &mut r);
            assert!(s == 0 || s == 43, "got {s}");
        }
        // LivingWood's [2629] → 23 always; Hallow's [633, 3903] → {3, 30}.
        assert_eq!(get_platform_style(&LIVING_WOOD, &mut r), 23);
        let s = get_platform_style(&HALLOW, &mut r);
        assert!(s == 3 || s == 30);
    }

    /// GetStyleForWall: the style-0 Main.wallDungeon arm, the extras walk
    /// and the sub-style descent (`DungeonGenerationStyles.cs:160-185`).
    #[test]
    fn w4_f24_style_for_wall_resolves_subs() {
        let quad0 = vec![
            CAVERN,
            CORRUPTION,
            JUNGLE,
            current_dungeon_style(41, 481, 7, 91, 96, 8),
        ];
        // The style-0 arm catches every Main.wallDungeon wall.
        let idx = style_for_wall(&quad0, 94).unwrap();
        assert_eq!(style_at(&quad0, idx).style, 0);
        // Cavern's own wall.
        let idx = style_for_wall(&quad0, 349).unwrap();
        assert_eq!(style_at(&quad0, idx).style, 1);
        // Shimmer's 322 resolves through Cavern's SubStyles.
        let idx = style_for_wall(&quad0, 322).unwrap();
        let s = style_at(&quad0, idx);
        assert_eq!(s.style, 11);
        assert_eq!(s.bookshelf_min_max, None);
        // LivingWood's 244 → the (3,7) bookshelf override.
        let idx = style_for_wall(&quad0, 244).unwrap();
        let s = style_at(&quad0, idx);
        assert_eq!(s.style, 13);
        assert_eq!(s.bookshelf_min_max, Some((3, 7)));
        // Corruption's extra wall 188.
        let idx = style_for_wall(&quad0, 188).unwrap();
        assert_eq!(style_at(&quad0, idx).style, 4);
        // An unknown wall resolves to nothing.
        assert!(style_for_wall(&quad0, 1).is_none());
    }

    /// The waterbolt book's `Main.tile[placeY, placeY]` quirk is kept 1:1
    /// (`DungeonUtils.cs:409`): the frameX write needs the (y,y) cell to
    /// hold type 50, not the (x,y) cell.
    #[test]
    fn w4_f24_waterbolt_book_keeps_the_yy_quirk() {
        let mut w = World::new("book-quirk".into(), 200, 400);
        w.world_surface = 50.0;
        w.rock_layer = 150.0; // midpoint 100 — place below it
        // A floor under both the target (30, 120) and the quirk cell
        // (120, 120)… the quirk cell needs a book AT (120,120).
        // A shelf (table 14) under the book sites: `WorldGen.PlaceTile`'s
        // type-50 arm routes through `PlaceOnTable1x1`, which needs a table
        // below (plain stone refuses).
        for x in 0..200 {
            if let Some(t) = w.tile_mut(x, 121) {
                t.kind = 14;
                t.flags |= FLAG_ACTIVE;
            }
        }
        // Pre-place a book at the quirk cell (120, 120).
        if let Some(t) = w.tile_mut(120, 120) {
            t.kind = 50;
            t.flags |= FLAG_ACTIVE;
        }
        generate_dungeon_book_waterbolt(&mut w, 30, 120, true);
        let t = w.tile(30, 120).copied().unwrap();
        assert_eq!(t.kind, 50);
        assert_eq!(t.frame_x, 90); // the quirk cell held a book
        // Without the quirk-cell book, no frameX write even when the
        // placed cell took the book.
        if let Some(t) = w.tile_mut(120, 120) {
            t.kind = 1;
        }
        generate_dungeon_book_waterbolt(&mut w, 40, 120, true);
        let t = w.tile(40, 120).copied().unwrap();
        assert_eq!(t.kind, 50);
        assert_ne!(t.frame_x, 90);
        // Above the depth midpoint: never a waterbolt frame.
        if let Some(t) = w.tile_mut(120, 120) {
            t.kind = 50;
        }
        if let Some(t) = w.tile_mut(50, 91) {
            t.kind = 1;
            t.flags |= FLAG_ACTIVE;
        }
        generate_dungeon_book_waterbolt(&mut w, 50, 90, true);
        assert_ne!(w.tile(50, 90).unwrap().frame_x, 90);
    }

    /// The potion bottle's frameX pick is 18/36 on a Next(2)
    /// (`DungeonUtils.cs:378-392`); a == b replay.
    #[test]
    fn w4_f24_potion_bottle_frame_pick() {
        let run = || {
            let mut w = World::new("bottle".into(), 200, 200);
            for x in 0..200 {
                if let Some(t) = w.tile_mut(x, 101) {
                    t.kind = 1;
                    t.flags |= FLAG_ACTIVE;
                }
            }
            let mut r = Rng::from_seed(11);
            for x in [30, 40, 50] {
                generate_dungeon_potion_bottle(&mut w, &mut r, x, 100);
            }
            [30, 40, 50].map(|x| {
                let t = w.tile(x, 100).copied().unwrap();
                (t.kind, t.frame_x)
            })
        };
        let a = run();
        assert_eq!(a, run());
        for (kind, fx) in a {
            if kind == 13 {
                assert!(fx == 18 || fx == 36, "frameX {fx}");
            }
        }
    }

    /// PlacePot keeps the remix centre veto and the halfBrick clear
    /// (`WorldGen.cs:54069/:54108`) — the two arms the spider_gem_moss
    /// twin misses.
    #[test]
    fn w4_f24_place_pot_remix_veto_and_halfbrick_clear() {
        let mut w = World::new("pot".into(), 200, 400);
        // Floor at y=300 (inside the remix band: y > 400-220 = 180).
        for x in 0..200 {
            if let Some(t) = w.tile_mut(x, 300) {
                t.kind = 1;
                t.flags |= FLAG_ACTIVE;
            }
        }
        let mut r = Rng::from_seed(5);
        // x=100 is the world centre (0.48*200=96 < 100 < 104) — vetoed.
        assert!(!place_pot(&mut w, &mut r, 100, 299, 28, 10, true));
        assert!(!w.tile(100, 299).unwrap().active());
        // The same spot passes with remix off.
        assert!(place_pot(&mut w, &mut r, 100, 299, 28, 10, false));
        let t = w.tile(100, 299).copied().unwrap();
        assert!(t.active() && t.kind == 28);
        assert!(!t.half_brick());
        assert_eq!(t.frame_y, (18 + 10 * 36) as i16); // bottom row, style 10
        // Outside the centre band the remix flag does not veto.
        assert!(place_pot(&mut w, &mut r, 20, 299, 28, 11, true));
    }

    /// The Bookshelves pass places shelf runs against a dungeon-brick
    /// wall column and stays deterministic (`DungeonGlobalBookshelves.cs:
    /// 23-207`). Whole-pass a == b replay on a hand-built room.
    #[test]
    fn w4_f24_bookshelves_pass_is_deterministic() {
        let run = || {
            let mut w = World::new("shelves".into(), 400, 300);
            w.world_surface = 40.0;
            w.rock_layer = 120.0;
            // A room: blue-brick wall columns at x=100 and x=140, rows
            // 60..=100; dungeon wall 7 inside; a dirt floor at y=101.
            for y in 60..=100 {
                for x in 100..=140 {
                    if let Some(t) = w.tile_mut(x, y) {
                        t.wall = 7;
                    }
                }
                for x in [100, 140] {
                    if let Some(t) = w.tile_mut(x, y) {
                        t.kind = 41;
                        t.flags |= FLAG_ACTIVE;
                    }
                }
            }
            for x in 0..400 {
                if let Some(t) = w.tile_mut(x, 101) {
                    t.kind = 1;
                    t.flags |= FLAG_ACTIVE;
                }
            }
            let mut bounds = DungeonBounds::new(400, 300);
            bounds.set_bounds(90, 50, 150, 110);
            let fp = DungeonFootprint {
                left: 90,
                right: 150,
                top: 50,
                bottom: 110,
                wall: 7,
                cracked: 481,
            };
            let styles = vec![current_dungeon_style(41, 481, 7, 91, 96, 8)];
            let sets = vec![DungeonTileSets::build(&styles, true)];
            let mut r = Rng::from_seed(23);
            let shelf_styles = roll_shelf_styles(&mut r);
            let data = BookshelfData {
                bounds: &bounds,
                wall_variants: [7, 94, 95],
                shelf_styles,
                styles: &styles,
                sets: &sets,
                current: 0,
                dual: true,
                footprint: &fp,
            };
            let runs = bookshelves(&mut w, &mut r, &data);
            // Collect the placed platforms (kind 19) inside the room.
            let mut shelves = Vec::new();
            for y in 50..110 {
                for x in 90..150 {
                    let t = w.tile(x, y).copied().unwrap();
                    if t.active() && t.kind == 19 {
                        shelves.push((x, y, t.frame_y));
                    }
                }
            }
            (runs, shelves)
        };
        let (runs_a, shelves_a) = run();
        let (runs_b, shelves_b) = run();
        assert_eq!(runs_a, runs_b);
        assert_eq!(shelves_a, shelves_b);
        // The pass targets maxTilesX/20 = 20 runs (placed or budgeted).
        assert_eq!(runs_a, 400 / 20);
        // Every shelf frame is a shelfStyles pick (9..13 → 162..216).
        for (_, _, fy) in &shelves_a {
            assert!((9 * 18..13 * 18).contains(fy), "frameY {fy}");
        }
    }

    /// The wall-keyed default furniture rows + the 94..105 override
    /// (`DungeonGlobalGroundFurniture.cs:275-340`).
    #[test]
    fn w4_f24_furniture_defaults_match_the_wall_rows() {
        let blue = furniture_defaults(7, 7);
        assert_eq!((blue.chair, blue.table, blue.workbench), (1396, 1397, 1398));
        assert_eq!((blue.candle, blue.vase_or_statue, blue.bookcase), (1405, 1408, 1414));
        assert_eq!((blue.bed, blue.piano, blue.clock), (1470, 2376, 3900));
        let green = furniture_defaults(8, 8);
        assert_eq!((green.chair, green.table, green.candelabra), (1399, 1400, 2646));
        let pink = furniture_defaults(9, 9);
        assert_eq!((pink.chair, pink.bathtub, pink.clock), (1402, 2660, 3902));
        // The painted-wall band overrides ANY colour row — and reads the
        // CELL wall, not the passed wallType.
        let painted = furniture_defaults(7, 94);
        assert_eq!((painted.chair, painted.table, painted.candle), (1509, 1510, 5743));
        assert_eq!(painted.vase_or_statue, -1);
        assert_eq!((painted.bed, painted.piano, painted.clock), (5740, 5750, 5746));
        let painted_green = furniture_defaults(8, 105);
        assert_eq!(painted_green.chair, 1509);
        // Outside the band the colour rows stand.
        assert_eq!(furniture_defaults(7, 93).chair, 1396);
        assert_eq!(furniture_defaults(7, 106).chair, 1396);
    }

    /// GetFurnitureItem: null → −1, empty/style-0 → default (no draws),
    /// else one Next(len) pick (`:862-873`); the substitution chain runs
    /// in the C#'s order with lamp before candelabra (`:348-361`).
    #[test]
    fn w4_f24_furniture_item_substitution() {
        let mut a = Rng::from_seed(7);
        let mut b = Rng::from_seed(7);
        // Null and empty draw nothing.
        assert_eq!(get_furniture_item(1, &mut a, 1397, None), -1);
        assert_eq!(get_furniture_item(1, &mut a, 1397, Some(&[])), 1397);
        // Style 0 keeps the default even with a non-empty list.
        assert_eq!(get_furniture_item(0, &mut a, 1397, Some(&[32, 5894])), 1397);
        assert_eq!(a.range(0, 1000), b.range(0, 1000)); // stream untouched
        // A real pick draws one Next(len).
        let v = get_furniture_item(1, &mut a, 1397, Some(&[32, 5894]));
        assert!(v == 32 || v == 5894);
        // The full chain: Temple's vase row is the only non-null one.
        let defaults = furniture_defaults(7, 7);
        let temple = furniture_style_items(10).unwrap();
        let mut r = Rng::from_seed(3);
        let (items, style0) = furniture_items_for_style(defaults, 10, &temple, &mut r);
        assert!(!style0);
        assert_eq!(items.table, 1144);
        assert!([1152, 1153, 1154].contains(&items.vase_or_statue));
        // Style 0 keeps every default (the GetCurrentDungeonStyle rows
        // are all empty — the substitution is a no-op with no draws).
        let mut c = Rng::from_seed(3);
        let mut d = Rng::from_seed(3);
        let empty = FurnitureStyleItems {
            table: Some(&[]), workbench: Some(&[]), candle: Some(&[]),
            vase_or_statue: Some(&[]), bookcase: Some(&[]), chair: Some(&[]),
            bed: Some(&[]), piano: Some(&[]), dresser: Some(&[]),
            sofa: Some(&[]), bathtub: Some(&[]), lamp: Some(&[]),
            candelabra: Some(&[]), clock: Some(&[]),
        };
        let (kept, style0) = furniture_items_for_style(defaults, 0, &empty, &mut c);
        assert!(style0);
        assert_eq!(kept, defaults);
        assert_eq!(c.range(0, 1000), d.range(0, 1000));
    }

    /// The Next(13) ladder: the 10/11/12 re-roll and the −1-item
    /// rejection loop (`:363-371`); a == b replay.
    #[test]
    fn w4_f24_furniture_kind_ladder() {
        let run = |items: FurnitureItems| {
            let mut r = Rng::from_seed(41);
            (0..32).map(|_| roll_furniture_kind(&items, &mut r)).collect::<Vec<_>>()
        };
        let blue = furniture_defaults(7, 7);
        let a = run(blue);
        assert_eq!(a, run(blue));
        assert!(a.iter().all(|&k| (0..13).contains(&k)));
        // With the vase item −1 (the painted-wall row), kind 2 never
        // survives the rejection loop.
        let painted = furniture_defaults(7, 94);
        let b = run(painted);
        assert!(!b.contains(&2));
        assert_eq!(b, run(painted));
        // An all-−1 tail row set forces the never-rejected kinds only.
        let mut stripped = blue;
        stripped.bed = -1;
        stripped.piano = -1;
        stripped.dresser = -1;
        stripped.sofa = -1;
        stripped.bathtub = -1;
        stripped.lamp = -1;
        stripped.candelabra = -1;
        stripped.clock = -1;
        stripped.vase_or_statue = -1;
        let c = run(stripped);
        assert!(c.iter().all(|&k| [0, 1, 3, 4].contains(&k)));
        // The `:368` swapped pair, verbatim: kind 10 is rejected by the
        // CANDELABRA being −1 (not the lamp), kind 11 by the LAMP.
        let mut swap_a = blue;
        swap_a.candelabra = -1; // lamp stays present
        let sa = run(swap_a);
        assert!(!sa.contains(&10)); // kind 10 rejected via candelabra
        let mut swap_b = blue;
        swap_b.lamp = -1; // candelabra stays present
        let sb = run(swap_b);
        assert!(!sb.contains(&11)); // kind 11 rejected via lamp
    }

    /// The area tables + the 1.75f width gate (`:372-554`).
    #[test]
    fn w4_f24_furniture_area_tables_and_gate() {
        assert_eq!(furniture_area(0, false), (5, 4));
        assert_eq!(furniture_area(0, true), (3, 4));
        assert_eq!(furniture_area(4, false), (3, 3));
        assert_eq!(furniture_area(4, true), (1, 3));
        assert_eq!(furniture_area(12, false), (2, 5));
        assert_eq!(furniture_area(12, true), (2, 5));
        assert_eq!(furniture_area(-1, false), (0, 0));
        // The gate: run 8 vs half-width 5 → 8 >= 8.75 false; no-checks
        // → 8 >= 5 true.
        assert!(!furniture_width_gate(8, 5, false));
        assert!(furniture_width_gate(9, 5, false));
        assert!(furniture_width_gate(8, 5, true));
        assert!(!furniture_width_gate(4, 5, true));
    }

    /// The budget scalars (`:37-63/:191-218`): the (int)num truncation,
    /// the style-0 gate, the 5/10/15 candle minimum by world size.
    #[test]
    fn w4_f24_furniture_budgets() {
        // Small world (4200): num = 1.0 → alch/bewitch = 2, candles 5.
        assert_eq!(dual_furniture_budgets(4200, 1.0, true), (2, 2, 5));
        // Medium (6400): num = 1.523… → (int)num = 1 → still 2, candles 10.
        assert_eq!(dual_furniture_budgets(6400, 1.0, true), (2, 2, 10));
        // Large (8400): num = 2.0 → 3, candles 15.
        assert_eq!(dual_furniture_budgets(8400, 1.0, true), (3, 3, 15));
        // No style-0 in the quad zeroes both tables.
        assert_eq!(dual_furniture_budgets(4200, 1.0, false), (0, 0, 5));
        // The scalar multiplies AFTER the truncation.
        assert_eq!(dual_furniture_budgets(4200, 0.5, true), (1, 1, 5));
        // Legacy: attempts = 2000*num*scalar; non-dual candles −1.
        assert_eq!(legacy_furniture_budgets(4200, 1.0, true, false), (2000, 2, 2, -1));
        assert_eq!(legacy_furniture_budgets(4200, 1.0, true, true), (2000, 2, 2, 5));
    }
}
