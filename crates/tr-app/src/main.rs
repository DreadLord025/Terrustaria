//! Terraria-Rust: Vulkan renderer prototype for a native reimplementation.

mod assets;
mod audio;
mod biome;
mod collision;
mod inv;
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
mod render;
mod spawner;
mod settings;
mod sound;
mod ui;
mod world_sim;

use std::collections::{HashMap, HashSet, VecDeque};
use std::path::PathBuf;
use std::sync::{Arc, OnceLock};
use std::time::Instant;

use anyhow::{Context, Result};
use biome::{advance_bg_alphas, advance_bg_style, is_desert_biome, is_ocean_biome, is_snow_biome};
use glam::Vec2;
use item::{
    EMPTY_SLOT, HOTBAR, INV, ItemDrop, ItemSlot, MAX_STACK, SWING_TIME,
};
use npc::{Npc, NpcKind};
use player::Player;
use projectile::Projectile;
use rain::Rain;
use tr_core::lighting::{LightMap, LightMode, LightParams, LightStyle};
use tr_core::liquid::LiquidSystem;
use tr_core::tiledata as td;
use tr_core::{TILE_PX, World, edit, framing, save, wld, worldgen};
use tr_render::{Atlas, Gpu, SpriteRenderer};
use ui::text::{font_width, hud_text, hud_text_width};
use world_sim::clock::{advance_time, time_frozen};
use world_sim::editing::apply_edit;
use world_sim::step::FixedStepper;

use render::bg::cave_parallax;
use render::debug::{capture_frame, draw_debug, DebugCache, StatsAcc};
use render::entities::{
    draw_held_item, draw_player_map_icon, make_npc_assets, make_player_assets, NpcAssets,
    PlayerAssets,
};
// `ui/char_creation.rs` reaches the preview through the crate root
// (`crate::draw_player_preview`); the render extraction keeps that path.
pub(crate) use render::entities::draw_player_preview;
use render::sky::{
    color_of_the_skies, draw_sky, make_clouds, make_sky_assets, make_stars, tile_color,
    CloudSprite, SkyAssets, StarSprite,
};
use render::tiles::{draw_tiles, DrawScratch, TileDrawStats};

use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::{ElementState, MouseButton, MouseScrollDelta, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Window, WindowId};

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
    // `TR_UI=<screen>` presets the front-end flow state after construction
    // (the `TR_PAGE` pattern; the plan Task 5 hook table).
    let tr_ui = std::env::var("TR_UI").ok();
    let skip_menu = (!force_menu && std::env::var_os("TR_SHOT").is_some())
        || std::env::var_os("TR_CAM").is_some()
        || std::env::var_os("TR_HOLD").is_some()
        || std::env::var_os("TR_NOMENU").is_some()
        || std::env::var_os("TR_MAP").is_some()
        || joined_before
        || auto_host.is_some()
        || tr_ui.is_some();
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
    let mut world_map = map::WorldMap::new(&world);
    if std::env::var_os("TR_MAP").is_some() {
        world_map.set_open(true);
    }
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
        player_step: FixedStepper::new(1.0 / 60.0, 8),
        npc_step: FixedStepper::new(1.0 / 60.0, 8),
        npcs,
        npc_assets,
        npc_timer: 6.0,
        npc_rng: world_seed,
        attack_cd: 0.0,
        inventory,
        equip: inv::EquipSlots::empty(),
        // `Main.EquipPage`; the page tabs are not clickable yet, so a debug env
        // var selects the page for screenshots (0 = equipment, 2 = misc).
        equip_page: std::env::var("TR_EQUIPPAGE")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(0),
        selected: 0,
        hotbar_scales: {
            let mut s = [0.75; HOTBAR];
            s[0] = 1.0;
            s
        },
        inventory_open: std::env::var_os("TR_INV").is_some(),
        // The panel ships open with the inventory (`Main.cs:41706-41709`);
        // `TR_NOCRAFT=1` turns it off and `TR_CRAFT=1` forces it back on
        // (the `TR_INV` presence pattern).
        crafting_open: std::env::var_os("TR_NOCRAFT").is_none()
            || std::env::var_os("TR_CRAFT").is_some(),
        menu: menu_state,
        front: None,
        front_backup: None,
        players: Vec::new(),
        worlds: Vec::new(),
        front_hint: None,
        front_mouse_down: false,
        session_draft: None,
        vk_grid: std::env::var_os("TR_VK_GRID").is_some(),
        settings: settings::Settings::load(),
        player_name,
        net: net_role,
        remote_players: HashMap::new(),
        local_id: 0,
        net_timer: 0.0,
        join_target,
        map: Some(world_map),
        map_grab: None,
        minimap_hover: None,
        // `Main.mapStyle = 1` (`Main.cs:66750`); `TR_MAPSTYLE=<0|1|2>` selects
        // it for screenshots (the `TR_EQUIPPAGE` pattern; an invalid or
        // out-of-range value falls back to 1).
        map_style: std::env::var("TR_MAPSTYLE")
            .ok()
            .and_then(|v| v.parse::<i32>().ok())
            .filter(|v| (0..=2).contains(v))
            .unwrap_or(1),
        audio: audio::Audio::new(assets::content_dir()),
        music: audio::MusicSystem::new(assets::content_dir()),
        last_hp: 100,
        autosave: 0.0,
        mouse_item: EMPTY_SLOT,
        ui_left_press: false,
        ui_right_press: false,
        mouse_pos: mouse_pin().unwrap_or(Vec2::ZERO),
        cursor: ui::cursor::CursorState::default(),
        os_cursor_hidden: None,
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
        liquid_step: FixedStepper::new(1.0 / 60.0, 8),
        liquid_tick: 0,
        waterfall: tr_core::waterfall::WaterfallManager::new(),
        waterfall_step: FixedStepper::new(1.0 / 60.0, 8),
        rain,
        items: Vec::new(),
        item_rng: world_seed ^ 0x5DEE_CE66_D1A4_9B5F,
        item_step: FixedStepper::new(1.0 / 60.0, 8),
        swing: 0.0,
        coins,
        projectiles: Vec::new(),
        projectile_step: FixedStepper::new(1.0 / 60.0, 8),
        dev_throw: std::env::var_os("TR_THROW").is_some(),
        world_key: None,
        hell_black_box_bottom: 0.0,
    };
    // `TR_UI=<screen>`: preset the front-end flow state for screenshots (the
    // `TR_PAGE` pattern — the plan Task 5 hook table).
    if let Some(preset) = tr_ui.as_deref().and_then(tr_ui_preset) {
        app.preset_ui(preset);
    }
    // `MapHelper.LoadMap`: restore the explored map from the `.map` sidecar.
    app.load_map();
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

/// The modern front-end screens (`Main.menuMode == 888`, `Main.cs:51059-
/// 51069`). `App.front` owns one of these while `App.menu` is `None` — the
/// same single variable `Main.menuMode` is; exactly one side is live, never
/// both (Review Focus 1). The Task 6-8 screens join as their own variants.
/// The Keyboard payload keeps the live textbox state, so the enum is
/// `Clone`-only (`CharCreation`/`WorldCreation` payloads make it so anyway).
#[derive(Clone, Debug, PartialEq)]
enum FrontScreen {
    /// `_characterSelectMenu` — the one `UICharacterSelect` singleton
    /// (`Main.cs:66797`); the port's list lives on `App.players`.
    CharSelect,
    /// `_worldSelectMenu` (`Main.cs:66798`); the list lives on `App.worlds`.
    WorldSelect,
    /// Task 7's `UICharacterCreation` — the draft plus the screen state the
    /// C# keeps on the `UIState` itself (`_selectedPicker`, `_lastSelected-
    /// Hairstyle`'s gate, the slider lock). The payload is the exact value the
    /// mode-40 round-trip parks in `App.front_backup`
    /// (`UICharacterCreation.BackupConfirmationState`, `:1215`).
    CharCreation(ui::char_creation::CreationState),
    /// Task 8's `UIWorldCreation` — the draft carries every option the screen
    /// edits (`_optionwWorldName`/`_optionSeed` and the size/difficulty/evil
    /// picks, `UIWorldCreation.cs:51-143`); the C# keeps no other screen state
    /// (the description panel swaps draw-side from the hover), so the payload
    /// is the plain draft.
    WorldCreation(ui::world_creation::NewWorldDraft),
    /// Task 6's `UIVirtualKeyboard` — the plan Task 5 enum arm, live now:
    /// the card renames and `TR_UI=vk` open it. Boxed: the session's `back`
    /// target is another `FrontScreen`.
    Keyboard(Box<KeyboardSession>),
}

/// The App-side payload of `FrontScreen::Keyboard` — the plan's
/// `KeyboardRequest` (`label`, `initial`, on-submit target) split into the
/// [`ui::keyboard::Request`] ctor args, the live [`ui::keyboard::KeyboardState`]
/// both the on-screen keys and the physical keyboard edit, and the screen the
/// Cancel/Submit tails return to (the `_cancelAction` target,
/// `UIVirtualKeyboard.cs:86`; the tails reopen the select screens,
/// `UICharacterListItem.cs:205-216`, `UIWorldListItem.cs:420-433`).
#[derive(Clone, Debug, PartialEq)]
struct KeyboardSession {
    request: ui::keyboard::Request,
    state: ui::keyboard::KeyboardState,
    back: FrontScreen,
    /// Which submit tail runs — the rename notice (Task 6) or the character
    /// creator's naming flows (`Click_Naming` -> `OnFinishedNaming`,
    /// `UICharacterCreation.cs:1659-1689`; `Click_NamingAndCreating` ->
    /// `OnFinishedNamingAndCreating` -> `FinishCreatingCharacter`, `:1668-1702`).
    tail: KeyboardTail,
}

/// The keyboard's on-submit behaviour (`UIVirtualKeyboard.cs:86`'s event pair).
#[derive(Clone, Copy, Debug, PartialEq)]
enum KeyboardTail {
    /// The card renames: the port has no `.plr` writer, so Submit notices
    /// (Task 6's documented deviation).
    Rename,
    /// `Click_Naming` (`:1659-1666`): the name lands in the draft, the
    /// creation screen returns.
    Name,
    /// `Click_NamingAndCreating` with an empty name (`:1668-1677`): the name
    /// lands, then `FinishCreatingCharacter` runs (`:1701`).
    NameAndCreate,
    /// `Click_SetName`'s submit (`UIWorldCreation.cs:689-695` ->
    /// `OnFinishedSettingName`, `:726-731`): the trimmed name lands in the
    /// world draft, the creation screen returns.
    WorldName,
    /// `OpenSeedInputMenu`'s submit (`:702-708` -> `OnFinishedSettingSeed`,
    /// `:817-831` minus the secret-seed paths): the trimmed seed lands in the
    /// draft, the creation screen returns.
    WorldSeed,
    /// `Click_NamingAndCreating` with an empty name (`UIWorldCreation.cs:
    /// 710-719`): the name lands, then `FinishCreatingWorld` runs.
    WorldNameAndCreate,
}

/// One click into the front-end flow — the rows of the
/// [`next_flow_state`] table. The card actions carry the clicked index where
/// the App needs it (`FlowClick::CharDelete(i)` parks the confirm target the
/// way `Main.selectedPlayer` is parked, `UICharacterListItem.cs:246-249`).
#[derive(Clone, Copy, Debug)]
enum FlowClick {
    /// `GoBackClick`: `menuMode = 0` (`UICharacterSelect.cs:116-120`).
    CharBack,
    /// `NewCharacterClick` (`:108-114`): `Main.PendingPlayer = new Player()`,
    /// `menuMode = 888` + `SetState(new UICharacterCreation(...))` — a fresh
    /// draft on the creation screen.
    CharNew,
    /// `PlayGame` (`:256-261`) -> `SelectPlayer`'s single-player tail
    /// `menuMode = 6` (`Main.cs:66245-66249`) -> 888 world select
    /// (`:49468-49472`).
    CharPlay,
    /// `DeleteButtonClick` (`UICharacterListItem.cs:242-254`): `menuMode = 5`.
    CharDelete(usize),
    /// `GoBackClick`: `menuMode = (!menuMultiplayer) ? 1 : 12`
    /// (`UIWorldSelect.cs:118-122`) — the port is single-player.
    WorldBack,
    /// `PlayGame` (`UIWorldListItem.cs:360-374`): the launch. `WorldGen.
    /// playWorld` + `UIWorldLoad` are the out-of-scope loading flow (spec
    /// §9.4), so the port closes everything and plays the loaded world.
    WorldPlay,
    /// `DeleteButtonClick` (`UIWorldListItem.cs:346-358`): `menuMode = 9`.
    WorldDelete(usize),
    /// `NewWorldClick` (`UIWorldSelect.cs:110-116`): the stand-in world name
    /// reads `Main.WorldList.Count + 1` (`:113`) and the Journey default
    /// reads the active player's difficulty (`UIWorldCreation.cs:319-322`),
    /// so the variant carries both. `menuMode = 888` +
    /// `SetState(new UIWorldCreation())`.
    WorldNew(usize, u8),
    /// `Click_GoBack` -> `GoBack` -> `Main.OpenWorldSelectUI()`
    /// (`UIWorldCreation.cs:665-674`) — straight back to world select.
    WorldCreateBack,
    /// `Click_NamingAndCreating` with a filled name ->
    /// `FinishCreatingWorld` (`UIWorldCreation.cs:710-724`, `:845-863`): the
    /// App generates + saves the `.trwld` first, then the table's re-list of
    /// world select takes over (the vanilla tail is the out-of-scope
    /// `UIWorldLoad` flow, spec §9.4).
    WorldCreate,
    /// The confirm pages' Yes row (`selectedMenu == 1`, `Main.cs:49431`,
    /// `:49681`, `:49454`).
    ConfirmYes,
    /// The No row (`selectedMenu == 2`) — Escape's `flag5` target on the
    /// confirms (`:49435-49438`, `:49687-49690`, `:49464-49467`).
    ConfirmNo,
}

/// One hop of the `Main.menuMode` alternation as a pure table — the plan's
/// `next_state(action) -> (front, menu)` flow-state function, returning
/// `(front, front_backup, menu)`. The parked backup rides along so the
/// mode-40 round-trip is assertable in one place (`App.front_backup` is
/// `UICharacterCreation.BackupConfirmationState`, parked at
/// `UICharacterCreation.cs:1215`). Exactly one of `front`/`menu` is `Some` in
/// every result — the callers hold the same invariant on the way in
/// (Review Focus 1).
fn next_flow_state(
    front: Option<FrontScreen>,
    backup: Option<FrontScreen>,
    page: Option<menu::Page>,
    click: FlowClick,
) -> (Option<FrontScreen>, Option<FrontScreen>, Option<menu::Page>) {
    use menu::Page as P;
    match click {
        FlowClick::CharBack => (None, backup, Some(P::Main)),
        // `NewCharacterClick` (`UICharacterSelect.cs:108-114`): a fresh
        // `PendingPlayer`, the creation screen opens in 888.
        FlowClick::CharNew => (
            Some(FrontScreen::CharCreation(ui::char_creation::CreationState::new())),
            None,
            None,
        ),
        FlowClick::CharPlay => (Some(FrontScreen::WorldSelect), None, None),
        // `menuMode = 5` clears the 888 state (`Main.cs:51066-51069`'s else
        // branch — `MenuUI.SetState(null)`).
        FlowClick::CharDelete(_) => (None, backup, Some(P::ConfirmDeletePlayer)),
        FlowClick::WorldBack => (Some(FrontScreen::CharSelect), None, None),
        // The launch closes both sides (the loading flow is out of scope).
        FlowClick::WorldPlay => (None, None, None),
        FlowClick::WorldDelete(_) => (None, backup, Some(P::ConfirmDeleteWorld)),
        // `NewWorldClick` (`UIWorldSelect.cs:110-116`): the world creator
        // opens with the stand-in name and the active player's difficulty
        // default (`SetDefaultOptions`, `UIWorldCreation.cs:313-326`).
        FlowClick::WorldNew(worlds, active) => (
            Some(FrontScreen::WorldCreation(ui::world_creation::NewWorldDraft::new(
                worlds, active,
            ))),
            None,
            None,
        ),
        // `GoBack` (`UIWorldCreation.cs:670-674`): `OpenWorldSelectUI` — no
        // discard confirm on this screen (its draft is choice-only).
        FlowClick::WorldCreateBack => (Some(FrontScreen::WorldSelect), None, None),
        // `FinishCreatingWorld` (`:845-863`) runs before the table row; the
        // reopen re-lists like `OpenWorldSelectUI`'s `OnActivate`.
        FlowClick::WorldCreate => (Some(FrontScreen::WorldSelect), None, None),
        // Modes 5/9: Yes and No both reopen the screen the confirm came from
        // (`Main.cs:49434-49439`, `:49684-49692`).
        FlowClick::ConfirmYes | FlowClick::ConfirmNo
            if page == Some(P::ConfirmDeletePlayer) =>
        {
            (Some(FrontScreen::CharSelect), None, None)
        }
        FlowClick::ConfirmYes | FlowClick::ConfirmNo
            if page == Some(P::ConfirmDeleteWorld) =>
        {
            (Some(FrontScreen::WorldSelect), None, None)
        }
        // Mode 40 Yes: `BackupConfirmationState = null` +
        // `OpenCharacterSelectUI` (`:49454-49459`).
        FlowClick::ConfirmYes if page == Some(P::ConfirmDiscard) => {
            (Some(FrontScreen::CharSelect), None, None)
        }
        // Mode 40 No: `menuMode = 888` + `SetState(BackupConfirmationState)`
        // (`:49460-49467`) — the exact parked screen returns.
        FlowClick::ConfirmNo if page == Some(P::ConfirmDiscard) => (backup, None, None),
        // Any other combination is a no-op (a click kind its screen never
        // sends); the state passes through unchanged.
        _ => (front, backup, page),
    }
}

