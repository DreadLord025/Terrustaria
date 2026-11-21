//! The NPC data half of `npc.rs` (S1-G-1 data split): the static
//! `SetDefaults`/sheet lookup tables, verbatim from `npc.rs` — behaviour
//! frozen. The `NpcKind` enum itself and the AI stay in `npc.rs`; the
//! `Npc`-shell accessors (`defense`/`value`/`knock_back_resist`) stay on
//! `impl Npc`. `npc.rs` re-exports every item below, so the
//! `crate::npc::*` seams are unchanged.

use super::{Npc, NpcKind};

/// The slime colour variants. Blue is `NPCID.BlueSlime` (1); the rest are the
/// negative net ids sharing its `aiStyle 1` machine
/// (`Lang.cs:520-533`, `NPC.SetDefaultsFromNetId`, `NPC.cs:7653-8000`).
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum SlimeVariant {
    #[default]
    Blue,
    Green,
    Pinky,
    Baby,
    Black,
    Purple,
    Red,
    Yellow,
}

/// One slime variant's `SetDefaults` line (`NPC.cs:7653-8000` for the net ids,
/// the `if (this.type == 1)` chain at `NPC.cs:8567` for blue): `life`, melee
/// `damage`, `defense`, the `SetDefaults_ForNetId` size scale, the sprite
/// tint `NPC.color` and the gel drop `min..=max` (0 = none;
/// `ItemDropDatabase.cs:1074-1082`).
#[derive(Clone, Copy)]
pub struct SlimeStats {
    pub life: i32,
    pub damage: i32,
    pub defense: i32,
    pub scale: f32,
    pub color: [u8; 4],
    pub gel: (u32, u32),
    /// `NPC.value` in copper (`NPCLoot_DropMoney`'s base).
    pub value: f32,
}

impl SlimeVariant {
    /// The per-variant `SetDefaults` values. Base 24x18 hitbox, `aiStyle 1`
    /// (blue: `NPC.cs:8567-8580`).
    pub fn stats(self) -> SlimeStats {
        match self {
            SlimeVariant::Blue => SlimeStats {
                life: 25,
                damage: 7,
                defense: 2,
                scale: 1.0,
                color: [0, 80, 255, 100],
                gel: (1, 2),
                value: 25.0,
            },
            SlimeVariant::Green => SlimeStats {
                life: 14,
                damage: 6,
                defense: 0,
                scale: 0.9,
                color: [0, 220, 40, 100],
                gel: (0, 0),
                value: 3.0,
            },
            SlimeVariant::Pinky => SlimeStats {
                life: 150,
                damage: 5,
                defense: 5,
                scale: 0.6,
                color: [250, 30, 90, 90],
                gel: (0, 0),
                value: 10000.0,
            },
            SlimeVariant::Baby => SlimeStats {
                life: 30,
                damage: 13,
                defense: 4,
                scale: 0.9,
                color: [0, 0, 0, 50],
                gel: (0, 0),
                value: 10.0,
            },
            SlimeVariant::Black => SlimeStats {
                life: 45,
                damage: 15,
                defense: 4,
                scale: 1.05,
                color: [0, 0, 0, 50],
                gel: (2, 5),
                value: 20.0,
            },
            SlimeVariant::Purple => SlimeStats {
                life: 40,
                damage: 12,
                defense: 6,
                scale: 1.2,
                color: [200, 0, 255, 150],
                gel: (2, 5),
                value: 10.0,
            },
            SlimeVariant::Red => SlimeStats {
                life: 35,
                damage: 12,
                defense: 4,
                scale: 1.025,
                color: [255, 30, 0, 100],
                gel: (2, 5),
                value: 8.0,
            },
            SlimeVariant::Yellow => SlimeStats {
                life: 45,
                damage: 15,
                defense: 7,
                scale: 1.2,
                color: [255, 255, 0, 100],
                gel: (2, 5),
                value: 10.0,
            },
        }
    }

    /// The `Lang._negativeNpcNameCache` names (localization keys are the C#
    /// strings; the port keeps the English base names).
    pub fn name(self) -> &'static str {
        match self {
            SlimeVariant::Blue => "Blue Slime",
            SlimeVariant::Green => "Green Slime",
            SlimeVariant::Pinky => "Pinky",
            SlimeVariant::Baby => "Baby Slime",
            SlimeVariant::Black => "Black Slime",
            SlimeVariant::Purple => "Purple Slime",
            SlimeVariant::Red => "Red Slime",
            SlimeVariant::Yellow => "Yellow Slime",
        }
    }
}

/// The Pumpkin Moon wave family's `SetDefaults` tail (`S1-F-13b`) — the
/// rows the port's tables carry piecewise for every other kind:
/// `NPC.aiStyle`, `NPC.scale`, `NPC.alpha` and the
/// `noGravity`/`noTileCollide` flags, plus the VANILLA `NPC.type`
/// (`NPC.cs:12929-13203`; the `SetDefaults` head
/// `NPC.cs:8510-8537` — aiStyle 0, scale 1f, alpha 0, both flags false —
/// is the default for every field and kind with no row). `None` for every
/// kind outside the family: the other kinds' scale/alpha ride `Npc::new`'s
/// 1.0/0 (their C# scale rows are baked into `size()` — the Wall of Flesh's
/// x1.2) and their machines live in `update_rng`'s dispatch.
///
/// [`PumpkinDefaults::vanilla_type`] is the kind→type seam: the wave
/// rosters pick by it (`spawner.rs`'s moon ladders), `MOON_BOSS_SLOTS`
/// (`spawner.rs:75`) weights by it and `CheckProgressPumpkinMoon`'s kill
/// points (`world.rs:726`) key on it — it equals the kind's `to_u16` and
/// round-trips through `from_u16`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PumpkinDefaults {
    /// `NPC.type` — the vanilla id the family rides on the wire.
    pub vanilla_type: u16,
    /// `NPC.aiStyle` — 3 for the 11 walkers (`AI_003_Fighters`), 26 for
    /// 315/329, 22 for 330, 57 for 325, 58 for 327.
    pub ai_style: i32,
    /// `NPC.scale` — the five scarecrow pairs' scaling rows
    /// (`NPC.cs:12940-12970`); the 305/310 pair carries no row.
    pub scale: f32,
    /// `NPC.alpha` — the Poltergeist's 100 row (`NPC.cs:13202`); every
    /// other kind spawns at 0.
    pub alpha: u8,
    /// `NPC.noGravity` — 327 and 330 (`NPC.cs:13154`/`:13189`).
    pub no_gravity: bool,
    /// `NPC.noTileCollide` — 327 and 330 (same rows).
    pub no_tile_collide: bool,
}

/// The Frost Moon family's kind-level `SetDefaults` rows (`S1-F-13c` slice A:
/// the first 7 kinds — 338-340 the Zombie Elf trio, 341 the Present Mimic,
/// 342 the Gingerbread Man, 343 the Yeti, 344 the Everscream;
/// `NPC.cs:13281-13368`). The same shape as [`PumpkinDefaults`]: the fields
/// the port's tables carry piecewise for every other kind. None of the 7
/// writes `noGravity`/`noTileCollide`/`alpha` (the `SetDefaults` head's
/// false/false/0 defaults apply, `NPC.cs:8510-8537`); only 339 (1.05f,
/// `:13303`) and 340 (0.9f, `:13311`) carry a scale row.
///
/// [`FrostDefaults::vanilla_type`] is the kind→type seam, exactly like the
/// pumpkin family's: the frost ladder's picks decode through it
/// (`spawner.rs`'s `frost_moon_ladder`), `MOON_BOSS_SLOTS` weights 344 by it
/// (`npcSlots = 3f`, `NPC.cs:13367`) and `CheckProgressFrostMoon`'s kill
/// points (`world.rs`'s `frost_moon_points`) key on it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FrostDefaults {
    /// `NPC.type` — the vanilla id the family rides on the wire.
    pub vanilla_type: u16,
    /// `NPC.aiStyle` — 3 for the five walkers (`AI_003_Fighters`), 25 for
    /// 341 (the chest machine), 57 for 344.
    pub ai_style: i32,
    /// `NPC.scale` — 339's 1.05f and 340's 0.9f (`NPC.cs:13303`/`:13311`);
    /// every other kind rides the head's 1f.
    pub scale: f32,
    /// `NPC.alpha` — no kind writes a row; all spawn at 0.
    pub alpha: u8,
    /// `NPC.noGravity` — no kind writes a row (all false).
    pub no_gravity: bool,
    /// `NPC.noTileCollide` — no kind writes a row (all false).
    pub no_tile_collide: bool,
}

impl NpcKind {
    /// `NPCID.Count` (`NPCID.cs:12581`, 697) — the length of the C#'s
    /// per-type tables (`Player.npcTypeNoAggro`, `Player.cs:2994`).
    pub const CS_ID_COUNT: usize = 697;

    // The protocol id tables below must stay a single live arm per kind/id:
    // the pirate/martian merges of 2026-11 appended duplicates that the
    // compiler only warns about, so 60 dead `to_u16` arms and 28 dead
    // `from_u16` arms silently survived — and two of them were the 78..84
    // collision that sent saved Deerclops back as pirates (`S1-F-1`). A
    // duplicate is a hard error here.
    #[deny(unreachable_patterns)]
    pub fn to_u16(self) -> u16 {
        match self {
            NpcKind::Slime => 0,
            NpcKind::DemonEye => 1,
            NpcKind::WanderingEye => 2,
            NpcKind::Zombie => 3,
            NpcKind::KingSlime => 4,
            NpcKind::Guide => 5,
            NpcKind::Skeleton => 6,
            NpcKind::UndeadMiner => 7,
            NpcKind::CaveBat => 8,
            NpcKind::JungleBat => 9,
            NpcKind::Hellbat => 10,
            NpcKind::VoodooDemon => 11,
            NpcKind::EaterOfSouls => 12,
            NpcKind::Hornet => 13,
            NpcKind::MeteorHead => 14,
            NpcKind::GoblinSorcerer => 15,
            NpcKind::GoblinThief => 57,
            NpcKind::GoblinWarrior => 58,
            NpcKind::GoblinArcher => 59,
            NpcKind::GoblinPeon => 60,
            NpcKind::GoblinSummoner => 61,
            NpcKind::SnowmanGangsta => 65,
            NpcKind::SnowBalla => 66,
            NpcKind::MysterySnowman => 67,
            NpcKind::MartianOfficer => 68,
            NpcKind::GrayGrunt => 69,
            NpcKind::MartianEngineer => 70,
            NpcKind::GigaZapper => 71,
            NpcKind::MartianWalker => 72,
            // The pirates + the probe + the drone: the martian/snowman
            // family owns 65-72 and the Golem family 73-77, so this band
            // rides 182-188 — `S1-F-1` renumbered it off the 78-84 slots the
            // Deerclops/Skeletron-Prime family owns (the 2026-11 pirate
            // renumber parked the family on those ids and every Deerclops or
            // Prime limb came back as a pirate). The ids deliberately sit
            // above 181: the free holes in 0..181 (62, 148, 149, 152-155)
            // are non-contiguous and the family must not split.
            NpcKind::PirateDeckhand => 182,
            NpcKind::PirateCaptainsMate => 183,
            NpcKind::PirateArcher => 184,
            NpcKind::PirateCorsair => 185,
            NpcKind::PirateCaptain => 186,
            NpcKind::MartianProbe => 187,
            NpcKind::MartianDrone => 188,
            // The bubble/turret pair (`S1-F-13d`): the next free band above
            // S1-D-3ff wave A's 189..212 — port-local like the rest of the
            // martian family (the vanilla 384/387 are NOT the wire ids).
            NpcKind::ForceBubble => 213,
            NpcKind::MartianTurret => 214,
            NpcKind::BrainScrambler => 85,
            NpcKind::RayGunner => 86,
            NpcKind::ScutlixRider => 87,
            NpcKind::Scutlix => 88,
            NpcKind::MartianSaucer => 89,
            NpcKind::MartianSaucerTurret => 90,
            NpcKind::MartianSaucerCannon => 91,
            NpcKind::MartianSaucerCore => 92,
            NpcKind::FlyingDutchman => 93,
            NpcKind::FlyingDutchmanCannon => 94,
            NpcKind::DD2GoblinT1 => 95,
            NpcKind::DD2GoblinT2 => 96,
            NpcKind::DD2GoblinT3 => 97,
            NpcKind::DD2GoblinBomberT1 => 98,
            NpcKind::DD2GoblinBomberT2 => 99,
            NpcKind::DD2GoblinBomberT3 => 100,
            NpcKind::DD2JavelinT1 => 101,
            NpcKind::DD2JavelinT2 => 102,
            NpcKind::DD2JavelinT3 => 103,
            NpcKind::DD2SkeletonT1 => 104,
            NpcKind::DD2SkeletonT3 => 105,
            NpcKind::DD2DarkMage => 106,
            NpcKind::DD2Betsy => 107,
            NpcKind::DD2WitherBeast => 108,
            NpcKind::DD2Bartender => 109,
            NpcKind::DD2EterniaCrystal => 110,
            // The Old One's Army's tier-2/3 families (M4-EVENT-19):
            // the port ids 148-160.
            NpcKind::DD2WyvernT1 => 169,
            NpcKind::DD2WyvernT2 => 170,
            NpcKind::DD2WyvernT3 => 171,
            NpcKind::DD2DarkMageT3 => 172,
            NpcKind::DD2WitherBeastT3 => 173,
            NpcKind::DD2DrakinT2 => 174,
            NpcKind::DD2DrakinT3 => 175,
            NpcKind::DD2KoboldWalkerT2 => 176,
            NpcKind::DD2KoboldWalkerT3 => 177,
            NpcKind::DD2KoboldFlyerT2 => 178,
            NpcKind::DD2KoboldFlyerT3 => 179,
            NpcKind::DD2OgreT2 => 180,
            NpcKind::DD2OgreT3 => 181,
            NpcKind::AncientCultist => 112,
            NpcKind::StardustTower => 113,
            NpcKind::NebulaTower => 114,
            NpcKind::SolarTower => 115,
            NpcKind::VortexTower => 146,
            NpcKind::DD2LanePortal => 147,
            NpcKind::StardustCellBig => 121,
            NpcKind::StardustCellSmall => 116,
            NpcKind::VortexHunter => 127,
            NpcKind::VortexAlien => 128,
            NpcKind::VortexHeadcrab => 129,
            NpcKind::StardustSpiderBig => 133,
            NpcKind::StardustSpiderSmall => 134,
            NpcKind::StardustSoldier => 135,
            NpcKind::SolarDrakomire => 136,
            NpcKind::SolarDrakomireRider => 137,
            NpcKind::SolarSroller => 138,
            NpcKind::SolarCorite => 139,
            NpcKind::SolarCrawltipedeHead => 141,
            NpcKind::SolarCrawltipedeBody => 142,
            NpcKind::SolarCrawltipedeTail => 143,
            NpcKind::StardustWormHead => 144,
            NpcKind::SolarSpearman => 145,
            NpcKind::SolarSelenian => 140,
            NpcKind::NebulaFloater => 130,
            NpcKind::BrainSuckler => 131,
            NpcKind::Predictor => 132,
            NpcKind::MoonLordHead => 122,
            NpcKind::MoonLordHand => 123,
            NpcKind::MoonLordCore => 124,
            NpcKind::MoonLordFreeEye => 125,
            NpcKind::MoonLordLeechBlob => 126,
            NpcKind::StardustJellyfishBig => 117,
            NpcKind::NebulaBeast => 118,
            NpcKind::VortexRifleman => 119,
            NpcKind::VortexSoldier => 120,
            NpcKind::Tim => 16,
            NpcKind::DarkCaster => 17,
            NpcKind::FireImp => 18,
            NpcKind::DesertDjinn => 62,
            NpcKind::ChaosBall => 19,
            NpcKind::BurningSphere => 20,
            NpcKind::WaterSphere => 21,
            NpcKind::Goldfish => 22,
            NpcKind::Piranha => 23,
            NpcKind::Shark => 24,
            NpcKind::DevourerHead => 25,
            NpcKind::DevourerBody => 26,
            NpcKind::DevourerTail => 27,
            NpcKind::GiantWormHead => 28,
            NpcKind::GiantWormBody => 29,
            NpcKind::GiantWormTail => 30,
            NpcKind::EaterOfWorldsHead => 31,
            NpcKind::EaterOfWorldsBody => 32,
            NpcKind::EaterOfWorldsTail => 33,
            NpcKind::Merchant => 34,
            NpcKind::Dryad => 35,
            NpcKind::Nurse => 36,
            NpcKind::ArmsDealer => 37,
            NpcKind::Demolitionist => 38,
            NpcKind::Painter => 39,
            NpcKind::WitchDoctor => 40,
            NpcKind::Clothier => 41,
            NpcKind::EyeOfCthulhu => 42,
            NpcKind::QueenBee => 43,
            NpcKind::SkeletronHead => 44,
            NpcKind::ServantOfCthulhu => 45,
            NpcKind::WallOfFlesh => 46,
            NpcKind::Hungry => 47,
            NpcKind::LeechHead => 48,
            NpcKind::LeechBody => 49,
            // W-A2-3: the leech tail takes the free 148 hole (the
            // port-local protocol space, the S1-F-1 pattern).
            NpcKind::LeechTail => 148,
            NpcKind::TheDestroyer => 50,
            // S1-D-3-wire-destroyer: the body/tail/probe kinds (vanilla
            // types 135/136/139, `NPCID.cs:10947-10949`/`:10955`; the
            // protocol space is port-local). GATE REASSIGN (L-B): the
            // branch's 148/149/152 collide with the landed queenbee bees
            // (149/152, save-compat) + R78's reserved LeechTail 148; the
            // three take the next-free 216/217/218 (213/214 R10 reserved).
            NpcKind::TheDestroyerBody => 216,
            NpcKind::TheDestroyerTail => 217,
            NpcKind::Probe => 218,
            NpcKind::SkeletronPrime => 51,
            NpcKind::Retinazer => 52,
            NpcKind::Spazmatism => 53,
            NpcKind::Plantera => 54,
            NpcKind::BrainOfCthulhu => 55,
            NpcKind::Creeper => 56,
            NpcKind::BoundGoblin => 63,
            NpcKind::GoblinTinkerer => 64,
            NpcKind::Golem => 73,
            NpcKind::GolemHead => 74,
            NpcKind::GolemFistLeft => 75,
            NpcKind::GolemFistRight => 76,
            NpcKind::GolemHeadFree => 77,
            NpcKind::Deerclops => 78,
            NpcKind::SkeletronHand => 79,
            NpcKind::WallOfFleshEye => 80,
            NpcKind::PrimeCannon => 81,
            NpcKind::PrimeSaw => 82,
            NpcKind::PrimeVice => 83,
            NpcKind::PrimeLaser => 84,
            NpcKind::TravellingMerchant => 111,
            NpcKind::Angler => 150,
            NpcKind::SleepingAngler => 151,
            NpcKind::Wizard => 156,
            NpcKind::BoundWizard => 157,
            NpcKind::Mechanic => 158,
            NpcKind::BoundMechanic => 159,
            NpcKind::Stylist => 160,
            NpcKind::WebbedStylist => 161,
            NpcKind::TaxCollector => 162,
            NpcKind::Golfer => 163,
            NpcKind::SleepingGolfer => 164,
            NpcKind::Zoologist => 165,
            NpcKind::Princess => 166,
            NpcKind::DyeTrader => 167,
            NpcKind::PartyGirl => 168,
            // The Pumpkin Moon family (`S1-F-13b`): these 16 kinds are the
            // roster that rides the VANILLA `NPC.type` on the wire — the
            // F-13a moon ladders pick by it, `MOON_BOSS_SLOTS`
            // (`spawner.rs:75`) and `CheckProgressPumpkinMoon`'s kill points
            // (`world.rs:726`) key on it, and the ids sit above the whole
            // 0..188 port-local space (the 78..84 Deerclops/Prime band
            // included) so nothing collides. The port's other kinds keep
            // their port-local ids; `NpcKind::pumpkin_moon_type` is the
            // named kind→type seam the spawn wiring and the kill walk use.
            // (`S1-F-13d` appended the ForceBubble/MartianTurret 213/214
            // pair above S1-D-3ff wave A's 189..212 band.)
            // `NPC.cs:12929-12977` (305-314), `:12978` (315), `:13115`
            // (325), `:13129` (326), `:13142` (327), `:13175` (329),
            // `:13188` (330).
            NpcKind::Scarecrow1 => 305,
            NpcKind::Scarecrow2 => 306,
            NpcKind::Scarecrow3 => 307,
            NpcKind::Scarecrow4 => 308,
            NpcKind::Scarecrow5 => 309,
            NpcKind::Scarecrow6 => 310,
            NpcKind::Scarecrow7 => 311,
            NpcKind::Scarecrow8 => 312,
            NpcKind::Scarecrow9 => 313,
            NpcKind::Scarecrow10 => 314,
            NpcKind::HeadlessHorseman => 315,
            NpcKind::MourningWood => 325,
            NpcKind::Splinterling => 326,
            NpcKind::Pumpking => 327,
            NpcKind::Hellhound => 329,
            NpcKind::Poltergeist => 330,
            // The Frost Moon family (`S1-F-13c` slice B): the vanilla ids
            // 345-352 encode here (the same ids `from_u16` decodes below) —
            // the frost wave roster, `MOON_BOSS_SLOTS` (`spawner.rs:75`) and
            // `CheckProgressFrostMoon`'s kill points (`world.rs:733`) key on
            // the vanilla `NPC.type`, and the ids sit above the whole
            // port-local space so nothing collides. `NpcKind::frost_moon_type`
            // is the named kind→type seam the spawn wiring and the kill walk
            // use. `NPC.cs:13369` (345), `:13386` (346), `:13400` (347),
            // `:13416` (348), `:13430` (349), `:13444` (350), `:13457`
            // (351), `:13471` (352).
            NpcKind::IceQueen => 345,
            NpcKind::SantaNK1 => 346,
            NpcKind::ElfCopter => 347,
            NpcKind::Nutcracker => 348,
            NpcKind::NutcrackerSpinning => 349,
            NpcKind::ElfArcher => 350,
            NpcKind::Krampus => 351,
            NpcKind::Flocko => 352,
            // S1-D-3ff wave A — the 24 LOW-id kinds take the free
            // 189..212 band (the port-local space; the moon kinds ride
            // the vanilla 305-330 ids above it).
            NpcKind::MotherSlime => 189,
            NpcKind::LavaSlime => 190,
            NpcKind::DungeonSlime => 191,
            NpcKind::CorruptSlime => 192,
            NpcKind::IlluminantSlime => 193,
            NpcKind::IceSlime => 194,
            NpcKind::Crimslime => 195,
            NpcKind::SpikedIceSlime => 196,
            NpcKind::UmbrellaSlime => 197,
            NpcKind::CataractEye => 198,
            NpcKind::SleepyEye => 199,
            NpcKind::DialatedEye => 200,
            NpcKind::GreenEye => 201,
            NpcKind::PurpleEye => 202,
            NpcKind::DoctorBones => 203,
            NpcKind::TheGroom => 204,
            NpcKind::BaldZombie => 205,
            NpcKind::ZombieEskimo => 206,
            NpcKind::PincushionZombie => 207,
            NpcKind::SlimedZombie => 208,
            NpcKind::SwampZombie => 209,
            NpcKind::TwiggyZombie => 210,
            NpcKind::FemaleZombie => 211,
            NpcKind::ZombieRaincoat => 212,
            // S1-D-3 W-9 fishron wiring — the boss + its three hazards take
            // the free 219..222 band (gate-reassigned; 213/214 R10-reserved).
            // The vanilla ids 370-373 stay free for the wire's
            // `net_id`/bestiary space.
            NpcKind::DukeFishron => 219,
            NpcKind::DetonatingBubble => 220,
            NpcKind::Sharkron => 221,
            NpcKind::Sharkron2 => 222,
            // S1-D-3z (zombie fighter audit) — the five holiday zombie
            // variants + the blood-moon pair. GATE REASSIGN (L-I, re-scan
            // after the wyvern forward-merge): the branch's 221..227
            // collide with the landed fishron 219-222 AND wyvern 223-228
            // bands — next-free ascending after a to_u16 scan of the
            // merged lane tree: 230..236 (213/214 martian-defaults taken,
            // 148 LeechTail / 353 wheel reserved). The vanilla types ride
            // `cs_id`/`net_id` (319-321/331-332/489/490), NOT these
            // protocol ids — the spawner's holiday arms
            // (`NPC.cs:4727-4735`) pick by the vanilla type.
            NpcKind::ZombieDoctor => 230,
            NpcKind::ZombieSuperman => 231,
            NpcKind::ZombiePixie => 232,
            NpcKind::ZombieXmas => 233,
            NpcKind::ZombieSweater => 234,
            NpcKind::BloodZombie => 235,
            NpcKind::Drippler => 236,
            // The Frost Moon family, slice A (`S1-F-13c`): the vanilla ids
            // 338-344, above the whole port-local space exactly like the
            // pumpkin band. `NPC.cs:13281-13313` (338-340), `:13314-13326`
            // (341), `:13327-13339` (342), `:13340-13354` (343),
            // `:13355-13368` (344).
            NpcKind::ZombieElf => 338,
            NpcKind::ZombieElfBeard => 339,
            NpcKind::ZombieElfGirl => 340,
            NpcKind::PresentMimic => 341,
            NpcKind::GingerbreadMan => 342,
            NpcKind::Yeti => 343,
            NpcKind::Everscream => 344,
            // The Queen Bee's volley minions (S1-D-3 W-3): the vanilla
            // type ids (`NPCID.Bee` 210 / `BeeSmall` 211, `NPCID.cs:11097-11099`).
            // The vanilla 210/211 protocol slots ride the wave-A Twiggy/
            // Female zombies (`S1-D-3ff`). GATE REASSIGN (L-B): the branch's
            // 353/354 collide with R54c smallkinds' BlazingWheel 353 band —
            // R54c keeps 353; the bees take the next-free 149/152 holes
            // (148 reserved for R78 LeechTail). Save-compat safe: neither
            // branch had merged when the ids moved.
            NpcKind::Bee => 149,
            NpcKind::BeeSmall => 152,
            // The Plantera minion family (S1-D-3-wire-plantera-minions,
            // census §5 W-8): the C# types 261/263/264/265 (`NPCID.cs:11199`/
            // `:11203`/`:11205`/`:11207`). GATE REASSIGN (L-B): the branch's
            // port-local 353-356 collide with R54c wheel 353 + queenbee's
            // 353/354 (now 149/152); the four take the next-free 153/154/155
            // + 215 holes (148 R78, 213/214 R10 reserved). cs_id unchanged.
            NpcKind::FungiSpore => 153,
            NpcKind::PlanterasHook => 154,
            NpcKind::PlanterasTentacle => 155,
            NpcKind::Spore => 215,
            // The small-kinds wiring wave (S1-D-3-wire-smallkinds): the
            // BlazingWheel's vanilla id 72 is the MartianWalker's protocol
            // slot (`to_u16`'s bijection guard), so it takes the next free
            // port-local id 353; AngryNimbus/DungeonSpirit keep their
            // vanilla ids (250/288 free). GATE DEDUPE (L-B): the branch's
            // ForceBubble 384 is the SAME kind S1-F-13d landed at 213 —
            // one variant, one id (213); 384 frees. `NPC.cs:9714-9731` /
            // `:12221-12233` / `:12738-12752`.
            NpcKind::BlazingWheel => 353,
            NpcKind::AngryNimbus => 250,
            NpcKind::DungeonSpirit => 288,
            // The Wyvern family (`S1-D-3` wyvern, `NPCID.cs:10851-10861`):
            // the vanilla ids 87-92 collide with the port-local
            // Scutlix/saucer band (87-92 above). GATE REASSIGN (L-C): the
            // branch's 213-218 band TRIPLE-collides with R10 martian
            // 213/214 (keeps, earliest push) + R50 fishron 219-222 + R55
            // destroyer 216-218 — the six take the next-free 223-228
            // (148 reserved R78 LeechTail). Save-compat safe: the branch
            // had not merged when the ids moved. The vanilla `NPC.type`
            // rides the bestiary entry (`net_id` 87, the segments fold
            // onto the head's entry like every other worm family).
            NpcKind::WyvernHead => 223,
            NpcKind::WyvernLegs => 224,
            NpcKind::WyvernBody => 225,
            NpcKind::WyvernBody2 => 226,
            NpcKind::WyvernBody3 => 227,
            NpcKind::WyvernTail => 228,
            // W4-d42 — the NPCID 578-630 roster: the port-local 354..404
            // band, one id per kind in vanilla-type order (the pirate
            // renumber's precedent — the vanilla 578-630 slots are NOT
            // the wire ids; the real types ride `cs_id`/`net_id`). 354
            // is the next free slot above the BlazingWheel's 353.
            NpcKind::DD2LightningBug => 354,
            NpcKind::BartenderUnconscious => 355,
            NpcKind::WalkingAntlion => 356,
            NpcKind::FlyingAntlion => 357,
            NpcKind::LarvaeAntlion => 358,
            NpcKind::FairyCritterPink => 359,
            NpcKind::FairyCritterGreen => 360,
            NpcKind::FairyCritterBlue => 361,
            NpcKind::ZombieMerman => 362,
            NpcKind::EyeballFlyingFish => 363,
        }
    }

