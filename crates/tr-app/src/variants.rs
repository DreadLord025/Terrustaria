//! `ItemVariants` (`GameContent/Items/ItemVariants.cs:8-147`) — the
//! world-seed-conditional item variants. Deterministic, no RNG: a variant is
//! selected per item id when its world condition is met, and the in-case
//! stat deltas ride the generated [`crate::itemdata::VARIANT_OVERRIDES`]
//! table (22 `==` deltas + 3 `!=` restores = the 25 `this.Variant` checks,
//! e.g. type 112 Stronger at `Item.cs:2716-2723`, type 544
//! DisabledBossSummon at `Item.cs:7873-7880`).
//!
//! Neither `select` nor `apply` consumes anything world-mutating — the
//! conditions are pure functions of the seed flags (`ItemVariants.cs:112-117`).

use crate::itemdata::{ItemDef, ItemVariant, SeedCondition, VariantOverride, VARIANT_OVERRIDES};

/// The four world flags the C# conditions read (`ItemVariants.cs:112-117`):
/// `Main.remixWorld`, `Main.getGoodWorld`, `SpecialSeedFeatures.Mechdusa`,
/// `Main.skyblockWorld`. Everything defaults to false until worldgen seed
/// parsing lands.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SeedFlags {
    pub remix_world: bool,
    pub get_good_world: bool,
    pub mechdusa: bool,
    pub skyblock_world: bool,
}

impl SeedFlags {
    pub const fn none() -> Self {
        Self {
            remix_world: false,
            get_good_world: false,
            mechdusa: false,
            skyblock_world: false,
        }
    }

    /// M5-SEED-1: the world's own flag set. `Main.remixWorld` also rides
    /// the wind module's state (`tr-core/src/wind.rs:96-97`), which stays
    /// the in-session source for the wind-driven rain; the item variants
    /// now read the generated world's identity
    /// (`World.seeds` — the `WorldGen.Reset()` set) so a seeded world gets
    /// its variants the moment it loads.
    pub fn from_world(world: &tr_core::world::World) -> Self {
        Self {
            remix_world: world.seeds.contains(tr_core::seeds::SeedFlags::REMIX),
            get_good_world: world.seeds.contains(tr_core::seeds::SeedFlags::FOR_THE_WORTHY),
            mechdusa: world.seeds.contains(tr_core::seeds::SeedFlags::EVERYTHING_DEPS),
            skyblock_world: world.seeds.contains(tr_core::seeds::SeedFlags::SKYBLOCK),
        }
    }

    /// The legacy flag source: `Main.remixWorld` rides the wind module's
    /// state (`tr-core/src/wind.rs:96-97`); the other three seeds have no
    /// wind-side source and stay false. Kept for the wind readers.
    pub fn from_wind(wind: &tr_core::wind::Wind) -> Self {
        Self {
            remix_world: wind.remix_world,
            ..Self::none()
        }
    }
}

impl SeedCondition {
    /// `ItemVariantCondition.IsMet` (`ItemVariantCondition.cs:5-15`) — each
    /// condition is exactly one world flag.
    fn is_met(self, flags: &SeedFlags) -> bool {
        match self {
            SeedCondition::RemixWorld => flags.remix_world,
            SeedCondition::GetGoodWorld => flags.get_good_world,
            SeedCondition::Mechdusa => flags.mechdusa,
            SeedCondition::Skyblock => flags.skyblock_world,
        }
    }
}

/// `ItemVariants.SelectVariant` (`ItemVariants.cs:88-102`): the first
/// registration for `id` whose condition is met, else `None`.
pub fn select(id: u16, flags: &SeedFlags) -> Option<ItemVariant> {
    VARIANT_OVERRIDES
        .iter()
        .find(|o| o.id == id && o.condition.is_some_and(|c| c.is_met(flags)))
        .map(|o| o.variant)
}

/// `ItemVariants.HasVariant` (`ItemVariants.cs:85-90`): the id is registered
/// for `variant` — the C# checks registration existence only, with NO
/// condition evaluation; `flags` is unused but kept for seam symmetry.
pub fn has_variant(id: u16, variant: ItemVariant, _flags: &SeedFlags) -> bool {
    VARIANT_OVERRIDES
        .iter()
        .any(|o| o.id == id && o.variant == variant && o.condition.is_some())
}

/// Applies the in-case stat deltas for `selected` to `row` — the generated
/// rows hold absolute values, so `apply` after `registry::get` reproduces the
/// C# `if (this.Variant == X) { … }` / `if (this.Variant != X) { … }` branches
/// exactly (`Item.cs:2716-2723` style). Rows with a `Some` condition are pure
/// registrations (empty `set`) and apply nothing.
pub fn apply(row: &mut ItemDef, id: u16, selected: Option<ItemVariant>) {
    let Some(selected) = selected else { return };
    for o in VARIANT_OVERRIDES.iter().filter(|o| o.id == id) {
        if o.variant == selected && o.condition.is_none() {
            for op in o.set {
                apply_op(row, *op);
            }
        }
    }
}

