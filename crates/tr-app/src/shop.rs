//! The town shops — `Chest.SetupShop` (`Chest.cs:1360+`) and the shop
//! pricing (`Player.GetItemExpectedPrice`, `Player.cs:35359-35389`). The
//! vanilla opens a shop through `NPCInteractions.Shop(npcType, shopIndex)`
//! (`NPCInteractions.cs:488-512`) — the `SetupShop` switch keys on the SHOP
//! INDEX, not the NPC type (the Guide (22) offers no shop at all — the
//! GuideTip/GuideReverseCrafting interactions only; the first seller is the
//! Merchant (17) through the shop index 1).

use crate::itemdata::ITEMS;
use tr_core::World;

/// The world state the stock gates read — `SetupShop`'s if-gates collapse
/// into one struct the caller fills (the port's flags exist for all of
/// these; the party/custom-seed gates sit constants-off).
#[derive(Clone, Copy, Debug)]
pub struct ShopFlags {
    pub hard_mode: bool,
    pub night: bool,
    pub blood_moon: bool,
    pub zone_snow: bool,
    pub zone_jungle: bool,
    pub downed_boss2: bool,
    pub downed_boss3: bool,
    /// The Stylist shop's gates (`Chest.cs:2704-2783`): the player's life
    /// and mana maxima, the copper purse, the Party Girl's presence, the
    /// mech bosses and the martian invasion.
    pub life_max_400: bool,
    pub mana_max_200: bool,
    pub coins_over_gold: bool,
    pub party_girl_present: bool,
    pub downed_mech_any: bool,
    pub downed_mech_1: bool,
    pub downed_mech_2: bool,
    pub downed_mech_3: bool,
    pub downed_martians: bool,
    /// `Main.LocalPlayer.golferScoreAccumulated` (`Chest.cs:3158`): the
    /// Golfer shop's score gates — the port has no golf system yet, so it
    /// reads 0 until one lands (the gated rows stay hidden).
    pub golfer_score: i32,
    /// The Zoologist's shop gates (`Chest.cs:3265`): the bestiary unlock
    /// report's completion percent.
    pub bestiary_completion: f32,
    /// `NPC.unlockedPrincessSpawn` (`Main.cs:65948`): the debug override
    /// that opens the Princess's arrival without the full roster.
    pub unlocked_princess: bool,
    /// `Main.travelShop` (`Main.cs:1779`) — the Travelling Merchant's
    /// per-visit stock (`Chest.SetupTravelShop`), captured at the open
    /// like the vanilla's `SetupShop(19)` reads it live.
    pub travel_stock: [u16; 40],
}

impl Default for ShopFlags {
    fn default() -> Self {
        Self {
            hard_mode: false,
            night: false,
            blood_moon: false,
            zone_snow: false,
            zone_jungle: false,
            downed_boss2: false,
            downed_boss3: false,
            life_max_400: false,
            mana_max_200: false,
            coins_over_gold: false,
            party_girl_present: false,
            downed_mech_any: false,
            downed_mech_1: false,
            downed_mech_2: false,
            downed_mech_3: false,
            downed_martians: false,
            golfer_score: 0,
            bestiary_completion: 0.0,
            unlocked_princess: false,
            travel_stock: [0; 40],
        }
    }
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
    /// The ODD moon phases only (`Chest.cs:2085`'s
    /// `moonPhase == 1 || 3 || 5 || 7`): the Mechanic's Rod row.
    MoonPhaseOdd,
    /// `statLifeMax >= 400` (`:2704`) — the Life Hair Dye row.
    LifeMax400,
    /// `statManaMax >= 200` (`:2709`) — the Mana Hair Dye row.
    ManaMax200,
    /// The purse's copper total ≥ 1,000,000 (`:2743`) — the Money Hair
    /// Dye row.
    CoinsOverGold,
    /// The moon-phase/day parity pair (`:2747`): even phase + day OR odd
    /// phase + night — the Time Hair Dye row.
    MoonPhaseDayParity,
    /// `NPC.AnyNPCs(208)` (`:2762`) — the Party Hair Dye row.
    PartyGirlPresent,
    /// `Main.hardMode && downedMechBoss1/2/3` (`:2767`).
    DownedMechAll,
    /// `Main.hardMode && NPC.downedMechBossAny` (`:2772`).
    DownedMechAny,
    /// `Main.hardMode && NPC.downedMartians` (`:2777`) — the two rows.
    DownedMartians,
    /// `Main.moonPhase == 0 && !Main.dayTime` (`Chest.cs:3271`).
    MoonPhaseNight0,
    /// The Zoologist's shop gates (`Chest.cs:3275-3310`): the bestiary
    /// completion tiers (the C# compares floats directly).
    BestiaryCompletion(f32),
    /// `Main.hardMode && NPC.downedMoonlord` (`Chest.cs:3432`).
    HardModeMoonlord,
    /// `NPC.downedSlimeKing && NPC.downedQueenSlime` (`:3428`).
    DownedBothSlimes,
    /// The Golfer shop's painting pair (`Chest.cs:3238-3260`): the score
    /// `>= 2000` gate AND the moon-phase pair, exactly one of four.
    GolfPainting(u8),
    /// The Golfer shop's score tier — the vanilla mixes `>=` and `>` per
    /// row (`Chest.cs:3169`/`:3186`/`:3202`/`:3207`/`:3223`/`:3228`/
    /// `:3233`), so each row names its own threshold + operator.
    GolfScore(i32, bool),
}

