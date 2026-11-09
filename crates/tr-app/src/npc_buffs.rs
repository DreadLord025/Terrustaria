//! NPC buff storage and the DoT tick — the `NPC.buffType[20]` / `buffTime[20]`
//! engine (`NPC.AddBuff`, `NPC.DelBuff`), the per-tick buff block of
//! `UpdateNPC` (set flags → clear expired → apply DOTs) and the roster's
//! debuff immunities.
//!
//! Source map (all line numbers into `decompiled/Terraria/`):
//! - `NPC.maxBuffs = 20`: `NPC.cs:95189`; `buffType`/`buffTime`/`buffImmune`:
//!   `NPC.cs:6193-6197`; `lifeRegen`/`lifeRegenCount`: `NPC.cs:6247-6249`
//! - `NPC.AddBuff`: `NPC.cs:89926-89983`; `NPC.DelBuff`: `NPC.cs:90002-90023`;
//!   `NPC.FindBuffIndex`: `NPC.cs:89910-89924`
//! - the `UpdateNPC` buff block order: `NPC.cs:90373-90377` (flags reset → set
//!   flags → soul drain → clear expired → apply DOTs)
//! - `UpdateNPC_BuffSetFlags` (timer decrement + id→flag map): `NPC.cs:91965-92149`
//! - `UpdateNPC_BuffFlagsReset`: `NPC.cs:92151-92192`
//! - `UpdateNPC_BuffClearExpiredBuffs`: `NPC.cs:91890-91909`
//! - `UpdateNPC_BuffApplyDOTs` + `DOTTally`: `NPC.cs:91594-91774`, `NPC.cs:59-141`
//! - `NPC.GetHurtByDebuff`: `NPC.cs:91864-91888`
//! - debuff immunities: `NPC.cs:17823-17833`, `DataStructures/NPCDebuffImmunityData.cs`,
//!   `NPCID.Sets.DebuffImmunitySets` (`NPCID.cs:6075+`)
//!
//! The Player-side chains live in `buffs.rs` (`Player.AddBuff` family); the C#
//! keeps the two chains separate (no fed/pet/melee stages, no
//! `DetermineBuffTimeToAdd` here) and so does the port.

use crate::buffs::{self, BUFF_ID_COUNT};
use crate::npc::NpcKind;

/// `NPC.maxBuffs` (`NPC.cs:95189`).
pub const NPC_MAX_BUFFS: usize = 20;

/// The per-tick buff flags of `UpdateNPC_BuffSetFlags` (`NPC.cs:91965-92149`),
/// restricted to the ids the ported systems consume. The C# keeps them as NPC
/// fields wiped by `UpdateNPC_BuffFlagsReset` (`NPC.cs:92151-92192`) at the
/// start of every tick; deriving them fresh each tick is equivalent.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
struct NpcBuffFlags {
    /// buff 20 `Poisoned` (`NPC.cs:91981-91984`).
    poisoned: bool,
    /// buff 30 `Bleeding` (`NPC.cs:91989-91992`).
    bleeding: bool,
    /// buff 375 `Hemorrhage` (`NPC.cs:91993-91996`).
    hemorrhage: bool,
    /// buff 70 `Venom` (`NPC.cs:91997-92000`).
    venom: bool,
    /// buff 395 `PotentAcid` (`NPC.cs:92005-92008`).
    potent_acid: bool,
    /// buff 397 `ChlorophyteSpore` (`NPC.cs:92001-92004`).
    chlorophyte_spore: bool,
    /// buff 24 `OnFire` (`NPC.cs:92009-92016`).
    on_fire: bool,
    /// buff 323 `OnFire3` (`NPC.cs:92125-92128`).
    on_fire3: bool,
    /// buff 44 `Frostburn` (`NPC.cs:92037-92044`).
    on_frost_burn: bool,
    /// buff 324 `Frostburn2` (`NPC.cs:92129-92132`).
    on_frost_burn2: bool,
    /// buff 39 `CursedInferno` (`NPC.cs:92033-92036`).
    on_fire2: bool,
    /// buff 153 `ShadowFlame` (`NPC.cs:92069-92072`).
    shadow_flame: bool,
    /// buff 204 `Oiled` (`NPC.cs:92105-92108`).
    oiled: bool,
    /// buff 398 `AcceleratePoisons` (`NPC.cs:91973-91976`).
    accelerate_poisons: bool,
    /// buff 137 `drippingSlime` (`NPC.cs:92049-92052`).
    dripping_slime: bool,
}

/// `DOTTally` (`NPC.cs:59-141`): one tick's sum of per-debuff `regenLoss`.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
struct DotTally {
    life_regen_loss: i32,
    life_regen_loss_that_can_be_boosted: i32,
    expected_damage_per_second: i32,
    block_natural_regeneration: bool,
}

impl DotTally {
    /// `DOTTally.Add` (`NPC.cs:80-88`).
    fn add(&mut self, regen_loss: i32, expected_amount_per_second: i32) {
        self.block_natural_regeneration = true;
        self.life_regen_loss += regen_loss;
        if self.expected_damage_per_second < expected_amount_per_second {
            self.expected_damage_per_second = expected_amount_per_second;
        }
    }

    /// `DOTTally.SupportAcceleration` (`NPC.cs:134-140`).
    fn support_acceleration(&mut self, regen_loss: i32, accelerate_poisons: bool) {
        if accelerate_poisons {
            self.life_regen_loss_that_can_be_boosted += regen_loss;
        }
    }

    /// `DOTTally.AddPoison` (`NPC.cs:90-94`).
    fn add_poison(&mut self, regen_loss: i32, expected: i32, accelerate_poisons: bool) {
        self.support_acceleration(regen_loss, accelerate_poisons);
        self.add(regen_loss, expected);
    }

    /// `DOTTally.AddBloodLoss` (`NPC.cs:96-100`).
    fn add_blood_loss(&mut self, regen_loss: i32, expected: i32, accelerate_poisons: bool) {
        self.support_acceleration(regen_loss, accelerate_poisons);
        self.add(regen_loss, expected);
    }

    /// `DOTTally.AddFlamable` (`NPC.cs:102-114`): `drippingSlime` (buff 137)
    /// doubles the fire family. The `+16` for the vampire-seed slime
    /// (`type == 1 && ai[1] == 9`, `NPC.cs:108-111`) has no roster kind — that
    /// ai[1] is only set on the vampireSeed spawn path (`NPC.cs:60569-60572`),
    /// while every ported slime variant is a plain net id (`NPC.cs:7678-7746`).
    fn add_flamable(&mut self, mut regen_loss: i32, expected: i32, flags: &NpcBuffFlags) {
        if flags.dripping_slime {
            regen_loss *= 2;
        }
        self.support_acceleration(regen_loss, flags.accelerate_poisons);
        self.add(regen_loss, expected);
    }
}

