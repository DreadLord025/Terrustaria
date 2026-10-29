//! Main menu (`Main.MenuUI`): singleplayer/multiplayer selection before entering a
//! world, with the original game's logo and fonts. Multiplayer exposes host port and
//! password or a join address, like the original's server settings.

use glam::Vec2;
use tr_render::SpriteRenderer;

use crate::assets;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Page {
    Main,
    Multiplayer,
    Host,
    Join,
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

pub struct Menu {
    pub page: Page,
    pub selected: usize,
    pub focus: usize,
    pub fields: Vec<Field>,
    pub error: Option<String>,
    pub connecting: bool,
    pub name: String,
    /// Host/join address shown after the server starts.
    pub status: Option<String>,
}

impl Menu {
    pub fn new(name: String) -> Self {
        Self {
            page: Page::Main,
            selected: 0,
            focus: 0,
            fields: Vec::new(),
            error: None,
            connecting: false,
            name,
            status: None,
        }
    }

    fn items(&self) -> &'static [&'static str] {
        match self.page {
            Page::Main => &["Singleplayer", "Multiplayer", "Quit"],
            Page::Multiplayer => &["Host & Play", "Join", "Back"],
            Page::Host => &["Start Server", "Back"],
            Page::Join => &["Connect", "Back"],
        }
    }

    fn open_host(&mut self) {
        self.page = Page::Host;
        self.selected = 0;
        self.focus = 0;
        self.fields = vec![
            Field { label: "Port", value: "7777".into(), secret: false, digits: true },
            Field { label: "Password", value: String::new(), secret: true, digits: false },
        ];
    }

    fn open_join(&mut self) {
        self.page = Page::Join;
        self.selected = 0;
        self.focus = 0;
        self.fields = vec![
            Field { label: "Server IP", value: "127.0.0.1".into(), secret: false, digits: false },
            Field { label: "Port", value: "7777".into(), secret: false, digits: true },
            Field { label: "Password", value: String::new(), secret: true, digits: false },
        ];
    }

    pub fn key(&mut self, code: winit::keyboard::KeyCode) -> Action {
        use winit::keyboard::KeyCode as K;
        match code {
            K::ArrowUp | K::KeyW => {
                let n = self.items().len();
                self.selected = (self.selected + n - 1) % n;
                return Action::None;
            }
            K::ArrowDown | K::KeyS => {
                let n = self.items().len();
                self.selected = (self.selected + 1) % n;
                return Action::None;
            }
            K::Tab => {
                if !self.fields.is_empty() && !self.connecting {
                    self.focus = (self.focus + 1) % self.fields.len();
                }
                return Action::None;
            }
            K::Backspace => {
                if matches!(self.page, Page::Host | Page::Join) && !self.connecting {
                    if let Some(f) = self.fields.get_mut(self.focus) {
                        f.value.pop();
                    }
                }
                return Action::None;
            }
            K::Escape => {
                match self.page {
                    Page::Main => {}
                    Page::Multiplayer => self.page = Page::Main,
                    Page::Host | Page::Join => self.page = Page::Multiplayer,
                }
                self.selected = 0;
                return Action::None;
            }
            K::Enter | K::NumpadEnter => {}
            _ => return Action::None,
        }
        // Enter activates the selected item.
        match self.page {
            Page::Main => match self.selected {
                0 => Action::Singleplayer,
                1 => {
                    self.page = Page::Multiplayer;
                    self.selected = 0;
                    Action::None
                }
                _ => Action::Quit,
            },
            Page::Multiplayer => match self.selected {
                0 => {
                    self.open_host();
                    Action::None
                }
                1 => {
                    self.open_join();
                    Action::None
                }
                _ => {
                    self.page = Page::Main;
                    self.selected = 0;
                    Action::None
                }
            },
            Page::Host => {
                if self.selected == 1 {
                    self.page = Page::Multiplayer;
                    return Action::None;
                }
                let port = self.fields[0].value.trim().parse::<u16>().unwrap_or(7777);
                let password = self.fields[1].value.clone();
                self.connecting = true;
                Action::Host { port, password }
            }
            Page::Join => {
                if self.selected == 1 {
                    self.page = Page::Multiplayer;
                    return Action::None;
                }
                let host = self.fields[0].value.trim().to_string();
                let port = self.fields[1].value.trim().parse::<u16>().unwrap_or(7777);
                let password = self.fields[2].value.clone();
                if host.is_empty() {
                    self.error = Some("enter a server address".into());
                    return Action::None;
                }
                let addr = if host.contains(':') {
                    host
                } else {
                    format!("{host}:{port}")
                };
                self.connecting = true;
                Action::Join { addr, password }
            }
        }
    }

    pub fn character(&mut self, c: char) {
        if !matches!(self.page, Page::Host | Page::Join) || self.connecting {
            return;
        }
        if self.selected != 0 {
            return;
        }
        if let Some(f) = self.fields.get_mut(self.focus) {
            if f.digits && !c.is_ascii_digit() {
                return;
            }
            if f.value.len() < 64 {
                f.value.push(c);
            }
        }
    }

    /// Draws the menu over the paused world.
    #[allow(clippy::too_many_arguments)]
    pub fn draw(
        &self,
        r: &mut SpriteRenderer,
        lookups: &assets::Lookups,
        screen: Vec2,
    ) {
        let v = screen.y / 900.0;
        r.push_ui_rect([0.0, 0.0], [screen.x, screen.y], [0.02, 0.03, 0.06, 0.62]);
        // Logo, centred near the top like the original title screen.
        let mut y = 70.0 * v;
        if let Some((uv, w, h)) = lookups.logo {
            let scale = (screen.x * 0.55 / w).min(1.0);
            let (dw, dh) = (w * scale, h * scale);
            r.push_ui_sprite([(screen.x - dw) * 0.5, y], [dw, dh], uv, [1.0; 4]);
            y += dh + 26.0 * v;
        } else {
            let title = "TERRARIA";
            let tw = crate::hud_text_width(r, lookups, title, 3.0);
            crate::hud_text(
                r,
                lookups,
                (screen.x - tw) * 0.5,
                y,
                3.0,
                [1.0, 0.95, 0.7, 1.0],
                title,
            );
            y += 90.0 * v;
        }
        y = y.max(240.0 * v) + 46.0 * v;

        match self.page {
            Page::Main | Page::Multiplayer => {
                for (i, item) in self.items().iter().enumerate() {
                    let sel = i == self.selected;
                    let scale = 1.15;
                    let tw = crate::hud_text_width(r, lookups, item, scale);
                    let x = (screen.x - tw) * 0.5;
                    let color = if sel { [1.0, 0.92, 0.45, 1.0] } else { [0.82, 0.84, 0.88, 1.0] };
                    if sel {
                        r.push_ui_rect(
                            [x - 18.0, y - 4.0],
                            [tw + 36.0, 40.0 * v * 0.85],
                            [0.95, 0.85, 0.35, 0.18],
                        );
                    }
                    crate::hud_text(r, lookups, x + 1.0, y + 1.0, scale, [0.0, 0.0, 0.0, 1.0], item);
                    crate::hud_text(r, lookups, x, y, scale, color, item);
                    y += 52.0 * v;
                }
                if self.page == Page::Multiplayer {
                    let hint = "Host the world on a port, or join a server by IP.";
                    let tw = crate::hud_text_width(r, lookups, hint, 0.85);
                    crate::hud_text(
                        r,
                        lookups,
                        (screen.x - tw) * 0.5,
                        y + 10.0,
                        0.85,
                        [0.7, 0.75, 0.8, 1.0],
                        hint,
                    );
                }
            }
            Page::Host | Page::Join => {
                let start = screen.x * 0.5 - 230.0;
                for (i, f) in self.fields.iter().enumerate() {
                    let focused = i == self.focus && self.selected == 0 && !self.connecting;
                    crate::hud_text(
                        r,
                        lookups,
                        start,
                        y,
                        0.9,
                        [0.75, 0.8, 0.85, 1.0],
                        f.label,
                    );
                    let shown = if f.secret {
                        "*".repeat(f.value.chars().count())
                    } else {
                        f.value.clone()
                    };
                    let box_x = start + 140.0;
                    let box_w = 300.0;
                    r.push_ui_rect(
                        [box_x, y - 6.0],
                        [box_w, 34.0],
                        if focused {
                            [0.85, 0.75, 0.3, 0.95]
                        } else {
                            [0.15, 0.16, 0.2, 0.95]
                        },
                    );
                    crate::hud_text(
                        r,
                        lookups,
                        box_x + 8.0,
                        y,
                        0.9,
                        [1.0, 1.0, 1.0, 1.0],
                        &shown,
                    );
                    y += 48.0 * v;
                }
                for (i, item) in self.items().iter().enumerate() {
                    let sel = i == self.selected;
                    let tw = crate::hud_text_width(r, lookups, item, 1.1);
                    let x = (screen.x - tw) * 0.5;
                    let color = if sel { [1.0, 0.92, 0.45, 1.0] } else { [0.82, 0.84, 0.88, 1.0] };
                    crate::hud_text(r, lookups, x + 1.0, y + 1.0, 1.1, [0.0, 0.0, 0.0, 1.0], item);
                    crate::hud_text(r, lookups, x, y, 1.1, color, item);
                    y += 48.0 * v;
                }
                let hint = "Tab: next field   Enter: confirm   Esc: back";
                let tw = crate::hud_text_width(r, lookups, hint, 0.8);
                crate::hud_text(
                    r,
                    lookups,
                    (screen.x - tw) * 0.5,
                    y + 6.0,
                    0.8,
                    [0.6, 0.65, 0.72, 1.0],
                    hint,
                );
            }
        }

        let mut line_y = screen.y - 74.0 * v;
        if self.connecting {
            let msg = "Connecting…";
            let tw = crate::hud_text_width(r, lookups, msg, 1.0);
            crate::hud_text(
                r,
                lookups,
                (screen.x - tw) * 0.5,
                screen.y * 0.5 - 10.0,
                1.0,
                [1.0, 0.9, 0.5, 1.0],
                msg,
            );
        }
        if let Some(err) = &self.error {
            let tw = crate::hud_text_width(r, lookups, err, 0.9);
            crate::hud_text(
                r,
                lookups,
                (screen.x - tw) * 0.5,
                screen.y - 110.0 * v,
                0.9,
                [1.0, 0.45, 0.4, 1.0],
                err,
            );
        }
        if let Some(status) = &self.status {
            let tw = crate::hud_text_width(r, lookups, status, 0.9);
            crate::hud_text(
                r,
                lookups,
                (screen.x - tw) * 0.5,
                line_y,
                0.9,
                [0.7, 0.95, 0.7, 1.0],
                status,
            );
            line_y += 26.0 * v;
        }
        let name = format!("Character: {}", self.name);
        let tw = crate::hud_text_width(r, lookups, &name, 0.85);
        crate::hud_text(
            r,
            lookups,
            (screen.x - tw) * 0.5,
            line_y,
            0.85,
            [0.85, 0.87, 0.9, 1.0],
            &name,
        );
        let ver = "Terraria-Rust  ·  Milestone 6";
        let tw = crate::hud_text_width(r, lookups, ver, 0.7);
        crate::hud_text(
            r,
            lookups,
            (screen.x - tw) * 0.5,
            screen.y - 26.0,
            0.7,
            [0.55, 0.58, 0.65, 1.0],
            ver,
        );
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
