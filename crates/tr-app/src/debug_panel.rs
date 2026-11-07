//! The debug cheat-sheet panel v2 — the visual selection with the item/NPC
//! icons from the atlas. The scrollable lists show the icon + the name for
//! every entry; the click selects/gives. The self-contained module (delete
//! this file + the mod line + the call sites to remove).
//!
//! Open/close: F3. The tabs: Items / NPCs / Bosses / World.

use crate::app::App;
use crate::assets::Lookups;
use crate::item::ItemSlot;
use crate::npc::{Npc, NpcKind};
use glam::Vec2;
use tr_core::TILE_PX;
use tr_render::SpriteRenderer;

// ---- The state ----

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tab {
    Items,
    Npcs,
    Bosses,
    World,
}

const TABS: [(&str, Tab); 4] = [
    ("Items", Tab::Items),
    ("NPCs", Tab::Npcs),
    ("Bosses", Tab::Bosses),
    ("World", Tab::World),
];

pub struct DebugPanel {
    pub open: bool,
    pub tab: Tab,
    /// The scroll offset in rows for the current tab.
    pub scroll: f32,
    /// The search/filter text (the typed characters filter the list).
    pub filter: String,
    /// The panel's top-left corner (the draggable position).
    pub pos: Vec2,
    /// The drag offset (the cursor minus the panel origin during the drag).
    pub drag_offset: Option<Vec2>,
}

impl DebugPanel {
    pub fn new() -> Self {
        Self {
            open: false,
            tab: Tab::Items,
            scroll: 0.0,
            filter: String::new(),
            pos: Vec2::new(900.0, 60.0),
            drag_offset: None,
        }
    }
}
impl Default for DebugPanel {
    fn default() -> Self {
        Self::new()
    }
}

// ---- The layout ----
const PW: f32 = 320.0;
const PH: f32 = 440.0;
const TAB_H: f32 = 20.0;
const ROW_H: f32 = 22.0;
const ICON: f32 = 18.0;
const PAD: f32 = 6.0;
const TITLE_H: f32 = 18.0;
const FILTER_H: f32 = 18.0;

fn panel_rect(pos: Vec2) -> [f32; 4] {
    [pos.x, pos.y, PW, PH]
}

fn content_rect(panel: &[f32; 4]) -> [f32; 4] {
    [
        panel[0] + PAD,
        panel[1] + TITLE_H + TAB_H + FILTER_H + PAD,
        panel[2] - PAD * 2.0,
        panel[3] - TITLE_H - TAB_H - FILTER_H - PAD * 2.0,
    ]
}

fn pt_in(p: Vec2, r: [f32; 4]) -> bool {
    p.x >= r[0] && p.x <= r[0] + r[2] && p.y >= r[1] && p.y <= r[1] + r[3]
}

// ---- The colours ----
const W: [f32; 4] = [1.0; 4];
const G: [f32; 4] = [0.4, 1.0, 0.4, 1.0];
const R: [f32; 4] = [1.0, 0.4, 0.4, 1.0];
const Y: [f32; 4] = [1.0, 1.0, 0.4, 1.0];
const C: [f32; 4] = [0.4, 1.0, 1.0, 1.0];
const GR: [f32; 4] = [0.6; 4];

fn txt(r: &mut SpriteRenderer, l: &Lookups, x: f32, y: f32, s: f32, c: [f32; 4], t: &str) {
    crate::ui::text::hud_text(r, l, x, y, s, c, t);
}

/// Draws the item/NPC icon from the atlas (the 18x18 thumbnail).
fn icon(r: &mut SpriteRenderer, sheet_name: &str, x: f32, y: f32, size: f32) {
    if let Some(uv) = r.uv(sheet_name, 0.0, 0.0, 100.0, 100.0) {
        r.push_sprite([x, y], [size, size], uv, [1.0; 4]);
    }
}

// ---- The data lists ----

/// All the item names/ids from the registry (the lazily built index).
fn all_items() -> Vec<(u16, &'static str)> {
    crate::itemdata::ITEMS
        .iter()
        .filter(|d| d.id > 0 && !d.name.is_empty())
        .map(|d| (d.id, d.name))
        .collect()
}

