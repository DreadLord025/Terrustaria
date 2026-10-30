//! Terraria-Rust: Vulkan renderer prototype for a native reimplementation.

mod assets;
mod audio;
mod collision;
mod item;
mod map;
mod menu;
mod net;
mod npc;
mod persist;
mod player;
mod projectile;
mod rain;
mod registry;
mod spawner;
mod settings;
mod sound;

use std::collections::{HashMap, HashSet, VecDeque};
use std::path::PathBuf;
use std::sync::{Arc, OnceLock};
use std::time::Instant;

use anyhow::{Context, Result};
use glam::Vec2;
use item::ItemDrop;
use npc::{Npc, NpcKind};
use player::{BREATH_MAX, Player, PlayerInput, WIDTH as PLAYER_WIDTH};
use projectile::Projectile;
use rain::Rain;
use tr_core::lighting::{LightMap, LightMode, LightParams, LightStyle};
use tr_core::liquid::LiquidSystem;
use tr_core::tiledata as td;
use tr_core::{TILE_PX, World, edit, framing, save, tree, wld, worldgen};
use tr_render::{Atlas, Gpu, SpriteRenderer};
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::{ElementState, MouseButton, MouseScrollDelta, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Window, WindowId};

const PANEL_BG: [f32; 4] = [0.03, 0.04, 0.06, 0.82];
const TEXT: [f32; 4] = [0.85, 0.90, 0.95, 1.0];
const ACCENT: [f32; 4] = [0.45, 0.85, 0.55, 1.0];
const WARN: [f32; 4] = [0.95, 0.70, 0.30, 1.0];

const HOTBAR: usize = 10;
/// `Player.inventory` length: hotbar plus the four storage rows of the open grid.
const INV: usize = 50;
/// Seconds of one item swing; `Player.itemAnimation` is longer for real items.
const SWING_TIME: f32 = 0.28;

#[derive(Clone, Copy)]
struct ItemSlot {
    kind: u16,
    name: &'static str,
    count: u32,
}

/// `Item.MAX_STACK` for the blocks and items we model.
const MAX_STACK: u32 = 999;

const EMPTY_SLOT: ItemSlot = ItemSlot {
    kind: 0,
    name: "",
    count: 0,
};

/// `Player.PickupItem`: stack into the hotbar first, then an empty hotbar slot,
/// then the storage rows.
fn slot_for_pickup(inventory: &mut [ItemSlot; INV], kind: u16) -> Option<usize> {
    for range in [0..HOTBAR, HOTBAR..INV] {
        if let Some(i) = range.clone().find(|&i| inventory[i].count > 0 && inventory[i].kind == kind)
        {
            return Some(i);
        }
    }
    for range in [0..HOTBAR, HOTBAR..INV] {
        if let Some(i) = range
            .clone()
            .find(|&i| inventory[i].count == 0 || (inventory[i].kind == kind && inventory[i].count < registry::max_stack(kind)))
        {
            if inventory[i].count == 0 {
                inventory[i] = ItemSlot {
                    kind,
                    name: registry::name(kind),
                    count: 0,
                };
            }
            return Some(i);
        }
    }
    None
}

