//! Scene sensors and state: `SceneMetrics` (the tile/window sensor), the
//! `Sandstorm` event and `SceneState` (the per-frame fades/intensities).
//!
//! Sources:
//! * `SceneMetrics` — `decompiled/Terraria/SceneMetrics.cs`.
//! * `Sandstorm` — `decompiled/Terraria/GameContent/Events/Sandstorm.cs`.
//! * `SceneState` — `decompiled/Terraria/SceneState.cs`.
//!
//! Data only: no render types cross this module. The tile-window scan that
//! feeds [`SceneMetrics`] lives in `tr-app` (`world_sim::scene_scan`) because
//! the scan is centered on the player, which is a `tr-app` type.

use crate::tile_light::FastRandom;

/// How many tile kinds the window counts cover (`_tileCounts`, `TileID.Count`).
const TILE_COUNT: usize = crate::tiledata::TILE_COUNT;

/// `SceneMetrics` — the M1 sensor subset: the scan window counts around the
/// player, the derived biome counts and the zone flags the screen filters read.
///
/// Fields and thresholds follow `SceneMetrics.cs`; the full type carries many
/// more counters (NPC positions, candles, monoliths) that no M1 filter reads —
/// add them when a reader lands.
#[derive(Clone, Debug, PartialEq)]
pub struct SceneMetrics {
    /// `metrics.Center`: the scan center in world pixels (the player center).
    pub center: [f32; 2],
    /// `metrics.TileCenter`: `Center.ToTileCoordinates().ClampedInWorld()`.
    pub tile_center: (i32, i32),
    /// `metrics.BehindBackwall`: the TileCenter tile has a wall behind it
    /// (`SceneMetrics.cs:671-672`).
    pub behind_backwall: bool,
    /// `_tileCounts[tile.type]` per active tile in the scan window
    /// (`SceneMetrics.cs:353-373`).
    pub tile_counts: Vec<u32>,
    /// `metrics.OceanSandTileCount` (`SceneMetrics.cs:377-379`).
    pub ocean_sand_tile_count: i32,
    /// `metrics.SandTileCount` (`SceneMetrics.cs:613`).
    pub sand_tile_count: i32,
    /// `metrics.HolyTileCount` (`SceneMetrics.cs:589`).
    pub holy_tile_count: i32,
    /// `metrics.SnowTileCount` (`SceneMetrics.cs:590`).
    pub snow_tile_count: i32,
    /// `metrics.GraveyardTileCount` (`SceneMetrics.cs:615-624`).
    pub graveyard_tile_count: i32,
    /// `metrics.DesertSandTileCount` (`SceneMetrics.cs:658`).
    pub desert_sand_tile_count: i32,
    /// `metrics.JungleTileCount` (`SceneMetrics.cs:610-612`, non-remix branch).
    pub jungle_tile_count: i32,
    /// `metrics.EvilTileCount` (`SceneMetrics.cs:613`, non-remix branch).
    pub evil_tile_count: i32,
    /// `metrics.BloodTileCount` (`SceneMetrics.cs:614`, non-remix branch).
    pub blood_tile_count: i32,
    /// `metrics.SurfaceAtmospherics` (`SceneMetrics.cs:698`, `WorldGen.cs:10976`).
    pub surface_atmospherics: bool,
    /// `metrics.ZoneRain` (`SceneMetrics.cs:701`).
    pub zone_rain: bool,
    /// `metrics.ZoneSnow` (`SceneMetrics.cs:675`).
    pub zone_snow: bool,
    /// `metrics.ZoneDesert` (`SceneMetrics.cs:676`).
    pub zone_desert: bool,
    /// `metrics.ZoneSandstorm` (`SceneMetrics.cs:703-707`).
    pub zone_sandstorm: bool,
    /// `metrics.ZoneCorrupt` (`SceneMetrics.cs:671`).
    pub zone_corrupt: bool,
    /// `metrics.ZoneCrimson` (`SceneMetrics.cs:672`).
    pub zone_crimson: bool,
    /// `metrics.ZoneHallow` (`SceneMetrics.cs:673`).
    pub zone_hallow: bool,
    /// `metrics.ZoneJungle` (`SceneMetrics.cs:674`).
    pub zone_jungle: bool,
    /// `metrics.ZoneBeach` (`SceneMetrics.cs:691`, `WorldGen.oceanDepths`).
    pub zone_beach: bool,
    /// `metrics.ShimmerTileCount` = `_liquidCounts[LiquidID.Shimmer]`
    /// (`SceneMetrics.cs:594`): the type-3 liquid cells in the scan window.
    pub shimmer_tile_count: i32,
    /// `metrics.MushroomTileCount` (`SceneMetrics.cs:610` — the blocks
    /// 70/71/72/528).
    pub mushroom_tile_count: i32,
    /// `metrics.UndergroundForShimmering` (`SceneMetrics.cs:704`).
    pub underground_for_shimmering: bool,
    /// `metrics.ZoneShimmer` (`SceneMetrics.cs:705`; the dungeon gate is not
    /// ported — no dungeon zone system yet).
    pub zone_shimmer: bool,
    /// `metrics.PerspectivePlayer.shimmering` via `AddPlayerEffects`
    /// (`SceneMetrics.cs:747+`): the local player stands in shimmer liquid.
    pub player_shimmering: bool,
    // — the M2-BUFF-5 auras (`SceneMetrics.ScanTiles`, `:378-409`) —
    /// `HasCampfire`: a `Campfires` tile (215) with `frameY < 36` (the lit
    /// frames) or tile 405 with `frameX < 54` (`:378-380, 396-398`).
    pub has_campfire: bool,
    /// `WaterCandleCount` — tile 49, `frameX < 18` (`:382-385`).
    pub water_candle_count: i32,
    /// `PeaceCandleCount` — tile 372, `frameX < 18` (`:386-389`).
    pub peace_candle_count: i32,
    /// `ShadowCandleCount` — tile 646, `frameX < 18` (`:390-393`).
    pub shadow_candle_count: i32,
    /// `HasCatBast` — tile 506, `frameX < 72` (`:400-402`).
    pub has_cat_bast: bool,
    /// `HasHeartLantern` — tile 42, `frameY 324-358` (`:404-406`).
    pub has_heart_lantern: bool,
    /// `HasStarInBottle` — tile 42, `frameY 252-286` (`:407-409`).
    pub has_star_in_bottle: bool,
    /// `HasSunflower` — any tile 27 in the window, cancelled by a graveyard
    /// above `GraveyardTileMin` (`AggregateTileCounts`, `:617-624`).
    pub has_sunflower: bool,
    // — the M2-BUFF-8 banner fields (`SceneMetrics.cs:442-445`) —
    /// `NPCBannerBuff[]`: the banner ids present in the scan window.
    pub npc_banner_buff: Vec<bool>,
    /// `hasBanner` — any strength-enabled banner in the window.
    pub has_banner: bool,
    /// `MeteorTileCount` — the meteorite brick (37) tiles (`:611`).
    pub meteor_tile_count: i32,
    /// `DungeonTileCount` — the brick families 41/43/44/481/482/483 (`:612`).
    pub dungeon_tile_count: i32,
    /// `ZoneMeteor` (`:678`).
    pub zone_meteor: bool,
    /// `ZoneGraveyard` (`:679`).
    pub zone_graveyard: bool,
    /// `ZoneDungeon` (`:682-684`) — the depth + wall gates fill in the
    /// scan driver (they read the centre tile's wall).
    pub zone_dungeon: bool,
    // — the M6-AUDIO-2 music-box fields (`SceneMetrics.cs:4489-4501`) —
    /// `ActiveMusicBox`: the box id playing in the window (`frameY / 36`),
    /// -1 = none.
    pub active_music_box: i32,
    /// `MusicBoxSilence` — the silent box (`frameY / 36 == 100`).
    pub music_box_silence: bool,
}

/// The `Player.SceneMetrics` aura read as a Copy snapshot — what
/// `UpdateLifeRegen` (`:19170-19178`), `UpdateManaRegen` (`:19448`) and the
/// scene-buff grants (`:25525-25555`) consume.
#[derive(Clone, Copy, Default, Debug, PartialEq)]
pub struct Auras {
    pub campfire: bool,
    pub heart_lantern: bool,
    pub star_in_bottle: bool,
    pub sunflower: bool,
    pub cat_bast: bool,
    pub shadow_candle: bool,
}

/// `SceneMetrics.AssumedConstantScreenSize` / `ZoneScanPadding` /
/// `ZoneScanSize` (`SceneMetrics.cs:915-919`): 1920/16 + 25*2 - 1 = 169 wide,
/// 1200/16 + 25*2 - 1 = 124 tall.
pub const ZONE_SCAN_SIZE: (i32, i32) = (1920 / 16 + 25 * 2 - 1, 1200 / 16 + 25 * 2 - 1);

/// The biome-zone flags the player reads as a Copy snapshot — the torch-luck
/// verdict table's inputs (`Player.ZoneSnow`-family, `Player.cs:17924-18036`).
/// `ZoneLihzhardTemple` is unmodelled (no jungle-temple content in the port),
/// so the C# skip gate `ZoneDungeon || ZoneLihzhardTemple` (`:17923`) runs on
/// dungeon only — deviation recorded in ANALYSIS.
#[derive(Clone, Copy, Default, Debug, PartialEq)]
pub struct Zones {
    pub snow: bool,
    pub desert: bool,
    pub corrupt: bool,
    pub crimson: bool,
    pub hallow: bool,
    pub jungle: bool,
    /// `ZoneGlowshroom` — `EnoughTilesForGlowingMushroom` (`:698-701`).
    pub glowshroom: bool,
    pub dungeon: bool,
}

/// The static thresholds from `SceneMetrics`'s static ctor (`:922-943`).
pub mod thresholds {
    /// `SnowTileNormalThreshold` (`SceneMetrics.cs:930`).
    pub const SNOW_TILE_NORMAL: i32 = 1500;
    /// `DesertTileNormalThreshold` (`SceneMetrics.cs:934`).
    pub const DESERT_TILE_NORMAL: i32 = 1500;
    /// `GraveyardTileThreshold` (`SceneMetrics.cs:941`).
    pub const GRAVEYARD_TILE: i32 = 28;
    /// `GraveyardTileMin` (`SceneMetrics.cs:940`).
    pub const GRAVEYARD_TILE_MIN: i32 = 16;
    /// `GraveyardTileMax` (`SceneMetrics.cs:939`).
    pub const GRAVEYARD_TILE_MAX: i32 = 36;
    /// `CorruptionTileThreshold` (`SceneMetrics.cs:924`).
    pub const CORRUPTION_TILE: i32 = 300;
    /// `CrimsonTileThreshold` (`SceneMetrics.cs:926`).
    pub const CRIMSON_TILE: i32 = 300;
    /// `HallowTileThreshold` (`SceneMetrics.cs:927`).
    pub const HALLOW_TILE: i32 = 125;
    /// `JungleTileThreshold` (`SceneMetrics.cs:929`).
    pub const JUNGLE_TILE: i32 = 140;
    /// `ShimmerTileThreshold` (`SceneMetrics.cs:921`).
    pub const SHIMMER_TILE: i32 = 300;
    /// `MushroomTileThreshold` (`SceneMetrics.cs:935`).
    pub const MUSHROOM_TILE: i32 = 100;
    /// `MeteorTileThreshold` (`SceneMetrics.cs:937`).
    pub const METEOR_TILE: i32 = 75;
    /// `DungeonTileThreshold` (`SceneMetrics.cs:938`).
    pub const DUNGEON_TILE: i32 = 250;
}

/// `Main.wallDungeon` (`Main.cs:10596-10607`): the brick/background walls
/// the dungeon zone requires behind the scan centre.
pub fn is_dungeon_wall(wall: u16) -> bool {
    matches!(wall, 7 | 8 | 9 | 94..=99)
}

impl SceneMetrics {
    /// `SceneMetrics.Reset` — zeroed counts and flags.
    pub fn new() -> Self {
        Self {
            center: [0.0; 2],
            tile_center: (0, 0),
            behind_backwall: false,
            shimmer_tile_count: 0,
            mushroom_tile_count: 0,
            underground_for_shimmering: false,
            zone_shimmer: false,
            player_shimmering: false,
            tile_counts: vec![0; TILE_COUNT],
            ocean_sand_tile_count: 0,
            sand_tile_count: 0,
            holy_tile_count: 0,
            snow_tile_count: 0,
            graveyard_tile_count: 0,
            desert_sand_tile_count: 0,
            jungle_tile_count: 0,
            evil_tile_count: 0,
            blood_tile_count: 0,
            surface_atmospherics: false,
            zone_rain: false,
            zone_snow: false,
            zone_desert: false,
            zone_sandstorm: false,
            zone_corrupt: false,
            zone_crimson: false,
            zone_hallow: false,
            zone_jungle: false,
            zone_beach: false,
            has_campfire: false,
            water_candle_count: 0,
            peace_candle_count: 0,
            shadow_candle_count: 0,
            has_cat_bast: false,
            has_heart_lantern: false,
            has_star_in_bottle: false,
            has_sunflower: false,
            npc_banner_buff: vec![false; 320],
            has_banner: false,
            meteor_tile_count: 0,
            dungeon_tile_count: 0,
            zone_meteor: false,
            zone_graveyard: false,
            zone_dungeon: false,
            active_music_box: -1,
            music_box_silence: false,
        }
    }

    /// The [`Auras`] snapshot of the scan.
    pub fn auras(&self) -> Auras {
        Auras {
            campfire: self.has_campfire,
            heart_lantern: self.has_heart_lantern,
            star_in_bottle: self.has_star_in_bottle,
            sunflower: self.has_sunflower,
            cat_bast: self.has_cat_bast,
            shadow_candle: self.shadow_candle_count > 0,
        }
    }

    /// The zone-flag snapshot the torch-luck table consumes (see [`Zones`]).
    pub fn zones(&self) -> Zones {
        Zones {
            snow: self.zone_snow,
            desert: self.zone_desert,
            corrupt: self.zone_corrupt,
            crimson: self.zone_crimson,
            hallow: self.zone_hallow,
            jungle: self.zone_jungle,
            glowshroom: self.enough_tiles_for_glowing_mushroom(),
            dungeon: self.zone_dungeon,
        }
    }

    /// `metrics.EnoughTilesForSnow` (`SceneMetrics.cs:258`; threshold
    /// `SnowTileNormalThreshold`, `:930` — the Skyblock variant is not ported).
    pub fn enough_tiles_for_snow(&self) -> bool {
        self.snow_tile_count >= thresholds::SNOW_TILE_NORMAL
    }

    /// `metrics.EnoughTilesForDesert` (`SceneMetrics.cs:262`; threshold
    /// `DesertTileNormalThreshold`, `:934`).
    pub fn enough_tiles_for_desert(&self) -> bool {
        self.desert_sand_tile_count >= thresholds::DESERT_TILE_NORMAL
    }

    /// `metrics.EnoughTilesForGraveyard` (`SceneMetrics.cs:272`; threshold
    /// `GraveyardTileThreshold`, `:941`).
    pub fn enough_tiles_for_graveyard(&self) -> bool {
        self.graveyard_tile_count >= thresholds::GRAVEYARD_TILE
    }

    /// `metrics.EnoughTilesForJungle` (`SceneMetrics.cs:251`).
    pub fn enough_tiles_for_jungle(&self) -> bool {
        self.jungle_tile_count >= thresholds::JUNGLE_TILE
    }

    /// `metrics.EnoughTilesForHallow` (`SceneMetrics.cs:253`).
    pub fn enough_tiles_for_hallow(&self) -> bool {
        self.holy_tile_count >= thresholds::HALLOW_TILE
    }

