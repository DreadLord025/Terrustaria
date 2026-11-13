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
#[derive(Clone, Copy, Debug, PartialEq)]
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
    /// M5-SEED-2: `Main.vampireSeed` (`WorldGen.cs:555-558`'s
    /// `SecretSeed.vampirism.Enabled` write) — the Dryad's and the Bestiary
    /// Girl's vampirism rows (`Chest.cs:1733`/`:3403`).
    pub vampirism: bool,
    /// `Main.infectedSeed` (`WorldGen.cs:560-562`, the `worldIsInfected`
    /// micro seed) — the Dryad arm's second conjunct (`Chest.cs:1733`).
    pub infected_seed: bool,
    /// `WorldGen.crimson` — the world's evil type; the Dryad's vampirism row
    /// picks the Crimson Torch (4386) over the Corrupt one (4385)
    /// (`Chest.cs:1735-1740`). The port persists no world-evil type (see
    /// [`ShopFlags::from_world`]), so the caller fills it like the zones.
    pub crimson_world: bool,
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
            vampirism: false,
            infected_seed: false,
            crimson_world: false,
            travel_stock: [0; 40],
        }
    }
}

impl ShopFlags {
    /// M5-SEED-2: the world-side feed of the custom-seed half — the two
    /// statics `Chest.SetupShop`'s vampirism gates read live
    /// (`Main.vampireSeed`/`Main.infectedSeed`, set by
    /// `InitializeSecretSeeds` `WorldGen.cs:555-562`). The shop-open seam
    /// (the port has no shop UI call yet) starts from this and fills the
    /// scene flags; a classic world is exactly [`ShopFlags::default`].
    ///
    /// `crimson_world` stays false: the port persists no world-evil type
    /// (the same gap `loot.rs`'s `LootFlags.crimson_world` documents), so the
    /// Dryad's 4385/4386 split rides the caller's own field.
    pub fn from_world(world: &World) -> Self {
        Self {
            vampirism: world
                .micro_seeds
                .contains(tr_core::seeds::MicroSeedFlags::VAMPIRISM),
            infected_seed: world
                .micro_seeds
                .contains(tr_core::seeds::MicroSeedFlags::WORLD_IS_INFECTED),
            ..Self::default()
        }
    }
}

