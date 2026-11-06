//! The town shops — `Chest.SetupShop` (`Chest.cs:1360+`) and the shop
//! pricing (`Player.GetItemExpectedPrice`, `Player.cs:35359-35389`). The
//! vanilla opens a shop through `NPCInteractions.Shop(npcType, shopIndex)`
//! (`NPCInteractions.cs:488-512`) — the `SetupShop` switch keys on the SHOP
//! INDEX, not the NPC type (the Guide (22) offers no shop at all — the
//! GuideTip/GuideReverseCrafting interactions only; the first seller is the
//! Merchant (17) through the shop index 1).

use crate::itemdata::ITEMS;

/// The world state the stock gates read — `SetupShop`'s if-gates collapse
/// into one struct the caller fills (the port's flags exist for all of
/// these; the party/custom-seed gates sit constants-off).
#[derive(Clone, Copy, Debug, Default)]
pub struct ShopFlags {
    pub hard_mode: bool,
    pub night: bool,
    pub blood_moon: bool,
    pub zone_snow: bool,
    pub zone_jungle: bool,
    pub downed_boss2: bool,
    pub downed_boss3: bool,
}

/// One `SetupShop` row: the item id + its gate.
#[derive(Clone, Copy, Debug)]
pub struct ShopEntry {
    pub item: u16,
    pub when: StockWhen,
}

/// The gate kinds (`Chest.cs`'s `if` arms around the `SetDefaults` rows).
#[derive(Clone, Copy, Debug)]
pub enum StockWhen {
    Always,
    HardMode,
    Night,
    BloodMoon,
    DownedBoss3,
    DownedBoss2Or3OrHard,
    ZoneSnow,
    ZoneJungle,
    /// `:1468-1475`'s inventory scan — the item appears when the player
    /// carries `0` (the Flower Boots 930 → the 931/1614 rows).
    PlayerHasItem(u16),
    /// The blood-moon corruption rows (`Chest.cs:1570-1593`'s else branch —
    /// the crimson variants ride the world's crimson flag, constants-off).
    BloodMoonCorruption,
    /// `ZoneUnderworldHeight` (`:1710`) — constants-off until the scene
    /// metrics' underworld flag reaches the shop call.
    ZoneUnderworld,
    /// `ZoneGlowshroom` (`:1713`) — constants-off like the underworld.
    ZoneGlowshroom,
    /// The moon-phase pairs (`:1740+`'s `moonPhase / 2` switches): the row
    /// appears when `moon_phase / 2 == n` (the non-hardmode pairs).
    MoonPhasePair(u8),
}

