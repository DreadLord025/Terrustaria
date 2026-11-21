//! The `UIWorldGenDebug` driver — W4-f23 closeout item 1: the F5 debug
//! pass-list overlay over the loading screen (`UIWorldLoad.cs:50-53`'s
//! `DebugOptions.enableDebugCommands && F5` → `UIWorldGenDebug.Open()`).
//!
//! The screen's nested element halves landed in the census lane
//! (`ui/worldgen_debug_elements.rs` — `PassState`, the colour machine, the
//! title/description builders, the icon factories; `ui/worldgen_pass_icons.rs`
//! — the 108-row `passIcons` table + `get_step_icon`). This is the DRIVER the
//! 4-point contract (research/126 §5) parked: it feeds `PassState` per executed
//! step from the live generation job, looks the icon up through
//! `pipeline::pass_name_id` → `get_step_icon`, and paints the pass list.
//!
//! Scope (the d85 dev-screens co-lane owns the full `UIWorldGenDebug` screen):
//! the port lands the driver as an OVERLAY over `FrontScreen::WorldLoad`, not
//! a new `FrontScreen` variant — the read-only pass-list surface (the rows,
//! their state colours, icons, titles). The controller actions (pause / step /
//! reset / snapshots, `UIWorldGenDebug.cs:928-1013`) need the resumable
//! `GenerationRunner` + the snapshot machinery (d86 scope); the abort rides
//! the loading screen's own Inventory trigger. The drunk-world `Main.rand`
//! arms (`UIGenProgressBar.cs:63-67`, `UIWorldLoad.cs:93-103`) stay deferred
//! with the RNG-order rule (ANALYSIS "The loading screen") — this driver draws
//! no RNG.

use glam::Vec2;
use tr_render::SpriteRenderer;

use super::color;
use super::in_rect;
use super::panel;
use super::text::hud_text;
use super::worldgen_debug_elements::{
    self as elements, PassIconEntry, PassState,
};
use super::worldgen_pass_icons::{self, IconSpec};
use crate::assets;
use tr_core::worldgen::pipeline::ExecutedStep;

/// The driver's per-frame feed — the live job's snapshot plus the executed
/// pipeline's descriptors. `None` when no generation job runs (the
/// `TR_UI=worldload` screenshot preset parks a fixed snapshot with no worker).
#[derive(Clone, Debug)]
pub(crate) struct DebugFeed {
    /// The executed steps in execution order (`pipeline::executed_steps`).
    pub steps: Vec<ExecutedStep>,
    /// `Controller.CurrentPass.Name` (`UIWorldGenDebug.cs:245`) — "" between
    /// steps.
    pub current_step: String,
    /// `WorldGen.Manifest.GenPassResults.Count` (`UIWorldGenDebug.cs:247`).
    pub completed_steps: usize,
}

/// `GenPassElement`'s row geometry as the pass list builds it
/// (`UIWorldGenDebug.cs:1061-1066`): `Height.Set(32f)` overrides the ctor's
/// 96 (`:450`), `PaddingLeft = 7f`, `Width = -4f` (the list width minus 4).
pub const ROW_HEIGHT: f32 = 32.0;
pub const ROW_PADDING_LEFT: f32 = 7.0;

/// The overlay's layout — the `controlListArea` (`UIWorldGenDebug.cs:914-920`:
/// 450 wide, `Height = -60f + 1f`, `Top = 30f`, `Left = 10f`) minus the control
/// panel the port does not draw (the controller actions are d86 scope). The
/// pass list fills the area; the scrollbar rides its right edge.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Layout {
    /// The pass-list rect (the `GenPassList`, `:1048-1057`).
    pub list: [f32; 4],
    /// The scrollbar rect (`UIScrollbar`, `:1070-1076`: `Top = 34f`,
    /// `Height = -38f + 1f`, `Left = -1f`, `HAlign = 1f`).
    pub scrollbar: [f32; 4],
}

/// Resolve the overlay layout at `screen` — the `controlListArea`'s
/// StyleDimensions through the screen height (`Height = -60f + 1f`).
pub(crate) fn layout(screen: Vec2) -> Layout {
    // `controlListArea`: Left 10, Top 30, Width 450, Height = screenH - 60.
    let area = [10.0, 30.0, 450.0, (screen.y - 60.0).max(0.0)];
    // The control panel (50 px + 2 gap, `:1015`) is not drawn (d86 scope), so
    // the scroll panel starts at the area top. `scrollPanel` PaddingTop 8
    // (`:1025`); the GenPassList Top 30 (`:1050`) sits below the search bar
    // (28 px, `:1037`) the port does not draw — the list starts at the panel
    // top + 8 (the search bar's slot is folded into the padding).
    let list = [area[0] + 4.0, area[1] + 8.0, area[2] - 8.0 - 20.0, area[3] - 16.0];
    // `UIScrollbar`: HAlign 1 (right edge), Left -1, Top 34, Height = -38 + 1f.
    let scrollbar = [
        area[0] + area[2] - 20.0 - 1.0,
        area[1] + 34.0,
        20.0,
        (area[3] - 38.0).max(0.0),
    ];
    Layout { list, scrollbar }
}

