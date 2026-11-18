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

/// The player snapshot the debug window reads (Phase 1 of the debug-window
/// plan). A copy out of [`Player`] so the formatter stays a pure function and
/// learns nothing about the entity type.
#[derive(Debug, Clone, Copy)]
pub(crate) struct DebugPlayer {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub on_ground: bool,
    pub in_water: bool,
    pub in_lava: bool,
    pub hp: i32,
    pub max_hp: i32,
    pub mana: i32,
    pub mana_max: i32,
    pub buffs: usize,
}

impl DebugPlayer {
    pub(crate) fn from_player(p: &Player) -> Self {
        Self {
            x: p.x,
            y: p.y,
            vx: p.vx,
            vy: p.vy,
            on_ground: p.on_ground,
            in_water: p.in_water,
            in_lava: p.in_lava,
            hp: p.hp,
            max_hp: p.max_hp,
            mana: p.mana,
            mana_max: p.mana_max,
            buffs: p.buffs.active_count(),
        }
    }
}

/// The tile under the cursor, snapshotted by the frame hook.
#[derive(Debug, Clone, Copy)]
pub(crate) struct DebugTile {
    pub x: i32,
    pub y: i32,
    pub active: bool,
    pub kind: u16,
    pub wall: u16,
    pub frame_x: i32,
    pub frame_y: i32,
    pub slope: u8,
    pub liquid: u8,
}

/// Everything the always-shown debug window paints. Phase 1 covers the static
/// values only (frame timing, player, the tile under the cursor, entity
/// counts); the world/biome rows are Phases 2-3.
pub(crate) struct DebugWindowInput {
    pub screen: Vec2,
    pub camera: Vec2,
    pub zoom: f32,
    pub mouse: Vec2,
    pub dt: f32,
    pub history: Vec<f32>,
    pub draw_calls: u32,
    pub instances: u32,
    pub player: Option<DebugPlayer>,
    pub tile: Option<DebugTile>,
    pub npcs: usize,
    pub projectiles: usize,
    pub items: usize,
    pub dust: usize,
    pub gore: usize,
}

/// The tile coordinates under a screen-space pointer, unclamped (so an
/// off-world cursor reads as an empty tile rather than the edge row).
pub(crate) fn cursor_tile(mouse: Vec2, screen: Vec2, camera: Vec2, zoom: f32) -> (i32, i32) {
    let world_x = camera.x + (mouse.x - screen.x * 0.5) / zoom.max(f32::EPSILON);
    let world_y = camera.y + (mouse.y - screen.y * 0.5) / zoom.max(f32::EPSILON);
    ((world_x / TILE_PX) as i32, (world_y / TILE_PX) as i32)
}

