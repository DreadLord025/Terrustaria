//! The Select World screen (the other half of `Main.menuMode == 888`): a
//! faithful immediate port of `UIWorldSelect` + `UIWorldListItem` +
//! `AWorldListItem`.
//!
//! The screen ctor is element-for-element the twin of `UICharacterSelect`'s
//! (`UIWorldSelect.cs:40-87` — the same setters in the same order, only the
//! header text differs), so [`layout`] and the screen chrome delegate to
//! `char_select` instead of duplicating it. The card is its own geometry
//! ([`card_layout`]): a 60x58 world icon instead of the 59x58 player preview,
//! the played-status icon instead of life/mana, and the
//! difficulty/size/created panel row instead of life/mana + difficulty +
//! playtime.
//!
//! Plan deviations carried here: no cloud button (`SocialAPI.Cloud == null`,
//! plan §J.2 — `Rename` takes its `Left` 52) and no seed button
//! (`WorldGeneratorVersion == 0` for every `.trwld` the port writes — the
//! field does not exist on `save_list::WorldEntry`, plan Task 4's card table).
//! The `IconCompletion`/`IconMixedSeed` overlays stay off (no Moon Lord /
//! mixed-seed data); their geometry is pinned by [`overlay_rects`].
//!
//! Same shape as `ui/char_select.rs`: a pure [`layout`], one [`card_layout`]
//! behind both the hit-tests and the draw (Review Focus 2), and a [`draw`]
//! that replays the source's child order. Nothing panics with
//! `Lookups::default()` — every sheet is an `Option` guard (Review Focus 7).
//!
//! Every pub item here is live since Task 5: the App-flow surface drives
//! `App.front = FrontScreen::WorldSelect` (`main.rs` — `resolved_layout` +
//! `hit` for the clicks, `draw` for the frame, `sort_worlds` for the re-list).

use glam::Vec2;
use tr_render::SpriteRenderer;

use super::cards::{button_sheet, draw_card_button, draw_inner_panel, fat_button_style, header_style};
use super::char_select::{
    button_label, list_content_height, resolved_layout, scrollbar_attaches, Layout,
    BUTTON_DELETE, BUTTON_FAVORITE, BUTTON_PLAY, BUTTON_RENAME, CARD_BG_REST, CARD_BORDER_HOVER,
    CARD_BORDER_REST, CARD_H, CARD_PADDING, CARD_PITCH, CONTAINER_PANEL_BG, GRAY,
};
use super::color;
use super::hover_pair;
use super::in_rect;
use super::panel;
use super::push_whole;
use super::save_list;
use super::text::{border_string, hud_text_width, large_text_width};
use super::uistate::{self, Style};
use crate::assets;

// The T2-owned ordering, re-exported so Task 5 has one import site (the plan's
// "sort_worlds delegation" — the chain lives in save_list.rs, this module does
// not duplicate it). The App's re-list and the favourite toggle call it
// through this path.
pub use super::save_list::sort_worlds;

// --- Source strings (Localization `UI.*`, English) -----------------------------

/// `UI.SelectWorld` — the header text (`UIWorldSelect.cs:62`).
pub fn select_world() -> String {
    crate::lang::text("UI.SelectWorld")
}

/// `UI.WorldSizeFormat` — `"{0} World"` (`UIWorldListItem.cs:522`), filled with
/// `WorldSizeName` (`WorldFileData.cs:336-342` = `UI.WorldSizeSmall/Medium/
/// Large`, "Small"/"Medium"/"Large"; the port's `size_id` buckets it,
/// `save_list::size_bucket`). The three size labels resolve `Lang.menu[92..94]`
/// (`UIWorldCreation.cs:368-370` sets the same names).
pub fn size_name(size_id: u8) -> String {
    crate::lang::menu(92 + size_id.min(2) as usize)
}

// --- Card metrics ---------------------------------------------------------------

// The card button sheet order (`UIWorldListItem.cs:95-158` maps onto
// `Lookups::card_buttons` = `[ButtonPlay, ButtonFavoriteActive,
// ButtonFavoriteInactive, ButtonRename, ButtonDelete]`, assets.rs:956-963):
// the ctor ordinals key [`button_label`]'s table, `ui::cards::button_sheet`
// maps them to sheets — the favourite owns two sheets, so every button after
// it sits one later (the Task 3b review fix 1 pattern).

// --- Colours -------------------------------------------------------------------

/// `SetColorsToHovered`'s not-joinable border, `new Color(150, 150, 150) * 1f`
/// (`UIWorldListItem.cs:238`).
pub const CARD_BORDER_HOVER_NOT_JOINABLE: [u8; 4] = [150, 150, 150, 255];
/// `SetColorsToHovered`'s not-joinable background,
/// `Color.Lerp(BackgroundColor, new Color(120, 120, 120), 0.5f) * 1f`
/// (`:239`) over the `:234` `(73, 94, 171)` — XNA byte truncation:
/// `(96.5, 107, 145.5)` -> `(96, 107, 145)`.
pub const CARD_BG_HOVER_NOT_JOINABLE: [u8; 4] = [96, 107, 145, 255];
/// `SetColorsToNotHovered`'s not-joinable border, `new Color(127, 127, 127) *
/// 0.7f` (`UIWorldListItem.cs:249`); XNA `(byte)(127*0.7) = 88`, alpha 178.
pub const CARD_BORDER_REST_NOT_JOINABLE: [u8; 4] = [88, 88, 88, 178];
/// `SetColorsToNotHovered`'s not-joinable background,
/// `Color.Lerp(new Color(63, 82, 151), new Color(80, 80, 80), 0.5f) * 0.7f`
/// (`:250`) — the lerp runs on the *unmultiplied* colour first: `(71, 81,
/// 115)`, then `*0.7` truncates to `(49, 56, 80, 178)`.
pub const CARD_BG_REST_NOT_JOINABLE: [u8; 4] = [49, 56, 80, 178];

/// `UIWorldListItem` hover colours (`MouseOver`->`SetColorsToHovered`
/// `:232-241`, `MouseOut`->`SetColorsToNotHovered` `:243-252`): the card
/// background/border pair as pre-tint bytes. A non-joinable world greys both
/// states — border `(150,150,150)`/`(127,127,127)`, the background lerped
/// toward `(120,120,120)`/`(80,80,80)` before the `*0.7`. Both colours pick
/// through `ui::hover_pair`, per joinability.
pub fn card_colors(hovered: bool, joinable: bool) -> ([u8; 4], [u8; 4]) {
    if joinable {
        (
            hover_pair(CARD_BG_REST, uistate::FADED_MOUSE_OVER_BG, hovered), // `:245-246` / `:234-235`
            hover_pair(CARD_BORDER_REST, CARD_BORDER_HOVER, hovered),
        )
    } else {
        (
            // `:250` / `:239` — the not-joinable grey pair.
            hover_pair(
                CARD_BG_REST_NOT_JOINABLE,
                CARD_BG_HOVER_NOT_JOINABLE,
                hovered,
            ),
            hover_pair(
                CARD_BORDER_REST_NOT_JOINABLE,
                CARD_BORDER_HOVER_NOT_JOINABLE,
                hovered,
            ),
        )
    }
}

/// `UIWorldSelect.CanWorldBeJoinedByActivePlayer` (`UIWorldSelect.cs:177-180`):
/// `Main.ActivePlayerFileData.Player.difficulty == 3 == (file.GameMode == 3)` —
/// C# parses it as `(active == 3) == (world == 3)`, an equality of the two
/// "is Journey" flags: both Journey or both not. (Only the Journey mismatch is
/// ever a rejection — `UIWorldListItem.TryMovingToRejectionMenuIfNeeded`
/// `:387-413` — so a Classic player "can join" an Expert world by this
/// predicate; it drives the sort order and the card grey-out, nothing else.)
pub fn joinable(active_difficulty: u8, world_difficulty: u8) -> bool {
    (active_difficulty == 3) == (world_difficulty == 3)
}

