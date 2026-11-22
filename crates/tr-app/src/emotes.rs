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

/// One `EmotesGroupListItem`'s roster (`UIEmotesMenu.TryAddingList`,
/// `UIEmotesMenu.cs:100-106`): the localized title key, the group index
/// (`currentGroupIndex++` only when the list is non-empty, `:104`) and the
/// emote ids in the C# list order.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EmoteGroup {
    /// `Language.GetText("UI.EmoteCategory…")` (`UIEmotesMenu.cs:91-97`).
    pub title_key: &'static str,
    /// `_groupIndex` (`EmotesGroupListItem.cs:19`, `CompareTo` `:87-94`).
    pub index: i32,
    pub emotes: Vec<i32>,
}

/// The `NPC.downed*` / `Main.hardMode` / `DD2Event` gates the Critters and
/// Bosses lists read (`UIEmotesMenu.GetEmotesCritters` `:153-176`,
/// `GetEmotesBosses` `:178-283`). The page is rebuilt on every open
/// (`OnActivate` → `InitializePage`), so the flags are sampled once per open.
///
/// The Empress / Halloween / Christmas flags have no ported world state
/// (`wld/sections/flags.rs:146-150`/`:217` skip them) — the caller passes
/// `false` and the six gated emotes (143, 54, 55, 56, 57, 58) stay out until
/// those flags land (documented in ANALYSIS, W4-g54).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct EmoteGates {
    /// `NPC.downedGoblins` (`:159`).
    pub goblins: bool,
    /// `NPC.downedFrost` (`:163`).
    pub frost: bool,
    /// `NPC.downedPirates` (`:167`, `:270`).
    pub pirates: bool,
    /// `NPC.downedMartians` (`:171`, `:274`).
    pub martians: bool,
    /// `NPC.downedBoss1` (`:181`).
    pub boss1: bool,
    /// `NPC.downedBoss2` (`:185`) — the port's `downed_eow` covers both the
    /// Eater and the Brain (`world_sim/npcs.rs:5295`/`:5322`).
    pub boss2: bool,
    /// `NPC.downedSlimeKing` (`:190`).
    pub slime_king: bool,
    /// `NPC.downedDeerclops` (`:194`).
    pub deerclops: bool,
    /// `NPC.downedQueenBee` (`:198`).
    pub queen_bee: bool,
    /// `NPC.downedBoss3` (`:202`).
    pub boss3: bool,
    /// `Main.hardMode` (`:206`).
    pub hard_mode: bool,
    /// `NPC.downedQueenSlime` (`:210`).
    pub queen_slime: bool,
    /// `NPC.downedMechBoss1` (`:214`).
    pub mech1: bool,
    /// `NPC.downedMechBoss3` (`:218`) — note the C# order: 3 before 2.
    pub mech3: bool,
    /// `NPC.downedMechBoss2` (`:222`).
    pub mech2: bool,
    /// `NPC.downedPlantBoss` (`:226`).
    pub plant: bool,
    /// `NPC.downedGolemBoss` (`:230`).
    pub golem: bool,
    /// `NPC.downedFishron` (`:234`).
    pub fishron: bool,
    /// `NPC.downedEmpressOfLight` (`:238`) — unowned in the port's world.
    pub empress: bool,
    /// `NPC.downedAncientCultist` (`:242`).
    pub cultist: bool,
    /// `NPC.downedMoonlord` (`:246`).
    pub moonlord: bool,
    /// `NPC.downedHalloweenTree` (`:250`) — unowned.
    pub halloween_tree: bool,
    /// `NPC.downedHalloweenKing` (`:254`) — unowned.
    pub halloween_king: bool,
    /// `NPC.downedChristmasTree` (`:258`) — unowned.
    pub christmas_tree: bool,
    /// `NPC.downedChristmasIceQueen` (`:262`) — unowned.
    pub christmas_ice_queen: bool,
    /// `NPC.downedChristmasSantank` (`:266`) — unowned.
    pub christmas_santank: bool,
    /// `DD2Event.DownedInvasionAnyDifficulty` (`:278`, `DD2Event.cs:89-98`:
    /// T1 || T2 || T3).
    pub dd2_any: bool,
}

