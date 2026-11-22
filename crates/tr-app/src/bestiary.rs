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
    /// (`BestiaryEntry.cs:37-41`: `nPC.boss ||
    /// NPCID.Sets.ShouldBeCountedAsBossForBestiary`) — one kill reveals
    /// tier 4 outright. The full kill count stays the banner chain's
    /// (`kills_needed_for`, default 50); quick-unlock is a flag, not a
    /// zeroed count. The per-entry provider overrides (town chat /
    /// critter sight / the boss-credit folds) live in `bestiary_data`.
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
    // W4-f42 — the sandstorm debris + desert family (the `NPC.cs:3949-4018`
    // spawn arm's picks). The vanilla net ids are the bestiary credits.
    BestiaryEntryDef {
        credit_id: "Tumbleweed",
        net_id: 546,
        kind: NpcKind::Tumbleweed,
        variant: None,
        quick_unlock: false,
        name: "Tumbleweed",
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
        credit_id: "Antlion",
        net_id: 69,
        kind: NpcKind::Antlion,
        variant: None,
        quick_unlock: false,
        name: "Antlion",
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
        credit_id: "BloodMummy",
        net_id: 630,
        kind: NpcKind::BloodMummy,
        variant: None,
        quick_unlock: false,
        name: "Blood Mummy",
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
        credit_id: "WalkingAntlion",
        net_id: 580,
        kind: NpcKind::WalkingAntlion,
        variant: None,
        quick_unlock: false,
        name: "Walking Antlion",
    },
    BestiaryEntryDef {
        credit_id: "FlyingAntlion",
        net_id: 581,
        kind: NpcKind::FlyingAntlion,
        variant: None,
        quick_unlock: false,
        name: "Flying Antlion",
    },
    // The spheres: the Chaos Ball entry also covers Tim's ChaosBallTim
    // clone 665 (the port folds it — identical stats and name).
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
    BestiaryEntryDef {
        credit_id: "ForceBubble",
        net_id: 384,
        kind: NpcKind::ForceBubble,
        variant: None,
        quick_unlock: false,
        name: "Force Bubble",
    },
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
    BestiaryEntryDef {
        credit_id: "Seagull",
        net_id: 602,
        kind: NpcKind::Seagull,
        variant: None,
        quick_unlock: false,
        name: "Seagull",
    },
    BestiaryEntryDef {
        credit_id: "Seagull2",
        net_id: 603,
        kind: NpcKind::Seagull2,
        variant: None,
        quick_unlock: false,
        name: "Seagull",
    },
    BestiaryEntryDef {
        credit_id: "Grebe",
        net_id: 608,
        kind: NpcKind::Grebe,
        variant: None,
        quick_unlock: false,
        name: "Grebe",
    },
    BestiaryEntryDef {
        credit_id: "Grebe2",
        net_id: 609,
        kind: NpcKind::Grebe2,
        variant: None,
        quick_unlock: false,
        name: "Grebe",
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
        credit_id: "GoldButterfly",
        net_id: 444,
        kind: NpcKind::GoldButterfly,
        variant: None,
        quick_unlock: false,
        name: "Gold Butterfly",
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
        credit_id: "GoldMouse",
        net_id: 447,
        kind: NpcKind::GoldMouse,
        variant: None,
        quick_unlock: false,
        name: "Gold Mouse",
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
    // W4-c4 (the style-44 swimmers): the four kinds' own entries — the
    // populator attaches info to all four
    // (`BestiaryDatabaseNPCsPopulator.cs:1133`/`:1523`/`:629`/`:1164`).
    // The display names ride `en-US NPCs.json` (581 is "Antlion
    // Swarmer", 509 "Giant Antlion Swarmer", 587 "Wandering Eye Fish" —
    // NOT the credit-id spellings).
    BestiaryEntryDef {
        credit_id: "FlyingFish",
        net_id: 224,
        kind: NpcKind::FlyingFish,
        variant: None,
        quick_unlock: false,
        name: "Flying Fish",
    },
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
    BestiaryEntryDef {
        credit_id: "CorruptGoldfish",
        net_id: 57,
        kind: NpcKind::CorruptGoldfish,
        variant: None,
        quick_unlock: false,
        name: "Corrupt Goldfish",
    },
    BestiaryEntryDef {
        credit_id: "Arapaima",
        net_id: 157,
        kind: NpcKind::Arapaima,
        variant: None,
        quick_unlock: false,
        name: "Arapaima",
    },
    BestiaryEntryDef {
        credit_id: "BloodFeeder",
        net_id: 241,
        kind: NpcKind::BloodFeeder,
        variant: None,
        quick_unlock: false,
        name: "Blood Feeder",
    },
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
    BestiaryEntryDef {
        credit_id: "Pufferfish",
        net_id: 688,
        kind: NpcKind::Pufferfish,
        variant: None,
        quick_unlock: false,
        name: "Pufferfish",
    },
    BestiaryEntryDef {
        credit_id: "Orca",
        net_id: 692,
        kind: NpcKind::Orca,
        variant: None,
        quick_unlock: false,
        name: "Orca",
    },
    // The man-eater family (W4-d23, aiStyle 13): the six wired kinds' own
    // entries — the populator registers all six by id (the normal-enemy
    // kill ladder, no quick-unlock).
    BestiaryEntryDef {
        credit_id: "ManEater",
        net_id: 43,
        kind: NpcKind::ManEater,
        variant: None,
        quick_unlock: false,
        name: "Man Eater",
    },
    BestiaryEntryDef {
        credit_id: "Snatcher",
        net_id: 56,
        kind: NpcKind::Snatcher,
        variant: None,
        quick_unlock: false,
        name: "Snatcher",
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
        credit_id: "AngryTrapper",
        net_id: 175,
        kind: NpcKind::AngryTrapper,
        variant: None,
        quick_unlock: false,
        name: "Angry Trapper",
    },
    BestiaryEntryDef {
        credit_id: "FungiBulb",
        net_id: 259,
        kind: NpcKind::FungiBulb,
        variant: None,
        quick_unlock: false,
        name: "Fungi Bulb",
    },
    BestiaryEntryDef {
        credit_id: "GiantFungiBulb",
        net_id: 260,
        kind: NpcKind::GiantFungiBulb,
        variant: None,
        quick_unlock: false,
        name: "Giant Fungi Bulb",
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
    },
    // The aiStyle-14 flyer family (W4-d23): the 13 wired kinds' own
    // entries — the populator attaches info to every id (48
    // `BestiaryDatabaseNPCsPopulator.cs:1653`, 62 `:1548`, 93 `:1129`,
    // 121 `:1271`, 137 `:1130`, 150 `:1352`, 151 `:1353`, 152 `:1317`,
    // 156 `:1673`, 158 `:1356`, 159 `:1465`, 226 `:1553`, 634 `:1950`);
    // 660's UI provider credits 657 (`:694`'s quick-unlock map) but its
    // entry + draw sheet ride its own id (the Sharkron precedent).
    BestiaryEntryDef {
        credit_id: "Harpy",
        net_id: 48,
        kind: NpcKind::Harpy,
        variant: None,
        quick_unlock: false,
        name: "Harpy",
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
        credit_id: "GiantBat",
        net_id: 93,
        kind: NpcKind::GiantBat,
        variant: None,
        quick_unlock: false,
        name: "Giant Bat",
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
        credit_id: "IlluminantBat",
        net_id: 137,
        kind: NpcKind::IlluminantBat,
        variant: None,
        quick_unlock: false,
        name: "Illuminant Bat",
    },
    BestiaryEntryDef {
        credit_id: "IceBat",
        net_id: 150,
        kind: NpcKind::IceBat,
        variant: None,
        quick_unlock: false,
        name: "Ice Bat",
    },
    BestiaryEntryDef {
        credit_id: "LavaBat",
        net_id: 151,
        kind: NpcKind::Lavabat,
        variant: None,
        quick_unlock: false,
        name: "Lava Bat",
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
    BestiaryEntryDef {
        credit_id: "Vampire",
        net_id: 159,
        kind: NpcKind::Vampire,
        variant: None,
        quick_unlock: false,
        name: "Vampire",
    },
    BestiaryEntryDef {
        credit_id: "FlyingSnake",
        net_id: 226,
        kind: NpcKind::FlyingSnake,
        variant: None,
        quick_unlock: false,
        name: "Flying Snake",
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
    },
    // W4-f18 first-20 port — batch 1's five critters. The credit ids are
    // the vanilla `NPCID` names (`BestiaryDatabaseNPCsPopulator.cs`'s
    // `ContentSamples.NpcBestiaryCreditIdsByNpcNetIds` rows); the display
    // names ride `en-US.NPCs.json`'s NPCName rows.
    BestiaryEntryDef {
        credit_id: "Firefly",
        net_id: 355,
        kind: NpcKind::Firefly,
        variant: None,
        quick_unlock: false,
        name: "Firefly",
    },
    BestiaryEntryDef {
        credit_id: "Butterfly",
        net_id: 356,
        kind: NpcKind::Butterfly,
        variant: None,
        quick_unlock: false,
        name: "Butterfly",
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
        credit_id: "Snail",
        net_id: 359,
        kind: NpcKind::Snail,
        variant: None,
        quick_unlock: false,
        name: "Snail",
    },
    // W4-f18 batch 2 — the GlowingSnail, the Frog and the two walker
    // ducks. The Duck2 363 is Hide=true (`NPCID.cs:3617-3622`) — it folds
    // onto the Duck 362's entry (no row of its own, the DetonatingBubble
    // precedent); the credit ids ride the vanilla NPCID names.
    BestiaryEntryDef {
        credit_id: "GlowingSnail",
        net_id: 360,
        kind: NpcKind::GlowingSnail,
        variant: None,
        quick_unlock: false,
        name: "Glowing Snail",
    },
    BestiaryEntryDef {
        credit_id: "Frog",
        net_id: 361,
        kind: NpcKind::Frog,
        variant: None,
        quick_unlock: false,
        name: "Frog",
    },
    BestiaryEntryDef {
        credit_id: "ScorpionBlack",
        net_id: 366,
        kind: NpcKind::ScorpionBlack,
        variant: None,
        quick_unlock: false,
        name: "Scorpion",
    },
    BestiaryEntryDef {
        credit_id: "Scorpion",
        net_id: 367,
        kind: NpcKind::Scorpion,
        variant: None,
        quick_unlock: false,
        name: "Scorpion",
    },
    BestiaryEntryDef {
        credit_id: "TruffleWorm",
        net_id: 374,
        kind: NpcKind::TruffleWorm,
        variant: None,
        quick_unlock: false,
        name: "Truffle Worm",
    },
    // W4-f18 batch 4 — the Grasshopper, the ChatteringTeethBomb, the
    // CultistArcherBlue and the ArmedZombie. The CultistArcherWhite 380
    // (Hide=true, `NPCID.cs:3734-3739`) folds onto the CultistArcherBlue
    // 379 — no row of its own (the DetonatingBubble precedent).
    BestiaryEntryDef {
        credit_id: "Grasshopper",
        net_id: 377,
        kind: NpcKind::Grasshopper,
        variant: None,
        quick_unlock: false,
        name: "Grasshopper",
    },
    BestiaryEntryDef {
        credit_id: "ChatteringTeethBomb",
        net_id: 378,
        kind: NpcKind::ChatteringTeethBomb,
        variant: None,
        quick_unlock: false,
        name: "Chattering Teeth Bomb",
    },
    BestiaryEntryDef {
        credit_id: "CultistArcherBlue",
        net_id: 379,
        kind: NpcKind::CultistArcherBlue,
        variant: None,
        quick_unlock: false,
        name: "Cultist Archer",
    },
    BestiaryEntryDef {
        credit_id: "ArmedZombie",
        net_id: 430,
        kind: NpcKind::ArmedZombie,
        variant: None,
        quick_unlock: false,
        name: "Zombie",
    },
];

/// The entry's current unlock tier — its `UIInfoProvider`'s
/// `GetEntryUICollectionInfo` (`BestiaryEntryUnlockState`). The provider
/// is `bestiary_data::gate(entry.net_id)` (the populator's override walk,
/// `BestiaryDatabaseNPCsPopulator.cs:649-730`); the three trackers are the
/// world's `BestiaryUnlocksTracker` (`Main.BestiaryTracker`).
pub fn unlock_state(entry: &BestiaryEntryDef, tracker: &WorldBestiary) -> UnlockState {
    eval_gate(
        crate::bestiary_data::gate(entry.net_id),
        entry.quick_unlock,
        tracker,
    )
}

/// One provider's evaluation. `entry_quick` is the entry factory's
/// boss-rule flag (`BestiaryEntry.cs:37-41`), ORed onto the Common
/// provider's own `quickUnlock` argument.
fn eval_gate(
    gate: crate::bestiary_data::Gate,
    entry_quick: bool,
    tracker: &WorldBestiary,
) -> UnlockState {
    use crate::bestiary_data::Gate;
    match gate {
        Gate::Common { credit_net, quick } => {
            // `CommonEnemyUICollectionInfoProvider.GetEntryUICollectionInfo`
            // (`:36-43`): the ladder over the CREDIT's kill count — the
            // fold rows read the boss's credit (`GetKillCountNeeded:21-34`
            // resolves the same credit's banner chain).
            let credit = credit_id_for_net(credit_net);
            let full = tr_core::bestiary::kills_needed_for(credit_net);
            get_unlock_state_by_kill_count(
                tracker.kills.get_kill_count(credit),
                entry_quick || quick,
                full,
            )
        }
        Gate::Town { credit_net } => {
            // `TownNPCUICollectionInfoProvider.cs:14-20`: chatted → 4.
            if tracker.was_chat_with(credit_id_for_net(credit_net)) {
                UnlockState::CanShowDropsWithDropRates
            } else {
                UnlockState::NotKnownAtAll
            }
        }
        Gate::Critter { credit_net } => {
            // `CritterUICollectionInfoProvider.cs:14-20`: sighted → 4.
            if tracker.was_seen_near_player(credit_id_for_net(credit_net)) {
                UnlockState::CanShowDropsWithDropRates
            } else {
                UnlockState::NotKnownAtAll
            }
        }
        Gate::Highest(gates) => {
            // `HighestOfMultipleUICollectionInfoProvider.cs:17-31`: the
            // max over the sub-providers (the entry flag rides each
            // Common half — the C# bakes it into the provider ctor).
            gates
                .iter()
                .map(|g| eval_gate(*g, entry_quick, tracker))
                .max()
                .unwrap_or(UnlockState::NotKnownAtAll)
        }
    }
}

/// `ContentSamples.NpcBestiaryCreditIdsByNpcNetIds[netId]`
/// (`ContentSamples.cs:862-873` + `ModifyNPCIds`): the port's roster rows
/// carry the credit ids verbatim; an id without a row (the unported
/// override targets) falls back to the raw net id's name lookup the same
/// way the roster does — `None` keeps the caller on the default ladder.
pub(crate) fn credit_id_for_net(net_id: i32) -> &'static str {
    ENTRIES
        .iter()
        .find(|e| e.net_id == net_id)
        .map(|e| e.credit_id)
        .unwrap_or("")
}