/// `AWorldListItem.GetDifficulty` (`AWorldListItem.cs:43-85`): the GameMode
/// map with the `ForTheWorthy` bump ported for shape — the port stores no FTW
/// flag, so the `num++` at `:63-66` is always `+0` and `num` 4
/// (`UI.Legendary`, `:80-83`) stays unreachable. Returns the `UI.*` label and
/// its colour: GameMode 3 -> `UI.Creative` + `Main.creativeModeColor` (`:47-52`);
/// else `num` 2 -> `UI.Expert` + `Main.mcColor` (`:72-75`), `num` 3 ->
/// `UI.Master` + `Main.hcColor` (`:76-79`), the `default`/`num` 1 arm ->
/// `UI.Normal` "Classic" + `Color.White` (`:69-71`).
pub fn difficulty_text(game_mode: u8) -> (String, [u8; 4]) {
    if game_mode == 3 {
        // `:47-52` — `UI.Creative`, checked before the switch.
        return (crate::lang::text("UI.Creative"), uistate::CREATIVE_MODE_COLOR);
    }
    let mut num = 1usize;
    match game_mode {
        1 => num = 2, // `:56-58`
        2 => num = 3, // `:59-61`
        _ => {}
    }
    // `if (this._data.ForTheWorthy) num++;` (`:63-66`) — always +0 in the port
    // (no FTW field on `save_list::WorldEntry`), so `num == 4` (Legendary)
    // never fires.
    match num {
        2 => (crate::lang::text("UI.Expert"), uistate::MC_COLOR), // `:72-75`
        3 => (crate::lang::text("UI.Master"), uistate::HC_COLOR), // `:76-79`
        _ => (crate::lang::text("UI.Normal"), [255, 255, 255, 255]), // `UI.Normal` (`:69-71`)
    }
}

// --- Screen layout ---------------------------------------------------------------

/// The whole screen (`UIWorldSelect.OnInitialize` `:38-88`). The ctor is the
/// `UICharacterSelect` twin setter-for-setter (Width 0.8/MaxWidth 650 root
/// `:41-45`, `(-110, 1)` panel `:47-48` with the same `(33, 43, 79) * 0.8f`
/// background `:49`, list + ListPadding 5 `:53-55`, scrollbar `:57-60`,
/// `UI.SelectWorld` header `:62-66`, Back `:68-78`, New `:79-86`), so the
/// rect table *is* Task 3's — only `header_w` changes, because the caller now
/// measures "Select World" instead of "Select Player" (`UIWorldSelect.cs:62`).
/// `list_w` carries the `Recalculate` branch (`:90-108`): 626 detached /
/// 601 attached, [`attach_scrollbar`].
#[allow(dead_code)] // pins tested only; consumed by Task 8 (the App routes
// the world clicks through `char_select::resolved_layout`, the twin body).
pub fn layout(screen: Vec2, header_w: f32, list_w: f32) -> Layout {
    super::char_select::layout(screen, header_w, list_w)
}

/// One world card's geometry (Review Focus 2's single layout function).
/// `row_rect` is the card list's rect and `index` the row: card i sits at
/// `(list.x, list.y + i*101, list.w, 96)` — the 96 px card
/// (`UIWorldListItem.cs:226` `Height.Set(96f, 0f)`) plus one 5 px
/// `ListPadding` per row (`UIWorldSelect.cs:55`). The plan's
/// `card_layout(card, has_been_played)` grew the `row_rect`/`index` front half
/// in the Task 3 review (one layout function feeds both the hit-tests and the
/// draw), keeping `has_been_played` last: it gates [`CardLayout::played_icon`],
/// the card's only status icon.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CardLayout {
    /// The card `UIPanel` itself — `Width (0, 1)`, `Height 96`, `SetPadding(6)`
    /// (`:226-228`).
    pub card: [f32; 4],
    /// The `UICyclingImage(GetIcons())` icon — `Left (4, 0)`
    /// (`AWorldListItem.cs:181-184`), texture-sized 60x58 (`UIImage.SetImage`
    /// resizes to the texture, `UIImage.cs` ctor; `IconCrimson` is the port's
    /// fixed icon — no evil/hardmode data, plan Task 9's deviation).
    pub icon: [f32; 4],
    /// `ButtonPlay` 22x22, `Left 4`, `VAlign 1` (`:94-103`).
    pub play: [f32; 4],
    /// `ButtonFavorite*` 22x22, `Left 28` (`:103-112`).
    pub favorite: [f32; 4],
    /// `ButtonRename` 22x22, `Left 52` — the cloud (`:113-124`) and seed
    /// (`:125-136`) slots are skipped, so rename takes `num` 52.
    pub rename: [f32; 4],
    /// `ButtonDelete` 22x22, `HAlign 1` + `VAlign 1` (`:146-158`).
    pub delete: [f32; 4],
    /// The hover label, `Left num` where the ctor's `num` walk is
    /// 4 (`:94`) + 24 (`:103`) + 24 (`:112`) + 24 (`:145`) + 4 (`:159`) = 80 —
    /// every slot's `num += 24` fires even though the cloud/seed buttons
    /// themselves are skipped — `Top -3`, `VAlign 1` (`:160-164`); the empty
    /// `UIText` is 0 wide and 16 tall (`UIText.cs:208-212`).
    pub label: [f32; 4],
    /// The delete hover label, `Left -30`, `HAlign 1`, `Top -3`, `VAlign 1`
    /// (`:165-170`) — its right edge is the anchor; the text grows leftward.
    pub delete_label: [f32; 4],
    /// The `IconPlayedBefore` status icon, drawn only when the active player
    /// has played the world (`:171-186`). The element is the texture-sized
    /// 25x28 at `HAlign 1, Left 0, Top -6` (`:176-178`); `ImageScale = 0.75f`
    /// (`:179`) shrinks the *draw* around the element box
    /// (`UIImage.DrawSelf`: position `+ size*(1-0.75)/2`, scale 0.75), so this
    /// stores the drawn 18.75x21 sprite rect.
    pub played_icon: Option<[f32; 4]>,
    /// `DrawBorderString` name position: `num + 6, dimensions.Y - 2` where
    /// `num` is the icon's right edge and `dimensions` the icon element
    /// (`:498-507`).
    pub name_pos: [f32; 2],
    /// The `Divider` strip at `(num, inner.Y + 21)`, stretched to the card's
    /// **outer** right edge (`:508` — `GetDimensions()`, not the inner one),
    /// the 8x4 texture scaled by `width / 8`.
    pub divider: [f32; 4],
    /// The difficulty panel: `num + 6, inner.Y + 29`, `num2 = 100` px
    /// (`:509-511`).
    pub difficulty_panel: [f32; 4],
    /// The size panel: `vector.X += num2 + 5f` (`:515`), `num3 = 150` px
    /// (English; the `+40f` arm at `:517-520` is the non-English culture)
    /// (`:515-521`).
    pub size_panel: [f32; 4],
    /// The created panel: the rest of the inner width —
    /// `num4 = inner right - vector.X` (`:526-527`).
    pub created_panel: [f32; 4],
}

