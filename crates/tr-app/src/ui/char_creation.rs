//! The character creation screen (`Main.menuMode == 888`'s
//! `UICharacterCreation`): the 500x384 root, the animated 80x80 player
//! preview, the ten-slot categories bar, the Info / HSL / Hairstyles /
//! Cloth-styles middle menus and the Back/Create pair — a faithful immediate
//! port, the same `layout()` + `draw()` shape as `ui/char_select.rs`
//! (plan §"Screen-model ruling").
//!
//! What the C# keeps on the `UIState` itself (`_selectedPicker`, the
//! `UICharacterCreation.dirty` flag, the slider grab) rides
//! [`CreationState`] next to the [`NewPlayerDraft`] (the `Main.PendingPlayer`
//! port) — the exact value the mode-40 discard round-trip parks in
//! `App.front_backup` (`UICharacterCreation.BackupConfirmationState`).
//!
//! Every draw reads the sheets from `Lookups` `Option`s: a missing sheet draws
//! nothing, so `Lookups::default()` renders blank chrome and panics nowhere
//! (plan Review Focus 7).
//!
//! Also not ported (documented deviation): the ctor's `GameTipsDisplay`
//! (`:170`) — the rotating hint line the `Draw` tail paints under the panel
//! (`:1998-2005`); the ANALYSIS pass records it in Task 9.

use glam::Vec2;
use tr_render::SpriteRenderer;

use super::cards::fat_button_style;
use super::color;
use super::in_rect;
use super::panel;
use super::push_whole;
use super::sliders;
use super::text::{border_string, hud_text, hud_text_width};
use super::uistate;
use crate::assets;

// --- The draft (Main.PendingPlayer) ------------------------------------------

/// `Main.PendingPlayer` — the fields the screen edits. Defaults from
/// `Player.cs:826,2554-2566` and `difficulty = 0`
/// (`UICharacterCreation.cs:169`'s ctor assignment).
#[derive(Clone, Debug, PartialEq)]
pub struct NewPlayerDraft {
    /// `Player.name = ""` (`Player.cs:826`).
    pub name: String,
    /// `this._player.difficulty = 0` (`UICharacterCreation.cs:169`).
    pub difficulty: u8,
    /// `public int hair;` (`Player.cs:2570`) — default 0.
    pub hair: i32,
    /// `public int skinVariant` — the cloth style
    /// (`Click_CharClothStyle`, `UICharacterCreation.cs:1286`).
    pub skin_variant: i32,
    /// `public Color hairColor = new Color(215, 90, 55)` (`Player.cs:2554`).
    pub hair_color: [u8; 4],
    /// `public Color eyeColor = new Color(105, 90, 75)` (`Player.cs:2558`).
    pub eye_color: [u8; 4],
    /// `public Color skinColor = new Color(255, 125, 90)` (`Player.cs:2556`).
    pub skin_color: [u8; 4],
    /// `public Color shirtColor = new Color(175, 165, 140)` (`Player.cs:2560`).
    pub shirt_color: [u8; 4],
    /// `public Color underShirtColor = new Color(160, 180, 215)` (`Player.cs:2562`).
    pub undershirt_color: [u8; 4],
    /// `public Color pantsColor = new Color(255, 230, 175)` (`Player.cs:2564`).
    pub pants_color: [u8; 4],
    /// `public Color shoeColor = new Color(160, 105, 60)` (`Player.cs:2566`).
    pub shoe_color: [u8; 4],
    /// `UICharacterCreation.dirty` (`:154`): the Draw-time check of `:2011-2021`
    /// — the name went non-empty or the template (hair/cloth/colours) left
    /// [`NewPlayerDraft::new`]'s initial state. Back consults it (`:1213`).
    pub dirty: bool,
}

impl Default for NewPlayerDraft {
    fn default() -> Self {
        Self::new()
    }
}

impl NewPlayerDraft {
    /// `Main.PendingPlayer = new Player()` (`UICharacterSelect.cs:110`) — the
    /// field initializers of `Player.cs:826,2554-2566`, plus the ctor's
    /// `difficulty = 0` (`UICharacterCreation.cs:169`).
    pub fn new() -> Self {
        Self {
            name: String::new(),
            difficulty: 0,
            hair: 0,
            skin_variant: 0,
            hair_color: [215, 90, 55, 255],    // `Player.cs:2554`
            eye_color: [105, 90, 75, 255],     // `Player.cs:2558`
            skin_color: [255, 125, 90, 255],   // `Player.cs:2556`
            shirt_color: [175, 165, 140, 255], // `Player.cs:2560`
            undershirt_color: [160, 180, 215, 255], // `Player.cs:2562`
            pants_color: [255, 230, 175, 255], // `Player.cs:2564`
            shoe_color: [160, 105, 60, 255],   // `Player.cs:2566`
            dirty: false,
        }
    }

    /// The colour one of the seven colour categories edits — the
    /// `ApplyPendingColor` switch (`UICharacterCreation.cs:1137-1163`) read
    /// back. `id` is the `CategoryId` (3..=9; anything else is `None` —
    /// CharInfo/Clothing/HairStyle carry no picker colour).
    pub fn color_of(&self, id: usize) -> Option<[u8; 4]> {
        Some(match id {
            3 => self.hair_color,
            4 => self.eye_color,
            5 => self.skin_color,
            6 => self.shirt_color,
            7 => self.undershirt_color,
            8 => self.pants_color,
            9 => self.shoe_color,
            _ => return None,
        })
    }

    /// The [`ApplyPendingColor`](UICharacterCreation.cs) switch (`:1137-1163`).
    pub fn set_color_of(&mut self, id: usize, c: [u8; 4]) {
        match id {
            3 => self.hair_color = c,
            4 => self.eye_color = c,
            5 => self.skin_color = c,
            6 => self.shirt_color = c,
            7 => self.undershirt_color = c,
            8 => self.pants_color = c,
            9 => self.shoe_color = c,
            _ => {}
        }
    }

    /// `GetPlayerTemplateValues() != initialState` (`UICharacterCreation.cs:
    /// 2017-2019`): the serialized template covers hair, clothing style and the
    /// seven colours (voice fields are the documented no-audio deviation) —
    /// not the name and not the difficulty.
    pub fn template_dirty(&self) -> bool {
        let initial = Self::new();
        self.hair != initial.hair
            || self.skin_variant != initial.skin_variant
            || self.hair_color != initial.hair_color
            || self.eye_color != initial.eye_color
            || self.skin_color != initial.skin_color
            || self.shirt_color != initial.shirt_color
            || self.undershirt_color != initial.undershirt_color
            || self.pants_color != initial.pants_color
            || self.shoe_color != initial.shoe_color
    }

    /// The Draw-time dirty check (`:2011-2021`), run once per frame while the
    /// flag is clear — exactly like the source's `if (!dirty)` gate.
    pub fn refresh_dirty(&mut self) {
        if !self.dirty {
            // `if (!string.IsNullOrEmpty(_player.name)) dirty = true;` (`:2013`).
            self.dirty = !self.name.is_empty() || self.template_dirty();
        }
    }
}

/// The screen state the C# keeps on the `UICharacterCreation` itself, riding
/// the parked `FrontScreen` so the mode-40 No branch restores the exact screen
/// (`BackupConfirmationState`, `UICharacterCreation.cs:1215`).
#[derive(Clone, Debug, PartialEq)]
pub struct CreationState {
    /// `Main.PendingPlayer`'s port.
    pub draft: NewPlayerDraft,
    /// `_selectedPicker` (`:80`): the active category, `CategoryId` 0..=9.
    pub category: usize,
    /// `_lastSelectedHairstyle == null`'s port (`:138`): cleared until a hair
    /// button is clicked, gating `TryAutoAssigningHair`'s per-cloth-style
    /// default (`:631-637`).
    pub hair_picked: bool,
    /// `UISliderBase.CurrentLockedSlider`'s port for the three HSL bars —
    /// 0 none, 1..=3 Hue/Saturation/Luminance while the button is held.
    pub hsl_grab: u8,
}

impl CreationState {
    /// `NewCharacterClick` (`UICharacterSelect.cs:109-113`): a fresh
    /// `PendingPlayer` on the CharInfo menu (`Click_CharInfo`'s seed, `:233`).
    pub fn new() -> Self {
        Self {
            draft: NewPlayerDraft::new(),
            category: 0,
            hair_picked: false,
            hsl_grab: 0,
        }
    }
}

impl Default for CreationState {
    fn default() -> Self {
        Self::new()
    }
}

/// The seven draft colours the preview tints (`Main.PlayerRenderer.DrawPlayer`'s
/// per-part colours, `UICharacter.cs:118`; the category stack tint, `:800-808`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PreviewColors {
    pub hair: [u8; 4],
    pub eye: [u8; 4],
    pub skin: [u8; 4],
    pub shirt: [u8; 4],
    pub undershirt: [u8; 4],
    pub pants: [u8; 4],
    pub shoe: [u8; 4],
}

impl From<&NewPlayerDraft> for PreviewColors {
    fn from(d: &NewPlayerDraft) -> Self {
        Self {
            hair: d.hair_color,
            eye: d.eye_color,
            skin: d.skin_color,
            shirt: d.shirt_color,
            undershirt: d.undershirt_color,
            pants: d.pants_color,
            shoe: d.shoe_color,
        }
    }
}

// --- The static tables --------------------------------------------------------

/// `CategoryId` (`:25-38`) — the bar order and the Left offsets derive from it:
/// `Left = xPositionStart + id * xPositionPerId` with `-240/48`
/// (`CreateColorPicker`, `:824-833`).
pub const CATEGORY_IDS: [usize; 10] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9];

/// `_validClothStyles` (`:58`) — the ten cloth-style buttons in build order;
/// the first five are the male variants (`PlayerVariantID.Sets.Male =
/// CreateBoolSet(0, 1, 2, 3, 8, 10)`, `ID/PlayerVariantID.cs:20`).
pub const VALID_CLOTH_STYLES: [i32; 10] = [0, 2, 1, 3, 8, 9, 7, 5, 6, 4];

/// `_defaultHairstylesForClothStyle` (`:60-72`) — `TryAutoAssigningHair`'s map
/// (`:631-637`), keyed by cloth style.
pub fn default_hairstyle_for_cloth_style(cloth_style: i32) -> Option<i32> {
    Some(match cloth_style {
        0 => 0,
        2 => 1,
        1 => 12,
        3 => 2,
        8 => 28,
        9 => 68,
        7 => 18,
        5 => 22,
        6 => 81,
        4 => 5,
        _ => return None,
    })
}

/// `PlayerVariantID.Sets.Male` (`ID/PlayerVariantID.cs:20`) —
/// `Player.Male`'s getter (`Player.cs:3405-3409`).
pub fn is_male(skin_variant: i32) -> bool {
    matches!(skin_variant, 0 | 1 | 2 | 3 | 8 | 10)
}

/// `Main.Hairstyles.AvailableHairstyles` at character creation —
/// `HairstyleUnlocksHelper.RebuildList`'s `_isAtCharacterCreation` branch
/// (`HairstyleUnlocksHelper.cs:49-88`): 0..=50, the styled brackets, 51..=122,
/// more singles, then 166..=227 — 213 ids, so the grid rows are
/// `48 * (213/10 + 1)` (`MakeHairstylesMenu`, `UICharacterCreation.cs:287`).
pub const HAIRSTYLES: [i32; 213] = {
    let mut list = [0i32; 213];
    let mut i = 0;
    // `for (int i = 0; i < 51; i++) Add(i);`
    let mut v = 0;
    while v < 51 {
        list[i] = v;
        i += 1;
        v += 1;
    }
    // `Add(136..=144)`, `Add(147..=151)`.
    let mut v = 136;
    while v <= 144 {
        list[i] = v;
        i += 1;
        v += 1;
    }
    let mut v = 147;
    while v <= 151 {
        list[i] = v;
        i += 1;
        v += 1;
    }
    // `Add(154,155,157,158,161)`.
    let mut s = 0;
    while s < 5 {
        list[i] = EARLY_SINGLES[s];
        i += 1;
        s += 1;
    }
    // `for (int j = 51; j < 123; j++) Add(j);`
    let mut v = 51;
    while v < 123 {
        list[i] = v;
        i += 1;
        v += 1;
    }
    // `Add(134,135,146,152,153,156,159,165,160)`.
    let mut s = 0;
    while s < LATE_SINGLES.len() {
        list[i] = LATE_SINGLES[s];
        i += 1;
        s += 1;
    }
    // `for (int k = 166; k < 228; k++) Add(k);`
    let mut v = 166;
    while v < 228 {
        list[i] = v;
        i += 1;
        v += 1;
    }
    list
};

/// The RebuildList singles before the 51..=122 range (`:63-67`).
const EARLY_SINGLES: [i32; 5] = [154, 155, 157, 158, 161];
/// The RebuildList singles after it (`:82-87`).
const LATE_SINGLES: [i32; 9] = [134, 135, 146, 152, 153, 156, 159, 165, 160];

/// `MakeInfoMenu`'s four difficulty rows in build order (`:889-932`) — the
/// buttons append Journey (3), Classic (0), Mediumcore (1), Hardcore (2).
pub const DIFFICULTY_IDS: [u8; 4] = [3, 0, 1, 2];

