//! The shared pieces of the Workshop/ResourcePack screen cluster (W4-d84,
//! research/18 rows 1-6): the list-screen chrome the four workshop screens
//! share, the `UIResourcePack` card, the `UIIconTextButton` Publish button and
//! the `UIText` word-wrap the hub/info descriptions ride.
//!
//! Every geometry helper cites its C# block; the screens
//! (`workshop_hub.rs`, `workshop_publish.rs`, `resource_packs.rs`,
//! `resource_pack_info.rs`) build their layouts through these so the draw and
//! the click table cannot drift apart (Review Focus 2).

use glam::Vec2;
use tr_render::SpriteRenderer;

use crate::assets;
use crate::resource_pack::ResourcePack;
use crate::ui::text::{self, font_width};
use crate::ui::uistate::{self, Style};
use crate::ui::{color, panel};

/// `new Color(33, 43, 79) * 0.8f` — the cluster's container panel background
/// (`UIWorkshopHub.cs:62`, `UIResourcePackSelectionMenu.cs:302`, the three
/// workshop lists' `:48/:51/:54`); XNA byte truncation `(26, 34, 63, 204)`.
pub(crate) const CLUSTER_PANEL_BG: [u8; 4] = [26, 34, 63, 204];

/// `new Color(73, 94, 171)` — the cluster's title-panel background
/// (`UIWorkshopHub.cs:71`, `UIResourcePackSelectionMenu.cs:375`, the workshop
/// titles' `:68/:65/:71`).
pub(crate) const TITLE_BG: [u8; 4] = [73, 94, 171, 255];

/// The workshop list screens' shared root (`UIWorkshopSelectResourcePackToPublish.cs:42-47`,
/// `UIWorkshopSelectWorldToPublish.cs:39-44`, `UIWorkshopWorldImport.cs:45-50`):
/// `Width (0, 0.8)`, `MaxWidth 650`, `Top 220`, `Height (-220, 1)`, `HAlign 0.5`.
pub(crate) fn workshop_root(screen: Vec2) -> [f32; 4] {
    uistate::resolve(
        [0.0, 0.0, screen.x, screen.y],
        Style::px(0.0),
        Style::px(220.0),
        Style::new(0.0, 0.8),
        Style::new(-220.0, 1.0),
        0.5,
        0.0,
        0.0,
        0.0,
        650.0,
        [0.0; 4],
    )
}

/// The workshop list screens' container panel (`:48-51` etc.): `Width (0, 1)`,
/// `Height (-110, 1)` of the root, `UIPanel`'s 12 px padding.
pub(crate) fn workshop_panel(root: [f32; 4]) -> [f32; 4] {
    uistate::resolve(
        root,
        Style::px(0.0),
        Style::px(0.0),
        Style::new(0.0, 1.0),
        Style::new(-110.0, 1.0),
        0.0,
        0.0,
        0.0,
        0.0,
        root[2],
        [0.0; 4],
    )
}

/// The workshop list screens' title panel (`UIWorkshopSelectResourcePackToPublish.cs:64-68`
/// and twins): `UITextPanel` 0.8 large, `HAlign 0.5`, `Top -40`,
/// `SetPadding(15)`, bg `TITLE_BG` — sized only by `SetText`'s minimums
/// (`UITextPanel.cs:132-134`). `text_w` is the unscaled Death-font width.
pub(crate) fn workshop_title(root: [f32; 4], text_w: f32) -> [f32; 4] {
    uistate::resolve(
        root,
        Style::px(0.0),
        Style::px(-40.0),
        Style::px(0.0),
        Style::px(0.0),
        0.5,
        0.0,
        text_w * 0.8 + 30.0,
        32.0 * 0.8 + 30.0,
        root[2],
        [0.0; 4],
    )
}

/// The workshop list screens' centred Back panel (`:70-75` and twins):
/// `Width (-10, 0.5)`, `Height 50`, `VAlign 1`, `HAlign 0.5`, `Top -45`.
pub(crate) fn workshop_back(root: [f32; 4]) -> [f32; 4] {
    uistate::resolve(
        root,
        Style::px(0.0),
        Style::px(-45.0),
        Style::new(-10.0, 0.5),
        Style::px(50.0),
        0.5,
        1.0,
        0.0,
        0.0,
        root[2],
        [0.0; 4],
    )
}

