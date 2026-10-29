//! `Main.DrawMenu`: the original immediate-mode menu.
//!
//! Geometry, font and colours follow `Main.DrawMenu` (`Main.cs`):
//!
//! * every item is a `FontAssets.DeathText` string centred on `Main.screenWidth / 2`;
//! * the first row sits at `num2` and rows are `num4` px apart, with a per-item
//!   `array4` y offset — those constants are per screen and live in [`Page::layout`];
//! * `array7` is the per-screen text scale (1.0 main menu, 0.75 settings, 0.78 general,
//!   0.6 interface/video) and multiplies the per-item `menuItemScale`;
//! * `menuItemScale` eases from 0.8 to 1.0 by 0.02 per frame for the focused item and
//!   back for the rest; the text alpha is `255 * (scale * 2 - 1)`, so unfocused items are
//!   dimmer, and the focused item is tinted `(255, 215, 0)`;
//! * each item is drawn five times — four black copies offset by ±2 px and the coloured
//!   pass — which is the original's drop shadow;
//! * the logo sits at `(screenWidth / 2, 100)` with its centre as origin and the
//!   `logoScale`/`logoRotation` easing, and the version string is the bottom-left
//!   `Mouse_Text` line (`Main.DrawVersionNumber`).
//!
//! The world backdrop behind the menu is ours (the original renders the world at the
//! surface); everything above it is the original layout.

use glam::Vec2;
use tr_render::SpriteRenderer;

use crate::assets;
use crate::settings::Settings;

/// `Main.maxMenuItems` is 20; the largest screen here is the 12-row interface list.
pub const MAX_ITEMS: usize = 20;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Page {
    Main,
    Settings,
    General,
    Interface,
    Video,
    Multiplayer,
    Host,
    Join,
}

/// `num2` / `num4` / `array7` of one screen.
#[derive(Clone, Copy)]
pub struct Layout {
    pub y0: f32,
    pub row: f32,
    pub scale: f32,
}

impl Page {
    /// The per-screen constants from `Main.DrawMenu`. Custom screens (multiplayer)
    /// reuse the main-menu spacing.
    pub fn layout(self) -> Layout {
        match self {
            Page::Main => Layout { y0: 220.0, row: 52.0, scale: 1.0 },
            Page::Settings => Layout { y0: 210.0, row: 37.0, scale: 0.75 },
            Page::General => Layout { y0: 250.0, row: 52.0, scale: 0.78 },
            Page::Interface => Layout { y0: 210.0, row: 28.0, scale: 0.6 },
            Page::Video => Layout { y0: 186.0, row: 30.0, scale: 0.6 },
            Page::Multiplayer | Page::Host | Page::Join => {
                Layout { y0: 220.0, row: 52.0, scale: 1.0 }
            }
        }
    }
}

pub struct Field {
    pub label: &'static str,
    pub value: String,
    pub secret: bool,
    pub digits: bool,
}

pub enum Action {
    None,
    Singleplayer,
    Host { port: u16, password: String },
    Join { addr: String, password: String },
    Quit,
}

/// One settings row. Every variant is a `Main`/`ItemSlot` field the screens toggle.
#[derive(Clone, Copy)]
enum Toggle {
    AutoSave,
    AutoPause,
    MapEnabled,
    HidePassword,
    PlayWhenUnfocused,
    PickupText,
    EventProgress,
    PlacementPreview,
    HighlightNew,
    TileGrid,
    GamepadInstructions,
    Parallax,
    FrameSkip,
    LightMode,
    Quality,
    Background,
    Blood,
    MinersWobble,
    TilesSway,
}

enum Act {
    None,
    Nav(Page),
    Out(Action),
    Toggle(Toggle),
    /// A vanilla screen that is not ported yet: clicking shows the reason.
    Hint(&'static str),
}

struct Item {
    label: String,
    /// `array7`: text scale for this row.
    scale: f32,
    /// `array4`: vertical offset in pixels.
    y: f32,
    /// `array5`: horizontal offset in pixels.
    x: f32,
    /// `array[]`: drawn but not clickable (screen titles).
    header: bool,
    act: Act,
}

impl Item {
    fn new(label: impl Into<String>, act: Act) -> Self {
        Self { label: label.into(), scale: 1.0, y: 0.0, x: 0.0, header: false, act }
    }

    fn at(mut self, y: f32) -> Self {
        self.y = y;
        self
    }

