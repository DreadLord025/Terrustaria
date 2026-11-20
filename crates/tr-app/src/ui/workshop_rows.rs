//! The Steam Workshop list rows — W4-d83 (research/18b rows 58-60):
//! `UIWorkshopImportWorldListItem`
//! (`GameContent/UI/Elements/UIWorkshopImportWorldListItem.cs:14-199`),
//! `UIWorkshopPublishResourcePackListItem`
//! (`UIWorkshopPublishResourcePackListItem.cs:13-216`) and
//! `UIWorkshopPublishWorldListItem` (`UIWorkshopPublishWorldListItem.cs`).
//! The port carries the colour state machines, the info-panel layout walks
//! and the click gates; the SocialAPI/Workshop calls, the rejection menu
//! and the virtual keyboard are the caller's seams (the keyboard itself is
//! `keyboard.rs`).

use super::char_creation::lerp_rgb;
use super::color;

/// The shared rest pair (`UIWorkshopImportWorldListItem.cs:80-84`,
/// `UIWorkshopPublishResourcePackListItem.cs:114-123`):
/// bg `(63,82,151)*0.7`, border `(89,116,213)*0.7` — XNA truncation.
pub const REST_BG: [u8; 4] = [44, 57, 105, 178];
pub const REST_BORDER: [u8; 4] = [62, 81, 149, 178];
/// The shared hover pair (`:74-78`, `:103-106`): opaque
/// `(73,94,171)` / `(89,116,213)`.
pub const HOVER_BG: [u8; 4] = [73, 94, 171, 255];
pub const HOVER_BORDER: [u8; 4] = [89, 116, 213, 255];

/// `UIWorkshopImportWorldListItem`
/// (`UIWorkshopImportWorldListItem.cs:14-199`), research/18b row 58
/// (W4-d83): the downloaded-world import row. 96 px tall, padding 6
/// (`:66-72`); the import click opens the rename keyboard only for a valid
/// world (`:96-105`), the label shows `UI.Import` only while the play
/// button is hovered (`:86-94`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WorkshopImportWorldRow {
    /// `_orderInList` (`:26`) — `CompareTo`'s key (`:123-130`).
    pub order_in_list: i32,
    /// `BackgroundColor` — starts [`REST_BG`].
    pub background_color: [u8; 4],
    /// `BorderColor` — starts [`REST_BORDER`].
    pub border_color: [u8; 4],
    /// The button label's text — `""` at rest, `UI.Import` on button hover.
    pub label_shown: bool,
}

impl WorkshopImportWorldRow {
    /// The ctor's appearance half (`:30-57`, `:66-72`): rest colours, no
    /// label; the world icon sits at Left 4, the play button at Left 4
    /// VAlign 1, the label at Left 28 (4+24) Top -3 VAlign 1 (`:38-54`).
    pub fn new(order_in_list: i32) -> Self {
        Self {
            order_in_list,
            background_color: REST_BG,
            border_color: REST_BORDER,
            label_shown: false,
        }
    }

    /// `MouseOver`/`MouseOut` (`:132-142`).
    pub fn mouse_over(&mut self) {
        self.background_color = HOVER_BG;
        self.border_color = HOVER_BORDER;
    }

    pub fn mouse_out(&mut self) {
        self.background_color = REST_BG;
        self.border_color = REST_BORDER;
    }

    /// `PlayMouseOver`/`ButtonMouseOut` (`:86-94`): the label swap.
    pub fn button_mouse_over(&mut self) {
        self.label_shown = true;
    }

    pub fn button_mouse_out(&mut self) {
        self.label_shown = false;
    }

    /// `ImportButtonClick_ImportWorldToLocalFiles`'s gate (`:96-105`):
    /// fires only when the listening element **is** the event target and
    /// the world data is valid — then `PlaySound(10)` + the rename keyboard
    /// (`Workshop.EnterNewNameForImportedWorld`, allowEmpty, max 27).
    /// Returns `true` when the caller must open the keyboard.
    pub fn import_click(&self, listening_is_target: bool, data_valid: bool) -> bool {
        listening_is_target && data_valid
    }

    /// `OnFinishedSettingName` (`:107-114`): the new display name is
    /// `name.Trim()` before the workshop import call.
    pub fn finished_name(name: &str) -> String {
        name.trim().to_string()
    }