fn main() -> Result<()> {
    env_logger::Builder::from_env(
        // The Vulkan loader logs a benign registry warning on every start when no
        // Vulkan SDK layer manifests are installed; keep it out of the console.
        env_logger::Env::default().default_filter_or("info,wgpu_hal::vulkan::instance=error"),
    )
    .init();
    // The release build has no console; leave evidence for any panic instead of
    // losing the message when the window closes.
    std::panic::set_hook(Box::new(|info| {
        let backtrace = std::backtrace::Backtrace::force_capture();
        let msg = format!("panic: {info}
{backtrace}
");
        let _ = std::fs::write("crash.log", &msg);
        eprintln!("{msg}");
    }));

    let args: Vec<String> = std::env::args().collect();
    // No argument loads our own generated world (`Documents/My Games/Terrustaria`).
    // Passing a path still loads a `.wld` from the original game or a saved `.trwld`.
    let world_path = if let Some(p) = std::env::var_os("TR_WORLD") {
        PathBuf::from(p)
    } else {
        match args.get(1) {
            Some(p) => PathBuf::from(p),
            None => terrstaria_world()?,
        }
    };

    log::info!("world: {}", world_path.display());
    let t0 = Instant::now();
    let mut world = if world_path.extension().is_some_and(|e| e == "trwld") {
        save::load(&world_path)?
    } else {
        wld::load(&world_path)?
    };
    log::info!(
        "loaded '{}' {}x{} ({} tiles, {:.1} MB) in {:.0} ms",
        world.name,
        world.width,
        world.height,
        world.tiles.len(),
        world.memory_bytes() as f32 / 1e6,
        t0.elapsed().as_secs_f32() * 1000.0
    );
    // TR_DAY also sets the world clock so NPC day/night logic matches the pinned sky.
    if let Some(d) = std::env::var("TR_DAY")
        .ok()
        .and_then(|v| v.parse::<f32>().ok())
        .map(|v| v.clamp(0.0, 1.0))
    {
        world.day_time = d >= 0.5;
        world.time = if world.day_time {
            d as f64 * 54000.0
        } else {
            (1.0 - d as f64) * 32400.0
        };
    }

    let t_frame = Instant::now();
    let framed = framing::frame_world(&mut world);
    let tile_frame_ms = t_frame.elapsed().as_secs_f32() * 1000.0;
    let t_walls = Instant::now();
    let walled = framing::frame_walls(&mut world);
    let wall_frame_ms = t_walls.elapsed().as_secs_f32() * 1000.0;
    world.compute_column_surface();
    world.compute_chunks();
    // Torches are important tiles; generated saves written before the hotbar existed
    // may lack this bit.
    if world.frame_important.len() > 4 {
        world.frame_important[4] = true;
    }
    let slopes = world
        .tiles
        .iter()
        .filter(|t| t.active() && (t.slope() != 0 || t.half_brick()))
        .count();
    log::info!(
        "framed {framed} tiles in {tile_frame_ms:.0} ms, {walled} walls in {wall_frame_ms:.0} ms, {slopes} slopes"
    );
    log::info!(
        "spawn {},{}, surface {:.1}, rock layer {:.1}, time {:.0} ({})",
        world.spawn_x,
        world.spawn_y,
        world.world_surface,
        world.rock_layer,
        world.time,
        if world.day_time { "day" } else { "night" }
    );

    let content_dir = find_content_dir().context(
        "assets/Content not found: copy the retail Content directory next to the executable",
    )?;
    assets::set_content_dir(content_dir.clone());
    log::info!("content: {}", content_dir.display());
    let assets_dir = content_dir.join("Images");

    if std::env::var("TR_DUMP_TILES").is_ok() {
        let cx = std::env::var("TR_DUMP_X")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(world.width / 2);
        let cy = std::env::var("TR_DUMP_Y")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(world.spawn_y + 12);
        for y in cy..cy + 8 {
            let mut line = String::new();
            for x in cx..cx + 16 {
                let t = world.tile(x, y).copied().unwrap_or_default();
                line.push_str(&format!(
                    "{}:{}:{}/{}/w{}L{} ",
                    if t.active() { t.kind } else { 999 },
                    t.slope(),
                    t.frame_x,
                    t.frame_y,
                    t.wall,
                    t.liquid
                ));
            }
            log::info!("row {y}: {line}");
        }
    }
    if std::env::var("TR_SCAN_WATER").is_ok() {
        // Finds the densest 40x40 window of liquid cells, for flow/liquid screenshots.
        // The map edges (oceans) are skipped so the inland lake wins.
        const STEP: i32 = 40;
        let mut best = (0usize, 0i32, 0i32);
        let mut y = 300;
        while y + STEP < world.height {
            let mut x = 300;
            while x + STEP < world.width - 300 {
                let mut n = 0usize;
                for yy in y..y + STEP {
                    for xx in x..x + STEP {
                        if world.tile(xx, yy).is_some_and(|t| t.liquid > 0) {
                            n += 1;
                        }
                    }
                }
                if n > best.0 {
                    best = (n, x, y);
                }
                x += STEP;
            }
            y += STEP;
        }
        log::info!(
            "densest liquid window: {} at {},{} -> TR_CAM={},{}",
            best.0,
            best.1,
            best.2,
            best.1 + STEP / 2,
            best.2 + STEP / 2
        );
    }
    if std::env::var("TR_SCAN_TORCH").is_ok() {
        // Finds the densest 40x40 window of torches, for lighting-mode screenshots.
        const STEP: i32 = 40;
        let mut best = (0usize, 0i32, 0i32);
        let mut y = 0;
        while y + STEP < world.height {
            let mut x = 0;
            while x + STEP < world.width {
                let mut n = 0usize;
                for yy in y..y + STEP {
                    for xx in x..x + STEP {
                        if world
                            .tile(xx, yy)
                            .is_some_and(|t| t.active() && t.kind == 4)
                        {
                            n += 1;
                        }
                    }
                }
                if n > best.0 {
                    best = (n, x, y);
                }
                x += STEP;
            }
            y += STEP;
        }
        log::info!(
            "densest torch window: {} at {},{} -> TR_CAM={},{}",
            best.0,
            best.1,
            best.2,
            best.1 + STEP / 2,
            best.2 + STEP / 2
        );
    }
    // `TR_SCAN_LIGHT`: finds the 40x40 tile window with the most self-lit tiles
    // (torches, glowing mushrooms, lava...) so screenshots can target a lit cave.
    if std::env::var("TR_SCAN_LIGHT").is_ok() {
        const STEP: i32 = 40;
        let bounds = std::env::var("TR_SCAN_Y")
            .ok()
            .and_then(|v| {
                let (a, b) = v.split_once(',')?;
                Some((a.trim().parse::<i32>().ok()?, b.trim().parse::<i32>().ok()?))
            })
            .unwrap_or((0, world.height));
        let mut best = (0usize, 0i32, 0i32);
        let mut y = bounds.0.max(0);
        while y + STEP < bounds.1.min(world.height) {
            let mut x = 0;
            while x + STEP < world.width {
                let mut n = 0usize;
                for yy in y..y + STEP {
                    for xx in x..x + STEP {
                        let t = world.tile(xx, yy).copied().unwrap_or_default();
                        if !t.active() {
                            continue;
                        }
                        let e = td::EMISSIVE[t.kind as usize];
                        let emissive =
                            (t.kind as usize) < td::TILE_COUNT && (e[0].max(e[1]).max(e[2]) > 0.2);
                        if emissive || t.liquid_kind() == tr_core::tile::LiquidKind::Lava {
                            n += 1;
                        }
                    }
                }
                if n > best.0 {
                    best = (n, x, y);
                }
                x += STEP;
            }
            y += STEP;
        }
        log::info!(
            "brightest 40x40 window: {} tiles at {},{} -> TR_CAM={},{} (tiles)",
            best.0,
            best.1,
            best.2,
            best.1 + STEP / 2,
            best.2 + STEP / 2
        );
    }

    let (mut tile_ids, wall_ids) = assets::usage(&world);
    // The hotbar can place these even when the loaded world does not use them yet.
    tile_ids.insert(4); // torch
    tile_ids.insert(7); // copper ore
    log::info!(
        "world uses {} tile types and {} wall types",
        tile_ids.len(),
        wall_ids.len()
    );

    let t1 = Instant::now();
    let mut images = assets::load_world_images(&assets_dir, &tile_ids, &wall_ids)?;
    let (font_images, ui_font_meta, item_font_meta, death_font_meta) = assets::load_fonts();
    images.extend(font_images);
    let font = tr_render::atlas::font_sheet();
    images.push(font.0);
    log::info!(
        "decoded {} sheets in {:.0} ms",
        images.len(),
        t1.elapsed().as_secs_f32() * 1000.0
    );

    let t2 = Instant::now();
    let atlas = Atlas::build(images, 8192);
    log::info!(
        "atlas {}x{} ({:.1} MB, {} sheets) in {:.0} ms",
        atlas.width,
        atlas.height,
        atlas.memory_bytes() as f32 / 1e6,
        atlas.sheets.len(),
        t2.elapsed().as_secs_f32() * 1000.0
    );
    if atlas.height > 8192 {
        log::warn!("atlas exceeds 8192 px in height; some sheets may be cut off on strict GPUs");
    }

    let ui_font = ui_font_meta
        .as_ref()
        .and_then(|(sheet, f)| assets::build_ui_font(&atlas, sheet, f));
    let item_font = item_font_meta
        .as_ref()
        .and_then(|(sheet, f)| assets::build_ui_font(&atlas, sheet, f));
    let death_font = death_font_meta
        .as_ref()
        .and_then(|(sheet, f)| assets::build_ui_font(&atlas, sheet, f));
    let lookups = assets::Lookups::build(
        &atlas,
        tile_ids
            .iter()
            .map(|&id| (id, assets::image_name_for_tile(id))),
        wall_ids
            .iter()
            .map(|&id| (id, assets::image_name_for_wall(id))),
        ui_font,
        item_font,
        death_font,
    );

    let atlas_stats = (
        atlas.width,
        atlas.height,
        atlas.memory_bytes(),
        atlas.sheets.len(),
    );

    if std::env::var("TR_CHECK_FRAMES").is_ok() {
        for (kind, slot) in lookups.tiles.iter().enumerate() {
            let Some(tt) = slot else { continue };
            if let Some(rect) = atlas.sheet(&tt.sheet) {
                let fx = rect.plain_frame.0.max(0) as u32;
                let fy = rect.plain_frame.1.max(0) as u32;
                let mut opaque = 0;
                for y in 0..16u32 {
                    for x in 0..16u32 {
                        let px = rect.x + fx + x;
                        let py = rect.y + fy + y;
                        let i = ((py * atlas.width + px) * 4 + 3) as usize;
                        if atlas.pixels[i] >= 250 {
                            opaque += 1;
                        }
                    }
                }
                if opaque < 200 {
                    log::warn!(
                        "tile {kind} ({}) plain frame is {opaque}/256 opaque",
                        tt.sheet
                    );
                }
            }
        }
        for (wall, slot) in lookups.walls.iter().enumerate() {
            let Some(tt) = slot else { continue };
            if let Some(rect) = atlas.sheet(&tt.sheet) {
                let fx = rect.plain_frame.0.max(0) as u32;
                let fy = rect.plain_frame.1.max(0) as u32;
                let mut opaque = 0;
                for y in 0..16u32 {
                    for x in 0..16u32 {
                        let px = rect.x + fx + x;
                        let py = rect.y + fy + y;
                        let i = ((py * atlas.width + px) * 4 + 3) as usize;
                        if atlas.pixels[i] >= 250 {
                            opaque += 1;
                        }
                    }
                }
                if opaque < 200 {
                    log::warn!(
                        "wall {wall} ({}) plain frame is {opaque}/256 opaque",
                        tt.sheet
                    );
                }
            }
        }
    }

    if let Ok(path) = std::env::var("TR_ATLAS_DUMP") {
        image::save_buffer(
            &path,
            &atlas.pixels,
            atlas.width,
            atlas.height,
            image::ColorType::Rgba8,
        )
        .ok();
        log::info!("atlas dumped to {path}");
        for name in [
            "Tiles_0",
            "Tiles_1",
            "Tiles_2",
            "Wall_1",
            "Background_0",
            "LiquidSlope_0",
            "Liquid_0",
            "Misc/water_0",
        ] {
            if let Some(s) = atlas.sheet(name) {
                log::info!(
                    "{name}: atlas rect {}x{} at {},{} plain {:?}",
                    s.width,
                    s.height,
                    s.x,
                    s.y,
                    s.plain_frame
                );
            }
        }
    }

    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(ControlFlow::Poll);
    let clouds = make_clouds(&world, &lookups);
    let sky = make_sky_assets(&atlas);
    let stars = make_stars(&world);
    let player_assets = make_player_assets(&atlas);
    let npc_assets = make_npc_assets(&atlas);
    let player_enabled = std::env::var_os("TR_NOPLAYER").is_none();
    let mut player = player_enabled.then(|| Player::spawn(&world));
    let mut npcs = Vec::new();
    if let Some(p) = &player {
        let xt = (p.x / TILE_PX) as i32;
        for (i, dx) in [-20, 16, 26].iter().enumerate() {
            let spawned = if world.day_time {
                Npc::spawn_slime(&world, xt + dx)
            } else {
                match i {
                    0 => Npc::spawn_zombie(&world, xt + dx),
                    1 => Npc::spawn_eye(&world, xt + dx),
                    _ => Npc::spawn_slime(&world, xt + dx),
                }
            };
            if let Some(n) = spawned {
                npcs.push(n);
            }
        }
    }
    let follow_player = player.is_some()
        && std::env::var_os("TR_CAM").is_none()
        && std::env::var_os("TR_PAN").is_none();
    let mut coins = 0u64;
    let mut inventory: [ItemSlot; INV] = std::array::from_fn(|i| match i {
        0 => ItemSlot {
            kind: 2, // ItemID.DirtBlock
            name: registry::name(2),
            count: 50,
        },
        1 => ItemSlot {
            kind: 3, // ItemID.StoneBlock
            name: registry::name(3),
            count: 50,
        },
        2 => ItemSlot {
            kind: 8, // ItemID.Torch
            name: registry::name(8),
            count: 99,
        },
        3 => ItemSlot {
            kind: 42, // ItemID.Shuriken
            name: registry::name(42),
            count: 99,
        },
        _ => ItemSlot {
            kind: 0,
            name: "",
            count: 0,
        },
    });
    if world_path.extension().is_some_and(|e| e == "trwld") {
        if let Some(saved) = persist::load(&world_path) {
            if let Some(p) = player.as_mut() {
                if !player::collides_player(&world, saved.x, saved.y) {
                    p.x = saved.x;
                    p.y = saved.y;
                }
                p.hp = saved.hp.clamp(1, p.max_hp);
                coins = saved.coins;
                for (i, (kind, count)) in saved.inventory.iter().enumerate().take(INV) {
                    inventory[i] = ItemSlot {
                        kind: *kind,
                        name: registry::name(*kind),
                        count: *count,
                    };
                }
            }
        }
    }
    if let Ok(v) = std::env::var("TR_COINS") {
        if let Ok(n) = v.parse::<u64>() {
            coins = n;
        }
    }
    // Debug spawn override: place the player at a tile coordinate (the camera follows).
    if let Some((tx, ty)) = std::env::var("TR_XY").ok().and_then(|v| parse_xy(&v)) {
        if let Some(p) = player.as_mut() {
            p.teleport(
                tx as f32 * TILE_PX + 8.0 - player::WIDTH * 0.5,
                ty as f32 * TILE_PX,
            );
        }
    }
    if std::env::var_os("TR_BOSS").is_some() {
        if let Some(p) = &player {
            if let Some(n) = Npc::spawn_king_slime(&world, (p.x / TILE_PX) as i32 + 12) {
                npcs.push(n);
            }
        }
    }
    if let Ok(v) = std::env::var("TR_SLIME_AT") {
        for x in v.split(',').filter_map(|s| s.trim().parse::<i32>().ok()) {
            if let Some(n) = Npc::spawn_slime(&world, x) {
                npcs.push(n);
            }
        }
    }
    let dev_break = std::env::var("TR_BREAK").ok().and_then(|v| parse_xy(&v));
    // `TR_PLACE=x,y,kind[;x,y,kind…]`: place tiles on the first update, in order.
    let dev_place: Vec<(i32, i32, u16)> = std::env::var("TR_PLACE")
        .ok()
        .map(|v| {
            v.split(';')
                .filter_map(|entry| {
                    let mut it = entry.split(',');
                    let x = it.next()?.trim().parse::<i32>().ok()?;
                    let y = it.next()?.trim().parse::<i32>().ok()?;
                    let k = it.next()?.trim().parse::<u16>().ok()?;
                    Some((x, y, k))
                })
                .collect()
        })
        .unwrap_or_default();
    // `TR_SPOUT=x,y`: build the `FindWaterfalls` half-brick case at a fixed tile (a half
    // brick with full liquid on its right and a clear drop below) so the waterfall
    // overlay can be screenshotted deterministically.
    if let Some((x, y)) = std::env::var("TR_SPOUT").ok().and_then(|v| parse_xy(&v)) {
        for j in 1..=14 {
            edit::break_tile(&mut world, x, y + j);
        }
        edit::break_tile(&mut world, x, y);
        if edit::place_tile(&mut world, x, y, 1) {
            if let Some(t) = world.tile_mut(x, y) {
                t.flags |= tr_core::tile::FLAG_HALF_BRICK;
            }
        }
        for (dx, dy) in [(1, 0), (1, -1)] {
            if let Some(t) = world.tile_mut(x + dx, y + dy) {
                t.liquid = 255;
            }
        }
    }
    let has_liquids = world.tiles.iter().any(|t| t.liquid > 0);
    let world_seed = world.seed;
    let liquids_sim = LiquidSystem::new(&world);
    let rain = Rain::new(world_seed);

    // Character name and the session menu (`M6`): screenshots and debug hooks skip
    // the menu so the deterministic output stays unchanged.
    let player_name =
        std::env::var("TR_NAME").unwrap_or_else(|_| menu::random_name());
    let join_target = std::env::var("TR_JOIN").ok().map(|addr| {
        (
            addr,
            std::env::var("TR_JOIN_PW").unwrap_or_default(),
        )
    });
    let auto_host = std::env::var("TR_HOST")
        .ok()
        .and_then(|p| p.parse::<u16>().ok());
    let joined_before = std::env::var_os("TR_JOINED").is_some();
    let force_menu = std::env::var_os("TR_MENU").is_some();
    let skip_menu = (!force_menu && std::env::var_os("TR_SHOT").is_some())
        || std::env::var_os("TR_CAM").is_some()
        || std::env::var_os("TR_HOLD").is_some()
        || std::env::var_os("TR_NOMENU").is_some()
        || joined_before
        || auto_host.is_some();
    let menu_state = if skip_menu {
        None
    } else {
        let mut m = menu::Menu::new(player_name.clone());
        m.connecting = join_target.is_some();
        Some(m)
    };
    let mut net_role = NetRole::Off;
    if let Some((addr, pw)) = join_target.as_ref() {
        match net::Client::connect(addr, player_name.clone(), pw.clone()) {
            Ok(c) => {
                log::info!("joining {addr}");
                net_role = NetRole::Client(c);
            }
            Err(e) => log::warn!("connect to {addr} failed: {e:#}"),
        }
    }
    if let Some(port) = auto_host {
        match net::Server::start(port, String::new(), compressed_world(&world)) {
            Ok(server) => {
                log::info!("hosting on {}", server.addr());
                net_role = NetRole::Host(server);
            }
            Err(e) => log::warn!("host on {port} failed: {e:#}"),
        }
    }
    let mut world_map = map::WorldMap::new(&world, &atlas, &lookups);
    world_map.open = std::env::var_os("TR_MAP").is_some();
    let mut app = App {
        window: None,
        gpu: None,
        renderer: None,
        world,
        atlas: Some(atlas),
        lookups,
        clouds,
        sky,
        // `Main` statics initialise `bgAlphaFrontLayer[0] = bgAlphaFarBackLayer[0] = 1`
        // (forest visible from the start).
        bg_style: 0,
        bg_delay: 0,
        // `Main.instantBGTransitionCounter = 2` on world load.
        bg_instant: 2,
        bg_alpha_front: {
            let mut a = [0.0; 16];
            a[0] = 1.0;
            a
        },
        bg_alpha_far: {
            let mut a = [0.0; 16];
            a[0] = 1.0;
            a
        },
        stars,
        player_assets,
        player,
        follow_player,
        player_accum: 0.0,
        npc_accum: 0.0,
        npcs,
        npc_assets,
        npc_timer: 6.0,
        npc_rng: world_seed,
        attack_cd: 0.0,
        inventory,
        selected: 0,
        hotbar_scales: {
            let mut s = [0.75; HOTBAR];
            s[0] = 1.0;
            s
        },
        inventory_open: std::env::var_os("TR_INV").is_some(),
        menu: menu_state,
        settings: settings::Settings::load(),
        player_name,
        net: net_role,
        remote_players: HashMap::new(),
        local_id: 0,
        net_timer: 0.0,
        join_target,
        map: Some(world_map),
        audio: audio::Audio::new(assets::content_dir()),
        music: audio::MusicSystem::new(assets::content_dir()),
        last_hp: 100,
        mouse_item: EMPTY_SLOT,
        ui_left_press: false,
        ui_right_press: false,
        mouse_pos: Vec2::ZERO,
        mine_down: false,
        place_down: false,
        mine_target: None,
        mine_timer: 0.0,
        place_cooldown: 0.0,
        edit_target: None,
        world_path: world_path.clone(),
        dev_break,
        dev_place,
        camera: Vec2::ZERO,
        zoom: 1.0,
        zoom_pinned: std::env::var_os("TR_ZOOM").is_some(),
        debug: std::env::var("TR_DEBUG").map(|v| v != "0").unwrap_or(false),
        keys: HashSet::new(),
        mouse_down: false,
        last_mouse: Vec2::ZERO,
        last_frame: Instant::now(),
        frame_history: VecDeque::with_capacity(256),
        last_stats: tr_render::sprite::RenderStats::default(),
        bench_frames: std::env::var("TR_BENCH").ok().and_then(|v| v.parse().ok()),
        bench_samples: Vec::new(),
        frame_count: 0,
        shot_path: std::env::var("TR_SHOT").ok().map(PathBuf::from),
        shot_at: std::env::var("TR_SHOT_FRAME")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(20),
        atlas_stats,
        light: LightMap::new(),
        scratch: DrawScratch::default(),
        tile_stats: TileDrawStats::default(),
        stats_on: std::env::var("TR_STATS").is_ok(),
        stats_acc: StatsAcc::default(),
        debug_cache: DebugCache::default(),
        light_mode: std::env::var("TR_LIGHT_MODE")
            .ok()
            .and_then(|v| parse_light_mode(&v))
            .unwrap_or(LightMode::Color),
        // `Main.mouseTextColor` starts at 225 and drifts toward 255.
        mouse_text_color: 225,
        mouse_text_color_dir: 1,
        // `Main.demonTorch = 1f; Main.demonTorchDir = 1` (Main.Initialize).
        demon_torch: 1.0,
        demon_torch_dir: 1.0,
        // `Main.DiscoR = 255` (Main.Initialize); G/B and the style start at 0.
        disco: [255, 0, 0],
        disco_style: 0,
        light_rng_seed: world_seed | 1,
        anim_time: 0.0,
        has_liquids,
        liquids: liquids_sim,
        liquid_accum: 0.0,
        liquid_tick: 0,
        waterfall: tr_core::waterfall::WaterfallManager::new(),
        waterfall_accum: 0.0,
        rain,
        items: Vec::new(),
        item_rng: world_seed ^ 0x5DEE_CE66_D1A4_9B5F,
        item_accum: 0.0,
        swing: 0.0,
        coins,
        projectiles: Vec::new(),
        projectile_accum: 0.0,
        dev_throw: std::env::var_os("TR_THROW").is_some(),
        world_key: None,
    };
    event_loop.run_app(&mut app)?;
    Ok(())
}

fn terrstaria_world() -> Result<PathBuf> {
    let path = save::default_world_path("Terrustaria")
        .context("cannot resolve Documents/My Games/Terrustaria")?;
    let regen = std::env::var_os("TR_REGEN").is_some();
    if !path.exists() || regen {
        let seed = std::env::var("TR_SEED")
            .ok()
            .and_then(|v| v.parse::<u64>().ok())
            .unwrap_or(1);
        let t = Instant::now();
        let world = worldgen::generate(&worldgen::GenOptions {
            seed,
            ..Default::default()
        });
        save::save(&path, &world)?;
        log::info!(
            "generated '{}' (seed {seed}) in {:.0} ms -> {}",
            world.name,
            t.elapsed().as_secs_f32() * 1000.0,
            path.display()
        );
    }
    Ok(path)
}

/// Locates the bundled retail `Content` directory: next to the executable first (the
/// release layout), then the project tree.
fn find_content_dir() -> Option<PathBuf> {
    let exe = std::env::current_exe().ok();
    let mut candidates = Vec::new();
    if let Some(exe) = exe.as_ref().and_then(|p| p.parent()) {
        candidates.push(exe.join("assets/Content"));
        candidates.push(exe.join("../../assets/Content"));
    }
    candidates.push(PathBuf::from("assets/Content"));
    candidates.push(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/Content"));
    candidates.into_iter().find(|p| p.join("Images").is_dir())
}

/// Multiplayer role of this process (`M6`).
enum NetRole {
    Off,
    Host(net::Server),
    Client(net::Client),
}

struct App {
    window: Option<Arc<Window>>,
    gpu: Option<Gpu>,
    renderer: Option<SpriteRenderer>,
    world: World,
    atlas: Option<Atlas>,
    lookups: assets::Lookups,
    clouds: Vec<CloudSprite>,
    sky: SkyAssets,
    /// `Main.bgStyle`: the biome background style currently being drawn (0 forest,
    /// 2 desert, 4 ocean, 7 snow; the full vanilla table also has 1/3/5/6/8/9/10..14).
    bg_style: i32,
    /// `Main.bgDelay`: ticks to wait before starting a new transition (30 normally,
    /// 60 for a forest->forest variant swap); negative while the post-switch
    /// cooldown counts back up.
    bg_delay: i32,
    /// `Main.instantBGTransitionCounter`: forces `bgStyle` on the first frames
    /// after a world load so the background starts on the right biome.
    bg_instant: i32,
    /// `Main.bgAlphaFrontLayer` (16 slots): per-front-layer cross-fade alpha.
    bg_alpha_front: [f32; 16],
    /// `Main.bgAlphaFarBackLayer` (16 slots): per far-back (mountain) layer alpha.
    bg_alpha_far: [f32; 16],
    stars: Vec<StarSprite>,
    player_assets: PlayerAssets,
    player: Option<Player>,
    follow_player: bool,
    player_accum: f32,
    npc_accum: f32,
    npcs: Vec<Npc>,
    npc_assets: NpcAssets,
    npc_timer: f32,
    npc_rng: u64,
    attack_cd: f32,
    inventory: [ItemSlot; INV],
    selected: usize,
    /// Vanilla-style per-slot hotbar scale (selected grows to 1.0).
    hotbar_scales: [f32; HOTBAR],
    /// Escape toggles the inventory panel (`Main.OpenInventory`), it no longer quits.
    inventory_open: bool,
    /// Main menu; `None` while playing.
    menu: Option<menu::Menu>,
    /// `config.json` options edited by the settings screens.
    settings: settings::Settings,
    player_name: String,
    net: NetRole,
    /// Players of other machines streaming from the host/client.
    remote_players: HashMap<u32, net::PlayerNet>,
    /// Our id assigned by the host (0 while hosting).
    local_id: u32,
    net_timer: f32,
    join_target: Option<(String, String)>,
    /// Explored-tile minimap and full map (`M`).
    map: Option<map::WorldMap>,
    audio: audio::Audio,
    music: audio::MusicSystem,
    last_hp: i32,
    /// Stack carried on the cursor while the inventory is open (`Main.mouseItem`).
    mouse_item: ItemSlot,
    ui_left_press: bool,
    ui_right_press: bool,
    mouse_pos: Vec2,
    mine_down: bool,
    place_down: bool,
    mine_target: Option<(i32, i32)>,
    mine_timer: f32,
    place_cooldown: f32,
    edit_target: Option<(i32, i32)>,
    world_path: PathBuf,
    dev_break: Option<(i32, i32)>,
    dev_place: Vec<(i32, i32, u16)>,
    camera: Vec2,
    zoom: f32,
    /// `TR_ZOOM` pins the zoom for screenshots, ignoring the settings value.
    zoom_pinned: bool,
    debug: bool,
    keys: HashSet<KeyCode>,
    mouse_down: bool,
    last_mouse: Vec2,
    last_frame: Instant,
    frame_history: VecDeque<f32>,
    last_stats: tr_render::sprite::RenderStats,
    bench_frames: Option<u32>,
    bench_samples: Vec<f32>,
    frame_count: u64,
    shot_path: Option<PathBuf>,
    shot_at: u64,
    atlas_stats: (u32, u32, usize, usize),
    /// Reused light-map storage (D1) and dark-rect merge scratch (A3/A4).
    light: LightMap,
    scratch: DrawScratch,
    tile_stats: TileDrawStats,
    stats_on: bool,
    stats_acc: StatsAcc,
    debug_cache: DebugCache,
    light_mode: LightMode,
    /// `Main.mouseTextColor`: oscillates 190..=255 each frame, driving torch/lava/shimmer
    /// crystal-wall flicker (`DoUpdate_AnimateCursorColors` in Main.cs).
    mouse_text_color: u8,
    mouse_text_color_dir: i8,
    /// `Main.demonTorch` / `Main.demonTorchDir` (`DoUpdate_AnimateTileGlows`).
    demon_torch: f32,
    demon_torch_dir: f32,
    /// `Main.DiscoR/G/B` and the `Main.DiscoStyle` cycle (`DoUpdate_AnimateDiscoRGB`).
    disco: [u8; 3],
    disco_style: u8,
    /// `TileLightScanner._random` seed; advanced once per frame like `NextSeed()`.
    light_rng_seed: u64,
    anim_time: f32,
    has_liquids: bool,
    /// Cellular liquid flow (`Liquid.UpdateLiquid`), ticked at 30 Hz.
    liquids: LiquidSystem,
    liquid_accum: f32,
    liquid_tick: u32,
    /// `Main.waterfallManager`: scanning, animation counters and draw geometry.
    waterfall: tr_core::waterfall::WaterfallManager,
    waterfall_accum: f32,
    /// Weather and raindrops (`Rain.MakeRain` / `Rain.Update`).
    rain: Rain,
    /// World item drops (`Terraria.Item` first pass).
    items: Vec<ItemDrop>,
    item_rng: u64,
    item_accum: f32,
    /// Remaining seconds of the current held-item swing.
    swing: f32,
    /// Coin purse value in copper coins.
    coins: u64,
    /// Live shuriken/projectile entities.
    projectiles: Vec<Projectile>,
    projectile_accum: f32,
    /// `TR_THROW` debug hook: throw one shuriken at startup.
    dev_throw: bool,
    /// G6: identity of the last world instance range; a hit lets the frame reuse it.
    world_key: Option<WorldFrameKey>,
}

/// G6: everything the world instance range (sky strips, tiles, walls, dark rects)
/// depends on, compared bit-exactly. A hit keeps the previous frame's instances and
/// only redraws the UI; the light map has its own cache (D2) inside `LightMap`.
#[derive(Clone, Copy, PartialEq)]
struct WorldFrameKey {
    /// Camera floored to its tile: sub-tile pans keep the world range (the GPU
    /// transform moves it) while the background range is rebuilt every frame.
    cam_tile: [i32; 2],
    zoom: u32,
    screen: [u32; 2],
    sky_light: [u32; 3],
    hell_glow: u32,
    /// `LightParams::identity()`: every tile-light parameter (mouseTextColor,
    /// demon torch, disco, shimmer...) in one value.
    light_params: u64,
    light_mode: u32,
    /// Non-zero only for modes whose light depends on time ("Trippy").
    anim: u32,
    /// Player position/animation identity; the player is part of the world range.
    player_key: u32,
    /// Hash of all NPC positions/animations.
    npc_key: u32,
    /// Item drops and their ground-bob step.
    items_key: u32,
    /// Held item kind + swing phase.
    hand_key: u32,
    /// Light-map generation; a recompute (D3) changes tile tints.
    light_generation: u64,
    /// Waterfall animation frames + visible-list revision (0 when none visible).
    waterfall: u64,
    world_revision: u64,
}

fn parse_light_mode(v: &str) -> Option<LightMode> {
    match v.trim().to_ascii_lowercase().as_str() {
        "color" | "colour" | "0" => Some(LightMode::Color),
        "white" | "1" => Some(LightMode::White),
        "retro" | "2" => Some(LightMode::Retro),
        "trippy" | "glitch" | "3" => Some(LightMode::Trippy),
        _ => None,
    }
}

fn parse_xy(v: &str) -> Option<(i32, i32)> {
    let (a, b) = v.split_once(',')?;
    Some((a.trim().parse().ok()?, b.trim().parse().ok()?))
}

/// `TR_DAY` pins the sky for screenshots; time-of-day advancement pauses.
fn time_frozen() -> bool {
    static FROZEN: OnceLock<bool> = OnceLock::new();
    *FROZEN.get_or_init(|| std::env::var_os("TR_DAY").is_some())
}

/// `Main.time`: a 15-minute day (`0..54000`) and a 9-minute night (`0..32400`) at 60
/// ticks per second.
fn advance_time(world: &mut World, dt: f32) {
    let mut t = world.time + dt as f64 * 60.0;
    if world.day_time {
        if t >= 54000.0 {
            t -= 54000.0;
            world.day_time = false;
        }
    } else if t >= 32400.0 {
        t -= 32400.0;
        world.day_time = true;
    }
    world.time = t;
}

/// `TR_NOCACHE=1` disables D2 and G6, so benchmarks measure the full per-frame
/// work instead of a cached scene. Also used to A/B the optimisations.
fn no_cache() -> bool {
    static NO_CACHE: OnceLock<bool> = OnceLock::new();
    *NO_CACHE.get_or_init(|| std::env::var_os("TR_NOCACHE").is_some())
}

/// `TR_PAN=<tiles/frame>` pans the camera deterministically. Benchmarks use it to
/// measure the moving-camera path, where the static-scene caches (D2/G6) mostly miss.
fn pan_step() -> Option<f32> {
    static PAN: OnceLock<Option<f32>> = OnceLock::new();
    *PAN.get_or_init(|| {
        std::env::var("TR_PAN")
            .ok()
            .and_then(|v| v.parse::<f32>().ok())
    })
}

/// Static overlay text, built once instead of every frame (G4).
#[derive(Default)]
struct DebugCache {
    ready: bool,
    gpu_line: String,
    world_line: String,
}

/// Accumulates per-phase numbers for `TR_STATS` and prints them every 120 frames.
#[derive(Default)]
struct StatsAcc {
    frames: u32,
    light_ms: f64,
    draw_ms: f64,
    visible: u64,
    walls: u64,
    walls_culled: u64,
    dark_cells: u64,
    dark_rects: u64,
    chunks_skipped: u64,
    instances: u64,
    reused: u64,
}

impl StatsAcc {
    fn add(&mut self, s: &TileDrawStats) {
        self.frames += 1;
        self.light_ms += s.light_ms as f64;
        self.draw_ms += s.draw_ms as f64;
        self.visible += s.visible as u64;
        self.walls += s.walls as u64;
        self.walls_culled += s.walls_culled as u64;
        self.dark_cells += s.dark_cells as u64;
        self.dark_rects += s.dark_rects as u64;
        self.chunks_skipped += s.chunks_skipped as u64;
        self.instances += s.instances as u64;
        self.reused += u64::from(s.world_reused);
    }

    fn log_and_reset(&mut self) {
        let n = self.frames.max(1) as f64;
        log::info!(
            "TR_STATS frames={} light={:.2}ms draw={:.2}ms reused={}% visible={} walls={} culled={} dark_cells={} rects={} chunks_skipped={} instances={}",
            self.frames,
            self.light_ms / n,
            self.draw_ms / n,
            self.reused * 100 / self.frames.max(1) as u64,
            self.visible / self.frames.max(1) as u64,
            self.walls / self.frames.max(1) as u64,
            self.walls_culled / self.frames.max(1) as u64,
            self.dark_cells / self.frames.max(1) as u64,
            self.dark_rects / self.frames.max(1) as u64,
            self.chunks_skipped / self.frames.max(1) as u64,
            self.instances / self.frames.max(1) as u64,
        );
        *self = Self::default();
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }

        let mut attrs = Window::default_attributes()
            .with_title("Terraria-Rust")
            .with_inner_size(PhysicalSize::new(1600, 900));
        // Window/taskbar icon, generated from the game logo by `build.rs`.
        if let Ok(img) = image::load_from_memory(include_bytes!(concat!(
            env!("OUT_DIR"),
            "/icon.png"
        ))) {
            let rgba = img.to_rgba8();
            if let Ok(icon) = winit::window::Icon::from_rgba(
                rgba.clone().into_raw(),
                rgba.width(),
                rgba.height(),
            ) {
                attrs = attrs.with_window_icon(Some(icon));
            }
        }
        let window = Arc::new(event_loop.create_window(attrs).expect("create window"));

        let gpu = pollster::block_on(Gpu::new(window.clone())).expect("gpu init");
        let atlas = self.atlas.as_ref().expect("atlas");
        let renderer = SpriteRenderer::new(&gpu, atlas, 8);

        let fmt = gpu.config.format;
        let info = gpu.adapter.get_info();
        log::info!("renderer: {} [{:?}] | {:?}", info.name, info.backend, fmt);

        self.camera = Vec2::new(
            self.world.width as f32 * TILE_PX * 0.5,
            self.world.spawn_y as f32 * TILE_PX,
        );
        if let Ok(cam) = std::env::var("TR_CAM") {
            let mut it = cam.split(',').map(|v| v.trim().parse::<f32>());
            if let (Some(Ok(x)), Some(Ok(y))) = (it.next(), it.next()) {
                self.camera = Vec2::new(x * TILE_PX, y * TILE_PX);
            }
        }
        self.zoom = self.settings.zoom;
        if let Ok(z) = std::env::var("TR_ZOOM") {
            if let Ok(z) = z.parse::<f32>() {
                self.zoom = z.clamp(0.25, 8.0);
                self.zoom_pinned = true;
            }
        }
        if self.follow_player {
            if let Some(p) = &self.player {
                let (cx, cy) = p.center();
                self.camera = Vec2::new(cx, cy);
            }
        }
        self.last_frame = Instant::now();

        self.gpu = Some(gpu);
        self.renderer = Some(renderer);
        self.window = Some(window);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                self.save_world();
                event_loop.exit();
            }
            WindowEvent::Resized(size) => {
                if let Some(gpu) = self.gpu.as_mut() {
                    gpu.resize(size);
                }
            }
            WindowEvent::RedrawRequested => {
                self.frame();
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if let PhysicalKey::Code(code) = event.physical_key {
                    if event.state == ElementState::Pressed {
                        self.keys.insert(code);
                        if self.menu.is_some() {
                            if let Some(text) = event.text.as_ref() {
                                if let Some(m) = self.menu.as_mut() {
                                    for ch in text.chars() {
                                        if !ch.is_control() {
                                            m.character(ch);
                                        }
                                    }
                                }
                            }
                            let action = self
                                .menu
                                .as_mut()
                                .map(|m| m.key(code))
                                .unwrap_or(menu::Action::None);
                            self.apply_menu_action(action, event_loop);
                            return;
                        }
                        match code {
                            KeyCode::Escape => {
                                // The original opens the inventory on Escape; the
                                // world is saved on window close and with F9.
                                self.inventory_open = !self.inventory_open;
                            }
                            KeyCode::F1 => self.debug = !self.debug,
                            KeyCode::F2 => {
                                self.light_mode = self.light_mode.next();
                                self.world_key = None;
                                log::info!("light mode: {}", self.light_mode.label());
                            }
                            KeyCode::F5 => {
                                if let Some(p) = self.player.as_mut() {
                                    p.respawn();
                                }
                                if self.follow_player {
                                    if let Some(p) = &self.player {
                                        let (cx, cy) = p.center();
                                        self.camera = Vec2::new(cx, cy);
                                    }
                                } else {
                                    self.camera = Vec2::new(
                                        self.world.width as f32 * TILE_PX * 0.5,
                                        self.world.spawn_y as f32 * TILE_PX,
                                    );
                                }
                                self.zoom = self.settings.zoom;
                                self.world_key = None;
                            }
                            KeyCode::F9 => self.save_world(),
                            KeyCode::KeyM => {
                                if let Some(m) = self.map.as_mut() {
                                    m.open = !m.open;
                                }
                            }
                            KeyCode::KeyB => {
                                if !self
                                    .npcs
                                    .iter()
                                    .any(|n| n.kind == NpcKind::KingSlime && n.hp > 0)
                                {
                                    if let Some(p) = &self.player {
                                        let xt = ((p.x / TILE_PX) as i32 + 12)
                                            .clamp(2, self.world.width - 3);
                                        if let Some(n) = Npc::spawn_king_slime(&self.world, xt) {
                                            log::info!("King Slime spawned");
                                            self.npcs.push(n);
                                        }
                                    }
                                }
                            }
                            KeyCode::Digit1 => self.selected = 0,
                            KeyCode::Digit2 => self.selected = 1,
                            KeyCode::Digit3 => self.selected = 2,
                            KeyCode::Digit4 => self.selected = 3,
                            KeyCode::Digit5 => self.selected = 4,
                            KeyCode::Digit6 => self.selected = 5,
                            KeyCode::Digit7 => self.selected = 6,
                            KeyCode::Digit8 => self.selected = 7,
                            KeyCode::Digit9 => self.selected = 8,
                            KeyCode::Digit0 => self.selected = 9,
                            _ => {}
                        }
                    } else {
                        self.keys.remove(&code);
                    }
                }
            }
            WindowEvent::MouseWheel { delta, .. } => {
                // Vanilla: the wheel scrolls the hotbar (`HotbarPlus`/`HotbarMinus`);
                // zoom is only changed by the View Zoom keys or the settings screen.
                let d = match delta {
                    MouseScrollDelta::LineDelta(_, y) => y,
                    MouseScrollDelta::PixelDelta(p) => p.y as f32 / 60.0,
                };
                let step = d.round() as i32;
                if step != 0 && self.menu.is_none() && !self.inventory_open {
                    self.selected = (self.selected as i32 + step).rem_euclid(HOTBAR as i32) as usize;
                }
            }
            WindowEvent::MouseInput { state, button, .. } => {
                let pressed = state == ElementState::Pressed;
                match button {
                    MouseButton::Middle => self.mouse_down = pressed,
                    MouseButton::Left => {
                        if self.menu.is_some() {
                            if pressed {
                                let action = self
                                    .menu
                                    .as_mut()
                                    .map(|m| {
                                        m.update_hover(self.mouse_pos);
                                        m.click()
                                    })
                                    .unwrap_or(menu::Action::None);
                                self.apply_menu_action(action, event_loop);
                            }
                            return;
                        }
                        self.mine_down = pressed;
                        if pressed {
                            self.ui_left_press = true;
                        }
                    }
                    MouseButton::Right => {
                        self.place_down = pressed;
                        if pressed {
                            self.ui_right_press = true;
                        }
                    }
                    _ => {}
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                let p = Vec2::new(position.x as f32, position.y as f32);
                self.mouse_pos = p;
                if let Some(m) = self.menu.as_mut() {
                    m.update_hover(p);
                }
                if self.mouse_down {
                    let delta = p - self.last_mouse;
                    self.camera -= delta / self.zoom;
                }
                self.last_mouse = p;
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(w) = &self.window {
            w.request_redraw();
        }
    }
}

impl App {
    fn update_player(&mut self, dt: f32) {
        // Deterministic held input for screenshots (`TR_HOLD=right,jump`).
        static HOLD: std::sync::OnceLock<(bool, bool)> = std::sync::OnceLock::new();
        let (hold_right, hold_jump) = *HOLD.get_or_init(|| {
            let v = std::env::var("TR_HOLD").unwrap_or_default().to_ascii_lowercase();
            (v.contains("right"), v.contains("jump"))
        });
        let input = PlayerInput {
            left: self.keys.contains(&KeyCode::KeyA) || self.keys.contains(&KeyCode::ArrowLeft),
            right: hold_right
                || self.keys.contains(&KeyCode::KeyD)
                || self.keys.contains(&KeyCode::ArrowRight),
            jump: hold_jump
                || self.keys.contains(&KeyCode::Space)
                || self.keys.contains(&KeyCode::KeyW)
                || self.keys.contains(&KeyCode::ArrowUp),
            down: self.keys.contains(&KeyCode::KeyS) || self.keys.contains(&KeyCode::ArrowDown),
        };
        if let Some(p) = self.player.as_mut() {
            let step = 1.0 / 60.0;
            let mut acc = self.player_accum + dt;
            let mut steps = 0;
            while acc >= step && steps < 8 {
                p.update(&self.world, input);
                acc -= step;
                steps += 1;
            }
            if acc > step * 8.0 {
                acc = 0.0;
            }
            self.player_accum = acc;
            if self.follow_player {
                // Follow the interpolated position so the camera is smooth between
                // the 60 Hz simulation steps.
                let alpha = (acc / step).clamp(0.0, 1.0);
                let render = p.interpolated(alpha);
                let (cx, cy) = render.center();
                self.camera = self
                    .camera
                    .lerp(Vec2::new(cx, cy), 1.0 - (-dt * 12.0).exp());
            }
        }
    }

    /// `Main.DoUpdate_AnimateDiscoRGB`: step the shared disco-ball colour cycle.
    ///
    /// The original runs its six `if`s sequentially, so a style that completes on
    /// this tick also advances the next style in the same tick; that cascade is kept.
    fn advance_disco(&mut self) {
        const NUM: i32 = 7;
        if self.disco_style == 0 {
            self.disco[1] = (self.disco[1] as i32 + NUM).min(255) as u8;
            if self.disco[1] >= 255 {
                self.disco[1] = 255;
                self.disco_style += 1;
            }
        }
        if self.disco_style == 1 {
            self.disco[0] = (self.disco[0] as i32 - NUM).max(0) as u8;
            if self.disco[0] <= 0 {
                self.disco[0] = 0;
                self.disco_style += 1;
            }
        }
        if self.disco_style == 2 {
            self.disco[2] = (self.disco[2] as i32 + NUM).min(255) as u8;
            if self.disco[2] >= 255 {
                self.disco[2] = 255;
                self.disco_style += 1;
            }
        }
        if self.disco_style == 3 {
            self.disco[1] = (self.disco[1] as i32 - NUM).max(0) as u8;
            if self.disco[1] <= 0 {
                self.disco[1] = 0;
                self.disco_style += 1;
            }
        }
        if self.disco_style == 4 {
            self.disco[0] = (self.disco[0] as i32 + NUM).min(255) as u8;
            if self.disco[0] >= 255 {
                self.disco[0] = 255;
                self.disco_style += 1;
            }
        }
        if self.disco_style == 5 {
            self.disco[2] = (self.disco[2] as i32 - NUM).max(0) as u8;
            if self.disco[2] <= 0 {
                self.disco[2] = 0;
                self.disco_style = 0;
            }
        }
    }

    /// Fixed 60 Hz liquid stepping; `Liquid.UpdateLiquid` itself runs every second
    /// tick, matching the original 30 Hz rate.
    fn update_liquids(&mut self, dt: f32) {
        let step = 1.0 / 60.0;
        let mut acc = self.liquid_accum + dt;
        let mut steps = 0;
        while acc >= step && steps < 8 {
            self.liquid_tick = self.liquid_tick.wrapping_add(1);
            if self.liquid_tick % 2 == 0 {
                self.liquids.update(&mut self.world);
            }
            acc -= step;
            steps += 1;
        }
        self.liquid_accum = acc.min(step * 8.0);
    }

    /// Fixed 60 Hz `WaterfallManager.UpdateFrame` / `FindWaterfalls`, matching the
    /// original's per-update tick and 30-tick rescan cadence.
    fn update_waterfalls(&mut self, dt: f32, screen: Vec2) {
        let step = 1.0 / 60.0;
        let mut acc = self.waterfall_accum + dt.min(0.25);
        let mut steps = 0;
        while acc >= step && steps < 8 {
            self.waterfall.update_tick();
            acc -= step;
            steps += 1;
        }
        self.waterfall_accum = acc.min(step * 8.0);
        if steps == 0 {
            return;
        }
        let half = screen * 0.5 / self.zoom;
        let view = [
            ((self.camera.x - half.x) / TILE_PX).floor() as i32 - 1,
            ((self.camera.y - half.y) / TILE_PX).floor() as i32 - 1,
            ((self.camera.x + half.x) / TILE_PX).ceil() as i32 + 1,
            ((self.camera.y + half.y) / TILE_PX).ceil() as i32 + 1,
        ];
        self.waterfall
            .find_waterfalls(&self.world, view, false);
    }

    /// `Main.GetPreferredBGStyleForPlayer`: the surface background style wanted by
    /// the biome under the screen centre. Supported styles: 0 forest, 2 desert,
    /// 4 ocean, 7 snow. `SceneMetrics` zones are approximated with the same
    /// camera-column tests the water style uses (beach before desert before snow).
    fn preferred_bg_style(&self) -> i32 {
        if is_ocean_biome(&self.world, self.camera) {
            4
        } else if is_desert_biome(&self.world, self.camera) {
            2
        } else if is_snow_biome(&self.world, self.camera) {
            7
        } else {
            0
        }
    }

    /// `Main.DrawBG` (the cross-fade half): `UpdateBGVisibility_BackLayer` +
    /// `UpdateBGVisibility_FrontLayer`, then the style/delay timers.
    ///
    /// `backgroundLayerTransitionSpeed = 0.05` per drawn frame, so a full
    /// transition takes 20 frames once the delay elapses.
    fn update_bg_visibility(&mut self) {
        let preferred = self.preferred_bg_style();
        let speed = if self.bg_instant > 0 {
            // `instantBGTransitionCounter`: adopt the loaded biome immediately.
            self.bg_instant -= 1;
            self.bg_style = preferred;
            1.0
        } else {
            advance_bg_style(&mut self.bg_style, &mut self.bg_delay, preferred);
            0.05 // `Main.backgroundLayerTransitionSpeed`
        };
        advance_bg_alphas(
            &mut self.bg_alpha_front,
            &mut self.bg_alpha_far,
            self.bg_style,
            speed,
        );
    }

    /// Rain runs on its own 60 Hz accumulator; drops are simulated at the original
    /// per-tick velocity regardless of frame rate.
    fn update_rain(&mut self, dt: f32, screen: Vec2) {
        let snow = is_snow_biome(&self.world, self.camera);
        self.rain
            .update(&self.world, self.camera, screen, self.zoom, dt, snow);
    }

    fn update_npcs(&mut self, dt: f32) {
        self.attack_cd = (self.attack_cd - dt).max(0.0);
        self.npc_timer -= dt;
        // Fixed 60 Hz NPC simulation, like NPC.cs. Updating per rendered frame made
        // enemies move faster on high-refresh displays.
        let step = 1.0 / 60.0;
        let mut acc = self.npc_accum + dt.min(0.25);
        let mut steps = 0;
        while acc >= step && steps < 8 {
            let world = &self.world;
            if let Some(player) = self.player.as_mut() {
                for n in self.npcs.iter_mut() {
                    n.update(world, player, step);
                }
            }
            acc -= step;
            steps += 1;
        }
        self.npc_accum = acc.min(step * 8.0);
        let dead: Vec<(NpcKind, f32, f32)> = self
            .npcs
            .iter()
            .filter(|n| n.hp <= 0)
            .map(|n| {
                let (cx, cy) = n.center();
                (n.kind, cx, cy)
            })
            .collect();
        self.npcs.retain(|n| n.hp > 0);
        for (kind, x, y) in dead {
            self.spawn_coins(kind, x, y);
        }
        // `AI_015_KingSlime`: landing bosses request a blue slime minion.
        let mut minions = self
            .npcs
            .iter()
            .filter(|n| n.kind == NpcKind::Slime)
            .count();
        let mut spawned = Vec::new();
        for n in self.npcs.iter_mut() {
            if n.wants_minion {
                n.wants_minion = false;
                if minions < 4 {
                    let xt = ((n.x / TILE_PX) as i32 + n.facing * 2).clamp(2, self.world.width - 3);
                    if let Some(m) = Npc::spawn_slime(&self.world, xt) {
                        spawned.push(m);
                        minions += 1;
                    }
                }
            }
        }
        self.npcs.extend(spawned);
        if let Some(player) = self.player.as_mut() {
            if player.hp <= 0 {
                log::info!("player died; respawning at the world spawn");
                player.respawn();
                self.npcs.clear();
                self.npc_timer = 4.0;
                if self.follow_player {
                    let (cx, cy) = player.center();
                    self.camera = Vec2::new(cx, cy);
                }
            }
        }
        // `Spawner.SpawnNPC()`: one ambient spawn attempt per 60 Hz tick
        // (NPC.cs:289-306). The rate, caps, area scan and on-screen check are
        // the ported pipeline in `spawner.rs`.
        if self.npcs.len() < 200 && self.npc_timer <= 0.0 {
            let spawned = if let Some(player) = &self.player {
                let (pcx, pcy) = player.center();
                // `player.nearbyActiveNPCs` (NPC.cs:77902-77922): enemy NPCs
                // within the screen-size rect around the player, counted as
                // one slot each (`npcSlots == 1` for the implemented kinds).
                let nearby = self
                    .npcs
                    .iter()
                    .filter(|n| {
                        let (cx, cy) = n.center();
                        (cx - pcx).abs() < spawner::S_WIDTH as f32 * 0.5
                            && (cy - pcy).abs() < spawner::S_HEIGHT as f32 * 0.5
                    })
                    .count() as f32;
                let ctx = spawner::SpawnContext {
                    world: &self.world,
                    player_center: (pcx, pcy),
                    player_pos: (player.x, player.y),
                    nearby_active_npcs: nearby,
                    town_npcs: 0, // no town NPCs yet (M4-TOWN)
                    day_time: self.world.day_time,
                };
                self.npc_rng = self
                    .npc_rng
                    .wrapping_mul(6_364_136_223_846_793_005)
                    .wrapping_add(1_442_695_040_888_963_407);
                let mut rng = spawner::FastRandom::new(self.npc_rng);
                spawner::try_spawn_an_npc(&ctx, &mut rng)
            } else {
                None
            };
            if let Some(n) = spawned {
                self.npcs.push(n);
            }
            self.npc_timer = 1.0 / 60.0;
        }
    }

    fn update_editing(&mut self, dt: f32, screen: Vec2) {
        self.swing = (self.swing - dt).max(0.0);
        if let Some((x, y)) = self.dev_break.take() {
            self.break_tile(x, y);
        }
        for (x, y, kind) in std::mem::take(&mut self.dev_place) {
            if edit::place_tile(&mut self.world, x, y, kind) {
                self.liquids.wake_around(&self.world, x, y, 3);
                self.world_key = None;
            }
        }
        self.place_cooldown = (self.place_cooldown - dt).max(0.0);
        let m = self.mouse_pos;
        let hud_top = screen.y - 96.0;
        if m.x < 0.0
            || m.y < 0.0
            || m.x > screen.x
            || m.y > screen.y
            || (m.y > hud_top && m.x < 300.0)
            || (self.debug && m.x < 780.0 && m.y < 340.0)
        {
            self.edit_target = None;
            self.mine_target = None;
            self.mine_timer = 0.0;
            return;
        }
        let world_pos = self.camera + (m - screen * 0.5) / self.zoom;
        let tx = (world_pos.x / TILE_PX).floor() as i32;
        let ty = (world_pos.y / TILE_PX).floor() as i32;
        if !self.world.in_bounds(tx, ty) {
            self.edit_target = None;
            return;
        }
        let Some(player) = &self.player else {
            self.edit_target = None;
            return;
        };
        let (pcx, pcy) = player.center();
        let dx = tx as f32 * TILE_PX + 8.0 - pcx;
        let dy = ty as f32 * TILE_PX + 8.0 - pcy;
        let reach = 6.0 * TILE_PX;
        if dx * dx + dy * dy > reach * reach {
            self.edit_target = None;
            self.mine_target = None;
            self.mine_timer = 0.0;
            return;
        }
        self.edit_target = Some((tx, ty));
        let held = self.inventory[self.selected];
        let held_weapon = held.count > 0 && registry::is_weapon(held.kind);
        if self.mine_down && held_weapon && !self.inventory_open {
            // Weapons use LMB toward the cursor instead of mining.
            if self.attack_cd <= 0.0 {
                let (pcx, pcy) = player.center();
                let dir = (world_pos - Vec2::new(pcx, pcy)).normalize_or_zero();
                if dir != Vec2::ZERO {
                    let spawn = Vec2::new(pcx, pcy) + dir * 10.0;
                    self.projectiles.push(Projectile::shuriken(
                        spawn.x - 3.0,
                        spawn.y - 3.0,
                        dir.x * 11.0,
                        dir.y * 11.0,
                    ));
                    self.inventory[self.selected].count =
                        self.inventory[self.selected].count.saturating_sub(1);
                    self.attack_cd = 0.25;
                    self.swing = SWING_TIME;
                }
            }
            return;
        }
        // Melee: LMB over an NPC within four tiles hits it instead of mining.
        if self.mine_down && self.attack_cd <= 0.0 && !self.inventory_open {
            let ax = world_pos.x;
            let ay = world_pos.y;
            let reach2 = 4.0 * TILE_PX;
            if (ax - pcx).powi(2) + (ay - pcy).powi(2) <= reach2 * reach2 {
                if let Some(n) = self.npcs.iter_mut().find(|n| n.contains(ax, ay)) {
                    let dmg = if held_weapon { registry::damage(held.kind) } else { 15 };
                    // `NPC.StrikeNPC` plays `this.HitSound` at `base.position`
                    // (`NPC.cs:81546`) so the hit is placed at the NPC corner.
                    let (nx, ny) = (n.x, n.y);
                    n.hurt(dmg, pcx);
                    self.audio.play_at("NPC_Hit_1", nx, ny, 1.0);
                    self.attack_cd = 0.3;
                    self.mine_target = None;
                    self.mine_timer = 0.0;
                    self.swing = SWING_TIME;
                    return;
                }
            }
        }
        let Some(tile) = self.world.tile(tx, ty).copied() else {
            self.edit_target = None;
            return;
        };
        let important = self
            .world
            .frame_important
            .get(tile.kind as usize)
            .copied()
            .unwrap_or(false)
            && tile.kind != 80; // cacti have fixed frames but are mineable
        let held_wall = held.count > 0 && registry::place_wall(held.kind).is_some();

        if self.mine_down && tile.active() && !important && !self.inventory_open {
            if self.swing <= 0.0 {
                self.swing = SWING_TIME;
            }
            if self.mine_target == Some((tx, ty)) {
                self.mine_timer += dt;
            } else {
                self.mine_target = Some((tx, ty));
                self.mine_timer = 0.0;
            }
            let hardness = if tile.kind == 1 || (6..=9).contains(&tile.kind) {
                0.55
            } else {
                0.25
            };
            if self.mine_timer >= hardness {
                self.break_tile(tx, ty);
                self.mine_target = None;
                self.mine_timer = 0.0;
            }
        } else if self.mine_down && !tile.active() && tile.wall != 0 && !self.inventory_open {
            // No block in the cell: mining it removes the exposed wall.
            if self.swing <= 0.0 {
                self.swing = SWING_TIME;
            }
            if self.mine_target == Some((tx, ty)) {
                self.mine_timer += dt;
            } else {
                self.mine_target = Some((tx, ty));
                self.mine_timer = 0.0;
            }
            let hardness = if tile.wall == 1 { 0.7 } else { 0.35 };
            if self.mine_timer >= hardness {
                self.break_wall(tx, ty);
                self.mine_target = None;
                self.mine_timer = 0.0;
            }
        } else {
            self.mine_target = None;
            self.mine_timer = 0.0;
        }

        if self.place_down && self.place_cooldown <= 0.0 && !self.inventory_open {
            if held_wall {
                if self.can_place_wall(tx, ty) {
                    self.place_wall_item(tx, ty, registry::place_wall(held.kind).unwrap_or(1));
                    self.place_cooldown = 0.12;
                    self.swing = SWING_TIME;
                }
            } else if !held_weapon && self.can_place(tx, ty) {
                let Some(kind) = registry::place_tile(held.kind) else {
                    return;
                };
                self.place_tile(tx, ty, kind);
                self.place_cooldown = 0.12;
                self.swing = SWING_TIME;
            }
        }
    }

    fn break_wall(&mut self, tx: i32, ty: i32) {
        let Some(wall) = edit::break_wall(&mut self.world, tx, ty) else {
            return;
        };
        if let Some(id) = registry::by_wall(wall) {
            self.spawn_item_drop(id, tx, ty);
        }
        self.audio
            .play_at("Dig_2", tx as f32 * TILE_PX, ty as f32 * TILE_PX, 1.0);
        self.request_edit(tx, ty, 2, 0);
        self.world_key = None;
    }

    fn can_place_wall(&self, tx: i32, ty: i32) -> bool {
        let Some(t) = self.world.tile(tx, ty) else {
            return false;
        };
        if t.wall != 0
            || self.inventory[self.selected].count == 0
            || registry::is_weapon(self.inventory[self.selected].kind)
        {
            return false;
        }
        // A neighbouring wall or block (including one in the cell itself) supports it.
        if t.active() && (t.kind as usize) < td::TILE_COUNT && td::TILE_SOLID.has(t.kind) {
            return true;
        }
        [(0, -1), (0, 1), (-1, 0), (1, 0)].iter().any(|&(dx, dy)| {
            self.world
                .tile(tx + dx, ty + dy)
                .is_some_and(|n| n.wall != 0)
        })
    }

    fn place_wall_item(&mut self, tx: i32, ty: i32, wall: u16) {
        if edit::place_wall(&mut self.world, tx, ty, wall) {
            self.inventory[self.selected].count =
                self.inventory[self.selected].count.saturating_sub(1);
            self.audio
                .play_at("Dig_0", tx as f32 * TILE_PX, ty as f32 * TILE_PX, 1.0);
            self.request_edit(tx, ty, 3, wall);
            self.world_key = None;
        }
    }

    /// Item physics and pickup on a fixed 60 Hz step (`Item.Update`).
    fn is_client(&self) -> bool {
        matches!(self.net, NetRole::Client(_))
    }

    fn update_items(&mut self, dt: f32) {
        if self.is_client() {
            // The host owns the drops: ask it to collect whatever lies under our feet
            // and hide those locally until the next authoritative snapshot.
            let Some(pc) = self
                .player
                .as_ref()
                .map(|p| Vec2::new(p.center().0, p.center().1))
            else {
                return;
            };
            let ids: Vec<u32> = self
                .items
                .iter()
                .enumerate()
                .filter(|(_, it)| (it.center() - pc).length() < 26.0)
                .map(|(i, _)| i as u32)
                .collect();
            if let NetRole::Client(client) = &self.net {
                for id in ids {
                    client.send_pickup(id);
                }
            }
            self.items.retain(|it| (it.center() - pc).length() >= 26.0);
            return;
        }
        let step = 1.0 / 60.0;
        let mut acc = self.item_accum + dt.min(0.25);
        let mut steps = 0;
        while acc >= step && steps < 8 {
            let player_center = self
                .player
                .as_ref()
                .map(|p| Vec2::new(p.center().0, p.center().1));
            let mut i = 0;
            while i < self.items.len() {
                let mut drop = self.items[i].clone();
                if drop.update(&self.world, player_center, step) {
                    if registry::is_coin(drop.kind) {
                        self.coins = self
                            .coins
                            .saturating_add(registry::coin_value(drop.kind) * drop.count as u64);
                        let c = drop.center();
                        self.audio.play_at("Coin_0", c.x, c.y, 1.0);
                        self.items.swap_remove(i);
                        continue;
                    }
                    if let Some(slot) = slot_for_pickup(&mut self.inventory, drop.kind) {
                        self.inventory[slot].count =
                            self.inventory[slot].count.saturating_add(drop.count);
                        let c = drop.center();
                        self.audio.play_at("Item_1", c.x, c.y, 1.0);
                        self.items.swap_remove(i);
                        continue;
                    }
                }
                self.items[i] = drop;
                i += 1;
            }
            acc -= step;
            steps += 1;
        }
        self.item_accum = acc.min(step * 8.0);
    }

    /// Fixed 60 Hz projectile stepping. Damage lands through `Npc::hurt`, so the
    /// hit cooldown and knockback are shared with melee.
    fn update_projectiles(&mut self, dt: f32) {
        if self.dev_throw {
            if let Some(p) = &self.player {
                let (cx, cy) = p.center();
                self.projectiles
                    .push(Projectile::shuriken(cx - 3.0, cy - 3.0, 11.0, 0.0));
            }
            self.dev_throw = false;
        }
        let step = 1.0 / 60.0;
        let mut acc = self.projectile_accum + dt.min(0.25);
        let mut steps = 0;
        while acc >= step && steps < 8 {
            let world = &self.world;
            let npcs = &mut self.npcs;
            for p in &mut self.projectiles {
                p.update(world, npcs, step);
            }
            self.projectiles.retain(|p| p.alive);
            acc -= step;
            steps += 1;
        }
        self.projectile_accum = acc.min(step * 8.0);
    }

    /// `Item.NewItem` coin drops: a few copper from small enemies, gold from the boss.
    fn spawn_coins(&mut self, kind: NpcKind, x: f32, y: f32) {
        let roll = rand01(&mut self.item_rng);
        let (denom, count) = match kind {
            NpcKind::KingSlime => (2u16, 3 + (roll * 4.0) as u32),
            NpcKind::Zombie => {
                if roll < 0.2 {
                    (1, 1)
                } else {
                    (0, 2 + (roll * 30.0) as u32 % 4)
                }
            }
            NpcKind::Slime => {
                if roll < 0.05 {
                    (1, 1)
                } else {
                    (0, 1 + (roll * 20.0) as u32 % 3)
                }
            }
            NpcKind::DemonEye => (0, 1 + (roll * 10.0) as u32 % 2),
        };
        let vx = (rand01(&mut self.item_rng) * 2.0 - 1.0) * 1.2;
        let vy = -1.5 - rand01(&mut self.item_rng);
        let phase = rand01(&mut self.item_rng) * std::f32::consts::TAU;
        self.items.push(ItemDrop::new(
            registry::coin_id(denom as usize),
            count,
            x - 7.0,
            y - 7.0,
            vx,
            vy,
            phase,
        ));
    }

    /// Spawns the drop left by a broken tile, with a small random pop like `Item.NewItem`.
    fn spawn_item_drop(&mut self, kind: u16, tx: i32, ty: i32) {
        let r1 = rand01(&mut self.item_rng);
        let r2 = rand01(&mut self.item_rng);
        let r3 = rand01(&mut self.item_rng);
        let drop = ItemDrop::new(
            kind,
            1,
            tx as f32 * TILE_PX + 1.0,
            ty as f32 * TILE_PX,
            (r1 * 2.0 - 1.0) * 1.4,
            -2.2 - r2 * 1.2,
            r3 * std::f32::consts::TAU,
        );
        self.items.push(drop);
        if self.items.len() > 300 {
            self.items.remove(0);
        }
    }

    fn break_tile(&mut self, tx: i32, ty: i32) {
        let Some(kind) = edit::break_tile(&mut self.world, tx, ty) else {
            return;
        };
        let Some(id) = registry::tile_drop(kind) else {
            self.request_edit(tx, ty, 0, 0);
            return;
        };
        self.spawn_item_drop(id, tx, ty);
        self.audio
            .play_at("Dig_1", tx as f32 * TILE_PX, ty as f32 * TILE_PX, 1.0);
        self.request_edit(tx, ty, 0, 0);
        self.liquids.wake_around(&self.world, tx, ty, 3);
        self.world_key = None;
    }

    fn can_place(&self, tx: i32, ty: i32) -> bool {
        let Some(t) = self.world.tile(tx, ty) else {
            return false;
        };
        if t.active()
            || self.inventory[self.selected].count == 0
            || registry::is_weapon(self.inventory[self.selected].kind)
            || self.inventory[self.selected].kind == 80
        {
            return false;
        }
        if let Some(p) = &self.player {
            let (x0, y0) = (tx as f32 * TILE_PX, ty as f32 * TILE_PX);
            if x0 < p.x + crate::player::WIDTH
                && x0 + TILE_PX > p.x
                && y0 < p.y + crate::player::HEIGHT
                && y0 + TILE_PX > p.y
            {
                return false;
            }
        }
        [(0, -1), (0, 1), (-1, 0), (1, 0)].iter().any(|&(dx, dy)| {
            self.world.tile(tx + dx, ty + dy).is_some_and(|n| {
                n.wall != 0
                    || (n.active()
                        && (n.kind as usize) < td::TILE_COUNT
                        && td::TILE_SOLID.has(n.kind))
            })
        })
    }

    fn place_tile(&mut self, tx: i32, ty: i32, kind: u16) {
        if edit::place_tile(&mut self.world, tx, ty, kind) {
            self.inventory[self.selected].count =
                self.inventory[self.selected].count.saturating_sub(1);
            self.liquids.wake_around(&self.world, tx, ty, 3);
            self.audio
                .play_at("Dig_0", tx as f32 * TILE_PX, ty as f32 * TILE_PX, 1.0);
            self.request_edit(tx, ty, 1, kind);
            self.world_key = None;
        }
    }

    /// The slot under the cursor, using the same geometry as `draw_hud`.
    fn slot_at_mouse(&self) -> Option<usize> {
        let m = self.mouse_pos;
        if self.inventory_open {
            let s = 0.85;
            let i = ((m.x - 20.0) / (56.0 * s)).floor() as i32;
            let j = ((m.y - 20.0) / (56.0 * s)).floor() as i32;
            if !(0..10).contains(&i) || !(0..5).contains(&j) {
                return None;
            }
            let lx = m.x - 20.0 - i as f32 * 56.0 * s;
            let ly = m.y - 20.0 - j as f32 * 56.0 * s;
            if lx > 52.0 * s || ly > 52.0 * s {
                return None;
            }
            Some((i + j * 10) as usize)
        } else {
            let mut x = 20.0;
            for i in 0..HOTBAR {
                let sc = self.hotbar_scales[i].clamp(0.75, 1.0);
                let slot = 52.0 * sc;
                let y = 20.0 + 22.0 * (1.0 - sc);
                if m.x >= x && m.x <= x + slot && m.y >= y && m.y <= y + slot {
                    return Some(i);
                }
                x += slot + 4.0;
            }
            None
        }
    }

    /// `ItemSlot.Handle` (context 0) subset: left click takes, places or swaps a whole
    /// stack, right click takes half or places one; on the closed hotbar a left click
    /// selects the slot like the original.
    fn handle_inventory_clicks(&mut self) {
        let left = self.ui_left_press;
        let right = self.ui_right_press;
        self.ui_left_press = false;
        self.ui_right_press = false;
        if !left && !right {
            return;
        }
        let Some(slot) = self.slot_at_mouse() else {
            return;
        };
        if !self.inventory_open {
            if left {
                self.selected = slot;
            }
            return;
        }
        let mut held = self.mouse_item;
        let mut target = self.inventory[slot];
        if left {
            if held.count == 0 {
                held = target;
                target = EMPTY_SLOT;
            } else if target.count == 0 {
                target = held;
                held = EMPTY_SLOT;
            } else if target.kind == held.kind && target.count < MAX_STACK {
                let moved = (MAX_STACK - target.count).min(held.count);
                target.count += moved;
                held.count -= moved;
                if held.count == 0 {
                    held = EMPTY_SLOT;
                }
            } else {
                std::mem::swap(&mut held, &mut target);
            }
        } else if held.count == 0 {
            if target.count > 0 {
                let take = (target.count + 1) / 2;
                held = ItemSlot {
                    kind: target.kind,
                    name: target.name,
                    count: take,
                };
                target.count -= take;
                if target.count == 0 {
                    target = EMPTY_SLOT;
                }
            }
        } else if target.count == 0 {
            target = ItemSlot {
                kind: held.kind,
                name: held.name,
                count: 1,
            };
            held.count -= 1;
            if held.count == 0 {
                held = EMPTY_SLOT;
            }
        } else if target.kind == held.kind && target.count < MAX_STACK {
            target.count += 1;
            held.count -= 1;
            if held.count == 0 {
                held = EMPTY_SLOT;
            }
        }
        self.mouse_item = held;
        self.inventory[slot] = target;
    }

    /// Applies a menu action (start singleplayer, host on a port, join a server).
    fn apply_menu_action(&mut self, action: menu::Action, event_loop: &ActiveEventLoop) {
        use menu::Action;
        match action {
            Action::None => {}
            Action::Quit => {
                self.save_world();
                event_loop.exit();
            }
            Action::Singleplayer => {
                self.close_menu();
            }
            Action::Host { port, password } => {
                let map = compressed_world(&self.world);
                match net::Server::start(port, password, map) {
                    Ok(server) => {
                        log::info!("hosting on {}", server.addr());
                        self.net = NetRole::Host(server);
                        self.close_menu();
                    }
                    Err(e) => {
                        if let Some(m) = self.menu.as_mut() {
                            m.connecting = false;
                            m.error = Some(format!("host failed: {e}"));
                        }
                    }
                }
            }
            Action::Join { addr, password } => match net::Client::connect(
                &addr,
                self.player_name.clone(),
                password.clone(),
            ) {
                Ok(client) => {
                    self.net = NetRole::Client(client);
                    self.join_target = Some((addr, password));
                }
                Err(e) => {
                    if let Some(m) = self.menu.as_mut() {
                        m.connecting = false;
                        m.error = Some(format!("connect failed: {e}"));
                    }
                }
            },
        }
    }

    /// Leaves the menu and applies the settings it edited (`Main.SaveSettings`).
    fn close_menu(&mut self) {
        if let Some(m) = self.menu.take() {
            self.settings = m.settings;
        }
        self.apply_settings();
    }

    /// Applies the options the renderer reads right away.
    fn apply_settings(&mut self) {
        self.light_mode = tr_core::lighting::LightMode::ALL
            [(self.settings.light_mode as usize).min(3)];
        if !self.zoom_pinned {
            self.zoom = self.settings.zoom;
        }
        // The light cache and the world range depend on the mode.
        self.world_key = None;
    }

    /// Drains network events without borrowing `self.net` across handlers.
    fn poll_net(&mut self) {
        let mut server_events = Vec::new();
        let mut client_events = Vec::new();
        match &self.net {
            NetRole::Host(server) => {
                while let Ok(ev) = server.events.try_recv() {
                    server_events.push(ev);
                }
            }
            NetRole::Client(client) => {
                while let Ok(ev) = client.events.try_recv() {
                    client_events.push(ev);
                }
            }
            NetRole::Off => {}
        }
        let role = std::mem::replace(&mut self.net, NetRole::Off);
        if let NetRole::Host(server) = &role {
            for ev in server_events {
                self.host_event(server, ev);
            }
        }
        if let NetRole::Client(client) = &role {
            for ev in client_events {
                self.client_event(client, ev);
            }
        }
        self.net = role;
    }

    /// Host side: applies client requests and keeps the state broadcast.
    fn host_event(&mut self, server: &net::Server, ev: net::ServerEvent) {
        match ev {
            net::ServerEvent::Joined { id, name } => {
                log::info!("{name} joined (id {id})");
                server.welcome(id, self.world.time, self.world.day_time);
                for p in self.remote_players.values() {
                    server.send(id, &net_player(p));
                }
                self.remote_players.insert(
                    id,
                    net::PlayerNet {
                        id,
                        name,
                        ..Default::default()
                    },
                );
                broadcast_joined(server, id, &self.remote_players);
            }
            net::ServerEvent::Left { id } => {
                self.remote_players.remove(&id);
                broadcast_left(server, id);
            }
            net::ServerEvent::Player(p) => {
                self.remote_players.insert(p.id, p.clone());
                broadcast_player(server, &p);
            }
            net::ServerEvent::Edit { x, y, kind, tile } => {
                log::info!("net edit kind {kind} at {x},{y}");
                apply_edit(&mut self.world, x, y, kind, tile);
                self.world_key = None;
                server.broadcast(&net_edit(x, y, kind, tile));
            }
            net::ServerEvent::Pickup { id, drop } => {
                let center = self
                    .remote_players
                    .get(&id)
                    .map(|p| Vec2::new(p.x + player::WIDTH * 0.5, p.y + player::HEIGHT * 0.5));
                if let Some(center) = center {
                    if let Some(c) = self.items.get(drop as usize).map(|it| it.center()) {
                        if (c - center).length() < 60.0 {
                            self.items.remove(drop as usize);
                            server.broadcast(&net_items(&self.items));
                        }
                    }
                }
            }
        }
    }

    /// Client side: clock, edits, NPCs, drops and other players come from the host.
    fn client_event(&mut self, _client: &net::Client, ev: net::ClientEvent) {
        match ev {
            net::ClientEvent::Welcome { id } => {
                self.local_id = id;
                log::info!("joined with id {id}");
            }
            net::ClientEvent::World { data } => {
                if self.menu.is_some() {
                    if let Some((addr, pw)) = self.join_target.clone() {
                        relaunch_into_world(&data, &addr, &pw, &self.player_name);
                    }
                }
            }
            net::ClientEvent::Joined { id, name } => {
                self.remote_players.entry(id).or_insert_with(|| net::PlayerNet {
                    id,
                    name,
                    ..Default::default()
                });
            }
            net::ClientEvent::Left { id } => {
                self.remote_players.remove(&id);
            }
            net::ClientEvent::Player(p) => {
                self.remote_players.insert(p.id, p);
            }
            net::ClientEvent::Time { time, day } => {
                self.world.time = time;
                self.world.day_time = day;
            }
            net::ClientEvent::Edit { x, y, kind, tile } => {
                apply_edit(&mut self.world, x, y, kind, tile);
                self.world_key = None;
            }
            net::ClientEvent::Npcs(list) => {
                self.npcs = list.iter().filter_map(|n| npc_from_net(&self.world, n)).collect();
            }
            net::ClientEvent::Items(list) => {
                self.items = list
                    .iter()
                    .map(|it| {
                        let mut d =
                            item::ItemDrop::new(it.item, it.count, it.x, it.y, 0.0, 0.0, 0.0);
                        d.age = 60.0;
                        d
                    })
                    .collect();
            }
            net::ClientEvent::Disconnect { reason } => {
                log::warn!("disconnected: {reason}");
                self.net = NetRole::Off;
                if self.menu.is_none() {
                    let mut m = menu::Menu::new(self.player_name.clone());
                    m.error = Some(format!("Disconnected: {reason}"));
                    self.menu = Some(m);
                }
            }
        }
    }

    /// Sends our player (and, when hosting, the authoritative state) at 10 Hz.
    fn net_tick(&mut self, dt: f32) {
        if matches!(self.net, NetRole::Off) {
            return;
        }
        self.net_timer += dt;
        if self.net_timer < 0.1 {
            return;
        }
        self.net_timer = 0.0;
        let Some(p) = self.player.as_ref() else {
            return;
        };
        let me = net::PlayerNet {
            id: self.local_id,
            name: self.player_name.clone(),
            x: p.x,
            y: p.y,
            vx: p.vx,
            vy: p.vy,
            hp: p.hp,
            max_hp: p.max_hp,
            facing: p.facing,
            selected: self.selected as u16,
            on_ground: p.on_ground,
            walk_time: p.walk_time,
        };
        match &self.net {
            NetRole::Host(server) => {
                broadcast_player(server, &me);
                let npcs: Vec<net::NpcNet> = self
                    .npcs
                    .iter()
                    .enumerate()
                    .map(|(i, n)| net::NpcNet {
                        id: i as u32,
                        kind: n.kind.to_u16(),
                        x: n.x,
                        y: n.y,
                        vx: n.vx,
                        vy: n.vy,
                        hp: n.hp,
                        max_hp: n.max_hp,
                        facing: n.facing,
                        on_ground: n.on_ground,
                        anim: n.anim,
                        hurt_cd: n.hurt_cd,
                    })
                    .collect();
                server.broadcast(&net_npcs(npcs));
                server.broadcast(&net_items(&self.items));
                server.broadcast(&net::ServerToClient::Time {
                    time: self.world.time,
                    day: self.world.day_time,
                });
            }
            NetRole::Client(client) => client.send_player(&me),
            NetRole::Off => {}
        }
    }

    /// Client: asks the host to apply an edit (the host echoes it back to everyone).
    fn request_edit(&self, x: i32, y: i32, kind: u8, tile: u16) {
        match &self.net {
            NetRole::Host(server) => server.broadcast(&net_edit(x, y, kind, tile)),
            NetRole::Client(client) => client.send_edit(x, y, kind, tile),
            NetRole::Off => {}
        }
    }

    fn save_world(&self) {
        if self.world_path.extension().is_some_and(|e| e == "trwld") {
            match save::save(&self.world_path, &self.world) {
                Ok(()) => log::info!("saved {}", self.world_path.display()),
                Err(e) => log::warn!("save failed: {e:#}"),
            }
            if let Some(p) = &self.player {
                let inv =
                    std::array::from_fn(|i| (self.inventory[i].kind, self.inventory[i].count));
                if let Err(e) = persist::save(&self.world_path, p, inv, self.coins) {
                    log::warn!("player save failed: {e}");
                }
            }
        } else {
            log::info!("not saving: the loaded world is not a native .trwld");
        }
    }

    fn frame(&mut self) {
        let now = Instant::now();
        let dt = now
            .duration_since(self.last_frame)
            .as_secs_f32()
            .clamp(0.0, 0.1);
        self.last_frame = now;
        self.frame_count += 1;
        self.anim_time += dt;
        // `Main.DoUpdate_AnimateDiscoRGB`: the disco-ball colour cycle.
        self.advance_disco();
        let anim_time = self.anim_time;
        if !self.follow_player {
            self.pan_camera(dt);
        }
        if !time_frozen() {
            advance_time(&mut self.world, dt);
        }

        let screen = self
            .gpu
            .as_ref()
            .map(|g| Vec2::new(g.config.width as f32, g.config.height as f32))
            .unwrap_or(Vec2::new(1600.0, 900.0));

        // `Main.Camera.Center` + the volume sliders for `LegacySoundPlayer.PlaySound`.
        self.audio.set_listener(sound::Listener {
            camera_center: (self.camera.x, self.camera.y),
            sound_volume: self.settings.sound_volume,
            ambient_volume: self.settings.ambient_volume,
            quiet_ambient_sounds: false,
            game_menu: self.menu.is_some(),
        });

        if self.menu.is_none() {
            self.handle_inventory_clicks();
            for (i, scale) in self.hotbar_scales.iter_mut().enumerate() {
                let target = if i == self.selected { 1.0 } else { 0.75 };
                if *scale < target {
                    *scale = (*scale + 0.05).min(target);
                } else if *scale > target {
                    *scale = (*scale - 0.05).max(target);
                }
            }
            self.update_player(dt);
            let hp = self.player.as_ref().map_or(0, |p| p.hp);
            if hp < self.last_hp {
                self.audio.play("Player_Hit_0");
            }
            self.last_hp = hp;
            // `Main.UpdateAudio_DecideOnNewMusic` reads the scene centre, not the
            // player: underground once the view centre passes the world surface
            // (`Main.cs:12815`).
            let boss_music = self
                .npcs
                .iter()
                .any(|n| n.kind == NpcKind::KingSlime && n.hp > 0)
                .then_some(93); // MusicID.KingSlime
            self.music.update(audio::MusicEnv {
                music_volume: self.settings.music_volume,
                day: self.world.day_time,
                underground: self.camera.y
                    >= self.world.world_surface as f32 * TILE_PX + screen.y * 0.5,
                rain: self.rain.is_raining(),
                boss_music,
                clock: self.world.time,
            });
            if !self.is_client() {
                self.update_npcs(dt);
            }
            self.update_liquids(dt);
            self.update_waterfalls(dt, screen);
            self.update_rain(dt, screen);
            self.update_items(dt);
            self.update_projectiles(dt);
            self.update_editing(dt, screen);
            self.net_tick(dt);
            if let Some(m) = self.map.as_mut() {
                m.update(self.player.as_ref(), dt);
            }
        }
        // `TR_NET_EDIT=tx,ty`: one client edit request, used to verify the round trip.
        if self.is_client() && self.frame_count == 90 {
            if let Ok(v) = std::env::var("TR_NET_EDIT") {
                if let Some((sx, sy)) = v.split_once(',') {
                    if let (Ok(x), Ok(y)) = (sx.trim().parse::<i32>(), sy.trim().parse::<i32>()) {
                        self.request_edit(x, y, 0, 0);
                        log::info!("net edit requested at {x},{y}");
                    }
                }
            }
        }
        self.poll_net();
        // `Main.DrawBG`: advance the surface-background cross-fade once per drawn
        // frame, exactly where the original runs it (inside the draw pass).
        self.update_bg_visibility();
        // Render between fixed steps so the jump/fall is not a 60 Hz staircase on
        // high-refresh displays.
        let player_alpha = (self.player_accum / (1.0 / 60.0)).clamp(0.0, 1.0);
        let render_player = self.player.as_ref().map(|p| p.interpolated(player_alpha));
        let player_key = render_player.as_ref().map_or(0, |p| p.render_key());
        let npc_key = npc::render_key(&self.npcs);
        let mut stats = tr_render::sprite::RenderStats::default();
        let mut tile_stats = self.tile_stats;

        {
            let App {
                gpu,
                renderer,
                world,
                atlas,
                lookups,
                clouds,
                sky,
                bg_alpha_front,
                bg_alpha_far,
                stars,
                player_assets,
                player,
                npcs,
                npc_assets,
                inventory,
                selected,
                edit_target,
                camera,
                zoom,
                debug,
                frame_history,
                last_stats,
                frame_count,
                shot_path,
                shot_at,
                atlas_stats,
                light,
                scratch,
                debug_cache,
                light_mode,
                has_liquids,
                liquids,
                waterfall,
                rain,
                items,
                swing,
                projectiles,
                world_key,
                menu,
                settings,
                remote_players,
                map,
                ..
            } = self;

            if let (Some(gpu), Some(renderer)) = (gpu.as_ref(), renderer.as_mut()) {
                // `TR_DAY` overrides the time of day (0 = midnight, 1 = noon) so a
                // screenshot can be compared against the original at a fixed time.
                let override_day = std::env::var("TR_DAY")
                    .ok()
                    .and_then(|v| v.parse::<f32>().ok())
                    .map(|v| v.clamp(0.0, 1.0));
                let (sky_tint, _sky_light, is_day, clock_time) = match override_day {
                    Some(d) => {
                        let night = [0.075f32, 0.08, 0.10];
                        let day = [1.0f32, 1.0, 1.0];
                        let tint = [
                            night[0] + (day[0] - night[0]) * d,
                            night[1] + (day[1] - night[1]) * d,
                            night[2] + (day[2] - night[2]) * d,
                        ];
                        let is_day = d >= 0.5;
                        let clock = if is_day {
                            d as f64 * 54000.0
                        } else {
                            (1.0 - d as f64) * 32400.0
                        };
                        (tint, tile_color(tint), is_day, clock)
                    }
                    // `Main.SetBackColor` + `ApplyColorOfTheSkiesToTiles`.
                    None => {
                        // Quantise the daylight/sky colour to one-second steps: the sun
                        // still moves smoothly on the draw pass, but the light map and
                        // world range can be reused for a second.
                        let quantised = (world.time / 60.0).floor() * 60.0;
                        let c = color_of_the_skies(quantised, world.day_time);
                        (c, tile_color(c), world.day_time, world.time)
                    }
                };
                // `SetBackColor`: rain multiplies the sky (and therefore the surface
                // sunlight) by `1 - cloudAlpha * 0.9`.
                let rain_alpha = rain.visual_alpha();
                let rain_dim = (1.0 - rain_alpha * 0.9).max(0.1);
                let sky_tint = [
                    sky_tint[0] * rain_dim,
                    sky_tint[1] * rain_dim,
                    sky_tint[2] * rain_dim,
                ];
                let sky_light = tile_color(sky_tint);
                // `Main.DoUpdate_AnimateCursorColors`: mouseTextColor oscillates
                // 190..=255, changing by 1 per frame, driving light flicker.
                self.mouse_text_color = (self.mouse_text_color as i32 + self.mouse_text_color_dir as i32).clamp(190, 255) as u8;
                if self.mouse_text_color >= 255 {
                    self.mouse_text_color_dir = -1;
                }
                if self.mouse_text_color <= 190 {
                    self.mouse_text_color_dir = 1;
                }
                // `Main.DoUpdate_AnimateTileGlows`: the demon torch colour ramp.
                self.demon_torch += self.demon_torch_dir * 0.01;
                if self.demon_torch > 1.0 {
                    self.demon_torch = 1.0;
                    self.demon_torch_dir = -1.0;
                }
                if self.demon_torch < 0.0 {
                    self.demon_torch = 0.0;
                    self.demon_torch_dir = 1.0;
                }
                // `TileLightScanner.Update()`: one `NextSeed()` per frame, then
                // `WithModifier(x, y)` per tile (done inside the scanner).
                let mut light_rng = tr_core::tile_light::FastRandom::new(self.light_rng_seed);
                light_rng.next_seed_in_place();
                self.light_rng_seed = light_rng.seed;
                let params = LightParams {
                    sky: sky_light,
                    hell_glow: 0.55,
                    mouse_text_color: self.mouse_text_color,
                    demon_torch: self.demon_torch,
                    disco: self.disco,
                    // `Main.GlobalTimeWrappedHourly = TotalGameTime.TotalSeconds % 3600`.
                    global_time: anim_time % 3600.0,
                    // `Main.timeForVisualEffects`: ticks since start, wrapped at 216000.
                    time_for_visuals: (anim_time * 60.0) % 216000.0,
                    time: clock_time as f32,
                    day_time: is_day,
                    raining: rain.is_raining(),
                    // `Main.shimmerDarken` / `Main.liquidAlpha[13]`: shimmer is not
                    // simulated yet, so both stay at their neutral value.
                    shimmer_darken: 0.0,
                    shimmer_alpha: 0.0,
                    sundial_cooldown: 0,
                    moondial_cooldown: 0,
                    // `Main.jellyfishCageMode`: `Main.AnimateTiles` is not ported, so
                    // every cage stays in mode 0.
                    cage_mode: [[0u8; tr_core::tile_light::CAGE_FRAMES]; 3],
                    rng_seed: self.light_rng_seed,
                };
                let sky_uv = atlas
                    .as_ref()
                    .and_then(|a| assets::Lookups::sheet_uv(a, "Background_0"));
                let style = LightStyle::for_mode(*light_mode);
                let surface_px = world.world_surface as f32 * TILE_PX;
                // Liquids animate the texture at 8 fps; include the frame in the G6
                // key only when the world has any liquid.
                let liquid_anim = if *has_liquids {
                    ((anim_time * 8.0) as u32) & 15
                } else {
                    0
                };

                // G6: the whole world range (sky strips, tiles, walls, dark rects)
                // depends on exactly these values. A hit reuses last frame's uploaded
                // instance range and only rebuilds the UI.
                let key = WorldFrameKey {
                    cam_tile: [
                        (camera.x / TILE_PX).floor() as i32,
                        (camera.y / TILE_PX).floor() as i32,
                    ],
                    zoom: zoom.to_bits(),
                    screen: [screen.x.to_bits(), screen.y.to_bits()],
                    sky_light: [
                        params.sky[0].to_bits(),
                        params.sky[1].to_bits(),
                        params.sky[2].to_bits(),
                    ],
                    hell_glow: params.hell_glow.to_bits(),
                    light_params: params.identity(),
                    light_mode: *light_mode as u32,
                    anim: if style.perturb > 0.0 {
                        anim_time.to_bits()
                    } else {
                        liquid_anim
                    },
                    player_key,
                    npc_key,
                    items_key: item::render_key(items, ((anim_time * 8.0) as u32) & 15),
                    hand_key: {
                        let held = inventory
                            .get(*selected)
                            .filter(|s| s.count > 0)
                            .map_or(0, |s| s.kind as u32);
                        let phase = if *swing > 0.0 {
                            (((1.0 - *swing / SWING_TIME) * 40.0) as u32).min(40)
                        } else {
                            0
                        };
                        held | (phase << 16)
                    },
                    light_generation: light.generation(),
                    waterfall: waterfall.anim_key(),
                    world_revision: world.revision,
                };
                let reuse_world = !no_cache() && *world_key == Some(key);
                if reuse_world {
                    renderer.begin_frame_keep_world();
                    tile_stats.draw_ms = 0.0;
                    tile_stats.light_ms = 0.0;
                    tile_stats.world_reused = true;
                } else {
                    renderer.begin_frame();
                }
                // The parallax sky follows the camera continuously; it lives in the
                // always-rebuilt back range so the world range can stay cached.
                draw_sky(
                    renderer,
                    *camera,
                    *zoom,
                    screen,
                    world,
                    clouds,
                    sky,
                    stars,
                    surface_px,
                    is_day,
                    clock_time,
                    anim_time,
                    sky_uv,
                    sky_tint,
                    rain.cloud_bg(),
                    settings.background,
                    bg_alpha_front,
                    bg_alpha_far,
                    light,
                );
                if !reuse_world {
                    tile_stats = draw_tiles(
                        renderer,
                        world,
                        lookups,
                        atlas.as_ref(),
                        *camera,
                        *zoom,
                        screen,
                        &params,
                        light,
                        scratch,
                        &style,
                        anim_time,
                        waterfall,
                        render_player.as_ref(),
                        player_assets,
                        npcs,
                        npc_assets,
                        items,
                        &remote_players.values().cloned().collect::<Vec<_>>(),
                        *edit_target,
                    );
                    if let Some(p) = render_player.as_ref() {
                        if let Some(slot) = inventory.get(*selected) {
                            if slot.count > 0 {
                                tile_stats.quads += draw_held_item(
                                    renderer, lookups, light, &style, p, slot.kind, *swing,
                                    anim_time,
                                );
                            }
                        }
                    }
                    *world_key = Some(key);
                }

                rain.draw(renderer, lookups, light, *camera, *zoom, screen);
                for p in projectiles.iter() {
                    p.draw(renderer, lookups, light, *camera, *zoom, screen);
                }

                let boss = npcs
                    .iter()
                    .find(|n| n.kind == NpcKind::KingSlime && n.hp > 0)
                    .map(|n| (n.hp, n.max_hp));
                if menu.is_none() {
                    draw_hud(
                        renderer,
                        *camera,
                        *zoom,
                        screen,
                        lookups,
                        inventory,
                        *selected,
                        &self.hotbar_scales,
                        player.as_ref(),
                        boss,
                        self.inventory_open,
                        &self.mouse_item,
                        self.mouse_pos,
                    );
                } else if let Some(menu) = menu.as_mut() {
                    menu.draw(renderer, lookups, screen, dt);
                }
                if let Some(map) = map.as_ref() {
                    let pt = player
                        .as_ref()
                        .map(|p| ((p.x / TILE_PX) as i32, (p.y / TILE_PX) as i32))
                        .unwrap_or((world.width / 2, world.height / 2));
                    if menu.is_none() && settings.map_enabled {
                        map.draw_minimap(renderer, world, screen, pt);
                        if map.open {
                            map.draw_full(renderer, lookups, world, screen, pt);
                        }
                    }
                }

                if *debug {
                    draw_debug(
                        renderer,
                        screen,
                        tile_stats,
                        frame_history,
                        *camera,
                        *zoom,
                        *last_stats,
                        world,
                        *atlas_stats,
                        gpu,
                        dt,
                        params.sky,
                        debug_cache,
                        style.mode.label(),
                        player.as_ref(),
                        npcs.len(),
                        liquids.active_len(),
                        rain.visual_alpha(),
                        rain.drop_count(),
                        rain.snow_count(),
                    );
                }

                let frame = match gpu.surface.get_current_texture() {
                    wgpu::CurrentSurfaceTexture::Success(f)
                    | wgpu::CurrentSurfaceTexture::Suboptimal(f) => f,
                    other => {
                        log::debug!("surface acquire skipped: {other:?}");
                        return;
                    }
                };
                let view = frame
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default());

                renderer.aberration = style.aberration_px;
                stats = renderer.render(
                    gpu,
                    &view,
                    camera.to_array(),
                    *zoom,
                    now.elapsed().as_secs_f32(),
                    wgpu::Color {
                        r: 0.045,
                        g: 0.055,
                        b: 0.085,
                        a: 1.0,
                    },
                );
                if let Some(path) = shot_path.as_ref() {
                    if *frame_count >= *shot_at {
                        log::info!(
                            "shot camera px {:.2},{:.2} (tile {:.3},{:.3}) zoom {:.3}",
                            camera.x,
                            camera.y,
                            camera.x / TILE_PX,
                            camera.y / TILE_PX,
                            *zoom
                        );
                        capture_frame(gpu, &frame.texture, path);
                        gpu.queue.present(frame);
                        std::process::exit(0);
                    }
                }
                gpu.queue.present(frame);
            }
        }

        self.tile_stats = tile_stats;
        if self.stats_on {
            self.stats_acc.add(&tile_stats);
            if self.stats_acc.frames >= 120 {
                self.stats_acc.log_and_reset();
            }
        }
        self.last_stats = stats;
        let ms = dt * 1000.0;
        self.frame_history.push_back(ms);
        if self.frame_history.len() > 240 {
            self.frame_history.pop_front();
        }

        if let Some(target) = self.bench_frames {
            self.bench_samples.push(ms);
            if self.bench_samples.len() as u32 >= target {
                self.finish_bench();
            }
        }
    }

    fn pan_camera(&mut self, dt: f32) {
        let mut dir = Vec2::ZERO;
        if self.keys.contains(&KeyCode::KeyA) || self.keys.contains(&KeyCode::ArrowLeft) {
            dir.x -= 1.0;
        }
        if self.keys.contains(&KeyCode::KeyD) || self.keys.contains(&KeyCode::ArrowRight) {
            dir.x += 1.0;
        }
        if self.keys.contains(&KeyCode::KeyW) || self.keys.contains(&KeyCode::ArrowUp) {
            dir.y -= 1.0;
        }
        if self.keys.contains(&KeyCode::KeyS) || self.keys.contains(&KeyCode::ArrowDown) {
            dir.y += 1.0;
        }
        if dir != Vec2::ZERO {
            self.camera += dir.normalize() * 900.0 * dt / self.zoom;
        }
        if let Some(step) = pan_step() {
            self.camera.x += step * TILE_PX;
        }
    }

    fn finish_bench(&mut self) {
        let mut s = self.bench_samples.clone();
        s.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let n = s.len();
        let avg = s.iter().sum::<f32>() / n as f32;
        let pct = |p: f32| s[((n as f32 - 1.0) * p) as usize];
        println!(
            "BENCH frames={n} avg={avg:.3}ms ({:.1} fps) p50={:.3} p95={:.3} p99={:.3} max={:.3}",
            if avg > 0.0 { 1000.0 / avg } else { 0.0 },
            pct(0.50),
            pct(0.95),
            pct(0.99),
            s[n - 1]
        );
        println!(
            "BENCH draws={} instances={} visible_world={} zoom={:.2}",
            self.last_stats.draw_calls,
            self.last_stats.instances,
            self.world.tiles.len(),
            self.zoom
        );
        self.bench_frames = None;
        std::process::exit(0);
    }
}

fn lerp3(a: [f32; 3], b: [f32; 3], t: f32) -> [f32; 4] {
    [
        a[0] + (b[0] - a[0]) * t,
        a[1] + (b[1] - a[1]) * t,
        a[2] + (b[2] - a[2]) * t,
        1.0,
    ]
}

/// The debug font is ASCII-only; transliterate Cyrillic names so they stay readable.
fn display_name(name: &str) -> String {
    const CYRILLIC: &str = "АБВГДЕЁЖЗИЙКЛМНОПРСТУФХЦЧШЩЪЫЬЭЮЯабвгдеёжзийклмнопрстуфхцчшщъыьэюя";
    const LATIN: &[&str] = &[
        "A", "B", "V", "G", "D", "E", "Yo", "Zh", "Z", "I", "Y", "K", "L", "M", "N", "O", "P", "R",
        "S", "T", "U", "F", "Kh", "Ts", "Ch", "Sh", "Shch", "", "Y", "", "E", "Yu", "Ya", "a", "b",
        "v", "g", "d", "e", "yo", "zh", "z", "i", "y", "k", "l", "m", "n", "o", "p", "r", "s", "t",
        "u", "f", "kh", "ts", "ch", "sh", "shch", "", "y", "", "e", "yu", "ya",
    ];
    let mut out = String::with_capacity(name.len());
    for ch in name.chars() {
        if let Some(idx) = CYRILLIC.chars().position(|c| c == ch) {
            out.push_str(LATIN[idx]);
        } else if ch.is_ascii() {
            out.push(ch);
        } else {
            out.push('?');
        }
    }
    out
}

/// `Main.SetBackColor`: the colour the sky and surface background are tinted with
/// (`Main.ColorOfTheSkies`), as a function of world time.
///
/// Day runs `0..54000` (0 = sunrise, 27000 = noon), night runs `0..32400`
/// (16200 = midnight). Between 13500 and 37800 the colour is pure white, which is why
/// vanilla daytime tiles are lit at 100 %.
fn color_of_the_skies(time: f64, day: bool) -> [f32; 3] {
    let byte = |v: f64| (v.clamp(0.0, 255.0) / 255.0) as f32;
    if day {
        if time < 13500.0 {
            let t = time / 13500.0;
            [
                byte(t * 230.0 + 25.0),
                byte(t * 220.0 + 35.0),
                byte(t * 220.0 + 35.0),
            ]
        } else if time > 45900.0 {
            let t = (1.0 - (time / 54000.0 - 0.85) * 6.666_666_666_666_667).clamp(0.0, 1.0);
            [
                byte(t * 200.0 + 35.0),
                byte(t * 85.0 + 35.0),
                byte(t * 135.0 + 35.0),
            ]
        } else if time > 37800.0 {
            let t = (1.0 - (time / 54000.0 - 0.7) * 6.666_666_666_666_667).clamp(0.0, 1.0);
            [
                byte(t * 20.0 + 235.0),
                byte(t * 135.0 + 120.0),
                byte(t * 85.0 + 170.0),
            ]
        } else {
            [1.0, 1.0, 1.0]
        }
    } else if time < 16200.0 {
        let t = 1.0 - time / 16200.0;
        let g = byte(t * 30.0 + 5.0).max(15.0 / 255.0);
        [g, g, g]
    } else {
        let t = (time / 32400.0 - 0.5) * 2.0;
        [
            byte(t * 20.0 + 5.0).max(15.0 / 255.0),
            byte(t * 30.0 + 5.0).max(15.0 / 255.0),
            byte(t * 30.0 + 5.0).max(15.0 / 255.0),
        ]
    }
}

/// `Main.ApplyColorOfTheSkiesToTiles`: surface sunlight is the sky colour pulled
/// 70 % towards its own channel.
fn tile_color(sky: [f32; 3]) -> [f32; 3] {
    let sum = sky[0] + sky[1] + sky[2];
    [
        (sum + 7.0 * sky[0]) / 10.0,
        (sum + 7.0 * sky[1]) / 10.0,
        (sum + 7.0 * sky[2]) / 10.0,
    ]
}

/// One background cloud. Positions are in a virtual "cloud space": the draw pass
/// adds `camera.x * 0.5`, giving the clouds a slower parallax than the world.
#[derive(Clone, Copy)]
struct CloudSprite {
    uv: [f32; 4],
    w: f32,
    h: f32,
    x: f32,
    /// Fraction of the sky half-height above the camera (0 = center, 1 = top).
    alt: f32,
    scale: f32,
}

fn next_rand(state: &mut u64) -> u64 {
    *state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut z = *state;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

fn rand01(state: &mut u64) -> f32 {
    (next_rand(state) >> 40) as f32 / (1u32 << 24) as f32
}

/// Deterministic cloud layout for a world. The original keeps a live cloud array;
/// a fixed seeded set is enough for a background layer and keeps the world range
/// reusable while the camera is static.
fn make_clouds(world: &World, lookups: &assets::Lookups) -> Vec<CloudSprite> {
    let available: Vec<assets::CloudTex> = lookups.clouds.iter().flatten().copied().collect();
    if available.is_empty() {
        return Vec::new();
    }
    let mut state = world.seed ^ 0x2545_F491_4F6C_DD1D;
    let span = world.width as f32 * TILE_PX * 0.55 + 3000.0;
    let count = ((span / 300.0) as usize).clamp(40, 400);
    (0..count)
        .map(|_| {
            let tex = available[(next_rand(&mut state) % available.len() as u64) as usize];
            CloudSprite {
                uv: tex.uv,
                w: tex.w,
                h: tex.h,
                x: rand01(&mut state) * span,
                alt: 0.12 + rand01(&mut state) * 0.83,
                scale: 0.45 + rand01(&mut state) * 0.55,
            }
        })
        .collect()
}

#[derive(Clone, Copy)]
struct BgTex {
    uv: [f32; 4],
    w: f32,
    h: f32,
}

struct SkyAssets {
    /// Forest far/near mountain layers (`Background_7`/`Background_8`).
    far: Option<BgTex>,
    near: Option<BgTex>,
    /// Forest foreground tree line (`Background_50..52`).
    forest: [Option<BgTex>; 3],
    /// Snow biome: mountains `Background_35/36`, pine line `Background_37..39`.
    snow_far: Option<BgTex>,
    snow_near: Option<BgTex>,
    snow_trees: [Option<BgTex>; 3],
    /// Ocean horizon with islands (`Background_28`, style 0).
    ocean: Option<BgTex>,
    /// Desert dunes (`Background_21` far, `Background_20` near) plus the mountain
    /// layers behind them (`Background_23` far, `Background_22` near).
    desert_far: Option<BgTex>,
    desert_near: Option<BgTex>,
    desert_far_mnt: Option<BgTex>,
    desert_near_mnt: Option<BgTex>,
    /// Storm cloud banks (`Background_112/113`), faded in by `cloudBGAlpha`.
    storm_far: Option<BgTex>,
    storm_near: Option<BgTex>,
    /// `Background_1`: the one-row surface transition into the underground backdrop.
    underground_transition: Option<BgTex>,
    /// `Background_2`: the tiled dirt backdrop (`_drawBackground_backTexture[1]`).
    underground: Option<BgTex>,
    sun: Option<BgTex>,
    moon: Option<BgTex>,
    stars: Vec<Option<BgTex>>,
}

/// Default-player texture parts (`PlayerTextureID`): head, eyes, torso skin, shirt,
/// legs, pants, shoes, hair. Frames are 40x56 on the 40-px-wide strips.
struct PlayerAssets {
    head: Option<BgTex>,
    eye_whites: Option<BgTex>,
    eyes: Option<BgTex>,
    torso: Option<BgTex>,
    shirt: Option<BgTex>,
    /// Back-arm skin frame from `Player_0_7` (composite arm frame 1).
    arm: Option<BgTex>,
    legs: Option<BgTex>,
    pants: Option<BgTex>,
    shoes: Option<BgTex>,
    hair: Option<BgTex>,
}

fn make_player_assets(atlas: &Atlas) -> PlayerAssets {
    PlayerAssets {
        head: sky_tex(atlas, "Player_0_0", None),
        eye_whites: sky_tex(atlas, "Player_0_1", None),
        eyes: sky_tex(atlas, "Player_0_2", None),
        // Torso and shirt sheets are 9 columns wide; keep only the first column.
        torso: sky_tex(atlas, "Player_0_3", Some((0, 0, 40, 224))),
        shirt: sky_tex(atlas, "Player_0_6", Some((0, 0, 40, 224))),
        arm: sky_tex(atlas, "Player_0_7", None),
        legs: sky_tex(atlas, "Player_0_10", None),
        pants: sky_tex(atlas, "Player_0_11", None),
        shoes: sky_tex(atlas, "Player_0_12", None),
        hair: sky_tex(atlas, "Player_Hair_1", None),
    }
}

/// UV of one 40x56 frame in a player part strip.
fn player_frame_uv(tex: &BgTex, row: u32) -> [f32; 4] {
    let rows = (tex.h / 56.0).floor().max(1.0);
    let r = (row as f32).min(rows - 1.0);
    let dv = (tex.uv[3] - tex.uv[1]) * (56.0 / tex.h);
    [
        tex.uv[0],
        tex.uv[1] + dv * r,
        tex.uv[2],
        tex.uv[1] + dv * (r + 1.0),
    ]
}

/// UV of a 40x56 composite cell in a 9x4 player part sheet.
fn player_cell_uv(tex: &BgTex, col: u32, row: u32) -> [f32; 4] {
    let cols = (tex.w / 40.0).floor().max(1.0);
    let rows = (tex.h / 56.0).floor().max(1.0);
    let du = (tex.uv[2] - tex.uv[0]) / cols;
    let dv = (tex.uv[3] - tex.uv[1]) / rows;
    let c = (col as f32).min(cols - 1.0);
    let r = (row as f32).min(rows - 1.0);
    [
        tex.uv[0] + du * c,
        tex.uv[1] + dv * r,
        tex.uv[0] + du * (c + 1.0),
        tex.uv[1] + dv * (r + 1.0),
    ]
}

/// `PlayerDrawSet.CreateCompositeData`: body frame index -> front-arm cell.
fn arm_cell(frame: u32) -> (u32, u32) {
    match frame {
        0 => (2, 0),
        1 => (3, 0),
        2 => (4, 0),
        3 => (5, 0),
        4 => (6, 0),
        5 => (2, 1),
        6 => (3, 1),
        7..=10 => (4, 1),
        11..=13 => (3, 1),
        14 => (5, 1),
        15..=16 => (6, 1),
        17 => (5, 1),
        18..=19 => (3, 1),
        _ => (2, 0),
    }
}

fn flip_uv(uv: [f32; 4], flip: bool) -> [f32; 4] {
    if flip {
        [uv[2], uv[1], uv[0], uv[3]]
    } else {
        uv
    }
}

struct NpcAssets {
    slime: Option<BgTex>,
    eye: Option<BgTex>,
    zombie: Option<BgTex>,
    king_slime: Option<BgTex>,
}

fn make_npc_assets(atlas: &Atlas) -> NpcAssets {
    NpcAssets {
        slime: sky_tex(atlas, "NPC_1", None),
        eye: sky_tex(atlas, "NPC_2", None),
        zombie: sky_tex(atlas, "NPC_3", None),
        king_slime: sky_tex(atlas, "NPC_50", None),
    }
}

/// UV of one slime / demon-eye frame (two frames stacked vertically).
fn npc_frame_uv(tex: &BgTex, frame_h: f32, frame: u32) -> [f32; 4] {
    let dv = (tex.uv[3] - tex.uv[1]) * (frame_h / tex.h);
    let r = frame as f32;
    [
        tex.uv[0],
        tex.uv[1] + dv * r,
        tex.uv[2],
        tex.uv[1] + dv * (r + 1.0),
    ]
}

#[derive(Clone, Copy)]
struct StarSprite {
    x: f32,
    y: f32,
    size: f32,
    tex: usize,
    phase: f32,
}

fn sky_tex(atlas: &Atlas, name: &str, sub: Option<(u32, u32, u32, u32)>) -> Option<BgTex> {
    let rect = atlas.sheet(name)?;
    let (sx, sy, w, h) = sub.unwrap_or((0, 0, rect.width, rect.height));
    if sx + w > rect.width || sy + h > rect.height {
        return None;
    }
    Some(BgTex {
        uv: [
            (rect.x + sx) as f32 / atlas.width as f32,
            (rect.y + sy) as f32 / atlas.height as f32,
            (rect.x + sx + w) as f32 / atlas.width as f32,
            (rect.y + sy + h) as f32 / atlas.height as f32,
        ],
        w: w as f32,
        h: h as f32,
    })
}

fn make_sky_assets(atlas: &Atlas) -> SkyAssets {
    SkyAssets {
        far: sky_tex(atlas, "Background_7", None),
        near: sky_tex(atlas, "Background_8", None),
        forest: [50, 51, 52].map(|i| sky_tex(atlas, &format!("Background_{i}"), None)),
        snow_far: sky_tex(atlas, "Background_35", None),
        snow_near: sky_tex(atlas, "Background_36", None),
        snow_trees: [37, 38, 39].map(|i| sky_tex(atlas, &format!("Background_{i}"), None)),
        ocean: sky_tex(atlas, "Background_28", None),
        desert_far: sky_tex(atlas, "Background_21", None),
        desert_near: sky_tex(atlas, "Background_20", None),
        desert_far_mnt: sky_tex(atlas, "Background_23", None),
        desert_near_mnt: sky_tex(atlas, "Background_22", None),
        storm_far: sky_tex(atlas, "Background_112", None),
        storm_near: sky_tex(atlas, "Background_113", None),
        underground_transition: sky_tex(atlas, "Background_1", None),
        underground: sky_tex(atlas, "Background_2", None),
        sun: sky_tex(atlas, "Sun", None),
        moon: sky_tex(atlas, "Moon_0", Some((0, 0, 50, 50))),
        stars: (0..4)
            .map(|i| sky_tex(atlas, &format!("Star_{i}"), None))
            .collect(),
    }
}

/// True near either ocean edge. The generator's beach is 140 tiles wide; 200 leaves
/// room for the blend into the background.
fn is_ocean_biome(world: &World, camera: Vec2) -> bool {
    let x = (camera.x / TILE_PX) as i32;
    x < 200 || x > world.width - 200
}

/// Sand at the surface away from the beaches marks the inland desert.
fn is_desert_biome(world: &World, camera: Vec2) -> bool {
    let cam_tile = (camera.x / TILE_PX) as i32;
    if cam_tile < 220 || cam_tile > world.width - 220 {
        return false;
    }
    for dx in [-16, -8, 0, 8, 16] {
        let x = cam_tile + dx;
        let top = world.surface_of(x);
        if let Some(t) = world.tile(x, top) {
            if t.kind == 53 {
                return true;
            }
        }
    }
    false
}

fn is_snow_biome(world: &World, camera: Vec2) -> bool {
    let cam_tile = (camera.x / TILE_PX) as i32;
    for dx in [-16, -8, 0, 8, 16] {
        let x = cam_tile + dx;
        let top = world.surface_of(x);
        if let Some(t) = world.tile(x, top) {
            if matches!(t.kind, 147 | 161 | 162 | 163 | 164 | 200) {
                return true;
            }
        }
    }
    false
}


/// `Main.CalculateWaterStyle`: the `Misc/water_*` style index for the biome under the
/// camera. 0 is the forest default, 5 snow, 6 desert (12 underground desert), 13 ocean,
/// 7 shallow cave, 8 deep cave; lava (1), honey (11) and shimmer (14) are per-liquid
/// and never depend on the biome.
fn calculate_water_style(world: &World, camera: Vec2) -> usize {
    let tile_y = camera.y / TILE_PX;
    if is_desert_biome(world, camera) {
        return if tile_y > world.world_surface as f32 { 12 } else { 6 };
    }
    if is_snow_biome(world, camera) {
        return 5;
    }
    let tile_x = (camera.x / TILE_PX) as i32;
    if tile_x < 220 || tile_x > world.width - 220 {
        // `ZoneBeach` -> bgStyle 4 -> water 13.
        return 13;
    }
    if tile_y > world.rock_layer as f32 + 40.0 {
        // `ZoneGlowshroom` would pick 7 here; our generator has no glowshroom biome.
        return 8;
    }
    if tile_y > world.world_surface as f32 {
        return 7;
    }
    0
}

/// `Main.DrawBG_HandleBackgroundTransition` plus the `bgDelay` branch of
/// `Main.DrawBG`. The original's instant swap only pairs two forest variant
/// sets (`GetCurrentBackgroundOfStyle`), and all forest sets are tree set 1 in
/// our port, so it can never fire here.
///
/// Delay is 60 frames when entering a forest style, 30 otherwise; after the
/// switch `bgDelay` becomes -60 and holds a 60-frame cooldown.
fn advance_bg_style(style: &mut i32, delay: &mut i32, preferred: i32) {
    // `SurfaceBackgroundID.Sets.IsForest`: 0 and the four forest set styles.
    fn is_forest(style: i32) -> bool {
        matches!(style, 0 | 10 | 11 | 12)
    }
    if *delay < 0 {
        *delay += 1;
    } else if preferred != *style {
        let new_is_forest = is_forest(preferred);
        let wait = if new_is_forest { 60 } else { 30 };
        *delay += 1;
        if *delay > wait {
            *delay = -60;
            *style = preferred;
            if new_is_forest {
                *delay = 0;
            }
        }
    } else if *delay > 0 {
        *delay -= 1;
    }
}

/// `Main.UpdateBGVisibility_BackLayer` + `Main.UpdateBGVisibility_FrontLayer`.
///
/// Far indices: forest 0, desert 1, ocean 3, snow 4. Front indices equal the
/// style in the supported subset (forest 0, desert 2, ocean 4, snow 7).
fn advance_bg_alphas(front: &mut [f32; 16], far: &mut [f32; 16], style: i32, speed: f32) {
    // `DrawBG_ModifyBGFarBackLayerAlpha`: only the wanted layer grows; every other
    // non-zero layer fades out. (The desert-variant swap guard never applies to
    // the styles we support.)
    let far_target = match style {
        2 => 1,
        4 => 3,
        7 => 4,
        _ => 0,
    };
    for i in 0..far.len() {
        let wanted = far_target == i as i32;
        if wanted || far[i] != 0.0 {
            far[i] = (far[i] + if wanted { speed } else { -speed }).clamp(0.0, 1.0);
        }
    }
    // `UpdateBGVisibility_FrontLayer`: the target layer fades in, the rest out.
    for i in 0..front.len() {
        if style == i as i32 {
            front[i] = (front[i] + speed).min(1.0);
        } else if front[i] != 0.0 {
            front[i] = (front[i] - speed).max(0.0);
        }
    }
}

fn make_stars(world: &World) -> Vec<StarSprite> {
    let mut state = world.seed ^ 0xA076_1D64_78BD_642F;
    (0..140)
        .map(|_| StarSprite {
            x: rand01(&mut state),
            // Upper half of the screen; the ground covers the rest.
            y: rand01(&mut state) * 0.62,
            size: 3.0 + rand01(&mut state) * 6.0,
            tex: (next_rand(&mut state) % 4) as usize,
            phase: rand01(&mut state) * std::f32::consts::TAU,
        })
        .collect()
}

fn draw_sky(
    r: &mut SpriteRenderer,
    camera: Vec2,
    zoom: f32,
    screen: Vec2,
    world: &World,
    clouds: &[CloudSprite],
    sky: &SkyAssets,
    stars: &[StarSprite],
    surface_px: f32,
    is_day: bool,
    clock_time: f64,
    anim_time: f32,
    sky_uv: Option<[f32; 4]>,
    tint: [f32; 3],
    storm: f32,
    background_enabled: bool,
    // `Main.bgAlphaFrontLayer` / `Main.bgAlphaFarBackLayer`: per-layer cross-fade
    // alphas; two biome layers blend while `Main.bgStyle` transitions.
    bg_alpha_front: &[f32; 16],
    bg_alpha_far: &[f32; 16],
    light: &LightMap,
) {
    // `Background_0.png` is the game's own 48x1400 sky gradient: deep blue at the top,
    // pale near the horizon. It is anchored to `Main.worldSurface`.
    const SKY_H: f32 = 1400.0;
    let half = screen * 0.5 / zoom;
    let origin = camera - half;
    let bottom = camera.y + half.y;
    let white = r.white_uv();
    let width = screen.x / zoom;

    let sky_top = surface_px - SKY_H;
    if let Some(uv) = sky_uv {
        let y0 = origin.y.max(sky_top);
        let y1 = bottom.min(surface_px);
        if y1 > y0 {
            let dv = uv[3] - uv[1];
            let v0 = ((y0 - sky_top) / SKY_H).clamp(0.0, 1.0);
            let v1 = ((y1 - sky_top) / SKY_H).clamp(0.0, 1.0);
            let sub = [uv[0], uv[1] + dv * v0, uv[2], uv[1] + dv * v1];
            r.push_back_sprite(
                [origin.x, y0],
                [width, y1 - y0],
                sub,
                [tint[0], tint[1], tint[2], 1.0],
            );
        }
        // Above the gradient's top edge the sky is a flat zenith blue.
        if y0 > origin.y {
            let zenith = [53.0 / 255.0, 43.0 / 255.0, 243.0 / 255.0];
            r.push_back_sprite(
                [origin.x, origin.y],
                [width, y0 - origin.y],
                white,
                [
                    zenith[0] * tint[0],
                    zenith[1] * tint[1],
                    zenith[2] * tint[2],
                    1.0,
                ],
            );
        }
        // Below the average surface level keep the horizon colour; valleys would
        // otherwise show the clear colour under the gradient's bottom edge.
        if bottom > surface_px {
            let horizon = [132.0 / 255.0, 170.0 / 255.0, 248.0 / 255.0];
            r.push_back_sprite(
                [origin.x, surface_px.max(origin.y)],
                [width, bottom - surface_px.max(origin.y)],
                white,
                [
                    horizon[0] * tint[0],
                    horizon[1] * tint[1],
                    horizon[2] * tint[2],
                    1.0,
                ],
            );
        }
    } else {
        r.push_back_sprite(
            [origin.x, origin.y],
            [width, screen.y / zoom],
            white,
            [tint[0], tint[1], tint[2], 1.0],
        );
    }

    // Celestial bodies, stars and the parallax surface background. These are drawn
    // in screen space (compensated for zoom) like `DrawSunAndMoon`/`DrawSurfaceBG`.

    let sky_luma = (tint[0] + tint[1] + tint[2]) / 3.0;
    let star_alpha = ((1.0 - sky_luma * 2.8).clamp(0.0, 1.0)) * 0.85;
    if !is_day && star_alpha > 0.02 {
        for s in stars {
            let Some(st) = sky.stars.get(s.tex).and_then(|t| *t) else {
                continue;
            };
            let twinkle = 0.65 + 0.35 * s.phase.sin();
            push_screen(
                r,
                camera,
                zoom,
                screen,
                [s.x * screen.x, s.y * screen.y],
                [s.size, s.size],
                st.uv,
                [1.0, 1.0, 1.0, star_alpha * twinkle],
            );
        }
    }

    let t = if is_day {
        (clock_time / 54000.0).clamp(0.0, 1.0)
    } else {
        (clock_time / 32400.0).clamp(0.0, 1.0)
    } as f32;
    let cx = -60.0 + t * (screen.x + 120.0);
    let cy = screen.y * 0.10 + (2.0 * t - 1.0).powi(2) * screen.y * 0.20;
    if is_day {
        if let Some(sun) = sky.sun {
            push_screen(
                r,
                camera,
                zoom,
                screen,
                [cx - 62.0, cy - 62.0],
                [125.0, 125.0],
                sun.uv,
                [1.0, 1.0, 1.0, 1.0],
            );
        }
    } else if let Some(moon) = sky.moon {
        push_screen(
            r,
            camera,
            zoom,
            screen,
            [cx - 30.0, cy - 30.0],
            [60.0, 60.0],
            moon.uv,
            [1.0, 1.0, 1.0, 0.95],
        );
    }

    // `DrawSurfaceBG_BackMountainsStep1/2`: far then near mountain layer, topped by the
    // biome's tree line. `backgroundTopMagicNumber` makes the layers move at ~30 % of
    // the camera's vertical speed; using the local horizon instead would drag the whole
    // forest down into the view whenever the player is on a hill.
    // `Main.bgAlphaFrontLayer`/`bgAlphaFarBackLayer`: each biome layer is drawn with
    // `ColorOfSurfaceBackgroundsBase * alpha`, so two biomes blend while the style
    // cross-fades (`backgroundLayerTransitionSpeed = 0.05`). Index tables:
    //   front: forest 0, desert 2, ocean 4, snow 7
    //   far  : forest 0, desert 1, ocean 3, snow 4     (`UpdateBGVisibility_BackLayer`)
    let front_forest = bg_alpha_front[0];
    let front_desert = bg_alpha_front[2];
    let front_snow = bg_alpha_front[7];
    let far_forest = bg_alpha_far[0];
    // `UpdateBGVisibility_BackLayer`: desert far = 1, ocean = 3, snow = 4.
    let far_desert = bg_alpha_far[1];
    let far_snow = bg_alpha_far[4];
    let far_ocean = bg_alpha_far[3];
    // `Main.screenPosition` is the *unzoomed* view top-left: the surface background
    // is laid out in that space (`DrawSurfaceBG` uses `screenPosition` directly), so
    // changing `GameZoomTarget` must not move or resize it.
    let world_surface_px = (world.world_surface as f32 * TILE_PX).max(1.0);
    let view_top = camera.y - screen.y * 0.5;
    let magic = (300.0 - view_top) / world_surface_px;
    let sc_adj = bg_scale_adjust(world, view_top, screen.y);
    // `Main.DrawSurfaceBG`: `pushBGTopHack = 30` for the mountain passes, then
    // `-180` after the horizon layer, so the front layers use `30 - 180 = -150`.
    // The `+30`/`-150` are folded into the layer tops below.
    // `Main.BackgroundEnabled`: the parallax layers only; the gradient and the
    // clouds above stay, exactly like the original toggle.
    let show_bg = background_enabled && view_top < world_surface_px + TILE_PX;
    // `DrawSurfaceBG` positions every layer from `screenPosition.Y` alone; anchoring
    // them to the surface of the camera's own column made the whole skyline jump
    // whenever the camera crossed a hill, so the horizon follows the world's average
    // surface (a constant) and moves only with the camera.
    if show_bg && storm > 0.01 {
        // `cloudBG` layers: two storm cloud banks behind the mountains, fading in
        // with cloudAlpha. They keep the sky busy between the horizon and the zenith.
        if let Some(t) = sky.storm_far {
            draw_bg_layer(
                r,
                camera,
                zoom,
                screen,
                &t,
                1.65,
                0.09,
                magic * 900.0 + 600.0 + sc_adj + 30.0,
                false,
                [tint[0] * 0.9, tint[1] * 0.92, tint[2] * 0.96],
                storm,
            );
        }
        if let Some(t) = sky.storm_near {
            draw_bg_layer(
                r,
                camera,
                zoom,
                screen,
                &t,
                1.85,
                0.12,
                magic * 1100.0 + 750.0 + sc_adj + 30.0,
                false,
                tint,
                (storm * 1.5).min(1.0),
            );
        }
    }
    // Layers are positioned like `DrawSurfaceBG` but drawn in screen pixels (the
    // original applies the zoom in its sprite-batch transform). Keep the bottom of
    // every layer below the local horizon so no sky leaks between it and the terrain.
    //
    // `DrawSurfaceBG_BackMountainsStep1`: far mountains, parallax 0.15, scale 2.0,
    // each biome at its `bgAlphaFarBackLayer` value. Vanilla order is 0, 1 (desert),
    // then 4 (snow). `Background_22/23` are skipped for `desertBG == 4`; our
    // generator never uses that style.
    if show_bg {
        if far_forest > 0.0 {
            if let Some(far) = sky.far {
                draw_bg_layer(
                    r, camera, zoom, screen, &far, 2.0, 0.15,
                    magic * 1300.0 + 1090.0 + sc_adj + 30.0,
                    false, tint, far_forest,
                );
            }
        }
        if far_desert > 0.0 {
            if let Some(far) = sky.desert_far_mnt {
                draw_bg_layer(
                    r, camera, zoom, screen, &far, 2.0, 0.15,
                    magic * 1300.0 + 1090.0 + sc_adj + 30.0,
                    false, tint, far_desert,
                );
            }
        }
        if far_snow > 0.0 {
            if let Some(far) = sky.snow_far {
                draw_bg_layer(
                    r, camera, zoom, screen, &far, 2.0, 0.15,
                    magic * 1300.0 + 1090.0 + sc_adj + 30.0,
                    false, tint, far_snow,
                );
            }
        }
    }

    // Parallax clouds. They are world objects: a fixed altitude above the average
    // surface (so they move with the world, not with the player's height) and a
    // horizontal drift of half the camera plus the wind, like the original's layers.
    let right = camera.x + half.x;
    let cloud_tint = [tint[0] * 0.97, tint[1] * 0.98, tint[2], 1.0];
    for c in clouds {
        let w = c.w * c.scale;
        let h = c.h * c.scale;
        // Vanilla clouds drift with the wind even when the camera is still.
        let wx = c.x + camera.x * 0.5 + anim_time * 4.0;
        let wy = world_surface_px - (60.0 + c.alt * 260.0) - h * 0.5;
        if wx + w < origin.x || wx > right || wy + h < origin.y || wy > bottom {
            continue;
        }
        r.push_back_sprite([wx, wy], [w, h], c.uv, cloud_tint);
    }

    // `DrawSurfaceBG_BackMountainsStep2`: near mountains, parallax 0.20, scale 2.3,
    // in vanilla order: forest (0), desert (1), ocean (`Main.oceanBG`,
    // `Background_28`, 3), snow (4).
    if show_bg {
        if far_forest > 0.0 {
            if let Some(near) = sky.near {
                draw_bg_layer(
                    r, camera, zoom, screen, &near, 2.3, 0.20,
                    magic * 1400.0 + 1260.0 + sc_adj + 30.0,
                    false, tint, far_forest,
                );
            }
        }
        if far_desert > 0.0 {
            if let Some(near) = sky.desert_near_mnt {
                draw_bg_layer(
                    r, camera, zoom, screen, &near, 2.3, 0.20,
                    magic * 1400.0 + 1260.0 + sc_adj + 30.0,
                    false, tint, far_desert,
                );
            }
        }
        if far_ocean > 0.0 {
            if let Some(sea) = sky.ocean {
                draw_bg_layer(
                    r, camera, zoom, screen, &sea, 2.3, 0.20,
                    magic * 1400.0 + 1260.0 + sc_adj + 30.0,
                    false, tint, far_ocean,
                );
            }
        }
        if far_snow > 0.0 {
            if let Some(near) = sky.snow_near {
                draw_bg_layer(
                    r, camera, zoom, screen, &near, 2.3, 0.20,
                    magic * 1400.0 + 1260.0 + sc_adj + 30.0,
                    false, tint, far_snow,
                );
            }
        }
    }

    // Front layer loop (`Main.DrawSurfaceBG` 59529): forest (0), desert (2), snow (7).
    // Each layer draws with `ColorOfSurfaceBackgroundsBase * bgAlphaFrontLayer[i]`.
    if show_bg {
        // `DrawSurfaceBG_Forest` (tree set 1 = `Background_50..52`): parallax
        // 0.40/0.43/0.49, scale 2.5/2.62/2.68, tops +1300/+1550/+1800. The middle
        // layer is mirrored horizontally (`tex == 51`).
        draw_front_trees(
            r, camera, zoom, screen, &sky.forest, front_forest,
            [2.5, 2.62, 2.68], [0.40, 0.43, 0.49], [1800.0, 1950.0, 2100.0],
            [1300.0, 1550.0, 1800.0], Some(1), magic, sc_adj, tint,
        );
        // `DrawSurfaceBG_Desert`: two dune layers (21/20), parallax 0.37/0.49,
        // scale 2.5/2.68, tops +1600/+2000, no tree line.
        let desert_trees = [sky.desert_far, sky.desert_near, None];
        draw_front_trees(
            r, camera, zoom, screen, &desert_trees, front_desert,
            [2.5, 2.68, 2.68], [0.37, 0.49, 0.49], [1800.0, 2100.0, 2100.0],
            [1600.0, 2000.0, 2000.0], None, magic, sc_adj, tint,
        );
        // `DrawSurfaceBG_Snow` (tree set `Main.snowBG` style 1 = `Background_37..39`):
        // same parallax/scale/top as the forest, but none of the layers is mirrored
        // and there is no -50 texture tweak.
        draw_front_trees(
            r, camera, zoom, screen, &sky.snow_trees, front_snow,
            [2.5, 2.62, 2.68], [0.40, 0.43, 0.49], [1800.0, 1950.0, 2100.0],
            [1350.0, 1600.0, 1850.0], None, magic, sc_adj, tint,
        );
    }

    // `Main.DrawBackground_SurfaceTransitionBackground` +
    // `Main.DrawBackground_DirtBackground`: the textured underground backdrop, anchored
    // to `worldSurface` across the whole screen. (A per-column gradient used to hang a
    // dark "wall" underneath every player-placed row of blocks.)
    static BG: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    if !*BG.get_or_init(|| std::env::var_os("TR_NO_BG").is_some()) {
        draw_underground_background(r, camera, zoom, screen, world, sky, light);
    }
}

/// `Math.IEEERemainder`: `x - y * Math.Round(x / y)` with banker's rounding.
fn ieee_remainder(x: f32, y: f32) -> f32 {
    x - y * round_half_even(x / y)
}

/// `Math.Round(double)`: round half to even.
fn round_half_even(v: f32) -> f32 {
    let f = v.floor();
    let d = v - f;
    if d > 0.5 {
        f + 1.0
    } else if d < 0.5 {
        f
    } else if (f as i64) % 2 == 0 {
        f
    } else {
        f + 1.0
    }
}

/// One 16x16 cell of the underground backdrop, lit by the world tile behind it.
///
/// `pos` is the screen-space top-left of the cell (C#'s
/// `bgStartX + num * i + 16 * k + num2`, `bgStartY + backgroundHeight * j + 16 * l`);
/// `sample` is the screen-space point the light is read at (C#'s `num3`). The original
/// reads the light at the *un-jittered* cell centre, so `sample.x = pos.x + 8 - num2`,
/// while the draw itself keeps the `num2` half-tile jitter.
#[allow(clippy::too_many_arguments)]
fn push_cave_cell(
    r: &mut SpriteRenderer,
    camera: Vec2,
    zoom: f32,
    screen: Vec2,
    tex: &BgTex,
    sx: f32,
    sy: f32,
    pos: [f32; 2],
    sample: [f32; 2],
    light: &LightMap,
    bg_brightness: f32,
) {
    if pos[0] >= screen.x || pos[1] >= screen.y || pos[0] + 16.0 <= 0.0 || pos[1] + 16.0 <= 0.0 {
        return;
    }
    // `Lighting.GetColor((int)((num3 + screenPosition.X) / 16f), ...)`: the light is read on
    // the *unscaled* grid (`Main.screenPosition = camera - screenSize / 2`), not at the tile
    // actually behind the pixel. `Lighting.GetColor` folds in the (ramped) `GlobalBrightness`,
    // which the light map stores unscaled, and both backdrop layers then multiply the result
    // by `backgroundColor` (0.9).
    let sp = camera - screen * 0.5;
    let wx = ((sample[0] + sp.x) / TILE_PX).floor() as i32;
    let wy = ((sample[1] + sp.y) / TILE_PX).floor() as i32;
    let c = light.get(wx, wy);
    let du = (tex.uv[2] - tex.uv[0]) / tex.w;
    let dv = (tex.uv[3] - tex.uv[1]) / tex.h;
    let k = bg_brightness * 0.9;
    r.push_back_sprite(
        [
            camera.x + (pos[0] - screen.x * 0.5) / zoom,
            camera.y + (pos[1] - screen.y * 0.5) / zoom,
        ],
        [16.0 / zoom, 16.0 / zoom],
        [
            tex.uv[0] + du * sx,
            tex.uv[1] + dv * sy,
            tex.uv[0] + du * (sx + 16.0),
            tex.uv[1] + dv * (sy + 16.0),
        ],
        [c[0] * k, c[1] * k, c[2] * k, 1.0],
    );
}

/// `Main.GetRockTransitionPoint`: first rock-backdrop row, in **world** pixels.
///
/// ```text
/// num  = (rockLayer - worldSurface) * 16
/// num2 = num / backgroundHeight[2]; if (num % h != 0) num2++
/// return worldSurface * 16 + num2 * h + 32
/// ```
fn get_rock_transition_point(world: &World, block: f32) -> f32 {
    let block = (block as f64).max(1.0);
    let span = (world.rock_layer - world.world_surface) * TILE_PX as f64;
    let mut blocks = (span / block) as i32;
    if span.rem_euclid(block) != 0.0 {
        blocks += 1;
    }
    (world.world_surface * TILE_PX as f64 + blocks as f64 * block + 32.0) as f32
}

/// Screen-space anchors of the underground backdrop, split out so the layout can be
/// unit-tested without a GPU.
#[derive(Clone, Copy, Debug, PartialEq)]
struct UndergroundBgLayout {
    /// `Main.DrawBG` uses `Main.BackgroundViewMatrix` (zoom `ForcedMinimumZoom` = 1), so
    /// the backdrop lives in *unscaled* screen space: `Main.screenPosition`, i.e.
    /// `camera - screenSize / 2`, used verbatim.
    screen_pos: Vec2,
    /// `DrawBackground_SurfaceTransitionBackground` `bgTopY = worldSurface*16 - screenPosition.Y`.
    transition_top: f32,
    /// `DrawBackground_DirtBackground` `bgTopY = worldSurface*16 - screenPosition.Y + 16`.
    dirt_top: f32,
    /// `Main.Camera.ScaledPosition.Y + screenHeight`: the dirt sheet is clamped at the
    /// rock transition while this bottom edge is below `GetRockTransitionPoint`.
    scaled_bottom: f32,
}

fn underground_bg_layout(world: &World, camera: Vec2, zoom: f32, screen: Vec2) -> UndergroundBgLayout {
    let screen_pos = camera - screen * 0.5;
    let transition_top = world.world_surface as f32 * TILE_PX - screen_pos.y;
    UndergroundBgLayout {
        screen_pos,
        transition_top,
        dirt_top: transition_top + 16.0,
        // `ScaledPosition.Y = UnscaledPosition.Y + GameViewMatrix.Translation.Y` and the
        // translation is `screenSize.Y / 2 * (1 - 1 / zoom)`, so the top-left of the
        // scaled view is `camera.Y - screenSize.Y / (2 * zoom)`.
        scaled_bottom: camera.y - screen.y / (2.0 * zoom) + screen.y,
    }
}

/// `Main.DrawBackground_SurfaceTransitionBackground` + `DrawBackground_DirtBackground`.
///
/// Both layers are screen-space and scroll with `Main.caveParallax`. The transition is
/// one `Background_1` row at `worldSurface`; the dirt backdrop (`Background_2`) starts
/// one tile lower and tiles down as `96x96` blocks. Every 16x16 cell takes the light of
/// the world tile it covers, which is what darkens the caves and keeps lit ground lit.
fn draw_underground_background(
    r: &mut SpriteRenderer,
    camera: Vec2,
    zoom: f32,
    screen: Vec2,
    world: &World,
    sky: &SkyAssets,
    light: &LightMap,
) {
    let Some(dirt) = sky.underground else {
        return;
    };
    // `Main.caveParallax = 1 - bgScroll / 500`; the options slider defaults to 50.
    const CAVE_PARALLAX: f32 = 0.9;
    // `Main.DrawBG` runs the backdrop under `Main.BackgroundViewMatrix`, whose zoom is
    // `ForcedMinimumZoom` (1 for any viewport <= 1920x1200), so the matrix is identity
    // and the layout is in *unzoomed* screen coordinates: `Main.screenPosition`
    // (`camera - screenSize / 2`) is used verbatim, never divided by the game zoom.
    // `push_cave_cell` maps those screen px 1:1 with a fixed 16 px cell.
    let layout = underground_bg_layout(world, camera, zoom, screen);
    let screen_pos = layout.screen_pos;
    let surface_screen = layout.transition_top;
    // `DrawBackground`: the whole background pass runs with
    // `Lighting.GlobalBrightness = 1.2*(1-num3) + num3`, where
    // `num3 = clamp((screenPosition.Y - worldSurface*16)/300, 0, 1)`. The light map stores
    // the unscaled engine value (tiles re-apply `1.2` through `LightStyle::shade`), so the
    // backdrop has to fold the ramp in itself: `1.2 - 0.2*num3`.
    let num3 = ((screen_pos.y - world.world_surface as f32 * TILE_PX) / 300.0).clamp(0.0, 1.0);
    let bg_brightness = 1.2 - 0.2 * num3;

    // The transition row sits exactly on the surface line.
    if let Some(transition) = sky.underground_transition {
        let num = transition.w - 32.0;
        if num >= 16.0 {
            let start_x =
                (0.0 - ieee_remainder(num + screen_pos.x * CAVE_PARALLAX, num) - num * 0.5) as i32
                    as f32;
            let loops = (screen.x / num) as i32 + 2;
            let mut n2 = round_half_even(-ieee_remainder(start_x + screen_pos.x, 16.0));
            if n2 as i32 == -8 {
                n2 = 8.0;
            }
            for i in 0..loops {
                for j in 0..(num as i32 / 16) {
                    let cell_x = start_x + num * i as f32 + 16.0 * j as f32;
                    push_cave_cell(
                        r,
                        camera,
                        zoom,
                        screen,
                        &transition,
                        16.0 * j as f32 + n2 + 16.0,
                        0.0,
                        [cell_x + n2, surface_screen],
                        [cell_x + 8.0, surface_screen],
                        light,
                        bg_brightness,
                    );
                }
            }
        }
    }

    // `if (!(worldSurface * 16 <= screenPosition.Y + screenHeight + offScreen)) return`:
    // vanilla still enters while the surface line is up to `offScreenRange` (192) px below
    // the screen bottom. Those cells are off-screen and clipped by `push_cave_cell`, so the
    // padding only matters for exactness, not for the visible picture.
    const OFF_SCREEN_RANGE: f32 = 192.0;
    if surface_screen > screen.y + OFF_SCREEN_RANGE {
        return;
    }
    let num = dirt.w - 32.0;
    if num < 16.0 {
        return;
    }
    let start_x =
        (0.0 - ieee_remainder(num + screen_pos.x * CAVE_PARALLAX, num) - num * 0.5) as i32 as f32;
    let loops = (screen.x / num) as i32 + 2;
    let mut n2 = round_half_even(-ieee_remainder(start_x + screen_pos.x, 16.0));
    if n2 as i32 == -8 {
        n2 = 8.0;
    }
    // `bgTopY = worldSurface * 16 - screenPosition.Y + 16` (DrawBackground_DirtBackground).
    let top = layout.dirt_top;
    // `bgStartY`: `if (worldSurface * 16 < screenPosition.Y - 16)` the sheet is repeated
    // from above the screen (`(int)(IEEERemainder(bgTopY, h) - h)`), otherwise it starts
    // exactly one tile below the surface line (`(int)bgTopY`). The test is on
    // `screenPosition.Y`, NOT on `bgTopY`, so it still holds while the surface line is a
    // few tiles off the top of the screen.
    let start_y = if world.world_surface as f32 * TILE_PX < screen_pos.y - 16.0 {
        (ieee_remainder(top, dirt.h) - dirt.h) as i32 as f32
    } else {
        top as i32 as f32
    };
    let mut loops_y = ((screen.y - start_y) / dirt.h) as i32 + 1;
    // `if (rockTransitionPoint < Main.Camera.ScaledPosition.Y + screenHeight - 16)` clamps
    // the dirt sheet at the rock transition (`GetRockTransitionPoint`): the rock backdrop
    // takes over below it (`DrawBackground_DrawRockLayer`), so the dirt must not bleed into
    // the caverns. The row count is measured from `bgStartY` (the wrapped start above).
    let rock_transition = get_rock_transition_point(world, dirt.h);
    if rock_transition < layout.scaled_bottom - 16.0 {
        // C#: `(int)(rockTransitionPoint - screenPosition.Y - bgStartY) / backgroundHeight[2]`
        // — the numerator is truncated to `int` *before* the (integer) division.
        loops_y = ((rock_transition - screen_pos.y - start_y) as i32) / (dirt.h as i32);
    }
    if loops_y <= 0 {
        return;
    }
    let rows = (dirt.h as i32 / 16).max(1);
    for i in 0..loops {
        for j in 0..loops_y {
            for k in 0..(num as i32 / 16) {
                for l in 0..rows {
                    let cell_x = start_x + num * i as f32 + 16.0 * k as f32;
                    let cell_y = start_y + dirt.h * j as f32 + 16.0 * l as f32;
                    push_cave_cell(
                        r,
                        camera,
                        zoom,
                        screen,
                        &dirt,
                        16.0 * k as f32 + n2 + 16.0,
                        16.0 * l as f32,
                        [cell_x + n2, cell_y],
                        [cell_x + 8.0, cell_y + 8.0],
                        light,
                        bg_brightness,
                    );
                }
            }
        }
    }
}

/// Push a sprite with a screen-space position/size, compensating for the camera zoom.
/// Used by the sky, sun/moon, stars and parallax background layers.
fn push_screen(
    r: &mut SpriteRenderer,
    camera: Vec2,
    zoom: f32,
    screen: Vec2,
    pos: [f32; 2],
    size: [f32; 2],
    uv: [f32; 4],
    color: [f32; 4],
) {
    let wx = camera.x + (pos[0] - screen.x * 0.5) / zoom;
    let wy = camera.y + (pos[1] - screen.y * 0.5) / zoom;
    r.push_back_sprite([wx, wy], [size[0] / zoom, size[1] / zoom], uv, color);
}

/// The three foreground layers of one biome (`DrawSurfaceBG_Forest` /
/// `_Desert` / `_Snow`).
///
/// `ks`/`tops` are the `backgroundTopMagicNumber * k + c + scAdj` terms from the
/// original; `pushBGTopHack` is already folded into `tops` (front layers use
/// `-150`). Every layer is tinted by `ColorOfSurfaceBackgroundsBase * alpha`,
/// where `alpha` is the biome's `Main.bgAlphaFrontLayer` entry.
#[allow(clippy::too_many_arguments)]
fn draw_front_trees(
    r: &mut SpriteRenderer,
    camera: Vec2,
    zoom: f32,
    screen: Vec2,
    trees: &[Option<BgTex>; 3],
    alpha: f32,
    scales: [f32; 3],
    parallax: [f32; 3],
    ks: [f32; 3],
    tops: [f32; 3],
    flip_layer: Option<usize>,
    magic: f32,
    sc_adj: f32,
    tint: [f32; 3],
) {
    if alpha <= 0.0 {
        return;
    }
    for i in 0..3 {
        if let Some(t) = trees[i] {
            draw_bg_layer(
                r,
                camera,
                zoom,
                screen,
                &t,
                scales[i],
                parallax[i],
                magic * ks[i] + tops[i] + sc_adj,
                flip_layer == Some(i),
                tint,
                alpha,
            );
        }
    }
}

/// `Main.ComputeScAdj`: small vertical screen-space correction for backgrounds.
#[allow(clippy::too_many_arguments)]
fn draw_bg_layer(
    r: &mut SpriteRenderer,
    camera: Vec2,
    zoom: f32,
    screen: Vec2,
    tex: &BgTex,
    scale: f32,
    parallax: f32,
    top: f32,
    flip: bool,
    tint: [f32; 3],
    alpha: f32,
) {
    // Terraria renders backgrounds through `BackgroundViewMatrix`, whose zoom is
    // `ForcedMinimumZoom` (1): the layers keep their screen size at any game zoom,
    // and `bgTopY` is already a screen-space top, so no further transform applies.
    let sw = tex.w * scale;
    let sh = tex.h * scale;
    let uv = if flip {
        [tex.uv[2], tex.uv[1], tex.uv[0], tex.uv[3]]
    } else {
        tex.uv
    };
    // `bgStartX = (int)(-IEEERemainder(screenPosition.X * bgParallax, bgWidthScaled)
    // - bgWidthScaled / 2)`: phase against the screen's left edge, not the camera
    // centre, and half a tile of offset so the tiling is centred on the view.
    let screen_left = camera.x - screen.x * 0.5;
    let start = -(ieee_remainder(screen_left * parallax, sw) as i32 as f32) - sw * 0.5;
    let mut x = start;
    while x < screen.x {
        push_screen(
            r,
            camera,
            zoom,
            screen,
            [x, top],
            [sw, sh],
            uv,
            [tint[0], tint[1], tint[2], alpha],
        );
        x += sw;
    }
}

/// `Main.ComputeScAdj`: a screen-height/altitude correction subtracted from every
/// background layer's top.
fn bg_scale_adjust(world: &World, view_top: f32, view_height: f32) -> f32 {
    let world_surface_px = (world.world_surface as f32 * TILE_PX).max(1.0);
    let num = world_surface_px / (view_top + view_height).max(1.0);
    let mut altitude = world.height as f32 * 0.15 * TILE_PX - view_top;
    if altitude < 0.0 {
        altitude = 0.0;
    }
    altitude *= 0.00025;
    let num = num * (0.45 - altitude * altitude);
    let factor = if world.height <= 1200 {
        -500.0
    } else if world.height <= 1800 {
        -300.0
    } else {
        -150.0
    };
    num * factor
}

#[derive(Default, Clone, Copy, Debug)]
struct TileDrawStats {
    visible: u32,
    walls: u32,
    walls_culled: u32,
    dark_cells: u32,
    dark_rects: u32,
    chunks_skipped: u32,
    lit_tiles: u32,
    quads: u32,
    foliage: u32,
    liquids: u32,
    waterfalls: u32,
    items: u32,
    light_ms: f32,
    draw_ms: f32,
    instances: u32,
    world_reused: bool,
}

/// Scratch storage for the greedy dark-rectangle merge (A3/A4).
#[derive(Default)]
struct DrawScratch {
    /// Rectangles still being extended downwards: `[x0, x1, y0, y1]`.
    open: Vec<[i32; 4]>,
    next: Vec<[i32; 4]>,
    /// Dark x-runs of the current row.
    cur: Vec<(i32, i32)>,
}

/// Vanilla stores light as bytes; a channel below `1/255` is zero. Such a cell is
/// guaranteed to rasterise to black whatever the texture colour, so it can be
/// replaced by one opaque black quad without changing a single pixel.
#[inline]
fn light_is_zero(l: [f32; 3]) -> bool {
    l[0] * 255.0 < 1.0 && l[1] * 255.0 < 1.0 && l[2] * 255.0 < 1.0
}

/// Extends open dark rectangles with this row's runs and emits the ones that end.
/// Returns the number of emitted quads.
fn merge_dark_row(
    r: &mut SpriteRenderer,
    white: [f32; 4],
    scratch: &mut DrawScratch,
    y: i32,
) -> u32 {
    let mut emitted = 0u32;
    scratch.next.clear();
    let mut oi = 0usize;
    let mut ci = 0usize;
    while oi < scratch.open.len() || ci < scratch.cur.len() {
        let take_old = if oi >= scratch.open.len() {
            false
        } else if ci >= scratch.cur.len() {
            true
        } else {
            let o = scratch.open[oi];
            let c = scratch.cur[ci];
            o[0] < c.0 || (o[0] == c.0 && o[1] != c.1)
        };
        if take_old {
            let o = scratch.open[oi];
            r.push_sprite(
                [o[0] as f32 * TILE_PX, o[2] as f32 * TILE_PX],
                [
                    (o[1] - o[0] + 1) as f32 * TILE_PX,
                    (o[3] - o[2] + 1) as f32 * TILE_PX,
                ],
                white,
                [0.0, 0.0, 0.0, 1.0],
            );
            emitted += 1;
            oi += 1;
        } else if oi >= scratch.open.len() || scratch.cur[ci].0 < scratch.open[oi][0] {
            let (s, e) = scratch.cur[ci];
            scratch.next.push([s, e, y, y]);
            ci += 1;
        } else {
            // Same x span: grow the rectangle one row down.
            let mut o = scratch.open[oi];
            o[3] = y;
            scratch.next.push(o);
            oi += 1;
            ci += 1;
        }
    }
    std::mem::swap(&mut scratch.open, &mut scratch.next);
    emitted
}

#[allow(clippy::too_many_arguments)]
fn draw_tiles(
    r: &mut SpriteRenderer,
    world: &World,
    lookups: &mut assets::Lookups,
    atlas: Option<&Atlas>,
    camera: Vec2,
    zoom: f32,
    screen: Vec2,
    params: &LightParams,
    light: &mut LightMap,
    scratch: &mut DrawScratch,
    style: &LightStyle,
    anim_time: f32,
    waterfall: &mut tr_core::waterfall::WaterfallManager,
    player: Option<&Player>,
    player_assets: &PlayerAssets,
    npcs: &[Npc],
    npc_assets: &NpcAssets,
    items: &[ItemDrop],
    remote: &[net::PlayerNet],
    edit_target: Option<(i32, i32)>,
) -> TileDrawStats {
    let t_total = Instant::now();
    let half = screen * 0.5 / zoom;
    let min = camera - half;
    let max = camera + half;
    let x0 = ((min.x / TILE_PX).floor() as i32 - 1).clamp(0, world.width - 1);
    let x1 = ((max.x / TILE_PX).ceil() as i32 + 1).clamp(0, world.width - 1);
    let y0 = ((min.y / TILE_PX).floor() as i32 - 1).clamp(0, world.height - 1);
    let y1 = ((max.y / TILE_PX).ceil() as i32 + 1).clamp(0, world.height - 1);

    // `Main.waterStyle` is a single biome value for the whole frame, exactly like
    // `Main.CalculateWaterStyle` (the original evaluates it from the player's biome).
    let water_style = calculate_water_style(world, camera);
    let t_light = Instant::now();
    light.compute_into(world, x0, y0, x1, y1, params);
    let light_ms = t_light.elapsed().as_secs_f32() * 1000.0;

    if let Ok(v) = std::env::var("TR_LIGHT_AT") {
        use std::sync::atomic::{AtomicBool, Ordering};
        static ONCE: AtomicBool = AtomicBool::new(false);
        if !ONCE.swap(true, Ordering::Relaxed) {
            if let Some((sx, sy)) = v.split_once(',') {
                if let (Ok(lx), Ok(ly)) = (sx.trim().parse::<i32>(), sy.trim().parse::<i32>()) {
                    let c = light.get(lx, ly);
                    let tile = world.tile(lx, ly).copied().unwrap_or_default();
                    log::info!(
                        "light at {lx},{ly} = {:.3},{:.3},{:.3} (tile {} wall {} liquid {})",
                        c[0],
                        c[1],
                        c[2],
                        tile.kind,
                        tile.wall,
                        tile.liquid
                    );
                }
            }
        }
    }

    static DRAW_WALLS: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    let draw_walls = *DRAW_WALLS.get_or_init(|| std::env::var("TR_NO_WALLS").is_err());
    let width = world.width as usize;
    let white = r.white_uv();
    let mut stats = TileDrawStats {
        lit_tiles: light.lit_tiles,
        light_ms,
        ..Default::default()
    };
    let chunk = tr_core::world::CHUNK;
    // Vanilla's quality thresholds (`TileDrawing.Draw`, default gfxQuality 0.5):
    // above the high mark blocks get nine light slices, above the medium mark four,
    // below it a single flat quad.
    const HIGH_LIGHT: f32 = 142.0 / 255.0;
    const MEDIUM_LIGHT: f32 = 26.0 / 255.0;

    // `LiquidRenderer.InternalPrepareDraw` runs once per frame; both the opaque
    // background pass and the translucent foreground pass consume this record.
    let liquid_draws = tr_core::liquid_draw::build(world, x0, y0, x1, y1);
    if std::env::var_os("TR_DUMP_LIQUID").is_some() {
        let cam_x = camera.x as i32 / 16;
        let cam_y = camera.y as i32 / 16;
        let mut n_vis = 0;
        let mut n_tot = 0;
        for d in &liquid_draws {
            if (d.x - cam_x).abs() < 40 && (d.y - cam_y).abs() < 40 {
                n_tot += 1;
                if d.visible {
                    n_vis += 1;
                }
            }
        }
        log::info!(
            "LIQDUMP near cam {},{}: total={} visible={}",
            cam_x,
            cam_y,
            n_tot,
            n_vis
        );
        for d in &liquid_draws {
            if d.visible && (d.x - cam_x).abs() < 40 && (d.y - cam_y).abs() < 40 {
                log::info!(
                    "  vis tile {},{} type{} lvl{} sw{} sh{} fy{} fx{} sx{} sy{}",
                    d.x,
                    d.y,
                    d.liquid_type,
                    d.src_w,
                    d.src_h,
                    d.is_surface as i32,
                    d.src_y,
                    d.src_x,
                    d.offset_x,
                    d.offset_y
                );
            }
        }
    }
    // `Main.DrawLiquid(bg: true, ...)`: water is drawn before walls and tiles at
    // `Opacity` (usually 1.0), so the parallax background never shows through it.
    if std::env::var_os("TR_NO_LIQBACK").is_none() {
        stats.liquids += draw_liquids(
            r,
            lookups,
            light,
            style,
            anim_time,
            &liquid_draws,
            LiquidPass::Background,
            water_style,
        );
    }

    // `Main.DoDraw_Waterfalls`: vanilla draws falls after the non-solid tile pass
    // and before solid tiles. This renderer keeps one row-major tile pass, so the
    // falls are emitted here; tile quads drawn later occlude their base, while a
    // wall behind a fall is drawn over it (documented approximation).
    if std::env::var_os("TR_NO_WATERFALL").is_none() {
        let mut falls = Vec::new();
        waterfall.draw(world, light, style, anim_time, water_style as i32, [x0, y0, x1, y1], &mut falls);
        for d in falls {
            let Some(uv) = lookups.waterfall_uv(d.texture as usize, d.source) else {
                continue;
            };
            r.push_sprite(
                d.pos,
                [d.source[2] as f32, d.source[3] as f32],
                flip_uv(uv, d.flip),
                d.color,
            );
            stats.waterfalls += 1;
        }
    }

    // `Main.DrawWalls`: the whole wall layer is drawn before the tile layer. Each
    // frame is a 32x32 sprite placed 8 px up/left of the cell, so neighbouring walls
    // join without seams; drawing it interleaved with tiles would let the overhang
    // cut into tiles drawn earlier in the row.
    if draw_walls {
        for ty in y0..=y1 {
            for tx in x0..=x1 {
                let tile = &world.tiles[ty as usize * width + tx as usize];
                if tile.wall == 0 {
                    continue;
                }
                let Some(wf) = lookups.wall_tex(tile.wall, tile.wall_col(), tile.wall_row()) else {
                    continue;
                };
                if tile.active()
                    && tile_hides_wall(
                        lookups,
                        atlas,
                        tile.kind,
                        tile.frame_x,
                        tile.frame_y,
                        tile.slope(),
                        tile.half_brick(),
                    )
                {
                    stats.walls_culled += 1;
                    continue;
                }
                // `WallDrawing.DrawWalls` skips black cells; the tile pass merges the
                // fully covered ones into black rectangles.
                if light_is_zero(light.get(tx, ty)) {
                    continue;
                }
                // The 32 px sprite is centred on the 16 px cell (`x*16 - 8`), like
                // `new Vector2(j * 16 - 8, i * 16 - 8)` in the original.
                stats.quads += push_wall_lit(
                    r,
                    [tx as f32 * TILE_PX - 8.0, ty as f32 * TILE_PX - 8.0],
                    wf.uv,
                    light,
                    style,
                    tx,
                    ty,
                    anim_time,
                );
                stats.walls += 1;
            }
        }
    }

    scratch.open.clear();
    for ty in y0..=y1 {
        scratch.cur.clear();
        let mut run_start = -1i32;
        let cy = ty / chunk;
        let row_base = ty as usize * width;
        for cx in (x0 / chunk)..=(x1 / chunk) {
            if !world.chunk_has(cx, cy) {
                stats.chunks_skipped += 1;
                if run_start >= 0 {
                    let sx = (cx * chunk).max(x0);
                    scratch.cur.push((run_start, sx - 1));
                    run_start = -1;
                }
                continue;
            }
            let sx = (cx * chunk).max(x0);
            let ex = ((cx + 1) * chunk - 1).min(x1);
            for tx in sx..=ex {
                let tile = &world.tiles[row_base + tx as usize];
                let l = light.get(tx, ty);
                let active = tile.active();
                let is_tree = active && tree::is_tree(tile.kind);
                let marker = if is_tree {
                    tree::TreePart::marker(tile.frame_x, tile.frame_y)
                } else {
                    None
                };

                // `GetTileDrawData`: per-type sprite size/top/flip (torches 20x20 with
                // `tileTop 4`, plants 20 px, herbs, candles, palms...).
                let dd = if active && marker.is_none() && !is_tree {
                    let solid_above = world
                        .tile(tx, ty - 1)
                        .is_some_and(|t| {
                            t.active()
                                && td::TILE_SOLID.has(t.kind)
                                && !td::TILE_SOLID_TOP.has(t.kind)
                        });
                    tr_core::drawdata::draw_data(tile.kind, tx, solid_above)
                } else {
                    tr_core::drawdata::TileDrawData::default()
                };

                // Resolve the drawn tile frame once; reused for opacity and drawing.
                let mut tile_frame = None;
                if active && marker.is_none() {
                    if is_tree {
                        if tile.frame_x >= 0 && tile.frame_y >= 0 {
                            let off = tree::trunk_sheet_offset_at(world, tx, ty);
                            // Tree trunks are 20x20 sprites on a 22 px grid.
                            tile_frame = atlas.and_then(|a| {
                                lookups.rect_uv(
                                    a,
                                    tile.kind,
                                    tile.frame_x + off as i16,
                                    tile.frame_y,
                                    tree::TRUNK_SIZE as u32,
                                    tree::TRUNK_SIZE as u32,
                                )
                            });
                        }
                    } else if tile.frame_x >= 0 && tile.frame_y >= 0 {
                        tile_frame = atlas.and_then(|a| {
                            if dd.width != 16 || dd.height != 16 || dd.add_fr_x != 0 || dd.add_fr_y != 0
                            {
                                lookups.rect_uv(
                                    a,
                                    tile.kind,
                                    tile.frame_x + dd.add_fr_x,
                                    tile.frame_y + dd.add_fr_y,
                                    dd.width as u32,
                                    dd.height as u32,
                                )
                            } else {
                                lookups.frame_tex(a, tile.kind, tile.frame_x, tile.frame_y)
                            }
                        });
                    }
                    if tile_frame.is_none() {
                        tile_frame = lookups.plain_tex(tile.kind);
                    }
                }
                let solid = (tile.kind as usize) < td::TILE_COUNT && td::TILE_SOLID.has(tile.kind);
                // A3/A4: fully black cell -> greedy merged rectangle.
                // Vanilla paints black coverage from two places, both under
                // `TileDrawingBase.DrawOwnBlacks`:
                //  * `TileDrawing.DrawSingleTile`: every *active* tile whose light is
                //    exactly zero emits a black quad behind its sprite (`DrawBlack`).
                //  * `WallDrawing.DrawWalls`: every cell with a wall that is not a
                //    `FullTile` (block already covering it) emits a black quad.
                // That coverage is what hides the parallax background inside unlit
                // caves. A 32x32 wall sheet frame is mostly transparent outside its
                // 16x16 core, so `frame.opaque` is false; a wall has to count as
                // covering on its own. A fully dark tile renders identically whether
                // its sprite is drawn or skipped (both are solid black), so the cell
                // is folded into one merged rectangle.
                if light_is_zero(l) && (active || (draw_walls && tile.wall != 0)) {
                    if run_start < 0 {
                        run_start = tx;
                    }
                    stats.dark_cells += 1;
                    continue;
                }
                if run_start >= 0 {
                    scratch.cur.push((run_start, tx - 1));
                    run_start = -1;
                }

                let pos = [tx as f32 * TILE_PX, ty as f32 * TILE_PX];
                // Walls are drawn in their own pass above (`Main.DrawWalls`).
                // `DrawTile_LiquidBehindTile` (solid layer): a slope that stands in
                // liquid draws the water mask behind its open half, so the shoreline
                // is water instead of the parallax background.
                if solid && tile_frame.is_none_or(|f| !f.opaque) {
                    stats.quads += draw_tile_liquid_behind(
                        r,
                        world,
                        lookups,
                        light,
                        style,
                        anim_time,
                        tx,
                        ty,
                        tile,
                        water_style,
                    );
                }
                let Some(tf) = tile_frame else {
                    continue;
                };
                stats.visible += 1;
                if is_tree {
                    // 20x20 trunk sprite, drawn like `TileDrawing` does: the 20 px
                    // sprite is centred on the 16 px cell (position shifted by -2),
                    // which is what makes branches meet the trunk exactly.
                    push_lit(
                        r,
                        [pos[0] - 2.0, pos[1]],
                        [tree::TRUNK_SIZE, tree::TRUNK_SIZE],
                        tf.uv,
                        l,
                        style,
                        tx,
                        ty,
                        anim_time,
                    );
                    stats.quads += 1;
                } else if dd.top != 0
                    || dd.width != 16
                    || dd.height != 16
                    || dd.flip
                {
                    // `x*16 - (tileWidth - 16) / 2`, `y*16 + tileTop`; horizontal flip
                    // swaps the source columns.
                    let (dw, dh) = (dd.width as f32, dd.height as f32);
                    push_lit(
                        r,
                        [pos[0] - (dw - 16.0) * 0.5, pos[1] + dd.top as f32],
                        [dw, dh],
                        flip_uv(tf.uv, dd.flip),
                        l,
                        style,
                        tx,
                        ty,
                        anim_time,
                    );
                    stats.quads += 1;
                } else if tile.slope() != 0 {
                    stats.quads +=
                        push_slope(r, pos, tf.uv, l, style, tx, ty, anim_time, tile.slope());
                } else if tile.half_brick() {
                    push_half(r, pos, tf.uv, l, style, tx, ty, anim_time);
                    stats.quads += 1;
                } else if style.sliced_blocks
                    && solid
                    && l[0].max(l[1]).max(l[2]) > MEDIUM_LIGHT
                {
                    stats.quads +=
                        push_sliced(r, pos, tf.uv, light, style, tx, ty, anim_time, HIGH_LIGHT);
                } else {
                    push_lit(
                        r,
                        pos,
                        [TILE_PX, TILE_PX],
                        tf.uv,
                        l,
                        style,
                        tx,
                        ty,
                        anim_time,
                    );
                    stats.quads += 1;
                }
            }
        }
        if run_start >= 0 {
            scratch.cur.push((run_start, x1));
        }
        stats.dark_rects += merge_dark_row(r, white, scratch, ty);
    }
    // Emit rectangles that reach the bottom edge of the visible area.
    for o in scratch.open.drain(..) {
        r.push_sprite(
            [o[0] as f32 * TILE_PX, o[2] as f32 * TILE_PX],
            [
                (o[1] - o[0] + 1) as f32 * TILE_PX,
                (o[3] - o[2] + 1) as f32 * TILE_PX,
            ],
            white,
            [0.0, 0.0, 0.0, 1.0],
        );
        stats.dark_rects += 1;
    }

    if std::env::var_os("TR_NO_FOLIAGE").is_none() {
        stats.foliage = draw_tree_foliage(r, world, lookups, light, style, anim_time, x0, y0, x1, y1);
    }
    for it in items {
        stats.items += it.draw(r, lookups, light, style, anim_time);
    }
    stats.quads += draw_npcs(
        r, npc_assets, npcs, light, style, anim_time, camera, zoom, screen,
    );
    if let Some(p) = player {
        stats.quads += draw_player(
            r, player_assets, p, light, style, anim_time, camera, zoom, screen,
        );
    }
    for rp in remote {
        let mut p = Player::spawn(world);
        p.x = rp.x;
        p.y = rp.y;
        p.prev_x = rp.x;
        p.prev_y = rp.y;
        p.vx = rp.vx;
        p.vy = rp.vy;
        p.facing = rp.facing;
        p.on_ground = rp.on_ground;
        p.walk_time = rp.walk_time;
        p.gfx_off_y = 0.0;
        stats.quads += draw_player(
            r, player_assets, &p, light, style, anim_time, camera, zoom, screen,
        );
        let sx = (p.x + player::WIDTH * 0.5 - camera.x) * zoom + screen.x * 0.5;
        let sy = (p.y - 14.0 - camera.y) * zoom + screen.y * 0.5;
        let tw = hud_text_width(r, lookups, &rp.name, 0.8);
        hud_text(r, lookups, sx + 1.0 - tw * 0.5, sy + 1.0, 0.8, [0.0, 0.0, 0.0, 1.0], &rp.name);
        hud_text(r, lookups, sx - tw * 0.5, sy, 0.8, [1.0, 1.0, 1.0, 1.0], &rp.name);
    }
    // `Main.DrawLiquid(bg: false, ...)` runs after entities: the foreground water
    // layer tints everything that stands in it.
    if std::env::var_os("TR_NO_LIQFRONT").is_none() {
        stats.liquids += draw_liquids(
            r,
            lookups,
            light,
            style,
            anim_time,
            &liquid_draws,
            LiquidPass::Foreground,
            water_style,
        );
    }
    if let Some((tx, ty)) = edit_target {
        let white = r.white_uv();
        let pos = [tx as f32 * TILE_PX, ty as f32 * TILE_PX];
        let c = [1.0, 1.0, 1.0, 0.55];
        for (dx, dy, w, h) in [
            (0.0, 0.0, 16.0, 1.0),
            (0.0, 15.0, 16.0, 1.0),
            (0.0, 0.0, 1.0, 16.0),
            (15.0, 0.0, 1.0, 16.0),
        ] {
            r.push_sprite([pos[0] + dx, pos[1] + dy], [w, h], white, c);
        }
    }
    stats.instances = stats.quads
        + stats.walls
        + stats.dark_rects
        + stats.foliage
        + stats.liquids
        + stats.waterfalls
        + stats.items;
    stats.draw_ms = t_total.elapsed().as_secs_f32() * 1000.0;
    stats
}

#[derive(Clone, Copy, PartialEq)]
enum LiquidPass {
    /// `isBackgroundDraw = true`: opaque, drawn before walls and tiles.
    Background,
    /// `isBackgroundDraw = false`: `DEFAULT_OPACITY`, drawn over entities.
    Foreground,
}

/// `LiquidRenderer.DrawNormalLiquids`: emit one pass of the prepared liquid records.
#[allow(clippy::too_many_arguments)]
fn draw_liquids(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    light: &LightMap,
    style: &LightStyle,
    anim_time: f32,
    draws: &[tr_core::liquid_draw::LiquidDraw],
    pass: LiquidPass,
    water_style: usize,
) -> u32 {
    // `LiquidRenderer.Update`: body/edge frames advance with the (wind-driven)
    // animation clock, interior frames with the slower waterfall clock.
    let anim_frame = ((anim_time * 8.0) as i32) & 15;
    let waterfall_frame = ((anim_time * 0.5) as i32) & 15;
    let mut quads = 0;
    for d in draws {
        if !d.visible || d.src_w <= 0 || d.src_h <= 0 {
            continue;
        }
        // `DEFAULT_OPACITY`: water 0.6, lava/honey 0.95, shimmer 0.75.
        let (tex, default_opacity) = match d.liquid_type {
            1 => (1, 0.95),
            2 => (11, 0.95),
            3 => (14, 0.75),
            _ => (water_style, 0.60),
        };
        let corners = light.corner4(d.x, d.y);
        // `DrawNormalLiquids`: surface tiles snap to the still frame at y=1280,
        // everything else advances inside its 80 px block.
        let src_y = if d.is_surface {
            1280
        } else if d.src_x == 16 {
            d.src_y + waterfall_frame * 80
        } else {
            d.src_y + anim_frame * 80
        };
        let Some(uv) = lookups.liquid_uv(
            tex,
            d.src_x,
            src_y,
            d.src_w as u32,
            d.src_h as u32,
        ) else {
            continue;
        };
        let alpha = if pass == LiquidPass::Background {
            d.opacity
        } else {
            d.opacity * default_opacity
        }
        .min(1.0);
        // `Lighting.GetCornerColors` interpolates between the four corners on the
        // GPU. The instance format carries one tint, so split the visible rect
        // into a 3x3 grid and evaluate the bilinear corner colour per cell.
        let xs = [0, d.src_w / 3, d.src_w * 2 / 3, d.src_w];
        let ys = [0, d.src_h / 3, d.src_h * 2 / 3, d.src_h];
        for i in 0..3 {
            for j in 0..3 {
                let cw = xs[i + 1] - xs[i];
                let ch = ys[j + 1] - ys[j];
                if cw <= 0 || ch <= 0 {
                    continue;
                }
                let u = (xs[i] + xs[i + 1]) as f32 * 0.5 / d.src_w as f32;
                let v = (ys[j] + ys[j + 1]) as f32 * 0.5 / d.src_h as f32;
                let col = bilerp_corners(corners, u, v);
                let tint = style.shade(col, d.x, d.y, anim_time);
                r.push_sprite(
                    [
                        d.x as f32 * TILE_PX + d.offset_x + xs[i] as f32,
                        d.y as f32 * TILE_PX + d.offset_y + ys[j] as f32,
                    ],
                    [cw as f32, ch as f32],
                    sub_uv_frac(
                        uv,
                        xs[i] as f32,
                        ys[j] as f32,
                        cw as f32,
                        ch as f32,
                        d.src_w as f32,
                        d.src_h as f32,
                    ),
                    [
                        tint[0].min(1.0),
                        tint[1].min(1.0),
                        tint[2].min(1.0),
                        alpha,
                    ],
                );
                quads += 1;
            }
        }
    }
    quads
}

/// `TileDrawing.DrawTile_LiquidBehindTile`: a block that stands in liquid gets the
/// liquid drawn *behind* it, so the transparent parts of edge and slope frames show
/// water instead of the parallax background.
#[allow(clippy::too_many_arguments)]
fn draw_tile_liquid_behind(
    r: &mut SpriteRenderer,
    world: &World,
    lookups: &assets::Lookups,
    light: &LightMap,
    style: &LightStyle,
    anim_time: f32,
    tx: i32,
    ty: i32,
    tile: &tr_core::tile::Tile,
    water_style: usize,
) -> u32 {
    // `BlocksWaterDrawingBehindSelf`: chests and friends never draw water behind
    // themselves, and while sloped they fall back to the plain liquid frame.
    const BLOCKS_WATER_BEHIND: [u16; 5] = [54, 541, 328, 459, 470];
    if std::env::var_os("TR_NO_LIQBEHIND").is_some() {
        return 0;
    }
    let blocks_behind = BLOCKS_WATER_BEHIND.contains(&tile.kind);
    // `!tileSolid[type] || tileSolidTop[type]` both bail out (line 4064).
    if !tile.active() || !td::TILE_SOLID.has(tile.kind) || td::TILE_SOLID_TOP.has(tile.kind) {
        return 0;
    }
    let slope = tile.slope();
    if blocks_behind && slope == 0 {
        return 0;
    }
    let neighbor = |dx: i32, dy: i32| world.tile(tx + dx, ty + dy).copied().unwrap_or_default();
    let right = neighbor(1, 0);
    let left = neighbor(-1, 0);
    let above = neighbor(0, -1);
    let below = neighbor(0, 1);
    let mut level = 0i32;
    let mut kind = tile.liquid_kind();
    let mut flag_self = false;
    let mut flag_left = false;
    let mut flag_right = false;
    let mut flag_above = false;
    let mut flag_below = false;
    // `flag5`: the tile's own liquid only counts when the block is not full size.
    if tile.liquid > 0 && (slope != 0 || (tile.half_brick() && tile.liquid > 160)) {
        flag_self = true;
        kind = tile.liquid_kind();
        level = tile.liquid as i32;
    }
    if left.liquid > 0 && slope != 1 && slope != 3 {
        flag_left = true;
        kind = left.liquid_kind();
        level = level.max(left.liquid as i32);
    }
    if right.liquid > 0 && slope != 2 && slope != 4 {
        flag_right = true;
        kind = right.liquid_kind();
        level = level.max(right.liquid as i32);
    }
    if above.liquid > 0 && slope != 3 && slope != 4 {
        flag_above = true;
        kind = above.liquid_kind();
    }
    if below.liquid > 0 && slope != 1 && slope != 2 {
        if below.liquid > 240 {
            flag_below = true;
        }
        kind = below.liquid_kind();
    }
    if !(flag_self || flag_left || flag_right || flag_above || flag_below) {
        return 0;
    }
    if flag_below && (flag_left || flag_right) {
        flag_left = true;
        flag_right = true;
    }
    let tex = match kind {
        tr_core::tile::LiquidKind::Lava => 1,
        tr_core::tile::LiquidKind::Honey => 11,
        tr_core::tile::LiquidKind::Shimmer => 14,
        tr_core::tile::LiquidKind::Water => water_style,
    };
    // `(!flag3 || !(flag | flag2)) && !(flag4 & flag3)`: liquid on every side keeps
    // the default full-cell rect at the tile origin instead of a partial strip.
    let partial = (!flag_above || !(flag_left || flag_right)) && !(flag_below && flag_above);
    let (px, py, src_x, src_y, src_w, src_h);
    if !partial {
        px = tx as f32 * TILE_PX;
        py = ty as f32 * TILE_PX;
        src_x = 0;
        // The default rect skips the four surface rows of `Liquid_*`; the slope
        // masks are full-cell triangles, so shifting them would open a seam along
        // the diagonal.
        src_y = if slope != 0 { 0 } else { 4 };
        src_w = 16;
        src_h = 16;
    } else if flag_above {
        px = tx as f32 * TILE_PX;
        py = ty as f32 * TILE_PX;
        src_x = 0;
        src_y = 4;
        src_w = 16;
        src_h = if slope != 0 || tile.half_brick() { 12 } else { 4 };
    } else if flag_below && !flag_left && !flag_right {
        px = tx as f32 * TILE_PX;
        py = ty as f32 * TILE_PX + 12.0;
        src_x = 0;
        src_y = 4;
        src_w = 16;
        src_h = 4;
    } else {
        // `num14`: how far the liquid surface sits inside the cell (0 at 255).
        let offset = ((256 - level) / 32) * 2;
        let sy = if above.liquid == 0
            && (slope != 0 || tile.half_brick() || !solid_full(world, tx, ty - 1))
        {
            0
        } else {
            4
        };
        let mut x = tx as f32 * TILE_PX;
        py = ty as f32 * TILE_PX + offset as f32;
        src_x = 0;
        src_y = if slope != 0 { offset } else { sy };
        src_h = 16 - offset as u32;
        src_w = if slope != 0 || (flag_left && flag_right) || tile.half_brick() {
            // A slope always draws the whole diagonal mask; clipping it to 4 px
            // leaves the shore step showing the parallax background.
            16
        } else if flag_left {
            4
        } else {
            // Liquid only on the right: a 4 px strip against the right edge.
            x += 12.0;
            4
        };
        px = x;
    }
    // `num15`: 1.0 at/above the surface, otherwise 0.5 (0.6/0.9 in front of a
    // wall), with the half-brick and open-corner exceptions.
    let mut opacity = 0.5f32;
    if ty as f64 <= world.world_surface {
        opacity = 1.0;
        if tile.wall == 21 {
            opacity = 0.9;
        } else if tile.wall > 0 {
            opacity = 0.6;
        }
    }
    if tile.half_brick() && above.liquid > 0 && tile.wall > 0 {
        opacity = 0.0;
    }
    // Fully submerged cells sit under an opaque water column: their open corner is
    // deep water, not a window to the parallax. (The original keys this on the
    // average surface height; per-cell submersion reads the same at the shoreline
    // and avoids semi-transparent holes below a lake.)
    if above.liquid > 0 {
        opacity = 1.0;
    }
    if slope == 4 && left.liquid == 0 && !solid_full(world, tx - 1, ty) {
        opacity = 0.0;
    }
    if slope == 3 && right.liquid == 0 && !solid_full(world, tx + 1, ty) {
        opacity = 0.0;
    }
    if opacity <= 0.0 || src_w == 0 || src_h == 0 {
        return 0;
    }
    let uv = if slope != 0 && !blocks_behind {
        lookups.liquid_slope_uv(tex, slope, src_y, src_h)
    } else {
        // `Liquid_*` is 16 px tall; the original's 4..20 strip relies on texture
        // clamp, which for these uniform body rows is the same as cropping.
        let h = src_h.min(16u32.saturating_sub(src_y as u32)).max(1);
        lookups.liquid_part_uv(tex, src_x, src_y, src_w, h)
    };
    let Some(uv) = uv else {
        return 0;
    };
    let corners = light.corner4(tx, ty);
    let mut quads = 0;
    let xs = [0u32, src_w / 3, src_w * 2 / 3, src_w];
    let ys = [0u32, src_h / 3, src_h * 2 / 3, src_h];
    for i in 0..3 {
        for j in 0..3 {
            let cw = xs[i + 1] - xs[i];
            let ch = ys[j + 1] - ys[j];
            if cw == 0 || ch == 0 {
                continue;
            }
            let u = (xs[i] + xs[i + 1]) as f32 * 0.5 / src_w as f32;
            let v = (ys[j] + ys[j + 1]) as f32 * 0.5 / src_h as f32;
            let tint = style.shade(bilerp_corners(corners, u, v), tx, ty, anim_time);
            r.push_sprite(
                [px + xs[i] as f32, py + ys[j] as f32],
                [cw as f32, ch as f32],
                sub_uv_frac(
                    uv,
                    xs[i] as f32,
                    ys[j] as f32,
                    cw as f32,
                    ch as f32,
                    src_w as f32,
                    src_h as f32,
                ),
                [
                    tint[0].min(1.0),
                    tint[1].min(1.0),
                    tint[2].min(1.0),
                    opacity,
                ],
            );
            quads += 1;
        }
    }
    quads
}

/// `WallDrawing.DrawWalls`: a 32x32 wall sprite lit by the cell's four corner colours
/// (`Lighting.GetCornerColors`). Flat-lit cells stay a single quad.
#[allow(clippy::too_many_arguments)]
fn push_wall_lit(
    r: &mut SpriteRenderer,
    pos: [f32; 2],
    uv: [f32; 4],
    light: &LightMap,
    style: &LightStyle,
    tx: i32,
    ty: i32,
    time: f32,
) -> u32 {
    let corners = light.corner4(tx, ty);
    let mut lo = f32::MAX;
    let mut hi = f32::MIN;
    for c in &corners {
        for ch in c {
            lo = lo.min(*ch);
            hi = hi.max(*ch);
        }
    }
    let tint = |c: [f32; 3]| {
        let c = style.shade(c, tx, ty, time);
        [c[0].min(1.0), c[1].min(1.0), c[2].min(1.0), 1.0]
    };
    if hi - lo < 1.5 / 255.0 {
        r.push_sprite(pos, [32.0, 32.0], uv, tint(corners[0]));
        return 1;
    }
    let mut quads = 0;
    for (i, u) in [0.25f32, 0.75].iter().enumerate() {
        for (j, v) in [0.25f32, 0.75].iter().enumerate() {
            let x = 16.0 * i as f32;
            let y = 16.0 * j as f32;
            r.push_sprite(
                [pos[0] + x, pos[1] + y],
                [16.0, 16.0],
                sub_uv_frac(uv, x, y, 16.0, 16.0, 32.0, 32.0),
                tint(bilerp_corners(corners, *u, *v)),
            );
            quads += 1;
        }
    }
    quads
}

/// `WallDrawing.FullTile`: a full, opaque, unsloped block hides its wall completely.
fn tile_hides_wall(
    lookups: &mut assets::Lookups,
    atlas: Option<&Atlas>,
    kind: u16,
    frame_x: i16,
    frame_y: i16,
    slope: u8,
    half_brick: bool,
) -> bool {
    if tree::is_tree(kind) || slope != 0 || half_brick || frame_x < 0 || frame_y < 0 {
        return false;
    }
    let Some(atlas) = atlas else {
        return false;
    };
    match lookups.frame_tex(atlas, kind, frame_x, frame_y) {
        Some(f) => f.opaque,
        None => lookups.plain_tex(kind).is_some_and(|f| f.opaque),
    }
}

/// `WorldGen.SolidTile`: active, solid, not a platform, and a full block.
#[inline]
fn solid_full(world: &World, x: i32, y: i32) -> bool {
    world.tile(x, y).is_some_and(|t| {
        t.active()
            && td::TILE_SOLID.has(t.kind)
            && !td::TILE_SOLID_TOP.has(t.kind)
            && !t.half_brick()
            && t.slope() == 0
    })
}

/// Bilinear sample of `LightMap::corner4` (TL, TR, BL, BR).
#[inline]
fn bilerp_corners(c: [[f32; 3]; 4], u: f32, v: f32) -> [f32; 3] {
    let mut out = [0.0f32; 3];
    for k in 0..3 {
        let top = c[0][k] + (c[1][k] - c[0][k]) * u;
        let bottom = c[2][k] + (c[3][k] - c[2][k]) * u;
        out[k] = top + (bottom - top) * v;
    }
    out
}

/// Snap a sprite's top-left to an integer screen pixel so nearest sampling keeps
/// the same source texels while the camera moves (no crawling pixel grid).
#[inline]
fn snap_to_screen(pos: [f32; 2], camera: Vec2, zoom: f32, screen: Vec2) -> [f32; 2] {
    let sx = (pos[0] - camera.x) * zoom + screen.x * 0.5;
    let sy = (pos[1] - camera.y) * zoom + screen.y * 0.5;
    [
        camera.x + (sx.round() - screen.x * 0.5) / zoom,
        camera.y + (sy.round() - screen.y * 0.5) / zoom,
    ]
}

/// Tint and push one flat-lit sprite.
#[allow(clippy::too_many_arguments)]
#[inline]
fn push_lit(
    r: &mut SpriteRenderer,
    pos: [f32; 2],
    size: [f32; 2],
    uv: [f32; 4],
    light: [f32; 3],
    style: &LightStyle,
    tx: i32,
    ty: i32,
    time: f32,
) {
    let c = style.shade(light, tx, ty, time);
    r.push_sprite(pos, size, uv, [c[0], c[1], c[2], 1.0]);
}

/// Sub-rectangle of a 16x16 frame's UV rect, in tile pixels.
#[inline]
fn sub_uv(uv: [f32; 4], x: f32, y: f32, w: f32, h: f32) -> [f32; 4] {
    sub_uv_frac(uv, x, y, w, h, 16.0, 16.0)
}

/// `sub_uv` over a destination rect that is not 16x16.
#[inline]
fn sub_uv_frac(
    uv: [f32; 4],
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    full_w: f32,
    full_h: f32,
) -> [f32; 4] {
    let du = (uv[2] - uv[0]) / full_w;
    let dv = (uv[3] - uv[1]) / full_h;
    [
        uv[0] + du * x,
        uv[1] + dv * y,
        uv[0] + du * (x + w),
        uv[1] + dv * (y + h),
    ]
}

/// `DrawBasicTile` slope path: eight 2 px columns plus a 2 px strip, sampling the
/// tile's own 16x16 frame. Slopes 1/2 are ground slopes, 3/4 ceiling slopes.
#[allow(clippy::too_many_arguments)]
fn push_slope(
    r: &mut SpriteRenderer,
    pos: [f32; 2],
    uv: [f32; 4],
    light: [f32; 3],
    style: &LightStyle,
    tx: i32,
    ty: i32,
    time: f32,
    slope: u8,
) -> u32 {
    let c = style.shade(light, tx, ty, time);
    let color = [c[0], c[1], c[2], 1.0];
    let mut quads = 0;
    for i in 0..8u8 {
        let f = i as f32 * 2.0;
        let (dx, dy, sx, sy, w, h) = match slope {
            1 => (f, f, f, 0.0, 2.0, 14.0 - f),
            2 => (14.0 - f, f, 14.0 - f, 0.0, 2.0, 14.0 - f),
            3 => (f, 0.0, f, f, 2.0, 16.0 - f),
            _ => (14.0 - f, 0.0, 14.0 - f, f, 2.0, 16.0 - f),
        };
        if h <= 0.0 {
            continue;
        }
        r.push_sprite(
            [pos[0] + dx, pos[1] + dy],
            [w, h],
            sub_uv(uv, sx, sy, w, h),
            color,
        );
        quads += 1;
    }
    let strip_y = if slope <= 2 { 14.0 } else { 0.0 };
    r.push_sprite(
        [pos[0], pos[1] + strip_y],
        [16.0, 2.0],
        sub_uv(uv, 0.0, strip_y, 16.0, 2.0),
        color,
    );
    quads + 1
}

/// Half bricks keep the top half of the art and sit on the bottom 8 px of the cell.
fn push_half(
    r: &mut SpriteRenderer,
    pos: [f32; 2],
    uv: [f32; 4],
    light: [f32; 3],
    style: &LightStyle,
    tx: i32,
    ty: i32,
    time: f32,
) {
    let c = style.shade(light, tx, ty, time);
    r.push_sprite(
        [pos[0], pos[1] + 8.0],
        [16.0, 8.0],
        sub_uv(uv, 0.0, 0.0, 16.0, 8.0),
        [c[0], c[1], c[2], 1.0],
    );
}

/// `DrawSingleTile_SlicedBlock`: the block becomes a 4/8/4 grid whose outer slices
/// take light from the adjacent tile, smoothing the seams between blocks.
#[allow(clippy::too_many_arguments)]
fn push_sliced(
    r: &mut SpriteRenderer,
    pos: [f32; 2],
    uv: [f32; 4],
    light: &LightMap,
    style: &LightStyle,
    tx: i32,
    ty: i32,
    time: f32,
    high_threshold: f32,
) -> u32 {
    const SLICE_POS: [(f32, f32, f32, f32); 9] = [
        (0.0, 0.0, 4.0, 4.0),
        (4.0, 0.0, 8.0, 4.0),
        (12.0, 0.0, 4.0, 4.0),
        (0.0, 4.0, 4.0, 8.0),
        (4.0, 4.0, 8.0, 8.0),
        (12.0, 4.0, 4.0, 8.0),
        (0.0, 12.0, 4.0, 4.0),
        (4.0, 12.0, 8.0, 4.0),
        (12.0, 12.0, 4.0, 4.0),
    ];
    let own = light.get(tx, ty);
    let mut quads = 0;
    if own[0].max(own[1]).max(own[2]) > high_threshold {
        let slices = light.slice9(tx, ty);
        for (i, &(x, y, w, h)) in SLICE_POS.iter().enumerate() {
            let c = style.shade(slices[i], tx, ty, time);
            r.push_sprite(
                [pos[0] + x, pos[1] + y],
                [w, h],
                sub_uv(uv, x, y, w, h),
                [c[0], c[1], c[2], 1.0],
            );
            quads += 1;
        }
    } else {
        let slices = light.quad4(tx, ty);
        const QUAD_POS: [(f32, f32); 4] = [(0.0, 0.0), (8.0, 0.0), (0.0, 8.0), (8.0, 8.0)];
        for (i, &(x, y)) in QUAD_POS.iter().enumerate() {
            let c = style.shade(slices[i], tx, ty, time);
            r.push_sprite(
                [pos[0] + x, pos[1] + y],
                [8.0, 8.0],
                sub_uv(uv, x, y, 8.0, 8.0),
                [c[0], c[1], c[2], 1.0],
            );
            quads += 1;
        }
    }
    quads
}

/// `TileDrawing.DrawTrees`: canopy and branch sprites anchored to the transparent
/// marker tiles of a tree. The pass scans a slightly larger rect than the tile loop
/// because an 80x80 canopy sticks out above and beside its anchor.
#[allow(clippy::too_many_arguments)]
/// Draws the starter player from the original 40x56 part strips. The composition
/// mirrors `PlayerDrawLayers`: legs/pants/shoes, torso/shirt, head/eyes/hair, all at
/// `(x-10, y-10)` so the 40x56 sprite is centred on the 20x42 hitbox.
/// `PlayerDrawLayers` first pass: the selected block in the player's hand, with a
/// `Player.itemAnimation`-style swing when mining, placing or attacking.
#[allow(clippy::too_many_arguments)]
fn draw_held_item(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    light: &LightMap,
    style: &LightStyle,
    player: &Player,
    kind: u16,
    swing: f32,
    anim_time: f32,
) -> u32 {
    let uv = if let Some((uv, _, _)) = lookups.item_icon(kind) {
        Some(uv)
    } else if registry::is_weapon(kind) {
        lookups.shuriken_uv()
    } else if let Some(wall) = registry::place_wall(kind) {
        lookups.wall_icon_uv(wall)
    } else {
        registry::place_tile(kind).and_then(|t| lookups.plain_tex(t).map(|f| f.uv))
    };
    let Some(uv) = uv else {
        return 0;
    };
    let (cx, cy) = player.center();
    let tx = (cx / TILE_PX) as i32;
    let ty = (cy / TILE_PX) as i32;
    let c = style.shade(light.get(tx, ty), tx, ty, anim_time);
    let dir = player.facing as f32;
    let progress = if swing > 0.0 {
        1.0 - (swing / SWING_TIME).clamp(0.0, 1.0)
    } else {
        0.0
    };
    // Arc from raised-behind to forward-down; mirrored for the left hand.
    let a = if progress > 0.0 {
        (-1.7 + progress * 2.5) * dir
    } else {
        0.0
    };
    let hand = [player.x + player::WIDTH * 0.5 + 5.0 * dir, player.y + 27.0];
    let reach = 10.0;
    let center = [hand[0] + a.cos() * reach * dir, hand[1] + a.sin() * reach];
    // `push_sprite_rot` rotates around the top-left, so offset the pivot by the
    // rotated half-size to spin the block around its centre.
    let half = 7.0f32;
    let (s, co) = a.sin_cos();
    let top_left = [
        center[0] - (half * co - half * s),
        center[1] - (half * s + half * co),
    ];
    r.push_sprite_rot(
        top_left,
        [half * 2.0, half * 2.0],
        uv,
        [c[0], c[1], c[2], 1.0],
        a,
    );
    1
}

fn draw_player(
    r: &mut SpriteRenderer,
    assets: &PlayerAssets,
    player: &Player,
    light: &LightMap,
    style: &LightStyle,
    anim_time: f32,
    camera: Vec2,
    zoom: f32,
    screen: Vec2,
) -> u32 {
    let (cx, cy) = player.center();
    let tile_x = (cx / TILE_PX) as i32;
    let tile_y = (cy / TILE_PX) as i32;
    let l = light.get(tile_x, tile_y);
    let c = style.shade(l, tile_x, tile_y, anim_time);
    let light_c = [c[0], c[1], c[2]];
    let mul = |part: [f32; 3]| {
        [
            light_c[0] * part[0],
            light_c[1] * part[1],
            light_c[2] * part[2],
            1.0,
        ]
    };
    let skin = [1.0, 125.0 / 255.0, 90.0 / 255.0];
    let eye_white = [1.0, 1.0, 1.0];
    let eye = [105.0 / 255.0, 90.0 / 255.0, 75.0 / 255.0];
    let shirt = [175.0 / 255.0, 165.0 / 255.0, 140.0 / 255.0];
    let pants = [255.0 / 255.0, 230.0 / 255.0, 175.0 / 255.0];
    let shoes = [160.0 / 255.0, 105.0 / 255.0, 60.0 / 255.0];
    let hair = [215.0 / 255.0, 90.0 / 255.0, 55.0 / 255.0];

    let pos = snap_to_screen([player.x - 10.0, player.y - 10.0], camera, zoom, screen);
    let size = [40.0, 56.0];
    let flip = player.facing < 0;
    let leg = player.leg_frame();
    let mut quads = 0;
    let (arm_col, arm_row) = arm_cell(leg);
    let frame_uv = |t: Option<&BgTex>| t.map(|t| flip_uv(player_frame_uv(t, leg), flip));
    let cell_uv = |t: Option<&BgTex>, col: u32, row: u32| {
        t.map(|t| flip_uv(player_cell_uv(t, col, row), flip))
    };
    let mut part = |tex: Option<&BgTex>, uv: Option<[f32; 4]>, color: [f32; 4]| {
        if tex.is_some() {
            if let Some(uv) = uv {
                r.push_sprite(pos, size, uv, color);
                quads += 1;
            }
        }
    };
    part(assets.legs.as_ref(), frame_uv(assets.legs.as_ref()), mul(skin));
    part(assets.pants.as_ref(), frame_uv(assets.pants.as_ref()), mul(pants));
    part(assets.shoes.as_ref(), frame_uv(assets.shoes.as_ref()), mul(shoes));
    // Back arm behind the torso, front arm in front of the shirt (composite layers).
    part(
        assets.arm.as_ref(),
        cell_uv(assets.arm.as_ref(), arm_col, (arm_row + 2).min(3)),
        mul(skin),
    );
    // `PlayerDrawSet`: the torso/undershirt composite is always frame (0,0);
    // only the arms and head follow the animation frame.
    part(assets.torso.as_ref(), cell_uv(assets.torso.as_ref(), 0, 0), mul(skin));
    part(assets.shirt.as_ref(), cell_uv(assets.shirt.as_ref(), 0, 0), mul(shirt));
    // The frame table's arm sits on the far side of the body; column 8 is the
    // near (item-holding) arm, which the original places with the composite
    // shoulder rotation we do not model yet.
    part(
        assets.arm.as_ref(),
        cell_uv(assets.arm.as_ref(), 8, arm_row),
        mul(skin),
    );
    part(assets.head.as_ref(), frame_uv(assets.head.as_ref()), mul(skin));
    part(
        assets.eye_whites.as_ref(),
        frame_uv(assets.eye_whites.as_ref()),
        mul(eye_white),
    );
    part(assets.eyes.as_ref(), frame_uv(assets.eyes.as_ref()), mul(eye));
    part(assets.hair.as_ref(), frame_uv(assets.hair.as_ref()), mul(hair));
    quads
}

/// First enemy pass: the blue slime, tinted and lit like the player, with an overhead
/// health bar once damaged.
#[allow(clippy::too_many_arguments)]
fn draw_npcs(
    r: &mut SpriteRenderer,
    assets: &NpcAssets,
    npcs: &[Npc],
    light: &LightMap,
    style: &LightStyle,
    anim_time: f32,
    camera: Vec2,
    zoom: f32,
    screen: Vec2,
) -> u32 {
    let white = r.white_uv();
    let blue = [0.52, 0.72, 1.0];
    let mut quads = 0;
    for n in npcs {
        let (cx, cy) = n.center();
        let tx = (cx / TILE_PX) as i32;
        let ty = (cy / TILE_PX) as i32;
        let l = light.get(tx, ty);
        let c = style.shade(l, tx, ty, anim_time);
        let (w, _) = n.kind.size();
        let (tex, frame_h, size, pos) = match n.kind {
            NpcKind::Slime => (
                assets.slime.as_ref(),
                26.0,
                [32.0, 26.0],
                // The 26 px frame has 2 transparent rows below the feet; align the
                // art bottom (row 23) with the hitbox bottom.
                [n.x - 1.0, n.y + 1.0],
            ),
            NpcKind::DemonEye => (
                assets.eye.as_ref(),
                24.0,
                [38.0, 24.0],
                [n.x - 4.0, n.y + 2.0],
            ),
            NpcKind::Zombie => (
                assets.zombie.as_ref(),
                48.0,
                [38.0, 48.0],
                [n.x - 10.0, n.y - 5.0],
            ),
            NpcKind::KingSlime => (
                assets.king_slime.as_ref(),
                120.0,
                [174.0, 120.0],
                [n.x - 38.0, n.y - 25.0],
            ),
        };
        let Some(tex) = tex else {
            continue;
        };
        let base = if n.kind == NpcKind::Slime {
            [c[0] * blue[0], c[1] * blue[1], c[2] * blue[2], 1.0]
        } else {
            [c[0], c[1], c[2], 1.0]
        };
        let tint = if n.hurt_cd > 0.0 {
            [c[0].max(0.8), c[1].max(0.8), c[2].max(0.8), 1.0]
        } else {
            base
        };
        let uv = flip_uv(npc_frame_uv(tex, frame_h, n.frame()), n.facing < 0);
        r.push_sprite(snap_to_screen(pos, camera, zoom, screen), size, uv, tint);
        quads += 1;
        if n.hp < n.max_hp {
            let frac = (n.hp as f32 / n.max_hp as f32).clamp(0.0, 1.0);
            r.push_sprite([n.x, n.y - 9.0], [w, 4.0], white, [0.02, 0.02, 0.02, 0.75]);
            r.push_sprite(
                [n.x, n.y - 9.0],
                [w * frac, 4.0],
                white,
                [0.85, 0.15, 0.15, 0.95],
            );
            quads += 2;
        }
    }
    quads
}

fn draw_tree_foliage(
    r: &mut SpriteRenderer,
    world: &World,
    lookups: &assets::Lookups,
    light: &LightMap,
    style: &LightStyle,
    anim_time: f32,
    x0: i32,
    y0: i32,
    x1: i32,
    y1: i32,
) -> u32 {
    let fx0 = (x0 - 4).max(0);
    let fx1 = (x1 + 4).min(world.width - 1);
    let fy0 = (y0 - 5).max(0);
    let fy1 = (y1 + 1).min(world.height - 1);
    let mut quads = 0;
    for ty in fy0..=fy1 {
        for tx in fx0..=fx1 {
            let Some(t) = world.tile(tx, ty) else {
                continue;
            };
            if !t.active() || !tree::is_tree(t.kind) {
                continue;
            }
            let Some(part) = tree::TreePart::marker(t.frame_x, t.frame_y) else {
                continue;
            };
            let f = tree::foliage_at(world, tx, ty, part, t.frame_y);
            if lookups.foliage.get(f.style).is_none_or(|s| s.is_none()) {
                continue;
            }
            let c = style.shade(light.get(tx, ty), tx, ty, anim_time);
            let color = [c[0], c[1], c[2], 1.0];
            let px = tx as f32 * TILE_PX;
            let py = ty as f32 * TILE_PX;
            match part {
                tree::TreePart::Top => {
                    let sx = f.frame as i32 * (f.top_w as i32 + 2);
                    if let Some(uv) =
                        lookups.foliage_uv(f.style, true, sx, 0, f.top_w as u32, f.top_h as u32)
                    {
                        r.push_sprite(
                            [px + 8.0 - f.top_w * 0.5, py + 16.0 - f.top_h],
                            [f.top_w, f.top_h],
                            uv,
                            color,
                        );
                        quads += 1;
                    }
                }
                tree::TreePart::BranchLeft => {
                    if let Some(uv) =
                        lookups.foliage_uv(f.style, false, 0, f.frame as i32 * 42, 40, 40)
                    {
                        r.push_sprite([px - 24.0, py - 12.0], [40.0, 40.0], uv, color);
                        quads += 1;
                    }
                }
                tree::TreePart::BranchRight => {
                    if let Some(uv) =
                        lookups.foliage_uv(f.style, false, 42, f.frame as i32 * 42, 40, 40)
                    {
                        r.push_sprite([px, py - 12.0], [40.0, 40.0], uv, color);
                        quads += 1;
                    }
                }
            }
        }
    }
    quads
}

/// `DynamicSpriteFont.DrawString`: the pen advances by `spacing + kerning.X`, the
/// glyph is drawn at `pen + padding`, then the pen advances by `kerning.Y + kerning.Z`.
fn push_font_text(
    r: &mut SpriteRenderer,
    font: &assets::UiFont,
    x: f32,
    y: f32,
    scale: f32,
    color: [f32; 4],
    text: &str,
) {
    let mut pen = Vec2::new(x, y);
    let mut first = true;
    for c in text.chars() {
        match c {
            '\n' => {
                pen.x = x;
                pen.y += font.line_spacing * scale;
                first = true;
                continue;
            }
            '\r' => continue,
            _ => {}
        }
        let Some(g) = font.glyph(c) else { continue };
        let mut bearing = g.kerning[0];
        if first {
            bearing = bearing.max(0.0);
        } else {
            pen.x += font.spacing * scale;
        }
        pen.x += bearing * scale;
        r.push_ui_sprite(
            [pen.x + g.pad[0] * scale, pen.y + g.pad[1] * scale],
            [g.w * scale, g.h * scale],
            g.uv,
            color,
        );
        pen.x += (g.kerning[1] + g.kerning[2]) * scale;
        first = false;
    }
}

/// `DynamicSpriteFont.MeasureString`.
fn font_width(font: &assets::UiFont, text: &str, scale: f32) -> f32 {
    let mut w = 0.0;
    let mut max_z = 0.0f32;
    let mut first = true;
    for c in text.chars() {
        if c == '\n' {
            break;
        }
        if c == '\r' {
            continue;
        }
        let Some(g) = font.glyph(c) else { continue };
        let mut bearing = g.kerning[0];
        if first {
            bearing = bearing.max(0.0);
        } else {
            w += (font.spacing + max_z) * scale;
        }
        w += (bearing + g.kerning[1]) * scale;
        max_z = g.kerning[2];
        first = false;
    }
    w + max_z.max(0.0) * scale
}

/// UI label with the original `Mouse_Text` font, or the built-in bitmap font when
/// the retail fonts are unavailable (PNG-only fallback).
fn hud_text(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    x: f32,
    y: f32,
    scale: f32,
    color: [f32; 4],
    text: &str,
) {
    match &lookups.ui_font {
        Some(f) => push_font_text(r, f, x, y, scale, color, text),
        None => r.push_text(x, y, scale, color, text),
    }
}

fn hud_item_text(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    x: f32,
    y: f32,
    scale: f32,
    color: [f32; 4],
    text: &str,
) {
    match &lookups.item_font {
        Some(f) => push_font_text(r, f, x, y, scale, color, text),
        None => r.push_text(x, y, scale, color, text),
    }
}

fn hud_text_width(r: &SpriteRenderer, lookups: &assets::Lookups, text: &str, scale: f32) -> f32 {
    match &lookups.ui_font {
        Some(f) => font_width(f, text, scale),
        None => r.text_width(text, scale),
    }
}

fn hud_item_text_width(
    r: &SpriteRenderer,
    lookups: &assets::Lookups,
    text: &str,
    scale: f32,
) -> f32 {
    match &lookups.item_font {
        Some(f) => font_width(f, text, scale),
        None => r.text_width(text, scale),
    }
}

/// `ItemSlot.Draw`: the item's own texture at inventory scale, centred in the slot,
/// with the stack size in the bottom-right corner.
fn draw_slot_item(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    item: &ItemSlot,
    x: f32,
    y: f32,
    slot: f32,
    scale: f32,
) {
    if item.count == 0 {
        return;
    }
    let (icon_uv, iw, ih) = match lookups.item_icon(item.kind) {
        Some(v) => v,
        None => match lookups.plain_tex(item.kind) {
            Some(t) => (t.uv, 16.0, 16.0),
            None => return,
        },
    };
    let w = iw * scale;
    let h = ih * scale;
    r.push_ui_sprite(
        [x + (slot - w) * 0.5, y + (slot - h) * 0.5],
        [w, h],
        icon_uv,
        [1.0; 4],
    );
    let count = format!("{}", item.count);
    let fs = 0.75 * scale;
    let tw = hud_item_text_width(r, lookups, &count, fs);
    let tx = x + slot - tw - 4.0 * scale;
    let ty = y + slot - 11.0 * scale;
    hud_item_text(r, lookups, tx + 1.0, ty + 1.0, fs, [0.0, 0.0, 0.0, 1.0], &count);
    hud_item_text(r, lookups, tx, ty, fs, [1.0, 1.0, 1.0, 1.0], &count);
}

/// The hotbar digits (`FontAssets.ItemStack` in the original): grey by default,
/// white at 1.4x on the selected slot, and slot 10 is drawn as "0".
fn draw_slot_number(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    x: f32,
    y: f32,
    i: usize,
    scale: f32,
    selected: bool,
) {
    let mut num = format!("{}", i + 1);
    if num == "10" {
        num = "0".into();
    }
    let (fs, dy, col) = if selected {
        (scale * 1.4, -2.0 * scale, [1.0, 1.0, 1.0, 200.0 / 255.0])
    } else {
        (scale, 0.0, [0.86, 0.86, 0.86, 1.0])
    };
    let px = x + 6.0 * scale;
    let py = y + 4.0 * scale + dy;
    hud_item_text(r, lookups, px + 1.0, py + 1.0, fs, [0.0, 0.0, 0.0, 1.0], &num);
    hud_item_text(r, lookups, px, py, fs, col, &num);
}

/// Vanilla HUD layout: hotbar top-left (or the open 10x5 inventory grid), hearts
/// top-right, breath bubbles above the player and the boss bar at the bottom centre
/// (`GUIHotbarDrawInner`, `Main.DrawInventory`, `ClassicPlayerResourcesDisplaySet`,
/// `BigProgressBarHelper`).
#[allow(clippy::too_many_arguments)]
fn draw_hud(
    r: &mut SpriteRenderer,
    camera: Vec2,
    zoom: f32,
    screen: Vec2,
    lookups: &assets::Lookups,
    inventory: &[ItemSlot; INV],
    selected: usize,
    hotbar_scales: &[f32; HOTBAR],
    player: Option<&Player>,
    boss: Option<(i32, i32)>,
    inventory_open: bool,
    mouse_item: &ItemSlot,
    mouse_pos: Vec2,
) {
    // `ItemSlot.Draw` / `Main.DrawInventory`: with the inventory open the hotbar is
    // row 0 of a 10x5 grid of 56 px cells at (20,20) at `inventoryScale = 0.85`
    // (context 0: grey `InventoryBack`, hotbar row `InventoryBack9`); closed, the ten
    // slots animate between 0.75 and 1.0 (context 13: grey back at 200/255, the
    // selected one yellow `InventoryBack14` at full white).
    if inventory_open {
        let inv_scale = 0.85f32;
        if let Some(back) = lookups.hud_uv(lookups.ui_back.as_ref()) {
            let hot_back = lookups.hud_uv(lookups.ui_back_hotbar.as_ref()).unwrap_or(back);
            for j in 0..5usize {
                for i in 0..10usize {
                    let idx = i + j * 10;
                    let x = 20.0 + i as f32 * 56.0 * inv_scale;
                    let y = 20.0 + j as f32 * 56.0 * inv_scale;
                    let slot = 52.0 * inv_scale;
                    let uv = if j == 0 { hot_back } else { back };
                    let c = 220.0 / 255.0;
                    r.push_ui_sprite([x, y], [slot, slot], uv, [c, c, c, c]);
                    let item = &inventory[idx];
                    draw_slot_item(r, lookups, item, x, y, slot, inv_scale);
                    if j == 0 {
                        draw_slot_number(
                            r,
                            lookups,
                            x,
                            y,
                            i,
                            inv_scale,
                            idx == selected.min(HOTBAR - 1),
                        );
                    }
                }
            }
        }
    } else if let Some(back) = lookups.hud_uv(lookups.ui_back.as_ref()) {
        let sel_back = lookups.hud_uv(lookups.ui_back_sel.as_ref()).unwrap_or(back);
        let mut x = 20.0;
        for i in 0..HOTBAR {
            let scale = hotbar_scales[i].clamp(0.75, 1.0);
            let slot = 52.0 * scale;
            let y = 20.0 + 22.0 * (1.0 - scale);
            let (uv, c) = if i == selected {
                (sel_back, 255.0 / 255.0)
            } else {
                (back, 200.0 / 255.0)
            };
            r.push_ui_sprite([x, y], [slot, slot], uv, [c, c, c, c]);
            draw_slot_item(r, lookups, &inventory[i], x, y, slot, scale);
            draw_slot_number(r, lookups, x, y, i, scale, i == selected);
            x += slot + 4.0;
        }
    }
    // Selected item name, centred above the bar like the original.
    let item = &inventory[selected.min(HOTBAR - 1)];
    if item.count > 0 && !inventory_open {
        let label = item.name;
        let tw = hud_text_width(r, lookups, label, 1.0);
        let tx = 236.0 - tw * 0.5;
        hud_text(r, lookups, tx + 1.0, 1.0, 1.0, [0.0, 0.0, 0.0, 1.0], label);
        hud_text(r, lookups, tx, 0.0, 1.0, [1.0, 0.95, 0.85, 1.0], label);
    }

    // `Main.mouseItem`: the stack carried on the cursor, centred on it.
    if mouse_item.count > 0 {
        draw_slot_item(
            r,
            lookups,
            mouse_item,
            mouse_pos.x - 26.0,
            mouse_pos.y - 26.0,
            52.0,
            1.0,
        );
    }

    // Hearts: 20 HP per heart, anchored to the right like the classic display.
    if let Some(p) = player {
        if let Some(heart) = lookups.hud_uv(lookups.ui_heart.as_ref()) {
            let hearts = ((p.max_hp as f32) / 20.0).ceil().max(1.0) as i32;
            let x0 = screen.x - 300.0;
            for i in 1..=hearts {
                let ratio = ((p.hp as f32 - (i - 1) as f32 * 20.0) / 20.0).clamp(0.0, 1.0);
                let (grey, scale) = if ratio >= 1.0 {
                    (255.0f32, 1.0f32)
                } else {
                    (
                        (30.0 + 225.0 * ratio).max(30.0),
                        (ratio / 4.0 + 0.75).max(0.75),
                    )
                };
                let size = 22.0 * scale;
                let x = x0 + 26.0 * (i - 1) as f32;
                let y = 32.0 + (22.0 - size) * 0.5;
                let t = grey / 255.0;
                r.push_ui_sprite([x, y], [size, size], heart, [t, t, t, 1.0]);
            }
            let label = "Life";
            let value = format!("{}/{}", p.hp, p.max_hp);
            let center = x0 + 13.0 * hearts as f32;
            let lw = hud_text_width(r, lookups, label, 1.0);
            hud_text(r, lookups, center - 3.0 - lw + 1.0, 7.0, 1.0, [0.0, 0.0, 0.0, 1.0], label);
            hud_text(r, lookups, center - 3.0 - lw, 6.0, 1.0, [1.0, 1.0, 1.0, 1.0], label);
            let vx = center + 3.0;
            hud_text(r, lookups, vx + 1.0, 7.0, 1.0, [0.0, 0.0, 0.0, 1.0], &value);
            hud_text(r, lookups, vx, 6.0, 1.0, [1.0, 1.0, 1.0, 1.0], &value);

            // Breath bubbles above the player's head while submerged.
            if p.breath < BREATH_MAX {
                if let Some(bubble) = lookups.hud_uv(lookups.ui_bubble.as_ref()) {
                    let psx = (p.x + PLAYER_WIDTH * 0.5 - camera.x) * zoom + screen.x * 0.5;
                    let psy = (p.y - camera.y) * zoom + screen.y * 0.5;
                    let bx = psx - 125.0;
                    let by = psy - 100.0;
                    for i in 1..=10 {
                        let ratio =
                            ((p.breath as f32 - (i - 1) as f32 * 20.0) / 20.0).clamp(0.0, 1.0);
                        let (grey, scale) = if p.breath >= i * 20 {
                            (255.0f32, 1.0f32)
                        } else {
                            (
                                (30.0 + 225.0 * ratio).max(30.0),
                                (ratio / 4.0 + 0.75).max(0.75),
                            )
                        };
                        let size = 22.0 * scale;
                        let x = bx + 26.0 * (i - 1) as f32;
                        let y = by + (22.0 - size) * 0.5;
                        let t = grey / 255.0;
                        r.push_ui_sprite([x, y], [size, size], bubble, [t, t, t, 1.0]);
                    }
                }
            }
        }
    }

    // Boss bar: `BigProgressBarHelper.DrawFancyBar` (456x22 bar at the bottom).
    if let Some((hp, max_hp)) = boss {
        let frac = (hp as f32 / max_hp.max(1) as f32).clamp(0.0, 1.0);
        let center = [screen.x * 0.5 - 228.0, screen.y - 61.0];
        let top_left = [center[0] - 32.0, center[1] - 24.0];
        if let (Some(back), Some(fill), Some(cap), Some(overlay)) = (
            lookups.boss_bar_uv(3, 0, 0, 516, 58),
            lookups.boss_bar_uv(2, 32, 24, 2, 22),
            lookups.boss_bar_uv(1, 32, 24, 2, 22),
            lookups.boss_bar_uv(0, 0, 0, 516, 58),
        ) {
            r.push_ui_sprite(top_left, [516.0, 58.0], back, [1.0, 1.0, 1.0, 0.2]);
            let width = ((456.0 * frac) as i32) & !1;
            if width > 0 {
                r.push_ui_sprite(
                    center,
                    [width as f32, 22.0],
                    fill,
                    [1.0, 1.0, 1.0, 1.0],
                );
                r.push_ui_sprite(
                    [center[0] + width as f32 - 2.0, center[1]],
                    [2.0, 22.0],
                    cap,
                    [1.0; 4],
                );
            }
            r.push_ui_sprite(top_left, [516.0, 58.0], overlay, [1.0; 4]);
            if let Some(icon) = lookups.hud_uv(lookups.ui_boss_head.as_ref()) {
                r.push_ui_sprite(
                    [top_left[0] + 2.0, top_left[1] + 19.0],
                    [30.0, 30.0],
                    icon,
                    [1.0; 4],
                );
            }
            let label = format!("{hp}/{max_hp}");
            let tw = r.text_width(&label, 1.0);
            let text_x = center[0] + 228.0 - tw * 0.5;
            r.push_text(
                text_x + 1.0,
                center[1] + 8.0,
                1.0,
                [0.0, 0.0, 0.0, 1.0],
                &label,
            );
            r.push_text(
                text_x,
                center[1] + 7.0,
                1.0,
                [1.0, 1.0, 1.0, 1.0],
                &label,
            );
        }
    }
}

fn draw_debug(
    r: &mut SpriteRenderer,
    screen: Vec2,
    tile: TileDrawStats,
    history: &VecDeque<f32>,
    camera: Vec2,
    zoom: f32,
    last_stats: tr_render::sprite::RenderStats,
    world: &World,
    atlas_stats: (u32, u32, usize, usize),
    gpu: &Gpu,
    dt: f32,
    sky: [f32; 3],
    cache: &mut DebugCache,
    light_mode: &str,
    player: Option<&Player>,
    npc_count: usize,
    liquid_active: usize,
    rain_alpha: f32,
    rain_drops: usize,
    snow_flakes: usize,
) {
    if !cache.ready {
        let info = gpu.adapter.get_info();
        cache.gpu_line = format!(
            "gpu {} [{:?}]   sheets {}",
            info.name, info.backend, atlas_stats.3
        );
        cache.world_line = format!(
            "world '{}'   tile mem {:.0} MB   atlas {}x{} ({:.0} MB)",
            display_name(&world.name),
            world.memory_bytes() as f32 / 1e6,
            atlas_stats.0,
            atlas_stats.1,
            atlas_stats.2 as f32 / 1e6,
        );
        cache.ready = true;
    }
    let scale = 2.0;
    let line_h = 18.0;
    let mut lines: Vec<(String, [f32; 4])> = Vec::new();

    let avg = if history.is_empty() {
        0.0
    } else {
        history.iter().sum::<f32>() / history.len() as f32
    };
    let fps = if avg > 0.0 { 1000.0 / avg } else { 0.0 };
    let worst = history.iter().copied().fold(0.0f32, f32::max);

    lines.push(("TERRARIA-RUST  //  native rust + vulkan".into(), ACCENT));
    lines.push((
        format!(
            "fps {fps:6.1}   frame {avg:5.2} ms   worst {worst:5.2} ms   dt {:5.2}",
            dt * 1000.0
        ),
        TEXT,
    ));
    lines.push((
        format!(
            "draws {}   instances {}   upload {:.1} KB",
            last_stats.draw_calls,
            last_stats.instances,
            last_stats.uploaded_bytes as f32 / 1024.0
        ),
        TEXT,
    ));
    lines.push((
        format!(
            "camera {:.0},{:.0} px   tile {:.0},{:.0}   zoom {:.2}",
            camera.x,
            camera.y,
            camera.x / TILE_PX,
            camera.y / TILE_PX,
            zoom
        ),
        TEXT,
    ));
    if let Some(p) = player {
        lines.push((
            format!(
                "player {:.0},{:.0} tile {:.1},{:.1} vx {:.2} vy {:.2} ground {} water {} lava {} hp {}/{} breath {}/{}",
                p.x,
                p.y,
                p.x / TILE_PX,
                p.y / TILE_PX,
                p.vx,
                p.vy,
                p.on_ground,
                p.in_water,
                p.in_lava,
                p.hp,
                p.max_hp,
                p.breath,
                player::BREATH_MAX
            ),
            TEXT,
        ));
    }
    lines.push((
        format!(
            "npcs {npc_count}   liquid cells {liquid_active}   rain {rain_alpha:.2} ({rain_drops} drops, {snow_flakes} snow)"
        ),
        TEXT,
    ));
    lines.push((
        format!(
            "time {:.0} {}   day length 15 min / night 9 min",
            world.time,
            if world.day_time { "day" } else { "night" }
        ),
        TEXT,
    ));
    lines.push((
        format!(
            "visible tiles {}   quads {}   foliage {}   liquids {}   items {}   total {}   grid {}x{}",
            tile.visible,
            tile.quads,
            tile.foliage,
            tile.liquids,
            tile.items,
            world.tiles.len(),
            world.width,
            world.height
        ),
        TEXT,
    ));
    lines.push((
        format!(
            "walls {}   culled {}   dark cells {} -> {} rects   chunks skipped {}",
            tile.walls, tile.walls_culled, tile.dark_cells, tile.dark_rects, tile.chunks_skipped
        ),
        TEXT,
    ));
    lines.push((
        format!(
            "draw {:.2} ms ({})   light {} {} seeded   {:.2} ms   sky {:.2},{:.2},{:.2}",
            tile.draw_ms,
            if tile.world_reused { "reused" } else { "built" },
            light_mode,
            tile.lit_tiles,
            tile.light_ms,
            sky[0],
            sky[1],
            sky[2]
        ),
        TEXT,
    ));
    lines.push((cache.world_line.clone(), TEXT));
    lines.push((cache.gpu_line.clone(), TEXT));
    lines.push((
        format!("screen {:.0}x{:.0}   atlas_max 8192", screen.x, screen.y),
        TEXT,
    ));
    lines.push((
        "LMB mine/attack   RMB place   1-4 select   F9 save".into(),
        WARN,
    ));
    lines.push((
        "WASD move   Space jump   F1 ui   F2 light   F5 respawn".into(),
        WARN,
    ));

    let panel_w = 760.0;
    let graph_h = 48.0;
    let panel_h = 14.0 + lines.len() as f32 * line_h + graph_h + 10.0;
    let x = 14.0;
    let y = 14.0;

    r.push_ui_rect([x, y], [panel_w, panel_h], PANEL_BG);

    let mut ty = y + 10.0;
    for (text, color) in &lines {
        r.push_text(x + 12.0, ty, scale, *color, text);
        ty += line_h;
    }

    let gx = x + 12.0;
    let gy = y + panel_h - graph_h - 6.0;
    r.push_ui_rect([gx, gy], [panel_w - 24.0, graph_h], [0.0, 0.0, 0.0, 0.45]);
    let bar_w = (panel_w - 24.0) / 240.0;
    let max_ms = 33.0f32;
    for (i, ms) in history.iter().enumerate() {
        let h = (ms / max_ms).clamp(0.0, 1.0) * (graph_h - 4.0);
        let col = if *ms < 16.7 {
            [0.35, 0.85, 0.45, 0.95]
        } else if *ms < 33.0 {
            [0.95, 0.75, 0.25, 0.95]
        } else {
            [0.95, 0.30, 0.25, 0.95]
        };
        r.push_ui_rect(
            [gx + i as f32 * bar_w, gy + graph_h - 2.0 - h],
            [bar_w.max(1.0) - 0.5, h],
            col,
        );
    }
}

fn capture_frame(gpu: &Gpu, texture: &wgpu::Texture, path: &std::path::Path) {
    let w = gpu.config.width;
    let h = gpu.config.height;
    let bpr = (w * 4).div_ceil(256) * 256;
    let buffer = gpu.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("screenshot"),
        size: (bpr as u64) * (h as u64),
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });

    let mut enc = gpu
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("screenshot"),
        });
    enc.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo {
            texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::TexelCopyBufferInfo {
            buffer: &buffer,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(bpr),
                rows_per_image: Some(h),
            },
        },
        wgpu::Extent3d {
            width: w,
            height: h,
            depth_or_array_layers: 1,
        },
    );
    gpu.queue.submit(Some(enc.finish()));

    let slice = buffer.slice(..);
    slice.map_async(wgpu::MapMode::Read, |r| {
        r.expect("map screenshot buffer");
    });
    let _ = gpu.device.poll(wgpu::PollType::wait_indefinitely());

    let data = slice.get_mapped_range().expect("mapped range");
    let mut rgba = Vec::with_capacity((w * h * 4) as usize);
    let bgra = matches!(
        gpu.config.format,
        wgpu::TextureFormat::Bgra8Unorm | wgpu::TextureFormat::Bgra8UnormSrgb
    );
    for row in 0..h {
        let start = (row * bpr) as usize;
        let row_data = &data[start..start + (w * 4) as usize];
        if bgra {
            for px in row_data.chunks_exact(4) {
                rgba.extend_from_slice(&[px[2], px[1], px[0], px[3]]);
            }
        } else {
            rgba.extend_from_slice(row_data);
        }
    }
    drop(data);

    image::save_buffer(path, &rgba, w, h, image::ColorType::Rgba8).expect("save screenshot");
    log::info!("screenshot saved to {}", path.display());
}