/// The workshop list screens' scrollbar attach rect (`:59-62` and twins):
/// `UIScrollbar` `Width 20` (`UIScrollbar.cs:58-59`), `Height (0, 1)`,
/// `HAlign 1` of the panel inner.
pub(crate) fn workshop_scrollbar(panel_inner: [f32; 4]) -> [f32; 4] {
    uistate::resolve(
        panel_inner,
        Style::px(0.0),
        Style::px(0.0),
        Style::px(20.0),
        Style::new(0.0, 1.0),
        1.0,
        0.0,
        0.0,
        0.0,
        20.0,
        [0.0; 4],
    )
}

/// The workshop publish rows' `UIList` content height (`UIList.cs:120-128`):
/// the 82 px row (`UIWorkshopPublishResourcePackListItem.cs:97`) plus one
/// 5 px `ListPadding` per row (`:57`); a single row drops the padding.
pub(crate) fn publish_content_height(rows: usize) -> f32 {
    match rows {
        0 => 0.0,
        1 => 82.0,
        n => n as f32 * 87.0,
    }
}

/// The `UIResourcePack` card's content height: 102 px rows
/// (`UIResourcePack.cs:20`, `:46`) + the selection menu's 5 px `ListPadding`
/// (`UIResourcePackSelectionMenu.cs:339/348`).
pub(crate) fn pack_content_height(rows: usize) -> f32 {
    match rows {
        0 => 0.0,
        1 => 102.0,
        n => n as f32 * 107.0,
    }
}

/// `UIResourcePack`'s four static colours (`UIResourcePack.cs:135-141`), XNA
/// byte truncation applied: rest bg `(26,40,89)*0.8` = `(20,32,71,204)`, rest
/// border `(13,20,44)*0.8` = `(10,16,35,204)`, hover bg `(46,60,119)`, hover
/// border `(20,30,56)`.
pub(crate) const PACK_BG_REST: [u8; 4] = [20, 32, 71, 204];
pub(crate) const PACK_BORDER_REST: [u8; 4] = [10, 16, 35, 204];
pub(crate) const PACK_BG_HOVER: [u8; 4] = [46, 60, 119, 255];
pub(crate) const PACK_BORDER_HOVER: [u8; 4] = [20, 30, 56, 255];

/// One `UIResourcePack` card's geometry (`UIResourcePack.cs:40-96`): the
/// 102 px panel (`:46-48`), `SetPadding(5)` (`:51`), the icon block
/// (`DrawIcon`, `:109-114`) and the `ContentPanel` button strip
/// (`BuildChildren`, `:87-95`). `row_rect` is the scrolled list rect; `index`
/// picks the row at the 107 px pitch.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct PackCard {
    /// The card panel itself.
    pub card: [f32; 4],
    /// The 64x64 icon at `inner + (4, 14)` (`:112`).
    pub icon: [f32; 4],
    /// The 72x72 `Achievement_Borders` frame at `inner + (0, 10)` (`:113`).
    pub icon_border: [f32; 4],
    /// The name `UIText` position — `Left 77`, `Top 4` (`:59-65`).
    pub name_pos: [f32; 2],
    /// The author line position — `Top 4 + 16 + 6 = 26` (`:67-72`), scale 0.7.
    pub author_pos: [f32; 2],
    /// The divider strip — `Left 72`, `Top 26 + 16*0.7 + 10 = 47.2`,
    /// `Width (-80, 1)` (`:74-83`); the sheet's own height.
    pub divider: [f32; 4],
    /// The `ContentPanel` — `Left 77`, `Top 47.2 + divider_h + 5`,
    /// `Height 92 - top`, `Width (-77, 1)` (`:86-95`).
    pub content: [f32; 4],
}

