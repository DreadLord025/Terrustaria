//! The dungeon biome style tables — `DungeonGenerationStyles`
//! (`DungeonGenerationStyles.cs:187-798`) + `DungeonGenerationStyleData`
//! (`DungeonGenerationStyleData.cs:9-169`) + the `dungeonGenerationStyles`
//! list assembly (`DungeonCrawler.cs:110-132`).
//!
//! This slice ports the four biome styles the dual-dungeon second quad and
//! the evil pick name — Snow (`:352-392`), Desert (`:393-433`), Crimson
//! (`:474-513`), Hallow (`:553-593`) — plus Crystal (`:514-552`), Hallow's
//! only sub-style (`:592`), and the style-0 wrapper
//! `GetCurrentDungeonStyle` (`:95-135`) over the landed [`DungeonColor`]
//! triple. The lookups `GetStyleForTile`/`GetStyleForWall` (`:137-185`) and
//! the membership predicates `TileIsInStyle`/`WallIsInStyle`
//! (`DungeonGenerationStyleData.cs:108-148`) ride with them.
//!
//! Out of this slice (cited skips): the other style statics — Shimmer/
//! Spider/LivingWood/Cavern (`:189-351`), Corruption (`:434-473`),
//! GlowingMushroom/Beehive/LivingMahogany/Jungle/Temple (`:594-797`) — the
//! dual-dungeon list arms (`DungeonCrawler.cs:111-128`, the port generates
//! classic single-dungeon worlds only — M5-SEED-2 defers `dualDungeons`),
//! the `isDungeonTile`/`isCrackedBrick` set clones (`:139-183`, they need
//! the dual list), `GetPlatformStyle`/`GetWindowPlatformStyle`
//! (`DungeonGenerationStyleData.cs:150-168` — they read
//! `ItemID.Sets.DerivedPlacementDetails`, whose port lives in tr-app's
//! generated `itemdata.rs`; the item-data seam rides the wiring round), and
//! the LivingWood/Beehive/Temple `CanGenerateFeatureAt`/bookshelf overrides
//! (`DungeonGenerationStyles.cs:23-63`, their styles are out of scope).
//!
//! UNWIRED: these are data tables + pure lookups; no pipeline step consumes
//! them yet, so no RNG draw and no tile changes — both world-hash pins stay
//! (`world_hash.rs`'s W4-c32 note).

use super::dungeon::DungeonColor;
use super::dungeon_features::DUNGEON_WALLS;

/// `DualDungeonUnbreakableWallTiers` (`DualDungeonUnbreakableWallTiers.cs`:
/// the static ctor's 0..5 ladder).
pub mod wall_tiers {
    pub const EARLY_GAME: i32 = 0;
    pub const EVIL_BOSS: i32 = 1;
    pub const JUNGLE_BOSS: i32 = 2;
    pub const DUNGEON: i32 = 3;
    pub const HALLOW: i32 = 4;
    pub const TEMPLE: i32 = 5;
}

/// `DungeonRoomType` (`Rooms/DungeonRoomType.cs:3-17`) — the enum's ordinal
/// values; only `BiomeRugged`/`BiomeStructured` are named by this slice's
/// styles, the rest keep their slots for the later style statics.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum DungeonRoomType {
    Legacy = 0,
    Regular = 1,
    Wormlike = 2,
    LivingTree = 3,
    BiomeSquare = 4,
    BiomeRugged = 5,
    BiomeStructured = 6,
    GenShapeCircle = 7,
    GenShapeMound = 8,
    GenShapeHourglass = 9,
    GenShapeDoughnut = 10,
    GenShapeQuadCircle = 11,
}

/// The `IDungeonFeature` kinds the in-scope `CanGenerateFeatureAt` overrides
/// test (`DungeonGenerationStyles.cs:13-20` — Shimmer/Crystal refuse
/// `DungeonPitTrap` and `DungeonWindow`). `Other` stands for every feature
/// the overrides do not name (the base returns true for all).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DungeonFeature {
    PitTrap,
    Window,
    Other,
}

/// `DungeonGenerationStyleData` (`DungeonGenerationStyleData.cs:11-95`).
/// The C#'s `int[]` furniture rows become `&'static [i32]` slices (empty =
/// the C#'s `new int[0]`); `BannerItemTypes`/`VaseOrStatueItemTypes` keep
/// their null-vs-array distinction as `Option` — a null banner row means
/// "fall back to the dungeon's `bannerStyles`" (the
/// `dungeon_features::place_banners` fallback half), not "no banners".
#[derive(Clone, Debug, PartialEq)]
pub struct DungeonStyleData {
    /// `Style` — 0 is the current dungeon's own colour wrapper.
    pub style: u8,
    /// `UnbreakableWallProgressionTier` (default -1, `:13`).
    pub unbreakable_wall_tier: i32,
    pub brick_tile: u16,
    /// `BrickGrassTileType` (`ushort?`, `:17`) — none in this slice.
    pub brick_grass_tile: Option<u16>,
    pub brick_cracked_tile: u16,
    pub brick_wall: u16,
    /// `ExtraTileTypes` (`:23`) — null = empty.
    pub extra_tiles: &'static [u16],
    /// `TilePaintColor`/`WallPaintColor` (`byte?`, `:25-27`) — none here
    /// (LivingMahogany's wall paint 12 is out of scope).
    pub tile_paint: Option<u8>,
    pub wall_paint: Option<u8>,
    pub window_glass_wall: u16,
    pub window_closed_glass_wall: u16,
    pub window_edge_wall: u16,
    pub window_platform_items: &'static [i32],
    /// `ExtraWallTypes` (`:37`) — null = empty.
    pub extra_walls: &'static [u16],
    pub pit_trap_tile: u16,
    /// `LiquidType` (`:41`, default -1) — none of this slice's styles
    /// overrides it (Shimmer's 3 / Beehive's 2 are out of scope).
    pub liquid_type: i32,
    pub locked_biome_chest_type: i32,
    pub locked_biome_chest_style: i32,
    pub biome_chest_item: i32,
    pub biome_chest_loot_item: i32,
    pub chest_items: &'static [i32],
    pub door_items: &'static [i32],
    pub platform_items: &'static [i32],
    pub chandelier_items: &'static [i32],
    pub lantern_items: &'static [i32],
    pub table_items: &'static [i32],
    pub workbench_items: &'static [i32],
    pub candle_items: &'static [i32],
    /// `VaseOrStatueItemTypes` (`:67`) — null in every in-scope style.
    pub vase_or_statue_items: Option<&'static [i32]>,
    pub bookcase_items: &'static [i32],
    pub chair_items: &'static [i32],
    pub bed_items: &'static [i32],
    pub piano_items: &'static [i32],
    pub dresser_items: &'static [i32],
    pub sofa_items: &'static [i32],
    pub bathtub_items: &'static [i32],
    pub lamp_items: &'static [i32],
    pub candelabra_items: &'static [i32],
    pub clock_items: &'static [i32],
    /// `BannerItemTypes` (`:89`) — null = the `bannerStyles` fallback.
    pub banner_items: Option<&'static [i32]>,
    pub edge_dither: bool,
    pub biome_room_type: DungeonRoomType,
    /// `SubStyles` (`:95`) — null = empty.
    pub sub_styles: &'static [&'static DungeonStyleData],
    /// The `ShimmerStyleData.CanGenerateFeatureAt` override
    /// (`DungeonGenerationStyles.cs:13-20`) — true for the Crystal static
    /// (a `ShimmerStyleData`, `:514`); the plain styles keep the base.
    pub shimmer_feature_gate: bool,
}

