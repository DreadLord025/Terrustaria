//! The workshop publish-info FORM's data/state half —
//! `AWorkshopPublishInfoState<T>` (`GameContent/UI/States/
//! AWorkshopPublishInfoState.cs:21-822`) + its two 46-line subclasses
//! (`WorkshopPublishInfoStateForWorld.cs`, `WorkshopPublishInfoStateForResourcePack.cs`),
//! W4-d86's owner decision (research/18c, f22 conflicts 4/5).
//!
//! Ported: the `Social/Base` data types (`WorkshopItemPublicSettingId.cs`,
//! `WorkshopTagOption.cs`, `WorkshopItemPublishSettings.cs`,
//! `FoundWorkshopEntryInfo.cs`), the publicity radio + tag-grid selection
//! machines (`:356-432`, `:473-489`), the publish-settings assembly
//! (`GetPublishSettings` `:668-678`), the `FoundWorkshopEntryInfo` re-import
//! (`SetTagsFromFoundEntry` `:179-199`), the `PrettifyPath` 30-char
//! truncation (`:693-713`), the >512 px preview rejection (`:732-742`), the
//! description strip (`:491-523`, `:553-581`), the back/publish routing
//! (`:587-608` + both subclasses' `GoToPublishConfirmation` `:27-35`) and
//! the scrollbar attach/detach (`UpdateScrollbar` `:284-300`).
//!
//! Excluded with `Social/**` (ANALYSIS:23673, the standing Workshop
//! exclusion): the Steam I/O — `FileBrowser.OpenFilePanel` (`:680-691`),
//! `Texture2D.FromStream` (`:724-725`), `Platform.OpenURL` (`:267-276`),
//! `SocialAPI.Workshop.Publish*`/`TryGetInfoFor*`/`SupportedTags`. The
//! seams take the dialog's RESULT / the decoded image's SIZE / the found
//! entry / the supported tags as INPUTS, so a future native picker or
//! Workshop module plugs in without touching the state machine. On a
//! no-Steam build the screen is unreachable exactly as vanilla
//! (`GetTagsToShow` reads `SocialAPI.Workshop.SupportedTags`; both ctors
//! are only called from the Workshop select screens — d84's lane).
//! `SetupGamepadPoints` (`:765-821`) is the Stage-1 gamepad exclusion.

#![allow(dead_code)] // wiring deferred — d84's select screens consume this module at the merge

use crate::ui::color;
use crate::ui::uistate::{Style, FANCY_UI_FAT_BUTTON_MOUSE_OVER, INVENTORY_DEFAULT_COLOR};

/// `WorkshopItemPublicSettingId` (`Social/Base/WorkshopItemPublicSettingId.cs:3-8`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum WorkshopItemPublicSettingId {
    #[default]
    Private,
    FriendsOnly,
    Public,
}

/// `WorkshopTagOption` (`Social/Base/WorkshopTagOption.cs:3-14`) — the
/// readonly pair the tag grid shows.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorkshopTagOption {
    pub name_key: String,
    pub internal_name_for_apis: String,
}

impl WorkshopTagOption {
    /// The ctor (`:9-13`).
    pub fn new(name_key: &str, internal_name: &str) -> Self {
        Self {
            name_key: name_key.to_string(),
            internal_name_for_apis: internal_name.to_string(),
        }
    }
}

/// `WorkshopItemPublishSettings` (`Social/Base/WorkshopItemPublishSettings.cs:5-17`).
#[derive(Clone, Debug, Default)]
pub struct WorkshopItemPublishSettings {
    /// `UsedTags` (`:7`, default empty).
    pub used_tags: Vec<WorkshopTagOption>,
    /// `Publicity` (`:9`, default Private = 0).
    pub publicity: WorkshopItemPublicSettingId,
    /// `PreviewImagePath` (`:11`, default null).
    pub preview_image_path: Option<String>,
}

impl WorkshopItemPublishSettings {
    /// `GetUsedTagsInternalNames` (`:13-16`).
    pub fn used_tags_internal_names(&self) -> Vec<&str> {
        self.used_tags.iter().map(|t| t.internal_name_for_apis.as_str()).collect()
    }
}

