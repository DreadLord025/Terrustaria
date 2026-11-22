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
    // S1-D-3 W-9 fishron wiring.
    (NpcKind::DukeFishron, "Duke Fishron", "NPC_370"),
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
        Tab::World => 16,
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
    god_mode: bool,
    raining: bool,
    sandstorm: bool,
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
            let rows = world_rows(world, stepper_paused, god_mode, raining, sandstorm);
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

/// The World tab's 16 rows. The order is the click dispatch's (`world_click`).
fn world_rows(
    world: &tr_core::World,
    stepper_paused: bool,
    god_mode: bool,
    raining: bool,
    sandstorm: bool,
) -> Vec<(String, [f32; 4])> {
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
            // The C# has one `invasionType` (1 goblins / 2 frost / 3 pirates /
            // 4 martians) — the row names whichever army is live.
            if world.invasion_type > 0 {
                let army = match world.invasion_type {
                    2 => "Frost Legion",
                    3 => "Pirates",
                    4 => "Martians",
                    _ => "Goblin Army",
                };
                format!("{army}: {} left (click to end)", world.invasion_size)
            } else if world.downed_goblins {
                "Goblin Army: defeated".to_string()
            } else {
                "Goblin Army: OFF (click to start)".to_string()
            },
            if world.invasion_type > 0 { R } else { Y },
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
        // S1-E-34 — the debug god-mode toggle. It writes the SAME flag the
        // journey god-mode power sets (`Player.creative_god_mode`, the
        // `Player.Hurt` arm `Player.cs:38203-38206`), so `hurt_inner`'s
        // first gate already swallows every hit; nothing new is needed on the
        // damage path.
        (
            format!("God Mode: {}", if god_mode { "ON" } else { "OFF" }),
            if god_mode { R } else { G },
        ),
        ("Reveal Map (pretty)".to_string(), W),
        ("Clear Map".to_string(), W),
        // S1-E-35 — the weather toggles. Both ride the vanilla entry
        // points (never a hand-written state write): the rain row is
        // `Main.StartRain` / `Main.StopRain`, the storm row is
        // `Sandstorm.StartSandstorm` / `StopSandstorm`. The mod cited by the
        // row (HEROsMod `GeneralMessages.cs:467-520`) is not in `decompiled/`
        // — its local half is exactly these calls, the network sync half is
        // out of scope for the single-player port (the recorded deviation).
        (
            format!("Rain: {}", if raining { "ON" } else { "OFF" }),
            if raining { R } else { G },
        ),
        (
            format!("Sandstorm: {}", if sandstorm { "ON" } else { "OFF" }),
            if sandstorm { R } else { G },
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
        | NpcKind::Plantera
        // S1-D-3 W-9 fishron wiring.
        | NpcKind::DukeFishron => {
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
            // S1-D-3-wire-destroyer: the head alone — the sim's
            // `update_destroyers` pass spawns the 81-link chain on its
            // first tick (`AI_037`, `NPC.cs:49833-49855`).
            let mut h = Npc::spawn_worm_head(&app.world, x, y, kind);
            h.hp = h.max_hp;
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
            // `WorldGen.StartHardmode` (`WorldGen.cs:31701-31719`): the real
            // entry — the strip walk plus the broadcast and ITS_HARD. The
            // panel cannot switch hardmode back off (vanilla can't either).
            crate::world_sim::growth::start_hardmode(app);
        }
        1 => {
            app.world.day_time = !app.world.day_time;
            app.world.time = 0.0;
        }
        2 => {
            // The blood-moon hit body (`Main.cs:65344-65346`) — the same
            // helper the dusk roll uses, so the dial cooldowns force-reset
            // exactly like a natural start. Off is the dawn clear
            // (`:65387`, the only place vanilla lowers the flag).
            if app.world.blood_moon {
                app.world.blood_moon = false;
            } else {
                app.world.start_blood_moon();
                crate::chat::broadcast_death_text(
                    &mut app.chat,
                    &crate::lang::text("LegacyMisc.8"),
                    crate::chat::COLOR_WORLD,
                );
            }
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
            // `Main.StartInvasion(1)` (`Main.cs:64435-64486`) through the
            // item path's gates (`Player.cs:43923-43932`): `CanStartInvasion`
            // then the delay clear + the start. No hand-written size — the
            // C# derives `80 + 40 x players` from the live roster.
            let life_max = app.player.as_ref().map(|p| p.max_hp).unwrap_or(0);
            if crate::world_sim::npcs::can_start_invasion(&app.world, life_max) {
                app.world.invasion_delay = 0;
                // The debug console is the chat command dispatcher's stand-in
                // (S1-E-29's recorded deviation): `Main.rand` at that point is
                // the bare stream, not a `SwapRandom` stage.
                crate::world_sim::npcs::start_invasion_in(app, 1, 1, None);
            } else {
                // Clicking an active invasion calls it off — the port's
                // `invasionSize = 0`, which the next `UpdateInvasion`
                // (`Main.cs:64332-64361`) resolves into the defeat flag, the
                // warning and the type clear.
                app.world.invasion_size = 0;
            }
        }
        8 => {
            // `Main.StartSlimeRain` / `Main.StopSlimeRain`
            // (`Main.cs:64742-64820`) with their own gates — the duration is
            // the C# `Next(32400, 54000)` draw, never a hardcoded midpoint.
            if app.world.slime_rain {
                crate::world_sim::weather::stop_slime_rain(
                    &mut app.world,
                    app.rngs.global(),
                );
            } else {
                let raining = app.rain.is_raining();
                crate::world_sim::weather::start_slime_rain(
                    &mut app.world,
                    raining,
                    app.rngs.global(),
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
        11 => {
            // S1-E-34 — god mode: flip the debug flag, which the player tick
            // ORs into `creative_god_mode` (the journey power's own flag must
            // not be clobbered, so the toggle rides a separate field).
            if let Some(p) = app.player.as_mut() {
                p.debug_god_mode = !p.debug_god_mode;
            }
        }
        12 => {
            // The `/map pretty` chat command (`ToolkitDebugCommands.cs:480-494`):
            // `Main.clearMap = true` (`:483`) then `DebugOptions.unlockMap = 2`
            // + `Main.refreshMap = true` (`:486-487`), or `unlockMap = 0` on
            // the toggle-off arm (`:491`). W4-f46 salvage (research/84 §4):
            // the row routes through the verbatim flag-path `unlock_map`
            // (the option is `App.debug_unlock_map`, the C# static's home).
            // The reveal itself rides the section sweep
            // (`Main.cs:61871-61879`): the `refreshMap` consume resets the
            // sweep (`:61890-61891`), and the 5 ms-budget walk unlocks
            // every loaded section over the following frames
            // (`:61873-61876`), zeroing the option once exhausted
            // (`:61880-61883`). The old direct bulk walk skipped the queue
            // entirely.
            if let Some(m) = app.map.as_mut() {
                m.unlock_map(&mut app.debug_unlock_map, true);
            }
        }
        13 => {
            // The `/clearmap` chat command (`ToolkitDebugCommands.cs:497-503`):
            // `Main.clearMap = true; Main.Map.Clear(); Main.refreshMap = true`.
            // The tile wipe is immediate; the texture wipe rides `clear_map`
            // (`MapRenderer.cs:161-173`, the port's full re-upload in
            // `prepare`), and the refresh resets the section sweep.
            if let Some(m) = app.map.as_mut() {
                m.clear_map = true; // `:499`
                m.clear(); // `:500` — WorldMap.Clear, WorldMap.cs:249-258
                m.refresh_map = true; // `:501`
            }
        }
        14 => {
            // S1-E-35 — `Main.StartRain` (`Main.cs:64659-64727`) /
            // `Main.StopRain` (`:64644-64658`): the vanilla entry points, so
            // the duration/strength ladders and their Rain-stream rolls run
            // exactly like a natural start (`rain.start_rain`), and the stop
            // clears `rainTime`/`maxRaining` for the visual ramp to decay.
            // `remixWorld` and `GetPlayerWithHighestLuck().luck` are the same
            // inputs `update_world_tick` passes (`weather.rs:178-187`).
            if app.rain.is_raining() {
                app.rain.stop_rain();
            } else {
                let remix_world = app.wind.remix_world;
                let player_luck = app.player.as_ref().map(|p| p.luck()).unwrap_or(0.0);
                app.rain.start_rain(&app.world, remix_world, player_luck);
            }
        }
        15 => {
            // S1-E-35 — `Sandstorm.StartSandstorm` (`Sandstorm.cs:117-122`) /
            // `StopSandstorm` (`:124-129`): the duration is the C#
            // `Next(28800, 86401)` draw on the storm's own stream, never a
            // hardcoded midpoint; the stop re-rolls the severity intention,
            // which `UpdateSeverity` then ramps the visible severity toward.
            if app.sandstorm.happening {
                app.sandstorm.stop_sandstorm();
            } else {
                app.sandstorm.start_sandstorm();
            }
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

    /// The World tab's `row` click, through the real dispatch.
    fn click_world(app: &mut App, row: usize) {
        app.debug_panel.open = true;
        app.debug_panel.tab = Tab::World;
        apply(app, Event::RowClicked { tab: Tab::World, row });
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

        // The hardmode row runs `WorldGen.StartHardmode` (`WorldGen.cs:31701-31719`).
        // A second click is a no-op: the C# early-returns when `hardMode` is
        // already set (`:31703`) and there is no un-hardmode in vanilla.
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
        assert!(app.world.hard_mode, "no un-hardmode exists (:31703)");
    }

    #[test]
    fn s1_e_35_weather_rows_use_the_vanilla_start_and_stop_paths() {
        let mut app = flow_app();
        // `Main.StartRain` consumes the Rain stream's duration/strength rolls;
        // the natural scheduler pin is off so the row uses those same entry
        // points. `Main.cs:64661` gates the coin-rain roll on
        // `Main.worldSurface > 50`.
        app.rain = crate::rain::Rain::with_forced(1, None, false);
        app.world.world_surface = 51.0;

        click_world(&mut app, 14);
        assert!(app.rain.is_raining());
        // `Main.cs:64676-64722`: the `rainTime` ladder — `Next(28800, 86400)`
        // plus the six bonuses, times the strength factor (<= 1.5).
        assert!(
            (28_800..=230_000).contains(&app.rain.rain_time()),
            "the rain duration band, got {}",
            app.rain.rain_time()
        );
        assert!(app.rain.max_raining() > 0.0);
        click_world(&mut app, 14);
        assert!(!app.rain.is_raining());
        assert_eq!(app.rain.rain_time(), 0);
        assert_eq!(app.rain.max_raining(), 0.0);

        // The storm uses `Next(28800, 86401)` and the C# intention ladder.
        click_world(&mut app, 15);
        assert!(app.sandstorm.happening);
        assert!((28800..=86400).contains(&app.sandstorm.time_left));
        // `Sandstorm.cs:83`: `0.4f + NextFloat()`.
        assert!((0.4..=1.4).contains(&app.sandstorm.intended_severity));
        click_world(&mut app, 15);
        assert!(!app.sandstorm.happening);
        assert_eq!(app.sandstorm.time_left, 0);
    }

    #[test]
    fn s1_e_29_the_world_tab_keeps_its_sixteen_rows() {
        let app = flow_app();
        assert_eq!(row_count(&app.debug_panel, Tab::World), 16);
        let rows = world_rows(&app.world, false, false, false, false);
        assert_eq!(rows.len(), 16, "the draw and the dispatch agree");
        assert_eq!(rows[10].0, "Pause World: OFF");
        // S1-E-34 — the god-mode toggle rides row 11.
        assert_eq!(rows[11].0, "God Mode: OFF");
        // S1-L-3 — the `/map pretty` reveal rides row 12.
        assert_eq!(rows[12].0, "Reveal Map (pretty)");
        // W4-e18 — the `/clearmap` command rides row 13.
        assert_eq!(rows[13].0, "Clear Map");
        // S1-E-35 — the weather toggles ride the last two rows.
        assert_eq!(rows[14].0, "Rain: OFF");
        assert_eq!(rows[15].0, "Sandstorm: OFF");
    }

    /// S1-E-34 — the F3 god-mode row flips the debug flag.
    #[test]
    fn s1_e_34_the_god_mode_row_flips_the_debug_flag() {
        let mut app = flow_app();
        app.player = Some(crate::player::Player::spawn(&app.world));
        app.debug_panel.open = true;
        app.debug_panel.tab = Tab::World;
        super::world_click(&mut app, 11);
        assert!(
            app.player.as_ref().unwrap().debug_god_mode,
            "the row toggled the debug flag on"
        );
        assert_eq!(
            world_rows(&app.world, false, true, false, false)[11].0,
            "God Mode: ON"
        );
        super::world_click(&mut app, 11);
        assert!(
            !app.player.as_ref().unwrap().debug_god_mode,
            "the row toggles back off"
        );
    }

    /// S1-L-3 — the tail row runs the `/map pretty` reveal
    /// (`ToolkitDebugCommands.cs:480-494`): it arms `DebugOptions.unlockMap
    /// = 2` + `Main.refreshMap`, and the section sweep (`Main.cs:61871-61879`)
    /// does the revealing over the following frames. W4-e18: the row no
    /// longer walks every section directly — the queue drives it.
    #[test]
    fn s1_l_3_the_reveal_row_unlocks_the_map() {
        let mut app = flow_app();
        // The reveal clamps into a 40-tile inset, so the fixture's 8x8 world is
        // too small to have an interior at all.
        app.world = tr_core::World::new("reveal".into(), 260, 200);
        app.map = Some(crate::map::WorldMap::new(&app.world));
        app.section_manager = crate::map_sections::MapSections::new(260, 200);
        click_world(&mut app, 12);
        // The command arms the option + the refresh, not the pixels.
        assert_eq!(app.debug_unlock_map, 2, "`:486`, the pretty mode");
        assert!(app.map.as_ref().unwrap().refresh_map, "`:487`");
        // The sweep's consume resets the walk, then the budget walk unlocks
        // the loaded sections (`:61873-61876`). One section fits the 260x200
        // grid (1x1), so a single sweep pass reveals it all.
        app.map_reveal_sweep_core(10_000.0);
        let m = app.map.as_ref().unwrap();
        assert!(
            m.tiles.iter().any(|t| t.light > 0),
            "the reveal walked every section"
        );
        // The second click is the toggle-off arm (`:491`).
        click_world(&mut app, 12);
        assert_eq!(app.debug_unlock_map, 0);
        // W4-e18: both arms also arm `Main.clearMap` (`:483`).
        assert!(app.map.as_ref().unwrap().clear_map);
    }

    /// W4-e18 residual census (written, not run — code-only): the
    /// `/clearmap` row (`ToolkitDebugCommands.cs:497-503`) — `clearMap =
    /// true`, `Main.Map.Clear()` (every tile zeroed), `refreshMap = true`.
    #[test]
    fn w4_e18_the_clearmap_row_wipes_the_tiles_and_arms_the_refresh() {
        let mut app = flow_app();
        app.world = tr_core::World::new("wipe".into(), 260, 200);
        app.map = Some(crate::map::WorldMap::new(&app.world));
        app.section_manager = crate::map_sections::MapSections::new(260, 200);
        // Reveal something first so the wipe is observable.
        click_world(&mut app, 12);
        app.map_reveal_sweep_core(10_000.0);
        assert!(app.map.as_ref().unwrap().tiles.iter().any(|t| t.light > 0));
        click_world(&mut app, 13);
        let m = app.map.as_ref().unwrap();
        assert!(m.clear_map, "`:499`");
        assert!(m.tiles.iter().all(|t| t.light == 0 && t.ty == 0), "`:500`");
        assert!(m.refresh_map, "`:501`");
    }

    /// S1-E-37: the goblin row is `Main.StartInvasion(1)`
    /// (`Main.cs:64435-64486`), not a hardcoded 80 — the size is
    /// `80 + 40 x the qualifying players`, the side is drawn from the bare
    /// stream, and the progress dial is initialised.
    #[test]
    fn s1_e_37_the_goblin_row_starts_a_sized_invasion() {
        let mut app = flow_app();
        let mut player = crate::player::Player::spawn(&app.world);
        player.max_hp = 200;
        app.player = Some(player);
        click_world(&mut app, 7);
        assert_eq!(app.world.invasion_type, 1);
        assert_eq!(
            app.world.invasion_size, 120,
            "80 + 40 x one qualifying player (not the old 80)"
        );
        assert_eq!(app.world.invasion_size_start, 120);
        assert_eq!(app.invasion_progress_icon, 4, "type + 3");
        assert_eq!(app.invasion_progress_max, 120);
        assert!(app.world.invasion_goblins, "the goblin flag stays synced");
        // The second click is the "call it off" arm: the size zeroes, which
        // the next `UpdateInvasion` resolves into the defeat.
        click_world(&mut app, 7);
        assert_eq!(app.world.invasion_size, 0);
    }

    /// S1-E-37: the goblin row refuses below the 200-life gate
    /// (`Main.cs:64427`) — it must not fabricate an invasion.
    #[test]
    fn s1_e_37_the_goblin_row_respects_the_life_gate() {
        let mut app = flow_app();
        let mut player = crate::player::Player::spawn(&app.world);
        player.max_hp = 100;
        app.player = Some(player);
        click_world(&mut app, 7);
        assert_eq!(app.world.invasion_type, 0, "the 200-life gate refuses");
        assert_eq!(app.world.invasion_size, 0);
    }

    /// S1-E-37: the slime-rain row is `Main.StartSlimeRain`
    /// (`Main.cs:64742-64785`) — the duration is the real
    /// `Next(32400, 54000)` draw, NEVER the old hardcoded 43200 midpoint.
    #[test]
    fn s1_e_37_the_slime_row_rolls_the_csharp_duration() {
        let mut app = flow_app();
        app.world.world_surface = 200.0; // `isThereAWorldSurface`
        // The seed-42 `UpdateTime`-equivalent draw: the bare stream's first
        // `Next(32400, 54000)`.
        let mut probe = tr_core::UnifiedRandom::new(app.world.seed as i32);
        let want = probe.next_range(32400, 54000) as f64;
        click_world(&mut app, 8);
        assert!(app.world.slime_rain, "the rain started");
        assert_eq!(
            app.world.slime_rain_time, want,
            "the C# Next(32400, 54000) draw, not a hardcoded 43200"
        );
        assert_ne!(app.world.slime_rain_time, 43200.0, "the old literal is gone");
        assert_eq!(app.world.slime_rain_kills, 0, ":64770");
        assert_eq!(app.world.slime_warning_time, 420, ":64776");
        // The stop arm: `StopSlimeRain`'s negative cooldown, not a zeroed
        // clock, and the kill counter is NOT cleared (`:64787-64820`).
        app.world.slime_rain_kills = 12;
        click_world(&mut app, 8);
        assert!(!app.world.slime_rain);
        assert!(
            app.world.slime_rain_time < 0.0,
            "the cooldown is -Next(3024, 6048) * 100, got {}",
            app.world.slime_rain_time
        );
        assert_eq!(app.world.slime_rain_kills, 12, "the stop keeps the counter");
        assert_eq!(app.world.slime_warning_time, 420, ":64811");
    }

    /// S1-E-37: hardmode goes through `WorldGen.StartHardmode`
    /// (`WorldGen.cs:31701-31719`) — the strip walk plus the broadcast and
    /// ITS_HARD — and cannot be toggled back off.
    #[test]
    fn s1_e_37_the_hardmode_row_runs_start_hardmode() {
        let mut app = flow_app();
        app.world = tr_core::World::new("hm".into(), 420, 300);
        let chat_before = app.chat.messages.len();
        click_world(&mut app, 0);
        assert!(app.world.hard_mode);
        assert_eq!(
            app.chat.messages.len(),
            chat_before + 1,
            "Lang.misc[15] broadcast (WorldGen.cs:31711)"
        );
        assert!(
            app.chat.messages.last().unwrap().text
                == crate::lang::text("LegacyMisc.15"),
            "the row broadcasts Lang.misc[15] (WorldGen.cs:31711), got {:?}",
            app.chat.messages.last().unwrap().text
        );
        // A second click is a no-op — vanilla cannot un-hardmode a world.
        let after_first = app.chat.messages.len();
        click_world(&mut app, 0);
        assert!(app.world.hard_mode);
        assert_eq!(app.chat.messages.len(), after_first, "no repeat broadcast");
    }

    /// S1-E-37: the blood-moon row takes the C# hit body
    /// (`Main.cs:65344-65346`) — both dial cooldowns force-reset, exactly
    /// like a natural dusk roll — and announces it (`:65353`).
    #[test]
    fn s1_e_37_the_blood_moon_row_uses_the_hit_body() {
        let mut app = flow_app();
        app.world.sundial_cooldown = 3;
        app.world.moondial_cooldown = 5;
        click_world(&mut app, 2);
        assert!(app.world.blood_moon);
        assert_eq!(app.world.sundial_cooldown, 0, ":65345");
        assert_eq!(app.world.moondial_cooldown, 0, ":65346");
        assert!(
            app.chat.messages.last().unwrap().text == crate::lang::text("LegacyMisc.8"),
            "the LegacyMisc.8 announcement, got {:?}",
            app.chat.messages.last().unwrap().text
        );
        // Off is the dawn clear (`:65387`).
        click_world(&mut app, 2);
        assert!(!app.world.blood_moon);
    }

    /// S1-E-37: the goblin row's label names whichever army is live — the C#
    /// has one `invasionType` (1 goblins / 2 frost / 3 pirates / 4 martians).
    #[test]
    fn s1_e_37_the_invasion_row_names_the_live_army() {
        let mut w = tr_core::World::new("t".into(), 64, 64);
        for (ty, want) in [
            (1, "Goblin Army: 100 left"),
            (2, "Frost Legion: 100 left"),
            (3, "Pirates: 100 left"),
            (4, "Martians: 100 left"),
        ] {
            w.invasion_type = ty;
            w.invasion_size = 100;
            let rows = world_rows(&w, false, false, false, false);
            assert!(
                rows[7].0.starts_with(want),
                "type {ty} row: {:?}",
                rows[7].0
            );
        }
        w.invasion_type = 0;
        assert!(
            world_rows(&w, false, false, false, false)[7]
                .0
                .starts_with("Goblin Army: OFF"),
            "the idle row"
        );
    }
}