/// `Main.GetBestiaryProgressReport()`'s completion fraction (the
/// `BestiaryUI` report): the ported entries past the portrait-only tier
/// over the whole roster. The Zoologist's shop gates read it
/// (`Chest.cs:3265`).
pub fn completion_percent(tracker: &WorldBestiary) -> f32 {
    // `BestiaryUnlockProgressReport.CompletionPercent`
    // (`BestiaryUnlockProgressReport.cs:9-19`): 1 for an empty roster,
    // else the known share (`UIBestiaryTest.GetUnlockProgress`, `:497-513`,
    // counts `> NotKnownAtAll_0`).
    if ENTRIES.is_empty() {
        return 1.0;
    }
    let seen = ENTRIES
        .iter()
        .filter(|e| unlock_state(e, tracker).known())
        .count();
    seen as f32 / ENTRIES.len() as f32
}

/// `Chest.BestiaryGirl_IsFairyTorchAvailable` (`Chest.cs:3544-3559`): the
/// Zoologist's Fairy Glowstick 4776 head row needs ALL THREE Fairy Critter
/// entries discovered — `DidDiscoverBestiaryEntry(585)` then `(584)` then
/// `(583)` (`:3561-3564`: `UnlockState > NotKnownAtAll_0`). The fairy
/// critters are `CountsAsACritter` (`NPC.cs:6887-6897` — lifeMax 5, damage
/// 0, `NPC.cs:16583-16607`), so the automated populator registers them
/// with the CRITTER provider (`BestiaryDatabaseNPCsPopulator.cs:851-860`)
/// whose unlock reads the SIGHTS tracker, not kills
/// (`CritterUICollectionInfoProvider.cs:15-21`:
/// `Sights.GetWasNearbyBefore(creditId)` → tier 4, else 0). The port has
/// no fairy-critter NPC kinds yet, so the three credit ids
/// (`NPCID.cs:11843-11847` = the `NPCID.Search` names) read the sights set
/// directly; the gate stays closed until the critter-sighting wiring
/// lands (the recorded deviation — the vanilla AI_112 nearby check,
/// `NPC.cs:48642-48646`, is unported).
pub fn fairy_torch_available(bestiary: &tr_core::bestiary::WorldBestiary) -> bool {
    ["FairyCritterBlue", "FairyCritterGreen", "FairyCritterPink"]
        .iter()
        .all(|id| bestiary.was_seen_near_player(id))
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
        // W4-c6 style-59 wiring: the blade's bestiary draw modifiers are
        // Hide=true (`NPCID.cs:3398-3403`) — the credit folds onto the
        // Pumpking (the fishron-hazard precedent). The DRAW sheet stays
        // NPC_328 through `texture_net_id`'s own row.
        NpcKind::PumpkingBlade => NpcKind::Pumpking,
        // W4-f18 batch 2 — the Duck2 363 is Hide=true (`NPCID.cs:3617-
        // 3622`): the swimming form folds onto the walker Duck 362's
        // entry (one credit per waterfowl, the DetonatingBubble precedent).
        NpcKind::Duck2 => NpcKind::Duck,
        // W4-f18 batch 3 — the DuckWhite2 365 folds onto the DuckWhite 364
        // (Hide=true, `NPCID.cs:3633-3638`); the TruffleWormDigger 375
        // folds onto the TruffleWorm 374 (Hide=true, `NPCID.cs:3713-3718`).
        NpcKind::DuckWhite2 => NpcKind::DuckWhite,
        NpcKind::TruffleWormDigger => NpcKind::TruffleWorm,
        // W4-f18 batch 4 — the CultistArcherWhite 380 folds onto the
        // CultistArcherBlue 379 (Hide=true, `NPCID.cs:3734-3739`).
        NpcKind::CultistArcherWhite => NpcKind::CultistArcherBlue,
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
        // W4-c6 style-59 wiring: the blade's Hide=true draw modifiers
        // (`NPCID.cs:3398-3403`) — no entry of its own; the credit folds
        // onto the Pumpking (the ForceBubble/fishron-hazard precedent).
        // The populator DOES attach invasion-spawn info to entry 328
        // (`BestiaryDatabaseNPCsPopulator.cs:1655`) — the entry exists in
        // the vanilla DB but is hidden from the grid; the port's fold
        // keeps the credit on the boss row like every Hide=true rider.
        NpcKind::PumpkingBlade => NpcKind::Pumpking,
        // W4-f18 batch 2 — the Duck2 363 folds onto the Duck 362's row
        // (Hide=true, `NPCID.cs:3617-3622`; the try_entry twin above).
        NpcKind::Duck2 => NpcKind::Duck,
        // W4-f18 batch 3 — the DuckWhite2 365 / TruffleWormDigger 375 fold
        // onto their walker rows (the try_entry twin above).
        NpcKind::DuckWhite2 => NpcKind::DuckWhite,
        NpcKind::TruffleWormDigger => NpcKind::TruffleWorm,
        // W4-f18 batch 4 — the CultistArcherWhite 380 folds onto the
        // CultistArcherBlue 379 (the try_entry twin above).
        NpcKind::CultistArcherWhite => NpcKind::CultistArcherBlue,
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
        let mut tracker = WorldBestiary::new();
        let blue = &ENTRIES[0];
        assert_eq!(unlock_state(blue, &tracker), UnlockState::NotKnownAtAll);
        tracker.kills.register_kill("BlueSlime");
        assert_eq!(
            unlock_state(blue, &tracker),
            UnlockState::CanShowPortraitOnly
        );
        for _ in 0..9 {
            tracker.kills.register_kill("BlueSlime");
        }
        assert_eq!(unlock_state(blue, &tracker), UnlockState::CanShowStats);
        // Pinky needs only 25 for the full reveal.
        let pinky = &ENTRIES[2];
        for _ in 0..25 {
            tracker.kills.register_kill("Pinky");
        }
        assert_eq!(
            unlock_state(pinky, &tracker),
            UnlockState::CanShowDropsWithDropRates
        );
        // The boss quick-unlocks on the first kill. (Index 11 since the
        // S1-D-3ff roster inserted MotherSlime at its vanilla-net-id slot.)
        let king = &ENTRIES[11];
        assert_eq!(king.quick_unlock, true);
        tracker.kills.register_kill("KingSlime");
        assert_eq!(
            unlock_state(king, &tracker),
            UnlockState::CanShowDropsWithDropRates
        );
    }

    /// W4-c37: the whole roster reads UNKNOWN on a fresh world — the
    /// research/30 #4 witness (the old default-0 denominator reported
    /// tier 4 for 164 of the entries with zero kills).
    #[test]
    fn w4_c37_a_fresh_world_knows_nothing() {
        let tracker = WorldBestiary::new();
        for e in ENTRIES {
            assert_eq!(
                unlock_state(e, &tracker),
                UnlockState::NotKnownAtAll,
                "{} must start locked",
                e.credit_id
            );
        }
        assert_eq!(completion_percent(&tracker), 0.0);
    }

    /// W4-c37: the town rows ride the CHAT provider, not the kill ladder
    /// (`TownNPCUICollectionInfoProvider.cs:14-20`) — kills never unlock
    /// the Guide; one chat jumps straight to tier 4.
    #[test]
    fn w4_c37_the_town_rows_unlock_by_chat_not_kills() {
        let mut tracker = WorldBestiary::new();
        let guide = ENTRIES.iter().find(|e| e.credit_id == "Guide").unwrap();
        for _ in 0..100 {
            tracker.kills.register_kill("Guide");
        }
        assert_eq!(unlock_state(guide, &tracker), UnlockState::NotKnownAtAll);
        tracker.set_was_chat_with_directly("Guide");
        assert_eq!(
            unlock_state(guide, &tracker),
            UnlockState::CanShowDropsWithDropRates
        );
    }

    /// W4-c37: the critter rows ride the SIGHT provider
    /// (`CritterUICollectionInfoProvider.cs:14-20`) — the Goldfish 55 and
    /// the Etherian Portal 549 (`CountsAsACritter`, `NPC.cs:6887-6897`).
    #[test]
    fn w4_c37_the_critter_rows_unlock_by_sight() {
        let mut tracker = WorldBestiary::new();
        let fish = ENTRIES.iter().find(|e| e.net_id == 55).unwrap();
        tracker.kills.register_kill("Goldfish");
        assert_eq!(unlock_state(fish, &tracker), UnlockState::NotKnownAtAll);
        tracker.set_was_seen_directly("Goldfish");
        assert_eq!(
            unlock_state(fish, &tracker),
            UnlockState::CanShowDropsWithDropRates
        );
    }

    /// W4-c37: the minion folds read the BOSS's credit with quick-unlock
    /// (`BestiaryDatabaseNPCsPopulator.cs:683-702`) — one Creeper kill
    /// credits BrainofCthulhu, and the Creeper's own entry jumps to 4.
    #[test]
    fn w4_c37_the_minion_folds_read_the_boss_credit() {
        let mut tracker = WorldBestiary::new();
        let creeper = ENTRIES.iter().find(|e| e.net_id == 267).unwrap();
        let brain = ENTRIES.iter().find(|e| e.net_id == 266).unwrap();
        tracker.kills.register_kill("BrainofCthulhu");
        assert_eq!(
            unlock_state(creeper, &tracker),
            UnlockState::CanShowDropsWithDropRates
        );
        assert_eq!(
            unlock_state(brain, &tracker),
            UnlockState::CanShowDropsWithDropRates
        );
        // A kill under the minion's OWN credit does nothing (the C#
        // provider reads the boss's persistent id).
        let mut t2 = WorldBestiary::new();
        t2.kills.register_kill("Creeper");
        assert_eq!(unlock_state(creeper, &t2), UnlockState::NotKnownAtAll);
    }

    /// W4-c37: the Skeletron head's Highest-of-two
    /// (`BestiaryDatabaseNPCsPopulator.cs:677`): the Clothier's chat OR
    /// the head's own quick-unlock kill.
    #[test]
    fn w4_c37_skeletron_unlocks_by_chat_or_kill() {
        let head = ENTRIES.iter().find(|e| e.net_id == 35).unwrap();
        let mut by_chat = WorldBestiary::new();
        by_chat.set_was_chat_with_directly("Clothier");
        assert_eq!(
            unlock_state(head, &by_chat),
            UnlockState::CanShowDropsWithDropRates
        );
        let mut by_kill = WorldBestiary::new();
        by_kill.kills.register_kill("SkeletronHead");
        assert_eq!(
            unlock_state(head, &by_kill),
            UnlockState::CanShowDropsWithDropRates
        );
        assert_eq!(
            unlock_state(head, &WorldBestiary::new()),
            UnlockState::NotKnownAtAll
        );
    }

    /// W4-c37: the boss-rule flag census — every `SetDefaults` boss and
    /// every `ShouldBeCountedAsBossForBestiary` id in the roster carries
    /// `quick_unlock: true`; the non-boss enemies carry false
    /// (`BestiaryEntry.cs:37-41`). The DD2 mini-boss overrides
    /// (`BestiaryDatabaseNPCsPopulator.cs:679-682`) ride the gate table,
    /// not the factory flag.
    #[test]
    fn w4_c37_the_boss_flag_matches_the_csharp_boss_rows() {
        for e in ENTRIES {
            let boss = crate::bestiary_data::set_defaults_boss(e.net_id)
                || crate::bestiary_data::counted_as_boss_for_bestiary(e.net_id);
            assert_eq!(e.quick_unlock, boss, "{} boss flag", e.credit_id);
        }
    }

    /// W4-c37: the threshold pins — the ladder's full count is the banner
    /// chain's `KillsToBanner` (default 50, `ItemID.cs:1303-1304`), and
    /// the ½/⅕ steps land on the C#'s integer divisions.
    #[test]
    fn w4_c37_the_threshold_pins() {
        // BlueSlime: banner 69 -> item 1683 -> default 50; 1/10/25/50.
        assert_eq!(tr_core::bestiary::kills_needed_for(1), 50);
        // Tim: rarity-4 wizard, banner 177 -> item 2986 -> 10 kills;
        // the steps are 1 / 2 / 5 / 10 (`GetUnlockStateByKillCount:50-76`).
        assert_eq!(tr_core::bestiary::kills_needed_for(45), 10);
        let tim = ENTRIES.iter().find(|e| e.net_id == 45).unwrap();
        let mut t = WorldBestiary::new();
        for _ in 0..2 {
            t.kills.register_kill("Tim");
        }
        assert_eq!(unlock_state(tim, &t), UnlockState::CanShowStats);
        for _ in 0..3 {
            t.kills.register_kill("Tim");
        }
        assert_eq!(
            unlock_state(tim, &t),
            UnlockState::CanShowDropsWithoutDropRates
        );
        for _ in 0..5 {
            t.kills.register_kill("Tim");
        }
        assert_eq!(
            unlock_state(tim, &t),
            UnlockState::CanShowDropsWithDropRates
        );
        // DD2GoblinT1: banner 258 -> item 3838 -> 1000 kills; the fifth
        // step is 200, the half 500.
        assert_eq!(tr_core::bestiary::kills_needed_for(552), 1000);
        // The DD2 Dark Mage T3 override (`:679`): quick-unlock on the
        // first kill even though the factory flag is false (not a boss).
        let mage = ENTRIES.iter().find(|e| e.net_id == 565).unwrap();
        assert!(!mage.quick_unlock);
        let mut t2 = WorldBestiary::new();
        t2.kills.register_kill("DD2DarkMageT3");
        assert_eq!(
            unlock_state(mage, &t2),
            UnlockState::CanShowDropsWithDropRates
        );
    }

    /// W4-c37: the star formula (`ContentSamples.cs:1245-1306`) — the
    /// rarity bump, the boss +0.5, the `damage + defense + lifeMax/4`
    /// bulk ladder, the 5-cap and the int truncation.
    #[test]
    fn w4_c37_the_star_formula_pins() {
        use crate::bestiary_data::stars_from_stats;
        // A plain enemy: 1 + 0 bulk -> 1 star (BlueSlime 7+2+25/4 = 35).
        assert_eq!(stars_from_stats(0, false, 7, 2, 25), 1);
        // The >50 bulk step: DevourerHead 31+2+100/4 = 58 -> 2.
        assert_eq!(stars_from_stats(0, false, 31, 2, 100), 2);
        // Rarity 2 adds 1.5 on top of the base 2: Pinky 5+5+150/4 = 47
        // (no bulk step) -> 4.5 truncated to 4.
        assert_eq!(stars_from_stats(2, false, 5, 5, 150), 4);
        // The boss +0.5 lands before the cap: 1 + 0.5 + 3.5 (>10000 bulk)
        // = 5 (Duke Fishron 100+50+60000/4 = 15150).
        assert_eq!(stars_from_stats(0, true, 100, 50, 60000), 5);
        // Rarity 4 (Tim): 1+4+2.5 = 7.5 capped to 5 before the bulk.
        assert_eq!(stars_from_stats(4, false, 20, 4, 200), 5);
        // The >50 bulk step: CorruptSlime 55+20+170/4 = 117 -> 2;
        // FireImp 30+16+70/4 = 63 -> 2 as well.
        assert_eq!(stars_from_stats(0, false, 55, 20, 170), 2);
        assert_eq!(stars_from_stats(0, false, 30, 16, 70), 2);
    }

    /// W4-c37: the override table beats the formula — the Eater of Worlds
    /// head's bulk would read 5, the `FillNpcRarities` row pins 3
    /// (`ContentSamples.cs:1239`); the Eternia Crystal's fresh-Main sample
    /// (1000 HP / 14 defense, `NPC.cs:16003-16004`) reads 3 — the runtime
    /// post-Golem 5000/20 row would read 4.
    #[test]
    fn w4_c37_the_override_rows_beat_the_formula() {
        let eow = ENTRIES.iter().find(|e| e.net_id == 13).unwrap();
        assert_eq!(crate::bestiary_data::rarity_stars(eow), 3);
        let crystal = ENTRIES.iter().find(|e| e.net_id == 548).unwrap();
        assert_eq!(crate::bestiary_data::rarity_stars(crystal), 3);
        // The Lava Slime's sample keeps the non-remix 50/15/10 row
        // (`NPC.cs:9518-9523`) -> 1 star, same as the remix row's result.
        let lava = ENTRIES.iter().find(|e| e.net_id == 59).unwrap();
        assert_eq!(crate::bestiary_data::rarity_stars(lava), 1);
        // The 170/180/171 trio rides the 3-star row, not the 2-star one
        // (`ContentSamples.cs:1194-1196`).
        assert_eq!(crate::bestiary_data::stars_override(170), Some(3));
        assert_eq!(crate::bestiary_data::stars_override(180), Some(3));
        assert_eq!(crate::bestiary_data::stars_override(171), Some(3));
        // The seven rows the first data pass dropped are back.
        for id in [47, 57, 168, 464, 465, 470] {
            assert_eq!(crate::bestiary_data::stars_override(id), Some(3));
        }
        assert_eq!(crate::bestiary_data::stars_override(301), Some(2));
    }

    /// W4-c37: the whole-roster star census — every entry's
    /// `NpcBestiaryRarityStars` value, derived from the C# (`FillNpcRarities`
    /// override rows `ContentSamples.cs:1106-1242` first, else the
    /// `GetNPCBestiaryRarityStarsCount` formula over the Classic
    /// `SetDefaults` sample with the fresh-Main progression flags false).
    /// 249 rows: 50x1 / 121x2 / 46x3 / 23x4 / 9x5.
    #[test]
    fn w4_c37_the_star_census() {
        const EXPECTED: &[(i32, i32)] = &[
            (1, 1),   // BlueSlime
            (-3, 1),  // GreenSlime
            (-4, 4),  // Pinky
            (-5, 1),  // BabySlime
            (-6, 1),  // BlackSlime
            (-7, 1),  // PurpleSlime
            (-8, 1),  // RedSlime
            (-9, 3),  // YellowSlime
            (2, 1),   // DemonEye
            (3, 1),   // Zombie
            (16, 2),  // MotherSlime
            (50, 2),  // KingSlime
            (4, 2),   // EyeofCthulhu
            (52, 5),  // DoctorBones
            (53, 5),  // TheGroom
            (59, 1),  // LavaSlime
            (71, 4),  // DungeonSlime
            (81, 2),  // CorruptSlime
            (115, 2), // TheHungry
            (117, 1), // LeechHead
            (5, 1),   // ServantofCthulhu
            (132, 1), // BaldZombie
            (133, 2), // WanderingEye
            (22, 1),  // Guide
            (21, 1),  // Skeleton
            (44, 3),  // UndeadMiner
            (49, 1),  // CaveBat
            (51, 1),  // JungleBat
            (60, 2),  // Hellbat
            (66, 3),  // VoodooDemon
            (6, 1),   // EaterofSouls
            (42, 1),  // Hornet
            (210, 1), // Bee
            (211, 1), // BeeSmall
            (23, 2),  // MeteorHead
            (29, 2),  // GoblinSorcerer
            (45, 5),  // Tim
            (32, 1),  // DarkCaster
            (24, 2),  // FireImp
            (138, 2), // IlluminantSlime
            (147, 1), // IceSlime
            (161, 2), // ZombieEskimo
            (183, 2), // Crimslime
            (184, 2), // SpikedIceSlime
            (186, 1), // PincushionZombie
            (187, 1), // SlimedZombie
            (188, 1), // SwampZombie
            (189, 1), // TwiggyZombie
            (190, 1), // CataractEye
            (191, 1), // SleepyEye
            (192, 1), // DialatedEye
            (193, 1), // GreenEye
            (194, 1), // PurpleEye
            (200, 1), // FemaleZombie
            (223, 2), // ZombieRaincoat
            (319, 3), // ZombieDoctor
            (320, 3), // ZombieSuperman
            (321, 3), // ZombiePixie
            (331, 3), // ZombieXmas
            (332, 3), // ZombieSweater
            (489, 1), // BloodZombie
            (490, 2), // Drippler
            (225, 2), // UmbrellaSlime
            (533, 2), // DesertDjinn
            (30, 1),  // ChaosBall
            (25, 1),  // BurningSphere
            (33, 1),  // WaterSphere
            (55, 1),  // Goldfish
            (58, 1),  // Piranha
            (65, 2),  // Shark
            (7, 2),   // DevourerHead
            (10, 1),  // GiantWormHead
            (87, 3),  // WyvernHead
            (17, 1),  // Merchant
            (20, 3),  // Dryad
            (18, 1),  // Nurse
            (19, 1),  // ArmsDealer
            (38, 1),  // Demolitionist
            (227, 2), // Painter
            (228, 2), // WitchDoctor
            (54, 2),  // Clothier
            (368, 3), // TravellingMerchant
            (369, 2), // Angler
            (108, 3), // Wizard
            (124, 2), // Mechanic
            (353, 2), // Stylist
            (441, 5), // TaxCollector
            (588, 2), // Golfer
            (633, 5), // BestiaryGirl
            (663, 5), // Princess
            (207, 2), // DyeTrader
            (208, 4), // PartyGirl
            (222, 3), // QueenBee
            (35, 3),  // SkeletronHead
            (113, 4), // WallofFlesh
            (134, 4), // TheDestroyer
            (127, 4), // SkeletronPrime
            (125, 4), // Retinazer
            (126, 4), // Spazmatism
            (261, 2), // FungiSpore
            (262, 4), // Plantera
            (263, 3), // PlanterasHook
            (264, 2), // PlanterasTentacle
            (265, 2), // Spore
            (266, 3), // BrainofCthulhu
            (267, 2), // Creeper
            (305, 2), // Scarecrow1
            (306, 2), // Scarecrow2
            (307, 2), // Scarecrow3
            (308, 2), // Scarecrow4
            (309, 2), // Scarecrow5
            (310, 2), // Scarecrow6
            (311, 2), // Scarecrow7
            (312, 2), // Scarecrow8
            (313, 2), // Scarecrow9
            (314, 2), // Scarecrow10
            (315, 3), // HeadlessHorseman
            (325, 3), // MourningWood
            (326, 2), // Splinterling
            (327, 4), // Pumpking
            (329, 3), // Hellhound
            (330, 2), // Poltergeist
            (338, 2), // ZombieElf
            (339, 2), // ZombieElfBeard
            (340, 2), // ZombieElfGirl
            (341, 2), // PresentMimic
            (342, 2), // GingerbreadMan
            (343, 3), // Yeti
            (344, 3), // Everscream
            (345, 4), // IceQueen
            (346, 3), // SantaNK1
            (347, 2), // ElfCopter
            (348, 3), // Nutcracker
            (349, 3), // NutcrackerSpinning
            (350, 2), // ElfArcher
            (351, 3), // Krampus
            (352, 2), // Flocko
            (27, 1),  // GoblinThief
            (28, 2),  // GoblinWarrior
            (111, 1), // GoblinArcher
            (26, 1),  // GoblinPeon
            (471, 4), // GoblinSummoner
            (143, 2), // SnowmanGangsta
            (144, 2), // MisterStabby
            (145, 2), // SnowBalla
            (383, 2), // MartianOfficer
            (385, 2), // GrayGrunt
            (386, 2), // MartianEngineer
            (389, 2), // GigaZapper
            (520, 3), // MartianWalker
            (399, 4), // MartianProbe
            (388, 2), // MartianDrone
            (381, 2), // BrainScrambler
            (382, 2), // RayGunner
            (390, 2), // ScutlixRider
            (391, 2), // Scutlix
            (392, 2), // MartianSaucer
            (393, 3), // MartianSaucerTurret
            (394, 3), // MartianSaucerCannon
            (395, 4), // MartianSaucerCore
            (387, 2), // MartianTurret
            (491, 4), // FlyingDutchman
            (492, 3), // FlyingDutchmanCannon
            (552, 1), // DD2GoblinT1
            (553, 2), // DD2GoblinT2
            (554, 2), // DD2GoblinT3
            (555, 2), // DD2GoblinBomberT1
            (556, 2), // DD2GoblinBomberT2
            (557, 2), // DD2GoblinBomberT3
            (561, 2), // DD2JavelinstT1
            (562, 2), // DD2JavelinstT2
            (563, 2), // DD2JavelinstT3
            (566, 1), // DD2SkeletonT1
            (567, 2), // DD2SkeletonT3
            (564, 2), // DD2DarkMageT1
            (551, 4), // DD2Betsy
            (568, 2), // DD2WitherBeastT2
            (550, 2), // DD2Bartender
            (548, 2), // DD2EterniaCrystal
            (549, 1), // DD2LanePortal
            (558, 1), // DD2WyvernT1
            (559, 2), // DD2WyvernT2
            (560, 2), // DD2WyvernT3
            (565, 3), // DD2DarkMageT3
            (569, 2), // DD2WitherBeastT3
            (570, 2), // DD2DrakinT2
            (571, 3), // DD2DrakinT3
            (572, 2), // DD2KoboldWalkerT2
            (573, 2), // DD2KoboldWalkerT3
            (574, 2), // DD2KoboldFlyerT2
            (575, 2), // DD2KoboldFlyerT3
            (576, 3), // DD2OgreT2
            (577, 3), // DD2OgreT3
            (439, 4), // AncientCultist
            (493, 4), // StardustTower
            (507, 4), // NebulaTower
            (517, 4), // SolarTower
            (422, 4), // VortexTower
            (405, 2), // StardustCellBig
            (406, 2), // StardustCellSmall
            (407, 2), // StardustJellyfishBig
            (423, 2), // NebulaBeast
            (425, 2), // VortexRifleman
            (429, 2), // VortexSoldier
            (426, 2), // VortexHunter
            (427, 2), // VortexAlien
            (420, 2), // NebulaFloater
            (421, 2), // BrainSuckler
            (409, 2), // StardustSpiderBig
            (410, 2), // StardustSpiderSmall
            (411, 2), // StardustSoldier
            (415, 2), // SolarDrakomire
            (416, 2), // SolarDrakomireRider
            (417, 2), // SolarSroller
            (418, 2), // SolarCorite
            (412, 3), // SolarCrawltipedeHead
            (413, 3), // SolarCrawltipedeBody
            (414, 3), // SolarCrawltipedeTail
            (402, 2), // StardustWormHead
            (518, 2), // SolarSpearman
            (419, 2), // SolarSelenian
            (424, 2), // Predictor
            (428, 2), // VortexHeadcrab
            (396, 5), // MoonLordHead
            (397, 4), // MoonLordHand
            (398, 5), // MoonLordCore
            (400, 2), // MoonLordFreeEye
            (401, 2), // MoonLordLeechBlob
            (212, 2), // PirateDeckhand
            (213, 2), // PirateCaptainsMate
            (214, 2), // PirateArcher
            (215, 2), // PirateCorsair
            (216, 3), // PirateCaptain
            (245, 4), // Golem
            (370, 5), // DukeFishron
            (372, 2), // Sharkron
            (128, 3), // PrimeCannon
            (129, 3), // PrimeSaw
            (130, 3), // PrimeVice
            (131, 3), // PrimeLaser
            (114, 3), // WallofFleshEye
            (36, 2),  // SkeletronHand
            (668, 3), // Deerclops
            (107, 3), // GoblinTinkerer
            (13, 3),  // EaterofWorldsHead
            (72, 2),  // BlazingWheel
            (250, 3), // AngryNimbus
            (288, 2), // DungeonSpirit
            (384, 2), // ForceBubble
        ];
        assert_eq!(EXPECTED.len(), ENTRIES.len());
        for (e, &(net, stars)) in ENTRIES.iter().zip(EXPECTED) {
            assert_eq!(e.net_id, net, "census order");
            assert_eq!(
                crate::bestiary_data::rarity_stars(e),
                stars,
                "{} stars",
                e.credit_id
            );
        }
    }

    /// W4-d122 remainder census — `Chest.BestiaryGirl_IsFairyTorchAvailable`
    /// (`Chest.cs:3544-3559`): ALL THREE fairy-critter sightings needed,
    /// and the critter provider reads SIGHTS not kills
    /// (`CritterUICollectionInfoProvider.cs:15-21`). Written, not run.
    #[test]
    fn the_fairy_torch_needs_all_three_critter_sights() {
        let mut b = tr_core::bestiary::WorldBestiary::new();
        assert!(!fairy_torch_available(&b), "no sights, no torch");
        b.set_was_seen_directly("FairyCritterBlue");
        b.set_was_seen_directly("FairyCritterGreen");
        assert!(!fairy_torch_available(&b), "two of three is not enough");
        // Kills do NOT open a critter entry — the provider reads sights.
        b.kills.register_kill("FairyCritterPink");
        assert!(!fairy_torch_available(&b), "kills are not sights");
        b.set_was_seen_directly("FairyCritterPink");
        assert!(fairy_torch_available(&b), "all three sighted");
    }
}
