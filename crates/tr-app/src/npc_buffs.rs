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
//!   `NPCID.Sets.DebuffImmunitySets` (`NPCID.cs:6075+`); the shimmer-353
//!   flag: `NPC.cs:17838` + `NPCID.Sets.ShimmerImmunity` (`NPCID.cs:10467`,
//!   W4-d6 residual 9)
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
pub struct NpcBuffFlags {
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
    /// buff 353 `Shimmering` (`NPC.cs:92133-92143`) — the immune-353 branch
    /// DELETES the buff mid-scan instead of setting this flag (verbatim
    /// `DelBuff(i)`, the shift-left skips the shifted-in slot this tick).
    pub shimmering: bool,
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

/// `NPCID.Sets.ShimmerImmunity` (`NPCID.cs:10467`) — the verbatim id list
/// (`CreateBoolSet` default-false, the listed ids TRUE). The buff-353
/// immune flag (`NPC.cs:17838`) reads it; the ShimmerGun's NPC contact
/// (`Projectile.cs:29039`) and the shimmer wet contact
/// (`NPC.cs:93073`) both ride `AddBuff`'s immune guard.
/// RETIRE KEEP-ONE (research/330 §5a): W4-c9's `shimmer_immune(cs_id)`
/// (`2042d77f1`, 3 h earlier) ports the SAME :17838 write with the
/// set-exact range-compressed predicate — c9 WINS first-claim; this
/// const + the `for_kind` write below DROP at merge (the S1–S3 hunks),
/// the set-gate witness folds onto c9's fn. f47's flag READS compose
/// with either writer unchanged.
const SHIMMER_IMMUNITY: &[u16] = &[
    637, 638, 656, 670, 684, 678, 679, 680, 681, 682, 683, 356, 669, 676, 244, 677, 594, 667,
    662, 5, 115, 116, 139, 245, 247, 248, 246, 249, 344, 325, 50, 535, 657, 658, 659, 660, 668,
    25, 30, 33, 70, 72, 665, 666, 112, 516, 517, 518, 519, 520, 521, 522, 523, 381, 382, 383,
    384, 385, 386, 387, 388, 389, 390, 391, 392, 393, 394, 395, 396, 397, 398, 399, 400, 401,
    402, 403, 404, 405, 406, 407, 408, 409, 410, 411, 412, 413, 414, 415, 416, 417, 418, 419,
    420, 421, 423, 424, 425, 426, 427, 428, 429, 548, 549, 551, 552, 553, 554, 555, 556, 557,
    558, 559, 560, 561, 562, 563, 564, 565, 566, 567, 568, 569, 570, 571, 572, 573, 574, 575,
    576, 577, 578,
];

/// The roster's slice of `NPCID.Sets.DebuffImmunitySets` (`NPCID.cs:6075+`):
/// the `SpecificallyImmuneTo` data of each ported kind. DemonEye (2,
/// `NPCID.cs:6085`), Zombie (3, `NPCID.cs:6086`) and WanderingEye (133,
/// `NPCID.cs:6905`) have null entries — no immunity, like the C# else-branch
/// (`NPC.cs:17826-17832`). The slime variants all share type 1.
/// `ImmuneToAllBuffsThatAreNotWhips` rows (the Poltergeist 330, the Force
/// Bubble 384, the saucer family 392-395) have no slot field here — this
/// table carries `SpecificallyImmuneTo` only, so those kinds ride `&[]`
/// (the port's documented gap, see the family arms below).
/// `ImmuneToWhips` rides [`whip_immune`] separately.
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
        // `NPCID.cs:8254-8308` has no `DebuffImmunitySets` row for 106/
        // 108/123/124 — the same empty row as the rest of the town set.
        NpcKind::Wizard => &[],
        NpcKind::BoundWizard => &[],
        NpcKind::Mechanic => &[],
        NpcKind::BoundMechanic => &[],
        // `NPCID.cs:8143-8148`/`:8604-8610`: the Stylist's, the webbed
        // Stylist's and the Tax Collector's `SpecificallyImmuneTo` rows —
        // the Confused debuff (31).
        NpcKind::Stylist => &[31],
        NpcKind::WebbedStylist => &[31],
        NpcKind::TaxCollector => &[31],
        // `NPCID.cs:6080-6089`: the Golfer's and the sleeping Golfer's
        // `SpecificallyImmuneTo` rows — the Confused debuff (31).
        NpcKind::Golfer => &[31],
        NpcKind::SleepingGolfer => &[31],
        // `NPCID.cs`'s DebuffImmunitySets has no row for 633/663.
        NpcKind::Zoologist => &[],
        NpcKind::Princess => &[],
        // `NPCID.cs`'s DebuffImmunitySets has no row for 207/208.
        NpcKind::DyeTrader => &[],
        NpcKind::PartyGirl => &[],
        NpcKind::EyeOfCthulhu => &[],
        NpcKind::QueenBee => &[],
        NpcKind::SkeletronHead => &[],
        NpcKind::WallOfFlesh => &[],
        NpcKind::TheDestroyer => &[],
        // 135/136/139 have no DebuffImmunitySets rows (`NPCID.cs:6075+`).
        NpcKind::TheDestroyerBody | NpcKind::TheDestroyerTail | NpcKind::Probe => &[],
        NpcKind::SkeletronPrime => &[],
        NpcKind::Retinazer => &[],
        NpcKind::Spazmatism => &[],
        NpcKind::Plantera => &[],
        // The Plantera minions (S1-D-3-wire-plantera-minions): no
        // `NPCID.Sets.SpecificallyImmuneTo` rows exist for 261/263/264/265
        // (`ID/NPCID.cs` - the four ids have no Sets entries at all).
        NpcKind::FungiSpore | NpcKind::PlanterasHook | NpcKind::PlanterasTentacle | NpcKind::Spore => &[],
        NpcKind::BrainOfCthulhu => &[],
        NpcKind::Creeper => &[],
        NpcKind::GoblinPeon => &[],
        NpcKind::GoblinThief => &[],
        NpcKind::GoblinWarrior => &[],
        NpcKind::GoblinArcher => &[],
        NpcKind::BoundGoblin => &[],
        NpcKind::GoblinTinkerer => &[],
        // W4-g4 P9 (the W4-d22 reconcile fold) — the skeleton family's
        // rows (`NPCID.cs:6207-6212`/`:6357-6362`): Skeleton 21 and
        // UndeadMiner 44 are [20] (the derived 30/375 ride `for_kind`);
        // the cave bat 49 has no row. The old "no immunity data rows"
        // fold was wrong.
        NpcKind::Skeleton | NpcKind::UndeadMiner => &[20],
        NpcKind::CaveBat => &[],
        NpcKind::JungleBat | NpcKind::Hellbat | NpcKind::VoodooDemon => &[],
        // The hovers: the Eater of Souls has no immunity row (the Hornet's
        // venom-immunity rides the hornet projectile set — folded out).
        NpcKind::EaterOfSouls | NpcKind::MeteorHead => &[],
        NpcKind::Hornet => &[20],
        // The volley bees: `NPCID.Sets.DebuffImmunitySets` rows 210/211
        // — `SpecificallyImmuneTo = { 20, 31 }` (`NPCID.cs:7312-7325`).
        NpcKind::Bee | NpcKind::BeeSmall => &[20, 31],
        // The casters' `NPCID.Sets.DebuffImmunitySets` rows (W4-g5 RETIRE
        // census — the base comment "no debuff-immunity rows in the data"
        // was wrong; the rows exist): 29 {31} (`NPCID.cs:6244-6250`),
        // 45 {20,31} (`:6364-6368`), 32 {20,31} (`:6272-6278`),
        // 24 {24,31,323} (`:6227-6233`), 533 the 14-row set (`:9085-9095`).
        // The Fire Imp's `lavaImmune` rides the liquid system separately.
        NpcKind::GoblinSorcerer => &[31],
        NpcKind::Tim | NpcKind::DarkCaster => &[20, 31],
        NpcKind::FireImp => &[24, 31, 323],
        NpcKind::DesertDjinn => &[20, 24, 31, 39, 44, 69, 70, 395, 153, 189, 203, 204, 323, 324],
        // The spheres: 25/30/33 each {31} (`NPCID.cs:6234-6240`/`:6251-6257`/
        // `:6279-6285`); the 665 ChaosBallTim clone shares the 30's row
        // (`:6258-6264`) — the R4 fold keeps ONE kind, so the {31} row
        // covers both net ids.
        NpcKind::ChaosBall | NpcKind::BurningSphere | NpcKind::WaterSphere => &[31],
        // The swimmers: no debuff-immunity rows.
        NpcKind::Goldfish | NpcKind::Piranha | NpcKind::Shark => &[],
        // W4-c4 (the style-16 swimmers): `Sets.DebuffImmunitySets` rows
        // 57/157/241/465/592/607/615 are `{ SpecificallyImmuneTo: [31] }`
        // (`NPCID.cs:6075+` — On Fire! immunity, the wyvern precedent);
        // 688/692 have NO row (the null else-branch, `NPC.cs:17822-17828`).
        // E8 merge drops: CorruptGoldfish/BloodFeeder (d34/d37 rows
        // above own them), Pufferfish/Orca (d43 lands later - the null
        // else-branch already covers their absence).
        NpcKind::Arapaima
        | NpcKind::CrimsonGoldfish
        | NpcKind::GoldGoldfish
        | NpcKind::Pupfish
        | NpcKind::Dolphin => &[31],
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
        // The wyvern family: `{ 87..92, SpecificallyImmuneTo: [31] }`
        // (`NPCID.cs:6600-6641`) — On Fire! immunity, all six kinds.
        NpcKind::WyvernHead
        | NpcKind::WyvernLegs
        | NpcKind::WyvernBody
        | NpcKind::WyvernBody2
        | NpcKind::WyvernBody3
        | NpcKind::WyvernTail => &[31],
        NpcKind::KingSlime => &[20, 31],
        // The eye and the servant: no `DebuffImmunitySets` rows.
        NpcKind::EyeOfCthulhu | NpcKind::ServantOfCthulhu => &[],
        // The hungry/leeches: no `DebuffImmunitySets` rows (119 neither).
        NpcKind::Hungry | NpcKind::LeechHead | NpcKind::LeechBody | NpcKind::LeechTail => &[],
        // The goblins: the invasion four have no `DebuffImmunitySets`
        // rows (`NPCID.cs:6241-6243`/`:6756`); W4-g4 P9 — the Summoner
        // 471 is [31, 153] (`NPCID.cs:8767-8771`) — Confused-immune, so
        // its P2 phase machine never sees the buff-31 latch.
        NpcKind::GoblinThief
        | NpcKind::GoblinWarrior
        | NpcKind::GoblinArcher
        | NpcKind::GoblinPeon => &[],
        NpcKind::GoblinSummoner => &[31, 153],
        // W4-g4 P9 — the frost trio's rows (`NPCID.cs:6970-6990`): the
        // Gangsta 143, the Stabby 144 (the port's SnowBalla) and the
        // Balla 145 (the port's MysterySnowman) are [20, 44, 324].
        NpcKind::SnowmanGangsta | NpcKind::SnowBalla | NpcKind::MysterySnowman => &[20, 44, 324],
        // The martians (`S1-F-13d` audit — `NPCID.Sets.DebuffImmunitySets`,
        // `NPCID.cs:8338-8401`/`:8423-8428`/`:9032-9037`): the Officer 383,
        // the Grunt 385, the Engineer 386, the Zapper 389, the rider 390,
        // the carrier 391 and the Ray Gunner 382 are null rows; the Brain
        // Scrambler 381 is [31] (`:8339-8343`), the drone 388 and the
        // probe 399 are [20, 31] (`:8363-8368`/`:8423-8428`), the walker
        // 520 is [20, 24, 31, 323] (`:9032-9037`), the Tesla Turret 387 is
        // [20, 31] (`:8356-8361`). The Force Bubble 384 (`:8347-8353`)
        // and the saucer family 392-395 (`:8373-8401`) carry
        // `ImmuneToAllBuffsThatAreNotWhips = true` — the mask loop is
        // unmodelled, so they ride `&[]` (the documented gap; the
        // saucer's `ImmuneToWhips` rides `whip_immune`).
        NpcKind::MartianOfficer
        | NpcKind::GrayGrunt
        | NpcKind::MartianEngineer
        | NpcKind::GigaZapper
        | NpcKind::RayGunner
        | NpcKind::ScutlixRider
        | NpcKind::Scutlix
        | NpcKind::ForceBubble
        | NpcKind::MartianSaucer
        | NpcKind::MartianSaucerTurret
        | NpcKind::MartianSaucerCannon
        | NpcKind::MartianSaucerCore => &[],
        NpcKind::BrainScrambler => &[31],
        NpcKind::MartianDrone | NpcKind::MartianProbe | NpcKind::MartianTurret => &[20, 31],
        NpcKind::MartianWalker => &[20, 24, 31, 323],
        // W4-g4 P9 (the W4-d22 reconcile fold) — the DD2 rows
        // (`NPCID.cs:9183-9390`), per kind: the crystal 548 / bartender
        // 550 / goblins 552-554 / bombers 555-557 / wyverns 558-560 /
        // javelineers 561-563 / dark mages 564+565 / wither beasts
        // 568+569 / drakins 570-571 / kobolds 572-575 / ogres 576-577
        // are ALL [31] (Confused-immune); Betsy 551 is [24, 31, 323]
        // (`:9205-9210`); the skeletons 566/567 are [20, 31]
        // (`:9310-9322`). The lane portal 549 is the
        // `ImmuneToAllBuffsThatAreNotWhips + ImmuneToWhips` mask
        // (`:9190-9196`) — the mask loop is unmodelled, `&[]` stands in
        // (the documented gap) and `ImmuneToWhips` rides `whip_immune`.
        NpcKind::DD2EterniaCrystal
        | NpcKind::DD2Bartender
        | NpcKind::DD2GoblinT1
        NpcKind::FlyingDutchman
        | NpcKind::FlyingDutchmanCannon
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
        | NpcKind::DD2WitherBeast
        | NpcKind::DD2WyvernT1
        | NpcKind::DD2WyvernT2
        | NpcKind::DD2WyvernT3
        | NpcKind::DD2JavelinT1
        | NpcKind::DD2JavelinT2
        | NpcKind::DD2JavelinT3
        | NpcKind::DD2DarkMage
        | NpcKind::DD2DarkMageT3
        | NpcKind::DD2WitherBeast
        | NpcKind::DD2WitherBeastT3
        | NpcKind::DD2DrakinT2
        | NpcKind::DD2DrakinT3
        | NpcKind::DD2KoboldWalkerT2
        | NpcKind::DD2KoboldWalkerT3
        | NpcKind::DD2KoboldFlyerT2
        | NpcKind::DD2KoboldFlyerT3
        | NpcKind::DD2OgreT2
        | NpcKind::DD2OgreT3 => &[31],
        NpcKind::DD2Betsy => &[24, 31, 323],
        NpcKind::DD2SkeletonT1 | NpcKind::DD2SkeletonT3 => &[20, 31],
        // W4-g4 P9 — the lunar-mob rows (`NPCID.cs:8445-8560`/`:9019`):
        // the worm head 402, cells 405/406, jellyfish 407, crawltipede
        // 412-414, floater 420, suckler 421, headcrab 428 and spearman
        // 518 are [31]; the Corite 418 is [20, 24, 31, 323]
        // (`:8521-8526`). The spiders 409/410, soldier 411, drakomire
        // 415 + rider 416, sroller 417, selenian 419, beast 423,
        // predictor 424, rifleman 425, hunter 426, alien 427 and
        // soldier 429 are null rows (`:8493-8495`/`:8517-8519`/
        // `:8527`/`:8549-8553`/`:8561`) — Confused CAN land on them.
        NpcKind::StardustWormHead
        | NpcKind::StardustCellBig
        | NpcKind::StardustCellSmall
        | NpcKind::StardustJellyfishBig
        | NpcKind::SolarCrawltipedeHead
        | NpcKind::SolarCrawltipedeBody
        | NpcKind::SolarCrawltipedeTail
        | NpcKind::NebulaFloater
        | NpcKind::BrainSuckler
        | NpcKind::VortexHeadcrab
        | NpcKind::SolarSpearman => &[31],
        NpcKind::SolarCorite => &[20, 24, 31, 323],
        // The null-row remainder: the towers 422/493/507/517 and the
        // cultist 439 carry `ImmuneToAllBuffsThatAreNotWhips`
        // (`NPCID.cs:8543-8548`/`:8590`/`:8910`/`:8954`/`:9012`) — the
        // unmodelled mask rides `&[]` (the documented gap); the
        // Dutchman pair 491/492 has no rows.
        NpcKind::FlyingDutchman
        | NpcKind::FlyingDutchmanCannon
        | NpcKind::DD2OgreT3
        | NpcKind::AncientCultist
        | NpcKind::StardustTower
        | NpcKind::NebulaTower
        | NpcKind::SolarTower
        | NpcKind::VortexTower
        | NpcKind::MoonLordHead
        | NpcKind::MoonLordHand
        | NpcKind::MoonLordCore
        | NpcKind::MoonLordFreeEye
        | NpcKind::MoonLordLeechBlob
        | NpcKind::VortexHunter
        | NpcKind::VortexAlien
        | NpcKind::Predictor
        | NpcKind::StardustSpiderBig
        | NpcKind::StardustSpiderSmall
        | NpcKind::StardustSoldier
        | NpcKind::SolarDrakomire
        | NpcKind::SolarDrakomireRider
        | NpcKind::SolarSroller
        | NpcKind::SolarSelenian
        | NpcKind::DD2LanePortal
        | NpcKind::VortexTower
        | NpcKind::StardustCellBig
        | NpcKind::VortexTower
        | NpcKind::StardustCellSmall
        | NpcKind::StardustJellyfishBig
        | NpcKind::NebulaBeast
        | NpcKind::VortexRifleman
        | NpcKind::VortexSoldier => &[],
        // The pirates: W4-g4 P9 — the Captain 216 is [31]
        // (`NPCID.cs:7331-7336`, Confused-immune); the Deckhand 212,
        // Mate 213, Deadeye 214 and Crossbower 215 are null rows
        // (`:7326-7329`).
        NpcKind::PirateDeckhand
        | NpcKind::PirateCaptainsMate
        | NpcKind::PirateArcher
        | NpcKind::PirateCorsair => &[],
        NpcKind::PirateCaptain => &[31],
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
        // The Pumpkin Moon family (`S1-F-13b` — `Sets.DebuffImmunitySets`,
        // `NPCID.cs:6075+`): the 305-314 group and the Splinterling have no
        // row; 315 and 327 the ({20, 24, 31, 323}) row (`:7945`/`:7997`), 325
        // the ({20, 31}) row (`:7989`), 329 the ({20, 24, 323}) row
        // (`:8011`). 330's row is `ImmuneToAllBuffsThatAreNotWhips = true`
        // (`:8018`) — this table carries `SpecificallyImmuneTo` only and the
        // mask loop is unmodelled, so the kind rides `&[]` here (the port's
        // documented gap for every such row).
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
        | NpcKind::Splinterling
        | NpcKind::Poltergeist => &[],
        NpcKind::HeadlessHorseman => &[20, 24, 31, 323],
        NpcKind::MourningWood => &[20, 31],
        NpcKind::Pumpking => &[20, 24, 31, 323],
        // W4-c6 style-59 wiring: the blade's `Sets.DebuffImmunitySets` row
        // (`NPCID.cs:8002-8008`): {20, 24, 31, 323} — the carrier's row.
        NpcKind::Hellhound => &[20, 24, 323],
        // The Frost Moon family (`S1-F-13c` slice B — `Sets.DebuffImmunitySets`,
        // `NPCID.cs:8104-8141`): 345/346/352 the ({20, 31, 44, 324}) row
        // (`:8104-8110`/`:8111-8117`/`:8135-8141`), 347 the ({31}) row
        // (`:8118-8124`), 348/349/350 the null rows (`:8125-8127`), 351 the
        // ({44, 324}) row (`:8128-8134`).
        NpcKind::IceQueen | NpcKind::SantaNK1 | NpcKind::Flocko => &[20, 31, 44, 324],
        NpcKind::ElfCopter => &[31],
        NpcKind::Nutcracker | NpcKind::NutcrackerSpinning | NpcKind::ElfArcher => &[],
        NpcKind::Krampus => &[44, 324],
        // The Frost Moon family, slice A (`S1-F-13c` —
        // `Sets.DebuffImmunitySets`, `NPCID.cs:8062-8102`): the 338-340 trio
        // and the Yeti the ({44, 324}) row (Frostburn + Frozen), 341 the
        // ({20, 24, 31, 323}) row, 344 the ({20, 31, 44, 324}) row; 342's
        // row is `null` (`:8089`).
        NpcKind::ZombieElf
        | NpcKind::ZombieElfBeard
        | NpcKind::ZombieElfGirl
        | NpcKind::Yeti => &[44, 324],
        NpcKind::PresentMimic => &[20, 24, 31, 323],
        NpcKind::GingerbreadMan => &[],
        NpcKind::Everscream => &[20, 31, 44, 324],
        NpcKind::DemonEye | NpcKind::Zombie | NpcKind::WanderingEye => &[],
        // S1-D-3ff wave A — the `DebuffImmunitySets` rows verbatim:
        // 16/71/81/138/183/225 `[20]` (`NPCID.cs:6171-6177`/`:6516-6522`/
        // `:6557-6563`/`:6934-6940`/`:7189-7195`/`:7381-7387`), 59
        // `[20, 24, 323]` (`:6431-6437`), 147/184 `[20, 44, 324]`
        // (`:6997-7003`/`:7196-7202`), 161 `[44, 324]` (`:7065-7071`);
        // the five eyes and 52/53/132/186-189/200/223 are null rows
        // (`:7214-7218`/`:6394-6395`/`:6904`/`:7210-7213`/`:7248`/
        // `:7379`) — no immunity, the C# else-branch.
        NpcKind::MotherSlime
        | NpcKind::DungeonSlime
        | NpcKind::CorruptSlime
        | NpcKind::IlluminantSlime
        | NpcKind::Crimslime
        | NpcKind::UmbrellaSlime => &[20],
        NpcKind::LavaSlime => &[20, 24, 323],
        NpcKind::IceSlime | NpcKind::SpikedIceSlime => &[20, 44, 324],
        NpcKind::ZombieEskimo => &[44, 324],
        NpcKind::CataractEye
        | NpcKind::SleepyEye
        | NpcKind::DialatedEye
        | NpcKind::GreenEye
        | NpcKind::PurpleEye
        | NpcKind::DoctorBones
        | NpcKind::TheGroom
        | NpcKind::BaldZombie
        | NpcKind::PincushionZombie
        | NpcKind::SlimedZombie
        | NpcKind::SwampZombie
        | NpcKind::TwiggyZombie
        | NpcKind::FemaleZombie
        | NpcKind::ZombieRaincoat
        // S1-D-3z — the holiday zombies' null `DebuffImmunitySets` rows
        // (`NPCID.cs:7964-7966` for 319-321, `:8024-8025` for 331-332).
        | NpcKind::ZombieDoctor
        | NpcKind::ZombieSuperman
        | NpcKind::ZombiePixie
        | NpcKind::ZombieXmas
        | NpcKind::ZombieSweater
        // S1-D-3z — the blood-moon pair's null `DebuffImmunitySets` rows
        // (`NPCID.cs:8892-8893`).
        | NpcKind::BloodZombie
        | NpcKind::Drippler => &[],
        // W4-d40: the `DebuffImmunitySets` rows (`NPCID.cs`).
        NpcKind::ArmedZombie
        | NpcKind::ArmedZombiePincussion
        | NpcKind::ArmedZombieSlimed
        | NpcKind::ArmedZombieSwamp => &[],
        NpcKind::ArmedZombieEskimo => &[44, 324],
        // W4-d40: the `DebuffImmunitySets` rows (`NPCID.cs`).
        NpcKind::ArmedZombieTwiggy
        | NpcKind::ArmedZombieCenx
        | NpcKind::CultistBossClone => &[],
        NpcKind::CultistTablet
        | NpcKind::CultistDevote => &[31],
        // W4-d40: the `DebuffImmunitySets` rows (`NPCID.cs`).
        NpcKind::GoldBird
        | NpcKind::GoldBunny
        | NpcKind::GoldButterfly
        | NpcKind::GoldFrog
        | NpcKind::GoldGrasshopper => &[31],
        // W4-d40: the `DebuffImmunitySets` rows (`NPCID.cs`).
        NpcKind::GoldMouse
        | NpcKind::GoldWorm => &[31],
        NpcKind::BoneThrowingSkeleton
        | NpcKind::BoneThrowingSkeleton2
        | NpcKind::BoneThrowingSkeleton3 => &[20],
        // W4-d40: the `DebuffImmunitySets` rows (`NPCID.cs`).
        NpcKind::BoneThrowingSkeleton4 => &[20],
        NpcKind::SkeletonMerchant => &[31],
        NpcKind::CultistDragonHead
        | NpcKind::CultistDragonBody1
        | NpcKind::CultistDragonBody2 => &[],
        // W4-d40: the `DebuffImmunitySets` rows (`NPCID.cs`).
        NpcKind::CultistDragonBody3
        | NpcKind::CultistDragonBody4
        | NpcKind::CultistDragonTail
        | NpcKind::Butcher => &[],
        NpcKind::CreatureFromTheDeep => &[31],
        // W4-d40: the `DebuffImmunitySets` rows (`NPCID.cs`).
        NpcKind::Fritz
        | NpcKind::Nailhead
        | NpcKind::CrimsonBunny
        | NpcKind::Psycho => &[],
        // W4-d40: the `DebuffImmunitySets` rows (`NPCID.cs`).
        NpcKind::DeadlySphere => &[20, 24, 31, 323],
        NpcKind::DrManFly
        | NpcKind::ThePossessed
        | NpcKind::CrimsonPenguin => &[],
        NpcKind::ShadowFlameApparition => &[31, 153],
        // W4-d40: the `DebuffImmunitySets` rows (`NPCID.cs`).
        NpcKind::BigMimicCorruption
        | NpcKind::BigMimicCrimson
        | NpcKind::BigMimicHallow
        | NpcKind::BigMimicJungle => &[20, 24, 31, 323],
        NpcKind::Mothron => &[31],
        // W4-d40: the `DebuffImmunitySets` rows (`NPCID.cs`).
        NpcKind::MothronEgg
        | NpcKind::MothronSpawn
        | NpcKind::Medusa => &[31],
        NpcKind::GreekSkeleton => &[20],
        NpcKind::GraniteGolem => &[20, 24, 31, 323],
        // W4-d40: the `DebuffImmunitySets` rows (`NPCID.cs`).
        NpcKind::GraniteFlyer => &[20, 24, 31, 323],
        NpcKind::EnchantedNightcrawler
        | NpcKind::Grubby
        | NpcKind::Sluggy
        | NpcKind::Buggy => &[31],
        // W4-d40: the `DebuffImmunitySets` rows (`NPCID.cs`).
        NpcKind::TargetDummy => &[31],
        // S1-D-3 W-9 fishron wiring — 370/371/372/373 have NO
        // `DebuffImmunitySets` rows (`NPCID.cs:6075+`'s dictionary carries
        // no 370-373 keys) — the null-row else-branch, no immunity.
        NpcKind::DukeFishron
        | NpcKind::DetonatingBubble
        | NpcKind::Sharkron
        | NpcKind::Sharkron2 => &[],
        // The small-kinds wiring wave: none of 72/250/288/384 has a
        // `DebuffImmunitySets` row (`NPCID.cs:6075+` has no entry for
        // them) — the C# else-branch's no-immunity (`NPC.cs:17826-17832`).
        NpcKind::BlazingWheel
        | NpcKind::AngryNimbus
        | NpcKind::DungeonSpirit => &[],
        // W4-d34 kinds 1-70 (`NPCID.cs` DebuffImmunitySets).
        NpcKind::AngryBones => &[20],          // :6266-6270
        NpcKind::CursedSkull => &[20, 31],     // :6287-6291
        NpcKind::OldMan => &[31],              // :6308-6312
        NpcKind::ManEater => &[20, 31],        // :6350-6354
        NpcKind::Bunny => &[31],               // :6371-6375
        NpcKind::BoneSerpentHead | NpcKind::BoneSerpentBody | NpcKind::BoneSerpentTail => &[20, 24, 31, 323], // :6322-6340
        NpcKind::CorruptBunny => &[],          // :6377 null row
        NpcKind::Harpy => &[31],               // :6379-6383
        NpcKind::Snatcher => &[31],            // :6410-6416
        NpcKind::CorruptGoldfish => &[31],     // :6417-6423
        NpcKind::Vulture => &[31],             // :6445-6451
        NpcKind::Demon => &[24, 31, 153, 323], // :6452-6458
        NpcKind::BlueJellyfish => &[31],       // :6459-6465
        NpcKind::Crab => &[31],                // :6487-6493
        // The Dungeon Guardian's ImmuneToAllBuffsThatAreNotWhips +
        // ImmuneToWhips row (`NPCID.cs:6495-6501`) — the mask loop rides the
        // documented `&[]` gap (the saucer-family precedent); the whip tag
        // rides `whip_immune` below.
        NpcKind::DungeonGuardian => &[],
        // W4-d38 b7 — SpecificallyImmuneTo {20}.
        NpcKind::SlimeRibbonWhite
        | NpcKind::SlimeRibbonYellow
        | NpcKind::SlimeRibbonGreen
        | NpcKind::SlimeRibbonRed => &[20],
        // W4-d38 b7 — SpecificallyImmuneTo {31}.
        NpcKind::BunnyXmas => &[31],
        // W4-d38 b6 — null DebuffImmunitySets row.
        NpcKind::DemonEyeSpaceship => &[],
        // W4-d38 b6 — SpecificallyImmuneTo {20}.
        NpcKind::SkeletonTopHat
        | NpcKind::SkeletonAstonaut
        | NpcKind::SkeletonAlien => &[20],
        // W4-d38 b6 — SpecificallyImmuneTo {20, 24, 31, 323}.
        NpcKind::PumpkingBlade => &[20, 24, 31, 323],
        // W4-d38 b5 — SpecificallyImmuneTo {20}.
        NpcKind::SlimeMasked
        | NpcKind::HoppinJack => &[20],
        // W4-d38 b5 — SpecificallyImmuneTo {31}.
        NpcKind::BunnySlimed => &[31],
        // W4-d38 b5 — SpecificallyImmuneTo {20, 24, 31, 39, 44, 69, 70, 395, 153, 189, 203, 204, 323, 324}.
        NpcKind::Ghost => &[20, 24, 31, 39, 44, 69, 70, 395, 153, 189, 203, 204, 323, 324],
        // W4-d38 b5 — null DebuffImmunitySets row.
        NpcKind::DemonEyeOwl => &[],
        // W4-d38 b4 — SpecificallyImmuneTo {31}.
        NpcKind::BirdBlue
        | NpcKind::BirdRed
        | NpcKind::Squirrel
        | NpcKind::Mouse
        | NpcKind::Raven => &[31],
        // W4-d38 b3 — SpecificallyImmuneTo {20}.
        NpcKind::TacticalSkeleton
        | NpcKind::SkeletonCommando
        | NpcKind::AngryBonesBig
        | NpcKind::AngryBonesBigMuscle
        | NpcKind::AngryBonesBigHelmet => &[20],
        // W4-d38 b2 — SpecificallyImmuneTo {20, 31}.
        NpcKind::DiabolistWhite
        | NpcKind::GiantCursedSkull => &[20, 31],
        // W4-d38 b2 — SpecificallyImmuneTo {20}.
        NpcKind::BoneLee
        | NpcKind::SkeletonSniper => &[20],
        // W4-d38 b2 — SpecificallyImmuneTo {20, 31, 69}.
        NpcKind::Paladin => &[20, 31, 69],
        // W4-d38 b1 — SpecificallyImmuneTo {20, 31}.
        NpcKind::RaggedCaster
        | NpcKind::RaggedCasterOpenCoat
        | NpcKind::Necromancer
        | NpcKind::NecromancerArmored
        | NpcKind::DiabolistRed => &[20, 31],
        // W4-d37 — the NPCID 211-280 band's `DebuffImmunitySets` rows
        // verbatim (`NPCID.cs:6075+`'s dictionary). The `[31]` Confused row:
        // the beetles/snail/squid (`:7341`/`:7348`/`:7355`/`:7362`/`:7369`),
        // the pirate + goldfish (`:7407`/`:7414`), the blood feeder/jelly
        // (`:7491`/`:7498`), the fungo fish (`:7572`). The `[20, 31]` row:
        // the hornets (`:7421`-`:7449`), the fungi bulbs (`:7581`/`:7588`).
        // The `[20]` Poisoned row: the crawlers/wall crawlers/rainbow slime
        // (`:7456`-`:7484`/`:7512`), the rusty + blue armored bones
        // (`:7651`-`:7700`). The `[20, 31, 44, 324]` ice golem (`:7505`).
        // The `[20, 69]` ichor sticker (`:7644`). The `[20, 24, 323]` hell
        // armored bones (`:7707`-`:7728`). The null rows (no immunity): the
        // flying fish/snake (`:7380`/`:7388`), eyezor/parrot (`:7557`/
        // `:7558`), the mushroom zombies/anomura/mushi (`:7566`/`:7567`/
        // `:7575`/`:7576`). The reaper's `ImmuneToAllBuffsThatAreNotWhips`
        // (`:7559-7564`) is the documented mask-loop gap — `&[]` here.
        NpcKind::CochinealBeetle
        | NpcKind::CyanBeetle
        | NpcKind::LacBeetle
        | NpcKind::SeaSnail
        | NpcKind::Squid
        | NpcKind::Pirate
        | NpcKind::GoldfishWalker
        | NpcKind::BloodFeeder
        | NpcKind::BloodJelly
        | NpcKind::FungoFish => &[31],
        NpcKind::HornetFatty
        | NpcKind::HornetHoney
        | NpcKind::HornetLeafy
        | NpcKind::HornetSpikey
        | NpcKind::HornetStingy
        | NpcKind::FungiBulb
        | NpcKind::GiantFungiBulb => &[20, 31],
        NpcKind::JungleCreeper
        | NpcKind::JungleCreeperWall
        | NpcKind::BlackRecluseWall
        | NpcKind::BloodCrawler
        | NpcKind::BloodCrawlerWall
        | NpcKind::RainbowSlime
        | NpcKind::RustyArmoredBonesAxe
        | NpcKind::RustyArmoredBonesFlail
        | NpcKind::RustyArmoredBonesSword
        | NpcKind::RustyArmoredBonesSwordNoArmor
        | NpcKind::BlueArmoredBones
        | NpcKind::BlueArmoredBonesMace
        | NpcKind::BlueArmoredBonesNoPants
        | NpcKind::BlueArmoredBonesSword => &[20],
        NpcKind::IceGolem => &[20, 31, 44, 324],
        NpcKind::IchorSticker => &[20, 69],
        NpcKind::HellArmoredBones
        | NpcKind::HellArmoredBonesSpikeShield
        | NpcKind::HellArmoredBonesMace
        | NpcKind::HellArmoredBonesSword => &[20, 24, 323],
        NpcKind::FlyingFish
        | NpcKind::FlyingSnake
        | NpcKind::Eyezor
        | NpcKind::Parrot
        | NpcKind::Reaper
        | NpcKind::ZombieMushroom
        | NpcKind::ZombieMushroomHat
        | NpcKind::AnomuraFungus
        | NpcKind::MushiLadybug => &[],
        // W4-d42 - the NPCID 578-582 immunity rows (`NPCID.cs:9395+`).
        // The null rows + the fairies' ImmuneToAllBuffsThatAreNotWhips
        // (the mask loop is unmodelled - the documented gap; the
        // fairies' ImmuneToWhips rides `whip_immune`).
        NpcKind::WalkingAntlion
        | NpcKind::LarvaeAntlion => &[],
        NpcKind::DD2LightningBug
        | NpcKind::BartenderUnconscious => &[31],
        // W4-d42 - the NPCID 583-587 immunity rows (`NPCID.cs:9395+`).
        // The null rows + the fairies' ImmuneToAllBuffsThatAreNotWhips
        // (the mask loop is unmodelled - the documented gap; the
        // fairies' ImmuneToWhips rides `whip_immune`).
        NpcKind::ZombieMerman => &[],
        // W4-d42 - the NPCID 590-594 immunity rows (`NPCID.cs:9395+`).
        // The null rows + the fairies' ImmuneToAllBuffsThatAreNotWhips
        // (the mask loop is unmodelled - the documented gap; the
        // fairies' ImmuneToWhips rides `whip_immune`).
        NpcKind::TorchZombie
        | NpcKind::ArmedTorchZombie => &[],
        NpcKind::GoldGoldfishWalker => &[31],
        // W4-d42 - the NPCID 600-604 immunity rows (`NPCID.cs:9395+`).
        NpcKind::Seagull
        | NpcKind::Seagull2 => &[31],
        // W4-d42 - the NPCID 605-609 immunity rows (`NPCID.cs:9395+`).
        NpcKind::Grebe
        | NpcKind::Grebe2 => &[31],
        // W4-d42 - the NPCID 610-614 immunity rows (`NPCID.cs:9395+`).
        NpcKind::Rat
        | NpcKind::Owl
        | NpcKind::WaterStrider
        | NpcKind::GoldWaterStrider
        | NpcKind::ExplosiveBunny => &[31],
        // W4-d42 - the NPCID 615-619 immunity rows (`NPCID.cs:9395+`).
        NpcKind::Turtle
        | NpcKind::TurtleJungle
        | NpcKind::BloodNautilus
        | NpcKind::BloodSquid => &[31],
        // W4-d42 - the NPCID 620-624 immunity rows (`NPCID.cs:9395+`).
        NpcKind::GoblinShark
        | NpcKind::BloodEelHead
        | NpcKind::BloodEelBody
        | NpcKind::BloodEelTail
        | NpcKind::Gnome => &[31],
        // W4-d42 - the NPCID 625-629 immunity rows (`NPCID.cs:9395+`).
        NpcKind::SeaTurtle
        | NpcKind::Seahorse
        | NpcKind::GoldSeahorse
        | NpcKind::Dandelion => &[31],
        NpcKind::IceMimic => &[20, 24, 31, 44, 323, 324],
        // W4-d42 - the NPCID 630-630 immunity rows (`NPCID.cs:9395+`).
        // The null rows + the fairies' ImmuneToAllBuffsThatAreNotWhips
        // (the mask loop is unmodelled - the documented gap; the
        // fairies' ImmuneToWhips rides `whip_immune`).
        NpcKind::BloodMummy => &[],
        // W4-d35 (A): the 73/74/75/77/78 DebuffImmunitySets rows (NPCID.cs).
        NpcKind::GoblinScout => &[],
        NpcKind::Bird => &[31],
        NpcKind::Pixie => &[],
        NpcKind::ArmoredSkeleton => &[20],
        // W4-d35 (B): the 79/80/82/83/84 DebuffImmunitySets rows (NPCID.cs).
        NpcKind::DarkMummy => &[],
        NpcKind::LightMummy => &[],
        // 82: ImmuneToAllBuffsThatAreNotWhips - unmodelled mask, &[] (Poltergeist precedent).
        NpcKind::Wraith => &[],
        NpcKind::CursedHammer => &[20, 24, 31, 323],
        // W4-d35 (C): the 85/86/93/94/95 DebuffImmunitySets rows (NPCID.cs).
        NpcKind::Mimic => &[20, 31],
        NpcKind::Unicorn => &[],
        NpcKind::GiantBat => &[],
        NpcKind::Corruptor => &[31],
        // W4-d35 (D): the 96/97/98/99/100 DebuffImmunitySets rows (NPCID.cs).
        NpcKind::DiggerBody => &[31],
        NpcKind::DiggerTail => &[31],
        NpcKind::SeekerHead => &[31],
        NpcKind::SeekerBody => &[31],
        // W4-d35 (E): the 101/102/103/104/109 DebuffImmunitySets rows (NPCID.cs).
        NpcKind::Clinger => &[39, 31],
        NpcKind::AnglerFish => &[31],
        NpcKind::GreenJellyfish => &[31],
        NpcKind::Werewolf => &[],
        // W4-d35 (F): the 110/112/116/120/121 DebuffImmunitySets rows (NPCID.cs).
        NpcKind::SkeletonArcher => &[20],
        NpcKind::VileSpit => &[31],
        NpcKind::TheHungryII => &[31],
        NpcKind::ChaosElemental => &[20],
        // W4-d35 (G): the 122/137/140 DebuffImmunitySets rows (NPCID.cs).
        NpcKind::Gastropod => &[20, 31],
        NpcKind::IlluminantBat => &[20],
        NpcKind::PossessedArmor => &[20, 24, 323],
        NpcKind::Slimer => &[20],
        NpcKind::Clown => &[31],
        NpcKind::SeekerTail => &[31],
        NpcKind::DiggerHead => &[31],
        NpcKind::EnchantedSword => &[20, 24, 31, 323],
        NpcKind::Mummy => &[],
        // W4-c3: the WaterBoltMimic carries `SpecificallyImmuneTo =
        // { 20, 31 }` (`NPCID.cs:10066-10072`) - Poisoned + On Fire!;
        // the 34/289 rows are d34/d38's above (E8 keep-both).
        NpcKind::WaterBoltMimic => &[20, 31],
        // W4-c4 (the style-44 swimmers): none of 224/509/581/587 has a
        // `DebuffImmunitySets` row (`NPCID.cs:6075+`'s dictionary carries
        // no such keys) — the null-row else-branch, no immunity.
        NpcKind::FlyingFish
        | NpcKind::GiantFlyingAntlion
        | NpcKind::FlyingAntlion
        | NpcKind::EyeballFlyingFish => &[],
        // W4-c7: none of 355/358/654/677 has a `DebuffImmunitySets` row
        // (`NPCID.cs:6075+`'s dictionary carries no keys for them) — the
        // C# else-branch's no-immunity (`NPC.cs:17826-17832`).
        NpcKind::Firefly | NpcKind::LightningBug | NpcKind::Lavafly | NpcKind::Shimmerfly => &[],
        // W4-c8 — the crawling-critter family's `SpecificallyImmuneTo`
        // rows: all nine carry `{ 31 }` (Confused) — `NPCID.cs:8171-8175`
        // (357), `:8290-8294` (374), `:8297-8301` (375), `:8654-8658`
        // (448), `:8858-8862` (484), `:8865-8869` (485), `:8872-8876`
        // (486), `:8879-8883` (487), `:9551-9555` (606).
        // E8 merge: the GoldWorm/EnchantedNightcrawler/Grubby/Sluggy/
        // Buggy rows are d40's above (the dup drop - same {31} values).
        NpcKind::Worm
        | NpcKind::TruffleWorm
        | NpcKind::TruffleWormDigger
        | NpcKind::Maggot => &[31],
        // The snail family + its shimmer target (W4-c9): none of
        // 359/360/655/677 has a `DebuffImmunitySets` row (`NPCID.cs:6075+`
        // carries no entry for them) — the null-row else-branch.
        // E8 merge: Shimmerfly's null row is c7's above; the 5 gold
        // {31} rows are d40's above (the dup drops).
        NpcKind::Snail | NpcKind::GlowingSnail | NpcKind::MagmaSnail => &[],
        // W4-c10 merge FIX: the C# NPCID.cs DebuffImmunitySets HAS {31}
        // rows for 362-365 (re-read at this splice - the c10 null claim
        // was wrong); the Seagull/Grebe {31} rows are d42's above.
        NpcKind::Duck
        | NpcKind::Duck2
        | NpcKind::DuckWhite
        | NpcKind::DuckWhite2 => &[31],
        // E8 merge: the c10 12-critter {31} tail dropped (d40/c8's rows above).
        // The man-eater family (W4-d23): the `DebuffImmunitySets` rows —
        // 43 `{20,31}` (`NPCID.cs:6349-6355`), 56 `{31}` (`:6410-6416`),
        // 101 `{39,31}` (`:6692-6698`), 175 `{20,31}` (`:7139-7145`),
        // 259/260 `{20,31}` (`:7577-7590`). The 20⇒30+375 derived rule
        // (`NPC.cs:17828-17833`) applies in `for_kind`.
        NpcKind::ManEater | NpcKind::AngryTrapper | NpcKind::FungiBulb | NpcKind::GiantFungiBulb => &[20, 31],
        NpcKind::Snatcher => &[31],
        NpcKind::Clinger => &[39, 31],
        // The aiStyle-9 spit/flare family (W4-d23): 112/666 `{31}`
        // (`NPCID.cs:6758-6764`/`:6765-6771`), 516 `{31}` (`:9005-9011`).
        NpcKind::VileSpit | NpcKind::VileSpitEoW | NpcKind::SolarFlare => &[31],
        // The aiStyle-14 flyer family (W4-d23): the `DebuffImmunitySets`
        // rows — 48 `{31}` (`NPCID.cs:6378-6384`), 62/156
        // `{24,31,153,323}` (`:6452-6458`/`:7042-7048`), 121/137 `{20}`
        // (`:6827-6833`/`:6927-6933`), 150 `{44,324}` (`:7018-7024`),
        // 151 `{24,323}` (`:7025-7031`), 660 `{20,31}` (`:9917-9923`);
        // 93/152/158/159/226 are null rows (`:6642`/`:7032`/`:7056-7057`/
        // `:7388`), 634 null (`:9741`). The 20⇒30+375 derived rule
        // (`NPC.cs:17828-17833`) applies in `for_kind`.
        NpcKind::Harpy => &[31],
        NpcKind::Demon | NpcKind::RedDevil => &[24, 31, 153, 323],
        NpcKind::Slimer | NpcKind::IlluminantBat => &[20],
        NpcKind::IceBat => &[44, 324],
        NpcKind::Lavabat => &[24, 323],
        NpcKind::QueenSlimeMinionPurple => &[20, 31],
        NpcKind::GiantBat
        | NpcKind::GiantFlyingFox
        | NpcKind::VampireBat
        | NpcKind::Vampire
        | NpcKind::FlyingSnake
        | NpcKind::SporeBat => &[],
        // W4-d25 STYLE-26 wiring: 86's `DebuffImmunitySets` row is null
        // (`NPCID.cs:6599`), 155's is null (`:7041`) — no immunity.
        // The Unicorn row mirrors d35 (C). 546 is SpecificallyImmuneTo[31]
        // (Confused) (`NPCID.cs:9169`).
        NpcKind::Unicorn | NpcKind::Wolf => &[],
        NpcKind::Tumbleweed => &[31],
        // W4-d120 — all five carry `SpecificallyImmuneTo {31}` (the
        // Confused debuff; `NPCID.cs`'s DebuffImmunitySets rows for
        // 142/160/178/209/579).
        NpcKind::SantaClaus
        | NpcKind::Truffle
        | NpcKind::Steampunker
        | NpcKind::Cyborg
        // W4-d125: 636 (`NPCID.cs:9749-9755`) and 661 (`:9923-9929`)
        // both carry `SpecificallyImmuneTo = { 31 }` — Confused.
        NpcKind::EmpressOfLight | NpcKind::PrismaticLacewing => &[31],
        // W4-g5 — the two casters' `SpecificallyImmuneTo` rows: {20, 31}
        // (`NPCID.cs:7119-7125` the RuneWizard, `:10060-10066` the
        // LibrarianSkeleton).
        NpcKind::RuneWizard | NpcKind::LibrarianSkeleton => &[20, 31],
        // W4-g8 lane 1 batch 1 — the `Sets.DebuffImmunitySets` rows
        // (`NPCID.cs:6075+`): 61/63/64/69 `[31]`, 70 `[20, 24, 31, 39, 70,
        // 395, 323]`, 83/84 `[20, 24, 31, 323]`, 75/85 `[20, 31]`. The
        // Wraith 82's row is `ImmuneToAllBuffsThatAreNotWhips = true` — the
        // mask loop is unmodelled, so it rides `&[]` (the Force Bubble /
        // saucer family's documented gap above).
        NpcKind::SpikeBall => &[20, 24, 31, 39, 70, 395, 323],
        // W4-g8 lane 1 batch 2 — the `Sets.DebuffImmunitySets` rows: 103/
        // 174/177/195 `[31]`, 122/182 `[20, 31]`, 164/165 `[20]`, 169
        // `[20, 31, 44, 324]`, 179 `[20, 24, 31, 323]`; 196 has no row
        // (the null-row else-branch, no immunity).
        NpcKind::WallCreeper | NpcKind::WallCreeperWall => &[20],
        NpcKind::IceElemental => &[20, 31, 44, 324],
        NpcKind::CrimsonAxe => &[20, 24, 31, 323],
        NpcKind::Nymph => &[],
        // W4-g8 lane 1 batch 3 — the `Sets.DebuffImmunitySets` rows: 34
        // `[20, 31]`, 62 `[24, 31, 153, 323]`, 121/137 `[20]`, 150
        // `[44, 324]`, 151 `[24, 323]`; 93/152/158/226 have no row (the
        // null-row else-branch, no immunity).
        NpcKind::GiantBat | NpcKind::GiantFlyingFox | NpcKind::VampireBat | NpcKind::FlyingSnake => &[],
        // W4-g8 lane 1 batch 4 — the `Sets.DebuffImmunitySets` rows: 48/221
        // `[31]`, 156 `[24, 31, 153, 323]`.
        // W4-g8 machine round 3 — 159 has a null row (`NPCID.cs:7057`
        // `{ 159, null }`, the else-branch: no immunity).
        NpcKind::Vampire => &[],
        // The dragonflies (W4-d33): all seven share the
        // `SpecificallyImmuneTo = [31]` (Confused) rows
        // (`NPCID.cs:9473-9521` — 595 through 601 each carry one; the
        // c8 crawlers' same-band precedent).
        NpcKind::BlackDragonfly
        | NpcKind::BlueDragonfly
        | NpcKind::GreenDragonfly
        | NpcKind::OrangeDragonfly
        | NpcKind::RedDragonfly
        | NpcKind::YellowDragonfly
        | NpcKind::GoldDragonfly => &[31],
        // The fairy critters (W4-d33 wiring): `ImmuneToAllBuffsThatAre
        // NotWhips = true` + `ImmuneToWhips = true` (`NPCID.cs:9410-
        // 9433`, 583/584/585 each) — the mask loop is unmodelled, so
        // they ride `&[]` (the ForceBubble documented gap); the whip
        // tag gate reads `whip_immune` below. The windy balloon 594:
        // `SpecificallyImmuneTo = [31]` (`NPCID.cs:9466-9472`).
        // d42 batch-2/3's rows, constants-copy (keep-one d37c-E3).
        NpcKind::FairyCritterPink
        | NpcKind::FairyCritterGreen
        | NpcKind::FairyCritterBlue => &[],
        NpcKind::WindyBalloon => &[31],
        // W4-e21 — the bunny pair (46/540) has NO `DebuffImmunitySets` row
        // (`NPCID.cs:6075+`'s dictionary carries no 46/540 keys) — the C#
        // else-branch's no-immunity (`NPC.cs:17826-17832`).
        NpcKind::Bunny | NpcKind::PartyBunny => &[],
        // W4-f42 — the sandstorm debris + desert family: none has a
        // `DebuffImmunitySets` row (`NPCID.cs:6075+`) — the no-immunity
        // else-branch.
        NpcKind::Tumbleweed
        | NpcKind::Vulture
        | NpcKind::Antlion
        | NpcKind::Mummy
        | NpcKind::DarkMummy
        | NpcKind::BloodMummy
        | NpcKind::LightMummy
        | NpcKind::WalkingAntlion
        | NpcKind::FlyingAntlion => &[],
        // W4-d9 wiring (d40 rows): the dragon family's
        // `ImmuneToAllBuffsThatAreNotWhips` rows carry NO
        // `SpecificallyImmuneTo` list (the &[] mask gap, `NPCID.cs`'s
        // DebuffImmunitySets) — the null-row else-branch, no immunity.
        | NpcKind::CultistDragonBody2
        | NpcKind::CultistDragonBody3
        | NpcKind::CultistDragonTail => &[],
        // W4-d9 wiring: the 521 squidhead's 14-id `SpecificallyImmuneTo`
        // row (`NPCID.cs:9040-9048`).
        NpcKind::AncientCultistSquidhead => &[20, 24, 31, 39, 44, 69, 70, 395, 153, 189, 203, 204, 323, 324],
        // W4-d39: the Butterfly's `{ 356, [31] }` row (`NPCID.cs:8164`
        // — the single Confused immunity, the catchable-critter set);
        // the scorpion pair `:8234`/`:8241`, the grasshopper `:8311`
        // — all the same `[31]` row.
        NpcKind::Butterfly
        | NpcKind::ScorpionBlack
        | NpcKind::Scorpion
        | NpcKind::Grasshopper => &[31],
        // W4-d39 batch 3: the teeth bomb (`NPCID.cs:8318`) + the archer
        // pair (`:8325`/`:8332`) — the same single `[31]` Confused row.
        NpcKind::ChatteringTeethBomb
        | NpcKind::CultistArcherBlue
        | NpcKind::CultistArcherWhite => &[31],
        // W4-d41: the DD2 SetDefaults census — the crystal 548
        // (`NPCID.cs:9182-9188`), the Tavernkeep 550 (`:9197-9203`) and
        // the etherian goblin 552 (`:9211-9217`) each carry the Confused
        // (31) row; Betsy 551 carries `{24, 31, 323}` (`:9204-9210`).
        NpcKind::DD2EterniaCrystal | NpcKind::DD2Bartender | NpcKind::DD2GoblinT1 => &[31],
        NpcKind::DD2Betsy => &[24, 31, 323],
        // The lane portal 549 (`NPCID.cs:9189-9196`) is an
        // `ImmuneToAllBuffsThatAreNotWhips` + `ImmuneToWhips` row — the
        // NotWhips mask is the documented unmodelled gap (`&[]` above),
        // the whip half rides `whip_immune`.
        NpcKind::DD2LanePortal => &[],
        // W4-d41 — the cavern crawdads/shellies: `{ 494..497,
        // SpecificallyImmuneTo: [31] }` (`NPCID.cs:8916-8943`) — the
        // Confused debuff, all four.
        NpcKind::Crawdad | NpcKind::Crawdad2 | NpcKind::GiantShelly | NpcKind::GiantShelly2 => &[31],
        // W4-d41 — the salamanders: `{ 498..506, null }`
        // (`NPCID.cs:8944-8952`) — no immunity rows, the C# else-branch.
        NpcKind::Salamander
        | NpcKind::Salamander2
        | NpcKind::Salamander3
        | NpcKind::Salamander4
        | NpcKind::Salamander5
        | NpcKind::Salamander6
        | NpcKind::Salamander7
        | NpcKind::Salamander8
        | NpcKind::Salamander9 => &[],
        // W4-d41 — the antlions: `{ 508, null }` / `{ 509, null }`
        // (`NPCID.cs:8960-8961`) — no immunity rows.
        NpcKind::GiantWalkingAntlion | NpcKind::GiantFlyingAntlion => &[],
        // W4-d41 — the Dune Splicer chain: `{ 510..512,
        // SpecificallyImmuneTo: [31] }` (`NPCID.cs:8962-8983`) — the
        // Confused debuff, all three.
        NpcKind::DuneSplicerHead
        | NpcKind::DuneSplicerBody
        | NpcKind::DuneSplicerTail => &[31],
        // W4-d41 — the Tomb Crawler chain: `{ 513..515,
        // SpecificallyImmuneTo: [31] }` (`NPCID.cs:8984-9003`); the
        // solar fragments: `{ 516, [31] }` / `{ 519, [31] }`
        // (`:9004-9010`/`:9025-9031`).
        NpcKind::TombCrawlerHead
        | NpcKind::TombCrawlerBody
        | NpcKind::TombCrawlerTail
        | NpcKind::SolarFlare
        | NpcKind::SolarGoop => &[31],
        // W4-d41 — the ancient orbs: `{ 522, [31] }` / `{ 523, [31] }`
        // (`NPCID.cs:9050-9063`); the desert ghouls: `{ 524..527, null }`
        // (`:9064-9067`) — no immunity rows.
        NpcKind::AncientLight | NpcKind::AncientDoom => &[31],
        NpcKind::DesertGhoul
        | NpcKind::DesertGhoulCorruption
        | NpcKind::DesertGhoulCrimson => &[],
        // W4-d41 — the hallowed ghoul + the lamias: `{ 527..529, null }`
        // (`NPCID.cs:9064-9069`); the sand poachers: `{ 530, [20] }` /
        // `{ 531, [20] }` (`:9070-9083`).
        NpcKind::DesertGhoulHallow
        | NpcKind::DesertLamiaLight
        | NpcKind::DesertLamiaDark => &[],
        NpcKind::DesertScorpionWalk | NpcKind::DesertScorpionWall => &[20],
        // W4-d41 — the basilisk: `{ 532, null }` (`NPCID.cs:9084`); the
        // tortured soul: `{ 534, null }` (`:9096`); the spiked slime:
        // `{ 535, [20] }` (`:9097-9103`); the bride: `{ 536, null }`
        // (`:9104`); the sand slime: `{ 537, [20] }` (`:9105-9111`).
        NpcKind::DesertBeast | NpcKind::DemonTaxCollector | NpcKind::TheBride => &[],
        NpcKind::SlimeSpiked | NpcKind::SandSlime => &[20],
        // W4-d41 — the critters: `{ 538..540, [31] }`
        // (`NPCID.cs:9112-9132`); the tumbler: `{ 546, [31] }`
        // (`:9168-9174`).
        // The luck critters (W4-e26, E8 keep-one): none of 361/604/605
        // has a `DebuffImmunitySets` row (`NPCID.cs:6075+`'s dictionary
        // carries no entry for them) — the null-row else-branch
        // (`NPC.cs:17822-17828`), no immunity. The Duck dropped to c10.
        NpcKind::Frog | NpcKind::LadyBug | NpcKind::GoldLadyBug => &[],
        // W4-g10 lane 3 — the town pets: `NPCID.cs`'s DebuffImmunitySets has
        // no row for 637/638/656/670/678-684 — the null-row else-branch, no
        // immunity (`NPC.cs:17826-17832`), same as the town residents.
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
        | NpcKind::TownSlimeCopper => &[],
        // Doc-sweep lane 36 — 676 has no `DebuffImmunitySets` row
        // (`NPCID.cs:6075+` carries no 676 entry) — the C# else-branch's
        // no-immunity (`NPC.cs:17826-17832`).
        NpcKind::ShimmerSlime => &[],
        // W4-f18 first-20 port — the critters' `SpecificallyImmuneTo`
        // rows: 355-359 each immune to the Confused debuff (31)
        // (`NPCID.cs:8156-8190`).
        NpcKind::Firefly
        | NpcKind::Butterfly
        | NpcKind::Worm
        | NpcKind::LightningBug
        // W4-f18 batch 2 — the critters' `SpecificallyImmuneTo` rows:
        // 360-364 each immune to the Confused debuff (31)
        // (`NPCID.cs:8191-8232`).
        NpcKind::GlowingSnail
        | NpcKind::Frog
        | NpcKind::Duck
        // W4-f18 batch 3 — the critters' `SpecificallyImmuneTo` rows:
        // 365-367/374/375 each immune to the Confused debuff (31)
        // (`NPCID.cs:8233-8246` / the 374/375 rows).
        NpcKind::DuckWhite2
        // W4-f18 batch 4 — 377-380 each immune to the Confused debuff (31)
        // (`NPCID.cs`'s 377/378/379/380 rows); the ArmedZombie 430 has NO
        // `DebuffImmunitySets` row (the null-row else-branch, no immunity).
        NpcKind::Grasshopper
        | NpcKind::ChatteringTeethBomb
        NpcKind::ArmedZombie => &[],
        // The Shimmerfly 677 (W4-c9's kind, rowed by W4-g62): the
        // `DebuffImmunitySets` row `{ 677, SpecificallyImmuneTo: [31] }`
        // (`NPCID.cs:10015-10021`) — the poisoned-debuff immunity.
        // W4-d42 - the NPCID 620-624 immunity rows (`NPCID.cs:9395+`).
            NpcKind::PalworldCattivaDistressed => &[31],
            NpcKind::PalworldFoxsparksDistressed => &[31],
            NpcKind::OwlMimic => &[31],
            NpcKind::StatueMimic => &[],
            NpcKind::MossZombie => &[],
            NpcKind::Orca => &[31],
            NpcKind::BoundTownSlimeOld => &[31],
            NpcKind::BoundTownSlimePurple => &[],
            NpcKind::BoundTownSlimeYellow => &[],
            NpcKind::Pufferfish => &[31],
            NpcKind::YellowCockatiel => &[31],
            NpcKind::GrayCockatiel => &[31],
            NpcKind::Stinkbug => &[31],
            NpcKind::ScarletMacaw => &[31],
            NpcKind::BlueMacaw => &[31],
            NpcKind::Toucan => &[31],
            NpcKind::PirateGhost => &[],
            NpcKind::TorchGod => &[],
            NpcKind::ChaosBallTim => &[31],
            NpcKind::VileSpitEaterOfWorlds => &[31],
            NpcKind::GoldenSlime => &[],
            NpcKind::QueenSlimeBoss => &[20, 31],
            NpcKind::QueenSlimeMinionBlue => &[20, 31],
            NpcKind::QueenSlimeMinionPink => &[20, 31],
            NpcKind::EmpressButterfly => &[31],
            NpcKind::GemBunnyAmber => &[31],
            NpcKind::HellButterfly => &[31],
            NpcKind::GemBunnyTopaz => &[31],
            NpcKind::GemBunnySapphire => &[31],
            NpcKind::GemBunnyEmerald => &[31],
            NpcKind::GemBunnyRuby => &[31],
            NpcKind::GemBunnyDiamond => &[31],
            NpcKind::GemSquirrelEmerald => &[31],
            NpcKind::GemSquirrelRuby => &[31],
            NpcKind::GemSquirrelDiamond => &[31],
            NpcKind::GemSquirrelAmber => &[31],
            NpcKind::GemBunnyAmethyst => &[31],
            NpcKind::GemSquirrelAmethyst => &[31],
            NpcKind::GemSquirrelTopaz => &[31],
            NpcKind::GemSquirrelSapphire => &[31],
            NpcKind::RockGolem => &[20, 24, 31, 323],
            NpcKind::MaggotZombie => &[],
            NpcKind::SporeBat => &[],
            NpcKind::SporeSkeleton => &[20],
            NpcKind::HallowBoss => &[31],
    }
}

