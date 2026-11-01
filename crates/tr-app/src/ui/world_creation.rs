//! The world creation screen (`Main.menuMode == 888`'s `UIWorldCreation` +
//! `UIWorldCreationPreview`): the 500x452 root, the name/seed plates with their
//! randomize icons, the three `GroupOptionButton` option rows, the 84x84
//! preview stack, the description panel and the Back/Create pair — a faithful
//! immediate port, the same `layout()` + `draw()` shape as `ui/char_creation.rs`
//! (plan §"Screen-model ruling").
//!
//! The Advanced seed tab (`ClickAdvancedSeedMenu` -> `UIWorldCreationAdvanced`)
//! is out of scope (spec §9.4 / plan Uncertainty 4): the seed icon's click
//! degrades to the plain seed keyboard, and the special-seed ring effect
//! (`DrawSpecialSeedRingCallback`) is not drawn — both documented deviations.
//!
//! Every draw reads the sheets from `Lookups` `Option`s: a missing sheet draws
//! nothing, so `Lookups::default()` renders blank chrome and panics nowhere
//! (plan Review Focus 7).

use anyhow::Context;
use glam::Vec2;
use std::path::{Path, PathBuf};
use tr_render::SpriteRenderer;

use super::cards::fat_button_style;
use super::char_creation::lerp_rgb;
use super::color;
use super::in_rect;
use super::panel;
use super::text::{border_string, hud_text, hud_text_width};
use super::uistate;
use crate::assets;

// --- The draft (Main.ActiveWorldFileData's creation options) -------------------

/// `UIWorldCreation.MAX_NAME_LENGTH` (`UIWorldCreation.cs:81`) — the naming
/// keyboard's cap (`Click_SetName`'s 27, `:693`) and the display trim
/// (`TrimDisplayIfOverElementDimensions(27)`, `:737` — not ported, see
/// `draw_plate`).
pub const MAX_NAME_LENGTH: usize = 27;

/// `Main.ActiveWorldFileData`'s creation defaults (`SetDefaultOptions`,
/// `UIWorldCreation.cs:313-326`): a stand-in random name, a blank seed
/// (`ClearSeed` -> `""`, `:317`), the Medium size (`:318`), Normal difficulty
/// — Journey only when the active player is Journey (`:319-322`) — and Random
/// evil (`:323`). The size/difficulty/evil ids follow the source's private
/// enums (`WorldSizeId` :27-32, `WorldDifficultyId` :34-40, `WorldEvilId`
/// :42-47): difficulty is `Main.GameMode`'s 0 Normal / 1 Expert / 2 Master /
/// 3 Creative.
#[derive(Clone, Debug, PartialEq)]
pub struct NewWorldDraft {
    /// `_optionwWorldName` — `AssignRandomWorldName` (`:1002-1025`) composes
    /// one from the `RandomWorldName_*` localization tables; the port has no
    /// composition tables, so the stand-in is `NewWorldClick`'s
    /// `Lang.gen[57] + " " + (WorldList.Count + 1)` shape
    /// (`UIWorldSelect.cs:113`) rendered as `"World {n+1}"` (documented
    /// deviation).
    pub name: String,
    /// `_optionSeed` — blank means random (`FinishCreatingWorld`'s
    /// `SetSeedToRandomWithCurrentEvents` branch, `:852-855`).
    pub seed: String,
    /// `_optionSize` (`WorldSizeId`): 0 Small / 1 Medium / 2 Large.
    pub size: u8,
    /// `_optionDifficulty` (`WorldDifficultyId` = `Main.GameMode`): 0 Normal /
    /// 1 Expert / 2 Master / 3 Creative.
    pub difficulty: u8,
    /// `_optionEvil` (`WorldEvilId`): 0 Random / 1 Corruption / 2 Crimson.
    pub evil: u8,
}

impl Default for NewWorldDraft {
    fn default() -> Self {
        Self::new(0, 0)
    }
}

impl NewWorldDraft {
    /// `SetDefaultOptions` (`UIWorldCreation.cs:313-326`) over the port's own
    /// inputs: `world_count` is `Main.WorldList.Count` at `NewWorldClick`
    /// (`UIWorldSelect.cs:113`), `active_difficulty` is
    /// `Main.ActivePlayerFileData.Player.difficulty` (`:319`).
    pub fn new(world_count: usize, active_difficulty: u8) -> Self {
        Self {
            name: Self::stand_in_name(world_count),
            seed: String::new(), // `ClearSeed` (`:317`)
            size: 1,             // `WorldSizeId.Medium` (`:318`)
            // `if (Main.ActivePlayerFileData.Player.difficulty == 3)
            // this._optionDifficulty = WorldDifficultyId.Creative;` (`:319-322`).
            difficulty: if active_difficulty == 3 { 3 } else { 0 },
            evil: 0, // `WorldEvilId.Random` (`:323`)
        }
    }

    /// The port's `AssignRandomWorldName` stand-in (`:1002-1025`): the
    /// `RandomWorldName_Composition/Adjective/Location/Noun` tables and the
    /// 1-in-10000 `SpecialWorldName.TheConstant` roll have no localization
    /// port, so the name re-derives `NewWorldClick`'s `"World {n+1}"`
    /// (`UIWorldSelect.cs:113`) — a no-op unless the player renamed it first
    /// (documented deviation).
    pub fn assign_random_world_name(&mut self, world_count: usize) {
        self.name = Self::stand_in_name(world_count);
    }

    /// `Lang.gen[57] + " " + (Main.WorldList.Count + 1)` (`UIWorldSelect.cs:
    /// 113`) as `"World {n+1}"`.
    pub fn stand_in_name(world_count: usize) -> String {
        format!("World {}", world_count + 1)
    }
}

// --- The row tables (`AddWorld*Options`, `UIWorldCreation.cs:358-495`) --------

/// The size row's display ids (`WorldSizeId.Small/Medium/Large`, `:360-365`)
/// — row index == draft id.
pub const SIZE_ROW_IDS: [u8; 3] = [0, 1, 2];

/// The difficulty row's display order (`:405-411`): `WorldDifficultyId`.
/// Creative, Normal, Expert, Master — the enum ids 3, 0, 1, 2.
pub const DIFFICULTY_ROW_IDS: [u8; 4] = [3, 0, 1, 2];

/// The evil row's display ids (`WorldEvilId.Random/Corruption/Crimson`,
/// `:454-459`) — row index == draft id.
pub const EVIL_ROW_IDS: [u8; 3] = [0, 1, 2];

/// (label, title tint) per size button (`:366-383`): `Lang.menu[92..94]`
/// ("Small"/"Medium"/"Large") with `Color.Cyan`,
/// `Color.Lerp(Cyan, LimeGreen, 0.5)`, `Color.LimeGreen`. The lerp truncates
/// through the vector space like every XNA `Color.Lerp` (`lerp_rgb`).
pub const SIZE_ROWS: [(&str, [u8; 4]); 3] = [
    ("Small", [0, 255, 255, 255]),    // Color.Cyan (`:380`)
    ("Medium", [25, 230, 152, 255]),  // Lerp(Cyan, LimeGreen, 0.5) (`:381`)
    ("Large", [50, 205, 50, 255]),    // Color.LimeGreen (`:382`)
];

/// (label, title tint) per difficulty button (`:412-432`): `UI.Creative`
/// "Journey" / `UI.Normal` "Classic" / `UI.Expert` / `UI.Master` with
/// `Main.creativeModeColor` / White / `Main.mcColor` / `Main.hcColor`.
pub const DIFFICULTY_ROWS: [(&str, [u8; 4]); 4] = [
    ("Journey", uistate::CREATIVE_MODE_COLOR), // (`:428`)
    ("Classic", [255, 255, 255, 255]),         // Color.White (`:429`)
    ("Expert", uistate::MC_COLOR),             // (`:430`)
    ("Master", uistate::HC_COLOR),             // (`:431`)
];

/// (label, title tint) per evil button (`:460-477`): `Lang.misc[103]` "Random"
/// / `Lang.misc[101]` "Corruption" / `Lang.misc[102]` "Crimson" with White /
/// `Color.MediumPurple` / `Color.IndianRed` (the XNA named colours,
/// #9370DB / #CD5C5C).
pub const EVIL_ROWS: [(&str, [u8; 4]); 3] = [
    ("Random", [255, 255, 255, 255]),      // Color.White (`:474`)
    ("Corruption", [147, 112, 219, 255]),  // Color.MediumPurple (`:475`)
    ("Crimson", [205, 92, 92, 255]),       // Color.IndianRed (`:476`)
];