/// Compresses a world for the `World` packet (zlib, speed over ratio: the tile array
/// is mostly empty cells and compresses well).
fn compressed_world(world: &World) -> Vec<u8> {
    use std::io::Write;
    let bytes = save::to_bytes(world).unwrap_or_default();
    let mut enc = flate2::write::ZlibEncoder::new(Vec::new(), flate2::Compression::fast());
    let _ = enc.write_all(&bytes);
    enc.finish().unwrap_or_default()
}

fn decompressed_world(data: &[u8]) -> Option<World> {
    use std::io::Read;
    let mut out = Vec::new();
    let mut dec = flate2::read::ZlibDecoder::new(data);
    dec.read_to_end(&mut out).ok()?;
    save::parse(&out).ok()
}

fn remote_world_path() -> PathBuf {
    save::default_world_path("Remote").unwrap_or_else(|| PathBuf::from("Remote.trwld"))
}

/// Joining receives the host world; saving it and relaunching is the simplest way to
/// rebuild every world-derived system (atlas, liquids, lighting) without tearing down
/// the renderer mid-frame. The child process auto-connects through `TR_JOIN`.
fn relaunch_into_world(data: &[u8], addr: &str, password: &str, name: &str) {
    let Some(world) = decompressed_world(data) else {
        log::warn!("received a world that failed to decode");
        return;
    };
    let path = remote_world_path();
    if let Err(e) = save::save(&path, &world) {
        log::warn!("saving the joined world failed: {e:#}");
        return;
    }
    let exe = std::env::current_exe().unwrap_or_default();
    let mut cmd = std::process::Command::new(exe);
    cmd.env("TR_JOIN", addr)
        .env("TR_JOIN_PW", password)
        .env("TR_WORLD", &path)
        .env("TR_NAME", name)
        .env("TR_JOINED", "1");
    match cmd.spawn() {
        Ok(_) => std::process::exit(0),
        Err(e) => log::warn!("relaunching failed: {e:#}"),
    }
}