/// Row `index`'s card geometry. `divider_h` is the `Images/UI/Divider` sheet
/// height (the C# `asset.Height()`, `:80`); 4 when the sheet is absent.
pub(crate) fn pack_card(row_rect: [f32; 4], index: usize, divider_h: f32) -> PackCard {
    let card = [row_rect[0], row_rect[1] + index as f32 * 107.0, row_rect[2], 102.0];
    let inner = uistate::inner(card, [5.0; 4]);
    // `BuildChildren`'s top walk (`:59-86`): name at 4 (16 tall, `UIText.cs:186`),
    // author at 4+16+6=26 (16*0.7 tall), divider at 26+11.2+10=47.2.
    let divider_top = 26.0 + 16.0 * 0.7 + 10.0;
    let content_top = divider_top + divider_h + 5.0;
    PackCard {
        card,
        icon: [inner[0] + 4.0, inner[1] + 4.0 + 10.0, 64.0, 64.0],
        icon_border: [inner[0], inner[1] + 10.0, 72.0, 72.0],
        name_pos: [inner[0] + 77.0, inner[1] + 4.0],
        author_pos: [inner[0] + 77.0, inner[1] + 26.0],
        divider: [inner[0] + 72.0, inner[1] + divider_top, inner[2] - 80.0, divider_h],
        content: [
            inner[0] + 77.0,
            inner[1] + content_top,
            inner[2] - 77.0,
            92.0 - content_top,
        ],
    }
}

/// `UIIconTextButton` (`UIIconTextButton.cs:36-76`) as the Publish button uses
/// it (`UIWorkshopPublishResourcePackListItem.cs:78`): the ctor's 44x34 is
/// replaced by `SetText`'s icon branch (`:71-75`) — `Width = title_w +
/// icon_w + 26`, `Height = max(title_h, icon_h) + 16`. The title is a scale-1
/// `UIText` (16 tall, `UIText.cs:186`); `icon` is `Images/UI/Workshop/Publish`.
/// Returns `(width, height)`; the sheet-less fallback keeps the ctor's 44x34.
pub(crate) fn publish_button_size(
    r: &SpriteRenderer,
    lookups: &assets::Lookups,
) -> [f32; 2] {
    let title = crate::lang::text("Workshop.Publish");
    let title_w = text::hud_text_width(r, lookups, &title, 1.0);
    match lookups.workshop_publish_icon {
        Some(icon) => [
            title_w + icon.width as f32 + 26.0,
            16.0f32.max(icon.height as f32) + 16.0,
        ],
        None => [44.0, 34.0],
    }
}

/// `UIIconTextButton.DrawSelf` (`UIIconTextButton.cs:78-100`): the
/// `PanelGrayscale` 10-slice at `Color.Lerp(Black, _color, FadeFromBlack=1) *
/// _opacity` (rest: `InventoryDefaultColor` at 1.0, `:47`/`:118`; hover:
/// `Lerp(InventoryDefaultColor, White, 0.7)` at 0.7, `:111`), the icon at
/// `right - icon_w - 5`, vertically centred (`:98`), then the title `UIText`
/// (`SetText`, `:60-67`: `Left 10`, `VAlign 0.5`, white). The hover border
/// sheet (`CategoryPanelHighlight`) is loaded (`:42`) but never drawn by this
/// version's `DrawSelf` — kept out, like the source.
pub(crate) fn draw_publish_button(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    rect: [f32; 4],
    hover: bool,
) {
    let (base, opacity) = if hover {
        // `MouseOver` (`:111`): Lerp(InventoryDefaultColor, White, 0.7), 0.7.
        (lerp_bytes(uistate::INVENTORY_DEFAULT_COLOR, [255; 4], 0.7), 0.7)
    } else {
        // `SetColor(Lerp(Black, InventoryDefaultColor, 1), 1)` (`:47`).
        (uistate::INVENTORY_DEFAULT_COLOR, 1.0)
    };
    let tint = color::tint(color::mul(base, opacity));
    uistate::draw_spliced_panel(
        r, lookups, lookups.cc_panel_gray, rect, 10.0, 10.0, 10.0, 10.0, tint,
    );
    let title = crate::lang::text("Workshop.Publish");
    if let Some(icon) = lookups.workshop_publish_icon {
        // `:98` — Lerp(color, White, 0.7) * opacity, right-anchored, centred.
        let icon_tint = color::tint(color::mul(lerp_bytes(base, [255; 4], 0.7), opacity));
        let ix = rect[0] + rect[2] - icon.width as f32 - 5.0;
        let iy = rect[1] + rect[3] * 0.5 - (icon.height / 2) as f32;
        crate::ui::push_whole(
            r,
            lookups,
            &icon,
            [ix, iy, icon.width as f32, icon.height as f32],
            icon_tint,
        );
    }
    // The title `UIText` (`:60-67` + `UIText.DrawSelf`, `UIText.cs:126-159`):
    // `Left 10`, `VAlign 0.5` against the 16 px text height, `TextOriginX 0`.
    let ty = rect[1] + (rect[3] - 16.0) * 0.5 - 2.0;
    text::border_string(r, lookups, rect[0] + 10.0, ty, [1.0; 4], &title);
}

