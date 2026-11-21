//! The Advanced world-creation tab (`UIWorldCreationAdvanced`,
//! `GameContent/UI/States/UIWorldCreationAdvanced.cs:16-532`) — W4-d85
//! row 11. The 500-wide root at Top 202 (MaxHeight 400), the -102 panel,
//! the randomize icon + seed plate row, the 6-per-row 60x60 seed-option
//! grid (the ten `WorldGenerationOptions` registrations), the 88 px
//! description panel with its title/separator/body, and the fat Apply
//! panel. The secret-seed button appears in the grid's next cell when a
//! micro seed is entered or parked (`RefreshSecretSeedButton`, `:164-204`).
//!
//! The option toggles run the draft's `toggle_seed_option` (the
//! `AWorldGenerationOption.Enabled` cascades — Normal clears the rest,
//! Everything carries its seven dependencies); the seed plate opens the
//! same `UI.EnterSeed` keyboard the base screen uses, and the icon
//! randomizes through `Main.rand` (`RandomizeSeed`, `UIWorldCreation.cs:
//! 532-539`).
//!
//! Deviations (docs/ANALYSIS.md "Advanced world creation (W4-d85)"): the
//! special-seed ring / particle systems (`DrawSeedSystems`,
//! `DrawSpecialSeedRingCallback`) are not drawn; the gamepad snap points
//! are out of scope (§J.3); the option list's scrollbar only exists under
//! `allowScrolling` (the ctor default false — the base screen's entry,
//! `:53`), so the port draws no scrollbar and the grid never scrolls
//! (10 options + the secret cell fit the 2-row region).

use glam::Vec2;
use tr_render::SpriteRenderer;

use super::text;
use super::uistate::{self, Style};
use super::world_creation::NewWorldDraft;
use super::{color, in_rect, panel};
use crate::assets::Lookups;
use crate::lang;
use tr_core::seeds::OPTION_TABLE;

/// The option grid's column count (`AddSpecialSeedOptions`'s `num = 6`,
/// `:215`).
pub const COLUMNS: usize = 6;
/// The option button size (`CreateButton`, `:266-268`).
pub const BUTTON: f32 = 60.0;
/// The row pitch (`:229` — `num3 * 67 + 3`).
pub const ROW_PITCH: f32 = 67.0;

/// The Everything icon's glitch walker — `WorldSeedOption_Everything.
/// ProvideUIElement`'s `UIImageFramed` closure (`WorldSeedOption_
/// Everything.cs:69-98`): a 7x16 frame sheet, the counter trips on
/// `Main.rand.Next(min, max + 1)` — frame 0 holds 15..120 ticks, every
/// other frame advances after 3 (`Next(3, 4)` is always 3) — and frames
/// 4/8/12 roll a 1-in-3 variation re-pick (`Next(7)`). The C# state lives
/// in the button's closure (one per `UIWorldCreationAdvanced` ctor); the
/// port keeps it on the App and resets it at the screen's entry.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct EverythingGlitch {
    /// `glitchFrameCounter` — the ticks since the last frame advance.
    pub counter: i32,
    /// `glitchFrame` — the sheet row, 0..15.
    pub frame: i32,
    /// `glitchVariation` — the sheet column, 0..6.
    pub variation: i32,
}

impl EverythingGlitch {
    /// The closure's `OnUpdate` body (`:75-96`), one UI tick. `rng` is the
    /// bare `Main.rand` stand-in (the front end runs outside every swap).
    pub fn tick(&mut self, rng: &mut tr_core::UnifiedRandom) {
        let (min, max) = if self.frame == 0 { (15, 120) } else { (3, 3) };
        self.counter += 1;
        if self.counter >= rng.next_range(min, max + 1) {
            self.counter = 0;
            self.frame = (self.frame + 1) % 16;
            if matches!(self.frame, 4 | 8 | 12) && rng.next_range(0, 3) == 0 {
                self.variation = rng.next_range(0, 7);
            }
        }
    }
}