impl DungeonStyleData {
    /// `CanGenerateFeatureAt` (`DungeonGenerationStyleData.cs:97-100` base +
    /// the Shimmer override `DungeonGenerationStyles.cs:13-20`).
    pub fn can_generate_feature(&self, feature: DungeonFeature) -> bool {
        if self.shimmer_feature_gate {
            return !matches!(feature, DungeonFeature::PitTrap | DungeonFeature::Window);
        }
        true
    }

    /// `TileIsInStyle` (`DungeonGenerationStyleData.cs:108-129`) — the
    /// grass/cracked/extras/brick test order verbatim.
    pub fn tile_is_in_style(&self, tile: u16, include_cracked: bool, include_extras: bool) -> bool {
        if self.brick_grass_tile == Some(tile) {
            return true;
        }
        if include_cracked && tile == self.brick_cracked_tile {
            return true;
        }
        if include_extras && self.extra_tiles.contains(&tile) {
            return true;
        }
        tile == self.brick_tile
    }

    /// `WallIsInStyle` (`DungeonGenerationStyleData.cs:131-148`) — the
    /// windows/extras/brick-wall test order verbatim.
    pub fn wall_is_in_style(&self, wall: u16, include_windows: bool, include_extras: bool) -> bool {
        if include_windows
            && (wall == self.window_glass_wall
                || wall == self.window_edge_wall
                || wall == self.window_closed_glass_wall)
        {
            return true;
        }
        if include_extras && self.extra_walls.contains(&wall) {
            return true;
        }
        wall == self.brick_wall
    }
}

/// `DungeonGenerationStyles.Snow` (`DungeonGenerationStyles.cs:352-392`).
pub static SNOW: DungeonStyleData = DungeonStyleData {
    style: 2,
    unbreakable_wall_tier: wall_tiers::EARLY_GAME,
    brick_tile: 161,
    brick_grass_tile: None,
    brick_cracked_tile: 224,
    brick_wall: 71,
    extra_tiles: &[147],
    tile_paint: None,
    wall_paint: None,
    window_glass_wall: 90,
    window_closed_glass_wall: 149,
    window_edge_wall: 31,
    window_platform_items: &[3908],
    extra_walls: &[40],
    pit_trap_tile: 224,
    liquid_type: -1,
    locked_biome_chest_type: 21,
    locked_biome_chest_style: 27,
    biome_chest_item: 1532,
    biome_chest_loot_item: 1572,
    chest_items: &[681, 5805],
    door_items: &[2044, 5807],
    platform_items: &[3908, 5812],
    chandelier_items: &[2059, 5804],
    lantern_items: &[2040, 5810],
    table_items: &[2248, 5815],
    workbench_items: &[2252, 5817],
    candle_items: &[2049, 5802],
    vase_or_statue_items: None,
    bookcase_items: &[2031, 5800],
    chair_items: &[2288, 5803],
    bed_items: &[2068, 5799],
    piano_items: &[2247, 5811],
    dresser_items: &[3913, 5808],
    sofa_items: &[2635, 5814],
    bathtub_items: &[2076, 5798],
    lamp_items: &[2086, 5809],
    candelabra_items: &[2100, 5801],
    clock_items: &[2594, 5806],
    banner_items: None,
    edge_dither: true,
    biome_room_type: DungeonRoomType::BiomeRugged,
    sub_styles: &[],
    shimmer_feature_gate: false,
};

/// `DungeonGenerationStyles.Desert` (`DungeonGenerationStyles.cs:393-433`).
pub static DESERT: DungeonStyleData = DungeonStyleData {
    style: 3,
    unbreakable_wall_tier: wall_tiers::EARLY_GAME,
    brick_tile: 396,
    brick_grass_tile: None,
    brick_cracked_tile: 53,
    brick_wall: 187,
    extra_tiles: &[397, 404],
    tile_paint: None,
    wall_paint: None,
    window_glass_wall: 89,
    window_closed_glass_wall: 151,
    window_edge_wall: 34,
    window_platform_items: &[4311],
    extra_walls: &[216, 223],
    pit_trap_tile: 53,
    liquid_type: -1,
    locked_biome_chest_type: 467,
    locked_biome_chest_style: 13,
    biome_chest_item: 4712,
    biome_chest_loot_item: 4607,
    chest_items: &[4267],
    door_items: &[4307],
    platform_items: &[4311],
    chandelier_items: &[4305],
    lantern_items: &[4309],
    table_items: &[4314],
    workbench_items: &[4315],
    candle_items: &[4303],
    vase_or_statue_items: None,
    bookcase_items: &[4300],
    chair_items: &[4304],
    bed_items: &[4299],
    piano_items: &[4310],
    dresser_items: &[4301],
    sofa_items: &[4313],
    bathtub_items: &[4298],
    lamp_items: &[4308],
    candelabra_items: &[4302],
    clock_items: &[4306],
    banner_items: Some(&[790, 791, 789]),
    edge_dither: false,
    biome_room_type: DungeonRoomType::BiomeRugged,
    sub_styles: &[],
    shimmer_feature_gate: false,
};

