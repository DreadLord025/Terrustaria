//! The capture-mode family — `Terraria.Graphics.Capture`
//! (`CaptureManager.cs`, `CaptureInterface.cs`, `CaptureCamera.cs`,
//! `CaptureSettings.cs`, `CaptureBiome.cs`). S1-E-17: the manager +
//! interface + camera plumbing, 1:1 on the state machines; the GPU
//! half (the chunk renders through `Main.DrawCapture`,
//! `Main.cs:56673-57066`) is the documented S1-G-13 render-target seam —
//! [`CaptureCamera::draw_tick`] drains the queue through the
//! [`CaptureRenderer`] trait whose GPU implementation lands with the
//! layer targets.
//!
//! The C# statics (`CaptureInterface.EdgeA/EdgeB/CameraLock/...`) live on
//! the one [`CaptureManager`] instance the App owns — the port's
//! `CaptureManager.Instance` (`CaptureManager.cs:10`, `:110-113`).

use crate::ui::color;

/// `CaptureSettings` (`CaptureSettings.cs:6-28`). `OutputName`'s
/// `DateTime.Now` stamp (`:24-28`) is the one non-deterministic field in
/// the family; the port keeps the same `"Capture yyyy-MM-dd HH_mm_ss"`
/// shape, stamped by the caller (`CaptureSettings::with_timestamp`).
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct CaptureSettings {
    /// `Area` (`:8`) — tile coordinates, inclusive width/height.
    pub area: CaptureRect,
    /// `UseScaling = true` (`:10`).
    pub use_scaling: bool,
    /// `OutputName` (`:12`) — the ctor stamps local time (`:24-28`).
    pub output_name: String,
    /// `CaptureEntities = true` (`:14`).
    pub capture_entities: bool,
    /// `Biome = CaptureBiome.DefaultPurity` (`:16`).
    pub biome: CaptureBiome,
    /// `CaptureMech` (`:18`).
    pub capture_mech: bool,
    /// `CaptureBackground` (`:20`).
    pub capture_background: bool,
    /// `CameraSpaceEffects` (`:22`) — `StartCamera` sets it from the
    /// full-screen containment (`CaptureInterface.cs:1591`).
    pub camera_space_effects: bool,
}

impl Default for CaptureSettings {
    /// The field initializers (`CaptureSettings.cs:10-22`); the ctor's
    /// timestamp stamp is `with_timestamp` (the tests pin the shape, not
    /// the wall clock).
    fn default() -> Self {
        CaptureSettings {
            area: CaptureRect::default(),
            use_scaling: true,
            output_name: String::new(),
            capture_entities: true,
            biome: CaptureBiome::DEFAULT_PURITY,
            capture_mech: false,
            capture_background: false,
            camera_space_effects: false,
        }
    }
}

impl CaptureSettings {
    /// The ctor's `OutputName` stamp (`CaptureSettings.cs:24-28`):
    /// `"Capture " + yyyy-MM-dd + " " + HH_mm_ss`, local time, D4/D2 pads.
    pub(crate) fn with_timestamp(
        year: i32,
        month: u32,
        day: u32,
        hour: u32,
        minute: u32,
        second: u32,
    ) -> Self {
        CaptureSettings {
            output_name: format!(
                "Capture {year:04}-{month:02}-{day:02} {hour:02}_{minute:02}_{second:02}"
            ),
            ..Self::default()
        }
    }
}

/// `Microsoft.Xna.Framework.Rectangle` — the capture family's area type
/// (tile coordinates for the selection, pixels for the chunk blits).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct CaptureRect {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

impl CaptureRect {
    pub(crate) const fn new(x: i32, y: i32, width: i32, height: i32) -> Self {
        CaptureRect { x, y, width, height }
    }

    /// `Rectangle.Contains(Point)` — half-open on the right/bottom edges.
    pub(crate) fn contains(&self, x: i32, y: i32) -> bool {
        self.x <= x && x < self.x + self.width && self.y <= y && y < self.y + self.height
    }

    /// `Rectangle.Intersect` (`System.Drawing`-style): the overlap rect,
    /// empty (zero width/height at the origin) when disjoint.
    pub(crate) fn intersect(&self, other: &CaptureRect) -> CaptureRect {
        let x1 = self.x.max(other.x);
        let y1 = self.y.max(other.y);
        let x2 = (self.x + self.width).min(other.x + other.width);
        let y2 = (self.y + self.height).min(other.y + other.height);
        if x2 > x1 && y2 > y1 {
            CaptureRect::new(x1, y1, x2 - x1, y2 - y1)
        } else {
            CaptureRect::default()
        }
    }
}

/// `CaptureBiome.TileColorStyle` (`CaptureBiome.cs:5-12`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum TileColorStyle {
    #[default]
    Normal,
    Jungle,
    Crimson,
    Corrupt,
    Mushroom,
}

/// `CaptureBiome` (`CaptureBiome.cs:94-109`): the background/water/tile
/// colour triple a capture renders with. The `Styles` table is the const
/// twin of the static ctor (`:56-74`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct CaptureBiome {
    pub water_style: i32,
    pub background_index: i32,
    pub tile_color: TileColorStyle,
}

impl CaptureBiome {
    /// `new CaptureBiome(backgroundIndex, waterStyle, tileColorStyle)`
    /// (`CaptureBiome.cs:104-109`).
    pub(crate) const fn new(
        background_index: i32,
        water_style: i32,
        tile_color: TileColorStyle,
    ) -> Self {
        CaptureBiome { water_style, background_index, tile_color }
    }

    /// `Styles.Purity` (`CaptureBiome.cs:58`).
    pub(crate) const PURITY: CaptureBiome = CaptureBiome::new(0, 0, TileColorStyle::Normal);
    /// `Styles.Purity2` (`:59`).
    pub(crate) const PURITY2: CaptureBiome = CaptureBiome::new(10, 0, TileColorStyle::Normal);
    /// `Styles.Purity3` (`:60`).
    pub(crate) const PURITY3: CaptureBiome = CaptureBiome::new(11, 0, TileColorStyle::Normal);
    /// `Styles.Purity4` (`:61`).
    pub(crate) const PURITY4: CaptureBiome = CaptureBiome::new(12, 0, TileColorStyle::Normal);
    /// `Styles.Corruption` (`:62`).
    pub(crate) const CORRUPTION: CaptureBiome = CaptureBiome::new(1, 2, TileColorStyle::Corrupt);
    /// `Styles.Jungle` (`:63`).
    pub(crate) const JUNGLE: CaptureBiome = CaptureBiome::new(3, 3, TileColorStyle::Jungle);
    /// `Styles.Hallow` (`:64`).
    pub(crate) const HALLOW: CaptureBiome = CaptureBiome::new(6, 4, TileColorStyle::Normal);
    /// `Styles.Snow` (`:65`).
    pub(crate) const SNOW: CaptureBiome = CaptureBiome::new(7, 5, TileColorStyle::Normal);
    /// `Styles.Desert` (`:66`).
    pub(crate) const DESERT: CaptureBiome = CaptureBiome::new(2, 6, TileColorStyle::Normal);
    /// `Styles.DirtLayer` (`:67`).
    pub(crate) const DIRT_LAYER: CaptureBiome = CaptureBiome::new(0, 7, TileColorStyle::Normal);
    /// `Styles.RockLayer` (`:68`).
    pub(crate) const ROCK_LAYER: CaptureBiome = CaptureBiome::new(0, 8, TileColorStyle::Normal);
    /// `Styles.BloodMoon` (`:69`).
    pub(crate) const BLOOD_MOON: CaptureBiome = CaptureBiome::new(0, 9, TileColorStyle::Normal);
    /// `Styles.Crimson` (`:70`).
    pub(crate) const CRIMSON: CaptureBiome = CaptureBiome::new(8, 10, TileColorStyle::Crimson);
    /// `Styles.UndergroundDesert` (`:71`).
    pub(crate) const UNDERGROUND_DESERT: CaptureBiome =
        CaptureBiome::new(2, 12, TileColorStyle::Normal);
    /// `Styles.Ocean` (`:72`).
    pub(crate) const OCEAN: CaptureBiome = CaptureBiome::new(4, 13, TileColorStyle::Normal);
    /// `Styles.Mushroom` (`:73`).
    pub(crate) const MUSHROOM: CaptureBiome = CaptureBiome::new(9, 7, TileColorStyle::Mushroom);
    /// `DefaultPurity` (`:94`, `:226`).
    pub(crate) const DEFAULT_PURITY: CaptureBiome = CaptureBiome::new(0, 0, TileColorStyle::Normal);

    /// `BiomesByWaterStyle` (`CaptureBiome.cs:227-244`) — the 15-slot
    /// table, `None` where the C# parks null (0, 1, 11).
    pub(crate) const BIOMES_BY_WATER_STYLE: [Option<CaptureBiome>; 15] = [
        None,
        None,
        Some(Self::CORRUPTION),
        Some(Self::JUNGLE),
        Some(Self::HALLOW),
        Some(Self::SNOW),
        Some(Self::DESERT),
        Some(Self::DIRT_LAYER),
        Some(Self::ROCK_LAYER),
        Some(Self::BLOOD_MOON),
        Some(Self::CRIMSON),
        None,
        Some(Self::UNDERGROUND_DESERT),
        Some(Self::OCEAN),
        Some(Self::MUSHROOM),
    ];

    /// `GetBiomeByWater` (`CaptureBiome.cs:156-168`): the first table slot
    /// whose `WaterStyle` matches `Main.CalculateWaterStyle(ignoreFountains:
    /// true)` — the port passes the resolved style in (the App's
    /// `biome::calculate_water_style`).
    pub(crate) fn biome_by_water(water_style: i32) -> Option<CaptureBiome> {
        Self::BIOMES_BY_WATER_STYLE
            .iter()
            .flatten()
            .find(|b| b.water_style == water_style)
            .copied()
    }

    /// `GetBiomeByLocation` (`CaptureBiome.cs:170-204`): the
    /// `Main.GetPreferredBGStyleForPlayer` switch, verbatim arms.
    pub(crate) fn biome_by_location(bg_style: i32) -> Option<CaptureBiome> {
        match bg_style {
            0 => Some(Self::PURITY),
            10 => Some(Self::PURITY2),
            11 => Some(Self::PURITY3),
            12 => Some(Self::PURITY4),
            1 => Some(Self::CORRUPTION),
            2 | 5 | 13 | 14 => Some(Self::DESERT),
            3 => Some(Self::JUNGLE),
            4 => Some(Self::OCEAN),
            6 => Some(Self::HALLOW),
            7 => Some(Self::SNOW),
            8 => Some(Self::CRIMSON),
            9 => Some(Self::MUSHROOM),
            _ => None,
        }
    }

    /// `GetPurityForPlayer` (`CaptureBiome.cs:206-222`): the player's tile
    /// X against `Main.treeX[0..2]`. The App passes `World::tree_x` in
    /// (W4-g70): generated worlds keep the `WorldGen.Reset` maxTilesX
    /// boundaries (every player lands in Purity), loaded worlds the file's.
    pub(crate) fn purity_for_player(player_center_x: f32, tree_x: [i32; 3]) -> CaptureBiome {
        let num = (player_center_x / 16.0) as i32;
        if num < tree_x[0] {
            return Self::PURITY;
        }
        if num < tree_x[1] {
            return Self::PURITY2;
        }
        if num < tree_x[2] {
            return Self::PURITY3;
        }
        Self::PURITY4
    }

