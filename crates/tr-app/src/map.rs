//! Minimap and full map (`Main.DrawMap`, `MapHelper`): explored-tile fog, one colour
//! per tile/wall type sampled from the original textures (plus water), drawn as 8x8
//! tile chunk blocks.

use glam::Vec2;
use tr_core::tile::LiquidKind;
use tr_core::{TILE_PX, World};
use tr_render::{Atlas, SpriteRenderer};

use crate::assets;
use crate::player::Player;
use crate::{hud_text, hud_text_width};

pub struct WorldMap {
    w: i32,
    h: i32,
    explored: Vec<u64>,
    tile_colors: Vec<[f32; 3]>,
    wall_colors: Vec<[f32; 3]>,
    pub open: bool,
    timer: f32,
}

const CHUNK: i32 = 8;

impl WorldMap {
    pub fn new(world: &World, atlas: &Atlas, lookups: &assets::Lookups) -> Self {
        let w = world.width;
        let h = world.height;
        let words = (w as usize * h as usize).div_ceil(64);
        let mut tile_colors = vec![[0.10, 0.08, 0.07]; tr_core::tiledata::TILE_COUNT];
        let mut wall_colors = vec![[0.06, 0.05, 0.05]; tr_core::tiledata::WALL_COUNT];
        for k in 0..tr_core::tiledata::TILE_COUNT as u16 {
            if let Some(c) = lookups.tile_color_avg(atlas, k) {
                tile_colors[k as usize] = c;
            }
        }
        for k in 0..tr_core::tiledata::WALL_COUNT as u16 {
            if let Some(c) = lookups.wall_color_avg(atlas, k) {
                wall_colors[k as usize] = c;
            }
        }
        Self {
            w,
            h,
            explored: vec![0; words],
            tile_colors,
            wall_colors,
            open: false,
            timer: 1.0,
        }
    }

    fn idx(&self, x: i32, y: i32) -> Option<usize> {
        if x < 0 || y < 0 || x >= self.w || y >= self.h {
            return None;
        }
        Some(y as usize * self.w as usize + x as usize)
    }

    fn reveal(&mut self, x: i32, y: i32) {
        if let Some(i) = self.idx(x, y) {
            self.explored[i / 64] |= 1 << (i % 64);
        }
    }

    fn explored(&self, x: i32, y: i32) -> bool {
        self.idx(x, y)
            .is_some_and(|i| self.explored[i / 64] & (1 << (i % 64)) != 0)
    }

    /// Reveals a screen-sized window around the player a few times per second.
    pub fn update(&mut self, player: Option<&Player>, dt: f32) {
        let Some(p) = player else { return };
        self.timer += dt;
        if self.timer < 0.25 {
            return;
        }
        self.timer = 0.0;
        let px = (p.x / TILE_PX) as i32;
        let py = (p.y / TILE_PX) as i32;
        for y in py - 22..=py + 22 {
            for x in px - 36..=px + 36 {
                self.reveal(x, y);
            }
        }
    }

    fn chunk_color(&self, world: &World, cx: i32, cy: i32) -> Option<[f32; 3]> {
        let mut acc = [0f32; 3];
        let mut n = 0u32;
        for ty in cy * CHUNK..(cy + 1) * CHUNK {
            for tx in cx * CHUNK..(cx + 1) * CHUNK {
                if !self.explored(tx, ty) {
                    continue;
                }
                let Some(t) = world.tile(tx, ty) else { continue };
                let mut c = if t.wall != 0 {
                    let w = self
                        .wall_colors
                        .get(t.wall as usize)
                        .copied()
                        .unwrap_or([0.08, 0.07, 0.06]);
                    [w[0] * 0.55, w[1] * 0.55, w[2] * 0.55]
                } else {
                    [0.02, 0.02, 0.025]
                };
                if t.active() {
                    c = self
                        .tile_colors
                        .get(t.kind as usize)
                        .copied()
                        .unwrap_or([0.4, 0.35, 0.3]);
                }
                if t.liquid > 0 {
                    let kind = t.liquid_kind();
                    let water = match kind {
                        LiquidKind::Lava => [0.95, 0.35, 0.08],
                        LiquidKind::Honey => [0.85, 0.6, 0.12],
                        LiquidKind::Shimmer => [0.6, 0.4, 0.95],
                        LiquidKind::Water => [0.12, 0.28, 0.65],
                    };
                    let k = (t.liquid as f32 / 255.0) * 0.75;
                    for i in 0..3 {
                        c[i] = c[i] * (1.0 - k) + water[i] * k;
                    }
                }
                acc[0] += c[0];
                acc[1] += c[1];
                acc[2] += c[2];
                n += 1;
            }
        }
        (n > 0).then(|| [acc[0] / n as f32, acc[1] / n as f32, acc[2] / n as f32])
    }

