//! The town shops — `Chest.SetupShop` (`Chest.cs:1360+`) and the shop
//! pricing (`Player.GetItemExpectedPrice`, `Player.cs:35359-35389`). The
//! vanilla opens a shop through `NPCInteractions.Shop(npcType, shopIndex)`
//! (`NPCInteractions.cs:488-512`) — the `SetupShop` switch keys on the SHOP
//! INDEX, not the NPC type (the Guide (22) offers no shop at all — the
//! GuideTip/GuideReverseCrafting interactions only; the first seller is the
//! Merchant (17) through the shop index 1).

use crate::itemdata::ITEMS;
use tr_core::{UnifiedRandom, World};

/// The world state the stock gates read — `SetupShop`'s if-gates collapse
/// into one struct the caller fills (the port's flags exist for all of
/// these; the halloween rows sit constants-off — no season system).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ShopFlags {
    pub hard_mode: bool,
    /// `NPC.downedSlimeKing`/`downedQueenSlime`/`downedMoonlord` — the
    /// Skeleton Merchant's rank gates (`Chest.cs:3428-3434`, S1-F-16).
    pub downed_slime_king: bool,
    pub downed_queen_slime: bool,
    pub downed_moon_lord: bool,
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
    /// `NPC.downedPlantBoss` (`Chest.cs:2122`) — the Truffle's Autohammer.
    pub downed_plant_boss: bool,
    /// The secret-seed trio the `!Main.remixWorld || (Main.tenthAnniversaryWorld
    /// && !Main.getGoodWorld)` gates read (`Chest.cs:2130`/`:2137`/`:2207`,
    /// set by `WorldGen.cs:11118-11121` from the seed options). The port's
    /// `World.seeds` bits feed them (`SeedFlags::REMIX`/`TENTH_ANNIVERSARY`/
    /// `FOR_THE_WORTHY`).
    pub remix_world: bool,
    pub anniversary_world: bool,
    pub get_good_world: bool,
    /// `Main.eclipse` (`Chest.cs:2209`/`:2368-2377`) — the Steampunker's
    /// solution split and the Cyborg's rocket tiers.
    pub eclipse: bool,
    /// `NPC.downedBoss1` (`Chest.cs:2174`) — the Steampunk Boiler's
    /// all-three-pre-hardmode-bosses gate (the port's `downed_eoc`).
    pub downed_boss1: bool,
    /// `NPC.downedGolemBoss` (`Chest.cs:2151`/`:2195`) — the Steampunk
    /// Wings and the jungle Lihzahrd Furnace.
    pub downed_golem: bool,
    /// `Main.player[Main.myPlayer].ZoneGraveyard` (`Chest.cs:2186`).
    pub zone_graveyard: bool,
    /// `Main.player[Main.myPlayer].ZoneSkyHeight` (`Chest.cs:2203`).
    pub zone_sky_height: bool,
    /// `Main.player[Main.myPlayer].ZoneHallow` (`Chest.cs:2220`).
    pub zone_hallow: bool,
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
    /// W4-d122 shop 1: `Main.notTheBeesWorld` (`WorldGen.cs:11116`, the
    /// `SecretSeed.notTheBees` option) — the Merchant's Honey Dispenser
    /// block 4388 (`Chest.cs:1387`) and the jungle-row anniversary arm
    /// (`:1417`). Fed from `World.seeds`' `NOT_THE_BEES` bit.
    pub not_the_bees: bool,
    /// W4-d122 shop 1: `Main.IsItAHappyWindyDay` (`Main.cs:2903`, the
    /// sticky windy-day flag the port's `Wind.happy_windy_day` carries) —
    /// the Pin Wheel 4074 (`Chest.cs:1422`'s `dayTime &&` conjunct).
    pub happy_windy_day: bool,
    /// W4-d122 shop 1: `BirthdayParty.PartyIsUp` (`BirthdayParty.cs:25-32`)
    /// — the Party Bundle 5643 (`Chest.cs:1435`).
    pub party_is_up: bool,
    /// W4-d122 shop 2: `WorldGen.SavedOreTiers.Silver` (`WorldGen.cs:3289`)
    /// — the world's rolled silver-tier ore id (9 = Silver, 168 = Tungsten;
    /// `WorldGen.cs:11290`); the Arms Dealer's bullet split compares it to
    /// 168 (`Chest.cs:1489`). Fed from `World.ore_tiers.silver`.
    pub silver_ore_tier: i32,
    /// W4-d122 shop 3: `Main.player[Main.myPlayer].ZoneUnderworldHeight`
    /// (`Chest.cs:1647`) — the Dryad's Hellcake 5214 (and the 194 else-if).
    pub zone_underworld: bool,
    /// W4-d122 shop 3: `Main.player[Main.myPlayer].ZoneGlowshroom`
    /// (`Chest.cs:1651`) — the Mushroom set 194 under the else-if.
    pub zone_glowshroom: bool,
    /// W4-d122 shop 4: `NPC.downedPirates` (`Chest.cs:1766`) — the
    /// Demolitionist's Holy Hand Grenade 937 triple conjunct. Fed from
    /// `World.downed_pirates`.
    pub downed_pirates: bool,
    /// W4-d122 shop 5: `NPC.downedClown` (`Chest.cs:1849`) — the port
    /// persists no clown latch, so the caller fills it (false until the
    /// hardmode blood-moon clown kill lands a bit).
    pub downed_clown: bool,
    /// W4-d122 shop 5: `NPC.downedAncientCultist` (`Chest.cs:1868`) —
    /// fed from `World.downed_ancient_cultist`.
    pub downed_cultist: bool,
    /// W4-d122 shop 5: `NPC.downedFrost` (`Chest.cs:1953`) — fed from
    /// `World.downed_frost`.
    pub downed_frost: bool,
    /// W4-d122 shop 5: `NPC.AnyNPCs(441)` (`Chest.cs:1881`) — the Tax
    /// Collector's presence (the live roster scan, like the Stylist's
    /// Party Girl gate).
    pub tax_collector_present: bool,
}

