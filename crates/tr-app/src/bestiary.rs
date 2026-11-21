//! The bestiary entry catalog — the port's NPC family as the
//! `BestiaryDatabase`'s rows (`BestiaryDatabaseNPCsPopulator.cs` builds one
//! entry per NPC net id; the port has the same twelve).

use tr_core::bestiary::{KillTracker, UnlockState, get_unlock_state_by_kill_count};

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
    // W4-d41 — the cavern crawdads/shellies (`BestiaryEntry.Enemy`, the
    // automated populator `BestiaryDatabaseNPCsPopulator.cs:851-868`;
    // the Caverns tag `:1121-1122`/`:1148-1149`). ONE entry per pair:
    // the 2-variants' draw modifiers hide them (`NPCID.cs`'s redigit
    // dictionary, the merge's last word) and their credits fold onto
    // the 1-variant (`ContentSamples.cs:1019-1020`, `entry_for_npc`).
    // The name rides `Lang.GetNPCName` (en-US NPCs.json).
    BestiaryEntryDef {
        credit_id: "Crawdad",
        net_id: 494,
        kind: NpcKind::Crawdad,
        variant: None,
        quick_unlock: false,
        name: "Crawdad",
    },
    BestiaryEntryDef {
        credit_id: "GiantShelly",
        net_id: 496,
        kind: NpcKind::GiantShelly,
        variant: None,
        quick_unlock: false,
        name: "Giant Shelly",
    },
    // W4-d41 — the salamanders: ONE entry for the nine colours. The
    // vanilla's automated populator registers 499 only (the redigit
    // dictionary hides 498/500-506), and `ContentSamples.cs:1021-1025`
    // folds ALL of 498-506 onto the 499 credit — the entry's kind is
    // Salamander2 (net 499), every other colour folds onto it
    // (`entry_for_npc`). The Caverns tag (`:1435-1443`).
    BestiaryEntryDef {
        credit_id: "Salamander2",
        net_id: 499,
        kind: NpcKind::Salamander2,
        variant: None,
        quick_unlock: false,
        name: "Salamander",
    },
    // W4-d41 — the desert antlions (`BestiaryEntry.Enemy`, the automated
    // populator; the UndergroundDesert tag `:1256-1257`/`:1523`).
    BestiaryEntryDef {
        credit_id: "GiantWalkingAntlion",
        net_id: 508,
        kind: NpcKind::GiantWalkingAntlion,
        variant: None,
        quick_unlock: false,
        name: "Giant Antlion Charger",
    },
    BestiaryEntryDef {
        credit_id: "GiantFlyingAntlion",
        net_id: 509,
        kind: NpcKind::GiantFlyingAntlion,
        variant: None,
        quick_unlock: false,
        name: "Giant Antlion Swarmer",
    },
    // W4-d41 — the Dune Splicer: ONE entry on the head's id 510 — the
    // body/tail's draw modifiers hide them (the leinfors dictionary's
    // `Hide = true`) and the kills credit the head (the Devourer's fold,
    // `entry_for_npc`). The UndergroundDesert tag (`:626`/`:1385`).
    BestiaryEntryDef {
        credit_id: "DuneSplicerHead",
        net_id: 510,
        kind: NpcKind::DuneSplicerHead,
        variant: None,
        quick_unlock: false,
        name: "Dune Splicer",
    },
    // W4-d41 — the Tomb Crawler: ONE entry on the head's id 513 — the
    // body/tail's draw modifiers hide them (the leinfors dictionary's
    // `Hide = true`) and the kills credit the head (the Devourer's fold,
    // `entry_for_npc`). The UndergroundDesert tag (`:1150`).
    BestiaryEntryDef {
        credit_id: "TombCrawlerHead",
        net_id: 513,
        kind: NpcKind::TombCrawlerHead,
        variant: None,
        quick_unlock: false,
        name: "Tomb Crawler",
    },
    // W4-d41 — the solar fragments (516/519) have NO bestiary entries:
    // their draw modifiers hide them (the leinfors dictionary's
    // `Hide = true`) and `entry_for_npc` keeps them entry-less (the
    // `has_entry` filter skips them in the frame sweep). The ancient
    // orbs (522/523) are Hide=true too — no entries.
    // W4-d41 — the desert ghouls (`BestiaryEntry.Enemy`, the automated
    // populator; the UndergroundDesert/CorruptUndergroundDesert/
    // CrimsonUndergroundDesert tags `:1420-1422`).
    BestiaryEntryDef {
        credit_id: "DesertGhoul",
        net_id: 524,
        kind: NpcKind::DesertGhoul,
        variant: None,
        quick_unlock: false,
        name: "Ghoul",
    },
    BestiaryEntryDef {
        credit_id: "DesertGhoulCorruption",
        net_id: 525,
        kind: NpcKind::DesertGhoulCorruption,
        variant: None,
        quick_unlock: false,
        name: "Vile Ghoul",
    },
    BestiaryEntryDef {
        credit_id: "DesertGhoulCrimson",
        net_id: 526,
        kind: NpcKind::DesertGhoulCrimson,
        variant: None,
        quick_unlock: false,
        name: "Tainted Ghoul",
    },
    // W4-d41 — the hallowed ghoul + the lamias (`BestiaryEntry.Enemy`,
    // the automated populator; the HallowUndergroundDesert tag
    // `:1423`, the UndergroundDesert/evil tags `:1557-1558`).
    BestiaryEntryDef {
        credit_id: "DesertGhoulHallow",
        net_id: 527,
        kind: NpcKind::DesertGhoulHallow,
        variant: None,
        quick_unlock: false,
        name: "Dreamer Ghoul",
    },
    BestiaryEntryDef {
        credit_id: "DesertLamiaLight",
        net_id: 528,
        kind: NpcKind::DesertLamiaLight,
        variant: None,
        quick_unlock: false,
        name: "Lamia",
    },
    BestiaryEntryDef {
        credit_id: "DesertLamiaDark",
        net_id: 529,
        kind: NpcKind::DesertLamiaDark,
        variant: None,
        quick_unlock: false,
        name: "Lamia",
    },
    // W4-d41 — the sand poacher: ONE entry on the walker's id 530 — the
    // wall form's draw modifiers hide it (the leinfors dictionary's
    // `Hide = true`) and its credit folds onto 530
    // (`ContentSamples.cs:1038`, `entry_for_npc`). The UndergroundDesert
    // tag (`:1337`/`:1647`).
    BestiaryEntryDef {
        credit_id: "DesertScorpionWalk",
        net_id: 530,
        kind: NpcKind::DesertScorpionWalk,
        variant: None,
        quick_unlock: false,
        name: "Sand Poacher",
    },
    // W4-d41 — the basilisk + the tortured soul + the slimes + the bride
    // (`BestiaryEntry.Enemy`, the automated populator; the
    // UndergroundDesert tag `:1257`, the TheUnderworld tag `:1444`, the
    // BloodMoon tag `:1424`, the UndergroundDesert tag `:1106`).
    BestiaryEntryDef {
        credit_id: "DesertBeast",
        net_id: 532,
        kind: NpcKind::DesertBeast,
        variant: None,
        quick_unlock: false,
        name: "Basilisk",
    },
    BestiaryEntryDef {
        credit_id: "DemonTaxCollector",
        net_id: 534,
        kind: NpcKind::DemonTaxCollector,
        variant: None,
        quick_unlock: false,
        name: "Tortured Soul",
    },
    BestiaryEntryDef {
        credit_id: "TheBride",
        net_id: 536,
        kind: NpcKind::TheBride,
        variant: None,
        quick_unlock: false,
        name: "The Bride",
    },
    BestiaryEntryDef {
        credit_id: "SandSlime",
        net_id: 537,
        kind: NpcKind::SandSlime,
        variant: None,
        quick_unlock: false,
        name: "Sand Slime",
    },
    // W4-d41 — the spiked slime (535) has NO bestiary entry: its draw
    // modifiers hide it (the redigit dictionary's `Hide = true`).
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
        // W4-d41: `BestiaryDatabaseNPCsPopulator.cs:681` — the boss's
        // `CommonEnemyUICollectionInfoProvider(..., quickUnlock: true)`.
        quick_unlock: true,
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
];

