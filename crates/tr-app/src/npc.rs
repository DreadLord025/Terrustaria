//! First enemy pass: a hopping blue slime, a flying demon eye and a walking zombie,
//! modelled on the simplest `NPC.cs` behaviours.

use tr_core::{TILE_PX, World};

use crate::npc_ai::HoverParams;
use crate::npc_buffs::NpcBuffs;
use crate::player::{Player, collides_rect};

pub(crate) const GRAVITY: f32 = 0.4;
pub(crate) const MAX_FALL: f32 = 8.0;

/// `Collision.WetCollision` (simplified): any water tile overlapping the body.
pub(crate) fn is_wet(world: &World, x: f32, y: f32, w: f32, h: f32) -> bool {
    let x0 = (x / TILE_PX).floor() as i32;
    let x1 = ((x + w - 0.01) / TILE_PX).floor() as i32;
    let y0 = (y / TILE_PX).floor() as i32;
    let y1 = ((y + h - 0.01) / TILE_PX).floor() as i32;
    for ty in y0..=y1 {
        for tx in x0..=x1 {
            if world.tile(tx, ty).is_some_and(|t| t.liquid > 0) {
                return true;
            }
        }
    }
    false
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum NpcKind {
    Slime,
    DemonEye,
    /// `NPCID.WanderingEye` (133) — the same `aiStyle 2` machine, night
    /// hardmode spawn (`NPC.cs:4555-4558`).
    WanderingEye,
    Zombie,
    KingSlime,
    /// `NPCID.Guide` (22) — the first town NPC (`aiStyle 7`): the port's
    /// walker is the zombie machine minus the chase (the town NPC wanders).
    Guide,
    /// `NPCID.Skeleton` (21) — the cave walker (`aiStyle 3`, the 15-frame
    /// humanoid strip).
    Skeleton,
    /// `NPCID.UndeadMiner` (44) — the rare cave walker (`aiStyle 3`).
    UndeadMiner,
    /// `NPCID.CaveBat` (49) — the cave flyer (`aiStyle 14`, the erratic
    /// flight, 5 frames).
    CaveBat,
    /// `NPCID.JungleBat` (51) — the jungle flyer (`aiStyle 14`).
    JungleBat,
    /// `NPCID.Hellbat` (60) — the underworld flyer (`aiStyle 14`).
    Hellbat,
    /// `NPCID.VoodooDemon` (66) — the underworld flyer (`aiStyle 14`).
    VoodooDemon,
    /// `NPCID.EaterofSouls` (6) — the corruption hover (`aiStyle 5`).
    EaterOfSouls,
    /// `NPCID.Hornet` (42) — the jungle hover (`aiStyle 5`).
    Hornet,
    /// `NPCID.MeteorHead` (23) — the meteor hover (`aiStyle 5`).
    MeteorHead,
    /// `NPCID.GoblinSorcerer` (29) — the goblin-invasion caster (`aiStyle 8`:
    /// 18x38, 40 HP).
    GoblinSorcerer,
    /// `NPCID.Tim` (45) — the deep-cavern caster (`aiStyle 8`, `NPC.cs:9322`:
    /// 18x40, 200 HP, value 5000).
    Tim,
    /// `NPCID.DarkCaster` (32) — the dungeon caster (`aiStyle 8`,
    /// `NPC.cs:9121`: 18x40, 50 HP).
    DarkCaster,
    /// `NPCID.FireImp` (24) — the underworld caster (`aiStyle 8`,
    /// `NPC.cs:9078`: 18x40, 70 HP, lava-immune).
    FireImp,
    /// `NPCID.ChaosBall` (30) — the caster's sphere (`aiStyle 9`, 16x16,
    /// 1 HP, `noGravity`/`noTileCollide`). Tim's `ChaosBallTim` clone 665
    /// folds here (identical stats and name, `NPC.cs:17398`).
    ChaosBall,
    /// `NPCID.BurningSphere` (25) — the Fire Imp's sphere (`aiStyle 9`,
    /// `NPC.cs:9093`).
    BurningSphere,
    /// `NPCID.WaterSphere` (33) — the Dark Caster's sphere (`aiStyle 9`,
    /// `NPC.cs:9135`).
    WaterSphere,
    /// `NPCID.Goldfish` (55) — the passive water critter (`aiStyle 16`,
    /// `NPC.cs:9460`: 20x18, 5 HP, damage 0, the flag25 exclusion).
    Goldfish,
    /// `NPCID.Piranha` (58) — the jungle/deep-water fish (`aiStyle 16`,
    /// `NPC.cs:9502`: 18x20, 30 HP).
    Piranha,
    /// `NPCID.Shark` (65) — the ocean fish (`aiStyle 16`, `NPC.cs:9610`:
    /// 100x24, 300 HP).
    Shark,
    /// `NPCID.DevourerHead` (7) — the corruption worm's head (`aiStyle 6`,
    /// `NPC.cs:8763`). The body 8 and the tail 9 chain behind it.
    DevourerHead,
    /// `NPCID.DevourerBody` (8) — the middle segments (`NPC.cs:8792`).
    DevourerBody,
    /// `NPCID.DevourerTail` (9) — the tail segment (`NPC.cs:8821`).
    DevourerTail,
    /// `NPCID.GiantWormHead` (10) — the cavern worm's head (`NPC.cs:8850`).
    GiantWormHead,
    /// `NPCID.GiantWormBody` (11) — the middle segments (`NPC.cs:8867`).
    GiantWormBody,
    /// `NPCID.GiantWormTail` (12) — the tail segment (`NPC.cs:8885`).
    GiantWormTail,
    /// `NPCID.EaterofWorldsHead` (13) — the boss worm's head (`NPC.cs:8903`);
    /// the natural spawn waits for the orb-break summon work.
    EaterOfWorldsHead,
    /// `NPCID.EaterofWorldsBody` (14) — the middle segments (`NPC.cs:8923`).
    EaterOfWorldsBody,
    /// `NPCID.EaterofWorldsTail` (15) — the tail segment (`NPC.cs:8943`).
    EaterOfWorldsTail,
    /// `NPCID.Merchant` (17) — the second town NPC (`NPC.cs:8980`: 18x40,
    /// dmg 10, def 15, 250 HP; the shop index 1 through M4-TOWN-3's
    /// engine; the arrival `SpawnAllowed_Merchant`, `NPC.cs:7175`).
    Merchant,
    /// `NPCID.Dryad` (20) — the nature town NPC (`NPC.cs:9022`: the town
    /// twin; the shop index 3; the arrival rides the downedBoss1 gate —
    /// the boss-downed tracking lands with the boss work).
    Dryad,
    /// `NPCID.Nurse` (18) — the healer town NPC (`NPC.cs:8994`: the town
    /// twin; no shop — the heal interaction rides the right-click; the
    /// arrival `SpawnAllowed_Nurse`, `NPC.cs:7158`).
    Nurse,
    /// `NPCID.ArmsDealer` (19) — the ammo seller (`NPC.cs:9008`: the town
    /// twin; the shop index 2; the arrival `SpawnAllowed_ArmsDealer`,
    /// `NPC.cs:7092` — the bullets in the inventory).
    ArmsDealer,
    /// `NPCID.Demolitionist` (38) — the explosive seller (`NPC.cs:9211`:
    /// the town twin; the shop index 4; the arrival
    /// `SpawnAllowed_Demolitionist`, `NPC.cs:7067` — the bombs in the
    /// inventory).
    Demolitionist,
    /// `NPCID.Painter` (227) — the paint seller (`NPC.cs:11925`: the town
    /// twin; the shop index 15; the arrival = at least 2 other town NPCs).
    Painter,
    /// `NPCID.WitchDoctor` (228) — the jungle shaman (`NPC.cs:11942`: the
    /// town twin; the shop index 16; the arrival rides the downedQueenBee
    /// gate — constants-off until the boss-downed tracking).
    WitchDoctor,
    /// `NPCID.Clothier` (54) — the tailor (`NPC.cs:9446`: the town twin;
    /// the shop index 5; the arrival rides the downedSkeletron gate —
    /// constants-off).
    Clothier,
    /// `NPCID.EyeofCthulhu` (4) — the first boss (`NPC.cs:8703-8719`:
    /// 100x110, dmg 15, def 12, 2800 HP, noGravity/noTileCollide; the
    /// death sets `downed_eoc` and opens the Dryad's arrival). The full
    /// aiStyle-4 phase machine rides M4-BOSS-2b.
    EyeOfCthulhu,
    /// `NPCID.QueenBee` (222) — the jungle boss (`NPC.cs:11855-11872`:
    /// 66x66, dmg 30, def 8, 3400 HP; the death sets `downed_queen_bee`
    /// and opens the Witch Doctor's arrival).
    QueenBee,
    /// `NPCID.SkeletronHead` (35) — the dungeon gatekeeper
    /// (`NPC.cs:9166-9182`: 80x102, dmg 32, def 10, 4400 HP; the death
    /// sets `downed_skeletron` and opens the Clothier's arrival).
    SkeletronHead,
    /// `NPCID.ServantofCthulhu` (5) — the EoC minion (`NPC.cs:8717-8727`:
    /// 20×20, aiStyle 5 (the generic hover, num 5 / num2 0.03
    /// `NPC.cs:50299-50301`), damage 12, lifeMax 8, noGravity,
    /// noTileCollide).
    ServantOfCthulhu,
    /// `NPCID.WallofFlesh` (113) — the underworld wall (`NPC.cs:10342`:
    /// 100x100 x1.2, dmg 50, def 12, 8000 HP; the death starts hardmode —
    /// `WorldGen.StartHardmode()`, `NPC.cs:79485`).
    WallOfFlesh,
    /// `NPCID.TheHungry` (115) — the wall's minion (`NPC.cs:10363+`:
    /// 30×30, aiStyle 29, dmg 30, def 10, 240 HP, noGravity,
    /// noTileCollide). The AI anchors to the wall's draw band
    /// (`Main.wofNPCIndex`/`wofDrawArea*` → the World fields) and reads the
    /// wall's hp for the damage/defense tiers.
    Hungry,
    /// `NPCID.LeechHead` (117) — the tongue-phase worm head
    /// (`NPC.cs`: 14×14, aiStyle 6 (the worm engine), dmg 26, def 2,
    /// 60 HP, noGravity/noTileCollide, behindTiles).
    LeechHead,
    /// `NPCID.LeechBody` (118) — the worm's link (`NPC.cs`: 14×14,
    /// aiStyle 6, dmg 22, def 6, 60 HP).
    LeechBody,
    /// `NPCID.TheDestroyer` (134) — the mech worm (`NPC.cs:10659+`,
    /// the same-kind chain; M4-BOSS-7).
    TheDestroyer,
    /// `NPCID.SkeletronPrime` (127) — the mech skull (`NPC.cs:10556+`,
    /// M4-BOSS-8).
    SkeletronPrime,
    /// `NPCID.Retinazer` (125) — the Twins' laser eye (`NPC.cs:10522+`,
    /// M4-BOSS-9).
    Retinazer,
    /// `NPCID.Spazmatism` (126) — the Twins' cursed eye (M4-BOSS-9).
    Spazmatism,
    /// `NPCID.Plantera` (262) — the jungle bulb boss (`NPC.cs:12379+`,
    /// M4-BOSS-10).
    Plantera,
    /// `NPCID.BrainofCthulhu` (266) — the crimson boss (`NPC.cs:12438`:
    /// 160x110, dmg 30, def 14, 1250 HP, dontTakeDamage in the phase 1;
    /// the death sets the downedBoss2 equivalent).
    BrainOfCthulhu,
    /// `NPCID.Creeper` (267) — the Brain's satellite (`NPC.cs:12456`:
    /// 30x30, dmg 20, def 10, 100 HP).
    Creeper,
    /// `NPCID.GoblinPeon` (26) — x0.9, dmg 12, def 4, 60 HP.
    GoblinPeon,
    /// `NPCID.GoblinThief` (27) — x0.95, dmg 20, def 6, 80 HP.
    GoblinThief,
    /// `NPCID.GoblinWarrior` (28) — x1.1, dmg 25, def 8, 110 HP.
    GoblinWarrior,
    /// `NPCID.GoblinArcher` (111) — x0.95, dmg 20, def 6, 80 HP.
    GoblinArcher,
    /// `NPCID.BoundGoblin` (105) — the tied Tinkerer awaiting the rescue
    /// (`SetDefaults` `NPC.cs:10245-10257`: 18x34, aiStyle 0 — no
    /// movement, dmg 10, def 15, 250 HP, kb 0.5, friendly).
    BoundGoblin,
    /// `NPCID.GoblinTinkerer` (107) — the freed Tinkerer (`SetDefaults`
    /// `NPC.cs:10271-10283`: 18x40, aiStyle 7 the town walker, 250 HP,
    /// def 15, kb 0.5).
    GoblinTinkerer,
}

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

impl NpcKind {
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
            NpcKind::Tim => 16,
            NpcKind::DarkCaster => 17,
            NpcKind::FireImp => 18,
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
            NpcKind::TheDestroyer => 50,
            NpcKind::SkeletronPrime => 51,
            NpcKind::Retinazer => 52,
            NpcKind::Spazmatism => 53,
            NpcKind::Plantera => 54,
            NpcKind::BrainOfCthulhu => 55,
            NpcKind::Creeper => 56,
            NpcKind::GoblinPeon => 57,
            NpcKind::GoblinThief => 58,
            NpcKind::GoblinWarrior => 59,
            NpcKind::GoblinArcher => 60,
            NpcKind::BoundGoblin => 61,
            NpcKind::GoblinTinkerer => 62,
        }
    }

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
            50 => NpcKind::TheDestroyer,
            51 => NpcKind::SkeletronPrime,
            52 => NpcKind::Retinazer,
            53 => NpcKind::Spazmatism,
            54 => NpcKind::Plantera,
            55 => NpcKind::BrainOfCthulhu,
            56 => NpcKind::Creeper,
            57 => NpcKind::GoblinPeon,
            58 => NpcKind::GoblinThief,
            59 => NpcKind::GoblinWarrior,
            60 => NpcKind::GoblinArcher,
            61 => NpcKind::BoundGoblin,
            62 => NpcKind::GoblinTinkerer,
            _ => return None,
        })
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
                | NpcKind::TheDestroyer
        )
    }

    /// The Destroyer: the head of its own same-kind chain (`AI_037`).
    pub fn is_worm_head(self) -> bool {
        matches!(
            self,
            NpcKind::DevourerHead
                | NpcKind::GiantWormHead
                | NpcKind::EaterOfWorldsHead
                | NpcKind::LeechHead
                | NpcKind::TheDestroyer
        )
    }

    /// The mid-body kinds that spawn another of themselves while the
    /// budget holds (`:51350`'s list plus the Destroyer's same-kind
    /// chain).
    pub fn is_worm_mid(self) -> bool {
        matches!(
            self,
            NpcKind::DevourerBody
                | NpcKind::GiantWormBody
                | NpcKind::EaterOfWorldsBody
                | NpcKind::LeechBody
                | NpcKind::TheDestroyer
        )
    }

    /// The next segment's kind down the chain (`type + 1` in the vanilla
    /// spawns, `:51347`/`:51357`).
    pub fn worm_next(self) -> NpcKind {
        if self == NpcKind::TheDestroyer {
            // The same-kind chain: every segment IS the head type.
            return self;
        }
        match self {
            NpcKind::DevourerHead => NpcKind::DevourerBody,
            NpcKind::DevourerBody | NpcKind::DevourerTail => NpcKind::DevourerTail,
            NpcKind::GiantWormHead => NpcKind::GiantWormBody,
            NpcKind::GiantWormBody | NpcKind::GiantWormTail => NpcKind::GiantWormTail,
            NpcKind::EaterOfWorldsHead => NpcKind::EaterOfWorldsBody,
            NpcKind::LeechHead => NpcKind::LeechBody,
            NpcKind::LeechBody => NpcKind::LeechBody,
            _ => NpcKind::EaterOfWorldsTail,
        }
    }

    /// The head's dig tunables — `num45` (the max speed) and `num46` (the
    /// steering accel) per family (`:51670-51706`).
    pub fn worm_tunables(self) -> (f32, f32) {
        match self {
            NpcKind::DevourerHead => (9.0, 0.1),
            NpcKind::GiantWormHead => (6.0, 0.05),
            NpcKind::EaterOfWorldsHead => (10.0, 0.07),
            _ => (8.0, 0.07),
        }
    }

    /// The vanilla hitboxes: blue slime 24x18 (`NPC.cs:8568-8569`), demon eye
    /// and Wandering Eye 30x32 (`NPC.cs` cases 2/133), zombie 18x40.
    pub fn size(self) -> (f32, f32) {
        match self {
            NpcKind::Slime => (24.0, 18.0),
            NpcKind::DemonEye | NpcKind::WanderingEye => (30.0, 32.0),
            NpcKind::Zombie => (18.0, 40.0),
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
            // `SetDefaults` case 127 (`NPC.cs:10558`): 80x102.
            NpcKind::SkeletronPrime => (80.0, 102.0),
            // The Twins: `SetDefaults` cases 125/126 are 100x110.
            NpcKind::Retinazer => (100.0, 110.0),
            NpcKind::Spazmatism => (100.0, 110.0),
            // `SetDefaults` case 262 (`NPC.cs:12381`): 86x86.
            NpcKind::Plantera => (86.0, 86.0),
            // `SetDefaults` case 266 (`NPC.cs:12440`): 160x110.
            NpcKind::BrainOfCthulhu => (160.0, 110.0),
            // `SetDefaults` case 267 (`NPC.cs:12458`): 30x30.
            NpcKind::Creeper => (30.0, 30.0),
            // The goblins: 18x38 x the scale.
            NpcKind::GoblinPeon => (16.0, 34.0),
            NpcKind::GoblinThief => (17.0, 36.0),
            NpcKind::GoblinWarrior => (20.0, 42.0),
            NpcKind::GoblinArcher => (17.0, 36.0),
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
            // The casters: `SetDefaults` case 29 is 18x38, the cases
            // 45/32/24 18x40 (`NPC.cs:8951`/`:9322`/`:9121`/`:9078`).
            NpcKind::GoblinSorcerer => (18.0, 38.0),
            NpcKind::Tim => (18.0, 40.0),
            NpcKind::DarkCaster => (18.0, 40.0),
            NpcKind::FireImp => (18.0, 40.0),
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
            NpcKind::CaveBat => (22.0, 18.0),
            NpcKind::KingSlime => (98.0, 92.0),
            // `NPC.cs:8705` (the eye 100×110) / `:8717` (the servant 20×20).
            NpcKind::EyeOfCthulhu => (100.0, 110.0),
            NpcKind::ServantOfCthulhu => (20.0, 20.0),
            // `NPC.cs`'s 115 (30×30) / 117-118 (14×14).
            NpcKind::Hungry => (30.0, 30.0),
            NpcKind::LeechHead | NpcKind::LeechBody => (14.0, 14.0),
        }
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
            // `SetDefaults` case 4 (`NPC.cs:8710`): 2800 HP.
            NpcKind::EyeOfCthulhu => 2800,
            // `SetDefaults` case 222 (`NPC.cs:11862`): 3400 HP.
            NpcKind::QueenBee => 3400,
            // `SetDefaults` case 35 (`NPC.cs:9172`): 4400 HP.
            NpcKind::SkeletronHead => 4400,
            // `SetDefaults` case 113 (`NPC.cs:10348`): 8000 HP.
            NpcKind::WallOfFlesh => 8000,
            // `NPC.cs`'s 115 (240) / 117-118 (60).
            NpcKind::Hungry => 240,
            NpcKind::LeechHead | NpcKind::LeechBody => 60,
            // `SetDefaults` case 134 (`NPC.cs:10666`): 8000 HP (the
            // vanilla's realLife pool — the v1 carries the per-segment HP).
            NpcKind::TheDestroyer => 8000,
            // `SetDefaults` case 127 (`NPC.cs:10562`): 28000 HP.
            NpcKind::SkeletronPrime => 28000,
            // `SetDefaults` cases 125/126.
            NpcKind::Retinazer => 20000,
            NpcKind::Spazmatism => 23000,
            // `SetDefaults` case 262 (`NPC.cs:12385`): 30000 HP.
            NpcKind::Plantera => 30000,
            // `SetDefaults` cases 266/267.
            NpcKind::BrainOfCthulhu => 1250,
            NpcKind::Creeper => 100,
            // The goblins.
            NpcKind::GoblinPeon => 60,
            NpcKind::GoblinThief => 80,
            NpcKind::GoblinWarrior => 110,
            NpcKind::GoblinArcher => 80,
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
            // The casters: `SetDefaults` cases 29/45/32/24.
            NpcKind::GoblinSorcerer => 40,
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
            NpcKind::KingSlime => 2000,
            // `NPC.cs:8710` (the eye) / `:8722` (the servant).
            NpcKind::EyeOfCthulhu => 2800,
            NpcKind::ServantOfCthulhu => 8,
        }
    }

    fn contact_damage(self) -> i32 {
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
            NpcKind::EyeOfCthulhu => 15,
            NpcKind::QueenBee => 30,
            NpcKind::SkeletronHead => 32,
            NpcKind::WallOfFlesh => 50,
            // `NPC.cs`'s 115 (30) / 117 (26) / 118 (22).
            NpcKind::Hungry => 30,
            NpcKind::LeechHead => 26,
            NpcKind::LeechBody => 22,
            NpcKind::TheDestroyer => 70,
            NpcKind::SkeletronPrime => 47,
            NpcKind::Retinazer => 45,
            NpcKind::Spazmatism => 50,
            NpcKind::Plantera => 50,
            NpcKind::BrainOfCthulhu => 30,
            NpcKind::Creeper => 20,
            NpcKind::GoblinPeon => 12,
            NpcKind::GoblinThief => 20,
            NpcKind::GoblinWarrior => 25,
            NpcKind::GoblinArcher => 20,
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
            // The casters: `SetDefaults` damage 20/20/20/30.
            NpcKind::GoblinSorcerer => 20,
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
            NpcKind::Guide => 0,
            NpcKind::KingSlime => 40,
            // `NPC.cs:8707` (the eye damage 15) / `:8720` (the servant 12).
            NpcKind::EyeOfCthulhu => 15,
            NpcKind::ServantOfCthulhu => 12,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Npc {
    pub kind: NpcKind,
    /// The slime colour variant (`SlimeVariant`); only meaningful while
    /// `kind == NpcKind::Slime`.
    pub slime: SlimeVariant,
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub hp: i32,
    pub max_hp: i32,
    pub on_ground: bool,
    pub facing: i32,
    pub hop_timer: f32,
    pub anim: f32,
    pub hurt_cd: f32,
    /// `NPC.immune[256]`: per-owner i-frame ticks, applied by projectile hits
    /// (`Projectile.cs:14087-14119`) and decremented once per NPC update
    /// (`NPC.cs:90454-90459`).
    pub immune: [i32; 256],
    /// `NPC.SpawnedFromStatue` (`NPC.cs:6061`): set by the wire-triggered
    /// statue spawns; gates the loot (`NPCLoot`'s StatueSpawnedDropRarity
    /// check, `NPC.cs:78849`).
    pub spawned_from_statue: bool,
    /// `NPC.buffType[20]` / `buffTime[20]` / `buffImmune` — the debuff engine
    /// (`npc_buffs.rs`, `NPC.cs:6193-6197`); the DoT tick runs from
    /// `Npc::update` like the C# buff block of `UpdateNPC`.
    pub buffs: NpcBuffs,
    /// `NPC.frame`: the rendered frame row, advanced by [`Npc::find_frame`] in
    /// the fixed step (the original advances animation in the sim too —
    /// `FindFrame` runs from `NPC.Update`, `NPC.cs:66684`).
    pub frame: u32,
    /// `NPC.frameCounter`: the per-kind accumulator `FindFrame` checks.
    frame_counter: f32,
    /// `NPC.rotation`: the aim tilt. Only the demon eye sets it
    /// (`NPC.cs:74019-74031`: `atan2(velocity.Y, velocity.X)`, `+ π` when
    /// `velocity.X < 0`).
    pub rotation: f32,
    /// Accumulated daylight burn time (zombies only).
    burn: f32,
    /// A boss that just landed wants the app to spawn a minion (`AI_015`).
    pub wants_minion: bool,
    /// The town NPC's assigned home tile (`homeTileX/Y`); `None` =
    /// homeless (`NPC.homeless` inverted). The in-memory state — the
    /// world-file persistence lands with M6-SAVE-1's NPC sections.
    pub home: Option<(i32, i32)>,
    /// `ai[0]`: the caster's cycle counter (`AI_008`'s 500 init, the
    /// 100/200/300 shot triggers, the 650 teleport threshold).
    pub caster_counter: i32,
    /// `ai[1]`: the caster's windup countdown (30 from a shot trigger,
    /// `num88` after a teleport; the shoot fires at the params' tick).
    pub cast_windup: i32,
    /// `ai[0]` for the swimmers (`AI_016`): the vertical bob phase — 0 the
    /// chase start, −1 rising, 1 sinking (`:23752`/`:23758`/`:23850`).
    pub swim_phase: f32,
    /// The worm chain id (`AI_006`): every segment of one worm shares it;
    /// 0 = not a worm.
    pub worm_id: u32,
    /// The segment's index down the chain — 0 = the head (`ai[1] == 0`'s
    /// leader), the tail = the last.
    pub seg_index: u32,
    /// `ai[2]`: the remaining segment budget this link passes down (the
    /// head rolls the total, each link decrements).
    pub worm_left: u32,
    /// The init latch — `ai[0] == 0`'s "no child spawned yet" analog: the
    /// link spawns its child on the first worm pass.
    pub worm_inited: bool,
    /// The WoF's Leech-head spawn request (the tongue-phase
    /// `NPC.CountNPCS(117) < 10` spawn; the sim counts and spawns).
    pub wants_leech: bool,
    /// The WoF's Hungry init request (`localAI[0]` 1 → 2's 11 spawns).
    pub wants_hungry: bool,
    /// The AI's per-tick contact-damage override (the Hungry's
    /// damage tiers, `aiStyle 29`'s `num388`).
    pub damage_override: Option<i32>,
    /// The WoF's draw-band scan targets (`AI_027`'s bottom/top walks) —
    /// the world sim eases `World.wof_draw_*` toward them (the AI holds
    /// `&World`, so the band state lives on the sim side).
    pub wof_bottom_target: f32,
    pub wof_top_target: f32,
    /// The caster's sphere spawn request — the vanilla `NPC.NewNPC` at the
    /// shoot tick (`AI_008`), resolved by the world sim after the update
    /// pass (the borrow-checker-safe `wants_minion` shape). The payload is
    /// the (kind, bottom-centre x, bottom-centre y) spawn point.
    pub wants_sphere: Option<(NpcKind, f32, f32)>,
    /// `ai[3]`: x position while descending through water, for the bounce flip.
    wet_x: f32,
    /// `NPC.ai[0..4]` — the EoC phase machine's state
    /// (`ai[0]` the phase, `ai[1]` the sub-state, `ai[2]` the timer,
    /// `ai[3]` the dash/servant counter; `AI()`'s aiStyle-4 block).
    pub ai: [f32; 4],
    /// The servant spawn request — the `NPC.NewNPC(..., 5)` calls in the
    /// aiStyle-4 block (`NPC.cs:4598`/`:4651` of the AI()), resolved by the
    /// world sim after the update pass (the `wants_sphere` shape; the payload
    /// is the (x, y, vx, vy) spawn state).
    pub wants_servant: Option<(f32, f32, f32, f32)>,
    /// `this.defense = 0` — the AI's per-frame stat override (the EoC's
    /// final phase zeroes it, `NPC.cs`'s aiStyle-4 block; the expert rage
    /// makes it negative).
    pub defense_override: Option<i32>,
}

impl Npc {
    pub fn spawn_slime(world: &World, x_tile: i32) -> Option<Self> {
        Self::spawn_slime_variant(world, x_tile, SlimeVariant::Blue)
    }

    /// Spawns one slime variant: the hitbox is the base 24x18 scaled by the
    /// variant's `SetDefaults_ForNetId` size scale, at the surface.
    pub fn spawn_slime_variant(world: &World, x_tile: i32, variant: SlimeVariant) -> Option<Self> {
        let top = world.surface_of(x_tile);
        if top >= world.height - 2 {
            return None;
        }
        let stats = variant.stats();
        let scale = stats.scale;
        let (w, h) = (
            NpcKind::Slime.size().0 * scale,
            NpcKind::Slime.size().1 * scale,
        );
        let x = x_tile as f32 * TILE_PX + 8.0 - w * 0.5;
        let y = top as f32 * TILE_PX - h;
        if collides_rect(world, x, y, w, h) {
            return None;
        }
        if world.tile(x_tile, top).is_some_and(|t| t.liquid > 128) {
            return None;
        }
        let mut seed = (x_tile as u32).wrapping_mul(2_654_435_761);
        seed ^= seed >> 15;
        let mut npc = Self::new(
            NpcKind::Slime,
            x,
            y,
            if seed & 1 == 0 { 1 } else { -1 },
            0.4 + (seed % 80) as f32 * 0.01,
        );
        npc.slime = variant;
        npc.max_hp = stats.life;
        npc.hp = stats.life;
        Some(npc)
    }

    /// `NPCID.KingSlime`: the first boss, spawned by the `B` debug key or `TR_BOSS`.
    pub fn spawn_king_slime(world: &World, x_tile: i32) -> Option<Self> {
        let top = world.surface_of(x_tile);
        if top >= world.height - 2 {
            return None;
        }
        let (w, h) = NpcKind::KingSlime.size();
        let x = x_tile as f32 * TILE_PX + 8.0 - w * 0.5;
        let y = top as f32 * TILE_PX - h;
        if collides_rect(world, x, y, w, h) {
            return None;
        }
        let mut npc = Self::new(NpcKind::KingSlime, x, y, -1, 1.2);
        npc.hp = npc.max_hp;
        Some(npc)
    }

    /// `NPC.SpawnOnPlayer(plr, 4)` (`NPC.cs:80475+`): the boss spawn scan
    /// looks for open space around the player's tile; the flying,
    /// no-tile-collide eye only needs an in-bounds spot above the player —
    /// the AI flies to the target from wherever it appears (the tile-scan
    /// fidelity is the M4-BOSS polish follow-up).
    pub fn spawn_eye_of_cthulhu(world: &World, player_x: f32, player_y: f32) -> Option<Self> {
        let (w, h) = NpcKind::EyeOfCthulhu.size();
        // SpawnBoss' tile-anchored position: the player's centre column,
        // one screen above (the vanilla scan prefers off-screen space).
        let x = player_x - w * 0.5;
        // Clamped into the world: near the surface the 600-px lift would
        // leave it; the AI flies to the target from wherever it appears.
        let y = (player_y - 600.0 - h * 0.5).max(16.0);
        if x < 0.0
            || y < 0.0
            || x + w > world.width as f32 * TILE_PX
            || y + h > world.height as f32 * TILE_PX
        {
            return None;
        }
        let mut npc = Self::new(NpcKind::EyeOfCthulhu, x, y, -1, 0.0);
        npc.hp = npc.max_hp;
        Some(npc)
    }

    /// The aiStyle-4 block's `NPC.NewNPC(..., 5)` minion spawn
    /// (`NPC.cs:4598` of the AI()): the servant appears 10 px out along the
    /// aim and inherits the launch velocity (`vector3.X/Y`).
    pub fn spawn_servant_of_cthulhu(x: f32, y: f32, vx: f32, vy: f32) -> Self {
        let mut npc = Self::new(NpcKind::ServantOfCthulhu, x - 10.0, y - 10.0, 1, 0.0);
        npc.vx = vx;
        npc.vy = vy;
        npc
    }

    /// Zombies walk on the surface and burn away at dawn.
    /// Spawns a cave walker at the cavern layer's surface (the vanilla
    /// cave spawns pick the open-air tiles below the rock layer).
    pub fn spawn_cave_walker(world: &World, x_tile: i32, kind: NpcKind) -> Option<Self> {
        let top = world.surface_of(x_tile);
        if top >= world.height - 2 {
            return None;
        }
        let (w, h) = kind.size();
        let x = x_tile as f32 * TILE_PX + 8.0 - w * 0.5;
        let y = top as f32 * TILE_PX - h;
        if collides_rect(world, x, y, w, h) {
            return None;
        }
        if world.tile(x_tile, top).is_some_and(|t| t.liquid > 128) {
            return None;
        }
        Some(Self::new(
            kind,
            x,
            y,
            if (x_tile % 2) as i32 == 0 { 1 } else { -1 },
            0.0,
        ))
    }

    /// Spawns a caster standing on the found ground tile (`SpawnNPC`'s
    /// `Bottom = (x * 16 + 8, y * 16)` semantics, `NPC.cs:4934`): the feet
    /// on the ground tile's top, the X centred on the tile.
    pub fn spawn_caster(world: &World, x_tile: i32, ground_y: i32, kind: NpcKind) -> Option<Self> {
        let (w, h) = kind.size();
        let x = x_tile as f32 * TILE_PX + 8.0 - w * 0.5;
        let y = ground_y as f32 * TILE_PX - h;
        if collides_rect(world, x, y, w, h) {
            return None;
        }
        Some(Self::new(
            kind,
            x,
            y,
            if (x_tile % 2) as i32 == 0 { 1 } else { -1 },
            0.0,
        ))
    }

    /// Spawns a worm head standing on the found ground tile (`SpawnNPC`'s
    /// `Bottom` placement — the chain assembles from the head's first worm
    /// pass: each link spawns its child at the parent's bottom-centre,
    /// `NPC.cs:51347`).
    pub fn spawn_worm_head(world: &World, x_tile: i32, ground_y: i32, kind: NpcKind) -> Self {
        let (w, h) = kind.size();
        let x = x_tile as f32 * TILE_PX + 8.0 - w * 0.5;
        let y = ground_y as f32 * TILE_PX - h;
        Self::new(kind, x, y, 1, 0.0)
    }

    /// Spawns a swimmer standing on the found ground tile inside its water
    /// column (`SpawnNPC`'s `Bottom = (x*16+8, y*16)` — the same placement
    /// the casters ride).
    pub fn spawn_swimmer(world: &World, x_tile: i32, ground_y: i32, kind: NpcKind) -> Option<Self> {
        Self::spawn_caster(world, x_tile, ground_y, kind)
    }

    /// Spawns a cave flyer hanging in the open air of the cave layer (the
    /// vanilla bat spawn picks the open tiles below the rock layer — the
    /// `SpawnAnNPC` cave arm, `NPC.cs:5233`).
    /// Spawns a hover in the open air of its layer (`NPC.NewNPC`).
    pub fn spawn_hover(world: &World, x_tile: i32, y_tile: i32, kind: NpcKind) -> Option<Self> {
        let (w, h) = kind.size();
        let x = x_tile as f32 * TILE_PX + 8.0 - w * 0.5;
        let y = y_tile as f32 * TILE_PX;
        if collides_rect(world, x, y, w, h) {
            return None;
        }
        if world.tile(x_tile, y_tile).is_some_and(|t| t.active()) {
            return None;
        }
        Some(Self::new(kind, x, y, 1, 0.0))
    }

    pub fn spawn_cave_flyer(
        world: &World,
        x_tile: i32,
        y_tile: i32,
        kind: NpcKind,
    ) -> Option<Self> {
        let (w, h) = kind.size();
        let x = x_tile as f32 * TILE_PX + 8.0 - w * 0.5;
        let y = y_tile as f32 * TILE_PX;
        if collides_rect(world, x, y, w, h) {
            return None;
        }
        if world.tile(x_tile, y_tile).is_some_and(|t| t.active()) {
            return None;
        }
        if !world.in_bounds(x_tile, (y / TILE_PX) as i32 + 1) {
            return None;
        }
        Some(Self::new(
            kind,
            x,
            y,
            if (x_tile % 2) as i32 == 0 { 1 } else { -1 },
            0.0,
        ))
    }

    /// Spawns the Guide homeless at the surface tile (`NPC.NewNPC`'s
    /// world-gen arrival).
    /// The town spawn ctor — the Guide and the Merchant share every stat
    /// (250 HP / 10 dmg / 15 def / 0.5 kb / 18x40), so one body serves both
    /// (`SetDefaults` cases 17/22).
    pub fn spawn_town(world: &World, x_tile: i32, kind: NpcKind) -> Option<Self> {
        let mut npc = Self::spawn_guide(world, x_tile)?;
        npc.kind = kind;
        npc.max_hp = kind.max_hp();
        npc.hp = npc.max_hp;
        Some(npc)
    }

    pub fn spawn_guide(world: &World, x_tile: i32) -> Option<Self> {
        let top = world.surface_of(x_tile);
        if top >= world.height - 2 {
            return None;
        }
        let (w, h) = NpcKind::Guide.size();
        let x = x_tile as f32 * TILE_PX + 8.0 - w * 0.5;
        let y = top as f32 * TILE_PX - h;
        if collides_rect(world, x, y, w, h) {
            return None;
        }
        Some(Self::new(NpcKind::Guide, x, y, 1, 0.0))
    }

    pub fn spawn_zombie(world: &World, x_tile: i32) -> Option<Self> {
        let top = world.surface_of(x_tile);
        if top >= world.height - 2 {
            return None;
        }
        let (w, h) = NpcKind::Zombie.size();
        let x = x_tile as f32 * TILE_PX + 8.0 - w * 0.5;
        let y = top as f32 * TILE_PX - h;
        if collides_rect(world, x, y, w, h) {
            return None;
        }
        if world.tile(x_tile, top).is_some_and(|t| t.liquid > 128) {
            return None;
        }
        Some(Self::new(NpcKind::Zombie, x, y, -1, 0.0))
    }

    /// Demon eyes spawn a few tiles above the surface and ignore gravity.
    pub fn spawn_eye(world: &World, x_tile: i32) -> Option<Self> {
        let top = world.surface_of(x_tile);
        if top >= world.height - 2 {
            return None;
        }
        let (w, h) = NpcKind::DemonEye.size();
        let x = x_tile as f32 * TILE_PX + 8.0 - w * 0.5;
        let y = top as f32 * TILE_PX - h - 4.0 * TILE_PX;
        if collides_rect(world, x, y, w, h) {
            return None;
        }
        Some(Self::new(NpcKind::DemonEye, x, y, 1, 0.0))
    }

    pub(crate) fn new(kind: NpcKind, x: f32, y: f32, facing: i32, hop_timer: f32) -> Self {
        Self {
            kind,
            slime: SlimeVariant::Blue,
            x,
            y,
            vx: 0.0,
            vy: 0.0,
            hp: kind.max_hp(),
            max_hp: kind.max_hp(),
            on_ground: false,
            facing,
            hop_timer,
            anim: 0.0,
            hurt_cd: 0.0,
            immune: [0; 256],
            spawned_from_statue: false,
            buffs: NpcBuffs::for_kind(kind),
            frame: 0,
            frame_counter: 0.0,
            rotation: 0.0,
            burn: 0.0,
            wants_minion: false,
            home: None,
            caster_counter: 0,
            cast_windup: 0,
            wants_sphere: None,
            swim_phase: 0.0,
            worm_id: 0,
            seg_index: 0,
            worm_left: 0,
            worm_inited: false,
            wants_leech: false,
            wants_hungry: false,
            damage_override: None,
            wof_bottom_target: -1.0,
            wof_top_target: -1.0,
            wet_x: 0.0,
            ai: [0.0; 4],
            wants_servant: None,
            defense_override: None,
        }
    }

    pub fn center(&self) -> (f32, f32) {
        let (w, h) = self.size();
        (self.x + w * 0.5, self.y + h * 0.5)
    }

    /// `NPC.knockBackResist` — `StrikeNPC_Inner`'s soft-cap chain input
    /// (blue `NPC.cs:8595` 0.8; demon eye 0.8, Wandering Eye 0.8,
    /// zombie 0.5, King Slime 0; the slime variants multiply the blue
    /// base per `SetDefaultsFromNetId`: green x1.2, pinky x1.4, baby
    /// x0.95, purple x0.9, black/red/yellow x1).
    pub fn knock_back_resist(&self) -> f32 {
        if self.kind == NpcKind::Slime {
            let mult = match self.slime {
                SlimeVariant::Blue
                | SlimeVariant::Black
                | SlimeVariant::Red
                | SlimeVariant::Yellow => 1.0,
                SlimeVariant::Green => 1.2,
                SlimeVariant::Pinky => 1.4,
                SlimeVariant::Baby => 0.95,
                SlimeVariant::Purple => 0.9,
            };
            return 0.8 * mult;
        }
        match self.kind {
            NpcKind::DemonEye => 0.8,
            NpcKind::WanderingEye => 0.8,
            NpcKind::Zombie => 0.5,
            // `SetDefaults` case 21/44: 0.5; case 49: 0.8.
            NpcKind::Skeleton => 0.5,
            NpcKind::UndeadMiner => 0.5,
            NpcKind::CaveBat => 0.8,
            // `SetDefaults` case 51/60/66: knockBackResist 0.8.
            NpcKind::JungleBat => 0.8,
            NpcKind::Hellbat => 0.8,
            NpcKind::VoodooDemon => 0.8,
            // `SetDefaults` case 6: 0.5; case 42: 0.5; case 23: 0.4.
            NpcKind::EaterOfSouls => 0.5,
            NpcKind::Hornet => 0.5,
            NpcKind::MeteorHead => 0.4,
            // The casters: `SetDefaults` knockBackResist 0.6/0.6/0.6/0.5.
            NpcKind::GoblinSorcerer => 0.6,
            NpcKind::Tim => 0.6,
            NpcKind::DarkCaster => 0.6,
            NpcKind::FireImp => 0.5,
            // The spheres: 0 (the knockback passes through).
            NpcKind::ChaosBall => 0.0,
            NpcKind::BurningSphere => 0.0,
            NpcKind::WaterSphere => 0.0,
            // The swimmers: the Goldfish 0.5 (`:9471`), the Shark 0.7
            // (`:9622`), the Piranha no row — the `SetDefaults` head's 1f
            // reset (`:8533`).
            NpcKind::Goldfish => 0.5,
            NpcKind::Piranha => 1.0,
            NpcKind::Shark => 0.7,
            // The worms: knockBackResist 0 (`:8763` family-wide).
            NpcKind::DevourerHead
            | NpcKind::DevourerBody
            | NpcKind::DevourerTail
            | NpcKind::GiantWormHead
            | NpcKind::GiantWormBody
            | NpcKind::GiantWormTail
            | NpcKind::EaterOfWorldsHead
            | NpcKind::EaterOfWorldsBody
            | NpcKind::EaterOfWorldsTail => 0.0,
            // The town NPCs barely knock (`NPC.SetDefaults` 22: 0.5).
            NpcKind::Guide => 0.5,
            // `SetDefaults` case 17: knockBackResist 0.5.
            NpcKind::Merchant => 0.5,
            NpcKind::Dryad => 0.5,
            NpcKind::Nurse => 0.5,
            NpcKind::ArmsDealer => 0.5,
            NpcKind::Demolitionist => 0.5,
            NpcKind::Painter => 0.5,
            NpcKind::WitchDoctor => 0.5,
            NpcKind::Clothier => 0.5,
            NpcKind::EyeOfCthulhu => 0.0,
            NpcKind::QueenBee => 0.0,
            NpcKind::SkeletronHead => 0.0,
            NpcKind::WallOfFlesh => 0.0,
            NpcKind::TheDestroyer => 0.0,
            // `SetDefaults` case 127: knockBackResist 0.
            NpcKind::SkeletronPrime => 0.0,
            NpcKind::Retinazer => 0.0,
            NpcKind::Spazmatism => 0.0,
            NpcKind::Plantera => 0.0,
            NpcKind::BrainOfCthulhu => 0.45,
            NpcKind::Creeper => 0.8,
            NpcKind::GoblinPeon => 0.8,
            NpcKind::GoblinThief => 0.7,
            NpcKind::GoblinWarrior => 0.5,
            NpcKind::GoblinArcher => 0.7,
            NpcKind::BoundGoblin => 0.5,
            NpcKind::GoblinTinkerer => 0.5,
            NpcKind::SkeletronPrime => 0.0,
            NpcKind::Retinazer => 0.0,
            NpcKind::Spazmatism => 0.0,
            NpcKind::KingSlime => 0.0,
            // 115/117/118: knockBackResist 0.
            NpcKind::Hungry | NpcKind::LeechHead | NpcKind::LeechBody => 0.0,
            // The eye/servant blocks set no knockBackResist — the
            // SetDefaults head's default applies.
            NpcKind::EyeOfCthulhu => 1.0,
            NpcKind::ServantOfCthulhu => 1.0,
            NpcKind::Slime => unreachable!(),
        }
    }

    /// `NPC.defense` — `Main.CalculateDamageNPCsTake`'s input
    /// (blue `NPC.cs:8573` 2, demon eye 2, Wandering Eye 20, zombie 6,
    /// King Slime 10; the slime variants ride their `SetDefaultsFromNetId`
    /// lines).
    pub fn defense(&self) -> i32 {
        if let Some(d) = self.defense_override {
            return d;
        }
        if self.kind == NpcKind::Slime {
            return self.slime.stats().defense;
        }
        match self.kind {
            NpcKind::DemonEye => 2,
            NpcKind::WanderingEye => 20,
            NpcKind::Zombie => 6,
            // `SetDefaults` case 21: defense 8; case 44: 9; case 49: 2.
            NpcKind::Skeleton => 8,
            NpcKind::UndeadMiner => 9,
            NpcKind::CaveBat => 2,
            // `SetDefaults` case 51: defense 4; case 60: 8; case 66: 8.
            NpcKind::JungleBat => 4,
            NpcKind::Hellbat => 8,
            NpcKind::VoodooDemon => 8,
            // `SetDefaults` case 6: defense 8; case 42: 12; case 23: 6.
            NpcKind::EaterOfSouls => 8,
            NpcKind::Hornet => 12,
            NpcKind::MeteorHead => 6,
            // The casters: `SetDefaults` defense 2/4/2/16.
            NpcKind::GoblinSorcerer => 2,
            NpcKind::Tim => 4,
            NpcKind::DarkCaster => 2,
            NpcKind::FireImp => 16,
            // The spheres: defense 0.
            NpcKind::ChaosBall => 0,
            NpcKind::BurningSphere => 0,
            NpcKind::WaterSphere => 0,
            // The swimmers: defense 0/2/2.
            NpcKind::Goldfish => 0,
            NpcKind::Piranha => 2,
            NpcKind::Shark => 2,
            // The worms: the head 2/0/2, the body 6/4/4, the tail 10/6/8.
            NpcKind::DevourerHead => 2,
            NpcKind::DevourerBody => 6,
            NpcKind::DevourerTail => 10,
            NpcKind::GiantWormHead => 0,
            NpcKind::GiantWormBody => 4,
            NpcKind::GiantWormTail => 6,
            NpcKind::EaterOfWorldsHead => 2,
            NpcKind::EaterOfWorldsBody => 4,
            NpcKind::EaterOfWorldsTail => 8,
            NpcKind::Guide => 15,
            // `SetDefaults` case 17: defense 15.
            NpcKind::Merchant => 15,
            NpcKind::Dryad => 15,
            NpcKind::Nurse => 15,
            NpcKind::ArmsDealer => 15,
            NpcKind::Demolitionist => 15,
            NpcKind::Painter => 15,
            NpcKind::WitchDoctor => 15,
            NpcKind::Clothier => 15,
            NpcKind::EyeOfCthulhu => 12,
            NpcKind::QueenBee => 8,
            NpcKind::SkeletronHead => 10,
            NpcKind::WallOfFlesh => 12,
            NpcKind::TheDestroyer => 0,
            NpcKind::SkeletronPrime => 24,
            NpcKind::Retinazer => 10,
            NpcKind::Spazmatism => 10,
            NpcKind::Plantera => 14,
            NpcKind::BrainOfCthulhu => 14,
            NpcKind::Creeper => 10,
            NpcKind::GoblinPeon => 4,
            NpcKind::GoblinThief => 6,
            NpcKind::GoblinWarrior => 8,
            NpcKind::GoblinArcher => 6,
            // def 15 both (`:10254`, `:10279`).
            NpcKind::BoundGoblin => 15,
            NpcKind::GoblinTinkerer => 15,
            NpcKind::KingSlime => 10,
            // `NPC.cs`'s 115 (10) / 117 (2) / 118 (6) — the hungry's
            // per-tick tiers ride `defense_override`.
            NpcKind::Hungry => 10,
            NpcKind::LeechHead => 2,
            NpcKind::LeechBody => 6,
            // `NPC.cs:8709` (the eye defense 12; the AI zeroes it in the
            // final phase) / `:8721` (the servant 0).
            NpcKind::EyeOfCthulhu => 12,
            NpcKind::ServantOfCthulhu => 0,
            NpcKind::Slime => unreachable!(),
        }
    }

    /// `NPC.value` in copper — `NPCLoot_DropMoney`'s base
    /// (blue `NPC.cs:8576` 25, demon eye 75, Wandering Eye 500, zombie 60,
    /// King Slime 10000).
    pub fn value(&self) -> f32 {
        if self.kind == NpcKind::Slime {
            return self.slime.stats().value;
        }
        match self.kind {
            NpcKind::DemonEye => 75.0,
            NpcKind::WanderingEye => 500.0,
            NpcKind::Zombie => 60.0,
            // `SetDefaults` case 21: value 100; case 44: 200; case 49: 60.
            NpcKind::Skeleton => 100.0,
            NpcKind::UndeadMiner => 200.0,
            NpcKind::CaveBat => 60.0,
            // `SetDefaults` case 51: value 80; case 60: 120; case 66: 300.
            NpcKind::JungleBat => 80.0,
            NpcKind::Hellbat => 120.0,
            NpcKind::VoodooDemon => 300.0,
            // `SetDefaults` case 6: value 150; case 42: 200; case 23: 250.
            NpcKind::EaterOfSouls => 150.0,
            NpcKind::Hornet => 200.0,
            NpcKind::MeteorHead => 250.0,
            // The casters: `SetDefaults` value 200/5000/140/350.
            NpcKind::GoblinSorcerer => 200.0,
            NpcKind::Tim => 5000.0,
            NpcKind::DarkCaster => 140.0,
            NpcKind::FireImp => 350.0,
            // The spheres carry no value.
            NpcKind::ChaosBall => 0.0,
            NpcKind::BurningSphere => 0.0,
            NpcKind::WaterSphere => 0.0,
            // The swimmers: the Piranha 50 (`:9514`), the Shark 400
            // (`:9621`), the Goldfish no row (the catch item 261 rides the
            // critter-catch work).
            NpcKind::Goldfish => 0.0,
            NpcKind::Piranha => 50.0,
            NpcKind::Shark => 400.0,
            // The worms: the per-segment coin value (`:8763`'s 140, `:8850`'s
            // 40, `:8903`'s 800 — the body/tail rows carry the same value).
            NpcKind::DevourerHead | NpcKind::DevourerBody | NpcKind::DevourerTail => 140.0,
            NpcKind::GiantWormHead | NpcKind::GiantWormBody | NpcKind::GiantWormTail => 40.0,
            NpcKind::EaterOfWorldsHead
            | NpcKind::EaterOfWorldsBody
            | NpcKind::EaterOfWorldsTail => 800.0,
            NpcKind::Guide => 0.0,
            // The town NPCs carry no coin value.
            NpcKind::Merchant => 0.0,
            NpcKind::Dryad => 0.0,
            NpcKind::Nurse => 0.0,
            NpcKind::ArmsDealer => 0.0,
            NpcKind::Demolitionist => 0.0,
            NpcKind::Painter => 0.0,
            NpcKind::WitchDoctor => 0.0,
            NpcKind::Clothier => 0.0,
            // `SetDefaults` case 4: value 30000 (the coin drop).
            NpcKind::EyeOfCthulhu => 30000.0,
            NpcKind::QueenBee => 50000.0,
            NpcKind::SkeletronHead => 50000.0,
            NpcKind::WallOfFlesh => 80000.0,
            NpcKind::TheDestroyer => 120000.0,
            NpcKind::SkeletronPrime => 120000.0,
            NpcKind::Retinazer => 120000.0,
            NpcKind::Spazmatism => 120000.0,
            NpcKind::Plantera => 150000.0,
            NpcKind::BrainOfCthulhu => 50000.0,
            NpcKind::Creeper => 0.0,
            NpcKind::GoblinPeon => 100.0,
            NpcKind::GoblinThief => 200.0,
            NpcKind::GoblinWarrior => 150.0,
            NpcKind::GoblinArcher => 200.0,
            NpcKind::BoundGoblin => 0.0,
            NpcKind::GoblinTinkerer => 0.0,
            NpcKind::KingSlime => 10_000.0,
            // The hungry/leech rows carry no value line.
            NpcKind::Hungry | NpcKind::LeechHead | NpcKind::LeechBody => 0.0,
            // `NPC.cs:8713` (the eye value 30000); the servant has no row.
            NpcKind::EyeOfCthulhu => 30_000.0,
            NpcKind::ServantOfCthulhu => 0.0,
            NpcKind::Slime => unreachable!(),
        }
    }

    /// The scaled size for slime variants (the hitbox rides
    /// `SetDefaults_ForNetId`'s size scale).
    pub fn size(&self) -> (f32, f32) {
        let (w, h) = self.kind.size();
        if self.kind == NpcKind::Slime {
            let s = self.slime.stats().scale;
            (w * s, h * s)
        } else {
            (w, h)
        }
    }

    pub fn contains(&self, wx: f32, wy: f32) -> bool {
        let (w, h) = self.size();
        wx >= self.x - 4.0 && wx <= self.x + w + 4.0 && wy >= self.y - 4.0 && wy <= self.y + h + 4.0
    }

    /// `NPC.StrikeNPC`'s knockback half (`NPC.cs:81389-81458`): the
    /// soft-capped impulse applied by `tr_core::damage::knock_back_npc`.
    /// `hit_direction` is the knockback side (+1 = pushed right) and
    /// `knock_back` the weapon's `Item.knockBack`.
    pub fn hurt(&mut self, amount: i32, from_x: f32, knock_back: f32, crit: bool) -> bool {
        if self.hurt_cd > 0.0 || self.hp <= 0 {
            return false;
        }
        self.hp -= amount;
        self.hurt_cd = 0.18;
        let (w, _) = self.size();
        let dir = if self.x + w * 0.5 >= from_x { 1 } else { -1 };
        let (vx, vy) = tr_core::damage::knock_back_npc(
            knock_back,
            self.knock_back_resist(),
            crit,
            amount,
            self.max_hp,
            (self.vx, self.vy),
            dir,
            false,
            false,
        );
        self.vx = vx;
        self.vy = vy;
        self.on_ground = false;
        true
    }

    /// The projectile-hit path: identical to [`Npc::hurt`] but without the
    /// shared `hurt_cd` gate — a projectile's right to hit is decided by the
    /// three C# immunity systems (`Projectile.cs:12577-12624`), not by the
    /// melee cooldown stand-in. The hit still arms `hurt_cd`, so melee is
    /// gated afterward, matching `StrikeNPC`'s 10-tick `NPC.immune` window.
    pub fn hurt_from_projectile(
        &mut self,
        amount: i32,
        from_x: f32,
        knock_back: f32,
        crit: bool,
    ) -> bool {
        if self.hp <= 0 {
            return false;
        }
        self.hp -= amount;
        self.hurt_cd = 0.18;
        let (w, _) = self.size();
        let dir = if self.x + w * 0.5 >= from_x { 1 } else { -1 };
        let (vx, vy) = tr_core::damage::knock_back_npc(
            knock_back,
            self.knock_back_resist(),
            crit,
            amount,
            self.max_hp,
            (self.vx, self.vy),
            dir,
            false,
            false,
        );
        self.vx = vx;
        self.vy = vy;
        self.on_ground = false;
        true
    }

    pub fn update(&mut self, world: &World, player: &mut Player, dt: f32) {
        // The deterministic scratch stream — the tests and every
        // non-caster kind never consume it (only the caster teleport
        // search draws `rand01`).
        let mut scratch = 0u64;
        self.update_rng(world, player, dt, &mut scratch);
    }

    /// The real step with the caller's RNG stream — the caster teleport
    /// search draws from it where the C# hits `Main.rand` (`NPC.cs:18948`).
    pub fn update_rng(&mut self, world: &World, player: &mut Player, dt: f32, rng: &mut u64) {
        // `NPC.Update`'s buff block (`NPC.cs:90373-90377`): timer decrement →
        // clear expired → ApplyDOTs, before the AI — DoT damage lands ahead of
        // the frame's movement, exactly like the original.
        self.buffs.update_buffs_tick(&mut self.hp, self.max_hp);
        self.hurt_cd = (self.hurt_cd - dt).max(0.0);
        // `NPC.Update`'s per-owner i-frame decay (`NPC.cs:90454-90459`).
        for v in self.immune.iter_mut() {
            *v = v.saturating_sub(1);
        }
        self.anim += dt;
        match self.kind {
            NpcKind::Slime => self.update_slime(world, player, dt),
            NpcKind::DemonEye | NpcKind::WanderingEye => self.update_eye(world, player, dt),
            NpcKind::Zombie => self.update_zombie(world, player, dt),
            NpcKind::Guide
            | NpcKind::Merchant
            | NpcKind::Dryad
            | NpcKind::Nurse
            | NpcKind::ArmsDealer
            | NpcKind::Demolitionist
            | NpcKind::Painter
            | NpcKind::WitchDoctor
            | NpcKind::Clothier
            | NpcKind::GoblinTinkerer => self.update_guide(world, dt),
            NpcKind::QueenBee => self.update_queen_bee(world, player),
            NpcKind::SkeletronHead => self.update_skeletron(world, player),
            NpcKind::WallOfFlesh => self.update_wof(world, player),
            NpcKind::Hungry => self.update_hungry(world, player),
            // The leeches ride the worm pass (aiStyle 6 — the chain move).
            NpcKind::TheDestroyer => {}
            NpcKind::SkeletronPrime => self.update_prime(world, player),
            NpcKind::Retinazer | NpcKind::Spazmatism => self.update_twin(world, player),
            NpcKind::Plantera => self.update_plantera(world, player),
            NpcKind::BrainOfCthulhu => self.update_brain(world, player),
            NpcKind::Creeper => {}
            // aiStyle 0 (`:10252`): the bound stand still until the talk.
            NpcKind::BoundGoblin => {}
            NpcKind::GoblinPeon
            | NpcKind::GoblinThief
            | NpcKind::GoblinWarrior
            | NpcKind::GoblinArcher => self.update_goblin(world, player),
            NpcKind::Skeleton => self.update_skeleton(world, player, dt),
            NpcKind::UndeadMiner => self.update_skeleton(world, player, dt),
            NpcKind::CaveBat | NpcKind::JungleBat | NpcKind::Hellbat | NpcKind::VoodooDemon => {
                self.update_cave_bat(world, player, dt)
            }
            NpcKind::EaterOfSouls => self.update_hover(world, player, HoverParams::EATER_OF_SOULS),
            NpcKind::Hornet => self.update_hover(world, player, HoverParams::HORNET),
            NpcKind::MeteorHead => self.update_hover(world, player, HoverParams::METEOR_HEAD),
            NpcKind::KingSlime => self.update_king_slime(world, player, dt),
            NpcKind::EyeOfCthulhu => self.update_eye_of_cthulhu(world, player),
            NpcKind::ServantOfCthulhu => {
                self.update_hover(world, player, HoverParams::SERVANT_OF_CTHULHU)
            }
            // The leeches ride the worm pass (aiStyle 6 — the chain move).
            NpcKind::GoblinSorcerer | NpcKind::Tim | NpcKind::DarkCaster | NpcKind::FireImp => {
                self.update_caster(world, player, rng)
            }
            NpcKind::ChaosBall | NpcKind::BurningSphere | NpcKind::WaterSphere => {
                self.update_sphere(world, player)
            }
            NpcKind::Goldfish | NpcKind::Piranha | NpcKind::Shark => {
                self.update_swimmer(world, player, rng)
            }
            // The worms: the movement runs in the world sim's worm pass
            // (the chain needs the sibling segments; `AI_006` reads
            // `Main.npc` the same way).
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
            | NpcKind::LeechBody => {}
        }
        self.find_frame();
        self.contact(player);
    }

    /// `NPC.FindFrame` (`NPC.cs:66684`) for the four ported kinds. `num` (the
    /// frame height unit) cancels out because frames are tracked as row indices;
    /// the per-kind frame counts are `Main.npcFrameCount` (`Main.cs:67082`):
    /// slime 2, demon eye 2, zombie 3, King Slime 6. `num2` is the motion class
    /// (`NPC.cs:66698-66703`): 2 jumping up, 3 falling, 1 moving, 0 idle —
    /// `aiAction == 4` (attacking) never happens for these kinds in the port.
    fn find_frame(&mut self) {
        let num2 = if self.vy < 0.0 {
            2
        } else if self.vy > 0.0 {
            3
        } else if self.vx != 0.0 {
            1
        } else {
            0
        };
        match self.kind {
            // The hungry (3 frames) / the leech segments (1).
            NpcKind::Hungry => {
                self.frame_counter += 1.0 + (num2 > 0) as i32 as f32 + (num2 == 4) as i32 as f32;
                if self.frame_counter >= 8.0 {
                    self.frame_counter = 0.0;
                    self.frame = if self.frame + 1 >= 3 {
                        0
                    } else {
                        self.frame + 1
                    };
                }
            }
            NpcKind::LeechHead | NpcKind::LeechBody => {
                self.frame = 0;
            }
            // Case 1 — blue slime (`NPC.cs:70801-70826`): the counter ticks once
            // per step, again when moving and a third time while attacking; at
            // 8.0 the frame advances and wraps over the 2-frame strip.
            NpcKind::Slime => {
                self.frame_counter += 1.0 + (num2 > 0) as i32 as f32 + (num2 == 4) as i32 as f32;
                if self.frame_counter >= 8.0 {
                    self.frame_counter = 0.0;
                    self.frame = if self.frame + 1 >= 2 {
                        0
                    } else {
                        self.frame + 1
                    };
                }
            }
            // Case 4/5 — the eye and the servant (`NPC.cs:67247-67256`): the
            // counter ticks with the motion class, the frame wraps over the
            // 6-frame (the eye) / 2-frame (the servant) strip.
            NpcKind::EyeOfCthulhu => {
                self.frame_counter += 1.0 + (num2 > 0) as i32 as f32 + (num2 == 4) as i32 as f32;
                if self.frame_counter >= 8.0 {
                    self.frame_counter = 0.0;
                    self.frame = if self.frame + 1 >= 6 {
                        0
                    } else {
                        self.frame + 1
                    };
                }
            }
            NpcKind::ServantOfCthulhu => {
                self.frame_counter += 1.0 + (num2 > 0) as i32 as f32 + (num2 == 4) as i32 as f32;
                if self.frame_counter >= 8.0 {
                    self.frame_counter = 0.0;
                    self.frame = if self.frame + 1 >= 2 {
                        0
                    } else {
                        self.frame + 1
                    };
                }
            }
            // Case 50 — King Slime (`NPC.cs:73380-73414`): airborne snaps to
            // frame 4, holds it 4 steps, then frame 5; landing returns to 4 and
            // the grounded cycle runs frames 0..=3 (never reaching 4).
            NpcKind::KingSlime => {
                if self.vy != 0.0 {
                    if self.frame < 4 {
                        self.frame = 4;
                        self.frame_counter = 0.0;
                    }
                    self.frame_counter += 1.0;
                    if self.frame_counter >= 4.0 {
                        self.frame = 5;
                    }
                } else {
                    if self.frame >= 5 {
                        self.frame = 4;
                        self.frame_counter = 0.0;
                    }
                    self.frame_counter +=
                        1.0 + (num2 > 0) as i32 as f32 + (num2 == 4) as i32 as f32;
                    if self.frame_counter >= 8.0 {
                        self.frame_counter = 0.0;
                        self.frame += 1;
                        if self.frame >= 4 {
                            self.frame = 0;
                        }
                    }
                }
            }
            // Case 2 — demon eye and Wandering Eye (`NPC.cs:73994-74057`): a
            // plain 8-step frame cycle (2 frames for the eye, 4 for the
            // Wandering Eye, `Main.cs:67082`), plus the aim: type 2 is in the
            // first rotation branch (`NPC.cs:74019-74031`) —
            // `rotation = atan2(vy, vx)`, `+ 3.14f` when moving left, and the
            // sprite mirrors (`spriteDirection`) so its pupil follows the
            // flight direction. `rotation` freezes while `velocity.X == 0`
            // exactly like the original (neither `if` fires).
            NpcKind::DemonEye | NpcKind::WanderingEye => {
                // Wandering Eye wraps over Main.npcFrameCount[133] = 4 frames
                // (Main.cs:67082); the demon eye over its 2.
                let frame_count = match self.kind {
                    NpcKind::WanderingEye => 4,
                    _ => 2,
                };
                self.frame_counter += 1.0;
                if self.frame_counter >= 8.0 {
                    self.frame_counter = 0.0;
                    self.frame = if self.frame + 1 >= frame_count {
                        0
                    } else {
                        self.frame + 1
                    };
                }
                if self.vx > 0.0 {
                    self.rotation = self.vy.atan2(self.vx);
                } else if self.vx < 0.0 {
                    // The original adds the literal `3.14f`, not PI.
                    self.rotation = self.vy.atan2(self.vx) + 3.14;
                }
            }
            // Case 3 — zombie (`NPC.cs:76324-76389`): falling or moving against
            // the facing direction holds the lurch frame 2; standing still is
            // frame 0; walking accumulates `|velocity.X|` through the band
            // 0 → 1 → 2 → 1 at thresholds 8/16/24/32 and restarts.
            NpcKind::Zombie => {
                if self.vy != 0.0
                    || (self.facing == -1 && self.vx > 0.0)
                    || (self.facing == 1 && self.vx < 0.0)
                {
                    self.frame_counter = 0.0;
                    self.frame = 2;
                } else if self.vx == 0.0 {
                    self.frame_counter = 0.0;
                    self.frame = 0;
                } else {
                    self.frame_counter += self.vx.abs();
                    if self.frame_counter < 8.0 {
                        self.frame = 0;
                    } else if self.frame_counter < 16.0 {
                        self.frame = 1;
                    } else if self.frame_counter < 24.0 {
                        self.frame = 2;
                    } else if self.frame_counter < 32.0 {
                        self.frame = 1;
                    } else {
                        // The original leaves `frame.Y` on its previous value
                        // here — the restart tick keeps frame 1.
                        self.frame_counter = 0.0;
                    }
                }
            }
            NpcKind::Skeleton | NpcKind::UndeadMiner => {
                if self.vy != 0.0
                    || (self.facing == -1 && self.vx > 0.0)
                    || (self.facing == 1 && self.vx < 0.0)
                {
                    self.frame_counter = 0.0;
                    self.frame = 2;
                } else if self.vx == 0.0 {
                    self.frame_counter = 0.0;
                    self.frame = 0;
                } else {
                    self.frame_counter += self.vx.abs();
                    if self.frame_counter < 8.0 {
                        self.frame = 0;
                    } else if self.frame_counter < 16.0 {
                        self.frame = 1;
                    } else if self.frame_counter < 24.0 {
                        self.frame = 2;
                    } else if self.frame_counter < 32.0 {
                        self.frame = 1;
                    } else {
                        // The original leaves `frame.Y` on its previous value
                        // here — the restart tick keeps frame 1.
                        self.frame_counter = 0.0;
                    }
                }
            }
            // Case 49 — Cave Bat (`AI_014_Bats`): the plain flap over the
            // 5-frame strip (`Main.npcFrameCount[49] = 5`, `Main.cs:67082`).
            // The hovers: the plain flap cycles (`Main.npcFrameCount[6] = 2`,
            // `[42] = 3`, `[23] = 2`, `Main.cs:67082`).
            NpcKind::EaterOfSouls | NpcKind::Hornet | NpcKind::MeteorHead => {
                let count = match self.kind {
                    NpcKind::Hornet => 3,
                    _ => 2,
                };
                self.frame_counter += 1.0;
                if self.frame_counter >= 8.0 {
                    self.frame_counter = 0.0;
                    self.frame = if self.frame + 1 >= count {
                        0
                    } else {
                        self.frame + 1
                    };
                }
            }
            NpcKind::CaveBat | NpcKind::JungleBat | NpcKind::Hellbat | NpcKind::VoodooDemon => {
                self.frame_counter += 1.0;
                if self.frame_counter >= 4.0 {
                    self.frame_counter = 0.0;
                    self.frame = if self.frame + 1 >= 5 {
                        0
                    } else {
                        self.frame + 1
                    };
                }
            }
            // The casters (`FindFrame`'s case 29/32/45 group,
            // `NPC.cs:77019-77054`): the pose machine — row 0 stands, row 1
            // is the cast windup (`ai[1] > 0`), row 2 airborne. The Dark
            // Caster's Skeletron pose (`localAI[3]`, `:77050-77053`) rides
            // the skeleton-head work.
            NpcKind::GoblinSorcerer | NpcKind::Tim | NpcKind::DarkCaster => {
                self.frame = if self.vy != 0.0 {
                    2
                } else if self.cast_windup > 0 {
                    1
                } else {
                    0
                };
            }
            // The Fire Imp (`NPC.cs:76979-77017`): the 10-frame strip —
            // rows 0..3 walk (5 steps each), the cast pose snaps to row 4
            // and advances to the held row 9.
            NpcKind::FireImp => {
                self.frame_counter += 1.0;
                if self.cast_windup > 0 {
                    if self.frame < 4 {
                        self.frame = 4;
                        self.frame_counter = 0.0;
                    }
                    if self.frame_counter > 5.0 {
                        self.frame_counter = 0.0;
                        self.frame += 1;
                    }
                    if self.frame > 9 {
                        self.frame = 9;
                    }
                } else {
                    if self.frame_counter > 5.0 {
                        self.frame_counter = 0.0;
                        self.frame += 1;
                    }
                    if self.frame > 3 {
                        self.frame = 0;
                    }
                }
            }
            // The spheres: `Main.npcFrameCount` 30/25/33 = 1.
            NpcKind::ChaosBall | NpcKind::BurningSphere | NpcKind::WaterSphere => {}
            // The swimmers: the plain flap cycles over `Main.npcFrameCount`
            // 55/58 = 6, 65 = 4 (`Main.cs:67082`).
            NpcKind::Goldfish | NpcKind::Piranha | NpcKind::Shark => {
                let count = match self.kind {
                    NpcKind::Shark => 4,
                    _ => 6,
                };
                self.frame_counter += 1.0;
                if self.frame_counter >= 8.0 {
                    self.frame_counter = 0.0;
                    self.frame = if self.frame + 1 >= count {
                        0
                    } else {
                        self.frame + 1
                    };
                }
            }
            // The worms: `Main.npcFrameCount` 7..15 = 1 (`Main.cs:67082`).
            NpcKind::DevourerHead
            | NpcKind::DevourerBody
            | NpcKind::DevourerTail
            | NpcKind::GiantWormHead
            | NpcKind::GiantWormBody
            | NpcKind::GiantWormTail
            | NpcKind::EaterOfWorldsHead
            | NpcKind::EaterOfWorldsBody
            | NpcKind::EaterOfWorldsTail => {}
            NpcKind::GoblinPeon
            | NpcKind::GoblinThief
            | NpcKind::GoblinWarrior
            | NpcKind::GoblinArcher => {
                // The goblins: the walker 3-frame machine (aiStyle 3).
                if self.vy != 0.0
                    || (self.facing == -1 && self.vx > 0.0)
                    || (self.facing == 1 && self.vx < 0.0)
                {
                    self.frame_counter = 0.0;
                    self.frame = 2;
                } else if self.vx == 0.0 {
                    self.frame_counter = 0.0;
                    self.frame = 0;
                } else {
                    self.frame_counter += self.vx.abs();
                    if self.frame_counter < 8.0 {
                        self.frame = 0;
                    } else if self.frame_counter < 16.0 {
                        self.frame = 1;
                    } else if self.frame_counter < 24.0 {
                        self.frame = 2;
                    } else if self.frame_counter < 32.0 {
                        self.frame = 1;
                    } else {
                        self.frame_counter = 0.0;
                    }
                }
            }
            NpcKind::GoblinTinkerer => {
                // The town 3-frame machine (`Main.npcFrameCount[107]`,
                // aiStyle 7's FindFrame — the Guide's arm).
                if self.vy != 0.0
                    || (self.facing == -1 && self.vx > 0.0)
                    || (self.facing == 1 && self.vx < 0.0)
                {
                    self.frame_counter = 0.0;
                    self.frame = 2;
                } else if self.vx == 0.0 {
                    self.frame_counter = 0.0;
                    self.frame = 0;
                } else {
                    self.frame_counter += self.vx.abs();
                    if self.frame_counter < 8.0 {
                        self.frame = 0;
                    } else if self.frame_counter < 16.0 {
                        self.frame = 1;
                    } else if self.frame_counter < 24.0 {
                        self.frame = 2;
                    } else if self.frame_counter < 32.0 {
                        self.frame = 1;
                    } else {
                        self.frame_counter = 0.0;
                    }
                }
            }
            NpcKind::BoundGoblin => {
                // The tied pose: the single frame holds (`aiStyle 0` runs
                // no FindFrame cycle).
                self.frame_counter = 0.0;
                self.frame = 0;
            }
            NpcKind::BrainOfCthulhu | NpcKind::Creeper => {
                // `Main.npcFrameCount[266]` = 1, [267] = 1 (`Main.cs:67082`).
            }
            NpcKind::Plantera => {
                // `Main.npcFrameCount[262]` = 5 (`Main.cs:67082`).
                self.frame_counter += 1.0;
                if self.frame_counter >= 8.0 {
                    self.frame_counter = 0.0;
                    self.frame = if self.frame + 1 >= 5 {
                        0
                    } else {
                        self.frame + 1
                    };
                }
            }
            NpcKind::Retinazer | NpcKind::Spazmatism => {
                // `Main.npcFrameCount[125/126]` = 2 (`Main.cs:67082`).
                self.frame_counter += 1.0;
                if self.frame_counter >= 8.0 {
                    self.frame_counter = 0.0;
                    self.frame = if self.frame + 1 >= 2 {
                        0
                    } else {
                        self.frame + 1
                    };
                }
            }
            NpcKind::SkeletronPrime => {
                // `Main.npcFrameCount[127]` = 1 (`Main.cs:67082`).
            }
            NpcKind::TheDestroyer => {
                // `Main.npcFrameCount[134]` = 1 (`Main.cs:67082`).
            }
            NpcKind::WallOfFlesh => {
                // `Main.npcFrameCount[113]` = 1 (`Main.cs:67082`).
            }
            NpcKind::SkeletronHead => {
                // `Main.npcFrameCount[35]` = 1 (`Main.cs:67082`).
            }
            NpcKind::QueenBee => {
                // `Main.npcFrameCount[222]` = 2 (`Main.cs:67082`).
                self.frame_counter += 1.0;
                if self.frame_counter >= 8.0 {
                    self.frame_counter = 0.0;
                    self.frame = if self.frame + 1 >= 2 {
                        0
                    } else {
                        self.frame + 1
                    };
                }
            }
            NpcKind::EyeOfCthulhu => {
                // `Main.npcFrameCount[4]` = 2 (`Main.cs:67082`): the pupil
                // blink over the 8-step accumulator (the AI_004 hover's
                // frame machine, `NPC.cs:73800+`).
                self.frame_counter += 1.0;
                if self.frame_counter >= 8.0 {
                    self.frame_counter = 0.0;
                    self.frame = if self.frame + 1 >= 2 {
                        0
                    } else {
                        self.frame + 1
                    };
                }
            }
            NpcKind::Clothier => {
                // The town wander rides the Guide's 3-frame humanoid machine
                // (`Main.npcFrameCount[54]` = 3, `Main.cs:67082`).
                if self.vy != 0.0
                    || (self.facing == -1 && self.vx > 0.0)
                    || (self.facing == 1 && self.vx < 0.0)
                {
                    self.frame_counter = 0.0;
                    self.frame = 2;
                } else if self.vx == 0.0 {
                    self.frame_counter = 0.0;
                    self.frame = 0;
                } else {
                    self.frame_counter += self.vx.abs();
                    if self.frame_counter < 8.0 {
                        self.frame = 0;
                    } else if self.frame_counter < 16.0 {
                        self.frame = 1;
                    } else if self.frame_counter < 24.0 {
                        self.frame = 2;
                    } else if self.frame_counter < 32.0 {
                        self.frame = 1;
                    } else {
                        self.frame_counter = 0.0;
                    }
                }
            }
            NpcKind::WitchDoctor => {
                // The town wander rides the Guide's 3-frame humanoid machine
                // (`Main.npcFrameCount[228]` = 3, `Main.cs:67082`).
                if self.vy != 0.0
                    || (self.facing == -1 && self.vx > 0.0)
                    || (self.facing == 1 && self.vx < 0.0)
                {
                    self.frame_counter = 0.0;
                    self.frame = 2;
                } else if self.vx == 0.0 {
                    self.frame_counter = 0.0;
                    self.frame = 0;
                } else {
                    self.frame_counter += self.vx.abs();
                    if self.frame_counter < 8.0 {
                        self.frame = 0;
                    } else if self.frame_counter < 16.0 {
                        self.frame = 1;
                    } else if self.frame_counter < 24.0 {
                        self.frame = 2;
                    } else if self.frame_counter < 32.0 {
                        self.frame = 1;
                    } else {
                        self.frame_counter = 0.0;
                    }
                }
            }
            NpcKind::Painter => {
                // The town wander rides the Guide's 3-frame humanoid machine
                // (`Main.npcFrameCount[227]` = 3, `Main.cs:67082`).
                if self.vy != 0.0
                    || (self.facing == -1 && self.vx > 0.0)
                    || (self.facing == 1 && self.vx < 0.0)
                {
                    self.frame_counter = 0.0;
                    self.frame = 2;
                } else if self.vx == 0.0 {
                    self.frame_counter = 0.0;
                    self.frame = 0;
                } else {
                    self.frame_counter += self.vx.abs();
                    if self.frame_counter < 8.0 {
                        self.frame = 0;
                    } else if self.frame_counter < 16.0 {
                        self.frame = 1;
                    } else if self.frame_counter < 24.0 {
                        self.frame = 2;
                    } else if self.frame_counter < 32.0 {
                        self.frame = 1;
                    } else {
                        self.frame_counter = 0.0;
                    }
                }
            }
            NpcKind::Demolitionist => {
                // The town wander rides the Guide's 3-frame humanoid machine
                // (`Main.npcFrameCount[38]` = 3, `Main.cs:67082`).
                if self.vy != 0.0
                    || (self.facing == -1 && self.vx > 0.0)
                    || (self.facing == 1 && self.vx < 0.0)
                {
                    self.frame_counter = 0.0;
                    self.frame = 2;
                } else if self.vx == 0.0 {
                    self.frame_counter = 0.0;
                    self.frame = 0;
                } else {
                    self.frame_counter += self.vx.abs();
                    if self.frame_counter < 8.0 {
                        self.frame = 0;
                    } else if self.frame_counter < 16.0 {
                        self.frame = 1;
                    } else if self.frame_counter < 24.0 {
                        self.frame = 2;
                    } else if self.frame_counter < 32.0 {
                        self.frame = 1;
                    } else {
                        self.frame_counter = 0.0;
                    }
                }
            }
            NpcKind::ArmsDealer => {
                // The town wander rides the Guide's 3-frame humanoid machine
                // (`Main.npcFrameCount[19]` = 3, `Main.cs:67082`).
                if self.vy != 0.0
                    || (self.facing == -1 && self.vx > 0.0)
                    || (self.facing == 1 && self.vx < 0.0)
                {
                    self.frame_counter = 0.0;
                    self.frame = 2;
                } else if self.vx == 0.0 {
                    self.frame_counter = 0.0;
                    self.frame = 0;
                } else {
                    self.frame_counter += self.vx.abs();
                    if self.frame_counter < 8.0 {
                        self.frame = 0;
                    } else if self.frame_counter < 16.0 {
                        self.frame = 1;
                    } else if self.frame_counter < 24.0 {
                        self.frame = 2;
                    } else if self.frame_counter < 32.0 {
                        self.frame = 1;
                    } else {
                        self.frame_counter = 0.0;
                    }
                }
            }
            NpcKind::Nurse => {
                // The town wander rides the Guide's 3-frame humanoid machine
                // (`Main.npcFrameCount[18]` = 3, `Main.cs:67082`).
                if self.vy != 0.0
                    || (self.facing == -1 && self.vx > 0.0)
                    || (self.facing == 1 && self.vx < 0.0)
                {
                    self.frame_counter = 0.0;
                    self.frame = 2;
                } else if self.vx == 0.0 {
                    self.frame_counter = 0.0;
                    self.frame = 0;
                } else {
                    self.frame_counter += self.vx.abs();
                    if self.frame_counter < 8.0 {
                        self.frame = 0;
                    } else if self.frame_counter < 16.0 {
                        self.frame = 1;
                    } else if self.frame_counter < 24.0 {
                        self.frame = 2;
                    } else if self.frame_counter < 32.0 {
                        self.frame = 1;
                    } else {
                        self.frame_counter = 0.0;
                    }
                }
            }
            NpcKind::Dryad => {
                // The town wander rides the Guide's 3-frame humanoid machine
                // (`Main.npcFrameCount[20]` = 3, `Main.cs:67082`).
                if self.vy != 0.0
                    || (self.facing == -1 && self.vx > 0.0)
                    || (self.facing == 1 && self.vx < 0.0)
                {
                    self.frame_counter = 0.0;
                    self.frame = 2;
                } else if self.vx == 0.0 {
                    self.frame_counter = 0.0;
                    self.frame = 0;
                } else {
                    self.frame_counter += self.vx.abs();
                    if self.frame_counter < 8.0 {
                        self.frame = 0;
                    } else if self.frame_counter < 16.0 {
                        self.frame = 1;
                    } else if self.frame_counter < 24.0 {
                        self.frame = 2;
                    } else if self.frame_counter < 32.0 {
                        self.frame = 1;
                    } else {
                        self.frame_counter = 0.0;
                    }
                }
            }
            NpcKind::Merchant => {
                // The town wander rides the Guide's 3-frame humanoid machine
                // (`Main.npcFrameCount[17]` = 3, `Main.cs:67082`; the
                // vanilla FindFrame's town machine, `NPC.cs:68632+`).
                if self.vy != 0.0
                    || (self.facing == -1 && self.vx > 0.0)
                    || (self.facing == 1 && self.vx < 0.0)
                {
                    self.frame_counter = 0.0;
                    self.frame = 2;
                } else if self.vx == 0.0 {
                    self.frame_counter = 0.0;
                    self.frame = 0;
                } else {
                    self.frame_counter += self.vx.abs();
                    if self.frame_counter < 8.0 {
                        self.frame = 0;
                    } else if self.frame_counter < 16.0 {
                        self.frame = 1;
                    } else if self.frame_counter < 24.0 {
                        self.frame = 2;
                    } else if self.frame_counter < 32.0 {
                        self.frame = 1;
                    } else {
                        self.frame_counter = 0.0;
                    }
                }
            }
            NpcKind::Guide => {
                if self.vy != 0.0
                    || (self.facing == -1 && self.vx > 0.0)
                    || (self.facing == 1 && self.vx < 0.0)
                {
                    self.frame_counter = 0.0;
                    self.frame = 2;
                } else if self.vx == 0.0 {
                    self.frame_counter = 0.0;
                    self.frame = 0;
                } else {
                    self.frame_counter += self.vx.abs();
                    if self.frame_counter < 8.0 {
                        self.frame = 0;
                    } else if self.frame_counter < 16.0 {
                        self.frame = 1;
                    } else if self.frame_counter < 24.0 {
                        self.frame = 2;
                    } else if self.frame_counter < 32.0 {
                        self.frame = 1;
                    } else {
                        // The original leaves `frame.Y` on its previous value
                        // here — the restart tick keeps frame 1.
                        self.frame_counter = 0.0;
                    }
                }
            }
        }
    }

    fn update_slime(&mut self, world: &World, player: &mut Player, _dt: f32) {
        let (w, h) = NpcKind::Slime.size();
        let (pcx, _) = player.center();
        let (cx, _) = self.center();
        self.hop_timer -= 1.0 / 60.0;
        let near = (pcx - cx).abs() < 60.0 * TILE_PX;
        if self.on_ground && self.hop_timer <= 0.0 && near {
            self.facing = if pcx >= cx { 1 } else { -1 };
            self.vx = self.facing as f32 * 1.4;
            self.vy = -4.2;
            self.on_ground = false;
            self.hop_timer = 1.0 + (self.anim * 7.0).sin().abs() * 0.5;
        }
        // `NPC.UpdateNPC_Gravity`: water lowers gravity and the fall cap.
        let wet = is_wet(world, self.x, self.y, w, h);
        if wet {
            self.vy = (self.vy + 0.2).min(7.0);
        } else {
            self.vy = (self.vy + GRAVITY).min(MAX_FALL);
        }

        let nx = self.x + self.vx;
        if !collides_rect(world, nx, self.y, w, h) {
            self.x = nx;
        } else {
            self.vx = -self.vx * 0.4;
        }
        let ny = self.y + self.vy;
        let mut collide_y = false;
        if !collides_rect(world, self.x, ny, w, h) {
            self.y = ny;
            self.on_ground = false;
        } else if self.vy > 0.0 {
            let ty = ((ny + h) / TILE_PX).floor() as i32;
            self.y = ty as f32 * TILE_PX - h;
            self.vy = 0.0;
            self.on_ground = true;
            self.vx *= 0.6;
            collide_y = true;
        } else if self.vy < 0.0 {
            let ty = (ny / TILE_PX).floor() as i32;
            self.y = (ty + 1) as f32 * TILE_PX;
            self.vy = 0.0;
            collide_y = true;
        }
        // `AI_001_Slimes` water tail: pushed up every tick, with a small bounce when
        // it touches the floor or the surface, so slimes bob at the water line.
        if wet {
            if collide_y {
                self.vy = -2.0;
            }
            if self.vy < 0.0 && self.wet_x == self.x {
                self.facing = -self.facing;
                self.hop_timer = 200.0 / 60.0;
            }
            if self.vy > 0.0 {
                self.wet_x = self.x;
            }
            if self.vy > 2.0 {
                self.vy *= 0.9;
            }
            self.vy -= 0.5;
            if self.vy < -4.0 {
                self.vy = -4.0;
            }
        }
        if self.vx != 0.0 {
            self.facing = if self.vx > 0.0 { 1 } else { -1 };
        }
        if self.y > world.height as f32 * TILE_PX + 200.0 {
            self.hp = 0;
        }
    }

    fn update_eye(&mut self, world: &World, player: &mut Player, _dt: f32) {
        let (w, h) = NpcKind::DemonEye.size();
        let (pcx, pcy) = player.center();
        let (cx, cy) = self.center();
        let dx = pcx - cx;
        let dy = pcy - cy;
        let d = (dx * dx + dy * dy).sqrt().max(1.0);
        self.vx = (self.vx + dx / d * 0.12).clamp(-3.0, 3.0);
        self.vy = (self.vy + dy / d * 0.12).clamp(-3.0, 3.0);
        let nx = self.x + self.vx;
        if !collides_rect(world, nx, self.y, w, h) {
            self.x = nx;
        } else {
            self.vx = -self.vx * 0.5;
        }
        let bob = (self.anim * 3.0).sin() * 0.4;
        let ny = self.y + self.vy + bob;
        if !collides_rect(world, self.x, ny, w, h) {
            self.y = ny;
        } else {
            self.vy = -self.vy * 0.5;
        }
        self.facing = if dx >= 0.0 { 1 } else { -1 };
        if self.y > world.height as f32 * TILE_PX + 200.0 {
            self.hp = 0;
        }
    }

    /// `AI_003_Fighters`: walk toward the player and jump when the path is blocked.
    fn update_zombie(&mut self, world: &World, player: &mut Player, _dt: f32) {
        let (w, h) = NpcKind::Zombie.size();
        let (pcx, pcy) = player.center();
        let (cx, cy) = self.center();
        let dx = pcx - cx;
        let dy = pcy - cy;
        let near = dx.abs() < 80.0 * TILE_PX && dy.abs() < 40.0 * TILE_PX;
        if near {
            self.facing = if dx >= 0.0 { 1 } else { -1 };
            self.vx = self.facing as f32 * 0.6;
        } else {
            self.vx *= 0.9;
        }
        self.vy = (self.vy + GRAVITY).min(MAX_FALL);

        let nx = self.x + self.vx;
        if !collides_rect(world, nx, self.y, w, h) {
            self.x = nx;
        } else if self.on_ground {
            // Jump a one-tile ledge or wall instead of grinding into it.
            self.vy = -6.2;
            self.on_ground = false;
        } else {
            self.vx = 0.0;
        }

        let ny = self.y + self.vy;
        if !collides_rect(world, self.x, ny, w, h) {
            self.y = ny;
            self.on_ground = false;
        } else if self.vy > 0.0 {
            let ty = ((ny + h) / TILE_PX).floor() as i32;
            self.y = ty as f32 * TILE_PX - h;
            self.vy = 0.0;
            self.on_ground = true;
        } else if self.vy < 0.0 {
            let ty = (ny / TILE_PX).floor() as i32;
            self.y = (ty + 1) as f32 * TILE_PX;
            self.vy = 0.0;
        }

        if world.day_time {
            self.burn += 1.0 / 60.0;
            if self.burn >= 1.0 {
                self.burn -= 1.0;
                self.hp -= 6;
            }
        } else {
            self.burn = 0.0;
        }
        if self.y > world.height as f32 * TILE_PX + 200.0 {
            self.hp = 0;
        }
    }

    /// Simplified `AI_015_KingSlime`: big hops toward the player, a teleport when the
    /// player escapes, and a burst of blue slimes when it lands. Minion spawning is
    /// reported through `spawn_minion` so the app can add them to its NPC list.
    /// `AI()`'s aiStyle-4 block — the Eye of Cthulhu's phase machine
    /// (`NPC.cs`'s aiStyle-4 `if` in `AI()`, 841 C# lines). The state:
    /// `ai[0]` the phase (0 the hover, 1/2 the spins, 3 the final),
    /// `ai[1]` the sub-state, `ai[2]` the timer, `ai[3]` the dash/servant
    /// counter. The expert-mode and `getGoodWorld` branches are out (the
    /// port has neither difficulty flag) — the classic path is verbatim,
    /// the skips carry the C# cites. The movement integrates directly
    /// (`noTileCollide` + `noGravity`).
    fn update_eye_of_cthulhu(&mut self, world: &World, player: &Player) {
        let (w, h) = self.size();
        let (pcx, pcy) = player.center();
        let dead = player.hp <= 0;
        // `num6`: the aim angle — `Math.Atan2(y: bottom - 59 - target.Y - h/2,
        // x: center.X - target.X - w/2) + 1.57` wrapped to 2π.
        let mut num6 = (self.y + h - 59.0 - pcy).atan2(self.x + w * 0.5 - pcx) + 1.57;
        if num6 < 0.0 {
            num6 += 6.283;
        } else if num6 > 6.283 {
            num6 -= 6.283;
        }
        // `num7`: the turn rate by the state (the C# num7 table).
        let mut num7 = 0.0;
        if self.ai[0] == 0.0 && self.ai[1] == 0.0 {
            num7 = 0.02;
        }
        if self.ai[0] == 0.0 && self.ai[1] == 2.0 && self.ai[2] > 40.0 {
            num7 = 0.05;
        }
        if self.ai[0] == 3.0 && self.ai[1] == 0.0 {
            num7 = 0.05;
        }
        if self.ai[0] == 3.0 && self.ai[1] == 2.0 && self.ai[2] > 40.0 {
            num7 = 0.08;
        }
        if self.ai[0] == 3.0 && self.ai[1] == 4.0 && self.ai[2] > 20.0 {
            num7 = 0.15;
        }
        if self.ai[0] == 3.0 && self.ai[1] == 5.0 {
            num7 = 0.05;
        }
        if self.rotation < num6 {
            if num6 - self.rotation > 3.1415 {
                self.rotation -= num7;
            } else {
                self.rotation += num7;
            }
        } else if self.rotation > num6 {
            if self.rotation - num6 > 3.1415 {
                self.rotation += num7;
            } else {
                self.rotation -= num7;
            }
        }
        if self.rotation > num6 - num7 && self.rotation < num6 + num7 {
            self.rotation = num6;
        }
        if self.rotation < 0.0 {
            self.rotation += 6.283;
        } else if self.rotation > 6.283 {
            self.rotation -= 6.283;
        }
        if self.rotation > num6 - num7 && self.rotation < num6 + num7 {
            self.rotation = num6;
        }
        // `Main.IsItDay() | dead`: the climb-away
        // (`velocity.Y -= 0.04f; EncourageDespawn(10)` — the removal rides
        // the world sim's distance cull).
        if world.day_time || dead {
            self.vy -= 0.04;
            self.integrate_no_tile_collide();
            return;
        }
        if self.ai[0] == 0.0 {
            if self.ai[1] == 0.0 {
                // The hover: 5 speed, 0.04 accel toward 200 px above the
                // player (`num9`/`num10` classic).
                let cx = self.x + w * 0.5;
                let cy = self.y + h * 0.5;
                let num11 = pcx - cx;
                let num12 = pcy - 200.0 - cy;
                let num14 = (num11 * num11 + num12 * num12).sqrt();
                let num13 = if num14 != 0.0 { 5.0 / num14 } else { 0.0 };
                let (dx, dy) = (num11 * num13, num12 * num13);
                if self.vx < dx {
                    self.vx += 0.04;
                    if self.vx < 0.0 && dx > 0.0 {
                        self.vx += 0.04;
                    }
                } else if self.vx > dx {
                    self.vx -= 0.04;
                    if self.vx > 0.0 && dx < 0.0 {
                        self.vx -= 0.04;
                    }
                }
                if self.vy < dy {
                    self.vy += 0.04;
                    if self.vy < 0.0 && dy > 0.0 {
                        self.vy += 0.04;
                    }
                } else if self.vy > dy {
                    self.vy -= 0.04;
                    if self.vy > 0.0 && dy < 0.0 {
                        self.vy -= 0.04;
                    }
                }
                self.ai[2] += 1.0;
                if self.ai[2] >= 600.0 {
                    self.ai[1] = 1.0;
                    self.ai[2] = 0.0;
                    self.ai[3] = 0.0;
                } else if (self.y + h < player.y && num14 < 500.0) && !dead {
                    self.ai[3] += 1.0;
                    if self.ai[3] >= 110.0 {
                        // The servant burst (the `NPC.NewNPC(..., 5)` arm).
                        self.ai[3] = 0.0;
                        self.rotation = num6;
                        let num18 = pcx - cx;
                        let num19 = pcy - cy;
                        let num20 = 5.0 / (num18 * num18 + num19 * num19).sqrt();
                        let (vx, vy) = (num18 * num20, num19 * num20);
                        self.wants_servant = Some((cx + vx * 10.0, cy + vy * 10.0, vx, vy));
                    }
                }
            } else if self.ai[1] == 1.0 {
                // The dash wind-up: the velocity locks onto the target
                // (6 speed classic).
                self.rotation = num6;
                let cx = self.x + w * 0.5;
                let cy = self.y + h * 0.5;
                let num23 = pcx - cx;
                let num24 = pcy - cy;
                let num25 = 6.0 / (num23 * num23 + num24 * num24).sqrt();
                self.vx = num23 * num25;
                self.vy = num24 * num25;
                self.ai[1] = 2.0;
            } else if self.ai[1] == 2.0 {
                // The dash flight: 40 free ticks, then the 0.98 damp; the
                // chain resets at 150.
                self.ai[2] += 1.0;
                if self.ai[2] >= 40.0 {
                    self.vx *= 0.98;
                    self.vy *= 0.98;
                    if self.vx.abs() < 0.1 {
                        self.vx = 0.0;
                    }
                    if self.vy.abs() < 0.1 {
                        self.vy = 0.0;
                    }
                } else {
                    self.rotation = self.vy.atan2(self.vx) - 1.57;
                }
                if self.ai[2] >= 150.0 {
                    self.ai[3] += 1.0;
                    self.ai[2] = 0.0;
                    self.rotation = num6;
                    if self.ai[3] >= 3.0 {
                        self.ai[1] = 0.0;
                        self.ai[3] = 0.0;
                    } else {
                        self.ai[1] = 1.0;
                    }
                }
            }
            if (self.hp as f32) < self.max_hp as f32 * 0.5 {
                self.ai[0] = 1.0;
                self.ai[1] = 0.0;
                self.ai[2] = 0.0;
                self.ai[3] = 0.0;
            }
            self.integrate_no_tile_collide();
            return;
        }
        if self.ai[0] == 1.0 || self.ai[0] == 2.0 {
            // The spin: `ai[2]` ramps to 0.5 while phase 1 (or the
            // getGood half-spin `ai[3] == 1`), rotation spins, `ai[1]`
            // counts to 100 → the next phase (the classic path; the
            // expert servant bursts and the `reflectsProjectiles` flag
            // are the getGood/expert arms).
            if self.ai[0] == 1.0 || self.ai[3] == 1.0 {
                self.ai[2] += 0.005;
                if self.ai[2] > 0.5 {
                    self.ai[2] = 0.5;
                }
            } else {
                self.ai[2] -= 0.005;
                if self.ai[2] < 0.0 {
                    self.ai[2] = 0.0;
                }
            }
            self.rotation += self.ai[2];
            self.ai[1] += 1.0;
            if self.ai[1] >= 100.0 {
                if self.ai[3] == 1.0 {
                    self.ai[3] = 0.0;
                    self.ai[1] = 0.0;
                } else {
                    self.ai[0] += 1.0;
                    self.ai[1] = 0.0;
                    if self.ai[0] != 3.0 {
                        self.ai[2] = 0.0;
                    }
                }
            }
            self.vx *= 0.98;
            self.vy *= 0.98;
            if self.vx.abs() < 0.1 {
                self.vx = 0.0;
            }
            if self.vy.abs() < 0.1 {
                self.vy = 0.0;
            }
            self.integrate_no_tile_collide();
            return;
        }
        // Phase 3: `this.defense = 0` and the faster dash machine (6.8
        // speed, 130-tick chains; the expert rage sub-states
        // `ai[1] = 3/4/5` are out with the flag2/flag3 gates).
        self.defense_override = Some(0);
        if self.ai[1] == 0.0 {
            // The hover at 120 px above (6 speed, 0.07 accel).
            let cx = self.x + w * 0.5;
            let cy = self.y + h * 0.5;
            let num39 = pcx - cx;
            let num40 = pcy - 120.0 - cy;
            let num41 = (num39 * num39 + num40 * num40).sqrt();
            let num42 = if num41 != 0.0 { 6.0 / num41 } else { 0.0 };
            let (dx, dy) = (num39 * num42, num40 * num42);
            if self.vx < dx {
                self.vx += 0.07;
                if self.vx < 0.0 && dx > 0.0 {
                    self.vx += 0.07;
                }
            } else if self.vx > dx {
                self.vx -= 0.07;
                if self.vx > 0.0 && dx < 0.0 {
                    self.vx -= 0.07;
                }
            }
            if self.vy < dy {
                self.vy += 0.07;
                if self.vy < 0.0 && dy > 0.0 {
                    self.vy += 0.07;
                }
            } else if self.vy > dy {
                self.vy -= 0.07;
                if self.vy > 0.0 && dy < 0.0 {
                    self.vy -= 0.07;
                }
            }
            self.ai[2] += 1.0;
            if self.ai[2] >= 200.0 {
                self.ai[1] = 1.0;
                self.ai[2] = 0.0;
                self.ai[3] = 0.0;
            }
        } else if self.ai[1] == 1.0 {
            self.rotation = num6;
            let cx = self.x + w * 0.5;
            let cy = self.y + h * 0.5;
            let num43 = pcx - cx;
            let num44 = pcy - cy;
            let num45 = 6.8 / (num43 * num43 + num44 * num44).sqrt();
            self.vx = num43 * num45;
            self.vy = num44 * num45;
            self.ai[1] = 2.0;
        } else if self.ai[1] == 2.0 {
            self.ai[2] += 1.0;
            if self.ai[2] >= 40.0 {
                self.vx *= 0.97;
                self.vy *= 0.97;
                if self.vx.abs() < 0.1 {
                    self.vx = 0.0;
                }
                if self.vy.abs() < 0.1 {
                    self.vy = 0.0;
                }
            } else {
                self.rotation = self.vy.atan2(self.vx) - 1.57;
            }
            if self.ai[2] >= 130.0 {
                self.ai[3] += 1.0;
                self.ai[2] = 0.0;
                self.rotation = num6;
                if self.ai[3] >= 3.0 {
                    self.ai[1] = 0.0;
                    self.ai[3] = 0.0;
                } else {
                    self.ai[1] = 1.0;
                }
            }
        }
        self.integrate_no_tile_collide();
    }

    /// `noTileCollide` + `noGravity` movement: the raw integration, no
    /// tile interaction (`SetDefaults` cases 4/5 set both flags).
    fn integrate_no_tile_collide(&mut self) {
        self.x += self.vx;
        self.y += self.vy;
    }

    fn update_king_slime(&mut self, world: &World, player: &mut Player, dt: f32) {
        let (w, h) = NpcKind::KingSlime.size();
        let (pcx, pcy) = player.center();
        let (cx, cy) = self.center();
        let dx = pcx - cx;
        self.facing = if dx >= 0.0 { 1 } else { -1 };
        self.hop_timer -= dt;
        if self.on_ground && self.hop_timer <= 0.0 {
            self.vx = self.facing as f32 * 2.4;
            self.vy = -8.0;
            self.on_ground = false;
            self.hop_timer = 1.4 + (self.anim * 2.7).sin().abs() * 0.8;
        }
        // Teleport next to the player when they outrun the boss.
        if (dx.abs() > 70.0 * TILE_PX || (pcy - cy).abs() > 50.0 * TILE_PX) && self.on_ground {
            let side = if dx >= 0.0 { -1 } else { 1 };
            let tx = ((pcx / TILE_PX) as i32 + side * 10).clamp(4, world.width - 5);
            let top = world.surface_of(tx);
            if top < world.height - 2 {
                let (w2, h2) = NpcKind::KingSlime.size();
                let nx = tx as f32 * TILE_PX + 8.0 - w2 * 0.5;
                let ny = top as f32 * TILE_PX - h2;
                if !collides_rect(world, nx, ny, w2, h2) {
                    self.x = nx;
                    self.y = ny;
                    self.vx = 0.0;
                    self.vy = 0.0;
                    self.on_ground = true;
                }
            }
        }
        self.vy = (self.vy + GRAVITY).min(MAX_FALL);
        let nx = self.x + self.vx;
        if !collides_rect(world, nx, self.y, w, h) {
            self.x = nx;
        } else {
            self.vx = -self.vx * 0.4;
        }
        let ny = self.y + self.vy;
        let was_air = !self.on_ground;
        if !collides_rect(world, self.x, ny, w, h) {
            self.y = ny;
            self.on_ground = false;
        } else if self.vy > 0.0 {
            let ty = ((ny + h) / TILE_PX).floor() as i32;
            self.y = ty as f32 * TILE_PX - h;
            self.vy = 0.0;
            self.on_ground = true;
            self.vx *= 0.5;
            if was_air {
                self.wants_minion = true;
            }
        } else if self.vy < 0.0 {
            let ty = (ny / TILE_PX).floor() as i32;
            self.y = (ty + 1) as f32 * TILE_PX;
            self.vy = 0.0;
        }
        if self.y > world.height as f32 * TILE_PX + 400.0 {
            self.hp = 0;
        }
    }

    /// The contact overlap test (`contact`'s AABB) as a pure read —
    /// `Player.StatusFromNPC`'s call site (`Player.cs:31425`) runs the
    /// per-NPC debuff rolls on the same touch branch.
    pub fn touches(&self, player: &Player) -> bool {
        let (w, h) = self.size();
        self.x < player.x + crate::player::WIDTH
            && self.x + w > player.x
            && self.y < player.y + crate::player::HEIGHT
            && self.y + h > player.y
    }

    /// `AI_007_TownNPCs`'s walk subset (`NPC.cs:10586+`): the town NPC
    /// wanders on the ground, turns at the walls/ledges, jumps the
    /// one-tile obstacles. The chase/doors/talk arms land with their
    /// systems.
    fn update_guide(&mut self, world: &World, _dt: f32) {
        let (w, h) = NpcKind::Guide.size();
        // The wander: a walking pace with the random turn every few
        // seconds (`ai[3]`'s idle timer shape).
        self.anim += 1.0;
        // The deterministic turn: every 240 ticks the wander flips (the
        // per-NPC randomness rides the shared streams' order).
        if self.anim % 240.0 < 1.0 {
            self.facing = -self.facing;
        }
        self.vx = self.facing as f32 * 0.5;
        self.vy = (self.vy + GRAVITY).min(MAX_FALL);

        let nx = self.x + self.vx;
        if !collides_rect(world, nx, self.y, w, h) {
            self.x = nx;
        } else if self.on_ground {
            self.vy = -6.2;
            self.on_ground = false;
        } else {
            self.vx = 0.0;
        }

        let ny = self.y + self.vy;
        if !collides_rect(world, self.x, ny, w, h) {
            self.y = ny;
            self.on_ground = false;
        } else if self.vy > 0.0 {
            let ty = ((ny + h) / TILE_PX).floor() as i32;
            self.y = ty as f32 * TILE_PX - h;
            self.vy = 0.0;
            self.on_ground = true;
        } else if self.vy < 0.0 {
            let ty = (ny / TILE_PX).floor() as i32;
            self.y = (ty + 1) as f32 * TILE_PX;
            self.vy = 0.0;
        }
    }

    /// `AI_003_Fighters` for the skeleton family (`NPC.cs:10586+`): the
    /// cave walkers chase the player through the framework's walker step
    /// with the vanilla-cited tunables — the 15-frame strip rides the same
    /// accumulator (the frame wraps over `Main.npcFrameCount[21] = 15`,
    /// `Main.cs:67082`).
    fn update_skeleton(&mut self, world: &World, player: &mut Player, _dt: f32) {
        let (pcx, pcy) = player.center();
        crate::npc_ai::walker_step(
            self,
            world,
            pcx,
            pcy,
            crate::npc_ai::WalkerParams {
                // `NPC.cs` case 21: velocity.X target ~0.8 (the skeleton is
                // a faster zombie), the jump -6.2, the aggro the zombie's
                // 80x40-tile window.
                speed: 0.8,
                jump: -6.2,
                aggro_x: 80.0 * TILE_PX,
                aggro_y: 40.0 * TILE_PX,
            },
            GRAVITY,
            MAX_FALL,
        );
        // The frame: the same accumulator as the zombie's, wrapping over
        // the 15-frame strip.
        self.frame_counter += (self.vx != 0.0) as i32 as f32 + 0.6;
        if self.frame_counter >= 8.0 {
            self.frame_counter = 0.0;
            self.frame = (self.frame + 1) % 15;
        }
    }

    /// `AI_014_Bats`' core (`NPC.cs` case 49's aiStyle): the erratic flight
    /// through the framework's flyer step with the direction perturbation
    /// on the aiTimer (the vanilla randomizes velocity.X/Y every ~30 ticks
    /// toward the player).
    fn update_cave_bat(&mut self, world: &World, player: &mut Player, _dt: f32) {
        let (pcx, pcy) = player.center();
        let (cx, cy) = self.center();
        // The vanilla: `AI_014`'s velocity reroll rides `ai[1]` counting to
        // 30 (`NPC.cs` case 49's AI block) — the deterministic hash stands
        // in for `Main.rand` (the shared streams' order).
        self.anim += 1.0;
        if self.anim % 30.0 < 1.0 {
            let jitter = ((self.x as i32).wrapping_mul(0x4982_2D4Eu32 as i32) >> 24) & 0x7;
            self.vx = if jitter % 2 == 0 { 1.2 } else { -1.2 };
            self.vy = if jitter < 4 { -0.8 } else { 0.8 };
        }
        crate::npc_ai::flyer_step(
            self,
            world,
            pcx,
            pcy,
            crate::npc_ai::FlyerParams {
                accel: 0.15,
                max_speed: 2.5,
            },
        );
        let _ = (cx, cy);
        // The 5-frame flap (`Main.npcFrameCount[49] = 5`).
        self.frame_counter += 1.0;
        if self.frame_counter >= 4.0 {
            self.frame_counter = 0.0;
            self.frame = (self.frame + 1) % 5;
        }
    }

    /// `AI_005_EaterOfSouls`'s hover core through the framework's
    /// `hover_step` — the no-gravity fly toward the player with the vanilla
    /// tunables; the 2-frame/3-frame flaps ride the anim clock.
    fn update_hover(&mut self, world: &World, player: &Player, params: HoverParams) {
        let (pcx, pcy) = player.center();
        crate::npc_ai::hover_step(self, world, pcx, pcy, params);
        self.frame_counter += 1.0;
        if self.frame_counter >= 8.0 {
            self.frame_counter = 0.0;
            let count = match self.kind {
                NpcKind::Hornet => 3,
                _ => 2,
            };
            self.frame = if self.frame + 1 >= count {
                0
            } else {
                self.frame + 1
            };
        }
    }

    /// `AI_008` through the framework's `caster_cycle` (`NPC.cs`'s aiStyle
    /// 8 block, `:20817-21278`). The shoot tick spawns the sphere through
    /// `wants_sphere`: the Goblin Sorcerer → the Chaos Ball 30 (`:21238`),
    /// Tim → its ChaosBallTim clone 665 (`:21242`, folded into `ChaosBall`
    /// — identical stats and name), the Dark Caster → the Water Sphere 33
    /// (`:21246`), the Fire Imp → the Burning Sphere 25 at `ai[1] == 10`
    /// (`:21164-21173`). The spawn point keeps `NPC.NewNPC`'s `Bottom`
    /// semantics (`:80771`).
    fn update_caster(&mut self, world: &World, player: &Player, rng: &mut u64) {
        let (pcx, pcy) = player.center();
        let params = match self.kind {
            NpcKind::FireImp => crate::npc_ai::CasterParams::FIRE_IMP,
            _ => crate::npc_ai::CasterParams::STANDARD,
        };
        let mut phase = crate::npc_ai::CasterPhase {
            counter: self.caster_counter,
            windup: self.cast_windup,
        };
        let shot = crate::npc_ai::caster_cycle(self, world, pcx, pcy, rng, params, &mut phase);
        self.caster_counter = phase.counter;
        self.cast_windup = phase.windup;
        if shot {
            let (w, _) = self.size();
            let (sx, sy) = match self.kind {
                // `:21238`/`:21242`/`:21246`: `(pos.X + width/2, pos.Y - 8)`.
                NpcKind::GoblinSorcerer | NpcKind::Tim | NpcKind::DarkCaster => {
                    (self.x + w * 0.5, self.y - 8.0)
                }
                // `:21171`: `(pos.X + width/2 + direction*8, pos.Y + 20)`.
                NpcKind::FireImp => (self.x + w * 0.5 + self.facing as f32 * 8.0, self.y + 20.0),
                _ => return,
            };
            let sphere = match self.kind {
                NpcKind::GoblinSorcerer | NpcKind::Tim => NpcKind::ChaosBall,
                NpcKind::DarkCaster => NpcKind::WaterSphere,
                NpcKind::FireImp => NpcKind::BurningSphere,
                _ => return,
            };
            self.wants_sphere = Some((sphere, sx, sy));
        }
    }

    /// `AI_043`'s Queen Bee v1 (`NPC.cs:30190+`) — the EoC's cycle shape
    /// with the bee's numbers: the hover (the ±0.07 accels, the 8 cap),
    /// the charge (the 480-tick cycle, the direct velocity x 7), the decel
    /// (the 40 ticks, x0.98), the flee (`ai[0]==5`'s world-edge drift —
    /// the v1 rises away). The Servant bees and the enraged-outside-the-
    /// jungle state = the follow-ups.
    fn update_queen_bee(&mut self, world: &World, player: &Player) {
        let (w, h) = self.size();
        let (pcx, pcy) = player.center();
        let cx = self.x + w * 0.5;
        let cy = self.y + h * 0.5;
        // The flee: rise away (the vanilla's `ai[0]==5` drift — the v1
        // rises, the off-world guard despawns).
        if self.cast_windup == 3 {
            self.vy -= 0.04;
            self.x += self.vx;
            self.y += self.vy;
            if self.y < -500.0 {
                self.hp = 0;
            }
            return;
        }
        let hover_y = pcy - 260.0;
        if self.cast_windup == 0 {
            let dx = pcx - cx;
            self.vx += dx.signum() * 0.07;
            if self.vx.abs() > 7.0 {
                self.vx = 7.0 * self.vx.signum();
            }
            let dy = hover_y - cy;
            self.vy += dy.signum() * 0.07;
            if self.vy.abs() > 7.0 {
                self.vy = 7.0 * self.vy.signum();
            }
            self.caster_counter += 1;
            if self.caster_counter >= 480 {
                self.cast_windup = 1;
                self.caster_counter = 0;
            }
        } else if self.cast_windup == 1 {
            let dx = pcx - cx;
            let dy = pcy - cy;
            let d = (dx * dx + dy * dy).sqrt().max(1.0);
            self.vx = dx / d * 7.0;
            self.vy = dy / d * 7.0;
            self.cast_windup = 2;
        } else {
            self.caster_counter += 1;
            if self.caster_counter >= 40 {
                self.vx *= 0.98;
                self.vy *= 0.98;
                if self.vx.abs() < 0.1 {
                    self.vx = 0.0;
                }
                if self.vy.abs() < 0.1 {
                    self.vy = 0.0;
                }
                if self.vx == 0.0 && self.vy == 0.0 {
                    self.cast_windup = 0;
                    self.caster_counter = 0;
                }
            }
        }
        self.facing = if self.vx >= 0.0 { 1 } else { -1 };
        self.x += self.vx;
        self.y += self.vy;
        if self.y > world.height as f32 * TILE_PX + 400.0 {
            self.hp = 0;
        }
    }

    /// `AI_011`'s Skeletron head v1 (`NPC.cs`'s aiStyle 11 machine) — the
    /// EoC's cycle shape with the skull's numbers: the hover (the ±0.07,
    /// the 7 cap), the charge (the 480-tick cycle, the direct velocity
    /// x 8), the decel (the 40 ticks x0.98). The spinning hands (the
    /// separate NPCs 36) = the follow-ups.
    fn update_skeletron(&mut self, world: &World, player: &Player) {
        let (w, h) = self.size();
        let (pcx, pcy) = player.center();
        let cx = self.x + w * 0.5;
        let cy = self.y + h * 0.5;
        let hover_y = pcy - 280.0;
        if self.cast_windup == 0 {
            let dx = pcx - cx;
            self.vx += dx.signum() * 0.07;
            if self.vx.abs() > 7.0 {
                self.vx = 7.0 * self.vx.signum();
            }
            let dy = hover_y - cy;
            self.vy += dy.signum() * 0.07;
            if self.vy.abs() > 7.0 {
                self.vy = 7.0 * self.vy.signum();
            }
            self.caster_counter += 1;
            if self.caster_counter >= 480 {
                self.cast_windup = 1;
                self.caster_counter = 0;
            }
        } else if self.cast_windup == 1 {
            let dx = pcx - cx;
            let dy = pcy - cy;
            let d = (dx * dx + dy * dy).sqrt().max(1.0);
            self.vx = dx / d * 8.0;
            self.vy = dy / d * 8.0;
            self.cast_windup = 2;
        } else {
            self.caster_counter += 1;
            if self.caster_counter >= 40 {
                self.vx *= 0.98;
                self.vy *= 0.98;
                if self.vx.abs() < 0.1 {
                    self.vx = 0.0;
                }
                if self.vy.abs() < 0.1 {
                    self.vy = 0.0;
                }
                if self.vx == 0.0 && self.vy == 0.0 {
                    self.cast_windup = 0;
                    self.caster_counter = 0;
                }
            }
        }
        self.facing = if self.vx >= 0.0 { 1 } else { -1 };
        self.x += self.vx;
        self.y += self.vy;
        if self.y > world.height as f32 * TILE_PX + 400.0 {
            self.hp = 0;
        }
    }

    /// `AI()`'s aiStyle-27 block — the Wall of Flesh's machine
    /// (`NPC.cs:25635-26050`, the 416-line `else if (this.aiStyle == 27)`).
    /// The classic path verbatim: the off-edge kill, the tongue timer
    /// (`ai[1]`/`ai[2]` → the Leech spawns), the underworld draw-band scan
    /// (`Main.wofDrawArea*` → the World fields) with the Y pinning, the
    /// slide speed by the hp thresholds, the direction lock, the
    /// dead-player abandonment (the `localAI[1]` timer folded into
    /// `ai[0]`), the init Hungry spawns (`localAI[0]` → `ai[3]`).
    /// The expert-mode burst block and the `getGoodWorld` spawn are out
    /// (no difficulty flags); the wall's eye/mouth mirrors (114) are the
    /// render-machinery follow-up, so the init spawns the 11 Hungries
    /// only; the roar sound rides `localAI[3]`'s cosmetic timer — skipped.
    fn update_wof(&mut self, world: &World, player: &Player) {
        let (w, h) = self.size();
        let (pcx, pcy) = player.center();
        // The world-edge kill (`position.X < 160 || > (maxTilesX - 10) * 16`).
        if self.x < 160.0 || self.x > (world.width as f32 - 10.0) * TILE_PX {
            self.hp = 0;
            return;
        }
        // The tongue timer (`ai[1]` with the hp-ratio bonuses; 2700 → the
        // phase machine `ai[2]`).
        self.ai[1] += 1.0;
        if self.ai[2] == 0.0 {
            if (self.hp as f64) < (self.max_hp as f64) * 0.5 {
                self.ai[1] += 1.0;
            }
            if (self.hp as f64) < (self.max_hp as f64) * 0.2 {
                self.ai[1] += 1.0;
            }
            if self.ai[1] > 2700.0 {
                self.ai[2] = 1.0;
            }
        }
        if self.ai[2] > 0.0 && self.ai[1] > 60.0 {
            let mut num344 = 3;
            if (self.hp as f64) < (self.max_hp as f64) * 0.3 {
                num344 += 1;
            }
            self.ai[2] += 1.0;
            self.ai[1] = 0.0;
            if self.ai[2] > num344 as f32 {
                self.ai[2] = 0.0;
            }
            // The Leech head spawn (`NPC.CountNPCS(117) < 10` — the count
            // check rides the sim resolve; the worm pass grows the chain).
            self.wants_leech = true;
        }
        // The underworld draw-band scan (`num346` = UnderworldLayer + 10,
        // `num347` = +70): the bottom walk finds 15 solid/liquid tiles down
        // the wall's column span, the top walk mirrors it; the areas ease
        // 1 px per tick toward the found rows and clamp to a 160 px band.
        // `Main.UnderworldLayer = maxTilesY - 200` (`Main.cs:2790`).
        let underworld_layer = world.height as i32 - 200;
        let num346 = underworld_layer + 10;
        let num347 = num346 + 70;
        let num348 = (self.x / TILE_PX) as i32;
        let num349 = ((self.x + w) / TILE_PX) as i32;
        let num350 = ((self.y + h * 0.5) / TILE_PX) as i32;
        // The `WorldGen.SolidTile` analog (the active + the solid set).
        let solid_tile = |x: i32, y: i32| -> bool {
            world
                .tile(x, y)
                .is_some_and(|t| t.active() && tr_core::tiledata::TILE_SOLID.has(t.kind))
        };
        // The bottom: start at +7, walk down until 15 solid/liquid tiles.
        let mut num351 = 0;
        let mut num352 = num350 + 7;
        while num351 < 15 && num352 > underworld_layer {
            num352 += 1;
            if num352 > world.height as i32 - 10 {
                num352 = world.height as i32 - 10;
                break;
            }
            if num352 < num346 {
                continue;
            }
            for num353 in num348..=num349 {
                if solid_tile(num353, num352) {
                    num351 += 1;
                }
            }
        }
        num352 += 4;
        let bottom_target = num352 as f32 * TILE_PX;
        self.wof_bottom_target = bottom_target;
        // The top: start at -7, walk up.
        num351 = 0;
        num352 = num350 - 7;
        while num351 < 15 && num352 < world.height as i32 - 10 {
            num352 -= 1;
            if num352 <= 10 {
                num352 = 10;
                break;
            }
            if num352 > num347 {
                continue;
            }
            if num352 < num346 {
                num352 = num346;
                break;
            }
            for num354 in num348..=num349 {
                // The liquid counts toward the band like the solid
                // (`SolidTile || liquid > 0`, `:25747`'s inner test).
                let liquid = world.tile(num354, num352).is_some_and(|t| t.liquid > 0);
                if solid_tile(num354, num352) || liquid {
                    num351 += 1;
                }
            }
        }
        num352 -= 4;
        let top_target = num352 as f32 * TILE_PX;
        self.wof_top_target = top_target;
        // The band lives on the World (the sim eases toward the targets);
        // the Y pinning rides the CURRENT band midpoint.
        let num355 = (world.wof_draw_bottom + world.wof_draw_top) / 2.0 - h / 2.0;
        self.vy = 0.0;
        self.y = num355;
        // The slide speed by the hp thresholds (the classic rows; the
        // expert/getGood arms are out).
        let mut num356 = 1.5f32;
        if (self.hp as f64) < (self.max_hp as f64) * 0.75 {
            num356 += 0.25;
        }
        if (self.hp as f64) < (self.max_hp as f64) * 0.5 {
            num356 += 0.4;
        }
        if (self.hp as f64) < (self.max_hp as f64) * 0.25 {
            num356 += 0.5;
        }
        if (self.hp as f64) < (self.max_hp as f64) * 0.1 {
            num356 += 0.6;
        }
        // The direction: locked to the sign, never damped (the wall slides
        // at the full speed toward its side).
        if self.vx == 0.0 {
            self.facing = if pcx >= self.x + w * 0.5 { 1 } else { -1 };
            self.vx = self.facing as f32;
        }
        if self.vx < 0.0 {
            self.vx = -num356;
            self.facing = -1;
        } else {
            self.vx = num356;
            self.facing = 1;
        }
        // The dead-player abandonment (`localAI[1]` += 1/180 → the wall
        // kills itself at 1; the recovery −1/30; folded into `ai[0]`).
        if player.hp <= 0 {
            self.ai[0] += 1.0 / 180.0;
            if self.ai[0] >= 1.0 {
                self.hp = 0;
                return;
            }
        } else {
            self.ai[0] = (self.ai[0] - 1.0 / 30.0).clamp(0.0, 1.0);
        }
        self.facing = self.vx.signum() as i32;
        // The rotation: the eye aims at the player when the player crosses
        // the wall's plane (`:25992-26010`).
        let cx = self.x + w * 0.5;
        let cy = self.y + h * 0.5;
        let num361 = pcx - cx;
        let num362 = pcy - cy;
        if self.facing > 0 {
            if pcx > self.x + w * 0.5 {
                self.rotation = (-num362).atan2(-num361) + 3.14;
            } else {
                self.rotation = 0.0;
            }
        } else if pcx < self.x + w * 0.5 {
            self.rotation = num362.atan2(num361) + 3.14;
        } else {
            self.rotation = 0.0;
        }
        // The Hungry init (`localAI[0] == 1 → 2`): the 11 Hungries along
        // the band (`ai[0] = i * 0.1 - 0.05`); the mirrors (114) are the
        // render follow-up. The request resolves in the world sim.
        if self.ai[3] == 0.0 {
            self.ai[3] = 1.0;
            self.wants_hungry = true;
        }
        // `noTileCollide` + `noGravity`: the raw integration.
        self.x += self.vx;
        self.y += self.vy;
    }

    /// `AI()`'s aiStyle-29 block — the Hungry's machine
    /// (`NPC.cs:26054-26210`): the anchor = the wall's x + the draw-band
    /// row at `ai[0]`, the chase toward the player clamped to `num387`
    /// from the anchor (300/500/700 by the wall's hp), the defense/damage
    /// tiers (30/20 by the hp), the 4-speed cap, the justHit recoil
    /// (`ai[1] = 10`, folded onto `hurt_cd > 0`). The wall's death
    /// (`wof_present` off) deactivates the minion. The expert arms are out.
    fn update_hungry(&mut self, world: &World, player: &Player) {
        if !world.wof_present {
            self.hp = 0;
            return;
        }
        // `justHit → ai[1] = 10` (the recoil pause; the hurt cooldown
        // stands in for the flag).
        if self.hurt_cd > 0.0 {
            self.ai[1] = 10.0;
        }
        // The tiers by the wall's hp (`num388` the damage, the defense,
        // `num387` the leash radius).
        let mut num387 = 300.0f32;
        let mut num388 = 0;
        if (world.wof_hp_ratio as f64) < 0.5 {
            num388 = 60;
            self.defense_override = Some(30);
            num387 = 700.0;
        } else if (world.wof_hp_ratio as f64) < 0.75 {
            num388 = 45;
            self.defense_override = Some(20);
            num387 = 500.0;
        } else {
            self.defense_override = None;
        }
        if num388 > 0 {
            self.damage_override = Some(num388);
        } else {
            self.damage_override = None;
        }
        // The dive cycle: after 100 ticks the leash grows ×1.3, resetting
        // at 200.
        self.ai[2] += 1.0;
        if self.ai[2] > 100.0 {
            num387 = (num387 * 1.3) as i32 as f32;
            if self.ai[2] > 200.0 {
                self.ai[2] = 0.0;
            }
        }
        // The anchor: the wall's x, the band row at `ai[0]`.
        let num389 = world.wof_x;
        let num390 = world.wof_draw_bottom - world.wof_draw_top;
        let y2 = world.wof_draw_top + num390 * self.ai[0];
        let (pcx, pcy) = player.center();
        let (w, h) = self.size();
        let mut num391 = pcx - w * 0.5 - num389;
        let mut num392 = pcy - h * 0.5 - y2;
        let mut num393 = (num391 * num391 + num392 * num392).sqrt();
        if self.ai[1] == 0.0 {
            if num393 > num387 {
                num393 = num387 / num393;
                num391 *= num393;
                num392 *= num393;
            }
            // The approach with the double-add on the sign flips
            // (0.1 accel, the ×2.5 flip bonus).
            if self.x < num389 + num391 {
                self.vx += 0.1;
                if self.vx < 0.0 && num391 > 0.0 {
                    self.vx += 0.25;
                }
            } else if self.x > num389 + num391 {
                self.vx -= 0.1;
                if self.vx > 0.0 && num391 < 0.0 {
                    self.vx -= 0.25;
                }
            }
            if self.y < y2 + num392 {
                self.vy += 0.1;
                if self.vy < 0.0 && num392 > 0.0 {
                    self.vy += 0.25;
                }
            } else if self.y > y2 + num392 {
                self.vy -= 0.1;
                if self.vy > 0.0 && num392 < 0.0 {
                    self.vy -= 0.25;
                }
            }
            // The speed cap 4.
            self.vx = self.vx.clamp(-4.0, 4.0);
            self.vy = self.vy.clamp(-4.0, 4.0);
        } else if self.ai[1] > 0.0 {
            self.ai[1] -= 1.0;
        }
        // The rotation toward the player.
        if num391 > 0.0 {
            self.rotation = num392.atan2(num391);
        }
        if num391 < 0.0 {
            self.rotation = num392.atan2(num391);
        }
        // `noTileCollide` + `noGravity`: the raw integration.
        self.x += self.vx;
        self.y += self.vy;
    }

    /// `AI_032`'s Skeletron Prime v1 (`NPC.cs`'s aiStyle 32 machine) — the
    /// EoC cycle shape with the Prime's numbers: the hover (the ±0.12,
    /// the 9 cap), the charge (the 480-tick cycle, the direct velocity
    /// x 9), the decel (the 40 ticks x0.98). The 4 limbs = the follow-ups.
    fn update_prime(&mut self, world: &World, player: &Player) {
        let (w, h) = self.size();
        let (pcx, pcy) = player.center();
        let cx = self.x + w * 0.5;
        let cy = self.y + h * 0.5;
        let hover_y = pcy - 280.0;
        if self.cast_windup == 0 {
            let dx = pcx - cx;
            self.vx += dx.signum() * 0.12;
            if self.vx.abs() > 9.0 {
                self.vx = 9.0 * self.vx.signum();
            }
            let dy = hover_y - cy;
            self.vy += dy.signum() * 0.12;
            if self.vy.abs() > 9.0 {
                self.vy = 9.0 * self.vy.signum();
            }
            self.caster_counter += 1;
            if self.caster_counter >= 480 {
                self.cast_windup = 1;
                self.caster_counter = 0;
            }
        } else if self.cast_windup == 1 {
            let dx = pcx - cx;
            let dy = pcy - cy;
            let d = (dx * dx + dy * dy).sqrt().max(1.0);
            self.vx = dx / d * 9.0;
            self.vy = dy / d * 9.0;
            self.cast_windup = 2;
        } else {
            self.caster_counter += 1;
            if self.caster_counter >= 40 {
                self.vx *= 0.98;
                self.vy *= 0.98;
                if self.vx.abs() < 0.1 {
                    self.vx = 0.0;
                }
                if self.vy.abs() < 0.1 {
                    self.vy = 0.0;
                }
                if self.vx == 0.0 && self.vy == 0.0 {
                    self.cast_windup = 0;
                    self.caster_counter = 0;
                }
            }
        }
        self.facing = if self.vx >= 0.0 { 1 } else { -1 };
        self.x += self.vx;
        self.y += self.vy;
        if self.y > world.height as f32 * TILE_PX + 400.0 {
            self.hp = 0;
        }
    }

    /// `AI_030`/`AI_031`'s Twins v1 (`NPC.cs`'s aiStyle 30/31 machines) —
    /// the EoC cycle shape per eye: the hover (the ±0.09, the 8.5 cap),
    /// the charge (the 480-tick cycle, the direct velocity x 8.5), the
    /// decel (the 40 ticks x0.98). The laser/spit phases and the second-
    /// form transformations = the follow-ups.
    fn update_twin(&mut self, world: &World, player: &Player) {
        let (w, h) = self.size();
        let (pcx, pcy) = player.center();
        let cx = self.x + w * 0.5;
        let cy = self.y + h * 0.5;
        let hover_y = pcy - 270.0;
        if self.cast_windup == 0 {
            let dx = pcx - cx;
            self.vx += dx.signum() * 0.09;
            if self.vx.abs() > 8.5 {
                self.vx = 8.5 * self.vx.signum();
            }
            let dy = hover_y - cy;
            self.vy += dy.signum() * 0.09;
            if self.vy.abs() > 8.5 {
                self.vy = 8.5 * self.vy.signum();
            }
            self.caster_counter += 1;
            if self.caster_counter >= 480 {
                self.cast_windup = 1;
                self.caster_counter = 0;
            }
        } else if self.cast_windup == 1 {
            let dx = pcx - cx;
            let dy = pcy - cy;
            let d = (dx * dx + dy * dy).sqrt().max(1.0);
            self.vx = dx / d * 8.5;
            self.vy = dy / d * 8.5;
            self.cast_windup = 2;
        } else {
            self.caster_counter += 1;
            if self.caster_counter >= 40 {
                self.vx *= 0.98;
                self.vy *= 0.98;
                if self.vx.abs() < 0.1 {
                    self.vx = 0.0;
                }
                if self.vy.abs() < 0.1 {
                    self.vy = 0.0;
                }
                if self.vx == 0.0 && self.vy == 0.0 {
                    self.cast_windup = 0;
                    self.caster_counter = 0;
                }
            }
        }
        self.facing = if self.vx >= 0.0 { 1 } else { -1 };
        self.x += self.vx;
        self.y += self.vy;
        if self.y > world.height as f32 * TILE_PX + 400.0 {
            self.hp = 0;
        }
    }

    /// `AI_051`'s Plantera v1 (`NPC.cs`'s aiStyle 51 machine) — the EoC
    /// cycle shape with Plantera's numbers: the hover (the ±0.1, the 8
    /// cap), the charge (the 480-tick cycle, the direct velocity x 7.5),
    /// the decel (the 40 ticks x0.98). The vine hooks and the second-form
    /// rage = the follow-ups.
    fn update_plantera(&mut self, world: &World, player: &Player) {
        let (w, h) = self.size();
        let (pcx, pcy) = player.center();
        let cx = self.x + w * 0.5;
        let cy = self.y + h * 0.5;
        let hover_y = pcy - 250.0;
        if self.cast_windup == 0 {
            let dx = pcx - cx;
            self.vx += dx.signum() * 0.1;
            if self.vx.abs() > 8.0 {
                self.vx = 8.0 * self.vx.signum();
            }
            let dy = hover_y - cy;
            self.vy += dy.signum() * 0.1;
            if self.vy.abs() > 8.0 {
                self.vy = 8.0 * self.vy.signum();
            }
            self.caster_counter += 1;
            if self.caster_counter >= 480 {
                self.cast_windup = 1;
                self.caster_counter = 0;
            }
        } else if self.cast_windup == 1 {
            let dx = pcx - cx;
            let dy = pcy - cy;
            let d = (dx * dx + dy * dy).sqrt().max(1.0);
            self.vx = dx / d * 7.5;
            self.vy = dy / d * 7.5;
            self.cast_windup = 2;
        } else {
            self.caster_counter += 1;
            if self.caster_counter >= 40 {
                self.vx *= 0.98;
                self.vy *= 0.98;
                if self.vx.abs() < 0.1 {
                    self.vx = 0.0;
                }
                if self.vy.abs() < 0.1 {
                    self.vy = 0.0;
                }
                if self.vx == 0.0 && self.vy == 0.0 {
                    self.cast_windup = 0;
                    self.caster_counter = 0;
                }
            }
        }
        self.facing = if self.vx >= 0.0 { 1 } else { -1 };
        self.x += self.vx;
        self.y += self.vy;
        if self.y > world.height as f32 * TILE_PX + 400.0 {
            self.hp = 0;
        }
    }

    /// `AI_054`'s Brain of Cthulhu v1 (`NPC.cs`'s aiStyle 54 machine) —
    /// the teleport-charge cycle: the telegraph pause, the dash at the
    /// player; the `dontTakeDamage` until the Creepers die (the vanilla's
    /// `ai[3]` tracks them). The v1: the teleport-dash cycle without the
    /// invulnerability gate (the Creepers' orbit = the follow-ups).
    fn update_brain(&mut self, world: &World, player: &Player) {
        let (w, h) = self.size();
        let (pcx, pcy) = player.center();
        let cx = self.x + w * 0.5;
        let cy = self.y + h * 0.5;
        if self.caster_counter == 0 && self.cast_windup == 0 {
            self.caster_counter = 1;
        }
        if self.cast_windup == 2 && self.caster_counter > 40 {
            let dx = pcx - cx;
            let dy = pcy - cy;
            let d = (dx * dx + dy * dy).sqrt().max(1.0);
            self.vx = dx / d * 9.0;
            self.vy = dy / d * 9.0;
            self.cast_windup = 3;
            self.caster_counter = 0;
        } else if self.cast_windup == 3 {
            self.vx *= 0.98;
            self.vy *= 0.98;
            if self.vx.abs() < 0.1 && self.vy.abs() < 0.1 {
                self.cast_windup = 0;
                self.caster_counter = 0;
            }
        } else {
            let dx = pcx - cx;
            let dy = pcy - cy;
            self.vx += dx.signum() * 0.05;
            self.vy += dy.signum() * 0.05;
            self.caster_counter += 1;
            if self.caster_counter >= 60 {
                self.cast_windup = 2;
            }
        }
        self.facing = if self.vx >= 0.0 { 1 } else { -1 };
        self.x += self.vx;
        self.y += self.vy;
        if self.y > world.height as f32 * TILE_PX + 400.0 {
            self.hp = 0;
        }
    }

    /// The Goblin AI (`AI_003`'s walker family): the goblins chase the
    /// player and jump the obstacles (the same machine as the zombie).
    fn update_goblin(&mut self, world: &World, player: &mut Player) {
        let (pcx, pcy) = player.center();
        crate::npc_ai::walker_step(
            self,
            world,
            pcx,
            pcy,
            crate::npc_ai::WalkerParams {
                speed: 1.0,
                jump: -6.2,
                aggro_x: 80.0 * TILE_PX,
                aggro_y: 40.0 * TILE_PX,
            },
            GRAVITY,
            MAX_FALL,
        );
    }

    /// `AI_009` through the framework's `sphere_step` (`NPC.cs`'s aiStyle 9
    /// core, `:21404-21429`): the one-shot launch, then the straight fly
    /// through the tiles (`noGravity`/`noTileCollide`).
    fn update_sphere(&mut self, world: &World, player: &Player) {
        let (pcx, pcy) = player.center();
        crate::npc_ai::sphere_step(self, pcx, pcy, world);
    }

    /// `AI_016` through the framework's `swim_step` (`NPC.cs`'s aiStyle 16
    /// block): the wet path rides the machine — `flag25` folds the kind
    /// gate (`:23701`'s Goldfish exclusion), the player's `wet`/alive and
    /// the `CanHit` stand-in; the dry path is the beached flop (`:23963-
    /// 23989`: the Shark's `velocity.X *= 0.94` snap, the others' random
    /// jump through `Main.rand`).
    fn update_swimmer(&mut self, world: &World, player: &Player, rng: &mut u64) {
        let (w, h) = self.size();
        let params = match self.kind {
            NpcKind::Shark => crate::npc_ai::SwimParams::SHARK,
            NpcKind::Piranha => crate::npc_ai::SwimParams::PIRANHA,
            _ => crate::npc_ai::SwimParams::GOLDFISH,
        };
        if is_wet(world, self.x, self.y, w, h) {
            let (pcx, pcy) = player.center();
            let (cx, cy) = (self.x + w * 0.5, self.y + h * 0.5);
            // `flag25` (`:23701-23707`): the kind gate + the player wet +
            // alive + `Collision.CanHit`.
            let aggro_ok = params.aggro
                && player.wet
                && !player.dead
                && crate::npc_ai::line_clear(world, (cx, cy), (pcx, pcy));
            crate::npc_ai::swim_step(self, world, pcx, pcy, aggro_ok, params);
        } else {
            // The beached flop (`:23963-23989`).
            if self.kind == NpcKind::Shark {
                // `:23967-23974`: the Shark's stand decay.
                if self.vy == 0.0 {
                    self.vx *= 0.94;
                    if self.vx > -0.2 && self.vx < 0.2 {
                        self.vx = 0.0;
                    }
                }
            } else {
                // `:23975-23981`: the random jump (`Next(-50,-20)*0.1`,
                // `Next(-20,20)*0.1`, the direction coin).
                self.vy = -(crate::app::frame::rand01(rng) * 2.9 + 2.1);
                self.vx = (crate::app::frame::rand01(rng) * 39.0 - 20.0) * 0.1;
                self.facing = if crate::app::frame::rand01(rng) < 0.5 {
                    1
                } else {
                    -1
                };
            }
            // `:23983-23988`: the gravity + the sink phase.
            self.vy = (self.vy + 0.3).min(10.0);
            self.swim_phase = 1.0;
            // The generic tile integration (`UpdateCollision`): the flop
            // stops at the walls (the wet path's bounce flips ride the wet
            // branch only).
            let nx = self.x + self.vx;
            if !collides_rect(world, nx, self.y, w, h) {
                self.x = nx;
            } else {
                self.vx = 0.0;
            }
            let ny = self.y + self.vy;
            if !collides_rect(world, self.x, ny, w, h) {
                self.y = ny;
                self.on_ground = false;
            } else if self.vy > 0.0 {
                let ty = ((ny + h) / TILE_PX).floor() as i32;
                self.y = ty as f32 * TILE_PX - h;
                self.vy = 0.0;
                self.on_ground = true;
            } else {
                self.vy = 0.0;
            }
        }
    }

    fn contact(&mut self, player: &mut Player) {
        let (w, h) = self.size();
        if self.x < player.x + crate::player::WIDTH
            && self.x + w > player.x
            && self.y < player.y + crate::player::HEIGHT
            && self.y + h > player.y
        {
            let (cx, _) = self.center();
            // `Player.Update_NPCCollision`'s EoC shield hit
            // (`Player.cs:21049-21116`): while `dash == 2 && eocDash > 0` the
            // dash deals 30 x meleeDamage with knockback 9, ends itself
            // (`dashDelay = 30`), bounces the player back, and grants the
            // 4-tick collision immunity. The vanilla hitbox is expanded by 8
            // and offset by half the velocity (the plain overlap stands in);
            // the crit roll (4% base) waits for the combat rng pass.
            if player.dash == 2 && player.eoc_dash > 0 {
                let dmg = (30.0 * player.melee_damage) as i32;
                self.hurt(dmg, cx, 9.0, false);
                player.eoc_dash = 10;
                player.dash_delay = 30;
                let dir = if player.vx != 0.0 {
                    player.vx.signum() as i32
                } else {
                    player.facing
                };
                player.vx = -(dir as f32) * 9.0;
                player.vy = -4.0;
                player.on_ground = false;
                player.hurt_cd = player.hurt_cd.max(4.0 / 60.0);
                return;
            }
            // The town NPCs never hurt the player (`aiStyle 7`'s no-contact);
            // the 0-damage kinds (the Goldfish) never hurt either (the
            // vanilla `damage > 0` gate).
            // The town NPCs never hurt the player (`aiStyle 7`'s no-contact;
            // the M4-TOWN-4's gate loss fixed here — the Merchant's dmg 10
            // reached the hurt path until this).
            if !matches!(
                self.kind,
                NpcKind::Guide
                    | NpcKind::Merchant
                    | NpcKind::Dryad
                    | NpcKind::Nurse
                    | NpcKind::ArmsDealer
                    | NpcKind::Demolitionist
                    | NpcKind::Painter
                    | NpcKind::WitchDoctor
                    | NpcKind::Clothier
                    | NpcKind::EyeOfCthulhu
                    | NpcKind::BoundGoblin
                    | NpcKind::GoblinTinkerer
            ) {
                let dmg = if self.kind == NpcKind::Slime {
                    self.slime.stats().damage
                } else {
                    self.damage_override
                        .unwrap_or_else(|| self.kind.contact_damage())
                };
                if dmg > 0 {
                    player.hurt(dmg, cx);
                }
            }
        }
    }

    pub fn render_key(&self) -> u32 {
        let mut h = self.x.to_bits().wrapping_mul(0x9E37_79B1);
        h ^= self.y.to_bits().wrapping_mul(0x85EB_CA77);
        h ^= (self.hp.max(0) as u32) << 4;
        h ^= (self.facing as u32) << 1;
        h ^= (self.hurt_cd > 0.0) as u32;
        // The rendered frame and the eye's aim tilt are the animation state the
        // world pass bakes in (`frame_counter`/`anim` only matter through them).
        h ^= self.frame << 2;
        h ^= self.rotation.to_bits().wrapping_mul(0x27D4_EB2F);
        h ^= match self.kind {
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
            NpcKind::Tim => 16,
            NpcKind::DarkCaster => 17,
            NpcKind::FireImp => 18,
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
            NpcKind::TheDestroyer => 50,
            NpcKind::SkeletronPrime => 51,
            NpcKind::Retinazer => 52,
            NpcKind::Spazmatism => 53,
            NpcKind::Plantera => 54,
            NpcKind::BrainOfCthulhu => 55,
            NpcKind::Creeper => 53,
            NpcKind::GoblinPeon => 54,
            NpcKind::GoblinThief => 55,
            NpcKind::GoblinWarrior => 56,
            NpcKind::GoblinArcher => 57,
            NpcKind::BoundGoblin => 58,
            NpcKind::GoblinTinkerer => 59,
        } << 5;
        if self.kind == NpcKind::Slime {
            h ^= (self.slime as u32) << 9;
        }
        h
    }

    pub fn frame(&self) -> u32 {
        self.frame
    }
}

/// Hash of all NPCs for the G6 world-range key.
pub fn render_key(npcs: &[Npc]) -> u32 {
    let mut h = npcs.len() as u32;
    for n in npcs {
        h = h.wrapping_mul(31).wrapping_add(n.render_key());
    }
    h
}

#[cfg(test)]
mod tests {
    #[test]
    fn the_hover_kinds_spawn_fly_and_map() {
        // The Eater of Souls (6)/Hornet (42)/Meteor Head (23): the hovers
        // spawn in the open air, fly via the hover machine, the bestiary
        // entries round-trip.
        let mut w = World::new("t".into(), 64, 800);
        for x in 0..64 {
            for y in 60..800 {
                let i = w.index(x, y);
                w.tiles[i] = tr_core::Tile {
                    kind: 1,
                    ..Default::default()
                };
                w.tiles[i].flags |= tr_core::tile::FLAG_ACTIVE;
            }
        }
        w.compute_column_surface();
        let eos = Npc::spawn_hover(&w, 20, 30, NpcKind::EaterOfSouls)
            .expect("the eater spawns in the open air");
        assert_eq!(eos.max_hp, 40);
        assert_eq!(eos.size(), (30.0, 30.0));
        let hornet = Npc::spawn_hover(&w, 24, 30, NpcKind::Hornet).expect("the hornet spawns");
        assert_eq!(hornet.max_hp, 48);
        let meteor =
            Npc::spawn_hover(&w, 28, 30, NpcKind::MeteorHead).expect("the meteor head spawns");
        assert_eq!(meteor.max_hp, 26);
        // The bestiary entries round-trip.
        let e = crate::bestiary::entry_for_npc(NpcKind::EaterOfSouls, SlimeVariant::Blue);
        assert_eq!(e.net_id, 6);
        let e = crate::bestiary::entry_for_npc(NpcKind::Hornet, SlimeVariant::Blue);
        assert_eq!(e.net_id, 42);
        let e = crate::bestiary::entry_for_npc(NpcKind::MeteorHead, SlimeVariant::Blue);
        assert_eq!(e.net_id, 23);
    }

    #[test]
    fn the_hover_chases_the_target() {
        // The hover machine: the Eater of Souls homes to the target over
        // time (the vanilla num 4 / num2 0.02 pair).
        let mut w = World::new("t".into(), 200, 200);
        for x in 0..200 {
            for y in 120..200 {
                let i = w.index(x, y);
                w.tiles[i] = tr_core::Tile {
                    kind: 1,
                    ..Default::default()
                };
                w.tiles[i].flags |= tr_core::tile::FLAG_ACTIVE;
            }
        }
        w.compute_column_surface();
        let mut npc = Npc::new(NpcKind::EaterOfSouls, 90.0 * 16.0, 100.0 * 16.0, 1, 0.0);
        let target = (60.0 * 16.0, 100.0 * 16.0);
        for _ in 0..200 {
            crate::npc_ai::hover_step(
                &mut npc,
                &w,
                target.0,
                target.1,
                crate::npc_ai::HoverParams::EATER_OF_SOULS,
            );
        }
        let dist = ((npc.x - target.0) / 16.0).abs();
        assert!(dist < 20.0, "the hover homed (dist {dist} tiles)");
    }

    #[test]
    fn the_cave_walker_spawn_and_the_bestiary_map() {
        // The Skeleton (21)/Undead Miner (44): the cave walkers spawn on
        // the ground, the bestiary entries round-trip via entry_for_npc.
        let mut w = World::new("t".into(), 64, 800);
        for x in 0..64 {
            for y in 40..800 {
                let i = w.index(x, y);
                w.tiles[i] = tr_core::Tile {
                    kind: 1,
                    ..Default::default()
                };
                w.tiles[i].flags |= tr_core::tile::FLAG_ACTIVE;
            }
        }
        w.compute_column_surface();
        let sk = Npc::spawn_cave_walker(&w, 20, NpcKind::Skeleton).expect("the skeleton spawns");
        assert_eq!(sk.kind, NpcKind::Skeleton);
        assert_eq!(sk.max_hp, 60);
        let um = Npc::spawn_cave_walker(&w, 24, NpcKind::UndeadMiner).expect("the miner spawns");
        assert_eq!(um.max_hp, 70);
        // The bestiary credit ids map.
        let e = crate::bestiary::entry_for_npc(NpcKind::Skeleton, SlimeVariant::Blue);
        assert_eq!(e.net_id, 21);
        let e = crate::bestiary::entry_for_npc(NpcKind::UndeadMiner, SlimeVariant::Blue);
        assert_eq!(e.net_id, 44);
    }

    #[test]
    fn the_cave_bat_spawn_flies_and_bites() {
        // The Cave Bat (49): spawns in the cave air (the open tile), flies
        // via the flyer machine, the contact bites for 13.
        let mut w = World::new("t".into(), 64, 800);
        for x in 0..64 {
            for y in 60..800 {
                let i = w.index(x, y);
                w.tiles[i] = tr_core::Tile {
                    kind: 1,
                    ..Default::default()
                };
                w.tiles[i].flags |= tr_core::tile::FLAG_ACTIVE;
            }
        }
        w.compute_column_surface();
        let mut bat = Npc::spawn_cave_flyer(&w, 20, 30, NpcKind::CaveBat)
            .expect("the bat spawns in the open air");
        assert_eq!(bat.kind, NpcKind::CaveBat);
        assert_eq!(bat.max_hp, 16);
        assert_eq!(bat.size(), (22.0, 18.0));
        // The jungle/hellbat/voodoo variants: the same frame, the vanilla stats.
        let jb =
            Npc::spawn_cave_flyer(&w, 22, 30, NpcKind::JungleBat).expect("the jungle bat spawns");
        assert_eq!(jb.max_hp, 34);
        let hb = Npc::spawn_cave_flyer(&w, 24, 30, NpcKind::Hellbat).expect("the hellbat spawns");
        assert_eq!(hb.max_hp, 46);
        let vd = Npc::spawn_cave_flyer(&w, 26, 30, NpcKind::VoodooDemon)
            .expect("the voodoo demon spawns");
        assert_eq!(vd.max_hp, 140);
        // The contact: an overlapping player takes 13 (the vanilla dmg).
        let mut player = crate::player::Player::spawn(&w);
        player.x = bat.x;
        player.y = bat.y;
        player.hurt_cd = 0.0;
        bat.contact(&mut player);
        assert!(player.hp < player.max_hp, "the bat bit for its contact dmg");
        // The bestiary entry maps.
        let e = crate::bestiary::entry_for_npc(NpcKind::CaveBat, SlimeVariant::Blue);
        assert_eq!(e.net_id, 49);
        let e = crate::bestiary::entry_for_npc(NpcKind::JungleBat, SlimeVariant::Blue);
        assert_eq!(e.net_id, 51);
        let e = crate::bestiary::entry_for_npc(NpcKind::Hellbat, SlimeVariant::Blue);
        assert_eq!(e.net_id, 60);
        let e = crate::bestiary::entry_for_npc(NpcKind::VoodooDemon, SlimeVariant::Blue);
        assert_eq!(e.net_id, 66);
    }

    use super::*;
    use tr_core::tile::{FLAG_ACTIVE, Tile};

    /// The test ground: dirt from y = 120 in a 200x200 world with a carved
    /// pit (x 60..140, y 125..134) — the caster tests' arena. The pit
    /// matters: on a flat surface every floor-level spot sits inside the
    /// player's ±5-tile telefrag rect (the vanilla casters legitimately
    /// fail to teleport there), so the search needs a floor outside the
    /// rect's y window.
    fn ground_air() -> World {
        let mut w = World::new("t".into(), 200, 200);
        for x in 0..200 {
            for y in 120..200 {
                let i = w.index(x, y);
                w.tiles[i] = Tile {
                    kind: 1,
                    ..Default::default()
                };
                w.tiles[i].flags |= FLAG_ACTIVE;
            }
        }
        for x in 60..140 {
            for y in 125..135 {
                let i = w.index(x, y);
                w.tiles[i] = Tile::default();
            }
        }
        w.compute_column_surface();
        w
    }

    /// The player parked on the surface at column 100 (the tests' fixed
    /// reference — `Player::spawn`'s world-spawn default rides the unset
    /// `spawn_x/spawn_y`).
    fn surface_player(w: &World) -> crate::player::Player {
        let mut p = crate::player::Player::spawn(w);
        p.x = 100.0 * TILE_PX;
        p.y = 116.0 * TILE_PX;
        p
    }

    fn ground() -> World {
        let mut w = World::new("t".into(), 64, 64);
        for x in 0..64 {
            for y in 30..64 {
                let i = w.index(x, y);
                w.tiles[i] = Tile {
                    kind: 1,
                    ..Default::default()
                };
                w.tiles[i].flags |= FLAG_ACTIVE;
            }
        }
        w.spawn_x = 20;
        w.spawn_y = 20;
        w.compute_column_surface();
        w.compute_chunks();
        w
    }

    #[test]
    fn slime_falls_onto_the_ground_and_hurts_the_player() {
        let w = ground();
        let mut player = Player::spawn(&w);
        for _ in 0..120 {
            player.update(&w, crate::player::PlayerInput::default());
        }
        let mut slime = Npc::spawn_slime(&w, 20).unwrap();
        for _ in 0..120 {
            slime.update(&w, &mut player, 1.0 / 60.0);
        }
        assert!(slime.on_ground);
        assert!(player.hp < player.max_hp, "hp {}", player.hp);
    }

    #[test]
    fn demon_eye_reaches_the_player() {
        let w = ground();
        let mut player = Player::spawn(&w);
        for _ in 0..120 {
            player.update(&w, crate::player::PlayerInput::default());
        }
        let mut eye = Npc::spawn_eye(&w, 20).unwrap();
        assert_eq!(eye.kind, NpcKind::DemonEye);
        for _ in 0..240 {
            eye.update(&w, &mut player, 1.0 / 60.0);
        }
        assert!(player.hp < player.max_hp, "hp {}", player.hp);
    }

    #[test]
    fn zombie_walks_to_the_player_and_burns_at_dawn() {
        let mut w = ground();
        let mut player = Player::spawn(&w);
        for _ in 0..120 {
            player.update(&w, crate::player::PlayerInput::default());
        }
        let mut zombie = Npc::spawn_zombie(&w, 30).unwrap();
        assert_eq!(zombie.kind, NpcKind::Zombie);
        let start_x = zombie.x;
        for _ in 0..600 {
            zombie.update(&w, &mut player, 1.0 / 60.0);
        }
        assert!(zombie.x < start_x, "zombie walked the wrong way");
        assert!(player.hp < player.max_hp, "hp {}", player.hp);

        // Daylight burns the zombie away.
        w.day_time = true;
        let hp = zombie.hp;
        for _ in 0..600 {
            zombie.update(&w, &mut player, 1.0 / 60.0);
        }
        assert!(zombie.hp < hp, "hp {} stayed at {hp}", zombie.hp);
    }

    #[test]
    fn king_slime_hops_toward_the_player_and_teleports() {
        let w = ground();
        let mut player = Player::spawn(&w);
        for _ in 0..120 {
            player.update(&w, crate::player::PlayerInput::default());
        }
        let mut king = Npc::spawn_king_slime(&w, 34).unwrap();
        assert_eq!(king.hp, 2000);
        let start_x = king.x;
        for _ in 0..1200 {
            king.update(&w, &mut player, 1.0 / 60.0);
        }
        assert!(king.x < start_x, "boss did not approach");
        assert!(player.hp < player.max_hp, "hp {}", player.hp);

        // Outrun the boss: it should teleport back next to the player.
        player.x = 4.0 * TILE_PX;
        player.y = 30.0 * TILE_PX - crate::player::HEIGHT;
        for _ in 0..120 {
            king.update(&w, &mut player, 1.0 / 60.0);
        }
        let dx = ((king.x + 49.0) - (player.x + crate::player::WIDTH * 0.5)).abs();
        assert!(dx < 20.0 * TILE_PX, "boss still {dx} px away");
    }

    #[test]
    fn attacking_kills_a_slime() {
        let w = ground();
        let mut s = Npc::spawn_slime(&w, 20).unwrap();
        for _ in 0..3 {
            assert!(s.hurt(10, 0.0, 5.5, false));
            s.hurt_cd = 0.0;
        }
        // Vanilla keeps `life` as the raw remainder and dies on
        // `life <= 0` (`NPC.checkDead`); the blue slime now carries the
        // vanilla 25 HP
        // (NPC.cs:8572), so three 10-damage hits leave -5.
        assert!(s.hp <= 0);
    }
    #[test]
    fn npc_update_runs_the_buff_tick_before_the_ai() {
        // NPC.cs:90373-90377 — the buff block runs at the top of UpdateNPC;
        // an OnFire slime dies from DoT alone (25 hp, 1 damage per 15 ticks —
        // slimes are immune to Poisoned 20, so the fire row drives the tick).
        let w = ground();
        let mut player = Player::spawn(&w);
        let mut s = Npc::spawn_slime(&w, 34).unwrap();
        s.buffs.add_buff(24, 2000); // OnFire — 8 regenLoss/tick
        for _ in 0..380 {
            s.update(&w, &mut player, 1.0 / 60.0);
        }
        assert_eq!(s.hp, 0, "hp {}", s.hp);
    }

    #[test]
    fn spawned_npcs_carry_their_debuff_immunity_table() {
        // SetDefaults' immunity application (NPC.cs:17818-17837) rides
        // Npc::new → NpcBuffs::for_kind: slimes (type 1) are immune to
        // Poisoned 20 (NPCID.cs:6079-6084) with the derived 30/375; zombies
        // (null entry) have no immunity.
        let s = Npc::spawn_slime(&ground(), 20).unwrap();
        assert!(s.buffs.buff_immune[20]);
        assert!(
            s.buffs.buff_immune[30] && s.buffs.buff_immune[375],
            "derived from 20"
        );
        let z = Npc::spawn_zombie(&ground(), 20).unwrap();
        assert!(!z.buffs.buff_immune[20]);
    }

    #[test]
    fn slime_variants_match_the_setdefaults_lines() {
        // NPC.cs:8567-8580 (blue) and the SetDefaultsFromNetId cases
        // (-3 green, -4 pinky, -5 baby, -6 black, -7 purple, -8 red,
        // -9 yellow).
        let cases: [(SlimeVariant, i32, i32, i32, f32, [u8; 4]); 8] = [
            (SlimeVariant::Blue, 25, 7, 2, 1.0, [0, 80, 255, 100]),
            (SlimeVariant::Green, 14, 6, 0, 0.9, [0, 220, 40, 100]),
            (SlimeVariant::Pinky, 150, 5, 5, 0.6, [250, 30, 90, 90]),
            (SlimeVariant::Baby, 30, 13, 4, 0.9, [0, 0, 0, 50]),
            (SlimeVariant::Black, 45, 15, 4, 1.05, [0, 0, 0, 50]),
            (SlimeVariant::Purple, 40, 12, 6, 1.2, [200, 0, 255, 150]),
            (SlimeVariant::Red, 35, 12, 4, 1.025, [255, 30, 0, 100]),
            (SlimeVariant::Yellow, 45, 15, 7, 1.2, [255, 255, 0, 100]),
        ];
        for (v, life, damage, defense, scale, color) in cases {
            let st = v.stats();
            assert_eq!(
                (st.life, st.damage, st.defense, st.scale, st.color),
                (life, damage, defense, scale, color),
                "variant {:?}",
                v
            );
        }
        // ItemDropDatabase.cs:1074-1082: the standard slimes drop 1-2 gel,
        // the red/purple/black family 2-5, green/baby/pinky none.
        assert_eq!(SlimeVariant::Blue.stats().gel, (1, 2));
        assert_eq!(SlimeVariant::Red.stats().gel, (2, 5));
        assert_eq!(SlimeVariant::Green.stats().gel, (0, 0));
        assert_eq!(SlimeVariant::Pinky.stats().gel, (0, 0));
    }

    #[test]
    fn slime_hitboxes_scale_with_the_variant() {
        // SetDefaults_ForNetId multiplies the base 24x18 by the variant scale.
        let mut p = Npc::spawn_slime(&ground(), 20).unwrap();
        p.slime = SlimeVariant::Purple;
        let (w, h) = p.size();
        assert!((w - 24.0 * 1.2).abs() < 1e-4 && (h - 18.0 * 1.2).abs() < 1e-4);
        p.slime = SlimeVariant::Pinky;
        let (w, h) = p.size();
        assert!((w - 24.0 * 0.6).abs() < 1e-4 && (h - 18.0 * 0.6).abs() < 1e-4);
    }

    #[test]
    fn wandering_eye_rides_the_eye_machine_with_four_frames() {
        // aiStyle 2 (`NPC.cs` case 133) with Main.npcFrameCount[133] = 4
        // (Main.cs:67082); stats: 300 life, 40 damage, 30x32.
        let mut e = bare(NpcKind::WanderingEye);
        assert_eq!(e.max_hp, 300);
        for _ in 0..9 {
            e.find_frame();
        }
        assert_eq!(e.frame, 1, "8 steps advance one of the 4 frames");
        // Each further batch of 8 steps advances one frame: 2, 3, then the
        // wrap 4 -> 0 (the strip never shows a fifth frame).
        for expect in [2u32, 3, 0] {
            for _ in 0..8 {
                e.find_frame();
            }
            assert_eq!(e.frame, expect);
        }
    }

    fn bare(kind: NpcKind) -> Npc {
        Npc::new(kind, 100.0, 100.0, 1, 0.0)
    }

    #[test]
    fn slime_frames_cycle_every_eight_steps_and_speed_up_while_moving() {
        // NPC.cs:70801-70826: one counter tick per step, +1 when num2 > 0.
        let mut s = bare(NpcKind::Slime);
        for _ in 0..7 {
            s.find_frame();
        }
        assert_eq!(s.frame, 0);
        s.find_frame();
        assert_eq!(s.frame, 1, "8 idle steps advance one frame");
        // Moving (num2 == 1) doubles the tick rate: 4 steps to the next frame.
        s.vx = 1.0;
        for _ in 0..3 {
            s.find_frame();
        }
        assert_eq!(s.frame, 1);
        s.find_frame();
        assert_eq!(s.frame, 0, "moving ticks twice per step: 4 steps advance");
        // The strip wraps over Main.npcFrameCount[1] = 2 frames.
        for _ in 0..16 {
            s.find_frame();
        }
        assert_eq!(s.frame, 0);
    }

    #[test]
    fn zombie_walk_band_is_zero_one_two_one_by_accumulated_speed() {
        // NPC.cs:76324-76389: the counter accumulates |velocity.X| and the
        // frame follows the bands <8 -> 0, <16 -> 1, <24 -> 2, <32 -> 1; at 32
        // the counter restarts WITHOUT touching the frame (it stays 1 until
        // the next tick re-enters the first band).
        let mut z = bare(NpcKind::Zombie);
        z.vx = 0.9; // aligned with facing 1
        z.frame_counter = 0.0;
        z.frame = 0;
        for _ in 0..9 {
            z.find_frame();
        }
        assert_eq!(z.frame, 1, "9 ticks = 8.1 accumulated: second band");
        for _ in 0..9 {
            z.find_frame();
        }
        assert_eq!(z.frame, 2, "18 ticks = 16.2: third band");
        for _ in 0..9 {
            z.find_frame();
        }
        assert_eq!(z.frame, 1, "27 ticks = 24.3: fourth band");
        for _ in 0..9 {
            z.find_frame();
        }
        assert_eq!(z.frame, 1, "36 ticks = 32.4: restart keeps the frame");
        z.find_frame();
        assert_eq!(z.frame, 0, "the next tick re-enters the first band");
    }

    #[test]
    fn zombie_holds_the_lurch_frame_when_airborne_or_backpedalling() {
        // NPC.cs:76334-76342.
        let mut z = bare(NpcKind::Zombie);
        z.vy = 0.1;
        z.find_frame();
        assert_eq!(z.frame, 2);
        z.vy = 0.0;
        z.facing = 1;
        z.vx = -0.5; // moving against the facing
        z.find_frame();
        assert_eq!(z.frame, 2);
        z.vx = 0.0;
        z.find_frame();
        assert_eq!(z.frame, 0, "standing still is the idle frame");
    }

    #[test]
    fn king_slime_snaps_to_four_five_in_the_air_and_cycles_zero_to_three_on_ground() {
        // NPC.cs:73380-73414: airborne snaps to frame 4, holds it 4 steps,
        // then frame 5; landing returns to 4 and the grounded cycle advances
        // every 8 steps with the wrap `>= 4 -> 0`, so frames 4/5 never show
        // while grounded.
        let mut k = bare(NpcKind::KingSlime);
        k.vy = -0.1; // leaving the ground
        k.find_frame();
        assert_eq!(k.frame, 4, "airborne snaps straight to frame 4");
        for _ in 0..3 {
            k.find_frame();
        }
        assert_eq!(k.frame, 5, "after 4 airborne steps the squash frame shows");
        k.vy = 0.0; // landed
        k.find_frame();
        assert_eq!(k.frame, 4, "landing returns to frame 4, counter reset");
        // The landing squash (4) is itself displayed while grounded until the
        // first advance; frames 0..=3 then cycle. Only 5 is airborne-only.
        for _ in 0..64 {
            k.find_frame();
            assert!(
                k.frame <= 4,
                "grounded frames never show 5, got {}",
                k.frame
            );
        }
        // The grounded cycle must actually move through 0..=3.
        let mut k2 = bare(NpcKind::KingSlime);
        k2.vy = 0.0;
        let mut seen = [false; 4];
        for _ in 0..32 {
            k2.find_frame();
            seen[k2.frame as usize] = true;
        }
        assert!(
            seen.iter().all(|&s| s),
            "one 32-step cycle covers frames 0..=3"
        );
    }
    #[test]
    fn the_caster_teleports_and_shoots() {
        // `AI_008` (`NPC.cs:20817-21278`): the 500 counter init, the
        // teleport attempt at 650 (the spot ≥ 5 tiles from the player —
        // the telefrag gate), then the counter resets and the 100 trigger
        // winds the 30-tick shot that fires at `ai[1] == 25`.
        let mut w = ground_air();
        let mut player = surface_player(&w);
        let pc = player.center();
        let mut tim = Npc::new(NpcKind::Tim, 40.0 * 16.0, 110.0 * 16.0, 1, 0.0);
        tim.hp = tim.max_hp;
        let start = (tim.x, tim.y);
        let mut rng = 7u64;
        let mut teleported = false;
        let mut shot = false;
        let _ = start;
        for _ in 0..400 {
            let (px, py) = (tim.x, tim.y);
            tim.update_rng(&w, &mut player, 1.0 / 60.0, &mut rng);
            if let Some((kind, sx, sy)) = tim.wants_sphere {
                assert_eq!(kind, NpcKind::ChaosBall);
                // The Chaos Ball spawn point: Tim's bottom-centre, 8 px
                // above the top (`:21242`'s NewNPC(x + w/2, y - 8) with the
                // `Bottom` semantics).
                let (tw, _) = tim.size();
                assert!((sx - (tim.x + tw * 0.5)).abs() < 0.01);
                assert!((sy - (tim.y - 8.0)).abs() < 0.01);
                shot = true;
                break;
            }
            if !teleported {
                // The teleport is a per-tick jump (the gravity fall moves
                // at most MAX_FALL = 8 px a tick, the stand decay < 1).
                let jumped = (tim.x - px).abs() + (tim.y - py).abs() > 16.0;
                if jumped {
                    teleported = true;
                    // The teleport respected the ±5-tile telefrag gate.
                    let dcx = ((tim.x + tim.size().0 * 0.5) - pc.0).abs();
                    let dcy = ((tim.y + tim.size().1 * 0.5) - pc.1).abs();
                    assert!(
                        dcx > 70.0 || dcy > 70.0,
                        "the telefrag gate held ({dcx}x{dcy})"
                    );
                    // The feet rest on a solid floor.
                    let floor = w
                        .tile(
                            (tim.x / 16.0) as i32,
                            ((tim.y + tim.size().1) / 16.0) as i32,
                        )
                        .is_some_and(|t| t.active() && tr_core::tiledata::TILE_SOLID.has(t.kind));
                    assert!(floor, "the teleport landed the feet on a solid floor");
                }
            }
        }
        assert!(teleported, "the caster never teleported");
        assert!(shot, "Tim never fired the Chaos Ball");
        // The post-teleport windup (20) never crosses the shoot tick (25)
        // — the shot came from the 100/200/300 trigger's 30-tick windup.
    }

    #[test]
    fn the_fire_imp_shoots_at_its_tick() {
        // The Fire Imp's shoot tick is `ai[1] == 10` (`:21166`) — 20 ticks
        // after the 30-tick windup, 15 after the standard casters'.
        let mut w = ground_air();
        let mut player = surface_player(&w);
        let mut imp = Npc::new(NpcKind::FireImp, 60.0 * 16.0, 110.0 * 16.0, 1, 0.0);
        imp.hp = imp.max_hp;
        let mut rng = 11u64;
        let mut shot = false;
        for _ in 0..400 {
            imp.update_rng(&w, &mut player, 1.0 / 60.0, &mut rng);
            if let Some((kind, _, _)) = imp.wants_sphere {
                assert_eq!(kind, NpcKind::BurningSphere);
                shot = true;
                break;
            }
        }
        assert!(shot, "the Fire Imp never fired the Burning Sphere");
    }

    #[test]
    fn the_sphere_launches_once_and_flies_straight() {
        // `AI_009` (`NPC.cs:21404-21429`): the velocity points at the
        // player once (the speed 6 for the Chaos Ball, 5 for the Burning
        // Sphere), then the ball flies straight through the tiles.
        let mut w = ground_air();
        let mut player = surface_player(&w);
        let (pcx, _) = player.center();
        // The ball sits up-left of the player: the launch aims right/down.
        let mut ball = Npc::new(NpcKind::ChaosBall, 80.0 * 16.0, 100.0 * 16.0, 1, 0.0);
        ball.update_rng(&w, &mut player, 1.0 / 60.0, &mut 0u64);
        let speed = (ball.vx * ball.vx + ball.vy * ball.vy).sqrt();
        assert!(
            (speed - 6.0).abs() < 0.05,
            "the launch speed is 6 ({speed})"
        );
        assert!(
            ball.vx > 0.0 && ball.vy > 0.0,
            "the ball dives at the player"
        );
        let v = (ball.vx, ball.vy);
        // The straight fly: the velocity never re-aims and the dirt passes
        // through (the floor line sits at y = 120*16).
        for _ in 0..100 {
            ball.update_rng(&w, &mut player, 1.0 / 60.0, &mut 0u64);
        }
        assert_eq!(ball.vx, v.0);
        assert_eq!(ball.vy, v.1);
        assert!(ball.x > pcx, "the ball crossed the player's column");
        assert!(
            ball.y > 120.0 * 16.0,
            "the ball flew through the floor line"
        );
        // The Burning Sphere's speed is 5 (`:21395-21399`).
        let mut sphere = Npc::new(NpcKind::BurningSphere, 80.0 * 16.0, 100.0 * 16.0, 1, 0.0);
        sphere.update_rng(&w, &mut player, 1.0 / 60.0, &mut 0u64);
        let speed = (sphere.vx * sphere.vx + sphere.vy * sphere.vy).sqrt();
        assert!((speed - 5.0).abs() < 0.05, "the burning sphere speed is 5");
    }

    #[test]
    fn the_casters_spawn_stand_and_map() {
        // `spawn_caster` plants the feet on the ground tile (`SpawnNPC`'s
        // `Bottom = (x*16+8, y*16)`), the stats ride the verified
        // `SetDefaults` rows, and the bestiary entries round-trip.
        let mut w = ground_air();
        let imp =
            Npc::spawn_caster(&w, 30, 120, NpcKind::FireImp).expect("the imp spawns on the ground");
        assert_eq!(imp.size(), (18.0, 40.0));
        assert_eq!(imp.max_hp, 70);
        assert!((imp.y - (120.0 * 16.0 - 40.0)).abs() < 0.01);
        let sorcerer =
            Npc::spawn_caster(&w, 34, 120, NpcKind::GoblinSorcerer).expect("the sorcerer spawns");
        assert_eq!(sorcerer.size(), (18.0, 38.0));
        assert_eq!(sorcerer.max_hp, 40);
        let tim = Npc::new(NpcKind::Tim, 0.0, 0.0, 1, 0.0);
        assert_eq!(tim.max_hp, 200);
        assert_eq!(tim.value(), 5000.0);
        for (kind, net) in [
            (NpcKind::GoblinSorcerer, 29),
            (NpcKind::Tim, 45),
            (NpcKind::DarkCaster, 32),
            (NpcKind::FireImp, 24),
            (NpcKind::ChaosBall, 30),
            (NpcKind::BurningSphere, 25),
            (NpcKind::WaterSphere, 33),
        ] {
            let e = crate::bestiary::entry_for_npc(kind, SlimeVariant::Blue);
            assert_eq!(e.net_id, net);
        }
    }

    /// The swim arena: the ground from y = 120 with a water pool (x
    /// 60..140, y 118..134 liquid-filled, the floor at 135) and a dry pit
    /// for the flop tests.
    fn water_pool() -> World {
        let mut w = World::new("t".into(), 200, 200);
        for x in 0..200 {
            for y in 120..200 {
                let i = w.index(x, y);
                w.tiles[i] = Tile {
                    kind: 1,
                    ..Default::default()
                };
                w.tiles[i].flags |= FLAG_ACTIVE;
            }
        }
        for x in 60..140 {
            for y in 118..135 {
                let i = w.index(x, y);
                // The water tile: no block (`active()` false — kind 0 is
                // DIRT and an active dirt tile is solid), just the liquid.
                w.tiles[i] = Tile::default();
                w.tiles[i].liquid = 255;
            }
        }
        w.compute_column_surface();
        w
    }

    #[test]
    fn the_swimmer_bobs_idly_in_water() {
        // `AI_016`'s idle machine (`NPC.cs:23848-23960`): the X drift to
        // the 1-px cap with the wall flip, the ±0.3 vertical bob through
        // `ai[0]`, the 0.95 damping.
        let w = water_pool();
        let mut player = surface_player(&w);
        player.wet = false;
        // The goldfish floats mid-pool: the idle bob never chases.
        let mut fish = Npc::new(NpcKind::Goldfish, 90.0 * 16.0, 124.0 * 16.0, 1, 0.0);
        fish.hp = fish.max_hp;
        let mut rng = 3u64;
        let mut rose = false;
        let mut sank = false;
        for _ in 0..600 {
            fish.update_rng(&w, &mut player, 1.0 / 60.0, &mut rng);
            if fish.swim_phase == -1.0 {
                rose = true;
            }
            if fish.swim_phase == 1.0 {
                sank = true;
            }
            // The idle drift's fixed point is 2.0 (x*0.95 + 0.1 = x) — the
            // 1.0 in `num258` is the damping THRESHOLD, not a cap
            // (`:23905-23914`).
            assert!(fish.vx.abs() <= 2.01, "the idle X drift ({})", fish.vx);
        }
        assert!(
            rose && sank,
            "the bob cycled both ways (rose {rose} sank {sank})"
        );
        // The fish stayed inside the pool's water column.
        assert!(
            fish.x > 55.0 * 16.0 && fish.x < 145.0 * 16.0,
            "the pool held"
        );
    }

    #[test]
    fn the_shark_chases_the_wet_player() {
        // The chase (`:23766-23845`): `ai[0] = 0`, the accel to the 5/3
        // caps while `flag25` holds (the player wet + the line clear).
        let w = water_pool();
        let mut player = surface_player(&w);
        player.wet = true;
        // The shark mid-pool, the player in the water at the pool's edge.
        player.x = 100.0 * 16.0;
        player.y = 128.0 * 16.0;
        let mut shark = Npc::new(NpcKind::Shark, 70.0 * 16.0, 124.0 * 16.0, 1, 0.0);
        shark.hp = shark.max_hp;
        let mut rng = 5u64;
        let start = shark.x;
        for _ in 0..120 {
            shark.update_rng(&w, &mut player, 1.0 / 60.0, &mut rng);
        }
        assert_eq!(shark.swim_phase, 0.0, "the chase phase set");
        assert!(shark.x > start + 10.0, "the shark closed in");
        let speed = (shark.vx * shark.vx + shark.vy * shark.vy).sqrt();
        assert!(speed > 1.0, "the chase reached speed ({speed})");
    }

    #[test]
    fn the_goldfish_never_chases() {
        // The flag25 exclusion (`:23701`): the Goldfish ignores the wet
        // player in the line of sight — the phase never hits the chase's 0
        // and the speed stays idle.
        let w = water_pool();
        let mut player = surface_player(&w);
        player.wet = true;
        player.x = 100.0 * 16.0;
        player.y = 128.0 * 16.0;
        let mut fish = Npc::new(NpcKind::Goldfish, 70.0 * 16.0, 124.0 * 16.0, 1, 0.0);
        fish.hp = fish.max_hp;
        let mut rng = 9u64;
        for _ in 0..300 {
            fish.update_rng(&w, &mut player, 1.0 / 60.0, &mut rng);
            assert!(fish.vx.abs() <= 2.01, "the goldfish stayed idle");
        }
        assert!(fish.swim_phase != 0.0, "the chase phase never set");
    }

    #[test]
    fn the_beached_fish_flops() {
        // The dry path (`:23963-23989`): the gravity pulls the fish down;
        // the Shark's stand decay holds it still on the ground.
        let mut w = water_pool();
        // A dry pocket inside the pool column: x 80..100, y 118..134 air.
        for x in 80..100 {
            for y in 118..135 {
                let i = w.index(x, y);
                w.tiles[i] = Tile::default();
            }
        }
        w.compute_column_surface();
        let mut player = surface_player(&w);
        let mut shark = Npc::new(NpcKind::Shark, 90.0 * 16.0, 120.0 * 16.0, 1, 0.0);
        shark.hp = shark.max_hp;
        let mut rng = 13u64;
        for _ in 0..200 {
            shark.update_rng(&w, &mut player, 1.0 / 60.0, &mut rng);
        }
        // The beached shark rests on the pocket floor (y = 135*16 - 24).
        let feet = shark.y + shark.size().1;
        assert!(
            (feet - 135.0 * 16.0).abs() < 1.0,
            "the shark beached on the floor ({feet})"
        );
        assert!(shark.vx.abs() < 0.2, "the stand decay held ({})", shark.vx);
        // The goldfish flops randomly out of water (the jump impulse).
        let mut fish = Npc::new(NpcKind::Goldfish, 95.0 * 16.0, 120.0 * 16.0, 1, 0.0);
        fish.hp = fish.max_hp;
        let mut jumped = false;
        for _ in 0..60 {
            fish.update_rng(&w, &mut player, 1.0 / 60.0, &mut rng);
            if fish.vy < -1.0 {
                jumped = true;
            }
        }
        assert!(jumped, "the flopping goldfish jumped");
    }

    #[test]
    fn the_swimmers_spawn_and_map() {
        // `spawn_swimmer` plants the feet on the pool floor; the stats ride
        // the verified `SetDefaults` rows; the bestiary entries round-trip.
        let w = water_pool();
        let shark =
            Npc::spawn_swimmer(&w, 100, 135, NpcKind::Shark).expect("the shark spawns in the pool");
        assert_eq!(shark.size(), (100.0, 24.0));
        assert_eq!(shark.max_hp, 300);
        assert!((shark.y - (135.0 * 16.0 - 24.0)).abs() < 0.01);
        let fish = Npc::spawn_swimmer(&w, 80, 135, NpcKind::Goldfish).expect("the goldfish spawns");
        assert_eq!(fish.size(), (20.0, 18.0));
        assert_eq!(fish.max_hp, 5);
        assert_eq!(fish.knock_back_resist(), 0.5);
        let piranha = Npc::new(NpcKind::Piranha, 0.0, 0.0, 1, 0.0);
        assert_eq!(piranha.max_hp, 30);
        assert_eq!(NpcKind::Piranha.contact_damage(), 25);
        assert_eq!(piranha.knock_back_resist(), 1.0);
        for (kind, net) in [
            (NpcKind::Goldfish, 55),
            (NpcKind::Piranha, 58),
            (NpcKind::Shark, 65),
        ] {
            let e = crate::bestiary::entry_for_npc(kind, SlimeVariant::Blue);
            assert_eq!(e.net_id, net);
        }
    }

    #[test]
    fn the_worm_chain_assembles_and_trails() {
        // `AI_006` (`NPC.cs:51305-51368`): the head rolls the budget and
        // spawns the chain front to back; the followers ride the rigid
        // trail at the width distance (`:51867-51922`).
        let mut w = ground_air();
        let mut head = Npc::spawn_worm_head(&w, 100, 125, NpcKind::GiantWormHead);
        head.hp = head.max_hp;
        let mut npcs = vec![head];
        let mut rng = 3u64;
        let mut next_id = 1u32;
        crate::world_sim::npcs::update_worms_in(&w, None, &mut npcs, &mut rng, &mut next_id);
        // The chain: the head + the bodies + the tail; the Giant Worm's
        // budget is Next(4, 7) → 4..=6 bodies → the total 6..=8 segments.
        assert!(
            npcs.len() >= 6 && npcs.len() <= 8,
            "the chain length {}",
            npcs.len()
        );
        assert_eq!(npcs[0].kind, NpcKind::GiantWormHead);
        assert_eq!(npcs[0].worm_left, npcs.len() as u32 - 2);
        assert_eq!(npcs[0].worm_id, 1);
        for (i, seg) in npcs.iter().enumerate() {
            assert_eq!(seg.worm_id, 1, "the chain id shared");
            assert_eq!(seg.seg_index, i as u32, "the indices run front to back");
        }
        assert_eq!(npcs[1].kind, NpcKind::GiantWormBody);
        assert_eq!(npcs.last().unwrap().kind, NpcKind::GiantWormTail);
        // The trail: the second segment sits one width behind the head.
        let (lw, lh) = npcs[1].size();
        let dx = npcs[1].x + lw * 0.5 - (npcs[0].x + npcs[0].size().0 * 0.5);
        let dy = npcs[1].y + lh * 0.5 - (npcs[0].y + npcs[0].size().1 * 0.5);
        let dist = (dx * dx + dy * dy).sqrt();
        assert!(
            (dist - lw).abs() < 0.5,
            "the rigid trail distance ({dist} vs {lw})"
        );
        // The second pass: the chain holds (the inited links spawn nothing).
        let len_before = npcs.len();
        crate::world_sim::npcs::update_worms_in(&w, None, &mut npcs, &mut rng, &mut next_id);
        assert_eq!(npcs.len(), len_before, "the chain is stable");
    }

    #[test]
    fn the_worm_chain_dies_with_the_head() {
        // `:51425-51433`: the leader gone — the link dies; the whole tail
        // falls in one pass.
        let w = ground_air();
        let mut head = Npc::spawn_worm_head(&w, 100, 125, NpcKind::DevourerHead);
        head.hp = head.max_hp;
        let mut npcs = vec![head];
        let mut rng = 7u64;
        let mut next_id = 1u32;
        crate::world_sim::npcs::update_worms_in(&w, None, &mut npcs, &mut rng, &mut next_id);
        assert!(npcs.len() >= 10, "the devourer assembled ({})", npcs.len());
        // The head dies: the chain falls.
        npcs[0].hp = 0;
        crate::world_sim::npcs::update_worms_in(&w, None, &mut npcs, &mut rng, &mut next_id);
        for seg in &npcs {
            assert!(seg.hp <= 0, "the segment followed the head down");
        }
    }

    #[test]
    fn the_worm_head_digs_and_stumbles() {
        // The head machine (`:51923-52051`): inside the solid the velocity
        // aims at the tile-snapped player at `num45`; in the open air the
        // gravity pulls.
        let mut w = ground_air();
        // The head deep in the dirt, the player above.
        let mut head = Npc::spawn_worm_head(&w, 40, 150, NpcKind::DevourerHead);
        head.hp = head.max_hp;
        head.worm_inited = true;
        head.worm_id = 1;
        let mut npcs = vec![head];
        let mut rng = 11u64;
        let mut next_id = 1u32;
        // The player parked above the dirt.
        let pc = (100.0 * 16.0, 116.0 * 16.0);
        crate::world_sim::npcs::update_worms_in(&w, Some(pc), &mut npcs, &mut rng, &mut next_id);
        let h0 = &npcs[0];
        // The dig: the velocity magnitude rides num45 (9 for the Devourer).
        let speed = (h0.vx * h0.vx + h0.vy * h0.vy).sqrt();
        assert!((speed - 9.0).abs() < 0.1, "the dig speed ({speed})");
        assert!(h0.vy < 0.0, "the head digs up toward the player");
        // The air stumble: the head in the open cavern falls.
        let mut w2 = ground_air();
        for x in 30..50 {
            for y in 130..160 {
                let i = w2.index(x, y);
                w2.tiles[i] = Tile::default();
            }
        }
        w2.compute_column_surface();
        let mut flyer = Npc::spawn_worm_head(&w2, 40, 140, NpcKind::GiantWormHead);
        flyer.hp = flyer.max_hp;
        flyer.worm_inited = true;
        flyer.worm_id = 2;
        let mut npcs2 = vec![flyer];
        let pc2 = (100.0 * 16.0, 116.0 * 16.0);
        crate::world_sim::npcs::update_worms_in(&w2, Some(pc2), &mut npcs2, &mut rng, &mut next_id);
        let f0 = &npcs2[0];
        assert!(f0.vy > 0.0, "the open-air head falls (+0.11 gravity)");
    }

    #[test]
    fn the_worms_spawn_and_map() {
        // The stats ride the verified `SetDefaults` rows; the bestiary
        // entries fold the body/tail credits onto the head ids.
        let w = ground_air();
        let head = Npc::spawn_worm_head(&w, 40, 130, NpcKind::DevourerHead);
        assert_eq!(head.size(), (22.0, 22.0));
        assert_eq!(head.max_hp, 100);
        assert_eq!(NpcKind::DevourerHead.contact_damage(), 31);
        assert_eq!(NpcKind::DevourerTail.contact_damage(), 13);
        assert_eq!(head.knock_back_resist(), 0.0);
        let gw = Npc::spawn_worm_head(&w, 50, 130, NpcKind::GiantWormHead);
        assert_eq!(gw.size(), (14.0, 14.0));
        assert_eq!(gw.max_hp, 30);
        let eow = Npc::new(NpcKind::EaterOfWorldsHead, 0.0, 0.0, 1, 0.0);
        assert_eq!(eow.size(), (38.0, 38.0));
        assert_eq!(eow.max_hp, 150);
        for (kind, net) in [
            (NpcKind::DevourerHead, 7),
            (NpcKind::DevourerBody, 7),
            (NpcKind::DevourerTail, 7),
            (NpcKind::GiantWormHead, 10),
            (NpcKind::GiantWormBody, 10),
            (NpcKind::GiantWormTail, 10),
            (NpcKind::EaterOfWorldsHead, 13),
            (NpcKind::EaterOfWorldsBody, 13),
            (NpcKind::EaterOfWorldsTail, 13),
        ] {
            let e = crate::bestiary::entry_for_npc(kind, SlimeVariant::Blue);
            assert_eq!(e.net_id, net);
        }
    }

    #[test]
    fn the_merchant_arrives_on_50_silver() {
        // `SpawnAllowed_Merchant` (`NPC.cs:7175-7219`): the copper total
        // >= 5000 — the silver x100, the gold x10000, the platinum
        // x1000000.
        use crate::spawner::spawn_allowed_merchant;
        assert!(!spawn_allowed_merchant(&[(71, 4999)]), "49.99 silver waits");
        assert!(spawn_allowed_merchant(&[(71, 5000)]), "50 silver arrives");
        assert!(spawn_allowed_merchant(&[(72, 50)]), "the gold counts x100");
        assert!(
            spawn_allowed_merchant(&[(73, 1)]),
            "the gold coin alone is 10000"
        );
        assert!(
            spawn_allowed_merchant(&[(74, 1)]),
            "the platinum alone clears"
        );
        assert!(!spawn_allowed_merchant(&[(72, 49)]), "49 gold = 4900 waits");
        assert!(!spawn_allowed_merchant(&[]), "the empty inventory waits");
        // The mixed stacks accumulate.
        assert!(
            spawn_allowed_merchant(&[(71, 1000), (72, 41)]),
            "10 silver + 41 gold"
        );
    }

    #[test]
    fn the_merchant_town_npc_spawns_and_maps() {
        // The stats ride `SetDefaults` case 17; the town ctor swaps the
        // kind on the Guide's body; the bestiary entry round-trips.
        let w = ground_air();
        let m = Npc::spawn_town(&w, 60, NpcKind::Merchant).expect("the merchant spawns");
        assert_eq!(m.size(), (18.0, 40.0));
        assert_eq!(m.max_hp, 250);
        assert_eq!(m.kind, NpcKind::Merchant);
        assert_eq!(m.knock_back_resist(), 0.5);
        let e = crate::bestiary::entry_for_npc(NpcKind::Merchant, SlimeVariant::Blue);
        assert_eq!(e.net_id, 17);
        assert_eq!(e.credit_id, "Merchant");
    }

    #[test]
    fn the_dryad_town_npc_spawns_and_maps() {
        // The stats ride `SetDefaults` case 20 (the town twin); the town
        // ctor swaps the kind; the bestiary entry round-trips; the arrival
        // gate rides the boss-downed flag.
        let w = ground_air();
        let d = Npc::spawn_town(&w, 60, NpcKind::Dryad).expect("the dryad spawns");
        assert_eq!(d.size(), (18.0, 40.0));
        assert_eq!(d.max_hp, 250);
        assert_eq!(d.knock_back_resist(), 0.5);
        let e = crate::bestiary::entry_for_npc(NpcKind::Dryad, SlimeVariant::Blue);
        assert_eq!(e.net_id, 20);
        assert_eq!(e.credit_id, "Dryad");
        // The arrival: the downedBoss1 gate (constants-off in the port).
        assert!(!crate::spawner::spawn_allowed_dryad(false));
        assert!(crate::spawner::spawn_allowed_dryad(true));
    }

    #[test]
    fn the_nurse_arrives_past_100_life() {
        // `SpawnAllowed_Nurse` (`NPC.cs:7158-7171`): statLifeMax / 20 > 5 —
        // the life maximum grew past the base 100.
        use crate::spawner::spawn_allowed_nurse;
        assert!(!spawn_allowed_nurse(100), "the base life waits");
        assert!(spawn_allowed_nurse(120), "the first crystal arrives");
        assert!(spawn_allowed_nurse(400), "the max life arrives");
        assert!(!spawn_allowed_nurse(99));
    }

    #[test]
    fn the_nurse_town_npc_spawns_and_maps() {
        // The stats ride `SetDefaults` case 18 (the town twin); the
        // bestiary entry round-trips; the biome prefs ride Register(18).
        let w = ground_air();
        let n = Npc::spawn_town(&w, 60, NpcKind::Nurse).expect("the nurse spawns");
        assert_eq!(n.size(), (18.0, 40.0));
        assert_eq!(n.max_hp, 250);
        assert_eq!(n.knock_back_resist(), 0.5);
        let e = crate::bestiary::entry_for_npc(NpcKind::Nurse, SlimeVariant::Blue);
        assert_eq!(e.net_id, 18);
        assert_eq!(e.credit_id, "Nurse");
        let prefs = crate::happiness::biome_preferences(NpcKind::Nurse);
        assert_eq!(prefs.len(), 2);
        assert_eq!(prefs[0].biome, crate::happiness::BiomeKind::Hallow);
        assert_eq!(prefs[1].biome, crate::happiness::BiomeKind::Snow);
    }

    #[test]
    fn the_nurse_heal_cost_math() {
        // `GetNurseHealCost`'s core: the missing life + the debuffs x100,
        // x the happiness multiplier; the rounding truncates (the C# cast).
        assert_eq!(crate::app::nurse_heal_cost_for(40, 0, 1.0), 40);
        assert_eq!(crate::app::nurse_heal_cost_for(40, 1, 1.0), 140);
        assert_eq!(crate::app::nurse_heal_cost_for(0, 2, 1.0), 200);
        // The content forest Guide multiplier (0.89) rides the service too.
        assert_eq!(crate::app::nurse_heal_cost_for(40, 1, 0.89), 124);
        // The ceiling 1.5.
        assert_eq!(crate::app::nurse_heal_cost_for(40, 1, 1.5), 210);
    }

    #[test]
    fn the_arms_dealer_arrives_on_bullets() {
        // `SpawnAllowed_ArmsDealer` (`NPC.cs:7092-7116`): any inventory
        // item with ammo == Bullet(97) or useAmmo == Bullet — the Musket
        // Ball 97's own row answers.
        use crate::spawner::spawn_allowed_arms_dealer;
        assert!(spawn_allowed_arms_dealer(&[97]), "the musket balls arrive");
        assert!(!spawn_allowed_arms_dealer(&[8, 28]), "the torches don't");
        assert!(!spawn_allowed_arms_dealer(&[]), "the empty waits");
    }

    #[test]
    fn the_arms_dealer_town_npc_spawns_and_maps() {
        // The stats ride `SetDefaults` case 19 (the town twin); the
        // bestiary entry round-trips; the prefs ride Register(19).
        let w = ground_air();
        let n = Npc::spawn_town(&w, 60, NpcKind::ArmsDealer).expect("the dealer spawns");
        assert_eq!(n.size(), (18.0, 40.0));
        assert_eq!(n.max_hp, 250);
        let e = crate::bestiary::entry_for_npc(NpcKind::ArmsDealer, SlimeVariant::Blue);
        assert_eq!(e.net_id, 19);
        assert_eq!(e.credit_id, "ArmsDealer");
        let prefs = crate::happiness::biome_preferences(NpcKind::ArmsDealer);
        assert_eq!(prefs[0].biome, crate::happiness::BiomeKind::Desert);
        assert_eq!(prefs[1].biome, crate::happiness::BiomeKind::Snow);
        // The shop index 2: the base rows + the PlayerHasItem gates.
        let stock = crate::shop::resolve_stock(2, &crate::shop::ShopFlags::default(), &[1258]);
        assert!(stock.contains(&97) && stock.contains(&95) && stock.contains(&98));
        assert!(stock.contains(&1261), "the has-item row opens");
        assert!(!stock.contains(&1836), "the other has-item row stays shut");
    }

    #[test]
    fn the_demolitionist_arrives_on_bombs() {
        // `SpawnAllowed_Demolitionist` (`NPC.cs:7067-7091`): the bomb set's
        // core rows (the Grenade 168, the Bomb 166, the Dynamite 157, the
        // sticky pair, the Bomb Fish).
        use crate::spawner::spawn_allowed_demolitionist;
        assert!(spawn_allowed_demolitionist(&[168]), "the grenade arrives");
        assert!(spawn_allowed_demolitionist(&[166]), "the bomb arrives");
        assert!(spawn_allowed_demolitionist(&[157]), "the dynamite arrives");
        assert!(
            !spawn_allowed_demolitionist(&[97, 8]),
            "the bullets don't count"
        );
        assert!(!spawn_allowed_demolitionist(&[]), "the empty waits");
    }

    #[test]
    fn the_demolitionist_town_npc_spawns_and_maps() {
        // The stats ride `SetDefaults` case 38 (the town twin); the
        // bestiary entry round-trips; the prefs ride Register(38); the
        // shop index 4 carries the explosives.
        let w = ground_air();
        let n = Npc::spawn_town(&w, 60, NpcKind::Demolitionist).expect("the demo spawns");
        assert_eq!(n.size(), (18.0, 40.0));
        assert_eq!(n.max_hp, 250);
        let e = crate::bestiary::entry_for_npc(NpcKind::Demolitionist, SlimeVariant::Blue);
        assert_eq!(e.net_id, 38);
        assert_eq!(e.credit_id, "Demolitionist");
        let prefs = crate::happiness::biome_preferences(NpcKind::Demolitionist);
        assert_eq!(prefs[0].biome, crate::happiness::BiomeKind::Underground);
        assert_eq!(prefs[1].biome, crate::happiness::BiomeKind::Ocean);
        let stock = crate::shop::resolve_stock(4, &crate::shop::ShopFlags::default(), &[]);
        assert!(stock.contains(&168) && stock.contains(&166) && stock.contains(&167));
        assert!(!stock.contains(&265), "the hardmode row leaks");
    }

    #[test]
    fn the_painter_arrives_with_two_neighbors() {
        // The town-count rule: at least 2 OTHER town NPCs present.
        use crate::spawner::spawn_allowed_painter;
        assert!(!spawn_allowed_painter(0));
        assert!(!spawn_allowed_painter(1));
        assert!(spawn_allowed_painter(2), "two neighbors open the gate");
    }

    #[test]
    fn the_painter_town_npc_spawns_and_maps() {
        // The stats ride `SetDefaults` case 227 (the town twin); the
        // bestiary entry round-trips; the prefs ride Register(227); the
        // shop index 15 carries the paints.
        let w = ground_air();
        let n = Npc::spawn_town(&w, 60, NpcKind::Painter).expect("the painter spawns");
        assert_eq!(n.size(), (18.0, 40.0));
        assert_eq!(n.max_hp, 250);
        let e = crate::bestiary::entry_for_npc(NpcKind::Painter, SlimeVariant::Blue);
        assert_eq!(e.net_id, 227);
        assert_eq!(e.credit_id, "Painter");
        let prefs = crate::happiness::biome_preferences(NpcKind::Painter);
        assert_eq!(prefs[0].biome, crate::happiness::BiomeKind::Jungle);
        assert_eq!(prefs[1].biome, crate::happiness::BiomeKind::Forest);
        let stock = crate::shop::resolve_stock(15, &crate::shop::ShopFlags::default(), &[]);
        for id in [1071, 1072, 1100, 1097, 1099, 1098, 1966] {
            assert!(stock.contains(&id), "the paint row {id} missing");
        }
    }
    #[test]
    fn eoc_hovers_dashes_and_phases() {
        // The aiStyle-4 machine: the hover (ai[0]=0) approaches 200 px above
        // the player, the dash chain (ai[1]=1→2×3) fires at 600 ticks, the
        // spin phase (ai[0]=1/2) at <50% hp, the final (ai[0]=3) zeroes the
        // defense.
        let mut w = ground();
        w.day_time = false;
        let mut player = Player::spawn(&w);
        for _ in 0..60 {
            player.update(&w, crate::player::PlayerInput::default());
        }
        let mut eye = Npc::spawn_eye_of_cthulhu(&w, player.center().0, player.center().1).unwrap();
        assert_eq!(eye.hp, 2800);
        assert_eq!(eye.defense(), 12);
        // The hover: the velocity steers toward the 200-px-above point.
        eye.ai = [0.0, 0.0, 0.0, 0.0];
        eye.update(&w, &mut player, 1.0 / 60.0);
        assert_eq!(eye.ai[0], 0.0);
        assert_eq!(eye.ai[1], 0.0);
        assert!(eye.vy != 0.0 || eye.vx != 0.0, "the hover steers");
        // 600 hover ticks flip to the dash chain.
        for _ in 0..600 {
            eye.update(&w, &mut player, 1.0 / 60.0);
        }
        assert!(
            eye.ai[1] == 1.0 || eye.ai[1] == 2.0,
            "the dash chain runs (the wind-up is instantaneous)"
        );
        // The dash locks the velocity onto the target, then the flight
        // counts to 150 and returns to the hover after 3 dashes — the
        // machine cycles (the second hover's 600-tick arm may be mid-chain
        // at the probe point).
        for _ in 0..600 {
            eye.update(&w, &mut player, 1.0 / 60.0);
        }
        assert!(
            eye.ai[3] <= 3.0 && eye.ai[0] == 0.0,
            "the machine cycles inside phase 0"
        );
        // The half-hp spin phase: the rotation ramps, then the final phase.
        eye.ai = [0.0, 0.0, 0.0, 0.0];
        eye.hp = 1000;
        for _ in 0..4 {
            eye.update(&w, &mut player, 1.0 / 60.0);
        }
        assert_eq!(eye.ai[0], 1.0, "phase 1 at <50% hp");
        // The spin counts 100 ticks per phase → phase 2 → phase 3.
        for _ in 0..300 {
            eye.update(&w, &mut player, 1.0 / 60.0);
        }
        assert_eq!(eye.ai[0], 3.0, "the final phase");
        assert_eq!(eye.defense(), 0, "the final phase zeroes the defense");
    }

    #[test]
    fn eoc_climbs_away_at_dawn() {
        // `Main.IsItDay() | dead`: `velocity.Y -= 0.04` per tick.
        let w = ground();
        let mut player = Player::spawn(&w);
        for _ in 0..60 {
            player.update(&w, crate::player::PlayerInput::default());
        }
        let mut eye = Npc::spawn_eye_of_cthulhu(&w, player.center().0, player.center().1).unwrap();
        let start_y = eye.y;
        for _ in 0..30 {
            eye.update(&w, &mut player, 1.0 / 60.0);
        }
        assert!(eye.y < start_y, "the dawn climb");
    }

    #[test]
    fn eoc_bursts_servants_when_below_the_player() {
        // The hover arm's servant burst: the eye below the player within
        // 500 px counts `ai[3]` to 110 and requests the spawn.
        let mut w = ground();
        w.day_time = false;
        let mut player = Player::spawn(&w);
        for _ in 0..60 {
            player.update(&w, crate::player::PlayerInput::default());
        }
        let (pcx, pcy) = player.center();
        let mut eye = Npc::spawn_eye_of_cthulhu(&w, pcx, pcy).unwrap();
        // Re-anchor the eye below the player: the spawn put it 600 px above;
        // the burst condition reads `self.y + h < player.y` (the eye ABOVE
        // the player) — flip the positions: put the EYE high and the PLAYER
        // below by moving the eye down and the player's Y under it.
        eye.y = pcy - 300.0 - 110.0;
        eye.ai = [0.0, 0.0, 0.0, 0.0];
        for _ in 0..130 {
            eye.update(&w, &mut player, 1.0 / 60.0);
        }
        assert!(eye.wants_servant.is_some(), "the servant burst request");
        let (sx, sy, svx, svy) = eye.wants_servant.unwrap();
        assert!(svx != 0.0 || svy != 0.0, "the launch velocity");
        // The world sim resolves the request into a real servant.
        let mut servant = Npc::spawn_servant_of_cthulhu(sx, sy, svx, svy);
        assert_eq!(servant.max_hp, 8);
        assert_eq!(servant.kind, NpcKind::ServantOfCthulhu);
        servant.update(&w, &mut player, 1.0 / 60.0);
        assert!(servant.hp > 0);
    }
    fn the_witch_doctor_arrival_gate_and_shop() {
        // The arrival: the downedQueenBee gate (constants-off in the port).
        assert!(!crate::spawner::spawn_allowed_witch_doctor(false));
        assert!(crate::spawner::spawn_allowed_witch_doctor(true));
        // The shop index 16: the Blowgun and the Flask always; the
        // conditionals constants-off.
        let stock = crate::shop::resolve_stock(16, &crate::shop::ShopFlags::default(), &[]);
        assert!(stock.contains(&1430) && stock.contains(&986));
        assert!(!stock.contains(&1158), "the night row leaks");
    }

    #[test]
    fn the_witch_doctor_town_npc_spawns_and_maps() {
        // The stats ride `SetDefaults` case 228 (the town twin); the
        // bestiary entry round-trips; the prefs ride Register(228).
        let w = ground_air();
        let n = Npc::spawn_town(&w, 60, NpcKind::WitchDoctor).expect("the witch doctor spawns");
        assert_eq!(n.size(), (18.0, 40.0));
        assert_eq!(n.max_hp, 250);
        let e = crate::bestiary::entry_for_npc(NpcKind::WitchDoctor, SlimeVariant::Blue);
        assert_eq!(e.net_id, 228);
        assert_eq!(e.credit_id, "WitchDoctor");
        let prefs = crate::happiness::biome_preferences(NpcKind::WitchDoctor);
        assert_eq!(prefs[0].biome, crate::happiness::BiomeKind::Jungle);
        assert_eq!(prefs[1].biome, crate::happiness::BiomeKind::Hallow);
    }

    #[test]
    fn the_clothier_arrival_gate_and_shop() {
        // The arrival: the downedSkeletron gate (constants-off in the
        // port).
        assert!(!crate::spawner::spawn_allowed_clothier(false));
        assert!(crate::spawner::spawn_allowed_clothier(true));
        // The shop index 5: the always-on rows.
        let stock = crate::shop::resolve_stock(5, &crate::shop::ShopFlags::default(), &[]);
        for id in [254, 981, 269, 270, 271] {
            assert!(stock.contains(&id), "the clothier row {id} missing");
        }
        assert!(!stock.contains(&242), "the day row leaks");
    }

    #[test]
    fn the_clothier_town_npc_spawns_and_maps() {
        // The stats ride `SetDefaults` case 54 (the town twin); the
        // bestiary entry round-trips; the prefs ride Register(54).
        let w = ground_air();
        let n = Npc::spawn_town(&w, 60, NpcKind::Clothier).expect("the clothier spawns");
        assert_eq!(n.size(), (18.0, 40.0));
        assert_eq!(n.max_hp, 250);
        let e = crate::bestiary::entry_for_npc(NpcKind::Clothier, SlimeVariant::Blue);
        assert_eq!(e.net_id, 54);
        assert_eq!(e.credit_id, "Clothier");
        let prefs = crate::happiness::biome_preferences(NpcKind::Clothier);
        assert_eq!(prefs[0].biome, crate::happiness::BiomeKind::Underground);
        assert_eq!(prefs[1].biome, crate::happiness::BiomeKind::Hallow);
    }

    #[test]
    fn the_eoc_charges_on_cycle() {
        // `AI_004`'s v1 cycle: the hover 600 ticks → the charge (the
        // direct velocity 6) → the 40-tick decel → the hover reset.
        let mut w = ground_air();
        w.day_time = false; // The boss fights at night (the day = the flee).
        let mut eoc = Npc::new(NpcKind::EyeOfCthulhu, 80.0 * 16.0, 20.0 * 16.0, 1, 0.0);
        eoc.hp = eoc.max_hp;
        let mut player = surface_player(&w);
        player.x = 100.0 * TILE_PX;
        player.y = 116.0 * TILE_PX;
        // The hover phase: 600 ticks move the boss toward the hover point,
        // then the dash chain arms (`ai[1]` = 1, the wind-up — the full
        // aiStyle-4 machine's state, not the v1 `cast_windup` latch).
        for _ in 0..600 {
            eoc.update_rng(&w, &mut player, 1.0 / 60.0, &mut 0u64);
        }
        assert!(
            eoc.ai[1] == 1.0 || eoc.ai[1] == 2.0,
            "the charge phase armed"
        );
        // The charge: the velocity 6 toward the player.
        eoc.update_rng(&w, &mut player, 1.0 / 60.0, &mut 0u64);
        let speed = (eoc.vx * eoc.vx + eoc.vy * eoc.vy).sqrt();
        assert!((speed - 6.0).abs() < 0.1, "the charge speed ({speed})");
        // The day flee: the boss rises.
        let mut w2 = ground_air();
        w2.day_time = true;
        let mut eoc2 = Npc::new(NpcKind::EyeOfCthulhu, 80.0 * 16.0, 20.0 * 16.0, 1, 0.0);
        eoc2.hp = eoc2.max_hp;
        eoc2.update_rng(&w2, &mut player, 1.0 / 60.0, &mut 0u64);
        assert!(eoc2.vy < 0.0, "the day flee rises");
    }

    #[test]
    fn the_eoc_spawns_and_maps() {
        // The stats ride `SetDefaults` case 4; the bestiary entry
        // round-trips.
        let w = ground_air();
        let e = crate::bestiary::entry_for_npc(NpcKind::EyeOfCthulhu, SlimeVariant::Blue);
        assert_eq!(e.net_id, 4);
        assert_eq!(e.credit_id, "EyeofCthulhu");
        let mut eoc = Npc::new(NpcKind::EyeOfCthulhu, 0.0, 0.0, 1, 0.0);
        assert_eq!(eoc.size(), (100.0, 110.0));
        assert_eq!(eoc.max_hp, 2800);
        assert_eq!(eoc.knock_back_resist(), 0.0);
        let _ = (&w, &mut eoc);
    }

    #[test]
    fn the_eow_chain_is_boss_wired() {
        // The EoW kinds exist from M4-NPC-2; the boss wire: the head's
        // death flag rides the sim's dead loop (the match arm) — this
        // test pins the downed_eow default + the head's spawn.
        let w = ground_air();
        let head = Npc::spawn_worm_head(&w, 60, 130, NpcKind::EaterOfWorldsHead);
        assert!(head.worm_inited == false, "the chain assembles in the pass");
        assert_eq!(head.max_hp, 150);
        assert_eq!(
            crate::bestiary::entry_for_npc(NpcKind::EaterOfWorldsHead, SlimeVariant::Blue).net_id,
            13
        );
    }

    #[test]
    fn the_queen_bee_cycles_and_flees() {
        // `AI_043`'s v1 cycle: the hover 480 ticks → the charge (the
        // velocity 7) → the decel; the flee phase rises away.
        let mut w = ground_air();
        w.day_time = false;
        let mut qb = Npc::new(NpcKind::QueenBee, 80.0 * 16.0, 20.0 * 16.0, 1, 0.0);
        qb.hp = qb.max_hp;
        let mut player = surface_player(&w);
        player.x = 100.0 * TILE_PX;
        player.y = 116.0 * TILE_PX;
        for _ in 0..480 {
            qb.update_rng(&w, &mut player, 1.0 / 60.0, &mut 0u64);
        }
        assert!(qb.cast_windup == 1, "the charge phase armed");
        qb.update_rng(&w, &mut player, 1.0 / 60.0, &mut 0u64);
        let speed = (qb.vx * qb.vx + qb.vy * qb.vy).sqrt();
        assert!((speed - 7.0).abs() < 0.1, "the charge speed ({speed})");
    }

    #[test]
    fn the_queen_bee_spawns_and_maps() {
        let w = ground_air();
        let e = crate::bestiary::entry_for_npc(NpcKind::QueenBee, SlimeVariant::Blue);
        assert_eq!(e.net_id, 222);
        assert_eq!(e.credit_id, "QueenBee");
        let mut qb = Npc::new(NpcKind::QueenBee, 0.0, 0.0, 1, 0.0);
        assert_eq!(qb.size(), (66.0, 66.0));
        assert_eq!(qb.max_hp, 3400);
        assert_eq!(qb.knock_back_resist(), 0.0);
        let _ = (&w, &mut qb);
    }

    #[test]
    fn the_skeletron_cycles() {
        // `AI_011`'s v1 cycle: the hover 480 ticks → the charge (the
        // velocity 8) → the decel.
        let mut w = ground_air();
        w.day_time = false;
        let mut sk = Npc::new(NpcKind::SkeletronHead, 80.0 * 16.0, 20.0 * 16.0, 1, 0.0);
        sk.hp = sk.max_hp;
        let mut player = surface_player(&w);
        player.x = 100.0 * TILE_PX;
        player.y = 116.0 * TILE_PX;
        for _ in 0..480 {
            sk.update_rng(&w, &mut player, 1.0 / 60.0, &mut 0u64);
        }
        assert!(sk.cast_windup == 1, "the charge phase armed");
        sk.update_rng(&w, &mut player, 1.0 / 60.0, &mut 0u64);
        let speed = (sk.vx * sk.vx + sk.vy * sk.vy).sqrt();
        assert!((speed - 8.0).abs() < 0.1, "the charge speed ({speed})");
    }

    #[test]
    fn the_skeletron_spawns_and_maps() {
        let w = ground_air();
        let e = crate::bestiary::entry_for_npc(NpcKind::SkeletronHead, SlimeVariant::Blue);
        assert_eq!(e.net_id, 35);
        assert_eq!(e.credit_id, "SkeletronHead");
        let mut sk = Npc::new(NpcKind::SkeletronHead, 0.0, 0.0, 1, 0.0);
        assert_eq!(sk.size(), (80.0, 102.0));
        assert_eq!(sk.max_hp, 4400);
        assert_eq!(sk.knock_back_resist(), 0.0);
        let _ = (&w, &mut sk);
    }

    #[test]
    fn the_wof_slides_toward_the_player() {
        // `AI_027`'s v1: the horizontal slide toward the player's side,
        // the vertical pinned; the off-edge despawn.
        let mut w = ground_air();
        w.day_time = false;
        let mut wof = Npc::new(NpcKind::WallOfFlesh, 60.0 * 16.0, 110.0 * 16.0, 1, 0.0);
        wof.hp = wof.max_hp;
        let mut player = surface_player(&w);
        player.x = 100.0 * TILE_PX;
        player.y = 116.0 * TILE_PX;
        for _ in 0..120 {
            wof.update_rng(&w, &mut player, 1.0 / 60.0, &mut 0u64);
        }
        assert!(wof.vx > 0.0, "the wall slides right toward the player");
        assert!(wof.vy == 0.0, "the wall stays level");
        assert!(wof.x > 60.0 * 16.0, "the wall advanced");
    }

    #[test]
    fn the_wof_spawns_and_maps() {
        let w = ground_air();
        let e = crate::bestiary::entry_for_npc(NpcKind::WallOfFlesh, SlimeVariant::Blue);
        assert_eq!(e.net_id, 113);
        assert_eq!(e.credit_id, "WallofFlesh");
        let mut wof = Npc::new(NpcKind::WallOfFlesh, 0.0, 0.0, 1, 0.0);
        assert_eq!(wof.size(), (120.0, 120.0));
        assert_eq!(wof.max_hp, 8000);
        let _ = (&w, &mut wof);
    }
    #[test]
    fn wof_slides_with_the_hp_speed_tiers_and_pins_y() {
        // AI_027: the wall slides at 1.5 (+0.25/0.4/0.5/0.6 by the hp
        // thresholds), the Y rides the draw-band midpoint, the direction
        // locks to the sign.
        let mut w = ground_air();
        w.day_time = false;
        let mut eoc_wall = Npc::new(
            NpcKind::WallOfFlesh,
            40.0 * TILE_PX,
            580.0 * TILE_PX,
            1,
            0.0,
        );
        eoc_wall.hp = eoc_wall.max_hp;
        let mut player = surface_player(&w);
        player.x = 100.0 * TILE_PX;
        player.y = 116.0 * TILE_PX;
        // The direction locks from the standstill.
        eoc_wall.update_rng(&w, &mut player, 1.0 / 60.0, &mut 0u64);
        assert!(eoc_wall.vx > 0.0, "the slide locks onto the player's side");
        let start_x = eoc_wall.x;
        for _ in 0..60 {
            eoc_wall.update_rng(&w, &mut player, 1.0 / 60.0, &mut 0u64);
        }
        assert!(eoc_wall.x > start_x, "the slide advances");
        let speed = eoc_wall.vx;
        assert!(
            (speed - 1.5).abs() < 0.01,
            "full hp slides at 1.5 ({speed})"
        );
        // The hp thresholds: 0.75 → 1.75, below-0.5 → 2.15 (the C#
        // compares `<` — the exact half keeps the 0.25 tier).
        eoc_wall.hp = eoc_wall.max_hp / 2 - 1;
        eoc_wall.vx = 0.0;
        eoc_wall.update_rng(&w, &mut player, 1.0 / 60.0, &mut 0u64);
        assert!((eoc_wall.vx - 2.15).abs() < 0.01, "half hp slides at 2.15");
        // The Y pinning: the wall rides the band's midpoint (vy forced 0;
        // the band starts unset (-1) — the first tick's scan pins it).
        assert_eq!(eoc_wall.vy, 0.0);
    }

    #[test]
    fn wof_dies_crossing_the_world_edge() {
        // `position.X < 160 || > (maxTilesX - 10) * 16` → `active = false`.
        let mut w = ground_air();
        w.day_time = false;
        let mut player = surface_player(&w);
        let mut eoc_wall = Npc::new(
            NpcKind::WallOfFlesh,
            60.0 * TILE_PX,
            580.0 * TILE_PX,
            -1,
            0.0,
        );
        eoc_wall.hp = eoc_wall.max_hp;
        eoc_wall.x = 100.0; // inside the 160 kill line
        eoc_wall.update_rng(&w, &mut player, 1.0 / 60.0, &mut 0u64);
        assert!(eoc_wall.hp <= 0, "the edge crossing kills the wall");
    }

    #[test]
    fn wof_abandons_when_the_player_stays_dead() {
        // The `localAI[1]` timer (folded into `ai[0]`): 180 ticks dead →
        // the wall kills itself; the recovery drains at 1/30.
        let mut w = ground_air();
        w.day_time = false;
        let mut player = surface_player(&w);
        player.x = 100.0 * TILE_PX;
        player.y = 116.0 * TILE_PX;
        let mut eoc_wall = Npc::new(
            NpcKind::WallOfFlesh,
            40.0 * TILE_PX,
            580.0 * TILE_PX,
            1,
            0.0,
        );
        eoc_wall.hp = eoc_wall.max_hp;
        player.hp = 0;
        for _ in 0..200 {
            eoc_wall.update_rng(&w, &mut player, 1.0 / 60.0, &mut 0u64);
            if eoc_wall.hp <= 0 {
                break;
            }
        }
        assert!(eoc_wall.hp <= 0, "the abandoned wall dies");
    }

    #[test]
    fn the_hungry_anchors_to_the_band_and_tiers_by_the_wall_hp() {
        // aiStyle 29: the anchor = the wall's x + the band row at `ai[0]`;
        // the leash 300/500/700 and the defense/damage tiers by the wall's
        // hp; the wall's death deactivates the minion.
        let mut w = ground_air();
        w.day_time = false;
        w.wof_present = true;
        w.wof_hp_ratio = 1.0;
        w.wof_x = 40.0 * TILE_PX;
        w.wof_draw_top = 580.0 * TILE_PX;
        w.wof_draw_bottom = 600.0 * TILE_PX;
        let mut player = surface_player(&w);
        player.x = 100.0 * TILE_PX;
        player.y = 116.0 * TILE_PX;
        let mut hungry = Npc::new(NpcKind::Hungry, 40.0 * TILE_PX, 590.0 * TILE_PX, 1, 0.0);
        hungry.hp = hungry.max_hp;
        hungry.ai[0] = 0.45;
        hungry.update_rng(&w, &mut player, 1.0 / 60.0, &mut 0u64);
        assert!(hungry.hp > 0, "the wall is alive");
        assert_eq!(hungry.defense(), 10, "full-hp wall keeps the base defense");
        // The wall's hp drops: the defense tier 20 (75%) then 30 (50%).
        w.wof_hp_ratio = 0.6;
        hungry.update_rng(&w, &mut player, 1.0 / 60.0, &mut 0u64);
        assert_eq!(hungry.defense(), 20);
        w.wof_hp_ratio = 0.4;
        hungry.update_rng(&w, &mut player, 1.0 / 60.0, &mut 0u64);
        assert_eq!(hungry.defense(), 30);
        assert_eq!(hungry.damage_override, Some(60));
        // The wall dies → the minion deactivates.
        w.wof_present = false;
        hungry.update_rng(&w, &mut player, 1.0 / 60.0, &mut 0u64);
        assert!(hungry.hp <= 0, "the hungry dies with the wall");
    }

    #[test]
    fn the_leech_head_grows_the_chain() {
        // The leech head rides the worm pass: the init rolls the 3..5
        // budget (`AI_006`'s `Next(3, 6)`), the links chain via
        // `worm_next`.
        let w = ground_air();
        let mut leech = Npc::new(NpcKind::LeechHead, 40.0 * TILE_PX, 580.0 * TILE_PX, 1, 0.0);
        leech.hp = leech.max_hp;
        assert!(leech.kind.is_worm() && leech.kind.is_worm_head());
        assert_eq!(leech.kind.worm_next(), NpcKind::LeechBody);
        assert_eq!(NpcKind::LeechBody.worm_next(), NpcKind::LeechBody);
        assert_eq!(leech.max_hp, 60);
        assert_eq!(leech.defense(), 2);
        let mut body = Npc::new(NpcKind::LeechBody, 40.0 * TILE_PX, 580.0 * TILE_PX, 1, 0.0);
        assert_eq!(body.max_hp, 60);
        assert_eq!(body.defense(), 6);
        assert_eq!(body.kind.contact_damage(), 22);
    }

    #[test]
    fn the_destroyer_chain_assembles_same_kind() {
        // `AI_037`: the all-same-type chain — the head spawns the
        // Destroyer segments through the worm pass (the 80 budget).
        let w = ground_air();
        let mut head = Npc::spawn_worm_head(&w, 100, 130, NpcKind::TheDestroyer);
        head.hp = head.max_hp;
        head.worm_left = 80;
        let mut npcs = vec![head];
        let mut rng = 3u64;
        let mut next_id = 1u32;
        crate::world_sim::npcs::update_worms_in(&w, None, &mut npcs, &mut rng, &mut next_id);
        // The chain: the head + 80 segments = 81 (the same-kind stop).
        assert_eq!(npcs.len(), 81, "the destroyer chain length {}", npcs.len());
        for seg in &npcs {
            assert_eq!(seg.kind, NpcKind::TheDestroyer);
        }
        // The trail: the second segment trails the head.
        let (lw, lh) = npcs[1].size();
        let dx = npcs[1].x + lw * 0.5 - (npcs[0].x + lw * 0.5);
        let dy = npcs[1].y + lh * 0.5 - (npcs[0].y + npcs[0].size().1 * 0.5);
        let dist = (dx * dx + dy * dy).sqrt();
        assert!((dist - lw).abs() < 0.5, "the rigid trail ({dist})");
    }

    #[test]
    fn the_destroyer_spawns_and_maps() {
        let w = ground_air();
        let e = crate::bestiary::entry_for_npc(NpcKind::TheDestroyer, SlimeVariant::Blue);
        assert_eq!(e.net_id, 134);
        assert_eq!(e.credit_id, "TheDestroyer");
        let mut d = Npc::new(NpcKind::TheDestroyer, 0.0, 0.0, 1, 0.0);
        assert_eq!(d.size(), (48.0, 48.0));
        assert_eq!(d.max_hp, 8000);
        assert_eq!(d.knock_back_resist(), 0.0);
        assert_eq!(NpcKind::TheDestroyer.contact_damage(), 70);
        let _ = (&w, &mut d);
    }

    #[test]
    fn the_prime_cycles() {
        // `AI_032`'s v1 cycle: the hover 480 ticks → the charge (the
        // velocity 9) → the decel.
        let mut w = ground_air();
        w.day_time = false;
        let mut pr = Npc::new(NpcKind::SkeletronPrime, 80.0 * 16.0, 20.0 * 16.0, 1, 0.0);
        pr.hp = pr.max_hp;
        let mut player = surface_player(&w);
        player.x = 100.0 * TILE_PX;
        player.y = 116.0 * TILE_PX;
        for _ in 0..480 {
            pr.update_rng(&w, &mut player, 1.0 / 60.0, &mut 0u64);
        }
        assert!(pr.cast_windup == 1, "the charge phase armed");
        pr.update_rng(&w, &mut player, 1.0 / 60.0, &mut 0u64);
        let speed = (pr.vx * pr.vx + pr.vy * pr.vy).sqrt();
        assert!((speed - 9.0).abs() < 0.1, "the charge speed ({speed})");
    }

    #[test]
    fn the_prime_spawns_and_maps() {
        let w = ground_air();
        let e = crate::bestiary::entry_for_npc(NpcKind::SkeletronPrime, SlimeVariant::Blue);
        assert_eq!(e.net_id, 127);
        assert_eq!(e.credit_id, "SkeletronPrime");
        let mut pr = Npc::new(NpcKind::SkeletronPrime, 0.0, 0.0, 1, 0.0);
        assert_eq!(pr.size(), (80.0, 102.0));
        assert_eq!(pr.max_hp, 28000);
        assert_eq!(pr.knock_back_resist(), 0.0);
        assert_eq!(NpcKind::SkeletronPrime.contact_damage(), 47);
        let _ = (&w, &mut pr);
    }

    #[test]
    fn the_twin_cycles_and_maps() {
        // `AI_030/031`'s v1 cycle: the hover 480 ticks → the charge (the
        // velocity 8.5) → the decel.
        let mut w = ground_air();
        w.day_time = false;
        let mut ret = Npc::new(NpcKind::Retinazer, 80.0 * 16.0, 20.0 * 16.0, 1, 0.0);
        ret.hp = ret.max_hp;
        let mut player = surface_player(&w);
        player.x = 100.0 * TILE_PX;
        player.y = 116.0 * TILE_PX;
        for _ in 0..480 {
            ret.update_rng(&w, &mut player, 1.0 / 60.0, &mut 0u64);
        }
        assert!(ret.cast_windup == 1, "the charge phase armed");
        ret.update_rng(&w, &mut player, 1.0 / 60.0, &mut 0u64);
        let speed = (ret.vx * ret.vx + ret.vy * ret.vy).sqrt();
        assert!((speed - 8.5).abs() < 0.1, "the charge speed ({speed})");
        // The stats round-trip.
        assert_eq!(ret.size(), (100.0, 110.0));
        assert_eq!(ret.max_hp, 20000);
        let spaz = Npc::new(NpcKind::Spazmatism, 0.0, 0.0, 1, 0.0);
        assert_eq!(spaz.max_hp, 23000);
        assert_eq!(NpcKind::Spazmatism.contact_damage(), 50);
        let e = crate::bestiary::entry_for_npc(NpcKind::Retinazer, SlimeVariant::Blue);
        assert_eq!(e.net_id, 125);
        let e = crate::bestiary::entry_for_npc(NpcKind::Spazmatism, SlimeVariant::Blue);
        assert_eq!(e.net_id, 126);
    }

    #[test]
    fn the_plantera_cycles_and_maps() {
        // `AI_051`'s v1 cycle: the hover 480 ticks → the charge (the
        // velocity 7.5) → the decel.
        let mut w = ground_air();
        w.day_time = false;
        let mut pl = Npc::new(NpcKind::Plantera, 80.0 * 16.0, 20.0 * 16.0, 1, 0.0);
        pl.hp = pl.max_hp;
        let mut player = surface_player(&w);
        player.x = 100.0 * TILE_PX;
        player.y = 116.0 * TILE_PX;
        for _ in 0..480 {
            pl.update_rng(&w, &mut player, 1.0 / 60.0, &mut 0u64);
        }
        assert!(pl.cast_windup == 1, "the charge phase armed");
        pl.update_rng(&w, &mut player, 1.0 / 60.0, &mut 0u64);
        let speed = (pl.vx * pl.vx + pl.vy * pl.vy).sqrt();
        assert!((speed - 7.5).abs() < 0.1, "the charge speed ({speed})");
        // The stats round-trip.
        assert_eq!(pl.size(), (86.0, 86.0));
        assert_eq!(pl.max_hp, 30000);
        assert_eq!(NpcKind::Plantera.contact_damage(), 50);
        let e = crate::bestiary::entry_for_npc(NpcKind::Plantera, SlimeVariant::Blue);
        assert_eq!(e.net_id, 262);
        assert_eq!(e.credit_id, "Plantera");
    }

    #[test]
    fn the_brain_teleport_dashes() {
        // `AI_054`'s v1 cycle: the telegraph 60 ticks -> the dash (the
        // velocity 9) -> the decay -> the telegraph reset.
        let mut w = ground_air();
        w.day_time = false;
        let mut br = Npc::new(NpcKind::BrainOfCthulhu, 80.0 * 16.0, 20.0 * 16.0, 1, 0.0);
        br.hp = br.max_hp;
        let mut player = surface_player(&w);
        player.x = 100.0 * TILE_PX;
        player.y = 116.0 * TILE_PX;
        for _ in 0..59 {
            br.update_rng(&w, &mut player, 1.0 / 60.0, &mut 0u64);
        }
        assert!(br.cast_windup == 2, "the dash armed");
        br.update_rng(&w, &mut player, 1.0 / 60.0, &mut 0u64);
        br.update_rng(&w, &mut player, 1.0 / 60.0, &mut 0u64);
        let speed = (br.vx * br.vx + br.vy * br.vy).sqrt();
        assert!((speed - 9.0).abs() < 0.2, "the dash speed ({speed})");
        // The stats round-trip.
        assert_eq!(br.size(), (160.0, 110.0));
        assert_eq!(br.max_hp, 1250);
        let cr = Npc::new(NpcKind::Creeper, 0.0, 0.0, 1, 0.0);
        assert_eq!(cr.size(), (30.0, 30.0));
        assert_eq!(cr.max_hp, 100);
        let e = crate::bestiary::entry_for_npc(NpcKind::BrainOfCthulhu, SlimeVariant::Blue);
        assert_eq!(e.net_id, 266);
        let e = crate::bestiary::entry_for_npc(NpcKind::Creeper, SlimeVariant::Blue);
        assert_eq!(e.net_id, 267);
    }

    #[test]
    fn the_goblin_kinds_spawn_and_maps() {
        // `SetDefaults` cases 26/27/28/111: the stats + the kind↔u16
        // round-trip + the bestiary rows.
        let w = ground_air();
        let peon = Npc::spawn_cave_walker(&w, 20, NpcKind::GoblinPeon).expect("the peon spawns");
        assert_eq!(peon.max_hp, 60);
        assert_eq!(NpcKind::GoblinPeon.contact_damage(), 12);
        let warrior =
            Npc::spawn_cave_walker(&w, 24, NpcKind::GoblinWarrior).expect("the warrior spawns");
        assert_eq!(warrior.max_hp, 110);
        assert_eq!(NpcKind::GoblinWarrior.contact_damage(), 25);
        for (kind, net) in [
            (NpcKind::GoblinPeon, 26),
            (NpcKind::GoblinThief, 27),
            (NpcKind::GoblinWarrior, 28),
            (NpcKind::GoblinArcher, 111),
        ] {
            let e = crate::bestiary::entry_for_npc(kind, SlimeVariant::Blue);
            assert_eq!(e.net_id, net);
        }
    }
    #[test]
    fn the_tinkerer_kinds_stats_roundtrip_and_shop() {
        // `SetDefaults` 105 (`NPC.cs:10245-10257`) and 107 (`:10271-10283`):
        // the 250/15/0.5 pair, the 18x34 vs 18x40 bodies, the kind<->u16
        // round-trip, the bestiary row and the shop map.
        assert_eq!(NpcKind::BoundGoblin.max_hp(), 250);
        assert_eq!(NpcKind::GoblinTinkerer.max_hp(), 250);
        let t = Npc::new(NpcKind::GoblinTinkerer, 0.0, 0.0, 1, 0.0);
        assert_eq!(t.defense(), 15);
        assert_eq!(t.knock_back_resist(), 0.5);
        assert_eq!(NpcKind::BoundGoblin.contact_damage(), 0);
        assert_eq!(NpcKind::BoundGoblin.size(), (18.0, 34.0));
        assert_eq!(NpcKind::GoblinTinkerer.size(), (18.0, 40.0));
        for (kind, v) in [(NpcKind::BoundGoblin, 61), (NpcKind::GoblinTinkerer, 62)] {
            assert_eq!(kind.to_u16(), v);
            assert_eq!(NpcKind::from_u16(v), Some(kind));
        }
        let e = crate::bestiary::entry_for_npc(NpcKind::GoblinTinkerer, SlimeVariant::Blue);
        assert_eq!(e.net_id, 107);
        // The shop map (`NPCInteractions.cs:488-498`): the Tinkerer's
        // shop 6; the Guide has none.
        assert_eq!(
            crate::world_sim::npcs::shop_index_for_kind(NpcKind::GoblinTinkerer),
            Some(6)
        );
        assert_eq!(
            crate::world_sim::npcs::shop_index_for_kind(NpcKind::Merchant),
            Some(1)
        );
        assert_eq!(
            crate::world_sim::npcs::shop_index_for_kind(NpcKind::Guide),
            None
        );
        // The stock (`Chest.cs:1989-2003`): the 7 rows in order, no gating.
        let ids = crate::shop::resolve_stock(6, &crate::shop::ShopFlags::default(), &[]);
        assert_eq!(ids, vec![128, 486, 398, 84, 407, 161, 5324]);
    }

    #[test]
    fn the_bound_goblin_rescues_into_tinkerer() {
        // `AI_000_TransformBoundNPC` (`NPC.cs:44884-44896`): the talk
        // swaps 105 -> 107, the freed stats restore, `savedGoblin` raises.
        let mut w = ground_air();
        let mut n = Npc::new(NpcKind::BoundGoblin, 20.0 * 16.0, 100.0 * 16.0, 1, 0.0);
        n.max_hp = 250;
        n.hp = 250;
        assert_eq!(w.saved_goblin, false);
        crate::world_sim::npcs::rescue_bound_goblin(&mut n, &mut w);
        assert_eq!(n.kind, NpcKind::GoblinTinkerer);
        assert_eq!(n.max_hp, 250);
        assert_eq!(n.hp, 250);
        assert_eq!(w.saved_goblin, true);
    }
}
