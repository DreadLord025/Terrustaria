//! The debug cheat-sheet panel — the self-contained debug UI. Only this
//! module calls the App/Npc/World methods; nothing references it back
//! (delete this file + the `mod` line + the call sites to remove).
//!
//! Open/close: F3. The tabs: Player / Items / NPCs / Bosses / World.

use crate::app::App;
use crate::item::ItemSlot;
use crate::npc::{Npc, NpcKind};
use crate::assets::Lookups;
use glam::Vec2;
use tr_core::TILE_PX;
use tr_render::SpriteRenderer;

// ---- The state ----

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tab { Player, Items, Npcs, Bosses, World }

const TABS: [(&str, Tab); 5] = [
    ("Player", Tab::Player), ("Items", Tab::Items), ("NPCs", Tab::Npcs),
    ("Bosses", Tab::Bosses), ("World", Tab::World),
];

pub struct DebugPanel {
    pub open: bool,
    pub tab: Tab,
    pub item_id: String,
}

impl DebugPanel {
    pub fn new() -> Self {
        Self { open: false, tab: Tab::Player, item_id: String::new() }
    }
}
impl Default for DebugPanel {
    fn default() -> Self { Self::new() }
}

// ---- The layout ----
const PW: f32 = 260.0;
const PH: f32 = 380.0;
const TAB_H: f32 = 20.0;
const ROW_H: f32 = 20.0;
const PAD: f32 = 8.0;

fn panel_rect(screen: Vec2) -> [f32; 4] {
    [screen.x - PW - 4.0, 4.0, PW, PH.min(screen.y - 8.0)]
}

fn content_rect(panel: &[f32; 4]) -> [f32; 4] {
    [panel[0] + PAD, panel[1] + TAB_H + PAD, panel[2] - PAD * 2.0, panel[3] - TAB_H - PAD * 2.0]
}

fn pt_in(p: Vec2, r: [f32; 4]) -> bool {
    p.x >= r[0] && p.x <= r[0] + r[2] && p.y >= r[1] && p.y <= r[1] + r[3]
}

fn row_y(content: &[f32; 4], row: usize) -> f32 {
    content[1] + row as f32 * ROW_H
}

// ---- The draw ----

const W: [f32; 4] = [1.0; 4];
const G: [f32; 4] = [0.4, 1.0, 0.4, 1.0];
const R: [f32; 4] = [1.0, 0.4, 0.4, 1.0];
const Y: [f32; 4] = [1.0, 1.0, 0.4, 1.0];
const C: [f32; 4] = [0.4, 1.0, 1.0, 1.0];
const GR: [f32; 4] = [0.6; 4];

fn txt(r: &mut SpriteRenderer, l: &Lookups, x: f32, y: f32, s: f32, c: [f32; 4], t: &str) {
    crate::ui::text::hud_text(r, l, x, y, s, c, t);
}