    /// `metrics.EnoughTilesForCorruption` (`SceneMetrics.cs:265`).
    pub fn enough_tiles_for_corruption(&self) -> bool {
        self.evil_tile_count >= thresholds::CORRUPTION_TILE
    }

    /// `metrics.EnoughTilesForCrimson` (`SceneMetrics.cs:267`).
    pub fn enough_tiles_for_crimson(&self) -> bool {
        self.blood_tile_count >= thresholds::CRIMSON_TILE
    }

    /// `metrics.EnoughTilesForShimmer` (`SceneMetrics.cs:249`): the type-3
    /// liquid cells in the window (the scan's liquid counting feeds it).
    pub fn enough_tiles_for_shimmer(&self) -> bool {
        self.shimmer_tile_count >= thresholds::SHIMMER_TILE
    }

    /// `metrics.EnoughTilesForGlowingMushroom` (`SceneMetrics.cs:257`).
    pub fn enough_tiles_for_glowing_mushroom(&self) -> bool {
        self.mushroom_tile_count >= thresholds::MUSHROOM_TILE
    }

    /// `SceneMetrics.AggregateTileCounts` (`SceneMetrics.cs:573-660`), M1
    /// subset. Mushroom/Meteor/Dungeon/Shimmer counts and the `HasSunflower`/
    /// `HasGardenGnome` markers have no M1 reader and stay unported; the
    /// `infectedSeed ×3` sunflower scaling (`:576-579`) and the remixWorld
    /// count branch (`:604-609`) belong to seeds the port does not generate.
    pub fn aggregate_tile_counts(&mut self) {
        // `num = -10` (`:575`): each sunflower counts as −10 evil/blood tiles
        // (vanilla's sunflower suppression). `infectedSeed` would scale it ×3.
        let sunflower_term: i32 = self.count(27) * -10;
        // `SceneMetrics.cs:589`:
        self.holy_tile_count = self.count(109)
            + self.count(492)
            + self.count(110)
            + self.count(113)
            + self.count(117)
            + self.count(116)
            + self.count(164)
            + self.count(403)
            + self.count(402);
        // `SceneMetrics.cs:590`:
        self.snow_tile_count = self.count(147)
            + self.count(148)
            + self.count(161)
            + self.count(162)
            + self.count(164)
            + self.count(163)
            + self.count(200);
        // `SceneMetrics.cs:610-614`, non-remix branch (`:616-619`):
        self.jungle_tile_count = self.count(60)
            + self.count(61)
            + self.count(62)
            + self.count(74)
            + self.count(226)
            + self.count(225);
        self.evil_tile_count = self.count(23)
            + self.count(661)
            + self.count(24)
            + self.count(25)
            + self.count(32)
            + self.count(112)
            + self.count(163)
            + self.count(400)
            + self.count(398)
            + sunflower_term;
        self.blood_tile_count = self.count(199)
            + self.count(662)
            + self.count(201)
            + self.count(203)
            + self.count(200)
            + self.count(401)
            + self.count(399)
            + self.count(234)
            + self.count(352)
            + sunflower_term;
        // `SceneMetrics.cs:610`: the glowing-mushroom block set.
        self.mushroom_tile_count =
            self.count(70) + self.count(71) + self.count(72) + self.count(528);
        // `SceneMetrics.cs:611-612`:
        self.meteor_tile_count = self.count(37);
        self.dungeon_tile_count = self.count(41)
            + self.count(43)
            + self.count(44)
            + self.count(481)
            + self.count(482)
            + self.count(483);
        // `SceneMetrics.cs:625`:
        self.sand_tile_count = self.count(53)
            + self.count(112)
            + self.count(116)
            + self.count(234)
            + self.count(397)
            + self.count(398)
            + self.count(402)
            + self.count(399)
            + self.count(396)
            + self.count(400)
            + self.count(403)
            + self.count(401);
        // `SceneMetrics.cs:615-624`: tombstones minus half the sunflowers
        // (integer division), floored at zero; `HasSunflower` rides the same
        // counts (`:617-624`) — any tile 27 raises it, a graveyard above
        // `GraveyardTileMin` (16) cancels it.
        self.graveyard_tile_count = self.count(85) - self.count(27) / 2;
        self.has_sunflower = self.count(27) > 0;
        if self.graveyard_tile_count < 0 {
            self.graveyard_tile_count = 0;
        }
        if self.graveyard_tile_count > thresholds::GRAVEYARD_TILE_MIN {
            self.has_sunflower = false;
        }
        // `SceneMetrics.cs:629-639`: clamp each sum at zero before the
        // cross-subtraction.
        self.holy_tile_count = self.holy_tile_count.max(0);
        self.evil_tile_count = self.evil_tile_count.max(0);
        self.blood_tile_count = self.blood_tile_count.max(0);
        // `SceneMetrics.cs:645-653`: the holy/evil/blood cross-subtraction, in
        // the original order, then clamp again (`:655-664`).
        let holy_before = self.holy_tile_count;
        self.holy_tile_count -= self.evil_tile_count;
        self.holy_tile_count -= self.blood_tile_count;
        self.evil_tile_count -= holy_before;
        self.blood_tile_count -= holy_before;
        self.holy_tile_count = self.holy_tile_count.max(0);
        self.evil_tile_count = self.evil_tile_count.max(0);
        self.blood_tile_count = self.blood_tile_count.max(0);
        // `SceneMetrics.cs:658`: `Math.Max(0, SandTileCount - OceanSandTileCount)`.
        self.desert_sand_tile_count = (self.sand_tile_count - self.ocean_sand_tile_count).max(0);
    }

    /// `_tileCounts[id]` as an `i32` summand.
    fn count(&self, id: u16) -> i32 {
        self.tile_counts[id as usize] as i32
    }

    /// `SceneMetrics.CalculateZones` (`SceneMetrics.cs:662-707`), M1 subset.
    /// `world_surface` is `Main.worldSurface`; `underworld_layer` is
    /// `Main.UnderworldLayer`; `rock_layer` is `Main.rockLayer`; `max_tiles_x`
    /// is `Main.maxTilesX`; `raining` is `Main.raining`; `sandstorm_happening`
    /// is `Sandstorm.Happening`. The depth flags the heat/water filters read
    /// (`ZoneOverworldHeight`, …) derive straight from `tile_center` in the
    /// phase-3 driver. The `drunkWorld` dungeon override (`:683-686`) and the
    /// remix atmospherics override (`:699-701`) belong to seeds the port does
    /// not generate.
    pub fn calculate_zones(
        &mut self,
        world_surface: f64,
        underworld_layer: i32,
        rock_layer: f64,
        max_tiles_x: i32,
        max_tiles_y: i32,
        raining: bool,
        sandstorm_happening: bool,
    ) {
        // `SceneMetrics.cs:671-674`:
        self.zone_corrupt = self.evil_tile_count >= thresholds::CORRUPTION_TILE;
        self.zone_crimson = self.blood_tile_count >= thresholds::CRIMSON_TILE;
        self.zone_hallow = self.holy_tile_count >= thresholds::HALLOW_TILE;
        let zone_underworld_height = self.tile_center.1 > underworld_layer;
        self.zone_jungle =
            self.jungle_tile_count >= thresholds::JUNGLE_TILE && !zone_underworld_height;
        // `SceneMetrics.cs:675-676`:
        self.zone_snow = self.enough_tiles_for_snow();
        self.zone_desert = self.enough_tiles_for_desert();
        // `SceneMetrics.cs:678-679`:
        self.zone_meteor = self.meteor_tile_count >= thresholds::METEOR_TILE;
        self.zone_graveyard = self.enough_tiles_for_graveyard();
        // `SceneMetrics.cs:691` via `WorldGen.oceanDepths` (`WorldGen.cs:4447`,
        // ported as [`ocean_depths`]):
        self.zone_beach = ocean_depths(
            world_surface,
            rock_layer,
            self.tile_center.0,
            max_tiles_x,
            self.tile_center.1,
        );
        // `SceneMetrics.cs:698` via `WorldGen.IsSurfaceForAtmospherics`
        // (`WorldGen.cs:10976-10985`; the remix branch is not ported):
        self.surface_atmospherics = self.tile_center.1 as f64 <= world_surface;
        // `SceneMetrics.cs:701`:
        self.zone_rain = raining && self.surface_atmospherics;
        // `SceneMetrics.cs:703`:
        self.zone_sandstorm = self.zone_desert && self.surface_atmospherics && sandstorm_happening;
        // `SceneMetrics.cs:704-707`: a sandstorm suppresses the rain zone.
        if self.zone_sandstorm {
            self.zone_rain = false;
        }
        // `SceneMetrics.cs:704-705`: the shimmer depth window and the zone —
        // `EnoughTilesForShimmer` is `_liquidCounts[3] >= 300`
        // (`:252`, threshold `:921`); the `!ZoneDungeon` term waits for the
        // dungeon zone (M4-NPC families).
        self.underground_for_shimmering = (self.tile_center.1 as f64) > world_surface + 84.0
            && (self.tile_center.1 as i64) < max_tiles_y as i64 - 396;
        self.zone_shimmer =
            self.shimmer_tile_count >= thresholds::SHIMMER_TILE && self.underground_for_shimmering;
    }
}

/// `WorldGen.oceanDepths` (`WorldGen.cs:…`): the beach bands at the world's
/// left/right edges, above `oceanLevel` = `(worldSurface + rockLayer)/2 + 40`
/// (`WorldGen.cs:4447`). `WorldGen.beachDistance` = 380 (`WorldGen.cs:86914`).
pub fn ocean_depths(world_surface: f64, rock_layer: f64, x: i32, max_tiles_x: i32, y: i32) -> bool {
    const BEACH_DISTANCE: i32 = 380;
    let ocean_level = (world_surface + rock_layer) / 2.0 + 40.0;
    if y as f64 > ocean_level {
        return false;
    }
    x < BEACH_DISTANCE || x > max_tiles_x - BEACH_DISTANCE
}

/// `TileID.Sets.isDesertBiomeSand` (`TileID.cs:501`): the sands that count as
/// ocean/desert sand.
pub fn is_desert_biome_sand(kind: u16) -> bool {
    matches!(kind, 53 | 397 | 396 | 400 | 403 | 401)
}

/// The `Sandstorm` event (`Events/Sandstorm.cs`).
///
/// The Severity transitions are pure; the start/severity-intention rolls draw
/// from this struct's own `FastRandom` stream — vanilla pulls them from the
/// shared `Main.rand` (`Sandstorm.cs:64,71,83,91,120`). `Main.rand`'s
/// `UnifiedRandom` is now ported ([`crate::UnifiedRandom`], S1-A-1) but not yet
/// wired here (consumer migration is S1-A-2+), and an isolated stream keeps the
/// state machine deterministic.
/// `TR_SANDSTORM` (see [`SceneState::force`]) is the primary determinism lever.
pub struct Sandstorm {
    /// `Sandstorm.Happening` (`Sandstorm.cs:14`).
    pub happening: bool,
    /// `Sandstorm.TimeLeft` (`Sandstorm.cs:16`).
    pub time_left: i32,
    /// `Sandstorm.Severity` (`Sandstorm.cs:18`).
    pub severity: f32,
    /// `Sandstorm.IntendedSeverity` (`Sandstorm.cs:20`).
    pub intended_severity: f32,
    rng: FastRandom,
}

/// `SANDSTORM_DURATION_MINIMUM` / `_MAXIMUM` (`Sandstorm.cs:10-12`).
const SANDSTORM_DURATION_MIN: i32 = 28800;
const SANDSTORM_DURATION_MAX: i32 = 86400;

impl Sandstorm {
    /// A fresh, not-happening storm with its own RNG stream.
    pub fn new(seed: u64) -> Self {
        Self {
            happening: false,
            time_left: 0,
            severity: 0.0,
            intended_severity: 0.0,
            rng: FastRandom::new(seed),
        }
    }

    /// `Sandstorm.HasSufficientWind` (`Sandstorm.cs:22-25`).
    pub fn has_sufficient_wind(wind_speed_current: f32) -> bool {
        wind_speed_current.abs() >= 0.6
    }

    /// `Sandstorm.UpdateTime` (`Sandstorm.cs:32-77`). `day_rate` is
    /// `Main.dayRate` (1 in our port: no sleeping/creative powers yet);
    /// `wind_speed_current` is `Main.windSpeedCurrent`. The `Main.netMode != 1`
    /// gate (`:34`) is always true for a single-player session. The start roll
    /// uses the no-hardMode branch (`:58-59`: `num = 21600 * 3`; hardmode is
    /// not ported yet).
    pub fn update_time(&mut self, day_rate: i32, wind_speed_current: f32) {
        if self.happening {
            // `Sandstorm.cs:38-41`:
            if self.time_left > SANDSTORM_DURATION_MAX {
                self.time_left = 0;
            }
            // `Sandstorm.cs:42-46`: decay, plus the 15×/tick wind penalty.
            self.time_left -= day_rate;
            if !Self::has_sufficient_wind(wind_speed_current) {
                self.time_left -= 15 * day_rate;
            }
            // `Sandstorm.cs:47-50`: dead-calm wind ends the storm at once.
            if wind_speed_current == 0.0 {
                self.time_left = 0;
            }
            // `Sandstorm.cs:51-54`:
            if self.time_left <= 0 {
                self.stop_sandstorm();
            }
        } else {
            // `Sandstorm.cs:58-59`: start-delay denominator.
            let num = 21600 * 3;
            // `Sandstorm.cs:60-69`: one roll per day-rate tick.
            if Self::has_sufficient_wind(wind_speed_current) {
                for _ in 0..day_rate {
                    if self.rng.next_range(0, num) == 0 {
                        self.start_sandstorm();
                    }
                }
            }
        }
        // `Sandstorm.cs:71-74`:
        if self.rng.next_range(0, 18000) == 0 {
            self.change_severity_intentions();
        }
        // `Sandstorm.cs:76`:
        self.update_severity();
    }

    /// `Sandstorm.StartSandstorm` (`Sandstorm.cs:117-122`).
    pub fn start_sandstorm(&mut self) {
        self.happening = true;
        self.time_left = self
            .rng
            .next_range(SANDSTORM_DURATION_MIN, SANDSTORM_DURATION_MAX + 1);
        self.change_severity_intentions();
    }

    /// `Sandstorm.StopSandstorm` (`Sandstorm.cs:124-129`).
    pub fn stop_sandstorm(&mut self) {
        self.happening = false;
        self.time_left = 0;
        self.change_severity_intentions();
    }

    /// `Sandstorm.ChangeSeverityIntentions` (`Sandstorm.cs:79-97`); the
    /// `NetMessage.SendData(7)` tail (`:93-96`) is multiplayer-only.
    fn change_severity_intentions(&mut self) {
        if self.happening {
            // `Sandstorm.cs:83`: 0.4..1.0 while a storm runs.
            self.intended_severity = 0.4 + self.rng.next_float();
        } else if self.rng.next_range(0, 3) == 0 {
            // `Sandstorm.cs:85-88`: a third of the time it wants to stay clear.
            self.intended_severity = 0.0;
        } else {
            // `Sandstorm.cs:89-92`: otherwise a low ambient severity.
            self.intended_severity = self.rng.next_float() * 0.3;
        }
    }