/// The Everything sheet's frame cell — `Utils.Frame(7, 16, variation,
/// frame)` (`Utils.cs:1344-1349`): integer cell size `width/7 × height/16`,
/// the cell at `(cellW * variation, cellH * frame)`. `UIImageFramed` draws
/// at the cell's own size (`UIImageFramed.cs:20-21`), not the sheet's.
pub fn everything_frame_src(sheet: &tr_render::SheetRect, glitch: EverythingGlitch) -> [f32; 4] {
    let cw = sheet.width / 7;
    let ch = sheet.height / 16;
    [
        (cw * glitch.variation as u32) as f32,
        (ch * glitch.frame as u32) as f32,
        cw as f32,
        ch as f32,
    ]
}

/// The `KeyName` per registration (`WorldSeedOption_*.cs:5-7`) — the
/// localization keys (`UI.{KeyName}` / `UI.{KeyName}_Title`) and the icon
/// sheet names (`Images/UI/WorldCreation/{KeyName}`,
/// `AWorldGenerationOption.cs:57-61`).
pub const KEY_NAMES: [&str; 10] = [
    "Seed_Normal",
    "Seed_NotTheBees",
    "Seed_Drunk",
    "Seed_Celebration",
    "Seed_TheConstant",
    "Seed_ForTheWorthy",
    "Seed_NoTraps",
    "Seed_Remix",
    "Seed_Everything",
    "Seed_Skyblock",
];

/// The description key of option `i` (`AWorldGenerationOption.Load`,
/// `:57`).
pub fn description_key(i: usize) -> String {
    format!("UI.{}", KEY_NAMES[i])
}

/// The title key of option `i` (`:58`).
pub fn title_key(i: usize) -> String {
    format!("UI.{}_Title", KEY_NAMES[i])
}

// --- Geometry (`BuildPage` → `MakeInfoMenu`, `:77-133`) -----------------------

/// The root: `Width 500`, `Height (-200, 1)` clamped to MaxHeight 400,
/// `Top 202`, `HAlign 0.5` (`:80-91`).
pub fn root_rect(screen: Vec2) -> [f32; 4] {
    uistate::resolve_with_max_h(
        [0.0, 0.0, screen.x, screen.y],
        Style::px(0.0),
        Style::px(202.0),
        Style::px(500.0),
        Style::new(-200.0, 1.0),
        0.5,
        0.0,
        0.0,
        0.0,
        500.0,
        400.0, // `MaxHeight = 400` when !allowScrolling (`:90`)
        [0.0; 4],
    )
}

/// The panel: `Width 100%`, `Height (-102, 1)`, padding 0 (`:94-100`).
pub fn panel_rect(root: &[f32; 4]) -> [f32; 4] {
    [root[0], root[1], root[2], root[3] - 102.0]
}

/// The info container's inner rect: the panel's `PaddingTop 8 /
/// PaddingBottom 12` column (`:103-113`) then the `SetPadding(10)` info
/// element with `PaddingTop/Bottom = 0` (`:119-128`).
pub fn info_rect(panel: &[f32; 4]) -> [f32; 4] {
    let column = uistate::inner(*panel, [0.0, 8.0, 0.0, 12.0]);
    uistate::inner(column, [10.0, 0.0, 10.0, 0.0])
}

/// The randomize button: 40x40 at HAlign 0, Top 0 (`AddSeedButtons`,
/// `:405-412`).
pub fn random_rect(info: &[f32; 4]) -> [f32; 4] {
    [info[0], info[1], 40.0, 40.0]
}

/// The seed plate: `Width (-44, 1)`, `Left 44`, Height 40 (the
/// `UICharacterNameButton` default, `UICharacterNameButton.cs:37`),
/// `DistanceFromTitleToOption = 29` (`:425-432`).
pub fn seed_plate_rect(info: &[f32; 4]) -> [f32; 4] {
    [info[0] + 44.0, info[1], info[2] - 44.0, 40.0]
}

/// The option list area: `Width (-48, 1)`, `Height (-138, 1)`, `Top 44`,
/// `Left 24` (`AddListArea`, `:138-146`).
pub fn list_rect(info: &[f32; 4]) -> [f32; 4] {
    [info[0] + 24.0, info[1] + 44.0, info[2] - 48.0, info[3] - 138.0]
}