/// `FoundWorkshopEntryInfo` (`Social/Base/FoundWorkshopEntryInfo.cs:3-14`) —
/// the already-published entry `TryGetInfoFor*` returns.
#[derive(Clone, Debug, Default)]
pub struct FoundWorkshopEntryInfo {
    pub workshop_entry_id: u64,
    pub publicity: WorkshopItemPublicSettingId,
    /// `tags` (`:9`) — null in C# when the entry carries no tags; the
    /// re-import only touches the selection when it is Some (`:185`).
    pub tags: Option<Vec<String>>,
    pub preview_image_path: Option<String>,
    pub published_version: i32,
}

/// Which subclass the form is (`WorkshopPublishInfoStateForWorld` vs
/// `ForResourcePack`) — the two differ ONLY in the text keys, the display
/// name source and the Steam calls (both `GoToPublishConfirmation`s are
/// byte-identical bar the `Publish*` method, `:27-35` in each).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PublishedObjectKind {
    World,
    ResourcePack,
}

impl PublishedObjectKind {
    /// `_instructionsTextKey` (both ctors `:14`).
    pub fn instructions_text_key(self) -> &'static str {
        match self {
            Self::World => "Workshop.WorldPublishDescription",
            Self::ResourcePack => "Workshop.ResourcePackPublishDescription",
        }
    }

    /// `_publishedObjectNameDescriptorTexKey` (both ctors `:15`).
    pub fn name_descriptor_text_key(self) -> &'static str {
        match self {
            Self::World => "Workshop.WorldName",
            Self::ResourcePack => "Workshop.ResourcePackName",
        }
    }
}

/// The publicity radio's three options in `CreatePublicSettingsRow`'s
/// array order (`:359-383`): Public, FriendsOnly, Private — NOT the enum
/// order. Each carries its label key, description key and icon path.
pub const PUBLICITY_OPTIONS: [(WorkshopItemPublicSettingId, &str, &str, &str); 3] = [
    (
        WorkshopItemPublicSettingId::Public,
        "Workshop.SettingsPublicityPublic",
        "Workshop.SettingsPublicityPublicDescription",
        "Images/UI/Workshop/PublicityPublic",
    ),
    (
        WorkshopItemPublicSettingId::FriendsOnly,
        "Workshop.SettingsPublicityFriendsOnly",
        "Workshop.SettingsPublicityFriendsOnlyDescription",
        "Images/UI/Workshop/PublicityFriendsOnly",
    ),
    (
        WorkshopItemPublicSettingId::Private,
        "Workshop.SettingsPublicityPrivate",
        "Workshop.SettingsPublicityPrivateDescription",
        "Images/UI/Workshop/PublicityPrivate",
    ),
];

/// The tag grid's column count (`CreateTagOptionsPanel`'s `num = 3`,
/// `:407`) and per-row height (`FillUIList` passes 44, `:125`).
pub const TAGS_PER_ROW: usize = 3;
pub const TAG_ROW_HEIGHT: f32 = 44.0;

/// The selected-tag highlight (`ClickTagOption` `:477`,
/// `SetTagsFromFoundEntry` `:190`): `new Color(152, 175, 235)`; the
/// unselected colour is `Colors.InventoryDefaultColor`.
pub const TAG_SELECTED_COLOR: [u8; 4] = [152, 175, 235, 255];

/// One tag button's grid placement (`:417-422`): row `i / 3`, column
/// `i - row * 3`; width `-4 * (3 - 1)` px + `(1/3) * 0.98` percent,
/// `HAlign = col / 2`, `Left = (1 - 0.98) * (1 - HAlign * 2)` percent,
/// `Top = row * 44` px. Returns `(left, top, width, h_align)`.
pub fn tag_button_layout(i: usize) -> (Style, Style, Style, f32) {
    let row = i / TAGS_PER_ROW;
    let col = i - row * TAGS_PER_ROW;
    let h_align = col as f32 / (TAGS_PER_ROW - 1) as f32;
    (
        Style::pct((1.0 - 0.98) * (1.0 - h_align * 2.0)),
        Style::px(row as f32 * TAG_ROW_HEIGHT),
        Style::new(-4.0 * (TAGS_PER_ROW as f32 - 1.0), 1.0 / TAGS_PER_ROW as f32 * 0.98),
        h_align,
    )
}