    /// The name row's text + colour (`:157-163`): a valid world draws its
    /// cropped name in White; an invalid one greys it and prefixes
    /// `"(" + StatusID.Search.GetName(LoadStatus) + ") "` — the status name
    /// is the caller's lookup (the port's `LoadStatus` ids are unported).
    pub fn name_row(name: &str, valid: bool, status_name: &str) -> (String, [u8; 4]) {
        if valid {
            (name.to_string(), [255, 255, 255, 255])
        } else {
            (
                format!("({status_name}) {name}"),
                [128, 128, 128, 255], // `Color.Gray`
            )
        }
    }

    /// The `DrawSelf` info-panel walk (`:151-198`). `icon_right` is
    /// `num = worldIcon.X + Width` (`:156`), `icon_y` the icon's Y (the
    /// name anchors 2 px above it, `:166`), `inner` the card's inner
    /// dimensions. Returns the three panel rects (difficulty 100 px,
    /// world-size 150/+40 px, creation-date fills the rest) and the name /
    /// divider anchors. The 3-piece panel itself is
    /// `cards::inner_panel_rects` (the identical `DrawPanel`, `:144-149`).
    pub fn info_layout(
        icon_right: f32,
        icon_y: f32,
        inner: [f32; 4],
        outer: [f32; 4],
        has_world_size: bool,
        english: bool,
    ) -> ImportInfoLayout {
        let mut x = icon_right + 6.0; // `:170`
        let y = inner[1] + 29.0;
        let difficulty = [x, y, 100.0, 0.0]; // `:171-172`
        x += 100.0 + 5.0; // `:178`
        let world_size = if has_world_size {
            let w = if english { 150.0 } else { 190.0 }; // `:181-185` +40
            let r = [x, y, w, 0.0];
            x += w + 5.0; // `:190`
            Some(r)
        } else {
            None
        };
        let date_w = inner[0] + inner[2] - x; // `:192`
        ImportInfoLayout {
            name_pos: [icon_right + 6.0, icon_y - 2.0], // `:166`
            divider_pos: [icon_right, inner[1] + 21.0], // `:169`
            divider_scale_x: (outer[0] + outer[2] - icon_right) / 8.0, // `:169`
            difficulty,
            world_size,
            date: [x, y, date_w, 0.0],
        }
    }
}

/// The import row's layout outputs (`UIWorkshopImportWorldListItem.cs:151-198`).
/// Panel heights are placeholders — the 3-piece draw uses the sheet height
/// (`cards::draw_inner_panel`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ImportInfoLayout {
    /// The world-name anchor (`:166`): `(num + 6, worldIcon.Y - 2)` — the
    /// caller passes the icon's Y through `name_pos[1]`.
    pub name_pos: [f32; 2],
    /// The divider's position (`:169`): `(num, inner.Y + 21)`.
    pub divider_pos: [f32; 2],
    /// The divider's x-scale (`:169`): `(outer.Right - num) / 8`.
    pub divider_scale_x: f32,
    /// The difficulty panel (`:170-172`), 100 px.
    pub difficulty: [f32; 4],
    /// The world-size panel (`:179-191`), 150 px (190 non-English) or none.
    pub world_size: Option<[f32; 4]>,
    /// The creation-date panel (`:192-193`), the remaining inner width.
    pub date: [f32; 4],
}

/// `UIWorkshopPublishResourcePackListItem`
/// (`UIWorkshopPublishResourcePackListItem.cs:13-216`), research/18b row 59
/// (W4-d83): the pack publish row. 82 px tall, padding 6 (`:95-101`); a
/// non-publishable pack (a zip) greys both colour states (`:107-122`) and
/// its click routes to the rejection menu instead (`:125-147`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WorkshopPublishPackRow {
    /// `_orderInList` (`:29`).
    pub order_in_list: i32,
    /// `_canPublish` (`:39`, ctor `:46`).
    pub can_publish: bool,
    /// `BackgroundColor`.
    pub background_color: [u8; 4],
    /// `BorderColor`.
    pub border_color: [u8; 4],
}

impl WorkshopPublishPackRow {
    /// The ctor's appearance half (`:41-48`, `:95-101`): the rest colours
    /// through the canPublish gate.
    pub fn new(order_in_list: i32, can_publish: bool) -> Self {
        let mut row = Self {
            order_in_list,
            can_publish,
            background_color: REST_BG,
            border_color: REST_BORDER,
        };
        row.set_colors_not_hovered();
        row
    }