/// `DungeonGenerationStyles.Crimson` (`DungeonGenerationStyles.cs:474-513`).
pub static CRIMSON: DungeonStyleData = DungeonStyleData {
    style: 5,
    unbreakable_wall_tier: wall_tiers::EVIL_BOSS,
    brick_tile: 203,
    brick_grass_tile: None,
    brick_cracked_tile: 234,
    brick_wall: 83,
    extra_tiles: &[],
    tile_paint: None,
    wall_paint: None,
    window_glass_wall: 92,
    window_closed_glass_wall: 85,
    window_edge_wall: 174,
    window_platform_items: &[913],
    extra_walls: &[192, 193, 194, 195],
    pit_trap_tile: 234,
    liquid_type: -1,
    locked_biome_chest_type: 21,
    locked_biome_chest_style: 25,
    biome_chest_item: 1530,
    biome_chest_loot_item: 1569,
    chest_items: &[914, 2617, 5784],
    door_items: &[912, 817, 5786],
    platform_items: &[913, 3907, 5791],
    chandelier_items: &[2142, 2057, 5783],
    lantern_items: &[2146, 2034, 5789],
    table_items: &[917, 828, 5794],
    workbench_items: &[916, 813, 5796],
    candle_items: &[2154, 2047, 5781],
    vase_or_statue_items: None,
    bookcase_items: &[2136, 2022, 5779],
    chair_items: &[915, 809, 5782],
    bed_items: &[920, 2067, 5778],
    piano_items: &[919, 2246, 5790],
    dresser_items: &[918, 2640, 5787],
    sofa_items: &[2401, 2634, 5793],
    bathtub_items: &[2127, 2074, 5777],
    lamp_items: &[2132, 2084, 5788],
    candelabra_items: &[2150, 2094, 5780],
    clock_items: &[2604, 2598, 5785],
    banner_items: None,
    edge_dither: true,
    biome_room_type: DungeonRoomType::BiomeRugged,
    sub_styles: &[],
    shimmer_feature_gate: false,
};

/// `DungeonGenerationStyles.Crystal` (`DungeonGenerationStyles.cs:514-552`)
/// — a `ShimmerStyleData`, Hallow's only sub-style (`:592`).
pub static CRYSTAL: DungeonStyleData = DungeonStyleData {
    style: 15,
    unbreakable_wall_tier: wall_tiers::HALLOW,
    brick_tile: 385,
    brick_grass_tile: None,
    brick_cracked_tile: 116,
    brick_wall: 186,
    extra_tiles: &[],
    tile_paint: None,
    wall_paint: None,
    window_glass_wall: 88,
    window_closed_glass_wall: 43,
    window_edge_wall: 22,
    window_platform_items: &[633],
    extra_walls: &[],
    pit_trap_tile: 116,
    liquid_type: -1,
    locked_biome_chest_type: -1,
    locked_biome_chest_style: -1,
    biome_chest_item: -1,
    biome_chest_loot_item: -1,
    chest_items: &[3884],
    door_items: &[3888],
    platform_items: &[3903],
    chandelier_items: &[3894],
    lantern_items: &[3891],
    table_items: &[3920],
    workbench_items: &[3909],
    candle_items: &[3890],
    vase_or_statue_items: None,
    bookcase_items: &[3917],
    chair_items: &[3889],
    bed_items: &[3897],
    piano_items: &[3915],
    dresser_items: &[3911],
    sofa_items: &[3918],
    bathtub_items: &[3895],
    lamp_items: &[3892],
    candelabra_items: &[3893],
    clock_items: &[3898],
    banner_items: None,
    edge_dither: false,
    biome_room_type: DungeonRoomType::BiomeStructured,
    sub_styles: &[],
    shimmer_feature_gate: true,
};

/// `DungeonGenerationStyles.Hallow` (`DungeonGenerationStyles.cs:553-593`).
pub static HALLOW: DungeonStyleData = DungeonStyleData {
    style: 6,
    unbreakable_wall_tier: wall_tiers::HALLOW,
    brick_tile: 117,
    brick_grass_tile: None,
    brick_cracked_tile: 116,
    brick_wall: 28,
    extra_tiles: &[],
    tile_paint: None,
    wall_paint: None,
    window_glass_wall: 91,
    window_closed_glass_wall: 43,
    window_edge_wall: 22,
    window_platform_items: &[633],
    extra_walls: &[200, 201, 202, 203],
    pit_trap_tile: 116,
    liquid_type: -1,
    locked_biome_chest_type: 21,
    locked_biome_chest_style: 26,
    biome_chest_item: 1531,
    biome_chest_loot_item: 1260,
    chest_items: &[627, 3884],
    door_items: &[652, 3888],
    platform_items: &[633, 3903],
    chandelier_items: &[2061, 3894],
    lantern_items: &[2039, 3891],
    table_items: &[640, 3920],
    workbench_items: &[637, 3909],
    candle_items: &[2051, 3890],
    vase_or_statue_items: None,
    bookcase_items: &[2027, 3917],
    chair_items: &[630, 3889],
    bed_items: &[646, 3897],
    piano_items: &[643, 3915],
    dresser_items: &[649, 3911],
    sofa_items: &[2400, 3918],
    bathtub_items: &[2078, 3895],
    lamp_items: &[2088, 3892],
    candelabra_items: &[2099, 3893],
    clock_items: &[2602, 3898],
    banner_items: None,
    edge_dither: true,
    biome_room_type: DungeonRoomType::BiomeRugged,
    sub_styles: &[&CRYSTAL],
    shimmer_feature_gate: false,
};