    /// `GetCaptureBiome` (`CaptureBiome.cs:111-154`): the explicit
    /// `BiomeChoiceIndex` arms, else the location → water → purity
    /// fall-through chain. The inputs are the three resolved probes the
    /// App owns (`preferred_bg_style`, `calculate_water_style`, the
    /// player centre + treeX).
    pub(crate) fn get_capture_biome(
        biome_choice: i32,
        bg_style: i32,
        water_style: i32,
        player_center_x: f32,
        tree_x: [i32; 3],
    ) -> CaptureBiome {
        match biome_choice {
            1 => Self::purity_for_player(player_center_x, tree_x), // Purity
            2 => Self::CORRUPTION,
            3 => Self::JUNGLE,
            4 => Self::HALLOW,
            5 => Self::SNOW,
            6 => Self::DESERT,
            7 => Self::DIRT_LAYER,
            8 => Self::ROCK_LAYER,
            9 => Self::CRIMSON,
            10 => Self::UNDERGROUND_DESERT,
            11 => Self::OCEAN,
            12 => Self::MUSHROOM,
            _ => {
                // `AutomatedForPlayer` (-1) and every other index: the
                // location probe, then the water probe, then purity.
                if let Some(b) = Self::biome_by_location(bg_style) {
                    return b;
                }
                if let Some(b) = Self::biome_by_water(water_style) {
                    return b;
                }
                Self::purity_for_player(player_center_x, tree_x)
            }
        }
    }
}

/// `CaptureInterface.Settings` (`CaptureInterface.cs:17-39`) — the static
/// settings block, ported as plain fields on the interface (the one
/// instance owns them). The ctor defaults (`:31-38`): PackImage true,
/// IncludeEntities true, BiomeChoiceIndex -1, ScreenAnchor 0,
/// MarkedAreaColor `new Color(0.8f, 0.8f, 0.8f, 0f) * 0.3f`.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct CaptureInterfaceSettings {
    pub pack_image: bool,
    pub include_entities: bool,
    pub transparent_background: bool,
    pub biome_choice_index: i32,
    pub screen_anchor: i32,
    /// XNA packed ABGR (`Color.PackedValue`): the ctor's
    /// `(0.8, 0.8, 0.8, 0) * 0.3` truncates every channel — R/G/B
    /// `(byte)(204 * 0.3) = 61`, alpha `(byte)(0 * 0.3) = 0`.
    pub marked_area_color: u32,
}

impl Default for CaptureInterfaceSettings {
    fn default() -> Self {
        CaptureInterfaceSettings {
            pack_image: true,
            include_entities: true,
            transparent_background: false,
            biome_choice_index: -1,
            screen_anchor: 0,
            // 0xFF000000 | (61 << 16) | (61 << 8) | 61 — ABGR, alpha 0.
            marked_area_color: 0x003D3D3D,
        }
    }
}

/// `CaptureInterface.SelectionContext` (`CaptureInterface.cs:68-72`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SelectionContext {
    World,
    Map,
}

/// The three selection modes (`CaptureInterface.FillModes`,
/// `CaptureInterface.cs:951-968`): 0 edge selection, 1 drag bounds,
/// 2 change settings.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CaptureMode {
    EdgeSelection = 0,
    DragBounds = 1,
    ChangeSettings = 2,
}

/// `ModeDragBounds`'s live drag state (`CaptureInterface.cs:342-350`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct DragBoundsState {
    /// `currentAim = -1` (`:344`) — the hovered/caught edge index 0..3.
    pub current_aim: i32,
    /// `dragging` (`:346`).
    pub dragging: bool,
    /// `caughtEdge = -1` (`:348`) — which pinned edge (0 = A, 1 = B) moves.
    pub caught_edge: i32,
    /// `inMap` (`:350`).
    pub in_map: bool,
}

/// `ModeChangeSettings`'s hover state (`CaptureInterface.cs:662-668`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct ChangeSettingsState {
    /// `hoveredButton = -1` (`:666`).
    pub hovered_button: i32,
    /// `inUI` (`:668`).
    pub in_ui: bool,
}

impl Default for ChangeSettingsState {
    fn default() -> Self {
        ChangeSettingsState { hovered_button: -1, in_ui: false }
    }
}

/// The per-frame input snapshot `CaptureInterface.Update` reads — the
/// `Main.mouseX/Y`, `Main.mouseLeft/Right`, `mouseLeftRelease`,
/// `Main.MouseWorld`, `PlayerInput.Triggers.Current.ToggleCameraMode`,
/// `Main.mapFullscreen` and the player gates (`:984`). The App fills it
/// once per update; the pure state machine below never touches the App.
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct CaptureInput {
    pub mouse_x: f32,
    pub mouse_y: f32,
    pub mouse_left: bool,
    pub mouse_right: bool,
    pub mouse_left_release: bool,
    /// `Main.MouseWorld` — world pixels.
    pub mouse_world_x: f32,
    pub mouse_world_y: f32,
    pub toggle_camera_mode: bool,
    pub map_fullscreen: bool,
    /// `Main.mouseItem.type == 0` (`:984`) — the held-item gate.
    pub mouse_item_empty: bool,
    /// `Main.player[Main.myPlayer].dead` / `.ghost` (`:984`).
    pub player_dead: bool,
    pub player_ghost: bool,
    /// `Main.player[Main.myPlayer].gravDir == -1f` (`:200`, `:426`).
    pub gravity_reversed: bool,
    /// `Main.graphics.IsFullScreen` (`:1064`) — the open-folder button gate.
    pub is_fullscreen: bool,
    /// `Main.mapEnabled` (`:1113`).
    pub map_enabled: bool,
    /// `WiresUI.Settings.DrawWires` (`:1088`) — CaptureMech's source.
    pub draw_wires: bool,
}

/// The world/screen probes the selection math needs — `Main.maxTilesX/Y`,
/// `Main.screenWidth/Height`, `Main.screenPosition` (world pixels).
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct CaptureWorldInfo {
    pub max_tiles_x: i32,
    pub max_tiles_y: i32,
    pub screen_width: i32,
    pub screen_height: i32,
    pub screen_position_x: f32,
    pub screen_position_y: f32,
}

/// `CaptureInterface` (`CaptureInterface.cs:15-1607`) — the selection UI's
/// state machine. The C# statics are fields here (the App owns the one
/// instance through [`CaptureManager`]).
#[derive(Clone, Debug)]
pub(crate) struct CaptureInterface {
    /// `Active` (`:895`).
    pub active: bool,
    /// `JustActivated` (`:897`) — the first-release latch (`:994-997`).
    pub just_activated: bool,
    /// `KeyToggleActiveHeld` (`:899`).
    key_toggle_active_held: bool,
    /// `SelectedMode` (`:901`).
    pub selected_mode: i32,
    /// `HoveredMode` (`:903`).
    pub hovered_mode: i32,
    /// `EdgeAPinned`/`EdgeBPinned` (`:905-907`).
    pub edge_a_pinned: bool,
    pub edge_b_pinned: bool,
    /// `EdgeA`/`EdgeB` (`:909-911`) — tile coordinates.
    pub edge_a: (i32, i32),
    pub edge_b: (i32, i32),
    /// `CameraLock` (`:913`).
    pub camera_lock: bool,
    /// `CameraFrame` (`:915`) — the 0..5 countdown.
    pub camera_frame: f32,
    /// `CameraWaiting` (`:917`) — the 0..60 wait counter.
    pub camera_waiting: f32,
    /// `CameraSettings` (`:923`).
    pub camera_settings: Option<CaptureSettings>,
    /// `_selectionContext` (`:891`).
    pub selection_context: SelectionContext,
    /// `Settings` (`:17-39`).
    pub settings: CaptureInterfaceSettings,
    /// `ModeDragBounds` (`:342-660`).
    pub drag_bounds: DragBoundsState,
    /// `ModeChangeSettings` (`:662-889`).
    pub change_settings: ChangeSettingsState,
}

impl Default for CaptureInterface {
    fn default() -> Self {
        CaptureInterface {
            active: false,
            just_activated: false,
            key_toggle_active_held: false,
            selected_mode: 0,
            hovered_mode: -1,
            edge_a_pinned: false,
            edge_b_pinned: false,
            edge_a: (0, 0),
            edge_b: (0, 0),
            camera_lock: false,
            camera_frame: 0.0,
            camera_waiting: 0.0,
            camera_settings: None,
            selection_context: SelectionContext::World,
            settings: CaptureInterfaceSettings::default(),
            drag_bounds: DragBoundsState { current_aim: -1, caught_edge: -1, ..Default::default() },
            change_settings: ChangeSettingsState::default(),
        }
    }
}

/// `CameraMaxFrame = 5f` / `CameraMaxWait = 60f`
/// (`CaptureInterface.cs:919-921`).
pub(crate) const CAMERA_MAX_FRAME: f32 = 5.0;
pub(crate) const CAMERA_MAX_WAIT: f32 = 60.0;

impl CaptureInterface {
    /// `GetArea` (`CaptureInterface.cs:970-973`): the min corner + the
    /// absolute span + 1 on each axis.
    pub(crate) fn get_area(&self) -> CaptureRect {
        CaptureRect::new(
            self.edge_a.0.min(self.edge_b.0),
            self.edge_a.1.min(self.edge_b.1),
            (self.edge_a.0 - self.edge_b.0).abs() + 1,
            (self.edge_a.1 - self.edge_b.1).abs() + 1,
        )
    }

    /// `ResetFocus` (`CaptureInterface.cs:1502-1508`).
    pub(crate) fn reset_focus(&mut self) {
        self.edge_a_pinned = false;
        self.edge_b_pinned = false;
        self.edge_a = (-1, -1);
        self.edge_b = (-1, -1);
    }

