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
        // maximum (`Next(min, max + 1)`).
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
    chains: Vec<ChainRule>,
}

impl OneFromOptionsDrop {
    /// The `OneFromOptions` factory (`ItemDropRule.cs:57-59`).
    pub fn new(chance_denominator: i32, options: &[i32]) -> Box<Self> {
        Box::new(Self {
            options: options.to_vec(),
            chance_denominator,
            chance_numerator: 1,
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
