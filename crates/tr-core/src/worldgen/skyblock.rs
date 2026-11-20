//! The Skyblock generator's statics + gate census — `WorldGen.Skyblock`
//! (`WorldGen.cs:3063-3281`) and the `denyAllGeneration` /
//! `denySomeGeneration` pass gates the C# reads inside each pass body.
//!
//! W4-f24 lane A (the gated `Skyblock` pass, `WorldGen.cs:11511`). The C#
//! `Skyblock.denyAllGeneration => WorldGen.skyblockWorldGen` (`:3131`) is
//! true for EVERY skyblock world, so the 75 passes whose body opens with
//! `if (!Skyblock.denyAllGeneration …)` never run; the 22 `denySomeGeneration`
//! passes (`:3133-3147`) run only when one of the eight listed micro-seeds
//! is on (or extraLivingTrees with none of them). The port models this as a
//! pruning table consulted by `pass_disabled_by_special_seeds` — the same
//! seam the dualDungeons pruning uses (`pipeline.rs`).
//!
//! The `Skyblock` statics (`noAltars`/`noDungeon`/…/`hasTile`/`hasWall`/
//! `lowTiles`) are the runtime tile-presence scan (`ScanTiles`/`Calculate`,
//! `:3149-3265`) the skyblock shop/spawn conditions read; the port carries
//! them on a struct the generator owns (the C# statics are world-global).
//! `Calculate`'s `NetMessage.SendData(7)` (`:3234`) is a runtime sync — no
//! worldgen effect, cited skip.
//!
//! UNWIRED until A4 lands the gated step: the pruning tables only fire under
//! `SeedFlags::SKYBLOCK`, so a classic world runs the byte-identical ledger —
//! both classic world-hash pins stay; `a == b` untouched.

use crate::seeds::{MicroSeedFlags, SeedFlags};
use crate::tile::FLAG_ACTIVE;
use crate::tiledata::{TILE_COUNT, WALL_COUNT};
use crate::world::World;

/// `Skyblock.denyAllGeneration` (`WorldGen.cs:3131`) — true for every
/// skyblock world. The 75 passes whose body gates on it (the C# anchor
/// census, `AddPasses` `:11502-22143`, mapped to the port's `VANILLA_CHAIN`
/// names). TerrainPass is a class pass with no inline gate (it always runs;
/// the Skyblock pass clears its tiles), so it is NOT here.
pub(crate) const SKYBLOCK_DENY_ALL_PASSES: &[&str] = &[
    "Dunes",                          // WorldGen.cs:11517
    "Ocean Sand",                     // :11580
    "Sand Patches",                   // :11689
    "Tunnels",                        // :11718
    "Mount Caves",                    // :11784
    "Dirt Wall Backgrounds",          // :11858
    "Rocks In Dirt",                  // :11896
    "Dirt In Rocks",                  // :11926
    "Clay",                           // :11959
    "Small Holes",                    // :12008
    "Dirt Layer Caves",               // :12067
    "Rock Layer Caves",               // :12107
    "Surface Caves",                  // :12162
    "Wavy Caves",                     // :12271
    "Generate Ice Biome",             // :12312
    "Grass",                          // :12423
    "Mud Caves To Grass",             // :12468
    "Full Desert",                    // :12500
    "Mushroom Patches",               // :12544
    "Marble",                         // :12757
    "Granite",                        // :12839
    "Dirt To Mud",                    // :13123
    "Silt",                           // :13143
    "Shinies",                        // :13182
    "Webs",                           // :13607
    "Underworld",                     // :13645
    "Corruption",                     // :13866
    "Lakes",                          // :14556
    "Slush",                          // :14697
    "Dungeon",                        // :14757
    "Mountain Caves",                 // :14873
    "Beaches",                        // :14888
    "Gems",                           // :15050
    "Gravitating Sand",               // :15138
    "Create Ocean Caves",             // :15167
    "Shimmer",                        // :15195
    "Clean Up Dirt",                  // :15246
    "Pyramids",                       // :15360
    "Dirt Rock Wall Runner",          // :15461
    "Living Trees",                   // :15476
    "Wood Tree Walls",                // :15717
    "Altars",                         // :15750
    "Wet Jungle",                     // :15810
    "Jungle Temple",                  // :15834
    "Hives",                          // :15927
    "Jungle Chests",                  // :15977
    "Settle Liquids",                 // :16132
    "Remove Water From Sand",         // :16211
    "Oasis",                          // :16243
    "Shell Piles",                    // :16266
    "Wall Variety",                   // :16697
    "Jungle Chests Placement",        // :17211
    "Water Chests",                   // :17245
    "Spider Caves",                   // :17356
    "Gem Caves",                      // :17428
    "Moss",                           // :17472
    "Temple",                         // :17701
    "Cave Walls",                     // :17711
    "Jungle Trees",                   // :17840
    "Quick Cleanup",                  // :17875
    "Hellforge",                      // :18187
    "Spreading Grass",                // :18237
    "Surface Ore and Stone",          // :18447
    "Grass Wall",                     // :19589
    "Herbs",                          // :19964
    "Dye Plants",                     // :19981
    "Gems In Ice Biome",              // :20697
    "Random Gems",                    // :20729
    "Muds Walls In Jungle",           // :20813
    "Larva",                          // :20868
    "Micro Biomes",                   // :20901
    "Settle Liquids Again",           // :21246
    "Cactus, Palm Trees, & Coral",    // :21310
    "Lihzahrd Altars",                // :21943
    "Water Plants",                   // :21973
];

/// `Skyblock.denySomeGeneration` (`WorldGen.cs:3133-3147`) — the 22 passes
/// whose body gates on it. They run under a skyblock world ONLY when one of
/// the eight micro-seeds below is on, or extraLivingTrees with none of them.
pub(crate) const SKYBLOCK_DENY_SOME_PASSES: &[&str] = &[
    "Smooth World",                       // WorldGen.cs:16419
    "Waterfalls",                         // :16610
    "Ice",                                // :16667
    "Life Crystals",                      // :16755
    "Statues",                            // :16853
    "Buried Chests",                      // :16971
    "Surface Chests",                     // :17130
    "Pots",                               // :17996
    "Place Fallen Log",                   // :18513
    "Traps",                              // :18657
    "Piles",                              // :18788
    "Sunflowers",                         // :19898
    "Planting Trees",                     // :19940
    "Webs And Honey",                     // :19995
    "Weeds",                              // :20064
    "Glowing Mushrooms and Jungle Plants", // :20122
    "Jungle Plants",                      // :20165
    "Vines",                              // :20193
    "Flowers",                            // :20447
    "Mushrooms",                          // :20599
    "Moss Grass",                         // :20769
    "Tile Cleanup",                       // :21520
];