/// Row `index`'s card geometry. See [`CardLayout`] for the field-by-field cites.
pub fn card_layout(row_rect: [f32; 4], index: usize, has_been_played: bool) -> CardLayout {
    let shift = index as f32 * CARD_PITCH; // 96 px card + 5 px ListPadding
    let card = [row_rect[0], row_rect[1] + shift, row_rect[2], CARD_H]; // `:226-227`
    let inner = uistate::inner(card, CARD_PADDING); // `SetPadding(6)` (`:228`)
    let (ix, iy, iw) = (inner[0], inner[1], inner[2]);
    let bottom = iy + inner[3]; // the `VAlign 1` anchor

    // `GetIconElement()`'s `Left = new StyleDimension(4f, 0f)`
    // (`AWorldListItem.cs:183`); the 60x58 `IconCrimson` texture size, `Top`
    // unset (0).
    let icon = [ix + 4.0, iy, 60.0, 58.0];
    let icon_right = icon[0] + icon[2]; // DrawSelf's `num` (`:499`)

    // The ctor's `num` walk (`:94-159`): 4 (play), +24 (favorite at 28),
    // +24 (rename at 52; the cloud slot's `:123` and the seed slot's `:135`
    // `num += 24f` are skipped with their buttons), +24 (`:145`, the slot
    // after rename), +4 (`:159`) -> the label at 80.
    let num = 4.0 + 24.0 + 24.0 + 24.0 + 4.0;
    // `UIText` is 16 tall at scale 1 and 0 wide while empty (`UIText.cs:208-212`).
    let label_h = 16.0;
    let label_y = bottom - label_h - 3.0; // `Top.Set(-3f)` + `VAlign 1`

    // The played icon element (`:172-186`): the texture-sized 25x28
    // (`IconPlayedBefore`, decoded), `HAlign 1`, `Left (0, 0)`,
    // `Top (-6, 0)`: x = inner right - 25, y = inner top - 6. `ImageScale
    // 0.75f` (`:179`) shrinks the draw to 18.75x21 around the element centre
    // (`UIImage.DrawSelf`: `dimensions.Position() + size*(1-ImageScale)/2`,
    // origin zero), which is the rect stored here.
    let played_icon = if has_been_played {
        let (ex, ey) = (ix + iw - 25.0, iy - 6.0);
        Some([
            ex + 25.0 * 0.125,
            ey + 28.0 * 0.125,
            25.0 * 0.75,
            28.0 * 0.75,
        ])
    } else {
        None
    };

    CardLayout {
        card,
        icon,
        // The 22x22 image buttons, `VAlign 1` (`:96/:105/:138/:148`).
        play: [ix + 4.0, bottom - 22.0, 22.0, 22.0],
        favorite: [ix + 28.0, bottom - 22.0, 22.0, 22.0],
        rename: [ix + 52.0, bottom - 22.0, 22.0, 22.0],
        // `HAlign = 1f` + `VAlign = 1f` (`:148-149`).
        delete: [ix + iw - 22.0, bottom - 22.0, 22.0, 22.0],
        label: [ix + num, label_y, 0.0, label_h],
        // `Left.Set(-30f)` + `HAlign 1` (`:166-168`) — the right edge anchors.
        delete_label: [ix + iw - 30.0, label_y, 0.0, label_h],
        played_icon,
        // `new Vector2(num + 6f, dimensions.Y - 2f)` (`:507`).
        name_pos: [icon_right + 6.0, icon[1] - 2.0],
        // `:508` — y = `innerDimensions.Y + 21f`, width to the card's OUTER
        // right (`GetDimensions()`), the 8x4 `Divider` scaled by `dx / 8f`.
        divider: [
            icon_right,
            iy + 21.0,
            card[0] + card[2] - icon_right,
            4.0,
        ],
        // `new Vector2(num + 6f, innerDimensions.Y + 29f)`, `num2 = 100f`
        // (`:509-510`).
        difficulty_panel: [icon_right + 6.0, iy + 29.0, 100.0, 27.0],
        // `vector.X += num2 + 5f`, `num3 = 150f` (English; RU would be +40,
        // `:517-520`).
        size_panel: [icon_right + 6.0 + 105.0, iy + 29.0, 150.0, 27.0],
        // `vector.X += num3 + 5f`, `num4 = inner right - vector.X` (`:525-526`).
        created_panel: [
            icon_right + 6.0 + 105.0 + 155.0,
            iy + 29.0,
            ix + iw - (icon_right + 6.0 + 105.0 + 155.0),
            27.0,
        ],
    }
}

/// The status icon overlays' rects — `IconCompletion` when the world has
/// defeated the Moon Lord (`UIWorldListItem.cs:71-82`: `HAlign 0.5, VAlign
/// 0.5, Top (-10, 0), Left (-3, 0)`, the 64x70 texture), `IconMixedSeed` for
/// the multi-seed worlds (`:83-93`: `HAlign 0, VAlign 1, Top/Left (0, 0)`,
/// 22x18) — both children of the [`CardLayout::icon`] element — and
/// `IconNewlyGenerated` for a never-played new world (`:186-199`: `HAlign 1,
/// Left = num2 - 2`, `Top -6`, the 24x28 texture at `ImageScale 0.75`), a
/// sibling of the played icon with the played draw's `num2 -= 24` ahead of it
/// (`:187-200`; the played twin is `:173-185`).
///
/// **`draw` never draws them**: `save_list::WorldEntry` carries no Moon Lord /
/// seed-flag data (the newly-generated pin is Task 8's), so the overlays stay
/// unconditionally off (plan Task 4 step 3's documented deviation). This pins
/// the geometry for the record — including the `MaxWidth`/`MaxHeight =
/// StyleDimension.Fill` clamp (`UIElement.cs:33-34`, `StyleDimension.cs:44-46`)
/// that squeezes the completion overlay's *element box* to the 60x58 icon
/// inner, while `UIImage.DrawSelf` still draws the texture at its own size
/// from that box's position.
#[allow(dead_code)] // pins tested only; consumed by Task 8 (the overlays
// stay unconditionally off until the world creator's data lands).
pub fn overlay_rects(icon: [f32; 4]) -> [[f32; 4]; 3] {
    // The element box: 64x70 clamped to the 60x58 icon inner by the Fill
    // maxima. Resolved with the pre-clamped size so the alignment matches the
    // C# (`x += parentW*HAlign - W*HAlign` on the clamped W/H).
    let completion = uistate::resolve(
        icon,
        Style::px(-3.0),  // `:78`
        Style::px(-10.0), // `:77`
        Style::px(60.0),  // 64 clamped by `MaxWidth = Fill` (60-wide parent)
        Style::px(58.0),  // 70 clamped by `MaxHeight = Fill` (58-tall parent)
        0.5,              // `HAlign = 0.5f` (`:75`)
        0.5,              // `VAlign = 0.5f` (`:76`)
        0.0,
        0.0,
        icon[2],
        [0.0; 4],
    );
    let mixed = uistate::resolve(
        icon,
        Style::px(0.0), // `Left = FromPixels(0f)` (`:90`)
        Style::px(0.0), // `Top = FromPixels(0f)` (`:89`)
        Style::px(22.0),
        Style::px(18.0),
        0.0, // `HAlign = 0f` (`:87`)
        1.0, // `VAlign = 1f` (`:88`)
        0.0,
        0.0,
        icon[2],
        [0.0; 4],
    );
    // `UIImage.DrawSelf` draws the texture at its own size (`ImageScale` 1,
    // origin zero) from the element box's position. The newly-generated twin
    // (Task 8's pin) resolves the same `UIImage.DrawSelf` shrink: element
    // `HAlign 1, Left = num2 - 2` with the played icon's `num2 -= 24` ahead
    // (`:187-200` — both icons, the plan's correction), so x =
    // `icon right - 24 - 26`, and the 24x28 texture draws 18x21 from the
    // shrunk-around-centre origin, `Top -6`.
    [
        [completion[0], completion[1], 64.0, 70.0],
        [mixed[0], mixed[1], 22.0, 18.0],
        [
            icon[0] + icon[2] - 50.0 + 24.0 * 0.125,
            icon[1] - 6.0 + 28.0 * 0.125,
            24.0 * 0.75,
            28.0 * 0.75,
        ],
    ]
}

// --- The created date ------------------------------------------------------------

/// The English month names of `DateTime.ToString("d MMMM yyyy")`
/// (`UIWorldListItem.cs:528`, the English-culture arm).
const MONTHS: [&str; 12] = [
    "January",
    "February",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "October",
    "November",
    "December",
];

/// The `.trwld`'s modified time in whole Unix seconds — the port's
/// `CreationTime` stand-in (`WorldFileData.Metadata`'s `CreationTime`; the
/// port writes no `FileMetadata`, plan Task 4's "Created: {mtime}" row).
/// `None` without a file (a fake entry) or an unreadable clock.
pub fn creation_time(path: Option<&std::path::Path>) -> Option<u64> {
    let modified = std::fs::metadata(path?).ok()?.modified().ok()?;
    modified
        .duration_since(std::time::UNIX_EPOCH)
        .ok()
        .map(|d| d.as_secs())
}

/// `UI.WorldCreatedFormat` "Created: {0}" filled with
/// `CreationTime.ToString("d MMMM yyyy")` (`:528`, English culture — the day
/// unpadded, the full month name). No time -> an empty panel text (the panel
/// itself still draws).
pub fn format_created(unix_secs: u64) -> String {
    // `86400` secs/day; the civil-from-days conversion (Hinnant) maps the day
    // number to `(year, month, day)` without a date library.
    let days = (unix_secs / 86_400) as i64;
    let (year, month, day) = civil_from_days(days);
    format!("{day} {} {year}", MONTHS[(month - 1) as usize])
}