fn filtered_items(filter: &str) -> Vec<(u16, &'static str)> {
    let all = all_items();
    if filter.is_empty() {
        return all;
    }
    let f = filter.to_lowercase();
    all.into_iter()
        .filter(|(_, name)| name.to_lowercase().contains(&f))
        .collect()
}

const NPC_LIST: [(NpcKind, &str, &str); 20] = [
    (NpcKind::Slime, "Blue Slime", "NPC_1"),
    (NpcKind::Zombie, "Zombie", "NPC_3"),
    (NpcKind::DemonEye, "Demon Eye", "NPC_2"),
    (NpcKind::Skeleton, "Skeleton", "NPC_21"),
    (NpcKind::CaveBat, "Cave Bat", "NPC_49"),
    (NpcKind::UndeadMiner, "Undead Miner", "NPC_44"),
    (NpcKind::DevourerHead, "Devourer", "NPC_7"),
    (NpcKind::GiantWormHead, "Giant Worm", "NPC_10"),
    (NpcKind::EaterOfSouls, "Eater of Souls", "NPC_6"),
    (NpcKind::Hornet, "Hornet", "NPC_42"),
    (NpcKind::MeteorHead, "Meteor Head", "NPC_23"),
    (NpcKind::GoblinSorcerer, "Goblin Sorc.", "NPC_29"),
    (NpcKind::Tim, "Tim", "NPC_45"),
    (NpcKind::DarkCaster, "Dark Caster", "NPC_32"),
    (NpcKind::FireImp, "Fire Imp", "NPC_24"),
    (NpcKind::Piranha, "Piranha", "NPC_58"),
    (NpcKind::Goldfish, "Goldfish", "NPC_55"),
    (NpcKind::Shark, "Shark", "NPC_65"),
    (NpcKind::GoblinTinkerer, "Gob. Tinkerer", "NPC_107"),
    (NpcKind::BoundGoblin, "Bound Goblin", "NPC_105"),
];

const BOSS_LIST: [(NpcKind, &str, &str); 12] = [
    (NpcKind::KingSlime, "King Slime", "NPC_50"),
    (NpcKind::EyeOfCthulhu, "Eye of Cthulhu", "NPC_4"),
    (NpcKind::BrainOfCthulhu, "Brain of Cthulhu", "NPC_266"),
    (NpcKind::EaterOfWorldsHead, "Eater of Worlds", "NPC_13"),
    (NpcKind::QueenBee, "Queen Bee", "NPC_222"),
    (NpcKind::SkeletronHead, "Skeletron", "NPC_35"),
    (NpcKind::WallOfFlesh, "Wall of Flesh", "NPC_113"),
    (NpcKind::TheDestroyer, "The Destroyer", "NPC_134"),
    (NpcKind::Retinazer, "Retinazer", "NPC_125"),
    (NpcKind::Spazmatism, "Spazmatism", "NPC_126"),
    (NpcKind::SkeletronPrime, "Skeletron Prime", "NPC_127"),
    (NpcKind::Plantera, "Plantera", "NPC_262"),
];

// ---- The draw ----

