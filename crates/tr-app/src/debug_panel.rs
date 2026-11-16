//! The F3 developer workbench — the `tr-app` adapter.
//!
//! The panel's state, layout, hit-testing and input→intent translation live in
//! the separate `tr-debug` crate ([`tr_debug`]); this file supplies the three
//! host-side pieces it cannot know:
//!
//! 1. the **rows** (item/NPC/boss/recipe/world tables),
//! 2. the **drawing** (`SpriteRenderer` + `Lookups`),
//! 3. the **application** of an [`Event`] to the game state.
//!
//! Compiled only with the `debug` feature: `cargo build --release` without it
//! contains no panel, no this module and no its call sites.
//!
//! Open/close: F3. Tabs: Items / NPCs / Bosses / World / Recipes.

use crate::app::App;
use crate::assets::Lookups;
use crate::item::ItemSlot;
use crate::npc::{Npc, NpcKind};
use glam::Vec2;
use tr_core::TILE_PX;
use tr_debug::{Event, Layout, Panel, Tab, ICON, PAD, PH, PW, ROW_H};
use tr_render::SpriteRenderer;

pub use tr_debug::{Panel as DebugPanel, Tab as PanelTab};

// ---- The data lists ----

/// All the item names/ids from the registry.
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

fn matches(filter: &str, name: &str) -> bool {
    filter.is_empty() || name.to_lowercase().contains(&filter.to_lowercase())
}