/// `Chest.SetupShop`'s stock list for one shop index. The port carries the
/// Merchant's shop (index 1, `Chest.cs:1371-1483`); the other indexes land
/// with their NPCs (M4-TOWN umbrella). The order is the vanilla's `num`
/// order (the UI fills the slots left to right).
pub fn setup_shop(
    shop_index: u32,
    inventory_ids: &[u16],
    flags: &ShopFlags,
) -> Vec<ShopEntry> {
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
        // The Goblin Tinkerer (`Chest.cs:1989-2003`): the 7 rows, no
        // gating — the Rocket Boots 128, the Ruler 486, the Tinkerer's
        // Workshop 398, the Grappling Hook 84, the Toolbelt 407, the
        // Spiky Balls 161, the Rubblemaker (small) 5324.
        6 => vec![
            row(128, StockWhen::Always),
            row(486, StockWhen::Always),
            row(398, StockWhen::Always),
            row(84, StockWhen::Always),
            row(407, StockWhen::Always),
            row(161, StockWhen::Always),
            row(5324, StockWhen::Always),
        ],
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
        16 => vec![row(1430, StockWhen::Always), row(986, StockWhen::Always)],
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
        // The Stylist's shop (index 18, `Chest.cs:2698-2790`): the
        // hair-dye family — the two always-on rows, the life/mana/money
        // gates, the moon-phase parity, the hardmode + Party Girl +
        // mech/martian rows; the multiplayer team dye (1982) and the
        // graveyard mirror (5577) sit constants-off.
        18 => vec![
            row(1990, StockWhen::Always),
            row(1979, StockWhen::Always),
            row(1977, StockWhen::LifeMax400),
            row(1978, StockWhen::ManaMax200),
            row(1980, StockWhen::CoinsOverGold),
            row(1981, StockWhen::MoonPhaseDayParity),
            row(1983, StockWhen::HardMode),
            row(1984, StockWhen::PartyGirlPresent),
            row(1985, StockWhen::DownedMechAll),
            row(1986, StockWhen::DownedMechAny),
            row(2863, StockWhen::DownedMartians),
            row(3259, StockWhen::DownedMartians),
            row(5104, StockWhen::Always),
        ],
        // The Wizard's shop (index 7, `Chest.cs:2004-2032`): the nine
        // magic rows always, the hardmode blood-moon deck (5461) and the
        // halloween hat (1739) constants-off (the port has neither
        // season).
        7 => vec![
            row(487, StockWhen::Always),
            row(496, StockWhen::Always),
            row(500, StockWhen::Always),
            row(507, StockWhen::Always),
            row(508, StockWhen::Always),
            row(531, StockWhen::Always),
            row(149, StockWhen::Always),
            row(576, StockWhen::Always),
            row(3186, StockWhen::Always),
        ],
        // The Mechanic's shop (index 8, `Chest.cs:2033-2090`): the wire
        // family's rows; the graveyard goggles (4409) constants-off and
        // the Mechanic's Rod (2295) rides the Angler-present moon-phase
        // pair (1/3/5/7).
        8 => vec![
            row(509, StockWhen::Always),
            row(850, StockWhen::Always),
            row(851, StockWhen::Always),
            row(3612, StockWhen::Always),
            row(510, StockWhen::Always),
            row(530, StockWhen::Always),
            row(513, StockWhen::Always),
            row(538, StockWhen::Always),
            row(529, StockWhen::Always),
            row(541, StockWhen::Always),
            row(542, StockWhen::Always),
            row(543, StockWhen::Always),
            row(852, StockWhen::Always),
            row(853, StockWhen::Always),
            row(4261, StockWhen::Always),
            row(3707, StockWhen::Always),
            row(2739, StockWhen::Always),
            row(849, StockWhen::Always),
            row(1263, StockWhen::Always),
            row(3616, StockWhen::Always),
            row(3725, StockWhen::Always),
            row(2799, StockWhen::Always),
            row(3619, StockWhen::Always),
            row(3627, StockWhen::Always),
            row(3629, StockWhen::Always),
            row(585, StockWhen::Always),
            row(584, StockWhen::Always),
            row(583, StockWhen::Always),
            row(4484, StockWhen::Always),
            row(4485, StockWhen::Always),
            // `:2085`'s `NPC.AnyNPCs(369) && moonPhase % 2 == 1` — the
            // port's town count reads the Angler's presence.
            row(2295, StockWhen::MoonPhaseOdd),
        ],
        // The Zoologist's shop (index 23, `Chest.cs:3263-3340`): the
        // bestiary tools — the progress-gated rows ride
        /// `BestiaryUnlockProgressReport.CompletionPercent`
        /// (`:3265`), the lumen/solar rows the flags they name.
        23 => vec![
            row(4767, StockWhen::Always),
            row(5253, StockWhen::MoonPhaseNight0),
            row(5635, StockWhen::BestiaryCompletion(0.45)),
            row(4759, StockWhen::BestiaryCompletion(0.10)),
            row(4672, StockWhen::BestiaryCompletion(0.03)),
            row(4829, StockWhen::Always),
            row(4830, StockWhen::BestiaryCompletion(0.25)),
            row(4910, StockWhen::BestiaryCompletion(0.45)),
            row(4871, StockWhen::BestiaryCompletion(0.30)),
            row(4907, StockWhen::BestiaryCompletion(0.30)),
            row(4676, StockWhen::BestiaryCompletion(0.10)),
        ],
        // The Princess's shop (index 24, `Chest.cs:3409-3471`): the
        /// 17 vanity dresses + the two slime-kings row + the Moon Lord's
        /// hat (the tenth-anniversary rows sit constants-off).
        24 => vec![
            row(5071, StockWhen::Always),
            row(5072, StockWhen::Always),
            row(5073, StockWhen::Always),
            row(5076, StockWhen::Always),
            row(5077, StockWhen::Always),
            row(5078, StockWhen::Always),
            row(5079, StockWhen::Always),
            row(5080, StockWhen::Always),
            row(5081, StockWhen::Always),
            row(5082, StockWhen::Always),
            row(5083, StockWhen::Always),
            row(5084, StockWhen::Always),
            row(5085, StockWhen::Always),
            row(5086, StockWhen::Always),
            row(5087, StockWhen::Always),
            row(5310, StockWhen::Always),
            row(5222, StockWhen::Always),
            row(5228, StockWhen::Always),
            row(5266, StockWhen::DownedBothSlimes),
            row(5044, StockWhen::HardModeMoonlord),
            row(5088, StockWhen::Always),
        ],
        // The Golfer's shop (index 22, `Chest.cs:3156-3262`): the golf
        // gear — the clubs, the cup flags, the tees/balls, the signs, the
        // clothes and the mower are always on; every better club, the
        // chest, the trophies, the cart and the paintings ride
        /// `golferScoreAccumulated`'s tiers (and the cart's Skeletron
        /// gate) — the port carries the score on the ShopFlags so the
        /// rows stay faithful once the golf system lands.
        22 => vec![
            row(4587, StockWhen::Always),
            row(4590, StockWhen::Always),
            row(4589, StockWhen::Always),
            row(4588, StockWhen::Always),
            row(4083, StockWhen::Always),
            row(4084, StockWhen::Always),
            row(4085, StockWhen::Always),
            row(4086, StockWhen::Always),
            row(4087, StockWhen::Always),
            row(4088, StockWhen::Always),
            row(4039, StockWhen::GolfScore(500, true)),
            row(4094, StockWhen::GolfScore(500, true)),
            row(4093, StockWhen::GolfScore(500, true)),
            row(4092, StockWhen::GolfScore(500, true)),
            row(4089, StockWhen::Always),
            row(3989, StockWhen::Always),
            row(4095, StockWhen::Always),
            row(4040, StockWhen::Always),
            row(4319, StockWhen::Always),
            row(4320, StockWhen::Always),
            row(4591, StockWhen::GolfScore(1000, false)),
            row(4594, StockWhen::GolfScore(1000, false)),
            row(4593, StockWhen::GolfScore(1000, false)),
            row(4592, StockWhen::GolfScore(1000, false)),
            row(4135, StockWhen::Always),
            row(4138, StockWhen::Always),
            row(4136, StockWhen::Always),
            row(4137, StockWhen::Always),
            row(4049, StockWhen::Always),
            row(4265, StockWhen::GolfScore(500, false)),
            row(4595, StockWhen::GolfScore(2000, false)),
            row(4598, StockWhen::GolfScore(2000, false)),
            row(4597, StockWhen::GolfScore(2000, false)),
            row(4596, StockWhen::GolfScore(2000, false)),
            row(4264, StockWhen::DownedBoss3),
            row(4599, StockWhen::GolfScore(500, false)),
            row(4600, StockWhen::GolfScore(1000, true)),
            row(4601, StockWhen::GolfScore(2000, true)),
            row(4658, StockWhen::GolfPainting(0)),
            row(4659, StockWhen::GolfPainting(1)),
            row(4660, StockWhen::GolfPainting(2)),
            row(4661, StockWhen::GolfPainting(3)),
        ],
        // The Travelling Merchant (`Chest.cs:2791-2810`): the stock is
        // NOT a fixed row list — it rides the 40 `Main.travelShop` slots
        // the visit roll generated (`Chest.SetupTravelShop`).
        19 => {
            let mut rows = Vec::new();
            // `:2794-2800`'s Palworld-collab rows: the items that gate the
            // Pal Metal armor's appearance — the Pal summons/orbs the
            // player may carry (5663-5667, 6148-6151, 6174).
            if TRAVEL_ARMOR_GATES
                .iter()
                .any(|id| inventory_ids.contains(id))
            {
                rows.push(row(5735, StockWhen::Always));
                rows.push(row(5736, StockWhen::Always));
            }
            // `:2801-2808`'s walk over the merchant's slots — a `0` slot
            // is an empty one (`Main.travelShop`'s zero fill).
            for id in flags.travel_stock.iter().copied() {
                if id != 0 {
                    rows.push(row(id, StockWhen::Always));
                }
            }
            rows
        }
        _ => Vec::new(),
    }
}

