//! First enemy pass: a hopping blue slime, a flying demon eye and a walking zombie,
//! modelled on the simplest `NPC.cs` behaviours.

use tr_core::{TILE_PX, World};

use crate::npc_buffs::NpcBuffs;
use crate::npc_ai::HoverParams;
use crate::player::{Player, collides_rect};

pub(crate) const GRAVITY: f32 = 0.4;
pub(crate) const MAX_FALL: f32 = 8.0;

/// `Collision.WetCollision` (simplified): any water tile overlapping the body.
fn is_wet(world: &World, x: f32, y: f32, w: f32, h: f32) -> bool {
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
            _ => return None,
        })
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
            NpcKind::CaveBat => (22.0, 18.0),
            NpcKind::KingSlime => (98.0, 92.0),
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
            NpcKind::KingSlime => 2000,
        }
    }

    fn contact_damage(self) -> i32 {
        match self {
            NpcKind::Slime => 7,         // NPC.cs:8571
            NpcKind::DemonEye => 18,     // case 2
            NpcKind::WanderingEye => 40, // case 133
            NpcKind::Zombie => 14,
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
            NpcKind::Guide => 0,
            NpcKind::KingSlime => 40,
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
    /// The caster's sphere spawn request — the vanilla `NPC.NewNPC` at the
    /// shoot tick (`AI_008`), resolved by the world sim after the update
    /// pass (the borrow-checker-safe `wants_minion` shape). The payload is
    /// the (kind, bottom-centre x, bottom-centre y) spawn point.
    pub wants_sphere: Option<(NpcKind, f32, f32)>,
    /// `ai[3]`: x position while descending through water, for the bounce flip.
    wet_x: f32,
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
        Some(Self::new(kind, x, y, if (x_tile % 2) as i32 == 0 { 1 } else { -1 }, 0.0))
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
        Some(Self::new(kind, x, y, if (x_tile % 2) as i32 == 0 { 1 } else { -1 }, 0.0))
    }

    /// Spawns the Guide homeless at the surface tile (`NPC.NewNPC`'s
    /// world-gen arrival).
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
            wet_x: 0.0,
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
            // The town NPCs barely knock (`NPC.SetDefaults` 22: 0.5).
            NpcKind::Guide => 0.5,
            NpcKind::KingSlime => 0.0,
            NpcKind::Slime => unreachable!(),
        }
    }

    /// `NPC.defense` — `Main.CalculateDamageNPCsTake`'s input
    /// (blue `NPC.cs:8573` 2, demon eye 2, Wandering Eye 20, zombie 6,
    /// King Slime 10; the slime variants ride their `SetDefaultsFromNetId`
    /// lines).
    pub fn defense(&self) -> i32 {
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
            NpcKind::Guide => 15,
            NpcKind::KingSlime => 10,
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
            NpcKind::Guide => 0.0,
            NpcKind::KingSlime => 10_000.0,
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
            NpcKind::Guide => self.update_guide(world, dt),
            NpcKind::Skeleton => self.update_skeleton(world, player, dt),
            NpcKind::UndeadMiner => self.update_skeleton(world, player, dt),
            NpcKind::CaveBat
            | NpcKind::JungleBat
            | NpcKind::Hellbat
            | NpcKind::VoodooDemon => self.update_cave_bat(world, player, dt),
            NpcKind::EaterOfSouls => {
                self.update_hover(world, player, HoverParams::EATER_OF_SOULS)
            }
            NpcKind::Hornet => self.update_hover(world, player, HoverParams::HORNET),
            NpcKind::MeteorHead => {
                self.update_hover(world, player, HoverParams::METEOR_HEAD)
            }
            NpcKind::KingSlime => self.update_king_slime(world, player, dt),
            NpcKind::GoblinSorcerer | NpcKind::Tim | NpcKind::DarkCaster
            | NpcKind::FireImp => self.update_caster(world, player, rng),
            NpcKind::ChaosBall | NpcKind::BurningSphere | NpcKind::WaterSphere => {
                self.update_sphere(world, player)
            }
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
            NpcKind::CaveBat
            | NpcKind::JungleBat
            | NpcKind::Hellbat
            | NpcKind::VoodooDemon => {
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
                NpcKind::FireImp => {
                    (self.x + w * 0.5 + self.facing as f32 * 8.0, self.y + 20.0)
                }
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

    /// `AI_009` through the framework's `sphere_step` (`NPC.cs`'s aiStyle 9
    /// core, `:21404-21429`): the one-shot launch, then the straight fly
    /// through the tiles (`noGravity`/`noTileCollide`).
    fn update_sphere(&mut self, world: &World, player: &Player) {
        let (pcx, pcy) = player.center();
        crate::npc_ai::sphere_step(self, pcx, pcy, world);
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
            // The town NPCs never hurt the player (`aiStyle 7`'s no-contact).
            if self.kind != NpcKind::Guide {
                let dmg = if self.kind == NpcKind::Slime {
                    self.slime.stats().damage
                } else {
                    self.kind.contact_damage()
                };
                player.hurt(dmg, cx);
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
        let meteor = Npc::spawn_hover(&w, 28, 30, NpcKind::MeteorHead)
            .expect("the meteor head spawns");
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
        let jb = Npc::spawn_cave_flyer(&w, 22, 30, NpcKind::JungleBat)
            .expect("the jungle bat spawns");
        assert_eq!(jb.max_hp, 34);
        let hb = Npc::spawn_cave_flyer(&w, 24, 30, NpcKind::Hellbat)
            .expect("the hellbat spawns");
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
        assert!((speed - 6.0).abs() < 0.05, "the launch speed is 6 ({speed})");
        assert!(ball.vx > 0.0 && ball.vy > 0.0, "the ball dives at the player");
        let v = (ball.vx, ball.vy);
        // The straight fly: the velocity never re-aims and the dirt passes
        // through (the floor line sits at y = 120*16).
        for _ in 0..100 {
            ball.update_rng(&w, &mut player, 1.0 / 60.0, &mut 0u64);
        }
        assert_eq!(ball.vx, v.0);
        assert_eq!(ball.vy, v.1);
        assert!(ball.x > pcx, "the ball crossed the player's column");
        assert!(ball.y > 120.0 * 16.0, "the ball flew through the floor line");
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
        let imp = Npc::spawn_caster(&w, 30, 120, NpcKind::FireImp)
            .expect("the imp spawns on the ground");
        assert_eq!(imp.size(), (18.0, 40.0));
        assert_eq!(imp.max_hp, 70);
        assert!((imp.y - (120.0 * 16.0 - 40.0)).abs() < 0.01);
        let sorcerer = Npc::spawn_caster(&w, 34, 120, NpcKind::GoblinSorcerer)
            .expect("the sorcerer spawns");
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

}
