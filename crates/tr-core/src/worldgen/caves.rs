//! Cave carving and the dirt/stone/ore blob mixes.

use super::noise::{Noise, Rng};
use super::{
    TILE_ASH, TILE_COPPER, TILE_DIRT, TILE_GOLD, TILE_HELLSTONE, TILE_IRON, TILE_SILVER, TILE_STONE,
};
use crate::tile::FLAG_ACTIVE;
use crate::world::World;

/// Carves cave air with a large cavern field plus a narrow worm-tunnel field,
/// keeping a crust under the surface and fading out near hell.
pub(crate) fn carve_caves(
    world: &mut World,
    width: i32,
    noise: &Noise,
    surface: &[i32],
    hell_top: i32,
) {
    // --- caves --------------------------------------------------------------
    for x in 0..width {
        let top = surface[x as usize] + 9;
        for y in top..hell_top {
            let fx = x as f32;
            let fy = y as f32;
            let large = noise.fbm(fx * 0.018, fy * 0.030, 3, 0.5);
            let tunnel = noise.fbm(fx * 0.055 + 40.0, fy * 0.045, 3, 0.5);
            let cavern = large > 0.30;
            let worm = tunnel.abs() < 0.035;
            // Keep a crust under the surface and fade caves out near hell.
            let depth = (y - surface[x as usize]) as f32;
            let fade = (depth / 24.0).min(1.0);
            if (cavern || worm) && fade > 0.15 && y < hell_top - 10 {
                let i = world.index(x, y);
                world.tiles[i].flags &= !FLAG_ACTIVE;
            }
        }
    }
}

/// Converts `count` round blobs of `from` tiles into `to` tiles.
pub(crate) fn blob_pass(
    world: &mut World,
    rng: &mut Rng,
    width: i32,
    height: i32,
    from: u16,
    to: u16,
    count: usize,
    radius: i32,
) {
    for _ in 0..count {
        let x = rng.range(4, width - 4);
        let y = rng.range(height / 3, height - 4);
        let r = rng.range(1, radius);
        for dy in -r..=r {
            for dx in -r..=r {
                if dx * dx + dy * dy > r * r {
                    continue;
                }
                let (tx, ty) = (x + dx, y + dy);
                if !world.in_bounds(tx, ty) {
                    continue;
                }
                let i = world.index(tx, ty);
                let t = &mut world.tiles[i];
                if t.active() && t.kind == from {
                    t.kind = to;
                }
            }
        }
    }
}

/// Scatters the ore tier blobs below the surface plus hellstone in the ash layer.
pub(crate) fn place_ores(
    world: &mut World,
    rng: &mut Rng,
    surface: &[i32],
    width: i32,
    height: i32,
    hell_top: i32,
) {
    // --- ores ---------------------------------------------------------------
    let ores: [(u16, i32, i32, usize, i32); 4] = [
        (TILE_COPPER, 20, 400, 1400, 4),
        (TILE_IRON, 60, 600, 1100, 4),
        (TILE_SILVER, 160, 800, 800, 3),
        (TILE_GOLD, 280, 1000, 600, 3),
    ];
    for (kind, y0, y1, count, size) in ores {
        for _ in 0..count {
            let x = rng.range(10, width - 10);
            let y = surface[x as usize] + rng.range(y0, y1);
            if y >= hell_top {
                continue;
            }
            ore_blob(world, x, y, kind, rng.range(2, size + 3), rng);
        }
    }
    // Hellstone in the ash layer.
    for _ in 0..900 {
        let x = rng.range(10, width - 10);
        let y = rng.range(hell_top, height - 4);
        ore_blob(world, x, y, TILE_HELLSTONE, rng.range(2, 5), rng);
    }
}

fn ore_blob(world: &mut World, x: i32, y: i32, kind: u16, size: i32, rng: &mut Rng) {
    let mut cx = x;
    let mut cy = y;
    for _ in 0..size {
        if world.in_bounds(cx, cy) {
            let i = world.index(cx, cy);
            let t = &mut world.tiles[i];
            if t.active() && matches!(t.kind, TILE_STONE | TILE_ASH | TILE_DIRT) {
                t.kind = kind;
            }
        }
        cx += rng.range(-1, 2);
        cy += rng.range(-1, 2);
    }
}
