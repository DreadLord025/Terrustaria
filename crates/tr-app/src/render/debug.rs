//! The debug overlay (`TR_DEBUG`) and the GPU screenshot capture, plus the
//! `TR_STATS` per-phase accumulator and the static-overlay text cache.
use crate::player::{self, Player};
use crate::render::sky::display_name;
use crate::render::tiles::TileDrawStats;
use glam::Vec2;
use std::collections::VecDeque;
use tr_core::{TILE_PX, World};
use tr_render::{Gpu, SpriteRenderer};

const PANEL_BG: [f32; 4] = [0.03, 0.04, 0.06, 0.82];
const TEXT: [f32; 4] = [0.85, 0.90, 0.95, 1.0];
const ACCENT: [f32; 4] = [0.45, 0.85, 0.55, 1.0];
const WARN: [f32; 4] = [0.95, 0.70, 0.30, 1.0];

/// Static overlay text, built once instead of every frame (G4).
#[derive(Default)]
pub(crate) struct DebugCache {
    ready: bool,
    gpu_line: String,
    world_line: String,
}

/// Accumulates per-phase numbers for `TR_STATS` and prints them every 120 frames.
#[derive(Default)]
pub(crate) struct StatsAcc {
    pub(crate) frames: u32,
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
    pub(crate) fn add(&mut self, s: &TileDrawStats) {
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

    pub(crate) fn log_and_reset(&mut self) {
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

pub(crate) fn draw_debug(
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

pub(crate) fn capture_frame(gpu: &Gpu, texture: &wgpu::Texture, path: &std::path::Path) {
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