    /// `ToggleCamera` (`CaptureInterface.cs:1039-1059`): the CameraLock
    /// gate, the mode-validity AND, the sound-10/11 edge (returned for the
    /// caller — `SoundEngine.PlaySound(On ? 10 : 11)`), the per-mode
    /// `ToggleActive` side effects and the JustActivated latch.
    pub(crate) fn toggle_camera(&mut self, on: bool) -> Option<&'static str> {
        if self.camera_lock {
            return None;
        }
        let active = self.active;
        // `Modes.ContainsKey(SelectedMode) & On` — modes 0..2 exist.
        self.active = (0..=2).contains(&self.selected_mode) && on;
        let cue = if active != self.active {
            // `SoundEngine.PlaySound(On ? 10 : 11)` — Menu_Open/Menu_Close.
            Some(if on { "Menu_Open" } else { "Menu_Close" })
        } else {
            None
        };
        // The per-mode ToggleActive (`:1051-1054`): DragBounds clears its
        // aim when its mode deactivates (`:371-377`), ChangeSettings
        // clears the hover when it activates (`:877-883`).
        let drag_selected = self.active && self.selected_mode == 1;
        if !drag_selected {
            self.drag_bounds.current_aim = -1;
        }
        let settings_selected = self.active && self.selected_mode == 2;
        if settings_selected {
            self.change_settings.hovered_button = -1;
        }
        if on && !active {
            self.just_activated = true;
        }
        cue
    }

    /// `PointWorldClamp` (`CaptureInterface.cs:1473-1491`): the fluff-40
    /// world-boundary clamp on one pinned edge.
    fn point_world_clamp(point: &mut (i32, i32), fluff: i32, info: &CaptureWorldInfo) {
        if point.0 < fluff {
            point.0 = fluff;
        }
        if point.0 > info.max_tiles_x - 1 - fluff {
            point.0 = info.max_tiles_x - 1 - fluff;
        }
        if point.1 < fluff {
            point.1 = fluff;
        }
        if point.1 > info.max_tiles_y - 1 - fluff {
            point.1 = info.max_tiles_y - 1 - fluff;
        }
    }

    /// `ConstraintPoints` (`CaptureInterface.cs:1460-1471`): clamp both
    /// pinned edges with fluff 40.
    pub(crate) fn constraint_points(&mut self, info: &CaptureWorldInfo) {
        let fluff = 40;
        if self.edge_a_pinned {
            Self::point_world_clamp(&mut self.edge_a, fluff, info);
        }
        if self.edge_b_pinned {
            Self::point_world_clamp(&mut self.edge_b, fluff, info);
        }
    }

    /// `ModeEdgeSelection.EdgePlacement` (`CaptureInterface.cs:105-139`)
    /// minus the fullscreen-map arm (`GetMapCoords` needs the map renderer
    /// — the port's map-fullscreen capture selection is the documented
    /// gap; the world arm is 1:1): left pins A at `MouseWorld`'s tile,
    /// right pins B, then the constraint clamp.
    fn edge_placement(&mut self, input: &CaptureInput, info: &CaptureWorldInfo) {
        if self.just_activated {
            return;
        }
        if !input.map_fullscreen {
            if input.mouse_left {
                self.edge_a_pinned = true;
                self.edge_a = (
                    (input.mouse_world_x / 16.0) as i32,
                    (input.mouse_world_y / 16.0) as i32,
                );
            }
            if input.mouse_right {
                self.edge_b_pinned = true;
                self.edge_b = (
                    (input.mouse_world_x / 16.0) as i32,
                    (input.mouse_world_y / 16.0) as i32,
                );
            }
        }
        self.constraint_points(info);
    }

    /// `ModeDragBounds.DragBounds` (`CaptureInterface.cs:384-565`), the
    /// world arm (the fullscreen-map arm is the same documented gap as
    /// `edge_placement`). The initial both-edges pin (`:386-421`), the
    /// marked-area screen intersection (`:422-455`), the drag/aim/caught-
    /// edge machine (`:456-563`) and the constraint tail (`:564`).
    fn drag_bounds_update(&mut self, input: &CaptureInput, info: &CaptureWorldInfo) {
        if !self.edge_a_pinned || !self.edge_b_pinned {
            if input.mouse_left {
                if !input.map_fullscreen {
                    let point = (
                        ((info.screen_position_x + input.mouse_x) / 16.0) as i32,
                        ((info.screen_position_y + input.mouse_y) / 16.0) as i32,
                    );
                    if !self.edge_a_pinned {
                        self.edge_a_pinned = true;
                        self.edge_a = point;
                    }
                    if !self.edge_b_pinned {
                        self.edge_b_pinned = true;
                        self.edge_b = point;
                    }
                }
                self.drag_bounds.current_aim = 3;
                self.drag_bounds.caught_edge = 1;
            }
        }
        let num = self.edge_a.0.min(self.edge_b.0);
        let num2 = self.edge_a.1.min(self.edge_b.1);
        let num3 = (self.edge_a.0 - self.edge_b.0).abs();
        let num4 = (self.edge_a.1 - self.edge_b.1).abs();
        // `num5 = 1 - gravDir==-1`, `num6 = gravDir==-1` (`:426-428`).
        let value = input.gravity_reversed;
        let num5 = if value { 0 } else { 1 };
        let num6 = if value { 1 } else { 0 };
        // The world arm's marked-area/screen intersection (`:432-442`).
        // `ReverseGravitySupport` is the identity without reversed gravity
        // (the port's gravity potion is unported — the flag rides in for
        // the caught-edge pick only).
        let value2 = CaptureRect::new(num * 16, num2 * 16, (num3 + 1) * 16, (num4 + 1) * 16);
        let value3 = CaptureRect::new(
            info.screen_position_x as i32,
            info.screen_position_y as i32,
            info.screen_width + 1,
            info.screen_height + 1,
        );
        let mut result2 = value3.intersect(&value2);
        if result2.width == 0 || result2.height == 0 {
            return;
        }
        result2.x -= value3.x;
        result2.y -= value3.y;
        self.drag_bounds.dragging = false;
        if !input.mouse_left {
            self.drag_bounds.current_aim = -1;
        }
        if self.drag_bounds.current_aim != -1 {
            self.drag_bounds.dragging = true;
            if input.map_fullscreen {
                return; // the GetMapCoords gap — no drag in map mode
            }
            let point = (
                (input.mouse_world_x / 16.0) as i32,
                (input.mouse_world_y / 16.0) as i32,
            );
            match self.drag_bounds.current_aim {
                0 | 1 => {
                    if self.drag_bounds.caught_edge == 0 {
                        self.edge_a.1 = point.1;
                    }
                    if self.drag_bounds.caught_edge == 1 {
                        self.edge_b.1 = point.1;
                    }
                }
                2 | 3 => {
                    if self.drag_bounds.caught_edge == 0 {
                        self.edge_a.0 = point.0;
                    }
                    if self.drag_bounds.caught_edge == 1 {
                        self.edge_b.0 = point.0;
                    }
                }
                _ => {}
            }
        } else {
            self.drag_bounds.caught_edge = -1;
            // `drawbox = value2.Offset(-value3.X, -value3.Y)` (`:506-508`).
            let drawbox = CaptureRect::new(
                value2.x - value3.x,
                value2.y - value3.y,
                value2.width,
                value2.height,
            );
            let mx = input.mouse_x as i32;
            let my = input.mouse_y as i32;
            self.drag_bounds.in_map = drawbox.contains(mx, my);
            for i in 0..4 {
                let bound = Self::get_bound(&drawbox, i);
                // `bound.Inflate(8, 8)` (`:512`).
                let inflated =
                    CaptureRect::new(bound.x - 8, bound.y - 8, bound.width + 16, bound.height + 16);
                if !inflated.contains(mx, my) {
                    continue;
                }
                self.drag_bounds.current_aim = i;
                match i {
                    0 => {
                        self.drag_bounds.caught_edge =
                            if self.edge_a.1 < self.edge_b.1 { num6 } else { num5 };
                    }
                    1 => {
                        self.drag_bounds.caught_edge =
                            if self.edge_a.1 >= self.edge_b.1 { num6 } else { num5 };
                    }
                    2 => {
                        self.drag_bounds.caught_edge =
                            if self.edge_a.0 < self.edge_b.0 { 0 } else { 1 };
                    }
                    3 => {
                        self.drag_bounds.caught_edge =
                            if self.edge_a.0 >= self.edge_b.0 { 0 } else { 1 };
                    }
                    _ => {}
                }
                break;
            }
        }
        self.constraint_points(info);
    }

    /// `ModeDragBounds.GetBound` (`CaptureInterface.cs:567-577`): the four
    /// 2-px edge strips of the marked area.
    pub(crate) fn get_bound(drawbox: &CaptureRect, bound_index: i32) -> CaptureRect {
        match bound_index {
            0 => CaptureRect::new(drawbox.x, drawbox.y - 2, drawbox.width, 2),
            1 => CaptureRect::new(drawbox.x, drawbox.y + drawbox.height, drawbox.width, 2),
            2 => CaptureRect::new(drawbox.x - 2, drawbox.y, 2, drawbox.height),
            3 => CaptureRect::new(drawbox.x + drawbox.width, drawbox.y, 2, drawbox.height),
            _ => CaptureRect::default(),
        }
    }

    /// `ModeChangeSettings.GetRect` (`CaptureInterface.cs:670-679`): the
    /// 224x170 panel at (227 - 112, 80) for ScreenAnchor 0.
    pub(crate) fn settings_panel_rect(&self) -> CaptureRect {
        let mut result = CaptureRect::new(0, 0, 224, 170);
        if self.settings.screen_anchor == 0 {
            result.x = 227 - result.width / 2;
            result.y = 80;
        }
        result
    }

    /// `ModeChangeSettings.Update` (`CaptureInterface.cs:820-847`): the
    /// panel hover + the seven 16-px button rows at 20-px pitch, the
    /// left-release press. Returns the pressed button (the caller runs
    /// `PressButton`'s toggle + the sound-12 cue).
    fn change_settings_update(&mut self, input: &CaptureInput) -> Option<i32> {
        if self.just_activated {
            return None;
        }
        let value = (input.mouse_x as i32, input.mouse_y as i32);
        self.change_settings.hovered_button = -1;
        let rect = self.settings_panel_rect();
        self.change_settings.in_ui = rect.contains(value.0, value.1);
        // `rect.Inflate(-20, -20); rect.Height = 16` (`:831-832`).
        let row_x = rect.x + 20;
        let row_w = rect.width - 40;
        let y = rect.y + 20;
        for i in 0..7 {
            let row = CaptureRect::new(row_x, y + i * 20, row_w, 16);
            if row.contains(value.0, value.1) {
                self.change_settings.hovered_button = i;
                break;
            }
        }
        if input.mouse_left && input.mouse_left_release && self.change_settings.hovered_button != -1
        {
            return Some(self.change_settings.hovered_button);
        }
        None
    }

    /// `ModeChangeSettings.PressButton` (`CaptureInterface.cs:708-737`):
    /// the three toggles + the reset row; returns whether a change fired
    /// (the caller plays sound 12 — `Menu_Tick`).
    pub(crate) fn press_settings_button(&mut self, button: i32) -> bool {
        let mut flag = false;
        match button {
            0 => {
                self.settings.pack_image = !self.settings.pack_image;
                flag = true;
            }
            1 => {
                self.settings.include_entities = !self.settings.include_entities;
                flag = true;
            }
            2 => {
                self.settings.transparent_background = !self.settings.transparent_background;
                flag = true;
            }
            6 => {
                self.settings.pack_image = true;
                self.settings.include_entities = true;
                self.settings.transparent_background = false;
                self.settings.biome_choice_index = -1;
                flag = true;
            }
            _ => {}
        }
        flag
    }

    /// `UpdateButtons` (`CaptureInterface.cs:1061-1138`): the nine 42x42
    /// toolbar buttons at (24 + 46*i, 24). Returns `(hover_consumed,
    /// action)` — the C# returns true when the pointer is over a button
    /// (the mode update is skipped while the left button is held,
    /// `:998-1001`); the action is the pressed button's effect.
    fn update_buttons(&mut self, input: &CaptureInput) -> (bool, Option<ToolbarAction>) {
        self.hovered_mode = -1;
        let flag = !input.is_fullscreen;
        let mx = input.mouse_x as i32;
        let my = input.mouse_y as i32;
        for i in 0..9 {
            let rect = CaptureRect::new(24 + 46 * i, 24, 42, 42);
            if !rect.contains(mx, my) {
                continue;
            }
            self.hovered_mode = i;
            let pressed = input.mouse_left && input.mouse_left_release;
            let action = match i {
                0 if pressed => Some(ToolbarAction::QuickScreenshot),
                1 if pressed && self.edge_a_pinned && self.edge_b_pinned => {
                    Some(ToolbarAction::CaptureArea)
                }
                2 if pressed && self.selected_mode != 0 => Some(ToolbarAction::SelectMode(0)),
                3 if pressed && self.selected_mode != 1 => Some(ToolbarAction::SelectMode(1)),
                4 if pressed => Some(ToolbarAction::ResetFocus),
                5 if pressed && input.map_enabled => Some(ToolbarAction::ToggleMapFullscreen),
                6 if pressed && self.selected_mode != 2 => Some(ToolbarAction::SelectMode(2)),
                7 if pressed && flag => Some(ToolbarAction::OpenFolder),
                8 if pressed => Some(ToolbarAction::Close),
                _ => None,
            };
            return (true, action);
        }
        (false, None)
    }

    /// `Update` (`CaptureInterface.cs:975-1008`): the context latch, the
    /// CameraLock early return, the F4 toggle edge (`:983-988`), the
    /// JustActivated release latch (`:994-997`), the toolbar gate
    /// (`:998-1001`) and the selected mode's update. Returns the cues and
    /// toolbar actions for the App to run (the sound plays and the
    /// camera start are side effects the pure machine reports).
    pub(crate) fn update(
        &mut self,
        context: SelectionContext,
        input: &CaptureInput,
        info: &CaptureWorldInfo,
    ) -> CaptureUpdateEffects {
        let mut effects = CaptureUpdateEffects::default();
        self.selection_context = context;
        if self.camera_lock {
            return effects;
        }
        let toggle = input.toggle_camera_mode;
        if toggle
            && !self.key_toggle_active_held
            && (input.mouse_item_empty || self.active)
            && !input.player_dead
            && !input.player_ghost
        {
            // `Main.CaptureModeDisabled` is the manager's gate — the App
            // only calls update when capture mode is enabled.
            if let Some(cue) = self.toggle_camera(!self.active) {
                effects.cues.push(cue);
            }
        }
        self.key_toggle_active_held = toggle;
        if !self.active {
            return effects;
        }
        // `Main.blockMouse = true` (`:993`) — the App's pointer gate reads
        // `active` directly; no field needed here.
        if self.just_activated && input.mouse_left_release && !input.mouse_left {
            self.just_activated = false;
        }
        let (over_button, action) = self.update_buttons(input);
        if let Some(action) = action {
            effects.toolbar = Some(action);
        }
        if over_button && input.mouse_left {
            return effects;
        }
        // The selected mode's Update (`:1002-1006`).
        match self.selected_mode {
            0 => self.edge_placement(input, info),
            1 => {
                if !self.just_activated {
                    self.drag_bounds_update(input, info);
                }
            }
            2 => {
                if let Some(button) = self.change_settings_update(input) {
                    if self.press_settings_button(button) {
                        effects.cues.push("Menu_Tick"); // sound 12 (`:735`)
                    }
                }
            }
            _ => {}
        }
        effects
    }

    /// `UsingMap` (`CaptureInterface.cs:1493-1500`): CameraLock forces
    /// true, else the selected mode's answer — EdgeSelection always true
    /// (`:100-103`), DragBounds while an edge is caught (`:379-382`),
    /// ChangeSettings while the pointer is in the panel (`:885-888`).
    pub(crate) fn using_map(&self) -> bool {
        if self.camera_lock {
            return true;
        }
        match self.selected_mode {
            0 => true,
            1 => self.drag_bounds.caught_edge != -1,
            2 => self.change_settings.in_ui,
            _ => false,
        }
    }

    /// `Scrolling` (`CaptureInterface.cs:1510-1531`): the wheel cycles the
    /// three modes; returns whether the mode changed (the caller plays
    /// sound 12).
    pub(crate) fn scrolling(&mut self, scroll_wheel_delta: i32) -> bool {
        let mut num = scroll_wheel_delta / 120 % 30;
        if num < 0 {
            num += 30;
        }
        let selected = self.selected_mode;
        self.selected_mode -= num;
        while self.selected_mode < 0 {
            self.selected_mode += 2;
        }
        while self.selected_mode > 2 {
            self.selected_mode -= 2;
        }
        self.selected_mode != selected
    }

    /// `UpdateCameraCountdown` (`CaptureInterface.cs:1533-1556`): at frame
    /// 4 the capture starts (returned as the settings to hand
    /// `CaptureManager.Capture`), the frame counts up while locked (clamped
    /// 0..5), the wait counter runs at frame 5 (clamped 60).
    pub(crate) fn update_camera_countdown(&mut self) -> Option<CaptureSettings> {
        let mut start = None;
        if self.camera_lock && self.camera_frame == 4.0 {
            start = self.camera_settings.clone();
        }
        self.camera_frame += if self.camera_lock { 1.0 } else { -1.0 };
        if self.camera_frame < 0.0 {
            self.camera_frame = 0.0;
        }
        if self.camera_frame > CAMERA_MAX_FRAME {
            self.camera_frame = CAMERA_MAX_FRAME;
        }
        if self.camera_frame == CAMERA_MAX_FRAME {
            self.camera_waiting += 1.0;
        }
        if self.camera_waiting > CAMERA_MAX_WAIT {
            self.camera_waiting = CAMERA_MAX_WAIT;
        }
        start
    }

    /// `StartCamera` (`CaptureInterface.cs:1589-1596`): the
    /// CameraSpaceEffects containment flag, sound 40 (returned), the
    /// settings park, the lock and the wait reset. `full_screen_area` is
    /// the caller's `FullScreenArea()` result.
    pub(crate) fn start_camera(
        &mut self,
        mut settings: CaptureSettings,
        full_screen_area: CaptureRect,
    ) -> &'static str {
        settings.camera_space_effects = full_screen_area.contains_rect(&settings.area);
        self.camera_settings = Some(settings);
        self.camera_lock = true;
        self.camera_waiting = 0.0;
        "Camera" // SoundEngine.PlaySound(40) — the camera shutter cue
    }

    /// `EndCamera` (`CaptureInterface.cs:1598-1601`).
    pub(crate) fn end_camera(&mut self) {
        self.camera_lock = false;
    }

    /// `FullScreenArea` (`CaptureInterface.cs:1140-1159`): the camera's
    /// scaled view rect in tiles — ceil on the min corner, floor on the
    /// max, width/height the difference (no +1). The port passes the
    /// scaled position/size in (the App's camera + screen/zoom).
    pub(crate) fn full_screen_area(
        scaled_position: (f32, f32),
        scaled_size: (f32, f32),
    ) -> CaptureRect {
        let min_x = (scaled_position.0 / 16.0).ceil() as i32;
        let min_y = (scaled_position.1 / 16.0).ceil() as i32;
        let max_x = ((scaled_position.0 + scaled_size.0) / 16.0).floor() as i32;
        let max_y = ((scaled_position.1 + scaled_size.1) / 16.0).floor() as i32;
        CaptureRect::new(min_x, min_y, max_x - min_x, max_y - min_y)
    }

    /// `DrawCameraLock`'s math (`CaptureInterface.cs:1558-1587`): the
    /// black-fade alpha (frame/5) and the progress readout's fade-in
    /// (`waiting - 60 + 5`, /5, only when > 0) + the clamped percent.
    /// Returns `(fade_alpha, Option<(percent_text, alpha)>)`.
    pub(crate) fn camera_lock_overlay(&self, progress: f32) -> (f32, Option<(String, f32)>) {
        if self.camera_frame == 0.0 {
            return (0.0, None);
        }
        let fade = self.camera_frame / CAMERA_MAX_FRAME;
        if self.camera_frame != CAMERA_MAX_FRAME {
            return (fade, None);
        }
        let mut num = self.camera_waiting - CAMERA_MAX_WAIT + CAMERA_MAX_FRAME;
        if num <= 0.0 {
            return (fade, None);
        }
        num /= CAMERA_MAX_FRAME;
        let mut percent = progress * 100.0;
        if percent > 100.0 {
            percent = 100.0;
        }
        // `num2.ToString("##")` — the integer part, no leading zero.
        let text = format!("{} ", percent as i32);
        (fade, Some((text, num)))
    }

    /// The marked-area draw rect both selection modes share
    /// (`ModeEdgeSelection.DrawMarkedArea`, `CaptureInterface.cs:141-183`;
    /// `ModeDragBounds.DrawMarkedArea`, `:579-639`): the tile span ×16,
    /// intersected with the screen, offset into screen space. `None` when
    /// an edge is unpinned or the intersection is empty. The world arm
    /// only (the map arm is the GetMapCoords gap).
    pub(crate) fn marked_area_screen_rect(&self, info: &CaptureWorldInfo) -> Option<CaptureRect> {
        if !self.edge_a_pinned || !self.edge_b_pinned {
            return None;
        }
        let num = self.edge_a.0.min(self.edge_b.0);
        let num2 = self.edge_a.1.min(self.edge_b.1);
        let num3 = (self.edge_a.0 - self.edge_b.0).abs();
        let num4 = (self.edge_a.1 - self.edge_b.1).abs();
        let value = CaptureRect::new(num * 16, num2 * 16, (num3 + 1) * 16, (num4 + 1) * 16);
        let value2 = CaptureRect::new(
            info.screen_position_x as i32,
            info.screen_position_y as i32,
            info.screen_width + 1,
            info.screen_height + 1,
        );
        let mut result = value2.intersect(&value);
        if result.width == 0 || result.height == 0 {
            return None;
        }
        result.x -= value2.x;
        result.y -= value2.y;
        Some(result)
    }
}

