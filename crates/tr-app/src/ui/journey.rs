//! The Journey powers menu — `CreativeUI` (`GameContent/Creative/CreativeUI.cs`)
//! + `UICreativePowersMenu` (`GameContent/UI/States/UICreativePowersMenu.cs`) +
//! `UICreativeInfiniteItemsDisplay` (`GameContent/UI/Elements/`), drawn as
//! static chrome over the open inventory (W4-c24).
//!
//! Phase 1: the toggle button, the main power strip's category buttons, the
//! research page (the 170x170 sacrifice panel, the cog trio, the piston strip,
//! the sacrifice slot, the progress text and the Research confirm button) and
//! the sacrifice animation. Phase 2 adds the duplication grid; Phase 3 the
//! time/weather/personal strips and the power toggles.
//!
//! Geometry: children are laid out against their parent's *inner* dimensions
//! (`UIElement.cs:287` `Recalculate`), the `ui::crafting` convention.

use glam::Vec2;
use tr_render::{SheetRect, SpriteRenderer};

use super::color;
use super::in_rect;
use super::panel;
use super::slot;
use super::text::{draw_border_string, hud_text_width, line_height, BorderFont};
use super::uistate::draw_spliced_panel;
use crate::assets;
use crate::craft_filter::FilterBoard;
use crate::creative::CreativePowers;
use crate::item::{ItemSlot, EMPTY_SLOT};
use crate::research::ResearchTracker;

/// `UICreativePowersMenu.OpenMainSubCategory` (`UICreativePowersMenu.cs:29-38`).
/// The discriminants are the C#'s (`None 0, InfiniteItems 1, ResearchWindow 2,
/// Time 3, Weather 4, EnemyStrengthSlider 5, PersonalPowers 6`).
pub const CATEGORY_NONE: u8 = 0;
pub const CATEGORY_DUPLICATION: u8 = 1;
pub const CATEGORY_RESEARCH: u8 = 2;
pub const CATEGORY_TIME: u8 = 3;
pub const CATEGORY_WEATHER: u8 = 4;
pub const CATEGORY_PERSONAL: u8 = 6;

/// `Main.inventoryScale` while the duplication grid draws
/// (`UIDynamicItemCollection.cs:40`: `3465f / 4095f`).
pub const GRID_SCALE: f32 = 3465.0 / 4095.0;
/// `Main.inventoryScale` with the inventory open (`Main.cs:41026` family) —
/// the sacrifice slot's draw scale (`UIItemSlot.cs:37`).
pub const INVENTORY_SCALE: f32 = 0.85;

/// One `CreativeSacrificeParticle` (`Graphics/Renderers/
/// CreativeSacrificeParticle.cs:29-50`): the piston spark. `_scale` starts at
/// 0.6 (`:38`) and shrinks by `ScaleOffsetPerFrame = -1/60`
/// (`UICreativeInfiniteItemsDisplay.cs:362`); the particle dies at scale <= 0
/// (`StopWhenBelowXScale = 0`, `:36`, `:46-49`).
#[derive(Clone, Copy, Debug)]
pub struct Spark {
    pub pos: [f32; 2],
    pub vel: [f32; 2],
    pub scale: f32,
}

/// The menu's live state — `CreativeUI`'s fields plus
/// `UICreativePowersMenu`'s `MenuTree.CurrentOption`s and
/// `UICreativeInfiniteItemsDisplay`'s animation state. Owned by `App`.
#[derive(Clone, Debug)]
pub struct JourneyMenu {
    /// `CreativeUI.Enabled` (`CreativeUI.cs:45`).
    pub enabled: bool,
    /// `_mainCategory.CurrentOption` (`UICreativePowersMenu.cs:75`).
    pub category: u8,
    /// `_timeCategory.CurrentOption` (`:79`): 0 none, 1 the rate slider.
    pub time_sub: u8,
    /// `_weatherCategory.CurrentOption` (`:77`): 0 none, 1 wind, 2 rain.
    pub weather_sub: u8,
    /// `_personalCategory.CurrentOption` (`:81`): 0 none, 1 spawn rate.
    pub personal_sub: u8,
    /// `CreativeUI._itemSlotsForUI[0]` (`CreativeUI.cs:37`) — the sacrifice
    /// slot's item (`ItemSlotIndexes_SacrificeItem = 0`, `:27`).
    pub sacrifice: ItemSlot,
    /// `_sacrificeAnimationTimeLeft` (`UICreativeInfiniteItemsDisplay.cs:61`):
    /// 60 at `BeginSacrificeAnimation` (`:482-485`), decremented per UI update
    /// (`:487-493`).
    pub anim_left: i32,
    /// `_researchComplete` (`:63`): the completion chime's latch (`:366-370`).
    pub research_complete: bool,
    /// The cog trio's rotations (`_sacrificeCogsSmall/Medium/Big`, `:49-53`):
    /// each pair's first cog carries +rot, the second -rot
    /// (`OffsetRotationsForCogs`, `:375-379`).
    pub cog_rot: [f32; 3],
    /// `_lastItemIdSacrificed/_lastItemAmountWeHad/_lastItemAmountWeNeededTotal`
    /// (`:67-71`, `RememberItemSacrifice` `:468-473`).
    pub remembered: (u16, u32, u32),
    /// `_pistonParticleSystem`'s particles (`:57`, `:351-364`).
    pub sparks: Vec<Spark>,
    /// The duplication page's `EntryFilterer` — the same eleven
    /// `ItemFilters` in the same order as `NewCraftingUI`
    /// (`UICreativeInfiniteItemsDisplay.cs:78-94`), so the crafting port's
    /// [`FilterBoard`] is the shared seam.
    pub filters: FilterBoard,
    /// The duplication list's `UIScrollbar._viewPosition`, in px.
    pub scroll: f32,
    /// `UIScrollbar._isDragging`'s `_dragYOffset` (`UIScrollbar.cs:131-135`).
    pub scroll_drag: Option<f32>,
    /// `Main.CreativeMenuMouseOver` (`CreativeUI.cs:171`'s
    /// `DoStatefulTickSound` latch).
    pub toggle_hovered: bool,
    /// `GroupOptionButton._soundedHover` (`GroupOptionButton.cs:40,212-223`)
    /// collapsed to one latch: the strip-global button id last hovered
    /// (-1 none). Only one button can be hovered at a time.
    pub hovered_button: i32,
    /// `ModifyTimeRate._sliderCurrentValueCache` (`CreativePowers.cs:873`):
    /// the raw [0,1] slider value; the derived 1..24 rate lives in
    /// `CreativePowers` (`PowerId::TimeRate`).
    pub time_rate_cache: f32,
    /// `ModifyWindDirectionAndStrength._sliderCurrentValueCache` — the port
    /// keeps no cache: `GetSliderValueInner` reads the LIVE wind target
    /// (`CreativePowers.cs:1193-1196`), so the draw derives the position
    /// from `Wind.speed_target` per frame (the walk moves it under the
    /// slider, exactly like the C#).
    /// `ModifyRainPower._sliderCurrentValueCache` — same: the slider reads
    /// the live `Main.cloudAlpha` (`:1332-1335`), so the draw derives from
    /// `Rain.cloud_alpha()`.
    /// `DifficultySliderPower._sliderCurrentValueCache` (`:1014`): the raw
    /// [0,1] cache; the derived strength lives in `CreativePowers`
    /// (`PowerId::Difficulty`). The C# reads the cache back (`GetSliderValue
    /// Inner` is not overridden), so the port keeps it.
    pub difficulty_cache: f32,
    /// `SpawnRateSliderPerPlayerPower._sliderCurrentValueCache` (`:1580`'s
    /// default 0.5): the raw [0,1] cache, mirrored in `CreativePowers`
    /// (`PowerId::SpawnRate`) for the sim.
    pub spawn_rate_cache: f32,
    /// `UISearchBar.actualContents` (`UISearchBar.cs:17`) — the duplication
    /// search box's text (`UIWrappedSearchBar._searchString`, `:32`).
    /// Empty = the "Name:" placeholder (`UI.PlayerNameSlot`).
    pub search: String,
    /// `UISearchBar.IsWritingText` (`:25`): the box takes keyboard input
    /// (`Main.GetInputText`, `Main.cs:20010-20121` — the port's chat-style
    /// `event.text` seam, `app/loop.rs`).
    pub search_writing: bool,
}

impl Default for JourneyMenu {
    fn default() -> Self {
        Self {
            enabled: false,
            category: CATEGORY_NONE,
            time_sub: 0,
            weather_sub: 0,
            personal_sub: 0,
            sacrifice: EMPTY_SLOT,
            anim_left: 0,
            research_complete: false,
            cog_rot: [0.0; 3],
            remembered: (0, 0, 0),
            sparks: Vec::new(),
            filters: FilterBoard::default(),
            scroll: 0.0,
            scroll_drag: None,
            toggle_hovered: false,
            hovered_button: -1,
            time_rate_cache: 0.0,
            difficulty_cache: 0.0,
            spawn_rate_cache: 0.5,
            search: String::new(),
            search_writing: false,
        }
    }
}

/// Every static rectangle of the menu, derived from `screen` exactly as the
/// `UIElement` layout pass does (`UIElement.cs:306-322`). Kept pure for tests.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Layout {
    /// The toggle button (`CreativeUI.cs:104,146-151`): at (28, 267), the
    /// `Journey_Toggle` sheet's own size; +52*inventoryScale on X when
    /// `screenHeight < 650 && Enabled` (`:108-111`).
    pub toggle: [f32; 4],
    /// `UICreativePowersMenu._container` (`:106-113`): Top 270,
    /// Height (-270-20, 1), full width.
    pub container: [f32; 4],
    /// The main power strip (`PowerStripUIElement.cs:17-29`): 48 wide,
    /// 44*n+4 tall, Left 20, VAlign 0.5 (`UICreativePowersMenu.cs:115-120`).
    pub main_strip: [f32; 4],
    /// `_infiniteItemsWindow` (`UICreativePowersMenu.cs:151-158`): Left 80,
    /// Width 480, Height (-88, 1), VAlign 0.5.
    pub window: [f32; 4],
    /// The research page's 170x170 panel
    /// (`UICreativeInfiniteItemsDisplay.cs:209-214`), VAlign 0.5, HAlign 0.
    pub research_panel: [f32; 4],
    /// The cog container (`:310-317`): 280x170 at the window's left.
    pub cogs_box: [f32; 4],
    /// The `Research_Slots` plate (`:221-227`): centred, Top -20.
    pub plate: [f32; 4],
    /// The sacrifice slot's 48x48 element box (`UIItemSlot.cs:20-21`,
    /// `UICreativeInfiniteItemsDisplay.cs:248-254`): centred in the plate,
    /// Top -15.
    pub slot: [f32; 4],
    /// The `ItemSlot.Draw` origin (`UIItemSlot.cs:37`): the element's centre
    /// minus (52,52)*0.5*inventoryScale.
    pub slot_draw: [f32; 2],
    /// The Research confirm button (`:265-273`): 124x30, HAlign 0.5,
    /// VAlign 1 inside the panel's inner rect.
    pub confirm: [f32; 4],
    /// The duplication page's background panel (`:167`, `:158-163`):
    /// Width fill, Height (-38, 1), Top 38.
    pub dup_panel: [f32; 4],
    /// The panel's inner rect (padding 12, `UIPanel.cs:32`).
    pub dup_inner: [f32; 4],
    /// The duplication `UIList` (`:179-185`): W (-25,1), H (-28,1),
    /// VAlign 1, HAlign 0.
    pub list: [f32; 4],
    /// The duplication `UIScrollbar` (`:187-194`): W 20, H (-28-8,1),
    /// Top -4, VAlign 1, HAlign 1.
    pub scrollbar: [f32; 4],
    /// The duplication filter strip (`:198-201`,
    /// `UICreativeItemsInfiniteFilteringOptions.cs:40-44`): Left 20, Top 4,
    /// 440x40 on the window.
    pub filter_strip: [f32; 4],
}

/// The main strip's button count (`CreateMainPowerStrip`,
/// `UICreativePowersMenu.cs:163-209`): duplication, research, time, weather,
/// personal, StopBiomeSpread, the difficulty-slider subcategory button.
pub const MAIN_BUTTONS: usize = 7;

/// `PowerStripUIElement`'s own rect (`PowerStripUIElement.cs:17-29`):
/// width 40+4*2, height 40*n + 4*(1+n), at `left`, vertically centred in
/// `container`.
pub fn strip_rect(container: [f32; 4], left: f32, count: usize) -> [f32; 4] {
    let w = 40.0 + 4.0 * 2.0;
    let h = 40.0 * count as f32 + 4.0 * (1 + count) as f32;
    [
        container[0] + left,
        container[1] + (container[3] - h) * 0.5,
        w,
        h,
    ]
}

/// The `i`-th button of a strip (`PowerStripUIElement.cs:34-40`): HAlign 0.5
/// (the 4 px side margin), Top 4 + 44*i, 40x40.
pub fn strip_button(strip: [f32; 4], i: usize) -> [f32; 4] {
    [strip[0] + 4.0, strip[1] + 4.0 + 44.0 * i as f32, 40.0, 40.0]
}