/// The difficulty row titles (`MakeInfoMenu`, `UICharacterCreation.cs:889-910`):
/// `Lang.menu[26]` "Classic", `[25]` "Mediumcore", `[24]` "Hardcore"
/// (the `LegacyMenu` localization), and `UI.Creative` "Journey".
pub const DIFFICULTY_TITLES: [&str; 4] = ["Journey", "Classic", "Mediumcore", "Hardcore"];

/// The difficulty row colours (`:889-910`): `Color.Cyan`, `Main.mcColor`,
/// `Main.hcColor`, `Main.creativeModeColor`.
pub const DIFFICULTY_COLORS: [[u8; 4]; 4] = [
    [0, 255, 255, 255],           // Color.Cyan (`:889`)
    uistate::MC_COLOR,            // `Main.mcColor` (`:896`)
    uistate::HC_COLOR,            // `Main.hcColor` (`:903`)
    uistate::CREATIVE_MODE_COLOR, // `Main.creativeModeColor` (`:910`)
];

/// The difficulty descriptions, in `UpdateDifficultyDescription`'s
/// difficulty order 0..=3 (`:946-965`): `Lang.menu[31/30/29]` and
/// `UI.CreativeDescriptionPlayer`.
pub const DIFFICULTY_DESCRIPTIONS: [&str; 4] = [
    "Classic characters drop money on death.",
    "Mediumcore characters drop items on death.",
    "Hardcore characters die for good.",
    "Journey characters start with extra equipment. Can only be played on Journey worlds.",
];

/// The name plate's title and empty-value strings (`MakeInfoMenu`, `:858`):
/// `UI.WorldCreationName` "Name:" and `UI.PlayerEmptyName` = `{$Net.EmptyName}`
/// = "Empty name." (the JSON's resolved value), drawn Gray
/// (`UICharacterNameButton.cs`'s `SetContents`).
pub const NAME_TITLE: &str = "Name:";
pub const NAME_EMPTY: &str = "Empty name.";

// --- Layout -------------------------------------------------------------------

/// Every static rect, resolved through `uistate::resolve` exactly as the
/// source's construction order does. `name_w` is the measured "Name:" width
/// (the Mouse-font title the name plate draws at `Left 10` — the P3
/// `title_width` convention; it feeds the value-text offset in the draw).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Layout {
    /// (550, 220, 500, 384) — `BuildPage`'s root (`UICharacterCreation.cs:186-193`).
    pub root: [f32; 4],
    /// (550, 270, 500, 230) — the `UIPanel` (`:196-204`).
    pub panel: [f32; 4],
    /// (760, 200, 80, 80) — the 80x80 preview (`MakeCharPreview`, `:236-252`).
    pub preview: [f32; 4],
    /// (550, 270, 500, 50) — the top (categories) container (`:207-215`).
    pub top: [f32; 4],
    /// (550, 274, 500, 46) — its `PaddingTop 4` inner (`:213`).
    pub top_inner: [f32; 4],
    /// The ten 44x44 category buttons (`MakeCategoriesBar` + `CreateColorPicker`).
    pub categories: [[f32; 4]; 10],
    /// (560, 322, 475, 4) — the categories-bar separator (`:770-778`).
    pub separator: [f32; 4],
    /// (550, 326, 500, 160) — the middle container (`:216-225`).
    pub middle: [f32; 4],
    /// (550, 329, 500, 157) — its `PaddingTop 3` inner (`:223`; the plan's row
    /// folds the info container's own 10 px side padding into this field).
    pub middle_inner: [f32; 4],
    /// (560, 329, 480, 40) — the name plate (`MakeInfoMenu`, `:847-861`).
    pub info_name: [f32; 4],
    /// (560, 379, 188, 107) — the difficulty column (`:865-876`).
    pub info_column: [f32; 4],
    /// (756, 379, 280, 107) — the description highlight (`:877-887`).
    pub info_highlight: [f32; 4],
    /// The four 188x26.75 difficulty rows, build order Journey/Classic/
    /// Mediumcore/Hardcore (`:889-932`).
    pub difficulty_buttons: [[f32; 4]; 4],
    /// (690, 329, 220, 158) — the HSL container (`MakeHSLMenu`, `:969-977`).
    pub hsl: [f32; 4],
    /// (690, 339, 220, 104) — the slider panel (`:978-988`).
    pub hsl_panel: [f32; 4],
    /// (bar right x, bar centre y) per slider — (`:1045-1055` +
    /// `UIColoredSlider.DrawSelf`'s `drawPosition`).
    pub hsl_sliders: [[f32; 2]; 3],
    /// (810, 455, 100, 32) — the hex panel (`:992-1005`).
    pub hsl_hex_panel: [f32; 4],
    /// (690/730/770, 455, 32, 32) — Copy/Paste/Randomize (`:1006-1032`).
    pub hsl_buttons: [[f32; 4]; 3],
    /// (559, 339, 464, 1056) — the hair-grid content box (`:267-302`): the
    /// 46/48 grid of 213 buttons (`hair_button`).
    pub hair_grid: [f32; 4],
    /// (1025, 345, 20, 127) — the hair scrollbar (`:274-282`).
    pub hair_scrollbar: [f32; 4],
    /// (555, 335, 490, 157) — the hairstyles menu container (`:257-265`); the
    /// hair list clips at its bottom the way `UIList.OverflowHidden` does.
    pub hair_menu: [f32; 4],
    /// The ten 44x80 cloth-style buttons (`MakeClothStylesMenu`, `:311-335`).
    pub cloth_buttons: [[f32; 4]; 10],
    /// (566, 415, 462, 4) — the cloth-menu separator (`:441-449`).
    pub cloth_separator: [f32; 4],
    /// (826/895/860.5/929.5/964, 423, 32, 32) — the five preview-armor buttons
    /// (`:338-390`; the item icons themselves are not decoded — documented).
    pub armor_buttons: [[f32; 4]; 5],
    /// (860/894/928, 459, 32, 32) — Copy/Paste/Randomize template (`:396-430`).
    pub template_buttons: [[f32; 4]; 3],
    /// (669, 420, 52, 52) — the voice play button (`:474-504`; the prev/next
    /// `TexturePackButtons` are not decoded — documented).
    pub voice_button: [f32; 4],
    /// (550, 509, 240, 50) — Back (`MakeBackAndCreatebuttons`, `:1177-1184`).
    pub back: [f32; 4],
    /// (810, 509, 240, 50) — Create (`:1190-1197`; HAlign 1 of the 500 root).
    pub create: [f32; 4],
}

/// One 44x44 hair button in the 46/48 grid (`MakeHairstylesMenu`, `:291-302`):
/// `Left = (i % 10) * 46 + 6`, `Top = (i / 10) * 48 + 1` inside the grid box.
pub fn hair_button(grid: [f32; 4], i: usize) -> [f32; 4] {
    [
        grid[0] + (i % 10) as f32 * 46.0 + 6.0,
        grid[1] + (i / 10) as f32 * 48.0 + 1.0,
        44.0,
        44.0,
    ]
}

/// `MakeCharPreview`'s `UICharacter.GetPlayerPosition` (`UICharacter.cs:122-130`):
/// `dimensions.Position() + (Width*0.5 - width>>1, Height*0.5 - height>>1)` —
/// the fresh player's 20x42 hitbox (`width>>1 = 10`, `height>>1 = 21`). The
/// `result.X -= 10` pet branch is dead here: `_petProjectiles = UICharacter.
/// NoPets` (`UICharacter.cs:51`) and a fresh player's `miscEquips[0].IsAir`
/// keeps it that way (`:55-66`).
pub fn preview_anchor(preview: [f32; 4]) -> [f32; 2] {
    [
        preview[0] + preview[2] * 0.5 - 10.0,
        preview[1] + preview[3] * 0.5 - 21.0,
    ]
}

/// The animation row of an animated preview —
/// `(int)(Main.GlobalTimeWrappedHourly / 0.07f) % 14 + 6`
/// (`UICharacter.cs:92`); `time` is `GlobalTimeWrappedHourly` (the App's
/// `anim_time`). A static preview (the cards) passes row 0 (`:89`).
pub fn animated_frame_row(time: f32) -> u32 {
    ((time / 0.07) as i32 % 14 + 6) as u32
}