fn apply_op(row: &mut ItemDef, op: crate::itemdata::VOp) {
    let crate::itemdata::VOp::Set(field, value) = op;
    let int = |v: crate::itemdata::VVal| -> i64 {
        match v {
            crate::itemdata::VVal::Int(i) => i,
            _ => 0,
        }
    };
    let float = |v: crate::itemdata::VVal| -> f32 {
        match v {
            crate::itemdata::VVal::Float(f) => f,
            crate::itemdata::VVal::Int(i) => i as f32,
            _ => 0.0,
        }
    };
    let boolean = |v: crate::itemdata::VVal| -> bool {
        match v {
            crate::itemdata::VVal::Bool(b) => b,
            _ => false,
        }
    };
    match field {
        crate::itemdata::VField::Rare => row.rare = int(value) as i32,
        crate::itemdata::VField::Value => row.value = int(value) as i32,
        crate::itemdata::VField::Damage => row.damage = int(value) as i32,
        crate::itemdata::VField::UseTime => row.use_time = int(value) as u8,
        crate::itemdata::VField::UseAnimation => row.use_animation = int(value) as u8,
        crate::itemdata::VField::Mana => row.mana = int(value) as i32,
        crate::itemdata::VField::Defense => row.defense = int(value) as u8,
        crate::itemdata::VField::UseStyle => row.use_style = int(value) as u8,
        crate::itemdata::VField::Consumable => row.consumable = boolean(value),
        crate::itemdata::VField::Material => row.material = boolean(value),
        crate::itemdata::VField::AutoReuse => row.auto_reuse = boolean(value),
        crate::itemdata::VField::KnockBack => row.knock_back = float(value),
        crate::itemdata::VField::Shoot => row.projectile = (int(value) as u16).try_into().ok(),
        crate::itemdata::VField::ShootSpeed => row.shoot_speed = float(value),
        crate::itemdata::VField::Scale => row.scale = float(value),
        crate::itemdata::VField::UseTurn => row.use_turn = boolean(value),
        crate::itemdata::VField::Width => row.width = int(value) as i32,
        crate::itemdata::VField::Height => row.height = int(value) as i32,
        crate::itemdata::VField::BuffType => row.buff_type = int(value) as i32,
        crate::itemdata::VField::Ammo => row.ammo = int(value) as i32,
        crate::itemdata::VField::MaxStack => row.max_stack = int(value) as u32,
        crate::itemdata::VField::Melee => row.class = crate::registry::DamageClass::Melee,
        crate::itemdata::VField::Ranged => row.class = crate::registry::DamageClass::Ranged,
        crate::itemdata::VField::Magic => row.class = crate::registry::DamageClass::Magic,
        crate::itemdata::VField::Summon => row.class = crate::registry::DamageClass::Summon,
        crate::itemdata::VField::Thrown => row.class = crate::registry::DamageClass::Thrown,
        crate::itemdata::VField::Hammer => row.hammer = int(value) as u16,
        crate::itemdata::VField::Pick => row.pick = int(value) as u16,
        crate::itemdata::VField::Axe => row.axe = int(value) as u16,
        crate::itemdata::VField::Bait => row.bait = int(value) as u16,
        crate::itemdata::VField::Crit => row.crit = int(value) as i32,
        crate::itemdata::VField::UseAmmo => row.use_ammo = int(value) as i32,
        crate::itemdata::VField::NoMelee => {}
        crate::itemdata::VField::NoUseGraphic => {}
        crate::itemdata::VField::Channel => row.channel = boolean(value),
        crate::itemdata::VField::ReuseDelay => row.reuse_delay = int(value) as i32,
        crate::itemdata::VField::BuffTime => row.buff_time = int(value) as i32,
        crate::itemdata::VField::HealLife => row.heal_life = int(value) as i32,
        crate::itemdata::VField::Potion => row.potion = boolean(value),
        crate::itemdata::VField::Accessory => row.accessory = boolean(value),
        crate::itemdata::VField::Vanity => {}
        crate::itemdata::VField::HeadSlot => row.head_slot = int(value) as i16,
        crate::itemdata::VField::BodySlot => row.body_slot = int(value) as i16,
        crate::itemdata::VField::LegSlot => row.leg_slot = int(value) as i16,
        crate::itemdata::VField::PlaceStyle => row.place_style = int(value) as i32,
        // TileBoost is not an ItemDef column (no variant row touches it).
        crate::itemdata::VField::TileBoost => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::registry;

    /// DW-3.1 — all 25 registrations with the right variant + condition
    /// (`ItemVariants.cs:103-147`), and `has_variant` answers per registration.
    #[test]
    fn test_dw_3_1_registrations_match_itemvariants_cs() {
        let expected: &[(u16, ItemVariant, SeedCondition)] = &[
            (112, ItemVariant::Stronger, SeedCondition::RemixWorld),
            (157, ItemVariant::Stronger, SeedCondition::RemixWorld),
            (1319, ItemVariant::Stronger, SeedCondition::RemixWorld),
            (1325, ItemVariant::Stronger, SeedCondition::RemixWorld),
            (3069, ItemVariant::Stronger, SeedCondition::RemixWorld),
            (5147, ItemVariant::Stronger, SeedCondition::RemixWorld),
            (517, ItemVariant::Weaker, SeedCondition::RemixWorld),
            (683, ItemVariant::Weaker, SeedCondition::RemixWorld),
            (725, ItemVariant::Weaker, SeedCondition::RemixWorld),
            (1314, ItemVariant::Weaker, SeedCondition::RemixWorld),
            (2623, ItemVariant::Weaker, SeedCondition::RemixWorld),
            (5279, ItemVariant::Weaker, SeedCondition::RemixWorld),
            (5280, ItemVariant::Weaker, SeedCondition::RemixWorld),
            (5281, ItemVariant::Weaker, SeedCondition::RemixWorld),
            (5282, ItemVariant::Weaker, SeedCondition::RemixWorld),
            (5283, ItemVariant::Weaker, SeedCondition::RemixWorld),
            (5284, ItemVariant::Weaker, SeedCondition::RemixWorld),
            (197, ItemVariant::Rebalanced, SeedCondition::GetGoodWorld),
            (4060, ItemVariant::Rebalanced, SeedCondition::GetGoodWorld),
            (556, ItemVariant::DisabledBossSummon, SeedCondition::Mechdusa),
            (557, ItemVariant::DisabledBossSummon, SeedCondition::Mechdusa),
            (544, ItemVariant::DisabledBossSummon, SeedCondition::Mechdusa),
            (5334, ItemVariant::Enabled, SeedCondition::Mechdusa),
            (5687, ItemVariant::Weaker, SeedCondition::Skyblock),
            (5688, ItemVariant::Weaker, SeedCondition::Skyblock),
        ];
        let flags = SeedFlags {
            remix_world: true,
            get_good_world: true,
            mechdusa: true,
            skyblock_world: true,
        };
        for &(id, variant, cond) in expected {
            assert_eq!(select(id, &flags), Some(variant), "id {id}");
            assert!(has_variant(id, variant, &flags), "id {id}");
        }
        // the registration rows exist exactly once each
        for &(id, variant, _) in expected {
            assert_eq!(
                VARIANT_OVERRIDES
                    .iter()
                    .filter(|o| o.id == id && o.variant == variant && o.condition.is_some())
                    .count(),
                1,
                "id {id}"
            );
        }
    }

    /// DW-3.2 — with every flag false, nothing is selected (the whole table).
    #[test]
    fn test_dw_3_2_all_flags_false_selects_none() {
        let flags = SeedFlags::none();
        for o in VARIANT_OVERRIDES.iter() {
            assert_eq!(select(o.id, &flags), None, "id {}", o.id);
        }
    }

    /// DW-3.3 — `apply` reproduces the C# branches: 112 Stronger
    /// (`Item.cs:2716-2723`) and the 544 restore (`Item.cs:7873-7880`).
    #[test]
    fn test_dw_3_3_apply_matches_item_cs_examples() {
        // 112 Stronger: rare 6, value 500000, damage 85, anim/time 12.
        let mut row = *registry::get(112).unwrap();
        apply(&mut row, 112, Some(ItemVariant::Stronger));
        assert_eq!((row.rare, row.value, row.damage), (6, 500000, 85));
        assert_eq!((row.use_animation, row.use_time), (12, 12));

        // 544 (Mechdusa boss summon): the `!=` branch's use block is present
        // in the base row and removed when the variant is selected
        // (`Item.cs:7873-7880` — useStyle 4/consumable until the restore).
        let base = *registry::get(544).unwrap();
        let mut with_variant = base;
        apply(&mut with_variant, 544, Some(ItemVariant::DisabledBossSummon));
        assert_eq!(with_variant.use_style, 0, "restore zeroes the use block");
        assert!(!with_variant.consumable, "restore unsets consumable");
        assert_eq!((with_variant.use_animation, with_variant.use_time), (100, 100));
        // the base row keeps the use block
        assert_eq!(base.use_style, 4);
        assert!(base.consumable);
    }

    /// DW-3.4 — the module is RNG-free (no rand imports) and the conditions
    /// are pure flag reads.
    #[test]
    fn test_dw_3_4_no_rng() {
        // only the implementation half — the test module's own needle
        // literals must not trip the check
        let impl_half = include_str!("variants.rs")
            .split("#[cfg(test)]")
            .next()
            .unwrap_or("");
        for needle in ["::rand", "thread_rng", "rand::", "SmallRng", "StdRng"] {
            assert!(!impl_half.contains(needle), "variants.rs must not use RNG: {needle}");
        }
        let flags = SeedFlags::none();
        // pure + deterministic: same input, same output
        assert_eq!(select(112, &flags), select(112, &flags));
    }
}
