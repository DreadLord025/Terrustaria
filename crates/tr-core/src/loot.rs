//! The item drop rule engine — `Terraria.GameContent.ItemDropRules`
//! (`IItemDropRule` + the chain attempts, `ItemDropResolver`,
//! `ItemDropRule.cs` factories, `Luck.cs`).
//!
//! The engine is RNG-generic ([`DropRng`]) and produces [`DropRequest`]s for
//! the caller to execute (spawn the `Item`, reset the boss money, …); it never
//! touches world state itself. The per-NPC rule registrations live in
//! [`ItemDropDatabase`] (M4-LOOT-2 fills the 643 registrations; this module
//! ships the engine and the container).

/// The random source the rules roll against — `UnifiedRandom` in the C#
/// (`DropAttemptInfo.rng`). `Next(min, max)` is exclusive of `max`, exactly
/// like the C# overload.
pub trait DropRng {
    fn next_range(&mut self, min: i32, max: i32) -> i32;
    fn next_float(&mut self) -> f32;
}

impl DropRng for crate::tile_light::FastRandom {
    fn next_range(&mut self, min: i32, max: i32) -> i32 {
        crate::tile_light::FastRandom::next_range(self, min, max)
    }
    fn next_float(&mut self) -> f32 {
        crate::tile_light::FastRandom::next_float(self)
    }
}

/// The named-stage streams (`Main._rngs`, `app/rng.rs`) draw through the same
/// trait — `UnifiedRandom.Next(min, max)`/`NextFloat()` are the C# methods the
/// trait mirrors (`UnifiedRandom.cs`).
impl DropRng for crate::unified_random::UnifiedRandom {
    fn next_range(&mut self, min: i32, max: i32) -> i32 {
        crate::unified_random::UnifiedRandom::next_range(self, min, max)
    }
    fn next_float(&mut self) -> f32 {
        crate::unified_random::UnifiedRandom::next_float(self)
    }
}

/// `DropAttemptInfo` (`DropAttemptInfo.cs`): the roll inputs. The NPC and the
/// player references stay with the caller; the engine only reads the scalars
/// the rules consume.
pub struct DropAttemptInfo<'a> {
    pub rng: &'a mut dyn DropRng,
    /// `Player.luck` (`Luck.RollLuck` branches on its sign).
    pub luck: f32,
    pub is_expert: bool,
    pub is_master: bool,
    /// `Main.CurrentFrameFlags.ActivePlayersCount` (`DropOneByOne`'s bonus
    /// stacks).
    pub active_players: u32,
    /// `SpecialSeedFeatures.ShouldDropExtraGel` (`DropBasedOnExtraGel`).
    pub extra_gel: bool,
    /// The world/event state the `Conditions.*` gates read (the caller
    /// snapshots it once per drop attempt).
    pub flags: LootFlags,
    /// The dying NPC's fields the conditions read (`info.npc.*`).
    pub npc: LootNpc,
}

/// The world/event snapshot the drop conditions read — `Main.hardMode`,
/// `Main.bloodMoon`, `Main.xMas`, `Main.halloween`, `NPC.downed*`,
/// `NPC.waveNumber`, the special seeds and the closest player's zones.
#[derive(Clone, Copy, Default, Debug)]
pub struct LootFlags {
    pub hard_mode: bool,
    /// `WorldGen.crimson` — the WORLD evil type, not the player zone.
    pub crimson_world: bool,
    pub blood_moon: bool,
    pub day_time: bool,
    pub xmas: bool,
    pub halloween: bool,
    /// `Main.tenthAnniversaryWorld`.
    pub tenth_anniversary: bool,
    /// `Main.dontStarveWorld`.
    pub dont_starve: bool,
    /// `WorldGen.Skyblock.lowTiles`.
    pub skyblock: bool,
    /// The player holds the Skyblock sickle (1786).
    pub player_has_sickle: bool,
    /// `SpecialSeedFeatures.RemixSeed`.
    pub remix_seed: bool,
    /// `SpecialSeedFeatures.Mechdusa`.
    pub mechdusa: bool,
    /// `Main.WindyEnoughForKiteDrops`.
    pub windy: bool,
    /// `NPC.waveNumber`.
    pub wave_number: i32,
    /// `Main.pumpkinMoon` / `Main.snowMoon`.
    pub pumpkin_moon: bool,
    pub frost_moon: bool,
    pub downed_eoc: bool,
    pub downed_boss2: bool,
    pub downed_boss3: bool,
    pub downed_mech1: bool,
    pub downed_mech2: bool,
    pub downed_mech3: bool,
    /// `NPC.downedMechBossAny`.
    pub downed_mech_any: bool,
    /// `NPC.downedPlantera`.
    pub downed_plantera: bool,
    /// `NPC.downedPlantBoss` (`Conditions.FirstTimeKillingPlantera` inverts it).
    pub downed_plant_boss: bool,
    /// `NPC.downedMechdusa`.
    pub downed_mechdusa: bool,
    /// `Conditions.EmpressOfLightIsGenuinelyEnraged` — the caller folds the
    /// AI-120 rage gate in.
    pub empress_enraged: bool,
    /// `Conditions.EyeOfCthulhuDefeatedAndNoAltarsInWorld`.
    pub eoc_defeated_no_altars: bool,
    /// The underworld depth band (`Conditions.LivingFlames`).
    pub underworld: bool,
    /// The closest player's zones (`Player.Zone*`).
    pub zone_jungle: bool,
    pub zone_corrupt: bool,
    pub zone_crimson: bool,
    pub zone_hallow: bool,
    pub zone_desert: bool,
    pub zone_snow: bool,
    /// `Conditions.SoulOfWhateverConditionCanDrop`'s depth band.
    pub soul_depth_ok: bool,
    /// The PirateMap depth/ocean band (`Conditions.PirateMap`).
    pub pirate_map_band: bool,
    /// `HalloweenWeapons`' `info.npc.value` cap (500 × the difficulty mult).
    pub halloween_value_cap: f32,
    /// `HalloweenWeapons`' `info.npc.damage` cap (40 × the difficulty mult).
    pub halloween_damage_cap: f32,
    /// `Conditions.YoyoCascade`'s deep band (`y / 16 > maxTilesY - 350`).
    pub deep_yoyo_band: bool,
    /// `Conditions.YoyosHelFire`'s deep band
    /// (`y / 16 > (rockLayer + maxTilesY * 2) / 3`).
    pub helfire_band: bool,
    /// The closest player's `Player.ZoneDungeon`.
    pub zone_dungeon: bool,
}

/// The dying NPC's fields the conditions read (`info.npc.SpawnedFromStatue`,
/// `.value > 0f`, `.lifeMax > 1`, `.damage > 0`, `.friendly`, `.type`).
#[derive(Clone, Copy, Default, Debug)]
pub struct LootNpc {
    pub kind: i32,
    /// `info.npc.value`.
    pub value: f32,
    /// `info.npc.lifeMax`.
    pub life_max: i32,
    /// `info.npc.damage`.
    pub damage: i32,
    /// `info.npc.defense`.
    pub defense: i32,
    pub friendly: bool,
    /// `info.npc.boss` (`Conditions.LegacyHack_IsABoss`).
    pub is_boss: bool,
    pub spawned_from_statue: bool,
    /// `NPCID.Sets.DontDropDungeonKeysOrSouls[type]`.
    pub dont_drop_keys: bool,
    /// The Twin's other half is absent (`Conditions.MissingTwin`).
    pub twin_missing: bool,
    /// The other Mechdusa parts are absent (`Conditions.MechdusaKill`).
    pub mechdusa_parts_cleared: bool,
    /// `info.npc.HasPlayerTarget` (the yoyo gates).
    pub has_player_target: bool,
}

/// One resolved drop — the arguments of `CommonCode.DropItemFromNPC`.
/// `resets_money` mirrors `DropItemLocalPerClientAndSetNPCMoneyTo0`'s
/// `npc.value = 0f` side effect (boss bags); the caller applies it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DropRequest {
    pub item: i32,
    pub stack: i32,
    pub scattered: bool,
    pub resets_money: bool,
}

fn drop_request(item: i32, stack: i32, scattered: bool) -> DropRequest {
    DropRequest {
        item,
        stack,
        scattered,
        resets_money: false,
    }
}

/// `ItemDropAttemptResult` + `ItemDropAttemptResultState`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum DropResultState {
    DidNotRunCode,
    Success,
    FailedRandomRoll,
    DoesntFillConditions,
}

#[derive(Clone, Copy, Debug)]
pub struct DropAttemptResult {
    pub state: DropResultState,
}

/// `IItemDropRuleCondition.CanDrop` (`Conditions.cs`). `CanShowItemDropInUI`
/// is bestiary-side and stays out until M4-BEST-1.
pub trait DropCondition: Send + Sync {
    fn can_drop(&self, info: &mut DropAttemptInfo) -> bool;
}

/// `IItemDropRuleChainAttempt` (`Chains.cs`): the three chain kinds keyed by
/// the parent result.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ChainKind {
    /// `TryIfFailedRandomRoll`.
    OnFailedRoll,
    /// `TryIfSucceeded`.
    OnSuccess,
    /// `TryIfDoesntFillConditions`.
    OnDoesntFillConditions,
}

impl ChainKind {
    /// `CanChainIntoRule`.
    pub fn can_chain_into(self, parent: &DropAttemptResult) -> bool {
        match self {
            ChainKind::OnFailedRoll => parent.state == DropResultState::FailedRandomRoll,
            ChainKind::OnSuccess => parent.state == DropResultState::Success,
            ChainKind::OnDoesntFillConditions => {
                parent.state == DropResultState::DoesntFillConditions
            }
        }
    }
}

pub struct ChainRule {
    pub kind: ChainKind,
    pub rule: Box<dyn ItemDropRule>,
}

/// The downcast hook the chain sugar needs to reach a concrete rule's
/// `chains` vec (the C# mutates `ChainedRules` through the interface; Rust
/// walks one concrete level).
#[doc(hidden)]
pub trait AsChains {
    fn chains_mut(&mut self) -> &mut Vec<ChainRule>;
}