    /// `Sandstorm.UpdateSeverity` (`Sandstorm.cs:99-115`): approach
    /// `intended_severity` at 0.003/tick, snapping on overshoot.
    pub fn update_severity(&mut self) {
        // `Sandstorm.cs:101-108`: NaN guards.
        if self.severity.is_nan() {
            self.severity = 0.0;
        }
        if self.intended_severity.is_nan() {
            self.intended_severity = 0.0;
        }
        // `Sandstorm.cs:109-110`: `Math.Sign` gives 0 on equality.
        let num = match self.intended_severity - self.severity {
            d if d > 0.0 => 1,
            d if d < 0.0 => -1,
            _ => 0,
        };
        self.severity = (self.severity + 0.003 * num as f32).clamp(0.0, 1.0);
        // `Sandstorm.cs:111-114`: crossed the target this tick — snap to it.
        let crossed = match self.intended_severity - self.severity {
            d if d > 0.0 => 1,
            d if d < 0.0 => -1,
            _ => 0,
        };
        if num != crossed {
            self.severity = self.intended_severity;
        }
    }

    /// `TR_SANDSTORM=<0..1>` pin: hold the severity at a fixed value with the
    /// storm running (value > 0). With `severity == intended_severity`,
    /// `update_severity` is a no-op, so the pin survives the state machine.
    pub fn force_severity(&mut self, value: f32) {
        let v = value.clamp(0.0, 1.0);
        self.severity = v;
        self.intended_severity = v;
        self.happening = v > 0.0;
    }

    /// `Sandstorm.ShowSandstormVisuals` (`Sandstorm.cs:131-138`): is the
    /// sandstorm filter/overlay visibly on. The bg inputs are the caller's
    /// (`Main.bgStyle` desert-variant check and `Main.bgDelay`).
    pub fn show_sandstorm_visuals(
        &self,
        zone_sandstorm: bool,
        bg_is_desert_variant: bool,
        bg_delay: i32,
    ) -> bool {
        if self.happening && zone_sandstorm && bg_is_desert_variant {
            return bg_delay < 50;
        }
        false
    }
}

/// `SceneState` (`SceneState.cs:14-62`) — the M1-owned fades and intensities.
/// The filter-parameter driver (`ApplyVisuals`) is phase 3 (`tr-app`); this
/// struct only mutates its own state.
pub struct SceneState {
    /// `SceneState.airLightDecay` (`SceneState.cs:15`).
    pub air_light_decay: f32,
    /// `SceneState.solidLightDecay` (`SceneState.cs:16`).
    pub solid_light_decay: f32,
    /// `SceneState.outsideWeatherEffectIntensity` (`SceneState.cs:20`).
    pub outside_weather_effect_intensity: f32,
    /// `SceneState._outsideWeatherEffectIntensityBackingValue` (`:22`).
    pub outside_weather_effect_intensity_backing: f32,
    /// `Main.GraveyardVisualIntensity` (`Main.cs:1327`): vanilla keeps it on
    /// Main, but `SceneState.UpdateGraveyard` (`:368-371`) is its only writer,
    /// so the port owns it here.
    pub graveyard_visual_intensity: f32,
    /// `Main.shimmerAlpha` (`Main.cs:917`): vanilla keeps both on Main, but
    /// `SceneState.UpdateShimmer` (`:283-309`) is their only writer, so the
    /// port owns them here. The alpha gates the horizon-cloud suppression and
    /// the draw-side shimmer effects; the darken multiplies the surface light
    /// seeds (`TileLightScanner.cs:3222`).
    pub shimmer_alpha: f32,
    pub shimmer_darken: f32,
    /// `SceneState._shimmerBrightenDelay` (`SceneState.cs:30`).
    shimmer_brighten_delay: f32,
    /// `SceneState._blizzardSoundVolume` (`SceneState.cs:33`): the
    /// blizzard loops' cross-fade (`SceneState.cs:210-228`), ramped by
    /// `MoveTowards` at 0.01 per update — M6-AUDIO-1's emitter, driven from
    /// the `tr-app` side (`ambient_sound::blizzard_loop_volumes`).
    pub blizzard_sound_volume: f32,
    /// `SceneState.skipTransitions` (`:35`).
    pub skip_transitions: bool,
    /// `Main.bloodMoon` — the only M1 reader is the phase-3 BloodMoon filter;
    /// set through [`SceneState::force`].
    pub blood_moon: bool,
    /// The `TR_SANDSTORM` pin, mirrored by `Sandstorm::force_severity` at the
    /// `tr-app` glue so the driver can also skip the storm update entirely.
    pub forced_severity: Option<f32>,
    /// The `TR_GRAVEYARD` pin: when set, `update` holds the intensity instead
    /// of fading it toward the tile-derived target.
    pub forced_graveyard: Option<f32>,
}

impl SceneState {
    /// `SceneState.Reset` (`SceneState.cs:44-54`).
    pub fn new() -> Self {
        let mut state = Self {
            air_light_decay: 1.0,
            solid_light_decay: 1.0,
            outside_weather_effect_intensity: 1.0,
            outside_weather_effect_intensity_backing: 1.0,
            graveyard_visual_intensity: 0.0,
            shimmer_alpha: 0.0,
            shimmer_darken: 0.0,
            shimmer_brighten_delay: 0.0,
            blizzard_sound_volume: 0.0,
            skip_transitions: true,
            blood_moon: false,
            forced_severity: None,
            forced_graveyard: None,
        };
        state.reset();
        state
    }

    /// `SceneState.Reset` (`SceneState.cs:44-54`); the graveyard intensity is
    /// zeroed with the world (`Main.cs:19860`). The TR_ pins clear too — the
    /// per-frame env glue (`scene_scan::apply_scene_forces_from_env`) re-applies
    /// them on the next tick.
    pub fn reset(&mut self) {
        self.air_light_decay = 1.0;
        self.solid_light_decay = 1.0;
        self.outside_weather_effect_intensity = 1.0;
        self.outside_weather_effect_intensity_backing = 1.0;
        self.graveyard_visual_intensity = 0.0;
        self.blizzard_sound_volume = 0.0;
        self.skip_transitions = true;
        self.blood_moon = false;
        self.forced_severity = None;
        self.forced_graveyard = None;
    }

    /// `SceneState.Update` (`SceneState.cs:56-62`), tr-core subset: the
    /// graveyard intensity fade, the light decays and the outside-weather
    /// intensity backing; then `skip_transitions = false` (`:62`). (The vanilla
    /// body also drives the filter params and the map capture — the filter
    /// driver is phase 3, the map capture already lives in `tr-core/map`.)
    pub fn update(&mut self, metrics: &SceneMetrics) {
        let skip = self.skip_transitions;
        // `SceneState.cs:76` → `UpdateShimmer` (`:283-309`).
        self.update_shimmer(metrics);
        // `SceneState.cs:72` → `UpdateGraveyard` (`:368-371`): fade toward the
        // count-derived 0..1 target (0.02/tick in, 0.1/tick out).
        let target = get_lerp_value(
            thresholds::GRAVEYARD_TILE_MIN as f32,
            thresholds::GRAVEYARD_TILE_MAX as f32,
            metrics.graveyard_tile_count as f32,
            true,
        );
        match self.forced_graveyard {
            // `TR_GRAVEYARD` pin: hold the forced value, skip the fade.
            Some(v) => self.graveyard_visual_intensity = v,
            None => move_towards_impl(
                skip,
                &mut self.graveyard_visual_intensity,
                target,
                0.02,
                0.1,
            ),
        }
        // `SceneState.cs:74` → `UpdateLightDecay` (`:255-281`): the targets are
        // `1 - shimmerAlpha * {0, 0.3}` with the `getGoodWorld` NPC terms —
        // shimmer and FTW seeds are not ported, so both targets are 1.
        move_towards_impl(skip, &mut self.air_light_decay, 1.0, 0.005, 0.005);
        move_towards_impl(skip, &mut self.solid_light_decay, 1.0, 0.005, 0.005);
        // `SceneState.cs:134-135`: the backing value chases -0.1 behind a
        // backwall, 1.1 in the open; the intensity clamps it to 0..1.
        move_towards_impl(
            skip,
            &mut self.outside_weather_effect_intensity_backing,
            if metrics.behind_backwall { -0.1 } else { 1.1 },
            0.005,
            0.005,
        );
        self.outside_weather_effect_intensity = self
            .outside_weather_effect_intensity_backing
            .clamp(0.0, 1.0);
        // `SceneState.cs:62`:
        self.skip_transitions = false;
    }

    /// `SceneState.MoveTowards(ref value, target, amount)` (`:472-477`).
    pub fn move_towards(&self, value: &mut f32, target: f32, amount: f32) {
        move_towards_impl(self.skip_transitions, value, target, amount, amount);
    }

    /// `SceneState.UpdateShimmer` (`SceneState.cs:283-309`): the two ramps.
    /// The monolith states (`:284,288`) and the perspective-player shimmering
    /// term land with their systems (M4 events / M2-BUFF shimmer); the active
    /// trigger is `metrics.ZoneShimmer` with its depth window.
    fn update_shimmer(&mut self, metrics: &SceneMetrics) {
        let flag = metrics.zone_shimmer
            || (metrics.player_shimmering && metrics.underground_for_shimmering);
        if flag {
            move_towards_impl(
                self.skip_transitions,
                &mut self.shimmer_alpha,
                1.0,
                0.025,
                0.025,
            );
            if self.shimmer_alpha >= 0.5 {
                move_towards_impl(
                    self.skip_transitions,
                    &mut self.shimmer_darken,
                    1.0,
                    0.025,
                    0.025,
                );
                self.shimmer_brighten_delay = 4.0;
            }
            return;
        }
        move_towards_impl(
            self.skip_transitions,
            &mut self.shimmer_darken,
            0.0,
            0.05,
            0.05,
        );
        if self.shimmer_darken == 0.0 {
            move_towards_impl(
                self.skip_transitions,
                &mut self.shimmer_brighten_delay,
                0.0,
                1.0,
                1.0,
            );
        }
        if self.shimmer_brighten_delay == 0.0 {
            move_towards_impl(
                self.skip_transitions,
                &mut self.shimmer_alpha,
                0.0,
                0.05,
                0.05,
            );
        }
    }

    /// `SceneState.MoveTowards(ref value, target, inc, dec)` (`:479-491`).
    pub fn move_towards_inc_dec(&self, value: &mut f32, target: f32, inc: f32, dec: f32) {
        move_towards_impl(self.skip_transitions, value, target, inc, dec);
    }

    /// The TR_ determinism forces: `SANDSTORM` records the severity pin
    /// (`Sandstorm::force_severity` applies it at the glue), `GRAVEYARD` pins
    /// the visual intensity, `BLOODMOON` sets `blood_moon`. Returns whether
    /// `name` was recognized.
    pub fn force(&mut self, name: &str, value: f32) -> bool {
        match name {
            "SANDSTORM" => self.forced_severity = Some(value.clamp(0.0, 1.0)),
            "GRAVEYARD" => self.forced_graveyard = Some(value.clamp(0.0, 1.0)),
            "BLOODMOON" => self.blood_moon = value != 0.0,
            _ => return false,
        }
        true
    }
}

/// The shared body of both `SceneState.MoveTowards` overloads
/// (`SceneState.cs:479-491`).
fn move_towards_impl(skip_transitions: bool, value: &mut f32, target: f32, inc: f32, dec: f32) {
    if skip_transitions {
        *value = target;
    } else if *value < target {
        *value = (*value + inc).min(target);
    } else if *value > target {
        *value = (*value - dec).max(target);
    }
}

/// Parses one `TR_<NAME>` / `TR_<NAME>=<value>` environment entry into the
/// `(name, value)` pair `SceneState::force` eats. A bare name means 1.0;
/// numeric values are clamped to 0..1; anything unparsable is `None`.
pub fn parse_env_force(raw: &str) -> Option<(&str, f32)> {
    let rest = raw.strip_prefix("TR_")?;
    let (name, value) = match rest.split_once('=') {
        Some((name, value)) => (name, value),
        None => (rest, ""),
    };
    if name.is_empty() {
        return None;
    }
    let v = if value.is_empty() {
        1.0
    } else {
        value.trim().parse::<f32>().ok()?
    };
    if v.is_nan() {
        return None;
    }
    Some((name, v.clamp(0.0, 1.0)))
}

/// `Utils.GetLerpValue` (`Utils.cs:315-343`): the clamped inverse lerp the
/// graveyard intensity target uses. Same body as the local copy in `wind.rs`
/// (`wind.rs:361`); shared here once a second caller appeared.
pub fn get_lerp_value(from: f32, to: f32, t: f32, clamped: bool) -> f32 {
    if clamped {
        if from < to {
            if t < from {
                return 0.0;
            }
            if t > to {
                return 1.0;
            }
        } else {
            if t < to {
                return 1.0;
            }
            if t > from {
                return 0.0;
            }
        }
    }
    (t - from) / (to - from)
}

/// `MathHelper.Lerp` (XNA): the linear interpolation the filter formulas use.
fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

/// `Utils.Remap` (`Utils.cs:345-348`): `Lerp(toMin, toMax,
/// GetLerpValue(fromMin, fromMax, value))`, always clamped on this path.
fn remap(value: f32, from_min: f32, from_max: f32, to_min: f32, to_max: f32) -> f32 {
    lerp(
        to_min,
        to_max,
        get_lerp_value(from_min, from_max, value, true),
    )
}

/// `Utils.SmoothStep` (`Utils.cs:126-129`): a clamped inverse lerp despite the
/// name (no cubic easing).
fn smooth_step(min: f32, max: f32, x: f32) -> f32 {
    ((x - min) / (max - min)).clamp(0.0, 1.0)
}

// ---------------------------------------------------------------------------
// Screen filters (M1.4-BG-18 phase 3): the M1 subset of the vanilla filter
// registry. Sources: `Graphics/Effects/{Filter,FilterManager,GameEffect,
// EffectPriority}.cs`, `Initializers/ScreenEffectInitializer.cs:14-48`,
// `GameContent/Shaders/{Sandstorm,Blizzard}ShaderData.cs`, `SceneState.cs`.
// The uniform assembly stays in tr-app (`FilterParams` is a render type);
// this module owns the state machines and the parameter formulas
// `ApplyVisuals` feeds them.

/// `EffectPriority` (`Graphics/Effects/EffectPriority.cs`): the ordering key
/// of the active-filter list; the highest-priority entry draws last.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum EffectPriority {
    VeryLow,
    Low,
    Medium,
    High,
    VeryHigh,
}

/// The M1 filter set — every `Filters.Scene`/`Overlays.Scene` entry the
/// driver drives. Towers/monoliths/MoonLord/Sepia/CRT/test passes are
/// M4/test content, Aurora is a `SkyManager` visual
/// (`ScreenEffectInitializer.cs:65`) — none has an M1 activation reader.
/// HeatDistortion/WaterDistortion (phase 4) are board slots too but never
/// compete for the tint winner: vanilla stacks every visible filter
/// (`FilterManager.EndCapture`, `:154-180`) and the distortions chain on top.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum FilterId {
    /// `Filters.Scene["Graveyard"]` (`ScreenEffectInitializer.cs:25`).
    Graveyard,
    /// `Filters.Scene["BloodMoon"]` — `BloodMoonScreenShaderData` (`:27`).
    BloodMoon,
    /// `Filters.Scene["Noir"]` (`:29`); monolith-gated, no M1 activation path.
    Noir,
    /// `Filters.Scene["Sandstorm"]` — the foreground Filter (`:30-31`).
    Sandstorm,
    /// `Overlays.Scene["Sandstorm"]` — the background overlay twin (`:32-33`).
    SandstormOverlay,
    /// `Filters.Scene["Blizzard"]` — the foreground Filter (`:34-36`).
    Blizzard,
    /// `Overlays.Scene["Blizzard"]` — the background overlay twin (`:37-39`).
    BlizzardOverlay,
    /// `Filters.Scene["HeatDistortion"]` (`:41`), `EffectPriority.Low`.
    HeatDistortion,
    /// `Filters.Scene["WaterDistortion"]` (`:42`), `EffectPriority.VeryHigh`.
    WaterDistortion,
    /// `Filters.Scene["MoonLordShake"]` (`ScreenEffectInitializer.cs:23`) —
    /// the MoonLordScreenShaderData shake, `EffectPriority.VeryHigh`. The
    /// activation rides the ritual countdown (`Main.cs:64976-64984`, S1-G-6).
    MoonLordShake,
}