/// `UIEmotesMenu.InitializePage`'s seven `TryAddingList` calls
/// (`UIEmotesMenu.cs:90-97`): the rosters in the C# order, the Critters and
/// Bosses tails gated by the downed flags, an empty list skipped WITHOUT
/// consuming a group index (`:100-106`'s `ref currentGroupIndex++` inside
/// the non-empty arm). The per-row caps (10/10/11/8/9/7/8) are dead — the
/// `EmotesGroupListItem` ctor hard-overrides `maxEmotesPerRow = 14`
/// (`EmotesGroupListItem.cs:25`), so the port keeps one 14-wide lattice.
pub fn build_groups(gates: &EmoteGates) -> Vec<EmoteGroup> {
    let mut groups: Vec<EmoteGroup> = Vec::new();
    let mut next_index = 0i32;
    let mut add = |title_key: &'static str, emotes: Vec<i32>| {
        if !emotes.is_empty() {
            groups.push(EmoteGroup {
                title_key,
                index: next_index,
                emotes,
            });
            next_index += 1;
        }
    };
    // `GetEmotesGeneral` (`:108-116`).
    add(
        "UI.EmoteCategoryGeneral",
        vec![
            0, 1, 2, 3, 15, 136, 94, 16, 135, 134, 137, 138, 139, 17, 87, 88, 89, 91, 92, 93,
            8, 9, 10, 11, 14, 100, 146, 147, 148,
        ],
    );
    // `GetEmotesRPS` (`:118-121`).
    add("UI.EmoteCategoryRPS", vec![36, 37, 38, 33, 34, 35]);
    // `GetEmotesItems` (`:123-131`).
    add(
        "UI.EmoteCategoryItems",
        vec![
            7, 73, 74, 75, 76, 131, 77, 78, 79, 80, 81, 82, 83, 84, 85, 86, 90, 132, 126, 127,
            128, 129, 149,
        ],
    );
    // `GetEmotesBiomesAndEvents` (`:133-141`).
    add(
        "UI.EmoteCategoryBiomesAndEvents",
        vec![
            22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 18, 19, 20, 21, 99, 4, 5, 6, 95, 96,
            97, 98,
        ],
    );
    // `GetEmotesTownNPCs` (`:143-151`).
    add(
        "UI.EmoteCategoryTownNPCs",
        vec![
            101, 102, 103, 104, 105, 106, 107, 108, 109, 110, 111, 112, 113, 114, 115, 116,
            117, 118, 119, 120, 121, 122, 123, 124, 125, 130, 140, 141, 142, 145,
        ],
    );
    // `GetEmotesCritters` (`:153-176`): the 10 fixed ids, then the four
    // invasion-gated ones in the C# append order.
    let mut critters = vec![12, 13, 61, 62, 63, 67, 68, 69, 70, 72];
    if gates.goblins {
        critters.push(64);
    }
    if gates.frost {
        critters.push(66);
    }
    if gates.pirates {
        critters.push(65);
    }
    if gates.martians {
        critters.push(71);
    }
    add("UI.EmoteCategoryCritters", critters);
    // `GetEmotesBosses` (`:178-283`): the downed ladder in the C# if-order
    // (boss2 adds TWO ids; mech3 before mech2; pirates/martians reuse the
    // critter flags).
    let mut bosses: Vec<i32> = Vec::new();
    if gates.boss1 {
        bosses.push(39);
    }
    if gates.boss2 {
        bosses.push(40);
        bosses.push(41);
    }
    if gates.slime_king {
        bosses.push(51);
    }
    if gates.deerclops {
        bosses.push(150);
    }
    if gates.queen_bee {
        bosses.push(42);
    }
    if gates.boss3 {
        bosses.push(43);
    }
    if gates.hard_mode {
        bosses.push(44);
    }
    if gates.queen_slime {
        bosses.push(144);
    }
    if gates.mech1 {
        bosses.push(45);
    }
    if gates.mech3 {
        bosses.push(46);
    }
    if gates.mech2 {
        bosses.push(47);
    }
    if gates.plant {
        bosses.push(48);
    }
    if gates.golem {
        bosses.push(49);
    }
    if gates.fishron {
        bosses.push(50);
    }
    if gates.empress {
        bosses.push(143);
    }
    if gates.cultist {
        bosses.push(52);
    }
    if gates.moonlord {
        bosses.push(53);
    }
    if gates.halloween_tree {
        bosses.push(54);
    }
    if gates.halloween_king {
        bosses.push(55);
    }
    if gates.christmas_tree {
        bosses.push(56);
    }
    if gates.christmas_ice_queen {
        bosses.push(57);
    }
    if gates.christmas_santank {
        bosses.push(58);
    }
    if gates.pirates {
        bosses.push(59);
    }
    if gates.martians {
        bosses.push(60);
    }
    if gates.dd2_any {
        bosses.push(133);
    }
    add("UI.EmoteCategoryBosses", bosses);
    groups
}

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
    /// `UIScrollbar._viewPosition` in PIXELS (`UIList.DrawSelf`'s
    /// `_innerList.Top = -GetValue()`, `UIList.cs:179-182`) — W4-g54 replaces
    /// the flat-roster row count with the real grouped-list scroll.
    pub scroll: f32,
    /// `InitializePage`'s seven `EmotesGroupListItem`s
    /// (`UIEmotesMenu.cs:90-97`), rebuilt on every open from the downed
    /// flags — the page the draw/hit/scroll all read.
    pub groups: Vec<EmoteGroup>,
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