/// `GroupOptionButton._color` (`GroupOptionButton.cs:133`) — every
/// `UIWorldCreation` button keeps the ctor default
/// `Colors.InventoryDefaultColor`; the row tints above colour the title text
/// and nothing else (the panels and icons derive from `_color`).
pub const OPTION_BUTTON_COLOR: [u8; 4] = uistate::INVENTORY_DEFAULT_COLOR;

/// The `GroupOptionButton` layer colours (`GroupOptionButton.cs:210-260`),
/// returned as `(fill, accent)`:
///
/// * `fill` — the `PanelGrayscale` spliced-panel colour, always drawn:
///   `Color.Lerp(Color.Black, color, FadeFromBlack = 1) * _opacity = 0.7`
///   (`:233`).
/// * `accent` — the colour of the button's top layer: the selected
///   `CategoryPanelHighlight` inset rim at `Lerp(color, White, _whiteLerp =
///   0.7) * 0.7` (`:236`), overridden by the hovered `CategoryPanelBorder`'s
///   plain white (`:240` — the border draws last when both states hold); with
///   neither state the same rim formula comes back — the 32x32 option-row
///   icons' idle tint (`:247`). The 40x40 icon buttons are always selected
///   (`option: true`), so their icons stay White (see `draw_icon_button`).
pub fn option_button_colors(selected: bool, hovered: bool, color: [u8; 4]) -> ([u8; 4], [u8; 4]) {
    // `Color.Lerp(Color.Black, color, this.FadeFromBlack) * num` (`:233`) —
    // the lerp with 1.0 is `color` itself.
    let fill = super::color::mul(color, 0.7);
    // `Color.Lerp(color, Color.White, this._whiteLerp) * num` (`:236`/`:247`).
    let rim = super::color::mul(lerp_rgb(color, [255, 255, 255, 255], 0.7), 0.7);
    let accent = match (selected, hovered) {
        // `:240` — the hovered border is `_borderColor` = White.
        (_, true) => [255, 255, 255, 255],
        // `:236` — the selected rim; `(false, false)` reads the same formula
        // back for the idle icon tint (`:247`).
        (true, false) | (false, false) => rim,
    };
    (fill, accent)
}

// --- The layout ------------------------------------------------------------------

/// Every static rect, resolved through `uistate::resolve` in the source's
/// construction order (`BuildPage` -> `MakeInfoMenu` -> the option builders).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Layout {
    /// (550, 152, 500, 452) (`UIWorldCreation.cs:166-176`).
    pub root: [f32; 4],
    /// (550, 202, 500, 298) (`:177-185`).
    pub panel: [f32; 4],
    /// The right column: the panel's full rect; its children resolve against
    /// the PaddingTop 8 / Bottom 12 inner (`:187-197`).
    pub right: [f32; 4],
    /// The info container's inner rect (560, 210, 480, 278) — the
    /// `SetPadding(10)` container every info child resolves against
    /// (`:208-218`).
    pub info: [f32; 4],
    /// The `IconRandomName` button, 40x40 (`:223-234`).
    pub random_name_icon: [f32; 4],
    /// The name plate: `UICharacterNameButton`, W `(-132, 1)`, Left 44
    /// (`:236-248`).
    pub name_button: [f32; 4],
    /// The `IconRandomSeed` button, 40x40 (`:250-263`).
    pub seed_button: [f32; 4],
    /// The seed plate (`:264-277`).
    pub seed_plate: [f32; 4],
    /// The 84x84 preview, HAlign 1 (`:278-286`).
    pub preview: [f32; 4],
    /// The four separators at `Top = accumulated - 8` (`:288-299`,
    /// `:303-311`).
    pub separators: [[f32; 4]; 4],
    /// The size row, three buttons of W `(-4*(n-1), 1/n)` (`:358-401`).
    pub size_buttons: [[f32; 4]; 3],
    /// The difficulty row, four buttons of W `(-1*(n-1), 1/n)` (`:403-450`).
    pub difficulty_buttons: [[f32; 4]; 4],
    /// The evil row (`:452-495`).
    pub evil_buttons: [[f32; 4]; 3],
    /// The 40-tall description panel (`:328-356`).
    pub description: [f32; 4],
    /// (550, 509, 240, 50) (`:637-648`; the `MakeBackAndCreatebuttons` twin
    /// of Task 7's — the plan's 559 was the button bottom, not its top).
    pub back: [f32; 4],
    /// (810, 509, 240, 50) (`:650-662`).
    pub create: [f32; 4],
}