/// The seed-button region's height (`:217-221`):
/// `ceil(count / 6) * 70 - 10` — 2 rows for the 10 options (130), the
/// secret cell rides the same region.
pub fn region_height(option_count: usize) -> f32 {
    (option_count as f32 / COLUMNS as f32).ceil() * 70.0 - 10.0
}

/// Option button `i`'s rect (`:226-229`): `HAlign = (i % 6) / 5`,
/// `Top = (i / 6) * 67 + 3`, 60x60.
pub fn option_rect(list: &[f32; 4], i: usize) -> [f32; 4] {
    let col = i % COLUMNS;
    let row = i / COLUMNS;
    let x = list[0] + (list[2] - BUTTON) * (col as f32 / 5.0);
    [x, list[1] + row as f32 * ROW_PITCH + 3.0, BUTTON, BUTTON]
}

/// Whether the secret-seed button shows (`RefreshSecretSeedButton`'s
/// flag, `:166`): a tracked interface seed, an entered text or a parked
/// text. The port's tracker is the draft's own lists (no settings
/// persistence — documented deviation).
pub fn secret_button_shows(draft: &NewWorldDraft) -> bool {
    draft.has_entered_special_seed() || draft.has_disabled_secret_seed()
}

/// The secret-seed button's rect (`:169-178`): the cell after the last
/// option — `HAlign = (count % 6) / 5`, `Top = (count / 6) * 67 + 3`.
pub fn secret_rect(list: &[f32; 4], option_count: usize) -> [f32; 4] {
    option_rect(list, option_count)
}

/// The description panel: `HAlign 0.5, VAlign 1, Width (0, 1),
/// Height 88, Top 2` (`AddDescriptionPanel`, `:312-320`).
pub fn description_rect(info: &[f32; 4]) -> [f32; 4] {
    [info[0], info[1] + info[3] - 88.0 + 2.0, info[2], 88.0]
}

/// The Apply panel: `Width (-10, 0.5)`, `Height 50`, `VAlign 1`,
/// `HAlign 0.5`, `Top -43` (`MakeBackAndCreatebuttons`, `:454-461`).
pub fn apply_rect(root: &[f32; 4]) -> [f32; 4] {
    let w = root[2] * 0.5 - 10.0;
    [
        root[0] + root[2] * 0.5 - w * 0.5,
        root[1] + root[3] - 43.0 - 50.0,
        w,
        50.0,
    ]
}

// --- The description text -----------------------------------------------------

/// The hovered element's (title, description) pair
/// (`ShowOptionDescription`/`ClearOptionDescription`, `:360-397`): an
/// option's own keys, the seed plate's `UI.WorldDescriptionSeed`, the
/// randomize icon's `UI.WorldCreationRandomizeSeedDescription`, the
/// secret button's joined-seed string under
/// `UI.WorldDescriptionSecretSeeds_Title`, else the default pair.
pub fn hover_description(
    draft: &NewWorldDraft,
    root: &[f32; 4],
    mouse: Vec2,
    joined_seeds: &str,
) -> (String, String) {
    let panel = panel_rect(root);
    let info = info_rect(&panel);
    let list = list_rect(&info);
    for i in 0..OPTION_TABLE.len() {
        if in_rect(mouse, option_rect(&list, i)) {
            return (lang::text(&title_key(i)), lang::text(&description_key(i)));
        }
    }
    if secret_button_shows(draft) && in_rect(mouse, secret_rect(&list, OPTION_TABLE.len())) {
        return (
            lang::text("UI.WorldDescriptionSecretSeeds_Title"),
            joined_seeds.to_string(),
        );
    }
    if in_rect(mouse, seed_plate_rect(&info)) {
        return (
            lang::text("UI.WorldDescriptionDefault_Title"),
            lang::text("UI.WorldDescriptionSeed"),
        );
    }
    if in_rect(mouse, random_rect(&info)) {
        return (
            lang::text("UI.WorldDescriptionDefault_Title"),
            lang::text("UI.WorldCreationRandomizeSeedDescription"),
        );
    }
    (
        lang::text("UI.WorldDescriptionDefault_Title"),
        lang::text("UI.WorldDescriptionDefault"),
    )
}