impl CaptureRect {
    /// `Rectangle.Contains(Rectangle)` — full containment
    /// (`StartCamera`'s CameraSpaceEffects gate, `:1591`).
    pub(crate) fn contains_rect(&self, other: &CaptureRect) -> bool {
        self.x <= other.x
            && self.y <= other.y
            && other.x + other.width <= self.x + self.width
            && other.y + other.height <= self.y + self.height
    }
}

/// The side effects [`CaptureInterface::update`] reports — the sound cues
/// (`SoundEngine.PlaySound` ids mapped to the port's cue names) and the
/// toolbar action the App runs.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct CaptureUpdateEffects {
    pub cues: Vec<&'static str>,
    pub toolbar: Option<ToolbarAction>,
}

/// `UpdateButtons`' nine pressed-button effects
/// (`CaptureInterface.cs:1075-1134`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ToolbarAction {
    /// Button 0 — `QuickScreenshot` (`:1077`).
    QuickScreenshot,
    /// Button 1 — the area capture (`:1079-1095`).
    CaptureArea,
    /// Buttons 2/3/6 — the mode pick + `ToggleCamera` (`:1096-1123`).
    SelectMode(i32),
    /// Button 4 — `ResetFocus` (`:1108-1112`).
    ResetFocus,
    /// Button 5 — the fullscreen-map toggle (`:1113-1117`).
    ToggleMapFullscreen,
    /// Button 7 — `Utils.OpenFolder(SavePath/Captures)` (`:1124-1128`).
    OpenFolder,
    /// Button 8 — close (`:1129-1134`).
    Close,
}

// --- CaptureCamera ----------------------------------------------------------

/// `CaptureCamera`'s constants (`CaptureCamera.cs:34-42`).
pub(crate) const CHUNK_SIZE: i32 = 128;
pub(crate) const FRAMEBUFFER_PIXEL_SIZE: i32 = 2048;
pub(crate) const INNER_CHUNK_SIZE: i32 = 126;
pub(crate) const MAX_IMAGE_SIZE: i32 = 4096;
pub(crate) const CAPTURE_DIRECTORY: &str = "Captures";