/// `layout(screen, plate_w)` — the whole page through the shared pass.
/// `plate_w` is the font-measured plate title width, the plan's font-free
/// convention parameter; every *rect* here is style-driven and font-free
/// (the plates resolve `Width (-132, 1)` against the 480 info inner), so —
/// exactly like Task 7's `name_w` — it feeds only the draw's text placement.
pub fn layout(screen: Vec2, plate_w: f32) -> Layout {
    let _ = plate_w; // the plate rect chain is `(-132, 1)` of the info inner; see `draw`.
    let full = [0.0, 0.0, screen.x, screen.y];
    // The root: `Width 500, Height 434+18, Top 170-18, HAlign .5, VAlign 0`
    // (`:166-176`), `SetPadding(0)` (`:175`).
    let root = uistate::resolve(
        full,
        uistate::Style::px(0.0),
        uistate::Style::px(170.0 - 18.0),
        uistate::Style::px(500.0),
        uistate::Style::px(434.0 + 18.0),
        0.5,
        0.0,
        0.0,
        0.0,
        f32::MAX,
        [0.0; 4],
    );
    let root_inner = uistate::inner(root, [0.0; 4]);
    // The panel: `Width 100%, Height 280+18, Top 50` (`:177-185`), padding 0.
    let panel = uistate::resolve(
        root_inner,
        uistate::Style::px(0.0),
        uistate::Style::px(50.0),
        uistate::Style::pct(1.0),
        uistate::Style::px(280.0 + 18.0),
        0.0,
        0.0,
        0.0,
        0.0,
        f32::MAX,
        [0.0; 4],
    );
    let panel_inner = uistate::inner(panel, [0.0; 4]);
    // The right column: `Top 0, Width (0, 1), Height (0, 1), HAlign 1` with
    // `PaddingTop 8 / PaddingBottom 12` (`:187-197`).
    let right = uistate::resolve(
        panel_inner,
        uistate::Style::px(0.0),
        uistate::Style::px(0.0),
        uistate::Style::new(0.0, 1.0),
        uistate::Style::new(0.0, 1.0),
        1.0,
        0.0,
        0.0,
        0.0,
        f32::MAX,
        [0.0, 8.0, 0.0, 12.0],
    );
    let right_inner = uistate::inner(right, [0.0, 8.0, 0.0, 12.0]);
    // The info container: `Width/Height (0, 1), HAlign .5, SetPadding(10),
    // PaddingBottom/Top = 0` (`:208-218`) — the field carries its INNER rect.
    let info_outer = uistate::resolve(
        right_inner,
        uistate::Style::px(0.0),
        uistate::Style::px(0.0),
        uistate::Style::new(0.0, 1.0),
        uistate::Style::new(0.0, 1.0),
        0.5,
        0.0,
        0.0,
        0.0,
        f32::MAX,
        [10.0, 0.0, 10.0, 0.0],
    );
    let info = uistate::inner(info_outer, [10.0, 0.0, 10.0, 0.0]);

    // `num = 0` (`:219`) — the two plate rows (`:223-277`).
    let row1_top = 0.0;
    // The randomize-name button: 40x40, `HAlign 0`, `Top num` (`:223-234`).
    let random_name_icon = uistate::resolve(
        info,
        uistate::Style::px(0.0),
        uistate::Style::px(row1_top),
        uistate::Style::px(40.0),
        uistate::Style::px(40.0),
        0.0,
        0.0,
        0.0,
        0.0,
        f32::MAX,
        [0.0; 4],
    );
    // The name plate: `Width (-num3, 1)`, `Left num2 = 44`, `Top num`
    // (`:236-248`); `num3 = 88 + num2 = 132` (`:220-221`).
    let name_button = uistate::resolve(
        info,
        uistate::Style::px(44.0), // `pixels = num2` (`:222`)
        uistate::Style::px(row1_top),
        uistate::Style::new(-132.0, 1.0),
        uistate::Style::px(40.0), // `UICharacterNameButton.cs:37`
        0.0,
        0.0,
        0.0,
        0.0,
        f32::MAX,
        [0.0; 4],
    );
    // `num += plate height + 4` (`:249`).
    let row2_top = row1_top + 40.0 + 4.0;
    // The seed (advanced) button: 40x40 at `Top num` (`:250-263`).
    let seed_button = uistate::resolve(
        info,
        uistate::Style::px(0.0),
        uistate::Style::px(row2_top),
        uistate::Style::px(40.0),
        uistate::Style::px(40.0),
        0.0,
        0.0,
        0.0,
        0.0,
        f32::MAX,
        [0.0; 4],
    );
    // The seed plate (`:264-277`).
    let seed_plate = uistate::resolve(
        info,
        uistate::Style::px(44.0),
        uistate::Style::px(row2_top),
        uistate::Style::new(-132.0, 1.0),
        uistate::Style::px(40.0),
        0.0,
        0.0,
        0.0,
        0.0,
        f32::MAX,
        [0.0; 4],
    );
    // The preview: `84x84, HAlign 1, VAlign 0` (`:278-286`).
    let preview = uistate::resolve(
        info,
        uistate::Style::px(0.0),
        uistate::Style::px(0.0),
        uistate::Style::px(84.0),
        uistate::Style::px(84.0),
        1.0,
        0.0,
        0.0,
        0.0,
        f32::MAX,
        [0.0; 4],
    );
    // `num += plate height + 10` (`:287`) — 94. The separators:
    // `AddHorizontalSeparator(container, num + 2)` first (`:288`), then one
    // per `num += 48` step (`:291-298`), each at `Top = accumulated - 8`
    // (`:308`) with the separator sheet's 4 px height
    // (`UIHorizontalSeparator.cs:23-27`).
    let sep_tops = [94.0 + 2.0, 142.0, 190.0, 238.0];
    let separators = sep_tops.map(|acc| {
        uistate::resolve(
            info,
            uistate::Style::px(0.0),
            uistate::Style::px(acc - 8.0),
            uistate::Style::pct(1.0),
            uistate::Style::px(4.0),
            0.0,
            0.0,
            0.0,
            0.0,
            f32::MAX,
            [0.0; 4],
        )
    });

    // The option rows (`AddWorldSizeOptions` :358-401, `AddWorldDifficulty-
    // Options` :403-450, `AddWorldEvilOptions` :452-495): each button keeps
    // the ctor's 34 px height (`GroupOptionButton.cs:125`), `Left (0, 0)`,
    // `HAlign i/(n-1)`, `Top = accumulated` (`:392-396`, `:438-441`,
    // `:484-487`), and `Width (-pitch*(n-1), 1/n)` (`:389`, `:438`, `:483`).
    fn row_rects<const N: usize>(info: [f32; 4], top: f32, pitch: f32) -> [[f32; 4]; N] {
        std::array::from_fn(|i: usize| {
            uistate::resolve(
                info,
                uistate::Style::px(0.0), // `Left = (1 - usable) = 0%` (`:390`)
                uistate::Style::px(top),
                uistate::Style::new(-pitch * (N as f32 - 1.0), 1.0 / N as f32),
                uistate::Style::px(34.0), // `GroupOptionButton.cs:125`
                i as f32 / (N as f32 - 1.0),
                0.0,
                0.0,
                0.0,
                f32::MAX,
                [0.0; 4],
            )
        })
    }
    // The rows sit at the accumulated `num`s 94 / 142 / 190 (`:290`, `:293`,
    // `:296`); the 48 px pitch is the builder tail (`:291`, `:294`, `:297`).
    let size_buttons = row_rects::<3>(info, 94.0, 4.0);
    let difficulty_buttons = row_rects::<4>(info, 142.0, 1.0);
    let evil_buttons = row_rects::<3>(info, 190.0, 4.0);

    // The description panel: `HAlign .5, VAlign 1, Width (0, 1), Height 40,
    // Top 2` (`:328-339`) — 278 - 40 + 2 below the info top.
    let description = uistate::resolve(
        info,
        uistate::Style::px(0.0),
        uistate::Style::px(2.0),
        uistate::Style::new(0.0, 1.0),
        uistate::Style::px(40.0),
        0.5,
        1.0,
        0.0,
        0.0,
        f32::MAX,
        [0.0; 4],
    );

    // Back / Create: `Width (-10, 0.5), Height 50, VAlign 1, HAlign 0/1,
    // Top -45` of the ROOT (`MakeBackAndCreatebuttons`, `:635-663`).
    let back = uistate::resolve(
        root_inner,
        uistate::Style::px(0.0),
        uistate::Style::px(-45.0),
        uistate::Style::new(-10.0, 0.5),
        uistate::Style::px(50.0),
        0.0,
        1.0,
        0.0,
        0.0,
        f32::MAX,
        [0.0; 4],
    );
    let create = uistate::resolve(
        root_inner,
        uistate::Style::px(0.0),
        uistate::Style::px(-45.0),
        uistate::Style::new(-10.0, 0.5),
        uistate::Style::px(50.0),
        1.0,
        1.0,
        0.0,
        0.0,
        f32::MAX,
        [0.0; 4],
    );

    Layout {
        root,
        panel,
        right,
        info,
        random_name_icon,
        name_button,
        seed_button,
        seed_plate,
        preview,
        separators,
        size_buttons,
        difficulty_buttons,
        evil_buttons,
        description,
        back,
        create,
    }
}

// --- Clicks -----------------------------------------------------------------------

/// What a click does — the `OnLeftMouseDown` subscriptions of
/// `UIWorldCreation.cs:231, 243, 258, 272, 393, 439, 484, 647, 660`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Click {
    /// The `IconRandomName` button (`ClickRandomizeName` -> `AssignRandomWorldName`,
    /// `:497-503`).
    RandomName,
    /// The seed row: the plate's `Click_SetSeed` -> `OpenSeedInputMenu`
    /// (`:697-708`) and the icon's `ClickAdvancedSeedMenu` (`:505-511`) — the
    /// Advanced tab is out of scope (plan Uncertainty 4), so the icon's click
    /// degrades to the same plain seed keyboard (documented deviation).
    Seed,
    /// A size option (`ClickSizeOption`, `:541-551`) — the `WorldSizeId`.
    Size(u8),
    /// A difficulty option (`ClickDifficultyOption`, `:553-563`) — the
    /// `WorldDifficultyId` (display order Journey/Classic/Expert/Master).
    Difficulty(u8),
    /// An evil option (`ClickEvilOption`, `:565-575`).
    Evil(u8),
    /// The name plate (`Click_SetName` -> `UIVirtualKeyboard`, `:689-695`).
    Name,
    /// The Create panel (`Click_NamingAndCreating` -> `FinishCreatingWorld`,
    /// `:710-724`).
    Create,
    /// The Back panel (`Click_GoBack` -> `GoBack` -> `OpenWorldSelectUI`,
    /// `:665-674`).
    Back,
    None,
}

/// The click table over the live geometry, in append order: the two 40x40
/// icons, the two plates, the three option rows, then Back/Create. Nothing
/// overlaps (the info rows end 19 px above the description, and the pair sits
/// 9 px below the panel), so the order only mirrors the C#'s tree.
pub fn click_at(l: &Layout, mouse: Vec2) -> Click {
    if in_rect(mouse, l.random_name_icon) {
        return Click::RandomName; // `:497-503`
    }
    if in_rect(mouse, l.seed_button) {
        return Click::Seed; // `:505-511` degraded, see `Click::Seed`
    }
    if in_rect(mouse, l.name_button) {
        return Click::Name; // `:689-695`
    }
    if in_rect(mouse, l.seed_plate) {
        return Click::Seed; // `:697-708`
    }
    for (k, id) in SIZE_ROW_IDS.iter().enumerate() {
        if in_rect(mouse, l.size_buttons[k]) {
            return Click::Size(*id); // `:541-551`
        }
    }
    for (k, id) in DIFFICULTY_ROW_IDS.iter().enumerate() {
        if in_rect(mouse, l.difficulty_buttons[k]) {
            return Click::Difficulty(*id); // `:553-563`
        }
    }
    for (k, id) in EVIL_ROW_IDS.iter().enumerate() {
        if in_rect(mouse, l.evil_buttons[k]) {
            return Click::Evil(*id); // `:565-575`
        }
    }
    if in_rect(mouse, l.back) {
        return Click::Back; // `:665-668`
    }
    if in_rect(mouse, l.create) {
        return Click::Create; // `:710-724`
    }
    Click::None
}

// --- The preview (`UIWorldCreationPreview.cs`) -------------------------------------