pub fn draw(
    r: &mut SpriteRenderer,
    lookups: &Lookups,
    screen: Vec2,
    panel: &DebugPanel,
    npcs: &[Npc],
    world: &tr_core::World,
) {
    if !panel.open {
        return;
    }
    let pn = panel_rect(panel.pos);
    let ct = content_rect(&pn);

    // The title bar (the drag zone) — above the tabs.
    txt(r, lookups, pn[0] + PAD, pn[1] + 2.0, 0.9, C, "DEBUG [F3]");

    // The tabs — below the title.
    let tab_y = pn[1] + TITLE_H;
    let tw = PW / TABS.len() as f32;
    for (i, (label, tab)) in TABS.iter().enumerate() {
        let c = if panel.tab == *tab { G } else { GR };
        txt(
            r,
            lookups,
            pn[0] + i as f32 * tw + 4.0,
            tab_y + 2.0,
            0.75,
            c,
            label,
        );
    }

    // The filter field.
    txt(
        r,
        lookups,
        ct[0],
        ct[1],
        0.75,
        C,
        &format!("Search: {}", panel.filter),
    );

    // The visible rows.
    let list_top = ct[1] + FILTER_H;
    let visible = ((ct[1] + ct[3] - list_top) / ROW_H) as usize;
    let scroll_row = panel.scroll as usize;

    match panel.tab {
        Tab::Items => {
            let items = filtered_items(&panel.filter);
            let total = items.len();
            for vi in 0..visible {
                let idx = scroll_row + vi;
                if idx >= total {
                    break;
                }
                let (id, name) = items[idx];
                let y = list_top + vi as f32 * ROW_H;
                // The icon.
                icon(r, &format!("Item_{}", id), ct[0], y, ICON);
                // The name.
                txt(
                    r,
                    lookups,
                    ct[0] + ICON + 4.0,
                    y + 2.0,
                    0.7,
                    W,
                    &format!("{} [{}]", name, id),
                );
            }
            // The scroll indicator.
            txt(
                r,
                lookups,
                pn[0] + PW - 50.0,
                pn[1] + 2.0,
                0.6,
                GR,
                &format!("{}/{}", scroll_row + 1, total),
            );
        }
        Tab::Npcs => {
            for vi in 0..visible {
                let idx = scroll_row + vi;
                if idx >= NPC_LIST.len() {
                    break;
                }
                let (kind, name, tex) = NPC_LIST[idx];
                let y = list_top + vi as f32 * ROW_H;
                icon(r, tex, ct[0], y, ICON);
                let count = npcs.iter().filter(|n| n.kind == kind && n.hp > 0).count();
                let c = if count > 0 { Y } else { W };
                txt(
                    r,
                    lookups,
                    ct[0] + ICON + 4.0,
                    y + 2.0,
                    0.7,
                    c,
                    &format!("{} [{}]", name, count),
                );
            }
        }
        Tab::Bosses => {
            for vi in 0..visible {
                let idx = scroll_row + vi;
                if idx >= BOSS_LIST.len() {
                    break;
                }
                let (kind, name, tex) = BOSS_LIST[idx];
                let y = list_top + vi as f32 * ROW_H;
                icon(r, tex, ct[0], y, ICON);
                let alive = npcs.iter().any(|n| n.kind == kind && n.hp > 0);
                let c = if alive { R } else { G };
                txt(
                    r,
                    lookups,
                    ct[0] + ICON + 4.0,
                    y + 2.0,
                    0.7,
                    c,
                    &format!("{}{}", name, if alive { " [LIVE]" } else { "" }),
                );
            }
        }
        Tab::World => {
            let rows: [(&str, [f32; 4]); 9] = [
                (
                    &format!("Hardmode: {}", if world.hard_mode { "ON" } else { "OFF" }),
                    if world.hard_mode { R } else { G },
                ),
                (
                    &format!("Time: {}", if world.day_time { "Day" } else { "Night" }),
                    W,
                ),
                (
                    &format!(
                        "Blood Moon: {}",
                        if world.blood_moon { "ON" } else { "OFF" }
                    ),
                    if world.blood_moon { R } else { G },
                ),
                ("Set Day", W),
                ("Set Night", W),
                ("Set Noon", W),
                ("Set Midnight", W),
                (
                    &format!(
                        "Goblin Army: {}",
                        if world.invasion_goblins {
                            format!("{} left (click to end)", world.invasion_size)
                        } else if world.downed_goblins {
                            "defeated".to_string()
                        } else {
                            "OFF (click to start)".to_string()
                        }
                    ),
                    if world.invasion_goblins { R } else { Y },
                ),
                ("Kill All NPCs", R),
            ];
            for (i, (label, c)) in rows.iter().enumerate() {
                txt(
                    r,
                    lookups,
                    ct[0],
                    list_top + i as f32 * ROW_H,
                    0.75,
                    *c,
                    label,
                );
            }
        }
    }
}

// ---- The click ----

