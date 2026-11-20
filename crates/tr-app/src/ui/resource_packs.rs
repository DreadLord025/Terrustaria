//! The resource-pack list card — W4-d83 (research/18b row 47):
//! `UIResourcePack` (`GameContent/UI/Elements/UIResourcePack.cs:10-142`),
//! the `UIPanel` subclass the resource-packs screen lists. The port carries
//! the static colour pairs, the fixed 102 px sizing, the `BuildChildren`
//! vertical layout walk and the icon/workshop-badge geometry; the panel
//! background/border draw rides `panel.rs` and the pack data
//! (`Terraria.IO.ResourcePack`) is the caller's.

use super::color;

/// `PANEL_PADDING` (`UIResourcePack.cs:12`) — `SetPadding(5f)` (`:51`).
pub const PANEL_PADDING: f32 = 5.0;
/// `ICON_SIZE` (`:14`) — the pack icon's 64×64 draw.
pub const ICON_SIZE: i32 = 64;
/// `ICON_BORDER_PADDING` (`:16`) — the icon's +4 inset inside the border.
pub const ICON_BORDER_PADDING: i32 = 4;
/// `HEIGHT_FLUFF` (`:18`) — the icon/border stack's +10 top offset (`:112-113`).
pub const HEIGHT_FLUFF: i32 = 10;
/// `HEIGHT` (`:20`) — Height = MinHeight = MaxHeight = 102 (`:46-48`).
pub const HEIGHT: f32 = 102.0;
/// `MIN_WIDTH` (`:22`) — MinWidth 102, Width 100% (`:49-50`).
pub const MIN_WIDTH: f32 = 102.0;

/// `DefaultBackgroundColor` (`:137`): `new Color(26, 40, 89) * 0.8f` —
/// XNA truncates every channel incl. alpha (255 * 0.8 = 204).
pub const DEFAULT_BACKGROUND: [u8; 4] = [20, 32, 71, 204];
/// `DefaultBorderColor` (`:138`): `new Color(13, 20, 44) * 0.8f`.
pub const DEFAULT_BORDER: [u8; 4] = [10, 16, 35, 204];
/// `HoverBackgroundColor` (`:139`): `new Color(46, 60, 119)` — opaque.
pub const HOVER_BACKGROUND: [u8; 4] = [46, 60, 119, 255];
/// `HoverBorderColor` (`:140`): `new Color(20, 30, 56)`.
pub const HOVER_BORDER: [u8; 4] = [20, 30, 56, 255];

/// `UIResourcePack` (`UIResourcePack.cs:10-142`), research/18b row 47
/// (W4-d83): the pack card's state — the hover colour swap and the sort
/// order. The pack itself (name/author/icon/branding) is the caller's data.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ResourcePackCard {
    /// `Order` (`:36`) — the manual sort key; `CompareTo` (`:116-119`)
    /// compares it ascending.
    pub order: i32,
    /// `BackgroundColor` — starts [`DEFAULT_BACKGROUND`] (`:44`), swaps on
    /// hover (`:121-126`), restores on out (`:128-133`).
    pub background_color: [u8; 4],
    /// `BorderColor` — the same swap (`:45`, `:125`, `:132`).
    pub border_color: [u8; 4],
}

impl ResourcePackCard {
    /// The ctor's state half (`:40-55`): default colours, 102 px fixed
    /// height, 102 px min width, padding 5, `OverflowHidden = true` (`:53`).
    pub fn new(order: i32) -> Self {
        Self {
            order,
            background_color: DEFAULT_BACKGROUND,
            border_color: DEFAULT_BORDER,
        }
    }

    /// `MouseOver` (`:121-126`): base first, then the hover pair.
    pub fn mouse_over(&mut self) {
        self.background_color = HOVER_BACKGROUND;
        self.border_color = HOVER_BORDER;
    }

    /// `MouseOut` (`:128-133`): back to the default pair.
    pub fn mouse_out(&mut self) {
        self.background_color = DEFAULT_BACKGROUND;
        self.border_color = DEFAULT_BORDER;
    }

    /// `DrawIcon` (`:109-114`): from the **inner** dimensions (padding 5
    /// already subtracted), the icon at `(X+4, Y+4+10)` 64×64 and the
    /// `Achievement_Borders` sheet at `(X, Y+10)` 72×72 — both int-truncated
    /// positions, both `Color.White`. Returns `(icon_rect, border_rect)`.
    pub fn icon_rects(inner: [f32; 4]) -> ([i32; 4], [i32; 4]) {
        (
            [
                inner[0] as i32 + ICON_BORDER_PADDING,
                inner[1] as i32 + ICON_BORDER_PADDING + HEIGHT_FLUFF,
                ICON_SIZE,
                ICON_SIZE,
            ],
            [inner[0] as i32, inner[1] as i32 + HEIGHT_FLUFF, 72, 72],
        )
    }