/// `Skyblock.denySomeGeneration` (`WorldGen.cs:3139-3141`): true when
/// skyblock AND none of the eight listed micro-seeds is on AND
/// extraLivingTrees is off. The eight: worldIsFrozen, surfaceIsDesert,
/// surfaceIsMushrooms, worldIsInfected, hallowOnTheSurface, noInfection,
/// extraFloatingIslands, extraLiquid.
pub(crate) fn deny_some_generation(flags: SeedFlags, micro: MicroSeedFlags) -> bool {
    if !flags.contains(SeedFlags::SKYBLOCK) {
        return false;
    }
    let any_of_eight = micro.intersects(
        MicroSeedFlags::WORLD_IS_FROZEN
            | MicroSeedFlags::SURFACE_IS_DESERT
            | MicroSeedFlags::SURFACE_IS_MUSHROOMS
            | MicroSeedFlags::WORLD_IS_INFECTED
            | MicroSeedFlags::HALLOW_ON_THE_SURFACE
            | MicroSeedFlags::NO_INFECTION
            | MicroSeedFlags::EXTRA_FLOATING_ISLANDS
            | MicroSeedFlags::EXTRA_LIQUID,
    );
    if any_of_eight {
        return false;
    }
    !micro.contains(MicroSeedFlags::EXTRA_LIVING_TREES)
}

/// `Skyblock.denyFloatingIslands` (`WorldGen.cs:3087-3097`): skyblock AND
/// NOT extraFloatingIslands.
pub(crate) fn deny_floating_islands(flags: SeedFlags, micro: MicroSeedFlags) -> bool {
    flags.contains(SeedFlags::SKYBLOCK) && !micro.contains(MicroSeedFlags::EXTRA_FLOATING_ISLANDS)
}

/// `Skyblock.spawnSolidifier` (`WorldGen.cs:3099-3113`): skyblock AND
/// (tenthAnniversary OR NOT getGoodWorld).
pub(crate) fn spawn_solidifier(flags: SeedFlags) -> bool {
    if !flags.contains(SeedFlags::SKYBLOCK) {
        return false;
    }
    if flags.contains(SeedFlags::TENTH_ANNIVERSARY) {
        return true;
    }
    !flags.contains(SeedFlags::FOR_THE_WORTHY)
}

/// `Skyblock.spawnShimmerPool` (`WorldGen.cs:3115-3129`): the same gate as
/// spawnSolidifier (the C# bodies are identical).
pub(crate) fn spawn_shimmer_pool(flags: SeedFlags) -> bool {
    spawn_solidifier(flags)
}

/// Is `vanilla` pruned by the skyblock gates? The denyAll list always prunes
/// under skyblock; the denySome list prunes when `denySomeGeneration` holds.
/// Called from `pass_disabled_by_special_seeds` (the classic/dualDungeons
/// arms run first; this is the skyblock arm).
pub(crate) fn skyblock_prunes(vanilla: &str, flags: SeedFlags, micro: MicroSeedFlags) -> bool {
    if !flags.contains(SeedFlags::SKYBLOCK) {
        return false;
    }
    if SKYBLOCK_DENY_ALL_PASSES.contains(&vanilla) {
        return true;
    }
    if SKYBLOCK_DENY_SOME_PASSES.contains(&vanilla) {
        return deny_some_generation(flags, micro);
    }
    false
}

/// The `WorldGen.Skyblock` statics (`WorldGen.cs:3063-3281`) — the runtime
/// tile-presence ledger the skyblock shop/spawn conditions read. The C#
/// statics are world-global; the port carries them on the generator's world
/// state. `ScanTiles` (`:3238-3265`) fills `hasTile`/`hasWall`/
/// `currentActiveTiles` from the 40-margin interior; `Calculate`
/// (`:3149-3236`) derives the no* flags + lowTiles from them.
#[derive(Clone, Debug)]
pub(crate) struct SkyblockState {
    pub no_altars: bool,
    pub no_dungeon: bool,
    pub no_temple: bool,
    pub no_hellstone: bool,
    pub no_fossils: bool,
    pub no_life_crystals: bool,
    pub no_hellforge: bool,
    pub low_tiles: bool,
    pub has_tile: Vec<bool>,
    pub has_wall: Vec<bool>,
    pub current_active_tiles: i32,
}

impl Default for SkyblockState {
    /// The static ctor (`WorldGen.cs:3267-3280`): all no* false, lowTiles
    /// false, the has* arrays sized to the ID counts, currentActiveTiles 0.
    fn default() -> Self {
        SkyblockState {
            no_altars: false,
            no_dungeon: false,
            no_temple: false,
            no_hellstone: false,
            no_fossils: false,
            no_life_crystals: false,
            no_hellforge: false,
            low_tiles: false,
            has_tile: vec![false; TILE_COUNT],
            has_wall: vec![false; WALL_COUNT],
            current_active_tiles: 0,
        }
    }
}

impl SkyblockState {
    /// `Skyblock.Calculate` (`WorldGen.cs:3149-3236`) — derive the no* flags
    /// from the scanned has* arrays, then lowTiles from the active-tile
    /// fraction. `is_dungeon_tile`/`is_dungeon_wall` are the `Main.tileDungeon`/
    /// `Main.wallDungeon` sets; `skyblock_world` is `Main.skyblockWorld`.
    /// The `NetMessage.SendData(7)` lowTiles-change sync (`:3232-3235`) is a
    /// runtime net message — no worldgen effect, cited skip.
    pub fn calculate(
        &mut self,
        max_x: i32,
        max_y: i32,
        skyblock_world: bool,
        is_dungeon_tile: impl Fn(u16) -> bool,
        is_dungeon_wall: impl Fn(u16) -> bool,
    ) {
        // `:3151-3157`: reset the no* flags to true.
        self.no_altars = true;
        self.no_dungeon = true;
        self.no_temple = true;
        self.no_hellstone = true;
        self.no_fossils = true;
        self.no_life_crystals = true;
        self.no_hellforge = true;
        // `:3158-3195`: the tile scan unsets the matching no* flag.
        for i in 0..TILE_COUNT {
            if self.has_tile[i] {
                let t = i as u16;
                if is_dungeon_tile(t) {
                    self.no_dungeon = false;
                }
                if t == 26 {
                    self.no_altars = false;
                }
                if t == 226 {
                    self.no_temple = false;
                }
                if t == 58 {
                    self.no_hellstone = false;
                }
                if t == 404 {
                    self.no_fossils = false;
                }
                if t == 12 {
                    self.no_life_crystals = false;
                }
                if t == 77 || t == 133 {
                    self.no_hellforge = false;
                }
            }
        }
        // `:3196-3209`: the wall scan.
        for j in 0..WALL_COUNT {
            if self.has_wall[j] {
                let w = j as u16;
                if is_dungeon_wall(w) {
                    self.no_dungeon = false;
                }
                if w == 87 {
                    self.no_temple = false;
                }
            }
        }
        // `:3210-3218`: lowTiles from the active fraction (< 0.1).
        let num = max_x * max_y;
        let num2 = self.current_active_tiles as f32 / num as f32;
        self.current_active_tiles = 0;
        self.low_tiles = false;
        if num2 < 0.1 && skyblock_world {
            self.low_tiles = true;
        }
        // `:3219-3226`: clear the has* arrays.
        for t in self.has_tile.iter_mut() {
            *t = false;
        }
        for w in self.has_wall.iter_mut() {
            *w = false;
        }
        // `:3227-3231`: noDungeon clears the dungeon anchor (the port's
        // world.dungeon_x/y — the caller applies it; the C# sets Main.dungeonX/Y
        // to -1 here). Returned via the no_dungeon flag the caller reads.
    }