/// Builds the debug window's text rows. Pure — no GPU, no `App` — so the
/// formatting is unit-testable; [`draw_debug_window`] only lays the rows out.
pub(crate) fn format_debug_lines(i: &DebugWindowInput) -> Vec<(String, [f32; 4])> {
    let avg = if i.history.is_empty() {
        0.0
    } else {
        i.history.iter().sum::<f32>() / i.history.len() as f32
    };
    let fps = if avg > 0.0 { 1000.0 / avg } else { 0.0 };
    let worst = i.history.iter().copied().fold(0.0f32, f32::max);

    let mut lines: Vec<(String, [f32; 4])> = Vec::new();
    lines.push(("DEBUG WINDOW  //  F1 overlay".into(), ACCENT));
    lines.push((
        format!(
            "fps {fps:6.1}   frame {avg:5.2} ms   worst {worst:5.2} ms   dt {:5.2} ms",
            i.dt * 1000.0
        ),
        TEXT,
    ));
    lines.push((
        format!(
            "draws {}   instances {}",
            i.draw_calls, i.instances
        ),
        TEXT,
    ));
    lines.push((
        format!(
            "camera {:.0},{:.0} px   tile {:.1},{:.1}   zoom {:.2}",
            i.camera.x,
            i.camera.y,
            i.camera.x / TILE_PX,
            i.camera.y / TILE_PX,
            i.zoom
        ),
        TEXT,
    ));
    match &i.player {
        Some(p) => {
            lines.push((
                format!(
                    "player {:.1},{:.1} px   tile {:.2},{:.2}",
                    p.x,
                    p.y,
                    p.x / TILE_PX,
                    p.y / TILE_PX
                ),
                TEXT,
            ));
            lines.push((
                format!(
                    "vel {:.2},{:.2}   ground {}   water {}   lava {}   hp {}/{}   mana {}/{}   buffs {}",
                    p.vx,
                    p.vy,
                    p.on_ground,
                    p.in_water,
                    p.in_lava,
                    p.hp,
                    p.max_hp,
                    p.mana,
                    p.mana_max,
                    p.buffs
                ),
                TEXT,
            ));
        }
        None => lines.push(("player none".into(), TEXT)),
    }
    lines.push((
        format!(
            "npcs {}   projectiles {}   items {}   dust {}   gore {}",
            i.npcs, i.projectiles, i.items, i.dust, i.gore
        ),
        TEXT,
    ));
    match &i.tile {
        Some(t) => lines.push((
            format!(
                "cursor tile {},{}\tactive {}   kind {}   wall {}   frame {},{}   slope {}   liquid {}",
                t.x, t.y, t.active, t.kind, t.wall, t.frame_x, t.frame_y, t.slope, t.liquid
            ),
            if t.active || t.wall != 0 { TEXT } else { WARN },
        )),
        None => lines.push((
            format!(
                "cursor {:.0},{:.0} screen px",
                i.mouse.x, i.mouse.y
            ),
            TEXT,
        )),
    }
    lines
}