    /// The Steam Workshop badge (`:102-106`): `Extra[243]` at
    /// `(X + Width - badgeW - 3, Y + 2)` from the **outer** dimensions,
    /// `Color.White`, only for `BrandingType.SteamWorkshop`.
    pub fn workshop_badge_pos(rect: [f32; 4], badge_size: [f32; 2]) -> [f32; 2] {
        [rect[0] + rect[2] - badge_size[0] - 3.0, rect[1] + 2.0]
    }
}

/// The `BuildChildren` vertical walk (`:57-96`): the name at (77, 4), the
/// 0.7-scale author line below it (+6 gap), the `Divider` image below that
/// (+10 gap, at Left 72, Width 100% - 80, ScaleToFit), then the content
/// panel filling `92 - top` px. `name_h`/`author_h`/`divider_h` are the
/// measured `GetOuterDimensions().Height`s (the `UIText` metrics are row 53's).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PackChildLayout {
    /// The name `UIText` position (`:59-65`).
    pub name_pos: [f32; 2],
    /// The author `UIText` position (`:67-72`), scale 0.7 (`:68`).
    pub author_pos: [f32; 2],
    /// The divider `UIImage` rect (`:74-86`): Left 72, Width = W - 80.
    pub divider_rect: [f32; 4],
    /// The content panel rect (`:87-95`): Left 77, Height = 92 - top,
    /// Width = W - 77.
    pub content_rect: [f32; 4],
}

/// The layout walk for a card of width `w` (the height is pinned 102).
/// Positions are relative to the panel's inner origin, like the C#'s
/// `Left`/`Top` styles.
pub fn pack_child_layout(w: f32, name_h: f32, author_h: f32, divider_h: f32) -> PackChildLayout {
    let left = 77.0; // `:59`
    let mut top = 4.0; // `:60`
    let name_pos = [left, top];
    top += name_h + 6.0; // `:67`
    let author_pos = [left, top];
    top += author_h + 10.0; // `:74`
    let divider_rect = [72.0, top, w - 80.0, divider_h]; // `:78-82` (-80f, 100%)
    top += divider_h + 5.0; // `:86`
    PackChildLayout {
        name_pos,
        author_pos,
        divider_rect,
        content_rect: [left, top, w - left, 92.0 - top], // `:89-92`
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The static colours (`UIResourcePack.cs:135-141`): the defaults are
    /// the `* 0.8f` truncation (alpha included: 255*0.8 = 204), the hover
    /// pair opaque; the swap round-trips.
    #[test]
    fn the_colour_pairs_match_the_static_ctor() {
        assert_eq!(DEFAULT_BACKGROUND, color::mul([26, 40, 89, 255], 0.8));
        assert_eq!(DEFAULT_BORDER, color::mul([13, 20, 44, 255], 0.8));
        let mut card = ResourcePackCard::new(3);
        assert_eq!(card.order, 3);
        assert_eq!(card.background_color, DEFAULT_BACKGROUND);
        card.mouse_over();
        assert_eq!(card.background_color, [46, 60, 119, 255]);
        assert_eq!(card.border_color, [20, 30, 56, 255]);
        card.mouse_out();
        assert_eq!(card.background_color, DEFAULT_BACKGROUND);
        assert_eq!(card.border_color, DEFAULT_BORDER);
    }

    /// The icon stack (`:109-114`): from the inner rect (10.9, 20.8) the
    /// icon lands at (14, 34) 64×64 and the border at (10, 30) 72×72 —
    /// int truncation before the offsets; the badge rides the outer rect's
    /// right edge minus 3 (`:105`).
    #[test]
    fn the_icon_stack_and_badge_ride_the_csharp_offsets() {
        let (icon, border) = ResourcePackCard::icon_rects([10.9, 20.8, 400.0, 92.0]);
        assert_eq!(icon, [14, 34, 64, 64]);
        assert_eq!(border, [10, 30, 72, 72]);
        assert_eq!(
            ResourcePackCard::workshop_badge_pos([10.0, 20.0, 400.0, 102.0], [22.0, 22.0]),
            [385.0, 22.0]
        );
    }

    /// The child walk (`:57-96`): with a 16 px name, 11.2 px author (0.7
    /// scale) and a 2 px divider the tops walk 4 → 26 → 47.2 → 54.2; the
    /// content panel fills 92 - 54.2 = 37.8 px at width W - 77.
    #[test]
    fn the_child_layout_walks_the_top_accumulator() {
        let l = pack_child_layout(500.0, 16.0, 11.2, 2.0);
        assert_eq!(l.name_pos, [77.0, 4.0]);
        assert_eq!(l.author_pos, [77.0, 26.0]);
        assert_eq!(l.divider_rect, [72.0, 47.2, 420.0, 2.0]);
        assert_eq!(l.content_rect[0], 77.0);
        assert_eq!(l.content_rect[1], 54.2);
        assert_eq!(l.content_rect[2], 423.0);
        assert_eq!(l.content_rect[3], 92.0 - 54.2);
        assert_eq!((HEIGHT, MIN_WIDTH, PANEL_PADDING), (102.0, 102.0, 5.0));
    }
}
