//! The emote bubbles — `EmoteBubble` (`GameContent/UI/EmoteBubble.cs`):
//! the by-ID map, the 8×N `Extra_48` sheet (row 0 = the pop shell, each
//! emote rides 2 animation frames in `emote*2%8 + frame` columns of row
//! `1 + emote/4`), the 360-tick lifetime with the 6-tick pop gates and the
//! `UIEmotesMenu` wheel (the 8-column grid of the `EmoteButton` sprites).
//!
//! The port anchors the bubbles to the local player only (the NPC reactions
//! need the town NPCs — `CheckForNPCsToReactToEmoteBubble` `ai[0] = 19`
//! stays out with M4-TOWN); the emote NAMES are the `EmoteID` const names
//! (the localized `EmojiName` table lands with M6-LOCALIZE).

/// `EmoteID.Count` (`EmoteID.cs:317`).
pub const EMOTE_COUNT: i32 = 151;
/// `EMOTE_SHEET_VERTICAL_FRAMES` (`EmoteBubble.cs:1071`).
pub const SHEET_ROWS: i32 = 2 + (EMOTE_COUNT - 1) / 4;

/// The `EmoteID` const names by id (`EmoteID.Search.GetName` — the port
/// reads the const table).
pub const EMOTE_NAMES: [(i32, &str); 151] = [
(0, "EmotionLove"),
    (1, "EmotionAnger"),
    (2, "EmotionCry"),
    (3, "EmotionAlert"),
    (4, "WeatherRain"),
    (5, "WeatherLightning"),
    (6, "WeatherRainbow"),
    (7, "ItemRing"),
    (8, "DebuffPoison"),
    (9, "DebuffBurn"),
    (10, "DebuffSilence"),
    (11, "DebuffCurse"),
    (12, "CritterBee"),
    (13, "CritterSlime"),
    (14, "MiscTree"),
    (15, "EmoteLaugh"),
    (16, "EmoteFear"),
    (17, "EmoteNote"),
    (18, "EventBloodmoon"),
    (19, "EventEclipse"),
    (20, "EventPumpkin"),
    (21, "EventSnow"),
    (22, "BiomeSky"),
    (23, "BiomeOtherworld"),
    (24, "BiomeJungle"),
    (25, "BiomeCrimson"),
    (26, "BiomeCorruption"),
    (27, "BiomeHallow"),
    (28, "BiomeDesert"),
    (29, "BiomeBeach"),
    (30, "BiomeRocklayer"),
    (31, "BiomeLavalayer"),
    (32, "BiomeSnow"),
    (33, "RPSWinScissors"),
    (34, "RPSWinRock"),
    (35, "RPSWinPaper"),
    (36, "RPSScissors"),
    (37, "RPSRock"),
    (38, "RPSPaper"),
    (39, "BossEoC"),
    (40, "BossEoW"),
    (41, "BossBoC"),
    (42, "BossQueenBee"),
    (43, "BossSkeletron"),
    (44, "BossWoF"),
    (45, "BossDestroyer"),
    (46, "BossSkeletronPrime"),
    (47, "BossTwins"),
    (48, "BossPlantera"),
    (49, "BossGolem"),
    (50, "BossFishron"),
    (51, "BossKingSlime"),
    (52, "BossCultist"),
    (53, "BossMoonmoon"),
    (54, "BossMourningWood"),
    (55, "BossPumpking"),
    (56, "BossEverscream"),
    (57, "BossIceQueen"),
    (58, "BossSantank"),
    (59, "BossPirateship"),
    (60, "BossMartianship"),
    (61, "CritterZombie"),
    (62, "CritterBunny"),
    (63, "CritterButterfly"),
    (64, "CritterGoblin"),
    (65, "CritterPirate"),
    (66, "CritterSnowman"),
    (67, "CritterSpider"),
    (68, "CritterBird"),
    (69, "CritterMouse"),
    (70, "CritterGoldfish"),
    (71, "CritterMartian"),
    (72, "CritterSkeleton"),
    (73, "ItemLifePotion"),
    (74, "ItemManaPotion"),
    (75, "ItemSoup"),
    (76, "ItemCookedFish"),
    (77, "ItemAle"),
    (78, "ItemSword"),
    (79, "ItemFishingRod"),
    (80, "ItemBugNet"),
    (81, "ItemDynamite"),
    (82, "ItemMinishark"),
    (83, "ItemCog"),
    (84, "ItemTombstone"),
    (85, "ItemGoldpile"),
    (86, "ItemDiamondRing"),
    (87, "EmoteConfused"),
    (88, "EmoteKiss"),
    (89, "EmoteSleep"),
    (90, "ItemPickaxe"),
    (91, "EmoteRun"),
    (92, "EmoteKick"),
    (93, "EmoteFight"),
    (94, "EmoteEating"),
    (95, "WeatherSunny"),
    (96, "WeatherCloudy"),
    (97, "WeatherStorming"),
    (98, "WeatherSnowstorm"),
    (99, "EventMeteor"),
    (100, "MiscFire"),
    (101, "TownMerchant"),
    (102, "TownNurse"),
    (103, "TownArmsDealer"),
    (104, "TownDryad"),
    (105, "TownGuide"),
    (106, "TownOldman"),
    (107, "TownDemolitionist"),
    (108, "TownClothier"),
    (109, "TownGoblinTinkerer"),
    (110, "TownWizard"),
    (111, "TownMechanic"),
    (112, "TownSanta"),
    (113, "TownTruffle"),
    (114, "TownSteampunker"),
    (115, "TownDyeTrader"),
    (116, "TownPartyGirl"),
    (117, "TownCyborg"),
    (118, "TownPainter"),
    (119, "TownWitchDoctor"),
    (120, "TownPirate"),
    (121, "TownStylist"),
    (122, "TownTravellingMerchant"),
    (123, "TownAngler"),
    (124, "TownSkeletonMerchant"),
    (125, "TownTaxCollector"),
    (126, "PartyPresent"),
    (127, "PartyBalloons"),
    (128, "PartyCake"),
    (129, "PartyHats"),
    (130, "TownBartender"),
    (131, "ItemBeer"),
    (132, "ItemDefenderMedal"),
    (133, "EventOldOnesArmy"),
    (134, "EmoteSadness"),
    (135, "EmoteAnger"),
    (136, "EmoteHappiness"),
    (137, "EmoteWink"),
    (138, "EmoteScowl"),
    (139, "EmoteSilly"),
    (140, "TownGolfer"),
    (141, "TownBestiaryGirl"),
    (142, "TownBestiaryGirlFox"),
    (143, "BossEmpressOfLight"),
    (144, "BossQueenSlime"),
    (145, "TownPrincess"),
    (146, "Peckish"),
    (147, "Hungry"),
    (148, "Starving"),
    (149, "LucyTheAxe"),
    (150, "BossDeerclops"),
];