impl FilterId {
    /// The board set, in `ScreenEffectInitializer.Load` order.
    pub const ALL: [FilterId; 10] = [
        FilterId::Graveyard,
        FilterId::BloodMoon,
        FilterId::Noir,
        FilterId::Sandstorm,
        FilterId::SandstormOverlay,
        FilterId::Blizzard,
        FilterId::BlizzardOverlay,
        FilterId::HeatDistortion,
        FilterId::WaterDistortion,
        FilterId::MoonLordShake,
    ];
    /// The UV-distortion filters (phase 4): they run as extra chained passes
    /// around the tint winner, in `priority` order (Low heat first, VeryHigh
    /// water then VeryHigh MoonLordShake last — the equal-priority
    /// activations sit after their earlier siblings,
    /// `FilterManager.OnActivate` `:70-76`) — the vanilla stack order.
    pub const DISTORTIONS: [FilterId; 3] = [
        FilterId::HeatDistortion,
        FilterId::WaterDistortion,
        FilterId::MoonLordShake,
    ];
    /// The post-pass competitors — the `Filters.Scene` entries only. The
    /// overlays render inside the scene (vanilla `RenderLayers.Landscape`),
    /// never in `FilterManager.EndCapture`.
    pub const FILTERS: [FilterId; 5] = [
        FilterId::Graveyard,
        FilterId::BloodMoon,
        FilterId::Noir,
        FilterId::Sandstorm,
        FilterId::Blizzard,
    ];

    /// The `Filters.Scene`/`Overlays.Scene` key (`post.rs::PASS_TABLE`).
    pub fn name(self) -> &'static str {
        match self {
            FilterId::Graveyard => "Graveyard",
            FilterId::BloodMoon => "BloodMoon",
            FilterId::Noir => "Noir",
            FilterId::Sandstorm => "Sandstorm",
            FilterId::SandstormOverlay => "SandstormOverlay",
            FilterId::Blizzard => "Blizzard",
            FilterId::BlizzardOverlay => "BlizzardOverlay",
            FilterId::HeatDistortion => "HeatDistortion",
            FilterId::WaterDistortion => "WaterDistortion",
            FilterId::MoonLordShake => "MoonLordShake",
        }
    }

    /// The `EffectPriority` of the initializer row
    /// (`ScreenEffectInitializer.cs:25-39`).
    pub fn priority(self) -> EffectPriority {
        match self {
            FilterId::Graveyard | FilterId::BloodMoon | FilterId::Noir => EffectPriority::Medium,
            FilterId::Sandstorm
            | FilterId::SandstormOverlay
            | FilterId::Blizzard
            | FilterId::BlizzardOverlay => EffectPriority::High,
            // `ScreenEffectInitializer.cs:41-42`.
            FilterId::HeatDistortion => EffectPriority::Low,
            FilterId::WaterDistortion => EffectPriority::VeryHigh,
            // `:23` — the MoonLordScreenShaderData shake row.
            FilterId::MoonLordShake => EffectPriority::VeryHigh,
        }
    }

    /// Whether this id is an `Overlays.Scene` twin (not a post-pass candidate).
    pub fn is_overlay(self) -> bool {
        matches!(self, FilterId::SandstormOverlay | FilterId::BlizzardOverlay)
    }

    fn index(self) -> usize {
        Self::ALL
            .iter()
            .position(|&x| x == self)
            .expect("FilterId in ALL")
    }
}

/// One `GameEffect`'s live state (`Graphics/Effects/GameEffect.cs`: the
/// `Opacity` field; `Filter.cs:8`: the `Active` flag).
#[derive(Clone, Copy, Debug)]
pub struct FilterSlot {
    active: bool,
    opacity: f32,
    /// `FilterManager.OnActivate`'s list-insertion sequence — the sorted
    /// `_activeFilters` list orders by (priority, insertion), and `EndCapture`
    /// lets the last visible entry win (`FilterManager.cs:154-180`), so a
    /// same-priority tie resolves to the later activation.
    activation_seq: u32,
    /// `Filter.IsHidden` (`Filter.cs:12`): `HeatDistortion` sets it while its
    /// intensity is 0 (`SceneState.cs:161`); a hidden filter is not visible.
    hidden: bool,
}

impl Default for FilterSlot {
    fn default() -> Self {
        Self {
            active: false,
            opacity: 0.0,
            activation_seq: 0,
            hidden: false,
        }
    }
}

/// The `Filters.Scene`/`Overlays.Scene` M1 subset — activation edges
/// (`SceneState.ManageSpecialBiomeVisuals`, `SceneState.cs:311-366`), the
/// opacity fades (`FilterManager.Update`, `FilterManager.cs:90-127`) and the
/// winner pick (`FilterManager.EndCapture`, `:154-180`, plan constraint: one
/// filter per frame — vanilla's stacking collapses to the last entry).
#[derive(Debug)]
pub struct FilterBoard {
    slots: [FilterSlot; 10],
    seq: u32,
}

impl FilterBoard {
    /// Fresh board: nothing active, nothing fading.
    pub fn new() -> Self {
        Self {
            slots: std::array::from_fn(|_| FilterSlot::default()),
            seq: 0,
        }
    }

    /// `SceneState.ManageSpecialBiomeVisuals` (`SceneState.cs:324-343`, the
    /// Filter/Overlay half): flip `Active` on an in-zone edge (a fresh
    /// activation records the insertion sequence, `FilterManager.OnActivate`
    /// `:44-81`), and the `alwaysInstant || skipTransitions` tail (`:355-365`)
    /// snaps the opacity instead of fading.
    pub fn manage(&mut self, id: FilterId, in_zone: bool, skip_transitions: bool) {
        let slot = &mut self.slots[id.index()];
        if in_zone != slot.active {
            if in_zone {
                self.seq += 1;
                slot.activation_seq = self.seq;
            }
            slot.active = in_zone;
        }
        if skip_transitions {
            slot.opacity = if in_zone { 1.0 } else { 0.0 };
        }
    }

    /// `FilterManager.Update` (`FilterManager.cs:90-127`): the opacity ramps
    /// toward `(Active ? 1 : 0)` at `OPACITY_RATE = 1` per second
    /// (`:111-116`); a drained inactive slot drops out of the list implicitly
    /// (its opacity is 0 and it stops being visible/in-list).
    pub fn update(&mut self, dt: f32) {
        for slot in &mut self.slots {
            if slot.active {
                slot.opacity = (slot.opacity + dt).min(1.0);
            } else {
                slot.opacity = (slot.opacity - dt).max(0.0);
            }
        }
    }

    /// `Filter.IsActive` (`Filter.cs:58-61`).
    pub fn is_active(&self, id: FilterId) -> bool {
        self.slots[id.index()].active
    }

    /// `Filter.IsHidden = hidden` (`Filter.cs:12`; `SceneState.cs:161`).
    pub fn set_hidden(&mut self, id: FilterId, hidden: bool) {
        self.slots[id.index()].hidden = hidden;
    }

    /// `Filter.IsVisible` (`Filter.cs:63-70`): `CombinedOpacity > 0 && !IsHidden`
    /// (the shader's own `UseOpacity` is folded in by the driver; the board
    /// sees the fade opacity).
    pub fn is_visible(&self, id: FilterId) -> bool {
        let slot = &self.slots[id.index()];
        slot.opacity > 0.0 && !slot.hidden
    }

    /// `GameEffect.Opacity` — the 1/s fade value.
    pub fn opacity(&self, id: FilterId) -> f32 {
        self.slots[id.index()].opacity
    }

    /// Whether the filter still sits in `_activeFilters`: active, or fading
    /// out (`FilterManager.Update` removes a filter only once it is inactive
    /// with opacity 0, `:117-124`). The storm shader `Update`s run for exactly
    /// this set.
    pub fn in_list(&self, id: FilterId) -> bool {
        let slot = &self.slots[id.index()];
        slot.active || slot.opacity > 0.0
    }

    /// [`FilterBoard::winner_by`] with every shader opacity at 1 (the board's
    /// fade alone decides visibility).
    pub fn winner(&self) -> Option<FilterId> {
        self.winner_by(|id| self.opacity(id))
    }

    /// `FilterManager.EndCapture`'s pick (`FilterManager.cs:154-180`, plan
    /// constraint "one active filter"): the last *visible* entry of the
    /// priority-ordered list — the maximum of (priority, activation_seq) among
    /// the filters whose `Filter.IsVisible` (`Filter.cs:63-70`:
    /// `CombinedOpacity > 0 && !IsHidden`) holds. `combined_opacity` supplies
    /// each filter's `ScreenShaderData.CombinedOpacity` (`:84`, the board fade
    /// times the shader's own `UseOpacity` value, computed by the driver for
    /// this frame). Overlays never compete (they render in the scene, not in
    /// `EndCapture`).
    pub fn winner_by(&self, combined_opacity: impl Fn(FilterId) -> f32) -> Option<FilterId> {
        let mut best: Option<FilterId> = None;
        for id in FilterId::FILTERS {
            if self.slots[id.index()].hidden || combined_opacity(id) <= 0.0 {
                continue;
            }
            let better = match best {
                None => true,
                Some(b) => {
                    (id.priority(), self.slots[id.index()].activation_seq)
                        > (b.priority(), self.slots[b.index()].activation_seq)
                }
            };
            if better {
                best = Some(id);
            }
        }
        best
    }
}

/// `SandstormShaderData` / `BlizzardShaderData` live state: the scrolling
/// `_texturePosition` and (blizzard only) the smoothed `windSpeed`
/// (`GameContent/Shaders/SandstormShaderData.cs:8`,
/// `BlizzardShaderData.cs:8-10`).
#[derive(Clone, Copy, Debug)]
pub struct StormShader {
    texture_pos: [f32; 2],
    direction: [f32; 2],
    wind_speed: f32,
}

impl StormShader {
    /// Field initializers: `_texturePosition = Vector2.Zero`,
    /// `windSpeed = 0.1f` (`BlizzardShaderData.cs:8-10`).
    pub const fn new() -> Self {
        Self {
            texture_pos: [0.0; 2],
            direction: [0.0, 1.0],
            wind_speed: 0.1,
        }
    }

    /// The last computed `UseDirection` vector (`SandstormShaderData.cs:21`).
    pub fn direction(&self) -> [f32; 2] {
        self.direction
    }

    /// The last computed `_texturePosition` — the storm shaders' `Apply`
    /// forces it into `uTargetPosition` (`SandstormShaderData.cs:25-29`,
    /// `BlizzardShaderData.cs:38-43`).
    pub fn texture_pos(&self) -> [f32; 2] {
        self.texture_pos
    }

    /// `SandstormShaderData.Update` (`:10-23`): direction = `(-wind, -1)`
    /// componentwise × `(20, 0.1)`, normalized, × `(2, 0.2)`; the texture
    /// position advances by direction × elapsed, wrapped mod 10
    /// (`FocusHelper.UpdateVisualEffects` is true in a focused window).
    pub fn update_sandstorm(&mut self, wind_speed_current: f32, dt: f32) {
        let mut vector = [-wind_speed_current * 20.0, -0.1];
        let len = (vector[0] * vector[0] + vector[1] * vector[1]).sqrt();
        if len > 0.0 {
            vector[0] /= len;
            vector[1] /= len;
        }
        vector[0] *= 2.0;
        vector[1] *= 0.2;
        self.direction = vector;
        self.texture_pos[0] = (self.texture_pos[0] + vector[0] * dt) % 10.0;
        self.texture_pos[1] = (self.texture_pos[1] + vector[1] * dt) % 10.0;
    }

    /// `BlizzardShaderData.Update` (`:12-36`): the wind clamps to ±0.1 at
    /// rest, smooths at 0.05/0.95, and the direction is `(-windSpeed, -1)` ×
    /// `(10, 2)` normalized × `(0.8, 0.6)`; the scroll wraps mod 10.
    pub fn update_blizzard(&mut self, wind_speed_current: f32, dt: f32) {
        let mut num = wind_speed_current;
        if (0.0..=0.1).contains(&num) {
            num = 0.1;
        } else if (-0.1..=0.0).contains(&num) {
            num = -0.1;
        }
        self.wind_speed = num * 0.05 + self.wind_speed * 0.95;
        let mut vector = [-self.wind_speed * 10.0, -2.0];
        let len = (vector[0] * vector[0] + vector[1] * vector[1]).sqrt();
        if len > 0.0 {
            vector[0] /= len;
            vector[1] /= len;
        }
        vector[0] *= 0.8;
        vector[1] *= 0.6;
        self.direction = vector;
        self.texture_pos[0] = (self.texture_pos[0] + vector[0] * dt) % 10.0;
        self.texture_pos[1] = (self.texture_pos[1] + vector[1] * dt) % 10.0;
    }
}

// --- ApplyVisuals parameter formulas (`SceneState.cs:107-188`, M1 subset) ---

/// `SceneState.cs:109`: `progress = Lerp(0, 0.75, GraveyardVisualIntensity)`.
/// (`:113` pins `UseIntensity(1.2)` — a constant, not a formula.)
pub fn graveyard_progress(graveyard_visual_intensity: f32) -> f32 {
    lerp(0.0, 0.75, graveyard_visual_intensity)
}

/// `BloodMoonScreenShaderData.Update` (`:13-25`): the depth fade
/// `1 - Utils.SmoothStep(worldSurface + 50, rockLayer + 100, viewCenterTileY)`,
/// scaled by 0.75. The remix branch (`:16-19`) and the shimmer term (`:20-23`)
/// have no M1 reader.
pub fn blood_moon_opacity(view_center_tile_y: f32, world_surface: f64, rock_layer: f64) -> f32 {
    let depth = 1.0
        - smooth_step(
            world_surface as f32 + 50.0,
            rock_layer as f32 + 100.0,
            view_center_tile_y,
        );
    depth * 0.75
}

/// `SceneState.cs:117` — the Noir intensity: a nested Lerp/Remap over
/// `Main.tileColor` brightness (`Vector3.Dot(tileColor, 1/3)`), the depth
/// below the surface (`(int)worldSurface - TileCenter.Y`, remapped over
/// ±40 tiles) and the depth into the underworld (`TileCenter.Y -
/// UnderworldLayer`; `Main.UnderworldLayer = maxTilesY - 200`, `Main.cs:2790`).
/// The tileColor leg's range is inverted (0.5 → 0.1): darker tiles read as
/// more noir. No M1 activation path (the monolith is M4 content) — the
/// formula is ported for DW-3.2 and wired to the FilterNoir pass.
pub fn noir_intensity(
    tile_color: [f32; 3],
    world_surface: i32,
    tile_center_y: i32,
    underworld_layer: i32,
) -> f32 {
    let brightness = (tile_color[0] + tile_color[1] + tile_color[2]) / 3.0;
    let surface_leg = remap(
        (world_surface - tile_center_y) as f32,
        -40.0,
        40.0,
        0.0,
        1.0,
    );
    let underworld_leg = remap(
        (tile_center_y - underworld_layer) as f32,
        -40.0,
        40.0,
        0.0,
        1.0,
    );
    let base = remap(brightness, 0.5, 0.1, 0.0, 0.2);
    let deep = lerp(0.1, base, surface_leg);
    lerp(deep, 0.15, underworld_leg)
}