/// `Chest.SetupShop`'s stock list for one shop index. The port carries the
/// Merchant's shop (index 1, `Chest.cs:1371-1483`); the other indexes land
/// with their NPCs (M4-TOWN umbrella). The order is the vanilla's `num`
/// order (the UI fills the slots left to right).
pub fn setup_shop(shop_index: u32, inventory_ids: &[u16]) -> Vec<ShopEntry> {
    let _ = inventory_ids;
    match shop_index {
        // The Merchant (`Chest.cs:1371-1483`).
        1 => vec![
            row(88, StockWhen::Always),
            row(87, StockWhen::Always),
            row(35, StockWhen::Always),
            row(1991, StockWhen::Always),
            row(3509, StockWhen::Always),
            row(3506, StockWhen::Always),
            row(8, StockWhen::Always),
            // `:1379-1382`'s notTheBees row sits constants-off (the custom
            // seeds).
            row(28, StockWhen::Always),
            row(188, StockWhen::HardMode),
            row(110, StockWhen::Always),
            row(189, StockWhen::HardMode),
            row(40, StockWhen::Always),
            row(42, StockWhen::Always),
            row(965, StockWhen::Always),
            row(967, StockWhen::ZoneSnow),
            row(33, StockWhen::ZoneJungle),
            // `:1428-1430`'s happy-windy-day row rides the weather pass.
            row(279, StockWhen::BloodMoon),
            row(282, StockWhen::Night),
            // `:1435-1437`'s party row sits constants-off (the BirthdayParty
            // system lands with the party event).
            row(346, StockWhen::DownedBoss3),
            row(488, StockWhen::HardMode),
            row(1786, StockWhen::Always),
            row(1348, StockWhen::HardMode),
            row(3198, StockWhen::HardMode),
            row(4063, StockWhen::DownedBoss2Or3OrHard),
            row(4673, StockWhen::DownedBoss2Or3OrHard),
            row(931, StockWhen::PlayerHasItem(930)),
            row(1614, StockWhen::PlayerHasItem(930)),
            // `:1480-1482`'s HasItem(3107) → 3108 row lands with the
            // inventory-scan pass.
        ],
        // The Arms Dealer (`Chest.cs:1484-1567`): the Musket Balls 97, the
        // Flintlock 95?/98 rows always; the PlayerHasItem rows; the
        // silver-tier flip (4915/278), the downedBoss2/night Minishark-ish
        // 47, the night 324, the hardmode 534/1432/2177 and the graveyard
        // 4703 sit constants-off.
        2 => vec![
            row(97, StockWhen::Always),
            row(95, StockWhen::Always),
            row(98, StockWhen::Always),
            row(1261, StockWhen::PlayerHasItem(1258)),
            row(1836, StockWhen::PlayerHasItem(1835)),
            row(3108, StockWhen::PlayerHasItem(3107)),
            row(1783, StockWhen::PlayerHasItem(1782)),
            row(1785, StockWhen::PlayerHasItem(1784)),
        ],
        // The Painter (`Chest.cs:2443-2483`): the paint rollers and the
        // paintings (1071/1072/1100/1097/1099/1098/1966); the moon-phase
        // painting row (num6) rides the moon-pair pass.
        15 => vec![
            row(1071, StockWhen::Always),
            row(1072, StockWhen::Always),
            row(1100, StockWhen::Always),
            row(1097, StockWhen::Always),
            row(1099, StockWhen::Always),
            row(1098, StockWhen::Always),
            row(1966, StockWhen::Always),
        ],
        // The Demolitionist (`Chest.cs:1746-1814`): the Grenade 168/the
        // Bomb 166/the Dynamite 167 always; the Land Mine 5542 rides the
        // downedBoss1/slimeKing night gate (constants-off); the hardmode
        // 265/5481/5464/1347 and the holy-arrow 937 rows constants-off.
        4 => vec![
            row(168, StockWhen::Always),
            row(166, StockWhen::Always),
            row(167, StockWhen::Always),
        ],
        // The Witch Doctor (`Chest.cs:2615-2672`): the Blowgun 1430 and the
        // Flask 986 always; the conditionals (the Queen Bee 2999, the
        // jungle 6147/1167, the night 1158, the hardmode 1159, the
        // Imbuing Station 1160?/1161, the 1339/1171/1162 rows) sit
        // constants-off until their flags land.
        16 => vec![
            row(1430, StockWhen::Always),
            row(986, StockWhen::Always),
        ],
        // The Clothier (`Chest.cs:1814-1989`): the base 254/981; the day
        // 242; the moon-phase hats; the full set in the tail — the v1
        // carries the always-on rows (the moon-phase and the clown rows
        // land with their flags).
        5 => vec![
            row(254, StockWhen::Always),
            row(981, StockWhen::Always),
            row(269, StockWhen::Always),
            row(270, StockWhen::Always),
            row(271, StockWhen::Always),
        ],
        // The Dryad (`Chest.cs:1568-1745`): the blood-moon corruption seed
        // branch, the base rows, the flower-seed moon-phase pairs. The
        // crimson variants (2886/2171/4508), the graveyard/underworld/
        // glowshroom rows and the halloween rows sit constants-off.
        3 => vec![
            row(67, StockWhen::BloodMoonCorruption),
            row(59, StockWhen::BloodMoonCorruption),
            row(4504, StockWhen::BloodMoonCorruption),
            row(66, StockWhen::Always),
            row(62, StockWhen::Always),
            row(63, StockWhen::Always),
            row(745, StockWhen::Always),
            row(27, StockWhen::Always),
            row(5309, StockWhen::Always),
            row(114, StockWhen::Always),
            row(1828, StockWhen::Always),
            row(747, StockWhen::Always),
            row(746, StockWhen::HardMode),
            row(369, StockWhen::HardMode),
            row(4505, StockWhen::HardMode),
            row(5214, StockWhen::ZoneUnderworld),
            row(194, StockWhen::ZoneGlowshroom),
            row(3215, StockWhen::Always),
            row(3216, StockWhen::Always),
            row(3219, StockWhen::Always),
            // `:1656-1661`'s 3217/3218 crimson split rides the world flag.
            row(3217, StockWhen::Always),
            row(3220, StockWhen::Always),
            row(3221, StockWhen::Always),
            row(3222, StockWhen::Always),
            row(4047, StockWhen::Always),
            row(4045, StockWhen::Always),
            row(4044, StockWhen::Always),
            row(4043, StockWhen::Always),
            row(4042, StockWhen::Always),
            row(4046, StockWhen::Always),
            row(4041, StockWhen::Always),
            row(4241, StockWhen::Always),
            row(4048, StockWhen::Always),
            row(4430, StockWhen::MoonPhasePair(0)),
            row(4431, StockWhen::MoonPhasePair(0)),
            row(4433, StockWhen::MoonPhasePair(1)),
            row(4434, StockWhen::MoonPhasePair(1)),
            row(4436, StockWhen::MoonPhasePair(2)),
            row(4437, StockWhen::MoonPhasePair(2)),
            row(4439, StockWhen::MoonPhasePair(3)),
            row(4440, StockWhen::MoonPhasePair(3)),
        ],
        _ => Vec::new(),
    }
}

