//! The bestiary entry catalog — the port's NPC family as the
//! `BestiaryDatabase`'s rows (`BestiaryDatabaseNPCsPopulator.cs` builds one
//! entry per NPC net id; the port has the same twelve).

use tr_core::bestiary::{KillTracker, UnlockState, WorldBestiary, get_unlock_state_by_kill_count};

use crate::npc::{NpcKind, SlimeVariant};

/// One entry: the credit id (`ContentSamples.NpcBestiaryCreditIdsByNpcNetIds`
/// — `NPCID.Search.GetName(netId)`, `ContentSamples.cs:862-873`), the net id
/// the banner chain needs and the port-side kind/variant pair that renders
/// the portrait and reads the stats.
pub struct BestiaryEntryDef {
    pub credit_id: &'static str,
    pub net_id: i32,
    pub kind: NpcKind,
    pub variant: Option<SlimeVariant>,
    /// The boss providers pass `quickUnlock: true`
    /// (`BestiaryDatabaseNPCsPopulator.cs:679`'s shape) — one kill reveals
    /// tier 4 outright; `kills_needed_for` returns 0 for them.
    pub quick_unlock: bool,
    /// `Lang.GetNPCNameValue` — the `SlimeVariant::name` table
    /// (`Lang.cs:520-533`) plus the kind names.
    pub name: &'static str,
}