    /// `Skyblock.ScanTiles` (`WorldGen.cs:3238-3265`) — count the active
    /// tiles + record the has* presence over the 40-margin interior.
    pub fn scan_tiles(&mut self, world: &crate::world::World) {
        self.current_active_tiles = 0;
        for t in self.has_tile.iter_mut() {
            *t = false;
        }
        for w in self.has_wall.iter_mut() {
            *w = false;
        }
        let num = world.width - 40;
        let num3 = world.height - 40;
        for k in 40..num {
            for l in 40..num3 {
                if let Some(t) = world.tile(k, l) {
                    if t.active() {
                        self.current_active_tiles += 1;
                        self.has_tile[t.kind as usize] = true;
                    }
                    self.has_wall[t.wall as usize] = true;
                }
            }
        }
    }
}

/// `WorldGen.beachDistance` (`WorldGen.cs:86914`) = 380.
const BEACH_DISTANCE: i32 = 380;

/// `LandmassData` (`WorldBuilding/LandmassData.cs:5-28`) — the skyblock
/// island ledger entry (`GenVars.landmassData`, `WorldGen.cs:23546-23554`).
#[derive(Clone, Copy, Debug)]
pub(crate) struct LandmassData {
    /// `LandmassDataType.SkyblockIsland` (`LandmassDataType.cs:6`) — the only
    /// type this lane writes; the enum's other arms belong to other passes.
    pub position: (f32, f32),
    pub radius_or_half_size: i32,
    pub style: i32,
    pub extra_data: i32,
}

/// `WorldGen.InWorld(int, int, int fluff)` (`WorldGen.cs:9688-9695`).
fn in_world(world: &World, x: i32, y: i32, fluff: i32) -> bool {
    x >= fluff && x < world.width - fluff && y >= fluff && y < world.height - fluff
}

/// `WorldGen.SolidTile(int, int)` (`WorldGen.cs:70111-70132`) — the strict
/// solid test (active + tileSolid + !tileSolidTop + !halfBrick + slope 0 +
/// !inActive); a null/out-of-bounds tile reads as solid.
fn solid_tile(world: &World, x: i32, y: i32) -> bool {
    let Some(t) = world.tile(x, y) else {
        return true;
    };
    t.active()
        && crate::tiledata::TILE_SOLID.has(t.kind)
        && !crate::tiledata::TILE_SOLID_TOP.has(t.kind)
        && !t.half_brick()
        && t.slope() == 0
        && !t.inactive()
}

/// `WorldGen.isTileSurroundedBySolidTiles` (`WorldGen.cs:23657-23670`) — the
/// 3×3 `SolidTile3` test (the loose solid: active + !inactive + tileSolid +
/// !tileSolidTop, no slope/half-brick term).
fn tile_surrounded_by_solid(world: &World, i: i32, j: i32) -> bool {
    for k in i - 1..=i + 1 {
        for l in j - 1..=j + 1 {
            let ok = world.tile(k, l).is_some_and(|t| {
                t.active()
                    && !t.inactive()
                    && crate::tiledata::TILE_SOLID.has(t.kind)
                    && !crate::tiledata::TILE_SOLID_TOP.has(t.kind)
            });
            if !ok {
                return false;
            }
        }
    }
    true
}

/// `Tile.anyShimmer()` (`Tile.cs:428-435`) — liquid > 0 AND the shimmer bit.
fn any_shimmer(world: &World, x: i32, y: i32) -> bool {
    world.tile(x, y).is_some_and(|t| {
        t.liquid > 0 && t.liquid_kind() == crate::tile::LiquidKind::Shimmer
    })
}

/// `WorldGen.IsTileNearby` (`WorldGen.cs:8862-8884`) — any active `kind` in
/// the square radius (the 235 stride-3 special case kept).
fn is_tile_nearby(world: &World, x: i32, y: i32, kind: u16, distance: i32) -> bool {
    let stride = if kind == 235 { 3 } else { 1 };
    let mut i = x - distance;
    while i <= x + distance {
        for j in y - distance..=y + distance {
            if in_world(world, i, j, 0)
                && world.tile(i, j).is_some_and(|t| t.active() && t.kind == kind)
            {
                return true;
            }
        }
        i += stride;
    }
    false
}

/// The island's tile write — `Main.tile[m,n].active(true); type = …` then
/// `SquareTileFrame` (the port's deferred post-pipeline framing seam; the
/// dirt/stone/sand kinds merge through their sets, so the stored frames stay
/// 0 until the framer runs — the mini_biomes/temple convention).
fn set_island_tile(world: &mut World, x: i32, y: i32, kind: u16) {
    if let Some(t) = world.tile_mut(x, y) {
        t.flags |= FLAG_ACTIVE;
        t.kind = kind;
    }
}

