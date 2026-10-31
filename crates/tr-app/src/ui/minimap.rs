//! The minimap frame chrome: the `MinimapFrame` / `MinimapFrameManager` port
//! (`GameContent/UI/Minimap/`) — the geometry table of the nine vanilla frame
//! styles, the zoom-button hitboxes and the anchor / `Main.mH` maths around
//! them. Pure geometry only: the two draw passes and the input side land in the
//! later whole-UI P4 tasks.
//!
//! Sheet sizes are never hard-coded — the port reads `SheetRect::width/height`
//! from the decoded sheets exactly like `Button.Size` reads the texture
//! (`MinimapFrame.cs:24`); the only pixel literals here are C# ones.

/// One entry of `MinimapFrameManager.PopulateOptionsAndLoadContent`
/// (`MinimapFrameManager.cs:23-33`), in dictionary insertion order (= the
/// `CycleSelection` cycle order — `CycleSelection` picks the entry *preceding*
/// the active one, wrapping to `.Last()`, `SelectionHolder.cs:48-65`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FrameStyle {
    /// `CreateAndAdd`'s 1st arg; also the `MinimapFrame` config-key value.
    pub name: &'static str,
    /// 2nd arg -> `MinimapFrame._frameOffset`.
    pub frame_offset: [f32; 2],
    /// 3rd arg: the reset button, raw frame-relative px.
    pub reset: [f32; 2],
    /// 4th arg: the zoom-in button.
    pub zoom_in: [f32; 2],
    /// 5th arg: the zoom-out button.
    pub zoom_out: [f32; 2],
}

/// The nine styles in `CreateAndAdd` order (`MinimapFrameManager.cs:25-33`).
/// `Default`'s raw button positions are the source's `(148f + num, 234f + num2)`
/// forms with `num = 2f`, `num2 = 6f` (`:23-24`), evaluated.
pub static FRAME_STYLES: [FrameStyle; 9] = [
    FrameStyle {
        name: "Default",
        frame_offset: [-8.0, -15.0],
        reset: [150.0, 240.0],
        zoom_in: [202.0, 240.0],
        zoom_out: [176.0, 240.0],
    }, // `MinimapFrameManager.cs:25` (148+2, 234+6)
    FrameStyle {
        name: "Golden",
        frame_offset: [-10.0, -10.0],
        reset: [136.0, 248.0],
        zoom_in: [96.0, 248.0],
        zoom_out: [116.0, 248.0],
    }, // `:26`
    FrameStyle {
        name: "Remix",
        frame_offset: [-10.0, -10.0],
        reset: [200.0, 234.0],
        zoom_in: [148.0, 234.0],
        zoom_out: [174.0, 234.0],
    }, // `:27`
    FrameStyle {
        name: "Sticks",
        frame_offset: [-10.0, -10.0],
        reset: [148.0, 234.0],
        zoom_in: [200.0, 234.0],
        zoom_out: [174.0, 234.0],
    }, // `:28`
    FrameStyle {
        name: "StoneGold",
        frame_offset: [-15.0, -15.0],
        reset: [220.0, 244.0],
        zoom_in: [244.0, 188.0],
        zoom_out: [244.0, 216.0],
    }, // `:29`
    FrameStyle {
        name: "TwigLeaf",
        frame_offset: [-20.0, -20.0],
        reset: [206.0, 242.0],
        zoom_in: [162.0, 242.0],
        zoom_out: [184.0, 242.0],
    }, // `:30`
    FrameStyle {
        name: "Leaf",
        frame_offset: [-20.0, -20.0],
        reset: [212.0, 244.0],
        zoom_in: [168.0, 246.0],
        zoom_out: [190.0, 246.0],
    }, // `:31`
    FrameStyle {
        name: "Retro",
        frame_offset: [-10.0, -10.0],
        reset: [150.0, 236.0],
        zoom_in: [202.0, 236.0],
        zoom_out: [176.0, 236.0],
    }, // `:32`
    FrameStyle {
        name: "Valkyrie",
        frame_offset: [-10.0, -10.0],
        reset: [154.0, 242.0],
        zoom_in: [206.0, 240.0],
        zoom_out: [180.0, 244.0],
    }, // `:33`
];