/// The `BestiaryDatabase.Entries` order — the populator walks the NPC ids
/// ascending, the slime variants ride their negative net ids right after
/// BlueSlime's 1; the list stays in that (net-id) order for the grid's
/// display indexes.
pub const ENTRIES: &[BestiaryEntryDef] = &[
    BestiaryEntryDef {
        credit_id: "BlueSlime",
        net_id: crate::loot_db::BLUE_SLIME,
        kind: NpcKind::Slime,
        variant: Some(SlimeVariant::Blue),
        quick_unlock: false,
        name: "Blue Slime",
    },
    BestiaryEntryDef {
        credit_id: "GreenSlime",
        net_id: crate::loot_db::GREEN_SLIME,
        kind: NpcKind::Slime,
        variant: Some(SlimeVariant::Green),
        quick_unlock: false,
        name: "Green Slime",
    },
    BestiaryEntryDef {
        credit_id: "Pinky",
        net_id: crate::loot_db::PINKY,
        kind: NpcKind::Slime,
        variant: Some(SlimeVariant::Pinky),
        quick_unlock: false,
        name: "Pinky",
    },
    BestiaryEntryDef {
        credit_id: "BabySlime",
        net_id: crate::loot_db::BABY_SLIME,
        kind: NpcKind::Slime,
        variant: Some(SlimeVariant::Baby),
        quick_unlock: false,
        name: "Baby Slime",
    },
    BestiaryEntryDef {
        credit_id: "BlackSlime",
        net_id: crate::loot_db::BLACK_SLIME,
        kind: NpcKind::Slime,
        variant: Some(SlimeVariant::Black),
        quick_unlock: false,
        name: "Black Slime",
    },
    BestiaryEntryDef {
        credit_id: "PurpleSlime",
        net_id: crate::loot_db::PURPLE_SLIME,
        kind: NpcKind::Slime,
        variant: Some(SlimeVariant::Purple),
        quick_unlock: false,
        name: "Purple Slime",
    },
    BestiaryEntryDef {
        credit_id: "RedSlime",
        net_id: crate::loot_db::RED_SLIME,
        kind: NpcKind::Slime,
        variant: Some(SlimeVariant::Red),
        quick_unlock: false,
        name: "Red Slime",
    },
    BestiaryEntryDef {
        credit_id: "YellowSlime",
        net_id: crate::loot_db::YELLOW_SLIME,
        kind: NpcKind::Slime,
        variant: Some(SlimeVariant::Yellow),
        quick_unlock: false,
        name: "Yellow Slime",
    },
    BestiaryEntryDef {
        credit_id: "DemonEye",
        net_id: 2,
        kind: NpcKind::DemonEye,
        variant: None,
        quick_unlock: false,
        name: "Demon Eye",
    },
    BestiaryEntryDef {
        credit_id: "Zombie",
        net_id: 3,
        kind: NpcKind::Zombie,
        variant: None,
        quick_unlock: false,
        name: "Zombie",
    },
    // S1-D-3ff wave A — `BestiaryEntry.Enemy(16)` (the automated
    // populator, `BestiaryDatabaseNPCsPopulator.cs:851-868`); the
    // name rides `Lang.GetNPCName` (en-US NPCs.json).
    BestiaryEntryDef {
        credit_id: "MotherSlime",
        net_id: 16,
        kind: NpcKind::MotherSlime,
        variant: None,
        quick_unlock: false,
        name: "Mother Slime",
    },
    BestiaryEntryDef {
        credit_id: "KingSlime",
        net_id: 50,
        kind: NpcKind::KingSlime,
        variant: None,
        quick_unlock: true,
        name: "King Slime",
    },
    BestiaryEntryDef {
        credit_id: "EyeofCthulhu",
        net_id: 4,
        kind: NpcKind::EyeOfCthulhu,
        variant: None,
        quick_unlock: true,
        name: "Eye of Cthulhu",
    },
    // S1-D-3ff wave A — `BestiaryEntry.Enemy(52)` (the automated
    // populator, `BestiaryDatabaseNPCsPopulator.cs:851-868`); the
    // name rides `Lang.GetNPCName` (en-US NPCs.json).
    BestiaryEntryDef {
        credit_id: "DoctorBones",
        net_id: 52,
        kind: NpcKind::DoctorBones,
        variant: None,
        quick_unlock: false,
        name: "Doctor Bones",
    },
    // S1-D-3ff wave A — `BestiaryEntry.Enemy(53)` (the automated
    // populator, `BestiaryDatabaseNPCsPopulator.cs:851-868`); the
    // name rides `Lang.GetNPCName` (en-US NPCs.json).
    BestiaryEntryDef {
        credit_id: "TheGroom",
        net_id: 53,
        kind: NpcKind::TheGroom,
        variant: None,
        quick_unlock: false,
        name: "The Groom",
    },
    // S1-D-3ff wave A — `BestiaryEntry.Enemy(59)` (the automated
    // populator, `BestiaryDatabaseNPCsPopulator.cs:851-868`); the
    // name rides `Lang.GetNPCName` (en-US NPCs.json).
    BestiaryEntryDef {
        credit_id: "LavaSlime",
        net_id: 59,
        kind: NpcKind::LavaSlime,
        variant: None,
        quick_unlock: false,
        name: "Lava Slime",
    },
    // S1-D-3ff wave A — `BestiaryEntry.Enemy(71)` (the automated
    // populator, `BestiaryDatabaseNPCsPopulator.cs:851-868`); the
    // name rides `Lang.GetNPCName` (en-US NPCs.json).
    BestiaryEntryDef {
        credit_id: "DungeonSlime",
        net_id: 71,
        kind: NpcKind::DungeonSlime,
        variant: None,
        quick_unlock: false,
        name: "Dungeon Slime",
    },
    // S1-D-3ff wave A — `BestiaryEntry.Enemy(81)` (the automated
    // populator, `BestiaryDatabaseNPCsPopulator.cs:851-868`); the
    // name rides `Lang.GetNPCName` (en-US NPCs.json).
    BestiaryEntryDef {
        credit_id: "CorruptSlime",
        net_id: 81,
        kind: NpcKind::CorruptSlime,
        variant: None,
        quick_unlock: false,
        name: "Corrupt Slime",
    },
    BestiaryEntryDef {
        credit_id: "TheHungry",
        net_id: 115,
        kind: NpcKind::Hungry,
        variant: None,
        quick_unlock: false,
        name: "The Hungry",
    },
    BestiaryEntryDef {
        credit_id: "LeechHead",
        net_id: 117,
        kind: NpcKind::LeechHead,
        variant: None,
        quick_unlock: false,
        name: "Leech",
    },
    BestiaryEntryDef {
        credit_id: "ServantofCthulhu",
        net_id: 5,
        kind: NpcKind::ServantOfCthulhu,
        variant: None,
        quick_unlock: false,
        name: "Servant of Cthulhu",
    },
    // S1-D-3ff wave A — `BestiaryEntry.Enemy(132)` (the automated
    // populator, `BestiaryDatabaseNPCsPopulator.cs:851-868`); the
    // name rides `Lang.GetNPCName` (en-US NPCs.json).
    BestiaryEntryDef {
        credit_id: "BaldZombie",
        net_id: 132,
        kind: NpcKind::BaldZombie,
        variant: None,
        quick_unlock: false,
        name: "Zombie",
    },
    BestiaryEntryDef {
        credit_id: "WanderingEye",
        net_id: 133,
        kind: NpcKind::WanderingEye,
        variant: None,
        quick_unlock: false,
        name: "Wandering Eye",
    },
    // The town NPCs unlock by chat/proximity (`TownNPCUICollectionInfoProvider`)
    // — the quick-unlock shape stands in for it.
    BestiaryEntryDef {
        credit_id: "Guide",
        net_id: 22,
        kind: NpcKind::Guide,
        variant: None,
        quick_unlock: false,
        name: "Guide",
    },
    BestiaryEntryDef {
        credit_id: "Skeleton",
        net_id: 21,
        kind: NpcKind::Skeleton,
        variant: None,
        quick_unlock: false,
        name: "Skeleton",
    },
    BestiaryEntryDef {
        credit_id: "UndeadMiner",
        net_id: 44,
        kind: NpcKind::UndeadMiner,
        variant: None,
        quick_unlock: false,
        name: "Undead Miner",
    },
    BestiaryEntryDef {
        credit_id: "CaveBat",
        net_id: 49,
        kind: NpcKind::CaveBat,
        variant: None,
        quick_unlock: false,
        name: "Cave Bat",
    },
    BestiaryEntryDef {
        credit_id: "JungleBat",
        net_id: 51,
        kind: NpcKind::JungleBat,
        variant: None,
        quick_unlock: false,
        name: "Jungle Bat",
    },
    BestiaryEntryDef {
        credit_id: "Hellbat",
        net_id: 60,
        kind: NpcKind::Hellbat,
        variant: None,
        quick_unlock: false,
        name: "Hellbat",
    },
    BestiaryEntryDef {
        credit_id: "VoodooDemon",
        net_id: 66,
        kind: NpcKind::VoodooDemon,
        variant: None,
        quick_unlock: false,
        name: "Voodoo Demon",
    },
    BestiaryEntryDef {
        credit_id: "EaterofSouls",
        net_id: 6,
        kind: NpcKind::EaterOfSouls,
        variant: None,
        quick_unlock: false,
        name: "Eater of Souls",
    },
    BestiaryEntryDef {
        credit_id: "Hornet",
        net_id: 42,
        kind: NpcKind::Hornet,
        variant: None,
        quick_unlock: false,
        name: "Hornet",
    },
    // The Queen Bee's volley minions (S1-D-3 W-3) — the automated
    // `BestiaryEntry.Enemy(210/211)` populator
    // (`BestiaryDatabaseNPCsPopulator.cs:851-868`); the credit ids ride
    // `ContentSamples.NpcBestiaryCreditIdsByNpcNetIds` (the persistent
    // names). Both display "Bee" (`en-US NPCs.json`'s Bee/BeeSmall rows).
    BestiaryEntryDef {
        credit_id: "Bee",
        net_id: 210,
        kind: NpcKind::Bee,
        variant: None,
        quick_unlock: false,
        name: "Bee",
    },
    BestiaryEntryDef {
        credit_id: "BeeSmall",
        net_id: 211,
        kind: NpcKind::BeeSmall,
        variant: None,
        quick_unlock: false,
        name: "Bee",
    },
    BestiaryEntryDef {
        credit_id: "MeteorHead",
        net_id: 23,
        kind: NpcKind::MeteorHead,
        variant: None,
        quick_unlock: false,
        name: "Meteor Head",
    },
    BestiaryEntryDef {
        credit_id: "GoblinSorcerer",
        net_id: 29,
        kind: NpcKind::GoblinSorcerer,
        variant: None,
        quick_unlock: false,
        name: "Goblin Sorcerer",
    },
    BestiaryEntryDef {
        credit_id: "Tim",
        net_id: 45,
        kind: NpcKind::Tim,
        variant: None,
        quick_unlock: false,
        name: "Tim",
    },
    BestiaryEntryDef {
        credit_id: "DarkCaster",
        net_id: 32,
        kind: NpcKind::DarkCaster,
        variant: None,
        quick_unlock: false,
        name: "Dark Caster",
    },
    BestiaryEntryDef {
        credit_id: "FireImp",
        net_id: 24,
        kind: NpcKind::FireImp,
        variant: None,
        quick_unlock: false,
        name: "Fire Imp",
    },
    // S1-D-3ff wave A — `BestiaryEntry.Enemy(138)` (the automated
    // populator, `BestiaryDatabaseNPCsPopulator.cs:851-868`); the
    // name rides `Lang.GetNPCName` (en-US NPCs.json).
    BestiaryEntryDef {
        credit_id: "IlluminantSlime",
        net_id: 138,
        kind: NpcKind::IlluminantSlime,
        variant: None,
        quick_unlock: false,
        name: "Illuminant Slime",
    },
    // S1-D-3ff wave A — `BestiaryEntry.Enemy(147)` (the automated
    // populator, `BestiaryDatabaseNPCsPopulator.cs:851-868`); the
    // name rides `Lang.GetNPCName` (en-US NPCs.json).
    BestiaryEntryDef {
        credit_id: "IceSlime",
        net_id: 147,
        kind: NpcKind::IceSlime,
        variant: None,
        quick_unlock: false,
        name: "Ice Slime",
    },
    // S1-D-3ff wave A — `BestiaryEntry.Enemy(161)` (the automated
    // populator, `BestiaryDatabaseNPCsPopulator.cs:851-868`); the
    // name rides `Lang.GetNPCName` (en-US NPCs.json).
    BestiaryEntryDef {
        credit_id: "ZombieEskimo",
        net_id: 161,
        kind: NpcKind::ZombieEskimo,
        variant: None,
        quick_unlock: false,
        name: "Frozen Zombie",
    },
    // S1-D-3ff wave A — `BestiaryEntry.Enemy(183)` (the automated
    // populator, `BestiaryDatabaseNPCsPopulator.cs:851-868`); the
    // name rides `Lang.GetNPCName` (en-US NPCs.json).
    BestiaryEntryDef {
        credit_id: "Crimslime",
        net_id: 183,
        kind: NpcKind::Crimslime,
        variant: None,
        quick_unlock: false,
        name: "Crimslime",
    },
    // S1-D-3ff wave A — `BestiaryEntry.Enemy(184)` (the automated
    // populator, `BestiaryDatabaseNPCsPopulator.cs:851-868`); the
    // name rides `Lang.GetNPCName` (en-US NPCs.json).
    BestiaryEntryDef {
        credit_id: "SpikedIceSlime",
        net_id: 184,
        kind: NpcKind::SpikedIceSlime,
        variant: None,
        quick_unlock: false,
        name: "Spiked Ice Slime",
    },
    // S1-D-3ff wave A — `BestiaryEntry.Enemy(186)` (the automated
    // populator, `BestiaryDatabaseNPCsPopulator.cs:851-868`); the
    // name rides `Lang.GetNPCName` (en-US NPCs.json).
    BestiaryEntryDef {
        credit_id: "PincushionZombie",
        net_id: 186,
        kind: NpcKind::PincushionZombie,
        variant: None,
        quick_unlock: false,
        name: "Zombie",
    },
    // S1-D-3ff wave A — `BestiaryEntry.Enemy(187)` (the automated
    // populator, `BestiaryDatabaseNPCsPopulator.cs:851-868`); the
    // name rides `Lang.GetNPCName` (en-US NPCs.json).
    BestiaryEntryDef {
        credit_id: "SlimedZombie",
        net_id: 187,
        kind: NpcKind::SlimedZombie,
        variant: None,
        quick_unlock: false,
        name: "Zombie",
    },
    // S1-D-3ff wave A — `BestiaryEntry.Enemy(188)` (the automated
    // populator, `BestiaryDatabaseNPCsPopulator.cs:851-868`); the
    // name rides `Lang.GetNPCName` (en-US NPCs.json).
    BestiaryEntryDef {
        credit_id: "SwampZombie",
        net_id: 188,
        kind: NpcKind::SwampZombie,
        variant: None,
        quick_unlock: false,
        name: "Zombie",
    },
    // S1-D-3ff wave A — `BestiaryEntry.Enemy(189)` (the automated
    // populator, `BestiaryDatabaseNPCsPopulator.cs:851-868`); the
    // name rides `Lang.GetNPCName` (en-US NPCs.json).
    BestiaryEntryDef {
        credit_id: "TwiggyZombie",
        net_id: 189,
        kind: NpcKind::TwiggyZombie,
        variant: None,
        quick_unlock: false,
        name: "Zombie",
    },
    // S1-D-3ff wave A — `BestiaryEntry.Enemy(190)` (the automated
    // populator, `BestiaryDatabaseNPCsPopulator.cs:851-868`); the
    // name rides `Lang.GetNPCName` (en-US NPCs.json).
    BestiaryEntryDef {
        credit_id: "CataractEye",
        net_id: 190,
        kind: NpcKind::CataractEye,
        variant: None,
        quick_unlock: false,
        name: "Demon Eye",
    },
    // S1-D-3ff wave A — `BestiaryEntry.Enemy(191)` (the automated
    // populator, `BestiaryDatabaseNPCsPopulator.cs:851-868`); the
    // name rides `Lang.GetNPCName` (en-US NPCs.json).
    BestiaryEntryDef {
        credit_id: "SleepyEye",
        net_id: 191,
        kind: NpcKind::SleepyEye,
        variant: None,
        quick_unlock: false,
        name: "Demon Eye",
    },
    // S1-D-3ff wave A — `BestiaryEntry.Enemy(192)` (the automated
    // populator, `BestiaryDatabaseNPCsPopulator.cs:851-868`); the
    // name rides `Lang.GetNPCName` (en-US NPCs.json).
    BestiaryEntryDef {
        credit_id: "DialatedEye",
        net_id: 192,
        kind: NpcKind::DialatedEye,
        variant: None,
        quick_unlock: false,
        name: "Demon Eye",
    },
    // S1-D-3ff wave A — `BestiaryEntry.Enemy(193)` (the automated
    // populator, `BestiaryDatabaseNPCsPopulator.cs:851-868`); the
    // name rides `Lang.GetNPCName` (en-US NPCs.json).
    BestiaryEntryDef {
        credit_id: "GreenEye",
        net_id: 193,
        kind: NpcKind::GreenEye,
        variant: None,
        quick_unlock: false,
        name: "Demon Eye",
    },
    // S1-D-3ff wave A — `BestiaryEntry.Enemy(194)` (the automated
    // populator, `BestiaryDatabaseNPCsPopulator.cs:851-868`); the
    // name rides `Lang.GetNPCName` (en-US NPCs.json).
    BestiaryEntryDef {
        credit_id: "PurpleEye",
        net_id: 194,
        kind: NpcKind::PurpleEye,
        variant: None,
        quick_unlock: false,
        name: "Demon Eye",
    },
    // S1-D-3ff wave A — `BestiaryEntry.Enemy(200)` (the automated
    // populator, `BestiaryDatabaseNPCsPopulator.cs:851-868`); the
    // name rides `Lang.GetNPCName` (en-US NPCs.json).
    BestiaryEntryDef {
        credit_id: "FemaleZombie",
        net_id: 200,
        kind: NpcKind::FemaleZombie,
        variant: None,
        quick_unlock: false,
        name: "Zombie",
    },
    // S1-D-3ff wave A — `BestiaryEntry.Enemy(223)` (the automated
    // populator, `BestiaryDatabaseNPCsPopulator.cs:851-868`); the
    // name rides `Lang.GetNPCName` (en-US NPCs.json).
    BestiaryEntryDef {
        credit_id: "ZombieRaincoat",
        net_id: 223,
        kind: NpcKind::ZombieRaincoat,
        variant: None,
        quick_unlock: false,
        name: "Raincoat Zombie",
    },
    // S1-D-3z — the five holiday zombies (`BestiaryEntry.Enemy(319-321/
    // 331-332)`, the automated populator `BestiaryDatabaseNPCsPopulator.cs:
    // 851-868`); the names ride `Lang.GetNPCName` (en-US NPCs.json — all
    // five resolve to "Zombie").
    BestiaryEntryDef {
        credit_id: "ZombieDoctor",
        net_id: 319,
        kind: NpcKind::ZombieDoctor,
        variant: None,
        quick_unlock: false,
        name: "Zombie",
    },
    BestiaryEntryDef {
        credit_id: "ZombieSuperman",
        net_id: 320,
        kind: NpcKind::ZombieSuperman,
        variant: None,
        quick_unlock: false,
        name: "Zombie",
    },
    BestiaryEntryDef {
        credit_id: "ZombiePixie",
        net_id: 321,
        kind: NpcKind::ZombiePixie,
        variant: None,
        quick_unlock: false,
        name: "Zombie",
    },
    BestiaryEntryDef {
        credit_id: "ZombieXmas",
        net_id: 331,
        kind: NpcKind::ZombieXmas,
        variant: None,
        quick_unlock: false,
        name: "Zombie",
    },
    BestiaryEntryDef {
        credit_id: "ZombieSweater",
        net_id: 332,
        kind: NpcKind::ZombieSweater,
        variant: None,
        quick_unlock: false,
        name: "Zombie",
    },
    // S1-D-3z (zombie fighter audit) — the blood-moon pair
    // (`BestiaryDatabaseNPCsPopulator.cs:1310`/`:1336` tag both with the
    // BloodMoon spawn condition); the names ride `Lang.GetNPCName`
    // (en-US NPCs.json — `NPCID.cs:12524-12525`'s name map).
    BestiaryEntryDef {
        credit_id: "BloodZombie",
        net_id: 489,
        kind: NpcKind::BloodZombie,
        variant: None,
        quick_unlock: false,
        name: "Blood Zombie",
    },
    BestiaryEntryDef {
        credit_id: "Drippler",
        net_id: 490,
        kind: NpcKind::Drippler,
        variant: None,
        quick_unlock: false,
        name: "Drippler",
    },
    // S1-D-3ff wave A — `BestiaryEntry.Enemy(225)` (the automated
    // populator, `BestiaryDatabaseNPCsPopulator.cs:851-868`); the
    // name rides `Lang.GetNPCName` (en-US NPCs.json).
    BestiaryEntryDef {
        credit_id: "UmbrellaSlime",
        net_id: 225,
        kind: NpcKind::UmbrellaSlime,
        variant: None,
        quick_unlock: false,
        name: "Umbrella Slime",
    },
    BestiaryEntryDef {
        credit_id: "DesertDjinn",
        net_id: 533,
        kind: NpcKind::DesertDjinn,
        variant: None,
        quick_unlock: false,
        name: "Desert Djinn",
    },
    // W4-g5 — the two unclaimed aiStyle-8 casters (`BestiaryEntry.Enemy`,
    // the automated populator `BestiaryDatabaseNPCsPopulator.cs:851-868`;
    // the display names ride `Lang.GetNPCName`, en-US NPCs.json).
    BestiaryEntryDef {
        credit_id: "RuneWizard",
        net_id: 172,
        kind: NpcKind::RuneWizard,
        variant: None,
        quick_unlock: false,
        name: "Rune Wizard",
    },
    BestiaryEntryDef {
        credit_id: "LibrarianSkeleton",
        net_id: 693,
        kind: NpcKind::LibrarianSkeleton,
        variant: None,
        quick_unlock: false,
        name: "Librarian Skeleton",
    },
    // The spheres: the Chaos Ball entry also covers Tim's ChaosBallTim
    // clone 665 (R4 FINAL, research/322 — stat-identical, one aiStyle-9
    // arm, one {31} immunity, one name; both net ids are Hide=true
    // exclusions in C# (`NPCID.cs:1443-1456`) — the port's entry is the
    // pre-existing base row, kept; the 665 banner split (Tim's 177) is
    // unreachable, no credit drains for either).
    BestiaryEntryDef {
        credit_id: "ChaosBall",
        net_id: 30,
        kind: NpcKind::ChaosBall,
        variant: None,
        quick_unlock: false,
        name: "Chaos Ball",
    },
    BestiaryEntryDef {
        credit_id: "BurningSphere",
        net_id: 25,
        kind: NpcKind::BurningSphere,
        variant: None,
        quick_unlock: false,
        name: "Burning Sphere",
    },
    BestiaryEntryDef {
        credit_id: "WaterSphere",
        net_id: 33,
        kind: NpcKind::WaterSphere,
        variant: None,
        quick_unlock: false,
        name: "Water Sphere",
    },
    BestiaryEntryDef {
        credit_id: "Goldfish",
        net_id: 55,
        kind: NpcKind::Goldfish,
        variant: None,
        quick_unlock: false,
        name: "Goldfish",
    },
    BestiaryEntryDef {
        credit_id: "Piranha",
        net_id: 58,
        kind: NpcKind::Piranha,
        variant: None,
        quick_unlock: false,
        name: "Piranha",
    },
    BestiaryEntryDef {
        credit_id: "Shark",
        net_id: 65,
        kind: NpcKind::Shark,
        variant: None,
        quick_unlock: false,
        name: "Shark",
    },
    BestiaryEntryDef {
        credit_id: "DevourerHead",
        net_id: 7,
        kind: NpcKind::DevourerHead,
        variant: None,
        quick_unlock: false,
        name: "Devourer",
    },
    BestiaryEntryDef {
        credit_id: "GiantWormHead",
        net_id: 10,
        kind: NpcKind::GiantWormHead,
        variant: None,
        quick_unlock: false,
        name: "Giant Worm",
    },
    // The wyvern family (`S1-D-3` wyvern): ONE entry on the head's id 87
    // — the segments' bestiary draw modifiers carry `Hide = true`
    // (`NPCID.cs:1821-1856`) and the kills credit the head (the
    // Devourer's fold below, `entry_for_npc`). The Sky spawn-condition
    // tag rides all six ids (`BestiaryDatabaseNPCsPopulator.cs:1713-1718`);
    // the name is the localization's `"WyvernHead": "Wyvern"` row
    // (`Terraria.Localization.Content.en-US.NPCs.json`).
    BestiaryEntryDef {
        credit_id: "WyvernHead",
        net_id: 87,
        kind: NpcKind::WyvernHead,
        variant: None,
        quick_unlock: false,
        name: "Wyvern",
    },
    // The worm segments fold onto the head's entry (`entry_for_npc`'s
    // mapping): the vanilla credits the whole worm to the head's bestiary
    // id — one entry per credit.
    BestiaryEntryDef {
        credit_id: "Merchant",
        net_id: 17,
        kind: NpcKind::Merchant,
        variant: None,
        quick_unlock: false,
        name: "Merchant",
    },
    BestiaryEntryDef {
        credit_id: "Dryad",
        net_id: 20,
        kind: NpcKind::Dryad,
        variant: None,
        quick_unlock: false,
        name: "Dryad",
    },
    BestiaryEntryDef {
        credit_id: "Nurse",
        net_id: 18,
        kind: NpcKind::Nurse,
        variant: None,
        quick_unlock: false,
        name: "Nurse",
    },
    BestiaryEntryDef {
        credit_id: "ArmsDealer",
        net_id: 19,
        kind: NpcKind::ArmsDealer,
        variant: None,
        quick_unlock: false,
        name: "Arms Dealer",
    },
    BestiaryEntryDef {
        credit_id: "Demolitionist",
        net_id: 38,
        kind: NpcKind::Demolitionist,
        variant: None,
        quick_unlock: false,
        name: "Demolitionist",
    },
    BestiaryEntryDef {
        credit_id: "Painter",
        net_id: 227,
        kind: NpcKind::Painter,
        variant: None,
        quick_unlock: false,
        name: "Painter",
    },
    BestiaryEntryDef {
        credit_id: "WitchDoctor",
        net_id: 228,
        kind: NpcKind::WitchDoctor,
        variant: None,
        quick_unlock: false,
        name: "Witch Doctor",
    },
    BestiaryEntryDef {
        credit_id: "Clothier",
        net_id: 54,
        kind: NpcKind::Clothier,
        variant: None,
        quick_unlock: false,
        name: "Clothier",
    },
    BestiaryEntryDef {
        credit_id: "TravellingMerchant",
        net_id: 368,
        kind: NpcKind::TravellingMerchant,
        variant: None,
        quick_unlock: false,
        name: "Travelling Merchant",
    },
    BestiaryEntryDef {
        credit_id: "Angler",
        net_id: 369,
        kind: NpcKind::Angler,
        variant: None,
        quick_unlock: false,
        name: "Angler",
    },
    BestiaryEntryDef {
        credit_id: "Wizard",
        net_id: 108,
        kind: NpcKind::Wizard,
        variant: None,
        quick_unlock: false,
        name: "Wizard",
    },
    BestiaryEntryDef {
        credit_id: "Mechanic",
        net_id: 124,
        kind: NpcKind::Mechanic,
        variant: None,
        quick_unlock: false,
        name: "Mechanic",
    },
    BestiaryEntryDef {
        credit_id: "Stylist",
        net_id: 353,
        kind: NpcKind::Stylist,
        variant: None,
        quick_unlock: false,
        name: "Stylist",
    },
    BestiaryEntryDef {
        credit_id: "TaxCollector",
        net_id: 441,
        kind: NpcKind::TaxCollector,
        variant: None,
        quick_unlock: false,
        name: "Tax Collector",
    },
    BestiaryEntryDef {
        credit_id: "Golfer",
        net_id: 588,
        kind: NpcKind::Golfer,
        variant: None,
        quick_unlock: false,
        name: "Golfer",
    },
    BestiaryEntryDef {
        credit_id: "BestiaryGirl",
        net_id: 633,
        kind: NpcKind::Zoologist,
        variant: None,
        quick_unlock: false,
        name: "Zoologist",
    },
    BestiaryEntryDef {
        credit_id: "Princess",
        net_id: 663,
        kind: NpcKind::Princess,
        variant: None,
        quick_unlock: false,
        name: "Princess",
    },
    BestiaryEntryDef {
        credit_id: "DyeTrader",
        net_id: 207,
        kind: NpcKind::DyeTrader,
        variant: None,
        quick_unlock: false,
        name: "Dye Trader",
    },
    BestiaryEntryDef {
        credit_id: "PartyGirl",
        net_id: 208,
        kind: NpcKind::PartyGirl,
        variant: None,
        quick_unlock: false,
        name: "Party Girl",
    },
    BestiaryEntryDef {
        credit_id: "QueenBee",
        net_id: 222,
        kind: NpcKind::QueenBee,
        variant: None,
        quick_unlock: true,
        name: "Queen Bee",
    },
    BestiaryEntryDef {
        credit_id: "SkeletronHead",
        net_id: 35,
        kind: NpcKind::SkeletronHead,
        variant: None,
        quick_unlock: true,
        name: "Skeletron",
    },
    BestiaryEntryDef {
        credit_id: "WallofFlesh",
        net_id: 113,
        kind: NpcKind::WallOfFlesh,
        variant: None,
        quick_unlock: true,
        name: "Wall of Flesh",
    },
    BestiaryEntryDef {
        credit_id: "TheDestroyer",
        net_id: 134,
        kind: NpcKind::TheDestroyer,
        variant: None,
        quick_unlock: true,
        name: "The Destroyer",
    },
    BestiaryEntryDef {
        credit_id: "SkeletronPrime",
        net_id: 127,
        kind: NpcKind::SkeletronPrime,
        variant: None,
        quick_unlock: true,
        name: "Skeletron Prime",
    },
    BestiaryEntryDef {
        credit_id: "Retinazer",
        net_id: 125,
        kind: NpcKind::Retinazer,
        variant: None,
        quick_unlock: true,
        name: "Retinazer",
    },
    BestiaryEntryDef {
        credit_id: "Spazmatism",
        net_id: 126,
        kind: NpcKind::Spazmatism,
        variant: None,
        quick_unlock: true,
        name: "Spazmatism",
    },
    BestiaryEntryDef {
        credit_id: "FungiSpore",
        net_id: 261,
        kind: NpcKind::FungiSpore,
        variant: None,
        quick_unlock: false,
        name: "Fungi Spore",
    },
    BestiaryEntryDef {
        credit_id: "Plantera",
        net_id: 262,
        kind: NpcKind::Plantera,
        variant: None,
        quick_unlock: true,
        name: "Plantera",
    },
    // The Plantera minions (S1-D-3-wire-plantera-minions): each rides its
    // OWN credit (`BestiaryDatabaseNPCsPopulator.cs:965`/`:1070`/`:1269`/
    // `:1396` attach info to 261/263/264/265) - no fold onto the boss.
    // None sets `boss` (`NPC.cs:12364-12437`), so quick_unlock is false.
    BestiaryEntryDef {
        credit_id: "PlanterasHook",
        net_id: 263,
        kind: NpcKind::PlanterasHook,
        variant: None,
        quick_unlock: false,
        name: "Plantera's Hook",
    },
    BestiaryEntryDef {
        credit_id: "PlanterasTentacle",
        net_id: 264,
        kind: NpcKind::PlanterasTentacle,
        variant: None,
        quick_unlock: false,
        name: "Plantera's Tentacle",
    },
    BestiaryEntryDef {
        credit_id: "Spore",
        net_id: 265,
        kind: NpcKind::Spore,
        variant: None,
        quick_unlock: false,
        name: "Spore",
    },
    BestiaryEntryDef {
        credit_id: "BrainofCthulhu",
        net_id: 266,
        kind: NpcKind::BrainOfCthulhu,
        variant: None,
        quick_unlock: true,
        name: "Brain of Cthulhu",
    },
    BestiaryEntryDef {
        credit_id: "Creeper",
        net_id: 267,
        kind: NpcKind::Creeper,
        variant: None,
        quick_unlock: false,
        name: "Brain of Cthulhu",
    },
    // The Pumpkin Moon wave family (`S1-F-13b`): one entry per kind — the
    // C#'s credit id is the NPC's own persistent name
    // (`ContentSamples.Initialize`, `ID/ContentSamples.cs:868`:
    // `NpcBestiaryCreditIdsByNpcNetIds[i] = NPCID.Search.GetName(netID)`), so
    // the ten Scarecrows each carry a row whose display name is the shared
    // "Scarecrow" (`en-US NPCs.json:260-270`); none of the sixteen sets `boss`
    // (`NPC.cs:12929-13203`), so the quickUnlock flag is false for all
    // (`Bestiary/BestiaryEntry.cs:38-44`).
    BestiaryEntryDef {
        credit_id: "Scarecrow1",
        net_id: 305,
        kind: NpcKind::Scarecrow1,
        variant: None,
        quick_unlock: false,
        name: "Scarecrow",
    },
    BestiaryEntryDef {
        credit_id: "Scarecrow2",
        net_id: 306,
        kind: NpcKind::Scarecrow2,
        variant: None,
        quick_unlock: false,
        name: "Scarecrow",
    },
    BestiaryEntryDef {
        credit_id: "Scarecrow3",
        net_id: 307,
        kind: NpcKind::Scarecrow3,
        variant: None,
        quick_unlock: false,
        name: "Scarecrow",
    },
    BestiaryEntryDef {
        credit_id: "Scarecrow4",
        net_id: 308,
        kind: NpcKind::Scarecrow4,
        variant: None,
        quick_unlock: false,
        name: "Scarecrow",
    },
    BestiaryEntryDef {
        credit_id: "Scarecrow5",
        net_id: 309,
        kind: NpcKind::Scarecrow5,
        variant: None,
        quick_unlock: false,
        name: "Scarecrow",
    },
    BestiaryEntryDef {
        credit_id: "Scarecrow6",
        net_id: 310,
        kind: NpcKind::Scarecrow6,
        variant: None,
        quick_unlock: false,
        name: "Scarecrow",
    },
    BestiaryEntryDef {
        credit_id: "Scarecrow7",
        net_id: 311,
        kind: NpcKind::Scarecrow7,
        variant: None,
        quick_unlock: false,
        name: "Scarecrow",
    },
    BestiaryEntryDef {
        credit_id: "Scarecrow8",
        net_id: 312,
        kind: NpcKind::Scarecrow8,
        variant: None,
        quick_unlock: false,
        name: "Scarecrow",
    },
    BestiaryEntryDef {
        credit_id: "Scarecrow9",
        net_id: 313,
        kind: NpcKind::Scarecrow9,
        variant: None,
        quick_unlock: false,
        name: "Scarecrow",
    },
    BestiaryEntryDef {
        credit_id: "Scarecrow10",
        net_id: 314,
        kind: NpcKind::Scarecrow10,
        variant: None,
        quick_unlock: false,
        name: "Scarecrow",
    },
    BestiaryEntryDef {
        credit_id: "HeadlessHorseman",
        net_id: 315,
        kind: NpcKind::HeadlessHorseman,
        variant: None,
        quick_unlock: false,
        name: "Headless Horseman",
    },
    BestiaryEntryDef {
        credit_id: "MourningWood",
        net_id: 325,
        kind: NpcKind::MourningWood,
        variant: None,
        quick_unlock: false,
        name: "Mourning Wood",
    },
    BestiaryEntryDef {
        credit_id: "Splinterling",
        net_id: 326,
        kind: NpcKind::Splinterling,
        variant: None,
        quick_unlock: false,
        name: "Splinterling",
    },
    BestiaryEntryDef {
        credit_id: "Pumpking",
        net_id: 327,
        kind: NpcKind::Pumpking,
        variant: None,
        quick_unlock: false,
        name: "Pumpking",
    },
    BestiaryEntryDef {
        credit_id: "Hellhound",
        net_id: 329,
        kind: NpcKind::Hellhound,
        variant: None,
        quick_unlock: false,
        name: "Hellhound",
    },
    BestiaryEntryDef {
        credit_id: "Poltergeist",
        net_id: 330,
        kind: NpcKind::Poltergeist,
        variant: None,
        quick_unlock: false,
        name: "Poltergeist",
    },
    // The Frost Moon family, slice A (`S1-F-13c`): the first 7 wave kinds
    // (`NPC.cs:13281-13368`). The Zombie Elf trio shares the display name
    // "Zombie Elf" (the en-US `NPCName` rows), like the Scarecrow pairs.
    BestiaryEntryDef {
        credit_id: "ZombieElf",
        net_id: 338,
        kind: NpcKind::ZombieElf,
        variant: None,
        quick_unlock: false,
        name: "Zombie Elf",
    },
    BestiaryEntryDef {
        credit_id: "ZombieElfBeard",
        net_id: 339,
        kind: NpcKind::ZombieElfBeard,
        variant: None,
        quick_unlock: false,
        name: "Zombie Elf",
    },
    BestiaryEntryDef {
        credit_id: "ZombieElfGirl",
        net_id: 340,
        kind: NpcKind::ZombieElfGirl,
        variant: None,
        quick_unlock: false,
        name: "Zombie Elf",
    },
    BestiaryEntryDef {
        credit_id: "PresentMimic",
        net_id: 341,
        kind: NpcKind::PresentMimic,
        variant: None,
        quick_unlock: false,
        name: "Present Mimic",
    },
    BestiaryEntryDef {
        credit_id: "GingerbreadMan",
        net_id: 342,
        kind: NpcKind::GingerbreadMan,
        variant: None,
        quick_unlock: false,
        name: "Gingerbread Man",
    },
    BestiaryEntryDef {
        credit_id: "Yeti",
        net_id: 343,
        kind: NpcKind::Yeti,
        variant: None,
        quick_unlock: false,
        name: "Yeti",
    },
    BestiaryEntryDef {
        credit_id: "Everscream",
        net_id: 344,
        kind: NpcKind::Everscream,
        variant: None,
        quick_unlock: false,
        name: "Everscream",
    },
    // The Frost Moon wave family (`S1-F-13c` slice B): the credit ids are
    // `NPCID.Search.GetName`'s persistent names (`ContentSamples.cs:868-870`;
    // `NPCID.cs:11367-11381`) — 348/349 carry DISTINCT credits despite the
    // shared "Nutcracker" display name (`NPCs.json`'s 349 row). None of the
    // eight rides the quick-unlock provider dict
    // (`BestiaryDatabaseNPCsPopulator.cs:683-701` — 345/346 included: the
    // event bosses unlock on the normal kill ladder), and each carries the
    // FrostMoon spawn-condition tag (`:1382`/`:1521`/`:1537`/`:1671-1672`/
    // `:1709`/`:1720`/`:1733`).
    BestiaryEntryDef {
        credit_id: "IceQueen",
        net_id: 345,
        kind: NpcKind::IceQueen,
        variant: None,
        quick_unlock: false,
        name: "Ice Queen",
    },
    BestiaryEntryDef {
        credit_id: "SantaNK1",
        net_id: 346,
        kind: NpcKind::SantaNK1,
        variant: None,
        quick_unlock: false,
        name: "Santa-NK1",
    },
    BestiaryEntryDef {
        credit_id: "ElfCopter",
        net_id: 347,
        kind: NpcKind::ElfCopter,
        variant: None,
        quick_unlock: false,
        name: "Elf Copter",
    },
    BestiaryEntryDef {
        credit_id: "Nutcracker",
        net_id: 348,
        kind: NpcKind::Nutcracker,
        variant: None,
        quick_unlock: false,
        name: "Nutcracker",
    },
    BestiaryEntryDef {
        credit_id: "NutcrackerSpinning",
        net_id: 349,
        kind: NpcKind::NutcrackerSpinning,
        variant: None,
        quick_unlock: false,
        name: "Nutcracker",
    },
    BestiaryEntryDef {
        credit_id: "ElfArcher",
        net_id: 350,
        kind: NpcKind::ElfArcher,
        variant: None,
        quick_unlock: false,
        name: "Elf Archer",
    },
    BestiaryEntryDef {
        credit_id: "Krampus",
        net_id: 351,
        kind: NpcKind::Krampus,
        variant: None,
        quick_unlock: false,
        name: "Krampus",
    },
    BestiaryEntryDef {
        credit_id: "Flocko",
        net_id: 352,
        kind: NpcKind::Flocko,
        variant: None,
        quick_unlock: false,
        name: "Flocko",
    },
    BestiaryEntryDef {
        credit_id: "GoblinThief",
        net_id: 27,
        kind: NpcKind::GoblinThief,
        variant: None,
        quick_unlock: false,
        name: "Goblin Thief",
    },
    BestiaryEntryDef {
        credit_id: "GoblinWarrior",
        net_id: 28,
        kind: NpcKind::GoblinWarrior,
        variant: None,
        quick_unlock: false,
        name: "Goblin Warrior",
    },
    BestiaryEntryDef {
        credit_id: "GoblinArcher",
        net_id: 111,
        kind: NpcKind::GoblinArcher,
        variant: None,
        quick_unlock: false,
        name: "Goblin Archer",
    },
    BestiaryEntryDef {
        credit_id: "GoblinPeon",
        net_id: 26,
        kind: NpcKind::GoblinPeon,
        variant: None,
        quick_unlock: false,
        name: "Goblin Peon",
    },
    BestiaryEntryDef {
        credit_id: "GoblinSummoner",
        net_id: 471,
        kind: NpcKind::GoblinSummoner,
        variant: None,
        quick_unlock: false,
        name: "Goblin Summoner",
    },
    BestiaryEntryDef {
        credit_id: "SnowmanGangsta",
        net_id: 143,
        kind: NpcKind::SnowmanGangsta,
        variant: None,
        quick_unlock: false,
        name: "Snowman Gangsta",
    },
    BestiaryEntryDef {
        // Net 144 = `NPCID.MisterStabby` (`NPCID.cs:10965`) — the credit
        // key and name follow the C#; the kind name `SnowBalla` is the
        // port's pre-existing roster label (S1-D-3 residuals).
        credit_id: "MisterStabby",
        net_id: 144,
        kind: NpcKind::SnowBalla,
        variant: None,
        quick_unlock: false,
        name: "Mister Stabby",
    },
    BestiaryEntryDef {
        // Net 145 = `NPCID.SnowBalla` (`NPCID.cs:10967`); the kind name
        // `MysterySnowman` is the port's pre-existing roster label.
        credit_id: "SnowBalla",
        net_id: 145,
        kind: NpcKind::MysterySnowman,
        variant: None,
        quick_unlock: false,
        name: "Snow Balla",
    },
    BestiaryEntryDef {
        credit_id: "MartianOfficer",
        net_id: 383,
        kind: NpcKind::MartianOfficer,
        variant: None,
        quick_unlock: false,
        name: "Martian Officer",
    },
    BestiaryEntryDef {
        credit_id: "GrayGrunt",
        net_id: 385,
        kind: NpcKind::GrayGrunt,
        variant: None,
        quick_unlock: false,
        name: "Gray Grunt",
    },
    BestiaryEntryDef {
        credit_id: "MartianEngineer",
        net_id: 386,
        kind: NpcKind::MartianEngineer,
        variant: None,
        quick_unlock: false,
        name: "Martian Engineer",
    },
    BestiaryEntryDef {
        credit_id: "GigaZapper",
        net_id: 389,
        kind: NpcKind::GigaZapper,
        variant: None,
        quick_unlock: false,
        name: "Giga Zapper",
    },
    BestiaryEntryDef {
        credit_id: "MartianWalker",
        net_id: 520,
        kind: NpcKind::MartianWalker,
        variant: None,
        quick_unlock: false,
        name: "Martian Walker",
    },
    BestiaryEntryDef {
        credit_id: "MartianProbe",
        net_id: 399,
        kind: NpcKind::MartianProbe,
        variant: None,
        quick_unlock: false,
        name: "Martian Probe",
    },
    BestiaryEntryDef {
        credit_id: "MartianDrone",
        net_id: 388,
        kind: NpcKind::MartianDrone,
        variant: None,
        quick_unlock: false,
        name: "Martian Drone",
    },
    BestiaryEntryDef {
        credit_id: "BrainScrambler",
        net_id: 381,
        kind: NpcKind::BrainScrambler,
        variant: None,
        quick_unlock: false,
        name: "Brain Scrambler",
    },
    BestiaryEntryDef {
        credit_id: "RayGunner",
        net_id: 382,
        kind: NpcKind::RayGunner,
        variant: None,
        quick_unlock: false,
        name: "Ray Gunner",
    },
    BestiaryEntryDef {
        credit_id: "ScutlixRider",
        net_id: 390,
        kind: NpcKind::ScutlixRider,
        variant: None,
        quick_unlock: false,
        name: "Scutlix Rider",
    },
    BestiaryEntryDef {
        credit_id: "Scutlix",
        net_id: 391,
        kind: NpcKind::Scutlix,
        variant: None,
        quick_unlock: false,
        name: "Scutlix",
    },
    BestiaryEntryDef {
        credit_id: "MartianSaucer",
        net_id: 392,
        kind: NpcKind::MartianSaucer,
        variant: None,
        quick_unlock: false,
        name: "Martian Saucer",
    },
    BestiaryEntryDef {
        credit_id: "MartianSaucerTurret",
        net_id: 393,
        kind: NpcKind::MartianSaucerTurret,
        variant: None,
        quick_unlock: false,
        name: "Martian Saucer Turret",
    },
    BestiaryEntryDef {
        credit_id: "MartianSaucerCannon",
        net_id: 394,
        kind: NpcKind::MartianSaucerCannon,
        variant: None,
        quick_unlock: false,
        name: "Martian Saucer Cannon",
    },
    BestiaryEntryDef {
        credit_id: "MartianSaucerCore",
        net_id: 395,
        kind: NpcKind::MartianSaucerCore,
        variant: None,
        quick_unlock: true,
        name: "Martian Saucer Core",
    },
    // The Tesla Turret (`S1-F-13d`): the Engineer's placed turret — the
    // C#'s name row is "Tesla Turret" (`NPCID.cs:12449`'s `NameToIdMap`),
    // the draw mods Position(2,0)/Velocity 3 (`NPCID.cs:3788-3794`). The
    // Force Bubble 384 has NO entry: its draw modifiers hide it
    // (`NPCID.cs:3766-3770`'s `Hide = true`) — it folds onto the Officer's
    // credit in `entry_for_npc` (the banner fold's precedent,
    // `BannerSystem.cs:777-779`: 383/384 share banner 141).
    BestiaryEntryDef {
        credit_id: "MartianTurret",
        net_id: 387,
        kind: NpcKind::MartianTurret,
        variant: None,
        quick_unlock: false,
        name: "Tesla Turret",
    },
    BestiaryEntryDef {
        credit_id: "FlyingDutchman",
        net_id: 491,
        kind: NpcKind::FlyingDutchman,
        variant: None,
        quick_unlock: false,
        name: "Flying Dutchman",
    },
    BestiaryEntryDef {
        credit_id: "FlyingDutchmanCannon",
        net_id: 492,
        kind: NpcKind::FlyingDutchmanCannon,
        variant: None,
        quick_unlock: false,
        name: "Flying Dutchman Cannon",
    },
    BestiaryEntryDef {
        credit_id: "DD2GoblinT1",
        net_id: 552,
        kind: NpcKind::DD2GoblinT1,
        variant: None,
        quick_unlock: false,
        name: "Etherian Goblin",
    },
    BestiaryEntryDef {
        credit_id: "DD2GoblinT2",
        net_id: 553,
        kind: NpcKind::DD2GoblinT2,
        variant: None,
        quick_unlock: false,
        name: "Etherian Goblin Warrior",
    },
    BestiaryEntryDef {
        credit_id: "DD2GoblinT3",
        net_id: 554,
        kind: NpcKind::DD2GoblinT3,
        variant: None,
        quick_unlock: false,
        name: "Etherian Goblin Elite",
    },
    BestiaryEntryDef {
        credit_id: "DD2GoblinBomberT1",
        net_id: 555,
        kind: NpcKind::DD2GoblinBomberT1,
        variant: None,
        quick_unlock: false,
        name: "Etherian Goblin Bomber",
    },
    BestiaryEntryDef {
        credit_id: "DD2GoblinBomberT2",
        net_id: 556,
        kind: NpcKind::DD2GoblinBomberT2,
        variant: None,
        quick_unlock: false,
        name: "Etherian Goblin Bomber Elite",
    },
    BestiaryEntryDef {
        credit_id: "DD2GoblinBomberT3",
        net_id: 557,
        kind: NpcKind::DD2GoblinBomberT3,
        variant: None,
        quick_unlock: false,
        name: "Etherian Goblin Bomber Captain",
    },
    BestiaryEntryDef {
        credit_id: "DD2JavelinstT1",
        net_id: 561,
        kind: NpcKind::DD2JavelinT1,
        variant: None,
        quick_unlock: false,
        name: "Etherian Javelin Thrower",
    },
    BestiaryEntryDef {
        credit_id: "DD2JavelinstT2",
        net_id: 562,
        kind: NpcKind::DD2JavelinT2,
        variant: None,
        quick_unlock: false,
        name: "Etherian Javelin Thrower Elite",
    },
    BestiaryEntryDef {
        credit_id: "DD2JavelinstT3",
        net_id: 563,
        kind: NpcKind::DD2JavelinT3,
        variant: None,
        quick_unlock: false,
        name: "Etherian Javelin Thrower Captain",
    },
    BestiaryEntryDef {
        credit_id: "DD2SkeletonT1",
        net_id: 566,
        kind: NpcKind::DD2SkeletonT1,
        variant: None,
        quick_unlock: false,
        name: "Etherian Skeleton",
    },
    BestiaryEntryDef {
        credit_id: "DD2SkeletonT3",
        net_id: 567,
        kind: NpcKind::DD2SkeletonT3,
        variant: None,
        quick_unlock: false,
        name: "Etherian Skeleton Captain",
    },
    BestiaryEntryDef {
        credit_id: "DD2DarkMageT1",
        net_id: 564,
        kind: NpcKind::DD2DarkMage,
        variant: None,
        quick_unlock: false,
        name: "Dark Mage",
    },
    BestiaryEntryDef {
        credit_id: "DD2Betsy",
        net_id: 551,
        kind: NpcKind::DD2Betsy,
        variant: None,
        quick_unlock: false,
        name: "Betsy",
    },
    BestiaryEntryDef {
        credit_id: "DD2WitherBeastT2",
        net_id: 568,
        kind: NpcKind::DD2WitherBeast,
        variant: None,
        quick_unlock: false,
        name: "Wither Beast",
    },
    BestiaryEntryDef {
        credit_id: "DD2Bartender",
        net_id: 550,
        kind: NpcKind::DD2Bartender,
        variant: None,
        quick_unlock: false,
        name: "Bartender",
    },
    BestiaryEntryDef {
        credit_id: "DD2EterniaCrystal",
        net_id: 548,
        kind: NpcKind::DD2EterniaCrystal,
        variant: None,
        quick_unlock: false,
        name: "Eternia Crystal",
    },
    BestiaryEntryDef {
        credit_id: "DD2LanePortal",
        net_id: 549,
        kind: NpcKind::DD2LanePortal,
        variant: None,
        quick_unlock: false,
        name: "Etherian Portal",
    },
    // The Old One's Army's tier-2/3 families (M4-EVENT-19): the
    // lane spawn table's unported members, in net-id order.
    BestiaryEntryDef {
        credit_id: "DD2WyvernT1",
        net_id: 558,
        kind: NpcKind::DD2WyvernT1,
        variant: None,
        quick_unlock: false,
        name: "Etherian Wyvern",
    },
    BestiaryEntryDef {
        credit_id: "DD2WyvernT2",
        net_id: 559,
        kind: NpcKind::DD2WyvernT2,
        variant: None,
        quick_unlock: false,
        name: "Etherian Wyvern",
    },
    BestiaryEntryDef {
        credit_id: "DD2WyvernT3",
        net_id: 560,
        kind: NpcKind::DD2WyvernT3,
        variant: None,
        quick_unlock: false,
        name: "Etherian Wyvern",
    },
    BestiaryEntryDef {
        credit_id: "DD2DarkMageT3",
        net_id: 565,
        kind: NpcKind::DD2DarkMageT3,
        variant: None,
        quick_unlock: false,
        name: "Dark Mage",
    },
    BestiaryEntryDef {
        credit_id: "DD2WitherBeastT3",
        net_id: 569,
        kind: NpcKind::DD2WitherBeastT3,
        variant: None,
        quick_unlock: false,
        name: "Wither Beast",
    },
    BestiaryEntryDef {
        credit_id: "DD2DrakinT2",
        net_id: 570,
        kind: NpcKind::DD2DrakinT2,
        variant: None,
        quick_unlock: false,
        name: "Drakin",
    },
    BestiaryEntryDef {
        credit_id: "DD2DrakinT3",
        net_id: 571,
        kind: NpcKind::DD2DrakinT3,
        variant: None,
        quick_unlock: false,
        name: "Drakin",
    },
    BestiaryEntryDef {
        credit_id: "DD2KoboldWalkerT2",
        net_id: 572,
        kind: NpcKind::DD2KoboldWalkerT2,
        variant: None,
        quick_unlock: false,
        name: "Kobold",
    },
    BestiaryEntryDef {
        credit_id: "DD2KoboldWalkerT3",
        net_id: 573,
        kind: NpcKind::DD2KoboldWalkerT3,
        variant: None,
        quick_unlock: false,
        name: "Kobold",
    },
    BestiaryEntryDef {
        credit_id: "DD2KoboldFlyerT2",
        net_id: 574,
        kind: NpcKind::DD2KoboldFlyerT2,
        variant: None,
        quick_unlock: false,
        name: "Kobold Glider",
    },
    BestiaryEntryDef {
        credit_id: "DD2KoboldFlyerT3",
        net_id: 575,
        kind: NpcKind::DD2KoboldFlyerT3,
        variant: None,
        quick_unlock: false,
        name: "Kobold Glider",
    },
    BestiaryEntryDef {
        credit_id: "DD2OgreT2",
        net_id: 576,
        kind: NpcKind::DD2OgreT2,
        variant: None,
        quick_unlock: false,
        name: "Ogre",
    },
    BestiaryEntryDef {
        credit_id: "DD2OgreT3",
        net_id: 577,
        kind: NpcKind::DD2OgreT3,
        variant: None,
        quick_unlock: false,
        name: "Ogre",
    },
    BestiaryEntryDef {
        credit_id: "AncientCultist",
        net_id: 439,
        kind: NpcKind::AncientCultist,
        variant: None,
        quick_unlock: true,
        name: "Ancient Cultist",
    },
    BestiaryEntryDef {
        credit_id: "StardustTower",
        net_id: 493,
        kind: NpcKind::StardustTower,
        variant: None,
        quick_unlock: true,
        name: "Stardust Pillar",
    },
    BestiaryEntryDef {
        credit_id: "NebulaTower",
        net_id: 507,
        kind: NpcKind::NebulaTower,
        variant: None,
        quick_unlock: true,
        name: "Nebula Pillar",
    },
    BestiaryEntryDef {
        credit_id: "SolarTower",
        net_id: 517,
        kind: NpcKind::SolarTower,
        variant: None,
        quick_unlock: true,
        name: "Solar Pillar",
    },
    BestiaryEntryDef {
        credit_id: "VortexTower",
        net_id: 422,
        kind: NpcKind::VortexTower,
        variant: None,
        quick_unlock: true,
        name: "Vortex Pillar",
    },
    BestiaryEntryDef {
        credit_id: "StardustCellBig",
        net_id: 405,
        kind: NpcKind::StardustCellBig,
        variant: None,
        quick_unlock: false,
        name: "Big Star Cell",
    },
    BestiaryEntryDef {
        credit_id: "StardustCellSmall",
        net_id: 406,
        kind: NpcKind::StardustCellSmall,
        variant: None,
        quick_unlock: false,
        name: "Small Star Cell",
    },
    BestiaryEntryDef {
        credit_id: "StardustJellyfishBig",
        net_id: 407,
        kind: NpcKind::StardustJellyfishBig,
        variant: None,
        quick_unlock: false,
        name: "Star Jellyfish",
    },
    BestiaryEntryDef {
        credit_id: "NebulaBeast",
        net_id: 423,
        kind: NpcKind::NebulaBeast,
        variant: None,
        quick_unlock: false,
        name: "Nebula Floater",
    },
    BestiaryEntryDef {
        credit_id: "VortexRifleman",
        net_id: 425,
        kind: NpcKind::VortexRifleman,
        variant: None,
        quick_unlock: false,
        name: "Vortex Rifleman",
    },
    BestiaryEntryDef {
        credit_id: "VortexSoldier",
        net_id: 429,
        kind: NpcKind::VortexSoldier,
        variant: None,
        quick_unlock: false,
        name: "Vortex Soldier",
    },
    BestiaryEntryDef {
        credit_id: "VortexHunter",
        net_id: 426,
        kind: NpcKind::VortexHunter,
        variant: None,
        quick_unlock: false,
        name: "Vortex Hunter",
    },
    BestiaryEntryDef {
        credit_id: "VortexAlien",
        net_id: 427,
        kind: NpcKind::VortexAlien,
        variant: None,
        quick_unlock: false,
        name: "Vortex Alien",
    },
    BestiaryEntryDef {
        credit_id: "NebulaFloater",
        net_id: 420,
        kind: NpcKind::NebulaFloater,
        variant: None,
        quick_unlock: false,
        name: "Nebula Floater",
    },
    BestiaryEntryDef {
        credit_id: "BrainSuckler",
        net_id: 421,
        kind: NpcKind::BrainSuckler,
        variant: None,
        quick_unlock: false,
        name: "Brain Suckler",
    },
    BestiaryEntryDef {
        credit_id: "StardustSpiderBig",
        net_id: 409,
        kind: NpcKind::StardustSpiderBig,
        variant: None,
        quick_unlock: false,
        name: "Stardust Spider",
    },
    BestiaryEntryDef {
        credit_id: "StardustSpiderSmall",
        net_id: 410,
        kind: NpcKind::StardustSpiderSmall,
        variant: None,
        quick_unlock: false,
        name: "Baby Spider",
    },
    BestiaryEntryDef {
        credit_id: "StardustSoldier",
        net_id: 411,
        kind: NpcKind::StardustSoldier,
        variant: None,
        quick_unlock: false,
        name: "Stardust Soldier",
    },
    BestiaryEntryDef {
        credit_id: "SolarDrakomire",
        net_id: 415,
        kind: NpcKind::SolarDrakomire,
        variant: None,
        quick_unlock: false,
        name: "Drakomire",
    },
    BestiaryEntryDef {
        credit_id: "SolarDrakomireRider",
        net_id: 416,
        kind: NpcKind::SolarDrakomireRider,
        variant: None,
        quick_unlock: false,
        name: "Drakomire Rider",
    },
    BestiaryEntryDef {
        credit_id: "SolarSroller",
        net_id: 417,
        kind: NpcKind::SolarSroller,
        variant: None,
        quick_unlock: false,
        name: "Sroller",
    },
    BestiaryEntryDef {
        credit_id: "SolarCorite",
        net_id: 418,
        kind: NpcKind::SolarCorite,
        variant: None,
        quick_unlock: false,
        name: "Corite",
    },
    BestiaryEntryDef {
        credit_id: "SolarCrawltipedeHead",
        net_id: 412,
        kind: NpcKind::SolarCrawltipedeHead,
        variant: None,
        quick_unlock: false,
        name: "Crawltipede Head",
    },
    BestiaryEntryDef {
        credit_id: "SolarCrawltipedeBody",
        net_id: 413,
        kind: NpcKind::SolarCrawltipedeBody,
        variant: None,
        quick_unlock: false,
        name: "Crawltipede Body",
    },
    BestiaryEntryDef {
        credit_id: "SolarCrawltipedeTail",
        net_id: 414,
        kind: NpcKind::SolarCrawltipedeTail,
        variant: None,
        quick_unlock: false,
        name: "Crawltipede Tail",
    },
    BestiaryEntryDef {
        credit_id: "StardustWormHead",
        net_id: 402,
        kind: NpcKind::StardustWormHead,
        variant: None,
        quick_unlock: false,
        name: "Stardust Worm",
    },
    BestiaryEntryDef {
        credit_id: "SolarSpearman",
        net_id: 518,
        kind: NpcKind::SolarSpearman,
        variant: None,
        quick_unlock: false,
        name: "Solar Spearman",
    },
    BestiaryEntryDef {
        credit_id: "SolarSelenian",
        net_id: 419,
        kind: NpcKind::SolarSelenian,
        variant: None,
        quick_unlock: false,
        name: "Selenian",
    },
    BestiaryEntryDef {
        credit_id: "Predictor",
        net_id: 424,
        kind: NpcKind::Predictor,
        variant: None,
        quick_unlock: false,
        name: "Predictor",
    },
    BestiaryEntryDef {
        credit_id: "VortexHeadcrab",
        net_id: 428,
        kind: NpcKind::VortexHeadcrab,
        variant: None,
        quick_unlock: false,
        name: "Vortex Headcrab",
    },
    BestiaryEntryDef {
        credit_id: "MoonLordHead",
        net_id: 396,
        kind: NpcKind::MoonLordHead,
        variant: None,
        quick_unlock: true,
        name: "Moon Lord",
    },
    BestiaryEntryDef {
        credit_id: "MoonLordHand",
        net_id: 397,
        kind: NpcKind::MoonLordHand,
        variant: None,
        quick_unlock: true,
        name: "Moon Lord's Hand",
    },
    BestiaryEntryDef {
        credit_id: "MoonLordCore",
        net_id: 398,
        kind: NpcKind::MoonLordCore,
        variant: None,
        quick_unlock: true,
        name: "Moon Lord's Core",
    },
    BestiaryEntryDef {
        credit_id: "MoonLordFreeEye",
        net_id: 400,
        kind: NpcKind::MoonLordFreeEye,
        variant: None,
        quick_unlock: false,
        name: "True Eye of Cthulhu",
    },
    BestiaryEntryDef {
        credit_id: "MoonLordLeechBlob",
        net_id: 401,
        kind: NpcKind::MoonLordLeechBlob,
        variant: None,
        quick_unlock: false,
        name: "Moon Leech Clot",
    },
    BestiaryEntryDef {
        credit_id: "PirateDeckhand",
        net_id: 212,
        kind: NpcKind::PirateDeckhand,
        variant: None,
        quick_unlock: false,
        name: "Pirate Deckhand",
    },
    BestiaryEntryDef {
        credit_id: "PirateCaptainsMate",
        net_id: 213,
        kind: NpcKind::PirateCaptainsMate,
        variant: None,
        quick_unlock: false,
        name: "Pirate Captains Mate",
    },
    BestiaryEntryDef {
        credit_id: "PirateArcher",
        net_id: 214,
        kind: NpcKind::PirateArcher,
        variant: None,
        quick_unlock: false,
        name: "Pirate Archer",
    },
    BestiaryEntryDef {
        credit_id: "PirateCorsair",
        net_id: 215,
        kind: NpcKind::PirateCorsair,
        variant: None,
        quick_unlock: false,
        name: "Pirate Corsair",
    },
    BestiaryEntryDef {
        credit_id: "PirateCaptain",
        net_id: 216,
        kind: NpcKind::PirateCaptain,
        variant: None,
        quick_unlock: false,
        name: "Pirate Captain",
    },
    BestiaryEntryDef {
        credit_id: "Golem",
        net_id: 245,
        kind: NpcKind::Golem,
        variant: None,
        quick_unlock: true,
        name: "Golem",
    },
    // S1-D-3 W-9 fishron wiring — the boss's entry (the vanilla
    // `NPCID.Search.GetName(370)` = "DukeFishron", the boss quick-unlock
    // rule) plus the Sharkron's own entry (372 has one; the C#'s
    // `BestiaryDatabaseNPCsPopulator.cs:691` maps its UI credit onto 370,
    // the kill credit stays "Sharkron"). 371/373 are Hide=true exclusions
    // (`NPCID.cs`'s NPCBestiaryDrawOffset rows) — no entries; the port's
    // `entry_for_npc` folds them onto the boss/Sharkron rows (the Golem
    // parts' precedent) so `net_id()` resolves; their DRAW sheet id rides
    // `texture_net_id`'s own rows, not the folded net id.
    BestiaryEntryDef {
        credit_id: "DukeFishron",
        net_id: 370,
        kind: NpcKind::DukeFishron,
        variant: None,
        quick_unlock: true,
        name: "Duke Fishron",
    },
    BestiaryEntryDef {
        credit_id: "Sharkron",
        net_id: 372,
        kind: NpcKind::Sharkron,
        variant: None,
        quick_unlock: false,
        name: "Sharkron",
    },
    // W4-d125 empress wiring — the boss's entry (the vanilla
    // `NPCID.Search.GetName(636)` = "HallowBoss", `NPCID.cs:11949`; the
    // boss quick-unlock rule) + the Prismatic Lacewing's own entry
    // (661 = "EmpressButterfly", `NPCID.cs:11999`; the populator
    // attaches info at `BestiaryDatabaseNPCsPopulator.cs:1185`).
    BestiaryEntryDef {
        credit_id: "HallowBoss",
        net_id: 636,
        kind: NpcKind::EmpressOfLight,
        variant: None,
        quick_unlock: true,
        name: "Empress of Light",
    },
    BestiaryEntryDef {
        credit_id: "EmpressButterfly",
        net_id: 661,
        kind: NpcKind::PrismaticLacewing,
        variant: None,
        quick_unlock: false,
        name: "Prismatic Lacewing",
    },
    BestiaryEntryDef {
        credit_id: "PrimeCannon",
        net_id: 128,
        kind: NpcKind::PrimeCannon,
        variant: None,
        quick_unlock: false,
        name: "Prime Cannon",
    },
    BestiaryEntryDef {
        credit_id: "PrimeSaw",
        net_id: 129,
        kind: NpcKind::PrimeSaw,
        variant: None,
        quick_unlock: false,
        name: "Prime Saw",
    },
    BestiaryEntryDef {
        credit_id: "PrimeVice",
        net_id: 130,
        kind: NpcKind::PrimeVice,
        variant: None,
        quick_unlock: false,
        name: "Prime Vice",
    },
    BestiaryEntryDef {
        credit_id: "PrimeLaser",
        net_id: 131,
        kind: NpcKind::PrimeLaser,
        variant: None,
        quick_unlock: false,
        name: "Prime Laser",
    },
    BestiaryEntryDef {
        credit_id: "WallofFleshEye",
        net_id: 114,
        kind: NpcKind::WallOfFleshEye,
        variant: None,
        quick_unlock: false,
        name: "Wall of Flesh Eye",
    },
    BestiaryEntryDef {
        credit_id: "SkeletronHand",
        net_id: 36,
        kind: NpcKind::SkeletronHand,
        variant: None,
        quick_unlock: false,
        name: "Skeletron Hand",
    },
    BestiaryEntryDef {
        credit_id: "Deerclops",
        net_id: 668,
        kind: NpcKind::Deerclops,
        variant: None,
        quick_unlock: true,
        name: "Deerclops",
    },
    BestiaryEntryDef {
        credit_id: "GoblinTinkerer",
        net_id: 107,
        kind: NpcKind::GoblinTinkerer,
        variant: None,
        quick_unlock: false,
        name: "Goblin Tinkerer",
    },
    BestiaryEntryDef {
        credit_id: "EaterofWorldsHead",
        net_id: 13,
        kind: NpcKind::EaterOfWorldsHead,
        variant: None,
        quick_unlock: true,
        name: "Eater of Worlds",
    },
    // The small-kinds wiring wave (S1-D-3-wire-smallkinds): the four
    // wired kinds' own entries — the populator registers all four
    // (`BestiaryDatabaseNPCsPopulator.cs:1174`/`:1180`/`:1282`/`:1413`
    // find their entries by id).
    BestiaryEntryDef {
        credit_id: "BlazingWheel",
        net_id: 72,
        kind: NpcKind::BlazingWheel,
        variant: None,
        quick_unlock: false,
        name: "Blazing Wheel",
    },
    BestiaryEntryDef {
        credit_id: "AngryNimbus",
        net_id: 250,
        kind: NpcKind::AngryNimbus,
        variant: None,
        quick_unlock: false,
        name: "Angry Nimbus",
    },
    BestiaryEntryDef {
        credit_id: "DungeonSpirit",
        net_id: 288,
        kind: NpcKind::DungeonSpirit,
        variant: None,
        quick_unlock: false,
        name: "Dungeon Spirit",
    },
    // W4-d34 kinds 1-70 coverage wave: the populator registers each by its
    // vanilla net id (`BestiaryDatabaseNPCsPopulator.cs`'s automated pass).
    BestiaryEntryDef {
        credit_id: "AngryBones",
        net_id: 31,
        kind: NpcKind::AngryBones,
        variant: None,
        quick_unlock: false,
        name: "Angry Bones",
    },
    BestiaryEntryDef {
        credit_id: "CursedSkull",
        net_id: 34,
        kind: NpcKind::CursedSkull,
        variant: None,
        quick_unlock: false,
        name: "Cursed Skull",
    },
    BestiaryEntryDef {
        credit_id: "OldMan",
        net_id: 37,
        kind: NpcKind::OldMan,
        variant: None,
        quick_unlock: false,
        name: "Old Man",
    },
    BestiaryEntryDef {
        credit_id: "ManEater",
        net_id: 43,
        kind: NpcKind::ManEater,
        variant: None,
        quick_unlock: false,
        name: "Man Eater",
    },
    // W4-d34 kinds 1-70 (phase 2): the Bone Serpent's ONE entry on the head's
    // id 39 — the body/tail Hide=true (`NPCID.cs:1527-1539`) fold onto it (the
    // Devourer precedent). The CorruptBunny + Harpy register by their ids.
    BestiaryEntryDef {
        credit_id: "BoneSerpentHead",
        net_id: 39,
        kind: NpcKind::BoneSerpentHead,
        variant: None,
        quick_unlock: false,
        name: "Bone Serpent",
    },
    BestiaryEntryDef {
        credit_id: "CorruptBunny",
        net_id: 47,
        kind: NpcKind::CorruptBunny,
        variant: None,
        quick_unlock: false,
        name: "Corrupt Bunny",
    },
    BestiaryEntryDef {
        credit_id: "Harpy",
        net_id: 48,
        kind: NpcKind::Harpy,
        variant: None,
        quick_unlock: false,
        name: "Harpy",
    },
    // W4-d34 kinds 1-70 (phase 3): the five register by their vanilla ids
    // (the populator's automated pass).
    BestiaryEntryDef {
        credit_id: "Snatcher",
        net_id: 56,
        kind: NpcKind::Snatcher,
        variant: None,
        quick_unlock: false,
        name: "Snatcher",
    },
    BestiaryEntryDef {
        credit_id: "CorruptGoldfish",
        net_id: 57,
        kind: NpcKind::CorruptGoldfish,
        variant: None,
        quick_unlock: false,
        name: "Corrupt Goldfish",
    },
    BestiaryEntryDef {
        credit_id: "Vulture",
        net_id: 61,
        kind: NpcKind::Vulture,
        variant: None,
        quick_unlock: false,
        name: "Vulture",
    },
    BestiaryEntryDef {
        credit_id: "Demon",
        net_id: 62,
        kind: NpcKind::Demon,
        variant: None,
        quick_unlock: false,
        name: "Demon",
    },
    BestiaryEntryDef {
        credit_id: "BlueJellyfish",
        net_id: 63,
        kind: NpcKind::BlueJellyfish,
        variant: None,
        quick_unlock: false,
        name: "Blue Jellyfish",
    },
    // W4-d34 kinds 1-70 (phase 4, merge-side per 367 §1): the Crab and
    // Dungeon Guardian register by their vanilla ids (the populator's
    // automated pass). The Dungeon Guardian's provider is quickUnlock
    // (`BestiaryDatabaseNPCsPopulator.cs:676`). The PinkJellyfish/Antlion/
    // SpikeBall entries dropped with their dup kinds — g8 owns 745-747.
    BestiaryEntryDef {
        credit_id: "Crab",
        net_id: 67,
        kind: NpcKind::Crab,
        variant: None,
        quick_unlock: false,
        name: "Crab",
    },
    BestiaryEntryDef {
        credit_id: "DungeonGuardian",
        net_id: 68,
        kind: NpcKind::DungeonGuardian,
        variant: None,
        quick_unlock: true,
        name: "Dungeon Guardian",
    },
    BestiaryEntryDef {
        credit_id: "Bunny",
        net_id: 46,
        kind: NpcKind::Bunny,
        variant: None,
        quick_unlock: false,
        name: "Bunny",
    },
    // W4-d38 b7 — BestiaryEntry.Enemy(333)
    // (the automated populator, BestiaryDatabaseNPCsPopulator.cs:851-868).
    BestiaryEntryDef {
        credit_id: "SlimeRibbonWhite",
        net_id: 333,
        kind: NpcKind::SlimeRibbonWhite,
        variant: None,
        quick_unlock: false,
        name: "Slime",
    },
    // W4-d38 b7 — BestiaryEntry.Enemy(334)
    // (the automated populator, BestiaryDatabaseNPCsPopulator.cs:851-868).
    BestiaryEntryDef {
        credit_id: "SlimeRibbonYellow",
        net_id: 334,
        kind: NpcKind::SlimeRibbonYellow,
        variant: None,
        quick_unlock: false,
        name: "Slime",
    },
    // W4-d38 b7 — BestiaryEntry.Enemy(335)
    // (the automated populator, BestiaryDatabaseNPCsPopulator.cs:851-868).
    BestiaryEntryDef {
        credit_id: "SlimeRibbonGreen",
        net_id: 335,
        kind: NpcKind::SlimeRibbonGreen,
        variant: None,
        quick_unlock: false,
        name: "Slime",
    },
    // W4-d38 b7 — BestiaryEntry.Enemy(336)
    // (the automated populator, BestiaryDatabaseNPCsPopulator.cs:851-868).
    BestiaryEntryDef {
        credit_id: "SlimeRibbonRed",
        net_id: 336,
        kind: NpcKind::SlimeRibbonRed,
        variant: None,
        quick_unlock: false,
        name: "Slime",
    },
    // W4-d38 b7 — BestiaryEntry.Critter(337)
    // (the automated populator, BestiaryDatabaseNPCsPopulator.cs:851-868).
    BestiaryEntryDef {
        credit_id: "BunnyXmas",
        net_id: 337,
        kind: NpcKind::BunnyXmas,
        variant: None,
        quick_unlock: false,
        name: "Bunny",
    },
    // W4-d38 b6 — BestiaryEntry.Enemy(318)
    // (the automated populator, BestiaryDatabaseNPCsPopulator.cs:851-868).
    BestiaryEntryDef {
        credit_id: "DemonEyeSpaceship",
        net_id: 318,
        kind: NpcKind::DemonEyeSpaceship,
        variant: None,
        quick_unlock: false,
        name: "Demon Eye",
    },
    // W4-d38 b6 — BestiaryEntry.Enemy(322)
    // (the automated populator, BestiaryDatabaseNPCsPopulator.cs:851-868).
    BestiaryEntryDef {
        credit_id: "SkeletonTopHat",
        net_id: 322,
        kind: NpcKind::SkeletonTopHat,
        variant: None,
        quick_unlock: false,
        name: "Skeleton",
    },
    // W4-d38 b6 — BestiaryEntry.Enemy(323)
    // (the automated populator, BestiaryDatabaseNPCsPopulator.cs:851-868).
    BestiaryEntryDef {
        credit_id: "SkeletonAstonaut",
        net_id: 323,
        kind: NpcKind::SkeletonAstonaut,
        variant: None,
        quick_unlock: false,
        name: "Skeleton",
    },
    // W4-d38 b6 — BestiaryEntry.Enemy(324)
    // (the automated populator, BestiaryDatabaseNPCsPopulator.cs:851-868).
    BestiaryEntryDef {
        credit_id: "SkeletonAlien",
        net_id: 324,
        kind: NpcKind::SkeletonAlien,
        variant: None,
        quick_unlock: false,
        name: "Skeleton",
    },
    // W4-d38 b6 — BestiaryEntry.Enemy(328)
    // (the automated populator, BestiaryDatabaseNPCsPopulator.cs:851-868).
    BestiaryEntryDef {
        credit_id: "PumpkingBlade",
        net_id: 328,
        kind: NpcKind::PumpkingBlade,
        variant: None,
        quick_unlock: false,
        name: "Pumpking",
    },
    // W4-d38 b5 — BestiaryEntry.Enemy(302)
    // (the automated populator, BestiaryDatabaseNPCsPopulator.cs:851-868).
    BestiaryEntryDef {
        credit_id: "SlimeMasked",
        net_id: 302,
        kind: NpcKind::SlimeMasked,
        variant: None,
        quick_unlock: false,
        name: "Slime",
    },
    // W4-d38 b5 — BestiaryEntry.Critter(303)
    // (the automated populator, BestiaryDatabaseNPCsPopulator.cs:851-868).
    BestiaryEntryDef {
        credit_id: "BunnySlimed",
        net_id: 303,
        kind: NpcKind::BunnySlimed,
        variant: None,
        quick_unlock: false,
        name: "Bunny",
    },
    // W4-d38 b5 — BestiaryEntry.Enemy(304)
    // (the automated populator, BestiaryDatabaseNPCsPopulator.cs:851-868).
    BestiaryEntryDef {
        credit_id: "HoppinJack",
        net_id: 304,
        kind: NpcKind::HoppinJack,
        variant: None,
        quick_unlock: false,
        name: "Hoppin' Jack",
    },
    // W4-d38 b5 — BestiaryEntry.Enemy(316)
    // (the automated populator, BestiaryDatabaseNPCsPopulator.cs:851-868).
    BestiaryEntryDef {
        credit_id: "Ghost",
        net_id: 316,
        kind: NpcKind::Ghost,
        variant: None,
        quick_unlock: false,
        name: "Ghost",
    },
    // W4-d38 b5 — BestiaryEntry.Enemy(317)
    // (the automated populator, BestiaryDatabaseNPCsPopulator.cs:851-868).
    BestiaryEntryDef {
        credit_id: "DemonEyeOwl",
        net_id: 317,
        kind: NpcKind::DemonEyeOwl,
        variant: None,
        quick_unlock: false,
        name: "Demon Eye",
    },
    // W4-d38 b4 — BestiaryEntry.Critter(297)
    // (the automated populator, BestiaryDatabaseNPCsPopulator.cs:851-868).
    BestiaryEntryDef {
        credit_id: "BirdBlue",
        net_id: 297,
        kind: NpcKind::BirdBlue,
        variant: None,
        quick_unlock: false,
        name: "Blue Jay",
    },
    // W4-d38 b4 — BestiaryEntry.Critter(298)
    // (the automated populator, BestiaryDatabaseNPCsPopulator.cs:851-868).
    BestiaryEntryDef {
        credit_id: "BirdRed",
        net_id: 298,
        kind: NpcKind::BirdRed,
        variant: None,
        quick_unlock: false,
        name: "Cardinal",
    },
    // W4-d38 b4 — BestiaryEntry.Critter(299)
    // (the automated populator, BestiaryDatabaseNPCsPopulator.cs:851-868).
    BestiaryEntryDef {
        credit_id: "Squirrel",
        net_id: 299,
        kind: NpcKind::Squirrel,
        variant: None,
        quick_unlock: false,
        name: "Squirrel",
    },
    // W4-d38 b4 — BestiaryEntry.Critter(300)
    // (the automated populator, BestiaryDatabaseNPCsPopulator.cs:851-868).
    BestiaryEntryDef {
        credit_id: "Mouse",
        net_id: 300,
        kind: NpcKind::Mouse,
        variant: None,
        quick_unlock: false,
        name: "Mouse",
    },
    // W4-d38 b4 — BestiaryEntry.Enemy(301)
    // (the automated populator, BestiaryDatabaseNPCsPopulator.cs:851-868).
    BestiaryEntryDef {
        credit_id: "Raven",
        net_id: 301,
        kind: NpcKind::Raven,
        variant: None,
        quick_unlock: false,
        name: "Raven",
    },
    // W4-d38 b3 — BestiaryEntry.Enemy(292)
    // (the automated populator, BestiaryDatabaseNPCsPopulator.cs:851-868).
    BestiaryEntryDef {
        credit_id: "TacticalSkeleton",
        net_id: 292,
        kind: NpcKind::TacticalSkeleton,
        variant: None,
        quick_unlock: false,
        name: "Tactical Skeleton",
    },
    // W4-d38 b3 — BestiaryEntry.Enemy(293)
    // (the automated populator, BestiaryDatabaseNPCsPopulator.cs:851-868).
    BestiaryEntryDef {
        credit_id: "SkeletonCommando",
        net_id: 293,
        kind: NpcKind::SkeletonCommando,
        variant: None,
        quick_unlock: false,
        name: "Skeleton Commando",
    },
    // W4-d38 b3 — BestiaryEntry.Enemy(294)
    // (the automated populator, BestiaryDatabaseNPCsPopulator.cs:851-868).
    BestiaryEntryDef {
        credit_id: "AngryBonesBig",
        net_id: 294,
        kind: NpcKind::AngryBonesBig,
        variant: None,
        quick_unlock: false,
        name: "Angry Bones",
    },
    // W4-d38 b3 — BestiaryEntry.Enemy(295)
    // (the automated populator, BestiaryDatabaseNPCsPopulator.cs:851-868).
    BestiaryEntryDef {
        credit_id: "AngryBonesBigMuscle",
        net_id: 295,
        kind: NpcKind::AngryBonesBigMuscle,
        variant: None,
        quick_unlock: false,
        name: "Angry Bones",
    },
    // W4-d38 b3 — BestiaryEntry.Enemy(296)
    // (the automated populator, BestiaryDatabaseNPCsPopulator.cs:851-868).
    BestiaryEntryDef {
        credit_id: "AngryBonesBigHelmet",
        net_id: 296,
        kind: NpcKind::AngryBonesBigHelmet,
        variant: None,
        quick_unlock: false,
        name: "Angry Bones",
    },
    // W4-d38 b2 — BestiaryEntry.Enemy(286)
    // (the automated populator, BestiaryDatabaseNPCsPopulator.cs:851-868).
    BestiaryEntryDef {
        credit_id: "DiabolistWhite",
        net_id: 286,
        kind: NpcKind::DiabolistWhite,
        variant: None,
        quick_unlock: false,
        name: "Diabolist",
    },
    // W4-d38 b2 — BestiaryEntry.Enemy(287)
    // (the automated populator, BestiaryDatabaseNPCsPopulator.cs:851-868).
    BestiaryEntryDef {
        credit_id: "BoneLee",
        net_id: 287,
        kind: NpcKind::BoneLee,
        variant: None,
        quick_unlock: false,
        name: "Bone Lee",
    },
    // W4-d38 b2 — BestiaryEntry.Enemy(289)
    // (the automated populator, BestiaryDatabaseNPCsPopulator.cs:851-868).
    BestiaryEntryDef {
        credit_id: "GiantCursedSkull",
        net_id: 289,
        kind: NpcKind::GiantCursedSkull,
        variant: None,
        quick_unlock: false,
        name: "Giant Cursed Skull",
    },
    // W4-d38 b2 — BestiaryEntry.Enemy(290)
    // (the automated populator, BestiaryDatabaseNPCsPopulator.cs:851-868).
    BestiaryEntryDef {
        credit_id: "Paladin",
        net_id: 290,
        kind: NpcKind::Paladin,
        variant: None,
        quick_unlock: false,
        name: "Paladin",
    },
    // W4-d38 b2 — BestiaryEntry.Enemy(291)
    // (the automated populator, BestiaryDatabaseNPCsPopulator.cs:851-868).
    BestiaryEntryDef {
        credit_id: "SkeletonSniper",
        net_id: 291,
        kind: NpcKind::SkeletonSniper,
        variant: None,
        quick_unlock: false,
        name: "Skeleton Sniper",
    },
    // W4-d38 b1 — BestiaryEntry.Enemy(281)
    // (the automated populator, BestiaryDatabaseNPCsPopulator.cs:851-868).
    BestiaryEntryDef {
        credit_id: "RaggedCaster",
        net_id: 281,
        kind: NpcKind::RaggedCaster,
        variant: None,
        quick_unlock: false,
        name: "Ragged Caster",
    },
    // W4-d38 b1 — BestiaryEntry.Enemy(282)
    // (the automated populator, BestiaryDatabaseNPCsPopulator.cs:851-868).
    BestiaryEntryDef {
        credit_id: "RaggedCasterOpenCoat",
        net_id: 282,
        kind: NpcKind::RaggedCasterOpenCoat,
        variant: None,
        quick_unlock: false,
        name: "Ragged Caster",
    },
    // W4-d38 b1 — BestiaryEntry.Enemy(283)
    // (the automated populator, BestiaryDatabaseNPCsPopulator.cs:851-868).
    BestiaryEntryDef {
        credit_id: "Necromancer",
        net_id: 283,
        kind: NpcKind::Necromancer,
        variant: None,
        quick_unlock: false,
        name: "Necromancer",
    },
    // W4-d38 b1 — BestiaryEntry.Enemy(284)
    // (the automated populator, BestiaryDatabaseNPCsPopulator.cs:851-868).
    BestiaryEntryDef {
        credit_id: "NecromancerArmored",
        net_id: 284,
        kind: NpcKind::NecromancerArmored,
        variant: None,
        quick_unlock: false,
        name: "Necromancer",
    },
    // W4-d38 b1 — BestiaryEntry.Enemy(285)
    // (the automated populator, BestiaryDatabaseNPCsPopulator.cs:851-868).
    BestiaryEntryDef {
        credit_id: "DiabolistRed",
        net_id: 285,
        kind: NpcKind::DiabolistRed,
        variant: None,
        quick_unlock: false,
        name: "Diabolist",
    },
    BestiaryEntryDef {
        credit_id: "ForceBubble",
        net_id: 384,
        kind: NpcKind::ForceBubble,
        variant: None,
        quick_unlock: false,
        name: "Force Bubble",
    },
    // W4-d37 — the NPCID 211-280 band's own bestiary entries (the automated
    // populator registers every non-Hide, non-town id; the names ride
    // `Lang.GetNPCName`, en-US NPCs.json). The three Hide=true ids (230 the
    // goldfish walker, 236 the jungle creeper, 239 the blood crawler) fold
    // onto their visible twin's credit in `entry_for_npc`/`try_entry_for_npc`
    // (the SleepingAngler precedent) — no own row here.
    BestiaryEntryDef { credit_id: "CochinealBeetle", net_id: 217, kind: NpcKind::CochinealBeetle, variant: None, quick_unlock: false, name: "Cochineal Beetle" },
    BestiaryEntryDef { credit_id: "CyanBeetle", net_id: 218, kind: NpcKind::CyanBeetle, variant: None, quick_unlock: false, name: "Cyan Beetle" },
    BestiaryEntryDef { credit_id: "LacBeetle", net_id: 219, kind: NpcKind::LacBeetle, variant: None, quick_unlock: false, name: "Lac Beetle" },
    BestiaryEntryDef { credit_id: "SeaSnail", net_id: 220, kind: NpcKind::SeaSnail, variant: None, quick_unlock: false, name: "Sea Snail" },
    BestiaryEntryDef { credit_id: "Squid", net_id: 221, kind: NpcKind::Squid, variant: None, quick_unlock: false, name: "Squid" },
    BestiaryEntryDef { credit_id: "FlyingFish", net_id: 224, kind: NpcKind::FlyingFish, variant: None, quick_unlock: false, name: "Flying Fish" },
    BestiaryEntryDef { credit_id: "FlyingSnake", net_id: 226, kind: NpcKind::FlyingSnake, variant: None, quick_unlock: false, name: "Flying Snake" },
    BestiaryEntryDef { credit_id: "Pirate", net_id: 229, kind: NpcKind::Pirate, variant: None, quick_unlock: false, name: "Pirate" },
    BestiaryEntryDef { credit_id: "HornetFatty", net_id: 231, kind: NpcKind::HornetFatty, variant: None, quick_unlock: false, name: "Hornet" },
    BestiaryEntryDef { credit_id: "HornetHoney", net_id: 232, kind: NpcKind::HornetHoney, variant: None, quick_unlock: false, name: "Hornet" },
    BestiaryEntryDef { credit_id: "HornetLeafy", net_id: 233, kind: NpcKind::HornetLeafy, variant: None, quick_unlock: false, name: "Hornet" },
    BestiaryEntryDef { credit_id: "HornetSpikey", net_id: 234, kind: NpcKind::HornetSpikey, variant: None, quick_unlock: false, name: "Hornet" },
    BestiaryEntryDef { credit_id: "HornetStingy", net_id: 235, kind: NpcKind::HornetStingy, variant: None, quick_unlock: false, name: "Hornet" },
    BestiaryEntryDef { credit_id: "JungleCreeperWall", net_id: 237, kind: NpcKind::JungleCreeperWall, variant: None, quick_unlock: false, name: "Jungle Creeper" },
    BestiaryEntryDef { credit_id: "BlackRecluseWall", net_id: 238, kind: NpcKind::BlackRecluseWall, variant: None, quick_unlock: false, name: "Black Recluse" },
    BestiaryEntryDef { credit_id: "BloodCrawlerWall", net_id: 240, kind: NpcKind::BloodCrawlerWall, variant: None, quick_unlock: false, name: "Blood Crawler" },
    BestiaryEntryDef { credit_id: "BloodFeeder", net_id: 241, kind: NpcKind::BloodFeeder, variant: None, quick_unlock: false, name: "Blood Feeder" },
    BestiaryEntryDef { credit_id: "BloodJelly", net_id: 242, kind: NpcKind::BloodJelly, variant: None, quick_unlock: false, name: "Blood Jelly" },
    BestiaryEntryDef { credit_id: "IceGolem", net_id: 243, kind: NpcKind::IceGolem, variant: None, quick_unlock: false, name: "Ice Golem" },
    BestiaryEntryDef { credit_id: "RainbowSlime", net_id: 244, kind: NpcKind::RainbowSlime, variant: None, quick_unlock: false, name: "Rainbow Slime" },
    BestiaryEntryDef { credit_id: "Eyezor", net_id: 251, kind: NpcKind::Eyezor, variant: None, quick_unlock: false, name: "Eyezor" },
    BestiaryEntryDef { credit_id: "Parrot", net_id: 252, kind: NpcKind::Parrot, variant: None, quick_unlock: false, name: "Parrot" },
    BestiaryEntryDef { credit_id: "Reaper", net_id: 253, kind: NpcKind::Reaper, variant: None, quick_unlock: false, name: "Reaper" },
    BestiaryEntryDef { credit_id: "ZombieMushroom", net_id: 254, kind: NpcKind::ZombieMushroom, variant: None, quick_unlock: false, name: "Spore Zombie" },
    BestiaryEntryDef { credit_id: "ZombieMushroomHat", net_id: 255, kind: NpcKind::ZombieMushroomHat, variant: None, quick_unlock: false, name: "Spore Zombie" },
    BestiaryEntryDef { credit_id: "FungoFish", net_id: 256, kind: NpcKind::FungoFish, variant: None, quick_unlock: false, name: "Fungo Fish" },
    BestiaryEntryDef { credit_id: "AnomuraFungus", net_id: 257, kind: NpcKind::AnomuraFungus, variant: None, quick_unlock: false, name: "Anomura Fungus" },
    BestiaryEntryDef { credit_id: "MushiLadybug", net_id: 258, kind: NpcKind::MushiLadybug, variant: None, quick_unlock: false, name: "Mushi Ladybug" },
    BestiaryEntryDef { credit_id: "FungiBulb", net_id: 259, kind: NpcKind::FungiBulb, variant: None, quick_unlock: false, name: "Fungi Bulb" },
    BestiaryEntryDef { credit_id: "GiantFungiBulb", net_id: 260, kind: NpcKind::GiantFungiBulb, variant: None, quick_unlock: false, name: "Giant Fungi Bulb" },
    BestiaryEntryDef { credit_id: "IchorSticker", net_id: 268, kind: NpcKind::IchorSticker, variant: None, quick_unlock: false, name: "Ichor Sticker" },
    BestiaryEntryDef { credit_id: "RustyArmoredBonesAxe", net_id: 269, kind: NpcKind::RustyArmoredBonesAxe, variant: None, quick_unlock: false, name: "Rusty Armored Bones" },
    BestiaryEntryDef { credit_id: "RustyArmoredBonesFlail", net_id: 270, kind: NpcKind::RustyArmoredBonesFlail, variant: None, quick_unlock: false, name: "Rusty Armored Bones" },
    BestiaryEntryDef { credit_id: "RustyArmoredBonesSword", net_id: 271, kind: NpcKind::RustyArmoredBonesSword, variant: None, quick_unlock: false, name: "Rusty Armored Bones" },
    BestiaryEntryDef { credit_id: "RustyArmoredBonesSwordNoArmor", net_id: 272, kind: NpcKind::RustyArmoredBonesSwordNoArmor, variant: None, quick_unlock: false, name: "Rusty Armored Bones" },
    BestiaryEntryDef { credit_id: "BlueArmoredBones", net_id: 273, kind: NpcKind::BlueArmoredBones, variant: None, quick_unlock: false, name: "Blue Armored Bones" },
    BestiaryEntryDef { credit_id: "BlueArmoredBonesMace", net_id: 274, kind: NpcKind::BlueArmoredBonesMace, variant: None, quick_unlock: false, name: "Blue Armored Bones" },
    BestiaryEntryDef { credit_id: "BlueArmoredBonesNoPants", net_id: 275, kind: NpcKind::BlueArmoredBonesNoPants, variant: None, quick_unlock: false, name: "Blue Armored Bones" },
    BestiaryEntryDef { credit_id: "BlueArmoredBonesSword", net_id: 276, kind: NpcKind::BlueArmoredBonesSword, variant: None, quick_unlock: false, name: "Blue Armored Bones" },
    BestiaryEntryDef { credit_id: "HellArmoredBones", net_id: 277, kind: NpcKind::HellArmoredBones, variant: None, quick_unlock: false, name: "Hell Armored Bones" },
    BestiaryEntryDef { credit_id: "HellArmoredBonesSpikeShield", net_id: 278, kind: NpcKind::HellArmoredBonesSpikeShield, variant: None, quick_unlock: false, name: "Hell Armored Bones" },
    BestiaryEntryDef { credit_id: "HellArmoredBonesMace", net_id: 279, kind: NpcKind::HellArmoredBonesMace, variant: None, quick_unlock: false, name: "Hell Armored Bones" },
    BestiaryEntryDef { credit_id: "HellArmoredBonesSword", net_id: 280, kind: NpcKind::HellArmoredBonesSword, variant: None, quick_unlock: false, name: "Hell Armored Bones" },
    // W4-d40: the new kinds' own bestiary entries (`BestiaryDatabaseNPCsPopulator`).
    BestiaryEntryDef {
        credit_id: "CultistTablet",
        net_id: 437,
        kind: NpcKind::CultistTablet,
        variant: None,
        quick_unlock: false,
        name: "Mysterious Tablet",
    },
    BestiaryEntryDef {
        credit_id: "CultistDevote",
        net_id: 438,
        kind: NpcKind::CultistDevote,
        variant: None,
        quick_unlock: false,
        name: "Lunatic Devotee",
    },
    BestiaryEntryDef {
        credit_id: "CultistBossClone",
        net_id: 440,
        kind: NpcKind::CultistBossClone,
        variant: None,
        quick_unlock: false,
        name: "Lunatic Cultist",
    },
    // W4-d40: the new kinds' own bestiary entries (`BestiaryDatabaseNPCsPopulator`).
    // The snail wall-crawler family + its shimmer target (W4-c9): the
    // populator registers all four (`BestiaryDatabaseNPCsPopulator.cs`'s
    // `AddEmptyEntries_CrittersAndEnemies_Automated` — the
    // `CountsAsACritter` set — plus the `FindEntryByNPCID(359)`/`(677)`
    // cavern-spawn info rows).
    BestiaryEntryDef {
        credit_id: "Snail",
        net_id: 359,
        kind: NpcKind::Snail,
        variant: None,
        quick_unlock: false,
        name: "Snail",
    },
    BestiaryEntryDef {
        credit_id: "GlowingSnail",
        net_id: 360,
        kind: NpcKind::GlowingSnail,
        variant: None,
        quick_unlock: false,
        name: "Glowing Snail",
    },
    BestiaryEntryDef {
        credit_id: "MagmaSnail",
        net_id: 655,
        kind: NpcKind::MagmaSnail,
        variant: None,
        quick_unlock: false,
        name: "Magma Snail",
    },
    // E8 merge: the Shimmerfly 677 entry is c7's above (the dup drop).
    // The gold critters (W4-c9): the populator registers each
    // (`BestiaryDatabaseNPCsPopulator.cs`'s
    // `AddEmptyEntries_CrittersAndEnemies_Automated` — the
    // `CountsAsACritter` set). The names ride the lang table
    // (`lang_tables.rs`'s 442/446/447 rows).
    // E8 merge: the GoldBird/GoldGrasshopper/GoldMouse/GoldBunny entries
    // are d40's above (the dup drops); the GoldButterfly 444 entry is d40's
    // own (it stays).
    BestiaryEntryDef {
        credit_id: "GoldButterfly",
        net_id: 444,
        kind: NpcKind::GoldButterfly,
        variant: None,
        quick_unlock: false,
        name: "Gold Butterfly",
    },
    // E8 merge: the GoldFrog 445 + the stray GoldGrasshopper 446 dups
    // dropped (d40's entries above own them).
    // W4-d40: the new kinds' own bestiary entries (`BestiaryDatabaseNPCsPopulator`).
    BestiaryEntryDef {
        credit_id: "GoldMouse",
        net_id: 447,
        kind: NpcKind::GoldMouse,
        variant: None,
        quick_unlock: false,
        name: "Gold Mouse",
    },
    // W4-c8 — the crawling-critter family's own entries (the populator
    // registers them all; `NPCID.Search.GetName` credits, the en-US
    // NPCs.json display names). The digger 375 is Hide=true
    // (`NPCID.cs:3714-3718`) — its entry exists (the ForceBubble
    // precedent) but `entry_for_npc` folds it onto the TruffleWorm.
    // W4-c10 — the waterfowl family's own entries (the populator
    // registers all eight: `FindEntryByNPCID(362/363/364/365/602/603/
    // 608/609).Info`, `BestiaryDatabaseNPCsPopulator.cs:1191-1266`;
    // `NPCID.Search.GetName` credits). The display names ride the
    // localization rows: Duck2/DuckWhite/DuckWhite2 all read "Duck",
    // Seagull2 "Seagull", Grebe2 "Grebe"
    // (`Terraria.Localization.Content.en-US.NPCs.json`).
    BestiaryEntryDef {
        credit_id: "Duck",
        net_id: 362,
        kind: NpcKind::Duck,
        variant: None,
        quick_unlock: false,
        name: "Duck",
    },
    BestiaryEntryDef {
        credit_id: "Duck2",
        net_id: 363,
        kind: NpcKind::Duck2,
        variant: None,
        quick_unlock: false,
        name: "Duck",
    },
    BestiaryEntryDef {
        credit_id: "DuckWhite",
        net_id: 364,
        kind: NpcKind::DuckWhite,
        variant: None,
        quick_unlock: false,
        name: "Duck",
    },
    BestiaryEntryDef {
        credit_id: "DuckWhite2",
        net_id: 365,
        kind: NpcKind::DuckWhite2,
        variant: None,
        quick_unlock: false,
        name: "Duck",
    },
    // W4-c10 roster tail — the twelve critters' own entries (the
    // populator registers all twelve: `FindEntryByNPCID(357/442-448/
    // 484-487).Info`, `BestiaryDatabaseNPCsPopulator.cs:890-1886`;
    // `NPCID.Search.GetName` credits). The display names ride the
    // en-US NPCs.json rows ("Gold Bird" … "Enchanted Nightcrawler").
    BestiaryEntryDef {
        credit_id: "Worm",
        net_id: 357,
        kind: NpcKind::Worm,
        variant: None,
        quick_unlock: false,
        name: "Worm",
    },
    BestiaryEntryDef {
        credit_id: "TruffleWorm",
        net_id: 374,
        kind: NpcKind::TruffleWorm,
        variant: None,
        quick_unlock: false,
        name: "Truffle Worm",
    },
    BestiaryEntryDef {
        credit_id: "TruffleWormDigger",
        net_id: 375,
        kind: NpcKind::TruffleWormDigger,
        variant: None,
        quick_unlock: false,
        name: "Truffle Worm",
    },
    // E8 merge: the GoldWorm 448 entry is d40's above (the dup drop).
    // W4-d40: the new kinds' own bestiary entries (`BestiaryDatabaseNPCsPopulator`).
    BestiaryEntryDef {
        credit_id: "SkeletonMerchant",
        net_id: 453,
        kind: NpcKind::SkeletonMerchant,
        variant: None,
        quick_unlock: false,
        name: "Skeleton Merchant",
    },
    BestiaryEntryDef {
        credit_id: "CultistDragonHead",
        net_id: 454,
        kind: NpcKind::CultistDragonHead,
        variant: None,
        quick_unlock: false,
        name: "Phantasm Dragon",
    },
    // W4-d40: the new kinds' own bestiary entries (`BestiaryDatabaseNPCsPopulator`).
    BestiaryEntryDef {
        credit_id: "Butcher",
        net_id: 460,
        kind: NpcKind::Butcher,
        variant: None,
        quick_unlock: false,
        name: "Butcher",
    },
    BestiaryEntryDef {
        credit_id: "CreatureFromTheDeep",
        net_id: 461,
        kind: NpcKind::CreatureFromTheDeep,
        variant: None,
        quick_unlock: false,
        name: "Creature from the Deep",
    },
    // W4-d40: the new kinds' own bestiary entries (`BestiaryDatabaseNPCsPopulator`).
    BestiaryEntryDef {
        credit_id: "Fritz",
        net_id: 462,
        kind: NpcKind::Fritz,
        variant: None,
        quick_unlock: false,
        name: "Fritz",
    },
    BestiaryEntryDef {
        credit_id: "Nailhead",
        net_id: 463,
        kind: NpcKind::Nailhead,
        variant: None,
        quick_unlock: false,
        name: "Nailhead",
    },
    BestiaryEntryDef {
        credit_id: "CrimsonBunny",
        net_id: 464,
        kind: NpcKind::CrimsonBunny,
        variant: None,
        quick_unlock: false,
        name: "Vicious Bunny",
    },
    // (merge-side drop, d37 tables: the CrimsonGoldfish 465 entry belongs to
    // c4's kept impl - its "Vicious Goldfish" row lands at the small-lanes
    // splice; d40's dup port dropped here.)
    BestiaryEntryDef {
        credit_id: "Psycho",
        net_id: 466,
        kind: NpcKind::Psycho,
        variant: None,
        quick_unlock: false,
        name: "Psycho",
    },
    // W4-d40: the new kinds' own bestiary entries (`BestiaryDatabaseNPCsPopulator`).
    BestiaryEntryDef {
        credit_id: "DeadlySphere",
        net_id: 467,
        kind: NpcKind::DeadlySphere,
        variant: None,
        quick_unlock: false,
        name: "Deadly Sphere",
    },
    BestiaryEntryDef {
        credit_id: "DrManFly",
        net_id: 468,
        kind: NpcKind::DrManFly,
        variant: None,
        quick_unlock: false,
        name: "Dr. Man Fly",
    },
    BestiaryEntryDef {
        credit_id: "ThePossessed",
        net_id: 469,
        kind: NpcKind::ThePossessed,
        variant: None,
        quick_unlock: false,
        name: "The Possessed",
    },
    BestiaryEntryDef {
        credit_id: "CrimsonPenguin",
        net_id: 470,
        kind: NpcKind::CrimsonPenguin,
        variant: None,
        quick_unlock: false,
        name: "Vicious Penguin",
    },
    BestiaryEntryDef {
        credit_id: "ShadowFlameApparition",
        net_id: 472,
        kind: NpcKind::ShadowFlameApparition,
        variant: None,
        quick_unlock: false,
        name: "Shadowflame Apparition",
    },
    // W4-d40: the new kinds' own bestiary entries (`BestiaryDatabaseNPCsPopulator`).
    BestiaryEntryDef {
        credit_id: "BigMimicCorruption",
        net_id: 473,
        kind: NpcKind::BigMimicCorruption,
        variant: None,
        quick_unlock: false,
        name: "Corrupt Mimic",
    },
    BestiaryEntryDef {
        credit_id: "BigMimicCrimson",
        net_id: 474,
        kind: NpcKind::BigMimicCrimson,
        variant: None,
        quick_unlock: false,
        name: "Crimson Mimic",
    },
    BestiaryEntryDef {
        credit_id: "BigMimicHallow",
        net_id: 475,
        kind: NpcKind::BigMimicHallow,
        variant: None,
        quick_unlock: false,
        name: "Hallowed Mimic",
    },
    BestiaryEntryDef {
        credit_id: "BigMimicJungle",
        net_id: 476,
        kind: NpcKind::BigMimicJungle,
        variant: None,
        quick_unlock: false,
        name: "Jungle Mimic",
    },
    BestiaryEntryDef {
        credit_id: "Mothron",
        net_id: 477,
        kind: NpcKind::Mothron,
        variant: None,
        quick_unlock: false,
        name: "Mothron",
    },
    // W4-d40 seal: the MothronEgg 478 is `Hide = true`
    // (`NPCID.cs:4405-4410`) so the populator's exclusion set skips it —
    // but `kind_from_net_id` still needs a row to resolve (the g8
    // VampireBat 158 precedent, its "still needs a row to resolve"
    // comment + own row; the g61 research/273 §3 478-resolution-gap fix
    // PUBLISHED FOR d40). The credit fold stays (`entry_for_npc`'s
    // `MothronEgg => Mothron` arm — the vanilla kills credit Mothron);
    // this row only makes the net id resolve, so a saved/remote egg
    // instantiates the egg, not the moth.
    BestiaryEntryDef {
        credit_id: "MothronEgg",
        net_id: 478,
        kind: NpcKind::MothronEgg,
        variant: None,
        quick_unlock: false,
        name: "Mothron Egg",
    },
    BestiaryEntryDef {
        credit_id: "MothronSpawn",
        net_id: 479,
        kind: NpcKind::MothronSpawn,
        variant: None,
        quick_unlock: false,
        name: "Baby Mothron",
    },
    BestiaryEntryDef {
        credit_id: "Medusa",
        net_id: 480,
        kind: NpcKind::Medusa,
        variant: None,
        quick_unlock: false,
        name: "Medusa",
    },
    BestiaryEntryDef {
        credit_id: "GreekSkeleton",
        net_id: 481,
        kind: NpcKind::GreekSkeleton,
        variant: None,
        quick_unlock: false,
        name: "Hoplite",
    },
    BestiaryEntryDef {
        credit_id: "GraniteGolem",
        net_id: 482,
        kind: NpcKind::GraniteGolem,
        variant: None,
        quick_unlock: false,
        name: "Granite Golem",
    },
    // W4-d40: the new kinds' own bestiary entries (`BestiaryDatabaseNPCsPopulator`).
    BestiaryEntryDef {
        credit_id: "GraniteFlyer",
        net_id: 483,
        kind: NpcKind::GraniteFlyer,
        variant: None,
        quick_unlock: false,
        name: "Granite Elemental",
    },
    // W4-c10 merge: the gold entries RESTORED (the c8/c9 merges dropped
    // them thinking d40 owned them - but d40's were the same entries
    // already in main; the c10 insert re-lands the canonical rows).
    BestiaryEntryDef {
        credit_id: "GoldBird",
        net_id: 442,
        kind: NpcKind::GoldBird,
        variant: None,
        quick_unlock: false,
        name: "Gold Bird",
    },
    BestiaryEntryDef {
        credit_id: "GoldBunny",
        net_id: 443,
        kind: NpcKind::GoldBunny,
        variant: None,
        quick_unlock: false,
        name: "Gold Bunny",
    },
    BestiaryEntryDef {
        credit_id: "GoldFrog",
        net_id: 445,
        kind: NpcKind::GoldFrog,
        variant: None,
        quick_unlock: false,
        name: "Gold Frog",
    },
    BestiaryEntryDef {
        credit_id: "GoldGrasshopper",
        net_id: 446,
        kind: NpcKind::GoldGrasshopper,
        variant: None,
        quick_unlock: false,
        name: "Gold Grasshopper",
    },
    BestiaryEntryDef {
        credit_id: "GoldWorm",
        net_id: 448,
        kind: NpcKind::GoldWorm,
        variant: None,
        quick_unlock: false,
        name: "Gold Worm",
    },
    BestiaryEntryDef {
        credit_id: "EnchantedNightcrawler",
        net_id: 484,
        kind: NpcKind::EnchantedNightcrawler,
        variant: None,
        quick_unlock: false,
        name: "Enchanted Nightcrawler",
    },
    BestiaryEntryDef {
        credit_id: "Grubby",
        net_id: 485,
        kind: NpcKind::Grubby,
        variant: None,
        quick_unlock: false,
        name: "Grubby",
    },
    BestiaryEntryDef {
        credit_id: "Sluggy",
        net_id: 486,
        kind: NpcKind::Sluggy,
        variant: None,
        quick_unlock: false,
        name: "Sluggy",
    },
    BestiaryEntryDef {
        credit_id: "Buggy",
        net_id: 487,
        kind: NpcKind::Buggy,
        variant: None,
        quick_unlock: false,
        name: "Buggy",
    },
    // W4-d42 - the NPCID 578-582 roster's own credits (the
    // Hide=true kinds fold in `entry_for_npc` - the SleepingGolfer row).
    BestiaryEntryDef {
        credit_id: "DD2LightningBugT3",
        net_id: 578,
        kind: NpcKind::DD2LightningBug,
        variant: None,
        quick_unlock: false,
        name: "Etherian Lightning Bug",
    },
    BestiaryEntryDef {
        credit_id: "WalkingAntlion",
        net_id: 580,
        kind: NpcKind::WalkingAntlion,
        variant: None,
        quick_unlock: false,
        name: "Antlion Charger",
    },
    BestiaryEntryDef {
        credit_id: "LarvaeAntlion",
        net_id: 582,
        kind: NpcKind::LarvaeAntlion,
        variant: None,
        quick_unlock: false,
        name: "Antlion Larva",
    },
    // W4-d42 - the NPCID 583-587 roster's own credits (the
    // Hide=true kinds fold in `entry_for_npc` - the SleepingGolfer row).
    BestiaryEntryDef {
        credit_id: "ZombieMerman",
        net_id: 586,
        kind: NpcKind::ZombieMerman,
        variant: None,
        quick_unlock: false,
        name: "Zombie Merman",
    },
    // W4-d42 - the NPCID 590-594 roster's own credits (the
    // Hide=true kinds fold in `entry_for_npc` - the SleepingGolfer row).
    BestiaryEntryDef {
        credit_id: "TorchZombie",
        net_id: 590,
        kind: NpcKind::TorchZombie,
        variant: None,
        quick_unlock: false,
        name: "Zombie",
    },
    // W4-c4 merge (148 item 5): the GoldGoldfishWalker's interim OWN
    // entry RETIRED - its vanilla credit folds onto the GoldGoldfish
    // (592, `ContentSamples.cs:1049`) now that c4's 592 entry landed;
    // both resolvers carry the fold arm below.
    // W4-c4 (the style-44 swimmers): the four kinds' own entries - the
    // populator attaches info to all four
    // (`BestiaryDatabaseNPCsPopulator.cs:1133`/`:1523`/`:629`/`:1164`).
    // The display names ride `en-US NPCs.json` (581 is "Antlion
    // Swarmer", 509 "Giant Antlion Swarmer", 587 "Wandering Eye Fish" —
    // NOT the credit-id spellings).
    // E8 merge: the FlyingFish 224 entry is d37's row above (dup drop).
    BestiaryEntryDef {
        credit_id: "GiantFlyingAntlion",
        net_id: 509,
        kind: NpcKind::GiantFlyingAntlion,
        variant: None,
        quick_unlock: false,
        name: "Giant Antlion Swarmer",
    },
    BestiaryEntryDef {
        credit_id: "FlyingAntlion",
        net_id: 581,
        kind: NpcKind::FlyingAntlion,
        variant: None,
        quick_unlock: false,
        name: "Antlion Swarmer",
    },
    BestiaryEntryDef {
        credit_id: "EyeballFlyingFish",
        net_id: 587,
        kind: NpcKind::EyeballFlyingFish,
        variant: None,
        quick_unlock: false,
        name: "Wandering Eye Fish",
    },
    // W4-c4 (the style-16 swimmers): the nine kinds' own entries — the
    // populator attaches info to all of them
    // (`BestiaryDatabaseNPCsPopulator.cs:1088`/`:1095`/`:1119`/`:1105`/
    // `:1107`/`:1108`/`:1298`/`:2025`/`:1607`). The display names ride
    // `en-US NPCs.json` (465 is "Vicious Goldfish" — NOT the credit-id
    // spelling "CrimsonGoldfish"). The GoldGoldfish 592 keeps its OWN
    // entry (its kill credit is "GoldGoldfish"); only the UI provider
    // folds it onto the Goldfish 55 (`:706`'s GoldCritterUICollectionInfoProvider).
    // E8 merge: the CorruptGoldfish 57 entry is d34's row above.
    BestiaryEntryDef {
        credit_id: "Arapaima",
        net_id: 157,
        kind: NpcKind::Arapaima,
        variant: None,
        quick_unlock: false,
        name: "Arapaima",
    },
    // E8 merge: the BloodFeeder 241 entry is d37's row above.
    BestiaryEntryDef {
        credit_id: "CrimsonGoldfish",
        net_id: 465,
        kind: NpcKind::CrimsonGoldfish,
        variant: None,
        quick_unlock: false,
        name: "Vicious Goldfish",
    },
    BestiaryEntryDef {
        credit_id: "GoldGoldfish",
        net_id: 592,
        kind: NpcKind::GoldGoldfish,
        variant: None,
        quick_unlock: false,
        name: "Gold Goldfish",
    },
    // W4-d42 - the NPCID 595-599 roster's own credits (the
    // Hide=true kinds fold in `entry_for_npc` - the SleepingGolfer row).
    // W4-d42 - the NPCID 600-604 roster's own credits (the
    // Hide=true kinds fold in `entry_for_npc` - the SleepingGolfer row).
    BestiaryEntryDef {
        credit_id: "Seagull",
        net_id: 602,
        kind: NpcKind::Seagull,
        variant: None,
        quick_unlock: false,
        name: "Seagull",
    },
    // W4-d42 - the NPCID 605-609 roster's own credits (the
    // Hide=true kinds fold in `entry_for_npc` - the SleepingGolfer row).
    BestiaryEntryDef {
        credit_id: "Grebe",
        net_id: 608,
        kind: NpcKind::Grebe,
        variant: None,
        quick_unlock: false,
        name: "Grebe",
    },
    // W4-d42 - the NPCID 610-614 roster's own credits (the
    // Hide=true kinds fold in `entry_for_npc` - the SleepingGolfer row).
    BestiaryEntryDef {
        credit_id: "Rat",
        net_id: 610,
        kind: NpcKind::Rat,
        variant: None,
        quick_unlock: false,
        name: "Rat",
    },
    BestiaryEntryDef {
        credit_id: "Owl",
        net_id: 611,
        kind: NpcKind::Owl,
        variant: None,
        quick_unlock: false,
        name: "Owl",
    },
    BestiaryEntryDef {
        credit_id: "WaterStrider",
        net_id: 612,
        kind: NpcKind::WaterStrider,
        variant: None,
        quick_unlock: false,
        name: "Water Strider",
    },
    BestiaryEntryDef {
        credit_id: "GoldWaterStrider",
        net_id: 613,
        kind: NpcKind::GoldWaterStrider,
        variant: None,
        quick_unlock: false,
        name: "Gold Water Strider",
    },
    BestiaryEntryDef {
        credit_id: "ExplosiveBunny",
        net_id: 614,
        kind: NpcKind::ExplosiveBunny,
        variant: None,
        quick_unlock: false,
        name: "Explosive Bunny",
    },
    // W4-d42 - the NPCID 615-619 roster's own credits (the
    // Hide=true kinds fold in `entry_for_npc` - the SleepingGolfer row).
    BestiaryEntryDef {
        credit_id: "Turtle",
        net_id: 616,
        kind: NpcKind::Turtle,
        variant: None,
        quick_unlock: false,
        name: "Turtle",
    },
    BestiaryEntryDef {
        credit_id: "TurtleJungle",
        net_id: 617,
        kind: NpcKind::TurtleJungle,
        variant: None,
        quick_unlock: false,
        name: "Jungle Turtle",
    },
    BestiaryEntryDef {
        credit_id: "BloodNautilus",
        net_id: 618,
        kind: NpcKind::BloodNautilus,
        variant: None,
        quick_unlock: false,
        name: "Dreadnautilus",
    },
    BestiaryEntryDef {
        credit_id: "BloodSquid",
        net_id: 619,
        kind: NpcKind::BloodSquid,
        variant: None,
        quick_unlock: false,
        name: "Blood Squid",
    },
    // W4-d42 - the NPCID 620-624 roster's own credits (the
    // Hide=true kinds fold in `entry_for_npc` - the SleepingGolfer row).
    BestiaryEntryDef {
        credit_id: "GoblinShark",
        net_id: 620,
        kind: NpcKind::GoblinShark,
        variant: None,
        quick_unlock: false,
        name: "Hemogoblin Shark",
    },
    BestiaryEntryDef {
        credit_id: "BloodEelHead",
        net_id: 621,
        kind: NpcKind::BloodEelHead,
        variant: None,
        quick_unlock: false,
        name: "Blood Eel",
    },
    BestiaryEntryDef {
        credit_id: "Gnome",
        net_id: 624,
        kind: NpcKind::Gnome,
        variant: None,
        quick_unlock: false,
        name: "Gnome",
    },
    // W4-d42 - the NPCID 625-629 roster's own credits (the
    // Hide=true kinds fold in `entry_for_npc` - the SleepingGolfer row).
    BestiaryEntryDef {
        credit_id: "SeaTurtle",
        net_id: 625,
        kind: NpcKind::SeaTurtle,
        variant: None,
        quick_unlock: false,
        name: "Sea Turtle",
    },
    BestiaryEntryDef {
        credit_id: "Seahorse",
        net_id: 626,
        kind: NpcKind::Seahorse,
        variant: None,
        quick_unlock: false,
        name: "Seahorse",
    },
    BestiaryEntryDef {
        credit_id: "GoldSeahorse",
        net_id: 627,
        kind: NpcKind::GoldSeahorse,
        variant: None,
        quick_unlock: false,
        name: "Gold Seahorse",
    },
    BestiaryEntryDef {
        credit_id: "Dandelion",
        net_id: 628,
        kind: NpcKind::Dandelion,
        variant: None,
        quick_unlock: false,
        name: "Angry Dandelion",
    },
    BestiaryEntryDef {
        credit_id: "IceMimic",
        net_id: 629,
        kind: NpcKind::IceMimic,
        variant: None,
        quick_unlock: false,
        name: "Ice Mimic",
    },
    // W4-d42 - the NPCID 630-630 roster's own credits (the
    // Hide=true kinds fold in `entry_for_npc` - the SleepingGolfer row).
    BestiaryEntryDef {
        credit_id: "BloodMummy",
        net_id: 630,
        kind: NpcKind::BloodMummy,
        variant: None,
        quick_unlock: false,
        name: "Blood Mummy",
    },
    // W4-d35: the 33 data-wave kinds' entries (73..140), id order.
    // The Hide kinds (96/97/99/100/112) have NO own row - their
    // credit folds onto the head (try_entry_for_npc/entry_for_npc).
    BestiaryEntryDef {
        credit_id: "GoblinScout",
        net_id: 73,
        kind: NpcKind::GoblinScout,
        variant: None,
        quick_unlock: false,
        name: "Goblin Scout",
    },
    BestiaryEntryDef {
        credit_id: "Bird",
        net_id: 74,
        kind: NpcKind::Bird,
        variant: None,
        quick_unlock: false,
        name: "Bird",
    // W4-g8 lane 1 batch 1 — the d22-d33 machines' kinds. The automated
    // populator registers every non-town, non-Hide enemy by its vanilla
    // net id (`BestiaryDatabaseNPCsPopulator.cs:851-868`); the Spike Ball
    // 70 is `Hide = true` (`NPCID.cs:1711-1715`) so it folds onto no entry
    // of its own — but `entry_for_npc` still needs a row to resolve, so it
    // carries one here (the BlazingWheel 72 precedent, also Hide=true).
BestiaryEntryDef {
        credit_id: "PinkJellyfish",
        net_id: 64,
        kind: NpcKind::PinkJellyfish,
        variant: None,
        quick_unlock: false,
        name: "Pink Jellyfish",
    },
    BestiaryEntryDef {
        credit_id: "Antlion",
        net_id: 69,
        kind: NpcKind::Antlion,
        variant: None,
        quick_unlock: false,
        name: "Antlion",
    },
    BestiaryEntryDef {
        credit_id: "SpikeBall",
        net_id: 70,
        kind: NpcKind::SpikeBall,
        variant: None,
        quick_unlock: false,
        name: "Spike Ball",
    }
    },
    BestiaryEntryDef {
        credit_id: "Pixie",
        net_id: 75,
        kind: NpcKind::Pixie,
        variant: None,
        quick_unlock: false,
        name: "Pixie",
    },
    BestiaryEntryDef {
        credit_id: "ArmoredSkeleton",
        net_id: 77,
        kind: NpcKind::ArmoredSkeleton,
        variant: None,
        quick_unlock: false,
        name: "Armored Skeleton",
    },
    BestiaryEntryDef {
        credit_id: "Mummy",
        net_id: 78,
        kind: NpcKind::Mummy,
        variant: None,
        quick_unlock: false,
        name: "Mummy",
    },
    BestiaryEntryDef {
        credit_id: "DarkMummy",
        net_id: 79,
        kind: NpcKind::DarkMummy,
        variant: None,
        quick_unlock: false,
        name: "Dark Mummy",
    },
    BestiaryEntryDef {
        credit_id: "LightMummy",
        net_id: 80,
        kind: NpcKind::LightMummy,
        variant: None,
        quick_unlock: false,
        name: "Light Mummy",
    },
    BestiaryEntryDef {
        credit_id: "Wraith",
        net_id: 82,
        kind: NpcKind::Wraith,
        variant: None,
        quick_unlock: false,
        name: "Wraith",
    },
    BestiaryEntryDef {
        credit_id: "CursedHammer",
        net_id: 83,
        kind: NpcKind::CursedHammer,
        variant: None,
        quick_unlock: false,
        name: "Cursed Hammer",
    },
    BestiaryEntryDef {
        credit_id: "EnchantedSword",
        net_id: 84,
        kind: NpcKind::EnchantedSword,
        variant: None,
        quick_unlock: false,
        name: "Enchanted Sword",
    },
    BestiaryEntryDef {
        credit_id: "Mimic",
        net_id: 85,
        kind: NpcKind::Mimic,
        variant: None,
        quick_unlock: false,
        name: "Mimic",
    },
    // W4-d25 STYLE-26 wiring: the Unicorn row mirrors W4-d35 batch C
    // (6ed8c22f) verbatim (the merge keep-one); the Wolf's row rides
    // `BestiaryDatabaseNPCsPopulator.cs:1513` (NightTime + Snow tags)
    // and the en-US `NPCs.json:93` name.
    BestiaryEntryDef {
        credit_id: "Unicorn",
        net_id: 86,
        kind: NpcKind::Unicorn,
        variant: None,
        quick_unlock: false,
        name: "Unicorn",
    },
    BestiaryEntryDef {
        credit_id: "GiantBat",
        net_id: 93,
        kind: NpcKind::GiantBat,
        variant: None,
        quick_unlock: false,
        name: "Giant Bat",
    },
    BestiaryEntryDef {
        credit_id: "Corruptor",
        net_id: 94,
        kind: NpcKind::Corruptor,
        variant: None,
        quick_unlock: false,
        name: "Corruptor",
    },
    BestiaryEntryDef {
        credit_id: "DiggerHead",
        net_id: 95,
        kind: NpcKind::DiggerHead,
        variant: None,
        quick_unlock: false,
        name: "Digger",
    },
    BestiaryEntryDef {
        credit_id: "SeekerHead",
        net_id: 98,
        kind: NpcKind::SeekerHead,
        variant: None,
        quick_unlock: false,
        name: "World Feeder",
    },
    BestiaryEntryDef {
        credit_id: "Clinger",
        net_id: 101,
        kind: NpcKind::Clinger,
        variant: None,
        quick_unlock: false,
        name: "Clinger",
    },
    BestiaryEntryDef {
        credit_id: "AnglerFish",
        net_id: 102,
        kind: NpcKind::AnglerFish,
        variant: None,
        quick_unlock: false,
        name: "Angler Fish",
    },
    // W4-g8 lane 1 batch 2 — the d22-d33 machines' kinds. The WallCreeper
    // 164 is `Hide = true` (`NPCID.cs`'s draw modifiers) so it folds onto
    // the WallCreeperWall's credit in `entry_for_npc`, but it still needs a
    // row to resolve (the Spike Ball precedent).
    BestiaryEntryDef {
        credit_id: "GreenJellyfish",
        net_id: 103,
        kind: NpcKind::GreenJellyfish,
        variant: None,
        quick_unlock: false,
        name: "Green Jellyfish",
    },
    BestiaryEntryDef {
        credit_id: "Werewolf",
        net_id: 104,
        kind: NpcKind::Werewolf,
        variant: None,
        quick_unlock: false,
        name: "Werewolf",
    },
    BestiaryEntryDef {
        credit_id: "Clown",
        net_id: 109,
        kind: NpcKind::Clown,
        variant: None,
        quick_unlock: false,
        name: "Clown",
    },
    BestiaryEntryDef {
        credit_id: "SkeletonArcher",
        net_id: 110,
        kind: NpcKind::SkeletonArcher,
        variant: None,
        quick_unlock: false,
        name: "Skeleton Archer",
    },
    BestiaryEntryDef {
        credit_id: "TheHungryII",
        net_id: 116,
        kind: NpcKind::TheHungryII,
        variant: None,
        quick_unlock: false,
        name: "The Hungry",
    },
    BestiaryEntryDef {
        credit_id: "ChaosElemental",
        net_id: 120,
        kind: NpcKind::ChaosElemental,
        variant: None,
        quick_unlock: false,
        name: "Chaos Elemental",
    },
    BestiaryEntryDef {
        credit_id: "Slimer",
        net_id: 121,
        kind: NpcKind::Slimer,
        variant: None,
        quick_unlock: false,
        name: "Slimer",
    },
    BestiaryEntryDef {
        credit_id: "Gastropod",
        net_id: 122,
        kind: NpcKind::Gastropod,
        variant: None,
        quick_unlock: false,
        name: "Gastropod",
    },
    BestiaryEntryDef {
        credit_id: "WallCreeper",
        net_id: 164,
        kind: NpcKind::WallCreeper,
        variant: None,
        quick_unlock: false,
        name: "Wall Creeper",
    },
    BestiaryEntryDef {
        credit_id: "WallCreeperWall",
        net_id: 165,
        kind: NpcKind::WallCreeperWall,
        variant: None,
        quick_unlock: false,
        name: "Wall Creeper",
    },
    BestiaryEntryDef {
        credit_id: "IceElemental",
        net_id: 169,
        kind: NpcKind::IceElemental,
        variant: None,
        quick_unlock: false,
        name: "Ice Elemental",
    },
    BestiaryEntryDef {
        credit_id: "Herpling",
        net_id: 174,
        kind: NpcKind::Herpling,
        variant: None,
        quick_unlock: false,
        name: "Herpling",
    },
    BestiaryEntryDef {
        credit_id: "Derpling",
        net_id: 177,
        kind: NpcKind::Derpling,
        variant: None,
        quick_unlock: false,
        name: "Derpling",
    },
    BestiaryEntryDef {
        credit_id: "CrimsonAxe",
        net_id: 179,
        kind: NpcKind::CrimsonAxe,
        variant: None,
        quick_unlock: false,
        name: "Crimson Axe",
    },
    BestiaryEntryDef {
        credit_id: "FloatyGross",
        net_id: 182,
        kind: NpcKind::FloatyGross,
        variant: None,
        quick_unlock: false,
        name: "Floaty Gross",
    },
    BestiaryEntryDef {
        credit_id: "LostGirl",
        net_id: 195,
        kind: NpcKind::LostGirl,
        variant: None,
        quick_unlock: false,
        name: "Lost Girl",
    },
    BestiaryEntryDef {
        credit_id: "Nymph",
        net_id: 196,
        kind: NpcKind::Nymph,
        variant: None,
        quick_unlock: false,
        name: "Nymph",
    },
    // W4-g8 lane 1 batch 3 — the CursedSkull + the nine bat/flyer family
    // members. The VampireBat 158 is `Hide = true` (it folds onto the
    // Vampire 159 in the vanilla) but still needs a row to resolve.
BestiaryEntryDef {
        credit_id: "IlluminantBat",
        net_id: 137,
        kind: NpcKind::IlluminantBat,
        variant: None,
        quick_unlock: false,
        name: "Illuminant Bat",
    },
    BestiaryEntryDef {
        credit_id: "PossessedArmor",
        net_id: 140,
        kind: NpcKind::PossessedArmor,
        variant: None,
        quick_unlock: false,
        name: "Possessed Armor",
    },
    // W4-c3 (the style-10 family): the WaterBoltMimic's own entry
    // (`BestiaryDatabaseNPCsPopulator.cs:1561` - the automated enemy
    // registration `:851-868`, not in the Hide exclusions; the credit id
    // is the vanilla `NPCID.Search.GetName` row, `NPCID.cs:12065`). The
    // CursedSkull 34 / GiantCursedSkull 289 defs dropped per the d37 E8
    // tables - d34/d38's rows above own them.
    BestiaryEntryDef {
        credit_id: "WaterBoltMimic",
        net_id: 694,
        kind: NpcKind::WaterBoltMimic,
        variant: None,
        quick_unlock: false,
        name: "Water Bolt Mimic",
    },
    BestiaryEntryDef {
        credit_id: "Pupfish",
        net_id: 607,
        kind: NpcKind::Pupfish,
        variant: None,
        quick_unlock: false,
        name: "Pupfish",
    },
    BestiaryEntryDef {
        credit_id: "Dolphin",
        net_id: 615,
        kind: NpcKind::Dolphin,
        variant: None,
        quick_unlock: false,
        name: "Dolphin",
    },
    // E8 merge: the Pufferfish 688 / Orca 692 entries drop with the
    // kinds (d43 rows them later - research/148's TRIGGERED conditional).
    // W4-c7 — the firefly family's own entries (the populator registers
    // all four; `NPCID.Search.GetName` credits).
    BestiaryEntryDef {
        credit_id: "Firefly",
        net_id: 355,
        kind: NpcKind::Firefly,
        variant: None,
        quick_unlock: false,
        name: "Firefly",
    },
    BestiaryEntryDef {
        credit_id: "LightningBug",
        net_id: 358,
        kind: NpcKind::LightningBug,
        variant: None,
        quick_unlock: false,
        name: "Lightning Bug",
    },
    BestiaryEntryDef {
        credit_id: "Lavafly",
        net_id: 654,
        kind: NpcKind::Lavafly,
        variant: None,
        quick_unlock: false,
        name: "Lavafly",
    },
    BestiaryEntryDef {
        credit_id: "Shimmerfly",
        net_id: 677,
        kind: NpcKind::Shimmerfly,
        variant: None,
        quick_unlock: false,
        name: "Shimmerfly",
    },
    BestiaryEntryDef {
        credit_id: "Maggot",
        net_id: 606,
        kind: NpcKind::Maggot,
        variant: None,
        quick_unlock: false,
        name: "Maggot",
    },BestiaryEntryDef {
        credit_id: "AngryTrapper",
        net_id: 175,
        kind: NpcKind::AngryTrapper,
        variant: None,
        quick_unlock: false,
        name: "Angry Trapper",
    },
// The aiStyle-9 spit/flare family (W4-d23): the three wired kinds' own
    // entries — the populator registers all three by id (112/666 the
    // Corruption spit, 516 the Solar Pillar flare; the normal-enemy kill
    // ladder, no quick-unlock).
    BestiaryEntryDef {
        credit_id: "VileSpit",
        net_id: 112,
        kind: NpcKind::VileSpit,
        variant: None,
        quick_unlock: false,
        name: "Vile Spit",
    },
    BestiaryEntryDef {
        credit_id: "VileSpitEaterOfWorlds",
        net_id: 666,
        kind: NpcKind::VileSpitEoW,
        variant: None,
        quick_unlock: false,
        name: "Vile Spit",
    },
    BestiaryEntryDef {
        credit_id: "SolarFlare",
        net_id: 516,
        kind: NpcKind::SolarFlare,
        variant: None,
        quick_unlock: false,
        name: "Solar Flare",
    },BestiaryEntryDef {
        credit_id: "IceBat",
        net_id: 150,
        kind: NpcKind::IceBat,
        variant: None,
        quick_unlock: false,
        name: "Ice Bat",
    },
    BestiaryEntryDef {
        credit_id: "LavaBat",
        credit_id: "Lavabat",
        net_id: 151,
        kind: NpcKind::Lavabat,
        variant: None,
        quick_unlock: false,
        name: "Lava Bat",
        name: "Lavabat",
    },
    BestiaryEntryDef {
        credit_id: "GiantFlyingFox",
        net_id: 152,
        kind: NpcKind::GiantFlyingFox,
        variant: None,
        quick_unlock: false,
        name: "Giant Flying Fox",
    },
    BestiaryEntryDef {
        credit_id: "RedDevil",
        net_id: 156,
        kind: NpcKind::RedDevil,
        variant: None,
        quick_unlock: false,
        name: "Red Devil",
    },
    BestiaryEntryDef {
        credit_id: "VampireBat",
        net_id: 158,
        kind: NpcKind::VampireBat,
        variant: None,
        quick_unlock: false,
        name: "Vampire Bat",
    },
    // The VampireBat's Transform(159) target (`NPC.cs:23353`) — aiStyle 3
    // (the fighter walker), NOT a style-14 member; it lands here because
    // the transform pair is one unit (the bat flees the surface by day,
    // the vampire walks it by night — `:58843-58851` transforms back).
// W4-g8 lane 1 batch 4 — the last three ≤230 d22-d33 machine members.
// W4-g8 machine round 3 — the VampireBat's Transform(159) target
    // (`BestiaryDatabaseNPCsPopulator.cs:1465` gives 159 its own Eclipse
    // info row; the legacy name "Vampire" maps to 158, `NPCID.cs:12301`).
    BestiaryEntryDef {
        credit_id: "Vampire",
        net_id: 159,
        kind: NpcKind::Vampire,
        variant: None,
        quick_unlock: false,
        name: "Vampire",
    },
BestiaryEntryDef {
        credit_id: "SporeBat",
        net_id: 634,
        kind: NpcKind::SporeBat,
        variant: None,
        quick_unlock: false,
        name: "Spore Bat",
    },
    BestiaryEntryDef {
        credit_id: "QueenSlimeMinionPurple",
        net_id: 660,
        kind: NpcKind::QueenSlimeMinionPurple,
        variant: None,
        quick_unlock: false,
        name: "Queen Slime Minion",
        credit_id: "Wolf",
        net_id: 155,
        kind: NpcKind::Wolf,
        variant: None,
        quick_unlock: false,
        name: "Wolf",
    },
    // W4-d25 STYLE-26 wiring (commit 2): the Tumbleweed's row rides
    // `BestiaryDatabaseNPCsPopulator.cs:1292` (Desert + Sandstorm tags)
    // and the en-US `NPCs.json:563` name ("Angry Tumbler"; the credit id
    // is `NPCID.Search.GetName(546)` = "Tumbleweed").
    BestiaryEntryDef {
        credit_id: "Tumbleweed",
        net_id: 546,
        kind: NpcKind::Tumbleweed,
        variant: None,
        quick_unlock: false,
        name: "Angry Tumbler",
    },
    // W4-d120 — AddTownNPCs_Manual (BestiaryDatabaseNPCsPopulator.cs:814-
    // 822): the four town kinds' TownNPC entries; the names ride the
    // en-US NPCs.json rows (`NPCName.<const>`).
    BestiaryEntryDef {
        credit_id: "SantaClaus",
        net_id: 142,
        kind: NpcKind::SantaClaus,
        variant: None,
        quick_unlock: false,
        name: "Santa Claus",
    },
    BestiaryEntryDef {
        credit_id: "Truffle",
        net_id: 160,
        kind: NpcKind::Truffle,
        variant: None,
        quick_unlock: false,
        name: "Truffle",
    },
    BestiaryEntryDef {
        credit_id: "Steampunker",
        net_id: 178,
        kind: NpcKind::Steampunker,
        variant: None,
        quick_unlock: false,
        name: "Steampunker",
    },
    BestiaryEntryDef {
        credit_id: "Cyborg",
        net_id: 209,
        kind: NpcKind::Cyborg,
        variant: None,
        quick_unlock: false,
        name: "Cyborg",
    },
    // The dragonflies (W4-d33): the populator's visible card is the Red
    // 599 "Dragonfly" (`NPCs.json:637`) — 595/596/597/598/600 are
    // `Hide = true` (`NPCID.cs:510-543`, `GetExclusions` drops them from
    // `AddEmptyEntries_CrittersAndEnemies_Automated`, `:851-885`) and
    // their kill credits fold onto 599 (`ContentSamples.cs:1043-1047`).
    // The hidden colours keep their OWN entries (the c8 TruffleWormDigger
    // precedent: own entry + cs_id for the catch/release chain's
    // `kind_from_net_id`; `entry_for_npc` folds the kill credit).
    BestiaryEntryDef {
        credit_id: "BlackDragonfly",
        net_id: 595,
        kind: NpcKind::BlackDragonfly,
        variant: None,
        quick_unlock: false,
        name: "Dragonfly",
    },
    BestiaryEntryDef {
        credit_id: "BlueDragonfly",
        net_id: 596,
        kind: NpcKind::BlueDragonfly,
        variant: None,
        quick_unlock: false,
        name: "Dragonfly",
    },
    BestiaryEntryDef {
        credit_id: "GreenDragonfly",
        net_id: 597,
        kind: NpcKind::GreenDragonfly,
        variant: None,
        quick_unlock: false,
        name: "Dragonfly",
    },
    BestiaryEntryDef {
        credit_id: "OrangeDragonfly",
        net_id: 598,
        kind: NpcKind::OrangeDragonfly,
        variant: None,
        quick_unlock: false,
        name: "Dragonfly",
    },
    BestiaryEntryDef {
        credit_id: "RedDragonfly",
        net_id: 599,
        kind: NpcKind::RedDragonfly,
        variant: None,
        quick_unlock: false,
        name: "Dragonfly",
    },
    // The Yellow 600 is Hide=true too (`NPCID.cs:538-543`) — its credit
    // folds onto 599; the Gold 601 keeps its OWN visible card
    // ("Gold Dragonfly", `NPCs.json:639` — no Hide row, no
    // `ContentSamples` fold, the GoldLadyBug precedent).
    BestiaryEntryDef {
        credit_id: "YellowDragonfly",
        net_id: 600,
        kind: NpcKind::YellowDragonfly,
        variant: None,
        quick_unlock: false,
        name: "Dragonfly",
    },
    BestiaryEntryDef {
        credit_id: "GoldDragonfly",
        net_id: 601,
        kind: NpcKind::GoldDragonfly,
        variant: None,
        quick_unlock: false,
        name: "Gold Dragonfly",
    },
    // The fairy critters + the windy balloon (W4-d33 wiring): all four
    // are VISIBLE cards — the `NPCID.cs:5148-5169` fairy draw modifiers
    // only shift Position/Direction (no Hide), the balloon's
    // `:5232-5239` row sets a CustomTexturePath + Scale 0.8 (no Hide);
    // no `ContentSamples` credit fold. The names ride
    // `Lang.GetNPCName` (en-US NPCs.json: "Pink Fairy" / "Green Fairy"
    // / "Blue Fairy" / "Windy Balloon"). d42 batch-2/3's rows,
    // constants-copy (keep-one d37c-E3).
    BestiaryEntryDef {
        credit_id: "FairyCritterPink",
        net_id: 583,
        kind: NpcKind::FairyCritterPink,
        variant: None,
        quick_unlock: false,
        name: "Pink Fairy",
    },
    BestiaryEntryDef {
        credit_id: "FairyCritterGreen",
        net_id: 584,
        kind: NpcKind::FairyCritterGreen,
        variant: None,
        quick_unlock: false,
        name: "Green Fairy",
    },
    BestiaryEntryDef {
        credit_id: "FairyCritterBlue",
        net_id: 585,
        kind: NpcKind::FairyCritterBlue,
        variant: None,
        quick_unlock: false,
        name: "Blue Fairy",
    },
    BestiaryEntryDef {
        credit_id: "WindyBalloon",
        net_id: 594,
        kind: NpcKind::WindyBalloon,
        variant: None,
        quick_unlock: false,
        name: "Windy Balloon",
    },
];