/// The layout at `screen`. `toggle_size`/`plate_size` are the sheets' own
/// sizes (`UIImage` resizes to the texture, `UIImage.cs:52-56`); the
/// fallbacks are the retail sizes (the toggle 36x36, the plate 74x74).
pub fn layout(
    screen: Vec2,
    enabled: bool,
    toggle_size: [f32; 2],
    plate_size: [f32; 2],
) -> Layout {
    // `CreativeUI.Draw` (`:104-112`): location (28, 267), shifted right by
    // 52*inventoryScale on a short screen while the menu is open.
    let mut tx = 28.0;
    if screen.y < 650.0 && enabled {
        tx += 52.0 * INVENTORY_SCALE;
    }
    let toggle = [tx, 267.0, toggle_size[0], toggle_size[1]];

    // `_container` (`UICreativePowersMenu.cs:104-113`): num = 270, num2 = 20.
    let container = [0.0, 270.0, screen.x, screen.y - 270.0 - 20.0];
    let main_strip = strip_rect(container, 20.0, MAIN_BUTTONS);

    // `_infiniteItemsWindow` (`:151-158`): Left 80, W 480, H (-88,1),
    // VAlign 0.5 — the window's top lands at container.top + 44.
    let win_h = container[3] - 88.0;
    let window = [80.0, container[1] + 44.0, 480.0, win_h];

    // The research panel (`UICreativeInfiniteItemsDisplay.cs:209-214`):
    // 170x170, VAlign 0.5, HAlign 0 (the window's left), Top default.
    let research_panel = [
        window[0],
        window[1] + (win_h - 170.0) * 0.5,
        170.0,
        170.0,
    ];
    // The cog container (`:310-317`): 280x170, same vertical centre.
    let cogs_box = [window[0], research_panel[1], 280.0, 170.0];
    // The plate (`:221-227`): HAlign/VAlign 0.5, Top -20 in the panel
    // (padding 0 — the plate is a UIImage child of the *panel*, whose inner
    // rect is inset 12: `UIPanel.cs:32`).
    let rp_inner = [
        research_panel[0] + 12.0,
        research_panel[1] + 12.0,
        146.0,
        146.0,
    ];
    let plate = [
        rp_inner[0] + (146.0 - plate_size[0]) * 0.5,
        rp_inner[1] + (146.0 - plate_size[1]) * 0.5 - 20.0,
        plate_size[0],
        plate_size[1],
    ];
    // The slot element (`:248-254`): 48x48 (`UIItemSlot.cs:20-21`), centred
    // in the plate (a UIImage, padding 0), Top -15.
    let slot_box = [
        plate[0] + (plate[2] - 48.0) * 0.5,
        plate[1] + (plate[3] - 48.0) * 0.5 - 15.0,
        48.0,
        48.0,
    ];
    // `UIItemSlot.DrawSelf` (`:37`): centre + (52,52)*-0.5*inventoryScale.
    let slot_draw = [
        slot_box[0] + 24.0 - 26.0 * INVENTORY_SCALE,
        slot_box[1] + 24.0 - 26.0 * INVENTORY_SCALE,
    ];
    // The confirm button (`:265-273`): 124x30, HAlign 0.5, VAlign 1, Top 0.
    let confirm = [
        rp_inner[0] + (146.0 - 124.0) * 0.5,
        rp_inner[1] + 146.0 - 30.0,
        124.0,
        30.0,
    ];

    // The duplication panel (`:167`, `:158-163`): fill width, H (-38,1),
    // Top 38; padding 12.
    let dup_panel = [window[0], window[1] + 38.0, 480.0, win_h - 38.0];
    let dup_inner = [
        dup_panel[0] + 12.0,
        dup_panel[1] + 12.0,
        dup_panel[2] - 24.0,
        dup_panel[3] - 24.0,
    ];
    // The list (`:179-185`): W (-25,1), H (-28,1), VAlign 1, HAlign 0.
    let list = [
        dup_inner[0],
        dup_inner[1] + 28.0,
        dup_inner[2] - 25.0,
        dup_inner[3] - 28.0,
    ];
    // The scrollbar (`:187-194`): num = 4, H (-28-8,1), Top -4, VAlign 1,
    // HAlign 1.
    let scrollbar = [
        dup_inner[0] + dup_inner[2] - 20.0,
        dup_inner[1] + 36.0 - 4.0,
        20.0,
        dup_inner[3] - 36.0,
    ];
    // The filter strip (`:198-201`): Left 20 on the window, Top 4,
    // 40*11 wide, 40 tall.
    let filter_strip = [window[0] + 20.0, window[1] + 4.0, 440.0, 40.0];

    Layout {
        toggle,
        container,
        main_strip,
        window,
        research_panel,
        cogs_box,
        plate,
        slot: slot_box,
        slot_draw,
        confirm,
        dup_panel,
        dup_inner,
        list,
        scrollbar,
        filter_strip,
    }
}

/// The sheet sizes the layout needs, with the retail fallbacks.
pub fn sheet_sizes(lookups: &assets::Lookups) -> ([f32; 2], [f32; 2]) {
    let size = |s: Option<SheetRect>, dw: f32, dh: f32| {
        s.map_or([dw, dh], |s| [s.width as f32, s.height as f32])
    };
    (
        size(lookups.ui_journey_toggle, 36.0, 36.0),
        size(lookups.ui_research_slots, 74.0, 74.0),
    )
}

/// `CreativeUI.Blocked` (`CreativeUI.cs:47-65`): talking blocks; the visible
/// crafting panel unblocks; an open chest blocks. The tile-entity anchor arm
/// (`:61`) has no port surface (no in-use TE UI), so it reads false.
pub fn blocked(talk_npc: bool, chest_open: bool, crafting_open: bool) -> bool {
    if talk_npc {
        return true;
    }
    if crafting_open {
        return false;
    }
    chest_open
}

/// The main strip's `i`-th button's category option
/// (`CreateMainPowerStrip`, `UICreativePowersMenu.cs:173-207`): the buttons
/// carry options 1, 2, 3, 4, 6, then StopBiomeSpread (a toggle, no category)
/// and the difficulty slider (option 5).
pub const MAIN_OPTIONS: [u8; MAIN_BUTTONS] = [1, 2, 3, 4, 6, 0, 5];

/// The main strip's icon frames (`CreativePowersHelper.CreativePowerIconLocations`,
/// `CreativePowersHelper.cs:64-86`): ItemDuplication 0, ItemResearch 1,
/// TimeCategory 2, WeatherCategory 3, PersonalCategory 20, StopBiomeSpread 8,
/// EnemyStrengthSlider 4.
pub const MAIN_ICONS: [usize; MAIN_BUTTONS] = [0, 1, 2, 3, 20, 8, 4];

/// `ToggleCategory` (`UICreativePowersMenu.cs:301-312`): clicking the open
/// category closes it, any other click opens it.
pub fn toggle_category(current: u8, option: u8) -> u8 {
    if current == option {
        CATEGORY_NONE
    } else {
        option
    }
}

/// `descriptionText_OnUpdate` (`UICreativeInfiniteItemsDisplay.cs:404-434`):
/// the progress label. The remembered-number arms read the state AFTER the
/// non-air-cursor forget (`:409-412`); the caller applies that forget first.
pub fn progress_text(
    sacrifice: &ItemSlot,
    remembered: (u16, u32, u32),
    tracker: &ResearchTracker,
) -> String {
    if sacrifice.count == 0 || sacrifice.kind == 0 {
        // `:413-423`: the empty slot shows the remembered partial numbers,
        // else "???".
        let (id, had, needed) = remembered;
        if id != 0 && needed != had {
            return format!("({had}/{needed})");
        }
        return "???".to_string();
    }
    // `:425-433`: a filled slot forgets the memory and prints its numbers,
    // "X" for the capless kinds.
    match tracker.sacrifice_numbers(sacrifice.kind) {
        None => "X".to_string(),
        Some((have, needed)) => format!("({have}/{needed})"),
    }
}

/// `GetSacrificeAnimationProgress` (`UICreativeInfiniteItemsDisplay.cs:495-498`):
/// `Utils.GetLerpValue(60, 0, timeLeft, clamped)` (`Utils.cs:315-343`).
pub fn anim_progress(time_left: i32) -> f32 {
    ((time_left as f32 - 60.0) / (0.0 - 60.0)).clamp(0.0, 1.0)
}

/// `UpdateVisualFrame`'s piston frame pick (`UICreativeInfiniteItemsDisplay.cs:341-346`):
/// 0 at rest, else the 8-step ladder over the progress.
pub fn piston_frame(time_left: i32) -> usize {
    if time_left == 0 {
        return 0;
    }
    let p = anim_progress(time_left);
    let num4 = 0.1f32;
    let num5 = 1.0f32 / 15.0;
    if p >= 1.0 - num4 {
        8
    } else if p >= 1.0 - num4 * 2.0 {
        7
    } else if p >= 1.0 - num4 * 3.0 {
        6
    } else if p >= num5 * 4.0 {
        5
    } else if p >= num5 * 3.0 {
        4
    } else if p >= num5 * 2.0 {
        3
    } else if p >= num5 {
        2
    } else {
        1
    }
}

/// `UpdateVisualFrame`'s cog step (`UICreativeInfiniteItemsDisplay.cs:330-340`):
/// `num = 0.05 * (1 + lerp^2 * 2)` with `lerp = GetLerpValue(1, 0.7, progress,
/// clamped)`; small += 2*num, medium += 1.1428572*num, big -= num.
pub fn cog_step(time_left: i32) -> [f32; 3] {
    let p = anim_progress(time_left);
    // `GetLerpValue(1f, 0.7f, p, clamped)` — from > to, so the clamp arms
    // invert (`Utils.cs:330-340`).
    let lerp = ((p - 1.0) / (0.7 - 1.0)).clamp(0.0, 1.0);
    let num = 0.05 * (1.0 + lerp * lerp * 2.0);
    [2.0 * num, 1.1428572 * num, -num]
}

/// One UI update of the research animation, the
/// `UICreativeInfiniteItemsDisplay_OnUpdate` tail (`:505-529`) +
/// `sacrificeWindow_OnUpdate`/`UpdateVisualFrame` (`:325-373`). The cog spin
/// and the piston/spark half run only while the research page shows (the
/// sacrifice container is only in the tree then, `:520-527`); the timer
/// decrements on any page. `play` receives the cue name at its exact site
/// (the thunk at timeLeft 56 — its `Next(1, 4)` bank pick,
/// `LegacySoundPlayer.cs:482`, draws BEFORE the 15 spark draws,
/// `UICreativeInfiniteItemsDisplay.cs:349-364`; the completion chime at 40),
/// so the caller's RNG interleave matches the C#'s. `rng` is the
/// bare-`Main.rand` stand-in.
pub fn tick_animation(
    m: &mut JourneyMenu,
    research_page: bool,
    rng: &mut tr_core::UnifiedRandom,
    play: &mut dyn FnMut(&str),
) {
    if research_page {
        // The cog rotations (`:332-340`).
        let step = cog_step(m.anim_left);
        for i in 0..3 {
            m.cog_rot[i] += step[i];
        }
        if m.anim_left != 0 {
            if m.anim_left == 56 {
                // `SoundEngine.PlaySound(63)` (`:349`) — the bank pick first.
                let k = rng.next_range(1, 4);
                play(&format!("Research_{k}"));
                // 15 sparks (`:351-364`): NextVector2Circular(4,3) = unit
                // vector * (4,3) * NextFloat (`Utils.cs:1266-1269` — the
                // angle draws first, `NextVector2Unit` `:1261-1264`), the
                // positive Y flipped, then -2; accel (0, 0.1635), scale
                // offset -1/60.
                for _ in 0..15 {
                    let angle = std::f32::consts::TAU * rng.next_float();
                    let radius = rng.next_float();
                    let (s, c) = angle.sin_cos();
                    let vx = c * 4.0 * radius;
                    let mut vy = s * 3.0 * radius;
                    if vy > 0.0 {
                        vy = -vy;
                    }
                    vy -= 2.0;
                    m.sparks.push(Spark {
                        pos: [0.0, 0.0],
                        vel: [vx, vy],
                        scale: 0.6,
                    });
                }
            }
            if m.anim_left == 40 && m.research_complete {
                m.research_complete = false;
                play("Research_0"); // `:366-370`, `SoundResearch[0]`
            }
        }
        // The spark integration (`CreativeSacrificeParticle.Update`,
        // `:41-50`): velocity += accel, position += velocity, scale -= 1/60,
        // removed at scale <= 0.
        for p in m.sparks.iter_mut() {
            p.vel[1] += 0.16350001;
            p.pos[0] += p.vel[0];
            p.pos[1] += p.vel[1];
            p.scale -= 1.0 / 60.0;
        }
        m.sparks.retain(|p| p.scale > 0.0);
    }
    // `UpdateSacrificeAnimation` (`:487-493`) — outside the page gate.
    if m.anim_left > 0 {
        m.anim_left -= 1;
    }
}

/// `StopPlayingAnimation` (`UICreativeInfiniteItemsDisplay.cs:460-466`).
pub fn stop_animation(m: &mut JourneyMenu) {
    m.remembered = (0, 0, 0);
    m.anim_left = 0;
    m.sparks.clear();
}