/// The publicity buttons' placement (`:389-392`) — the same formula with
/// `Top = accumulatedHeight` (0 from `FillUIList`, `:124`).
pub fn publicity_button_layout(i: usize) -> (Style, Style, f32) {
    let count = PUBLICITY_OPTIONS.len();
    let h_align = i as f32 / (count - 1) as f32;
    (
        Style::pct((1.0 - 0.98) * (1.0 - h_align * 2.0)),
        Style::new(-4.0 * (count as f32 - 1.0), 1.0 / count as f32 * 0.98),
        h_align,
    )
}

/// The tag panel's height (`:408`): `44 * ceil(count / 3)` — the inner
/// panel; the stylized category plate adds `44 + 4` (`:436`, `:443`).
pub fn tag_panel_height(tag_count: usize) -> f32 {
    TAG_ROW_HEIGHT * (tag_count as f32 / TAGS_PER_ROW as f32).ceil()
}

/// The form's state — `AWorkshopPublishInfoState`'s data fields
/// (`:23-67`) minus the UIElement tree. `_optionPublicity` defaults Public
/// (`:47`); the tag list is `GetTagsToShow()`'s result (the Steam seam's
/// input — empty on a no-Steam build, where the screen is unreachable).
#[derive(Clone, Debug)]
pub struct WorkshopPublishInfoState {
    pub kind: PublishedObjectKind,
    /// `_dataObject`'s presence + `GetPublishedObjectDisplayName`
    /// (both subclasses `:18-25`: null → "null", else the object's Name).
    pub data_object_name: Option<String>,
    /// `_previousUIState != null` (`:594`) — the back/publish routing gate.
    pub has_previous_state: bool,
    /// `SocialAPI.Workshop != null` — the publish gate (both subclasses
    /// `:29`). False on the port's no-Steam build.
    pub workshop_available: bool,
    /// `_optionPublicity` (`:47`).
    pub option_publicity: WorkshopItemPublicSettingId,
    /// `_tagOptions`' selection flags, parallel to `tags` (`:416` — every
    /// button starts unselected).
    pub tags: Vec<WorkshopTagOption>,
    pub tag_selected: Vec<bool>,
    /// `_previewImagePath` (`:59`).
    pub preview_image_path: Option<String>,
    /// Whether the preview plate shows the DEFAULT image
    /// (`UpdateImagePreview` `:740`, `:755`) vs a loaded transient texture.
    pub preview_is_default: bool,
    /// The description strip's current text key (`_descriptionText`,
    /// `:568` default; `ShowOptionDescription`/`ClearOptionDescription`
    /// `:491-523`).
    pub description_key: String,
    /// `_isScrollbarAttached` (`:37`) + the list rect's width style
    /// (`UpdateScrollbar` `:284-300`).
    pub scrollbar_attached: bool,
}

impl WorkshopPublishInfoState {
    /// The ctor pair (`:69-73` + the subclasses' `:11-16`) +
    /// `SetDefaultOptions` (`:101-111`): publicity Public, the found-entry
    /// re-import, the preview refresh. `found_entry` is the
    /// `TryFindingTags` seam's result (None = the C# `false` return).
    pub fn new(
        kind: PublishedObjectKind,
        data_object_name: Option<String>,
        has_previous_state: bool,
        tags: Vec<WorkshopTagOption>,
        found_entry: Option<FoundWorkshopEntryInfo>,
    ) -> Self {
        let tag_selected = vec![false; tags.len()];
        let mut s = Self {
            kind,
            data_object_name,
            has_previous_state,
            workshop_available: false,
            option_publicity: WorkshopItemPublicSettingId::Public,
            tags,
            tag_selected,
            preview_image_path: None,
            preview_is_default: true,
            description_key: "Workshop.InfoDescriptionDefault".to_string(),
            scrollbar_attached: false,
        };
        s.set_tags_from_found_entry(found_entry.as_ref());
        s
    }

    /// `GetPublishedObjectDisplayName` (both subclasses `:18-25`).
    pub fn display_name(&self) -> &str {
        self.data_object_name.as_deref().unwrap_or("null")
    }

