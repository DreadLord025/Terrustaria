//! The town NPC arrival gates — `Main.UpdateTime_SpawnTownNPCs`
//! (`Main.cs:65542-66116`).
//!
//! The vanilla checks every `7200 / WorldGen.GetWorldUpdateRate()` ticks
//! (`:65548-65555`): it counts the live residents of each town type, clears
//! `Main.townNPCCanSpawn`, sets one `canSpawn` entry per kind whose condition
//! holds *and* which has no resident yet (`:65800-65956`), and then picks the
//! first entry of a fixed priority ladder into
//! `WorldGen.prioritizedTownNPCType` (`:65957-66114`). The consumer
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
/// first few is only reachable once its own condition holds. W4-f43b: the
/// eleven g10 town-pet kinds join at the ladder's tail seats
/// (`Main.cs:66066-66113` — Princess, Copper, Blue, Green, Old, Purple,
/// RED, Yellow, RAINBOW, Bunny, Cat, Dog; the Red-before-Rainbow quirk is
/// the C# order, NOT the canSpawn write order); Santa 142's seat
/// (`:66042-66045`) stays closed — no ported Santa kind.
pub(crate) const LADDER: [NpcKind; 32] = [
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
    // `Main.cs:66070-66113` — the pet/slime tail (W4-f43b).
    NpcKind::TownSlimeCopper,
    NpcKind::TownSlimeBlue,
    NpcKind::TownSlimeGreen,
    NpcKind::TownSlimeOld,
    NpcKind::TownSlimePurple,
    NpcKind::TownSlimeRed,
    NpcKind::TownSlimeYellow,
    NpcKind::TownSlimeRainbow,
    NpcKind::TownBunny,
    NpcKind::TownCat,
    NpcKind::TownDog,
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

    /// `Main.townNPCCanSpawn[type] = true` — the table write
    /// (`Main.cs:65800-65956`); a kind outside the LADDER has no entry.
    pub(crate) fn set_can_spawn(&mut self, kind: NpcKind) {
        if let Some(i) = LADDER.iter().position(|k| *k == kind) {
            self.can_spawn[i] = true;
        }
    }

    /// `Main.townNPCCanSpawn[type] = false` — the resident clear
    /// (`WorldGen.cs:4903`, `:5454`, `:5562`).
    pub(crate) fn clear_can_spawn(&mut self, kind: NpcKind) {
        if let Some(i) = LADDER.iter().position(|k| *k == kind) {
            self.can_spawn[i] = false;
        }
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
    /// check like the C# (`Main.cs:65785-65789`).
    pub(crate) party_roll: bool,
    /// `num40` — the live town-resident total, the `>= 8` / `>= 20` terms the
    /// C# reads inline (`Main.cs:65880`, `:65790`).
    pub(crate) residents: i32,
    /// `WorldGen.GetWorldUpdateRate()` (`WorldGen.cs:72020-72030`): the
    /// check interval is `7200 / rate` and a zero rate skips the whole
    /// function (`Main.cs:65544-65552`, W4-d119 phase 2b).
    pub(crate) world_update_rate: i32,
    /// `Main.GetBestiaryProgressReport().CompletionPercent`
    /// (`Main.cs:65770`, the Zoologist's `>= 0.1f` gate `:65896-65899` /
    /// `:66062-66065`, W4-d119 phase 2b). The vampire-seed lead term
    /// (`vampireSeed && !infectedSeed`) stays constants-off — the
    /// special-seed lane's claim (f43 census).
    pub(crate) bestiary_completion: f32,
    /// `BirthdayParty.GenuineParty` (`Main.cs:65795`) — flag8's first term,
    /// the TownSlimeGreen 678 gate (W4-d119 seal: the f41 census's remaining
    /// tail). NOT `PartyIsUp` — a manual monolith party does not open it.
    pub(crate) genuine_party: bool,
    /// W4-f43b: the census-tail flags (`town_arrival_pets::TailFlags`) — the
    /// bought trio, the eight slime unlocks, flag8's GenuineParty term, the
    /// season/anniversary overrides. The caller assembles them via
    /// `tail_flags_from_world` (the party/season inputs live on the App).
    pub(crate) tail: crate::town_arrival_pets::TailFlags,
    /// W4-f43b: flag9 precomputed over the LIVE roster
    /// (`town_arrival_pets::princess_flag9` with the by-id resident count) —
    /// the 24-type conjunction needs the full vanilla-id walk the per-kind
    /// `count` cannot give (unported types read 0, closing the roster term
    /// exactly as the C# would with those residents absent).
    pub(crate) princess_flag9: bool,
}

/// The `flag`/`flag2`… set of `Main.cs:65772-65799`, keyed by kind.
pub(crate) fn condition_holds(kind: NpcKind, ctx: &ArrivalContext<'_>) -> bool {
    let w = ctx.world;
    match kind {
        // `num6 < 1` — the Guide is the unconditional one (`:65800-65803`).
        NpcKind::Guide => true,
        NpcKind::Merchant => spawn_allowed_merchant(ctx.inventory),
        // `flag3 && num3 < 1 && num2 > 0`: the Nurse needs a resident
        // Merchant as well as the life gate (`:65808-65811`).
        NpcKind::Nurse => spawn_allowed_nurse(ctx.max_hp),
        NpcKind::ArmsDealer => spawn_allowed_arms_dealer(ctx.inventory),
        // `num4 < 1` and any of the first three bosses (or the infected
        // seed) (`:65816-65819`).
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
        NpcKind::DyeTrader => {
            // `flag4 && num16 < 1 && num40 >= 4` (`Main.cs:65860-65863`) —
            // the four-resident term the landed core dropped (W4-d119
            // phase 2b, the f43 census's "known LANDED gap").
            spawn_allowed_dye_trader(ctx.inventory, ctx.equipped_dyes) && ctx.residents >= 4
        }
        // `num40 >= 8` residents (`:65880-65883`).
        NpcKind::Painter => ctx.residents >= 8,
        // `flag7 = (rand.Next(40) == 0) && num40 >= 20`, with
        // `unlockedPartyGirlSpawn` forcing the flag true (`:65785-65794`).
        NpcKind::PartyGirl => {
            (ctx.party_roll && ctx.residents >= 20) || w.unlocked_party_girl
        }
        // `((vampireSeed && !infectedSeed) || bestiary >= 0.1f) && num27 < 1`
        // (`Main.cs:65896-65899`) — the bestiary term wired (W4-d119 phase
        // 2b); the vampire-seed lead stays constants-off (special-seed lane).
        NpcKind::Zoologist => ctx.bestiary_completion >= 0.1,
        // `flag9`: every other town type resident (`:65944-65955`); the port
        // ships the unlock flag, the roster term needs the full type set.
        NpcKind::Princess => w.unlocked_princess,
        NpcKind::Zoologist => false,
        // W4-f43b — flag9's full form (`Main.cs:65944-65952`): ALL 24
        // roster types resident, OR the tenth-anniversary override (unless
        // getGoodWorld), OR the debug unlock. The roster walks the live
        // residents by vanilla id (`princess_flag9`'s injected count).
        NpcKind::Princess => ctx.princess_flag9,
        NpcKind::TravellingMerchant => false,
        // W4-f43b — the census tail's pet/slime arms
        // (`Main.cs:65900-65943`), through the pure `tail_can_spawn` rows.
        NpcKind::TownCat => ctx.tail.bought_cat,
        NpcKind::TownDog => ctx.tail.bought_dog,
        NpcKind::TownBunny => ctx.tail.bought_bunny,
        NpcKind::TownSlimeBlue => ctx.tail.unlocked_slime_blue,
        // flag8 = GenuineParty || unlockedSlimeGreenSpawn (`:65795-65799`).
        NpcKind::TownSlimeGreen => ctx.tail.genuine_party || ctx.tail.unlocked_slime_green,
        NpcKind::TownSlimeOld => ctx.tail.unlocked_slime_old,
        NpcKind::TownSlimePurple => ctx.tail.unlocked_slime_purple,
        NpcKind::TownSlimeRainbow => ctx.tail.unlocked_slime_rainbow,
        NpcKind::TownSlimeRed => ctx.tail.unlocked_slime_red,
        NpcKind::TownSlimeYellow => ctx.tail.unlocked_slime_yellow,
        NpcKind::TownSlimeCopper => ctx.tail.unlocked_slime_copper,
        _ => false,
    }
}

/// `flag8` (`Main.cs:65795-65799`): `BirthdayParty.GenuineParty`, forced true
/// by `NPC.unlockedSlimeGreenSpawn`. The TownSlimeGreen 678 gate — the
/// canSpawn arm (`:65916-65919`, `flag8 && num32 < 1`) and the ladder arm
/// (`:66078-66081`, `((num42 == 0) & flag8) && num32 < 1`, the C#'s
/// non-short-circuit `&`; both sides are pure so the shape is behaviourally
/// identical to `&&`). ZERO draws.
///
/// f43 owns the verbatim twin inside its UNWIRED
/// `town_arrival_pets::tail_can_spawn`/`pet_slime_princess_ladder_tail`
/// (`TailFlags.genuine_party || TailFlags.unlocked_slime_green`, research/64
/// census) — at the tail splice this local folds into f43's copy, ONE gate.
/// The 678 resident count (`num32`) has no ported kind yet (the town-slime
/// kinds are the d43 lane's), so the production gate cannot fire until that
/// splice; the predicate is the f41 census's claimed remaining tail, landed
/// here with its witnesses (the f41e party census × f43 arrival compose).
#[allow(dead_code)] // fires in production at the d43-kind + f43-tail splice
pub(crate) fn slime_green_flag8(genuine_party: bool, unlocked_slime_green: bool) -> bool {
    genuine_party || unlocked_slime_green
}

/// The 678 canSpawn/ladder gate: `flag8 && num32 < 1`
/// (`Main.cs:65916-65919`/`:66078-66081`). `count_678` is the live
/// TownSlimeGreen resident count and `unlocked_slime_green` the
/// `NPC.unlockedSlimeGreenSpawn` flag — the port's world carries neither yet
/// (only the rainbow/red slime flags, `world.rs:303-307`; the f43 census's
/// save-widening note), so the splice's production call passes `false, 0`
/// until the save widens; `ctx.genuine_party` is already fed live.
#[allow(dead_code)] // fires in production at the d43-kind + f43-tail splice
pub(crate) fn slime_green_can_spawn(
    ctx: &ArrivalContext<'_>,
    unlocked_slime_green: bool,
    count_678: i32,
) -> bool {
    slime_green_flag8(ctx.genuine_party, unlocked_slime_green) && count_678 < 1
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
    // `:65544-65555` — the client early return (the caller gates it), the
    // rate-0 skip and the tick gate: `checkForSpawns < 7200 /
    // worldUpdateRate` (W4-d119 phase 2b — the landed core folded the rate
    // to 1; the journey time-rate slider / all-asleep x5 now divide it).
    if ctx.world_update_rate <= 0 {
        return None;
    }
    state.check_for_spawns += 1;
    if state.check_for_spawns < 7200 / ctx.world_update_rate {
        return None;
    }
    state.check_for_spawns = 0;

    let count = resident_counts(npcs);
    state.can_spawn = [false; LADDER.len()];

    // `:65800-65956` — the canSpawn table. The C# sets an entry when the
    // condition holds AND the kind has no resident yet.
    for (i, kind) in LADDER.iter().enumerate() {
        if count(*kind) < 1 && condition_holds(*kind, ctx) {
            state.can_spawn[i] = true;
        }
    }
    // The two gates that also require another resident (`:65808`, `:65820`).
    state.can_spawn[LADDER_POS_NURSE] &= count(NpcKind::Merchant) > 0;
    state.can_spawn[LADDER_POS_DEMOLITIONIST] &= count(NpcKind::Merchant) > 0;
    // `:65944-65956` — the Princess at one of every other town type; the
    // full flag9 roster is the f43 lane's pushed claim
    // (`town_arrival_pets::princess_flag9`, UNWIRED); the unlock-only stub
    // stays until that splice. The Zoologist's bestiary term is wired above.
    // `:65916-65919` — the TownSlimeGreen 678 flag8 arm has NO LADDER entry
    // (no ported kind — d43's kinds lane); its gate lands as the pure
    // `slime_green_can_spawn` above, fed `ctx.genuine_party`, folding into
    // f43's `tail_can_spawn` row at the splice (ONE gate, research/333).

    // `:65957-66114` — the priority ladder: the first eligible kind wins.
    // `:65927-65933` — the Zoologist opens at 10% bestiary completion; the
    // port does not model the completion percent yet, so his arm stays
    // closed rather than guessing. The Princess's flag9 IS live now
    // (W4-f43b — `ctx.princess_flag9`, the full 24-type roster walk +
    // the anniversary/unlock overrides).

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
            world_update_rate: 1,
            bestiary_completion: 0.0,
            genuine_party: false,
            tail: crate::town_arrival_pets::tail_flags_from_world(world, false, false),
            princess_flag9: world.unlocked_princess,
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
            world_update_rate: 1,
            bestiary_completion: 0.0,
            genuine_party: false,
            tail: crate::town_arrival_pets::tail_flags_from_world(&world, false, false),
            princess_flag9: world.unlocked_princess,
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

    /// W4-d119 phase 2b — `Main.cs:65544-65552`: the interval is
    /// `7200 / GetWorldUpdateRate()`; at rate 5 (all-asleep) the check fires
    /// on tick 1440, and at rate 0 (FreezeTime) the counter never advances.
    #[test]
    fn the_check_interval_divides_by_the_world_update_rate() {
        let world = World::new("t".into(), 50, 50);
        let inv = [slot(71, 5000)];
        let npcs = Vec::new();
        let mut state = ArrivalState::new();
        let mut c = ctx(&world, &inv);
        c.world_update_rate = 5;
        let mut checks = 0;
        for _ in 0..1440 {
            update_time_spawn_town_npcs(&mut state, &c, &npcs, &|_| false);
            if state.check_for_spawns == 0 {
                checks += 1;
            }
        }
        assert_eq!(checks, 1, "one check per 7200/5 = 1440 ticks");
        // Rate 0: the whole function skips before the counter increment.
        let mut state = ArrivalState::new();
        c.world_update_rate = 0;
        for _ in 0..7200 {
            assert!(update_time_spawn_town_npcs(&mut state, &c, &npcs, &|_| false).is_none());
        }
        assert_eq!(state.check_for_spawns, 0, "the frozen clock never counts");
    }

    /// W4-d119 phase 2b — `Main.cs:65860-65863`: the DyeTrader's gate needs
    /// `num40 >= 4` residents on top of the dye; `:65896-65899`: the
    /// Zoologist opens at `CompletionPercent >= 0.1f`.
    #[test]
    fn the_dye_trader_wants_four_residents_and_the_zoologist_the_bestiary() {
        let world = World::new("t".into(), 50, 50);
        let inv = [slot(1109, 1)];
        let mut state = ArrivalState::new();
        let mut c = ctx(&world, &inv);
        c.residents = 3;
        state.check_for_spawns = 7199;
        update_time_spawn_town_npcs(&mut state, &c, &[], &|_| false);
        assert!(!state.can_spawn(NpcKind::DyeTrader), "three residents wait");
        c.residents = 4;
        state.check_for_spawns = 7199;
        update_time_spawn_town_npcs(&mut state, &c, &[], &|_| false);
        assert!(state.can_spawn(NpcKind::DyeTrader), "the fourth opens it");
        // The Zoologist: below 10% closed, at 10% open (the f32 compare).
        assert!(!state.can_spawn(NpcKind::Zoologist));
        c.bestiary_completion = 0.1;
        state.check_for_spawns = 7199;
        update_time_spawn_town_npcs(&mut state, &c, &[], &|_| false);
        assert!(state.can_spawn(NpcKind::Zoologist), "10% bestiary opens her");
    }

    /// W4-d119 seal (the f41e census × f43 compose) — `Main.cs:65795-65799`:
    /// flag8 = GenuineParty || unlockedSlimeGreenSpawn, the four-row truth
    /// table; `:65916-65919`/`:66078-66081`: the 678 arm also needs num32 < 1.
    /// ZERO draws — the gate never touches the party Next(40) slot.
    #[test]
    fn the_slime_green_flag8_gate_is_party_or_unlock_and_no_resident() {
        // The flag8 truth table (`:65795-65799`).
        assert!(!slime_green_flag8(false, false));
        assert!(slime_green_flag8(true, false), "a genuine party opens it");
        assert!(slime_green_flag8(false, true), "the unlock forces it true");
        assert!(slime_green_flag8(true, true));
        let world = World::new("t".into(), 50, 50);
        let mut c = ctx(&world, &[]);
        // No party, no unlock: closed even with zero residents.
        assert!(!slime_green_can_spawn(&c, false, 0));
        c.genuine_party = true;
        assert!(slime_green_can_spawn(&c, false, 0));
        // A resident Green slime closes it (`num32 < 1`).
        assert!(!slime_green_can_spawn(&c, false, 1));
        // The unlock term works without a party.
        c.genuine_party = false;
        assert!(slime_green_can_spawn(&c, true, 0));
    }

    /// The flag8 gate rides the SAME rate-aware check boundary as the party
    /// Next(40) roll and draws NOTHING itself: the ctx field is a read, not a
    /// roll — a check with genuine_party true leaves every LADDER arm exactly
    /// as the party-less check did (678 has no LADDER entry until the d43
    /// kinds + f43 tail splice; the PartyGirl flag7 gate is untouched).
    #[test]
    fn the_flag8_feed_changes_no_landed_ladder_arm() {
        let world = World::new("t".into(), 50, 50);
        let inv = [slot(71, 5000)];
        let npcs = Vec::new();
        let mut a = ArrivalState::new();
        let mut b = ArrivalState::new();
        let mut ca = ctx(&world, &inv);
        ca.genuine_party = false;
        let mut cb = ctx(&world, &inv);
        cb.genuine_party = true;
        a.check_for_spawns = 7199;
        b.check_for_spawns = 7199;
        let ra = update_time_spawn_town_npcs(&mut a, &ca, &npcs, &|_| true);
        let rb = update_time_spawn_town_npcs(&mut b, &cb, &npcs, &|_| true);
        assert_eq!(ra, rb, "the same ladder winner");
        assert_eq!(a.can_spawn, b.can_spawn, "the same canSpawn table");
        assert!(!a.can_spawn(NpcKind::PartyGirl), "flag7 stays its own gate");
        // But the 678 predicate itself flips on the feed.
        assert!(!slime_green_can_spawn(&ca, false, 0));
        assert!(slime_green_can_spawn(&cb, false, 0));
    }
}