    fn scaled(mut self, scale: f32) -> Self {
        self.scale = scale;
        self
    }
}

pub struct Menu {
    pub page: Page,
    /// Keyboard cursor (`focusMenu` while the mouse is elsewhere).
    pub selected: usize,
    /// Item under the mouse, if any.
    pub hover: Option<usize>,
    /// True once the player used the mouse or the arrow keys: until then no row is
    /// focused, which is how `Main.DrawMenu` starts (every item at 0.8).
    focus_keyboard: bool,
    item_scale: [f32; MAX_ITEMS],
    /// Screen-space boxes of the last frame, for hit-testing.
    boxes: Vec<[f32; 4]>,
    pub fields: Vec<Field>,
    pub error: Option<String>,
    pub connecting: bool,
    pub name: String,
    /// Host/join address shown after the server starts.
    pub status: Option<String>,
    /// The settings screens edit this; `App` copies it when the menu closes.
    pub settings: Settings,
    /// "Not ported yet" notice for the vanilla screens we do not have.
    pub hint: Option<String>,
    /// Write `config.json` on every toggle (off in unit tests).
    persist: bool,
    time: f32,
    logo_scale: f32,
    logo_dir: f32,
    logo_speed: f32,
    logo_rot: f32,
    rot_dir: f32,
    rot_speed: f32,
}

impl Menu {
    pub fn new(name: String) -> Self {
        let mut me = Self {
            page: Page::Main,
            selected: 0,
            hover: None,
            focus_keyboard: false,
            item_scale: [0.8; MAX_ITEMS],
            boxes: Vec::new(),
            fields: Vec::new(),
            error: None,
            connecting: false,
            name,
            status: None,
            settings: Settings::load(),
            hint: None,
            persist: true,
            time: 0.0,
            logo_scale: 0.1,
            logo_dir: 1.0,
            logo_speed: 0.0,
            logo_rot: 0.0,
            rot_dir: 1.0,
            rot_speed: 0.0,
        };
        // `TR_PAGE=settings|general|interface|video|multiplayer`: open a screen directly
        // so the layout can be screenshotted deterministically.
        if let Ok(focus) = std::env::var("TR_FOCUS") {
            if let Ok(index) = focus.parse::<usize>() {
                me.selected = index;
                me.focus_keyboard = true;
            }
        }
        if let Ok(page) = std::env::var("TR_PAGE") {
            me.page = match page.to_ascii_lowercase().as_str() {
                "settings" => Page::Settings,
                "general" => Page::General,
                "interface" => Page::Interface,
                "video" => Page::Video,
                "multiplayer" => Page::Multiplayer,
                _ => Page::Main,
            };
        }
        me
    }

    fn open_host(&mut self) {
        self.page = Page::Host;
        self.selected = 0;
        self.fields = vec![
            Field { label: "Port", value: "7777".into(), secret: false, digits: true },
            Field { label: "Password", value: String::new(), secret: true, digits: false },
        ];
    }

    fn open_join(&mut self) {
        self.page = Page::Join;
        self.selected = 0;
        self.fields = vec![
            Field { label: "Server IP", value: "127.0.0.1".into(), secret: false, digits: false },
            Field { label: "Port", value: "7777".into(), secret: false, digits: true },
            Field { label: "Password", value: String::new(), secret: true, digits: false },
        ];
    }

