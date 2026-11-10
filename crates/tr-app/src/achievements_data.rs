//! The full achievement registry — `AchievementInitializer.Load`
//! (`Initializers/AchievementInitializer.cs:13-441`) transcribed row by row:
//! every `new Achievement("NAME")` with its `AddCondition` calls, the
//! `RegisterIconIndex` pass (`:445-582`) and the `RegisterAchievementCategory`
//! pass (`:584-736`). Data only; the runtime lives in [`crate::achievements`].
//!
//! The rows keep the initializer's construction order — `Achievement.Id` is
//! assigned in that order (`Achievement.cs:17`) and `UIAchievementListItem.
//! CompareTo` sorts the menu list by it, while the icon index has its own
//! order (e.g. `BENCHED` is constructed second but carries icon 88).
//!
//! One row (`GAIN_TORCH_GODS_FAVOR`) has no category: the initializer never
//! passes it to `RegisterAchievementCategory`, so it stays
//! `AchievementCategory.None` (`AchievementCategory.cs`: `None = -1`).

use crate::achievements::Category;

/// `Achievement.UseTrackerFromCondition(name)` / `UseConditionsCompletedTracker()`
/// (`Achievement.cs:34-48`).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Tracker {
    /// `UseConditionsCompletedTracker` — `(completed, conditions count)`
    /// (`ConditionsCompletedTracker.cs`).
    ConditionsCompleted,
    /// `UseTrackerFromCondition` — the named condition's own
    /// `(value, max)`.
    FromCondition(&'static str),
}