/// The panel draw (called from the frame pass where the borrows allow it).
pub fn draw(
    r: &mut SpriteRenderer,
    lookups: &Lookups,
    screen: Vec2,
    panel: &DebugPanel,
    player: Option<&crate::player::Player>,
    npcs: &[Npc],
    world: &tr_core::World,
    coins: u64,
    mana: i32,
) {
    if !panel.open { return; }
    let pn = panel_rect(screen);
    let ct = content_rect(&pn);
    txt(r, lookups, pn[0] + PAD, pn[1] + 2.0, 0.9, C, "DEBUG [F3]");

    // The tabs.
    let tw = PW / TABS.len() as f32;
    for (i, (label, tab)) in TABS.iter().enumerate() {
        let c = if panel.tab == *tab { G } else { GR };
        txt(r, lookups, pn[0] + i as f32 * tw + 4.0, pn[1] + TAB_H - 14.0, 0.75, c, label);
    }

    match panel.tab {
        Tab::Player => {
            if let Some(p) = player {
                txt(r, lookups, ct[0], row_y(&ct, 0), 0.8, W, &format!("HP {}/{}", p.hp, p.max_hp));
                txt(r, lookups, ct[0] + 80.0, row_y(&ct, 0), 0.8, C, &format!("MP {}", mana));
                txt(r, lookups, ct[0], row_y(&ct, 1), 0.8, Y, &format!("Coins {}", coins));
                for (i, label) in ["Respawn", "Heal Full", "Mana Full", "+100 HP", "Godmode", "+100 Coins"].iter().enumerate() {
                    txt(r, lookups, ct[0], row_y(&ct, i + 2), 0.8, G, label);
                }
            } else {
                txt(r, lookups, ct[0], row_y(&ct, 0), 0.8, R, "no player");
            }
        }
        Tab::Items => {
            txt(r, lookups, ct[0], row_y(&ct, 0), 0.8, C, &format!("ID: {}", panel.item_id));
            txt(r, lookups, ct[0], row_y(&ct, 1), 0.7, GR, "Enter = spawn, Backspace = del");
            const QI: [(u16, &str); 8] = [
                (8, "Torch"), (110, "Iron Pick"), (122, "Iron Sword"),
                (188, "Heal Potion"), (97, "Musket Ball"), (168, "Grenade"),
                (75, "Copper Coin"), (5309, "Herb Bag"),
            ];
            for (i, (id, label)) in QI.iter().enumerate() {
                txt(r, lookups, ct[0], row_y(&ct, i + 2), 0.75, G, &format!("{} ({})", label, id));
            }
        }
        Tab::Npcs => {
            const SN: [(NpcKind, &str); 14] = [
                (NpcKind::Slime, "Blue Slime"), (NpcKind::Zombie, "Zombie"),
                (NpcKind::DemonEye, "Demon Eye"), (NpcKind::Skeleton, "Skeleton"),
                (NpcKind::CaveBat, "Cave Bat"), (NpcKind::UndeadMiner, "Und. Miner"),
                (NpcKind::DevourerHead, "Devourer"), (NpcKind::GiantWormHead, "Giant Worm"),
                (NpcKind::EaterOfSouls, "Eater of Souls"), (NpcKind::Hornet, "Hornet"),
                (NpcKind::MeteorHead, "Meteor Head"), (NpcKind::GoblinSorcerer, "Goblin Sorc."),
                (NpcKind::Tim, "Tim"), (NpcKind::FireImp, "Fire Imp"),
            ];
            for (i, (kind, label)) in SN.iter().enumerate() {
                let count = npcs.iter().filter(|n| n.kind == *kind && n.hp > 0).count();
                let c = if count > 0 { Y } else { GR };
                txt(r, lookups, ct[0], row_y(&ct, i), 0.75, c, &format!("{} [{}]", label, count));
            }
        }
        Tab::Bosses => {
            const BL: [(NpcKind, &str); 12] = [
                (NpcKind::KingSlime, "King Slime"), (NpcKind::EyeOfCthulhu, "Eye of Cthulhu"),
                (NpcKind::BrainOfCthulhu, "Brain of Cthulhu"), (NpcKind::EaterOfWorldsHead, "Eater of Worlds"),
                (NpcKind::QueenBee, "Queen Bee"), (NpcKind::SkeletronHead, "Skeletron"),
                (NpcKind::WallOfFlesh, "Wall of Flesh"), (NpcKind::TheDestroyer, "The Destroyer"),
                (NpcKind::Retinazer, "Retinazer"), (NpcKind::Spazmatism, "Spazmatism"),
                (NpcKind::SkeletronPrime, "Skeletron Prime"), (NpcKind::Plantera, "Plantera"),
            ];
            for (i, (kind, label)) in BL.iter().enumerate() {
                let alive = npcs.iter().any(|n| n.kind == *kind && n.hp > 0);
                let c = if alive { R } else { G };
                txt(r, lookups, ct[0], row_y(&ct, i), 0.75, c,
                    &format!("{}{}", label, if alive { " [LIVE]" } else { "" }));
            }
        }
        Tab::World => {
            let rows: [(&str, [f32; 4]); 8] = [
                (&format!("Hardmode: {}", if world.hard_mode { "ON" } else { "OFF" }),
                    if world.hard_mode { R } else { G }),
                (&format!("Time: {}", if world.day_time { "Day" } else { "Night" }), W),
                (&format!("Blood Moon: {}", if world.blood_moon { "ON" } else { "OFF" }),
                    if world.blood_moon { R } else { G }),
                ("Set Day", W), ("Set Night", W), ("Set Noon", W), ("Set Midnight", W),
                ("Kill All NPCs", R),
            ];
            for (i, (label, c)) in rows.iter().enumerate() {
                txt(r, lookups, ct[0], row_y(&ct, i), 0.75, *c, label);
            }
        }
    }
}