    /// `SetTagsFromFoundEntry` (`:179-199`): no entry → nothing; a null
    /// `tags` array leaves the selection alone but STILL imports the
    /// publicity (`:193` runs outside the `:185` gate).
    pub fn set_tags_from_found_entry(&mut self, info: Option<&FoundWorkshopEntryInfo>) {
        let Some(info) = info else { return };
        if let Some(tags) = &info.tags {
            for (i, tag) in self.tags.iter().enumerate() {
                self.tag_selected[i] = tags.contains(&tag.internal_name_for_apis);
            }
        }
        self.option_publicity = info.publicity;
    }

    /// `ClickPublicityOption` (`:480-489`): the clicked option becomes the
    /// selection for the whole radio group.
    pub fn click_publicity(&mut self, option: WorkshopItemPublicSettingId) {
        self.option_publicity = option;
    }

    /// `ClickTagOption` (`:473-478`): the toggle. Returns the button's new
    /// colour (`SetColor` `:477`).
    pub fn click_tag(&mut self, i: usize) -> [u8; 4] {
        self.tag_selected[i] = !self.tag_selected[i];
        self.tag_color(i)
    }

    /// The tag button's colour (`:477`, `:190`).
    pub fn tag_color(&self, i: usize) -> [u8; 4] {
        if self.tag_selected[i] { TAG_SELECTED_COLOR } else { INVENTORY_DEFAULT_COLOR }
    }

    /// `GetPublishSettings` (`:668-678`): the selected tags in grid order,
    /// the publicity, the preview path.
    pub fn publish_settings(&self) -> WorkshopItemPublishSettings {
        WorkshopItemPublishSettings {
            used_tags: self
                .tags
                .iter()
                .zip(&self.tag_selected)
                .filter(|(_, sel)| **sel)
                .map(|(t, _)| t.clone())
                .collect(),
            publicity: self.option_publicity,
            preview_image_path: self.preview_image_path.clone(),
        }
    }

    /// `HandleBackButtonUsage` (`:592-601`): no previous state → menuMode
    /// 0; else 888 + `SetState(_previousUIState)`. Returns the menu mode.
    pub fn handle_back_button(&self) -> i32 {
        if self.has_previous_state { 888 } else { 0 }
    }

    /// `GoToPublishConfirmation` (both subclasses `:27-35`): the Steam
    /// publish fires only when `Workshop != null && _dataObject != null`;
    /// the return to `_previousUIState` via menuMode 888 is UNCONDITIONAL.
    /// Returns `(settings-to-publish, menuMode)` — the settings are the
    /// `Publish*` call's argument, None when the gate blocks it.
    pub fn go_to_publish_confirmation(&self) -> (Option<WorkshopItemPublishSettings>, i32) {
        let settings = if self.workshop_available && self.data_object_name.is_some() {
            Some(self.publish_settings())
        } else {
            None
        };
        (settings, 888)
    }

    /// `Click_SetPreviewImage` (`:203-207`) + `OpenFileDialogueToSelectPreviewImage`
    /// (`:680-691`): sound 10 (the caller's side effect), then the dialog.
    /// A null result changes NOTHING (`:686`); a path lands and refreshes
    /// the preview. `decoded_size` is the `Texture2D.FromStream` seam's
    /// result — None = the load failed (`:727-730`, the error printer is
    /// the excluded fancy-error half) or no path.
    pub fn set_preview_image(&mut self, dialog_result: Option<String>, decoded_size: Option<(u32, u32)>) {
        let Some(path) = dialog_result else { return };
        self.preview_image_path = Some(path);
        self.update_image_preview(decoded_size);
    }

    /// `UpdateImagePreview` (`:715-757`) minus the file I/O: the plate
    /// shows [`prettify_path`]; a decoded image larger than 512 px on
    /// either axis is REJECTED — the path clears, the plate clears, the
    /// default image returns (`:732-742`; the `IssueReporter` call is the
    /// excluded Steam half). A failed load keeps the path but shows the
    /// default (`:753-756`).
    pub fn update_image_preview(&mut self, decoded_size: Option<(u32, u32)>) {
        if let Some((w, h)) = decoded_size {
            if w > 512 || h > 512 {
                self.preview_image_path = None;
                self.preview_is_default = true;
                return;
            }
        }
        self.preview_is_default = decoded_size.is_none();
    }