/// `CommonEnemyUICollectionInfoProvider.GetEntryUICollectionInfo`
/// (`CommonEnemyUICollectionInfoProvider.cs:39-46`): the entry's current
/// unlock tier from the kill tracker. The CRITTER entries never read
/// this — [`unlock_state_world`] dispatches them to the sight providers
/// (the W4-c8 sweep census).
pub fn unlock_state(entry: &BestiaryEntryDef, kills: &KillTracker) -> UnlockState {
    let full = tr_core::bestiary::kills_needed_for(entry.net_id);
    get_unlock_state_by_kill_count(
        kills.get_kill_count(entry.credit_id),
        entry.quick_unlock,
        full,
    )
}

/// W4-c8 final — `NPC.CountsAsACritter` (`NPC.cs:6887-6897`): the
/// GENERIC gate, not a kind list — `lifeMax <= 5 && damage == 0 &&
/// type != 594 && type != 686` (the WindyBalloon 594 / BoundTownSlimePurple
/// 686 exclusions, `NPCID.cs:11865`/`:12049`, both unported kinds). The
/// port reads the kind's `SetDefaults` rows (`max_hp`/`contact_damage`,
/// the same stand-ins [`scan_world_for_critter_sights`] uses on the live
/// NPC). This is the PROVIDER CONTRACT for every critter lane: a kind
/// rowed by c7 (fireflies 355/358/654/677), c9 (snails 359/360/655),
/// c10 (waterfowl 362-365/602/603/608/609), e26 (Frog 361 / LadyBug 604)
/// or c4 (Goldfish 55 / GoldGoldfish 592 / GoldGoldfishWalker 593) is
/// auto-covered the moment its `SetDefaults` rows land — all those C#
/// rows are lifeMax 5 / damage 0 (e.g. `NPC.cs:13601-13655` the
/// frog/duck block), so no per-lane provider wiring is needed.
fn counts_as_a_critter(kind: NpcKind) -> bool {
    kind.max_hp() <= 5
        && kind.contact_damage() == 0
        && kind.cs_id() != 594
        && kind.cs_id() != 686
}