/// XNA `Color.Lerp` per byte with `(byte)` truncation — the cluster's colour
/// maths (`Color.Lerp(a, b, amount)`, the `UIIconTextButton`/`GroupOptionButton`
/// tints). The port's [`crate::ui::world_load::lerp_color_packed`] is the
/// packed-u32 twin; this is the byte-array form the draw tints read.
pub(crate) fn lerp_bytes(a: [u8; 4], b: [u8; 4], amount: f32) -> [u8; 4] {
    std::array::from_fn(|i| (a[i] as f32 + (b[i] as f32 - a[i] as f32) * amount) as u8)
}

/// `UIText.IsWrapped`'s word wrap (`UIText.cs:178`'s `LayoutSnippets` at the
/// inner width): the words greedily fill `max_w` at `scale`; a word longer
/// than the line rides its own line. At least one line. The bestiary's
/// `wrap_text` twin, over the Death/Mouse font choice `large` picks.
pub(crate) fn wrap_text(
    r: &SpriteRenderer,
    lookups: &assets::Lookups,
    text_in: &str,
    scale: f32,
    max_w: f32,
    large: bool,
) -> Vec<String> {
    wrap_text_with(text_in, max_w, |s| {
        if large {
            match &lookups.death_font {
                Some(f) => font_width(f, s, scale),
                None => r.text_width(s, scale),
            }
        } else {
            text::hud_text_width(r, lookups, s, scale)
        }
    })
}