/// One `SetupShop` row: the item id + its gate + the per-row price fields
/// the C# stamps on the stock *instance* (`Item` carries them; the port's
/// stock is a fresh `SetDefaults` row, so they live on the entry).
#[derive(Clone, Copy, Debug)]
pub struct ShopEntry {
    pub item: u16,
    pub when: StockWhen,
    /// `Item.shopCustomPrice` — the per-row price override. The vanilla sets
    /// it on the Tavernkeep's rows (`Chest.cs:2972-3149`); it wins over the
    /// instance value in `Item.GetStoreValue` (`Item.cs:49901-49908`).
    pub shop_custom_price: Option<i32>,
    /// `Item.buyOnce` — the vanilla stamps it on rows the player sold into
    /// the shop (`Chest.AddItemToShop`, `Chest.cs:701`); a buyOnce row buys
    /// at the store value ÷ the adjustment ÷ 5 (`Player.cs:35381-35395`).
    pub buy_once: bool,
    /// `Item.isAShopItem` — the vanilla stamps it on whole shops
    /// (`Chest.cs:3538-3541`); under the Discount Card the sell value
    /// mirrors the discounted buy value (`Player.cs:35375-35378`).
    pub is_a_shop_item: bool,
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
    /// M5-SEED-2: `Chest.cs:3403-3406`'s Bestiary Girl tail — `Main.vampireSeed
    /// && !Main.infectedSeed` adds one Torch (8) after the moon-phase rows.
    VampirismNotInfected,
    /// M5-SEED-2: `Chest.cs:1733-1742`'s Dryad tail — `!Main.hardMode &&
    /// Main.vampireSeed && Main.infectedSeed` adds the Torch (8).
    VampirismInfected,
    /// The Dryad tail's `WorldGen.crimson` split — the Crimson Torch (4386).
    VampirismInfectedCrimson,
    /// The Dryad tail's other half — the Corrupt Torch (4385).
    VampirismInfectedCorrupt,
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
    /// `!Main.dayTime && Main.bloodMoon` (`case 12`'s 4663 row).
    BloodMoonNight,
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
            // M5-SEED-2: `:1733-1742`'s vampirism tail — the case's LAST rows,
            // after the `moonPhase / 2` flower-seed pairs: `!Main.hardMode &&
            // Main.vampireSeed && Main.infectedSeed` adds the Torch (8) and
            // then the world-evil torch (`WorldGen.crimson` → 4386, else 4385).
            row(8, StockWhen::VampirismInfected),
            row(4386, StockWhen::VampirismInfectedCrimson),
            row(4385, StockWhen::VampirismInfectedCorrupt),
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
        // The Dye Trader's shop (index 12, `Chest.cs:case 12`): the two
        // always rows, the halloween rows (constants-off), the day/night
        // moon-phase 0 pair, the blood-moon-night row and the graveyard
        /// goggles (constants-off); the multiplayer-only row is off.
        12 => vec![
            row(1120, StockWhen::Always),
            row(5920, StockWhen::Always),
            row(1037, StockWhen::Always),
            row(2874, StockWhen::Always),
            row(2871, StockWhen::MoonPhasePair(0)),
            row(2872, StockWhen::MoonPhasePair(0)),
            row(4663, StockWhen::BloodMoonNight),
        ],
        // The Party Girl's shop (index 13, `Chest.cs:case 13`): the
        /// 18 always rows (the food/party set) + the graveyard goggles
        /// and the lantern-night row (both constants-off) + the
        /// golfer-score row + the Pirate's presence row (unported)
        /// + the Golem's row + the hardmode dye set + the BirthdayParty
        /// block (constants-off until the party event).
        13 => vec![
            row(859, StockWhen::Always),
            row(1000, StockWhen::Always),
            row(1168, StockWhen::Always),
            row(1449, StockWhen::Always), // `Main.dayTime` — the port has no day gate yet
            row(4552, StockWhen::Night),
            row(1345, StockWhen::Always),
            row(1450, StockWhen::Always),
            row(3253, StockWhen::Always),
            row(4553, StockWhen::Always),
            row(2700, StockWhen::Always),
            row(2738, StockWhen::Always),
            row(4470, StockWhen::Always),
            row(4681, StockWhen::Always),
            row(4791, StockWhen::Always),
            row(3747, StockWhen::Always),
            row(3732, StockWhen::Always),
            row(3742, StockWhen::Always),
        ],
        // The Zoologist's shop (index 23, `Chest.cs:3263-3340`): the
        // bestiary tools — the progress-gated rows ride
        /// `BestiaryUnlockProgressReport.CompletionPercent`
        /// (`:3265`), the lumen/solar rows the flags they name. The case's
        // vampirism tail (`Main.vampireSeed && !Main.infectedSeed` → one
        /// Torch 8, `Chest.cs:3403-3406`) closes the list — the M5-SEED-2
        /// arm, folded into this one arm so the match keeps one entry.
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
            // The vampirism tail (`Chest.cs:3403-3406`).
            row(8, StockWhen::VampirismNotInfected),
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
    ShopEntry {
        item,
        when,
        shop_custom_price: None,
        buy_once: false,
        is_a_shop_item: false,
    }
}

/// The resolved stock — the gated rows filtered by the world state. The
/// `PlayerHasItem` rows read `inventory_ids` (the player inventory's item
/// ids, `:1468-1475`'s 58-slot scan).
/// The tests' id-view over `resolve_stock_entries` (the production stock
/// prices the ROW, so the buy path consumes the entries).
#[cfg(test)]
pub fn resolve_stock(shop_index: u32, flags: &ShopFlags, inventory_ids: &[u16]) -> Vec<u16> {
    resolve_stock_moon(shop_index, flags, inventory_ids, 0)
}