/// W4-c8 sweep — `BestiaryEntry.Critter` (`BestiaryEntry.cs:82-101`):
/// every `CountsAsACritter` NPC registers with the
/// `CritterUICollectionInfoProvider`, NOT the kill-count provider
/// (`AddEmptyEntries_CrittersAndEnemies_Automated`,
/// `BestiaryDatabaseNPCsPopulator.cs:851-868`). W4-c8 final: the dispatch
/// is the GENERIC [`counts_as_a_critter`] gate (was the nine-kind list)
/// — the Goldfish 55's C# provider IS the critter one (`NPC.cs:9460-9473`
/// lifeMax 5 / damage 0), so the c4-wave freeze is lifted here (the c4
/// follow-up the sweep recorded). The one exclusion: the DD2 lane
/// portal 549 passes the raw gate (lifeMax 5 / damage 0,
/// `NPC.cs:16035-16037`) but is `Hide = true` (`NPCID.cs:4910-4915`) —
/// the populator's `GetExclusions` (`:870-886`) never registers an entry
/// for it, so the port's fold row keeps the kill provider (the
/// ForceBubble precedent). The digger 375 is Hide too but its own row
/// was landed WITH the sight provider by the sweep (the witness pins
/// it) — kept.
fn is_critter_entry(entry: &BestiaryEntryDef) -> bool {
    entry.kind != NpcKind::DD2LanePortal && counts_as_a_critter(entry.kind)
}