/// `CreativeUI.SacrificeItem`'s amount maths (`CreativeUI.cs:288-329`), pure:
/// `(sacrificed, done)` or `None` for `CannotSacrifice`. `needed =
/// Clamp(cap - have, 0, cap)`; 0 needed refuses; `take = Min(needed, stack)`;
/// done when take == needed.
pub fn sacrifice_amounts(
    kind: u16,
    stack: u32,
    tracker: &ResearchTracker,
) -> Option<(u32, bool)> {
    let (have, cap) = tracker.sacrifice_numbers(kind)?;
    let needed = (cap - have).min(cap); // the Clamp's lower bound 0 is inert (have <= cap)
    if needed == 0 {
        return None;
    }
    let take = needed.min(stack);
    Some((take, take == needed))
}

/// `SacrificeWhatYouCan` (`UICreativeInfiniteItemsDisplay.cs:441-458`) +
/// `CreativeUI.SacrificeItem` (`CreativeUI.cs:288-329`): registers the
/// sacrifice, drains the slot, starts the animation and remembers the
/// numbers. The excess-return arm (`:320-323`, `Player.GetItem`) is the
/// caller's — it needs the inventory. Returns the legacy cue ids the C#
/// plays here (none — the thunk/chime ride the animation) and whether an
/// excess stack remains for the caller to return.
pub fn sacrifice_confirm(m: &mut JourneyMenu, tracker: &mut ResearchTracker) -> bool {
    let kind = m.sacrifice.kind;
    if m.sacrifice.count == 0 {
        return false;
    }
    let Some((take, done)) = sacrifice_amounts(kind, m.sacrifice.count, tracker) else {
        return false;
    };
    let (have, cap) = tracker.sacrifice_numbers(kind).unwrap_or((0, 0));
    // `:308-313`: the net report is single-player inert; the local register
    // runs (`Main.ServerSideCharacter` false).
    tracker.register_sacrifice(kind, take);
    m.sacrifice.count -= take;
    if m.sacrifice.count == 0 {
        m.sacrifice = EMPTY_SLOT; // `TurnToAir` (`:315-318`)
    }
    // `:447-456`: both success arms animate + remember.
    m.research_complete = done;
    m.anim_left = 60; // `BeginSacrificeAnimation` (`:482-485`)
    m.remembered = (kind, have + take, cap);
    m.sacrifice.count > 0
}

// ---- the duplication grid (Phase 2) ------------------------------------------

/// `UpdateContents` (`UICreativeInfiniteItemsDisplay.cs:536-549`) + the
/// sorter (`:96-102`): every kind with positive progress that passes the
/// filterer — the search filter FIRST (`EntryFilterer.FitsFilter`,
/// `EntryFilterer.cs:30-35`), then the active tab — sorted `ByUnlockStatus`
/// (newly-researched first, then fully researched — `SortingSteps.cs:91-113`),
/// then `ByCreativeSortingId`, then `Alphabetical` (`:143-146`).
///
/// Deviation (docs/ANALYSIS.md): `ContentSamples.ItemCreativeSortingId`
/// (`SortingSteps.cs:125-131`) is a generated group/order table the port does
/// not carry; the middle step falls through to the alphabetical tie-break.
pub fn grid_items(
    tracker: &ResearchTracker,
    filters: &FilterBoard,
    search: &str,
    player: Option<&crate::player::Player>,
    journey_difficulty: u8,
) -> Vec<u16> {
    let mut items: Vec<u16> = tracker
        .items_with_progress()
        .filter(|&id| {
            search_fits(id, search, player, journey_difficulty, tracker) && filters.fits(id)
        })
        .collect();
    items.sort_by(|&a, &b| {
        let rank = |id: u16| {
            (
                !tracker.is_newly_researched(id),
                !tracker.is_fully_researched(id),
            )
        };
        rank(a)
            .cmp(&rank(b))
            .then_with(|| crate::lang::item_name(a).cmp(&crate::lang::item_name(b)))
    });
    items
}

/// `ItemFilters.BySearch.FitsFilter` (`ItemFilters.cs:28-48`) + the
/// `EntryFilterer.SetSearchFilter` whitespace gate (`EntryFilterer.cs:85-94`):
/// a null/whitespace search passes everything; else the item's tooltip lines
/// (`Main.MouseText_DrawItemTooltip_GetLinesInfo` at stack 1, `:34-39`) are
/// scanned case-insensitively for the substring. The port's tooltip seam is
/// `ui::tooltip::item_tooltip_lines` (the same rows, the player's layers
/// folded in); the 30-line cap is the C#'s `_toolTipLines` buffer.
pub fn search_fits(
    kind: u16,
    search: &str,
    player: Option<&crate::player::Player>,
    journey_difficulty: u8,
    tracker: &ResearchTracker,
) -> bool {
    if search.trim().is_empty() {
        return true; // `SetSearchFilter`'s IsNullOrWhiteSpace arm
    }
    let Some(d) = crate::registry::get(kind) else {
        return false;
    };
    let lines = super::tooltip::item_tooltip_lines(
        kind,
        d,
        super::tooltip::TooltipContext {
            prefix: 0,
            stack: 1, // `entry.stack = 1` (`ItemFilters.cs:37`)
            player,
            held_crit: 0,
            // W4-c24c: the search entry is a fresh `new Item()` whose
            // `tooltipContext` stays -1 (`Item.cs:117`, `UICreativeItemGrid`
            // never sets it on the filter entries) — the Journey sacrifice
            // row (`Main.cs:21318`) is OUTSIDE the whitelist here, but the
            // difficulty + tracker ride for the row set's other gates.
            // (W4-f28 compose: the context rides `HoverFlags.context`, the
            // difficulty `TooltipContext.difficulty`.)
            flags: super::tooltip_lines2::HoverFlags { context: -1, ..Default::default() },
            difficulty: journey_difficulty,
            research: Some(tracker),
            // The search-entry probe draws no shop/inventory/world context
            // (a fresh `new Item()`, `ItemFilters.cs:37`).
            ..Default::default()
        },
    );
    let needle = search.to_lowercase();
    lines
        .iter()
        .take(30) // `_tooltipMaxLines` (`:18`)
        .any(|(text, _)| text.to_lowercase().contains(&needle))
}

/// `UIDynamicItemCollection.UpdateSize` (`UIDynamicItemCollection.cs:160-165`):
/// `_itemsPerLine = Width / 44` (integer), the grid's MinHeight 44*rows.
pub fn grid_cols(list: [f32; 4]) -> usize {
    ((list[2] as i32 / 44) as usize).max(1)
}

/// The grid's total height (`UpdateSize`'s `44 * ceil(count / cols)`).
pub fn grid_height(count: usize, cols: usize) -> f32 {
    44.0 * (count as f32 / cols as f32).ceil()
}

/// The `i`-th cell (`GetItemSlotHitbox`, `UIDynamicItemCollection.cs:75-79` +
/// `GetGridParameters` `:81-97`): 44x44, the row lattice centred on the list
/// (`startX = center.X - 44*cols/2`), scrolled by `scroll` px.
pub fn grid_cell(list: [f32; 4], scroll: f32, i: usize) -> [f32; 4] {
    let cols = grid_cols(list);
    let center_x = (list[0] + list[2] * 0.5) as i32 as f32;
    let start_x = center_x - (44 * cols) as f32 * 0.5;
    let col = (i % cols) as f32;
    let row = (i / cols) as f32;
    [start_x + col * 44.0, list[1] - scroll + row * 44.0, 44.0, 44.0]
}

/// The hovered grid index, clipped to the list's scissor
/// (`UIList.OverflowHidden`, `UIList.cs:59`).
pub fn grid_hit(list: [f32; 4], scroll: f32, count: usize, mouse: Vec2) -> Option<usize> {
    if !in_rect(mouse, list) {
        return None;
    }
    (0..count).find(|&i| in_rect(mouse, grid_cell(list, scroll, i)))
}

/// `UICreativeItemGrid.DrawSlot`'s context pick (`UICreativeItemGrid.cs:21`):
/// 29 fully researched, 34 partial.
pub fn grid_context(tracker: &ResearchTracker, kind: u16) -> i32 {
    if tracker.is_fully_researched(kind) {
        29
    } else {
        34
    }
}

/// `ItemSlot.Draw`'s context tints (`ItemSlot.cs:2253-2259`): 29
/// `(53,69,127,255)`, 34 `(25,44,65,180) * 0.9f` — both on
/// `Inventory_Back18`.
pub fn grid_tint(context: i32) -> [f32; 4] {
    if context == 29 {
        color::tint([53, 69, 127, 255])
    } else {
        color::tint(color::mul([25, 44, 65, 180], 0.9))
    }
}

// ---- drawing ------------------------------------------------------------------

/// Pushes one sprite cut out of `sheet` by `src` (the sheet's own pixels) at
/// `dst` — the `ui::crafting` helper shape.
fn push_sheet_rect(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    sheet: Option<&SheetRect>,
    dst: [f32; 4],
    src: [f32; 4],
    tint: [f32; 4],
) {
    let Some(sheet) = sheet else { return };
    if dst[2] <= 0.0 || dst[3] <= 0.0 {
        return;
    }
    let (aw, ah) = (lookups.atlas_size.0 as f32, lookups.atlas_size.1 as f32);
    let x0 = sheet.x as f32 + src[0];
    let y0 = sheet.y as f32 + src[1];
    let uv = [x0 / aw, y0 / ah, (x0 + src[2]) / aw, (y0 + src[3]) / ah];
    r.push_ui_sprite([dst[0], dst[1]], [dst[2], dst[3]], uv, tint);
}

/// `UIText.DrawSelf`'s shadowed label (`UIText.cs:138,158`): the draw origin
/// shifts up 2*scale, the shadow spreads 1.5 in four directions.
fn ui_text(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    x: f32,
    y: f32,
    scale: f32,
    tint: [f32; 4],
    text: &str,
) {
    draw_border_string(
        r,
        lookups,
        BorderFont::Ui,
        x,
        y - 2.0 * scale,
        scale,
        &[[-1.5, 0.0], [1.5, 0.0], [0.0, -1.5], [0.0, 1.5]],
        tint,
        text,
    );
}

/// `GroupOptionButton.DrawSelf` (`GroupOptionButton.cs:210-251`) for the
/// creative buttons: the spliced `PanelGrayscale` base
/// (`Lerp(Black, color, FadeFromBlack=1) * opacity`), no selected highlight
/// (`ShowHighlightWhenSelected = false`, `CreativePowersHelper.cs:119,131,142`),
/// the `CategoryPanelBorder` hover frame, then the icon child
/// (`GetIconImage`, `CreativePowersHelper.cs:101-112`: MarginLeft/Top 4,
/// HAlign 1, VAlign 0.5; the icon dims to `Lerp(color, White, 0.7) * opacity`
/// when neither hovered nor selected, `GroupOptionButton.cs:244-248`).
#[allow(clippy::too_many_arguments)]
fn draw_power_button(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    rect: [f32; 4],
    icon: usize,
    selected: bool,
    hovered: bool,
    picked: [u8; 4],
    unpicked: [u8; 4],
    opacity_picked: f32,
    opacity_unpicked: f32,
) {
    let base = if selected { picked } else { unpicked };
    let opacity = if selected { opacity_picked } else { opacity_unpicked };
    // `Utils.DrawSplicedPanel(..., 10, 10, 10, 10, Lerp(Black, color, 1) * num)`
    // (`:233`) — FadeFromBlack is 1, so the lerp is the colour itself.
    draw_spliced_panel(
        r,
        lookups,
        lookups.cc_panel_gray,
        rect,
        10.0,
        10.0,
        10.0,
        10.0,
        color::tint(color::mul(base, opacity)),
    );
    if hovered {
        // `:238-241`: the border at `_borderColor` — White for the
        // single-player unlocked powers (`UpdateUnlockStateByPowerInternal`,
        // `CreativePowersHelper.cs:227`).
        draw_spliced_panel(
            r,
            lookups,
            lookups.cc_panel_border,
            rect,
            10.0,
            10.0,
            10.0,
            10.0,
            [1.0; 4],
        );
    }
    // The icon: `Infinite_Powers` Frame(21, 1, k) — the full cell.
    let Some(sheet) = lookups.ui_infinite_powers else {
        return;
    };
    let cw = sheet.width as f32 / 21.0;
    let ch = sheet.height as f32;
    // MarginLeft/Top 4, HAlign 1, VAlign 0.5 on the 40x40 button.
    let ix = rect[0] + 4.0 + (40.0 - 4.0 - cw);
    let iy = rect[1] + 4.0 + (40.0 - 4.0 - ch) * 0.5;
    let tint = if hovered || selected {
        [1.0; 4]
    } else {
        // `Lerp(color, White, _whiteLerp = 0.7) * num` (`:247`).
        color::tint(color::mul(
            super::char_creation::lerp_rgb(base, [255, 255, 255, 255], 0.7),
            opacity,
        ))
    };
    push_sheet_rect(
        r,
        lookups,
        Some(&sheet),
        [ix, iy, cw, ch],
        [cw * icon as f32, 0.0, cw, ch],
        tint,
    );
}

