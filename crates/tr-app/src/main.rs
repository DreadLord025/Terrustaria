//! Terraria-Rust: Vulkan renderer prototype for a native reimplementation.

mod assets;

use std::collections::{HashSet, VecDeque};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;

use anyhow::{Context, Result};
use glam::Vec2;
use tr_core::{wld, World, TILE_PX};
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
    let world_path = match args.get(1) {
        Some(p) => PathBuf::from(p),
        None => find_world().context("no .wld given and none found in Documents/My Games/Terraria/Worlds")?,
    };

    log::info!("world: {}", world_path.display());
    let t0 = Instant::now();
    let world = wld::load(&world_path)?;
    log::info!(
        "loaded '{}' {}x{} ({} tiles, {:.1} MB) in {:.0} ms",
        world.name,
        world.width,
        world.height,
        world.tiles.len(),
        world.memory_bytes() as f32 / 1e6,
        t0.elapsed().as_secs_f32() * 1000.0
    );

    let assets_dir = find_assets_dir().context("assets/Images not found (run tools/XnbExtract first)")?;

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
                    "{}:{}/{}/w{} ",
                    if t.active() { t.kind } else { 999 },
                    t.frame_x,
                    t.frame_y,
                    t.wall
                ));
            }
            log::info!("row {y}: {line}");
        }
    }
    let (tile_ids, wall_ids) = assets::usage(&world);
    log::info!("world uses {} tile types and {} wall types", tile_ids.len(), wall_ids.len());

    let t1 = Instant::now();
    let mut images = assets::load_world_images(&assets_dir, &tile_ids, &wall_ids)?;
    let font = tr_render::atlas::font_sheet();
    images.push(font.0);
    log::info!("decoded {} sheets in {:.0} ms", images.len(), t1.elapsed().as_secs_f32() * 1000.0);

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
        tile_ids.iter().map(|&id| (id, assets::image_name_for_tile(id))),
        wall_ids.iter().map(|&id| (id, assets::image_name_for_wall(id))),
    );

    let atlas_stats = (atlas.width, atlas.height, atlas.memory_bytes(), atlas.sheets.len());

    if std::env::var("TR_CHECK_FRAMES").is_ok() {
        for (kind, tt) in &lookups.tiles {
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
                    log::warn!("tile {kind} ({}) plain frame is {opaque}/256 opaque", tt.sheet);
                }
            }
        }
        for (wall, tt) in &lookups.walls {
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
                    log::warn!("wall {wall} ({}) plain frame is {opaque}/256 opaque", tt.sheet);
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
        for name in ["Tiles_0", "Tiles_1", "Tiles_2", "Wall_1"] {
            if let Some(s) = atlas.sheet(name) {
                log::info!(
                    "{name}: atlas rect {}x{} at {},{} plain {:?}",
                    s.width, s.height, s.x, s.y, s.plain_frame
                );
            }
        }
    }

    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App {
        window: None,
        gpu: None,
        renderer: None,
        world,
        atlas: Some(atlas),
        lookups,
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
        shot_at: std::env::var("TR_SHOT_FRAME").ok().and_then(|v| v.parse().ok()).unwrap_or(20),
        atlas_stats,
    };
    event_loop.run_app(&mut app)?;
    Ok(())
}

fn find_world() -> Option<PathBuf> {
    let dir = dirs_worlds()?;
    let mut best: Option<(std::time::SystemTime, PathBuf)> = None;
    for e in std::fs::read_dir(dir).ok()? {
        let e = e.ok()?;
        let p = e.path();
        if p.extension().is_some_and(|x| x == "wld") {
            let m = e.metadata().ok()?.modified().ok()?;
            if best.as_ref().is_none_or(|(t, _)| m > *t) {
                best = Some((m, p));
            }
        }
    }
    best.map(|(_, p)| p)
}