    /// The protocol id decode — the inverse of [`NpcKind::to_u16`], a
    /// bijection over the whole kind roster (`S1-F-1`); the unmatched ids
    /// (the free 62 hole + the gaps above the landed bands) decode to
    /// `None` (148 = LeechTail since W-A2-3). One
    /// live arm per id: a duplicate is a hard error (`to_u16`'s guard above).
    #[deny(unreachable_patterns)]
    pub fn from_u16(v: u16) -> Option<Self> {
        Some(match v {
            0 => NpcKind::Slime,
            1 => NpcKind::DemonEye,
            2 => NpcKind::WanderingEye,
            3 => NpcKind::Zombie,
            4 => NpcKind::KingSlime,
            5 => NpcKind::Guide,
            6 => NpcKind::Skeleton,
            7 => NpcKind::UndeadMiner,
            8 => NpcKind::CaveBat,
            9 => NpcKind::JungleBat,
            10 => NpcKind::Hellbat,
            11 => NpcKind::VoodooDemon,
            12 => NpcKind::EaterOfSouls,
            13 => NpcKind::Hornet,
            14 => NpcKind::MeteorHead,
            15 => NpcKind::GoblinSorcerer,
            57 => NpcKind::GoblinThief,
            58 => NpcKind::GoblinWarrior,
            59 => NpcKind::GoblinArcher,
            60 => NpcKind::GoblinPeon,
            61 => NpcKind::GoblinSummoner,
            65 => NpcKind::SnowmanGangsta,
            66 => NpcKind::SnowBalla,
            67 => NpcKind::MysterySnowman,
            68 => NpcKind::MartianOfficer,
            69 => NpcKind::GrayGrunt,
            70 => NpcKind::MartianEngineer,
            71 => NpcKind::GigaZapper,
            72 => NpcKind::MartianWalker,
            // The martian ground family: `to_u16` encoded the 85-88 ids but
            // `from_u16` had no arms, so a save or net message carrying one
            // decoded to `None` and the NPC was dropped (`S1-F-1`).
            85 => NpcKind::BrainScrambler,
            86 => NpcKind::RayGunner,
            87 => NpcKind::ScutlixRider,
            88 => NpcKind::Scutlix,
            89 => NpcKind::MartianSaucer,
            90 => NpcKind::MartianSaucerTurret,
            91 => NpcKind::MartianSaucerCannon,
            92 => NpcKind::MartianSaucerCore,
            93 => NpcKind::FlyingDutchman,
            94 => NpcKind::FlyingDutchmanCannon,
            95 => NpcKind::DD2GoblinT1,
            96 => NpcKind::DD2GoblinT2,
            97 => NpcKind::DD2GoblinT3,
            98 => NpcKind::DD2GoblinBomberT1,
            99 => NpcKind::DD2GoblinBomberT2,
            100 => NpcKind::DD2GoblinBomberT3,
            101 => NpcKind::DD2JavelinT1,
            102 => NpcKind::DD2JavelinT2,
            103 => NpcKind::DD2JavelinT3,
            104 => NpcKind::DD2SkeletonT1,
            105 => NpcKind::DD2SkeletonT3,
            106 => NpcKind::DD2DarkMage,
            107 => NpcKind::DD2Betsy,
            108 => NpcKind::DD2WitherBeast,
            109 => NpcKind::DD2Bartender,
            110 => NpcKind::DD2EterniaCrystal,
            147 => NpcKind::DD2LanePortal,
            169 => NpcKind::DD2WyvernT1,
            170 => NpcKind::DD2WyvernT2,
            171 => NpcKind::DD2WyvernT3,
            172 => NpcKind::DD2DarkMageT3,
            173 => NpcKind::DD2WitherBeastT3,
            174 => NpcKind::DD2DrakinT2,
            175 => NpcKind::DD2DrakinT3,
            176 => NpcKind::DD2KoboldWalkerT2,
            177 => NpcKind::DD2KoboldWalkerT3,
            178 => NpcKind::DD2KoboldFlyerT2,
            179 => NpcKind::DD2KoboldFlyerT3,
            180 => NpcKind::DD2OgreT2,
            181 => NpcKind::DD2OgreT3,
            112 => NpcKind::AncientCultist,
            113 => NpcKind::StardustTower,
            114 => NpcKind::NebulaTower,
            115 => NpcKind::SolarTower,
            146 => NpcKind::VortexTower,
            116 => NpcKind::StardustCellSmall,
            117 => NpcKind::StardustJellyfishBig,
            118 => NpcKind::NebulaBeast,
            119 => NpcKind::VortexRifleman,
            120 => NpcKind::VortexSoldier,
            127 => NpcKind::VortexHunter,
            128 => NpcKind::VortexAlien,
            129 => NpcKind::VortexHeadcrab,
            133 => NpcKind::StardustSpiderBig,
            134 => NpcKind::StardustSpiderSmall,
            135 => NpcKind::StardustSoldier,
            136 => NpcKind::SolarDrakomire,
            137 => NpcKind::SolarDrakomireRider,
            138 => NpcKind::SolarSroller,
            139 => NpcKind::SolarCorite,
            141 => NpcKind::SolarCrawltipedeHead,
            142 => NpcKind::SolarCrawltipedeBody,
            143 => NpcKind::SolarCrawltipedeTail,
            144 => NpcKind::StardustWormHead,
            145 => NpcKind::SolarSpearman,
            140 => NpcKind::SolarSelenian,
            130 => NpcKind::NebulaFloater,
            131 => NpcKind::BrainSuckler,
            132 => NpcKind::Predictor,
            122 => NpcKind::MoonLordHead,
            123 => NpcKind::MoonLordHand,
            124 => NpcKind::MoonLordCore,
            125 => NpcKind::MoonLordFreeEye,
            126 => NpcKind::MoonLordLeechBlob,
            121 => NpcKind::StardustCellBig,
            16 => NpcKind::Tim,
            17 => NpcKind::DarkCaster,
            18 => NpcKind::FireImp,
            19 => NpcKind::ChaosBall,
            20 => NpcKind::BurningSphere,
            21 => NpcKind::WaterSphere,
            22 => NpcKind::Goldfish,
            23 => NpcKind::Piranha,
            24 => NpcKind::Shark,
            25 => NpcKind::DevourerHead,
            26 => NpcKind::DevourerBody,
            27 => NpcKind::DevourerTail,
            28 => NpcKind::GiantWormHead,
            29 => NpcKind::GiantWormBody,
            30 => NpcKind::GiantWormTail,
            31 => NpcKind::EaterOfWorldsHead,
            32 => NpcKind::EaterOfWorldsBody,
            33 => NpcKind::EaterOfWorldsTail,
            34 => NpcKind::Merchant,
            35 => NpcKind::Dryad,
            36 => NpcKind::Nurse,
            37 => NpcKind::ArmsDealer,
            38 => NpcKind::Demolitionist,
            39 => NpcKind::Painter,
            40 => NpcKind::WitchDoctor,
            41 => NpcKind::Clothier,
            42 => NpcKind::EyeOfCthulhu,
            43 => NpcKind::QueenBee,
            44 => NpcKind::SkeletronHead,
            45 => NpcKind::ServantOfCthulhu,
            46 => NpcKind::WallOfFlesh,
            47 => NpcKind::Hungry,
            48 => NpcKind::LeechHead,
            49 => NpcKind::LeechBody,
            148 => NpcKind::LeechTail,
            50 => NpcKind::TheDestroyer,
            // S1-D-3-wire-destroyer: the gate-reassigned 216/217/218
            // holes (`to_u16` above; vanilla types 135/136/139).
            216 => NpcKind::TheDestroyerBody,
            217 => NpcKind::TheDestroyerTail,
            218 => NpcKind::Probe,
            51 => NpcKind::SkeletronPrime,
            52 => NpcKind::Retinazer,
            53 => NpcKind::Spazmatism,
            54 => NpcKind::Plantera,
            55 => NpcKind::BrainOfCthulhu,
            56 => NpcKind::Creeper,
            62 => NpcKind::DesertDjinn,
            63 => NpcKind::BoundGoblin,
            64 => NpcKind::GoblinTinkerer,
            73 => NpcKind::Golem,
            74 => NpcKind::GolemHead,
            75 => NpcKind::GolemFistLeft,
            76 => NpcKind::GolemFistRight,
            77 => NpcKind::GolemHeadFree,
            // 78-84: the Deerclops/Skeletron-Prime family — the band the
            // pirates vacated in the `S1-F-1` renumber (`from_u16` resolved
            // these ids to the pirates before it, which is how a saved
            // Deerclops came back as a Pirate Deckhand).
            78 => NpcKind::Deerclops,
            79 => NpcKind::SkeletronHand,
            80 => NpcKind::WallOfFleshEye,
            81 => NpcKind::PrimeCannon,
            82 => NpcKind::PrimeSaw,
            83 => NpcKind::PrimeVice,
            84 => NpcKind::PrimeLaser,
            111 => NpcKind::TravellingMerchant,
            150 => NpcKind::Angler,
            151 => NpcKind::SleepingAngler,
            156 => NpcKind::Wizard,
            157 => NpcKind::BoundWizard,
            158 => NpcKind::Mechanic,
            159 => NpcKind::BoundMechanic,
            160 => NpcKind::Stylist,
            161 => NpcKind::WebbedStylist,
            162 => NpcKind::TaxCollector,
            163 => NpcKind::Golfer,
            164 => NpcKind::SleepingGolfer,
            165 => NpcKind::Zoologist,
            166 => NpcKind::Princess,
            167 => NpcKind::DyeTrader,
            168 => NpcKind::PartyGirl,
            // The pirates + the probe + the drone — the `S1-F-1` renumber
            // (`to_u16` above): the family moved off the 78-84
            // Deerclops/Skeletron-Prime band to the contiguous 182-188.
            182 => NpcKind::PirateDeckhand,
            183 => NpcKind::PirateCaptainsMate,
            184 => NpcKind::PirateArcher,
            185 => NpcKind::PirateCorsair,
            186 => NpcKind::PirateCaptain,
            187 => NpcKind::MartianProbe,
            188 => NpcKind::MartianDrone,
            // The bubble/turret pair (`S1-F-13d`) — `NPC.cs:13882` (384)
            // and `:13922` (387) ride the port-local 213/214.
            213 => NpcKind::ForceBubble,
            214 => NpcKind::MartianTurret,
            // The Pumpkin Moon family (`S1-F-13b`): the vanilla ids
            // 305-314/315/325/326/327/329/330 decode here (the same ids
            // `to_u16` encodes above) — they sit above the whole port-local
            // space, so no decode arm below ever sees them.
            // `NPC.cs:12929-12977` (305-314), `:12978` (315), `:13115`
            // (325), `:13129` (326), `:13142` (327), `:13175` (329),
            // `:13188` (330).
            305 => NpcKind::Scarecrow1,
            306 => NpcKind::Scarecrow2,
            307 => NpcKind::Scarecrow3,
            308 => NpcKind::Scarecrow4,
            309 => NpcKind::Scarecrow5,
            310 => NpcKind::Scarecrow6,
            311 => NpcKind::Scarecrow7,
            312 => NpcKind::Scarecrow8,
            313 => NpcKind::Scarecrow9,
            314 => NpcKind::Scarecrow10,
            315 => NpcKind::HeadlessHorseman,
            325 => NpcKind::MourningWood,
            326 => NpcKind::Splinterling,
            327 => NpcKind::Pumpking,
            329 => NpcKind::Hellhound,
            330 => NpcKind::Poltergeist,
            // The Frost Moon family (`S1-F-13c` slice B): the vanilla ids
            // 345-352 decode here (the same ids `to_u16` encodes above) —
            // they sit above the whole port-local space, so no decode arm
            // below ever sees them. The 338-344 band lands on a separate
            // task row. `NPC.cs:13369` (345), `:13386` (346), `:13400`
            // (347), `:13416` (348), `:13430` (349), `:13444` (350),
            // `:13457` (351), `:13471` (352).
            345 => NpcKind::IceQueen,
            346 => NpcKind::SantaNK1,
            347 => NpcKind::ElfCopter,
            348 => NpcKind::Nutcracker,
            349 => NpcKind::NutcrackerSpinning,
            350 => NpcKind::ElfArcher,
            351 => NpcKind::Krampus,
            352 => NpcKind::Flocko,
            189 => NpcKind::MotherSlime,
            190 => NpcKind::LavaSlime,
            191 => NpcKind::DungeonSlime,
            192 => NpcKind::CorruptSlime,
            193 => NpcKind::IlluminantSlime,
            194 => NpcKind::IceSlime,
            195 => NpcKind::Crimslime,
            196 => NpcKind::SpikedIceSlime,
            197 => NpcKind::UmbrellaSlime,
            198 => NpcKind::CataractEye,
            199 => NpcKind::SleepyEye,
            200 => NpcKind::DialatedEye,
            201 => NpcKind::GreenEye,
            202 => NpcKind::PurpleEye,
            203 => NpcKind::DoctorBones,
            204 => NpcKind::TheGroom,
            205 => NpcKind::BaldZombie,
            206 => NpcKind::ZombieEskimo,
            207 => NpcKind::PincushionZombie,
            208 => NpcKind::SlimedZombie,
            209 => NpcKind::SwampZombie,
            210 => NpcKind::TwiggyZombie,
            211 => NpcKind::FemaleZombie,
            212 => NpcKind::ZombieRaincoat,
            // S1-D-3 W-9 fishron wiring — the gate-reassigned 219..222 decode arms (`to_u16` above).
            219 => NpcKind::DukeFishron,
            220 => NpcKind::DetonatingBubble,
            221 => NpcKind::Sharkron,
            222 => NpcKind::Sharkron2,
            // S1-D-3z — the holiday zombie band + blood-moon pair decode
            // here (the same ids `to_u16` encodes above; GATE REASSIGN
            // L-I re-scan post-wyvern: 230..236 next-free; fishron
            // 219-222 + wyvern 223-228 landed). `NPC.cs:13033-13074`
            // (319-321), `:13204-13229` (331-332), `:14605-14631` (489/490).
            230 => NpcKind::ZombieDoctor,
            231 => NpcKind::ZombieSuperman,
            232 => NpcKind::ZombiePixie,
            233 => NpcKind::ZombieXmas,
            234 => NpcKind::ZombieSweater,
            235 => NpcKind::BloodZombie,
            236 => NpcKind::Drippler,
            // The Frost Moon family, slice A (`S1-F-13c`): the vanilla ids
            // 338-344 decode here (the same ids `to_u16` encodes above).
            // `NPC.cs:13281-13368`.
            338 => NpcKind::ZombieElf,
            339 => NpcKind::ZombieElfBeard,
            340 => NpcKind::ZombieElfGirl,
            341 => NpcKind::PresentMimic,
            342 => NpcKind::GingerbreadMan,
            343 => NpcKind::Yeti,
            344 => NpcKind::Everscream,
            // The Queen Bee's volley minions (S1-D-3 W-3) — the gate-
            // reassigned 149/152 holes (`to_u16` above; the vanilla 210/211
            // slots ride the wave-A Twiggy/Female zombies; 353 stays R54c's).
            149 => NpcKind::Bee,
            152 => NpcKind::BeeSmall,
            // The Plantera minion family (S1-D-3-wire-plantera-minions):
            // the gate-reassigned 153/154/155/215 holes (`to_u16` above).
            153 => NpcKind::FungiSpore,
            154 => NpcKind::PlanterasHook,
            155 => NpcKind::PlanterasTentacle,
            215 => NpcKind::Spore,
            // The small-kinds wiring wave: 353 is the BlazingWheel's
            // port-local id (the vanilla 72 slot is the MartianWalker's);
            // 250/288 keep their vanilla ids (`to_u16` above). ForceBubble
            // decodes at S1-F-13d's 213 (gate dedupe — the branch's 384 freed).
            353 => NpcKind::BlazingWheel,
            250 => NpcKind::AngryNimbus,
            288 => NpcKind::DungeonSpirit,
            // The Wyvern family (`S1-D-3` wyvern): the gate-reassigned
            // 223-228 band (`to_u16` above; the vanilla 87-92 collide with
            // the Scutlix/saucer ids). `NPCID.cs:10851-10861`.
            223 => NpcKind::WyvernHead,
            224 => NpcKind::WyvernLegs,
            225 => NpcKind::WyvernBody,
            226 => NpcKind::WyvernBody2,
            227 => NpcKind::WyvernBody3,
            228 => NpcKind::WyvernTail,
            // W4-d42 — the NPCID 578-630 roster's port-local 354..404
            // band (`to_u16` above).
            354 => NpcKind::DD2LightningBug,
            355 => NpcKind::BartenderUnconscious,
            356 => NpcKind::WalkingAntlion,
            357 => NpcKind::FlyingAntlion,
            358 => NpcKind::LarvaeAntlion,
            359 => NpcKind::FairyCritterPink,
            360 => NpcKind::FairyCritterGreen,
            361 => NpcKind::FairyCritterBlue,
            362 => NpcKind::ZombieMerman,
            363 => NpcKind::EyeballFlyingFish,
            _ => return None,
        })
    }

    /// `npc.townNPC` (`NPC.cs:6537`) — the flag the town `SetDefaults`
    /// cases raise: the town roster, the goblin pair and the Travelling
    /// Merchant himself (`NPC.cs:13675`'s `townNPC = true`). The Old Man
    /// (37) and the Skeleton Merchant (453) carry the flag in the
    /// vanilla; neither is ported, so the spawner's town counts need no
    /// exclusions (`Main.cs:65030`).
    pub fn is_town_resident(self) -> bool {
        matches!(
            self,
            NpcKind::Guide
                | NpcKind::Merchant
                | NpcKind::Dryad
                | NpcKind::Nurse
                | NpcKind::ArmsDealer
                | NpcKind::Demolitionist
                | NpcKind::Painter
                | NpcKind::WitchDoctor
                | NpcKind::Clothier
                | NpcKind::GoblinTinkerer
                | NpcKind::TravellingMerchant
                | NpcKind::Angler
                | NpcKind::Wizard
                | NpcKind::Mechanic
                | NpcKind::Stylist
                | NpcKind::TaxCollector
                | NpcKind::Golfer
                | NpcKind::Zoologist
                | NpcKind::Princess
                | NpcKind::DyeTrader
                | NpcKind::PartyGirl
        )
    }

    /// The worm family role (`AI_006`'s type groups): the head digs, the
    /// body links, the tail ends (`:51393`'s body list, `:51436`'s
    /// head/mid list).
    pub fn is_worm(self) -> bool {
        matches!(
            self,
            NpcKind::DevourerHead
                | NpcKind::DevourerBody
                | NpcKind::DevourerTail
                | NpcKind::GiantWormHead
                | NpcKind::GiantWormBody
                | NpcKind::GiantWormTail
                | NpcKind::EaterOfWorldsHead
                | NpcKind::EaterOfWorldsBody
                | NpcKind::EaterOfWorldsTail
                | NpcKind::LeechHead
                | NpcKind::LeechBody
                | NpcKind::LeechTail
                | NpcKind::SolarCrawltipedeHead
                | NpcKind::SolarCrawltipedeBody
                | NpcKind::SolarCrawltipedeTail
                // W-A2-3 (T3f): the stardust worm head IS an AI_006 kind
                // (`NPC.cs:51564-51567`'s `flag3 = true`, the `:51740`
                // tunables) — a lone minion with no chain (type 402 sits
                // outside the `:51301` spawn lists, so `is_worm_head`
                // stays false and the init never grows links).
                | NpcKind::StardustWormHead
        )
    }

    /// `NPC.realLife` (`NPC.cs:6161`) — the worm families whose segments
    /// share the HEAD's life pool. The vanilla's `AI_006_Worms` gives the
    /// head `ai[3] = whoAmI` + `realLife = whoAmI` and hands the link to
    /// every child (`NPC.cs:51305-51363`); `AI_037_Destroyer` does the same
    /// for the Destroyer (`NPC.cs:49836-49853`) and the crawltipede's own
    /// arm for its chain (`NPC.cs:51369-51392`). The Eater of Worlds is the
    /// exception: C# forces `realLife = -1` for types 13-15
    /// (`NPC.cs:51129-51132`), so every EoW segment owns its own life.
    pub fn shares_carrier_life(self) -> bool {
        matches!(
            self,
            NpcKind::DevourerHead
                | NpcKind::DevourerBody
                | NpcKind::DevourerTail
                | NpcKind::GiantWormHead
                | NpcKind::GiantWormBody
                | NpcKind::GiantWormTail
                | NpcKind::LeechHead
                | NpcKind::LeechBody
                | NpcKind::LeechTail
                | NpcKind::TheDestroyer
                | NpcKind::TheDestroyerBody
                | NpcKind::TheDestroyerTail
                | NpcKind::SolarCrawltipedeHead
                | NpcKind::SolarCrawltipedeBody
                | NpcKind::SolarCrawltipedeTail
                // The wyvern chain shares the head's life pool
                // (`NPC.cs:51158-51159`/`:51182-51183`: `ai[3] = whoAmI`,
                // `realLife = whoAmI` on the head and every link). Inert
                // until the wiring sets `worm_id`/`seg_index` (the
                // carrier resolution keys on them, `npc.rs:3419`).
                | NpcKind::WyvernHead
                | NpcKind::WyvernLegs
                | NpcKind::WyvernBody
                | NpcKind::WyvernBody2
                | NpcKind::WyvernBody3
                | NpcKind::WyvernTail
        )
    }

    /// `NPC.takenDamageMultiplier` (`NPC.cs:6521`) — the damage multiplier
    /// the vanilla applies to the strike before it lands
    /// (`NPC.cs:81244-81246`). The solar crawltipede's tail (414) carries
    /// 10 (`NPC.cs:15316`); every other type keeps 1.
    pub fn taken_damage_multiplier(self) -> f32 {
        match self {
            NpcKind::SolarCrawltipedeTail => 10.0,
            _ => 1.0,
        }
    }

    /// The Destroyer family left the worm pass with S1-D-3-wire-destroyer:
    /// `AI_037` is its own machine (`npc_ai_style37.rs` +
    /// `world_sim/npcs.rs::update_destroyers`), NOT `AI_006` — the family is
    /// deliberately absent from `is_worm`/`is_worm_head`/`is_worm_mid`.
    pub fn is_worm_head(self) -> bool {
        matches!(
            self,
            NpcKind::DevourerHead
                | NpcKind::GiantWormHead
                | NpcKind::EaterOfWorldsHead
                | NpcKind::LeechHead
                | NpcKind::SolarCrawltipedeHead
        )
    }

    /// The mid-body kinds that spawn another of themselves while the
    /// budget holds (`:51350`'s list; the Destroyer family runs its own
    /// `AI_037` chain, not this pass).
    pub fn is_worm_mid(self) -> bool {
        matches!(
            self,
            NpcKind::DevourerBody
                | NpcKind::GiantWormBody
                | NpcKind::EaterOfWorldsBody
                | NpcKind::LeechBody
                | NpcKind::SolarCrawltipedeBody
        )
    }

    /// The next segment's kind down the chain (`type + 1` in the vanilla
    /// spawns, `:51347`/`:51357`).
    pub fn worm_next(self) -> NpcKind {
        match self {
            NpcKind::DevourerHead => NpcKind::DevourerBody,
            NpcKind::DevourerBody | NpcKind::DevourerTail => NpcKind::DevourerTail,
            NpcKind::GiantWormHead => NpcKind::GiantWormBody,
            NpcKind::GiantWormBody | NpcKind::GiantWormTail => NpcKind::GiantWormTail,
            NpcKind::EaterOfWorldsHead => NpcKind::EaterOfWorldsBody,
            NpcKind::LeechHead => NpcKind::LeechBody,
            // W-A2-3 (T3e): the budget-exhausted body spawns `type + 1`
            // = the 119 tail (`NPC.cs:51357`); the tail closes the chain
            // (it sits outside both spawn lists, `:51301`/`:51350`).
            NpcKind::LeechBody | NpcKind::LeechTail => NpcKind::LeechTail,
            // The crawltipede chain (`:51375-51381`'s per-index kind):
            // the bodies self-chain and the tail closes it.
            NpcKind::SolarCrawltipedeHead => NpcKind::SolarCrawltipedeBody,
            NpcKind::SolarCrawltipedeBody | NpcKind::SolarCrawltipedeTail => {
                NpcKind::SolarCrawltipedeTail
            }
            _ => NpcKind::EaterOfWorldsTail,
        }
    }

    /// The head's dig tunables — `num45` (the max speed) and `num46` (the
    /// steering accel) per family (`:51670-51706`).
    pub fn worm_tunables(self) -> (f32, f32) {
        match self {
            // The stardust worm head's 9/0.3 (`:51740-51743`) and the
            // crawltipede chain's 10/0.3 (`:51779-51880`).
            NpcKind::StardustWormHead => (9.0, 0.3),
            NpcKind::SolarCrawltipedeHead
            | NpcKind::SolarCrawltipedeBody
            | NpcKind::SolarCrawltipedeTail => (10.0, 0.3),
            NpcKind::DevourerHead => (9.0, 0.1),
            NpcKind::GiantWormHead => (6.0, 0.05),
            NpcKind::EaterOfWorldsHead => (10.0, 0.07),
            // The wyvern head's 11/0.25 (`:51720-51724`). The kind stays
            // OUT of `is_worm`/`is_worm_head` (the StardustWormHead
            // precedent): the shared pass's budget chain + dig head are
            // NOT the wyvern's machine — the fixed 14-link chain and the
            // altitude-gate flyer live in `npc_ai_wyvern.rs` (UNWIRED).
            NpcKind::WyvernHead => (11.0, 0.25),
            _ => (8.0, 0.07),
        }
    }

    /// `NPC.townNPC` — the types `SetDefaults` marks as town NPCs
    /// (`NPC.cs:8979-11943`, the `this.townNPC = true` arms: 17/18/19/20/22/
    /// 38/54/107/227/228 here, and 37/368/441/… with no ported kind). The
    /// world-file section filters on it (`SaveNPCs:1717`, `FixAgainstExploits:
    /// 1881`), and `town::quick_find_home`'s evaluation is the same set.
    pub fn is_town_npc(self) -> bool {
        matches!(
            self,
            NpcKind::Merchant
                | NpcKind::Nurse
                | NpcKind::ArmsDealer
                | NpcKind::Dryad
                | NpcKind::Guide
                | NpcKind::Demolitionist
                | NpcKind::Clothier
                | NpcKind::GoblinTinkerer
                | NpcKind::Painter
                | NpcKind::WitchDoctor
        )
    }