/// The category buttons' colours (`CreateCategoryButton`,
/// `CreativePowersHelper.cs:137-146`): picked (152,175,235) at 1.0, unpicked
/// `Colors.InventoryDefaultColor` (63,65,151) at 0.7 (`Colors.cs:158`).
const CATEGORY_PICKED: [u8; 4] = [152, 175, 235, 255];
const INVENTORY_DEFAULT: [u8; 4] = [63, 65, 151, 255];
/// `Main.OurFavoriteColor` (`Main.cs:66748`) — the toggle powers' picked
/// colour (`ASharedTogglePower.ProvidePowerButtons`, `CreativePowers.cs:504`).
const FAVORITE: [u8; 4] = [255, 231, 69, 255];

/// `PowerStripUIElement`'s panel (`PowerStripUIElement.cs:22-33`): border
/// `(89,116,213) * 0.9`, background `(73,94,171) * 0.9`, padding 0.
fn draw_strip_panel(r: &mut SpriteRenderer, lookups: &assets::Lookups, strip: [f32; 4]) {
    let bg = color::tint(color::mul([73, 94, 171, 255], 0.9));
    let border = color::tint(color::mul([89, 116, 213, 255], 0.9));
    panel::draw_panel(r, lookups, lookups.ui_panel_bg, strip, bg);
    panel::draw_panel(r, lookups, lookups.ui_panel_border, strip, border);
}

/// The toggle button (`DrawToggleButton`, `CreativeUI.cs:146-176`): the base
/// sheet, then the hover border over it. The notification-icon arm
/// (`:172-175`) is teammate-only — single-player inert.
fn draw_toggle(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    l: &Layout,
    hovered: bool,
) {
    if let Some(s) = lookups.ui_journey_toggle {
        super::push_whole(r, lookups, &s, l.toggle, [1.0; 4]);
    }
    if hovered {
        if let Some(s) = lookups.ui_journey_toggle_hover {
            super::push_whole(r, lookups, &s, l.toggle, [1.0; 4]);
        }
    }
}

/// One cog pair (`AddSymetricalCogsPair`, `UICreativeInfiniteItemsDisplay.cs:381-402`):
/// the offsets already carry `-size/2` (`:384`), and `UIImage` with
/// `NormalizedOrigin = 0.5` rotates about `position + size/2` (`UIImage.cs:93-98`)
/// — so the cog centres land at `box + baseOffset` and its mirror at
/// `box.x + 280 - baseOffset.x`.
fn draw_cog_pair(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    l: &Layout,
    sheet: Option<SheetRect>,
    base: [f32; 2],
    rot: f32,
) {
    let Some(sheet) = sheet else { return };
    let (w, h) = (sheet.width as f32, sheet.height as f32);
    let Some(uv) = lookups.hud_uv(Some(&sheet)) else {
        return;
    };
    for (cx, rot) in [
        (l.cogs_box[0] + base[0], rot),
        (l.cogs_box[0] + 280.0 - base[0], -rot),
    ] {
        let cy = l.cogs_box[1] + base[1];
        // Rotate about the centre: shift the top-left by origin - R*origin
        // (the `ui::bestiary` pattern, `bestiary.rs:723-734`).
        let origin = [w * 0.5, h * 0.5];
        let (s, c) = rot.sin_cos();
        let tl = [
            cx - origin[0] + origin[0] - (origin[0] * c - origin[1] * s),
            cy - origin[1] + origin[1] - (origin[0] * s + origin[1] * c),
        ];
        r.push_ui_sprite_rot(tl, [w, h], uv, [1.0; 4], rot);
    }
}

/// The research page (`BuildSacrificeMenuContents`,
/// `UICreativeInfiniteItemsDisplay.cs:207-287`): the panel, the cogs behind
/// it, the plate + piston + slot, the progress text and the confirm button.
fn draw_research_page(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    l: &Layout,
    m: &JourneyMenu,
    tracker: &ResearchTracker,
    mouse: Vec2,
) {
    // The panel (`CreateBasicPanel`, `:149-156`): the default blue at *0.8.
    let bg = color::tint(color::mul(color::mul([63, 82, 151, 255], 0.7), 0.8));
    let border = color::tint(color::mul([0, 0, 0, 255], 0.8));
    // The cogs are appended FIRST (`:218`) — behind the plate, over the
    // panel? No: `AddCogsForSacrificeMenu(uIPanel)` appends the cog container
    // to the panel, so the panel background draws first, then the cogs.
    panel::draw_panel(r, lookups, lookups.ui_panel_bg, l.research_panel, bg);
    panel::draw_panel(r, lookups, lookups.ui_panel_border, l.research_panel, border);
    // The cog trio (`:319-322`): base offsets (22,1)/(1,28)/(5,5) + (-10,-10).
    draw_cog_pair(r, lookups, l, lookups.ui_research_gears[2], [12.0, -9.0], m.cog_rot[0]);
    draw_cog_pair(r, lookups, l, lookups.ui_research_gears[1], [-9.0, 18.0], m.cog_rot[1]);
    draw_cog_pair(r, lookups, l, lookups.ui_research_gears[0], [-5.0, -5.0], m.cog_rot[2]);
    // The plate (`:221-228`).
    if let Some(s) = lookups.ui_research_slots {
        super::push_whole(r, lookups, &s, l.plate, [1.0; 4]);
    }
    // The piston strip (`:229-239`): Frame(1, 9, 0, frameY) at the plate's
    // position.
    if let Some(s) = lookups.ui_research_pistons {
        let fh = s.height as f32 / 9.0;
        let fy = piston_frame(m.anim_left);
        push_sheet_rect(
            r,
            lookups,
            Some(&s),
            [l.plate[0], l.plate[1], s.width as f32, fh],
            [0.0, fh * fy as f32, s.width as f32, fh],
            [1.0; 4],
        );
        // The sparks (`UIParticleLayer` anchored at the piston frame's
        // centre, `:240-247`; `CreativeSacrificeParticle.Draw`, `:52-55`:
        // origin frame/2, `Lerp(White, transparent, GetLerpValue(0.1, 0.5,
        // scale))` — Color.Lerp clamps).
        if let Some(spark) = lookups.ui_research_spark {
            let anchor = [l.plate[0] + s.width as f32 * 0.5, l.plate[1] + fh * 0.5];
            let (sw, sh) = (spark.width as f32, spark.height as f32);
            if let Some(uv) = lookups.hud_uv(Some(&spark)) {
                for p in &m.sparks {
                    let t = ((p.scale - 0.1) / (0.5 - 0.1)).clamp(0.0, 1.0);
                    let alpha = 1.0 - t;
                    let (dw, dh) = (sw * p.scale, sh * p.scale);
                    r.push_ui_sprite(
                        [anchor[0] + p.pos[0] - dw * 0.5, anchor[1] + p.pos[1] - dh * 0.5],
                        [dw, dh],
                        uv,
                        [1.0, 1.0, 1.0, alpha],
                    );
                }
            }
        }
    }
    // The sacrifice slot (`UIItemSlot.DrawSelf`, `:33-38`): context 30 draws
    // NO background (`ItemSlot.cs:2261-2263`, flag3 = !flag2 with no gamepad).
    slot::draw_slot(
        r,
        lookups,
        &m.sacrifice,
        l.slot_draw[0],
        l.slot_draw[1],
        30,
        0,
        false,
        INVENTORY_SCALE,
        [1.0; 4],
        super::UiPulse::default(),
        // ctx 30 draws no background and takes no CoinSlot state; the cursor
        // is treated as air (no dim) and carries no dye.
        None,
        -1,
        true,
        false,
    );
    // The progress text (`:255-264`): scale 0.8, Top 10, centred in the
    // panel's inner rect.
    let label = progress_text(&m.sacrifice, m.remembered, tracker);
    let tw = hud_text_width(r, lookups, &label, 0.8);
    let th = line_height(lookups) * 0.8;
    let inner = [l.research_panel[0] + 12.0, l.research_panel[1] + 12.0, 146.0, 146.0];
    ui_text(
        r,
        lookups,
        inner[0] + (146.0 - tw) * 0.5,
        inner[1] + (146.0 - th) * 0.5 + 10.0,
        0.8,
        [1.0; 4],
        &label,
    );
    // The confirm button (`:265-285`): a default UIPanel, border Black at
    // rest / `Colors.FancyUIFatButtonMouseOver` = OurFavoriteColor on hover
    // (`FadedMouseOver/Out`, `:297-306`, `Colors.cs:157`).
    let hovered = in_rect(mouse, l.confirm);
    let cbg = color::tint(color::mul([63, 82, 151, 255], 0.7));
    let cborder = if hovered {
        color::tint(FAVORITE)
    } else {
        [0.0, 0.0, 0.0, 1.0]
    };
    panel::draw_panel(r, lookups, lookups.ui_panel_bg, l.confirm, cbg);
    panel::draw_panel(r, lookups, lookups.ui_panel_border, l.confirm, cborder);
    let text = crate::lang::text("CreativePowers.ConfirmInfiniteItemSacrifice");
    let tw = hud_text_width(r, lookups, &text, 0.8);
    let th = line_height(lookups) * 0.8;
    // The label centres in the button's inner rect (padding 12 → 100x6).
    ui_text(
        r,
        lookups,
        l.confirm[0] + 12.0 + (100.0 - tw) * 0.5,
        l.confirm[1] + 12.0 + (6.0 - th) * 0.5,
        0.8,
        [1.0; 4],
        &text,
    );
}

/// The duplication page's chrome (Phase 2 fills the grid): the panel, the
/// search bar, the scrollbar and the Blue filter strip.
fn draw_duplication_page(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    l: &Layout,
    m: &JourneyMenu,
    tracker: &ResearchTracker,
    mouse: Vec2,
    player: Option<&crate::player::Player>,
) {
    let bg = color::tint(color::mul(color::mul([63, 82, 151, 255], 0.7), 0.8));
    let border = color::tint(color::mul([0, 0, 0, 255], 0.8));
    panel::draw_panel(r, lookups, lookups.ui_panel_bg, l.dup_panel, bg);
    panel::draw_panel(r, lookups, lookups.ui_panel_border, l.dup_panel, border);

    // The search bar (`UIWrappedSearchBar`, Blue theme — `:97-151`): the
    // magnifier (Button_Search), the (35,40,83) box, the text (the "Name:"
    // placeholder when empty, `UISearchBar.SetContents:72-82`) and the
    // cancel button. While writing, the box border turns OurFavoriteColor
    // (`OnStartTakingInput`, `:199-202`).
    let search = search_bar_rect(l);
    let button = search_button_rect(l);
    if let Some(s) = lookups.ui_search_icon_blue {
        super::push_whole(r, lookups, &s, button, [1.0; 4]);
    }
    if in_rect(mouse, button) {
        if let Some(s) = lookups.ui_search_border {
            super::push_whole(r, lookups, &s, button, [1.0; 4]);
        }
    }
    let search_box = search_box_rect(l);
    let box_w = search_box[2];
    let field = color::tint([35, 40, 83, 255]);
    let border_tint = if m.search_writing {
        color::tint(FAVORITE)
    } else {
        field
    };
    panel::draw_panel(r, lookups, lookups.ui_panel_bg, search_box, field);
    panel::draw_panel(r, lookups, lookups.ui_panel_border, search_box, border_tint);
    if m.search.is_empty() {
        // The empty placeholder (`UI.PlayerNameSlot`, Gray at 0.8 scale).
        ui_text(
            r,
            lookups,
            search_box[0] + 12.0,
            search_box[1] + 5.6,
            0.8,
            color::tint([128, 128, 128, 255]),
            &crate::lang::text("UI.PlayerNameSlot"),
        );
    } else {
        // The contents (`UISearchBar.SetContents`'s white arm, `:79-81`).
        ui_text(
            r,
            lookups,
            search_box[0] + 12.0,
            search_box[1] + 5.6,
            0.8,
            [1.0; 4],
            &m.search,
        );
    }
    let cancel = search_cancel_rect(l);
    let cancel_tint = if in_rect(mouse, cancel) {
        [1.0; 4]
    } else {
        color::tint(color::mul([255, 255, 255, 255], 0.4))
    };
    if let Some(s) = lookups.ui_search_cancel {
        super::push_whole(r, lookups, &s, cancel, cancel_tint);
    }

    // The grid (`UIDynamicItemCollection.DrawSelf`, `:38-71`): the items at
    // GRID_SCALE on the 44 px lattice, clipped to the list. The draw gate is
    // difficulty 3 (`CreativeUI.cs:98-101`) — the search scan's tooltip rows
    // read the same session difficulty.
    let items = grid_items(tracker, &m.filters, &m.search, player, 3);
    let cols = grid_cols(l.list);
    let view = l.list[3];
    let max_view = grid_height(items.len(), cols).max(view);
    r.push_clip(l.list);
    for (i, &kind) in items.iter().enumerate() {
        let cell = grid_cell(l.list, m.scroll, i);
        if cell[1] + 44.0 < l.list[1] || cell[1] > l.list[1] + l.list[3] {
            continue; // the view culling (`GetGridParameters`, `:89-96`)
        }
        let ctx = grid_context(tracker, kind);
        let item = ItemSlot {
            kind,
            name: crate::registry::name(kind),
            count: 1,
            prefix: 0,
        };
        slot::draw_slot(
            r,
            lookups,
            &item,
            cell[0],
            cell[1],
            ctx,
            i as i32,
            false,
            GRID_SCALE,
            grid_tint(ctx),
            super::UiPulse::default(),
            None,
            -1,
            true,
            false,
        );
    }
    r.pop_clip();
    // The scrollbar (`UIScrollbar.DrawSelf`): AutoHide is NOT set here
    // (unlike crafting) — the bar always draws.
    panel::draw_scrollbar(r, lookups, l.scrollbar, view, max_view, m.scroll, false);

    // The Blue filter strip (`UICreativeItemsInfiniteFilteringOptions`,
    // `:46-49` — Infinite_Tabs_B, not the crafting Cyan _2). The frame/icon
    // geometry is the shared crafting seam.
    for i in 0..11 {
        let button = super::crafting::filter_button_rect(l.filter_strip, i);
        let flag = m.filters.is_active(i);
        let (col, row) = super::crafting::filter_frame_col_row(flag, in_rect(mouse, button));
        push_sheet_rect(
            r,
            lookups,
            lookups.ui_infinite_tabs_b.as_ref(),
            button,
            super::crafting::filter_button_src(col, row),
            [1.0; 4],
        );
        let tint = if flag {
            [1.0; 4]
        } else {
            color::tint(color::mul([255, 255, 255, 255], 0.5))
        };
        push_sheet_rect(
            r,
            lookups,
            lookups.ui_infinite_icons.as_ref(),
            super::crafting::filter_icon_rect(l.filter_strip, i),
            super::crafting::filter_icon_src(super::crafting::FILTER_ICON_K[i]),
            tint,
        );
    }
}