/// Applies a replicated edit. `kind`: 0 break tile, 1 place tile, 2 break wall,
/// 3 place wall.
fn apply_edit(world: &mut World, x: i32, y: i32, kind: u8, tile: u16) {
    match kind {
        0 => {
            edit::break_tile(world, x, y);
        }
        1 => {
            edit::place_tile(world, x, y, tile);
        }
        2 => {
            edit::break_wall(world, x, y);
        }
        3 => {
            edit::place_wall(world, x, y, tile);
        }
        _ => {}
    }
}

fn npc_from_net(world: &World, n: &net::NpcNet) -> Option<crate::npc::Npc> {
    let kind = crate::npc::NpcKind::from_u16(n.kind)?;
    let tx = (n.x / TILE_PX) as i32;
    let mut npc = match kind {
        crate::npc::NpcKind::Slime => crate::npc::Npc::spawn_slime(world, tx)?,
        crate::npc::NpcKind::DemonEye => crate::npc::Npc::spawn_eye(world, tx)?,
        crate::npc::NpcKind::Zombie => crate::npc::Npc::spawn_zombie(world, tx)?,
        crate::npc::NpcKind::KingSlime => crate::npc::Npc::spawn_king_slime(world, tx)?,
    };
    npc.x = n.x;
    npc.y = n.y;
    npc.vx = n.vx;
    npc.vy = n.vy;
    npc.hp = n.hp;
    npc.max_hp = n.max_hp;
    npc.facing = n.facing;
    npc.on_ground = n.on_ground;
    npc.anim = n.anim;
    npc.hurt_cd = n.hurt_cd;
    Some(npc)
}