/// `layout(screen, name_w)` — the whole page through the shared pass, in the
/// source's construction order. `name_w` only feeds the name plate's value
/// offset (the draw), not a rect, so every returned rect is font-free.
pub fn layout(screen: Vec2, name_w: f32) -> Layout {
    let _ = name_w; // the plate's rect is style-driven (400x40 -> 100%); see `draw_info_menu`.
    let full = [0.0, 0.0, screen.x, screen.y];
    // The root: `Width 500, Height 380+4, Top 220, HAlign .5` (`:186-193`).
    let root = uistate::resolve(
        full,
        uistate::Style::px(0.0),
        uistate::Style::px(220.0),
        uistate::Style::px(500.0),
        uistate::Style::px(384.0),
        0.5,
        0.0,
        0.0,
        0.0,
        f32::MAX,
        [0.0; 4],
    );
    // `SetPadding(0)` (`:194`) — the root's inner is its outer.
    let root_inner = uistate::inner(root, [0.0; 4]);
    // The panel: `Width 100%, Height 384-150-4, Top 50` (`:196-204`).
    let panel = uistate::resolve(
        root_inner,
        uistate::Style::px(0.0),
        uistate::Style::px(50.0),
        uistate::Style::pct(1.0),
        uistate::Style::px(230.0),
        0.0,
        0.0,
        0.0,
        0.0,
        f32::MAX,
        [0.0; 4],
    );
    // `uIPanel.SetPadding(0)` (`:203`).
    let panel_inner = uistate::inner(panel, [0.0; 4]);
    // The preview: `80x80, Top -70, VAlign 0, HAlign .5` of the panel
    // (`MakeCharPreview`, `:236-252`).
    let preview = uistate::resolve(
        panel_inner,
        uistate::Style::px(0.0),
        uistate::Style::px(-70.0),
        uistate::Style::px(80.0),
        uistate::Style::px(80.0),
        0.5,
        0.0,
        0.0,
        0.0,
        f32::MAX,
        [0.0; 4],
    );
    // The top container: `Width 100%, Height 50`, `PaddingTop 4` (`:207-215`).
    let top = uistate::resolve(
        panel_inner,
        uistate::Style::px(0.0),
        uistate::Style::px(0.0),
        uistate::Style::pct(1.0),
        uistate::Style::px(50.0),
        0.0,
        0.0,
        0.0,
        0.0,
        f32::MAX,
        [0.0; 4],
    );
    let top_inner = uistate::inner(top, [0.0, 4.0, 0.0, 0.0]);
    // The categories: `Left (-240 + 48*id, 0.5), VAlign 0, HAlign 0` of the top
    // inner, 44x44 texture-sized (`CreateColorPicker`, `:828-833`).
    let categories = std::array::from_fn(|id| {
        uistate::resolve(
            top_inner,
            uistate::Style::new(-240.0 + 48.0 * id as f32, 0.5),
            uistate::Style::px(0.0),
            uistate::Style::px(44.0),
            uistate::Style::px(44.0),
            0.0,
            0.0,
            0.0,
            0.0,
            f32::MAX,
            [0.0; 4],
        )
    });
    // The separator: `Width (-25, 1), Top 6, Left -2.5, VAlign 1, HAlign .5`
    // (`:770-778`); the element's own height is the 6x4 sheet's (`:28`).
    let separator = uistate::resolve(
        top_inner,
        uistate::Style::px(-2.5),
        uistate::Style::px(6.0),
        uistate::Style::new(-25.0, 1.0),
        uistate::Style::px(4.0),
        0.5,
        1.0,
        0.0,
        0.0,
        f32::MAX,
        [0.0; 4],
    );
    // The middle container: `Top (50 + 6, 0), Width 100%, Height 230-70`,
    // `PaddingTop 3` (`:216-225`).
    let middle = uistate::resolve(
        panel_inner,
        uistate::Style::px(0.0),
        uistate::Style::px(56.0),
        uistate::Style::pct(1.0),
        uistate::Style::px(160.0),
        0.0,
        0.0,
        0.0,
        0.0,
        f32::MAX,
        [0.0; 4],
    );
    let middle_inner = uistate::inner(middle, [0.0, 3.0, 0.0, 0.0]);

    // --- The Info menu (`MakeInfoMenu`, `:845-944`) ---
    // The info container: `Width/Height 100%`, `HAlign .5`, `SetPadding(10)`,
    // `PaddingBottom/Top = 0` (`:847-856`).
    let info = uistate::resolve(
        middle_inner,
        uistate::Style::px(0.0),
        uistate::Style::px(0.0),
        uistate::Style::pct(1.0),
        uistate::Style::pct(1.0),
        0.5,
        0.0,
        0.0,
        0.0,
        f32::MAX,
        [10.0, 0.0, 10.0, 0.0],
    );
    let info_inner = uistate::inner(info, [10.0, 0.0, 10.0, 0.0]);
    // The name plate: `UICharacterNameButton` (400x40 style), `Width 100%`,
    // `HAlign .5` (`:858-861`).
    let info_name = uistate::resolve(
        info_inner,
        uistate::Style::px(0.0),
        uistate::Style::px(0.0),
        uistate::Style::pct(1.0),
        uistate::Style::px(40.0),
        0.5,
        0.0,
        0.0,
        0.0,
        f32::MAX,
        [0.0; 4],
    );
    // The difficulty column: `HAlign 0, VAlign 1, Width (-4, 0.4), Height
    // (-50, 1)` (`:865-876`).
    let info_column = uistate::resolve(
        info_inner,
        uistate::Style::px(0.0),
        uistate::Style::px(0.0),
        uistate::Style::new(-4.0, 0.4),
        uistate::Style::new(-50.0, 1.0),
        0.0,
        1.0,
        0.0,
        0.0,
        f32::MAX,
        [0.0; 4],
    );
    // The description highlight: `HAlign 1, VAlign 1, Width (-8, 0.6), Left -4,
    // Height = the column's style` (`:877-887`).
    let info_highlight = uistate::resolve(
        info_inner,
        uistate::Style::px(-4.0),
        uistate::Style::px(0.0),
        uistate::Style::new(-8.0, 0.6),
        uistate::Style::new(-50.0, 1.0),
        1.0,
        1.0,
        0.0,
        0.0,
        f32::MAX,
        [0.0; 4],
    );
    // The four rows: `Height (0, 1/4)` at `VAlign 0 / 1/3 / 2/3 / 1`
    // (`:889-916`, build order Journey/Classic/Mediumcore/Hardcore).
    let difficulty_buttons = std::array::from_fn(|k| {
        uistate::resolve(
            info_column,
            uistate::Style::px(0.0),
            uistate::Style::px(0.0),
            uistate::Style::pct(1.0),
            uistate::Style::pct(1.0 / 4.0),
            0.0,
            match k {
                0 => 0.0,       // Journey: `VAlign = 0f` (`:913`)
                1 => 1.0 / 3.0, // Classic: `1/(num4-1)` (`:892`)
                2 => 2.0 / 3.0, // Mediumcore: `2/(num4-1)` (`:899`)
                _ => 1.0,       // Hardcore: `1f` (`:906`)
            },
            0.0,
            0.0,
            f32::MAX,
            [0.0; 4],
        )
    });

    // --- The HSL menu (`MakeHSLMenu`, `:967-1038`) ---
    // The container: `220x158, HAlign .5, VAlign 0` (`:969-977`).
    let hsl = uistate::resolve(
        middle_inner,
        uistate::Style::px(0.0),
        uistate::Style::px(0.0),
        uistate::Style::px(220.0),
        uistate::Style::px(158.0),
        0.5,
        0.0,
        0.0,
        0.0,
        f32::MAX,
        [0.0; 4],
    );
    let hsl_inner = uistate::inner(hsl, [0.0; 4]);
    // The slider panel: `220x104, Top 10, HAlign .5`, `PaddingTop 3` (`:978-988`).
    let hsl_panel = uistate::resolve(
        hsl_inner,
        uistate::Style::px(0.0),
        uistate::Style::px(10.0),
        uistate::Style::px(220.0),
        uistate::Style::px(104.0),
        0.5,
        0.0,
        0.0,
        0.0,
        f32::MAX,
        [0.0; 4],
    );
    let hsl_panel_inner = uistate::inner(hsl_panel, [0.0, 3.0, 0.0, 0.0]);
    // The sliders: `Width (-10, 1), Top 30*id, VAlign 0, HAlign 0` (`:1045-1055`);
    // `UIColoredSlider.DrawSelf` draws the bar at `(X + W - 10, Y + 10 + 8)`
    // (`UIColoredSlider.cs:66-67`).
    let hsl_sliders = std::array::from_fn(|k| {
        let dims = uistate::resolve(
            hsl_panel_inner,
            uistate::Style::px(0.0),
            uistate::Style::px(30.0 * k as f32),
            uistate::Style::new(-10.0, 1.0),
            uistate::Style::px(25.0),
            0.0,
            0.0,
            0.0,
            0.0,
            f32::MAX,
            [0.0; 4],
        );
        [dims[0] + dims[2] - 10.0, dims[1] + 18.0]
    });
    // The hex panel: `VAlign 1, HAlign 1, 100x32` of the container (`:992-998`).
    let hsl_hex_panel = uistate::resolve(
        hsl_inner,
        uistate::Style::px(0.0),
        uistate::Style::px(0.0),
        uistate::Style::px(100.0),
        uistate::Style::px(32.0),
        1.0,
        1.0,
        0.0,
        0.0,
        f32::MAX,
        [0.0; 4],
    );
    // Copy/Paste/Randomize: `VAlign 1, HAlign 0, Left 0/40/80`, SmallPanel
    // 32x32 (`:1006-1032`).
    let hsl_buttons = std::array::from_fn(|k| {
        uistate::resolve(
            hsl_inner,
            uistate::Style::px(40.0 * k as f32),
            uistate::Style::px(0.0),
            uistate::Style::px(32.0),
            uistate::Style::px(32.0),
            0.0,
            1.0,
            0.0,
            0.0,
            f32::MAX,
            [0.0; 4],
        )
    });

    // --- The hairstyles menu (`MakeHairstylesMenu`, `:254-304`) ---
    // The menu container: `Width (-10, 1), Height 100%, HAlign .5, VAlign .5,
    // Top 6` (`:257-264`).
    let hair_menu = uistate::resolve(
        middle_inner,
        uistate::Style::px(0.0),
        uistate::Style::px(6.0),
        uistate::Style::new(-10.0, 1.0),
        uistate::Style::pct(1.0),
        0.5,
        0.5,
        0.0,
        0.0,
        f32::MAX,
        [0.0; 4],
    );
    let hair_menu_inner = uistate::inner(hair_menu, [0.0; 4]);
    // The UIList: `Width (-18, 1), Height (-6, 1)`, `SetPadding(4)` (`:267-273`).
    let hair_list = uistate::resolve(
        hair_menu_inner,
        uistate::Style::px(0.0),
        uistate::Style::px(0.0),
        uistate::Style::new(-18.0, 1.0),
        uistate::Style::new(-6.0, 1.0),
        0.0,
        0.0,
        0.0,
        0.0,
        f32::MAX,
        [4.0; 4],
    );
    // The grid content: `Width 100%`, `Height 48 * rows` of the list inner
    // (`:284-289`).
    let rows = (HAIRSTYLES.len() / 10 + usize::from(HAIRSTYLES.len() % 10 != 0)) as f32;
    let hair_grid = uistate::resolve(
        uistate::inner(hair_list, [4.0; 4]),
        uistate::Style::px(0.0),
        uistate::Style::px(0.0),
        uistate::Style::pct(1.0),
        uistate::Style::px(48.0 * rows),
        0.0,
        0.0,
        0.0,
        0.0,
        f32::MAX,
        [0.0; 4],
    );
    // The scrollbar: `HAlign 1, Height (-30, 1), Top 10` (`:274-280`).
    let hair_scrollbar = uistate::resolve(
        hair_menu_inner,
        uistate::Style::px(0.0),
        uistate::Style::px(10.0),
        uistate::Style::px(20.0),
        uistate::Style::new(-30.0, 1.0),
        1.0,
        0.0,
        0.0,
        0.0,
        f32::MAX,
        [0.0; 4],
    );

    // --- The cloth-styles menu (`MakeClothStylesMenu`, `:311-528`) ---
    // The menu container: `Width/Height 100%, HAlign .5, VAlign .5` (`:313-320`).
    let cloth_menu = uistate::resolve(
        middle_inner,
        uistate::Style::px(0.0),
        uistate::Style::px(0.0),
        uistate::Style::pct(1.0),
        uistate::Style::pct(1.0),
        0.5,
        0.5,
        0.0,
        0.0,
        f32::MAX,
        [0.0; 4],
    );
    let cloth_inner = uistate::inner(cloth_menu, [0.0; 4]);
    // The buttons: `Left i*46 + (11 if i < 5 else 29), Top 0`, 44x80
    // (`:322-335` + `UIClothStyleButton.cs:36-38`).
    let cloth_buttons = std::array::from_fn(|i| {
        let num2 = if i < VALID_CLOTH_STYLES.len() / 2 { 11.0 } else { 29.0 };
        uistate::resolve(
            cloth_inner,
            uistate::Style::px(i as f32 * 46.0 + num2),
            uistate::Style::px(0.0),
            uistate::Style::px(44.0),
            uistate::Style::px(80.0),
            0.0,
            0.0,
            0.0,
            0.0,
            f32::MAX,
            [0.0; 4],
        )
    });
    // The separator: `Width (-38, 1), HAlign .5, VAlign 1, Top -67, Left -3`
    // (`:441-449`).
    let cloth_separator = uistate::resolve(
        cloth_inner,
        uistate::Style::px(-3.0),
        uistate::Style::px(-52.0 - 15.0),
        uistate::Style::new(-38.0, 1.0),
        uistate::Style::px(4.0),
        0.5,
        1.0,
        0.0,
        0.0,
        f32::MAX,
        [0.0; 4],
    );
    // The armor row: `170x50, Left (26, 0.5), VAlign 1, Top -22` (`:338-346`).
    let armor_row = uistate::resolve(
        cloth_inner,
        uistate::Style::new(60.0 - 34.0, 0.5),
        uistate::Style::px(-22.0),
        uistate::Style::px(170.0),
        uistate::Style::px(50.0),
        0.0,
        1.0,
        0.0,
        0.0,
        f32::MAX,
        [0.0; 4],
    );
    let armor_row_inner = uistate::inner(armor_row, [0.0; 4]);
    // The five armor buttons: `VAlign .5` at `HAlign 0 / 0.5 / 0.25 / 0.75 / 1`
    // (`:348-384`, build order none/hallowed/silver/formal/swimming).
    let armor_aligns = [0.0, 0.5, 0.25, 0.75, 1.0];
    let armor_buttons = std::array::from_fn(|k| {
        uistate::resolve(
            armor_row_inner,
            uistate::Style::px(0.0),
            uistate::Style::px(0.0),
            uistate::Style::px(32.0),
            uistate::Style::px(32.0),
            armor_aligns[k],
            0.5,
            0.0,
            0.0,
            f32::MAX,
            [0.0; 4],
        )
    });
    // The template row: `100x50, Left (60, 0.5), VAlign 1, Top 14` (`:396-405`).
    let template_row = uistate::resolve(
        cloth_inner,
        uistate::Style::new(60.0, 0.5),
        uistate::Style::px(-15.0 + 38.0 - 9.0),
        uistate::Style::px(100.0),
        uistate::Style::px(50.0),
        0.0,
        1.0,
        0.0,
        0.0,
        f32::MAX,
        [0.0; 4],
    );
    let template_row_inner = uistate::inner(template_row, [0.0; 4]);
    // Copy/Paste/Randomize: `VAlign .5` at `HAlign 0 / 0.5 / 1` (`:406-430`).
    let template_buttons = std::array::from_fn(|k| {
        uistate::resolve(
            template_row_inner,
            uistate::Style::px(0.0),
            uistate::Style::px(0.0),
            uistate::Style::px(32.0),
            uistate::Style::px(32.0),
            [0.0, 0.5, 1.0][k],
            0.5,
            0.0,
            0.0,
            f32::MAX,
            [0.0; 4],
        )
    });
    // The voice row: `90x50, HAlign 1, Left (-60, -0.5), VAlign 1, Top -15`
    // (`:431-440`); the play button: 52x52, `VAlign .5, HAlign .5` (`:474-480`).
    let voice_row = uistate::resolve(
        cloth_inner,
        uistate::Style::new(-60.0, -0.5),
        uistate::Style::px(-15.0),
        uistate::Style::px(90.0),
        uistate::Style::px(50.0),
        1.0,
        1.0,
        0.0,
        0.0,
        f32::MAX,
        [0.0; 4],
    );
    let voice_button = uistate::resolve(
        uistate::inner(voice_row, [0.0; 4]),
        uistate::Style::px(0.0),
        uistate::Style::px(0.0),
        uistate::Style::px(52.0),
        uistate::Style::px(52.0),
        0.5,
        0.5,
        0.0,
        0.0,
        f32::MAX,
        [0.0; 4],
    );

    // --- Back / Create (`MakeBackAndCreatebuttons`, `:1177-1203`) ---
    // `Width (-10, 0.5), Height 50, VAlign 1, Top -45`, `HAlign 0/1` of the root.
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
        preview,
        top,
        top_inner,
        categories,
        separator,
        middle,
        middle_inner,
        info_name,
        info_column,
        info_highlight,
        difficulty_buttons,
        hsl,
        hsl_panel,
        hsl_sliders,
        hsl_hex_panel,
        hsl_buttons,
        hair_grid,
        hair_scrollbar,
        hair_menu,
        cloth_buttons,
        cloth_separator,
        armor_buttons,
        template_buttons,
        voice_button,
        back,
        create,
    }
}

/// The active middle menu per category — the `Click_*` re-append
/// (`:1250-1275`) and `SelectColorPicker`'s fall-through (`:1810-1833`):
/// CharInfo -> Info, Clothing -> ClothStyles, HairStyle -> Hairstyles,
/// HairColor..Shoes -> the HSL menu over that colour.
pub fn middle_for(category: usize) -> MiddleMenu {
    match category {
        0 => MiddleMenu::Info,
        1 => MiddleMenu::ClothStyles,
        2 => MiddleMenu::Hairstyles,
        3..=9 => MiddleMenu::Hsl { color_id: category },
        _ => MiddleMenu::Info,
    }
}

/// The middle menus (`_middleContainer`'s four children, `:228-231`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MiddleMenu {
    Info,
    Hsl { color_id: usize },
    Hairstyles,
    ClothStyles,
}