/// What the draw pass hovered — the HUD's tooltip layer reads it (the C#
/// `Main.HoverItem` / `MouseTextNoOverride` interplay).
pub struct Hover {
    /// The grid cell's item (`ItemSlot.Handle` on hover,
    /// `UICreativeItemGrid.cs:22-29`).
    pub item: Option<ItemSlot>,
    /// The grid cell's `ItemSlot.Draw` context (`UICreativeItemGrid.cs:21`):
    /// 29 fully researched, 34 partial — the tooltip's Journey sacrifice
    /// row reads it (`Main.cs:21318`'s whitelist).
    pub context: i32,
    /// A `MouseTextNoOverride` line was drawn — the HUD's own item tooltip
    /// stands down (`Main.mouseText` already set).
    pub text_drawn: bool,
}

/// The whole menu (`CreativeUI.Draw`, `:92-118` + `UICreativePowersMenu.Draw`).
/// The caller gates on difficulty 3 + !blocked; `inventory_open` gates the
/// powers UI half (`CreativeUI.Update`, `:84-90` — the port draws the window
/// only while the inventory is open, the C# close rule
/// `Main.cs:45664` keeps them in step). `wind_target`/`cloud_alpha` are the
/// live weather values the wind/rain sliders' `GetSliderValueInner` read
/// (`CreativePowers.cs:1193-1196`/`:1332-1335`); `player` feeds the search
/// filter's tooltip scan (`ItemFilters.BySearch`, the local player's
/// damage/crit layers).
#[allow(clippy::too_many_arguments)]
pub fn draw(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    screen: Vec2,
    mouse: Vec2,
    m: &JourneyMenu,
    tracker: &ResearchTracker,
    powers: &CreativePowers,
    inventory_open: bool,
    pulse: super::UiPulse,
    wind_target: f32,
    cloud_alpha: f32,
    max_raining: f32,
    player: Option<&crate::player::Player>,
) -> Hover {
    let (toggle_size, plate_size) = sheet_sizes(lookups);
    let l = layout(screen, m.enabled, toggle_size, plate_size);
    let mut hover = Hover { item: None, context: 34, text_drawn: false };
    let toggle_hover = in_rect(mouse, l.toggle);
    draw_toggle(r, lookups, &l, toggle_hover);
    if toggle_hover {
        // `:157-164`: the open/close hint.
        let key = if m.enabled {
            "CreativePowers.PowersMenuOpen"
        } else {
            "CreativePowers.PowersMenuClosed"
        };
        super::tooltip::draw_mouse_text(r, lookups, mouse, &crate::lang::text(key), screen, pulse);
        hover.text_drawn = true;
    }
    if !m.enabled || !inventory_open {
        return hover;
    }
    // The main strip.
    draw_strip_panel(r, lookups, l.main_strip);
    for i in 0..MAIN_BUTTONS {
        let rect = strip_button(l.main_strip, i);
        let hovered = in_rect(mouse, rect);
        let option = MAIN_OPTIONS[i];
        let selected = option != 0 && m.category == option;
        // StopBiomeSpread (i == 5) is a toggle: its selected state is the
        // power's (`ASharedTogglePower.button_OnUpdate`,
        // `CreativePowers.cs:511-515`), picked colour OurFavoriteColor.
        let (selected, picked) = if i == 5 {
            (powers.stop_biome_spread(), FAVORITE)
        } else {
            (selected, CATEGORY_PICKED)
        };
        draw_power_button(
            r, lookups, rect, MAIN_ICONS[i], selected, hovered,
            picked, INVENTORY_DEFAULT, 1.0, 0.7,
        );
        if hovered {
            if let Some(key) = main_button_tooltip(i, m, powers) {
                super::tooltip::draw_mouse_text(r, lookups, mouse, &key, screen, pulse);
                hover.text_drawn = true;
            }
        }
    }
    // The open category's window / strips.
    match m.category {
        CATEGORY_RESEARCH => {
            draw_research_page(r, lookups, &l, m, tracker, mouse);
            if in_rect(mouse, l.confirm) {
                // `research_OnUpdate` (`:289-295`).
                super::tooltip::draw_mouse_text(
                    r, lookups, mouse,
                    &crate::lang::text("CreativePowers.ResearchButtonTooltip"),
                    screen, pulse,
                );
                hover.text_drawn = true;
            }
        }
        CATEGORY_DUPLICATION => {
            draw_duplication_page(r, lookups, &l, m, tracker, mouse, player);
            let items = grid_items(tracker, &m.filters, &m.search, player, 3);
            if let Some(i) = grid_hit(l.list, m.scroll, items.len(), mouse) {
                let kind = items[i];
                hover.item = Some(ItemSlot {
                    kind,
                    name: crate::registry::name(kind),
                    count: 1,
                    prefix: 0,
                });
                // `DrawSlot`'s context pick (`UICreativeItemGrid.cs:21`).
                hover.context = grid_context(tracker, kind);
            } else if let Some(i) = filter_hit(&l, mouse) {
                // `ShowButtonName` (`UICreativeItemsInfiniteFilteringOptions.cs:125-132`).
                super::tooltip::draw_mouse_text(
                    r, lookups, mouse,
                    &crate::lang::text(crate::craft_filter::FILTERS[i].display_name_key()),
                    screen, pulse,
                );
                hover.text_drawn = true;
            }
        }
        CATEGORY_TIME => draw_time_strip(r, lookups, &l, m, powers, mouse, screen, pulse, &mut hover),
        CATEGORY_WEATHER => draw_weather_strip(
            r, lookups, &l, m, powers, wind_target, cloud_alpha, max_raining, mouse, screen, pulse, &mut hover,
        ),
        CATEGORY_PERSONAL => draw_personal_strip(r, lookups, &l, m, powers, mouse, screen, pulse, &mut hover),
        _ => {
            // `RefreshElementsOrder` (`UICreativePowersMenu.cs:432-435`):
            // the difficulty slider (option 5) appends its slider element.
            if m.category == 5 {
                draw_difficulty_slider(r, lookups, &l, m, powers.difficulty_strength(), mouse);
                // The slider's hover text (`UpdateSliderColorAndShowMultiplier
                // MouseOver`, `CreativePowers.cs:1142-1149`): "x" + strength F2.
                let bar = slider_bar(difficulty_slider_panel(&l));
                if in_rect(mouse, slider_hit_rect(bar)) {
                    let text = format!("x{:.2}", powers.difficulty_strength());
                    super::tooltip::draw_mouse_text(r, lookups, mouse, &text, screen, pulse);
                    hover.text_drawn = true;
                }
                // The indication icons' hover texts (`MouseOver_Master/Expert/
                // Normal/Journey`, `:1106-1140`): the difficulty names.
                let icon_size = lookups
                    .wc_difficulty_icons
                    .iter()
                    .find_map(|s| s.map(|s| [s.width as f32, s.height as f32]))
                    .unwrap_or([32.0, 32.0]);
                let rects = difficulty_icon_rects(difficulty_slider_panel(&l), icon_size);
                for (i, key) in ["UI.Master", "UI.Expert", "UI.Normal", "UI.Creative"].iter().enumerate() {
                    if in_rect(mouse, rects[i]) {
                        super::tooltip::draw_mouse_text(
                            r, lookups, mouse, &crate::lang::text(key), screen, pulse,
                        );
                        hover.text_drawn = true;
                    }
                }
            }
        }
    }
    hover
}

/// The main strip's hover keys (`CategoryButton_OnUpdate_DisplayTooltips`,
/// `UICreativePowersMenu.cs:211-245`): `<key>Opened/Closed` by selection.
/// `AddDescriptionIfNeeded` is inert (`ShouldPowersBeElaborated` defaults
/// false and nothing sets it — `CreativePowerSettings.cs:5`).
fn main_button_tooltip(i: usize, m: &JourneyMenu, powers: &CreativePowers) -> Option<String> {
    let key = match i {
        0 => "CreativePowers.InfiniteItemsCategory",
        1 => "CreativePowers.ResearchItemsCategory",
        2 => "CreativePowers.TimeCategory",
        3 => "CreativePowers.WeatherCategory",
        4 => "CreativePowers.PersonalCategory",
        5 => {
            // `StopBiomeSpreadPower.GetButtonTextKey`
            // (`CreativePowers.cs:1540-1571`).
            return Some(crate::lang::text(if powers.stop_biome_spread() {
                "CreativePowers.StopBiomeSpread_Enabled"
            } else {
                "CreativePowers.StopBiomeSpread_Disabled"
            }));
        }
        6 => "CreativePowers.DifficultySlider",
        _ => return None,
    };
    let suffix = if i == 6 {
        if m.category == 5 { "_Opened" } else { "_Closed" }
    } else if m.category == MAIN_OPTIONS[i] {
        "Opened"
    } else {
        "Closed"
    };
    Some(crate::lang::text(&format!("{key}{suffix}")))
}

/// The duplication filter strip's tab hit — the shared crafting geometry.
pub fn filter_hit(l: &Layout, mouse: Vec2) -> Option<usize> {
    (0..11).find(|&i| in_rect(mouse, super::crafting::filter_button_rect(l.filter_strip, i)))
}

/// The search bar's outer rect (`UIWrappedSearchBar`, `UIWrappedSearchBar.cs:
/// 81-83`): H 24, W fill, padding 0 — the duplication panel's inner top row.
pub fn search_bar_rect(l: &Layout) -> [f32; 4] {
    [l.dup_inner[0], l.dup_inner[1], l.dup_inner[2], 24.0]
}

/// The magnifier button (`AddSearchBar`, `:102-105`): the 22x22
/// `Button_Search`, VAlign 0.5.
pub fn search_button_rect(l: &Layout) -> [f32; 4] {
    let s = search_bar_rect(l);
    [s[0], s[1] + 1.0, 22.0, 22.0]
}

/// The text box panel (`:110-116`): W (-button-3, 1), H fill, HAlign 1.
pub fn search_box_rect(l: &Layout) -> [f32; 4] {
    let s = search_bar_rect(l);
    let w = s[2] - 22.0 - 3.0;
    [s[0] + 25.0, s[1], w, 24.0]
}

/// The cancel button (`:142-147`): the 24x24 `SearchCancel`, HAlign 1,
/// VAlign 0.5, Left -2 — a child of the box.
pub fn search_cancel_rect(l: &Layout) -> [f32; 4] {
    let b = search_box_rect(l);
    [b[0] + b[2] - 24.0 - 2.0, b[1], 24.0, 24.0]
}

// ---- the sub-strips (Phase 3) ---------------------------------------------------

/// The time strip's button count (`CreateTimePowerStrip`,
/// `UICreativePowersMenu.cs:314-332`): FreezeTime, Dawn, Noon, Dusk, Midnight,
/// the rate-slider subcategory button.
pub const TIME_BUTTONS: usize = 6;
/// The time strip's icons (`CreativePowersHelper.cs:76-85`): FreezeTime 10,
/// TimeDawn 11, TimeNoon 12, TimeDusk 13, TimeMidnight 14, ModifyTime 19.
pub const TIME_ICONS: [usize; TIME_BUTTONS] = [10, 11, 12, 13, 14, 19];
/// The weather strip (`CreateWeatherPowerStrip`, `:351-369`): wind slider,
/// wind freeze, rain slider, rain freeze — icons 15, 16, 17, 18.
pub const WEATHER_BUTTONS: usize = 4;
pub const WEATHER_ICONS: [usize; WEATHER_BUTTONS] = [15, 16, 17, 18];
/// The personal strip (`CreatePersonalPowerStrip`, `:334-349`): Godmode,
/// FarPlacementRange, the spawn-rate slider — icons 6, 7, 9.
pub const PERSONAL_BUTTONS: usize = 3;
pub const PERSONAL_ICONS: [usize; PERSONAL_BUTTONS] = [6, 7, 9];