/// `GetJoinedSecretSeedString` (`UIWorldCreation.cs:786-815`): the
/// entered (or parked) texts joined with `|`, wrapped at `maxWidth` by
/// the font measure, lines dropped past `maxHeight`. Empty list → "-".
/// `measure` is the caller's font-width fn at scale 1 (the C# divides
/// the description's inner dimensions by 0.7 first, `:380`).
pub fn joined_secret_seed_string(
    texts: &[String],
    max_width: f32,
    max_height: f32,
    line_spacing: f32,
    measure: impl Fn(&str) -> f32,
) -> String {
    if texts.is_empty() {
        return "-".to_string();
    }
    let mut used_h = 0.0f32;
    let mut out = String::new();
    let mut current = texts[0].clone();
    for next in &texts[1..] {
        let candidate = format!("{current}|{next}");
        if measure(&candidate) >= max_width {
            if used_h <= max_height {
                out.push_str(&current);
                out.push('\n');
            }
            used_h += line_spacing;
            current = next.clone();
        } else {
            current = candidate;
        }
    }
    if used_h <= max_height {
        out.push_str(&current);
    }
    out
}

// --- Hits --------------------------------------------------------------------

/// What a press does — the `OnLeftMouseDown` subscriptions.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Click {
    /// `ClickRandomizeSeed` (`:441-445`).
    Randomize,
    /// `Click_SetSeed` → `OpenSeedInputMenu` (`:447-450`).
    Seed,
    /// `ClickSeedOption` (`:255-260`) — the option index.
    Option(usize),
    /// `SecretSeedButton_OnLeftClick` (`:206-211`) — the seeds list opens.
    SecretSeeds,
    /// `Click_GoBack` → `GoBack` (`:470-481`) — the base creation screen.
    Apply,
    None,
}

/// The click table over the live geometry.
pub fn click_at(draft: &NewWorldDraft, screen: Vec2, mouse: Vec2) -> Click {
    let root = root_rect(screen);
    if in_rect(mouse, apply_rect(&root)) {
        return Click::Apply;
    }
    let panel = panel_rect(&root);
    let info = info_rect(&panel);
    if in_rect(mouse, random_rect(&info)) {
        return Click::Randomize;
    }
    if in_rect(mouse, seed_plate_rect(&info)) {
        return Click::Seed;
    }
    let list = list_rect(&info);
    for i in 0..OPTION_TABLE.len() {
        if in_rect(mouse, option_rect(&list, i)) {
            return Click::Option(i);
        }
    }
    if secret_button_shows(draft) && in_rect(mouse, secret_rect(&list, OPTION_TABLE.len())) {
        return Click::SecretSeeds;
    }
    Click::None
}

// --- Draw --------------------------------------------------------------------

/// The panel background — `new Color(33, 43, 79) * 0.8f` (`:98`).
const PANEL_BG: [u8; 4] = [26, 34, 63, 204];
/// The description highlight — `Color.LightGray * 0.7f` (`:322`).
const DESC_HIGHLIGHT: [u8; 4] = [147, 147, 147, 178];
/// The description separator — `new Color(131, 135, 183)` (`:342`).
const DESC_SEPARATOR: [u8; 4] = [131, 135, 183, 255];