/// Builders for the host-side broadcast messages.
fn net_player(p: &net::PlayerNet) -> net::ServerToClient {
    net::ServerToClient::Player(p.clone())
}

fn net_edit(x: i32, y: i32, kind: u8, tile: u16) -> net::ServerToClient {
    net::ServerToClient::Edit {
        x,
        y,
        kind,
        id: tile,
    }
}

fn net_npcs(list: Vec<net::NpcNet>) -> net::ServerToClient {
    net::ServerToClient::Npcs(list)
}

fn net_items(items: &[item::ItemDrop]) -> net::ServerToClient {
    net::ServerToClient::Items(
        items
            .iter()
            .enumerate()
            .map(|(i, it)| net::ItemNet {
                id: i as u32,
                item: it.kind,
                count: it.count,
                x: it.pos.x,
                y: it.pos.y,
            })
            .collect(),
    )
}

fn broadcast_player(server: &net::Server, p: &net::PlayerNet) {
    server.broadcast(&net_player(p));
}

fn broadcast_left(server: &net::Server, id: u32) {
    server.broadcast(&net::ServerToClient::Left { id });
}

fn broadcast_joined(server: &net::Server, id: u32, players: &HashMap<u32, net::PlayerNet>) {
    let name = players.get(&id).map(|p| p.name.clone()).unwrap_or_default();
    server.broadcast(&net::ServerToClient::Joined { id, name });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pickup_prefers_stacks_then_empty_slots() {
        let mut inv: [ItemSlot; INV] = std::array::from_fn(|_| ItemSlot {
            kind: 0,
            name: "",
            count: 0,
        });
        inv[0] = ItemSlot {
            kind: 3,
            name: "Stone Block",
            count: 4,
        };
        assert_eq!(slot_for_pickup(&mut inv, 3), Some(0));
        assert_eq!(slot_for_pickup(&mut inv, 12), Some(1));
        assert_eq!(inv[1].kind, 12);
        assert_eq!(inv[1].name, "Copper Ore");
        assert_eq!(inv[1].count, 0);
        for slot in &mut inv {
            slot.count = 1;
        }
        assert_eq!(
            slot_for_pickup(&mut inv, 53),
            None,
            "full hotbar must refuse"
        );
    }

    #[test]
    fn day_night_clock_wraps() {
        let mut w = World::new("t".into(), 8, 8);
        w.time = 53999.0;
        w.day_time = true;
        advance_time(&mut w, 1.0);
        assert!(!w.day_time);
        assert!(w.time < 100.0, "time {}", w.time);
        w.time = 32399.0;
        advance_time(&mut w, 1.0);
        assert!(w.day_time);
        assert!(w.time < 100.0, "time {}", w.time);
    }
}

