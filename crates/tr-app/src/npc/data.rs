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
            // W4-c6 style-59 wiring: the blade keeps its vanilla id 328
            // (`NPC.cs:13158-13174`; the slot sits between the Pumpking's
            // 327 and the Hellhound's 329 — free in the port's id space).
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
            // W4-d38 b7 — vanilla ids kept (free protocol slots).
            NpcKind::SlimeRibbonWhite => 333,
            NpcKind::SlimeRibbonYellow => 334,
            NpcKind::SlimeRibbonGreen => 335,
            NpcKind::SlimeRibbonRed => 336,
            NpcKind::BunnyXmas => 337,
            // W4-d38 b6 — vanilla ids kept (free protocol slots).
            NpcKind::DemonEyeSpaceship => 318,
            NpcKind::SkeletonTopHat => 322,
            NpcKind::SkeletonAstonaut => 323,
            NpcKind::SkeletonAlien => 324,
            NpcKind::PumpkingBlade => 328,
            // W4-d38 b5 — vanilla ids kept (free protocol slots).
            NpcKind::SlimeMasked => 302,
            NpcKind::BunnySlimed => 303,
            NpcKind::HoppinJack => 304,
            NpcKind::Ghost => 316,
            NpcKind::DemonEyeOwl => 317,
            // W4-d38 b4 — vanilla ids kept (free protocol slots).
            NpcKind::BirdBlue => 297,
            NpcKind::BirdRed => 298,
            NpcKind::Squirrel => 299,
            NpcKind::Mouse => 300,
            NpcKind::Raven => 301,
            // W4-d38 b3 — vanilla ids kept (free protocol slots).
            NpcKind::TacticalSkeleton => 292,
            NpcKind::SkeletonCommando => 293,
            NpcKind::AngryBonesBig => 294,
            NpcKind::AngryBonesBigMuscle => 295,
            NpcKind::AngryBonesBigHelmet => 296,
            // W4-d38 b2 — vanilla ids kept (free protocol slots).
            NpcKind::DiabolistWhite => 286,
            NpcKind::BoneLee => 287,
            NpcKind::GiantCursedSkull => 289,
            NpcKind::Paladin => 290,
            NpcKind::SkeletonSniper => 291,
            // W4-d38 b1 — vanilla ids kept (free protocol slots).
            NpcKind::RaggedCaster => 281,
            NpcKind::RaggedCasterOpenCoat => 282,
            NpcKind::Necromancer => 283,
            NpcKind::NecromancerArmored => 284,
            NpcKind::DiabolistRed => 285,
            // The man-eater family (W4-d23, aiStyle 13): the four vanilla
            // ids 43/56/101/175 are the QueenBee/Creeper/DD2JavelinT1/
            // DD2DrakinT3 protocol slots (`to_u16`'s bijection guard), so
            // they take the next-free port-local band 354-357; the two
            // bulbs keep their vanilla ids (259/260 free).
            // `NPC.cs:9293-9307` / `:9474-9488` / `:10186-10201` /
            // `:11208-11222` / `:12332-12347` / `:12348-12363`.
            // E8 merge (d37 tables): AngryTrapper rides 743 (the vanilla
            // 175 is d23's cs_id; the 357 slot is d37's SeaSnail); the
            // ManEater/Snatcher/Clinger dups drop (d34@269/276 + d35@718
            // own them), the FungiBulb/GiantFungiBulb dups drop (d37@385/
            // 386 own them).
            NpcKind::AngryTrapper => 743,
            // The aiStyle-9 spit/flare family (W4-d23): VileSpit 112 was
            // rowed by d35 at protocol 455 (the roster's claim — the
            // vanilla 112 slot is the AncientCultist's, `to_u16`'s guard).
            // SolarFlare 516 / VileSpitEoW 666 are UNROWED: the reconcile
            // directive parks them on the 400+ free band (408/411) even
            // though their vanilla slots are free — the d37-354-399
            // collision's union numbering (research/108 addendum).
            // `NPC.cs:10326-10341` / `:15394-15408` / `:17418-17433`.
            // E8 merge: VileSpit's 455 is d35's 724 (the dup drop);
            // VileSpitEoW collapses to d43's VileSpitEaterOfWorlds@666
            // (the same vanilla 666); SolarFlare 408 keeps (d41's 516
            // dup dropped at its merge).
            NpcKind::SolarFlare => 408,
            // The aiStyle-14 flyer family (W4-d23): the five ROWED kinds
            // adopt their roster protocol ids (Harpy 48 → d34's 275,
            // GiantBat 93 → d35's 441, Slimer 121 → d35's 458,
            // IlluminantBat 137 → d35's 460, FlyingSnake 226 → d37's 360
            // — the vanilla slots are the LeechHead/FlyingDutchman/
            // StardustCellBig/SolarDrakomireRider/WyvernBody2's, `to_u16`'s
            // bijection guard). The nine UNROWED kinds take the 400+ free
            // band (Demon 62 → 400, IceBat 150 → 401, Lavabat 151 → 402,
            // GiantFlyingFox 152 → 403, RedDevil 156 → 404, VampireBat
            // 158 → 405, Vampire 159 → 406, SporeBat 634 → 407,
            // QueenSlimeMinionPurple 660 → 409) even where the vanilla
            // slot is free (634/660) — the d37-354-399 collision's union
            // numbering (research/108 addendum). `NPC.cs:9360-9372` /
            // `:9567-9581` / `:10047-10060` / `:10466-10478` /
            // `:10735-10747` / `:10885-10929` / `:10974-10988` /
            // `:11911-11924` / `:15466-15493` / `:17161-17174` /
            // `:17338-17350`.
            // E8 merge (d37 tables): the Harpy/Demon/GiantBat/Slimer/
            // IlluminantBat/FlyingSnake dups drop (d34@275/279 + d35@710/
            // 727/729 + d37@360 own them); the IceBat/Lavabat/GiantFlyingFox/
            // RedDevil/VampireBat/Vampire dups drop (g8@757-762 own them);
            // SporeBat rides 634, QueenSlimeMinionPurple rides 660 (the
            // d43 stand-ins drop at its merge).
            NpcKind::SporeBat => 634,
            NpcKind::QueenSlimeMinionPurple => 660,
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
            // W4-d40: ArmedZombie, ArmedZombieEskimo, ArmedZombiePincussion, ArmedZombieSlimed, ArmedZombieSwamp keep their vanilla ids (free in the protocol space).
            NpcKind::ArmedZombie => 430,
            NpcKind::ArmedZombieEskimo => 431,
            NpcKind::ArmedZombiePincussion => 432,
            NpcKind::ArmedZombieSlimed => 433,
            NpcKind::ArmedZombieSwamp => 434,
            // W4-d40: ArmedZombieTwiggy, ArmedZombieCenx, CultistTablet, CultistDevote, CultistBossClone keep their vanilla ids (free in the protocol space).
            NpcKind::ArmedZombieTwiggy => 435,
            NpcKind::ArmedZombieCenx => 436,
            NpcKind::CultistTablet => 437,
            NpcKind::CultistDevote => 438,
            NpcKind::CultistBossClone => 440,
            // W4-d40: GoldBird, GoldBunny, GoldButterfly, GoldFrog, GoldGrasshopper keep their vanilla ids (free in the protocol space).
            NpcKind::GoldBird => 442,
            NpcKind::GoldBunny => 443,
            NpcKind::GoldButterfly => 444,
            NpcKind::GoldFrog => 445,
            NpcKind::GoldGrasshopper => 446,
            // W4-d40: GoldMouse, GoldWorm, BoneThrowingSkeleton, BoneThrowingSkeleton2, BoneThrowingSkeleton3 keep their vanilla ids (free in the protocol space).
            NpcKind::GoldMouse => 447,
            NpcKind::GoldWorm => 448,
            NpcKind::BoneThrowingSkeleton => 449,
            NpcKind::BoneThrowingSkeleton2 => 450,
            NpcKind::BoneThrowingSkeleton3 => 451,
            // W4-d40: BoneThrowingSkeleton4, SkeletonMerchant, CultistDragonHead, CultistDragonBody1, CultistDragonBody2 keep their vanilla ids (free in the protocol space).
            NpcKind::BoneThrowingSkeleton4 => 452,
            NpcKind::SkeletonMerchant => 453,
            NpcKind::CultistDragonHead => 454,
            NpcKind::CultistDragonBody1 => 455,
            NpcKind::CultistDragonBody2 => 456,
            // W4-d40: CultistDragonBody3, CultistDragonBody4, CultistDragonTail, Butcher, CreatureFromTheDeep keep their vanilla ids (free in the protocol space).
            NpcKind::CultistDragonBody3 => 457,
            NpcKind::CultistDragonBody4 => 458,
            NpcKind::CultistDragonTail => 459,
            NpcKind::Butcher => 460,
            NpcKind::CreatureFromTheDeep => 461,
            // W4-d40: Fritz, Nailhead, CrimsonBunny, Psycho keep their vanilla ids (free in the protocol space).
            // (CrimsonGoldfish 465 dropped - c4 owns @465, d37 tables.)
            NpcKind::Fritz => 462,
            NpcKind::Nailhead => 463,
            NpcKind::CrimsonBunny => 464,
            NpcKind::Psycho => 466,
            // W4-d40: DeadlySphere, DrManFly, ThePossessed, CrimsonPenguin, ShadowFlameApparition keep their vanilla ids (free in the protocol space).
            NpcKind::DeadlySphere => 467,
            NpcKind::DrManFly => 468,
            NpcKind::ThePossessed => 469,
            NpcKind::CrimsonPenguin => 470,
            NpcKind::ShadowFlameApparition => 472,
            // W4-d40: BigMimicCorruption, BigMimicCrimson, BigMimicHallow, BigMimicJungle, Mothron keep their vanilla ids (free in the protocol space).
            NpcKind::BigMimicCorruption => 473,
            NpcKind::BigMimicCrimson => 474,
            NpcKind::BigMimicHallow => 475,
            NpcKind::BigMimicJungle => 476,
            NpcKind::Mothron => 477,
            // W4-d40: MothronEgg, MothronSpawn, Medusa, GreekSkeleton, GraniteGolem keep their vanilla ids (free in the protocol space).
            NpcKind::MothronEgg => 478,
            NpcKind::MothronSpawn => 479,
            NpcKind::Medusa => 480,
            NpcKind::GreekSkeleton => 481,
            NpcKind::GraniteGolem => 482,
            // W4-d40: GraniteFlyer, EnchantedNightcrawler, Grubby, Sluggy, Buggy keep their vanilla ids (free in the protocol space).
            NpcKind::GraniteFlyer => 483,
            NpcKind::EnchantedNightcrawler => 484,
            NpcKind::Grubby => 485,
            NpcKind::Sluggy => 486,
            NpcKind::Buggy => 487,
            // W4-d40: TargetDummy keep their vanilla ids (free in the protocol space).
            NpcKind::TargetDummy => 488,
            // W4-d35 (A): the 73/74/75/77/78 band - port-local protocol ids
            // 698.. (MERGE-SIDE +269 renumber per 367 s1 E7 / d37 tables:
            // 429-461 collided with d40's landed 430-488 band; the vanilla ids collide with the port-local space;
            // the pirate/wyvern renumber precedent). The vanilla NPC.type
            // rides cs_id/net_id, NOT these protocol ids.
            NpcKind::GoblinScout => 698,
            NpcKind::Bird => 699,
            NpcKind::Pixie => 700,
            NpcKind::ArmoredSkeleton => 701,
            NpcKind::Mummy => 702,
            // W4-d35 (B): the 79/80/82/83/84 band - port-local protocol ids
            // 698.. (MERGE-SIDE +269 renumber per 367 s1 E7 / d37 tables:
            // 429-461 collided with d40's landed 430-488 band; the vanilla ids collide with the port-local space;
            // the pirate/wyvern renumber precedent). The vanilla NPC.type
            // rides cs_id/net_id, NOT these protocol ids.
            NpcKind::DarkMummy => 703,
            NpcKind::LightMummy => 704,
            NpcKind::Wraith => 705,
            NpcKind::CursedHammer => 706,
            NpcKind::EnchantedSword => 707,
            // W4-d35 (C): the 85/86/93/94/95 band - port-local protocol ids
            // 698.. (MERGE-SIDE +269 renumber per 367 s1 E7 / d37 tables:
            // 429-461 collided with d40's landed 430-488 band; the vanilla ids collide with the port-local space;
            // the pirate/wyvern renumber precedent). The vanilla NPC.type
            // rides cs_id/net_id, NOT these protocol ids.
            NpcKind::Mimic => 708,
            NpcKind::Unicorn => 709,
            NpcKind::GiantBat => 710,
            NpcKind::Corruptor => 711,
            NpcKind::DiggerHead => 712,
            // W4-d35 (D): the 96/97/98/99/100 band - port-local protocol ids
            // 698.. (MERGE-SIDE +269 renumber per 367 s1 E7 / d37 tables:
            // 429-461 collided with d40's landed 430-488 band; the vanilla ids collide with the port-local space;
            // the pirate/wyvern renumber precedent). The vanilla NPC.type
            // rides cs_id/net_id, NOT these protocol ids.
            NpcKind::DiggerBody => 713,
            NpcKind::DiggerTail => 714,
            NpcKind::SeekerHead => 715,
            NpcKind::SeekerBody => 716,
            NpcKind::SeekerTail => 717,
            // W4-d35 (E): the 101/102/103/104/109 band - port-local protocol ids
            // 698.. (MERGE-SIDE +269 renumber per 367 s1 E7 / d37 tables:
            // 429-461 collided with d40's landed 430-488 band; the vanilla ids collide with the port-local space;
            // the pirate/wyvern renumber precedent). The vanilla NPC.type
            // rides cs_id/net_id, NOT these protocol ids.
            NpcKind::Clinger => 718,
            NpcKind::AnglerFish => 719,
            NpcKind::GreenJellyfish => 720,
            NpcKind::Werewolf => 721,
            NpcKind::Clown => 722,
            // W4-d35 (F): the 110/112/116/120/121 band - port-local protocol ids
            // 698.. (MERGE-SIDE +269 renumber per 367 s1 E7 / d37 tables:
            // 429-461 collided with d40's landed 430-488 band; the vanilla ids collide with the port-local space;
            // the pirate/wyvern renumber precedent). The vanilla NPC.type
            // rides cs_id/net_id, NOT these protocol ids.
            NpcKind::SkeletonArcher => 723,
            NpcKind::VileSpit => 724,
            NpcKind::TheHungryII => 725,
            NpcKind::ChaosElemental => 726,
            NpcKind::Slimer => 727,
            // W4-d35 (G): the 122/137/140 band - port-local protocol ids
            // 698.. (MERGE-SIDE +269 renumber per 367 s1 E7 / d37 tables:
            // 429-461 collided with d40's landed 430-488 band; the vanilla ids collide with the port-local space;
            // the pirate/wyvern renumber precedent). The vanilla NPC.type
            // rides cs_id/net_id, NOT these protocol ids.
            NpcKind::Gastropod => 728,
            NpcKind::IlluminantBat => 729,
            NpcKind::PossessedArmor => 730,
            NpcKind::WyvernTail => 228,
            // W4-d34 kinds 1-70 coverage wave: the port-local 266..280 band
            // (free in to_u16/from_u16 AND render_key; the vanilla types ride
            // cs_id/net_id, NOT these protocol ids — the S1-F-1 separation).
            NpcKind::AngryBones => 266,
            NpcKind::CursedSkull => 267,
            NpcKind::OldMan => 268,
            NpcKind::ManEater => 269,
            NpcKind::Bunny => 270,
            NpcKind::BoneSerpentHead => 271,
            NpcKind::BoneSerpentBody => 272,
            NpcKind::BoneSerpentTail => 273,
            NpcKind::CorruptBunny => 274,
            NpcKind::Harpy => 275,
            NpcKind::Snatcher => 276,
            NpcKind::CorruptGoldfish => 277,
            NpcKind::Vulture => 278,
            NpcKind::Demon => 279,
            NpcKind::BlueJellyfish => 280,
            // W4-d34 merge-side moves (367 §1, d37 tables): Crab 282→791,
            // DungeonGuardian 283→792; the PinkJellyfish/Antlion/SpikeBall
            // dups dropped (g8 owns 745-747). 281-285 cleared for d38.
            NpcKind::Crab => 791,
            NpcKind::DungeonGuardian => 792,
            // W4-d37 — the NPCID 211-280 kinds-coverage band (census 65, 46
            // missing ids). The vanilla ids 217-280 collide with the landed
            // port-local bands (217/218 destroyer, 219-222 fishron, 223-228
            // wyvern, 230-236 zombies, 250 nimbus), so the 46 take the next
            // free port-local band 354-399 (above the 353 BlazingWheel; the
            // vanilla `NPC.type` rides `cs_id`/`net_id`, NOT these protocol
            // ids — the S1-F-1 renumber precedent). Save-compat safe: the
            // band is free on main (to_u16 scan, 0 dups).
            NpcKind::CochinealBeetle => 354,
            NpcKind::CyanBeetle => 355,
            NpcKind::LacBeetle => 356,
            NpcKind::SeaSnail => 357,
            NpcKind::Squid => 358,
            NpcKind::FlyingFish => 359,
            NpcKind::FlyingSnake => 360,
            NpcKind::Pirate => 361,
            NpcKind::GoldfishWalker => 362,
            NpcKind::HornetFatty => 363,
            NpcKind::HornetHoney => 364,
            NpcKind::HornetLeafy => 365,
            NpcKind::HornetSpikey => 366,
            NpcKind::HornetStingy => 367,
            NpcKind::JungleCreeper => 368,
            NpcKind::JungleCreeperWall => 369,
            NpcKind::BlackRecluseWall => 370,
            NpcKind::BloodCrawler => 371,
            NpcKind::BloodCrawlerWall => 372,
            NpcKind::BloodFeeder => 373,
            NpcKind::BloodJelly => 374,
            NpcKind::IceGolem => 375,
            NpcKind::RainbowSlime => 376,
            NpcKind::Eyezor => 377,
            NpcKind::Parrot => 378,
            NpcKind::Reaper => 379,
            NpcKind::ZombieMushroom => 380,
            NpcKind::ZombieMushroomHat => 381,
            NpcKind::FungoFish => 382,
            NpcKind::AnomuraFungus => 383,
            NpcKind::MushiLadybug => 384,
            NpcKind::FungiBulb => 385,
            NpcKind::GiantFungiBulb => 386,
            NpcKind::IchorSticker => 387,
            NpcKind::RustyArmoredBonesAxe => 388,
            NpcKind::RustyArmoredBonesFlail => 389,
            NpcKind::RustyArmoredBonesSword => 390,
            NpcKind::RustyArmoredBonesSwordNoArmor => 391,
            NpcKind::BlueArmoredBones => 392,
            NpcKind::BlueArmoredBonesMace => 393,
            NpcKind::BlueArmoredBonesNoPants => 394,
            NpcKind::BlueArmoredBonesSword => 395,
            NpcKind::HellArmoredBones => 396,
            NpcKind::HellArmoredBonesSpikeShield => 397,
            NpcKind::HellArmoredBonesMace => 398,
            NpcKind::HellArmoredBonesSword => 399,
            // W4-d42 — the NPCID 578-630 roster rides its VANILLA ids
            // (protocol == NPCID, the d38/d40 vanilla-free precedent; the
            // 108 union plan §3: protocol space >=489 is free on the union,
            // so 578-630 need no renumber). The port-local 354-404 band the
            // first pass used COLLIDED with d37's 354-399 (108 §2a) — lifted.
            // The eight kinds other lanes own are NOT here: 581/587 (c4
            // style-44), 592/607/615 (c4b style-16), 604/605 (e26b luck),
            // 606 (c8 style-66) — their ids stay free for those branches.
            NpcKind::DD2LightningBug => 578,
            NpcKind::BartenderUnconscious => 579,
            NpcKind::WalkingAntlion => 580,
            NpcKind::LarvaeAntlion => 582,
            NpcKind::ZombieMerman => 586,
            NpcKind::TorchZombie => 590,
            NpcKind::ArmedTorchZombie => 591,
            NpcKind::GoldGoldfishWalker => 593,
            NpcKind::Seagull => 602,
            NpcKind::Seagull2 => 603,
            NpcKind::Grebe => 608,
            NpcKind::Grebe2 => 609,
            NpcKind::Rat => 610,
            NpcKind::Owl => 611,
            NpcKind::WaterStrider => 612,
            NpcKind::GoldWaterStrider => 613,
            NpcKind::ExplosiveBunny => 614,
            NpcKind::Turtle => 616,
            NpcKind::TurtleJungle => 617,
            NpcKind::BloodNautilus => 618,
            NpcKind::BloodSquid => 619,
            NpcKind::GoblinShark => 620,
            NpcKind::BloodEelHead => 621,
            NpcKind::BloodEelBody => 622,
            NpcKind::BloodEelTail => 623,
            NpcKind::Gnome => 624,
            NpcKind::SeaTurtle => 625,
            NpcKind::Seahorse => 626,
            NpcKind::GoldSeahorse => 627,
            NpcKind::Dandelion => 628,
            NpcKind::IceMimic => 629,
            NpcKind::BloodMummy => 630,
            // W4-c10 — the waterfowl family keeps its vanilla ids (362-365/
            // 602/603/608/609 are free in the protocol space — a to_u16 scan
            // of the tree shows no arm on them). `NPCID.cs:11401-11407`/
            // `:11881-11895`.
            // E8 merge (d37 tables): the Duck quartet rides 731-734
            // (d37's GoldfishWalker/HornetFatty/HornetHoney/HornetLeafy
            // own the vanilla slots; cs_id/net_id stay 362-365); the
            // Seagull/Grebe pairs + the Rat..BloodMummy band are d42's
            // rows above (the dup drops - c10's waterfowl wiring composes).
            NpcKind::Duck => 731,
            NpcKind::Duck2 => 732,
            NpcKind::DuckWhite => 733,
            NpcKind::DuckWhite2 => 734,
            // W4-c3 (the style-10 family): the WaterBoltMimic keeps its
            // vanilla 694 (`NPC.cs:17737-17751`). The CursedSkull and
            // GiantCursedSkull ports dropped per the d37 E8 tables
            // (research/329 s3) - d34@267 / d38@289 own them above.
            NpcKind::WaterBoltMimic => 694,
            // W4-c4 (the style-44 swimmers): the FlyingFish's vanilla 224
            // is the WyvernLegs' port-local slot above (`to_u16`'s
            // bijection guard). Its original port-local 354 HARD-collided
            // d37's pushed CochinealBeetle 354 (and d23/d42/g8 claim 354
            // too) — RETIRED on this branch: the row adopts d37's
            // FlyingFish id 359 (its frozen 354-399 band, research/108
            // §3; the dup-check verdict keeps ONE FlyingFish at 359/cs
            // 224 with c4's world-sim wiring — the identical arm dedupes
            // at merge). 509/581/587 keep their vanilla ids (free on
            // every pushed tip — tools/_w4c4_proto_census.js).
            // `NPC.cs:11885-11897` / `:14841-14853` / `:14868-14880` /
            // `:16623-16637`.
            // E8 merge: FlyingFish 359 is d37's arm above (identical-id
            // dedupe, c4's wiring keeps); GiantFlyingAntlion 509 c4's own.
            NpcKind::GiantFlyingAntlion => 509,
            NpcKind::FlyingAntlion => 581,
            NpcKind::EyeballFlyingFish => 587,
            // W4-c4 (the style-16 swimmers): the CorruptGoldfish's vanilla
            // 57 is the GoblinThief's port-local slot above and the
            // Arapaima's 157 is the BoundWizard's (`to_u16`'s bijection
            // guard). Their original port-local 229/237 squatted the 108
            // §3 reserved cs-owned holes (the Pirate's/JungleCreeperWall's
            // vanilla types — now also claimed by d125's EmpressOfLight/
            // PrismaticLacewing), so they take the free-port band 405/406
            // (above d37's frozen 354-399 + d42's pushed 354-404, below
            // d35's 108 §3 renumber target 429-461; free on ALL 354 pushed
            // tips — tools/_w4c4_proto_census.js). 241/465/592/607/615/
            // 688/692 keep their vanilla ids; BloodFeeder 241 is the
            // dup-check's merge-retire (d37's 373 survives — the row
            // stays until the merge dedupe). `NPC.cs:9489-9501` /
            // `:10989-11002` / `:12105-12118` / `:14483-14495` /
            // `:16679-16693` / `:16795-16808` / `:16892-16904` /
            // `:17653-17666` / `:17709-17722`.
            // E8 merge drops: CorruptGoldfish 405 (d34@277 owns),
            // BloodFeeder 241 (d37@373 owns), Pufferfish 688 / Orca 692
            // (d43 rows them, lands later - the c4 impl retires per
            // research/148's TRIGGERED conditional).
            NpcKind::Arapaima => 406,
            NpcKind::CrimsonGoldfish => 465,
            NpcKind::GoldGoldfish => 592,
            NpcKind::Pupfish => 607,
            NpcKind::Dolphin => 615,
            // W4-c7 — the firefly family keeps its vanilla ids (355/358/
            // 654/677 are free in the protocol space — a to_u16 scan of
            // the merged tree shows no arm on them). `NPCID.cs:11387`/
            // `:11393`/`:11985`/`:12031`.
            // E8 merge renumber (d37 tables): 355/358 are d37's
            // CyanBeetle/Squid in the settled roster - the firefly pair
            // rides the port-local 735/736 (cs_id/net_id stay 355/358).
            NpcKind::Firefly => 735,
            NpcKind::LightningBug => 736,
            NpcKind::Lavafly => 654,
            NpcKind::Shimmerfly => 677,
            // W4-c8 — the crawling-critter family keeps its vanilla ids
            // (357/374/375/448/484-487/606 are free in the protocol space
            // — a to_u16 scan of the tree shows no arm on them).
            // `NPCID.cs:11391`/`:11425`/`:11427`/`:11569`/`:11641-11647`/
            // `:11889`.
            // E8 merge (d37 tables): Worm/TruffleWorm/TruffleWormDigger
            // ride 737/738/739 (d37's SeaSnail/BloodJelly/IceGolem own
            // the vanilla slots; cs_id/net_id stay 357/374/375); the
            // GoldWorm 448 + EnchantedNightcrawler/Grubby/Sluggy/Buggy
            // 484-487 dups DROP (d40's rows above own them - c8's
            // style-66 wiring composes onto the canonical kinds).
            NpcKind::Worm => 737,
            NpcKind::TruffleWorm => 738,
            NpcKind::TruffleWormDigger => 739,
            NpcKind::Maggot => 606,
            // The snail wall-crawler family + its shimmer target (W4-c9):
            // the vanilla ids 359/360/655/677 are free protocol slots (the
            // to_u16 scan: the 354+ band above the landed families is
            // unclaimed), so they keep their vanilla ids — the
            // AngryNimbus/DungeonSpirit precedent.
            // E8 merge (d37 tables): Snail/GlowingSnail ride 740/741
            // (d37's FlyingFish/FlyingSnake own the vanilla slots;
            // cs_id/net_id stay 359/360); MagmaSnail 655 keeps; the
            // Shimmerfly 677 dup + the 5 gold dups drop (c7/d40's rows
            // above own them - c9's wiring composes, 349 s6).
            NpcKind::Snail => 740,
            NpcKind::GlowingSnail => 741,
            NpcKind::MagmaSnail => 655,
            // W4-c10 roster tail — the twelve critters keep their vanilla
            // ids (357/442-448/484-487 are free in the protocol space — a
            // to_u16 scan of the tree shows no arm on them).
            // `NPCID.cs:11391`/`:11557-11569`/`:11641-11647`.
            // E8 merge: the 12-critter tail dropped - Worm rides c8@737,
            // the gold septet + the bait tail are d40's rows above (the
            // dup drops; c10's wiring composes onto the canonical kinds).
            // W4-d25 STYLE-26 wiring: the Unicorn mirrors W4-d35 batch C
            // (6ed8c22f) verbatim — protocol 440 (the vanilla 86 slot is
            // the RayGunner's; the merge keep-one). The Wolf's vanilla
            // 155 is PlanterasTentacle's protocol slot, so it takes the
            // next-free port-local 355 (353 BlazingWheel, 354
            // WebbedStylist; 355/356 free across the co-lanes).
            NpcKind::Wolf => 772,
            // W4-d25 STYLE-26 wiring (commit 2): the Tumbleweed keeps its
            // vanilla id 546 (free across the co-lanes — g10 carries only
            // the lang_tables display name, no NpcKind).
            NpcKind::Tumbleweed => 546,
            // W4-d120 — the unclaimed chatter kinds' port-local band
            // (the claim doc `tools/_d120_claim.md`; the vanilla
            // 142/160/178/209 collide with the crawltipede/Stylist/
            // kobold/SwampZombie slots above, 579 is free but the family
            // keeps one band). Free on main + every pushed sibling.
            NpcKind::SantaClaus => 241,
            NpcKind::Truffle => 242,
            NpcKind::Steampunker => 243,
            NpcKind::Cyborg => 244,
            // W4-d125 empress wiring — research/108 R4/E8 renumber EXECUTED
            // (the d37 roster-master reconcile): 229/237 were reserved-cs
            // hole squats (229 Pirate — d37-owned; 237 JungleCreeperWall).
            // EmpressOfLight 229→636 RESTORES protocol==cs (its OWN vanilla
            // id, `NPCID.cs:11949`); PrismaticLacewing 237→742 keeps the
            // small-lane block contiguous (cs 661 rides `cs_id`/ENTRIES).
            NpcKind::EmpressOfLight => 636,
            NpcKind::PrismaticLacewing => 742,
            // W4-g5 — the two unclaimed aiStyle-8 casters. The RuneWizard's
            // vanilla 172 collides with DD2DarkMageT3's port-local 172, so
            // it takes a port-local id; the RETIRE round moved it 354 → 827
            // (research/322: 354 squats d37's CochinealBeetle corridor
            // 354-399, the research/108 E2 kinds-lane rule; 827 is above
            // d39's pushed 817-823 band + the R7 817-826 pin). The
            // LibrarianSkeleton's 693 is free (vanilla).
            NpcKind::RuneWizard => 827,
            NpcKind::LibrarianSkeleton => 693,
            // W4-g8 lane 1 batch 1 — the d22-d33 UNWIRED machines' kinds.
            // E8 EXECUTED (d37 research/108 S0-S4, the g8 move-table): the
            // 17 OWNED kinds renumber to the port-local 745-761 band (the
            // 354-387 corridor is d37/d42's — g8 was the squatter); the 17
            // DUPLICATE kinds keep their pushed ids as DROP-PENDING rows
            // (the canonical owners d34/d35/d37 land via their lanes; the
            // keep-one drops the g8 row at merge — the axis wirings ALL
            // stay, the machine-wiring lane's whole point). The vanilla
            // `NPC.type` rides `cs_id`/the bestiary `net_id` unchanged.
            NpcKind::PinkJellyfish => 745, // E8: 356 -> 745 (cs 64)
            NpcKind::Antlion => 746, // E8: 357 -> 746 (cs 69)
            NpcKind::SpikeBall => 747, // E8: 358 -> 747 (cs 70)
            // W4-g8 lane 1 batch 2 — E8: the owned kinds move to 748-756.
            NpcKind::WallCreeper => 748, // E8: 366 -> 748 (cs 164)
            NpcKind::WallCreeperWall => 749, // E8: 367 -> 749 (cs 165)
            NpcKind::IceElemental => 750, // E8: 368 -> 750 (cs 169)
            NpcKind::Herpling => 751, // E8: 369 -> 751 (cs 174)
            NpcKind::Derpling => 752, // E8: 370 -> 752 (cs 177)
            NpcKind::CrimsonAxe => 753, // E8: 371 -> 753 (cs 179)
            NpcKind::FloatyGross => 754, // E8: 372 -> 754 (cs 182)
            NpcKind::LostGirl => 755, // E8: 373 -> 755 (cs 195)
            NpcKind::Nymph => 756, // E8: 374 -> 756 (cs 196)
            // W4-g8 lane 1 batch 3 — E8: the owned bats move to 757-760.
            // W4-g8 lane 1 batch 4 — E8: RedDevil moves to 761.
            // W4-g8 machine round 3 — the Vampire 159 census row (the
            // VampireBat's Transform target). E8 COLLISION FIX: the pushed
            // 388 is d37's RustyArmoredBonesAxe (the 354-399 corridor); the
            // Vampire takes 762, the first free id above the S4 CORE max
            // 766 (d37 SEAL fbb948172 line 1408-1411: only 762 + 788-789
            // were free in d36's old band; 788-790 are RESERVED for d39's
            // 3 unclaimed kinds — `_d37_seal_final.json` carries no 762).
            // The dragonflies (W4-d33): the vanilla ids 595-601 are free
            // protocol slots (the to_u16 scan: no arm above rides them),
            // so they keep their vanilla ids — the W4-e26 ladybug
            // precedent. The c10 swim-forms 603/609 stay clear.
            NpcKind::BlackDragonfly => 595,
            NpcKind::BlueDragonfly => 596,
            NpcKind::GreenDragonfly => 597,
            NpcKind::OrangeDragonfly => 598,
            NpcKind::RedDragonfly => 599,
            NpcKind::YellowDragonfly => 600,
            NpcKind::GoldDragonfly => 601,
            // The fairy critters + the windy balloon (W4-d33 wiring): the
            // constants-copy of d42 batch-2/3's wire rows (86182c57f /
            // 44f007096 — the 359/360/361/368 port-local band). KEEP-ONE
            // (d37c-E3): the union keeps d42's numbering verbatim; the
            // vanilla types 583-585/594 ride cs_id/net_id below (the
            // dragonfly wiring kept its free vanilla slots; these four
            // ride d42's band instead so the merge dedupes to one row).
            NpcKind::FairyCritterPink => 583,
            NpcKind::FairyCritterGreen => 584,
            NpcKind::FairyCritterBlue => 585,
            NpcKind::WindyBalloon => 594,
            // W4-e21 — the bunny statue's pair (46/540): the next-free
            // port-local holes above the wyvern band (the vanilla 46 is
            // the WallOfFlesh's protocol slot, 540 is free but the pair
            // keeps one band). `NPC.cs:9336-9347`.
            NpcKind::PartyBunny => 237,
            // W4-f42 — the sandstorm debris + desert family: the next-free
            // 354..362 band (353 BlazingWheel reserved; the vanilla types
            // ride cs_id/net_id, the bestiary rows).
            // W4-d9 wiring (the d40 7a88c61c/45dbafc3 dragon rows, verbatim):
            // CultistDragonHead..Tail keep their vanilla ids (free in the
            // protocol space). The 521 squidhead (the d41 band, unrowed —
            // ported here from `NPC.cs:15768-15779`) keeps 521 too.
            NpcKind::AncientCultistSquidhead => 521,
            // W4-d39 (research/125 §3 + 108 R7): the band 351-420's
            // unclaimed kinds take the port-local 817-826 corridor
            // (R7 re-pin; the original 781-804 guidance was superseded
            // by the R4-R6 small-lane moves) — vanilla 356 sits inside
            // d37's pushed 354-399 protocol band, so vanilla-free
            // numbering is NOT available here.
            NpcKind::Butterfly => 817,
            NpcKind::ScorpionBlack => 818,
            NpcKind::Scorpion => 819,
            NpcKind::Grasshopper => 820,
            NpcKind::ChatteringTeethBomb => 821,
            NpcKind::CultistArcherBlue => 822,
            NpcKind::CultistArcherWhite => 823,
            // W4-d41 — the cavern crawdads/shellies keep their vanilla ids
            // (494-497 are free in the wire space; the pumpkin/frost
            // precedent). `NPCID.cs:11661-11667`.
            NpcKind::Crawdad => 494,
            NpcKind::Crawdad2 => 495,
            NpcKind::GiantShelly => 496,
            NpcKind::GiantShelly2 => 497,
            // W4-d41 — the nine salamander colours keep their vanilla ids
            // (498-506 free in the wire space). `NPCID.cs:11669-11685`.
            NpcKind::Salamander => 498,
            NpcKind::Salamander2 => 499,
            NpcKind::Salamander3 => 500,
            NpcKind::Salamander4 => 501,
            NpcKind::Salamander5 => 502,
            NpcKind::Salamander6 => 503,
            NpcKind::Salamander7 => 504,
            NpcKind::Salamander8 => 505,
            NpcKind::Salamander9 => 506,
            // W4-d41 — the antlions + the Dune Splicer chain keep their
            // vanilla ids (508-512 free in the wire space).
            // `NPCID.cs:11687-11695`.
            NpcKind::GiantWalkingAntlion => 508,
            NpcKind::DuneSplicerHead => 510,
            NpcKind::DuneSplicerBody => 511,
            NpcKind::DuneSplicerTail => 512,
            // W4-d41 — the Tomb Crawler chain + the solar fragments keep
            // their vanilla ids (513-516/519 free in the wire space).
            // `NPCID.cs:11697-11709`.
            NpcKind::TombCrawlerHead => 513,
            NpcKind::TombCrawlerBody => 514,
            NpcKind::TombCrawlerTail => 515,
            NpcKind::SolarGoop => 519,
            // W4-d41 — the ancient orbs + the desert ghouls keep their
            // vanilla ids (522-526 free in the wire space).
            // `NPCID.cs:11721-11729`.
            NpcKind::AncientLight => 522,
            NpcKind::AncientDoom => 523,
            NpcKind::DesertGhoul => 524,
            NpcKind::DesertGhoulCorruption => 525,
            NpcKind::DesertGhoulCrimson => 526,
            // W4-d41 — the hallowed ghoul + the lamias + the sand
            // poachers keep their vanilla ids (527-531 free in the wire
            // space). `NPCID.cs:11731-11739`.
            NpcKind::DesertGhoulHallow => 527,
            NpcKind::DesertLamiaLight => 528,
            NpcKind::DesertLamiaDark => 529,
            NpcKind::DesertScorpionWalk => 530,
            NpcKind::DesertScorpionWall => 531,
            // W4-d41 — the basilisk + the tortured soul + the slimes +
            // the bride keep their vanilla ids (532/534-537 free in the
            // wire space). `NPCID.cs:11741-11751`.
            NpcKind::DesertBeast => 532,
            NpcKind::DemonTaxCollector => 534,
            NpcKind::SlimeSpiked => 535,
            NpcKind::TheBride => 536,
            NpcKind::SandSlime => 537,
            // W4-d41 — the critters + the tumbler keep their vanilla ids
            // (538-540/546 free in the wire space). `NPCID.cs:11753-11769`.
            NpcKind::SquirrelRed => 538,
            NpcKind::SquirrelGold => 539,
            // The luck critters (W4-e26, E8 keep-one EXECUTED): the Frog
            // takes the d37-E8 small-lane slot 744 (its vanilla proto 361
            // is contested by d37's Pirate@361 — research/108 E1/E8, the
            // 731-744 band; cs_id/net_id stay 361, only the wire id moves,
            // branch-only so no landed save wrote it — the S1-F-1 gate).
            // The Duck 362 row DROPS to c10 (the canonical waterfowl owner,
            // E8/SEAL keep-one → c10@731); the ladybug pair 604/605 stays
            // e26's (the d42 ownership flip, research/108 S1 — d42 dropped
            // its rows at 49346dba6). 604/605 are free protocol slots.
            NpcKind::Frog => 744,
            NpcKind::LadyBug => 604,
            NpcKind::GoldLadyBug => 605,
            // W4-g10 lane 3 — the town pets ride their VANILLA ids (the
            // pumpkin/frost precedent): 637/638/656/670/678-684 are all
            // free in the protocol space (the port-local bands stop at
            // 353) and the housing/bestiary/`net_id` consumers key on the
            // vanilla `NPC.type`, so one id serves the wire and the type.
            NpcKind::TownCat => 637,
            NpcKind::TownDog => 638,
            NpcKind::TownBunny => 656,
            NpcKind::TownSlimeBlue => 670,
            NpcKind::TownSlimeGreen => 678,
            NpcKind::TownSlimeOld => 679,
            NpcKind::TownSlimePurple => 680,
            NpcKind::TownSlimeRainbow => 681,
            NpcKind::TownSlimeRed => 682,
            NpcKind::TownSlimeYellow => 683,
            NpcKind::TownSlimeCopper => 684,
            // Doc-sweep lane 36 — the ShimmerSlime keeps its vanilla id
            // (676 is free in the protocol space; the 677 twin is
            // W4-c7/c9's pushed claim). `NPCID.cs:12029`.
            NpcKind::ShimmerSlime => 676,
            // W4-f18 first-20 port — the critters take the next-free
            // port-local band 237.. (229 free but kept as a gap; 230-236
            // are the holiday zombies + blood-moon pair). The vanilla
            // types ride cs_id/net_id (355-360), NOT these protocol ids.
            // W4-f18 batch 2 — the critters take the next-free band 242..
            // (the vanilla types 360-364 ride cs_id/net_id, NOT these).
            // W4-f18 batch 3 — the critters take the next-free band 247..
            // (250/288 are the AngryNimbus/DungeonSpirit's port-local ids;
            // the vanilla types 365-375 ride cs_id/net_id, NOT these).
            // W4-f18 batch 4 — the next-free band 254.. (the vanilla types
            // 377-430 ride cs_id/net_id, NOT these protocol ids).
            // The Shimmerfly (W4-c9's rowing, adopted by W4-g62 for the
            // TRAILCACHE WIDENING's 677→60 row): the vanilla id 677 is a
            // free protocol slot (the to_u16 scan: the 354+ band above
            // the landed families is unclaimed), so it keeps its vanilla
            // id — the AngryNimbus/DungeonSpirit precedent. The snail
            // trio 359/360/655 stays c9's (the merge dedupes).
            // W4-d42 - the vanilla ids 620-624 are free protocol slots
            // (the 108 union plan's VANILLA-FREE band; d42 2d7ac08c3's
            // rowing, adopted by W4-g62 for the 620→20 trail row — the
            // rest of the roster stays d42's, the merge dedupes).
        }
    
            // d23's g8/d43-owned rows (MERGE NOTE 5fdb002b): these kinds
            // exist in the enum via d23's style-14 wave, but g8 (757-762)
            // and d43 (666) are NOT in main yet, so d23 carries their id
            // rows at its own free-band ids. RE-KEY OBLIGATION: when g8 /
            // d43 land, move these arms to 757-762 / 666 (d37 table E3).
            NpcKind::IceBat => 401,
            NpcKind::Lavabat => 402,
            NpcKind::GiantFlyingFox => 403,
            NpcKind::RedDevil => 404,
            NpcKind::VampireBat => 405,
            NpcKind::Vampire => 400,
            NpcKind::VileSpitEoW => 411,
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
            // W4-c6 style-59 wiring: the blade's vanilla id (`to_u16`).
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
            // W4-d38 b7 — decode at the vanilla ids.
            333 => NpcKind::SlimeRibbonWhite,
            334 => NpcKind::SlimeRibbonYellow,
            335 => NpcKind::SlimeRibbonGreen,
            336 => NpcKind::SlimeRibbonRed,
            337 => NpcKind::BunnyXmas,
            // W4-d38 b6 — decode at the vanilla ids.
            318 => NpcKind::DemonEyeSpaceship,
            322 => NpcKind::SkeletonTopHat,
            323 => NpcKind::SkeletonAstonaut,
            324 => NpcKind::SkeletonAlien,
            328 => NpcKind::PumpkingBlade,
            // W4-d38 b5 — decode at the vanilla ids.
            302 => NpcKind::SlimeMasked,
            303 => NpcKind::BunnySlimed,
            304 => NpcKind::HoppinJack,
            316 => NpcKind::Ghost,
            317 => NpcKind::DemonEyeOwl,
            // W4-d38 b4 — decode at the vanilla ids.
            297 => NpcKind::BirdBlue,
            298 => NpcKind::BirdRed,
            299 => NpcKind::Squirrel,
            300 => NpcKind::Mouse,
            301 => NpcKind::Raven,
            // W4-d38 b3 — decode at the vanilla ids.
            292 => NpcKind::TacticalSkeleton,
            293 => NpcKind::SkeletonCommando,
            294 => NpcKind::AngryBonesBig,
            295 => NpcKind::AngryBonesBigMuscle,
            296 => NpcKind::AngryBonesBigHelmet,
            // W4-d38 b2 — decode at the vanilla ids.
            286 => NpcKind::DiabolistWhite,
            287 => NpcKind::BoneLee,
            289 => NpcKind::GiantCursedSkull,
            290 => NpcKind::Paladin,
            291 => NpcKind::SkeletonSniper,
            // W4-d38 b1 — decode at the vanilla ids.
            281 => NpcKind::RaggedCaster,
            282 => NpcKind::RaggedCasterOpenCoat,
            283 => NpcKind::Necromancer,
            284 => NpcKind::NecromancerArmored,
            285 => NpcKind::DiabolistRed,
            // The man-eater family (W4-d23): the port-local 354-357 band
            // + the two bulbs' free vanilla ids (`to_u16` above).
            // E8 merge: 743 (the 354-356 slots are d37's/d34's; the
            // ManEater/Snatcher/Clinger/FungiBulb/GiantFungiBulb dups
            // dropped - d34/d35/d37's decode arms above).
            743 => NpcKind::AngryTrapper,
            // The aiStyle-9 spit/flare family (W4-d23): the d35-rowed 455
            // + the unrowed 400+ band (`to_u16` above).
            // E8 merge: the 455/411 dups dropped (d35@724 / d43@666);
            // SolarFlare 408 keeps.
            408 => NpcKind::SolarFlare,
            // The aiStyle-14 flyer family (W4-d23): the five rowed roster
            // ids + the nine unrowed 400+ band (`to_u16` above).
            // E8 merge: the 12 flyer dups dropped (d34/d35/d37/g8's
            // decode arms above); SporeBat 634 + QueenSlimeMinionPurple
            // 660 renumbered (the d37 tables).
            634 => NpcKind::SporeBat,
            660 => NpcKind::QueenSlimeMinionPurple,
            // The Wyvern family (`S1-D-3` wyvern): the gate-reassigned
            // 223-228 band (`to_u16` above; the vanilla 87-92 collide with
            // the Scutlix/saucer ids). `NPCID.cs:10851-10861`.
            223 => NpcKind::WyvernHead,
            224 => NpcKind::WyvernLegs,
            225 => NpcKind::WyvernBody,
            226 => NpcKind::WyvernBody2,
            227 => NpcKind::WyvernBody3,
            // W4-d40: the vanilla ids (`to_u16` above).
            430 => NpcKind::ArmedZombie,
            431 => NpcKind::ArmedZombieEskimo,
            432 => NpcKind::ArmedZombiePincussion,
            433 => NpcKind::ArmedZombieSlimed,
            434 => NpcKind::ArmedZombieSwamp,
            // W4-d40: the vanilla ids (`to_u16` above).
            435 => NpcKind::ArmedZombieTwiggy,
            436 => NpcKind::ArmedZombieCenx,
            437 => NpcKind::CultistTablet,
            438 => NpcKind::CultistDevote,
            440 => NpcKind::CultistBossClone,
            // W4-d40: the vanilla ids (`to_u16` above).
            442 => NpcKind::GoldBird,
            443 => NpcKind::GoldBunny,
            444 => NpcKind::GoldButterfly,
            445 => NpcKind::GoldFrog,
            446 => NpcKind::GoldGrasshopper,
            // W4-d40: the vanilla ids (`to_u16` above).
            447 => NpcKind::GoldMouse,
            448 => NpcKind::GoldWorm,
            449 => NpcKind::BoneThrowingSkeleton,
            450 => NpcKind::BoneThrowingSkeleton2,
            451 => NpcKind::BoneThrowingSkeleton3,
            // W4-d40: the vanilla ids (`to_u16` above).
            452 => NpcKind::BoneThrowingSkeleton4,
            453 => NpcKind::SkeletonMerchant,
            454 => NpcKind::CultistDragonHead,
            455 => NpcKind::CultistDragonBody1,
            456 => NpcKind::CultistDragonBody2,
            // W4-d40: the vanilla ids (`to_u16` above).
            457 => NpcKind::CultistDragonBody3,
            458 => NpcKind::CultistDragonBody4,
            459 => NpcKind::CultistDragonTail,
            460 => NpcKind::Butcher,
            461 => NpcKind::CreatureFromTheDeep,
            // W4-d40: the vanilla ids (`to_u16` above).
            462 => NpcKind::Fritz,
            463 => NpcKind::Nailhead,
            464 => NpcKind::CrimsonBunny,
            466 => NpcKind::Psycho,
            // W4-d40: the vanilla ids (`to_u16` above).
            467 => NpcKind::DeadlySphere,
            468 => NpcKind::DrManFly,
            469 => NpcKind::ThePossessed,
            470 => NpcKind::CrimsonPenguin,
            472 => NpcKind::ShadowFlameApparition,
            // W4-d40: the vanilla ids (`to_u16` above).
            473 => NpcKind::BigMimicCorruption,
            474 => NpcKind::BigMimicCrimson,
            475 => NpcKind::BigMimicHallow,
            476 => NpcKind::BigMimicJungle,
            477 => NpcKind::Mothron,
            // W4-d40: the vanilla ids (`to_u16` above).
            478 => NpcKind::MothronEgg,
            479 => NpcKind::MothronSpawn,
            480 => NpcKind::Medusa,
            481 => NpcKind::GreekSkeleton,
            482 => NpcKind::GraniteGolem,
            // W4-d40: the vanilla ids (`to_u16` above).
            483 => NpcKind::GraniteFlyer,
            484 => NpcKind::EnchantedNightcrawler,
            485 => NpcKind::Grubby,
            486 => NpcKind::Sluggy,
            487 => NpcKind::Buggy,
            // W4-d40: the vanilla ids (`to_u16` above).
            488 => NpcKind::TargetDummy,
            698 => NpcKind::GoblinScout,
            699 => NpcKind::Bird,
            700 => NpcKind::Pixie,
            701 => NpcKind::ArmoredSkeleton,
            702 => NpcKind::Mummy,
            703 => NpcKind::DarkMummy,
            704 => NpcKind::LightMummy,
            705 => NpcKind::Wraith,
            706 => NpcKind::CursedHammer,
            707 => NpcKind::EnchantedSword,
            708 => NpcKind::Mimic,
            709 => NpcKind::Unicorn,
            710 => NpcKind::GiantBat,
            711 => NpcKind::Corruptor,
            712 => NpcKind::DiggerHead,
            713 => NpcKind::DiggerBody,
            714 => NpcKind::DiggerTail,
            715 => NpcKind::SeekerHead,
            716 => NpcKind::SeekerBody,
            717 => NpcKind::SeekerTail,
            718 => NpcKind::Clinger,
            719 => NpcKind::AnglerFish,
            720 => NpcKind::GreenJellyfish,
            721 => NpcKind::Werewolf,
            722 => NpcKind::Clown,
            723 => NpcKind::SkeletonArcher,
            724 => NpcKind::VileSpit,
            725 => NpcKind::TheHungryII,
            726 => NpcKind::ChaosElemental,
            727 => NpcKind::Slimer,
            728 => NpcKind::Gastropod,
            729 => NpcKind::IlluminantBat,
            730 => NpcKind::PossessedArmor,
            228 => NpcKind::WyvernTail,
            266 => NpcKind::AngryBones,
            267 => NpcKind::CursedSkull,
            268 => NpcKind::OldMan,
            269 => NpcKind::ManEater,
            270 => NpcKind::Bunny,
            271 => NpcKind::BoneSerpentHead,
            272 => NpcKind::BoneSerpentBody,
            273 => NpcKind::BoneSerpentTail,
            274 => NpcKind::CorruptBunny,
            275 => NpcKind::Harpy,
            276 => NpcKind::Snatcher,
            277 => NpcKind::CorruptGoldfish,
            278 => NpcKind::Vulture,
            279 => NpcKind::Demon,
            280 => NpcKind::BlueJellyfish,
            // W4-d34 merge-side moves (367 §1): Crab@791 / DungeonGuardian@792
            // (the 281-285 squat cleared for d38's anchor band).
            791 => NpcKind::Crab,
            792 => NpcKind::DungeonGuardian,
            // W4-d37 — the NPCID 211-280 band's port-local protocol ids
            // (`to_u16` above; the vanilla 217-280 collide with the landed
            // bands). 354-399, free on main.
            354 => NpcKind::CochinealBeetle,
            355 => NpcKind::CyanBeetle,
            356 => NpcKind::LacBeetle,
            357 => NpcKind::SeaSnail,
            358 => NpcKind::Squid,
            359 => NpcKind::FlyingFish,
            360 => NpcKind::FlyingSnake,
            361 => NpcKind::Pirate,
            362 => NpcKind::GoldfishWalker,
            363 => NpcKind::HornetFatty,
            364 => NpcKind::HornetHoney,
            365 => NpcKind::HornetLeafy,
            366 => NpcKind::HornetSpikey,
            367 => NpcKind::HornetStingy,
            368 => NpcKind::JungleCreeper,
            369 => NpcKind::JungleCreeperWall,
            370 => NpcKind::BlackRecluseWall,
            371 => NpcKind::BloodCrawler,
            372 => NpcKind::BloodCrawlerWall,
            373 => NpcKind::BloodFeeder,
            374 => NpcKind::BloodJelly,
            375 => NpcKind::IceGolem,
            376 => NpcKind::RainbowSlime,
            377 => NpcKind::Eyezor,
            378 => NpcKind::Parrot,
            379 => NpcKind::Reaper,
            380 => NpcKind::ZombieMushroom,
            381 => NpcKind::ZombieMushroomHat,
            382 => NpcKind::FungoFish,
            383 => NpcKind::AnomuraFungus,
            384 => NpcKind::MushiLadybug,
            385 => NpcKind::FungiBulb,
            386 => NpcKind::GiantFungiBulb,
            387 => NpcKind::IchorSticker,
            388 => NpcKind::RustyArmoredBonesAxe,
            389 => NpcKind::RustyArmoredBonesFlail,
            390 => NpcKind::RustyArmoredBonesSword,
            391 => NpcKind::RustyArmoredBonesSwordNoArmor,
            392 => NpcKind::BlueArmoredBones,
            393 => NpcKind::BlueArmoredBonesMace,
            394 => NpcKind::BlueArmoredBonesNoPants,
            395 => NpcKind::BlueArmoredBonesSword,
            396 => NpcKind::HellArmoredBones,
            397 => NpcKind::HellArmoredBonesSpikeShield,
            398 => NpcKind::HellArmoredBonesMace,
            399 => NpcKind::HellArmoredBonesSword,
            // W4-d42 — the NPCID 578-630 roster's VANILLA ids (`to_u16`
            // above; the 108 union plan's vanilla-free band). 581/587/592/
            // 604/605/606/607/615 stay free for c4/c4b/e26b/c8.
            578 => NpcKind::DD2LightningBug,
            579 => NpcKind::BartenderUnconscious,
            580 => NpcKind::WalkingAntlion,
            582 => NpcKind::LarvaeAntlion,
            586 => NpcKind::ZombieMerman,
            590 => NpcKind::TorchZombie,
            591 => NpcKind::ArmedTorchZombie,
            593 => NpcKind::GoldGoldfishWalker,
            602 => NpcKind::Seagull,
            603 => NpcKind::Seagull2,
            608 => NpcKind::Grebe,
            609 => NpcKind::Grebe2,
            610 => NpcKind::Rat,
            611 => NpcKind::Owl,
            612 => NpcKind::WaterStrider,
            613 => NpcKind::GoldWaterStrider,
            614 => NpcKind::ExplosiveBunny,
            616 => NpcKind::Turtle,
            617 => NpcKind::TurtleJungle,
            618 => NpcKind::BloodNautilus,
            619 => NpcKind::BloodSquid,
            620 => NpcKind::GoblinShark,
            621 => NpcKind::BloodEelHead,
            622 => NpcKind::BloodEelBody,
            623 => NpcKind::BloodEelTail,
            624 => NpcKind::Gnome,
            625 => NpcKind::SeaTurtle,
            626 => NpcKind::Seahorse,
            627 => NpcKind::GoldSeahorse,
            628 => NpcKind::Dandelion,
            629 => NpcKind::IceMimic,
            630 => NpcKind::BloodMummy,
            // W4-c10 — the waterfowl family's vanilla ids (`to_u16` above).
            // E8 merge: 731-734 (the 362-365 slots are d37's); the
            // Seagull/Grebe + Rat..BloodMummy band are d42's arms above.
            731 => NpcKind::Duck,
            732 => NpcKind::Duck2,
            733 => NpcKind::DuckWhite,
            734 => NpcKind::DuckWhite2,
            // W4-c3: 694 keeps its vanilla id (`to_u16` above); the
            // 355/289 dup arms dropped per the d37 E8 tables (d34/d38 own).
            694 => NpcKind::WaterBoltMimic,
            // W4-c4 (the style-44 swimmers): 359 is the FlyingFish's
            // port-local id — d37's frozen-band row ADOPTED here (the
            // original 354 is retired: d37's CochinealBeetle owns it);
            // 509/581/587 keep their vanilla ids (`to_u16` above).
            // E8 merge: 359 decodes at d37's arm above (identical-id).
            509 => NpcKind::GiantFlyingAntlion,
            581 => NpcKind::FlyingAntlion,
            587 => NpcKind::EyeballFlyingFish,
            // W4-c4 (the style-16 swimmers): 405/406 are the
            // CorruptGoldfish/Arapaima port-local ids (the vanilla 57/157
            // slots are the GoblinThief's/BoundWizard's; the original
            // 229/237 squatted the reserved cs-owned holes — renumbered
            // to the free-port band, `to_u16` above); 241/465/592/607/
            // 615/688/692 keep their vanilla ids.
            // E8 merge drops: 405/241/688/692 (d34/d37/d43 own).
            406 => NpcKind::Arapaima,
            465 => NpcKind::CrimsonGoldfish,
            592 => NpcKind::GoldGoldfish,
            607 => NpcKind::Pupfish,
            615 => NpcKind::Dolphin,
            // W4-c7 — the firefly family's vanilla ids (`to_u16` above).
            // E8 merge renumber: 735/736 (the 355/358 slots are d37's).
            735 => NpcKind::Firefly,
            736 => NpcKind::LightningBug,
            654 => NpcKind::Lavafly,
            677 => NpcKind::Shimmerfly,
            // W4-c8 — the crawling-critter family's vanilla ids (`to_u16`
            // above).
            // E8 merge: 737/738/739 (the 357/374/375 slots are d37's);
            // the 448/484-487 dups dropped (d40's decode arms above).
            737 => NpcKind::Worm,
            738 => NpcKind::TruffleWorm,
            739 => NpcKind::TruffleWormDigger,
            606 => NpcKind::Maggot,
            // The snail family + its shimmer target (W4-c9): the vanilla
            // ids kept (`to_u16` above).
            // E8 merge: 740/741 (the 359/360 slots are d37's); the
            // 677 + 442/443/445/446/447 dups dropped (c7/d40's arms above).
            740 => NpcKind::Snail,
            741 => NpcKind::GlowingSnail,
            655 => NpcKind::MagmaSnail,
            // W4-c10 roster tail — the twelve critters' vanilla ids
            // (`to_u16` above).
            // E8 merge: the 12-critter tail dropped (c8/d40's arms above).
            // W4-d25 STYLE-26 wiring (the to_u16 rows above).
            772 => NpcKind::Wolf,
            546 => NpcKind::Tumbleweed,
            // W4-d120 — the chatter kinds' port-local band (`to_u16` above).
            241 => NpcKind::SantaClaus,
            242 => NpcKind::Truffle,
            243 => NpcKind::Steampunker,
            244 => NpcKind::Cyborg,
            // W4-d125 empress wiring (`to_u16` above — the 108 R4/E8
            // renumber: 636 protocol==cs, 742 the small-lane block).
            636 => NpcKind::EmpressOfLight,
            742 => NpcKind::PrismaticLacewing,
            // W4-g5 — the two unclaimed aiStyle-8 casters (`to_u16` above).
            827 => NpcKind::RuneWizard,
            693 => NpcKind::LibrarianSkeleton,
            // W4-g8 lane 1 batch 1 — E8 EXECUTED: the owned kinds decode
            // from 745-761; the DROP-PENDING dups keep their pushed ids
            // until the merge keep-one (see `to_u16`'s E8 note).
            745 => NpcKind::PinkJellyfish, // E8: 356 -> 745
            746 => NpcKind::Antlion, // E8: 357 -> 746
            747 => NpcKind::SpikeBall, // E8: 358 -> 747
            // W4-g8 lane 1 batch 2 — E8: the owned kinds decode 748-756.
            748 => NpcKind::WallCreeper, // E8: 366 -> 748
            749 => NpcKind::WallCreeperWall, // E8: 367 -> 749
            750 => NpcKind::IceElemental, // E8: 368 -> 750
            751 => NpcKind::Herpling, // E8: 369 -> 751
            752 => NpcKind::Derpling, // E8: 370 -> 752
            753 => NpcKind::CrimsonAxe, // E8: 371 -> 753
            754 => NpcKind::FloatyGross, // E8: 372 -> 754
            755 => NpcKind::LostGirl, // E8: 373 -> 755
            756 => NpcKind::Nymph, // E8: 374 -> 756
            // W4-g8 lane 1 batch 3 — E8: the owned bats decode 757-760.
            // W4-g8 lane 1 batch 4 — E8: RedDevil decodes from 761.
            // W4-g8 machine round 3 — the Vampire 159 census row (E8
            // COLLISION FIX: 388 is d37's RustyArmoredBonesAxe; 762 —
            // 788-790 stay reserved for d39's 3 unclaimed kinds).
            // The dragonflies (W4-d33): the vanilla ids kept (`to_u16`
            // above).
            595 => NpcKind::BlackDragonfly,
            596 => NpcKind::BlueDragonfly,
            597 => NpcKind::GreenDragonfly,
            598 => NpcKind::OrangeDragonfly,
            599 => NpcKind::RedDragonfly,
            600 => NpcKind::YellowDragonfly,
            601 => NpcKind::GoldDragonfly,
            // The fairy critters + the windy balloon (W4-d33 wiring):
            // d42 batch-2/3's wire rows, constants-copy (keep-one d37c-E3).
            583 => NpcKind::FairyCritterPink,
            584 => NpcKind::FairyCritterGreen,
            585 => NpcKind::FairyCritterBlue,
            594 => NpcKind::WindyBalloon,
            // W4-e21 — the bunny pair's port-local holes (`to_u16` above).
            237 => NpcKind::PartyBunny,
            // W4-f42 — the sandstorm debris + desert family (`to_u16`'s
            // 354..362 band).
            // W4-d9 wiring: the vanilla ids (`to_u16` above) — the d40
            // dragon rows + the ported 521 squidhead.
            521 => NpcKind::AncientCultistSquidhead,
            // W4-d39: the port-local 817+ band (`to_u16` above, 108 R7).
            817 => NpcKind::Butterfly,
            818 => NpcKind::ScorpionBlack,
            819 => NpcKind::Scorpion,
            820 => NpcKind::Grasshopper,
            821 => NpcKind::ChatteringTeethBomb,
            822 => NpcKind::CultistArcherBlue,
            823 => NpcKind::CultistArcherWhite,
            // W4-d41 — the cavern crawdads/shellies (`to_u16` above).
            494 => NpcKind::Crawdad,
            495 => NpcKind::Crawdad2,
            496 => NpcKind::GiantShelly,
            497 => NpcKind::GiantShelly2,
            // W4-d41 — the nine salamander colours (`to_u16` above).
            498 => NpcKind::Salamander,
            499 => NpcKind::Salamander2,
            500 => NpcKind::Salamander3,
            501 => NpcKind::Salamander4,
            502 => NpcKind::Salamander5,
            503 => NpcKind::Salamander6,
            504 => NpcKind::Salamander7,
            505 => NpcKind::Salamander8,
            506 => NpcKind::Salamander9,
            // W4-d41 — the antlions + the Dune Splicer chain (`to_u16`).
            508 => NpcKind::GiantWalkingAntlion,
            510 => NpcKind::DuneSplicerHead,
            511 => NpcKind::DuneSplicerBody,
            512 => NpcKind::DuneSplicerTail,
            // W4-d41 — the Tomb Crawler chain + the solar fragments.
            513 => NpcKind::TombCrawlerHead,
            514 => NpcKind::TombCrawlerBody,
            515 => NpcKind::TombCrawlerTail,
            519 => NpcKind::SolarGoop,
            // W4-d41 — the ancient orbs + the desert ghouls.
            522 => NpcKind::AncientLight,
            523 => NpcKind::AncientDoom,
            524 => NpcKind::DesertGhoul,
            525 => NpcKind::DesertGhoulCorruption,
            526 => NpcKind::DesertGhoulCrimson,
            // W4-d41 — the hallowed ghoul + the lamias + the sand poachers.
            527 => NpcKind::DesertGhoulHallow,
            528 => NpcKind::DesertLamiaLight,
            529 => NpcKind::DesertLamiaDark,
            530 => NpcKind::DesertScorpionWalk,
            531 => NpcKind::DesertScorpionWall,
            // W4-d41 — the basilisk + the tortured soul + the slimes +
            // the bride.
            532 => NpcKind::DesertBeast,
            534 => NpcKind::DemonTaxCollector,
            535 => NpcKind::SlimeSpiked,
            536 => NpcKind::TheBride,
            537 => NpcKind::SandSlime,
            // W4-d41 — the critters + the tumbler.
            538 => NpcKind::SquirrelRed,
            539 => NpcKind::SquirrelGold,
            // The luck critters (W4-e26, E8 keep-one EXECUTED): the Frog
            // decodes at its E8 wire slot 744 (`to_u16` above); the Duck
            // 362 arm DROPS to c10 (the canonical waterfowl owner). The
            // ladybug pair 604/605 stays e26's (the d42 ownership flip).
            744 => NpcKind::Frog,
            604 => NpcKind::LadyBug,
            605 => NpcKind::GoldLadyBug,
            // W4-g10 lane 3 — the town pets decode at their vanilla ids
            // (`to_u16` above; `NPCID.cs:11951`/`:11953`/`:11989`/
            // `:12017`/`:12033-12043`).
            637 => NpcKind::TownCat,
            638 => NpcKind::TownDog,
            656 => NpcKind::TownBunny,
            670 => NpcKind::TownSlimeBlue,
            678 => NpcKind::TownSlimeGreen,
            679 => NpcKind::TownSlimeOld,
            680 => NpcKind::TownSlimePurple,
            681 => NpcKind::TownSlimeRainbow,
            682 => NpcKind::TownSlimeRed,
            683 => NpcKind::TownSlimeYellow,
            684 => NpcKind::TownSlimeCopper,
            // Doc-sweep lane 36 — the ShimmerSlime's vanilla id (`to_u16`).
            676 => NpcKind::ShimmerSlime,
            // W4-f18 first-20 port — the critters' 237.. band (`to_u16`
            // above). `NPCID.cs:11387-11395`.
            // W4-f18 batch 2 — the critters' 242.. band (`to_u16` above).
            // `NPCID.cs:11397-11405`.
            // W4-f18 batch 3 — the critters' 247.. band (`to_u16` above;
            // 250/288 are the AngryNimbus/DungeonSpirit's). `NPCID.cs:
            // 11407-11427`.
            // W4-f18 batch 4 — the 254.. band (`to_u16` above).
            // `NPCID.cs:11415-11541`.
            // The Shimmerfly (W4-c9's rowing, adopted by W4-g62): the
            // vanilla id kept (`to_u16` above).
            // W4-d42 - the vanilla-free band (d42 2d7ac08c3's rowing,
            // adopted by W4-g62 for the 620→20 trail row).
            _ => return None,
        })
    
            // d23's g8/d43-owned decode arms (see the to_u16 note above).
            401 => NpcKind::IceBat,
            402 => NpcKind::Lavabat,
            403 => NpcKind::GiantFlyingFox,
            404 => NpcKind::RedDevil,
            405 => NpcKind::VampireBat,
            400 => NpcKind::Vampire,
            411 => NpcKind::VileSpitEoW,
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
                // W4-d120 — the four town kinds' `townNPC = true` rows
                // (`NPC.cs:10793`/`:11005`/`:11254`/`:11684`): the
                // save/resident-count set (the M4-TOWN-19 precedent).
                | NpcKind::SantaClaus
                | NpcKind::Truffle
                | NpcKind::Steampunker
                | NpcKind::Cyborg
                // W4-d41: the Tavernkeep's `townNPC = true` + `friendly =
                // true` rows (`NPC.cs:16263-16264`, g58b) — the ctor's
                // friendly seed and the save filter admit him.
                | NpcKind::DD2Bartender
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
                // W4-d34: the Bone Serpent chain (`NPC.cs:51301`/`:51350`/
                // `:51436`'s type-39/40 lists).
                | NpcKind::BoneSerpentHead
                | NpcKind::BoneSerpentBody
                | NpcKind::BoneSerpentTail
                // W4-c8: the TruffleWormDigger (375) is the same shape —
                // an AI_006 kind (`NPC.cs:13781` aiStyle 6, the `:51598`
                // 1000-px dig list, the `:51730-51734` 6/0.15 tunables,
                // the `:51861-51865` aim inversion) with NO chain (375
                // sits outside the `:51301`/`:51436` lists;
                // `is_worm_head`/`is_worm_mid` stay false).
                | NpcKind::TruffleWormDigger
                // W4-d41 — the Dune Splicer chain (510-512): the head's
                // own fixed-budget spawn (`NPC.cs:51253-51276`, the
                // Next(12,21) roll) + the shared follower trail.
                | NpcKind::DuneSplicerHead
                | NpcKind::DuneSplicerBody
                | NpcKind::DuneSplicerTail
                // W4-d41 — the Tomb Crawler chain (513-515): the head's
                // own fixed-budget spawn (`NPC.cs:51229-51252`, the
                // Next(6,10) roll) + the shared follower trail.
                | NpcKind::TombCrawlerHead
                | NpcKind::TombCrawlerBody
                | NpcKind::TombCrawlerTail
                // W4-f18 batch 3 — the TruffleWormDigger 375 IS an AI_006
                // kind (`NPC.cs:51730-51734`'s 6/0.15 tunables, the
                // `:51598` 1000-px dig list) — a lone burrower with no
                // chain (type 375 sits outside the `:51301` spawn lists,
                // so `is_worm_head` stays false; the StardustWormHead
                // precedent). Its dig aim is INVERTED (`:51861-51865`).
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
                // W4-d35 (C): the 95 worm chain shares the
                // head's life pool (NPC.cs:51305-51363). Inert until wiring.
                | NpcKind::DiggerHead
                // W4-d35 (D): the 96/97/98/99/100 worm chain shares the
                // head's life pool (NPC.cs:51305-51363). Inert until wiring.
                | NpcKind::DiggerBody
                | NpcKind::DiggerTail
                | NpcKind::SeekerHead
                | NpcKind::SeekerBody
                | NpcKind::SeekerTail
                | NpcKind::WyvernTail
                // W4-d34: the Bone Serpent shares the head's life pool
                // (`NPC.cs:51307-51308`: ai[3]/realLife = whoAmI).
                | NpcKind::BoneSerpentHead
                | NpcKind::BoneSerpentBody
                | NpcKind::BoneSerpentTail
                // W4-d41 — the Dune Splicer chain shares the head's life
                // pool (`NPC.cs:51255-51256`/`:51268-51269`: `ai[3] =
                // whoAmI`, `realLife = whoAmI` on the head and every link).
                | NpcKind::DuneSplicerHead
                | NpcKind::DuneSplicerBody
                | NpcKind::DuneSplicerTail
                // W4-d41 — the Tomb Crawler chain shares the head's life
                // pool (`NPC.cs:51231-51232`/`:51244-51245`).
                | NpcKind::TombCrawlerHead
                | NpcKind::TombCrawlerBody
                | NpcKind::TombCrawlerTail
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
                | NpcKind::BoneSerpentHead
                // W4-d41 — the Dune Splicer head (510): its own
                // fixed-budget chain (`NPC.cs:51253-51276`), NOT the
                // shared `:51301` budget list — the init's budget match
                // rolls Next(12,21) for it directly.
                | NpcKind::DuneSplicerHead
                // W4-d41 — the Tomb Crawler head (513): its own
                // fixed-budget chain (`NPC.cs:51229-51252`), NOT the
                // shared `:51301` budget list.
                | NpcKind::TombCrawlerHead
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
                | NpcKind::BoneSerpentBody
                // W4-d41 — the Dune Splicer body (511): the head's chain
                // spawns it directly (the budget loop), so the mid-body
                // self-chain flag is NOT used for the spawn — but the
                // followerless-link death gate (`:51419-51433`) reads it.
                | NpcKind::DuneSplicerBody
                // W4-d41 — the Tomb Crawler body (514): the head's chain
                // spawns it directly; the followerless-link death gate
                // (`:51421-51433`) reads it.
                | NpcKind::TombCrawlerBody
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
            NpcKind::BoneSerpentHead => NpcKind::BoneSerpentBody,
            NpcKind::BoneSerpentBody | NpcKind::BoneSerpentTail => NpcKind::BoneSerpentTail,

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
            // W4-d41 — the Dune Splicer chain (`NPC.cs:51262-51266`):
            // the head spawns the body, the body self-chains while the
            // budget holds, the last link is the tail.
            NpcKind::DuneSplicerHead => NpcKind::DuneSplicerBody,
            NpcKind::DuneSplicerBody | NpcKind::DuneSplicerTail => NpcKind::DuneSplicerTail,
            // W4-d41 — the Tomb Crawler chain (`NPC.cs:51238-51242`).
            NpcKind::TombCrawlerHead => NpcKind::TombCrawlerBody,
            NpcKind::TombCrawlerBody | NpcKind::TombCrawlerTail => NpcKind::TombCrawlerTail,
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
            // W4-f18 batch 3 — the TruffleWormDigger 375's 6/0.15
            // (`NPC.cs:51730-51734`).
            NpcKind::SolarCrawltipedeHead
            | NpcKind::SolarCrawltipedeBody
            | NpcKind::SolarCrawltipedeTail => (10.0, 0.3),
            NpcKind::DevourerHead => (9.0, 0.1),
            NpcKind::GiantWormHead => (6.0, 0.05),
            // W4-d34: the Bone Serpent head's 9/0.1 (`NPC.cs:51764-51767`;
            // the getGood 10/0.12 is unported — single-difficulty).
            NpcKind::BoneSerpentHead => (9.0, 0.1),

            NpcKind::EaterOfWorldsHead => (10.0, 0.07),
            // W4-d41 — the Dune Splicer head's 10/0.25 base
            // (`NPC.cs:51716-51717`); the sandstorm 16/0.35 override
            // (`:51711-51712`) reads the target's ZoneSandstorm — the
            // worm pass's ctx carries it (`worm_head_step`).
            NpcKind::DuneSplicerHead => (10.0, 0.25),
            // W4-d41 — the Tomb Crawler head's 7/0.1
            // (`NPC.cs:51682-51686`).
            NpcKind::TombCrawlerHead => (7.0, 0.1),
            // The wyvern head's 11/0.25 (`:51720-51724`). The kind stays
            // OUT of `is_worm`/`is_worm_head` (the StardustWormHead
            // precedent): the shared pass's budget chain + dig head are
            // NOT the wyvern's machine — the fixed 14-link chain and the
            // altitude-gate flyer live in `npc_ai_wyvern.rs` (UNWIRED).
            NpcKind::WyvernHead => (11.0, 0.25),
            // W4-c8 — the TruffleWormDigger's 6/0.15 (`NPC.cs:51730-
            // 51734`): the lone fleeing digger (the StardustWormHead
            // precedent — `is_worm` true, no chain).
            NpcKind::TruffleWormDigger => (6.0, 0.15),
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
                // W4-d41: the Tavernkeep IS a town NPC (`NPC.cs:16263`'s
                // `townNPC = true`, g58b's census — no separate event twin).
                | NpcKind::DD2Bartender
        )
        // W4-g10 lane 3 — the town pets are housed town NPCs too
        // (`townNPC = true`, `housingCategory 1`): the housing scans,
        // the scene-metrics count (`SceneMetrics.cs:739` counts every
        // `townNPC`), the Rotten Egg's refusal (`Projectile.cs:23572`)
        // and the world-file town block (`WorldFile.cs:1717`) all read
        // the flag. They are NOT residents ([`is_town_resident`] stays
        // pet-free — `CollectTaxes`/the arrival ladder exclude them).
        || self.is_town_pet()
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
                    // W4-d34: the Old Man carries `townNPC = true`
                    // (`NPC.cs:9199`) — `isLikeATownNPC` (`NPC.cs:6856`).
                    | NpcKind::OldMan
                    // W4-d37 — the Pirate (229) is a town NPC (`NPC.cs:11955`
                    // `townNPC = true`) with no ported housing entry yet (the
                    // TravellingMerchant precedent). `is_friendly` reads this.
                    | NpcKind::Pirate
                    // W4-d40: the Skeleton Merchant's `isLikeATownNPC` row
                    // (`NPC.cs:6860`: `type == 453`).
                    | NpcKind::SkeletonMerchant
                    // W4-d120 — the four town kinds' `townNPC` flag
                    // (`NPC.cs:10793`/`:11005`/`:11254`/`:11684`); the
                    // talk/melee/net consumers read this set.
                    | NpcKind::SantaClaus
                    | NpcKind::Truffle
                    | NpcKind::Steampunker
                    | NpcKind::Cyborg
            )
            // W4-g10 lane 3 — the town pets write `townNPC = true` too
            // (`NPC.cs:17209`/`:17297`/`:17483`/`:17596`), so
            // `isLikeATownNPC` holds for them (the stat ladder's
            // `:53014` gate + the door-open gate read it).
            || self.is_town_pet()
    }

    /// `NPCID.Sets.IsTownPet` (`ID/NPCID.cs:10118`: 637/638/656/670/
    /// 678-684) — the kind-level twin of `npc_ai_town::is_town_pet`
    /// (which keys on the net id). The pets are town NPCs for the
    /// `townNPC`-flag consumers (housing, the scene-metrics count, the
    /// Rotten Egg's refusal, the world-file town block) but NOT for the
    /// resident counts (`CollectTaxes` `Player.cs:24145` and the
    /// arrival ladder exclude them) nor the party
    /// (`BirthdayParty.CanNPCParty` `:66-69`).
    pub fn is_town_pet(self) -> bool {
        matches!(
            self,
            NpcKind::TownCat
                | NpcKind::TownDog
                | NpcKind::TownBunny
                | NpcKind::TownSlimeBlue
                | NpcKind::TownSlimeGreen
                | NpcKind::TownSlimeOld
                | NpcKind::TownSlimePurple
                | NpcKind::TownSlimeRainbow
                | NpcKind::TownSlimeRed
                | NpcKind::TownSlimeYellow
                | NpcKind::TownSlimeCopper
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
                    // W4-d120 — the bound Tavernkeep's `friendly = true`
                    // row (`NPC.cs:16556`), NOT townNPC (`Sets.BoundTownNPCs`,
                    // `NPCID.cs:10441`).
                    | NpcKind::BartenderUnconscious
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
            // The aiStyle-14 flyer family (W4-d23): NPCHit1 for
            // 48/93/121/137/150/151/152/158/159/634/660 (`NPC.cs:9368`/
            // `:10056`/`:10474`/`:10743`/`:10894`/`:10909`/`:10924`/
            // `:15475`/`:15489`/`:17170`/`:17346`), NPCHit21 for 62/156
            // (`:9576`/`:10983`), NPCHit23 for 226 (`:11920`).
            NpcKind::Harpy
            | NpcKind::GiantBat
            | NpcKind::Slimer
            | NpcKind::IlluminantBat
            | NpcKind::IceBat
            | NpcKind::Lavabat
            | NpcKind::GiantFlyingFox
            | NpcKind::VampireBat
            | NpcKind::Vampire
            | NpcKind::SporeBat
            | NpcKind::QueenSlimeMinionPurple => "NPC_Hit_1",
            NpcKind::Demon | NpcKind::RedDevil => "NPC_Hit_21",
            NpcKind::FlyingSnake => "NPC_Hit_23",
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
            // W4-c4 (the style-16 swimmers): NPCHit1 for all but the
            // BloodFeeder's NPCHit13 (`NPC.cs:9498`/`:10999`/`:12115`/
            // `:14492`/`:16688`/`:16803`/`:16901`/`:17662`/`:17718`).
            // E8 merge drops: CorruptGoldfish/BloodFeeder (d34/d37 rows
            // above), Pufferfish/Orca (d43 lands later).
            NpcKind::Arapaima
            | NpcKind::CrimsonGoldfish
            | NpcKind::GoldGoldfish
            | NpcKind::Pupfish
            | NpcKind::Dolphin => "NPC_Hit_1",
            // W4-e21 — the bunny pair: `SoundID.NPCHit1` (`NPC.cs:9344`).
            NpcKind::Bunny | NpcKind::PartyBunny => "NPC_Hit_1",
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
            // W4-g10 lane 3 — the town pets: `SoundID.NPCHit1`
            // (`NPC.cs:17221` for 637/638, `:17305` for 656, `:17491`/
            // `:17604` for the slime rows).
            NpcKind::TownCat
            | NpcKind::TownDog
            | NpcKind::TownBunny
            | NpcKind::TownSlimeBlue
            | NpcKind::TownSlimeGreen
            | NpcKind::TownSlimeOld
            | NpcKind::TownSlimePurple
            | NpcKind::TownSlimeRainbow
            | NpcKind::TownSlimeRed
            | NpcKind::TownSlimeYellow
            | NpcKind::TownSlimeCopper => "NPC_Hit_1",
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
            // W4-d34 kinds 1-70 (`NPC.cs` SetDefaults HitSound).
            NpcKind::AngryBones => "NPC_Hit_2",   // :9116
            NpcKind::CursedSkull => "NPC_Hit_2",  // :9158
            NpcKind::OldMan => "NPC_Hit_1",       // :9207
            NpcKind::ManEater => "NPC_Hit_1",     // :9303
            NpcKind::Bunny => "NPC_Hit_1",        // :9344
            NpcKind::BoneSerpentHead | NpcKind::BoneSerpentBody | NpcKind::BoneSerpentTail => "NPC_Hit_2", // :9235/:9252/:9270
            NpcKind::CorruptBunny => "NPC_Hit_1",  // :9356
            NpcKind::Harpy => "NPC_Hit_1",         // :9368
            NpcKind::Snatcher => "NPC_Hit_1",         // :9484
            NpcKind::CorruptGoldfish => "NPC_Hit_1",  // :9498
            NpcKind::Vulture => "NPC_Hit_28",         // :9563
            NpcKind::Demon => "NPC_Hit_21",           // :9576
            NpcKind::BlueJellyfish => "NPC_Hit_25",   // :9591
            NpcKind::Crab => "NPC_Hit_1",             // :9648
            NpcKind::DungeonGuardian => "NPC_Hit_2",  // :9660
            // W4-d38 b7 — SetDefaults HitSound rows.
            NpcKind::SlimeRibbonWhite => "NPC_Hit_1",
            NpcKind::SlimeRibbonYellow => "NPC_Hit_1",
            NpcKind::SlimeRibbonGreen => "NPC_Hit_1",
            NpcKind::SlimeRibbonRed => "NPC_Hit_1",
            NpcKind::BunnyXmas => "NPC_Hit_1",
            // W4-d38 b6 — SetDefaults HitSound rows.
            NpcKind::DemonEyeSpaceship => "NPC_Hit_1",
            NpcKind::SkeletonTopHat => "NPC_Hit_2",
            NpcKind::SkeletonAstonaut => "NPC_Hit_2",
            NpcKind::SkeletonAlien => "NPC_Hit_2",
            NpcKind::PumpkingBlade => "NPC_Hit_1",
            // W4-d38 b5 — SetDefaults HitSound rows.
            NpcKind::SlimeMasked => "NPC_Hit_1",
            NpcKind::BunnySlimed => "NPC_Hit_1",
            NpcKind::HoppinJack => "NPC_Hit_1",
            NpcKind::Ghost => "NPC_Hit_1",
            NpcKind::DemonEyeOwl => "NPC_Hit_1",
            // W4-d38 b4 — SetDefaults HitSound rows.
            NpcKind::BirdBlue => "NPC_Hit_1",
            NpcKind::BirdRed => "NPC_Hit_1",
            NpcKind::Squirrel => "NPC_Hit_1",
            NpcKind::Mouse => "NPC_Hit_1",
            NpcKind::Raven => "NPC_Hit_1",
            // W4-d38 b3 — SetDefaults HitSound rows.
            NpcKind::TacticalSkeleton => "NPC_Hit_2",
            NpcKind::SkeletonCommando => "NPC_Hit_2",
            NpcKind::AngryBonesBig => "NPC_Hit_2",
            NpcKind::AngryBonesBigMuscle => "NPC_Hit_2",
            NpcKind::AngryBonesBigHelmet => "NPC_Hit_2",
            // W4-d38 b2 — SetDefaults HitSound rows.
            NpcKind::DiabolistWhite => "NPC_Hit_2",
            NpcKind::BoneLee => "NPC_Hit_2",
            NpcKind::GiantCursedSkull => "NPC_Hit_2",
            NpcKind::Paladin => "NPC_Hit_4",
            NpcKind::SkeletonSniper => "NPC_Hit_2",
            // W4-d38 b1 — SetDefaults HitSound rows.
            NpcKind::RaggedCaster => "NPC_Hit_2",
            NpcKind::RaggedCasterOpenCoat => "NPC_Hit_2",
            NpcKind::Necromancer => "NPC_Hit_2",
            NpcKind::NecromancerArmored => "NPC_Hit_2",
            NpcKind::DiabolistRed => "NPC_Hit_2",
            // W4-d40: the SetDefaults HitSound rows (`NPC.cs`).
            NpcKind::ArmedZombie
            | NpcKind::ArmedZombieEskimo
            | NpcKind::ArmedZombiePincussion
            | NpcKind::ArmedZombieSlimed
            | NpcKind::ArmedZombieSwamp => "NPC_Hit_1",
            // W4-d40: the SetDefaults HitSound rows (`NPC.cs`).
            NpcKind::ArmedZombieTwiggy
            | NpcKind::ArmedZombieCenx
            | NpcKind::CultistDevote => "NPC_Hit_1",
            NpcKind::CultistTablet => "NPC_Hit_4",
            NpcKind::CultistBossClone => "NPC_Hit_2",
            // W4-d40: the SetDefaults HitSound rows (`NPC.cs`).
            NpcKind::GoldBird
            | NpcKind::GoldBunny
            | NpcKind::GoldButterfly
            | NpcKind::GoldFrog
            | NpcKind::GoldGrasshopper => "NPC_Hit_1",
            // W4-d40: the SetDefaults HitSound rows (`NPC.cs`).
            NpcKind::GoldMouse
            | NpcKind::GoldWorm => "NPC_Hit_1",
            NpcKind::BoneThrowingSkeleton
            | NpcKind::BoneThrowingSkeleton2
            | NpcKind::BoneThrowingSkeleton3 => "NPC_Hit_2",
            // W4-d40: the SetDefaults HitSound rows (`NPC.cs`).
            NpcKind::BoneThrowingSkeleton4
            | NpcKind::SkeletonMerchant => "NPC_Hit_2",
            NpcKind::CultistDragonHead
            | NpcKind::CultistDragonBody1
            | NpcKind::CultistDragonBody2 => "NPC_Hit_56",
            // W4-d40: the SetDefaults HitSound rows (`NPC.cs`).
            NpcKind::CultistDragonBody3
            | NpcKind::CultistDragonBody4
            | NpcKind::CultistDragonTail => "NPC_Hit_56",
            NpcKind::Butcher => "NPC_Hit_7",
            NpcKind::CreatureFromTheDeep => "NPC_Hit_1",
            // W4-d40: the SetDefaults HitSound rows (`NPC.cs`).
            NpcKind::Fritz
            | NpcKind::Nailhead
            | NpcKind::CrimsonBunny
 => "NPC_Hit_1",
            NpcKind::Psycho => "NPC_Hit_48",
            // W4-d40: the SetDefaults HitSound rows (`NPC.cs`).
            NpcKind::DeadlySphere => "NPC_Hit_34",
            NpcKind::DrManFly => "NPC_Hit_35",
            NpcKind::ThePossessed => "NPC_Hit_47",
            NpcKind::CrimsonPenguin => "NPC_Hit_1",
            NpcKind::ShadowFlameApparition => "NPC_Hit_52",
            // W4-d40: the SetDefaults HitSound rows (`NPC.cs`).
            NpcKind::BigMimicCorruption
            | NpcKind::BigMimicCrimson
            | NpcKind::BigMimicHallow
            | NpcKind::BigMimicJungle => "NPC_Hit_4",
            NpcKind::Mothron => "NPC_Hit_44",
            // W4-d40: the SetDefaults HitSound rows (`NPC.cs`).
            NpcKind::MothronEgg
            | NpcKind::MothronSpawn
            | NpcKind::Medusa => "NPC_Hit_1",
            NpcKind::GreekSkeleton => "NPC_Hit_2",
            NpcKind::GraniteGolem => "NPC_Hit_41",
            // W4-d40: the SetDefaults HitSound rows (`NPC.cs`).
            NpcKind::GraniteFlyer => "NPC_Hit_7",
            NpcKind::EnchantedNightcrawler
            | NpcKind::Grubby
            | NpcKind::Sluggy
            | NpcKind::Buggy => "NPC_Hit_1",
            // W4-d40: the SetDefaults HitSound rows (`NPC.cs`).
            NpcKind::TargetDummy => "NPC_Hit_15",
            // W4-c3: 694 DD2_BookStaffCast (`NPC.cs:17745` - the
            // `CreateTrackable("dd2_book_staff_cast", 3)` style,
            // `SoundID.cs:1676`; the `_0` pin, the DD2 families'
            // convention). The 34/289 NPCHit2 rows are d34/d38's above.
            NpcKind::WaterBoltMimic => "dd2_book_staff_cast_0",
            // W4-c4 (the style-44 swimmers): 224/587 NPCHit1
            // (`NPC.cs:11893`/`:16631`), 509/581 NPCHit32
            // (`:14850`/`:14877`).
            NpcKind::EyeballFlyingFish => "NPC_Hit_1",
            NpcKind::GiantFlyingAntlion | NpcKind::FlyingAntlion => "NPC_Hit_32",
            // W4-c7 — the firefly family's `SoundID.NPCHit1` rows
            // (`NPC.cs:13525`/`:13567`/`:17273`/`:17587`).
            NpcKind::Firefly | NpcKind::LightningBug | NpcKind::Lavafly | NpcKind::Shimmerfly => "NPC_Hit_1",
            // W4-c8 — the crawling-critter family's `SoundID.NPCHit1`
            // rows (`NPC.cs:13553`/`:13770`/`:13785`/`:14340`/`:14584`/
            // `:16789`).
            // E8 merge: the GoldWorm/EnchantedNightcrawler/Grubby/
            // Sluggy/Buggy rows are d40's above (the dup drop).
            NpcKind::Worm
            | NpcKind::TruffleWorm
            | NpcKind::TruffleWormDigger
            | NpcKind::Maggot => "NPC_Hit_1",
            // The snail family + its shimmer target (W4-c9): NPCHit1
            // each (`NPC.cs:13581` / `:13595` / `:17288` / `:17587`).
            // E8 merge: Shimmerfly's row is c7's above; the 5 gold rows
            // are d40's above (the dup drops).
            NpcKind::Snail | NpcKind::GlowingSnail | NpcKind::MagmaSnail => {
                "NPC_Hit_1"
            }
            // W4-c10 — the waterfowl family's `SoundID.NPCHit1` rows
            // (`NPC.cs:13621`/`:13633`/`:13645`/`:13657`/`:16747`/
            // `:16759`/`:16817`/`:16829`).
            // E8 merge: the Seagull/Grebe rows are d42's above.
            NpcKind::Duck
            | NpcKind::Duck2
            | NpcKind::DuckWhite
            | NpcKind::DuckWhite2 => "NPC_Hit_1",
            // W4-c10 roster tail — the twelve critters' `SoundID.NPCHit1`
            // rows (`NPC.cs:13553`, `:14255`/`:14270`/`:14283`/`:14298`/
            // `:14311`/`:14326`/`:14340`, the `:14584` range row).
            // E8 merge: the 12-critter hit rows are c8/d40's above.
            // The man-eater family (W4-d23): all six NPCHit1
            // (`NPC.cs:9303`/`:9484`/`:10197`/`:11218`/`:12342`/`:12358`).
            NpcKind::ManEater
            | NpcKind::Snatcher
            | NpcKind::Clinger
            | NpcKind::AngryTrapper
            | NpcKind::FungiBulb
            | NpcKind::GiantFungiBulb => "NPC_Hit_1",
            // The aiStyle-9 spit/flare family (W4-d23): 112/666/516 all
            // carry `HitSound = null` (`NPC.cs:10334`/`:17426`/`:15402`) —
            // no hit cue (the `_ => return None` fallthrough).
            // W4-c6 style-59 wiring: 328 `SoundID.NPCHit1` (`NPC.cs:13166`,
            // `SoundID.cs:1257` = (3, 1)). The blade is `dontTakeDamage`,
            // so the cue only plays on a force-kill path — the row is
            // carried for the census.
            // W4-d120 — SetDefaults HitSound rows: 142/160/178 NPCHit1
            // (`NPC.cs:10801`/`:11013`/`:11262`), 209 NPCHit4 (`:11690`),
            // 579 NPCHit1 (`:16563`).
            NpcKind::SantaClaus | NpcKind::Truffle | NpcKind::Steampunker => "NPC_Hit_1",
            NpcKind::Cyborg => "NPC_Hit_4",
            // W4-d41 — the cavern crawdads' `SoundID.NPCHit33`
            // (`NPC.cs:14674`) and the shellies' `NPCHit38` (`:14686`).
            NpcKind::Crawdad | NpcKind::Crawdad2 => "NPC_Hit_33",
            NpcKind::GiantShelly | NpcKind::GiantShelly2 => "NPC_Hit_38",
            // W4-d41 — the salamanders' `SoundID.NPCHit50`
            // (`NPC.cs:14699`).
            NpcKind::Salamander
            | NpcKind::Salamander2
            | NpcKind::Salamander3
            | NpcKind::Salamander4
            | NpcKind::Salamander5
            | NpcKind::Salamander6
            | NpcKind::Salamander7
            | NpcKind::Salamander8
            | NpcKind::Salamander9 => "NPC_Hit_50",
            // W4-d41 — the antlion charger's `SoundID.NPCHit31`
            // (`NPC.cs:14836`) and the swarmer's `NPCHit32` (`:14850`).
            NpcKind::GiantWalkingAntlion => "NPC_Hit_31",
            NpcKind::GiantFlyingAntlion => "NPC_Hit_32",
            // W4-d41 — the Dune Splicer chain's `SoundID.NPCHit1`
            // (`NPC.cs:14782`/`:14799`/`:14816`).
            NpcKind::DuneSplicerHead
            | NpcKind::DuneSplicerBody
            | NpcKind::DuneSplicerTail => "NPC_Hit_1",
            // W4-d41 — the Tomb Crawler chain's `SoundID.NPCHit1`
            // (`NPC.cs:14727`/`:14744`/`:14761`).
            NpcKind::TombCrawlerHead
            | NpcKind::TombCrawlerBody
            | NpcKind::TombCrawlerTail => "NPC_Hit_1",
            // W4-d41 — the solar fragments write `HitSound = null`
            // (`NPC.cs:15402`/`:15417`) — no hit sound (the null guard).
            // W4-d41 — the Ancient Light's `HitSound = null`
            // (`NPC.cs:15788`); the Ancient Doom's `NPCHit1` (`:15803`).
            NpcKind::AncientDoom => "NPC_Hit_1",
            // W4-d41 — the desert ghouls' `SoundID.NPCHit37`
            // (`NPC.cs:14890`).
            NpcKind::DesertGhoul
            | NpcKind::DesertGhoulCorruption
            | NpcKind::DesertGhoulCrimson => "NPC_Hit_37",
            // W4-d41 — the hallowed ghoul's `NPCHit37` (`NPC.cs:14890`),
            // the lamias' `NPCHit1` (`:14928`), the sand poachers'
            // `NPCHit13` (`:14941`/`:14955`).
            NpcKind::DesertGhoulHallow => "NPC_Hit_37",
            NpcKind::DesertLamiaLight | NpcKind::DesertLamiaDark => "NPC_Hit_1",
            NpcKind::DesertScorpionWalk | NpcKind::DesertScorpionWall => "NPC_Hit_13",
            // W4-d41 — the basilisk's + the tortured soul's + the
            // slimes' + the bride's `NPCHit1` (`NPC.cs:14968`/`:15821`/
            // `:15838`/`:15851`/`:15866`).
            NpcKind::DesertBeast
            | NpcKind::DemonTaxCollector
            | NpcKind::SlimeSpiked
            | NpcKind::TheBride
            | NpcKind::SandSlime => "NPC_Hit_1",
            // W4-d41 — the critters' `NPCHit1` (`NPC.cs:15881`/`:15893`/
            // `:9344`), the tumbler's `NPCHit11` (`:15980`).
            NpcKind::SquirrelRed | NpcKind::SquirrelGold | NpcKind::PartyBunny => "NPC_Hit_1",
            // Doc-sweep lane 36 — `SoundID.NPCHit1` (`NPC.cs:17576`).
            NpcKind::ShimmerSlime => "NPC_Hit_1",
            // The Shimmerfly (W4-c9's rowing, adopted by W4-g62):
            // NPCHit1 (`NPC.cs:17587`).
            NpcKind::Shimmerfly => "NPC_Hit_1",
            // W4-d42 - the roster's HitSound rows.
            // The wyvern family: `SoundID.NPCHit7` (`NPC.cs:9950`/`:9967`/
            // `:9984`/`:10002`/`:10020`/`:10038`).
            NpcKind::WyvernHead
            | NpcKind::WyvernLegs
            | NpcKind::WyvernBody
            | NpcKind::WyvernBody2
            | NpcKind::WyvernBody3
            | NpcKind::WyvernTail => "NPC_Hit_7",
            // W4-d37 — the NPCID 211-280 band's `SetDefaults` HitSound rows
            // (the `this.HitSound = SoundID.NPCHitN;` cites in the enum).
            NpcKind::CochinealBeetle
            | NpcKind::CyanBeetle
            | NpcKind::LacBeetle
            | NpcKind::SeaSnail
            | NpcKind::Squid
            | NpcKind::FlyingFish
            | NpcKind::Pirate
            | NpcKind::GoldfishWalker
            | NpcKind::HornetFatty
            | NpcKind::HornetHoney
            | NpcKind::HornetLeafy
            | NpcKind::HornetSpikey
            | NpcKind::HornetStingy
            | NpcKind::JungleCreeper
            | NpcKind::JungleCreeperWall
            | NpcKind::RainbowSlime
            | NpcKind::Eyezor
            | NpcKind::ZombieMushroom
            | NpcKind::ZombieMushroomHat
            | NpcKind::FungoFish
            | NpcKind::AnomuraFungus
            | NpcKind::FungiBulb
            | NpcKind::GiantFungiBulb => "NPC_Hit_1",
            NpcKind::FlyingSnake => "NPC_Hit_23",
            NpcKind::BlackRecluseWall => "NPC_Hit_29",
            NpcKind::BloodCrawler | NpcKind::BloodCrawlerWall => "NPC_Hit_20",
            NpcKind::BloodFeeder | NpcKind::BloodJelly | NpcKind::IchorSticker => "NPC_Hit_13",
            NpcKind::IceGolem => "NPC_Hit_5",
            NpcKind::Parrot => "NPC_Hit_46",
            NpcKind::Reaper => "NPC_Hit_49",
            NpcKind::MushiLadybug => "NPC_Hit_45",
            NpcKind::RustyArmoredBonesAxe
            | NpcKind::RustyArmoredBonesFlail
            | NpcKind::RustyArmoredBonesSword
            | NpcKind::RustyArmoredBonesSwordNoArmor
            | NpcKind::BlueArmoredBones
            | NpcKind::BlueArmoredBonesMace
            | NpcKind::BlueArmoredBonesNoPants
            | NpcKind::BlueArmoredBonesSword
            | NpcKind::HellArmoredBones
            | NpcKind::HellArmoredBonesSpikeShield
            | NpcKind::HellArmoredBonesMace
            | NpcKind::HellArmoredBonesSword => "NPC_Hit_2",
            // W4-d42 - the roster's HitSound rows.
            NpcKind::DD2LightningBug => "dd2_lightning_bug_hurt_0",
            NpcKind::BartenderUnconscious => "NPC_Hit_1",
            NpcKind::WalkingAntlion => "NPC_Hit_31",
            NpcKind::LarvaeAntlion => "NPC_Hit_31",
            // W4-d42 - the roster's HitSound rows.
            NpcKind::ZombieMerman => "NPC_Hit_18",
            // W4-d42 - the roster's HitSound rows.
            NpcKind::TorchZombie => "NPC_Hit_1",
            NpcKind::ArmedTorchZombie => "NPC_Hit_1",
            NpcKind::GoldGoldfishWalker => "NPC_Hit_1",
            // W4-d42 - the roster's HitSound rows.
            // W4-d42 - the roster's HitSound rows.
            NpcKind::Seagull => "NPC_Hit_1",
            NpcKind::Seagull2 => "NPC_Hit_1",
            // W4-d42 - the roster's HitSound rows.
            NpcKind::Grebe => "NPC_Hit_1",
            NpcKind::Grebe2 => "NPC_Hit_1",
            // W4-d42 - the roster's HitSound rows.
            NpcKind::Rat => "NPC_Hit_1",
            NpcKind::Owl => "NPC_Hit_1",
            NpcKind::WaterStrider => "NPC_Hit_1",
            NpcKind::GoldWaterStrider => "NPC_Hit_1",
            NpcKind::ExplosiveBunny => "NPC_Hit_1",
            // W4-d42 - the roster's HitSound rows.
            NpcKind::Turtle => "NPC_Hit_1",
            NpcKind::TurtleJungle => "NPC_Hit_1",
            NpcKind::BloodNautilus => "NPC_Hit_1",
            NpcKind::BloodSquid => "NPC_Hit_1",
            // W4-d42 - the roster's HitSound rows.
            NpcKind::GoblinShark => "dd2_drakin_hurt_0",
            NpcKind::BloodEelHead => "NPC_Hit_1",
            NpcKind::BloodEelBody => "NPC_Hit_1",
            NpcKind::BloodEelTail => "NPC_Hit_1",
            NpcKind::Gnome => "NPC_Hit_1",
            // W4-d42 - the roster's HitSound rows.
            NpcKind::SeaTurtle => "NPC_Hit_1",
            NpcKind::Seahorse => "NPC_Hit_1",
            NpcKind::GoldSeahorse => "NPC_Hit_1",
            NpcKind::Dandelion => "NPC_Hit_1",
            NpcKind::IceMimic => "NPC_Hit_4",
            // W4-d42 - the roster's HitSound rows.
            NpcKind::BloodMummy => "NPC_Hit_1",
            // W4-d35 (G): the 122/137/140 HitSound rows.
            NpcKind::Gastropod => "NPC_Hit_1",
            NpcKind::IlluminantBat => "NPC_Hit_1",
            NpcKind::PossessedArmor => "NPC_Hit_4",
            // W4-d35 (F): the 110/112/116/120/121 HitSound rows.
            NpcKind::SkeletonArcher => "NPC_Hit_2",
            NpcKind::TheHungryII => "NPC_Hit_9",
            NpcKind::ChaosElemental => "NPC_Hit_1",
            NpcKind::Slimer => "NPC_Hit_1",
            // W4-d35 (E): the 101/102/103/104/109 HitSound rows.
            NpcKind::Clinger => "NPC_Hit_1",
            NpcKind::AnglerFish => "NPC_Hit_1",
            NpcKind::GreenJellyfish => "NPC_Hit_25",
            NpcKind::Werewolf => "NPC_Hit_6",
            NpcKind::Clown => "NPC_Hit_1",
            // W4-d35 (D): the 96/97/98/99/100 HitSound rows.
            NpcKind::DiggerBody => "NPC_Hit_1",
            NpcKind::DiggerTail => "NPC_Hit_1",
            NpcKind::SeekerHead => "NPC_Hit_1",
            NpcKind::SeekerBody => "NPC_Hit_1",
            NpcKind::SeekerTail => "NPC_Hit_1",
            // W4-d35 (C): the 85/86/93/94/95 HitSound rows.
            NpcKind::Mimic => "NPC_Hit_4",
            NpcKind::Unicorn => "NPC_Hit_12",
            NpcKind::GiantBat => "NPC_Hit_1",
            NpcKind::Corruptor => "NPC_Hit_1",
            NpcKind::DiggerHead => "NPC_Hit_1",
            // W4-d35 (B): the 79/80/82/83/84 HitSound rows.
            NpcKind::DarkMummy => "NPC_Hit_1",
            NpcKind::LightMummy => "NPC_Hit_1",
            NpcKind::Wraith => "NPC_Hit_54",
            NpcKind::CursedHammer => "NPC_Hit_4",
            NpcKind::EnchantedSword => "NPC_Hit_4",
            // W4-d35 (A): the 73/74/75/77/78 HitSound rows.
            NpcKind::GoblinScout => "NPC_Hit_1",
            NpcKind::Bird => "NPC_Hit_1",
            NpcKind::Pixie => "NPC_Hit_5",
            NpcKind::ArmoredSkeleton => "NPC_Hit_2",
            NpcKind::Mummy => "NPC_Hit_1",
            // W4-d25 STYLE-26 wiring: 86 NPCHit12 (`NPC.cs:9934`),
            // 155 NPCHit1 (`:10969`), 546 NPCHit11 (`:15980`). The
            // Unicorn row mirrors d35 (C).
            NpcKind::Wolf => "NPC_Hit_1",
            NpcKind::Tumbleweed => "NPC_Hit_11",
            // W4-d125: 636 `SoundID.NPCHit1` (`NPC.cs:17196`), 661
            // `NPCHit1` (`:17358`).
            NpcKind::EmpressOfLight | NpcKind::PrismaticLacewing => "NPC_Hit_1",
            // W4-g5 — the two casters' `HitSound` rows: `SoundID.NPCHit2`
            // (`NPC.cs:11162`/`:17729`).
            NpcKind::RuneWizard | NpcKind::LibrarianSkeleton => "NPC_Hit_2",
            // W4-g8 lane 1 batch 1 — the `HitSound` rows: 61 NPCHit28
            // (`NPC.cs:9563`), 63/64 NPCHit25 (`:9591`/`:9605`), 69 NPCHit31
            // (`:9674`), 70 NPCHit1 (`:9689`), 75 NPCHit5 (`:9781`), 82
            // NPCHit54 (`:9862`), 83/84/85 NPCHit4 (`:9887`/`:9900`/`:9913`).
            NpcKind::Vulture => "NPC_Hit_28",
            NpcKind::BlueJellyfish | NpcKind::PinkJellyfish => "NPC_Hit_25",
            NpcKind::Antlion => "NPC_Hit_31",
            NpcKind::SpikeBall => "NPC_Hit_1",
            NpcKind::CursedHammer | NpcKind::EnchantedSword | NpcKind::Mimic => "NPC_Hit_4",
            // W4-g8 lane 1 batch 2 — the `HitSound` rows: 103 NPCHit25
            // (`NPC.cs:10225`), 122/174/182/195/196 NPCHit1 (`:10488`/
            // `:11203`/`:11326`/`:11500`/`:11514`), 164/165 NPCHit29
            // (`:11067`/`:11082`), 169 NPCHit5 (`:11122`), 177 NPCHit22
            // (`:11247`), 179 NPCHit4 (`:11274`).
            NpcKind::Gastropod | NpcKind::Herpling | NpcKind::FloatyGross
            | NpcKind::LostGirl | NpcKind::Nymph => "NPC_Hit_1",
            NpcKind::WallCreeper | NpcKind::WallCreeperWall => "NPC_Hit_29",
            NpcKind::IceElemental => "NPC_Hit_5",
            NpcKind::Derpling => "NPC_Hit_22",
            NpcKind::CrimsonAxe => "NPC_Hit_4",
            // W4-g8 lane 1 batch 3 — the `HitSound` rows: 34 NPCHit2
            // (`NPC.cs:9158`), 62 NPCHit21 (`:9576`), 93/121/137/150/151/
            // 152/158 NPCHit1 (`:10056`/`:10474`/`:10743`/`:10894`/`:10909`/
            // `:10925`/`:15475`), 226 NPCHit23 (`:11920`).
            NpcKind::CursedSkull => "NPC_Hit_2",
            NpcKind::Demon => "NPC_Hit_21",
            NpcKind::GiantBat | NpcKind::Slimer | NpcKind::IlluminantBat
            | NpcKind::IceBat | NpcKind::Lavabat | NpcKind::GiantFlyingFox
            | NpcKind::VampireBat => "NPC_Hit_1",
            // W4-g8 lane 1 batch 4 — the `HitSound` rows: 221/48 NPCHit1
            // (`NPC.cs:11849`/`:9368`), 156 NPCHit21 (`:10983`).
            NpcKind::Squid | NpcKind::Harpy => "NPC_Hit_1",
            NpcKind::RedDevil => "NPC_Hit_21",
            // W4-g8 machine round 3 — the Vampire 159's NPCHit1
            // (`NPC.cs:15489`).
            NpcKind::Vampire => "NPC_Hit_1",
            // The dragonflies (W4-d33): NPCHit1 each (`NPC.cs:16729`,
            // the shared 595-601 row).
            NpcKind::BlackDragonfly
            | NpcKind::BlueDragonfly
            | NpcKind::GreenDragonfly
            | NpcKind::OrangeDragonfly
            | NpcKind::RedDragonfly
            | NpcKind::YellowDragonfly
            | NpcKind::GoldDragonfly => "NPC_Hit_1",
            // The fairy critters (W4-d33 wiring): NPCHit1 each
            // (`NPC.cs:16591`, the shared 583-585 row). d42 batch-2's
            // rows, constants-copy (keep-one d37c-E3). The WindyBalloon
            // 594 writes NO HitSound (`NPC.cs:16707-16719` — it only
            // pops) and rides the None fall-through.
            NpcKind::FairyCritterPink
            | NpcKind::FairyCritterGreen
            | NpcKind::FairyCritterBlue => "NPC_Hit_1",
            // W4-f42 — the sandstorm debris + desert family's hit cues:
            // 546 NPCHit11 (`NPC.cs:15980`), 61 NPCHit28 (`:9563`),
            // 69 NPCHit31 (`:9674`), 78/79/80 NPCHit1 (`:9807`/`:9820`/
            // `:9833`), 630 NPCHit1 (`:17115`), 580 NPCHit31 (`:14862`),
            // 581 NPCHit32 (`:14877`).
            NpcKind::Antlion | NpcKind::WalkingAntlion => "NPC_Hit_31",
            NpcKind::FlyingAntlion => "NPC_Hit_32",
            NpcKind::Mummy
            | NpcKind::DarkMummy
            | NpcKind::BloodMummy
            | NpcKind::LightMummy => "NPC_Hit_1",
            // W4-d9 wiring (d40 rows): the SetDefaults HitSound rows —
            // the dragon family NPCHit56 (`NPC.cs:14425`/`:14443`/
            // `:14462`), the 521 squidhead NPCHit1 (`NPC.cs:15776`).
            | NpcKind::CultistDragonBody2
            | NpcKind::CultistDragonBody3
            NpcKind::AncientCultistSquidhead => "NPC_Hit_1",
            // W4-d39: the Butterfly's `SoundID.NPCHit1` row
            // (`NPC.cs:13539`); the scorpion pair `:13669`, the
            // grasshopper `:13819`.
            NpcKind::Butterfly
            | NpcKind::ScorpionBlack
            | NpcKind::Scorpion
            | NpcKind::Grasshopper => "NPC_Hit_1",
            // W4-d39 batch 3: the teeth bomb (`NPC.cs:13834`) + the
            // archer pair (`:13847`) — NPCHit1.
            NpcKind::ChatteringTeethBomb
            | NpcKind::CultistArcherBlue
            | NpcKind::CultistArcherWhite => "NPC_Hit_1",
            // The luck critters (W4-e26, E8 keep-one): NPCHit1 each
            // (`NPC.cs:13608` / `:16771`).
            NpcKind::Frog | NpcKind::LadyBug | NpcKind::GoldLadyBug => "NPC_Hit_1",
            // W4-f18 first-20 port — the critters' `SoundID.NPCHit1` rows
            // (`NPC.cs:13525`/`:13539`/`:13553`/`:13567`/`:13581`).
            NpcKind::Firefly | NpcKind::Butterfly | NpcKind::Worm | NpcKind::LightningBug | NpcKind::Snail => "NPC_Hit_1",
            // W4-f18 batch 2 — the critters' `SoundID.NPCHit1` rows
            // (`NPC.cs:13591`/`:13605`/`:13617`/`:13629`/`:13641`).
            NpcKind::GlowingSnail | NpcKind::Frog | NpcKind::Duck | NpcKind::Duck2 | NpcKind::DuckWhite => "NPC_Hit_1",
            // W4-f18 batch 3 — the critters' `SoundID.NPCHit1` rows
            // (`NPC.cs:13653`/`:13665`/`:13766`/`:13781`).
            NpcKind::DuckWhite2 | NpcKind::ScorpionBlack | NpcKind::Scorpion | NpcKind::TruffleWorm | NpcKind::TruffleWormDigger => "NPC_Hit_1",
            // W4-f18 batch 4 — the `SoundID.NPCHit1` rows (`NPC.cs:13815`/
            // `:13829`/`:13843`/`:8616`).
            NpcKind::Grasshopper | NpcKind::ChatteringTeethBomb | NpcKind::CultistArcherBlue | NpcKind::CultistArcherWhite | NpcKind::ArmedZombie => "NPC_Hit_1",
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
            // The aiStyle-14 flyer family (W4-d23): NPCDeath1 for 48/660
            // (`NPC.cs:9370`/`:17347`), NPCDeath4 for
            // 93/150/151/152/634 (`:10058`/`:10896`/`:10911`/`:10926`/
            // `:17172`), NPCDeath6 for 137/158/159 (`:10745`/`:15477`/
            // `:15490`), NPCDeath24 for 62/156 (`:9578`/`:10985`),
            // NPCDeath26 for 226 (`:11922`). The Slimer 121 writes NO
            // DeathSound row (`:10466-10478`) — the `_ => return None`
            // fallthrough (the C# null default).
            NpcKind::Harpy | NpcKind::QueenSlimeMinionPurple => "NPC_Killed_1",
            NpcKind::GiantBat
            | NpcKind::IceBat
            | NpcKind::Lavabat
            | NpcKind::GiantFlyingFox
            | NpcKind::SporeBat => "NPC_Killed_4",
            NpcKind::IlluminantBat | NpcKind::VampireBat | NpcKind::Vampire => "NPC_Killed_6",
            NpcKind::Demon | NpcKind::RedDevil => "NPC_Killed_24",
            NpcKind::FlyingSnake => "NPC_Killed_26",
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
            // W4-c4 (the style-16 swimmers): NPCDeath1 for all but the
            // BloodFeeder's NPCDeath19 (`NPC.cs:9499`/`:11000`/`:12116`/
            // `:14493`/`:16689`/`:16804`/`:16902`/`:17663`/`:17719`).
            // E8 merge drops: CorruptGoldfish/BloodFeeder (d34/d37 rows
            // above), Pufferfish/Orca (d43 lands later).
            NpcKind::Arapaima
            | NpcKind::CrimsonGoldfish
            | NpcKind::GoldGoldfish
            | NpcKind::Pupfish
            | NpcKind::Dolphin => "NPC_Killed_1",
            // W4-e21 — the bunny pair: `SoundID.NPCDeath1` (`NPC.cs:9345`).
            NpcKind::Bunny | NpcKind::PartyBunny => "NPC_Killed_1",
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
            // W4-g10 lane 3 — the town pets: `SoundID.NPCDeath6`
            // (`NPC.cs:17222` for 637/638, `:17306` for 656, `:17492`/
            // `:17605` for the slime rows).
            NpcKind::TownCat
            | NpcKind::TownDog
            | NpcKind::TownBunny
            | NpcKind::TownSlimeBlue
            | NpcKind::TownSlimeGreen
            | NpcKind::TownSlimeOld
            | NpcKind::TownSlimePurple
            | NpcKind::TownSlimeRainbow
            | NpcKind::TownSlimeRed
            | NpcKind::TownSlimeYellow
            | NpcKind::TownSlimeCopper => "NPC_Killed_6",
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
            // W4-d34 kinds 1-70 (`NPC.cs` SetDefaults DeathSound).
            NpcKind::AngryBones => "NPC_Killed_2",   // :9117
            NpcKind::CursedSkull => "NPC_Killed_2",  // :9159
            NpcKind::OldMan => "NPC_Killed_1",       // :9208
            NpcKind::ManEater => "NPC_Killed_1",     // :9305
            NpcKind::Bunny => "NPC_Killed_1",        // :9345
            NpcKind::BoneSerpentHead | NpcKind::BoneSerpentBody | NpcKind::BoneSerpentTail => "NPC_Killed_5", // :9236/:9253/:9271
            NpcKind::CorruptBunny => "NPC_Killed_1",  // :9357
            NpcKind::Harpy => "NPC_Killed_1",         // :9370
            NpcKind::Snatcher => "NPC_Killed_1",         // :9486
            NpcKind::CorruptGoldfish => "NPC_Killed_1",  // :9499
            NpcKind::Vulture => "NPC_Killed_31",         // :9564
            NpcKind::Demon => "NPC_Killed_24",           // :9578
            NpcKind::BlueJellyfish => "NPC_Killed_28",   // :9592
            NpcKind::Crab => "NPC_Killed_1",             // :9649
            NpcKind::DungeonGuardian => "NPC_Killed_2",  // :9661
            // W4-d38 b7 — SetDefaults DeathSound rows.
            NpcKind::SlimeRibbonWhite => "NPC_Killed_1",
            NpcKind::SlimeRibbonYellow => "NPC_Killed_1",
            NpcKind::SlimeRibbonGreen => "NPC_Killed_1",
            NpcKind::SlimeRibbonRed => "NPC_Killed_1",
            NpcKind::BunnyXmas => "NPC_Killed_1",
            // W4-d38 b6 — SetDefaults DeathSound rows.
            NpcKind::DemonEyeSpaceship => "NPC_Killed_1",
            NpcKind::SkeletonTopHat => "NPC_Killed_2",
            NpcKind::SkeletonAstonaut => "NPC_Killed_2",
            NpcKind::SkeletonAlien => "NPC_Killed_2",
            NpcKind::PumpkingBlade => "NPC_Killed_1",
            // W4-d38 b5 — SetDefaults DeathSound rows.
            NpcKind::SlimeMasked => "NPC_Killed_1",
            NpcKind::BunnySlimed => "NPC_Killed_1",
            NpcKind::HoppinJack => "NPC_Killed_1",
            NpcKind::Ghost => "NPC_Killed_6",
            NpcKind::DemonEyeOwl => "NPC_Killed_1",
            // W4-d38 b4 — SetDefaults DeathSound rows.
            NpcKind::BirdBlue => "NPC_Killed_1",
            NpcKind::BirdRed => "NPC_Killed_1",
            NpcKind::Squirrel => "NPC_Killed_1",
            NpcKind::Mouse => "NPC_Killed_4",
            NpcKind::Raven => "NPC_Killed_1",
            // W4-d38 b3 — SetDefaults DeathSound rows.
            NpcKind::TacticalSkeleton => "NPC_Killed_2",
            NpcKind::SkeletonCommando => "NPC_Killed_2",
            NpcKind::AngryBonesBig => "NPC_Killed_2",
            NpcKind::AngryBonesBigMuscle => "NPC_Killed_2",
            NpcKind::AngryBonesBigHelmet => "NPC_Killed_2",
            // W4-d38 b2 — SetDefaults DeathSound rows.
            NpcKind::DiabolistWhite => "NPC_Killed_2",
            NpcKind::BoneLee => "NPC_Killed_2",
            NpcKind::GiantCursedSkull => "NPC_Killed_2",
            NpcKind::Paladin => "NPC_Killed_6",
            NpcKind::SkeletonSniper => "NPC_Killed_2",
            // W4-d38 b1 — SetDefaults DeathSound rows.
            NpcKind::RaggedCaster => "NPC_Killed_2",
            NpcKind::RaggedCasterOpenCoat => "NPC_Killed_2",
            NpcKind::Necromancer => "NPC_Killed_2",
            NpcKind::NecromancerArmored => "NPC_Killed_2",
            NpcKind::DiabolistRed => "NPC_Killed_2",
            // W4-d40: the SetDefaults DeathSound rows (`NPC.cs`).
            NpcKind::ArmedZombie
            | NpcKind::ArmedZombieEskimo
            | NpcKind::ArmedZombiePincussion
            | NpcKind::ArmedZombieSlimed
            | NpcKind::ArmedZombieSwamp => "NPC_Killed_2",
            // W4-d40: the SetDefaults DeathSound rows (`NPC.cs`).
            NpcKind::ArmedZombieTwiggy
            | NpcKind::ArmedZombieCenx
            | NpcKind::CultistBossClone => "NPC_Killed_2",
            NpcKind::CultistTablet => "NPC_Killed_14",
            NpcKind::CultistDevote => "NPC_Killed_1",
            // W4-d40: the SetDefaults DeathSound rows (`NPC.cs`).
            NpcKind::GoldBird
            | NpcKind::GoldBunny
            | NpcKind::GoldButterfly
            | NpcKind::GoldFrog
            | NpcKind::GoldGrasshopper => "NPC_Killed_1",
            // W4-d40: the SetDefaults DeathSound rows (`NPC.cs`).
            NpcKind::GoldMouse => "NPC_Killed_4",
            NpcKind::GoldWorm => "NPC_Killed_1",
            NpcKind::BoneThrowingSkeleton
            | NpcKind::BoneThrowingSkeleton2
            | NpcKind::BoneThrowingSkeleton3 => "NPC_Killed_2",
            // W4-d40: the SetDefaults DeathSound rows (`NPC.cs`).
            NpcKind::BoneThrowingSkeleton4
            | NpcKind::SkeletonMerchant => "NPC_Killed_2",
            NpcKind::CultistDragonHead
            | NpcKind::CultistDragonBody1
            | NpcKind::CultistDragonBody2 => "NPC_Killed_60",
            // W4-d40: the SetDefaults DeathSound rows (`NPC.cs`).
            NpcKind::CultistDragonBody3
            | NpcKind::CultistDragonBody4
            | NpcKind::CultistDragonTail => "NPC_Killed_60",
            NpcKind::Butcher => "NPC_Killed_17",
            NpcKind::CreatureFromTheDeep => "NPC_Killed_1",
            // W4-d40: the SetDefaults DeathSound rows (`NPC.cs`).
            NpcKind::Fritz
            | NpcKind::Nailhead
            | NpcKind::CrimsonBunny
 => "NPC_Killed_1",
            NpcKind::Psycho => "NPC_Killed_50",
            // W4-d40: the SetDefaults DeathSound rows (`NPC.cs`).
            NpcKind::DeadlySphere => "NPC_Killed_37",
            NpcKind::DrManFly => "NPC_Killed_38",
            NpcKind::ThePossessed => "NPC_Killed_49",
            NpcKind::CrimsonPenguin => "NPC_Killed_1",
            NpcKind::ShadowFlameApparition => "NPC_Killed_55",
            // W4-d40: the SetDefaults DeathSound rows (`NPC.cs`).
            NpcKind::BigMimicCorruption
            | NpcKind::BigMimicCrimson
            | NpcKind::BigMimicHallow
            | NpcKind::BigMimicJungle => "NPC_Killed_6",
            NpcKind::Mothron => "NPC_Killed_46",
            // W4-d40: the SetDefaults DeathSound rows (`NPC.cs`).
            NpcKind::MothronEgg
            | NpcKind::MothronSpawn
            | NpcKind::Medusa => "NPC_Killed_1",
            NpcKind::GreekSkeleton => "NPC_Killed_2",
            NpcKind::GraniteGolem => "NPC_Killed_43",
            // W4-d40: the SetDefaults DeathSound rows (`NPC.cs`).
            NpcKind::GraniteFlyer => "NPC_Killed_6",
            NpcKind::EnchantedNightcrawler
            | NpcKind::Grubby
            | NpcKind::Sluggy
            | NpcKind::Buggy => "NPC_Killed_1",
            // W4-d40: the SetDefaults DeathSound rows (`NPC.cs`).
            NpcKind::TargetDummy => "NPC_Killed_2",
            // W4-c3: 694 NPCDeath52 (`NPC.cs:17746`); the 34/289
            // NPCDeath2 rows are d34/d38's above.
            NpcKind::WaterBoltMimic => "NPC_Killed_52",
            // W4-c4 (the style-44 swimmers): 224/587 NPCDeath1
            // (`NPC.cs:11895`/`:16633`), 509/581 NPCDeath35
            // (`:14851`/`:14878`).
            NpcKind::EyeballFlyingFish => "NPC_Killed_1",
            NpcKind::GiantFlyingAntlion | NpcKind::FlyingAntlion => "NPC_Killed_35",
            // W4-c7 — the firefly family's `SoundID.NPCDeath1` rows
            // (`NPC.cs:13526`/`:13568`/`:17274`/`:17588`).
            NpcKind::Firefly | NpcKind::LightningBug | NpcKind::Lavafly | NpcKind::Shimmerfly => "NPC_Killed_1",
            // W4-c8 — the crawling-critter family's `SoundID.NPCDeath1`
            // rows (`NPC.cs:13554`/`:13771`/`:13786`/`:14341`/`:14585`/
            // `:16790`).
            // E8 merge: the 5 d40-owned dups dropped (rows above).
            NpcKind::Worm
            | NpcKind::TruffleWorm
            | NpcKind::TruffleWormDigger
            | NpcKind::Maggot => "NPC_Killed_1",
            // The snail family + its shimmer target (W4-c9): NPCDeath1
            // each (`NPC.cs:13582` / `:13596` / `:17289` / `:17588`).
            // E8 merge: Shimmerfly's row is c7's above; the 5 gold rows
            // are d40's above (the dup drops).
            NpcKind::Snail | NpcKind::GlowingSnail | NpcKind::MagmaSnail => {
                "NPC_Killed_1"
            }
            // W4-c10 — the waterfowl family's `SoundID.NPCDeath1` rows
            // (`NPC.cs:13622`/`:13634`/`:13646`/`:13658`/`:16748`/
            // `:16760`/`:16818`/`:16830`).
            // E8 merge: the Seagull/Grebe rows are d42's above.
            NpcKind::Duck
            | NpcKind::Duck2
            | NpcKind::DuckWhite
            | NpcKind::DuckWhite2 => "NPC_Killed_1",
            // W4-c10 roster tail — the GoldMouse's `SoundID.NPCDeath4`
            // (`NPC.cs:14327`, (4, 4) `ID/SoundID.cs:1318`) — the mouse
            // squeak; every other critter `NPCDeath1`.
            // E8 merge: the GoldMouse NPCDeath4 + the 12-critter death
            // rows are d40/c8's above (the dup drops).
            // The man-eater family (W4-d23): all six NPCDeath1
            // (`NPC.cs:9305`/`:9486`/`:10199`/`:11220`/`:12344`/`:12360`).
            NpcKind::ManEater
            | NpcKind::Snatcher
            | NpcKind::Clinger
            | NpcKind::AngryTrapper
            | NpcKind::FungiBulb
            | NpcKind::GiantFungiBulb => "NPC_Killed_1",
            // The aiStyle-9 spit/flare family (W4-d23): 112/666 NPCDeath9
            // (`NPC.cs:10335`/`:17427`); 516 `DeathSound = null`
            // (`:15403`) — the `_ => return None` fallthrough.
            NpcKind::VileSpit | NpcKind::VileSpitEoW => "NPC_Killed_9",
            // W4-c6 style-59 wiring: 328 `SoundID.NPCDeath1` (`NPC.cs:13167`,
            // `SoundID.cs:1315` = (4, 1)) — the despawn's `checkDead` plays
            // it (`:78186-78194`); the one `Next(-10,11) * 0.01` pitch roll
            // (`LegacySoundPlayer.cs:693`) rides the death-sound batch on
            // the `UpdateNpcs` stream (`world_sim/npcs.rs`'s dead pass,
            // the S1-E-21c convention — NOT the style-59 arm).
            // W4-d120 — SetDefaults DeathSound rows: 142/160/178/209
            // NPCDeath1 (`NPC.cs:10802`/`:11014`/`:11263`/`:11692`),
            // 579 NPCDeath6 (`:16564`).
            NpcKind::SantaClaus | NpcKind::Truffle | NpcKind::Steampunker | NpcKind::Cyborg => {
                "NPC_Killed_1"
            }
            // W4-d41 — the cavern crawdads' `SoundID.NPCDeath36`
            // (`NPC.cs:14675`) and the shellies' `NPCDeath41` (`:14687`).
            NpcKind::Crawdad | NpcKind::Crawdad2 => "NPC_Killed_36",
            NpcKind::GiantShelly | NpcKind::GiantShelly2 => "NPC_Killed_41",
            // W4-d41 — the salamanders' `SoundID.NPCDeath53`
            // (`NPC.cs:14700`).
            NpcKind::Salamander
            | NpcKind::Salamander2
            | NpcKind::Salamander3
            | NpcKind::Salamander4
            | NpcKind::Salamander5
            | NpcKind::Salamander6
            | NpcKind::Salamander7
            | NpcKind::Salamander8
            | NpcKind::Salamander9 => "NPC_Killed_53",
            // W4-d41 — the antlion charger's `SoundID.NPCDeath34`
            // (`NPC.cs:14837`) and the swarmer's `NPCDeath35` (`:14851`).
            NpcKind::GiantWalkingAntlion => "NPC_Killed_34",
            NpcKind::GiantFlyingAntlion => "NPC_Killed_35",
            // W4-d41 — the Dune Splicer chain's `SoundID.NPCDeath1`
            // (`NPC.cs:14783`/`:14800`/`:14817`).
            NpcKind::DuneSplicerHead
            | NpcKind::DuneSplicerBody
            | NpcKind::DuneSplicerTail => "NPC_Killed_1",
            // W4-d41 — the Tomb Crawler chain's `SoundID.NPCDeath1`
            // (`NPC.cs:14728`/`:14745`/`:14762`).
            NpcKind::TombCrawlerHead
            | NpcKind::TombCrawlerBody
            | NpcKind::TombCrawlerTail => "NPC_Killed_1",
            // W4-d41 — the solar fragments write `DeathSound = null`
            // (`NPC.cs:15403`/`:15418`) — no death sound (the null guard).
            // W4-d41 — the Ancient Light's `DeathSound = null`
            // (`NPC.cs:15789`); the Ancient Doom's `NPCDeath6` (`:15804`).
            NpcKind::AncientDoom => "NPC_Killed_6",
            // W4-d41 — the desert ghouls' `SoundID.NPCDeath40`
            // (`NPC.cs:14891`).
            NpcKind::DesertGhoul
            | NpcKind::DesertGhoulCorruption
            | NpcKind::DesertGhoulCrimson => "NPC_Killed_40",
            // W4-d41 — the hallowed ghoul's `NPCDeath40` (`NPC.cs:14891`),
            // the lamias' `NPCDeath1` (`:14929`), the sand poachers'
            // `NPCDeath19` (`:14942`/`:14956`).
            NpcKind::DesertGhoulHallow => "NPC_Killed_40",
            NpcKind::DesertLamiaLight | NpcKind::DesertLamiaDark => "NPC_Killed_1",
            NpcKind::DesertScorpionWalk | NpcKind::DesertScorpionWall => "NPC_Killed_19",
            // W4-d41 — the basilisk's + the tortured soul's + the
            // slimes' `NPCDeath1` (`NPC.cs:14969`/`:15822`/`:15839`/
            // `:15867`), the bride's `NPCDeath2` (`:15852`).
            NpcKind::DesertBeast
            | NpcKind::DemonTaxCollector
            | NpcKind::SlimeSpiked
            | NpcKind::SandSlime => "NPC_Killed_1",
            NpcKind::TheBride => "NPC_Killed_2",
            // W4-d41 — the critters' `NPCDeath1` (`NPC.cs:15882`/`:15894`/
            // `:9345`), the tumbler's `NPCDeath15` (`:15981`).
            NpcKind::SquirrelRed | NpcKind::SquirrelGold | NpcKind::PartyBunny => "NPC_Killed_1",
            // Doc-sweep lane 36 — `SoundID.NPCDeath1` (`NPC.cs:17577`).
            NpcKind::ShimmerSlime => "NPC_Killed_1",
            // The Shimmerfly (W4-c9's rowing, adopted by W4-g62):
            // NPCDeath1 (`NPC.cs:17588`).
            NpcKind::Shimmerfly => "NPC_Killed_1",
            // W4-d42 - the roster's DeathSound rows (614's null row
            // rides the None fall-through).
            // The wyvern family: `SoundID.NPCDeath8` (`NPC.cs:9951`/
            // `:9968`/`:9985`/`:10003`/`:10021`/`:10039`).
            NpcKind::WyvernHead
            | NpcKind::WyvernLegs
            | NpcKind::WyvernBody
            | NpcKind::WyvernBody2
            | NpcKind::WyvernBody3
            | NpcKind::WyvernTail => "NPC_Killed_8",
            // W4-d37 — the NPCID 211-280 band's `SetDefaults` DeathSound rows
            // (the `this.DeathSound = SoundID.NPCDeathN;` cites in the enum).
            NpcKind::CochinealBeetle | NpcKind::CyanBeetle | NpcKind::LacBeetle => "NPC_Killed_16",
            NpcKind::SeaSnail
            | NpcKind::Squid
            | NpcKind::FlyingFish
            | NpcKind::Pirate
            | NpcKind::GoldfishWalker
            | NpcKind::HornetFatty
            | NpcKind::HornetHoney
            | NpcKind::HornetLeafy
            | NpcKind::HornetSpikey
            | NpcKind::HornetStingy
            | NpcKind::JungleCreeper
            | NpcKind::JungleCreeperWall
            | NpcKind::RainbowSlime
            | NpcKind::FungiBulb
            | NpcKind::GiantFungiBulb => "NPC_Killed_1",
            NpcKind::FlyingSnake => "NPC_Killed_26",
            NpcKind::BlackRecluseWall => "NPC_Killed_32",
            NpcKind::BloodCrawler | NpcKind::BloodCrawlerWall => "NPC_Killed_23",
            NpcKind::BloodFeeder | NpcKind::BloodJelly | NpcKind::IchorSticker => "NPC_Killed_19",
            NpcKind::IceGolem => "NPC_Killed_7",
            NpcKind::Eyezor
            | NpcKind::RustyArmoredBonesAxe
            | NpcKind::RustyArmoredBonesFlail
            | NpcKind::RustyArmoredBonesSword
            | NpcKind::RustyArmoredBonesSwordNoArmor
            | NpcKind::BlueArmoredBones
            | NpcKind::BlueArmoredBonesMace
            | NpcKind::BlueArmoredBonesNoPants
            | NpcKind::BlueArmoredBonesSword
            | NpcKind::HellArmoredBones
            | NpcKind::HellArmoredBonesSpikeShield
            | NpcKind::HellArmoredBonesMace
            | NpcKind::HellArmoredBonesSword => "NPC_Killed_2",
            NpcKind::Parrot => "NPC_Killed_48",
            NpcKind::Reaper => "NPC_Killed_51",
            NpcKind::ZombieMushroom | NpcKind::ZombieMushroomHat | NpcKind::FungoFish | NpcKind::AnomuraFungus => {
                "NPC_Killed_6"
            }
            NpcKind::MushiLadybug => "NPC_Killed_47",
            // W4-d42 - the roster's DeathSound rows (614's null row
            // rides the None fall-through).
            NpcKind::DD2LightningBug => "dd2_lightning_bug_death_0",
            NpcKind::BartenderUnconscious => "NPC_Killed_6",
            NpcKind::WalkingAntlion => "NPC_Killed_34",
            NpcKind::LarvaeAntlion => "NPC_Killed_34",
            // W4-d42 - the roster's DeathSound rows (614's null row
            // rides the None fall-through).
            NpcKind::ZombieMerman => "NPC_Killed_21",
            // W4-d42 - the roster's DeathSound rows (614's null row
            // rides the None fall-through).
            NpcKind::TorchZombie => "NPC_Killed_2",
            NpcKind::ArmedTorchZombie => "NPC_Killed_2",
            NpcKind::GoldGoldfishWalker => "NPC_Killed_1",
            // W4-d42 - the roster's DeathSound rows (614's null row
            // rides the None fall-through).
            // W4-d42 - the roster's DeathSound rows (614's null row
            // rides the None fall-through).
            NpcKind::Seagull => "NPC_Killed_1",
            NpcKind::Seagull2 => "NPC_Killed_1",
            // W4-d42 - the roster's DeathSound rows (614's null row
            // rides the None fall-through).
            NpcKind::Grebe => "NPC_Killed_1",
            NpcKind::Grebe2 => "NPC_Killed_1",
            // W4-d42 - the roster's DeathSound rows (614's null row
            // rides the None fall-through).
            NpcKind::Rat => "NPC_Killed_4",
            NpcKind::Owl => "NPC_Killed_1",
            NpcKind::WaterStrider => "NPC_Killed_1",
            NpcKind::GoldWaterStrider => "NPC_Killed_1",
            // W4-d42 - the roster's DeathSound rows (614's null row
            // rides the None fall-through).
            NpcKind::Turtle => "NPC_Killed_1",
            NpcKind::TurtleJungle => "NPC_Killed_1",
            NpcKind::BloodNautilus => "NPC_Killed_1",
            NpcKind::BloodSquid => "NPC_Killed_1",
            // W4-d42 - the roster's DeathSound rows (614's null row
            // rides the None fall-through).
            NpcKind::GoblinShark => "dd2_drakin_death_0",
            NpcKind::BloodEelHead => "NPC_Killed_1",
            NpcKind::BloodEelBody => "NPC_Killed_1",
            NpcKind::BloodEelTail => "NPC_Killed_1",
            NpcKind::Gnome => "NPC_Killed_1",
            // W4-d42 - the roster's DeathSound rows (614's null row
            // rides the None fall-through).
            NpcKind::SeaTurtle => "NPC_Killed_1",
            NpcKind::Seahorse => "NPC_Killed_1",
            NpcKind::GoldSeahorse => "NPC_Killed_1",
            NpcKind::Dandelion => "NPC_Killed_1",
            NpcKind::IceMimic => "NPC_Killed_6",
            // W4-d42 - the roster's DeathSound rows (614's null row
            // rides the None fall-through).
            NpcKind::BloodMummy => "NPC_Killed_6",
            // W4-d35 (G): the 122/137/140 DeathSound rows.
            NpcKind::Gastropod => "NPC_Killed_1",
            NpcKind::IlluminantBat => "NPC_Killed_6",
            NpcKind::PossessedArmor => "NPC_Killed_6",
            // W4-d35 (F): the 110/112/116/120/121 DeathSound rows.
            NpcKind::SkeletonArcher => "NPC_Killed_2",
            NpcKind::VileSpit => "NPC_Killed_9",
            NpcKind::TheHungryII => "NPC_Killed_12",
            NpcKind::ChaosElemental => "NPC_Killed_6",
            // W4-d35 (E): the 101/102/103/104/109 DeathSound rows.
            NpcKind::Clinger => "NPC_Killed_1",
            NpcKind::AnglerFish => "NPC_Killed_1",
            NpcKind::GreenJellyfish => "NPC_Killed_28",
            NpcKind::Werewolf => "NPC_Killed_1",
            NpcKind::Clown => "NPC_Killed_2",
            // W4-d35 (D): the 96/97/98/99/100 DeathSound rows.
            NpcKind::DiggerBody => "NPC_Killed_1",
            NpcKind::DiggerTail => "NPC_Killed_1",
            NpcKind::SeekerHead => "NPC_Killed_1",
            NpcKind::SeekerBody => "NPC_Killed_1",
            NpcKind::SeekerTail => "NPC_Killed_1",
            // W4-d35 (C): the 85/86/93/94/95 DeathSound rows.
            NpcKind::Mimic => "NPC_Killed_6",
            NpcKind::Unicorn => "NPC_Killed_18",
            NpcKind::GiantBat => "NPC_Killed_4",
            NpcKind::Corruptor => "NPC_Killed_1",
            NpcKind::DiggerHead => "NPC_Killed_1",
            // W4-d35 (B): the 79/80/82/83/84 DeathSound rows.
            NpcKind::DarkMummy => "NPC_Killed_6",
            NpcKind::LightMummy => "NPC_Killed_6",
            NpcKind::Wraith => "NPC_Killed_52",
            NpcKind::CursedHammer => "NPC_Killed_6",
            NpcKind::EnchantedSword => "NPC_Killed_6",
            // W4-d35 (A): the 73/74/75/77/78 DeathSound rows.
            NpcKind::GoblinScout => "NPC_Killed_1",
            NpcKind::Bird => "NPC_Killed_1",
            NpcKind::Pixie => "NPC_Killed_7",
            NpcKind::ArmoredSkeleton => "NPC_Killed_2",
            NpcKind::Mummy => "NPC_Killed_6",
            // W4-d25 STYLE-26 wiring: 86 NPCDeath18 (`NPC.cs:9935`),
            // 155 NPCDeath1 (`:10970`), 546 NPCDeath15 (`:15981`). The
            // Unicorn row mirrors d35 (C).
            NpcKind::Wolf => "NPC_Killed_1",
            NpcKind::Tumbleweed => "NPC_Killed_15",
            // W4-d125: 636 `SoundID.NPCDeath65` (`NPC.cs:17197`), 661
            // `NPCDeath1` (`:17359`).
            NpcKind::EmpressOfLight => "NPC_Killed_65",
            NpcKind::PrismaticLacewing => "NPC_Killed_1",
            // W4-g5 — the two casters' `DeathSound` rows: `SoundID.NPCDeath2`
            // (`NPC.cs:11163`/`:17730`).
            NpcKind::RuneWizard | NpcKind::LibrarianSkeleton => "NPC_Killed_2",
            // W4-g8 lane 1 batch 1 — the `DeathSound` rows: 61 NPCDeath31
            // (`NPC.cs:9564`), 63/64 NPCDeath28 (`:9592`/`:9606`), 69
            // NPCDeath34 (`:9675`), 70 NPCDeath1 (`:9690`), 75 NPCDeath7
            // (`:9783`), 82 NPCDeath52 (`:9863`), 83/84/85 NPCDeath6
            // (`:9888`/`:9901`/`:9914`).
            NpcKind::Vulture => "NPC_Killed_31",
            NpcKind::BlueJellyfish | NpcKind::PinkJellyfish => "NPC_Killed_28",
            NpcKind::Antlion => "NPC_Killed_34",
            NpcKind::SpikeBall => "NPC_Killed_1",
            NpcKind::CursedHammer | NpcKind::EnchantedSword | NpcKind::Mimic => "NPC_Killed_6",
            // W4-g8 lane 1 batch 2 — the `DeathSound` rows: 103 NPCDeath28
            // (`NPC.cs:10226`), 122/174/195 NPCDeath1 (`:10490`/`:11204`/
            // `:11501`), 164/165 NPCDeath32 (`:11068`/`:11083`), 169
            // NPCDeath7 (`:11124`), 177 NPCDeath25 (`:11248`), 179
            // NPCDeath6 (`:11275`), 182/196 NPCDeath2 (`:11327`/`:11515`).
            NpcKind::Gastropod | NpcKind::Herpling | NpcKind::LostGirl => "NPC_Killed_1",
            NpcKind::WallCreeper | NpcKind::WallCreeperWall => "NPC_Killed_32",
            NpcKind::IceElemental => "NPC_Killed_7",
            NpcKind::Derpling => "NPC_Killed_25",
            NpcKind::CrimsonAxe => "NPC_Killed_6",
            NpcKind::FloatyGross | NpcKind::Nymph => "NPC_Killed_2",
            // W4-g8 lane 1 batch 3 — the `DeathSound` rows: 34 NPCDeath2
            // (`NPC.cs:9159`), 62 NPCDeath24 (`:9578`), 93/150/151/152
            // NPCDeath4 (`:10058`/`:10896`/`:10911`/`:10927`), 121
            // NPCDeath1 (no row, the head default), 137/158 NPCDeath6
            // (`:10745`/`:15477`), 226 NPCDeath26 (`:11922`).
            NpcKind::CursedSkull => "NPC_Killed_2",
            NpcKind::Demon => "NPC_Killed_24",
            NpcKind::GiantBat | NpcKind::IceBat | NpcKind::Lavabat | NpcKind::GiantFlyingFox => "NPC_Killed_4",
            NpcKind::Slimer => "NPC_Killed_1",
            NpcKind::IlluminantBat | NpcKind::VampireBat => "NPC_Killed_6",
            // W4-g8 lane 1 batch 4 — the `DeathSound` rows: 221/48
            // NPCDeath1 (`NPC.cs:11850`/`:9370`), 156 NPCDeath24 (`:10985`).
            NpcKind::Squid | NpcKind::Harpy => "NPC_Killed_1",
            NpcKind::RedDevil => "NPC_Killed_24",
            // W4-g8 machine round 3 — the Vampire 159's NPCDeath6
            // (`NPC.cs:15490`).
            NpcKind::Vampire => "NPC_Killed_6",
            // The dragonflies (W4-d33): NPCDeath1 each (`NPC.cs:16730`).
            NpcKind::BlackDragonfly
            | NpcKind::BlueDragonfly
            | NpcKind::GreenDragonfly
            | NpcKind::OrangeDragonfly
            | NpcKind::RedDragonfly
            | NpcKind::YellowDragonfly
            | NpcKind::GoldDragonfly => "NPC_Killed_1",
            // The fairy critters (W4-d33 wiring): NPCDeath1 each
            // (`NPC.cs:16592`). d42 batch-2's rows, constants-copy
            // (keep-one d37c-E3).
            NpcKind::FairyCritterPink
            | NpcKind::FairyCritterGreen
            | NpcKind::FairyCritterBlue => "NPC_Killed_1",
            // The windy balloon (W4-d33 wiring): NPCDeath63
            // (`NPC.cs:16716`, `SoundID.cs:1377`). d42 batch-3's row,
            // constants-copy (keep-one d37c-E3).
            NpcKind::WindyBalloon => "NPC_Killed_63",
            // W4-f42 — the sandstorm debris + desert family's death cues:
            // 546 NPCDeath15 (`NPC.cs:15981`), 61 NPCDeath31 (`:9564`),
            // 69 NPCDeath34 (`:9675`), 78/79/80 NPCDeath6 (`:9808`/`:9821`/
            // `:9834`), 630 NPCDeath6 (`:17116`), 580 NPCDeath34 (`:14863`),
            // 581 NPCDeath35 (`:14878`).
            NpcKind::Antlion | NpcKind::WalkingAntlion => "NPC_Killed_34",
            NpcKind::FlyingAntlion => "NPC_Killed_35",
            NpcKind::Mummy
            | NpcKind::DarkMummy
            | NpcKind::BloodMummy
            | NpcKind::LightMummy => "NPC_Killed_6",
            // W4-d9 wiring (d40 rows): the SetDefaults DeathSound rows —
            // the dragon family NPCDeath60 (`NPC.cs:14426`/`:14444`/
            // `:14463`), the 521 squidhead NPCDeath6 (`NPC.cs:15777`).
            | NpcKind::CultistDragonBody2
            | NpcKind::CultistDragonBody3
            NpcKind::AncientCultistSquidhead => "NPC_Killed_6",
            // W4-d39: the Butterfly's `SoundID.NPCDeath1` row
            // (`NPC.cs:13540`); the scorpion pair `:13670`, the
            // grasshopper `:13820`.
            NpcKind::Butterfly
            | NpcKind::ScorpionBlack
            | NpcKind::Scorpion
            | NpcKind::Grasshopper => "NPC_Killed_1",
            // W4-d39 batch 3: the teeth bomb (`NPC.cs:13835`) + the
            // archer pair (`:13848`) — NPCDeath1.
            NpcKind::ChatteringTeethBomb
            | NpcKind::CultistArcherBlue
            | NpcKind::CultistArcherWhite => "NPC_Killed_1",
            // The luck critters (W4-e26, E8 keep-one): NPCDeath1 each
            // (`NPC.cs:13609` / `:16772`).
            NpcKind::Frog | NpcKind::LadyBug | NpcKind::GoldLadyBug => "NPC_Killed_1",
            // W4-f18 first-20 port — the critters' `SoundID.NPCDeath1` rows
            // (`NPC.cs:13526`/`:13540`/`:13554`/`:13568`/`:13582`).
            NpcKind::Firefly | NpcKind::Butterfly | NpcKind::Worm | NpcKind::LightningBug | NpcKind::Snail => "NPC_Killed_1",
            // W4-f18 batch 2 — the critters' `SoundID.NPCDeath1` rows
            // (`NPC.cs:13592`/`:13606`/`:13618`/`:13630`/`:13642`).
            NpcKind::GlowingSnail | NpcKind::Frog | NpcKind::Duck | NpcKind::Duck2 | NpcKind::DuckWhite => "NPC_Killed_1",
            // W4-f18 batch 3 — the critters' `SoundID.NPCDeath1` rows
            // (`NPC.cs:13654`/`:13666`/`:13767`/`:13782`).
            NpcKind::DuckWhite2 | NpcKind::ScorpionBlack | NpcKind::Scorpion | NpcKind::TruffleWorm | NpcKind::TruffleWormDigger => "NPC_Killed_1",
            // W4-f18 batch 4 — the `SoundID.NPCDeath1` rows (`NPC.cs:
            // 13816`/`:13830`/`:13844`); the ArmedZombie 430's NPCDeath2
            // (`:8617`).
            NpcKind::Grasshopper | NpcKind::ChatteringTeethBomb | NpcKind::CultistArcherBlue | NpcKind::CultistArcherWhite => "NPC_Killed_1",
            NpcKind::ArmedZombie => "NPC_Killed_2",
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
            // The aiStyle-14 flyer family (W4-d23): the same AI-head
            // `noGravity = true` (`NPC.cs:22851`) reaches all thirteen —
            // their SetDefaults rows write none, the axis arm does (the
            // four-bat row's precedent). The Vampire 159 is aiStyle 3
            // (gravity walker), NOT here.
            NpcKind::Harpy
            | NpcKind::Demon
            | NpcKind::GiantBat
            | NpcKind::Slimer
            | NpcKind::IlluminantBat
            | NpcKind::IceBat
            | NpcKind::Lavabat
            | NpcKind::GiantFlyingFox
            | NpcKind::RedDevil
            | NpcKind::VampireBat
            | NpcKind::FlyingSnake
            | NpcKind::SporeBat
            | NpcKind::QueenSlimeMinionPurple => true,
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
            // W4-c3: the WaterBoltMimic writes `noGravity = true` +
            // kb 0.2 (`NPC.cs:17747-17750`); the 34/289 rows are
            // d34/d38's below (E8 keep-both).
            | NpcKind::WaterBoltMimic
            // W4-c6 style-59 wiring: the blade's `noGravity = true`
            // (`NPC.cs:13168`; kb 0, `:13170`).
            | NpcKind::PumpkingBlade
            | NpcKind::Piranha
            | NpcKind::Shark => true,
            // W4-d41 — the antlion swarmer (509, `NPC.cs:14841-14853`
            // writes no noGravity row, but the aiStyle-44 head sets it
            // every tick, `:30863`; kb 0.3 `:14849`) and the Dune Splicer
            // chain (510-512, `:14784`/`:14801`/`:14820`; kb 0 — the
            // knockback gate refuses them, the row kept for the census).
            | NpcKind::DuneSplicerHead
            | NpcKind::DuneSplicerBody
            | NpcKind::DuneSplicerTail => true,
            // W4-d41 — the Tomb Crawler chain (513-515,
            // `NPC.cs:14729`/`:14746`/`:14765`; kb 0) and the solar
            // fragments (516/519, `:15404`/`:15419`; kb 0).
            NpcKind::TombCrawlerHead
            | NpcKind::TombCrawlerBody
            | NpcKind::TombCrawlerTail
            | NpcKind::SolarFlare
            | NpcKind::SolarGoop => true,
            // W4-d41 — the ancient orbs (522/523,
            // `NPC.cs:15790`/`:15805`; kb 0).
            NpcKind::AncientLight | NpcKind::AncientDoom => true,
            // W4-d41 — the sand poacher's wall form (531,
            // `NPC.cs:14948`; kb 0.5).
            NpcKind::DesertScorpionWall => true,
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
            // W4-c4 (the style-16 swimmers): every `SetDefaults` row writes
            // `noGravity = true` (`NPC.cs:9491`/`:10992`/`:12108`/`:14485`/
            // `:16681`/`:16797`/`:16894`/`:17655`/`:17711`); the kb rows
            // are 1 (no row, the head's `:8533` reset) for 57/157/241/465
            // and 0.5/0.7 for the critters/Orca.
            // E8 merge drops: CorruptGoldfish/BloodFeeder (d34/d37 rows
            // above), Pufferfish/Orca (d43 lands later).
            NpcKind::Arapaima
            | NpcKind::CrimsonGoldfish
            | NpcKind::GoldGoldfish
            | NpcKind::Pupfish
            | NpcKind::Dolphin => true,
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
            // W4-d38 b2 — the noGravity SetDefaults rows.
            NpcKind::GiantCursedSkull => true,
            // W4-d38 b5 — the noGravity SetDefaults rows.
            NpcKind::Ghost => true,
            // W4-d38 b6 — the noGravity SetDefaults rows.
            NpcKind::PumpkingBlade => true,
            // W4-d40: the SetDefaults noGravity rows (`NPC.cs`).
            NpcKind::CultistTablet
            | NpcKind::CultistBossClone => true,
            // W4-d40: the SetDefaults noGravity rows (`NPC.cs`).
            NpcKind::GoldButterfly => true,
            // W4-d40: the SetDefaults noGravity rows (`NPC.cs`).
            NpcKind::CultistDragonHead
            | NpcKind::CultistDragonBody1
            | NpcKind::CultistDragonBody2 => true,
            // W4-d40: the SetDefaults noGravity rows (`NPC.cs`).
            NpcKind::CultistDragonBody3
            | NpcKind::CultistDragonBody4
            | NpcKind::CultistDragonTail => true,
            // W4-d40: the SetDefaults noGravity rows (`NPC.cs`).
            NpcKind::DeadlySphere => true,
            // W4-d40: the SetDefaults noGravity rows (`NPC.cs`).
            NpcKind::Mothron => true,
            // W4-d42 - the noGravity rows.
            NpcKind::DD2LightningBug
            => true,
            // W4-d42 - the noGravity rows.
            NpcKind::BloodNautilus
            | NpcKind::BloodSquid
            => true,
            // W4-d42 - the noGravity rows.
            NpcKind::BloodEelHead
            | NpcKind::BloodEelBody
            | NpcKind::BloodEelTail
            => true,
            // W4-d35 (A): the 75 noGravity flyers (kb>0).
            NpcKind::Pixie => true,
            // W4-d35 (B): the 82 noGravity flyers (kb>0).
            NpcKind::Wraith => true,
            // W4-d35 (C): the 93/94 noGravity flyers (kb>0).
            NpcKind::GiantBat
            | NpcKind::Corruptor => true,
            // W4-d35 (E): the 101/102/103 noGravity flyers (kb>0).
            NpcKind::Clinger
            | NpcKind::AnglerFish
            | NpcKind::GreenJellyfish => true,
            // W4-d35 (F): the 121 noGravity flyers (kb>0).
            NpcKind::Slimer => true,
            // W4-d35 (G): the 122/137 noGravity flyers (kb>0).
            NpcKind::Gastropod
            | NpcKind::IlluminantBat => true,
            NpcKind::AngryNimbus => true,
            // W4-d34: the Cursed Skull's `noGravity = true` (`NPC.cs:9160`,
            // kb 0.2 `:9163`). The ManEater (43) writes noGravity but kb 0
            // (`:9304`) — the Y-pop selector never reaches it (leech precedent).
            NpcKind::CursedSkull => true,
            // W4-d34: the Harpy's aiStyle-14 head write (`NPC.cs:22851`, kb 0.6
            // `:9369`). The Bone Serpent writes noGravity but kb 0 (`:9239`) —
            // the Y-pop selector never reaches it (the leech precedent).
            NpcKind::Harpy => true,
            // W4-d34 (phase 3): the CorruptGoldfish's `noGravity = true`
            // (`NPC.cs:9491`, kb 1 — no row); the Vulture's aiStyle-17 head
            // write (`:24024`, kb 0.8 `:9562`); the Demon's aiStyle-14 head
            // write (`:22851`, kb 0.8 `:9577`); the BlueJellyfish's
            // `noGravity = true` (`:9584`, kb 1 — no row). The Snatcher (56)
            // writes noGravity but kb 0 (`:9485`) — the Y-pop selector never
            // reaches it (the ManEater precedent).
            NpcKind::CorruptGoldfish | NpcKind::Vulture | NpcKind::Demon | NpcKind::BlueJellyfish => true,
            // W4-d34 (phase 4, merge-side): the Dungeon Guardian (68)
            // writes noGravity but kb 0 (`:9664`) — the Y-pop selector never
            // reaches it (the ManEater precedent); the Crab is a grounded
            // walker. PinkJellyfish/Antlion/SpikeBall dropped (g8-owned).
            // W4-d37 — the NPCID 211-280 band's flyers whose `SetDefaults`
            // writes `noGravity = true` AND `knockBackResist > 0` (the Y-pop's
            // 0.5 selector reaches them). The squid/blood jelly/fungo fish
            // (aiStyle 18, `NPC.cs:11842`/`:12121`/`:12291`, kb 1f default),
            // the flying snake (aiStyle-14 head, `:22851`, kb 0.65), the
            // hornets + parrot (aiStyle 5, `:11991`/`:12250`, kb 0.3-0.7),
            // the wall crawlers (aiStyle 40, `:12065`/`:11046`/`:12093`,
            // kb 0.25/0.5), the blood feeder (aiStyle 16, `:12108`, kb 1f),
            // the reaper (aiStyle 22, `:15452`, kb 0.6) and the ichor sticker
            // (aiStyle 22, `:12472`, kb 0.6). The flying fish's noGravity is
            // the aiStyle-44 head's per-tick write (`:30863`, kb 0.8). The
            // fungi bulbs (259/260) are noGravity but kb 0 — the knockback
            // gate refuses them, so they keep the false row.
            NpcKind::Squid
            | NpcKind::BloodJelly
            | NpcKind::FungoFish
            | NpcKind::FlyingSnake
            | NpcKind::FlyingFish
            | NpcKind::HornetFatty
            | NpcKind::HornetHoney
            | NpcKind::HornetLeafy
            | NpcKind::HornetSpikey
            | NpcKind::HornetStingy
            | NpcKind::Parrot
            | NpcKind::JungleCreeperWall
            | NpcKind::BlackRecluseWall
            | NpcKind::BloodCrawlerWall
            | NpcKind::BloodFeeder
            | NpcKind::Reaper
            | NpcKind::IchorSticker => true,
            // W4-c4 (the style-44 swimmers) — the aiStyle-44 AI head sets
            // `noGravity = true` every tick (`NPC.cs:30863`); kb 0.8/0.3/
            // 0.5/0 (`:11894`/`:14849`/`:14876`/`:16632`).
            // E8 merge: FlyingFish's row is d37's group above.
            NpcKind::GiantFlyingAntlion
            | NpcKind::FlyingAntlion
            | NpcKind::EyeballFlyingFish => true,
            // W4-c7 — the firefly family's `noGravity = true` rows
            // (`NPC.cs:13528`/`:13570`/`:17276`/`:17590`); all four keep
            // the class-default kb 1, so the Y-pop selector reaches them.
            NpcKind::Firefly | NpcKind::LightningBug | NpcKind::Lavafly | NpcKind::Shimmerfly => true,
            // W4-c8 — the TruffleWormDigger's `noGravity = true` row
            // (`NPC.cs:13787`); the eight style-66 crawlers keep the head
            // default false (`:8535`) — they walk on gravity.
            NpcKind::TruffleWormDigger => true,
            // The snail family + its shimmer target (W4-c9): the
            // `SetDefaults` noGravity rows (`NPC.cs:13584` / `:13598` /
            // `:17291` / `:17590`). kb 1 (the head default — the rows
            // write none), so a strike can push them; the crawl's own
            // per-tick gravity gate rides `SnailEvents::no_gravity`.
            // E8 merge: Shimmerfly's row is c7's above.
            NpcKind::Snail | NpcKind::GlowingSnail | NpcKind::MagmaSnail => true,
            // W4-c10 roster tail — the GoldButterfly's `noGravity = true`
            // row (`NPC.cs:14286`); the GoldBird's flag is the aiStyle-24
            // head's per-tick write (`:25325`, the AngryNimbus precedent —
            // the perch arm clears it, the knockback read sees the flyer
            // default). Every other critter keeps the false default.
            // E8 merge: GoldButterfly's row is d40's above; the GoldBird's
            // noGravity is the aiStyle-24 head's per-tick write (`:25325`,
            // the AngryNimbus precedent - the superset addition).
            NpcKind::GoldBird => true,
            // The man-eater family (W4-d23): all six `SetDefaults` rows
            // write `noGravity = true` (`NPC.cs:9295`/`:9477`/`:10188`/
            // `:11210`/`:12334`/`:12350`).
            NpcKind::ManEater
            | NpcKind::Snatcher
            | NpcKind::Clinger
            | NpcKind::AngryTrapper
            | NpcKind::FungiBulb
            | NpcKind::GiantFungiBulb => true,
            // The aiStyle-9 spit/flare family (W4-d23): 112/666/516 all
            // write `noGravity = true` (`NPC.cs:10336`/`:17428`/`:15404`).
            NpcKind::VileSpit | NpcKind::VileSpitEoW | NpcKind::SolarFlare => true,
            // W4-d125: the Prismatic Lacewing (661, `NPC.cs:17361`) —
            // noGravity, no kb row → the class default 1f (`:8533`), so
            // the Y-pop selector reaches it. The Empress (636) is
            // noGravity too (`:17190`) but kb 0 (`:17198`) — the
            // knockback gate refuses resist 0, so no row is needed.
            NpcKind::PrismaticLacewing => true,
            // The dragonflies (W4-d33): the `SetDefaults` noGravity row
            // (`NPC.cs:16732`). kb 1 (the head default — the row writes
            // none), so a strike can push them; the aiStyle-114 arm owns
            // the flight.
            NpcKind::BlackDragonfly
            | NpcKind::BlueDragonfly
            | NpcKind::GreenDragonfly
            | NpcKind::OrangeDragonfly
            | NpcKind::RedDragonfly
            | NpcKind::YellowDragonfly
            | NpcKind::GoldDragonfly => true,
            // The fairy critters + the windy balloon (W4-d33 wiring): the
            // `SetDefaults` noGravity rows (`NPC.cs:16605` / `:16718`).
            // d42 batch-2/3's rows, constants-copy (keep-one d37c-E3).
            NpcKind::FairyCritterPink
            | NpcKind::FairyCritterGreen
            | NpcKind::FairyCritterBlue
            | NpcKind::WindyBalloon => true,
            // W4-f42 — the desert flyers' AI heads set noGravity every tick:
            // the Vulture (61, aiStyle 17 `NPC.cs:24024`), the FlyingAntlion
            // (581, aiStyle 44 `:30863`). The Antlion (69) toggles it per
            // tick (the emergence), so its SetDefaults default stays false.
            NpcKind::Vulture | NpcKind::FlyingAntlion => true,
            // W4-d9 wiring (d40 rows): the dragon family's `noGravity =
            // true` SetDefaults rows (`NPC.cs:14427`/`:14445`/`:14465`);
            // kb 0 keeps them out of the Y-pop selector, the rows ride
            // for the other `no_gravity` readers. The 521 squidhead's
            // SetDefaults writes no flag, but its aiStyle-86 AI head sets
            // `noGravity = true` every tick (`NPC.cs:37231`, the
            // AngryNimbus precedent).
            | NpcKind::CultistDragonBody2
            | NpcKind::CultistDragonBody3
            | NpcKind::CultistDragonTail
            | NpcKind::AncientCultistSquidhead => true,
            // W4-d39: the Butterfly's `noGravity` row (`NPC.cs:13542`).
            NpcKind::Butterfly => true,
            // The luck critters (W4-e26): the `SetDefaults` noGravity
            // rows (`NPC.cs:16774`). kb 1 (the head default — the rows
            // write none), so a strike can push them; the aiStyle-115
            // arm owns the flight. The Frog 361 writes NO noGravity row
            // (the aiStyle-7 walker falls) — it stays false here. The
            // Duck dropped to c10 (E8 keep-one).
            NpcKind::LadyBug | NpcKind::GoldLadyBug => true,
            // W4-f18 first-20 port — the critters' `noGravity = true` rows
            // with kb 1 (the class default, no kb row): 355 (`NPC.cs:13528`),
            // 356 (`:13542`), 358 (`:13570`), 359 (`:13584`). The Worm 357
            // writes no flag (grounded); the Snail's AI-head noGravity writes
            // are live state (the static table keeps the SetDefaults row).
            NpcKind::Firefly | NpcKind::Butterfly | NpcKind::LightningBug | NpcKind::Snail => true,
            // W4-f18 batch 2 — the GlowingSnail 360's `noGravity = true`
            // (`NPC.cs:13594`, kb 1). The Frog/Ducks write no flag
            // (grounded); the Duck2 363's noGravity is the AI-head's live
            // write (`npc_ai_style68`, the static table keeps SetDefaults).
            NpcKind::GlowingSnail => true,
            // W4-f18 batch 3 — the DuckWhite2/scorpions/TruffleWorm write
            // no noGravity flag. The TruffleWormDigger 375's SetDefaults
            // noGravity is true (`NPC.cs:13783`) BUT its kb is 0 (`:13782`),
            // so the strike-time Y-pop never reaches it — this table keeps
            // the W3-i11 convention (noGravity AND kb>0 only), so 375 reads
            // false here BY DESIGN (the census doc's note). Its burrow
            // movement is noTileCollide raw integration regardless.
            // The Shimmerfly (W4-c9's rowing, adopted by W4-g62): the
            // `SetDefaults` noGravity row (`NPC.cs:17590`). kb 1 (the
            // head default — the row writes none), so a strike can push
            // it; the snail trio's rows stay c9's (the merge dedupes).
            NpcKind::Shimmerfly => true,
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
            // E8 merge: the c10 GoldGrasshopper aiStyle-1 dup dropped (d40's).
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
            // aiStyle 3 — the Vampire (159), the VampireBat's Transform
            // target (`NPC.cs:15485`, W4-d23): the fighter walker.
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
            // aiStyle 3 — W4-f42: the desert walkers (`NPC.cs:9799`/
            // `:9812`/`:17107`/`:9825` the mummies, `:14854` the Walking
            // Antlion).
            NpcKind::Mummy
            | NpcKind::DarkMummy
            | NpcKind::BloodMummy
            | NpcKind::LightMummy
            | NpcKind::WalkingAntlion => 3,
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
            // aiStyle 7 — W4-c10, the waterfowl WALKERS: `NPC.cs:13617`
            // (362), `:13641` (364), `:16743` (602), `:16813` (608). The
            // swimming "2" forms ride aiStyle 68 below.
            // E8 merge: Seagull/Grebe's 7 rows are d42's above.
            NpcKind::Duck | NpcKind::DuckWhite => 7,
            // E8 merge: the c10 12-critter aiStyle tail dropped - d40/c8's
            // rows above own them (the dup drops; the wiring composes).
            // W4-e21 — the bunny pair (46/540) shares the SAME aiStyle-7
            // row (`NPC.cs:9340`): the town-critter hop.
            NpcKind::Bunny | NpcKind::PartyBunny => 7,
            // W4-g10 lane 3 — the town pets' `aiStyle = 7` rows: the
            // 637/638 arm (`NPC.cs:17217`), 656 (`:17301`), 670
            // (`:17487`), the 678-684 arm (`:17600`).
            NpcKind::TownCat
            | NpcKind::TownDog
            | NpcKind::TownBunny
            | NpcKind::TownSlimeBlue
            | NpcKind::TownSlimeGreen
            | NpcKind::TownSlimeOld
            | NpcKind::TownSlimePurple
            | NpcKind::TownSlimeRainbow
            | NpcKind::TownSlimeRed
            | NpcKind::TownSlimeYellow
            | NpcKind::TownSlimeCopper => 7,
            // aiStyle 8 - NPC.cs:9083, :9125, :9326, :14978, :15690
            NpcKind::GoblinSorcerer | NpcKind::Tim | NpcKind::DarkCaster | NpcKind::FireImp | NpcKind::DesertDjinn => 8,
            // aiStyle 8 - NPC.cs:9083, :9125, :9326, :11158, :14978, :15690, :17727
            NpcKind::GoblinSorcerer | NpcKind::Tim | NpcKind::DarkCaster | NpcKind::FireImp | NpcKind::DesertDjinn
            | NpcKind::RuneWizard | NpcKind::LibrarianSkeleton => 8,
            // aiStyle 9 - NPC.cs:9097, :9139, :15703
            NpcKind::ChaosBall | NpcKind::BurningSphere | NpcKind::WaterSphere => 9,
            // aiStyle 9 — the spit/flare family (W4-d23): 112 (`NPC.cs:10330`),
            // 516 (`:15398`), 666 (`:17422`).
            NpcKind::VileSpit | NpcKind::SolarFlare | NpcKind::VileSpitEoW => 9,
            // aiStyle 11 - NPC.cs:9170
            NpcKind::SkeletronHead => 11,
            // aiStyle 12 - NPC.cs:9187
            NpcKind::SkeletronHand => 12,
            // aiStyle 13 - NPC.cs:9299 (43), :9480 (56), :10193 (101),
            // :11214 (175), :12338 (259), :12354 (260): the man-eater
            // family (W4-d23).
            | NpcKind::AngryTrapper
            // aiStyle 14 - NPC.cs:9378, :9409, :9543, :9629
            NpcKind::CaveBat | NpcKind::JungleBat | NpcKind::Hellbat | NpcKind::VoodooDemon => 14,
            // aiStyle 14 — the W4-d23 flyer family (NPC.cs:9364 (48),
            // :9572 (62), :10052 (93), :10470 (121), :10739 (137),
            // :10890 (150), :10905 (151), :10921 (152), :10979 (156),
            // :15471 (158), :11916 (226), :17166 (634), :17342 (660)).
            // The Vampire (159) is aiStyle 3 — the transform target, NOT
            // a style-14 member (its row rides the aiStyle-3 group below).
            NpcKind::Harpy
            | NpcKind::Demon
            | NpcKind::GiantBat
            | NpcKind::Slimer
            | NpcKind::IlluminantBat
            | NpcKind::IceBat
            | NpcKind::Lavabat
            | NpcKind::GiantFlyingFox
            | NpcKind::RedDevil
            | NpcKind::VampireBat
            | NpcKind::FlyingSnake
            | NpcKind::SporeBat
            | NpcKind::QueenSlimeMinionPurple => 14,
            // aiStyle 15 - NPC.cs:9392
            NpcKind::KingSlime => 15,
            // aiStyle 16 - NPC.cs:9465, :9508, :9615
            NpcKind::Goldfish | NpcKind::Piranha | NpcKind::Shark => 16,
            // aiStyle 16 — W4-c4 (the style-16 swimmers): NPC.cs:9494
            // (the CorruptGoldfish 57), :10995 (the Arapaima 157), :12111
            // (the BloodFeeder 241), :14488 (the CrimsonGoldfish 465),
            // :16684 (the GoldGoldfish 592), :16800 (the Pupfish 607),
            // :16897 (the Dolphin 615), :17658 (the Pufferfish 688),
            // :17714 (the Orca 692). The AnglerFish 102 (`:10208`) is the
            // tenth unported member — its kind is the W4-d35 roster lane.
            // E8 merge drops: CorruptGoldfish/BloodFeeder (d34/d37 rows
            // above), Pufferfish/Orca (d43 lands later).
            NpcKind::Arapaima
            | NpcKind::CrimsonGoldfish
            | NpcKind::GoldGoldfish
            | NpcKind::Pupfish
            | NpcKind::Dolphin => 16,
            // aiStyle 17 — W4-f42: the Vulture (`NPC.cs:9558`).
            // aiStyle 19 — W4-f42: the Antlion (`NPC.cs:9670`).
            // aiStyle 44 — W4-f42: the Flying Antlion (`NPC.cs:14872`).
            // aiStyle 22 - NPC.cs:13194
            NpcKind::Poltergeist => 22,
            // aiStyle 22 — the Drippler (S1-D-3z; `NPC.cs:14623`).
            NpcKind::Drippler => 22,
            // aiStyle 25 - NPC.cs:13318 (the Present Mimic's chest machine,
            // `AI_025`, `:25538-25630`)
            NpcKind::PresentMimic => 25,
            // aiStyle 26 - NPC.cs:12982, :13179, :15078, :15121
            NpcKind::NebulaBeast | NpcKind::StardustSpiderSmall | NpcKind::HeadlessHorseman | NpcKind::Hellhound => 26,
            // aiStyle 26 — W4-f42: the Tumbleweed (`NPC.cs:15976`).
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
            // aiStyle 59 - NPC.cs:13162 (W4-c6 style-59 wiring: the
            // Pumpking's blade, the style's only member — the census).
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
            // W4-d40: the SetDefaults aiStyle rows (`NPC.cs`).
            NpcKind::ArmedZombie
            | NpcKind::ArmedZombieEskimo
            | NpcKind::ArmedZombiePincussion
            | NpcKind::ArmedZombieSlimed
            | NpcKind::ArmedZombieSwamp => 3,
            // W4-d40: the SetDefaults aiStyle rows (`NPC.cs`).
            NpcKind::ArmedZombieTwiggy
            | NpcKind::ArmedZombieCenx => 3,
            NpcKind::CultistTablet
            | NpcKind::CultistDevote => 83,
            NpcKind::CultistBossClone => 84,
            // W4-d40: the SetDefaults aiStyle rows (`NPC.cs`).
            NpcKind::GoldBird => 24,
            NpcKind::GoldBunny
            | NpcKind::GoldFrog => 7,
            NpcKind::GoldButterfly => 65,
            NpcKind::GoldGrasshopper => 1,
            // W4-d40: the SetDefaults aiStyle rows (`NPC.cs`).
            NpcKind::GoldMouse => 7,
            NpcKind::GoldWorm => 66,
            NpcKind::BoneThrowingSkeleton
            | NpcKind::BoneThrowingSkeleton2
            | NpcKind::BoneThrowingSkeleton3 => 3,
            // W4-d40: the SetDefaults aiStyle rows (`NPC.cs`).
            NpcKind::BoneThrowingSkeleton4 => 3,
            NpcKind::SkeletonMerchant => 7,
            NpcKind::CultistDragonHead
            | NpcKind::CultistDragonBody1
            | NpcKind::CultistDragonBody2 => 6,
            // W4-d40: the SetDefaults aiStyle rows (`NPC.cs`).
            NpcKind::CultistDragonBody3
            | NpcKind::CultistDragonBody4
            | NpcKind::CultistDragonTail => 6,
            NpcKind::Butcher
            | NpcKind::CreatureFromTheDeep => 3,
            // W4-d40: the SetDefaults aiStyle rows (`NPC.cs`).
            NpcKind::Fritz
            | NpcKind::Nailhead
            | NpcKind::CrimsonBunny
            | NpcKind::Psycho => 3,
            // W4-d40: the SetDefaults aiStyle rows (`NPC.cs`).
            NpcKind::DeadlySphere => 85,
            NpcKind::DrManFly
            | NpcKind::ThePossessed
            | NpcKind::CrimsonPenguin => 3,
            NpcKind::ShadowFlameApparition => 86,
            // W4-d40: the SetDefaults aiStyle rows (`NPC.cs`).
            NpcKind::BigMimicCorruption
            | NpcKind::BigMimicCrimson
            | NpcKind::BigMimicHallow
            | NpcKind::BigMimicJungle => 87,
            NpcKind::Mothron => 88,
            // W4-d40: the SetDefaults aiStyle rows (`NPC.cs`).
            NpcKind::MothronEgg => 89,
            NpcKind::MothronSpawn => 90,
            NpcKind::Medusa
            | NpcKind::GreekSkeleton
            | NpcKind::GraniteGolem => 3,
            // W4-d40: the SetDefaults aiStyle rows (`NPC.cs`).
            NpcKind::GraniteFlyer => 91,
            NpcKind::EnchantedNightcrawler
            | NpcKind::Grubby
            | NpcKind::Sluggy
            | NpcKind::Buggy => 66,
            // W4-d40: the SetDefaults aiStyle rows (`NPC.cs`).
            NpcKind::TargetDummy => 92,
            // aiStyle 3 - W4-d35 (A; NPC.cs :9732, :9786, :9799).
            NpcKind::GoblinScout
            | NpcKind::ArmoredSkeleton
            | NpcKind::Mummy => 3,
            // aiStyle 22 - W4-d35 (A; NPC.cs :9772).
            // aiStyle 24 - W4-d35 (A; NPC.cs :9747).
            NpcKind::Bird => 24,
            // aiStyle 3 - W4-d35 (B; NPC.cs :9812, :9825).
            NpcKind::DarkMummy
            | NpcKind::LightMummy => 3,
            // aiStyle 22 - W4-d35 (B; NPC.cs :9852).
            // aiStyle 23 - W4-d35 (B; NPC.cs :9879, :9892).
            NpcKind::CursedHammer
            | NpcKind::EnchantedSword => 23,
            // aiStyle 5 - W4-d35 (C; NPC.cs :10061).
            NpcKind::Corruptor => 5,
            // aiStyle 6 - W4-d35 (C; NPC.cs :10076).
            NpcKind::DiggerHead => 6,
            // aiStyle 14 - W4-d35 (C; NPC.cs :10047).
            // aiStyle 25 - W4-d35 (C; NPC.cs :9905).
            // aiStyle 26 - W4-d35 (C; NPC.cs :9926).
            NpcKind::Unicorn => 26,
            // aiStyle 6 - W4-d35 (D; NPC.cs :10094, :10113, :10132, :10150, :10168).
            NpcKind::DiggerBody
            | NpcKind::DiggerTail
            | NpcKind::SeekerHead
            | NpcKind::SeekerBody
            | NpcKind::SeekerTail => 6,
            // aiStyle 3 - W4-d35 (E; NPC.cs :10230, :10299).
            NpcKind::Werewolf
            | NpcKind::Clown => 3,
            // aiStyle 13 - W4-d35 (E; NPC.cs :10186).
            NpcKind::Clinger => 13,
            // aiStyle 16 - W4-d35 (E; NPC.cs :10202).
            NpcKind::AnglerFish => 16,
            // aiStyle 18 - W4-d35 (E; NPC.cs :10216).
            // aiStyle 2 - W4-d35 (F; NPC.cs :10393).
            NpcKind::TheHungryII => 2,
            // aiStyle 3 - W4-d35 (F; NPC.cs :10313, :10453).
            NpcKind::SkeletonArcher
            | NpcKind::ChaosElemental => 3,
            // aiStyle 9 - W4-d35 (F; NPC.cs :10326).
            NpcKind::VileSpit => 9,
            // aiStyle 14 - W4-d35 (F; NPC.cs :10466).
            // aiStyle 3 - W4-d35 (G; NPC.cs :10763).
            NpcKind::PossessedArmor => 3,
            // aiStyle 14 - W4-d35 (G; NPC.cs :10735).
            // aiStyle 22 - W4-d35 (G; NPC.cs :10479).
            NpcKind::DungeonSpirit => 56,
            // W4-d34 kinds 1-70 (`NPC.cs` SetDefaults aiStyle rows).
            NpcKind::AngryBones => 3,   // :9112
            NpcKind::OldMan => 7,       // :9203
            NpcKind::ManEater => 13,    // :9299
            NpcKind::Bunny => 7,        // :9340
            NpcKind::BoneSerpentHead | NpcKind::BoneSerpentBody | NpcKind::BoneSerpentTail => 6, // :9230/:9247/:9265
            NpcKind::CorruptBunny => 3,  // :9352
            NpcKind::Snatcher => 13,         // :9480
            NpcKind::CorruptGoldfish => 16,  // :9494
            NpcKind::Crab => 3,              // :9644
            NpcKind::DungeonGuardian => 11,  // :9656
            // W4-d38 b7 — SetDefaults aiStyle rows.
            NpcKind::SlimeRibbonWhite => 1,
            NpcKind::SlimeRibbonYellow => 1,
            NpcKind::SlimeRibbonGreen => 1,
            NpcKind::SlimeRibbonRed => 1,
            NpcKind::BunnyXmas => 7,
            // W4-d38 b6 — SetDefaults aiStyle rows.
            NpcKind::DemonEyeSpaceship => 2,
            NpcKind::SkeletonTopHat => 3,
            NpcKind::SkeletonAstonaut => 3,
            NpcKind::SkeletonAlien => 3,
            NpcKind::PumpkingBlade => 59,
            // W4-d38 b5 — SetDefaults aiStyle rows.
            NpcKind::SlimeMasked => 1,
            NpcKind::BunnySlimed => 7,
            NpcKind::HoppinJack => 1,
            NpcKind::Ghost => 22,
            NpcKind::DemonEyeOwl => 2,
            // W4-d38 b4 — SetDefaults aiStyle rows.
            NpcKind::BirdBlue => 24,
            NpcKind::BirdRed => 24,
            NpcKind::Squirrel => 7,
            NpcKind::Mouse => 7,
            NpcKind::Raven => 17,
            // W4-d38 b3 — SetDefaults aiStyle rows.
            NpcKind::TacticalSkeleton => 3,
            NpcKind::SkeletonCommando => 3,
            NpcKind::AngryBonesBig => 3,
            NpcKind::AngryBonesBigMuscle => 3,
            NpcKind::AngryBonesBigHelmet => 3,
            // W4-d38 b2 — SetDefaults aiStyle rows.
            NpcKind::DiabolistWhite => 8,
            NpcKind::BoneLee => 3,
            NpcKind::GiantCursedSkull => 10,
            NpcKind::Paladin => 3,
            NpcKind::SkeletonSniper => 3,
            // W4-d38 b1 — SetDefaults aiStyle rows.
            NpcKind::RaggedCaster => 8,
            NpcKind::RaggedCasterOpenCoat => 8,
            NpcKind::Necromancer => 8,
            NpcKind::NecromancerArmored => 8,
            NpcKind::DiabolistRed => 8,
            // W4-d37 — the NPCID 211-280 band's `SetDefaults` aiStyle rows
            // (the `this.aiStyle = N;` cites in `npc.rs`'s enum).
            // aiStyle 1 — RainbowSlime (`NPC.cs:12152`).
            NpcKind::RainbowSlime => 1,
            // aiStyle 3 — the beetles/snail/crawlers/fungi/walkers/armored
            // bones (`NPC.cs:11792`/`:11805`/`:11818`/`:11831`/`:12053`/
            // `:12082`/`:12137`/`:12238`/`:12267`/`:12280`/`:12308`/
            // `:12322`/`:12488`-`:12631`).
            NpcKind::CochinealBeetle
            | NpcKind::CyanBeetle
            | NpcKind::LacBeetle
            | NpcKind::SeaSnail
            | NpcKind::JungleCreeper
            | NpcKind::BloodCrawler
            | NpcKind::IceGolem
            | NpcKind::Eyezor
            | NpcKind::ZombieMushroom
            | NpcKind::ZombieMushroomHat
            | NpcKind::AnomuraFungus
            | NpcKind::MushiLadybug
            | NpcKind::RustyArmoredBonesAxe
            | NpcKind::RustyArmoredBonesFlail
            | NpcKind::RustyArmoredBonesSword
            | NpcKind::RustyArmoredBonesSwordNoArmor
            | NpcKind::BlueArmoredBones
            | NpcKind::BlueArmoredBonesMace
            | NpcKind::BlueArmoredBonesNoPants
            | NpcKind::BlueArmoredBonesSword
            | NpcKind::HellArmoredBones
            | NpcKind::HellArmoredBonesSpikeShield
            | NpcKind::HellArmoredBonesMace
            | NpcKind::HellArmoredBonesSword => 3,
            // aiStyle 5 — the hornets + the parrot (`NPC.cs:11983`/`:11997`/
            // `:12011`/`:12025`/`:12039`/`:12254`).
            NpcKind::HornetFatty
            | NpcKind::HornetHoney
            | NpcKind::HornetLeafy
            | NpcKind::HornetSpikey
            | NpcKind::HornetStingy
            | NpcKind::Parrot => 5,
            // aiStyle 7 — the pirate town NPC + the goldfish critter
            // (`NPC.cs:11959`/`:11971`).
            NpcKind::Pirate | NpcKind::GoldfishWalker => 7,
            // aiStyle 13 — the fungi bulbs (`NPC.cs:12338`/`:12354`).
            NpcKind::FungiBulb | NpcKind::GiantFungiBulb => 13,
            // aiStyle 14 — the flying snake (`NPC.cs:11916`).
            // aiStyle 16 — the blood feeder (`NPC.cs:12111`).
            NpcKind::BloodFeeder => 16,
            // aiStyle 18 — the squid/blood jelly/fungo fish (`NPC.cs:11845`/
            // `:12124`/`:12294`).
            NpcKind::Squid | NpcKind::BloodJelly | NpcKind::FungoFish => 18,
            // aiStyle 22 — the reaper + the ichor sticker (`NPC.cs:15456`/
            // `:12475`).
            NpcKind::Reaper | NpcKind::IchorSticker => 22,
            // aiStyle 40 — the wall crawlers (`NPC.cs:12068`/`:11049`/
            // `:12096`).
            NpcKind::JungleCreeperWall | NpcKind::BlackRecluseWall | NpcKind::BloodCrawlerWall => 40,
            // aiStyle 44 — the flying fish (`NPC.cs:11889`).
            NpcKind::FlyingFish => 44,
            // W4-d42 - the NPCID 578-582 SetDefaults rows.
            NpcKind::DD2LightningBug => 111,
            NpcKind::BartenderUnconscious => 0,
            NpcKind::WalkingAntlion => 3,
            NpcKind::LarvaeAntlion => 3,
            // W4-d42 - the NPCID 583-587 SetDefaults rows.
            NpcKind::ZombieMerman => 3,
            // W4-d42 - the NPCID 590-594 SetDefaults rows.
            NpcKind::TorchZombie => 3,
            NpcKind::ArmedTorchZombie => 3,
            NpcKind::GoldGoldfishWalker => 7,
            // W4-d42 - the NPCID 595-599 SetDefaults rows.
            // W4-d42 - the NPCID 600-604 SetDefaults rows.
            NpcKind::Seagull => 7,
            NpcKind::Seagull2 => 68,
            // W4-d42 - the NPCID 605-609 SetDefaults rows.
            NpcKind::Grebe => 7,
            NpcKind::Grebe2 => 68,
            // W4-d42 - the NPCID 610-614 SetDefaults rows.
            NpcKind::Rat => 7,
            NpcKind::Owl => 24,
            NpcKind::WaterStrider => 116,
            NpcKind::GoldWaterStrider => 116,
            NpcKind::ExplosiveBunny => 7,
            // W4-d42 - the NPCID 615-619 SetDefaults rows.
            NpcKind::Turtle => 7,
            NpcKind::TurtleJungle => 7,
            NpcKind::BloodNautilus => 117,
            NpcKind::BloodSquid => 5,
            // W4-d42 - the NPCID 620-624 SetDefaults rows.
            NpcKind::GoblinShark => 107,
            NpcKind::BloodEelHead => 6,
            NpcKind::BloodEelBody => 6,
            NpcKind::BloodEelTail => 6,
            NpcKind::Gnome => 3,
            // W4-d42 - the NPCID 625-629 SetDefaults rows.
            NpcKind::SeaTurtle => 7,
            NpcKind::Seahorse => 118,
            NpcKind::GoldSeahorse => 118,
            NpcKind::Dandelion => 119,
            NpcKind::IceMimic => 25,
            // W4-d42 - the NPCID 630-630 SetDefaults rows.
            NpcKind::BloodMummy => 3,
            // W4-c3 (the style-10 family): aiStyle 10 - NPC.cs:17741
            // (the WaterBoltMimic 694); the 34/289 aiStyle-10 rows are
            // d34/d38's above (E8 keep-both, same axis machine).
            NpcKind::WaterBoltMimic => 10,
            // W4-c4 (the style-44 swimmers): aiStyle 44 - NPC.cs:11889
            // (the FlyingFish 224), :14845 (the GiantFlyingAntlion 509),
            // :14872 (the FlyingAntlion 581), :16627 (the EyeballFlyingFish
            // 587) - the census is exactly these four.
            // E8 merge: FlyingFish's 44 row is d37's above.
            NpcKind::GiantFlyingAntlion
            | NpcKind::FlyingAntlion
            | NpcKind::EyeballFlyingFish => 44,
            // aiStyle 64 — W4-c7, the firefly family: `NPC.cs:13521` (355),
            // `:13563` (358), `:17269` (654), `:17583` (677) — the four
            // `this.aiStyle = 64;` rows of the SetDefaults census.
            NpcKind::Firefly | NpcKind::LightningBug | NpcKind::Lavafly | NpcKind::Shimmerfly => 64,
            // aiStyle 66 — W4-c8, the crawling critters: `NPC.cs:13549`
            // (357), `:13766` (374), `:14336` (448), `:14580` (the
            // 484-487 range row), `:16785` (606) — the eight
            // `this.aiStyle = 66;` rows of the SetDefaults census.
            // E8 merge: the 5 d40-owned dups dropped - d40's aiStyle-66
            // rows above own them (the superset wiring composes).
            NpcKind::Worm
            | NpcKind::TruffleWorm
            | NpcKind::Maggot => 66,
            // aiStyle 6 — W4-c8, the TruffleWorm's dig target
            // (`NPC.cs:13781`): the worm pass's lone-minion member (the
            // StardustWormHead precedent — `is_worm` true, no chain).
            NpcKind::TruffleWormDigger => 6,
            // The snail wall-crawler family (W4-c9) — `NPC.cs:13577` /
            // `:13591` / `:17284` (the three `this.aiStyle = 67;` rows).
            NpcKind::Snail | NpcKind::GlowingSnail | NpcKind::MagmaSnail => 67,
            // E8 merge: the Shimmerfly 64 row is c7's above (the dup drop).
            // The gold critters (W4-c9) — `NPC.cs:14251` (the GoldBird's
            // aiStyle 24), `:14307` (the GoldGrasshopper's aiStyle 1),
            // `:14322` (the GoldMouse's aiStyle 7).
            // E8 merge: the 5 gold aiStyle rows are d40's above (24/1/7/
            // 7/7 - the dup drops; c9's wiring composes onto them).
            // aiStyle 68 — W4-c10, the waterfowl SWIMMERS: `NPC.cs:13629`
            // (363), `:13653` (365), `:16755` (603), `:16825` (609) — the
            // four `this.aiStyle = 68;` rows of the SetDefaults census.
            // E8 merge: Seagull2/Grebe2's 68 rows are d42's above.
            NpcKind::Duck2 | NpcKind::DuckWhite2 => 68,
            // W4-d25 STYLE-26 wiring: aiStyle 26 — `NPC.cs:9930` (86) /
            // `:10965` (155) / `:15976` (546). The Unicorn row mirrors
            // d35 (C).
            NpcKind::Unicorn | NpcKind::Wolf | NpcKind::Tumbleweed => 26,
            // W4-d120 — the four town kinds' aiStyle 7 (`NPC.cs:10797`/
            // `:11009`/`:11258`/`:11688`); the bound Tavernkeep's aiStyle 0
            // (`:16560`).
            NpcKind::SantaClaus | NpcKind::Truffle | NpcKind::Steampunker | NpcKind::Cyborg => 7,
            // W4-d125: 636 aiStyle 120 (`NPC.cs:17193`,
            // `AI_120_HallowBoss`), 661 aiStyle 65 (`:17354`,
            // `AI_065_Butterflies`' 661 branch).
            NpcKind::EmpressOfLight => 120,
            NpcKind::PrismaticLacewing => 65,
            // W4-g8 lane 1 batch 1 — the d22-d33 machines' `SetDefaults`
            // aiStyle rows: 61 17 (`NPC.cs:9558`), 63/64 18 (`:9587`/
            // `:9601`), 69 19 (`:9670`), 70 20 (`:9685`), 75/82 22
            // (`:9777`/`:9858`), 83/84 23 (`:9883`/`:9896`), 85 25
            // (`:9909`).
            NpcKind::Antlion => 19,
            NpcKind::SpikeBall => 20,
            // W4-g8 lane 1 batch 2 — the `SetDefaults` aiStyle rows: 103 18
            // (`NPC.cs:10221`), 122/169/182 22 (`:10484`/`:11118`/`:11322`),
            // 164 3 (`:11063`), 165 40 (`:11078`), 174/177 41 (`:11199`/
            // `:11243`), 179 23 (`:11270`), 195 42 (`:11496`), 196 3
            // (`:11510`).
            NpcKind::WallCreeper => 3,
            NpcKind::WallCreeperWall => 40,
            NpcKind::Herpling | NpcKind::Derpling => 41,
            NpcKind::CrimsonAxe => 23,
            NpcKind::LostGirl => 42,
            NpcKind::Nymph => 3,
            // W4-g8 lane 1 batch 3 — the `SetDefaults` aiStyle rows: 34 10
            // (`NPC.cs:9154`), 62/93/121/137/150/151/152/158/226 14
            // (`:9572`/`:10052`/`:10470`/`:10739`/`:10890`/`:10905`/
            // `:10921`/`:15471`/`:11916`).
            NpcKind::Demon | NpcKind::GiantBat | NpcKind::Slimer | NpcKind::IlluminantBat
            | NpcKind::IceBat | NpcKind::Lavabat | NpcKind::GiantFlyingFox
            // W4-g8 lane 1 batch 4 — the `SetDefaults` aiStyle rows: 221 18
            // (`NPC.cs:11845`), 48/156 14 (`:9364`/`:10979`).
            // W4-g8 machine round 3 — the Vampire 159's aiStyle 3
            // (`NPC.cs:15485`, the AI_003 fighter).
            NpcKind::Vampire => 3,
            // The dragonflies (W4-d33) — aiStyle 114 (`NPC.cs:16725`, the
            // shared 595-601 row): `AI_114_Dragonflies`
            // (`NPC.cs:47716-47886`).
            NpcKind::BlackDragonfly
            | NpcKind::BlueDragonfly
            | NpcKind::GreenDragonfly
            | NpcKind::OrangeDragonfly
            | NpcKind::RedDragonfly
            | NpcKind::YellowDragonfly
            | NpcKind::GoldDragonfly => 114,
            // The fairy critters (W4-d33 wiring) — aiStyle 112
            // (`NPC.cs:16587`, the shared 583-585 row): `AI_112_FairyCritter`
            // (`NPC.cs:48088-48650`). The windy balloon — aiStyle 113
            // (`NPC.cs:16711`): `AI_113_WindyBalloon` (`:47888-48065`).
            // d42 batch-2/3's rows, constants-copy (keep-one d37c-E3).
            NpcKind::FairyCritterPink
            | NpcKind::FairyCritterGreen
            | NpcKind::FairyCritterBlue => 112,
            NpcKind::WindyBalloon => 113,
            // W4-d9 wiring (d40 rows): the SetDefaults aiStyle rows —
            // the dragon family 6 (`NPC.cs:14420`/`:14438`/`:14457`),
            // the 521 squidhead 86 (`NPC.cs:15772`).
            | NpcKind::CultistDragonBody2
            | NpcKind::CultistDragonBody3
            NpcKind::AncientCultistSquidhead => 86,
            // W4-d39: the Butterfly's `SetDefaults` row — 356 aiStyle 65
            // (`NPC.cs:13535`); the scorpion pair 7 (`:13665`), the
            // grasshopper 1 (`:13815`).
            NpcKind::Butterfly => 65,
            NpcKind::ScorpionBlack | NpcKind::Scorpion => 7,
            NpcKind::Grasshopper => 1,
            // W4-d39 batch 3: the teeth bomb 41 (`NPC.cs:13830`), the
            // archer pair 3 (`:13843`).
            NpcKind::ChatteringTeethBomb => 41,
            NpcKind::CultistArcherBlue | NpcKind::CultistArcherWhite => 3,
            // W4-d41 — aiStyle 3, the crawdad pair (`NPC.cs:14670`).
            NpcKind::Crawdad | NpcKind::Crawdad2 => 3,
            // W4-d41 — aiStyle 3, the nine salamanders (`NPC.cs:14695`).
            NpcKind::Salamander
            | NpcKind::Salamander2
            | NpcKind::Salamander3
            | NpcKind::Salamander4
            | NpcKind::Salamander5
            | NpcKind::Salamander6
            | NpcKind::Salamander7
            | NpcKind::Salamander8
            | NpcKind::Salamander9 => 3,
            // W4-d41 — aiStyle 3, the antlion charger (`NPC.cs:14832`).
            NpcKind::GiantWalkingAntlion => 3,
            // W4-d41 — aiStyle 44, the antlion swarmer (`NPC.cs:14845`).
            // W4-d41 — aiStyle 6, the Dune Splicer chain
            // (`NPC.cs:14778`/`:14794`/`:14813`).
            NpcKind::DuneSplicerHead
            | NpcKind::DuneSplicerBody
            | NpcKind::DuneSplicerTail => 6,
            // W4-d41 — aiStyle 6, the Tomb Crawler chain
            // (`NPC.cs:14723`/`:14739`/`:14758`).
            NpcKind::TombCrawlerHead
            | NpcKind::TombCrawlerBody
            | NpcKind::TombCrawlerTail => 6,
            // W4-d41 — aiStyle 9, the Solar Flare (`NPC.cs:15398`).
            NpcKind::SolarFlare => 9,
            // W4-d41 — aiStyle 99, the Solar Goop (`NPC.cs:15413`).
            NpcKind::SolarGoop => 99,
            // W4-d41 — aiStyle 100, the Ancient Light (`NPC.cs:15784`).
            NpcKind::AncientLight => 100,
            // W4-d41 — aiStyle 101, the Ancient Doom (`NPC.cs:15799`).
            NpcKind::AncientDoom => 101,
            // W4-d41 — aiStyle 3, the desert ghouls (`NPC.cs:14886`).
            NpcKind::DesertGhoul
            | NpcKind::DesertGhoulCorruption
            | NpcKind::DesertGhoulCrimson => 3,
            // W4-d41 — aiStyle 3, the hallowed ghoul + the lamias
            // (`NPC.cs:14886`/`:14924`).
            NpcKind::DesertGhoulHallow
            | NpcKind::DesertLamiaLight
            | NpcKind::DesertLamiaDark => 3,
            // W4-d41 — aiStyle 3, the sand poacher's walker form
            // (`NPC.cs:14937`); aiStyle 40, the wall form (`:14951`).
            NpcKind::DesertScorpionWalk => 3,
            NpcKind::DesertScorpionWall => 40,
            // W4-d41 — aiStyle 3, the basilisk + the tortured soul + the
            // bride (`NPC.cs:14964`/`:15817`/`:15847`); aiStyle 1, the
            // slimes (`:15833`/`:15862`).
            NpcKind::DesertBeast | NpcKind::DemonTaxCollector | NpcKind::TheBride => 3,
            NpcKind::SlimeSpiked | NpcKind::SandSlime => 1,
            // W4-d41 — aiStyle 7, the critters (`NPC.cs:15877`/`:15889`/
            // `:9340`); aiStyle 26, the tumbler (`:15976`).
            NpcKind::SquirrelRed | NpcKind::SquirrelGold | NpcKind::PartyBunny => 7,
            // W4-d41 — aiStyle 39, the shelly pair (`NPC.cs:14682`).
            NpcKind::GiantShelly | NpcKind::GiantShelly2 => 39,
            // The Frog walker (W4-e26, E8 keep-one) — aiStyle 7
            // (`NPC.cs:13605`). DATA-ONLY: the town machine is its own
            // wave; the row lands for the catch/release chain. The Duck
            // dropped to c10 (the waterfowl owner, SEAL S1).
            NpcKind::Frog => 7,
            // The luck critters (W4-e26) — aiStyle 115 (`NPC.cs:16767`,
            // the shared 604/605 row): `AI_115_LadyBugs`
            // (`NPC.cs:47580-47714`).
            NpcKind::LadyBug | NpcKind::GoldLadyBug => 115,
            // aiStyle 1 — doc-sweep lane 36, the ShimmerSlime
            // (`NPC.cs:17570`), the shared slime family.
            NpcKind::ShimmerSlime => 1,
            // W4-f18 first-20 port — the critters' aiStyle rows: 355/358
            // 64 (`NPC.cs:13521`/`:13563`), 356 65 (`:13535`), 357 66
            // (`:13549`), 359 67 (`:13577`).
            // W4-f18 batch 2 — 360 67 (`NPC.cs:13589`), 361/362/364 7
            // (`:13603`/`:13615`/`:13639`), 363 68 (`:13627`).
            NpcKind::Frog | NpcKind::Duck | NpcKind::DuckWhite => 7,
            // W4-f18 batch 3 — 365 68 (`NPC.cs:13651`), 366/367 7
            // (`:13663`), 374 66 (`:13764`), 375 6 (`:13779`).
            // W4-f18 batch 4 — 377 1 (`NPC.cs:13813`), 378 41
            // (`:13827`), 379/380 3 (`:13841`), 430 3 (`:8614`).
            NpcKind::ArmedZombie => 3,
            // The shimmer transform target (W4-c9's rowing, adopted by
            // W4-g62) — `NPC.cs:17583`.
            // W4-d42 - the NPCID 620-624 SetDefaults rows.
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
            NpcKind::AngryBones => 1.5,        // the num83 group (`:57606`)
            NpcKind::CorruptBunny => 1.5,      // the num83 group (`:57606`)
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
            // W4-d34 (phase 4): the Crab (67) rides the 67/220/428 slow arm
            // (`NPC.cs:57881-57906`) — cap 0.5, accel 0.03, brake 0.7 (the
            // `npc_ai3_speeds.rs:340` arm already carries the per-type step).
            NpcKind::Crab => 0.5,
            // W4-d38 b6 — AI_003 per-type walk caps.
            NpcKind::SkeletonTopHat => 1.0,
            NpcKind::SkeletonAstonaut => 1.0,
            NpcKind::SkeletonAlien => 1.0,
            // W4-d38 b3 — AI_003 per-type walk caps.
            NpcKind::TacticalSkeleton => 1.0,
            NpcKind::SkeletonCommando => 1.0,
            NpcKind::AngryBonesBig => 2.0,
            NpcKind::AngryBonesBigMuscle => 1.75,
            NpcKind::AngryBonesBigHelmet => 1.25,
            // W4-d38 b2 — AI_003 per-type walk caps.
            NpcKind::BoneLee => 5.0,
            NpcKind::Paladin => 1.0,
            NpcKind::SkeletonSniper => 1.0,
            // W4-d37 — the NPCID 211-280 band's aiStyle-3 walk caps (the
            // `AI_003_Fighters` ladder groups, `NPC.cs:57403-58401`). The
            // beetles + the mushroom zombie ride the `num83` group base 1.5f
            // (`:57606-57608`, no per-type override); the sea snail the
            // 67/220/428 slow arm 0.5f (`:57881`); the jungle/blood crawlers
            // + the anomura fungus the cap-2 group 2f (`:57554`); the mushi
            // ladybug the cap-3 group 3f (`:57520`). The ice golem + eyezor
            // carry life-scaled caps (`:57972`/`:58002`) — the full-HP base
            // pins here, the live `npc_ai3_speeds` ladder scales them. The
            // armored bones ride the `num84` group (`:57741-57791`, the
            // scale factor is 1 at the 1f default scale).
            NpcKind::CochinealBeetle | NpcKind::CyanBeetle | NpcKind::LacBeetle | NpcKind::ZombieMushroom => 1.5,
            NpcKind::SeaSnail => 0.5,
            NpcKind::JungleCreeper | NpcKind::BloodCrawler | NpcKind::AnomuraFungus => 2.0,
            NpcKind::MushiLadybug => 3.0,
            NpcKind::IceGolem | NpcKind::Eyezor => 1.0,
            // 255 is NOT in the `num83` list (`:57606`) — the `num107`
            // default 1f (`:58324`, no per-type override).
            NpcKind::ZombieMushroomHat => 1.0,
            NpcKind::RustyArmoredBonesAxe | NpcKind::BlueArmoredBonesSword => 2.0,
            NpcKind::RustyArmoredBonesFlail => 1.0,
            NpcKind::RustyArmoredBonesSword => 1.5,
            NpcKind::RustyArmoredBonesSwordNoArmor | NpcKind::BlueArmoredBonesMace => 3.0,
            NpcKind::BlueArmoredBones => 1.25,
            NpcKind::BlueArmoredBonesNoPants => 3.25,
            NpcKind::HellArmoredBones => 2.75,
            NpcKind::HellArmoredBonesSpikeShield => 1.8,
            NpcKind::HellArmoredBonesMace => 1.3,
            NpcKind::HellArmoredBonesSword => 2.5,
            // W4-d42 - the ladder's per-type caps (`npc_ai3_speeds.rs`;
            // 586's row is the full-life base of the life-scaled cap).
            NpcKind::BloodMummy => 1.5,
            // W4-d42 - the ladder's per-type caps (`npc_ai3_speeds.rs`;
            // 586's row is the full-life base of the life-scaled cap).
            NpcKind::Gnome => 2.5,
            // W4-d42 - the ladder's per-type caps (`npc_ai3_speeds.rs`;
            // 586's row is the full-life base of the life-scaled cap).
            NpcKind::WalkingAntlion => 2.5,
            NpcKind::LarvaeAntlion => 2.25,
            // W4-d42 - the ladder's per-type caps (`npc_ai3_speeds.rs`;
            // 586's row is the full-life base of the life-scaled cap).
            NpcKind::ZombieMerman => 1.5,
            // W4-d42 - the ladder's per-type caps (`npc_ai3_speeds.rs`;
            // 586's row is the full-life base of the life-scaled cap).
            NpcKind::TorchZombie => 1.0,
            NpcKind::ArmedTorchZombie => 1.0,
            // The Vampire (159, W4-d23) — the 159/349 spin arm's 6f cap
            // (`NPC.cs:57409-57438`, the `npc_ai3_speeds` ladder's own
            // arm); the row is the cap pin the walker family reads.
            // W4-g8 lane 1 batch 2 — the Nymph 196 + the WallCreeper 164
            // ride the cap-2 group (`NPC.cs:57554-57578`, ±2f, accel 0.07).
            NpcKind::Nymph | NpcKind::WallCreeper => 2.0,
            // W4-g8 machine round 3 — the Vampire 159 rides the cap-6
            // spin arm (`NPC.cs:57403-57440`, ±6f, accel 0.07, the 0.95
            // turn brake — `npc_ai3_speeds.rs`'s 159 row).
            NpcKind::Vampire => 6.0,
            // W4-d41 — the cavern crawdads' charge-family cap
            // (`NPC.cs:58267`: `num = 1f * (1f + (1f - scale))` — 1f at
            // the 1f default scale). The ladder's charge arm owns the
            // live walk; this row feeds the walker fallback + the pin.
            NpcKind::Crawdad | NpcKind::Crawdad2 => 1.0,
            // W4-d41 — the antlion charger's lerp-machine base cap
            // (`NPC.cs:58141`: `num98 = 2.5f` — the speed-tiered bumps
            // ride the ladder's own arm; this row feeds the walker
            // fallback + the pin).
            NpcKind::GiantWalkingAntlion => 2.5,
            // W4-d41 — the desert ghouls' cap-2 group
            // (`NPC.cs:57554-57579`: the 524-527 range).
            NpcKind::DesertGhoul
            | NpcKind::DesertGhoulCorruption
            | NpcKind::DesertGhoulCrimson => 2.0,
            // W4-d41 — the hallowed ghoul's cap-2 group
            // (`NPC.cs:57554-57579`), the lamias' cap-3 group
            // (`:57520-57553`), the sand poacher walker's cap-2 group.
            NpcKind::DesertGhoulHallow => 2.0,
            NpcKind::DesertLamiaLight | NpcKind::DesertLamiaDark => 3.0,
            NpcKind::DesertScorpionWalk => 2.0,
            // W4-d41 — the basilisk's carrier-group cap
            // (`NPC.cs:58232-58237`: the 5/0.15/0.98 row), the tortured
            // soul's + the bride's num107 default 1f (`:58322`).
            NpcKind::DesertBeast => 5.0,
            NpcKind::DemonTaxCollector | NpcKind::TheBride => 1.0,
            NpcKind::Salamander
            | NpcKind::Salamander2
            | NpcKind::Salamander3
            | NpcKind::Salamander4
            | NpcKind::Salamander5
            | NpcKind::Salamander6
            | NpcKind::Salamander7
            | NpcKind::Salamander8
            | NpcKind::Salamander9 => 1.0,
            // W4-f18 batch 4 — the cultist archers' shooter-default 1f
            // (`NPC.cs:59717`, no 379/380 override); the ArmedZombie 430's
            // charge-family cap 1f (`:58267`, the scale factor is 1 at the
            // 1f default scale — the ladder applies it).
            NpcKind::CultistArcherBlue | NpcKind::CultistArcherWhite => 1.0,
            NpcKind::ArmedZombie => 1.0,
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
            // W4-c4 (the style-16 swimmers): 57/241/465 18x20
            // (`NPC.cs:9492-9493`/`:12109-12110`/`:14486-14487`), 157
            // 74x20 (`:10993-10994`), 592/607/615 20x18 (`:16682-16683`/
            // `:16798-16799`/`:16895-16896`), 688 32x16 (`:17656-17657`),
            // 692 120x34 (`:17712-17713`).
            // E8 merge drops: CorruptGoldfish/BloodFeeder (d34/d37 rows
            // above), Pufferfish/Orca (d43 lands later).
            NpcKind::CrimsonGoldfish => (18.0, 20.0),
            NpcKind::Arapaima => (74.0, 20.0),
            NpcKind::GoldGoldfish | NpcKind::Pupfish | NpcKind::Dolphin => (20.0, 18.0),
            // W4-e21 — the bunny pair: 18x20 (`NPC.cs:9338-9339`).
            NpcKind::Bunny | NpcKind::PartyBunny => (18.0, 20.0),
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
            // The aiStyle-14 flyer family (W4-d23): the thirteen kinds'
            // SetDefaults boxes — 48 24x34 (`NPC.cs:9362-9363`), 62/156
            // 28x48 (`:9570-9571`/`:10977-10978`), 93/137 26x20
            // (`:10050-10051`/`:10737-10738`), 121/660 40x30
            // (`:10468-10469`/`:17340-17341`), 150/151/158 22x22
            // (`:10888-10889`/`:10903-10904`/`:15469-15470`), 152 38x34
            // (`:10919-10920`), 159 18x40 (`:15483-15484`), 226 34x50
            // (`:11914-11915`), 634 22x18 (`:17164-17165`).
            NpcKind::Harpy => (24.0, 34.0),
            NpcKind::Demon | NpcKind::RedDevil => (28.0, 48.0),
            NpcKind::GiantBat | NpcKind::IlluminantBat => (26.0, 20.0),
            NpcKind::Slimer | NpcKind::QueenSlimeMinionPurple => (40.0, 30.0),
            NpcKind::IceBat | NpcKind::Lavabat | NpcKind::VampireBat => (22.0, 22.0),
            NpcKind::GiantFlyingFox => (38.0, 34.0),
            NpcKind::Vampire => (18.0, 40.0),
            NpcKind::FlyingSnake => (34.0, 50.0),
            NpcKind::SporeBat => (22.0, 18.0),
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
            // W4-c6 style-59 wiring: the blade's 80x80 box
            // (`NPC.cs:13160-13161`).
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
            // W4-d40: the SetDefaults width/height boxes (`NPC.cs`).
            NpcKind::ArmedZombie
            | NpcKind::ArmedZombieEskimo
            | NpcKind::ArmedZombiePincussion
            | NpcKind::ArmedZombieSlimed
            | NpcKind::ArmedZombieSwamp => (18.0, 40.0),
            // W4-d40: the SetDefaults width/height boxes (`NPC.cs`).
            NpcKind::ArmedZombieTwiggy
            | NpcKind::ArmedZombieCenx => (18.0, 40.0),
            NpcKind::CultistTablet => (54.0, 54.0),
            NpcKind::CultistDevote => (22.0, 40.0),
            NpcKind::CultistBossClone => (24.0, 50.0),
            // W4-d40: the SetDefaults width/height boxes (`NPC.cs`).
            NpcKind::GoldBird => (14.0, 14.0),
            NpcKind::GoldBunny => (18.0, 20.0),
            NpcKind::GoldButterfly => (10.0, 10.0),
            NpcKind::GoldFrog => (12.0, 10.0),
            NpcKind::GoldGrasshopper => (14.0, 10.0),
            // W4-d40: the SetDefaults width/height boxes (`NPC.cs`).
            NpcKind::GoldMouse => (14.0, 12.0),
            NpcKind::GoldWorm => (10.0, 4.0),
            NpcKind::BoneThrowingSkeleton
            | NpcKind::BoneThrowingSkeleton2
            | NpcKind::BoneThrowingSkeleton3 => (18.0, 40.0),
            // W4-d40: the SetDefaults width/height boxes (`NPC.cs`).
            NpcKind::BoneThrowingSkeleton4
            | NpcKind::SkeletonMerchant => (18.0, 40.0),
            NpcKind::CultistDragonHead
            | NpcKind::CultistDragonBody1
            | NpcKind::CultistDragonBody2 => (32.0, 32.0),
            // W4-d40: the SetDefaults width/height boxes (`NPC.cs`).
            NpcKind::CultistDragonBody3
            | NpcKind::CultistDragonBody4
            | NpcKind::CultistDragonTail => (32.0, 32.0),
            NpcKind::Butcher
            | NpcKind::CreatureFromTheDeep => (18.0, 40.0),
            // W4-d40: the SetDefaults width/height boxes (`NPC.cs`).
            NpcKind::Fritz => (20.0, 24.0),
            NpcKind::Nailhead
            | NpcKind::Psycho => (18.0, 40.0),
            NpcKind::CrimsonBunny
 => (18.0, 20.0),
            // W4-d40: the SetDefaults width/height boxes (`NPC.cs`).
            NpcKind::DeadlySphere => (22.0, 22.0),
            NpcKind::DrManFly => (18.0, 40.0),
            NpcKind::ThePossessed => (38.0, 26.0),
            NpcKind::CrimsonPenguin => (16.0, 34.0),
            NpcKind::ShadowFlameApparition => (40.0, 24.0),
            // W4-d40: the SetDefaults width/height boxes (`NPC.cs`).
            NpcKind::BigMimicCorruption
            | NpcKind::BigMimicCrimson
            | NpcKind::BigMimicHallow
            | NpcKind::BigMimicJungle => (28.0, 44.0),
            NpcKind::Mothron => (80.0, 50.0),
            // W4-d40: the SetDefaults width/height boxes (`NPC.cs`).
            NpcKind::MothronEgg => (34.0, 34.0),
            NpcKind::MothronSpawn => (46.0, 30.0),
            NpcKind::Medusa
            | NpcKind::GreekSkeleton => (18.0, 40.0),
            NpcKind::GraniteGolem => (28.0, 48.0),
            // W4-d40: the SetDefaults width/height boxes (`NPC.cs`).
            NpcKind::GraniteFlyer => (20.0, 30.0),
            NpcKind::EnchantedNightcrawler
            | NpcKind::Grubby
            | NpcKind::Sluggy
            | NpcKind::Buggy => (10.0, 4.0),
            // W4-d40: the SetDefaults width/height boxes (`NPC.cs`).
            NpcKind::TargetDummy => (18.0, 40.0),
            // W4-d35 (A): the 73/74/75/77/78 SetDefaults boxes.
            NpcKind::GoblinScout => (18.0, 40.0),
            NpcKind::Bird => (14.0, 14.0),
            NpcKind::Pixie => (20.0, 20.0),
            NpcKind::ArmoredSkeleton => (18.0, 40.0),
            NpcKind::Mummy => (18.0, 40.0),
            // W4-d35 (B): the 79/80/82/83/84 SetDefaults boxes.
            NpcKind::DarkMummy => (18.0, 40.0),
            NpcKind::LightMummy => (18.0, 40.0),
            NpcKind::Wraith => (24.0, 44.0),
            NpcKind::CursedHammer => (40.0, 40.0),
            NpcKind::EnchantedSword => (40.0, 40.0),
            // W4-d35 (C): the 85/86/93/94/95 SetDefaults boxes.
            NpcKind::Mimic => (24.0, 24.0),
            NpcKind::Unicorn => (46.0, 42.0),
            NpcKind::GiantBat => (26.0, 20.0),
            NpcKind::Corruptor => (44.0, 44.0),
            NpcKind::DiggerHead => (22.0, 22.0),
            // W4-d35 (D): the 96/97/98/99/100 SetDefaults boxes.
            NpcKind::DiggerBody => (22.0, 22.0),
            NpcKind::DiggerTail => (22.0, 22.0),
            NpcKind::SeekerHead => (22.0, 22.0),
            NpcKind::SeekerBody => (22.0, 22.0),
            NpcKind::SeekerTail => (22.0, 22.0),
            // W4-d35 (E): the 101/102/103/104/109 SetDefaults boxes.
            NpcKind::Clinger => (30.0, 30.0),
            NpcKind::AnglerFish => (18.0, 20.0),
            NpcKind::GreenJellyfish => (26.0, 26.0),
            NpcKind::Werewolf => (18.0, 40.0),
            NpcKind::Clown => (34.0, 78.0),
            // W4-d35 (F): the 110/112/116/120/121 SetDefaults boxes.
            NpcKind::SkeletonArcher => (18.0, 40.0),
            NpcKind::VileSpit => (16.0, 16.0),
            NpcKind::TheHungryII => (30.0, 32.0),
            NpcKind::ChaosElemental => (18.0, 40.0),
            NpcKind::Slimer => (40.0, 30.0),
            // W4-d35 (G): the 122/137/140 SetDefaults boxes.
            NpcKind::Gastropod => (20.0, 20.0),
            NpcKind::IlluminantBat => (26.0, 20.0),
            NpcKind::PossessedArmor => (18.0, 40.0),
            NpcKind::DungeonSpirit => (20.0, 20.0),
            // W4-d34 kinds 1-70 (`NPC.cs` SetDefaults width/height).
            NpcKind::AngryBones => (18.0, 40.0),  // :9110-9111
            NpcKind::CursedSkull => (26.0, 28.0), // :9152-9153
            NpcKind::OldMan => (18.0, 40.0),      // :9201-9202
            NpcKind::ManEater => (30.0, 30.0),    // :9297-9298
            NpcKind::Bunny => (18.0, 20.0),       // :9338-9339
            NpcKind::BoneSerpentHead | NpcKind::BoneSerpentBody | NpcKind::BoneSerpentTail => (22.0, 22.0), // :9228/:9245/:9263
            NpcKind::CorruptBunny => (18.0, 20.0),  // :9350-9351
            NpcKind::Harpy => (24.0, 34.0),         // :9362-9363
            NpcKind::Snatcher => (30.0, 30.0),        // :9478-9479
            NpcKind::CorruptGoldfish => (18.0, 20.0), // :9492-9493
            NpcKind::Vulture => (36.0, 36.0),         // :9556-9557
            NpcKind::Demon => (28.0, 48.0),           // :9570-9571
            NpcKind::BlueJellyfish => (26.0, 26.0),   // :9585-9586
            NpcKind::Crab => (28.0, 20.0),            // :9642-9643
            NpcKind::DungeonGuardian => (80.0, 102.0), // :9654-9655
            // W4-d38 b7 — SetDefaults width/height rows.
            NpcKind::SlimeRibbonWhite => (24.0, 18.0),
            NpcKind::SlimeRibbonYellow => (24.0, 18.0),
            NpcKind::SlimeRibbonGreen => (24.0, 18.0),
            NpcKind::SlimeRibbonRed => (24.0, 18.0),
            NpcKind::BunnyXmas => (18.0, 20.0),
            // W4-d38 b6 — SetDefaults width/height rows.
            NpcKind::DemonEyeSpaceship => (30.0, 32.0),
            NpcKind::SkeletonTopHat => (18.0, 40.0),
            NpcKind::SkeletonAstonaut => (18.0, 40.0),
            NpcKind::SkeletonAlien => (18.0, 40.0),
            NpcKind::PumpkingBlade => (80.0, 80.0),
            // W4-d38 b5 — SetDefaults width/height rows.
            NpcKind::SlimeMasked => (24.0, 18.0),
            NpcKind::BunnySlimed => (18.0, 20.0),
            NpcKind::HoppinJack => (34.0, 28.0),
            NpcKind::Ghost => (24.0, 44.0),
            NpcKind::DemonEyeOwl => (30.0, 32.0),
            // W4-d38 b4 — SetDefaults width/height rows.
            NpcKind::BirdBlue => (14.0, 14.0),
            NpcKind::BirdRed => (14.0, 14.0),
            NpcKind::Squirrel => (18.0, 20.0),
            NpcKind::Mouse => (14.0, 12.0),
            NpcKind::Raven => (36.0, 26.0),
            // W4-d38 b3 — SetDefaults width/height rows.
            NpcKind::TacticalSkeleton => (18.0, 40.0),
            NpcKind::SkeletonCommando => (18.0, 40.0),
            NpcKind::AngryBonesBig => (18.0, 40.0),
            NpcKind::AngryBonesBigMuscle => (18.0, 40.0),
            NpcKind::AngryBonesBigHelmet => (18.0, 40.0),
            // W4-d38 b2 — SetDefaults width/height rows.
            NpcKind::DiabolistWhite => (18.0, 40.0),
            NpcKind::BoneLee => (18.0, 40.0),
            NpcKind::GiantCursedSkull => (44.0, 44.0),
            NpcKind::Paladin => (34.0, 62.0),
            NpcKind::SkeletonSniper => (18.0, 40.0),
            // W4-d38 b1 — SetDefaults width/height rows.
            NpcKind::RaggedCaster => (18.0, 40.0),
            NpcKind::RaggedCasterOpenCoat => (18.0, 40.0),
            NpcKind::Necromancer => (18.0, 40.0),
            NpcKind::NecromancerArmored => (18.0, 40.0),
            NpcKind::DiabolistRed => (18.0, 40.0),
            // W4-d37 — the NPCID 211-280 band's `SetDefaults` boxes (the
            // `base.width`/`base.height` cites in `npc.rs`'s enum).
            NpcKind::CochinealBeetle | NpcKind::CyanBeetle | NpcKind::LacBeetle | NpcKind::SeaSnail => {
                (28.0, 20.0)
            }
            NpcKind::Squid | NpcKind::BloodJelly | NpcKind::FungoFish => (26.0, 26.0),
            NpcKind::FlyingFish => (32.0, 18.0),
            NpcKind::FlyingSnake => (34.0, 50.0),
            NpcKind::Pirate => (18.0, 40.0),
            NpcKind::GoldfishWalker => (18.0, 20.0),
            NpcKind::HornetFatty
            | NpcKind::HornetHoney
            | NpcKind::HornetLeafy
            | NpcKind::HornetSpikey
            | NpcKind::HornetStingy => (34.0, 32.0),
            NpcKind::JungleCreeper | NpcKind::BloodCrawler => (50.0, 20.0),
            NpcKind::JungleCreeperWall | NpcKind::BlackRecluseWall | NpcKind::BloodCrawlerWall => {
                (36.0, 36.0)
            }
            NpcKind::BloodFeeder => (18.0, 20.0),
            NpcKind::IceGolem => (30.0, 114.0),
            NpcKind::RainbowSlime => (60.0, 42.0),
            NpcKind::Eyezor
            | NpcKind::ZombieMushroom
            | NpcKind::ZombieMushroomHat
            | NpcKind::RustyArmoredBonesAxe
            | NpcKind::RustyArmoredBonesFlail
            | NpcKind::RustyArmoredBonesSword
            | NpcKind::RustyArmoredBonesSwordNoArmor
            | NpcKind::BlueArmoredBones
            | NpcKind::BlueArmoredBonesMace
            | NpcKind::BlueArmoredBonesNoPants
            | NpcKind::BlueArmoredBonesSword
            | NpcKind::HellArmoredBones
            | NpcKind::HellArmoredBonesSpikeShield
            | NpcKind::HellArmoredBonesMace
            | NpcKind::HellArmoredBonesSword => (18.0, 40.0),
            NpcKind::Parrot => (32.0, 32.0),
            NpcKind::Reaper => (24.0, 44.0),
            NpcKind::AnomuraFungus => (44.0, 34.0),
            NpcKind::MushiLadybug => (30.0, 24.0),
            NpcKind::FungiBulb => (20.0, 20.0),
            NpcKind::GiantFungiBulb => (36.0, 36.0),
            NpcKind::IchorSticker => (28.0, 56.0),
            // W4-d42 - the NPCID 578-582 SetDefaults rows.
            NpcKind::DD2LightningBug => (30.0, 24.0),
            NpcKind::BartenderUnconscious => (34.0, 8.0),
            NpcKind::WalkingAntlion => (26.0, 26.0),
            NpcKind::LarvaeAntlion => (18.0, 18.0),
            // W4-d42 - the NPCID 583-587 SetDefaults rows.
            NpcKind::ZombieMerman => (18.0, 40.0),
            // W4-d42 - the NPCID 590-594 SetDefaults rows.
            NpcKind::TorchZombie => (18.0, 40.0),
            NpcKind::ArmedTorchZombie => (18.0, 40.0),
            NpcKind::GoldGoldfishWalker => (18.0, 20.0),
            // W4-d42 - the NPCID 595-599 SetDefaults rows.
            // W4-d42 - the NPCID 600-604 SetDefaults rows.
            NpcKind::Seagull => (22.0, 26.0),
            NpcKind::Seagull2 => (28.0, 22.0),
            // W4-d42 - the NPCID 605-609 SetDefaults rows.
            NpcKind::Grebe => (22.0, 26.0),
            NpcKind::Grebe2 => (28.0, 22.0),
            // W4-d42 - the NPCID 610-614 SetDefaults rows.
            NpcKind::Rat => (14.0, 12.0),
            NpcKind::Owl => (18.0, 34.0),
            NpcKind::WaterStrider => (10.0, 10.0),
            NpcKind::GoldWaterStrider => (10.0, 10.0),
            NpcKind::ExplosiveBunny => (18.0, 20.0),
            // W4-d42 - the NPCID 615-619 SetDefaults rows.
            NpcKind::Turtle => (22.0, 20.0),
            NpcKind::TurtleJungle => (22.0, 20.0),
            NpcKind::BloodNautilus => (100.0, 100.0),
            NpcKind::BloodSquid => (44.0, 44.0),
            // W4-d42 - the NPCID 620-624 SetDefaults rows.
            NpcKind::GoblinShark => (34.0, 58.0),
            NpcKind::BloodEelHead => (28.0, 28.0),
            NpcKind::BloodEelBody => (28.0, 28.0),
            NpcKind::BloodEelTail => (28.0, 28.0),
            NpcKind::Gnome => (14.0, 30.0),
            // W4-d42 - the NPCID 625-629 SetDefaults rows.
            NpcKind::SeaTurtle => (22.0, 20.0),
            NpcKind::Seahorse => (16.0, 20.0),
            NpcKind::GoldSeahorse => (16.0, 20.0),
            NpcKind::Dandelion => (22.0, 40.0),
            NpcKind::IceMimic => (24.0, 24.0),
            // W4-d42 - the NPCID 630-630 SetDefaults rows.
            NpcKind::BloodMummy => (18.0, 40.0),
            // W4-c3: 694 44x44 (`NPC.cs:17739-17740`); the 34/289 size
            // rows are d34/d38's above (identical values, E8 keep-both).
            NpcKind::WaterBoltMimic => (44.0, 44.0),
            // W4-c4 (the style-44 swimmers): 224 32x18 (`NPC.cs:11887-
            // 11888`), 509 40x31 (`:14843-14844`), 581 30x22 (`:14870-
            // 14871`), 587 32x18 (`:16625-16626`).
            NpcKind::EyeballFlyingFish => (32.0, 18.0),
            NpcKind::GiantFlyingAntlion => (40.0, 31.0),
            NpcKind::FlyingAntlion => (30.0, 22.0),
            // W4-c7 — the firefly family's boxes: 355/654/677 10x10
            // (`NPC.cs:13519-13520`/`:17267-17268`/`:17581-17582`), 358
            // 12x12 (`:13561-13562`).
            NpcKind::Firefly | NpcKind::Lavafly | NpcKind::Shimmerfly => (10.0, 10.0),
            NpcKind::LightningBug => (12.0, 12.0),
            // W4-c8 — the crawling critters' boxes: the eight style-66
            // kinds are 10x4 except the TruffleWorm's 20x10
            // (`NPC.cs:13547-13548`/`:13764-13765`/`:14334-14335`/
            // `:14578-14579`/`:16783-16784`); the digger 375 is 10x10
            // (`:13779-13780`).
            NpcKind::TruffleWorm => (20.0, 10.0),
            NpcKind::TruffleWormDigger => (10.0, 10.0),
            // E8 merge: the 5 d40-owned dups dropped (rows above).
            NpcKind::Worm
            | NpcKind::Maggot => (10.0, 4.0),
            // The snail family + its shimmer target (W4-c9): 359 12x12
            // (`NPC.cs:13575-13576`), 360/655 14x14 (`:13589-13590` /
            // `:17282-17283`), 677 10x10 (`:17581-17582`). The 359 scale
            // roll's resize rides `size_override` (the machine's).
            // E8 merge: Shimmerfly's size is c7's above; the 5 gold
            // sizes are d40's above (the dup drops).
            NpcKind::Snail => (12.0, 12.0),
            NpcKind::GlowingSnail | NpcKind::MagmaSnail => (14.0, 14.0),
            // W4-c10 — the waterfowl boxes: the walkers 22x26
            // (`NPC.cs:13615-13616`/`:13639-13640`/`:16741-16742`/
            // `:16811-16812`), the swimmers 28x22 (`:13627-13628`/
            // `:13651-13652`/`:16753-16754`/`:16823-16824`).
            // E8 merge: Seagull/Grebe's size rows are d42's above.
            NpcKind::Duck | NpcKind::DuckWhite => {
                (22.0, 26.0)
            }
            NpcKind::Duck2 | NpcKind::DuckWhite2 => {
                (28.0, 22.0)
            }
            // E8 merge: the c10 12-critter size tail dropped (d40/c8's rows).
            // The man-eater family (W4-d23): 43/56/101/175 30x30
            // (`NPC.cs:9297-9298`/`:9478-9479`/`:10191-10192`/
            // `:11212-11213`), 259 20x20 (`:12336-12337`), 260 36x36
            // (`:12352-12353`).
            NpcKind::ManEater | NpcKind::Snatcher | NpcKind::Clinger | NpcKind::AngryTrapper => (30.0, 30.0),
            NpcKind::FungiBulb => (20.0, 20.0),
            NpcKind::GiantFungiBulb => (36.0, 36.0),
            // The aiStyle-9 spit/flare family (W4-d23): 112/666 16x16
            // (`NPC.cs:10328-10329`/`:17420-17421`), 516 26x26
            // (`:15396-15397`).
            NpcKind::VileSpit | NpcKind::VileSpitEoW => (16.0, 16.0),
            NpcKind::SolarFlare => (26.0, 26.0),
            // W4-d25 STYLE-26 wiring: 86 46x42 (`NPC.cs:9928-9929`),
            // 155 46x30 (`:10963-10964`), 546 30x30 (`:15974-15975`).
            // The Unicorn row mirrors d35 (C).
            NpcKind::Wolf => (46.0, 30.0),
            NpcKind::Tumbleweed => (30.0, 30.0),
            // W4-d120 — SetDefaults width/height rows: the four town kinds
            // 18x40 (`NPC.cs:10795-10796`/`:11007-11008`/`:11256-11257`/
            // `:11686-11687`), the bound Tavernkeep 34x8 (`:16558-16559`).
            NpcKind::SantaClaus | NpcKind::Truffle | NpcKind::Steampunker | NpcKind::Cyborg => {
                (18.0, 40.0)
            }
            // W4-d125: 636 100x100 (`NPC.cs:17191-17192`), 661 10x10
            // (`:17352-17353`).
            NpcKind::EmpressOfLight => (100.0, 100.0),
            NpcKind::PrismaticLacewing => (10.0, 10.0),
            // W4-g5 — the two casters' `SetDefaults` boxes: 172 18x40
            // (`NPC.cs:11156-11157`), 693 18x40 (`:17725-17726`).
            NpcKind::RuneWizard | NpcKind::LibrarianSkeleton => (18.0, 40.0),
            // W4-g8 lane 1 batch 1 — the `SetDefaults` boxes: 61 36x36
            // (`NPC.cs:9556-9557`), 63/64 26x26 (`:9585-9586`/`:9599-9600`),
            // 69 24x24 (`:9668-9669`), 70 34x34 (`:9683-9684`), 75 20x20
            // (`:9775-9776`), 82 24x44 (`:9856-9857`), 83/84 40x40
            // (`:9881-9882`/`:9894-9895`), 85 24x24 (`:9907-9908`).
            NpcKind::Antlion => (24.0, 24.0),
            NpcKind::SpikeBall => (34.0, 34.0),
            // W4-g8 lane 1 batch 2 — the `SetDefaults` boxes: 103 26x26
            // (`NPC.cs:10219-10220`), 122 20x20 (`:10482-10483`), 164 50x20
            // (`:11061-11062`), 165 36x36 (`:11076-11077`), 169 18x40
            // (`:11116-11117`), 174 52x26 (`:11197-11198`), 177 58x44
            // (`:11241-11242`), 179 40x40 (`:11268-11269`), 182 24x62
            // (`:11320-11321`), 195/196 18x40 (`:11494-11495`/`:11508-11509`).
            NpcKind::WallCreeper => (50.0, 20.0),
            NpcKind::WallCreeperWall => (36.0, 36.0),
            NpcKind::IceElemental => (18.0, 40.0),
            NpcKind::Herpling => (52.0, 26.0),
            NpcKind::Derpling => (58.0, 44.0),
            NpcKind::CrimsonAxe => (40.0, 40.0),
            NpcKind::FloatyGross => (24.0, 62.0),
            NpcKind::LostGirl | NpcKind::Nymph => (18.0, 40.0),
            // W4-g8 lane 1 batch 3 — the `SetDefaults` boxes: 34 26x28
            // (`NPC.cs:9152-9153`), 62 28x48 (`:9570-9571`), 93/137 26x20
            // (`:10050-10051`/`:10737-10738`), 121 40x30 (`:10468-10469`),
            // 150/151/158 22x22 (`:10888-10889`/`:10903-10904`/`:15469-15470`),
            // 152 38x34 (`:10919-10920`), 226 34x50 (`:11914-11915`).
            // W4-g8 lane 1 batch 4 — the `SetDefaults` boxes: 221 26x26
            // (`NPC.cs:11843-11844`), 48 24x34 (`:9362-9363`), 156 28x48
            // (`:10977-10978`).
            // W4-g8 machine round 3 — the Vampire 159's 18x40 box
            // (`NPC.cs:15483-15484`).
            // The dragonflies (W4-d33): 10x10 each (`NPC.cs:16723-16724`,
            // the shared 595-601 row).
            NpcKind::BlackDragonfly
            | NpcKind::BlueDragonfly
            | NpcKind::GreenDragonfly
            | NpcKind::OrangeDragonfly
            | NpcKind::RedDragonfly
            | NpcKind::YellowDragonfly
            | NpcKind::GoldDragonfly => (10.0, 10.0),
            // The fairy critters (W4-d33 wiring): 18x20 each
            // (`NPC.cs:16585-16586`, the shared 583-585 row); the windy
            // balloon 20x20 (`:16709-16710` — the aiStyle-113 arm scales
            // the box to the slave each tick). d42 batch-2/3's rows,
            // constants-copy (keep-one d37c-E3).
            NpcKind::FairyCritterPink
            | NpcKind::FairyCritterGreen
            | NpcKind::FairyCritterBlue => (18.0, 20.0),
            NpcKind::WindyBalloon => (20.0, 20.0),
            // W4-f42 — the sandstorm debris + desert family's `SetDefaults`
            // boxes: 546 30x30 (`NPC.cs:15974-15975`), 61 36x36
            // (`:9556-9557`), 69 24x24 (`:9668-9669`), the mummies 18x40
            // (`:9801-9802`/`:9814-9815`/`:17109-17110`/`:9827-9828`),
            // 580 26x26 (`:14856-14857`), 581 30x22 (`:14870-14871`).
            NpcKind::Mummy | NpcKind::DarkMummy | NpcKind::BloodMummy | NpcKind::LightMummy => {
                (18.0, 40.0)
            }
            // W4-d9 wiring (d40 rows): the SetDefaults width/height boxes
            // (`NPC.cs:14418-14419`/`:14436-14437`/`:14455-14456`); the
            // 521 squidhead's 60x60 (`NPC.cs:15770-15771`).
            | NpcKind::CultistDragonBody2
            | NpcKind::CultistDragonBody3
            NpcKind::AncientCultistSquidhead => (60.0, 60.0),
            // W4-d39: the Butterfly's `SetDefaults` box — 356 10x10
            // (`NPC.cs:13533-13534`); the scorpion pair 26x18
            // (`:13663-13664`), the grasshopper 14x10 (`:13813-13814`).
            NpcKind::Butterfly => (10.0, 10.0),
            NpcKind::ScorpionBlack | NpcKind::Scorpion => (26.0, 18.0),
            NpcKind::Grasshopper => (14.0, 10.0),
            // W4-d39 batch 3: the teeth bomb 20x26 (`NPC.cs:13828-
            // 13829` — the explosion's 160x160 resize rides
            // `size_override`), the archer pair 18x40 (`:13841-13842`).
            NpcKind::ChatteringTeethBomb => (20.0, 26.0),
            NpcKind::CultistArcherBlue | NpcKind::CultistArcherWhite => (18.0, 40.0),
            // W4-d41 — the cavern crawdads'/shellies' 28x22 boxes
            // (`NPC.cs:14668-14669`/`:14680-14681`).
            NpcKind::Crawdad | NpcKind::Crawdad2 => (28.0, 22.0),
            NpcKind::GiantShelly | NpcKind::GiantShelly2 => (28.0, 22.0),
            // W4-d41 — the salamanders' 24x44 box (`NPC.cs:14693-14694`).
            NpcKind::Salamander
            | NpcKind::Salamander2
            | NpcKind::Salamander3
            | NpcKind::Salamander4
            | NpcKind::Salamander5
            | NpcKind::Salamander6
            | NpcKind::Salamander7
            | NpcKind::Salamander8
            | NpcKind::Salamander9 => (24.0, 44.0),
            // W4-d41 — the antlion charger's 50x31 box
            // (`NPC.cs:14830-14831`), the swarmer's 40x31
            // (`:14843-14844`), the Dune Splicer chain's 34x34
            // (`:14776-14777`/`:14792-14793`/`:14811-14812`).
            NpcKind::GiantWalkingAntlion => (50.0, 31.0),
            NpcKind::DuneSplicerHead
            | NpcKind::DuneSplicerBody
            | NpcKind::DuneSplicerTail => (34.0, 34.0),
            // W4-d41 — the Tomb Crawler chain's 22x22 box
            // (`NPC.cs:14721-14722`/`:14737-14738`/`:14756-14757`), the
            // solar fragments' 26x26 (`:15396-15397`/`:15411-15412`).
            NpcKind::TombCrawlerHead
            | NpcKind::TombCrawlerBody
            | NpcKind::TombCrawlerTail => (22.0, 22.0),
            // W4-d41 — the Ancient Light's 26x26 box
            // (`NPC.cs:15782-15783`), the Ancient Doom's 42x42
            // (`:15797-15798`), the desert ghouls' 24x44
            // (`:14884-14885`).
            NpcKind::AncientLight => (26.0, 26.0),
            NpcKind::AncientDoom => (42.0, 42.0),
            NpcKind::DesertGhoul
            | NpcKind::DesertGhoulCorruption
            | NpcKind::DesertGhoulCrimson => (24.0, 44.0),
            // W4-d41 — the hallowed ghoul's + the lamias' 24x44 box
            // (`NPC.cs:14884-14885`/`:14922-14923`), the sand poacher
            // walker's 50x20 (`:14935-14936`), the wall's 36x36
            // (`:14949-14950`).
            NpcKind::DesertGhoulHallow
            | NpcKind::DesertLamiaLight
            | NpcKind::DesertLamiaDark => (24.0, 44.0),
            NpcKind::DesertScorpionWalk => (50.0, 20.0),
            NpcKind::DesertScorpionWall => (36.0, 36.0),
            // W4-d41 — the basilisk's 32x31 box (`NPC.cs:14962-14963`),
            // the tortured soul's + the bride's 18x40
            // (`:15815-15816`/`:15845-15846`), the spiked slime's 24x18
            // (`:15831-15832`), the sand slime's 30x24
            // (`:15860-15861`).
            NpcKind::DesertBeast => (32.0, 31.0),
            NpcKind::DemonTaxCollector | NpcKind::TheBride => (18.0, 40.0),
            NpcKind::SlimeSpiked => (24.0, 18.0),
            NpcKind::SandSlime => (30.0, 24.0),
            // W4-d41 — the critters' 18x20 box (`NPC.cs:15875-15876`/
            // `:15887-15888`/`:9338-9339`), the tumbler's 30x30
            // (`:15974-15975`).
            // The luck critters (W4-e26, E8 keep-one): Frog 12x10
            // (`NPC.cs:13603-13604`), the ladybug pair 10x10
            // (`:16765-16766`). The Duck dropped to c10.
            NpcKind::Frog => (12.0, 10.0),
            NpcKind::LadyBug | NpcKind::GoldLadyBug => (10.0, 10.0),
            // W4-g10 lane 3 — the town pets' `SetDefaults` boxes: the
            // 637/638 arm's 18x28 base with the 637-only 20-height
            // override (`NPC.cs:17211-17216`), 656 18x20 (`:17299-17300`),
            // 670 18x20 (`:17485-17486`), the 678-684 arm 18x20
            // (`:17598-17599`).
            NpcKind::TownCat => (18.0, 20.0),
            NpcKind::TownDog => (18.0, 28.0),
            NpcKind::TownBunny
            | NpcKind::TownSlimeBlue
            | NpcKind::TownSlimeGreen
            | NpcKind::TownSlimeOld
            | NpcKind::TownSlimePurple
            | NpcKind::TownSlimeRainbow
            | NpcKind::TownSlimeRed
            | NpcKind::TownSlimeYellow
            | NpcKind::TownSlimeCopper => (18.0, 20.0),
            // Doc-sweep lane 36 — 676 24x18 (`NPC.cs:17568-17569`).
            NpcKind::ShimmerSlime => (24.0, 18.0),
            // W4-f18 first-20 port — the critters' `SetDefaults` boxes:
            // 355 10x10 (`NPC.cs:13519-13520`), 356 10x10
            // (`:13533-13534`), 357 10x4 (`:13547-13548`), 358 12x12
            // (`:13561-13562`), 359 12x12 (`:13575-13576`).
            // W4-f18 batch 2 — 360 14x14 (`NPC.cs:13587-13588`), 361 12x10
            // (`:13601-13602`), 362/364 22x26 (`:13613-13614`/
            // `:13637-13638`), 363 28x22 (`:13625-13626`).
            // W4-f18 batch 3 — 365 28x22 (`NPC.cs:13649-13650`), 366/367
            // 26x18 (`:13661-13662`), 374 20x10 (`:13762-13763`), 375
            // 10x10 (`:13777-13778`).
            // W4-f18 batch 4 — 377 14x10 (`NPC.cs:13811-13812`), 378 20x26
            // (`:13825-13826`), 379/380 18x40 (`:13839-13840`), 430 18x40
            // (`:8612-8613`).
            // The Shimmerfly (W4-c9's rowing, adopted by W4-g62): 677
            // 10x10 (`NPC.cs:17581-17582`).
            // W4-d42 - the NPCID 620-624 SetDefaults rows.
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
            // W4-d120 — the chatter kinds' vanilla types (`NPCID.cs:10961`/
            // `:10997`/`:11033`/`:11095`/`:11835`); the protocol ids ride
            // the port-local 241-245 band (`to_u16`).
            NpcKind::SantaClaus => 142,
            NpcKind::Truffle => 160,
            NpcKind::Steampunker => 178,
            NpcKind::Cyborg => 209,
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
            // W4-d35 (A): the 73/74/75/77/78 vanilla NPC.type rows.
            NpcKind::GoblinScout => 73,
            NpcKind::Bird => 74,
            NpcKind::ArmoredSkeleton => 77,
            NpcKind::Mummy => 78,
            // W4-d35 (B): the 79/80/82/83/84 vanilla NPC.type rows.
            NpcKind::DarkMummy => 79,
            NpcKind::LightMummy => 80,
            // W4-d35 (C): the 85/86/93/94/95 vanilla NPC.type rows.
            NpcKind::Unicorn => 86,
            NpcKind::Corruptor => 94,
            NpcKind::DiggerHead => 95,
            // W4-d35 (D): the 98 vanilla NPC.type rows.
            NpcKind::SeekerHead => 98,
            // W4-d35 (E): the 101/102/103/104/109 vanilla NPC.type rows.
            NpcKind::Clinger => 101,
            NpcKind::AnglerFish => 102,
            NpcKind::Werewolf => 104,
            NpcKind::Clown => 109,
            // W4-d35 (F): the 110/116/120/121 vanilla NPC.type rows.
            NpcKind::SkeletonArcher => 110,
            NpcKind::TheHungryII => 116,
            NpcKind::ChaosElemental => 120,
            // W4-d35 (G): the 122/137/140 vanilla NPC.type rows.
            NpcKind::PossessedArmor => 140,
            NpcKind::BlazingWheel => 72,
            NpcKind::AngryNimbus => 250,
            NpcKind::DungeonSpirit => 288,
            NpcKind::ForceBubble => 384,
            // W4-d34 kinds 1-70: the vanilla `NPC.type` rows (the AI arms'
            // type gates + the statue caps read these, not the protocol ids).
            NpcKind::AngryBones => 31,
            NpcKind::OldMan => 37,
            NpcKind::ManEater => 43,
            NpcKind::Bunny => 46,
            NpcKind::BoneSerpentHead => 39,
            // The Bone Serpent's body/tail keep NO hand row — like every
            // folded segment family they fall through to the bestiary net id
            // (the head's 39; the s1_f_3 cs_id==net_id invariant, the
            // Destroyer precedent).
            NpcKind::CorruptBunny => 47,
            NpcKind::Snatcher => 56,
            NpcKind::CorruptGoldfish => 57,
            NpcKind::Crab => 67,
            NpcKind::DungeonGuardian => 68,
            // W4-c3 (the style-10 family): the vanilla `NPC.type` ids
            // (`NPCID.cs:11255`/`:12065`); the CursedSkull's 34 row is
            // d34's above. The GiantCursedSkull 289 row rides here (the
            // superset-module rule - d38 carries no cs_id arm for it).
            NpcKind::GiantCursedSkull => 289,
            NpcKind::WaterBoltMimic => 694,
            // W4-c4 (the style-44 swimmers): the vanilla `NPC.type` ids
            // (`NPCID.cs:11125`/`:11695`/`:11839`/`:11851`). The
            // FlyingFish's protocol id is port-local 359 (d37's
            // frozen-band row adopted; the original 354 retired), so its
            // C#-type consumers (the style-44 arm's type gates + the
            // world-sim pre-pass's sibling scan) read 224 here.
            NpcKind::FlyingFish => 224,
            NpcKind::GiantFlyingAntlion => 509,
            NpcKind::FlyingAntlion => 581,
            NpcKind::EyeballFlyingFish => 587,
            // W4-c7 — the firefly family's vanilla `NPC.type` ids (the AI
            // arm's per-type gates read them): `NPCID.cs:11387`/`:11393`/
            // `:11985`/`:12031`.
            NpcKind::Firefly => 355,
            NpcKind::LightningBug => 358,
            NpcKind::Lavafly => 654,
            NpcKind::Shimmerfly => 677,
            // W4-c10 — the waterfowl family's own bestiary credits (the
            // populator registers all eight: `FindEntryByNPCID(362/363/
            // 364/365/602/603/608/609)`, `BestiaryDatabaseNPCsPopulator.cs:
            // 1191-1266`) — no fold needed, the vanilla ids ride.
            NpcKind::Duck => 362,
            NpcKind::Duck2 => 363,
            NpcKind::DuckWhite => 364,
            NpcKind::DuckWhite2 => 365,
            // E8 merge: Seagull/Grebe's cs_id rows are d42's above.
            // E8 merge: the c10 12-critter cs_id tail dropped (d40/c8's rows).
            // W4-d25 STYLE-26 wiring: the vanilla `NPC.type` ids
            // (`NPCID.cs:10849`/`:10987`/`:11769`) — the machine's type
            // gates and the spawn rolls speak these, NOT the protocol ids
            // (440/355; the Tumbleweed's 546 IS its protocol id).
            NpcKind::Wolf => 155,
            NpcKind::Tumbleweed => 546,
            // `NPCID.DesertDjinn` (533)
            NpcKind::DesertDjinn => 533,
            // W4-d40: hand rows for the kinds with no bestiary entry
            // (the Hide=true exclusions with no credit-fold).
            NpcKind::BoneThrowingSkeleton2 => 450,
            NpcKind::BoneThrowingSkeleton3 => 451,
            // W4-d40: hand rows for the kinds with no bestiary entry
            // (the Hide=true exclusions with no credit-fold).
            NpcKind::BoneThrowingSkeleton4 => 452,
            // W4-d40: hand rows for the kinds with no bestiary entry
            // (the Hide=true exclusions with no credit-fold).
            NpcKind::TargetDummy => 488,
            // W4-d41 — the cavern crawdads/shellies' vanilla types
            // (`NPCID.cs:11661-11667`). The 1-variants keep hand rows; the
            // 2-variants have NO hand row — like every folded segment
            // family (the Destroyer body/tail precedent, the L-B gate fix)
            // they fall through to the folded bestiary net id (494/496),
            // keeping s1_f_3's cs_id==net_id invariant. The C# type gates
            // still work: the ladder's charge family matches 494|495
            // identically, and the shelly AI reads `Style39Type`, not
            // cs_id. The REAL draw types ride render_key/texture_net_id.
            NpcKind::Crawdad => 494,
            NpcKind::GiantShelly => 496,
            // W4-d41 — the salamanders: 498 keeps its hand row; 499-506
            // have NO hand row — they fold onto 499's bestiary net id
            // (`ContentSamples.cs:1021-1025` maps ALL of 498-506 to the
            // 499 credit), keeping s1_f_3's cs_id==net_id invariant.
            // The shooter block's type gates read the 498..=506 RANGE,
            // which 499 satisfies identically; the REAL draw types ride
            // render_key/texture_net_id.
            NpcKind::Salamander => 498,
            // W4-d41 — the antlions keep their vanilla types
            // (`NPCID.cs:11687-11689`); the Dune Splicer head keeps its
            // own (510). The body/tail have NO hand row — they fold onto
            // the head's bestiary net id (the worm-family precedent,
            // s1_f_3's cs_id==net_id invariant); the chain's type gates
            // read the KIND, and the REAL draw types ride
            // render_key/texture_net_id.
            NpcKind::GiantWalkingAntlion => 508,
            NpcKind::DuneSplicerHead => 510,
            // W4-d41 — the Tomb Crawler head keeps its own (513); the
            // body/tail fold onto the head's bestiary net id (the
            // worm-family precedent). The solar fragments keep their
            // own (516/519 — Hide=true, no bestiary entry, so cs_id
            // falls through to the sentinel unless hand-rowed; the
            // hand rows keep the C# type gates readable).
            NpcKind::TombCrawlerHead => 513,
            NpcKind::SolarGoop => 519,
            // W4-d41 — the ancient orbs + the desert ghouls keep their
            // vanilla types (`NPCID.cs:11721-11729`). The orbs are
            // Hide=true (no bestiary entry), so the hand rows keep the
            // C# type gates readable; the ghouls carry their own entries.
            NpcKind::AncientLight => 522,
            NpcKind::AncientDoom => 523,
            NpcKind::DesertGhoul => 524,
            NpcKind::DesertGhoulCorruption => 525,
            NpcKind::DesertGhoulCrimson => 526,
            // W4-d41 — the hallowed ghoul + the lamias keep their vanilla
            // types (`NPCID.cs:11731-11735`); the sand poacher walker
            // keeps its own (530). The wall form (531) folds onto the
            // walker's bestiary net id (`ContentSamples.cs:1038`) — NO
            // hand row (the worm-family precedent, s1_f_3's
            // cs_id==net_id invariant); the wall-crawler's type gates
            // read the KIND, and the REAL draw type rides
            // render_key/texture_net_id.
            NpcKind::DesertGhoulHallow => 527,
            NpcKind::DesertLamiaLight => 528,
            NpcKind::DesertLamiaDark => 529,
            NpcKind::DesertScorpionWalk => 530,
            // W4-d41 — the basilisk + the tortured soul + the slimes +
            // the bride keep their vanilla types (`NPCID.cs:11741-11751`).
            NpcKind::DesertBeast => 532,
            NpcKind::DemonTaxCollector => 534,
            NpcKind::SlimeSpiked => 535,
            NpcKind::TheBride => 536,
            NpcKind::SandSlime => 537,
            // W4-d41 — the critters + the tumbler keep their vanilla
            // types (`NPCID.cs:11753-11769`).
            NpcKind::SquirrelRed => 538,
            NpcKind::SquirrelGold => 539,
            NpcKind::Goldfish => 55,
            NpcKind::Piranha => 58,
            NpcKind::Shark => 65,
            // W4-c4 (the style-16 swimmers): the vanilla `NPC.type` ids
            // (`NPCID.cs:10791`/`:10991`/`:11159`/`:11603`/`:11861`/
            // `:11891`/`:11907`/`:12053`/`:12061`). The CorruptGoldfish/
            // Arapaima protocol ids are port-local 405/406 (the free-port
            // band; the original 229/237 reserved-hole squat retired),
            // so the style-16 arm's type gates read 57/157 here.
            // E8 merge drops: CorruptGoldfish (d34's 57 row above),
            // BloodFeeder (d37's 241 - the ENTRIES fallback resolves),
            // Pufferfish/Orca (d43 lands later).
            NpcKind::Arapaima => 157,
            NpcKind::CrimsonGoldfish => 465,
            NpcKind::GoldGoldfish => 592,
            NpcKind::Pupfish => 607,
            NpcKind::Dolphin => 615,
            // W4-e21 — the bunny pair's vanilla `NPC.type` (`NPCID.cs:10769`/
            // `:11757`): the statue drain's `NPC.MechSpawn` group reads these
            // (the bunny group {46, 540, 303, 337}, `NPC.cs:7443-7449`).
            NpcKind::PartyBunny => 540,
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
            // W4-d37 — the NPCID 211-280 band's three Hide=true bestiary
            // folds keep their OWN vanilla `NPC.type` here (the AI ladder's
            // type gates read it — the BlazingWheel precedent: protocol 353,
            // cs_id 72). The bestiary CREDIT folds onto the visible twin
            // (`entry_for_npc`), but `cs_id` is the C# type, not the credit.
            NpcKind::GoldfishWalker => 230,
            NpcKind::JungleCreeper => 236,
            NpcKind::BloodCrawler => 239,
            // W4-d42 - the roster's TRUE vanilla types (the folded kinds
            // keep their own - the AI arms read cs_id; the bestiary
            // folds in entry_for_npc only).
            NpcKind::DD2LightningBug => 578,
            NpcKind::BartenderUnconscious => 579,
            NpcKind::WalkingAntlion => 580,
            NpcKind::LarvaeAntlion => 582,
            // W4-d42 - the roster's TRUE vanilla types (the folded kinds
            // keep their own - the AI arms read cs_id; the bestiary
            // folds in entry_for_npc only).
            NpcKind::ZombieMerman => 586,
            // W4-d42 - the roster's TRUE vanilla types (the folded kinds
            // keep their own - the AI arms read cs_id; the bestiary
            // folds in entry_for_npc only).
            NpcKind::TorchZombie => 590,
            NpcKind::ArmedTorchZombie => 591,
            NpcKind::GoldGoldfishWalker => 593,
            // W4-d42 - the roster's TRUE vanilla types (the folded kinds
            // keep their own - the AI arms read cs_id; the bestiary
            // folds in entry_for_npc only).
            // W4-d42 - the roster's TRUE vanilla types (the folded kinds
            // keep their own - the AI arms read cs_id; the bestiary
            // folds in entry_for_npc only).
            NpcKind::Seagull => 602,
            NpcKind::Seagull2 => 603,
            // W4-d42 - the roster's TRUE vanilla types (the folded kinds
            // keep their own - the AI arms read cs_id; the bestiary
            // folds in entry_for_npc only).
            NpcKind::Grebe => 608,
            NpcKind::Grebe2 => 609,
            // W4-d42 - the roster's TRUE vanilla types (the folded kinds
            // keep their own - the AI arms read cs_id; the bestiary
            // folds in entry_for_npc only).
            NpcKind::Rat => 610,
            NpcKind::Owl => 611,
            NpcKind::WaterStrider => 612,
            NpcKind::GoldWaterStrider => 613,
            NpcKind::ExplosiveBunny => 614,
            // W4-d42 - the roster's TRUE vanilla types (the folded kinds
            // keep their own - the AI arms read cs_id; the bestiary
            // folds in entry_for_npc only).
            NpcKind::Turtle => 616,
            NpcKind::TurtleJungle => 617,
            NpcKind::BloodNautilus => 618,
            NpcKind::BloodSquid => 619,
            // The Shimmerfly (W4-c9's rowing, adopted by W4-g62): the
            // vanilla `NPC.type` id — the AI arm's type gate (the 677
            // threat scan) and the catch/convert tables read it
            // (`NPCID.cs:12031`). It has its own bestiary entry (the
            // populator's `FindEntryByNPCID(677)` info row), so no fold
            // is needed.
            // W4-d42 - the roster's TRUE vanilla types (the folded kinds
            // keep their own - the AI arms read cs_id; the bestiary
            // folds in entry_for_npc only).
            NpcKind::GoblinShark => 620,
            NpcKind::BloodEelHead => 621,
            NpcKind::BloodEelBody => 622,
            NpcKind::BloodEelTail => 623,
            NpcKind::Gnome => 624,
            // W4-d42 - the roster's TRUE vanilla types (the folded kinds
            // keep their own - the AI arms read cs_id; the bestiary
            // folds in entry_for_npc only).
            NpcKind::SeaTurtle => 625,
            NpcKind::Seahorse => 626,
            NpcKind::GoldSeahorse => 627,
            NpcKind::Dandelion => 628,
            NpcKind::IceMimic => 629,
            // W4-d42 - the roster's TRUE vanilla types (the folded kinds
            // keep their own - the AI arms read cs_id; the bestiary
            // folds in entry_for_npc only).
            NpcKind::BloodMummy => 630,
            // W4-c8 — the crawling-critter family's vanilla `NPC.type`
            // ids (the AI arm's per-type gates read them):
            // `NPCID.cs:11391`/`:11425`/`:11427`/`:11569`/`:11641-11647`/
            // `:11889`. The digger 375 keeps its OWN hand row — it is
            // Hide=true in the bestiary (`NPCID.cs:3714-3718`, folds onto
            // the TruffleWorm in `entry_for_npc`) but NOT `npcCatchable`
            // (`Main.cs:9326-9418` lists 374, not 375), so cs_id must stay
            // 375 to keep `friendly`/immunity false (the ForceBubble
            // precedent — own cs_id, folded net_id).
            // E8 merge: the trio keeps its vanilla cs hand rows (the
            // protocol ids renumbered, the C#-type gates unchanged); the
            // 5 d40-owned dups dropped (d40's rows/fallback above).
            NpcKind::Worm => 357,
            NpcKind::TruffleWorm => 374,
            NpcKind::TruffleWormDigger => 375,
            NpcKind::Maggot => 606,
            // The snail family + its shimmer target (W4-c9): the vanilla
            // `NPC.type` ids — the AI arms' type gates (the 359 scale
            // roll, the 360/655 lights, the 677 threat scan) and the
            // catch/convert tables read them (`NPCID.cs:11395`/`:11397`/
            // `:11987`/`:12031`). All four have their own bestiary
            // entries (the populator's `FindEntryByNPCID(359)`/`(677)`
            // info rows), so no fold is needed.
            // E8 merge: the snail trio keeps its vanilla cs hand rows;
            // Shimmerfly's 677 is c7's above; the 5 gold rows are d40's
            // above (the dup drops).
            NpcKind::Snail => 359,
            NpcKind::GlowingSnail => 360,
            NpcKind::MagmaSnail => 655,
            // W4-c6 style-59 wiring — NO hand row for the blade: its
            // bestiary entry is Hide=true (`NPCID.cs:3398-3403`'s draw
            // modifiers) and folds onto the Pumpking in
            // `try_entry_for_npc`, so the net-id fallback reads the folded
            // 327 — the s1_f_3 cs_id==net_id invariant (the fishron
            // hazards' L-B gate fix, the destroyer precedent). The vanilla
            // type 328 rides `to_u16`/`texture_net_id`; the machine reads
            // its own kind (the aiStyle-59 axis).
            // W4-g8 lane 1 batch 1 — the vanilla `NPC.type` ids the AI
            // machines' type gates read (`NPCID.cs:10799`/`:10803`/
            // `:10805`/`:10815`/`:10817`/`:10827`/`:10841`/`:10843`/
            // `:10845`/`:10847`). The protocol ids are port-local — E8
            // moved the owned kinds to 745-761 (the dups stay drop-pending).
            NpcKind::PinkJellyfish => 64,
            NpcKind::Antlion => 69,
            NpcKind::SpikeBall => 70,
            // W4-g8 lane 1 batch 2 — the vanilla `NPC.type` ids
            // (`NPCID.cs:10883`/`:10921`/`:11005`/`:11007`/`:11015`/
            // `:11025`/`:11031`/`:11035`/`:11041`/`:11067`/`:11069`).
            NpcKind::WallCreeper => 164,
            NpcKind::WallCreeperWall => 165,
            NpcKind::IceElemental => 169,
            NpcKind::Herpling => 174,
            NpcKind::Derpling => 177,
            NpcKind::CrimsonAxe => 179,
            NpcKind::FloatyGross => 182,
            NpcKind::LostGirl => 195,
            NpcKind::Nymph => 196,
            // W4-g8 lane 1 batch 3 — the vanilla `NPC.type` ids
            // (`NPCID.cs:10745`/`:10801`/`:10863`/`:10919`/`:10951`/
            // `:10977`/`:10979`/`:10981`/`:10993`/`:11129`).
            // W4-g8 lane 1 batch 4 — the vanilla `NPC.type` ids
            // (`NPCID.cs:11119`/`:10773`/`:10987`).
            // W4-g8 machine round 3 — the Vampire 159 (`NPCID.cs:10995`).
            // The dragonflies (W4-d33): the vanilla `NPC.type` ids — the
            // bug-net catch tables (`bug_net::catch_item`'s 595-601 row),
            // the AI arms' type gates and the statue rosters read them
            // (`NPCID.cs:11867-11875`). The four Hide=true colours fold
            // their bestiary CREDIT onto 599 (`entry_for_npc`), but each
            // keeps its own type here — the e26 ladybug / c8 crawler
            // precedent.
            NpcKind::BlackDragonfly => 595,
            NpcKind::BlueDragonfly => 596,
            NpcKind::GreenDragonfly => 597,
            NpcKind::OrangeDragonfly => 598,
            NpcKind::RedDragonfly => 599,
            NpcKind::YellowDragonfly => 600,
            NpcKind::GoldDragonfly => 601,
            // The fairy critters + the windy balloon (W4-d33 wiring): the
            // TRUE vanilla types — the bug-net catch tables
            // (`bug_net::catch_item`'s 583/584/585 rows), the AI arms' type
            // gates and the release chain read them (`NPCID.cs:11843-11847`
            // / `:11865`). None of the four folds (all three fairies are
            // VISIBLE cards — the `NPCID.cs:5148-5169` draw modifiers carry
            // Position/Direction, no Hide; the balloon has its own card
            // with a CustomTexturePath, `:5232-5239`). d42 batch-2/3's
            // hand rows, constants-copy (keep-one d37c-E3).
            NpcKind::FairyCritterPink => 583,
            NpcKind::FairyCritterGreen => 584,
            NpcKind::FairyCritterBlue => 585,
            NpcKind::WindyBalloon => 594,
            // The luck critters (W4-e26, E8 keep-one): the vanilla
            // `NPC.type` ids — the AI arms' type gates (the 115 flight,
            // the 604/605 luck windows' gold flag) and the catch/convert
            // tables read them (`NPCID.cs:11399`/`:11885`/`:11887`).
            // All three have their own bestiary entries (the populator's
            // `CountsAsACritter` automated rows), so no fold. The Frog's
            // cs_id stays 361 (only the wire id moved to 744); the Duck
            // dropped to c10 (the waterfowl owner, SEAL S1).
            NpcKind::Frog => 361,
            NpcKind::LadyBug => 604,
            NpcKind::GoldLadyBug => 605,
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

    /// `NPCID.Sets.TrailCacheLength` (`NPCID.cs:10450`), the FULL
    /// table: `CreateIntSet(10, 402, 36, 519, 20, 522, 20, 620, 20,
    /// 677, 60, 685, 10)` — default 10; only four kinds deviate
    /// (685's explicit 10 IS the default; the max is 677's 60).
    /// `SetDefaults` reads it at `NPC.cs:8456` and resizes
    /// `oldPos`/`oldRot` (`:8458-8462`), then zeroes the whole array
    /// (`:8463-8468`). The port keeps max-size `[Vec2; 60]` storage
    /// and this logical length (`Npc.trail_len`). W4-g62 follow-up:
    /// the 677 Shimmerfly (c9's kind) and the 620 GoblinShark (d42's
    /// kind) are ROWED here — 677→60 (the MAX) and 620→20 — and their
    /// `update_trails` mode-7 gates widen the same day. Kinds not yet
    /// rowed (519 SolarGoop / 522 AncientLight / 685 BoundTownSlimeOld)
    /// fall through to 10 — their wiring lanes add the row here when
    /// the KIND lands (522 is NOT rowed by d42 either — its 561-630
    /// band excludes the pillar mob; the cultist lane's).
    pub fn trail_cache_length(self) -> usize {
        match self {
            // 402 — the Stardust Worm Head, 36 slots (`:10450`).
            NpcKind::StardustWormHead => 36,
            // 677 — the Shimmerfly, 60 slots: the census MAX (`:10450`).
            // 620 — the GoblinShark, 20 slots (`:10450`).
            _ => 10,
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
            | NpcKind::PartyGirl
            // W4-d120 — the Truffle/Steampunker's 23-row strips
            // (`Main.npcFrameCount[160/178]`, `Main.cs:67082`).
            | NpcKind::Truffle
            | NpcKind::Steampunker => 23,
            NpcKind::Dryad => 21,
            // W4-d120 — Santa's 25 / the Cyborg's 26 (`Main.cs:67082`).
            NpcKind::SantaClaus => 25,
            NpcKind::Cyborg => 26,
            NpcKind::GoblinArcher => 21,
            // W4-d37 — the Pirate (229) rides the humanoid-group arm
            // (`NPC.cs:68590`'s case list) on the 26-row town strip
            // (`Main.npcFrameCount[229]` = 26, the Guide's shape).
            NpcKind::Pirate => 26,
            NpcKind::GoblinPeon | NpcKind::GoblinThief | NpcKind::GoblinWarrior => 16,
            NpcKind::Skeleton | NpcKind::UndeadMiner => 15,
            // W4-d34: the Angry Bones (31) rides the humanoid-group case list
            // (`NPC.cs:68552`) on its 15-row strip; the Old Man (37) on 21.
            NpcKind::AngryBones => 15,
            NpcKind::OldMan => 21,
            // The Vampire (159, W4-d23) rides `FindFrame`'s humanoid-group
            // case list (`NPC.cs:68569`) on its 15-frame strip
            // (`Main.npcFrameCount[159]` = 15, `Main.cs:67082`); the C#'s
            // `ExtraFramesCount` set carries no 159 (`NPCID.cs:10464`),
            // so the walk band wraps at 15.
            NpcKind::Vampire => 15,
            // The Zombie Elf trio (`S1-F-13c`): `FindFrame`'s humanoid-group
            // case list carries 338-340 (`NPC.cs:68603-68605`) on their
            // 15-frame strips (`Main.npcFrameCount[338..340]`, `Main.cs`).
            NpcKind::ZombieElf | NpcKind::ZombieElfBeard | NpcKind::ZombieElfGirl => 15,
            // W4-d38 b6 — humanoid-group FindFrame strips.
            NpcKind::SkeletonTopHat => 16,
            NpcKind::SkeletonAstonaut => 15,
            NpcKind::SkeletonAlien => 16,
            // W4-d38 b3 — humanoid-group FindFrame strips.
            NpcKind::AngryBonesBig => 16,
            NpcKind::AngryBonesBigMuscle => 15,
            NpcKind::AngryBonesBigHelmet => 15,
            // W4-d38 b2 — humanoid-group FindFrame strips.
            NpcKind::BoneLee => 16,
            // W4-d42 - the humanoid-group case list carries 630
            // (`NPC.cs:68618`), 16 rows (`Main.npcFrameCount[630]`).
            NpcKind::BloodMummy => 16,
            // W4-g8 lane 1 batch 2 — the Nymph 196 rides the humanoid-group
            // case list (`NPC.cs:68576`) on its 14-frame strip
            // (`Main.npcFrameCount[196]` = 14).
            NpcKind::Nymph => 14,
            // W4-g8 machine round 3 — the Vampire 159 rides the
            // humanoid-group case list (`NPC.cs:68569`) on its 15-frame
            // strip (`Main.npcFrameCount[159]` = 15, `Main.cs:67100`).
            // W4-f42 — the mummies' 16-frame humanoid strips
            // (`Main.npcFrameCount[78/79/80/630]` = 16, `Main.cs:67082`).
            // W4-g10 lane 3 — the town pets' `Main.npcFrameCount` rows:
            // 637/638 = 28, 656 = 27, the town slimes 670/678-684 = 14
            // (`Main.cs`'s npcFrameCount init; `FindFrame`'s pet branches
            // `NPC.cs:68610-69655` wrap the walk band on these strips).
            NpcKind::TownCat | NpcKind::TownDog => 28,
            NpcKind::TownBunny => 27,
            NpcKind::TownSlimeBlue
            | NpcKind::TownSlimeGreen
            | NpcKind::TownSlimeOld
            | NpcKind::TownSlimePurple
            | NpcKind::TownSlimeRainbow
            | NpcKind::TownSlimeRed
            | NpcKind::TownSlimeYellow
            | NpcKind::TownSlimeCopper => 14,
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
            | NpcKind::Princess
            // W4-d120 — the Truffle's 7 attack rows (`NPCID.cs:10464`).
            | NpcKind::Truffle => 7,
            NpcKind::Angler | NpcKind::Mechanic => 9,
            // W4-d34: the Old Man's `ExtraFramesCount[37] = 5` (`NPCID.cs:10464`).
            NpcKind::OldMan => 5,
            // W4-d37 — the Pirate (229) reserves 10 attack rows
            // (`NPCID.cs:10464`'s ExtraFramesCount set: `229, 10`).
            NpcKind::Pirate => 10,
            // W4-d120 — Santa/Steampunker 9, the Cyborg 10
            // (`NPCID.cs:10464`'s ExtraFramesCount rows).
            NpcKind::SantaClaus | NpcKind::Steampunker => 9,
            NpcKind::Cyborg => 10,
            // W4-g10 lane 3 — the town pets' `ExtraFramesCount` rows
            // (`NPCID.cs:10464`): 637 = 18, 638 = 11, 656 = 20, the town
            // slimes = 6 — the reserved bottom rows the pet idle poses
            // (`NPC.cs:68680-68917`'s state-20/21/22 arms) animate through.
            NpcKind::TownCat => 18,
            NpcKind::TownDog => 11,
            NpcKind::TownBunny => 20,
            NpcKind::TownSlimeBlue
            | NpcKind::TownSlimeGreen
            | NpcKind::TownSlimeOld
            | NpcKind::TownSlimePurple
            | NpcKind::TownSlimeRainbow
            | NpcKind::TownSlimeRed
            | NpcKind::TownSlimeYellow
            | NpcKind::TownSlimeCopper => 6,
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
                // W4-d35 (C): the 95 behindTiles rows.
                | NpcKind::DiggerHead
                // W4-d35 (D): the 96/97/98/99/100 behindTiles rows.
                | NpcKind::DiggerBody
                | NpcKind::DiggerTail
                | NpcKind::SeekerHead
                | NpcKind::SeekerBody
                | NpcKind::SeekerTail
                // W4-d35 (E): the 101 behindTiles rows.
                | NpcKind::Clinger
                | NpcKind::BlazingWheel
                // W4-d34: the Bone Serpent's `behindTiles = true` rows
                // (`NPC.cs:9240`/`:9257`/`:9275`).
                | NpcKind::BoneSerpentHead
                | NpcKind::BoneSerpentBody
                | NpcKind::BoneSerpentTail
                // W4-d42 - the Blood Eel's behindTiles rows (`NPC.cs:16983` etc.).
                | NpcKind::BloodEelHead
                | NpcKind::BloodEelBody
                | NpcKind::BloodEelTail
                // W4-c8 — the TruffleWormDigger's `behindTiles = true`
                // row (`NPC.cs:13790`): the digger draws under the tiles.
                | NpcKind::TruffleWormDigger
                // The Clinger's `behindTiles = true` row (`NPC.cs:10190`,
                // W4-d23 — the family's only behind-tiles member).
                // W4-g8 lane 1 batch 1 — the Antlion's `behindTiles = true`
                // row (`NPC.cs:9678`); it fires from behind the sand.
                // W4-f42 — the Antlion's `behindTiles = true` row
                // (`NPC.cs:9678`).
                | NpcKind::Antlion
                // W4-d41: the lane portal's `behindTiles = true` row
                // (`NPC.cs:16048`).
                | NpcKind::DD2LanePortal
                // W4-d41 — the Dune Splicer chain's `behindTiles = true`
                // rows (`NPC.cs:14787`/`:14804`/`:14823`).
                | NpcKind::DuneSplicerHead
                | NpcKind::DuneSplicerBody
                | NpcKind::DuneSplicerTail
                // W4-d41 — the Tomb Crawler chain's `behindTiles = true`
                // rows (`NPC.cs:14732`/`:14749`/`:14768`).
                | NpcKind::TombCrawlerHead
                | NpcKind::TombCrawlerBody
                | NpcKind::TombCrawlerTail
                // W4-f18 batch 3 — the TruffleWormDigger 375's
                // `behindTiles = true` row (`NPC.cs:13785`).
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
            // The aiStyle-14 flyer family (W4-d23): 48 100 (`NPC.cs:9367`),
            // 62 120 (`:9575`), 93 100 (`:10055`), 121 60 (`:10473`),
            // 137 200 (`:10742`), 150 30 (`:10893`), 151 160 (`:10908`),
            // 152 220 (`:10923`), 156 600 (`:10982`), 158/159 750
            // (`:15474`/`:15488`), 226 260 (`:11919`), 634 16 (`:17169`),
            // 660 120 (`:17345`).
            NpcKind::Harpy | NpcKind::GiantBat => 100,
            NpcKind::Demon | NpcKind::QueenSlimeMinionPurple => 120,
            NpcKind::IceBat => 30,
            NpcKind::Lavabat => 160,
            NpcKind::GiantFlyingFox => 220,
            NpcKind::RedDevil => 600,
            NpcKind::VampireBat | NpcKind::Vampire => 750,
            NpcKind::SporeBat => 16,
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
            // W4-c4 (the style-16 swimmers): 57 100 (`NPC.cs:9497`), 157
            // 200 (`:10998`), 241 150 (`:12114`), 465 110 (`:14491`),
            // 592/607/615/688 5 (`:16687`/`:16802`/`:16900`/`:17661`),
            // 692 400 (`:17717`).
            // E8 merge drops: CorruptGoldfish/BloodFeeder (d34/d37 rows
            // above), Pufferfish/Orca (d43 lands later).
            NpcKind::Arapaima => 200,
            NpcKind::CrimsonGoldfish => 110,
            NpcKind::GoldGoldfish | NpcKind::Pupfish | NpcKind::Dolphin => 5,
            // W4-e21 — the bunny pair: lifeMax 5 (`NPC.cs:9343`).
            NpcKind::Bunny | NpcKind::PartyBunny => 5,
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
            // W4-c6 style-59 wiring: the blade's `lifeMax = 5000`
            // (`NPC.cs:13165`).
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
            // W4-d40: the SetDefaults lifeMax rows (`NPC.cs`).
            NpcKind::ArmedZombie => 45,
            NpcKind::ArmedZombieEskimo => 50,
            NpcKind::ArmedZombiePincussion => 50,
            NpcKind::ArmedZombieSlimed => 40,
            NpcKind::ArmedZombieSwamp => 45,
            // W4-d40: the SetDefaults lifeMax rows (`NPC.cs`).
            NpcKind::ArmedZombieTwiggy => 45,
            NpcKind::ArmedZombieCenx => 38,
            NpcKind::CultistTablet => 400,
            NpcKind::CultistDevote => 400,
            NpcKind::CultistBossClone => 10000,
            // W4-d40: the SetDefaults lifeMax rows (`NPC.cs`).
            NpcKind::GoldBird => 5,
            NpcKind::GoldBunny => 5,
            NpcKind::GoldButterfly => 5,
            NpcKind::GoldFrog => 5,
            NpcKind::GoldGrasshopper => 5,
            // W4-d40: the SetDefaults lifeMax rows (`NPC.cs`).
            NpcKind::GoldMouse => 5,
            NpcKind::GoldWorm => 5,
            NpcKind::BoneThrowingSkeleton => 60,
            NpcKind::BoneThrowingSkeleton2 => 55,
            NpcKind::BoneThrowingSkeleton3 => 65,
            // W4-d40: the SetDefaults lifeMax rows (`NPC.cs`).
            NpcKind::BoneThrowingSkeleton4 => 60,
            NpcKind::SkeletonMerchant => 250,
            NpcKind::CultistDragonHead => 10000,
            NpcKind::CultistDragonBody1 => 10000,
            NpcKind::CultistDragonBody2 => 10000,
            // W4-d40: the SetDefaults lifeMax rows (`NPC.cs`).
            NpcKind::CultistDragonBody3 => 10000,
            NpcKind::CultistDragonBody4 => 10000,
            NpcKind::CultistDragonTail => 10000,
            NpcKind::Butcher => 700,
            NpcKind::CreatureFromTheDeep => 400,
            // W4-d40: the SetDefaults lifeMax rows (`NPC.cs`).
            NpcKind::Fritz => 270,
            NpcKind::Nailhead => 4000,
            NpcKind::CrimsonBunny => 75,
            NpcKind::Psycho => 550,
            // W4-d40: the SetDefaults lifeMax rows (`NPC.cs`).
            NpcKind::DeadlySphere => 350,
            NpcKind::DrManFly => 500,
            NpcKind::ThePossessed => 600,
            NpcKind::CrimsonPenguin => 75,
            NpcKind::ShadowFlameApparition => 180,
            // W4-d40: the SetDefaults lifeMax rows (`NPC.cs`).
            NpcKind::BigMimicCorruption => 3500,
            NpcKind::BigMimicCrimson => 3500,
            NpcKind::BigMimicHallow => 3500,
            NpcKind::BigMimicJungle => 3500,
            NpcKind::Mothron => 6000,
            // W4-d40: the SetDefaults lifeMax rows (`NPC.cs`).
            NpcKind::MothronEgg => 200,
            NpcKind::MothronSpawn => 700,
            NpcKind::Medusa => 400,
            NpcKind::GreekSkeleton => 70,
            NpcKind::GraniteGolem => 110,
            // W4-d40: the SetDefaults lifeMax rows (`NPC.cs`).
            NpcKind::GraniteFlyer => 40,
            NpcKind::EnchantedNightcrawler => 5,
            NpcKind::Grubby => 5,
            NpcKind::Sluggy => 5,
            NpcKind::Buggy => 5,
            // W4-d40: the SetDefaults lifeMax rows (`NPC.cs`).
            NpcKind::TargetDummy => 1000,
            // W4-d35 (A): the 73/74/75/77/78 lifeMax rows.
            NpcKind::GoblinScout => 80,
            NpcKind::Bird => 5,
            NpcKind::ArmoredSkeleton => 260,
            NpcKind::Mummy => 130,
            // W4-d35 (B): the 79/80/82/83/84 lifeMax rows.
            NpcKind::DarkMummy => 180,
            NpcKind::LightMummy => 200,
            // W4-d35 (C): the 85/86/93/94/95 lifeMax rows.
            NpcKind::Unicorn => 400,
            NpcKind::Corruptor => 230,
            NpcKind::DiggerHead => 200,
            // W4-d35 (D): the 96/97/98/99/100 lifeMax rows.
            NpcKind::DiggerBody => 200,
            NpcKind::DiggerTail => 200,
            NpcKind::SeekerHead => 500,
            NpcKind::SeekerBody => 500,
            NpcKind::SeekerTail => 500,
            // W4-d35 (E): the 101/102/103/104/109 lifeMax rows.
            NpcKind::Clinger => 320,
            NpcKind::AnglerFish => 90,
            NpcKind::Werewolf => 350,
            NpcKind::Clown => 800,
            // W4-d35 (F): the 110/112/116/120/121 lifeMax rows.
            NpcKind::SkeletonArcher => 210,
            NpcKind::VileSpit => 1,
            NpcKind::TheHungryII => 80,
            NpcKind::ChaosElemental => 370,
            // W4-d35 (G): the 122/137/140 lifeMax rows.
            NpcKind::PossessedArmor => 260,
            NpcKind::DungeonSpirit => 200,
            // W4-d34 kinds 1-70 (`NPC.cs` SetDefaults lifeMax).
            NpcKind::AngryBones => 80,   // :9115
            NpcKind::OldMan => 250,      // :9206
            NpcKind::ManEater => 110,    // :9302
            NpcKind::Bunny => 5,         // :9343
            NpcKind::BoneSerpentHead | NpcKind::BoneSerpentBody | NpcKind::BoneSerpentTail => 300, // :9234/:9251/:9269
            NpcKind::CorruptBunny => 70,   // :9355
            NpcKind::Snatcher => 60,          // :9483
            NpcKind::CorruptGoldfish => 100,  // :9497
            NpcKind::Crab => 40,              // :9647
            NpcKind::DungeonGuardian => 9999, // :9659
            // W4-d38 b7 — SetDefaults lifeMax rows.
            NpcKind::SlimeRibbonWhite => 25,
            NpcKind::SlimeRibbonYellow => 23,
            NpcKind::SlimeRibbonGreen => 29,
            NpcKind::SlimeRibbonRed => 22,
            NpcKind::BunnyXmas => 5,
            // W4-d38 b6 — SetDefaults lifeMax rows.
            NpcKind::DemonEyeSpaceship => 60,
            NpcKind::SkeletonTopHat => 115,
            NpcKind::SkeletonAstonaut => 65,
            NpcKind::SkeletonAlien => 70,
            NpcKind::PumpkingBlade => 5000,
            // W4-d38 b5 — SetDefaults lifeMax rows.
            NpcKind::SlimeMasked => 25,
            NpcKind::BunnySlimed => 5,
            NpcKind::HoppinJack => 175,
            NpcKind::Ghost => 50,
            NpcKind::DemonEyeOwl => 75,
            // W4-d38 b4 — SetDefaults lifeMax rows.
            NpcKind::BirdBlue => 5,
            NpcKind::BirdRed => 5,
            NpcKind::Squirrel => 5,
            NpcKind::Mouse => 5,
            NpcKind::Raven => 35,
            // W4-d38 b3 — SetDefaults lifeMax rows.
            NpcKind::TacticalSkeleton => 400,
            NpcKind::SkeletonCommando => 400,
            NpcKind::AngryBonesBig => 70,
            NpcKind::AngryBonesBigMuscle => 70,
            NpcKind::AngryBonesBigHelmet => 120,
            // W4-d38 b2 — SetDefaults lifeMax rows.
            NpcKind::DiabolistWhite => 250,
            NpcKind::BoneLee => 1000,
            NpcKind::GiantCursedSkull => 400,
            NpcKind::Paladin => 5000,
            NpcKind::SkeletonSniper => 400,
            // W4-d38 b1 — SetDefaults lifeMax rows.
            NpcKind::RaggedCaster => 400,
            NpcKind::RaggedCasterOpenCoat => 450,
            NpcKind::Necromancer => 300,
            NpcKind::NecromancerArmored => 450,
            NpcKind::DiabolistRed => 200,
            // W4-d37 — the NPCID 211-280 band's `SetDefaults` lifeMax rows
            // (the `this.lifeMax = N;` cites in `npc.rs`'s enum).
            NpcKind::CochinealBeetle | NpcKind::CyanBeetle | NpcKind::LacBeetle | NpcKind::SeaSnail => 40,
            NpcKind::FlyingFish => 20,
            NpcKind::Pirate => 250,
            NpcKind::GoldfishWalker => 5,
            NpcKind::HornetFatty => 50,
            NpcKind::HornetHoney | NpcKind::HornetSpikey => 42,
            NpcKind::HornetLeafy | NpcKind::HornetStingy => 38,
            NpcKind::JungleCreeper | NpcKind::JungleCreeperWall => 400,
            NpcKind::BlackRecluseWall => 350,
            NpcKind::BloodCrawler | NpcKind::BloodCrawlerWall => 60,
            NpcKind::BloodFeeder | NpcKind::BloodJelly => 150,
            NpcKind::IceGolem => 4000,
            NpcKind::RainbowSlime => 400,
            NpcKind::Eyezor => 1000,
            NpcKind::Parrot => 100,
            NpcKind::Reaper => 700,
            NpcKind::ZombieMushroom => 180,
            NpcKind::ZombieMushroomHat | NpcKind::MushiLadybug => 220,
            NpcKind::FungoFish => 140,
            NpcKind::AnomuraFungus => 230,
            NpcKind::FungiBulb => 90,
            NpcKind::GiantFungiBulb => 300,
            NpcKind::IchorSticker => 340,
            NpcKind::RustyArmoredBonesAxe | NpcKind::BlueArmoredBonesNoPants => 550,
            NpcKind::RustyArmoredBonesFlail | NpcKind::RustyArmoredBonesSwordNoArmor | NpcKind::HellArmoredBones => 400,
            NpcKind::RustyArmoredBonesSword | NpcKind::HellArmoredBonesSpikeShield => 450,
            NpcKind::BlueArmoredBones | NpcKind::BlueArmoredBonesSword | NpcKind::HellArmoredBonesMace | NpcKind::HellArmoredBonesSword => 500,
            NpcKind::BlueArmoredBonesMace => 350,
            // W4-d42 - the NPCID 578-582 SetDefaults rows.
            NpcKind::DD2LightningBug => 500,
            NpcKind::BartenderUnconscious => 250,
            NpcKind::WalkingAntlion => 80,
            NpcKind::LarvaeAntlion => 30,
            // W4-d42 - the NPCID 583-587 SetDefaults rows.
            NpcKind::ZombieMerman => 400,
            // W4-d42 - the NPCID 590-594 SetDefaults rows.
            NpcKind::TorchZombie => 45,
            NpcKind::ArmedTorchZombie => 45,
            NpcKind::GoldGoldfishWalker => 5,
            // W4-d42 - the NPCID 595-599 SetDefaults rows.
            // W4-d42 - the NPCID 600-604 SetDefaults rows.
            NpcKind::Seagull => 5,
            NpcKind::Seagull2 => 5,
            // W4-d42 - the NPCID 605-609 SetDefaults rows.
            NpcKind::Grebe => 5,
            NpcKind::Grebe2 => 5,
            // W4-d42 - the NPCID 610-614 SetDefaults rows.
            NpcKind::Rat => 5,
            NpcKind::Owl => 5,
            NpcKind::WaterStrider => 5,
            NpcKind::GoldWaterStrider => 5,
            NpcKind::ExplosiveBunny => 5,
            // W4-d42 - the NPCID 615-619 SetDefaults rows.
            NpcKind::Turtle => 5,
            NpcKind::TurtleJungle => 5,
            NpcKind::BloodNautilus => 7000,
            NpcKind::BloodSquid => 750,
            // W4-d42 - the NPCID 620-624 SetDefaults rows.
            NpcKind::GoblinShark => 5000,
            NpcKind::BloodEelHead => 6000,
            NpcKind::BloodEelBody => 6000,
            NpcKind::BloodEelTail => 6000,
            NpcKind::Gnome => 25,
            // W4-d42 - the NPCID 625-629 SetDefaults rows.
            NpcKind::SeaTurtle => 5,
            NpcKind::Seahorse => 5,
            NpcKind::GoldSeahorse => 5,
            NpcKind::Dandelion => 50,
            NpcKind::IceMimic => 500,
            // W4-d42 - the NPCID 630-630 SetDefaults rows.
            NpcKind::BloodMummy => 180,
            // W4-c3: 694 60 (`NPC.cs:17744`); the 34/289 lifeMax rows
            // are d34/d38's above (identical values, E8 keep-both).
            NpcKind::WaterBoltMimic => 60,
            // W4-c4 (the style-44 swimmers): 224 20 (`NPC.cs:11892`),
            // 509 90 (`:14848`), 581 60 (`:14875`), 587 300 (`:16630`).
            // E8 merge: FlyingFish's 20 is d37's row above.
            NpcKind::GiantFlyingAntlion => 90,
            NpcKind::FlyingAntlion => 60,
            NpcKind::EyeballFlyingFish => 300,
            // W4-c7 — the firefly family's `lifeMax = 5` rows
            // (`NPC.cs:13524`/`:13566`/`:17272`/`:17586`).
            NpcKind::Firefly | NpcKind::LightningBug | NpcKind::Lavafly | NpcKind::Shimmerfly => 5,
            // W4-c8 — the crawling-critter family's `lifeMax = 5` rows
            // (`NPC.cs:13552`/`:13769`/`:13784`/`:14339`/`:14583`/
            // `:16788`).
            // E8 merge: the 5 d40-owned dups dropped (rows above).
            NpcKind::Worm
            | NpcKind::TruffleWorm
            | NpcKind::TruffleWormDigger
            | NpcKind::Maggot => 5,
            // The snail family + its shimmer target (W4-c9): lifeMax 5
            // each (`NPC.cs:13580` / `:13594` / `:17287` / `:17586`).
            // E8 merge: Shimmerfly's row is c7's above; the 5 gold rows
            // are d40's above (the dup drops).
            NpcKind::Snail | NpcKind::GlowingSnail | NpcKind::MagmaSnail => 5,
            // W4-c10 — the waterfowl family's `lifeMax = 5` rows
            // (`NPC.cs:13620`/`:13632`/`:13644`/`:13656`/`:16746`/
            // `:16758`/`:16816`/`:16828`).
            // E8 merge: Seagull/Grebe's lifeMax rows are d42's above.
            NpcKind::Duck
            | NpcKind::Duck2
            | NpcKind::DuckWhite
            | NpcKind::Duck
            | NpcKind::Duck2
            | NpcKind::DuckWhite
            | NpcKind::DuckWhite2 => 5,
            // E8 merge: the c10 12-critter lifeMax tail dropped (d40/c8's rows).
            // The man-eater family (W4-d23): 43 110 (`NPC.cs:9302`), 56 60
            // (`:9483`), 101 320 (`:10196`), 175 300 (`:11217`), 259 90
            // (`:12341`), 260 300 (`:12357`).
            NpcKind::ManEater => 110,
            NpcKind::Snatcher => 60,
            NpcKind::Clinger => 320,
            NpcKind::AngryTrapper => 300,
            NpcKind::FungiBulb => 90,
            NpcKind::GiantFungiBulb => 300,
            // The aiStyle-9 spit/flare family (W4-d23): 112/666/516 all
            // `lifeMax = 1` (`NPC.cs:10333`/`:17425`/`:15401`).
            NpcKind::VileSpit | NpcKind::VileSpitEoW | NpcKind::SolarFlare => 1,
            // W4-d25 STYLE-26 wiring: 86 400 (`NPC.cs:9933`), 155 300
            // (`:10968`), 546 50 (`:15979`). The Unicorn row mirrors
            // d35 (C).
            NpcKind::Wolf => 300,
            NpcKind::Tumbleweed => 50,
            // W4-d120 — SetDefaults lifeMax rows: 142/160/178/579 250
            // (`NPC.cs:10800`/`:11012`/`:11261`/`:16562`), 209 500
            // (`:11691`).
            NpcKind::SantaClaus | NpcKind::Truffle | NpcKind::Steampunker => 250,
            NpcKind::Cyborg => 500,
            // W4-d125: 636 lifeMax 70000 (`NPC.cs:17195`), 661 lifeMax 5
            // (`:17357`). The expert/master `ScaleStats` halves
            // (`:18400-18404`) ride the difficulty system's seam.
            NpcKind::EmpressOfLight => 70000,
            NpcKind::PrismaticLacewing => 5,
            // W4-g5 — the two casters' `lifeMax` rows: 172 600
            // (`NPC.cs:11161`), 693 70 (`:17728`).
            NpcKind::RuneWizard => 600,
            NpcKind::LibrarianSkeleton => 70,
            // W4-g8 lane 1 batch 1 — the `lifeMax` rows: 61 40
            // (`NPC.cs:9561`), 63 34 (`:9590`), 64 70 (`:9604`), 69 45
            // (`:9673`), 70 100 (`:9688`), 75 150 (`:9780`), 82 160
            // (`:9861`), 83/84 200 (`:9886`/`:9899`), 85 500 (`:9912`).
            // The Mimic's pre-hardmode 300 fold (`:9922`) rides the spawn
            // resolver (`world.hard_mode`), not this hardmode table.
            NpcKind::PinkJellyfish => 70,
            NpcKind::Antlion => 45,
            NpcKind::SpikeBall => 100,
            // W4-g8 lane 1 batch 2 — the `lifeMax` rows: 103 120
            // (`NPC.cs:10224`), 122 220 (`:10487`), 164/165 80 (`:11066`/
            // `:11081`), 169 200 (`:11121`), 174 220 (`:11202`), 177 300
            // (`:11246`), 179 200 (`:11273`), 182 240 (`:11325`), 195 250
            // (`:11499`), 196 300 (`:11513`).
            NpcKind::WallCreeper | NpcKind::WallCreeperWall => 80,
            NpcKind::IceElemental => 200,
            NpcKind::Herpling => 220,
            NpcKind::Derpling => 300,
            NpcKind::CrimsonAxe => 200,
            NpcKind::FloatyGross => 240,
            NpcKind::LostGirl => 250,
            NpcKind::Nymph => 300,
            // W4-g8 lane 1 batch 3 — the `lifeMax` rows: 34 40
            // (`NPC.cs:9157`), 62 120 (`:9575`), 93 100 (`:10055`), 121 60
            // (`:10473`), 137 200 (`:10742`), 150 30 (`:10893`), 151 160
            // (`:10908`), 152 220 (`:10924`), 158 750 (`:15474`), 226 260
            // (`:11919`).
            // W4-g8 lane 1 batch 4 — the `lifeMax` rows: 221 30
            // (`NPC.cs:11848`), 48 100 (`:9367`), 156 600 (`:10982`).
            // W4-g8 machine round 3 — the Vampire 159's 750
            // (`NPC.cs:15488`).
            // The dragonflies (W4-d33): lifeMax 5 each (`NPC.cs:16728`).
            NpcKind::BlackDragonfly
            | NpcKind::BlueDragonfly
            | NpcKind::GreenDragonfly
            | NpcKind::OrangeDragonfly
            | NpcKind::RedDragonfly
            | NpcKind::YellowDragonfly
            | NpcKind::GoldDragonfly => 5,
            // The fairy critters (W4-d33 wiring): lifeMax 5 each
            // (`NPC.cs:16590`); the windy balloon lifeMax 1 (`:16715`).
            // d42 batch-2/3's rows, constants-copy (keep-one d37c-E3).
            NpcKind::FairyCritterPink
            | NpcKind::FairyCritterGreen
            | NpcKind::FairyCritterBlue => 5,
            NpcKind::WindyBalloon => 1,
            // W4-f42 — the sandstorm debris + desert family's `lifeMax`:
            // 546 50 (`NPC.cs:15979`), 61 40 (`:9561`), 69 45 (`:9672`),
            // 78 130 (`:9806`), 79/630 180 (`:9819`/`:17114`), 80 200
            // (`:9832`), 580 80 (`:14861`), 581 60 (`:14875`).
            // W4-d9 wiring (d40 rows): the SetDefaults lifeMax rows —
            // the dragon family 10000 (`NPC.cs:14424`/`:14442`/`:14461`),
            // the 521 squidhead 2000 (`NPC.cs:15775`).
            | NpcKind::CultistDragonBody2
            | NpcKind::CultistDragonBody3
            | NpcKind::CultistDragonTail => 10000,
            NpcKind::AncientCultistSquidhead => 2000,
            // W4-d39: the Butterfly's `lifeMax = 5` row
            // (`NPC.cs:13538`); the scorpion pair `:13668`, the
            // grasshopper `:13818`.
            NpcKind::Butterfly
            | NpcKind::ScorpionBlack
            | NpcKind::Scorpion
            | NpcKind::Grasshopper => 5,
            // W4-d39 batch 3: the teeth bomb 200 (`NPC.cs:13833`), the
            // archer pair 210 (`:13846`).
            NpcKind::ChatteringTeethBomb => 200,
            NpcKind::CultistArcherBlue | NpcKind::CultistArcherWhite => 210,
            // W4-d41 — the cavern crawdads'/shellies' lifeMax 50
            // (`NPC.cs:14673`/`:14685`).
            NpcKind::Crawdad | NpcKind::Crawdad2 => 50,
            NpcKind::GiantShelly | NpcKind::GiantShelly2 => 50,
            // W4-d41 — the salamanders' lifeMax 65 (`NPC.cs:14698`).
            NpcKind::Salamander
            | NpcKind::Salamander2
            | NpcKind::Salamander3
            | NpcKind::Salamander4
            | NpcKind::Salamander5
            | NpcKind::Salamander6
            | NpcKind::Salamander7
            | NpcKind::Salamander8
            | NpcKind::Salamander9 => 65,
            // W4-d41 — the antlion charger's lifeMax 110
            // (`NPC.cs:14835`), the swarmer's 90 (`:14848`), the Dune
            // Splicer chain's 500 (`:14781`/`:14798`/`:14815`).
            NpcKind::GiantWalkingAntlion => 110,
            NpcKind::DuneSplicerHead
            | NpcKind::DuneSplicerBody
            | NpcKind::DuneSplicerTail => 500,
            // W4-d41 — the Tomb Crawler chain's lifeMax 60
            // (`NPC.cs:14726`/`:14743`/`:14760`), the solar fragments'
            // 1 (`:15401`/`:15416`).
            NpcKind::TombCrawlerHead
            | NpcKind::TombCrawlerBody
            | NpcKind::TombCrawlerTail => 60,
            NpcKind::SolarFlare | NpcKind::SolarGoop => 1,
            // W4-d41 — the Ancient Light's lifeMax 400 (`NPC.cs:15787`),
            // the Ancient Doom's 500 (`:15802`); the desert ghouls'
            // 180/250/220 (`:14889` + the `:14898`/`:14905` adds).
            NpcKind::AncientLight => 400,
            NpcKind::AncientDoom => 500,
            NpcKind::DesertGhoul => 180,
            NpcKind::DesertGhoulCorruption => 250,
            NpcKind::DesertGhoulCrimson => 220,
            // W4-d41 — the hallowed ghoul's 300 (`NPC.cs:14889` + the
            // `:14912` +120), the lamias' 350 (`:14927`), the sand
            // poachers' 320 (`:14940`/`:14954`).
            NpcKind::DesertGhoulHallow => 300,
            NpcKind::DesertLamiaLight | NpcKind::DesertLamiaDark => 350,
            NpcKind::DesertScorpionWalk | NpcKind::DesertScorpionWall => 320,
            // W4-d41 — the basilisk's lifeMax 270 (`NPC.cs:14967`), the
            // tortured soul's 400 (`:15820`), the slimes' 50
            // (`:15836`/`:15865`), the bride's 200 (`:15850`).
            NpcKind::DesertBeast => 270,
            NpcKind::DemonTaxCollector => 400,
            NpcKind::SlimeSpiked | NpcKind::SandSlime => 50,
            NpcKind::TheBride => 200,
            // W4-d41 — the critters' lifeMax 5 (`NPC.cs:15880`/`:15892`/
            // `:9343`), the tumbler's 50 (`:15979`).
            NpcKind::SquirrelRed | NpcKind::SquirrelGold | NpcKind::PartyBunny => 5,
            // The luck critters (W4-e26, E8 keep-one): lifeMax 5 each
            // (`NPC.cs:13607` / `:16770`).
            NpcKind::Frog | NpcKind::LadyBug | NpcKind::GoldLadyBug => 5,
            // W4-g10 lane 3 — the town pets' `lifeMax = 250` rows: the
            // 637/638 arm (`NPC.cs:17220`), 656 (`:17304`), 670
            // (`:17490`), the 678-684 arm (`:17603`).
            NpcKind::TownCat
            | NpcKind::TownDog
            | NpcKind::TownBunny
            | NpcKind::TownSlimeBlue
            | NpcKind::TownSlimeGreen
            | NpcKind::TownSlimeOld
            | NpcKind::TownSlimePurple
            | NpcKind::TownSlimeRainbow
            | NpcKind::TownSlimeRed
            | NpcKind::TownSlimeYellow
            | NpcKind::TownSlimeCopper => 250,
            // Doc-sweep lane 36 — 676 lifeMax 80 (`NPC.cs:17573`).
            NpcKind::ShimmerSlime => 80,
            // W4-f18 first-20 port — the critters' `lifeMax = 5` rows
            // (`NPC.cs:13524`/`:13538`/`:13552`/`:13566`/`:13580`).
            NpcKind::Firefly | NpcKind::Butterfly | NpcKind::Worm | NpcKind::LightningBug | NpcKind::Snail => 5,
            // W4-f18 batch 2 — the critters' `lifeMax = 5` rows
            // (`NPC.cs:13590`/`:13604`/`:13616`/`:13628`/`:13640`).
            NpcKind::GlowingSnail | NpcKind::Frog | NpcKind::Duck | NpcKind::Duck2 | NpcKind::DuckWhite => 5,
            // W4-f18 batch 3 — the critters' `lifeMax = 5` rows
            // (`NPC.cs:13652`/`:13664`/`:13765`/`:13780`).
            NpcKind::DuckWhite2 | NpcKind::ScorpionBlack | NpcKind::Scorpion | NpcKind::TruffleWorm | NpcKind::TruffleWormDigger => 5,
            // W4-f18 batch 4 — 377 5 (`NPC.cs:13814`), 378 200
            // (`:13828`), 379/380 210 (`:13842`), 430 45 (`:8615`).
            NpcKind::Grasshopper => 5,
            // The Shimmerfly (W4-c9's rowing, adopted by W4-g62):
            // lifeMax 5 (`NPC.cs:17586`).
            NpcKind::Shimmerfly => 5,
            // W4-d42 - the NPCID 620-624 SetDefaults rows.
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
            // W4-c4 (the style-16 swimmers): 57 30 (`NPC.cs:9495`), 157 75
            // (`:10996`), 241 50 (`:12112`), 465 31 (`:14489`), 692 50
            // (`:17715`); the critters 592/607/615/688 damage 0
            // (`:16685`/`:16801`/`:16898`/`:17659`).
            // E8 merge drops: CorruptGoldfish/BloodFeeder (d34/d37 rows
            // above), Pufferfish/Orca (d43 lands later).
            NpcKind::Arapaima => 75,
            NpcKind::CrimsonGoldfish => 31,
            NpcKind::GoldGoldfish | NpcKind::Pupfish | NpcKind::Dolphin => 0,
            // W4-e21 — the bunny pair: damage 0 (`NPC.cs:9341`).
            NpcKind::Bunny | NpcKind::PartyBunny => 0,
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
            // W4-c6 style-59 wiring: the blade's `damage = 65`
            // (`NPC.cs:13163`).
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
            // W4-d40: the SetDefaults damage rows (`NPC.cs`).
            NpcKind::ArmedZombie => 14,
            NpcKind::ArmedZombieEskimo => 16,
            NpcKind::ArmedZombiePincussion => 16,
            NpcKind::ArmedZombieSlimed => 13,
            NpcKind::ArmedZombieSwamp => 13,
            // W4-d40: the SetDefaults damage rows (`NPC.cs`).
            NpcKind::ArmedZombieTwiggy => 16,
            NpcKind::ArmedZombieCenx => 12,
            NpcKind::CultistTablet => 0,
            NpcKind::CultistDevote => 0,
            NpcKind::CultistBossClone => 0,
            // W4-d40: the SetDefaults damage rows (`NPC.cs`).
            NpcKind::GoldBird => 0,
            NpcKind::GoldBunny => 0,
            NpcKind::GoldButterfly => 0,
            NpcKind::GoldFrog => 0,
            NpcKind::GoldGrasshopper => 0,
            // W4-d40: the SetDefaults damage rows (`NPC.cs`).
            NpcKind::GoldMouse => 0,
            NpcKind::GoldWorm => 0,
            NpcKind::BoneThrowingSkeleton => 20,
            NpcKind::BoneThrowingSkeleton2 => 20,
            NpcKind::BoneThrowingSkeleton3 => 18,
            // W4-d40: the SetDefaults damage rows (`NPC.cs`).
            NpcKind::BoneThrowingSkeleton4 => 22,
            NpcKind::SkeletonMerchant => 10,
            NpcKind::CultistDragonHead => 100,
            NpcKind::CultistDragonBody1 => 50,
            NpcKind::CultistDragonBody2 => 50,
            // W4-d40: the SetDefaults damage rows (`NPC.cs`).
            NpcKind::CultistDragonBody3 => 50,
            NpcKind::CultistDragonBody4 => 50,
            NpcKind::CultistDragonTail => 50,
            NpcKind::Butcher => 70,
            NpcKind::CreatureFromTheDeep => 60,
            // W4-d40: the SetDefaults damage rows (`NPC.cs`).
            NpcKind::Fritz => 70,
            NpcKind::Nailhead => 100,
            NpcKind::CrimsonBunny => 21,
            NpcKind::Psycho => 70,
            // W4-d40: the SetDefaults damage rows (`NPC.cs`).
            NpcKind::DeadlySphere => 100,
            NpcKind::DrManFly => 65,
            NpcKind::ThePossessed => 68,
            NpcKind::CrimsonPenguin => 21,
            NpcKind::ShadowFlameApparition => 50,
            // W4-d40: the SetDefaults damage rows (`NPC.cs`).
            NpcKind::BigMimicCorruption => 90,
            NpcKind::BigMimicCrimson => 90,
            NpcKind::BigMimicHallow => 90,
            NpcKind::BigMimicJungle => 90,
            NpcKind::Mothron => 80,
            // W4-d40: the SetDefaults damage rows (`NPC.cs`).
            NpcKind::MothronEgg => 0,
            NpcKind::MothronSpawn => 50,
            NpcKind::Medusa => 30,
            NpcKind::GreekSkeleton => 22,
            NpcKind::GraniteGolem => 30,
            // W4-d40: the SetDefaults damage rows (`NPC.cs`).
            NpcKind::GraniteFlyer => 24,
            NpcKind::EnchantedNightcrawler => 0,
            NpcKind::Grubby => 0,
            NpcKind::Sluggy => 0,
            NpcKind::Buggy => 0,
            // W4-d40: the SetDefaults damage rows (`NPC.cs`).
            NpcKind::TargetDummy => 0,
            // W4-d35 (A): the 73/74/75/77/78 damage rows.
            NpcKind::GoblinScout => 20,
            NpcKind::Bird => 0,
            NpcKind::ArmoredSkeleton => 40,
            NpcKind::Mummy => 50,
            // W4-d35 (B): the 79/80/82/83/84 damage rows.
            NpcKind::DarkMummy => 60,
            NpcKind::LightMummy => 55,
            // W4-d35 (C): the 85/86/93/94/95 damage rows.
            NpcKind::Unicorn => 65,
            NpcKind::Corruptor => 60,
            NpcKind::DiggerHead => 45,
            // W4-d35 (D): the 96/97/98/99/100 damage rows.
            NpcKind::DiggerBody => 28,
            NpcKind::DiggerTail => 26,
            NpcKind::SeekerHead => 70,
            NpcKind::SeekerBody => 55,
            NpcKind::SeekerTail => 40,
            // W4-d35 (E): the 101/102/103/104/109 damage rows.
            NpcKind::Clinger => 70,
            NpcKind::AnglerFish => 80,
            NpcKind::Werewolf => 70,
            NpcKind::Clown => 60,
            // W4-d35 (F): the 110/112/116/120/121 damage rows.
            NpcKind::SkeletonArcher => 45,
            NpcKind::VileSpit => 65,
            NpcKind::TheHungryII => 30,
            NpcKind::ChaosElemental => 40,
            // W4-d35 (G): the 122/137/140 damage rows.
            NpcKind::PossessedArmor => 55,
            NpcKind::DungeonSpirit => 70,
            // W4-d34 kinds 1-70 (`NPC.cs` SetDefaults damage).
            NpcKind::AngryBones => 26,   // :9113
            NpcKind::OldMan => 10,       // :9204
            NpcKind::ManEater => 34,     // :9300
            NpcKind::Bunny => 0,         // :9341
            NpcKind::BoneSerpentHead => 36,   // :9232
            NpcKind::BoneSerpentBody => 20,   // :9249
            NpcKind::BoneSerpentTail => 16,   // :9267
            NpcKind::CorruptBunny => 20,      // :9353
            NpcKind::Snatcher => 25,          // :9481
            NpcKind::CorruptGoldfish => 30,   // :9495
            NpcKind::Crab => 20,              // :9645
            NpcKind::DungeonGuardian => 9999, // :9657
            // W4-d38 b7 — SetDefaults damage rows.
            NpcKind::SlimeRibbonWhite => 7,
            NpcKind::SlimeRibbonYellow => 6,
            NpcKind::SlimeRibbonGreen => 8,
            NpcKind::SlimeRibbonRed => 5,
            NpcKind::BunnyXmas => 0,
            // W4-d38 b6 — SetDefaults damage rows.
            NpcKind::DemonEyeSpaceship => 20,
            NpcKind::SkeletonTopHat => 23,
            NpcKind::SkeletonAstonaut => 18,
            NpcKind::SkeletonAlien => 22,
            NpcKind::PumpkingBlade => 65,
            // W4-d38 b5 — SetDefaults damage rows.
            NpcKind::SlimeMasked => 7,
            NpcKind::BunnySlimed => 0,
            NpcKind::HoppinJack => 80,
            NpcKind::Ghost => 15,
            NpcKind::DemonEyeOwl => 16,
            // W4-d38 b4 — SetDefaults damage rows.
            NpcKind::BirdBlue => 0,
            NpcKind::BirdRed => 0,
            NpcKind::Squirrel => 0,
            NpcKind::Mouse => 0,
            NpcKind::Raven => 12,
            // W4-d38 b3 — SetDefaults damage rows.
            NpcKind::TacticalSkeleton => 60,
            NpcKind::SkeletonCommando => 60,
            NpcKind::AngryBonesBig => 34,
            NpcKind::AngryBonesBigMuscle => 28,
            NpcKind::AngryBonesBigHelmet => 24,
            // W4-d38 b2 — SetDefaults damage rows.
            NpcKind::DiabolistWhite => 60,
            NpcKind::BoneLee => 90,
            NpcKind::GiantCursedSkull => 60,
            NpcKind::Paladin => 100,
            NpcKind::SkeletonSniper => 60,
            // W4-d38 b1 — SetDefaults damage rows.
            NpcKind::RaggedCaster => 40,
            NpcKind::RaggedCasterOpenCoat => 35,
            NpcKind::Necromancer => 50,
            NpcKind::NecromancerArmored => 35,
            NpcKind::DiabolistRed => 50,
            // W4-d37 — the NPCID 211-280 band's `SetDefaults` damage rows
            // (the `this.damage = N;` cites in `npc.rs`'s enum).
            NpcKind::CochinealBeetle | NpcKind::CyanBeetle | NpcKind::LacBeetle | NpcKind::SeaSnail | NpcKind::Squid => 20,
            NpcKind::FlyingFish => 9,
            NpcKind::Pirate => 10,
            NpcKind::GoldfishWalker => 0,
            NpcKind::HornetFatty => 22,
            NpcKind::HornetHoney => 28,
            NpcKind::HornetLeafy => 30,
            NpcKind::HornetSpikey => 32,
            NpcKind::HornetStingy => 34,
            NpcKind::JungleCreeper | NpcKind::JungleCreeperWall | NpcKind::BlackRecluseWall => 100,
            NpcKind::BloodCrawler | NpcKind::BloodCrawlerWall => 30,
            NpcKind::BloodFeeder => 50,
            NpcKind::BloodJelly => 75,
            NpcKind::IceGolem => 60,
            NpcKind::RainbowSlime => 85,
            NpcKind::Eyezor => 50,
            NpcKind::Parrot | NpcKind::Reaper => 80,
            NpcKind::ZombieMushroom => 40,
            NpcKind::ZombieMushroomHat | NpcKind::AnomuraFungus => 38,
            NpcKind::FungoFish => 90,
            NpcKind::MushiLadybug => 60,
            NpcKind::FungiBulb => 24,
            NpcKind::GiantFungiBulb => 70,
            NpcKind::IchorSticker => 55,
            NpcKind::RustyArmoredBonesAxe | NpcKind::RustyArmoredBonesSword | NpcKind::HellArmoredBones => 70,
            NpcKind::RustyArmoredBonesFlail => 55,
            NpcKind::RustyArmoredBonesSwordNoArmor | NpcKind::HellArmoredBonesSword => 75,
            NpcKind::BlueArmoredBones | NpcKind::BlueArmoredBonesNoPants => 45,
            NpcKind::BlueArmoredBonesMace | NpcKind::HellArmoredBonesSpikeShield => 65,
            NpcKind::BlueArmoredBonesSword => 85,
            NpcKind::HellArmoredBonesMace => 40,
            // W4-d42 - the NPCID 578-582 SetDefaults rows.
            NpcKind::DD2LightningBug => 80,
            NpcKind::BartenderUnconscious => 10,
            NpcKind::WalkingAntlion => 25,
            NpcKind::LarvaeAntlion => 10,
            // W4-d42 - the NPCID 583-587 SetDefaults rows.
            NpcKind::ZombieMerman => 40,
            // W4-d42 - the NPCID 590-594 SetDefaults rows.
            NpcKind::TorchZombie => 14,
            NpcKind::ArmedTorchZombie => 14,
            NpcKind::GoldGoldfishWalker => 0,
            // W4-d42 - the NPCID 595-599 SetDefaults rows.
            // W4-d42 - the NPCID 600-604 SetDefaults rows.
            NpcKind::Seagull => 0,
            NpcKind::Seagull2 => 0,
            // W4-d42 - the NPCID 605-609 SetDefaults rows.
            NpcKind::Grebe => 0,
            NpcKind::Grebe2 => 0,
            // W4-d42 - the NPCID 610-614 SetDefaults rows.
            NpcKind::Rat => 0,
            NpcKind::Owl => 0,
            NpcKind::WaterStrider => 0,
            NpcKind::GoldWaterStrider => 0,
            NpcKind::ExplosiveBunny => 0,
            // W4-d42 - the NPCID 615-619 SetDefaults rows.
            NpcKind::Turtle => 0,
            NpcKind::TurtleJungle => 0,
            NpcKind::BloodNautilus => 55,
            NpcKind::BloodSquid => 60,
            // W4-d42 - the NPCID 620-624 SetDefaults rows.
            NpcKind::GoblinShark => 70,
            NpcKind::BloodEelHead => 90,
            NpcKind::BloodEelBody => 60,
            NpcKind::BloodEelTail => 50,
            NpcKind::Gnome => 10,
            // W4-d42 - the NPCID 625-629 SetDefaults rows.
            NpcKind::SeaTurtle => 0,
            NpcKind::Seahorse => 0,
            NpcKind::GoldSeahorse => 0,
            NpcKind::Dandelion => 15,
            NpcKind::IceMimic => 80,
            // W4-d42 - the NPCID 630-630 SetDefaults rows.
            NpcKind::BloodMummy => 60,
            // W4-c3: 694 20 (`NPC.cs:17742`); the 34/289 damage rows
            // are d34/d38's above (identical values, E8 keep-both).
            NpcKind::WaterBoltMimic => 20,
            // W4-c4 (the style-44 swimmers): 224 9 (`NPC.cs:11890`),
            // 509 34 (`:14846`), 581 29 (`:14873`), 587 35 (`:16628`).
            // E8 merge: FlyingFish's 9 is d37's row above.
            NpcKind::GiantFlyingAntlion => 34,
            NpcKind::FlyingAntlion => 29,
            NpcKind::EyeballFlyingFish => 35,
            // W4-c7 — the firefly family's `damage = 0` rows
            // (`NPC.cs:13522`/`:13564`/`:17270`/`:17584`) — the 677 threat
            // scan's own `damage > 0` gate excludes them from each other.
            NpcKind::Firefly | NpcKind::LightningBug | NpcKind::Lavafly | NpcKind::Shimmerfly => 0,
            // W4-c8 — the crawling-critter family's `damage = 0` rows
            // (`NPC.cs:13550`/`:13767`/`:13782`/`:14337`/`:14581`/
            // `:16786`) — the friendly critters never hurt the player.
            // E8 merge: the 5 d40-owned dups dropped (rows above).
            NpcKind::Worm
            | NpcKind::TruffleWorm
            | NpcKind::TruffleWormDigger
            | NpcKind::Maggot => 0,
            // The snail family + its shimmer target (W4-c9): damage 0
            // each (`NPC.cs:13578` / `:13592` / `:17285` / `:17584`).
            // E8 merge: Shimmerfly's row is c7's above; the 5 gold rows
            // are d40's above (the dup drops).
            NpcKind::Snail | NpcKind::GlowingSnail | NpcKind::MagmaSnail => 0,
            // W4-c10 — the waterfowl family's `damage = 0` rows
            // (`NPC.cs:13618`/`:13630`/`:13642`/`:13654`/`:16744`/
            // `:16756`/`:16814`/`:16826`).
            // E8 merge: Seagull/Grebe's damage rows are d42's above.
            NpcKind::Duck
            | NpcKind::Duck2
            | NpcKind::DuckWhite
            | NpcKind::Duck
            | NpcKind::Duck2
            | NpcKind::DuckWhite
            | NpcKind::DuckWhite2 => 0,
            // E8 merge: the c10 12-critter damage tail dropped (d40/c8's rows).
            // The man-eater family (W4-d23): 43 34 (`NPC.cs:9300`), 56 25
            // (`:9481`), 101 70 (`:10194`), 175 100 (`:11215`), 259 24
            // (`:12339`), 260 70 (`:12355`).
            NpcKind::ManEater => 34,
            NpcKind::Snatcher => 25,
            NpcKind::Clinger => 70,
            NpcKind::AngryTrapper => 100,
            NpcKind::FungiBulb => 24,
            NpcKind::GiantFungiBulb => 70,
            // The aiStyle-9 spit/flare family (W4-d23): 112/666 dmg 65
            // (`NPC.cs:10331`/`:17423`), 516 dmg 100 (`:15399`).
            NpcKind::VileSpit | NpcKind::VileSpitEoW => 65,
            NpcKind::SolarFlare => 100,
            // The aiStyle-14 flyer family (W4-d23): 48 dmg 25
            // (`NPC.cs:9365`), 62 32 (`:9573`), 93 45 (`:10053`), 121 45
            // (`:10471`), 137 75 (`:10740`), 150 18 (`:10891`), 151 50
            // (`:10906`), 152 80 (`:10922`), 156 50 (`:10980`), 158 60
            // (`:15472`), 159 80 (`:15486`), 226 85 (`:11917`), 634 13
            // (`:17167`), 660 50 (`:17343`).
            NpcKind::GiantBat | NpcKind::Slimer => 45,
            NpcKind::IceBat => 18,
            NpcKind::Lavabat | NpcKind::RedDevil | NpcKind::QueenSlimeMinionPurple => 50,
            NpcKind::GiantFlyingFox => 80,
            NpcKind::VampireBat => 60,
            NpcKind::Vampire => 80,
            NpcKind::SporeBat => 13,
            // W4-d25 STYLE-26 wiring: 86/155 dmg 65 (`NPC.cs:9931`/
            // `:10966`), 546 dmg 30 (`:15977`). The Unicorn row mirrors
            // d35 (C).
            NpcKind::Unicorn | NpcKind::Wolf => 65,
            NpcKind::Tumbleweed => 30,
            // W4-d120 — SetDefaults damage rows: all five write 10
            // (`NPC.cs:10798`/`:11010`/`:11259`/`:11689`/`:16561`).
            NpcKind::SantaClaus
            | NpcKind::Truffle
            | NpcKind::Steampunker
            | NpcKind::Cyborg
            | NpcKind::BartenderUnconscious => 10,
            // W4-d125: 636 damage 80 (`NPC.cs:17194` — AI_120's tail
            // rewrites it per tick from defDamage, 9999 enraged), 661
            // damage 0 (`:17355`).
            NpcKind::EmpressOfLight => 80,
            NpcKind::PrismaticLacewing => 0,
            // W4-g5 — the two casters' `damage` rows: 172 200
            // (`NPC.cs:11159`), 693 18 (`:17727`).
            NpcKind::RuneWizard => 200,
            NpcKind::LibrarianSkeleton => 18,
            // W4-g8 lane 1 batch 1 — the `damage` rows: 61 15
            // (`NPC.cs:9559`), 63 25 (`:9588`), 64 30 (`:9602`), 69 10
            // (`:9671`), 70 32 (`:9686`), 75 55 (`:9778`), 82 65
            // (`:9859`), 83/84 80 (`:9884`/`:9897`), 85 80 (`:9910`).
            // The Mimic's pre-hardmode 30 fold (`:9920`) rides the spawn
            // resolver (`world.hard_mode`).
            NpcKind::PinkJellyfish => 30,
            NpcKind::Antlion => 10,
            NpcKind::SpikeBall => 32,
            // W4-g8 lane 1 batch 2 — the `damage` rows: 103 80
            // (`NPC.cs:10222`), 122 60 (`:10485`), 164/165 30 (`:11064`/
            // `:11079`), 169 55 (`:11119`), 174 65 (`:11200`), 177 80
            // (`:11244`), 179 80 (`:11271`), 182 65 (`:11323`), 195 10
            // (`:11497`), 196 35 (`:11511`).
            NpcKind::WallCreeper | NpcKind::WallCreeperWall => 30,
            NpcKind::IceElemental => 55,
            NpcKind::Herpling => 65,
            NpcKind::Derpling => 80,
            NpcKind::CrimsonAxe => 80,
            NpcKind::FloatyGross => 65,
            NpcKind::LostGirl => 10,
            NpcKind::Nymph => 35,
            // W4-g8 lane 1 batch 3 — the `damage` rows: 34 35
            // (`NPC.cs:9155`), 62 32 (`:9573`), 93 45 (`:10053`), 121 45
            // (`:10471`), 137 75 (`:10740`), 150 18 (`:10891`), 151 50
            // (`:10906`), 152 80 (`:10922`), 158 60 (`:15472`), 226 85
            // (`:11917`).
            // W4-g8 lane 1 batch 4 — the `damage` rows: 221 20
            // (`NPC.cs:11846`), 48 25 (`:9365`), 156 50 (`:10980`).
            // W4-g8 machine round 3 — the Vampire 159's 80
            // (`NPC.cs:15486`).
            // The dragonflies (W4-d33): damage 0 each (`NPC.cs:16726`).
            NpcKind::BlackDragonfly
            | NpcKind::BlueDragonfly
            | NpcKind::GreenDragonfly
            | NpcKind::OrangeDragonfly
            | NpcKind::RedDragonfly
            | NpcKind::YellowDragonfly
            | NpcKind::GoldDragonfly => 0,
            // The fairy critters (W4-d33 wiring): damage 0 each
            // (`NPC.cs:16588`); the windy balloon damage 0 (`:16712`).
            // d42 batch-2/3's rows, constants-copy (keep-one d37c-E3).
            NpcKind::FairyCritterPink
            | NpcKind::FairyCritterGreen
            | NpcKind::FairyCritterBlue
            | NpcKind::WindyBalloon => 0,
            // W4-f42 — the sandstorm debris + desert family's `damage`:
            // 546 30 (`NPC.cs:15977`), 61 15 (`:9559`), 69 10 (`:9671`),
            // 78 50 (`:9804`), 79/630 60 (`:9817`/`:17112`), 80 55
            // (`:9830`), 580 25 (`:14859`), 581 29 (`:14873`).
            // W4-d9 wiring (d40 rows): the SetDefaults damage rows — the
            // dragon head 100 (`NPC.cs:14422`), the bodies/tail 50
            // (`:14440`/`:14459`), the 521 squidhead 90 (`NPC.cs:15773`).
            NpcKind::CultistDragonBody1
            | NpcKind::CultistDragonBody2
            | NpcKind::CultistDragonBody3
            NpcKind::AncientCultistSquidhead => 90,
            // W4-d39: the Butterfly's `damage = 0` row
            // (`NPC.cs:13536`); the scorpion pair `:13665`, the
            // grasshopper `:13816`.
            NpcKind::Butterfly
            | NpcKind::ScorpionBlack
            | NpcKind::Scorpion
            | NpcKind::Grasshopper => 0,
            // W4-d39 batch 3: the teeth bomb 120 (`NPC.cs:13831`), the
            // archer pair 45 (`:13844`).
            NpcKind::ChatteringTeethBomb => 120,
            NpcKind::CultistArcherBlue | NpcKind::CultistArcherWhite => 45,
            // W4-d41 — the cavern crawdads' damage 28 (`NPC.cs:14671`)
            // and the shellies' 16 (`:14683`).
            NpcKind::Crawdad | NpcKind::Crawdad2 => 28,
            NpcKind::GiantShelly | NpcKind::GiantShelly2 => 16,
            // W4-d41 — the salamanders' damage 18 (`NPC.cs:14696`).
            NpcKind::Salamander
            | NpcKind::Salamander2
            | NpcKind::Salamander3
            | NpcKind::Salamander4
            | NpcKind::Salamander5
            | NpcKind::Salamander6
            | NpcKind::Salamander7
            | NpcKind::Salamander8
            | NpcKind::Salamander9 => 18,
            // W4-d41 — the antlion charger's damage 38 (`NPC.cs:14833`),
            // the swarmer's 34 (`:14846`), the Dune Splicer head's 58
            // (`:14779`), body's 54 (`:14796`), tail's 50 (`:14814`).
            NpcKind::GiantWalkingAntlion => 38,
            NpcKind::DuneSplicerHead => 58,
            NpcKind::DuneSplicerBody => 54,
            NpcKind::DuneSplicerTail => 50,
            // W4-d41 — the Tomb Crawler head's damage 18
            // (`NPC.cs:14724`), body/tail's 7 (`:14741`/`:14759`); the
            // Solar Flare's 100 (`:15399`), the Solar Goop's 120
            // (`:15414`).
            NpcKind::TombCrawlerHead => 18,
            NpcKind::TombCrawlerBody | NpcKind::TombCrawlerTail => 7,
            NpcKind::SolarGoop => 120,
            // W4-d41 — the Ancient Light's damage 120 (`NPC.cs:15785`),
            // the Ancient Doom's 30 (`:15800`); the desert ghouls'
            // 50/60/64 (`:14887` + the `:14900`/`:14907` adds).
            NpcKind::AncientLight => 120,
            NpcKind::AncientDoom => 30,
            NpcKind::DesertGhoul => 50,
            NpcKind::DesertGhoulCorruption => 60,
            NpcKind::DesertGhoulCrimson => 64,
            // W4-d41 — the hallowed ghoul's 54 (`NPC.cs:14887` + the
            // `:14914` +4), the lamias' 52 (`:14925`), the sand
            // poachers' 66 (`:14938`/`:14952`).
            NpcKind::DesertGhoulHallow => 54,
            NpcKind::DesertLamiaLight | NpcKind::DesertLamiaDark => 52,
            NpcKind::DesertScorpionWalk | NpcKind::DesertScorpionWall => 66,
            // W4-d41 — the basilisk's damage 65 (`NPC.cs:14965`), the
            // tortured soul's + the bride's 14 (`:15818`/`:15848`), the
            // spiked slime's 14 (`:15834`), the sand slime's 15
            // (`:15863`).
            NpcKind::DesertBeast => 65,
            NpcKind::DemonTaxCollector | NpcKind::TheBride => 14,
            NpcKind::SlimeSpiked => 14,
            NpcKind::SandSlime => 15,
            // W4-d41 — the critters' damage 0 (`NPC.cs:15878`/`:15890`/
            // `:9341`), the tumbler's 30 (`:15977`).
            NpcKind::SquirrelRed | NpcKind::SquirrelGold | NpcKind::PartyBunny => 0,
            // The luck critters (W4-e26, E8 keep-one): damage 0 each
            // (`NPC.cs:13606` / `:16768`).
            NpcKind::Frog | NpcKind::LadyBug | NpcKind::GoldLadyBug => 0,
            // W4-g10 lane 3 — the town pets' `damage = 10` rows: the
            // 637/638 arm (`NPC.cs:17218`), 656 (`:17302`), 670
            // (`:17488`), the 678-684 arm (`:17601`).
            NpcKind::TownCat
            | NpcKind::TownDog
            | NpcKind::TownBunny
            | NpcKind::TownSlimeBlue
            | NpcKind::TownSlimeGreen
            | NpcKind::TownSlimeOld
            | NpcKind::TownSlimePurple
            | NpcKind::TownSlimeRainbow
            | NpcKind::TownSlimeRed
            | NpcKind::TownSlimeYellow
            | NpcKind::TownSlimeCopper => 10,
            // Doc-sweep lane 36 — 676 damage 20 (`NPC.cs:17571`).
            NpcKind::ShimmerSlime => 20,
            // W4-f18 first-20 port — the critters' `damage = 0` rows
            // (`NPC.cs:13522`/`:13536`/`:13550`/`:13564`/`:13578`).
            NpcKind::Firefly | NpcKind::Butterfly | NpcKind::Worm | NpcKind::LightningBug | NpcKind::Snail => 0,
            // W4-f18 batch 2 — the critters' `damage = 0` rows
            // (`NPC.cs:13588`/`:13602`/`:13614`/`:13626`/`:13638`).
            NpcKind::GlowingSnail | NpcKind::Frog | NpcKind::Duck | NpcKind::Duck2 | NpcKind::DuckWhite => 0,
            // W4-f18 batch 3 — the critters' `damage = 0` rows
            // (`NPC.cs:13650`/`:13662`/`:13763`/`:13778`).
            NpcKind::DuckWhite2 | NpcKind::ScorpionBlack | NpcKind::Scorpion | NpcKind::TruffleWorm | NpcKind::TruffleWormDigger => 0,
            // W4-f18 batch 4 — 377 0 (`NPC.cs:13812`), 378 120
            // (`:13826`), 379/380 45 (`:13840`), 430 14 (`:8613`).
            NpcKind::Grasshopper => 0,
            // The Shimmerfly (W4-c9's rowing, adopted by W4-g62):
            // damage 0 (`NPC.cs:17584`).
            NpcKind::Shimmerfly => 0,
            // W4-d42 - the NPCID 620-624 SetDefaults rows.
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