pub fn handle_click(app: &mut App, mouse: Vec2) -> bool {
    if !app.debug_panel.open {
        return false;
    }
    let screen = Vec2::new(1280.0, 720.0);
    let pn = panel_rect(app.debug_panel.pos);
    if !pt_in(mouse, pn) {
        return false;
    }

    // The title-bar drag: the top TITLE_H pixels only.
    if mouse.y < pn[1] + TITLE_H && mouse.y >= pn[1] {
        app.debug_panel.drag_offset = Some(Vec2::new(mouse.x - pn[0], mouse.y - pn[1]));
        return true;
    }

    // The tabs — below the title bar.
    let tab_y = pn[1] + TITLE_H;
    let tw = PW / TABS.len() as f32;
    for (i, (_, tab)) in TABS.iter().enumerate() {
        let tr = [pn[0] + i as f32 * tw, tab_y, tw, TAB_H];
        if pt_in(mouse, tr) {
            app.debug_panel.tab = *tab;
            app.debug_panel.scroll = 0.0;
            return true;
        }
    }

    let ct = content_rect(&pn);
    if mouse.y < ct[1] + FILTER_H {
        return true;
    }
    let list_top = ct[1] + FILTER_H;
    let row = ((mouse.y - list_top) / ROW_H) as usize;
    let idx = app.debug_panel.scroll as usize + row;

    match app.debug_panel.tab {
        Tab::Items => {
            let items = filtered_items(&app.debug_panel.filter);
            if let Some((id, _)) = items.get(idx) {
                give_item(app, *id, 1);
            }
        }
        Tab::Npcs => {
            if let Some((kind, _, _)) = NPC_LIST.get(idx) {
                spawn_npc(app, *kind);
            }
        }
        Tab::Bosses => {
            if let Some((kind, _, _)) = BOSS_LIST.get(idx) {
                spawn_boss(app, *kind);
            }
        }
        Tab::World => world_click(app, row),
    }
    true
}

fn give_item(app: &mut App, item: u16, count: u32) -> bool {
    let idx = match crate::item::slot_for_pickup(&mut app.inventory, item, 0) {
        Some(i) => i,
        None => return false,
    };
    let sl = &mut app.inventory[idx];
    if sl.kind == item && sl.count > 0 {
        sl.count += count;
    } else {
        *sl = ItemSlot {
            kind: item,
            name: crate::itemdata::ITEMS[item as usize].name,
            count,
            prefix: 0,
        };
    }
    true
}

fn spawn_npc(app: &mut App, kind: NpcKind) {
    let x = app
        .player
        .as_ref()
        .map(|p| ((p.x / TILE_PX) as i32 + 8).clamp(2, app.world.width - 3))
        .unwrap_or(60);
    let y = app
        .player
        .as_ref()
        .map(|p| ((p.y / TILE_PX) as i32 - 3).clamp(2, app.world.height - 3))
        .unwrap_or(60);
    let npc = match kind {
        NpcKind::Slime => Npc::spawn_slime(&app.world, x),
        NpcKind::Zombie => Npc::spawn_zombie(&app.world, x),
        NpcKind::DemonEye => Npc::spawn_eye(&app.world, x),
        NpcKind::Skeleton | NpcKind::UndeadMiner => Npc::spawn_cave_walker(&app.world, x, kind),
        NpcKind::CaveBat => Npc::spawn_cave_flyer(&app.world, x, y, kind),
        NpcKind::DevourerHead | NpcKind::GiantWormHead => {
            let mut h = Npc::spawn_worm_head(&app.world, x, y, kind);
            h.hp = h.max_hp;
            Some(h)
        }
        NpcKind::EaterOfSouls | NpcKind::Hornet | NpcKind::MeteorHead => {
            Npc::spawn_hover(&app.world, x, y, kind)
        }
        NpcKind::GoblinSorcerer | NpcKind::Tim | NpcKind::DarkCaster | NpcKind::FireImp => {
            Npc::spawn_caster(&app.world, x, y, kind)
        }
        NpcKind::Piranha | NpcKind::Goldfish | NpcKind::Shark => {
            Npc::spawn_swimmer(&app.world, x, y, kind)
        }
        NpcKind::GoblinTinkerer => Npc::spawn_town(&app.world, x, kind),
        NpcKind::BoundGoblin => Npc::spawn_caster(&app.world, x, y, kind),
        _ => None,
    };
    if let Some(n) = npc {
        app.npcs.push(n);
    }
}