/// The picker texture per category (`CreateColorPicker`'s texture paths,
/// `:735-743`, plus `:765`'s CharInfo): the index into `Lookups::cc_colors`
/// (0..=7 — Hair, Eye, EyeBack, Skin, Shirt, Undershirt, Pants, Shoes,
/// `assets.rs:988-997`) — `HairStyle`/`CharInfo` use their own Lookups fields.
pub fn category_texture(id: usize) -> Option<usize> {
    match id {
        3 => Some(0), // ColorHair (`:735`)
        4 => Some(1), // ColorEye (`:736`; the `ColorEyeBack` middle is 2)
        5 => Some(3), // ColorSkin (`:737`)
        6 => Some(4), // ColorShirt (`:738`)
        7 => Some(5), // ColorUndershirt (`:739`)
        8 => Some(6), // ColorPants (`:740`)
        9 => Some(7), // ColorShoes (`:741`)
        _ => None,
    }
}

// --- HSL maths (the scaled triple) ---------------------------------------------

/// `UICharacterCreation.ScaledHslToRgb` (`:2354-2357`):
/// `Main.hslToRgb(hue, saturation, luminosity * 0.85f + 0.15f)`.
pub fn hsl_to_rgb_scaled(hsl: [f32; 3]) -> [u8; 4] {
    color::hsl_to_rgb(hsl[0] as f64, hsl[1] as f64, (hsl[2] * 0.85 + 0.15) as f64, 255)
}

/// `UICharacterCreation.RgbToScaledHsl` (`:2359-2365`):
/// `Main.rgbToHsl(color)` with `Z = (Z - 0.15f) / 0.85f`, then
/// `Vector3.Clamp(value, Zero, One)`.
pub fn rgb_to_hsl_scaled(rgb: [u8; 4]) -> [f32; 3] {
    let (h, s, l) = color::rgb_to_hsl(rgb);
    let clamp = |v: f32| v.clamp(0.0, 1.0); // `Vector3.Clamp` (`:2364`)
    [
        clamp(h as f32),
        clamp(s as f32),
        clamp((l as f32 - 0.15) / 0.85),
    ]
}

/// `GetHexText` (`:1170-1173`): `"#" + pendingColor.Hex3().ToUpper()` — the
/// three colour bytes as two uppercase hex digits each.
pub fn hex_text(rgb: [u8; 4]) -> String {
    format!("#{:02X}{:02X}{:02X}", rgb[0], rgb[1], rgb[2])
}

/// `GetHSLSliderColorAt` (`:1126-1135`): the per-bar blip colours. `hsl` is the
/// live `_currentColorHSL`; `point_at` is the blip's `i/167` position.
pub fn slider_blip_color(id: usize, point_at: f32, hsl: [f32; 3]) -> [u8; 4] {
    match id {
        0 => hsl_to_rgb_scaled([point_at, 1.0, 0.5]),       // `:1130`
        1 => hsl_to_rgb_scaled([hsl[0], point_at, hsl[2]]), // `:1131`
        _ => hsl_to_rgb_scaled([hsl[0], hsl[1], point_at]), // `:1132`
    }
}

/// XNA `Color.Lerp(a, b, t)` in vector space, byte-truncated on the way out —
/// `Color.Lerp` lerps the `Vector4`s and `Color(Vector4)` casts `(byte)(x *
/// 255f)`. Alpha rides along like every channel.
pub fn lerp_rgb(a: [u8; 4], b: [u8; 4], t: f32) -> [u8; 4] {
    let f = |x: u8, y: u8| {
        let av = x as f32 / 255.0;
        let bv = y as f32 / 255.0;
        ((av + (bv - av) * t) * 255.0) as i64 as u8
    };
    [f(a[0], b[0]), f(a[1], b[1]), f(a[2], b[2]), f(a[3], b[3])]
}

/// The slider chrome at `(right_x, centre_y)` — `UIColoredSlider.DrawSelf`
/// (`UIColoredSlider.cs:66-67`) + `DrawValueBar` (`:83-95`): the bar rect
/// (178x16, right edge at `right_x`) and the in-bar hit rectangle
/// `(X + 5 - 2, Y + 4, W - 4, H - 8)` (`:105-108`).
pub fn hsl_slider_rects(right_x: f32, centre_y: f32) -> ([f32; 4], [f32; 4]) {
    let bar = [
        right_x - sliders::BAR_W,
        centre_y - sliders::BAR_H / 2.0,
        sliders::BAR_W,
        sliders::BAR_H,
    ];
    let hit = [bar[0] + 3.0, bar[1] + 4.0, sliders::BAR_W - 4.0, sliders::BAR_H - 8.0];
    (bar, hit)
}

/// The knob centre (`UIColoredSlider.cs:125-127`): `(num + 167*scale*value,
/// num2 + 4*scale)` with the sheet's float-half centre origin — at scale 1,
/// `(bar.x + 5 + 167*value, bar.y + 8)`.
pub fn hsl_slider_knob(bar: [f32; 4], value: f32) -> [f32; 2] {
    [bar[0] + 5.0 + sliders::BLIPS as f32 * value, bar[1] + 8.0]
}

/// The read a drag produces (`UIColoredSlider.cs:130-133`):
/// `(mouseX - rectangle.X) / rectangle.Width` inside the hit rectangle, 0 left
/// of it, 1 right of it.
fn hsl_slider_value(hit: [f32; 4], mouse_x: f32) -> f32 {
    if mouse_x >= hit[0] && mouse_x <= hit[0] + hit[2] {
        return (mouse_x - hit[0]) / hit[2];
    }
    if hit[0] >= mouse_x {
        return 0.0;
    }
    1.0
}

/// The draw-time drag half of the HSL menu — `UIColoredSlider.DrawSelf`'s
/// `CurrentAimedSlider`/`CurrentLockedSlider` latch (`UIColoredSlider.cs:50-90`)
/// driving `UpdateHSLValue` (`:1102-1124`): while the button is held a row
/// latches on hover (`hsl_grab` 1..=3) and re-writes its HSL component every
/// frame, the conversion landing back in the draft's colour through
/// `ScaledHslToRgb`/`ApplyPendingColor`; releasing clears the latch. Returns
/// the three hover flags (the `ColorHighlight` overlay).
pub fn hsl_sliders_update(
    l: &Layout,
    state: &mut CreationState,
    mouse: Vec2,
    mouse_down: bool,
) -> [bool; 3] {
    let mut highlight = [false; 3];
    if state.category < 3 {
        // The HSL menu is only live over a colour category (`:1826`).
        state.hsl_grab = 0;
        return highlight;
    }
    let color_id = state.category;
    if !mouse_down {
        // `if (!Main.mouseLeft) UISliderBase.CurrentLockedSlider = null;`
        // (`UIColoredSlider.cs:50-52`).
        state.hsl_grab = 0;
    }
    for k in 0..3 {
        let [right_x, centre_y] = l.hsl_sliders[k];
        let (_, hit) = hsl_slider_rects(right_x, centre_y);
        let in_bar = in_rect(mouse, hit);
        // `(CurrentLockedSlider == this) | wasInBar` aims this row; the lock
        // latches only from an unlocked state (`:87-90`).
        let active = (in_bar && state.hsl_grab == 0) || state.hsl_grab == (k + 1) as u8;
        highlight[k] = active;
        if active && mouse_down {
            if in_bar {
                state.hsl_grab = (k + 1) as u8;
            }
            if state.hsl_grab == (k + 1) as u8 {
                // `UpdateHSLValue(id, obj)` (`:1102-1124`): the component
                // writes, the colour converts, `ApplyPendingColor` lands it.
                let mut hsl =
                    rgb_to_hsl_scaled(state.draft.color_of(color_id).unwrap_or([255; 4]));
                hsl[k] = hsl_slider_value(hit, mouse.x);
                state.draft.set_color_of(color_id, hsl_to_rgb_scaled(hsl));
            }
        }
    }
    highlight
}

// --- Clicks ---------------------------------------------------------------------

/// `UICharacterCreation.MAX_NAME_LENGTH` (`:142`) — the naming keyboard's cap
/// (`UIVirtualKeyboard.cs:159`'s default `maxLength = 20`).
pub const MAX_NAME_LENGTH: usize = 20;

/// What a click does — the `OnLeftMouseDown` subscriptions of `:735-768`,
/// `:832`, `:863`, `:889-938`, `:1187`, `:1200`, `:295-301`, `:332`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Click {
    /// A category button (`CreateColorPicker`/`CreatePickerWithoutClick` +
    /// the three `Click_*` re-appends).
    Category(usize),
    /// A difficulty row (`UIDifficultyButton.LeftMouseDown`,
    /// `UIDifficultyButton.cs:67-71`).
    Difficulty(u8),
    /// A hair button (`UIHairStyleButton.LeftMouseDown`,
    /// `UIHairStyleButton.cs:66-70`).
    Hair(i32),
    /// A cloth-style button (`Click_CharClothStyle` -> `skinVariant`,
    /// `:1277-1291`).
    ClothStyle(i32),
    /// The name plate (`Click_Naming`, `:1659-1666`).
    Name,
    /// The Create panel (`Click_NamingAndCreating`, `:1668-1682`).
    Create,
    /// The Back panel (`Click_GoBack` -> `GoBack`, `:1205-1222`).
    Back,
    None,
}

/// The click table over the live geometry: the categories bar (always), the
/// Back/Create pair (always), and the active middle menu's own controls —
/// the name plate + difficulty column (Info), the hair grid (Hairstyles), the
/// cloth-style buttons (ClothStyles). The HSL rows drag through
/// [`hsl_sliders_update`] (the draw pass), and the small Copy/Paste/Randomize
/// buttons click nowhere in the port (no clipboard pipeline — documented).
pub fn click_at(l: &Layout, category: usize, mouse: Vec2) -> Click {
    // The categories bar (`MakeCategoriesBar`, `:730-768`).
    for id in CATEGORY_IDS {
        if in_rect(mouse, l.categories[id]) {
            return Click::Category(id);
        }
    }
    // The always-present pair (`:1177-1203`).
    if in_rect(mouse, l.back) {
        return Click::Back;
    }
    if in_rect(mouse, l.create) {
        return Click::Create;
    }
    match middle_for(category) {
        MiddleMenu::Info => {
            if in_rect(mouse, l.info_name) {
                return Click::Name; // `Click_Naming` (`:863`)
            }
            for (k, id) in DIFFICULTY_IDS.iter().enumerate() {
                if in_rect(mouse, l.difficulty_buttons[k]) {
                    return Click::Difficulty(*id); // `UIDifficultyButton.cs:67-71`
                }
            }
        }
        MiddleMenu::Hairstyles => {
            for (i, hair) in HAIRSTYLES.iter().enumerate() {
                if in_rect(mouse, hair_button(l.hair_grid, i)) {
                    return Click::Hair(*hair); // `UIHairStyleButton.cs:66-70`
                }
            }
        }
        MiddleMenu::ClothStyles => {
            for (i, style) in VALID_CLOTH_STYLES.iter().enumerate() {
                if in_rect(mouse, l.cloth_buttons[i]) {
                    return Click::ClothStyle(*style); // `:1284-1287`
                }
            }
        }
        // The HSL menu's rows drag in the draw pass; the hex buttons and the
        // slider *element* rects click nothing (the C# slider's click is its
        // drag latch; `Click_ColorPicker` is the category's own handler).
        MiddleMenu::Hsl { .. } => {}
    }
    Click::None
}

// --- Tooltips ---------------------------------------------------------------------

/// The hovered element, as `UICharacterCreation.Draw`'s tooltip chain reads
/// them (`:1876-1979`). The voice entries exist so the chain mirrors the
/// source; the port draws their chrome without audio or clipboard.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum HoverTarget {
    Category(usize),
    CopyHex,
    PasteHex,
    RandomizeColor,
    CopyTemplate,
    PasteTemplate,
    RandomizePlayer,
    /// The pitch slider's `UISliderBase.CurrentAimedSlider` read (`:1924-1927`)
    /// — the pitch drag chrome is the documented no-audio deviation.
    #[allow(dead_code)]
    VoicePitch,
    /// The prev/next `TexturePackButtons` are not decoded (documented).
    #[allow(dead_code)]
    VoicePrev,
    #[allow(dead_code)]
    VoiceNext,
    VoicePlay,
}

/// `UICharacterCreation.Draw`'s tooltip strings (`:1880-1979`), from the
/// `en-US` localization (`UI.CopyColorToClipboard` etc.).
pub fn tooltip_for(hovered: HoverTarget) -> Option<&'static str> {
    Some(match hovered {
        HoverTarget::CopyHex => "Copy color code to clipboard",           // `:1880-1883`
        HoverTarget::PasteHex => "Paste color code from clipboard",       // `:1884-1887`
        HoverTarget::RandomizeColor => "Randomize color",                 // `:1888-1891`
        HoverTarget::CopyTemplate => "Copy player template to clipboard", // `:1892-1895`
        HoverTarget::PasteTemplate => "Paste player template from clipboard", // `:1896-1899`
        HoverTarget::RandomizePlayer => "Randomize player",               // `:1900-1903`
        HoverTarget::VoicePitch => "Change voice pitch",                  // `:1924-1927`
        HoverTarget::VoicePrev => "Select previous voice type",           // `:1928-1932`
        HoverTarget::VoiceNext => "Select next voice type",               // `:1933-1935`
        HoverTarget::VoicePlay => "Play selected voice",                  // `:1936-1939`
        // The categories (`:1940-1979`): `UI.PlayerCreateCategory*`.
        HoverTarget::Category(0) => "Info",
        HoverTarget::Category(1) => "Style",
        HoverTarget::Category(2) => "Hairstyle",
        HoverTarget::Category(3) => "Hair",
        HoverTarget::Category(4) => "Eyes",
        HoverTarget::Category(5) => "Skin ", // the JSON carries a trailing space
        HoverTarget::Category(6) => "Shirt ",
        HoverTarget::Category(7) => "Undershirt",
        HoverTarget::Category(8) => "Pants",
        HoverTarget::Category(9) => "Shoes",
        HoverTarget::Category(_) => return None,
    })
}