fn row(item: u16, when: StockWhen) -> ShopEntry {
    ShopEntry { item, when }
}

/// The resolved stock — the gated rows filtered by the world state. The
/// `PlayerHasItem` rows read `inventory_ids` (the player inventory's item
/// ids, `:1468-1475`'s 58-slot scan).
pub fn resolve_stock(
    shop_index: u32,
    flags: &ShopFlags,
    inventory_ids: &[u16],
) -> Vec<u16> {
    resolve_stock_moon(shop_index, flags, inventory_ids, 0)
}

/// The moon-phase-aware twin — `Main.moonPhase` rides the world clock.
pub fn resolve_stock_moon(
    shop_index: u32,
    flags: &ShopFlags,
    inventory_ids: &[u16],
    moon_phase: u32,
) -> Vec<u16> {
    setup_shop(shop_index, inventory_ids)
        .into_iter()
        .filter(|e| match e.when {
            StockWhen::Always => true,
            StockWhen::HardMode => flags.hard_mode,
            StockWhen::Night => flags.night,
            StockWhen::BloodMoon => flags.blood_moon,
            StockWhen::DownedBoss3 => flags.downed_boss3,
            StockWhen::DownedBoss2Or3OrHard => {
                flags.downed_boss2 || flags.downed_boss3 || flags.hard_mode
            }
            StockWhen::ZoneSnow => flags.zone_snow,
            StockWhen::ZoneJungle => flags.zone_jungle,
            StockWhen::PlayerHasItem(id) => inventory_ids.contains(&id),
            StockWhen::BloodMoonCorruption => flags.blood_moon,
            StockWhen::ZoneUnderworld | StockWhen::ZoneGlowshroom => false,
            StockWhen::MoonPhasePair(n) => moon_phase / 2 == n as u32,
        })
        .map(|e| e.item)
        .collect()
}

/// `Player.GetItemExpectedPrice`'s buying half (`Player.cs:35387`): the
/// shop item's price = the store value × the happiness adjustment, rounded
/// as the C# `Math.Round` does (the banker's rounding — the port matches
/// the damage.rs convention). The discount's 0.8 (`:35369-35374`) rides the
/// discount work.
pub fn buy_price(item: u16, adjustment: f32) -> i32 {
    let store = ITEMS[item as usize].value as f32;
    round_half_to_even(store * adjustment)
}

/// The selling half (`:35388`): the store value ÷ the adjustment — the
/// player's own items price at `value / 5` first (the vanilla's 20% sale
/// rate lives in the shop-sell path; the port keeps the rate here).
pub fn sell_price(item: u16, adjustment: f32) -> i32 {
    let store = ITEMS[item as usize].value as f32 / 5.0;
    round_half_to_even(store / adjustment)
}

