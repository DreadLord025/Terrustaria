//! The town NPC arrival gates — `Main.UpdateTime_SpawnTownNPCs`
//! (`Main.cs:65542-66116`).
//!
//! The vanilla checks every `7200 / WorldGen.GetWorldUpdateRate()` ticks
//! (`:65548-65555`): it counts the live residents of each town type, clears
//! `Main.townNPCCanSpawn`, sets one `canSpawn` entry per kind whose condition
//! holds *and* which has no resident yet (`:65785-65955`), and then picks the
//! first entry of a fixed priority ladder into
//! `WorldGen.prioritizedTownNPCType` (`:65957-66116`). The consumer
//! (`WorldGen.IsThereASpawnablePrioritizedTownNPC`, `WorldGen.cs:4880-4920`)
//! walks the `canSpawn` table in type order and returns the first kind whose
//! room is ready, preferring `prioritizedTownNPCType`.
//!
//! The port used to fold this whole machine into a per-2-second block of
//! ~25 separate `if` gates with per-kind magic spawn offsets
//! (`world_sim/npcs.rs`), which could not express the vanilla's ordering, its
//! two-minute cadence or its "one arrival per check" rule. This module is the
//! verbatim port: [`update_time_spawn_town_npcs`] is the C# function, and the
//! caller only places the returned kind (`WorldGen.SpawnTownNPC`).

use crate::item::ItemSlot;
use crate::npc::{Npc, NpcKind};
use tr_core::world::World;

/// `NPCID.Count` (`Main.townNPCCanSpawn`'s length).
pub(crate) const NPCID_COUNT: usize = 697;

/// The town kinds the port can actually place, in the exact order of the C#
/// priority ladder (`Main.cs:65957-66116`). Every entry beyond the ladder's
/// first few is only reachable once its own condition holds.
pub(crate) const LADDER: [NpcKind; 21] = [
    NpcKind::Guide,
    NpcKind::Merchant,
    NpcKind::Nurse,
    NpcKind::ArmsDealer,
    NpcKind::GoblinTinkerer,
    NpcKind::Wizard,
    NpcKind::Dryad,
    NpcKind::Demolitionist,
    NpcKind::WitchDoctor,
    NpcKind::Mechanic,
    NpcKind::Angler,
    NpcKind::Clothier,
    NpcKind::Stylist,
    NpcKind::DyeTrader,
    NpcKind::Painter,
    NpcKind::PartyGirl,
    NpcKind::Golfer,
    NpcKind::TaxCollector,
    NpcKind::Zoologist,
    NpcKind::Princess,
    NpcKind::TravellingMerchant,
];

/// `Main.checkForSpawns`, `Main.townNPCCanSpawn` and
/// `WorldGen.prioritizedTownNPCType` — the three statics the C# function
/// carries between ticks.
#[derive(Debug)]
pub(crate) struct ArrivalState {
    /// `Main.checkForSpawns`: the tick counter against the 7200 gate.
    pub(crate) check_for_spawns: i32,
    /// `WorldGen.prioritizedTownNPCType`, as a kind.
    pub(crate) prioritized: Option<NpcKind>,
    /// `Main.townNPCCanSpawn`, indexed by [`LADDER`] position so the table
    /// stays checkable in the ladder's order (`NpcKind` has no stable index).
    pub(crate) can_spawn: [bool; LADDER.len()],
}

impl Default for ArrivalState {
    fn default() -> Self {
        Self::new()
    }
}

impl ArrivalState {
    pub(crate) fn new() -> Self {
        Self {
            check_for_spawns: 0,
            prioritized: None,
            can_spawn: [false; LADDER.len()],
        }
    }

    /// `Main.townNPCCanSpawn[type]`.
    pub(crate) fn can_spawn(&self, kind: NpcKind) -> bool {
        LADDER
            .iter()
            .position(|k| *k == kind)
            .is_some_and(|i| self.can_spawn[i])
    }
}

/// `ItemID.Sets.ItemsThatCountAsBombsForDemolitionistToSpawn`
/// (`ItemID.cs:343`) — the bomb/bomb-adjacent ids the Demolitionist's gate
/// scans for.
pub(crate) const BOMB_IDS: [u16; 21] = [
    168, 2586, 3116, 166, 235, 3115, 167, 2896, 3547, 3196, 4423, 1130, 1168, 4824, 4825, 4826,
    4827, 4908, 4909, 5594, 5595,
];