/// `UpdateOption`'s layer mapping (`UIWorldCreationPreview.cs:64-69` +
/// `DrawSelf`'s switches, `:71-132`), returned as
/// `(background, size, evil, bunny)` — indices into `Lookups::wc_preview_bg`
/// (Normal1/Expert1/Master1), `wc_preview_size`, `wc_preview_evil` and
/// `wc_preview_bunny` (Normal2/Creative2/Expert2/Master2 in the Lookups'
/// build order). The bg switch: difficulty 0 and 3 both draw Normal1 (`:76-82`),
/// 1 Expert1, 2 Master1; the bunny switch adds Creative2 for difficulty 3
/// (`:116-130`).
pub fn preview_layers(draft: &NewWorldDraft) -> (usize, usize, usize, usize) {
    let bg = match draft.difficulty {
        1 => 1,             // `_BackgroundExpertTexture` (`:83-86`)
        2 => 2,             // `_BackgroundMasterTexture` (`:87-90`)
        _ => 0,             // 0/3 -> `_BackgroundNormalTexture` (`:76-82`)
    };
    let bunny = match draft.difficulty {
        0 => 0, // `_BunnyNormalTexture` (`:118-120`)
        1 => 2, // `_BunnyExpertTexture` (`:121-123`)
        2 => 3, // `_BunnyMasterTexture` (`:124-126`)
        3 => 1, // `_BunnyCreativeTexture` (`:127-129`)
        _ => 0,
    };
    (bg, draft.size as usize, draft.evil as usize, bunny)
}

/// XNA `Color.DarkGray` — the size/evil/bunny tint under Expert/Master
/// (`UIWorldCreationPreview.cs:85,89`).
const DARK_GRAY: [u8; 4] = [169, 169, 169, 255];

/// `UIWorldCreationPreview.DrawSelf` (`:71-132`): the 76x76 layer stack at
/// `dimensions + (4, 4)` (`:74`), then the 84x84 `PreviewBorder` at the
/// element origin (`:131`). The bg layer draws White; the size/evil layers
/// and the bunny draw the difficulty tint (`color`, `:75-91`), the Master
/// bunny multiplied `* 1.2` (`:125`) through the XNA `(byte)` wrap — e.g.
/// White → (50, 50, 50, 50), DarkGray → (202, 202, 202, 50): the vanilla
/// Master bunny draws translucent.
pub fn draw_preview(r: &mut SpriteRenderer, lookups: &assets::Lookups, l: &Layout, draft: &NewWorldDraft) {
    let (bg, size, evil, bunny) = preview_layers(draft);
    let pos = [l.preview[0] + 4.0, l.preview[1] + 4.0]; // `:74`
    let frame = [76.0, 76.0];
    let color = match draft.difficulty {
        1 | 2 => DARK_GRAY,          // `:85`, `:89`
        _ => [255, 255, 255, 255],   // `:81`
    };
    // The background: White (`:80`, `:84`, `:88`).
    push_whole(r, lookups, lookups.wc_preview_bg[bg], pos, frame, [1.0; 4]);
    // The size and evil overlays (`:92-115`).
    push_whole(r, lookups, lookups.wc_preview_size[size], pos, frame, color::tint(color));
    push_whole(r, lookups, lookups.wc_preview_evil[evil], pos, frame, color::tint(color));
    // The bunny (`:116-130`), the Master one `* 1.2` (`:125`) — the XNA
    // `Color * float` `(byte)` wrap (mod 256, `color::mul`), not a clamp.
    let bunny_tint = if draft.difficulty == 2 {
        color::mul(color, 1.2)
    } else {
        color
    };
    push_whole(r, lookups, lookups.wc_preview_bunny[bunny], pos, frame, color::tint(bunny_tint));
    // The border at the element origin (`:131`), the element's own 84x84.
    push_whole(
        r,
        lookups,
        lookups.wc_preview_border,
        [l.preview[0], l.preview[1]],
        [l.preview[2], l.preview[3]],
        [1.0; 4],
    );
}

// --- World creation (`FinishCreatingWorld`) ------------------------------------------

/// The `(width, height)` pair per size row (`WorldSizeId`): the legacy
/// `menuMode == 16` table's `maxTilesX/maxTilesY` — Small 4200x1200
/// (`Main.cs:50990-50991`), Medium 6400x1800 (`:50995-50996`), Large
/// 8400x2400 (`:51000-51001`) — the tile counts the modern Create button
/// generates with.
pub fn size_dimensions(size: u8) -> (i32, i32) {
    match size {
        0 => (4200, 1200),
        2 => (8400, 2400),
        _ => (6400, 1800),
    }
}

/// `ReLogic.Utilities.Crc32.Calculate(string)` (`Crc32.cs`): the standard
/// reflected CRC-32 — poly `0xEDB88320`, init `0xFFFFFFFF`, final complement —
/// returning `(int)(~num)`. The `Tabs` table (`Crc32.cs:9-59`) is derived in
/// the const block instead of pasted; the loop walks the string's UTF-16
/// chars like the C# `foreach (char c in value)` (`Crc32.cs:66-71`), each
/// char contributing its low byte (`(num ^ c) & 0xFF`).
fn crc32(text: &str) -> i32 {
    const TABS: [u32; 256] = {
        let mut t = [0u32; 256];
        let mut i = 0usize;
        while i < 256 {
            let mut n = i as u32;
            let mut k = 0;
            while k < 8 {
                n = if n & 1 != 0 { 0xEDB8_8320 ^ (n >> 1) } else { n >> 1 };
                k += 1;
            }
            t[i] = n;
            i += 1;
        }
        t
    };
    let mut num = u32::MAX;
    for c in text.encode_utf16() {
        num = TABS[((num ^ c as u32) & 0xFF) as usize] ^ (num >> 8);
    }
    (!num) as i32
}

/// `WorldFileData.TranslateSeed` (`WorldFileData.cs:380-392`): a numeric text
/// parses i32 and takes `Math.Abs` (`"-5"` -> 5; `int.MinValue` clamps to
/// `int.MaxValue`); everything else — non-numeric, out-of-i32, or empty —
/// goes through [`crc32`]. (`int.TryParse`'s `NumberStyles.Integer` also
/// tolerates surrounding whitespace, mimicked with the trim; the CRC still
/// hashes the raw text.)
fn translate_seed(seed_text: &str) -> i32 {
    match seed_text.trim().parse::<i32>() {
        Ok(n) if n != i32::MIN => n.abs(),
        Ok(_) => i32::MAX,
        Err(_) => crc32(seed_text),
    }
}

/// `FinishCreatingWorld`'s seed step (`UIWorldCreation.cs:852-858`): a blank
/// seed takes `SetSeedToRandomWithCurrentEvents` (`:852-855`), a filled one
/// `SetSeed(text)` (`:856-858`) whose translation is `translate_seed`. The
/// port has no `Main.rand` session, so the blank case hashes the name through
/// the same `Crc32` — deterministic per name, documented as the random-seed
/// stand-in. The i32 widens zero-extended to the pipeline's u64 (`as u32 as
/// u64` — the 32-bit pattern the vanilla seed carries).
pub fn seed_value(name: &str, seed_text: &str) -> u64 {
    if seed_text.is_empty() {
        return crc32(name) as u32 as u64;
    }
    translate_seed(seed_text) as u32 as u64
}

/// `FinishCreatingWorld` (`UIWorldCreation.cs:845-863`) through the port's
/// pipeline: the trimmed name, the size row's tile counts, the seed, then
/// `worldgen::generate` + `tr_core::save::save` into `worlds_dir`. The world's
/// evil/difficulty have no `.trwld` fields in the port's header — documented
/// deviation (the listing reads GameMode as Classic, `save_list.rs`). The
/// seed text: vanilla truncates it first with `Utils.TrimUserString(
/// this._optionSeed, 40)` (`UIWorldCreation.cs:828`;
/// `WorldFileData.MAX_USER_SEED_TEXT_LENGTH = 40`, `WorldFileData.cs:481`) —
/// the port's seed keyboard has no length cap, so the trim is not ported
/// (documented deviation). Returns the saved `.trwld` path.
pub fn create_world_in(worlds_dir: &Path, draft: &NewWorldDraft) -> anyhow::Result<PathBuf> {
    let name = draft.name.trim();
    let (width, height) = size_dimensions(draft.size);
    let seed = seed_value(name, &draft.seed);
    let mut world = tr_core::worldgen::generate(&tr_core::worldgen::GenOptions {
        name: name.to_string(),
        width,
        height,
        seed,
    });
    // `generate` seeds its noise/RNG but leaves `World::new`'s `seed: 0` —
    // the header records the draft's (`save.rs:61`'s `world.seed` write).
    world.seed = seed;
    let path = worlds_dir.join(format!("{name}.trwld"));
    tr_core::save::save(&path, &world)?;
    Ok(path)
}