/// The stock with its per-row price fields (`Item.shopCustomPrice` /
/// `buyOnce` / `isAShopItem`) — the buy path prices the ROW
/// (`Item.GetStoreValue`, `Item.cs:49901-49908`), not the kind default.
pub fn resolve_stock_entries(
    shop_index: u32,
    flags: &ShopFlags,
    inventory_ids: &[u16],
) -> Vec<ShopEntry> {
    resolve_stock_entries_moon(shop_index, flags, inventory_ids, 0)
}

/// The moon-phase-aware twin (the tests' id-view — see `resolve_stock`).
#[cfg(test)]
pub fn resolve_stock_moon(
    shop_index: u32,
    flags: &ShopFlags,
    inventory_ids: &[u16],
    moon_phase: u32,
) -> Vec<u16> {
    resolve_stock_entries_moon(shop_index, flags, inventory_ids, moon_phase)
        .iter()
        .map(|e| e.item)
        .collect()
}

pub fn resolve_stock_entries_moon(
    shop_index: u32,
    flags: &ShopFlags,
    inventory_ids: &[u16],
    moon_phase: u32,
) -> Vec<ShopEntry> {
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
            StockWhen::VampirismNotInfected => flags.vampirism && !flags.infected_seed,
            StockWhen::VampirismInfected => {
                !flags.hard_mode && flags.vampirism && flags.infected_seed
            }
            StockWhen::VampirismInfectedCrimson => {
                !flags.hard_mode
                    && flags.vampirism
                    && flags.infected_seed
                    && flags.crimson_world
            }
            StockWhen::VampirismInfectedCorrupt => {
                !flags.hard_mode
                    && flags.vampirism
                    && flags.infected_seed
                    && !flags.crimson_world
            }
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
            StockWhen::BloodMoonNight => flags.night && flags.blood_moon,
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
        .collect()
}

/// `Item.GetStoreValue` (`Item.cs:49901-49908`): the per-row
/// `shopCustomPrice` override wins over the instance's current `value`
/// (the kind default after the prefix's ×value², `Item.cs:614-615` — the
/// port carries it as `prefix::prefixed_stats(kind, prefix).value`).
pub fn get_store_value(instance_value: i32, shop_custom_price: Option<i32>) -> i32 {
    shop_custom_price.unwrap_or(instance_value)
}

/// `Player.GetItemExpectedPrice` (`Player.cs:35359-35396`) — the
/// `(calc_for_selling, calc_for_buying)` pair. The `shopSpecialCurrency
/// != -1` early return (`:35361-35365`, the Tavernkeep's defender-medal
/// rows) needs the `CustomCurrencyManager` — not ported (no custom
/// currencies), so those rows must not reach this helper yet. The
/// arithmetic rides the C#'s f32 casts: the discount multiply truncates
/// (the `(int)` cast — no rounding), the adjustment multiply/divide round
/// as `Math.Round` does (the banker's rounding).
pub fn get_item_expected_price(
    store_value: i32,
    buy_once: bool,
    is_a_shop_item: bool,
    discount_available: bool,
    price_adjustment: f32,
) -> (i64, i64) {
    let store_value = store_value as i64; // `int storeValue = item.GetStoreValue();`
    let mut calc_for_selling = store_value;
    let mut calc_for_buying = store_value;
    if discount_available {
        if !buy_once {
            // `(int)((float)calcForBuying * 0.8f)` — truncate, no rounding.
            calc_for_buying = (calc_for_buying as f32 * 0.8f32) as i32 as i64;
        }
        if is_a_shop_item {
            calc_for_selling = calc_for_buying;
        }
    }
    if buy_once {
        calc_for_buying = round_half_to_even(calc_for_buying as f32 / price_adjustment) as i64;
        calc_for_selling = round_half_to_even(calc_for_selling as f32 / price_adjustment) as i64;
    } else {
        calc_for_buying = round_half_to_even(calc_for_buying as f32 * price_adjustment) as i64;
        calc_for_selling = round_half_to_even(calc_for_selling as f32 / price_adjustment) as i64;
    }
    if buy_once {
        calc_for_buying /= 5;
        if store_value != 0 && calc_for_buying < 1 {
            calc_for_buying = 1;
        }
    }
    (calc_for_selling, calc_for_buying)
}