/// Button list index order = the source's draw/hit-test order
/// (`MinimapFrame.cs:189-191,197-208`): zoom-in, zoom-out, reset.
pub const BUTTON_ZOOM_IN: usize = 0;
/// `MinimapFrame.cs:190,201`.
pub const BUTTON_ZOOM_OUT: usize = 1;
/// `MinimapFrame.cs:191,205`.
pub const BUTTON_RESET: usize = 2;

/// One of the three frame buttons, resolved by `button_under_mouse`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MinimapButton {
    ZoomIn,
    ZoomOut,
    Reset,
}

/// `MinimapFrameManager.Configuration_OnLoad` (`:13`): config key with fallback
/// `"Default"`; unknown names fall back to index 0 (`SetActiveFrame(string)`
/// misses -> `Options.Values.First()`, `SelectionHolder.cs:71-78`).
pub fn style_index(name: &str) -> usize {
    FRAME_STYLES
        .iter()
        .position(|s| s.name == name)
        .unwrap_or(0)
}

/// `Main.UpdateMinimapAnchors` (`Main.cs:55275-55276`):
/// `(52 + (int)(240f * MapScale), 90)`. (`:55271-55274` are dead overwrites.)
pub fn minimap_anchor(map_scale: f32) -> (i32, i32) {
    (
        // `Main._minimapTopRightAnchorOffsetTowardsLeft = 52 + (int)(240f * Main.MapScale);`
        52 + (240.0f32 * map_scale) as i32,
        // `Main._minimapTopRightAnchorOffsetTowardsBottom = 90;`
        90,
    )
}

/// `Main.cs:55508-55509` at `MapScale = 1` (the port's reference, see the
/// whole-UI P4 plan's Uncertainty 5 ruling): `(screen.x - anchor.0 as f32,
/// anchor.1 as f32)` — `miniMapX = screenWidth - anchorLeft`,
/// `miniMapY = anchorBottom` (the `*= num16` fixups are no-ops at `MapScale = 1`).
pub fn minimap_origin(screen: glam::Vec2) -> [f32; 2] {
    let (left, bottom) = minimap_anchor(1.0);
    [screen.x - left as f32, bottom as f32]
}

/// `Main.cs:55546`: `DrawTo` position = origin + (-6 + 10, -6 + 10) — this is
/// what `MinimapFrame.MinimapPosition` is set to (`MinimapFrameManager.cs:44`).
pub fn draw_to_position(origin: [f32; 2]) -> [f32; 2] {
    [
        origin[0] - 6.0 + 10.0, // `num3 - 6f + 10f`
        origin[1] - 6.0 + 10.0, // `num4 - 6f + 10f`
    ]
}

/// `MinimapFrame.cs:81-91`: `MinimapPosition + _frameOffset`.
pub fn frame_position(minimap_pos: [f32; 2], style: &FrameStyle) -> [f32; 2] {
    [
        minimap_pos[0] + style.frame_offset[0],
        minimap_pos[1] + style.frame_offset[1],
    ]
}

/// `MinimapFrame.cs:182`:
/// `[(int)MinimapPosition.X - 6, (int)MinimapPosition.Y - 6, 244, 244]`.
pub fn backdrop_rect(minimap_pos: [f32; 2]) -> [f32; 4] {
    [
        (minimap_pos[0] as i32 - 6) as f32, // `(int)this.MinimapPosition.X - 6`
        (minimap_pos[1] as i32 - 6) as f32, // `(int)this.MinimapPosition.Y - 6`
        244.0,                              // the C# literal (the 240 panel + 2 * 4)
        244.0,
    ]
}

