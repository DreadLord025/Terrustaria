//! The worldgen-debug screen's nested element halves — W4-f23 (census lane 4,
//! research/18e): `UIWorldGenDebug.TooltipElement`
//! (`GameContent/UI/States/UIWorldGenDebug.cs:29-57`),
//! `UIWorldGenDebug.UIImageButtonWithExtraIcon` (`:93-170`) and
//! `UIWorldGenDebug.GenPassElement` (`:172-735`). The screen itself is the
//! d86 scope-out (dev-only, `UIWorldLoad.cs:50-52` F5); its nested
//! `UIElement` subclasses were in no element census row — this ports their
//! pure state/geometry halves so the scope-out loses no element behaviour.
//! The controller/manifest/snapshot plumbing stays the screen's seam
//! (inputs below stand in for `UIWorldGenDebug.Controller`/`WorldGen.Manifest`).
#![allow(dead_code)] // dev-only screen (d86 scope-out); helpers parked for it

use super::char_creation::lerp_rgb;
use super::color;

// --- TooltipElement (UIWorldGenDebug.cs:29-57) --------------------------------

/// `TooltipElement` (`:29-57`): a mouse-transparent child (`:38`
/// `IgnoresMouseInteraction = true`) that, while the PARENT is hovered,
/// publishes a fake item's name/description tooltip (`:44-54`). The two
/// `Func<string>` providers are the caller's; the port takes their outputs.
pub const TOOLTIP_IGNORES_MOUSE: bool = true; // `:38`

/// `TooltipElement.DrawSelf` (`:41-56`): nothing unless
/// `base.Parent.IsMouseHovering` (`:43`); then the title is the provider's
/// string, the description is `provider?.Invoke() ?? string.Empty`
/// (`:46-51` — a null description becomes "", never stays null), and the
/// fake item is rarity `ItemRarityColor.StrongRed10` (= 10,
/// `Enums/ItemRarityColor.cs:17`) via `Main.DisplayAndGetFakeItem` (`:52`).
/// Returns `(name_override, tooltip_text)` when the tooltip publishes.
pub fn tooltip_publish(
    parent_hovering: bool,
    title: &str,
    description: Option<&str>,
) -> Option<(String, String)> {
    if !parent_hovering {
        return None; // `:43`
    }
    Some((title.to_string(), description.unwrap_or("").to_string()))
}

/// The fake item's rarity (`:52`, `ItemRarityColor.StrongRed10`).
pub const TOOLTIP_ITEM_RARITY: i32 = 10;

// --- UIImageButtonWithExtraIcon (UIWorldGenDebug.cs:93-170) -------------------

/// `UIImageButtonWithExtraIcon.DrawSelf`'s icon half (`:113-152`): after the
/// base button draw, the icon rides the element rect truncated to ints
/// (`ToRectangle`, `CalculatedStyle.cs:15-18`), `Inflate(-2, -2)` (`:121` —
/// XNA adds the offsets to X/Y and subtracts twice from W/H), then the
/// non-square aspect fit (`:134-150`): the SHORT side's ratio shrinks the
/// LONG side, the removed pixels split with integer-halved centre shift.
/// `frame` is `_iconFrame` when set, else the whole texture (`:124-133`).
/// Returns the destination rect `[x, y, w, h]` (ints).
pub fn extra_icon_rect(rect: [f32; 4], frame: Option<[i32; 4]>, tex_size: [i32; 2]) -> [i32; 4] {
    // `:120-121`: ToRectangle truncates; XNA `Inflate(h, v)` is
    // `X -= h; Y -= v; Width += 2h; Height += 2v`, so (-2,-2) insets 2 px
    // per side (X/Y +2, W/H -4).
    let mut d = [
        rect[0] as i32 - (-2),
        rect[1] as i32 - (-2),
        rect[2] as i32 + (-2) * 2,
        rect[3] as i32 + (-2) * 2,
    ];
    let (width, height) = match frame {
        Some(f) => (f[2], f[3]),            // `:126-127`
        None => (tex_size[0], tex_size[1]), // `:131-132`
    };
    if width != height {
        if width < height {
            // `:138-141`: num = w/h; num2 = W - (int)(W*num); W -= num2; X += num2/2.
            let num = width as f32 / height as f32;
            let num2 = d[2] - (d[2] as f32 * num) as i32;
            d[2] -= num2;
            d[0] += num2 / 2;
        } else {
            // `:145-148`: the mirrored vertical arm.
            let num3 = height as f32 / width as f32;
            let num4 = d[3] - (d[3] as f32 * num3) as i32;
            d[3] -= num4;
            d[1] += num4 / 2;
        }
    }
    d
}