/// The NPC's 20 buff slots plus the `buffImmune` table and the drain
/// accumulator.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NpcBuffs {
    /// `NPC.buffType[20]` (`NPC.cs:6193`); 0 = empty slot.
    pub buff_type: [i32; NPC_MAX_BUFFS],
    /// `NPC.buffTime[20]`, in ticks (`NPC.cs:6195`).
    pub buff_time: [i32; NPC_MAX_BUFFS],
    /// `NPC.buffImmune[BuffID.Count]` (`NPC.cs:6197`).
    pub buff_immune: [bool; BUFF_ID_COUNT],
    /// `NPC.lifeRegenCount` (`NPC.cs:6249`): the ±120 damage/heal chunk
    /// accumulator carried between ticks.
    pub life_regen_count: i32,
}

impl Default for NpcBuffs {
    fn default() -> Self {
        Self {
            buff_type: [0; NPC_MAX_BUFFS],
            buff_time: [0; NPC_MAX_BUFFS],
            buff_immune: [false; BUFF_ID_COUNT],
            life_regen_count: 0,
        }
    }
}

/// The roster's slice of `NPCID.Sets.DebuffImmunitySets` (`NPCID.cs:6075+`):
/// the `SpecificallyImmuneTo` data of each ported kind. DemonEye (2,
/// `NPCID.cs:6085`), Zombie (3, `NPCID.cs:6086`) and WanderingEye (133,
/// `NPCID.cs:6905`) have null entries — no immunity, like the C# else-branch
/// (`NPC.cs:17826-17832`). The slime variants all share type 1.
/// `ImmuneToWhips` / `ImmuneToAllBuffsThatAreNotWhips` are false for every
/// roster entry, so `ApplyToNPC`'s mask loops never fire
/// (`NPCDebuffImmunityData.cs:10-19`).
fn debuff_immune_ids(kind: NpcKind) -> &'static [i32] {
    match kind {
        // `{ 1, SpecificallyImmuneTo: [20] }` (`NPCID.cs:6079-6084`).
        NpcKind::Slime => &[20],
        // King Slime: `{ 50, [20, 31] }` (`NPCID.cs:6386-6392`).
        // The Guide: no immunities (the town NPCs have no data rows).
        NpcKind::Guide => &[],
        NpcKind::Merchant => &[],
        NpcKind::Dryad => &[],
        NpcKind::Nurse => &[],
        NpcKind::ArmsDealer => &[],
        NpcKind::Demolitionist => &[],
        NpcKind::Painter => &[],
        NpcKind::WitchDoctor => &[],
        NpcKind::Clothier => &[],
        // `NPCID.cs:8247-8253`: the Travelling Merchant's `SpecificallyImmuneTo`
        // row — the Confused debuff (31).
        NpcKind::TravellingMerchant => &[31],
        // `NPCID.cs:8254-8259` and `:8303-8308`: the Angler's and the
        // Sleeping Angler's `SpecificallyImmuneTo` row — the Confused
        // debuff (31).
        NpcKind::Angler => &[31],
        NpcKind::SleepingAngler => &[31],
        NpcKind::EyeOfCthulhu => &[],
        NpcKind::QueenBee => &[],
        NpcKind::SkeletronHead => &[],
        NpcKind::WallOfFlesh => &[],
        NpcKind::TheDestroyer => &[],
        NpcKind::SkeletronPrime => &[],
        NpcKind::Retinazer => &[],
        NpcKind::Spazmatism => &[],
        NpcKind::Plantera => &[],
        NpcKind::BrainOfCthulhu => &[],
        NpcKind::Creeper => &[],
        NpcKind::GoblinPeon => &[],
        NpcKind::GoblinThief => &[],
        NpcKind::GoblinWarrior => &[],
        NpcKind::GoblinArcher => &[],
        NpcKind::BoundGoblin => &[],
        NpcKind::GoblinTinkerer => &[],
        // The skeleton family/ the cave bat: no immunity data rows.
        NpcKind::Skeleton | NpcKind::UndeadMiner | NpcKind::CaveBat => &[],
        NpcKind::JungleBat | NpcKind::Hellbat | NpcKind::VoodooDemon => &[],
        // The hovers: the Eater of Souls has no immunity row (the Hornet's
        // venom-immunity rides the hornet projectile set — folded out).
        NpcKind::EaterOfSouls | NpcKind::MeteorHead => &[],
        NpcKind::Hornet => &[20],
        // The casters/spheres: no debuff-immunity rows in the data (the
        // Fire Imp's `lavaImmune` rides the liquid system).
        NpcKind::GoblinSorcerer | NpcKind::Tim | NpcKind::DarkCaster | NpcKind::FireImp => &[],
        NpcKind::ChaosBall | NpcKind::BurningSphere | NpcKind::WaterSphere => &[],
        // The swimmers: no debuff-immunity rows.
        NpcKind::Goldfish | NpcKind::Piranha | NpcKind::Shark => &[],
        // The worms: no debuff-immunity rows.
        NpcKind::DevourerHead
        | NpcKind::DevourerBody
        | NpcKind::DevourerTail
        | NpcKind::GiantWormHead
        | NpcKind::GiantWormBody
        | NpcKind::GiantWormTail
        | NpcKind::EaterOfWorldsHead
        | NpcKind::EaterOfWorldsBody
        | NpcKind::EaterOfWorldsTail => &[],
        NpcKind::KingSlime => &[20, 31],
        // The eye and the servant: no `DebuffImmunitySets` rows.
        NpcKind::EyeOfCthulhu | NpcKind::ServantOfCthulhu => &[],
        // The hungry/leeches: no `DebuffImmunitySets` rows.
        NpcKind::Hungry | NpcKind::LeechHead | NpcKind::LeechBody => &[],
        // The goblins: no `DebuffImmunitySets` rows.
        NpcKind::GoblinThief
        | NpcKind::GoblinWarrior
        | NpcKind::GoblinArcher
        | NpcKind::GoblinPeon
        | NpcKind::GoblinSummoner => &[],
        // The frost trio: no `DebuffImmunitySets` rows.
        NpcKind::SnowmanGangsta | NpcKind::SnowBalla | NpcKind::MysterySnowman => &[],
        // The martians: no `DebuffImmunitySets` rows.
        NpcKind::MartianOfficer
        | NpcKind::GrayGrunt
        | NpcKind::MartianEngineer
        | NpcKind::GigaZapper
        | NpcKind::MartianWalker
        | NpcKind::MartianProbe
        | NpcKind::MartianDrone
        | NpcKind::BrainScrambler
        | NpcKind::RayGunner
        | NpcKind::ScutlixRider
        | NpcKind::Scutlix
        | NpcKind::MartianSaucer
        | NpcKind::MartianSaucerTurret
        | NpcKind::MartianSaucerCannon
        | NpcKind::MartianSaucerCore
        | NpcKind::FlyingDutchman
        | NpcKind::FlyingDutchmanCannon
        | NpcKind::DD2GoblinT1
        | NpcKind::DD2GoblinT2
        | NpcKind::DD2GoblinT3
        | NpcKind::DD2GoblinBomberT1
        | NpcKind::DD2GoblinBomberT2
        | NpcKind::DD2GoblinBomberT3
        | NpcKind::DD2JavelinT1
        | NpcKind::DD2JavelinT2
        | NpcKind::DD2JavelinT3
        | NpcKind::DD2SkeletonT1
        | NpcKind::DD2SkeletonT3
        | NpcKind::DD2DarkMage
        | NpcKind::DD2Betsy
        | NpcKind::DD2WitherBeast
        | NpcKind::DD2Bartender
        | NpcKind::DD2EterniaCrystal
        | NpcKind::AncientCultist
        | NpcKind::StardustTower
        | NpcKind::NebulaTower
        | NpcKind::SolarTower
        | NpcKind::MoonLordHead
        | NpcKind::MoonLordHand
        | NpcKind::MoonLordCore
        | NpcKind::MoonLordFreeEye
        | NpcKind::MoonLordLeechBlob
        | NpcKind::VortexHunter
        | NpcKind::VortexAlien
        | NpcKind::VortexHeadcrab
        | NpcKind::NebulaFloater
        | NpcKind::BrainSuckler
        | NpcKind::Predictor
        | NpcKind::StardustSpiderBig
        | NpcKind::StardustSpiderSmall
        | NpcKind::StardustSoldier
        | NpcKind::SolarDrakomire
        | NpcKind::SolarDrakomireRider
        | NpcKind::SolarSroller
        | NpcKind::SolarCorite
        | NpcKind::SolarCrawltipedeHead
        | NpcKind::SolarCrawltipedeBody
        | NpcKind::SolarCrawltipedeTail
        | NpcKind::StardustWormHead
        | NpcKind::SolarSpearman
        | NpcKind::SolarSelenian
        | NpcKind::VortexTower
        | NpcKind::StardustCellBig
        | NpcKind::VortexTower
        | NpcKind::StardustCellSmall
        | NpcKind::StardustJellyfishBig
        | NpcKind::NebulaBeast
        | NpcKind::VortexRifleman
        | NpcKind::VortexSoldier => &[],
        // The pirates: no `DebuffImmunitySets` rows.
        NpcKind::PirateDeckhand
        | NpcKind::PirateCaptainsMate
        | NpcKind::PirateArcher
        | NpcKind::PirateCorsair
        | NpcKind::PirateCaptain => &[],
        NpcKind::Golem
        | NpcKind::GolemHead
        | NpcKind::GolemFistLeft
        | NpcKind::GolemFistRight
        | NpcKind::GolemHeadFree => &[],
        NpcKind::Deerclops => &[],
        NpcKind::SkeletronHand => &[],
        NpcKind::WallOfFleshEye => &[],
        NpcKind::PrimeCannon => &[],
        NpcKind::PrimeSaw => &[],
        NpcKind::PrimeVice => &[],
        NpcKind::PrimeLaser => &[],
        NpcKind::DemonEye | NpcKind::Zombie | NpcKind::WanderingEye => &[],
    }
}