/// `NPCID.Sets.ShimmerImmunity` (`NPCID.cs:10467`) — the C# `NPC.type`
/// ids whose `SetDefaults` writes `buffImmune[353] = true` (`NPC.cs:17838`).
/// The set verbatim, compressed to ranges where the C# lists contiguous
/// runs (245-249, 381-421, 423-429, 516-523, 551-578, 657-660, 678-683).
/// Keyed on `cs_id` — the snail trio 359/360/655 is NOT in the set (their
/// aiStyle-67 probe owns the shimmer transform, `NPC.cs:34350-34360`),
/// the Shimmerfly 677 IS (the transform target never re-shimmers).
fn shimmer_immune(cs_id: u16) -> bool {
    matches!(
        cs_id,
        637 | 638
            | 656
            | 670
            | 684
            | 678..=683
            | 356
            | 669
            | 676
            | 244
            | 677
            | 594
            | 667
            | 662
            | 5
            | 115
            | 116
            | 139
            | 245..=249
            | 344
            | 325
            | 50
            | 535
            | 657..=660
            | 668
            | 25
            | 30
            | 33
            | 70
            | 72
            | 665
            | 666
            | 112
            | 516..=523
            | 381..=421
            | 423..=429
            | 548
            | 549
            | 551..=578
    )
}