/// The time strip's rect: Left 80 (`UICreativePowersMenu.cs:124-128`).
pub fn time_strip(l: &Layout) -> [f32; 4] {
    strip_rect(l.container, 80.0, TIME_BUTTONS)
}
pub fn weather_strip(l: &Layout) -> [f32; 4] {
    strip_rect(l.container, 80.0, WEATHER_BUTTONS)
}
pub fn personal_strip(l: &Layout) -> [f32; 4] {
    strip_rect(l.container, 80.0, PERSONAL_BUTTONS)
}

/// `ModifyTimeRate.ProvideSlider` (`CreativePowers.cs:882-924`): the 87x180
/// panel at Left 20 + 2*60 = 140 (`CreateSubcategoryButton`,
/// `UICreativePowersMenu.cs:375`), VAlign 0.5.
pub fn time_slider_panel(l: &Layout) -> [f32; 4] {
    [
        l.container[0] + 140.0,
        l.container[1] + (l.container[3] - 180.0) * 0.5,
        87.0,
        180.0,
    ]
}

/// The vertical slider's bar (`CreativePowersHelper.CreateSlider`,
/// `CreativePowersHelper.cs:172-184`): W 12, H (-10,1), Left 6, VAlign 0.5
/// inside the panel's inner rect (padding 12).
pub fn time_slider_bar(panel: [f32; 4]) -> [f32; 4] {
    [panel[0] + 12.0 + 6.0, panel[1] + 12.0 + 5.0, 12.0, 156.0 - 10.0]
}

/// `UIVerticalSlider.DrawValueBarDynamicWidth`'s inner fill rect
/// (`UIVerticalSlider.cs:86-92`): the spliced ColorBar's (5,4,4,4) insets.
pub fn slider_fill_rect(bar: [f32; 4]) -> [f32; 4] {
    [bar[0] + 5.0, bar[1] + 4.0, bar[2] - 9.0, bar[3] - 8.0]
}

/// `UIVerticalSlider`'s hover/drag rect (`:105-107`): the fill rect inflated
/// 4 px horizontally.
pub fn slider_hit_rect(bar: [f32; 4]) -> [f32; 4] {
    let f = slider_fill_rect(bar);
    [f[0] - 4.0, f[1], f[2] + 8.0, f[3]]
}

/// The drag value (`UIVerticalSlider.cs:135`):
/// `GetLerpValue(bottom, top, mouseY, clamped)`.
pub fn slider_value_at(bar: [f32; 4], mouse_y: f32) -> f32 {
    let f = slider_fill_rect(bar);
    let bottom = f[1] + f[3];
    let top = f[1];
    ((mouse_y - bottom) / (top - bottom)).clamp(0.0, 1.0)
}

/// `ModifyTimeRate.UpdateInfoFromSliderValueCache` (`CreativePowers.cs:877-880`):
/// `TargetTimeRate = (int)Math.Round(Remap(v, 0, 1, 1, 24))` — banker's
/// rounding (`Math.Round`), so 0.5 → 12.5 → 12.
pub fn time_rate_from_cache(v: f32) -> f32 {
    (1.0 + v * 23.0).round_ties_even()
}

/// The x24/x12/x1 labels' rects (`ProvideSlider`, `:893-922`): HAlign 1,
/// VAlign 0/0.5/1 in the panel's inner rect; the hit boxes are the measured
/// text (the draw pass measures; the click path uses the same fallback).
pub fn time_label_rects(panel: [f32; 4], sizes: [[f32; 2]; 3]) -> [[f32; 4]; 3] {
    let inner = [panel[0] + 12.0, panel[1] + 12.0, 63.0, 156.0];
    std::array::from_fn(|i| {
        let [w, h] = sizes[i];
        let y = inner[1] + (inner[3] - h) * (i as f32 * 0.5);
        [inner[0] + inner[2] - w, y, w, h]
    })
}

/// The weather sliders' 132x180 panel (`ModifyWindDirectionAndStrength.
/// ProvideSlider`, `CreativePowers.cs:1207-1211`; `ModifyRainPower`'s is the
/// same, `:1346-1350`): Left 20 + 2*60 = 140 (`CreateSubcategoryButton`,
/// `UICreativePowersMenu.cs:375`), VAlign 0.5.
pub fn weather_slider_panel(l: &Layout) -> [f32; 4] {
    [
        l.container[0] + 140.0,
        l.container[1] + (l.container[3] - 180.0) * 0.5,
        132.0,
        180.0,
    ]
}

/// The spawn-rate slider's 77x180 panel (`SpawnRateSliderPerPlayerPower.
/// ProvideSlider`, `CreativePowers.cs:1614-1618`): same Left 140.
pub fn spawn_slider_panel(l: &Layout) -> [f32; 4] {
    [
        l.container[0] + 140.0,
        l.container[1] + (l.container[3] - 180.0) * 0.5,
        77.0,
        180.0,
    ]
}

/// The difficulty slider's 82x180 panel (`DifficultySliderPower.
/// ProvideSlider`, `CreativePowers.cs:1034-1038`): the MAIN strip's
/// subcategory at depth 1 (`UICreativePowersMenu.cs:205`), so Left
/// 20 + 1*60 = 80.
pub fn difficulty_slider_panel(l: &Layout) -> [f32; 4] {
    [
        l.container[0] + 80.0,
        l.container[1] + (l.container[3] - 180.0) * 0.5,
        82.0,
        180.0,
    ]
}

/// The vertical slider's bar inside an arbitrary-width panel — the same
/// `CreateSlider` geometry as [`time_slider_bar`] (`CreativePowersHelper.cs:
/// 172-184`): W 12, H (-10,1), Left 6, VAlign 0.5 in the inner rect.
pub fn slider_bar(panel: [f32; 4]) -> [f32; 4] {
    [panel[0] + 12.0 + 6.0, panel[1] + 12.0 + 5.0, 12.0, panel[3] - 24.0 - 10.0]
}

/// The three text labels' rects in an arbitrary-width panel — the same
/// HAlign 1 / VAlign 0,0.5,1 shape as [`time_label_rects`].
pub fn slider_label_rects(panel: [f32; 4], sizes: [[f32; 2]; 3]) -> [[f32; 4]; 3] {
    let inner = [panel[0] + 12.0, panel[1] + 12.0, panel[2] - 24.0, panel[3] - 24.0];
    std::array::from_fn(|i| {
        let [w, h] = sizes[i];
        let y = inner[1] + (inner[3] - h) * (i as f32 * 0.5);
        [inner[0] + inner[2] - w, y, w, h]
    })
}

/// The difficulty slider's four indication icons (`AddIndication`,
/// `CreativePowers.cs:1042-1045,1073-1094`): the 32x32 difficulty icons at
/// HAlign 1, VAlign 0/1⁄3/2⁄3/1, Left +4, Top +2 — the y anchors are the
/// CLICK values' inverses (x3 at 0, x2 at 1/3, x1 at 2/3, x0.5 at 1).
pub fn difficulty_icon_rects(panel: [f32; 4], icon: [f32; 2]) -> [[f32; 4]; 4] {
    let inner = [panel[0] + 12.0, panel[1] + 12.0, panel[2] - 24.0, panel[3] - 24.0];
    std::array::from_fn(|i| {
        let anchor = i as f32 / 3.0;
        [
            inner[0] + inner[2] - icon[0] + 4.0,
            inner[1] + (inner[3] - icon[1]) * anchor + 2.0,
            icon[0],
            icon[1],
        ]
    })
}

/// The difficulty icons' click caches (`Click_Master/Expert/Normal/Journey`,
/// `CreativePowers.cs:1049-1071`): 1, 0.66, 0.33, 0 — top to bottom.
pub const DIFFICULTY_ICON_CACHES: [f32; 4] = [1.0, 0.66, 0.33, 0.0];

/// The slider panel + the rate labels (`ProvideSlider`'s draw half).
fn draw_time_slider(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    l: &Layout,
    m: &JourneyMenu,
    mouse: Vec2,
) {
    let panel = time_slider_panel(l);
    draw_slider_chrome(r, lookups, panel, m.time_rate_cache, [255, 69, 0, 255], [100, 149, 237, 255]);
    // The labels.
    let labels = ["x24", "x12", "x1"];
    let sizes: [[f32; 2]; 3] = std::array::from_fn(|i| {
        [
            hud_text_width(r, lookups, labels[i], 1.0),
            line_height(lookups),
        ]
    });
    let rects = time_label_rects(panel, sizes);
    for i in 0..3 {
        // The hovered label's shadow turns OurFavoriteColor
        // (`Button_OnMouseOver/Out`, `CreativePowers.cs:944-960`) — the port
        // keeps the black shadow (the shadow-colour seam is a recorded
        // cosmetic gap) and draws the white text.
        let _ = in_rect(mouse, rects[i]);
        ui_text(r, lookups, rects[i][0], rects[i][1], 1.0, [1.0; 4], labels[i]);
    }
}

/// The shared slider chrome — the panel, the ColorBar track, the fill and
/// the knob (`UIPanel`'s default colours + `UIVerticalSlider.
/// DrawValueBarDynamicWidth`, `UIVerticalSlider.cs:81-104`). The fill
/// percent is the RAW `GetSliderValue` (unclamped — the wind slider's
/// walked target can read past [0,1]; the C# draws the over-height fill
/// verbatim, `(int)(h * FillPercent)`).
fn draw_slider_chrome(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    panel: [f32; 4],
    fill_percent: f32,
    empty: [u8; 4],
    filled: [u8; 4],
) {
    let bg = color::tint(color::mul([63, 82, 151, 255], 0.7));
    panel::draw_panel(r, lookups, lookups.ui_panel_bg, panel, bg);
    panel::draw_panel(r, lookups, lookups.ui_panel_border, panel, [0.0, 0.0, 0.0, 1.0]);
    let bar = slider_bar(panel);
    // The ColorBar track (`UIVerticalSlider.cs:84-87`): spliced (5,4,4,4).
    draw_spliced_panel(r, lookups, lookups.ui_color_bar, bar, 5.0, 4.0, 4.0, 4.0, [1.0; 4]);
    let fill = slider_fill_rect(bar);
    r.push_ui_rect([fill[0], fill[1]], [fill[2], fill[3]], color::tint(empty));
    let fh = (fill[3] * fill_percent) as i32 as f32;
    let fy = fill[1] + fill[3] - fh;
    r.push_ui_rect([fill[0], fy], [fill[2], fh], color::tint(filled));
    // The knob (`:100-104`): a (w+16)x4 white bar centred on the fill top
    // (+1 x), with a 2 px black inflate behind it.
    let kw = fill[2] + 16.0;
    let kx = fill[0] + fill[2] * 0.5 + 1.0 - kw * 0.5;
    r.push_ui_rect([kx - 2.0, fy - 4.0], [kw + 4.0, 8.0], [0.0, 0.0, 0.0, 1.0]);
    r.push_ui_rect([kx, fy - 2.0], [kw, 4.0], [1.0; 4]);
}

/// The three text labels shared by the wind/rain/spawn sliders
/// (`ProvideSlider`'s `UIText` children): HAlign 1, VAlign 0/0.5/1, the
/// black-shadow cosmetic gap as the time slider's.
fn draw_slider_labels(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    panel: [f32; 4],
    labels: [&str; 3],
    mouse: Vec2,
) {
    let sizes: [[f32; 2]; 3] = std::array::from_fn(|i| {
        [
            hud_text_width(r, lookups, labels[i], 1.0),
            line_height(lookups),
        ]
    });
    let rects = slider_label_rects(panel, sizes);
    for i in 0..3 {
        let _ = in_rect(mouse, rects[i]);
        ui_text(r, lookups, rects[i][0], rects[i][1], 1.0, [1.0; 4], labels[i]);
    }
}

/// The wind slider (`ModifyWindDirectionAndStrength.ProvideSlider`,
/// `CreativePowers.cs:1203-1245`): the West/No Wind/East labels and the
/// live-target fill (`GetSliderValueInner`, `:1193-1196`).
fn draw_wind_slider(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    l: &Layout,
    wind_target: f32,
    mouse: Vec2,
) {
    let panel = weather_slider_panel(l);
    let v = crate::creative::wind_cache_from_target(wind_target);
    draw_slider_chrome(r, lookups, panel, v, [255, 69, 0, 255], [100, 149, 237, 255]);
    draw_slider_labels(
        r,
        lookups,
        panel,
        [
            &crate::lang::text("CreativePowers.WindWest"),
            &crate::lang::text("CreativePowers.WindNone"),
            &crate::lang::text("CreativePowers.WindEast"),
        ],
        mouse,
    );
}

/// The rain slider (`ModifyRainPower.ProvideSlider`, `:1342-1384`): the
/// Monsoon/Drizzle/Clear Sky labels and the live-`cloudAlpha` fill
/// (`GetSliderValueInner`, `:1332-1335`).
fn draw_rain_slider(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    l: &Layout,
    cloud_alpha: f32,
    mouse: Vec2,
) {
    let panel = weather_slider_panel(l);
    draw_slider_chrome(r, lookups, panel, cloud_alpha, [255, 69, 0, 255], [100, 149, 237, 255]);
    draw_slider_labels(
        r,
        lookups,
        panel,
        [
            &crate::lang::text("CreativePowers.WeatherMonsoon"),
            &crate::lang::text("CreativePowers.WeatherDrizzle"),
            &crate::lang::text("CreativePowers.WeatherClearSky"),
        ],
        mouse,
    );
}