/// `SceneState.cs:118-124`: the Noir dither offset re-rolls from
/// `new Random((int)(Main.GlobalTimeWrappedHourly * 10f))` per frame. The
/// .NET `System.Random` sequence is not ported — the house `FastRandom`
/// stands in (unobservable in M1: Noir never activates; deviation recorded
/// for ANALYSIS).
pub fn noir_image_offset(global_time: f32) -> [f32; 2] {
    let mut rng = FastRandom::new((global_time * 10.0) as i32 as u32 as u64);
    [rng.next_float(), rng.next_float()]
}

/// `SceneState.cs:136-141` — the Sandstorm Filter (foreground) parameters:
/// intensity `outsideWeatherEffectIntensity * 0.4 * Min(1, Severity)`, opacity
/// `Min(1, Severity * 1.5) * outsideWeatherEffectIntensity`. Returns
/// `(intensity, opacity)`.
pub fn sandstorm_foreground_params(severity: f32, outside_weather: f32) -> (f32, f32) {
    (
        outside_weather * 0.4 * severity.min(1.0),
        (severity * 1.5).min(1.0) * outside_weather,
    )
}

/// `SceneState.cs:140` — the Sandstorm overlay (background) opacity, the
/// complement of the foreground: `Min(1, Severity * 1.5) *
/// (1 - outsideWeatherEffectIntensity)`. (Its intensity stays at the
/// initializer's 0.4 — `ApplyVisuals` never re-sets it.)
pub fn sandstorm_background_opacity(severity: f32, outside_weather: f32) -> f32 {
    (severity * 1.5).min(1.0) * (1.0 - outside_weather)
}

/// `SceneState.cs:163-188` — the Blizzard parameters. `cloud_alpha` is
/// `Main.cloudAlpha`, `deerclops_smoothed` is
/// `_deerclopsBlizzardSmoothedEffect` (the deerclops-proximity fade — no M1
/// spawner, so the driver passes 0 and both Lerps are no-ops). Returns
/// `(intensity num10, foreground opacity num9, background opacity 1 - num9)`.
pub fn blizzard_foreground_params(
    cloud_alpha: f32,
    outside_weather: f32,
    deerclops_smoothed: f32,
) -> (f32, f32, f32) {
    let mut num9 = (cloud_alpha * 2.0).min(1.0) * outside_weather;
    let mut num10 = outside_weather * 0.4 * (cloud_alpha * 2.0).min(1.0) * 0.9 + 0.1;
    num10 = lerp(num10, num10 * 0.5, deerclops_smoothed);
    num9 = lerp(num9, num9 * 0.5, deerclops_smoothed);
    (num10, num9, 1.0 - num9)
}

// --- Distortion filters (M1.4-BG-18 phase 4) ---

/// `WaterShaderData.cs:126-127`: the per-frame progress clock
/// (`_progress += dt * Intensity * 0.75; _progress %= 86400`). `Intensity` is
/// the shader's last `UseIntensity` (the initializer's 1 until `ApplyVisuals`
/// sets it, `ScreenEffectInitializer.cs:42`). `Update` runs only while the
/// filter sits in the active list (`FilterManager.Update`, `:90-127`).
#[derive(Clone, Copy, Debug)]
pub struct WaterShader {
    /// `WaterShaderData._progress` (`WaterShaderData.cs:72`) — `uProgress`.
    pub progress: f32,
    /// `ScreenShaderData.Intensity` (`UseIntensity`).
    pub intensity: f32,
}

impl WaterShader {
    /// `_progress` starts at 0, `UseIntensity(1f)` (`ScreenEffectInitializer.cs:42`).
    pub const fn new() -> Self {
        Self {
            progress: 0.0,
            intensity: 1.0,
        }
    }

    /// `WaterShaderData.Update` (`WaterShaderData.cs:117-134`, progress part;
    /// the wave-quality toggles and the ripple `_queuedSteps` belong to the
    /// unported entity-wave buffer).
    pub fn update(&mut self, dt: f32) {
        self.progress += dt * self.intensity * 0.75;
        self.progress %= 86400.0;
    }
}

/// `SceneState.cs:105`: `ManageSpecialBiomeVisuals("WaterDistortion",
/// Main.WaveQuality > 0)`. `Main.WaveQuality` defaults to 3
/// (`Main.cs:66571`) and clamps to 0..=3 on load (`:4439-4446`).
pub fn water_distortion_active(wave_quality: i32) -> bool {
    wave_quality > 0
}

/// `SceneState.cs:128-132` — the WaterDistortion `UseIntensity` value:
/// wind + rain + world-edge terms, faded to 0.75 flat with depth below the
/// surface (`num5`) and ramped up again near the underworld, clamped to
/// 0..=2.5.
pub fn water_distortion_intensity(
    tile_center: (i32, i32),
    max_tiles_x: i32,
    max_tiles_y: i32,
    wind_speed_current: f32,
    max_raining: f32,
    world_surface: f64,
    rock_layer: f64,
) -> f32 {
    let half_x = max_tiles_x as f32 * 0.5;
    // `:128` — distance from the nearest world edge.
    let edge_dist = half_x - (tile_center.0 as f32 - half_x).abs();
    // `:129`
    let num4 = 1.0
        + wind_speed_current.abs() * 1.25
        + max_raining.clamp(0.0, 1.0) * 1.25
        + -((((edge_dist - 380.0) / 100.0).clamp(0.0, 1.0)) * 0.5 - 0.25);
    // `:130` — `(float)((double)Y - worldSurface) / (float)(rockLayer - worldSurface)`.
    let depth =
        ((tile_center.1 as f64 - world_surface) as f32) / ((rock_layer - world_surface) as f32);
    let num5 = 1.0 - (3.0 * depth).clamp(0.0, 1.0);
    // `:131`
    let underworld = ((max_tiles_y - tile_center.1 - 200) as f32 / 300.0).clamp(0.0, 1.0);
    (num4 * num5 + (0.9 - underworld * 0.9) + (1.0 - num5) * 0.75).clamp(0.0, 2.5)
}

/// `SceneState.cs:79-80`: the two zone flags that gate HeatDistortion —
/// `flag2` (the underworld band, `Y > maxTilesY - 320`) and `flag3` (surface
/// desert without rain or sandstorm). Returns `(flag2, flag3)`.
pub fn heat_zone_flags(
    tile_center_y: i32,
    max_tiles_y: i32,
    world_surface: f64,
    zone_desert: bool,
    zone_rain: bool,
    zone_sandstorm: bool,
) -> (bool, bool) {
    (
        tile_center_y > max_tiles_y - 320,
        (tile_center_y as f64) < world_surface && zone_desert && !zone_rain && !zone_sandstorm,
    )
}

/// `SceneState.cs:98-99`: `UseHeatDistortion & ((flag2 | flag3) ||
/// sunScorchCounter > 0)`. `Main.UseHeatDistortion` defaults true
/// (`Main.cs:66573`).
pub fn heat_distortion_active(
    use_heat_distortion: bool,
    flag2: bool,
    flag3: bool,
    sun_scorch_counter: i32,
) -> bool {
    use_heat_distortion && (flag2 || flag3 || sun_scorch_counter > 0)
}

/// `SceneState.cs:145-160` — the HeatDistortion `UseIntensity` value (the
/// caller also sets `IsHidden = intensity <= 0`, `:161`): the sun-scorch
/// ramp, the underworld ramp, or the desert surface term scaled by the
/// outside-weather fade, clear sky and tile brightness.
#[allow(clippy::too_many_arguments)]
pub fn heat_distortion_intensity(
    sun_scorch_counter: i32,
    underworld: bool,
    desert: bool,
    tile_center_y: i32,
    max_tiles_y: i32,
    outside_weather: f32,
    cloud_alpha: f32,
    tile_color: [f32; 3],
) -> f32 {
    let mut num6 = 0.0f32;
    if sun_scorch_counter > 0 {
        num6 = num6.max(get_lerp_value(0.0, 300.0, sun_scorch_counter as f32, true) * 4.0);
    }
    if underworld {
        num6 = num6.max(((tile_center_y - (max_tiles_y - 320)) as f32 / 120.0).min(1.0) * 2.0);
    } else if desert {
        let brightness = (tile_color[0] + tile_color[1] + tile_color[2]) / 3.0;
        num6 = num6.max(outside_weather * 4.0 * (0.5 - cloud_alpha).max(0.0) * brightness);
    }
    num6
}