#[cfg(test)]
mod water_style_tests {
    use super::*;
    use tr_core::tile::FLAG_ACTIVE;

    fn test_world() -> World {
        let mut w = World::new("t".into(), 600, 800);
        w.world_surface = 200.0;
        w.rock_layer = 360.0;
        w.column_surface = vec![200; 600];
        w
    }

    fn put(w: &mut World, x: i32, kind: u16) {
        let i = w.index(x, 200);
        w.tiles[i].kind = kind;
        w.tiles[i].flags |= FLAG_ACTIVE;
    }

    fn cam(x: i32, y: i32) -> Vec2 {
        Vec2::new(x as f32 * TILE_PX, y as f32 * TILE_PX)
    }

    #[test]
    fn forest_surface_is_style_zero() {
        let w = test_world();
        assert_eq!(calculate_water_style(&w, cam(300, 150)), 0);
    }

    #[test]
    fn shallow_cave_is_seven_and_deep_cave_is_eight() {
        let w = test_world();
        // `TileCenter.Y > worldSurface` but not past rockLayer + 40 -> 7.
        assert_eq!(calculate_water_style(&w, cam(300, 250)), 7);
        // Past rockLayer + 40 -> 8.
        assert_eq!(calculate_water_style(&w, cam(300, 450)), 8);
    }