/// W4-c8 sweep — `CritterUICollectionInfoProvider.GetEntryUICollectionInfo`
/// (`CritterUICollectionInfoProvider.cs:14-20`): the critter entry is
/// tier 4 (`CanShowDropsWithDropRates_4`) once the player has been NEAR
/// it (`Sights.GetWasNearbyBefore`), else tier 0. NOT caught, NOT
/// killed — the sight tracker (`NPCWasNearPlayerTracker.ScanWorldForFinds`,
/// `:102-130`, run from `Main.cs:17914` every update) is the credit.
fn critter_unlock_state(credit_id: &str, sights: &WorldBestiary) -> UnlockState {
    if sights.was_seen_near_player(credit_id) {
        UnlockState::CanShowDropsWithDropRates
    } else {
        UnlockState::NotKnownAtAll
    }
}

/// W4-c8 final — the gold-critter PEER table
/// (`BestiaryDatabaseNPCsPopulator.cs:703-716`): the 13
/// `GoldCritterUICollectionInfoProvider` registrations — gold net id →
/// its normal-critter peers' credit ids (`NPCID.Search.GetName` names).
/// 593 GoldGoldfishWalker is in the COLLECTION but NOT in this table —
/// it rides the plain critter provider (its own entry, `NPCID.cs:5227`
/// no Hide). The 539 squirrel is the one TWO-peer row (`{299, 538}`).
/// This is the cross-lane contract: e26's GoldLadyBug 605 → {604} and
/// c4's GoldGoldfish 592 → {55} auto-ride the gold provider once their
/// rows land; d40's 442-447 band and the unported 601/613/627/539 rows
/// are pinned here for their lanes.
const GOLD_PEER_TABLE: &[(u16, &[&str])] = &[
    (443, &["Bunny"]),                    // GoldBunny
    (442, &["Bird"]),                     // GoldBird
    (592, &["Goldfish"]),                 // GoldGoldfish
    (444, &["Butterfly"]),                // GoldButterfly
    (601, &["RedDragonfly"]),             // GoldDragonfly
    (445, &["Frog"]),                     // GoldFrog
    (446, &["Grasshopper"]),              // GoldGrasshopper
    (605, &["LadyBug"]),                  // GoldLadyBug
    (447, &["Mouse"]),                    // GoldMouse
    (627, &["Seahorse"]),                 // GoldSeahorse
    (613, &["WaterStrider"]),             // GoldWaterStrider
    (448, &["Worm"]),                     // GoldWorm
    (539, &["Squirrel", "SquirrelRed"]),  // SquirrelGold
];