/// `UIWorldCreationAdvanced.Draw` (`:496-501`, minus the gamepad points
/// and `DrawSeedSystems`). `glitch` is the Everything icon's live frame
/// state (`ProvideUIElement`'s closure — the App ticks it once per UI
/// update while the screen is live).
pub fn draw_screen(
    r: &mut SpriteRenderer,
    lookups: &Lookups,
    draft: &NewWorldDraft,
    screen: Vec2,
    mouse: Vec2,
    glitch: EverythingGlitch,
) {
    let root = root_rect(screen);
    let panel = panel_rect(&root);
    panel::draw_panel(r, lookups, lookups.ui_panel_bg, panel, color::tint(PANEL_BG));
    panel::draw_panel(
        r,
        lookups,
        lookups.ui_panel_border,
        panel,
        color::tint(uistate::TEXT_PANEL_BORDER),
    );
    let info = info_rect(&panel);

    // The randomize icon (`GroupOptionButton<bool>`, option: true,
    // ShowHighlightWhenSelected = false — the base screen's icon-button
    // chrome with the 38x38 `IconEvilRandom`).
    let random = random_rect(&info);
    super::world_creation::draw_icon_button(
        r,
        lookups,
        random,
        lookups.wc_evil_icons[0],
        in_rect(mouse, random),
    );

    // The seed plate (`UICharacterNameButton`, DistanceFromTitleToOption
    // 29) — the base screen's plate draw.
    super::world_creation::draw_plate(
        r,
        lookups,
        seed_plate_rect(&info),
        &lang::text("UI.WorldCreationSeed"),
        &draft.seed,
        &lang::text("UI.WorldCreationSeedEmpty"),
        29.0,
        mouse,
    );

    // The option grid.
    let list = list_rect(&info);
    for i in 0..OPTION_TABLE.len() {
        let rect = option_rect(&list, i);
        let selected = draft.options_enabled[i];
        let hovered = in_rect(mouse, rect);
        draw_seed_button(r, lookups, rect, lookups.wc_seed_icons[i], selected, hovered, i, glitch);
    }
    // The secret-seed button (`RefreshSecretSeedButton`): the
    // `Seed_Secret` icon, selected = HasEnteredSpecialSeed.
    if secret_button_shows(draft) {
        let rect = secret_rect(&list, OPTION_TABLE.len());
        draw_seed_button(
            r,
            lookups,
            rect,
            lookups.wc_seed_secret,
            draft.has_entered_special_seed(),
            in_rect(mouse, rect),
            usize::MAX, // the secret cell is a plain UIImage (`:182-185`)
            glitch,
        );
    }

    // The description panel (`AddDescriptionPanel`, `:309-358`): the
    // highlight slice, the title at 0.82 (PaddingLeft/Right 20,
    // PaddingTop 6), the separator at Top 22, the wrapped body at 0.7
    // (Top 25 + PaddingTop 6).
    let desc = description_rect(&info);
    uistate::draw_spliced_panel(
        r,
        lookups,
        lookups.cc_panel_highlight,
        desc,
        10.0,
        10.0,
        10.0,
        10.0,
        color::tint(DESC_HIGHLIGHT),
    );
    let joined = {
        let texts = if draft.has_entered_special_seed() {
            &draft.secret_texts
        } else {
            &draft.disabled_secret_texts
        };
        let inner_w = (desc[2] - 40.0) / 0.7;
        let inner_h = (desc[3] - 30.0 - 6.0) / 0.7;
        joined_secret_seed_string(
            texts,
            inner_w,
            inner_h,
            text::line_height(lookups),
            |s| text::hud_text_width(r, lookups, s, 1.0),
        )
    };
    let (title, body) = hover_description(draft, &root, mouse, &joined);
    text::hud_text(r, lookups, desc[0] + 20.0, desc[1] + 6.0, 0.82, [1.0; 4], &title);
    panel::draw_horizontal_separator(
        r,
        lookups,
        desc[0],
        desc[1] + 22.0,
        desc[2],
        color::tint(DESC_SEPARATOR),
    );
    // The wrapped body (`IsWrapped = true`, `:354`).
    let mut y = desc[1] + 25.0 + 6.0;
    for line in wrap_lines(r, lookups, &body, 0.7, desc[2] - 40.0) {
        text::hud_text(r, lookups, desc[0] + 20.0, y, 0.7, [1.0; 4], &line);
        y += text::line_height(lookups) * 0.7;
    }

    // The Apply panel (`UI.Apply`, 0.65 large, the faded pair).
    let apply = apply_rect(&root);
    let label = lang::text("UI.Apply");
    let style = uistate::TextPanelStyle {
        text_scale: 0.65,
        large: true,
        padding: 12.0,
        bg: uistate::TEXT_PANEL_BG,
        border: uistate::TEXT_PANEL_BORDER,
    };
    let w = text::hud_text_width(r, lookups, &label, style.text_scale);
    uistate::draw_text_panel(r, lookups, apply, &style, &label, w, in_rect(mouse, apply));
}