/// `Chest.cs:2794`'s gate list — the Palworld-collab items that put the
/// Pal Metal armor (5735/5736) into the merchant's stock.
const TRAVEL_ARMOR_GATES: [u16; 10] = [5667, 5663, 5664, 5665, 5666, 6174, 6148, 6149, 6150, 6151];

/// `Main.rand.Next(min, max)` over the port's LCG.
fn roll_int(rng: &mut u64, min: i32, max: i32) -> i32 {
    min + (crate::app::rand01(rng) * (max - min) as f32) as i32
}

/// `Player.RollLuck` -> `Luck.RollLuck` (`Player.cs:18188-18191`,
/// `Luck.cs:5-16`): a hit is the returned roll landing on 0, so the base
/// chance is `1 / range` with the luck narrowing/widening the range.
fn roll_luck(luck: f32, range: i32, rng: &mut u64) -> i32 {
    if luck > 0.0 && crate::app::rand01(rng) < luck {
        return roll_int(rng, range / 2, range);
    }
    if luck < 0.0 && crate::app::rand01(rng) < -luck {
        return roll_int(rng, range, range * 2);
    }
    roll_int(rng, 0, range)
}

/// `Chest.SetupTravelShop` (`Chest.cs:1275-1358`) — the travelling merchant's
/// per-visit stock roll.
///
/// The shape: a `Next(4, 7)` slot count grown by the four luck rolls
/// (`:1283-1298`), then the guaranteed rows — the hardmode first item
/// (`:1320-1334`), the `num` main items (`:1335-1343`) and the closing
/// painting (`:1347-1357`) — each through a 5000-attempt loop whose rarity
/// denominators ramp to a certainty as the attempts climb
/// (`SetupTravelShop_AdjustSlotRarities`, `Chest.cs:956-982`).
///
/// `luck` is `Player.GetPlayerWithHighestLuck().luck` (`:1281`) — the port's
/// single player.
pub fn setup_travel_shop(world: &World, luck: f32, rng: &mut u64) -> [u16; 40] {
    // `Main.travelShop` (`Main.cs:1779`) — the 40 slots start zeroed, which
    // shop index 19's walk reads as the empty ones.
    let mut shop = [0u16; 40];
    // `:1282`: the slot count `Main.rand.Next(4, 7)`.
    let mut num = roll_int(rng, 4, 7);
    // `:1283-1298`: the four `RollLuck` rows (the expert-mode `RollLuck(2)`
    // at `:1299` and the satchel/tenth-anniversary rows sit constants-off).
    if roll_luck(luck, 4, rng) == 0 {
        num += 1;
    }
    if roll_luck(luck, 8, rng) == 0 {
        num += 1;
    }
    if roll_luck(luck, 16, rng) == 0 {
        num += 1;
    }
    if roll_luck(luck, 32, rng) == 0 {
        num += 1;
    }
    // `:1317-1318`: the base rarity denominators, rarest tier first.
    let base: [u16; 6] = [100, 200, 300, 400, 500, 600];
    let mut added = 0usize;
    let mut count = 0usize;
    // `:1320-1334`: the hardmode arm rolls ONE item at `minimumRarity = 2`
    // (so only the top two tiers can answer) before the main loop.
    if world.hard_mode {
        let mut rarity = base;
        let mut attempts = 0;
        let mut it = 0u16;
        while attempts < 5000 {
            attempts += 1;
            adjust_slot_rarities(attempts, &mut rarity);
            get_travel_item(luck, &rarity, &mut it, 2, world, rng);
            if can_add_item_to_shop(&shop, it) {
                add_to_shop(&mut shop, &mut added, &mut count, it, rng);
                break;
            }
        }
    }
    // `:1335-1343`: the main loop — the base rarities, `minimumRarity = 0`.
    while added < num as usize {
        let mut it = 0u16;
        get_travel_item(luck, &base, &mut it, 0, world, rng);
        if can_add_item_to_shop(&shop, it) {
            add_to_shop(&mut shop, &mut added, &mut count, it, rng);
        }
    }
    // `:1344-1357`: the painting arm — a fresh rarity ramp and a fresh
    // attempt counter (the C# resets both).
    let mut rarity = base;
    let mut attempts = 0;
    let mut it = 0u16;
    while attempts < 5000 {
        attempts += 1;
        adjust_slot_rarities(attempts, &mut rarity);
        get_travel_painting(luck, &rarity, &mut it, 0, world, rng);
        if can_add_item_to_shop(&shop, it) {
            add_to_shop(&mut shop, &mut added, &mut count, it, rng);
            break;
        }
    }
    shop
}