/// `CaptureCamera.CaptureChunk` (`CaptureCamera.cs:19-30`): the tile area
/// plus its scaled pixel rect in the output image.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct CaptureChunk {
    pub area: CaptureRect,
    pub scaled_area: CaptureRect,
}

/// The GPU seam (S1-G-13): `CaptureCamera.DrawTick`'s render half
/// (`CaptureCamera.cs:186-209`) — prepare the area, draw the chunk through
/// `Main.DrawCapture` into the 2048² framebuffer, downscale when
/// `UseScaling`, read the pixels back. The port's trait keeps the queue
/// machine pure; the wgpu implementation lands with the render-target
/// family. `render_chunk` returns the chunk's RGBA bytes (scaled size when
/// `use_scaling`, else the raw 16-px tile render cropped to the chunk).
pub(crate) trait CaptureRenderer {
    fn render_chunk(
        &mut self,
        chunk: &CaptureChunk,
        settings: &CaptureSettings,
        scaled_framebuffer_size: i32,
    ) -> Option<Vec<u8>>;
}

/// `CaptureCamera` (`CaptureCamera.cs:17-425`) — the capture entity: the
/// active settings, the chunk queue, the output buffer and the progress
/// counters. The render targets themselves are the S1-G-13 seam; the
/// queue/scaling/blit/finish machine is ported whole.
#[derive(Clone, Debug)]
pub(crate) struct CaptureCamera {
    /// `_activeSettings` (`:60`) — `Some` while capturing (`IsCapturing`).
    active_settings: Option<CaptureSettings>,
    /// `_renderQueue` (`:62`).
    render_queue: std::collections::VecDeque<CaptureChunk>,
    /// `_outputData` (`:68`) — the packed RGBA output image.
    output_data: Option<Vec<u8>>,
    /// `_outputImageSize` (`:70`).
    output_image_size: (i32, i32),
    /// `_scaledFrameData`'s width (`:153-154`: `floor(num * 2048)`).
    scaled_framebuffer_size: i32,
    /// `_tilesProcessed` (`:74`).
    tiles_processed: f32,
    /// `_totalTiles` (`:76`).
    total_tiles: f32,
    /// `_isDisposed` (`:58`).
    is_disposed: bool,
}

impl Default for CaptureCamera {
    fn default() -> Self {
        CaptureCamera {
            active_settings: None,
            render_queue: std::collections::VecDeque::new(),
            output_data: None,
            output_image_size: (0, 0),
            scaled_framebuffer_size: 0,
            tiles_processed: 0.0,
            total_tiles: 0.0,
            is_disposed: false,
        }
    }
}

/// `Capture`'s error arms (`CaptureCamera.cs:110-134`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CaptureError {
    /// `Invariant.Assert(!_isDisposed)` (`:113`).
    Disposed,
    /// `InvalidOperationException` (`:114-117`).
    AlreadyCapturing,
}

impl CaptureCamera {
    /// `IsCapturing` (`CaptureCamera.cs:78-87`): `_activeSettings != null`.
    pub(crate) fn is_capturing(&self) -> bool {
        self.active_settings.is_some()
    }

    /// `Capture` + `_Capture` (`CaptureCamera.cs:110-176`): the disposed /
    /// already-active guards, the scaling math (`:139-155`), the output
    /// buffer allocation and the 126-stride chunk queue (`:161-175`).
    /// `Main.GlobalTimerPaused = true` (`:112`) is the App's side effect —
    /// reported through the return (the caller pauses the clock).
    pub(crate) fn capture(&mut self, settings: CaptureSettings) -> Result<(), CaptureError> {
        if self.is_disposed {
            return Err(CaptureError::Disposed);
        }
        if self.active_settings.is_some() {
            return Err(CaptureError::AlreadyCapturing);
        }
        self.active_settings = Some(settings.clone());
        let area = settings.area;
        let mut num = 1.0f32;
        if settings.use_scaling {
            if area.width * 16 > MAX_IMAGE_SIZE {
                num = MAX_IMAGE_SIZE as f32 / (area.width * 16) as f32;
            }
            if area.height * 16 > MAX_IMAGE_SIZE {
                num = num.min(MAX_IMAGE_SIZE as f32 / (area.height * 16) as f32);
            }
            num = num.min(1.0);
            // `MathHelper.Clamp((int)(num * (area.Width * 16)), 1, 4096)`
            // (`:151`) — the (int) truncation first, then the clamp.
            let w = ((num * (area.width * 16) as f32) as i32).clamp(1, MAX_IMAGE_SIZE);
            let h = ((num * (area.height * 16) as f32) as i32).clamp(1, MAX_IMAGE_SIZE);
            self.output_image_size = (w, h);
            self.output_data = Some(vec![0u8; 4 * w as usize * h as usize]);
            let num2 = (num * FRAMEBUFFER_PIXEL_SIZE as f32).floor() as i32;
            self.scaled_framebuffer_size = num2;
        } else {
            // `new byte[16777216]` (`:159`) — the 2048² RGBA scratch.
            self.output_data = Some(vec![0u8; 16_777_216]);
        }
        self.tiles_processed = 0.0;
        self.total_tiles = (area.width * area.height) as f32;
        // The chunk grid (`:163-175`): 126-stride, 128-cap chunks.
        let mut i = area.x;
        while i < area.x + area.width {
            let mut j = area.y;
            while j < area.y + area.height {
                let num3 = CHUNK_SIZE.min(area.x + area.width - i);
                let num4 = CHUNK_SIZE.min(area.y + area.height - j);
                let width = (num * (num3 * 16) as f32).floor() as i32;
                let height = (num * (num4 * 16) as f32).floor() as i32;
                let x = (num * ((i - area.x) * 16) as f32).floor() as i32;
                let y = (num * ((j - area.y) * 16) as f32).floor() as i32;
                self.render_queue.push_back(CaptureChunk {
                    area: CaptureRect::new(i, j, num3, num4),
                    scaled_area: CaptureRect::new(x, y, width, height),
                });
                j += INNER_CHUNK_SIZE;
            }
            i += INNER_CHUNK_SIZE;
        }
        Ok(())
    }

    /// `DrawTick` (`CaptureCamera.cs:178-217`): dequeue one chunk, render
    /// it through the [`CaptureRenderer`] seam, blit the scaled bytes into
    /// the output (or hand the raw chunk to the renderer's save path),
    /// advance the tile counter; when the queue drains, `FinishCapture`.
    /// Returns the finish flag (the caller unpauses the clock + runs
    /// `CaptureInterface.EndCamera`).
    pub(crate) fn draw_tick(&mut self, renderer: &mut dyn CaptureRenderer) -> bool {
        if self.active_settings.is_none() {
            return false;
        }
        if let Some(chunk) = self.render_queue.pop_front() {
            let settings = self.active_settings.clone().expect("active above");
            if let Some(bytes) =
                renderer.render_chunk(&chunk, &settings, self.scaled_framebuffer_size)
            {
                if settings.use_scaling {
                    // `DrawBytesToBuffer` (`:203`, `:238-272`).
                    if let Some(out) = self.output_data.as_mut() {
                        draw_bytes_to_buffer(
                            &bytes,
                            out,
                            self.scaled_framebuffer_size,
                            self.output_image_size.0,
                            &chunk.scaled_area,
                        );
                    }
                }
                // The no-scaling arm saves the chunk PNG directly
                // (`:205-209`) — the renderer's save path owns it (the
                // bytes are the cropped chunk; the seam's GPU impl writes
                // `Captures/<name>/<x>-<y>.png`).
            }
            self.tiles_processed += (chunk.area.width * chunk.area.height) as f32;
        }
        if self.render_queue.is_empty() {
            self.finish_capture();
            return true;
        }
        false
    }

    /// `GetProgress` (`CaptureCamera.cs:274-277`): processed / total tiles.
    pub(crate) fn get_progress(&self) -> f32 {
        self.tiles_processed / self.total_tiles
    }

    /// `FinishCapture` (`CaptureCamera.cs:370-398`) minus the PNG write
    /// retry loop (the seam's save path owns the file IO): the output
    /// buffer drops, the settings clear. The caller unpauses
    /// `Main.GlobalTimerPaused` and runs `CaptureInterface.EndCamera`
    /// (`:390-391`).
    pub(crate) fn finish_capture(&mut self) {
        self.output_data = None;
        self.active_settings = None;
    }

    /// The finished output image (the seam's save path writes it as
    /// `Captures/<OutputName>.png`, `:375`).
    pub(crate) fn output_image(&self) -> Option<(&[u8], i32, i32)> {
        self.output_data
            .as_deref()
            .map(|d| (d, self.output_image_size.0, self.output_image_size.1))
    }

    /// `Dispose` (`CaptureCamera.cs:400-424`): the one-shot disposed latch.
    pub(crate) fn dispose(&mut self) {
        if self.is_disposed {
            return;
        }
        self.is_disposed = true;
    }
}

/// `DrawBytesToBuffer` (`CaptureCamera.cs:238-272`), the FNA arm
/// (`Program.IsXna` false — the port keeps RGBA order, no BGRA swap):
/// copy the `area` sub-rect of the source framebuffer into the output
/// image at the same rect, row-stride aware.
pub(crate) fn draw_bytes_to_buffer(
    source: &[u8],
    destination: &mut [u8],
    source_width: i32,
    destination_width: i32,
    area: &CaptureRect,
) {
    let sw = source_width as usize;
    let dw = destination_width as usize;
    for i in 0..area.height as usize {
        for j in 0..area.width as usize {
            let src = ((area.y as usize + i) * sw + (area.x as usize + j)) * 4;
            let dst = ((area.y as usize + i) * dw + (area.x as usize + j)) * 4;
            if src + 4 <= source.len() && dst + 4 <= destination.len() {
                destination[dst..dst + 4].copy_from_slice(&source[src..src + 4]);
            }
        }
    }
}

// --- CaptureManager ---------------------------------------------------------

/// `CaptureManager` (`CaptureManager.cs:8-114`) — the facade over the
/// interface + camera. The C# `Instance` static is the App's one owned
/// value; `Main.dedServ` is always false in the port (no dedicated server
/// render path), so the camera always exists.
#[derive(Clone, Debug, Default)]
pub(crate) struct CaptureManager {
    pub interface: CaptureInterface,
    pub camera: CaptureCamera,
    /// `Main.CaptureModeDisabled` (`Main.cs:293`) — the ctor's
    /// render-target allocation failure sets it (`CaptureCamera.cs:104`);
    /// the port starts false (the targets are the S1-G-13 seam).
    pub capture_mode_disabled: bool,
}

impl CaptureManager {
    /// `IsCapturing` (`CaptureManager.cs:16-27`): the camera's active flag
    /// (dedServ false in the port).
    pub(crate) fn is_capturing(&self) -> bool {
        self.camera.is_capturing()
    }