/// One `AddCondition` call, in the initializer's shape
/// (`AchievementInitializer.cs:16-441`).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Cond {
    /// `NPCKilledCondition.Create(id)` — the net id is a C# `short`, so the
    /// gel variants are negative (`GELATIN_WORLD_TOUR`'s -24/-25).
    Npc(i32),
    /// `NPCKilledCondition.Create(id, …)` — one condition, any listed id.
    Npcs(&'static [i32]),
    /// `ItemPickupCondition.Create(id)`.
    Item(u16),
    /// `ItemPickupCondition.Create(id, …)`.
    Items(&'static [u16]),
    /// `TileDestroyedCondition.Create(id)`.
    Tile(u16),
    /// `TileDestroyedCondition.Create(id, …)`.
    Tiles(&'static [u16]),
    /// `ItemCraftCondition.Create(id)`.
    Craft(u16),
    /// `ItemCraftCondition.Create(id, …)` — `BENCHED` expands
    /// `ItemID.Sets.Workbenches` (`ItemID.cs:1375-1384`, 44 workbenches).
    Crafts(&'static [u16]),
    /// `ProgressionEventCondition.Create(id)`.
    Progression(u32),
    /// `CustomFlagCondition.Create(flag)`.
    Flag(&'static str),
    /// `CustomIntCondition.Create(name, max)`.
    Int(&'static str, i32),
    /// `CustomFloatCondition.Create(name, max)`.
    Float(&'static str, i32),
}

/// One registry row: the `new Achievement` block + its registration pass lines.
#[derive(Clone, Copy, Debug)]
pub struct Row {
    pub name: &'static str,
    pub category: Category,
    pub icon: u16,
    pub conditions: &'static [Cond],
    pub tracker: Option<Tracker>,
}

/// The 137 rows, in `AchievementInitializer.Load`'s construction order.
pub const ROWS: &[Row] = &[
	Row {
		name: "TIMBER",
		// `AchievementInitializer.cs:16-18`
		category: Category::Collector,
		icon: 0,
		conditions: &[Cond::Items(&[9, 619, 2504, 620, 2503, 2260, 621, 911, 1729, 5215])],
		tracker: None,
	},
	Row {
		name: "BENCHED",
		// `AchievementInitializer.cs:19-21`
		category: Category::Collector,
		icon: 88,
		conditions: &[Cond::Crafts(&[36, 635, 636, 637, 673, 811, 812, 813, 814, 815, 916, 1145, 1398, 1401, 1404, 1461, 1511, 1795, 1817, 2229, 2251, 2252, 2253, 2534, 2631, 2632, 2633, 2826, 3156, 3157, 3158, 3909, 3910, 3949, 3975, 4163, 4184, 4205, 4226, 4315, 4584, 5166, 5187, 5208])],
		tracker: None,
	},
	Row {
		name: "NO_HOBO",
		// `AchievementInitializer.cs:22-24`
		category: Category::Explorer,
		icon: 1,
		conditions: &[Cond::Progression(8)],
		tracker: None,
	},
	Row {
		name: "OBTAIN_HAMMER",
		// `AchievementInitializer.cs:25-27`
		category: Category::Collector,
		icon: 2,
		conditions: &[Cond::Items(&[2775, 2746, 5283, 3505, 654, 3517, 7, 3493, 2780, 1513, 2516, 660, 3481, 657, 922, 3511, 2785, 3499, 3487, 196, 367, 104, 797, 2320, 787, 1234, 1262, 3465, 204, 217, 1507, 3524, 3522, 3525, 3523, 4317, 1305])],
		tracker: None,
	},
	Row {
		name: "OOO_SHINY",
		// `AchievementInitializer.cs:28-30`
		category: Category::Explorer,
		icon: 4,
		conditions: &[Cond::Tiles(&[7, 6, 9, 8, 166, 167, 168, 169, 22, 204, 58, 107, 108, 111, 221, 222, 223, 211])],
		tracker: None,
	},
	Row {
		name: "HEART_BREAKER",
		// `AchievementInitializer.cs:31-33`
		category: Category::Explorer,
		icon: 3,
		conditions: &[Cond::Tile(12)],
		tracker: None,
	},
	Row {
		name: "HEAVY_METAL",
		// `AchievementInitializer.cs:34-36`
		category: Category::Collector,
		icon: 5,
		conditions: &[Cond::Items(&[35, 716])],
		tracker: None,
	},
	Row {
		name: "I_AM_LOOT",
		// `AchievementInitializer.cs:37-39`
		category: Category::Explorer,
		icon: 6,
		conditions: &[Cond::Flag("Peek")],
		tracker: None,
	},
	Row {
		name: "STAR_POWER",
		// `AchievementInitializer.cs:40-42`
		category: Category::Collector,
		icon: 7,
		conditions: &[Cond::Flag("Use")],
		tracker: None,
	},
	Row {
		name: "HOLD_ON_TIGHT",
		// `AchievementInitializer.cs:43-45`
		category: Category::Collector,
		icon: 8,
		conditions: &[Cond::Flag("Equip")],
		tracker: None,
	},
	Row {
		name: "EYE_ON_YOU",
		// `AchievementInitializer.cs:46-48`
		category: Category::Slayer,
		icon: 9,
		conditions: &[Cond::Npc(4)],
		tracker: None,
	},
	Row {
		name: "SMASHING_POPPET",
		// `AchievementInitializer.cs:49-51`
		category: Category::Explorer,
		icon: 10,
		conditions: &[Cond::Progression(7)],
		tracker: None,
	},
	Row {
		name: "WORM_FODDER",
		// `AchievementInitializer.cs:52-54`
		category: Category::Slayer,
		icon: 11,
		conditions: &[Cond::Npcs(&[13, 14, 15])],
		tracker: None,
	},
	Row {
		name: "MASTERMIND",
		// `AchievementInitializer.cs:55-57`
		category: Category::Slayer,
		icon: 12,
		conditions: &[Cond::Npc(266)],
		tracker: None,
	},
	Row {
		name: "WHERES_MY_HONEY",
		// `AchievementInitializer.cs:58-60`
		category: Category::Explorer,
		icon: 13,
		conditions: &[Cond::Flag("Reach")],
		tracker: None,
	},
	Row {
		name: "STING_OPERATION",
		// `AchievementInitializer.cs:61-63`
		category: Category::Slayer,
		icon: 14,
		conditions: &[Cond::Npc(222)],
		tracker: None,
	},
	Row {
		name: "BONED",
		// `AchievementInitializer.cs:64-66`
		category: Category::Slayer,
		icon: 15,
		conditions: &[Cond::Npc(35)],
		tracker: None,
	},
	Row {
		name: "DUNGEON_HEIST",
		// `AchievementInitializer.cs:67-70`
		category: Category::Explorer,
		icon: 16,
		conditions: &[Cond::Item(327), Cond::Progression(19)],
		tracker: None,
	},
	Row {
		name: "ITS_GETTING_HOT_IN_HERE",
		// `AchievementInitializer.cs:71-73`
		category: Category::Explorer,
		icon: 17,
		conditions: &[Cond::Flag("Reach")],
		tracker: None,
	},
	Row {
		name: "MINER_FOR_FIRE",
		// `AchievementInitializer.cs:74-76`
		category: Category::Collector,
		icon: 18,
		conditions: &[Cond::Craft(122)],
		tracker: None,
	},
	Row {
		name: "STILL_HUNGRY",
		// `AchievementInitializer.cs:77-79`
		category: Category::Slayer,
		icon: 19,
		conditions: &[Cond::Npcs(&[113, 114])],
		tracker: None,
	},
	Row {
		name: "ITS_HARD",
		// `AchievementInitializer.cs:80-82`
		category: Category::Explorer,
		icon: 20,
		conditions: &[Cond::Progression(9)],
		tracker: None,
	},
	Row {
		name: "BEGONE_EVIL",
		// `AchievementInitializer.cs:83-85`
		category: Category::Explorer,
		icon: 21,
		conditions: &[Cond::Progression(6)],
		tracker: None,
	},
	Row {
		name: "EXTRA_SHINY",
		// `AchievementInitializer.cs:86-88`
		category: Category::Explorer,
		icon: 22,
		conditions: &[Cond::Tiles(&[107, 108, 111, 221, 222, 223])],
		tracker: None,
	},
	Row {
		name: "HEAD_IN_THE_CLOUDS",
		// `AchievementInitializer.cs:89-91`
		category: Category::Collector,
		icon: 23,
		conditions: &[Cond::Flag("Equip")],
		tracker: None,
	},
	Row {
		name: "LIKE_A_BOSS",
		// `AchievementInitializer.cs:92-94`
		category: Category::Collector,
		icon: 24,
		conditions: &[Cond::Items(&[1133, 1331, 1307, 267, 1293, 5334, 557, 544, 556, 560, 43, 70, 3601, 5120, 4961, 4988, 2673])],
		tracker: None,
	},
	Row {
		name: "BUCKETS_OF_BOLTS",
		// `AchievementInitializer.cs:95-97`
		category: Category::Slayer,
		icon: 25,
		conditions: &[Cond::Npcs(&[125, 126])],
		tracker: Some(Tracker::ConditionsCompleted),
	},
	Row {
		name: "DRAX_ATTAX",
		// `AchievementInitializer.cs:100-102`
		category: Category::Collector,
		icon: 26,
		conditions: &[Cond::Crafts(&[579, 990])],
		tracker: None,
	},
	Row {
		name: "PHOTOSYNTHESIS",
		// `AchievementInitializer.cs:103-105`
		category: Category::Explorer,
		icon: 27,
		conditions: &[Cond::Tile(211)],
		tracker: None,
	},
	Row {
		name: "GET_A_LIFE",
		// `AchievementInitializer.cs:106-108`
		category: Category::Explorer,
		icon: 28,
		conditions: &[Cond::Flag("Use")],
		tracker: None,
	},
	Row {
		name: "THE_GREAT_SOUTHERN_PLANTKILL",
		// `AchievementInitializer.cs:109-111`
		category: Category::Slayer,
		icon: 29,
		conditions: &[Cond::Npc(262)],
		tracker: None,
	},
	Row {
		name: "TEMPLE_RAIDER",
		// `AchievementInitializer.cs:112-114`
		category: Category::Collector,
		icon: 30,
		conditions: &[Cond::Progression(22)],
		tracker: None,
	},
	Row {
		name: "LIHZAHRDIAN_IDOL",
		// `AchievementInitializer.cs:115-117`
		category: Category::Slayer,
		icon: 31,
		conditions: &[Cond::Npc(245)],
		tracker: None,
	},
	Row {
		name: "ROBBING_THE_GRAVE",
		// `AchievementInitializer.cs:118-120`
		category: Category::Explorer,
		icon: 32,
		conditions: &[Cond::Items(&[1513, 938, 963, 977, 1300, 1254, 1514, 679, 759, 1446, 1445, 1444, 1183, 1266, 671, 3291, 4679, 4789])],
		tracker: None,
	},
	Row {
		name: "BIG_BOOTY",
		// `AchievementInitializer.cs:121-123`
		category: Category::Explorer,
		icon: 33,
		conditions: &[Cond::Progression(20)],
		tracker: None,
	},
	Row {
		name: "FISH_OUT_OF_WATER",
		// `AchievementInitializer.cs:124-126`
		category: Category::Slayer,
		icon: 34,
		conditions: &[Cond::Npc(370)],
		tracker: None,
	},
	Row {
		name: "OBSESSIVE_DEVOTION",
		// `AchievementInitializer.cs:127-129`
		category: Category::Slayer,
		icon: 35,
		conditions: &[Cond::Npc(439)],
		tracker: None,
	},
	Row {
		name: "STAR_DESTROYER",
		// `AchievementInitializer.cs:130-131`
		category: Category::Slayer,
		icon: 36,
		conditions: &[],
		tracker: None,
	},
	Row {
		name: "CHAMPION_OF_TERRARIA",
		// `AchievementInitializer.cs:133-135`
		category: Category::Slayer,
		icon: 37,
		conditions: &[Cond::Npc(398)],
		tracker: None,
	},
	Row {
		name: "BLOODBATH",
		// `AchievementInitializer.cs:136-138`
		category: Category::Explorer,
		icon: 38,
		conditions: &[Cond::Progression(5)],
		tracker: None,
	},
	Row {
		name: "SLIPPERY_SHINOBI",
		// `AchievementInitializer.cs:139-141`
		category: Category::Slayer,
		icon: 46,
		conditions: &[Cond::Npc(50)],
		tracker: None,
	},
	Row {
		name: "GOBLIN_PUNTER",
		// `AchievementInitializer.cs:142-144`
		category: Category::Slayer,
		icon: 39,
		conditions: &[Cond::Progression(10)],
		tracker: None,
	},
	Row {
		name: "WALK_THE_PLANK",
		// `AchievementInitializer.cs:145-147`
		category: Category::Slayer,
		icon: 41,
		conditions: &[Cond::Progression(11)],
		tracker: None,
	},
	Row {
		name: "KILL_THE_SUN",
		// `AchievementInitializer.cs:148-150`
		category: Category::Explorer,
		icon: 40,
		conditions: &[Cond::Progression(3)],
		tracker: None,
	},
	Row {
		name: "DO_YOU_WANT_TO_SLAY_A_SNOWMAN",
		// `AchievementInitializer.cs:151-153`
		category: Category::Slayer,
		icon: 42,
		conditions: &[Cond::Progression(12)],
		tracker: None,
	},
	Row {
		name: "TIN_FOIL_HATTER",
		// `AchievementInitializer.cs:154-156`
		category: Category::Slayer,
		icon: 43,
		conditions: &[Cond::Progression(13)],
		tracker: None,
	},
	Row {
		name: "BALEFUL_HARVEST",
		// `AchievementInitializer.cs:157-159`
		category: Category::Slayer,
		icon: 44,
		conditions: &[Cond::Progression(15)],
		tracker: None,
	},
	Row {
		name: "ICE_SCREAM",
		// `AchievementInitializer.cs:160-162`
		category: Category::Slayer,
		icon: 45,
		conditions: &[Cond::Progression(14)],
		tracker: None,
	},
	Row {
		name: "STICKY_SITUATION",
		// `AchievementInitializer.cs:163-165`
		category: Category::Explorer,
		icon: 47,
		conditions: &[Cond::Progression(16)],
		tracker: None,
	},
	Row {
		name: "REAL_ESTATE_AGENT",
		// `AchievementInitializer.cs:166-168`
		category: Category::Challenger,
		icon: 48,
		conditions: &[Cond::Progression(17)],
		tracker: None,
	},
	Row {
		name: "NOT_THE_BEES",
		// `AchievementInitializer.cs:169-171`
		category: Category::Challenger,
		icon: 49,
		conditions: &[Cond::Flag("Use")],
		tracker: None,
	},
	Row {
		name: "JEEPERS_CREEPERS",
		// `AchievementInitializer.cs:172-174`
		category: Category::Explorer,
		icon: 50,
		conditions: &[Cond::Flag("Reach")],
		tracker: None,
	},
	Row {
		name: "FUNKYTOWN",
		// `AchievementInitializer.cs:175-177`
		category: Category::Explorer,
		icon: 51,
		conditions: &[Cond::Flag("Reach")],
		tracker: None,
	},
	Row {
		name: "INTO_ORBIT",
		// `AchievementInitializer.cs:178-180`
		category: Category::Explorer,
		icon: 52,
		conditions: &[Cond::Flag("Reach")],
		tracker: None,
	},
	Row {
		name: "ROCK_BOTTOM",
		// `AchievementInitializer.cs:181-183`
		category: Category::Explorer,
		icon: 53,
		conditions: &[Cond::Flag("Reach")],
		tracker: None,
	},
	Row {
		name: "MECHA_MAYHEM",
		// `AchievementInitializer.cs:184-186`
		category: Category::Challenger,
		icon: 54,
		conditions: &[Cond::Progression(21)],
		tracker: None,
	},
	Row {
		name: "GELATIN_WORLD_TOUR",
		// `AchievementInitializer.cs:187-189`
		category: Category::Challenger,
		icon: 55,
		conditions: &[Cond::Npcs(&[183, -24, -25])],
		tracker: Some(Tracker::ConditionsCompleted),
	},
	Row {
		name: "FASHION_STATEMENT",
		// `AchievementInitializer.cs:193-195`
		category: Category::Collector,
		icon: 56,
		conditions: &[Cond::Flag("Equip")],
		tracker: None,
	},
	Row {
		name: "VEHICULAR_MANSLAUGHTER",
		// `AchievementInitializer.cs:196-198`
		category: Category::Slayer,
		icon: 57,
		conditions: &[Cond::Flag("Hit")],
		tracker: None,
	},
	Row {
		name: "BULLDOZER",
		// `AchievementInitializer.cs:199-201`
		category: Category::Challenger,
		icon: 58,
		conditions: &[Cond::Int("Pick", 10000)],
		tracker: Some(Tracker::FromCondition("Pick")),
	},
	Row {
		name: "THERE_ARE_SOME_WHO_CALL_HIM",
		// `AchievementInitializer.cs:203-205`
		category: Category::Slayer,
		icon: 59,
		conditions: &[Cond::Npc(45)],
		tracker: None,
	},
	Row {
		name: "DECEIVER_OF_FOOLS",
		// `AchievementInitializer.cs:206-208`
		category: Category::Slayer,
		icon: 60,
		conditions: &[Cond::Npc(196)],
		tracker: None,
	},
	Row {
		name: "SWORD_OF_THE_HERO",
		// `AchievementInitializer.cs:209-211`
		category: Category::Collector,
		icon: 61,
		conditions: &[Cond::Item(757)],
		tracker: None,
	},
	Row {
		name: "LUCKY_BREAK",
		// `AchievementInitializer.cs:212-214`
		category: Category::Challenger,
		icon: 62,
		conditions: &[Cond::Flag("Hit")],
		tracker: None,
	},
	Row {
		name: "THROWING_LINES",
		// `AchievementInitializer.cs:215-217`
		category: Category::Challenger,
		icon: 63,
		conditions: &[Cond::Flag("Use")],
		tracker: None,
	},
	Row {
		name: "DYE_HARD",
		// `AchievementInitializer.cs:218-220`
		category: Category::Collector,
		icon: 64,
		conditions: &[Cond::Flag("Equip")],
		tracker: None,
	},
	Row {
		name: "SICK_THROW",
		// `AchievementInitializer.cs:221-223`
		category: Category::Collector,
		icon: 86,
		conditions: &[Cond::Item(3389)],
		tracker: None,
	},
	Row {
		name: "FREQUENT_FLYER",
		// `AchievementInitializer.cs:224-226`
		category: Category::Challenger,
		icon: 65,
		conditions: &[Cond::Float("Pay", 10000)],
		tracker: Some(Tracker::FromCondition("Pay")),
	},
	Row {
		name: "THE_CAVALRY",
		// `AchievementInitializer.cs:228-230`
		category: Category::Collector,
		icon: 66,
		conditions: &[Cond::Flag("Equip")],
		tracker: None,
	},
	Row {
		name: "COMPLETELY_AWESOME",
		// `AchievementInitializer.cs:231-233`
		category: Category::Collector,
		icon: 67,
		conditions: &[Cond::Item(98)],
		tracker: None,
	},
	Row {
		name: "TIL_DEATH",
		// `AchievementInitializer.cs:234-236`
		category: Category::Slayer,
		icon: 68,
		conditions: &[Cond::Npc(53)],
		tracker: None,
	},
	Row {
		name: "ARCHAEOLOGIST",
		// `AchievementInitializer.cs:237-239`
		category: Category::Slayer,
		icon: 69,
		conditions: &[Cond::Npc(52)],
		tracker: None,
	},
	Row {
		name: "PRETTY_IN_PINK",
		// `AchievementInitializer.cs:240-242`
		category: Category::Slayer,
		icon: 70,
		conditions: &[Cond::Npc(-4)],
		tracker: None,
	},
	Row {
		name: "RAINBOWS_AND_UNICORNS",
		// `AchievementInitializer.cs:243-245`
		category: Category::Challenger,
		icon: 71,
		conditions: &[Cond::Flag("Use")],
		tracker: None,
	},
	Row {
		name: "YOU_AND_WHAT_ARMY",
		// `AchievementInitializer.cs:246-248`
		category: Category::Challenger,
		icon: 72,
		conditions: &[Cond::Flag("Spawn")],
		tracker: None,
	},
	Row {
		name: "PRISMANCER",
		// `AchievementInitializer.cs:249-251`
		category: Category::Collector,
		icon: 73,
		conditions: &[Cond::Item(495)],
		tracker: None,
	},
	Row {
		name: "IT_CAN_TALK",
		// `AchievementInitializer.cs:252-254`
		category: Category::Explorer,
		icon: 74,
		conditions: &[Cond::Progression(18)],
		tracker: None,
	},
	Row {
		name: "WATCH_YOUR_STEP",
		// `AchievementInitializer.cs:255-257`
		category: Category::Explorer,
		icon: 75,
		conditions: &[Cond::Flag("Hit")],
		tracker: None,
	},
	Row {
		name: "MARATHON_MEDALIST",
		// `AchievementInitializer.cs:258-260`
		category: Category::Challenger,
		icon: 76,
		conditions: &[Cond::Float("Move", 1106688)],
		tracker: Some(Tracker::FromCondition("Move")),
	},
	Row {
		name: "GLORIOUS_GOLDEN_POLE",
		// `AchievementInitializer.cs:262-264`
		category: Category::Collector,
		icon: 77,
		conditions: &[Cond::Item(2294)],
		tracker: None,
	},
	Row {
		name: "SERVANT_IN_TRAINING",
		// `AchievementInitializer.cs:265-267`
		category: Category::Challenger,
		icon: 78,
		conditions: &[Cond::Flag("Finish")],
		tracker: None,
	},
	Row {
		name: "GOOD_LITTLE_SLAVE",
		// `AchievementInitializer.cs:268-270`
		category: Category::Challenger,
		icon: 79,
		conditions: &[Cond::Int("Finish", 10)],
		tracker: Some(Tracker::FromCondition("Finish")),
	},
	Row {
		name: "TROUT_MONKEY",
		// `AchievementInitializer.cs:272-274`
		category: Category::Challenger,
		icon: 80,
		conditions: &[Cond::Int("Finish", 25)],
		tracker: Some(Tracker::FromCondition("Finish")),
	},
	Row {
		name: "FAST_AND_FISHIOUS",
		// `AchievementInitializer.cs:276-278`
		category: Category::Challenger,
		icon: 81,
		conditions: &[Cond::Int("Finish", 50)],
		tracker: Some(Tracker::FromCondition("Finish")),
	},
	Row {
		name: "SUPREME_HELPER_MINION",
		// `AchievementInitializer.cs:280-282`
		category: Category::Challenger,
		icon: 82,
		conditions: &[Cond::Int("Finish", 200)],
		tracker: Some(Tracker::FromCondition("Finish")),
	},
	Row {
		name: "TOPPED_OFF",
		// `AchievementInitializer.cs:284-286`
		category: Category::Challenger,
		icon: 83,
		conditions: &[Cond::Flag("Use")],
		tracker: None,
	},
	Row {
		name: "SLAYER_OF_WORLDS",
		// `AchievementInitializer.cs:287-291`
		category: Category::Challenger,
		icon: 84,
		conditions: &[Cond::Npcs(&[13, 14, 15]), Cond::Npcs(&[113, 114]), Cond::Npcs(&[125, 126])],
		tracker: Some(Tracker::ConditionsCompleted),
	},
	Row {
		name: "YOU_CAN_DO_IT",
		// `AchievementInitializer.cs:294-296`
		category: Category::Explorer,
		icon: 85,
		conditions: &[Cond::Progression(1)],
		tracker: None,
	},
	Row {
		name: "MATCHING_ATTIRE",
		// `AchievementInitializer.cs:297-299`
		category: Category::Collector,
		icon: 87,
		conditions: &[Cond::Flag("Equip")],
		tracker: None,
	},
	Row {
		name: "DEFEAT_EMPRESS_OF_LIGHT",
		// `AchievementInitializer.cs:300-302`
		category: Category::Slayer,
		icon: 90,
		conditions: &[Cond::Npc(636)],
		tracker: None,
	},
	Row {
		name: "DEFEAT_QUEEN_SLIME",
		// `AchievementInitializer.cs:303-305`
		category: Category::Slayer,
		icon: 89,
		conditions: &[Cond::Npc(657)],
		tracker: None,
	},
	Row {
		name: "DEFEAT_DREADNAUTILUS",
		// `AchievementInitializer.cs:306-308`
		category: Category::Slayer,
		icon: 93,
		conditions: &[Cond::Npc(618)],
		tracker: None,
	},
	Row {
		name: "DEFEAT_OLD_ONES_ARMY_TIER3",
		// `AchievementInitializer.cs:309-311`
		category: Category::Slayer,
		icon: 94,
		conditions: &[Cond::Progression(23)],
		tracker: None,
	},
	Row {
		name: "GET_ZENITH",
		// `AchievementInitializer.cs:312-314`
		category: Category::Collector,
		icon: 91,
		conditions: &[Cond::Item(4956)],
		tracker: None,
	},
	Row {
		name: "GET_TERRASPARK_BOOTS",
		// `AchievementInitializer.cs:315-317`
		category: Category::Collector,
		icon: 98,
		conditions: &[Cond::Item(5000)],
		tracker: None,
	},
	Row {
		name: "FLY_A_KITE_ON_A_WINDY_DAY",
		// `AchievementInitializer.cs:318-320`
		category: Category::Challenger,
		icon: 95,
		conditions: &[Cond::Flag("Use")],
		tracker: None,
	},
	Row {
		name: "FOUND_GRAVEYARD",
		// `AchievementInitializer.cs:321-323`
		category: Category::Explorer,
		icon: 102,
		conditions: &[Cond::Flag("Reach")],
		tracker: None,
	},
	Row {
		name: "GO_LAVA_FISHING",
		// `AchievementInitializer.cs:324-326`
		category: Category::Challenger,
		icon: 101,
		conditions: &[Cond::Flag("Do")],
		tracker: None,
	},
	Row {
		name: "TURN_GNOME_TO_STATUE",
		// `AchievementInitializer.cs:327-329`
		category: Category::Challenger,
		icon: 96,
		conditions: &[Cond::Progression(24)],
		tracker: None,
	},
	Row {
		name: "TALK_TO_NPC_AT_MAX_HAPPINESS",
		// `AchievementInitializer.cs:330-332`
		category: Category::Challenger,
		icon: 97,
		conditions: &[Cond::Flag("Do")],
		tracker: None,
	},
	Row {
		name: "PET_THE_PET",
		// `AchievementInitializer.cs:333-335`
		category: Category::Challenger,
		icon: 100,
		conditions: &[Cond::Flag("Do")],
		tracker: None,
	},
	Row {
		name: "FIND_A_FAIRY",
		// `AchievementInitializer.cs:336-338`
		category: Category::Explorer,
		icon: 92,
		conditions: &[Cond::Flag("Do")],
		tracker: None,
	},
	Row {
		name: "THROW_A_PARTY",
		// `AchievementInitializer.cs:339-341`
		category: Category::Challenger,
		icon: 99,
		conditions: &[Cond::Progression(25)],
		tracker: None,
	},
	Row {
		name: "DIE_TO_DEAD_MANS_CHEST",
		// `AchievementInitializer.cs:342-344`
		category: Category::Challenger,
		icon: 103,
		conditions: &[Cond::Flag("Do")],
		tracker: None,
	},
	Row {
		name: "DEFEAT_DEERCLOPS",
		// `AchievementInitializer.cs:345-347`
		category: Category::Slayer,
		icon: 104,
		conditions: &[Cond::Npc(668)],
		tracker: None,
	},
	Row {
		name: "GET_GOLDEN_DELIGHT",
		// `AchievementInitializer.cs:348-350`
		category: Category::Collector,
		icon: 105,
		conditions: &[Cond::Item(4022)],
		tracker: None,
	},
	Row {
		name: "DRINK_BOTTLED_WATER_WHILE_DROWNING",
		// `AchievementInitializer.cs:351-353`
		category: Category::Challenger,
		icon: 106,
		conditions: &[Cond::Flag("Use")],
		tracker: None,
	},
	Row {
		name: "GET_CELL_PHONE",
		// `AchievementInitializer.cs:354-356`
		category: Category::Collector,
		icon: 107,
		conditions: &[Cond::Item(3124)],
		tracker: None,
	},
	Row {
		name: "GET_ANKH_SHIELD",
		// `AchievementInitializer.cs:357-359`
		category: Category::Collector,
		icon: 108,
		conditions: &[Cond::Item(1613)],
		tracker: None,
	},
	Row {
		name: "GAIN_TORCH_GODS_FAVOR",
		// `AchievementInitializer.cs:360-362`
		category: Category::None,
		icon: 109,
		conditions: &[Cond::Flag("Use")],
		tracker: None,
	},
	Row {
		name: "PLAY_ON_A_SPECIAL_SEED",
		// `AchievementInitializer.cs:363-365`
		category: Category::Explorer,
		icon: 110,
		conditions: &[Cond::Flag("Do")],
		tracker: None,
	},
	Row {
		name: "ALL_TOWN_SLIMES",
		// `AchievementInitializer.cs:366-368`
		category: Category::Challenger,
		icon: 111,
		conditions: &[Cond::Progression(26)],
		tracker: None,
	},
	Row {
		name: "TRANSMUTE_ITEM",
		// `AchievementInitializer.cs:369-371`
		category: Category::Explorer,
		icon: 112,
		conditions: &[Cond::Progression(27)],
		tracker: None,
	},
	Row {
		name: "PURIFY_ENTIRE_WORLD",
		// `AchievementInitializer.cs:372-374`
		category: Category::Challenger,
		icon: 113,
		conditions: &[Cond::Flag("Do")],
		tracker: None,
	},
	Row {
		name: "TO_INFINITY_AND_BEYOND",
		// `AchievementInitializer.cs:375-377`
		category: Category::Challenger,
		icon: 114,
		conditions: &[Cond::Flag("Do")],
		tracker: None,
	},
	Row {
		name: "BOOK_WORM",
		// `AchievementInitializer.cs:378-380`
		category: Category::Challenger,
		icon: 115,
		conditions: &[Cond::Progression(29)],
		tracker: None,
	},
	Row {
		name: "BOULDER_LORD",
		// `AchievementInitializer.cs:381-383`
		category: Category::Challenger,
		icon: 116,
		conditions: &[Cond::Progression(30)],
		tracker: None,
	},
	Row {
		name: "QUEEN_MACHINE",
		// `AchievementInitializer.cs:384-386`
		category: Category::Challenger,
		icon: 117,
		conditions: &[Cond::Progression(31)],
		tracker: None,
	},
	Row {
		name: "ROLLIN_IN_YOUR_GRAVE",
		// `AchievementInitializer.cs:387-389`
		category: Category::Challenger,
		icon: 118,
		conditions: &[Cond::Progression(32)],
		tracker: None,
	},
	Row {
		name: "FEAR_THE_SUN",
		// `AchievementInitializer.cs:390-392`
		category: Category::Challenger,
		icon: 119,
		conditions: &[Cond::Progression(33)],
		tracker: None,
	},
	Row {
		name: "ITS_SHALING_OUTSIDE",
		// `AchievementInitializer.cs:393-395`
		category: Category::Explorer,
		icon: 120,
		conditions: &[Cond::Progression(34)],
		tracker: None,
	},
	Row {
		name: "EXTRA_LIFE",
		// `AchievementInitializer.cs:396-398`
		category: Category::Challenger,
		icon: 121,
		conditions: &[Cond::Progression(35)],
		tracker: None,
	},
	Row {
		name: "GRAVE_MISTAKE",
		// `AchievementInitializer.cs:399-401`
		category: Category::Challenger,
		icon: 122,
		conditions: &[Cond::Progression(36)],
		tracker: None,
	},
	Row {
		name: "SPICY_LICKS",
		// `AchievementInitializer.cs:402-404`
		category: Category::Challenger,
		icon: 123,
		conditions: &[Cond::Progression(37)],
		tracker: None,
	},
	Row {
		name: "ORGANIZED_CHAOS",
		// `AchievementInitializer.cs:405-407`
		category: Category::Collector,
		icon: 124,
		conditions: &[Cond::Item(1326)],
		tracker: None,
	},
	Row {
		name: "ON_FLEEK",
		// `AchievementInitializer.cs:408-410`
		category: Category::Collector,
		icon: 125,
		conditions: &[Cond::Progression(38)],
		tracker: None,
	},
	Row {
		name: "FORTUNE_FAVORS_THE_BOULD",
		// `AchievementInitializer.cs:411-413`
		category: Category::Explorer,
		icon: 126,
		conditions: &[Cond::Progression(39)],
		tracker: None,
	},
	Row {
		name: "TRAINING_DAY",
		// `AchievementInitializer.cs:414-416`
		category: Category::Explorer,
		icon: 127,
		conditions: &[Cond::Progression(40)],
		tracker: None,
	},
	Row {
		name: "MINI_ME",
		// `AchievementInitializer.cs:417-419`
		category: Category::Collector,
		icon: 128,
		conditions: &[Cond::Progression(41)],
		tracker: None,
	},
	Row {
		name: "TERRARIST",
		// `AchievementInitializer.cs:420-422`
		category: Category::Challenger,
		icon: 129,
		conditions: &[Cond::Progression(42)],
		tracker: None,
	},
	Row {
		name: "NEW_DIGS",
		// `AchievementInitializer.cs:423-425`
		category: Category::Collector,
		icon: 130,
		conditions: &[Cond::Progression(43)],
		tracker: None,
	},
	Row {
		name: "MY_PEOPLE_NEED_ME",
		// `AchievementInitializer.cs:426-428`
		category: Category::Challenger,
		icon: 131,
		conditions: &[Cond::Progression(44)],
		tracker: None,
	},
	Row {
		name: "GOING_OLDSCHOOL",
		// `AchievementInitializer.cs:429-431`
		category: Category::Challenger,
		icon: 132,
		conditions: &[Cond::Flag("Do")],
		tracker: None,
	},
	Row {
		name: "SEA_YOU_LATER",
		// `AchievementInitializer.cs:432-434`
		category: Category::Collector,
		icon: 133,
		conditions: &[Cond::Item(5358)],
		tracker: None,
	},
	Row {
		name: "TRASH_COMPACTOR",
		// `AchievementInitializer.cs:435-437`
		category: Category::Challenger,
		icon: 134,
		conditions: &[Cond::Progression(45)],
		tracker: None,
	},
	Row {
		name: "CONSERVATIONIST",
		// `AchievementInitializer.cs:438-440`
		category: Category::Challenger,
		icon: 135,
		conditions: &[Cond::Int("Do", 100)],
		tracker: None,
	},
	Row {
		name: "INTERDIMENSIONAL_RECYCLING",
		// `AchievementInitializer.cs:441-443`
		category: Category::Challenger,
		icon: 136,
		conditions: &[Cond::Progression(46)],
		tracker: None,
	},
];