/// [`create_world_in`] into `tr_core::save::default_world_dir` — the App's
/// Create tail. The dir resolves through the `USERPROFILE` stand-in for
/// `Main.WorldPath` (`save.rs:20-29`).
pub fn create_world(draft: &NewWorldDraft) -> anyhow::Result<PathBuf> {
    let dir = tr_core::save::default_world_dir()
        .context("cannot resolve Documents/My Games/Terrustaria/Worlds")?;
    create_world_in(&dir, draft)
}

// --- Draw ----------------------------------------------------------------------------

/// One atlas sprite stretched over `pos`/`size` from the whole `sheet`. `None`
/// draws nothing (`Lookups::default()` stays blank, Review Focus 7) — the
/// Option-guarded adapter over `ui::push_whole`, which carries the shared UV
/// maths.
fn push_whole(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    sheet: Option<tr_render::SheetRect>,
    pos: [f32; 2],
    size: [f32; 2],
    tint: [f32; 4],
) {
    if let Some(sheet) = sheet {
        super::push_whole(r, lookups, &sheet, [pos[0], pos[1], size[0], size[1]], tint);
    }
}

/// The Mouse-font width of `text` at scale 1 — the `UIText` metric the plate
/// value offsets read (`UICharacterNameButton.cs:98`).
fn mouse_text_width(r: &SpriteRenderer, lookups: &assets::Lookups, text: &str) -> f32 {
    hud_text_width(r, lookups, text, 1.0)
}

// The 0.7-large fat panel style of Back/Create (`MakeBackAndCreatebuttons`,
// `:637-662`) is `ui::cards::fat_button_style` — the `UICharacterSelect`
// twin's shared copy.

/// The plate strings (`UICharacterNameButton`'s ctor args, `:236` and `:264`).
const NAME_TITLE: &str = "Name:";             // UI.WorldCreationName
const SEED_TITLE: &str = "Seed:";             // UI.WorldCreationSeed
const NAME_EMPTY: &str = "<Empty>";           // UI.WorldCreationNameEmpty
const SEED_EMPTY: &str = "<Leave Blank For Random>"; // UI.WorldCreationSeedEmpty

/// `UICharacterNameButton.DrawSelf` (`UICharacterNameButton.cs:63-83`) plus
/// the title/value text (`:44-59`, `SetContents` `:85-99`): the spliced
/// `CategoryPanel` at `White * 0.5`, the hovered `CategoryPanelBorder` white,
/// the title at `Left 10` and the value at `titleWidth +
/// DistanceFromTitleToOption` (`:98`; 20 default `:32`, 29 for the seed,
/// `:270`). An empty value shows the empty text in Gray (`:88-91`).
/// The displayed value: vanilla trims it to fit with
/// `TrimDisplayIfOverElementDimensions` — 27 for the name, 40 for the seed
/// (`UIWorldCreation.cs:737, 746`) — which the port does not port; it draws
/// the raw strings (documented deviation).
fn draw_plate(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    rect: [f32; 4],
    title: &str,
    value: &str,
    empty_text: &str,
    distance: f32,
    mouse: Vec2,
) {
    let hovered = in_rect(mouse, rect);
    uistate::draw_spliced_panel(
        r,
        lookups,
        lookups.cc_panel,
        rect,
        10.0,
        10.0,
        10.0,
        10.0,
        [0.5; 4], // `Color.White * 0.5f` (`:78`)
    );
    if hovered {
        uistate::draw_spliced_panel(
            r,
            lookups,
            lookups.cc_panel_border,
            rect,
            10.0,
            10.0,
            10.0,
            10.0,
            [1.0; 4], // `Color.White` (`:81`)
        );
    }
    let cy = rect[1] + rect[3] * 0.5;
    let title_w = mouse_text_width(r, lookups, title);
    // The title `UIText`: `HAlign 0, VAlign 0.5, Left 10` (`:44-49`).
    border_string(r, lookups, rect[0] + 10.0, cy - 7.0, [1.0; 4], title);
    let (text, tint) = if value.is_empty() {
        (empty_text, color::tint([128, 128, 128, 255])) // `Color.Gray` (`:90`)
    } else {
        (value, [1.0; 4]) // `Color.White` (`:95`)
    };
    border_string(
        r,
        lookups,
        rect[0] + 10.0 + title_w + distance,
        cy - 7.0,
        tint,
        text,
    );
}

/// One `GroupOptionButton` (`GroupOptionButton.cs:210-260`): the
/// `PanelGrayscale` fill, the selected `CategoryPanelHighlight` rim inset by
/// `InnerHighlightRim = 7` (`:32`), the hovered `CategoryPanelBorder` white,
/// then the 32x32 icon at `+1, +1` (`:249`) — tinted White while
/// hovered/selected, the rim formula otherwise (`:244-248`). `title` draws at
/// the row tint, centred 8px right of the button's centre: the rows build the
/// title UIText with `titleAlignmentX = 1f, titleWidthReduction = 16f`
/// (`UIWorldCreation.cs:388, 437, 482`) — `Width (-16, 1)`, `HAlign 1`.
fn draw_option_button(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    rect: [f32; 4],
    title: &str,
    title_color: [u8; 4],
    icon: Option<tr_render::SheetRect>,
    selected: bool,
    hovered: bool,
) {
    let (fill, accent) = option_button_colors(selected, hovered, OPTION_BUTTON_COLOR);
    uistate::draw_spliced_panel(
        r,
        lookups,
        lookups.cc_panel_gray,
        rect,
        10.0,
        10.0,
        10.0,
        10.0,
        color::tint(fill),
    );
    if selected {
        let rim = 7.0; // `InnerHighlightRim` (`GroupOptionButton.cs:32`)
        uistate::draw_spliced_panel(
            r,
            lookups,
            lookups.cc_panel_highlight,
            [
                rect[0] + rim,
                rect[1] + rim,
                rect[2] - rim * 2.0,
                rect[3] - rim * 2.0,
            ],
            10.0,
            10.0,
            10.0,
            10.0,
            color::tint(accent),
        );
    }
    if hovered {
        uistate::draw_spliced_panel(
            r,
            lookups,
            lookups.cc_panel_border,
            rect,
            10.0,
            10.0,
            10.0,
            10.0,
            color::tint(accent), // `_borderColor` = White (`:120`)
        );
    }
    // The icon at `+1, +1` (`:249`), 32x32 as decoded (`UIWorldCreation.cs:
    // 384, 433, 478`). The tint: White while hovered/selected (`:245`), the
    // rim formula otherwise (`:247`).
    let icon_tint = if hovered || selected {
        [255, 255, 255, 255]
    } else {
        option_button_colors(false, false, OPTION_BUTTON_COLOR).1
    };
    push_whole(
        r,
        lookups,
        icon,
        [rect[0] + 1.0, rect[1] + 1.0],
        [32.0, 32.0],
        color::tint(icon_tint),
    );
    // The title `UIText` (`:136-144`): scale 1, V-centred. The rows build it
    // `titleAlignmentX = 1f, titleWidthReduction = 16f` (`UIWorldCreation.cs:
    // 388, 437, 482`) — `Width (-16, 1)`, `HAlign 1` — so the box ends 16px
    // short of the button's right edge and `UIText` centres the text on it
    // (`TextOriginX = 0.5`): the centre sits 8px right of the button's.
    let w = mouse_text_width(r, lookups, title);
    hud_text(
        r,
        lookups,
        rect[0] + (rect[2] - w) * 0.5 + 8.0,
        rect[1] + (rect[3] - 14.0) * 0.5,
        1.0,
        color::tint(title_color),
        title,
    );
}

/// The 40x40 `GroupOptionButton<bool>` icon pair (`:223-234`, `:250-263`):
/// the same `GroupOptionButton` chrome with `ShowHighlightWhenSelected =
/// false` (no rim even selected, `:229`/`:256`) and a 38x38 icon. Both
/// construct `option: true` (`:223`, `:250`) and are never re-optioned, so
/// `IsSelected` always holds (`GroupOptionButton.cs:68`) and the icon keeps
/// `IconColor` = White in every state.
fn draw_icon_button(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    rect: [f32; 4],
    icon: Option<tr_render::SheetRect>,
    hovered: bool,
) {
    // The fill only ever reads `_color` (`:233` — the same
    // `Colors.InventoryDefaultColor` as the option rows).
    let (fill, _) = option_button_colors(false, false, OPTION_BUTTON_COLOR);
    uistate::draw_spliced_panel(
        r,
        lookups,
        lookups.cc_panel_gray,
        rect,
        10.0,
        10.0,
        10.0,
        10.0,
        color::tint(fill),
    );
    if hovered {
        uistate::draw_spliced_panel(
            r,
            lookups,
            lookups.cc_panel_border,
            rect,
            10.0,
            10.0,
            10.0,
            10.0,
            [1.0; 4], // `_borderColor` = White (`GroupOptionButton.cs:120`)
        );
    }
    // The icon keeps `IconColor` = White in every state: constructed
    // `option: true` (`UIWorldCreation.cs:223, 250`) and never re-optioned,
    // `IsSelected` always holds (`GroupOptionButton.cs:68`), so the
    // `!hovered && !isSelected` recolour (`:245-247`) never fires;
    // `ShowHighlightWhenSelected = false` mutes only the rim.
    let icon_tint = [255, 255, 255, 255];
    push_whole(
        r,
        lookups,
        icon,
        [rect[0] + 1.0, rect[1] + 1.0],
        [38.0, 38.0], // decoded 38x38 (`UIWorldCreation.cs:223, 250`)
        color::tint(icon_tint),
    );
}

