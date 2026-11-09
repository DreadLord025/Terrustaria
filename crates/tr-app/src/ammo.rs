//! The ammo system (`Player.PickAmmo`, `Player.cs:53458-53683`): the pick
//! order, the projectile resolution chain, the drain and the not-consume
//! rolls.
//!
//! The pick order is `Player.AmmoSlotOrder_Default` (`Player.cs:57141-57144`):
//! the coin slots 50-53, the ammo slots 54-57, then the main grid 0-53 — the
//! port's `equip.coins` → `equip.ammo` → `App.inventory`. The vanilla's
//! `ammoCyclingMode` defaults to `None` (no cycling), so the first matching
//! slot in the order wins; a granted mode rotates the pick behind
//! `ammoCyclingOffset` instead (`pick`'s `cycling` parameter, Phase 4).
//!
//! The drain lives inside `PickAmmo` itself
//! (`Player.cs:53667-53672`: `if (!flag && item.consumable) item.stack--`) —
//! `flag` starts as the `dontConsume` argument (`ItemID.Sets.gunProj` — only
//! 4 special endgame items, `ItemID.cs:1327`) and the not-consume rolls set
//! it. The port's rolls cover the shooters whose items the registry carries.
//!
//! `PickAmmo` also resolves WHICH projectile the pair fires
//! (`Player.cs:53458-53568`): the specific-pair table
//! (`AmmoID.Sets.SpecificLauncherAmmoProjectileMatches`, `AmmoID.cs:22-128`)
//! wins first, then the Rocket/Solution additive arms (`:53483-53490`), then the
//! ammo's own `Item.shoot` (`:53491-53494`), then the per-weapon overrides
//! (`:53495-53541`) and the damage/speed fuse (`:53542-53568`) — the whole chain
//! is [`resolve_projectile`]. The two formula arms at `Player.cs:53475-53482`
//! (the 1946 Snowman Cannon's `338 + item.type - 771` and the 3930
//! Celebration Mk2's `715 + item.type - 771`) are DEAD in 1.4.5.8 — the table
//! covers both launchers for all twelve rocket kinds, so those arms can never
//! be reached — and are deliberately not ported.
//!
//! [`resolve_projectile`] owns the AMMO side of the stat fuse: `AmmoShot.speed`
//! is fused from the weapon's own `shootSpeed`, while `damage` and `knock_back`
//! are the ammo's own contributions (the vanilla's `Damage`/`KnockBack` start at
//! the weapon's own values, which the caller keeps). Two consequences the wire
//! (Phase 4) must carry: the Nanite-bullet arm's `Damage = (int)(Damage * 1.25f)`
//! scales the FUSED damage, so the weapon's part takes the same factor when the
//! resolved projectile is 285; and `magicQuiver`'s `KnockBack *= 1.1f` runs
//! BEFORE `KnockBack += item.knockBack` (`:53568`), so it scales the weapon's own
//! knockback only.
//!
//! ## The not-consume chain (`not_consume`)
//!
//! `Player.cs:53569-53682`, ported as a FLAT sequence of one-line arms in the
//! source's order — the order IS the contract, not an accident of layout:
//!
//! 1. Every arm SETS `flag = true` (there is no `else`), so an earlier arm's
//!    save is only wiped by the LAST arm, the Acorn Slingshot's
//!    `flag = false` (`:53671`), which therefore beats the Ammo Box / Ammo
//!    Reservation arms that sit above it.
//! 2. Every arm is a short-circuit gate: an arm that does not apply spends NO
//!    draw, because the C#'s `&&` never evaluates `Main.rand.Next` (`:53579`,
//!    `:53595`, `:53643`, …). One pick therefore spends exactly the vanilla draw
//!    count — the determinism invariant the golden-stream test pins.
//! 3. The arms are ordered as the C# lists them, INCLUDING the interleaving of
//!    the item-keyed and the flag-gated families. The ten item-keyed arms are
//!    mutually exclusive (one `sItem.type` per shot), so their order among
//!    themselves is stream-neutral; their order relative to the flag-gated arms
//!    is NOT (a Minishark + Ammo Box shot rolls ammoBox → ammoPotion → 98), so
//!    the chain must not be re-sorted into "all item arms first".
//!
//! The two arms the source carries and this port cannot reach:
//! - `sItem.type == 3821` (the Ale Throwing Glove, `:53639-53641`): `useAmmo == 0`
//!   on the item and `PickAmmo` only runs `if (sItem.useAmmo > 0)`
//!   (`Player.cs:47069`), so the roll can never fire — DEAD, not ported.
//! - `Main.remixWorld && sItem.type == 1319` (the Snowball Cannon,
//!   `:53659-53661`): the celebration-seed flag has no port stand-in, so the
//!   gate is UNREACHABLE rather than unconditional — PARKED, recorded here, and
//!   the arm is omitted rather than ported ungated (which would give every world
//!   a 1/2 save the vanilla reserves for that seed).
//!
//! The three window booleans arrive pre-derived ([`VolleyWindows`]) because the
//! C# reads `ItemAnimationJustStarted` / `itemAnimation` / `itemAnimationMax` /
//! `useTime` (`:4066`) and the port's shoot wire owns the seconds↔ticks
//! conversion — `ammo.rs` stays tick-semantics-free (the plan's Window
//! booleans-as-seam decision).

use crate::inv::EquipSlots;
use crate::item::{INV, ItemSlot};

/// The picked slot's location — the drain mutates through it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AmmoSource {
    Coin(usize),
    Ammo(usize),
    Grid(usize),
}

/// `AmmoID.Coin` (71) (`ID/AmmoID.cs:161`-adjacent const block) — the coin-gun
/// category `PickAmmo_PickAmmoItem` branches on for its `AmmoSlots` first pass.
const AMMO_COIN: i32 = 71;

/// The slot match `PickAmmo_IterateRange` reads: `item.ammo == sItem.useAmmo
/// && item.stack > 0` (`Player.cs:53699`) — the predicate both walks share.
fn is_ammo_match(slot: &ItemSlot, use_ammo: i32) -> bool {
    slot.count > 0 && crate::registry::get(slot.kind).is_some_and(|d| d.ammo == use_ammo)
}

/// `PickAmmo_IterateRange`'s tail (`Player.cs:53713-53726`): walk one slot
/// range and return the index of its `skip`-th match (`skip == 0` → the
/// first), together with how many matches the range holds. The count is what
/// the caller carries into the next range when the skip is not consumed
/// there, because the C# walks ONE 59-slot `inventory` array where the port
/// has three (coins / ammo / grid). `(None, seen)` means "no match at this
/// skip; consume `seen` from the skip".
fn nth_match(slots: &[ItemSlot], use_ammo: i32, mut skip: i32) -> (Option<usize>, usize) {
    let mut seen = 0usize;
    for (i, slot) in slots.iter().enumerate() {
        if is_ammo_match(slot, use_ammo) {
            if skip == 0 {
                return (Some(i), seen + 1);
            }
            skip -= 1;
            seen += 1;
        }
    }
    (None, seen)
}

/// `AmmoSlotOrder_Default` (`Player.cs:57141-57144`) as one walk: the coin
/// slots 50-53, the ammo slots 54-57, then the grid 0-53 — the port's three
/// arrays, with the cycler's `skip` carried across them.
fn pick_ordered(
    coins: &[ItemSlot],
    ammo: &[ItemSlot],
    inventory: &[ItemSlot; INV],
    use_ammo: i32,
    skip: i32,
) -> Option<AmmoSource> {
    let groups: [(fn(usize) -> AmmoSource, &[ItemSlot]); 3] = [
        (AmmoSource::Coin as fn(usize) -> AmmoSource, coins),
        (AmmoSource::Ammo as fn(usize) -> AmmoSource, ammo),
        (AmmoSource::Grid as fn(usize) -> AmmoSource, inventory),
    ];
    let mut skip = skip;
    for (ctor, slots) in groups {
        let (hit, seen) = nth_match(slots, use_ammo, skip);
        if let Some(i) = hit {
            return Some(ctor(i));
        }
        skip -= seen as i32;
    }
    None
}

/// `AmmoSlotOrder_Default`'s matching-stack count — `PickAmmo_IterateRange`'s
/// `num2` (`Player.cs:53693-53698`) over the whole order, which the C# can sum
/// in one loop because its inventory is one 59-slot array.
fn count_ordered(
    coins: &[ItemSlot],
    ammo: &[ItemSlot],
    inventory: &[ItemSlot; INV],
    use_ammo: i32,
) -> usize {
    [coins, ammo, inventory]
        .iter()
        .flat_map(|slots| slots.iter())
        .filter(|slot| is_ammo_match(slot, use_ammo))
        .count()
}

/// `PickAmmo_IterateRange`'s cycling pass over ONE range (`Player.cs:53689-53712`):
/// count the matching stacks (0 → `None`), then take the `offset % count`-th.
fn pick_cycling(slots: &[ItemSlot], use_ammo: i32, offset: i32) -> Option<usize> {
    let count = slots.iter().filter(|s| is_ammo_match(s, use_ammo)).count();
    if count == 0 {
        return None;
    }
    // `this.ammoCyclingOffset % num2` — the offset only ever increments, resets
    // to 0 (`Update_AmmoCycler`, `Player.cs:28419-28426`) or is drawn
    // non-negative from `Main.rand` (`:53577`), so the Euclidean remainder is
    // the C#'s `%` for every reachable value.
    nth_match(slots, use_ammo, offset.rem_euclid(count as i32)).0
}

/// `PickAmmo_IterateRange`'s cycling pass over the WHOLE default order
/// (`allowAmmoCycling: true`, `Player.cs:53752` — the `FullInventory` and
/// `Random` modes).
fn pick_ordered_cycling(
    coins: &[ItemSlot],
    ammo: &[ItemSlot],
    inventory: &[ItemSlot; INV],
    use_ammo: i32,
    offset: i32,
) -> Option<AmmoSource> {
    let count = count_ordered(coins, ammo, inventory, use_ammo);
    if count == 0 {
        return None;
    }
    pick_ordered(
        coins,
        ammo,
        inventory,
        use_ammo,
        offset.rem_euclid(count as i32),
    )
}

/// `Player.PickAmmo_PickAmmoItem` + `PickAmmo_IterateRange`
/// (`Player.cs:53727-53753`, `:53685-53726`): the slot in the order whose
/// item's `ammo` field matches the weapon's `useAmmo` and whose stack is
/// positive. `None` = no ammo — the shot is refused (`canShoot` stays
/// false, `Player.cs:53462-53466`).
///
/// `cycling` is the shooter's [`AmmoCycling`] (the `Player.ammoCyclingMode` /
/// `ammoCyclingOffset` pair, `Player.cs:1244`/`:1248`). `None` — or the
/// `AmmoCyclingMode::None` mode — is the vanilla default and keeps the
/// pre-cycling behaviour bit-for-bit: the FIRST matching slot in
/// `AmmoSlotOrder_Default` wins. A granted mode rotates the pick behind the
/// offset instead (`num = ammoCyclingOffset % num2`, `:53704`):
/// `AmmoSlots` counts and indexes the coin/ammo ranges alone — coin ammo
/// through `AmmoSlotOrder_CoinsOnly`, everything else through
/// `AmmoSlotOrder_AmmoOnly` (`:53729-53745`) — and falls through to the
/// default order with `allowAmmoCycling: false` when those ranges hold no
/// match (`:53752`); `FullInventory` and `Random` pass the offset through the
/// whole default order.
///
/// The two Clentaminators' forced index 0 (`num = 0` for 779/5134, `:53708-53711`)
/// is applied by the caller, which knows the shooter; `pick` stays
/// shooter-agnostic.
pub fn pick(
    inventory: &[ItemSlot; INV],
    equip: &EquipSlots,
    use_ammo: i32,
    cycling: Option<&AmmoCycling>,
) -> Option<AmmoSource> {
    if use_ammo <= 0 {
        return None;
    }
    let mode = cycling.map_or(AmmoCyclingMode::None, |c| c.mode);
    let offset = cycling.map_or(0, |c| c.offset);
    match mode {
        AmmoCyclingMode::AmmoSlots => {
            // `PickAmmo_PickAmmoItem`'s first pass (`:53729-53745`), the only
            // pass of this mode that rides the offset.
            let (ctor, slots): (fn(usize) -> AmmoSource, &[ItemSlot]) = if use_ammo == AMMO_COIN {
                (AmmoSource::Coin as fn(usize) -> AmmoSource, &equip.coins)
            } else {
                (AmmoSource::Ammo as fn(usize) -> AmmoSource, &equip.ammo)
            };
            if let Some(i) = pick_cycling(slots, use_ammo, offset) {
                return Some(ctor(i));
            }
        }
        AmmoCyclingMode::FullInventory | AmmoCyclingMode::Random => {
            // `:53752` — the default order with `allowAmmoCycling: true`: the
            // count is over the WHOLE order, so the offset wraps across the
            // coin, ammo and grid ranges alike.
            if let Some(src) =
                pick_ordered_cycling(&equip.coins, &equip.ammo, inventory, use_ammo, offset)
            {
                return Some(src);
            }
        }
        // `None` — no cycling: nothing to do before the default walk below.
        AmmoCyclingMode::None => {}
    }
    // The default order, first match (the AmmoSlots fall-through lands here
    // with `allowAmmoCycling: false`, `Player.cs:53752`).
    pick_ordered(&equip.coins, &equip.ammo, inventory, use_ammo, 0)
}