/// `WorldGen.SkyblockIsland` (`WorldGen.cs:22818-23556`) — place one skyblock
/// island: the proximity veto, the style-dependent tile/wall palette, the
/// two drifting ellipsoid blobs (the dirt body + the inner retype/liquid
/// core), the stalactite drip, the shimmer pool (style 1), the wall backing,
/// the solidifier/monolith/music-box objects, the cloud puffs and the grass
/// fringe. Returns false when the proximity check vetoes the site.
///
/// `rng` is the pass stream (`WorldGen.genRand`); `flags`/`micro` gate the
/// notTheBees/crimson/drunk palette arms and the spawnSolidifier object;
/// `underworld_layer` is `Main.UnderworldLayer` (`maxTilesY - 200`).
pub(crate) fn skyblock_island(
    world: &mut World,
    rng: &mut super::Rng,
    flags: SeedFlags,
    micro: MicroSeedFlags,
    i: i32,
    j: i32,
    island_style: i32,
    extra_data: i32,
    force_proximity_check: bool,
    landmass: &mut Vec<LandmassData>,
) -> bool {
    let max_x = world.width;
    let max_y = world.height;
    let world_surface = world.world_surface;
    let underworld_layer = max_y - 200;
    // `:22820-22832`: the proximity veto (forced, or any style > 1).
    if force_proximity_check || island_style > 1 {
        for k in i - 200..=i + 200 {
            for l in j - 100..=j + 100 {
                if in_world(world, k, l, 0) && world.tile(k, l).is_some_and(|t| t.active()) {
                    return false;
                }
            }
        }
    }
    // `:22833-22843`: the palette defaults.
    let mut num = rng.range(220, 250) as f64; // the blob budget
    let mut num2 = 4.5f64; // the x ellipse scale
    let mut num3 = 4.0f64; // the y ellipse scale
    let mut num4 = 0u16; // the inner retype kind
    let mut num5 = -1i32; // the liquid type (-1 = no liquid)
    let mut num6 = -1i32; // the grass-fringe kind (-1 = none)
    let mut num7 = 189u16; // the body tile (cloud)
    let mut num8 = 189u16; // the cloud-puff tile A
    let mut num9 = 196u16; // the cloud-puff tile B
    let mut num10 = 73u16; // the backing wall
    let num11 = 56u16; // the shimmer-rim tile
    // `:22844-22856`: the underworld / notTheBees overrides.
    if island_style <= 1 || island_style == 13 {
        if j > underworld_layer {
            num4 = 57;
        } else if flags.contains(SeedFlags::NOT_THE_BEES) && (island_style == 0 || island_style == 13) {
            num4 = 59;
            num7 = 225;
            num10 = 86;
        }
    }
    // `:22857-22872`: the style switch (the budget + ellipse scales).
    match island_style {
        1 => {
            num = rng.range(145, 156) as f64;
            num2 = 2.25;
            num3 = 1.75;
        }
        0 | 13 => {}
        _ => {
            num = rng.range(145, 156) as f64;
            num2 = 2.5;
            num3 = 2.5;
        }
    }
    // `:22873-22960`: the per-style palette.
    if island_style == 2 {
        num4 = if (j as f64) > world_surface { 161 } else { 147 };
        num9 = 460;
    }
    if island_style == 3 {
        num4 = 53;
        num9 = 189;
    }
    if island_style == 4 {
        if j > underworld_layer {
            num4 = 57;
            num6 = 633;
            num9 = 717;
        } else if (j as f64) > world_surface {
            num4 = 1;
        } else {
            num6 = 2;
        }
    }
    if island_style == 5 {
        if (j as f64) > world_surface {
            num4 = 117;
        } else {
            num6 = 109;
        }
    }
    if island_style == 6 {
        if (j as f64) > world_surface {
            num4 = 25;
        } else {
            num6 = 23;
        }
    }
    if island_style == 7 {
        if (j as f64) > world_surface {
            num4 = 203;
        } else {
            num6 = 199;
        }
    }
    if island_style == 8 {
        num4 = 59;
        num6 = 70;
    }
    if island_style == 9 {
        num5 = 0;
        num7 = 196;
        num8 = 196;
        num9 = 189;
    }
    if island_style == 10 {
        num5 = 1;
        num9 = 189;
        num7 = 717;
    }
    if island_style == 11 {
        num5 = 2;
        num9 = 189;
    }
    if island_style == 12 {
        num5 = 3;
        num9 = 189;
    }
    // `:22961-22964`: the deep cloud-puff swap.
    if num9 == 196 && (j as f64) > (world.rock_layer + underworld_layer as f64) / 2.0 {
        num9 = 717;
    }
    // `:22965-22981`: the blob drift setup.
    let mut num12 = num;
    let mut num13 = rng.range(20, 30) as f64;
    let num14 = num;
    let mut num15 = i; // min x
    let mut num16 = i; // max x
    let mut num17 = j; // min y
    let mut num18 = j; // max y
    let mut pos = super::snake_math::Vec2::new(i as f64, j as f64);
    let mut vel = super::snake_math::Vec2::new(rng.range(-20, 21) as f64 * 0.2, 0.0);
    while vel.x > -2.0 && vel.x < 2.0 {
        vel.x = rng.range(-20, 21) as f64 * 0.2;
    }
    vel.y = rng.range(-20, -10) as f64 * 0.02;
    // `:22982-23072`: the first blob (the dirt/cloud body).
    while num > 0.0 && num13 > 0.0 {
        num -= rng.range(0, 4) as f64;
        num13 -= 1.0;
        let mut x_lo = (pos.x - num * 0.5) as i32;
        let mut x_hi = (pos.x + num * 0.5) as i32;
        let mut y_lo = (pos.y - num * 0.5) as i32;
        let mut y_hi = (pos.y + num * 0.5) as i32;
        if x_lo < 0 {
            x_lo = 0;
        }
        if x_hi > max_x {
            x_hi = max_x;
        }
        if y_lo < 0 {
            y_lo = 0;
        }
        if y_hi > max_y {
            y_hi = max_y;
        }
        num12 = num * rng.range(80, 120) as f64 * 0.01;
        let mut surface = pos.y + 1.0;
        for m in x_lo..x_hi {
            if rng.range(0, 2) == 0 {
                surface += rng.range(-1, 2) as f64;
            }
            if surface < pos.y {
                surface = pos.y;
            }
            if surface > pos.y + 2.0 {
                surface = pos.y + 2.0;
            }
            for n in y_lo..y_hi {
                if !in_world(world, m, n, 5) || !((n as f64) > surface + 1.0) {
                    continue;
                }
                let dx = (m as f64 - pos.x).abs();
                let dy = (n as f64 - pos.y).abs() * num2;
                if (dx * dx + dy * dy).sqrt() < num12 * 0.4 {
                    if m < num15 {
                        num15 = m;
                    }
                    if m > num16 {
                        num16 = m;
                    }
                    if n < num17 {
                        num17 = n;
                    }
                    if n > num18 {
                        num18 = n;
                    }
                    set_island_tile(world, m, n, num7);
                }
            }
        }
        pos = pos + vel;
        vel.x += rng.range(-20, 21) as f64 * 0.05;
        if vel.x > 1.0 {
            vel.x = 1.0;
        }
        if vel.x < -1.0 {
            vel.x = -1.0;
        }
        if vel.y > 0.2 {
            vel.y = -0.2;
        }
        if vel.y < -0.2 {
            vel.y = -0.2;
        }
    }
    // `:23073-23116`: the stalactite drip under the body.
    let mut num26 = num15 + rng.range(0, 5);
    while num26 < num16 {
        let mut num27 = num18 + 10;
        while !world.tile(num26, num27).is_some_and(|t| t.active()) {
            num27 -= 1;
            if num27 <= 0 {
                break;
            }
        }
        if num27 <= 0 {
            num26 += rng.range(0, 5);
            continue;
        }
        num27 += rng.range(-3, 4);
        let num28 = rng.range(4, 8);
        let mut num29 = num7;
        if rng.range(0, 4) == 0 {
            num29 = num9;
        }
        for x in num26 - num28..=num26 + num28 {
            for y in num27 - num28..=num27 + num28 {
                if in_world(world, x, y, 5) && y > num17 {
                    let dx = (x - num26).abs() as f64;
                    let dy = (y - num27).abs() as f64 * 2.0;
                    if (dx * dx + dy * dy).sqrt() < (num28 + rng.range(0, 2)) as f64 {
                        set_island_tile(world, x, y, num29);
                    }
                }
            }
        }
        num26 += rng.range(num28, (num28 as f64 * 1.5) as i32);
    }
    // `:23117-23218`: the second blob (the inner retype / liquid core).
    num = num14 * 0.800000011920929;
    num12 = num;
    num13 = rng.range(10, 15) as f64;
    pos.x = i as f64;
    pos.y = num17 as f64;
    vel.x = rng.range(-20, 21) as f64 * 0.2;
    while vel.x > -2.0 && vel.x < 2.0 {
        vel.x = rng.range(-20, 21) as f64 * 0.2;
    }
    vel.y = rng.range(-20, -10) as f64 * 0.02;
    while num > 0.0 && num13 > 0.0 {
        num -= rng.range(0, 4) as f64;
        num13 -= 1.0;
        let mut x_lo = (pos.x - num * 0.5) as i32;
        let mut x_hi = (pos.x + num * 0.5) as i32;
        let y_lo = num17 - 1;
        let mut y_hi = (pos.y + num * 0.5) as i32;
        if x_lo < 0 {
            x_lo = 0;
        }
        if x_hi > max_x {
            x_hi = max_x;
        }
        let y_lo = if y_lo < 0 { 0 } else { y_lo };
        if y_hi > max_y {
            y_hi = max_y;
        }
        num12 = num * rng.range(80, 120) as f64 * 0.01;
        let mut surface = pos.y + 1.0;
        for m in x_lo..x_hi {
            if rng.range(0, 2) == 0 {
                surface += rng.range(-1, 2) as f64;
            }
            if surface < pos.y {
                surface = pos.y;
            }
            if surface > pos.y + 2.0 {
                surface = pos.y + 2.0;
            }
            for n in y_lo..y_hi {
                if !in_world(world, m, n, 5) || !((n as f64) > surface) {
                    continue;
                }
                let dx = (m as f64 - pos.x).abs();
                let dy = (n as f64 - pos.y).abs() * num3;
                if (dx * dx + dy * dy).sqrt() < num12 * 0.4
                    && world.tile(m, n).is_some_and(|t| t.kind == num7)
                {
                    if num5 >= 0 {
                        if let Some(t) = world.tile_mut(m, n) {
                            t.liquid = u8::MAX;
                            t.set_liquid_kind(match num5 {
                                1 => crate::tile::LiquidKind::Lava,
                                2 => crate::tile::LiquidKind::Honey,
                                3 => crate::tile::LiquidKind::Shimmer,
                                _ => crate::tile::LiquidKind::Water,
                            });
                            t.flags &= !FLAG_ACTIVE;
                        }
                    } else if let Some(t) = world.tile_mut(m, n) {
                        t.kind = num4;
                    }
                }
            }
        }
        if num5 >= 0 {
            pos.x += vel.x;
            pos.y -= 0.5;
        } else {
            pos = pos + vel;
            vel.x += rng.range(-20, 21) as f64 * 0.05;
        }
        if vel.x > 1.0 {
            vel.x = 1.0;
        }
        if vel.x < -1.0 {
            vel.x = -1.0;
        }
        if vel.y > 0.2 {
            vel.y = -0.2;
        }
        if vel.y < -0.2 {
            vel.y = -0.2;
        }
    }
    // `:23219-23276`: the liquid-surface / retype tidy sweep.
    let mut num39 = num15 + rng.range(0, 5);
    while num39 < num16 {
        let mut num40 = num18;
        if num5 >= 0 {
            while world.tile(num39, num40).is_some_and(|t| t.liquid == 0) && num39 < num16 {
                num40 -= 1;
                if num40 < num17 {
                    num40 = num18;
                    num39 += rng.range(1, 4);
                }
            }
        } else {
            while world
                .tile(num39, num40)
                .is_some_and(|t| !t.active() || t.kind != num4)
                && num39 < num16
            {
                num40 -= 1;
                if num40 < num17 {
                    num40 = num18;
                    num39 += rng.range(1, 4);
                }
            }
        }
        if num39 >= num16 {
            continue;
        }
        num40 += rng.range(0, 4);
        let num41 = rng.range(2, 8);
        let num42 = num7;
        for x in num39 - num41..=num39 + num41 {
            for y in num40 - num41..=num40 + num41 {
                if !in_world(world, x, y, 5) || y <= num17 {
                    continue;
                }
                let dx = (x - num39).abs() as f64;
                let dy = (y - num40).abs() as f64 * 2.0;
                if (dx * dx + dy * dy).sqrt() < num41 as f64 {
                    if let Some(t) = world.tile_mut(x, y) {
                        t.kind = num42;
                        if num5 >= 0 {
                            t.flags |= FLAG_ACTIVE;
                        }
                    }
                }
            }
        }
        num39 += rng.range(num41, (num41 as f64 * 1.5) as i32);
    }
    // `:23277-23381`: the shimmer pool (style 1 only).
    if island_style == 1 {
        num = rng.range(40, 51) as f64;
        num12 = num;
        num13 = rng.range(3, 5) as f64;
        pos.x = i as f64;
        pos.y = num17 as f64;
        vel.x = rng.range(-20, 21) as f64 * 0.2;
        while vel.x > -2.0 && vel.x < 2.0 {
            vel.x = rng.range(-20, 21) as f64 * 0.2;
        }
        while num > 0.0 && num13 > 0.0 {
            num -= rng.range(0, 4) as f64;
            num13 -= 1.0;
            let mut x_lo = (pos.x - num * 0.5) as i32;
            let mut x_hi = (pos.x + num * 0.5) as i32;
            let y_lo = num17 - 1;
            let mut y_hi = (pos.y + num * 0.5) as i32;
            if x_lo < 0 {
                x_lo = 0;
            }
            if x_hi > max_x {
                x_hi = max_x;
            }
            let y_lo = if y_lo < 0 { 0 } else { y_lo };
            if y_hi > max_y {
                y_hi = max_y;
            }
            num12 = num * rng.range(80, 120) as f64 * 0.01;
            let mut surface = pos.y + 1.0;
            for m in x_lo..x_hi {
                if surface < pos.y {
                    surface = pos.y;
                }
                if surface > pos.y + 2.0 {
                    surface = pos.y + 2.0;
                }
                for n in y_lo..y_hi {
                    if !in_world(world, m, n, 5) {
                        continue;
                    }
                    if (n as f64) > surface {
                        let dx = (m as f64 - pos.x).abs();
                        let dy = (n as f64 - pos.y).abs() * 1.3;
                        let dist = (dx * dx + dy * dy).sqrt();
                        if dist < num12 * 0.4 {
                            if !any_shimmer(world, m, n) {
                                set_island_tile(world, m, n, num11);
                            }
                            if !any_shimmer(world, m, n + 1) {
                                set_island_tile(world, m, n + 1, num11);
                            }
                        }
                        if dist < num12 * 0.3 {
                            if let Some(t) = world.tile_mut(m, n) {
                                t.flags &= !FLAG_ACTIVE;
                                t.liquid = u8::MAX;
                                t.set_liquid_kind(crate::tile::LiquidKind::Shimmer);
                            }
                            if world.tile(m, n - 1).is_some_and(|t| t.active()) {
                                if let Some(t) = world.tile_mut(m, n - 1) {
                                    t.flags &= !FLAG_ACTIVE;
                                    t.liquid = u8::MAX;
                                    t.set_liquid_kind(crate::tile::LiquidKind::Shimmer);
                                }
                            }
                        }
                    }
                    if (n as f64) > surface - 5.0 && (n as f64) <= surface {
                        if let Some(t) = world.tile_mut(m, n) {
                            t.flags &= !FLAG_ACTIVE;
                        }
                    }
                }
            }
            pos.x += vel.x;
            vel.x += rng.range(-20, 21) as f64 * 0.05;
            if vel.x > 1.0 {
                vel.x = 1.0;
            }
            if vel.x < -1.0 {
                vel.x = -1.0;
            }
        }
    }
    // `:23382-23407`: the wall backing (the 3×3 all-active-or-num10 test).
    for x in num15 - 20..=num16 + 20 {
        for y in num17 - 20..=num18 + 20 {
            if !in_world(world, x, y, 5) {
                continue;
            }
            let mut ok = true;
            for dx in x - 1..=x + 1 {
                for dy in y - 1..=y + 1 {
                    if in_world(world, dx, dy, 0) {
                        let t = world.tile(dx, dy);
                        if t.is_some_and(|t| !t.active() || (t.wall > 0 && t.wall != num10)) {
                            ok = false;
                        }
                    }
                }
            }
            if ok {
                if let Some(t) = world.tile_mut(x, y) {
                    t.wall = num10;
                }
            }
        }
    }
    // `:23408-23440`: the solidifier (style 0/13, spawnSolidifier gate).
    if (island_style == 0 || island_style == 13) && spawn_solidifier(flags) {
        let mut placing = true;
        let mut budget = 10000;
        let mut radius = 20;
        while placing && budget > 0 {
            budget -= 1;
            if budget % 20 == 0 {
                radius += 1;
            }
            let x = super::snake_math::clamp_i(rng.range(i - radius, i + radius), num15, num16);
            let mut y = 6;
            if !in_world(world, x, y, 5) {
                continue;
            }
            while !world.tile(x, y).is_some_and(|t| t.active()) && y < max_y - 10 {
                y += 1;
            }
            if world.tile(x, y).is_some_and(|t| t.kind == num4) {
                y -= 1;
                place_object(world, rng, x, y, 220, 0);
                if world.tile(x, y - 1).is_some_and(|t| t.kind == 220) {
                    placing = false;
                }
            }
        }
    }
    // `:23441-23497`: the shimmer monolith + music box (style 1).
    if island_style == 1 {
        for pass in 0..2 {
            let mut placing = true;
            let mut budget = 10000;
            let mut radius = 10;
            while placing && budget > 0 {
                budget -= 1;
                if budget % 50 == 0 {
                    radius += 1;
                }
                let x = super::snake_math::clamp_i(rng.range(i - radius, i + radius), num15, num16);
                let mut y = num17 - 30;
                if !in_world(world, x, y, 5) {
                    continue;
                }
                while !world.tile(x, y).is_some_and(|t| t.active()) && y < max_y - 10 {
                    y += 1;
                }
                if !world.tile(x, y).is_some_and(|t| t.kind == num4 || t.kind == num11) {
                    continue;
                }
                y -= 1;
                if world.tile(x, y).is_some_and(|t| t.liquid != 0)
                    || world.tile(x + 2, y + 1).is_some_and(|t| t.liquid != 0)
                    || world.tile(x - 1, y + 1).is_some_and(|t| t.liquid != 0)
                {
                    continue;
                }
                match pass {
                    0 => {
                        place_object(world, rng, x, y, 658, 0);
                        if world.tile(x, y).is_some_and(|t| t.kind == 658) {
                            switch_monolith(world, x, y);
                            placing = false;
                        }
                    }
                    _ => {
                        if budget <= 2000 || !is_tile_nearby(world, x, y, 658, 10) {
                            place_object(world, rng, x, y, 139, 87);
                            if world.tile(x, y).is_some_and(|t| t.kind == 139) {
                                switch_mb(world, x, y);
                                placing = false;
                            }
                        }
                    }
                }
            }
        }
    }
    // `:23498-23523`: the cloud puffs above the island.
    let num67 = rng.range(0, 4);
    for _ in 0..=num67 {
        let x = rng.range(num15 - 5, num16 + 5);
        let y = num17 - rng.range(20, 40);
        let r = rng.range(4, 8);
        let mut kind = num8;
        if rng.range(0, 2) == 0 {
            kind = num9;
        }
        for px in x - r..=x + r {
            for py in y - r..=y + r {
                let dx = (px - x).abs() as f64;
                let dy = (py - y).abs() as f64 * 2.0;
                if (dx * dx + dy * dy).sqrt() < (r + rng.range(-1, 2)) as f64
                    && in_world(world, px, py, 0)
                {
                    set_island_tile(world, px, py, kind);
                }
            }
        }
    }
    // `:23524-23545`: the grass fringe (the num6 > 0 styles).
    if num6 > 0 {
        let mut target = 0u16;
        if num6 == 60 || num6 == 70 {
            target = 59;
        }
        if num6 == 633 {
            target = 57;
        }
        for x in num15 - 20..=num16 + 20 {
            for y in j - 50..=j + 50 {
                if in_world(world, x, y, 5)
                    && world.tile(x, y).is_some_and(|t| t.active() && t.kind == target)
                    && !tile_surrounded_by_solid(world, x, y)
                {
                    if let Some(t) = world.tile_mut(x, y) {
                        t.kind = num6 as u16;
                    }
                }
            }
        }
    }
    // `:23546-23554`: the landmass ledger entry.
    landmass.push(LandmassData {
        position: (i as f32, j as f32),
        radius_or_half_size: 10,
        style: island_style,
        extra_data,
    });
    true
}