/// The hovered element's description (`ShowOptionDescription`/`ClearOption-
/// Description`, `UIWorldCreation.cs:601-633`): the option's own
/// `WorldDescription*` string, or the default when nothing hovers.
fn hover_description(l: &Layout, mouse: Vec2) -> &'static str {
    const SIZE_DESCRIPTIONS: [&str; 3] = [
        // `UI.WorldDescriptionSizeSmall/Medium/Large`
        "Ideal for a quick, single player experience.",
        "Bigger and better for prolonged exploration, or bringing a friend.",
        "Super-sized for long term progression and multiple players.",
    ];
    const DIFFICULTY_DESCRIPTIONS: [&str; 4] = [
        // `UI.WorldDescriptionCreative/Normal/Expert/Master`
        "With great power comes great responsibility (Unearth your creativity)",
        "Your Adventure Begins... (The Standard Terraria Experience)",
        "Fortune & Glory, Kid. (Far Greater Difficulty & Loot)",
        "Game over man, game over! (Brutally hard, for the truly brave)",
    ];
    const EVIL_DESCRIPTIONS: [&str; 3] = [
        // `UI.WorldDescriptionEvilRandom/Corrupt/Crimson`
        "Let nature decide whether corruption or crimson infects your world.",
        "Disease-like corruption is guaranteed to be present in your world.",
        "Macabre, grotesque crimson is guaranteed to be present in your world.",
    ];
    for (k, text) in SIZE_DESCRIPTIONS.into_iter().enumerate() {
        if in_rect(mouse, l.size_buttons[k]) {
            return text;
        }
    }
    for (k, text) in DIFFICULTY_DESCRIPTIONS.into_iter().enumerate() {
        if in_rect(mouse, l.difficulty_buttons[k]) {
            return text;
        }
    }
    for (k, text) in EVIL_DESCRIPTIONS.into_iter().enumerate() {
        if in_rect(mouse, l.evil_buttons[k]) {
            return text;
        }
    }
    // The plates and icons (`:616-623`'s `UICharacterNameButton`/
    // `GroupOptionButton<bool>` arms).
    if in_rect(mouse, l.name_button) {
        return "Give your world a name! You can always change it later if needed!"; // UI.WorldDescriptionName
    }
    if in_rect(mouse, l.seed_plate) {
        return "The DNA marker that defines your world's specific shape and traits."; // UI.WorldDescriptionSeed
    }
    if in_rect(mouse, l.random_name_icon) {
        return "Randomize your world's name."; // UI.WorldCreationRandomizeNameDescription
    }
    if in_rect(mouse, l.seed_button) {
        // UI.WorldCreationSeedMenuDescription — the Advanced tab's entry
        // point; the tab itself is out of scope (plan Uncertainty 4).
        return "Customize your world's seed, or choose some game-changing ones!";
    }
    "Please choose how your world will be built with the options above." // UI.WorldDescriptionDefault
}

/// The whole screen in the source's z-order (`UIElement.Draw` draws self, then
/// the children in append order): the panel, the info subtree (the two icon
/// buttons, the two plates, the preview, the four separators, the three option
/// rows, the description), then Back/Create — root-level siblings appended
/// right after the panel (`:186`) whose subtree still draws first; the pair
/// overlaps nothing, so the port draws it last like Task 7 does.
pub fn draw(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    draft: &NewWorldDraft,
    screen: Vec2,
    mouse: Vec2,
) {
    let l = layout(screen, 0.0);

    // The panel: bg `(33, 43, 79) * 0.8` (`:182`), border the `UIPanel`
    // default black (`UIPanel.cs:18`).
    let panel_bg = color::mul([33, 43, 79, 255], 0.8);
    panel::draw_panel(r, lookups, lookups.ui_panel_bg, l.panel, color::tint(panel_bg));
    panel::draw_panel(
        r,
        lookups,
        lookups.ui_panel_border,
        l.panel,
        color::tint(uistate::TEXT_PANEL_BORDER),
    );

    // The info subtree, in `MakeInfoMenu`'s append order (`:223-356`).
    draw_icon_button(r, lookups, l.random_name_icon, lookups.wc_icons[0], in_rect(mouse, l.random_name_icon));
    draw_plate(
        r,
        lookups,
        l.name_button,
        NAME_TITLE,
        &draft.name,
        NAME_EMPTY,
        20.0, // `DistanceFromTitleToOption` default (`UICharacterNameButton.cs:32`)
        mouse,
    );
    draw_icon_button(r, lookups, l.seed_button, lookups.wc_icons[1], in_rect(mouse, l.seed_button));
    draw_plate(
        r,
        lookups,
        l.seed_plate,
        SEED_TITLE,
        &draft.seed,
        SEED_EMPTY,
        29.0, // `DistanceFromTitleToOption = 29f` (`UIWorldCreation.cs:270`)
        mouse,
    );
    draw_preview(r, lookups, &l, draft);

    // The separators (`:303-311`): `Color.Lerp(White, (63,65,151), 0.85) * 0.9`
    // — the shared `CREATION_SEPARATOR_COLOR`.
    for sep in l.separators {
        panel::draw_horizontal_separator(
            r,
            lookups,
            sep[0],
            sep[1],
            sep[2],
            color::tint(uistate::CREATION_SEPARATOR_COLOR),
        );
    }

    // The option rows (`:358-495`), each button against the live draft.
    for (k, (title, tint)) in SIZE_ROWS.iter().enumerate() {
        let rect = l.size_buttons[k];
        draw_option_button(
            r,
            lookups,
            rect,
            title,
            *tint,
            lookups.wc_size_icons[k],
            draft.size == SIZE_ROW_IDS[k],
            in_rect(mouse, rect),
        );
    }
    for (k, (title, tint)) in DIFFICULTY_ROWS.iter().enumerate() {
        let rect = l.difficulty_buttons[k];
        draw_option_button(
            r,
            lookups,
            rect,
            title,
            *tint,
            lookups.wc_difficulty_icons[k],
            draft.difficulty == DIFFICULTY_ROW_IDS[k],
            in_rect(mouse, rect),
        );
    }
    for (k, (title, tint)) in EVIL_ROWS.iter().enumerate() {
        let rect = l.evil_buttons[k];
        draw_option_button(
            r,
            lookups,
            rect,
            title,
            *tint,
            lookups.wc_evil_icons[k],
            draft.evil == EVIL_ROW_IDS[k],
            in_rect(mouse, rect),
        );
    }

    // The description panel (`AddDescriptionPanel`, `:328-356`): the
    // `CategoryPanelHighlight` sliced panel at `LightGray * 0.7` with the
    // hovered description at 0.82 (`:343`), the 20 px side padding and
    // `Top 5 + PaddingTop 6` inside (`:347-353`).
    uistate::draw_spliced_panel(
        r,
        lookups,
        lookups.cc_panel_highlight,
        l.description,
        10.0,
        10.0,
        10.0,
        10.0, // `SetSliceDepths(10)` (`:340`)
        color::tint(color::mul([211, 211, 211, 255], 0.7)), // `Color.LightGray * 0.7f` (`:341`)
    );
    let desc = hover_description(&l, mouse);
    hud_text(
        r,
        lookups,
        l.description[0] + 20.0,  // `PaddingLeft = 20` (`:351`)
        l.description[1] + 11.0,  // `Top 5` (`:349`) + `PaddingTop 6` (`:353`)
        0.82,                     // the `UIText` scale (`:343`)
        [1.0; 4],
        desc,
    );

    // Back / Create (`:635-663`): the hover pair swaps inside
    // `uistate::draw_text_panel` (`FadedMouseOver`, `:676-687`).
    uistate::draw_text_panel(
        r,
        lookups,
        l.back,
        &fat_button_style(),
        "Back",
        mouse_text_width(r, lookups, "Back"),
        in_rect(mouse, l.back),
    );
    uistate::draw_text_panel(
        r,
        lookups,
        l.create,
        &fat_button_style(),
        "Create",
        mouse_text_width(r, lookups, "Create"),
        in_rect(mouse, l.create),
    );
}