impl NpcBuffs {
    /// `SetDefaults`' immunity application (`NPC.cs:17823-17833`): the kind's
    /// `DebuffImmunitySets` entry, then the derived rules — immune 20 ⇒
    /// immune 30 + 375 (`NPC.cs:17828-17833`), immune 69 ⇒ immune 36
    /// (`:17834`-shaped second rule). The 69/36 rule is data-driven in the C#
    /// (no roster entry carries 69) and stays unconditional like the original.
    pub fn for_kind(kind: NpcKind) -> Self {
        let mut buffs = Self::default();
        for &id in debuff_immune_ids(kind) {
            buffs.buff_immune[id as usize] = true;
        }
        if buffs.buff_immune[20] {
            buffs.buff_immune[30] = true;
            buffs.buff_immune[375] = buffs.buff_immune[30];
        }
        if buffs.buff_immune[69] {
            buffs.buff_immune[36] = true;
        }
        buffs
    }

    /// `NPC.AddBuff` (`NPC.cs:89926-89983`): the immune guard, the same-id
    /// early-outs, then the evict loop — first non-debuff slot, first empty
    /// slot from there, evict and retry, otherwise reject. A bar of 20
    /// debuffs rejects everything. The net/quiet branches (`NPC.cs:89932-89935`,
    /// `:89979-89982`) are single-player dead code; unlike `Player.AddBuff`
    /// there are no fed/pet/melee/reduce-time/`DetermineBuffTimeToAdd` stages.
    pub fn add_buff(&mut self, buff_type: i32, time: i32) {
        if self.buff_immune[buff_type as usize] {
            return; // NPC.cs:89928-89931
        }
        let mut num: i32 = -1;
        for i in 0..NPC_MAX_BUFFS {
            if self.buff_type[i] == buff_type {
                if self.buff_time[i] >= time {
                    return; // the longer existing time wins (NPC.cs:89941-89944)
                }
                num = i as i32; // refresh this slot below (NPC.cs:89945-89946)
                break;
            }
        }
        while num == -1 {
            // First slot that is not a debuff — an empty slot 0 qualifies,
            // `Main.debuff[0]` is false (NPC.cs:89952-89959).
            let mut num2: i32 = -1;
            for j in 0..NPC_MAX_BUFFS {
                if !buffs::is_debuff(self.buff_type[j]) {
                    num2 = j as i32;
                    break;
                }
            }
            if num2 == -1 {
                return; // 20 debuffs — nothing to evict (NPC.cs:89960-89963)
            }
            for k in num2 as usize..NPC_MAX_BUFFS {
                if self.buff_type[k] == 0 {
                    num = k as i32;
                    break;
                }
            }
            if num == -1 {
                self.del_buff(num2 as usize); // evict and retry (NPC.cs:89972-89975)
            }
        }
        self.buff_type[num as usize] = buff_type; // NPC.cs:89977-89978
        self.buff_time[num as usize] = time;
    }