/// The icon's draw tint (`:151`): `IconColor * (hover ? _visibilityActive :
/// _visibilityInactive)` — the base `UIImageButton` visibilities
/// (`UIImageButton.cs:13-15`, 1.0 / 0.4 defaults) with `SetVisibility`'s
/// XNA clamp (`:71-75`, min-wins when min > max). XNA byte-truncates the
/// product (`color::mul`).
pub fn extra_icon_tint(icon_color: [u8; 4], hover: bool, active: f32, inactive: f32) -> [u8; 4] {
    let xna_clamp = |v: f32| v.min(1.0).max(0.0);
    let m = if hover {
        xna_clamp(active)
    } else {
        xna_clamp(inactive)
    };
    color::mul(icon_color, m)
}

/// `UIWorldGenDebug.SetButtonState` (`:840-852`): `Enabled` →
/// `SetVisibility(1f, 0.4f)`, `NotVisible` → `SetVisibility(0f, 0f)`;
/// `IgnoresMouseInteraction = state != Enabled` (`:851`). Returns
/// `(active, inactive, ignores_mouse)`.
pub fn set_button_state(enabled: bool) -> (f32, f32, bool) {
    if enabled {
        (1.0, 0.4, false) // `:844-846`
    } else {
        (0.0, 0.0, true) // `:847-849`, `:851`
    }
}

// --- GenPassElement (UIWorldGenDebug.cs:172-735) ------------------------------

/// `GenPassElement.PassIconEntry` (`:174-233`): the icon sheet name, source
/// region and draw size. The two image factories share the 20 px fit with
/// the 1.2 cap (`:203-208`, `:221-226`): `num = 20f / (region.Width >
/// region.Height ? region.Width : asset.Height())` — NOTE the else branch
/// reads the whole ASSET height (`:203`/`:221`), not the region's own.
#[derive(Clone, Debug, PartialEq)]
pub struct PassIconEntry {
    pub icon: String,
    pub region: [i32; 4],
    pub width: i32,
    pub height: i32,
}

/// `Utils.Frame` (`Utils.cs:1344-1349`): integer cell sizes, no padding.
fn frame_rect(tex: [i32; 2], cols: i32, rows: i32, fx: i32, fy: i32) -> [i32; 4] {
    let w = tex[0] / cols;
    let h = tex[1] / rows;
    [w * fx, h * fy, w, h]
}

/// The shared 20 px fit (`:203-208`): `20 / (regionW > regionH ? regionW :
/// assetH)`, capped at 1.2, then `(int)(region * num)` truncation.
fn fit_size(region: [i32; 4], asset_h: i32) -> (i32, i32) {
    let denom = if region[2] > region[3] {
        region[2]
    } else {
        asset_h
    };
    let mut num = 20.0 / denom as f32;
    if num > 1.2 {
        num = 1.2;
    }
    (
        (region[2] as f32 * num) as i32,
        (region[3] as f32 * num) as i32,
    )
}

impl PassIconEntry {
    /// `FromBestiaryIcon` (`:184-196`): `Icon_Tags_Shadow` framed 16×5 at
    /// `(index % 16, index / 16)`, fixed 26×26 draw size (no fit).
    pub fn from_bestiary_icon(index: i32, tex: [i32; 2]) -> Self {
        Self {
            icon: "Images/UI/Bestiary/Icon_Tags_Shadow".to_string(),
            region: frame_rect(tex, 16, 5, index % 16, index / 16),
            width: 26,
            height: 26,
        }
    }

    /// `FromItem` (`:197-215`): `Images/Item_<index>`, whole-sheet frame,
    /// the 20 px fit (asset height = the sheet's own height here).
    pub fn from_item(index: i32, tex: [i32; 2]) -> Self {
        let region = frame_rect(tex, 1, 1, 0, 0);
        let (width, height) = fit_size(region, tex[1]);
        Self {
            icon: format!("Images/Item_{index}"),
            region,
            width,
            height,
        }
    }