    #[allow(clippy::too_many_arguments)]
    fn draw_chunks(
        &self,
        r: &mut SpriteRenderer,
        world: &World,
        origin: Vec2,
        tiles_w: i32,
        tiles_h: i32,
        px_per_tile: f32,
        center: (i32, i32),
        alpha: f32,
    ) {
        let half_w = tiles_w / 2;
        let half_h = tiles_h / 2;
        let start_x = center.0 - half_w;
        let start_y = center.1 - half_h;
        let block = CHUNK as f32 * px_per_tile;
        let cx0 = start_x.div_euclid(CHUNK);
        let cy0 = start_y.div_euclid(CHUNK);
        let cx1 = (start_x + tiles_w).div_euclid(CHUNK);
        let cy1 = (start_y + tiles_h).div_euclid(CHUNK);
        for cy in cy0..=cy1 {
            for cx in cx0..=cx1 {
                let Some(c) = self.chunk_color(world, cx, cy) else {
                    continue;
                };
                let px = origin.x + (cx * CHUNK - start_x) as f32 * px_per_tile;
                let py = origin.y + (cy * CHUNK - start_y) as f32 * px_per_tile;
                r.push_ui_rect([px, py], [block, block], [c[0], c[1], c[2], alpha]);
            }
        }
    }

    /// Small map under the hearts (vanilla minimap), 2 px per tile.
    pub fn draw_minimap(
        &self,
        r: &mut SpriteRenderer,
        world: &World,
        screen: Vec2,
        player_tile: (i32, i32),
    ) {
        let px_per_tile = 2.0;
        let tiles_w = 90;
        let tiles_h = 60;
        let size = Vec2::new(tiles_w as f32 * px_per_tile, tiles_h as f32 * px_per_tile);
        let origin = Vec2::new(screen.x - size.x - 20.0, 92.0);
        r.push_ui_rect(
            [origin.x - 2.0, origin.y - 2.0],
            [size.x + 4.0, size.y + 4.0],
            [0.05, 0.06, 0.1, 0.9],
        );
        self.draw_chunks(
            r, world, origin, tiles_w, tiles_h, px_per_tile, player_tile, 1.0,
        );
        // Player marker at the centre of the centred window.
        let mx = origin.x + (tiles_w as f32 * 0.5) * px_per_tile - 2.0;
        let my = origin.y + (tiles_h as f32 * 0.5) * px_per_tile - 3.0;
        r.push_ui_rect([mx, my], [5.0, 7.0], [1.0, 1.0, 0.85, 1.0]);
    }

    /// Full-screen map (`Main.DrawMap`), toggled with M.
    pub fn draw_full(
        &self,
        r: &mut SpriteRenderer,
        lookups: &assets::Lookups,
        world: &World,
        screen: Vec2,
        player_tile: (i32, i32),
    ) {
        r.push_ui_rect([0.0, 0.0], [screen.x, screen.y], [0.02, 0.03, 0.05, 0.93]);
        let px_per_tile = 3.0;
        let tiles_w = (screen.x / px_per_tile) as i32 - 40;
        let tiles_h = (screen.y / px_per_tile) as i32 - 80;
        let origin = Vec2::new(
            (screen.x - tiles_w as f32 * px_per_tile) * 0.5,
            (screen.y - tiles_h as f32 * px_per_tile) * 0.5 + 10.0,
        );
        self.draw_chunks(
            r, world, origin, tiles_w, tiles_h, px_per_tile, player_tile, 1.0,
        );
        let mx = origin.x + (tiles_w as f32 * 0.5) - 3.0;
        let my = origin.y + (tiles_h as f32 * 0.5) - 4.0;
        r.push_ui_rect([mx, my], [7.0, 9.0], [1.0, 1.0, 0.8, 1.0]);
        let title = "Map";
        let tw = hud_text_width(r, lookups, title, 1.3);
        hud_text(
            r,
            lookups,
            (screen.x - tw) * 0.5,
            18.0,
            1.3,
            [0.95, 0.92, 0.8, 1.0],
            title,
        );
        let hint = "M to close";
        let tw = hud_text_width(r, lookups, hint, 0.8);
        hud_text(
            r,
            lookups,
            (screen.x - tw) * 0.5,
            screen.y - 34.0,
            0.8,
            [0.6, 0.65, 0.72, 1.0],
            hint,
        );
    }
}