/// `AmmoID.Bullet` (`AmmoID.cs:164`).
pub(crate) const AMMOID_BULLET: i32 = 97;

/// `NPC.SpawnAllowed_Merchant` (`NPC.cs:7175-7212`): 5000 copper across the
/// inventory (`unlockedMerchantSpawn` overrides; the port has no unlock, so
/// the money is the whole gate). The copper sum itself is
/// [`crate::spawner::spawn_allowed_merchant`]'s — one implementation, whose
/// per-coin tests live with it.
pub(crate) fn spawn_allowed_merchant(inventory: &[ItemSlot]) -> bool {
    let pairs: Vec<(u16, u32)> = inventory
        .iter()
        .filter(|s| s.count > 0)
        .map(|s| (s.kind, s.count))
        .collect();
    crate::spawner::spawn_allowed_merchant(&pairs)
}

/// `NPC.SpawnAllowed_Nurse` (`NPC.cs:7158-7173`): any player at
/// `statLifeMax / 20 > 5`, i.e. above the 100-HP start.
pub(crate) fn spawn_allowed_nurse(max_hp: i32) -> bool {
    max_hp / 20 > 5
}

/// `NPC.SpawnAllowed_ArmsDealer` (`NPC.cs:7092-7115`): a bullet in hand —
/// either the ammo itself (`Item.ammo == AmmoID.Bullet`) or a weapon that
/// consumes it (`Item.useAmmo == AmmoID.Bullet`).
pub(crate) fn spawn_allowed_arms_dealer(inventory: &[ItemSlot]) -> bool {
    inventory.iter().filter(|s| s.count > 0).any(|s| {
        crate::registry::get(s.kind).is_some_and(|d| {
            d.ammo == AMMOID_BULLET || d.use_ammo == AMMOID_BULLET
        })
    })
}

/// `NPC.SpawnAllowed_Demolitionist` (`NPC.cs:7067-7090`): any item of the
/// `ItemsThatCountAsBombsForDemolitionistToSpawn` set.
pub(crate) fn spawn_allowed_demolitionist(inventory: &[ItemSlot]) -> bool {
    inventory
        .iter()
        .filter(|s| s.count > 0)
        .any(|s| BOMB_IDS.contains(&s.kind))
}

/// `NPC.SpawnAllowed_DyeTrader` (`NPC.cs:7117-7156`): an inventory item with
/// `item.dye > 0` (the port's `ItemSlot` carries no `dye` field, so the two
/// id bands the C# names literally stand in — the dye materials 1107-1120 and
/// the Strange Plants 3385-3388), or one of the equipped dye slots. The
/// equipment half is `Player.GetEffectiveDye` / `miscDyes`; the port's dye
/// slots live on the equipment and are passed in as `equipped_dyes`.
pub(crate) fn spawn_allowed_dye_trader(inventory: &[ItemSlot], equipped_dyes: &[ItemSlot]) -> bool {
    let is_dye_band = |kind: u16| (1107..=1120).contains(&kind) || (3385..=3388).contains(&kind);
    inventory
        .iter()
        .filter(|s| s.count > 0)
        .any(|s| is_dye_band(s.kind))
        || equipped_dyes
            .iter()
            .filter(|s| s.count > 0)
            .any(|s| is_dye_band(s.kind))
}

/// The live resident count per kind (`Main.cs:65563-65760`'s `num2..num40`).
fn resident_counts(npcs: &[Npc]) -> impl Fn(NpcKind) -> i32 + '_ {
    move |kind| npcs.iter().filter(|n| n.hp > 0 && n.kind == kind).count() as i32
}

/// Everything the gates read besides the residents.
pub(crate) struct ArrivalContext<'a> {
    pub(crate) world: &'a World,
    pub(crate) inventory: &'a [ItemSlot],
    pub(crate) equipped_dyes: &'a [ItemSlot],
    pub(crate) max_hp: i32,
    /// `Main.rand.Next(40) == 0` for the Party Girl's roll, evaluated once per
    /// check like the C# (`Main.cs:65780-65784`).
    pub(crate) party_roll: bool,
    /// `num40` — the live town-resident total, the `>= 8` / `>= 20` terms the
    /// C# reads inline (`Main.cs:65929`, `:65785`).
    pub(crate) residents: i32,
}