/// The pass list's content height — `UIList`'s inner list sums the row heights
/// (`ListPadding = 0f`, `:1053`): `rows * 32`.
pub(crate) fn content_height(rows: usize) -> f32 {
    rows as f32 * ROW_HEIGHT
}

/// `GenPassElement`'s state for one executed step (`UIWorldGenDebug.cs:243-275`):
/// `IsRunning` = `Controller.CurrentPass == Pass` (`:245`), `HasCompleted` =
/// `GenPassResults.Count > Index` (`:247`), `Skipped` = the result's flag
/// (`:249-259`). `IsHighlighted`/`CurrentTargetPass` are the controller's
/// alt-click highlight / pause target — both off in the read-only driver (no
/// controller actions, d86 scope).
///
/// Skipped-pass limitation (documented, dual-dungeons only): the C# `RunPass`
/// appends a `Skipped=true` `GenPassResult` for a disabled pass WITHOUT the
/// `Start`/`End` bookends (`WorldGenerator.cs:488-495`), so `GenPassResults.
/// Count` counts it; the port's `apply_step` skips a disabled pass without
/// notifying the `GenProgressHook`, so `completed_steps` (bumped by `end_step`)
/// counts only the enabled passes that ran. For a classic world (no disabled
/// passes) the two align exactly; for a DUAL_DUNGEONS world the completed
/// marker's row alignment shifts past the first disabled step. The port's
/// progress feed already excludes disabled passes from `total_weight`, so this
/// is the same model — the driver reads `enabled` from the special-seed gate
/// and leaves `skipped` false (the hook reports no per-step skip flag).
pub(crate) fn pass_state(
    step: &ExecutedStep,
    index: usize,
    feed: &DebugFeed,
    search: &str,
) -> PassState {
    PassState {
        running: !feed.current_step.is_empty() && feed.current_step == step.name,
        completed: feed.completed_steps > index,
        skipped: false,
        enabled: step.enabled,
        highlighted: false,
        is_target: false,
        matches_search: elements::matches_search(step.name, search),
    }
}

/// The pass row's icon spec — `GetPassIcon` (`UIWorldGenDebug.cs:418-429`)
/// through the executed-step name mapping (`pipeline::pass_name_id` →
/// `get_step_icon`, the W4-f23 deviation fix). A port-only step (no vanilla
/// pass) takes the bestiary-64 fallback.
pub(crate) fn step_icon(step: &ExecutedStep) -> IconSpec {
    worldgen_pass_icons::get_step_icon(step.name)
}

/// The icon's draw rect inside a row — `AddIcon` (`UIWorldGenDebug.cs:431-443`):
/// the `UIImage` child centred in the 26 px box with integer halving
/// (`icon_child_offsets`), at the row's `PaddingLeft` (7) + the panel padding
/// (2). Returns `(left, top, width, height)` relative to the row origin.
pub(crate) fn icon_rect(spec: IconSpec, tex_size: [i32; 2]) -> [f32; 4] {
    let entry = match spec {
        IconSpec::Bestiary(i) => PassIconEntry::from_bestiary_icon(i, tex_size),
        IconSpec::Item(i) => PassIconEntry::from_item(i, tex_size),
        IconSpec::ImageFrame(img, i, rows, cols) => {
            PassIconEntry::from_image_frame(img, i, rows, cols, tex_size)
        }
    };
    let (ox, oy) = elements::icon_child_offsets(&entry);
    [
        ROW_PADDING_LEFT + 2.0 + ox as f32,
        2.0 + oy as f32,
        entry.width as f32,
        entry.height as f32,
    ]
}