/// Days since 1970-01-01 -> `(year, month, day)` (Howard Hinnant's
/// `civil_from_days`, valid over the whole `u64` seconds range).
fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097; // [0, 146096]
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365; // [0, 399]
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100); // [0, 365]
    let mp = (5 * doy + 2) / 153; // [0, 11]
    let d = doy - (153 * mp + 2) / 5 + 1; // [1, 31]
    let m = if mp < 10 { mp + 3 } else { mp - 9 }; // [1, 12]
    (y + (m <= 2) as i64, m as u32, d as u32)
}

// --- Hit-testing -----------------------------------------------------------------

/// What a click does (`UIWorldSelect.cs:110-122`, `UIWorldListItem.cs:346-464`
/// + the card handlers). `Play` reaches the App only for a `valid` entry
/// (`PlayGame`'s `base._data.IsValid` guard, `:362` — the same App-side gate
/// as the character card); `Rename` opens the virtual keyboard with
/// `Lang.menu[48]` "Enter World Name:" (`:415-425`); `Delete` routes to the
/// Task 5 confirm (`menuMode = 9`, `:354`); `Back` is
/// `menuMode = multiplayer ? 12 : 1` (`:121`) — the multiplayer target stays
/// with a future multiplayer-flow task (plan phase close-out).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CardAction {
    Play(usize),
    ToggleFavorite(usize),
    Rename(usize),
    Delete(usize),
    /// The favourited card's delete click: `:151-154` never *subscribes* the
    /// handler, so the click is dead — the hit-test surfaces that as
    /// `DeleteBlocked` instead of skipping the button (Review Focus 4), and
    /// the `Cannot Delete (Favorited)` label already explains why.
    DeleteBlocked,
    New,
    Back,
}

/// The screen's click table. `favorites` carries one flag per card — the
/// delete gate needs it (Review Focus 4), so it replaces the plan's bare
/// `cards: usize` (the count is `favorites.len()`). Returns `(hovered card,
/// action)`: a card body without a button is `(Some(i), None)`; `Back`/`New`
/// come back without a card. The caller passes a [`resolved_layout`] so the
/// attached-scrollbar card widths agree with the draw. Double-click play
/// (`:69`, `:99`) is skipped — single click only (documented in the plan).
///
/// Back/New are checked *before* the cards: `GetElementAt` walks the root's
/// children in reverse append order (`UIElement.cs:324-334`) — New (`:85`),
/// Back (`:77`), then the container with the list (`:56`) — so wherever the
/// rects overlap (the last card's rows run under the fat buttons once the
/// list scrolls, 6+ worlds) the click is theirs, not the card's. New and Back
/// are disjoint, so their relative order is cosmetic.
pub fn card_hit(l: &Layout, favorites: &[bool], mouse: Vec2) -> (Option<usize>, Option<CardAction>) {
    if in_rect(mouse, l.new) {
        return (None, Some(CardAction::New)); // `NewWorldClick` (`:110-116`)
    }
    if in_rect(mouse, l.back) {
        return (None, Some(CardAction::Back)); // `GoBackClick` (`:118-122`)
    }
    for i in 0..favorites.len() {
        // The `UIList`'s `OverflowHidden` clips rows past the list bottom, so
        // a click below it never reaches a card — the same gate the draw's
        // `break` uses (Task 3/4 review fix; a row straddling the edge stays
        // hit, like the draw that still paints it).
        let row_top = l.list[1] + i as f32 * CARD_PITCH;
        if row_top >= l.list[1] + l.list[3] {
            break;
        }
        let c = card_layout(l.list, i, false);
        if !in_rect(mouse, c.card) {
            continue;
        }
        // The buttons are the card's clickable children; the favourite-gated
        // delete returns `DeleteBlocked` rather than nothing (Review Focus 4).
        let action = if in_rect(mouse, c.play) {
            Some(CardAction::Play(i))
        } else if in_rect(mouse, c.favorite) {
            Some(CardAction::ToggleFavorite(i))
        } else if in_rect(mouse, c.rename) {
            Some(CardAction::Rename(i))
        } else if in_rect(mouse, c.delete) {
            Some(if favorites[i] {
                CardAction::DeleteBlocked
            } else {
                CardAction::Delete(i)
            })
        } else {
            None
        };
        return (Some(i), action);
    }
    (None, None)
}

/// The flat form of [`card_hit`]: just the action, if any.
pub fn hit(l: &Layout, favorites: &[bool], mouse: Vec2) -> Option<CardAction> {
    card_hit(l, favorites, mouse).1
}

// --- Draw ------------------------------------------------------------------------

// The `UITextPanel` styles (`UIWorldSelect.cs:62-66`, `:68-86`), the 3-piece
// `DrawPanel` info panel (`UIWorldListItem.cs:487-492`) and the image button
// are the `ui::cards` shared copies — the `UICharacterListItem` twins.