/// The spawn-rate slider (`SpawnRateSliderPerPlayerPower.ProvideSlider`,
/// `:1610-1652`): the x10/x1/x0 labels and the raw-cache fill.
fn draw_spawn_slider(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    l: &Layout,
    m: &JourneyMenu,
    mouse: Vec2,
) {
    let panel = spawn_slider_panel(l);
    draw_slider_chrome(r, lookups, panel, m.spawn_rate_cache, [255, 69, 0, 255], [100, 149, 237, 255]);
    draw_slider_labels(r, lookups, panel, ["x10", "x1", "x0"], mouse);
}

/// The difficulty slider (`DifficultySliderPower.ProvideSlider`,
/// `:1031-1047`): the four indication icons (Master/Expert/Normal/Journey,
/// top to bottom) and the cache fill. The fill colour is the difficulty
/// colour (`UpdateSliderColorAndShowMultiplierMouseOver`, `:1150-1154`):
/// the port is single-difficulty classic, so the strength < 1 arm answers
/// `Main.creativeModeColor` (`uistate::CREATIVE_MODE_COLOR`, the pinned
/// XNA-truncated Lerp(HotPink, White, 0.1)); the EmptyColor is Black
/// (`:1152`).
fn draw_difficulty_slider(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    l: &Layout,
    m: &JourneyMenu,
    strength: f32,
    mouse: Vec2,
) {
    let panel = difficulty_slider_panel(l);
    let filled = if strength < 1.0 {
        super::uistate::CREATIVE_MODE_COLOR
    } else {
        [255, 255, 255, 255]
    };
    draw_slider_chrome(r, lookups, panel, m.difficulty_cache, [0, 0, 0, 255], filled);
    // The indication icons (`AddIndication`, `:1073-1094`): the world-
    // creation difficulty sheets, top to bottom Master/Expert/Normal/
    // Journey — the `wc_difficulty_icons` table is Creative/Normal/Expert/
    // Master, so the indices reverse.
    let icon_size = lookups
        .wc_difficulty_icons
        .iter()
        .find_map(|s| s.map(|s| [s.width as f32, s.height as f32]))
        .unwrap_or([32.0, 32.0]);
    let rects = difficulty_icon_rects(panel, icon_size);
    for (i, sheet) in [3usize, 2, 1, 0].iter().enumerate() {
        if let Some(s) = lookups.wc_difficulty_icons[*sheet] {
            super::push_whole(r, lookups, &s, rects[i], [1.0; 4]);
        }
    }
}

/// The time strip (`CreateTimePowerStrip`): the freeze toggle, the four
/// momentary buttons, the rate subcategory button, then the open slider.
#[allow(clippy::too_many_arguments)]
fn draw_time_strip(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    l: &Layout,
    m: &JourneyMenu,
    powers: &CreativePowers,
    mouse: Vec2,
    screen: Vec2,
    pulse: super::UiPulse,
    hover: &mut Hover,
) {
    let strip = time_strip(l);
    draw_strip_panel(r, lookups, strip);
    for i in 0..TIME_BUTTONS {
        let rect = strip_button(strip, i);
        let hovered = in_rect(mouse, rect);
        let selected = match i {
            0 => powers.toggle(crate::creative::PowerId::FreezeTime),
            5 => m.time_sub == 1,
            _ => false, // the momentary buttons never latch
        };
        let picked = if i == 0 { FAVORITE } else { CATEGORY_PICKED };
        draw_power_button(
            r, lookups, rect, TIME_ICONS[i], selected, hovered,
            picked, INVENTORY_DEFAULT, 1.0, 0.7,
        );
        if hovered {
            let key = match i {
                0 => if selected { "CreativePowers.FreezeTime_Enabled" } else { "CreativePowers.FreezeTime_Disabled" },
                1 => "CreativePowers.StartDayImmediately",
                2 => "CreativePowers.StartNoonImmediately",
                3 => "CreativePowers.StartNightImmediately",
                4 => "CreativePowers.StartMidnightImmediately",
                _ => if selected { "CreativePowers.ModifyTimeRate_Opened" } else { "CreativePowers.ModifyTimeRate_Closed" },
            };
            super::tooltip::draw_mouse_text(r, lookups, mouse, &crate::lang::text(key), screen, pulse);
            hover.text_drawn = true;
        }
    }
    if m.time_sub == 1 {
        draw_time_slider(r, lookups, l, m, mouse);
    }
}

/// The weather strip (`CreateWeatherPowerStrip`, `UICreativePowersMenu.cs:
/// 351-369`): the wind slider + freeze, the rain slider + freeze, then the
/// open slider panel. The freeze toggles latch on the powers
/// (`ASharedTogglePower.button_OnUpdate`, `CreativePowers.cs:511-525`).
#[allow(clippy::too_many_arguments)]
fn draw_weather_strip(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    l: &Layout,
    m: &JourneyMenu,
    powers: &CreativePowers,
    wind_target: f32,
    cloud_alpha: f32,
    max_raining: f32,
    mouse: Vec2,
    screen: Vec2,
    pulse: super::UiPulse,
    hover: &mut Hover,
) {
    let strip = weather_strip(l);
    draw_strip_panel(r, lookups, strip);
    for i in 0..WEATHER_BUTTONS {
        let rect = strip_button(strip, i);
        let hovered = in_rect(mouse, rect);
        let selected = match i {
            0 => m.weather_sub == 1,
            1 => powers.freeze_wind(),
            2 => m.weather_sub == 2,
            _ => powers.freeze_rain(),
        };
        let picked = if i == 1 || i == 3 { FAVORITE } else { CATEGORY_PICKED };
        draw_power_button(
            r, lookups, rect, WEATHER_ICONS[i], selected, hovered,
            picked, INVENTORY_DEFAULT, 1.0, 0.7,
        );
        if hovered {
            let key = match i {
                0 => if selected { "CreativePowers.ModifyWindDirectionAndStrength_Opened" } else { "CreativePowers.ModifyWindDirectionAndStrength_Closed" },
                1 => if selected { "CreativePowers.FreezeWindDirectionAndStrength_Enabled" } else { "CreativePowers.FreezeWindDirectionAndStrength_Disabled" },
                2 => if selected { "CreativePowers.ModifyRainPower_Opened" } else { "CreativePowers.ModifyRainPower_Closed" },
                _ => if selected { "CreativePowers.FreezeRainPower_Enabled" } else { "CreativePowers.FreezeRainPower_Disabled" },
            };
            super::tooltip::draw_mouse_text(r, lookups, mouse, &crate::lang::text(key), screen, pulse);
            hover.text_drawn = true;
        }
    }
    match m.weather_sub {
        1 => {
            draw_wind_slider(r, lookups, l, wind_target, mouse);
            // The slider's hover text (`UpdateSliderAndShowMultiplierMouseOver`,
            // `CreativePowers.cs:1291-1308`): `(int)(windSpeedCurrent * 50f)`
            // mph, East when negative / West when positive, nothing at 0.
            let bar = slider_bar(weather_slider_panel(l));
            if in_rect(mouse, slider_hit_rect(bar)) {
                let num = (wind_target * 50.0) as i32;
                let text = if num < 0 {
                    crate::lang::text_args("GameUI.EastWind", &[num.abs().to_string()])
                } else if num > 0 {
                    crate::lang::text_args("GameUI.WestWind", &[num.to_string()])
                } else {
                    String::new()
                };
                super::tooltip::draw_mouse_text(r, lookups, mouse, &text, screen, pulse);
                hover.text_drawn = true;
            }
        }
        2 => {
            draw_rain_slider(r, lookups, l, cloud_alpha, mouse);
            // `Main.maxRaining.ToString("P0")` (`:1430-1438`) — the percent
            // with no decimals; the C# reads `Main.maxRaining` (the ramp
            // target), NOT the cloudAlpha the fill reads.
            let bar = slider_bar(weather_slider_panel(l));
            if in_rect(mouse, slider_hit_rect(bar)) {
                let pct = (max_raining * 100.0).round_ties_even() as i32;
                let text = format!("{pct} %");
                super::tooltip::draw_mouse_text(r, lookups, mouse, &text, screen, pulse);
                hover.text_drawn = true;
            }
        }
        _ => {}
    }
}