/// `Chest.SetupTravelShop_AdjustSlotRarities` (`Chest.cs:956-982`): each rarity
/// denominator drops to 1 — a guaranteed hit — once the attempts pass its
/// threshold, so the loop always resolves within the 5000-attempt cap.
fn adjust_slot_rarities(attempts: i32, rarity: &mut [u16; 6]) {
    if rarity[5] > 1 && attempts > 4700 {
        rarity[5] = 1;
    }
    if rarity[4] > 1 && attempts > 4600 {
        rarity[4] = 1;
    }
    if rarity[3] > 1 && attempts > 4500 {
        rarity[3] = 1;
    }
    if rarity[2] > 1 && attempts > 4400 {
        rarity[2] = 1;
    }
    if rarity[1] > 1 && attempts > 4300 {
        rarity[1] = 1;
    }
    if rarity[0] > 1 && attempts > 4200 {
        rarity[0] = 1;
    }
}

/// `Chest.SetupTravelShop_CanAddItemToShop` (`Chest.cs:797-819`): the
/// duplicate suppression — a candidate already in any slot (or one of the
/// other slime-staff families, when the candidate is 3637) is rejected.
fn can_add_item_to_shop(shop: &[u16; 40], it: u16) -> bool {
    if it == 0 {
        return false;
    }
    for slot in shop.iter() {
        if *slot == it {
            return false;
        }
        if it == 3637 {
            // `:809-816`: the staff-family exclusion ranges (3621-3622 and
            // 3633-3642) — `(uint)(num - a) <= n`.
            let num = *slot as i32;
            if (num - 3621) as u32 <= 1 || (num - 3633) as u32 <= 9 {
                return false;
            }
        }
    }
    true
}