/// One 60x60 seed-option button (`CreateButton`, `:262-290` +
/// `GroupOptionButton.DrawSelf`): the `PanelGrayscale` fill, the selected
/// highlight rim inset 4 (`InnerHighlightRim = 4`, `:268`), the hovered
/// border, then the icon at `Left -1` (`ProvideUIElement`,
/// `AWorldGenerationOption.cs:66-72`) tinted White when
/// selected/hovered, `White * 0.5` otherwise (`UpdateIconOpacity`,
/// `:292-307`). Option 8 (Everything) is a `UIImageFramed` over the 7x16
/// glitch sheet — only the live frame cell draws, at the cell's own size
/// (`WorldSeedOption_Everything.cs:69-98`).
#[allow(clippy::too_many_arguments)]
fn draw_seed_button(
    r: &mut SpriteRenderer,
    lookups: &Lookups,
    rect: [f32; 4],
    icon: Option<tr_render::SheetRect>,
    selected: bool,
    hovered: bool,
    option_index: usize,
    glitch: EverythingGlitch,
) {
    let (fill, accent) = super::world_creation::option_button_colors(
        selected,
        hovered,
        super::world_creation::OPTION_BUTTON_COLOR,
    );
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
        let rim = 4.0; // `InnerHighlightRim = 4` (`:268`)
        uistate::draw_spliced_panel(
            r,
            lookups,
            lookups.cc_panel_highlight,
            [rect[0] + rim, rect[1] + rim, rect[2] - rim * 2.0, rect[3] - rim * 2.0],
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
            [1.0; 4],
        );
    }
    // The icon: texture-sized at `Left -1` (`:70`), the opacity rule.
    if let Some(sheet) = icon {
        let tint = if selected || hovered { [1.0; 4] } else { [0.5; 4] };
        if option_index == 8 {
            // The Everything `UIImageFramed`: the live cell at its own
            // size (`UIImageFramed.cs:20-21/48`), `Left -1` (`:73`).
            let src = everything_frame_src(&sheet, glitch);
            let (aw, ah) = (lookups.atlas_size.0 as f32, lookups.atlas_size.1 as f32);
            let x0 = sheet.x as f32 + src[0];
            let y0 = sheet.y as f32 + src[1];
            let uv = [x0 / aw, y0 / ah, (x0 + src[2]) / aw, (y0 + src[3]) / ah];
            r.push_ui_sprite([rect[0] - 1.0, rect[1]], [src[2], src[3]], uv, tint);
        } else {
            super::push_whole(
                r,
                lookups,
                &sheet,
                [rect[0] - 1.0, rect[1], sheet.width as f32, sheet.height as f32],
                tint,
            );
        }
    }
}

