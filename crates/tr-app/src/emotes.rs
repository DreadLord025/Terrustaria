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
    /// The shared `EmoteButton._frameCounter` (`EmoteButton.cs:20`,
    /// `UpdateFrame` `:38-44`): every button on the page ticks the same
    /// 0..19 counter (the page is rebuilt on open, so they all start at 0
    /// together — `UIEmotesMenu.OnActivate` `:30-44`). The port rides the
    /// world step (`update_all`) instead of the UI update; the wheel is
    /// only reachable in-game where the world keeps stepping.
    pub button_tick: i32,
    /// The hovered cell (`EmoteButton._hovered`, `MouseOver`/`MouseOut`
    /// `:77-88`) — the port's hover-edge tracker for the sound-12 cue
    /// (`loop.rs`'s `CursorMoved`, the Bug-17 front-end pattern); the draw
    /// derives its highlight from the pointer directly (house style).
    pub hover: Option<i32>,
}

impl EmoteSystem {
    /// `UIEmotesMenu.OnActivate`'s rebuild (`UIEmotesMenu.cs:30-44`): the page
    /// is rebuilt from scratch on every open, so the wheel's scroll starts at 0
    /// (`UIList`'s `Clear` + `InitializePage`).
    pub fn open(&mut self) {
        self.menu_open = true;
        self.scroll = 0;
        // The rebuilt page's buttons all start their frame counter at 0
        // (`InitializePage`'s fresh `EmoteButton`s, `EmoteButton.cs:20`).
        self.button_tick = 0;
        self.hover = None;
    }

    /// `UIEmotesMenu`'s close (`IngameFancyUI.Close` `:162-188` / the pick and
    /// Back arms `:281-286`): the state drops with the panel.
    pub fn close(&mut self) {
        self.menu_open = false;
        self.scroll = 0;
        self.hover = None;
    }

    /// The wheel scroll (`UIList.ScrollWheel`, `UIList.cs:107-114`): the offset
    /// moves by the wheel's notches and clamps at the roster's end. `visible_rows`
    /// is how many rows the open panel fits — the caller's (`ui::emotes::grid_rect`
    /// divided by the 36 px cell), the port's stand-in for `UIScrollbar`'s
    /// `viewSize`/`maxViewSize` pair.
    pub fn scroll_by(&mut self, step: i32, visible_rows: usize) {
        let total = crate::emotes::EMOTE_COUNT as usize;
        let rows = total.div_ceil(crate::ui::emotes::COLS);
        let max = rows.saturating_sub(visible_rows) as i32;
        self.scroll = (self.scroll + step).clamp(0, max);
    }
}

impl EmoteSystem {
    /// `EmoteBubble.NewBubble` (`:116-131`) + `MakeLocalPlayerEmote`'s
    /// 360-tick arm (`:284`): a bubble on the local player.
    pub fn new_bubble(&mut self, emote: i32) {
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
        // `OnBubbleChange` (`:262-276`): every OTHER bubble on the same
        // player pops out — the C# assigns `lifeTime = 6` unconditionally
        // (`:273`), even for a bubble already inside its closing pop.
        for b in &mut self.bubbles {
            if b.anchored_to_player && b.id != self.next_id {
                b.life_time = 6;
            }
        }
    }

    /// `EmoteBubble.UpdateAll` (`:46-63`) over `Update` (`:177-187`): the
    /// lifetime tick with the short-circuit frame advance (`--lifeTime > 0
    /// && ++frameCounter >= 8`, `frameSpeed = 8` `:32`) and the expired
    /// cleanup. Also drives the wheel buttons' shared 20-tick frame counter
    /// (`EmoteButton.UpdateFrame` `:38-44`).
    pub fn update_all(&mut self) {
        for b in &mut self.bubbles {
            b.life_time -= 1;
            if b.life_time > 0 {
                b.frame_counter += 1;
                if b.frame_counter >= 8 {
                    b.frame_counter = 0;
                    b.frame += 1;
                    if b.frame >= 2 {
                        b.frame = 0;
                    }
                }
            }
        }
        self.bubbles.retain(|b| b.life_time > 0);
        self.button_tick += 1;
        if self.button_tick >= 20 {
            self.button_tick = 0;
        }
    }