    #[test]
    fn snow_water_is_five() {
        let mut w = test_world();
        put(&mut w, 300, 147);
        assert_eq!(calculate_water_style(&w, cam(300, 150)), 5);
    }

    #[test]
    fn desert_water_is_six_and_twelve_underground() {
        let mut w = test_world();
        put(&mut w, 300, 53);
        assert_eq!(calculate_water_style(&w, cam(300, 150)), 6);
        assert_eq!(calculate_water_style(&w, cam(300, 250)), 12);
    }

    #[test]
    fn ocean_edges_use_thirteen() {
        let w = test_world();
        assert_eq!(calculate_water_style(&w, cam(100, 150)), 13);
        assert_eq!(calculate_water_style(&w, cam(550, 150)), 13);
    }
}

#[cfg(test)]
mod bg_transition_tests {
    use super::*;

    #[test]
    fn switching_into_a_non_forest_style_waits_thirty_frames() {
        // `DrawBG_HandleBackgroundTransition`: delay 30, switch when bgDelay > 30.
        let (mut style, mut delay) = (0, 0);
        for _ in 0..30 {
            advance_bg_style(&mut style, &mut delay, 7);
        }
        assert_eq!((style, delay), (0, 30));
        advance_bg_style(&mut style, &mut delay, 7);
        assert_eq!((style, delay), (7, -60));
    }