    /// `NPC.isLikeATownNPC` (`NPC.cs:6856-6864`: `townNPC || type == 453` —
    /// the Skeleton Merchant, which the port does not ship): the vanilla
    /// `townNPC` flag for every town kind the port carries — the housing set
    /// of [`NpcKind::is_town_npc`] plus the eleven sellers that have no
    /// ported housing entry yet (TravellingMerchant 368, Angler 369, Stylist
    /// 353, TaxCollector 441, Golfer 588, Zoologist 633, Princess 663,
    /// DyeTrader 207, PartyGirl 208, Wizard 108, Mechanic 124 — all
    /// `this.townNPC = true` in `SetDefaults`, `NPC.cs:9011-17397`).
    /// `ProcessHitAgainstNPC`'s melee exceptions read this set
    /// (`Player.cs:44274`, `:44344`).
    pub fn is_town_npc_like(self) -> bool {
        self.is_town_npc()
            || matches!(
                self,
                NpcKind::TravellingMerchant
                    | NpcKind::Angler
                    | NpcKind::Stylist
                    | NpcKind::TaxCollector
                    | NpcKind::Golfer
                    | NpcKind::Zoologist
                    | NpcKind::Princess
                    | NpcKind::DyeTrader
                    | NpcKind::PartyGirl
                    | NpcKind::Wizard
                    | NpcKind::Mechanic
            )
    }

    /// `NPC.friendly` (`NPC.cs:8519`'s default `false` plus the `SetDefaults`
    /// arms `:8983-9060`, `:10257`, `:10493-10507`, `:13502-13516`,
    /// `:13796-13809`, `:16652-16665`): every town NPC and the six
    /// bound/rescue forms the port ships. `ProcessHitAgainstNPC` refuses
    /// every friendly NPC (`Player.cs:44272-44276`), and
    /// `Damage_PVE_Inner` gates projectiles on the same flag
    /// (`Projectile.cs:12601`).
    pub fn is_friendly(self) -> bool {
        self.is_town_npc_like()
            || matches!(
                self,
                NpcKind::BoundGoblin
                    | NpcKind::BoundWizard
                    | NpcKind::BoundMechanic
                    | NpcKind::WebbedStylist
                    | NpcKind::SleepingAngler
                    | NpcKind::SleepingGolfer
            )
    }

    /// `NPC.netID` as the `.wld` section writes it (`SaveNPCs:1718`): the
    /// vanilla id, which `crate::bestiary::ENTRIES` carries per kind.
    /// **Not** [`NpcKind::to_u16`] — that is the port's internal protocol id
    /// space; the wire needs the real one.
    pub fn net_id(self) -> i32 {
        crate::bestiary::entry_for_npc(self, SlimeVariant::Blue).net_id
    }

    /// The family-scoped defaults above, or `None` outside the 16 kinds.
    pub fn pumpkin_moon_defaults(self) -> Option<PumpkinDefaults> {
        Some(match self {
            // `NPC.cs:12929-12977` — the 305-314 group: the shared row plus
            // the five stat pairs' scale rows.
            NpcKind::Scarecrow1 => PumpkinDefaults {
                vanilla_type: 305,
                ai_style: 3,
                scale: 1.0,
                alpha: 0,
                no_gravity: false,
                no_tile_collide: false,
            },
            NpcKind::Scarecrow2 => PumpkinDefaults {
                vanilla_type: 306,
                ai_style: 3,
                scale: 1.05,
                alpha: 0,
                no_gravity: false,
                no_tile_collide: false,
            },
            NpcKind::Scarecrow3 => PumpkinDefaults {
                vanilla_type: 307,
                ai_style: 3,
                scale: 0.9,
                alpha: 0,
                no_gravity: false,
                no_tile_collide: false,
            },
            NpcKind::Scarecrow4 => PumpkinDefaults {
                vanilla_type: 308,
                ai_style: 3,
                scale: 0.95,
                alpha: 0,
                no_gravity: false,
                no_tile_collide: false,
            },
            NpcKind::Scarecrow5 => PumpkinDefaults {
                vanilla_type: 309,
                ai_style: 3,
                scale: 1.1,
                alpha: 0,
                no_gravity: false,
                no_tile_collide: false,
            },
            NpcKind::Scarecrow6 => PumpkinDefaults {
                vanilla_type: 310,
                ai_style: 3,
                scale: 1.0,
                alpha: 0,
                no_gravity: false,
                no_tile_collide: false,
            },
            NpcKind::Scarecrow7 => PumpkinDefaults {
                vanilla_type: 311,
                ai_style: 3,
                scale: 1.05,
                alpha: 0,
                no_gravity: false,
                no_tile_collide: false,
            },
            NpcKind::Scarecrow8 => PumpkinDefaults {
                vanilla_type: 312,
                ai_style: 3,
                scale: 0.9,
                alpha: 0,
                no_gravity: false,
                no_tile_collide: false,
            },
            NpcKind::Scarecrow9 => PumpkinDefaults {
                vanilla_type: 313,
                ai_style: 3,
                scale: 0.95,
                alpha: 0,
                no_gravity: false,
                no_tile_collide: false,
            },
            NpcKind::Scarecrow10 => PumpkinDefaults {
                vanilla_type: 314,
                ai_style: 3,
                scale: 1.1,
                alpha: 0,
                no_gravity: false,
                no_tile_collide: false,
            },
            // `NPC.cs:12978-12996` (315), `:13115-13128` (325), `:13129-13141`
            // (326), `:13142-13174` (327), `:13175-13187` (329),
            // `:13188-13203` (330).
            NpcKind::HeadlessHorseman => PumpkinDefaults {
                vanilla_type: 315,
                ai_style: 26,
                scale: 1.0,
                alpha: 0,
                no_gravity: false,
                no_tile_collide: false,
            },
            NpcKind::MourningWood => PumpkinDefaults {
                vanilla_type: 325,
                ai_style: 57,
                scale: 1.0,
                alpha: 0,
                no_gravity: false,
                no_tile_collide: false,
            },
            NpcKind::Splinterling => PumpkinDefaults {
                vanilla_type: 326,
                ai_style: 3,
                scale: 1.0,
                alpha: 0,
                no_gravity: false,
                no_tile_collide: false,
            },
            NpcKind::Pumpking => PumpkinDefaults {
                vanilla_type: 327,
                ai_style: 58,
                scale: 1.0,
                alpha: 0,
                no_gravity: true,
                no_tile_collide: true,
            },
            NpcKind::Hellhound => PumpkinDefaults {
                vanilla_type: 329,
                ai_style: 26,
                scale: 1.0,
                alpha: 0,
                no_gravity: false,
                no_tile_collide: false,
            },
            NpcKind::Poltergeist => PumpkinDefaults {
                vanilla_type: 330,
                ai_style: 22,
                scale: 1.0,
                alpha: 100,
                no_gravity: true,
                no_tile_collide: true,
            },
            _ => return None,
        })
    }

    /// The kind→type map — [`PumpkinDefaults::vanilla_type`] as the named seam
    /// Phase 2's spawn wiring and the kill walk read. `None` outside the
    /// family.
    pub fn pumpkin_moon_type(self) -> Option<u16> {
        self.pumpkin_moon_defaults().map(|d| d.vanilla_type)
    }

    /// The Frost Moon family-scoped defaults above (`S1-F-13c` slice A), or
    /// `None` outside the 7 kinds.
    pub fn frost_moon_defaults(self) -> Option<FrostDefaults> {
        Some(match self {
            // `NPC.cs:13281-13313` — the 338-340 group: the shared row
            // (18x40, aiStyle 3, NPCHit1/NPCDeath1, value 1200, npcSlots
            // 0.5) plus the three stat rows; 339 and 340 carry scale rows.
            NpcKind::ZombieElf => FrostDefaults {
                vanilla_type: 338,
                ai_style: 3,
                scale: 1.0,
                alpha: 0,
                no_gravity: false,
                no_tile_collide: false,
            },
            NpcKind::ZombieElfBeard => FrostDefaults {
                vanilla_type: 339,
                ai_style: 3,
                scale: 1.05,
                alpha: 0,
                no_gravity: false,
                no_tile_collide: false,
            },
            NpcKind::ZombieElfGirl => FrostDefaults {
                vanilla_type: 340,
                ai_style: 3,
                scale: 0.9,
                alpha: 0,
                no_gravity: false,
                no_tile_collide: false,
            },
            // `NPC.cs:13314-13326` (341), `:13327-13339` (342),
            // `:13340-13354` (343), `:13355-13368` (344).
            NpcKind::PresentMimic => FrostDefaults {
                vanilla_type: 341,
                ai_style: 25,
                scale: 1.0,
                alpha: 0,
                no_gravity: false,
                no_tile_collide: false,
            },
            NpcKind::GingerbreadMan => FrostDefaults {
                vanilla_type: 342,
                ai_style: 3,
                scale: 1.0,
                alpha: 0,
                no_gravity: false,
                no_tile_collide: false,
            },
            NpcKind::Yeti => FrostDefaults {
                vanilla_type: 343,
                ai_style: 3,
                scale: 1.0,
                alpha: 0,
                no_gravity: false,
                no_tile_collide: false,
            },
            NpcKind::Everscream => FrostDefaults {
                vanilla_type: 344,
                ai_style: 57,
                scale: 1.0,
                alpha: 0,
                no_gravity: false,
                no_tile_collide: false,
            },
            // ---- slice B (S1-F-13c, kinds 345-352): NPC.cs:13369-13487.
            NpcKind::IceQueen => FrostDefaults {
                vanilla_type: 345,
                ai_style: 60,
                scale: 1.0,
                alpha: 0,
                no_gravity: true,
                no_tile_collide: true,
            },
            // `NPC.cs:13386-13399` — the Santa-NK1: SetDefaults writes neither
            // flag; the AI itself sets noGravity + noTileCollide every tick
            // (`:33684-33685`).
            NpcKind::SantaNK1 => FrostDefaults {
                vanilla_type: 346,
                ai_style: 61,
                scale: 1.0,
                alpha: 0,
                no_gravity: false,
                no_tile_collide: false,
            },
            // `NPC.cs:13400-13415` — the Elf Copter (`:13412-13413`).
            NpcKind::ElfCopter => FrostDefaults {
                vanilla_type: 347,
                ai_style: 62,
                scale: 1.0,
                alpha: 0,
                no_gravity: true,
                no_tile_collide: true,
            },
            // `NPC.cs:13416-13429` / `:13430-13443` / `:13444-13456` /
            // `:13457-13470` — the four aiStyle-3 walkers: no flag rows.
            NpcKind::Nutcracker => FrostDefaults {
                vanilla_type: 348,
                ai_style: 3,
                scale: 1.0,
                alpha: 0,
                no_gravity: false,
                no_tile_collide: false,
            },
            NpcKind::NutcrackerSpinning => FrostDefaults {
                vanilla_type: 349,
                ai_style: 3,
                scale: 1.0,
                alpha: 0,
                no_gravity: false,
                no_tile_collide: false,
            },
            NpcKind::ElfArcher => FrostDefaults {
                vanilla_type: 350,
                ai_style: 3,
                scale: 1.0,
                alpha: 0,
                no_gravity: false,
                no_tile_collide: false,
            },
            NpcKind::Krampus => FrostDefaults {
                vanilla_type: 351,
                ai_style: 3,
                scale: 1.0,
                alpha: 0,
                no_gravity: false,
                no_tile_collide: false,
            },
            // `NPC.cs:13471-13487` — Flocko (`:13483-13484`).
            NpcKind::Flocko => FrostDefaults {
                vanilla_type: 352,
                ai_style: 63,
                scale: 1.0,
                alpha: 0,
                no_gravity: true,
                no_tile_collide: true,
            },
            _ => return None,
        })
    }

    /// The kind→type map — [`FrostDefaults::vanilla_type`] as the named seam
    /// the frost spawn wiring and the kill walk read. `None` outside the
    /// family.
    pub fn frost_moon_type(self) -> Option<u16> {
        self.frost_moon_defaults().map(|d| d.vanilla_type)
    }

    /// The wave roster's vanilla `NPC.type` → the ported kind — the inverse of
    /// [`NpcKind::pumpkin_moon_type`]/[`NpcKind::frost_moon_type`] the F-13a
    /// moon ladders' picks decode through (`S1-F-13b` Phase 2's spawn wiring,
    /// extended by `S1-F-13c` slices A+B). `None` for every id outside the two
    /// families: the pumpkin
    /// wave-20 unported bosses and every port-local id consume the pick and
    /// spawn nothing.
    pub fn moon_wave_kind(vanilla_type: u16) -> Option<NpcKind> {
        NpcKind::from_u16(vanilla_type).filter(|kind| {
            kind.pumpkin_moon_type() == Some(vanilla_type)
                || kind.frost_moon_type() == Some(vanilla_type)
        })
    }