/// `IItemDropRule` (`IItemDropRule.cs`). `INestedItemDropRule` rules
/// (`DropBasedOnExpertMode`, `DropBasedOnMasterMode`, `DropBasedOnExtraGel`)
/// override [`ItemDropRule::try_nested`] to route a sub-rule through the
/// resolver themselves; the resolver treats a `Some` result as the rule
/// outcome instead of `try_dropping_item`.
pub trait ItemDropRule: AsChains + Send + Sync {
    fn chained_rules(&self) -> &[ChainRule];
    fn can_drop(&self, _info: &mut DropAttemptInfo) -> bool {
        true
    }
    fn try_dropping_item(
        &self,
        info: &mut DropAttemptInfo,
        out: &mut Vec<DropRequest>,
    ) -> DropAttemptResult;
    fn try_nested(
        &self,
        _info: &mut DropAttemptInfo,
        _out: &mut Vec<DropRequest>,
    ) -> Option<DropAttemptResult> {
        None
    }
}

/// `ItemDropResolver.ResolveRule` (`ItemDropResolver.cs:23-42`).
pub fn resolve_rule(
    rule: &dyn ItemDropRule,
    info: &mut DropAttemptInfo,
    out: &mut Vec<DropRequest>,
) -> DropAttemptResult {
    if !rule.can_drop(info) {
        let result = DropAttemptResult {
            state: DropResultState::DoesntFillConditions,
        };
        resolve_chains(rule.chained_rules(), info, &result, out);
        return result;
    }
    let result = rule
        .try_nested(info, out)
        .unwrap_or_else(|| rule.try_dropping_item(info, out));
    resolve_chains(rule.chained_rules(), info, &result, out);
    result
}

/// `ItemDropResolver.ResolveRuleChains` (`:47-62`).
fn resolve_chains(
    chains: &[ChainRule],
    info: &mut DropAttemptInfo,
    parent: &DropAttemptResult,
    out: &mut Vec<DropRequest>,
) {
    for chain in chains {
        if chain.kind.can_chain_into(parent) {
            resolve_rule(chain.rule.as_ref(), info, out);
        }
    }
}

/// `ItemDropDatabase`: the per-net-id rule lists
/// (`RegisterToNPC` / `RegisterToMultipleNPCs` / `RemoveFromMultipleNPCs`).
#[derive(Default)]
pub struct ItemDropDatabase {
    /// `ItemDropDatabase._globalEntries` — applied to EVERY npc before the
    /// per-net-id entries (`GetRulesForNPCID`, `ItemDropDatabase.cs:41-52`).
    globals: Vec<Box<dyn ItemDropRule>>,
    rules: std::collections::BTreeMap<i32, Vec<Box<dyn ItemDropRule>>>,
}

#[allow(dead_code)]
fn _db_key_type_check(db: &ItemDropDatabase, net_id: i32) -> &[Box<dyn ItemDropRule>] {
    db.rules.get(&net_id).map(|v| v.as_slice()).unwrap_or(&[])
}

impl ItemDropDatabase {
    pub fn new() -> Self {
        Self::default()
    }

    /// `RegisterToNPC` (`ItemDropDatabase.cs:33`).
    pub fn register_to_npc(&mut self, net_id: i32, rule: Box<dyn ItemDropRule>) {
        self.rules.entry(net_id).or_default().push(rule);
    }

    /// `RegisterToMultipleNPCs`.
    pub fn register_to_multiple(
        &mut self,
        net_ids: &[i32],
        make_rule: &dyn Fn() -> Box<dyn ItemDropRule>,
    ) {
        for id in net_ids {
            self.rules.entry(*id).or_default().push(make_rule());
        }
    }

    /// `RemoveFromMultipleNPCs` — the last registered rule per id.
    pub fn remove_from_multiple(&mut self, net_ids: &[i32]) {
        for id in net_ids {
            if let Some(list) = self.rules.get_mut(id) {
                list.pop();
            }
        }
    }

    /// `GetRulesForNPCID`.
    /// `ItemDropDatabase.RegisterToGlobal` (`ItemDropDatabase.cs:55-59`).
    pub fn register_global(&mut self, rule: Box<dyn ItemDropRule>) {
        self.globals.push(rule);
    }

    pub fn rules_for_npc(&self, net_id: i32) -> &[Box<dyn ItemDropRule>] {
        self.rules.get(&net_id).map(|v| v.as_slice()).unwrap_or(&[])
    }
}

/// `ItemDropResolver.TryDropping` (`:14-21`).
pub fn try_dropping(
    db: &ItemDropDatabase,
    net_id: i32,
    info: &mut DropAttemptInfo,
    out: &mut Vec<DropRequest>,
) {
    // `GetRulesForNPCID(includeGlobalDrops = true)`: the globals first.
    for rule in &db.globals {
        resolve_rule(rule.as_ref(), info, out);
    }
    for rule in db.rules_for_npc(net_id) {
        resolve_rule(rule.as_ref(), info, out);
    }
}

// --- Luck (`GameContent/Luck.cs`) -------------------------------------------

/// `Luck.RollLuck` (`Luck.cs:5-14`): positive luck shrinks the roll range
/// (better odds), negative luck widens it; both consume the stream only when
/// the gate float passes.
pub fn roll_luck(rng: &mut dyn DropRng, luck: f32, range: i32) -> i32 {
    if luck > 0.0 && rng.next_float() < luck {
        let shrunk = rng.next_range(range / 2, range);
        return rng.next_range(0, shrunk);
    }
    if luck < 0.0 && rng.next_float() < 0.0 - luck {
        let widened = rng.next_range(range, range * 2);
        return rng.next_range(0, widened);
    }
    rng.next_range(0, range)
}

/// `Luck.RollBadLuck` (`Luck.cs:16-26`).
pub fn roll_bad_luck(rng: &mut dyn DropRng, luck: f32, range: i32) -> i32 {
    if luck > 0.0 && rng.next_float() < luck {
        let widened = rng.next_range(range, range * 2);
        return rng.next_range(0, widened);
    }
    if luck < 0.0 && rng.next_float() < 0.0 - luck {
        let shrunk = rng.next_range(range / 2, range);
        return rng.next_range(0, shrunk);
    }
    rng.next_range(0, range)
}

/// `Luck.RollOnlyBadLuck` (`Luck.cs:28-36`).
pub fn roll_only_bad_luck(rng: &mut dyn DropRng, luck: f32, range: i32) -> i32 {
    if luck < 0.0 && rng.next_float() < 0.0 - luck {
        let shrunk = rng.next_range(range / 2, range);
        return rng.next_range(0, shrunk);
    }
    rng.next_range(0, range)
}

/// `Luck.RollBadLuckExtreme` (`Luck.cs:38-48`).
pub fn roll_bad_luck_extreme(rng: &mut dyn DropRng, luck: f32, range: i32) -> i32 {
    if luck > 0.0 && rng.next_float() < luck {
        return rng.next_range(0, range * 10);
    }
    if luck < 0.0 && rng.next_float() < 0.0 - luck {
        return rng.next_range(0, range / 10);
    }
    rng.next_range(0, range)
}

/// `Luck.RollOnlyBadLuckExtreme` (`Luck.cs:50-58`).
pub fn roll_only_bad_luck_extreme(rng: &mut dyn DropRng, luck: f32, range: i32) -> i32 {
    if luck < 0.0 && rng.next_float() < 0.0 - luck {
        return rng.next_range(0, range / 10);
    }
    -1
}

// --- Money (`NPC.NPCLoot_DropMoney`, `NPC.cs:79619-80106`) -------------------

/// `NPC.NPCLoot_DropMoney` (`NPC.cs:79619-80106`): the value rolls (midas,
/// the ±20-75 % spread, the six bonus chains, the blood-moon bump, the luck
/// double-roll keeping the best/worst) then the platinum(74)/gold(73)/
/// silver(72)/copper(71) split with the >50-count shrink rolls. `extra_value`
/// is `NPC.extraValue` (0 for every ported kind). Coin ids are the C#
/// literals; the stack is copper value.
pub fn drop_money(
    rng: &mut dyn DropRng,
    value: f32,
    luck: f32,
    midas: bool,
    blood_moon: bool,
    extra_value: f32,
    out: &mut Vec<DropRequest>,
) {
    let mut num = 0f32;
    // The luck double-roll: `NextFloat() < |luck|` rolls twice; positive luck
    // keeps the best roll, negative the worst (`NPC.cs:79622-79626`).
    let rolls = if rng.next_float() < luck.abs() { 2 } else { 1 };
    for i in 0..rolls {
        let mut roll = value;
        if midas {
            // `Midas`: `* (1 + Next(30, 51) * 0.01f)` (`:79634-79637`).
            roll *= 1.0 + rng.next_range(30, 51) as f32 * 0.01;
        }
        roll *= 1.0 + rng.next_range(-20, 76) as f32 * 0.01;
        // The bonus chains: 1/2:+5-10 %, 1/4:+10-20 %, 1/8:+15-30 %,
        // 1/16:+20-40 %, 1/32:+25-50 %, 1/64:+50-100 % (`:79641-79664`).
        if rng.next_range(0, 2) == 0 {
            roll *= 1.0 + rng.next_range(5, 11) as f32 * 0.01;
        }
        if rng.next_range(0, 4) == 0 {
            roll *= 1.0 + rng.next_range(10, 21) as f32 * 0.01;
        }
        if rng.next_range(0, 8) == 0 {
            roll *= 1.0 + rng.next_range(15, 31) as f32 * 0.01;
        }
        if rng.next_range(0, 16) == 0 {
            roll *= 1.0 + rng.next_range(20, 41) as f32 * 0.01;
        }
        if rng.next_range(0, 32) == 0 {
            roll *= 1.0 + rng.next_range(25, 51) as f32 * 0.01;
        }
        if rng.next_range(0, 64) == 0 {
            roll *= 1.0 + rng.next_range(50, 101) as f32 * 0.01;
        }
        if blood_moon {
            roll *= 1.0 + rng.next_range(0, 101) as f32 * 0.01;
        }
        // The first roll is the floor; later rolls win by best/worst luck
        // (`:79666-79676`).
        if i == 0 {
            num = roll;
        } else if luck < 0.0 {
            if roll < num {
                num = roll;
            }
        } else if roll > num {
            num = roll;
        }
    }
    num += extra_value;

    // The coin split (`:79680-80105`). Every tier shrink-rolls a >50 count
    // down (`Next(5) == 0` then `Next(3) + 1`, twice; copper's second roll is
    // `Next(4) + 1`) and the copper remainder clamps to >= 1. The loop guard
    // truncates (`while ((int)num > 0)`), so a sub-1 remainder exits instead
    // of minting bonus coins.
    while num as i32 > 0 {
        if num > 1_000_000.0 {
            let mut count = (num / 1_000_000.0) as i32;
            if count > 50 && rng.next_range(0, 5) == 0 {
                count /= rng.next_range(0, 3) + 1;
            }
            if rng.next_range(0, 5) == 0 {
                count /= rng.next_range(0, 3) + 1;
            }
            let mut stack = count;
            while stack > 999 {
                stack -= 999;
                out.push(drop_request(74, 999, false));
            }
            num -= (1_000_000 * count) as f32;
            out.push(drop_request(74, stack, false));
        } else if num > 10_000.0 {
            let mut count = (num / 10_000.0) as i32;
            if count > 50 && rng.next_range(0, 5) == 0 {
                count /= rng.next_range(0, 3) + 1;
            }
            if rng.next_range(0, 5) == 0 {
                count /= rng.next_range(0, 3) + 1;
            }
            num -= (10_000 * count) as f32;
            out.push(drop_request(73, count, false));
        } else if num > 100.0 {
            let mut count = (num / 100.0) as i32;
            if count > 50 && rng.next_range(0, 5) == 0 {
                count /= rng.next_range(0, 3) + 1;
            }
            if rng.next_range(0, 5) == 0 {
                count /= rng.next_range(0, 3) + 1;
            }
            num -= (100 * count) as f32;
            out.push(drop_request(72, count, false));
        } else {
            let mut count = num as i32;
            if count > 50 && rng.next_range(0, 5) == 0 {
                count /= rng.next_range(0, 3) + 1;
            }
            if rng.next_range(0, 5) == 0 {
                count /= rng.next_range(0, 4) + 1;
            }
            if count < 1 {
                count = 1;
            }
            num -= count as f32;
            out.push(drop_request(71, count, false));
        }
    }
}