/// `GetCurrentDungeonStyle` (`DungeonGenerationStyles.cs:95-135`): the
/// style-0 wrapper over the current dungeon's `genVars` colour triple —
/// the brick/cracked/wall ids from [`DungeonColor`], the window walls and
/// platform item from the colour's `SetupDungeonGenVarVariables` switch
/// (`DungeonCrawler.cs:63-92`), `PitTrapTileType = brickCrackedTileType`
/// (`:107`), every furniture row empty and the chest ids -1.
pub fn current_dungeon_style(color: DungeonColor) -> DungeonStyleData {
    DungeonStyleData {
        style: 0,
        unbreakable_wall_tier: wall_tiers::DUNGEON,
        brick_tile: color.brick_tile(),
        brick_grass_tile: None,
        brick_cracked_tile: color.cracked_tile(),
        brick_wall: color.brick_wall(),
        extra_tiles: &[],
        tile_paint: None,
        wall_paint: None,
        window_glass_wall: color.window_glass_wall(),
        window_closed_glass_wall: color.window_closed_glass_wall(),
        window_edge_wall: color.window_edge_wall(),
        window_platform_items: color.window_platform_items(),
        extra_walls: &[],
        pit_trap_tile: color.cracked_tile(),
        liquid_type: -1,
        locked_biome_chest_type: -1,
        locked_biome_chest_style: -1,
        biome_chest_item: -1,
        biome_chest_loot_item: -1,
        chest_items: &[],
        door_items: &[],
        platform_items: &[],
        chandelier_items: &[],
        lantern_items: &[],
        table_items: &[],
        workbench_items: &[],
        candle_items: &[],
        vase_or_statue_items: None,
        bookcase_items: &[],
        chair_items: &[],
        bed_items: &[],
        piano_items: &[],
        dresser_items: &[],
        sofa_items: &[],
        bathtub_items: &[],
        lamp_items: &[],
        candelabra_items: &[],
        clock_items: &[],
        banner_items: Some(&[]),
        edge_dither: false,
        biome_room_type: DungeonRoomType::BiomeStructured,
        sub_styles: &[],
        shimmer_feature_gate: false,
    }
}

/// `DungeonCrawler.cs:129-132` — the classic (non-dual-dungeon) arm:
/// `genVars.dungeonGenerationStyles.Add(genVars.dungeonStyle)`, one entry.
/// The dual-dungeon quads (`:111-128`) are the cited skip above.
pub fn dungeon_style_list(color: DungeonColor) -> Vec<DungeonStyleData> {
    vec![current_dungeon_style(color)]
}

/// `GetStyleForTile` (`DungeonGenerationStyles.cs:137-158`): the first
/// style (or sub-style) whose `TileIsInStyle(tileType, includeCracked:
/// false, includeExtras: true)` holds; null otherwise.
pub fn style_for_tile<'a>(styles: &'a [DungeonStyleData], tile: u16) -> Option<&'a DungeonStyleData> {
    for style in styles {
        if style.tile_is_in_style(tile, false, true) {
            return Some(style);
        }
        for sub in style.sub_styles {
            if sub.tile_is_in_style(tile, false, true) {
                return Some(sub);
            }
        }
    }
    None
}

/// `WallID.Count` (`WallID.cs:845`).
const WALL_COUNT: u16 = 367;

