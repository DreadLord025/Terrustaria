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
/// list; the cap mirrors `byID`'s unbounded growth bounded by the
/// lifetimes).
#[derive(Clone, Debug)]
pub struct EmoteBubble {
    pub id: u32,
    pub emote: i32,
    /// Anchored to the local player (`WorldUIAnchor(Main.LocalPlayer)`).
    /// Kept as the `anchor_kind == 1 && anchor_index == local` shortcut the
    /// draw and `OnBubbleChange` read; the net anchor fields below carry
    /// the full `WorldUIAnchor` for the `91`-synced peer/NPC bubbles.
    pub anchored_to_player: bool,
    /// `SerializeNetAnchor`'s kind (`EmoteBubble.cs:78-98`): 0 NPC,
    /// 1 Player, 2 Projectile. The local wheel bubbles are kind 1.
    pub anchor_kind: u8,
    /// The anchor entity's `whoAmI` (the local player's slot for the wheel
    /// bubbles, the host's slot index for the `91`-synced ones).
    pub anchor_index: u16,
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
        self.new_bubble_on(emote, 1, 0, 0);
    }

    /// `EmoteBubble.NewBubble(emoticon, anchor, time)` with an explicit
    /// `SerializeNetAnchor` pair (W4-g50): the wheel's local bubbles are
    /// kind 1 / the local slot; the `120` host arm anchors on the sender's
    /// slot. The id is the port's local `NextID` — the host's `91` sync
    /// carries its own ids through [`Self::upsert_net`].
    pub fn new_bubble_on(
        &mut self,
        emote: i32,
        anchor_kind: u8,
        anchor_index: u16,
        local_id: u16,
    ) {
        self.next_id += 1;
        let id = self.next_id;
        self.bubbles.push(EmoteBubble {
            id,
            emote,
            anchored_to_player: anchor_kind == 1 && anchor_index == local_id,
            anchor_kind,
            anchor_index,
            life_time: 360,
            life_time_start: 360,
            frame: 0,
            frame_counter: 0,
        });
        self.on_bubble_change(id);
    }

    /// `EmoteBubble.OnBubbleChange` (`:262-276`): every OTHER bubble on the
    /// same anchor entity pops out — the C# assigns `lifeTime = 6`
    /// unconditionally (`:273`), even for a bubble already inside its
    /// closing pop. The anchor match is the full (kind, index) pair.
    pub fn on_bubble_change(&mut self, id: u32) {
        let Some(anchor) = self
            .bubbles
            .iter()
            .find(|b| b.id == id)
            .map(|b| (b.anchor_kind, b.anchor_index))
        else {
            return;
        };
        for b in &mut self.bubbles {
            if (b.anchor_kind, b.anchor_index) == anchor && b.id != id {
                b.life_time = 6;
            }
        }
    }

    /// The `91 SyncEmoteBubble` reader's upsert
    /// (`MessageBuffer.cs:3423-3438`): a new id lands with the wire's
    /// lifetime/emote/anchor; an existing id refreshes all four
    /// (`lifeTime`, `lifeTimeStart`, `emote`, `anchor`) and keeps its frame
    /// state. `OnBubbleChange` fires either way (`:3438`). `local_id` is
    /// the receiving client's slot — a kind-1 anchor on it is the local
    /// player (`DeserializeNetAnchor`, `EmoteBubble.cs:100-109`).
    pub fn upsert_net(&mut self, b: &crate::net::EmoteBubble, local_id: u16) {
        let id = b.id.max(0) as u32;
        let anchored = b.anchor_kind == 1 && b.anchor_index == local_id;
        if let Some(existing) = self.bubbles.iter_mut().find(|x| x.id == id) {
            existing.life_time = b.life_time as i32;
            existing.life_time_start = b.life_time as i32;
            existing.emote = b.emote;
            existing.anchor_kind = b.anchor_kind;
            existing.anchor_index = b.anchor_index;
            existing.anchored_to_player = anchored;
        } else {
            self.bubbles.push(EmoteBubble {
                id,
                emote: b.emote,
                anchored_to_player: anchored,
                anchor_kind: b.anchor_kind,
                anchor_index: b.anchor_index,
                life_time: b.life_time as i32,
                life_time_start: b.life_time as i32,
                frame: 0,
                frame_counter: 0,
            });
            self.next_id = self.next_id.max(id);
        }
        self.on_bubble_change(id);
    }

    /// The `91` removal arm (`MessageBuffer.cs:3402-3408`): the id leaves
    /// the by-ID map.
    pub fn remove_net(&mut self, id: i32) {
        self.bubbles.retain(|b| b.id != id.max(0) as u32);
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

impl EmoteBubble {
    /// `EmoteBubble.Draw`'s pop gate (`EmoteBubble.cs:204`): the first and
    /// last 6 ticks of the lifetime draw the closed shell only —
    /// `lifeTime < 6 || lifeTimeStart - lifeTime < 6`. S1-E-26 wave B:
    /// extracted from the draw closure so the gate is unit-testable
    /// without a renderer.
    pub fn popping(&self) -> bool {
        self.life_time < 6 || self.life_time_start - self.life_time < 6
    }

    /// The shell frame's column (`EmoteBubble.cs:205`): column 0 while
    /// popping, column 1 (the open bubble) otherwise — always row 0.
    pub fn shell_column(&self) -> i32 {
        i32::from(!self.popping())
    }

    /// The emote sprite's sheet cell (`EmoteBubble.cs:225`):
    /// `Frame(8, EMOTE_SHEET_VERTICAL_FRAMES, emote*2%8 + frame,
    /// 1 + emote/4)` — each emote owns two adjacent columns (its two
    /// animation frames) in row `1 + emote/4`. Only meaningful for
    /// `emote >= 0` (`:218`); the `-1` NPC-head arm (`:227-240`) is
    /// deferred with the NPC anchors.
    pub fn emote_frame(&self) -> (i32, i32) {
        ((self.emote * 2) % 8 + self.frame, 1 + self.emote / 4)
    }
}

/// `EmoteBubble.Draw`'s 87/89 un-flip (`EmoteBubble.cs:220-224`): the
/// confused (87) and sleep (89) emotes keep their orientation when the
/// anchor faces right — the horizontal flip is cleared and the sprite
/// shifts 4 px right instead. Applies to the emote sprite only, after
/// the shell drew with the original flip (`:213`).
pub fn bubble_unflips(emote: i32, flip: bool) -> bool {
    flip && (emote == 87 || emote == 89)
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

    // S1-E-26 wave B — the bubble frame-selection math, verified against
    // `EmoteBubble.Draw` (`EmoteBubble.cs:199-241`) line by line. Tests
    // written, not run (code-only mode).

    #[test]
    fn the_pop_gate_is_the_first_and_last_6_ticks() {
        // `EmoteBubble.cs:204`: `lifeTime < 6 || lifeTimeStart - lifeTime < 6`.
        let mut sys = EmoteSystem::default();
        sys.new_bubble(0); // lifeTime = lifeTimeStart = 360 (`:284`).
        let b = &sys.bubbles[0];
        assert!(b.popping(), "age 0 < 6: the opening pop");
        assert_eq!(b.shell_column(), 0, "`:205`'s flag arm draws column 0");
        // Age 5 is still popping; age 6 is the first open tick.
        let mut open = sys.bubbles[0].clone();
        open.life_time = 355; // age 5
        assert!(open.popping());
        open.life_time = 354; // age 6
        assert!(!open.popping());
        assert_eq!(open.shell_column(), 1, "the open shell is column 1");
        // The closing pop: lifeTime 6..1 pops, 0 is already cleaned up
        // (`UpdateAll` `:54`).
        for lt in (1..=5).rev() {
            open.life_time = lt;
            assert!(open.popping(), "lifeTime {lt} < 6");
            assert_eq!(open.shell_column(), 0);
        }
        open.life_time = 6;
        assert!(!open.popping(), "lifeTime 6 is the last open tick");
        assert_eq!(open.shell_column(), 1);
    }

    #[test]
    fn the_pop_gate_runs_through_the_lifetime() {
        // Drive the real update: the gate flips at age 6 and back at
        // lifeTime 5, exactly once each over the 360-tick life.
        let mut sys = EmoteSystem::default();
        sys.new_bubble(3);
        let mut open_ticks = 0;
        for _ in 0..360 {
            if let Some(b) = sys.bubbles.first() {
                if !b.popping() {
                    open_ticks += 1;
                }
            }
            sys.update_all();
        }
        // 360 ticks − 6 opening − 5 closing (lifeTime 5..1) = 349 open.
        assert_eq!(open_ticks, 349);
    }

    #[test]
    fn the_emote_frame_is_the_c_sharp_cell_formula() {
        // `EmoteBubble.cs:225`: column `emote*2%8 + frame`, row
        // `1 + emote/4`. Emote 0 → cols 0/1 row 1; emote 3 → cols 6/7
        // row 1; emote 4 wraps to cols 0/1 row 2; emote 87 → col 6 row 22;
        // emote 150 (the last, `EmoteID.cs:311`) → col 4 row 38.
        let mut sys = EmoteSystem::default();
        sys.new_bubble(0);
        let mut b = sys.bubbles[0].clone();
        assert_eq!(b.emote_frame(), (0, 1));
        b.frame = 1;
        assert_eq!(b.emote_frame(), (1, 1));
        b.emote = 3;
        b.frame = 0;
        assert_eq!(b.emote_frame(), (6, 1));
        b.emote = 4;
        assert_eq!(b.emote_frame(), (0, 2));
        b.emote = 87;
        assert_eq!(b.emote_frame(), (6, 22));
        b.emote = 150;
        assert_eq!(b.emote_frame(), (4, 38));
        // The last row fits the sheet: SHEET_ROWS = 2 + (151-1)/4 = 39
        // (`EmoteBubble.cs:1071`), rows 1..38 hold the emotes.
        assert_eq!(SHEET_ROWS, 39);
        assert!(b.emote_frame().1 < SHEET_ROWS);
    }

    #[test]
    fn only_87_and_89_unflip_when_facing_right() {
        // `EmoteBubble.cs:220-224`: `(emote == 87 || emote == 89) &&
        // FlipHorizontally` — every other emote keeps the anchor's flip.
        assert!(bubble_unflips(87, true));
        assert!(bubble_unflips(89, true));
        assert!(!bubble_unflips(87, false), "no flip to clear");
        assert!(!bubble_unflips(88, true), "the kiss emote flips normally");
        assert!(!bubble_unflips(0, true));
        assert!(!bubble_unflips(150, true));
    }

    #[test]
    fn the_sheet_geometry_matches_the_frame_division() {
        // `Utils.Frame` is integer division: Extra_48 is 272x1092, so
        // 272/8 = 34 wide and 1092/39 = 28 tall — the frame size the
        // `EmoteButton` ctor reads (`EmoteButton.cs:27-29`).
        let (sheet_w, sheet_h) = (272u32, 1092u32);
        assert_eq!(sheet_w / 8, 34);
        assert_eq!(sheet_h / SHEET_ROWS as u32, 28);
        // `EMOTE_SHEET_EMOTES_PER_ROW = 4` (`EmoteBubble.cs:40`): four
        // emotes per row, two columns each — 8 columns total (`:38`).
        assert_eq!(4 * 2, 8);
    }
}