/// W4-c8 sweep — `GoldCritterUICollectionInfoProvider`
/// (`GoldCritterUICollectionInfoProvider.cs:21-75`): the gold entry's
/// state is the MAX of its own sight and its normal-critter peers'
/// sights (the [`GOLD_PEER_TABLE`] row), BUT the whole entry stays
/// tier 0 until ANY gold critter in `NPCID.Sets.GoldCrittersCollection`
/// (`NPCID.cs:10121-10125`) has been seen (`:46-52`'s
/// `TryFindingOneGoldCritterThatIsAlreadyUnlocked`).
/// W4-c8 final: SET-COMPLETE census — the 14-member collection below
/// matches the C# list verbatim (443/442/592/593/444/601/445/446/605/
/// 447/627/613/448/539), the `IsGoldCritter` bool set (`:10126`) is the
/// SAME 14 ids (the sparkle arm's family — only 448 is ported on this
/// branch); d40's gold band 442-447 + the c4 592/593 + e26 605 rows
/// slot into the credit-id strings unchanged when they merge.
fn gold_critter_unlock_state(
    gold_credit: &str,
    peers: &[&str],
    sights: &WorldBestiary,
) -> UnlockState {
    // `:23-37` — max(own, the normal peers).
    let mut state = critter_unlock_state(gold_credit, sights);
    for peer in peers {
        let peer_state = critter_unlock_state(peer, sights);
        if peer_state > state {
            state = peer_state;
        }
    }
    if state == UnlockState::NotKnownAtAll {
        return state;
    }
    // `:56-66` — the collection gate: any gold critter seen unlocks the
    // gold entries' display. The credit ids are the NPCID names
    // (`ContentSamples.NpcBestiaryCreditIdsByNpcNetIds` —
    // `NPCID.Search.GetName`); 539 is SquirrelGold (`NPCID.cs:11755`).
    const GOLD_COLLECTION: [&str; 14] = [
        "GoldBunny",        // 443
        "GoldBird",         // 442
        "GoldGoldfish",     // 592
        "GoldGoldfishWalker", // 593
        "GoldButterfly",    // 444
        "GoldDragonfly",    // 601
        "GoldFrog",         // 445
        "GoldGrasshopper",  // 446
        "GoldLadyBug",      // 605
        "GoldMouse",        // 447
        "GoldSeahorse",     // 627
        "GoldWaterStrider", // 613
        "GoldWorm",         // 448
        "SquirrelGold",     // 539
    ];
    if GOLD_COLLECTION.iter().any(|id| sights.was_seen_near_player(id)) {
        state
    } else {
        UnlockState::NotKnownAtAll
    }
}