/// The wrapped description body — the bestiary's greedy word wrap.
fn wrap_lines(
    r: &SpriteRenderer,
    lookups: &Lookups,
    text_in: &str,
    scale: f32,
    max_w: f32,
) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current = String::new();
    for word in text_in.split_whitespace() {
        let candidate = if current.is_empty() {
            word.to_string()
        } else {
            format!("{current} {word}")
        };
        if !current.is_empty() && text::hud_text_width(r, lookups, &candidate, scale) > max_w {
            lines.push(std::mem::take(&mut current));
            current = word.to_string();
        } else {
            current = candidate;
        }
    }
    if !current.is_empty() || lines.is_empty() {
        lines.push(current);
    }
    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `BuildPage`'s geometry at 1600x900, hand-derived from the C#.
    #[test]
    fn the_layout_matches_the_source_geometry() {
        let screen = Vec2::new(1600.0, 900.0);
        let root = root_rect(screen);
        // Height -200 of 900 = 700 clamped to MaxHeight 400.
        assert_eq!(root, [550.0, 202.0, 500.0, 400.0]);
        let panel = panel_rect(&root);
        assert_eq!(panel, [550.0, 202.0, 500.0, 298.0]);
        // The column inner: 202+8 .. 500-12 → (550, 210, 500, 278);
        // the info inner drops 10 each side.
        assert_eq!(info_rect(&panel), [560.0, 210.0, 480.0, 278.0]);
        let info = info_rect(&panel);
        assert_eq!(random_rect(&info), [560.0, 210.0, 40.0, 40.0]);
        assert_eq!(seed_plate_rect(&info), [604.0, 210.0, 436.0, 40.0]);
        assert_eq!(list_rect(&info), [584.0, 254.0, 432.0, 140.0]);
        // Option 0: HAlign 0, Top 3. Option 5: HAlign 1 → x = 584+372.
        assert_eq!(option_rect(&list_rect(&info), 0), [584.0, 257.0, 60.0, 60.0]);
        assert_eq!(option_rect(&list_rect(&info), 5), [956.0, 257.0, 60.0, 60.0]);
        // Option 6: row 1, col 0 → y = 254 + 67 + 3 = 324.
        assert_eq!(option_rect(&list_rect(&info), 6), [584.0, 324.0, 60.0, 60.0]);
        // The secret cell: index 10 → row 1, col 4 → HAlign 4/5.
        assert_eq!(secret_rect(&list_rect(&info), 10), [881.6, 324.0, 60.0, 60.0]);
        assert_eq!(region_height(10), 130.0);
        // The description: VAlign 1 of 278, Height 88, Top 2 → y 402.
        assert_eq!(description_rect(&info), [560.0, 402.0, 480.0, 88.0]);
        // Apply: y = 202 + 400 - 43 - 50 = 509, W 240 centred.
        assert_eq!(apply_rect(&root), [680.0, 509.0, 240.0, 50.0]);
    }

    /// The MaxHeight clamp releases on a short screen.
    #[test]
    fn the_root_height_clamps_at_400() {
        assert_eq!(root_rect(Vec2::new(1600.0, 900.0))[3], 400.0);
        // 480 - 200 = 280 < 400 — the percent height wins.
        assert_eq!(root_rect(Vec2::new(1600.0, 480.0))[3], 280.0);
    }

    /// `GetJoinedSecretSeedString` (`UIWorldCreation.cs:786-815`): the
    /// `|` join wraps at the measure width, "-" for an empty list.
    #[test]
    fn the_joined_seed_string_wraps_and_defaults() {
        let measure = |s: &str| s.chars().count() as f32 * 8.0;
        assert_eq!(joined_secret_seed_string(&[], 100.0, 100.0, 20.0, measure), "-");
        let texts = vec!["aaaa".to_string(), "bbbb".to_string(), "cccc".to_string()];
        // "aaaa|bbbb" = 9 chars = 72 < 100; +"|cccc" = 14 = 112 >= 100 → wrap.
        assert_eq!(
            joined_secret_seed_string(&texts, 100.0, 100.0, 20.0, measure),
            "aaaa|bbbb\ncccc"
        );
        // A tight height drops the overflow lines.
        assert_eq!(
            joined_secret_seed_string(&texts, 100.0, 10.0, 20.0, measure),
            "aaaa|bbbb\n"
        );
    }

    /// The secret button's show rule (`RefreshSecretSeedButton`, `:166`).
    #[test]
    fn the_secret_button_shows_on_entered_or_parked_texts() {
        let mut draft = NewWorldDraft::new(0, 0);
        assert!(!secret_button_shows(&draft));
        draft.add_seed_from_seedmenu("pumpkinseason");
        assert!(secret_button_shows(&draft));
        draft.enable_secret_seed_options(false);
        assert!(secret_button_shows(&draft), "the parked texts keep it");
        assert!(draft.has_disabled_secret_seed());
        draft.enable_secret_seed_options(true);
        assert!(draft.has_entered_special_seed());
        assert!(!draft.has_disabled_secret_seed());
    }

    /// The click table: the option grid, the secret cell, Apply.
    #[test]
    fn the_click_table_routes_the_grid() {
        let mut draft = NewWorldDraft::new(0, 0);
        let screen = Vec2::new(1600.0, 900.0);
        // Option 0's centre (614, 287).
        assert_eq!(click_at(&draft, screen, Vec2::new(614.0, 287.0)), Click::Option(0));
        // The randomize icon.
        assert_eq!(click_at(&draft, screen, Vec2::new(570.0, 220.0)), Click::Randomize);
        // The seed plate.
        assert_eq!(click_at(&draft, screen, Vec2::new(800.0, 220.0)), Click::Seed);
        // Apply.
        assert_eq!(click_at(&draft, screen, Vec2::new(800.0, 520.0)), Click::Apply);
        // The secret cell only when shown.
        assert_eq!(click_at(&draft, screen, Vec2::new(900.0, 340.0)), Click::None);
        draft.add_seed_from_seedmenu("pumpkinseason");
        assert_eq!(click_at(&draft, screen, Vec2::new(900.0, 340.0)), Click::SecretSeeds);
    }

    /// The Everything glitch walker (`WorldSeedOption_Everything.cs:75-96`):
    /// frame 0 holds `Next(15, 121)` ticks, every later frame advances
    /// after exactly 3 (`Next(3, 4)`), the frame wraps mod 16, and the
    /// 4/8/12 variation roll draws `Next(3)` then `Next(7)` on a hit.
    #[test]
    fn the_everything_glitch_walks_the_frame_sheet() {
        let mut g = EverythingGlitch::default();
        let mut rng = tr_core::UnifiedRandom::new(42);
        // Frame 0's threshold: one Next(15, 121) draw per tick until trip.
        let mut probe = tr_core::UnifiedRandom::new(42);
        let mut ticks = 0;
        loop {
            ticks += 1;
            if ticks >= probe.next_range(15, 121) {
                break;
            }
        }
        for _ in 0..ticks {
            g.tick(&mut rng);
        }
        assert_eq!(g.frame, 1, "frame 0 advanced after the rolled hold");
        assert_eq!(g.counter, 0);
        // Frames 1..3 advance after exactly 3 ticks each (Next(3,4) == 3).
        for want in [2, 3, 4] {
            for _ in 0..3 {
                g.tick(&mut rng);
            }
            assert_eq!(g.frame, want);
        }
        // The walker reaches frame 15 and wraps to 0.
        let mut g = EverythingGlitch { counter: 0, frame: 14, variation: 0 };
        for _ in 0..6 {
            g.tick(&mut rng);
        }
        assert_eq!(g.frame, 0, "mod-16 wrap");
        // The variation roll fires only on frames 4/8/12 with Next(3) == 0;
        // a forced stream proves the draw order (the Next(3,4) threshold
        // draw consumes a sample too, then Next(3), then Next(7) on a hit).
        let mut g = EverythingGlitch { counter: 2, frame: 3, variation: 0 };
        let mut rng = tr_core::UnifiedRandom::new(42);
        g.tick(&mut rng); // trips to frame 4, rolls Next(3)
        assert_eq!(g.frame, 4);
        let mut probe = tr_core::UnifiedRandom::new(42);
        let _ = probe.next_range(3, 4); // the threshold draw
        let roll = probe.next_range(0, 3);
        if roll == 0 {
            assert_eq!(g.variation, probe.next_range(0, 7));
        } else {
            assert_eq!(g.variation, 0, "the miss keeps the variation");
        }
    }

    /// `Utils.Frame(7, 16, variation, frame)` (`Utils.cs:1344-1349`): the
    /// integer cell size and the cell origin — a 420x960 sheet gives 60x60
    /// cells, variation 3 / frame 5 → (180, 300, 60, 60).
    #[test]
    fn the_everything_frame_cell_matches_utils_frame() {
        let sheet = tr_render::SheetRect { x: 100, y: 200, width: 420, height: 960, plain_frame: (0, 0) };
        let g = EverythingGlitch { counter: 0, frame: 5, variation: 3 };
        assert_eq!(everything_frame_src(&sheet, g), [180.0, 300.0, 60.0, 60.0]);
        // The default state draws cell (0, 0).
        assert_eq!(
            everything_frame_src(&sheet, EverythingGlitch::default()),
            [0.0, 0.0, 60.0, 60.0]
        );
    }
}
