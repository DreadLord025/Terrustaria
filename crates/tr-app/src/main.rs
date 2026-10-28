//! Terraria-Rust: Vulkan renderer prototype for a native reimplementation.

mod assets;

use std::collections::{HashSet, VecDeque};
use std::path::PathBuf;
use std::sync::{Arc, OnceLock};
use std::time::Instant;

use anyhow::{Context, Result};
use glam::Vec2;
use tr_core::lighting::{LightMap, LightMode, LightParams, LightStyle};
use tr_core::tile::LiquidKind;
use tr_core::tiledata as td;
use tr_core::{TILE_PX, World, framing, save, tree, wld, worldgen};
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

fn main() -> Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let args: Vec<String> = std::env::args().collect();
    // No argument loads our own generated world (`Documents/My Games/Terrustaria`).
    // Passing a path still loads a `.wld` from the original game or a saved `.trwld`.
    let world_path = match args.get(1) {
        Some(p) => PathBuf::from(p),
        None => terrstaria_world()?,
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

    let t_frame = Instant::now();
    let framed = framing::frame_world(&mut world);
    let tile_frame_ms = t_frame.elapsed().as_secs_f32() * 1000.0;
    let t_walls = Instant::now();
    let walled = framing::frame_walls(&mut world);
    let wall_frame_ms = t_walls.elapsed().as_secs_f32() * 1000.0;
    world.compute_column_surface();
    world.compute_chunks();
    let slopes = world
        .tiles
        .iter()
        .filter(|t| t.active() && (t.slope != 0 || t.half_brick()))
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

    let assets_dir =
        find_assets_dir().context("assets/Images not found (run tools/XnbExtract first)")?;

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
                    "{}:{}:{}/{}/w{} ",
                    if t.active() { t.kind } else { 999 },
                    t.slope,
                    t.frame_x,
                    t.frame_y,
                    t.wall
                ));
            }
            log::info!("row {y}: {line}");
        }
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

    let (tile_ids, wall_ids) = assets::usage(&world);
    log::info!(
        "world uses {} tile types and {} wall types",
        tile_ids.len(),
        wall_ids.len()
    );

    let t1 = Instant::now();
    let mut images = assets::load_world_images(&assets_dir, &tile_ids, &wall_ids)?;
    let font = tr_render::atlas::font_sheet();
    images.push(font.0);
    log::info!(
        "decoded {} sheets in {:.0} ms",
        images.len(),
        t1.elapsed().as_secs_f32() * 1000.0
    );

    let t2 = Instant::now();
    let atlas = Atlas::build(images, 4096);
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

    let lookups = assets::Lookups::build(
        &atlas,
        tile_ids
            .iter()
            .map(|&id| (id, assets::image_name_for_tile(id))),
        wall_ids
            .iter()
            .map(|&id| (id, assets::image_name_for_wall(id))),
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
        for name in ["Tiles_0", "Tiles_1", "Tiles_2", "Wall_1", "Background_0"] {
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
    let has_liquids = world.tiles.iter().any(|t| t.liquid > 0);

    let mut app = App {
        window: None,
        gpu: None,
        renderer: None,
        world,
        atlas: Some(atlas),
        lookups,
        clouds,
        sky,
        stars,
        camera: Vec2::ZERO,
        zoom: 1.5,
        debug: std::env::var("TR_DEBUG").map(|v| v != "0").unwrap_or(true),
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
        anim_time: 0.0,
        has_liquids,
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

fn find_assets_dir() -> Option<PathBuf> {
    let candidates = [
        PathBuf::from("assets/Images"),
        PathBuf::from("../../assets/Images"),
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/Images"),
    ];
    candidates.into_iter().find(|p| p.is_dir())
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
    stars: Vec<StarSprite>,
    camera: Vec2,
    zoom: f32,
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
    anim_time: f32,
    has_liquids: bool,
    /// G6: identity of the last world instance range; a hit lets the frame reuse it.
    world_key: Option<WorldFrameKey>,
}

/// G6: everything the world instance range (sky strips, tiles, walls, dark rects)
/// depends on, compared bit-exactly. A hit keeps the previous frame's instances and
/// only redraws the UI; the light map has its own cache (D2) inside `LightMap`.
#[derive(Clone, Copy, PartialEq)]
struct WorldFrameKey {
    cam: [u32; 2],
    zoom: u32,
    screen: [u32; 2],
    sky_tint: [u32; 3],
    sky_light: [u32; 3],
    hell_glow: u32,
    torch_flicker: u32,
    light_mode: u32,
    /// Non-zero only for modes whose light depends on time ("Trippy").
    anim: u32,
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
    }

    fn log_and_reset(&mut self) {
        let n = self.frames.max(1) as f64;
        log::info!(
            "TR_STATS frames={} light={:.2}ms draw={:.2}ms visible={} walls={} culled={} dark_cells={} rects={} chunks_skipped={} instances={}",
            self.frames,
            self.light_ms / n,
            self.draw_ms / n,
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

        let attrs = Window::default_attributes()
            .with_title("Terraria-Rust  |  Vulkan")
            .with_inner_size(PhysicalSize::new(1600, 900));
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
        if let Ok(z) = std::env::var("TR_ZOOM") {
            if let Ok(z) = z.parse::<f32>() {
                self.zoom = z.clamp(0.25, 8.0);
            }
        }
        self.last_frame = Instant::now();

        self.gpu = Some(gpu);
        self.renderer = Some(renderer);
        self.window = Some(window);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
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
                        match code {
                            KeyCode::Escape => event_loop.exit(),
                            KeyCode::F1 => self.debug = !self.debug,
                            KeyCode::F2 => {
                                self.light_mode = self.light_mode.next();
                                self.world_key = None;
                                log::info!("light mode: {}", self.light_mode.label());
                            }
                            KeyCode::F5 => {
                                self.camera = Vec2::new(
                                    self.world.width as f32 * TILE_PX * 0.5,
                                    self.world.spawn_y as f32 * TILE_PX,
                                );
                                self.zoom = 1.5;
                            }
                            KeyCode::Equal | KeyCode::NumpadAdd => {
                                self.zoom = (self.zoom * 1.25).min(8.0)
                            }
                            KeyCode::Minus | KeyCode::NumpadSubtract => {
                                self.zoom = (self.zoom / 1.25).max(0.25)
                            }
                            _ => {}
                        }
                    } else {
                        self.keys.remove(&code);
                    }
                }
            }
            WindowEvent::MouseWheel { delta, .. } => {
                let d = match delta {
                    MouseScrollDelta::LineDelta(_, y) => y,
                    MouseScrollDelta::PixelDelta(p) => p.y as f32 / 60.0,
                };
                self.zoom = (self.zoom * 1.1f32.powf(d)).clamp(0.25, 8.0);
            }
            WindowEvent::MouseInput { state, button, .. } => {
                if button == MouseButton::Middle || button == MouseButton::Right {
                    self.mouse_down = state == ElementState::Pressed;
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                let p = Vec2::new(position.x as f32, position.y as f32);
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
    fn frame(&mut self) {
        let now = Instant::now();
        let dt = now
            .duration_since(self.last_frame)
            .as_secs_f32()
            .clamp(0.0, 0.1);
        self.last_frame = now;
        self.frame_count += 1;
        self.anim_time += dt;
        let anim_time = self.anim_time;

        self.pan_camera(dt);

        let screen = self
            .gpu
            .as_ref()
            .map(|g| Vec2::new(g.config.width as f32, g.config.height as f32))
            .unwrap_or(Vec2::new(1600.0, 900.0));

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
                stars,
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
                world_key,
                ..
            } = self;

            if let (Some(gpu), Some(renderer)) = (gpu.as_ref(), renderer.as_mut()) {
                // `TR_DAY` overrides the time of day (0 = midnight, 1 = noon) so a
                // screenshot can be compared against the original at a fixed time.
                let override_day = std::env::var("TR_DAY")
                    .ok()
                    .and_then(|v| v.parse::<f32>().ok())
                    .map(|v| v.clamp(0.0, 1.0));
                let (sky_tint, sky_light, is_day, clock_time) = match override_day {
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
                        let c = color_of_the_skies(world.time, world.day_time);
                        (c, tile_color(c), world.day_time, world.time)
                    }
                };
                let params = LightParams {
                    sky: sky_light,
                    hell_glow: 0.55,
                    torch_flicker: 1.0,
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
                    cam: [camera.x.to_bits(), camera.y.to_bits()],
                    zoom: zoom.to_bits(),
                    screen: [screen.x.to_bits(), screen.y.to_bits()],
                    sky_tint: [
                        sky_tint[0].to_bits(),
                        sky_tint[1].to_bits(),
                        sky_tint[2].to_bits(),
                    ],
                    sky_light: [
                        params.sky[0].to_bits(),
                        params.sky[1].to_bits(),
                        params.sky[2].to_bits(),
                    ],
                    hell_glow: params.hell_glow.to_bits(),
                    torch_flicker: params.torch_flicker.to_bits(),
                    light_mode: *light_mode as u32,
                    anim: if style.perturb > 0.0 {
                        anim_time.to_bits()
                    } else {
                        liquid_anim
                    },
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
                    draw_sky(
                        renderer, *camera, *zoom, screen, world, clouds, sky, stars, surface_px,
                        is_day, clock_time, sky_uv, sky_tint,
                    );

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
                    );
                    *world_key = Some(key);
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

/// Background anchor for one column: the lowest ground within a small window. Using
/// the column's own surface would paint the cave fade over the open corner of a slope
/// or an overhang; taking the deepest neighbour keeps sky there.
fn bg_surface(world: &World, x: i32) -> i32 {
    let mut s = world.surface_of(x);
    for d in 1..=2 {
        let left = (x - d).clamp(0, world.width - 1);
        let right = (x + d).clamp(0, world.width - 1);
        s = s.max(world.surface_of(left)).max(world.surface_of(right));
    }
    s
}

#[derive(Clone, Copy)]
struct BgTex {
    uv: [f32; 4],
    w: f32,
    h: f32,
}

struct SkyAssets {
    far: Option<BgTex>,
    near: Option<BgTex>,
    sun: Option<BgTex>,
    moon: Option<BgTex>,
    stars: Vec<Option<BgTex>>,
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
    let mountains = sky_tex(atlas, "Background_7", None);
    SkyAssets {
        far: mountains,
        near: mountains,
        sun: sky_tex(atlas, "Sun", None),
        moon: sky_tex(atlas, "Moon_0", Some((0, 0, 50, 50))),
        stars: (0..4)
            .map(|i| sky_tex(atlas, &format!("Star_{i}"), None))
            .collect(),
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
    sky_uv: Option<[f32; 4]>,
    tint: [f32; 3],
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
            r.push_sprite(
                [origin.x, y0],
                [width, y1 - y0],
                sub,
                [tint[0], tint[1], tint[2], 1.0],
            );
        }
        // Above the gradient's top edge the sky is a flat zenith blue.
        if y0 > origin.y {
            let zenith = [53.0 / 255.0, 43.0 / 255.0, 243.0 / 255.0];
            r.push_sprite(
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
            r.push_sprite(
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
        r.push_sprite(
            [origin.x, origin.y],
            [width, screen.y / zoom],
            white,
            [tint[0], tint[1], tint[2], 1.0],
        );
    }

    // Celestial bodies, stars and the parallax surface background. These are drawn
    // in screen space (compensated for zoom) like `DrawSunAndMoon`/`DrawSurfaceBG`.
    let cam_tile = (camera.x / TILE_PX) as i32;
    let local_surface = world.surface_of(cam_tile) as f32 * TILE_PX;
    let horizon = (local_surface - camera.y) * zoom + screen.y * 0.5;

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

    if horizon > 0.0 {
        if let Some(far) = sky.far {
            draw_bg_layer(
                r, camera, zoom, screen, &far, 1.25, 0.15, horizon, 80.0, tint,
            );
        }
        if let Some(near) = sky.near {
            draw_bg_layer(
                r,
                camera,
                zoom,
                screen,
                &near,
                1.31,
                0.43,
                horizon,
                20.0,
                [tint[0] * 0.78, tint[1] * 0.82, tint[2] * 0.86],
            );
        }
    }

    // Parallax clouds. Vertical motion follows the sky horizon, horizontal motion
    // is half the camera's, like the original's cloud layers.
    let right = camera.x + half.x;
    let cloud_tint = [tint[0] * 0.97, tint[1] * 0.98, tint[2], 1.0];
    for c in clouds {
        let w = c.w * c.scale;
        let h = c.h * c.scale;
        let wx = c.x + camera.x * 0.5;
        // Clouds live in a screen-relative band above the camera; anchoring them to
        // `worldSurface` would hide them whenever the player stands on a hill above
        // the average surface.
        let wy = camera.y - (0.08 + c.alt * 0.9) * half.y - h * 0.5;
        if wx + w < origin.x || wx > right || wy + h < origin.y || wy > bottom {
            continue;
        }
        r.push_sprite([wx, wy], [w, h], c.uv, cloud_tint);
    }

    // Underground: one smooth column gradient starting at that column's own first
    // solid tile. Anchoring per column keeps sky visible in valleys and overhangs,
    // while the long vertical fade avoids the flat "brown sea" and hard rectangle
    // edges of the old screen-wide band.
    let bright =
        ((tile_color(tint)[0] + tile_color(tint)[1] + tile_color(tint)[2]) / 3.0).clamp(0.0, 1.0);
    let cave_top = [0.30 * bright, 0.22 * bright, 0.16 * bright];
    let cave_deep = [0.020f32, 0.017, 0.021];
    const CAVE_STRIPS: i32 = 8;
    const CAVE_SPAN: f32 = 34.0 * TILE_PX;
    static BG: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    if !*BG.get_or_init(|| std::env::var_os("TR_NO_BG").is_some()) {
        let tx0 = (origin.x / TILE_PX).floor() as i32 - 1;
        let tx1 = ((camera.x + half.x) / TILE_PX).ceil() as i32 + 1;
        for tx in tx0..=tx1 {
            // Start the underground fill at the lowest ground in a small window,
            // not at this column's own surface: the empty corner of a slope must
            // show sky, exactly like an exposed hillside in the original.
            let sx = bg_surface(world, tx) as f32 * TILE_PX;
            let top = sx.max(origin.y);
            if top >= bottom {
                continue;
            }
            let span_end = (sx + CAVE_SPAN).min(bottom);
            let strip_h = (span_end - top) / CAVE_STRIPS as f32;
            if strip_h > 0.0 {
                for i in 0..CAVE_STRIPS {
                    let y = top + i as f32 * strip_h;
                    let depth = ((y - sx) / CAVE_SPAN).clamp(0.0, 1.0);
                    let c = lerp3(cave_top, cave_deep, depth);
                    r.push_sprite(
                        [tx as f32 * TILE_PX, y],
                        [TILE_PX + 0.5, strip_h + 0.5],
                        white,
                        [c[0], c[1], c[2], 1.0],
                    );
                }
            }
            if span_end < bottom {
                r.push_sprite(
                    [tx as f32 * TILE_PX, span_end],
                    [TILE_PX + 0.5, bottom - span_end],
                    white,
                    [cave_deep[0], cave_deep[1], cave_deep[2], 1.0],
                );
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
    r.push_sprite([wx, wy], [size[0] / zoom, size[1] / zoom], uv, color);
}

/// One tiled parallax mountain/forest layer, anchored to the local terrain horizon.
#[allow(clippy::too_many_arguments)]
fn draw_bg_layer(
    r: &mut SpriteRenderer,
    camera: Vec2,
    zoom: f32,
    screen: Vec2,
    tex: &BgTex,
    scale: f32,
    parallax: f32,
    horizon: f32,
    bottom_offset: f32,
    tint: [f32; 3],
) {
    let sw = tex.w * scale;
    let sh = tex.h * scale;
    let bottom = horizon + bottom_offset;
    let top = bottom - sh;
    let ox = -((camera.x * zoom * parallax).rem_euclid(sw));
    let mut x = ox - sw;
    while x < screen.x {
        push_screen(
            r,
            camera,
            zoom,
            screen,
            [x, top],
            [sw, sh],
            tex.uv,
            [tint[0], tint[1], tint[2], 1.0],
        );
        x += sw;
    }
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
) -> TileDrawStats {
    let t_total = Instant::now();
    let half = screen * 0.5 / zoom;
    let min = camera - half;
    let max = camera + half;
    let x0 = ((min.x / TILE_PX).floor() as i32 - 1).clamp(0, world.width - 1);
    let x1 = ((max.x / TILE_PX).ceil() as i32 + 1).clamp(0, world.width - 1);
    let y0 = ((min.y / TILE_PX).floor() as i32 - 1).clamp(0, world.height - 1);
    let y1 = ((max.y / TILE_PX).ceil() as i32 + 1).clamp(0, world.height - 1);

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
                            lookups.frame_tex(a, tile.kind, tile.frame_x, tile.frame_y)
                        });
                    }
                    if tile_frame.is_none() {
                        tile_frame = lookups.plain_tex(tile.kind);
                    }
                }
                let solid = (tile.kind as usize) < td::TILE_COUNT && td::TILE_SOLID.has(tile.kind);
                let tile_covers = !is_tree
                    && tile.slope == 0
                    && !tile.half_brick()
                    && tile_frame.is_some_and(|f| f.opaque);
                let wall_frame = if tile.wall != 0 && draw_walls {
                    lookups.wall_tex(tile.wall, tile.wall_col, tile.wall_row)
                } else {
                    None
                };

                // A3/A4: fully covered, fully black cell -> greedy merged rectangle.
                if light_is_zero(l) {
                    let covered = (active && tile_covers)
                        || (!active && wall_frame.is_some_and(|f| f.opaque));
                    if covered {
                        if run_start < 0 {
                            run_start = tx;
                        }
                        stats.dark_cells += 1;
                        continue;
                    }
                }
                if run_start >= 0 {
                    scratch.cur.push((run_start, tx - 1));
                    run_start = -1;
                }

                let pos = [tx as f32 * TILE_PX, ty as f32 * TILE_PX];
                if let Some(wf) = wall_frame {
                    // A2: a fully opaque, non-sloped block hides the wall completely.
                    if active && tile_covers {
                        stats.walls_culled += 1;
                    } else {
                        push_lit(
                            r,
                            pos,
                            [TILE_PX, TILE_PX],
                            wf.uv,
                            l,
                            style,
                            tx,
                            ty,
                            anim_time,
                        );
                        stats.walls += 1;
                    }
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
                } else if tile.slope != 0 {
                    stats.quads +=
                        push_slope(r, pos, tf.uv, l, style, tx, ty, anim_time, tile.slope);
                } else if tile.half_brick() {
                    push_half(r, pos, tf.uv, l, style, tx, ty, anim_time);
                    stats.quads += 1;
                } else if style.sliced_blocks
                    && solid
                    && tf.opaque
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

    stats.liquids = draw_liquids(r, world, lookups, light, style, anim_time, x0, y0, x1, y1);
    stats.foliage = draw_tree_foliage(r, world, lookups, light, style, anim_time, x0, y0, x1, y1);
    stats.instances = stats.quads + stats.walls + stats.dark_rects + stats.foliage + stats.liquids;
    stats.draw_ms = t_total.elapsed().as_secs_f32() * 1000.0;
    stats
}

/// `LiquidRenderer.DrawNormalLiquids`, first pass: one animated 16x16 sprite per
/// liquid tile, cropped to the liquid height, with the surface frame on the top
/// layer and a small per-column sine wave. Lighting is flat per tile.
#[allow(clippy::too_many_arguments)]
fn draw_liquids(
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
    let frame = ((anim_time * 8.0) as usize) % 16;
    let mut quads = 0;
    for ty in y0..=y1 {
        for tx in x0..=x1 {
            let Some(t) = world.tile(tx, ty) else {
                continue;
            };
            if t.liquid == 0 {
                continue;
            }
            let (tex, opacity, amplitude) = match t.liquid_kind() {
                LiquidKind::Lava => (1, 0.95, 0.25),
                LiquidKind::Honey => (11, 0.95, 0.15),
                LiquidKind::Shimmer => (14, 0.75, 1.3),
                LiquidKind::Water => (0, 0.60, 1.0),
            };
            let l = light.get(tx, ty);
            if t.liquid_kind() != LiquidKind::Lava && light_is_zero(l) {
                continue;
            }
            let full = t.liquid >= 255;
            let surface = !full
                || world
                    .tile(tx, ty - 1)
                    .is_none_or(|a| a.liquid == 0 || a.liquid_kind() != t.liquid_kind());
            let h = if full {
                16.0
            } else {
                (t.liquid as f32 / 255.0 * 16.0).max(1.0)
            };
            // 80 px animation blocks; block 16 is the still surface frame.
            let base_y = if surface {
                1280
            } else {
                32 + frame as i32 * 80
            };
            let crop_y = base_y + (16.0 - h) as i32;
            let Some(uv) = lookups.liquid_uv(tex, 16, crop_y, 16, h as u32) else {
                continue;
            };
            let wave = (tx as f32 * 0.55 + anim_time * 2.0).sin() * amplitude;
            // Slight depth falloff; the light map already attenuates, but the sky
            // showing through a shallow open lake otherwise keeps deep water bright.
            let mut depth = 0u32;
            for d in 1..=8 {
                if world
                    .tile(tx, ty - d)
                    .is_some_and(|a| a.liquid > 0 && a.liquid_kind() == t.liquid_kind())
                {
                    depth += 1;
                } else {
                    break;
                }
            }
            let depth_tint = 1.0 - (depth as f32 * 0.035).min(0.35);
            let c = style.shade(l, tx, ty, anim_time);
            r.push_sprite_flags(
                [tx as f32 * TILE_PX, ty as f32 * TILE_PX + 16.0 - h + wave],
                [TILE_PX, h + 1.0],
                uv,
                [
                    c[0] * depth_tint,
                    c[1] * depth_tint,
                    c[2] * depth_tint,
                    opacity,
                ],
                1.0,
            );
            quads += 1;
        }
    }
    quads
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
    let du = (uv[2] - uv[0]) / 16.0;
    let dv = (uv[3] - uv[1]) / 16.0;
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

#[allow(clippy::too_many_arguments)]
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
    lines.push((
        format!(
            "visible tiles {}   quads {}   foliage {}   liquids {}   total {}   grid {}x{}",
            tile.visible,
            tile.quads,
            tile.foliage,
            tile.liquids,
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
        "WASD pan   wheel zoom   F1 ui   F5 spawn   ESC quit".into(),
        WARN,
    ));

    let panel_w = 660.0;
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