fn spawn_boss(app: &mut App, kind: NpcKind) {
    let x = app
        .player
        .as_ref()
        .map(|p| ((p.x / TILE_PX) as i32 + 12).clamp(2, app.world.width - 3))
        .unwrap_or(60);
    let y = app
        .player
        .as_ref()
        .map(|p| ((p.y / TILE_PX) as i32 - 12).clamp(2, app.world.height - 3))
        .unwrap_or(60);
    let x_px = x as f32 * TILE_PX;
    let y_px = y as f32 * TILE_PX;
    let boss = match kind {
        NpcKind::KingSlime => Npc::spawn_king_slime(&app.world, x),
        NpcKind::EyeOfCthulhu
        | NpcKind::BrainOfCthulhu
        | NpcKind::QueenBee
        | NpcKind::SkeletronHead
        | NpcKind::WallOfFlesh
        | NpcKind::Retinazer
        | NpcKind::Spazmatism
        | NpcKind::SkeletronPrime
        | NpcKind::Plantera => {
            let mut b = Npc::new(kind, x_px, y_px, 1, 0.0);
            b.hp = b.max_hp;
            Some(b)
        }
        NpcKind::EaterOfWorldsHead => {
            let mut h = Npc::spawn_worm_head(&app.world, x, y, kind);
            h.hp = h.max_hp;
            h.worm_left = 65;
            Some(h)
        }
        NpcKind::Golem => {
            // The 4-piece set (`AI_045`'s init shape): the body at the
            // click, the head locked above, the fists flanking.
            let mut body = Npc::new(NpcKind::Golem, x_px, y_px, 1, 0.0);
            body.hp = body.max_hp;
            let mut head = Npc::new(
                NpcKind::GolemHead,
                x_px + (140.0 - 70.0) * 0.5,
                y_px - 70.0,
                1,
                0.0,
            );
            head.hp = head.max_hp;
            let mut fl = Npc::new(NpcKind::GolemFistLeft, x_px - 80.0, y_px + 40.0, 1, 0.0);
            fl.hp = fl.max_hp;
            let mut fr = Npc::new(NpcKind::GolemFistRight, x_px + 180.0, y_px + 40.0, 1, 0.0);
            fr.hp = fr.max_hp;
            app.npcs.push(head);
            app.npcs.push(fl);
            app.npcs.push(fr);
            Some(body)
        }
        NpcKind::TheDestroyer => {
            let mut h = Npc::spawn_worm_head(&app.world, x, y, kind);
            h.hp = h.max_hp;
            h.worm_left = 80;
            Some(h)
        }
        _ => None,
    };
    if let Some(b) = boss {
        app.npcs.push(b);
    }
}

fn world_click(app: &mut App, row: usize) {
    match row {
        0 => {
            app.world.hard_mode = !app.world.hard_mode;
        }
        1 => {
            app.world.day_time = !app.world.day_time;
            app.world.time = 0.0;
        }
        2 => {
            app.world.blood_moon = !app.world.blood_moon;
        }
        3 => {
            app.world.day_time = true;
            app.world.time = 0.0;
        }
        4 => {
            app.world.day_time = false;
            app.world.time = 0.0;
        }
        5 => {
            app.world.day_time = true;
            app.world.time = 27000.0;
        }
        6 => {
            app.world.day_time = false;
            app.world.time = 16200.0;
        }
        7 => {
            // The invasion toggle: start (80 invaders = the vanilla's
            // `invasionSize` for the army) or call it off.
            if app.world.invasion_goblins {
                app.world.invasion_goblins = false;
                app.world.invasion_size = 0;
            } else {
                app.world.invasion_goblins = true;
                app.world.invasion_size = 80;
            }
        }
        8 => {
            // The Slime Rain toggle: the vanilla roll/kill machinery
            // runs off the flag; the start uses the vanilla duration
            // band's midpoint.
            if app.world.slime_rain {
                app.world.slime_rain = false;
                app.world.slime_rain_time = 0.0;
                app.world.slime_rain_kills = 0;
            } else {
                app.world.slime_rain = true;
                app.world.slime_rain_time = 43200.0;
                app.world.slime_rain_kills = 0;
            }
        }
        9 => {
            for n in app.npcs.iter_mut() {
                if n.hp > 0 && !n.kind.is_worm_head() {
                    n.hp = 0;
                }
            }
        }
        _ => {}
    }
}