    /// The rows of the current screen, in `Main.DrawMenu` order.
    fn items(&self) -> Vec<Item> {
        let s = &self.settings;
        let on_off = |b: bool, on: &str, off: &str| -> String {
            if b { on.to_string() } else { off.to_string() }
        };
        match self.page {
            Page::Main => vec![
                Item::new("Single Player", Act::Out(Action::Singleplayer)),
                Item::new("Multiplayer", Act::Nav(Page::Multiplayer)),
                Item::new("Achievements", Act::Hint("Achievements are not ported yet")),
                Item::new("Resource Packs", Act::Hint("Resource packs are not ported yet")),
                Item::new("Settings", Act::Nav(Page::Settings)),
                Item::new("Credits", Act::Hint("Credits are not ported yet")),
                Item::new("Exit", Act::Out(Action::Quit)),
            ],
            // `menuMode == 11`.
            Page::Settings => vec![
                Item::new("General", Act::Nav(Page::General)).scaled(0.75),
                Item::new("Interface", Act::Nav(Page::Interface)).scaled(0.75),
                Item::new("Video", Act::Nav(Page::Video)).scaled(0.75),
                Item::new("Volume", Act::Hint("The volume sliders are not ported yet"))
                    .scaled(0.75),
                Item::new("Cursor", Act::Hint("Cursor colour is not ported yet")).scaled(0.75),
                Item::new("Controls", Act::Hint("Key rebinding is not ported yet")).scaled(0.75),
                Item::new("Language", Act::Hint("Only English is bundled")).scaled(0.75),
                Item::new("Back", Act::Nav(Page::Main)).scaled(0.75).at(8.0),
            ],
            // `menuMode == 112`.
            Page::General => vec![
                Item::new(on_off(s.auto_save, "Autosave On", "Autosave Off"), Act::Toggle(Toggle::AutoSave)).scaled(0.78),
                Item::new(on_off(s.auto_pause, "Autopause On", "Autopause Off"), Act::Toggle(Toggle::AutoPause)).scaled(0.78),
                Item::new(on_off(s.map_enabled, "Map Enabled", "Map Disabled"), Act::Toggle(Toggle::MapEnabled)).scaled(0.78),
                Item::new(on_off(s.hide_password, "Passwords: Hidden", "Passwords: Visible"), Act::Toggle(Toggle::HidePassword)).scaled(0.78),
                Item::new(on_off(s.play_when_unfocused, "Pause When Unfocused: Off", "Pause When Unfocused: On"), Act::Toggle(Toggle::PlayWhenUnfocused)).scaled(0.78),
                Item::new("Back", Act::Nav(Page::Settings)).scaled(0.78).at(18.0),
            ],
            // `menuMode == 1112`.
            Page::Interface => vec![
                Item::new(on_off(s.pickup_text, "Pickup Text On", "Pickup Text Off"), Act::Toggle(Toggle::PickupText)).scaled(0.6),
                Item::new(
                    format!("Event Progress Bar {}", ["Off", "Timed", "On"][s.event_progress as usize % 3]),
                    Act::Toggle(Toggle::EventProgress),
                ).scaled(0.6),
                Item::new(on_off(s.placement_preview, "Placement Preview On", "Placement Preview Off"), Act::Toggle(Toggle::PlacementPreview)).scaled(0.6),
                Item::new(on_off(s.highlight_new_items, "Highlight New Items On", "Highlight New Items Off"), Act::Toggle(Toggle::HighlightNew)).scaled(0.6),
                Item::new(on_off(s.tile_grid, "Tile Grid On", "Tile Grid Off"), Act::Toggle(Toggle::TileGrid)).scaled(0.6),
                Item::new(on_off(s.gamepad_instructions, "Gamepad Instructions On", "Gamepad Instructions Off"), Act::Toggle(Toggle::GamepadInstructions)).scaled(0.6),
                Item::new("Minimap Border: Default", Act::Hint("Minimap borders are not ported yet")).scaled(0.6),
                Item::new("Health and Mana Style: Classic", Act::Hint("Health styles are not ported yet")).scaled(0.6),
                Item::new("Portraits Style: Detailed", Act::Hint("Portraits are not ported yet")).scaled(0.6),
                Item::new("Boss Health Bar Numbers: Off", Act::Hint("Boss bar numbers are not ported yet")).scaled(0.6),
                Item::new("Inventory Effects: On", Act::Hint("Inventory effects are not ported yet")).scaled(0.6),
                Item::new("Back", Act::Nav(Page::Settings)).scaled(0.6).at(16.0),
            ],
            // `menuMode == 1111`. Note the original leaves `array7` at 1.0 for the Back
            // row here (only the first ten rows are set to 0.6), so Back is bigger.
            Page::Video => vec![
                Item::new("Resolution", Act::Hint("Window resolution is not ported yet")).scaled(0.6),
                Item::new(on_off(s.parallax, "Parallax On", "Parallax Off"), Act::Toggle(Toggle::Parallax)).scaled(0.6),
                Item::new(["Frame Skip Off", "Frame Skip On", "Frame Skip Subtle"][s.frame_skip as usize % 3], Act::Toggle(Toggle::FrameSkip)).scaled(0.6),
                Item::new(["Lighting: Color", "Lighting: White", "Lighting: Retro", "Lighting: Trippy"][s.light_mode as usize % 4], Act::Toggle(Toggle::LightMode)).scaled(0.6),
                Item::new(["Quality: Auto", "Quality: High", "Quality: Medium", "Quality: Low"][s.quality as usize % 4], Act::Toggle(Toggle::Quality)).scaled(0.6),
                Item::new(on_off(s.background, "Background On", "Background Off"), Act::Toggle(Toggle::Background)).scaled(0.6),
                Item::new(on_off(s.blood, "Blood and Gore On", "Blood and Gore Off"), Act::Toggle(Toggle::Blood)).scaled(0.6),
                Item::new(on_off(s.miners_wobble, "Miner's Wobble: Enabled", "Miner's Wobble: Disabled"), Act::Toggle(Toggle::MinersWobble)).scaled(0.6),
                Item::new(on_off(s.tiles_sway, "Windy Environment: Enabled", "Windy Environment: Disabled"), Act::Toggle(Toggle::TilesSway)).scaled(0.6),
                Item::new("Effects", Act::Hint("The effects screen is not ported yet")).scaled(0.6),
                Item::new("Back", Act::Nav(Page::Settings)).at(8.0),
            ],
            Page::Multiplayer => vec![
                Item::new("Host & Play", Act::Nav(Page::Host)),
                Item::new("Join", Act::Nav(Page::Join)),
                Item::new("Back", Act::Nav(Page::Main)),
            ],
            Page::Host => vec![
                Item::new("Start Server", Act::Out(Action::Host {
                    port: self.fields.first().and_then(|f| f.value.trim().parse().ok()).unwrap_or(7777),
                    password: self.fields.get(1).map(|f| f.value.clone()).unwrap_or_default(),
                })),
                Item::new("Back", Act::Nav(Page::Multiplayer)),
            ],
            Page::Join => {
                let host = self.fields.first().map(|f| f.value.trim().to_string()).unwrap_or_default();
                let port = self.fields.get(1).and_then(|f| f.value.trim().parse().ok()).unwrap_or(7777);
                let addr = if host.contains(':') { host } else { format!("{host}:{port}") };
                vec![
                    Item::new("Connect", Act::Out(Action::Join {
                        addr,
                        password: self.fields.get(2).map(|f| f.value.clone()).unwrap_or_default(),
                    })),
                    Item::new("Back", Act::Nav(Page::Multiplayer)),
                ]
            }
        }
    }