    /// `NPC.HitSound` (`NPC.cs:6475`, the per-kind `SetDefaults` assignment)
    /// — the cue the strike plays (`NPC.StrikeNPC`, `:81544-81553`).
    ///
    /// `None` is the C#'s `null` (`NPC.cs:8537`): the kinds with no row in
    /// `SetDefaults` play no hit sound. The cue names are the legacy asset
    /// names — the sound id 3 family is `NPC_Hit_N` (`SoundID.NPCHitN`).
    pub fn hit_sound(self) -> Option<&'static str> {
        Some(match self {
            NpcKind::Slime => "NPC_Hit_1",
            NpcKind::DemonEye => "NPC_Hit_1",
            NpcKind::Zombie => "NPC_Hit_1",
            NpcKind::KingSlime => "NPC_Hit_1",
            NpcKind::EyeOfCthulhu => "NPC_Hit_1",
            NpcKind::Hungry => "NPC_Hit_9",
            // 117-119 share NPCHit9 (`NPC.cs:10414`/`:10430`/`:10446`).
            NpcKind::LeechHead | NpcKind::LeechBody | NpcKind::LeechTail => "NPC_Hit_9",
            NpcKind::ServantOfCthulhu => "NPC_Hit_1",
            NpcKind::WanderingEye => "NPC_Hit_1",
            NpcKind::Guide => "NPC_Hit_1",
            NpcKind::Skeleton => "NPC_Hit_2",
            NpcKind::UndeadMiner => "NPC_Hit_2",
            NpcKind::CaveBat => "NPC_Hit_1",
            NpcKind::JungleBat => "NPC_Hit_1",
            NpcKind::Hellbat => "NPC_Hit_1",
            NpcKind::VoodooDemon => "NPC_Hit_21",
            NpcKind::EaterOfSouls => "NPC_Hit_1",
            NpcKind::Hornet => "NPC_Hit_1",
            NpcKind::MeteorHead => "NPC_Hit_3",
            // The volley bees: `SoundID.NPCHit1` (`NPC.cs:11704`/`:11717`).
            NpcKind::Bee | NpcKind::BeeSmall => "NPC_Hit_1",
            NpcKind::GoblinSorcerer => "NPC_Hit_1",
            NpcKind::Tim => "NPC_Hit_2",
            NpcKind::DarkCaster => "NPC_Hit_2",
            NpcKind::FireImp => "NPC_Hit_1",
            // DesertDjinn: `SoundID.NPCHit1` (`NPC.cs:14982`).
            NpcKind::DesertDjinn => "NPC_Hit_1",
            NpcKind::ChaosBall => "NPC_Hit_3",
            NpcKind::BurningSphere => "NPC_Hit_3",
            NpcKind::WaterSphere => "NPC_Hit_3",
            NpcKind::Goldfish => "NPC_Hit_1",
            NpcKind::Piranha => "NPC_Hit_1",
            NpcKind::Shark => "NPC_Hit_1",
            NpcKind::DevourerHead => "NPC_Hit_1",
            NpcKind::GiantWormHead => "NPC_Hit_1",
            NpcKind::Merchant => "NPC_Hit_1",
            NpcKind::Dryad => "NPC_Hit_1",
            NpcKind::Nurse => "NPC_Hit_1",
            NpcKind::ArmsDealer => "NPC_Hit_1",
            NpcKind::Demolitionist => "NPC_Hit_1",
            NpcKind::Painter => "NPC_Hit_1",
            NpcKind::WitchDoctor => "NPC_Hit_1",
            NpcKind::Clothier => "NPC_Hit_1",
            NpcKind::TravellingMerchant => "NPC_Hit_1",
            NpcKind::Angler => "NPC_Hit_1",
            NpcKind::Wizard => "NPC_Hit_1",
            NpcKind::Mechanic => "NPC_Hit_1",
            NpcKind::Stylist => "NPC_Hit_1",
            NpcKind::TaxCollector => "NPC_Hit_1",
            NpcKind::Golfer => "NPC_Hit_1",
            NpcKind::Zoologist => "NPC_Hit_1",
            NpcKind::Princess => "NPC_Hit_1",
            NpcKind::DyeTrader => "NPC_Hit_1",
            NpcKind::PartyGirl => "NPC_Hit_1",
            NpcKind::QueenBee => "NPC_Hit_1",
            NpcKind::SkeletronHead => "NPC_Hit_2",
            NpcKind::WallOfFlesh => "NPC_Hit_8",
            NpcKind::TheDestroyer => "NPC_Hit_4",
            // 135/136/139 (`NPC.cs:10689`/`:10709`/`:10729`): NPCHit4.
            NpcKind::TheDestroyerBody | NpcKind::TheDestroyerTail | NpcKind::Probe => "NPC_Hit_4",
            NpcKind::SkeletronPrime => "NPC_Hit_4",
            NpcKind::Retinazer => "NPC_Hit_1",
            NpcKind::Spazmatism => "NPC_Hit_1",
            NpcKind::Plantera => "NPC_Hit_1",
            // The Plantera minions: `SoundID.NPCHit1` on all four
            // (`NPC.cs:12372`/`:12407`/`:12418`/`:12431`).
            NpcKind::FungiSpore | NpcKind::PlanterasHook | NpcKind::PlanterasTentacle | NpcKind::Spore => "NPC_Hit_1",
            NpcKind::BrainOfCthulhu => "NPC_Hit_9",
            NpcKind::Creeper => "NPC_Hit_9",
            NpcKind::GoblinThief => "NPC_Hit_1",
            NpcKind::GoblinWarrior => "NPC_Hit_1",
            NpcKind::GoblinArcher => "NPC_Hit_1",
            NpcKind::GoblinPeon => "NPC_Hit_1",
            NpcKind::GoblinSummoner => "NPC_Hit_40",
            NpcKind::SnowmanGangsta => "NPC_Hit_11",
            NpcKind::SnowBalla => "NPC_Hit_11",
            NpcKind::MysterySnowman => "NPC_Hit_11",
            NpcKind::MartianOfficer => "NPC_Hit_39",
            NpcKind::GrayGrunt => "NPC_Hit_39",
            NpcKind::MartianEngineer => "NPC_Hit_39",
            NpcKind::GigaZapper => "NPC_Hit_39",
            NpcKind::MartianWalker => "NPC_Hit_4",
            NpcKind::MartianProbe => "NPC_Hit_4",
            NpcKind::MartianDrone => "NPC_Hit_42",
            // The bubble: NPCHit43 (`NPC.cs:13889`); the turret: NPCHit53
            // (`NPC.cs:13930`).
            NpcKind::ForceBubble => "NPC_Hit_43",
            NpcKind::MartianTurret => "NPC_Hit_53",
            NpcKind::ScutlixRider => "NPC_Hit_39",
            NpcKind::Scutlix => "NPC_Hit_51",
            NpcKind::MartianSaucer => "NPC_Hit_4",
            NpcKind::MartianSaucerTurret => "NPC_Hit_4",
            NpcKind::MartianSaucerCannon => "NPC_Hit_4",
            NpcKind::MartianSaucerCore => "NPC_Hit_4",
            NpcKind::FlyingDutchman => "NPC_Hit_19",
            NpcKind::FlyingDutchmanCannon => "NPC_Hit_4",
            NpcKind::DD2GoblinT1 => "dd2_goblin_hurt_0",
            NpcKind::DD2GoblinT2 => "dd2_goblin_hurt_0",
            NpcKind::DD2GoblinT3 => "dd2_goblin_hurt_0",
            NpcKind::DD2GoblinBomberT1 => "dd2_goblin_bomber_hurt_0",
            NpcKind::DD2GoblinBomberT2 => "dd2_goblin_bomber_hurt_0",
            NpcKind::DD2GoblinBomberT3 => "dd2_goblin_bomber_hurt_0",
            NpcKind::DD2JavelinT1 => "dd2_javelin_throwers_hurt_0",
            NpcKind::DD2JavelinT2 => "dd2_javelin_throwers_hurt_0",
            NpcKind::DD2JavelinT3 => "dd2_javelin_throwers_hurt_0",
            NpcKind::DD2SkeletonT1 => "dd2_skeleton_hurt_0",
            NpcKind::DD2SkeletonT3 => "dd2_skeleton_hurt_0",
            NpcKind::DD2DarkMage => "dd2_dark_mage_hurt_0",
            NpcKind::DD2Betsy => "dd2_betsy_hurt_0",
            NpcKind::DD2WitherBeast => "dd2_wither_beast_hurt_0",
            NpcKind::DD2Bartender => "NPC_Hit_1",
            NpcKind::DD2EterniaCrystal => "dd2_crystal_cart_impact_0",
            NpcKind::DD2LanePortal => "NPC_Hit_1",
            NpcKind::DD2WyvernT1 => "dd2_wyvern_hurt_0",
            NpcKind::DD2WyvernT2 => "dd2_wyvern_hurt_0",
            NpcKind::DD2WyvernT3 => "dd2_wyvern_hurt_0",
            NpcKind::DD2DarkMageT3 => "dd2_dark_mage_hurt_0",
            NpcKind::DD2WitherBeastT3 => "dd2_wither_beast_hurt_0",
            NpcKind::DD2DrakinT2 => "dd2_drakin_hurt_0",
            NpcKind::DD2DrakinT3 => "dd2_drakin_hurt_0",
            NpcKind::DD2KoboldWalkerT2 => "dd2_kobold_hurt_0",
            NpcKind::DD2KoboldWalkerT3 => "dd2_kobold_hurt_0",
            NpcKind::DD2KoboldFlyerT2 => "dd2_kobold_flyer_hurt_0",
            NpcKind::DD2KoboldFlyerT3 => "dd2_kobold_flyer_hurt_0",
            NpcKind::DD2OgreT2 => "dd2_ogre_hurt_0",
            NpcKind::DD2OgreT3 => "dd2_ogre_hurt_0",
            NpcKind::AncientCultist => "NPC_Hit_55",
            NpcKind::StardustTower => "NPC_Hit_1",
            NpcKind::NebulaTower => "NPC_Hit_1",
            NpcKind::SolarTower => "NPC_Hit_1",
            NpcKind::VortexTower => "NPC_Hit_1",
            NpcKind::StardustCellBig => "NPC_Hit_1",
            NpcKind::StardustCellSmall => "NPC_Hit_1",
            NpcKind::StardustJellyfishBig => "NPC_Hit_1",
            NpcKind::NebulaBeast => "NPC_Hit_6",
            NpcKind::VortexRifleman => "NPC_Hit_1",
            NpcKind::VortexSoldier => "NPC_Hit_1",
            NpcKind::VortexHunter => "NPC_Hit_1",
            NpcKind::VortexAlien => "NPC_Hit_1",
            NpcKind::NebulaFloater => "NPC_Hit_1",
            NpcKind::BrainSuckler => "NPC_Hit_1",
            NpcKind::StardustSpiderBig => "NPC_Hit_6",
            NpcKind::StardustSpiderSmall => "NPC_Hit_1",
            NpcKind::StardustSoldier => "NPC_Hit_6",
            NpcKind::SolarDrakomire => "NPC_Hit_1",
            NpcKind::SolarDrakomireRider => "NPC_Hit_1",
            NpcKind::SolarSroller => "NPC_Hit_7",
            NpcKind::SolarCorite => "NPC_Hit_3",
            NpcKind::SolarCrawltipedeHead => "NPC_Hit_11",
            NpcKind::SolarCrawltipedeBody => "NPC_Hit_11",
            NpcKind::SolarCrawltipedeTail => "NPC_Hit_8",
            NpcKind::StardustWormHead => "NPC_Hit_1",
            NpcKind::SolarSpearman => "NPC_Hit_1",
            NpcKind::SolarSelenian => "NPC_Hit_1",
            NpcKind::Predictor => "NPC_Hit_1",
            NpcKind::VortexHeadcrab => "NPC_Hit_1",
            NpcKind::MoonLordHead => "NPC_Hit_57",
            NpcKind::MoonLordHand => "NPC_Hit_57",
            NpcKind::MoonLordCore => "NPC_Hit_57",
            NpcKind::MoonLordFreeEye => "NPC_Hit_1",
            NpcKind::MoonLordLeechBlob => "NPC_Hit_57",
            NpcKind::PirateDeckhand => "NPC_Hit_1",
            NpcKind::PirateCaptainsMate => "NPC_Hit_1",
            NpcKind::PirateArcher => "NPC_Hit_1",
            NpcKind::PirateCorsair => "NPC_Hit_1",
            NpcKind::PirateCaptain => "NPC_Hit_1",
            NpcKind::Golem => "NPC_Hit_4",
            NpcKind::PrimeCannon => "NPC_Hit_4",
            NpcKind::PrimeSaw => "NPC_Hit_4",
            NpcKind::PrimeVice => "NPC_Hit_4",
            NpcKind::PrimeLaser => "NPC_Hit_4",
            NpcKind::WallOfFleshEye => "NPC_Hit_8",
            NpcKind::SkeletronHand => "NPC_Hit_2",
            NpcKind::Deerclops => "deerclops_hit_0",
            NpcKind::GoblinTinkerer => "NPC_Hit_1",
            NpcKind::EaterOfWorldsHead => "NPC_Hit_1",
            NpcKind::DevourerBody => "NPC_Hit_1",
            NpcKind::DevourerTail => "NPC_Hit_1",
            NpcKind::GiantWormBody => "NPC_Hit_1",
            NpcKind::GiantWormTail => "NPC_Hit_1",
            NpcKind::EaterOfWorldsBody => "NPC_Hit_1",
            NpcKind::EaterOfWorldsTail => "NPC_Hit_1",
            NpcKind::GolemHead => "NPC_Hit_4",
            NpcKind::GolemFistLeft => "NPC_Hit_4",
            NpcKind::GolemHeadFree => "NPC_Hit_4",
            NpcKind::BoundGoblin => "NPC_Hit_1",
            NpcKind::BoundWizard => "NPC_Hit_1",
            NpcKind::BoundMechanic => "NPC_Hit_1",
            NpcKind::SleepingAngler => "NPC_Hit_1",
            NpcKind::WebbedStylist => "NPC_Hit_1",
            NpcKind::SleepingGolfer => "NPC_Hit_1",
            // The Pumpkin Moon family (`S1-F-13b`): the 305-314 group's
            // `NPCHit1` (`NPC.cs:12929-12977`), 315's `NPCHit12` (`:12978`),
            // 326's `NPCHit7` (`:13129`), 327's `NPCHit1` (`:13142`), 325's
            // `NPCHit7` (`:13115`), 329's `NPCHit1` (`:13175`), 330's
            // `NPCHit1` (`:13188`).
            NpcKind::Scarecrow1
            | NpcKind::Scarecrow2
            | NpcKind::Scarecrow3
            | NpcKind::Scarecrow4
            | NpcKind::Scarecrow5
            | NpcKind::Scarecrow6
            | NpcKind::Scarecrow7
            | NpcKind::Scarecrow8
            | NpcKind::Scarecrow9
            | NpcKind::Scarecrow10 => "NPC_Hit_1",
            NpcKind::HeadlessHorseman => "NPC_Hit_12",
            NpcKind::MourningWood => "NPC_Hit_7",
            NpcKind::Splinterling => "NPC_Hit_7",
            NpcKind::Pumpking => "NPC_Hit_1",
            NpcKind::Hellhound => "NPC_Hit_1",
            NpcKind::Poltergeist => "NPC_Hit_1",
            // S1-D-3ff wave A — every kind's `HitSound = SoundID.NPCHit1`
            // (the `SetDefaults` rows cited in `npc.rs`'s enum).
            NpcKind::MotherSlime
            | NpcKind::LavaSlime
            | NpcKind::DungeonSlime
            | NpcKind::CorruptSlime
            | NpcKind::IlluminantSlime
            | NpcKind::IceSlime
            | NpcKind::Crimslime
            | NpcKind::SpikedIceSlime
            | NpcKind::UmbrellaSlime
            | NpcKind::CataractEye
            | NpcKind::SleepyEye
            | NpcKind::DialatedEye
            | NpcKind::GreenEye
            | NpcKind::PurpleEye
            | NpcKind::DoctorBones
            | NpcKind::TheGroom
            | NpcKind::BaldZombie
            | NpcKind::ZombieEskimo
            | NpcKind::PincushionZombie
            | NpcKind::SlimedZombie
            | NpcKind::SwampZombie
            | NpcKind::TwiggyZombie
            | NpcKind::FemaleZombie
            | NpcKind::ZombieRaincoat
            // S1-D-3z — the holiday zombies' `NPCHit1` (`NPC.cs:13041`/
            // `:13055`/`:13069`/`:13212`/`:13225`).
            | NpcKind::ZombieDoctor
            | NpcKind::ZombieSuperman
            | NpcKind::ZombiePixie
            | NpcKind::ZombieXmas
            | NpcKind::ZombieSweater => "NPC_Hit_1",
            // S1-D-3z — the blood-moon pair's `SetDefaults` HitSound rows:
            // 489 NPCHit18 (`NPC.cs:14613`), 490 NPCHit19 (`:14627`).
            NpcKind::BloodZombie => "NPC_Hit_18",
            NpcKind::Drippler => "NPC_Hit_19",
            // The Frost Moon family, slice A (`S1-F-13c`): the 338-340 group
            // and the Yeti's `NPCHit1` (`NPC.cs:13286`/`:13348`), 341's
            // `NPCHit4` (`:13322`), 342 and 344's `NPCHit7`
            // (`:13335`/`:13363`).
            NpcKind::ZombieElf
            | NpcKind::ZombieElfBeard
            | NpcKind::ZombieElfGirl
            | NpcKind::Yeti => "NPC_Hit_1",
            NpcKind::PresentMimic => "NPC_Hit_4",
            NpcKind::GingerbreadMan | NpcKind::Everscream => "NPC_Hit_7",
            // The Frost Moon family (`S1-F-13c` slice B): 345's `NPCHit7`
            // (`NPC.cs:13377`), 346/347's `NPCHit4` (`:13394`/`:13408`),
            // the four walkers and Flocko's `NPCHit1` (`:13424`/`:13438`/
            // `:13452`/`:13465`/`:13479`).
            NpcKind::IceQueen => "NPC_Hit_7",
            NpcKind::SantaNK1 | NpcKind::ElfCopter => "NPC_Hit_4",
            NpcKind::Nutcracker
            | NpcKind::NutcrackerSpinning
            | NpcKind::ElfArcher
            | NpcKind::Krampus
            | NpcKind::Flocko => "NPC_Hit_1",
            // S1-D-3 W-9 fishron wiring — 370 `SoundID.NPCHit14`
            // (`NPC.cs:13713`), 371 `NPCHit3` (`:13727`), 372/373
            // `NPCHit1` (`:13742`/`:13756`).
            NpcKind::DukeFishron => "NPC_Hit_14",
            NpcKind::DetonatingBubble => "NPC_Hit_3",
            NpcKind::Sharkron | NpcKind::Sharkron2 => "NPC_Hit_1",
            // The small-kinds wiring wave: 72 NPCHit1 (`NPC.cs:9725`),
            // 250 NPCHit30 (`:12229`), 288 NPCHit36 (`:12747`), 384
            // NPCHit43 (`:13889`). Unregistered bank cues are the
            // wave-A bank-gap skip (`audio.rs::play_npc_hit_sound`).
            NpcKind::BlazingWheel => "NPC_Hit_1",
            NpcKind::AngryNimbus => "NPC_Hit_30",
            NpcKind::DungeonSpirit => "NPC_Hit_36",
            // The wyvern family: `SoundID.NPCHit7` (`NPC.cs:9950`/`:9967`/
            // `:9984`/`:10002`/`:10020`/`:10038`).
            NpcKind::WyvernHead
            | NpcKind::WyvernLegs
            | NpcKind::WyvernBody
            | NpcKind::WyvernBody2
            | NpcKind::WyvernBody3
            | NpcKind::WyvernTail => "NPC_Hit_7",
            _ => return None,
        })
    }

    /// `NPC.DeathSound` (`NPC.cs:6477`) — the cue the death plays
    /// (`NPC.checkDead`, `:91852`). The sound id 4 family is the
    /// `NPC_Killed_N` assets (`SoundID.NPCDeathN`).
    pub fn death_sound(self) -> Option<&'static str> {
        Some(match self {
            NpcKind::Slime => "NPC_Killed_1",
            NpcKind::DemonEye => "NPC_Killed_1",
            NpcKind::Zombie => "NPC_Killed_2",
            NpcKind::KingSlime => "NPC_Killed_1",
            NpcKind::EyeOfCthulhu => "NPC_Killed_1",
            NpcKind::Hungry => "NPC_Killed_11",
            // 117-119 share NPCDeath12 (`NPC.cs:10415`/`:10431`/`:10447`).
            NpcKind::LeechHead | NpcKind::LeechBody | NpcKind::LeechTail => "NPC_Killed_12",
            NpcKind::ServantOfCthulhu => "NPC_Killed_1",
            NpcKind::WanderingEye => "NPC_Killed_1",
            NpcKind::Guide => "NPC_Killed_1",
            NpcKind::Skeleton => "NPC_Killed_2",
            NpcKind::UndeadMiner => "NPC_Killed_2",
            NpcKind::CaveBat => "NPC_Killed_4",
            NpcKind::JungleBat => "NPC_Killed_4",
            NpcKind::Hellbat => "NPC_Killed_4",
            NpcKind::VoodooDemon => "NPC_Killed_24",
            NpcKind::EaterOfSouls => "NPC_Killed_1",
            NpcKind::Hornet => "NPC_Killed_1",
            NpcKind::MeteorHead => "NPC_Killed_3",
            // The volley bees: `SoundID.NPCDeath1` (`NPC.cs:11707`/`:11720`).
            NpcKind::Bee | NpcKind::BeeSmall => "NPC_Killed_1",
            NpcKind::GoblinSorcerer => "NPC_Killed_1",
            NpcKind::Tim => "NPC_Killed_2",
            NpcKind::DarkCaster => "NPC_Killed_2",
            NpcKind::FireImp => "NPC_Killed_1",
            // DesertDjinn: `SoundID.NPCDeath6` (`NPC.cs:14983`).
            NpcKind::DesertDjinn => "NPC_Killed_6",
            NpcKind::ChaosBall => "NPC_Killed_3",
            NpcKind::BurningSphere => "NPC_Killed_3",
            NpcKind::WaterSphere => "NPC_Killed_3",
            NpcKind::Goldfish => "NPC_Killed_1",
            NpcKind::Piranha => "NPC_Killed_1",
            NpcKind::Shark => "NPC_Killed_1",
            NpcKind::DevourerHead => "NPC_Killed_1",
            NpcKind::GiantWormHead => "NPC_Killed_1",
            NpcKind::Merchant => "NPC_Killed_1",
            NpcKind::Dryad => "NPC_Killed_1",
            NpcKind::Nurse => "NPC_Killed_1",
            NpcKind::ArmsDealer => "NPC_Killed_1",
            NpcKind::Demolitionist => "NPC_Killed_1",
            NpcKind::Painter => "NPC_Killed_1",
            NpcKind::WitchDoctor => "NPC_Killed_1",
            NpcKind::Clothier => "NPC_Killed_1",
            NpcKind::TravellingMerchant => "NPC_Killed_1",
            NpcKind::Angler => "NPC_Killed_6",
            NpcKind::Wizard => "NPC_Killed_1",
            NpcKind::Mechanic => "NPC_Killed_1",
            NpcKind::Stylist => "NPC_Killed_1",
            NpcKind::TaxCollector => "NPC_Killed_1",
            NpcKind::Golfer => "NPC_Killed_1",
            NpcKind::Zoologist => "NPC_Killed_1",
            NpcKind::Princess => "NPC_Killed_6",
            NpcKind::DyeTrader => "NPC_Killed_1",
            NpcKind::PartyGirl => "NPC_Killed_1",
            NpcKind::QueenBee => "NPC_Killed_66",
            NpcKind::SkeletronHead => "NPC_Killed_2",
            NpcKind::WallOfFlesh => "NPC_Killed_10",
            NpcKind::TheDestroyer => "NPC_Killed_14",
            // 135/136/139 (`NPC.cs:10690`/`:10710`/`:10730`): NPCDeath14.
            NpcKind::TheDestroyerBody | NpcKind::TheDestroyerTail | NpcKind::Probe => "NPC_Killed_14",
            NpcKind::SkeletronPrime => "NPC_Killed_14",
            NpcKind::Retinazer => "NPC_Killed_14",
            NpcKind::Spazmatism => "NPC_Killed_14",
            NpcKind::Plantera => "NPC_Killed_1",
            // The Plantera minions: `SoundID.NPCDeath1` on all four
            // (`NPC.cs:12373`/`:12408`/`:12419`/`:12432`).
            NpcKind::FungiSpore | NpcKind::PlanterasHook | NpcKind::PlanterasTentacle | NpcKind::Spore => "NPC_Killed_1",
            NpcKind::BrainOfCthulhu => "NPC_Killed_11",
            NpcKind::Creeper => "NPC_Killed_11",
            NpcKind::GoblinThief => "NPC_Killed_1",
            NpcKind::GoblinWarrior => "NPC_Killed_1",
            NpcKind::GoblinArcher => "NPC_Killed_1",
            NpcKind::GoblinPeon => "NPC_Killed_1",
            NpcKind::GoblinSummoner => "NPC_Killed_42",
            NpcKind::SnowmanGangsta => "NPC_Killed_15",
            NpcKind::SnowBalla => "NPC_Killed_15",
            NpcKind::MysterySnowman => "NPC_Killed_15",
            NpcKind::MartianOfficer => "NPC_Killed_57",
            NpcKind::GrayGrunt => "NPC_Killed_57",
            NpcKind::MartianEngineer => "NPC_Killed_57",
            NpcKind::GigaZapper => "NPC_Killed_57",
            NpcKind::MartianWalker => "NPC_Killed_14",
            NpcKind::MartianProbe => "NPC_Killed_14",
            NpcKind::MartianDrone => "NPC_Killed_44",
            // The bubble: NPCDeath45 (`NPC.cs:13890`); the turret:
            // NPCDeath56 (`NPC.cs:13931`).
            NpcKind::ForceBubble => "NPC_Killed_45",
            NpcKind::MartianTurret => "NPC_Killed_56",
            NpcKind::ScutlixRider => "NPC_Killed_57",
            NpcKind::Scutlix => "NPC_Killed_54",
            NpcKind::MartianSaucer => "NPC_Killed_14",
            NpcKind::MartianSaucerTurret => "NPC_Killed_14",
            NpcKind::MartianSaucerCannon => "NPC_Killed_14",
            NpcKind::MartianSaucerCore => "NPC_Killed_14",
            NpcKind::FlyingDutchman => "NPC_Killed_22",
            NpcKind::FlyingDutchmanCannon => "NPC_Killed_14",
            NpcKind::DD2GoblinT1 => "dd2_goblin_death_0",
            NpcKind::DD2GoblinT2 => "dd2_goblin_death_0",
            NpcKind::DD2GoblinT3 => "dd2_goblin_death_0",
            NpcKind::DD2GoblinBomberT1 => "dd2_goblin_bomber_death_0",
            NpcKind::DD2GoblinBomberT2 => "dd2_goblin_bomber_death_0",
            NpcKind::DD2GoblinBomberT3 => "dd2_goblin_bomber_death_0",
            NpcKind::DD2JavelinT1 => "dd2_javelin_throwers_death_0",
            NpcKind::DD2JavelinT2 => "dd2_javelin_throwers_death_0",
            NpcKind::DD2JavelinT3 => "dd2_javelin_throwers_death_0",
            NpcKind::DD2SkeletonT1 => "dd2_skeleton_death_0",
            NpcKind::DD2SkeletonT3 => "dd2_skeleton_death_0",
            NpcKind::DD2DarkMage => "dd2_dark_mage_death_0",
            NpcKind::DD2Betsy => "dd2_betsy_death_0",
            NpcKind::DD2WitherBeast => "dd2_wither_beast_death_0",
            NpcKind::DD2Bartender => "NPC_Killed_1",
            NpcKind::DD2EterniaCrystal => "NPC_Killed_15",
            NpcKind::DD2LanePortal => "NPC_Killed_1",
            NpcKind::DD2WyvernT1 => "dd2_wyvern_death_0",
            NpcKind::DD2WyvernT2 => "dd2_wyvern_death_0",
            NpcKind::DD2WyvernT3 => "dd2_wyvern_death_0",
            NpcKind::DD2DarkMageT3 => "dd2_dark_mage_death_0",
            NpcKind::DD2WitherBeastT3 => "dd2_wither_beast_death_0",
            NpcKind::DD2DrakinT2 => "dd2_drakin_death_0",
            NpcKind::DD2DrakinT3 => "dd2_drakin_death_0",
            NpcKind::DD2KoboldWalkerT2 => "dd2_kobold_death_0",
            NpcKind::DD2KoboldWalkerT3 => "dd2_kobold_death_0",
            NpcKind::DD2KoboldFlyerT2 => "dd2_kobold_flyer_death_0",
            NpcKind::DD2KoboldFlyerT3 => "dd2_kobold_flyer_death_0",
            NpcKind::DD2OgreT2 => "dd2_ogre_death_0",
            NpcKind::DD2OgreT3 => "dd2_ogre_death_0",
            NpcKind::AncientCultist => "NPC_Killed_59",
            NpcKind::StardustTower => "NPC_Killed_1",
            NpcKind::NebulaTower => "NPC_Killed_1",
            NpcKind::SolarTower => "NPC_Killed_1",
            NpcKind::VortexTower => "NPC_Killed_1",
            NpcKind::StardustCellBig => "NPC_Killed_1",
            NpcKind::StardustCellSmall => "NPC_Killed_1",
            NpcKind::StardustJellyfishBig => "NPC_Killed_1",
            NpcKind::NebulaBeast => "NPC_Killed_1",
            NpcKind::VortexRifleman => "NPC_Killed_6",
            NpcKind::VortexSoldier => "NPC_Killed_5",
            NpcKind::VortexHunter => "NPC_Killed_6",
            NpcKind::VortexAlien => "NPC_Killed_6",
            NpcKind::NebulaFloater => "NPC_Killed_1",
            NpcKind::BrainSuckler => "NPC_Killed_1",
            NpcKind::StardustSpiderBig => "NPC_Killed_1",
            NpcKind::StardustSpiderSmall => "NPC_Killed_5",
            NpcKind::StardustSoldier => "NPC_Killed_1",
            NpcKind::SolarDrakomire => "NPC_Killed_1",
            NpcKind::SolarDrakomireRider => "NPC_Killed_1",
            NpcKind::SolarSroller => "NPC_Killed_5",
            NpcKind::SolarCorite => "NPC_Killed_3",
            NpcKind::SolarCrawltipedeHead => "NPC_Killed_22",
            NpcKind::SolarCrawltipedeBody => "NPC_Killed_22",
            NpcKind::SolarCrawltipedeTail => "NPC_Killed_22",
            NpcKind::StardustWormHead => "NPC_Killed_1",
            NpcKind::SolarSpearman => "NPC_Killed_1",
            NpcKind::SolarSelenian => "NPC_Killed_1",
            NpcKind::Predictor => "NPC_Killed_5",
            NpcKind::VortexHeadcrab => "NPC_Killed_1",
            NpcKind::MoonLordHead => "NPC_Killed_62",
            NpcKind::MoonLordHand => "NPC_Killed_62",
            NpcKind::MoonLordFreeEye => "NPC_Killed_1",
            NpcKind::MoonLordLeechBlob => "NPC_Killed_62",
            NpcKind::PirateDeckhand => "NPC_Killed_1",
            NpcKind::PirateCaptainsMate => "NPC_Killed_1",
            NpcKind::PirateArcher => "NPC_Killed_1",
            NpcKind::PirateCorsair => "NPC_Killed_1",
            NpcKind::PirateCaptain => "NPC_Killed_1",
            NpcKind::Golem => "NPC_Killed_14",
            NpcKind::PrimeCannon => "NPC_Killed_14",
            NpcKind::PrimeSaw => "NPC_Killed_14",
            NpcKind::PrimeVice => "NPC_Killed_14",
            NpcKind::PrimeLaser => "NPC_Killed_14",
            NpcKind::WallOfFleshEye => "NPC_Killed_10",
            NpcKind::SkeletronHand => "NPC_Killed_2",
            NpcKind::Deerclops => "deerclops_death",
            NpcKind::GoblinTinkerer => "NPC_Killed_1",
            NpcKind::EaterOfWorldsHead => "NPC_Killed_1",
            NpcKind::DevourerBody => "NPC_Killed_1",
            NpcKind::DevourerTail => "NPC_Killed_1",
            NpcKind::GiantWormBody => "NPC_Killed_1",
            NpcKind::GiantWormTail => "NPC_Killed_1",
            NpcKind::EaterOfWorldsBody => "NPC_Killed_1",
            NpcKind::EaterOfWorldsTail => "NPC_Killed_1",
            NpcKind::GolemFistLeft => "NPC_Killed_14",
            NpcKind::GolemHeadFree => "NPC_Killed_14",
            NpcKind::BoundGoblin => "NPC_Killed_1",
            NpcKind::BoundWizard => "NPC_Killed_1",
            NpcKind::BoundMechanic => "NPC_Killed_1",
            NpcKind::SleepingAngler => "NPC_Killed_6",
            NpcKind::WebbedStylist => "NPC_Killed_1",
            NpcKind::SleepingGolfer => "NPC_Killed_1",
            // The Pumpkin Moon family (`S1-F-13b`): the 305-314 group's
            // `NPCDeath6` (`NPC.cs:12929-12977`), 315's `NPCDeath18`
            // (`:12978`), 326's `NPCDeath6` (`:13129`), 327's `NPCDeath1`
            // (`:13142`), 325's `NPCDeath5` (`:13115`), 329's `NPCDeath5`
            // (`:13175`), 330's `NPCDeath6` (`:13188`).
            NpcKind::Scarecrow1
            | NpcKind::Scarecrow2
            | NpcKind::Scarecrow3
            | NpcKind::Scarecrow4
            | NpcKind::Scarecrow5
            | NpcKind::Scarecrow6
            | NpcKind::Scarecrow7
            | NpcKind::Scarecrow8
            | NpcKind::Scarecrow9
            | NpcKind::Scarecrow10 => "NPC_Killed_6",
            NpcKind::HeadlessHorseman => "NPC_Killed_18",
            NpcKind::MourningWood => "NPC_Killed_5",
            NpcKind::Splinterling => "NPC_Killed_6",
            NpcKind::Pumpking => "NPC_Killed_1",
            NpcKind::Hellhound => "NPC_Killed_5",
            NpcKind::Poltergeist => "NPC_Killed_6",
            // S1-D-3ff wave A — the `DeathSound` rows: the slimes and
            // eyes NPCDeath1 (`NPC.cs:8973` etc.), the Illuminant Slime
            // NPCDeath6 (`:10757`), the ten walkers NPCDeath2
            // (`:9427`/`:9441`/`:10642`/`:11026`/`:11384`/`:11397`/
            // `:11410` region/`:11423` region/`:11568` region/`:11881`
            // region).
            NpcKind::MotherSlime
            | NpcKind::LavaSlime
            | NpcKind::DungeonSlime
            | NpcKind::CorruptSlime
            | NpcKind::IceSlime
            | NpcKind::Crimslime
            | NpcKind::SpikedIceSlime
            | NpcKind::UmbrellaSlime
            | NpcKind::CataractEye
            | NpcKind::SleepyEye
            | NpcKind::DialatedEye
            | NpcKind::GreenEye
            | NpcKind::PurpleEye => "NPC_Killed_1",
            NpcKind::IlluminantSlime => "NPC_Killed_6",
            NpcKind::DoctorBones
            | NpcKind::TheGroom
            | NpcKind::BaldZombie
            | NpcKind::ZombieEskimo
            | NpcKind::PincushionZombie
            | NpcKind::SlimedZombie
            | NpcKind::SwampZombie
            | NpcKind::TwiggyZombie
            | NpcKind::FemaleZombie
            | NpcKind::ZombieRaincoat
            // S1-D-3z — the holiday zombies' `NPCDeath2` (`NPC.cs:13042`/
            // `:13056`/`:13070`/`:13213`/`:13226`).
            | NpcKind::ZombieDoctor
            | NpcKind::ZombieSuperman
            | NpcKind::ZombiePixie
            | NpcKind::ZombieXmas
            | NpcKind::ZombieSweater => "NPC_Killed_2",
            // S1-D-3z — the blood-moon pair's `SetDefaults` DeathSound
            // rows: 489 NPCDeath21 (`NPC.cs:14614`), 490 NPCDeath22
            // (`:14629`).
            NpcKind::BloodZombie => "NPC_Killed_21",
            NpcKind::Drippler => "NPC_Killed_22",
            // The Frost Moon family, slice A (`S1-F-13c`): the 338-340 group
            // and the Yeti's `NPCDeath1` (`NPC.cs:13287`/`:13349`), 341 and
            // 342's `NPCDeath6` (`:13323`/`:13336`), 344's `NPCDeath5`
            // (`:13364`).
            NpcKind::ZombieElf
            | NpcKind::ZombieElfBeard
            | NpcKind::ZombieElfGirl
            | NpcKind::Yeti => "NPC_Killed_1",
            NpcKind::PresentMimic | NpcKind::GingerbreadMan => "NPC_Killed_6",
            NpcKind::Everscream => "NPC_Killed_5",
            // The Frost Moon family (`S1-F-13c` slice B): 345's `NPCDeath5`
            // (`NPC.cs:13378`), 346/347's `NPCDeath14` (`:13395`/`:13410`),
            // the four walkers' `NPCDeath1` (`:13425`/`:13439`/`:13453`/
            // `:13466`), Flocko's `NPCDeath6` (`:13480`).
            NpcKind::IceQueen => "NPC_Killed_5",
            NpcKind::SantaNK1 | NpcKind::ElfCopter => "NPC_Killed_14",
            NpcKind::Nutcracker
            | NpcKind::NutcrackerSpinning
            | NpcKind::ElfArcher
            | NpcKind::Krampus => "NPC_Killed_1",
            NpcKind::Flocko => "NPC_Killed_6",
            // S1-D-3 W-9 fishron wiring — 370 `SoundID.NPCDeath20`
            // (`NPC.cs:13714`), 371 `NPCDeath3` (`:13728`), 372/373
            // `NPCDeath1` (`:13743`/`:13757`).
            NpcKind::DukeFishron => "NPC_Killed_20",
            NpcKind::DetonatingBubble => "NPC_Killed_3",
            NpcKind::Sharkron | NpcKind::Sharkron2 => "NPC_Killed_1",
            // The small-kinds wiring wave: 72 NPCDeath1 (`NPC.cs:9726`),
            // 250 NPCDeath33 (`:12230`), 288 NPCDeath39 (`:12748`), 384
            // NPCDeath45 (`:13890`). Unregistered bank cues are the
            // wave-A bank-gap skip (`audio.rs::play_npc_death_sound`).
            NpcKind::BlazingWheel => "NPC_Killed_1",
            NpcKind::AngryNimbus => "NPC_Killed_33",
            NpcKind::DungeonSpirit => "NPC_Killed_39",
            // The wyvern family: `SoundID.NPCDeath8` (`NPC.cs:9951`/
            // `:9968`/`:9985`/`:10003`/`:10021`/`:10039`).
            NpcKind::WyvernHead
            | NpcKind::WyvernLegs
            | NpcKind::WyvernBody
            | NpcKind::WyvernBody2
            | NpcKind::WyvernBody3
            | NpcKind::WyvernTail => "NPC_Killed_8",
            _ => return None,
        })
    }

    /// `NPC.noGravity` as `StrikeNPC_Inner`'s knockback reads it
    /// (`NPC.cs:81453`/`:81465`) — the Y-pop's 0.5-vs-0.75 selector and the
    /// weak-hit branch's coefficient. The value is the LIVE flag at strike
    /// time: the `SetDefaults` row plus the AI-head writes. The port models
    /// the AI-head sources that reach a knockback-relevant kind:
    /// `AI_002_FloatingEye` sets it for every eye (`NPC.cs:52280`), the
    /// aiStyle-14 bat block for the bat family + Voodoo Demon
    /// (`NPC.cs:22851`), and the aiStyle-49 head for the Angry Nimbus
    /// (`NPC.cs:31589`). The Pumpkin/Frost tables carry their own rows.
    /// The CONDITIONAL AI writes (the aiStyle-107 DD2 charge state
    /// `:63016`/`:63797`/`:63956`, the Goblin Summoner's ghost `:56497` —
    /// which also zeroes kb) are live state the static table does not model
    /// (ANALYSIS "The player→NPC strike mechanics").
    ///
    /// Only the kinds with `knockBackResist > 0` matter here (the knockback
    /// gate refuses resist 0); the flyers below are the ones a strike can
    /// actually push. The grounded walkers keep the `SetDefaults` false.
    pub fn no_gravity(self) -> bool {
        // The Pumpkin/Frost families carry their own SetDefaults rows.
        if let Some(d) = self.pumpkin_moon_defaults() {
            return d.no_gravity;
        }
        if let Some(d) = self.frost_moon_defaults() {
            return d.no_gravity;
        }
        match self {
            // AI_002_FloatingEye's head write (`NPC.cs:52280`) — every eye.
            NpcKind::DemonEye
            | NpcKind::WanderingEye
            | NpcKind::CataractEye
            | NpcKind::SleepyEye
            | NpcKind::DialatedEye
            | NpcKind::GreenEye
            | NpcKind::PurpleEye => true,
            // The aiStyle-14 bat block (`NPC.cs:22851`) — the bat family and
            // the Voodoo Demon (66, `NPC.cs:9629`).
            NpcKind::CaveBat | NpcKind::JungleBat | NpcKind::Hellbat | NpcKind::VoodooDemon => true,
            // The aiStyle-5 flyers' SetDefaults rows: Eater of Souls (6,
            // `:8744`), Hornet (42, `:9291`), Meteor Head (23, `:9073`),
            // Servant of Cthulhu (5, `:8730`), the volley bees (210/211,
            // `:11707`/`:11720`).
            NpcKind::EaterOfSouls
            | NpcKind::Hornet
            | NpcKind::MeteorHead
            | NpcKind::ServantOfCthulhu
            | NpcKind::Bee
            | NpcKind::BeeSmall => true,
            // The Creeper (267, `NPC.cs:12466`), the Probe (139, `:10731`),
            // the Martian Drone (388, `:13948`), the Drippler (490,
            // `:14620`), the Poltergeist (330, `:13190`), the Dungeon
            // Spirit (288, `:12751`), the Piranha (58, `:9505` — 57 is the
            // CorruptGoldfish) and the Shark (65, `:9612`).
            NpcKind::Creeper
            | NpcKind::Probe
            | NpcKind::MartianDrone
            | NpcKind::Drippler
            | NpcKind::Poltergeist
            | NpcKind::DungeonSpirit
            | NpcKind::Piranha
            | NpcKind::Shark => true,
            // The Hungry (115, `NPC.cs:10388`) — knockBackResist 1.1, so its
            // noGravity matters to the knockback (the leeches are resist 0).
            NpcKind::Hungry => true,
            // W3-i11 census completion — the remaining ported kinds whose
            // `SetDefaults` writes `noGravity = true` AND `knockBackResist > 0`
            // (the Y-pop's 0.5 selector reaches them). The Goldfish (55,
            // `:9462`, kb 0.5 `:9471`), the Brain of Cthulhu (266, `:12450`,
            // kb 0.45 `:12449`), the Plantera's Hook/Tentacle (263/264,
            // `:12399`/`:12420`, no kb row → the class default 1f `:8533`).
            NpcKind::Goldfish
            | NpcKind::BrainOfCthulhu
            | NpcKind::PlanterasHook
            | NpcKind::PlanterasTentacle => true,
            // The Stardust pillars' flyers: the Cell Big/Small (405/406,
            // `:15033`/`:15047`, kb 0.3/0.4), the Jellyfish Big (407,
            // `:15097`, kb 0.03), the Solar Corite (418, `:15378`, kb 0.2),
            // the Nebula Floater (420, `:15169`, kb 0.5) and the Brain
            // Suckler (421, `:15141`, kb 0.5).
            NpcKind::StardustCellBig
            | NpcKind::StardustCellSmall
            | NpcKind::StardustJellyfishBig
            | NpcKind::SolarCorite
            | NpcKind::NebulaFloater
            | NpcKind::BrainSuckler => true,
            // The DD2 flyers: the Wyvern T1/T2/T3 (558/559/560,
            // `:16217`/`:16235`/`:16253`, kb 0.2/0.2/0.05) and the Kobold
            // Flyer T2/T3 (574/575, `:16491`/`:16509`, kb 0.4/0.25).
            NpcKind::DD2WyvernT1
            | NpcKind::DD2WyvernT2
            | NpcKind::DD2WyvernT3
            | NpcKind::DD2KoboldFlyerT2
            | NpcKind::DD2KoboldFlyerT3 => true,
            // The Angry Nimbus (250) — `SetDefaults` writes no flag, but the
            // aiStyle-49 AI head sets `noGravity = true` every tick
            // (`NPC.cs:31589`); kb 0.3 (`:12231`).
            NpcKind::AngryNimbus => true,
            _ => false,
        }
    }

    /// The C#'s `aiStyle` for this kind - the first dispatch axis of
    /// `NPC.AI` (`NPC.cs:19819-41132`, the `if (this.aiStyle == N)` chain).
    /// Every row is the `SetDefaults` assignment for the kind's vanilla
    /// `NPCID` (the citation is the `this.aiStyle = N;` line). S1-D-2.
    pub fn ai_style(self) -> u8 {
        match self {
            // aiStyle 0 - NPC.cs:10248, :10262, :10498, :13507, :13801, :16657
            NpcKind::SleepingAngler | NpcKind::WebbedStylist | NpcKind::SleepingGolfer | NpcKind::BoundWizard |
            NpcKind::BoundMechanic | NpcKind::BoundGoblin => 0,
            // aiStyle 1 - NPC.cs:8576; wave A: :8968, :9520, :9702,
            // :9842, :10752, :10851 region, :11336 region, :11350
            // region, :11902 region.
            NpcKind::Slime
            | NpcKind::MotherSlime
            | NpcKind::LavaSlime
            | NpcKind::DungeonSlime
            | NpcKind::CorruptSlime
            | NpcKind::IlluminantSlime
            | NpcKind::IceSlime
            | NpcKind::Crimslime
            | NpcKind::SpikedIceSlime
            | NpcKind::UmbrellaSlime => 1,
            // aiStyle 2 - NPC.cs:8590, :10650
            NpcKind::DemonEye
            | NpcKind::WanderingEye
            | NpcKind::CataractEye
            | NpcKind::SleepyEye
            | NpcKind::DialatedEye
            | NpcKind::GreenEye
            | NpcKind::PurpleEye => 2,
            // aiStyle 3 - NPC.cs:8603, :9040, :9312, :11726, :11739, :11752, :11765, :11778, :12933, :13133, :13285, :13331, :13344, :13860, :13876, :13902, :13916, :13959, :13986, :15053, :15065, :15148, :15192, :15204, :15217, :15228, :15241, :15326, :15350, :15389, :15649, :15663, :15677, :15719, :15732, :15760
            NpcKind::Zombie | NpcKind::Skeleton | NpcKind::UndeadMiner | NpcKind::GoblinThief | NpcKind::GoblinWarrior |
            NpcKind::GoblinArcher | NpcKind::GoblinPeon | NpcKind::GoblinSummoner | NpcKind::MartianOfficer |
            NpcKind::GrayGrunt | NpcKind::MartianEngineer | NpcKind::GigaZapper | NpcKind::MartianWalker |
            NpcKind::BrainScrambler | NpcKind::RayGunner | NpcKind::Scutlix | NpcKind::VortexRifleman |
            NpcKind::VortexSoldier | NpcKind::VortexHunter | NpcKind::VortexAlien | NpcKind::VortexHeadcrab |
            NpcKind::Predictor | NpcKind::StardustSpiderBig | NpcKind::StardustSoldier | NpcKind::SolarDrakomire |
            NpcKind::SolarSelenian | NpcKind::SolarSpearman | NpcKind::PirateDeckhand | NpcKind::PirateCaptainsMate |
            NpcKind::PirateArcher | NpcKind::PirateCorsair | NpcKind::PirateCaptain | NpcKind::Scarecrow1 |
            NpcKind::Scarecrow2 | NpcKind::Scarecrow3 | NpcKind::Scarecrow4 | NpcKind::Scarecrow5 |
            NpcKind::Scarecrow6 | NpcKind::Scarecrow7 | NpcKind::Scarecrow8 | NpcKind::Scarecrow9 |
            NpcKind::Scarecrow10 | NpcKind::Splinterling | NpcKind::ZombieElf | NpcKind::ZombieElfBeard |
            NpcKind::ZombieElfGirl | NpcKind::GingerbreadMan | NpcKind::Yeti => 3,
            // The Frost Moon walkers (S1-F-13c slice B) - aiStyle 3:
            // NPC.cs:13420 (348), :13434 (349), :13448 (350), :13461 (351).
            NpcKind::Nutcracker | NpcKind::NutcrackerSpinning | NpcKind::ElfArcher | NpcKind::Krampus => 3,
            // aiStyle 3 — wave A (`NPC.cs:9422`, :9436, :10637, :11021,
            // :11379, :11392, :11405 region, :11418 region, :11563
            // region, :11876 region).
            NpcKind::DoctorBones
            | NpcKind::TheGroom
            | NpcKind::BaldZombie
            | NpcKind::ZombieEskimo
            | NpcKind::PincushionZombie
            | NpcKind::SlimedZombie
            | NpcKind::SwampZombie
            | NpcKind::TwiggyZombie
            | NpcKind::FemaleZombie
            | NpcKind::ZombieRaincoat => 3,
            // aiStyle 3 — the holiday zombies (S1-D-3z; `NPC.cs:13037`/
            // `:13051`/`:13065`/`:13208`/`:13221`).
            NpcKind::ZombieDoctor
            | NpcKind::ZombieSuperman
            | NpcKind::ZombiePixie
            | NpcKind::ZombieXmas
            | NpcKind::ZombieSweater => 3,
            // aiStyle 3 — the Blood Zombie (S1-D-3z; `NPC.cs:14609`).
            NpcKind::BloodZombie => 3,
            // aiStyle 4 - NPC.cs:8707
            NpcKind::EyeOfCthulhu => 4,
            // aiStyle 5 - NPC.cs:8724, :8738, :9067, :9283
            NpcKind::EaterOfSouls | NpcKind::Hornet | NpcKind::MeteorHead | NpcKind::ServantOfCthulhu => 5,
            // The Queen Bee's volley bees — `NPC.cs:11700`/`:11713`
            // (the type-210/211 branch of AI_005, `:50384-50413`).
            NpcKind::Bee | NpcKind::BeeSmall => 5,
            // aiStyle 6 - NPC.cs:8768, :8796, :8825, :8854, :8871, :8889, :8908, :8927, :8947, :10409, :10425, :10441 (the leech tail, W-A2-3), :15010, :15269, :15287, :15305
            NpcKind::SolarCrawltipedeHead | NpcKind::SolarCrawltipedeBody | NpcKind::SolarCrawltipedeTail |
            NpcKind::StardustWormHead | NpcKind::DevourerHead | NpcKind::DevourerBody | NpcKind::DevourerTail |
            NpcKind::GiantWormHead | NpcKind::GiantWormBody | NpcKind::GiantWormTail | NpcKind::EaterOfWorldsHead |
            NpcKind::EaterOfWorldsBody | NpcKind::EaterOfWorldsTail | NpcKind::LeechHead | NpcKind::LeechBody |
            NpcKind::LeechTail => 6,
            // aiStyle 6 — the wyvern family (`NPC.cs:9945`, :9962, :9980,
            // :9998, :10016, :10034).
            NpcKind::WyvernHead | NpcKind::WyvernLegs | NpcKind::WyvernBody |
            NpcKind::WyvernBody2 | NpcKind::WyvernBody3 | NpcKind::WyvernTail => 6,
            // aiStyle 7 - NPC.cs:8986, :9000, :9014, :9028, :9055, :9217, :9452, :10277, :10291, :10514, :11660, :11674, :11931, :11945, :13494, :13679, :13693, :14710, :16267, :16644, :17153, :17390
            NpcKind::Guide | NpcKind::DD2Bartender | NpcKind::Merchant | NpcKind::Dryad | NpcKind::Nurse |
            NpcKind::ArmsDealer | NpcKind::Demolitionist | NpcKind::Painter | NpcKind::WitchDoctor |
            NpcKind::Clothier | NpcKind::TravellingMerchant | NpcKind::Angler | NpcKind::Stylist | NpcKind::TaxCollector |
            NpcKind::Golfer | NpcKind::Zoologist | NpcKind::Princess | NpcKind::DyeTrader | NpcKind::PartyGirl |
            NpcKind::Wizard | NpcKind::Mechanic | NpcKind::GoblinTinkerer => 7,
            // aiStyle 8 - NPC.cs:9083, :9125, :9326, :14978, :15690
            NpcKind::GoblinSorcerer | NpcKind::Tim | NpcKind::DarkCaster | NpcKind::FireImp | NpcKind::DesertDjinn => 8,
            // aiStyle 9 - NPC.cs:9097, :9139, :15703
            NpcKind::ChaosBall | NpcKind::BurningSphere | NpcKind::WaterSphere => 9,
            // aiStyle 11 - NPC.cs:9170
            NpcKind::SkeletronHead => 11,
            // aiStyle 12 - NPC.cs:9187
            NpcKind::SkeletronHand => 12,
            // aiStyle 14 - NPC.cs:9378, :9409, :9543, :9629
            NpcKind::CaveBat | NpcKind::JungleBat | NpcKind::Hellbat | NpcKind::VoodooDemon => 14,
            // aiStyle 15 - NPC.cs:9392
            NpcKind::KingSlime => 15,
            // aiStyle 16 - NPC.cs:9465, :9508, :9615
            NpcKind::Goldfish | NpcKind::Piranha | NpcKind::Shark => 16,
            // aiStyle 22 - NPC.cs:13194
            NpcKind::Poltergeist => 22,
            // aiStyle 22 — the Drippler (S1-D-3z; `NPC.cs:14623`).
            NpcKind::Drippler => 22,
            // aiStyle 25 - NPC.cs:13318 (the Present Mimic's chest machine,
            // `AI_025`, `:25538-25630`)
            NpcKind::PresentMimic => 25,
            // aiStyle 26 - NPC.cs:12982, :13179, :15078, :15121
            NpcKind::NebulaBeast | NpcKind::StardustSpiderSmall | NpcKind::HeadlessHorseman | NpcKind::Hellhound => 26,
            // aiStyle 27 - NPC.cs:10347
            NpcKind::WallOfFlesh => 27,
            // aiStyle 28 - NPC.cs:10365
            NpcKind::WallOfFleshEye => 28,
            // aiStyle 29 - NPC.cs:10382
            NpcKind::Hungry => 29,
            // aiStyle 30 - NPC.cs:10526
            NpcKind::Retinazer => 30,
            // aiStyle 31 - NPC.cs:10543
            NpcKind::Spazmatism => 31,
            // aiStyle 32 - NPC.cs:10560
            NpcKind::SkeletronPrime => 32,
            // aiStyle 33 - NPC.cs:10592
            NpcKind::PrimeSaw => 33,
            // aiStyle 34 - NPC.cs:10607
            NpcKind::PrimeVice => 34,
            // aiStyle 35 - NPC.cs:10577
            NpcKind::PrimeCannon => 35,
            // aiStyle 36 - NPC.cs:10622
            NpcKind::PrimeLaser => 36,
            // aiStyle 37 - NPC.cs:10664, :10685, :10705
            NpcKind::TheDestroyer | NpcKind::TheDestroyerBody | NpcKind::TheDestroyerTail => 37,
            // aiStyle 5 - NPC.cs:10725 (the Probe rides AI_005's type-139 arm)
            NpcKind::Probe => 5,
            // aiStyle 38 - NPC.cs:10809, :10823, :10837
            NpcKind::SnowmanGangsta | NpcKind::SnowBalla | NpcKind::MysterySnowman => 38,
            // aiStyle 39 - NPC.cs:15360
            NpcKind::SolarSroller => 39,
            // aiStyle 43 - NPC.cs:11859
            NpcKind::QueenBee => 43,
            // aiStyle 45 - NPC.cs:12167
            NpcKind::Golem => 45,
            // aiStyle 46 - NPC.cs:12184
            NpcKind::GolemHead => 46,
            // aiStyle 47 - NPC.cs:12198
            NpcKind::GolemFistLeft | NpcKind::GolemFistRight => 47,
            // aiStyle 48 - NPC.cs:12212
            NpcKind::GolemHeadFree => 48,
            // aiStyle 50 - NPC.cs:12368 (261), :12427 (265): the spore
            // pair (S1-D-3-wire-plantera-minions, census §5 W-8).
            NpcKind::FungiSpore | NpcKind::Spore => 50,
            // aiStyle 51 - NPC.cs:12384
            NpcKind::Plantera => 51,
            // aiStyle 52 - NPC.cs:12402 (263, the family's only member).
            NpcKind::PlanterasHook => 52,
            // aiStyle 53 - NPC.cs:12414 (264).
            NpcKind::PlanterasTentacle => 53,
            // aiStyle 54 - NPC.cs:12443
            NpcKind::BrainOfCthulhu => 54,
            // aiStyle 55 - NPC.cs:12460
            NpcKind::Creeper => 55,
            // aiStyle 57 - NPC.cs:13119, :13359
            NpcKind::MourningWood | NpcKind::Everscream => 57,
            // aiStyle 58 - NPC.cs:13146
            NpcKind::Pumpking => 58,
            // aiStyle 60 - NPC.cs:13373 (`S1-F-13c` slice B)
            NpcKind::IceQueen => 60,
            // aiStyle 61 - NPC.cs:13390
            NpcKind::SantaNK1 => 61,
            // aiStyle 62 - NPC.cs:13404
            NpcKind::ElfCopter => 62,
            // aiStyle 63 - NPC.cs:13475
            NpcKind::Flocko => 63,
            // aiStyle 72 - NPC.cs:13888 (the Officer's shield bubble)
            NpcKind::ForceBubble => 72,
            // aiStyle 73 - NPC.cs:13926 (the Engineer's Tesla Turret)
            NpcKind::MartianTurret => 73,
            // aiStyle 74 - NPC.cs:13939, :15372
            NpcKind::MartianDrone | NpcKind::SolarCorite => 74,
            // aiStyle 75 - NPC.cs:13973, :14000, :14018, :14035, :14655, :15338
            NpcKind::ScutlixRider | NpcKind::MartianSaucer | NpcKind::MartianSaucerTurret | NpcKind::MartianSaucerCannon |
            NpcKind::SolarDrakomireRider | NpcKind::FlyingDutchmanCannon => 75,
            // aiStyle 76 - NPC.cs:14052
            NpcKind::MartianSaucerCore => 76,
            // aiStyle 77 - NPC.cs:14127
            NpcKind::MoonLordCore => 77,
            // aiStyle 78 - NPC.cs:14108
            NpcKind::MoonLordHand => 78,
            // aiStyle 79 - NPC.cs:14089
            NpcKind::MoonLordHead => 79,
            // aiStyle 80 - NPC.cs:14069
            NpcKind::MartianProbe => 80,
            // aiStyle 81 - NPC.cs:14146
            NpcKind::MoonLordFreeEye => 81,
            // aiStyle 82 - NPC.cs:14166
            NpcKind::MoonLordLeechBlob => 82,
            // aiStyle 84 - NPC.cs:14216
            NpcKind::AncientCultist => 84,
            // aiStyle 85 - NPC.cs:15026, :15134
            NpcKind::StardustCellBig | NpcKind::BrainSuckler => 85,
            // aiStyle 93 - NPC.cs:14637
            NpcKind::FlyingDutchman => 93,
            // aiStyle 94 - NPC.cs:14995, :15108, :15179, :15256
            NpcKind::StardustTower | NpcKind::NebulaTower | NpcKind::SolarTower | NpcKind::VortexTower => 94,
            // aiStyle 95 - NPC.cs:15040
            NpcKind::StardustCellSmall => 95,
            // aiStyle 96 - NPC.cs:15091
            NpcKind::StardustJellyfishBig => 96,
            // aiStyle 97 - NPC.cs:15161
            NpcKind::NebulaFloater => 97,
            // aiStyle 105 - NPC.cs:16002
            NpcKind::DD2EterniaCrystal => 105,
            // aiStyle 106 - NPC.cs:16040
            NpcKind::DD2LanePortal => 106,
            // aiStyle 107 - NPC.cs:16061, :16078, :16095, :16112, :16129, :16146, :16163, :16180, :16197, :16282, :16299, :16316, :16333, :16350, :16367, :16384, :16401, :16418, :16435
            NpcKind::DD2GoblinT1 | NpcKind::DD2GoblinT2 | NpcKind::DD2GoblinT3 | NpcKind::DD2GoblinBomberT1 |
            NpcKind::DD2GoblinBomberT2 | NpcKind::DD2GoblinBomberT3 | NpcKind::DD2JavelinT1 | NpcKind::DD2JavelinT2 |
            NpcKind::DD2JavelinT3 | NpcKind::DD2SkeletonT1 | NpcKind::DD2SkeletonT3 | NpcKind::DD2WitherBeastT3 |
            NpcKind::DD2DrakinT2 | NpcKind::DD2DrakinT3 | NpcKind::DD2KoboldWalkerT2 | NpcKind::DD2KoboldWalkerT3 |
            NpcKind::DD2OgreT2 | NpcKind::DD2OgreT3 | NpcKind::DD2WitherBeast => 107,
            // aiStyle 108 - NPC.cs:16211, :16229, :16247, :16485, :16503
            NpcKind::DD2WyvernT1 | NpcKind::DD2WyvernT2 | NpcKind::DD2WyvernT3 | NpcKind::DD2KoboldFlyerT2 |
            NpcKind::DD2KoboldFlyerT3 => 108,
            // aiStyle 109 - NPC.cs:16452, :16470
            NpcKind::DD2DarkMageT3 | NpcKind::DD2DarkMage => 109,
            // aiStyle 110 - NPC.cs:16524
            NpcKind::DD2Betsy => 110,
            // aiStyle 123 - NPC.cs:17453
            NpcKind::Deerclops => 123,
            // S1-D-3 W-9 fishron wiring — `AI_069_DukeFishron`
            // (`NPC.cs:48774-49754`), aiStyle 69 (`NPC.cs:13705`); the
            // Detonating Bubble's aiStyle 70 (`:13723`) and the Sharkron
            // pair's aiStyle 71 (`:13739`/`:13753`).
            NpcKind::DukeFishron => 69,
            NpcKind::DetonatingBubble => 70,
            NpcKind::Sharkron | NpcKind::Sharkron2 => 71,
            // The small-kinds wiring wave (S1-D-3-wire-smallkinds):
            // aiStyle 21 - NPC.cs:9719 (the BlazingWheel, the only user);
            // 49 - :12225 (the AngryNimbus); 56 - :12742 (the Dungeon
            // Spirit); 72 - :13888 (the ForceBubble).
            NpcKind::BlazingWheel => 21,
            NpcKind::AngryNimbus => 49,
            NpcKind::DungeonSpirit => 56,
        }
    }

    /// `AI_003_Fighters`'s per-type walk-speed caps (`NPC.cs:57403-59800`):
    /// the velocity ceiling each kind walks at.
    ///
    /// The C#'s groups:
    /// - the `num107` default (`:58322-58374`): 1f;
    /// - the `num83` group (`:57606-57691`): 1.5f with the per-type rows;
    /// - the cap-2 group (`:57554-57578`);
    /// - the cap-3 group (`:57520-57552`);
    /// - the shooters' group (`:59717-59750`): 1f with the per-type rows;
    /// - the Martian engineer's own arm (`:58032-58069`): 1.5f.
    ///
    /// The kinds whose arm is bespoke (the pillar mobs, the vortex family,
    /// the Goblin Archer) carry their own speed and are not listed — the
    /// table is the walkers' shared source.
    pub fn ai3_walk_speed(self) -> Option<f32> {
        Some(match self {
            NpcKind::Zombie => 1.0,             // the num107 default (`:58322`)
            NpcKind::Skeleton => 1.5,           // the num83 group (`:57606`)
            NpcKind::UndeadMiner => 1.0,        // the num107 default
            NpcKind::GoblinPeon => 1.5,         // the num83 group
            NpcKind::GoblinThief => 2.0,        // the cap-2 group (`:57554`)
            NpcKind::GoblinWarrior => 1.0,      // the num107 default
            NpcKind::Spazmatism => 2.0,        // the cap-2 group (`:57554`)
            NpcKind::PirateDeckhand => 2.0,     // the cap-2 group
            NpcKind::PirateCaptainsMate => 3.0, // the cap-3 group (`:57520`)
            NpcKind::PirateArcher => 2.0,       // the shooters' group (`:59722`)
            NpcKind::PirateCorsair => 1.5,      // the shooters' group (`:59727`)
            NpcKind::PirateCaptain => 1.0,      // the shooters' default (`:59717`)
            NpcKind::BrainScrambler => 2.0,     // the shooters' group (`:59731`)
            NpcKind::RayGunner => 2.0,          // the shooters' group (`:59731`)
            NpcKind::MartianOfficer => 1.0,     // the shooters' default (`:59717`)
            NpcKind::GrayGrunt => 1.8,          // the num83 group (`:57649`)
            NpcKind::MartianEngineer => 1.5,    // its own arm (`:58032`)
            NpcKind::GigaZapper => 2.25,        // the num83 group (`:57653`)
            NpcKind::MartianWalker => 4.0,      // the shooters' group (`:59735`)
            // The Pumpkin Moon family (`S1-F-13b`): the 11 aiStyle-3
            // walkers ride the C#'s own scarecrow arm (`NPC.cs:57816-57840`):
            // the five stat pairs' `num85` caps, one row per pair —
            // 305/310 → 2f, 306/311 → 1.25f, 307/312 → 2.25f, 308/313 → 1.5f,
            // 309/314 → 1f — with Splinterling (326) in the cap-2 group
            // (`:57554-57578`). The Headless Horseman (315) and the
            // Hellhound (329) are aiStyle 26 (`AI_026_Unicorns`) — their
            // named machine landed with S1-F-13b2 slice A
            // (`npc_ai_unicorn.rs`); these rows stay as the cap pins the
            // family tests read, matching the machine's own chase
            // ceiling (`num10 = 6f`, `NPC.cs:62617`).
            NpcKind::Scarecrow1 | NpcKind::Scarecrow6 => 2.0,
            NpcKind::Scarecrow2 | NpcKind::Scarecrow7 => 1.25,
            NpcKind::Scarecrow3 | NpcKind::Scarecrow8 => 2.25,
            NpcKind::Scarecrow4 | NpcKind::Scarecrow9 => 1.5,
            NpcKind::Scarecrow5 | NpcKind::Scarecrow10 => 1.0,
            NpcKind::Splinterling => 2.0,
            NpcKind::HeadlessHorseman | NpcKind::Hellhound => 6.0,
            // S1-D-3ff wave A — the `num107` group's per-type rows
            // (`AI_003_Fighters`, `NPC.cs:58324-58376`): the default 1f
            // (52/53/161 carry no override), 132 0.95f (`:58345-58348`),
            // 186 1.1f (`:58329-58332`), 187 0.9f (`:58333-58336`),
            // 188 1.2f (`:58337-58340`), 189 0.8f (`:58341-58344`),
            // 200 0.87f (`:58349-58352`), 223 1.05f (`:58353-58356`).
            // The `num107 *= 1f + (1f - scale)` factor (`:58374-58377`)
            // is 1 for these kinds (the port carries no per-kind scale
            // outside the slime variants — documented deviation).
            NpcKind::DoctorBones | NpcKind::TheGroom | NpcKind::ZombieEskimo => 1.0,
            // S1-D-3z — the holiday zombies ride the `num107` default 1f
            // (no per-type override at `:58329-58364`). 319/320/321 are NOT
            // in the `:58374` scale-factor set; 331/332 are, but their C#
            // scale is 1f (no `SetDefaults` scale row), so the factor is 1
            // and the effective cap is 1f for all five.
            NpcKind::ZombieDoctor
            | NpcKind::ZombieSuperman
            | NpcKind::ZombiePixie
            | NpcKind::ZombieXmas
            | NpcKind::ZombieSweater => 1.0,
            NpcKind::BaldZombie => 0.95,
            NpcKind::PincushionZombie => 1.1,
            NpcKind::SlimedZombie => 0.9,
            NpcKind::SwampZombie => 1.2,
            NpcKind::TwiggyZombie => 0.8,
            NpcKind::FemaleZombie => 0.87,
            NpcKind::ZombieRaincoat => 1.05,
            // The Frost Moon family, slice A (`S1-F-13c`): the five
            // aiStyle-3 walkers. The Zombie Elf trio and the Gingerbread Man
            // ride the num83 group (`NPC.cs:57606`) — 338 → 1.75f
            // (`:57637-57639`), 339 → 1.25f (`:57641-57643`), 340 → 2f
            // (`:57645-57647`), 342 → the group base 1.5f (`:57608`; the C#'s
            // `:57693-57695` scale factor `1 + (1 - scale)` is 1 at the 1f
            // default scale). The Yeti rides the cap-2 group (`:57554`,
            // ±2f, accel 0.07). The Present Mimic (341, aiStyle 25) rides the
            // documented walker stand-in at its hop launch's 3.5f cap
            // (`NPC.cs:25617`); the Everscream (344, aiStyle 57) hovers on
            // the Mourning Wood stand-in's rows — neither reads this table.
            NpcKind::ZombieElf => 1.75,
            NpcKind::ZombieElfBeard => 1.25,
            NpcKind::ZombieElfGirl => 2.0,
            NpcKind::GingerbreadMan => 1.5,
            NpcKind::Yeti => 2.0,
            NpcKind::PresentMimic => 3.5,
            // The Frost Moon family (`S1-F-13c` slice B): the four aiStyle-3
            // walkers' caps — 348 and 351 in the cap-2 group
            // (`NPC.cs:57554-57578`), 349 in the cap-6 group
            // (`:57403-57427`), 350 the shooters' `num180 = 1f` default
            // (`:59717` — no type-350 override exists).
            NpcKind::Nutcracker | NpcKind::Krampus => 2.0,
            NpcKind::NutcrackerSpinning => 6.0,
            NpcKind::ElfArcher => 1.0,
            _ => return None,
        })
    }

    /// S1-D-3z — the holiday zombies' `NPC.scale` `SetDefaults` rows
    /// (`NPC.cs:13045`/`:13059`/`:13073`): 319 0.9f, 320 1.05f, 321 1.1f.
    /// The xmas pair (331/332) writes no scale row — the `SetDefaults`
    /// head's 1f default (`NPC.cs:8510-8537`). `None` for every kind
    /// outside the halloween trio (the other kinds' scale rides
    /// `pumpkin_moon_defaults`/`frost_moon_defaults` or `size()`).
    pub fn holiday_zombie_scale(self) -> Option<f32> {
        Some(match self {
            NpcKind::ZombieDoctor => 0.9,
            NpcKind::ZombieSuperman => 1.05,
            NpcKind::ZombiePixie => 1.1,
            _ => return None,
        })
    }

    /// `NPC.GetNPCInvasionGroup(type)` (`NPC.cs`): the goblin army's members
    /// (26/27/28/29/111/471) — every ported invasion NPC. The other groups
    /// (pirate 2, snow 3, ...) have no NPCs in the ported set.
    pub fn is_invasion_npc(self) -> bool {
        matches!(
            self,
            NpcKind::GoblinThief
                | NpcKind::GoblinWarrior
                | NpcKind::GoblinArcher
                | NpcKind::GoblinSorcerer
                | NpcKind::GoblinPeon
                | NpcKind::GoblinSummoner
        )
    }

    /// The vanilla hitboxes: blue slime 24x18 (`NPC.cs:8568-8569`), demon eye
    /// and Wandering Eye 30x32 (`NPC.cs` cases 2/133), zombie 18x40.
    pub fn size(self) -> (f32, f32) {
        match self {
            NpcKind::Slime => (24.0, 18.0),
            // S1-D-3ff wave A — the `SetDefaults` boxes: 16/71 36x24
            // (`NPC.cs:8966-8967`/`:9700-9701`), 59/138/147/184 24x18
            // (`:9518-9519`/`:10750-10751`/`:10849-10850` region/
            // `:11348-11349` region), 81/183 40x30 (`:9840-9841`/
            // `:11334-11335` region), 225 38x26 (`:11900-11901` region).
            // The C# `scale` rows (1.25/1.1/1.05) are unported outside
            // the slime variants — documented deviation.
            NpcKind::MotherSlime | NpcKind::DungeonSlime => (36.0, 24.0),
            NpcKind::LavaSlime
            | NpcKind::IlluminantSlime
            | NpcKind::IceSlime
            | NpcKind::SpikedIceSlime => (24.0, 18.0),
            NpcKind::CorruptSlime | NpcKind::Crimslime => (40.0, 30.0),
            NpcKind::UmbrellaSlime => (38.0, 26.0),
            NpcKind::DemonEye
            | NpcKind::WanderingEye
            | NpcKind::CataractEye
            | NpcKind::SleepyEye
            | NpcKind::DialatedEye
            | NpcKind::GreenEye
            | NpcKind::PurpleEye => (30.0, 32.0),
            NpcKind::Zombie
            | NpcKind::DoctorBones
            | NpcKind::TheGroom
            | NpcKind::BaldZombie
            | NpcKind::ZombieEskimo
            | NpcKind::PincushionZombie
            | NpcKind::SlimedZombie
            | NpcKind::SwampZombie
            | NpcKind::TwiggyZombie
            | NpcKind::FemaleZombie
            | NpcKind::ZombieRaincoat
            // S1-D-3z — the holiday zombies' 18x40 boxes (`NPC.cs:13035-13036`/
            // `:13049-13050`/`:13063-13064`/`:13206-13207`/`:13219-13220`).
            // The C# `scale` rows (319 0.9 / 320 1.05 / 321 1.1) ride
            // `Npc::scale`, not the hitbox — the box stays 18x40.
            | NpcKind::ZombieDoctor
            | NpcKind::ZombieSuperman
            | NpcKind::ZombiePixie
            | NpcKind::ZombieXmas
            | NpcKind::ZombieSweater
            // S1-D-3z — the Blood Zombie's 18x40 box (`NPC.cs:14607-14608`).
            | NpcKind::BloodZombie => (18.0, 40.0),
            // S1-D-3z — the Drippler's 28x30 box (`NPC.cs:14621-14622`).
            NpcKind::Drippler => (28.0, 30.0),
            // The Guide's hitbox rides the same humanoid frame.
            NpcKind::Guide => (18.0, 40.0),
            NpcKind::Merchant => (18.0, 40.0),
            NpcKind::Dryad => (18.0, 40.0),
            NpcKind::Nurse => (18.0, 40.0),
            NpcKind::ArmsDealer => (18.0, 40.0),
            NpcKind::Demolitionist => (18.0, 40.0),
            NpcKind::Painter => (18.0, 40.0),
            NpcKind::WitchDoctor => (18.0, 40.0),
            NpcKind::Clothier => (18.0, 40.0),
            NpcKind::TravellingMerchant => (18.0, 40.0),
            NpcKind::Angler => (18.0, 40.0),
            NpcKind::SleepingAngler => (34.0, 8.0),
            NpcKind::Wizard => (18.0, 40.0),
            NpcKind::BoundWizard => (18.0, 40.0),
            NpcKind::Mechanic => (18.0, 40.0),
            NpcKind::BoundMechanic => (18.0, 34.0),
            NpcKind::Stylist => (18.0, 40.0),
            NpcKind::WebbedStylist => (18.0, 34.0),
            NpcKind::TaxCollector => (18.0, 40.0),
            NpcKind::Golfer => (18.0, 40.0),
            NpcKind::SleepingGolfer => (18.0, 34.0),
            NpcKind::Zoologist => (18.0, 40.0),
            NpcKind::Princess => (18.0, 40.0),
            NpcKind::DyeTrader => (18.0, 40.0),
            NpcKind::PartyGirl => (18.0, 40.0),
            // `SetDefaults` case 4 (`NPC.cs:8706`): 100x110.
            NpcKind::EyeOfCthulhu => (100.0, 110.0),
            // `SetDefaults` case 222 (`NPC.cs:11858`): 66x66.
            NpcKind::QueenBee => (66.0, 66.0),
            // `SetDefaults` case 35 (`NPC.cs:9168`): 80x102.
            NpcKind::SkeletronHead => (80.0, 102.0),
            // `SetDefaults` case 113 (`NPC.cs:10344`): 100x100 x1.2 = 120.
            NpcKind::WallOfFlesh => (120.0, 120.0),
            // `SetDefaults` case 134 (`NPC.cs:10662`): 38x38 x1.25.
            NpcKind::TheDestroyer => (48.0, 48.0),
            // 135/136 (`NPC.cs:10683-10684`/`:10703-10704`): the same box.
            NpcKind::TheDestroyerBody | NpcKind::TheDestroyerTail => (48.0, 48.0),
            // 139 (`NPC.cs:10723-10724`): 30x30, no scale row.
            NpcKind::Probe => (30.0, 30.0),
            // `SetDefaults` case 127 (`NPC.cs:10558`): 80x102.
            NpcKind::SkeletronPrime => (80.0, 102.0),
            // The Twins: `SetDefaults` cases 125/126 are 100x110.
            NpcKind::Retinazer => (100.0, 110.0),
            NpcKind::Spazmatism => (100.0, 110.0),
            // `SetDefaults` case 262 (`NPC.cs:12381`): 86x86.
            NpcKind::Plantera => (86.0, 86.0),
            // The Plantera minions (S1-D-3-wire-plantera-minions): the
            // `SetDefaults` boxes - 261 12x12 (`NPC.cs:12366-12367`),
            // 263 40x40 (`:12400-12401`), 264 24x24 (`:12412-12413`),
            // 265 18x18 (`:12425-12426`).
            NpcKind::FungiSpore => (12.0, 12.0),
            NpcKind::PlanterasHook => (40.0, 40.0),
            NpcKind::PlanterasTentacle => (24.0, 24.0),
            NpcKind::Spore => (18.0, 18.0),
            // `SetDefaults` case 266 (`NPC.cs:12440`): 160x110.
            NpcKind::BrainOfCthulhu => (160.0, 110.0),
            // `SetDefaults` case 267 (`NPC.cs:12458`): 30x30.
            NpcKind::Creeper => (30.0, 30.0),
            // The goblins: 18x38 (`NPC.cs`'s cases 26/27/28/111/29/471).
            NpcKind::GoblinThief
            | NpcKind::GoblinWarrior
            | NpcKind::GoblinArcher
            | NpcKind::GoblinPeon
            | NpcKind::GoblinSummoner => (18.0, 38.0),
            // The frost trio: 26x40 (`NPC.cs`'s cases 143/144/145).
            NpcKind::SnowmanGangsta | NpcKind::SnowBalla | NpcKind::MysterySnowman => (26.0, 40.0),
            // The martians: 18x40 (`NPC.cs`'s cases 383/385/386/389) + the walker's 34x110.
            NpcKind::MartianOfficer => (18.0, 40.0),
            NpcKind::GrayGrunt => (18.0, 40.0),
            NpcKind::MartianEngineer => (18.0, 40.0),
            NpcKind::GigaZapper => (18.0, 40.0),
            NpcKind::MartianWalker => (34.0, 110.0),
            // The probe: 42x72 (`NPC.cs`'s case 399).
            NpcKind::MartianProbe => (42.0, 72.0),
            // The drone: 40x30 (`NPC.cs`'s case 388).
            NpcKind::MartianDrone => (40.0, 30.0),
            // The bubble: 18x40 (`NPC.cs:13885`/`:13887`); the turret:
            // 20x50 (`NPC.cs:13924-13925`).
            NpcKind::ForceBubble => (18.0, 40.0),
            NpcKind::MartianTurret => (20.0, 50.0),
            // The shooters: 18x40 (`NPC.cs`'s 381/382 grouped row).
            NpcKind::BrainScrambler | NpcKind::RayGunner => (18.0, 40.0),
            // The rider + its carrier: 18x40 / 60x40 (`NPC.cs`'s cases
            // 390/391).
            NpcKind::ScutlixRider => (18.0, 40.0),
            NpcKind::Scutlix => (60.0, 40.0),
            // The saucer family: the shell 150x80, the turret 40x16, the
            // cannon 46x36, the core 120x90 (`NPC.cs`'s cases 392-395).
            NpcKind::MartianSaucer => (150.0, 80.0),
            NpcKind::MartianSaucerTurret => (40.0, 16.0),
            NpcKind::MartianSaucerCannon => (46.0, 36.0),
            NpcKind::MartianSaucerCore => (120.0, 90.0),
            // The Dutchman 350x120 + its cannons 30x30 (`NPC.cs`'s cases
            // 491/492).
            NpcKind::FlyingDutchman => (350.0, 120.0),
            NpcKind::FlyingDutchmanCannon => (30.0, 30.0),
            // The DD2 goblins: 18x40 (`NPC.cs`'s cases 552-554).
            NpcKind::DD2GoblinT1 | NpcKind::DD2GoblinT2 | NpcKind::DD2GoblinT3 => (18.0, 40.0),
            // The DD2 bombers: 18x40; the javelin throwers 26x52; the
            // etherian skeletons 18x40 (`NPC.cs`'s 555-567 rows).
            NpcKind::DD2GoblinBomberT1
            | NpcKind::DD2GoblinBomberT2
            | NpcKind::DD2GoblinBomberT3 => (18.0, 40.0),
            NpcKind::DD2JavelinT1 | NpcKind::DD2JavelinT2 | NpcKind::DD2JavelinT3 => (26.0, 52.0),
            NpcKind::DD2SkeletonT1 | NpcKind::DD2SkeletonT3 => (18.0, 40.0),
            // The Dark Mage: 34x62 (`NPC.cs`'s case 564).
            NpcKind::DD2DarkMage => (34.0, 62.0),
            // Betsy: 190x90 (`NPC.cs`'s case 551).
            NpcKind::DD2Betsy => (190.0, 90.0),
            // The WitherBeast: 18x40 (`NPC.cs`'s case 568).
            NpcKind::DD2WitherBeast => (18.0, 40.0),
            NpcKind::DD2WyvernT1
            | NpcKind::DD2WyvernT2
            | NpcKind::DD2WyvernT3
            | NpcKind::DD2KoboldFlyerT2
            | NpcKind::DD2KoboldFlyerT3 => (38.0, 38.0),
            // The Dark Mage T3 rides the T1's 34x62 box.
            NpcKind::DD2DarkMageT3 => (34.0, 62.0),
            // The Wither Beast T3 rides the T2's 18x40 box.
            NpcKind::DD2WitherBeastT3 => (18.0, 40.0),
            // The Drakin: the 42x58 caster box.
            NpcKind::DD2DrakinT2 | NpcKind::DD2DrakinT3 => (42.0, 58.0),
            // The Kobold walker: the 18x40 goblin box.
            NpcKind::DD2KoboldWalkerT2 | NpcKind::DD2KoboldWalkerT3 => (18.0, 40.0),
            // The Ogre: the 96x124 box.
            NpcKind::DD2OgreT2 | NpcKind::DD2OgreT3 => (96.0, 124.0),
            // The Lunar Pillars: 130x270 (`NPC.cs` cases 422/493/507/517).
            NpcKind::StardustTower
            | NpcKind::NebulaTower
            | NpcKind::SolarTower
            | NpcKind::VortexTower => (130.0, 270.0),
            // Lunatic Cultist (439): 24x50 (`NPC.cs:14214-14215`). S1-D-3b.
            NpcKind::AncientCultist => (24.0, 50.0),
            // The lane gate: 78x130 (`NPC.cs`'s case 549).
            NpcKind::DD2LanePortal => (78.0, 130.0),
            NpcKind::VortexHunter => (50.0, 62.0),
            NpcKind::VortexAlien => (40.0, 28.0),
            NpcKind::VortexHeadcrab => (24.0, 26.0),
            NpcKind::StardustSpiderBig => (34.0, 42.0),
            NpcKind::StardustSpiderSmall => (22.0, 22.0),
            NpcKind::StardustSoldier => (22.0, 56.0),
            NpcKind::SolarDrakomire => (60.0, 40.0),
            NpcKind::SolarDrakomireRider => (18.0, 40.0),
            NpcKind::SolarSroller => (46.0, 52.0),
            NpcKind::SolarCorite => (38.0, 38.0),
            NpcKind::SolarCrawltipedeHead
            | NpcKind::SolarCrawltipedeBody
            | NpcKind::SolarCrawltipedeTail => (20.0, 20.0),
            NpcKind::StardustWormHead => (32.0, 32.0),
            NpcKind::SolarSpearman => (22.0, 56.0),
            NpcKind::SolarCrawltipedeHead
            | NpcKind::SolarCrawltipedeBody
            | NpcKind::SolarCrawltipedeTail => (20.0, 20.0),
            NpcKind::StardustWormHead => (32.0, 32.0),
            NpcKind::SolarSpearman => (22.0, 56.0),
            NpcKind::SolarCrawltipedeHead
            | NpcKind::SolarCrawltipedeBody
            | NpcKind::SolarCrawltipedeTail => (20.0, 20.0),
            NpcKind::StardustWormHead => (32.0, 32.0),
            NpcKind::SolarSpearman => (22.0, 56.0),
            NpcKind::SolarCrawltipedeHead
            | NpcKind::SolarCrawltipedeBody
            | NpcKind::SolarCrawltipedeTail => (20.0, 20.0),
            NpcKind::StardustWormHead => (32.0, 32.0),
            NpcKind::SolarSpearman => (22.0, 56.0),
            NpcKind::SolarCrawltipedeHead
            | NpcKind::SolarCrawltipedeBody
            | NpcKind::SolarCrawltipedeTail => (20.0, 20.0),
            NpcKind::StardustWormHead => (32.0, 32.0),
            NpcKind::SolarSpearman => (22.0, 56.0),
            NpcKind::SolarSelenian => (22.0, 56.0),
            NpcKind::NebulaFloater => (40.0, 40.0),
            NpcKind::BrainSuckler => (44.0, 44.0),
            NpcKind::Predictor => (22.0, 56.0),
            NpcKind::MoonLordHead => (38.0, 56.0),
            NpcKind::MoonLordHand | NpcKind::MoonLordCore => (46.0, 66.0),
            NpcKind::MoonLordFreeEye => (60.0, 60.0),
            NpcKind::MoonLordLeechBlob => (30.0, 30.0),
            NpcKind::StardustCellBig => (44.0, 44.0),
            NpcKind::StardustCellSmall => (22.0, 22.0),
            NpcKind::StardustJellyfishBig => (40.0, 60.0),
            NpcKind::NebulaBeast => (50.0, 44.0),
            NpcKind::VortexRifleman => (30.0, 56.0),
            NpcKind::VortexSoldier => (22.0, 56.0),
            // The tavernkeeper + the etherian crystal: 18x40 / 40x40.
            NpcKind::DD2Bartender => (18.0, 40.0),
            NpcKind::DD2EterniaCrystal => (40.0, 40.0),
            // The pirates: 18x40 (`NPC.cs`'s cases 212-216).
            NpcKind::PirateDeckhand
            | NpcKind::PirateCaptainsMate
            | NpcKind::PirateArcher
            | NpcKind::PirateCorsair
            | NpcKind::PirateCaptain => (18.0, 40.0),
            // `SetDefaults` 245-249 (`NPC.cs:12163-12220`): the body
            // 140x140, the head/free head 70x70, the fists 40x30.
            NpcKind::Golem => (140.0, 140.0),
            NpcKind::GolemHead | NpcKind::GolemHeadFree => (70.0, 70.0),
            NpcKind::GolemFistLeft | NpcKind::GolemFistRight => (40.0, 30.0),
            // `SetDefaults` 668 (`:17451-17452`): 60x154.
            NpcKind::Deerclops => (60.0, 154.0),
            NpcKind::SkeletronHand => (52.0, 52.0),
            NpcKind::WallOfFleshEye => (100.0, 100.0),
            NpcKind::PrimeCannon | NpcKind::PrimeSaw | NpcKind::PrimeVice | NpcKind::PrimeLaser => {
                (52.0, 52.0)
            }
            // The bound: 18x34 (`NPC.cs:10249-10250`), the freed: 18x40
            // (`:10274-10275`).
            NpcKind::BoundGoblin => (18.0, 34.0),
            NpcKind::GoblinTinkerer => (18.0, 40.0),
            // The skeleton humanoids ride the zombie's 18x40 frame
            // (`SetDefaults` case 21/44: width 18, height 40).
            NpcKind::Skeleton => (18.0, 40.0),
            NpcKind::UndeadMiner => (18.0, 40.0),
            // `SetDefaults` case 49: width 22, height 18.
            // The bat family shares the Cave Bat's 22x18 frame
            // (`SetDefaults` cases 51/60/66: width 22, height 18).
            NpcKind::JungleBat => (22.0, 18.0),
            NpcKind::Hellbat => (22.0, 18.0),
            NpcKind::VoodooDemon => (22.0, 18.0),
            // `SetDefaults` case 6: 30x30; case 42: 34x32; case 23: 22x22.
            NpcKind::EaterOfSouls => (30.0, 30.0),
            NpcKind::Hornet => (34.0, 32.0),
            NpcKind::MeteorHead => (22.0, 22.0),
            // The volley bees: `SetDefaults` case 210 is 12x12
            // (`NPC.cs:11698-11699`), case 211 is 8x8 (`:11711-11712`).
            NpcKind::Bee => (12.0, 12.0),
            NpcKind::BeeSmall => (8.0, 8.0),
            // The casters: `SetDefaults` case 29 is 18x38, the cases
            // 45/32/24 18x40 (`NPC.cs:8951`/`:9322`/`:9121`/`:9078`).
            // The goblins: 18x38 (`NPC.cs`'s cases 26/27/28/29/111/471).
            NpcKind::GoblinThief
            | NpcKind::GoblinWarrior
            | NpcKind::GoblinArcher
            | NpcKind::GoblinPeon
            | NpcKind::GoblinSummoner
            | NpcKind::GoblinSorcerer => (18.0, 38.0),
            NpcKind::Tim => (18.0, 40.0),
            NpcKind::DarkCaster => (18.0, 40.0),
            NpcKind::FireImp => (18.0, 40.0),
            // `NPCID.DesertDjinn` (533) — 28x62 (`NPC.cs:14976-14977`).
            NpcKind::DesertDjinn => (28.0, 62.0),
            // The spheres: the cases 30/25/33 are 16x16.
            NpcKind::ChaosBall => (16.0, 16.0),
            NpcKind::BurningSphere => (16.0, 16.0),
            NpcKind::WaterSphere => (16.0, 16.0),
            // The swimmers: `SetDefaults` cases 55/58/65 (`NPC.cs:9460`/
            // `:9502`/`:9610`).
            NpcKind::Goldfish => (20.0, 18.0),
            NpcKind::Piranha => (18.0, 20.0),
            NpcKind::Shark => (100.0, 24.0),
            // The worms: the Devourer 22x22 (`:8763`), the Giant Worm 14x14
            // (`:8850`), the Eater of Worlds 38x38 (`:8903`).
            NpcKind::DevourerHead | NpcKind::DevourerBody | NpcKind::DevourerTail => (22.0, 22.0),
            NpcKind::GiantWormHead | NpcKind::GiantWormBody | NpcKind::GiantWormTail => {
                (14.0, 14.0)
            }
            NpcKind::EaterOfWorldsHead
            | NpcKind::EaterOfWorldsBody
            | NpcKind::EaterOfWorldsTail => (38.0, 38.0),
            // The wyvern family: every kind 32x32 (`NPC.cs:9943-9944`/
            // `:9960-9961`/`:9978-9979`/`:9996-9997`/`:10014-10015`/
            // `:10032-10033`).
            NpcKind::WyvernHead
            | NpcKind::WyvernLegs
            | NpcKind::WyvernBody
            | NpcKind::WyvernBody2
            | NpcKind::WyvernBody3
            | NpcKind::WyvernTail => (32.0, 32.0),
            NpcKind::CaveBat => (22.0, 18.0),
            NpcKind::KingSlime => (98.0, 92.0),
            // `NPC.cs:8705` (the eye 100×110) / `:8717` (the servant 20×20).
            NpcKind::EyeOfCthulhu => (100.0, 110.0),
            NpcKind::ServantOfCthulhu => (20.0, 20.0),
            // `NPC.cs`'s 115 (30×30) / 117-118 (14×14).
            NpcKind::Hungry => (30.0, 30.0),
            // `NPC.cs:10407`/`:10423`/`:10439`: 14x14 (W-A2-3: the tail).
            NpcKind::LeechHead | NpcKind::LeechBody | NpcKind::LeechTail => (14.0, 14.0),
            // The Pumpkin Moon family (`S1-F-13b`): the 305-314 group and
            // the Splinterling share the C#'s 18x40 (`NPC.cs:12929-12977`
            // /`:13129-13133`); 315 is 74x70 (`:12978`), 325 164x154
            // (`:13115`), 327 100x100 (`:13142`), 329 46x30 (`:13175`),
            // 330 24x44 (`:13188`).
            NpcKind::Scarecrow1
            | NpcKind::Scarecrow2
            | NpcKind::Scarecrow3
            | NpcKind::Scarecrow4
            | NpcKind::Scarecrow5
            | NpcKind::Scarecrow6
            | NpcKind::Scarecrow7
            | NpcKind::Scarecrow8
            | NpcKind::Scarecrow9
            | NpcKind::Scarecrow10
            | NpcKind::Splinterling => (18.0, 40.0),
            NpcKind::HeadlessHorseman => (74.0, 70.0),
            NpcKind::MourningWood => (164.0, 154.0),
            NpcKind::Pumpking => (100.0, 100.0),
            NpcKind::Hellhound => (46.0, 30.0),
            NpcKind::Poltergeist => (24.0, 44.0),
            // The Frost Moon family, slice A (`S1-F-13c`): the 338-340 group
            // and the Gingerbread Man share the C#'s 18x40
            // (`NPC.cs:13283-13284`/`:13329-13330`); 341 is 24x24
            // (`:13316-13317`), 343 38x78 (`:13342-13343`), 344 172x130
            // (`:13357-13358`).
            NpcKind::ZombieElf
            | NpcKind::ZombieElfBeard
            | NpcKind::ZombieElfGirl
            | NpcKind::GingerbreadMan => (18.0, 40.0),
            NpcKind::PresentMimic => (24.0, 24.0),
            NpcKind::Yeti => (38.0, 78.0),
            NpcKind::Everscream => (172.0, 130.0),
            // The Frost Moon family (`S1-F-13c` slice B): 345 is 130x140
            // (`NPC.cs:13371-13372`), 346 112x140 (`:13388-13389`), 347
            // 50x50 (`:13402-13403`), the nutcracker pair 28x76
            // (`:13418-13419`/`:13432-13433`), 350 18x40
            // (`:13446-13447`), 351 18x90 (`:13459-13460`), 352 54x54
            // (`:13473-13474`).
            NpcKind::IceQueen => (130.0, 140.0),
            NpcKind::SantaNK1 => (112.0, 140.0),
            NpcKind::ElfCopter => (50.0, 50.0),
            NpcKind::Nutcracker | NpcKind::NutcrackerSpinning => (28.0, 76.0),
            NpcKind::ElfArcher => (18.0, 40.0),
            NpcKind::Krampus => (18.0, 90.0),
            NpcKind::Flocko => (54.0, 54.0),
            // S1-D-3 W-9 fishron wiring — the `SetDefaults` boxes:
            // 370 150x100 (`NPC.cs:13703-13704`), 371 36x36
            // (`:13721-13722`), 372 120x24 (`:13736-13737`), 373 100x24
            // (`:13750-13751`).
            NpcKind::DukeFishron => (150.0, 100.0),
            NpcKind::DetonatingBubble => (36.0, 36.0),
            NpcKind::Sharkron => (120.0, 24.0),
            NpcKind::Sharkron2 => (100.0, 24.0),
            // The small-kinds wiring wave: 72 34x34 (`NPC.cs:9717-9718`),
            // 250 34x26 (`:12223-12224`), 288 20x20 (`:12740-12741`),
            // 384 18x40 (`:13885`/`:13887`).
            NpcKind::BlazingWheel => (34.0, 34.0),
            NpcKind::AngryNimbus => (34.0, 26.0),
            NpcKind::DungeonSpirit => (20.0, 20.0),
        }
    }

    /// The C# `NPCID` this kind ports — the statue spawns' ids, the
    /// `MechSpawn` caps (`NPC.cs:7420`) and the teleporting statues' rosters
    /// (`Wiring.cs:2424`/`:2452`) all speak C# ids. Rows arrive with the
    /// kinds; `u16::MAX` marks a kind the caps can never match (the C#'s
    /// `MechSpawn` counts by type, and no statue spawns an unmapped kind).
    pub(crate) fn cs_id(self) -> u16 {
        match self {
            NpcKind::Slime => 1,
            NpcKind::DemonEye => 2,
            NpcKind::WanderingEye => 133,
            // The vanilla `NPCID.Zombie` is 3 — the hand table's 21 was the
            // net-id sweep's finding (S1-F-3; the generated loot-db net id
            // confirms).
            NpcKind::Zombie => 3,
            NpcKind::KingSlime => 50,
            NpcKind::Guide => 22,
            // The teleporting statues' rosters (`Wiring.cs:2424`/`:2452`) are the
            // town NPCs — the port has most of them.
            NpcKind::Merchant => 17,
            NpcKind::Nurse => 18,
            NpcKind::ArmsDealer => 19,
            NpcKind::Dryad => 20,
            NpcKind::Demolitionist => 38,
            NpcKind::Clothier => 54,
            NpcKind::TravellingMerchant => 368,
            NpcKind::Angler => 369,
            NpcKind::SleepingAngler => 376,
            NpcKind::Wizard => 108,
            NpcKind::BoundWizard => 106,
            NpcKind::Mechanic => 124,
            NpcKind::BoundMechanic => 123,
            NpcKind::Stylist => 353,
            NpcKind::WebbedStylist => 354,
            NpcKind::TaxCollector => 441,
            NpcKind::Golfer => 588,
            NpcKind::SleepingGolfer => 589,
            NpcKind::Zoologist => 633,
            NpcKind::Princess => 663,
            NpcKind::DyeTrader => 207,
            NpcKind::PartyGirl => 208,
            NpcKind::Painter => 227,
            NpcKind::WitchDoctor => 228,
            NpcKind::Skeleton => 21, // `NPCID.Skeleton` (the net id's correction, S1-F-3)
            // The Destroyer head keeps its vanilla type; the body/tail/Probe
            // have NO hand row — like every folded segment family (Devourer/
            // GiantWorm/EoW/Golem parts) they fall through to the bestiary
            // net id (the head's 134, `BestiaryDatabaseNPCsPopulator.cs:688-
            // 690`). GATE FIX (L-B): the branch's explicit 135/136/139 rows
            // broke s1_f_3's cs_id==net_id invariant; the machine reads its
            // own `family_npc_type` (134/135/136) and `render_key` keeps the
            // real types — neither rides cs_id.
            NpcKind::TheDestroyer => 134,
            NpcKind::UndeadMiner => 44, // `NPCID.UndeadMiner` (the net id's correction, S1-F-3)
            NpcKind::CaveBat => 49,
            NpcKind::JungleBat => 51,
            NpcKind::Hellbat => 60,
            NpcKind::VoodooDemon => 66,
            NpcKind::EaterOfSouls => 6,
            NpcKind::Hornet => 42,
            NpcKind::MeteorHead => 23, // `NPCID.MeteorHead` (the net id's correction, S1-F-3)
            // The volley bees' vanilla `NPC.type` (the AI branch reads it):
            // `NPCID.Bee` 210 / `BeeSmall` 211 (`NPCID.cs:11097-11099`).
            NpcKind::Bee => 210,
            NpcKind::BeeSmall => 211,
            NpcKind::FireImp => 24,
            // The small-kinds wiring wave: the four kinds' vanilla
            // `NPC.type` ids (`NPCID.cs:10821`/`:11177`/`:11253`/`:11445`).
            // The BlazingWheel's protocol id is port-local 353, so its
            // C#-type consumers (the AI arms' type gates) read 72 here.
            NpcKind::BlazingWheel => 72,
            NpcKind::AngryNimbus => 250,
            NpcKind::DungeonSpirit => 288,
            NpcKind::ForceBubble => 384,
            // `NPCID.DesertDjinn` (533)
            NpcKind::DesertDjinn => 533,
            NpcKind::Goldfish => 55,
            NpcKind::Piranha => 58,
            NpcKind::Shark => 65,
            // The Plantera minion family (S1-D-3-wire-plantera-minions):
            // `NPCID.FungiSpore` 261 / `PlanterasHook` 263 /
            // `PlanterasTentacle` 264 / `Spore` 265 (`ID/NPCID.cs:11199`/
            // `:11203`/`:11205`/`:11207`). Hand rows: the hook/tentacle/
            // spore bestiary credits ride their own ids (the populator
            // `BestiaryDatabaseNPCsPopulator.cs:965`/`:1070`/`:1269`/
            // `:1396` attaches info to all four), so no fold is needed.
            NpcKind::FungiSpore => 261,
            NpcKind::PlanterasHook => 263,
            NpcKind::PlanterasTentacle => 264,
            NpcKind::Spore => 265,
            // S1-D-3 W-9 fishron wiring — the hand rows for the kinds with
            // their OWN bestiary entries. GATE FIX (L-B): the Hide=true
            // hazards (Bubble 371 / Sharkron2 373) fold onto the boss in
            // `try_entry_for_npc` — like every folded segment family they
            // keep NO hand row and fall through to the folded net id
            // (s1_f_3's cs_id==net_id invariant; the destroyer precedent).
            // The 373-only sway gate reads a KIND match in ai_071_sharkron,
            // not cs_id; the render sheets keep 371/373 via render_key.
            NpcKind::DukeFishron => 370,
            NpcKind::Sharkron => 372,
            // S1-F-3: the remaining kinds ride the generated loot-db net ids
            // (`bestiary.rs`'s entries are the C#'s `NPC.type` values) — the
            // hand table above keeps the roster-verified rows; the negative
            // net ids (none in the port's kind set) and the air sentinel stay
            // `u16::MAX` (the caps can never match them).
            _ => {
                // The kinds without a bestiary entry (the boss segments the
                // roster skips) keep the sentinel — the caps never match them.
                match crate::bestiary::try_entry_for_npc(self, SlimeVariant::Blue) {
                    Some(e) if e.net_id > 0 => e.net_id as u16,
                    _ => u16::MAX,
                }
            }
        }
    }

    /// The humanoid walkers' `Main.npcFrameCount` rows (`Main.cs:67082`):
    /// the town group (17 Merchant / 18 Nurse / 19 ArmsDealer / 20 Dryad /
    /// 22 Guide / 38 Demolitionist / 54 Clothier / 107 Tinkerer / 227
    /// Painter / 228 WitchDoctor), the invasion goblins (26 Peon / 27
    /// Thief / 28 Warrior / 111 Archer) and the cave walkers (21 Skeleton
    /// / 44 UndeadMiner) all ride `FindFrame`'s humanoid-group arm
    /// (`NPC.cs:68547-69587`). The Zombie (9) has no case and a single
    /// frame.
    pub fn humanoid_frame_count(self) -> u32 {
        match self {
            NpcKind::Guide | NpcKind::WitchDoctor | NpcKind::TravellingMerchant => 26,
            NpcKind::Merchant
            | NpcKind::ArmsDealer
            | NpcKind::Demolitionist
            | NpcKind::Painter
            | NpcKind::GoblinTinkerer
            | NpcKind::TaxCollector
            | NpcKind::Golfer
            | NpcKind::DyeTrader => 25,
            NpcKind::Nurse
            | NpcKind::Clothier
            | NpcKind::Angler
            | NpcKind::Wizard
            | NpcKind::Mechanic
            | NpcKind::Stylist
            | NpcKind::Zoologist
            | NpcKind::Princess
            | NpcKind::PartyGirl => 23,
            NpcKind::Dryad => 21,
            NpcKind::GoblinArcher => 21,
            NpcKind::GoblinPeon | NpcKind::GoblinThief | NpcKind::GoblinWarrior => 16,
            NpcKind::Skeleton | NpcKind::UndeadMiner => 15,
            // The Zombie Elf trio (`S1-F-13c`): `FindFrame`'s humanoid-group
            // case list carries 338-340 (`NPC.cs:68603-68605`) on their
            // 15-frame strips (`Main.npcFrameCount[338..340]`, `Main.cs`).
            NpcKind::ZombieElf | NpcKind::ZombieElfBeard | NpcKind::ZombieElfGirl => 15,
            _ => 1,
        }
    }

    /// `NPCID.Sets.ExtraFramesCount` (`NPCID.cs:10464`) — the rows the
    /// vanilla reserves at the *bottom* of a town sheet for the attack
    /// poses. `FindFrame`'s humanoid arm wraps the walk band at
    /// `Main.npcFrameCount - ExtraFramesCount` (`NPC.cs:69534`'s `num57`
    /// and the `num57`-subtracted wrap), so a walker must never reach
    /// them: doing so animates the NPC through its attack/pose rows
    /// while it is merely walking. Only the kinds the town machine
    /// animates today carry a value; everything else is 0 (the C#
    /// `CreateIntSet` default).
    pub fn extra_frames_count(self) -> u32 {
        match self {
            NpcKind::Guide | NpcKind::WitchDoctor | NpcKind::TravellingMerchant => 10,
            NpcKind::Merchant
            | NpcKind::Nurse
            | NpcKind::ArmsDealer
            | NpcKind::Demolitionist
            | NpcKind::Painter
            | NpcKind::GoblinTinkerer
            | NpcKind::TaxCollector
            | NpcKind::Golfer
            | NpcKind::DyeTrader
            | NpcKind::PartyGirl
            | NpcKind::Zoologist => 9,
            NpcKind::Dryad
            | NpcKind::Clothier
            | NpcKind::Wizard
            | NpcKind::Stylist
            | NpcKind::Princess => 7,
            NpcKind::Angler | NpcKind::Mechanic => 9,
            _ => 0,
        }
    }

    /// `NPC.behindTiles` (`NPC.cs:6517`, `:8520` default false): kinds drawn in
    /// the behind-tiles NPC pass (`Main.cs:22320,22336` - `DrawNPCs(true)`).
    /// Ported subset: worms/segments and the WoF family (e.g. type 113
    /// `NPC.cs:10355`, Devourer 7-9 `:8777/:8806/:8835`, Leech 117-118
    /// `:10419/:10435`, Destroyer 134 `:10673`).
    pub fn behind_tiles(self) -> bool {
        matches!(
            self,
            NpcKind::DevourerHead
                | NpcKind::DevourerBody
                | NpcKind::DevourerTail
                | NpcKind::GiantWormHead
                | NpcKind::GiantWormBody
                | NpcKind::GiantWormTail
                | NpcKind::EaterOfWorldsHead
                | NpcKind::EaterOfWorldsBody
                | NpcKind::EaterOfWorldsTail
                | NpcKind::WallOfFlesh
                | NpcKind::WallOfFleshEye
                | NpcKind::Hungry
                | NpcKind::LeechHead
                | NpcKind::LeechBody
                | NpcKind::LeechTail
                | NpcKind::TheDestroyer
                | NpcKind::TheDestroyerBody
                | NpcKind::TheDestroyerTail
                // The BlazingWheel's `behindTiles = true` row
                // (`NPC.cs:9724`, the small-kinds wiring wave).
                | NpcKind::BlazingWheel
        )
    }

    /// The Prime's four limbs (`NPCID` 128-131, `aiStyle 33-36`) - the
    /// kinds `AI_032`'s init spawns and the dock tether drives.
    pub fn is_prime_limb(self) -> bool {
        matches!(
            self,
            NpcKind::PrimeCannon | NpcKind::PrimeSaw | NpcKind::PrimeVice | NpcKind::PrimeLaser
        )
    }

    /// The Martian Madness family — the kinds whose `FindFrame` cases live
    /// in `NPC.cs:71578-71921` and whose display rides the drawn
    /// `spriteDirection` (the Scutlix family's inversion, the saucer
    /// mounts' inheritance) rather than the motion `facing`.
    pub fn is_martian(self) -> bool {
        matches!(
            self,
            NpcKind::MartianOfficer
                | NpcKind::GrayGrunt
                | NpcKind::MartianEngineer
                | NpcKind::GigaZapper
                | NpcKind::MartianWalker
                | NpcKind::MartianProbe
                | NpcKind::MartianDrone
                | NpcKind::ForceBubble
                | NpcKind::MartianTurret
                | NpcKind::BrainScrambler
                | NpcKind::RayGunner
                | NpcKind::ScutlixRider
                | NpcKind::Scutlix
                | NpcKind::MartianSaucer
                | NpcKind::MartianSaucerTurret
                | NpcKind::MartianSaucerCannon
                | NpcKind::MartianSaucerCore
        )
    }

    pub(crate) fn max_hp(self) -> i32 {
        match self {
            NpcKind::Slime => 25,         // NPC.cs:8572
            NpcKind::DemonEye => 60,      // case 2
            NpcKind::WanderingEye => 300, // case 133
            NpcKind::Zombie => 45,
            // `NPC.SetDefaults` case 22 (`NPC.cs:8815`): 250 HP.
            NpcKind::Guide => 250,
            // `SetDefaults` case 17 (`NPC.cs:8989`): 250 HP.
            NpcKind::Merchant => 250,
            // `SetDefaults` case 20 (`NPC.cs:9032`): 250 HP.
            NpcKind::Dryad => 250,
            // `SetDefaults` case 18 (`NPC.cs:9002` region): 250 HP.
            NpcKind::Nurse => 250,
            // `SetDefaults` case 19 (`NPC.cs:9016`): 250 HP.
            NpcKind::ArmsDealer => 250,
            // `SetDefaults` case 38 (`NPC.cs:9019` region): 250 HP.
            NpcKind::Demolitionist => 250,
            // `SetDefaults` case 227 (`NPC.cs:11933`): 250 HP.
            NpcKind::Painter => 250,
            // `SetDefaults` case 228 (`NPC.cs:11950`): 250 HP.
            NpcKind::WitchDoctor => 250,
            // `SetDefaults` case 54 (`NPC.cs:9456`): 250 HP.
            NpcKind::Clothier => 250,
            NpcKind::Angler => 250,
            NpcKind::Wizard => 250,
            NpcKind::BoundWizard => 250,
            NpcKind::Mechanic => 250,
            NpcKind::Stylist => 250,
            NpcKind::WebbedStylist => 250,
            NpcKind::Golfer => 250,
            NpcKind::Zoologist => 250,
            NpcKind::DyeTrader => 250,
            NpcKind::PartyGirl => 250,
            NpcKind::Princess => 250,
            NpcKind::SleepingGolfer => 250,
            NpcKind::TaxCollector => 250,
            NpcKind::BoundMechanic => 250,
            NpcKind::SleepingAngler => 250,
            NpcKind::TravellingMerchant => 250,
            // `SetDefaults` case 4 (`NPC.cs:8710`): 2800 HP.
            NpcKind::EyeOfCthulhu => 2800,
            // `SetDefaults` case 222 (`NPC.cs:11862`): 3400 HP.
            NpcKind::QueenBee => 3400,
            // `SetDefaults` case 35 (`NPC.cs:9172`): 4400 HP.
            NpcKind::SkeletronHead => 4400,
            // `SetDefaults` case 113 (`NPC.cs:10348`): 8000 HP.
            NpcKind::WallOfFlesh => 8000,
            // `NPC.cs`'s 115 (240) / 117-119 (60 — the tail `:10445`,
            // W-A2-3).
            NpcKind::Hungry => 240,
            NpcKind::LeechHead | NpcKind::LeechBody | NpcKind::LeechTail => 60,
            // `SetDefaults` case 134 (`NPC.cs:10667`): 80000 HP — the
            // vanilla's `realLife` pool: every segment of the Destroyer
            // shares the head's life (`NPC.cs:49836-49853`), so the worm
            // has 80000 HP in total, not 80000 per link.
            NpcKind::TheDestroyer => 80000,
            // 135/136 (`NPC.cs:10688`/`:10708`): the same 80000 row — the
            // realLife mirror makes the segment show the head's pool.
            NpcKind::TheDestroyerBody | NpcKind::TheDestroyerTail => 80000,
            // 139 (`NPC.cs:10728`): 200 HP, its own life (no realLife).
            NpcKind::Probe => 200,
            // `SetDefaults` case 127 (`NPC.cs:10562`): 28000 HP.
            NpcKind::SkeletronPrime => 28000,
            // `SetDefaults` cases 125/126.
            NpcKind::Retinazer => 20000,
            NpcKind::Spazmatism => 23000,
            // `SetDefaults` case 262 (`NPC.cs:12385`): 30000 HP.
            NpcKind::Plantera => 30000,
            // The Plantera minions (S1-D-3-wire-plantera-minions):
            // 261 1 HP (`NPC.cs:12371`), 263 4000 (`:12405`),
            // 264 1000 (`:12417`), 265 1 (`:12430`).
            NpcKind::FungiSpore | NpcKind::Spore => 1,
            NpcKind::PlanterasHook => 4000,
            NpcKind::PlanterasTentacle => 1000,
            // `SetDefaults` cases 266/267.
            NpcKind::BrainOfCthulhu => 1250,
            NpcKind::Creeper => 100,
            // The goblins.
            // 250 HP both (`:10255`, `:10280`).
            NpcKind::BoundGoblin => 250,
            NpcKind::GoblinTinkerer => 250,
            // `NPC.cs:53945`'s table: the Skeleton 60, the Undead Miner 70,
            // the Cave Bat 16.
            NpcKind::Skeleton => 60,
            NpcKind::UndeadMiner => 70,
            NpcKind::CaveBat => 16,
            // `SetDefaults` case 51: 34; case 60: 46; case 66: 140.
            NpcKind::JungleBat => 34,
            NpcKind::Hellbat => 46,
            NpcKind::VoodooDemon => 140,
            // `SetDefaults` case 6: 40; case 42: 48; case 23: 26.
            NpcKind::EaterOfSouls => 40,
            NpcKind::Hornet => 48,
            NpcKind::MeteorHead => 26,
            // The volley bees: `SetDefaults` case 210 lifeMax 20
            // (`NPC.cs:11703`), case 211 lifeMax 10 (`:11716`).
            NpcKind::Bee => 20,
            NpcKind::BeeSmall => 10,
            // The casters: `SetDefaults` cases 29/45/32/24.
            // `NPC.cs`'s cases 26 (60) / 27+111 (80) / 28 (110) / 29 (40) /
            // 471 (2000).
            NpcKind::GoblinPeon => 60,
            NpcKind::GoblinThief | NpcKind::GoblinArcher => 80,
            NpcKind::GoblinWarrior => 110,
            NpcKind::GoblinSorcerer => 40,
            NpcKind::GoblinSummoner => 2000,
            // The frost trio: 200/240/220 (`NPC.cs`'s cases 143-145).
            NpcKind::SnowmanGangsta => 200,
            NpcKind::SnowBalla => 240,
            NpcKind::MysterySnowman => 220,
            // The martians: 300/750/400/600/2000 (`NPC.cs`'s cases 383-520).
            NpcKind::MartianOfficer => 300,
            NpcKind::GrayGrunt => 750,
            NpcKind::MartianEngineer => 400,
            NpcKind::GigaZapper => 600,
            NpcKind::MartianWalker => 2000,
            // The probe: 500 (`NPC.cs`'s case 399).
            NpcKind::MartianProbe => 500,
            // The drone: 300 (`NPC.cs`'s case 388).
            NpcKind::MartianDrone => 300,
            // The bubble: 1000 (`NPC.cs:13884`); the turret: 200
            // (`NPC.cs:13929`).
            NpcKind::ForceBubble => 1000,
            NpcKind::MartianTurret => 200,
            // The shooters: 350/350 (`NPC.cs`'s 381/382 grouped row).
            NpcKind::BrainScrambler | NpcKind::RayGunner => 350,
            // The rider 350, the carrier 600 (`NPC.cs`'s cases 390/391).
            NpcKind::ScutlixRider => 350,
            NpcKind::Scutlix => 600,
            // The saucer family: 100/5000/3500/10000 HP (`NPC.cs`'s cases
            // 392-395).
            NpcKind::MartianSaucer => 100,
            NpcKind::MartianSaucerTurret => 5000,
            NpcKind::MartianSaucerCannon => 3500,
            NpcKind::MartianSaucerCore => 10000,
            // The Dutchman 50 (the ship's own bar), its cannons 2000
            // (`NPC.cs`'s cases 491/492).
            NpcKind::FlyingDutchman => 50,
            NpcKind::FlyingDutchmanCannon => 2000,
            // The DD2 goblins: 30/170/560 HP (`NPC.cs`'s cases 552-554).
            NpcKind::DD2GoblinT1 => 30,
            NpcKind::DD2GoblinT2 => 170,
            NpcKind::DD2GoblinT3 => 560,
            // The support rows (`NPC.cs`): the bombers 50/200/700, the
            // javelineers 60/300/1000, the skeletons 25/480.
            NpcKind::DD2GoblinBomberT1 => 50,
            NpcKind::DD2GoblinBomberT2 => 200,
            NpcKind::DD2GoblinBomberT3 => 700,
            NpcKind::DD2JavelinT1 => 60,
            NpcKind::DD2JavelinT2 => 300,
            NpcKind::DD2JavelinT3 => 1000,
            NpcKind::DD2SkeletonT1 => 25,
            NpcKind::DD2SkeletonT3 => 480,
            // The Dark Mage: 800 HP.
            NpcKind::DD2DarkMage => 800,
            // The wyverns: 60/180/600 (`NPC.cs`'s rows).
            NpcKind::DD2WyvernT1 => 60,
            NpcKind::DD2WyvernT2 => 180,
            NpcKind::DD2WyvernT3 => 600,
            // The Dark Mage T3: 4000 (`:16468`).
            NpcKind::DD2DarkMageT3 => 4000,
            // The Wither Beast T3: 1400 (`:16331`).
            NpcKind::DD2WitherBeastT3 => 1400,
            // The Drakin: 900/3000 (`:16416`/`:16433`).
            NpcKind::DD2DrakinT2 => 900,
            NpcKind::DD2DrakinT3 => 3000,
            // The Kobold walkers: 260/800 (`:16382`/`:16399`).
            NpcKind::DD2KoboldWalkerT2 => 260,
            NpcKind::DD2KoboldWalkerT3 => 800,
            // The Kobold gliders: 170/580 (`:16486`/`:16504`).
            NpcKind::DD2KoboldFlyerT2 => 170,
            NpcKind::DD2KoboldFlyerT3 => 580,
            // The Ogres: 5000/13000 (`:16280`/`:16297`).
            NpcKind::DD2OgreT2 => 5000,
            NpcKind::DD2OgreT3 => 13000,
            // Betsy: 50000 HP (`NPC.cs`'s case 551).
            NpcKind::DD2Betsy => 50000,
            // The WitherBeast: 500 HP.
            NpcKind::DD2WitherBeast => 500,
            // The Lunar Pillars: 20000 HP.
            NpcKind::StardustTower
            | NpcKind::NebulaTower
            | NpcKind::SolarTower
            | NpcKind::VortexTower => 20000,
            // Lunatic Cultist (439): 32000 HP (`NPC.cs:14219`). S1-D-3b.
            NpcKind::AncientCultist => 32000,
            // The lane gate: 5 HP (`NPC.cs`'s case 549).
            NpcKind::DD2LanePortal => 5,
            NpcKind::VortexHunter => 1000,
            NpcKind::VortexAlien => 500,
            NpcKind::VortexHeadcrab => 200,
            NpcKind::StardustSpiderBig => 800,
            NpcKind::StardustSpiderSmall => 200,
            NpcKind::StardustSoldier => 700,
            NpcKind::SolarDrakomire => 800,
            NpcKind::SolarDrakomireRider => 800,
            NpcKind::SolarSroller => 700,
            NpcKind::SolarCorite => 600,
            NpcKind::SolarCrawltipedeHead
            | NpcKind::SolarCrawltipedeBody
            | NpcKind::SolarCrawltipedeTail => 10000,
            NpcKind::StardustWormHead => 1200,
            NpcKind::SolarSpearman => 1000,
            NpcKind::SolarCrawltipedeHead
            | NpcKind::SolarCrawltipedeBody
            | NpcKind::SolarCrawltipedeTail => 10000,
            NpcKind::StardustWormHead => 1200,
            NpcKind::SolarSpearman => 1000,
            NpcKind::SolarCrawltipedeHead
            | NpcKind::SolarCrawltipedeBody
            | NpcKind::SolarCrawltipedeTail => 10000,
            NpcKind::StardustWormHead => 1200,
            NpcKind::SolarSpearman => 1000,
            NpcKind::SolarCrawltipedeHead
            | NpcKind::SolarCrawltipedeBody
            | NpcKind::SolarCrawltipedeTail => 10000,
            NpcKind::StardustWormHead => 1200,
            NpcKind::SolarSpearman => 1000,
            NpcKind::SolarCrawltipedeHead
            | NpcKind::SolarCrawltipedeBody
            | NpcKind::SolarCrawltipedeTail => 10000,
            NpcKind::StardustWormHead => 1200,
            NpcKind::SolarSpearman => 1000,
            NpcKind::SolarSelenian => 800,
            NpcKind::NebulaFloater => 1300,
            NpcKind::BrainSuckler => 330,
            NpcKind::Predictor => 700,
            NpcKind::MoonLordHead => 45000,
            NpcKind::MoonLordHand => 25000,
            NpcKind::MoonLordCore => 50000,
            NpcKind::MoonLordFreeEye => 100,
            NpcKind::MoonLordLeechBlob => 400,
            NpcKind::StardustCellBig => 300,
            NpcKind::StardustCellSmall => 300,
            NpcKind::StardustJellyfishBig => 1500,
            NpcKind::NebulaBeast => 850,
            NpcKind::VortexRifleman => 800,
            NpcKind::VortexSoldier => 700,
            // The tavernkeeper 250, the crystal 1000 (`NPC.cs`'s rows).
            NpcKind::DD2Bartender => 250,
            NpcKind::DD2EterniaCrystal => 1000,
            // The pirates: 300/450/225/350/3000 (`NPC.cs`'s cases 212-216).
            NpcKind::PirateDeckhand => 300,
            NpcKind::PirateCaptainsMate => 450,
            NpcKind::PirateArcher => 225,
            NpcKind::PirateCorsair => 350,
            NpcKind::PirateCaptain => 3000,
            // `SetDefaults` 245-249: the body 15000, the head 25000, the
            // fists 10000, the free head 16000.
            NpcKind::Golem => 15000,
            NpcKind::GolemHead => 25000,
            NpcKind::GolemFistLeft | NpcKind::GolemFistRight => 10000,
            NpcKind::GolemHeadFree => 16000,
            // `:17456`: 7000.
            NpcKind::Deerclops => 7000,
            NpcKind::SkeletronHand => 600,
            NpcKind::WallOfFleshEye => 8000,
            NpcKind::PrimeCannon => 7000,
            NpcKind::PrimeSaw => 9000,
            NpcKind::PrimeVice => 9000,
            NpcKind::PrimeLaser => 6000,
            NpcKind::Tim => 200,
            NpcKind::DarkCaster => 50,
            NpcKind::FireImp => 70,
            // The spheres: 1 HP.
            NpcKind::ChaosBall => 1,
            NpcKind::BurningSphere => 1,
            NpcKind::WaterSphere => 1,
            // The swimmers: `SetDefaults` cases 55/58/65.
            NpcKind::Goldfish => 5,
            NpcKind::Piranha => 30,
            NpcKind::Shark => 300,
            // The worms: the segment HP per family (`:8763`/`:8850`/`:8903`).
            NpcKind::DevourerHead | NpcKind::DevourerBody | NpcKind::DevourerTail => 100,
            NpcKind::GiantWormHead | NpcKind::GiantWormBody | NpcKind::GiantWormTail => 30,
            NpcKind::EaterOfWorldsHead
            | NpcKind::EaterOfWorldsBody
            | NpcKind::EaterOfWorldsTail => 150,
            // The wyvern family: every kind lifeMax 4000 (`NPC.cs:9949`/
            // `:9966`/`:9983`/`:10001`/`:10019`/`:10037`) — one shared
            // pool through `realLife` (`:51158-51183`).
            NpcKind::WyvernHead
            | NpcKind::WyvernLegs
            | NpcKind::WyvernBody
            | NpcKind::WyvernBody2
            | NpcKind::WyvernBody3
            | NpcKind::WyvernTail => 4000,
            NpcKind::KingSlime => 2000,
            // `NPC.cs:8710` (the eye) / `:8722` (the servant).
            NpcKind::EyeOfCthulhu => 2800,
            NpcKind::ServantOfCthulhu => 8,
            // DesertDjinn: lifeMax 220 (`NPC.cs:14981`).
            NpcKind::DesertDjinn => 220,
            // The Pumpkin Moon family (`S1-F-13b`): the 305-314 group's five
            // pair rows (`NPC.cs:12929-12977` — 500/400/600/650/450),
            // 315 5000 (`:12978`), 326 1200 (`:13129`), 327 26000
            // (`:13142`), 325 14000 (`:13115`), 329 1800 (`:13175`),
            // 330 1250 (`:13188`).
            NpcKind::Scarecrow1 | NpcKind::Scarecrow6 => 500,
            NpcKind::Scarecrow2 | NpcKind::Scarecrow7 => 400,
            NpcKind::Scarecrow3 | NpcKind::Scarecrow8 => 600,
            NpcKind::Scarecrow4 | NpcKind::Scarecrow9 => 650,
            NpcKind::Scarecrow5 | NpcKind::Scarecrow10 => 450,
            NpcKind::HeadlessHorseman => 5000,
            NpcKind::MourningWood => 14000,
            NpcKind::Splinterling => 1200,
            NpcKind::Pumpking => 26000,
            NpcKind::Hellhound => 1800,
            NpcKind::Poltergeist => 1250,
            // S1-D-3ff wave A — the `SetDefaults` lifeMax rows: 16 90
            // (`NPC.cs:8971`), 59 50 (`:9523`), 71 150 (`:9705`), 81 170
            // (`:9845`), 138 180 (`:10755`), 147 30 (`:10854` region),
            // 183 200 (`:11339` region), 184 60 (`:11353` region),
            // 225 35 (`:11905` region); 190 65 (`:11434`), 191/193/194
            // 60 (`:11447`/`:11473`/`:11486`), 192 50 (`:11460`); 52 500
            // (`:9425`), 53 200 (`:9439`), 132/187 40 (`:10640`/
            // `:11395`), 161/186/223 50 (`:11024`/`:11382`/`:11879`
            // region), 188/189 45 (`:11408` region/`:11421` region),
            // 200 38 (`:11566` region).
            NpcKind::MotherSlime => 90,
            NpcKind::LavaSlime => 50,
            NpcKind::DungeonSlime => 150,
            NpcKind::CorruptSlime => 170,
            NpcKind::IlluminantSlime => 180,
            NpcKind::IceSlime => 30,
            NpcKind::Crimslime => 200,
            NpcKind::SpikedIceSlime => 60,
            NpcKind::UmbrellaSlime => 35,
            NpcKind::CataractEye => 65,
            NpcKind::SleepyEye | NpcKind::GreenEye | NpcKind::PurpleEye => 60,
            NpcKind::DialatedEye => 50,
            NpcKind::DoctorBones => 500,
            NpcKind::TheGroom => 200,
            NpcKind::BaldZombie | NpcKind::SlimedZombie => 40,
            NpcKind::ZombieEskimo | NpcKind::PincushionZombie | NpcKind::ZombieRaincoat => 50,
            NpcKind::SwampZombie | NpcKind::TwiggyZombie => 45,
            NpcKind::FemaleZombie => 38,
            // S1-D-3z — the holiday zombies' `SetDefaults` lifeMax rows:
            // 319 40 (`NPC.cs:13040`), 320 60 (`:13054`), 321 34 (`:13068`),
            // 331/332 45 (`:13211`/`:13224` — the base Zombie's row).
            NpcKind::ZombieDoctor => 40,
            NpcKind::ZombieSuperman => 60,
            NpcKind::ZombiePixie => 34,
            NpcKind::ZombieXmas | NpcKind::ZombieSweater => 45,
            // S1-D-3z — the blood-moon pair's `SetDefaults` lifeMax rows:
            // 489 75 (`NPC.cs:14612`), 490 50 (`:14626`).
            NpcKind::BloodZombie => 75,
            NpcKind::Drippler => 50,
            // The Frost Moon family, slice A (`S1-F-13c`): the 338-340
            // group's three rows (`NPC.cs:13294`/`:13301`/`:13309` —
            // 600/700/500), 341 900 (`:13321`), 342 750 (`:13334`),
            // 343 3500 (`:13347`), 344 13000 (`:13362`).
            NpcKind::ZombieElf => 600,
            NpcKind::ZombieElfBeard => 700,
            NpcKind::ZombieElfGirl => 500,
            NpcKind::PresentMimic => 900,
            NpcKind::GingerbreadMan => 750,
            NpcKind::Yeti => 3500,
            NpcKind::Everscream => 13000,
            // The Frost Moon family (`S1-F-13c` slice B): 345 34000
            // (`NPC.cs:13376`), 346 18000 (`:13393`), 347 1200 (`:13407`),
            // the nutcracker pair 1800 (`:13423`/`:13437`), 350 900
            // (`:13451`), 351 2500 (`:13464`), 352 450 (`:13478`).
            NpcKind::IceQueen => 34000,
            NpcKind::SantaNK1 => 18000,
            NpcKind::ElfCopter => 1200,
            NpcKind::Nutcracker | NpcKind::NutcrackerSpinning => 1800,
            NpcKind::ElfArcher => 900,
            NpcKind::Krampus => 2500,
            NpcKind::Flocko => 450,
            // S1-D-3 W-9 fishron wiring — the `lifeMax` rows: 370 60000
            // (`NPC.cs:13708`), 371 1 (`:13726`), 372/373 100
            // (`:13741`/`:13755`).
            NpcKind::DukeFishron => 60000,
            NpcKind::DetonatingBubble => 1,
            NpcKind::Sharkron | NpcKind::Sharkron2 => 100,
            // The small-kinds wiring wave: 72 100 (`NPC.cs:9722`), 250
            // 300 (`:12228`), 288 200 (`:12745`), 384 1000 (`:13884`).
            NpcKind::BlazingWheel => 100,
            NpcKind::AngryNimbus => 300,
            NpcKind::DungeonSpirit => 200,
        }
    }

    /// `NPC.defDamage`'s stand-in — the kind's `SetDefaults` contact
    /// damage. `pub(crate)` since S1-D-3: the aiStyle-3 ladder's charge
    /// family (`NPC.cs:58266`/`:58308`) restores/boosts it from `npc_ai`.
    pub(crate) fn contact_damage(self) -> i32 {
        match self {
            NpcKind::Slime => 7,         // NPC.cs:8571
            NpcKind::DemonEye => 18,     // case 2
            NpcKind::WanderingEye => 40, // case 133
            NpcKind::Zombie => 14,
            // `SetDefaults` case 17: damage 10 — the town NPCs never hurt
            // (the contact gate skips them).
            NpcKind::Merchant => 10,
            NpcKind::Dryad => 10,
            NpcKind::Nurse => 10,
            NpcKind::ArmsDealer => 10,
            NpcKind::Demolitionist => 10,
            NpcKind::Painter => 10,
            NpcKind::WitchDoctor => 10,
            NpcKind::Clothier => 10,
            NpcKind::Angler => 10,
            NpcKind::Wizard => 10,
            NpcKind::BoundWizard => 10,
            NpcKind::Mechanic => 10,
            NpcKind::Stylist => 10,
            NpcKind::WebbedStylist => 10,
            NpcKind::Golfer => 10,
            NpcKind::Zoologist => 10,
            NpcKind::DyeTrader => 10,
            NpcKind::PartyGirl => 10,
            NpcKind::Princess => 10,
            NpcKind::SleepingGolfer => 10,
            NpcKind::TaxCollector => 10,
            NpcKind::BoundMechanic => 10,
            NpcKind::SleepingAngler => 10,
            NpcKind::TravellingMerchant => 10,
            NpcKind::EyeOfCthulhu => 15,
            NpcKind::QueenBee => 30,
            NpcKind::SkeletronHead => 32,
            NpcKind::WallOfFlesh => 50,
            // `NPC.cs`'s 115 (30) / 117 (26) / 118 (22) / 119 (18,
            // `:10443` — W-A2-3).
            NpcKind::Hungry => 30,
            NpcKind::LeechHead => 26,
            NpcKind::LeechBody => 22,
            NpcKind::LeechTail => 18,
            NpcKind::TheDestroyer => 70,
            // `NPC.cs`'s 135 (55, `:10686`) / 136 (40, `:10706`) / 139 (50,
            // `:10726`).
            NpcKind::TheDestroyerBody => 55,
            NpcKind::TheDestroyerTail => 40,
            NpcKind::Probe => 50,
            NpcKind::SkeletronPrime => 47,
            NpcKind::Retinazer => 45,
            NpcKind::Spazmatism => 50,
            NpcKind::Plantera => 50,
            // The Plantera minions (S1-D-3-wire-plantera-minions):
            // 261 80 (`NPC.cs:12369`), 263 60 (`:12403`),
            // 264 60 (`:12415`), 265 70 (`:12428`).
            NpcKind::FungiSpore => 80,
            NpcKind::PlanterasHook | NpcKind::PlanterasTentacle => 60,
            NpcKind::Spore => 70,
            NpcKind::BrainOfCthulhu => 30,
            NpcKind::Creeper => 20,
            // The `friendly` pair (`:10246`/`:10272`) — the contact gate
            // excludes them; 0 stands for the vanilla's unused dmg 10.
            NpcKind::BoundGoblin => 0,
            NpcKind::GoblinTinkerer => 0,
            // `SetDefaults` case 21: damage 20; case 44: 22; case 49: 13.
            NpcKind::Skeleton => 20,
            NpcKind::UndeadMiner => 22,
            NpcKind::CaveBat => 13,
            // `SetDefaults` case 51: damage 20; case 60: 35; case 66: 32.
            NpcKind::JungleBat => 20,
            NpcKind::Hellbat => 35,
            NpcKind::VoodooDemon => 32,
            // `SetDefaults` case 6: damage 22; case 42: 26; case 23: 40.
            NpcKind::EaterOfSouls => 22,
            NpcKind::Hornet => 26,
            NpcKind::MeteorHead => 40,
            // The volley bees: `SetDefaults` case 210 damage 20
            // (`NPC.cs:11701`), case 211 damage 15 (`:11714`).
            NpcKind::Bee => 20,
            NpcKind::BeeSmall => 15,
            // The casters: `SetDefaults` damage 20/20/20/30.
            // `NPC.cs`'s cases 26 (12) / 27+111 (20) / 28 (25) / 29 (20) /
            // 471 (80).
            NpcKind::GoblinPeon => 12,
            NpcKind::GoblinThief | NpcKind::GoblinArcher | NpcKind::GoblinSorcerer => 20,
            NpcKind::GoblinWarrior => 25,
            NpcKind::GoblinSummoner => 80,
            // DesertDjinn: damage 40 (`NPC.cs:14979`).
            NpcKind::DesertDjinn => 40,
            // The frost trio: 50/65/55.
            NpcKind::SnowmanGangsta => 50,
            NpcKind::SnowBalla => 65,
            NpcKind::MysterySnowman => 55,
            // The martians: 75/80/40/75/60.
            NpcKind::MartianOfficer => 75,
            NpcKind::GrayGrunt => 80,
            NpcKind::MartianEngineer => 40,
            NpcKind::GigaZapper => 75,
            NpcKind::MartianWalker => 60,
            // The probe: 5 (`NPC.cs`'s case 399).
            NpcKind::MartianProbe => 5,
            // The drone: 60 (`NPC.cs`'s case 388 — the explode arm's 80
            // rides damage_override).
            NpcKind::MartianDrone => 60,
            // The bubble: no damage row — the head default 0
            // (`NPC.cs:8563`; case 384 `:13882-13894`); the turret: 10
            // (`NPC.cs:13927`).
            NpcKind::ForceBubble => 0,
            NpcKind::MartianTurret => 10,
            // The shooters: 50/50 (`NPC.cs`'s 381/382 grouped row).
            NpcKind::BrainScrambler | NpcKind::RayGunner => 50,
            // The rider 65, the carrier 85 (`NPC.cs`'s cases 390/391).
            NpcKind::ScutlixRider => 65,
            NpcKind::Scutlix => 85,
            // The saucer family: 50/60/60/80 (`NPC.cs`'s cases 392-395).
            NpcKind::MartianSaucer => 50,
            NpcKind::MartianSaucerTurret => 60,
            NpcKind::MartianSaucerCannon => 60,
            NpcKind::MartianSaucerCore => 80,
            // The Dutchman 0 (contactless), its cannons 30.
            NpcKind::FlyingDutchman => 0,
            NpcKind::FlyingDutchmanCannon => 30,
            // The DD2 goblins: 18/46/70 dmg (`NPC.cs`'s cases 552-554).
            NpcKind::DD2GoblinT1 => 18,
            NpcKind::DD2GoblinT2 => 46,
            NpcKind::DD2GoblinT3 => 70,
            // The support's damage: 26/55/75, 30/60/80, 18/70.
            NpcKind::DD2GoblinBomberT1 => 26,
            NpcKind::DD2GoblinBomberT2 => 55,
            NpcKind::DD2GoblinBomberT3 => 75,
            NpcKind::DD2JavelinT1 => 30,
            NpcKind::DD2JavelinT2 => 60,
            NpcKind::DD2JavelinT3 => 80,
            NpcKind::DD2SkeletonT1 => 18,
            NpcKind::DD2SkeletonT3 => 70,
            // The Dark Mage: dmg 40.
            NpcKind::DD2DarkMage => 40,
            // Betsy: dmg 80.
            NpcKind::DD2Betsy => 80,
            // The WitherBeast: dmg 50.
            NpcKind::DD2WitherBeast => 50,
            NpcKind::DD2WyvernT1 => 30,
            NpcKind::DD2WyvernT2 => 75,
            NpcKind::DD2WyvernT3 => 100,
            NpcKind::DD2DarkMageT3 => 90,
            NpcKind::DD2WitherBeastT3 => 80,
            NpcKind::DD2DrakinT2 => 60,
            NpcKind::DD2DrakinT3 => 90,
            NpcKind::DD2KoboldWalkerT2 => 60,
            NpcKind::DD2KoboldWalkerT3 => 80,
            NpcKind::DD2KoboldFlyerT2 => 50,
            NpcKind::DD2KoboldFlyerT3 => 80,
            NpcKind::DD2OgreT2 => 70,
            NpcKind::DD2OgreT3 => 90,
            // The Lunar Pillars: dmg 0.
            NpcKind::StardustTower
            | NpcKind::NebulaTower
            | NpcKind::SolarTower
            | NpcKind::VortexTower => 0,
            // Lunatic Cultist (439): damage 50 (`NPC.cs:14217`). S1-D-3b.
            NpcKind::AncientCultist => 50,
            // The lane gate: damage 0 (`NPC.cs`'s case 549).
            NpcKind::DD2LanePortal => 0,
            // The vortex trio's damage: the hunter's 100, the alien's 75,
            // the headcrab's 50.
            NpcKind::StardustSpiderBig => 70,
            NpcKind::StardustSpiderSmall => 80,
            NpcKind::StardustSoldier => 80,
            NpcKind::SolarDrakomire => 55,
            NpcKind::SolarDrakomireRider => 80,
            NpcKind::SolarSroller => 80,
            NpcKind::SolarCorite => 70,
            NpcKind::SolarCrawltipedeHead => 120,
            NpcKind::SolarCrawltipedeBody => 80,
            NpcKind::SolarCrawltipedeTail => 50,
            NpcKind::StardustWormHead => 80,
            NpcKind::SolarSpearman => 80,
            NpcKind::SolarCrawltipedeHead => 120,
            NpcKind::SolarCrawltipedeBody => 80,
            NpcKind::SolarCrawltipedeTail => 50,
            NpcKind::StardustWormHead => 80,
            NpcKind::SolarSpearman => 80,
            NpcKind::SolarCrawltipedeHead => 120,
            NpcKind::SolarCrawltipedeBody => 80,
            NpcKind::SolarCrawltipedeTail => 50,
            NpcKind::StardustWormHead => 80,
            NpcKind::SolarSpearman => 80,
            NpcKind::SolarCrawltipedeHead => 120,
            NpcKind::SolarCrawltipedeBody => 80,
            NpcKind::SolarCrawltipedeTail => 50,
            NpcKind::StardustWormHead => 80,
            NpcKind::SolarSpearman => 80,
            NpcKind::SolarCrawltipedeHead => 120,
            NpcKind::SolarCrawltipedeBody => 80,
            NpcKind::SolarCrawltipedeTail => 50,
            NpcKind::StardustWormHead => 80,
            NpcKind::SolarSpearman => 80,
            NpcKind::SolarSelenian => 90,
            NpcKind::NebulaFloater => 75,
            NpcKind::BrainSuckler => 70,
            NpcKind::Predictor => 80,
            NpcKind::VortexHunter => 100,
            NpcKind::VortexAlien => 75,
            NpcKind::VortexHeadcrab => 50,
            NpcKind::MoonLordHead | NpcKind::MoonLordHand | NpcKind::MoonLordCore => 0,
            NpcKind::MoonLordFreeEye => 60,
            NpcKind::MoonLordLeechBlob => 0,
            NpcKind::StardustCellBig => 120,
            NpcKind::StardustCellSmall | NpcKind::StardustJellyfishBig => 70,
            NpcKind::NebulaBeast => 90,
            NpcKind::VortexRifleman => 100,
            NpcKind::VortexSoldier => 90,
            // The tavernkeeper 10, the crystal 0 (its contact damage is 0).
            NpcKind::DD2Bartender => 10,
            NpcKind::DD2EterniaCrystal => 0,
            // The pirates: 35/50/30/35/70 (`NPC.cs`'s cases 212-216).
            NpcKind::PirateDeckhand => 35,
            NpcKind::PirateCaptainsMate => 50,
            NpcKind::PirateArcher => 30,
            NpcKind::PirateCorsair => 35,
            NpcKind::PirateCaptain => 70,
            // `SetDefaults` 245-249: 72/64/59/59/80.
            NpcKind::Golem => 72,
            NpcKind::GolemHead => 64,
            NpcKind::GolemFistLeft | NpcKind::GolemFistRight => 59,
            NpcKind::GolemHeadFree => 80,
            // `:17455`: 20.
            NpcKind::Deerclops => 20,
            NpcKind::SkeletronHand => 20,
            NpcKind::WallOfFleshEye => 50,
            NpcKind::PrimeCannon => 30,
            NpcKind::PrimeSaw => 56,
            NpcKind::PrimeVice => 52,
            NpcKind::PrimeLaser => 29,
            NpcKind::Tim => 20,
            NpcKind::DarkCaster => 20,
            NpcKind::FireImp => 30,
            // The spheres: damage 20/30/20.
            NpcKind::ChaosBall => 20,
            NpcKind::BurningSphere => 30,
            NpcKind::WaterSphere => 20,
            // The swimmers: damage 0/25/40 — the Goldfish never hurts.
            NpcKind::Goldfish => 0,
            NpcKind::Piranha => 25,
            NpcKind::Shark => 40,
            // The worms: the head hits hardest, the tail softest
            // (`:8763`'s 31/16/13, `:8850`'s 8/4/4, `:8903`'s 22/13/11).
            NpcKind::DevourerHead => 31,
            NpcKind::DevourerBody => 16,
            NpcKind::DevourerTail => 13,
            NpcKind::GiantWormHead => 8,
            NpcKind::GiantWormBody | NpcKind::GiantWormTail => 4,
            NpcKind::EaterOfWorldsHead => 22,
            NpcKind::EaterOfWorldsBody => 13,
            NpcKind::EaterOfWorldsTail => 11,
            // The wyvern: the head 80, every segment 40 (`NPC.cs:9947`/
            // `:9964`/`:9981`/`:9999`/`:10017`/`:10035`).
            NpcKind::WyvernHead => 80,
            NpcKind::WyvernLegs
            | NpcKind::WyvernBody
            | NpcKind::WyvernBody2
            | NpcKind::WyvernBody3
            | NpcKind::WyvernTail => 40,
            NpcKind::Guide => 0,
            NpcKind::KingSlime => 40,
            // `NPC.cs:8707` (the eye damage 15) / `:8720` (the servant 12).
            NpcKind::EyeOfCthulhu => 15,
            NpcKind::ServantOfCthulhu => 12,
            // The Pumpkin Moon family (`S1-F-13b`): the 305-314 group's five
            // pair rows (`NPC.cs:12929-12977` — 60/52/78/66/52),
            // 315 130 (`:12978`), 326 100 (`:13129`), 327 50 (`:13142`),
            // 325 120 (`:13115`), 329 80 (`:13175`), 330 90 (`:13188`).
            NpcKind::Scarecrow1 | NpcKind::Scarecrow6 => 60,
            NpcKind::Scarecrow2 | NpcKind::Scarecrow7 => 52,
            NpcKind::Scarecrow3 | NpcKind::Scarecrow8 => 78,
            NpcKind::Scarecrow4 | NpcKind::Scarecrow9 => 66,
            NpcKind::Scarecrow5 | NpcKind::Scarecrow10 => 52,
            NpcKind::HeadlessHorseman => 130,
            NpcKind::MourningWood => 120,
            NpcKind::Splinterling => 100,
            NpcKind::Pumpking => 50,
            NpcKind::Hellhound => 80,
            NpcKind::Poltergeist => 90,
            // S1-D-3ff wave A — the `SetDefaults` damage rows: 16 20
            // (`NPC.cs:8969`), 59 15 (`:9521`), 71 30 (`:9703`), 81 55
            // (`:9843`), 138 70 (`:10753`), 147 8 (`:10852` region),
            // 183 60 (`:11337` region), 184 12 (`:11351` region), 225 10
            // (`:11903` region); 190/192 18 (`:11432`/`:11458`), 191 16
            // (`:11445`), 193 20 (`:11471`), 194 14 (`:11484`); 52 20
            // (`:9423`), 53 14 (`:9437`), 132 15 (`:10638`), 161/186/189/
            // 223 16 (`:11022`/`:11380`/`:11419` region/`:11877` region),
            // 187/188 13 (`:11393`/`:11406` region), 200 12 (`:11564`
            // region).
            NpcKind::MotherSlime => 20,
            NpcKind::LavaSlime => 15,
            NpcKind::DungeonSlime => 30,
            NpcKind::CorruptSlime => 55,
            NpcKind::IlluminantSlime => 70,
            NpcKind::IceSlime => 8,
            NpcKind::Crimslime => 60,
            NpcKind::SpikedIceSlime => 12,
            NpcKind::UmbrellaSlime => 10,
            NpcKind::CataractEye | NpcKind::DialatedEye => 18,
            NpcKind::SleepyEye => 16,
            NpcKind::GreenEye => 20,
            NpcKind::PurpleEye => 14,
            NpcKind::DoctorBones => 20,
            NpcKind::TheGroom => 14,
            NpcKind::BaldZombie => 15,
            NpcKind::ZombieEskimo
            | NpcKind::PincushionZombie
            | NpcKind::TwiggyZombie
            | NpcKind::ZombieRaincoat => 16,
            NpcKind::SlimedZombie | NpcKind::SwampZombie => 13,
            NpcKind::FemaleZombie => 12,
            // S1-D-3z — the holiday zombies' `SetDefaults` damage rows:
            // 319 20 (`NPC.cs:13038`), 320 15 (`:13052`), 321 20 (`:13066`),
            // 331/332 14 (`:13209`/`:13222` — the base Zombie's row).
            NpcKind::ZombieDoctor | NpcKind::ZombiePixie => 20,
            NpcKind::ZombieSuperman => 15,
            NpcKind::ZombieXmas | NpcKind::ZombieSweater => 14,
            // S1-D-3z — the blood-moon pair's `SetDefaults` damage rows:
            // 489 20 (`NPC.cs:14610`), 490 28 (`:14624`).
            NpcKind::BloodZombie => 20,
            NpcKind::Drippler => 28,
            // The Frost Moon family, slice A (`S1-F-13c`): the 338-340
            // group's three rows (`NPC.cs:13292`/`:13299`/`:13307` —
            // 65/52/78), 341 100 (`:13319`), 342 90 (`:13332`), 343 140
            // (`:13345`), 344 110 (`:13360`).
            NpcKind::ZombieElf => 65,
            NpcKind::ZombieElfBeard => 52,
            NpcKind::ZombieElfGirl => 78,
            NpcKind::PresentMimic => 100,
            NpcKind::GingerbreadMan => 90,
            NpcKind::Yeti => 140,
            NpcKind::Everscream => 110,
            // The Frost Moon family (`S1-F-13c` slice B): 345/346 120
            // (`NPC.cs:13374`/`:13391`), 347 60 (`:13405`), 348 80
            // (`:13421`), 349/351 100 (`:13435`/`:13462`), 350 70
            // (`:13449`), 352 75 (`:13476`).
            NpcKind::IceQueen | NpcKind::SantaNK1 => 120,
            NpcKind::ElfCopter => 60,
            NpcKind::Nutcracker => 80,
            NpcKind::NutcrackerSpinning | NpcKind::Krampus => 100,
            NpcKind::ElfArcher => 70,
            NpcKind::Flocko => 75,
            // S1-D-3 W-9 fishron wiring — the `damage` rows: 370 100
            // (`NPC.cs:13706` — the AI rewrites it per tick from
            // `defDamage`), 371 100 (`:13724`), 372 100 (`:13739`),
            // 373 120 (`:13753`).
            NpcKind::DukeFishron | NpcKind::DetonatingBubble | NpcKind::Sharkron => 100,
            NpcKind::Sharkron2 => 120,
            // The small-kinds wiring wave: 72 24 (`NPC.cs:9720`), 250 50
            // (`:12226`), 288 70 (`:12743`), 384 keeps the `SetDefaults`
            // head's `damage = 0` (`:8563`, the branch writes none).
            NpcKind::BlazingWheel => 24,
            NpcKind::AngryNimbus => 50,
            NpcKind::DungeonSpirit => 70,
        }
    }
}