/// The overlay draw — `UIWorldGenDebug.DrawChildren` (`:1425-1431`) over the
/// loading screen: the pass-list rows in execution order, each a `UIPanel`
/// (`pass_panel_colors`) with its icon, index and name (`pass_text_color`),
/// lifted by the scroll position and clipped to the list rect. The hovered
/// row publishes its title/description tooltip (`TooltipElement`, `:41-56`).
///
/// `scroll` is the live `UIScrollbar.ViewPosition`; `mouse` feeds the hover.
/// The drunk-world RNG arms are NOT drawn here (the RNG-order rule).
pub(crate) fn draw(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    screen: Vec2,
    feed: &DebugFeed,
    scroll: f32,
    mouse: Vec2,
) {
    let l = layout(screen);
    let rows = feed.steps.len();
    let content_h = content_height(rows);
    let scroll = super::char_select::clamp_scroll(scroll, l.list[3], content_h);

    // The scroll panel's background (`UIPanel`, `:1016-1028`) — the default
    // panel colour the port's crafting panel also uses.
    let bg = color::tint(color::mul(color::shift_blue_to_cyan([63, 82, 151, 255]), 0.7));
    let border = color::tint(color::mul(color::shift_blue_to_cyan([89, 116, 213, 255]), 0.7));
    panel::draw_panel(r, lookups, lookups.ui_panel_bg, l.list, bg);
    panel::draw_panel(r, lookups, lookups.ui_panel_border, l.list, border);

    // The rows, clipped to the list (`UIList`'s OverflowHidden, `UIList.cs:59`).
    r.push_clip(l.list);
    let mut hovered: Option<(String, String)> = None;
    for (i, step) in feed.steps.iter().enumerate() {
        let y = l.list[1] + i as f32 * ROW_HEIGHT - scroll;
        // Cull rows outside the list (the straddling rows draw clipped).
        if y + ROW_HEIGHT < l.list[1] || y > l.list[1] + l.list[3] {
            continue;
        }
        let row = [l.list[0], y, l.list[2], ROW_HEIGHT];
        let state = pass_state(step, i, feed, "");
        let is_hover = in_rect(mouse, row);
        let (row_bg, row_border) = elements::pass_panel_colors(&state, is_hover);
        panel::draw_panel(r, lookups, lookups.ui_panel_bg, row, color::tint(row_bg));
        panel::draw_panel(r, lookups, lookups.ui_panel_border, row, color::tint(row_border));

        // The icon (`AddIcon`, `:431-443`) — the bestiary tags sheet for the
        // bestiary specs; the item specs fall back to the item icon.
        let spec = step_icon(step);
        draw_icon(r, lookups, spec, [row[0], row[1]]);

        // The index (`UIText(this.Index.ToString(), 0.5f)`, Left 2, Top 2,
        // `:455-460`) and the name (`UIText(pass.Name)`, Left 32, Top 4,
        // `:462-467`), both in `pass_text_color` (`:468-473`).
        let text_col = color::tint(elements::pass_text_color(&state));
        hud_text(r, lookups, row[0] + 2.0, row[1] + 2.0, 0.5, text_col, &i.to_string());
        hud_text(r, lookups, row[0] + 32.0, row[1] + 4.0, 1.0, text_col, step.name);

        if is_hover {
            // `TooltipElement` (`:41-56`): the title + description publish
            // while the row is hovered. `CanSubmitActions` is false (the
            // read-only driver has no paused controller, d86 scope).
            let title = elements::pass_title(step.name, &state);
            let desc = elements::pass_description(&state, false, false);
            hovered = Some((title, desc));
        }
    }
    r.pop_clip();

    // The scrollbar (`UIScrollbar`, `:1070-1078`) at the live view.
    panel::draw_scrollbar(
        r,
        lookups,
        l.scrollbar,
        l.list[3],
        content_h,
        scroll,
        true,
    );

    // The hovered row's tooltip (`Main.DisplayAndGetFakeItem`'s name/desc,
    // `:52`) — the port prints it as mouse text below the cursor.
    if let Some((title, desc)) = hovered {
        let text = if desc.is_empty() { title } else { format!("{title}\n{desc}") };
        super::tooltip::draw_mouse_text(
            r,
            lookups,
            mouse,
            &text,
            screen,
            super::UiPulse::default(),
        );
    }
}