/// One card, in the source's z-order (`UIElement.cs:199-230`): the card's own
/// `DrawSelf` first (the `UIPanel` slice, then the name/divider/info-panel
/// extras of `:494-532`), then the appended children — the icon (`:70`), the
/// buttons (`:102/:111/:144/:158`), the labels (`:160-170`), the played icon
/// (`:174-186`).
fn draw_card(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    l: &Layout,
    index: usize,
    entry: &save_list::WorldEntry,
    active_difficulty: u8,
    mouse: Vec2,
) {
    let c = card_layout(l.list, index, entry.has_been_played);
    let hovered = in_rect(mouse, c.card); // `MouseOver`/`MouseOut` (`:475-485`)
    let joinable = joinable(active_difficulty, entry.difficulty);
    let (bg, border) = card_colors(hovered, joinable);
    // `UIPanel.DrawSelf` (`UIPanel.cs:68-76`): background, then border.
    panel::draw_panel(r, lookups, lookups.ui_panel_bg, c.card, color::tint(bg));
    panel::draw_panel(r, lookups, lookups.ui_panel_border, c.card, color::tint(border));

    // DrawSelf extras (`:494-532`).
    // The name: `DrawBorderString` at `(num + 6, dimensions.Y - 2)`; an invalid
    // world draws Gray (`:502-505`) — the port carries no `StatusID` name for
    // the prefix, so like the character card the Gray arm draws the bare name.
    let (name_tint, name) = if entry.valid {
        ([1.0; 4], entry.name.as_str())
    } else {
        (color::tint(GRAY), entry.name.as_str())
    };
    border_string(r, lookups, c.name_pos[0], c.name_pos[1], name_tint, name);
    // The `Divider` 8x4 stretched to the card's outer right (`:508`).
    if let Some(sheet) = lookups.ui_divider {
        push_whole(r, lookups, &sheet, c.divider, [1.0; 4]);
    }
    // Difficulty panel (`:509-514`), text centred at `+ (num2/2 - w/2, 3)`.
    draw_inner_panel(
        r,
        lookups,
        c.difficulty_panel[0],
        c.difficulty_panel[1],
        c.difficulty_panel[2],
    );
    let (diff_text, diff_color) = difficulty_text(entry.difficulty);
    let diff_w = hud_text_width(r, lookups, &diff_text, 1.0);
    border_string(
        r,
        lookups,
        c.difficulty_panel[0] + c.difficulty_panel[2] * 0.5 - diff_w * 0.5,
        c.difficulty_panel[1] + 3.0,
        color::tint(diff_color),
        &diff_text,
    );
    // Size panel (`:515-524`): `"{0} World"` with the `WorldSizeName`, White,
    // centred the same way.
    draw_inner_panel(r, lookups, c.size_panel[0], c.size_panel[1], c.size_panel[2]);
    let size_text =
        crate::lang::text_args("UI.WorldSizeFormat", &[size_name(entry.size_id)]);
    let size_w = hud_text_width(r, lookups, &size_text, 1.0);
    border_string(
        r,
        lookups,
        c.size_panel[0] + c.size_panel[2] * 0.5 - size_w * 0.5,
        c.size_panel[1] + 3.0,
        [1.0; 4],
        &size_text,
    );
    // Created panel (`:525-530`): `"Created: {mtime, d MMMM yyyy}"`, White —
    // the mtime stand-in; a disk-less entry draws an empty panel text.
    draw_inner_panel(
        r,
        lookups,
        c.created_panel[0],
        c.created_panel[1],
        c.created_panel[2],
    );
    let created =
        crate::lang::text_args("UI.WorldCreatedFormat", &[created_text(entry.path.as_deref())]);
    let created_w = hud_text_width(r, lookups, &created, 1.0);
    border_string(
        r,
        lookups,
        c.created_panel[0] + c.created_panel[2] * 0.5 - created_w * 0.5,
        c.created_panel[1] + 3.0,
        [1.0; 4],
        &created,
    );

    // Children, in append order (`:70/:102/:111/:144/:158/:160/:165/:174`).
    // The 60x58 world icon (`AWorldListItem.cs:181-184`): the fixed
    // `IconCrimson` sheet whole — `GetIcon()`'s seed-variant list needs evil /
    // hardmode data the port does not store, so no cycling (documented
    // deviation), and the `IconCompletion`/`IconMixedSeed` overlays stay off
    // (their geometry is pinned by [`overlay_rects`]).
    if let Some(sheet) = lookups.ui_world_icon {
        push_whole(r, lookups, &sheet, c.icon, [1.0; 4]);
    }
    // The buttons (`:94-158`), each `VAlign 1` except delete's `HAlign 1` —
    // sheets resolve through [`button_sheet`], never a raw ordinal.
    let play_hover = hovered && in_rect(mouse, c.play);
    let fav_hover = hovered && in_rect(mouse, c.favorite);
    let rename_hover = hovered && in_rect(mouse, c.rename);
    let delete_hover = hovered && in_rect(mouse, c.delete);
    draw_card_button(
        r,
        lookups,
        lookups.card_buttons[button_sheet(BUTTON_PLAY, entry.is_favorite)],
        c.play,
        play_hover,
        0.4,
    );
    // The favourite sheet flips with the flag, idle 0.8 when favourited
    // (`:104`, `:110`'s `SetVisibility(1f, IsFavorite ? 0.8f : 0.4f)`).
    draw_card_button(
        r,
        lookups,
        lookups.card_buttons[button_sheet(BUTTON_FAVORITE, entry.is_favorite)],
        c.favorite,
        fav_hover,
        if entry.is_favorite { 0.8 } else { 0.4 },
    );
    // Rename is ctor ordinal 2 but sheet 3 — the favourite's second sheet
    // shifts everything after it ([`button_sheet`], the Task 3b review fix 1).
    draw_card_button(
        r,
        lookups,
        lookups.card_buttons[button_sheet(BUTTON_RENAME, entry.is_favorite)],
        c.rename,
        rename_hover,
        0.4,
    );
    // The delete button draws (and labels) even while favourited — only its
    // click is dead (`:151-156`, Review Focus 4).
    draw_card_button(
        r,
        lookups,
        lookups.card_buttons[button_sheet(BUTTON_DELETE, entry.is_favorite)],
        c.delete,
        delete_hover,
        0.4,
    );

    // The hover labels (`:254-303`): the left label for play/favorite/rename,
    // the right-aligned one for delete. `UIText.DrawSelf` (`UIText.cs:136-166`)
    // shifts the text up `2*scale` from the element rect. The played icon's
    // own hover label (`:182-183`) is not drawn — its hit-test has no action
    // in the plan's click table.
    let hovered_button = if play_hover {
        Some(BUTTON_PLAY)
    } else if fav_hover {
        Some(BUTTON_FAVORITE)
    } else if rename_hover {
        Some(BUTTON_RENAME)
    } else if delete_hover {
        Some(BUTTON_DELETE)
    } else {
        None
    };
    if let Some(button) = hovered_button {
        let text = button_label(button, entry.is_favorite);
        if button == BUTTON_DELETE {
            // `Left -30, HAlign 1` (`:166-170`): the right edge anchors, the
            // text grows leftward.
            let w = hud_text_width(r, lookups, &text, 1.0);
            border_string(
                r,
                lookups,
                c.delete_label[0] - w,
                c.delete_label[1] - 2.0,
                [1.0; 4],
                &text,
            );
        } else {
            border_string(r, lookups, c.label[0], c.label[1] - 2.0, [1.0; 4], &text);
        }
    }

    // The played-status icon, the card's last child (`:171-186`); it carries
    // hover handlers in the source but no click.
    if let Some(rect) = c.played_icon {
        if let Some(sheet) = lookups.ui_icon_played_before {
            push_whole(r, lookups, &sheet, rect, [1.0; 4]);
        }
    }
}

/// `UIWorldListItem.cs:460-464` through [`creation_time`]: the panel text —
/// `"Created: "` + the `d MMMM yyyy` date, or an empty string without a file.
fn created_text(path: Option<&std::path::Path>) -> String {
    match creation_time(path) {
        Some(secs) => format_created(secs),
        None => String::new(),
    }
}