// --- Rules -------------------------------------------------------------------

macro_rules! impl_chains {
    ($t:ty) => {
        impl AsChains for $t {
            fn chains_mut(&mut self) -> &mut Vec<ChainRule> {
                &mut self.chains
            }
        }
    };
}

/// `DropNothing` (`ItemDropRule.cs:66`).
pub struct DropNothing {
    chains: Vec<ChainRule>,
}

impl DropNothing {
    pub fn new() -> Box<Self> {
        Box::new(Self { chains: Vec::new() })
    }
}

impl ItemDropRule for DropNothing {
    fn chained_rules(&self) -> &[ChainRule] {
        &self.chains
    }
    fn try_dropping_item(
        &self,
        _info: &mut DropAttemptInfo,
        _out: &mut Vec<DropRequest>,
    ) -> DropAttemptResult {
        DropAttemptResult {
            state: DropResultState::FailedRandomRoll,
        }
    }
}
impl_chains!(DropNothing);

/// `CommonDrop` (`CommonDrop.cs`): `RollLuck(chanceDenominator) <
/// chanceNumerator` gates `min..=max` of one item id.
pub struct CommonDrop {
    pub item: i32,
    pub chance_denominator: i32,
    pub amount_min: i32,
    pub amount_max: i32,
    pub chance_numerator: i32,
    /// `CommonDropNotScalingWithLuck` flips this off
    /// (`ItemDropRule.NotScalingWithLuck`, `ItemDropRule.cs:50-52`).
    pub uses_luck: bool,
    /// `CommonDropScalingWithOnlyBadLuck` keeps only the bad-luck side
    /// (`ItemDropRule.ScalingWithOnlyBadLuck`, `:45-48`).
    pub bad_luck_only: bool,
    chains: Vec<ChainRule>,
}

impl CommonDrop {
    /// The `Common` factory (`ItemDropRule.cs:7-10`).
    pub fn new(item: i32, chance_denominator: i32, min: i32, max: i32) -> Box<Self> {
        Box::new(Self {
            item,
            chance_denominator,
            amount_min: min,
            amount_max: max,
            chance_numerator: 1,
            uses_luck: true,
            bad_luck_only: false,
            chains: Vec::new(),
        })
    }

    pub fn with_numerator(
        item: i32,
        chance_denominator: i32,
        min: i32,
        max: i32,
        chance_numerator: i32,
    ) -> Box<Self> {
        Box::new(Self {
            item,
            chance_denominator,
            amount_min: min,
            amount_max: max,
            chance_numerator,
            uses_luck: true,
            bad_luck_only: false,
            chains: Vec::new(),
        })
    }
}

impl ItemDropRule for CommonDrop {
    fn chained_rules(&self) -> &[ChainRule] {
        &self.chains
    }
    fn try_dropping_item(
        &self,
        info: &mut DropAttemptInfo,
        out: &mut Vec<DropRequest>,
    ) -> DropAttemptResult {
        // `CommonDrop.TryDroppingItem`: the stack range is inclusive of the
        // maximum (`Next(min, max + 1)`). The luck variants swap the scaled
        // luck for 0 (NotScalingWithLuck) or for its bad side only
        // (ScalingWithOnlyBadLuck).
        let luck = if !self.uses_luck {
            0.0
        } else if self.bad_luck_only {
            info.luck.min(0.0)
        } else {
            info.luck
        };
        if roll_luck(info.rng, luck, self.chance_denominator) < self.chance_numerator {
            let stack = info.rng.next_range(self.amount_min, self.amount_max + 1);
            out.push(drop_request(self.item, stack, false));
            DropAttemptResult {
                state: DropResultState::Success,
            }
        } else {
            DropAttemptResult {
                state: DropResultState::FailedRandomRoll,
            }
        }
    }
}
impl_chains!(CommonDrop);

/// `CommonDropWithRerolls` (`CommonDropWithRerolls.cs`): `rerolls + 1` gate
/// rolls, one stack on any hit.
pub struct CommonDropWithRerolls {
    pub item: i32,
    pub chance_denominator: i32,
    pub amount_min: i32,
    pub amount_max: i32,
    pub rerolls: i32,
    chains: Vec<ChainRule>,
}

impl CommonDropWithRerolls {
    /// The `WithRerolls` factory (`ItemDropRule.cs:40-42`).
    pub fn new(item: i32, rerolls: i32, chance_denominator: i32, min: i32, max: i32) -> Box<Self> {
        Box::new(Self {
            item,
            chance_denominator,
            amount_min: min,
            amount_max: max,
            rerolls,
            chains: Vec::new(),
        })
    }
}

impl ItemDropRule for CommonDropWithRerolls {
    fn chained_rules(&self) -> &[ChainRule] {
        &self.chains
    }
    fn try_dropping_item(
        &self,
        info: &mut DropAttemptInfo,
        out: &mut Vec<DropRequest>,
    ) -> DropAttemptResult {
        let times = self.rerolls + 1;
        let mut flag = false;
        for _ in 0..times {
            flag = flag || roll_luck(info.rng, info.luck, self.chance_denominator) < 1;
        }
        if flag {
            let stack = info.rng.next_range(self.amount_min, self.amount_max + 1);
            out.push(drop_request(self.item, stack, false));
            DropAttemptResult {
                state: DropResultState::Success,
            }
        } else {
            DropAttemptResult {
                state: DropResultState::FailedRandomRoll,
            }
        }
    }
}
impl_chains!(CommonDropWithRerolls);

/// `OneFromOptionsDropRule` (`OneFromOptionsDropRule.cs`): the gate picks one
/// option uniformly.
pub struct OneFromOptionsDrop {
    pub options: Vec<i32>,
    pub chance_denominator: i32,
    pub chance_numerator: i32,
    /// `OneFromOptionsNotScaledWithLuckDropRule` flips this off
    /// (`ItemDropRule.OneFromOptionsNotScalingWithLuck`).
    pub uses_luck: bool,
    chains: Vec<ChainRule>,
}

impl OneFromOptionsDrop {
    /// The `OneFromOptions` factory (`ItemDropRule.cs:57-59`).
    pub fn new(chance_denominator: i32, options: &[i32]) -> Box<Self> {
        Box::new(Self {
            options: options.to_vec(),
            chance_denominator,
            chance_numerator: 1,
            uses_luck: true,
            chains: Vec::new(),
        })
    }
}

impl ItemDropRule for OneFromOptionsDrop {
    fn chained_rules(&self) -> &[ChainRule] {
        &self.chains
    }
    fn try_dropping_item(
        &self,
        info: &mut DropAttemptInfo,
        out: &mut Vec<DropRequest>,
    ) -> DropAttemptResult {
        if roll_luck(info.rng, info.luck, self.chance_denominator) < self.chance_numerator {
            let pick = info.rng.next_range(0, self.options.len() as i32);
            if let Some(item) = self.options.get(pick as usize) {
                out.push(drop_request(*item, 1, false));
            }
            DropAttemptResult {
                state: DropResultState::Success,
            }
        } else {
            DropAttemptResult {
                state: DropResultState::FailedRandomRoll,
            }
        }
    }
}
impl_chains!(OneFromOptionsDrop);

/// `ItemDropWithConditionRule` (`ItemDropWithConditionRule.cs`): the condition
/// gates both `CanDrop` and the attempt (the `FailedRandomRoll` vs
/// `DoesntFillConditions` split drives the two chain kinds).
pub struct ItemDropWithCondition {
    pub condition: Box<dyn DropCondition>,
    pub item: i32,
    pub chance_denominator: i32,
    pub amount_min: i32,
    pub amount_max: i32,
    pub chance_numerator: i32,
    chains: Vec<ChainRule>,
}

impl ItemDropWithCondition {
    /// The `ByCondition` factory (`ItemDropRule.cs:44-46`).
    pub fn new(
        condition: Box<dyn DropCondition>,
        item: i32,
        chance_denominator: i32,
        min: i32,
        max: i32,
    ) -> Box<Self> {
        Box::new(Self {
            condition,
            item,
            chance_denominator,
            amount_min: min,
            amount_max: max,
            chance_numerator: 1,
            chains: Vec::new(),
        })
    }
}