/// The bubble state — one `EmoteBubble` instance (the by-ID map folds to a
/// list; the port's single player owns at most one at a time in practice,
/// the cap mirrors `byID`'s unbounded growth bounded by the lifetimes).
#[derive(Clone, Debug)]
pub struct EmoteBubble {
    pub id: u32,
    pub emote: i32,
    /// Anchored to the local player (`WorldUIAnchor(Main.LocalPlayer)`).
    pub anchored_to_player: bool,
    pub life_time: i32,
    pub life_time_start: i32,
    pub frame: i32,
    pub frame_counter: i32,
}

/// `EmoteBubble.byID` + `NextID`.
#[derive(Default)]
pub struct EmoteSystem {
    pub bubbles: Vec<EmoteBubble>,
    next_id: u32,
    /// The `UIEmotesMenu` wheel state: open + the scroll offset.
    pub menu_open: bool,
    pub scroll: i32,
}

impl EmoteSystem {
    /// `EmoteBubble.NewBubble` (`:116-131`): a 360-tick bubble on the
    /// local player.
    pub fn new_bubble(&mut self, emote: i32) {
        // `MakeLocalPlayerEmote`'s dead gate (`:282`).
        if self.bubbles.iter().any(|b| b.anchored_to_player) {
            // `OnBubbleChange` (`:255-268`): the same-anchor bubbles fade
            // out (the 6-tick pop).
            for b in &mut self.bubbles {
                if b.anchored_to_player && b.life_time > 6 {
                    b.life_time = 6;
                }
            }
        }
        self.next_id += 1;
        self.bubbles.push(EmoteBubble {
            id: self.next_id,
            emote,
            anchored_to_player: true,
            life_time: 360,
            life_time_start: 360,
            frame: 0,
            frame_counter: 0,
        });
    }

    /// `EmoteBubble.UpdateAll` (`:46-63`): the lifetime tick, the 8-tick
    /// frame flip over the 2 animation frames, the expired cleanup.
    pub fn update_all(&mut self) {
        for b in &mut self.bubbles {
            b.life_time -= 1;
            b.frame_counter += 1;
            if b.frame_counter >= 8 {
                b.frame_counter = 0;
                b.frame = (b.frame + 1) % 2;
            }
        }
        self.bubbles.retain(|b| b.life_time > 0);
    }
}

/// The emote id from the `/emoji` argument — the numeric form or the
/// `EmoteID` const name (`EmojiCommand.ProcessOutgoingMessage`
/// `:44-72`: the int parse first, then the name table).
pub fn resolve(arg: &str) -> Option<i32> {
    if let Ok(id) = arg.parse::<i32>() {
        return (0..EMOTE_COUNT).contains(&id).then_some(id);
    }
    let lower = arg.trim_matches('"').to_ascii_lowercase();
    EMOTE_NAMES
        .iter()
        .find(|(_, name)| name.eq_ignore_ascii_case(&lower))
        .map(|(id, _)| *id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_bubble_lifecycle_pops_and_expires() {
        let mut sys = EmoteSystem::default();
        sys.new_bubble(0);
        assert_eq!(sys.bubbles.len(), 1);
        // The second bubble fades the first (`OnBubbleChange`).
        sys.new_bubble(4);
        assert_eq!(sys.bubbles.len(), 2);
        assert_eq!(sys.bubbles[0].life_time, 6);
        // The first pops out after 6 ticks.
        for _ in 0..6 {
            sys.update_all();
        }
        assert_eq!(sys.bubbles.len(), 1);
        assert_eq!(sys.bubbles[0].emote, 4);
        // The frame flips every 8 ticks over 2 frames.
        for _ in 0..8 {
            sys.update_all();
        }
        assert_eq!(sys.bubbles[0].frame, 1);
        // The 360-tick lifetime expires.
        for _ in 0..400 {
            sys.update_all();
        }
        assert!(sys.bubbles.is_empty());
    }

    #[test]
    fn the_resolver_takes_ids_and_names() {
        assert_eq!(resolve("4"), Some(4));
        assert_eq!(resolve("150"), Some(150));
        assert_eq!(resolve("151"), None);
        assert_eq!(resolve("EmotionLove"), Some(0));
        assert_eq!(resolve("emotionlove"), Some(0));
        assert_eq!(resolve("nope"), None);
    }
}