    /// The preview plate's contents (`:718-719`, `:739`): the prettified
    /// path, or None when no path is set.
    pub fn preview_plate_contents(&self) -> Option<String> {
        self.preview_image_path.as_deref().map(prettify_path)
    }

    /// `ShowOptionDescription` (`:491-518`): the hovered element's
    /// description key lands in the strip. Publicity buttons carry their
    /// `Description` key; tag buttons `NameKey + "Description"` (`:414`);
    /// the preview plate `Workshop.PreviewImagePathDescription` (`:165`);
    /// the disclaimer `Workshop.SteamDisclaimerDescrpition` (`:512` — the
    /// source's own typo, kept).
    pub fn show_option_description(&mut self, hovered: HoveredOption) {
        self.description_key = match hovered {
            HoveredOption::Publicity(i) => PUBLICITY_OPTIONS[i].2.to_string(),
            HoveredOption::Tag(i) => format!("{}Description", self.tags[i].name_key),
            HoveredOption::PreviewPlate => "Workshop.PreviewImagePathDescription".to_string(),
            HoveredOption::SteamDisclaimer => "Workshop.SteamDisclaimerDescrpition".to_string(),
        };
    }

    /// `ClearOptionDescription` (`:520-523`).
    pub fn clear_option_description(&mut self) {
        self.description_key = "Workshop.InfoDescriptionDefault".to_string();
    }

    /// `UpdateScrollbar` (`:284-300`): attach when scrollable, detach when
    /// not; the list rect widens to 100% with the bar gone, narrows to
    /// -25 px with it present. Returns the list rect's width style.
    pub fn update_scrollbar(&mut self, can_scroll: bool) -> Style {
        if self.scrollbar_attached && !can_scroll {
            self.scrollbar_attached = false;
        } else if !self.scrollbar_attached && can_scroll {
            self.scrollbar_attached = true;
        }
        if self.scrollbar_attached { Style::new(-25.0, 1.0) } else { Style::new(0.0, 1.0) }
    }
}

/// The hover sources `ShowOptionDescription` dispatches on (`:494-513`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HoveredOption {
    Publicity(usize),
    Tag(usize),
    PreviewPlate,
    SteamDisclaimer,
}

/// `PrettifyPath` (`:693-713`): strip everything through the last `/` or
/// `\` (both separators on every platform — `Path.DirectorySeparatorChar`
/// + `Path.AltDirectorySeparatorChar`), then truncate to 30 chars + `…`.
pub fn prettify_path(path: &str) -> String {
    let cut = path.rfind(['/', '\\']);
    let name = match cut {
        Some(i) => &path[i + 1..],
        None => path,
    };
    if name.chars().count() > 30 {
        let head: String = name.chars().take(30).collect();
        format!("{head}…")
    } else {
        name.to_string()
    }
}

/// The Steam disclaimer's text colour pair (`:237`, `:254-265`): Cyan at
/// rest, LightCyan on hover (sound 12 on over is the caller's side effect;
/// the click's `OpenURL` `:267-276` is the excluded Steam half).
pub fn disclaimer_text_color(hover: bool) -> [u8; 4] {
    if hover { [224, 255, 255, 255] } else { [0, 255, 255, 255] }
}

/// The Back/Publish buttons' faded pair (`FadedMouseOver`/`FadedMouseOut`,
/// `:610-621`): over → `(73, 94, 171)` + `Colors.FancyUIFatButtonMouseOver`;
/// out → `(63, 82, 151) * 0.8f` + black. NOTE the 0.8 — the manage-controls
/// screen's pair uses 0.7 (`UIManageControls.cs:933`).
pub fn faded_button_colors(hover: bool) -> ([u8; 4], [u8; 4]) {
    if hover {
        ([73, 94, 171, 255], FANCY_UI_FAT_BUTTON_MOUSE_OVER)
    } else {
        (color::mul([63, 82, 151, 255], 0.8), [0, 0, 0, 255])
    }
}