/// `Chest.SetupTravelShop_AddToShop` (`Chest.cs:708-795`): the row lands in
/// the next slot, and the bundle families write their tails right behind it
/// (the Dynasty Wood 2260 writes 2261/2262; the 3637 staff family rolls one
/// of six pairs instead).
fn add_to_shop(
    shop: &mut [u16; 40],
    added: &mut usize,
    count: &mut usize,
    item: u16,
    rng: &mut u64,
) {
    if item == 0 {
        return;
    }
    *added += 1;
    shop[*count] = item;
    *count += 1;
    let tails: &[u16] = match item {
        2260 => &[2261, 2262],
        5680 => &[5681, 5682],
        4555 => &[4556, 4557],
        4321 => &[4322],
        4323 => &[4324, 4365],
        5390 => &[5386, 5387],
        4666 => &[4664, 4665],
        _ => &[],
    };
    for tail in tails {
        if *count < shop.len() {
            shop[*count] = *tail;
            *count += 1;
        }
    }
    if item == 3637 {
        // `:764-793`: the staff family replaces its own row with one pair
        // out of six.
        *count -= 1;
        let pair: &[u16] = match roll_int(rng, 0, 6) {
            0 => &[3637, 3642],
            1 => &[3621, 3622],
            2 => &[3634, 3639],
            3 => &[3633, 3638],
            4 => &[3635, 3640],
            _ => &[3636, 3641],
        };
        for id in pair {
            if *count < shop.len() {
                shop[*count] = *id;
                *count += 1;
            }
        }
    }
}