/// The stock row's `(selling, buying)` pair: the row prices its store
/// value — `GetStoreValue` over the `shopCustomPrice` override and the
/// kind default (`SetupShop` rows are fresh `SetDefaults` instances,
/// prefix 0).
fn row_prices(entry: &ShopEntry, discount_available: bool, price_adjustment: f32) -> (i64, i64) {
    let store = get_store_value(ITEMS[entry.item as usize].value, entry.shop_custom_price);
    get_item_expected_price(
        store,
        entry.buy_once,
        entry.is_a_shop_item,
        discount_available,
        price_adjustment,
    )
}

/// The buy charge for one stock row (`ItemSlot.HandleShopSlot`,
/// `UI/ItemSlot.cs:2059-2060`): `BuyItem(calcForBuying, …)` per unit —
/// the happiness adjustment and the Discount Card included.
pub(crate) fn buy_price_row(
    entry: &ShopEntry,
    discount_available: bool,
    price_adjustment: f32,
) -> i64 {
    row_prices(entry, discount_available, price_adjustment).1
}

/// The price a stock row DISPLAYS (`Main.MouseText_DrawItemTooltip_AddShopLines`,
/// `Main.cs:20457`): buyOnce/isAShopItem rows show the buying value, every
/// other row shows the SELLING value — the two diverge whenever
/// `PriceAdjustment != 1` (the display quirk is verbatim).
pub(crate) fn display_price_row(
    entry: &ShopEntry,
    discount_available: bool,
    price_adjustment: f32,
) -> i64 {
    let (calc_for_selling, calc_for_buying) = row_prices(entry, discount_available, price_adjustment);
    if entry.buy_once || entry.is_a_shop_item {
        calc_for_buying
    } else {
        calc_for_selling
    }
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

/// The sell transaction's per-item price (`Player.SellItem`,
/// `Player.cs:35168-35192`): `GetItemExpectedPrice`'s selling value ÷ 5,
/// min 1 — the vanilla's 20 % sale rate lives HERE, after the adjustment
/// divide (the port used to fold it into the helper in the wrong order).
/// The `calcForSelling <= 0` refusal (`:35170-35172`) returns 0; the
/// sellback correction (`Main.shopSellbackHelper`, `:35191-35193`) and the
/// stack multiply ride the sell click's own pass. The caller prices the
/// instance — `prefix::prefixed_stats(kind, prefix).value`'s ×value².
pub fn sell_price(instance_value: i32, price_adjustment: f32) -> i64 {
    let (calc_for_selling, _) =
        get_item_expected_price(instance_value, false, false, false, price_adjustment);
    if calc_for_selling <= 0 {
        return 0;
    }
    (calc_for_selling / 5).max(1)
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
        // `Player.cs:35386-35388`: the buy = the store value × the
        // adjustment, the sell = the store value ÷ the adjustment — the
        // Mining Helmet 88's store value is 40000.
        assert_eq!(
            get_item_expected_price(40000, false, false, false, 1.0),
            (40000, 40000)
        );
        assert_eq!(
            get_item_expected_price(40000, false, false, false, 0.75),
            (53333, 30000)
        );
        assert_eq!(
            get_item_expected_price(40000, false, false, false, 1.5),
            (26667, 60000)
        );
        // The Guide-in-the-forest multiplier from the happiness tests.
        assert_eq!(
            get_item_expected_price(40000, false, false, false, 0.89),
            (44944, 35600)
        );
    }

    #[test]
    fn the_rounding_is_the_bankers_midpoint() {
        // `Math.Round`'s midpoint → even: 5×0.5f = 2.5f → 2, 15×0.5f =
        // 7.5f → 8 (`Player.cs:35387-35388`).
        assert_eq!(get_item_expected_price(5, false, false, false, 0.5), (10, 2));
        assert_eq!(get_item_expected_price(15, false, false, false, 0.5), (30, 8));
        // The buyOnce divide rounds first, then `/5`: 1/2f = 0.5f → 0
        // (even) → the `storeValue != 0` clamp lifts it to 1.
        assert_eq!(get_item_expected_price(1, true, false, false, 2.0), (0, 1));
    }

    #[test]
    fn the_discount_card_marks_the_buy_price_down() {
        // `Player.cs:35369-35378`: the 0.8 multiply TRUNCATES (the
        // `(int)` cast — no rounding), and only `isAShopItem` rows mirror
        // it into the sell value.
        assert_eq!(get_item_expected_price(40000, false, false, true, 1.0), (40000, 32000));
        // (int)((float)7 * 0.8f) = (int)5.599999904632568f = 5 — a
        // rounding here would say 6.
        assert_eq!(get_item_expected_price(7, false, false, true, 1.0), (7, 5));
        assert_eq!(get_item_expected_price(40000, false, true, true, 1.0), (32000, 32000));
    }

    #[test]
    fn the_buy_once_rows_price_at_a_fifth() {
        // `Player.cs:35381-35395`: a buyOnce row (the vanilla stamps it on
        // rows the player sold in, `Chest.cs:701`) buys at the store
        // value ÷ the adjustment ÷ 5 — the sell value keeps the plain
        // divide; the `< 1` clamp fires only for a non-zero store value
        // (`storeValue != 0`, `:35392-35394`).
        assert_eq!(get_item_expected_price(40000, true, false, false, 1.0), (40000, 8000));
        // The round-then-divide order: 53333 / 5 = 10666 (the port used to
        // divide first and round — 10667).
        assert_eq!(get_item_expected_price(40000, true, false, false, 0.75), (53333, 10666));
        assert_eq!(get_item_expected_price(3, true, false, false, 1.0), (3, 1));
        // A zero-value buyOnce row stays 0 (no clamp on a zero store value).
        assert_eq!(get_item_expected_price(0, true, false, false, 1.0), (0, 0));
    }

    #[test]
    fn the_store_value_prefers_the_shop_custom_price() {
        // `Item.cs:49901-49908`: `shopCustomPrice ?? value`.
        assert_eq!(get_store_value(40000, None), 40000);
        assert_eq!(get_store_value(40000, Some(50)), 50);
    }

    #[test]
    fn the_stock_row_prices_the_row_not_the_kind() {
        // The buy charge (`ItemSlot.cs:2060`) rides the row's
        // `shopCustomPrice` override (`Item.cs:49901-49908`).
        let custom = ShopEntry {
            item: 88,
            when: StockWhen::Always,
            shop_custom_price: Some(50),
            buy_once: false,
            is_a_shop_item: false,
        };
        assert_eq!(buy_price_row(&custom, false, 1.0), 50);
        assert_eq!(buy_price_row(&row(88, StockWhen::Always), false, 1.0), 40000);
        // The display quirk (`Main.cs:20457`): a plain row DISPLAYS the
        // selling value — 53333 at the 0.75 mood, not the 30000 charge —
        // while buyOnce/isAShopItem rows display the charge.
        let plain = row(88, StockWhen::Always);
        assert_eq!(display_price_row(&plain, false, 0.75), 53333);
        assert_eq!(buy_price_row(&plain, false, 0.75), 30000);
        let once = ShopEntry {
            buy_once: true,
            ..plain
        };
        assert_eq!(display_price_row(&once, false, 0.75), 10666);
    }

    #[test]
    fn the_entry_stock_matches_the_id_stock() {
        // The ids wrapper stays the tests' surface; the entries carry the
        // row's price fields (all unset on the ported shops).
        let ids = resolve_stock(1, &ShopFlags::default(), &[]);
        let entries = resolve_stock_entries(1, &ShopFlags::default(), &[]);
        assert_eq!(ids, entries.iter().map(|e| e.item).collect::<Vec<_>>());
        assert!(entries
            .iter()
            .all(|e| e.shop_custom_price.is_none() && !e.buy_once && !e.is_a_shop_item));
    }

    // M5-SEED-2: `Main.vampireSeed`/`Main.infectedSeed`
    // (`WorldGen.cs:555-562`) arm two shops — the Dryad's
    // `!hardMode && vampireSeed && infectedSeed` tail (`Chest.cs:1733-1742`)
    // and the Bestiary Girl's `vampireSeed && !infectedSeed` torch
    // (`Chest.cs:3403-3406`). Both gates read the world, so a classic world
    // resolves neither shop's extra rows.
    #[test]
    fn the_vampirism_arms_add_the_torch_rows_under_the_flag() {
        let classic = ShopFlags::default();
        let vamp = ShopFlags {
            vampirism: true,
            ..ShopFlags::default()
        };
        let both = ShopFlags {
            vampirism: true,
            infected_seed: true,
            ..ShopFlags::default()
        };
        // The Dryad (shop 3): the torch rows are the case's LAST rows, after
        // the flower-seed pairs, and only while the world is not in hardmode.
        let base = resolve_stock_moon(3, &classic, &[], 0);
        assert!(!base.contains(&8), "the Dryad sells no torch by default");
        assert!(!base.contains(&4385) && !base.contains(&4386));
        let seeded = resolve_stock_moon(3, &both, &[], 0);
        let tail = seeded.len() - base.len();
        assert_eq!(tail, 2, "the torch plus the world-evil torch");
        assert!(seeded.contains(&8), "the Torch 8");
        assert!(seeded.contains(&4385), "the Corrupt Torch (no crimson flag)");
        assert!(!seeded.contains(&4386), "the Crimson Torch stays out");
        // `WorldGen.crimson` flips the split.
        let crimson = ShopFlags {
            crimson_world: true,
            ..both
        };
        let rows = resolve_stock_moon(3, &crimson, &[], 0);
        assert!(rows.contains(&4386) && !rows.contains(&4385), "the crimson split");
        // Hardmode closes the arm entirely (`:1733`'s `!Main.hardMode`).
        let hard = ShopFlags {
            hard_mode: true,
            ..both
        };
        let hard_rows = resolve_stock_moon(3, &hard, &[], 0);
        assert!(!hard_rows.contains(&8) && !hard_rows.contains(&4385));
        // A vampirism-only world (no `infectedSeed`): the Dryad arm stays off
        // and the Bestiary Girl's opens instead.
        let girl = resolve_stock(23, &vamp, &[]);
        assert!(girl.contains(&8), "the Bestiary Girl's vampirism torch: {girl:?}");
        assert!(
            !resolve_stock(23, &classic, &[]).contains(&8),
            "a classic world gets no vampirism torch"
        );
        assert!(
            !resolve_stock(23, &both, &[]).contains(&8),
            "`!infectedSeed` is required"
        );
        // The world-side feed: a classic world is the default.
        let w = World::new("t".into(), 200, 200);
        assert_eq!(ShopFlags::from_world(&w), ShopFlags::default());
    }
}