/// `OnInitialize`'s layout constants (`:78-98`): the outer container is
/// 600 px wide, top 200, height -200; the panel's anti-height is
/// `50 + 40 + 10 = 100`; the list's anti-height `6 + 70 = 76`; the
/// description strip `70 - 6 = 64` px; the back/publish buttons lift 40.
pub const OUTER_WIDTH: f32 = 600.0;
pub const OUTER_TOP: f32 = 200.0;
pub const PANEL_ANTI_HEIGHT: f32 = 100.0;
pub const LIST_ANTI_HEIGHT: f32 = 76.0;
pub const DESCRIPTION_HEIGHT: f32 = 64.0;
pub const BUTTON_Y_LIFT: f32 = 40.0;

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_tags() -> Vec<WorkshopTagOption> {
        vec![
            WorkshopTagOption::new("Workshop.TagLanguage", "Language"),
            WorkshopTagOption::new("Workshop.TagOverhaul", "Overhaul"),
            WorkshopTagOption::new("Workshop.TagTweaks", "Tweaks"),
            WorkshopTagOption::new("Workshop.TagUtility", "Utility"),
        ]
    }

    /// `SetDefaultOptions` (`:101-111`): publicity starts Public, every
    /// tag unselected, the preview default; the description strip reads
    /// the default key.
    #[test]
    fn the_form_boots_with_public_and_no_tags() {
        let s = WorkshopPublishInfoState::new(
            PublishedObjectKind::World,
            Some("My World".into()),
            true,
            sample_tags(),
            None,
        );
        assert_eq!(s.option_publicity, WorkshopItemPublicSettingId::Public);
        assert!(s.tag_selected.iter().all(|x| !x));
        assert!(s.preview_is_default);
        assert_eq!(s.description_key, "Workshop.InfoDescriptionDefault");
        assert_eq!(s.display_name(), "My World");
        assert_eq!(s.publish_settings().used_tags_internal_names(), Vec::<&str>::new());
    }

    /// `SetTagsFromFoundEntry` (`:179-199`): the found entry's tags select
    /// by InternalNameForAPIs; a NULL tags array leaves the selection but
    /// still imports the publicity (`:193` is outside the `:185` gate).
    #[test]
    fn the_found_entry_reimports_tags_and_publicity() {
        let info = FoundWorkshopEntryInfo {
            workshop_entry_id: 42,
            publicity: WorkshopItemPublicSettingId::FriendsOnly,
            tags: Some(vec!["Overhaul".into(), "Utility".into()]),
            preview_image_path: None,
            published_version: 3,
        };
        let mut s = WorkshopPublishInfoState::new(
            PublishedObjectKind::ResourcePack, None, true, sample_tags(), Some(info),
        );
        assert_eq!(s.option_publicity, WorkshopItemPublicSettingId::FriendsOnly);
        assert_eq!(s.tag_selected, vec![false, true, false, true]);
        assert_eq!(s.tag_color(1), TAG_SELECTED_COLOR);
        assert_eq!(s.tag_color(0), INVENTORY_DEFAULT_COLOR);
        // A null-tags entry: selection untouched, publicity still imported.
        let info2 = FoundWorkshopEntryInfo {
            publicity: WorkshopItemPublicSettingId::Private,
            tags: None,
            ..Default::default()
        };
        s.set_tags_from_found_entry(Some(&info2));
        assert_eq!(s.tag_selected, vec![false, true, false, true]);
        assert_eq!(s.option_publicity, WorkshopItemPublicSettingId::Private);
        // No entry at all: nothing moves.
        s.set_tags_from_found_entry(None);
        assert_eq!(s.option_publicity, WorkshopItemPublicSettingId::Private);
    }

    /// `ClickTagOption`/`ClickPublicityOption` (`:473-489`) +
    /// `GetPublishSettings` (`:668-678`): the toggles flip, the settings
    /// carry the selected tags in GRID order (not click order).
    #[test]
    fn the_click_machines_assemble_the_publish_settings() {
        let mut s = WorkshopPublishInfoState::new(
            PublishedObjectKind::World, Some("W".into()), true, sample_tags(), None,
        );
        assert_eq!(s.click_tag(3), TAG_SELECTED_COLOR);
        assert_eq!(s.click_tag(1), TAG_SELECTED_COLOR);
        assert_eq!(s.click_tag(3), INVENTORY_DEFAULT_COLOR); // toggle off
        s.click_publicity(WorkshopItemPublicSettingId::Private);
        s.preview_image_path = Some("C:/pics/icon.png".into());
        let settings = s.publish_settings();
        assert_eq!(settings.used_tags_internal_names(), vec!["Overhaul"]);
        assert_eq!(settings.publicity, WorkshopItemPublicSettingId::Private);
        assert_eq!(settings.preview_image_path.as_deref(), Some("C:/pics/icon.png"));
        // The radio is exclusive.
        s.click_publicity(WorkshopItemPublicSettingId::Public);
        assert_eq!(s.publish_settings().publicity, WorkshopItemPublicSettingId::Public);
    }

    /// The back/publish routing (`:592-601` + both subclasses `:27-35`):
    /// back → 0 without a previous state, 888 with; publish ALWAYS returns
    /// 888 but the Steam call only fires when Workshop + dataObject exist.
    #[test]
    fn the_routing_gates_match_the_source() {
        let mut s = WorkshopPublishInfoState::new(
            PublishedObjectKind::World, Some("W".into()), true, sample_tags(), None,
        );
        assert_eq!(s.handle_back_button(), 888);
        let (settings, mode) = s.go_to_publish_confirmation();
        assert_eq!(mode, 888);
        assert!(settings.is_none(), "no Workshop module — the publish gate blocks");
        s.workshop_available = true;
        let (settings, mode) = s.go_to_publish_confirmation();
        assert_eq!(mode, 888);
        assert!(settings.is_some());
        s.data_object_name = None;
        let (settings, _) = s.go_to_publish_confirmation();
        assert!(settings.is_none(), "null dataObject blocks the publish too");
        assert_eq!(s.display_name(), "null");
        s.has_previous_state = false;
        assert_eq!(s.handle_back_button(), 0);
        // The publish return is unconditional even without a previous state.
        assert_eq!(s.go_to_publish_confirmation().1, 888);
    }

    /// `PrettifyPath` (`:693-713`): both separators strip, the 30-char
    /// truncation appends the ellipsis, short names pass through.
    #[test]
    fn the_path_prettifier_strips_and_truncates() {
        assert_eq!(prettify_path("C:/Users/me/pics/icon.png"), "icon.png");
        assert_eq!(prettify_path("C:\\Users\\me\\icon.png"), "icon.png");
        assert_eq!(prettify_path("icon.png"), "icon.png");
        let long = "a".repeat(31);
        assert_eq!(prettify_path(&long), format!("{}…", "a".repeat(30)));
        let exact = "b".repeat(30);
        assert_eq!(prettify_path(&exact), exact);
        // A trailing separator yields the empty tail.
        assert_eq!(prettify_path("C:/pics/"), "");
    }

    /// `UpdateImagePreview` (`:715-757`): >512 on EITHER axis rejects and
    /// clears the path; a failed load keeps the path but shows the
    /// default; a null dialog result changes nothing (`:686`).
    #[test]
    fn the_preview_rejects_oversized_images() {
        let mut s = WorkshopPublishInfoState::new(
            PublishedObjectKind::World, Some("W".into()), true, sample_tags(), None,
        );
        s.set_preview_image(Some("C:/pics/big.png".into()), Some((513, 100)));
        assert!(s.preview_image_path.is_none());
        assert!(s.preview_is_default);
        assert_eq!(s.preview_plate_contents(), None);
        // Exactly 512 passes.
        s.set_preview_image(Some("C:/pics/ok.png".into()), Some((512, 512)));
        assert_eq!(s.preview_plate_contents().as_deref(), Some("ok.png"));
        assert!(!s.preview_is_default);
        // Height alone rejects.
        s.set_preview_image(Some("C:/pics/tall.png".into()), Some((100, 513)));
        assert!(s.preview_image_path.is_none());
        // A failed decode keeps the path, shows the default.
        s.set_preview_image(Some("C:/pics/broken.png".into()), None);
        assert_eq!(s.preview_plate_contents().as_deref(), Some("broken.png"));
        assert!(s.preview_is_default);
        // A cancelled dialog changes nothing.
        s.set_preview_image(None, None);
        assert_eq!(s.preview_plate_contents().as_deref(), Some("broken.png"));
    }

    /// The description strip (`:491-523`): each hover source's key, the
    /// tag key's `NameKey + "Description"` concat, the disclaimer's
    /// source-typo key, the clear back to the default.
    #[test]
    fn the_description_strip_follows_the_hover() {
        let mut s = WorkshopPublishInfoState::new(
            PublishedObjectKind::World, Some("W".into()), true, sample_tags(), None,
        );
        s.show_option_description(HoveredOption::Publicity(1));
        assert_eq!(s.description_key, "Workshop.SettingsPublicityFriendsOnlyDescription");
        s.show_option_description(HoveredOption::Tag(0));
        assert_eq!(s.description_key, "Workshop.TagLanguageDescription");
        s.show_option_description(HoveredOption::PreviewPlate);
        assert_eq!(s.description_key, "Workshop.PreviewImagePathDescription");
        s.show_option_description(HoveredOption::SteamDisclaimer);
        assert_eq!(s.description_key, "Workshop.SteamDisclaimerDescrpition");
        s.clear_option_description();
        assert_eq!(s.description_key, "Workshop.InfoDescriptionDefault");
    }

    /// The grid layout (`:407-422`): 3 per row, row = i/3, col = i%3,
    /// HAlign col/2; the panel height is 44 * ceil(n/3). The publicity
    /// radio's order is Public/FriendsOnly/Private (`:359-364`).
    #[test]
    fn the_grid_and_radio_layouts_walk_the_source_formulas() {
        let (left, top, width, h) = tag_button_layout(4); // row 1, col 1
        assert_eq!(top, Style::px(44.0));
        assert_eq!(h, 0.5);
        assert_eq!(left, Style::pct(0.0));
        assert_eq!(width, Style::new(-8.0, (1.0 / 3.0) * 0.98));
        let (_, top0, _, h0) = tag_button_layout(0);
        assert_eq!(top0, Style::px(0.0));
        assert_eq!(h0, 0.0);
        assert_eq!(tag_panel_height(4), 88.0);
        assert_eq!(tag_panel_height(3), 44.0);
        assert_eq!(tag_panel_height(0), 0.0);
        assert_eq!(PUBLICITY_OPTIONS[0].0, WorkshopItemPublicSettingId::Public);
        assert_eq!(PUBLICITY_OPTIONS[2].0, WorkshopItemPublicSettingId::Private);
        let (_, _, h2) = publicity_button_layout(2);
        assert_eq!(h2, 1.0);
    }

    /// `UpdateScrollbar` (`:284-300`): the attach/detach flips the list
    /// width between -25 px and full; the disclaimer/faded colours match
    /// the source pairs (0.8 here, not the manage-controls 0.7).
    #[test]
    fn the_scrollbar_attaches_and_the_colours_match() {
        let mut s = WorkshopPublishInfoState::new(
            PublishedObjectKind::World, Some("W".into()), true, sample_tags(), None,
        );
        assert_eq!(s.update_scrollbar(false), Style::new(0.0, 1.0));
        assert!(!s.scrollbar_attached);
        assert_eq!(s.update_scrollbar(true), Style::new(-25.0, 1.0));
        assert!(s.scrollbar_attached);
        assert_eq!(s.update_scrollbar(true), Style::new(-25.0, 1.0));
        assert_eq!(s.update_scrollbar(false), Style::new(0.0, 1.0));
        assert!(!s.scrollbar_attached);
        assert_eq!(disclaimer_text_color(false), [0, 255, 255, 255]);
        assert_eq!(disclaimer_text_color(true), [224, 255, 255, 255]);
        let (bg, border) = faded_button_colors(false);
        assert_eq!(bg, color::mul([63, 82, 151, 255], 0.8));
        assert_eq!(bg, [50, 65, 120, 204]);
        assert_eq!(border, [0, 0, 0, 255]);
        let (bg, border) = faded_button_colors(true);
        assert_eq!(bg, [73, 94, 171, 255]);
        assert_eq!(border, FANCY_UI_FAT_BUTTON_MOUSE_OVER);
        // The subclass text keys.
        assert_eq!(PublishedObjectKind::World.instructions_text_key(), "Workshop.WorldPublishDescription");
        assert_eq!(PublishedObjectKind::ResourcePack.name_descriptor_text_key(), "Workshop.ResourcePackName");
    }
}