    /// `NPC.FindBuffIndex` (`NPC.cs:89910-89924`): the immune table hides the
    /// buff, then the first slot with time left. The `type <= 0` bound guard
    /// follows the `buffs.rs` port of the Player variant.
    pub fn find_buff_index(&self, buff_type: i32) -> Option<usize> {
        if buff_type <= 0
            || buff_type as usize >= BUFF_ID_COUNT
            || self.buff_immune[buff_type as usize]
        {
            return None;
        }
        (0..NPC_MAX_BUFFS).find(|&i| self.buff_time[i] >= 1 && self.buff_type[i] == buff_type)
    }

    /// `NPC.DelBuff` (`NPC.cs:90002-90023`). Unlike `Player.DelBuff` (which
    /// compacts every non-empty slot to the front), the NPC version shifts the
    /// tail left by one per empty slot, scanned left to right without
    /// rescanning: `[empty, empty, C]` compacts to `[empty, C, 0]`, not
    /// `[C, 0, 0]`. The quiet/net tail (`NPC.cs:90019-90022`) is dead code.
    fn del_buff(&mut self, buff_index: usize) {
        self.buff_time[buff_index] = 0;
        self.buff_type[buff_index] = 0;
        for i in 0..NPC_MAX_BUFFS - 1 {
            if self.buff_time[i] == 0 || self.buff_type[i] == 0 {
                for j in (i + 1)..NPC_MAX_BUFFS {
                    self.buff_time[j - 1] = self.buff_time[j];
                    self.buff_type[j - 1] = self.buff_type[j];
                    self.buff_time[j] = 0;
                    self.buff_type[j] = 0;
                }
            }
        }
    }

    /// The buff block of `UpdateNPC` (`NPC.cs:90373-90377`), once per fixed
    /// step before the AI: flags reset (implicit — the flags are derived
    /// fresh) → set flags with the timer decrement → clear expired → apply
    /// DOTs. `UpdateNPC_SoulDrainDebuff` (`:90375`) and the VFX pass
    /// (`:90378`) have no port systems yet.
    pub fn update_buffs_tick(&mut self, hp: &mut i32, max_hp: i32) {
        let flags = self.set_flags_tick();
        self.clear_expired_buffs();
        self.apply_dots(&flags, hp, max_hp);
    }

    /// `UpdateNPC_BuffSetFlags` (`NPC.cs:91965-92149`): per non-empty slot
    /// with time left — decrement `buffTime`, then map the id to its flag
    /// (the port maps only the ids with ported effects; tipsy 25, midas 72,
    /// ichor 69, confused 31 and the whip marks have none yet). The
    /// vampire-slime clamps (`buffTime = 60` for buffs 24/44 when
    /// `ai[1] == 9`, `NPC.cs:92009-92016`, `:92037-92044`) have no roster
    /// kind. The `dripping` tail (`TryRemovingWaterPerishableEffects`,
    /// `:92145-92148`) needs the unported NPC Wet source.
    fn set_flags_tick(&mut self) -> NpcBuffFlags {
        let mut flags = NpcBuffFlags::default();
        for i in 0..NPC_MAX_BUFFS {
            let t = self.buff_type[i];
            if t <= 0 || self.buff_time[i] <= 0 {
                continue; // NPC.cs:91969-91972
            }
            if t == 398 {
                flags.accelerate_poisons = true; // NPC.cs:91973-91976
            }
            self.buff_time[i] -= 1; // NPC.cs:91977-91980
            match t {
                20 => flags.poisoned = true,
                30 => flags.bleeding = true,
                375 => flags.hemorrhage = true,
                70 => flags.venom = true,
                395 => flags.potent_acid = true,
                397 => flags.chlorophyte_spore = true,
                24 => flags.on_fire = true,
                323 => flags.on_fire3 = true,
                44 => flags.on_frost_burn = true,
                324 => flags.on_frost_burn2 = true,
                39 => flags.on_fire2 = true,
                153 => flags.shadow_flame = true,
                204 => flags.oiled = true,
                137 => flags.dripping_slime = true,
                _ => {}
            }
        }
        flags
    }

    /// `UpdateNPC_BuffClearExpiredBuffs` (`NPC.cs:91890-91909`). Like the C#,
    /// the loop does not rescan after `DelBuff`: an expired buff shifted into
    /// an already-passed slot is only removed on a later tick.
    fn clear_expired_buffs(&mut self) {
        for i in 0..NPC_MAX_BUFFS {
            if self.buff_type[i] > 0 && self.buff_time[i] <= 0 {
                self.del_buff(i);
            }
        }
    }