// --- Tests ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // Expected values hand-derived from the C# (`UIWorldCreation.cs:163-663`,
    // `UIWorldCreationPreview.cs:44-132`, `GroupOptionButton.cs:118-260`,
    // `UICharacterNameButton.cs:34-99`) — never from this code.

    /// The full 1600x900 layout, asserted against the C#-derived table.
    #[test]
    fn layout_matches_the_source_geometry() {
        let l = layout(Vec2::new(1600.0, 900.0), 40.0);
        // `:166-176` — 500 x (434+18) at Top 170-18, HAlign .5 of 1600.
        assert_eq!(l.root, [550.0, 152.0, 500.0, 452.0]);
        // `:177-185` — 100% x (280+18) at Top 50.
        assert_eq!(l.panel, [550.0, 202.0, 500.0, 298.0]);
        // `:187-197` — the full-panel rect (HAlign 1 of an equal width).
        assert_eq!(l.right, [550.0, 202.0, 500.0, 298.0]);
        // `:208-218` — the `SetPadding(10)` container's inner.
        assert_eq!(l.info, [560.0, 210.0, 480.0, 278.0]);
        // `:223-234` — the 40x40 icon at HAlign 0.
        assert_eq!(l.random_name_icon, [560.0, 210.0, 40.0, 40.0]);
        // `:236-248` — `Width (-132, 1)` = 348, `Left num2 = 44`.
        assert_eq!(l.name_button, [604.0, 210.0, 348.0, 40.0]);
        // `:250-263` — `num += 40 + 4` after the name row (`:249`).
        assert_eq!(l.seed_button, [560.0, 254.0, 40.0, 40.0]);
        assert_eq!(l.seed_plate, [604.0, 254.0, 348.0, 40.0]);
        // `:278-286` — 84x84, HAlign 1 of the 480 inner.
        assert_eq!(l.preview, [956.0, 210.0, 84.0, 84.0]);
        // `:288-299` + `:303-311` — `num` 94+2/142/190/238, each `Top - 8`.
        assert_eq!(l.separators[0], [560.0, 298.0, 480.0, 4.0]);
        assert_eq!(l.separators[1], [560.0, 344.0, 480.0, 4.0]);
        assert_eq!(l.separators[2], [560.0, 392.0, 480.0, 4.0]);
        assert_eq!(l.separators[3], [560.0, 440.0, 480.0, 4.0]);
        // `:358-401` — W `(-8, 1/3)` = 152, H 34 (`GroupOptionButton.cs:125`),
        // `HAlign i/2` over the 480 inner.
        assert_eq!(l.size_buttons[0], [560.0, 304.0, 152.0, 34.0]);
        assert_eq!(l.size_buttons[1], [724.0, 304.0, 152.0, 34.0]);
        assert_eq!(l.size_buttons[2], [888.0, 304.0, 152.0, 34.0]);
        // `:403-450` — W `(-3, 1/4)` = 117, `HAlign i/3`.
        assert_eq!(l.difficulty_buttons[0], [560.0, 352.0, 117.0, 34.0]);
        assert_eq!(l.difficulty_buttons[1], [681.0, 352.0, 117.0, 34.0]);
        assert_eq!(l.difficulty_buttons[2], [802.0, 352.0, 117.0, 34.0]);
        assert_eq!(l.difficulty_buttons[3], [923.0, 352.0, 117.0, 34.0]);
        // `:452-495` — the size row's geometry at `num = 190`.
        assert_eq!(l.evil_buttons[0], [560.0, 400.0, 152.0, 34.0]);
        assert_eq!(l.evil_buttons[1], [724.0, 400.0, 152.0, 34.0]);
        assert_eq!(l.evil_buttons[2], [888.0, 400.0, 152.0, 34.0]);
        // `:328-356` — `VAlign 1, Top 2` of the 278 inner: 210 + 278 - 40 + 2.
        assert_eq!(l.description, [560.0, 450.0, 480.0, 40.0]);
        // `:637-662` — the pair at `VAlign 1, Top -45` of the root:
        // 152 + 452 - 50 - 45 = 509; W `(-10, 0.5)` = 240.
        assert_eq!(l.back, [550.0, 509.0, 240.0, 50.0]);
        assert_eq!(l.create, [810.0, 509.0, 240.0, 50.0]);
    }

    /// The layout is screen-derived, not hard-coded: a 1280x720 frame scales
    /// the alignment halves (the literals stay).
    #[test]
    fn layout_derives_from_the_screen() {
        let l = layout(Vec2::new(1280.0, 720.0), 40.0);
        assert_eq!(l.root, [390.0, 152.0, 500.0, 452.0]); // 0.5*(1280-500)
        assert_eq!(l.back, [390.0, 509.0, 240.0, 50.0]);
        assert_eq!(l.create, [650.0, 509.0, 240.0, 50.0]);
    }

    #[test]
    fn draft_defaults_follow_set_default_options() {
        // `:313-326` with `WorldList.Count = 0` and a Classic active player.
        let d = NewWorldDraft::new(0, 0);
        assert_eq!(d.name, "World 1"); // the `AssignRandomWorldName` stand-in
        assert_eq!(d.seed, "");        // `ClearSeed` (`:317`)
        assert_eq!(d.size, 1);         // `WorldSizeId.Medium` (`:318`)
        assert_eq!(d.difficulty, 0);   // not Journey -> Normal (`:319-322`)
        assert_eq!(d.evil, 0);         // `WorldEvilId.Random` (`:323`)
        // A Journey active player seeds Creative (`:319-322`).
        assert_eq!(NewWorldDraft::new(2, 3).difficulty, 3);
        assert_eq!(NewWorldDraft::new(2, 3).name, "World 3");
        // `NewWorldClick`'s count rides in (`UIWorldSelect.cs:113`).
        assert_eq!(NewWorldDraft::stand_in_name(5), "World 6");
    }

    #[test]
    fn preview_layers_map_the_difficulty_switches() {
        // `UIWorldCreationPreview.cs:76-91` (bg) and `:116-130` (bunny, over
        // the Lookups' Normal2/Creative2/Expert2/Master2 order).
        let mut d = NewWorldDraft::new(0, 0);
        d.size = 1;
        d.evil = 2;
        // Difficulty 0 (Normal): the Normal bg and the Normal bunny.
        assert_eq!(preview_layers(&d), (0, 1, 2, 0));
        // Difficulty 3 (Creative): the *Normal* bg (`:76-82`) and Creative2.
        d.difficulty = 3;
        assert_eq!(preview_layers(&d), (0, 1, 2, 1));
        // Difficulty 1 (Expert): Expert1 + Expert2.
        d.difficulty = 1;
        assert_eq!(preview_layers(&d), (1, 1, 2, 2));
        // Difficulty 2 (Master): Master1 + Master2.
        d.difficulty = 2;
        assert_eq!(preview_layers(&d), (2, 1, 2, 3));
    }

    #[test]
    fn option_button_colors_recompute_the_xna_chain() {
        let inv = OPTION_BUTTON_COLOR; // `GroupOptionButton.cs:133`
        // The fill: `Lerp(Black, color, 1) * 0.7` = `color * 0.7` (`:233`).
        let (fill, _) = option_button_colors(false, false, inv);
        assert_eq!(fill, [44, 45, 105, 178]);
        // The selected rim: `Lerp(color, White, 0.7) * 0.7` (`:236`), through
        // the f32 vector-space chain (the exact-math G byte 198.0 rounds just
        // under and truncates to 197 — `lerp_rgb`'s pinned behaviour).
        let (_, rim) = option_button_colors(true, false, inv);
        assert_eq!(rim, color::mul(lerp_rgb(inv, [255, 255, 255, 255], 0.7), 0.7));
        assert_eq!(rim, [137, 137, 156, 178]);
        // The hovered border: `_borderColor` White (`:120`, `:240`).
        let (_, hover) = option_button_colors(false, true, inv);
        assert_eq!(hover, [255, 255, 255, 255]);
        // Both states: the border draws last, white wins (`:236` then `:240`).
        let (_, both) = option_button_colors(true, true, inv);
        assert_eq!(both, [255, 255, 255, 255]);
        // The 32x32 option-row idle icon tint reads the rim formula (`:247`;
        // the always-selected 40x40 icons stay White — `draw_icon_button`).
        let (_, idle) = option_button_colors(false, false, inv);
        assert_eq!(idle, rim);
        // A row tint through the same formula (Cyan, had a button used it).
        let (fill, rim) = option_button_colors(false, false, [0, 255, 255, 255]);
        assert_eq!(fill, [0, 178, 178, 178]);
        assert_eq!(rim, [124, 178, 178, 178]);
    }

    #[test]
    fn row_tables_carry_the_source_labels_and_tints() {
        // `:366-383` (Lang.menu[92..94] + Cyan/LimeGreen lerps).
        assert_eq!(SIZE_ROWS[0].0, "Small");
        assert_eq!(SIZE_ROWS[0].1, [0, 255, 255, 255]);
        assert_eq!(SIZE_ROWS[1].1, [25, 230, 152, 255]); // Lerp(Cyan, LimeGreen, 0.5)
        assert_eq!(SIZE_ROWS[2].1, [50, 205, 50, 255]);  // LimeGreen
        // `:412-432` (UI.Creative/Normal/Expert/Master + the Main colours).
        assert_eq!(DIFFICULTY_ROWS[0].0, "Journey");
        assert_eq!(DIFFICULTY_ROWS[0].1, [255, 120, 187, 255]); // creativeModeColor
        assert_eq!(DIFFICULTY_ROWS[1].0, "Classic");
        assert_eq!(DIFFICULTY_ROWS[1].1, [255, 255, 255, 255]);
        assert_eq!(DIFFICULTY_ROWS[2].0, "Expert");
        assert_eq!(DIFFICULTY_ROWS[2].1, [255, 153, 0, 255]); // mcColor
        assert_eq!(DIFFICULTY_ROWS[3].0, "Master");
        assert_eq!(DIFFICULTY_ROWS[3].1, [255, 38, 25, 255]); // hcColor
        // The display order maps to the enum ids (`:405-411`).
        assert_eq!(DIFFICULTY_ROW_IDS, [3, 0, 1, 2]);
        // `:460-477` (Lang.misc[103/101/102] + the XNA named colours).
        assert_eq!(EVIL_ROWS[0].0, "Random");
        assert_eq!(EVIL_ROWS[1].0, "Corruption");
        assert_eq!(EVIL_ROWS[1].1, [147, 112, 219, 255]); // Color.MediumPurple
        assert_eq!(EVIL_ROWS[2].0, "Crimson");
        assert_eq!(EVIL_ROWS[2].1, [205, 92, 92, 255]); // Color.IndianRed
    }

    #[test]
    fn size_dimensions_match_the_vanilla_tile_counts() {
        // `Main.cs:50990-50991`, `:50995-50996`, `:51000-51001`.
        assert_eq!(size_dimensions(0), (4200, 1200));
        assert_eq!(size_dimensions(1), (6400, 1800));
        assert_eq!(size_dimensions(2), (8400, 2400));
    }

    #[test]
    fn seed_value_translates_like_vanilla() {
        // `WorldFileData.TranslateSeed` (`WorldFileData.cs:380-392`): numeric
        // i32 + `Math.Abs` (`"-5"` -> 5, `int.MinValue` -> `int.MaxValue`),
        // everything else through `Crc32.Calculate` — the standard CRC-32
        // (`"123456789"` -> 0xCBF43926). The blank case rides the port's
        // deterministic name stand-in (`crc32(name)`); the i32 widens
        // zero-extended.
        assert_eq!(seed_value("World 1", "42"), 42);
        assert_eq!(seed_value("World 1", "-5"), 5);
        assert_eq!(seed_value("World 1", "-2147483648"), 2147483647);
        // Out-of-i32 and non-numeric texts fall to the CRC (pinned vectors).
        assert_eq!(crc32("123456789") as u32, 0xCBF4_3926);
        assert_eq!(seed_value("World 1", "99999999999"), 4059124685);
        assert_eq!(seed_value("World 1", "corruption"), 2248887921);
        assert_eq!(seed_value("World 1", ""), 4225370552); // crc32("World 1")
    }

    #[test]
    fn create_world_writes_a_parsable_trwld() {
        // `FinishCreatingWorld` (`:845-863`) through the port's pipeline:
        // the header the world list reads carries the draft's name and the
        // size row's width.
        let dir = std::env::temp_dir().join("tr-worldcreate");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let mut draft = NewWorldDraft::new(0, 0);
        draft.name = "Port Test".into();
        draft.size = 0; // Small -> 4200x1200 (`Main.cs:50990-50991`)
        draft.seed = "12345".into();
        let path = create_world_in(&dir, &draft).expect("the world saves");
        assert_eq!(path, dir.join("Port Test.trwld"));
        assert!(path.exists());
        // The re-list parses it: the name/width round-trip through the
        // `tr_core::save` header (`save_list::list_worlds`).
        let listed = super::super::save_list::list_worlds(&dir, 0);
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].name, "Port Test");
        assert_eq!(listed[0].size_id, 0); // `size_bucket(4200)`
        assert!(listed[0].valid);
        // The seed rode in as the generation seed (the header's u64).
        let bytes = std::fs::read(&path).unwrap();
        assert_eq!(&bytes[0..8], b"TRWLD01\0"); // `save.rs:16`'s magic
        let seed = u64::from_le_bytes(bytes[12..20].try_into().unwrap());
        assert_eq!(seed, 12345);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn click_at_maps_every_control_centre() {
        let l = layout(Vec2::new(1600.0, 900.0), 40.0);
        let centre = |r: [f32; 4]| Vec2::new(r[0] + r[2] * 0.5, r[1] + r[3] * 0.5);
        assert_eq!(click_at(&l, centre(l.random_name_icon)), Click::RandomName);
        assert_eq!(click_at(&l, centre(l.seed_button)), Click::Seed);
        assert_eq!(click_at(&l, centre(l.name_button)), Click::Name);
        assert_eq!(click_at(&l, centre(l.seed_plate)), Click::Seed);
        assert_eq!(click_at(&l, centre(l.size_buttons[0])), Click::Size(0));
        assert_eq!(click_at(&l, centre(l.size_buttons[2])), Click::Size(2));
        // The display order Journey/Classic/Expert/Master -> ids 3/0/1/2.
        assert_eq!(click_at(&l, centre(l.difficulty_buttons[0])), Click::Difficulty(3));
        assert_eq!(click_at(&l, centre(l.difficulty_buttons[1])), Click::Difficulty(0));
        assert_eq!(click_at(&l, centre(l.difficulty_buttons[3])), Click::Difficulty(2));
        assert_eq!(click_at(&l, centre(l.evil_buttons[1])), Click::Evil(1));
        assert_eq!(click_at(&l, centre(l.back)), Click::Back);
        assert_eq!(click_at(&l, centre(l.create)), Click::Create);
        // Empty space clicks nothing.
        assert_eq!(click_at(&l, Vec2::new(100.0, 100.0)), Click::None);
    }

    #[test]
    fn lookups_default_leaves_every_creation_sheet_none() {
        // Review Focus 7: the draw helpers guard every sheet with `Option`,
        // so the default set renders the chrome blank without panicking.
        let lookups = assets::Lookups::default();
        assert!(lookups.wc_preview_border.is_none());
        assert!(lookups.wc_icons.iter().all(|s| s.is_none()));
        assert!(lookups.wc_size_icons.iter().all(|s| s.is_none()));
        assert!(lookups.wc_difficulty_icons.iter().all(|s| s.is_none()));
        assert!(lookups.wc_evil_icons.iter().all(|s| s.is_none()));
        assert!(lookups.wc_preview_bg.iter().all(|s| s.is_none()));
        assert!(lookups.wc_preview_bunny.iter().all(|s| s.is_none()));
        assert!(lookups.wc_preview_evil.iter().all(|s| s.is_none()));
        assert!(lookups.wc_preview_size.iter().all(|s| s.is_none()));
        assert!(lookups.cc_panel.is_none());
        assert!(lookups.cc_panel_gray.is_none());
        assert!(lookups.cc_panel_highlight.is_none());
        assert!(lookups.cc_panel_border.is_none());
    }

    #[test]
    fn master_bunny_tint_wraps_like_xna() {
        // `color * 1.2f` (`UIWorldCreationPreview.cs:125`): XNA's op_Multiply
        // casts through `(byte)`, which wraps modulo 256 (ECMA-335 conv.u1 —
        // the repo convention, `color::mul`); nothing clamps.
        // White * 1.2 = 306 -> 50 on every channel.
        assert_eq!(color::mul([255, 255, 255, 255], 1.2), [50, 50, 50, 50]);
        // DarkGray * 1.2 = 202.8 -> 202 per colour channel; the alpha wraps
        // 306 -> 50 too (`:89`'s tint path — the vanilla ghosted bunny).
        assert_eq!(color::mul([169, 169, 169, 255], 1.2), [202, 202, 202, 50]);
    }
}