/// The personal strip (`CreatePersonalPowerStrip`): Godmode +
/// FarPlacementRange toggles, the spawn-rate subcategory button + its
/// slider panel.
#[allow(clippy::too_many_arguments)]
fn draw_personal_strip(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    l: &Layout,
    m: &JourneyMenu,
    powers: &CreativePowers,
    mouse: Vec2,
    screen: Vec2,
    pulse: super::UiPulse,
    hover: &mut Hover,
) {
    let strip = personal_strip(l);
    draw_strip_panel(r, lookups, strip);
    for i in 0..PERSONAL_BUTTONS {
        let rect = strip_button(strip, i);
        let hovered = in_rect(mouse, rect);
        let selected = match i {
            0 => powers.god_mode(),
            1 => powers.far_placement_range(),
            _ => m.personal_sub == 1,
        };
        let picked = if i == 2 { CATEGORY_PICKED } else { FAVORITE };
        draw_power_button(
            r, lookups, rect, PERSONAL_ICONS[i], selected, hovered,
            picked, INVENTORY_DEFAULT, 1.0, 0.7,
        );
        if hovered {
            let key = match i {
                0 => if selected { "CreativePowers.Godmode_Enabled" } else { "CreativePowers.Godmode_Disabled" },
                1 => if selected { "CreativePowers.InfinitePlacementRange_Enabled" } else { "CreativePowers.InfinitePlacementRange_Disabled" },
                _ => if selected { "CreativePowers.NPCSpawnRateSlider_Opened" } else { "CreativePowers.NPCSpawnRateSlider_Closed" },
            };
            super::tooltip::draw_mouse_text(r, lookups, mouse, &crate::lang::text(key), screen, pulse);
            hover.text_drawn = true;
        }
    }
    if m.personal_sub == 1 {
        draw_spawn_slider(r, lookups, l, m, mouse);
        // The slider's hover text (`UpdateSliderAndShowMultiplierMouseOver`,
        // `CreativePowers.cs:1698-1710`): "x" + the remapped factor at F2,
        // or the Disabled key at the exact-zero cache.
        let bar = slider_bar(spawn_slider_panel(l));
        if in_rect(mouse, slider_hit_rect(bar)) {
            let text = if powers.spawn_rate_disabled() {
                crate::lang::text("CreativePowers.NPCSpawnRateSliderEnemySpawnsDisabled")
            } else {
                format!("x{:.2}", crate::creative::spawn_rate_multiplier(m.spawn_rate_cache))
            };
            super::tooltip::draw_mouse_text(r, lookups, mouse, &text, screen, pulse);
            hover.text_drawn = true;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SCREEN: Vec2 = Vec2::new(1600.0, 900.0);

    fn lay() -> Layout {
        layout(SCREEN, false, [36.0, 36.0], [74.0, 74.0])
    }

    #[test]
    fn layout_matches_the_source_table() {
        let l = lay();
        // `CreativeUI.cs:104`: (28, 267), the sheet's size.
        assert_eq!(l.toggle, [28.0, 267.0, 36.0, 36.0]);
        // `UICreativePowersMenu.cs:104-113`: Top 270, H = 900-290.
        assert_eq!(l.container, [0.0, 270.0, 1600.0, 610.0]);
        // The main strip: 48 x (44*7+4) = 312, centred → y = 270+149.
        assert_eq!(l.main_strip, [20.0, 419.0, 48.0, 312.0]);
        // The window: Left 80, Top 270+44, 480 x (610-88).
        assert_eq!(l.window, [80.0, 314.0, 480.0, 522.0]);
        // The research panel: 170x170 centred → y = 314 + (522-170)/2.
        assert_eq!(l.research_panel, [80.0, 490.0, 170.0, 170.0]);
        assert_eq!(l.cogs_box, [80.0, 490.0, 280.0, 170.0]);
        // The plate: inner (92,502,146,146), centred 74x74, Top -20.
        assert_eq!(l.plate, [128.0, 518.0, 74.0, 74.0]);
        // The slot box: centred 48x48 in the plate, Top -15.
        assert_eq!(l.slot, [141.0, 516.0, 48.0, 48.0]);
        // The draw origin: centre - 26*0.85.
        assert_eq!(l.slot_draw, [165.0 - 22.1, 540.0 - 22.1]);
        // The confirm: 124x30 at the inner bottom.
        assert_eq!(l.confirm, [103.0, 618.0, 124.0, 30.0]);
        // The duplication panel/list/scrollbar.
        assert_eq!(l.dup_panel, [80.0, 352.0, 480.0, 484.0]);
        assert_eq!(l.dup_inner, [92.0, 364.0, 456.0, 460.0]);
        assert_eq!(l.list, [92.0, 392.0, 431.0, 432.0]);
        assert_eq!(l.scrollbar, [528.0, 396.0, 20.0, 424.0]);
        assert_eq!(l.filter_strip, [100.0, 318.0, 440.0, 40.0]);
    }

    #[test]
    fn the_short_screen_shifts_the_open_toggle() {
        // `CreativeUI.cs:108-111`: +52*0.85 while Enabled on screenH < 650.
        let l = layout(Vec2::new(1280.0, 600.0), true, [36.0, 36.0], [74.0, 74.0]);
        assert_eq!(l.toggle[0], 28.0 + 52.0 * 0.85);
        let closed = layout(Vec2::new(1280.0, 600.0), false, [36.0, 36.0], [74.0, 74.0]);
        assert_eq!(closed.toggle[0], 28.0);
    }

    #[test]
    fn the_strip_buttons_ride_the_44_pitch() {
        let l = lay();
        assert_eq!(strip_button(l.main_strip, 0), [24.0, 423.0, 40.0, 40.0]);
        assert_eq!(strip_button(l.main_strip, 6), [24.0, 423.0 + 264.0, 40.0, 40.0]);
    }

    #[test]
    fn the_category_toggle_closes_the_open_one() {
        // `ToggleCategory` (`UICreativePowersMenu.cs:301-312`).
        assert_eq!(toggle_category(CATEGORY_NONE, 2), 2);
        assert_eq!(toggle_category(2, 2), CATEGORY_NONE);
        assert_eq!(toggle_category(2, 1), 1);
    }

    #[test]
    fn the_progress_text_arms_match_the_source() {
        let mut t = ResearchTracker::default();
        t.register_sacrifice(8, 40);
        let empty = EMPTY_SLOT;
        // Empty slot, no memory → "???" (`:421`).
        assert_eq!(progress_text(&empty, (0, 0, 0), &t), "???");
        // Empty slot, partial memory → "(had/needed)" (`:415-418`).
        assert_eq!(progress_text(&empty, (8, 60, 100), &t), "(60/100)");
        // Completed memory (had == needed) → "???" (`:415`'s inequality).
        assert_eq!(progress_text(&empty, (8, 100, 100), &t), "???");
        // A filled slot prints its own numbers (`:432`).
        let torch = ItemSlot { kind: 8, name: "Torch", count: 5, prefix: 0 };
        assert_eq!(progress_text(&torch, (0, 0, 0), &t), "(40/100)");
        // The capless kind prints "X" (`:426-429`).
        let capless = ItemSlot { kind: 6196, name: "?", count: 1, prefix: 0 };
        assert_eq!(progress_text(&capless, (0, 0, 0), &t), "X");
    }

    #[test]
    fn the_piston_ladder_and_cog_speeds_match() {
        // `:341-346`: at rest frame 0; the ladder over the progress.
        assert_eq!(piston_frame(0), 0);
        assert_eq!(piston_frame(60), 1); // progress 0
        assert_eq!(piston_frame(54), 2); // progress 0.1 >= 1/15
        assert_eq!(piston_frame(6), 8); // progress 0.9
        assert_eq!(piston_frame(1), 8);
        // The cog step at the animation's END (progress 1): lerp 0 -> base 0.05.
        let step = cog_step(0);
        assert!((step[0] - 0.1).abs() < 1e-6);
        assert!((step[1] - 0.05 * 1.1428572).abs() < 1e-6);
        assert!((step[2] + 0.05).abs() < 1e-6);
        // At REST (timeLeft 60 -> progress 0) `GetLerpValue(1, .7, 0)` is 1:
        // num = 0.05*3, so the small pair spins at 0.3.
        let fast = cog_step(60);
        assert!((fast[0] - 0.3).abs() < 1e-6);
    }

    #[test]
    fn the_sacrifice_maths_clamp_and_complete() {
        let mut t = ResearchTracker::default();
        t.register_sacrifice(8, 90);
        // needed 10, stack 25 → take 10, done, excess 15.
        assert_eq!(sacrifice_amounts(8, 25, &t), Some((10, true)));
        // needed 10, stack 4 → take 4, not done.
        assert_eq!(sacrifice_amounts(8, 4, &t), Some((4, false)));
        // The capless kind refuses.
        assert_eq!(sacrifice_amounts(6196, 5, &t), None);
        let mut m = JourneyMenu::default();
        m.sacrifice = ItemSlot { kind: 8, name: "Torch", count: 25, prefix: 0 };
        assert!(sacrifice_confirm(&mut m, &mut t), "the excess returns");
        assert_eq!(m.sacrifice.count, 15);
        assert_eq!(t.sacrificed(8), 100);
        assert!(m.research_complete);
        assert_eq!(m.anim_left, 60);
        assert_eq!(m.remembered, (8, 100, 100));
        // A full row refuses (`:298-301`).
        assert_eq!(sacrifice_amounts(8, 15, &t), None);
    }

    #[test]
    fn the_animation_ticks_the_timer_and_fires_the_cues() {
        let mut m = JourneyMenu::default();
        m.anim_left = 57;
        m.research_complete = true;
        let mut rng = tr_core::UnifiedRandom::new(7);
        let cues = std::cell::RefCell::new(Vec::new());
        let mut play = |c: &str| cues.borrow_mut().push(c.to_string());
        // 57 → 56: no cue yet (the check reads the pre-decrement value).
        tick_animation(&mut m, true, &mut rng, &mut play);
        assert!(cues.borrow().is_empty());
        assert_eq!(m.anim_left, 56);
        // 56: the thunk (one of Research_1..3) + 15 sparks.
        tick_animation(&mut m, true, &mut rng, &mut play);
        assert_eq!(cues.borrow().len(), 1);
        assert!(["Research_1", "Research_2", "Research_3"].contains(&cues.borrow()[0].as_str()));
        assert_eq!(m.sparks.len(), 15);
        // Every spark starts at scale 0.6 - 1/60 after the first integration
        // and carries the -2 y bias.
        for p in &m.sparks {
            assert!((p.scale - (0.6 - 1.0 / 60.0)).abs() < 1e-6);
            assert!(p.vel[1] <= -2.0 + 0.16350001 + 1e-5);
        }
        // Run down to 40: the completion chime.
        while m.anim_left > 39 {
            tick_animation(&mut m, true, &mut rng, &mut play);
        }
        assert!(cues.borrow().iter().any(|c| c == "Research_0"));
        assert!(!m.research_complete);
        // Off the research page the timer still decrements, no cues.
        m.anim_left = 56;
        let n = cues.borrow().len();
        tick_animation(&mut m, false, &mut rng, &mut play);
        assert_eq!(cues.borrow().len(), n);
        assert_eq!(m.anim_left, 55);
    }

    #[test]
    fn the_grid_sorts_unlocked_first_then_alphabetical() {
        let mut t = ResearchTracker::default();
        t.register_sacrifice(8, 100); // Torch, fully + newly
        t.clear_newly_researched(8);
        t.register_sacrifice(97, 99); // Musket Ball, fully + newly
        t.register_sacrifice(2, 5); // Dirt Block, partial
        let items = grid_items(&t, &FilterBoard::default(), "", None, 3);
        // Newly first (Musket Ball), then fully (Torch), then partial.
        assert_eq!(items, vec![97, 8, 2]);
        // The filter narrows (`UpdateContents`' FitsFilter, `:542`).
        let mut f = FilterBoard::default();
        f.click(0); // Weapon
        // `ItemFilters.Weapon.FitsFilter` is `damage > 0` (`ItemFilters.cs:
        // 193-196`); the Musket Ball (97) carries damage, so only it survives.
        assert_eq!(grid_items(&t, &f, "", None, 3), vec![97]);
    }

    /// `ItemFilters.BySearch` (`ItemFilters.cs:28-48`): the tooltip-line
    /// substring scan, case-insensitive; the whitespace gate passes all.
    #[test]
    fn the_search_filter_scans_the_tooltip_lines() {
        let mut t = ResearchTracker::default();
        t.register_sacrifice(8, 100); // Torch
        t.register_sacrifice(2, 5); // Dirt Block
        // The name row matches case-insensitively.
        let items = grid_items(&t, &FilterBoard::default(), "torch", None, 3);
        assert_eq!(items, vec![8]);
        let items = grid_items(&t, &FilterBoard::default(), "DIRT", None, 3);
        assert_eq!(items, vec![2]);
        // A whitespace search passes everything (`SetSearchFilter`,
        // `EntryFilterer.cs:87-91`).
        assert_eq!(grid_items(&t, &FilterBoard::default(), "   ", None, 3).len(), 2);
        // A miss narrows to nothing.
        assert!(grid_items(&t, &FilterBoard::default(), "zzz", None, 3).is_empty());
        // The tooltip rows scan too: the Torch's "Provides light when..." —
        // the ItemTooltip row (`Lang._itemTooltipCache`).
        let torch_tip = crate::lang::item_tooltip(8);
        if !torch_tip.is_empty() && torch_tip != "ItemTooltip.Torch" {
            let word = torch_tip.split_whitespace().next().unwrap();
            assert!(search_fits(8, word, None, 3, &t), "the tooltip row scans: {word}");
        }
        // W4-c24c: the search entry's context stays -1 (`Item.cs:117`) — the
        // Journey sacrifice row is outside the whitelist, so its text never
        // matches even for a partial kind in a Journey session.
        let mut partial = ResearchTracker::default();
        partial.register_sacrifice(2, 5);
        assert!(!search_fits(2, "more to unlock", None, 3, &partial));
    }

    /// The new sliders' panel geometry (`ProvideSlider` widths + the
    /// `CreateSubcategoryButton` Left offsets).
    #[test]
    fn the_new_slider_panels_match_the_source() {
        let l = lay();
        // The weather panels: 132x180 at Left 140, VAlign 0.5.
        assert_eq!(weather_slider_panel(&l), [140.0, 485.0, 132.0, 180.0]);
        // The spawn panel: 77x180, same Left.
        assert_eq!(spawn_slider_panel(&l), [140.0, 485.0, 77.0, 180.0]);
        // The difficulty panel: 82x180 at Left 80 (depth 1).
        assert_eq!(difficulty_slider_panel(&l), [80.0, 485.0, 82.0, 180.0]);
        // The bar geometry matches the time slider's shape.
        let panel = weather_slider_panel(&l);
        assert_eq!(slider_bar(panel), time_slider_bar([panel[0], panel[1], 87.0, 180.0]));
        // The difficulty icons' click caches, top to bottom.
        assert_eq!(DIFFICULTY_ICON_CACHES, [1.0, 0.66, 0.33, 0.0]);
        // The icon rects: HAlign 1 + Left 4, Top +2, the four y anchors.
        let rects = difficulty_icon_rects(difficulty_slider_panel(&l), [32.0, 32.0]);
        let inner_x = 80.0 + 12.0;
        let inner_w = 82.0 - 24.0;
        assert_eq!(rects[0][0], inner_x + inner_w - 32.0 + 4.0);
        assert_eq!(rects[0][1], 485.0 + 12.0 + 2.0);
        assert_eq!(rects[3][1], 485.0 + 12.0 + (156.0 - 32.0) + 2.0);
    }

    #[test]
    fn the_grid_geometry_matches_the_dynamic_collection() {
        let l = lay();
        // 431 / 44 = 9 columns (`UpdateSize`, integer division).
        assert_eq!(grid_cols(l.list), 9);
        // startX = center - 44*9/2 = (92+215) - 198 = 109.
        let cell = grid_cell(l.list, 0.0, 0);
        assert_eq!(cell, [109.0, 392.0, 44.0, 44.0]);
        let cell9 = grid_cell(l.list, 0.0, 9);
        assert_eq!(cell9, [109.0, 436.0, 44.0, 44.0]);
        // The scroll shifts the lattice up.
        assert_eq!(grid_cell(l.list, 44.0, 0)[1], 348.0);
        assert_eq!(grid_height(10, 9), 88.0);
        // The hit clips to the list.
        assert_eq!(grid_hit(l.list, 0.0, 20, Vec2::new(120.0, 400.0)), Some(0));
        assert_eq!(grid_hit(l.list, 0.0, 20, Vec2::new(600.0, 400.0)), None);
    }

    #[test]
    fn the_time_rate_remap_rounds_bankers() {
        // `CreativePowers.cs:879`: Remap(v,0,1,1,24) then Math.Round.
        assert_eq!(time_rate_from_cache(0.0), 1.0);
        assert_eq!(time_rate_from_cache(1.0), 24.0);
        assert_eq!(time_rate_from_cache(0.5), 12.0); // 12.5 → ties-to-even
    }

    #[test]
    fn the_slider_value_inverts_the_fill() {
        let panel = time_slider_panel(&lay());
        let bar = time_slider_bar(panel);
        let f = slider_fill_rect(bar);
        assert_eq!(slider_value_at(bar, f[1] + f[3]), 0.0);
        assert_eq!(slider_value_at(bar, f[1]), 1.0);
        assert!((slider_value_at(bar, f[1] + f[3] * 0.5) - 0.5).abs() < 1e-6);
    }

    #[test]
    fn the_blocked_gate_matches_the_source() {
        // `CreativeUI.cs:47-65`.
        assert!(blocked(true, false, false));
        assert!(blocked(true, false, true), "talking blocks even with crafting");
        assert!(!blocked(false, true, true), "the visible crafting unblocks");
        assert!(blocked(false, true, false));
        assert!(!blocked(false, false, false));
    }
}