    /// `FromImageFrame` (`:216-232`): the sheet framed `rowCount × lineCount`
    /// at `(index % rowCount, index / rowCount)`, the same 20 px fit against
    /// the WHOLE asset height (`:221`).
    pub fn from_image_frame(
        image: &str,
        index: i32,
        row_count: i32,
        line_count: i32,
        tex: [i32; 2],
    ) -> Self {
        let region = frame_rect(
            tex,
            row_count,
            line_count,
            index % row_count,
            index / row_count,
        );
        let (width, height) = fit_size(region, tex[1]);
        Self {
            icon: image.to_string(),
            region,
            width,
            height,
        }
    }
}

/// `GenPassElement.AddIcon` (`:431-444`): the `UIImage` child centred in the
/// 26 px box with INTEGER halving — `Top = (26 - Height) / 2`,
/// `Left = (26 - Width) / 2` (`:439-440`), `ScaleToFit = true`. Returns the
/// child's `(left, top)` offsets.
pub fn icon_child_offsets(entry: &PassIconEntry) -> (i32, i32) {
    ((26 - entry.width) / 2, (26 - entry.height) / 2)
}

/// `GenPassElement`'s ctor pins (`:449-450`): `SetPadding(2f)`,
/// `Height.Set(96f, 0f)`.
pub const PASS_ELEMENT_PADDING: f32 = 2.0;
pub const PASS_ELEMENT_HEIGHT: f32 = 96.0;

/// The pass row's state inputs — the C# reads them off
/// `UIWorldGenDebug.Controller` / `WorldGen.Manifest` (`:243-275`); the port
/// takes them as data (the controller is the screen's unported seam).
#[derive(Clone, Copy, Debug, Default)]
pub struct PassState {
    /// `IsRunning` (`:245`): `Controller.CurrentPass == Pass`.
    pub running: bool,
    /// `HasCompleted` (`:247`): `GenPassResults.Count > Index`.
    pub completed: bool,
    /// `Skipped` (`:249-259`): the result's flag, false before completion.
    pub skipped: bool,
    /// `Pass.Enabled`.
    pub enabled: bool,
    /// `IsHighlighted` (`:275`): the config's name set.
    pub highlighted: bool,
    /// `CurrentTargetPass == Pass` (`:583`, `:604`).
    pub is_target: bool,
    /// `parent.MatchesSearch(pass)` (`:471`, `:1373-1380`).
    pub matches_search: bool,
}

/// `MatchesSearch` (`:1373-1380`): an all-whitespace search matches
/// everything; otherwise `pass.Name.ToLowerInvariant().Contains(
/// searchText.Trim().ToLowerInvariant())`.
pub fn matches_search(pass_name: &str, search_text: &str) -> bool {
    if search_text.trim().is_empty() {
        return true;
    }
    pass_name
        .to_lowercase()
        .contains(&search_text.trim().to_lowercase())
}

/// The name/index text colour (`:468-473`): running → Yellow, skipped →
/// DarkGreen, completed → (0,230,0), disabled → DarkGray, else White; then
/// `*= matchesSearch ? 1f : 0.6f` (XNA truncation, alpha rides).
pub fn pass_text_color(s: &PassState) -> [u8; 4] {
    let base = if s.running {
        [255, 255, 0, 255] // Color.Yellow
    } else if s.skipped {
        [0, 100, 0, 255] // Color.DarkGreen
    } else if s.completed {
        [0, 230, 0, 255] // `new Color(0, 230, 0)`
    } else if !s.enabled {
        [169, 169, 169, 255] // Color.DarkGray
    } else {
        [255, 255, 255, 255] // Color.White
    };
    color::mul(base, if s.matches_search { 1.0 } else { 0.6 })
}