/// Paints the debug window at the top-left. Phase 1 only lays out the rows
/// from [`format_debug_lines`]; the draw call itself stays GPU-only.
pub(crate) fn draw_debug_window(r: &mut SpriteRenderer, i: &DebugWindowInput) {
    let lines = format_debug_lines(i);
    let scale = 1.0;
    let line_h = 14.0;
    let pad = 8.0;
    let panel_w = lines
        .iter()
        .map(|(text, _)| r.text_width(text, scale))
        .fold(0.0f32, f32::max)
        + pad * 2.0;
    let panel_h = lines.len() as f32 * line_h + pad * 2.0;
    let x = 14.0;
    let y = 14.0;

    r.push_ui_rect([x, y], [panel_w, panel_h], PANEL_BG);
    let mut ty = y + pad;
    for (text, color) in &lines {
        r.push_text(x + pad, ty, scale, *color, text);
        ty += line_h;
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

#[cfg(test)]
mod tests {
    use super::*;

    fn base_input() -> DebugWindowInput {
        DebugWindowInput {
            screen: Vec2::new(1920.0, 1080.0),
            camera: Vec2::new(1600.0, 900.0),
            zoom: 1.5,
            mouse: Vec2::new(960.0, 540.0),
            dt: 1.0 / 60.0,
            history: vec![16.0, 17.0],
            draw_calls: 42,
            instances: 1234,
            player: None,
            tile: None,
            npcs: 3,
            projectiles: 4,
            items: 5,
            dust: 6,
            gore: 7,
        }
    }

    /// A pointer dead-centre reads the camera's own tile.
    #[test]
    fn cursor_tile_maps_the_screen_centre_to_the_camera_tile() {
        let (x, y) = cursor_tile(
            Vec2::new(960.0, 540.0),
            Vec2::new(1920.0, 1080.0),
            Vec2::new(1600.0, 900.0),
            1.5,
        );
        assert_eq!((x, y), (100, 56));
    }

    /// Zoom 2 makes the pointer cover half the world distance zoom 1 does.
    #[test]
    fn cursor_tile_shrinks_with_zoom() {
        let screen = Vec2::new(1920.0, 1080.0);
        let camera = Vec2::new(1600.0, 900.0);
        let mouse = Vec2::new(1920.0, 1080.0);
        let (x1, y1) = cursor_tile(mouse, screen, camera, 1.0);
        let (x2, y2) = cursor_tile(mouse, screen, camera, 2.0);
        assert!(x2 < x1, "zoom 2 must cover less distance ({x2} vs {x1})");
        assert!(y2 < y1, "zoom 2 must cover less distance ({y2} vs {y1})");
    }

    /// Everything in the header and the counter row is present, and the
    /// frame-rate line formats the measured values.
    #[test]
    fn lines_carry_frame_stats_and_entity_counts() {
        let lines = format_debug_lines(&base_input());
        let text: Vec<&str> = lines.iter().map(|(t, _)| t.as_str()).collect();
        assert!(text[0].contains("DEBUG WINDOW"));
        assert!(text[1].contains("frame 16.50 ms"));
        assert!(text[1].contains("worst 17.00 ms"));
        assert!(text[1].contains("fps"));
        assert!(text[2].contains("draws 42") && text[2].contains("instances 1234"));
        assert!(
            text.iter().any(|t| t == &"npcs 3   projectiles 4   items 5   dust 6   gore 7"),
            "entity row missing: {text:?}"
        );
    }

    /// A player snapshot renders the position, the collision flags and the
    /// vitals; a missing player says so instead of panicking.
    #[test]
    fn player_row_reports_vitals_and_absence() {
        let mut input = base_input();
        input.player = Some(DebugPlayer {
            x: 2400.0,
            y: 1200.0,
            vx: -0.5,
            vy: 2.25,
            on_ground: true,
            in_water: false,
            in_lava: false,
            hp: 90,
            max_hp: 100,
            mana: 10,
            mana_max: 20,
            buffs: 2,
        });
        let lines = format_debug_lines(&input);
        let text: Vec<&str> = lines.iter().map(|(t, _)| t.as_str()).collect();
        assert!(text.iter().any(|t| t.contains("player 2400.0,1200.0 px")));
        let vitals = text
            .iter()
            .find(|t| t.starts_with("vel "))
            .expect("velocity row missing");
        assert!(vitals.contains("ground true"));
        assert!(vitals.contains("water false"));
        assert!(vitals.contains("hp 90/100"));
        assert!(vitals.contains("mana 10/20"));
        assert!(vitals.contains("buffs 2"));

        input.player = None;
        let text: Vec<String> = format_debug_lines(&input)
            .into_iter()
            .map(|(t, _)| t)
            .collect();
        assert!(text.iter().any(|t| t == "player none"));
    }

    /// An empty cursor tile is flagged WARN; a solid one is not, and both
    /// show the tile's own fields.
    #[test]
    fn cursor_row_shows_tile_fields_and_warns_when_empty() {
        let mut input = base_input();
        input.tile = Some(DebugTile {
            x: 100,
            y: 56,
            active: true,
            kind: 1,
            wall: 2,
            frame_x: 3,
            frame_y: 4,
            slope: 1,
            liquid: 5,
        });
        let lines = format_debug_lines(&input);
        let (row, color) = lines
            .iter()
            .find(|(t, _)| t.starts_with("cursor tile "))
            .expect("cursor row missing");
        assert!(row.contains("100,56"));
        assert!(row.contains("kind 1"));
        assert!(row.contains("wall 2"));
        assert!(row.contains("frame 3,4"));
        assert!(row.contains("slope 1"));
        assert!(row.contains("liquid 5"));
        assert_eq!(*color, TEXT);

        input.tile = Some(DebugTile {
            active: false,
            wall: 0,
            ..input.tile.unwrap()
        });
        let lines = format_debug_lines(&input);
        let (_, color) = lines
            .iter()
            .find(|(t, _)| t.starts_with("cursor tile "))
            .expect("cursor row missing");
        assert_eq!(*color, WARN);
    }

    /// With no tile snapshot the cursor's screen position is reported, so the
    /// row never silently disappears.
    #[test]
    fn cursor_row_falls_back_to_the_screen_position() {
        let lines = format_debug_lines(&base_input());
        assert!(lines
            .iter()
            .any(|(t, _)| t == "cursor 960,540 screen px"));
    }
}