const NPC_LIST: [(NpcKind, &str, &str); 35] = [
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
    (NpcKind::Deerclops, "Deerclops", "NPC_668"),
    (NpcKind::GoblinTinkerer, "Gob. Tinkerer", "NPC_107"),
    (NpcKind::BoundGoblin, "Bound Goblin", "NPC_105"),
    (NpcKind::TravellingMerchant, "Travel Merchant", "NPC_368"),
    (NpcKind::Angler, "Angler", "NPC_369"),
    (NpcKind::SleepingAngler, "Sleeping Angler", "NPC_376"),
    (NpcKind::Wizard, "Wizard", "NPC_108"),
    (NpcKind::BoundWizard, "Bound Wizard", "NPC_106"),
    (NpcKind::Mechanic, "Mechanic", "NPC_124"),
    (NpcKind::BoundMechanic, "Bound Mechanic", "NPC_123"),
    (NpcKind::Stylist, "Stylist", "NPC_353"),
    (NpcKind::WebbedStylist, "Webbed Stylist", "NPC_354"),
    (NpcKind::TaxCollector, "Tax Collector", "NPC_441"),
    (NpcKind::Golfer, "Golfer", "NPC_588"),
    (NpcKind::SleepingGolfer, "Sleeping Golfer", "NPC_589"),
    (NpcKind::Zoologist, "Zoologist", "NPC_633"),
    (NpcKind::Princess, "Princess", "NPC_663"),
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

/// The row count of a tab, after the filter. `World` is a fixed menu; `Recipes`
/// counts the generated recipe table.
pub(crate) fn row_count(panel: &Panel, tab: Tab) -> usize {
    match tab {
        Tab::Items => filtered_items(&panel.filter).len(),
        Tab::Npcs => NPC_LIST
            .iter()
            .filter(|(_, name, _)| matches(&panel.filter, name))
            .count(),
        Tab::Bosses => BOSS_LIST
            .iter()
            .filter(|(_, name, _)| matches(&panel.filter, name))
            .count(),
        Tab::World => 11,
        Tab::Recipes => crate::craft::RECIPES.len(),
    }
}

// ---- The colours ----

const W: [f32; 4] = [1.0; 4];
const G: [f32; 4] = [0.4, 1.0, 0.4, 1.0];
const R: [f32; 4] = [1.0, 0.4, 0.4, 1.0];
const Y: [f32; 4] = [1.0, 1.0, 0.4, 1.0];
const C: [f32; 4] = [0.4, 1.0, 1.0, 1.0];
const GR: [f32; 4] = [0.6; 4];
const BG: [f32; 4] = [0.05, 0.05, 0.08, 0.92];
const SEL: [f32; 4] = [0.10, 0.16, 0.24, 0.95];

fn txt(r: &mut SpriteRenderer, l: &Lookups, x: f32, y: f32, s: f32, c: [f32; 4], t: &str) {
    crate::ui::text::hud_text(r, l, x, y, s, c, t);
}

/// A flat filled rect (the panel background / the selected row).
fn fill(r: &mut SpriteRenderer, rect: [f32; 4], c: [f32; 4]) {
    if let Some(uv) = r.uv("Item_1", 0.0, 0.0, 100.0, 100.0) {
        r.push_sprite([rect[0], rect[1]], [rect[2], rect[3]], uv, c);
    }
}

/// Draws the item/NPC icon from the atlas (the 18x18 thumbnail).
fn icon(r: &mut SpriteRenderer, sheet_name: &str, x: f32, y: f32, size: f32) {
    if let Some(uv) = r.uv(sheet_name, 0.0, 0.0, 100.0, 100.0) {
        r.push_sprite([x, y], [size, size], uv, [1.0; 4]);
    }
}

// ---- The draw ----

pub fn draw(
    r: &mut SpriteRenderer,
    lookups: &Lookups,
    screen: Vec2,
    panel: &Panel,
    npcs: &[Npc],
    world: &tr_core::World,
    stepper_paused: bool,
) {
    if !panel.open {
        return;
    }
    let l = panel.layout(screen);
    fill(r, l.panel, BG);
    fill(r, l.title, SEL);

    // The title bar (the drag handle).
    txt(r, lookups, l.title[0] + PAD, l.title[1] + 2.0, 0.9, C, "DEBUG [F3]");
    txt(
        r,
        lookups,
        l.title[0] + PW - 150.0,
        l.title[1] + 3.0,
        0.6,
        GR,
        &format!("rows {}/{}", panel.scroll_row() + 1, row_count(panel, panel.tab)),
    );

    // The tab rail.
    for (i, (label, tab)) in tr_debug::TABS.iter().enumerate() {
        let tr = l.tab(i);
        if panel.tab == *tab {
            fill(r, tr, SEL);
        }
        let c = if panel.tab == *tab { G } else { GR };
        txt(r, lookups, tr[0] + 4.0, tr[1] + 4.0, 0.75, c, label);
    }

    // The filter field.
    txt(
        r,
        lookups,
        l.filter[0],
        l.filter[1] + 2.0,
        0.75,
        C,
        &format!("Search: {}", panel.filter),
    );

    // The rows.
    let visible = (l.list[3] / ROW_H) as usize;
    let scroll_row = panel.scroll_row();
    match panel.tab {
        Tab::Items => {
            let items = filtered_items(&panel.filter);
            for vi in 0..visible {
                let idx = scroll_row + vi;
                let Some((id, name)) = items.get(idx) else {
                    break;
                };
                row_bg(r, &l, &panel, vi);
                let y = l.row_y(vi);
                icon(r, &format!("Item_{}", id), l.list[0], y, ICON);
                txt(
                    r,
                    lookups,
                    l.list[0] + ICON + 4.0,
                    y + 2.0,
                    0.7,
                    W,
                    &format!("{} [{}]", name, id),
                );
            }
        }
        Tab::Npcs | Tab::Bosses => {
            let (list, boss) = match panel.tab {
                Tab::Bosses => (Some(&BOSS_LIST[..]), true),
                _ => (None, false),
            };
            let entries: Vec<&(NpcKind, &str, &str)> = match list {
                Some(l) => l
                    .iter()
                    .filter(|(_, name, _)| matches(&panel.filter, name))
                    .collect(),
                None => NPC_LIST
                    .iter()
                    .filter(|(_, name, _)| matches(&panel.filter, name))
                    .collect(),
            };
            for vi in 0..visible {
                let idx = scroll_row + vi;
                let Some((kind, name, tex)) = entries.get(idx) else {
                    break;
                };
                row_bg(r, &l, &panel, vi);
                let y = l.row_y(vi);
                icon(r, tex, l.list[0], y, ICON);
                let alive = npcs.iter().any(|n| n.kind == *kind && n.hp > 0);
                let count = npcs.iter().filter(|n| n.kind == *kind && n.hp > 0).count();
                let (c, suffix) = if boss {
                    if alive {
                        (R, " [LIVE]".to_string())
                    } else {
                        (G, String::new())
                    }
                } else if count > 0 {
                    (Y, format!(" [{}]", count))
                } else {
                    (W, String::new())
                };
                txt(
                    r,
                    lookups,
                    l.list[0] + ICON + 4.0,
                    y + 2.0,
                    0.7,
                    c,
                    &format!("{}{}", name, suffix),
                );
            }
        }
        Tab::World => {
            let rows = world_rows(world, stepper_paused);
            for vi in 0..visible {
                let idx = scroll_row + vi;
                let Some((label, c)) = rows.get(idx) else {
                    break;
                };
                row_bg(r, &l, &panel, vi);
                txt(r, lookups, l.list[0], l.row_y(vi) + 2.0, 0.75, *c, label);
            }
        }
        Tab::Recipes => {
            for vi in 0..visible {
                let idx = scroll_row + vi;
                let Some(recipe) = crate::craft::RECIPES.get(idx) else {
                    break;
                };
                let (id, count) = recipe.create;
                row_bg(r, &l, &panel, vi);
                let y = l.row_y(vi);
                icon(r, &format!("Item_{}", id), l.list[0], y, ICON);
                let name = crate::itemdata::ITEMS
                    .get(id as usize)
                    .map(|d| d.name)
                    .unwrap_or("?");
                txt(
                    r,
                    lookups,
                    l.list[0] + ICON + 4.0,
                    y + 2.0,
                    0.7,
                    W,
                    &format!("{} x{}", name, count),
                );
            }
        }
    }

    // The footer: the selected row's details (the inspector strip).
    let hint = match panel.focus {
        Some(i) => format!("selected: row {} — Enter to apply", i + 1),
        None => "wheel: scroll · arrows: focus · Enter: apply · F3: close".to_string(),
    };
    txt(r, lookups, l.footer[0], l.footer[1] + 2.0, 0.7, GR, &hint);
}

/// The keyboard-focused row's highlight.
fn row_bg(r: &mut SpriteRenderer, l: &Layout, panel: &Panel, vi: usize) {
    let idx = panel.scroll_row() + vi;
    if panel.focus == Some(idx) {
        fill(
            r,
            [l.list[0], l.row_y(vi), l.list[2], ROW_H],
            SEL,
        );
    }
}

/// The World tab's 11 rows. The order is the click dispatch's (`world_click`).
fn world_rows(world: &tr_core::World, stepper_paused: bool) -> Vec<(String, [f32; 4])> {
    vec![
        (
            format!("Hardmode: {}", if world.hard_mode { "ON" } else { "OFF" }),
            if world.hard_mode { R } else { G },
        ),
        (
            format!("Time: {}", if world.day_time { "Day" } else { "Night" }),
            W,
        ),
        (
            format!("Blood Moon: {}", if world.blood_moon { "ON" } else { "OFF" }),
            if world.blood_moon { R } else { G },
        ),
        ("Set Day".to_string(), W),
        ("Set Night".to_string(), W),
        ("Set Noon".to_string(), W),
        ("Set Midnight".to_string(), W),
        (
            if world.invasion_goblins {
                format!("Goblin Army: {} left (click to end)", world.invasion_size)
            } else if world.downed_goblins {
                "Goblin Army: defeated".to_string()
            } else {
                "Goblin Army: OFF (click to start)".to_string()
            },
            if world.invasion_goblins { R } else { Y },
        ),
        (
            format!("Slime Rain: {}", if world.slime_rain { "ON" } else { "OFF" }),
            if world.slime_rain { R } else { G },
        ),
        ("Kill All NPCs".to_string(), R),
        (
            format!("Pause World: {}", if stepper_paused { "ON" } else { "OFF" }),
            if stepper_paused { R } else { G },
        ),
    ]
}

// ---- The input ----

/// The left click: the panel hit-tests, then the host applies the intent.
pub fn handle_click(app: &mut App, mouse: Vec2) -> bool {
    if !app.debug_panel.open {
        return false;
    }
    let rows = row_count(&app.debug_panel, app.debug_panel.tab);
    let Some(event) = app.debug_panel.click(mouse, rows) else {
        // A click inside the panel is still consumed (it must not reach the
        // world behind it); only a click outside falls through.
        let l = app.debug_panel.layout(Vec2::ZERO);
        return tr_debug::in_rect(mouse, l.panel);
    };
    apply(app, event);
    true
}

/// Applies an intent. The **semantics are the pre-existing ones**, verbatim —
/// the crate only moved the hit-testing out.
pub(crate) fn apply(app: &mut App, event: Event) {
    match event {
        Event::Toggled(_) | Event::TabSelected(_) | Event::FilterChanged | Event::Scrolled => {}
        Event::RowClicked { tab, row } => match tab {
            Tab::Items => {
                if let Some((id, _)) = filtered_items(&app.debug_panel.filter).get(row) {
                    give_item(app, *id, 1);
                }
            }
            Tab::Npcs => {
                let entries: Vec<(NpcKind, &str, &str)> = NPC_LIST
                    .iter()
                    .filter(|(_, name, _)| matches(&app.debug_panel.filter, name))
                    .copied()
                    .collect();
                if let Some((kind, _, _)) = entries.get(row) {
                    spawn_npc(app, *kind);
                }
            }
            Tab::Bosses => {
                let entries: Vec<(NpcKind, &str, &str)> = BOSS_LIST
                    .iter()
                    .filter(|(_, name, _)| matches(&app.debug_panel.filter, name))
                    .copied()
                    .collect();
                if let Some((kind, _, _)) = entries.get(row) {
                    spawn_boss(app, *kind);
                }
            }
            Tab::World => world_click(app, row),
            Tab::Recipes => {
                if let Some(recipe) = crate::craft::RECIPES.get(row) {
                    let (id, count) = recipe.create;
                    give_item(app, id, count);
                }
            }
        },
    }
}

/// The keys: F3 toggles, the rest is the filter or the list navigation.
pub fn handle_key(app: &mut App, code: winit::keyboard::KeyCode) -> bool {
    use winit::keyboard::KeyCode;
    if code == KeyCode::F3 {
        app.debug_panel.toggle();
        return true;
    }
    if !app.debug_panel.open {
        return false;
    }
    let rows = row_count(&app.debug_panel, app.debug_panel.tab);
    if let Some(c) = key_char(code) {
        app.debug_panel.push_filter(c);
        return true;
    }
    match code {
        KeyCode::Backspace => {
            app.debug_panel.pop_filter();
            true
        }
        KeyCode::ArrowDown => app.debug_panel.move_focus(1, rows),
        KeyCode::ArrowUp => app.debug_panel.move_focus(-1, rows),
        KeyCode::Enter | KeyCode::NumpadEnter => {
            match app.debug_panel.activate(rows) {
                Some(e) => {
                    apply(app, e);
                    true
                }
                None => false,
            }
        }
        _ => false,
    }
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

/// The wheel (the call site is `app/loop.rs`'s `MouseWheel` arm).
pub fn handle_scroll(app: &mut App, delta: f32) -> bool {
    let rows = row_count(&app.debug_panel, app.debug_panel.tab);
    app.debug_panel.scroll_by(delta, rows)
}

/// The drag move: updates the panel's position while dragging.
pub fn handle_drag(app: &mut App, mouse: Vec2) -> bool {
    app.debug_panel.drag(mouse)
}

/// The drag end (the mouse release).
pub fn end_drag(app: &mut App) -> bool {
    app.debug_panel.end_drag()
}

// ---- The actions (the semantics are frozen: they predate this work) ----

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
            // `WorldGen.StartHardmode` (`WorldGen.cs:31701-31720`): the flag
            // then the runner half (`initializeHardMode` `:31570` —
            // `spread::initialize_hard_mode` carries the two V runners and
            // their RNG order). The C# returns early when already hardmode
            // unless forced, so the row keeps its toggle-off as the debug
            // revert (the recorded F3 deviation: the port has no
            // `UndoSpawnedItemProtection`/achievement half).
            if app.world.hard_mode {
                app.world.hard_mode = false;
            } else {
                app.world.hard_mode = true;
                let corruption = (app.world.seed >> 56) & 1 == 1;
                crate::world_sim::growth::run_hardmode_init(
                    &mut app.world,
                    corruption,
                    app.rngs.stream(crate::app::RngKey::UpdateWorldGen),
                );
            }
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
            // The invasion toggle. Both arms ride the C# entry points rather
            // than writing the state by hand (S1-E-37): `StartInvasion`
            // (`Main.cs:64435-64486`) computes `80 + 40 * players` (plus the
            // type-3 `+40+20x` and the type-4 `160+40x`), the march origin
            // and the size-start bookkeeping; calling it off is the size-0
            // "cleared" form `CanStartInvasion`'s `invasionType != 0 &&
            // invasionSize == 0` arm resets (`:64437-64440`).
            if app.world.invasion_goblins {
                // The C# clear (`:64437-64440`): a stale `invasionType != 0`
                // with a zero size resets the type — the type rides along so
                // the next tick cannot mistake the cancel for a defeat.
                app.world.invasion_goblins = false;
                app.world.invasion_size = 0;
                app.world.invasion_type = 0;
            } else {
                // `CanStartInvasion`'s 200-life gate (`:64424-64432`).
                let qualifying = i32::from(
                    app.player.as_ref().is_some_and(|p| p.max_hp >= 200),
                );
                if qualifying > 0 {
                    // `StartInvasion`'s `Main.rand.Next(2)` march-origin pick
                    // (`:64476-64483`) draws from the `UpdateTime` stream.
                    let roll = app.rngs.stream(crate::app::RngKey::UpdateTime).next_max(2);
                    crate::world_sim::npcs::start_invasion(
                        &mut app.world,
                        qualifying,
                        1,
                    );
                    app.world.invasion_x = if roll == 0 {
                        0.0
                    } else {
                        app.world.width as f64
                    };
                }
            }
        }
        8 => {
            // The Slime Rain toggle through the C# entry points (S1-E-37):
            // `StartSlimeRain` (`Main.cs:64742-64785`) draws the duration
            // `Main.rand.Next(32400, 54000)` and resets the kill counter;
            // `StopSlimeRain` (`:64787`) drops the flag and arms the negative
            // `-Next(3024, 6048) * 100` cooldown. Both draw off `UpdateTime`.
            if app.world.slime_rain {
                crate::world_sim::weather::stop_slime_rain(
                    &mut app.world,
                    app.rngs.stream(crate::app::RngKey::UpdateTime),
                );
            } else if !app.rain.is_raining() {
                // `:64761-64764`: the start refuses while it is already
                // raining; `:64767` only re-rolls below zero.
                crate::world_sim::weather::start_slime_rain(
                    &mut app.world,
                    app.rngs.stream(crate::app::RngKey::UpdateTime),
                );
            }
        }
        9 => {
            for n in app.npcs.iter_mut() {
                if n.hp > 0 && !n.kind.is_worm_head() {
                    n.hp = 0;
                }
            }
        }
        10 => {
            // The `pauseworld` toggle (`ToolkitDebugCommands.cs:742-749`):
            // `WorldUpdateStepper.TogglePaused` (`WorldUpdateStepper.cs:45-49`)
            // flips `Paused` and re-pins `StartUpdateCount` to the current
            // `GameUpdateCount` — the port's F3-panel surface (the recorded
            // deviation for the absent chat DebugCommand dispatcher).
            app.stepper.toggle_paused(app.game_update_count);
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn flow_app() -> App {
        crate::app::flow::front_flow_tests::flow_app(None, vec![], vec![])
    }

    /// The pause row is `EXACT_EQUIVALENT` to the C# (`WorldUpdateStepper.cs:45-49`)
    /// — the refactor must not move its semantics.
    #[test]
    fn s1_b_3_debug_panel_pauseworld_row_toggles() {
        let mut app = flow_app();
        app.game_update_count = 70000;
        app.debug_panel.open = true;
        app.debug_panel.tab = Tab::World;
        apply(
            &mut app,
            Event::RowClicked {
                tab: Tab::World,
                row: 10,
            },
        );
        assert!(app.stepper.paused, "the row toggled pauseworld on");
        assert_eq!(
            app.stepper.start_update_count, 70000,
            "the toggle pinned StartUpdateCount to GameUpdateCount"
        );
        apply(
            &mut app,
            Event::RowClicked {
                tab: Tab::World,
                row: 10,
            },
        );
        assert!(!app.stepper.paused, "the second click resumed the world");
    }

    #[test]
    fn s1_e_29_recipes_tab_gives_the_result() {
        let mut app = flow_app();
        app.debug_panel.open = true;
        app.debug_panel.tab = Tab::Recipes;
        let (id, count) = crate::craft::RECIPES[0].create;
        assert!(row_count(&app.debug_panel, Tab::Recipes) > 0);
        apply(
            &mut app,
            Event::RowClicked {
                tab: Tab::Recipes,
                row: 0,
            },
        );
        assert!(
            app.inventory
                .iter()
                .any(|s| s.kind == id && s.count >= count),
            "the recipe result landed in the inventory"
        );
    }

    #[test]
    fn s1_e_29_the_filter_narrows_the_npc_and_boss_tabs() {
        let mut app = flow_app();
        app.debug_panel.filter = "slime".to_string();
        assert_eq!(
            row_count(&app.debug_panel, Tab::Npcs),
            1,
            "only Blue Slime matches"
        );
        assert_eq!(
            row_count(&app.debug_panel, Tab::Bosses),
            1,
            "only King Slime matches"
        );
        app.debug_panel.filter = "zzz".to_string();
        assert_eq!(row_count(&app.debug_panel, Tab::Npcs), 0);
        assert_eq!(row_count(&app.debug_panel, Tab::Bosses), 0);
    }

    #[test]
    fn s1_e_29_the_wheel_scrolls_the_open_panel() {
        let mut app = flow_app();
        assert!(!handle_scroll(&mut app, -1.0), "a closed panel ignores the wheel");
        app.debug_panel.open = true;
        app.debug_panel.tab = Tab::Items;
        assert!(handle_scroll(&mut app, -1.0), "the open panel consumes it");
        assert_eq!(app.debug_panel.scroll, 3.0);
    }

    #[test]
    fn s1_e_29_the_click_outside_the_panel_falls_through() {
        let mut app = flow_app();
        app.debug_panel.open = true;
        assert!(!handle_click(&mut app, Vec2::new(5.0, 5.0)));
        let l = app.debug_panel.layout(Vec2::ZERO);
        assert!(
            handle_click(&mut app, Vec2::new(l.panel[0] + 4.0, l.panel[1] + 4.0)),
            "a title-bar click is consumed by the panel"
        );
    }

    /// `S1-E-37`: the World tab's event rows ride the C# entry points —
    /// `StartInvasion` sizes the army `80 + 40 x the qualifying players`
    /// (`Main.cs:64456`) behind the 200-life gate (`:64424-64432`), and
    /// `StartSlimeRain`/`StopSlimeRain` own the duration/cooldown draws
    /// (`:64767`, `:64805`) instead of the old hard-coded 43200 midpoint.
    #[test]
    fn s1_e_37_world_rows_ride_the_csharp_event_entry_points() {
        let mut app = flow_app();
        app.debug_panel.open = true;
        app.debug_panel.tab = Tab::World;
        app.player = Some(crate::player::Player::spawn(&app.world));

        // The 200-life gate: a 100-life character cannot call the army.
        app.player.as_mut().unwrap().max_hp = 100;
        apply(
            &mut app,
            Event::RowClicked {
                tab: Tab::World,
                row: 7,
            },
        );
        assert!(!app.world.invasion_goblins, "under 200 life refuses");

        app.player.as_mut().unwrap().max_hp = 200;
        apply(
            &mut app,
            Event::RowClicked {
                tab: Tab::World,
                row: 7,
            },
        );
        assert!(app.world.invasion_goblins);
        assert_eq!(
            app.world.invasion_size, 120,
            "80 + 40 x 1 qualifying player"
        );
        assert_eq!(app.world.invasion_size_start, 120);
        assert_eq!(app.world.invasion_type, 1, "the goblin type");
        // The march origin is `StartInvasion`'s `Next(2)` pick (`:64476-64483`).
        assert!(
            app.world.invasion_x == 0.0 || app.world.invasion_x == app.world.width as f64,
            "the side pick lands on one pole"
        );

        // The slime rain: the C# duration band, not the old 43200 midpoint.
        apply(
            &mut app,
            Event::RowClicked {
                tab: Tab::World,
                row: 8,
            },
        );
        assert!(app.world.slime_rain);
        assert!(
            (32400.0..54000.0).contains(&app.world.slime_rain_time),
            "Next(32400, 54000), got {}",
            app.world.slime_rain_time
        );
        assert_eq!(app.world.slime_rain_kills, 0);
        // The stop arms the negative `-Next(3024, 6048) * 100` cooldown.
        apply(
            &mut app,
            Event::RowClicked {
                tab: Tab::World,
                row: 8,
            },
        );
        assert!(!app.world.slime_rain);
        assert!(
            (-604_800.0..=-302_400.0).contains(&app.world.slime_rain_time),
            "the cooldown band, got {}",
            app.world.slime_rain_time
        );

        // The hardmode row runs the `initializeHardMode` runner behind the
        // flag (`WorldGen.cs:31707-31709`); the second click is the recorded
        // debug revert (the C# has no un-hardmode).
        apply(
            &mut app,
            Event::RowClicked {
                tab: Tab::World,
                row: 0,
            },
        );
        assert!(app.world.hard_mode);
        apply(
            &mut app,
            Event::RowClicked {
                tab: Tab::World,
                row: 0,
            },
        );
        assert!(!app.world.hard_mode, "the debug revert");
    }

    #[test]
    fn s1_e_29_the_world_tab_keeps_its_eleven_rows() {
        let app = flow_app();
        assert_eq!(row_count(&app.debug_panel, Tab::World), 11);
        let rows = world_rows(&app.world, false);
        assert_eq!(rows.len(), 11, "the draw and the dispatch agree");
        assert_eq!(rows[10].0, "Pause World: OFF");
    }
}