// ---- The keys ----

pub fn handle_key(app: &mut App, code: winit::keyboard::KeyCode) -> bool {
    use winit::keyboard::KeyCode;
    if code == KeyCode::F3 {
        app.debug_panel.open = !app.debug_panel.open;
        return true;
    }
    if !app.debug_panel.open {
        return false;
    }
    // The filter field: the letter/digit keys filter the list.
    if let Some(c) = key_char(code) {
        app.debug_panel.filter.push(c);
        app.debug_panel.scroll = 0.0;
        return true;
    }
    if code == KeyCode::Backspace {
        app.debug_panel.filter.pop();
        app.debug_panel.scroll = 0.0;
        return true;
    }
    false
}

fn key_char(code: winit::keyboard::KeyCode) -> Option<char> {
    use winit::keyboard::KeyCode;
    match code {
        KeyCode::KeyA => Some('a'),
        KeyCode::KeyB => Some('b'),
        KeyCode::KeyC => Some('c'),
        KeyCode::KeyD => Some('d'),
        KeyCode::KeyE => Some('e'),
        KeyCode::KeyF => Some('f'),
        KeyCode::KeyG => Some('g'),
        KeyCode::KeyH => Some('h'),
        KeyCode::KeyI => Some('i'),
        KeyCode::KeyJ => Some('j'),
        KeyCode::KeyK => Some('k'),
        KeyCode::KeyL => Some('l'),
        KeyCode::KeyM => Some('m'),
        KeyCode::KeyN => Some('n'),
        KeyCode::KeyO => Some('o'),
        KeyCode::KeyP => Some('p'),
        KeyCode::KeyQ => Some('q'),
        KeyCode::KeyR => Some('r'),
        KeyCode::KeyS => Some('s'),
        KeyCode::KeyT => Some('t'),
        KeyCode::KeyU => Some('u'),
        KeyCode::KeyV => Some('v'),
        KeyCode::KeyW => Some('w'),
        KeyCode::KeyX => Some('x'),
        KeyCode::KeyY => Some('y'),
        KeyCode::KeyZ => Some('z'),
        KeyCode::Digit0 => Some('0'),
        KeyCode::Digit1 => Some('1'),
        KeyCode::Digit2 => Some('2'),
        KeyCode::Digit3 => Some('3'),
        KeyCode::Digit4 => Some('4'),
        KeyCode::Digit5 => Some('5'),
        KeyCode::Digit6 => Some('6'),
        KeyCode::Digit7 => Some('7'),
        KeyCode::Digit8 => Some('8'),
        KeyCode::Digit9 => Some('9'),
        KeyCode::Space => Some(' '),
        _ => None,
    }
}

pub fn handle_scroll(app: &mut App, delta: f32) -> bool {
    if !app.debug_panel.open {
        return false;
    }
    app.debug_panel.scroll = (app.debug_panel.scroll - delta * 3.0).max(0.0);
    true
}

/// The drag move: updates the panel's position while dragging.
pub fn handle_drag(app: &mut App, mouse: Vec2) -> bool {
    if !app.debug_panel.open {
        return false;
    }
    if let Some(offset) = app.debug_panel.drag_offset {
        app.debug_panel.pos = Vec2::new(mouse.x - offset.x, mouse.y - offset.y);
        return true;
    }
    false
}

/// The drag end (the mouse release).
pub fn end_drag(app: &mut App) -> bool {
    if app.debug_panel.drag_offset.take().is_some() {
        return true;
    }
    false
}