/// `NPCDebuffImmunityData.ImmuneToWhips` — the 11 `NPCID.cs` rows
/// (`:6499`/`:6528`/`:8378`/`:8601`/`:8899`/`:9194`/`:9415`/`:9423`/`:9431`/
/// `:10034`/`:10042`): DungeonGuardian 68, BlazingWheel 72, MartianSaucer 392,
/// CultistBossClone 440, FlyingDutchman 491, DD2LanePortal 549, the fairy
/// critters 583/584/585, the bound town slimes 686/687. The saucer, the
/// pirate ship and the Dungeon Guardian (W4-d34) are on the ported roster —
/// the whip tag gate
/// (`WhipTagEffect.CanApplyTagToNPC`) reads this.
    // MERGE-SIDE (367 s1): the union keeps d34's DungeonGuardian row and
    // W4-g4/d22's BlazingWheel + DD2LanePortal rows; d42's fairy critters
    // 583-585 drop with the d33-owned dups (d33 re-adds their
    // `ImmuneToWhips` rows at the small-lanes splice, `NPCID.cs:9420+`).
/// pirate ship and (W4-d33 wiring) the three fairy critters are on the
/// ported roster — the whip tag gate
/// (`WhipTagEffect.CanApplyTagToNPC`) reads this.
/// pirate ship and the lane portal (W4-d41) are on the ported roster — the
/// whip tag gate (`WhipTagEffect.CanApplyTagToNPC`) reads this.
pub fn whip_immune(kind: NpcKind) -> bool {
    // W4-d43: the bound town slimes (686/687) carry `ImmuneToWhips = true`
    // (`NPCID.cs:10029-10043`).
    matches!(
        kind,
        NpcKind::MartianSaucer
            | NpcKind::FlyingDutchman
            | NpcKind::DungeonGuardian
            | NpcKind::BlazingWheel
            | NpcKind::DD2LanePortal
            | NpcKind::FairyCritterPink
            | NpcKind::FairyCritterGreen
            | NpcKind::FairyCritterBlue
            | NpcKind::DD2LanePortal
            // W4-d41 — the BlazingWheel 72's `ImmuneToWhips` row
            // (`NPCID.cs:6524-6529`) — the wheel is on the ported roster.
            | NpcKind::BlazingWheel
            | NpcKind::BoundTownSlimePurple
            | NpcKind::BoundTownSlimeYellow
    )
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
        // `NPC.cs:17838` — `buffImmune[353] = NPCID.Sets.ShimmerImmunity
        // [this.type]`: the unconditional post-dictionary write, keyed on
        // the C# `NPC.type` (the roster's `cs_id`), NOT the debuff rows
        // above. The set (`NPCID.cs:10467`) is ported above as
        // `shimmer_immune`. W4-c9: without the write the Shimmerfly 677
        // (a set member) accepts buff 353 and re-shimmers; the snail
        // trio (non-members) stays susceptible — the aiStyle-67 probe
        // owns their transform (`NPC.cs:34350-34360`).
        buffs.buff_immune[353] = shimmer_immune(kind.cs_id());
        // [type]` (W4-d6 residual 9: the ShimmerGun's buff-353 contact
        // reads it through `AddBuff`'s immune guard). The verbatim set
        // (`NPCID.cs:10467`, `CreateBoolSet` default-false + the listed
        // ids true), keyed on the C# type via `cs_id`.
        buffs.buff_immune[353] = SHIMMER_IMMUNITY.contains(&kind.cs_id());
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

    /// S1-P-i18 — whether the buff is active (the `FindBuffIndex` gate: the
    /// immune table hides it, then a slot with time left). The special-bullet
    /// debuff witnesses read it.
    pub fn has_buff(&self, buff_type: i32) -> bool {
        self.find_buff_index(buff_type).is_some()
    }

    /// S1-P-i18 — the buff's remaining ticks (0 when absent). The nano's
    /// Confused-roll witness pins the 180/60 branch through it.
    pub fn buff_time_of(&self, buff_type: i32) -> i32 {
        self.find_buff_index(buff_type)
            .map_or(0, |i| self.buff_time[i])
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
    /// (`:90378`) have no port systems yet. Returns the tick's flags — the
    /// caller stores `shimmering` (buff 353) for the transparency ramp
    /// (`NPC.cs:91514`, W4-f47).
    pub fn update_buffs_tick(&mut self, hp: &mut i32, max_hp: i32) -> NpcBuffFlags {
        let flags = self.set_flags_tick();
        self.clear_expired_buffs();
        self.apply_dots(&flags, hp, max_hp);
        flags
    }

    /// `NPC.TryAddingRepeatedBuff` (`NPC.cs:93303-93313`): not immune, and
    /// (absent OR `buffTime <= low_threshold`) → `AddBuff(id, time)`. The
    /// shimmer contact calls it with (353, 100, default 10) (`:93073`).
    pub fn try_adding_repeated_buff(&mut self, buff_id: i32, time: i32, low_threshold: i32) {
        if self.buff_immune[buff_id as usize] {
            return;
        }
        match self.find_buff_index(buff_id) {
            None => self.add_buff(buff_id, time),
            Some(i) if self.buff_time[i] <= low_threshold => self.add_buff(buff_id, time),
            Some(_) => {}
        }
    }

    /// The `FindBuffIndex(353)` → `DelBuff(num)` pair the town shimmer arm
    /// runs (`NPC.cs:91586-91590`).
    pub fn remove_buff(&mut self, buff_type: i32) {
        if let Some(i) = self.find_buff_index(buff_type) {
            self.del_buff(i);
        }
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
                353 => {
                    // `NPC.cs:92133-92143`: immune → DelBuff(i) MID-scan
                    // (the shift-left skips the shifted-in slot this tick,
                    // exactly like the C# — the index loop re-reads the
                    // arrays each pass); else the shimmering flag arms the
                    // transparency ramp (`:91514`).
                    if self.buff_immune[353] {
                        self.del_buff(i);
                    } else {
                        flags.shimmering = true;
                    }
                }
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

    /// W4-d6 residual 9 — `NPC.cs:17838`: `buffImmune[353] =
    /// NPCID.Sets.ShimmerImmunity[type]` (`NPCID.cs:10467`). The ShimmerGun's
    /// NPC contact (`Projectile.cs:29039` AddBuff(353, 100)) rides the
    /// immune guard: King Slime (50) and the whole martian family refuse,
    /// the plain slime/zombie accept. WRITTEN NOT RUN (code-only).
    #[test]
    fn w4_d6_the_shimmer_immunity_set_gates_buff_353() {
        // The listed ids: King Slime 50, the ForceBubble 384, the walker
        // 520, the Hungry 115 — all in the :10467 set.
        for kind in [
            NpcKind::KingSlime,
            NpcKind::ForceBubble,
            NpcKind::MartianWalker,
            NpcKind::Hungry,
        ] {
            let mut b = NpcBuffs::for_kind(kind);
            assert!(b.buff_immune[353], "{kind:?} in the ShimmerImmunity set");
            b.add_buff(353, 100);
            assert_eq!(b.find_buff_index(353), None, "{kind:?} AddBuff refuses");
        }
        // The unlisted ids: the plain slime (1) and the zombie (3).
        for kind in [NpcKind::Slime, NpcKind::Zombie] {
            let mut b = NpcBuffs::for_kind(kind);
            assert!(!b.buff_immune[353], "{kind:?} outside the set");
            b.add_buff(353, 100);
            assert!(b.has_buff(353), "{kind:?} AddBuff lands");
        }
        // The set is EXACTLY the :10467 list — the 422 gap stays false
        // (the list jumps 421 → 423) and 550 is absent (549 → 551).
        assert!(!SHIMMER_IMMUNITY.contains(&422));
        assert!(!SHIMMER_IMMUNITY.contains(&550));
        assert!(SHIMMER_IMMUNITY.contains(&421) && SHIMMER_IMMUNITY.contains(&423));
        assert_eq!(SHIMMER_IMMUNITY.len(), 131, "the :10467 list length");
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

    /// W4-g4 P9 (the W4-d22 reconcile fold) — the fighter-roster
    /// immunity census fix (`NPCID.Sets.DebuffImmunitySets`,
    /// `NPCID.cs:6075+`): the old table folded whole families to `&[]`
    /// where the C# carries rows. The Confused (31) half is the fighter
    /// lane's: every kind below that is [31]-carrying REJECTS the
    /// buff-31 add, so the P3/P6 confused arms never fire for it — the
    /// null-row kinds keep the flip. The [20] rows derive 30/375
    /// (`NPC.cs:17828-17833`). WRITTEN NOT RUN (code-only mode).
    #[test]
    fn w4_g4_the_fighter_roster_immunity_rows_match_the_csharp_sets() {
        // The Confused-immune fighters: the add is rejected outright.
        for kind in [
            NpcKind::GoblinSummoner,      // 471 [31, 153] (:8767-8771)
            NpcKind::VortexHeadcrab,      // 428 [31] (:8555-8559)
            NpcKind::NebulaFloater,       // 420 [31] (:8529-8533)
            NpcKind::BrainSuckler,        // 421 [31] (:8536-8540)
            NpcKind::StardustCellBig,     // 405 [31] (:8466-8470)
            NpcKind::StardustCellSmall,   // 406 [31] (:8473-8477)
            NpcKind::StardustJellyfishBig, // 407 [31] (:8480-8484)
            NpcKind::StardustWormHead,    // 402 [31] (:8445-8449)
            NpcKind::SolarSpearman,       // 518 [31] (:9019-9023)
            NpcKind::SolarCrawltipedeHead, // 412 [31] (:8497-8501)
            NpcKind::SolarCorite,         // 418 [20, 24, 31, 323] (:8521)
            NpcKind::PirateCaptain,       // 216 [31] (:7331-7336)
            NpcKind::DD2GoblinT1,         // 552 [31] (:9212-9216)
            NpcKind::DD2SkeletonT1,       // 566 [20, 31] (:9310-9314)
            NpcKind::DD2Betsy,            // 551 [24, 31, 323] (:9205)
            NpcKind::DD2Bartender,        // 550 [31] (:9198-9202)
            NpcKind::DD2EterniaCrystal,   // 548 [31] (:9183-9187)
            NpcKind::DD2OgreT2,           // 576 [31] (:9380-9384)
            NpcKind::DD2KoboldFlyerT2,    // 574 [31] (:9366-9370)
        ] {
            let mut b = NpcBuffs::for_kind(kind);
            assert!(b.buff_immune[31], "{kind:?} Confused-immune");
            b.add_buff(31, 600);
            assert!(b.find_buff_index(31).is_none(), "{kind:?} rejects the add");
        }
        // The null-row fighters: Confused LANDS (the P3/P6 arms fire).
        for kind in [
            NpcKind::Zombie,              // 3 null (:6086)
            NpcKind::GoblinArcher,        // 111 null (:6756)
            NpcKind::VortexRifleman,      // 425 null (:8551)
            NpcKind::VortexSoldier,       // 429 null (:8561)
            NpcKind::MartianEngineer,     // 386 null (:8355)
            NpcKind::GigaZapper,          // 389 null (:8370)
            NpcKind::RayGunner,           // 382 null (:8345)
            NpcKind::Predictor,           // 424 null (:8550)
            NpcKind::StardustSoldier,     // 411 null (:8495)
            NpcKind::SolarDrakomire,      // 415 null (:8517)
            NpcKind::PirateDeckhand,      // 212 null (:7326)
            NpcKind::Scarecrow1,          // 305 null (:7934)
            NpcKind::Yeti,                // 343 [44, 324] — no 31 (:8091)
            NpcKind::Krampus,             // 351 [44, 324] — no 31 (:8129)
        ] {
            let mut b = NpcBuffs::for_kind(kind);
            assert!(!b.buff_immune[31], "{kind:?} Confused can land");
            b.add_buff(31, 600);
            assert!(b.find_buff_index(31).is_some(), "{kind:?} takes the add");
        }
        // The [20] rows and their derived 30/375 chain.
        for kind in [
            NpcKind::Skeleton,            // 21 [20] (:6207-6212)
            NpcKind::UndeadMiner,         // 44 [20] (:6357-6362)
            NpcKind::SnowmanGangsta,      // 143 [20, 44, 324] (:6970)
            NpcKind::SnowBalla,           // 144 [20, 44, 324] (:6977)
            NpcKind::MysterySnowman,      // 145 [20, 44, 324] (:6984)
            NpcKind::DD2SkeletonT3,       // 567 [20, 31] (:9317)
        ] {
            let b = NpcBuffs::for_kind(kind);
            assert!(b.buff_immune[20], "{kind:?} Poisoned-immune");
            assert!(b.buff_immune[30] && b.buff_immune[375], "{kind:?} derived");
        }
        // The whip rows: the wheel 72 + the lane portal 549 join the
        // saucer 392 + the Dutchman 491 (`NPCID.cs:6528`/`:9194`).
        for kind in [
            NpcKind::MartianSaucer,
            NpcKind::FlyingDutchman,
            NpcKind::BlazingWheel,
            NpcKind::DD2LanePortal,
        ] {
            assert!(whip_immune(kind), "{kind:?} ImmuneToWhips");
        }
        assert!(!whip_immune(NpcKind::Zombie));
        assert!(!whip_immune(NpcKind::DD2GoblinT1));
    }

    #[test]
    fn w4_c9_shimmer_immunity_write_keys_the_cs_type() {
        // NPC.cs:17838 — `buffImmune[353] = ShimmerImmunity[type]` runs
        // AFTER the debuff dictionary for EVERY kind. NPCID.cs:10467:
        // the snail trio 359/360/655 is NOT in the set (the aiStyle-67
        // probe owns their transform); the Shimmerfly 677 IS — the
        // transform target must reject buff 353 so it never re-shimmers
        // (the f47 transparency ramp reads `buff_immune[353]`).
        let snail = NpcBuffs::for_kind(NpcKind::Snail);
        let glowing = NpcBuffs::for_kind(NpcKind::GlowingSnail);
        let magma = NpcBuffs::for_kind(NpcKind::MagmaSnail);
        assert!(!snail.buff_immune[353]);
        assert!(!glowing.buff_immune[353]);
        assert!(!magma.buff_immune[353]);
        let mut fly = NpcBuffs::for_kind(NpcKind::Shimmerfly);
        assert!(fly.buff_immune[353], "NPCID.cs:10467 lists 677");
        fly.add_buff(353, 100);
        assert!(occupied(&fly).is_empty(), "the immune guard rejects 353");
        // Set members on the landed roster: BlazingWheel 72, ForceBubble
        // 384 (NPCID.cs:10821/:11445), the butterfly-free zombie 3 is not.
        assert!(NpcBuffs::for_kind(NpcKind::BlazingWheel).buff_immune[353]);
        assert!(NpcBuffs::for_kind(NpcKind::ForceBubble).buff_immune[353]);
        assert!(!NpcBuffs::for_kind(NpcKind::Zombie).buff_immune[353]);
        // The full-set compare ran scripted against NPCID.cs:10467
        // (131 ids, zero drift) — the range compression is exact.
    /// W4-g5 RETIRE — the caster/sphere family's `DebuffImmunitySets` rows
    /// (the base's "no debuff-immunity rows" comment was wrong): 29 {31}
    /// (`NPCID.cs:6244-6250`), 45/32 {20,31} (`:6364-6368`/`:6272-6278`),
    /// 24 {24,31,323} (`:6227-6233`), 533 the 14-row set (`:9085-9095`),
    /// 25/30/33 {31} (`:6234-6240`/`:6251-6257`/`:6279-6285`); the 665
    /// ChaosBallTim clone shares the 30's {31} (`:6258-6264`) — the R4
    /// fold's one kind carries both net ids' immunity. WRITTEN NOT RUN.
    #[test]
    fn w4_g5_the_caster_family_immunity_rows_match_the_id_sets() {
        let sorc = NpcBuffs::for_kind(NpcKind::GoblinSorcerer);
        assert!(sorc.buff_immune[31] && !sorc.buff_immune[20]);
        for kind in [NpcKind::Tim, NpcKind::DarkCaster] {
            let b = NpcBuffs::for_kind(kind);
            assert!(b.buff_immune[20] && b.buff_immune[31], "{kind:?}");
        }
        let imp = NpcBuffs::for_kind(NpcKind::FireImp);
        assert!(imp.buff_immune[24] && imp.buff_immune[31] && imp.buff_immune[323]);
        assert!(!imp.buff_immune[20], "the 24 row has no Poisoned");
        let djinn = NpcBuffs::for_kind(NpcKind::DesertDjinn);
        for id in [20, 24, 31, 39, 44, 69, 70, 395, 153, 189, 203, 204, 323, 324] {
            assert!(djinn.buff_immune[id], "the 533 row's {id}");
        }
        assert!(!djinn.buff_immune[46], "no row outside the 14");
        for kind in [NpcKind::ChaosBall, NpcKind::BurningSphere, NpcKind::WaterSphere] {
            let b = NpcBuffs::for_kind(kind);
            assert!(b.buff_immune[31], "{kind:?} Confused-immune");
            assert!(!b.buff_immune[20], "{kind:?} not Poisoned-immune");
        }
            }
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

    // -----------------------------------------------------------------
    // W4-f47 — buff 353 (Shimmering) + the repeated-buff helper witnesses
    // (WRITTEN NOT RUN, code-only mode)
    // -----------------------------------------------------------------

    #[test]
    fn w4_f47_buff_353_sets_the_shimmering_flag() {
        // NPC.cs:92133-92143 — the non-immune branch sets shimmering.
        let mut b = NpcBuffs::default();
        b.add_buff(353, 100);
        let mut hp = 100;
        let flags = b.update_buffs_tick(&mut hp, 100);
        assert!(flags.shimmering);
        assert_eq!(b.buff_time_of(353), 99, "the tick decremented");
        // No 353 → the flag is false (the reset is implicit).
        let mut b2 = NpcBuffs::default();
        let flags2 = b2.update_buffs_tick(&mut hp, 100);
        assert!(!flags2.shimmering);
    }

    #[test]
    fn w4_f47_buff_353_immune_deletes_mid_scan() {
        // NPC.cs:92135-92138 — buffImmune[353] → DelBuff(i) IN the scan:
        // the shift-left skips the shifted-in slot this tick.
        let mut b = NpcBuffs::default();
        b.buff_immune[353] = true;
        b.buff_type[0] = 353;
        b.buff_time[0] = 50;
        b.buff_type[1] = 20; // Poisoned shifts into slot 0, skipped this tick
        b.buff_time[1] = 50;
        let mut hp = 100;
        let flags = b.update_buffs_tick(&mut hp, 100);
        assert!(!flags.shimmering, "the immune branch never sets the flag");
        assert!(!flags.poisoned, "the shifted-in slot is skipped this tick");
        assert_eq!(b.buff_type[0], 20);
        assert_eq!(b.buff_time[0], 50, "the shifted-in buff did not decrement");
        assert_eq!(b.find_buff_index(353), None, "353 is gone");
        // The next tick processes the shifted-in Poisoned.
        let flags2 = b.update_buffs_tick(&mut hp, 100);
        assert!(flags2.poisoned);
    }

    #[test]
    fn w4_f47_try_adding_repeated_buff_respects_the_low_threshold() {
        // NPC.cs:93303-93313 — absent → add; present with time <= 10 →
        // refresh; present above 10 → no-op; immune → never.
        let mut b = NpcBuffs::default();
        b.try_adding_repeated_buff(353, 100, 10);
        assert_eq!(b.buff_time_of(353), 100);
        // Above the threshold: no refresh.
        b.try_adding_repeated_buff(353, 100, 10);
        assert_eq!(b.buff_time_of(353), 100);
        // At/below the threshold: refresh.
        b.buff_time[0] = 10;
        b.try_adding_repeated_buff(353, 100, 10);
        assert_eq!(b.buff_time_of(353), 100);
        // Immune: never added.
        let mut b2 = NpcBuffs::default();
        b2.buff_immune[353] = true;
        b2.try_adding_repeated_buff(353, 100, 10);
        assert_eq!(b2.find_buff_index(353), None);
    }

    #[test]
    fn w4_f47_remove_buff_deletes_the_found_index() {
        // The town arm's FindBuffIndex(353) → DelBuff (NPC.cs:91586-91590).
        let mut b = NpcBuffs::default();
        b.add_buff(20, 60);
        b.add_buff(353, 100);
        b.remove_buff(353);
        assert_eq!(b.find_buff_index(353), None);
        assert_eq!(b.find_buff_index(20), Some(0), "the tail shifted left");
        // Absent → no-op.
        b.remove_buff(353);
        assert_eq!(b.find_buff_index(20), Some(0));
    }
}