/// `Chest.SetupTravelShop_GetItem` (`Chest.cs:984-1273`): the main item pool —
/// one `RollLuck` row per candidate, walking the rarity tiers from the
/// rarest down, each hit overwriting `it` (so the LAST hit in the tier order
/// wins, not a weighted table). The `minimumRarity` early-returns cut the
/// lower tiers the way the C#'s `if (minimumRarity > n) return;` arms do.
fn get_travel_item(
    luck: f32,
    rarity: &[u16; 6],
    it: &mut u16,
    minimum_rarity: i32,
    world: &World,
    rng: &mut u64,
) {
    let (r0, r1, r2, r3, r4, r5) = (rarity[0], rarity[1], rarity[2], rarity[3], rarity[4], rarity[5]);
    // `NPC.downedMechBossAny` (`NPC.cs:984`'s gates) — the port's three fields.
    let mech_any = world.downed_mech_boss1 || world.downed_mech_boss2 || world.downed_mech_boss3;
    let hit = |denom: u16, rng: &mut u64| roll_luck(luck, denom as i32, rng) == 0;
    // `:986-997`'s first three rows.
    if minimum_rarity <= 4 && hit(r4, rng) {
        *it = 3309;
    }
    if minimum_rarity <= 3 && hit(r3, rng) {
        *it = 3314;
    }
    if hit(r5, rng) {
        *it = 1987;
    }
    if minimum_rarity > 4 {
        return;
    }
    // `:1002-1017`'s rarity-4 tier.
    if hit(r4, rng) && world.hard_mode {
        *it = 2270;
    }
    if hit(r4, rng) && world.hard_mode {
        *it = 4760;
    }
    if hit(r4, rng) {
        *it = 2278;
    }
    if hit(r4, rng) {
        *it = 2271;
    }
    if minimum_rarity > 3 {
        return;
    }
    // `:1022-1085`'s rarity-3 tier.
    if hit(r3, rng)
        && world.hard_mode
        && mech_any
    {
        *it = 2223;
    }
    if hit(r3, rng) {
        *it = 2272;
    }
    if hit(r3, rng) {
        *it = 2276;
    }
    if hit(r3, rng) {
        *it = 2284;
    }
    if hit(r3, rng) {
        *it = 2285;
    }
    if hit(r3, rng) {
        *it = 2286;
    }
    if hit(r3, rng) {
        *it = 2287;
    }
    if hit(r3, rng) {
        *it = 4744;
    }
    if hit(r3, rng) && world.downed_skeletron {
        *it = 2296;
    }
    if hit(r3, rng) {
        *it = 3628;
    }
    if hit(r3, rng) && world.hard_mode {
        *it = 4091;
    }
    if hit(r3, rng) {
        *it = 4603;
    }
    if hit(r3, rng) {
        *it = 4604;
    }
    if hit(r3, rng) {
        *it = 5297;
    }
    if hit(r3, rng) {
        *it = 4605;
    }
    if hit(r3, rng) {
        *it = 4550;
    }
    if minimum_rarity > 2 {
        return;
    }
    // `:1090-1189`'s rarity-2 tier.
    if hit(r2, rng) {
        *it = 5680;
    }
    if hit(r2, rng) {
        *it = 2268;
    }
    if hit(r2, rng) && world.shadow_orb_smashed {
        *it = 2269;
    }
    if hit(r2, rng) {
        *it = 1988;
    }
    if hit(r2, rng) {
        *it = 2275;
    }
    if hit(r2, rng) {
        *it = 2279;
    }
    if hit(r2, rng) {
        *it = 2277;
    }
    if hit(r2, rng) {
        *it = 4555;
    }
    if hit(r2, rng) {
        *it = 4321;
    }
    if hit(r2, rng) {
        *it = 4323;
    }
    if hit(r2, rng) {
        *it = 5390;
    }
    if hit(r2, rng) {
        *it = 4549;
    }
    if hit(r2, rng) {
        *it = 4561;
    }
    if hit(r2, rng) {
        *it = 4774;
    }
    if hit(r2, rng) {
        *it = 5136;
    }
    if hit(r2, rng) {
        *it = 5305;
    }
    if hit(r2, rng) {
        *it = 4562;
    }
    if hit(r2, rng) {
        *it = 4558;
    }
    if hit(r2, rng) {
        *it = 4559;
    }
    if hit(r2, rng) {
        *it = 4563;
    }
    if hit(r2, rng) {
        *it = 4666;
    }
    if hit(r2, rng)
        && (world.downed_deerclops
            || world.downed_slime_king
            || world.downed_eoc
            || world.downed_eow
            || world.downed_skeletron
            || world.downed_queen_bee
            || world.hard_mode)
    {
        *it = 4347;
        if world.hard_mode {
            *it = 4348;
        }
    }
    if hit(r2, rng) && world.downed_eoc {
        *it = 3262;
    }
    if hit(r2, rng) && mech_any {
        *it = 3284;
    }
    if minimum_rarity > 1 {
        return;
    }
    // `:1194-1229`'s rarity-1 tier.
    if hit(r1, rng) {
        *it = 5600;
    }
    if hit(r1, rng) {
        *it = 2267;
    }
    if hit(r1, rng) {
        *it = 2214;
    }
    if hit(r1, rng) {
        *it = 2215;
    }
    if hit(r1, rng) {
        *it = 2216;
    }
    if hit(r1, rng) {
        *it = 2217;
    }
    if hit(r1, rng) {
        *it = 3624;
    }
    if hit(r1, rng) {
        *it = 2273;
    }
    if hit(r1, rng) {
        *it = 2274;
    }
    if minimum_rarity > 0 {
        return;
    }
    // `:1230-1272`'s rarity-0 tier — the commonest rows (the 2281 row rolls
    // three variants).
    if hit(r0, rng) {
        *it = 2266;
    }
    if hit(r0, rng) {
        *it = (2281 + roll_int(rng, 0, 3)) as u16;
    }
    if hit(r0, rng) {
        *it = 2258;
    }
    if hit(r0, rng) {
        *it = 2242;
    }
    if hit(r0, rng) {
        *it = 2260;
    }
    if hit(r0, rng) {
        *it = 3637;
    }
    if hit(r0, rng) {
        *it = 4420;
    }
    if hit(r0, rng) {
        *it = 3119;
    }
    if hit(r0, rng) {
        *it = 3118;
    }
    if hit(r0, rng) {
        *it = 3099;
    }
}