/// The C# `Math.Round`'s default = the banker's rounding (the damage.rs
/// convention).
fn round_half_to_even(v: f32) -> i32 {
    let r = v.round();
    if (v - v.trunc()).abs() == 0.5 && r % 2.0 != 0.0 {
        let down = (v - 0.5).round();
        return down as i32;
    }
    r as i32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_merchant_stock_carries_the_base_rows() {
        // `Chest.cs:1371-1483`: the day/easy-mode base stock (88/87/35/
        // 1991/3509/3506/8/28/110/40/42/965/1786) + the gated rows.
        let stock = resolve_stock(1, &ShopFlags::default(), &[]);
        for id in [88, 87, 35, 1991, 3509, 3506, 8, 28, 110, 40, 42, 965, 1786] {
            assert!(stock.contains(&id), "the base row {id} missing");
        }
        // The gated rows stay out on the default (day, pre-hardmode) flags.
        assert!(!stock.contains(&188), "the hardmode row leaked");
        assert!(!stock.contains(&967), "the snow row leaked");
        assert!(!stock.contains(&33), "the jungle row leaked");
        assert!(!stock.contains(&279), "the blood-moon row leaked");
        assert!(!stock.contains(&282), "the night row leaked");
        assert!(!stock.contains(&346), "the skeletron row leaked");
    }

    #[test]
    fn the_stock_gates_flip_with_the_world_state() {
        let flags = ShopFlags {
            hard_mode: true,
            night: true,
            blood_moon: true,
            zone_snow: true,
            zone_jungle: true,
            downed_boss2: true,
            downed_boss3: true,
        };
        let stock = resolve_stock(1, &flags, &[930]);
        for id in [188, 189, 488, 1348, 3198, 967, 33, 279, 282, 346, 4063, 4673, 931, 1614] {
            assert!(stock.contains(&id), "the gated row {id} missing");
        }
        // The PlayerHasItem rows need the scan hit.
        let without = resolve_stock(1, &flags, &[]);
        assert!(!without.contains(&931));
    }

    #[test]
    fn the_unknown_shop_index_is_empty() {
        // The port carries the Merchant's shop only; the other indexes
        // land with their NPCs.
        assert!(resolve_stock(22, &ShopFlags::default(), &[]).is_empty());
    }

    #[test]
    fn the_prices_ride_the_happiness_adjustment() {
        // `Player.cs:35387`: the buy = the store value × the adjustment;
        // the sell (`:35388` + the 20% rate) = value / 5 ÷ the adjustment.
        // The Mining Helmet 88: value 40000.
        assert_eq!(buy_price(88, 1.0), 40000);
        assert_eq!(buy_price(88, 0.75), 30000);
        assert_eq!(buy_price(88, 1.5), 60000);
        assert_eq!(sell_price(88, 1.0), 8000);
        assert_eq!(sell_price(88, 0.75), 10667);
        // The Guide-in-the-forest multiplier from the happiness tests.
        assert_eq!(buy_price(88, 0.89), 35600);
    }
}

#[cfg(test)]
mod town5_tests {
    use super::*;

    #[test]
    fn the_buy_price_matches_the_happiness_multiplier() {
        // The round trip: the Mining Helmet's 40000 base rides the mood.
        assert_eq!(buy_price(88, 1.5), 60000);
        assert_eq!(buy_price(88, 0.89), 35600);
    }

    #[test]
    fn the_stock_conditions_cover_the_merchant_gates() {
        // The night row (282 = the Glowstick) opens only at night; the
        // hardmode rows stay constants-off in the default flags.
        let flags = ShopFlags {
            night: true,
            ..ShopFlags::default()
        };
        let stock = resolve_stock(1, &flags, &[]);
        assert!(stock.contains(&282));
        assert!(!stock.contains(&188));
    }
    #[test]
    fn the_dryad_stock_rides_the_moon_and_the_blood_moon() {
        // `Chest.cs:1568-1745`: the base rows always; the corruption seeds
        // only under the blood moon; the flower-seed pairs by `moonPhase/2`.
        let base = resolve_stock_moon(3, &ShopFlags::default(), &[], 0);
        for id in [66, 62, 63, 745, 27, 5309, 114, 1828, 747, 3215, 4047] {
            assert!(base.contains(&id), "the base row {id} missing");
        }
        assert!(!base.contains(&67), "the blood-moon row leaked");
        assert!(base.contains(&4430) && base.contains(&4431), "the pair 0 rows");
        assert!(!base.contains(&4433), "the pair 1 rows leaked at phase 0");
        // The phase 4 (the pair 2): 4436/4437.
        let p4 = resolve_stock_moon(3, &ShopFlags::default(), &[], 4);
        assert!(p4.contains(&4436) && !p4.contains(&4430), "the pair 2 rows");
        // The blood moon: the corruption seeds.
        let bm = resolve_stock_moon(
            3,
            &ShopFlags {
                blood_moon: true,
                ..ShopFlags::default()
            },
            &[],
            0,
        );
        for id in [67, 59, 4504] {
            assert!(bm.contains(&id), "the blood-moon row {id} missing");
        }
        // The hardmode rows stay out (the constants-off gates).
        assert!(!base.contains(&746));
        // The unknown index stays empty.
        assert!(resolve_stock_moon(9, &ShopFlags::default(), &[], 0).is_empty());
    }

}