    /// `EmoteButton.GetFrame`'s hover-frame pick (`:32-36`): frame 1 while
    /// the shared counter is in the second half of its 20-tick cycle
    /// (`_frameCounter >= 10`) — the button's idle blink, NOT the hover
    /// highlight (that is the border tint, `DrawSelf` `:63-69`).
    pub fn button_frame(&self) -> i32 {
        i32::from(self.button_tick >= 10)
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
    fn the_bubble_lives_exactly_360_ticks() {
        // `MakeLocalPlayerEmote`'s `NewBubble(emoteId, anchor, 360)`
        // (`EmoteBubble.cs:284`) + `Update`'s `--lifeTime` (`:179`) and
        // `UpdateAll`'s `lifeTime <= 0` cleanup (`:54`): the bubble is
        // still alive after 359 ticks and gone on the 360th — not before.
        let mut sys = EmoteSystem::default();
        sys.new_bubble(0);
        for _ in 0..359 {
            sys.update_all();
        }
        assert_eq!(sys.bubbles.len(), 1, "359 ticks: still alive");
        assert_eq!(sys.bubbles[0].life_time, 1);
        sys.update_all();
        assert!(sys.bubbles.is_empty(), "the 360th tick expires it");
    }

    #[test]
    fn the_frame_advance_stops_at_expiry_and_flips_every_8_ticks() {
        // `Update` (`EmoteBubble.cs:177-187`): `--lifeTime > 0 &&
        // ++frameCounter >= 8` — the frame counter short-circuits once the
        // lifetime is spent, and the frame wraps 0 → 1 → 0.
        let mut sys = EmoteSystem::default();
        sys.new_bubble(0);
        for tick in 1..=16 {
            sys.update_all();
            let b = &sys.bubbles[0];
            assert_eq!(b.frame, (tick / 8) % 2, "tick {tick}");
        }
        // Fade the bubble to its last tick and pin the frozen counter.
        sys.bubbles[0].life_time = 1;
        sys.bubbles[0].frame_counter = 3;
        sys.update_all();
        assert!(sys.bubbles.is_empty());
    }

    #[test]
    fn the_second_bubble_pops_the_first_even_mid_pop() {
        // `OnBubbleChange` (`EmoteBubble.cs:262-276`): every other bubble
        // on the same player gets `lifeTime = 6` unconditionally (`:273`)
        // — including one already inside its closing pop.
        let mut sys = EmoteSystem::default();
        sys.new_bubble(0);
        sys.bubbles[0].life_time = 3; // already popping out
        sys.new_bubble(4);
        assert_eq!(sys.bubbles[0].life_time, 6, "reset to the full pop");
        assert_eq!(sys.bubbles[1].life_time, 360, "the new one untouched");
    }

    #[test]
    fn the_button_blink_is_a_20_tick_cycle_not_the_hover() {
        // `EmoteButton.UpdateFrame` (`:38-44`) + `GetFrame` (`:32-36`):
        // `++_frameCounter >= 20` wraps; frame 1 while `>= 10`. The wheel's
        // buttons share the counter (the page rebuild starts them all at 0).
        let mut sys = EmoteSystem::default();
        sys.open();
        assert_eq!(sys.button_frame(), 0);
        for _ in 0..10 {
            sys.update_all();
        }
        assert_eq!(sys.button_frame(), 1);
        for _ in 0..10 {
            sys.update_all();
        }
        assert_eq!(sys.button_tick, 0);
        assert_eq!(sys.button_frame(), 0);
        // Each open rebuilds the page (`UIEmotesMenu.OnActivate` `:30-44`).
        for _ in 0..12 {
            sys.update_all();
        }
        sys.open();
        assert_eq!(sys.button_frame(), 0);
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