/// `CommonEnemyUICollectionInfoProvider.GetEntryUICollectionInfo`
/// (`CommonEnemyUICollectionInfoProvider.cs:39-46`): the entry's current
/// unlock tier from the kill tracker.
pub fn unlock_state(entry: &BestiaryEntryDef, kills: &KillTracker) -> UnlockState {
    let full = tr_core::bestiary::kills_needed_for(entry.net_id);
    get_unlock_state_by_kill_count(
        kills.get_kill_count(entry.credit_id),
        entry.quick_unlock,
        full,
    )
}

/// `Main.GetBestiaryProgressReport()`'s completion fraction (the
/// `BestiaryUI` report): the ported entries past the portrait-only tier
/// over the whole roster. The Zoologist's shop gates read it
/// (`Chest.cs:3265`).
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
        // W4-d41 — the cavern 2-variants fold onto the 1-variants' credits
        // (`ContentSamples.cs:1019-1020`: {495→494}, {497→496}); their
        // draw modifiers hide them (the redigit dictionary's `Hide=true`).
        NpcKind::Crawdad2 => NpcKind::Crawdad,
        NpcKind::GiantShelly2 => NpcKind::GiantShelly,
        // W4-d41 — the salamander colours fold onto the 499 entry
        // (`ContentSamples.cs:1021-1025`: ALL of 498-506 → the 499
        // credit; the redigit dictionary hides 498/500-506).
        NpcKind::Salamander
        | NpcKind::Salamander3
        | NpcKind::Salamander4
        | NpcKind::Salamander5
        | NpcKind::Salamander6
        | NpcKind::Salamander7
        | NpcKind::Salamander8
        | NpcKind::Salamander9 => NpcKind::Salamander2,
        // W4-d41 — the Dune Splicer segments fold onto the head's entry
        // (the leinfors dictionary's `Hide = true` — one credit for the
        // whole worm, the Devourer precedent).
        NpcKind::DuneSplicerBody | NpcKind::DuneSplicerTail => NpcKind::DuneSplicerHead,
        // W4-d41 — the Tomb Crawler segments fold onto the head's entry
        // (the leinfors dictionary's `Hide = true`).
        NpcKind::TombCrawlerBody | NpcKind::TombCrawlerTail => NpcKind::TombCrawlerHead,
        // W4-d41 — the sand poacher's wall form folds onto the walker's
        // entry (`ContentSamples.cs:1038`: {531→530}; the leinfors
        // dictionary's `Hide = true`).
        NpcKind::DesertScorpionWall => NpcKind::DesertScorpionWalk,
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
        // W4-d41 — the cavern 2-variants ride the 1-variants' rows
        // (`ContentSamples.cs:1019-1020`'s credit fold; the redigit
        // dictionary's `Hide = true` keeps them out of the roster).
        NpcKind::Crawdad2 => NpcKind::Crawdad,
        NpcKind::GiantShelly2 => NpcKind::GiantShelly,
        // W4-d41 — the salamander colours ride the 499 row
        // (`ContentSamples.cs:1021-1025`'s credit fold).
        NpcKind::Salamander
        | NpcKind::Salamander3
        | NpcKind::Salamander4
        | NpcKind::Salamander5
        | NpcKind::Salamander6
        | NpcKind::Salamander7
        | NpcKind::Salamander8
        | NpcKind::Salamander9 => NpcKind::Salamander2,
        // W4-d41 — the Dune Splicer segments ride the head's row (the
        // leinfors dictionary's `Hide = true`).
        NpcKind::DuneSplicerBody | NpcKind::DuneSplicerTail => NpcKind::DuneSplicerHead,
        // W4-d41 — the Tomb Crawler segments ride the head's row (the
        // leinfors dictionary's `Hide = true`).
        NpcKind::TombCrawlerBody | NpcKind::TombCrawlerTail => NpcKind::TombCrawlerHead,
        // W4-d41 — the sand poacher's wall form rides the walker's row
        // (`ContentSamples.cs:1038`'s credit fold).
        NpcKind::DesertScorpionWall => NpcKind::DesertScorpionWalk,
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
}