/// The whole screen in the source's z-order: the container panel's `UIPanel`
/// slice (`:46-50`), the cards (the list's children), the attached scrollbar,
/// then the root's header / Back / New. `header_w` is the measured
/// `Select World` width (Death font, unscaled) — the same input [`layout`]
/// took, so the header text centres on the rect the layout produced.
/// `active_difficulty` is `Main.ActivePlayerFileData.Player.difficulty`
/// (`UIWorldSelect.cs:179`): it greys every card the active player cannot
/// join.
pub fn draw(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    screen: Vec2,
    header_w: f32,
    worlds: &[save_list::WorldEntry],
    active_difficulty: u8,
    mouse: Vec2,
) {
    let l = resolved_layout(screen, header_w, worlds.len());
    // `_containerPanel`: bg `(33, 43, 79) * 0.8` (`:49`), border the `UIPanel`
    // default black (`UIPanel.cs:18`).
    panel::draw_panel(r, lookups, lookups.ui_panel_bg, l.panel, color::tint(CONTAINER_PANEL_BG));
    panel::draw_panel(r, lookups, lookups.ui_panel_border, l.panel, color::tint(uistate::TEXT_PANEL_BORDER));

    // The cards, in list order. Rows past the list bottom would be clipped by
    // the `UIList`'s `OverflowHidden`; the port has no scissor, so they are
    // skipped (a row straddling the edge still draws — the Back/New buttons
    // cover the small strip below the panel).
    let list_bottom = l.list[1] + l.list[3];
    for (i, entry) in worlds.iter().enumerate() {
        if l.list[1] + i as f32 * CARD_PITCH >= list_bottom {
            break;
        }
        draw_card(r, lookups, &l, i, entry, active_difficulty, mouse);
    }
    // The scrollbar draws only while attached (a `Recalculate`-appended panel
    // child, `:100-104`), the view over the content height
    // `UIList.UpdateScrollbar` fed it (`UIList.cs:130-136`).
    let content_h = list_content_height(worlds.len());
    if scrollbar_attaches(l.list[3], content_h) {
        panel::draw_scrollbar(r, lookups, l.scrollbar, l.list[3], content_h, false);
    }

    // The header never hovers — no `OnMouseOver` is subscribed (`:62-66`).
    uistate::draw_text_panel(
        r,
        lookups,
        l.header,
        &header_style(),
        &select_world(),
        header_w,
        false,
    );
    // Back/New: `FadedMouseOver`/`FadedMouseOut` (`:74-75`, `:82-83`) — the
    // hover pair `uistate::draw_text_panel` swaps, the idle pair the style
    // carries.
    uistate::draw_text_panel(
        r,
        lookups,
        l.back,
        &fat_button_style(),
        &super::char_select::back(),
        large_text_width(r, lookups, &super::char_select::back()),
        in_rect(mouse, l.back),
    );
    uistate::draw_text_panel(
        r,
        lookups,
        l.new,
        &fat_button_style(),
        &super::char_select::new_button(),
        large_text_width(r, lookups, &super::char_select::new_button()),
        in_rect(mouse, l.new),
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    // Test-only pins: the attach branch's two knobs (the draw routes through
    // `resolved_layout`, which folds them in).
    use super::super::char_select::{attach_scrollbar, detached_list_width};

    // Expected values hand-derived from the C# (`UIWorldSelect.cs:38-186`,
    // `UIWorldListItem.cs:59-532`, `AWorldListItem.cs:43-200`,
    // `UIImage.cs` DrawSelf, `UIText.cs:175-212`) — never from this code.
    const SCREEN: Vec2 = Vec2::new(1600.0, 900.0);
    const LIST: [f32; 4] = [487.0, 232.0, 626.0, 546.0];

    fn world(name: &str, difficulty: u8, favorite: bool, played: bool) -> save_list::WorldEntry {
        save_list::WorldEntry {
            name: name.into(),
            difficulty,
            size_id: 0,
            is_favorite: favorite,
            valid: true,
            has_been_played: played,
            path: None,
        }
    }

    #[test]
    fn layout_matches_the_twin_ctor_table_at_1600x900() {
        // The `UIWorldSelect.OnInitialize` setters are the `UICharacterSelect`
        // ones verbatim (`:40-87`), so the rect table is Task 3's:
        let l = layout(SCREEN, 100.0, 626.0);
        // Root: Width 0.8*1600 = 1280 -> MaxWidth 650, HAlign .5 -> x 475;
        // Height -220 + 900 = 680, Top 220 (`:41-45`).
        assert_eq!(l.root, [475.0, 220.0, 650.0, 680.0]);
        // Panel: Height -110 + 680 = 570 (`:47-48`).
        assert_eq!(l.panel, [475.0, 220.0, 650.0, 570.0]);
        // List: the panel inner (padding 12), 626 wide detached (`:53-54`).
        assert_eq!(l.list, [487.0, 232.0, 626.0, 546.0]);
        // Scrollbar attach rect: HAlign 1 of the panel inner, 20 wide (`:57-60`).
        assert_eq!(l.scrollbar, [1093.0, 232.0, 20.0, 546.0]);
        // Header: MinWidth-sized — 100*0.8 + 30 = 110, centred on 800; height
        // 32*0.8 + 30 = 55.6 (`:62-65`, `UITextPanel.cs:132-134`).
        assert_eq!(l.header, [745.0, 180.0, 110.0, 55.6]);
        // Back/New: Width 0.5*650 - 10 = 315, H 50, bottom of root - 45
        // (`:68-78` / `:79-86`).
        assert_eq!(l.back, [475.0, 805.0, 315.0, 50.0]);
        assert_eq!(l.new, [810.0, 805.0, 315.0, 50.0]);
        // The detached width is screen-derived: 650 - 24.
        assert_eq!(detached_list_width(SCREEN), 626.0);
        // The header width is the caller's measurement of "Select World"
        // (`:62`): a wider measurement moves the MinWidth size and the centre.
        let l = layout(SCREEN, 200.0, 626.0);
        assert_eq!(
            l.header,
            [475.0 + (650.0 - 190.0) * 0.5, 180.0, 190.0, 55.6]
        );
    }

    #[test]
    fn scrollbar_branch_attaches_at_six_worlds() {
        // The `Recalculate` branch (`:90-108`) is the `UICharacterSelect` twin
        // (Review Focus 3): ported and unit-tested on a synthetic tall list.
        let l = layout(SCREEN, 100.0, detached_list_width(SCREEN));
        assert_eq!(l.list[2], 626.0);
        assert_eq!(list_content_height(0), 0.0);
        assert_eq!(list_content_height(1), 96.0);
        assert_eq!(list_content_height(5), 505.0);
        assert_eq!(list_content_height(6), 606.0);
        // 5 cards: 505 < 546 — never attaches below 6 worlds.
        assert!(!scrollbar_attaches(546.0, list_content_height(5)));
        // 6 cards: 606 > 546 — attached.
        assert!(scrollbar_attaches(546.0, list_content_height(6)));
        // The plan's synthetic 700-tall list.
        assert!(scrollbar_attaches(546.0, 700.0));
        // Attached: `Width.Set(-25f, 1f)` -> 601 (`:104`), and the scrollbar
        // rect stands ready at the panel's right edge.
        let attached = attach_scrollbar(l, true);
        assert_eq!(attached.list, [487.0, 232.0, 601.0, 546.0]);
        assert_eq!(attached.scrollbar, [1093.0, 232.0, 20.0, 546.0]);
        // Detached: `Width.Set(0f, 1f)` -> 626 (`:98`).
        assert_eq!(attach_scrollbar(l, false).list[2], 626.0);
        assert_eq!(resolved_layout(SCREEN, 100.0, 5).list[2], 626.0);
        assert_eq!(resolved_layout(SCREEN, 100.0, 6).list[2], 601.0);
    }

    #[test]
    fn card_layout_matches_the_source_rows() {
        // `card_layout([487, 232, 626, 96], 0, true)` — the plan's oracle as
        // the index-0 call, played icon on.
        let c = card_layout([487.0, 232.0, 626.0, 96.0], 0, true);
        // The card + its 6 px inner (`:226-228`).
        assert_eq!(c.card, [487.0, 232.0, 626.0, 96.0]);
        assert_eq!(uistate::inner(c.card, CARD_PADDING), [493.0, 238.0, 614.0, 84.0]);
        // The 60x58 icon at Left 4 (`AWorldListItem.cs:183` + the texture size).
        assert_eq!(c.icon, [497.0, 238.0, 60.0, 58.0]);
        // The buttons: VAlign 1 -> bottom of the inner 322 - 22; pitch 24 with
        // the cloud (`:113-124`) and seed (`:125-136`) slots skipped so rename
        // takes 52 (`:94-144`); delete HAlign 1 (`:146-158`).
        assert_eq!(c.play, [497.0, 300.0, 22.0, 22.0]);
        assert_eq!(c.favorite, [521.0, 300.0, 22.0, 22.0]);
        assert_eq!(c.rename, [545.0, 300.0, 22.0, 22.0]);
        assert_eq!(c.delete, [1085.0, 300.0, 22.0, 22.0]);
        // The labels: Left 80 (4+24+24+24+4, `:94-159`), Top -3 + VAlign 1 with
        // the 16-tall empty `UIText` (`:160-164`, `UIText.cs:208-212`); the
        // delete label's right edge anchors 30 px in (`:165-170`).
        assert_eq!(c.label, [573.0, 303.0, 0.0, 16.0]);
        assert_eq!(c.delete_label, [1077.0, 303.0, 0.0, 16.0]);
        // The played icon: element `HAlign 1, Left 0, Top -6` at the texture
        // size 25x28 -> (1082, 232, 25, 28); `ImageScale 0.75` (`:176-179`)
        // shrinks the draw around the element centre (`UIImage.DrawSelf`'s
        // `+ size*(1-0.75)/2`, scale 0.75) -> the stored sprite rect.
        assert_eq!(c.played_icon, Some([1085.125, 235.5, 18.75, 21.0]));
        // DrawSelf extras (`:494-532`).
        assert_eq!(c.name_pos, [563.0, 236.0]); // icon right 557 + 6 / 238 - 2
        // The divider stretches to the CARD's outer right (`:508`'s
        // `base.GetDimensions()` — the card is `base`, 487 + 626 = 1113):
        // 1113 - 557 = 556, the Task 3 reading (`base` is the card, not the
        // container panel).
        assert_eq!(c.divider, [557.0, 259.0, 556.0, 4.0]);
        assert_eq!(c.difficulty_panel, [563.0, 267.0, 100.0, 27.0]); // `num2 = 100`
        assert_eq!(c.size_panel, [668.0, 267.0, 150.0, 27.0]); // + 105 (`:515`)
        assert_eq!(c.created_panel, [823.0, 267.0, 284.0, 27.0]); // inner right 1107 - 823
        // Without the played flag the icon is absent (`:171-186`).
        let c = card_layout([487.0, 232.0, 626.0, 96.0], 0, false);
        assert_eq!(c.played_icon, None);
        // ...and every other rect is unchanged by the flag.
        assert_eq!(c.icon, [497.0, 238.0, 60.0, 58.0]);
        assert_eq!(c.created_panel, [823.0, 267.0, 284.0, 27.0]);
    }

    #[test]
    fn card_layout_row_3_shifts_every_rect() {
        let a = card_layout(LIST, 0, true);
        let b = card_layout(LIST, 3, true);
        let dy = 3.0 * CARD_PITCH; // 3*101 = 303
        assert_eq!(b.card, [a.card[0], a.card[1] + dy, a.card[2], a.card[3]]);
        assert_eq!(b.icon, [a.icon[0], a.icon[1] + dy, 60.0, 58.0]);
        assert_eq!(b.play, [a.play[0], a.play[1] + dy, 22.0, 22.0]);
        assert_eq!(b.favorite, [a.favorite[0], a.favorite[1] + dy, 22.0, 22.0]);
        assert_eq!(b.rename, [a.rename[0], a.rename[1] + dy, 22.0, 22.0]);
        assert_eq!(b.delete, [a.delete[0], a.delete[1] + dy, 22.0, 22.0]);
        assert_eq!(b.label, [a.label[0], a.label[1] + dy, 0.0, 16.0]);
        assert_eq!(b.delete_label, [a.delete_label[0], a.delete_label[1] + dy, 0.0, 16.0]);
        assert_eq!(
            b.played_icon,
            Some([
                a.played_icon.unwrap()[0],
                a.played_icon.unwrap()[1] + dy,
                18.75,
                21.0
            ])
        );
        assert_eq!(b.name_pos, [a.name_pos[0], a.name_pos[1] + dy]);
        assert_eq!(b.divider, [a.divider[0], a.divider[1] + dy, a.divider[2], 4.0]);
        assert_eq!(
            b.difficulty_panel,
            [a.difficulty_panel[0], a.difficulty_panel[1] + dy, 100.0, 27.0]
        );
        assert_eq!(b.size_panel, [a.size_panel[0], a.size_panel[1] + dy, 150.0, 27.0]);
        assert_eq!(
            b.created_panel,
            [a.created_panel[0], a.created_panel[1] + dy, a.created_panel[2], 27.0]
        );
        // Absolute anchor: row 3's card top is 232 + 303.
        assert_eq!(b.card, [487.0, 535.0, 626.0, 96.0]);
    }

    #[test]
    fn card_layout_follows_the_attached_list_width() {
        // Review Focus 3: the scrollbar changes the LIST width, and the card
        // widths (so the HAlign-1 delete, the played icon and the divider)
        // follow it.
        let l = resolved_layout(SCREEN, 100.0, 6); // attached
        assert_eq!(l.list[2], 601.0);
        let c = card_layout(l.list, 0, true);
        assert_eq!(c.card, [487.0, 232.0, 601.0, 96.0]);
        assert_eq!(c.delete, [487.0 + 601.0 - 6.0 - 22.0, 300.0, 22.0, 22.0]);
        assert_eq!(c.divider[2], 487.0 + 601.0 - 557.0);
        // The played icon's element rides the inner right too:
        // 493 + 589 - 25 = 1057, +3.125 -> the sprite x.
        assert_eq!(c.played_icon, Some([1060.125, 235.5, 18.75, 21.0]));
    }

    #[test]
    fn overlay_rects_pin_the_not_drawn_icon_overlays() {
        // `UIWorldListItem.cs:71-93` + `:187-200`, children/siblings of the
        // icon element (497, 238, 60, 58). All stay undrawn (no Moon Lord /
        // mixed-seed / newly-generated data) — this pins the geometry for the
        // record.
        let o = overlay_rects([497.0, 238.0, 60.0, 58.0]);
        // IconCompletion: `HAlign/VAlign 0.5, Top -10, Left -3` (`:75-78`) —
        // the element box clamps to the 60x58 parent (`MaxWidth`/`MaxHeight =
        // Fill`, `UIElement.cs:33-34`), so x = 497 - 3 + 0.5*(60-60) = 494 and
        // y = 238 - 10 + 0.5*(58-58) = 228; `UIImage.DrawSelf` still draws the
        // 64x70 texture from there.
        assert_eq!(o[0], [494.0, 228.0, 64.0, 70.0]);
        // IconMixedSeed: `HAlign 0, VAlign 1, Top/Left 0` (`:87-90`) —
        // (497, 238 + 58 - 18), no clamp (22x18 fits).
        assert_eq!(o[1], [497.0, 278.0, 22.0, 18.0]);
        // IconNewlyGenerated (`:187-200`): `HAlign 1, Left = num2 - 2` with
        // the played icon's `num2 -= 24` ahead (`:185` — both icons, the
        // plan's correction), `Top -6`, `ImageScale 0.75` — the element box
        // is x = 557 - 24 - 26 = 507, y = 232; the 24x28 texture draws 18x21
        // from the shrunk-around-centre origin (24*0.125 / 28*0.125 in).
        assert_eq!(o[2], [510.0, 235.5, 18.0, 21.0]);
    }

    #[test]
    fn difficulty_texts_follow_the_getdifficulty_map() {
        // `AWorldListItem.cs:43-85` + the `UI.*` strings ("Normal" is
        // "Classic", `Main.cs:66765-66768` for the colours).
        assert_eq!(difficulty_text(0), (crate::lang::text("UI.Normal"), [255, 255, 255, 255]));
        assert_eq!(difficulty_text(1), (crate::lang::text("UI.Expert"), uistate::MC_COLOR));
        assert_eq!(difficulty_text(2), (crate::lang::text("UI.Master"), uistate::HC_COLOR));
        assert_eq!(difficulty_text(3), (crate::lang::text("UI.Creative"), uistate::CREATIVE_MODE_COLOR));
        // GameMode 3 returns before the switch (`:47-52`).
        // The `default` arm catches every other GameMode (`:69-71`).
        assert_eq!(difficulty_text(9).0, crate::lang::text("UI.Normal"));
        // The FTW bump (`:63-66`) is always +0 in the port, so `num == 4`
        // (Legendary) is unreachable — nothing maps to `UI.Legendary`.
    }

    #[test]
    fn joinable_is_the_equality_of_the_two_journey_flags() {
        // `UIWorldSelect.cs:179`: `(active == 3) == (world == 3)`.
        assert!(joinable(0, 0));
        assert!(joinable(3, 3));
        assert!(!joinable(0, 3));
        assert!(!joinable(3, 0));
        // The surprising consequence of the equality: only the Journey
        // mismatch rejects (`UIWorldListItem.cs:387-413`), so a Classic
        // player "can join" an Expert/Master world by this predicate.
        assert!(joinable(0, 1));
        assert!(joinable(0, 2));
        assert!(joinable(1, 3) == false);
    }

    #[test]
    fn card_colors_match_the_xna_lerps() {
        // Rest, joinable (`:245-246`): the `(63,82,151)*0.7` /
        // `(89,116,213)*0.7` pair, XNA byte truncation.
        assert_eq!(card_colors(false, true), ([44, 57, 105, 178], [62, 81, 149, 178]));
        assert_eq!(card_colors(false, true).0, color::mul([63, 82, 151, 255], 0.7));
        assert_eq!(card_colors(false, true).1, color::mul([89, 116, 213, 255], 0.7));
        // Rest, NOT joinable (`:248-250`): `Lerp((63,82,151), (80,80,80), 0.5)`
        // on the *unmultiplied* colour -> (71, 81, 115), then `*0.7`.
        let lerp_half = |a: f32, b: f32| (a + (b - a) * 0.5f32) as u8;
        let lerped = [
            lerp_half(63.0, 80.0),
            lerp_half(82.0, 80.0),
            lerp_half(151.0, 80.0),
            255u8,
        ];
        assert_eq!(lerped, [71, 81, 115, 255]);
        assert_eq!(card_colors(false, false).0, color::mul(lerped, 0.7));
        assert_eq!(card_colors(false, false).0, [49, 56, 80, 178]);
        assert_eq!(card_colors(false, false).1, color::mul([127, 127, 127, 255], 0.7));
        assert_eq!(card_colors(false, false).1, [88, 88, 88, 178]);
        // Hover, joinable (`:234-235`).
        assert_eq!(card_colors(true, true), ([73, 94, 171, 255], [89, 116, 213, 255]));
        // Hover, NOT joinable (`:236-240`): border `(150,150,150)`, background
        // `Lerp((73,94,171), (120,120,120), 0.5)` = (96.5, 107, 145.5) ->
        // (96, 107, 145) truncated.
        assert_eq!(card_colors(true, false).1, [150, 150, 150, 255]);
        let hovered_grey = [
            lerp_half(73.0, 120.0),
            lerp_half(94.0, 120.0),
            lerp_half(171.0, 120.0),
            255u8,
        ];
        assert_eq!(hovered_grey, [96, 107, 145, 255]);
        assert_eq!(card_colors(true, false).0, hovered_grey);
    }

    #[test]
    fn sort_worlds_is_the_save_list_implementation() {
        // T2 owns the `UpdateWorldsList` ordering (`UIWorldSelect.cs:153-166`);
        // the re-export keeps one copy instead of duplicating the chain.
        let mut worlds = vec![
            world("a", 0, false, false),
            world("b", 3, false, false),
            world("c", 3, true, false),
            world("d", 0, false, true),
        ];
        // A Journey active player joins Journey worlds first (`:156-160`).
        sort_worlds(&mut worlds, true);
        let names: Vec<&str> = worlds.iter().map(|w| w.name.as_str()).collect();
        assert_eq!(names, vec!["c", "b", "d", "a"]);
        // A Classic active player joins non-Journey worlds first.
        sort_worlds(&mut worlds, false);
        let names: Vec<&str> = worlds.iter().map(|w| w.name.as_str()).collect();
        assert_eq!(names, vec!["d", "a", "c", "b"]);
    }

    #[test]
    fn button_labels_and_sheets_match_the_card() {
        // Labels (`:254-303` — the same `UI.*` table as the character card):
        assert_eq!(button_label(BUTTON_PLAY, false), "Play");
        assert_eq!(button_label(BUTTON_FAVORITE, false), "Favorite");
        assert_eq!(button_label(BUTTON_FAVORITE, true), "Unfavorite");
        assert_eq!(button_label(BUTTON_RENAME, false), "Rename");
        assert_eq!(button_label(BUTTON_DELETE, false), "Delete");
        assert_eq!(button_label(BUTTON_DELETE, true), "Cannot Delete (Favorited)");
        // Sheets: the ctor ordinals map onto the
        // `[Play, FavoriteActive, FavoriteInactive, Rename, Delete]` array —
        // the favourite's second sheet shifts Rename and Delete one later
        // (`BUTTON_RENAME` = 2, sheet 3).
        assert_eq!(button_sheet(BUTTON_PLAY, false), 0); // ButtonPlay
        assert_eq!(button_sheet(BUTTON_FAVORITE, true), 1); // ButtonFavoriteActive
        assert_eq!(button_sheet(BUTTON_FAVORITE, false), 2); // ButtonFavoriteInactive
        assert_eq!(button_sheet(BUTTON_RENAME, false), 3); // ButtonRename — not 2!
        assert_eq!(button_sheet(BUTTON_DELETE, false), 4); // ButtonDelete
    }

    #[test]
    fn card_hit_routes_the_button_centres() {
        let l = resolved_layout(SCREEN, 100.0, 2); // 2 cards -> detached, 626 wide
        let center = |rect: [f32; 4]| Vec2::new(rect[0] + rect[2] * 0.5, rect[1] + rect[3] * 0.5);
        let c0 = card_layout(l.list, 0, false);
        let c1 = card_layout(l.list, 1, false);
        let favorites = [false, true];
        assert_eq!(
            card_hit(&l, &favorites, center(c0.play)),
            (Some(0), Some(CardAction::Play(0)))
        );
        assert_eq!(
            card_hit(&l, &favorites, center(c0.favorite)),
            (Some(0), Some(CardAction::ToggleFavorite(0)))
        );
        assert_eq!(
            card_hit(&l, &favorites, center(c0.rename)),
            (Some(0), Some(CardAction::Rename(0)))
        );
        // Card 0 is not favourited -> the delete click routes to the confirm.
        assert_eq!(
            card_hit(&l, &favorites, center(c0.delete)),
            (Some(0), Some(CardAction::Delete(0)))
        );
        // Card 1 IS favourited -> `DeleteBlocked`, not `Delete` (Review Focus
        // 4: `:151-154` never subscribes the handler).
        assert_eq!(
            card_hit(&l, &favorites, center(c1.delete)),
            (Some(1), Some(CardAction::DeleteBlocked))
        );
        // A card body away from every button: hovered, but no action.
        let body = Vec2::new(c0.name_pos[0] + 30.0, c0.card[1] + 8.0);
        assert_eq!(card_hit(&l, &favorites, body), (Some(0), None));
        // The fat buttons.
        assert_eq!(card_hit(&l, &favorites, center(l.back)), (None, Some(CardAction::Back)));
        assert_eq!(card_hit(&l, &favorites, center(l.new)), (None, Some(CardAction::New)));
        // Off-screen: nothing.
        assert_eq!(card_hit(&l, &favorites, Vec2::new(10.0, 10.0)), (None, None));
        // The flat form.
        assert_eq!(hit(&l, &favorites, center(c0.play)), Some(CardAction::Play(0)));
        assert_eq!(hit(&l, &favorites, Vec2::new(10.0, 10.0)), None);
    }

    #[test]
    fn back_and_new_beat_the_cards_in_the_overlap_zone() {
        // `GetElementAt`'s reverse-append priority (`UIElement.cs:324-334`)
        // is observable: with 6 worlds the scrollbar attaches and the last
        // card's rows run under the fat buttons — card 5 spans y 737-833
        // while Back/New sit at y 805-855 — and a click at y >= 805 in the
        // overlap is Back's/New's, never the card's.
        let l = resolved_layout(SCREEN, 100.0, 6); // attached
        let c5 = card_layout(l.list, 5, false);
        assert_eq!(c5.card, [487.0, 737.0, 601.0, 96.0]); // y 737-833
        assert_eq!(l.back, [475.0, 805.0, 315.0, 50.0]); // y 805-855
        assert_eq!(l.new, [810.0, 805.0, 315.0, 50.0]);
        let favorites = [false; 6];
        // Card 5's rename button and Back share (550, 815) — Back wins.
        let p = Vec2::new(550.0, 815.0);
        assert!(in_rect(p, c5.rename));
        assert!(in_rect(p, l.back));
        assert_eq!(card_hit(&l, &favorites, p), (None, Some(CardAction::Back)));
        // Card 5's delete button and New share (1070, 815) — New wins.
        let p = Vec2::new(1070.0, 815.0);
        assert!(in_rect(p, c5.delete));
        assert!(in_rect(p, l.new));
        assert_eq!(card_hit(&l, &favorites, p), (None, Some(CardAction::New)));
        // Just above the fat buttons the card still owns the click (a
        // hovered body, no button).
        assert_eq!(card_hit(&l, &favorites, Vec2::new(550.0, 800.0)), (Some(5), None));
    }

    #[test]
    fn clicks_below_the_list_bottom_never_reach_a_card() {
        // The `OverflowHidden` break gate (Task 3/4 review fix): with 6
        // worlds the list scrolls, card 6 would start at y 838 — past the
        // list bottom 778 — and the draw never paints it, so the hit-test
        // must not answer for it either. (550, 860) is inside that phantom
        // row and below Back's hit box (y 805-855).
        let l = resolved_layout(SCREEN, 100.0, 6); // attached
        assert_eq!(l.list[1] + l.list[3], 778.0);
        assert_eq!(card_layout(l.list, 6, false).card[1], 838.0, "the phantom row's top");
        assert!(!in_rect(Vec2::new(550.0, 860.0), l.back), "below Back too");
        assert_eq!(card_hit(&l, &[false; 7], Vec2::new(550.0, 860.0)), (None, None));
        // Card 5 straddles the bottom (top 737 < 778) and stays hit — the
        // same rows the draw still paints.
        let favorites = [false; 6];
        assert_eq!(card_hit(&l, &favorites, Vec2::new(550.0, 777.0)), (Some(5), None));
    }

    #[test]
    fn created_text_formats_the_d_mmmm_yyyy_date() {
        // `UIWorldListItem.cs:528`'s English arm: `CreationTime.ToString(
        // "d MMMM yyyy")` — the day unpadded, the full month name.
        assert_eq!(format_created(0), "1 January 1970");
        // A leap day (2000-02-29) and a modern date (2026-10-31) pin the
        // calendar maths.
        assert_eq!(format_created(951_782_400), "29 February 2000");
        assert_eq!(format_created(1_793_404_800), "31 October 2026");
        // No time -> an empty panel text (the panel itself still draws).
        assert_eq!(created_text(None), "");
        // The draw-facing form reads the file's mtime stand-in — a fresh
        // file's date today, so pin the `d MMMM yyyy` shape (3 tokens).
        let dir = std::env::temp_dir().join("tr-worldsel-created");
        let _ = std::fs::create_dir_all(&dir);
        let file = dir.join("any.trwld");
        std::fs::write(&file, b"x").unwrap();
        let text = created_text(Some(&file));
        assert_eq!(text.split_whitespace().count(), 3, "{text}");
        std::fs::remove_file(&file).unwrap();
    }
}