    fn parent(page: Page) -> Option<Page> {
        match page {
            Page::Main => None,
            Page::Settings => Some(Page::Main),
            Page::General | Page::Interface | Page::Video => Some(Page::Settings),
            Page::Multiplayer => Some(Page::Main),
            Page::Host | Page::Join => Some(Page::Multiplayer),
        }
    }

    fn activate(&mut self, index: usize) -> Action {
        let items = self.items();
        let Some(item) = items.get(index) else {
            return Action::None;
        };
        if item.header {
            return Action::None;
        }
        self.hint = None;
        match &item.act {
            Act::None => Action::None,
            Act::Nav(page) => {
                let page = *page;
                if page == Page::Host {
                    self.open_host();
                } else if page == Page::Join {
                    self.open_join();
                } else {
                    self.page = page;
                    self.selected = 0;
                    self.fields.clear();
                }
                self.error = None;
                Action::None
            }
            Act::Out(action) => match action {
                Action::Singleplayer => Action::Singleplayer,
                Action::Quit => Action::Quit,
                Action::Host { port, password } => {
                    Action::Host { port: *port, password: password.clone() }
                }
                Action::Join { addr, password } => {
                    Action::Join { addr: addr.clone(), password: password.clone() }
                }
                Action::None => Action::None,
            },
            Act::Toggle(toggle) => {
                self.toggle(*toggle);
                Action::None
            }
            Act::Hint(message) => {
                self.hint = Some((*message).to_string());
                Action::None
            }
        }
    }