    #[test]
    fn switching_into_forest_waits_sixty_frames_and_resets_the_delay() {
        let (mut style, mut delay) = (7, 0);
        for _ in 0..60 {
            advance_bg_style(&mut style, &mut delay, 0);
        }
        assert_eq!((style, delay), (7, 60));
        advance_bg_style(&mut style, &mut delay, 0);
        assert_eq!((style, delay), (0, 0));
    }

    #[test]
    fn negative_delay_is_a_sixty_frame_cooldown() {
        let (mut style, mut delay) = (7, -60);
        for _ in 0..60 {
            advance_bg_style(&mut style, &mut delay, 7);
            assert_eq!(style, 7);
        }
        assert_eq!(delay, 0);
        // Returning to style 0 is a forest style, so it waits 60 frames.
        for _ in 0..31 {
            advance_bg_style(&mut style, &mut delay, 0);
        }
        assert_eq!((style, delay), (7, 31));
        for _ in 0..30 {
            advance_bg_style(&mut style, &mut delay, 0);
        }
        assert_eq!(style, 0);
    }

    #[test]
    fn keeping_the_style_decays_a_positive_delay() {
        let (mut style, mut delay) = (0, 12);
        advance_bg_style(&mut style, &mut delay, 0);
        assert_eq!((style, delay), (0, 11));
    }

    #[test]
    fn snow_cross_fade_moves_both_alpha_sets() {
        let mut front = [0.0; 16];
        let mut far = [0.0; 16];
        front[0] = 1.0;
        far[0] = 1.0;
        // 20 frames at `backgroundLayerTransitionSpeed = 0.05` completes a fade.
        for _ in 0..20 {
            advance_bg_alphas(&mut front, &mut far, 7, 0.05);
        }
        assert_eq!(front[0], 0.0);
        assert_eq!(far[0], 0.0);
        assert_eq!(front[7], 1.0);
        // `UpdateBGVisibility_BackLayer`: snow is far index 4.
        assert_eq!(far[4], 1.0);
        // No other slot may light up.
        assert_eq!(front.iter().filter(|a| **a != 0.0).count(), 1);
        assert_eq!(far.iter().filter(|a| **a != 0.0).count(), 1);
    }

    #[test]
    fn each_supported_style_selects_its_far_layer() {
        for (style, index) in [(0, 0), (2, 1), (4, 3), (7, 4)] {
            let mut front = [0.0; 16];
            let mut far = [0.0; 16];
            advance_bg_alphas(&mut front, &mut far, style, 0.05);
            assert_eq!(front[style as usize], 0.05, "front style {style}");
            assert_eq!(far[index], 0.05, "far style {style}");
        }
    }
}

#[cfg(test)]
mod underground_bg_tests {
    use super::*;

    fn world() -> World {
        let mut w = World::new("t".into(), 4200, 1200);
        w.world_surface = 356.0;
        w.rock_layer = 516.0;
        w
    }

    #[test]
    fn layout_matches_dirt_background_with_identity_background_matrix() {
        // `Terrustaria.trwld` reference camera: tile (2196, 361) at 1600x900.
        let w = world();
        let cam = Vec2::new(2196.0 * TILE_PX, 361.0 * TILE_PX);
        let l = underground_bg_layout(&w, cam, 1.28, Vec2::new(1600.0, 900.0));
        assert_eq!(l.screen_pos, Vec2::new(cam.x - 800.0, cam.y - 450.0));
        // bgTopY = worldSurface*16 - screenPosition.Y  (+16 for the dirt sheet)
        assert_eq!(l.transition_top, 356.0 * 16.0 - (cam.y - 450.0));
        assert_eq!(l.dirt_top, l.transition_top + 16.0);
    }

    #[test]
    fn backdrop_is_zoom_invariant() {
        // `Main.BackgroundViewMatrix.Zoom == ForcedMinimumZoom == 1`, so the game zoom
        // must not move the backdrop: only `ScaledPosition`, the rock clamp, does.
        let w = world();
        let cam = Vec2::new(2196.0 * TILE_PX, 361.0 * TILE_PX);
        let screen = Vec2::new(1600.0, 900.0);
        let a = underground_bg_layout(&w, cam, 1.0, screen);
        let b = underground_bg_layout(&w, cam, 2.0, screen);
        assert_eq!(a.transition_top, b.transition_top);
        assert_eq!(a.dirt_top, b.dirt_top);
        // The rock clamp *is* scaled: `camera.Y - screenHeight / (2 * zoom)`.
        assert_eq!(a.scaled_bottom, cam.y - 450.0 + 900.0);
        assert_eq!(b.scaled_bottom, cam.y - 225.0 + 900.0);
    }

    #[test]
    fn rock_transition_point_matches_getrocktransitionpoint() {
        // num = (516-356)*16 = 2560; 2560/96 = 26.67 -> 27 blocks; 356*16 + 27*96 + 32.
        let w = world();
        assert_eq!(get_rock_transition_point(&w, 96.0), 5696.0 + 27.0 * 96.0 + 32.0);
        // Exact multiple of the block height must not round up.
        assert_eq!(get_rock_transition_point(&w, 2560.0), 5696.0 + 2560.0 + 32.0);
    }
}