    /// `Active`'s setter (`CaptureManager.cs:29-42`): the
    /// CaptureModeDisabled gate + the change edge → `ToggleCamera`.
    /// Returns the sound cue the toggle fired.
    pub(crate) fn set_active(&mut self, value: bool) -> Option<&'static str> {
        if !self.capture_mode_disabled && self.interface.active != value {
            self.interface.toggle_camera(value)
        } else {
            None
        }
    }

    /// `UsingMap` (`CaptureManager.cs:44-54`): false while inactive, else
    /// the interface's answer.
    pub(crate) fn using_map(&self) -> bool {
        if !self.interface.active {
            return false;
        }
        self.interface.using_map()
    }

    /// `Scrolling` (`CaptureManager.cs:65-68`).
    pub(crate) fn scrolling(&mut self, scroll_wheel_delta: i32) -> bool {
        self.interface.scrolling(scroll_wheel_delta)
    }

    /// `Update` (`CaptureManager.cs:70-73`).
    pub(crate) fn update(
        &mut self,
        context: SelectionContext,
        input: &CaptureInput,
        info: &CaptureWorldInfo,
    ) -> CaptureUpdateEffects {
        self.interface.update(context, input, info)
    }

    /// `GetProgress` (`CaptureManager.cs:80-83`).
    pub(crate) fn get_progress(&self) -> f32 {
        self.camera.get_progress()
    }

    /// `Capture()` (`CaptureManager.cs:85-92`): the debug default —
    /// area (2660, 100, 1000, 1000), UseScaling false.
    pub(crate) fn capture_default(&mut self) -> Result<(), CaptureError> {
        self.capture(CaptureSettings {
            area: CaptureRect::new(2660, 100, 1000, 1000),
            use_scaling: false,
            ..CaptureSettings::default()
        })
    }

    /// `Capture(settings)` (`CaptureManager.cs:94-97`).
    pub(crate) fn capture(&mut self, settings: CaptureSettings) -> Result<(), CaptureError> {
        self.camera.capture(settings)
    }

    /// `DrawTick` (`CaptureManager.cs:99-103`): the countdown first (its
    /// frame-4 arm starts the capture through `Capture`), then the
    /// camera's queue drain. Returns `(start_settings, finished)` — the
    /// App runs the start (the `GlobalTimerPaused` pause) and the finish
    /// (the unpause + `EndCamera`).
    pub(crate) fn draw_tick(
        &mut self,
        renderer: &mut dyn CaptureRenderer,
    ) -> (Option<CaptureSettings>, bool) {
        let start = self.interface.update_camera_countdown();
        if let Some(settings) = start.clone() {
            // `CaptureManager.Instance.Capture(CameraSettings)` (`:1537`).
            let _ = self.camera.capture(settings);
        }
        let finished = self.camera.draw_tick(renderer);
        if finished {
            // `FinishCapture`'s `CaptureInterface.EndCamera()` (`:391`).
            self.interface.end_camera();
        }
        (start, finished)
    }

    /// `Dispose` (`CaptureManager.cs:105-108`).
    pub(crate) fn dispose(&mut self) {
        self.camera.dispose();
    }
}

/// The marked-area colour as a renderer tint — the packed ABGR unpack.
pub(crate) fn packed_to_tint(packed: u32) -> [f32; 4] {
    color::tint([
        (packed & 0xFF) as u8,
        ((packed >> 8) & 0xFF) as u8,
        ((packed >> 16) & 0xFF) as u8,
        ((packed >> 24) & 0xFF) as u8,
    ])
}

#[cfg(test)]
mod tests {
    use super::*;

    fn info() -> CaptureWorldInfo {
        CaptureWorldInfo {
            max_tiles_x: 4200,
            max_tiles_y: 1200,
            screen_width: 1600,
            screen_height: 900,
            screen_position_x: 16000.0,
            screen_position_y: 4000.0,
        }
    }

    // --- CaptureBiome -------------------------------------------------------

    /// `GetCaptureBiome`'s explicit arms (`CaptureBiome.cs:113-138`).
    #[test]
    fn s1_e_17_biome_choice_arms_match_the_styles_table() {
        let pick = |choice| CaptureBiome::get_capture_biome(choice, 0, 0, 0.0, [0; 3]);
        assert_eq!(pick(2), CaptureBiome::CORRUPTION);
        assert_eq!(pick(3), CaptureBiome::JUNGLE);
        assert_eq!(pick(4), CaptureBiome::HALLOW);
        assert_eq!(pick(5), CaptureBiome::SNOW);
        assert_eq!(pick(6), CaptureBiome::DESERT);
        assert_eq!(pick(7), CaptureBiome::DIRT_LAYER);
        assert_eq!(pick(8), CaptureBiome::ROCK_LAYER);
        assert_eq!(pick(9), CaptureBiome::CRIMSON);
        assert_eq!(pick(10), CaptureBiome::UNDERGROUND_DESERT);
        assert_eq!(pick(11), CaptureBiome::OCEAN);
        assert_eq!(pick(12), CaptureBiome::MUSHROOM);
        // The Styles constants themselves (`:58-73`).
        assert_eq!(CaptureBiome::CORRUPTION.background_index, 1);
        assert_eq!(CaptureBiome::CORRUPTION.water_style, 2);
        assert_eq!(CaptureBiome::CORRUPTION.tile_color, TileColorStyle::Corrupt);
        assert_eq!(CaptureBiome::MUSHROOM.background_index, 9);
        assert_eq!(CaptureBiome::MUSHROOM.water_style, 7);
        assert_eq!(CaptureBiome::MUSHROOM.tile_color, TileColorStyle::Mushroom);
    }

    /// The automated fall-through (`CaptureBiome.cs:139-152`): location
    /// first, then water, then the treeX purity split.
    #[test]
    fn s1_e_17_automated_biome_falls_through_location_water_purity() {
        // Location wins: bg 3 -> Jungle regardless of water.
        assert_eq!(
            CaptureBiome::get_capture_biome(-1, 3, 5, 0.0, [0; 3]),
            CaptureBiome::JUNGLE
        );
        // The desert-variant group 2/5/13/14 (`:184-188`).
        for bg in [2, 5, 13, 14] {
            assert_eq!(
                CaptureBiome::get_capture_biome(-1, bg, 0, 0.0, [0; 3]),
                CaptureBiome::DESERT
            );
        }
        // No location match (bg 99): water 5 -> Snow.
        assert_eq!(
            CaptureBiome::get_capture_biome(-1, 99, 5, 0.0, [0; 3]),
            CaptureBiome::SNOW
        );
        // Water 0/1/11 have no table slot -> purity; treeX splits it.
        let tree_x = [100, 200, 300];
        assert_eq!(
            CaptureBiome::get_capture_biome(-1, 99, 0, 50.0 * 16.0, tree_x),
            CaptureBiome::PURITY
        );
        assert_eq!(
            CaptureBiome::get_capture_biome(-1, 99, 0, 150.0 * 16.0, tree_x),
            CaptureBiome::PURITY2
        );
        assert_eq!(
            CaptureBiome::get_capture_biome(-1, 99, 0, 250.0 * 16.0, tree_x),
            CaptureBiome::PURITY3
        );
        assert_eq!(
            CaptureBiome::get_capture_biome(-1, 99, 0, 350.0 * 16.0, tree_x),
            CaptureBiome::PURITY4
        );
        // Choice 1 (Purity) runs the same treeX split (`:115-116`).
        assert_eq!(
            CaptureBiome::get_capture_biome(1, 3, 5, 150.0 * 16.0, tree_x),
            CaptureBiome::PURITY2
        );
    }

    // --- the interface state machine ----------------------------------------

    /// `ToggleCamera` (`CaptureInterface.cs:1039-1059`): the sound edge,
    /// the JustActivated latch, the CameraLock gate.
    #[test]
    fn s1_e_17_toggle_camera_edges_and_latches() {
        let mut i = CaptureInterface::default();
        assert_eq!(i.toggle_camera(true), Some("Menu_Open")); // sound 10
        assert!(i.active);
        assert!(i.just_activated);
        // No edge on a repeat.
        assert_eq!(i.toggle_camera(true), None);
        assert_eq!(i.toggle_camera(false), Some("Menu_Close")); // sound 11
        assert!(!i.active);
        // CameraLock blocks the toggle entirely.
        i.camera_lock = true;
        assert_eq!(i.toggle_camera(true), None);
        assert!(!i.active);
    }

    /// `Update`'s F4 toggle edge (`:983-988`): the held-key gate, the
    /// mouse-item gate, the dead/ghost gates.
    #[test]
    fn s1_e_17_f4_toggle_respects_the_gates() {
        let mut i = CaptureInterface::default();
        let mut input = CaptureInput {
            toggle_camera_mode: true,
            mouse_item_empty: true,
            ..Default::default()
        };
        let effects = i.update(SelectionContext::World, &input, &info());
        assert!(i.active);
        assert_eq!(effects.cues, vec!["Menu_Open"]);
        // Held: no re-toggle.
        let effects = i.update(SelectionContext::World, &input, &info());
        assert!(i.active);
        assert!(effects.cues.is_empty());
        // Release + re-press toggles off.
        input.toggle_camera_mode = false;
        i.update(SelectionContext::World, &input, &info());
        input.toggle_camera_mode = true;
        i.update(SelectionContext::World, &input, &info());
        assert!(!i.active);
        // A held item blocks activation (but not deactivation).
        input.mouse_item_empty = false;
        input.toggle_camera_mode = false;
        i.update(SelectionContext::World, &input, &info());
        input.toggle_camera_mode = true;
        i.update(SelectionContext::World, &input, &info());
        assert!(!i.active, "the held item blocked the toggle");
        // Dead player blocks too.
        input.mouse_item_empty = true;
        input.player_dead = true;
        input.toggle_camera_mode = false;
        i.update(SelectionContext::World, &input, &info());
        input.toggle_camera_mode = true;
        i.update(SelectionContext::World, &input, &info());
        assert!(!i.active);
    }

    /// `EdgePlacement` (`:105-139`): left pins A, right pins B at the
    /// MouseWorld tile; the fluff-40 clamp rides after.
    #[test]
    fn s1_e_17_edge_selection_pins_and_clamps() {
        let mut i = CaptureInterface::default();
        i.active = true;
        i.selected_mode = 0;
        let mut input = CaptureInput {
            mouse_left: true,
            mouse_world_x: 100.0 * 16.0,
            mouse_world_y: 200.0 * 16.0,
            ..Default::default()
        };
        i.update(SelectionContext::World, &input, &info());
        assert!(i.edge_a_pinned);
        assert_eq!(i.edge_a, (100, 200));
        assert!(!i.edge_b_pinned);
        input.mouse_left = false;
        input.mouse_right = true;
        input.mouse_world_x = 300.0 * 16.0;
        input.mouse_world_y = 400.0 * 16.0;
        i.update(SelectionContext::World, &input, &info());
        assert!(i.edge_b_pinned);
        assert_eq!(i.edge_b, (300, 400));
        // The clamp: beyond the world edge snaps to maxTiles-1-40.
        input.mouse_world_x = 5000.0 * 16.0;
        input.mouse_world_y = 5000.0 * 16.0;
        i.update(SelectionContext::World, &input, &info());
        assert_eq!(i.edge_b, (4200 - 1 - 40, 1200 - 1 - 40));
        // Below the fluff snaps to 40.
        input.mouse_world_x = 0.0;
        input.mouse_world_y = 0.0;
        i.update(SelectionContext::World, &input, &info());
        assert_eq!(i.edge_b, (40, 40));
    }