// ---- The click ----

pub fn handle_click(app: &mut App, mouse: Vec2) -> bool {
    if !app.debug_panel.open { return false; }
    let screen = Vec2::new(1280.0, 720.0);
    let pn = panel_rect(screen);
    if !pt_in(mouse, pn) { return false; }

    // The tabs.
    let tw = PW / TABS.len() as f32;
    for (i, (_, tab)) in TABS.iter().enumerate() {
        let tr = [pn[0] + i as f32 * tw, pn[1], tw, TAB_H];
        if pt_in(mouse, tr) { app.debug_panel.tab = *tab; return true; }
    }

    let ct = content_rect(&pn);
    if mouse.y < ct[1] { return true; }
    let row = ((mouse.y - ct[1]) / ROW_H) as usize;

    match app.debug_panel.tab {
        Tab::Player => player_click(app, row),
        Tab::Items => items_click(app, row),
        Tab::Npcs => npc_click(app, row),
        Tab::Bosses => boss_click(app, row),
        Tab::World => world_click(app, row),
    }
    true
}

fn give_item(app: &mut App, item: u16, count: u32) -> bool {
    let idx = match crate::item::slot_for_pickup(&mut app.inventory, item) {
        Some(i) => i, None => return false,
    };
    let sl = &mut app.inventory[idx];
    if sl.kind == item && sl.count > 0 { sl.count += count; } else {
        *sl = ItemSlot { kind: item, name: crate::itemdata::ITEMS[item as usize].name, count };
    }
    true
}

fn spawn_x(app: &App, dx: i32) -> i32 {
    app.player.as_ref()
        .map(|p| ((p.x / TILE_PX) as i32 + dx).clamp(2, app.world.width - 3))
        .unwrap_or(60)
}

fn spawn_y(app: &App, dy: i32) -> i32 {
    app.player.as_ref()
        .map(|p| ((p.y / TILE_PX) as i32 + dy).clamp(2, app.world.height - 3))
        .unwrap_or(60)
}

fn simple_boss(kind: NpcKind, x: f32, y: f32) -> Option<Npc> {
    let mut b = Npc::new(kind, x, y, 1, 0.0);
    b.hp = b.max_hp;
    Some(b)
}

fn worm_boss(world: &tr_core::World, x: i32, y: i32, kind: NpcKind, segments: u32) -> Option<Npc> {
    let mut head = Npc::spawn_worm_head(world, x, y, kind);
    head.hp = head.max_hp;
    head.worm_left = segments;
    Some(head)
}

fn player_click(app: &mut App, row: usize) {
    match row {
        2 => { if let Some(p) = app.player.as_mut() { p.respawn(); } }
        3 => { if let Some(p) = app.player.as_mut() { p.hp = p.max_hp; } }
        4 => { if let Some(p) = app.player.as_mut() { p.mana = p.mana_max; } }
        5 => { if let Some(p) = app.player.as_mut() { p.hp = (p.hp + 100).min(p.max_hp); } }
        6 => { if let Some(p) = app.player.as_mut() { p.hp = p.max_hp; } }
        7 => { app.coins += 10000; }
        _ => {}
    }
}

fn items_click(app: &mut App, row: usize) {
    if row < 2 { return; }
    const QI: [u16; 8] = [8, 110, 122, 188, 97, 168, 75, 5309];
    if let Some(id) = QI.get(row - 2) { give_item(app, *id, 1); }
}