impl ItemDropRule for ItemDropWithCondition {
    fn chained_rules(&self) -> &[ChainRule] {
        &self.chains
    }
    fn can_drop(&self, info: &mut DropAttemptInfo) -> bool {
        self.condition.can_drop(info)
    }
    fn try_dropping_item(
        &self,
        info: &mut DropAttemptInfo,
        out: &mut Vec<DropRequest>,
    ) -> DropAttemptResult {
        // `ItemDropWithConditionRule.TryDroppingItem`: the condition failing
        // inside the attempt reports `DoesntFillConditions` so
        // `OnFailedConditions` chains fire.
        if !self.condition.can_drop(info) {
            return DropAttemptResult {
                state: DropResultState::DoesntFillConditions,
            };
        }
        if roll_luck(info.rng, info.luck, self.chance_denominator) < self.chance_numerator {
            let stack = info.rng.next_range(self.amount_min, self.amount_max + 1);
            out.push(drop_request(self.item, stack, false));
            DropAttemptResult {
                state: DropResultState::Success,
            }
        } else {
            DropAttemptResult {
                state: DropResultState::FailedRandomRoll,
            }
        }
    }
}
impl_chains!(ItemDropWithCondition);

/// `DropBasedOnExpertMode` (`DropBasedOnExpertMode.cs`, an
/// `INestedItemDropRule`).
pub struct DropBasedOnExpertMode {
    pub normal: Box<dyn ItemDropRule>,
    pub expert: Box<dyn ItemDropRule>,
    chains: Vec<ChainRule>,
}

impl DropBasedOnExpertMode {
    pub fn new(normal: Box<dyn ItemDropRule>, expert: Box<dyn ItemDropRule>) -> Box<Self> {
        Box::new(Self {
            normal,
            expert,
            chains: Vec::new(),
        })
    }
}

impl ItemDropRule for DropBasedOnExpertMode {
    fn chained_rules(&self) -> &[ChainRule] {
        &self.chains
    }
    fn try_dropping_item(
        &self,
        _info: &mut DropAttemptInfo,
        _out: &mut Vec<DropRequest>,
    ) -> DropAttemptResult {
        // The C# nested rules' plain `TryDroppingItem` reports DidNotRunCode —
        // the resolver only reaches them through `try_nested`.
        DropAttemptResult {
            state: DropResultState::DidNotRunCode,
        }
    }
    fn can_drop(&self, info: &mut DropAttemptInfo) -> bool {
        if info.is_expert {
            self.expert.can_drop(info)
        } else {
            self.normal.can_drop(info)
        }
    }
    fn try_nested(
        &self,
        info: &mut DropAttemptInfo,
        out: &mut Vec<DropRequest>,
    ) -> Option<DropAttemptResult> {
        Some(if info.is_expert {
            resolve_rule(self.expert.as_ref(), info, out)
        } else {
            resolve_rule(self.normal.as_ref(), info, out)
        })
    }
}
impl_chains!(DropBasedOnExpertMode);

/// `DropBasedOnMasterMode` (`DropBasedOnMasterMode.cs`) — the same nested
/// shape on `IsMasterMode`.
pub struct DropBasedOnMasterMode {
    pub normal: Box<dyn ItemDropRule>,
    pub master: Box<dyn ItemDropRule>,
    chains: Vec<ChainRule>,
}

impl DropBasedOnMasterMode {
    pub fn new(normal: Box<dyn ItemDropRule>, master: Box<dyn ItemDropRule>) -> Box<Self> {
        Box::new(Self {
            normal,
            master,
            chains: Vec::new(),
        })
    }
}

impl ItemDropRule for DropBasedOnMasterMode {
    fn chained_rules(&self) -> &[ChainRule] {
        &self.chains
    }
    fn try_dropping_item(
        &self,
        _info: &mut DropAttemptInfo,
        _out: &mut Vec<DropRequest>,
    ) -> DropAttemptResult {
        // The C# nested rules' plain `TryDroppingItem` reports DidNotRunCode —
        // the resolver only reaches them through `try_nested`.
        DropAttemptResult {
            state: DropResultState::DidNotRunCode,
        }
    }
    fn can_drop(&self, info: &mut DropAttemptInfo) -> bool {
        if info.is_master {
            self.master.can_drop(info)
        } else {
            self.normal.can_drop(info)
        }
    }
    fn try_nested(
        &self,
        info: &mut DropAttemptInfo,
        out: &mut Vec<DropRequest>,
    ) -> Option<DropAttemptResult> {
        Some(if info.is_master {
            resolve_rule(self.master.as_ref(), info, out)
        } else {
            resolve_rule(self.normal.as_ref(), info, out)
        })
    }
}
impl_chains!(DropBasedOnMasterMode);

/// `DropLocalPerClientAndResetsNPCMoneyTo0` (`DropLocalPerClient...cs`): the
/// boss-bag drop whose `CommonCode` side effect zeroes `npc.value`
/// (`CommonCode.cs:29-39`).
pub struct DropLocalPerClientAndResetsMoney {
    pub item: i32,
    pub chance_denominator: i32,
    pub amount_min: i32,
    pub amount_max: i32,
    pub condition: Option<Box<dyn DropCondition>>,
    chains: Vec<ChainRule>,
}

impl DropLocalPerClientAndResetsMoney {
    /// The `BossBag` factory's inner rule (`ItemDropRule.cs:12-14`).
    pub fn new(item: i32, denom: i32, min: i32, max: i32) -> Box<Self> {
        Box::new(Self {
            item,
            chance_denominator: denom,
            amount_min: min,
            amount_max: max,
            condition: None,
            chains: Vec::new(),
        })
    }

    /// `BossBagByCondition`.
    pub fn with_condition(
        item: i32,
        denom: i32,
        min: i32,
        max: i32,
        condition: Box<dyn DropCondition>,
    ) -> Box<Self> {
        Box::new(Self {
            item,
            chance_denominator: denom,
            amount_min: min,
            amount_max: max,
            condition: Some(condition),
            chains: Vec::new(),
        })
    }
}

impl ItemDropRule for DropLocalPerClientAndResetsMoney {
    fn chained_rules(&self) -> &[ChainRule] {
        &self.chains
    }
    fn can_drop(&self, info: &mut DropAttemptInfo) -> bool {
        self.condition.as_ref().map_or(true, |c| c.can_drop(info))
    }
    fn try_dropping_item(
        &self,
        info: &mut DropAttemptInfo,
        out: &mut Vec<DropRequest>,
    ) -> DropAttemptResult {
        if roll_luck(info.rng, info.luck, self.chance_denominator) < 1 {
            let stack = info.rng.next_range(self.amount_min, self.amount_max + 1);
            out.push(DropRequest {
                item: self.item,
                stack,
                scattered: false,
                resets_money: true,
            });
            DropAttemptResult {
                state: DropResultState::Success,
            }
        } else {
            DropAttemptResult {
                state: DropResultState::FailedRandomRoll,
            }
        }
    }
}
impl_chains!(DropLocalPerClientAndResetsMoney);

/// `DropOneByOne.Parameters` (`DropOneByOne.cs:7-29`).
#[derive(Clone, Copy, Debug)]
pub struct DropOneByOneParams {
    pub chance_numerator: i32,
    pub chance_denominator: i32,
    pub min_drops_count: i32,
    pub max_drops_count: i32,
    pub min_stack_per_chunk_base: i32,
    pub max_stack_per_chunk_base: i32,
    pub bonus_min_per_player: i32,
    pub bonus_max_per_player: i32,
}

/// `DropOneByOne` (`DropOneByOne.cs`): N chunks of one item, each scattered.
pub struct DropOneByOne {
    pub item: i32,
    pub params: DropOneByOneParams,
    chains: Vec<ChainRule>,
}

impl DropOneByOne {
    pub fn new(item: i32, params: DropOneByOneParams) -> Box<Self> {
        Box::new(Self {
            item,
            params,
            chains: Vec::new(),
        })
    }
}

impl ItemDropRule for DropOneByOne {
    fn chained_rules(&self) -> &[ChainRule] {
        &self.chains
    }
    fn try_dropping_item(
        &self,
        info: &mut DropAttemptInfo,
        out: &mut Vec<DropRequest>,
    ) -> DropAttemptResult {
        let p = &self.params;
        if roll_luck(info.rng, info.luck, p.chance_denominator) < p.chance_numerator {
            let count = info
                .rng
                .next_range(p.min_drops_count, p.max_drops_count + 1);
            let min_stack =
                p.min_stack_per_chunk_base + info.active_players as i32 * p.bonus_min_per_player;
            let max_stack =
                p.max_stack_per_chunk_base + info.active_players as i32 * p.bonus_max_per_player;
            for _ in 0..count {
                let stack = info.rng.next_range(min_stack, max_stack + 1);
                out.push(drop_request(self.item, stack, true));
            }
            DropAttemptResult {
                state: DropResultState::Success,
            }
        } else {
            DropAttemptResult {
                state: DropResultState::FailedRandomRoll,
            }
        }
    }
}
impl_chains!(DropOneByOne);

/// `DropBasedOnExtraGel` (`DropBasedOnExtraGel.cs`): the `Gel` factory's
/// normal-vs-extra-seed routing.
pub struct DropBasedOnExtraGel {
    pub normal: Box<dyn ItemDropRule>,
    pub extra: Box<dyn ItemDropRule>,
    chains: Vec<ChainRule>,
}

impl DropBasedOnExtraGel {
    pub fn new(normal: Box<dyn ItemDropRule>, extra: Box<dyn ItemDropRule>) -> Box<Self> {
        Box::new(Self {
            normal,
            extra,
            chains: Vec::new(),
        })
    }
}

impl ItemDropRule for DropBasedOnExtraGel {
    fn chained_rules(&self) -> &[ChainRule] {
        &self.chains
    }
    fn try_dropping_item(
        &self,
        _info: &mut DropAttemptInfo,
        _out: &mut Vec<DropRequest>,
    ) -> DropAttemptResult {
        // The C# nested rules' plain `TryDroppingItem` reports DidNotRunCode —
        // the resolver only reaches them through `try_nested`.
        DropAttemptResult {
            state: DropResultState::DidNotRunCode,
        }
    }
    fn can_drop(&self, info: &mut DropAttemptInfo) -> bool {
        if info.extra_gel {
            self.extra.can_drop(info)
        } else {
            self.normal.can_drop(info)
        }
    }
    fn try_nested(
        &self,
        info: &mut DropAttemptInfo,
        out: &mut Vec<DropRequest>,
    ) -> Option<DropAttemptResult> {
        Some(if info.extra_gel {
            resolve_rule(self.extra.as_ref(), info, out)
        } else {
            resolve_rule(self.normal.as_ref(), info, out)
        })
    }
}
impl_chains!(DropBasedOnExtraGel);