fn dirs_worlds() -> Option<PathBuf> {
    let home = std::env::var_os("USERPROFILE")?;
    Some(PathBuf::from(home).join("Documents").join("My Games").join("Terraria").join("Worlds"))
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
        let atlas = self.atlas.take().expect("atlas");

        let fmt = gpu.config.format;
        let renderer = SpriteRenderer::new(&gpu, &atlas, 8);

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
                            KeyCode::F5 => {
                                self.camera = Vec2::new(
                                    self.world.width as f32 * TILE_PX * 0.5,
                                    self.world.spawn_y as f32 * TILE_PX,
                                );
                                self.zoom = 1.5;
                            }
                            KeyCode::Equal | KeyCode::NumpadAdd => self.zoom = (self.zoom * 1.25).min(8.0),
                            KeyCode::Minus | KeyCode::NumpadSubtract => self.zoom = (self.zoom / 1.25).max(0.25),
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
        let dt = now.duration_since(self.last_frame).as_secs_f32().clamp(0.0, 0.1);
        self.last_frame = now;
        self.frame_count += 1;

        self.pan_camera(dt);

        let screen = self
            .gpu
            .as_ref()
            .map(|g| Vec2::new(g.config.width as f32, g.config.height as f32))
            .unwrap_or(Vec2::new(1600.0, 900.0));

        let mut stats = tr_render::sprite::RenderStats::default();

        {
            let App {
                gpu,
                renderer,
                world,
                atlas,
                lookups,
                camera,
                zoom,
                debug,
                frame_history,
                last_stats,
                frame_count,
                shot_path,
                shot_at,
                atlas_stats,
                ..
            } = self;

            if let (Some(gpu), Some(renderer)) = (gpu.as_ref(), renderer.as_mut()) {
                renderer.begin_frame();

                draw_sky(
                    renderer,
                    *camera,
                    *zoom,
                    screen,
                    world.spawn_y as f32 * TILE_PX,
                );

                let visible = draw_tiles(
                    renderer,
                    world,
                    lookups,
                    atlas.as_ref(),
                    *camera,
                    *zoom,
                    screen,
                );

                if *debug {
                    draw_debug(
                        renderer,
                        screen,
                        visible,
                        frame_history,
                        *camera,
                        *zoom,
                        *last_stats,
                        world,
                        *atlas_stats,
                        gpu,
                        dt,
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
                let view = frame.texture.create_view(&wgpu::TextureViewDescriptor::default());

                stats = renderer.render(
                    gpu,
                    &view,
                    camera.to_array(),
                    *zoom,
                    now.elapsed().as_secs_f32(),
                    wgpu::Color { r: 0.045, g: 0.055, b: 0.085, a: 1.0 },
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
            self.last_stats.draw_calls, self.last_stats.instances, self.world.tiles.len(), self.zoom
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
        "A", "B", "V", "G", "D", "E", "Yo", "Zh", "Z", "I", "Y", "K", "L", "M", "N", "O",
        "P", "R", "S", "T", "U", "F", "Kh", "Ts", "Ch", "Sh", "Shch", "", "Y", "", "E", "Yu",
        "Ya", "a", "b", "v", "g", "d", "e", "yo", "zh", "z", "i", "y", "k", "l", "m", "n",
        "o", "p", "r", "s", "t", "u", "f", "kh", "ts", "ch", "sh", "shch", "", "y", "", "e",
        "yu", "ya",
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

fn draw_sky(r: &mut SpriteRenderer, camera: Vec2, zoom: f32, screen: Vec2, horizon_px: f32) {
    // Screen-space gradient: sky at the surface, fading to cave darkness with depth.
    // Replaced later by the real sky/parallax/background layers.
    const STRIPS: usize = 64;
    let half = screen * 0.5 / zoom;
    let origin = camera - half;
    let strip_h = screen.y / zoom / STRIPS as f32;
    let white = r.white_uv();

    let top = [0.10f32, 0.16, 0.30];
    let horizon = [0.62f32, 0.70, 0.82];
    let cave_near = [0.085f32, 0.075, 0.075];
    let cave_far = [0.02f32, 0.018, 0.022];

    // 0 at the surface line, 1 about 20 tiles below it.
    let depth = ((camera.y - horizon_px) / (20.0 * TILE_PX)).clamp(0.0, 1.0);

    for i in 0..STRIPS {
        // `t` is screen-relative so the gradient never produces a hard world-space band.
        let t = i as f32 / (STRIPS - 1) as f32;
        let sky = lerp3(top, horizon, t);
        let cave = lerp3(cave_near, cave_far, t);
        let c = [
            sky[0] + (cave[0] - sky[0]) * depth,
            sky[1] + (cave[1] - sky[1]) * depth,
            sky[2] + (cave[2] - sky[2]) * depth,
            1.0,
        ];
        r.push_sprite(
            [origin.x, origin.y + i as f32 * strip_h],
            [screen.x / zoom, strip_h + 1.0],
            white,
            c,
        );
    }
}

fn draw_tiles(
    r: &mut SpriteRenderer,
    world: &World,
    lookups: &mut assets::Lookups,
    atlas: Option<&Atlas>,
    camera: Vec2,
    zoom: f32,
    screen: Vec2,
) -> u32 {
    let half = screen * 0.5 / zoom;
    let min = camera - half;
    let max = camera + half;
    let x0 = ((min.x / TILE_PX).floor() as i32 - 1).clamp(0, world.width - 1);
    let x1 = ((max.x / TILE_PX).ceil() as i32 + 1).clamp(0, world.width - 1);
    let y0 = ((min.y / TILE_PX).floor() as i32 - 1).clamp(0, world.height - 1);
    let y1 = ((max.y / TILE_PX).ceil() as i32 + 1).clamp(0, world.height - 1);

    let mut visible = 0u32;
    let wall_tint = [0.72, 0.72, 0.74, 1.0];

    for ty in y0..=y1 {
        for tx in x0..=x1 {
            let Some(tile) = world.tile(tx, ty) else { continue };
            let pos = [tx as f32 * TILE_PX, ty as f32 * TILE_PX];

            if tile.wall != 0 {
                if let Some(tt) = lookups.walls.get(&tile.wall) {
                    r.push_sprite(pos, [TILE_PX, TILE_PX], tt.plain_uv, wall_tint);
                }
            }

            if tile.active() {
                let uv = if tile.frame_x >= 0 && tile.frame_y >= 0 {
                    atlas.and_then(|a| lookups.frame_uv(a, tile.kind, tile.frame_x, tile.frame_y))
                } else {
                    None
                };
                let uv = uv.or_else(|| lookups.tiles.get(&tile.kind).map(|t| t.plain_uv));
                if let Some(uv) = uv {
                    r.push_sprite(pos, [TILE_PX, TILE_PX], uv, [1.0, 1.0, 1.0, 1.0]);
                    visible += 1;
                }
            }
        }
    }
    visible
}

#[allow(clippy::too_many_arguments)]
fn draw_debug(
    r: &mut SpriteRenderer,
    screen: Vec2,
    visible: u32,
    history: &VecDeque<f32>,
    camera: Vec2,
    zoom: f32,
    last_stats: tr_render::sprite::RenderStats,
    world: &World,
    atlas_stats: (u32, u32, usize, usize),
    gpu: &Gpu,
    dt: f32,
) {
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

    let info = gpu.adapter.get_info();

    lines.push(("TERRARIA-RUST  //  native rust + vulkan".into(), ACCENT));
    lines.push((format!("fps {fps:6.1}   frame {avg:5.2} ms   worst {worst:5.2} ms   dt {:5.2}", dt * 1000.0), TEXT));
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
            "visible tiles {}   total {}   grid {}x{}",
            visible,
            world.tiles.len(),
            world.width,
            world.height
        ),
        TEXT,
    ));
    lines.push((
        format!(
            "world '{}'   tile mem {:.0} MB   atlas {}x{} ({:.0} MB)",
            display_name(&world.name),
            world.memory_bytes() as f32 / 1e6,
            atlas_stats.0,
            atlas_stats.1,
            atlas_stats.2 as f32 / 1e6,
        ),
        TEXT,
    ));
    lines.push((
        format!(
            "gpu {} [{:?}]   sheets {}",
            info.name, info.backend, atlas_stats.3
        ),
        TEXT,
    ));
    lines.push((format!("screen {:.0}x{:.0}   atlas_max 8192", screen.x, screen.y), TEXT));
    lines.push(("WASD pan   wheel zoom   F1 ui   F5 spawn   ESC quit".into(), WARN));

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
        .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("screenshot") });
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
        wgpu::Extent3d { width: w, height: h, depth_or_array_layers: 1 },
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