/// `Chest.SetupTravelShop_GetPainting` (`Chest.cs:821-954`): the painting arm —
/// the same last-hit-wins roll ladder, its `minimumRarity` early-returns
/// cutting the lower tiers.
fn get_travel_painting(
    luck: f32,
    rarity: &[u16; 6],
    it: &mut u16,
    minimum_rarity: i32,
    world: &World,
    rng: &mut u64,
) {
    let (r1, r2, r3) = (rarity[1], rarity[2], rarity[3]);
    let hit = |denom: u16, rng: &mut u64| roll_luck(luck, denom as i32, rng) == 0;
    // `:823-838`'s rarity-3 rows (the C# rolls each of the four separately,
    // so the last success wins).
    if hit(r3, rng) {
        *it = 5121;
    }
    if hit(r3, rng) {
        *it = 5122;
    }
    if hit(r3, rng) {
        *it = 5124;
    }
    if hit(r3, rng) {
        *it = 5123;
    }
    if minimum_rarity > 2 {
        return;
    }
    // `:843-890`'s rarity-2 tier (the `dontStarveWorld` gate on 5121-5124 is
    // constants-off — the port has no such seed).
    if hit(r2, rng) && world.hard_mode {
        *it = 3596;
    }
    if hit(r2, rng) && world.downed_martians {
        *it = 2865;
    }
    if hit(r2, rng) && world.downed_martians {
        *it = 2866;
    }
    if hit(r2, rng) && world.downed_martians {
        *it = 2867;
    }
    for id in [3055u16, 3056, 3057, 3058, 3059] {
        if hit(r2, rng) && world.downed_frost {
            *it = id;
        }
    }
    if hit(r2, rng) && world.hard_mode {
        *it = 5243;
    }
    if hit(r2, rng) {
        *it = 5530;
    }
    if hit(r2, rng) {
        *it = 5633;
    }
    if hit(r2, rng) {
        *it = 5636;
    }
    if minimum_rarity > 1 {
        return;
    }
    // `:895-953`'s rarity-1 tier.
    if hit(r1, rng) {
        *it = 5121;
    }
    if hit(r1, rng) {
        *it = 5122;
    }
    if hit(r1, rng) {
        *it = 5124;
    }
    if hit(r1, rng) {
        *it = 5123;
    }
    if hit(r1, rng) {
        *it = 5225;
    }
    if hit(r1, rng) {
        *it = 5229;
    }
    if hit(r1, rng) {
        *it = 5232;
    }
    if hit(r1, rng) {
        *it = 5389;
    }
    if hit(r1, rng) {
        *it = 5233;
    }
    if hit(r1, rng) {
        *it = 5241;
    }
    if hit(r1, rng) {
        *it = 5244;
    }
    if hit(r1, rng) {
        *it = 5487;
    }
    if hit(r1, rng) {
        *it = 5242;
    }
    if hit(r1, rng) {
        *it = 5531;
    }
}
fn row(item: u16, when: StockWhen) -> ShopEntry {
    ShopEntry { item, when }
}

/// The resolved stock — the gated rows filtered by the world state. The
/// `PlayerHasItem` rows read `inventory_ids` (the player inventory's item
/// ids, `:1468-1475`'s 58-slot scan).
pub fn resolve_stock(shop_index: u32, flags: &ShopFlags, inventory_ids: &[u16]) -> Vec<u16> {
    resolve_stock_moon(shop_index, flags, inventory_ids, 0)
}