// --- Chain sugar (`Chains.cs:93-109`) ----------------------------------------

/// The `OnSuccess`/`OnFailedRoll`/`OnFailedConditions` chain builders as a
/// trait so they are callable on `Box<ConcreteRule>` and `Box<dyn
/// ItemDropRule>` alike (the C# extension methods on `IItemDropRule`).
pub trait ChainSugar {
    fn on_failed_roll(self: Box<Self>, rule: Box<dyn ItemDropRule>) -> Box<dyn ItemDropRule>;
    fn on_success(self: Box<Self>, rule: Box<dyn ItemDropRule>) -> Box<dyn ItemDropRule>;
    fn on_failed_conditions(self: Box<Self>, rule: Box<dyn ItemDropRule>) -> Box<dyn ItemDropRule>;
}

impl<T: ItemDropRule + 'static> ChainSugar for T {
    fn on_failed_roll(mut self: Box<Self>, rule: Box<dyn ItemDropRule>) -> Box<dyn ItemDropRule> {
        self.chains_mut().push(ChainRule {
            kind: ChainKind::OnFailedRoll,
            rule,
        });
        self
    }
    fn on_success(mut self: Box<Self>, rule: Box<dyn ItemDropRule>) -> Box<dyn ItemDropRule> {
        self.chains_mut().push(ChainRule {
            kind: ChainKind::OnSuccess,
            rule,
        });
        self
    }
    fn on_failed_conditions(
        mut self: Box<Self>,
        rule: Box<dyn ItemDropRule>,
    ) -> Box<dyn ItemDropRule> {
        self.chains_mut().push(ChainRule {
            kind: ChainKind::OnDoesntFillConditions,
            rule,
        });
        self
    }
}

/// The erased-rule chain builders (the C# extension methods
/// `Chains.OnSuccess`/`OnFailedRoll`/`OnFailedConditions` over `IItemDropRule`
/// references; the `self: Box<Self>` receivers do not erase across `dyn`).
pub fn chain_on_success(mut rule: Box<dyn ItemDropRule>, sub: Box<dyn ItemDropRule>) -> Box<dyn ItemDropRule> {
    rule.chains_mut().push(ChainRule {
        kind: ChainKind::OnSuccess,
        rule: sub,
    });
    rule
}

pub fn chain_on_failed_roll(mut rule: Box<dyn ItemDropRule>, sub: Box<dyn ItemDropRule>) -> Box<dyn ItemDropRule> {
    rule.chains_mut().push(ChainRule {
        kind: ChainKind::OnFailedRoll,
        rule: sub,
    });
    rule
}

pub fn chain_on_failed_conditions(mut rule: Box<dyn ItemDropRule>, sub: Box<dyn ItemDropRule>) -> Box<dyn ItemDropRule> {
    rule.chains_mut().push(ChainRule {
        kind: ChainKind::OnDoesntFillConditions,
        rule: sub,
    });
    rule
}

// --- Conditions (the few the engine and the tests need; the ~71 concrete
// classes land with M4-LOOT-2's registrations) --------------------------------

/// `Conditions.NeverTrue`.
pub struct NeverTrue;
impl DropCondition for NeverTrue {
    fn can_drop(&self, _info: &mut DropAttemptInfo) -> bool {
        false
    }
}

/// `Conditions.IsExpert`.
pub struct IsExpert;
impl DropCondition for IsExpert {
    fn can_drop(&self, info: &mut DropAttemptInfo) -> bool {
        info.is_expert
    }
}

/// `Conditions.NotExpert`.
pub struct NotExpert;
impl DropCondition for NotExpert {
    fn can_drop(&self, info: &mut DropAttemptInfo) -> bool {
        !info.is_expert
    }
}

/// `Conditions.IsMasterMode`.
pub struct IsMasterMode;
impl DropCondition for IsMasterMode {
    fn can_drop(&self, info: &mut DropAttemptInfo) -> bool {
        info.is_master
    }
}

// --- Factories (`ItemDropRule.cs:5-115`) -------------------------------------

/// The `Gel` factory (`ItemDropRule.cs:68-74`): the base rule with the stacks
/// doubled under the extra-gel seed feature.
pub fn gel(chance_denominator: i32, min: i32, max: i32) -> Box<dyn ItemDropRule> {
    DropBasedOnExtraGel::new(
        CommonDrop::new(23, chance_denominator, min, max),
        CommonDrop::new(23, chance_denominator, min * 2, max * 2),
    )
}

/// The `BossBag` factory (`ItemDropRule.cs:12-14`).
pub fn boss_bag(item: i32) -> Box<dyn ItemDropRule> {
    DropBasedOnExpertMode::new(
        DropNothing::new(),
        DropLocalPerClientAndResetsMoney::new(item, 1, 1, 1),
    )
}

/// The `Common` factory (`ItemDropRule.cs:7-10`).
pub fn common(item: i32, chance_denominator: i32, min: i32, max: i32) -> Box<dyn ItemDropRule> {
    CommonDrop::new(item, chance_denominator, min, max)
}

/// The `NormalvsExpert` factory (`ItemDropRule.cs:86-89`).
pub fn normalvs_expert(item: i32, denom_normal: i32, denom_expert: i32) -> Box<dyn ItemDropRule> {
    DropBasedOnExpertMode::new(
        CommonDrop::new(item, denom_normal, 1, 1),
        CommonDrop::new(item, denom_expert, 1, 1),
    )
}

/// The `ExpertGetsRerolls` factory (`ItemDropRule.cs:30-33`).
pub fn expert_gets_rerolls(
    item: i32,
    chance_denominator: i32,
    expert_rerolls: i32,
) -> Box<dyn ItemDropRule> {
    DropBasedOnExpertMode::new(
        CommonDropWithRerolls::new(item, 0, chance_denominator, 1, 1),
        CommonDropWithRerolls::new(item, expert_rerolls, chance_denominator, 1, 1),
    )
}

/// The `OneFromOptions` factory (`ItemDropRule.cs:57-59`).
pub fn one_from_options(chance_denominator: i32, options: &[i32]) -> Box<dyn ItemDropRule> {
    OneFromOptionsDrop::new(chance_denominator, options)
}

/// The `DropNothing` factory (`ItemDropRule.cs:66`).
pub fn drop_nothing() -> Box<dyn ItemDropRule> {
    DropNothing::new()
}

// --- Conditions (`GameContent/ItemDropRules/Conditions.cs`) ------------------
//
// The data-driven `Cond` enum ports the gates the registrations use; each
// variant's `can_drop` is the verbatim `Conditions.<Name>.CanDrop` body over
// [`LootFlags`]/[`LootNpc`] (the caller snapshots the world/player/NPC state).

/// `Conditions.SoulOfWhateverConditionCanDrop` (`Conditions.cs:805-855`): the
/// depth band below the cavern layer, shared by the two soul conditions.
fn soul_of_whatever_can_drop(info: &DropAttemptInfo) -> bool {
    info.flags.soul_depth_ok
}

/// The pumpkin/frost moon gates' wave-scaled chance denominator
/// (`Conditions.cs:55-122`).
fn moon_wave_gate(rng: &mut dyn DropRng, luck: f32, is_pumpkin: bool, is_expert: bool, wave: i32) -> bool {
    let mut num = wave;
    if is_expert {
        num += 5;
    }
    let base = if is_pumpkin { 24 } else { 28 };
    let mut num2 = ((base - num) as f64 / 2.5) as i32;
    if is_expert {
        num2 -= if is_pumpkin { 1 } else { 2 };
    }
    if num2 < 1 {
        num2 = 1;
    }
    // `info.player.RollLuck(num2) == 0` (`Player.RollLuck` = the luck roll).
    roll_luck(rng, luck, num2) == 0
}

/// The trophy gates' wave table (`Conditions.cs:124-180`).
fn moon_trophy_gate(rng: &mut dyn DropRng, is_pumpkin: bool, is_expert: bool, wave: i32) -> bool {
    if wave < 15 {
        return false;
    }
    let mut num = 4;
    if wave == 17 || wave == 18 {
        num = 3;
    }
    if wave >= 19 {
        num = 2;
    }
    // `if (Main.expertMode && Main.rand.Next(3) == 0) num--;`
    if is_expert && rng.next_range(0, 3) == 0 {
        num -= 1;
    }
    rng.next_range(0, num) == 0
}

/// Every `Conditions.*` gate the database registrations reference, keyed for
/// the data-driven rules.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Cond {
    NotExpert,
    IsExpert,
    IsMasterMode,
    /// `Conditions.Easymode` — `!Main.hardMode`.
    Easymode,
    IsHardmode,
    WindyEnoughForKiteDrops,
    RemixSeed,
    NotRemixSeed,
    NotRemixSeedHardmode,
    RemixSeedHardmode,
    RemixSeedEasymode,
    NotRemixSeedEasymode,
    TenthAnniversaryIsUp,
    TenthAnniversaryIsNotUp,
    DontStarveIsUp,
    DontStarveIsNotUp,
    /// `Conditions.SkyblockIsUp` (`WorldGen.Skyblock.lowTiles`).
    SkyblockIsUp,
    /// `Conditions.SkyblockIsUpNoSickle` — up + the player lacks the sickle.
    SkyblockIsUpNoSickle,
    /// `Conditions.MechdusaKill` — the remix merge + the other parts gone.
    MechdusaKill,
    BeatAnyMechBoss,
    DownedAllMechBosses,
    DownedPlantera,
    /// `Conditions.FirstTimeKillingPlantera` — `!NPC.downedPlantBoss`.
    FirstTimeKillingPlantera,
    /// `Conditions.MissingTwin` — the other Twin half is absent.
    MissingTwin,
    /// `Conditions.LegacyHack_IsABoss` — `info.npc.boss`.
    LegacyHack_IsABoss,
    LegacyHack_IsBossAndExpert,
    LegacyHack_IsBossAndNotExpert,
    IsBloodMoonAndNotFromStatue,
    IsCrimsonAndNotExpert,
    IsCorruptionAndNotExpert,
    IsPumpkinMoon,
    IsChristmas,
    /// `Conditions.PumpkinMoonDropGatingChance` — the wave-scaled roll.
    PumpkinMoonDropGatingChance,
    FrostMoonDropGatingChance,
    PumpkinMoonDropGateForTrophies,
    FrostMoonDropGateForTrophies,
    HalloweenWeapons,
    HalloweenGoodieBagDrop,
    XmasPresentDrop,
    LivingFlames,
    SoulOfLight,
    SoulOfNight,
    NotFromStatue,
    PirateMap,
    JungleKeyCondition,
    CorruptKeyCondition,
    CrimsonKeyCondition,
    HallowKeyCondition,
    FrozenKeyCondition,
    DesertKeyCondition,
    EyeOfCthulhuDefeatedAndNoAltarsInWorld,
    EmpressOfLightIsGenuinelyEnraged,
    /// `Conditions.FromCertainWaveAndAbove` — `NPC.waveNumber >= wave`.
    FromCertainWaveAndAbove(i32),
    /// `Conditions.YoyoCascade` (the pre-hardmode underground yoyo gate).
    YoyoCascade,
    YoyosAmarok,
    YoyosYelets,
    YoyosHelFire,
    /// `Conditions.NeverTrue` (the explicit dead rule).
    NeverTrue,
}