    /// `SetColorsToHovered` (`:103-112`): the shared hover pair; a zip
    /// swaps the border to `(150,150,150)` and lerps the bg halfway to
    /// `(120,120,120)` — `(96,107,145,255)`.
    pub fn set_colors_hovered(&mut self) {
        self.background_color = HOVER_BG;
        self.border_color = HOVER_BORDER;
        if !self.can_publish {
            self.border_color = [150, 150, 150, 255]; // `* 1f` is a no-op
            self.background_color = color::mul(
                lerp_rgb(HOVER_BG, [120, 120, 120, 255], 0.5),
                1.0,
            ); // `:110`
        }
    }

    /// `SetColorsToNotHovered` (`:114-123`): the shared rest pair; a zip
    /// swaps the border to `(127,127,127)*0.7` = `(88,88,88,178)` and lerps
    /// `(63,82,151)`→`(80,80,80)` 0.5 then `*0.7` = `(49,56,80,178)`.
    pub fn set_colors_not_hovered(&mut self) {
        self.background_color = REST_BG;
        self.border_color = REST_BORDER;
        if !self.can_publish {
            self.border_color = color::mul([127, 127, 127, 255], 0.7); // `:120`
            self.background_color = color::mul(
                lerp_rgb([63, 82, 151, 255], [80, 80, 80, 255], 0.5),
                0.7,
            ); // `:121`
        }
    }

    /// `MouseOver`/`MouseOut` (`:170-180`).
    pub fn mouse_over(&mut self) {
        self.set_colors_hovered();
    }

    pub fn mouse_out(&mut self) {
        self.set_colors_not_hovered();
    }

    /// `PublishButtonClick` + `TryMovingToRejectionMenuIfNeeded`
    /// (`:125-147`): the click fires only for the target element; a zip
    /// plays sound 10 and routes to the rejection menu
    /// (`Workshop.ReportIssue_CannotPublishZips`, menuMode 1000001),
    /// a publishable pack opens `WorkshopPublishInfoStateForResourcePack`.
    /// Returns `None` (swallowed), `Some(true)` (rejection) or
    /// `Some(false)` (publish info).
    pub fn publish_click(&self, listening_is_target: bool) -> Option<bool> {
        if !listening_is_target {
            return None; // `:127`
        }
        Some(!self.can_publish) // `:133-147`
    }

    /// The `DrawSelf` two-panel walk (`:187-215`): both panels share
    /// `num2 = (inner.W - 22 - iconArea.W - publishButton.W) / 2` (`:194`),
    /// the publish button's height, at `y = inner.Y + inner.H - height`
    /// (`:196`); the second sits `num2 + 5` right (`:205`). The panels are
    /// 10/10/10/10 **spliced** (`:182-185`, unlike the import row's
    /// 3-piece). The name draws at `(iconRight + 8, inner.Y + 3)` (`:193`).
    pub fn info_layout(
        inner: [f32; 4],
        icon_area_w: f32,
        publish_button_size: [f32; 2],
    ) -> PackInfoLayout {
        let num = inner[0] + 4.0 + icon_area_w; // the icon area sits Left 4 (`:53`)
        let panel_w = (inner[2] - 22.0 - icon_area_w - publish_button_size[0]) / 2.0; // `:194`
        let h = publish_button_size[1]; // `:195`
        let y = inner[1] + inner[3] - h; // `:196`
        let x = num + 8.0;
        PackInfoLayout {
            name_pos: [num + 8.0, inner[1] + 3.0], // `:193`
            author_panel: [x, y, panel_w, h],      // `:197-198`
            version_panel: [x + panel_w + 5.0, y, panel_w, h], // `:205-207`
        }
    }
}

/// The publish-pack row's layout outputs
/// (`UIWorkshopPublishResourcePackListItem.cs:187-215`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PackInfoLayout {
    pub name_pos: [f32; 2],
    /// The author panel (`UI.Author`), spliced 10/10/10/10.
    pub author_panel: [f32; 4],
    /// The version panel (`UI.Version`), same size, +5 gap.
    pub version_panel: [f32; 4],
}

/// `UIWorkshopPublishWorldListItem`
/// (`UIWorkshopPublishWorldListItem.cs:11-140`), research/18b row 60
/// (W4-d83): the world publish row. Same 82 px / padding 6 chrome as the
/// pack row (`:53-59`) but **no** canPublish grey arms (`:61-71` are the
/// plain shared pairs) and no rejection routing — the click only checks
/// the target (`:73-79`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WorkshopPublishWorldRow {
    /// `_orderInList` (`:21`).
    pub order_in_list: i32,
    /// `BackgroundColor`.
    pub background_color: [u8; 4],
    /// `BorderColor`.
    pub border_color: [u8; 4],
}