    /// `UpdateNPC_BuffApplyDOTs` (`NPC.cs:91594-91756`): build the tick's
    /// `DOTTally` from the flags in the C# branch order, fold it into
    /// `lifeRegen`/`lifeRegenCount` and drain `hp` in chunks. The
    /// `dontTakeDamage` gate (`NPC.cs:91596`) has no port flag — deviation
    /// recorded for ANALYSIS. The out-of-table flags (javelined, celled,
    /// daybreak, dryadBane, soulDrain, …) and `SuperNaturalLifeRegen`
    /// (`:91682-91700`, `:91712`) have no sources; the spore-particle tail
    /// (`:91758-91773`) is visual-only.
    fn apply_dots(&mut self, flags: &NpcBuffFlags, hp: &mut i32, max_hp: i32) {
        let mut tally = DotTally::default();
        if flags.poisoned {
            tally.add_poison(12, 0, flags.accelerate_poisons); // NPC.cs:91601-91604
        }
        if flags.bleeding {
            tally.add_blood_loss(24, 4, flags.accelerate_poisons); // NPC.cs:91605-91608
        }
        if flags.hemorrhage {
            tally.add_blood_loss(200, 40, flags.accelerate_poisons); // NPC.cs:91609-91612
        }
        // The getGood exclusion of the ai[1]==8 slime (`NPC.cs:91613-91614`)
        // cannot fire without a getGood world.
        if flags.on_fire {
            tally.add_flamable(8, 0, flags); // NPC.cs:91615-91616
        }
        if flags.on_fire3 {
            tally.add_flamable(30, 5, flags); // NPC.cs:91617-91620
        }
        if flags.on_frost_burn {
            tally.add_flamable(16, 2, flags); // NPC.cs:91621-91624
        }
        if flags.on_frost_burn2 {
            tally.add_flamable(50, 10, flags); // NPC.cs:91625-91628
        }
        if flags.on_fire2 {
            tally.add_flamable(48, 10, flags); // NPC.cs:91629-91632
        }
        if flags.venom {
            tally.add_poison(60, 15, flags.accelerate_poisons); // NPC.cs:91633-91636
        }
        if flags.potent_acid {
            tally.add_poison(180, 60, flags.accelerate_poisons); // NPC.cs:91637-91640
        }
        if flags.chlorophyte_spore {
            tally.add_poison(180, 60, flags.accelerate_poisons); // NPC.cs:91641-91644
        }
        if flags.shadow_flame {
            tally.add_flamable(30, 5, flags); // NPC.cs:91645-91648
        }
        if flags.oiled
            && (flags.on_fire
                || flags.on_fire2
                || flags.on_fire3
                || flags.on_frost_burn
                || flags.on_frost_burn2
                || flags.shadow_flame)
        {
            tally.add(50, 10); // NPC.cs:91649-91652 — a plain Add: not boostable
        }

        // `NPC.lifeRegen` after `UpdateNPC_BuffFlagsReset` (`NPC.cs:92153`);
        // the port has no other NPC regen source yet, so it starts at 0.
        let mut life_regen = 0i32;
        if tally.block_natural_regeneration && life_regen > 0 {
            life_regen = 0; // NPC.cs:91701-91704
        }
        let mut num = -1i32;
        let mut regen_loss = tally.life_regen_loss;
        if flags.accelerate_poisons {
            let boosted = tally.life_regen_loss_that_can_be_boosted;
            // `loss - boosted + (int)((float)boosted * 2f)` — exact for the
            // ported values (NPC.cs:91705-91710).
            regen_loss = tally.life_regen_loss - boosted + (boosted as f32 * 2.0) as i32;
        }
        life_regen -= regen_loss; // NPC.cs:91711
        // `lifeRegen += SuperNaturalLifeRegen` (`NPC.cs:91712`): no source yet.
        if tally.expected_damage_per_second != 0 {
            num = tally.expected_damage_per_second; // NPC.cs:91713-91716
        }
        self.life_regen_count += life_regen; // NPC.cs:91717
        if num == -1 {
            // Default chunk: grow with the accumulated deficit
            // (NPC.cs:91718-91725).
            let default_num = self.life_regen_count / -120;
            if default_num > 1 {
                num = default_num;
            }
        }
        // The heal half (`NPC.cs:91726-91740`): +1 hp per positive 120 count.
        // The `immortal` gate (`:91729`) has no port flag.
        while self.life_regen_count >= 120 {
            self.life_regen_count -= 120;
            if *hp < max_hp {
                *hp += 1;
            }
            if *hp > max_hp {
                *hp = max_hp;
            }
        }
        if num > 0 {
            while self.life_regen_count <= -120 * num {
                self.life_regen_count += 120 * num; // NPC.cs:91743-91746
                get_hurt_by_debuff(hp, num);
                if *hp <= 0 {
                    // Deviation: the C# keeps looping while the kill half of
                    // GetHurtByDebuff (`life = 1` + StrikeNPCNoInteraction(9999),
                    // `NPC.cs:91880-91887`) finishes the job; the port stops at
                    // hp <= 0 so the existing hp<=0 collect
                    // (world_sim/npcs.rs:30) drops coins/gel exactly once.
                    // ANALYSIS "NPC DoT drain" (Phase 4).
                    break;
                }
            }
        } else {
            while self.life_regen_count <= -120 {
                self.life_regen_count += 120; // NPC.cs:91751-91755
                get_hurt_by_debuff(hp, 1);
                if *hp <= 0 {
                    break; // see the deviation note above
                }
            }
        }
    }
}

/// `NPC.GetHurtByDebuff` (`NPC.cs:91864-91888`): the DoT drain bypasses
/// `StrikeNPC` — a direct `life -= amount` with no i-frames and no knockback.
/// The `NPCDamageTracker` (source 255, `:91868`) and the combat text (`:91879`)
/// have no port systems; `realLife`/`immortal` (`:91871-91878`) have no port
/// flags. The C# kill half (`life = 1` + `StrikeNPCNoInteraction(9999)`,
/// `:91880-91887`) is replaced by letting `hp` cross 0 — the existing hp<=0
/// collect (`world_sim/npcs.rs:30`) runs the normal coin/gel kill path.
fn get_hurt_by_debuff(hp: &mut i32, amount: i32) {
    *hp -= amount; // NPC.cs:91877
}

#[cfg(test)]
mod tests {
    use super::*;
    use tr_core::tile::{FLAG_ACTIVE, Tile};

    /// The occupied slot ids, in slot order.
    fn occupied(b: &NpcBuffs) -> Vec<i32> {
        b.buff_type.iter().copied().filter(|&t| t > 0).collect()
    }

    /// Distinct debuff ids for filling slots (all in `Main.debuff`); none of
    /// the ids the DoT tests drive individually (24, 30, 39, 44, 70, 137,
    /// 204, 398, 400).
    const FILLER_DEBUFFS: [i32; 21] = [
        20, 21, 22, 23, 25, 28, 31, 32, 33, 34, 35, 36, 37, 38, 43, 46, 47, 67, 68, 69, 72,
    ];

    fn fill_debuffs(b: &mut NpcBuffs, n: usize) -> Vec<i32> {
        let ids: Vec<i32> = FILLER_DEBUFFS.iter().copied().take(n).collect();
        for (i, &id) in ids.iter().enumerate() {
            b.buff_type[i] = id;
            b.buff_time[i] = 100;
        }
        ids
    }