/// `NPC.SetDefaults(int netID)`'s resolution of a modern record
/// (`LoadNPCs:2896`): the vanilla net id maps to the kind through the same
/// table the bestiary credits from. `None` for an id the port does not model.
pub fn kind_from_net_id(net_id: i32) -> Option<(NpcKind, SlimeVariant)> {
    crate::bestiary::ENTRIES
        .iter()
        .find(|e| e.net_id == net_id)
        .map(|e| (e.kind, e.variant.unwrap_or(SlimeVariant::Blue)))
}

/// `NPCID.FromLegacyName` (`LoadNPCs:2900`) narrowed to the port's family: the
/// legacy name is the English base name (`NPCID.cs:12085+`'s
/// `LegacyNameToIdMap`), which the port's bestiary rows already carry. The
/// "Little/Big <name>" sub-variant names and the unported NPCs map to nothing
/// here — those records are parsed and discarded (recorded deviation).
pub fn kind_from_legacy_name(name: &str) -> Option<(NpcKind, SlimeVariant)> {
    crate::bestiary::ENTRIES
        .iter()
        .find(|e| e.name == name)
        .map(|e| (e.kind, e.variant.unwrap_or(SlimeVariant::Blue)))
}

/// The bestiary info-page stat rows for one entry — the same tables
/// `Npc::max_hp`/`defense`/`value`/`knock_back_resist` read, exposed for the
/// UI without constructing an `Npc` (`EntryStats` mirrors
/// `UIBestiaryEntryInfoPage`'s `Stat_*` lines).
pub struct EntryStats {
    pub hp: i32,
    pub damage: i32,
    pub defense: i32,
    pub value: f32,
    pub kb: f32,
}

/// The stat rows for one kind/variant pair; the slime variants ride their
/// `SlimeStats`, the other kinds the `Npc` accessors' tables.
pub(crate) fn entry_stats(kind: NpcKind, variant: SlimeVariant) -> EntryStats {
    // The accessors live on `Npc` instances; the private ctor builds the
    // shell and the variant rides the only other field the tables consult.
    let mut shell = Npc::new(kind, 0.0, 0.0, 1, 0.0);
    shell.slime = variant;
    EntryStats {
        hp: kind.max_hp(),
        damage: if kind == NpcKind::Slime {
            variant.stats().damage
        } else {
            kind.contact_damage()
        },
        defense: shell.defense(),
        value: shell.value(),
        kb: shell.knock_back_resist(),
    }
}