impl WorkshopPublishWorldRow {
    /// The ctor's appearance half (`:25-31`, `:53-59`).
    pub fn new(order_in_list: i32) -> Self {
        Self {
            order_in_list,
            background_color: REST_BG,
            border_color: REST_BORDER,
        }
    }

    /// `MouseOver`/`MouseOut` (`:90-100`) — the plain shared pairs.
    pub fn mouse_over(&mut self) {
        self.background_color = HOVER_BG;
        self.border_color = HOVER_BORDER;
    }

    pub fn mouse_out(&mut self) {
        self.background_color = REST_BG;
        self.border_color = REST_BORDER;
    }

    /// `PublishButtonClick` (`:73-79`): only the target check — opens
    /// `WorkshopPublishInfoStateForWorld` (the caller's state push).
    pub fn publish_click(&self, listening_is_target: bool) -> bool {
        listening_is_target
    }

    /// The `DrawSelf` two-panel walk (`:107-139`): like the pack row's
    /// split, but the panels start at `num + 6` (not +8, `:116`), the
    /// name anchors at `inner.Y + 3` (`:113`), and the **second** panel
    /// (world size) grows +40 for non-English cultures (`:127-131`) —
    /// the first (difficulty) keeps `num2`.
    pub fn info_layout(
        inner: [f32; 4],
        icon_w: f32,
        publish_button_size: [f32; 2],
        english: bool,
    ) -> PublishWorldLayout {
        let num = inner[0] + 4.0 + icon_w; // the icon sits Left 4 (`:33`)
        let num2 = (inner[2] - 22.0 - icon_w - publish_button_size[0]) / 2.0; // `:114`
        let h = publish_button_size[1]; // `:115`
        let y = inner[1] + inner[3] - h; // `:116`
        let x = num + 6.0;
        let size_w = if english { num2 } else { num2 + 40.0 }; // `:127-131`
        PublishWorldLayout {
            name_pos: [num + 6.0, inner[1] + 3.0], // `:113`
            difficulty_panel: [x, y, num2, h],     // `:117-118`
            size_panel: [x + num2 + 5.0, y, size_w, h], // `:126-132`
        }
    }
}