    /// A flat dirt world with a surface, for the Npc-backed tests.
    fn flat_world() -> tr_core::World {
        let mut w = tr_core::World::new("t".into(), 64, 64);
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
    fn add_to_empty_occupies_the_first_slot() {
        // NPC.cs:89949-89978 — an empty bar places at slot 0, the next add
        // behind it.
        let mut b = NpcBuffs::default();
        b.add_buff(20, 60);
        assert_eq!(b.buff_type[0], 20);
        assert_eq!(b.buff_time[0], 60);
        b.add_buff(24, 60);
        assert_eq!(b.buff_type[1], 24);
    }

    #[test]
    fn dw_3_1_immune_reject_blocks_the_add() {
        // NPC.cs:89928-89931 — buffImmune is the first guard.
        let mut b = NpcBuffs::default();
        b.buff_immune[24] = true; // OnFire
        b.add_buff(24, 100);
        assert!(occupied(&b).is_empty());
    }

    #[test]
    fn dw_3_1_same_id_only_raises_the_time() {
        // NPC.cs:89937-89947 — an existing slot with time >= the new time
        // returns unchanged; a greater time replaces it (no additive stack).
        let mut b = NpcBuffs::default();
        b.add_buff(20, 60);
        b.add_buff(20, 30);
        assert_eq!(b.buff_time[0], 60);
        assert_eq!(occupied(&b), vec![20]);
        b.add_buff(20, 90);
        assert_eq!(b.buff_time[0], 90);
        assert_eq!(occupied(&b), vec![20]);
    }

    #[test]
    fn dw_3_1_mixed_bar_evicts_the_first_non_debuff() {
        // NPC.cs:89949-89976 — 19 debuffs plus one positive buff: the new
        // debuff evicts the positive slot and re-places into the freed slot.
        let mut b = NpcBuffs::default();
        let debuffs = fill_debuffs(&mut b, 19);
        b.add_buff(2, 100); // a positive buff lands at slot 19
        assert_eq!(b.buff_type[19], 2);
        b.add_buff(400, 100); // a 20th distinct debuff
        assert!(debuffs.iter().all(|&id| occupied(&b).contains(&id)));
        assert!(occupied(&b).contains(&400));
        assert!(!occupied(&b).contains(&2), "the positive buff was evicted");
        assert_eq!(b.buff_type[19], 400);
    }

    #[test]
    fn dw_3_1_twenty_debuffs_reject_the_21st() {
        // NPC.cs:89960-89963 — a full debuff bar finds no non-debuff slot to
        // evict and rejects both debuffs and positives.
        let mut b = NpcBuffs::default();
        fill_debuffs(&mut b, 20);
        b.add_buff(400, 100);
        assert_eq!(occupied(&b).len(), 20);
        assert!(!occupied(&b).contains(&400));
        b.add_buff(2, 100); // a positive is rejected the same way
        assert!(!occupied(&b).contains(&2));
    }

    #[test]
    fn dw_3_2_fire_family_dots_stack() {
        // NPC.cs:91613-91648 — each fire flag adds its own AddFlamable;
        // nothing replaces an earlier fire buff (the only replace is the
        // same-id branch of AddBuff, and the ids differ).
        let mut b = NpcBuffs::default();
        b.add_buff(24, 100); // OnFire — 8
        b.add_buff(323, 100); // OnFire3 — 30
        b.add_buff(44, 100); // Frostburn — 16
        let mut hp = 100;
        b.update_buffs_tick(&mut hp, 100);
        assert_eq!(b.life_regen_count, -(8 + 30 + 16));
        assert_eq!(hp, 100, "-54 has not reached the first -120 chunk");
    }

    #[test]
    fn dw_3_2_dps_is_regen_loss_over_two_on_120_ticks() {
        // Poisoned: regenLoss 12/tick drains 1 hp per -120 count → 12 hp over
        // 120 ticks = 6 DPS = 12/2 (NPC.cs:91717-91756).
        let mut b = NpcBuffs::default();
        b.add_buff(20, 1000);
        let mut hp = 100;
        for _ in 0..120 {
            b.update_buffs_tick(&mut hp, 100);
        }
        assert_eq!(hp, 88);
        assert_eq!(b.life_regen_count, 0);
    }

    #[test]
    fn dw_3_2_expected_damage_per_second_sets_the_chunk_size() {
        // Bleeding: AddBloodLoss(24, 4) — ExpectedDamagePerSecond 4 chunks the
        // drain at -480 with 4 damage per chunk (NPC.cs:91605-91608,
        // :91713-91748): the same regenLoss/2 DPS, bigger chunks.
        let mut b = NpcBuffs::default();
        b.add_buff(30, 1000);
        let mut hp = 100;
        for _ in 0..20 {
            b.update_buffs_tick(&mut hp, 100);
        }
        assert_eq!(hp, 96, "the first -480 chunk lands at tick 20");
        for _ in 20..120 {
            b.update_buffs_tick(&mut hp, 100);
        }
        assert_eq!(hp, 76, "6 chunks x 4 over 120 ticks = 24 damage");
        assert_eq!(b.life_regen_count, 0);
    }

    #[test]
    fn default_chunk_grows_with_the_accumulated_deficit() {
        // NPC.cs:91718-91725 — when no ExpectedDamagePerSecond row fired, the
        // chunk size becomes lifeRegenCount / -120 once that exceeds 1.
        // Poisoned (12/tick, expected 0) over a pre-seeded -300 count: the
        // deficit reaches -312, num3 = 2 → one 2-damage chunk at -240.
        let mut b = NpcBuffs::default();
        b.life_regen_count = -300;
        b.add_buff(20, 1000);
        let mut hp = 100;
        b.update_buffs_tick(&mut hp, 100);
        assert_eq!(hp, 98, "one chunk of num = 2 damage");
        assert_eq!(b.life_regen_count, -72, "-312 + 240");
    }

    #[test]
    fn dot_drain_crosses_zero_by_at_most_one_chunk_per_tick() {
        // NPC.cs:91741-91756 — the C# loop keeps chunking (its kill half
        // resets life to 1 and StrikeNPCNoInteraction(9999) deactivates the
        // NPC); the port breaks at hp <= 0 so at most one chunk lands past the
        // boundary. Bleeding(24) + Poisoned(12) = 36/tick with
        // ExpectedDamagePerSecond 4 → 4-damage chunks at -480; 480/36 = 13.33,
        // so the first chunk lands on tick 14 (-504, overshoot -24).
        let mut b = NpcBuffs::default();
        b.add_buff(30, 1000);
        b.add_buff(20, 1000);
        let mut hp = 2;
        for _ in 0..14 {
            b.update_buffs_tick(&mut hp, 45);
        }
        assert_eq!(hp, -2, "the death chunk takes 2 hp to -2 in one step");
        // Ten further ticks accumulate -360 (count -384, above the -480
        // threshold): no second chunk in the window where the collect
        // (world_sim/npcs.rs:30) would already have removed the NPC.
        for _ in 0..10 {
            b.update_buffs_tick(&mut hp, 45);
        }
        assert_eq!(hp, -2, "no death spiral once hp <= 0");
    }

    #[test]
    fn dw_3_3_buff_time_decrements_and_expires() {
        // NPC.cs:91977-91980 (decrement) + :91890-91909 (expiry). A 3-tick
        // buff ticks its DoT three times — the last tick still applies
        // (SetFlags runs before ClearExpired) — and is then removed.
        let mut b = NpcBuffs::default();
        b.add_buff(20, 3);
        let mut hp = 100;
        b.update_buffs_tick(&mut hp, 100);
        assert_eq!(b.buff_time[0], 2);
        b.update_buffs_tick(&mut hp, 100);
        assert_eq!(b.buff_time[0], 1);
        b.update_buffs_tick(&mut hp, 100);
        assert_eq!(b.buff_type[0], 0, "the expired buff is removed");
        assert_eq!(b.life_regen_count, -36, "three damage ticks at 12");
    }

    #[test]
    fn dw_3_3_one_tick_buff_still_ticks_then_expires() {
        // Boundary NPC.cs:91969/91977 — a time-1 buff: the decrement lands on
        // 0, the flag still applies this tick, ClearExpired removes it.
        let mut b = NpcBuffs::default();
        b.add_buff(20, 1);
        let mut hp = 100;
        b.update_buffs_tick(&mut hp, 100);
        assert_eq!(b.buff_type[0], 0);
        assert_eq!(b.life_regen_count, -12);
    }

    #[test]
    fn dw_3_3_zero_time_add_is_removed_without_effect() {
        // NPC.cs:91969-91972 — a time-0 slot is skipped by SetFlags and swept
        // by ClearExpired without ever applying its effect.
        let mut b = NpcBuffs::default();
        b.add_buff(20, 0);
        assert_eq!(b.buff_type[0], 20, "the C# stores it first");
        let mut hp = 100;
        b.update_buffs_tick(&mut hp, 100);
        assert_eq!(b.buff_type[0], 0);
        assert_eq!(b.life_regen_count, 0);
    }

    #[test]
    fn dw_3_4_dot_damage_bypasses_hurt_cd_and_knockback() {
        // NPC.cs:91877 — GetHurtByDebuff subtracts life directly; StrikeNPC's
        // hurt_cd stand-in and the knockback impulse never run. A zombie: no
        // DebuffImmunitySets entry, so Poisoned sticks (slimes are immune
        // to 20).
        let w = flat_world();
        let mut npc = crate::npc::Npc::spawn_zombie(&w, 20).unwrap();
        npc.hurt_cd = 0.5;
        npc.vx = 3.0;
        npc.buffs.add_buff(20, 1000);
        let before = npc.hp;
        for _ in 0..10 {
            npc.buffs.update_buffs_tick(&mut npc.hp, npc.max_hp);
        }
        assert_eq!(npc.hp, before - 1, "one -120 chunk over 10 poison ticks");
        assert_eq!(npc.hurt_cd, 0.5, "hurt_cd untouched");
        assert_eq!(npc.vx, 3.0, "no knockback");
    }

    #[test]
    fn dw_3_4_drain_stops_at_zero_and_death_reaches_the_collect() {
        // NPC.cs:91880-91887 kills via life = 1 + StrikeNPCNoInteraction(9999);
        // the port lets hp cross 0 once and stops draining — the existing
        // hp<=0 collect (world_sim/npcs.rs:30) then runs the normal coin/gel
        // drop path exactly once (the dead NPC is retained out in the same
        // update_npcs pass, so it never ticks again). A zombie: 45 hp, no
        // poison immunity.
        let w = flat_world();
        let mut npc = crate::npc::Npc::spawn_zombie(&w, 20).unwrap(); // 45 hp
        npc.buffs.add_buff(20, 1000); // Poisoned: 1 damage per 10 ticks
        for _ in 0..450 {
            npc.buffs.update_buffs_tick(&mut npc.hp, npc.max_hp);
        }
        assert_eq!(
            npc.hp, 0,
            "45 hp drained in 45 chunks; not clamped to 1 like the C# kill half"
        );
        // One more tick on the dead NPC: the count sits below the chunk
        // threshold, so nothing further drains within the tick.
        npc.buffs.update_buffs_tick(&mut npc.hp, npc.max_hp);
        assert_eq!(npc.hp, 0);
        // The world_sim collect predicate (world_sim/npcs.rs:30) picks it up —
        // the same filter a melee kill satisfies, so coins/gel drop once.
        let dead: Vec<&crate::npc::Npc> = std::iter::once(&npc).filter(|n| n.hp <= 0).collect();
        assert_eq!(dead.len(), 1);
    }

    #[test]
    fn dw_3_5_immune_kind_rejects_buffs_and_derives_30_375() {
        // NPCID.cs:6386-6392 (King Slime [20, 31]) + NPC.cs:17828-17833 —
        // poison immunity pulls Bleeding (30) and Hemorrhage (375) with it.
        let mut king = NpcBuffs::for_kind(NpcKind::KingSlime);
        assert!(king.buff_immune[20] && king.buff_immune[31]);
        assert!(
            king.buff_immune[30] && king.buff_immune[375],
            "derived from 20"
        );
        assert!(!king.buff_immune[24]);
        king.add_buff(20, 100);
        king.add_buff(31, 100);
        assert!(occupied(&king).is_empty(), "immune kinds reject the add");
        // All slime variants share type 1 (`NPCID.cs:6079-6084`).
        let blue = NpcBuffs::for_kind(NpcKind::Slime);
        assert!(blue.buff_immune[20] && blue.buff_immune[30] && blue.buff_immune[375]);
        // Null dictionary entries: fully susceptible (`NPC.cs:17826-17832`).
        let zombie = NpcBuffs::for_kind(NpcKind::Zombie);
        assert!(!zombie.buff_immune[20] && !zombie.buff_immune[69]);
        let eye = NpcBuffs::for_kind(NpcKind::DemonEye);
        let wandering = NpcBuffs::for_kind(NpcKind::WanderingEye);
        assert!(!eye.buff_immune[20] && !wandering.buff_immune[20]);
    }

    #[test]
    fn dw_3_5_accelerate_poisons_doubles_the_boostable_part() {
        // NPC.cs:91705-91710 — num2 = loss - boosted + boosted*2; AddPoison,
        // AddBloodLoss and AddFlamable all feed SupportAcceleration
        // (NPC.cs:59-149), so the fire family doubles too; Oiled's plain Add
        // (:91649-91652) does not.
        let mut b = NpcBuffs::default();
        b.add_buff(20, 100); // Poisoned 12, boostable
        b.add_buff(398, 100); // AcceleratePoisons
        let mut hp = 100;
        b.update_buffs_tick(&mut hp, 100);
        assert_eq!(b.life_regen_count, -24, "12 doubled");

        let mut c = NpcBuffs::default();
        c.add_buff(24, 100); // OnFire 8, boostable
        c.add_buff(398, 100);
        c.update_buffs_tick(&mut hp, 100);
        assert_eq!(c.life_regen_count, -16, "the fire part doubles too");

        let mut d = NpcBuffs::default();
        d.add_buff(20, 100); // 12 boostable
        d.add_buff(24, 100); // 8 boostable
        d.add_buff(204, 100); // Oiled +50, only with fire, not boostable
        d.add_buff(398, 100);
        d.update_buffs_tick(&mut hp, 100);
        // loss 70, boosted 20 → 70 - 20 + 40 = 90.
        assert_eq!(d.life_regen_count, -90);
    }

    #[test]
    fn dot_table_rows_match_the_decompile() {
        // One tick per row (NPC.cs:91601-91652). Every row accumulates exactly
        // -regenLoss into lifeRegenCount; rows with an ExpectedDamagePerSecond
        // chunk at -120·expected (NPC.cs:91713-91716), far beyond one tick, so
        // no row changes hp in a single tick.
        let rows: [(i32, i32, i32, i32); 12] = [
            // (buff id, regenLoss, expected count, expected hp delta)
            (20, 12, -12, 0),    // Poisoned — expected 0, chunks at -120
            (70, 60, -60, 0),    // Venom — expected 15
            (395, 180, -180, 0), // PotentAcid — expected 60, chunks at -7200
            (397, 180, -180, 0), // ChlorophyteSpore — expected 60
            (30, 24, -24, 0),    // Bleeding — expected 4
            (375, 200, -200, 0), // Hemorrhage — expected 40, chunks at -4800
            (24, 8, -8, 0),      // OnFire — expected 0
            (323, 30, -30, 0),   // OnFire3 — expected 5
            (44, 16, -16, 0),    // Frostburn — expected 2
            (324, 50, -50, 0),   // Frostburn2 — expected 10
            (39, 48, -48, 0),    // CursedInferno — expected 10
            (153, 30, -30, 0),   // ShadowFlame — expected 5
        ];
        for (id, loss, count, hp_delta) in rows {
            let mut b = NpcBuffs::default();
            b.add_buff(id, 10);
            let mut hp = 10_000;
            b.update_buffs_tick(&mut hp, 10_000);
            assert_eq!(b.life_regen_count, count, "buff {id} regenLoss {loss}");
            assert_eq!(hp, 10_000 + hp_delta, "buff {id}");
        }
    }

    #[test]
    fn dripping_slime_doubles_flamable_losses() {
        // DOTTally.AddFlamable (NPC.cs:102-114): buff 137 present → x2.
        let mut b = NpcBuffs::default();
        b.add_buff(24, 100); // OnFire 8
        b.add_buff(137, 100); // drippingSlime
        let mut hp = 100;
        b.update_buffs_tick(&mut hp, 100);
        assert_eq!(b.life_regen_count, -16);
        // Poison is not flammable: unaffected.
        let mut c = NpcBuffs::default();
        c.add_buff(20, 100);
        c.add_buff(137, 100);
        c.update_buffs_tick(&mut hp, 100);
        assert_eq!(c.life_regen_count, -12);
    }

    #[test]
    fn oiled_alone_adds_nothing() {
        // NPC.cs:91649-91652 — Oiled needs a fire flag in the same tick.
        let mut b = NpcBuffs::default();
        b.add_buff(204, 100);
        let mut hp = 100;
        b.update_buffs_tick(&mut hp, 100);
        assert_eq!(b.life_regen_count, 0);
        b.add_buff(39, 100); // CursedInferno lights the combo
        b.update_buffs_tick(&mut hp, 100);
        assert_eq!(b.life_regen_count, -(48 + 50));
    }

    #[test]
    fn positive_count_heals_one_per_120() {
        // NPC.cs:91726-91740 — the heal half of the count loop (reachable by
        // a pre-seeded count until a positive-regen source exists), with the
        // lifeMax clamp.
        let mut b = NpcBuffs::default();
        b.life_regen_count = 240;
        let mut hp = 10;
        b.update_buffs_tick(&mut hp, 20);
        assert_eq!(hp, 12);
        assert_eq!(b.life_regen_count, 0);

        let mut c = NpcBuffs::default();
        c.life_regen_count = 120;
        let mut hp2 = 20;
        c.update_buffs_tick(&mut hp2, 20);
        assert_eq!(hp2, 20, "lifeMax clamps the heal");
    }

    #[test]
    fn clear_expired_quirk_keeps_a_shifted_in_expired_buff_one_tick() {
        // NPC.cs:91897-91904 — the loop does not rescan after DelBuff: the
        // expired buff shifted into an already-passed slot survives one tick.
        let mut b = NpcBuffs::default();
        b.buff_type[0] = 20;
        b.buff_time[0] = 0; // Poisoned expired
        b.buff_type[1] = 30;
        b.buff_time[1] = 0; // Bleeding expired
        b.buff_type[2] = 70;
        b.buff_time[2] = 5; // Venom alive
        let mut hp = 100;
        b.update_buffs_tick(&mut hp, 100);
        // DelBuff(0) pulled Bleeding into slot 0; the scan never re-checks it.
        assert_eq!(b.buff_type[0], 30);
        assert_eq!(b.buff_time[0], 0);
        assert_eq!(b.buff_type[1], 70);
        assert_eq!(b.buff_time[1], 4);
        // The next tick's pass removes it.
        b.update_buffs_tick(&mut hp, 100);
        assert_eq!(b.buff_type[0], 70, "the shifted-in expired buff is gone");
        assert_eq!(b.buff_type[1], 0);
    }

    #[test]
    fn del_buff_shifts_the_tail_once_per_empty_without_rescanning() {
        // NPC.cs:90002-90023 — unlike Player.DelBuff's full compaction, the
        // NPC version shifts the tail left by one per empty slot: two leading
        // empties leave the buff at slot 1, not slot 0.
        let mut b = NpcBuffs::default();
        b.buff_type[2] = 70;
        b.buff_time[2] = 5;
        b.del_buff(0);
        assert_eq!(b.buff_type[0], 0);
        assert_eq!(b.buff_type[1], 70, "one shift per empty slot");
        assert_eq!(b.buff_type[2], 0);
    }

    #[test]
    fn find_buff_index_matches_the_c_sharp() {
        // NPC.cs:89910-89924 — the immune table hides the buff, then the
        // first slot with time >= 1.
        let mut b = NpcBuffs::default();
        assert_eq!(b.find_buff_index(20), None);
        b.add_buff(20, 60);
        b.add_buff(70, 60);
        assert_eq!(b.find_buff_index(70), Some(1));
        b.buff_immune[20] = true;
        assert_eq!(b.find_buff_index(20), None);
        assert_eq!(b.find_buff_index(0), None);
    }
}
