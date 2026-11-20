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
use crate::tiledata::{TILE_COUNT, WALL_COUNT};

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
}