/// The `flag`/`flag2`… set of `Main.cs:65763-65799`, keyed by kind.
pub(crate) fn condition_holds(kind: NpcKind, ctx: &ArrivalContext<'_>) -> bool {
    let w = ctx.world;
    match kind {
        // `num6 < 1` — the Guide is the unconditional one (`:65802`).
        NpcKind::Guide => true,
        NpcKind::Merchant => spawn_allowed_merchant(ctx.inventory),
        // `flag3 && num3 < 1 && num2 > 0`: the Nurse needs a resident
        // Merchant as well as the life gate (`:65810`).
        NpcKind::Nurse => spawn_allowed_nurse(ctx.max_hp),
        NpcKind::ArmsDealer => spawn_allowed_arms_dealer(ctx.inventory),
        // `num4 < 1` and any of the first three bosses (or the infected
        // seed) (`:65814`).
        NpcKind::Dryad => w.downed_eoc || w.downed_eow || w.downed_skeletron,
        // `flag5 && num2 > 0` — the Demolitionist also wants a Merchant.
        NpcKind::Demolitionist => spawn_allowed_demolitionist(ctx.inventory),
        NpcKind::Clothier => w.downed_skeletron,
        NpcKind::GoblinTinkerer => w.saved_goblin,
        NpcKind::Wizard => w.saved_wizard,
        NpcKind::Mechanic => w.saved_mech,
        NpcKind::WitchDoctor => w.downed_queen_bee,
        NpcKind::Angler => w.saved_angler,
        NpcKind::Stylist => w.saved_stylist,
        NpcKind::TaxCollector => w.saved_tax_collector,
        NpcKind::Golfer => w.saved_golfer,
        NpcKind::DyeTrader => spawn_allowed_dye_trader(ctx.inventory, ctx.equipped_dyes),
        // `num40 >= 8` residents (`:65929`).
        NpcKind::Painter => ctx.residents >= 8,
        // `flag7 = (rand.Next(40) == 0) && num40 >= 20`, with
        // `unlockedPartyGirlSpawn` forcing the flag true (`:65785-65799`).
        NpcKind::PartyGirl => {
            (ctx.party_roll && ctx.residents >= 20) || w.unlocked_party_girl
        }
        NpcKind::Zoologist => false,
        // `flag9`: every other town type resident (`:65944-65955`); the port
        // ships the unlock flag, the roster term needs the full type set.
        NpcKind::Princess => w.unlocked_princess,
        NpcKind::TravellingMerchant => false,
        _ => false,
    }
}

/// `Main.UpdateTime_SpawnTownNPCs` (`Main.cs:65542-66116`).
///
/// Returns the type the C# parks in `WorldGen.prioritizedTownNPCType` once the
/// ladder has a winner, or `None` while the 7200-tick gate is closed, the
/// server gate blocks it (`Main.netMode == 1` is the client; the port's client
/// never runs this), or nothing is eligible.
///
/// The caller owns the placement: the vanilla feeds the result to
/// `WorldGen.IsThereASpawnablePrioritizedTownNPC`/`SpawnTownNPC`, which needs a
/// finished, furnished room (`town::quick_find_home` in the port).
#[allow(clippy::too_many_arguments)]
pub(crate) fn update_time_spawn_town_npcs(
    state: &mut ArrivalState,
    ctx: &ArrivalContext<'_>,
    npcs: &[Npc],
    rooms_ready: &dyn Fn(NpcKind) -> bool,
) -> Option<NpcKind> {
    // `:65544-65555` — the client early return and the tick gate. The port's
    // world update rate is 1 (60 Hz ticks), so the interval is the plain 7200.
    state.check_for_spawns += 1;
    if state.check_for_spawns < 7200 {
        return None;
    }
    state.check_for_spawns = 0;

    let count = resident_counts(npcs);
    state.can_spawn = [false; LADDER.len()];

    // `:65785-65955` — the canSpawn table. The C# sets an entry when the
    // condition holds AND the kind has no resident yet.
    for (i, kind) in LADDER.iter().enumerate() {
        if count(*kind) < 1 && condition_holds(*kind, ctx) {
            state.can_spawn[i] = true;
        }
    }
    // The two gates that also require another resident (`:65810`, `:65818`).
    state.can_spawn[LADDER_POS_NURSE] &= count(NpcKind::Merchant) > 0;
    state.can_spawn[LADDER_POS_DEMOLITIONIST] &= count(NpcKind::Merchant) > 0;
    // `:65927-65933` — the Zoologist opens at 10% bestiary completion, and the
    // Princess at one of every other town type. Both need data the port does
    // not model yet; they stay closed rather than guessing.

    // `:65957-66116` — the priority ladder: the first eligible kind wins.
    for (i, kind) in LADDER.iter().enumerate() {
        if state.can_spawn[i] && rooms_ready(*kind) {
            state.prioritized = Some(*kind);
            return Some(*kind);
        }
    }
    state.prioritized = None;
    None
}