/// The slot read through a [`AmmoSource`].
pub fn slot<'a>(
    source: AmmoSource,
    inventory: &'a [ItemSlot; INV],
    equip: &'a EquipSlots,
) -> &'a ItemSlot {
    match source {
        AmmoSource::Coin(i) => &equip.coins[i],
        AmmoSource::Ammo(i) => &equip.ammo[i],
        AmmoSource::Grid(i) => &inventory[i],
    }
}

/// The drain half (`Player.cs:53667-53672`): `stack--`, `TurnToAir` at 0.
/// Returns false when the stack was already empty (the caller re-checks).
pub fn drain(source: AmmoSource, inventory: &mut [ItemSlot; INV], equip: &mut EquipSlots) -> bool {
    let slot = match source {
        AmmoSource::Coin(i) => &mut equip.coins[i],
        AmmoSource::Ammo(i) => &mut equip.ammo[i],
        AmmoSource::Grid(i) => &mut inventory[i],
    };
    if slot.count == 0 {
        return false;
    }
    slot.count -= 1;
    if slot.count == 0 {
        *slot = crate::item::EMPTY_SLOT;
    }
    true
}

// ---------------------------------------------------------------- projToShoot

/// `AmmoID.Arrow` (40), `AmmoID.Rocket` (771), `AmmoID.Solution` (780) and
/// `AmmoID.Stake` (1836) (`ID/AmmoID.cs:161`, `:167`, `:168`, `:174`) — the ammo
/// categories the resolution chain keys on.
#[allow(dead_code)] // Phase 4's shooter wire consumes the resolver
const AMMO_ARROW: i32 = 40;
#[allow(dead_code)] // Phase 4's shooter wire consumes the resolver
const AMMO_ROCKET: i32 = 771;
#[allow(dead_code)] // Phase 4's shooter wire consumes the resolver
const AMMO_SOLUTION: i32 = 780;
#[allow(dead_code)] // Phase 4's shooter wire consumes the resolver
const AMMO_STAKE: i32 = 1836;

/// `AmmoID.Sets.SpecificLauncherAmmoProjectileMatches` (`ID/AmmoID.cs:22-128`):
/// `(launcher, ammo kind) → projectile`, the five launchers × the twelve rocket
/// kinds, transcribed verbatim in the C# literal's order (759, 758, 760, 1946,
/// 3930). Every cell is distinct — only the 3930 row repeats 717/718, because
/// the four 1.4.4 rockets collapse onto the plain Celebration projectiles
/// (`AmmoID.cs:104-111`) and even that result is discarded at the spawn (3930's
/// fixed gun projectile 714). Do NOT "harmonise" any value.
#[allow(dead_code)] // Phase 4's shooter wire consumes the resolver
pub const SPECIFIC_LAUNCHER_AMMO_PROJECTILE_MATCHES: [(u16, u16, u16); 60] = [
    // 759 Rocket Launcher (`AmmoID.cs:24-41`).
    (759, 771, 134),
    (759, 772, 137),
    (759, 773, 140),
    (759, 774, 143),
    (759, 4445, 776),
    (759, 4446, 780),
    (759, 4457, 793),
    (759, 4458, 796),
    (759, 4459, 799),
    (759, 4447, 784),
    (759, 4448, 787),
    (759, 4449, 790),
    // 758 Grenade Launcher (`AmmoID.cs:42-59`).
    (758, 771, 133),
    (758, 772, 136),
    (758, 773, 139),
    (758, 774, 142),
    (758, 4445, 777),
    (758, 4446, 781),
    (758, 4457, 794),
    (758, 4458, 797),
    (758, 4459, 800),
    (758, 4447, 785),
    (758, 4448, 788),
    (758, 4449, 791),
    // 760 Proximity Mine Launcher (`AmmoID.cs:60-77`).
    (760, 771, 135),
    (760, 772, 138),
    (760, 773, 141),
    (760, 774, 144),
    (760, 4445, 778),
    (760, 4446, 782),
    (760, 4457, 795),
    (760, 4458, 798),
    (760, 4459, 801),
    (760, 4447, 786),
    (760, 4448, 789),
    (760, 4449, 792),
    // 1946 Snowman Cannon (`AmmoID.cs:78-95`).
    (1946, 771, 338),
    (1946, 772, 339),
    (1946, 773, 340),
    (1946, 774, 341),
    (1946, 4445, 803),
    (1946, 4446, 804),
    (1946, 4457, 808),
    (1946, 4458, 809),
    (1946, 4459, 810),
    (1946, 4447, 805),
    (1946, 4448, 806),
    (1946, 4449, 807),
    // 3930 Celebration Mk2 (`AmmoID.cs:96-113`).
    (3930, 771, 715),
    (3930, 772, 716),
    (3930, 773, 717),
    (3930, 774, 718),
    (3930, 4445, 717),
    (3930, 4446, 718),
    (3930, 4457, 717),
    (3930, 4458, 718),
    (3930, 4459, 717),
    (3930, 4447, 717),
    (3930, 4448, 717),
    (3930, 4449, 717),
];

/// `PickAmmo_TryFindingSpecificMatches` (`Player.cs:53448-53456`): the table
/// lookup — the FIRST arm of the resolution chain, and the only source of the
/// per-ammo-kind projectile for the five launchers.
#[allow(dead_code)] // Phase 4's shooter wire consumes the resolver
fn specific_launcher_ammo_match(launcher: u16, ammo: u16) -> Option<u16> {
    SPECIFIC_LAUNCHER_AMMO_PROJECTILE_MATCHES
        .iter()
        .find(|&&(l, a, _)| l == launcher && a == ammo)
        .map(|&(_, _, proj)| proj)
}

/// `AmmoID.Sets.IsArrow` (`AmmoID.cs:115`): `CreateBoolSet(Arrow, Stake)`.
///
/// The C# arms read `IsArrow[item.ammo]` (the AMMO item's category), and
/// `PickAmmo_PickAmmoItem` only ever picks an item whose `ammo` field equals
/// `sItem.useAmmo` (`Player.cs:53699`), so for every pair this chain can see
/// `IsArrow[item.ammo]` is exactly `IsArrow[sItem.useAmmo]` — which the frozen
/// signature carries as `use_ammo`.
#[allow(dead_code)] // Phase 4's shooter wire consumes the resolver
fn is_arrow_ammo(use_ammo: i32) -> bool {
    use_ammo == AMMO_ARROW || use_ammo == AMMO_STAKE
}

/// The accessory/buff flags `Player.PickAmmo`'s fuse and roll arms read
/// (`Player.cs:53537`, `:53543`, `:53548`, `:53560`). Their real owners are
/// M3-EQUIP-3 / M2-BUFF, so every field defaults `false` and the arms below stay
/// inert until they land. Phase 3 APPENDS the six roll flags — the field names
/// and the read order stay frozen.
#[allow(dead_code)] // Phase 4's shooter wire consumes the resolver
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AmmoFlags {
    /// `this.hasMoltenQuiver` — 4002 Molten Quiver / 6181 Phoenix Quiver
    /// (`Player.cs:14467`, `:14596`): converts the wooden arrow to the FireArrow
    /// and adds 2 ammo damage.
    pub has_molten_quiver: bool,
    /// `this.magicQuiver` — 4002 / 4006 / 6181 (`Player.cs:14465`, `:14483`,
    /// `:14596`): ×1.1 speed for arrow/stake ammo (the knockback half scales the
    /// weapon's own value, see [`resolve_projectile`]) and a 1/5 roll arm.
    pub magic_quiver: bool,
    /// `this.accSharpBarb` — 6160 Sharp Barb (`Player.cs:14501`): +1 ammo damage
    /// on arrow ammo.
    pub acc_sharp_barb: bool,
    /// `this.archery` — buff 16, the Archery Potion (`Player.cs:10018`): ×1.2
    /// speed (capped at 20) for arrow ammo.
    pub archery: bool,
    /// `this.ammoBox` — buff 93, the Ammo Box (`Player.cs:10080`, cleared in
    /// `ResetEffects` `:18450`): the 1/5 roll arm on ANY ammo kind.
    pub ammo_box: bool,
    /// `this.ammoPotion` — buff 112, the Ammo Reservation (`Player.cs:10195`,
    /// cleared `:18451`): the 1/5 roll arm on ANY ammo kind.
    pub ammo_potion: bool,
    /// `this.huntressAmmoCost90` — 3804 Huntress's Jerkin / 4983 Crystal Ninja
    /// Chestplate (`Player.cs:13179`, `:13634`, cleared `:18391`): the 1/10 roll
    /// arm.
    pub huntress_ammo_cost90: bool,
    /// `this.chloroAmmoCost80` — 1002 Chlorophyte Helmet (`Player.cs:13649`,
    /// cleared `:18390`): the 1/5 roll arm.
    pub chloro_ammo_cost80: bool,
    /// `this.ammoCost80` — 1549 Shroomite Breastplate / 3878 Huntress's Alt
    /// Shirt (`Player.cs:13715`, `:13222`, cleared `:18392`): the 1/5 roll arm.
    pub ammo_cost80: bool,
    /// `this.ammoCost75` — 2758 Vortex Breastplate (`Player.cs:13908`, cleared
    /// `:18393`): the 1/4 roll arm.
    pub ammo_cost75: bool,
}

/// The volley/window states the C#'s not-consume arms read at the shot tick
/// (`Player.cs:53553` the Clockwork, `:53626` the Fairy Queen, `:53674` the
/// spray arms), derived by the shoot wire (Phase 4) from its per-frame item
/// timers in INTEGER tick semantics — the C#'s comparisons are integer ones.
///
/// They are pure booleans so `ammo.rs` stays free of tick/seconds conversion:
/// the derivation has exactly one owner. `just_started` is the port's
/// stand-in for `ItemAnimationJustStarted`
/// (`itemAnimation == itemAnimationMax - 1`, `Player.cs:4066`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct VolleyWindows {
    /// `this.ItemAnimationJustStarted` (`Player.cs:4066`) — the shot is the
    /// swing's first (the press tick). `false` on every later shot of a volley.
    pub just_started: bool,
    /// `this.itemAnimation > sItem.useAnimation - 8` (`Player.cs:53626`) — the
    /// Fairy Queen's first-eight-ticks window.
    pub anim_gt_use_anim_minus_8: bool,
    /// `this.itemAnimation < this.itemAnimationMax - sItem.useTime`
    /// (`Player.cs:53674`) — the flamethrower/Clentaminator first-shot window.
    pub anim_lt_anim_max_minus_use_time: bool,
}

/// `PlayerAmmoCyclingMode` (`GameContent/Items/PlayerAmmoCyclingMode.cs`):
/// the accessory-granted pick cycling. `None` is the default
/// (`Player.ResetEffects` `:18479`) and picks the first matching stack.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum AmmoCyclingMode {
    /// No cycling — the first matching slot wins.
    #[default]
    None,
    /// 6168 Ammo Cycler — cycle within the ammo slots only.
    AmmoSlots,
    /// 6169 Ammo Cycler (Full Inventory) — cycle over every matching stack.
    FullInventory,
    /// 6193 Ammo Cycler (Random) — the offset is a `Main.rand` draw.
    Random,
}

/// `Player.ammoCyclingMode` / `ammoCyclingOffset` / `ammoCyclingCooldown`
/// (`Player.cs:1244`, `:1248`, `:1246`) — the three co-located fields the C#
/// keeps on `Player`, carried as one struct so the pick path takes a single
/// seam parameter (the plan's Decision Log). The mode is granted per tick by
/// the accessory scan (`Player.cs:14528-14544`); the offset and the cooldown
/// are written inside a successful, non-dontConsume pick
/// (`Player.cs:53569-53577`, Phase 4) and by the per-tick
/// [`AmmoCycling::tick`] (`Player.cs:28419-28426`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AmmoCycling {
    /// The granted mode (`Player.ammoCyclingMode`).
    pub mode: AmmoCyclingMode,
    /// `Player.ammoCyclingOffset` — the pick's rotating index; `Main.rand.Next()`
    /// in the `Random` mode.
    pub offset: i32,
    /// `Player.ammoCyclingCooldown` — `useAnimation + 120` ticks per successful
    /// pick, decremented every tick.
    pub cooldown: i32,
}