/// `WorldGen.PlaceTile`'s object arm for the skyblock objects — the
/// solidifier 220 (`Place3x3`, `WorldGen.cs:59967-59970`), the shimmer
/// monolith 658 (`Place2xX`, `:60093-60098`) and the music box 139
/// (`:59923`). The port routes through the `tileobject` registry (the
/// faithful `TileObject.CanPlace`/`Place` seam, the temple buried-chest
/// convention); `style` is the C# `style` argument. The placement RNG is the
/// world's `place_rng` (the C# `Main.rand` at the call site — the objects'
/// random-style draws ride it; the pass stream `rng` is untouched, matching
/// the C# where `PlaceTile` draws on `Main.rand`, not `genRand`).
fn place_object(world: &mut World, _rng: &mut super::Rng, x: i32, y: i32, kind: u16, style: i32) {
    let Some(def) = crate::tileobject::def_for_tile(kind) else {
        return;
    };
    let mut rand = std::mem::take(&mut world.place_rng);
    let instance = crate::tileobject::can_place(
        world,
        x,
        y,
        def,
        style,
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

/// `WorldGen.SwitchMonolith` (`WorldGen.cs:51004-51148`) — toggle the
/// monolith family's frameY band (the 658 shimmer monolith's +54 mod 162,
/// the 410/480/509/657/720/721/725/733 ±54/56 flips). The `Wiring.SkipWire`
/// + `NetMessage.SendTileSquare` tails are runtime sync — no worldgen effect,
/// cited skip.
fn switch_monolith(world: &mut World, i: i32, j: i32) {
    let kind = world.tile(i, j).map(|t| t.kind).unwrap_or(0);
    let mut num = 2;
    let num2 = 3;
    if kind == 733 {
        num = 3;
    }
    let mut num5 = world.tile_frame_x(i, j) / 18;
    while num5 >= num {
        num5 -= num;
    }
    let mut num6 = world.tile_frame_y(i, j) / 18;
    while num6 >= num2 {
        num6 -= num2;
    }
    let (x0, y0) = (i - num5, j - num6);
    for x in x0..x0 + num {
        for y in y0..y0 + num2 {
            let Some(t) = world.tile(x, y).copied() else {
                continue;
            };
            if !t.active() {
                continue;
            }
            let f = world.tile_frame_y(x, y);
            match t.kind {
                410 => {
                    world.set_tile_frame_y(x, y, if f < 56 { f + 56 } else { f - 56 });
                }
                658 => {
                    let mut nf = f + 54;
                    if nf >= 162 {
                        nf -= 162;
                    }
                    world.set_tile_frame_y(x, y, nf);
                }
                480 | 509 | 657 | 720 | 721 | 725 | 733 => {
                    world.set_tile_frame_y(x, y, if f < 54 { f + 54 } else { f - 54 });
                }
                _ => {}
            }
        }
    }
}

/// `WorldGen.SwitchMB` (`WorldGen.cs:50958-51002`) — toggle the music box
/// 139/35's frameX ±36 band. The `Wiring.SkipWire` + `NetMessage` tails are
/// runtime sync — cited skip.
fn switch_mb(world: &mut World, i: i32, j: i32) {
    let mut num3 = world.tile_frame_y(i, j) / 18;
    while num3 >= 2 {
        num3 -= 2;
    }
    let mut num4 = world.tile_frame_x(i, j) / 18;
    if num4 >= 2 {
        num4 -= 2;
    }
    let (x0, y0) = (i - num4, j - num3);
    for x in x0..x0 + 2 {
        for y in y0..y0 + 2 {
            if world.tile(x, y).is_some_and(|t| t.active() && (t.kind == 139 || t.kind == 35)) {
                let f = world.tile_frame_x(x, y);
                world.set_tile_frame_x(x, y, if f < 36 { f + 36 } else { f - 36 });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The denyAll gate prunes every listed pass under skyblock and nothing
    /// on a classic world (`WorldGen.cs:3131`).
    #[test]
    fn w4_f24_deny_all_prunes_under_skyblock() {
        let sky = SeedFlags::from(SeedFlags::SKYBLOCK);
        let classic = SeedFlags::none();
        let micro = MicroSeedFlags::none();
        for name in SKYBLOCK_DENY_ALL_PASSES {
            assert!(skyblock_prunes(name, sky, micro), "{name} should prune");
            assert!(!skyblock_prunes(name, classic, micro), "{name} classic");
        }
        assert_eq!(SKYBLOCK_DENY_ALL_PASSES.len(), 75);
        // Terrain is a class pass — never pruned by the skyblock arm.
        assert!(!skyblock_prunes("Terrain", sky, micro));
    }

    /// denySomeGeneration: the eight micro-seeds each open the denySome
    /// passes; extraLivingTrees alone opens them; none of them prunes
    /// (`WorldGen.cs:3133-3147`).
    #[test]
    fn w4_f24_deny_some_reads_the_eight_seeds() {
        let sky = SeedFlags::from(SeedFlags::SKYBLOCK);
        // No micro-seed: denySome holds -> the denySome passes prune.
        assert!(deny_some_generation(sky, MicroSeedFlags::none()));
        assert!(skyblock_prunes("Flowers", sky, MicroSeedFlags::none()));
        // Each of the eight opens them.
        for bit in [
            MicroSeedFlags::WORLD_IS_FROZEN,
            MicroSeedFlags::SURFACE_IS_DESERT,
            MicroSeedFlags::SURFACE_IS_MUSHROOMS,
            MicroSeedFlags::WORLD_IS_INFECTED,
            MicroSeedFlags::HALLOW_ON_THE_SURFACE,
            MicroSeedFlags::NO_INFECTION,
            MicroSeedFlags::EXTRA_FLOATING_ISLANDS,
            MicroSeedFlags::EXTRA_LIQUID,
        ] {
            let m = MicroSeedFlags::from(bit);
            assert!(!deny_some_generation(sky, m));
            assert!(!skyblock_prunes("Flowers", sky, m));
        }
        // extraLivingTrees alone (none of the eight) also opens them.
        let trees = MicroSeedFlags::from(MicroSeedFlags::EXTRA_LIVING_TREES);
        assert!(!deny_some_generation(sky, trees));
        // A classic world never prunes via denySome.
        assert!(!deny_some_generation(SeedFlags::none(), MicroSeedFlags::none()));
        assert_eq!(SKYBLOCK_DENY_SOME_PASSES.len(), 22);
    }

    /// spawnSolidifier/spawnShimmerPool: tenthAnniversary forces true,
    /// getGoodWorld forces false, else true (`WorldGen.cs:3099-3129`).
    #[test]
    fn w4_f24_spawn_solidifier_gate() {
        let sky = SeedFlags::from(SeedFlags::SKYBLOCK);
        assert!(spawn_solidifier(sky));
        assert!(spawn_shimmer_pool(sky));
        let good = SeedFlags::from(SeedFlags::SKYBLOCK | SeedFlags::FOR_THE_WORTHY);
        assert!(!spawn_solidifier(good));
        let tenth = SeedFlags::from(SeedFlags::SKYBLOCK | SeedFlags::TENTH_ANNIVERSARY | SeedFlags::FOR_THE_WORTHY);
        assert!(spawn_solidifier(tenth));
        assert!(!spawn_solidifier(SeedFlags::none()));
        // denyFloatingIslands: skyblock without extraFloatingIslands.
        assert!(deny_floating_islands(sky, MicroSeedFlags::none()));
        assert!(!deny_floating_islands(sky, MicroSeedFlags::from(MicroSeedFlags::EXTRA_FLOATING_ISLANDS)));
    }

    /// Calculate derives the no* flags from the scanned presence
    /// (`WorldGen.cs:3149-3236`); lowTiles from the < 0.1 active fraction.
    #[test]
    fn w4_f24_calculate_derives_no_flags() {
        let mut s = SkyblockState::default();
        // An empty world: every no* flag stays true, lowTiles true (0 < 0.1).
        s.calculate(100, 100, true, |_| false, |_| false);
        assert!(s.no_dungeon && s.no_altars && s.no_temple && s.no_hellstone);
        assert!(s.no_fossils && s.no_life_crystals && s.no_hellforge);
        assert!(s.low_tiles);
        // Presence unsets the matching flag.
        let mut s2 = SkyblockState::default();
        s2.has_tile[26] = true; // altar
        s2.has_tile[58] = true; // hellstone
        s2.has_tile[12] = true; // life crystal
        s2.has_wall[87] = true; // temple wall
        s2.current_active_tiles = 5000; // 5000/10000 = 0.5 >= 0.1
        s2.calculate(100, 100, true, |_| false, |_| false);
        assert!(!s2.no_altars);
        assert!(!s2.no_hellstone);
        assert!(!s2.no_life_crystals);
        assert!(!s2.no_temple);
        assert!(!s2.low_tiles);
        // The has* arrays clear after Calculate.
        assert!(!s2.has_tile[26]);
    }

    /// `SkyblockIsland` places a deterministic island (a == b replay) and
    /// records the landmass entry; the proximity veto refuses a site next to
    /// an active tile (`WorldGen.cs:22818-23556`). Written NOT run.
    #[test]
    fn w4_f24_skyblock_island_is_deterministic_and_vetoes() {
        use crate::worldgen::GenOptions;
        let build = || {
            let mut w = crate::worldgen::generate(&GenOptions {
                width: 512,
                height: 512,
                seed: 42,
                ..Default::default()
            });
            // Clear a region so the island has room (the classic world is
            // full; the skyblock generator clears everything first).
            for y in 100..400 {
                for x in 100..400 {
                    if let Some(t) = w.tile_mut(x, y) {
                        *t = Default::default();
                    }
                }
            }
            let mut rng = super::super::Rng::from_seed(7);
            let mut landmass = Vec::new();
            let ok = skyblock_island(
                &mut w,
                &mut rng,
                SeedFlags::none(),
                MicroSeedFlags::none(),
                250,
                250,
                0,
                -1,
                false,
                &mut landmass,
            );
            (w, ok, landmass)
        };
        let (wa, oka, la) = build();
        let (wb, okb, lb) = build();
        assert!(oka && okb);
        assert_eq!(la.len(), 1);
        assert_eq!(lb.len(), 1);
        assert_eq!(la[0].style, 0);
        assert_eq!(la[0].radius_or_half_size, 10);
        // a == b: the island tiles replay identically.
        for y in 100..400 {
            for x in 100..400 {
                let ta = wa.tile(x, y).unwrap();
                let tb = wb.tile(x, y).unwrap();
                assert_eq!(
                    (ta.kind, ta.wall, ta.flags, ta.liquid),
                    (tb.kind, tb.wall, tb.flags, tb.liquid)
                );
            }
        }
        // The island placed some cloud tiles (kind 189).
        let clouds = (100..400)
            .flat_map(|y| (100..400).map(move |x| (x, y)))
            .filter(|&(x, y)| wa.tile(x, y).is_some_and(|t| t.active() && t.kind == 189))
            .count();
        assert!(clouds > 0);
        // The proximity veto: a style-2 island next to an active tile refuses.
        let mut w = crate::worldgen::generate(&GenOptions {
            width: 512,
            height: 512,
            seed: 42,
            ..Default::default()
        });
        let mut rng = super::super::Rng::from_seed(7);
        let mut landmass = Vec::new();
        // The classic world is full of active tiles -> the veto fires.
        let ok = skyblock_island(
            &mut w, &mut rng, SeedFlags::none(), MicroSeedFlags::none(),
            250, 250, 2, -1, false, &mut landmass,
        );
        assert!(!ok);
        assert!(landmass.is_empty());
    }
}