/// The publish-world row's layout outputs
/// (`UIWorkshopPublishWorldListItem.cs:107-139`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PublishWorldLayout {
    pub name_pos: [f32; 2],
    /// The difficulty panel (`GetDifficulty`), spliced 10/10/10/10.
    pub difficulty_panel: [f32; 4],
    /// The world-size panel (`UI.WorldSizeFormat`), +40 non-English.
    pub size_panel: [f32; 4],
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The publish-world row (`UIWorkshopPublishWorldListItem.cs:53-139`):
    /// plain colour pairs (no grey arms), target-only click, the +6 panel
    /// origin and the second panel's non-English +40.
    #[test]
    fn the_publish_world_row_keeps_plain_pairs_and_grows_the_size_panel() {
        let mut row = WorkshopPublishWorldRow::new(4);
        assert_eq!(row.background_color, REST_BG);
        row.mouse_over();
        assert_eq!(row.background_color, HOVER_BG);
        assert_eq!(row.border_color, HOVER_BORDER);
        row.mouse_out();
        assert_eq!(row.border_color, REST_BORDER);
        assert!(row.publish_click(true));
        assert!(!row.publish_click(false));
        // inner (10, 50, 500, 70), icon 60, button 100x41:
        // num2 = (500-22-60-100)/2 = 159; x = 10+4+60+6 = 80; y = 79.
        let l = WorkshopPublishWorldRow::info_layout([10.0, 50.0, 500.0, 70.0], 60.0, [100.0, 41.0], true);
        assert_eq!(l.name_pos, [80.0, 53.0]);
        assert_eq!(l.difficulty_panel, [80.0, 79.0, 159.0, 41.0]);
        assert_eq!(l.size_panel, [244.0, 79.0, 159.0, 41.0]);
        let ru = WorkshopPublishWorldRow::info_layout([10.0, 50.0, 500.0, 70.0], 60.0, [100.0, 41.0], false);
        assert_eq!(ru.size_panel, [244.0, 79.0, 199.0, 41.0], "+40 (`:130`)");
        assert_eq!(ru.difficulty_panel[2], 159.0, "the first panel keeps num2");
    }

    /// The import row (`UIWorkshopImportWorldListItem.cs:74-105`): the
    /// colour swap, the label latch, the valid+target click gate and the
    /// name trim.
    #[test]
    fn the_import_row_swaps_colours_and_gates_the_click() {
        let mut row = WorkshopImportWorldRow::new(2);
        assert_eq!(row.background_color, [44, 57, 105, 178]);
        assert_eq!(row.border_color, [62, 81, 149, 178]);
        row.mouse_over();
        assert_eq!(row.background_color, [73, 94, 171, 255]);
        row.mouse_out();
        assert_eq!(row.border_color, [62, 81, 149, 178]);
        assert!(!row.label_shown);
        row.button_mouse_over();
        assert!(row.label_shown);
        row.button_mouse_out();
        assert!(!row.label_shown);
        assert!(row.import_click(true, true));
        assert!(!row.import_click(true, false), "invalid world: no keyboard");
        assert!(!row.import_click(false, true), "not the target: swallowed");
        assert_eq!(WorkshopImportWorldRow::finished_name("  My World  "), "My World");
        let (text, c) = WorkshopImportWorldRow::name_row("My World", true, "");
        assert_eq!((text.as_str(), c), ("My World", [255, 255, 255, 255]));
        let (text, c) = WorkshopImportWorldRow::name_row("My World", false, "LoadFailed");
        assert_eq!(text, "(LoadFailed) My World");
        assert_eq!(c, [128, 128, 128, 255]);
    }

    /// The import layout walk (`:170-197`): 100 px difficulty, 150/190 px
    /// size (culture gate), the date fills the inner remainder; the
    /// divider scales by (outer.Right - num)/8.
    #[test]
    fn the_import_info_panels_walk_the_widths() {
        let l = WorkshopImportWorldRow::info_layout(
            100.0,
            48.0,
            [10.0, 50.0, 500.0, 84.0],
            [4.0, 44.0, 512.0, 96.0],
            true,
            true,
        );
        assert_eq!(l.name_pos, [106.0, 46.0]);
        assert_eq!(l.difficulty, [106.0, 79.0, 100.0, 0.0]);
        assert_eq!(l.world_size, Some([211.0, 79.0, 150.0, 0.0]));
        assert_eq!(l.date, [366.0, 79.0, 510.0 - 366.0, 0.0]);
        assert_eq!(l.divider_pos, [100.0, 71.0]);
        assert_eq!(l.divider_scale_x, (516.0 - 100.0) / 8.0);
        let non_en = WorkshopImportWorldRow::info_layout(
            100.0,
            48.0,
            [10.0, 50.0, 500.0, 84.0],
            [4.0, 44.0, 512.0, 96.0],
            true,
            false,
        );
        assert_eq!(non_en.world_size, Some([211.0, 79.0, 190.0, 0.0]));
        let no_size = WorkshopImportWorldRow::info_layout(
            100.0,
            48.0,
            [10.0, 50.0, 500.0, 84.0],
            [4.0, 44.0, 512.0, 96.0],
            false,
            true,
        );
        assert_eq!(no_size.world_size, None);
        assert_eq!(no_size.date[0], 211.0, "the date slides left (`:192`)");
    }

    /// The publish-pack row (`UIWorkshopPublishResourcePackListItem.cs:103-147`):
    /// the zip greys both states with the exact lerps, the click routes to
    /// the rejection menu, and the two panels split the leftover width.
    #[test]
    fn the_publish_row_greys_zips_and_splits_the_panels() {
        let mut zip = WorkshopPublishPackRow::new(0, false);
        assert_eq!(zip.background_color, [49, 56, 80, 178]);
        assert_eq!(zip.border_color, [88, 88, 88, 178]);
        zip.mouse_over();
        assert_eq!(zip.background_color, [96, 107, 145, 255]);
        assert_eq!(zip.border_color, [150, 150, 150, 255]);
        zip.mouse_out();
        assert_eq!(zip.background_color, [49, 56, 80, 178]);
        assert_eq!(zip.publish_click(true), Some(true), "zip -> rejection");
        assert_eq!(zip.publish_click(false), None);
        let ok = WorkshopPublishPackRow::new(1, true);
        assert_eq!(ok.background_color, REST_BG);
        assert_eq!(ok.publish_click(true), Some(false), "pack -> publish info");
        // inner (10, 50, 500, 70), icon 72, button 100x41:
        // num2 = (500 - 22 - 72 - 100)/2 = 153; y = 50 + 70 - 41 = 79.
        let l = WorkshopPublishPackRow::info_layout([10.0, 50.0, 500.0, 70.0], 72.0, [100.0, 41.0]);
        assert_eq!(l.name_pos, [90.0, 53.0]);
        assert_eq!(l.author_panel, [90.0, 79.0, 153.0, 41.0]);
        assert_eq!(l.version_panel, [248.0, 79.0, 153.0, 41.0]);
    }
}