/// One pass icon (`AddIcon`'s `UIImage`, `:431-443`): the bestiary specs frame
/// the `Icon_Tags_Shadow` sheet; the item specs draw the item icon; the
/// image-frame specs fall back to the bestiary sheet (the port packs no
/// `Tiles_3` UI sheet — the Grass row's `FromImageFrame` quirk draws the
/// fallback, documented). Missing sheets draw nothing.
fn draw_icon(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    spec: IconSpec,
    row_origin: [f32; 2],
) {
    match spec {
        IconSpec::Bestiary(i) => {
            let Some(sheet) = lookups.ui_bestiary_tags_shadow else { return };
            let tex = [sheet.width as i32, sheet.height as i32];
            let rect = icon_rect(spec, tex);
            // `FromBestiaryIcon` frames 16×5 at (i % 16, i / 16) (`:191`).
            let cell_w = sheet.width / 16;
            let cell_h = sheet.height / 5;
            let sx = (i % 16) as u32 * cell_w;
            let sy = (i / 16) as u32 * cell_h;
            let uv = assets::uv_rect(&sheet, lookups.atlas_size, sx as i32, sy as i32, cell_w, cell_h);
            r.push_ui_sprite(
                [row_origin[0] + rect[0], row_origin[1] + rect[1]],
                [rect[2], rect[3]],
                uv,
                [1.0; 4],
            );
        }
        IconSpec::Item(i) => {
            // `FromItem` (`:197-215`): the item's own sheet at the 20 px fit.
            let Some((uv, w, h)) = lookups.item_icon(i as u16) else { return };
            let rect = icon_rect(spec, [w as i32, h as i32]);
            r.push_ui_sprite(
                [row_origin[0] + rect[0], row_origin[1] + rect[1]],
                [rect[2], rect[3]],
                uv,
                [1.0; 4],
            );
        }
        IconSpec::ImageFrame(_, i, _, _) => {
            // The `FromImageFrame` specs (the Grass row's Tiles_3, `:325`) —
            // the port packs no such UI sheet; fall back to the bestiary
            // tags cell the index names (the quirk's draw is documented).
            draw_icon(r, lookups, IconSpec::Bestiary(i), row_origin);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tr_core::seeds::MicroSeedFlags;

    fn feed() -> DebugFeed {
        DebugFeed {
            steps: tr_core::worldgen::pipeline::executed_steps(MicroSeedFlags::none()),
            current_step: String::new(),
            completed_steps: 0,
        }
    }

    /// The 4-point contract's point 2: the pass list feeds `PassState` per
    /// executed step — running marks the current step, completed marks every
    /// step before the result count, the icon lookup rides the name mapping.
    #[test]
    fn the_pass_state_tracks_the_live_job() {
        let mut f = feed();
        assert!(!f.steps.is_empty(), "the executed pipeline has steps");
        // Before any step: nothing running, nothing completed.
        let s0 = pass_state(&f.steps[0], 0, &f, "");
        assert!(!s0.running && !s0.completed);
        // The first step runs.
        f.current_step = f.steps[0].name.to_string();
        let s0 = pass_state(&f.steps[0], 0, &f, "");
        assert!(s0.running, "CurrentPass == Pass (`:245`)");
        assert!(!s0.completed);
        // Two steps completed: rows 0 and 1 completed, row 2 not.
        f.current_step = f.steps[2].name.to_string();
        f.completed_steps = 2;
        assert!(pass_state(&f.steps[0], 0, &f, "").completed);
        assert!(pass_state(&f.steps[1], 1, &f, "").completed);
        let s2 = pass_state(&f.steps[2], 2, &f, "");
        assert!(!s2.completed && s2.running);
    }

    /// The 4-point contract's point 2 (icon half): every named executed step
    /// looks its icon up through `pass_name_id` → `get_step_icon`; a port-only
    /// step (vanilla == "") takes the bestiary-64 fallback (`:426`).
    #[test]
    fn every_executed_step_resolves_an_icon() {
        let f = feed();
        for step in &f.steps {
            let spec = step_icon(step);
            if step.vanilla.is_empty() {
                assert_eq!(spec, IconSpec::Bestiary(64), "port-only step fallback");
            } else {
                // The named step's icon is the table's row for its GenPassNameID
                // key (or the fallback if the table has no row — both are the
                // C#'s `GetPassIcon` arms).
                assert_eq!(spec, worldgen_pass_icons::get_pass_icon(step.vanilla));
            }
        }
    }

    /// The search filter (`MatchesSearch`, `:1373-1380`): whitespace matches
    /// all, a substring narrows, case-insensitive.
    #[test]
    fn the_search_filter_dims_non_matching_rows() {
        let f = feed();
        let terrain = f.steps.iter().find(|s| s.name == "terrain fill").unwrap();
        let idx = f.steps.iter().position(|s| s.name == "terrain fill").unwrap();
        assert!(pass_state(terrain, idx, &f, "  ").matches_search);
        assert!(pass_state(terrain, idx, &f, " TERRAIN ").matches_search);
        assert!(!pass_state(terrain, idx, &f, "lava").matches_search);
    }

    /// The row geometry: 32 px tall, the content height sums the rows, the
    /// scroll clamps into `[0, content - view]`.
    #[test]
    fn the_row_geometry_and_scroll_clamp() {
        let f = feed();
        let l = layout(Vec2::new(1600.0, 900.0));
        let ch = content_height(f.steps.len());
        assert_eq!(ch, f.steps.len() as f32 * 32.0);
        assert!(ch > l.list[3], "the full pipeline scrolls");
        assert_eq!(crate::ui::char_select::clamp_scroll(99999.0, l.list[3], ch), ch - l.list[3]);
        assert_eq!(crate::ui::char_select::clamp_scroll(-5.0, l.list[3], ch), 0.0);
    }
}