/// `MinimapFrame.cs:46-56`: `(center, half_extent)` of one button whose
/// top-left is `button_pos + frame_pos` and whose sheet size is `size`.
/// `half_extent = Max(size, (22, 22)) * 0.5` (the 22 is the C# literal).
pub fn button_hitbox(
    button_pos: [f32; 2],
    frame_pos: [f32; 2],
    size: [f32; 2],
) -> ([f32; 2], [f32; 2]) {
    let center = [
        // `this._position + parentPosition + this.Size * 0.5f` (`:48`)
        button_pos[0] + frame_pos[0] + size[0] * 0.5,
        button_pos[1] + frame_pos[1] + size[1] * 0.5,
    ];
    let half_extent = [
        // `Vector2.Max(this.Size, new Vector2(22f, 22f)) * 0.5f` (`:49`)
        size[0].max(22.0) * 0.5,
        size[1].max(22.0) * 0.5,
    ];
    (center, half_extent)
}

/// `MinimapFrame.GetButtonUnderMouse` (`:194-210`): the pointer against each
/// hitbox in the source's order (zoom-in, zoom-out, reset); `None` outside.
/// `sizes` is indexed by `BUTTON_*`; `[0, 0]` = sheet missing (the 22 px floor
/// of `:49` still applies, and nothing draws).
pub fn button_under_mouse(
    mouse: glam::Vec2,
    frame_pos: [f32; 2],
    style: &FrameStyle,
    sizes: [[f32; 2]; 3], // indexed by BUTTON_*; `[0,0]` = sheet missing
) -> Option<MinimapButton> {
    for (index, button, pos) in [
        (BUTTON_ZOOM_IN, MinimapButton::ZoomIn, style.zoom_in),
        (BUTTON_ZOOM_OUT, MinimapButton::ZoomOut, style.zoom_out),
        (BUTTON_RESET, MinimapButton::Reset, style.reset),
    ] {
        let (center, half) = button_hitbox(pos, frame_pos, sizes[index]);
        // `Math.Abs(vector3.X) < vector2.X && Math.Abs(vector3.Y) < vector2.Y`
        // (`:50-55`, strict on both axes).
        if (mouse.x - center[0]).abs() < half[0] && (mouse.y - center[1]).abs() < half[1] {
            return Some(button);
        }
    }
    None
}

/// `MinimapFrame.cs:59`.
pub const DEFAULT_ZOOM: f32 = 1.05;
/// `MinimapFrame.cs:61`.
pub const ZOOM_OUT_MULTIPLIER: f32 = 0.975;
/// `MinimapFrame.cs:63`.
pub const ZOOM_IN_MULTIPLIER: f32 = 1.025;

/// `Main.cs:55515-55522`: `Clamp(scale, 0.2, 3)`.
pub fn clamp_minimap_scale(scale: f32) -> f32 {
    scale.clamp(0.2, 3.0)
}

/// `Main.RecommendedEquipmentAreaPushUp` (`Main.cs:2967-2977`): `600` without
/// the Demon Heart / master-mode accessory rows (the port has neither, and
/// `PushEquipmentAreaUp` stays off).
const RECOMMENDED_EQUIPMENT_AREA_PUSH_UP: f32 = 600.0;