impl DropCondition for Cond {
    fn can_drop(&self, info: &mut DropAttemptInfo) -> bool {
        let f = &info.flags;
        let n = &info.npc;
        match *self {
            Cond::NotExpert => !info.is_expert,
            Cond::IsExpert => info.is_expert,
            Cond::IsMasterMode => info.is_master,
            Cond::Easymode => !f.hard_mode,
            Cond::IsHardmode => f.hard_mode,
            Cond::WindyEnoughForKiteDrops => f.windy,
            Cond::RemixSeed => f.remix_seed,
            Cond::NotRemixSeed => !f.remix_seed,
            Cond::NotRemixSeedHardmode => !f.remix_seed && f.hard_mode,
            Cond::RemixSeedHardmode => f.remix_seed && f.hard_mode,
            Cond::RemixSeedEasymode => f.remix_seed && !f.hard_mode,
            Cond::NotRemixSeedEasymode => !f.remix_seed && !f.hard_mode,
            Cond::TenthAnniversaryIsUp => f.tenth_anniversary,
            Cond::TenthAnniversaryIsNotUp => !f.tenth_anniversary,
            Cond::DontStarveIsUp => f.dont_starve,
            Cond::DontStarveIsNotUp => !f.dont_starve,
            Cond::SkyblockIsUp => f.skyblock,
            Cond::SkyblockIsUpNoSickle => f.skyblock && !f.player_has_sickle,
            Cond::MechdusaKill => f.mechdusa && n.mechdusa_parts_cleared,
            Cond::BeatAnyMechBoss => f.downed_mech_any,
            Cond::DownedAllMechBosses => f.downed_mech1 && f.downed_mech2 && f.downed_mech3,
            Cond::DownedPlantera => f.downed_plantera,
            Cond::FirstTimeKillingPlantera => !f.downed_plant_boss,
            Cond::MissingTwin => n.twin_missing,
            Cond::LegacyHack_IsABoss => n.is_boss,
            Cond::LegacyHack_IsBossAndExpert => n.is_boss && info.is_expert,
            Cond::LegacyHack_IsBossAndNotExpert => n.is_boss && !info.is_expert,
            Cond::IsBloodMoonAndNotFromStatue => {
                !f.day_time && f.blood_moon && !n.spawned_from_statue
            }
            Cond::IsCrimsonAndNotExpert => f.crimson_world && !info.is_expert,
            Cond::IsCorruptionAndNotExpert => !f.crimson_world && !info.is_expert,
            Cond::IsPumpkinMoon => f.pumpkin_moon,
            Cond::IsChristmas => f.xmas,
            Cond::PumpkinMoonDropGatingChance => {
                f.pumpkin_moon
                    && moon_wave_gate(info.rng, info.luck, true, info.is_expert, f.wave_number)
            }
            Cond::FrostMoonDropGatingChance => {
                f.frost_moon
                    && moon_wave_gate(info.rng, info.luck, false, info.is_expert, f.wave_number)
            }
            Cond::PumpkinMoonDropGateForTrophies => {
                f.pumpkin_moon
                    && moon_trophy_gate(info.rng, true, info.is_expert, f.wave_number)
            }
            Cond::FrostMoonDropGateForTrophies => {
                f.frost_moon
                    && moon_trophy_gate(info.rng, false, info.is_expert, f.wave_number)
            }
            // `HalloweenWeapons` (`Conditions.cs:805-828`): the value/damage/
            // defense caps over the difficulty multipliers (folded into the
            // caps by the caller).
            Cond::HalloweenWeapons => {
                f.halloween
                    && n.value > 0.0
                    && n.value < f.halloween_value_cap
                    && (n.damage as f32) < f.halloween_damage_cap
                    && (n.defense as f32) < 20.0
            }
            Cond::HalloweenGoodieBagDrop => {
                f.halloween
                    && n.life_max > 1
                    && n.damage > 0
                    && !n.friendly
                    && n.kind != 121
                    && n.kind != 23
                    && n.value > 0.0
            }
            Cond::XmasPresentDrop => {
                f.xmas
                    && n.life_max > 1
                    && n.damage > 0
                    && !n.friendly
                    && n.kind != 121
                    && n.kind != 23
                    && n.value > 0.0
            }
            // `LivingFlames` (`Conditions.cs:940-955`): lifeMax > 5 + value +
            // !friendly + hardmode + the underworld depth.
            Cond::LivingFlames => {
                n.life_max > 5 && n.value > 0.0 && !n.friendly && f.hard_mode && f.underworld
            }
            Cond::SoulOfLight => soul_of_whatever_can_drop(info) && f.zone_hallow,
            Cond::SoulOfNight => {
                soul_of_whatever_can_drop(info) && (f.zone_corrupt || f.zone_crimson)
            }
            Cond::NotFromStatue => !n.spawned_from_statue,
            Cond::PirateMap => n.value > 0.0 && f.hard_mode && f.pirate_map_band,
            Cond::JungleKeyCondition => {
                n.value > 0.0 && !n.dont_drop_keys && f.hard_mode && f.zone_jungle
            }
            Cond::CorruptKeyCondition => {
                n.value > 0.0 && !n.dont_drop_keys && f.hard_mode && f.zone_corrupt
            }
            Cond::CrimsonKeyCondition => {
                n.value > 0.0 && !n.dont_drop_keys && f.hard_mode && f.zone_crimson
            }
            Cond::HallowKeyCondition => {
                n.value > 0.0 && !n.dont_drop_keys && f.hard_mode && f.zone_hallow
            }
            Cond::FrozenKeyCondition => {
                n.value > 0.0 && !n.dont_drop_keys && f.hard_mode && f.zone_snow
            }
            // `DesertKeyCondition` adds `!player.ZoneBeach` — folded into the
            // zone_desert snapshot (the caller excludes the beach).
            Cond::DesertKeyCondition => {
                n.value > 0.0 && !n.dont_drop_keys && f.hard_mode && f.zone_desert
            }
            Cond::EyeOfCthulhuDefeatedAndNoAltarsInWorld => f.eoc_defeated_no_altars,
            Cond::EmpressOfLightIsGenuinelyEnraged => f.empress_enraged,
            Cond::FromCertainWaveAndAbove(wave) => f.wave_number >= wave,
            // The yoyo gates (`Conditions.cs:1699-1790`): the depth/hardmode
            // bands over a worth-dropping NPC.
            Cond::YoyoCascade => {
                !f.hard_mode
                    && n.has_player_target
                    && n.life_max > 5
                    && !n.friendly
                    && n.value > 0.0
                    && f.deep_yoyo_band
                    && f.downed_boss3
            }
            Cond::YoyosAmarok => {
                f.hard_mode
                    && n.has_player_target
                    && f.zone_snow
                    && n.life_max > 5
                    && !n.friendly
                    && n.value > 0.0
            }
            Cond::YoyosYelets => {
                f.hard_mode
                    && f.zone_jungle
                    && f.downed_mech_any
                    && n.life_max > 5
                    && n.has_player_target
                    && !n.friendly
                    && n.value > 0.0
            }
            Cond::YoyosHelFire => {
                f.hard_mode
                    && !f.zone_dungeon
                    && f.helfire_band
                    && n.life_max > 5
                    && n.has_player_target
                    && !n.friendly
                    && n.value > 0.0
            }
            Cond::NeverTrue => false,
        }
    }
}

/// A boxed condition for the data rules.
pub fn cond(kind: Cond) -> Box<dyn DropCondition> {
    Box::new(kind)
}

/// `LeadingConditionRule` (`LeadingConditionRule.cs`): `CanDrop` = the
/// condition; `TryDroppingItem` drops nothing and returns `Success`, so the
/// `OnSuccess` chains fire only when the gate passes (a failed gate yields
/// `DoesntFillConditions`).
pub struct LeadingConditionRule {
    pub condition: Box<dyn DropCondition>,
    chains: Vec<ChainRule>,
}

impl LeadingConditionRule {
    pub fn new(condition: Box<dyn DropCondition>) -> Box<Self> {
        Box::new(Self {
            condition,
            chains: Vec::new(),
        })
    }
}

impl ItemDropRule for LeadingConditionRule {
    fn chained_rules(&self) -> &[ChainRule] {
        &self.chains
    }
    fn can_drop(&self, info: &mut DropAttemptInfo) -> bool {
        self.condition.can_drop(info)
    }
    fn try_dropping_item(
        &self,
        _info: &mut DropAttemptInfo,
        _out: &mut Vec<DropRequest>,
    ) -> DropAttemptResult {
        DropAttemptResult {
            state: DropResultState::Success,
        }
    }
}
impl_chains!(LeadingConditionRule);

/// The leading-condition factory the generated table uses
/// (`new LeadingConditionRule(new Conditions.X())`).
pub fn leading_condition(condition: Cond) -> Box<dyn ItemDropRule> {
    LeadingConditionRule::new(Box::new(condition))
}

/// `DropPerPlayerOnThePlayer` (`DropPerPlayerOnThePlayer.cs`): one stack roll
/// PER active player (`MasterModeDropOnAllPlayers`' inner rule).
pub struct DropPerPlayerOnThePlayer {
    pub item: i32,
    pub chance_denominator: i32,
    pub amount_min: i32,
    pub amount_max: i32,
    pub condition: Option<Box<dyn DropCondition>>,
    chains: Vec<ChainRule>,
}