const LADDER_POS_NURSE: usize = 2;
const LADDER_POS_DEMOLITIONIST: usize = 7;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::item::EMPTY_SLOT;

    fn slot(kind: u16, count: u32) -> ItemSlot {
        ItemSlot {
            kind,
            count,
            ..EMPTY_SLOT
        }
    }

    fn ctx<'a>(world: &'a World, inventory: &'a [ItemSlot]) -> ArrivalContext<'a> {
        ArrivalContext {
            world,
            inventory,
            equipped_dyes: &[],
            max_hp: 100,
            party_roll: false,
            residents: 1,
        }
    }

    /// `Main.cs:65548-65555`: the check runs every 7200 ticks and then resets
    /// the counter — the port's old ad-hoc gates ran every two seconds.
    #[test]
    fn the_arrival_check_runs_every_seven_thousand_two_hundred_ticks() {
        let world = World::new("t".into(), 50, 50);
        let inv = [slot(71, 5000)];
        let npcs = Vec::new();
        let mut state = ArrivalState::new();
        let mut checks = 0;
        for _ in 0..7200 {
            if update_time_spawn_town_npcs(&mut state, &ctx(&world, &inv), &npcs, &|_| false)
                .is_none()
                && state.check_for_spawns == 0
            {
                checks += 1;
            }
        }
        assert_eq!(checks, 1, "one check per 7200 ticks");
        assert_eq!(state.check_for_spawns, 0, "the counter resets on the check");
    }

    /// `NPC.cs:7181-7206`: the Merchant's 5000-copper gate (through the
    /// inventory adapter — the coin maths itself is `spawner`'s).
    #[test]
    fn the_merchant_gate_counts_the_coins() {
        assert!(!spawn_allowed_merchant(&[slot(71, 4999)]));
        assert!(spawn_allowed_merchant(&[slot(71, 5000)]));
        assert!(spawn_allowed_merchant(&[slot(73, 1)]), "one gold coin");
        assert!(!spawn_allowed_merchant(&[EMPTY_SLOT]));
    }

    /// `NPC.cs:7158-7173`: `statLifeMax / 20 > 5` — above the 100-HP start.
    #[test]
    fn the_nurse_gate_is_the_life_maximum() {
        assert!(!spawn_allowed_nurse(100), "the starting 100 HP waits");
        assert!(spawn_allowed_nurse(120), "a life crystal opens it");
    }

    /// `NPC.cs:7092-7115`: a bullet or a bullet-firing weapon.
    #[test]
    fn the_arms_dealer_gate_wants_a_bullet() {
        // 97 = the Musket Ball (`AmmoID.Bullet`), 96 = the Musket (uses it).
        let ball = [slot(97, 20)];
        assert!(
            spawn_allowed_arms_dealer(&ball),
            "the ammo itself (`Item.ammo`)"
        );
        let musket = [slot(96, 1)];
        assert!(
            spawn_allowed_arms_dealer(&musket),
            "a weapon that consumes it (`Item.useAmmo`)"
        );
        assert!(!spawn_allowed_arms_dealer(&[slot(1, 99)]), "dirt is not a gun");
    }

    /// `NPC.cs:7067-7090` over the `ItemsThatCountAsBombsForDemolitionistToSpawn`
    /// set (`ItemID.cs:343`).
    #[test]
    fn the_demolitionist_gate_wants_a_bomb() {
        assert!(spawn_allowed_demolitionist(&[slot(166, 1)]), "a bomb");
        assert!(spawn_allowed_demolitionist(&[slot(4824, 1)]), "a dynamite");
        assert!(!spawn_allowed_demolitionist(&[slot(71, 999)]), "coins are not");
    }

    /// The ladder's order is the C# one: the Guide first, then the Merchant,
    /// then the Nurse — and one check yields one kind.
    #[test]
    fn the_ladder_picks_one_kind_in_the_vanilla_order() {
        let mut world = World::new("t".into(), 50, 50);
        world.saved_golfer = true;
        let inv = [slot(71, 5000)];
        let mut state = ArrivalState::new();
        state.check_for_spawns = 7199;
        let npcs = Vec::new();
        let all = |_: NpcKind| true;
        // The Guide is eligible (no resident) and leads the ladder.
        assert_eq!(
            update_time_spawn_town_npcs(&mut state, &ctx(&world, &inv), &npcs, &all),
            Some(NpcKind::Guide)
        );
        // With the Guide resident the Merchant's money opens his arm — he is
        // ahead of the flag-gated Golfer, which the old per-kind gates could
        // not express.
        let mut guide = Npc::new(NpcKind::Guide, 0.0, 0.0, 1, 0.0);
        guide.hp = guide.max_hp;
        let roster = vec![guide];
        state.check_for_spawns = 7199;
        assert_eq!(
            update_time_spawn_town_npcs(&mut state, &ctx(&world, &inv), &roster, &all),
            Some(NpcKind::Merchant)
        );
        // The canSpawn table carries both open arms at that check.
        assert!(state.can_spawn(NpcKind::Merchant));
        assert!(
            state.can_spawn(NpcKind::Golfer),
            "the saved Golfer's arm stays open in the table"
        );
    }

    /// The Nurse and the Demolitionist want a resident Merchant as well as
    /// their own condition (`Main.cs:65810`, `:65818`).
    #[test]
    fn the_second_tier_gates_want_a_merchant_first() {
        let world = World::new("t".into(), 50, 50);
        let inv = [slot(71, 5000), slot(166, 1)];
        let mut state = ArrivalState::new();
        let npcs = Vec::new();
        let c = ArrivalContext {
            world: &world,
            inventory: &inv,
            equipped_dyes: &[],
            max_hp: 200,
            party_roll: false,
            residents: 1,
        };
        state.check_for_spawns = 7199;
        update_time_spawn_town_npcs(&mut state, &c, &npcs, &|_| false);
        assert!(
            !state.can_spawn(NpcKind::Nurse),
            "no Merchant, no Nurse — even with the life gate open"
        );
        assert!(!state.can_spawn(NpcKind::Demolitionist));
        // With the Merchant resident both open.
        let mut merchant = Npc::new(NpcKind::Merchant, 0.0, 0.0, 1, 0.0);
        merchant.hp = merchant.max_hp;
        let roster = vec![merchant];
        state.check_for_spawns = 7199;
        update_time_spawn_town_npcs(&mut state, &c, &roster, &|_| false);
        assert!(state.can_spawn(NpcKind::Nurse));
        assert!(state.can_spawn(NpcKind::Demolitionist));
    }

    /// A kind with a resident never re-opens (`count < 1` in every gate).
    #[test]
    fn a_resident_closes_his_own_gate() {
        let world = World::new("t".into(), 50, 50);
        let inv = [slot(71, 5000)];
        let mut state = ArrivalState::new();
        let mut merchant = Npc::new(NpcKind::Merchant, 0.0, 0.0, 1, 0.0);
        merchant.hp = merchant.max_hp;
        let roster = vec![merchant];
        state.check_for_spawns = 7199;
        update_time_spawn_town_npcs(&mut state, &ctx(&world, &inv), &roster, &|_| false);
        assert!(!state.can_spawn(NpcKind::Merchant));
    }
}