// --- Draw --------------------------------------------------------------------------

/// The Mouse-font width of `text` at scale 1 — the `UIText` metric the name
/// plate and hex readouts centre with.
fn mouse_text_width(r: &SpriteRenderer, lookups: &assets::Lookups, text: &str) -> f32 {
    hud_text_width(r, lookups, text, 1.0)
}

/// The `Utils.DrawSplicedPanel` base the creation buttons share
/// (`UIClothStyleButton.cs:55-57`, `UIDifficultyButton.cs:56`,
/// `UICharacterNameButton.cs:52`): 10/10/10/10 slices of the sheet.
fn draw_spliced_base(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    sheet: Option<tr_render::SheetRect>,
    rect: [f32; 4],
    tint: [f32; 4],
) {
    uistate::draw_spliced_panel(r, lookups, sheet, rect, 10.0, 10.0, 10.0, 10.0, tint);
}

/// `UIHairStyleButton.DrawSelf` (`UIHairStyleButton.cs:32-63`) minus the
/// `DrawPlayerHead` half: the `CategoryPanel` at the rect, tinted
/// `Color * (hover ? 1 : 0.4)` (`UIImageButton.cs:51`'s visibility pair), the
/// selected border centred when the hair matches, the hovered border over it.
fn draw_hair_button(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    rect: [f32; 4],
    selected: bool,
    hovered: bool,
) {
    let base = if hovered { 1.0 } else { 0.4 };
    if let Some(sheet) = lookups.cc_panel {
        push_whole(r, lookups, &sheet, rect, [base, base, base, base]);
    }
    // The borders draw centred on the button's centre at the sheet's own size
    // (`:47-57`).
    let cx = rect[0] + rect[2] * 0.5;
    let cy = rect[1] + rect[3] * 0.5;
    let mut border = |sheet: Option<tr_render::SheetRect>| {
        if let Some(sheet) = sheet {
            let w = sheet.width as f32;
            let h = sheet.height as f32;
            let uv = lookups.hud_uv(Some(&sheet)).unwrap_or([0.0; 4]);
            r.push_ui_sprite([cx - w / 2.0, cy - h / 2.0], [w, h], uv, [1.0; 4]);
        }
    };
    if selected {
        border(lookups.cc_panel_highlight); // `:47-50`
    }
    if hovered {
        border(lookups.cc_panel_border); // `:52-55`
    }
    // `Main.PlayerRenderer.DrawPlayerHead` (`:59-61`): the per-style head
    // previews need the 213 `Player_Hair_*` sheets; the port decodes only
    // `Player_Hair_1`, so the buttons draw the panel chrome and borders only
    // (documented deviation — 213 copies of one style would misrepresent every
    // other cell).
}

/// The whole screen in the source's z-order (`UIElement.Draw` draws self,
/// then the children in append order): the panel, the preview, the top
/// container (categories + separator), then the one active middle menu, then
/// Back/Create, then the hover tooltip. `BuildPage` appends Back/Create
/// before the preview (`:204-206`), not after the middle menus (`:220-233`)
/// — as later siblings of the panel the pair still draws after its subtree,
/// so the port draws them last as well; the buttons overlap nothing, which
/// makes the timing unobservable. `time` is `Main.GlobalTimeWrappedHourly`
/// (the preview's animation clock, `:92`); `mouse_down` feeds the HSL drag
/// latch. Mutates the draft like the source's draw-side paths do
/// (`:1102-1124`, `:2011-2021`).
pub fn draw(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    player_assets: &crate::render::entities::PlayerAssets,
    state: &mut CreationState,
    screen: Vec2,
    time: f32,
    mouse: Vec2,
    mouse_down: bool,
) {
    let l = layout(screen, 0.0);

    // The Draw-time dirty check (`:2011-2021`) runs before Back's consult.
    state.draft.refresh_dirty();
    // The HSL drag pass runs in the draw like the source's `DrawSelf` chain
    // (`UIColoredSlider.cs:47-90`): hit-test -> mutate -> render.
    let _highlight = hsl_sliders_update(&l, state, mouse, mouse_down);

    // The creation panel: bg `(33, 43, 79) * 0.8` (`:201`), border the
    // `UIPanel` default black (`UIPanel.cs:18`).
    let panel_bg = color::mul([33, 43, 79, 255], 0.8);
    panel::draw_panel(r, lookups, lookups.ui_panel_bg, l.panel, color::tint(panel_bg));
    panel::draw_panel(
        r,
        lookups,
        lookups.ui_panel_border,
        l.panel,
        color::tint(uistate::TEXT_PANEL_BORDER),
    );

    // The preview (`MakeCharPreview`, `:236-252`): 80x80, Top -70, scale 1.5,
    // animated, `hasBackPanel: false` — no `PlayerBackground` (`:100-103`'s
    // guard). `PreparePreview_Main` faces right and auto-assigns the hair
    // (`:615-621`).
    let anchor = preview_anchor(l.preview);
    let colors = PreviewColors::from(&state.draft);
    crate::render::entities::draw_player_preview(
        r,
        player_assets,
        anchor,
        1.5,
        animated_frame_row(time),
        &colors,
    );

    // The categories bar (`MakeCategoriesBar`, `:730-768`), in append order —
    // the seven pickers, then Clothing, HairStyle, CharInfo.
    for id in CATEGORY_IDS {
        let rect = l.categories[id];
        let hovered = in_rect(mouse, rect);
        let selected = state.category == id;
        let (icon, icon_color, middle) = match id {
            // The seven colour pickers: the picker's own colour (`:735-741`),
            // the Eye button's `ColorEyeBack` middle (`:742`) — `cc_colors[2]`.
            3..=9 => (
                category_texture(id).and_then(|k| lookups.cc_colors[k]),
                state.draft.color_of(id).unwrap_or([255; 4]),
                if id == 4 { lookups.cc_colors[2] } else { None },
            ),
            // Clothing: the male/female icon at `Color.Transparent` (`:743`,
            // `:748`) — the `ColorCharacter` stack draws instead (`:747-759`).
            1 => (
                if is_male(state.draft.skin_variant) {
                    lookups.cc_cloth_styles[0]
                } else {
                    lookups.cc_cloth_styles[1]
                },
                [0, 0, 0, 0],
                None,
            ),
            // HairStyle: the `HairStyle_Hair` icon in the live hair colour
            // (`:760-764`, `:1867`) with the `HairStyle_Arrow` middle (`:762`).
            2 => (
                lookups.cc_hair_style,
                state.draft.hair_color,
                lookups.cc_hair_arrow,
            ),
            // CharInfo (`:765-768`).
            _ => (lookups.cc_char_info, [255; 4], None),
        };
        uistate::draw_colored_image_button(
            r,
            lookups,
            false,
            rect,
            middle,
            icon,
            icon_color,
            selected,
            hovered,
        );
        // The Clothing button's 7-frame `ColorCharacter` stack, tinted by the
        // draft colours (`:747-759`, `:800-808`) — 26x34 frames of a 26x238
        // sheet (`Terraria.IO.Data.ResourcePacksDefaultInfo.tsv`).
        if id == 1 {
            if let Some(sheet) = lookups.cc_color_character {
                let frame_h = sheet.height as f32 / 7.0;
                let (aw, ah) = (lookups.atlas_size.0 as f32, lookups.atlas_size.1 as f32);
                let stack = [
                    state.draft.hair_color,
                    state.draft.eye_color,
                    state.draft.skin_color,
                    state.draft.shirt_color,
                    state.draft.undershirt_color,
                    state.draft.pants_color,
                    state.draft.shoe_color,
                ];
                let cx = rect[0] + rect[2] * 0.5;
                let cy = rect[1] + rect[3] * 0.5;
                for (i, c) in stack.iter().enumerate() {
                    let y0 = sheet.y as f32 + frame_h * i as f32;
                    let uv = [
                        sheet.x as f32 / aw,
                        y0 / ah,
                        (sheet.x as f32 + sheet.width as f32) / aw,
                        (y0 + frame_h) / ah,
                    ];
                    r.push_ui_sprite(
                        [cx - sheet.width as f32 / 2.0, cy - frame_h / 2.0],
                        [sheet.width as f32, frame_h],
                        uv,
                        color::tint(*c),
                    );
                }
            }
        }
    }

    // The separator (`:770-778`).
    panel::draw_horizontal_separator(
        r,
        lookups,
        l.separator[0],
        l.separator[1],
        l.separator[2],
        color::tint(uistate::CREATION_SEPARATOR_COLOR),
    );

    // The one active middle menu (`Click_CharInfo`'s seed, `:233`).
    match middle_for(state.category) {
        MiddleMenu::Info => draw_info_menu(r, lookups, state, &l, mouse),
        MiddleMenu::Hsl { color_id } => draw_hsl_menu(r, lookups, state, &l, color_id, mouse),
        MiddleMenu::Hairstyles => draw_hairstyles_menu(r, lookups, state, &l, mouse),
        MiddleMenu::ClothStyles => draw_cloth_styles_menu(r, lookups, state, &l, mouse),
    }

    // Back / Create (`:1177-1203`): the hover pair swaps inside
    // `uistate::draw_text_panel` (`FadedMouseOver`, `:1224-1229`).
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

    // The hover tooltip (`:1980-1993`): the hovered element's string at the
    // cursor + (16, 16), clamped to the screen the way `:1984-1991` does.
    if let Some(text) = hover_tooltip(&l, state.category, mouse) {
        let w = hud_text_width(r, lookups, text, 1.0);
        let mut x = mouse.x + 16.0;
        let mut y = mouse.y + 16.0;
        if y > screen.y - 30.0 {
            y = screen.y - 30.0;
        }
        if x > screen.x - w {
            x = screen.x - 460.0;
        }
        border_string(r, lookups, x, y, [1.0; 4], text);
    }
}

/// `UICharacterCreation.Draw`'s tooltip resolution (`:1880-1979`) over the
/// port's live elements.
fn hover_tooltip(l: &Layout, category: usize, mouse: Vec2) -> Option<&'static str> {
    for id in CATEGORY_IDS {
        if in_rect(mouse, l.categories[id]) {
            return tooltip_for(HoverTarget::Category(id));
        }
    }
    match middle_for(category) {
        MiddleMenu::Hsl { .. } => {
            for (k, target) in [
                HoverTarget::CopyHex,
                HoverTarget::PasteHex,
                HoverTarget::RandomizeColor,
            ]
            .into_iter()
            .enumerate()
            {
                if in_rect(mouse, l.hsl_buttons[k]) {
                    return tooltip_for(target);
                }
            }
        }
        MiddleMenu::ClothStyles => {
            for (k, target) in [
                HoverTarget::CopyTemplate,
                HoverTarget::PasteTemplate,
                HoverTarget::RandomizePlayer,
            ]
            .into_iter()
            .enumerate()
            {
                if in_rect(mouse, l.template_buttons[k]) {
                    return tooltip_for(target);
                }
            }
            if in_rect(mouse, l.voice_button) {
                return tooltip_for(HoverTarget::VoicePlay); // `:1936-1939`
            }
        }
        _ => {}
    }
    None
}