    /// `Main`/`ItemSlot` toggles. Every row changes the stored setting; the ones the
    /// renderer reads (`LightMode`, `BackgroundEnabled`, `MapEnabled`, …) are applied by
    /// `App` when the menu closes.
    fn toggle(&mut self, toggle: Toggle) {
        let s = &mut self.settings;
        match toggle {
            Toggle::AutoSave => s.auto_save = !s.auto_save,
            Toggle::AutoPause => s.auto_pause = !s.auto_pause,
            Toggle::MapEnabled => s.map_enabled = !s.map_enabled,
            Toggle::HidePassword => s.hide_password = !s.hide_password,
            Toggle::PlayWhenUnfocused => s.play_when_unfocused = !s.play_when_unfocused,
            Toggle::PickupText => s.pickup_text = !s.pickup_text,
            Toggle::EventProgress => s.event_progress = (s.event_progress + 1) % 3,
            Toggle::PlacementPreview => s.placement_preview = !s.placement_preview,
            Toggle::HighlightNew => s.highlight_new_items = !s.highlight_new_items,
            Toggle::TileGrid => s.tile_grid = !s.tile_grid,
            Toggle::GamepadInstructions => {
                s.gamepad_instructions = !s.gamepad_instructions
            }
            Toggle::Parallax => s.parallax = !s.parallax,
            Toggle::FrameSkip => s.frame_skip = (s.frame_skip + 1) % 3,
            Toggle::LightMode => s.light_mode = (s.light_mode + 1) % 4,
            Toggle::Quality => s.quality = (s.quality + 1) % 4,
            Toggle::Background => s.background = !s.background,
            Toggle::Blood => s.blood = !s.blood,
            Toggle::MinersWobble => s.miners_wobble = !s.miners_wobble,
            Toggle::TilesSway => s.tiles_sway = !s.tiles_sway,
        }
        if self.persist {
            let _ = s.save();
        }
    }

    pub fn key(&mut self, code: winit::keyboard::KeyCode) -> Action {
        use winit::keyboard::KeyCode as K;
        let count = self.items().len();
        match code {
            K::ArrowUp | K::KeyW => {
                if count > 0 {
                    self.selected = (self.selected + count - 1) % count;
                    self.focus_keyboard = true;
                }
                return Action::None;
            }
            K::ArrowDown | K::KeyS => {
                if count > 0 {
                    self.selected = (self.selected + 1) % count;
                    self.focus_keyboard = true;
                }
                return Action::None;
            }
            K::Tab => {
                if !self.fields.is_empty() && !self.connecting {
                    self.selected = 0;
                }
                return Action::None;
            }
            K::Backspace => {
                if matches!(self.page, Page::Host | Page::Join) && !self.connecting {
                    if let Some(f) = self.fields.get_mut(0) {
                        f.value.pop();
                    }
                }
                return Action::None;
            }
            K::Escape => {
                if let Some(parent) = Self::parent(self.page) {
                    self.page = parent;
                    self.selected = 0;
                    self.fields.clear();
                    self.hint = None;
                }
                return Action::None;
            }
            K::Enter | K::NumpadEnter => {}
            _ => return Action::None,
        }
        self.activate(self.selected)
    }

    /// Text typed into the host/join fields (the original uses `UIInput`).
    pub fn character(&mut self, c: char) {
        if !matches!(self.page, Page::Host | Page::Join) || self.connecting {
            return;
        }
        if self.selected != 0 {
            return;
        }
        let field = if self.fields.len() > 1 { 1 } else { 0 };
        if let Some(f) = self.fields.get_mut(field) {
            if f.digits && !c.is_ascii_digit() {
                return;
            }
            if f.value.len() < 64 {
                f.value.push(c);
            }
        }
    }

    /// Hit-tests the boxes recorded by the last [`Menu::draw`].
    pub fn update_hover(&mut self, pos: Vec2) {
        self.hover = self.boxes.iter().position(|b| {
            pos.x >= b[0] && pos.x < b[0] + b[2] && pos.y >= b[1] && pos.y < b[1] + b[3]
        });
        if let Some(i) = self.hover {
            self.selected = i;
            self.focus_keyboard = true;
        }
    }

    /// Left click on the hovered row.
    pub fn click(&mut self) -> Action {
        match self.hover {
            Some(i) => self.activate(i),
            None => Action::None,
        }
    }

    fn step_animation(&mut self, dt: f32) {
        let f = (dt * 60.0).clamp(0.0, 3.0);
        self.time += dt;

        // `Main.DrawMenu`: the logo scale eases between 1.0 and 1.35 and the rotation
        // between ±0.08 rad; both speeds ramp by ±1 per frame up to ±50 / ±20.
        if self.logo_scale < 0.98 {
            self.logo_scale *= 1.05;
        }
        self.logo_scale += self.logo_speed * 9.0e-6 * f;
        if self.logo_scale > 1.35 {
            self.logo_dir = -1.0;
        } else if self.logo_scale < 1.0 {
            self.logo_dir = 1.0;
        }
        if self.logo_speed < 50.0 && self.logo_dir == 1.0 {
            self.logo_speed += f;
        } else if self.logo_speed > -50.0 && self.logo_dir == -1.0 {
            self.logo_speed -= f;
        }
        self.logo_rot += self.rot_speed * 4.0e-6 * f;
        if self.logo_rot > 0.08 {
            self.rot_dir = -1.0;
        } else if self.logo_rot < -0.08 {
            self.rot_dir = 1.0;
        }
        if self.rot_speed < 20.0 && self.rot_dir == 1.0 {
            self.rot_speed += f;
        } else if self.rot_speed > -20.0 && self.rot_dir == -1.0 {
            self.rot_speed -= f;
        }
    }