/// The moon-phase-aware twin — `Main.moonPhase` rides the world clock.
pub fn resolve_stock_moon(
    shop_index: u32,
    flags: &ShopFlags,
    inventory_ids: &[u16],
    moon_phase: u32,
) -> Vec<u16> {
    setup_shop(shop_index, inventory_ids, flags)
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
            StockWhen::MoonPhaseOdd => moon_phase % 2 == 1,
            StockWhen::LifeMax400 => flags.life_max_400,
            StockWhen::ManaMax200 => flags.mana_max_200,
            StockWhen::CoinsOverGold => flags.coins_over_gold,
            StockWhen::MoonPhaseDayParity => (moon_phase % 2 == 0) == !flags.night,
            StockWhen::PartyGirlPresent => flags.party_girl_present,
            StockWhen::DownedMechAll => {
                flags.hard_mode && flags.downed_mech_1 && flags.downed_mech_2 && flags.downed_mech_3
            }
            StockWhen::DownedMechAny => flags.hard_mode && flags.downed_mech_any,
            StockWhen::DownedMartians => flags.hard_mode && flags.downed_martians,
            StockWhen::GolfPainting(pair) => {
                flags.golfer_score >= 2000 && moon_phase / 2 == pair as u32
            }
            StockWhen::MoonPhaseNight0 => moon_phase == 0 && flags.night,
            StockWhen::BestiaryCompletion(min) => flags.bestiary_completion >= min,
            StockWhen::HardModeMoonlord => false, // `downedMoonlord` is not ported yet
            StockWhen::DownedBothSlimes => false, // `downedQueenSlime` is not ported yet
            StockWhen::GolfScore(min, inclusive) => {
                if inclusive {
                    flags.golfer_score >= min
                } else {
                    flags.golfer_score > min
                }
            }
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

/// The reforge cost (`Main.DrawInventory`'s inline block,
/// `Main.cs:41584-41601`): `value * stack`, ×0.8 behind the Discount Card
/// (not ported — no discount accessory yet), × the happiness
/// `PriceAdjustment`, then the integer `/3`. The vanilla `<1 → 1` clamp is
/// the DISPLAY only — the actual charge at `:41658` is the unclamped
/// product (`BuyItem(0)` always succeeds on a value-0 item). `value` is
/// the instance's CURRENT value — the kind's defaults after the current
/// prefix's ×value² (`Item.cs:614-615`), so the cost scales with prefix
/// quality; the reforge slot always holds exactly one item
/// (`ItemSlot.cs:821-836`'s transfer splits stacks), so `stack` is 1.
pub(crate) fn reforge_cost(value: i32, stack: u32, mood: f32) -> u64 {
    // `Main.cs:41584-41601`'s order: value×stack (long), the float mood
    // multiply, THEN the integer /3.
    let product = (value as f32 * stack as f32 * mood) as i64;
    (product / 3).max(0) as u64
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
    use tr_core::World;

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
            ..Default::default()
        };
        let stock = resolve_stock(1, &flags, &[930]);
        for id in [
            188, 189, 488, 1348, 3198, 967, 33, 279, 282, 346, 4063, 4673, 931, 1614,
        ] {
            assert!(stock.contains(&id), "the gated row {id} missing");
        }
        // The PlayerHasItem rows need the scan hit.
        let without = resolve_stock(1, &flags, &[]);
        assert!(!without.contains(&931));
    }

    #[test]
    fn the_reforge_cost_follows_the_inline_block() {
        // `Main.cs:41584-41601`: the value × the stack, × the mood, then the
        // integer /3 — 2000 copper at mood 1 = 666, the happiness 1.25 =
        // 833; the <1 → 1 clamp is the display only, so a value-0 item
        // costs 0 and `BuyItem(0)` always succeeds.
        assert_eq!(reforge_cost(2000, 1, 1.0), 666);
        assert_eq!(reforge_cost(2000, 1, 1.25), 833);
        assert_eq!(reforge_cost(0, 1, 1.0), 0);
        assert_eq!(reforge_cost(1, 1, 1.0), 0);
    }

    #[test]
    fn the_unknown_shop_index_is_empty() {
        // The unclaimed indexes carry nothing (the ported ones land with
        // their NPCs — 1..8, 15..19, 22).
        assert!(resolve_stock(0, &ShopFlags::default(), &[]).is_empty());
        assert!(resolve_stock(30, &ShopFlags::default(), &[]).is_empty());
    }


    /// `Chest.SetupTravelShop` (`Chest.cs:1275-1358`): the visit's stock —
    /// the seed-7 roll pins the whole ladder (the `Next(4, 7)` count, the
    /// luck rolls, the main loop, the painting tail) and the output proves
    /// the bundle rows travel with their heads (`_AddToShop`'s tails) and the
    /// duplicate suppression holds across all 40 slots.
    #[test]
    fn the_travel_stock_rolls_its_bundles_dup_free() {
        let w = World::new("t".into(), 200, 200);
        let stock = setup_travel_shop(&w, 0.0, &mut 7);
        // The seed-7 snapshot — 4 main rows (the `Next(4, 7)` = 4 at luck 0)
        // plus the closing painting (5232 here), with the 4323 bundle's two
        // tails (4324/4365) and the staff family's 3635/3640 pair.
        assert_eq!(
            &stock[..10],
            &[3119, 4323, 4324, 4365, 2267, 5136, 3635, 3640, 4561, 5633],
            "the generated rows"
        );
        // The zeros: the tail slots are the empty ones (`Main.travelShop`'s
        // zero fill) and no slot repeats an id.
        assert_eq!(stock[..10].iter().filter(|v| **v != 0).count(), 10);
        assert!(stock[10..].iter().all(|v| *v == 0), "the tail is empty");
        let seen: Vec<u16> = stock.iter().copied().filter(|v| *v != 0).collect();
        for (i, id) in seen.iter().enumerate() {
            assert!(!seen[i + 1..].contains(id), "the duplicate {id}");
        }
        // The bundles: every head's tails sit directly behind it (the 2260
        // -> 2261/2262 arm of `_AddToShop`).
        let hm = {
            let mut w2 = World::new("t".into(), 200, 200);
            w2.hard_mode = true;
            setup_travel_shop(&w2, 0.0, &mut 99)
        };
        assert_eq!(&hm[..4], &[4091, 2279, 4555, 4556]);
        assert_eq!(hm[4], 4557, "the 4555 bundle's second tail");
        assert_eq!(&hm[5..8], &[2277, 2260, 2261]);
        assert_eq!(hm[8], 2262, "the Dynasty Wood bundle");
        // The hardmode arm's `minimumRarity = 2` tier reaches the hardmode
        // row (4091) — the first slot is a tier-3 row, never a rarity-0 one.
        assert!([2270, 4760, 2223, 4091, 4348].contains(&hm[0]), "the tier row");
        // The luck rows widen the stock: the same seed at luck 0.3 fills more
        // than the plain band's 4.
        let lucky = setup_travel_shop(&w, 0.3, &mut 7);
        assert!(
            lucky.iter().filter(|v| **v != 0).count() > 4,
            "the luck rolls add slots"
        );
    }

    /// `Chest.cs:2791-2810`'s shop index 19: the stock is the merchant's
    /// generated slots (not a fixed row list), the Palworld-collab rows ride
    /// the inventory gate and every row is `Always`.
    #[test]
    fn the_travel_shop_index_19_rides_the_generated_stock() {
        let stock = setup_travel_shop(&World::new("t".into(), 200, 200), 0.0, &mut 7);
        let flags = ShopFlags {
            travel_stock: stock,
            ..Default::default()
        };
        let rows = resolve_stock(19, &flags, &[]);
        assert_eq!(
            rows,
            vec![3119, 4323, 4324, 4365, 2267, 5136, 3635, 3640, 4561, 5633],
            "the shop rides the stock"
        );
        // The Palworld gate (`:2794-2800`): the Digtoise 5667 (or any of the
        // ten) puts the Pal Metal armor (5735/5736) ahead of the stock.
        let gated = resolve_stock(19, &flags, &[5667]);
        assert_eq!(&gated[..2], &[5735, 5736]);
        assert_eq!(&gated[2..], &rows[..]);
        let empty = resolve_stock(19, &ShopFlags::default(), &[]);
        assert!(empty.is_empty(), "an unvisited merchant's shop is empty");
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
        assert!(
            base.contains(&4430) && base.contains(&4431),
            "the pair 0 rows"
        );
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