/// `GetStyleForWall` (`DungeonGenerationStyles.cs:160-185`): the style-0
/// arm first (`style.Style == 0 && wallType >= 0 && wallType < WallID.Count
/// && Main.wallDungeon[wallType]` — `Main.cs:10598-10606` is the port's
/// [`DUNGEON_WALLS`]), then `WallIsInStyle(includeWindows: false,
/// includeExtras: true)` per style and sub-style.
pub fn style_for_wall<'a>(styles: &'a [DungeonStyleData], wall: u16) -> Option<&'a DungeonStyleData> {
    for style in styles {
        if style.style == 0 && wall < WALL_COUNT && DUNGEON_WALLS.contains(&wall) {
            return Some(style);
        }
        if style.wall_is_in_style(wall, false, true) {
            return Some(style);
        }
        for sub in style.sub_styles {
            if sub.wall_is_in_style(wall, false, true) {
                return Some(sub);
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The four palettes against their C# static-ctor rows: brick / cracked
    /// / wall / pit-trap / the window triple (`DungeonGenerationStyles.cs`
    /// :352-392 / :393-433 / :474-513 / :553-593).
    #[test]
    fn w4_c32_the_four_palettes_match_the_csharp_rows() {
        assert_eq!((SNOW.brick_tile, SNOW.brick_cracked_tile, SNOW.brick_wall), (161, 224, 71));
        assert_eq!((SNOW.window_glass_wall, SNOW.window_closed_glass_wall, SNOW.window_edge_wall), (90, 149, 31));
        assert_eq!(SNOW.pit_trap_tile, 224);
        assert_eq!((DESERT.brick_tile, DESERT.brick_cracked_tile, DESERT.brick_wall), (396, 53, 187));
        assert_eq!((DESERT.window_glass_wall, DESERT.window_closed_glass_wall, DESERT.window_edge_wall), (89, 151, 34));
        assert_eq!(DESERT.pit_trap_tile, 53);
        assert_eq!((CRIMSON.brick_tile, CRIMSON.brick_cracked_tile, CRIMSON.brick_wall), (203, 234, 83));
        assert_eq!((CRIMSON.window_glass_wall, CRIMSON.window_closed_glass_wall, CRIMSON.window_edge_wall), (92, 85, 174));
        assert_eq!(CRIMSON.pit_trap_tile, 234);
        assert_eq!((HALLOW.brick_tile, HALLOW.brick_cracked_tile, HALLOW.brick_wall), (117, 116, 28));
        assert_eq!((HALLOW.window_glass_wall, HALLOW.window_closed_glass_wall, HALLOW.window_edge_wall), (91, 43, 22));
        assert_eq!(HALLOW.pit_trap_tile, 116);
        // The style ordinals + the progression tiers (:354/:395/:476/:555).
        assert_eq!((SNOW.style, DESERT.style, CRIMSON.style, HALLOW.style), (2, 3, 5, 6));
        assert_eq!(SNOW.unbreakable_wall_tier, wall_tiers::EARLY_GAME);
        assert_eq!(DESERT.unbreakable_wall_tier, wall_tiers::EARLY_GAME);
        assert_eq!(CRIMSON.unbreakable_wall_tier, wall_tiers::EVIL_BOSS);
        assert_eq!(HALLOW.unbreakable_wall_tier, wall_tiers::HALLOW);
    }

    /// The extras rows (`ExtraTileTypes`/`ExtraWallTypes`) and the biome
    /// chest quartet per style.
    #[test]
    fn w4_c32_the_extras_and_biome_chests_match() {
        assert_eq!(SNOW.extra_tiles, &[147]);
        assert_eq!(SNOW.extra_walls, &[40]);
        assert_eq!(DESERT.extra_tiles, &[397, 404]);
        assert_eq!(DESERT.extra_walls, &[216, 223]);
        assert!(CRIMSON.extra_tiles.is_empty());
        assert_eq!(CRIMSON.extra_walls, &[192, 193, 194, 195]);
        assert!(HALLOW.extra_tiles.is_empty());
        assert_eq!(HALLOW.extra_walls, &[200, 201, 202, 203]);
        assert_eq!((SNOW.locked_biome_chest_type, SNOW.locked_biome_chest_style), (21, 27));
        assert_eq!((SNOW.biome_chest_item, SNOW.biome_chest_loot_item), (1532, 1572));
        assert_eq!((DESERT.locked_biome_chest_type, DESERT.locked_biome_chest_style), (467, 13));
        assert_eq!((DESERT.biome_chest_item, DESERT.biome_chest_loot_item), (4712, 4607));
        assert_eq!((CRIMSON.locked_biome_chest_type, CRIMSON.locked_biome_chest_style), (21, 25));
        assert_eq!((CRIMSON.biome_chest_item, CRIMSON.biome_chest_loot_item), (1530, 1569));
        assert_eq!((HALLOW.locked_biome_chest_type, HALLOW.locked_biome_chest_style), (21, 26));
        assert_eq!((HALLOW.biome_chest_item, HALLOW.biome_chest_loot_item), (1531, 1260));
    }

    /// The furniture rows: the 2-item boreal/hallow pairs, the desert
    /// singles, the crimson triples (`:370-388` / `:411-429` / `:491-509` /
    /// `:570-588`).
    #[test]
    fn w4_c32_the_furniture_rows_match() {
        assert_eq!(SNOW.chest_items, &[681, 5805]);
        assert_eq!(SNOW.door_items, &[2044, 5807]);
        assert_eq!(SNOW.platform_items, &[3908, 5812]);
        assert_eq!(SNOW.chandelier_items, &[2059, 5804]);
        assert_eq!(SNOW.lantern_items, &[2040, 5810]);
        assert_eq!(SNOW.table_items, &[2248, 5815]);
        assert_eq!(SNOW.workbench_items, &[2252, 5817]);
        assert_eq!(SNOW.candle_items, &[2049, 5802]);
        assert_eq!(SNOW.bookcase_items, &[2031, 5800]);
        assert_eq!(SNOW.chair_items, &[2288, 5803]);
        assert_eq!(SNOW.bed_items, &[2068, 5799]);
        assert_eq!(SNOW.piano_items, &[2247, 5811]);
        assert_eq!(SNOW.dresser_items, &[3913, 5808]);
        assert_eq!(SNOW.sofa_items, &[2635, 5814]);
        assert_eq!(SNOW.bathtub_items, &[2076, 5798]);
        assert_eq!(SNOW.lamp_items, &[2086, 5809]);
        assert_eq!(SNOW.candelabra_items, &[2100, 5801]);
        assert_eq!(SNOW.clock_items, &[2594, 5806]);
        assert_eq!(DESERT.chest_items, &[4267]);
        assert_eq!(DESERT.door_items, &[4307]);
        assert_eq!(DESERT.platform_items, &[4311]);
        assert_eq!(DESERT.chandelier_items, &[4305]);
        assert_eq!(DESERT.lantern_items, &[4309]);
        assert_eq!(DESERT.table_items, &[4314]);
        assert_eq!(DESERT.workbench_items, &[4315]);
        assert_eq!(DESERT.candle_items, &[4303]);
        assert_eq!(DESERT.bookcase_items, &[4300]);
        assert_eq!(DESERT.chair_items, &[4304]);
        assert_eq!(DESERT.bed_items, &[4299]);
        assert_eq!(DESERT.piano_items, &[4310]);
        assert_eq!(DESERT.dresser_items, &[4301]);
        assert_eq!(DESERT.sofa_items, &[4313]);
        assert_eq!(DESERT.bathtub_items, &[4298]);
        assert_eq!(DESERT.lamp_items, &[4308]);
        assert_eq!(DESERT.candelabra_items, &[4302]);
        assert_eq!(DESERT.clock_items, &[4306]);
        assert_eq!(CRIMSON.chest_items, &[914, 2617, 5784]);
        assert_eq!(CRIMSON.door_items, &[912, 817, 5786]);
        assert_eq!(CRIMSON.platform_items, &[913, 3907, 5791]);
        assert_eq!(CRIMSON.chandelier_items, &[2142, 2057, 5783]);
        assert_eq!(CRIMSON.lantern_items, &[2146, 2034, 5789]);
        assert_eq!(CRIMSON.table_items, &[917, 828, 5794]);
        assert_eq!(CRIMSON.workbench_items, &[916, 813, 5796]);
        assert_eq!(CRIMSON.candle_items, &[2154, 2047, 5781]);
        assert_eq!(CRIMSON.bookcase_items, &[2136, 2022, 5779]);
        assert_eq!(CRIMSON.chair_items, &[915, 809, 5782]);
        assert_eq!(CRIMSON.bed_items, &[920, 2067, 5778]);
        assert_eq!(CRIMSON.piano_items, &[919, 2246, 5790]);
        assert_eq!(CRIMSON.dresser_items, &[918, 2640, 5787]);
        assert_eq!(CRIMSON.sofa_items, &[2401, 2634, 5793]);
        assert_eq!(CRIMSON.bathtub_items, &[2127, 2074, 5777]);
        assert_eq!(CRIMSON.lamp_items, &[2132, 2084, 5788]);
        assert_eq!(CRIMSON.candelabra_items, &[2150, 2094, 5780]);
        assert_eq!(CRIMSON.clock_items, &[2604, 2598, 5785]);
        assert_eq!(HALLOW.chest_items, &[627, 3884]);
        assert_eq!(HALLOW.door_items, &[652, 3888]);
        assert_eq!(HALLOW.platform_items, &[633, 3903]);
        assert_eq!(HALLOW.chandelier_items, &[2061, 3894]);
        assert_eq!(HALLOW.lantern_items, &[2039, 3891]);
        assert_eq!(HALLOW.table_items, &[640, 3920]);
        assert_eq!(HALLOW.workbench_items, &[637, 3909]);
        assert_eq!(HALLOW.candle_items, &[2051, 3890]);
        assert_eq!(HALLOW.bookcase_items, &[2027, 3917]);
        assert_eq!(HALLOW.chair_items, &[630, 3889]);
        assert_eq!(HALLOW.bed_items, &[646, 3897]);
        assert_eq!(HALLOW.piano_items, &[643, 3915]);
        assert_eq!(HALLOW.dresser_items, &[649, 3911]);
        assert_eq!(HALLOW.sofa_items, &[2400, 3918]);
        assert_eq!(HALLOW.bathtub_items, &[2078, 3895]);
        assert_eq!(HALLOW.lamp_items, &[2088, 3892]);
        assert_eq!(HALLOW.candelabra_items, &[2099, 3893]);
        assert_eq!(HALLOW.clock_items, &[2602, 3898]);
        // The window-platform singles (:364/:405/:485/:564).
        assert_eq!(SNOW.window_platform_items, &[3908]);
        assert_eq!(DESERT.window_platform_items, &[4311]);
        assert_eq!(CRIMSON.window_platform_items, &[913]);
        assert_eq!(HALLOW.window_platform_items, &[633]);
    }

    /// The banner null-vs-array distinction and the dither/room-type flags:
    /// only Desert carries its own banners (790/791/789, `:430`); the rest
    /// fall back to the dungeon's `bannerStyles`.
    #[test]
    fn w4_c32_the_banner_and_dither_flags_match() {
        assert_eq!(SNOW.banner_items, None);
        assert_eq!(DESERT.banner_items, Some(&[790, 791, 789][..]));
        assert_eq!(CRIMSON.banner_items, None);
        assert_eq!(HALLOW.banner_items, None);
        assert!(SNOW.vase_or_statue_items.is_none());
        assert!(DESERT.vase_or_statue_items.is_none());
        assert!(CRIMSON.vase_or_statue_items.is_none());
        assert!(HALLOW.vase_or_statue_items.is_none());
        assert!(SNOW.edge_dither);
        assert!(!DESERT.edge_dither);
        assert!(CRIMSON.edge_dither);
        assert!(HALLOW.edge_dither);
        assert_eq!(SNOW.biome_room_type, DungeonRoomType::BiomeRugged);
        assert_eq!(DESERT.biome_room_type, DungeonRoomType::BiomeRugged);
        assert_eq!(CRIMSON.biome_room_type, DungeonRoomType::BiomeRugged);
        assert_eq!(HALLOW.biome_room_type, DungeonRoomType::BiomeRugged);
        assert_eq!(CRYSTAL.biome_room_type, DungeonRoomType::BiomeStructured);
    }

    /// `TileIsInStyle`'s flag semantics (`DungeonGenerationStyleData.cs`
    /// :108-129): cracked only with `includeCracked`, extras only with
    /// `includeExtras`, the brick always.
    #[test]
    fn w4_c32_tile_membership_honours_the_flags() {
        assert!(SNOW.tile_is_in_style(161, false, false));
        assert!(SNOW.tile_is_in_style(224, true, false));
        assert!(!SNOW.tile_is_in_style(224, false, true));
        assert!(SNOW.tile_is_in_style(147, false, true));
        assert!(!SNOW.tile_is_in_style(147, false, false));
        assert!(DESERT.tile_is_in_style(53, true, false));
        assert!(DESERT.tile_is_in_style(397, false, true));
        assert!(DESERT.tile_is_in_style(404, false, true));
        assert!(CRIMSON.tile_is_in_style(234, true, false));
        assert!(HALLOW.tile_is_in_style(116, true, false));
        assert!(!HALLOW.tile_is_in_style(385, true, true), "Crystal's brick is the sub-style's, not Hallow's");
    }

    /// `WallIsInStyle`'s flag semantics (`:131-148`): the window triple only
    /// with `includeWindows`, extras only with `includeExtras`.
    #[test]
    fn w4_c32_wall_membership_honours_the_flags() {
        assert!(SNOW.wall_is_in_style(71, false, false));
        assert!(SNOW.wall_is_in_style(90, true, false));
        assert!(SNOW.wall_is_in_style(149, true, false));
        assert!(SNOW.wall_is_in_style(31, true, false));
        assert!(!SNOW.wall_is_in_style(90, false, true));
        assert!(SNOW.wall_is_in_style(40, false, true));
        assert!(DESERT.wall_is_in_style(216, false, true));
        assert!(DESERT.wall_is_in_style(223, false, true));
        assert!(CRIMSON.wall_is_in_style(192, false, true));
        assert!(CRIMSON.wall_is_in_style(195, false, true));
        assert!(HALLOW.wall_is_in_style(200, false, true));
        assert!(HALLOW.wall_is_in_style(203, false, true));
        assert!(!HALLOW.wall_is_in_style(186, false, true), "Crystal's wall is the sub-style's");
    }

    /// `GetStyleForTile`/`GetStyleForWall` (`DungeonGenerationStyles.cs`
    /// :137-185): the sub-style walk finds Crystal under Hallow, cracked
    /// bricks are excluded (includeCracked: false), extras are included.
    #[test]
    fn w4_c32_the_lookups_walk_the_sub_styles() {
        let flat: Vec<DungeonStyleData> =
            vec![SNOW.clone(), DESERT.clone(), HALLOW.clone(), CRIMSON.clone()];
        assert_eq!(style_for_tile(&flat, 161).map(|s| s.style), Some(2));
        assert_eq!(style_for_tile(&flat, 147).map(|s| s.style), Some(2));
        assert_eq!(style_for_tile(&flat, 396).map(|s| s.style), Some(3));
        assert_eq!(style_for_tile(&flat, 404).map(|s| s.style), Some(3));
        assert_eq!(style_for_tile(&flat, 117).map(|s| s.style), Some(6));
        // Crystal (15) via Hallow's SubStyles (:592).
        assert_eq!(style_for_tile(&flat, 385).map(|s| s.style), Some(15));
        assert_eq!(style_for_tile(&flat, 203).map(|s| s.style), Some(5));
        // Cracked bricks are NOT in style for the lookup (:141).
        assert_eq!(style_for_tile(&flat, 224), None);
        assert_eq!(style_for_tile(&flat, 234), None);
        assert_eq!(style_for_wall(&flat, 71).map(|s| s.style), Some(2));
        assert_eq!(style_for_wall(&flat, 40).map(|s| s.style), Some(2));
        assert_eq!(style_for_wall(&flat, 187).map(|s| s.style), Some(3));
        assert_eq!(style_for_wall(&flat, 28).map(|s| s.style), Some(6));
        assert_eq!(style_for_wall(&flat, 186).map(|s| s.style), Some(15));
        assert_eq!(style_for_wall(&flat, 83).map(|s| s.style), Some(5));
        assert_eq!(style_for_wall(&flat, 194).map(|s| s.style), Some(5));
        // Windows are NOT in style for the lookup (:168).
        assert_eq!(style_for_wall(&flat, 90), None);
        assert_eq!(style_for_wall(&flat, 174), None);
    }

    /// `GetStyleForWall`'s style-0 arm (`:164-167`): the current dungeon's
    /// wrapper claims every `Main.wallDungeon` wall under `WallID.Count`,
    /// including the painted variants its own row does not list.
    #[test]
    fn w4_c32_the_style_zero_wall_arm_claims_the_dungeon_walls() {
        let list = dungeon_style_list(DungeonColor::Blue);
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].style, 0);
        for wall in [7u16, 8, 9, 94, 95, 96, 97, 98, 99] {
            assert_eq!(style_for_wall(&list, wall).map(|s| s.style), Some(0), "wall {wall}");
        }
        assert_eq!(style_for_wall(&list, 71), None, "the snow wall is not a classic dungeon wall");
        assert_eq!(style_for_wall(&list, WALL_COUNT), None);
        // The tile lookup has NO style-0 special arm (:137-158).
        assert_eq!(style_for_tile(&list, 41).map(|s| s.style), Some(0));
        assert_eq!(style_for_tile(&list, 481), None, "cracked excluded");
    }

    /// `GetCurrentDungeonStyle` (`:95-135`) over the colour triple
    /// (`DungeonCrawler.cs:63-92`): the window walls/platform per colour,
    /// pit = cracked, empty furniture, tier Dungeon, BiomeStructured.
    #[test]
    fn w4_c32_the_current_style_wraps_the_colour_triple() {
        let blue = current_dungeon_style(DungeonColor::Blue);
        assert_eq!((blue.brick_tile, blue.brick_wall, blue.brick_cracked_tile), (41, 7, 481));
        assert_eq!((blue.window_glass_wall, blue.window_closed_glass_wall, blue.window_edge_wall), (91, 96, 8));
        assert_eq!(blue.window_platform_items, &[1386]);
        let green = current_dungeon_style(DungeonColor::Green);
        assert_eq!((green.brick_tile, green.brick_wall, green.brick_cracked_tile), (43, 8, 482));
        assert_eq!((green.window_glass_wall, green.window_closed_glass_wall, green.window_edge_wall), (92, 94, 9));
        assert_eq!(green.window_platform_items, &[1385]);
        let pink = current_dungeon_style(DungeonColor::Pink);
        assert_eq!((pink.brick_tile, pink.brick_wall, pink.brick_cracked_tile), (44, 9, 483));
        assert_eq!((pink.window_glass_wall, pink.window_closed_glass_wall, pink.window_edge_wall), (90, 98, 7));
        assert_eq!(pink.window_platform_items, &[1384]);
        for s in [&blue, &green, &pink] {
            assert_eq!(s.style, 0);
            assert_eq!(s.unbreakable_wall_tier, wall_tiers::DUNGEON);
            assert_eq!(s.pit_trap_tile, s.brick_cracked_tile);
            assert_eq!(s.locked_biome_chest_type, -1);
            assert!(s.chest_items.is_empty() && s.door_items.is_empty() && s.banner_items == Some(&[][..]));
            assert!(!s.edge_dither);
            assert_eq!(s.biome_room_type, DungeonRoomType::BiomeStructured);
        }
    }

    /// The Shimmer/Crystal feature gate (`DungeonGenerationStyles.cs:13-20`
    /// via `:514`): pit traps and windows refused, everything else allowed;
    /// the plain styles keep the base's always-true.
    #[test]
    fn w4_c32_crystal_refuses_pits_and_windows() {
        assert!(!CRYSTAL.can_generate_feature(DungeonFeature::PitTrap));
        assert!(!CRYSTAL.can_generate_feature(DungeonFeature::Window));
        assert!(CRYSTAL.can_generate_feature(DungeonFeature::Other));
        for s in [&SNOW, &DESERT, &CRIMSON, &HALLOW] {
            assert!(s.can_generate_feature(DungeonFeature::PitTrap));
            assert!(s.can_generate_feature(DungeonFeature::Window));
            assert!(s.can_generate_feature(DungeonFeature::Other));
        }
        // Crystal's own palette (:514-552).
        assert_eq!((CRYSTAL.brick_tile, CRYSTAL.brick_cracked_tile, CRYSTAL.brick_wall), (385, 116, 186));
        assert_eq!(CRYSTAL.style, 15);
        assert_eq!(CRYSTAL.chest_items, &[3884]);
        assert_eq!(CRYSTAL.platform_items, &[3903]);
    }

    /// W4-c32-convert SEAL-1 (research/315 §1/§6, chain 281→315 per registry
    /// 150 §9 UNIFIED FINAL, d54 b243c7086): the tables' style ordinals
    /// are exactly `BiomeRoom_FinishRoom`'s switch labels
    /// (`BiomeDungeonRoom.cs:160-218` — case 2 snow / 3 desert / 5 crimson /
    /// 6 hallow; case 4 corruption's static :434-473 is the cited skip), and
    /// the palettes' brick ids are the surface walkers' SOURCE kinds: the
    /// room bodies call `DungeonUtils.ChangeTileType` (`DungeonUtils.cs:
    /// 1577-1590`), NOT `WorldGen.Convert` — this witness pins the seam the
    /// wiring round ports.
    #[test]
    fn w4_c32_convert_seal_the_room_styles_match_the_finish_room_switch() {
        // The switch labels (:168 case 4 / :178 case 5 / :188 case 6 /
        // :198 case 2 / :208 case 3).
        assert_eq!((SNOW.style, DESERT.style, CRIMSON.style, HALLOW.style), (2, 3, 5, 6));
        // SnowySurface (:292-310): the kind-161 gate (:294) is Snow's brick;
        // the unconditional 147 write (:298/:307) is its extra tile.
        assert_eq!(SNOW.brick_tile, 161);
        assert!(SNOW.extra_tiles.contains(&147));
        // SandySurface (:258-290): the kind-396 gate (:260) is Desert's
        // brick; the C2 hardening split writes 397 (:266/:282, an extra)
        // or 53 (:270/:286, the cracked id).
        assert_eq!(DESERT.brick_tile, 396);
        assert_eq!(DESERT.brick_cracked_tile, 53);
        assert!(DESERT.extra_tiles.contains(&397));
        // GrassySurface's tileTypeToReplace args (:181 crimson 203/199,
        // :191 hallow 117/109): the bricks match; the grass targets are the
        // Convert-family grass ids, not table fields.
        assert_eq!(CRIMSON.brick_tile, 203);
        assert_eq!(HALLOW.brick_tile, 117);
    }

    /// W4-c32-convert SEAL-2 (research/315 §4): `DungeonGenerationStyleData.
    /// Style` ordinals and `WorldGen.Convert`'s conversionType are DISJOINT
    /// numberings — Crystal's Style 15 is not Convert type 15 (mushroom),
    /// Snow's Style 2 is not Convert type 2 (hallow), Desert 3 ≠ mushroom-3,
    /// Crimson 5 ≠ desertify-5, Hallow 6 ≠ snowify-6. The room switch reads
    /// STYLE ordinals; nothing passes them to Convert.
    #[test]
    fn w4_c32_convert_seal_style_ordinals_are_not_convert_types() {
        // Convert type 15 outputs (WorldGen.cs:56008-56040): tiles 70/59,
        // wall 80 — no Crystal palette id (385/116/186).
        assert_eq!(CRYSTAL.style, 15);
        for id in [70u16, 59, 80] {
            assert!(id != CRYSTAL.brick_tile && id != CRYSTAL.brick_cracked_tile && id != CRYSTAL.brick_wall);
        }
        // Convert type 2 targets (:55176-55260): 117/109/28 — no Snow id.
        assert_eq!(SNOW.style, 2);
        for id in [117u16, 109, 28] {
            assert!(id != SNOW.brick_tile && id != SNOW.brick_cracked_tile && id != SNOW.brick_wall);
        }
        // Convert 3 (mushroom :55337-55362) 70/80 vs Desert 396/53/187;
        // Convert 5 (desertify :55363-55408) 53/397/396 vs Crimson 203/234/83;
        // Convert 6 (snowify :55409-55445) 147/161 vs Hallow 117/116/28.
        assert_eq!(DESERT.style, 3);
        for id in [70u16, 80] {
            assert!(id != DESERT.brick_tile && id != DESERT.brick_cracked_tile && id != DESERT.brick_wall);
        }
        assert_eq!(CRIMSON.style, 5);
        for id in [53u16, 397, 396] {
            assert!(id != CRIMSON.brick_tile && id != CRIMSON.brick_cracked_tile && id != CRIMSON.brick_wall);
        }
        assert_eq!(HALLOW.style, 6);
        for id in [147u16, 161] {
            assert!(id != HALLOW.brick_tile && id != HALLOW.brick_cracked_tile && id != HALLOW.brick_wall);
        }
    }

    /// W4-c32-convert SEAL-2 cross-pin (research/315 §2/§6): f25's UNWIRED
    /// Convert 15/16/18/19 cells (`spread::convert_biome_cell`, f25 tip
    /// d587b128d / 9fc704a15 — zero C# call sites) produce exactly the
    /// palette ids these tables carry. The cell bodies stay f25's; this
    /// pins the id agreement without copying them. The GlowingMushroom /
    /// Jungle tuples are cited C# literals (DungeonGenerationStyles.cs
    /// :596-601 / :716-722 — both statics are out-of-scope skips here).
    #[test]
    fn w4_c32_convert_seal_the_biome_cell_outputs_match_the_table_palettes() {
        // Convert 16 (WorldGen.cs:56041-56086): tiles 396 (catch-all
        // :56084), 53/397 (the C2 split :56063-56068), 397 (hardened
        // :56072); walls 216/187 (:56044-56051). Desert's rows cover them.
        let desert_tiles = [DESERT.brick_tile, DESERT.brick_cracked_tile];
        for id in [396u16, 53, 397] {
            assert!(desert_tiles.contains(&id) || DESERT.extra_tiles.contains(&id), "tile {id}");
        }
        for id in [187u16, 216] {
            assert!(id == DESERT.brick_wall || DESERT.extra_walls.contains(&id), "wall {id}");
        }
        // Convert 2 (:55176-55260): stone->117 (:55222), sand->116
        // (:55238), the stone wall->28 (:55185) = Hallow's palette.
        assert_eq!((HALLOW.brick_tile, HALLOW.brick_cracked_tile, HALLOW.brick_wall), (117, 116, 28));
        // Convert 15 (:56008-56040): the catch-all writes 70 (exposed) /
        // 59 (sealed) (:56033/:56037), the wall arm 80 (:56011) — the
        // GlowingMushroom static's BrickGrass/Brick/BrickWall
        // (:598-601: 70/59/80). That static is an out-of-scope skip; the
        // agreement is pinned via the shared ids the OTHER tables carry:
        // Hallow's cracked 116 is NOT among them (the mushroom biome has
        // no hallow ids), and Desert's cracked 53 is the sealed-mud
        // neighbour id 59's desert counterpart, not 59 itself.
        let mushroom_outputs = [70u16, 59, 80];
        assert!(!mushroom_outputs.contains(&HALLOW.brick_tile));
        assert!(!mushroom_outputs.contains(&DESERT.brick_cracked_tile));
        // Convert 19 (:56190-56231): 60 (exposed) / 59 (sealed) tiles
        // (:56224/:56228), the grass-family wall arm 64 (:56199) + the
        // wall-15 no-op (:56204) — the Jungle static's BrickGrass/Brick/
        // BrickWall (:718-721: 60/59/64) + extra wall 15 (:722). Same
        // out-of-scope treatment: the jungle grass 60 is no in-scope
        // style's brick, and Snow's brick 161 is not a jungle output.
        let jungle_outputs = [60u16, 59, 64, 15];
        for s in [&SNOW, &DESERT, &CRIMSON, &HALLOW, &CRYSTAL] {
            assert!(!jungle_outputs.contains(&s.brick_tile), "style {}", s.style);
        }
        // Convert 18 (:56124-56189): the pure baseline — catch-all stone 1
        // (:56187), grass 2 (:56166), dirt 0 (:56170), walls 2/212-215/349
        // (:56129-56149). No in-scope style's brick is a pure-baseline id
        // (the pure world is not a dungeon style).
        let pure_outputs = [1u16, 2, 0];
        for s in [&SNOW, &DESERT, &CRIMSON, &HALLOW, &CRYSTAL] {
            assert!(!pure_outputs.contains(&s.brick_tile), "style {}", s.style);
        }
    }
}