/// The `TR_UI=<screen>` presets (the plan Task 5 hook table): the front-end
/// screens plus the legacy pages their clicks park onto.
#[derive(Clone, Copy, Debug, PartialEq)]
enum TrUiPreset {
    CharSelect,
    WorldSelect,
    /// Task 7's screen — parsed now, presettable when its variant lands.
    CharNew,
    /// Task 8's screen.
    WorldNew,
    /// Task 6's keyboard.
    Keyboard,
    /// menuMode 3000.
    Credits,
    /// menuMode 5.
    DelPlayer,
    /// menuMode 9.
    DelWorld,
    /// menuMode 40.
    Discard,
}

/// The `TR_UI` name table (the `TR_PAGE` pattern, `menu.rs`
/// `page_from_name`): the plan Task 5 hook list, case-insensitive; unknown
/// names resolve to nothing.
fn tr_ui_preset(name: &str) -> Option<TrUiPreset> {
    Some(match name.to_ascii_lowercase().as_str() {
        "charsel" => TrUiPreset::CharSelect,
        "worldsel" => TrUiPreset::WorldSelect,
        "charnew" => TrUiPreset::CharNew,
        "worldnew" => TrUiPreset::WorldNew,
        "vk" => TrUiPreset::Keyboard,
        "credits" => TrUiPreset::Credits,
        "delplayer" => TrUiPreset::DelPlayer,
        "delworld" => TrUiPreset::DelWorld,
        "discard" => TrUiPreset::Discard,
        _ => return None,
    })
}