    /// `GetArea` (`:970-973`) and `ResetFocus` (`:1502-1508`).
    #[test]
    fn s1_e_17_get_area_and_reset_focus() {
        let mut i = CaptureInterface::default();
        i.edge_a = (300, 400);
        i.edge_b = (100, 200);
        assert_eq!(i.get_area(), CaptureRect::new(100, 200, 201, 201));
        i.reset_focus();
        assert!(!i.edge_a_pinned);
        assert!(!i.edge_b_pinned);
        assert_eq!(i.edge_a, (-1, -1));
        assert_eq!(i.edge_b, (-1, -1));
    }

    /// `Scrolling` (`:1510-1531`): the /120 %30 wrap, the ±2 fold, the
    /// change edge.
    #[test]
    fn s1_e_17_scrolling_cycles_the_three_modes() {
        let mut i = CaptureInterface::default();
        i.selected_mode = 0;
        // One notch down: 0 - 1 = -1 -> +2 = 1.
        assert!(i.scrolling(120));
        assert_eq!(i.selected_mode, 1);
        // One notch up: num = -1 % 30 = -1 -> +30 = 29; 1 - 29 = -28 ->
        // the +2 fold walks it to 0 (the C#'s while loop, `:1519-1522`).
        assert!(i.scrolling(-120));
        assert_eq!(i.selected_mode, 0);
        // Another up: 0 - 29 = -29 -> +2 fold -> 1.
        assert!(i.scrolling(-120));
        assert_eq!(i.selected_mode, 1);
        // A zero delta is no change.
        assert!(!i.scrolling(0));
        assert_eq!(i.selected_mode, 1);
    }

    /// `ModeChangeSettings` (`:820-847`, `:708-737`): the panel hover, the
    /// button rows, the toggles and the reset row.
    #[test]
    fn s1_e_17_settings_panel_buttons_toggle_and_reset() {
        let mut i = CaptureInterface::default();
        i.active = true;
        i.selected_mode = 2;
        let rect = i.settings_panel_rect();
        assert_eq!(rect, CaptureRect::new(115, 80, 224, 170));
        // Row 0 (PackImage) at (rect.X+20, rect.Y+20), 16 px tall.
        let mut input = CaptureInput {
            mouse_x: (rect.x + 30) as f32,
            mouse_y: (rect.y + 25) as f32,
            mouse_left: true,
            mouse_left_release: true,
            ..Default::default()
        };
        let effects = i.update(SelectionContext::World, &input, &info());
        assert_eq!(effects.cues, vec!["Menu_Tick"]); // sound 12
        assert!(!i.settings.pack_image);
        assert!(i.change_settings.in_ui);
        // Row 6 (reset) at y + 20 + 6*20.
        input.mouse_y = (rect.y + 20 + 120 + 5) as f32;
        i.update(SelectionContext::World, &input, &info());
        assert!(i.settings.pack_image);
        assert!(i.settings.include_entities);
        assert!(!i.settings.transparent_background);
        assert_eq!(i.settings.biome_choice_index, -1);
        // Rows 3/4/5 are no-ops (`:701-704`).
        input.mouse_y = (rect.y + 20 + 3 * 20 + 5) as f32;
        let effects = i.update(SelectionContext::World, &input, &info());
        assert!(effects.cues.is_empty());
    }

    /// `UpdateButtons` (`:1061-1138`): the toolbar geometry + the pressed
    /// actions; the mode-select rows re-toggle the camera.
    #[test]
    fn s1_e_17_toolbar_buttons_route_their_actions() {
        let mut i = CaptureInterface::default();
        i.active = true;
        // Button 1 needs both edges pinned.
        let mut input = CaptureInput {
            mouse_x: 24.0 + 46.0 + 10.0,
            mouse_y: 30.0,
            mouse_left: true,
            mouse_left_release: true,
            ..Default::default()
        };
        let effects = i.update(SelectionContext::World, &input, &info());
        assert_eq!(effects.toolbar, None, "unpinned edges: no capture");
        i.edge_a_pinned = true;
        i.edge_b_pinned = true;
        let effects = i.update(SelectionContext::World, &input, &info());
        assert_eq!(effects.toolbar, Some(ToolbarAction::CaptureArea));
        // Button 8 closes.
        input.mouse_x = 24.0 + 46.0 * 8.0 + 10.0;
        let effects = i.update(SelectionContext::World, &input, &info());
        assert_eq!(effects.toolbar, Some(ToolbarAction::Close));
        // Button 7 is fullscreen-gated.
        input.mouse_x = 24.0 + 46.0 * 7.0 + 10.0;
        input.is_fullscreen = true;
        let effects = i.update(SelectionContext::World, &input, &info());
        assert_eq!(effects.toolbar, None);
        input.is_fullscreen = false;
        let effects = i.update(SelectionContext::World, &input, &info());
        assert_eq!(effects.toolbar, Some(ToolbarAction::OpenFolder));
        // Outside the toolbar: no hover.
        input.mouse_x = 1000.0;
        let effects = i.update(SelectionContext::World, &input, &info());
        assert_eq!(effects.toolbar, None);
        assert_eq!(i.hovered_mode, -1);
    }

    /// `UsingMap` (`:1493-1500`) per mode + the CameraLock override.
    #[test]
    fn s1_e_17_using_map_follows_the_mode() {
        let mut i = CaptureInterface::default();
        i.selected_mode = 0;
        assert!(i.using_map());
        i.selected_mode = 1;
        assert!(!i.using_map());
        i.drag_bounds.caught_edge = 0;
        assert!(i.using_map());
        i.selected_mode = 2;
        i.drag_bounds.caught_edge = -1;
        assert!(!i.using_map());
        i.change_settings.in_ui = true;
        assert!(i.using_map());
        i.camera_lock = true;
        i.change_settings.in_ui = false;
        assert!(i.using_map(), "CameraLock forces true");
    }

    /// `ModeDragBounds` (`:384-565`): the initial pin, the edge hover
    /// catch and the drag move.
    #[test]
    fn s1_e_17_drag_bounds_pins_catches_and_drags() {
        let mut i = CaptureInterface::default();
        i.active = true;
        i.selected_mode = 1;
        let inf = info();
        // First press pins both edges at the screen+mouse tile.
        let mut input = CaptureInput {
            mouse_x: 100.0,
            mouse_y: 100.0,
            mouse_left: true,
            // The drag machine reads MouseWorld for the move; the pin
            // itself uses screenPosition + mouse (`:399`).
            mouse_world_x: inf.screen_position_x + 100.0,
            mouse_world_y: inf.screen_position_y + 100.0,
            ..Default::default()
        };
        i.update(SelectionContext::World, &input, &inf);
        assert!(i.edge_a_pinned && i.edge_b_pinned);
        let tile = (
            ((inf.screen_position_x + 100.0) / 16.0) as i32,
            ((inf.screen_position_y + 100.0) / 16.0) as i32,
        );
        assert_eq!(i.edge_a, tile);
        assert_eq!(i.edge_b, tile);
        assert_eq!(i.drag_bounds.current_aim, 3);
        assert_eq!(i.drag_bounds.caught_edge, 1);
        // Spread the edges so the marked area has a real on-screen rect
        // (the screen covers tiles x 1000..1100, y 250..306).
        i.edge_a = (1010, 260);
        i.edge_b = (1090, 300);
        // Release: the aim clears, the hover catches an edge. The top
        // edge strip sits at screen y = 260*16 - 4000 - 2 = 158.
        input.mouse_left = false;
        input.mouse_x = (1050 * 16 - inf.screen_position_x as i32) as f32;
        input.mouse_y = (260 * 16 - inf.screen_position_y as i32 - 2) as f32;
        i.update(SelectionContext::World, &input, &inf);
        assert_eq!(i.drag_bounds.current_aim, 0, "the top edge caught");
        // A.Y < B.Y -> caughtEdge = num6 = 0 (normal gravity): edge A moves.
        assert_eq!(i.drag_bounds.caught_edge, 0);
        // Press + drag: the caught edge follows the MouseWorld tile Y.
        input.mouse_left = true;
        input.mouse_world_x = 1050.0 * 16.0;
        input.mouse_world_y = 280.0 * 16.0;
        i.update(SelectionContext::World, &input, &inf);
        assert!(i.drag_bounds.dragging);
        assert_eq!(i.edge_a.1, 280);
        assert_eq!(i.edge_b.1, 300, "the other edge stayed");
    }

    // --- the camera countdown + capture lifecycle ----------------------------

    /// `UpdateCameraCountdown` (`:1533-1556`): the frame-4 capture start,
    /// the 0..5 clamp, the wait counter at 5.
    #[test]
    fn s1_e_17_camera_countdown_starts_at_frame_four() {
        let mut i = CaptureInterface::default();
        let settings = CaptureSettings::default();
        i.start_camera(settings.clone(), CaptureRect::new(0, 0, 100, 100));
        assert!(i.camera_lock);
        assert_eq!(i.camera_waiting, 0.0);
        // `StartCamera` stamped CameraSpaceEffects (`:1591`) — the parked
        // settings carry it.
        let parked = i.camera_settings.clone().unwrap();
        // Frames 0->1->2->3->4: the start fires exactly at the 4->5 tick.
        for frame in 1..=4 {
            assert_eq!(i.update_camera_countdown(), None);
            assert_eq!(i.camera_frame, frame as f32);
        }
        assert_eq!(i.update_camera_countdown(), Some(parked));
        assert_eq!(i.camera_frame, 5.0);
        // The wait counter runs at frame 5, clamped at 60.
        for _ in 0..70 {
            i.update_camera_countdown();
        }
        assert_eq!(i.camera_waiting, 60.0);
        // Unlock: the frame counts back down to 0.
        i.end_camera();
        for _ in 0..10 {
            i.update_camera_countdown();
        }
        assert_eq!(i.camera_frame, 0.0);
    }

    /// `StartCamera`'s CameraSpaceEffects containment (`:1591`) and
    /// `FullScreenArea`'s ceil/floor tile rect (`:1140-1159`).
    #[test]
    fn s1_e_17_start_camera_sets_the_space_effects_flag() {
        let mut i = CaptureInterface::default();
        let full = CaptureInterface::full_screen_area((16000.0, 4000.0), (1600.0, 900.0));
        // ceil(1000)=1000, ceil(250)=250, floor(1100)=1100, floor(306.25)=306.
        assert_eq!(full, CaptureRect::new(1000, 250, 100, 56));
        let inside = CaptureSettings { area: CaptureRect::new(1010, 260, 10, 10), ..Default::default() };
        i.start_camera(inside, full);
        assert!(i.camera_settings.as_ref().unwrap().camera_space_effects);
        i.end_camera();
        let outside = CaptureSettings { area: CaptureRect::new(0, 0, 10, 10), ..Default::default() };
        i.start_camera(outside, full);
        assert!(!i.camera_settings.as_ref().unwrap().camera_space_effects);
    }

    /// `DrawCameraLock`'s overlay math (`:1558-1587`).
    #[test]
    fn s1_e_17_camera_lock_overlay_fades_and_reports() {
        let mut i = CaptureInterface::default();
        assert_eq!(i.camera_lock_overlay(0.5), (0.0, None));
        i.camera_frame = 2.0;
        assert_eq!(i.camera_lock_overlay(0.5), (0.4, None));
        i.camera_frame = 5.0;
        // waiting 0: num = -55 -> no readout yet.
        assert_eq!(i.camera_lock_overlay(0.5), (1.0, None));
        i.camera_waiting = 58.0;
        // num = 58 - 60 + 5 = 3 -> alpha 0.6, percent 50.
        let (fade, readout) = i.camera_lock_overlay(0.5);
        assert_eq!(fade, 1.0);
        assert_eq!(readout, Some(("50 ".to_string(), 0.6)));
        // The percent clamps at 100.
        let (_, readout) = i.camera_lock_overlay(1.5);
        assert_eq!(readout.unwrap().0, "100 ");
    }