/// [`wrap_text`] over an injected measure — the renderer-free core the tests
/// drive (the bestiary twin's shape).
pub(crate) fn wrap_text_with(
    text_in: &str,
    max_w: f32,
    width: impl Fn(&str) -> f32,
) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current = String::new();
    for word in text_in.split_whitespace() {
        let candidate = if current.is_empty() {
            word.to_string()
        } else {
            format!("{current} {word}")
        };
        if !current.is_empty() && width(&candidate) > max_w {
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

/// The wrapped description's total height — `ChatManager.GetStringSize`'s Y
/// over the wrapped layout (`UIText.cs:179-183`): one line spacing per line
/// plus `WrappedTextBottomPadding * scale` (the ctor default 20,
/// `UIText.cs:87`). The info menu's scrollbar view reads it.
pub(crate) fn wrapped_height(
    lookups: &assets::Lookups,
    lines: usize,
    scale: f32,
    large: bool,
    bottom_padding: f32,
) -> f32 {
    let line = if large {
        text::death_line_height(lookups) * scale
    } else {
        text::line_height(lookups) * scale
    };
    lines as f32 * line + bottom_padding * scale
}

/// `UIWorkshopPublishResourcePackListItem.DrawSelf`'s two info panels
/// (`:182-214`): the `InnerPanelBackground` 10-slice plates behind the
/// author/version lines, each `num2 = (inner_w - 22 - icon_w - button_w) / 2`
/// wide at the button's height, bottom-aligned. Shared by the world twin
/// (`UIWorkshopPublishWorldListItem.cs:102-138`, the difficulty/size plates).
pub(crate) fn draw_info_plate(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    rect: [f32; 4],
) {
    uistate::draw_spliced_panel(
        r, lookups, lookups.ui_inner_panel, rect, 10.0, 10.0, 10.0, 10.0, [1.0; 4],
    );
}

/// The pack's author line — `Language.GetTextValue("UI.Author", Author)`
/// (`UIResourcePack.cs:68`, `UIResourcePackInfoMenu.cs:76`).
pub(crate) fn author_line(pack: &ResourcePack) -> String {
    crate::lang::text_args("UI.Author", &[pack.author.clone()])
}

/// The pack's version line — `UI.Version` with `GetFormattedVersion`
/// (`UIResourcePackInfoMenu.cs:83`, `ResourcePackVersion.cs:84-87`).
pub(crate) fn version_line(pack: &ResourcePack) -> String {
    crate::lang::text_args("UI.Version", &[pack.version.formatted()])
}

/// The scrollbar attach test the cluster's `Recalculate` overrides share
/// (`UIWorkshopSelectResourcePackToPublish.cs:84-102` and twins):
/// `UIList.UpdateScrollbar`'s `SetView(view, content)` then `CanScroll`
/// (`UIScrollbar.cs:49`) — the same clamp the select screens port
/// (`char_select::scrollbar_attaches`).
pub(crate) fn list_scrollbar_attaches(view_h: f32, content_h: f32) -> bool {
    panel::scrollbar_can_scroll(view_h.min(content_h), content_h)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `UIResourcePack.cs:135-141` — the four static colours' XNA truncation.
    #[test]
    fn pack_card_colours_match_the_xna_truncation() {
        assert_eq!(PACK_BG_REST, color::mul([26, 40, 89, 255], 0.8));
        assert_eq!(PACK_BORDER_REST, color::mul([13, 20, 44, 255], 0.8));
        assert_eq!(PACK_BG_HOVER, [46, 60, 119, 255]);
        assert_eq!(PACK_BORDER_HOVER, [20, 30, 56, 255]);
    }

    /// `UIResourcePack.BuildChildren` (`:57-95`): the top walk 4 -> 26 ->
    /// 47.2 -> content, the icon block at `inner + (4, 14)` / `(0, 10)`.
    #[test]
    fn pack_card_geometry_follows_the_buildchildren_walk() {
        let list = [100.0, 200.0, 300.0, 500.0];
        let c = pack_card(list, 1, 4.0);
        assert_eq!(c.card, [100.0, 307.0, 300.0, 102.0]); // 107 pitch
        // inner = card + 5 padding.
        assert_eq!(c.icon, [109.0, 326.0, 64.0, 64.0]);
        assert_eq!(c.icon_border, [105.0, 322.0, 72.0, 72.0]);
        assert_eq!(c.name_pos, [182.0, 316.0]);
        assert_eq!(c.author_pos, [182.0, 338.0]);
        assert_eq!(c.divider, [177.0, 359.2, 210.0, 4.0]);
        // content top = 47.2 + 4 + 5 = 56.2; height 92 - 56.2 = 35.8.
        assert_eq!(c.content[0], 182.0);
        assert!((c.content[1] - 368.2).abs() < 1e-3);
        assert_eq!(c.content[2], 213.0);
        assert!((c.content[3] - 35.8).abs() < 1e-3);
    }

    /// `UIList.cs:120-128` — the content heights: one row drops the padding.
    #[test]
    fn content_heights_drop_the_padding_for_a_single_row() {
        assert_eq!(publish_content_height(0), 0.0);
        assert_eq!(publish_content_height(1), 82.0);
        assert_eq!(publish_content_height(3), 261.0);
        assert_eq!(pack_content_height(0), 0.0);
        assert_eq!(pack_content_height(1), 102.0);
        assert_eq!(pack_content_height(3), 321.0);
    }

    /// `Color.Lerp` per byte with truncation — the `UIIconTextButton` hover
    /// tint (`:111`): Lerp((63,65,151), White, 0.7) = (197,198,223).
    #[test]
    fn lerp_bytes_truncates_like_xna() {
        assert_eq!(
            lerp_bytes([63, 65, 151, 255], [255; 4], 0.7),
            [197, 198, 223, 255]
        );
        assert_eq!(lerp_bytes([0; 4], [255; 4], 0.0), [0; 4]);
        assert_eq!(lerp_bytes([0; 4], [255; 4], 1.0), [255; 4]);
    }

    /// The wrap fills greedily and keeps an over-long word on its own line
    /// (`UIText.cs:178`'s `LayoutSnippets` shape, the bestiary twin). The
    /// injected measure stands in for the bitmap fallback's 8 px/char.
    #[test]
    fn wrap_text_fills_greedily() {
        let width = |s: &str| s.chars().count() as f32 * 8.0;
        // "aa bb cc" at max 40 wraps after "aa bb" (5 chars = 40 fits).
        let lines = wrap_text_with("aa bb cc", 40.0, width);
        assert_eq!(lines, vec!["aa bb", "cc"]);
        // An empty string is one empty line.
        assert_eq!(wrap_text_with("", 40.0, width), vec![""]);
        // An over-long word rides its own line, never split.
        let lines = wrap_text_with("a abcdefghij b", 40.0, width);
        assert_eq!(lines, vec!["a", "abcdefghij", "b"]);
    }
}