#[cfg(test)]
mod town5_tests {
    use super::*;

    #[test]
    fn the_buy_price_matches_the_happiness_multiplier() {
        // The round trip: the Mining Helmet's 40000 base rides the mood
        // (`Player.cs:35387`).
        let plain = row(88, StockWhen::Always);
        assert_eq!(buy_price_row(&plain, false, 1.5), 60000);
        assert_eq!(buy_price_row(&plain, false, 0.89), 35600);
    }

    #[test]
    fn the_sell_price_prices_the_instance_and_divides_after_the_round() {
        // The seed test (research/07): the sell chain prices the INSTANCE
        // (`prefix::prefixed_stats`'s ×value², `Item.cs:614-615`) and
        // divides by 5 AFTER the adjustment round (`Player.cs:35189`).
        let large = crate::prefix::prefixed_stats(88, 1).unwrap().value;
        assert_eq!(large, 50176); // (int)((float)40000 × 1.12f²)
        assert_eq!(sell_price(large, 1.0), 10035); // 50176 / 5
        assert_eq!(sell_price(40000, 1.0), 8000);
        assert_eq!(sell_price(40000, 0.75), 10666);
        // The min-1 clamp and the ≤ 0 refusal (`Player.cs:35170-35192`).
        assert_eq!(sell_price(1, 1.0), 1);
        assert_eq!(sell_price(0, 1.0), 0);
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