/// `Main.DrawInterface_16_MapOrMinimap` (`Main.cs:45771-45798`): `0`, or `256`
/// while the minimap HUD is live, clamped so `mH + 600 <= screen_h`
/// (`RecommendedEquipmentAreaPushUp`, `Main.cs:2967-2977`).
///
/// The `false` branch merges the source's `!mapEnabled` early return (taken
/// before the clamp, `:45774-45776`) with the style-gated `mH = 0` fall-through;
/// clamping a zero `mH` only moves below a 600 px screen, which the port's
/// reference resolutions never reach.
pub fn main_m_h(minimap_live: bool, screen_h: f32) -> f32 {
    if !minimap_live {
        return 0.0; // `Main.mH = 0` (`Main.cs:45772`)
    }
    let m_h = 256.0; // `Main.mH = 256` (`Main.cs:45780`)
    // `if (Main.mH + this.RecommendedEquipmentAreaPushUp > Main.screenHeight)
    //      Main.mH = Main.screenHeight - this.RecommendedEquipmentAreaPushUp;`
    // (`Main.cs:45795-45798`).
    if m_h + RECOMMENDED_EQUIPMENT_AREA_PUSH_UP > screen_h {
        return screen_h - RECOMMENDED_EQUIPMENT_AREA_PUSH_UP;
    }
    m_h
}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::Vec2;

    // Oracles from `MinimapFrameManager.cs:23-33`, `MinimapFrame.cs:24-210`,
    // `Main.cs:45771-45798`, `Main.cs:55267-55277`, `Main.cs:55503-55547` and
    // the whole-UI P4 plan's Task 1 test table — never from this code.

    #[test]
    fn frame_styles_match_the_source_table() {
        let expected: [(&str, [f32; 2], [f32; 2], [f32; 2], [f32; 2]); 9] = [
            // (name, frame_offset, reset, zoom_in, zoom_out).
            ("Default", [-8.0, -15.0], [150.0, 240.0], [202.0, 240.0], [176.0, 240.0]), // :25
            ("Golden", [-10.0, -10.0], [136.0, 248.0], [96.0, 248.0], [116.0, 248.0]), // :26
            ("Remix", [-10.0, -10.0], [200.0, 234.0], [148.0, 234.0], [174.0, 234.0]), // :27
            ("Sticks", [-10.0, -10.0], [148.0, 234.0], [200.0, 234.0], [174.0, 234.0]), // :28
            ("StoneGold", [-15.0, -15.0], [220.0, 244.0], [244.0, 188.0], [244.0, 216.0]), // :29
            ("TwigLeaf", [-20.0, -20.0], [206.0, 242.0], [162.0, 242.0], [184.0, 242.0]), // :30
            ("Leaf", [-20.0, -20.0], [212.0, 244.0], [168.0, 246.0], [190.0, 246.0]), // :31
            ("Retro", [-10.0, -10.0], [150.0, 236.0], [202.0, 236.0], [176.0, 236.0]), // :32
            ("Valkyrie", [-10.0, -10.0], [154.0, 242.0], [206.0, 240.0], [180.0, 244.0]), // :33
        ];
        assert_eq!(FRAME_STYLES.len(), expected.len());
        for (style, (name, frame_offset, reset, zoom_in, zoom_out)) in
            FRAME_STYLES.iter().zip(expected)
        {
            assert_eq!(style.name, name);
            assert_eq!(style.frame_offset, frame_offset, "{name}: frame_offset");
            assert_eq!(style.reset, reset, "{name}: reset");
            assert_eq!(style.zoom_in, zoom_in, "{name}: zoom_in");
            assert_eq!(style.zoom_out, zoom_out, "{name}: zoom_out");
        }
    }

    #[test]
    fn anchors_origin_and_draw_to_match_the_reference_frame() {
        // `Main.cs:55275-55276` at MapScale = 1 and the slider floor 0.5.
        assert_eq!(minimap_anchor(1.0), (292, 90));
        assert_eq!(minimap_anchor(0.5).0, 172);
        // `Main.cs:55508-55509` at 1600x900, MapScale = 1.
        let origin = minimap_origin(Vec2::new(1600.0, 900.0));
        assert_eq!(origin, [1308.0, 90.0]);
        // `Main.cs:55546`: `num3 - 6f + 10f` / `num4 - 6f + 10f`.
        assert_eq!(draw_to_position(origin), [1312.0, 94.0]);
    }

    #[test]
    fn default_frame_position_and_backdrop() {
        let style = &FRAME_STYLES[style_index("Default")];
        // `(1312, 94) + (-8, -15)` (`MinimapFrame.cs:81-91`).
        assert_eq!(frame_position([1312.0, 94.0], style), [1304.0, 79.0]);
        // `MinimapFrame.cs:182`.
        assert_eq!(backdrop_rect([1312.0, 94.0]), [1306.0, 88.0, 244.0, 244.0]);
    }

    #[test]
    fn default_buttons_abs_top_lefts_and_hitboxes() {
        let style = &FRAME_STYLES[0];
        let frame_pos = frame_position([1312.0, 94.0], style);
        assert_eq!(frame_pos, [1304.0, 79.0]);
        let sizes = [[18.0, 18.0]; 3]; // the decoded Default button sheets
        // Raw + FramePosition (`MinimapFrameManager.cs:25`).
        let abs = |p: [f32; 2]| [p[0] + frame_pos[0], p[1] + frame_pos[1]];
        assert_eq!(abs(style.reset), [1454.0, 319.0]);
        assert_eq!(abs(style.zoom_in), [1506.0, 319.0]);
        assert_eq!(abs(style.zoom_out), [1480.0, 319.0]);
        // `Max((18,18),(22,22)) * 0.5 = (11, 11)` (`MinimapFrame.cs:49`).
        for (button, pos, center) in [
            (MinimapButton::Reset, style.reset, [1463.0, 328.0]),
            (MinimapButton::ZoomIn, style.zoom_in, [1515.0, 328.0]),
            (MinimapButton::ZoomOut, style.zoom_out, [1489.0, 328.0]),
        ] {
            let (c, half) = button_hitbox(pos, frame_pos, sizes[BUTTON_ZOOM_IN]);
            assert_eq!(c, center);
            assert_eq!(half, [11.0, 11.0]);
            // The point resolves to the button; centre - (10, 10) is the last
            // inside corner; centre + 11 px on an axis is outside (strict `<`).
            assert_eq!(
                button_under_mouse(Vec2::from(center), frame_pos, style, sizes),
                Some(button)
            );
            assert_eq!(
                button_under_mouse(
                    Vec2::new(center[0] - 10.0, center[1] - 10.0),
                    frame_pos,
                    style,
                    sizes
                ),
                Some(button)
            );
            assert_eq!(
                button_under_mouse(
                    Vec2::new(center[0] + 11.0, center[1]),
                    frame_pos,
                    style,
                    sizes
                ),
                None
            );
        }
        // The plan's explicit bounds — nominal rectangles Reset
        // (1452,317)-(1474,339), ZoomIn (1504,317)-(1526,339),
        // ZoomOut (1478,317)-(1500,339). The source's test is strict `<`
        // (`:51-53`), so the edges themselves are outside; interior points
        // resolve to their button.
        assert_eq!(
            button_under_mouse(Vec2::new(1452.5, 317.5), frame_pos, style, sizes),
            Some(MinimapButton::Reset)
        );
        assert_eq!(
            button_under_mouse(Vec2::new(1473.5, 338.5), frame_pos, style, sizes),
            Some(MinimapButton::Reset)
        );
        assert_eq!(
            button_under_mouse(Vec2::new(1504.5, 317.5), frame_pos, style, sizes),
            Some(MinimapButton::ZoomIn)
        );
        assert_eq!(
            button_under_mouse(Vec2::new(1478.5, 317.5), frame_pos, style, sizes),
            Some(MinimapButton::ZoomOut)
        );
    }

    #[test]
    fn hitbox_floor_22px_applies_below_it() {
        // A Valkyrie-like synthetic size under the floor still yields the
        // (11, 11) half-extent (`MinimapFrame.cs:49`).
        let (center, half) = button_hitbox([0.0, 0.0], [0.0, 0.0], [12.0, 4.0]);
        assert_eq!(half, [11.0, 11.0]);
        assert_eq!(center, [6.0, 2.0]);
        let style = FrameStyle {
            name: "Synthetic",
            frame_offset: [0.0; 2],
            reset: [500.0, 500.0],
            zoom_in: [0.0; 2],
            zoom_out: [500.0, 500.0],
        };
        let sizes = [[12.0, 4.0]; 3];
        // 23 px right of the centre is outside (`MinimapFrame.cs:50-55`).
        assert_eq!(
            button_under_mouse(Vec2::new(6.0 + 23.0, 2.0), [0.0; 2], &style, sizes),
            None
        );
        // ... while 10.9 px right of it is inside.
        assert_eq!(
            button_under_mouse(Vec2::new(6.0 + 10.9, 2.0), [0.0; 2], &style, sizes),
            Some(MinimapButton::ZoomIn)
        );
    }

    #[test]
    fn button_under_mouse_resolves_in_the_source_order() {
        let sizes = [[22.0, 22.0]; 3];
        // A synthetic layout where the zoom-in and zoom-out hitboxes coincide:
        // the source checks zoom-in first (`MinimapFrame.cs:197-208`).
        let overlap = FrameStyle {
            name: "Synthetic",
            frame_offset: [0.0; 2],
            reset: [500.0, 500.0],
            zoom_in: [0.0; 2],
            zoom_out: [0.0; 2],
        };
        assert_eq!(
            button_under_mouse(Vec2::new(11.0, 11.0), [0.0; 2], &overlap, sizes),
            Some(MinimapButton::ZoomIn)
        );
        // A point inside only zoom-out resolves zoom-out.
        let spread = FrameStyle {
            name: "Synthetic",
            frame_offset: [0.0; 2],
            reset: [500.0, 500.0],
            zoom_in: [0.0; 2],
            zoom_out: [100.0, 0.0],
        };
        assert_eq!(
            button_under_mouse(Vec2::new(111.0, 11.0), [0.0; 2], &spread, sizes),
            Some(MinimapButton::ZoomOut)
        );
        assert_eq!(
            button_under_mouse(Vec2::new(50.0, 50.0), [0.0; 2], &spread, sizes),
            None
        );
    }

    #[test]
    fn missing_sheets_keep_the_floor_hitbox_without_panicking() {
        // `[0, 0]` sizes (a missing sheet, `Lookups::default()`): the C#
        // formula still yields the 22 px floor around the button's top-left
        // (`MinimapFrame.cs:48-49`) — no panic, and nothing would draw.
        let style = &FRAME_STYLES[0];
        let sizes = [[0.0, 0.0]; 3];
        assert_eq!(
            button_under_mouse(
                Vec2::new(1454.0 + 9.0, 319.0 + 9.0),
                [1304.0, 79.0],
                style,
                sizes
            ),
            Some(MinimapButton::Reset)
        );
    }

    #[test]
    fn zoom_maths_match_the_source_constants() {
        assert_eq!(DEFAULT_ZOOM, 1.05); // `MinimapFrame.cs:59`
        assert_eq!(ZOOM_OUT_MULTIPLIER, 0.975); // `:61`
        assert_eq!(ZOOM_IN_MULTIPLIER, 1.025); // `:63`
        // `ResetZoom` (`:107-110`), `ZoomInButton` (`:120-123`),
        // `ZoomOutButton` (`:133-136`). The products are f32 maths, so the
        // decimal oracles compare with a tolerance.
        let zoom_in = |scale: f32| scale * ZOOM_IN_MULTIPLIER;
        let zoom_out = |scale: f32| scale * ZOOM_OUT_MULTIPLIER;
        let zoom_reset = || DEFAULT_ZOOM;
        assert!((zoom_in(1.05) - 1.07625).abs() < 1e-6);
        assert!((zoom_out(1.05) - 1.02375).abs() < 1e-6);
        assert_eq!(zoom_reset(), 1.05);
        // `Main.cs:55515-55522`.
        assert_eq!(clamp_minimap_scale(0.1), 0.2);
        assert_eq!(clamp_minimap_scale(5.0), 3.0);
        assert_eq!(clamp_minimap_scale(1.05), 1.05);
    }

    #[test]
    fn style_index_resolves_config_keys_with_the_default_fallback() {
        assert_eq!(style_index("Default"), 0);
        assert_eq!(style_index("Valkyrie"), 8);
        assert_eq!(style_index("Nope"), 0); // unknown -> `Options.Values.First()`
    }

    #[test]
    fn main_m_h_truth_table() {
        // 856 <= 900: no clamp (`Main.cs:45795-45798`).
        assert_eq!(main_m_h(true, 900.0), 256.0);
        assert_eq!(main_m_h(false, 900.0), 0.0);
        // 256 + 600 > 800: `Main.mH = 800 - 600`.
        assert_eq!(main_m_h(true, 800.0), 200.0);
    }
}