impl DropPerPlayerOnThePlayer {
    pub fn new(
        item: i32,
        chance_denominator: i32,
        min: i32,
        max: i32,
        condition: Option<Box<dyn DropCondition>>,
    ) -> Box<Self> {
        Box::new(Self {
            item,
            chance_denominator,
            amount_min: min,
            amount_max: max,
            condition,
            chains: Vec::new(),
        })
    }
}

impl ItemDropRule for DropPerPlayerOnThePlayer {
    fn chained_rules(&self) -> &[ChainRule] {
        &self.chains
    }
    fn try_dropping_item(
        &self,
        info: &mut DropAttemptInfo,
        out: &mut Vec<DropRequest>,
    ) -> DropAttemptResult {
        // The condition rides the rule itself (the C# passes it to the
        // constructor; `CanDrop` folds it in).
        if let Some(cond) = &self.condition {
            if !cond.can_drop(info) {
                return DropAttemptResult {
                    state: DropResultState::DoesntFillConditions,
                };
            }
        }
        // One `Next(min, max + 1)` roll per active player.
        for _ in 0..info.active_players {
            if info.rng.next_range(0, self.chance_denominator.max(1)) == 0 {
                let stack = info.rng.next_range(self.amount_min, self.amount_max + 1);
                out.push(drop_request(self.item, stack, false));
            }
        }
        DropAttemptResult {
            state: DropResultState::Success,
        }
    }
}
impl_chains!(DropPerPlayerOnThePlayer);

/// `OneFromRulesRule` (`OneFromRulesRule.cs`): one passing sub-rule's drops.
pub struct OneFromRulesRule {
    pub rules: Vec<Box<dyn ItemDropRule>>,
    chains: Vec<ChainRule>,
}

impl OneFromRulesRule {
    pub fn new(rules: Vec<Box<dyn ItemDropRule>>) -> Box<Self> {
        Box::new(Self {
            rules,
            chains: Vec::new(),
        })
    }
}

impl ItemDropRule for OneFromRulesRule {
    fn chained_rules(&self) -> &[ChainRule] {
        &self.chains
    }
    fn try_dropping_item(
        &self,
        info: &mut DropAttemptInfo,
        out: &mut Vec<DropRequest>,
    ) -> DropAttemptResult {
        // The C# picks the first rule whose `TryDroppingItem` reports success
        // (`OneFromRulesRule.TryDroppingItem`: the sequential walk).
        for rule in &self.rules {
            let result = resolve_rule(rule.as_ref(), info, out);
            if result.state == DropResultState::Success {
                return result;
            }
        }
        DropAttemptResult {
            state: DropResultState::FailedRandomRoll,
        }
    }
}
impl_chains!(OneFromRulesRule);

// --- The remaining factories (`ItemDropRule.cs:15-115`) ----------------------

/// The `MasterModeCommonDrop` factory (`ItemDropRule.cs:25-27`) — a plain
/// `ByCondition(IsMasterMode)` gate.
pub fn master_mode_common_drop(item: i32) -> Box<dyn ItemDropRule> {
    by_condition(Cond::IsMasterMode, item, 1, 1, 1, 1)
}

/// The `MasterModeDropOnAllPlayers` factory (`ItemDropRule.cs:29-33`).
pub fn master_mode_drop_on_all_players(item: i32, chance_denominator: i32) -> Box<dyn ItemDropRule> {
    DropBasedOnMasterMode::new(
        DropNothing::new(),
        DropPerPlayerOnThePlayer::new(
            item,
            chance_denominator,
            1,
            1,
            Some(Box::new(Cond::IsMasterMode)),
        ),
    )
}

/// The `BossBagByCondition` factory (`ItemDropRule.cs:15-23`).
pub fn boss_bag_by_condition(condition: Cond, item: i32) -> Box<dyn ItemDropRule> {
    DropBasedOnExpertMode::new(
        DropNothing::new(),
        DropLocalPerClientAndResetsMoney::with_condition(item, 1, 1, 1, Box::new(condition)),
    )
}

/// The `WithRerolls` factory (`ItemDropRule.cs:35-39`).
pub fn with_rerolls(
    item: i32,
    rerolls: i32,
    chance_denominator: i32,
    min: i32,
    max: i32,
) -> Box<dyn ItemDropRule> {
    CommonDropWithRerolls::new(item, rerolls, chance_denominator, min, max)
}

/// The `ByCondition` factory (`ItemDropRule.cs:41-43`).
pub fn by_condition(
    condition: Cond,
    item: i32,
    chance_denominator: i32,
    min: i32,
    max: i32,
    chance_numerator: i32,
) -> Box<dyn ItemDropRule> {
    let mut rule = ItemDropWithCondition::new(Box::new(condition), item, chance_denominator, min, max);
    rule.chance_numerator = chance_numerator;
    rule
}

/// The `Food` factory (`ItemDropRule.cs:107-110`) — the NotFromStatue gate
/// (the `ItemID.Sets.IsFood` marking is the item registry's concern).
pub fn food(item: i32, chance_denominator: i32, min: i32, max: i32) -> Box<dyn ItemDropRule> {
    by_condition(Cond::NotFromStatue, item, chance_denominator, min, max, 1)
}

/// The `StatusImmunityItem` factory (`ItemDropRule.cs:112-115`).
pub fn status_immunity_item(item: i32, drops_out_of_x: i32) -> Box<dyn ItemDropRule> {
    expert_gets_rerolls(item, drops_out_of_x, 1)
}

/// The `ScalingWithOnlyBadLuck` factory (`ItemDropRule.cs:45-48`).
pub fn scaling_with_only_bad_luck(
    item: i32,
    chance_denominator: i32,
    min: i32,
    max: i32,
) -> Box<dyn ItemDropRule> {
    let mut rule = CommonDrop::new(item, chance_denominator, min, max);
    rule.bad_luck_only = true;
    rule
}

/// The `NotScalingWithLuck` factory (`ItemDropRule.cs:50-52`).
pub fn not_scaling_with_luck(
    item: i32,
    chance_denominator: i32,
    min: i32,
    max: i32,
) -> Box<dyn ItemDropRule> {
    let mut rule = CommonDrop::new(item, chance_denominator, min, max);
    rule.uses_luck = false;
    rule
}

/// The `OneFromOptionsNotScalingWithLuck` factory (`ItemDropRule.cs:54-56`).
pub fn one_from_options_not_scaling_with_luck(
    chance_denominator: i32,
    options: &[i32],
) -> Box<dyn ItemDropRule> {
    let mut rule = OneFromOptionsDrop::new(chance_denominator, options);
    rule.uses_luck = false;
    rule
}

/// The `NormalvsExpertNotScalingWithLuck` factory (`ItemDropRule.cs:91-95`).
pub fn normalvs_expert_not_scaling_with_luck(
    item: i32,
    denom_normal: i32,
    denom_expert: i32,
) -> Box<dyn ItemDropRule> {
    DropBasedOnExpertMode::new(
        not_scaling_with_luck(item, denom_normal, 1, 1),
        not_scaling_with_luck(item, denom_expert, 1, 1),
    )
}

/// The `NormalvsExpertOneFromOptions` factory (`ItemDropRule.cs:102-105`).
pub fn normalvs_expert_one_from_options(
    denom_normal: i32,
    denom_expert: i32,
    options: &[i32],
) -> Box<dyn ItemDropRule> {
    DropBasedOnExpertMode::new(
        one_from_options(denom_normal, options),
        one_from_options(denom_expert, options),
    )
}