/// `MakeInfoMenu` (`:845-944`): the name plate, the difficulty column's four
/// spliced fills and the description highlight.
fn draw_info_menu(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    state: &CreationState,
    l: &Layout,
    mouse: Vec2,
) {
    // The name plate (`UICharacterNameButton.DrawSelf`, `UICharacterNameButton.
    // cs:49-67`): the spliced `CategoryPanel` at `White * 0.5`, the hovered
    // border, the "Name:" title at `Left 10` and the value at
    // `titleWidth + 20` (`DistanceFromTitleToOption`, `:39`).
    let name_hover = in_rect(mouse, l.info_name);
    draw_spliced_base(r, lookups, lookups.cc_panel, l.info_name, [0.5; 4]);
    if name_hover {
        draw_spliced_base(r, lookups, lookups.cc_panel_border, l.info_name, [1.0; 4]);
    }
    let title_w = mouse_text_width(r, lookups, NAME_TITLE);
    let cy = l.info_name[1] + l.info_name[3] * 0.5;
    border_string(r, lookups, l.info_name[0] + 10.0, cy - 8.0, [1.0; 4], NAME_TITLE);
    let (value, tint) = if state.draft.name.is_empty() {
        (NAME_EMPTY, color::tint([128, 128, 128, 255])) // Gray (`SetContents`)
    } else {
        (state.draft.name.as_str(), [1.0; 4])
    };
    border_string(
        r,
        lookups,
        l.info_name[0] + 10.0 + title_w + 20.0,
        cy - 8.0,
        tint,
        value,
    );

    // The four difficulty fills (`UIDifficultyButton.DrawSelf`,
    // `UIDifficultyButton.cs:47-66`): `Lerp(Black, color, 0.8) * 0.5`, the
    // selected rim at the inset `num` (7, or 5 under 30 px — the rows are
    // 26.75 tall, so 5), the hovered `CategoryPanelBorder`.
    for (k, id) in DIFFICULTY_IDS.iter().enumerate() {
        let rect = l.difficulty_buttons[k];
        let color = DIFFICULTY_COLORS[k];
        let selected = state.draft.difficulty == *id;
        let hovered = in_rect(mouse, rect);
        let fill = color::mul(lerp_rgb([0, 0, 0, 255], color, 0.8), 0.5);
        draw_spliced_base(r, lookups, lookups.cc_panel_gray, rect, color::tint(fill));
        if selected {
            let num = if rect[3] < 30.0 { 5.0 } else { 7.0 };
            let rim = color::mul(lerp_rgb(color, [255, 255, 255, 255], 0.7), 0.5);
            let inset = [
                rect[0] + num,
                rect[1] + num - 2.0,
                rect[2] - num * 2.0,
                rect[3] - num * 2.0,
            ];
            draw_spliced_base(r, lookups, lookups.cc_panel_gray, inset, color::tint(rim));
        }
        if hovered {
            draw_spliced_base(r, lookups, lookups.cc_panel_border, rect, [1.0; 4]);
        }
        // The row's title `UIText` (`UIDifficultyButton.cs:38-44`): 0.9 scale,
        // `Top 5`, centred.
        let title = DIFFICULTY_TITLES[k];
        let w = hud_text_width(r, lookups, title, 0.9);
        hud_text(
            r,
            lookups,
            rect[0] + (rect[2] - w) * 0.5,
            rect[1] + 5.0,
            0.9,
            [1.0; 4],
            title,
        );
    }

    // The description highlight (`:877-887`): the `CategoryPanelHighlight`
    // spliced 10s at `LightGray * 0.7`, with the live difficulty's description
    // (`UpdateDifficultyDescription`, `:946-965`) inside the 20 px padding
    // (`:926-927`).
    draw_spliced_base(
        r,
        lookups,
        lookups.cc_panel_highlight,
        l.info_highlight,
        color::tint(color::mul([211, 211, 211, 255], 0.7)),
    );
    let desc = DIFFICULTY_DESCRIPTIONS[(state.draft.difficulty % 4) as usize];
    hud_text(
        r,
        lookups,
        l.info_highlight[0] + 20.0,
        l.info_highlight[1] + (l.info_highlight[3] - 14.0) * 0.5,
        1.0,
        [1.0; 4],
        desc,
    );
}

/// `MakeHSLMenu` (`:967-1038`) over `color_id`: the three bars
/// (`UIColoredSlider.DrawValueBar`, `UIColoredSlider.cs:83-133`), the hex
/// panel and the Copy/Paste/Randomize buttons.
fn draw_hsl_menu(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    state: &CreationState,
    l: &Layout,
    color_id: usize,
    mouse: Vec2,
) {
    // The slider panel's chrome (`UIPanel` defaults, `:978`).
    panel::draw_panel(
        r,
        lookups,
        lookups.ui_panel_bg,
        l.hsl_panel,
        color::tint(uistate::TEXT_PANEL_BG),
    );
    panel::draw_panel(
        r,
        lookups,
        lookups.ui_panel_border,
        l.hsl_panel,
        color::tint(uistate::TEXT_PANEL_BORDER),
    );

    // The live `_currentColorHSL` — re-derived from the draft colour the way
    // `SelectColorPicker` seeds it (`:1834-1859`).
    let hsl = rgb_to_hsl_scaled(state.draft.color_of(color_id).unwrap_or([255; 4]));

    for k in 0..3 {
        let [right_x, centre_y] = l.hsl_sliders[k];
        let (bar, _) = hsl_slider_rects(right_x, centre_y);
        // The bar + 167 blips (`UIColoredSlider.cs:96-103`), coloured by
        // `GetHSLSliderColorAt`.
        if let Some(uv) = lookups.hud_uv(lookups.ui_color_bar.as_ref()) {
            r.push_ui_sprite([bar[0], bar[1]], [bar[2], bar[3]], uv, [1.0; 4]);
        }
        if let Some(uv) = lookups.hud_uv(lookups.ui_color_blip.as_ref()) {
            for i in 0..sliders::BLIPS {
                let t = i as f32 / sliders::BLIPS as f32;
                let c = slider_blip_color(k, t, hsl);
                r.push_ui_sprite(
                    [bar[0] + 5.0 + i as f32, bar[1] + 4.0],
                    [sliders::BLIP_W, sliders::BLIP_H],
                    uv,
                    color::tint(c),
                );
            }
        }
        // The `ColorHighlight` overlay while the row aims (`:113-121`).
        let (_, hit) = hsl_slider_rects(right_x, centre_y);
        if in_rect(mouse, hit) || state.hsl_grab == (k + 1) as u8 {
            if let Some(uv) = lookups.hud_uv(lookups.ui_slider_highlight.as_ref()) {
                r.push_ui_sprite(
                    [bar[0], bar[1]],
                    [bar[2], bar[3]],
                    uv,
                    color::tint(uistate::OUR_FAVORITE_COLOR),
                );
            }
        }
        // The knob, float-half centred (`:125-127`).
        if let Some(uv) = lookups.hud_uv(lookups.ui_color_slider.as_ref()) {
            let centre = hsl_slider_knob(bar, hsl[k]);
            r.push_ui_sprite(
                [
                    centre[0] - sliders::KNOB_W / 2.0,
                    centre[1] - sliders::KNOB_H / 2.0,
                ],
                [sliders::KNOB_W, sliders::KNOB_H],
                uv,
                [1.0; 4],
            );
        }
    }

    // The hex panel (`:992-1005`): the `UIPanel` + the live hex readout
    // (`UpdateHexText`, `:1165-1168`).
    panel::draw_panel(
        r,
        lookups,
        lookups.ui_panel_bg,
        l.hsl_hex_panel,
        color::tint(uistate::TEXT_PANEL_BG),
    );
    panel::draw_panel(
        r,
        lookups,
        lookups.ui_panel_border,
        l.hsl_hex_panel,
        color::tint(uistate::TEXT_PANEL_BORDER),
    );
    let hex = hex_text(state.draft.color_of(color_id).unwrap_or([255; 4]));
    let w = mouse_text_width(r, lookups, &hex);
    hud_text(
        r,
        lookups,
        l.hsl_hex_panel[0] + (l.hsl_hex_panel[2] - w) * 0.5,
        l.hsl_hex_panel[1] + (l.hsl_hex_panel[3] - 14.0) * 0.5,
        1.0,
        [1.0; 4],
        &hex,
    );

    // Copy/Paste/Randomize (`:1006-1032`): SmallPanel-backed buttons. Their
    // clicks are dead in the port (no clipboard pipeline — documented).
    for k in 0..3 {
        let sheet = lookups.cc_copy_paste_random[k];
        uistate::draw_colored_image_button(
            r,
            lookups,
            true,
            l.hsl_buttons[k],
            None,
            sheet,
            [255; 4],
            false,
            in_rect(mouse, l.hsl_buttons[k]),
        );
    }
}

/// `MakeHairstylesMenu` (`:254-304`): the 46/48 grid of hair buttons and the
/// scrollbar. Rows past the container bottom would be clipped by the
/// `UIList`'s `OverflowHidden`; the port has no scissor, so they are skipped.
/// The scrollbar draws statically — it never scrolls, leaving the ~40 first
/// of the 213 styles visible — while clicks on the clipped rows stay valid
/// (the source's `UIElement.GetElementAt` does not cull clipped children
/// either). The boundary row (row 3 at the 1600x900 reference: top 484) has
/// only ~8 px above the menu's bottom edge (492) and hangs ~36 px below it,
/// an artifact C# clips away to that sliver.
fn draw_hairstyles_menu(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    state: &CreationState,
    l: &Layout,
    mouse: Vec2,
) {
    let bottom = l.hair_menu[1] + l.hair_menu[3];
    for (i, hair) in HAIRSTYLES.iter().enumerate() {
        let rect = hair_button(l.hair_grid, i);
        if rect[1] >= bottom {
            break;
        }
        draw_hair_button(r, lookups, rect, state.draft.hair == *hair, in_rect(mouse, rect));
    }
    panel::draw_scrollbar(r, lookups, l.hair_scrollbar, l.hair_menu[3], l.hair_grid[3], false);
}