impl EmoteGates {
    /// The world's downed flags (`NPC.downed*` statics → the port's `World`
    /// fields, `tr-core/src/world.rs:192-237`/`:368`/`:401-404`/`:450-452`).
    /// The six unowned flags (Empress, Halloween×2, Christmas×3 —
    /// `wld/sections/flags.rs:146-150`/`:217` skip them on read AND write)
    /// stay `false`; `DD2Event.DownedInvasionAnyDifficulty` is
    /// `T1 || T2 || T3` (`DD2Event.cs:89-98`).
    pub fn from_world(w: &tr_core::World) -> Self {
        Self {
            goblins: w.downed_goblins,
            frost: w.downed_frost,
            pirates: w.downed_pirates,
            martians: w.downed_martians,
            boss1: w.downed_eoc,
            boss2: w.downed_eow,
            slime_king: w.downed_slime_king,
            deerclops: w.downed_deerclops,
            queen_bee: w.downed_queen_bee,
            boss3: w.downed_skeletron,
            hard_mode: w.hard_mode,
            queen_slime: w.downed_queen_slime,
            mech1: w.downed_mech_boss1,
            mech3: w.downed_mech_boss3,
            mech2: w.downed_mech_boss2,
            plant: w.downed_plant_boss,
            golem: w.downed_golem,
            fishron: w.downed_fishron,
            empress: false,
            cultist: w.downed_ancient_cultist,
            moonlord: w.downed_moon_lord,
            halloween_tree: false,
            halloween_king: false,
            christmas_tree: false,
            christmas_ice_queen: false,
            christmas_santank: false,
            dd2_any: w.dd2_downed_t1 || w.dd2_downed_t2 || w.dd2_downed_t3,
        }
    }
}

impl EmoteSystem {
    /// `UIEmotesMenu.OnActivate`'s rebuild (`UIEmotesMenu.cs:30-44`): the page
    /// is rebuilt from scratch on every open (`InitializePage`'s seven
    /// `TryAddingList` calls, `:90-97`), so the wheel's scroll starts at 0
    /// (`UIList`'s fresh `UIScrollbar`, `_viewPosition = 0`).
    pub fn open_with(&mut self, gates: &EmoteGates) {
        self.menu_open = true;
        self.scroll = 0.0;
        self.groups = build_groups(gates);
        // The rebuilt page's buttons all start their frame counter at 0
        // (`InitializePage`'s fresh `EmoteButton`s, `EmoteButton.cs:20`).
        self.button_tick = 0;
        self.hover = None;
    }

    /// [`open_with`](Self::open_with) with every gate closed — the fresh
    /// world's page (6 groups, no Bosses).
    pub fn open(&mut self) {
        self.open_with(&EmoteGates::default());
    }

    /// `UIEmotesMenu`'s close (`IngameFancyUI.Close` `:162-188` / the pick and
    /// Back arms `:281-286`): the state drops with the panel.
    pub fn close(&mut self) {
        self.menu_open = false;
        self.scroll = 0.0;
        self.hover = None;
    }

    /// The wheel scroll (`UIList.ScrollWheel`, `UIList.cs:107-114`:
    /// `_scrollbar.ViewPosition -= evt.ScrollWheelValue`) in PIXELS, clamped
    /// by the `ViewPosition` setter (`UIScrollbar.cs:45`:
    /// `MathHelper.Clamp(value, 0, maxViewSize - viewSize)`). `max_scroll`
    /// is the caller's `ui::emotes::max_scroll` over the rebuilt page.
    pub fn scroll_by(&mut self, delta: f32, max_scroll: f32) {
        self.scroll = (self.scroll - delta).clamp(0.0, max_scroll.max(0.0));
    }

    /// `UIScrollbar.LeftMouseDown`'s track jump (`UIScrollbar.cs:137-141`)
    /// and drag (`:109-112`) share the clamp — the caller computes the value
    /// through `panel::scrollbar_jump_value`/`scrollbar_drag_value`.
    pub fn scroll_to(&mut self, value: f32, max_scroll: f32) {
        self.scroll = value.clamp(0.0, max_scroll.max(0.0));
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