impl AmmoCycling {
    /// `Player.Update_AmmoCycler` (`Player.cs:28419-28426`): the per-tick
    /// decrement, and the offset reset at zero — NOT a per-shot-only hook, so an
    /// idle player's offset returns to 0 while no weapon is held (the C# runs
    /// the method at the head of `Player.Update`, `:24566`, before `ItemCheck`).
    pub fn tick(&mut self) {
        if self.cooldown > 0 {
            self.cooldown -= 1;
            if self.cooldown == 0 {
                self.offset = 0;
            }
        }
    }
}

/// The resolved shot: the projectile the pair fires plus the stat fuse.
#[allow(dead_code)] // Phase 4's shooter wire consumes the resolver
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AmmoShot {
    /// The C#'s `projToShoot` after the whole chain.
    pub projectile: u16,
    /// The AMMO-side damage contribution (`Damage += (int)(num * multiplier)`,
    /// `Player.cs:53552-53555`) — the weapon's own damage stays the caller's, it
    /// is not a parameter here.
    pub damage: i32,
    /// `speed += item.shootSpeed` (`Player.cs:53542`) starting from the weapon's
    /// own `shootSpeed`, through the magicQuiver ×1.1 and the archery ×1.2/cap-20
    /// arms.
    pub speed: f32,
    /// The AMMO-side knockback contribution (`KnockBack += item.knockBack`,
    /// `Player.cs:53568`) — never scaled by `magic_quiver`, because the C#'s
    /// `KnockBack *= 1.1f` runs BEFORE that addition and so scales only the
    /// weapon's own value.
    pub knock_back: f32,
}

/// `Player.PickAmmo`'s projToShoot resolution chain + stat fuse
/// (`Player.cs:53458-53568`), as a pure function: `weapon` ⇒ `sItem`, `ammo` ⇒
/// the picked ammo item (`item`), `use_ammo` ⇒ `sItem.useAmmo`, `ammo_damage` ⇒
/// `item.damage`, `ammo_shoot_speed` ⇒ `item.shootSpeed`, `ammo_knock_back` ⇒
/// `item.knockBack`, `weapon_shoot`/`weapon_shoot_speed` ⇒ the `projToShoot` and
/// `speed` values `ItemCheck_Shoot` seeds the call with (`Player.cs:47058`,
/// `:47059`), `selected_item` ⇒ `inventory[selectedItem].type` (`:53533`).
///
/// Order is the C#'s, and the order is load-bearing: the pair table wins over
/// the ammo's own `shoot`, and every per-weapon override runs after it.
/// `None` = nothing to resolve — a melee weapon (`use_ammo <= 0`, the
/// `if (sItem.useAmmo > 0)` gate at `Player.cs:47069`) or an ammo kind the
/// registry does not carry.
#[allow(dead_code)] // Phase 4's shooter wire consumes the resolver
pub fn resolve_projectile(
    weapon: u16,
    ammo: u16,
    use_ammo: i32,
    ammo_damage: i32,
    ammo_shoot_speed: f32,
    ammo_knock_back: f32,
    weapon_shoot: i32,
    weapon_shoot_speed: f32,
    selected_item: u16,
    flags: &AmmoFlags,
) -> Option<AmmoShot> {
    if use_ammo <= 0 {
        return None;
    }
    // The pick already read the ammo's row (`pick`'s `d.ammo == use_ammo`), so a
    // missing row means the kind is not ported at all: refuse the shot rather
    // than silently resolving to the launcher's own projectile.
    let Some(ammo_def) = crate::registry::get(ammo) else {
        return None;
    };
    // `Item.shoot` (`Player.cs:53483-53494`) — the ammo's own projectile.
    let ammo_shoot = ammo_def.projectile.map_or(0, |p| p as i32);
    // `int num = item.damage` (`:53469`); only the molten/sand/SharpBarb arms
    // mutate it.
    let mut num = ammo_damage;
    // `int projToShoot = sItem.shoot` (`:47058`) — the chain's one local.
    let mut proj = weapon_shoot;
    if let Some(picked) = specific_launcher_ammo_match(weapon, ammo) {
        // `:53471-53474` — the first arm, and it excludes everything below.
        proj = picked as i32;
    } else if use_ammo == AMMO_ROCKET {
        // `:53483-53486` — a rocket launcher the table does not cover (the
        // Electrosphere Launcher 2796) ADDS the ammo's own `shoot`.
        proj += ammo_shoot;
    } else if use_ammo == AMMO_SOLUTION {
        // `:53487-53490` — the Clentaminator's path: its own projectile plus the
        // solution's offset.
        proj += ammo_shoot;
    } else {
        // `:53491-53494` — otherwise the ammo's own `shoot`, when it has one.
        if ammo_shoot > 0 {
            proj = ammo_shoot;
        }
    }
    // The per-weapon overrides, in the C#'s code order.
    // `:53495-53498` — the Hellwing Bow, only for the wooden-arrow default.
    if weapon == 3019 && proj == 1 {
        proj = 485;
    }
    // `:53499-53502` — the Shadowflame Bow: unconditional.
    if weapon == 3052 {
        proj = 495;
    }
    // `:53503-53506` — the Fairy Queen's item: only for the wooden-arrow default.
    if weapon == 4953 && proj == 1 {
        proj = 932;
    }
    // `:53507-53510` — the Blood Rain Bow: unconditional.
    if weapon == 4381 {
        proj = 819;
    }
    // `:53511-53514` — the Skeleton Bow: only for the Bone Arrow From Merchant.
    if weapon == 4058 && proj == 474 {
        proj = 117;
    }
    if proj == 42 {
        // `:53515-53532` — the sand-block kinds, when the pair already resolves
        // to the Sandgun's own projectile 42; each adds 5 ammo damage.
        if ammo == 370 {
            proj = 65;
            num += 5;
        } else if ammo == 408 {
            proj = 68;
            num += 5;
        } else if ammo == 1246 {
            proj = 354;
            num += 5;
        }
    }
    // `:53533-53536` — the Bees' Knees: gated on the SELECTED item's kind, not
    // on `sItem`.
    if selected_item == 2888 && proj == 1 {
        proj = 469;
    }
    // `:53537-53541` — the Molten Quiver: converts the wooden arrow and adds 2.
    if flags.has_molten_quiver && proj == 1 {
        proj = 2;
        num += 2;
    }
    // The stat fuse, `:53542-53568`.
    let mut speed = weapon_shoot_speed + ammo_shoot_speed;
    if flags.magic_quiver && is_arrow_ammo(use_ammo) {
        // `:53543-53547`: `speed *= 1.1f`. The `KnockBack *= 1.1f` next to it
        // runs before `KnockBack += item.knockBack` (`:53568`), so it scales the
        // weapon's own knockback — held by the caller, not by this struct.
        speed *= 1.1;
    }
    if flags.acc_sharp_barb && is_arrow_ammo(use_ammo) {
        // `:53548-53551`
        num += 1;
    }
    let mut damage = if num > 0 { num } else { 0 };
    if proj == 285 {
        // `:53556-53559` — the Nanite bullets: ×1.25 on the fused damage. This
        // resolver owns the ammo side only, so the caller's weapon-side damage
        // takes the same factor (see the module's Phase-4 note).
        damage = (damage as f32 * 1.25) as i32;
    }
    if is_arrow_ammo(use_ammo) && flags.archery && speed < 20.0 {
        // `:53560-53567` — the Archery Potion: ×1.2, capped at 20.
        speed *= 1.2;
        if speed > 20.0 {
            speed = 20.0;
        }
    }
    // `:53568` — the ammo's own knockback rides on after every arm above.
    Some(AmmoShot {
        projectile: proj as u16,
        damage,
        speed,
        knock_back: ammo_knock_back,
    })
}

/// One `Main.rand.Next(m) == 0` roll — the 1/m save family. The draw is spent
/// only when the caller's gate already passed, exactly like the C#'s `&&`
/// (`Player.cs:53595`, `:53599`, `:53643`, …): a gate that fails spends nothing,
/// which is what keeps one pick's draw count equal to the vanilla's.
#[inline]
fn one_in(m: u32, rng: &mut u64) -> bool {
    next(rng) % m == 0
}

/// One `Main.rand.Next(m) != 0` roll — the (m-1)/m save family
/// (`Player.cs:53579`, `:53587`, `:53623`, `:53627`).
#[inline]
fn all_but_one_in(m: u32, rng: &mut u64) -> bool {
    next(rng) % m != 0
}

/// The not-consume rolls — `Player.PickAmmo`'s whole flag chain
/// (`Player.cs:53578-53674`). `true` = the shot skips the drain
/// (`:53675-53682`). The chain is ported as a flat sequence of one-line arms in
/// the source's order; the order is load-bearing (see the module doc), so a
/// re-sort is a behaviour change, not a style change. The arms short-circuit:
/// an arm that does not apply draws nothing.
///
/// `shooter` ⇒ `sItem`, `ammo_kind` ⇒ `sItem.useAmmo` (the ammo CATEGORY the
/// `magicQuiver` arm reads), `projectile` ⇒ `projToShoot` (the spray arm's key),
/// `windows` are the pre-derived volley states, `flags` the accessory/buff
/// grants, `dont_consume` the C#'s `dontConsume` argument — the value `flag`
/// starts from (`ItemID.Sets.gunProj`, `ItemID.cs:1327`: the four endgame items
/// never drain, and their own rolls still spend the vanilla draws, so the
/// argument seeds the result rather than skipping the chain).
pub fn not_consume(
    shooter: u16,
    ammo_kind: i32,
    projectile: u16,
    windows: &VolleyWindows,
    flags: &AmmoFlags,
    dont_consume: bool,
    rng: &mut u64,
) -> bool {
    // `bool flag = dontConsume` (`Player.cs:53578`).
    let mut flag = dont_consume;
    // `:53579` — the Vortex Beater: 2/3.
    if shooter == 3475 && all_but_one_in(3, rng) {
        flag = true;
    }
    // `:53583` — the Celebration Mk2: 1/2.
    if shooter == 3930 && one_in(2, rng) {
        flag = true;
    }
    // `:53587` — the Phantasm: 2/3.
    if shooter == 3540 && all_but_one_in(3, rng) {
        flag = true;
    }
    // `:53591` — the Clentaminator 2: 1/3.
    if shooter == 5134 && one_in(3, rng) {
        flag = true;
    }
    // `:53595` — the Magic Quiver: 1/5 on arrow/stake ammo only
    // (`IsArrow[sItem.useAmmo]`, `AmmoID.cs:115`).
    if flags.magic_quiver && is_arrow_ammo(ammo_kind) && one_in(5, rng) {
        flag = true;
    }
    // `:53599` — the Ammo Box (buff 93): 1/5 on ANY ammo.
    if flags.ammo_box && one_in(5, rng) {
        flag = true;
    }
    // `:53603` — the Ammo Reservation (buff 112): 1/5 on ANY ammo.
    if flags.ammo_potion && one_in(5, rng) {
        flag = true;
    }
    // `:53607` — the Candy Corn Rifle: 1/3.
    if shooter == 1782 && one_in(3, rng) {
        flag = true;
    }
    // `:53611` — the Minishark: 1/3.
    if shooter == 98 && one_in(3, rng) {
        flag = true;
    }
    // `:53615` — the Gatligator: 1/2.
    if shooter == 2270 && one_in(2, rng) {
        flag = true;
    }
    // `:53619` — the Megashark: 1/2.
    if shooter == 533 && one_in(2, rng) {
        flag = true;
    }
    // `:53623` — the Chain Gun: 2/3.
    if shooter == 1929 && all_but_one_in(3, rng) {
        flag = true;
    }
    // `:53627` — the SDMG: 2/3.
    if shooter == 1553 && all_but_one_in(3, rng) {
        flag = true;
    }
    // `:53631` — the Clockwork: every shot after the swing's first saves
    // (its useTime 4 < useAnimation 12 fires 3 shots per swing, so 1 ammo per 3
    // shots).
    if shooter == 434 && !windows.just_started {
        flag = true;
    }
    // `:53635` — the Fairy Queen's item: the first eight ticks of the animation
    // save (anim 30 / useTime 2 → 4 of the 15 shots).
    if shooter == 4953 && windows.anim_gt_use_anim_minus_8 {
        flag = true;
    }
    // `:53639` — the Ale Throwing Glove: DEAD, not ported. Its `useAmmo == 0`
    // means `PickAmmo` never runs for it (`Player.cs:47069`), so the roll cannot
    // fire.
    // `:53643` — the Huntress's Jerkin / the Crystal Ninja Chestplate: 1/10.
    if flags.huntress_ammo_cost90 && one_in(10, rng) {
        flag = true;
    }
    // `:53647` — the Chlorophyte Helmet: 1/5.
    if flags.chloro_ammo_cost80 && one_in(5, rng) {
        flag = true;
    }
    // `:53651` — the Shroomite Breastplate / the Huntress's Alt Shirt: 1/5.
    if flags.ammo_cost80 && one_in(5, rng) {
        flag = true;
    }
    // `:53655` — the Vortex Breastplate: 1/4.
    if flags.ammo_cost75 && one_in(4, rng) {
        flag = true;
    }
    // `:53659` — the Snowball Cannon's remix-world arm: PARKED, not ported (no
    // celebration-seed stand-in, see the module doc).
    // `:53663` — the Flames projectile: the spray's first shot consumes, the
    // rest save (`itemAnimation < itemAnimationMax - useTime`).
    if projectile == 85 && windows.anim_lt_anim_max_minus_use_time {
        flag = true;
    }
    // `:53667` — the two Clentaminators: the same first-shot window.
    if (shooter == 779 || shooter == 5134) && windows.anim_lt_anim_max_minus_use_time {
        flag = true;
    }
    // `:53671` — the Acorn Slingshot: the ONLY arm that clears, and the LAST
    // one, so it beats the Ammo Box / Ammo Reservation saves above it.
    if shooter == 5629 {
        flag = false;
    }
    flag
}