    /// Draws the menu over the paused world.
    pub fn draw(
        &mut self,
        r: &mut SpriteRenderer,
        lookups: &assets::Lookups,
        screen: Vec2,
        dt: f32,
    ) {
        self.step_animation(dt);

        // `TextureAssets.Logo` at (screenWidth / 2, 100), centre origin.
        if let Some((uv, w, h)) = lookups.logo {
            let scale = self.logo_scale;
            let (dw, dh) = (w * scale, h * scale);
            let pos = [(screen.x - dw) * 0.5, 100.0 - dh * 0.5];
            r.push_ui_sprite_rot(pos, [dw, dh], uv, [1.0; 4], self.logo_rot);
        }

        let items = self.items();
        let layout = self.page.layout();
        let Some(font) = lookups.death_font.as_ref() else {
            return;
        };
        let focus = self
            .hover
            .or(if self.focus_keyboard { Some(self.selected) } else { None })
            .filter(|i| *i < items.len());

        // `menuItemScale`: +0.02 per frame toward 1.0 for the focused row, -0.02 back.
        for (i, scale) in self.item_scale.iter_mut().enumerate() {
            let target = if Some(i) == focus { 1.0 } else { 0.8 };
            let step = 0.02 * (dt * 60.0).clamp(0.0, 3.0);
            if *scale < target {
                *scale = (*scale + step).min(target);
            } else if *scale > target {
                *scale = (*scale - step).max(target);
            }
        }

        self.boxes.clear();
        let cx = screen.x * 0.5;
        for (i, item) in items.iter().enumerate() {
            let measured = crate::font_width(font, &item.label, 1.0);
            let origin = [measured * 0.5, font.line_spacing * 0.5];
            let scale_now = self.item_scale[i] * item.scale;
            let row_y = layout.y0 + layout.row * i as f32 + item.y;
            // `origin.Y * array7` keeps the row centred while `menuItemScale` eases.
            let top = [
                cx + item.x - origin[0] * scale_now,
                row_y + origin[1] * item.scale * (1.0 - self.item_scale[i]),
            ];
            self.boxes.push([
                cx - origin[0] * item.scale + item.x,
                row_y,
                measured * item.scale,
                50.0 * item.scale,
            ]);

            // `num101 = 255 * (menuItemScale * 2 - 1)`.
            let alpha = (255.0 * (self.item_scale[i] * 2.0 - 1.0)).clamp(0.0, 255.0);
            let dim = (255.0 - alpha) / 255.0;
            let mut main = [1.0f32, 1.0, 1.0];
            if Some(i) == focus {
                let t = alpha / 255.0;
                main[0] = main[0] * (1.0 - t) + t;
                main[1] = main[1] * (1.0 - t) + (215.0 / 255.0) * t;
                main[2] *= 1.0 - t;
            }
            let tint = [
                (main[0] - dim).max(0.0),
                (main[1] - dim).max(0.0),
                (main[2] - dim).max(0.0),
                alpha / 255.0,
            ];
            // Four black passes at ±2 px, then the coloured pass.
            let shadow = [0.0, 0.0, 0.0, alpha / 255.0];
            for (dx, dy) in [(-2.0, 0.0), (2.0, 0.0), (0.0, -2.0), (0.0, 2.0)] {
                crate::push_font_text(
                    r,
                    font,
                    top[0] + dx,
                    top[1] + dy,
                    scale_now,
                    shadow,
                    &item.label,
                );
            }
            crate::push_font_text(r, font, top[0], top[1], scale_now, tint, &item.label);
        }

        // Host/join text fields (the original uses `UIInput` with `Mouse_Text`).
        if matches!(self.page, Page::Host | Page::Join) {
            let start = cx - 230.0;
            let mut y = layout.y0 + layout.row * items.len() as f32 + 10.0;
            for (i, f) in self.fields.iter().enumerate() {
                let active = i == 1 && self.selected == 0 && !self.connecting;
                crate::hud_text(r, lookups, start, y, 0.9, [0.78, 0.82, 0.88, 1.0], f.label);
                let shown = if f.secret {
                    "*".repeat(f.value.chars().count())
                } else {
                    f.value.clone()
                };
                let box_x = start + 140.0;
                r.push_ui_rect(
                    [box_x, y - 6.0],
                    [300.0, 34.0],
                    if active { [0.85, 0.75, 0.3, 0.95] } else { [0.15, 0.16, 0.2, 0.95] },
                );
                crate::hud_text(r, lookups, box_x + 8.0, y, 0.9, [1.0; 4], &shown);
                y += 44.0;
            }
        }

        // `Main.DrawVersionNumber`: bottom-left `Mouse_Text`.
        if let Some(ui) = lookups.ui_font.as_ref() {
            let text = "v1.4.5.8";
            let h = ui.line_spacing;
            crate::hud_text(
                r,
                lookups,
                10.0,
                screen.y - h - 2.0,
                1.0,
                [0.85, 0.85, 0.85, 0.7],
                text,
            );
        }

        // Not-ported notices and connection state (ours, not in the original).
        let mut note_y = screen.y - 64.0;
        if let Some(hint) = &self.hint {
            let w = crate::hud_text_width(r, lookups, hint, 0.8);
            crate::hud_text(
                r,
                lookups,
                (screen.x - w) * 0.5,
                note_y,
                0.8,
                [1.0, 0.8, 0.45, 1.0],
                hint,
            );
            note_y -= 24.0;
        }
        if let Some(err) = &self.error {
            let w = crate::hud_text_width(r, lookups, err, 0.9);
            crate::hud_text(
                r,
                lookups,
                (screen.x - w) * 0.5,
                note_y,
                0.9,
                [1.0, 0.45, 0.4, 1.0],
                err,
            );
            note_y -= 24.0;
        }
        if self.connecting {
            let msg = "Connecting…";
            let w = crate::hud_text_width(r, lookups, msg, 1.0);
            crate::hud_text(
                r,
                lookups,
                (screen.x - w) * 0.5,
                screen.y * 0.5 - 10.0,
                1.0,
                [1.0, 0.9, 0.5, 1.0],
                msg,
            );
        }
        if let Some(status) = &self.status {
            let w = crate::hud_text_width(r, lookups, status, 0.9);
            crate::hud_text(
                r,
                lookups,
                (screen.x - w) * 0.5,
                note_y,
                0.9,
                [0.7, 0.95, 0.7, 1.0],
                status,
            );
        }
    }
}

/// `Player.name` style random character names ("Zealot", "Terra", …): two syllables
/// from the game's name lists, so every launch shows a different character.
pub fn random_name() -> String {
    const A: &[&str] = &[
        "Al", "Ai", "Ar", "Bel", "Bra", "Ca", "Cle", "Da", "Dra", "E", "El", "Fa", "Fi", "Ga",
        "Gra", "Ha", "Ig", "Ja", "Ka", "Kri", "Lu", "Ly", "Ma", "Mi", "Na", "Ne", "Ni", "No",
        "O", "Or", "Pa", "Qu", "Ra", "Re", "Ri", "Sa", "Se", "Si", "Sky", "Ta", "Te", "Tha",
        "Ti", "To", "Tri", "Ul", "Va", "Ve", "Vi", "Wa", "Xa", "Za", "Ze",
    ];
    const B: &[&str] = &[
        "bert", "dan", "dor", "dric", "fen", "fred", "gar", "gus", "ius", "lan", "las", "lios",
        "lon", "mar", "mon", "nar", "nath", "ndra", "nel", "nia", "nor", "phin", "ran", "rax",
        "ric", "ril", "ron", "rus", "san", "sin", "ster", "sus", "tia", "tor", "tra", "tus",
        "vin", "vor", "wick", "wyn", "zar",
    ];
    let seed = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(0)
        ^ ((std::process::id() as u64) << 32);
    let mut s = seed | 1;
    let mut next = || {
        s ^= s << 13;
        s ^= s >> 7;
        s ^= s << 17;
        s
    };
    let a = A[(next() % A.len() as u64) as usize];
    let b = B[(next() % B.len() as u64) as usize];
    format!("{a}{b}")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn menu(page: Page) -> Menu {
        let mut m = Menu::new("Test".into());
        m.page = page;
        m.persist = false;
        m
    }

    #[test]
    fn screens_use_the_original_row_metrics() {
        // `num2`, `num4` and `array7` from `Main.DrawMenu`.
        let main = Page::Main.layout();
        assert_eq!((main.y0, main.row), (220.0, 52.0));
        let settings = Page::Settings.layout();
        assert_eq!((settings.y0, settings.row), (210.0, 37.0));
        let general = Page::General.layout();
        assert_eq!((general.y0, general.row), (250.0, 52.0));
        let interface = Page::Interface.layout();
        assert_eq!((interface.y0, interface.row), (210.0, 28.0));
        let video = Page::Video.layout();
        assert_eq!((video.y0, video.row), (186.0, 30.0));
    }

    #[test]
    fn main_menu_has_the_seven_vanilla_rows() {
        let items = menu(Page::Main).items();
        let labels: Vec<&str> = items.iter().map(|i| i.label.as_str()).collect();
        assert_eq!(
            labels,
            [
                "Single Player",
                "Multiplayer",
                "Achievements",
                "Resource Packs",
                "Settings",
                "Credits",
                "Exit"
            ]
        );
        assert!(items.iter().all(|i| i.scale == 1.0));
    }

    #[test]
    fn settings_screen_matches_the_vanilla_rows() {
        let items = menu(Page::Settings).items();
        let labels: Vec<&str> = items.iter().map(|i| i.label.as_str()).collect();
        assert_eq!(
            labels,
            ["General", "Interface", "Video", "Volume", "Cursor", "Controls", "Language", "Back"]
        );
        assert!(items.iter().all(|i| i.scale == 0.75));
        assert_eq!(items[7].y, 8.0, "array4[num5 - 1] = 8");
    }

    #[test]
    fn general_rows_are_toggles_with_state_labels() {
        let mut m = menu(Page::General);
        assert_eq!(m.items().len(), 6);
        assert_eq!(m.items()[0].label, "Autosave On");
        m.settings.auto_save = false;
        assert_eq!(m.items()[0].label, "Autosave Off");
        assert_eq!(m.items()[5].y, 18.0, "array4[num5 - 1] = 18");
    }

    #[test]
    fn interface_screen_has_twelve_rows() {
        let m = menu(Page::Interface);
        let items = m.items();
        assert_eq!(items.len(), 12);
        assert!(items[..11].iter().all(|i| i.scale == 0.6));
        assert_eq!(items[11].label, "Back");
        assert_eq!(items[11].y, 16.0);
    }

    #[test]
    fn video_screen_leaves_back_at_scale_one() {
        let m = menu(Page::Video);
        let items = m.items();
        assert_eq!(items.len(), 11);
        assert!(items[..10].iter().all(|i| i.scale == 0.6));
        // The original only sets `array7` for the first ten rows.
        assert_eq!(items[10].label, "Back");
        assert_eq!(items[10].scale, 1.0);
        assert_eq!(items[10].y, 8.0);
    }

    #[test]
    fn toggles_cycle_the_vanilla_modes() {
        let mut m = menu(Page::Video);
        m.toggle(Toggle::LightMode);
        assert_eq!(m.settings.light_mode, 1);
        m.toggle(Toggle::LightMode);
        m.toggle(Toggle::LightMode);
        m.toggle(Toggle::LightMode);
        assert_eq!(m.settings.light_mode, 0, "four modes");
        m.toggle(Toggle::FrameSkip);
        m.toggle(Toggle::FrameSkip);
        m.toggle(Toggle::FrameSkip);
        assert_eq!(m.settings.frame_skip, 0, "three modes");
    }

    #[test]
    fn back_navigates_to_the_parent_screen() {
        let mut m = menu(Page::General);
        m.activate(5);
        assert_eq!(m.page, Page::Settings);
        let mut m = menu(Page::Settings);
        m.activate(7);
        assert_eq!(m.page, Page::Main);
        let mut m = menu(Page::Video);
        m.activate(10);
        assert_eq!(m.page, Page::Settings);
    }

    #[test]
    fn unported_screens_show_a_notice_instead_of_navigating() {
        let mut m = menu(Page::Settings);
        m.activate(3); // Volume
        assert_eq!(m.page, Page::Settings);
        assert!(m.hint.is_some());
    }
}