/// `SetColorsToHovered` (`:572-590`) / `SetColorsToNotHovered` (`:592-610`):
/// the panel pair with the highlight/target/disabled overrides in the C#'s
/// order — target border wins over highlight, disabled lerps the CURRENT
/// background toward grey and re-multiplies. Returns `(background, border)`.
pub fn pass_panel_colors(s: &PassState, hovered: bool) -> ([u8; 4], [u8; 4]) {
    let (mut bg, mut border) = if hovered {
        ([73, 94, 171, 255], [89, 116, 213, 255]) // `:574-575`
    } else {
        (
            color::mul([63, 82, 151, 255], 0.7),  // `:594`
            color::mul([89, 116, 213, 255], 0.7), // `:595`
        )
    };
    if s.highlighted {
        if hovered {
            bg = [110, 30, 150, 255]; // `:578`
            border = [171, 53, 255, 255]; // `:579`
        } else {
            bg = color::mul([110, 30, 150, 255], 0.7); // `:598`
            border = color::mul([171, 53, 255, 255], 0.7); // `:599`
        }
    }
    if s.is_target {
        border = [255, 231, 69, 255]; // `:583`/`:604` — NOT dimmed
    }
    if !s.enabled {
        if hovered {
            border = color::mul([150, 150, 150, 255], 1.0); // `:587`
            bg = color::mul(lerp_rgb(bg, [120, 120, 120, 255], 0.5), 1.0); // `:588`
        } else {
            border = color::mul([127, 127, 127, 255], 0.7); // `:608`
            bg = color::mul(lerp_rgb(bg, [80, 80, 80, 255], 0.5), 0.7); // `:609`
        }
    }
    (bg, border)
}

/// `GetTitle` (`:626-637`): "Skipped: name" / "Disabled: name" /
/// "Run to name" (not completed) / "Rerun to name".
pub fn pass_title(name: &str, s: &PassState) -> String {
    if s.skipped {
        format!("Skipped: {name}")
    } else if !s.enabled {
        format!("Disabled: {name}")
    } else {
        format!("{} to {name}", if s.completed { "Rerun" } else { "Run" })
    }
}