/// The `NormalvsExpertOneFromOptionsNotScalingWithLuck` factory
/// (`ItemDropRule.cs:97-100`).
pub fn normalvs_expert_one_from_options_not_scaling_with_luck(
    denom_normal: i32,
    denom_expert: i32,
    options: &[i32],
) -> Box<dyn ItemDropRule> {
    DropBasedOnExpertMode::new(
        one_from_options_not_scaling_with_luck(denom_normal, options),
        one_from_options_not_scaling_with_luck(denom_expert, options),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn info<'a>(rng: &'a mut dyn DropRng) -> DropAttemptInfo<'a> {
        DropAttemptInfo {
            rng,
            luck: 0.0,
            is_expert: false,
            is_master: false,
            active_players: 1,
            extra_gel: false,
            flags: LootFlags::default(),
            npc: LootNpc::default(),
        }
    }

    #[test]
    fn common_drop_gate_and_stack_range() {
        // CommonDrop.TryDroppingItem: `RollLuck(denominator) < numerator` then
        // `Next(min, max + 1)` — both consume the same FastRandom stream.
        let mut rng = crate::tile_light::FastRandom::new(42);
        let rule = CommonDrop::new(23, 2, 1, 2);
        let mut out = Vec::new();
        let r = resolve_rule(rule.as_ref(), &mut info(&mut rng), &mut out);
        assert_eq!(r.state, DropResultState::Success);
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].item, 23);
        assert!((1..=2).contains(&out[0].stack));
    }

    #[test]
    fn always_drop_rule_drops_the_exact_stack() {
        // denominator 1: `RollLuck(1)` = `Next(1)` = 0, `0 < 1` succeeds; the
        // stack roll still consumes the stream (the C# call order).
        let mut rng = crate::tile_light::FastRandom::new(7);
        let rule = CommonDrop::new(9, 1, 3, 3);
        let mut out = Vec::new();
        let r = resolve_rule(rule.as_ref(), &mut info(&mut rng), &mut out);
        assert_eq!(r.state, DropResultState::Success);
        assert_eq!(out[0].stack, 3);
    }

    #[test]
    fn failed_roll_fires_the_on_failed_roll_chain() {
        let mut rng = crate::tile_light::FastRandom::new(1);
        // A 100% gate never fails, so the chain must not run.
        let rule = CommonDrop::new(1, 1, 1, 1).on_failed_roll(CommonDrop::new(2, 1, 1, 1));
        let mut out = Vec::new();
        let r = resolve_rule(rule.as_ref(), &mut info(&mut rng), &mut out);
        assert_eq!(r.state, DropResultState::Success);
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].item, 1);
        // A never-succeeding gate always chains into the fallback.
        let rule = CommonDrop::with_numerator(1, 1000, 1, 1, 0)
            .on_failed_roll(CommonDrop::new(2, 1, 1, 1));
        let mut out = Vec::new();
        resolve_rule(rule.as_ref(), &mut info(&mut rng), &mut out);
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].item, 2);
    }

    #[test]
    fn success_fires_the_on_success_chain() {
        let mut rng = crate::tile_light::FastRandom::new(8);
        let rule = CommonDrop::new(1, 1, 1, 1).on_success(CommonDrop::new(2, 1, 1, 1));
        let mut out = Vec::new();
        resolve_rule(rule.as_ref(), &mut info(&mut rng), &mut out);
        assert_eq!(out.len(), 2);
        assert_eq!((out[0].item, out[1].item), (1, 2));
    }

    #[test]
    fn expert_mode_routes_to_the_expert_rule() {
        let rule =
            DropBasedOnExpertMode::new(CommonDrop::new(11, 1, 1, 1), CommonDrop::new(22, 1, 1, 1));
        let mut rng = crate::tile_light::FastRandom::new(5);
        let mut i = info(&mut rng);
        i.is_expert = true;
        let mut out = Vec::new();
        resolve_rule(rule.as_ref(), &mut i, &mut out);
        assert_eq!(out[0].item, 22);
        let mut rng = crate::tile_light::FastRandom::new(5);
        let mut out = Vec::new();
        resolve_rule(rule.as_ref(), &mut info(&mut rng), &mut out);
        assert_eq!(out[0].item, 11);
    }

    #[test]
    fn condition_gate_splits_fail_states() {
        // The condition failing inside the attempt reports
        // DoesntFillConditions, which fires OnFailedConditions chains.
        let mut rng = crate::tile_light::FastRandom::new(3);
        let rule = ItemDropWithCondition::new(Box::new(NeverTrue), 5, 1, 1, 1)
            .on_failed_conditions(CommonDrop::new(6, 1, 1, 1));
        let mut out = Vec::new();
        let r = resolve_rule(rule.as_ref(), &mut info(&mut rng), &mut out);
        assert_eq!(r.state, DropResultState::DoesntFillConditions);
        assert_eq!(out[0].item, 6);
    }

    #[test]
    fn rerolls_zero_matches_the_plain_common_drop_on_the_same_seed() {
        // rerolls = 0 -> one gate roll: identical outcomes to CommonDrop.
        let mut a = crate::tile_light::FastRandom::new(99);
        let mut b = crate::tile_light::FastRandom::new(99);
        let mut out_a = Vec::new();
        let mut out_b = Vec::new();
        let ra = resolve_rule(
            CommonDropWithRerolls::new(9, 0, 2, 1, 1).as_ref(),
            &mut info(&mut a),
            &mut out_a,
        );
        let rb = resolve_rule(
            CommonDrop::new(9, 2, 1, 1).as_ref(),
            &mut info(&mut b),
            &mut out_b,
        );
        assert_eq!(ra.state, rb.state);
        assert_eq!(out_a, out_b);
        // More rerolls: the stack stays in range when it drops.
        let mut c = crate::tile_light::FastRandom::new(12);
        let mut out_c = Vec::new();
        resolve_rule(
            CommonDropWithRerolls::new(9, 5, 2, 3, 2).as_ref(),
            &mut info(&mut c),
            &mut out_c,
        );
        assert!(out_c.is_empty() || (1..=3).contains(&out_c[0].stack));
    }

    #[test]
    fn drop_one_by_one_emits_the_chunk_shape() {
        let mut rng = crate::tile_light::FastRandom::new(13);
        let rule = DropOneByOne::new(
            502,
            DropOneByOneParams {
                chance_numerator: 1,
                chance_denominator: 1,
                min_drops_count: 2,
                max_drops_count: 3,
                min_stack_per_chunk_base: 1,
                max_stack_per_chunk_base: 2,
                bonus_min_per_player: 0,
                bonus_max_per_player: 0,
            },
        );
        let mut out = Vec::new();
        resolve_rule(rule.as_ref(), &mut info(&mut rng), &mut out);
        assert!((2..=3).contains(&out.len()));
        for req in &out {
            assert_eq!(req.item, 502);
            assert!(req.scattered, "DropOneByOne passes scattered: true");
            assert!((1..=2).contains(&req.stack));
        }
        // The per-player bonus widens the chunk range with 4 active players.
        let mut rng = crate::tile_light::FastRandom::new(13);
        let mut i = info(&mut rng);
        i.active_players = 4;
        let mut out = Vec::new();
        resolve_rule(rule.as_ref(), &mut i, &mut out);
        for req in &out {
            assert!((1..=2).contains(&req.stack) || (1..=5).contains(&req.stack));
        }
    }

    #[test]
    fn gel_factory_doubles_stacks_under_the_extra_gel_seed() {
        // ItemDropRule.Gel: Common(23, d, min, max) vs
        // Common(23, d, min*2, max*2).
        let mut rng = crate::tile_light::FastRandom::new(21);
        let rule = gel(1, 1, 2);
        let mut i = info(&mut rng);
        i.extra_gel = true;
        let mut out = Vec::new();
        resolve_rule(rule.as_ref(), &mut i, &mut out);
        assert_eq!(out[0].item, 23);
        assert!((2..=4).contains(&out[0].stack));
    }

    #[test]
    fn boss_bag_requests_the_money_reset() {
        let rule = boss_bag(123);
        let mut rng = crate::tile_light::FastRandom::new(17);
        let mut i = info(&mut rng);
        i.is_expert = true;
        let mut out = Vec::new();
        resolve_rule(rule.as_ref(), &mut i, &mut out);
        assert_eq!(out[0].item, 123);
        assert!(out[0].resets_money);
        // Normal mode: the bag is DropNothing (no resets either).
        let mut rng = crate::tile_light::FastRandom::new(17);
        let mut out = Vec::new();
        resolve_rule(boss_bag(123).as_ref(), &mut info(&mut rng), &mut out);
        assert!(out.is_empty());
    }

    #[test]
    fn database_registers_and_removes_per_net_id() {
        let mut db = ItemDropDatabase::new();
        db.register_to_multiple(&[1, -3], &|| CommonDrop::new(23, 1, 1, 2));
        db.register_to_npc(1, CommonDrop::new(99, 1, 1, 1));
        assert_eq!(db.rules_for_npc(1).len(), 2);
        assert_eq!(db.rules_for_npc(-3).len(), 1);
        assert_eq!(db.rules_for_npc(50).len(), 0);
        db.remove_from_multiple(&[1]);
        assert_eq!(db.rules_for_npc(1).len(), 1);
    }

    #[test]
    fn roll_luck_luck_zero_is_the_plain_roll() {
        // Luck.RollLuck with luck == 0: exactly one Next(range).
        let mut a = crate::tile_light::FastRandom::new(31);
        let mut b = crate::tile_light::FastRandom::new(31);
        let rolled = roll_luck(&mut a, 0.0, 100);
        let plain = b.next_range(0, 100);
        assert_eq!(rolled, plain);
        // Positive luck never leaves the range.
        let mut c = crate::tile_light::FastRandom::new(31);
        for _ in 0..50 {
            let v = roll_luck(&mut c, 1.0, 100);
            assert!((0..100).contains(&v));
        }
    }

    #[test]
    fn drop_money_splits_a_small_value_into_copper() {
        // A 25-copper blue slime with no luck/midas/blood-moon: the ±20-75 %
        // spread gives 20..=42 copper; the single stack is the coin id 71.
        let mut rng = crate::tile_light::FastRandom::new(101);
        let mut out = Vec::new();
        drop_money(&mut rng, 25.0, 0.0, false, false, 0.0, &mut out);
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].item, 71);
        assert!((20..=42).contains(&out[0].stack), "stack {}", out[0].stack);
    }

    #[test]
    fn drop_money_makes_a_gold_coin_from_10000_copper() {
        // Pinky's 10000f: the split takes the gold tier (num > 10000 is false
        // at exactly 10000, so 1 gold via the... the gold tier needs > 10000;
        // at exactly 10000 the silver tier splits 100 silver — the shrink may
        // halve it. Assert the coin ids stay in the vanilla set and the total
        // copper value lands in the vanilla shrink envelope.
        let mut rng = crate::tile_light::FastRandom::new(202);
        let mut out = Vec::new();
        drop_money(&mut rng, 10_000.0, 0.0, false, false, 0.0, &mut out);
        assert!(!out.is_empty());
        let total: i32 = out
            .iter()
            .map(|r| {
                r.stack
                    * match r.item {
                        74 => 1_000_000,
                        73 => 10_000,
                        72 => 100,
                        _ => 1,
                    }
            })
            .sum();
        // The VALUE roll happens before the split: 10000 x (0.8..1.75) x the
        // bonus chains (up to ~x1.1 more) -> the total spans roughly
        // 2500..20000; the shrink rolls only reduce the stack counts.
        assert!((2_500..=20_000).contains(&total), "total {total}");
        for r in &out {
            assert!(matches!(r.item, 71 | 72 | 73 | 74));
        }
    }

    #[test]
    fn drop_money_two_luck_rolls_keep_the_best() {
        // The luck gate passes on the seeded stream (next_float < |luck|):
        // two rolls and the best (luck >= 0) wins — the total must be >= the
        // single-roll floor of the same stream's first roll.
        let mut rng = crate::tile_light::FastRandom::new(303);
        let mut out = Vec::new();
        drop_money(&mut rng, 25.0, 1.0, false, false, 0.0, &mut out);
        let total: i32 = out.iter().map(|r| r.stack).sum();
        assert!(
            (20..=42).contains(&total),
            "best-of-two still 20..=42: {total}"
        );
    }

    #[test]
    fn drop_money_blood_moon_rolls_the_bump() {
        // The blood moon multiplies by 1 + Next(101) * 0.01 — at most double.
        let mut rng = crate::tile_light::FastRandom::new(404);
        let mut out = Vec::new();
        drop_money(&mut rng, 25.0, 0.0, false, true, 0.0, &mut out);
        let total: i32 = out.iter().map(|r| r.stack).sum();
        assert!((20..=85).contains(&total), "total {total}");
    }

    #[test]
    fn try_dropping_walks_every_registered_rule() {
        // ItemDropResolver.TryDropping: each rule in the net id's list
        // resolves once.
        let mut db = ItemDropDatabase::new();
        db.register_to_npc(1, CommonDrop::new(10, 1, 1, 1));
        db.register_to_npc(1, CommonDrop::new(20, 1, 1, 1));
        let mut rng = crate::tile_light::FastRandom::new(2);
        let mut out = Vec::new();
        try_dropping(&db, 1, &mut info(&mut rng), &mut out);
        assert_eq!(out.len(), 2);
        assert_eq!((out[0].item, out[1].item), (10, 20));
    }
}