    /// A stub renderer: every chunk returns its scaled-area-sized RGBA
    /// block filled with a marker byte.
    struct StubRenderer;
    impl CaptureRenderer for StubRenderer {
        fn render_chunk(
            &mut self,
            chunk: &CaptureChunk,
            _settings: &CaptureSettings,
            scaled_framebuffer_size: i32,
        ) -> Option<Vec<u8>> {
            let n = (scaled_framebuffer_size * scaled_framebuffer_size * 4) as usize;
            let mut bytes = vec![0u8; n];
            // Fill the chunk's scaled sub-rect with 0xAB.
            let w = scaled_framebuffer_size as usize;
            for y in 0..chunk.scaled_area.height as usize {
                for x in 0..chunk.scaled_area.width as usize {
                    let off =
                        ((chunk.scaled_area.y as usize + y) * w + chunk.scaled_area.x as usize + x)
                            * 4;
                    bytes[off..off + 4].fill(0xAB);
                }
            }
            Some(bytes)
        }
    }

    /// `Capture`/`_Capture`'s chunk grid (`CaptureCamera.cs:136-176`):
    /// the 126-stride / 128-cap tiling and the scaling math.
    #[test]
    fn s1_e_17_capture_queues_the_chunk_grid() {
        let mut cam = CaptureCamera::default();
        // 300x300 tiles at scale 1: 16*300 = 4800 > 4096 -> num = 4096/4800.
        let settings = CaptureSettings {
            area: CaptureRect::new(100, 200, 300, 300),
            ..CaptureSettings::default()
        };
        cam.capture(settings).unwrap();
        assert!(cam.is_capturing());
        // 300 = 126 + 126 + 48 -> 3x3 = 9 chunks.
        assert_eq!(cam.render_queue.len(), 9);
        let first = cam.render_queue[0];
        assert_eq!(first.area, CaptureRect::new(100, 200, 128, 128));
        assert_eq!(first.scaled_area.x, 0);
        assert_eq!(first.scaled_area.y, 0);
        let last = cam.render_queue[8];
        assert_eq!(last.area, CaptureRect::new(100 + 252, 200 + 252, 48, 48));
        // The output size: (int)(4096/4800 * 4800) = 4096.
        assert_eq!(cam.output_image_size, (4096, 4096));
        // The double-capture guard (`:114-117`).
        assert_eq!(
            cam.capture(CaptureSettings::default()),
            Err(CaptureError::AlreadyCapturing)
        );
        // The progress starts at 0.
        assert_eq!(cam.get_progress(), 0.0);
    }

    /// The no-scaling arm (`:157-160`): the 16 MiB scratch, the chunk
    /// rects at scale 1.
    #[test]
    fn s1_e_17_no_scaling_keeps_full_size_chunks() {
        let mut cam = CaptureCamera::default();
        let settings = CaptureSettings {
            area: CaptureRect::new(0, 0, 10, 10),
            use_scaling: false,
            ..CaptureSettings::default()
        };
        cam.capture(settings).unwrap();
        assert_eq!(cam.output_data.as_ref().unwrap().len(), 16_777_216);
        assert_eq!(cam.render_queue.len(), 1);
        let chunk = cam.render_queue[0];
        assert_eq!(chunk.area, CaptureRect::new(0, 0, 10, 10));
        assert_eq!(chunk.scaled_area, CaptureRect::new(0, 0, 160, 160));
    }

    /// The full lifecycle: capture -> draw_tick drains the queue ->
    /// finish clears the active settings; the progress climbs to 1.
    #[test]
    fn s1_e_17_capture_entity_lifecycle() {
        let mut cam = CaptureCamera::default();
        let settings = CaptureSettings {
            area: CaptureRect::new(0, 0, 126, 126),
            ..CaptureSettings::default()
        };
        cam.capture(settings).unwrap();
        assert!(cam.is_capturing());
        let mut renderer = StubRenderer;
        // One chunk: the first draw_tick drains AND finishes.
        let finished = cam.draw_tick(&mut renderer);
        assert!(finished);
        assert!(!cam.is_capturing(), "FinishCapture cleared the settings");
        assert_eq!(cam.get_progress(), 1.0);
        assert!(cam.output_data.is_none(), "the output buffer dropped");
        // A disposed camera refuses new captures (`:113`).
        cam.dispose();
        assert_eq!(
            cam.capture(CaptureSettings::default()),
            Err(CaptureError::Disposed)
        );
        // Dispose is one-shot.
        cam.dispose();
    }

    /// The multi-chunk drain: the progress climbs chunk by chunk and the
    /// output image carries every chunk's blit.
    #[test]
    fn s1_e_17_multi_chunk_capture_blits_the_output() {
        let mut cam = CaptureCamera::default();
        // 126x126 tiles = 2016 px < 4096 -> scale 1, one chunk... use
        // 200x126: 3200 px wide, two columns (126 + 74).
        let settings = CaptureSettings {
            area: CaptureRect::new(0, 0, 200, 126),
            ..CaptureSettings::default()
        };
        cam.capture(settings).unwrap();
        assert_eq!(cam.render_queue.len(), 2);
        let mut renderer = StubRenderer;
        assert!(!cam.draw_tick(&mut renderer));
        assert!(cam.is_capturing());
        // 128*126 of 200*126 tiles processed.
        let expected = (128 * 126) as f32 / (200 * 126) as f32;
        assert_eq!(cam.get_progress(), expected);
        assert!(cam.draw_tick(&mut renderer));
        // The C# counter overshoots: the 128-cap chunks sum 128+72 = 200
        // columns... the second chunk is 72 wide (200-126=74? no: the
        // stride is 126, so chunk 2 starts at 126 with width min(128,
        // 200-126)=74) — 128+74 = 202 > 200, so `_tilesProcessed /
        // _totalTiles` = 202/200 = 1.01 (`CaptureCamera.cs:210`, no clamp;
        // the UI clamps at 100, `CaptureInterface.cs:1574-1577`).
        assert_eq!(cam.get_progress(), 202.0 / 200.0);
    }

    /// `DrawBytesToBuffer` (`:238-272`): the row-stride-aware sub-rect
    /// copy, RGBA order (the FNA arm).
    #[test]
    fn s1_e_17_draw_bytes_to_buffer_copies_the_subrect() {
        // A 4x4 source, copy the 2x2 rect at (1,1) into a 3x3 dest.
        let mut source = vec![0u8; 4 * 4 * 4];
        for y in 0..4 {
            for x in 0..4 {
                let off = (y * 4 + x) * 4;
                source[off..off + 4].fill((y * 4 + x) as u8);
            }
        }
        let mut dest = vec![0u8; 3 * 3 * 4];
        draw_bytes_to_buffer(&source, &mut dest, 4, 3, &CaptureRect::new(1, 1, 2, 2));
        for y in 1..3 {
            for x in 1..3 {
                let off = (y * 3 + x) * 4;
                assert_eq!(dest[off], (y * 4 + x) as u8);
            }
        }
        // Outside the rect stays zero.
        assert_eq!(dest[0], 0);
    }

    /// The manager facade: the Active gate, the draw_tick countdown ->
    /// capture -> finish chain, the EndCamera tail.
    #[test]
    fn s1_e_17_manager_draw_tick_runs_the_whole_chain() {
        let mut m = CaptureManager::default();
        // Active setter: the disabled gate.
        m.capture_mode_disabled = true;
        assert_eq!(m.set_active(true), None);
        assert!(!m.interface.active);
        m.capture_mode_disabled = false;
        assert_eq!(m.set_active(true), Some("Menu_Open"));
        assert!(m.interface.active);
        // UsingMap is false while inactive.
        m.interface.active = false;
        assert!(!m.using_map());
        // Start the camera through the interface, then tick: the frame-4
        // arm starts the capture, the queue drains, EndCamera unlocks.
        let settings = CaptureSettings {
            area: CaptureRect::new(0, 0, 10, 10),
            ..CaptureSettings::default()
        };
        m.interface
            .start_camera(settings, CaptureRect::new(0, 0, 1000, 1000));
        let mut renderer = StubRenderer;
        let mut started = false;
        let mut finished = false;
        for _ in 0..10 {
            let (start, done) = m.draw_tick(&mut renderer);
            started |= start.is_some();
            finished |= done;
            if finished {
                break;
            }
        }
        assert!(started, "the countdown started the capture");
        assert!(finished, "the queue drained");
        assert!(!m.interface.camera_lock, "EndCamera ran");
        assert!(!m.is_capturing());
    }

    /// `CaptureSettings`' defaults + the timestamp shape
    /// (`CaptureSettings.cs:10-28`).
    #[test]
    fn s1_e_17_settings_defaults_and_timestamp() {
        let s = CaptureSettings::default();
        assert!(s.use_scaling);
        assert!(s.capture_entities);
        assert!(!s.capture_mech);
        assert!(!s.capture_background);
        assert!(!s.camera_space_effects);
        assert_eq!(s.biome, CaptureBiome::DEFAULT_PURITY);
        let s = CaptureSettings::with_timestamp(2026, 11, 18, 6, 5, 9);
        assert_eq!(s.output_name, "Capture 2026-11-18 06_05_09");
    }

    /// The marked-area screen rect (`:141-183`): the tile span ×16
    /// intersected with the screen, offset into screen space.
    #[test]
    fn s1_e_17_marked_area_screen_rect() {
        let mut i = CaptureInterface::default();
        let inf = info();
        assert_eq!(i.marked_area_screen_rect(&inf), None, "unpinned");
        i.edge_a_pinned = true;
        i.edge_b_pinned = true;
        // Tiles (1000,250)-(1010,260): pixels (16000,4000)-(16176,4176).
        i.edge_a = (1000, 250);
        i.edge_b = (1010, 260);
        let r = i.marked_area_screen_rect(&inf).unwrap();
        assert_eq!(r, CaptureRect::new(0, 0, 176, 176));
        // Fully offscreen: no rect.
        i.edge_a = (0, 0);
        i.edge_b = (10, 10);
        assert_eq!(i.marked_area_screen_rect(&inf), None);
    }

    /// The Settings ctor's MarkedAreaColor (`CaptureInterface.cs:37`):
    /// `(0.8,0.8,0.8,0) * 0.3` truncates to RGB 61, alpha 0.
    #[test]
    fn s1_e_17_marked_area_color_truncates() {
        let s = CaptureInterfaceSettings::default();
        assert_eq!(s.marked_area_color & 0xFF, 61);
        assert_eq!((s.marked_area_color >> 8) & 0xFF, 61);
        assert_eq!((s.marked_area_color >> 16) & 0xFF, 61);
        assert_eq!((s.marked_area_color >> 24) & 0xFF, 0);
        assert!(s.pack_image);
        assert!(s.include_entities);
        assert_eq!(s.biome_choice_index, -1);
        assert_eq!(s.screen_anchor, 0);
    }
}