/// `MakeClothStylesMenu` (`:311-528`): the ten 44x80 cloth-style buttons, the
/// separator, the armor/template/voice chrome. The inner `UICharacter` and the
/// item icons are not drawn — the port decodes no per-variant body or
/// `Images/Item_*` sheets for them (documented deviation).
fn draw_cloth_styles_menu(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    state: &CreationState,
    l: &Layout,
    mouse: Vec2,
) {
    for (i, style) in VALID_CLOTH_STYLES.iter().enumerate() {
        let rect = l.cloth_buttons[i];
        let hovered = in_rect(mouse, rect);
        // `UIClothStyleButton.DrawSelf` (`UIClothStyleButton.cs:49-66`): the
        // `CategoryPanel` at `White * 0.5`, the selected
        // `CategoryPanelHighlight` inset 3, the hovered border.
        draw_spliced_base(r, lookups, lookups.cc_panel, rect, [0.5; 4]);
        if state.draft.skin_variant == *style {
            draw_spliced_base(
                r,
                lookups,
                lookups.cc_panel_highlight,
                [rect[0] + 3.0, rect[1] + 3.0, rect[2] - 6.0, rect[3] - 6.0],
                [1.0; 4],
            );
        }
        if hovered {
            draw_spliced_base(r, lookups, lookups.cc_panel_border, rect, [1.0; 4]);
        }
    }
    // The separator (`:441-449`).
    panel::draw_horizontal_separator(
        r,
        lookups,
        l.cloth_separator[0],
        l.cloth_separator[1],
        l.cloth_separator[2],
        color::tint(uistate::CREATION_SEPARATOR_COLOR),
    );
    // The armor row (`:348-390`): SmallPanel-backed buttons, no item icons
    // (`Images/Item_271/5660/91/239/237` are not decoded).
    for rect in l.armor_buttons {
        uistate::draw_colored_image_button(
            r,
            lookups,
            true,
            rect,
            None,
            None,
            [255; 4],
            false,
            in_rect(mouse, rect),
        );
    }
    // The template row (`:396-430`): Copy/Paste/Randomize.
    for k in 0..3 {
        let sheet = lookups.cc_copy_paste_random[k];
        uistate::draw_colored_image_button(
            r,
            lookups,
            true,
            l.template_buttons[k],
            None,
            sheet,
            [255; 4],
            false,
            in_rect(mouse, l.template_buttons[k]),
        );
    }
    // The voice play button (`:474-504`): a null-texture `UIColoredImageButton`
    // — the SmallPanel back panel still draws (`UIColoredImageButton.cs:33-45`,
    // `:72`), the null only silences the icon slot (`:87`) — here carrying the
    // `Voice` icon in `OurFavoriteColor`.
    uistate::draw_colored_image_button(
        r,
        lookups,
        true,
        l.voice_button,
        None,
        lookups.cc_voice,
        uistate::OUR_FAVORITE_COLOR,
        false,
        in_rect(mouse, l.voice_button),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    // Expected values hand-derived from the C# (`UICharacterCreation.cs:25-234`,
    // `:254-1055`, `:1175-1235`, `:1659-1711`, `:1876-2022`,
    // `HairstyleUnlocksHelper.cs:49-88`, `ID/PlayerVariantID.cs:20`,
    // `Player.cs:826,2554-2566`, `UIColoredSlider.cs:47-133`,
    // `UICharacter.cs:34-130`, `Main.cs:47890-47980`) — never from this code.

    const SCREEN: Vec2 = Vec2::new(1600.0, 900.0);

    #[test]
    fn draft_defaults_match_the_player_field_initializers() {
        // `Player.cs:826,2554-2566` + `UICharacterCreation.cs:169`.
        let d = NewPlayerDraft::new();
        assert!(d.name.is_empty());
        assert_eq!(d.difficulty, 0);
        assert_eq!(d.hair, 0);
        assert_eq!(d.skin_variant, 0);
        assert_eq!(d.hair_color, [215, 90, 55, 255]);
        assert_eq!(d.skin_color, [255, 125, 90, 255]);
        assert_eq!(d.eye_color, [105, 90, 75, 255]);
        assert_eq!(d.shirt_color, [175, 165, 140, 255]);
        assert_eq!(d.undershirt_color, [160, 180, 215, 255]);
        assert_eq!(d.pants_color, [255, 230, 175, 255]);
        assert_eq!(d.shoe_color, [160, 105, 60, 255]);
        assert!(!d.dirty);
    }

    #[test]
    fn dirty_tracks_the_name_and_the_template_only() {
        // `:2011-2021`: the name or the template (hair/cloth/colours — not the
        // difficulty) flips the flag, and it never un-flips.
        let mut d = NewPlayerDraft::new();
        d.refresh_dirty();
        assert!(!d.dirty, "a fresh draft is clean");
        d.difficulty = 2;
        d.refresh_dirty();
        assert!(!d.dirty, "the difficulty is not part of the template");
        d.name = "Zealot".into();
        d.refresh_dirty();
        assert!(d.dirty, "the name dirties");
        let mut d = NewPlayerDraft::new();
        d.hair = 5;
        d.refresh_dirty();
        assert!(d.dirty, "the hair dirties");
        let mut d = NewPlayerDraft::new();
        d.shoe_color = [1, 2, 3, 255];
        d.refresh_dirty();
        assert!(d.dirty, "a colour dirties");
        let mut d = NewPlayerDraft::new();
        d.dirty = true;
        d.refresh_dirty();
        assert!(d.dirty, "the flag never clears");
    }

    #[test]
    fn hairstyle_table_matches_the_rebuild_list() {
        // `HairstyleUnlocksHelper.cs:49-88`: the creation-time list is 213 ids,
        // ordered 0..=50, the bracket singles, 51..=122, more singles,
        // 166..=227.
        assert_eq!(HAIRSTYLES.len(), 213);
        assert_eq!(HAIRSTYLES[0], 0);
        assert_eq!(HAIRSTYLES[50], 50);
        assert_eq!(HAIRSTYLES[51], 136);
        assert_eq!(HAIRSTYLES[59], 144);
        assert_eq!(HAIRSTYLES[60], 147);
        assert_eq!(HAIRSTYLES[64], 151);
        assert_eq!(HAIRSTYLES[65], 154);
        assert_eq!(HAIRSTYLES[69], 161);
        assert_eq!(HAIRSTYLES[70], 51);
        assert_eq!(HAIRSTYLES[141], 122);
        assert_eq!(HAIRSTYLES[142], 134);
        assert_eq!(HAIRSTYLES[150], 160);
        assert_eq!(HAIRSTYLES[151], 166);
        assert_eq!(HAIRSTYLES[212], 227);
        // The independent sum of the seven RebuildList ranges.
        assert_eq!(
            HAIRSTYLES.iter().sum::<i32>(),
            1275 + 1260 + 745 + 785 + 6228 + 1360 + 12183
        );
    }

    #[test]
    fn cloth_styles_and_gender_tables_match_the_source() {
        // `_validClothStyles` (`:58`) and `ID/PlayerVariantID.cs:20`.
        assert_eq!(VALID_CLOTH_STYLES, [0, 2, 1, 3, 8, 9, 7, 5, 6, 4]);
        assert!(is_male(0) && is_male(1) && is_male(2) && is_male(3) && is_male(8) && is_male(10));
        assert!(!is_male(9) && !is_male(7) && !is_male(5) && !is_male(6) && !is_male(4));
        // `_defaultHairstylesForClothStyle` (`:60-72`).
        assert_eq!(default_hairstyle_for_cloth_style(0), Some(0));
        assert_eq!(default_hairstyle_for_cloth_style(2), Some(1));
        assert_eq!(default_hairstyle_for_cloth_style(1), Some(12));
        assert_eq!(default_hairstyle_for_cloth_style(3), Some(2));
        assert_eq!(default_hairstyle_for_cloth_style(8), Some(28));
        assert_eq!(default_hairstyle_for_cloth_style(9), Some(68));
        assert_eq!(default_hairstyle_for_cloth_style(7), Some(18));
        assert_eq!(default_hairstyle_for_cloth_style(5), Some(22));
        assert_eq!(default_hairstyle_for_cloth_style(6), Some(81));
        assert_eq!(default_hairstyle_for_cloth_style(4), Some(5));
        assert_eq!(default_hairstyle_for_cloth_style(11), None);
    }

    #[test]
    fn layout_matches_the_source_geometry_at_1600x900() {
        let l = layout(SCREEN, 40.0);
        // Root: 500 x (380+4) at Top 220, HAlign .5 (`:186-193`).
        assert_eq!(l.root, [550.0, 220.0, 500.0, 384.0]);
        // Panel: Top 50, H 384-150-4 (`:196-204`).
        assert_eq!(l.panel, [550.0, 270.0, 500.0, 230.0]);
        // Preview: 80x80, Top -70, HAlign .5 (`:236-252`).
        assert_eq!(l.preview, [760.0, 200.0, 80.0, 80.0]);
        // Top container + PaddingTop-4 inner (`:207-215`).
        assert_eq!(l.top, [550.0, 270.0, 500.0, 50.0]);
        assert_eq!(l.top_inner, [550.0, 274.0, 500.0, 46.0]);
        // Middle: Top 50+6, H 230-70, PaddingTop 3 (`:216-225`).
        assert_eq!(l.middle, [550.0, 326.0, 500.0, 160.0]);
        assert_eq!(l.middle_inner, [550.0, 329.0, 500.0, 157.0]);
        // Separator: Width 475, Left -2.5, HAlign .5, VAlign 1 + Top 6
        // (`:770-778`) — x = 550 - 2.5 + (500-475)/2 = 560 (the plan's 562.5
        // drops the Left offset).
        assert_eq!(l.separator, [560.0, 322.0, 475.0, 4.0]);
        // Back/Create: VAlign 1 of the root + Top -45 — y = 220+384-50-45 = 509
        // (the plan's 559 is the button *bottom*; its own "604-45" arithmetic
        // skips the height subtraction). Create's HAlign 1 lands at
        // 550+500-240 = 810 — the char-select New's twin (`:1190-1197`).
        assert_eq!(l.back, [550.0, 509.0, 240.0, 50.0]);
        assert_eq!(l.create, [810.0, 509.0, 240.0, 50.0]);
    }

    #[test]
    fn category_bar_lands_the_ten_buttons() {
        // `CreateColorPicker` (`:828-833`): `Left (-240 + 48*id, 0.5)` of the
        // 500-wide top inner -> x = 560 + 48*id, y = 274 (the plan's table).
        let l = layout(SCREEN, 40.0);
        let xs: Vec<f32> = l.categories.iter().map(|c| c[0]).collect();
        assert_eq!(
            xs,
            vec![560.0, 608.0, 656.0, 704.0, 752.0, 800.0, 848.0, 896.0, 944.0, 992.0]
        );
        for c in l.categories.iter() {
            assert_eq!((c[1], c[2], c[3]), (274.0, 44.0, 44.0));
        }
    }

    #[test]
    fn info_menu_lands_the_plate_column_and_highlight() {
        let l = layout(SCREEN, 40.0);
        // The info container (padding 10 sides) inside the middle inner.
        assert_eq!(l.info_name, [560.0, 329.0, 480.0, 40.0]); // `:858-861`
        // The column: W 0.4*480-4 = 188, H 157-50 = 107, VAlign 1 (`:865-876`).
        assert_eq!(l.info_column, [560.0, 379.0, 188.0, 107.0]);
        // The highlight: W 0.6*480-8 = 280, Left -4, HAlign 1 -> right edge at
        // 1040-4 = 1036, x = 756 (the plan's 760 drops the Left).
        assert_eq!(l.info_highlight, [756.0, 379.0, 280.0, 107.0]);
        // The rows: 26.75 tall at VAlign 0 / 1/3 / 2/3 / 1 (`:889-916`).
        let h = 107.0 / 4.0;
        assert_eq!(l.difficulty_buttons[0], [560.0, 379.0, 188.0, h]);
        assert_eq!(l.difficulty_buttons[1], [560.0, 379.0 + h, 188.0, h]);
        assert_eq!(l.difficulty_buttons[2], [560.0, 379.0 + h * 2.0, 188.0, h]);
        assert_eq!(l.difficulty_buttons[3], [560.0, 379.0 + h * 3.0, 188.0, h]);
    }

    #[test]
    fn hsl_menu_lands_the_container_sliders_and_buttons() {
        let l = layout(SCREEN, 40.0);
        assert_eq!(l.hsl, [690.0, 329.0, 220.0, 158.0]); // `:969-977`
        assert_eq!(l.hsl_panel, [690.0, 339.0, 220.0, 104.0]); // `:978-988`
        // The bars: right edge 690 + 210 - 10 = 890, centre y = 342 + 30k + 18
        // (`:1045-1055` + `UIColoredSlider.cs:66-67`).
        assert_eq!(l.hsl_sliders[0], [890.0, 360.0]);
        assert_eq!(l.hsl_sliders[1], [890.0, 390.0]);
        assert_eq!(l.hsl_sliders[2], [890.0, 420.0]);
        // The bar rects: (712, 352+30k, 178, 16).
        let (bar, hit) = hsl_slider_rects(890.0, 360.0);
        assert_eq!(bar, [712.0, 352.0, 178.0, 16.0]);
        // The hit rectangle: `(X+5-2, Y+4, W-4, H-8)` (`UIColoredSlider.cs:105-108`).
        assert_eq!(hit, [715.0, 356.0, 174.0, 8.0]);
        // Hex panel + buttons (`:992-1032`).
        assert_eq!(l.hsl_hex_panel, [810.0, 455.0, 100.0, 32.0]);
        assert_eq!(l.hsl_buttons[0], [690.0, 455.0, 32.0, 32.0]);
        assert_eq!(l.hsl_buttons[1], [730.0, 455.0, 32.0, 32.0]);
        assert_eq!(l.hsl_buttons[2], [770.0, 455.0, 32.0, 32.0]);
    }

    #[test]
    fn hair_grid_lands_the_container_and_buttons() {
        let l = layout(SCREEN, 40.0);
        // The menu container: W 490 (500-10), HAlign .5, VAlign .5, Top 6
        // (`:257-264`).
        assert_eq!(l.hair_menu, [555.0, 335.0, 490.0, 157.0]);
        // The grid content: the list inner (559, 339) with 48 * 22 rows.
        assert_eq!(l.hair_grid[0], 559.0);
        assert_eq!(l.hair_grid[1], 339.0);
        assert!((l.hair_grid[2] - 464.0).abs() < 1e-4);
        assert_eq!(l.hair_grid[3], 48.0 * 22.0);
        // The buttons: `(i%10)*46 + 6`, `(i/10)*48 + 1` (`:291-302`).
        assert_eq!(hair_button(l.hair_grid, 0), [565.0, 340.0, 44.0, 44.0]);
        assert_eq!(hair_button(l.hair_grid, 9), [565.0 + 9.0 * 46.0, 340.0, 44.0, 44.0]);
        assert_eq!(hair_button(l.hair_grid, 10), [565.0, 340.0 + 48.0, 44.0, 44.0]);
        // The scrollbar: HAlign 1, Top 10, H 127 (`:274-280`).
        assert_eq!(l.hair_scrollbar, [1025.0, 345.0, 20.0, 127.0]);
    }

    #[test]
    fn cloth_menu_lands_the_buttons_and_rows() {
        let l = layout(SCREEN, 40.0);
        // The buttons: `i*46 + 11/29`, `Top 0` (`:322-335`) — the plan's
        // "y = 367.5" drops the source's zero Top.
        for (i, b) in l.cloth_buttons.iter().enumerate() {
            let num2 = if i < 5 { 11.0 } else { 29.0 };
            assert_eq!(b[0], 550.0 + i as f32 * 46.0 + num2);
            assert_eq!((b[1], b[2], b[3]), (329.0, 44.0, 80.0));
        }
        assert_eq!(l.cloth_buttons[0][0], 561.0);
        assert_eq!(l.cloth_buttons[5][0], 809.0);
        // The separator: W 462, Left -3, HAlign .5, VAlign 1, Top -67 (`:441-449`).
        assert_eq!(l.cloth_separator, [566.0, 415.0, 462.0, 4.0]);
        // The armor row: Left (26, 0.5) -> x 826, VAlign 1, Top -22 -> y 414.
        assert_eq!(l.armor_buttons[0], [826.0, 423.0, 32.0, 32.0]);
        assert_eq!(l.armor_buttons[1], [895.0, 423.0, 32.0, 32.0]); // HAlign .5
        assert_eq!(l.armor_buttons[2], [860.5, 423.0, 32.0, 32.0]); // HAlign .25
        assert_eq!(l.armor_buttons[3], [929.5, 423.0, 32.0, 32.0]); // HAlign .75
        assert_eq!(l.armor_buttons[4], [964.0, 423.0, 32.0, 32.0]); // HAlign 1
        // The template row: Left (60, 0.5) -> x 860, Top 14 -> y 450.
        assert_eq!(l.template_buttons[0], [860.0, 459.0, 32.0, 32.0]);
        assert_eq!(l.template_buttons[1], [894.0, 459.0, 32.0, 32.0]);
        assert_eq!(l.template_buttons[2], [928.0, 459.0, 32.0, 32.0]);
        // The voice play button: 52x52 centred in the voice row — the row's
        // `Left (-60, -0.5)` + `HAlign 1` puts it at 650 (`:431-440`, `:474-480`).
        assert_eq!(l.voice_button, [669.0, 420.0, 52.0, 52.0]);
    }

    #[test]
    fn middle_menu_maps_the_categories() {
        // `Click_*`/`SelectColorPicker` (`:1250-1275`, `:1810-1833`).
        assert_eq!(middle_for(0), MiddleMenu::Info);
        assert_eq!(middle_for(1), MiddleMenu::ClothStyles);
        assert_eq!(middle_for(2), MiddleMenu::Hairstyles);
        for id in 3..=9 {
            assert_eq!(middle_for(id), MiddleMenu::Hsl { color_id: id });
        }
        // The category texture table (`:735-741`) over the 8 `cc_colors`
        // sheets — Hair, Eye, EyeBack, Skin, Shirt, Undershirt, Pants, Shoes
        // (`assets.rs:988-997`); Skin..Shoes sit past the `EyeBack` entry.
        assert_eq!(category_texture(3), Some(0));
        assert_eq!(category_texture(4), Some(1));
        assert_eq!(category_texture(5), Some(3));
        assert_eq!(category_texture(6), Some(4));
        assert_eq!(category_texture(7), Some(5));
        assert_eq!(category_texture(8), Some(6));
        assert_eq!(category_texture(9), Some(7));
        assert_eq!(category_texture(0), None);
        assert_eq!(category_texture(1), None);
        assert_eq!(category_texture(2), None);
    }

    #[test]
    fn scaled_hsl_round_trips_the_grey_and_saturated_branches() {
        // `[0, 0, 1]` -> `hslToRgb(0, 0, 1.0)`'s zero-saturation branch
        // (`Main.cs:47895-47900`): `round(1.0 * 255) = 255` grey.
        assert_eq!(hsl_to_rgb_scaled([0.0, 0.0, 1.0]), [255, 255, 255, 255]);
        // A saturated case: `[0, 1, 0.5]` -> `hslToRgb(0, 1, 0.575)` —
        // num2 = 1, t = 0.15; red = 255, green/blue = round(0.15*255) = 38.
        assert_eq!(hsl_to_rgb_scaled([0.0, 1.0, 0.5]), [255, 38, 38, 255]);
        // `RgbToScaledHsl` inverts the 0.85 luminosity squeeze (`:2359-2365`).
        let rgb = [215u8, 90, 55, 255]; // the default hair colour
        let hsl = rgb_to_hsl_scaled(rgb);
        assert_eq!(hsl_to_rgb_scaled(hsl), rgb);
        // A grey's z is `(l - 0.15) / 0.85` with l = 128/255.
        let hsl = rgb_to_hsl_scaled([128, 128, 128, 255]);
        assert!(hsl[0].abs() < 1e-6 && hsl[1].abs() < 1e-6);
        let expected_z = (128.0 / 255.0 - 0.15) / 0.85;
        assert!((hsl[2] - expected_z).abs() < 1e-5);
    }

    #[test]
    fn hex_text_matches_gethextext() {
        // `"#" + Hex3().ToUpper()` (`:1170-1173`).
        assert_eq!(hex_text([215, 90, 55, 255]), "#D75A37");
        assert_eq!(hex_text([0, 0, 0, 255]), "#000000");
        assert_eq!(hex_text([255, 255, 255, 255]), "#FFFFFF");
    }

    #[test]
    fn hue_bar_mid_blip_computes_through_the_scaled_converter() {
        // `GetHSLSliderColorAt(Hue, 0.5)` = `ScaledHslToRgb(0.5, 1, 0.5)`
        // (`:1130`) — hue 0.5 is cyan, so the bytes are (38, 255, 255) (the
        // plan's "red-ish" note mislabels the hue wheel).
        assert_eq!(slider_blip_color(0, 0.5, [0.0; 3]), [38, 255, 255, 255]);
        // The saturation bar at the live hue reads the live axes (`:1131`).
        let hsl = [0.0, 0.5, 0.5];
        assert_eq!(
            slider_blip_color(1, 1.0, hsl),
            slider_blip_color(0, 0.0, hsl)
        );
        // The luminance bar rides `point_at` (`:1132`): 0 -> black-luminosity
        // 0.15, 1 -> white.
        assert_eq!(slider_blip_color(2, 0.0, [0.5, 1.0, 0.5]), [0, 77, 77, 255]);
        assert_eq!(slider_blip_color(2, 1.0, [0.5, 1.0, 0.5]), [255, 255, 255, 255]);
    }

    #[test]
    fn lerp_rgb_truncates_through_the_vector_space() {
        // `Color.Lerp(Black, Cyan, 0.8) * 0.5` (`UIDifficultyButton.cs:56`):
        // (0, 204, 204) then (0, 102, 102).
        assert_eq!(
            color::mul(lerp_rgb([0, 0, 0, 255], [0, 255, 255, 255], 0.8), 0.5),
            [0, 102, 102, 127]
        );
        // `Color.Lerp(Cyan, White, 0.7) * 0.5` (`:59`): 178.5 -> 178, then 89.
        assert_eq!(
            color::mul(lerp_rgb([0, 255, 255, 255], [255, 255, 255, 255], 0.7), 0.5),
            [89, 127, 127, 127]
        );
    }

    #[test]
    fn click_table_routes_every_control() {
        let l = layout(SCREEN, 40.0);
        let centre = |r: [f32; 4]| Vec2::new(r[0] + r[2] / 2.0, r[1] + r[3] / 2.0);
        // The categories (`:735-768`).
        assert_eq!(click_at(&l, 0, centre(l.categories[0])), Click::Category(0));
        assert_eq!(click_at(&l, 0, centre(l.categories[9])), Click::Category(9));
        // Back/Create (`:1187`, `:1200`).
        assert_eq!(click_at(&l, 0, centre(l.back)), Click::Back);
        assert_eq!(click_at(&l, 0, centre(l.create)), Click::Create);
        // Info: the name plate and the four difficulty rows (`:863`,
        // `UIDifficultyButton.cs:67-71`).
        assert_eq!(click_at(&l, 0, centre(l.info_name)), Click::Name);
        assert_eq!(click_at(&l, 0, centre(l.difficulty_buttons[0])), Click::Difficulty(3));
        assert_eq!(click_at(&l, 0, centre(l.difficulty_buttons[1])), Click::Difficulty(0));
        assert_eq!(click_at(&l, 0, centre(l.difficulty_buttons[2])), Click::Difficulty(1));
        assert_eq!(click_at(&l, 0, centre(l.difficulty_buttons[3])), Click::Difficulty(2));
        // Hairstyles: the first grid cell is hair 0 (`UIHairStyleButton.cs:66-70`).
        assert_eq!(click_at(&l, 2, centre(hair_button(l.hair_grid, 0))), Click::Hair(0));
        assert_eq!(click_at(&l, 2, centre(hair_button(l.hair_grid, 12))), Click::Hair(12));
        assert_eq!(click_at(&l, 2, centre(hair_button(l.hair_grid, 70))), Click::Hair(51));
        // ClothStyles: the buttons map through `_validClothStyles` (`:1284-1287`).
        assert_eq!(click_at(&l, 1, centre(l.cloth_buttons[0])), Click::ClothStyle(0));
        assert_eq!(click_at(&l, 1, centre(l.cloth_buttons[5])), Click::ClothStyle(9));
        assert_eq!(click_at(&l, 1, centre(l.cloth_buttons[9])), Click::ClothStyle(4));
        // The HSL menu's rows drag, they do not click.
        assert_eq!(click_at(&l, 3, centre([712.0, 352.0, 178.0, 16.0])), Click::None);
    }

    #[test]
    fn tooltip_table_mirrors_the_draw_chain() {
        assert_eq!(tooltip_for(HoverTarget::Category(0)), Some("Info"));
        assert_eq!(tooltip_for(HoverTarget::Category(1)), Some("Style"));
        assert_eq!(tooltip_for(HoverTarget::Category(2)), Some("Hairstyle"));
        assert_eq!(tooltip_for(HoverTarget::Category(3)), Some("Hair"));
        assert_eq!(tooltip_for(HoverTarget::Category(4)), Some("Eyes"));
        assert_eq!(tooltip_for(HoverTarget::Category(5)), Some("Skin "));
        assert_eq!(tooltip_for(HoverTarget::Category(6)), Some("Shirt "));
        assert_eq!(tooltip_for(HoverTarget::Category(7)), Some("Undershirt"));
        assert_eq!(tooltip_for(HoverTarget::Category(8)), Some("Pants"));
        assert_eq!(tooltip_for(HoverTarget::Category(9)), Some("Shoes"));
        assert_eq!(
            tooltip_for(HoverTarget::CopyHex),
            Some("Copy color code to clipboard")
        );
        assert_eq!(
            tooltip_for(HoverTarget::PasteHex),
            Some("Paste color code from clipboard")
        );
        assert_eq!(tooltip_for(HoverTarget::RandomizeColor), Some("Randomize color"));
    }

    #[test]
    fn preview_anchor_matches_get_player_position() {
        // `UICharacter.cs:122-130`: TL + (W/2 - 10, H/2 - 21) — the pet branch
        // is dead (`UICharacter.NoPets`, `:51`). For the 80x80 preview at
        // (760, 200): (790, 219). The plan's "(785, 208.5)" multiplies the
        // hitbox half by the sprite scale and centres on the element, which is
        // not what the source computes.
        assert_eq!(preview_anchor([760.0, 200.0, 80.0, 80.0]), [790.0, 219.0]);
        // The 59x58 card preview would anchor the same way (Task 3's hook).
        assert_eq!(preview_anchor([0.0, 0.0, 59.0, 58.0]), [19.5, 8.0]);
    }

    #[test]
    fn animation_row_follows_the_global_time() {
        // `(int)(time / 0.07) % 14 + 6` (`UICharacter.cs:92`): rows 6..=19.
        assert_eq!(animated_frame_row(0.0), 6);
        assert_eq!(animated_frame_row(0.07), 7);
        assert_eq!(animated_frame_row(0.07 * 14.0), 6);
        assert_eq!(animated_frame_row(0.07 * 27.0), 19);
    }

    #[test]
    fn slider_drag_latches_updates_and_releases() {
        let l = layout(SCREEN, 40.0);
        let mut state = CreationState::new();
        state.category = 3; // HairColor -> the HSL menu.
        let before = state.draft.hair_color;
        // A press outside the bars latches nothing.
        hsl_sliders_update(&l, &mut state, Vec2::new(10.0, 10.0), true);
        assert_eq!(state.hsl_grab, 0);
        assert_eq!(state.draft.hair_color, before);
        // Pressing in the hue bar latches row 1 and writes the hue.
        let (_, hit) = hsl_slider_rects(l.hsl_sliders[0][0], l.hsl_sliders[0][1]);
        let mid = Vec2::new(hit[0] + hit[2] * 0.5, hit[1] + hit[3] * 0.5);
        hsl_sliders_update(&l, &mut state, mid, true);
        assert_eq!(state.hsl_grab, 1);
        // Holding and sliding right of the bar clamps to 1. Hue 1.0 through
        // `Main.hue2rgb` does NOT wrap to red: `c2 = 1.0` fails `c > 1` and
        // reads `t1` — the byte triple is (215, 55, 55) for the default
        // saturation/luminosity, not (255, 90, 55).
        let far = Vec2::new(hit[0] + hit[2] + 40.0, hit[1] + 1.0);
        hsl_sliders_update(&l, &mut state, far, true);
        assert_eq!(state.hsl_grab, 1);
        assert_eq!(state.draft.hair_color, [215, 55, 55, 255]);
        // Releasing clears the latch and freezes the colour.
        let frozen = state.draft.hair_color;
        hsl_sliders_update(&l, &mut state, far, false);
        assert_eq!(state.hsl_grab, 0);
        hsl_sliders_update(&l, &mut state, mid, false);
        assert_eq!(state.draft.hair_color, frozen);
        // The saturation row over the default skin colour.
        let mut state = CreationState::new();
        state.category = 5; // Skin
        let before = state.draft.skin_color;
        let (_, hit1) = hsl_slider_rects(l.hsl_sliders[1][0], l.hsl_sliders[1][1]);
        let mid1 = Vec2::new(hit1[0] + hit1[2] * 0.5, hit1[1] + 1.0);
        hsl_sliders_update(&l, &mut state, mid1, true);
        assert_eq!(state.hsl_grab, 2);
        assert_ne!(state.draft.skin_color, before);
        // rgbToHsl(255,125,90) = (0.035354, 1, 0.67647); sat -> 0.5 gives
        // `hslToRgb(0.035354, 0.5, 0.67647)` = (214, 149, 131).
        assert_eq!(state.draft.skin_color, [214, 149, 131, 255]);
    }

    #[test]
    fn slider_value_divides_over_the_hit_rect_and_clamps() {
        let hit = [100.0, 0.0, 174.0, 8.0];
        assert!((hsl_slider_value(hit, 187.0) - 0.5).abs() < 1e-6); // `:130-133`
        assert_eq!(hsl_slider_value(hit, 100.0), 0.0);
        assert_eq!(hsl_slider_value(hit, 274.0), 1.0);
        assert_eq!(hsl_slider_value(hit, 50.0), 0.0);
        assert_eq!(hsl_slider_value(hit, 500.0), 1.0);
    }

    #[test]
    fn difficulty_tables_map_the_rows() {
        // `MakeInfoMenu` (`:889-910`) + `UpdateDifficultyDescription`
        // (`:946-965`) — the build order is Journey, Classic, Mediumcore,
        // Hardcore; the description index is the difficulty byte.
        assert_eq!(DIFFICULTY_IDS, [3, 0, 1, 2]);
        assert_eq!(DIFFICULTY_COLORS[0], [0, 255, 255, 255]);
        assert_eq!(DIFFICULTY_COLORS[1], uistate::MC_COLOR);
        assert_eq!(DIFFICULTY_COLORS[2], uistate::HC_COLOR);
        assert_eq!(DIFFICULTY_COLORS[3], uistate::CREATIVE_MODE_COLOR);
        assert_eq!(DIFFICULTY_DESCRIPTIONS[0], "Classic characters drop money on death.");
        assert_eq!(DIFFICULTY_DESCRIPTIONS[2], "Hardcore characters die for good.");
        assert!(DIFFICULTY_DESCRIPTIONS[3].starts_with("Journey characters"));
    }

    #[test]
    fn lookups_default_leaves_every_creation_sheet_none() {
        // Review Focus 7: the draw helpers guard every sheet, so the default
        // set renders the chrome blank without panicking.
        let lookups = assets::Lookups::default();
        assert!(lookups.cc_color_character.is_none());
        assert!(lookups.cc_panel.is_none());
        assert!(lookups.cc_colors.iter().all(|s| s.is_none()));
    }
}