/// `SurfaceBackgroundID.Sets.IsDesertVariant` (`SurfaceBackgroundID.cs:18`):
/// the bg styles `Sandstorm.ShowSandstormVisuals` accepts
/// (`Sandstorm.cs:131-138`).
pub fn is_desert_bg_style(bg_style: i32) -> bool {
    matches!(bg_style, 2 | 5 | 13 | 14)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A metrics with only the given window counts set.
    fn metrics_with_counts(counts: &[(u16, u32)]) -> SceneMetrics {
        let mut m = SceneMetrics::new();
        for &(id, n) in counts {
            m.tile_counts[id as usize] = n;
        }
        m
    }

    // --- AggregateTileCounts (SceneMetrics.cs:573-660) ---

    #[test]
    fn holy_and_snow_sums_match_source_id_lists() {
        // `SceneMetrics.cs:589-590`: 164 belongs to BOTH sums.
        // Snow ids 163/200 also belong to the evil/blood sums
        // (`SceneMetrics.cs:613-614`), so this id-list test uses the snow-only
        // ids 147/148/161/162 and a holy sum that survives the
        // cross-subtraction (no evil/blood tiles in the window).
        let mut m = metrics_with_counts(&[
            (109, 10),
            (492, 1),
            (110, 2),
            (113, 3),
            (117, 4),
            (116, 5),
            (164, 6),
            (403, 7),
            (402, 8),
            (147, 100),
            (148, 150),
            (161, 200),
            (162, 250),
        ]);
        m.aggregate_tile_counts();
        assert_eq!(m.holy_tile_count, 10 + 1 + 2 + 3 + 4 + 5 + 6 + 7 + 8);
        assert_eq!(m.snow_tile_count, 6 + 100 + 150 + 200 + 250);
    }

    #[test]
    fn sand_sums_match_source_id_list() {
        // `SceneMetrics.cs:613`.
        let mut m = metrics_with_counts(&[
            (53, 1),
            (112, 2),
            (116, 4),
            (234, 8),
            (397, 16),
            (398, 32),
            (402, 64),
            (399, 128),
            (396, 256),
            (400, 512),
            (403, 1024),
            (401, 2048),
        ]);
        m.aggregate_tile_counts();
        let total: i32 = 1 + 2 + 4 + 8 + 16 + 32 + 64 + 128 + 256 + 512 + 1024 + 2048;
        assert_eq!(m.sand_tile_count, total);
        // No ocean sand seen in the window: desert sand = sand.
        assert_eq!(m.desert_sand_tile_count, total);
    }

    #[test]
    fn graveyard_is_tombstones_minus_half_sunflowers_floored_at_zero() {
        // `SceneMetrics.cs:615-624`: `counts[85] - counts[27]/2` (integer
        // division), floored at zero.
        let mut m = metrics_with_counts(&[(85, 30), (27, 10)]);
        m.aggregate_tile_counts();
        assert_eq!(m.graveyard_tile_count, 25);

        let mut m = metrics_with_counts(&[(85, 4), (27, 9)]);
        m.aggregate_tile_counts();
        assert_eq!(m.graveyard_tile_count, 0); // 4 - 4 = 0

        let mut m = metrics_with_counts(&[(85, 0), (27, 20)]);
        m.aggregate_tile_counts();
        assert_eq!(m.graveyard_tile_count, 0); // -10 -> 0
    }

    #[test]
    fn desert_sand_subtracts_ocean_sand_floored_at_zero() {
        // `SceneMetrics.cs:658`. The sand sum is re-derived from the window
        // counts; the ocean count is filled by the scan (`:377-379`).
        let mut m = metrics_with_counts(&[(53, 100)]);
        m.ocean_sand_tile_count = 40;
        m.aggregate_tile_counts();
        assert_eq!(m.sand_tile_count, 100);
        assert_eq!(m.desert_sand_tile_count, 60);

        let mut m = metrics_with_counts(&[(53, 10)]);
        m.ocean_sand_tile_count = 40;
        m.aggregate_tile_counts();
        assert_eq!(m.desert_sand_tile_count, 0);
    }

    // --- CalculateZones (SceneMetrics.cs:662-707) ---

    #[test]
    fn zones_at_exact_thresholds() {
        // `SceneMetrics.cs:258,262`: `>=` comparisons.
        let mut m = SceneMetrics::new();
        m.snow_tile_count = thresholds::SNOW_TILE_NORMAL;
        m.desert_sand_tile_count = thresholds::DESERT_TILE_NORMAL;
        m.calculate_zones(100.0, 1000, 135.0, 4200, 400, false, false);
        assert!(m.zone_snow);
        assert!(m.zone_desert);

        let mut m = SceneMetrics::new();
        m.snow_tile_count = thresholds::SNOW_TILE_NORMAL - 1;
        m.desert_sand_tile_count = thresholds::DESERT_TILE_NORMAL - 1;
        m.calculate_zones(100.0, 1000, 135.0, 4200, 400, false, false);
        assert!(!m.zone_snow);
        assert!(!m.zone_desert);
    }

    #[test]
    fn zone_rain_needs_raining_and_surface() {
        let mut m = SceneMetrics::new();
        m.tile_center = (100, 100);
        m.calculate_zones(150.0, 1000, 135.0, 4200, 400, true, false); // y=100 <= surface=150
        assert!(m.zone_rain && m.surface_atmospherics);

        let mut m = SceneMetrics::new();
        m.tile_center = (100, 200);
        m.calculate_zones(150.0, 1000, 135.0, 4200, 400, true, false); // below the surface
        assert!(!m.zone_rain && !m.surface_atmospherics);

        let mut m = SceneMetrics::new();
        m.tile_center = (100, 100);
        m.calculate_zones(150.0, 1000, 135.0, 4200, 400, false, false);
        assert!(!m.zone_rain);
    }

    #[test]
    fn zone_rain_and_zone_snow_coexist() {
        // The blizzard gate is `ZoneRain && ZoneSnow` (`SceneState.cs:120`) —
        // both flags must survive together.
        let mut m = SceneMetrics::new();
        m.snow_tile_count = thresholds::SNOW_TILE_NORMAL;
        m.tile_center = (0, 10);
        m.calculate_zones(100.0, 1000, 135.0, 4200, 400, true, false);
        assert!(m.zone_rain);
        assert!(m.zone_snow);
    }

    #[test]
    fn zone_sandstorm_suppresses_zone_rain() {
        // `SceneMetrics.cs:703-707`.
        let mut m = SceneMetrics::new();
        m.desert_sand_tile_count = thresholds::DESERT_TILE_NORMAL;
        m.tile_center = (0, 10);
        m.calculate_zones(100.0, 1000, 135.0, 4200, 400, true, true);
        assert!(m.zone_sandstorm);
        assert!(!m.zone_rain);

        // Without the happening storm the rain zone stays.
        let mut m = SceneMetrics::new();
        m.desert_sand_tile_count = thresholds::DESERT_TILE_NORMAL;
        m.tile_center = (0, 10);
        m.calculate_zones(100.0, 1000, 135.0, 4200, 400, true, false);
        assert!(!m.zone_sandstorm);
        assert!(m.zone_rain);

        // A sandstorm underground does not count (needs the surface).
        let mut m = SceneMetrics::new();
        m.desert_sand_tile_count = thresholds::DESERT_TILE_NORMAL;
        m.tile_center = (0, 500);
        m.calculate_zones(100.0, 1000, 135.0, 4200, 400, false, true);
        assert!(!m.zone_sandstorm);
    }

    // --- oceanDepths / isDesertBiomeSand (WorldGen.cs:4447, TileID.cs:501) ---

    #[test]
    fn ocean_depths_beach_bands() {
        // worldSurface=100, rockLayer=180 -> oceanLevel = 180.
        // Left band: x < 380; right band: x > W - 380.
        assert!(ocean_depths(100.0, 180.0, 379, 1000, 100));
        assert!(ocean_depths(100.0, 180.0, 621, 1000, 100));
        assert!(!ocean_depths(100.0, 180.0, 380, 1000, 100));
        assert!(!ocean_depths(100.0, 180.0, 620, 1000, 100));
        // Below oceanLevel the bands do not count.
        assert!(!ocean_depths(100.0, 180.0, 100, 1000, 181));
        // Exactly at oceanLevel still counts (`>` comparison).
        assert!(ocean_depths(100.0, 180.0, 100, 1000, 180));
    }

    #[test]
    fn desert_biome_sand_set_matches_tile_ids() {
        // `TileID.cs:501`: CreateBoolSet(53, 397, 396, 400, 403, 401).
        for id in [53u16, 397, 396, 400, 403, 401] {
            assert!(is_desert_biome_sand(id));
        }
        for id in [112u16, 234, 398, 399, 402, 147] {
            assert!(!is_desert_biome_sand(id));
        }
    }

    // --- Sandstorm (Events/Sandstorm.cs) - DW-1.2 ---

    #[test]
    fn test_DW_1_2_severity_approaches_at_0_003() {
        // `Sandstorm.cs:109-110`: +0.003 per tick toward the intention.
        let mut s = Sandstorm::new(1);
        s.severity = 0.0;
        s.intended_severity = 1.0;
        s.update_severity();
        assert!((s.severity - 0.003).abs() < 1e-6);
        for _ in 0..99 {
            s.update_severity();
        }
        assert!((s.severity - 0.3).abs() < 1e-4);
    }

    #[test]
    fn test_DW_1_2_severity_snaps_on_overshoot() {
        // `Sandstorm.cs:111-114`: crossing the target snaps to it.
        let mut s = Sandstorm::new(1);
        s.severity = 0.0;
        s.intended_severity = 0.002;
        s.update_severity();
        assert_eq!(s.severity, 0.002);

        // Downward as well.
        let mut s = Sandstorm::new(1);
        s.severity = 0.001;
        s.intended_severity = 0.0;
        s.update_severity();
        assert_eq!(s.severity, 0.0);
    }

    #[test]
    fn test_DW_1_2_severity_clamped_0_1() {
        // `Sandstorm.cs:110`: MathHelper.Clamp(..., 0, 1).
        let mut s = Sandstorm::new(1);
        s.severity = 0.999;
        s.intended_severity = 2.0;
        s.update_severity();
        assert_eq!(s.severity, 1.0);

        let mut s = Sandstorm::new(1);
        s.severity = 0.001;
        s.intended_severity = -1.0;
        s.update_severity();
        assert_eq!(s.severity, 0.0);
    }

    #[test]
    fn test_DW_1_2_nan_severity_resets() {
        // `Sandstorm.cs:101-108`.
        let mut s = Sandstorm::new(1);
        s.severity = f32::NAN;
        s.intended_severity = 0.5;
        s.update_severity();
        assert!((s.severity - 0.003).abs() < 1e-6);

        let mut s = Sandstorm::new(1);
        s.severity = 0.5;
        s.intended_severity = f32::NAN;
        s.update_severity();
        // Intended reset to 0: severity walks down from 0.5 by 0.003.
        assert!((s.severity - (0.5 - 0.003)).abs() < 1e-5);
    }

    #[test]
    fn test_DW_1_2_time_left_decay_and_stop() {
        // `Sandstorm.cs:42-46`: `-dayRate`, plus `-15*dayRate` when the wind
        // is too weak (0 < |wind| < 0.6).
        let mut s = Sandstorm::new(1);
        s.happening = true;
        s.time_left = 100;
        s.update_time(1, 0.5);
        assert_eq!(s.time_left, 84); // 100 - 1 - 15

        let mut s = Sandstorm::new(1);
        s.happening = true;
        s.time_left = 100;
        s.update_time(1, 0.7);
        assert_eq!(s.time_left, 99);

        // `Sandstorm.cs:47-54`: wind exactly 0 zeroes TimeLeft (`:47-50`) and
        // the `<= 0` check stops the storm.
        let mut s = Sandstorm::new(1);
        s.happening = true;
        s.time_left = 100;
        s.update_time(1, 0.0);
        assert!(!s.happening);
        assert_eq!(s.time_left, 0);
    }

    #[test]
    fn test_DW_1_2_time_left_over_maximum_resets_first() {
        // `Sandstorm.cs:38-41`: a TimeLeft above the maximum is zeroed at
        // entry, so the next tick stops the storm.
        let mut s = Sandstorm::new(1);
        s.happening = true;
        s.time_left = 86401;
        s.update_time(1, 0.7);
        assert!(!s.happening);
        assert_eq!(s.time_left, 0);
    }

    #[test]
    fn test_DW_1_2_start_sets_duration_range_and_intent() {
        // `Sandstorm.cs:117-122`: duration 28800..=86400, intent 0.4..1.4.
        for seed in 0..8u64 {
            let mut s = Sandstorm::new(seed);
            s.start_sandstorm();
            assert!(s.happening);
            assert!((SANDSTORM_DURATION_MIN..=SANDSTORM_DURATION_MAX).contains(&s.time_left));
            assert!((0.4..1.4).contains(&s.intended_severity));
        }
    }

    #[test]
    fn test_DW_1_2_has_sufficient_wind_0_6() {
        // `Sandstorm.cs:22-25`: `|wind| >= 0.6`.
        assert!(Sandstorm::has_sufficient_wind(0.6));
        assert!(Sandstorm::has_sufficient_wind(-0.6));
        assert!(Sandstorm::has_sufficient_wind(0.8));
        assert!(!Sandstorm::has_sufficient_wind(0.599));
        assert!(!Sandstorm::has_sufficient_wind(0.0));
    }

    #[test]
    fn storm_starts_from_the_roll_branch_deterministically() {
        // `Sandstorm.cs:56-69`: with wind, one `Next(64800)` roll per
        // day-rate tick; a huge day rate starts the storm.
        let mut s = Sandstorm::new(42);
        s.update_time(70000, 0.8);
        assert!(s.happening);
        assert!((SANDSTORM_DURATION_MIN..=SANDSTORM_DURATION_MAX).contains(&s.time_left));

        // Same seed -> same outcome.
        let mut s2 = Sandstorm::new(42);
        s2.update_time(70000, 0.8);
        assert_eq!(s2.happening, s.happening);
        assert_eq!(s2.time_left, s.time_left);
        assert_eq!(s2.severity.to_bits(), s.severity.to_bits());
    }

    #[test]
    fn show_sandstorm_visuals_gate() {
        // `Sandstorm.cs:131-138`.
        let mut s = Sandstorm::new(1);
        s.happening = true;
        assert!(s.show_sandstorm_visuals(true, true, 49));
        assert!(!s.show_sandstorm_visuals(true, true, 50)); // bgDelay < 50
        assert!(!s.show_sandstorm_visuals(true, false, 0)); // not a desert bg
        assert!(!s.show_sandstorm_visuals(false, true, 0)); // not happening
    }

    // --- SceneState (SceneState.cs) - DW-1.3 ---

    #[test]
    fn test_DW_1_3_move_towards_inc_dec() {
        // `SceneState.cs:479-491`.
        let state = SceneState::new();
        let mut v = 0.0;
        // `new()` resets with skipTransitions = true: the first call snaps.
        state.move_towards(&mut v, 1.0, 0.005);
        assert_eq!(v, 1.0);

        // After the first update the transitions are live.
        let mut state = SceneState::new();
        let metrics = SceneMetrics::new();
        state.update(&metrics);
        let mut v = 0.0;
        state.move_towards(&mut v, 1.0, 0.005);
        assert!((v - 0.005).abs() < 1e-7);
        state.move_towards(&mut v, 1.0, 0.005);
        assert!((v - 0.01).abs() < 1e-7);
        // Approaching from below never overshoots the target.
        let mut v = 0.999;
        state.move_towards(&mut v, 1.0, 0.005);
        assert_eq!(v, 1.0);
        // Separate inc/dec overload.
        let mut v = 2.0;
        state.move_towards_inc_dec(&mut v, 0.0, 0.02, 0.1);
        assert!((v - 1.9).abs() < 1e-6);
        // Decrement never overshoots the target.
        let mut v = 0.999;
        state.move_towards_inc_dec(&mut v, 1.0, 0.02, 0.1);
        assert_eq!(v, 1.0);
    }

    #[test]
    fn test_DW_1_3_skip_transitions_snaps_then_lives() {
        // `SceneState.cs:53` sets skipTransitions on Reset, `:62` clears it
        // after the first Update.
        let mut state = SceneState::new();
        let mut metrics = SceneMetrics::new();
        metrics.behind_backwall = true;
        state.update(&metrics);
        // The first update ran with skip=true: the backing snapped to -0.1.
        assert_eq!(state.outside_weather_effect_intensity_backing, -0.1);
        assert_eq!(state.outside_weather_effect_intensity, 0.0);
        assert!(!state.skip_transitions);
        // The second update fades at 0.005/tick from where it snapped.
        state.outside_weather_effect_intensity_backing = 0.0;
        state.update(&metrics);
        assert!((state.outside_weather_effect_intensity_backing - (-0.005)).abs() < 1e-6);
        assert!((state.outside_weather_effect_intensity - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_DW_1_3_outside_weather_backwall_fade() {
        // `SceneState.cs:134-135`.
        let mut state = SceneState::new();
        let mut metrics = SceneMetrics::new();
        state.update(&metrics); // clear skipTransitions
        metrics.behind_backwall = true;
        for _ in 0..240 {
            state.update(&metrics);
        }
        // Backing settles at -0.1; the intensity clamps to 0.
        assert!((state.outside_weather_effect_intensity_backing - (-0.1)).abs() < 1e-6);
        assert_eq!(state.outside_weather_effect_intensity, 0.0);
        metrics.behind_backwall = false;
        for _ in 0..300 {
            state.update(&metrics);
        }
        // Backing settles at 1.1; the intensity clamps to 1.
        assert!((state.outside_weather_effect_intensity_backing - 1.1).abs() < 1e-5);
        assert!((state.outside_weather_effect_intensity - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_DW_1_3_graveyard_intensity_lerp_rates() {
        // `SceneState.cs:368-371`: target = GetLerpValue(16, 36, count),
        // in at 0.02/tick, out at 0.1/tick.
        let mut state = SceneState::new();
        let mut metrics = SceneMetrics::new();
        state.update(&metrics); // clear skipTransitions

        metrics.graveyard_tile_count = thresholds::GRAVEYARD_TILE_MAX;
        state.update(&metrics);
        assert!((state.graveyard_visual_intensity - 0.02).abs() < 1e-6);

        for _ in 0..49 {
            state.update(&metrics);
        }
        assert!((state.graveyard_visual_intensity - 1.0).abs() < 1e-5);

        // Halfway target at count 26.
        metrics.graveyard_tile_count = 26;
        for _ in 0..200 {
            state.update(&metrics);
        }
        assert!((state.graveyard_visual_intensity - 0.5).abs() < 1e-5);

        // Zero graves: fades back out at 0.1/tick.
        metrics.graveyard_tile_count = 0;
        state.update(&metrics);
        assert!((state.graveyard_visual_intensity - 0.4).abs() < 1e-5);
    }

    #[test]
    fn light_decay_fades_to_one() {
        // `SceneState.cs:255-281`: targets 1/1 (shimmer and FTW terms are 0).
        let mut state = SceneState::new();
        state.air_light_decay = 0.9;
        state.solid_light_decay = 0.5;
        let metrics = SceneMetrics::new();
        state.update(&metrics); // skip=true: snap to target
        assert_eq!(state.air_light_decay, 1.0);
        assert_eq!(state.solid_light_decay, 1.0);
    }

    // --- TR_ forces - DW-1.4 ---

    #[test]
    fn test_DW_1_4_parse_env_force() {
        assert_eq!(
            parse_env_force("TR_SANDSTORM=0.75"),
            Some(("SANDSTORM", 0.75))
        );
        assert_eq!(parse_env_force("TR_BLOODMOON"), Some(("BLOODMOON", 1.0)));
        assert_eq!(parse_env_force("TR_BLOODMOON="), Some(("BLOODMOON", 1.0)));
        assert_eq!(parse_env_force("TR_GRAVEYARD=1"), Some(("GRAVEYARD", 1.0)));
        // Clamped to 0..1.
        assert_eq!(parse_env_force("TR_SANDSTORM=5"), Some(("SANDSTORM", 1.0)));
        assert_eq!(parse_env_force("TR_SANDSTORM=-3"), Some(("SANDSTORM", 0.0)));
        // Junk is rejected, not defaulted.
        assert_eq!(parse_env_force("TR_SANDSTORM=abc"), None);
        assert_eq!(parse_env_force("TR_SANDSTORM=nan"), None);
        assert_eq!(parse_env_force("WRONG_X=1"), None);
        assert_eq!(parse_env_force("TR_=1"), None);
    }

    #[test]
    fn test_DW_1_4_force_pins_severity_and_graveyard() {
        // SANDSTORM: the pin lands on the storm via force_severity.
        let mut state = SceneState::new();
        assert!(state.force("SANDSTORM", 0.75));
        assert_eq!(state.forced_severity, Some(0.75));
        let mut storm = Sandstorm::new(1);
        storm.force_severity(state.forced_severity.unwrap());
        assert!(storm.happening);
        for _ in 0..50 {
            storm.update_severity();
        }
        assert_eq!(storm.severity, 0.75);

        // Zero severity pins "no storm".
        let mut storm = Sandstorm::new(1);
        storm.force_severity(0.0);
        assert!(!storm.happening);

        // GRAVEYARD: update() holds the forced value against the fade.
        let mut state = SceneState::new();
        assert!(state.force("GRAVEYARD", 0.8));
        let metrics = SceneMetrics::new();
        state.update(&metrics);
        assert!((state.graveyard_visual_intensity - 0.8).abs() < 1e-6);
        for _ in 0..10 {
            state.update(&metrics);
        }
        assert!((state.graveyard_visual_intensity - 0.8).abs() < 1e-6);

        // BLOODMOON: boolean flag.
        let mut state = SceneState::new();
        assert!(state.force("BLOODMOON", 1.0));
        assert!(state.blood_moon);
        assert!(state.force("BLOODMOON", 0.0));
        assert!(!state.blood_moon);

        // Unknown names are refused.
        let mut state = SceneState::new();
        assert!(!state.force("SOMETHINGELSE", 1.0));
    }

    #[test]
    fn reset_restores_vanilla_defaults() {
        // `SceneState.cs:44-54`.
        let state = SceneState::new();
        assert_eq!(state.air_light_decay, 1.0);
        assert_eq!(state.solid_light_decay, 1.0);
        assert_eq!(state.outside_weather_effect_intensity, 1.0);
        assert_eq!(state.outside_weather_effect_intensity_backing, 1.0);
        assert_eq!(state.graveyard_visual_intensity, 0.0);
        // `_blizzardSoundVolume = 0f` (`SceneState.cs:44`).
        assert_eq!(state.blizzard_sound_volume, 0.0);
        assert!(state.skip_transitions);
        assert!(!state.blood_moon);
    }

    #[test]
    fn get_lerp_value_matches_utils() {
        // `Utils.cs:315-343` clamped form.
        assert_eq!(get_lerp_value(16.0, 36.0, 16.0, true), 0.0);
        assert_eq!(get_lerp_value(16.0, 36.0, 36.0, true), 1.0);
        assert_eq!(get_lerp_value(16.0, 36.0, 26.0, true), 0.5);
        assert_eq!(get_lerp_value(16.0, 36.0, 0.0, true), 0.0);
        assert_eq!(get_lerp_value(16.0, 36.0, 100.0, true), 1.0);
    }

    // --- FilterBoard (Filter.cs / FilterManager.cs / SceneState.cs:311-366) ---

    #[test]
    fn test_DW_3_4_board_activate_fade_in() {
        // `FilterManager.Update` (`:111-112`): Active → Opacity += 1/sec.
        let mut board = FilterBoard::new();
        board.manage(FilterId::BloodMoon, true, false);
        assert!(board.is_active(FilterId::BloodMoon));
        assert_eq!(board.opacity(FilterId::BloodMoon), 0.0);
        board.update(1.0 / 60.0);
        assert!((board.opacity(FilterId::BloodMoon) - 1.0 / 60.0).abs() < 1e-7);
        for _ in 0..59 {
            board.update(1.0 / 60.0);
        }
        assert!((board.opacity(FilterId::BloodMoon) - 1.0).abs() < 1e-5);
        // Saturation at 1: more time does not overshoot.
        board.update(0.5);
        assert_eq!(board.opacity(FilterId::BloodMoon), 1.0);
    }

    #[test]
    fn test_DW_3_4_board_deactivate_fade_out_and_removal() {
        // `FilterManager.Update` (`:114-115`): inactive → Opacity -= 1/sec,
        // removal once it drains (`:117-124`).
        let mut board = FilterBoard::new();
        board.manage(FilterId::BloodMoon, true, true); // snap to 1
        assert_eq!(board.opacity(FilterId::BloodMoon), 1.0);
        board.manage(FilterId::BloodMoon, false, false);
        assert!(!board.is_active(FilterId::BloodMoon));
        assert!(board.in_list(FilterId::BloodMoon), "still fading");
        board.update(0.4);
        assert!((board.opacity(FilterId::BloodMoon) - 0.6).abs() < 1e-6);
        board.update(0.6);
        assert_eq!(board.opacity(FilterId::BloodMoon), 0.0);
        assert!(
            !board.in_list(FilterId::BloodMoon),
            "drained slots leave the list"
        );
    }

    #[test]
    fn test_DW_3_4_winner_priority_tie_activation_order() {
        // `EndCapture`'s last-visible-wins over the (priority, insertion)-sorted
        // list (`FilterManager.cs:71-80,154-180`).
        let mut board = FilterBoard::new();
        // Blood moon first, graveyard the same frame later (ApplyVisuals order:
        // `:92` then `:100`) — both Medium, the later activation wins.
        board.manage(FilterId::BloodMoon, true, true);
        board.manage(FilterId::Graveyard, true, true);
        assert_eq!(board.winner(), Some(FilterId::Graveyard));

        // The reverse order flips the winner.
        let mut board = FilterBoard::new();
        board.manage(FilterId::Graveyard, true, true);
        board.manage(FilterId::BloodMoon, true, true);
        assert_eq!(board.winner(), Some(FilterId::BloodMoon));

        // A High storm beats both Medium filters regardless of order.
        board.manage(FilterId::Sandstorm, true, true);
        assert_eq!(board.winner(), Some(FilterId::Sandstorm));

        // Overlays never compete (`EndCapture` walks `Filters.Scene` only).
        board.manage(FilterId::SandstormOverlay, true, true);
        assert_eq!(board.winner(), Some(FilterId::Sandstorm));

        // During fade-out a filter stays visible and keeps its list position:
        // deactivating the storm hands the pass back to the graveyard.
        board.manage(FilterId::Sandstorm, false, false);
        assert_eq!(board.winner(), Some(FilterId::Sandstorm), "still fading");
        board.update(1.0);
        assert_eq!(board.winner(), Some(FilterId::BloodMoon));

        // Nothing visible: no filter runs.
        let empty = FilterBoard::new();
        assert_eq!(empty.winner(), None);
    }

    #[test]
    fn winner_by_skips_zero_combined_opacity() {
        // `Filter.IsVisible` (`Filter.cs:63-70`): a later same-priority filter
        // with CombinedOpacity 0 does not shadow the earlier visible one.
        let mut board = FilterBoard::new();
        board.manage(FilterId::Graveyard, true, true);
        board.manage(FilterId::BloodMoon, true, true);
        // Inactive filters have a drained fade (0), as the driver reports.
        let combined = |moon: f32| {
            let board = &board;
            move |id| match id {
                FilterId::BloodMoon => moon,
                _ => board.opacity(id),
            }
        };
        assert_eq!(board.winner_by(combined(0.0)), Some(FilterId::Graveyard));
        assert_eq!(board.winner_by(combined(0.5)), Some(FilterId::BloodMoon));
        assert_eq!(board.winner_by(|_| 0.0), None);
    }

    #[test]
    fn test_DW_3_4_skip_transitions_snap() {
        // `ManageSpecialBiomeVisuals`'s instant tail (`SceneState.cs:355-365`).
        let mut board = FilterBoard::new();
        board.manage(FilterId::Graveyard, true, true);
        assert_eq!(board.opacity(FilterId::Graveyard), 1.0);
        board.manage(FilterId::Graveyard, false, true);
        assert_eq!(board.opacity(FilterId::Graveyard), 0.0);

        // Without the snap the fade runs; the snap also applies with no edge.
        let mut board = FilterBoard::new();
        board.manage(FilterId::Graveyard, true, false);
        board.update(0.25);
        board.manage(FilterId::Graveyard, true, true); // already active, still snaps
        assert_eq!(board.opacity(FilterId::Graveyard), 1.0);
    }

    #[test]
    fn test_DW_3_4_reactivation_resumes_opacity() {
        // `FilterManager.OnActivate` resets the opacity only for a filter that
        // had fully left the list (`:58-61` vs `:46-57`).
        let mut board = FilterBoard::new();
        board.manage(FilterId::BloodMoon, true, false);
        board.update(0.5);
        board.manage(FilterId::BloodMoon, false, false);
        board.update(0.25); // opacity 0.25, still fading
        board.manage(FilterId::BloodMoon, true, false);
        board.update(0.25);
        assert!((board.opacity(FilterId::BloodMoon) - 0.5).abs() < 1e-6);

        // After a full drain the fade restarts from 0.
        board.manage(FilterId::BloodMoon, false, false);
        board.update(1.0);
        board.manage(FilterId::BloodMoon, true, false);
        assert_eq!(board.opacity(FilterId::BloodMoon), 0.0);
    }

    // --- StormShader (SandstormShaderData.cs / BlizzardShaderData.cs) ---

    #[test]
    fn test_DW_3_3_storm_sandstorm_direction_animation() {
        // `SandstormShaderData.Update` (`:10-23`): (-wind*20, -0.1) normalized,
        // × (2, 0.2); with wind = 0 the direction is (0, -0.2) normalized.
        let mut fx = StormShader::new();
        fx.update_sandstorm(0.0, 1.0);
        let d = fx.direction();
        let len = (d[0] * d[0] + d[1] * d[1]).sqrt();
        assert!((len - 0.2).abs() < 1e-6, "normalized × (2, 0.2)");
        assert!(d[0].abs() < 1e-6 && (d[1] + 0.2).abs() < 1e-6);

        // The scroll advances by direction × dt, wrapped mod 10: two 1-second
        // steps at (0, -0.2) reach -0.4.
        fx.update_sandstorm(0.0, 1.0);
        assert!((fx.texture_pos()[1] + 0.4).abs() < 1e-6);
        // A strong wind drives the horizontal component.
        let mut fx = StormShader::new();
        fx.update_sandstorm(1.0, 1.0);
        assert!(fx.direction()[0] < -1.0, "wind 1 pushes the noise left");
    }

    #[test]
    fn test_DW_3_3_storm_blizzard_wind_smoothing() {
        // `BlizzardShaderData.Update` (`:12-36`): |wind| < 0.1 clamps to ±0.1,
        // then windSpeed smooths at 0.05/0.95 from the 0.1 initializer.
        let mut fx = StormShader::new();
        fx.update_blizzard(0.0, 1.0);
        assert!((fx.wind_speed - (0.1 * 0.05 + 0.1 * 0.95)).abs() < 1e-6); // 0.1
        let mut fx = StormShader::new();
        fx.update_blizzard(1.0, 1.0);
        assert!((fx.wind_speed - (1.0 * 0.05 + 0.1 * 0.95)).abs() < 1e-6); // 0.145
        // Direction: (-windSpeed, -1) × (10, 2) normalized × (0.8, 0.6).
        let d = fx.direction();
        assert!(d[0] < 0.0 && d[1] < 0.0);
        // The scroll wraps into ±10.
        for _ in 0..6000 {
            fx.update_blizzard(1.0, 1.0);
        }
        assert!(fx.texture_pos()[0].abs() <= 10.0 && fx.texture_pos()[1].abs() <= 10.0);
    }

    // --- Parameter formulas (SceneState.cs:107-188 / BloodMoonShaderData) ---

    #[test]
    fn test_DW_3_2_graveyard_progress() {
        // `SceneState.cs:109`: Lerp(0, 0.75, intensity).
        assert_eq!(graveyard_progress(0.0), 0.0);
        assert!((graveyard_progress(0.5) - 0.375).abs() < 1e-6);
        assert_eq!(graveyard_progress(1.0), 0.75);
    }

    #[test]
    fn test_DW_3_1_blood_moon_opacity_depth_fade() {
        // `BloodMoonScreenShaderData.cs:15,24`: full above worldSurface + 50,
        // zero below rockLayer + 100, smooth in between, × 0.75.
        let surface = 100.0f64;
        let rock = 180.0f64;
        assert_eq!(blood_moon_opacity(100.0, surface, rock), 0.75);
        assert_eq!(blood_moon_opacity(150.0, surface, rock), 0.75); // at +50
        assert_eq!(blood_moon_opacity(280.0, surface, rock), 0.0); // at rock+100
        // Midpoint: SmoothStep is a plain inverse lerp (`Utils.cs:126-129`).
        let mid = blood_moon_opacity(215.0, surface, rock); // (215-150)/(280-150) = 0.5
        assert!((mid - 0.375).abs() < 1e-6);
    }

    #[test]
    fn test_DW_3_2_noir_intensity_formula() {
        // `SceneState.cs:117`. Structure: Lerp(Lerp(0.1, tileColorTerm,
        // surfaceLeg), 0.15, underworldLeg) — the surface leg is 1 when the
        // TileCenter sits 40+ tiles ABOVE the surface ((int)surface - Y >= 40),
        // 0 at 40 below; the underworld leg is 1 deep into the underworld.
        // Bright sky high above the surface: base = Remap(1, 0.5, 0.1, 0, 0.2)
        // = 0 (inverted range, clamped) → intensity 0.
        assert!((noir_intensity([1.0, 1.0, 1.0], 100, 60, 200) - 0.0).abs() < 1e-6);
        // Dark tiles up there read as full noir: base = 0.2.
        assert!((noir_intensity([0.1, 0.1, 0.1], 100, 60, 200) - 0.2).abs() < 1e-6);
        // Mid-brightness: GetLerpValue(0.5, 0.1, 0.3) = 0.5 → base 0.1.
        assert!((noir_intensity([0.3, 0.3, 0.3], 100, 60, 200) - 0.1).abs() < 1e-6);
        // Underground the surface leg is 0 → the floor is 0.1, rising toward
        // 0.15 with underworld depth (Y=180 is 20 above the underworld layer:
        // leg 0.25 → 0.1125).
        assert!((noir_intensity([1.0, 1.0, 1.0], 100, 180, 200) - 0.1125).abs() < 1e-6);
        // Deep underworld: the third leg saturates at 0.15.
        assert!((noir_intensity([1.0, 1.0, 1.0], 100, 240, 200) - 0.15).abs() < 1e-6);
    }

    #[test]
    fn test_DW_3_3_sandstorm_params() {
        // `SceneState.cs:138-140`.
        let (intensity, opacity) = sandstorm_foreground_params(0.8, 1.0);
        assert!((intensity - 0.4 * 0.8).abs() < 1e-6);
        let saturate = (0.8f32 * 1.5).min(1.0);
        assert!((opacity - saturate).abs() < 1e-6);
        // Severity saturates both terms.
        let (intensity, opacity) = sandstorm_foreground_params(1.0, 1.0);
        assert_eq!(intensity, 0.4);
        assert_eq!(opacity, 1.0);
        // The overlay is the complement (`:140`).
        let bg = sandstorm_background_opacity(0.8, 1.0);
        assert_eq!(bg, 0.0);
        let bg = sandstorm_background_opacity(0.8, 0.0);
        let saturate = (0.8f32 * 1.5).min(1.0);
        assert!((bg - saturate).abs() < 1e-6);
        // Behind a backwall the intensity zeroes (`ow = 0`).
        let (intensity, opacity) = sandstorm_foreground_params(0.8, 0.0);
        assert_eq!(intensity, 0.0);
        assert_eq!(opacity, 0.0);
    }

    #[test]
    fn test_DW_3_3_blizzard_params() {
        // `SceneState.cs:175-181` with the deerclops term 0.
        let (intensity, fg, bg) = blizzard_foreground_params(0.5, 1.0, 0.0);
        assert!((intensity - (1.0 * 0.4 * 1.0 * 0.9 + 0.1)).abs() < 1e-6);
        assert!((fg - 1.0).abs() < 1e-6);
        assert!((bg - 0.0).abs() < 1e-6);
        // Calm clouds halve the terms: min(1, 0.2*2) = 0.4.
        let (intensity, fg, bg) = blizzard_foreground_params(0.2, 1.0, 0.0);
        assert!((intensity - (1.0 * 0.4 * 0.4 * 0.9 + 0.1)).abs() < 1e-6);
        assert!((fg - 0.4).abs() < 1e-6);
        assert!((bg - 0.6).abs() < 1e-6);
        // Behind a backwall the foreground fades out, the background fills in.
        let (_, fg, bg) = blizzard_foreground_params(0.5, 0.0, 0.0);
        assert_eq!(fg, 0.0);
        assert_eq!(bg, 1.0);
    }

    #[test]
    fn desert_bg_variant_set_matches_surface_background_id() {
        // `SurfaceBackgroundID.cs:18`: CreateBoolSet(false, 2, 5, 13, 14).
        for style in [2, 5, 13, 14] {
            assert!(is_desert_bg_style(style));
        }
        for style in [0, 1, 4, 7, 10] {
            assert!(!is_desert_bg_style(style));
        }
    }

    #[test]
    fn noir_image_offset_is_deterministic_per_tenth_second() {
        // `SceneState.cs:118`: the seed is (int)(GlobalTime * 10) — stable
        // within a tenth of a second, changing across tenths.
        let a = noir_image_offset(1.234);
        let b = noir_image_offset(1.239);
        assert_eq!(a, b);
        let c = noir_image_offset(1.301);
        assert_ne!(a, c);
        // Repeats exactly (determinism).
        assert_eq!(a, noir_image_offset(1.234));
    }

    // --- Distortion filters (phase 4) ---

    /// Shared fixture: 4200x1200 world, surface 300, rock 450.
    fn water_i(x: i32, y: i32, wind: f32, rain: f32) -> f32 {
        water_distortion_intensity((x, y), 4200, 1200, wind, rain, 300.0, 450.0)
    }

    #[test]
    fn test_DW_4_3_water_intensity_world_centre_and_edge() {
        // `SceneState.cs:128-129`: mid-world the edge term is -0.25 (clamp = 1),
        // at the world edge it is +0.25 (clamp = 0).
        assert!((water_i(2100, 300, 0.0, 0.0) - 0.75).abs() < 1e-6);
        assert!((water_i(100, 300, 0.0, 0.0) - 1.25).abs() < 1e-6);
        // Edge ramp: dist 430 -> (430-380)/100 = 0.5 -> 0.25 - 0.25 -> num4 = 1.0.
        assert!((water_i(430, 300, 0.0, 0.0) - 1.0).abs() < 1e-6);
        // The east edge mirrors the west one.
        assert!((water_i(4200 - 100, 300, 0.0, 0.0) - 1.25).abs() < 1e-6);
    }

    #[test]
    fn test_DW_4_3_water_intensity_wind_and_rain_terms() {
        // `:129`: |wind| * 1.25 + clamp(maxRaining, 0, 1) * 1.25.
        assert!((water_i(2100, 300, -0.4, 0.5) - 1.875).abs() < 1e-6);
        // maxRaining clamps at 1 (`MathHelper.Clamp(maxRaining, 0, 1)`).
        assert!((water_i(2100, 300, 0.0, 3.0) - water_i(2100, 300, 0.0, 1.0)).abs() < 1e-6);
        // The final clamp caps at 2.5.
        assert_eq!(water_i(100, 300, 1.2, 1.0), 2.5);
    }

    #[test]
    fn test_DW_4_3_water_intensity_depth_and_underworld_layers() {
        // `:130-131`: num5 fades the wind/rain/edge terms out over the first
        // third of surface..rock, leaving the flat 0.75 term.
        assert!((water_i(100, 325, 0.0, 0.0) - 1.0).abs() < 1e-5);
        assert!((water_i(100, 350, 0.0, 0.0) - 0.75).abs() < 1e-5);
        // Above the surface num5 stays 1 (clamp of a negative depth).
        assert!((water_i(100, 100, 0.0, 0.0) - 1.25).abs() < 1e-6);
        // Underworld ramp: +0.9 at maxTilesY - 200 and below.
        assert!((water_i(2100, 900, 0.0, 0.0) - 1.35).abs() < 1e-5);
        assert!((water_i(2100, 1000, 0.0, 0.0) - 1.65).abs() < 1e-5);
        assert!((water_i(2100, 1150, 0.0, 0.0) - 1.65).abs() < 1e-5);
    }

    #[test]
    fn test_DW_4_3_water_distortion_gate_and_progress_clock() {
        // `SceneState.cs:105`: WaveQuality > 0.
        assert!(!water_distortion_active(0));
        assert!(water_distortion_active(1));
        assert!(water_distortion_active(3));
        // `WaterShaderData.cs:126-127`.
        let mut shader = WaterShader::new();
        assert_eq!((shader.progress, shader.intensity), (0.0, 1.0));
        shader.update(1.0);
        assert!((shader.progress - 0.75).abs() < 1e-6);
        shader.intensity = 2.0;
        shader.update(0.5);
        assert!((shader.progress - 1.5).abs() < 1e-6);
        shader.progress = 86399.9;
        shader.update(1.0);
        assert!(shader.progress < 2.0, "wraps at 86400");
    }

    #[test]
    fn test_DW_4_1_heat_zone_flags_and_activation() {
        // `SceneState.cs:79-80`: flag2 = Y > maxTilesY - 320; flag3 = above the
        // surface desert without rain/sandstorm.
        assert_eq!(
            heat_zone_flags(881, 1200, 300.0, false, false, false),
            (true, false)
        );
        assert_eq!(
            heat_zone_flags(880, 1200, 300.0, false, false, false),
            (false, false),
            "strictly greater"
        );
        assert_eq!(
            heat_zone_flags(299, 1200, 300.0, true, false, false),
            (false, true)
        );
        assert_eq!(
            heat_zone_flags(300, 1200, 300.0, true, false, false),
            (false, false),
            "Y < worldSurface is strict"
        );
        assert_eq!(
            heat_zone_flags(299, 1200, 300.0, true, true, false),
            (false, false),
            "rain cancels the desert flag"
        );
        assert_eq!(
            heat_zone_flags(299, 1200, 300.0, true, false, true),
            (false, false),
            "sandstorm cancels the desert flag"
        );
        // `:98-99`: the user toggle gates everything.
        assert!(heat_distortion_active(true, true, false, 0));
        assert!(heat_distortion_active(true, false, true, 0));
        assert!(heat_distortion_active(true, false, false, 1), "sun scorch");
        assert!(!heat_distortion_active(true, false, false, 0));
        assert!(!heat_distortion_active(false, true, true, 99));
    }

    #[test]
    fn test_DW_4_1_heat_intensity_branches() {
        let white = [1.0; 3];
        let h = 1200;
        let heat =
            |sc, uw, de, y, ow, ca, tc| heat_distortion_intensity(sc, uw, de, y, h, ow, ca, tc);
        // Underworld: min(1, (Y - (H-320)) / 120) * 2.
        assert!((heat(0, true, false, 880 + 60, 1.0, 0.0, white) - 1.0).abs() < 1e-6);
        assert!((heat(0, true, false, 880 + 120, 1.0, 0.0, white) - 2.0).abs() < 1e-6);
        assert!((heat(0, true, false, 1190, 1.0, 0.0, white) - 2.0).abs() < 1e-6);
        // Desert surface: weather fade * 4 * max(0, 0.5 - cloudAlpha) * brightness.
        assert!((heat(0, false, true, 250, 1.0, 0.0, [0.8; 3]) - 1.6).abs() < 1e-6);
        assert_eq!(heat(0, false, true, 250, 1.0, 0.5, white), 0.0);
        assert_eq!(heat(0, false, true, 250, 0.0, 0.0, white), 0.0);
        // The underworld branch wins over the desert branch (`else if`).
        assert!((heat(0, true, true, 880 + 60, 1.0, 0.0, white) - 1.0).abs() < 1e-6);
        // Sun scorch: lerp(0, 300, counter) * 4, max-merged with the zone term.
        assert!((heat(150, false, false, 250, 1.0, 0.0, white) - 2.0).abs() < 1e-6);
        assert!((heat(600, false, false, 250, 1.0, 0.0, white) - 4.0).abs() < 1e-6);
        assert!((heat(75, true, false, 880 + 120, 1.0, 0.0, white) - 2.0).abs() < 1e-6);
        assert_eq!(heat(0, false, false, 250, 1.0, 0.0, white), 0.0);
    }

    #[test]
    fn test_DW_4_1_distortion_ids_stay_out_of_the_winner_pool() {
        // Priorities `ScreenEffectInitializer.cs:41-42`.
        assert_eq!(FilterId::HeatDistortion.priority(), EffectPriority::Low);
        assert_eq!(
            FilterId::WaterDistortion.priority(),
            EffectPriority::VeryHigh
        );
        assert_eq!(FilterId::HeatDistortion.name(), "HeatDistortion");
        assert_eq!(FilterId::WaterDistortion.name(), "WaterDistortion");
        for id in FilterId::DISTORTIONS {
            assert!(!FilterId::FILTERS.contains(&id));
            assert!(FilterId::ALL.contains(&id));
            assert!(!id.is_overlay());
        }
        // A VeryHigh water slot never steals the tint winner.
        let mut board = FilterBoard::new();
        board.manage(FilterId::WaterDistortion, true, true);
        assert_eq!(board.winner(), None);
        board.manage(FilterId::BloodMoon, true, true);
        assert_eq!(board.winner(), Some(FilterId::BloodMoon));
        assert!(board.is_visible(FilterId::WaterDistortion));
    }

    #[test]
    fn test_DW_4_1_hidden_filter_is_not_visible() {
        // `Filter.IsVisible` (`Filter.cs:63-70`); HeatDistortion hides itself at 0.
        let mut board = FilterBoard::new();
        board.manage(FilterId::HeatDistortion, true, true);
        assert!(board.is_visible(FilterId::HeatDistortion));
        board.set_hidden(FilterId::HeatDistortion, true);
        assert!(!board.is_visible(FilterId::HeatDistortion));
        assert!(
            board.is_active(FilterId::HeatDistortion),
            "hidden != inactive"
        );
        board.set_hidden(FilterId::HeatDistortion, false);
        assert!(board.is_visible(FilterId::HeatDistortion));
        // A hidden tint filter drops out of the winner pick too.
        board.manage(FilterId::BloodMoon, true, true);
        board.set_hidden(FilterId::BloodMoon, true);
        assert_eq!(board.winner(), None);
    }
    // --- biome zones for the ambience gates (SceneMetrics.cs:671-691) ---

    #[test]
    fn corruption_crimson_hallow_jungle_thresholds_bound_their_zones() {
        // Thresholds: corruption/crimson 300, hallow 125, jungle 140
        // (`SceneMetrics.cs:922-929`).
        let cases: [(u16, u16, i32, &str); 4] = [
            (23, 0, 300, "corrupt"),  // ebonstone
            (199, 0, 300, "crimson"), // crimstone
            (109, 0, 125, "hallow"),  // hallowed grass
            (60, 0, 140, "jungle"),   // jungle grass
        ];
        for &(id, _, threshold, name) in &cases {
            // One tile short: no zone.
            let mut m = metrics_with_counts(&[(id, (threshold - 1) as u32)]);
            m.tile_center = (100, 100);
            m.aggregate_tile_counts();
            m.calculate_zones(300.0, 1000, 135.0, 4200, 400, false, false);
            let zone = match name {
                "corrupt" => m.zone_corrupt,
                "crimson" => m.zone_crimson,
                "hallow" => m.zone_hallow,
                _ => m.zone_jungle,
            };
            assert!(!zone, "{name} below threshold");

            // Exactly at the threshold: zone on.
            let mut m = metrics_with_counts(&[(id, threshold as u32)]);
            m.tile_center = (100, 100);
            m.aggregate_tile_counts();
            m.calculate_zones(300.0, 1000, 135.0, 4200, 400, false, false);
            let zone = match name {
                "corrupt" => m.zone_corrupt,
                "crimson" => m.zone_crimson,
                "hallow" => m.zone_hallow,
                _ => m.zone_jungle,
            };
            assert!(zone, "{name} at threshold");
        }
    }

    #[test]
    fn jungle_needs_the_overworld_and_sunflowers_suppress_evil() {
        // `ZoneJungle = EnoughTilesForJungle && !ZoneUnderworldHeight`
        // (`SceneMetrics.cs:674`): the same count in the underworld is not a
        // jungle (UnderworldLayer = maxTilesY - 200 -> 1000 here).
        let mut m = metrics_with_counts(&[(60, 140)]);
        m.tile_center = (100, 1001);
        m.aggregate_tile_counts();
        m.calculate_zones(300.0, 1000, 135.0, 4200, 400, false, false);
        assert!(!m.zone_jungle);

        // Sunflowers count as -10 evil tiles each (`SceneMetrics.cs:575,613`),
        // so 31 sunflowers cancel 300 ebonstone... 31 * -10 = -310: exactly
        // one tile short stays short, one more cancels the zone entirely.
        let mut m = metrics_with_counts(&[(23, 300), (27, 31)]);
        m.tile_center = (100, 100);
        m.aggregate_tile_counts();
        assert_eq!(m.evil_tile_count, 0); // 300 - 310 clamped to 0
        m.calculate_zones(300.0, 1000, 135.0, 4200, 400, false, false);
        assert!(!m.zone_corrupt);
    }

    #[test]
    fn holy_and_evil_cancel_each_other_before_the_thresholds() {
        // The cross-subtraction (`SceneMetrics.cs:645-653`): holy loses to both
        // evil and blood, evil/blood each lose to the *pre-subtraction* holy.
        // 200 holy + 250 evil: holy -> 0, evil -> 50 (< 300: no zone).
        let mut m = metrics_with_counts(&[(109, 200), (23, 250)]);
        m.tile_center = (100, 100);
        m.aggregate_tile_counts();
        assert_eq!(m.holy_tile_count, 0);
        assert_eq!(m.evil_tile_count, 50);
        m.calculate_zones(300.0, 1000, 135.0, 4200, 400, false, false);
        assert!(!m.zone_hallow);
        assert!(!m.zone_corrupt);
    }

    #[test]
    fn zone_beach_follows_ocean_depths() {
        // `ZoneBeach = WorldGen.oceanDepths(TileCenter)` (`SceneMetrics.cs:691`):
        // inside the 380-tile edge bands above oceanLevel, else false.
        let mut m = SceneMetrics::new();
        m.tile_center = (100, 50); // x < 380, y <= oceanLevel
        m.aggregate_tile_counts();
        m.calculate_zones(300.0, 1000, 135.0, 4200, 400, false, false);
        assert!(m.zone_beach);

        let mut m = SceneMetrics::new();
        m.tile_center = (2000, 50); // mid-world
        m.aggregate_tile_counts();
        m.calculate_zones(300.0, 1000, 135.0, 4200, 400, false, false);
        assert!(!m.zone_beach);
    }

    /// `SceneState.UpdateShimmer` (`SceneState.cs:283-309`): the zone ramps
    /// the alpha at 0.025/tick, the darken follows past 0.5; losing the zone
    /// decays the darken first, then (after the 4-tick delay) the alpha.
    #[test]
    fn shimmer_ramps_follow_the_zone() {
        let mut m = SceneMetrics::new();
        m.shimmer_tile_count = thresholds::SHIMMER_TILE;
        m.underground_for_shimmering = true;
        m.zone_shimmer = true;
        let mut st = SceneState::new();
        for _ in 0..40 {
            st.update(&m);
        }
        assert!(
            st.shimmer_alpha >= 0.99,
            "alpha ramps in: {}",
            st.shimmer_alpha
        );
        assert!(
            st.shimmer_darken > 0.9,
            "darken follows: {}",
            st.shimmer_darken
        );

        // The zone off: the darken decays at 0.05/tick while the delay holds
        // the alpha; the alpha decays after the delay drains.
        m.zone_shimmer = false;
        for _ in 0..20 {
            st.update(&m);
        }
        assert!(
            st.shimmer_darken < 0.05,
            "darken decays: {}",
            st.shimmer_darken
        );
        assert!(
            st.shimmer_alpha > 0.9,
            "alpha held by the delay: {}",
            st.shimmer_alpha
        );
        for _ in 0..60 {
            st.update(&m);
        }
        assert!(
            st.shimmer_alpha < 0.05,
            "alpha decays after the delay: {}",
            st.shimmer_alpha
        );
    }

    /// The zone needs the depth window (`SceneMetrics.cs:704`): the count
    /// alone is not enough above `worldSurface + 84`.
    #[test]
    fn shimmer_zone_needs_the_depth_window() {
        let mut m = SceneMetrics::new();
        m.shimmer_tile_count = thresholds::SHIMMER_TILE;
        m.tile_center = (500, 100); // above the surface+84 line
        m.calculate_zones(200.0, 1000, 135.0, 4200, 1200, false, false);
        assert!(!m.zone_shimmer, "too shallow");
        m.tile_center = (500, 400); // 200+84 < 400 < 1200-396
        m.calculate_zones(200.0, 1000, 135.0, 4200, 1200, false, false);
        assert!(m.zone_shimmer, "in the window");
    }
}
