//! The ammo system (`Player.PickAmmo`, `Player.cs:53458-53683`): the pick
//! order, the projectile resolution chain, the drain and the not-consume
//! rolls.
//!
//! The pick order is `Player.AmmoSlotOrder_Default` (`Player.cs:57141-57144`):
//! the coin slots 50-53, the ammo slots 54-57, then the main grid 0-53 — the
//! port's `equip.coins` → `equip.ammo` → `App.inventory`. The vanilla's
//! `ammoCyclingMode` defaults to `None` (no cycling), so the first matching
//! slot in the order wins.
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

use crate::inv::EquipSlots;
use crate::item::{INV, ItemSlot};

/// The picked slot's location — the drain mutates through it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AmmoSource {
    Coin(usize),
    Ammo(usize),
    Grid(usize),
}

/// `Player.PickAmmo_PickAmmoItem` + `PickAmmo_IterateRange`
/// (`Player.cs:53727-53753`, `:53685-53726`): the first slot in the order
/// whose item's `ammo` field matches the weapon's `useAmmo` and whose stack
/// is positive. `None` = no ammo — the shot is refused (`canShoot` stays
/// false, `Player.cs:53462-53466`).
pub fn pick(inventory: &[ItemSlot; INV], equip: &EquipSlots, use_ammo: i32) -> Option<AmmoSource> {
    if use_ammo <= 0 {
        return None;
    }
    let matches = |slot: &ItemSlot| -> bool {
        slot.count > 0 && crate::registry::get(slot.kind).is_some_and(|d| d.ammo == use_ammo)
    };
    for (i, slot) in equip.coins.iter().enumerate() {
        if matches(slot) {
            return Some(AmmoSource::Coin(i));
        }
    }
    for (i, slot) in equip.ammo.iter().enumerate() {
        if matches(slot) {
            return Some(AmmoSource::Ammo(i));
        }
    }
    for (i, slot) in inventory.iter().enumerate() {
        if matches(slot) {
            return Some(AmmoSource::Grid(i));
        }
    }
    None
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
    /// weapon's own value, see [`resolve_projectile`]).
    pub magic_quiver: bool,
    /// `this.accSharpBarb` — 6160 Sharp Barb (`Player.cs:14501`): +1 ammo damage
    /// on arrow ammo.
    pub acc_sharp_barb: bool,
    /// `this.archery` — buff 16, the Archery Potion (`Player.cs:10018`): ×1.2
    /// speed (capped at 20) for arrow ammo.
    pub archery: bool,
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

/// The not-consume rolls (`Player.cs:53563-53660` + the tail) — the shooters
/// whose items the registry carries, verbatim. `true` = the shot skips the
/// drain. The accessory/buff rolls (`magicQuiver`/`ammoBox`/`ammoPotion`/
/// `chloroAmmoCost80`/`ammoCost80/75`) wait for their systems; the volley
/// specials (the Clockwork's `!ItemAnimationJustStarted`, the Fairy Queen's
/// first-8-ticks) have no port volley state — every shot is the first.
pub fn not_consume(shooter: u16, rng: &mut u64) -> bool {
    match shooter {
        // `Player.cs:53568-53570` — the Minishark: 1/3.
        98 => next(rng) % 3 == 0,
        // `:53596-53598` — the Megashark: 1/2.
        533 => next(rng) % 2 == 0,
        // `:53593-53595` — the Gatligator: 1/2.
        2270 => next(rng) % 2 == 0,
        // `:53599-53601` — the Candy Corn Rifle: 1/3.
        1782 => next(rng) % 3 == 0,
        // `:53602-53604` — the Chain Gun: 2/3 (the code's `Next(3) != 0`).
        1929 => next(rng) % 3 != 0,
        // `:53605-53607` — the SDMG: 2/3.
        1553 => next(rng) % 3 != 0,
        _ => false,
    }
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
        let pick = super::pick(&inv, &equip, 40).expect("ammo exists");
        assert_eq!(pick, AmmoSource::Ammo(0), "the ammo slot precedes the grid");

        // No ammo-slot entry: the grid's first match wins.
        let equip = EquipSlots::empty();
        let pick = super::pick(&inv, &equip, 40).expect("grid ammo");
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
        let pick = super::pick(&coin_inv, &equip, 71).expect("coins");
        assert_eq!(pick, AmmoSource::Coin(1));
    }

    #[test]
    fn the_pick_refuses_the_wrong_ammo_and_the_empty_stacks() {
        // The Musket (96) wants Bullet (97); the arrows (ammo 40) don't match.
        let inv = inv_with(&[(3, 40, 99)]);
        let equip = EquipSlots::empty();
        assert_eq!(super::pick(&inv, &equip, 97), None);
        // An empty stack doesn't match (`item.stack > 0`, `Player.cs:53699`).
        let inv = inv_with(&[(3, 97, 0)]);
        assert_eq!(super::pick(&inv, &equip, 97), None);
        // useAmmo <= 0 refuses outright (the melee weapons never pick).
        assert_eq!(super::pick(&inv, &equip, 0), None);
    }

    #[test]
    fn the_drain_decrements_and_turns_to_air() {
        // `Player.cs:53667-53672`: `stack--`, `TurnToAir` at 0.
        let mut inv = inv_with(&[(3, 97, 2)]);
        let mut equip = EquipSlots::empty();
        let src = pick(&inv, &equip, 97).unwrap();
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
        let src = pick(&inv, &equip, 40).unwrap();
        assert_eq!(src, AmmoSource::Ammo(2));
        assert!(drain(src, &mut inv, &mut equip));
        assert_eq!(equip.ammo[2].count, 4);
    }

    #[test]
    fn the_not_consume_rolls_are_seed_deterministic() {
        // The shooters' rolls (`Player.cs:53568-53607`) — the same seed gives
        // the same save pattern.
        let mut a = 42u64;
        let mut b = 42u64;
        let a_seq: Vec<bool> = (0..30).map(|_| not_consume(98, &mut a)).collect();
        let b_seq: Vec<bool> = (0..30).map(|_| not_consume(98, &mut b)).collect();
        assert_eq!(a_seq, b_seq, "byte-equal patterns");
        assert!(a_seq.contains(&true), "the Minishark saves sometimes");
        // The shooters without rolls never save.
        let mut r = 7u64;
        assert!((0..50).all(|_| !not_consume(39, &mut r)), "the wooden bow");
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