/// W4-c8 sweep — the WORLD-aware unlock dispatch (the C#
/// `UIInfoProvider` polymorphism): the gold-table entries ride the gold
/// provider, the other critter entries the sight provider, everything
/// else the kill-count provider unchanged. The kill-only
/// [`unlock_state`] stays for the kill-provider callers/tests. W4-c8
/// final: the gold arm is table-driven (`net_id` in
/// [`GOLD_PEER_TABLE`]) — a merged lane's gold row needs no dispatch
/// edit.
pub fn unlock_state_world(entry: &BestiaryEntryDef, bestiary: &WorldBestiary) -> UnlockState {
    if let Some((_, peers)) = GOLD_PEER_TABLE
        .iter()
        .find(|(net, _)| *net as i32 == entry.net_id)
    {
        return gold_critter_unlock_state(entry.credit_id, peers, bestiary);
    }
    if is_critter_entry(entry) {
        return critter_unlock_state(entry.credit_id, bestiary);
    }
    unlock_state(entry, &bestiary.kills)
}

/// `Main.GetBestiaryProgressReport()`'s completion fraction (the
/// `BestiaryUI` report): the ported entries past the portrait-only tier
/// over the whole roster. The Zoologist's shop gates read it
/// (`Chest.cs:3265`). The kill-only form stays for the callers without
/// a world tracker; [`completion_percent_world`] is the W4-c8 sweep's
/// sight-aware dispatch (the critter entries unlock on proximity, not
/// kills).
pub fn completion_percent(kills: &KillTracker) -> f32 {
    // `BestiaryUnlockProgressReport.CompletionPercent`
    // (`BestiaryUnlockProgressReport.cs:9-19`): 1 for an empty roster,
    // else the known share (`UIBestiaryTest.GetUnlockProgress`, `:497-513`,
    // counts `> NotKnownAtAll_0`).
    if ENTRIES.is_empty() {
        return 1.0;
    }
    let seen = ENTRIES
        .iter()
        .filter(|e| unlock_state(e, kills).known())
        .count();
    seen as f32 / ENTRIES.len() as f32
}

/// W4-c8 sweep — the sight-aware completion (the same fraction over
/// [`unlock_state_world`]'s dispatch).
pub fn completion_percent_world(bestiary: &WorldBestiary) -> f32 {
    if ENTRIES.is_empty() {
        return 1.0;
    }
    let seen = ENTRIES
        .iter()
        .filter(|e| unlock_state_world(e, bestiary).known())
        .count();
    seen as f32 / ENTRIES.len() as f32
}

/// W4-c8 sweep — `NPCWasNearPlayerTracker.ScanWorldForFinds`
/// (`NPCWasNearPlayerTracker.cs:102-130`, called from `Main.cs:17914`
/// under `netMode != 1` every update, BEFORE the `UpdateNPC` loop):
/// every ACTIVE `CountsAsACritter` NPC whose hitbox intersects a
/// player's `HitboxForBestiaryNearbyCheck` (the player box inflated
/// 300×200, `Player.cs:3313-3321`) registers its credit id once
/// (`_wasSeenNearPlayerByNetId` dedups the scan; the port's BTreeSet
/// insert is idempotent — the same result). `CountsAsACritter`
/// (`NPC.cs:6887-6897`): `lifeMax <= 5 && damage == 0 && type != 594
/// && type != 686` — the port reads the kind's SetDefaults contact
/// damage + the live max_hp (the 594/686 exclusions are unported
/// kinds). The session player is the one hitbox (the S1-D-3p
/// convention); a dead player still counts (the C# scan has no dead
/// gate — only `player.active`). RNG-free.
/// W4-c8 final — the CROSS-LANE PROVIDER CONTRACT: the gate below is
/// GENERIC (no kind list), so a critter rowed by ANY lane is scanned
/// the moment its `SetDefaults` rows land — c7's Firefly 355 /
/// LightningBug 358 / Lavafly 654 / Shimmerfly 677, c9's Snail 359 /
/// GlowingSnail 360 / MagmaSnail 655, c10's Duck 362-365 / Seagull
/// 602/603 / Grebe 608/609, e26's Frog 361 / LadyBug 604 / GoldLadyBug
/// 605 and c4's Goldfish 55 / GoldGoldfish 592 / GoldGoldfishWalker
/// 593 are ALL lifeMax 5 / damage 0 in the C# (`NPC.cs:13601-13655`
/// the frog/duck block, `:9460-9473` the goldfish, `:16763+` the
/// ladybugs). Their `entry_for_npc` credit lands in the same sights
/// set the [`crate::bestiary::unlock_state_world`] dispatch reads —
/// the generic [`crate::bestiary::counts_as_a_critter`] gate + the
/// `GOLD_PEER_TABLE` gold arm cover the UI side. NO per-lane scan or
/// provider edit is needed. (The bestiary-entry Hide folds — 549 lane
/// portal — keep the kill provider; the digger 375 folds onto the
/// TruffleWorm credit here, its own row stands for the sweep witness.)
pub(crate) fn scan_world_for_critter_sights(app: &mut crate::app::App) {
    let Some(p) = app.player.as_ref() else {
        return;
    };
    // `HitboxForBestiaryNearbyCheck`: the box inflated 300 px per
    // horizontal side, 200 per vertical side.
    let (px, py) = (p.x as i32 - 300, p.y as i32 - 200);
    let (pw, ph) = (
        crate::player::WIDTH as i32 + 600,
        crate::player::HEIGHT as i32 + 400,
    );
    let mut found: Vec<&'static str> = Vec::new();
    for n in &app.npcs {
        if n.hp <= 0 {
            continue;
        }
        // `CountsAsACritter` — the live lifeMax + the kind's contact
        // damage (the SetDefaults row; no ported kind overrides damage).
        if n.max_hp > 5 || n.kind.contact_damage() != 0 {
            continue;
        }
        let (nw, nh) = n.size();
        let (nx, ny) = (n.x as i32, n.y as i32);
        // XNA `Rectangle.Intersects` — half-open on both axes.
        if !(nx < px + pw && nx + nw as i32 > px && ny < py + ph && ny + nh as i32 > py) {
            continue;
        }
        let entry = crate::bestiary::entry_for_npc(n.kind, n.slime);
        if !app.bestiary.was_seen_near_player(entry.credit_id) {
            found.push(entry.credit_id);
        }
    }
    for id in found {
        // `RegisterWasNearby` (`:32-41`) → `SetWasSeenDirectly`
        // (`:43-52`): the set insert + the 100 % achievement hook.
        app.bestiary.set_was_seen_directly(id);
        let _ = app.try_grant_bestiary_100_net();
    }
}

/// `Main.cs:65944`'s flag9: every other town NPC present — the C#'s 24
/// counters; the port folds them to its own town roster (the kinds the
/// port lacks sit in their own tasks), and the debug override
/// (`NPC.unlockedPrincessSpawn`) opens it without the roster.
pub(crate) fn all_town_kinds_present(npcs: &[crate::npc::Npc]) -> bool {
    let alive = |kind: crate::npc::NpcKind| npcs.iter().any(|n| n.kind == kind && n.hp > 0);
    use crate::npc::NpcKind as K;
    [
        K::Guide,
        K::Merchant,
        K::Nurse,
        K::Dryad,
        K::ArmsDealer,
        K::Demolitionist,
        K::Clothier,
        K::GoblinTinkerer,
        K::TravellingMerchant,
        K::Wizard,
        K::Mechanic,
        K::Stylist,
        K::TaxCollector,
        K::Golfer,
        K::Zoologist,
        K::Angler,
        K::Painter,
        K::WitchDoctor,
    ]
    .iter()
    .all(|kind| alive(*kind))
}

/// The entry an NPC death credits — the death hook's
/// `IsNPCValidForBestiaryKillCredit` stand-in (every ported kind has an
/// entry, so the lookup never fails).
/// Whether the bestiary carries a row for this kind/variant — the
/// non-panicking probe `entry_for_npc` needs for callers that must tolerate a
/// gap (the audit tests, the WIP kinds).
pub fn has_entry(kind: NpcKind, variant: SlimeVariant) -> bool {
    ENTRIES.iter().any(|e| {
        e.kind == kind
            && match (e.variant, kind) {
                (Some(v), NpcKind::Slime) => v == variant,
                _ => true,
            }
    })
}

/// `entry_for_npc`'s non-panicking twin (S1-F-3): the kinds the bestiary
/// does not list (the boss segments the credits fold, the critter twins the
/// roster skips) return `None`.
pub fn try_entry_for_npc(
    kind: NpcKind,
    variant: SlimeVariant,
) -> Option<&'static BestiaryEntryDef> {
    // The worm segments fold onto the head's entry (the vanilla credits the
    // whole worm to the head's bestiary id — one entry per credit).
    let kind = match kind {
        NpcKind::DevourerBody | NpcKind::DevourerTail => NpcKind::DevourerHead,
        NpcKind::GiantWormBody | NpcKind::GiantWormTail => NpcKind::GiantWormHead,
        NpcKind::BoneSerpentBody | NpcKind::BoneSerpentTail => NpcKind::BoneSerpentHead,
        NpcKind::EaterOfWorldsBody | NpcKind::EaterOfWorldsTail => NpcKind::EaterOfWorldsHead,
        // The Destroyer's segments + the Probe fold onto the head
        // (`BestiaryDatabaseNPCsPopulator.cs:688-690`: {135,134},{136,134},
        // {139,134}).
        NpcKind::TheDestroyerBody | NpcKind::TheDestroyerTail | NpcKind::Probe => {
            NpcKind::TheDestroyer
        }
        // W-A2-3: the leech links fold like every other worm family (the
        // vanilla credits the whole leech to 117).
        NpcKind::LeechBody | NpcKind::LeechTail => NpcKind::LeechHead,
        // The wyvern segments fold onto the head's entry (`NPCID.cs`:
        // 1821-1856's `Hide = true` draw modifiers — the whole serpent is
        // one bestiary credit, the Devourer precedent).
        NpcKind::WyvernLegs
        | NpcKind::WyvernBody
        | NpcKind::WyvernBody2
        | NpcKind::WyvernBody3
        | NpcKind::WyvernTail => NpcKind::WyvernHead,
        // W4-d35 (F): the 112 Hide=true kinds fold onto
        // the head's entry (NPCID.cs DrawOffset Hide; the populator's
        // GetExclusions skips them - the Wyvern-segment precedent).
        NpcKind::VileSpit => NpcKind::Corruptor,
        // W4-d35 (D): the 96/97/99/100 Hide=true kinds fold onto
        // the head's entry (NPCID.cs DrawOffset Hide; the populator's
        // GetExclusions skips them - the Wyvern-segment precedent).
        NpcKind::DiggerBody => NpcKind::DiggerHead,
        NpcKind::DiggerTail => NpcKind::DiggerHead,
        NpcKind::SeekerBody => NpcKind::SeekerHead,
        NpcKind::SeekerTail => NpcKind::SeekerHead,
        // The Golem's parts fold onto the body's entry (the Brain's
        // Creeper precedent — one credit per boss).
        NpcKind::GolemHead
        | NpcKind::GolemFistLeft
        | NpcKind::GolemFistRight
        | NpcKind::GolemHeadFree => NpcKind::Golem,
        // S1-D-3 W-9: the Hide=true hazards fold onto the boss (the Golem
        // parts' precedent); the Sharkron 372 keeps its OWN entry (its
        // kill credit is "Sharkron" — only the UI provider maps onto 370,
        // `BestiaryDatabaseNPCsPopulator.cs:691`).
        NpcKind::DetonatingBubble | NpcKind::Sharkron2 => NpcKind::DukeFishron,
        // W4-d37 — the NPCID 211-280 band's Hide=true exclusions
        // (`NPCID.cs`'s NPCBestiaryDrawOffset): the goldfish walker 230
        // (`:616-620`) folds onto the Goldfish (55) credit, the jungle
        // creeper 236 (`:2790-2794`) onto the JungleCreeperWall (237), the
        // blood crawler 239 (`:2813-2817`) onto the BloodCrawlerWall (240)
        // — the SleepingAngler precedent (no own bestiary entry).
        NpcKind::GoldfishWalker => NpcKind::Goldfish,
        NpcKind::JungleCreeper => NpcKind::JungleCreeperWall,
        NpcKind::BloodCrawler => NpcKind::BloodCrawlerWall,
        // W4-d40: the bestiary credit folds (`ContentSamples.cs`;
        // the Hide=true variants have no own entry).
        NpcKind::ArmedZombie => NpcKind::Zombie,
        NpcKind::ArmedZombieEskimo => NpcKind::ZombieEskimo,
        NpcKind::ArmedZombiePincussion => NpcKind::PincushionZombie,
        NpcKind::ArmedZombieSlimed => NpcKind::SlimedZombie,
        NpcKind::ArmedZombieSwamp => NpcKind::SwampZombie,
        // W4-d40: the bestiary credit folds (`ContentSamples.cs`;
        // the Hide=true variants have no own entry).
        NpcKind::ArmedZombieTwiggy => NpcKind::TwiggyZombie,
        NpcKind::ArmedZombieCenx => NpcKind::FemaleZombie,
        // W4-d40: the bestiary credit folds (`ContentSamples.cs`;
        // the Hide=true variants have no own entry).
        NpcKind::BoneThrowingSkeleton => NpcKind::Skeleton,
        // W4-d40: the bestiary credit folds (`ContentSamples.cs`;
        // the Hide=true variants have no own entry).
        NpcKind::CultistDragonBody1 => NpcKind::CultistDragonHead,
        NpcKind::CultistDragonBody2 => NpcKind::CultistDragonHead,
        // W4-d40: the bestiary credit folds (`ContentSamples.cs`;
        // the Hide=true variants have no own entry).
        NpcKind::CultistDragonBody3 => NpcKind::CultistDragonHead,
        NpcKind::CultistDragonBody4 => NpcKind::CultistDragonHead,
        NpcKind::CultistDragonTail => NpcKind::CultistDragonHead,
        // W4-d40: the bestiary credit folds (`ContentSamples.cs`;
        // the Hide=true variants have no own entry).
        NpcKind::MothronEgg => NpcKind::Mothron,
        // W4-d42 (merge-side mirror, 367 s1): the Hide=true folds ride BOTH
        // resolvers (the d37/d40 precedent) — d42's own commit wired only
        // `entry_for_npc`; `try_entry_for_npc` gets the same arms.
        NpcKind::BartenderUnconscious => NpcKind::DD2Bartender,
        NpcKind::ArmedTorchZombie => NpcKind::TorchZombie,
        // W4-c4 merge (148 item 5): the walker's 593 credit folds onto
        // the GoldGoldfish 592 (`ContentSamples.cs:1049`).
        NpcKind::GoldGoldfishWalker => NpcKind::GoldGoldfish,
        // (the fairy/balloon/dragonfly dups 583-585/594/595-601 dropped -
        // d33 owns them, d37 tables; their folds land with its splice.)
        NpcKind::Seagull2 => NpcKind::Seagull,
        NpcKind::Grebe2 => NpcKind::Grebe,
        NpcKind::BloodEelBody => NpcKind::BloodEelHead,
        NpcKind::BloodEelTail => NpcKind::BloodEelHead,
        // (W4-c8: the digger's fold is entry_for_npc-ONLY - it keeps its
        // OWN ENTRIES entry net 375 for the cs_id fallback, the
        // ForceBubble precedent; mirroring it here would break the
        // s1_f_3 cs_id==net gate.)
        // W4-c6 style-59 wiring: the blade's bestiary draw modifiers are
        // Hide=true (`NPCID.cs:3398-3403`) — the credit folds onto the
        // Pumpking (the fishron-hazard precedent). The DRAW sheet stays
        // NPC_328 through `texture_net_id`'s own row.
        NpcKind::PumpkingBlade => NpcKind::Pumpking,
        other => other,
    };
    ENTRIES.iter().find(|e| {
        e.kind == kind
            && match (e.variant, kind) {
                (Some(v), NpcKind::Slime) => v == variant,
                _ => true,
            }
    })
}

pub fn entry_for_npc(kind: NpcKind, variant: SlimeVariant) -> &'static BestiaryEntryDef {
    // The worm segments fold onto the head's entry (the vanilla credits the
    // whole worm to the head's bestiary id — one entry per credit).
    let kind = match kind {
        NpcKind::DevourerBody | NpcKind::DevourerTail => NpcKind::DevourerHead,
        NpcKind::BoneSerpentBody | NpcKind::BoneSerpentTail => NpcKind::BoneSerpentHead,
        NpcKind::GiantWormBody | NpcKind::GiantWormTail => NpcKind::GiantWormHead,
        NpcKind::EaterOfWorldsBody | NpcKind::EaterOfWorldsTail => NpcKind::EaterOfWorldsHead,
        // The Destroyer's segments (Hide=true, `NPCID.cs:2192-2205`) and
        // the Probe (`BestiaryDatabaseNPCsPopulator.cs:690`: {139,134})
        // fold onto the head's entry.
        NpcKind::TheDestroyerBody | NpcKind::TheDestroyerTail | NpcKind::Probe => {
            NpcKind::TheDestroyer
        }
        // W-A2-3: the leech links fold onto 117 like every other worm.
        NpcKind::LeechBody | NpcKind::LeechTail => NpcKind::LeechHead,
        // The wyvern segments fold onto the head's entry (`NPCID.cs`:
        // 1821-1856's `Hide = true` draw modifiers — one credit for the
        // whole serpent).
        NpcKind::WyvernLegs
        | NpcKind::WyvernBody
        | NpcKind::WyvernBody2
        | NpcKind::WyvernBody3
        | NpcKind::WyvernTail => NpcKind::WyvernHead,
        // W4-d35 (F): the 112 Hide=true kinds fold onto
        // the head's entry (NPCID.cs DrawOffset Hide; the populator's
        // GetExclusions skips them - the Wyvern-segment precedent).
        NpcKind::VileSpit => NpcKind::Corruptor,
        // W4-d35 (D): the 96/97/99/100 Hide=true kinds fold onto
        // the head's entry (NPCID.cs DrawOffset Hide; the populator's
        // GetExclusions skips them - the Wyvern-segment precedent).
        NpcKind::DiggerBody => NpcKind::DiggerHead,
        NpcKind::DiggerTail => NpcKind::DiggerHead,
        NpcKind::SeekerBody => NpcKind::SeekerHead,
        NpcKind::SeekerTail => NpcKind::SeekerHead,
        // The Golem's parts fold onto the body's entry (the Brain's
        // Creeper precedent — one credit per boss).
        NpcKind::GolemHead
        | NpcKind::GolemFistLeft
        | NpcKind::GolemFistRight
        | NpcKind::GolemHeadFree => NpcKind::Golem,
        // The Sleeping Angler rides the Angler's row — the vanilla's
        // bestiary draw modifiers hide it (`NPCID.cs:3721-3725`), so it
        // never has an entry of its own.
        NpcKind::SleepingAngler => NpcKind::Angler,
        // W4-d120 — the bound Tavernkeep rides the Tavernkeep's row: the
        // vanilla's draw modifier Hide=true (`NPCID.cs:5178-5184`), the
        // Sleeping Angler precedent.
        // The bound Wizard/Mechanic ride their freed rows (the C#'s
        // bestiary ids 106/123 have no entries of their own — the bound
        // Goblin 105 has none either).
        NpcKind::BoundWizard => NpcKind::Wizard,
        NpcKind::BoundMechanic => NpcKind::Mechanic,
        // The webbed Stylist rides the Stylist's row — the vanilla's
        // bestiary draw modifiers hide it (`NPCID.cs:3557-3561`), so it
        // never has an entry of its own.
        NpcKind::WebbedStylist => NpcKind::Stylist,
        // The sleeping Golfer rides the Golfer's row — the vanilla's
        // bestiary draw modifiers hide it (`NPCID.cs:6072-6075`), so it
        // never has an entry of its own.
        NpcKind::SleepingGolfer => NpcKind::Golfer,
        NpcKind::BoundGoblin => NpcKind::GoblinTinkerer,
        // S1-D-3 W-9 fishron wiring — the Hide=true exclusions (371 the
        // Detonating Bubble, 373 the Sharkron2 — `NPCID.cs`'s
        // NPCBestiaryDrawOffset rows) have no entries of their own; they
        // fold onto the boss / the plain Sharkron row (the Golem parts'
        // one-credit precedent). The DRAW sheet ids stay 371/373 through
        // `texture_net_id`'s own rows.
        NpcKind::DetonatingBubble => NpcKind::DukeFishron,
        NpcKind::Sharkron2 => NpcKind::Sharkron,
        // The Force Bubble rides the Officer's row — the vanilla's bestiary
        // draw modifiers hide it (`NPCID.cs:3766-3770`'s `Hide = true`), so
        // it never has an entry of its own (the Sleeping Angler precedent;
        // the banner fold agrees — `BannerSystem.cs:777-779` shares 141).
        NpcKind::ForceBubble => NpcKind::MartianOfficer,
        // W4-d37 — the NPCID 211-280 band's Hide=true exclusions (the
        // `try_entry_for_npc` folds above): 230 → Goldfish, 236 →
        // JungleCreeperWall, 239 → BloodCrawlerWall.
        NpcKind::GoldfishWalker => NpcKind::Goldfish,
        NpcKind::JungleCreeper => NpcKind::JungleCreeperWall,
        NpcKind::BloodCrawler => NpcKind::BloodCrawlerWall,
        // W4-d40: the bestiary credit folds (`ContentSamples.cs`;
        // the Hide=true variants have no own entry).
        NpcKind::ArmedZombie => NpcKind::Zombie,
        NpcKind::ArmedZombieEskimo => NpcKind::ZombieEskimo,
        NpcKind::ArmedZombiePincussion => NpcKind::PincushionZombie,
        NpcKind::ArmedZombieSlimed => NpcKind::SlimedZombie,
        NpcKind::ArmedZombieSwamp => NpcKind::SwampZombie,
        // W4-d40: the bestiary credit folds (`ContentSamples.cs`;
        // the Hide=true variants have no own entry).
        NpcKind::ArmedZombieTwiggy => NpcKind::TwiggyZombie,
        NpcKind::ArmedZombieCenx => NpcKind::FemaleZombie,
        // W4-d40: the bestiary credit folds (`ContentSamples.cs`;
        // the Hide=true variants have no own entry).
        NpcKind::BoneThrowingSkeleton => NpcKind::Skeleton,
        // W4-d40: the bestiary credit folds (`ContentSamples.cs`;
        // the Hide=true variants have no own entry).
        NpcKind::CultistDragonBody1 => NpcKind::CultistDragonHead,
        NpcKind::CultistDragonBody2 => NpcKind::CultistDragonHead,
        // W4-d40: the bestiary credit folds (`ContentSamples.cs`;
        // the Hide=true variants have no own entry).
        NpcKind::CultistDragonBody3 => NpcKind::CultistDragonHead,
        NpcKind::CultistDragonBody4 => NpcKind::CultistDragonHead,
        NpcKind::CultistDragonTail => NpcKind::CultistDragonHead,
        // W4-d40: the bestiary credit folds (`ContentSamples.cs`;
        // the Hide=true variants have no own entry).
        NpcKind::MothronEgg => NpcKind::Mothron,
        // W4-d42 - the Hide=true folds (`NPCID.cs`'s draw modifiers):
        // one credit per family (the SleepingGolfer precedent).
        NpcKind::BartenderUnconscious => NpcKind::DD2Bartender,
        // W4-d42 - the Hide=true folds (`NPCID.cs`'s draw modifiers):
        // one credit per family (the SleepingGolfer precedent).
        NpcKind::ArmedTorchZombie => NpcKind::TorchZombie,
        // W4-c4 merge (148 item 5): the walker's 593 credit folds onto
        // the GoldGoldfish 592 (`ContentSamples.cs:1049`) - c4's entry
        // landed, the interim own-row retired.
        NpcKind::GoldGoldfishWalker => NpcKind::GoldGoldfish,
        // W4-d42 - the Hide=true folds (`NPCID.cs`'s draw modifiers):
        // one credit per family (the SleepingGolfer precedent).
        // (merge-side drop, d37 tables: the fairy/balloon/dragonfly dups
        // 583-585/594/595-601 belong to d33 - their folds land with its
        // small-lanes splice; d42 keeps the waterfowl.)
        // W4-d42 - the Hide=true folds (`NPCID.cs`'s draw modifiers):
        // one credit per family (the SleepingGolfer precedent).
        NpcKind::Seagull2 => NpcKind::Seagull,
        // W4-d42 - the Hide=true folds (`NPCID.cs`'s draw modifiers):
        // one credit per family (the SleepingGolfer precedent).
        NpcKind::Grebe2 => NpcKind::Grebe,
        // W4-d42 - the Hide=true folds (`NPCID.cs`'s draw modifiers):
        // one credit per family (the SleepingGolfer precedent).
        NpcKind::BloodEelBody => NpcKind::BloodEelHead,
        NpcKind::BloodEelTail => NpcKind::BloodEelHead,
        // W4-c8 — the TruffleWormDigger rides the TruffleWorm's row — the
        // vanilla's bestiary draw modifiers hide it (`NPCID.cs:3714-3718`'s
        // `Hide = true`), so it folds onto the visible worm (the Force
        // Bubble precedent; its own ENTRIES entry + cs_id 375 stand for
        // the not-catchable friendly/immunity gate).
        NpcKind::TruffleWormDigger => NpcKind::TruffleWorm,
        // W4-c6 style-59 wiring: the blade's Hide=true draw modifiers
        // (`NPCID.cs:3398-3403`) — no entry of its own; the credit folds
        // onto the Pumpking (the ForceBubble/fishron-hazard precedent).
        // The populator DOES attach invasion-spawn info to entry 328
        // (`BestiaryDatabaseNPCsPopulator.cs:1655`) — the entry exists in
        // the vanilla DB but is hidden from the grid; the port's fold
        // keeps the credit on the boss row like every Hide=true rider.
        NpcKind::PumpkingBlade => NpcKind::Pumpking,
        // The dragonflies (W4-d33): the five Hide=true colours fold onto
        // the visible RedDragonfly 599 card — the kill credit
        // `ContentSamples.cs:1043-1047` (`NpcBestiaryCreditIdsByNpcNetIds`
        // [595/596/597/598/600] = [599]). The Gold 601 keeps its own card.
        NpcKind::BlackDragonfly
        | NpcKind::BlueDragonfly
        | NpcKind::GreenDragonfly
        | NpcKind::OrangeDragonfly
        | NpcKind::YellowDragonfly => NpcKind::RedDragonfly,
        other => other,
    };
    ENTRIES
        .iter()
        .find(|e| {
            e.kind == kind
                && match (e.variant, kind) {
                    (Some(v), NpcKind::Slime) => v == variant,
                    _ => true,
                }
        })
        .expect("every ported NPC kind has a bestiary entry")
}