/// `TR_FAKE_CHARS`/`TR_FAKE_WORLDS=<n>`: the screenshot lists' fake counts
/// (`save_list::list_players`/`list_worlds` prepend `Player n`/`World n`).
fn tr_fake_count(key: &str) -> usize {
    std::env::var(key).ok().and_then(|v| v.parse().ok()).unwrap_or(0)
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
    player_step: FixedStepper,
    npc_step: FixedStepper,
    npcs: Vec<Npc>,
    npc_assets: NpcAssets,
    npc_timer: f32,
    npc_rng: u64,
    attack_cd: f32,
    inventory: [ItemSlot; INV],
    /// The non-grid inventory slots (`EquipSlots`): coins, ammo, equipment,
    /// vanity, dye, misc equips and the trash slot (`Main.DrawInventory`).
    equip: inv::EquipSlots,
    /// `Main.EquipPage`: 0 armour/vanity/dye, 2 misc equips, 3 camera.
    equip_page: i32,
    selected: usize,
    /// Vanilla-style per-slot hotbar scale (selected grows to 1.0).
    hotbar_scales: [f32; HOTBAR],
    /// Escape toggles the inventory panel (`Main.OpenInventory`), it no longer quits.
    inventory_open: bool,
    /// The modern crafting panel, open alongside the inventory
    /// (`NewCraftingUI`, `Main.cs:41706-41709`); `TR_NOCRAFT=1` hides it,
    /// `TR_CRAFT=1` forces it on.
    crafting_open: bool,
    /// Main menu; `None` while playing.
    menu: Option<menu::Menu>,
    /// `Main.menuMode == 888`'s half of the single-mode invariant
    /// (`Main.cs:51059-51069`): exactly one of `front` and `menu` is `Some` —
    /// the draw site picks `front` first, then `menu`, and every navigation
    /// sets/clears both sides through [`next_flow_state`]'s table
    /// (Review Focus 1).
    front: Option<FrontScreen>,
    /// `UICharacterCreation.BackupConfirmationState` (`UICharacterCreation.cs:
    /// 152`, parked at `:1215`): the screen menuMode 40's No branch restores.
    /// `None` except across a discard confirm.
    front_backup: Option<FrontScreen>,
    /// `_characterSelectMenu`'s card list — `OnActivate`'s `LoadPlayers` +
    /// `UpdatePlayersList` (`UICharacterSelect.cs:135-157`).
    players: Vec<ui::save_list::PlayerEntry>,
    /// `_worldSelectMenu`'s card list (`UIWorldSelect.cs:137-166`).
    worlds: Vec<ui::save_list::WorldEntry>,
    /// The port's not-ported notice for front-screen clicks whose target
    /// lands with Tasks 6-8 (ours, like `Menu.hint`).
    front_hint: Option<String>,
    /// `Main.mouseLeft`'s port for the front screens (`Main.cs:51101`'s role):
    /// held between the press and release events while `front` is live — the
    /// creation screen's HSL sliders drag off it, like the menu's value bars.
    front_mouse_down: bool,
    /// `Main.PendingPlayer`'s recorded twin — `FinishCreatingCharacter`'s
    /// `PlayerFileData.CreateAndSave` + `Main.LoadPlayers` port: the created
    /// draft feeds the implicit session entry's name/difficulty
    /// (`UICharacterCreation.cs:1704-1711`; no `.plr` writer — documented).
    session_draft: Option<ui::char_creation::NewPlayerDraft>,
    /// `PlayerInput.SettingsForUI.ShowGamepadHints`'s port stand-in: the
    /// keyboard's two-layer gate (`UIVirtualKeyboard.cs:675-678`) — false
    /// without a gamepad, so the desktop `outerLayer2` draws (Review Focus 6);
    /// `TR_VK_GRID=1` overrides it for screenshots (the plan Task 6 hook).
    vk_grid: bool,
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
    /// `Main.grabMapX`/`grabMapY`: the cursor anchor of an in-progress
    /// fullscreen-map drag (`Main.cs:55346-55354`).
    map_grab: Option<[f32; 2]>,
    /// `MinimapFrame.Update`'s hovered button (`MinimapFrame.cs:163`),
    /// resolved every frame in `handle_minimap_clicks` and consumed by the
    /// minimap branch's `draw_foreground`; `Some` also mirrors `:164`'s
    /// `Main.LocalPlayer.mouseInterface = true` (the mine/place suppression).
    minimap_hover: Option<ui::minimap::MinimapButton>,
    /// `Main.mapStyle` (`Main.cs:66750`, default 1): 0 = off, 1 = the minimap
    /// frame, 2 = the overlay (out of P4 scope — nothing draws, the whole-UI P4
    /// plan's Uncertainty 7). The map-style buttons land in Task 3, so a debug
    /// env var selects it for screenshots.
    map_style: i32,
    audio: audio::Audio,
    music: audio::MusicSystem,
    last_hp: i32,
    /// `Main.saveTime`: seconds since the last autosave (`Main.cs:19751-19765`).
    autosave: f32,
    /// Stack carried on the cursor while the inventory is open (`Main.mouseItem`).
    mouse_item: ItemSlot,
    ui_left_press: bool,
    ui_right_press: bool,
    mouse_pos: Vec2,
    /// `Main.cursorAlpha`/`cursorColorDirection`/`cursorScale`: the vanilla cursor
    /// pulse, advanced once per frame by `Main.CursorColor` (`Main.cs:52477`).
    cursor: ui::cursor::CursorState,
    /// Last value passed to `Window::set_cursor_visible`, so the OS pointer is only
    /// toggled when the in-world HUD (and thus the vanilla cursor) appears.
    os_cursor_hidden: Option<bool>,
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
    liquid_step: FixedStepper,
    liquid_tick: u32,
    /// `Main.waterfallManager`: scanning, animation counters and draw geometry.
    waterfall: tr_core::waterfall::WaterfallManager,
    waterfall_step: FixedStepper,
    /// Weather and raindrops (`Rain.MakeRain` / `Rain.Update`).
    rain: Rain,
    /// World item drops (`Terraria.Item` first pass).
    items: Vec<ItemDrop>,
    item_rng: u64,
    item_step: FixedStepper,
    /// Remaining seconds of the current held-item swing.
    swing: f32,
    /// Coin purse value in copper coins.
    coins: u64,
    /// Live shuriken/projectile entities.
    projectiles: Vec<Projectile>,
    projectile_step: FixedStepper,
    /// `TR_THROW` debug hook: throw one shuriken at startup.
    dev_throw: bool,
    /// G6: identity of the last world instance range; a hit lets the frame reuse it.
    world_key: Option<WorldFrameKey>,
    /// `Main.hellBlackBoxBottom` (`Main.cs:2463`): world Y of the hell floor line the
    /// previous frame's magma pass produced, clamping the underworld black box. It is
    /// written after the box draws, so the box always consumes the previous frame's
    /// value (0 on the first frame — a negative height draws nothing, like XNA).
    hell_black_box_bottom: f32,
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

/// `TR_NOCACHE=1` disables D2 and G6, so benchmarks measure the full per-frame
/// work instead of a cached scene. Also used to A/B the optimisations.
fn no_cache() -> bool {
    static NO_CACHE: OnceLock<bool> = OnceLock::new();
    *NO_CACHE.get_or_init(|| std::env::var_os("TR_NOCACHE").is_some())
}

/// `TR_NOHUD=1` hides the HUD, hotbar and minimap so screenshots show only the
/// world. The debug overlay (`TR_DEBUG`) is independent and still drawn.
fn no_hud() -> bool {
    static NO_HUD: OnceLock<bool> = OnceLock::new();
    *NO_HUD.get_or_init(|| std::env::var_os("TR_NOHUD").is_some())
}

/// `TR_MOUSE=x,y` pins the logical pointer position and ignores real cursor
/// motion, so screenshots can deterministically show the cursor and a hovered
/// item tooltip (like `TR_CAM`/`TR_DAY` do for the world).
fn mouse_pin() -> Option<Vec2> {
    static PIN: OnceLock<Option<Vec2>> = OnceLock::new();
    *PIN.get_or_init(|| {
        let s = std::env::var("TR_MOUSE").ok()?;
        let mut it = s.split(',').map(|v| v.trim().parse::<f32>());
        match (it.next(), it.next()) {
            (Some(Ok(x)), Some(Ok(y))) => Some(Vec2::new(x, y)),
            _ => None,
        }
    })
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
                        if self.front.is_some() {
                            // menuMode 888 takes no world keyboard routes
                            // (the screens are mouse-driven; the gamepad snap
                            // points are out of scope, §J.3) — the world
                            // controls stay parked while a front screen is
                            // live. The virtual keyboard is the exception:
                            // `Main.GetInputText` edits its text every frame
                            // (`UIVirtualKeyboard.cs:750-788`), Enter submits
                            // through `CanSubmit` (`:763-766`) and Escape
                            // runs the cancel (`:771-774`'s
                            // `TryEscapingMenu`).
                            if let Some(text) = event.text.as_ref() {
                                for ch in text.chars() {
                                    if !ch.is_control() {
                                        self.keyboard_character(ch);
                                    }
                                }
                            }
                            match code {
                                KeyCode::Backspace => self.keyboard_backspace(),
                                KeyCode::Escape => self.keyboard_cancel(),
                                KeyCode::Enter | KeyCode::NumpadEnter => self.keyboard_enter(),
                                _ => {}
                            }
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
                                // `Main.TryOpeningFullscreenMap` (`Player.cs:29105`):
                                // `M` only toggles the map when `Main.mapEnabled`.
                                if self.settings.map_enabled {
                                    if let Some(m) = self.map.as_mut() {
                                        let open = !m.open;
                                        m.set_open(open);
                                    }
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
                if self.map.as_ref().is_some_and(|m| m.open) {
                    // `Player.cs:25190-25197`: while the map is fullscreen the wheel
                    // zooms it instead of cycling the hotbar.
                    let (sw, _) = self.screen_size();
                    if let Some(m) = self.map.as_mut() {
                        m.scale = map::map_scale_clamp(
                            sw,
                            self.world.width,
                            map::map_wheel_scale(m.scale, d * 120.0),
                        );
                    }
                } else {
                    let step = d.round() as i32;
                    if step != 0
                        && self.menu.is_none()
                        && self.front.is_none()
                        && !self.inventory_open
                    {
                        self.selected =
                            (self.selected as i32 + step).rem_euclid(HOTBAR as i32) as usize;
                    }
                }
            }
            WindowEvent::MouseInput { state, button, .. } => {
                let pressed = state == ElementState::Pressed;
                match button {
                    MouseButton::Middle => self.mouse_down = pressed,
                    MouseButton::Left => {
                        // The flag belongs to whichever side owns the input;
                        // while a legacy page is live the 888 side is closed,
                        // so a release here must not leak a stale `true` into
                        // a restored front screen.
                        if self.menu.is_some() {
                            self.front_mouse_down = false;
                        }
                        if let Some(m) = self.menu.as_mut() {
                            // `Main.mouseLeft` (`Main.cs:51101`): the value
                            // bars drag off this flag (`Main.cs:51533-51576`),
                            // held between the press and release events.
                            m.mouse_down = pressed;
                            if pressed {
                                m.update_hover(self.mouse_pos);
                                let action = m.click();
                                self.apply_menu_action(action, event_loop);
                            }
                            return;
                        }
                        if self.front.is_some() {
                            // The 888 screens are click-driven: the press
                            // routes through the card-hit table (`Main.
                            // MouseUI`'s click while `MenuUI` is live). The
                            // held flag rides along for the creation screen's
                            // HSL drags (`Main.mouseLeft`, `Main.cs:51101`).
                            self.front_mouse_down = pressed;
                            if pressed {
                                self.handle_front_click();
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
                if mouse_pin().is_none() {
                    self.mouse_pos = p;
                }
                // The menu reads the pinned pointer when `TR_MOUSE` is set, so
                // its hover and value-bar drags stay deterministic.
                let pointer = self.mouse_pos;
                if let Some(m) = self.menu.as_mut() {
                    m.update_hover(pointer);
                }
                // The middle-button world-camera drag is suppressed while the map is
                // fullscreen, like the other world controls (`Player.cs:24798`).
                if self.mouse_down && !self.map.as_ref().is_some_and(|m| m.open) {
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
    /// The swapchain size in pixels (the deterministic fallback mirrors the
    /// `TR_SHOT`/headless default of 1280x720).
    fn screen_size(&self) -> (f32, f32) {
        world_sim::camera::screen_size(self)
    }

    fn update_player(&mut self, dt: f32) {
        world_sim::player::update_player(self, dt);
    }

    /// `Player.cs:24774-24793` + `24898-24908`: the fullscreen-map controls. The pan
    /// step is `16 / scale` tiles **once per frame, with no `dt` factor** (the
    /// original runs it in the per-frame input pass).
    fn update_map_input(&mut self) {
        let (sw, _) = self.screen_size();
        let Some(m) = self.map.as_mut() else { return };
        let step = 16.0 / m.scale;
        let k = |code| self.keys.contains(&code);
        if k(KeyCode::KeyW) || k(KeyCode::ArrowUp) {
            m.pos[1] -= step;
        }
        if k(KeyCode::KeyS) || k(KeyCode::ArrowDown) {
            m.pos[1] += step;
        }
        if k(KeyCode::KeyA) || k(KeyCode::ArrowLeft) {
            m.pos[0] -= step;
        }
        if k(KeyCode::KeyD) || k(KeyCode::ArrowRight) {
            m.pos[0] += step;
        }
        // `Main.cs:50453-50454`: `cMapZoomIn = "Add"`, `cMapZoomOut = "Subtract"`.
        if k(KeyCode::NumpadAdd) {
            m.scale = map::map_key_scale(m.scale, true);
        }
        if k(KeyCode::NumpadSubtract) {
            m.scale = map::map_key_scale(m.scale, false);
        }
        // `Main.cs:55342-55359`: the left button drags the fullscreen map by the
        // per-frame cursor delta (`Main.grabMapX`/`grabMapY`).
        let cursor = self.mouse_pos;
        if self.mine_down {
            if let Some(anchor) = self.map_grab {
                let d = map::map_drag_delta([cursor.x - anchor[0], cursor.y - anchor[1]], m.scale);
                m.pos[0] += d[0];
                m.pos[1] += d[1];
            }
            self.map_grab = Some([cursor.x, cursor.y]);
        } else {
            self.map_grab = None;
        }
        m.scale = map::map_scale_clamp(sw, self.world.width, m.scale);
        m.pos = map::map_clamp_pos(m.pos, &self.world);
    }

    /// `Main.DoUpdate_AnimateDiscoRGB`: step the shared disco-ball colour cycle.
    fn advance_disco(&mut self) {
        world_sim::clock::advance_disco(self);
    }

    /// Fixed 60 Hz liquid stepping; `Liquid.UpdateLiquid` itself runs every second
    /// tick, matching the original 30 Hz rate.
    fn update_liquids(&mut self, dt: f32) {
        world_sim::liquids::update_liquids(self, dt);
    }

    /// Fixed 60 Hz `WaterfallManager.UpdateFrame` / `FindWaterfalls`, matching the
    /// original's per-update tick and 30-tick rescan cadence.
    fn update_waterfalls(&mut self, dt: f32, screen: Vec2) {
        world_sim::liquids::update_waterfalls(self, dt, screen);
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
        world_sim::weather::update_rain(self, dt, screen);
    }

    fn update_npcs(&mut self, dt: f32) {
        world_sim::npcs::update_npcs(self, dt);
    }

    fn update_editing(&mut self, dt: f32, screen: Vec2) {
        world_sim::editing::update_editing(self, dt, screen);
    }

    fn is_client(&self) -> bool {
        matches!(self.net, NetRole::Client(_))
    }

    /// Item physics and pickup on a fixed 60 Hz step (`Item.Update`).
    fn update_items(&mut self, dt: f32) {
        world_sim::items::update_items(self, dt);
    }

    /// Fixed 60 Hz projectile stepping. Damage lands through `Npc::hurt`, so the
    /// hit cooldown and knockback are shared with melee.
    fn update_projectiles(&mut self, dt: f32) {
        world_sim::projectiles::update_projectiles(self, dt);
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

    /// `MinimapFrameManager.DrawTo` -> `MinimapFrame.Update`
    /// (`MinimapFrameManager.cs:45`, `MinimapFrame.cs:138-177`): the hover +
    /// held-click handling of the three zoom buttons, run every frame while the
    /// minimap HUD is live (the same gate as the draw branch). Clicks repeat
    /// every frame while the button is held (`:169-171`); `apply` mutates the
    /// minimap scale (`:107-136`). The resulting hover also feeds the
    /// `draw_foreground` highlight (the source's `IsHighlighted`, `:163`) and
    /// the mine/place suppression (`:164`'s `mouseInterface`).
    ///
    /// Deviation: `:167,174` play `SoundEngine.PlaySound(12)` on hover changes
    /// and click releases — the port loads no UI cue (`audio.rs:22-31`), so it
    /// stays silent, consistent with the menus.
    fn handle_minimap_clicks(&mut self) {
        // The buttons exist only while the frame draws (`Main.cs:55546`'s
        // `DrawTo` gate): `mapEnabled && !mapFullscreen && mapStyle == 1`,
        // plus the port's screenshot `no_hud` knob that hides the draw too.
        let live = self.settings.map_enabled
            && !no_hud()
            && self.map.as_ref().is_some_and(|m| !m.open)
            && self.map_style == 1;
        if !live {
            self.minimap_hover = None;
            return;
        }
        let (sw, sh) = self.screen_size();
        let screen = Vec2::new(sw, sh);
        // The frame position from the same formula as the draw branch
        // (`Main.cs:55546`'s `DrawTo` position + `_frameOffset`).
        let pos = ui::minimap::draw_to_position(ui::minimap::minimap_origin(screen));
        let s = ui::minimap::style_index(&self.settings.minimap_frame);
        let frame_pos = ui::minimap::frame_position(pos, &ui::minimap::FRAME_STYLES[s]);
        let sizes = ui::minimap::button_sizes(&self.lookups, s);
        // `Main.mouseLeft` is the held state (`MinimapFrame.cs:169`).
        let action = ui::minimap::update(
            self.mouse_pos,
            frame_pos,
            &ui::minimap::FRAME_STYLES[s],
            sizes,
            self.mine_down,
        );
        // `buttonUnderMouse.IsHighlighted = true` (`:163`); a click implies it.
        self.minimap_hover = match action {
            ui::minimap::MinimapAction::None => None,
            ui::minimap::MinimapAction::Hover(b) | ui::minimap::MinimapAction::Click(b) => Some(b),
        };
        // `buttonUnderMouse.Click()` -> the zoom handlers (`:171`, `:107-136`);
        // `apply` is a no-op unless the action is a click.
        if let Some(m) = self.map.as_mut() {
            ui::minimap::apply(action, &mut m.minimap_scale);
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
        // `Main.cs:41087-41120`: the map-style icon row's clicks — one-shot on
        // the press (`Main.mouseLeft && Main.mouseLeftRelease`), only while the
        // inventory is open (their home is `DrawInventory`, `:41055`) and the
        // map system is enabled (`:41056`'s `Main.mapEnabled` wraps the row).
        // A hit consumes the press and returns before the slot handling.
        // `PlaySound(10)/(12)` (`:41100,41109-41119`) is not ported — no UI cue
        // is loaded (`audio.rs:22-31`).
        if left && self.inventory_open && self.settings.map_enabled {
            let (sw, sh) = self.screen_size();
            let screen = Vec2::new(sw, sh);
            for k in 0..4usize {
                let [x, y] = ui::chrome::map_icon_rect(k, screen);
                // `Main.cs:41087`: inclusive on all four edges.
                if self.mouse_pos.x >= x
                    && self.mouse_pos.x <= x + ui::chrome::MAP_ICON_HIT[0]
                    && self.mouse_pos.y >= y
                    && self.mouse_pos.y <= y + ui::chrome::MAP_ICON_HIT[1]
                {
                    if k == 0 {
                        // `Main.cs:41095-41104`: close the inventory, open the
                        // fullscreen map. `set_open(true)` already resets the
                        // scale to `2.5` and arms the recentre (`Main.cs:41101,
                        // 41104`, `crates/tr-app/src/map.rs:451-457`);
                        // `SetTalkNPC(-1)` / `npcChatCornerItem = 0` have no
                        // port counterpart.
                        self.inventory_open = false;
                        if let Some(m) = self.map.as_mut() {
                            m.set_open(true);
                        }
                    } else {
                        // `Main.cs:41106-41120`: `k == 1/2/3` ->
                        // `Main.mapStyle = 0/1/2`.
                        self.map_style = k as i32 - 1;
                    }
                    return;
                }
            }
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
                // `Main.cs:48981-48987`: menuMode 1 -> `OpenCharacterSelectUI`
                // (888) — the real screen flow replaces the old "Single
                // Player closes the menu" shortcut.
                self.enter_front(FrontScreen::CharSelect);
            }
            // The confirm pages (`Main.cs:49420-49439`, `:49674-49692`,
            // `:49440-49467`): the resolution runs the erase on Yes, then
            // reopens the screen the confirm came from.
            Action::ConfirmYes | Action::ConfirmNo => {
                let yes = matches!(action, menu::Action::ConfirmYes);
                let (page, index) = self
                    .menu
                    .as_ref()
                    .map_or((None, 0usize), |m| (Some(m.page), m.confirm_index));
                let click = if yes { FlowClick::ConfirmYes } else { FlowClick::ConfirmNo };
                // Clones: `FrontScreen` is no longer `Copy`, and the erase
                // calls below need `self` whole.
                let (front, backup, _) =
                    next_flow_state(self.front.clone(), self.front_backup.clone(), page, click);
                // Mode 40's Yes drops the parked state (`Main.cs:49455`:
                // `BackupConfirmationState = null`); the flow function returns
                // `None` for it, and No's restore value for the reopen.
                self.front_backup = backup;
                if yes && page == Some(menu::Page::ConfirmDeletePlayer) {
                    // `Main.ErasePlayer` (`:49432-49434` -> `:4772-4791`).
                    self.erase_player_entry(index);
                }
                if yes && page == Some(menu::Page::ConfirmDeleteWorld) {
                    // `Main.EraseWorld` (`:49682-49684` -> `:4796-4813`).
                    self.erase_world_entry(index);
                }
                match (page, front) {
                    // Mode 40's No restores the exact parked screen without
                    // the `OnActivate` re-list (`menuMode = 888` +
                    // `SetState(BackupConfirmationState)`, `:49460-49467`).
                    (Some(menu::Page::ConfirmDiscard), Some(f)) if !yes => {
                        self.menu = None;
                        self.front = Some(f);
                    }
                    // Modes 5/9 (both rows) and 40's Yes re-enter through the
                    // openers, which run the `OnActivate` re-list
                    // (`menuMode = 1/6` -> `OpenCharacterSelectUI` /
                    // `OpenWorldSelectUI`).
                    (_, Some(f)) => self.enter_front(f),
                    // Unreachable from the confirm pages (every branch
                    // navigates); keep the invariant by closing both sides.
                    (_, None) => {
                        self.front = None;
                        self.park_menu();
                    }
                }
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

    /// Takes the legacy menu into the App without the launch semantics — the
    /// settings sync of [`App::close_menu`] for every hop where a front screen
    /// takes over or the world launches.
    fn park_menu(&mut self) {
        if let Some(m) = self.menu.take() {
            self.settings = m.settings;
        }
        self.apply_settings();
    }

    /// Enters a front screen the way `OpenCharacterSelectUI`/
    /// `OpenWorldSelectUI` do (`Main.cs:52346-52350`, `:52362-52366`): the
    /// singleton state is set and `OnActivate` re-lists (`LoadPlayers`/
    /// `LoadWorlds` + `UpdatePlayersList`/`UpdateWorldsList`,
    /// `UICharacterSelect.cs:135-157` / `UIWorldSelect.cs:137-166`). The menu
    /// side closes with the settings sync first.
    fn enter_front(&mut self, front: FrontScreen) {
        match front {
            FrontScreen::CharSelect => {
                let fakes = tr_fake_count("TR_FAKE_CHARS");
                self.players =
                    ui::save_list::list_players(None, Some(self.session_entry()), fakes);
                ui::char_select::sort_players(&mut self.players);
            }
            FrontScreen::WorldSelect => {
                let fakes = tr_fake_count("TR_FAKE_WORLDS");
                // The listing root is the port's `Documents/My Games/
                // Terrustaria/Worlds` (`Main.WorldPath`'s directory,
                // `Main.cs:67065-67075`; `save.rs:20-29`).
                let dir = save::default_world_dir().unwrap_or_default();
                self.worlds = ui::save_list::list_worlds(&dir, fakes);
                ui::world_select::sort_worlds(&mut self.worlds, false);
            }
            // The keyboard re-lists nothing (`UIVirtualKeyboard.OnActivate`'s
            // only setup is the gamepad snap point, `:843-850` — out of scope,
            // §J.3). It enters through `open_keyboard`, which parks the menu
            // side the same way.
            FrontScreen::Keyboard(_) => {}
            // The creation screen re-lists nothing — `NewCharacterClick`
            // (`UICharacterSelect.cs:108-114`) just sets `Main.PendingPlayer`
            // and the state. Entering through `enter_front` (the Back/New
            // hops) keeps the single-mode invariant's write pattern.
            FrontScreen::CharCreation(_) => {}
            // The world creator re-lists nothing either — `NewWorldClick`
            // (`UIWorldSelect.cs:110-116`) sets `Main.newWorldName` and the
            // state; the draft rode in from the table row.
            FrontScreen::WorldCreation(_) => {}
        }
        self.park_menu();
        self.front = Some(front);
    }

    /// Opens the virtual keyboard the way `RenameButtonClick` does
    /// (`Main.clrInput()` + `Main.MenuUI.SetState(state)`,
    /// `UICharacterListItem.cs:197-201`): the ctor args become the session's
    /// [`ui::keyboard::Request`], the [`ui::keyboard::KeyboardState`] seeds
    /// from them (`:429-433`), and `back` is the `_cancelAction` target. The
    /// menu side closes with the settings sync first, so exactly one of
    /// `front`/`menu` stays live (Review Focus 1).
    fn open_keyboard(&mut self, request: ui::keyboard::Request, back: FrontScreen) {
        self.open_keyboard_with_tail(request, back, KeyboardTail::Rename);
    }

    /// The [`App::open_keyboard`] twin for the creator's naming flows — the
    /// `tail` selects the submit behaviour (`KeyboardTail`).
    fn open_keyboard_with_tail(
        &mut self,
        request: ui::keyboard::Request,
        back: FrontScreen,
        tail: KeyboardTail,
    ) {
        let state = ui::keyboard::KeyboardState::new(&request);
        self.park_menu();
        self.front = Some(FrontScreen::Keyboard(Box::new(KeyboardSession {
            request,
            state,
            back,
            tail,
        })));
    }

    /// The submit tail. The rename tail keeps Task 6's notice (no `.plr`
    /// writer, spec §9.4); the creator's naming tails run
    /// `OnFinishedNaming` (`:1684-1689`: the trimmed name lands in the draft,
    /// `SetState(this)` restores the screen) and `OnFinishedNamingAndCreating`
    /// (`:1696-1702`: the same, then `FinishCreatingCharacter`).
    fn keyboard_submit(&mut self) {
        let Some(FrontScreen::Keyboard(session)) = self.front.as_ref() else {
            return;
        };
        let name = session.state.text.trim().to_string();
        match session.tail {
            KeyboardTail::Rename => {
                let back = session.back.clone();
                self.front_hint = Some("Renaming is not ported yet (no save-file writer)".into());
                self.enter_front(back);
            }
            KeyboardTail::Name => {
                let mut back = session.back.clone();
                if let FrontScreen::CharCreation(state) = &mut back {
                    state.draft.name = name;
                }
                self.enter_front(back);
            }
            KeyboardTail::NameAndCreate => {
                let mut back = session.back.clone();
                if let FrontScreen::CharCreation(state) = &mut back {
                    state.draft.name = name;
                }
                self.enter_front(back);
                self.finish_creating_character();
            }
            // `OnFinishedSettingName` (`UIWorldCreation.cs:726-731`): the
            // trimmed name lands (`UpdateInputFields` -> `SetContents`), the
            // creation screen returns through `GoBackHere` (`:730`).
            KeyboardTail::WorldName => {
                let mut back = session.back.clone();
                if let FrontScreen::WorldCreation(draft) = &mut back {
                    draft.name = name;
                }
                self.enter_front(back);
            }
            // `OnFinishedSettingSeed` (`:817-831`): the trimmed seed lands
            // (`TryApplyingCopiedSeed`'s secret-seed paths are the out-of-
            // scope Advanced tab, plan Uncertainty 4).
            KeyboardTail::WorldSeed => {
                let mut back = session.back.clone();
                if let FrontScreen::WorldCreation(draft) = &mut back {
                    draft.seed = name;
                }
                self.enter_front(back);
            }
            // `OnFinishedNamingAndCreating` (`:838-842`): the name lands,
            // then `FinishCreatingWorld` (`:845-863`).
            KeyboardTail::WorldNameAndCreate => {
                let mut back = session.back.clone();
                if let FrontScreen::WorldCreation(draft) = &mut back {
                    draft.name = name;
                }
                self.enter_front(back);
                if let Some(FrontScreen::WorldCreation(draft)) = self.front.clone() {
                    self.finish_creating_world(draft);
                }
            }
        }
    }

    /// `FinishCreatingCharacter` (`UICharacterCreation.cs:1704-1711`) minus
    /// the out-of-scope pieces: `TryAutoAssigningHair` (the cloth-style
    /// default, `:631-637`), then the stat/inventory setup is the fresh
    /// player the port already runs, `PlayerFileData.CreateAndSave` has no
    /// `.plr` writer (the draft records into `App.session_draft` — the
    /// documented deviation), and `Main.LoadPlayers` + `menuMode = 1` reopen
    /// the character select through the real opener.
    fn finish_creating_character(&mut self) {
        let Some(FrontScreen::CharCreation(state)) = self.front.as_ref() else {
            return;
        };
        let mut draft = state.draft.clone();
        if !state.hair_picked {
            // `TryAutoAssigningHair` (`:630-637`).
            if let Some(hair) =
                ui::char_creation::default_hairstyle_for_cloth_style(draft.skin_variant)
            {
                draft.hair = hair;
            }
        }
        draft.name = draft.name.trim().to_string();
        self.player_name = draft.name.clone();
        self.session_draft = Some(draft);
        self.front_backup = None;
        self.front = None;
        self.enter_front(FrontScreen::CharSelect);
    }

    /// `FinishCreatingWorld` (`UIWorldCreation.cs:845-863`) minus the
    /// out-of-scope tail: `WorldGen.CreateNewWorld`'s `UIWorldLoad` loading
    /// screen is not ported (spec §9.4), so the generated world saves and
    /// world select re-opens (`OpenWorldSelectUI`'s effect, `:52362-52366`).
    /// The metadata pieces the port's `.trwld` header has no fields for —
    /// GameMode/evil (`WorldFile.CreateMetadata`, `:848-851`) — are the
    /// documented deviation the listing reads as Classic.
    fn finish_creating_world(&mut self, draft: ui::world_creation::NewWorldDraft) {
        match ui::world_creation::create_world(&draft) {
            Ok(path) => log::info!("created '{}' -> {}", draft.name.trim(), path.display()),
            Err(e) => {
                // `FrontHint`'s not-ported notice carries the failure — the
                // vanilla flow has no error UI on this hop.
                self.front_hint = Some(format!("world creation failed: {e:#}"));
            }
        }
        self.front_backup = None;
        self.front = None;
        self.enter_front(FrontScreen::WorldSelect);
    }

    /// The cancel tail — `_cancelAction` (`UIVirtualKeyboard.cs:86`) is the
    /// card lists' `GoBackHere` (`UICharacterListItem.cs:214-216`,
    /// `UIWorldListItem.cs:428-430`): reopen the screen the keyboard came
    /// from, no rename, no notice.
    fn keyboard_cancel(&mut self) {
        let Some(FrontScreen::Keyboard(session)) = self.front.as_ref() else { return };
        let back = session.back.clone();
        self.enter_front(back);
    }

    /// One printable character from the physical keyboard — `Main.GetInputText`'s
    /// result landing in `this.Text` (`UIVirtualKeyboard.cs:778-788`). That
    /// path never touches `KeyState`, so a seeded Shift stays latched (the
    /// on-screen keys' [`ui::keyboard::type_char`] is the only unlatcher).
    fn keyboard_character(&mut self, c: char) {
        if let Some(FrontScreen::Keyboard(session)) = self.front.as_mut() {
            ui::keyboard::write_str(&mut session.state, &c.to_string());
        }
    }

    /// The physical Backspace — `Main.GetInputText` shrinks the string the
    /// same frame (`Main.cs:50793`'s handler the keyboard state feeds through).
    fn keyboard_backspace(&mut self) {
        if let Some(FrontScreen::Keyboard(session)) = self.front.as_mut() {
            session.state.backspace();
        }
    }

    /// The physical Enter — `Main.inputTextEnter && UIVirtualKeyboard.CanSubmit`
    /// fires the submit (`:763-766`); `CanSubmit` is `ValidateText`'s cached
    /// `TextIsValidForSubmit` (`:108-145`, `:585-626`), so an invalid text
    /// keeps the keyboard open, exactly like the dead Submit click.
    fn keyboard_enter(&mut self) {
        let valid = match self.front.as_ref() {
            Some(FrontScreen::Keyboard(session)) => {
                ui::keyboard::text_valid(&session.state.text, session.request.allow_empty)
            }
            _ => return,
        };
        if valid {
            self.keyboard_submit();
        }
    }

    /// The implicit session character as a card entry: the fresh `Player`
    /// defaults — difficulty 0 (`Player.cs:1654`), life 100 (`:1924`), mana 20
    /// (`:56075`) — named by the port's seeded `player_name`, overridden by
    /// the recorded creation draft once one is created
    /// (`FinishCreatingCharacter`'s port). No `.plr` exists, so it is not
    /// deletable (`Main.ErasePlayer` needs a file) and always playable.
    fn session_entry(&self) -> ui::save_list::PlayerEntry {
        ui::save_list::PlayerEntry {
            name: self
                .session_draft
                .as_ref()
                .map_or_else(|| self.player_name.clone(), |d| d.name.clone()),
            difficulty: self.session_draft.as_ref().map_or(0, |d| d.difficulty),
            life_max: self.player.as_ref().map_or(100, |p| p.max_hp),
            mana_max: 20,
            is_favorite: false,
            playable: true,
            deletable: false,
        }
    }

    /// `Main.ErasePlayer` (`Main.cs:4772-4791`) for the port: no `.plr`
    /// exists (plan §9.4), so `erase_player(None)` drops the entry in memory
    /// only — the reopen's re-list rebuilds from the (unchanged) disk state,
    /// which is what the vanilla call's file deletion achieves.
    fn erase_player_entry(&mut self, index: usize) {
        let _ = ui::save_list::erase_player(None);
        if index < self.players.len() {
            self.players.remove(index);
        }
    }

    /// `Main.EraseWorld` (`Main.cs:4796-4813`): the `.trwld` and its sidecars
    /// go (`save_list::erase_world`); the reopen's re-list then reads the
    /// directory without it.
    fn erase_world_entry(&mut self, index: usize) {
        if let Some(path) = self.worlds.get(index).and_then(|w| w.path.clone()) {
            if let Err(e) = ui::save_list::erase_world(&path) {
                log::warn!("world erase failed: {e:#}");
            }
        }
    }

    /// The Death-font width a large `UITextPanel` measures (`UITextPanel.cs:
    /// 126-131`), or the bitmap fallback's — the front screens' `header_w`
    /// convention (the P3 `title_width` rule).
    fn front_header_w(&self, text: &str) -> f32 {
        match (&self.lookups.death_font, self.renderer.as_ref()) {
            (Some(f), _) => font_width(f, text, 1.0),
            (None, Some(r)) => r.text_width(text, 1.0),
            (None, None) => 0.0,
        }
    }

    /// One left click on a live front screen: the card-hit table routed into
    /// the flow (`UICharacterSelect.cs:108-133` / `UIWorldSelect.cs:110-135`'s
    /// handlers through the App). The favourite toggles re-order the list like
    /// `FavoriteButtonClick`'s `UpdateOrder` (`UICharacterListItem.cs:280-282`
    /// / `UIWorldListItem.cs:455-458`).
    fn handle_front_click(&mut self) {
        // `FrontScreen` stopped being `Copy` when the keyboard variant grew a
        // payload — the snapshot clones instead of moving out of `self`.
        let Some(front) = self.front.clone() else { return };
        // A click clears the notice, like `Menu::activate`'s `hint = None`.
        self.front_hint = None;
        let (sw, sh) = self.screen_size();
        let screen = Vec2::new(sw, sh);
        let mouse = self.mouse_pos;
        match front {
            FrontScreen::CharSelect => {
                let l = ui::char_select::resolved_layout(
                    screen,
                    self.front_header_w(ui::char_select::SELECT_PLAYER),
                    self.players.len(),
                );
                let favorites: Vec<bool> = self.players.iter().map(|p| p.is_favorite).collect();
                let mut flow = None;
                let mut delete_name = None;
                match ui::char_select::hit(&l, &favorites, mouse) {
                    Some(ui::char_select::CardAction::Back) => flow = Some(FlowClick::CharBack),
                    // `NewCharacterClick` (`:108-114`): a fresh `PendingPlayer`
                    // and the creation screen (`FlowClick::CharNew`).
                    Some(ui::char_select::CardAction::New) => flow = Some(FlowClick::CharNew),
                    // `PlayGame` (`:256-261`): only a playable entry reaches
                    // `SelectPlayer` (`loadStatus == Ok`); the rest is a dead
                    // click, like the source.
                    Some(ui::char_select::CardAction::Play(i))
                        if self.players.get(i).is_some_and(|p| p.playable) =>
                    {
                        flow = Some(FlowClick::CharPlay);
                    }
                    // `FavoriteButtonClick` (`:264-283`): the flag flips and
                    // the list re-orders (`UpdateOrder`). No `.plr` exists to
                    // persist it — the toggle is session-only (documented
                    // deviation).
                    Some(ui::char_select::CardAction::ToggleFavorite(i)) => {
                        if let Some(p) = self.players.get_mut(i) {
                            p.is_favorite = !p.is_favorite;
                        }
                        ui::char_select::sort_players(&mut self.players);
                    }
                    // `RenameButtonClick` (`:196-206`): the virtual keyboard
                    // opens over the card's name — `Lang.menu[45]` "Enter
                    // Character Name:", `allowEmpty: true`, the ctor's
                    // 20-character cap (`:199`, `UIVirtualKeyboard.cs:159`).
                    // There is no `.plr` writer to rename into (spec §9.4),
                    // so Submit stays the not-ported notice (Task 6's flow).
                    Some(ui::char_select::CardAction::Rename(i)) => {
                        let name =
                            self.players.get(i).map(|p| p.name.clone()).unwrap_or_default();
                        self.open_keyboard(
                            ui::keyboard::Request {
                                label: ui::keyboard::ENTER_CHARACTER_NAME.into(),
                                initial: name,
                                allow_empty: true,
                                max_length: 20,
                            },
                            FrontScreen::CharSelect,
                        );
                    }
                    // `DeleteButtonClick` (`:242-254`): parks the target,
                    // then `menuMode = 5`. The favourited gate never subscribed
                    // the handler (`:114-117`), so only `deletable` entries
                    // reach it — and the implicit session entry has no file.
                    Some(ui::char_select::CardAction::Delete(i))
                        if self.players.get(i).is_some_and(|p| p.deletable) =>
                    {
                        delete_name = self.players.get(i).map(|p| p.name.clone());
                        flow = Some(FlowClick::CharDelete(i));
                    }
                    _ => {}
                }
                if let Some(click) = flow {
                    let delete_index = match click {
                        FlowClick::CharDelete(i) | FlowClick::WorldDelete(i) => Some(i),
                        _ => None,
                    };
                    let (front, backup, page) =
                        next_flow_state(Some(front), self.front_backup.clone(), None, click);
                    self.front_backup = backup;
                    // Review Focus 1: the table result is the App's state —
                    // `front` is written on every hop, so the page arms below
                    // (`menuMode = 5` / `= 0`) clear the 888 side
                    // (`Main.cs:51066-51069`'s else branch) and the legacy page
                    // goes live instead of hiding under the front screen.
                    // `front` stays owned for the `None => match front` tail
                    // below (`FrontScreen` is no longer `Copy`).
                    self.front = front.clone();
                    match page {
                        // Mode 5 opens with the parked name/index
                        // (`Main.selectedPlayer`, `UICharacterListItem.cs:
                        // 246-249`).
                        Some(menu::Page::ConfirmDeletePlayer) => {
                            self.park_menu();
                            let mut m = menu::Menu::open_confirm_delete_player(
                                delete_name.unwrap_or_default(),
                            );
                            m.confirm_index = delete_index.unwrap_or(0);
                            self.menu = Some(m);
                        }
                        Some(p) => {
                            // A plain legacy page (the Back row's
                            // `menuMode = 0`).
                            self.park_menu();
                            let mut m = menu::Menu::new(self.player_name.clone());
                            m.page = p;
                            self.menu = Some(m);
                        }
                        None => match front {
                            Some(f) => self.enter_front(f),
                            None => {
                                self.front = None;
                                self.park_menu();
                            }
                        },
                    }
                }
            }
            FrontScreen::CharCreation(_) => {
                // The creator's click table (`UICharacterCreation`'s handlers
                // through the App): the categories, the Back/Create pair, and
                // the live middle menu's controls. The HSL rows drag in the
                // draw pass, not here (`UIColoredSlider`'s lock latch).
                let l = ui::char_creation::layout(screen, 0.0);
                // The screen-state mutations run under the borrow; the
                // navigation clicks clone the payload they need and drop it.
                enum Nav {
                    None,
                    Name(ui::char_creation::CreationState),
                    Create(ui::char_creation::CreationState),
                    Back(ui::char_creation::CreationState),
                }
                let nav = {
                    let Some(FrontScreen::CharCreation(state)) = self.front.as_mut() else {
                        return;
                    };
                    match ui::char_creation::click_at(&l, state.category, mouse) {
                        // `SelectColorPicker`/`Click_*` (`:1810-1833`): the
                        // picker selects and the middle menu follows
                        // (`middle_for`), seeding `_currentColorHSL` from the
                        // live colour — the draw re-derives it.
                        ui::char_creation::Click::Category(id) => {
                            state.category = id;
                            Nav::None
                        }
                        // `UIDifficultyButton.LeftMouseDown`
                        // (`UIDifficultyButton.cs:67-71`).
                        ui::char_creation::Click::Difficulty(d) => {
                            state.draft.difficulty = d;
                            Nav::None
                        }
                        // `UIHairStyleButton.LeftMouseDown` (`UIHairStyleButton.
                        // cs:66-70`) + `RecordThatHairWasSelected` (`:306-309`):
                        // the auto-assign gate closes.
                        ui::char_creation::Click::Hair(h) => {
                            state.draft.hair = h;
                            state.hair_picked = true;
                            Nav::None
                        }
                        // `Click_CharClothStyle` (`:1277-1291`): the armor
                        // guard is out of scope (no armor preview); the
                        // variant lands and the hair auto-assigns while
                        // untouched (`TryAutoAssigningHair`, `:630-637`).
                        ui::char_creation::Click::ClothStyle(v) => {
                            state.draft.skin_variant = v;
                            if !state.hair_picked {
                                if let Some(hair) =
                                    ui::char_creation::default_hairstyle_for_cloth_style(v)
                                {
                                    state.draft.hair = hair;
                                }
                            }
                            Nav::None
                        }
                        ui::char_creation::Click::Name => Nav::Name(state.clone()),
                        ui::char_creation::Click::Create => Nav::Create(state.clone()),
                        ui::char_creation::Click::Back => Nav::Back(state.clone()),
                        ui::char_creation::Click::None => Nav::None,
                    }
                };
                match nav {
                    Nav::None => {}
                    Nav::Name(state) => {
                        // `Click_Naming` (`:1659-1666`): the name clears
                        // (`Main.clrInput()`'s effect on the draft), the
                        // keyboard opens with `allowEmpty: true`.
                        let back = FrontScreen::CharCreation(state);
                        self.open_keyboard_with_tail(
                            ui::keyboard::Request {
                                label: ui::keyboard::ENTER_CHARACTER_NAME.into(),
                                initial: String::new(),
                                allow_empty: true,
                                max_length: ui::char_creation::MAX_NAME_LENGTH,
                            },
                            back,
                            KeyboardTail::Name,
                        );
                    }
                    Nav::Create(state) => {
                        // `Click_NamingAndCreating` (`:1668-1682`): an empty
                        // name opens the naming keyboard (no `allowEmpty` —
                        // the ctor default `false`), a filled one finishes.
                        if state.draft.name.is_empty() {
                            let back = FrontScreen::CharCreation(state);
                            self.open_keyboard_with_tail(
                                ui::keyboard::Request {
                                    label: ui::keyboard::ENTER_CHARACTER_NAME.into(),
                                    initial: String::new(),
                                    allow_empty: false,
                                    max_length: ui::char_creation::MAX_NAME_LENGTH,
                                },
                                back,
                                KeyboardTail::NameAndCreate,
                            );
                        } else {
                            self.finish_creating_character();
                        }
                    }
                    Nav::Back(state) => {
                        // `GoBack` (`:1210-1222`): dirty parks the exact
                        // screen for the mode-40 round trip
                        // (`BackupConfirmationState`, `:1215`); clean goes
                        // straight back to the char select
                        // (`OpenCharacterSelectUI`).
                        if state.draft.dirty {
                            let name =
                                (!state.draft.name.is_empty()).then(|| state.draft.name.clone());
                            self.front = None;
                            self.front_backup = Some(FrontScreen::CharCreation(state));
                            self.park_menu();
                            self.menu = Some(menu::Menu::open_confirm_discard(name));
                        } else {
                            self.front = None;
                            self.enter_front(FrontScreen::CharSelect);
                        }
                    }
                }
            }
            FrontScreen::WorldCreation(_) => {
                // The world creator's click table (`UIWorldCreation`'s
                // handlers through the App): the two plate icons, the two
                // plates, the three option rows and the Back/Create pair.
                let l = ui::world_creation::layout(screen, 0.0);
                // The draft mutations run under the borrow; the keyboard
                // clicks clone the draft and drop it, and the navigation
                // clicks ride the flow table like the select arms do.
                enum Nav {
                    None,
                    Name(ui::world_creation::NewWorldDraft),
                    Seed(ui::world_creation::NewWorldDraft),
                    NameAndCreate(ui::world_creation::NewWorldDraft),
                }
                let world_count = self.worlds.len();
                let mut nav = Nav::None;
                let mut flow = None;
                let mut create_draft: Option<ui::world_creation::NewWorldDraft> = None;
                {
                    let Some(FrontScreen::WorldCreation(draft)) = self.front.as_mut() else {
                        return;
                    };
                    match ui::world_creation::click_at(&l, mouse) {
                        // `ClickRandomizeName` (`:497-503`): the port's
                        // `AssignRandomWorldName` stand-in re-derives the
                        // name (`:1002-1025`'s composition tables have no
                        // localization port — documented).
                        ui::world_creation::Click::RandomName => {
                            draft.assign_random_world_name(world_count);
                        }
                        // `ClickSizeOption`/`ClickDifficultyOption`/
                        // `ClickEvilOption` (`:541-575`): the pick lands and
                        // `UpdatePreviewPlate` re-derives draw-side.
                        ui::world_creation::Click::Size(size) => draft.size = size,
                        ui::world_creation::Click::Difficulty(difficulty) => {
                            draft.difficulty = difficulty;
                        }
                        ui::world_creation::Click::Evil(evil) => draft.evil = evil,
                        // `Click_SetName` (`:689-695`): `Lang.menu[48]`,
                        // `allowEmpty: true`, the 27 cap (`:693`).
                        ui::world_creation::Click::Name => nav = Nav::Name(draft.clone()),
                        // `Click_SetSeed` -> `OpenSeedInputMenu` (`:697-708`):
                        // "UI.EnterSeed", `allowEmpty: true`, no cap (the
                        // source's `int.MaxValue`); the icon's Advanced-tab
                        // click degrades to the same keyboard (plan
                        // Uncertainty 4).
                        ui::world_creation::Click::Seed => nav = Nav::Seed(draft.clone()),
                        // `Click_NamingAndCreating` (`:710-724`): an empty
                        // name opens the naming keyboard (`allowEmpty: false`,
                        // 27), a filled one finishes (`:722`).
                        ui::world_creation::Click::Create => {
                            if draft.name.is_empty() {
                                nav = Nav::NameAndCreate(draft.clone());
                            } else {
                                // `FinishCreatingWorld` (`:845-863`) saves
                                // before the table's re-list takes over.
                                create_draft = Some(draft.clone());
                                flow = Some(FlowClick::WorldCreate);
                            }
                        }
                        // `Click_GoBack` -> `GoBack` -> `OpenWorldSelectUI`
                        // (`:665-674`) — no discard confirm on this screen.
                        ui::world_creation::Click::Back => {
                            flow = Some(FlowClick::WorldCreateBack);
                        }
                        ui::world_creation::Click::None => {}
                    }
                }
                match nav {
                    Nav::None => {}
                    Nav::Name(draft) => {
                        // The ctor's `startingText` is "" — the box opens
                        // empty even on a renamed draft (`:693`).
                        self.open_keyboard_with_tail(
                            ui::keyboard::Request {
                                label: ui::keyboard::ENTER_WORLD_NAME.into(),
                                initial: String::new(),
                                allow_empty: true,
                                max_length: ui::world_creation::MAX_NAME_LENGTH,
                            },
                            FrontScreen::WorldCreation(draft),
                            KeyboardTail::WorldName,
                        );
                    }
                    Nav::Seed(draft) => {
                        // `UI.EnterSeed` ("Enter Seed (Leave Blank For
                        // Random)"), `allowEmpty: true`, `int.MaxValue`
                        // (`UIWorldCreation.cs:706`); the starting text is ""
                        // there too (`:706`).
                        self.open_keyboard_with_tail(
                            ui::keyboard::Request {
                                label: "Enter Seed (Leave Blank For Random)".into(),
                                initial: String::new(),
                                allow_empty: true,
                                max_length: usize::MAX,
                            },
                            FrontScreen::WorldCreation(draft),
                            KeyboardTail::WorldSeed,
                        );
                    }
                    Nav::NameAndCreate(draft) => {
                        self.open_keyboard_with_tail(
                            ui::keyboard::Request {
                                label: ui::keyboard::ENTER_WORLD_NAME.into(),
                                initial: String::new(),
                                allow_empty: false,
                                max_length: ui::world_creation::MAX_NAME_LENGTH,
                            },
                            FrontScreen::WorldCreation(draft),
                            KeyboardTail::WorldNameAndCreate,
                        );
                    }
                }
                // The Create save (`:845-863`) runs before the table hop; a
                // failure shows the not-ported notice (the vanilla flow has
                // no error UI on this hop) and the re-list still happens.
                if let Some(draft) = create_draft {
                    match ui::world_creation::create_world(&draft) {
                        Ok(path) => {
                            log::info!("created '{}' -> {}", draft.name.trim(), path.display());
                        }
                        Err(e) => {
                            self.front_hint = Some(format!("world creation failed: {e:#}"));
                        }
                    }
                }
                if let Some(click) = flow {
                    let (front, backup, page) =
                        next_flow_state(Some(front), self.front_backup.clone(), None, click);
                    self.front_backup = backup;
                    // Review Focus 1: the table result is the App's state —
                    // `front` is written on every hop, so the page arms below
                    // would clear the 888 side; these two clicks always land
                    // on world select through the openers' re-list.
                    self.front = front.clone();
                    match page {
                        Some(p) => {
                            self.park_menu();
                            let mut m = menu::Menu::new(self.player_name.clone());
                            m.page = p;
                            self.menu = Some(m);
                        }
                        None => match front {
                            Some(f) => self.enter_front(f),
                            None => {
                                self.front = None;
                                self.park_menu();
                            }
                        },
                    }
                }
            }
            FrontScreen::WorldSelect => {
                // The world screen's ctor is the char select's twin
                // (`UIWorldSelect.cs:40-87`), so the shared `resolved_layout`
                // lives on `char_select` and serves both hit-tests.
                let l = ui::char_select::resolved_layout(
                    screen,
                    self.front_header_w(ui::world_select::SELECT_WORLD),
                    self.worlds.len(),
                );
                let favorites: Vec<bool> = self.worlds.iter().map(|w| w.is_favorite).collect();
                let mut flow = None;
                let mut delete_name = None;
                match ui::world_select::hit(&l, &favorites, mouse) {
                    Some(ui::world_select::CardAction::Back) => flow = Some(FlowClick::WorldBack),
                    // `NewWorldClick` (`:110-116`): the stand-in name reads
                    // `Main.WorldList.Count + 1` and the Journey default reads
                    // the active player's difficulty (`UIWorldCreation.cs:
                    // 319-322`) — the session entry's.
                    Some(ui::world_select::CardAction::New) => {
                        flow = Some(FlowClick::WorldNew(
                            self.worlds.len(),
                            self.session_entry().difficulty,
                        ));
                    }
                    // `PlayGame` (`:360-374`): `_data.IsValid` gates the
                    // launch; an invalid world's click is dead, like the
                    // source's rejection route (out of scope, spec §9.4).
                    Some(ui::world_select::CardAction::Play(i))
                        if self.worlds.get(i).is_some_and(|w| w.valid) =>
                    {
                        flow = Some(FlowClick::WorldPlay);
                    }
                    // `FavoriteButtonClick` (`:439-458`): `ToggleFavorite`
                    // saves (`FileData.cs:38-40`) and the list re-orders. The
                    // port's marker is the `.trfav` sidecar
                    // (`save_list::WorldEntry::is_favorite`'s reader).
                    Some(ui::world_select::CardAction::ToggleFavorite(i)) => {
                        if let Some(w) = self.worlds.get_mut(i) {
                            w.is_favorite = !w.is_favorite;
                            if let Some(path) = &w.path {
                                let marker = path.with_extension("trfav");
                                if w.is_favorite {
                                    let _ = std::fs::write(&marker, b"");
                                } else {
                                    let _ = std::fs::remove_file(&marker);
                                }
                            }
                        }
                        ui::world_select::sort_worlds(&mut self.worlds, false);
                    }
                    // `RenameButtonClick` (`UIWorldListItem.cs:415-421`): the
                    // world twin — `Lang.menu[48]` "Enter World Name:",
                    // `allowEmpty: true`, the 27-character cap (`:419`).
                    Some(ui::world_select::CardAction::Rename(i)) => {
                        let name =
                            self.worlds.get(i).map(|w| w.name.clone()).unwrap_or_default();
                        self.open_keyboard(
                            ui::keyboard::Request {
                                label: ui::keyboard::ENTER_WORLD_NAME.into(),
                                initial: name,
                                allow_empty: true,
                                max_length: 27,
                            },
                            FrontScreen::WorldSelect,
                        );
                    }
                    // `DeleteButtonClick` (`:346-358`): parks the target,
                    // then `menuMode = 9`.
                    Some(ui::world_select::CardAction::Delete(i))
                        if self.worlds.get(i).is_some_and(|w| w.path.is_some()) =>
                    {
                        delete_name = self.worlds.get(i).map(|w| w.name.clone());
                        flow = Some(FlowClick::WorldDelete(i));
                    }
                    _ => {}
                }
                if let Some(click) = flow {
                    let delete_index = match click {
                        FlowClick::CharDelete(i) | FlowClick::WorldDelete(i) => Some(i),
                        _ => None,
                    };
                    let (front, backup, page) =
                        next_flow_state(Some(front), self.front_backup.clone(), None, click);
                    self.front_backup = backup;
                    // Review Focus 1: the table result is the App's state —
                    // `front` is written on every hop, so the page arms below
                    // (`menuMode = 9` / `= 1`) clear the 888 side
                    // (`Main.cs:51066-51069`'s else branch) and the legacy page
                    // goes live instead of hiding under the front screen.
                    // `front` stays owned for the `None => match front` tail
                    // below (`FrontScreen` is no longer `Copy`).
                    self.front = front.clone();
                    match page {
                        // Mode 9 opens with the parked name/index
                        // (`Main.selectedWorld`, `UIWorldListItem.cs:350-353`).
                        Some(menu::Page::ConfirmDeleteWorld) => {
                            self.park_menu();
                            let mut m = menu::Menu::open_confirm_delete_world(
                                delete_name.unwrap_or_default(),
                            );
                            m.confirm_index = delete_index.unwrap_or(0);
                            self.menu = Some(m);
                        }
                        Some(p) => {
                            self.park_menu();
                            let mut m = menu::Menu::new(self.player_name.clone());
                            m.page = p;
                            self.menu = Some(m);
                        }
                        None => match front {
                            Some(f) => self.enter_front(f),
                            None => {
                                // The launch: everything closes and the
                                // settings apply (`close_menu`'s tail).
                                self.front = None;
                                self.park_menu();
                            }
                        },
                    }
                }
            }
            FrontScreen::Keyboard(_) => {
                // The keyboard's own click table (`UIVirtualKeyboard`'s
                // handlers): the on-screen keys type, the toggles cycle the
                // `KeyState`, Submit re-checks the validation, Cancel runs
                // the `_cancelAction`.
                let Some(FrontScreen::Keyboard(session)) = self.front.as_mut() else {
                    return;
                };
                let l = ui::keyboard::layout(screen, self.vk_grid);
                match ui::keyboard::click_at(&session.state, &l, self.vk_grid, mouse) {
                    ui::keyboard::Click::Char(c) => ui::keyboard::type_char(&mut session.state, c),
                    // The shift button: `(keyState != Shift) ? Shift : Default`
                    // (`:250-254`).
                    ui::keyboard::Click::Shift => {
                        let next = if session.state.key_state == ui::keyboard::KeyState::Shift {
                            ui::keyboard::KeyState::Default
                        } else {
                            ui::keyboard::KeyState::Shift
                        };
                        session.state.set_key_state(next);
                    }
                    // The symbol button: `(keyState != Symbol) ? Symbol : Default`
                    // (`:284-289`).
                    ui::keyboard::Click::Symbol => {
                        let next = if session.state.key_state == ui::keyboard::KeyState::Symbol {
                            ui::keyboard::KeyState::Default
                        } else {
                            ui::keyboard::KeyState::Symbol
                        };
                        session.state.set_key_state(next);
                    }
                    // `PressSpace` writes without the Shift unlatch (`:491-502`).
                    ui::keyboard::Click::Space => ui::keyboard::write_str(&mut session.state, " "),
                    // The backspace key pops (`:392-397`).
                    ui::keyboard::Click::Backspace => session.state.backspace(),
                    // The desktop pair's handler checks the validation first
                    // (`:358-365`); the small pair's `InternalSubmit` re-checks
                    // too (`:970-978`) — the invalid click is dead either way.
                    ui::keyboard::Click::Submit => {
                        if ui::keyboard::text_valid(&session.state.text, session.request.allow_empty)
                        {
                            self.keyboard_submit();
                        }
                    }
                    // `_cancelAction` (`:319-323`, `:385-389`).
                    ui::keyboard::Click::Cancel => self.keyboard_cancel(),
                    ui::keyboard::Click::None => {}
                }
            }
        }
    }

    /// Applies a `TR_UI` preset (App init): the screens re-list through the
    /// real openers; the legacy pages park the front exactly as their vanilla
    /// entries do — modes 5/9 clear the 888 state (`Main.cs:51066-51069`'s
    /// else branch), mode 40 parks it in `front_backup`
    /// (`UICharacterCreation.BackupConfirmationState`).
    fn preset_ui(&mut self, preset: TrUiPreset) {
        match preset {
            TrUiPreset::CharSelect => self.enter_front(FrontScreen::CharSelect),
            TrUiPreset::WorldSelect => self.enter_front(FrontScreen::WorldSelect),
            // Task 7's screen: `NewCharacterClick`'s fresh draft
            // (`UICharacterSelect.cs:108-114`).
            TrUiPreset::CharNew => {
                self.enter_front(FrontScreen::CharCreation(
                    ui::char_creation::CreationState::new(),
                ));
            }
            // Task 8's screen (`NewWorldClick`, `UIWorldSelect.cs:110-116`):
            // the stand-in name reads `Main.WorldList.Count + 1`, so the list
            // populates first; the Journey default reads the session player
            // (`UIWorldCreation.cs:319-322`).
            TrUiPreset::WorldNew => {
                self.enter_front(FrontScreen::WorldSelect);
                let count = self.worlds.len();
                let active = self.session_entry().difficulty;
                self.enter_front(FrontScreen::WorldCreation(
                    ui::world_creation::NewWorldDraft::new(count, active),
                ));
            }
            TrUiPreset::Keyboard => {
                // The keyboard screenshot (`TR_UI=vk`): the naming request the
                // character creator opens with (`Lang.menu[45]`, empty text —
                // `Player.cs:826`'s fresh name, so the Shift state seeds),
                // Cancel returning to the char select. `TR_VK_GRID=1` shows
                // the 50-key grid layer instead (Review Focus 6).
                self.open_keyboard(
                    ui::keyboard::Request {
                        label: ui::keyboard::ENTER_CHARACTER_NAME.into(),
                        initial: String::new(),
                        allow_empty: true,
                        max_length: 20,
                    },
                    FrontScreen::CharSelect,
                );
            }
            TrUiPreset::Credits => {
                self.front = None;
                self.front_backup = None;
                self.menu = Some(menu::Menu::open_credits());
            }
            TrUiPreset::DelPlayer => {
                self.enter_front(FrontScreen::CharSelect);
                let name =
                    self.players.first().map(|p| p.name.clone()).unwrap_or_default();
                self.front = None;
                self.menu = Some(menu::Menu::open_confirm_delete_player(name));
            }
            TrUiPreset::DelWorld => {
                self.enter_front(FrontScreen::WorldSelect);
                let name = self.worlds.first().map(|w| w.name.clone()).unwrap_or_default();
                self.front = None;
                self.menu = Some(menu::Menu::open_confirm_delete_world(name));
            }
            TrUiPreset::Discard => {
                // Mode 40's screenshot: the parked state is the character
                // creation — vanilla parks the live `UICharacterCreation`
                // (`BackupConfirmationState`, `UICharacterCreation.cs:1215`),
                // named so the header shows `Discard {0}?`.
                let mut state = ui::char_creation::CreationState::new();
                state.draft.name = "Zealot".into();
                state.draft.refresh_dirty();
                self.front = None;
                self.front_backup = Some(FrontScreen::CharCreation(state));
                self.menu = Some(menu::Menu::open_confirm_discard(Some("Zealot".into())));
            }
        }
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
            leg_frame: p.leg_frame,
            body_frame: p.body_frame,
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
            // `Player.InternalSaveMap` (`Player.cs:55046`): the explored map is
            // written alongside the player/world save.
            self.save_map();
        } else {
            log::info!("not saving: the loaded world is not a native .trwld");
        }
    }

    /// `MapHelper.SaveMap` (`MapHelper.cs:3007`): write `<world>.map` next to the
    /// world. Gated on `Main.mapEnabled` (`MapHelper.cs:3009`).
    fn save_map(&self) {
        if !self.settings.map_enabled {
            return;
        }
        let Some(map) = self.map.as_ref() else { return };
        let path = persist::map_path(&self.world_path);
        if let Err(e) = map::save_map_file(&self.world, &map.tiles, &path) {
            log::warn!("map save failed: {e}");
        }
    }

    /// `MapHelper.LoadMap`: load the explored map at world load. The wiped-map
    /// symptom (the map resetting on every restart) came from never calling this.
    fn load_map(&mut self) {
        if !self.world_path.extension().is_some_and(|e| e == "trwld")
            || !self.settings.map_enabled
        {
            return;
        }
        let path = persist::map_path(&self.world_path);
        let Some(m) = self.map.as_mut() else { return };
        if m.load(&self.world, &path) {
            log::info!("map: loaded {}", path.display());
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
        // `Main.cs:19751-19765`: with Autosave on, `WorldGen.saveAndPlay` runs
        // every 600000 ms (10 minutes); the player/map save rides along.
        if self.menu.is_none() && self.front.is_none() && self.settings.auto_save {
            self.autosave += dt;
            if self.autosave >= 600.0 {
                self.autosave = 0.0;
                self.save_world();
            }
        }

        let screen = self
            .gpu
            .as_ref()
            .map(|g| Vec2::new(g.config.width as f32, g.config.height as f32))
            .unwrap_or(Vec2::new(1600.0, 900.0));

        // `Main.DrawInterface_16_MapOrMinimap` (`Main.cs:45771-45783`): `mH =
        // 256` while the minimap HUD is live — `mapEnabled && !mapFullscreen
        // && mapStyle == 1` (the fullscreen map draws no HUD at all,
        // `Main.cs:62147-62181`); the `+ 600` push-up clamp sits in `main_m_h`.
        let minimap_live = self.settings.map_enabled
            && self.map.as_ref().is_some_and(|m| !m.open)
            && self.map_style == 1;
        let m_h = ui::minimap::main_m_h(minimap_live, screen.y);

        // `FocusHelper.cs:160-163`: while the focused window draws the game's own
        // cursor the OS pointer is hidden (`Main.instance.IsMouseVisible = false`).
        // We only draw the vanilla cursor together with the in-world HUD so far, so
        // the OS pointer stays in menus and on the fullscreen map.
        let vanilla_cursor = self.menu.is_none()
            && self.front.is_none()
            && !no_hud()
            && !(self.settings.map_enabled && self.map.as_ref().is_some_and(|m| m.open));
        if self.os_cursor_hidden != Some(vanilla_cursor) {
            if let Some(w) = &self.window {
                w.set_cursor_visible(!vanilla_cursor);
            }
            self.os_cursor_hidden = Some(vanilla_cursor);
        }

        // `Main.Camera.Center` + the volume sliders for `LegacySoundPlayer.PlaySound`.
        self.audio.set_listener(sound::Listener {
            camera_center: (self.camera.x, self.camera.y),
            sound_volume: self.settings.sound_volume,
            ambient_volume: self.settings.ambient_volume,
            quiet_ambient_sounds: false,
            game_menu: self.menu.is_some() || self.front.is_some(),
        });

        if self.menu.is_none() && self.front.is_none() {
            // `MinimapFrame.Update` runs before the inventory's click handling
            // (`MinimapFrameManager.cs:45`, `Main.cs:41037`'s `ItemSlot.Handle`).
            self.handle_minimap_clicks();
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
                m.update(&self.world, self.player.as_ref(), &self.light, dt);
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
        let player_alpha = (self.player_step.residual() / (1.0 / 60.0)).clamp(0.0, 1.0);
        let mut render_player = self.player.as_ref().map(|p| p.interpolated(player_alpha));
        // `PlayerFrame` item-use branch: while a swing is active the body shows the use
        // rows instead of the idle/air pose. `swing` counts down from `SWING_TIME`,
        // which is `itemAnimation / itemAnimationMax` in the original.
        if let Some(p) = render_player.as_mut() {
            if self.swing > 0.0 && self.inventory[self.selected].count > 0 {
                p.apply_item_use_frame(self.swing / SWING_TIME);
            }
        }
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
                front,
                front_hint,
                settings,
                remote_players,
                map,
                hell_black_box_bottom,
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
                // `Main.mouseColor = Main.selColor` runs every frame the colour
                // screen is open (`Main.cs:50324`), and `CursorColor`
                // (`Main.cs:52490`) reads that static, so the tint follows the
                // bars live during a drag and keeps the stored colour once the
                // menu closes. The menu owns the live copy while it is open.
                self.cursor.mouse_color = match menu {
                    Some(m) => m.settings.mouse_color,
                    None => settings.mouse_color,
                };
                // `Main.CursorColor()` (`Main.cs:52477`): the cursor fade pulse.
                self.cursor.update();
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
                // Snap the render camera so integer world coordinates land on whole
                // screen pixels at any zoom. With a fractional camera, nearest-neighbour
                // sampling makes every tile texture crawl sub-pixel as the camera moves
                // while the player (drawn with `snap_to_screen`) stayed crisp, so the
                // world appeared to slide under the character. Rounding `cam * zoom`
                // keeps the step at one screen pixel even at 1.5x / 2x.
                let draw_cam = (*camera * *zoom).round() / *zoom;
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
                    draw_cam,
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
                    cave_parallax(settings.parallax_scroll),
                    bg_alpha_front,
                    bg_alpha_far,
                    light,
                    hell_black_box_bottom,
                );
                if !reuse_world {
                    tile_stats = draw_tiles(
                        renderer,
                        world,
                        lookups,
                        atlas.as_ref(),
                        draw_cam,
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
                    // `PlayerDrawLayers.DrawPlayer_27_HeldItem`: the item is only drawn
                    // while it is being used (`itemAnimation > 0 && useStyle != 0`), so
                    // the hand is empty at rest. Hold-style items (torches etc.) are not
                    // modelled yet.
                    if *swing > 0.0 {
                        if let Some(p) = render_player.as_ref() {
                            if let Some(slot) = inventory.get(*selected) {
                                if slot.count > 0 {
                                    tile_stats.quads += draw_held_item(
                                        renderer, lookups, light, &style, p, slot.kind,
                                        *swing, anim_time,
                                    );
                                }
                            }
                        }
                    }
                    *world_key = Some(key);
                }

                rain.draw(renderer, lookups, light, draw_cam, *zoom, screen);
                for p in projectiles.iter() {
                    p.draw(renderer, lookups, light, draw_cam, *zoom, screen);
                }

                let boss = npcs
                    .iter()
                    .find(|n| n.kind == NpcKind::KingSlime && n.hp > 0)
                    .map(|n| (n.hp, n.max_hp));
                // `Main.cs:62147-62181`: while the fullscreen map is open `Main.Draw`
                // returns early after `DrawMap`, before `DrawInterface` (`62432`), so
                // the HUD (hotbar, life, inventory) is not drawn over the map.
                let fullscreen_map =
                    settings.map_enabled && map.as_ref().is_some_and(|m| m.open);
                // The minimap branch draws **before** the HUD: the vanilla
                // interface passes are ordered 16 (map/minimap,
                // `Main.cs:44796`) < 25/27/30 (resources/inventory/hotbar,
                // `:44831/:44841/:44861`), so the hearts and the open inventory
                // paint over the frame. `Main.DrawMap`'s `mapStyle == 1`
                // branch (`Main.cs:55503-55547`): the three vanilla passes in
                // draw order — the `DrawTo` black backdrop (`:55546`, via
                // `map_back` so it lands under the tiles), the tile pass
                // (`MapRenderer.DrawMap`, `:55609`) and the frame foreground
                // batch (`:55768-55769`). `mapStyle == 2` (the overlay) draws
                // nothing (out of P4 scope, the whole-UI P4 plan's
                // Uncertainty 7).
                let view_center = [camera.x / TILE_PX, camera.y / TILE_PX];
                if settings.map_enabled
                    && menu.is_none()
                    && front.is_none()
                    && !no_hud()
                    && !fullscreen_map
                    && self.map_style == 1
                {
                    if let Some(m) = map.as_ref() {
                        let pos = ui::minimap::draw_to_position(ui::minimap::minimap_origin(
                            screen,
                        ));
                        // `Main.mapMinimapAlpha = 1` (`Main.cs:66753`).
                        ui::minimap::draw_background(renderer, lookups, pos, 1.0);
                        m.draw_minimap(renderer, world, screen, view_center);
                        // The hover highlight is `MinimapFrame.Update`'s
                        // `buttonUnderMouse.IsHighlighted`
                        // (`MinimapFrame.cs:163`), resolved per frame in
                        // `handle_minimap_clicks` — like the source, where
                        // `DrawTo` runs `Update` before the foreground batch
                        // (`MinimapFrameManager.cs:45`, `Main.cs:55769`).
                        let s = ui::minimap::style_index(&settings.minimap_frame);
                        ui::minimap::draw_foreground(
                            renderer,
                            lookups,
                            s,
                            pos,
                            self.minimap_hover,
                        );
                    }
                }
                if let Some(front) = front.as_mut() {
                    // `Main.menuMode == 888` (`Main.cs:51059-51064`): the
                    // front-end draws over the world backdrop the frame
                    // already rendered — the draw site picks `front` first,
                    // then `menu` (Review Focus 1).
                    match front {
                        FrontScreen::CharSelect => {
                            let text = ui::char_select::SELECT_PLAYER;
                            let header_w = match &lookups.death_font {
                                Some(f) => font_width(f, text, 1.0),
                                None => renderer.text_width(text, 1.0),
                            };
                            ui::char_select::draw(
                                renderer,
                                lookups,
                                screen,
                                header_w,
                                &self.players,
                                self.mouse_pos,
                            );
                        }
                        FrontScreen::WorldSelect => {
                            let text = ui::world_select::SELECT_WORLD;
                            let header_w = match &lookups.death_font {
                                Some(f) => font_width(f, text, 1.0),
                                None => renderer.text_width(text, 1.0),
                            };
                            // The active player's difficulty for the joinable
                            // grey-out (`Main.ActivePlayerFileData.Player.
                            // difficulty`, `UIWorldSelect.cs:179`): the port's
                            // active character is the fresh session player —
                            // difficulty 0 (`Player.cs:1654`'s default).
                            ui::world_select::draw(
                                renderer,
                                lookups,
                                screen,
                                header_w,
                                &self.worlds,
                                0,
                                self.mouse_pos,
                            );
                        }
                        FrontScreen::CharCreation(state) => {
                            // `Main.MenuUI.Draw`'s `UICharacterCreation` state
                            // (`:1876-2022`'s Draw): the panel, the animated
                            // preview (`GlobalTimeWrappedHourly`'s clock —
                            // `anim_time`), the categories, the one middle
                            // menu, Back/Create and the hover tooltip. The
                            // HSL drag runs inside (the `CurrentLockedSlider`
                            // latch) off `front_mouse_down`.
                            ui::char_creation::draw(
                                renderer,
                                lookups,
                                player_assets,
                                state,
                                screen,
                                anim_time,
                                self.mouse_pos,
                                self.front_mouse_down,
                            );
                        }
                        FrontScreen::WorldCreation(draft) => {
                            // `Main.MenuUI.Draw`'s `UIWorldCreation` state
                            // (`:1024-1033`, minus the `SetupGamepadPoints`
                            // snap points and the `DrawSeedSystems` special-
                            // seed ring — both out of scope): the panel, the
                            // plates + preview, the three option rows, the
                            // description panel and Back/Create.
                            ui::world_creation::draw(
                                renderer,
                                lookups,
                                draft,
                                screen,
                                self.mouse_pos,
                            );
                        }
                        FrontScreen::Keyboard(session) => {
                            // `Main.MenuUI.Draw`'s keyboard state
                            // (`UIVirtualKeyboard.DrawSelf`, `:680-746`): the
                            // desktop layer by default (`ShowGamepadHints` is
                            // false without a gamepad), the 50-key grid under
                            // `TR_VK_GRID=1` (Review Focus 6). `frame_count`
                            // is the caret's `UITextBox._frameCount`.
                            let l = ui::keyboard::layout(screen, self.vk_grid);
                            ui::keyboard::draw(
                                renderer,
                                lookups,
                                &session.state,
                                &session.request,
                                &l,
                                self.vk_grid,
                                self.mouse_pos,
                                *frame_count,
                            );
                        }
                    }
                    // The port's not-ported notice for the front screens
                    // (ours, like `Menu.hint` — not in the original).
                    if let Some(hint) = front_hint {
                        let w = hud_text_width(renderer, lookups, hint, 0.8);
                        hud_text(
                            renderer,
                            lookups,
                            (screen.x - w) * 0.5,
                            screen.y - 64.0,
                            0.8,
                            [1.0, 0.8, 0.45, 1.0],
                            hint,
                        );
                    }
                } else if menu.is_none() && !no_hud() && !fullscreen_map {
                    ui::draw_hud(
                        renderer,
                        *camera,
                        *zoom,
                        screen,
                        lookups,
                        inventory,
                        &self.equip,
                        self.equip_page,
                        *selected,
                        &self.hotbar_scales,
                        player.as_ref(),
                        boss,
                        self.inventory_open,
                        self.crafting_open,
                        &self.mouse_item,
                        self.mouse_pos,
                        &self.cursor,
                        ui::UiPulse {
                            // `Main.mouseTextColor / 255` (`Main.cs:20355`).
                            text: self.mouse_text_color as f32 / 255.0,
                            cursor_scale: self.cursor.scale(),
                        },
                        m_h,
                        self.map_style,
                    );
                } else if let Some(menu) = menu.as_mut() {
                    menu.draw(renderer, lookups, screen, dt);
                }
                // Task 6b: upload the revealed tiles into the world-sized map texture
                // and clamp/recentre the fullscreen map. Vanilla centres the recentre
                // and the minimap on the view centre (`Main.cs:55420-55421`,
                // `55537-55538`): `(screenPosition + screenSize/2)/16` — `view_center`
                // is computed above, where the minimap branch consumes it.
                if settings.map_enabled {
                    if let Some(m) = map.as_mut() {
                        m.prepare(renderer, gpu, world, screen, view_center);
                    }
                }
                if let Some(map) = map.as_ref() {
                    if !no_hud() && menu.is_none() && front.is_none() && settings.map_enabled {
                        if map.open {
                            // `Main.cs:55483`: the biome `MapBG` backdrop is
                            // drawn before the wooden frame, so it goes into
                            // `map_back` first (`draw_full` pushes the frame).
                            // `Main.player[myPlayer]` always exists in vanilla;
                            // with no player we fall back to the view centre.
                            let player_tile = player
                                .as_ref()
                                .map(|p| {
                                    let (cx, cy) = p.center();
                                    ((cx / TILE_PX) as i32, (cy / TILE_PX) as i32)
                                })
                                .unwrap_or((
                                    (camera.x / TILE_PX) as i32,
                                    (camera.y / TILE_PX) as i32,
                                ));
                            if let Some(bg) =
                                map::map_bg_index(world, *camera, screen, player_tile)
                            {
                                let sky = [sky_tint[0], sky_tint[1], sky_tint[2], 1.0];
                                map::draw_map_background(renderer, bg, screen, sky);
                            }
                            map.draw_full(renderer, lookups, world, screen);
                            // `Main.cs:55931-55954`: the local player's head icon,
                            // drawn after the map tiles (`Main.cs:55609`).
                            if let Some(p) = player.as_ref() {
                                let (cx, cy) = p.center();
                                // `Main.cs:55938`: `position.Y + gfxOffY + height/2`.
                                let center_px = [cx, cy + p.gfx_off_y];
                                let icon_pos =
                                    map::head_icon_pos(center_px, map.pos, map.scale, screen);
                                // `Main.GetPlayerHeadBordersColor` (`Main.cs:56616-56618`)
                                // → `Main.teamColor[0]` = White (singleplayer).
                                draw_player_map_icon(
                                    renderer, player_assets, p, icon_pos, 1.0, [1.0; 4],
                                );
                            }
                        }
                        // The `else if self.map_style == 1` minimap branch
                        // (`Main.cs:55503-55547`) draws above `ui::draw_hud`
                        // now — vanilla renders interface layer 16 (the
                        // minimap, `Main.cs:44796`) before layers 25/27/30
                        // (`:44831/:44841/:44861`), so the hearts and the
                        // inventory paint over the frame.
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
                    draw_cam.to_array(),
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
        world_sim::camera::pan_camera(self, dt);
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

pub(crate) fn next_rand(state: &mut u64) -> u64 {
    *state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut z = *state;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

pub(crate) fn rand01(state: &mut u64) -> f32 {
    (next_rand(state) >> 40) as f32 / (1u32 << 24) as f32
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
mod front_flow_tests {
    use super::*;

    use menu::Page as P;

    #[test]
    fn the_flow_table_routes_every_front_hop() {
        // Char select Back -> menuMode 0 (`UICharacterSelect.cs:116-120`).
        let (front, backup, page) =
            next_flow_state(Some(FrontScreen::CharSelect), None, None, FlowClick::CharBack);
        assert_eq!((front.is_some(), backup.is_some(), page), (false, false, Some(P::Main)));
        // Play -> the world select (`Main.cs:66245-66249` + `:49468-49472`).
        let (front, backup, page) =
            next_flow_state(Some(FrontScreen::CharSelect), None, None, FlowClick::CharPlay);
        assert_eq!((front, backup.is_some(), page.is_some()), (Some(FrontScreen::WorldSelect), false, false));
        // Delete -> mode 5 clears the 888 state (`UICharacterListItem.cs:
        // 242-254`, `Main.cs:51066-51069`'s else branch).
        let (front, backup, page) =
            next_flow_state(Some(FrontScreen::CharSelect), None, None, FlowClick::CharDelete(2));
        assert_eq!(
            (front.is_some(), backup.is_some(), page),
            (false, false, Some(P::ConfirmDeletePlayer))
        );
        // World Back -> the char select (`UIWorldSelect.cs:118-122`, the
        // single-player arm of `menuMode = (!menuMultiplayer) ? 1 : 12`).
        let (front, backup, page) =
            next_flow_state(Some(FrontScreen::WorldSelect), None, None, FlowClick::WorldBack);
        assert_eq!((front, backup.is_some(), page.is_some()), (Some(FrontScreen::CharSelect), false, false));
        // World Play launches: both sides close (the loading flow is out of
        // scope, spec §9.4).
        let (front, backup, page) =
            next_flow_state(Some(FrontScreen::WorldSelect), None, None, FlowClick::WorldPlay);
        assert_eq!((front.is_some(), backup.is_some(), page.is_some()), (false, false, false));
        // World Delete -> mode 9 (`UIWorldListItem.cs:346-358`).
        let (front, backup, page) =
            next_flow_state(Some(FrontScreen::WorldSelect), None, None, FlowClick::WorldDelete(1));
        assert_eq!(
            (front.is_some(), backup.is_some(), page),
            (false, false, Some(P::ConfirmDeleteWorld))
        );
    }

    #[test]
    fn confirm_resolutions_reopen_their_screen() {
        // Mode 5: both branches reopen the char select (`Main.cs:49434-49439`).
        for click in [FlowClick::ConfirmYes, FlowClick::ConfirmNo] {
            let (front, backup, page) =
                next_flow_state(None, None, Some(P::ConfirmDeletePlayer), click);
            assert_eq!(front, Some(FrontScreen::CharSelect), "{click:?}");
            assert!((backup.is_none()) && page.is_none(), "{click:?}");
        }
        // Mode 9: the world select (`:49684-49690`).
        for click in [FlowClick::ConfirmYes, FlowClick::ConfirmNo] {
            let (front, backup, page) =
                next_flow_state(None, None, Some(P::ConfirmDeleteWorld), click);
            assert_eq!(front, Some(FrontScreen::WorldSelect), "{click:?}");
            assert!((backup.is_none()) && page.is_none(), "{click:?}");
        }
        // Mode 40 Yes: the parked state drops and the char select reopens
        // (`BackupConfirmationState = null` + `OpenCharacterSelectUI`,
        // `:49454-49459`).
        let (front, backup, page) = next_flow_state(
            None,
            Some(FrontScreen::CharSelect),
            Some(P::ConfirmDiscard),
            FlowClick::ConfirmYes,
        );
        assert_eq!((front.is_some(), backup.is_some(), page.is_some()), (true, false, false));
    }

    #[test]
    fn the_mode_40_no_branch_restores_the_exact_parked_screen() {
        // `Main.cs:49460-49467`: `menuMode = 888` +
        // `SetState(BackupConfirmationState)` — the parked value returns
        // untouched and the backup clears (Review Focus 1's round-trip;
        // `App.front_backup` is the port's `BackupConfirmationState`). The
        // creation screen joins with Task 7: its draft/category/grab payload
        // is exactly what `GoBack` parks (`UICharacterCreation.cs:1215`).
        let mut creation = ui::char_creation::CreationState::new();
        creation.category = 4;
        creation.draft.name = "Zealot".into();
        creation.hsl_grab = 2;
        for parked in [
            FrontScreen::CharSelect,
            FrontScreen::WorldSelect,
            FrontScreen::CharCreation(creation),
            FrontScreen::WorldCreation(ui::world_creation::NewWorldDraft::new(1, 0)),
        ] {
            let (front, backup, page) = next_flow_state(
                None,
                Some(parked.clone()),
                Some(P::ConfirmDiscard),
                FlowClick::ConfirmNo,
            );
            assert_eq!(front, Some(parked), "the exact parked screen returns");
            assert!((backup.is_none()) && page.is_none());
        }
    }

    #[test]
    fn front_and_menu_are_never_both_some() {
        // Review Focus 1: every row of the flow table keeps the single-mode
        // invariant `Main.menuMode` carries (`Main.cs:51059-51069` — 888
        // draws `MenuUI`, every other mode has `SetState(null)`). The matrix
        // covers every live `FrontScreen` variant — the keyboard session was
        // the Task 6 gap, the creation screen joins with Task 7.
        let session = || {
            let request = ui::keyboard::Request {
                label: ui::keyboard::ENTER_CHARACTER_NAME.into(),
                initial: String::new(),
                allow_empty: true,
                max_length: 20,
            };
            let state = ui::keyboard::KeyboardState::new(&request);
            Box::new(KeyboardSession {
                request,
                state,
                back: FrontScreen::CharSelect,
                tail: KeyboardTail::Rename,
            })
        };
        let fronts = [
            None,
            Some(FrontScreen::CharSelect),
            Some(FrontScreen::WorldSelect),
            Some(FrontScreen::CharCreation(ui::char_creation::CreationState::new())),
            // The world creator joins with Task 8.
            Some(FrontScreen::WorldCreation(ui::world_creation::NewWorldDraft::new(
                0, 0,
            ))),
            Some(FrontScreen::Keyboard(session())),
        ];
        let backs = [
            None,
            Some(FrontScreen::CharSelect),
            Some(FrontScreen::WorldSelect),
            Some(FrontScreen::CharCreation(ui::char_creation::CreationState::new())),
            Some(FrontScreen::WorldCreation(ui::world_creation::NewWorldDraft::new(
                0, 0,
            ))),
            Some(FrontScreen::Keyboard(session())),
        ];
        let pages = [
            None,
            Some(P::Main),
            Some(P::ConfirmDeletePlayer),
            Some(P::ConfirmDeleteWorld),
            Some(P::ConfirmDiscard),
            Some(P::Credits),
        ];
        let clicks = [
            FlowClick::CharBack,
            FlowClick::CharNew,
            FlowClick::CharPlay,
            FlowClick::CharDelete(0),
            FlowClick::WorldBack,
            FlowClick::WorldPlay,
            FlowClick::WorldDelete(0),
            FlowClick::WorldNew(0, 0),
            FlowClick::WorldCreateBack,
            FlowClick::WorldCreate,
            FlowClick::ConfirmYes,
            FlowClick::ConfirmNo,
        ];
        for front in &fronts {
            for backup in &backs {
                for page in &pages {
                    // The invariant holds on the way in: the App never parks
                    // both sides, so front+page inputs are unreachable.
                    if front.is_some() && page.is_some() {
                        continue;
                    }
                    for click in &clicks {
                        let (front, backup, menu) = next_flow_state(
                            front.clone(),
                            backup.clone(),
                            *page,
                            click.clone(),
                        );
                        assert!(
                            !(front.is_some() && menu.is_some()),
                            "{click:?} from front {front:?}/page {page:?} left both sides live"
                        );
                        let _ = backup;
                    }
                }
            }
        }
    }

    #[test]
    fn tr_ui_names_map_the_hook_table() {
        // The plan Task 5 hook list (the `TR_PAGE` pattern); unknown names
        // resolve to nothing and the match is case-insensitive.
        assert_eq!(tr_ui_preset("charsel"), Some(TrUiPreset::CharSelect));
        assert_eq!(tr_ui_preset("worldsel"), Some(TrUiPreset::WorldSelect));
        assert_eq!(tr_ui_preset("charnew"), Some(TrUiPreset::CharNew));
        assert_eq!(tr_ui_preset("worldnew"), Some(TrUiPreset::WorldNew));
        assert_eq!(tr_ui_preset("vk"), Some(TrUiPreset::Keyboard));
        assert_eq!(tr_ui_preset("credits"), Some(TrUiPreset::Credits));
        assert_eq!(tr_ui_preset("delplayer"), Some(TrUiPreset::DelPlayer));
        assert_eq!(tr_ui_preset("delworld"), Some(TrUiPreset::DelWorld));
        assert_eq!(tr_ui_preset("discard"), Some(TrUiPreset::Discard));
        assert_eq!(tr_ui_preset("CHARSEL"), Some(TrUiPreset::CharSelect));
        assert_eq!(tr_ui_preset("nonsense"), None);
    }

    // --- App-level click tests (Review Focus 1's regression) -----------------
    //
    // `next_flow_state` is pure — the bug these tests pin was in the
    // *application* of its result: `handle_front_click` set `menu` but never
    // the table's `front`, so the opened page lived under the still-live
    // screen.

    /// One list entry for the flow tests: playable, optionally deletable (the
    /// `DeleteButtonClick` gate, `UICharacterListItem.cs:242-254`).
    fn entry(name: &str, deletable: bool) -> ui::save_list::PlayerEntry {
        ui::save_list::PlayerEntry {
            name: name.into(),
            difficulty: 0,
            life_max: 100,
            mana_max: 20,
            is_favorite: false,
            playable: true,
            deletable,
        }
    }

    /// A headless App for the flow tests: no window/GPU/renderer, the audio
    /// pair without a content dir, an 8x8 world — everything the flow state
    /// does not read stays at its cheap default. `screen_size()` falls back
    /// to the deterministic 1280x720, so the tests aim the pointer through
    /// the same layout the click handler builds.
    fn flow_app(
        front: Option<FrontScreen>,
        players: Vec<ui::save_list::PlayerEntry>,
        worlds: Vec<ui::save_list::WorldEntry>,
    ) -> App {
        let world = World::new("flow".into(), 8, 8);
        let liquids = LiquidSystem::new(&world);
        App {
            window: None,
            gpu: None,
            renderer: None,
            atlas: None,
            world,
            lookups: assets::Lookups::default(),
            clouds: Vec::new(),
            sky: SkyAssets::default(),
            bg_style: 0,
            bg_delay: 0,
            bg_instant: 0,
            bg_alpha_front: [0.0; 16],
            bg_alpha_far: [0.0; 16],
            stars: Vec::new(),
            player_assets: PlayerAssets::default(),
            player: None,
            follow_player: false,
            player_step: FixedStepper::new(1.0 / 60.0, 8),
            npc_step: FixedStepper::new(1.0 / 60.0, 8),
            npcs: Vec::new(),
            npc_assets: NpcAssets::default(),
            npc_timer: 0.0,
            npc_rng: 0,
            attack_cd: 0.0,
            inventory: std::array::from_fn(|_| EMPTY_SLOT),
            equip: inv::EquipSlots::empty(),
            equip_page: 0,
            selected: 0,
            hotbar_scales: [0.75; HOTBAR],
            inventory_open: false,
            crafting_open: false,
            menu: None,
            front,
            front_backup: None,
            players,
            worlds,
            front_hint: None,
            front_mouse_down: false,
            session_draft: None,
            vk_grid: false,
            settings: settings::Settings::default(),
            player_name: "flow".into(),
            net: NetRole::Off,
            remote_players: HashMap::new(),
            local_id: 0,
            net_timer: 0.0,
            join_target: None,
            map: None,
            map_grab: None,
            minimap_hover: None,
            map_style: 1,
            audio: audio::Audio::new(None),
            music: audio::MusicSystem::new(None),
            last_hp: 100,
            autosave: 0.0,
            mouse_item: EMPTY_SLOT,
            ui_left_press: false,
            ui_right_press: false,
            mouse_pos: Vec2::ZERO,
            cursor: ui::cursor::CursorState::default(),
            os_cursor_hidden: None,
            mine_down: false,
            place_down: false,
            mine_target: None,
            mine_timer: 0.0,
            place_cooldown: 0.0,
            edit_target: None,
            world_path: PathBuf::new(),
            dev_break: None,
            dev_place: Vec::new(),
            camera: Vec2::ZERO,
            zoom: 1.0,
            zoom_pinned: false,
            debug: false,
            keys: HashSet::new(),
            mouse_down: false,
            last_mouse: Vec2::ZERO,
            last_frame: Instant::now(),
            frame_history: VecDeque::new(),
            last_stats: tr_render::sprite::RenderStats::default(),
            bench_frames: None,
            bench_samples: Vec::new(),
            frame_count: 0,
            shot_path: None,
            shot_at: 0,
            atlas_stats: (0, 0, 0, 0),
            light: LightMap::new(),
            scratch: DrawScratch::default(),
            tile_stats: TileDrawStats::default(),
            stats_on: false,
            stats_acc: StatsAcc::default(),
            debug_cache: DebugCache::default(),
            light_mode: LightMode::Color,
            mouse_text_color: 225,
            mouse_text_color_dir: 1,
            demon_torch: 1.0,
            demon_torch_dir: 1.0,
            disco: [255, 0, 0],
            disco_style: 0,
            light_rng_seed: 1,
            anim_time: 0.0,
            has_liquids: false,
            liquids,
            liquid_step: FixedStepper::new(1.0 / 60.0, 8),
            liquid_tick: 0,
            waterfall: tr_core::waterfall::WaterfallManager::new(),
            waterfall_step: FixedStepper::new(1.0 / 60.0, 8),
            rain: Rain::new(1),
            items: Vec::new(),
            item_rng: 0,
            item_step: FixedStepper::new(1.0 / 60.0, 8),
            swing: 0.0,
            coins: 0,
            projectiles: Vec::new(),
            projectile_step: FixedStepper::new(1.0 / 60.0, 8),
            dev_throw: false,
            world_key: None,
            hell_black_box_bottom: 0.0,
        }
    }

    #[test]
    fn back_click_lands_the_legacy_main_page_with_the_front_cleared() {
        // The Back row's `menuMode = 0` hop (`UICharacterSelect.cs:116-120`
        // through the table) must clear the 888 side (`Main.cs:51066-51069`'s
        // else branch): the port used to leave `front` live, so the Main page
        // was invisible under the screen and the invariant broke.
        let mut app =
            flow_app(Some(FrontScreen::CharSelect), vec![entry("flow", false)], vec![]);
        let l = ui::char_select::resolved_layout(
            Vec2::new(1280.0, 720.0),
            app.front_header_w(ui::char_select::SELECT_PLAYER),
            app.players.len(),
        );
        app.mouse_pos = Vec2::new(l.back[0] + l.back[2] * 0.5, l.back[1] + l.back[3] * 0.5);
        app.handle_front_click();
        // Review Focus 1's invariant: exactly one side is live, and it is the
        // legacy page.
        assert!(app.front.is_none(), "the 888 state clears");
        let m = app.menu.as_ref().expect("the legacy page opens");
        assert_eq!(m.page, P::Main);
    }

    #[test]
    fn delete_click_opens_the_mode_5_confirm_with_the_front_cleared() {
        // `DeleteButtonClick` (`UICharacterListItem.cs:242-254`) -> `menuMode
        // = 5`: the confirm opens alone, parking the target name/index
        // (`Main.selectedPlayer`).
        let mut app = flow_app(Some(FrontScreen::CharSelect), vec![entry("Bob", true)], vec![]);
        let l = ui::char_select::resolved_layout(
            Vec2::new(1280.0, 720.0),
            app.front_header_w(ui::char_select::SELECT_PLAYER),
            app.players.len(),
        );
        let delete = ui::char_select::card_layout(l.list, 0).delete;
        app.mouse_pos = Vec2::new(delete[0] + delete[2] * 0.5, delete[1] + delete[3] * 0.5);
        app.handle_front_click();
        assert!(app.front.is_none(), "mode 5 clears the 888 state");
        let m = app.menu.as_ref().expect("the confirm opens");
        assert_eq!(m.page, P::ConfirmDeletePlayer);
        assert_eq!(m.confirm_name.as_deref(), Some("Bob"));
        assert_eq!(m.confirm_index, 0);
    }

    #[test]
    fn world_delete_click_opens_the_mode_9_confirm_with_the_front_cleared() {
        // `DeleteButtonClick` (`UIWorldListItem.cs:346-358`) -> `menuMode = 9`
        // — the world twin of the mode-5 hop.
        let mut app = flow_app(
            Some(FrontScreen::WorldSelect),
            vec![],
            vec![ui::save_list::WorldEntry {
                name: "World 1".into(),
                difficulty: 0,
                size_id: 0,
                is_favorite: false,
                valid: true,
                has_been_played: false,
                path: Some(PathBuf::from("World 1.trwld")),
            }],
        );
        let l = ui::char_select::resolved_layout(
            Vec2::new(1280.0, 720.0),
            app.front_header_w(ui::world_select::SELECT_WORLD),
            app.worlds.len(),
        );
        let delete = ui::world_select::card_layout(l.list, 0, false).delete;
        app.mouse_pos = Vec2::new(delete[0] + delete[2] * 0.5, delete[1] + delete[3] * 0.5);
        app.handle_front_click();
        assert!(app.front.is_none(), "mode 9 clears the 888 state");
        let m = app.menu.as_ref().expect("the confirm opens");
        assert_eq!(m.page, P::ConfirmDeleteWorld);
        assert_eq!(m.confirm_name.as_deref(), Some("World 1"));
        assert_eq!(m.confirm_index, 0);
    }

    // --- Task 8: the world creator's flow hops --------------------------------

    /// One fake world entry for the creator's flow tests.
    fn world_entry(name: &str) -> ui::save_list::WorldEntry {
        ui::save_list::WorldEntry {
            name: name.into(),
            difficulty: 0,
            size_id: 1,
            is_favorite: false,
            valid: true,
            has_been_played: false,
            path: None,
        }
    }

    fn world_creation_app() -> App {
        flow_app(
            Some(FrontScreen::WorldCreation(ui::world_creation::NewWorldDraft::new(
                1, 0,
            ))),
            vec![],
            vec![world_entry("World 1")],
        )
    }

    /// The click handler's layout at the flow tests' deterministic 1280x720.
    fn world_creation_layout(app: &App) -> ui::world_creation::Layout {
        let (sw, sh) = app.screen_size();
        ui::world_creation::layout(Vec2::new(sw, sh), 0.0)
    }

    #[test]
    fn world_new_click_opens_the_creator_with_the_stand_in_name() {
        // `NewWorldClick` (`UIWorldSelect.cs:110-116`): the stand-in name
        // reads `WorldList.Count + 1` (`:113`) — two listed worlds name the
        // draft "World 3".
        let mut app = flow_app(
            Some(FrontScreen::WorldSelect),
            vec![],
            vec![world_entry("Alpha"), world_entry("Beta")],
        );
        let l = ui::char_select::resolved_layout(
            Vec2::new(1280.0, 720.0),
            app.front_header_w(ui::world_select::SELECT_WORLD),
            app.worlds.len(),
        );
        let new = l.new;
        app.mouse_pos = Vec2::new(new[0] + new[2] * 0.5, new[1] + new[3] * 0.5);
        app.handle_front_click();
        let Some(FrontScreen::WorldCreation(draft)) = app.front.as_ref() else {
            panic!("the New click opens the world creator");
        };
        assert_eq!(draft.name, "World 3");
        // `SetDefaultOptions` (`UIWorldCreation.cs:313-326`): the Classic
        // session player seeds Normal/Medium/Random.
        assert_eq!(draft.size, 1);
        assert_eq!(draft.difficulty, 0);
        assert_eq!(draft.evil, 0);
        assert!(app.menu.is_none(), "the invariant holds");
    }

    #[test]
    fn world_creation_back_click_reopens_world_select() {
        // `Click_GoBack` -> `GoBack` -> `OpenWorldSelectUI`
        // (`UIWorldCreation.cs:665-674`) — no discard confirm on this screen.
        let mut app = world_creation_app();
        let l = world_creation_layout(&app);
        app.mouse_pos = Vec2::new(
            l.back[0] + l.back[2] * 0.5,
            l.back[1] + l.back[3] * 0.5,
        );
        app.handle_front_click();
        assert_eq!(app.front, Some(FrontScreen::WorldSelect));
        assert!(app.menu.is_none(), "the invariant holds");
    }

    #[test]
    fn world_creation_option_clicks_update_the_draft() {
        // `ClickSizeOption`/`ClickDifficultyOption`/`ClickEvilOption`/
        // `ClickRandomizeName` (`UIWorldCreation.cs:497-575`): the picks land
        // in the draft; the display order maps Journey/Classic/Expert/Master
        // to the ids 3/0/1/2.
        let mut app = world_creation_app();
        let l = world_creation_layout(&app);
        let centre = |r: [f32; 4]| Vec2::new(r[0] + r[2] * 0.5, r[1] + r[3] * 0.5);
        app.mouse_pos = centre(l.difficulty_buttons[2]); // Expert
        app.handle_front_click();
        app.mouse_pos = centre(l.size_buttons[2]); // Large
        app.handle_front_click();
        app.mouse_pos = centre(l.evil_buttons[1]); // Corruption
        app.handle_front_click();
        let Some(FrontScreen::WorldCreation(draft)) = app.front.as_ref() else {
            panic!("the options clicked on the live creator");
        };
        assert_eq!(draft.difficulty, 1);
        assert_eq!(draft.size, 2);
        assert_eq!(draft.evil, 1);
        // The randomize stand-in re-derives the name from the list count.
        app.mouse_pos = centre(l.random_name_icon);
        app.handle_front_click();
        let Some(FrontScreen::WorldCreation(draft)) = app.front.as_ref() else {
            panic!("the creator stays live");
        };
        assert_eq!(draft.name, "World 2"); // `worlds.len() = 1`
    }

    #[test]
    fn world_creation_name_click_opens_the_naming_keyboard() {
        // `Click_SetName` (`UIWorldCreation.cs:689-695`): `Lang.menu[48]`,
        // empty start, `allowEmpty: true`, the 27 cap; the tail lands the
        // trimmed name back in the draft (`OnFinishedSettingName`, `:726-731`).
        let mut app = world_creation_app();
        let l = world_creation_layout(&app);
        app.mouse_pos = Vec2::new(
            l.name_button[0] + l.name_button[2] * 0.5,
            l.name_button[1] + l.name_button[3] * 0.5,
        );
        app.handle_front_click();
        let Some(FrontScreen::Keyboard(session)) = app.front.as_ref() else {
            panic!("the name plate opens the keyboard");
        };
        assert_eq!(session.request.label, ui::keyboard::ENTER_WORLD_NAME);
        assert!(session.request.initial.is_empty(), "the box opens empty");
        assert!(session.request.allow_empty);
        assert_eq!(session.request.max_length, 27);
        assert_eq!(session.tail, KeyboardTail::WorldName);
        // The submit tail: the name lands, the creator returns (`:730`).
        app.keyboard_character('P');
        app.keyboard_submit();
        let Some(FrontScreen::WorldCreation(draft)) = app.front.as_ref() else {
            panic!("the creator returns");
        };
        assert_eq!(draft.name, "P");
        assert!(app.menu.is_none(), "the invariant holds");
    }

    #[test]
    fn world_creation_seed_click_opens_the_seed_keyboard() {
        // `OpenSeedInputMenu` (`UIWorldCreation.cs:697-708`): "UI.EnterSeed",
        // `allowEmpty: true`, `int.MaxValue`; the seed plate and the icon
        // both route here (the Advanced tab is out of scope).
        let mut app = world_creation_app();
        let l = world_creation_layout(&app);
        app.mouse_pos = Vec2::new(
            l.seed_plate[0] + l.seed_plate[2] * 0.5,
            l.seed_plate[1] + l.seed_plate[3] * 0.5,
        );
        app.handle_front_click();
        let Some(FrontScreen::Keyboard(session)) = app.front.as_ref() else {
            panic!("the seed plate opens the keyboard");
        };
        assert_eq!(session.request.label, "Enter Seed (Leave Blank For Random)");
        assert!(session.request.allow_empty);
        assert_eq!(session.request.max_length, usize::MAX);
        assert_eq!(session.tail, KeyboardTail::WorldSeed);
        // The submit tail: the trimmed seed lands (`OnFinishedSettingSeed`,
        // `:817-819`), the creator returns.
        app.keyboard_character('5');
        app.keyboard_character('1');
        app.keyboard_submit();
        let Some(FrontScreen::WorldCreation(draft)) = app.front.as_ref() else {
            panic!("the creator returns");
        };
        assert_eq!(draft.seed, "51");
    }

    #[test]
    fn world_creation_create_with_an_empty_name_opens_the_naming_keyboard() {
        // `Click_NamingAndCreating` (`UIWorldCreation.cs:710-719`): the empty
        // branch opens `Lang.menu[48]` with `allowEmpty: false`; the filled
        // branch's save is covered by the module-level temp-dir test (the
        // App click would write the real world directory).
        let mut app = world_creation_app();
        if let Some(FrontScreen::WorldCreation(draft)) = app.front.as_mut() {
            // The stand-in name is non-empty (`SetDefaultOptions`'s
            // `AssignRandomWorldName`); the empty branch needs the C#'s
            // `string.IsNullOrEmpty(this._optionwWorldName)` (`:713`).
            draft.name = String::new();
        }
        let l = world_creation_layout(&app);
        app.mouse_pos = Vec2::new(
            l.create[0] + l.create[2] * 0.5,
            l.create[1] + l.create[3] * 0.5,
        );
        app.handle_front_click();
        let Some(FrontScreen::Keyboard(session)) = app.front.as_ref() else {
            panic!("the empty-name Create opens the naming keyboard");
        };
        assert!(!session.request.allow_empty);
        assert_eq!(session.tail, KeyboardTail::WorldNameAndCreate);
        assert!(app.menu.is_none(), "the invariant holds");
    }

    // --- Task 6: the keyboard's flow hops ------------------------------------

    /// An App with the keyboard front live over the char select — the session
    /// `RenameButtonClick` builds (`UICharacterListItem.cs:196-206`).
    fn keyboard_app() -> App {
        let request = ui::keyboard::Request {
            label: ui::keyboard::ENTER_CHARACTER_NAME.into(),
            initial: "Bob".into(),
            allow_empty: true,
            max_length: 20,
        };
        let state = ui::keyboard::KeyboardState::new(&request);
        flow_app(
            Some(FrontScreen::Keyboard(Box::new(KeyboardSession {
                request,
                state,
                back: FrontScreen::CharSelect,
                tail: KeyboardTail::Rename,
            }))),
            vec![entry("Bob", false)],
            vec![],
        )
    }

    #[test]
    fn rename_click_opens_the_keyboard_session() {
        // `RenameButtonClick` (`:196-206`): `Lang.menu[45]`, the card's name,
        // `allowEmpty: true`, the ctor's 20-character cap; the cancel target
        // is the screen the keyboard came from.
        let mut app = flow_app(Some(FrontScreen::CharSelect), vec![entry("Bob", false)], vec![]);
        let l = ui::char_select::resolved_layout(
            Vec2::new(1280.0, 720.0),
            app.front_header_w(ui::char_select::SELECT_PLAYER),
            app.players.len(),
        );
        let rename = ui::char_select::card_layout(l.list, 0).rename;
        app.mouse_pos = Vec2::new(rename[0] + rename[2] * 0.5, rename[1] + rename[3] * 0.5);
        app.handle_front_click();
        let Some(FrontScreen::Keyboard(session)) = app.front.as_ref() else {
            panic!("the rename click opens the keyboard");
        };
        assert_eq!(session.request.label, ui::keyboard::ENTER_CHARACTER_NAME);
        assert_eq!(session.request.initial, "Bob");
        assert!(session.request.allow_empty);
        assert_eq!(session.request.max_length, 20);
        assert_eq!(session.back, FrontScreen::CharSelect);
        // The state seeds from the request (`UIVirtualKeyboard.cs:429-433`) —
        // a non-empty start stays Default.
        assert_eq!(session.state.text, "Bob");
        assert_eq!(session.state.key_state, ui::keyboard::KeyState::Default);
        // Review Focus 1: the menu side stayed closed while the keyboard is up.
        assert!(app.menu.is_none());
    }

    #[test]
    fn keyboard_cancel_click_reopens_the_parked_screen() {
        // `_cancelAction` -> `GoBackHere` (`:214-216`): the reopen, no notice.
        let mut app = keyboard_app();
        let (sw, sh) = app.screen_size();
        let l = ui::keyboard::layout(Vec2::new(sw, sh), false);
        app.mouse_pos = Vec2::new(l.cancel[0] + l.cancel[2] * 0.5, l.cancel[1] + l.cancel[3] * 0.5);
        app.handle_front_click();
        assert_eq!(app.front, Some(FrontScreen::CharSelect));
        assert!(app.front_hint.is_none(), "cancel notices nothing");
        assert!(app.menu.is_none(), "the invariant holds");
    }

    #[test]
    fn keyboard_submit_reopens_with_the_not_ported_notice() {
        // The rename tail (`:205-213`) reopens the screen; the port's rename
        // itself is the notice (no save-file writer, spec §9.4).
        let mut app = keyboard_app();
        app.keyboard_submit();
        assert_eq!(app.front, Some(FrontScreen::CharSelect));
        assert!(app.front_hint.is_some(), "the submit tail notices");
    }

    #[test]
    fn keyboard_enter_gates_on_the_validation() {
        // `:763-766`: `Main.inputTextEnter && CanSubmit` — an empty text with
        // `allowEmpty: false` keeps the keyboard up; typing fixes it.
        let request = ui::keyboard::Request {
            label: ui::keyboard::ENTER_CHARACTER_NAME.into(),
            initial: String::new(),
            allow_empty: false,
            max_length: 20,
        };
        let state = ui::keyboard::KeyboardState::new(&request);
        let mut app = flow_app(
            Some(FrontScreen::Keyboard(Box::new(KeyboardSession {
                request,
                state,
                back: FrontScreen::CharSelect,
                tail: KeyboardTail::Rename,
            }))),
            vec![],
            vec![],
        );
        // The empty start seeds Shift (`:430-433`).
        let Some(FrontScreen::Keyboard(session)) = app.front.as_ref() else {
            panic!("the keyboard is live");
        };
        assert_eq!(session.state.key_state, ui::keyboard::KeyState::Shift);
        app.keyboard_enter();
        assert!(
            matches!(app.front, Some(FrontScreen::Keyboard(_))),
            "the invalid Enter is dead"
        );
        // The physical path never unlatches the seeded Shift (`:778-788`).
        app.keyboard_character('N');
        let Some(FrontScreen::Keyboard(session)) = app.front.as_ref() else {
            panic!("the keyboard stays live");
        };
        assert_eq!(session.state.text, "N");
        assert_eq!(session.state.key_state, ui::keyboard::KeyState::Shift);
        app.keyboard_enter();
        assert_eq!(app.front, Some(FrontScreen::CharSelect), "the valid Enter submits");
    }

    #[test]
    fn tr_ui_vk_presets_the_keyboard_now() {
        // Task 5 parsed `vk` without presetting; Task 6's variant is live.
        let mut app = flow_app(None, vec![], vec![]);
        app.preset_ui(TrUiPreset::Keyboard);
        let Some(FrontScreen::Keyboard(session)) = app.front.as_ref() else {
            panic!("TR_UI=vk presets the keyboard");
        };
        assert_eq!(session.request.label, ui::keyboard::ENTER_CHARACTER_NAME);
        assert!(session.state.text.is_empty(), "the fresh name is empty (`Player.cs:826`)");
        assert_eq!(
            session.state.key_state,
            ui::keyboard::KeyState::Shift,
            "the empty text seeds Shift (`UIVirtualKeyboard.cs:430-433`)"
        );
        assert_eq!(session.back, FrontScreen::CharSelect);
    }

    // --- Task 7: the character creator's flow hops ---------------------------

    /// One creation screen for the flow tests, optionally pre-dirtied.
    fn creation_app(dirty: bool) -> App {
        let mut state = ui::char_creation::CreationState::new();
        if dirty {
            state.draft.name = "Zealot".into();
            state.draft.refresh_dirty();
        }
        flow_app(Some(FrontScreen::CharCreation(state)), vec![], vec![])
    }

    /// Aims the pointer at a char-creation rect's centre and clicks.
    fn click_creation(app: &mut App, rect: [f32; 4]) {
        app.mouse_pos = Vec2::new(rect[0] + rect[2] * 0.5, rect[1] + rect[3] * 0.5);
        app.handle_front_click();
    }

    #[test]
    fn new_click_opens_a_fresh_creation_screen() {
        // `NewCharacterClick` (`UICharacterSelect.cs:108-114`): a fresh
        // `PendingPlayer`, the creation screen live alone.
        let mut app = flow_app(Some(FrontScreen::CharSelect), vec![], vec![]);
        let (sw, sh) = app.screen_size();
        let l = ui::char_select::resolved_layout(
            Vec2::new(sw, sh),
            app.front_header_w(ui::char_select::SELECT_PLAYER),
            app.players.len(),
        );
        click_creation(&mut app, l.new);
        let Some(FrontScreen::CharCreation(state)) = app.front.as_ref() else {
            panic!("the New click opens the creator");
        };
        assert_eq!(state.draft, ui::char_creation::NewPlayerDraft::new());
        assert_eq!(state.category, 0, "the CharInfo menu seeds (`:233`)");
        assert!(app.menu.is_none(), "the invariant holds");
        assert!(app.front_hint.is_none());
    }

    #[test]
    fn tr_ui_charnew_presets_the_creation_screen() {
        // The Task 5 hook table's `charnew` now presets the real screen.
        let mut app = flow_app(None, vec![], vec![]);
        app.preset_ui(TrUiPreset::CharNew);
        assert!(matches!(app.front, Some(FrontScreen::CharCreation(_))));
        assert!(app.menu.is_none());
    }

    #[test]
    fn creation_clicks_mutate_the_draft() {
        // Category 4 (Eye) switches the picker; a difficulty row sets the
        // byte (`UIDifficultyButton.cs:67-71`); a cloth-style button maps
        // through `_validClothStyles` and auto-assigns the hair (`:630-637`).
        let mut app = creation_app(false);
        let (sw, sh) = app.screen_size();
        let l = ui::char_creation::layout(Vec2::new(sw, sh), 0.0);
        // The Info menu is live (category 0): a difficulty row sets the byte.
        click_creation(&mut app, l.difficulty_buttons[3]);
        let Some(FrontScreen::CharCreation(state)) = app.front.as_ref() else {
            panic!("the creator stays live");
        };
        assert_eq!(state.draft.difficulty, 2, "the Hardcore row");
        // The Clothing menu: a cloth-style button maps through
        // `_validClothStyles` and auto-assigns the hair (`:630-637`).
        click_creation(&mut app, l.cloth_buttons[0]);
        let Some(FrontScreen::CharCreation(state)) = app.front.as_ref() else {
            panic!("the creator stays live");
        };
        assert_eq!(state.draft.skin_variant, 0);
        assert_eq!(
            state.draft.hair,
            ui::char_creation::default_hairstyle_for_cloth_style(0).unwrap_or_default(),
            "the untouched hair auto-assigns"
        );
        // Category 4 (Eye) switches the picker (`Click_ColorPicker`).
        click_creation(&mut app, l.categories[4]);
        let Some(FrontScreen::CharCreation(state)) = app.front.as_ref() else {
            panic!("the creator stays live");
        };
        assert_eq!(state.category, 4);
    }

    #[test]
    fn clean_creation_back_reopens_the_char_select() {
        // `GoBack`'s not-dirty branch (`UICharacterCreation.cs:1218-1221`):
        // `OpenCharacterSelectUI`, no backup, no confirm.
        let mut app = creation_app(false);
        let (sw, sh) = app.screen_size();
        let l = ui::char_creation::layout(Vec2::new(sw, sh), 0.0);
        click_creation(&mut app, l.back);
        assert_eq!(app.front, Some(FrontScreen::CharSelect));
        assert!(app.front_backup.is_none());
        assert!(app.menu.is_none(), "the invariant holds");
    }

    #[test]
    fn dirty_creation_back_parks_the_mode_40_round_trip() {
        // `GoBack`'s dirty branch (`:1213-1217`): the exact screen parks in
        // `front_backup` (`BackupConfirmationState`) and menuMode 40 opens —
        // the header carries the draft's name (`:49441-49445`).
        let mut app = creation_app(true);
        let (sw, sh) = app.screen_size();
        let l = ui::char_creation::layout(Vec2::new(sw, sh), 0.0);
        click_creation(&mut app, l.back);
        assert!(app.front.is_none(), "mode 40 clears the 888 state");
        let Some(FrontScreen::CharCreation(parked)) = app.front_backup.as_ref() else {
            panic!("the creation screen parks");
        };
        assert_eq!(parked.draft.name, "Zealot");
        assert_eq!(parked.draft.dirty, true);
        let m = app.menu.as_ref().expect("mode 40 opens");
        assert_eq!(m.page, menu::Page::ConfirmDiscard);
        assert_eq!(m.confirm_name.as_deref(), Some("Zealot"));
        // The No branch's restore is the flow table's (`:49460-49467`), and
        // Yes drops the parked state for the char select (`:49454-49459`).
        let (front, backup, page) = next_flow_state(
            app.front.clone(),
            app.front_backup.clone(),
            Some(menu::Page::ConfirmDiscard),
            FlowClick::ConfirmNo,
        );
        assert_eq!(front, Some(FrontScreen::CharCreation(parked.clone())));
        assert!(backup.is_none() && page.is_none());
        let (front, backup, page) = next_flow_state(
            app.front.clone(),
            app.front_backup.clone(),
            Some(menu::Page::ConfirmDiscard),
            FlowClick::ConfirmYes,
        );
        assert_eq!(front, Some(FrontScreen::CharSelect));
        assert!(backup.is_none() && page.is_none());
    }

    #[test]
    fn naming_keyboard_round_trips_into_the_draft() {
        // `Click_Naming` (`:1659-1666`) -> `OnFinishedNaming` (`:1684-1689`):
        // the keyboard opens with `allowEmpty: true`, the trimmed name lands
        // in the parked draft and the creation screen returns.
        let mut app = creation_app(false);
        let (sw, sh) = app.screen_size();
        let l = ui::char_creation::layout(Vec2::new(sw, sh), 0.0);
        click_creation(&mut app, l.info_name);
        let Some(FrontScreen::Keyboard(session)) = app.front.as_ref() else {
            panic!("the name plate opens the keyboard");
        };
        assert!(session.request.allow_empty);
        assert_eq!(session.request.max_length, ui::char_creation::MAX_NAME_LENGTH);
        assert!(matches!(session.back, FrontScreen::CharCreation(_)));
        assert_eq!(session.tail, KeyboardTail::Name);
        // The empty name also cleared the draft's own (`:1662`).
        if let FrontScreen::CharCreation(back) = &session.back {
            assert!(back.draft.name.is_empty());
        }
        if let Some(FrontScreen::Keyboard(session)) = app.front.as_mut() {
            ui::keyboard::write_str(&mut session.state, "  Zealot ");
        }
        app.keyboard_submit();
        let Some(FrontScreen::CharCreation(state)) = app.front.as_ref() else {
            panic!("the creation screen returns");
        };
        assert_eq!(state.draft.name, "Zealot", "`name.Trim()` (`:1686`)");
        assert!(app.menu.is_none(), "the invariant holds");
    }

    #[test]
    fn create_with_an_empty_name_opens_the_naming_keyboard() {
        // `Click_NamingAndCreating`'s empty branch (`:1668-1677`): the
        // keyboard opens WITHOUT `allowEmpty` (the ctor default `false`), and
        // the submit tail finishes the creation (`OnFinishedNamingAnd-
        // Creating`, `:1696-1702`).
        let mut app = creation_app(false);
        let (sw, sh) = app.screen_size();
        let l = ui::char_creation::layout(Vec2::new(sw, sh), 0.0);
        click_creation(&mut app, l.create);
        let Some(FrontScreen::Keyboard(session)) = app.front.as_ref() else {
            panic!("Create with no name opens the keyboard");
        };
        assert!(!session.request.allow_empty);
        assert_eq!(session.tail, KeyboardTail::NameAndCreate);
        if let Some(FrontScreen::Keyboard(session)) = app.front.as_mut() {
            ui::keyboard::write_str(&mut session.state, "Zealot");
        }
        app.keyboard_submit();
        assert_eq!(app.front, Some(FrontScreen::CharSelect), "`menuMode = 1` (`:1710`)");
        assert!(app.menu.is_none());
    }

    #[test]
    fn create_with_a_name_records_the_draft_as_the_session_player() {
        // `FinishCreatingCharacter` (`:1704-1711`): the draft records (no
        // `.plr` writer — documented), the session entry carries the name and
        // difficulty, and the char select re-lists.
        let mut app = creation_app(false);
        if let Some(FrontScreen::CharCreation(state)) = app.front.as_mut() {
            state.draft.name = "Zealot".into();
            state.draft.difficulty = 2;
            state.category = 0;
        }
        let (sw, sh) = app.screen_size();
        let l = ui::char_creation::layout(Vec2::new(sw, sh), 0.0);
        click_creation(&mut app, l.create);
        assert_eq!(app.front, Some(FrontScreen::CharSelect));
        assert!(app.front_backup.is_none(), "the backup drops");
        let Some(draft) = app.session_draft.as_ref() else {
            panic!("the draft records");
        };
        assert_eq!(draft.name, "Zealot");
        assert_eq!(draft.difficulty, 2);
        // `Main.LoadPlayers` re-lists with the created entry first-shaped.
        let entry = app.session_entry();
        assert_eq!(entry.name, "Zealot");
        assert_eq!(entry.difficulty, 2);
        assert!(app.players.iter().any(|p| p.name == "Zealot"));
    }
}