/// `Main.rand.Next(n)` against the stream (the splitmix mix of `rain.rs`).
fn next(rng: &mut u64) -> u32 {
    *rng = rng.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut z = *rng;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    (z ^ (z >> 31)) as u32
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::inv::EquipSlots;

    fn inv_with(entries: &[(usize, u16, u32)]) -> [ItemSlot; INV] {
        let mut inv = [crate::item::EMPTY_SLOT; INV];
        for &(i, kind, count) in entries {
            inv[i] = ItemSlot {
                kind,
                name: crate::registry::name(kind),
                count,
                prefix: 0,
            };
        }
        inv
    }

    fn equip_with_ammo(entries: &[(usize, u16, u32)]) -> EquipSlots {
        let mut e = EquipSlots::empty();
        for &(i, kind, count) in entries {
            e.ammo[i] = ItemSlot {
                kind,
                name: crate::registry::name(kind),
                count,
                prefix: 0,
            };
        }
        e
    }

    #[test]
    fn the_pick_walks_coins_then_ammo_then_the_grid() {
        // `AmmoSlotOrder_Default` (`Player.cs:57141`): the coins 50-53, the
        // ammo 54-57, the grid 0-53 — the first `ammo == useAmmo` match wins.
        let inv = inv_with(&[(5, 40, 10), (10, 40, 99)]);
        let equip = equip_with_ammo(&[(0, 40, 3)]);
        let pick = super::pick(&inv, &equip, 40, None).expect("ammo exists");
        assert_eq!(pick, AmmoSource::Ammo(0), "the ammo slot precedes the grid");

        // No ammo-slot entry: the grid's first match wins.
        let equip = EquipSlots::empty();
        let pick = super::pick(&inv, &equip, 40, None).expect("grid ammo");
        assert_eq!(pick, AmmoSource::Grid(5), "the grid order is slot order");

        // The coin gun's coins: useAmmo 71 matches the coins slots first.
        let coin_inv = inv_with(&[]);
        let mut equip = EquipSlots::empty();
        equip.coins[1] = ItemSlot {
            kind: 71,
            name: "Copper Coin",
            count: 30,
            prefix: 0,
        };
        let pick = super::pick(&coin_inv, &equip, 71, None).expect("coins");
        assert_eq!(pick, AmmoSource::Coin(1));
    }

    #[test]
    fn the_pick_refuses_the_wrong_ammo_and_the_empty_stacks() {
        // The Musket (96) wants Bullet (97); the arrows (ammo 40) don't match.
        let inv = inv_with(&[(3, 40, 99)]);
        let equip = EquipSlots::empty();
        assert_eq!(super::pick(&inv, &equip, 97, None), None);
        // An empty stack doesn't match (`item.stack > 0`, `Player.cs:53699`).
        let inv = inv_with(&[(3, 97, 0)]);
        assert_eq!(super::pick(&inv, &equip, 97, None), None);
        // useAmmo <= 0 refuses outright (the melee weapons never pick).
        assert_eq!(super::pick(&inv, &equip, 0, None), None);
    }

    /// A cycling state with one field set (the mode defaults to `None`, so the
    /// struct literal spells the mode and keeps the rest defaulted).
    fn cycling(mode: AmmoCyclingMode, offset: i32) -> AmmoCycling {
        AmmoCycling {
            mode,
            offset,
            cooldown: 0,
        }
    }

    #[test]
    fn the_cycling_pick_rotates_behind_the_offset() {
        // `PickAmmo_IterateRange` (`Player.cs:53685-53726`): with cycling on,
        // count the matching stacks, `num = offset % count`, skip that many.
        // Two arrow stacks in the ammo slots: offset 0 takes the first, 1 the
        // second, and the offset wraps modulo the count (no stack is left
        // unreachable, and no pick ever runs off the end).
        let equip = equip_with_ammo(&[(0, 40, 5), (1, 40, 7)]);
        let inv = inv_with(&[(6, 40, 9)]);
        let ammo_slots = cycling(AmmoCyclingMode::AmmoSlots, 0);
        assert_eq!(
            super::pick(&inv, &equip, 40, Some(&ammo_slots)),
            Some(AmmoSource::Ammo(0))
        );
        let ammo_slots = cycling(AmmoCyclingMode::AmmoSlots, 1);
        assert_eq!(
            super::pick(&inv, &equip, 40, Some(&ammo_slots)),
            Some(AmmoSource::Ammo(1))
        );
        let ammo_slots = cycling(AmmoCyclingMode::AmmoSlots, 2);
        assert_eq!(
            super::pick(&inv, &equip, 40, Some(&ammo_slots)),
            Some(AmmoSource::Ammo(0))
        );
        let ammo_slots = cycling(AmmoCyclingMode::AmmoSlots, 5);
        assert_eq!(
            super::pick(&inv, &equip, 40, Some(&ammo_slots)),
            Some(AmmoSource::Ammo(1))
        );
        // `FullInventory` counts over the whole default order (coins → ammo →
        // grid): the third arrow stack in the grid is reachable only through
        // it, and the offset picks it at 2.
        let full = cycling(AmmoCyclingMode::FullInventory, 2);
        assert_eq!(
            super::pick(&inv, &equip, 40, Some(&full)),
            Some(AmmoSource::Grid(6))
        );
        let full = cycling(AmmoCyclingMode::FullInventory, 3);
        assert_eq!(
            super::pick(&inv, &equip, 40, Some(&full)),
            Some(AmmoSource::Ammo(0))
        );
        let full = cycling(AmmoCyclingMode::FullInventory, 4);
        assert_eq!(
            super::pick(&inv, &equip, 40, Some(&full)),
            Some(AmmoSource::Ammo(1))
        );
        let full = cycling(AmmoCyclingMode::FullInventory, 5);
        assert_eq!(
            super::pick(&inv, &equip, 40, Some(&full)),
            Some(AmmoSource::Grid(6))
        );
        // `Random` rides the same order (the offset is the cycler's draw) and
        // an empty stack never counts (`item.stack > 0`, `:53700`).
        let equip = equip_with_ammo(&[(0, 40, 0), (1, 40, 7)]);
        let random = cycling(AmmoCyclingMode::Random, 0);
        assert_eq!(
            super::pick(&inv, &equip, 40, Some(&random)),
            Some(AmmoSource::Ammo(1))
        );
        let random = cycling(AmmoCyclingMode::Random, 1);
        assert_eq!(
            super::pick(&inv, &equip, 40, Some(&random)),
            Some(AmmoSource::Grid(6))
        );
    }

    #[test]
    fn the_cycling_pick_falls_through_and_refuses_empty_ranges() {
        // `PickAmmo_PickAmmoItem` (`Player.cs:53728-53753`): the AmmoSlots pass
        // sees no match in the ammo slots, so it falls through to the default
        // order with `allowAmmoCycling: false` — the FIRST match wins, no
        // matter the offset.
        let equip = equip_with_ammo(&[]);
        let inv = inv_with(&[(2, 40, 3), (7, 40, 4)]);
        for offset in [0, 1, 5, 99] {
            let ammo_slots = cycling(AmmoCyclingMode::AmmoSlots, offset);
            assert_eq!(
                super::pick(&inv, &equip, 40, Some(&ammo_slots)),
                Some(AmmoSource::Grid(2)),
                "AmmoSlots' fall-through ignores the offset (offset {offset})"
            );
        }
        // `num2 == 0` → `null` (`:53702-53705`): no matching stack anywhere and
        // the cycler armed → the shot is refused.
        let empty = inv_with(&[(3, 97, 9)]);
        let full = cycling(AmmoCyclingMode::FullInventory, 0);
        assert_eq!(super::pick(&empty, &equip, 40, Some(&full)), None);
        // A cycling mode other than `None` never widens the pick: with only a
        // grid stack the AmmoSlots mode still lands on it.
        let inv = inv_with(&[(9, 40, 1)]);
        let random = cycling(AmmoCyclingMode::Random, 7);
        assert_eq!(
            super::pick(&inv, &equip, 40, Some(&random)),
            Some(AmmoSource::Grid(9))
        );
    }

    #[test]
    fn the_pick_with_a_disarmed_cycler_is_the_first_match() {
        // The vanilla default (`Player.ResetEffects` `:18479`): `None` mode —
        // either passed as `None` or carried on the struct — picks the first
        // match exactly like the pre-Phase-4 `pick`.
        let equip = equip_with_ammo(&[(1, 40, 7), (3, 40, 2)]);
        let inv = inv_with(&[(4, 40, 9)]);
        let armed = cycling(AmmoCyclingMode::None, 3);
        assert_eq!(
            super::pick(&inv, &equip, 40, None),
            Some(AmmoSource::Ammo(1))
        );
        assert_eq!(
            super::pick(&inv, &equip, 40, Some(&armed)),
            Some(AmmoSource::Ammo(1)),
            "the None mode never rotates"
        );
        // A coin gun keeps its coins-first order under the AmmoSlots mode
        // (`AmmoSlotOrder_CoinsOnly`, `Player.cs:57143`).
        let mut equip = EquipSlots::empty();
        equip.coins[2] = ItemSlot {
            kind: 71,
            name: "Copper Coin",
            count: 50,
            prefix: 0,
        };
        equip.coins[3] = ItemSlot {
            kind: 72,
            name: "Silver Coin",
            count: 3,
            prefix: 0,
        };
        equip.ammo[0] = ItemSlot {
            kind: 71,
            name: "Copper Coin",
            count: 9,
            prefix: 0,
        };
        let inv = inv_with(&[]);
        let ammo_slots = cycling(AmmoCyclingMode::AmmoSlots, 1);
        assert_eq!(
            super::pick(&inv, &equip, 71, Some(&ammo_slots)),
            Some(AmmoSource::Coin(3)),
            "the coin order counts the coin slots"
        );
    }

    #[test]
    fn the_drain_decrements_and_turns_to_air() {
        // `Player.cs:53667-53672`: `stack--`, `TurnToAir` at 0.
        let mut inv = inv_with(&[(3, 97, 2)]);
        let mut equip = EquipSlots::empty();
        let src = pick(&inv, &equip, 97, None).unwrap();
        assert!(drain(src, &mut inv, &mut equip));
        assert_eq!(inv[3].count, 1);
        assert!(drain(src, &mut inv, &mut equip));
        assert_eq!(inv[3].count, 0);
        assert_eq!(inv[3].kind, 0, "TurnToAir");
    }

    #[test]
    fn the_drain_hits_the_equip_slots() {
        let mut inv = inv_with(&[]);
        let mut equip = equip_with_ammo(&[(2, 40, 5)]);
        let src = pick(&inv, &equip, 40, None).unwrap();
        assert_eq!(src, AmmoSource::Ammo(2));
        assert!(drain(src, &mut inv, &mut equip));
        assert_eq!(equip.ammo[2].count, 4);
    }

    #[test]
    fn the_not_consume_rolls_are_seed_deterministic() {
        // The shooters' rolls (`Player.cs:53611`, `:53619`…) — the same seed gives
        // the same save pattern.
        let mut a = 42u64;
        let mut b = 42u64;
        let a_seq: Vec<bool> = (0..30).map(|_| roll(98, &mut a)).collect();
        let b_seq: Vec<bool> = (0..30).map(|_| roll(98, &mut b)).collect();
        assert_eq!(a_seq, b_seq, "byte-equal patterns");
        assert!(a_seq.contains(&true), "the Minishark saves sometimes");
        // The shooters without rolls never save.
        let mut r = 7u64;
        assert!((0..50).all(|_| !roll(39, &mut r)), "the wooden bow");
    }

    // ---- the not-consume chain (`Player.cs:53578-53674`) ----

    /// The Phase-3 windows: the plain "not a volley, not the first shot of one"
    /// baseline the chain reads.
    fn windows() -> VolleyWindows {
        VolleyWindows::default()
    }

    /// One roll with the baseline windows, no flags and no `dontConsume`.
    fn roll(shooter: u16, rng: &mut u64) -> bool {
        not_consume(
            shooter,
            AMMO_ARROW,
            0,
            &windows(),
            &AmmoFlags::default(),
            false,
            rng,
        )
    }

    /// One roll with the flag-bearing flags and the baseline windows.
    fn roll_flags(shooter: u16, flags: &AmmoFlags, rng: &mut u64) -> bool {
        not_consume(shooter, AMMO_ARROW, 0, &windows(), flags, false, rng)
    }

    #[test]
    fn test_DW_3_1_the_four_endgame_shooters_save_at_their_vanilla_rates() {
        // `Player.cs:53579`, `:53583`, `:53587`, `:53591`: 3475 2/3, 3930 1/2,
        // 3540 2/3, 5134 1/3. Each shooter's save frequency over a long stream
        // must land on its vanilla rate (the Minishark-style determinism test's
        // pattern, now per-arm).
        let cases: [(u16, usize, f64, &str); 4] = [
            (3475, 300, 2.0 / 3.0, "the Vortex Beater's 2/3"),
            (3930, 300, 1.0 / 2.0, "the Celebration Mk2's 1/2"),
            (3540, 300, 2.0 / 3.0, "the Phantasm's 2/3"),
            (5134, 300, 1.0 / 3.0, "the Clentaminator 2's 1/3"),
        ];
        for (shooter, shots, rate, what) in cases {
            let mut rng = 0xC0FF_EE12u64;
            let saves = (0..shots).filter(|_| roll(shooter, &mut rng)).count();
            let got = saves as f64 / shots as f64;
            assert!(
                (got - rate).abs() < 0.05,
                "{what}: saved {saves}/{shots} = {got:.3}, want ~{rate:.3}"
            );
        }
        // The two 2/3 shooters are NOT each other's swap: 3475 saves when the
        // draw is non-zero, 3930 when it is zero — the same stream therefore
        // gives a different pattern for each.
        let mut a = 99u64;
        let mut b = 99u64;
        let a_seq: Vec<bool> = (0..120).map(|_| roll(3475, &mut a)).collect();
        let b_seq: Vec<bool> = (0..120).map(|_| roll(3930, &mut b)).collect();
        assert_ne!(a_seq, b_seq, "2/3 (`!= 0`) and 1/2 (`== 0`) differ");
    }

    #[test]
    fn test_DW_3_1_the_flag_arms_draw_nothing_while_their_flag_is_false() {
        // `Player.cs:53595`, `:53599`, `:53603`, `:53643`, `:53647`, `:53651`,
        // `:53655`: every flag arm's `&&` short-circuits, so with every flag
        // false the chain spends NO draw and returns false — the C#'s `&&`
        // never evaluates `Main.rand.Next`, and neither does the port.
        for (shooter, ammo_kind, projectile) in
            [(39u16, AMMO_ARROW, 0u16), (96, 97, 14), (266, 169, 42)]
        {
            let mut used = 12345u64;
            let before = used;
            for _ in 0..20 {
                assert!(
                    !not_consume(
                        shooter,
                        ammo_kind,
                        projectile,
                        &windows(),
                        &AmmoFlags::default(),
                        false,
                        &mut used
                    ),
                    "shooter {shooter}: no save without a flag or an item arm"
                );
            }
            assert_eq!(used, before, "shooter {shooter}: the stream is untouched");
        }
    }

    #[test]
    fn test_DW_3_1_the_flag_arms_save_at_their_vanilla_rates() {
        // `Player.cs:53595`/`:53599`/`:53603`/`:53647`/`:53651` 1/5, `:53655`
        // 1/4, `:53643` 1/10, `:53595` (the quiver) 1/5. The plain wooden arrow
        // is `use_ammo == Arrow`, so every arm applies to all 600 shots.
        let mut quiver = AmmoFlags::default();
        quiver.magic_quiver = true;
        let mut box_ = AmmoFlags::default();
        box_.ammo_box = true;
        let mut potion = AmmoFlags::default();
        potion.ammo_potion = true;
        let mut huntress = AmmoFlags::default();
        huntress.huntress_ammo_cost90 = true;
        let mut chloro = AmmoFlags::default();
        chloro.chloro_ammo_cost80 = true;
        let mut cost80 = AmmoFlags::default();
        cost80.ammo_cost80 = true;
        let mut cost75 = AmmoFlags::default();
        cost75.ammo_cost75 = true;
        let cases: [(&AmmoFlags, f64, &str); 7] = [
            (&quiver, 0.2, "the Magic Quiver's 1/5"),
            (&box_, 0.2, "the Ammo Box's 1/5"),
            (&potion, 0.2, "the Ammo Reservation's 1/5"),
            (&huntress, 0.1, "the Huntress set's 1/10"),
            (&chloro, 0.2, "the Chlorophyte Helmet's 1/5"),
            (&cost80, 0.2, "the Shroomite Breastplate's 1/5"),
            (&cost75, 0.25, "the Vortex Breastplate's 1/4"),
        ];
        for (flags, rate, what) in cases {
            let mut rng = 0x5EED_1234u64;
            let saves = (0..600).filter(|_| roll_flags(39, flags, &mut rng)).count();
            let got = saves as f64 / 600.0;
            assert!(
                (got - rate).abs() < 0.03,
                "{what}: saved {saves}/600 = {got:.3}, want ~{rate:.3}"
            );
        }
        // The flags are independent: two arms on the same shot spend two draws
        // and each keeps its own rate's worth of saves (the union exceeds either
        // single arm's 1/5 or 1/4).
        let mut two = AmmoFlags::default();
        two.ammo_box = true;
        two.ammo_cost75 = true;
        let mut rng = 77u64;
        let saves = (0..600).filter(|_| roll_flags(39, &two, &mut rng)).count();
        assert!(
            saves > 600 * 4 / 25,
            "two arms save more often than one (1/5 ∪ 1/4 ≈ 2/5): {saves}"
        );
        assert!(saves <= 600, "the chain never invents saves");
    }

    #[test]
    fn test_DW_3_1_the_four_new_arms_match_the_existing_six() {
        // Phase 3's four item-keyed arms sit BEFORE the six Phase-1 arms in the
        // chain, but an unmatched item arm is stream-invisible: a Minishark shot
        // with 3475-3540/3930/5134 rollers unset spends exactly ONE draw, the
        // one its own arm takes — the arms the brief's new set added did not
        // change the six guns' streams.
        let mut a = 4242u64;
        let mut b = 4242u64;
        for _ in 0..200 {
            assert_eq!(roll(98, &mut a), roll(98, &mut b), "the Minishark's stream");
        }
        // The two 1/3 additions (3930, 5134) draw from the same modulus as the
        // Minishark but only for their own kinds: a 3930 shot and a 98 shot from
        // one seed differ, because the arms are keyed on the kind.
        let mut c = 4242u64;
        let mut d = 4242u64;
        let c_seq: Vec<bool> = (0..120).map(|_| roll(3930, &mut c)).collect();
        let d_seq: Vec<bool> = (0..120).map(|_| roll(98, &mut d)).collect();
        assert_ne!(c_seq, d_seq, "the kind gate, not the modulus, decides");
    }

    #[test]
    fn test_DW_3_2_the_acorn_slingshot_clears_the_ammo_box_save() {
        // `Player.cs:53599` (the Ammo Box's 1/5 save) is wiped by
        // `:53671`'s `flag = false`, which runs LAST: a 5629 shot drains no
        // matter what the ammoBox/ammoPotion arms did above it.
        let mut box_ = AmmoFlags::default();
        box_.ammo_box = true;
        let mut potion = AmmoFlags::default();
        potion.ammo_potion = true;
        let mut rng = 31337u64;
        // With the Acorn as the shooter the chain always drains — over a stream
        // long enough that the ammoBox arm would have saved ~120 times.
        let saved = (0..600)
            .filter(|_| not_consume(5629, AMMO_ARROW, 0, &windows(), &box_, false, &mut rng))
            .count();
        assert_eq!(saved, 0, "the Acorn's clear beats the Ammo Box save");
        // The same flags with a NON-5629 shooter do save — the arms above the
        // clear are not inert, they are only overridden by it.
        let mut rng = 31337u64;
        let others = (0..600).filter(|_| roll_flags(39, &box_, &mut rng)).count();
        assert!(others > 60, "the Ammo Box saves ~1/5 for a normal shooter");
        // The potion half behaves the same, and a 5629 shot with BOTH flags
        // still drains.
        let mut rng = 7u64;
        assert!(
            !(0..200).any(|_| not_consume(
                5629,
                AMMO_ARROW,
                0,
                &windows(),
                &potion,
                false,
                &mut rng,
            )),
            "the clear also beats the Ammo Reservation"
        );
    }

    #[test]
    fn test_DW_3_2_the_clockwork_saves_only_after_the_first_shot() {
        // `Player.cs:53631`: `sItem.type == 434 && !ItemAnimationJustStarted`
        // (`:4066` — the press tick's first shot has `itemAnimation == max - 1`),
        // so the Clockwork's first shot drains and its later volley shots save.
        // With no volley windows at all the arm cannot fire.
        let off = VolleyWindows {
            just_started: true,
            ..VolleyWindows::default()
        };
        let on = VolleyWindows {
            just_started: false,
            ..VolleyWindows::default()
        };
        let mut rng = 2024u64;
        for _ in 0..50 {
            assert!(
                !not_consume(
                    434,
                    AMMO_ARROW,
                    0,
                    &off,
                    &AmmoFlags::default(),
                    false,
                    &mut rng
                ),
                "the swing's first shot consumes"
            );
        }
        let mut rng = 2024u64;
        assert!(
            (0..50).all(|_| not_consume(
                434,
                AMMO_ARROW,
                0,
                &on,
                &AmmoFlags::default(),
                false,
                &mut rng
            )),
            "every later shot saves — no draw is spent either way"
        );
        // The arm is kind-gated: another shooter with the same window saves
        // nothing.
        let mut rng = 5u64;
        assert!(
            (0..20).all(|_| !roll_bare(96, &on, &mut rng)),
            "only the Clockwork rides the window"
        );
    }

    /// A roll with the baseline flags (the window-only arms' isolation helper).
    fn roll_bare(shooter: u16, windows: &VolleyWindows, rng: &mut u64) -> bool {
        not_consume(
            shooter,
            AMMO_ARROW,
            0,
            windows,
            &AmmoFlags::default(),
            false,
            rng,
        )
    }

    #[test]
    fn test_DW_3_2_the_fairy_queen_saves_only_inside_the_eight_tick_window() {
        // `Player.cs:53635`: `itemAnimation > sItem.useAnimation - 8` — with the
        // Fairy Queen's anim 30 / useTime 2 the window is anim 29/27/25/23, i.e.
        // 4 of the 15 shots save. The wire derives the boolean; here the two
        // edges of that derived window prove the gate.
        let inside = VolleyWindows {
            anim_gt_use_anim_minus_8: true,
            ..VolleyWindows::default()
        };
        let outside = VolleyWindows {
            anim_gt_use_anim_minus_8: false,
            ..VolleyWindows::default()
        };
        let mut rng = 1u64;
        for _ in 0..50 {
            assert!(
                not_consume(
                    4953,
                    AMMO_ARROW,
                    0,
                    &inside,
                    &AmmoFlags::default(),
                    false,
                    &mut rng
                ),
                "anim 29 > 22 saves"
            );
        }
        let mut rng = 1u64;
        for _ in 0..50 {
            assert!(
                !not_consume(
                    4953,
                    AMMO_ARROW,
                    0,
                    &outside,
                    &AmmoFlags::default(),
                    false,
                    &mut rng
                ),
                "anim 21 <= 22 drains"
            );
        }
        // Kind-gated: the same window on another shooter saves nothing.
        let mut rng = 9u64;
        assert!(
            (0..20).all(|_| !roll_bare(96, &inside, &mut rng)),
            "only the Fairy Queen's item rides this window"
        );
    }

    #[test]
    fn test_DW_3_2_the_spray_arms_save_only_after_the_first_shot() {
        // `Player.cs:53663` (projToShoot == 85, the Flames) and `:53667`
        // (779/5134, the Clentaminators): `itemAnimation < itemAnimationMax -
        // useTime` is FALSE on a spray's first shot (anim = max - 1) and true for
        // every later one — the first solution/gel shot consumes, the rest save.
        let first = VolleyWindows {
            anim_lt_anim_max_minus_use_time: false,
            just_started: true,
            ..VolleyWindows::default()
        };
        let later = VolleyWindows {
            anim_lt_anim_max_minus_use_time: true,
            just_started: false,
            ..VolleyWindows::default()
        };
        // The 85 arm keys on the RESOLVED projectile and applies to any shooter;
        // the 779/5134 arm keys on the shooter and applies to any projectile.
        // Both are inert outside their key, so a shot with neither never saves.
        let cases: [(u16, u16, bool, &str); 5] = [
            (96, 85, true, "the flamethrower's resolved projectile"),
            (96, 0, false, "neither key"),
            (779, 0, true, "the Clentaminator's own arm"),
            (
                5134,
                42,
                true,
                "the Clentaminator 2, any resolved projectile",
            ),
            (779, 85, true, "both keys on one shot"),
        ];
        for (shooter, projectile, want, what) in cases {
            let mut rng = 64u64;
            let got = (0..30)
                .filter(|_| {
                    not_consume(
                        shooter,
                        AMMO_ARROW,
                        projectile,
                        &later,
                        &AmmoFlags::default(),
                        false,
                        &mut rng,
                    )
                })
                .count();
            assert_eq!(
                got > 0,
                want,
                "shooter {shooter} + proj {projectile}: {what}"
            );
        }
        // The first shot of the spray consumes, for both window arms — checked on
        // the two shooters that carry no item-keyed roll of their own. The
        // Clentaminator 2 (5134) is deliberately not in this half: its own 1/3
        // arm (`Player.cs:53591`) can save the first shot independently of the
        // window, so its drain rate is the arm's product, not the window's.
        for (shooter, projectile) in [(96u16, 85u16), (779, 0)] {
            let mut rng = 64u64;
            let saved = (0..30)
                .filter(|_| {
                    not_consume(
                        shooter,
                        AMMO_ARROW,
                        projectile,
                        &first,
                        &AmmoFlags::default(),
                        false,
                        &mut rng,
                    )
                })
                .count();
            assert_eq!(saved, 0, "shooter {shooter}: the spray's first shot drains");
        }
    }

    #[test]
    fn test_the_rng_golden_stream_spends_the_vanilla_draw_count() {
        // The determinism invariant, pinned against the stream itself: one pick
        // from a fixed seed spends EXACTLY the vanilla's draw count for that
        // pick, so a later refactor cannot silently shift the stream.
        //  - `434` Clockwork: `!ItemAnimationJustStarted` draws nothing (0) — the
        //    arm has no `Main.rand` (`Player.cs:53631`);
        //  - each of the six guns spends 1 (their own roll);
        //  - each of the four gunProj items spends 1 (its own roll) even when
        //    `dontConsume` seeds the flag true — the C# evaluates the arms
        //    regardless of the argument and only the RESULT is discarded;
        //  - `Random` cycling spends 1 more (`Player.cs:53577`, Phase 4's pick).
        let mut rng = 0xABCD_1234u64;
        let stream_after = |mut rng: u64, n: usize| {
            for _ in 0..n {
                super::next(&mut rng);
            }
            rng
        };
        // The Clockwork spends 0 draws on a first shot AND on a later one: the
        // arm is a pure window compare.
        let mut a = 0xABCD_1234u64;
        let off = VolleyWindows {
            just_started: false,
            ..VolleyWindows::default()
        };
        not_consume(
            434,
            AMMO_ARROW,
            0,
            &off,
            &AmmoFlags::default(),
            false,
            &mut a,
        );
        assert_eq!(a, 0xABCD_1234u64, "the 434 arm draws nothing");
        // Each of the six guns spends exactly one draw, whatever it returns.
        for shooter in [98u16, 533, 2270, 1782, 1929, 1553] {
            let mut b = 0xABCD_1234u64;
            let saved = roll(shooter, &mut b);
            assert_eq!(
                b,
                stream_after(0xABCD_1234u64, 1),
                "shooter {shooter}: one draw, saved {saved}"
            );
        }
        // Each gunProj item spends exactly one draw with dontConsume true.
        for shooter in [3475u16, 3930, 3540] {
            let mut c = 0xABCD_1234u64;
            let _ = not_consume(
                shooter,
                AMMO_ARROW,
                0,
                &windows(),
                &AmmoFlags::default(),
                true,
                &mut c,
            );
            assert_eq!(
                c,
                stream_after(0xABCD_1234u64, 1),
                "gunProj {shooter}: its own roll, dontConsume true"
            );
        }
        // An unmatched shooter with every flag false spends nothing at all.
        let mut d = 0xABCD_1234u64;
        assert!(!roll(39, &mut d), "the wooden bow");
        assert_eq!(d, 0xABCD_1234u64, "no arm applies → no draw");
        // Two flag arms on one shot spend two draws, in the chain's order:
        // ammoBox (draw 1) then ammoCost75 (draw 2) — the second draw's value
        // is the one the cost75 arm's save depends on.
        let mut flags = AmmoFlags::default();
        flags.ammo_box = true;
        flags.ammo_cost75 = true;
        let mut e = 0xABCD_1234u64;
        roll_flags(39, &flags, &mut e);
        assert_eq!(
            e,
            stream_after(0xABCD_1234u64, 2),
            "two arms, two draws, in the chain's order"
        );
        // The interleave check: a Minishark + Ammo Box shot draws ammoBox FIRST
        // (`:53599`) and the Minishark's own roll SECOND (`:53611`). On every
        // seed where the two orders disagree, the chain's result must equal the
        // ammoBox-first reading — a re-sorted chain ("all item arms first")
        // would flip those seeds.
        let mut box_only = AmmoFlags::default();
        box_only.ammo_box = true;
        let mut disagreements = 0;
        for seed in 1u64..400 {
            let box_first = {
                let mut d = seed;
                let a = one_in(5, &mut d);
                let b = one_in(3, &mut d);
                a || b
            };
            let shark_first = {
                let mut d = seed;
                let a = one_in(3, &mut d);
                let b = one_in(5, &mut d);
                a || b
            };
            if box_first == shark_first {
                continue;
            }
            disagreements += 1;
            let mut rng = seed;
            let got = not_consume(98, AMMO_ARROW, 0, &windows(), &box_only, false, &mut rng);
            assert_eq!(got, box_first, "seed {seed}: ammoBox draws first");
        }
        assert!(
            disagreements > 20,
            "the order is observable: {disagreements}"
        );
    }

    #[test]
    fn test_the_magic_quiver_arm_filters_on_the_ammo_kind() {
        // `Player.cs:53595`: `magicQuiver && (IsArrow[useAmmo] || Stake)` — the
        // arm applies to the Arrow (40) and Stake (1836) categories only, never
        // to a bullet or a rocket weapon, no matter the quiver.
        let mut quiver = AmmoFlags::default();
        quiver.magic_quiver = true;
        for ammo_kind in [AMMO_ARROW, AMMO_STAKE] {
            let mut rng = 606u64;
            let saves = (0..300)
                .filter(|_| not_consume(39, ammo_kind, 0, &windows(), &quiver, false, &mut rng))
                .count();
            assert!(
                saves > 30,
                "the quiver's 1/5 applies to the ammo kind {ammo_kind}"
            );
        }
        // A bullet weapon draws nothing: the gate is stream-invisible.
        for ammo_kind in [97i32, 771, 780, 40 + 1] {
            let mut rng = 606u64;
            let before = rng;
            let saves = (0..50)
                .filter(|_| not_consume(39, ammo_kind, 0, &windows(), &quiver, false, &mut rng))
                .count();
            assert_eq!(
                saves, 0,
                "the quiver's arm ignores the ammo kind {ammo_kind}"
            );
            assert_eq!(rng, before, "and spends no draw for it");
        }
        // With the flag false the arm is inert for the arrow kinds too.
        let mut rng = 606u64;
        let before = rng;
        assert!(
            !(0..50).any(|_| roll_flags(39, &AmmoFlags::default(), &mut rng)),
            "no quiver → no save"
        );
        assert_eq!(rng, before, "no quiver → no draw");
    }

    #[test]
    fn test_the_ammo_flags_roll_fields_default_to_false_and_are_independent() {
        // The six appended Phase-3 fields (`AmmoFlags`, the append-only contract)
        // default false, so the chain is the plain vanilla one until
        // M3-EQUIP-3 / M2-BUFF grant them through the Player seam.
        let f = AmmoFlags::default();
        assert!(!f.ammo_box && !f.ammo_potion && !f.huntress_ammo_cost90);
        assert!(!f.chloro_ammo_cost80 && !f.ammo_cost80 && !f.ammo_cost75);
        // The Phase-1 four are still there, still in their frozen order.
        assert!(!f.has_molten_quiver && !f.magic_quiver && !f.acc_sharp_barb && !f.archery);
        // A single flag set leaves the other five inert.
        let mut one = AmmoFlags::default();
        one.ammo_cost75 = true;
        let mut rng = 8128u64;
        let saves = (0..300).filter(|_| roll_flags(39, &one, &mut rng)).count();
        assert!(saves > 40, "1/4 saves: {saves}");
        assert_eq!(one.ammo_box, false, "the other five stay false");
    }

    #[test]
    fn test_the_volley_windows_default_to_all_false() {
        // The three booleans are pure data; `Default` is all-false so an
        // unconfigured chain runs with no volley state (the Phase-3 stand-in
        // `editing.rs` passes the today-equivalent values).
        let w = VolleyWindows::default();
        assert!(
            !w.just_started && !w.anim_gt_use_anim_minus_8 && !w.anim_lt_anim_max_minus_use_time
        );
        // With every window false the two window-keyed arms are inert for a
        // non-window shooter and the 434 arm saves (the "not just started" case
        // the Phase-4 wire will derive).
        let mut rng = 3u64;
        assert!(
            roll_bare(434, &w, &mut rng),
            "default windows: the Clockwork's arm fires (no first shot)"
        );
        let mut rng = 3u64;
        assert!(
            !roll_bare(4953, &w, &mut rng),
            "default windows: the Fairy Queen's arm does not"
        );
    }

    #[test]
    fn test_the_ammo_cycling_defaults_and_its_tick() {
        // `PlayerAmmoCyclingMode` (`PlayerAmmoCyclingMode.cs`) defaults to None
        // and the struct defaults to None/0/0 (`Player.ResetEffects` `:18479`).
        let mut c = AmmoCycling::default();
        assert_eq!(c.mode, AmmoCyclingMode::None);
        assert_eq!((c.offset, c.cooldown), (0, 0));
        // `Player.Update_AmmoCycler` (`Player.cs:28419-28426`): while the
        // cooldown counts down the offset is untouched; the tick that zeroes the
        // cooldown resets the offset. A cooldown of 1 or less resets at once.
        c.cooldown = 3;
        c.offset = 7;
        c.tick();
        assert_eq!((c.cooldown, c.offset), (2, 7), "still counting");
        c.tick();
        assert_eq!((c.cooldown, c.offset), (1, 7), "still counting");
        c.tick();
        assert_eq!(
            (c.cooldown, c.offset),
            (0, 0),
            "the expiry resets the offset"
        );
        // An idle cycler (cooldown 0) is a no-op: the offset stays where it was.
        c.cooldown = 0;
        c.offset = 5;
        c.tick();
        assert_eq!((c.cooldown, c.offset), (0, 5), "no cooldown → no reset");
        // A pending cooldown of 1 resets on the first tick.
        c.cooldown = 1;
        c.offset = 9;
        c.tick();
        assert_eq!((c.cooldown, c.offset), (0, 0));
        // The mode is never touched by the tick.
        c.mode = AmmoCyclingMode::Random;
        c.cooldown = 2;
        c.tick();
        c.tick();
        assert_eq!(c.mode, AmmoCyclingMode::Random, "the tick is offset-only");
    }

    #[test]
    fn test_the_cycling_modes_are_the_four_vanilla_ones() {
        // `GameContent/Items/PlayerAmmoCyclingMode.cs`: None, AmmoSlots,
        // FullInventory, Random — the port's `AmmoCyclingMode` is the same set
        // in the same order, and the grant seam maps the four accessories
        // 6168/6169/6193/6194 onto it (`Player.cs:14528-14544`).
        let modes = [
            AmmoCyclingMode::None,
            AmmoCyclingMode::AmmoSlots,
            AmmoCyclingMode::FullInventory,
            AmmoCyclingMode::Random,
        ];
        assert_eq!(modes[0], AmmoCyclingMode::default(), "None is the default");
        assert_ne!(modes[1], modes[2]);
        assert_ne!(modes[2], modes[3]);
        assert_ne!(modes[3], modes[0]);
    }

    // ---- the projToShoot resolution chain (`Player.cs:53458-53568`) ----

    /// The four fuse flags' Phase-1 fields; the baseline is
    /// `AmmoFlags::default()` — all false, the real owners being
    /// M3-EQUIP-3 / M2-BUFF.
    fn flags(molten: bool, magic_quiver: bool, sharp_barb: bool, archery: bool) -> AmmoFlags {
        AmmoFlags {
            has_molten_quiver: molten,
            magic_quiver,
            acc_sharp_barb: sharp_barb,
            archery,
            ..AmmoFlags::default()
        }
    }

    /// One shot: the weapon's `shoot`/`shootSpeed` seed (`ItemCheck_Shoot`,
    /// `Player.cs:47058-47059`) plus the ammo's own registry row, with the
    /// selected item being the weapon itself (the normal case — the 2888 gate is
    /// the one that calls `resolve_projectile` directly).
    fn shot(
        weapon: u16,
        use_ammo: i32,
        ammo: u16,
        weapon_shoot: i32,
        weapon_shoot_speed: f32,
        flags: &AmmoFlags,
    ) -> Option<AmmoShot> {
        let a = crate::registry::get(ammo).expect("the ammo kind has a registry row");
        super::resolve_projectile(
            weapon,
            ammo,
            use_ammo,
            a.damage,
            a.shoot_speed,
            a.knock_back,
            weapon_shoot,
            weapon_shoot_speed,
            weapon,
            flags,
        )
    }

    #[test]
    fn test_DW_1_1_the_pair_table_transcribes_all_60_ammoid_pairs() {
        // The expected transcription, re-read from `ID/AmmoID.cs:22-128` at build
        // time and written out here independently of the const under test (so a
        // typo on either side fails this test): the five launchers × the twelve
        // rocket kinds, in the C# literal's row order (759, 758, 760, 1946, 3930).
        let expected: [(u16, u16, u16); 60] = [
            // 759 Rocket Launcher (`AmmoID.cs:24-41`).
            (759, 771, 134),
            (759, 772, 137),
            (759, 773, 140),
            (759, 774, 143),
            (759, 4445, 776),
            (759, 4446, 780),
            (759, 4457, 793),
            (759, 4458, 796),
            (759, 4459, 799),
            (759, 4447, 784),
            (759, 4448, 787),
            (759, 4449, 790),
            // 758 Grenade Launcher (`AmmoID.cs:42-59`).
            (758, 771, 133),
            (758, 772, 136),
            (758, 773, 139),
            (758, 774, 142),
            (758, 4445, 777),
            (758, 4446, 781),
            (758, 4457, 794),
            (758, 4458, 797),
            (758, 4459, 800),
            (758, 4447, 785),
            (758, 4448, 788),
            (758, 4449, 791),
            // 760 Proximity Mine Launcher (`AmmoID.cs:60-77`).
            (760, 771, 135),
            (760, 772, 138),
            (760, 773, 141),
            (760, 774, 144),
            (760, 4445, 778),
            (760, 4446, 782),
            (760, 4457, 795),
            (760, 4458, 798),
            (760, 4459, 801),
            (760, 4447, 786),
            (760, 4448, 789),
            (760, 4449, 792),
            // 1946 Snowman Cannon (`AmmoID.cs:78-95`).
            (1946, 771, 338),
            (1946, 772, 339),
            (1946, 773, 340),
            (1946, 774, 341),
            (1946, 4445, 803),
            (1946, 4446, 804),
            (1946, 4457, 808),
            (1946, 4458, 809),
            (1946, 4459, 810),
            (1946, 4447, 805),
            (1946, 4448, 806),
            (1946, 4449, 807),
            // 3930 Celebration Mk2 (`AmmoID.cs:96-113`).
            (3930, 771, 715),
            (3930, 772, 716),
            (3930, 773, 717),
            (3930, 774, 718),
            (3930, 4445, 717),
            (3930, 4446, 718),
            (3930, 4457, 717),
            (3930, 4458, 718),
            (3930, 4459, 717),
            (3930, 4447, 717),
            (3930, 4448, 717),
            (3930, 4449, 717),
        ];
        assert_eq!(expected.len(), 60, "5 launchers × 12 rocket kinds");
        for &(launcher, ammo, proj) in &expected {
            assert_eq!(
                super::specific_launcher_ammo_match(launcher, ammo),
                Some(proj),
                "AmmoID.cs:22-128 — launcher {launcher} + ammo {ammo}"
            );
        }
        // The const carries exactly those 60 cells — no extra, none missing.
        let mut table = super::SPECIFIC_LAUNCHER_AMMO_PROJECTILE_MATCHES;
        let mut want = expected;
        table.sort_unstable();
        want.sort_unstable();
        assert_eq!(table, want, "the const is the 60 transcribed pairs");
    }

    #[test]
    fn test_DW_1_1_the_pair_table_is_injective_except_the_3930_row() {
        use std::collections::BTreeMap;
        let mut rows: BTreeMap<u16, Vec<u16>> = BTreeMap::new();
        let mut keys: Vec<(u16, u16)> = Vec::new();
        for &(launcher, ammo, proj) in &super::SPECIFIC_LAUNCHER_AMMO_PROJECTILE_MATCHES {
            rows.entry(launcher).or_default().push(proj);
            keys.push((launcher, ammo));
        }
        assert_eq!(rows.len(), 5, "the five launchers 758/759/760/1946/3930");
        // The table is a function: no (launcher, ammo) key repeats.
        let keyed = keys.len();
        keys.sort_unstable();
        keys.dedup();
        assert_eq!(keys.len(), keyed, "no repeated (launcher, ammo) key");
        for (launcher, row) in &rows {
            assert_eq!(row.len(), 12, "launcher {launcher}: twelve rocket kinds");
            let mut distinct = row.clone();
            distinct.sort_unstable();
            distinct.dedup();
            if *launcher == 3930 {
                // The 1.4.4 rockets collapse onto the plain Celebration
                // projectiles: 715 ×1, 716 ×1, 717 ×7, 718 ×3
                // (`AmmoID.cs:100-111`).
                assert_eq!(distinct, vec![715, 716, 717, 718], "3930 collapses");
                let count = |p: u16| row.iter().filter(|&&x| x == p).count();
                assert_eq!(count(715), 1);
                assert_eq!(count(716), 1);
                assert_eq!(count(717), 7);
                assert_eq!(count(718), 3);
            } else {
                assert_eq!(distinct.len(), 12, "launcher {launcher} is injective");
            }
        }
        // The four un-collapsed rows are pairwise distinct from each other.
        let mut rest: Vec<u16> = rows
            .iter()
            .filter(|(launcher, _)| **launcher != 3930)
            .flat_map(|(_, row)| row.iter().copied())
            .collect();
        let cells = rest.len();
        rest.sort_unstable();
        rest.dedup();
        assert_eq!(rest.len(), cells, "48 distinct projectiles outside 3930");
    }

    #[test]
    fn test_DW_1_1_a_non_table_launcher_or_ammo_never_matches() {
        // The table keys on the LAUNCHER's kind and the ammo KIND (category), not
        // on the ammo item: a launcher the table does not list (the Electrosphere
        // Launcher 2796) and a non-rocket ammo kind (the Wooden Arrow 40, the
        // Green Solution 780) both miss.
        assert_eq!(super::specific_launcher_ammo_match(2796, 771), None);
        assert_eq!(super::specific_launcher_ammo_match(759, 40), None);
        assert_eq!(super::specific_launcher_ammo_match(759, 780), None);
        assert_eq!(
            super::specific_launcher_ammo_match(96, 1350),
            None,
            "the Musket is not a launcher"
        );
        assert_eq!(super::specific_launcher_ammo_match(3930, 771), Some(715));
    }

    #[test]
    fn test_DW_1_2_the_rocket_launcher_resolves_through_the_pair_table() {
        let f = AmmoFlags::default();
        // 759 Rocket Launcher (`shoot 134`, `shootSpeed 5`) + Rocket I: the
        // table's `759 + 771` cell (`AmmoID.cs:28`). Rocket I's own `shoot` is 0
        // (`Item.cs:10392-10401`), so without the table the chain would leave the
        // launcher's own 134 in place — but for a different reason, so the Rocket
        // II case below is the one that proves the table arm ran.
        let s = shot(759, 771, 771, 134, 5.0, &f).expect("the launcher picks the rocket");
        assert_eq!(s.projectile, 134);
        assert_eq!(s.damage, 40, "the Rocket I's own `Item.damage` 40");
        assert_eq!(s.speed, 5.0, "5.0 + the rocket's `shootSpeed` 0");
        assert_eq!(s.knock_back, 4.0, "the rocket's own `knockBack` 4");
        // Rocket II's own `shoot` is 3 (`Item.cs:10403+`): the table's 137 wins.
        let s = shot(759, 771, 772, 134, 5.0, &f).expect("Rocket II");
        assert_eq!(s.projectile, 137, "the table, never the ammo's own shoot 3");
    }

    #[test]
    fn test_DW_1_2_a_non_table_rocket_launcher_falls_through_to_the_additive_arm() {
        let f = AmmoFlags::default();
        // 2796 Electrosphere Launcher (`use_ammo: Rocket`, `shoot 442`,
        // `shootSpeed 12`) is NOT in the table → the additive arm
        // `projToShoot += item.shoot` (`Player.cs:53483-53486`).
        let s = shot(2796, 771, 771, 442, 12.0, &f).expect("the Electrosphere picks the rocket");
        assert_eq!(s.projectile, 442, "442 + Rocket I's own shoot 0");
        let s = shot(2796, 771, 772, 442, 12.0, &f).expect("Rocket II");
        assert_eq!(
            s.projectile, 445,
            "442 + Rocket II's own shoot 3 — additive, not `=`"
        );
        let s = shot(2796, 771, 4445, 442, 12.0, &f).expect("Cluster Rocket I");
        assert_eq!(s.projectile, 442, "442 + the cluster rocket's own shoot 0");
        // The Solution arm is the Clentaminator's path (`:53487-53490`): 779's own
        // 145 plus the solution's own offset.
        let s = shot(779, 780, 780, 145, 7.0, &f).expect("Green Solution");
        assert_eq!(s.projectile, 145, "145 + the Green Solution's own shoot 0");
        let s = shot(779, 780, 781, 145, 7.0, &f).expect("Blue Solution");
        assert_eq!(s.projectile, 146, "145 + the Blue Solution's own shoot 1");
        assert_eq!(s.damage, 0, "the solution's `damage -1` → no damage add");
    }

    #[test]
    fn test_DW_1_2_the_pair_table_beats_the_additive_arm() {
        // The C# order trap: a table hit EXCLUDES the additive and the
        // `item.shoot > 0` arms (`Player.cs:53471` before `:53483`/`:53491`), so
        // the eight 1.4.4 rocket kinds do not fall back onto the Rocket
        // Launcher's own 134.
        let f = AmmoFlags::default();
        for &(ammo, proj) in &[
            (4445u16, 776u16),
            (4446, 780),
            (4447, 784),
            (4448, 787),
            (4449, 790),
            (4457, 793),
            (4458, 796),
            (4459, 799),
        ] {
            let s = shot(759, 771, ammo, 134, 5.0, &f).expect("the rocket kind");
            assert_eq!(s.projectile, proj, "759 + {ammo}");
        }
    }

    #[test]
    fn test_DW_1_3_hellwing_bow_converts_only_the_wooden_arrow() {
        let f = AmmoFlags::default();
        // 3019 Hellwing Bow (`shoot 485`, `shootSpeed 6`, `use_ammo: Arrow`) +
        // the Wooden Arrow (own `shoot 1`, damage 5) → 485
        // (`Player.cs:53495-53498`).
        let s = shot(3019, 40, 40, 485, 6.0, &f).expect("the bow picks the arrow");
        assert_eq!(s.projectile, 485);
        assert_eq!(s.damage, 5, "`num` untouched by the conversion");
        assert_eq!(s.speed, 9.0, "6.0 + the arrow's `shootSpeed` 3.0");
        assert_eq!(s.knock_back, 2.0);
        // The Flaming Arrow's own `shoot` is 2 (`Item.cs`): `projToShoot != 1`, so
        // the arm does not fire.
        let s = shot(3019, 40, 41, 485, 6.0, &f).expect("Flaming Arrow");
        assert_eq!(s.projectile, 2);
    }

    #[test]
    fn test_DW_1_3_shadow_flame_bow_always_shoots_the_shadowflame_arrow() {
        let f = AmmoFlags::default();
        // 3052 Shadowflame Bow → 495, unconditionally (`Player.cs:53499-53502`).
        let s = shot(3052, 40, 40, 495, 11.0, &f).expect("the bow picks the arrow");
        assert_eq!(s.projectile, 495);
        assert_eq!(s.damage, 5);
        let s = shot(3052, 40, 41, 495, 11.0, &f).expect("Flaming Arrow");
        assert_eq!(s.projectile, 495, "the arm has no `projToShoot == 1` gate");
        assert_eq!(s.damage, 7, "the Flaming Arrow's own damage");
        assert_eq!(s.speed, 14.5, "11.0 + 3.5");
        assert_eq!(s.knock_back, 2.0);
    }

    #[test]
    fn test_DW_1_3_the_fairy_queen_item_converts_only_the_wooden_arrow() {
        let f = AmmoFlags::default();
        // 4953 Fairy Queen's item (`shoot 1`, `use_ammo: Arrow`) → 932 for the
        // wooden-arrow default only (`Player.cs:53503-53506`).
        let s = shot(4953, 40, 40, 1, 10.0, &f).expect("the bow picks the arrow");
        assert_eq!(s.projectile, 932);
        let s = shot(4953, 40, 41, 1, 10.0, &f).expect("Flaming Arrow");
        assert_eq!(s.projectile, 2, "not 1 → no conversion");
    }

    #[test]
    fn test_DW_1_3_blood_rain_bow_always_shoots_the_blood_arrow() {
        let f = AmmoFlags::default();
        // 4381 Blood Rain Bow → 819, unconditionally (`Player.cs:53507-53510`).
        let s = shot(4381, 40, 40, 1, 9.0, &f).expect("the bow picks the arrow");
        assert_eq!(s.projectile, 819);
        assert_eq!(s.damage, 5);
        let s = shot(4381, 40, 41, 1, 9.0, &f).expect("Flaming Arrow");
        assert_eq!(s.projectile, 819, "the arm has no gate");
    }

    #[test]
    fn test_DW_1_3_the_skeleton_bow_converts_the_merchant_bone_arrow() {
        let f = AmmoFlags::default();
        // 4058 Skeleton Bow (`Item.cs:35652-35656`: `shoot 1`, `shootSpeed 11`,
        // `use_ammo: Arrow`) + the Bone Arrow (own `shoot 474`,
        // `ProjectileID.BoneArrowFromMerchant`) → 117 (`Player.cs:53511-53514`).
        let s = shot(4058, 40, 3003, 1, 11.0, &f).expect("the bow picks the bone arrow");
        assert_eq!(s.projectile, 117);
        assert_eq!(s.damage, 8, "the Bone Arrow's own damage");
        assert_eq!(s.speed, 14.5, "11.0 + 3.5");
        assert_eq!(s.knock_back, 2.5);
        // The wooden arrow (`projToShoot == 1`) is not 474 → no conversion.
        let s = shot(4058, 40, 40, 1, 11.0, &f).expect("Wooden Arrow");
        assert_eq!(s.projectile, 1);
    }

    #[test]
    fn test_DW_1_3_the_bees_knees_gate_keys_on_the_selected_item() {
        let f = AmmoFlags::default();
        // 2888 Bees' Knees (`shoot 469`, `shootSpeed 8`, `use_ammo: Arrow`):
        // `inventory[selectedItem].type == 2888 && projToShoot == 1` → 469
        // (`Player.cs:53533-53536`).
        let s = shot(2888, 40, 40, 469, 8.0, &f).expect("the bow picks the arrow");
        assert_eq!(s.projectile, 469);
        assert_eq!(s.damage, 5);
        assert_eq!(s.speed, 11.0, "8.0 + 3.0");
        // The gate reads the SELECTED item's kind, not `sItem`: the same weapon
        // resolved while another item is selected leaves the arrow's own 1.
        let s = super::resolve_projectile(
            2888, 40, 40, 5, 3.0, 2.0, 469, 8.0, 99, // the Iron Bow selected
            &f,
        )
        .expect("the bow picks the arrow");
        assert_eq!(
            s.projectile, 1,
            "the gate keys on `inventory[selectedItem].type`"
        );
    }

    #[test]
    fn test_DW_1_3_the_molten_quiver_converts_the_arrow_and_adds_two_damage() {
        let off = AmmoFlags::default();
        // `hasMoltenQuiver && projToShoot == 1` → 2 (FireArrow) and `num += 2`
        // (`Player.cs:53537-53541`); the accessory is not granted yet, so the
        // default-false flag must stay inert.
        let s = shot(99, 40, 40, 1, 6.6, &off).expect("the bow picks the arrow");
        assert_eq!(s.projectile, 1, "no quiver → the wooden arrow stays");
        assert_eq!(s.damage, 5);
        let on = flags(true, false, false, false);
        let s = shot(99, 40, 40, 1, 6.6, &on).expect("the bow picks the arrow");
        assert_eq!(s.projectile, 2, "the FireArrow");
        assert_eq!(s.damage, 7, "`num` 5 + 2");
        assert_eq!(
            s.speed, 9.6,
            "6.6 + 3.0 — the quiver's fuse arm is `magicQuiver`"
        );
        // A flaming arrow is not the wooden-arrow default → no conversion, no +2.
        let s = shot(99, 40, 41, 1, 6.6, &on).expect("Flaming Arrow");
        assert_eq!(s.projectile, 2);
        assert_eq!(s.damage, 7, "the Flaming Arrow's own damage, no +2");
    }

    #[test]
    fn test_DW_1_3_the_sand_blocks_resolve_their_own_projectile_and_add_five_damage() {
        let f = AmmoFlags::default();
        // The Sandgun's own `shoot` is 42 (`use_ammo: Sand`) and the sand-block
        // kinds take over: 370 Ebonsand → 65, 408 Pearlsand → 68, 1246 Crimsand
        // → 354, each `num += 5` (`Player.cs:53515-53532`).
        for &(ammo, proj) in &[(370u16, 65u16), (408, 68), (1246, 354)] {
            let s = shot(266, 169, ammo, 42, 12.0, &f).expect("the gun picks the sand block");
            assert_eq!(s.projectile, proj, "the Sandgun + ammo {ammo}");
            assert_eq!(s.damage, 4, "`num` −1 + 5");
            assert_eq!(s.speed, 12.0, "12.0 + the block's `shootSpeed` 0");
            assert_eq!(s.knock_back, 0.0);
        }
        // The plain Sand Block (same `ammo` category, `shoot 0`) is not a sand
        // block: `projToShoot` stays the Sandgun's own 42 and `num` stays −1, so
        // the `num > 0` guard adds no damage (`Player.cs:53552-53555`).
        let s = shot(266, 169, 169, 42, 12.0, &f).expect("the gun picks the sand");
        assert_eq!(s.projectile, 42);
        assert_eq!(s.damage, 0, "`num` −1 → the `num > 0` guard skips the add");
    }

    #[test]
    fn test_DW_1_3_the_nano_bullet_multiplies_the_fused_damage() {
        let f = AmmoFlags::default();
        // `projToShoot == 285` (the Nano Bullet's own `shoot`) → ×1.25
        // (`Player.cs:53556-53559`). The resolver owns the ammo side: the bullet's
        // own `Item.damage` 10 → `(int)(10 * 1.25)` = 12.
        let s = shot(96, 97, 1350, 10, 9.0, &f).expect("the gun picks the bullet");
        assert_eq!(s.projectile, 285);
        assert_eq!(s.damage, 12, "the ammo-side ×1.25");
        assert_eq!(s.speed, 13.6, "9.0 + the bullet's `shootSpeed` 4.6");
        assert_eq!(s.knock_back, 3.6);
        // A plain Musket Ball (own `shoot` 14) takes no multiplier.
        let s = shot(96, 97, 97, 10, 9.0, &f).expect("Musket Ball");
        assert_eq!(s.projectile, 14);
        assert_eq!(s.damage, 7, "the ball's own `Item.damage`");
    }

    #[test]
    fn test_DW_1_3_the_magic_quiver_fuses_the_speed_but_not_the_ammos_knockback() {
        let f = AmmoFlags::default();
        // `magicQuiver` on arrow/stake ammo (`Player.cs:53543-53547`): `speed *=
        // 1.1f` AFTER the `speed += item.shootSpeed` fuse, but the sibling
        // `KnockBack *= 1.1f` runs BEFORE `KnockBack += item.knockBack`
        // (`:53568`), so it scales only the weapon's own knockback — the caller's
        // part, never the ammo's.
        let s = shot(99, 40, 40, 1, 6.6, &f).expect("the bow picks the arrow");
        assert_eq!(s.speed, 9.6, "no quiver → 6.6 + 3.0");
        let quiver = flags(false, true, false, false);
        let s = shot(99, 40, 40, 1, 6.6, &quiver).expect("the bow picks the arrow");
        assert_eq!(
            s.speed,
            (6.6f32 + 3.0) * 1.1,
            "the ×1.1 rides on the fused speed"
        );
        assert_eq!(
            s.knock_back, 2.0,
            "the ammo's own `knockBack` is added after the ×1.1"
        );
        // A bullet weapon is not arrow/stake → the arm does not fire.
        let s = shot(96, 97, 97, 10, 9.0, &quiver).expect("the gun picks the ball");
        assert_eq!(s.speed, 13.0, "9.0 + 4.0, no ×1.1");
    }

    #[test]
    fn test_DW_1_3_the_archery_buff_fuses_the_speed_and_caps_it_at_twenty() {
        let f = AmmoFlags::default();
        // `archery` (buff 16) on arrow ammo: `speed *= 1.2f` capped at 20
        // (`Player.cs:53560-53567`).
        let s = shot(99, 40, 40, 1, 6.6, &f).expect("the bow picks the arrow");
        assert_eq!(s.speed, 9.6);
        let archery = flags(false, false, false, true);
        let s = shot(99, 40, 40, 1, 6.6, &archery).expect("the bow picks the arrow");
        assert_eq!(
            s.speed,
            (6.6f32 + 3.0) * 1.2,
            "9.6 → 11.52, far below the cap"
        );
        // The `< 20f` guard: a bow fast enough that the fused speed already
        // reaches 20 takes no arm (a real bow's `shootSpeed` never does, so this
        // is the arm's own boundary, not a vanilla pair).
        let s = shot(99, 40, 40, 1, 17.0, &archery).expect("the bow picks the arrow");
        assert_eq!(s.speed, 20.0, "17.0 + 3.0 = 20 is not `< 20f`");
        // Just under: 19.0 → ×1.2 = 22.8, then the cap.
        let s = shot(99, 40, 40, 1, 16.0, &archery).expect("the bow picks the arrow");
        assert_eq!(s.speed, 20.0, "the ×1.2 caps at 20");
        // A bullet weapon is not arrow ammo → no arm.
        let s = shot(96, 97, 97, 10, 9.0, &archery).expect("the gun picks the ball");
        assert_eq!(s.speed, 13.0, "no ×1.2 on bullets");
    }

    #[test]
    fn test_DW_1_3_the_sharp_barb_adds_one_arrow_damage() {
        let f = AmmoFlags::default();
        // `accSharpBarb` on arrow ammo: `num++` (`Player.cs:53548-53551`).
        let s = shot(99, 40, 40, 1, 6.6, &f).expect("the bow picks the arrow");
        assert_eq!(s.damage, 5);
        let barb = flags(false, false, true, false);
        let s = shot(99, 40, 40, 1, 6.6, &barb).expect("the bow picks the arrow");
        assert_eq!(s.damage, 6, "`num` 5 + 1");
        // A bullet weapon is not arrow ammo → no arm.
        let s = shot(96, 97, 97, 10, 9.0, &barb).expect("the gun picks the ball");
        assert_eq!(s.damage, 7, "the Musket Ball's own damage");
    }

    #[test]
    fn test_DW_1_4b_a_melee_weapon_resolves_to_nothing() {
        // `PickAmmo` only runs `if (sItem.useAmmo > 0)` (`Player.cs:47069`), so a
        // melee weapon (the Wooden Sword, `use_ammo: 0`) never reaches the chain
        // — there is nothing to resolve and no shot to fuse.
        let f = AmmoFlags::default();
        assert_eq!(
            shot(24, 0, 40, 0, 0.0, &f),
            None,
            "the Wooden Sword is melee"
        );
    }

    #[test]
    fn test_DW_1_4b_an_unported_ammo_kind_resolves_to_nothing() {
        // A kind the registry does not carry cannot have picked it (`pick` reads
        // the row's `ammo` field), so refuse the shot instead of silently
        // resolving to the launcher's own projectile.
        let f = AmmoFlags::default();
        assert_eq!(
            super::resolve_projectile(759, 6196, 771, 40, 0.0, 4.0, 134, 5.0, 759, &f),
            None,
            "no registry row → no resolution"
        );
    }

    #[test]
    fn test_the_ammo_flags_default_to_false() {
        // `AmmoFlags` is append-only across phases: Phase 1's four fuse flags all
        // default false, so every flag arm is inert until M3-EQUIP-3 / M2-BUFF
        // grant it.
        let f = AmmoFlags::default();
        assert!(!f.has_molten_quiver);
        assert!(!f.magic_quiver);
        assert!(!f.acc_sharp_barb);
        assert!(!f.archery);
        // With every flag false the chain is the plain vanilla one.
        let s = shot(99, 40, 40, 1, 6.6, &f).expect("the bow picks the arrow");
        assert_eq!(s.projectile, 1, "the wooden arrow's own `shoot`");
        assert_eq!(s.damage, 5);
        assert_eq!(s.speed, 9.6);
        assert_eq!(s.knock_back, 2.0);
    }
}