/// `NPCKillsTracker`'s per-entry progress line — the kill count against the
/// full-unlock count (the `NPCKillCounterInfoElement` display data).
pub fn kill_progress(entry: &BestiaryEntryDef, kills: &KillTracker) -> (i32, i32) {
    (
        kills.get_kill_count(entry.credit_id),
        tr_core::bestiary::kills_needed_for(entry.net_id),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// W4-c8 sweep — the critter entries unlock on SIGHT, not kills
    /// (`CritterUICollectionInfoProvider.cs:14-20`): no sight → tier 0
    /// even with 50 kills; one sight → tier 4 outright with ZERO kills.
    /// W4-c8 final: the nine-kind slice stays pinned (the generic gate
    /// now also covers the Goldfish — its own witness below).
    /// WRITTEN, NOT RUN (code-only mode).
    #[test]
    fn w4_c8_the_critter_entries_unlock_on_sight_not_kills() {
        let mut bestiary = WorldBestiary::new();
        let crawler_entries: Vec<&BestiaryEntryDef> = ENTRIES
            .iter()
            .filter(|e| {
                is_critter_entry(e)
                    && matches!(
                        e.kind,
                        NpcKind::Worm
                            | NpcKind::TruffleWorm
                            | NpcKind::TruffleWormDigger
                            | NpcKind::GoldWorm
                            | NpcKind::EnchantedNightcrawler
                            | NpcKind::Grubby
                            | NpcKind::Sluggy
                            | NpcKind::Buggy
                            | NpcKind::Maggot
                    )
            })
            .collect();
        assert_eq!(crawler_entries.len(), 9, "the nine crawler rows");
        let crawler_entries: Vec<&BestiaryEntryDef> = crawler_entries
            .into_iter()
            .filter(|e| e.kind != NpcKind::GoldWorm)
            .collect();
        assert_eq!(crawler_entries.len(), 8, "the eight non-gold crawler rows");
        for e in &crawler_entries {
            // Kills alone never unlock a critter entry.
            for _ in 0..50 {
                bestiary.kills.register_kill(e.credit_id);
            }
            assert_eq!(
                unlock_state_world(e, &bestiary),
                UnlockState::NotKnownAtAll,
                "{} — kills do not unlock",
                e.credit_id
            );
        }
        // One sight unlocks tier 4 outright (no kill tiers).
        bestiary.set_was_seen_directly("Maggot");
        let maggot = crawler_entries.iter().find(|e| e.credit_id == "Maggot").unwrap();
        assert_eq!(
            unlock_state_world(maggot, &bestiary),
            UnlockState::CanShowDropsWithDropRates,
            "606 — one sight is tier 4 (:18)"
        );
        // The other entries stay locked (per-credit sights).
        let worm = crawler_entries.iter().find(|e| e.credit_id == "Worm").unwrap();
        assert_eq!(unlock_state_world(worm, &bestiary), UnlockState::NotKnownAtAll);
        // A non-critter entry keeps the kill provider (the BlueSlime row).
        let slime = &ENTRIES[0];
        bestiary.kills.register_kill(slime.credit_id);
        assert!(unlock_state_world(slime, &bestiary).known(), "the kill provider stands");
    }

    /// W4-c8 sweep — the GoldWorm's gold provider
    /// (`GoldCritterUICollectionInfoProvider.cs:21-75`): the state is
    /// max(own sight, the 357 peer's sight) — the populator's peer list
    /// `{448: [357]}` (`BestiaryDatabaseNPCsPopulator.cs:715`) — but the
    /// entry stays tier 0 until ANY gold-critter-collection member has
    /// been seen (`:46-52`). WRITTEN, NOT RUN.
    #[test]
    fn w4_c8_the_gold_worm_entry_gates_on_any_gold_critter_seen() {
        let mut bestiary = WorldBestiary::new();
        let gold = ENTRIES.iter().find(|e| e.kind == NpcKind::GoldWorm).unwrap();
        // Nothing seen: locked.
        assert_eq!(unlock_state_world(gold, &bestiary), UnlockState::NotKnownAtAll);
        // The NORMAL peer's sight alone is not enough — the collection
        // gate holds the entry at 0 (the C# :46-52 early return).
        bestiary.set_was_seen_directly("Worm");
        assert_eq!(
            unlock_state_world(gold, &bestiary),
            UnlockState::NotKnownAtAll,
            "357 seen but no gold critter — the gate holds"
        );
        // ANY gold-collection member seen opens the gate; the 357 peer
        // sight then carries the state to tier 4 (the max, :23-37).
        bestiary.set_was_seen_directly("GoldBunny");
        assert_eq!(
            unlock_state_world(gold, &bestiary),
            UnlockState::CanShowDropsWithDropRates,
            "the peer sight + any gold seen = tier 4"
        );
        // The own sight alone also unlocks (max picks it, gate passes).
        let mut b2 = WorldBestiary::new();
        b2.set_was_seen_directly("GoldWorm");
        assert_eq!(
            unlock_state_world(gold, &b2),
            UnlockState::CanShowDropsWithDropRates
        );
        // A non-peer gold member's sight does NOT carry the state — the
        // gate opens but max(own, 357) is still 0.
        let mut b3 = WorldBestiary::new();
        b3.set_was_seen_directly("GoldFrog");
        assert_eq!(
            unlock_state_world(gold, &b3),
            UnlockState::NotKnownAtAll,
            "the gate opens but neither 448 nor 357 was seen"
        );
    }

    /// W4-c8 sweep — the completion fraction rides the sight-aware
    /// dispatch: a critter sight moves the bar exactly like a kill of a
    /// non-critter entry. WRITTEN, NOT RUN.
    #[test]
    fn w4_c8_the_completion_counts_critter_sights() {
        let mut bestiary = WorldBestiary::new();
        assert_eq!(completion_percent_world(&bestiary), 0.0);
        bestiary.set_was_seen_directly("Maggot");
        assert_eq!(
            completion_percent_world(&bestiary),
            1.0 / ENTRIES.len() as f32,
            "one critter sight moves the bar"
        );
        // The kill-only form does NOT see the sight (the two forms
        // differ exactly on the critter rows).
        assert_eq!(completion_percent(&bestiary.kills), 0.0);
    }

    /// W4-c8 final — the GENERIC critter gate
    /// (`NPC.CountsAsACritter`, `NPC.cs:6887-6897`): every ported
    /// 5-HP/0-dmg kind auto-rides the sight provider — the Goldfish 55
    /// (the c4 lane's row, the sweep's recorded follow-up) now unlocks
    /// on sight like the crawlers, and the DD2 lane portal 549 (raw
    /// gate passes but Hide=true → no C# entry, `NPCID.cs:4910-4915`)
    /// keeps the kill provider. The cross-lane contract: c7/c9/c10/e26
    /// kinds are 5-HP/0-dmg in the C# (`NPC.cs:13601-13655` etc.) —
    /// once rowed they pass this gate with NO provider wiring.
    /// WRITTEN, NOT RUN (code-only mode).
    #[test]
    fn w4_c8_final_the_generic_critter_gate_covers_the_goldfish() {
        // The raw gate: the nine crawlers + the Goldfish pass; the
        // hostile/bulky kinds never do.
        for kind in [
            NpcKind::Worm,
            NpcKind::TruffleWorm,
            NpcKind::TruffleWormDigger,
            NpcKind::GoldWorm,
            NpcKind::EnchantedNightcrawler,
            NpcKind::Grubby,
            NpcKind::Sluggy,
            NpcKind::Buggy,
            NpcKind::Maggot,
            NpcKind::Goldfish,
        ] {
            assert!(counts_as_a_critter(kind), "{kind:?} is a critter");
        }
        for kind in [
            NpcKind::Zombie,       // 45 HP / 14 dmg
            NpcKind::Piranha,      // 30 HP / 25 dmg
            NpcKind::ForceBubble,  // 1000 HP (0 dmg alone is not enough)
            NpcKind::DetonatingBubble, // 1 HP but 100 dmg
            NpcKind::FungiSpore,   // 1 HP but 80 dmg
        ] {
            assert!(!counts_as_a_critter(kind), "{kind:?} is NOT a critter");
        }
        // The Goldfish entry dispatches to the sight provider now.
        let fish = ENTRIES.iter().find(|e| e.kind == NpcKind::Goldfish).unwrap();
        assert!(is_critter_entry(fish));
        let mut bestiary = WorldBestiary::new();
        for _ in 0..50 {
            bestiary.kills.register_kill("Goldfish");
        }
        assert_eq!(
            unlock_state_world(fish, &bestiary),
            UnlockState::NotKnownAtAll,
            "55 — kills do not unlock the critter row"
        );
        bestiary.set_was_seen_directly("Goldfish");
        assert_eq!(
            unlock_state_world(fish, &bestiary),
            UnlockState::CanShowDropsWithDropRates,
            "55 — one sight is tier 4"
        );
        // The 549 lane portal: raw gate true, Hide exclusion — the kill
        // provider stands (the ForceBubble precedent).
        assert!(counts_as_a_critter(NpcKind::DD2LanePortal));
        let portal = ENTRIES
            .iter()
            .find(|e| e.kind == NpcKind::DD2LanePortal)
            .unwrap();
        assert!(!is_critter_entry(portal), "549 Hide — no critter entry");
        let mut b2 = WorldBestiary::new();
        b2.set_was_seen_directly("DD2LanePortal");
        assert_eq!(
            unlock_state_world(portal, &b2),
            UnlockState::NotKnownAtAll,
            "a sight does not unlock the kill-provider row"
        );
    }

    /// W4-c8 final — the gold set census: the 14-member collection
    /// matches `NPCID.cs:10121-10125` verbatim and every collection id
    /// opens the gate (the `TryFindingOneGoldCritterThatIsAlreadyUnlocked`
    /// loop `:56-66`); the peer table carries all 13 registrations
    /// (`BestiaryDatabaseNPCsPopulator.cs:703-716`) — 593 is collection-
    /// only (no gold provider, the plain critter row). Each member's
    /// sight + the GoldWorm's own sight unlocks 448 — proving the gate
    /// reads the WHOLE set, not just 448's own id. WRITTEN, NOT RUN.
    #[test]
    fn w4_c8_final_the_gold_collection_is_the_full_fourteen() {
        let gold = ENTRIES.iter().find(|e| e.kind == NpcKind::GoldWorm).unwrap();
        // The collection: the C# list order (443/442/592/593/444/601/
        // 445/446/605/447/627/613/448/539) as credit ids.
        let collection = [
            ("GoldBunny", 443u16),
            ("GoldBird", 442),
            ("GoldGoldfish", 592),
            ("GoldGoldfishWalker", 593),
            ("GoldButterfly", 444),
            ("GoldDragonfly", 601),
            ("GoldFrog", 445),
            ("GoldGrasshopper", 446),
            ("GoldLadyBug", 605),
            ("GoldMouse", 447),
            ("GoldSeahorse", 627),
            ("GoldWaterStrider", 613),
            ("GoldWorm", 448),
            ("SquirrelGold", 539),
        ];
        for (credit, _net) in collection {
            let mut b = WorldBestiary::new();
            // The member seen + the GoldWorm itself seen: gate opens,
            // max(own, 357) = tier 4.
            b.set_was_seen_directly(credit);
            b.set_was_seen_directly("GoldWorm");
            assert_eq!(
                unlock_state_world(gold, &b),
                UnlockState::CanShowDropsWithDropRates,
                "{credit} opens the collection gate"
            );
        }
        // The peer table: 13 rows, 593 absent (collection-only), the
        // 539 squirrel the one two-peer row.
        assert_eq!(GOLD_PEER_TABLE.len(), 13);
        assert!(!GOLD_PEER_TABLE.iter().any(|(net, _)| *net == 593));
        let squirrel = GOLD_PEER_TABLE.iter().find(|(net, _)| *net == 539).unwrap();
        assert_eq!(squirrel.1, &["Squirrel", "SquirrelRed"]);
        // The ported gold row (448) dispatches through the table; a
        // non-table critter (357) does not gate on the collection.
        let worm = ENTRIES.iter().find(|e| e.kind == NpcKind::Worm).unwrap();
        let mut b3 = WorldBestiary::new();
        b3.set_was_seen_directly("Worm");
        assert_eq!(
            unlock_state_world(worm, &b3),
            UnlockState::CanShowDropsWithDropRates,
            "357 is the plain critter provider — no gold gate"
        );
    }

    #[test]
    fn every_ported_kind_maps_to_its_entry() {
        for kind in [
            NpcKind::Slime,
            NpcKind::DemonEye,
            NpcKind::WanderingEye,
            NpcKind::Zombie,
            NpcKind::KingSlime,
        ] {
            for variant in [
                SlimeVariant::Blue,
                SlimeVariant::Green,
                SlimeVariant::Pinky,
                SlimeVariant::Baby,
                SlimeVariant::Black,
                SlimeVariant::Purple,
                SlimeVariant::Red,
                SlimeVariant::Yellow,
            ] {
                let entry = entry_for_npc(kind, variant);
                if kind == NpcKind::Slime {
                    assert_eq!(entry.variant, Some(variant));
                }
                assert_eq!(entry.kind, kind);
            }
        }
        // The credit ids are unique (`ContentSamples`' dictionary keys).
        let mut ids: Vec<_> = ENTRIES.iter().map(|e| e.credit_id).collect();
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), ENTRIES.len());
    }

    #[test]
    fn the_mothron_egg_resolves_through_its_own_row_but_credits_mothron() {
        // W4-d40 seal witness (WRITTEN NOT RUN — code-only): the g61
        // research/273 §3 478-resolution-gap fix — the Hide=true egg
        // (`NPCID.cs:4405-4410`) carries an own ENTRIES row so
        // `kind_from_net_id(478)` resolves (the g8 VampireBat precedent),
        // while the credit fold keeps `entry_for_npc` on Mothron's row.
        let (kind, _) = crate::npc::kind_from_net_id(478)
            .expect("the egg's own row resolves the net id");
        assert_eq!(kind, NpcKind::MothronEgg);
        let entry = entry_for_npc(NpcKind::MothronEgg, SlimeVariant::Blue);
        assert_eq!(entry.net_id, 477, "the credit folds onto Mothron");
        assert_eq!(entry.credit_id, "Mothron");
        // The spawn keeps its own row (the Transform(479) target).
        let (kind, _) = crate::npc::kind_from_net_id(479).expect("the spawn row");
        assert_eq!(kind, NpcKind::MothronSpawn);
    }

    #[test]
    fn the_frost_trio_credit_ids_follow_the_vanilla_npc_names() {
        // `ContentSamples.cs:868-870`: the bestiary credit id is
        // `NPCID.Search.GetName(netID)` — net 144 is `MisterStabby` and
        // net 145 is `SnowBalla` (`NPCID.cs:10965`/`:10967`), whatever
        // the port's kind names call them.
        let stabby = entry_for_npc(NpcKind::SnowBalla, SlimeVariant::Blue);
        assert_eq!(stabby.net_id, 144);
        assert_eq!(stabby.credit_id, "MisterStabby");
        assert_eq!(stabby.name, "Mister Stabby");
        let balla = entry_for_npc(NpcKind::MysterySnowman, SlimeVariant::Blue);
        assert_eq!(balla.net_id, 145);
        assert_eq!(balla.credit_id, "SnowBalla");
        assert_eq!(balla.name, "Snow Balla");
        let gangsta = entry_for_npc(NpcKind::SnowmanGangsta, SlimeVariant::Blue);
        assert_eq!(gangsta.net_id, 143);
        assert_eq!(gangsta.credit_id, "SnowmanGangsta");
    }

    #[test]
    fn the_tiers_climb_with_kills() {
        let mut kills = KillTracker::new();
        let blue = &ENTRIES[0];
        assert_eq!(unlock_state(blue, &kills), UnlockState::NotKnownAtAll);
        kills.register_kill("BlueSlime");
        assert_eq!(unlock_state(blue, &kills), UnlockState::CanShowPortraitOnly);
        for _ in 0..9 {
            kills.register_kill("BlueSlime");
        }
        assert_eq!(unlock_state(blue, &kills), UnlockState::CanShowStats);
        // Pinky needs only 25 for the full reveal.
        let pinky = &ENTRIES[2];
        for _ in 0..25 {
            kills.register_kill("Pinky");
        }
        assert_eq!(
            unlock_state(pinky, &kills),
            UnlockState::CanShowDropsWithDropRates
        );
        // The boss quick-unlocks on the first kill. (Index 11 since the
        // S1-D-3ff roster inserted MotherSlime at its vanilla-net-id slot.)
        let king = &ENTRIES[11];
        assert_eq!(king.quick_unlock, true);
        kills.register_kill("KingSlime");
        assert_eq!(
            unlock_state(king, &kills),
            UnlockState::CanShowDropsWithDropRates
        );
    }

    /// W4-d38 FINAL (research/293 §8) — the 35 kinds' unlock ladders match
    /// the C# banner chain: `GetKillCountNeeded`
    /// (`CommonEnemyUICollectionInfoProvider.cs:21-34`) =
    /// `KillsToBanner[BannerToItem(NPCtoBanner(BannerID()))]`
    /// (`BannerSystem.cs:341/:403`, `ItemID.cs:1303-1304`). Mechanically
    /// re-derived by `tools/_d38_final_unlock.py` (all 35 chains agree):
    /// 290 Paladin is the ONE non-default ladder (banner 149 → item 2958 →
    /// 25 kills, tiers 1/5/12/25); every other kind rides the default 50
    /// (tiers 1/10/25/50). The 6 critters' COMPUTED ladder is pinned here
    /// too — their vanilla provider is sight-based
    /// (`CritterUICollectionInfoProvider.cs:14-20`, GAP-1 in research/293
    /// §8, a pre-existing base-wide gap the wiring round owns), so this
    /// witness pins the port's CURRENT behavior, not the vanilla tier.
    #[test]
    fn w4_d38_final_the_unlock_ladders_match_the_banner_chain() {
        use crate::npc::NpcKind as K;
        let d38 = [
            K::RaggedCaster, K::RaggedCasterOpenCoat, K::Necromancer,
            K::NecromancerArmored, K::DiabolistRed, K::DiabolistWhite,
            K::BoneLee, K::GiantCursedSkull, K::Paladin, K::SkeletonSniper,
            K::TacticalSkeleton, K::SkeletonCommando, K::AngryBonesBig,
            K::AngryBonesBigMuscle, K::AngryBonesBigHelmet, K::BirdBlue,
            K::BirdRed, K::Squirrel, K::Mouse, K::Raven, K::SlimeMasked,
            K::BunnySlimed, K::HoppinJack, K::Ghost, K::DemonEyeOwl,
            K::DemonEyeSpaceship, K::SkeletonTopHat, K::SkeletonAstonaut,
            K::SkeletonAlien, K::PumpkingBlade, K::SlimeRibbonWhite,
            K::SlimeRibbonYellow, K::SlimeRibbonGreen, K::SlimeRibbonRed,
            K::BunnyXmas,
        ];
        assert_eq!(d38.len(), 35);
        for kind in d38 {
            let e = entry_for_npc(kind, SlimeVariant::Blue);
            // protocol == vanilla (research/293 §9, the 108 corridor).
            assert_eq!(e.net_id, kind.cs_id() as i32, "{kind:?}");
            // No boss / ShouldBeCountedAsBossForBestiary member in the band
            // (`NPCID.cs:10481` = {517,422,507,493,13,664}).
            assert!(!e.quick_unlock, "{kind:?}");
            let full = tr_core::bestiary::kills_needed_for(e.net_id);
            let want = if kind == K::Paladin { 25 } else { 50 };
            assert_eq!(full, want, "{kind:?} full kill count");
            // The tier ladder at the exact thresholds (the C#'s integer
            // half/fifth, `CommonEnemyUICollectionInfoProvider.cs:53-54`).
            let mut kills = KillTracker::new();
            assert_eq!(unlock_state(e, &kills), UnlockState::NotKnownAtAll);
            kills.set_kill_count_directly(e.credit_id, 1);
            assert_eq!(unlock_state(e, &kills), UnlockState::CanShowPortraitOnly);
            kills.set_kill_count_directly(e.credit_id, full / 5);
            assert_eq!(unlock_state(e, &kills), UnlockState::CanShowStats);
            kills.set_kill_count_directly(e.credit_id, full / 2);
            assert_eq!(unlock_state(e, &kills), UnlockState::CanShowDropsWithoutDropRates);
            kills.set_kill_count_directly(e.credit_id, full);
            assert_eq!(unlock_state(e, &kills), UnlockState::CanShowDropsWithDropRates);
        }
        // The 6 critters (`CountsAsACritter`, `NPC.cs:6887-6897`: lifeMax 5
        // + damage 0) — their entries exist on the kill ladder today
        // (GAP-1: vanilla's sight → tier 4 provider is unwired).
        for kind in [K::BirdBlue, K::BirdRed, K::Squirrel, K::Mouse,
                     K::BunnySlimed, K::BunnyXmas] {
            let e = entry_for_npc(kind, SlimeVariant::Blue);
            let kills = KillTracker::new();
            assert_eq!(unlock_state(e, &kills), UnlockState::NotKnownAtAll,
                "{kind:?}: the port needs kills where vanilla needs a sighting");
        }
        // 328 PumpkingBlade: the vanilla draw modifiers Hide it
        // (`NPCID.cs`'s row, ported verbatim) — the populator's
        // `GetExclusions` (`BestiaryDatabaseNPCsPopulator.cs:870-886`)
        // gives it NO entry; the port's extra row is GAP-3 (research/293
        // §8), pinned here so the wiring-round drop is a deliberate edit.
        assert!(crate::ui::bestiary_draw_offsets::get(328).unwrap().hide);
        assert!(ENTRIES.iter().any(|e| e.net_id == 328),
            "GAP-3: the port carries the Hide kind's entry (vanilla has none)");
    }
}