impl Default for ShopFlags {
    fn default() -> Self {
        Self {
            hard_mode: false,
            downed_slime_king: false,
            downed_queen_slime: false,
            downed_moon_lord: false,
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
            downed_plant_boss: false,
            remix_world: false,
            anniversary_world: false,
            get_good_world: false,
            eclipse: false,
            downed_boss1: false,
            downed_golem: false,
            zone_graveyard: false,
            zone_sky_height: false,
            zone_hallow: false,
            golfer_score: 0,
            bestiary_completion: 0.0,
            unlocked_princess: false,
            vampirism: false,
            infected_seed: false,
            crimson_world: false,
            travel_stock: [0; 40],
            not_the_bees: false,
            happy_windy_day: false,
            party_is_up: false,
            // `SavedOreTiers::new`'s static default (`WorldGen.cs:3303`).
            silver_ore_tier: 9,
            zone_underworld: false,
            zone_glowshroom: false,
            downed_pirates: false,
            downed_clown: false,
            downed_cultist: false,
            downed_frost: false,
            tax_collector_present: false,
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
            hard_mode: world.hard_mode,
            downed_slime_king: world.downed_slime_king,
            downed_queen_slime: world.downed_queen_slime,
            downed_moon_lord: world.downed_moon_lord,
            night: !world.day_time,
            blood_moon: world.blood_moon,
            vampirism: world
                .micro_seeds
                .contains(tr_core::seeds::MicroSeedFlags::VAMPIRISM),
            infected_seed: world
                .micro_seeds
                .contains(tr_core::seeds::MicroSeedFlags::WORLD_IS_INFECTED),
            downed_plant_boss: world.downed_plant_boss,
            // `WorldGen.cs:11118-11121`: the three statics ride the seed
            // options — the port's persisted `World.seeds` bits.
            remix_world: world.seeds.contains(tr_core::seeds::SeedFlags::REMIX),
            anniversary_world: world
                .seeds
                .contains(tr_core::seeds::SeedFlags::TENTH_ANNIVERSARY),
            get_good_world: world
                .seeds
                .contains(tr_core::seeds::SeedFlags::FOR_THE_WORTHY),
            eclipse: world.eclipse,
            downed_boss1: world.downed_eoc,
            downed_golem: world.downed_golem,
            // W4-d122 shop 1: `Main.notTheBeesWorld` rides the seed options
            // (`WorldGen.cs:11116`) — the port's persisted `World.seeds` bit.
            not_the_bees: world.seeds.contains(tr_core::seeds::SeedFlags::NOT_THE_BEES),
            // W4-d122 shop 2: the rolled silver tier (`WorldGen.cs:11268`/
            // `:11290`) — the port's persisted `ore_tiers`.
            silver_ore_tier: world.ore_tiers.silver,
            // W4-d122 shop 4: `NPC.downedPirates` — the world's latch.
            downed_pirates: world.downed_pirates,
            // W4-d122 shop 5: the Clothier's cultist/frost latches.
            downed_cultist: world.downed_ancient_cultist,
            downed_frost: world.downed_frost,
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
    /// `Item.prefix` — 0 on the fresh `SetDefaults` stock rows; a row the
    /// player sold in keeps the instance's prefix (`Chest.AddItemToShop`'s
    /// `Clone`, `Chest.cs:699`), and the buy copies it onto the cursor
    /// (`ItemSlot.cs:2078-2081`).
    pub prefix: u16,
    /// `Item.stack` — the row's remaining count. Only `buyOnce` rows
    /// deplete (`ItemSlot.cs:2090-2093`); the plain stock rows keep the
    /// `SetDefaults` 1 and buy forever.
    pub stack: u32,
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
    /// `ZoneUnderworldHeight` (`Chest.cs:1647`) — the Hellcake 5214; the
    /// scene metrics feed it at the open seam.
    ZoneUnderworld,
    /// `ZoneGlowshroom` (`Chest.cs:1651`'s else-if — the Mushroom set 194
    /// only when NOT underworld) — the scene's glowing-mushroom tile
    /// count feeds it.
    ZoneGlowshroom,
    /// The moon-phase pairs (`Chest.cs:1685-1732`'s `moonPhase / 2`
    /// switches): the row appears when `moon_phase / 2 == n`; pair 3 is
    /// the C#'s `default:` arm (phases 6/7).
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
    /// `NPC.downedMechBossAny` ALONE (`Chest.cs:2113` — the Truffle's
    /// Mushroom Spear/Hammush pair; no hardmode conjunct in the C#).
    DownedMechAnyBare,
    /// `NPC.downedPlantBoss` (`Chest.cs:2122`) — the Autohammer 1551.
    DownedPlantBoss,
    /// `!Main.remixWorld || (Main.tenthAnniversaryWorld &&
    /// !Main.getGoodWorld)` (`Chest.cs:2130`/`:2137`/`:2207`) — the
    /// Dark Blue Solution 783, the Clentaminator 779 and the whole
    /// solution/moonlord tail block.
    NotRemixOrAnniversary,
    /// `Main.hardMode || !Main.getGoodWorld` (`Chest.cs:2170`) — the
    /// Blend-O-Matic 995.
    HardModeOrNotGood,
    /// `Main.eclipse` (`Chest.cs:2209`/`:2373`) — the bare event flag.
    Eclipse,
    /// `NPC.downedGolemBoss` (`Chest.cs:2151`) — the bare Golem flag.
    DownedGolem,
    /// `NPC.downedBoss1 && NPC.downedBoss2 && NPC.downedBoss3`
    /// (`Chest.cs:2174`) — the Steampunk Boiler 2203.
    DownedAllPreHardBosses,
    /// `WorldGen.crimson` (`Chest.cs:2178`) — the Flesh Cloning Vaat 2193
    /// over the Lesion Station 4142 split.
    CrimsonWorld,
    /// `Main.player[Main.myPlayer].ZoneGraveyard` (`Chest.cs:2186`).
    ZoneGraveyard,
    /// `Main.player[Main.myPlayer].ZoneSkyHeight` (`Chest.cs:2203`).
    ZoneSkyHeight,
    /// `Main.player[Main.myPlayer].ZoneHallow` (`Chest.cs:2220`).
    ZoneHallow,
    /// `NPC.downedMoonlord` (`Chest.cs:2228`) — the bare Moon Lord flag
    /// (the solution trio 5392/5393/5394; NOT the hardmode conjunct).
    DownedMoonlord,
    /// `NPC.downedMartians` ALONE (`Chest.cs:2378`/`:2430` — the Cyborg's
    /// cluster rockets and the HiTek/NightVision pair; no hardmode
    /// conjunct in the C#, unlike the Stylist's `DownedMartians`).
    DownedMartiansBare,
    /// `Main.halloween` (`Chest.cs:2240`) — constants-off: the port has no
    /// season system yet (the Steampunk Goggles 1742 stay hidden).
    Halloween,
    /// `Main.moonPhase >= 4` (`Chest.cs:2141`'s first conjunct) — the
    /// Steampunker's Jetpack-vs-outfit split reads it with HardMode.
    MoonPhaseAtLeast(u32),
    /// The conjunction of the child gates — the C#'s `&&` chains
    /// (`Chest.cs:2141`'s `moonPhase >= 4 && hardMode`, `:2195`'s
    /// `zoneJungle && downedGolemBoss`). The children evaluate in order.
    All(&'static [StockWhen]),
    /// The disjunction of the child gates — the C#'s `||` chains
    /// (`Chest.cs:2209`'s `eclipse || bloodMoon`).
    Any(&'static [StockWhen]),
    /// The negation — the C#'s `else` arms (`Chest.cs:2145-2150`'s outfit
    /// trio is the NOT of the Jetpack gate; `:2224-2227`'s Green Solution
    /// is the NOT of both prior branches).
    Not(&'static StockWhen),
    /// W4-d122 shop 1: `Main.notTheBeesWorld && !Main.remixWorld`
    /// (`Chest.cs:1387`) — the Honey Dispenser block 4388.
    NotTheBeesNotRemix,
    /// W4-d122 shop 1: `Main.player[Main.myPlayer].ZoneJungle ||
    /// (Main.tenthAnniversaryWorld && Main.notTheBeesWorld &&
    /// !Main.remixWorld)` (`Chest.cs:1417`) — the Jungle Rose 33.
    JungleOrBeesAnniversary,
    /// W4-d122 shop 1: `Main.dayTime && Main.IsItAHappyWindyDay`
    /// (`Chest.cs:1422`) — the Pin Wheel 4074.
    DayHappyWindyDay,
    /// W4-d122 shop 1: `BirthdayParty.PartyIsUp` (`Chest.cs:1435`) —
    /// the Party Bundle 5643.
    PartyIsUp,
    /// W4-d122 shop 2: `WorldGen.SavedOreTiers.Silver == 168`
    /// (`Chest.cs:1489`) — the Tungsten Bullet 4915 over the Silver
    /// Bullet 278 split inside the blood-moon/hardmode gate.
    SilverIsTungsten,
    /// W4-d122 shop 2: `NPC.downedBoss2` ALONE (`Chest.cs:1500`'s first
    /// conjunct — the Minishark gate reads it with `!dayTime`).
    DownedBoss2,
    /// W4-d122 shop 3: `!Main.remixWorld || Main.infectedSeed ||
    /// (Main.tenthAnniversaryWorld && !Main.getGoodWorld)`
    /// (`Chest.cs:1598`) — the Purification Powder 66 inside the
    /// NOT-blood-moon else branch (the infected-seed disjunct is the
    /// difference from [`StockWhen::NotRemixOrAnniversary`]).
    NotRemixOrInfectedOrAnniversary,
    /// W4-d122 shop 4: `(NPC.downedBoss1 || NPC.downedSlimeKing) &&
    /// !Main.dayTime` (`Chest.cs:1752`) — the Land Mine 5542 BETWEEN the
    /// Bomb 166 and the Dynamite 167 rows.
    Boss1OrSlimeKingNight,
    /// W4-d122 shop 4: `NPC.downedPirates` (`Chest.cs:1766`'s third
    /// conjunct — the Holy Hand Grenade gate reads it with hardmode +
    /// Plantera). The port's `World.downed_pirates` feeds it; no other
    /// shop case reads the flag alone yet.
    DownedPirates,
    /// W4-d122 shop 5: `NPC.downedClown` (`Chest.cs:1849`) — the clown
    /// outfit trio. The port persists no `downedClown` latch (the world
    /// file has no bit for it), so the flag rides the caller like
    /// `crimson_world` and stays false until one lands.
    DownedClown,
    /// W4-d122 shop 5: `NPC.downedAncientCultist` (`Chest.cs:1868`) —
    /// the cultist robe day/night split.
    DownedCultist,
    /// W4-d122 shop 5: `NPC.downedFrost` (`Chest.cs:1953`) — the Eskimo
    /// hood day/night split.
    DownedFrost,
    /// W4-d122 shop 5: `NPC.AnyNPCs(441)` (`Chest.cs:1881`) — the Tax
    /// Collector's presence opens the tam-o'-shanter trio.
    TaxCollectorPresent,
    /// W4-d122 shop 5: the EXACT moon phase (`Chest.cs:1824`/`:1836`/
    /// `:1910-1951`'s `moonPhase == n` — NOT the `/2` pairs).
    MoonPhaseExact(u32),
    /// W4-d122 shop 5: the `num < N` guard (`Chest.cs:1979`/`:1984`) —
    /// the row lands only while FEWER than N rows have resolved so far.
    /// The resolve loop counts the accepted rows and checks this BEFORE
    /// the child gate (the C# evaluates `num < 38` first).
    NumBelow(u32, &'static StockWhen),
}

/// `Chest.SetupShop`'s stock list for one shop index. The port carries the
/// Merchant's shop (index 1, `Chest.cs:1371-1483`); the other indexes land
/// with their NPCs (M4-TOWN umbrella). The order is the vanilla's `num`
/// order (the UI fills the slots left to right).
pub fn setup_shop(shop_index: u32, inventory_ids: &[u16], flags: &ShopFlags) -> Vec<ShopEntry> {
    let _ = inventory_ids;
    match shop_index {
        // The Merchant (`Chest.cs:1371-1483`, the registry
        // `NPCInteractions.Shop(17, 1)`, `NPCInteractions.cs:488`): every
        // row in the C#'s num order, every gate verbatim.
        1 => vec![
            row(88, StockWhen::Always),
            row(87, StockWhen::Always),
            row(35, StockWhen::Always),
            row(1991, StockWhen::Always),
            row(3509, StockWhen::Always),
            row(3506, StockWhen::Always),
            row(8, StockWhen::Always),
            // `:1387-1391`: `Main.notTheBeesWorld && !Main.remixWorld`.
            row(4388, StockWhen::NotTheBeesNotRemix),
            row(28, StockWhen::Always),
            row(188, StockWhen::HardMode), // `:1394-1398`
            row(110, StockWhen::Always),
            row(189, StockWhen::HardMode), // `:1401-1405`
            row(40, StockWhen::Always),
            row(42, StockWhen::Always),
            row(965, StockWhen::Always),
            row(967, StockWhen::ZoneSnow), // `:1412-1416`
            // `:1417-1421`: `ZoneJungle || (tenthAnniversaryWorld &&
            // notTheBeesWorld && !remixWorld)`.
            row(33, StockWhen::JungleOrBeesAnniversary),
            // `:1422-1425`: `Main.dayTime && Main.IsItAHappyWindyDay`.
            row(4074, StockWhen::DayHappyWindyDay),
            row(279, StockWhen::BloodMoon), // `:1426-1430`
            row(282, StockWhen::Night),     // `:1431-1434`
            // `:1435-1438`: `BirthdayParty.PartyIsUp`.
            row(5643, StockWhen::PartyIsUp),
            row(346, StockWhen::DownedBoss3), // `:1439-1443`
            row(488, StockWhen::HardMode),    // `:1444-1448`
            // `:1449-1459`: the 58-slot scan for the Flower Boots 930 →
            // the 931/1614 pair (BEFORE the 1786 row in the num order).
            row(931, StockWhen::PlayerHasItem(930)),
            row(1614, StockWhen::PlayerHasItem(930)),
            row(1786, StockWhen::Always),
            row(1348, StockWhen::HardMode), // `:1462-1466`
            row(3198, StockWhen::HardMode), // `:1467-1471`
            // `:1472-1476`: `downedBoss2 || downedBoss3 || hardMode`.
            row(4063, StockWhen::DownedBoss2Or3OrHard),
            row(4673, StockWhen::DownedBoss2Or3OrHard),
            // `:1477-1481`: `HasItem(3107)` — the Combat Wrench 3108.
            row(3108, StockWhen::PlayerHasItem(3107)),
        ],
        // The Arms Dealer (`Chest.cs:1484-1567`, the registry
        // `NPCInteractions.Shop(19, 2)`, `NPCInteractions.cs:489` —
        // `NPCID.ArmsDealer = 19`): the silver/tungsten bullet split, the
        // Minishark night-or-hardmode gate, the graveyard Quad-Barrel and
        // the hardmode shotgun trio — every gate verbatim.
        2 => vec![
            row(97, StockWhen::Always),
            // `:1487-1499`: `bloodMoon || hardMode` → `Silver == 168` ?
            // the Tungsten Bullet 4915 : the Silver Bullet 278.
            row(4915, ARMS_BULLET_TUNGSTEN),
            row(278, ARMS_BULLET_SILVER),
            // `:1500-1504`: `(downedBoss2 && !dayTime) || hardMode`.
            row(47, ARMS_MINISHARK),
            row(95, StockWhen::Always),
            row(98, StockWhen::Always),
            // `:1509-1512`: `ZoneGraveyard && NPC.downedBoss3`.
            row(4703, ARMS_GRAVEYARD_QUAD),
            row(324, StockWhen::Night),     // `:1513-1517` the Handgun
            row(534, StockWhen::HardMode),  // `:1518-1522` the Shotgun
            row(1432, StockWhen::HardMode), // `:1523-1527` the Clockwork Assault Rifle
            row(2177, StockWhen::HardMode), // `:1528-1532` the Illegal Gun Parts
            // `:1533-1557`: the five HasItem rows in the C#'s num order.
            row(1261, StockWhen::PlayerHasItem(1258)),
            row(1836, StockWhen::PlayerHasItem(1835)),
            row(3108, StockWhen::PlayerHasItem(3107)),
            row(1783, StockWhen::PlayerHasItem(1782)),
            row(1785, StockWhen::PlayerHasItem(1784)),
            // `:1558-1566`: `Main.halloween` — the undertaker outfit
            // (constants-off, no season system).
            row(1736, StockWhen::Halloween),
            row(1737, StockWhen::Halloween),
            row(1738, StockWhen::Halloween),
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
        // The Demolitionist (`Chest.cs:1746-1813`, the registry
        // `NPCInteractions.Shop(38, 4)`, `NPCInteractions.cs:491` —
        // `NPCID.Demolitionist = 38`): the night-gated Freeze Bomb BETWEEN
        // the Bomb and the Dynamite, the always-on Portable Kiln/Mitey
        // Titey pair, the hardmode rows and the four 58-slot bomb scans.
        4 => vec![
            row(168, StockWhen::Always), // `:1748` the Grenade
            row(166, StockWhen::Always), // `:1750` the Bomb
            // `:1752-1756`: `(downedBoss1 || downedSlimeKing) && !dayTime`
            // — the Freeze Bomb 5542 (`ItemID.cs:12767`).
            row(5542, StockWhen::Boss1OrSlimeKingNight),
            row(167, StockWhen::Always),   // `:1757` the Dynamite
            row(265, StockWhen::HardMode), // `:1759-1763` the Hellfire Arrow
            row(5481, StockWhen::Always),  // `:1764` the Portable Kiln
            row(5464, StockWhen::Always),  // `:1765` the Mitey Titey
            // `:1766-1770`: `hardMode && downedPlantBoss && downedPirates`
            // — the Land Mine 937 (`ItemID.cs:3557`).
            row(937, DEMO_LAND_MINE),
            row(1347, StockWhen::HardMode), // `:1771-1775` the Explosive Powder
            // `:1776-1811`: the four 58-slot scans, in the C#'s order —
            // the Dry 4827, Wet 4824, Lava 4825 and Honey 4826 bombs.
            row(4827, StockWhen::PlayerHasItem(4827)),
            row(4824, StockWhen::PlayerHasItem(4824)),
            row(4825, StockWhen::PlayerHasItem(4825)),
            row(4826, StockWhen::PlayerHasItem(4826)),
        ],
        // The Witch Doctor (`Chest.cs:2615-2672`): the Blowgun 1430 and the
        // Flask 986 always; the conditionals (the Queen Bee 2999, the
        // jungle 6147/1167, the night 1158, the hardmode 1159, the
        // Imbuing Station 1160?/1161, the 1339/1171/1162 rows) sit
        // constants-off until their flags land.
        16 => vec![row(1430, StockWhen::Always), row(986, StockWhen::Always)],
        // The Clothier (`Chest.cs:1814-1988`, the registry
        // `NPCInteractions.Shop(54, 5)`, `NPCInteractions.cs:492` —
        // `NPCID.Clothier = 54`): the day/night + moon-phase hats, the
        // clown/cultist/frost splits, the Tax Collector trio, the
        // graveyard set, the hardmode moon-phase costumes, the party
        // block and the two `num < 38` tail guards — every gate verbatim.
        5 => vec![
            row(254, StockWhen::Always), // `:1815`
            row(981, StockWhen::Always), // `:1817`
            // `:1819-1823`: `Main.dayTime` — the Plumber's Hat 242.
            row(242, StockWhen::Not(&StockWhen::Night)),
            // `:1824-1835`: `moonPhase == 0` — the Familiar set 245/246,
            // plus the night-only Wig pair 1288/1289.
            row(245, StockWhen::MoonPhaseExact(0)),
            row(246, StockWhen::MoonPhaseExact(0)),
            row(1288, CLOTHIER_PHASE0_NIGHT),
            row(1289, CLOTHIER_PHASE0_NIGHT),
            // `:1836-1842`'s else-if: `moonPhase == 1` — the The Doctor's
            // set 325/326 (the exact phases make the arms exclusive).
            row(325, StockWhen::MoonPhaseExact(1)),
            row(326, StockWhen::MoonPhaseExact(1)),
            row(269, StockWhen::Always), // `:1843-1848`
            row(270, StockWhen::Always),
            row(271, StockWhen::Always),
            // `:1849-1857`: `NPC.downedClown` — the clown outfit trio.
            row(503, StockWhen::DownedClown),
            row(504, StockWhen::DownedClown),
            row(505, StockWhen::DownedClown),
            // `:1858-1867`: `bloodMoon` — the Mime Mask 322, plus the
            // night-only Gem robe pair 3362/3363.
            row(322, StockWhen::BloodMoon),
            row(3362, CLOTHIER_BM_NIGHT),
            row(3363, CLOTHIER_BM_NIGHT),
            // `:1868-1880`: `NPC.downedAncientCultist` — the day
            // (2856/2858) / night (2857/2859) robe split.
            row(2856, CLOTHIER_CULTIST_DAY),
            row(2858, CLOTHIER_CULTIST_DAY),
            row(2857, CLOTHIER_CULTIST_NIGHT),
            row(2859, CLOTHIER_CULTIST_NIGHT),
            // `:1881-1886`: `NPC.AnyNPCs(441)` — the Tax Collector's
            // tam-o'-shanter trio 3242/3243/3244.
            row(3242, StockWhen::TaxCollectorPresent),
            row(3243, StockWhen::TaxCollectorPresent),
            row(3244, StockWhen::TaxCollectorPresent),
            // `:1887-1897`: the graveyard — the eight funeral rows.
            row(4685, StockWhen::ZoneGraveyard),
            row(4686, StockWhen::ZoneGraveyard),
            row(4704, StockWhen::ZoneGraveyard),
            row(4705, StockWhen::ZoneGraveyard),
            row(4706, StockWhen::ZoneGraveyard),
            row(4707, StockWhen::ZoneGraveyard),
            row(4708, StockWhen::ZoneGraveyard),
            row(4709, StockWhen::ZoneGraveyard),
            // `:1898-1902`: the snow biome — the Eskimo set 1429.
            row(1429, StockWhen::ZoneSnow),
            // `:1903-1907`: `Main.halloween` — the Bride of Frankenstein
            // mask 1740 (constants-off, no season system).
            row(1740, StockWhen::Halloween),
            // `:1908-1952`: hardmode — the EXACT-phase costume sets
            // (2 → 869, 3 → 4994/4997, 4 → 864/865, 5 → 4995/4998,
            // 6 → 873/874/875, 7 → 4996/4999).
            row(869, CLOTHIER_HM_PHASE2),
            row(4994, CLOTHIER_HM_PHASE3),
            row(4997, CLOTHIER_HM_PHASE3),
            row(864, CLOTHIER_HM_PHASE4),
            row(865, CLOTHIER_HM_PHASE4),
            row(4995, CLOTHIER_HM_PHASE5),
            row(4998, CLOTHIER_HM_PHASE5),
            row(873, CLOTHIER_HM_PHASE6),
            row(874, CLOTHIER_HM_PHASE6),
            row(875, CLOTHIER_HM_PHASE6),
            row(4996, CLOTHIER_HM_PHASE7),
            row(4999, CLOTHIER_HM_PHASE7),
            // `:1953-1965`: `NPC.downedFrost` — the day (1275) / night
            // (1276) hood split.
            row(1275, CLOTHIER_FROST_DAY),
            row(1276, CLOTHIER_FROST_NIGHT),
            // `:1966-1970`: `Main.halloween` — the Cat Ears/Bunny Hood
            // pair (constants-off).
            row(3246, StockWhen::Halloween),
            row(3247, StockWhen::Halloween),
            // `:1971-1978`: `BirthdayParty.PartyIsUp` — the party set.
            row(3730, StockWhen::PartyIsUp),
            row(3731, StockWhen::PartyIsUp),
            row(3733, StockWhen::PartyIsUp),
            row(3734, StockWhen::PartyIsUp),
            row(3735, StockWhen::PartyIsUp),
            // `:1979-1982`: `num < 38 && golferScoreAccumulated >= 2000`
            // — the Golf Ball painting 4744.
            row(4744, StockWhen::NumBelow(38, &CLOTHIER_GOLF_TAIL)),
            row(5308, StockWhen::Always), // `:1983`
            // `:1984-1987`: `num < 38` — the Pretty Mirror 5630.
            row(5630, StockWhen::NumBelow(38, &StockWhen::Always)),
        ],
        // The Dryad (`Chest.cs:1568-1745`, the registry
        // `NPCInteractions.Shop(20, 3)`, `NPCInteractions.cs:490` —
        // `NPCID.Dryad = 20`): the blood-moon evil-seed split, the
        // NOT-blood-moon powder else branch, the graveyard hardmode seed,
        // the underworld/glowshroom else-if, the crimson planter split and
        // the hardmode flower-seed TRIPLETS — every gate verbatim.
        3 => vec![
            // `:1569-1595`: `bloodMoon` → the crimson/corruption split;
            // the powder row (2886/67) rides the remix gate on top, the
            // seed + mushroom rows (2171/4508 vs 59/4504) are bare.
            row(2886, DRYAD_BM_CRIMSON_POWDER),
            row(2171, DRYAD_BM_CRIMSON),
            row(4508, DRYAD_BM_CRIMSON),
            row(67, DRYAD_BM_CORRUPT_POWDER),
            row(59, DRYAD_BM_CORRUPT),
            row(4504, DRYAD_BM_CORRUPT),
            // `:1596-1609`'s else branch: the Purification Powder 66 under
            // `!remix || infectedSeed || (anniversary && !getGood)`, then
            // the three bare rows.
            row(66, DRYAD_POWDER_ELSE),
            row(62, StockWhen::Not(&StockWhen::BloodMoon)),
            row(63, StockWhen::Not(&StockWhen::BloodMoon)),
            row(745, StockWhen::Not(&StockWhen::BloodMoon)),
            // `:1610-1621`: `hardMode && ZoneGraveyard` — the crimson world
            // sells the Corrupt Seeds 59, the corruption world the Crimson
            // Seeds 2171 (the C#'s flipped split is verbatim).
            row(59, DRYAD_GRAVE_CRIMSON),
            row(2171, DRYAD_GRAVE_CORRUPT),
            row(27, StockWhen::Always), // `:1622`
            row(5309, StockWhen::Always),
            row(114, StockWhen::Always),
            row(1828, StockWhen::Always),
            row(747, StockWhen::Always),
            row(746, StockWhen::HardMode),  // `:1632-1636`
            row(369, StockWhen::HardMode),  // `:1637-1641`
            row(4505, StockWhen::HardMode), // `:1642-1646`
            // `:1647-1654`: the underworld/glowshroom else-if — the
            // Hellcake 5214 wins over the Mushroom set 194.
            row(5214, StockWhen::ZoneUnderworld),
            row(194, DRYAD_GLOWSHROOM_ELSE),
            // `:1655-1661`: `Main.halloween` — the Goodie Bag pair
            // (constants-off, no season system).
            row(1853, StockWhen::Halloween),
            row(1854, StockWhen::Halloween),
            row(3215, StockWhen::Always), // `:1662-1675`'s planter block
            row(3216, StockWhen::Always),
            row(3219, StockWhen::Always),
            // `:1665-1672`: `WorldGen.crimson` → 3218, else 3217.
            row(3218, StockWhen::CrimsonWorld),
            row(3217, StockWhen::Not(&StockWhen::CrimsonWorld)),
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
            // `:1685-1710`: hardmode — the flower-seed TRIPLETS by
            // `moonPhase / 2` (the third row is the hardmode-only one).
            row(4430, DRYAD_FLOWER_PAIR0),
            row(4431, DRYAD_FLOWER_PAIR0),
            row(4432, DRYAD_FLOWER_TRIPLE0),
            row(4433, DRYAD_FLOWER_PAIR1),
            row(4434, DRYAD_FLOWER_PAIR1),
            row(4435, DRYAD_FLOWER_TRIPLE1),
            row(4436, DRYAD_FLOWER_PAIR2),
            row(4437, DRYAD_FLOWER_PAIR2),
            row(4438, DRYAD_FLOWER_TRIPLE2),
            // The `default:` arm — phases 6/7 (pair 3).
            row(4439, DRYAD_FLOWER_PAIR3),
            row(4440, DRYAD_FLOWER_PAIR3),
            row(4441, DRYAD_FLOWER_TRIPLE3),
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
        // Santa Claus's shop (index 9, `Chest.cs:2091-2111`, the registry
        // `NPCInteractions.Shop(142, 9)`, `NPCInteractions.cs:496` —
        // `NPCID.SantaClaus = 142`, `NPCID.cs:10961`): the six always rows
        // — the Santa set 588/589/590 + the coloured lights 597/598/596
        // (`ItemID.cs:2859-2879`) — then the `for (int m = 1873;
        // m < 1906; m++)` walk (`:2105-2109`): the 33 Christmas-decoration
        // rows 1873..=1905 (`ItemID.cs:5429-5493`), no gates at all.
        9 => {
            let mut rows = vec![
                row(588, StockWhen::Always),
                row(589, StockWhen::Always),
                row(590, StockWhen::Always),
                row(597, StockWhen::Always),
                row(598, StockWhen::Always),
                row(596, StockWhen::Always),
            ];
            // `:2105-2109`: `m < 1906` — the last decoration is 1905.
            for m in 1873u16..1906 {
                rows.push(row(m, StockWhen::Always));
            }
            rows
        }
        // The Truffle's shop (index 10, `Chest.cs:2112-2134`, the registry
        // `NPCInteractions.Shop(160, 10)`, `NPCInteractions.cs:497` —
        // `NPCID.Truffle = 160`, `NPCID.cs:10997`): the mech-any pair, the
        // Mushroom Cap, the Plantera Autohammer, the Strange Glowing
        // Mushroom, My Son and the remix-gated Dark Blue Solution.
        10 => vec![
            // `:2113-2119`: `if (NPC.downedMechBossAny)` — the Mushroom
            // Spear 756 + the Hammush 787 (NO hardmode conjunct).
            row(756, StockWhen::DownedMechAnyBare),
            row(787, StockWhen::DownedMechAnyBare),
            row(868, StockWhen::Always), // `:2120` the Mushroom Cap
            // `:2122-2126`: `if (NPC.downedPlantBoss)` — the Autohammer.
            row(1551, StockWhen::DownedPlantBoss),
            row(1181, StockWhen::Always), // `:2127` Strange Glowing Mushroom
            row(5231, StockWhen::Always), // `:2129` My Son
            // `:2130-2133`: `!Main.remixWorld || (Main.tenthAnniversaryWorld
            // && !Main.getGoodWorld)` — the Dark Blue Solution 783.
            row(783, StockWhen::NotRemixOrAnniversary),
        ],
        // The Steampunker's shop (index 11, `Chest.cs:2135-2245`, the
        // registry `NPCInteractions.Shop(178, 11)`, `NPCInteractions.cs:498`
        // — `NPCID.Steampunker = 178`, `NPCID.cs:11033`): the Clentaminator,
        // the Jetpack/outfit split, the logic gates, the crafting-station
        // biome rows and the solution if/else chain — every gate verbatim.
        11 => vec![
            // `:2137-2140`: the remix gate — the Clentaminator 779.
            row(779, StockWhen::NotRemixOrAnniversary),
            // `:2141-2150`: `moonPhase >= 4 && hardMode` → the Jetpack 748,
            // ELSE the steampunk outfit 839/840/841 (the NOT arm).
            row(748, STEAM_JETPACK),
            row(839, StockWhen::Not(&STEAM_JETPACK)),
            row(840, StockWhen::Not(&STEAM_JETPACK)),
            row(841, StockWhen::Not(&STEAM_JETPACK)),
            // `:2151-2154`: `NPC.downedGolemBoss` — the Steampunk Wings 948.
            row(948, StockWhen::DownedGolem),
            // `:2155-2158`: hardmode — the Static Hook 3623.
            row(3623, StockWhen::HardMode),
            // `:2159-2169`: the always rows in the C#'s num order — the
            // logic gates AND/OR/XOR/NAND/NOR/NXOR, the On lamp, the Off
            // lamp, the Faulty lamp, the two conveyor belts.
            row(3603, StockWhen::Always),
            row(3604, StockWhen::Always),
            row(3607, StockWhen::Always),
            row(3605, StockWhen::Always),
            row(3606, StockWhen::Always),
            row(3608, StockWhen::Always),
            row(3618, StockWhen::Always),
            row(3602, StockWhen::Always),
            row(3663, StockWhen::Always),
            row(3609, StockWhen::Always),
            row(3610, StockWhen::Always),
            // `:2170-2173`: `hardMode || !getGoodWorld` — Blend-O-Matic 995.
            row(995, StockWhen::HardModeOrNotGood),
            // `:2174-2177`: all three pre-hardmode bosses — the Steampunk
            // Boiler 2203.
            row(2203, StockWhen::DownedAllPreHardBosses),
            // `:2178-2185`: `WorldGen.crimson` → the Flesh Cloning Vaat
            // 2193, ELSE the Lesion Station 4142.
            row(2193, StockWhen::CrimsonWorld),
            row(4142, StockWhen::Not(&StockWhen::CrimsonWorld)),
            // `:2186-2189`: the graveyard — the Bone Welder 2192.
            row(2192, StockWhen::ZoneGraveyard),
            // `:2190-2194`: the jungle — the Honey Dispenser 2204.
            row(2204, StockWhen::ZoneJungle),
            // `:2195-2198`: `zoneJungle && downedGolemBoss` — the Lihzahrd
            // Furnace 2195.
            row(2195, STEAM_JUNGLE_GOLEM),
            // `:2199-2202`: the snow biome — the Ice Machine 2198.
            row(2198, StockWhen::ZoneSnow),
            // `:2203-2206`: the sky height — the Sky Mill 2197.
            row(2197, StockWhen::ZoneSkyHeight),
            // `:2207-2234`: the whole solution block rides the remix gate.
            // The if/else chain: `eclipse || bloodMoon` → the evil split
            // (crimson 784 / corrupt 782), ELSE `ZoneHallow` → 781, ELSE
            // the Green Solution 780.
            row(784, STEAM_SOLUTION_RED),
            row(782, STEAM_SOLUTION_PURPLE),
            row(781, STEAM_SOLUTION_BLUE),
            row(780, STEAM_SOLUTION_GREEN),
            // `:2228-2233`: `NPC.downedMoonlord` inside the same remix
            // block — the Sand/Snow/Dirt solutions 5392/5393/5394.
            row(5392, STEAM_MOONLORD_SOLUTIONS),
            row(5393, STEAM_MOONLORD_SOLUTIONS),
            row(5394, STEAM_MOONLORD_SOLUTIONS),
            // `:2235-2239`: hardmode — the Cog 1344 + the Steampunk
            // Minecart 4472.
            row(1344, StockWhen::HardMode),
            row(4472, StockWhen::HardMode),
            // `:2240-2243`: `Main.halloween` — the Steampunk Goggles 1742
            // (constants-off, no season system).
            row(1742, StockWhen::Halloween),
        ],
        // The Cyborg's shop (index 14, `Chest.cs:2360-2442`, the registry
        // `NPCInteractions.Shop(209, 14)`, `NPCInteractions.cs:501` —
        // `NPCID.Cyborg = 209`, `NPCID.cs:11095`): the rocket tiers, the
        // martian rows, the hardmode set, the graveyard pair, the halloween
        // outfit and the Portal-Gun-station inventory scan.
        14 => vec![
            row(771, StockWhen::Always),    // `:2361` Rocket I
            row(772, StockWhen::BloodMoon), // `:2363-2367` Rocket II
            // `:2368-2372`: `!Main.dayTime || Main.eclipse` — Rocket III.
            row(773, CYBORG_ROCKET_III),
            row(774, StockWhen::Eclipse), // `:2373-2377` Rocket IV
            // `:2378-2385`: `NPC.downedMartians` (bare) — the Cluster
            // Rocket I, and the II under `bloodMoon || eclipse` on top.
            row(4445, StockWhen::DownedMartiansBare),
            row(4446, CYBORG_CLUSTER_II),
            row(4459, StockWhen::HardMode), // `:2386-2390` Dry Rocket
            row(760, StockWhen::HardMode),  // `:2391-2395` Proximity Mine Launcher
            row(1346, StockWhen::HardMode), // `:2396-2400` Nanites
            // `:2401-2406`: hardmode — Jim's Drone Visor/Drone + the RC Car.
            row(5452, StockWhen::HardMode),
            row(5451, StockWhen::HardMode),
            row(5738, StockWhen::HardMode),
            row(5598, StockWhen::Always), // `:2407` CRT Monolith
            row(5599, StockWhen::Always), // `:2409` Retro Monolith
            // `:2411-2420`: the graveyard — Spectre Goggles + Echo Block.
            row(4409, StockWhen::ZoneGraveyard),
            row(4392, StockWhen::ZoneGraveyard),
            // `:2421-2429`: halloween — the cyborg outfit (constants-off).
            row(1743, StockWhen::Halloween),
            row(1744, StockWhen::Halloween),
            row(1745, StockWhen::Halloween),
            // `:2430-2434`: `NPC.downedMartians` again — the HiTek
            // Sunglasses + the Night Vision Helmet.
            row(2862, StockWhen::DownedMartiansBare),
            row(3109, StockWhen::DownedMartiansBare),
            // `:2435-2439`: `HasItem(3384) || HasItem(3664)` — the Portal
            // Gun Station 3664 (the inventory-scan gate).
            row(3664, CYBORG_PORTAL_STATION),
            row(5928, StockWhen::Always), // `:2440` Toy Breaker Block
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

// — W4-d122: the Steampunker's compound gates (`Chest.cs:2135-2245`),
// named once so the if/else arms share ONE condition object (the `Not`
// arms negate the exact same expression the C# `else` negates). —

/// `Chest.cs:2141`: `Main.moonPhase >= 4 && Main.hardMode` — the Jetpack.
const STEAM_JETPACK: StockWhen =
    StockWhen::All(&[StockWhen::MoonPhaseAtLeast(4), StockWhen::HardMode]);
/// `Chest.cs:2195`: `zoneJungle && NPC.downedGolemBoss` — Lihzahrd Furnace.
const STEAM_JUNGLE_GOLEM: StockWhen =
    StockWhen::All(&[StockWhen::ZoneJungle, StockWhen::DownedGolem]);
/// `Chest.cs:2207-2209`: the remix gate AND `(eclipse || bloodMoon)`.
const STEAM_SOLUTION_EVIL: StockWhen = StockWhen::All(&[
    StockWhen::NotRemixOrAnniversary,
    StockWhen::Any(&[StockWhen::Eclipse, StockWhen::BloodMoon]),
]);
/// `:2211-2218`'s crimson half — the Red Solution 784.
const STEAM_SOLUTION_RED: StockWhen =
    StockWhen::All(&[STEAM_SOLUTION_EVIL, StockWhen::CrimsonWorld]);
/// `:2215-2218`'s corruption half — the Purple Solution 782.
const STEAM_SOLUTION_PURPLE: StockWhen = StockWhen::All(&[
    STEAM_SOLUTION_EVIL,
    StockWhen::Not(&StockWhen::CrimsonWorld),
]);
/// `:2220-2223`: the remix gate AND NOT the evil branch AND `ZoneHallow`
/// — the Blue Solution 781.
const STEAM_SOLUTION_BLUE: StockWhen = StockWhen::All(&[
    StockWhen::NotRemixOrAnniversary,
    StockWhen::Not(&StockWhen::Any(&[StockWhen::Eclipse, StockWhen::BloodMoon])),
    StockWhen::ZoneHallow,
]);
/// `:2224-2227`: the final else — neither the evil branch nor the hallow
/// one — the Green Solution 780.
const STEAM_SOLUTION_GREEN: StockWhen = StockWhen::All(&[
    StockWhen::NotRemixOrAnniversary,
    StockWhen::Not(&StockWhen::Any(&[StockWhen::Eclipse, StockWhen::BloodMoon])),
    StockWhen::Not(&StockWhen::ZoneHallow),
]);
/// `:2228-2233`: the remix gate AND `NPC.downedMoonlord` — the solution trio.
const STEAM_MOONLORD_SOLUTIONS: StockWhen =
    StockWhen::All(&[StockWhen::NotRemixOrAnniversary, StockWhen::DownedMoonlord]);

// — W4-d122: the Cyborg's compound gates (`Chest.cs:2360-2442`). —

/// `Chest.cs:2368`: `!Main.dayTime || Main.eclipse` — the Rocket III 773
/// (`Night` is `!dayTime` on the flags).
const CYBORG_ROCKET_III: StockWhen = StockWhen::Any(&[StockWhen::Night, StockWhen::Eclipse]);
/// `Chest.cs:2378-2384`: `downedMartians && (bloodMoon || eclipse)` — the
/// Cluster Rocket II 4446.
const CYBORG_CLUSTER_II: StockWhen = StockWhen::All(&[
    StockWhen::DownedMartiansBare,
    StockWhen::Any(&[StockWhen::BloodMoon, StockWhen::Eclipse]),
]);
/// `Chest.cs:2435`: `HasItem(3384) || HasItem(3664)` — the Portal Gun
/// Station 3664 row.
const CYBORG_PORTAL_STATION: StockWhen = StockWhen::Any(&[
    StockWhen::PlayerHasItem(3384),
    StockWhen::PlayerHasItem(3664),
]);

// — W4-d122: the Arms Dealer's compound gates (`Chest.cs:1484-1567`). —

/// `Chest.cs:1487-1493`: `(bloodMoon || hardMode) && Silver == 168` —
/// the Tungsten Bullet 4915.
const ARMS_BULLET_TUNGSTEN: StockWhen = StockWhen::All(&[
    StockWhen::Any(&[StockWhen::BloodMoon, StockWhen::HardMode]),
    StockWhen::SilverIsTungsten,
]);
/// `:1494-1498`'s else arm — the Silver Bullet 278 (the SAME event gate,
/// the NOT of the ore comparison).
const ARMS_BULLET_SILVER: StockWhen = StockWhen::All(&[
    StockWhen::Any(&[StockWhen::BloodMoon, StockWhen::HardMode]),
    StockWhen::Not(&StockWhen::SilverIsTungsten),
]);
/// `Chest.cs:1500`: `(NPC.downedBoss2 && !Main.dayTime) || Main.hardMode`
/// — the Minishark 47.
const ARMS_MINISHARK: StockWhen = StockWhen::Any(&[
    StockWhen::All(&[StockWhen::DownedBoss2, StockWhen::Night]),
    StockWhen::HardMode,
]);
/// `Chest.cs:1509`: `ZoneGraveyard && NPC.downedBoss3` — the Quad-Barrel
/// Shotgun 4703.
const ARMS_GRAVEYARD_QUAD: StockWhen =
    StockWhen::All(&[StockWhen::ZoneGraveyard, StockWhen::DownedBoss3]);

// — W4-d122: the Dryad's compound gates (`Chest.cs:1568-1745`), named once
// so the if/else arms share ONE condition object. —

/// `Chest.cs:1571-1577`: `bloodMoon && crimson` + the remix gate — the
/// Crimson Powder 2886 (the corruption twin's gate is the NOT-crimson).
const DRYAD_BM_CRIMSON_POWDER: StockWhen = StockWhen::All(&[
    StockWhen::BloodMoon,
    StockWhen::CrimsonWorld,
    StockWhen::NotRemixOrAnniversary,
]);
/// `:1578-1581`: `bloodMoon && crimson` — the Crimson Seeds 2171 + the
/// Crimson Mushroom Grass Seeds 4508 (no remix conjunct).
const DRYAD_BM_CRIMSON: StockWhen =
    StockWhen::All(&[StockWhen::BloodMoon, StockWhen::CrimsonWorld]);
/// `:1585-1589`: the corruption half's powder — the Vile Powder 67.
const DRYAD_BM_CORRUPT_POWDER: StockWhen = StockWhen::All(&[
    StockWhen::BloodMoon,
    StockWhen::Not(&StockWhen::CrimsonWorld),
    StockWhen::NotRemixOrAnniversary,
]);
/// `:1590-1593`: the Corrupt Seeds 59 + the Corrupt Mushroom Grass
/// Seeds 4504.
const DRYAD_BM_CORRUPT: StockWhen = StockWhen::All(&[
    StockWhen::BloodMoon,
    StockWhen::Not(&StockWhen::CrimsonWorld),
]);
/// `:1596-1602`: the else branch's powder gate — `!bloodMoon &&
/// (!remix || infectedSeed || (anniversary && !getGood))`.
const DRYAD_POWDER_ELSE: StockWhen = StockWhen::All(&[
    StockWhen::Not(&StockWhen::BloodMoon),
    StockWhen::NotRemixOrInfectedOrAnniversary,
]);
/// `:1610-1615`: `hardMode && ZoneGraveyard && crimson` — the Corrupt
/// Seeds 59 (the C#'s flipped split is verbatim).
const DRYAD_GRAVE_CRIMSON: StockWhen = StockWhen::All(&[
    StockWhen::HardMode,
    StockWhen::ZoneGraveyard,
    StockWhen::CrimsonWorld,
]);
/// `:1616-1619`'s else arm — the Crimson Seeds 2171 in a corruption world.
const DRYAD_GRAVE_CORRUPT: StockWhen = StockWhen::All(&[
    StockWhen::HardMode,
    StockWhen::ZoneGraveyard,
    StockWhen::Not(&StockWhen::CrimsonWorld),
]);
/// `:1651-1654`: the glowshroom ELSE-IF — `!ZoneUnderworldHeight &&
/// ZoneGlowshroom` (the Mushroom set 194).
const DRYAD_GLOWSHROOM_ELSE: StockWhen = StockWhen::All(&[
    StockWhen::Not(&StockWhen::ZoneUnderworld),
    StockWhen::ZoneGlowshroom,
]);
/// `:1685-1732`: the flower-seed pairs — BOTH hardmode/non-hardmode
/// switches carry the same first two rows per `moonPhase / 2` pair, so
/// the pair rows are the moon gate alone; the THIRD row is hardmode-only.
const DRYAD_FLOWER_PAIR0: StockWhen = StockWhen::MoonPhasePair(0);
const DRYAD_FLOWER_PAIR1: StockWhen = StockWhen::MoonPhasePair(1);
const DRYAD_FLOWER_PAIR2: StockWhen = StockWhen::MoonPhasePair(2);
const DRYAD_FLOWER_PAIR3: StockWhen = StockWhen::MoonPhasePair(3);
const DRYAD_FLOWER_TRIPLE0: StockWhen =
    StockWhen::All(&[StockWhen::HardMode, StockWhen::MoonPhasePair(0)]);
const DRYAD_FLOWER_TRIPLE1: StockWhen =
    StockWhen::All(&[StockWhen::HardMode, StockWhen::MoonPhasePair(1)]);
const DRYAD_FLOWER_TRIPLE2: StockWhen =
    StockWhen::All(&[StockWhen::HardMode, StockWhen::MoonPhasePair(2)]);
const DRYAD_FLOWER_TRIPLE3: StockWhen =
    StockWhen::All(&[StockWhen::HardMode, StockWhen::MoonPhasePair(3)]);

// — W4-d122: the Demolitionist's compound gate (`Chest.cs:1766`). —

/// `Chest.cs:1766`: `Main.hardMode && NPC.downedPlantBoss &&
/// NPC.downedPirates` — the Land Mine 937.
const DEMO_LAND_MINE: StockWhen = StockWhen::All(&[
    StockWhen::HardMode,
    StockWhen::DownedPlantBoss,
    StockWhen::DownedPirates,
]);

// — W4-d122: the Clothier's compound gates (`Chest.cs:1814-1988`). —

/// `Chest.cs:1824-1834`: `moonPhase == 0 && !dayTime` — the Wig pair.
const CLOTHIER_PHASE0_NIGHT: StockWhen =
    StockWhen::All(&[StockWhen::MoonPhaseExact(0), StockWhen::Night]);
/// `Chest.cs:1858-1866`: `bloodMoon && !dayTime` — the Gem robe pair.
const CLOTHIER_BM_NIGHT: StockWhen =
    StockWhen::All(&[StockWhen::BloodMoon, StockWhen::Night]);
/// `Chest.cs:1868-1874`: `downedAncientCultist && dayTime` — the day robes.
const CLOTHIER_CULTIST_DAY: StockWhen = StockWhen::All(&[
    StockWhen::DownedCultist,
    StockWhen::Not(&StockWhen::Night),
]);
/// `:1875-1879`'s else arm — the night robes.
const CLOTHIER_CULTIST_NIGHT: StockWhen =
    StockWhen::All(&[StockWhen::DownedCultist, StockWhen::Night]);
/// `Chest.cs:1908-1951`: `hardMode && moonPhase == n` — the costume sets.
const CLOTHIER_HM_PHASE2: StockWhen =
    StockWhen::All(&[StockWhen::HardMode, StockWhen::MoonPhaseExact(2)]);
const CLOTHIER_HM_PHASE3: StockWhen =
    StockWhen::All(&[StockWhen::HardMode, StockWhen::MoonPhaseExact(3)]);
const CLOTHIER_HM_PHASE4: StockWhen =
    StockWhen::All(&[StockWhen::HardMode, StockWhen::MoonPhaseExact(4)]);
const CLOTHIER_HM_PHASE5: StockWhen =
    StockWhen::All(&[StockWhen::HardMode, StockWhen::MoonPhaseExact(5)]);
const CLOTHIER_HM_PHASE6: StockWhen =
    StockWhen::All(&[StockWhen::HardMode, StockWhen::MoonPhaseExact(6)]);
const CLOTHIER_HM_PHASE7: StockWhen =
    StockWhen::All(&[StockWhen::HardMode, StockWhen::MoonPhaseExact(7)]);
/// `Chest.cs:1953-1959`: `downedFrost && dayTime` — the day hood 1275.
const CLOTHIER_FROST_DAY: StockWhen =
    StockWhen::All(&[StockWhen::DownedFrost, StockWhen::Not(&StockWhen::Night)]);
/// `:1960-1964`'s else arm — the night hood 1276.
const CLOTHIER_FROST_NIGHT: StockWhen =
    StockWhen::All(&[StockWhen::DownedFrost, StockWhen::Night]);
/// `Chest.cs:1979`: `Main.LocalPlayer.golferScoreAccumulated >= 2000`
/// (the `num < 38` half rides the `NumBelow` wrapper).
const CLOTHIER_GOLF_TAIL: StockWhen = StockWhen::GolfScore(2000, true);

/// `Player.RollLuck` -> `Luck.RollLuck` (`Player.cs:18188-18191`,
/// `Luck.cs:5-16`): a hit is the returned roll landing on 0, so the base
/// chance is `1 / range` with the luck narrowing/widening the range.
fn roll_luck(luck: f32, range: i32, rng: &mut UnifiedRandom) -> i32 {
    // `Luck.RollLuck` (`Luck.cs:5-16`): the luck hit narrows (or widens) the
    // range, then the OUTER roll happens inside it -- `Next(Next(range/2,
    // range))`, two draws, the hit being the outer 0.
    if luck > 0.0 && rng.next_float() < luck {
        // `Next(Next(range/2, range))` — the inner draw picks the outer
        // range, the outer draw rolls in it (sequential, not nested borrows).
        let outer_range = rng.next_range(range / 2, range);
        return rng.next_max(outer_range);
    }
    if luck < 0.0 && rng.next_float() < -luck {
        let outer_range = rng.next_range(range, range * 2);
        return rng.next_max(outer_range);
    }
    rng.next_max(range)
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
pub fn setup_travel_shop(world: &World, luck: f32, rng: &mut UnifiedRandom) -> [u16; 40] {
    // `Main.travelShop` (`Main.cs:1779`) — the 40 slots start zeroed, which
    // shop index 19's walk reads as the empty ones.
    let mut shop = [0u16; 40];
    // `:1282`: the slot count `Main.rand.Next(4, 7)`.
    let mut num = rng.next_range(4, 7);
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
    rng: &mut UnifiedRandom,
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
        let pair: &[u16] = match rng.next_max(6) {
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
    rng: &mut UnifiedRandom,
) {
    let (r0, r1, r2, r3, r4, r5) = (
        rarity[0], rarity[1], rarity[2], rarity[3], rarity[4], rarity[5],
    );
    // `NPC.downedMechBossAny` (`NPC.cs:984`'s gates) — the port's three fields.
    let mech_any = world.downed_mech_boss1 || world.downed_mech_boss2 || world.downed_mech_boss3;
    let hit = |denom: u16, rng: &mut UnifiedRandom| roll_luck(luck, denom as i32, rng) == 0;
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
    if hit(r3, rng) && world.hard_mode && mech_any {
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
        *it = (2281 + rng.next_max(3)) as u16;
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
    rng: &mut UnifiedRandom,
) {
    let (r1, r2, r3) = (rarity[1], rarity[2], rarity[3]);
    let hit = |denom: u16, rng: &mut UnifiedRandom| roll_luck(luck, denom as i32, rng) == 0;
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
        prefix: 0,
        stack: 1, // `Item.SetDefaults`'s default stack (`Item.cs:49022`)
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
/// The moon-phase-0 view; the shop open resolves the live phase through
/// [`resolve_stock_entries_moon`].
#[cfg(test)]
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
    // The C#'s `num` — the count of the rows that RESOLVED so far; the
    // `num < 38` guards (`Chest.cs:1979`/`:1984`) read it live as the
    // case walks, so the port counts the accepted rows the same way.
    let mut num = 0u32;
    let mut out = Vec::new();
    for e in setup_shop(shop_index, inventory_ids, flags) {
        if stock_gate(&e.when, flags, inventory_ids, moon_phase, num) {
            num += 1;
            out.push(e);
        }
    }
    out
}

/// One `StockWhen` gate against the live state — the `SetupShop` if-arms'
/// conditions, verbatim. The combinators (`All`/`Any`/`Not`) evaluate their
/// children recursively so the C#'s compound `&&`/`||`/`else` chains keep
/// their exact shape (W4-d122). `num` is the resolved-row count so far —
/// only the `NumBelow` guards read it.
fn stock_gate(
    when: &StockWhen,
    flags: &ShopFlags,
    inventory_ids: &[u16],
    moon_phase: u32,
    num: u32,
) -> bool {
    match when {
        StockWhen::All(children) => children
            .iter()
            .all(|c| stock_gate(c, flags, inventory_ids, moon_phase, num)),
        StockWhen::Any(children) => children
            .iter()
            .any(|c| stock_gate(c, flags, inventory_ids, moon_phase, num)),
        StockWhen::Not(child) => !stock_gate(child, flags, inventory_ids, moon_phase, num),
        // `Chest.cs:1979`'s short-circuit order: `num < 38` FIRST.
        StockWhen::NumBelow(n, child) => {
            num < *n && stock_gate(child, flags, inventory_ids, moon_phase, num)
        }
        _ => match *when {
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
            StockWhen::VampirismNotInfected => flags.vampirism && !flags.infected_seed,
            StockWhen::VampirismInfected => {
                !flags.hard_mode && flags.vampirism && flags.infected_seed
            }
            StockWhen::VampirismInfectedCrimson => {
                !flags.hard_mode && flags.vampirism && flags.infected_seed && flags.crimson_world
            }
            StockWhen::VampirismInfectedCorrupt => {
                !flags.hard_mode && flags.vampirism && flags.infected_seed && !flags.crimson_world
            }
            StockWhen::ZoneUnderworld => flags.zone_underworld,
            StockWhen::ZoneGlowshroom => flags.zone_glowshroom,
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
            // `Chest.cs:3432-3434` (`Main.hardMode && NPC.downedMoonlord`).
            StockWhen::HardModeMoonlord => flags.hard_mode && flags.downed_moon_lord,
            // `Chest.cs:3428-3430` (`NPC.downedSlimeKing && downedQueenSlime`).
            StockWhen::DownedBothSlimes => flags.downed_slime_king && flags.downed_queen_slime,
            StockWhen::GolfScore(min, inclusive) => {
                if inclusive {
                    flags.golfer_score >= min
                } else {
                    flags.golfer_score > min
                }
            }
            // `Chest.cs:2113`: no hardmode conjunct — the bare flag.
            StockWhen::DownedMechAnyBare => flags.downed_mech_any,
            StockWhen::DownedPlantBoss => flags.downed_plant_boss,
            // `Chest.cs:2130` verbatim.
            StockWhen::NotRemixOrAnniversary => {
                !flags.remix_world || (flags.anniversary_world && !flags.get_good_world)
            }
            // `Chest.cs:2170` verbatim.
            StockWhen::HardModeOrNotGood => flags.hard_mode || !flags.get_good_world,
            StockWhen::Eclipse => flags.eclipse,
            StockWhen::DownedGolem => flags.downed_golem,
            // `Chest.cs:2174` verbatim.
            StockWhen::DownedAllPreHardBosses => {
                flags.downed_boss1 && flags.downed_boss2 && flags.downed_boss3
            }
            StockWhen::CrimsonWorld => flags.crimson_world,
            StockWhen::ZoneGraveyard => flags.zone_graveyard,
            StockWhen::ZoneSkyHeight => flags.zone_sky_height,
            StockWhen::ZoneHallow => flags.zone_hallow,
            StockWhen::DownedMoonlord => flags.downed_moon_lord,
            StockWhen::DownedMartiansBare => flags.downed_martians,
            // The season system is unported — the halloween rows stay
            // constants-off (the recorded deviation).
            StockWhen::Halloween => false,
            StockWhen::MoonPhaseAtLeast(n) => moon_phase >= n,
            // W4-d122 shop 1 (`Chest.cs:1387` verbatim).
            StockWhen::NotTheBeesNotRemix => flags.not_the_bees && !flags.remix_world,
            // `Chest.cs:1417` verbatim.
            StockWhen::JungleOrBeesAnniversary => {
                flags.zone_jungle
                    || (flags.anniversary_world && flags.not_the_bees && !flags.remix_world)
            }
            // `Chest.cs:1422` verbatim (`Night` is `!dayTime` on the flags).
            StockWhen::DayHappyWindyDay => !flags.night && flags.happy_windy_day,
            StockWhen::PartyIsUp => flags.party_is_up,
            // `Chest.cs:1489` verbatim — the rolled ore id comparison.
            StockWhen::SilverIsTungsten => flags.silver_ore_tier == 168,
            StockWhen::DownedBoss2 => flags.downed_boss2,
            // `Chest.cs:1598` verbatim — the infected-seed disjunct.
            StockWhen::NotRemixOrInfectedOrAnniversary => {
                !flags.remix_world
                    || flags.infected_seed
                    || (flags.anniversary_world && !flags.get_good_world)
            }
            // `Chest.cs:1752` verbatim.
            StockWhen::Boss1OrSlimeKingNight => {
                (flags.downed_boss1 || flags.downed_slime_king) && flags.night
            }
            StockWhen::DownedPirates => flags.downed_pirates,
            StockWhen::DownedClown => flags.downed_clown,
            StockWhen::DownedCultist => flags.downed_cultist,
            StockWhen::DownedFrost => flags.downed_frost,
            StockWhen::TaxCollectorPresent => flags.tax_collector_present,
            StockWhen::MoonPhaseExact(n) => moon_phase == n,
            // The combinators + the num guard are handled above — unreachable.
            StockWhen::All(_)
            | StockWhen::Any(_)
            | StockWhen::Not(_)
            | StockWhen::NumBelow(_, _) => unreachable!(),
        },
    }
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
/// `Main.cs:20454-20455`): buyOnce/isAShopItem rows show the buying value, every
/// other row shows the SELLING value — the two diverge whenever
/// `PriceAdjustment != 1` (the display quirk is verbatim). The shop grid
/// (S1-P-i44) replaced the text panel; the tooltip shop-lines wiring is the
/// recorded follow-up, so the helper stays pinned by its golden vectors.
#[allow(dead_code)]
pub(crate) fn display_price_row(
    entry: &ShopEntry,
    discount_available: bool,
    price_adjustment: f32,
) -> i64 {
    let (calc_for_selling, calc_for_buying) =
        row_prices(entry, discount_available, price_adjustment);
    if entry.buy_once || entry.is_a_shop_item {
        calc_for_buying
    } else {
        calc_for_selling
    }
}

/// The reforge cost (`Main.DrawInventory`'s inline block,
/// `Main.cs:41584-41591`): `value * stack` (long), ×0.8 behind the Discount
/// Card (`:41587-41590`, a DOUBLE multiply then a long truncation), × the
/// happiness `PriceAdjustment` (a FLOAT multiply then a long truncation,
/// `:41591`), then the integer `/3`. The vanilla `<1 → 1` clamp
/// (`num58`, `:41598-41601`) is the DISPLAY only — the actual charge at
/// `:41658` is the unclamped `num53` (`BuyItem(0)` always succeeds on a
/// value-0 item). `value` is the instance's CURRENT value — the kind's
/// defaults after the current prefix's ×value² (`Item.cs:614-615`), so the
/// cost scales with prefix quality; the reforge slot always holds exactly
/// one item (`ItemSlot.cs:821-836`'s transfer splits stacks), so `stack` is 1.
pub(crate) fn reforge_cost(value: i32, stack: u32, mood: f32, discount_available: bool) -> u64 {
    // `Main.cs:41586` — `long num53 = (long)value * (long)stack`.
    let mut num53 = value as i64 * stack as i64;
    // `:41587-41590` — `num53 = (long)((double)num53 * 0.8)` (truncate).
    if discount_available {
        num53 = (num53 as f64 * 0.8) as i64;
    }
    // `:41591` — `num53 = (long)((float)num53 * PriceAdjustment) / 3`: the
    // float multiply truncates to long FIRST, then the integer /3.
    num53 = (num53 as f32 * mood) as i64 / 3;
    num53.max(0) as u64
}

/// The sell transaction's payout (`Player.SellItem`,
/// `Player.cs:35168-35195`): `GetItemExpectedPrice`'s selling value ÷ 5,
/// min 1 — the vanilla's 20 % sale rate lives HERE, after the adjustment
/// divide (the port used to fold it into the helper in the wrong order) —
/// × the sold stack, plus the sellback correction: the first
/// `min(memo_amount, item.stack)` units refund at the price they were
/// BOUGHT for (`(-num2 + calcForBuying) * min`, `:35191-35195`; the memo
/// is `Main.shopSellbackHelper`, filled by every shop buy,
/// `ItemSlot.cs:2084-2087`). The `calcForSelling <= 0` refusal
/// (`:35170-35174`) returns 0 — the caller then runs the value-0 giveaway
/// arm (`ItemSlot.cs:1232-1238`). `stack` defaults to the whole instance
/// (`stack == -1`, `:35175-35178`); the caller prices the instance —
/// `prefix::prefixed_stats(kind, prefix).value`'s ×value².
pub fn sell_payout(
    instance_value: i32,
    buy_once: bool,
    is_a_shop_item: bool,
    discount_available: bool,
    price_adjustment: f32,
    stack: u32,
    sellback_amount: u32,
) -> i64 {
    let (calc_for_selling, calc_for_buying) = get_item_expected_price(
        instance_value,
        buy_once,
        is_a_shop_item,
        discount_available,
        price_adjustment,
    );
    if calc_for_selling <= 0 {
        return 0;
    }
    // `long num = calcForSelling / 5; if (num < 1) num = 1L;`
    let num2 = (calc_for_selling / 5).max(1);
    let mut num = num2 * stack as i64;
    // `:35191-35195` — `Math.Min(amount, item.stack)` (the WHOLE stack,
    // not the sold count).
    if sellback_amount > 0 {
        num += (-num2 + calc_for_buying) * sellback_amount.min(stack) as i64;
    }
    num
}

/// The sell transaction's per-item price — [`sell_payout`]'s plain
/// whole-stack, no-memo shape (the `Player.SellItem` price half).
pub fn sell_price(instance_value: i32, price_adjustment: f32) -> i64 {
    let payout = sell_payout(instance_value, false, false, false, price_adjustment, 1, 0);
    if payout <= 0 {
        return 0;
    }
    payout
}

/// The bulk-buy unit count (`ItemSlot.GetBulkBuyAmount`,
/// `ItemSlot.cs:3121-3129`): 10, clamped to a buyOnce row's remaining
/// stack.
pub fn bulk_buy_amount(entry: &ShopEntry) -> u32 {
    if entry.buy_once {
        10u32.min(entry.stack)
    } else {
        10
    }
}

/// The bulk-buy gate (`ItemSlot.CanBulkBuy`, `ItemSlot.cs:3131-3138`):
/// only isAShopItem/buyOnce rows bulk, and only while Shift is in use.
pub fn can_bulk_buy(entry: &ShopEntry, shift_in_use: bool) -> bool {
    (entry.is_a_shop_item || entry.buy_once) && shift_in_use
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

/// `Item.IsConsideredSameItemAsType` (`Item.cs:47851-47933`) — the
/// variant families the sellback memo matches across (the Rubblemakers,
/// the echo-chamber family, ...). The plain kinds fall to the default
/// `type == compareType` arm.
pub fn is_considered_same_item_as_type(kind: u16, compare: u16) -> bool {
    const GROUPS: &[&[u16]] = &[
        &[5324, 5329, 5330],
        &[4346, 5391],
        &[4131, 5325],
        &[6190, 6195],
        &[5323, 5455],
        &[4767, 5453],
        &[5309, 5454],
        &[5358, 5359, 5360, 5361, 5437],
        &[6168, 6169, 6193, 6194],
        &[2611, 5526],
    ];
    for g in GROUPS {
        if g.contains(&kind) {
            return g.contains(&compare);
        }
    }
    kind == compare
}

/// `Main.shopSellbackHelper` (`GameContent/ItemShopSellbackHelper.cs:5-73`):
/// the per-session memo of what the player BOUGHT from the current shop —
/// every plain-currency, non-buyOnce buy adds its unit (`ItemSlot.cs:
/// 2084-2087`), a sell refunds the memo'd units at their buy price
/// (`Player.cs:35191-35195`) and `Chest.AddItemToShop` removes them before
/// re-stocking (`Chest.cs:672-676`). `Main.UpdateOldNPCShop`
/// (`Main.cs:18204-18211`) clears the whole memo whenever the shop index
/// changes (the close included).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SellbackHelper {
    /// The `ItemMemo` list — `(type, prefix, stack)` (`:7-30`).
    memos: Vec<(u16, u16, u32)>,
}

impl SellbackHelper {
    fn matches(memo: &(u16, u16, u32), kind: u16, prefix: u16) -> bool {
        // `ItemMemo.Matches` (`:22-29`): the variant-family type match AND
        // the exact prefix.
        is_considered_same_item_as_type(kind, memo.0) && prefix == memo.1
    }

    /// `Add` (`:34-45`): the first matching memo grows, else a new one.
    pub fn add(&mut self, kind: u16, prefix: u16, stack: u32) {
        if let Some(m) = self.memos.iter_mut().find(|m| Self::matches(m, kind, prefix)) {
            m.2 += stack;
        } else {
            self.memos.push((kind, prefix, stack));
        }
    }

    /// `Clear` (`:47-50`).
    pub fn clear(&mut self) {
        self.memos.clear();
    }

    /// `GetAmount` (`:52-55`): the memo'd count, 0 without a memo.
    pub fn get_amount(&self, kind: u16, prefix: u16) -> u32 {
        self.memos
            .iter()
            .find(|m| Self::matches(m, kind, prefix))
            .map_or(0, |m| m.2)
    }

    /// `Remove` (`:57-72`): subtracts `stack` from the first matching memo
    /// and returns the amount actually removed (the memo's whole count
    /// when it depletes, else `stack`); 0 without a memo.
    pub fn remove(&mut self, kind: u16, prefix: u16, stack: u32) -> u32 {
        let Some(i) = self
            .memos
            .iter()
            .position(|m| Self::matches(m, kind, prefix))
        else {
            return 0;
        };
        let before = self.memos[i].2;
        // The C# int subtraction may go negative; the `<= 0` arm then
        // removes the memo and returns its whole pre-count.
        let after = before as i64 - stack as i64;
        if after <= 0 {
            self.memos.remove(i);
            before
        } else {
            self.memos[i].2 = after as u32;
            before - after as u32
        }
    }
}

/// `Chest.AddItemToShop` (`Chest.cs:670-706`): a sold stack re-enters the
/// shop as buyOnce stock. The sellback memo removes its own units first
/// (`:672-676` — a full re-sell of bought goods adds NOTHING back); the
/// remainder merges into the first stackable buyOnce row (`:678-690`,
/// `Item.CanStack` = same kind AND prefix, `Item.cs:49865-49871`) and
/// spills into the first air slot as a fresh buyOnce clone (`:691-705`).
/// The scan covers the first 39 of the shop's 40 slots (`Chest.cs:678`/
/// `:691`'s `i < 39` loops; the 40th slot never takes a sell); a full shop
/// drops the overflow, exactly like the C#.
pub fn add_item_to_shop(
    stock: &mut Vec<ShopEntry>,
    sellback: &mut SellbackHelper,
    item: &crate::item::ItemSlot,
) {
    let removed = sellback.remove(item.kind, item.prefix, item.count);
    if removed >= item.count {
        return; // `:673-676`
    }
    let mut num2 = item.count - removed;
    for e in stock.iter_mut().take(39) {
        if num2 == 0 {
            break;
        }
        if e.buy_once && e.item == item.kind && e.prefix == item.prefix {
            let max = crate::registry::max_stack(e.item);
            let num3 = num2.min(max.saturating_sub(e.stack));
            e.stack += num3;
            num2 -= num3;
        }
    }
    if num2 == 0 {
        return;
    }
    // `:691-705` — the first AIR slot takes the clone. A depleted buyOnce
    // row turns to air in place (`ItemSlot.cs:2092`'s `SetDefaults(0)`), so
    // the port reuses that gap before appending; the 39-slot window caps
    // both (the C# loops never reach slot 39).
    let clone = ShopEntry {
        item: item.kind,
        when: StockWhen::Always,
        shop_custom_price: None,
        buy_once: true,
        is_a_shop_item: false,
        prefix: item.prefix,
        stack: num2,
    };
    if let Some(e) = stock.iter_mut().take(39).find(|e| e.item == 0 || e.stack == 0) {
        *e = clone;
    } else if stock.len() < 39 {
        stock.push(clone);
    }
}

#[cfg(test)]
mod s1_f_16_tests {
    use super::*;

    #[test]
    fn the_princess_rank_gates_read_the_real_flags() {
        // `Chest.cs:3428-3434` (the Princess's shop, index 24): the 5266 row
        // needs `downedSlimeKing && downedQueenSlime`, the 5044 row
        // `hardMode && downedMoonlord` — the port's gates read the World
        // flags through `ShopFlags` (`from_world`) instead of the hard-false
        // stand-ins.
        let mut flags = ShopFlags::default();
        flags.hard_mode = true;
        flags.downed_moon_lord = true;
        let stock = resolve_stock(24, &flags, &[]);
        assert!(stock.contains(&5044), "the moonlord row shows: {:?}", stock);
        let mut flags = ShopFlags::default();
        flags.downed_slime_king = true;
        flags.downed_queen_slime = true;
        let stock = resolve_stock(24, &flags, &[]);
        assert!(
            stock.contains(&5266),
            "the both-slimes row shows: {:?}",
            stock
        );
        // The plain defaults keep both rows hidden.
        let stock = resolve_stock(24, &ShopFlags::default(), &[]);
        assert!(!stock.contains(&5044) && !stock.contains(&5266));
    }

    #[test]
    fn from_world_carries_the_downed_flags() {
        let mut w = World::new("t".into(), 64, 48);
        w.downed_slime_king = true;
        w.downed_queen_slime = true;
        w.downed_moon_lord = true;
        w.hard_mode = true;
        let f = ShopFlags::from_world(&w);
        assert!(f.downed_slime_king && f.downed_queen_slime && f.downed_moon_lord && f.hard_mode);
    }
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
        // `Main.cs:41584-41591`: the value × the stack (long), the Discount
        // Card ×0.8 (double, truncate), × the mood (float, truncate), then the
        // integer /3 — 2000 copper at mood 1 = 666, the happiness 1.25 =
        // 833; the <1 → 1 clamp is the display only, so a value-0 item
        // costs 0 and `BuyItem(0)` always succeeds.
        assert_eq!(reforge_cost(2000, 1, 1.0, false), 666);
        assert_eq!(reforge_cost(2000, 1, 1.25, false), 833);
        assert_eq!(reforge_cost(0, 1, 1.0, false), 0);
        assert_eq!(reforge_cost(1, 1, 1.0, false), 0);
    }

    #[test]
    fn the_reforge_cost_discount_card_arm() {
        // `Main.cs:41587-41590`: `num53 = (long)((double)num53 * 0.8)` runs
        // BEFORE the mood multiply. 2000 → 1600 → /3 = 533 (not 666); at
        // mood 1.25 the float multiply truncates 1600×1.25 = 2000 → /3 = 666.
        assert_eq!(reforge_cost(2000, 1, 1.0, true), 533);
        assert_eq!(reforge_cost(2000, 1, 1.25, true), 666);
        // The double truncation: 1×0.8 = 0.8 → (long)0 → /3 = 0.
        assert_eq!(reforge_cost(1, 1, 1.0, true), 0);
        // A value whose ×0.8 is not integral truncates toward zero:
        // 3×0.8 = 2.4 → 2 → /3 = 0; 5×0.8 = 4.0 → 4 → /3 = 1.
        assert_eq!(reforge_cost(3, 1, 1.0, true), 0);
        assert_eq!(reforge_cost(5, 1, 1.0, true), 1);
        // The stack multiplies first (long): 2000×2 = 4000 → ×0.8 = 3200 →
        // /3 = 1066.
        assert_eq!(reforge_cost(2000, 2, 1.0, true), 1066);
    }

    #[test]
    fn the_unknown_shop_index_is_empty() {
        // The unclaimed indexes carry nothing (the ported ones land with
        // their NPCs — 1..8, 15..19, 22).
        assert!(resolve_stock(0, &ShopFlags::default(), &[]).is_empty());
        assert!(resolve_stock(30, &ShopFlags::default(), &[]).is_empty());
    }

    /// W4-d122 shop 1 — the Merchant (`Chest.cs:1371-1483`, the registry
    /// `NPCInteractions.Shop(17, 1)`, `NPCInteractions.cs:488`): the FULL
    /// num order, every gate verbatim — the notTheBees block, the
    /// jungle-or-anniversary-bees arm, the happy-windy-day Pin Wheel, the
    /// party bundle, the Flower-Boots scan BEFORE the 1786 row and the
    /// HasItem(3107) tail. Written, not run (code-only).
    #[test]
    fn the_merchant_full_stock_pins_the_num_order_and_every_gate() {
        // Classic day, phase 0, no carried items: the always rows only.
        let base = resolve_stock(1, &ShopFlags::default(), &[]);
        assert_eq!(
            base,
            vec![88, 87, 35, 1991, 3509, 3506, 8, 28, 110, 40, 42, 965, 1786]
        );
        // `:1387`: notTheBees && !remix → 4388 right after the Torch 8.
        let bees = ShopFlags {
            not_the_bees: true,
            ..Default::default()
        };
        let b = resolve_stock(1, &bees, &[]);
        assert_eq!(&b[..9], &[88, 87, 35, 1991, 3509, 3506, 8, 4388, 28]);
        // remix ON kills the block again (`:1387`'s second conjunct).
        let bees_remix = ShopFlags {
            remix_world: true,
            ..bees
        };
        assert!(!resolve_stock(1, &bees_remix, &[]).contains(&4388));
        // `:1417`: the jungle alone opens 33; so does the anniversary +
        // bees + !remix trio WITHOUT the jungle.
        let jungle = ShopFlags {
            zone_jungle: true,
            ..Default::default()
        };
        assert!(resolve_stock(1, &jungle, &[]).contains(&33));
        let anni_bees = ShopFlags {
            anniversary_world: true,
            not_the_bees: true,
            ..Default::default()
        };
        assert!(resolve_stock(1, &anni_bees, &[]).contains(&33));
        let anni_bees_remix = ShopFlags {
            remix_world: true,
            ..anni_bees
        };
        assert!(!resolve_stock(1, &anni_bees_remix, &[]).contains(&33));
        // `:1422`: DAY + happy-windy-day → the Pin Wheel 4074; night kills it.
        let windy = ShopFlags {
            happy_windy_day: true,
            ..Default::default()
        };
        assert!(resolve_stock(1, &windy, &[]).contains(&4074));
        let windy_night = ShopFlags {
            night: true,
            ..windy
        };
        assert!(!resolve_stock(1, &windy_night, &[]).contains(&4074));
        // `:1435`: the party bundle sits between the Glowstick 282 and the
        // Skeletron row 346 in the num order.
        let party = ShopFlags {
            party_is_up: true,
            night: true,
            downed_boss3: true,
            ..Default::default()
        };
        let p = resolve_stock(1, &party, &[]);
        let i282 = p.iter().position(|v| *v == 282).unwrap();
        assert_eq!(p[i282 + 1], 5643, "the party row follows the Glowstick");
        assert_eq!(p[i282 + 2], 346);
        // The Flower-Boots scan rows sit BEFORE 1786 (`:1449-1461`).
        let boots = resolve_stock(1, &ShopFlags::default(), &[930]);
        let i931 = boots.iter().position(|v| *v == 931).unwrap();
        assert_eq!(boots[i931 + 1], 1614);
        assert_eq!(boots[i931 + 2], 1786, "the pair precedes the 1786 row");
        // The FULL pin: hardmode + night + blood moon + snow + jungle +
        // boss2/3 + windy + party + bees + both scan items.
        let full = ShopFlags {
            hard_mode: true,
            night: true,
            blood_moon: true,
            zone_snow: true,
            zone_jungle: true,
            downed_boss2: true,
            downed_boss3: true,
            not_the_bees: true,
            happy_windy_day: true,
            party_is_up: true,
            ..Default::default()
        };
        assert_eq!(
            resolve_stock(1, &full, &[930, 3107]),
            vec![
                88, 87, 35, 1991, 3509, 3506, 8, 4388, 28, 188, 110, 189, 40, 42, 965, 967, 33,
                279, 282, 5643, 346, 488, 931, 1614, 1786, 1348, 3198, 4063, 4673, 3108
            ],
            "the exact num order (`:1373-1481`; the day-gated 4074 stays out at night)"
        );
        // The day twin: the Pin Wheel enters at its num slot (`:1422`).
        let day = ShopFlags {
            night: false,
            ..full
        };
        let d = resolve_stock(1, &day, &[930, 3107]);
        let i33 = d.iter().position(|v| *v == 33).unwrap();
        assert_eq!(d[i33 + 1], 4074);
    }

    /// W4-d122 shop 2 — the Arms Dealer (`Chest.cs:1484-1567`, the registry
    /// `NPCInteractions.Shop(19, 2)`, `NPCInteractions.cs:489`): the
    /// silver/tungsten bullet split under the event gate, the Minishark's
    /// `(downedBoss2 && night) || hardMode` disjunction, the graveyard
    /// Quad-Barrel conjunct and the hardmode trio — verbatim. Written,
    /// not run (code-only).
    #[test]
    fn the_arms_dealer_stock_splits_the_bullets_and_the_minishark() {
        // Classic day: the three always rows only.
        let base = resolve_stock(2, &ShopFlags::default(), &[]);
        assert_eq!(base, vec![97, 95, 98]);
        // `:1487`: the blood moon opens the Silver Bullet 278 (tier 9).
        let bm = ShopFlags {
            blood_moon: true,
            ..Default::default()
        };
        assert_eq!(resolve_stock(2, &bm, &[]), vec![97, 278, 95, 98]);
        // The tungsten tier flips the split to 4915 (`:1489`).
        let tungsten = ShopFlags {
            silver_ore_tier: 168,
            ..bm
        };
        assert_eq!(resolve_stock(2, &tungsten, &[]), vec![97, 4915, 95, 98]);
        // Hardmode alone opens the same split PLUS the Minishark, the
        // Handgun stays night-gated, the shotgun trio enters (`:1518-1532`).
        let hm = ShopFlags {
            hard_mode: true,
            ..Default::default()
        };
        assert_eq!(
            resolve_stock(2, &hm, &[]),
            vec![97, 278, 47, 95, 98, 534, 1432, 2177]
        );
        // `:1500`: boss2 + NIGHT opens the Minishark without hardmode;
        // boss2 by DAY does not.
        let b2n = ShopFlags {
            downed_boss2: true,
            night: true,
            ..Default::default()
        };
        let n = resolve_stock(2, &b2n, &[]);
        assert_eq!(n, vec![97, 47, 95, 98, 324], "the night Handgun rides along");
        let b2d = ShopFlags {
            downed_boss2: true,
            ..Default::default()
        };
        assert!(!resolve_stock(2, &b2d, &[]).contains(&47));
        // `:1509`: the graveyard AND boss3 — neither alone opens 4703.
        let grave_only = ShopFlags {
            zone_graveyard: true,
            ..Default::default()
        };
        assert!(!resolve_stock(2, &grave_only, &[]).contains(&4703));
        let boss3_only = ShopFlags {
            downed_boss3: true,
            ..Default::default()
        };
        assert!(!resolve_stock(2, &boss3_only, &[]).contains(&4703));
        let quad = ShopFlags {
            downed_boss3: true,
            ..grave_only
        };
        let q = resolve_stock(2, &quad, &[]);
        assert_eq!(q, vec![97, 95, 98, 4703], "the Quad-Barrel after the 98 row");
        // The FULL pin: hardmode + night + blood moon + graveyard + boss3 +
        // tungsten + every carried scan item.
        let full = ShopFlags {
            hard_mode: true,
            night: true,
            blood_moon: true,
            zone_graveyard: true,
            downed_boss3: true,
            silver_ore_tier: 168,
            ..Default::default()
        };
        assert_eq!(
            resolve_stock(2, &full, &[1258, 1835, 3107, 1782, 1784]),
            vec![
                97, 4915, 47, 95, 98, 4703, 324, 534, 1432, 2177, 1261, 1836, 3108, 1783, 1785
            ],
            "the exact num order (`:1485-1557`; the halloween trio stays off)"
        );
    }

    /// W4-d122 shop 3 — the Dryad (`Chest.cs:1568-1745`, the registry
    /// `NPCInteractions.Shop(20, 3)`, `NPCInteractions.cs:490`): the
    /// blood-moon evil split with the remix-gated powder heads, the
    /// NOT-blood-moon else branch, the flipped graveyard seed split, the
    /// underworld/glowshroom else-if, the crimson planter split and the
    /// hardmode flower TRIPLETS — verbatim. Written, not run (code-only).
    #[test]
    fn the_dryad_full_stock_pins_the_evil_splits_and_the_flower_triplets() {
        // Classic day, phase 0, corruption world: the else branch + the
        // base rows + the 3217 planter + the pair-0 rows.
        let base = resolve_stock_moon(3, &ShopFlags::default(), &[], 0);
        assert_eq!(
            base,
            vec![
                66, 62, 63, 745, 27, 5309, 114, 1828, 747, 3215, 3216, 3219, 3217, 3220, 3221,
                3222, 4047, 4045, 4044, 4043, 4042, 4046, 4041, 4241, 4048, 4430, 4431
            ],
            "the exact num order (`:1598-1731`, no hardmode)"
        );
        // `:1569`: the blood moon REPLACES the else branch with the
        // corruption seeds (the powder rides the remix gate).
        let bm = ShopFlags {
            blood_moon: true,
            ..Default::default()
        };
        let b = resolve_stock_moon(3, &bm, &[], 0);
        assert_eq!(&b[..3], &[67, 59, 4504], "the corruption head rows");
        assert!(!b.contains(&66) && !b.contains(&62), "the else rows are gone");
        // The crimson world flips the split (`:1571-1581`).
        let crimson = ShopFlags {
            crimson_world: true,
            ..bm
        };
        let c = resolve_stock_moon(3, &crimson, &[], 0);
        assert_eq!(&c[..3], &[2886, 2171, 4508]);
        // `:1573`/`:1585`: remix ON kills ONLY the powder head.
        let remix = ShopFlags {
            remix_world: true,
            ..bm
        };
        let r = resolve_stock_moon(3, &remix, &[], 0);
        assert_eq!(&r[..2], &[59, 4504], "the seeds survive the remix gate");
        // `:1598`: remix ON also kills the else-branch powder — but the
        // infected seed brings it back (the middle disjunct).
        let remix_day = ShopFlags {
            remix_world: true,
            ..Default::default()
        };
        let rd = resolve_stock_moon(3, &remix_day, &[], 0);
        assert!(!rd.contains(&66) && rd.contains(&62), "the bare else rows stay");
        let infected = ShopFlags {
            infected_seed: true,
            ..remix_day
        };
        assert!(resolve_stock_moon(3, &infected, &[], 0).contains(&66));
        // `:1610`: hardmode + graveyard — the FLIPPED split (crimson world
        // sells 59, corruption world 2171).
        let grave = ShopFlags {
            hard_mode: true,
            zone_graveyard: true,
            ..Default::default()
        };
        let g = resolve_stock_moon(3, &grave, &[], 0);
        let i747 = g.iter().position(|v| *v == 747).unwrap();
        assert_eq!(g[i747 - 1], 2171, "the corruption world's grave seed");
        let grave_c = ShopFlags {
            crimson_world: true,
            ..grave
        };
        let gc = resolve_stock_moon(3, &grave_c, &[], 0);
        let i747c = gc.iter().position(|v| *v == 747).unwrap();
        assert_eq!(gc[i747c - 1], 59, "the crimson world's grave seed");
        // `:1647-1654`: the underworld wins over the glowshroom else-if.
        let under = ShopFlags {
            zone_underworld: true,
            zone_glowshroom: true,
            ..Default::default()
        };
        let u = resolve_stock_moon(3, &under, &[], 0);
        assert!(u.contains(&5214) && !u.contains(&194), "the else-if precedence");
        let glow = ShopFlags {
            zone_glowshroom: true,
            ..Default::default()
        };
        assert!(resolve_stock_moon(3, &glow, &[], 0).contains(&194));
        // `:1665`: the crimson planter split.
        let planter = ShopFlags {
            crimson_world: true,
            ..Default::default()
        };
        let pl = resolve_stock_moon(3, &planter, &[], 0);
        assert!(pl.contains(&3218) && !pl.contains(&3217));
        // `:1685-1710`: hardmode adds the THIRD flower row per pair; the
        // pairs themselves are hardmode-independent.
        let hm = ShopFlags {
            hard_mode: true,
            ..Default::default()
        };
        let h0 = resolve_stock_moon(3, &hm, &[], 0);
        assert_eq!(&h0[h0.len() - 3..], &[4430, 4431, 4432]);
        let h6 = resolve_stock_moon(3, &hm, &[], 6);
        assert_eq!(&h6[h6.len() - 3..], &[4439, 4440, 4441], "the default arm");
        let p6 = resolve_stock_moon(3, &ShopFlags::default(), &[], 6);
        assert_eq!(&p6[p6.len() - 2..], &[4439, 4440], "the non-hardmode pair");
        // The FULL pin: hardmode + blood moon + crimson + graveyard +
        // underworld + remix + anniversary (the powder survives via the
        // anniversary disjunct), phase 5 (pair 2).
        let full = ShopFlags {
            hard_mode: true,
            blood_moon: true,
            crimson_world: true,
            zone_graveyard: true,
            zone_underworld: true,
            remix_world: true,
            anniversary_world: true,
            ..Default::default()
        };
        assert_eq!(
            resolve_stock_moon(3, &full, &[], 5),
            vec![
                2886, 2171, 4508, 59, 27, 5309, 114, 1828, 747, 746, 369, 4505, 5214, 3215,
                3216, 3219, 3218, 3220, 3221, 3222, 4047, 4045, 4044, 4043, 4042, 4046, 4041,
                4241, 4048, 4436, 4437, 4438
            ],
            "the exact num order (`:1569-1731`; the halloween pair stays off)"
        );
    }

    /// W4-d122 shop 4 — the Demolitionist (`Chest.cs:1746-1813`, the
    /// registry `NPCInteractions.Shop(38, 4)`, `NPCInteractions.cs:491`):
    /// the Freeze Bomb's boss-or-slime-king night gate BETWEEN 166/167,
    /// the always-on Kiln/Mitey Titey pair, the Land Mine triple conjunct
    /// and the four bomb scans — verbatim. Written, not run (code-only).
    #[test]
    fn the_demolitionist_stock_gates_the_freeze_bomb_and_the_land_mine() {
        // Classic day: the five always rows.
        let base = resolve_stock(4, &ShopFlags::default(), &[]);
        assert_eq!(base, vec![168, 166, 167, 5481, 5464]);
        // `:1752`: boss1 + NIGHT opens the Freeze Bomb between 166 and 167;
        // the slime king is the other disjunct; day kills both.
        let b1n = ShopFlags {
            downed_boss1: true,
            night: true,
            ..Default::default()
        };
        assert_eq!(
            resolve_stock(4, &b1n, &[]),
            vec![168, 166, 5542, 167, 5481, 5464]
        );
        let skn = ShopFlags {
            downed_slime_king: true,
            night: true,
            ..Default::default()
        };
        assert!(resolve_stock(4, &skn, &[]).contains(&5542));
        let b1d = ShopFlags {
            downed_boss1: true,
            ..Default::default()
        };
        assert!(!resolve_stock(4, &b1d, &[]).contains(&5542));
        // `:1766`: the Land Mine needs ALL THREE — hardmode alone or
        // hardmode+Plantera do not open it.
        let hm = ShopFlags {
            hard_mode: true,
            ..Default::default()
        };
        let h = resolve_stock(4, &hm, &[]);
        assert_eq!(h, vec![168, 166, 167, 265, 5481, 5464, 1347]);
        let hmp = ShopFlags {
            downed_plant_boss: true,
            ..hm
        };
        assert!(!resolve_stock(4, &hmp, &[]).contains(&937));
        let full = ShopFlags {
            downed_pirates: true,
            ..hmp
        };
        let f = resolve_stock(4, &full, &[]);
        let i5464 = f.iter().position(|v| *v == 5464).unwrap();
        assert_eq!(f[i5464 + 1], 937, "the Land Mine follows the Mitey Titey");
        // The four scans ride the carried bombs in the C#'s order
        // (`:1776-1811`: 4827 first, then 4824/4825/4826).
        let scans = resolve_stock(4, &ShopFlags::default(), &[4826, 4824, 4827, 4825]);
        assert_eq!(
            &scans[scans.len() - 4..],
            &[4827, 4824, 4825, 4826],
            "the scan order is the C#'s, not the inventory's"
        );
        // The FULL pin: hardmode + plant + pirates + night + boss1 + every
        // carried bomb.
        let loud = ShopFlags {
            night: true,
            downed_boss1: true,
            ..full
        };
        assert_eq!(
            resolve_stock(4, &loud, &[4827, 4824, 4825, 4826]),
            vec![168, 166, 5542, 167, 265, 5481, 5464, 937, 1347, 4827, 4824, 4825, 4826],
            "the exact num order (`:1748-1811`)"
        );
    }

    /// W4-d122 shop 5 — the Clothier (`Chest.cs:1814-1988`, the registry
    /// `NPCInteractions.Shop(54, 5)`, `NPCInteractions.cs:492`): the
    /// day/night + exact-moon-phase hats, the cultist/frost day-night
    /// splits, the Tax Collector trio, the hardmode costume sets, the
    /// party block and the two `num < 38` tail guards — verbatim.
    /// Written, not run (code-only).
    #[test]
    fn the_clothier_full_stock_pins_the_splits_and_the_num_guards() {
        // Classic DAY, phase 0: the day hat + the Familiar set + the base
        // trio + the two tail rows (num stays well under 38).
        let base = resolve_stock_moon(5, &ShopFlags::default(), &[], 0);
        assert_eq!(
            base,
            vec![254, 981, 242, 245, 246, 269, 270, 271, 5308, 5630],
            "the exact num order (`:1815-1987`, day phase 0)"
        );
        // NIGHT phase 0: no day hat, the Wig pair enters.
        let night = ShopFlags {
            night: true,
            ..Default::default()
        };
        assert_eq!(
            resolve_stock_moon(5, &night, &[], 0),
            vec![254, 981, 245, 246, 1288, 1289, 269, 270, 271, 5308, 5630]
        );
        // Phase 1: the Doctor's set REPLACES the Familiar set (`:1836`).
        let p1 = resolve_stock_moon(5, &ShopFlags::default(), &[], 1);
        assert_eq!(
            p1,
            vec![254, 981, 242, 325, 326, 269, 270, 271, 5308, 5630]
        );
        // `:1849`: the clown latch opens the outfit trio after the 271 row.
        let clown = ShopFlags {
            downed_clown: true,
            ..Default::default()
        };
        let c = resolve_stock_moon(5, &clown, &[], 2);
        let i271 = c.iter().position(|v| *v == 271).unwrap();
        assert_eq!(&c[i271 + 1..i271 + 4], &[503, 504, 505]);
        // `:1858`: the blood moon opens the Mime Mask; night adds the pair.
        let bm = ShopFlags {
            blood_moon: true,
            night: true,
            ..Default::default()
        };
        let b = resolve_stock_moon(5, &bm, &[], 2);
        let i322 = b.iter().position(|v| *v == 322).unwrap();
        assert_eq!(&b[i322 + 1..i322 + 3], &[3362, 3363]);
        // `:1868`: the cultist day/night split.
        let cult_day = ShopFlags {
            downed_cultist: true,
            ..Default::default()
        };
        let cd = resolve_stock_moon(5, &cult_day, &[], 2);
        assert!(cd.contains(&2856) && cd.contains(&2858) && !cd.contains(&2857));
        let cult_night = ShopFlags {
            night: true,
            ..cult_day
        };
        let cn = resolve_stock_moon(5, &cult_night, &[], 2);
        assert!(cn.contains(&2857) && cn.contains(&2859) && !cn.contains(&2856));
        // `:1881`: the Tax Collector trio.
        let tax = ShopFlags {
            tax_collector_present: true,
            ..Default::default()
        };
        let t = resolve_stock_moon(5, &tax, &[], 2);
        assert!(t.contains(&3242) && t.contains(&3243) && t.contains(&3244));
        // `:1887`: the graveyard octet; `:1898`: the snow set.
        let grave = ShopFlags {
            zone_graveyard: true,
            zone_snow: true,
            ..Default::default()
        };
        let g = resolve_stock_moon(5, &grave, &[], 2);
        for id in [4685, 4686, 4704, 4705, 4706, 4707, 4708, 4709, 1429] {
            assert!(g.contains(&id), "the grave/snow row {id}");
        }
        // `:1908-1951`: the hardmode EXACT-phase costumes — phase 6 carries
        // the trio, phase 4 the pair, and the non-hardmode twins stay out.
        let hm = ShopFlags {
            hard_mode: true,
            ..Default::default()
        };
        let h6 = resolve_stock_moon(5, &hm, &[], 6);
        assert!(h6.contains(&873) && h6.contains(&874) && h6.contains(&875));
        assert!(!h6.contains(&864), "phase 4's set stays out");
        let p6 = resolve_stock_moon(5, &ShopFlags::default(), &[], 6);
        assert!(!p6.contains(&873), "no costume without hardmode");
        // `:1953`: the frost day/night split.
        let frost = ShopFlags {
            downed_frost: true,
            ..Default::default()
        };
        let fd = resolve_stock_moon(5, &frost, &[], 2);
        assert!(fd.contains(&1275) && !fd.contains(&1276));
        let fn_ = ShopFlags {
            night: true,
            ..frost
        };
        let fnn = resolve_stock_moon(5, &fn_, &[], 2);
        assert!(fnn.contains(&1276) && !fnn.contains(&1275));
        // The `num < 38` guards (`:1979`/`:1984`): the LOUD world at
        // NIGHT + phase 0 resolves 35 rows before the golf tail — 4744
        // lands (36), 5308 (37), then 5630's `num < 38` FAILS (num == 38)
        // and the mirror drops out.
        let loud = ShopFlags {
            night: true,
            blood_moon: true,
            hard_mode: true,
            downed_clown: true,
            downed_cultist: true,
            downed_frost: true,
            tax_collector_present: true,
            zone_graveyard: true,
            zone_snow: true,
            party_is_up: true,
            golfer_score: 2500,
            ..Default::default()
        };
        let l = resolve_stock_moon(5, &loud, &[], 0);
        assert_eq!(l.len(), 37, "the second guard drops the mirror");
        assert!(l.contains(&4744), "the golf tail had room (num 35 < 38)");
        assert!(!l.contains(&5630), "num == 38 blocks the mirror");
        assert_eq!(*l.last().unwrap(), 5308, "the unguarded row lands last");
        // Phase 7 resolves one costume PAIR instead of the four phase-0
        // rows (num 33 before the tail), so BOTH guarded rows land.
        let p7 = resolve_stock_moon(5, &loud, &[], 7);
        assert_eq!(p7.len(), 36);
        assert_eq!(&p7[p7.len() - 3..], &[4744, 5308, 5630]);
        // The golf gate itself: under 2000 the 4744 row stays out even
        // with room, and the mirror still lands (num 34 < 38).
        let poor = ShopFlags {
            golfer_score: 1999,
            ..loud
        };
        let pr = resolve_stock_moon(5, &poor, &[], 7);
        assert!(!pr.contains(&4744) && pr.contains(&5630));
    }

    /// `Chest.SetupTravelShop` (`Chest.cs:1275-1358`): the visit's stock —
    /// the seed-7 roll pins the whole ladder (the `Next(4, 7)` count, the
    /// luck rolls, the main loop, the painting tail) and the output proves
    /// the bundle rows travel with their heads (`_AddToShop`'s tails) and the
    /// duplicate suppression holds across all 40 slots.
    #[test]
    fn the_travel_stock_rolls_its_bundles_dup_free() {
        let w = World::new("t".into(), 200, 200);
        let stock = setup_travel_shop(&w, 0.0, &mut UnifiedRandom::new(7));
        // The seed-7 snapshot — 7 rows under the named stream (S1-A-11 P2:
        // the splitmix stream re-pinned to `UnifiedRandom`, and `roll_luck`
        // now ports `Luck.RollLuck`'s two-draw shape, `Luck.cs:5-16`).
        assert_eq!(
            &stock[..7],
            &[2274, 4420, 2281, 2266, 3118, 3099, 5636],
            "the generated rows"
        );
        // The zeros: the tail slots are the empty ones (`Main.travelShop`'s
        // zero fill) and no slot repeats an id.
        assert_eq!(stock[..7].iter().filter(|v| **v != 0).count(), 7);
        assert!(stock[7..].iter().all(|v| *v == 0), "the tail is empty");
        let seen: Vec<u16> = stock.iter().copied().filter(|v| *v != 0).collect();
        for (i, id) in seen.iter().enumerate() {
            assert!(!seen[i + 1..].contains(id), "the duplicate {id}");
        }
        // The bundles: every head's tails sit directly behind it (the 2260
        // -> 2261/2262 arm of `_AddToShop`).
        let hm = {
            let mut w2 = World::new("t".into(), 200, 200);
            w2.hard_mode = true;
            setup_travel_shop(&w2, 0.0, &mut UnifiedRandom::new(99))
        };
        assert_eq!(
            &hm[..9],
            &[4666, 4664, 4665, 4605, 1988, 3636, 3641, 4091, 5123],
            "the hardmode stock (S1-A-11 P2 re-pin)"
        );
        // The bundle structure survives the re-pin: the 4666 head carries its
        // 4664/4665 tails and the 3637 staff family's 3636/3641 pair rides
        // together (the `AddToShop` tail arms).
        // The hardmode arm's `minimumRarity = 2` tier — the first slot is a
        // tier-2+ row (4666 = r2, `Chest.cs:1170`), never a rarity-0/1-only
        // row.
        assert!(
            ![2266u16, 2242, 2260, 3637, 4420, 3119, 3118, 3099, 2216, 2217, 3624, 2273, 2274]
                .contains(&hm[0]),
            "the first slot is a tier-2+ row"
        );
        // The luck rows: the same seed at luck 0.3 — the four luck rolls
        // all miss on seed 7's prefix, so the stock stays at the plain
        // band's 7 (the widening mechanic itself is pinned directly on
        // `roll_luck` below).
        let lucky = setup_travel_shop(&w, 0.3, &mut UnifiedRandom::new(7));
        let lucky_rows = lucky.iter().filter(|v| **v != 0).count();
        assert_eq!(lucky_rows, 7, "seed 7's luck rolls all miss");
    }

    /// `Luck.RollLuck`'s two-draw hit path (`Luck.cs:5-16`), pinned directly:
    /// with a strong positive luck the seed-42 prefix hits on the first call
    /// — `NextFloat() < luck` (draw 1), the narrowed `Next(range/2, range)`
    /// (draw 2) and the outer `Next(...)` (draw 3), the hit being the outer
    /// 0. Exactly three draws (twin equality), and the pre-P2 shape (one
    /// draw returning the narrowed range) could never land this hit.
    #[test]
    fn the_roll_luck_hit_is_the_outer_zero_of_two_draws() {
        let mut rng = UnifiedRandom::new(42);
        assert_eq!(roll_luck(0.9, 4, &mut rng), 0, "the luck hit lands");
        let mut twin = UnifiedRandom::new(42);
        assert!(twin.next_float() < 0.9, "draw 1: the NextFloat gate");
        let outer_range = twin.next_range(4 / 2, 4);
        assert_eq!(outer_range, 2, "draw 2: the narrowed range");
        assert_eq!(twin.next_max(outer_range), 0, "draw 3: the outer hit");
        // ...and the base path (luck 0) is one plain `Next(range)` draw.
        let mut base = UnifiedRandom::new(42);
        let hit = roll_luck(0.0, 4, &mut base);
        let mut twin2 = UnifiedRandom::new(42);
        assert_eq!(hit, twin2.next_max(4), "the base roll is one Next(range)");
        assert_eq!(base, twin2, "exactly one draw on the base path");
    }

    /// `Chest.cs:2791-2810`'s shop index 19: the stock is the merchant's
    /// generated slots (not a fixed row list), the Palworld-collab rows ride
    /// the inventory gate and every row is `Always`.
    #[test]
    fn the_travel_shop_index_19_rides_the_generated_stock() {
        let stock = setup_travel_shop(
            &World::new("t".into(), 200, 200),
            0.0,
            &mut UnifiedRandom::new(7),
        );
        let flags = ShopFlags {
            travel_stock: stock,
            ..Default::default()
        };
        let rows = resolve_stock(19, &flags, &[]);
        assert_eq!(
            rows,
            vec![2274, 4420, 2281, 2266, 3118, 3099, 5636],
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
        assert_eq!(
            get_item_expected_price(5, false, false, false, 0.5),
            (10, 2)
        );
        assert_eq!(
            get_item_expected_price(15, false, false, false, 0.5),
            (30, 8)
        );
        // The buyOnce divide rounds first, then `/5`: 1/2f = 0.5f → 0
        // (even) → the `storeValue != 0` clamp lifts it to 1.
        assert_eq!(get_item_expected_price(1, true, false, false, 2.0), (0, 1));
    }

    #[test]
    fn the_discount_card_marks_the_buy_price_down() {
        // `Player.cs:35369-35378`: the 0.8 multiply TRUNCATES (the
        // `(int)` cast — no rounding), and only `isAShopItem` rows mirror
        // it into the sell value.
        assert_eq!(
            get_item_expected_price(40000, false, false, true, 1.0),
            (40000, 32000)
        );
        // (int)((float)7 * 0.8f) = (int)5.599999904632568f = 5 — a
        // rounding here would say 6.
        assert_eq!(get_item_expected_price(7, false, false, true, 1.0), (7, 5));
        assert_eq!(
            get_item_expected_price(40000, false, true, true, 1.0),
            (32000, 32000)
        );
    }

    #[test]
    fn the_buy_once_rows_price_at_a_fifth() {
        // `Player.cs:35381-35395`: a buyOnce row (the vanilla stamps it on
        // rows the player sold in, `Chest.cs:701`) buys at the store
        // value ÷ the adjustment ÷ 5 — the sell value keeps the plain
        // divide; the `< 1` clamp fires only for a non-zero store value
        // (`storeValue != 0`, `:35392-35394`).
        assert_eq!(
            get_item_expected_price(40000, true, false, false, 1.0),
            (40000, 8000)
        );
        // The round-then-divide order: 53333 / 5 = 10666 (the port used to
        // divide first and round — 10667).
        assert_eq!(
            get_item_expected_price(40000, true, false, false, 0.75),
            (53333, 10666)
        );
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
            prefix: 0,
            stack: 1,
        };
        assert_eq!(buy_price_row(&custom, false, 1.0), 50);
        assert_eq!(
            buy_price_row(&row(88, StockWhen::Always), false, 1.0),
            40000
        );
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
        assert!(
            entries
                .iter()
                .all(|e| e.shop_custom_price.is_none() && !e.buy_once && !e.is_a_shop_item)
        );
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
        assert!(
            seeded.contains(&4385),
            "the Corrupt Torch (no crimson flag)"
        );
        assert!(!seeded.contains(&4386), "the Crimson Torch stays out");
        // `WorldGen.crimson` flips the split.
        let crimson = ShopFlags {
            crimson_world: true,
            ..both
        };
        let rows = resolve_stock_moon(3, &crimson, &[], 0);
        assert!(
            rows.contains(&4386) && !rows.contains(&4385),
            "the crimson split"
        );
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
        assert!(
            girl.contains(&8),
            "the Bestiary Girl's vampirism torch: {girl:?}"
        );
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
        // The unknown index stays empty (Santa's 9 now carries stock).
        assert!(resolve_stock_moon(26, &ShopFlags::default(), &[], 0).is_empty());
    }

    /// W4-d122 shop 9 — Santa Claus (`Chest.cs:2091-2111`, the registry
    /// `NPCInteractions.Shop(142, 9)`, `NPCInteractions.cs:496`): the six
    /// head rows then the 1873..=1905 decoration walk, EVERY row
    /// unconditioned — the flags cannot change the stock. Written, not
    /// run (code-only).
    #[test]
    fn the_santa_stock_is_the_santa_heads_plus_the_decoration_walk() {
        let stock = resolve_stock(9, &ShopFlags::default(), &[]);
        let mut expect: Vec<u16> = vec![588, 589, 590, 597, 598, 596];
        expect.extend(1873u16..1906); // `:2105`'s `m < 1906` — 33 rows
        assert_eq!(stock, expect, "the exact num order (`:2093-2109`)");
        assert_eq!(stock.len(), 39);
        // The gates are all Always — hardmode/night/blood-moon change nothing.
        let loud = ShopFlags {
            hard_mode: true,
            night: true,
            blood_moon: true,
            ..Default::default()
        };
        assert_eq!(resolve_stock(9, &loud, &[]), expect);
    }

    /// W4-d122 shop 10 — the Truffle (`Chest.cs:2112-2134`,
    /// `NPCInteractions.Shop(160, 10)`): the verbatim gates — the bare
    /// `downedMechBossAny` pair (`:2113`, NO hardmode conjunct), the
    /// `downedPlantBoss` Autohammer (`:2122`) and the remix/anniversary/
    /// getGoodWorld solution gate (`:2130`). Written, not run (code-only).
    #[test]
    fn the_truffle_stock_gates_on_mech_plantera_and_the_remix_trio() {
        // Classic world: 868/1181/5231/783 — the pair and the hammer stay out.
        let base = resolve_stock(10, &ShopFlags::default(), &[]);
        assert_eq!(base, vec![868, 1181, 5231, 783]);
        // The bare mech flag alone opens the pair (no hardmode needed —
        // the C# `:2113` reads `NPC.downedMechBossAny` directly).
        let mech = ShopFlags {
            downed_mech_any: true,
            ..Default::default()
        };
        assert_eq!(
            resolve_stock(10, &mech, &[]),
            vec![756, 787, 868, 1181, 5231, 783]
        );
        // Plantera adds the Autohammer in its num slot (after 868).
        let plant = ShopFlags {
            downed_plant_boss: true,
            ..mech
        };
        assert_eq!(
            resolve_stock(10, &plant, &[]),
            vec![756, 787, 868, 1551, 1181, 5231, 783]
        );
        // `:2130`: remix ON kills 783; anniversary+remix (not getGood)
        // brings it back; getGood on top kills it again.
        let remix = ShopFlags {
            remix_world: true,
            ..plant
        };
        assert!(!resolve_stock(10, &remix, &[]).contains(&783));
        let anni = ShopFlags {
            anniversary_world: true,
            ..remix
        };
        assert!(resolve_stock(10, &anni, &[]).contains(&783));
        let good = ShopFlags {
            get_good_world: true,
            ..anni
        };
        assert!(!resolve_stock(10, &good, &[]).contains(&783));
    }

    /// W4-d122 shop 11 — the Steampunker (`Chest.cs:2135-2245`,
    /// `NPCInteractions.Shop(178, 11)`): the Jetpack/outfit moon-phase
    /// split, the solution if/else chain and the crafting-station biome
    /// rows, verbatim. Written, not run (code-only).
    #[test]
    fn the_steampunker_stock_splits_the_jetpack_and_the_solutions() {
        // Classic day, phase 0: the outfit trio (the NOT-Jetpack arm), the
        // logic-gate block, the Blend-O-Matic (not-getGood), the Lesion
        // Station (not-crimson) and the Green Solution (the final else).
        let base = resolve_stock_moon(11, &ShopFlags::default(), &[], 0);
        let mut expect = vec![779u16, 839, 840, 841];
        expect.extend([
            3603, 3604, 3607, 3605, 3606, 3608, 3618, 3602, 3663, 3609, 3610, 995, 4142, 780,
        ]);
        assert_eq!(base, expect, "the exact num order");
        // Phase 4 + hardmode: the Jetpack REPLACES the outfit trio in its
        // num slot; hardmode adds the Static Hook + the Cog + the minecart.
        let jet = ShopFlags {
            hard_mode: true,
            ..Default::default()
        };
        let p4 = resolve_stock_moon(11, &jet, &[], 4);
        assert_eq!(&p4[..2], &[779, 748], "the Jetpack takes the slot");
        assert!(!p4.contains(&839) && !p4.contains(&840) && !p4.contains(&841));
        assert!(p4.contains(&3623) && p4.contains(&1344) && p4.contains(&4472));
        // The same hardmode at phase 3 keeps the outfit (`moonPhase >= 4`).
        let p3 = resolve_stock_moon(11, &jet, &[], 3);
        assert!(p3.contains(&839) && !p3.contains(&748));
        // The evil branch: blood moon + corruption → 782 (not 780/781);
        // crimson flips it to 784 (and the Vaat replaces the Lesion Station).
        let bm = ShopFlags {
            blood_moon: true,
            ..Default::default()
        };
        let evil = resolve_stock_moon(11, &bm, &[], 0);
        assert!(evil.contains(&782) && !evil.contains(&780) && !evil.contains(&781));
        let crimson = ShopFlags {
            crimson_world: true,
            ..bm
        };
        let red = resolve_stock_moon(11, &crimson, &[], 0);
        assert!(red.contains(&784) && red.contains(&2193) && !red.contains(&4142));
        // The eclipse alone opens the same evil branch (`:2209`'s ||).
        let ecl = ShopFlags {
            eclipse: true,
            ..Default::default()
        };
        assert!(resolve_stock_moon(11, &ecl, &[], 0).contains(&782));
        // The hallow else-if: no evil event → the Blue Solution 781.
        let hallow = ShopFlags {
            zone_hallow: true,
            ..Default::default()
        };
        let blue = resolve_stock_moon(11, &hallow, &[], 0);
        assert!(blue.contains(&781) && !blue.contains(&780));
        // The evil event WINS over the hallow (the if precedes the else-if).
        let both = ShopFlags {
            blood_moon: true,
            ..hallow
        };
        let won = resolve_stock_moon(11, &both, &[], 0);
        assert!(won.contains(&782) && !won.contains(&781));
        // The Moon Lord trio rides the remix gate + the bare downed flag.
        let ml = ShopFlags {
            downed_moon_lord: true,
            ..Default::default()
        };
        let trio = resolve_stock_moon(11, &ml, &[], 0);
        for id in [5392u16, 5393, 5394] {
            assert!(trio.contains(&id), "the moonlord solution {id}");
        }
        let remix = ShopFlags {
            remix_world: true,
            ..ml
        };
        let cut = resolve_stock_moon(11, &remix, &[], 0);
        assert!(!cut.contains(&779) && !cut.contains(&780) && !cut.contains(&5392));
        // The station rows: all-three-bosses Boiler, jungle Dispenser,
        // jungle+Golem Furnace, snow Ice Machine, sky Mill, graveyard Welder.
        let stations = ShopFlags {
            downed_boss1: true,
            downed_boss2: true,
            downed_boss3: true,
            downed_golem: true,
            zone_jungle: true,
            zone_snow: true,
            zone_sky_height: true,
            zone_graveyard: true,
            ..Default::default()
        };
        let st = resolve_stock_moon(11, &stations, &[], 0);
        for id in [2203u16, 2204, 2195, 2198, 2197, 2192, 948] {
            assert!(st.contains(&id), "the station row {id}");
        }
        // The Golem alone (no jungle) gives the Wings but NOT the Furnace.
        let golem_only = ShopFlags {
            downed_golem: true,
            ..Default::default()
        };
        let go = resolve_stock_moon(11, &golem_only, &[], 0);
        assert!(go.contains(&948) && !go.contains(&2195));
        // getGoodWorld (pre-hardmode) cuts the Blend-O-Matic (`:2170`).
        let ftw = ShopFlags {
            get_good_world: true,
            ..Default::default()
        };
        assert!(!resolve_stock_moon(11, &ftw, &[], 0).contains(&995));
    }

    /// W4-d122 shop 14 — the Cyborg (`Chest.cs:2360-2442`,
    /// `NPCInteractions.Shop(209, 14)`): the rocket-tier ladder, the bare
    /// martian gates, the graveyard pair and the Portal-Gun-station scan
    /// (`Player.HasItem` — the port's inventory-ids view, the same seam
    /// the Merchant's Flower Boots rows use). Written, not run (code-only).
    #[test]
    fn the_cyborg_stock_ladders_the_rockets_and_the_martian_rows() {
        // Classic day: Rocket I + the two monoliths + the Toy Breaker Block.
        let base = resolve_stock(14, &ShopFlags::default(), &[]);
        assert_eq!(base, vec![771, 5598, 5599, 5928]);
        // Night opens Rocket III (`!dayTime || eclipse`); the blood moon
        // opens II (and III through the night half).
        let night = ShopFlags {
            night: true,
            ..Default::default()
        };
        assert_eq!(
            resolve_stock(14, &night, &[]),
            vec![771, 773, 5598, 5599, 5928]
        );
        let bm = ShopFlags {
            blood_moon: true,
            night: true,
            ..Default::default()
        };
        let bmstock = resolve_stock(14, &bm, &[]);
        assert!(bmstock.contains(&772) && bmstock.contains(&773));
        // The DAYTIME eclipse still opens III and IV (`:2368`'s || arm).
        let ecl = ShopFlags {
            eclipse: true,
            ..Default::default()
        };
        let est = resolve_stock(14, &ecl, &[]);
        assert!(est.contains(&773) && est.contains(&774) && !est.contains(&772));
        // Bare martians (NO hardmode): Cluster I + the HiTek/NightVision
        // pair; the Cluster II needs the blood-moon/eclipse conjunct.
        let mart = ShopFlags {
            downed_martians: true,
            ..Default::default()
        };
        let m = resolve_stock(14, &mart, &[]);
        assert!(m.contains(&4445) && m.contains(&2862) && m.contains(&3109));
        assert!(!m.contains(&4446), "the Cluster II needs the event");
        let mart_bm = ShopFlags {
            blood_moon: true,
            ..mart
        };
        assert!(resolve_stock(14, &mart_bm, &[]).contains(&4446));
        // Hardmode adds the Dry Rocket / launcher / Nanites / Jim's set /
        // the RC car in their num order.
        let hm = ShopFlags {
            hard_mode: true,
            ..Default::default()
        };
        let h = resolve_stock(14, &hm, &[]);
        assert_eq!(
            h,
            vec![771, 4459, 760, 1346, 5452, 5451, 5738, 5598, 5599, 5928]
        );
        // The graveyard pair; the Portal Gun Station needs either carried
        // item (3384 or 3664).
        let grave = ShopFlags {
            zone_graveyard: true,
            ..Default::default()
        };
        let g = resolve_stock(14, &grave, &[3384]);
        assert!(g.contains(&4409) && g.contains(&4392) && g.contains(&3664));
        let g2 = resolve_stock(14, &grave, &[3664]);
        assert!(g2.contains(&3664), "either scan id opens the station");
        assert!(!resolve_stock(14, &grave, &[]).contains(&3664));
        // The full-order pin: hardmode + martians + blood moon + night +
        // graveyard + the portal gun carried.
        let full = ShopFlags {
            hard_mode: true,
            downed_martians: true,
            blood_moon: true,
            night: true,
            zone_graveyard: true,
            ..Default::default()
        };
        assert_eq!(
            resolve_stock(14, &full, &[3384]),
            vec![
                771, 772, 773, 4445, 4446, 4459, 760, 1346, 5452, 5451, 5738, 5598, 5599, 4409,
                4392, 2862, 3109, 3664, 5928
            ],
            "the exact num order (`:2361-2441`; the halloween trio stays off)"
        );
    }
}

/// S1-P-i44 — the shop TRANSACTION witnesses: the sell payout's stack +
/// sellback shape (`Player.SellItem`, `Player.cs:35184-35195`), the
/// sellback memo (`ItemShopSellbackHelper.cs`), the sold-in restock
/// (`Chest.AddItemToShop`, `Chest.cs:670-706`) and the bulk-buy gates
/// (`ItemSlot.cs:3121-3138`). Written, not run (the code-only swarm).
#[cfg(test)]
mod s1_p_i44_tests {
    use super::*;
    use crate::item::ItemSlot;

    fn held(kind: u16, count: u32, prefix: u16) -> ItemSlot {
        ItemSlot {
            kind,
            name: crate::registry::name(kind),
            count,
            prefix,
        }
    }

    #[test]
    fn the_sell_payout_multiplies_the_stack_and_refunds_the_memo() {
        // `Player.cs:35184-35190`: 40000 @ mood 1 → 8000 per unit, × the
        // sold stack.
        assert_eq!(sell_payout(40000, false, false, false, 1.0, 3, 0), 24000);
        // `:35191-35195`: the first min(memo, stack) units refund at the
        // BUY price — 2 memo'd units of a 40000-value item pay 40000 each
        // instead of 8000: 3×8000 + (40000−8000)×2 = 88000.
        assert_eq!(sell_payout(40000, false, false, false, 1.0, 3, 2), 88000);
        // The memo clamps at the sold stack (`Math.Min(amount, item.stack)`).
        assert_eq!(sell_payout(40000, false, false, false, 1.0, 1, 9), 40000);
        // The mood rides BOTH halves: @0.75 the sell is 10666, the buy
        // 30000 (`Player.cs:35387-35388`): 2×10666 + (30000−10666)×1.
        assert_eq!(sell_payout(40000, false, false, false, 0.75, 2, 1), 40666);
        // The ≤0 refusal (`:35170-35174`) — a value-0 item pays nothing.
        assert_eq!(sell_payout(0, false, false, false, 1.0, 5, 0), 0);
        // The min-1 clamp before the stack multiply (`:35185-35188`).
        assert_eq!(sell_payout(1, false, false, false, 1.0, 4, 0), 4);
    }

    #[test]
    fn the_sellback_memo_adds_gets_and_removes_by_kind_and_prefix() {
        // `ItemShopSellbackHelper.cs:34-72`.
        let mut h = SellbackHelper::default();
        assert_eq!(h.get_amount(8, 0), 0);
        assert_eq!(h.remove(8, 0, 3), 0, "no memo, nothing removed");
        h.add(8, 0, 3);
        h.add(8, 0, 2); // the matching memo grows (`:36-40`)
        assert_eq!(h.get_amount(8, 0), 5);
        // The prefix is part of the match (`ItemMemo.Matches`, `:22-29`).
        h.add(8, 5, 1);
        assert_eq!(h.get_amount(8, 5), 1);
        assert_eq!(h.get_amount(8, 0), 5);
        // A partial remove returns the removed count (`:71`).
        assert_eq!(h.remove(8, 0, 2), 2);
        assert_eq!(h.get_amount(8, 0), 3);
        // An over-remove depletes the memo and returns its WHOLE pre-count
        // (`:66-70`), not the requested stack.
        assert_eq!(h.remove(8, 0, 9), 3);
        assert_eq!(h.get_amount(8, 0), 0);
        // Clear empties every memo (`:47-50`).
        h.clear();
        assert_eq!(h.get_amount(8, 5), 0);
    }

    #[test]
    fn the_sellback_memo_matches_the_variant_families() {
        // `Item.IsConsideredSameItemAsType` (`Item.cs:47851-47933`): the
        // Rubblemaker family (5324/5329/5330) matches across kinds.
        let mut h = SellbackHelper::default();
        h.add(5324, 0, 2);
        assert_eq!(h.get_amount(5329, 0), 2, "the family match");
        assert_eq!(h.get_amount(5330, 0), 2);
        assert_eq!(h.get_amount(8, 0), 0, "an alien kind stays out");
        assert!(is_considered_same_item_as_type(4346, 5391));
        assert!(!is_considered_same_item_as_type(4346, 5324));
        assert!(is_considered_same_item_as_type(8, 8));
    }

    #[test]
    fn the_sold_stack_restocks_as_buy_once_and_merges() {
        // `Chest.cs:691-705`: the first air slot takes the clone — buyOnce,
        // the sold stack, the instance's prefix.
        let mut stock = vec![row(88, StockWhen::Always)];
        let mut memo = SellbackHelper::default();
        add_item_to_shop(&mut stock, &mut memo, &held(9, 5, 0));
        assert_eq!(stock.len(), 2);
        let e = stock[1];
        assert_eq!((e.item, e.stack, e.prefix), (9, 5, 0));
        assert!(e.buy_once && !e.is_a_shop_item);
        // `:684-689`: a second sell of the same kind merges into the
        // buyOnce row (capped at the kind's maxStack).
        add_item_to_shop(&mut stock, &mut memo, &held(9, 3, 0));
        assert_eq!(stock.len(), 2);
        assert_eq!(stock[1].stack, 8);
        // A prefixed sell never merges into the unprefixed row
        // (`Item.CanStack`, `Item.cs:49865-49871`) — a fresh row.
        add_item_to_shop(&mut stock, &mut memo, &held(9, 2, 5));
        assert_eq!(stock.len(), 3);
        assert_eq!((stock[2].stack, stock[2].prefix), (2, 5));
    }

    #[test]
    fn the_memoed_resell_adds_nothing_back() {
        // `Chest.cs:672-676`: `Remove` eats the bought units first — a
        // full re-sell of bought goods adds NO row.
        let mut stock = vec![row(88, StockWhen::Always)];
        let mut memo = SellbackHelper::default();
        memo.add(8, 0, 5); // five torches bought
        add_item_to_shop(&mut stock, &mut memo, &held(8, 5, 0));
        assert_eq!(stock.len(), 1, "the bought units never re-stock");
        assert_eq!(memo.get_amount(8, 0), 0);
        // A partial memo: only the excess re-stocks (`num2 = stack - num`).
        memo.add(8, 0, 2);
        add_item_to_shop(&mut stock, &mut memo, &held(8, 5, 0));
        assert_eq!(stock.len(), 2);
        assert_eq!(stock[1].stack, 3, "5 sold − 2 memo'd = 3 re-stocked");
    }

    #[test]
    fn the_restock_window_is_39_slots() {
        // `Chest.cs:678`/`:691`: both loops scan `i < 39` — the 40th slot
        // never takes a sell; a full window drops the overflow.
        let mut stock: Vec<ShopEntry> = (0..39).map(|_| row(88, StockWhen::Always)).collect();
        let mut memo = SellbackHelper::default();
        add_item_to_shop(&mut stock, &mut memo, &held(9, 5, 0));
        assert_eq!(stock.len(), 39, "the full window drops the sell");
        // One slot free: the clone lands.
        stock.pop();
        add_item_to_shop(&mut stock, &mut memo, &held(9, 5, 0));
        assert_eq!(stock.len(), 39);
        assert_eq!(stock[38].item, 9);
    }

    #[test]
    fn the_restock_reuses_a_depleted_air_gap() {
        // `Chest.cs:697`'s `IsAir` arm: a buyOnce row that depleted to air
        // in place (`ItemSlot.cs:2092`) is the FIRST air slot — the sell
        // re-fills that gap, not the append position.
        let mut stock = vec![
            row(88, StockWhen::Always),
            crate::shop::ShopEntry {
                item: 0,
                when: StockWhen::Always,
                shop_custom_price: None,
                buy_once: false,
                is_a_shop_item: false,
                prefix: 0,
                stack: 0,
            },
        ];
        let mut memo = SellbackHelper::default();
        add_item_to_shop(&mut stock, &mut memo, &held(9, 4, 0));
        assert_eq!(stock.len(), 2, "the gap is reused, no append");
        assert_eq!((stock[1].item, stock[1].stack), (9, 4));
        assert!(stock[1].buy_once);
    }

    #[test]
    fn the_bulk_buy_gates_and_amounts() {
        // `ItemSlot.cs:3131-3138`: only isAShopItem/buyOnce rows bulk, and
        // only under Shift; `:3121-3129`: 10, clamped to the buyOnce stack.
        let plain = row(88, StockWhen::Always);
        assert!(!can_bulk_buy(&plain, true));
        assert_eq!(bulk_buy_amount(&plain), 10);
        let once = ShopEntry {
            buy_once: true,
            stack: 3,
            ..plain
        };
        assert!(can_bulk_buy(&once, true));
        assert!(!can_bulk_buy(&once, false));
        assert_eq!(bulk_buy_amount(&once), 3);
        let shop_item = ShopEntry {
            is_a_shop_item: true,
            ..plain
        };
        assert!(can_bulk_buy(&shop_item, true));
        assert_eq!(bulk_buy_amount(&shop_item), 10);
    }
}