fn npc_click(app: &mut App, row: usize) {
    const SN: [(NpcKind, i32); 14] = [
        (NpcKind::Slime, 8), (NpcKind::Zombie, 8), (NpcKind::DemonEye, 8),
        (NpcKind::Skeleton, 8), (NpcKind::CaveBat, 8), (NpcKind::UndeadMiner, 8),
        (NpcKind::DevourerHead, 8), (NpcKind::GiantWormHead, 8),
        (NpcKind::EaterOfSouls, 8), (NpcKind::Hornet, 8), (NpcKind::MeteorHead, 8),
        (NpcKind::GoblinSorcerer, 8), (NpcKind::Tim, 8), (NpcKind::FireImp, 8),
    ];
    if let Some((kind, dx)) = SN.get(row) {
        let x = spawn_x(app, *dx);
        let y = spawn_y(app, -3);
        let npc = match kind {
            NpcKind::Slime => Npc::spawn_slime(&app.world, x),
            NpcKind::Zombie => Npc::spawn_zombie(&app.world, x),
            NpcKind::DemonEye => Npc::spawn_eye(&app.world, x),
            NpcKind::Skeleton | NpcKind::UndeadMiner => Npc::spawn_cave_walker(&app.world, x, *kind),
            NpcKind::CaveBat => Npc::spawn_cave_flyer(&app.world, x, y, *kind),
            NpcKind::DevourerHead | NpcKind::GiantWormHead => {
                let mut h = Npc::spawn_worm_head(&app.world, x, y, *kind);
                h.hp = h.max_hp;
                Some(h)
            }
            NpcKind::EaterOfSouls | NpcKind::Hornet | NpcKind::MeteorHead => {
                Npc::spawn_hover(&app.world, x, y, *kind)
            }
            NpcKind::GoblinSorcerer | NpcKind::Tim | NpcKind::DarkCaster | NpcKind::FireImp => {
                Npc::spawn_caster(&app.world, x, y, *kind)
            }
            _ => None,
        };
        if let Some(n) = npc { app.npcs.push(n); }
    }
}

fn boss_click(app: &mut App, row: usize) {
    let x = spawn_x(app, 12);
    let y = spawn_y(app, -12);
    let x_px = x as f32 * TILE_PX;
    let y_px = y as f32 * TILE_PX;
    let boss = match row {
        0 => Npc::spawn_king_slime(&app.world, x),
        1 => simple_boss(NpcKind::EyeOfCthulhu, x_px, y_px),
        2 => simple_boss(NpcKind::BrainOfCthulhu, x_px, y_px),
        3 => worm_boss(&app.world, x, y, NpcKind::EaterOfWorldsHead, 65),
        4 => simple_boss(NpcKind::QueenBee, x_px, y_px),
        5 => simple_boss(NpcKind::SkeletronHead, x_px, y_px),
        6 => simple_boss(NpcKind::WallOfFlesh, x_px, y_px),
        7 => worm_boss(&app.world, x, y, NpcKind::TheDestroyer, 80),
        8 => simple_boss(NpcKind::Retinazer, x_px - 200.0, y_px),
        9 => simple_boss(NpcKind::Spazmatism, x_px + 200.0, y_px),
        10 => simple_boss(NpcKind::SkeletronPrime, x_px, y_px),
        11 => simple_boss(NpcKind::Plantera, x_px, y_px),
        _ => None,
    };
    if let Some(b) = boss { app.npcs.push(b); }
}

fn world_click(app: &mut App, row: usize) {
    match row {
        0 => { app.world.hard_mode = !app.world.hard_mode; }
        1 => { app.world.day_time = !app.world.day_time; app.world.time = 0.0; }
        2 => { app.world.blood_moon = !app.world.blood_moon; }
        3 => { app.world.day_time = true; app.world.time = 0.0; }
        4 => { app.world.day_time = false; app.world.time = 0.0; }
        5 => { app.world.day_time = true; app.world.time = 27000.0; }
        6 => { app.world.day_time = false; app.world.time = 16200.0; }
        7 => { for n in app.npcs.iter_mut() { if n.hp > 0 && !n.kind.is_worm_head() { n.hp = 0; } } }
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
    if !app.debug_panel.open { return false; }
    if app.debug_panel.tab == Tab::Items {
        let digit = match code {
            KeyCode::Digit0 => Some('0'), KeyCode::Digit1 => Some('1'),
            KeyCode::Digit2 => Some('2'), KeyCode::Digit3 => Some('3'),
            KeyCode::Digit4 => Some('4'), KeyCode::Digit5 => Some('5'),
            KeyCode::Digit6 => Some('6'), KeyCode::Digit7 => Some('7'),
            KeyCode::Digit8 => Some('8'), KeyCode::Digit9 => Some('9'),
            KeyCode::Backspace => { app.debug_panel.item_id.pop(); return true; }
            KeyCode::Enter => {
                if let Ok(id) = app.debug_panel.item_id.parse::<u16>() {
                    give_item(app, id, 1);
                }
                return true;
            }
            _ => None,
        };
        if let Some(d) = digit {
            if app.debug_panel.item_id.len() < 5 { app.debug_panel.item_id.push(d); }
            return true;
        }
    }
    false
}