/// `GetDescription` (`:639-655`): the enabled block ("Hold shift…", the
/// orange must-be-paused line when actions are locked AND (completed OR
/// shift held)), then the not-completed-and-not-skipped right/left-click
/// line + "Shift to edit ranges", then the always-on highlight line.
/// `can_submit_actions` = `UIWorldGenDebug.CanSubmitActions` (`:798`).
pub fn pass_description(s: &PassState, can_submit_actions: bool, shift_held: bool) -> String {
    let mut text = String::new();
    if s.enabled {
        text += "Hold shift to ignore snapshots\n"; // `:644`
        if !can_submit_actions && (s.completed || shift_held) {
            text += "[c/FFA500:Must be paused to rerun or load snapshots]\n"; // `:647`
        }
    }
    if !s.completed && !s.skipped {
        // `:652`: string.Concat(text + click-line, "Shift to edit ranges\n").
        text += if s.enabled {
            "Right click to disable\n"
        } else {
            "Left click to enable\n"
        };
        text += "Shift to edit ranges\n";
    }
    text + "Alt click to toggle highlight\n" // `:654`
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `TooltipElement.DrawSelf` (`:41-56`): parent-hover gate, null
    /// description → empty string, StrongRed10 rarity pin.
    #[test]
    fn the_tooltip_publishes_only_under_the_parent_hover() {
        assert_eq!(tooltip_publish(false, "T", Some("D")), None);
        assert_eq!(
            tooltip_publish(true, "T", Some("D")),
            Some(("T".to_string(), "D".to_string()))
        );
        assert_eq!(
            tooltip_publish(true, "T", None),
            Some(("T".to_string(), String::new())),
            "null description becomes \"\" (`:48-51`)"
        );
        assert!(TOOLTIP_IGNORES_MOUSE);
        assert_eq!(TOOLTIP_ITEM_RARITY, 10);
    }

    /// `UIImageButtonWithExtraIcon.DrawSelf` (`:120-150`): ToRectangle
    /// truncation + Inflate(-2,-2), the non-square fit's int truncation and
    /// integer-halved centre shift, square icons untouched.
    #[test]
    fn the_extra_icon_insets_and_aspect_fits_in_ints() {
        // Square icon: only the inset (10.9→10, +2; 40→40-4=36).
        assert_eq!(
            extra_icon_rect([10.9, 20.9, 40.0, 40.0], None, [32, 32]),
            [12, 22, 36, 36]
        );
        // Wide icon 2:1 in a 36x36 box: num3 = 0.5, num4 = 36-18 = 18,
        // H -= 18 → 18, Y += 18/2 = 9 (`:145-148`).
        assert_eq!(
            extra_icon_rect([0.0, 0.0, 40.0, 40.0], None, [64, 32]),
            [2, 11, 36, 18]
        );
        // Tall icon 1:2: num = 0.5, num2 = 18, W → 18, X += 9 (`:138-141`).
        assert_eq!(
            extra_icon_rect([0.0, 0.0, 40.0, 40.0], None, [32, 64]),
            [11, 2, 18, 36]
        );
        // The frame's own size drives the fit, not the texture (`:124-127`).
        assert_eq!(
            extra_icon_rect([0.0, 0.0, 40.0, 40.0], Some([0, 0, 20, 20]), [64, 32]),
            [2, 2, 36, 36]
        );
    }

    /// The icon tint + `SetButtonState` (`:151`, `:840-852`): XNA
    /// truncation on the visibility product, NotVisible zeroes both
    /// visibilities and ignores the mouse.
    #[test]
    fn button_states_and_icon_tints_truncate() {
        assert_eq!(set_button_state(true), (1.0, 0.4, false));
        assert_eq!(set_button_state(false), (0.0, 0.0, true));
        // White * 0.4 = 102 exactly; PaleVioletRed (219,112,147) * 0.4
        // truncates 87.6→87, 44.8→44, 58.8→58.
        assert_eq!(
            extra_icon_tint([255, 255, 255, 255], false, 1.0, 0.4),
            [102, 102, 102, 102]
        );
        assert_eq!(
            extra_icon_tint([219, 112, 147, 255], false, 1.0, 0.4),
            [87, 44, 58, 102]
        );
        assert_eq!(
            extra_icon_tint([219, 112, 147, 255], true, 1.0, 0.4),
            [219, 112, 147, 255]
        );
        // NotVisible: zero tint either way.
        assert_eq!(
            extra_icon_tint([255, 255, 255, 255], true, 0.0, 0.0),
            [0, 0, 0, 0]
        );
    }

    /// `PassIconEntry`'s factories (`:184-232`): the 16×5 bestiary grid,
    /// the 20 px fit's region-width/ASSET-height denominator + 1.2 cap,
    /// and `AddIcon`'s integer-halved centring (`:439-440`).
    #[test]
    fn pass_icons_frame_and_fit_like_the_source() {
        // Icon_Tags_Shadow is 16 cols × 5 rows of 26 px cells (416×130).
        let e = PassIconEntry::from_bestiary_icon(26, [416, 130]);
        assert_eq!(e.region, [260, 26, 26, 26], "26 = col 10, row 1");
        assert_eq!((e.width, e.height), (26, 26));
        assert_eq!(icon_child_offsets(&e), (0, 0));
        // FromItem: a 40×40 sheet fits at 20/40 = 0.5 → 20×20.
        let i = PassIconEntry::from_item(169, [40, 40]);
        assert_eq!((i.width, i.height), (20, 20));
        assert_eq!(icon_child_offsets(&i), (3, 3));
        // The 1.2 cap: a 10×10 region would fit at 2.0 → capped 1.2 → 12.
        let small = PassIconEntry::from_item(1, [10, 10]);
        assert_eq!((small.width, small.height), (12, 12));
        // FromImageFrame("Images/Tiles_3", 5, 45, 1) (`:325`): Frame(45, 1,
        // 5, 0) — the region is 18 wide but the FULL 810 tall, and the fit's
        // else branch divides by the whole asset height (`:221`): num =
        // 20/810, so width truncates to 0 while height stays 20. The quirk
        // is the port.
        let g = PassIconEntry::from_image_frame("Images/Tiles_3", 5, 45, 1, [810, 810]);
        assert_eq!(g.region, [90, 0, 18, 810]);
        assert_eq!(
            (g.width, g.height),
            (0, 20),
            "the asset-height denominator quirk"
        );
        // Odd sizes halve toward zero: (26-21)/2 = 2, not 2.5.
        let odd = PassIconEntry {
            icon: String::new(),
            region: [0; 4],
            width: 21,
            height: 25,
        };
        assert_eq!(icon_child_offsets(&odd), (2, 0));
    }

    /// The colour machine (`:468-473`, `:572-610`): state precedence
    /// running > skipped > completed > disabled > white, the 0.6 search
    /// dim, target border over highlight, disabled lerp+re-multiply order.
    #[test]
    fn pass_colors_follow_the_override_order() {
        let mut s = PassState {
            enabled: true,
            matches_search: true,
            ..Default::default()
        };
        assert_eq!(pass_text_color(&s), [255, 255, 255, 255]);
        s.running = true;
        s.skipped = true; // running wins (`:470`)
        assert_eq!(pass_text_color(&s), [255, 255, 0, 255]);
        s.running = false;
        assert_eq!(pass_text_color(&s), [0, 100, 0, 255]);
        s.matches_search = false;
        assert_eq!(
            pass_text_color(&s),
            [0, 60, 0, 153],
            "0.6 truncation incl alpha"
        );
        // Rest pair: (63,82,151)*0.7 = (44,57,105), border (62,81,149).
        let plain = PassState {
            enabled: true,
            matches_search: true,
            ..Default::default()
        };
        let (bg, border) = pass_panel_colors(&plain, false);
        assert_eq!(bg, [44, 57, 105, 178]);
        assert_eq!(border, [62, 81, 149, 178]);
        // Hover + highlight + target: border is the UNdimmed target gold.
        let ht = PassState {
            highlighted: true,
            is_target: true,
            ..plain
        };
        let (bg, border) = pass_panel_colors(&ht, true);
        assert_eq!(bg, [110, 30, 150, 255]);
        assert_eq!(border, [255, 231, 69, 255]);
        // Disabled rest: bg = Lerp(restBg → (80,80,80), 0.5) * 0.7 in that
        // ORDER (`:609`) — the lerp rides the already-dimmed pair, alpha
        // included; pin the composition (the f32 lerp truncation is
        // `lerp_rgb`'s own pinned behaviour, char_creation.rs:1118).
        let dis = PassState {
            enabled: false,
            ..plain
        };
        let (bg, border) = pass_panel_colors(&dis, false);
        assert_eq!(
            bg,
            color::mul(lerp_rgb([44, 57, 105, 178], [80, 80, 80, 255], 0.5), 0.7)
        );
        assert_eq!(border, [88, 88, 88, 178], "(127)*0.7 = 88.9 → 88");
    }

    /// `MatchesSearch` (`:1373-1380`) + the title/description builders
    /// (`:626-655`): whitespace search matches all, case-insensitive
    /// substring; the description's line order and gates.
    #[test]
    fn search_titles_and_descriptions_match_the_source_strings() {
        assert!(matches_search("Generate Ice Biome", "  "));
        assert!(matches_search("Generate Ice Biome", " ICE "));
        assert!(!matches_search("Generate Ice Biome", "lava"));
        let run = PassState {
            enabled: true,
            matches_search: true,
            ..Default::default()
        };
        assert_eq!(pass_title("Terrain", &run), "Run to Terrain");
        let done = PassState {
            completed: true,
            ..run
        };
        assert_eq!(pass_title("Terrain", &done), "Rerun to Terrain");
        let skip = PassState {
            skipped: true,
            ..run
        };
        assert_eq!(pass_title("Terrain", &skip), "Skipped: Terrain");
        let off = PassState {
            enabled: false,
            ..run
        };
        assert_eq!(pass_title("Terrain", &off), "Disabled: Terrain");
        // Fresh enabled pass, actions allowed: shift line + right-click +
        // ranges + highlight (`:644`, `:652-654`).
        assert_eq!(
            pass_description(&run, true, false),
            "Hold shift to ignore snapshots\nRight click to disable\nShift to edit ranges\nAlt click to toggle highlight\n"
        );
        // Completed + locked: the orange pause line, no click line (`:647`).
        assert_eq!(
            pass_description(&done, false, false),
            "Hold shift to ignore snapshots\n[c/FFA500:Must be paused to rerun or load snapshots]\nAlt click to toggle highlight\n"
        );
        // Disabled fresh pass: left-click-to-enable, no shift line (`:652`).
        assert_eq!(
            pass_description(&off, true, false),
            "Left click to enable\nShift to edit ranges\nAlt click to toggle highlight\n"
        );
    }
}
