//! `Terraria.Graphics.Light.TileLightScanner` — the per-tile seed light.
//!
//! The original seeds each visible tile with `GetTileLight`
//! (`TileLightScanner.cs:87`), which combines surface sunlight, the underworld glow,
//! glowing walls, self-lit tile types and liquids. The self-lit dispatch lives in
//! [`scanner`]; the colour tables and the per-tile `FastRandom` live in [`palette`].
//!
//! Tile types whose light is a plain constant are still taken from the generated
//! `tiledata::EMISSIVE` table; these modules only own the cases that the table
//! cannot express (conditions, animation, per-tile random jitter, paint overrides).

mod palette;
mod scanner;

pub use palette::{
    hsl_to_rgb, paint_color, shimmer_base_color, torch_color, FastRandom, CAGE_FRAMES, TORCH_LIGHT,
};
pub use scanner::apply_tile_light;

use crate::lighting::LightParams;
use crate::tile::*;
use crate::world::World;

/// `TileLightScanner.ApplyWallLight` (`TileLightScanner.cs:164`) — glowing walls
/// (crystal, gem, living fire...). `wall_paint` is the sparse
/// `Tile.wallColor()` stand-in (`World::wall_paint`); a painted glowing wall
/// takes the paint colour at the /765 scaling of `TileLightScanner.cs:318-324`.
pub fn apply_wall_light(
    tile: &Tile,
    wall_paint: u8,
    rng: &mut FastRandom,
    p: &LightParams,
    c: &mut [f32; 3],
) {
    let blocked = light_blocked(tile);
    let mut l = [0.0f32; 3];
    match tile.wall {
        357 if !blocked => l = [0.15, 0.27, 0.3],
        182 if !blocked => l = [0.24, 0.12, 0.09],
        33 if !blocked => l = [0.09, 0.0525, 0.24],
        174 if !blocked => l = [0.2975, 0.0, 0.0],
        175 if !blocked => l = [0.075, 0.15, 0.4],
        176 if !blocked => l = [0.1, 0.1, 0.1],
        137 if !blocked => {
            // `0.4 + (270 - mouseTextColor) / 1500 + random * 0.0005`
            let n = 0.4
                + (270.0 - p.mouse_text_color as f32) / 1500.0
                + rng.next_range(0, 50) as f32 * 0.0005;
            l = [n, 0.5 * n, 0.1 * n];
        }
        44 if !blocked => {
            l = [
                p.disco[0] as f32 / 255.0 * 0.15,
                p.disco[1] as f32 / 255.0 * 0.15,
                p.disco[2] as f32 / 255.0 * 0.15,
            ]
        }
        154 => l = [0.6, 0.0, 0.6],
        166 => l = [0.6, 0.6, 0.0],
        165 => l = [0.0, 0.0, 0.6],
        156 => l = [0.0, 0.6, 0.0],
        164 => l = [0.6, 0.0, 0.0],
        155 => l = [0.6, 0.6, 0.6],
        153 => l = [0.6, 0.3, 0.0],
        341 if !blocked => l = [0.25, 0.1, 0.0],
        343 if !blocked => l = [0.0, 0.25, 0.0],
        344 if !blocked => l = [0.0, 0.16, 0.34],
        342 if !blocked => l = [0.3, 0.0, 0.17],
        345 if !blocked => l = [0.3, 0.0, 0.35],
        346 if !blocked => {
            l = [
                p.disco[0] as f32 / 255.0 * 0.25,
                p.disco[1] as f32 / 255.0 * 0.25,
                p.disco[2] as f32 / 255.0 * 0.25,
            ]
        }
        _ => {}
    }
    // `TileLightScanner.cs:318-324`: `if (flag && tile.wallColor() != 0)` —
    // the `flag` arms are the `!blocked` glow walls; the paint colour lands at
    // the /765 scaling (3x dimmer than the painted-tile override).
    if wall_paint != 0 && blocked == false && l != [0.0f32; 3] {
        let pc = crate::tile_light::paint_color(wall_paint);
        l = [pc[0] / 3.0, pc[1] / 3.0, pc[2] / 3.0];
    }
    for i in 0..3 {
        if c[i] < l[i] {
            c[i] = l[i];
        }
    }
}

/// `TileLightScanner.LightIsBlocked` (`TileLightScanner.cs:151`).
pub fn light_blocked(tile: &Tile) -> bool {
    tile.active() && crate::tiledata::TILE_BLOCK_LIGHT.has(tile.kind)
}

/// `TileLightScanner.ApplySurfaceLight` (`TileLightScanner.cs:3131`) — daylight
/// through walls and gem walls.
pub fn apply_surface_light(
    world: &World,
    tile: &Tile,
    x: i32,
    y: i32,
    rng: &mut FastRandom,
    p: &LightParams,
    c: &mut [f32; 3],
) {
    let mut l = [0.0f32; 3];
    let avg = (p.sky[0] + p.sky[1] + p.sky[2]) / 3.0;
    let wall_light = wall_is_light(tile.wall);

    if tile.active() && crate::tiledata::SET_ALLOW_LIGHT_IN_WATER.has(tile.kind) {
        if c[0] < avg && wall_light {
            l = p.sky;
        }
    } else if (!tile.active()
        || !crate::tiledata::TILE_NO_SUN_LIGHT.has(tile.kind)
        || ((tile.slope() != 0 || tile.half_brick())
            && world.tile(x, y - 1).map_or(true, |t| t.liquid == 0)
            && world.tile(x, y + 1).map_or(true, |t| t.liquid == 0)
            && world.tile(x - 1, y).map_or(true, |t| t.liquid == 0)
            && world.tile(x + 1, y).map_or(true, |t| t.liquid == 0)))
        && c[0] < avg
        && wall_light
    {
        if tile.liquid < 200 {
            if !tile.half_brick() || world.tile(x, y - 1).map_or(true, |t| t.liquid < 200) {
                l = p.sky;
            }
        } else if p.shimmer_alpha > 0.0 {
            // `Main.liquidAlpha[13]` — shimmer brightens the blue channel.
            l[2] = p.sky[2] * 0.175 * (1.0 + rng.next_float() * 0.13) * p.shimmer_alpha;
        }
    }

    if (!tile.active() || tile.half_brick() || !crate::tiledata::TILE_NO_SUN_LIGHT.has(tile.kind))
        && ((88..=93).contains(&tile.wall) || tile.wall == 241)
        && tile.liquid < 255
    {
        let mut gem = p.sky;
        let which = if tile.wall == 241 {
            6
        } else {
            tile.wall as i32 - 88
        };
        match which {
            0 => gem = [gem[0] * 0.9, gem[1] * 0.15, gem[2] * 0.9],
            1 => gem = [gem[0] * 0.9, gem[1] * 0.9, gem[2] * 0.15],
            2 => gem = [gem[0] * 0.15, gem[1] * 0.15, gem[2] * 0.9],
            3 => gem = [gem[0] * 0.15, gem[1] * 0.9, gem[2] * 0.15],
            4 => gem = [gem[0] * 0.9, gem[1] * 0.15, gem[2] * 0.15],
            5 => {
                let n = 0.2;
                let m = 0.7 - n;
                gem = [
                    gem[0] * (m + p.disco[0] as f32 / 255.0 * n),
                    gem[1] * (m + p.disco[1] as f32 / 255.0 * n),
                    gem[2] * (m + p.disco[2] as f32 / 255.0 * n),
                ];
            }
            6 => gem = [gem[0] * 0.9, gem[1] * 0.5, 0.0],
            _ => {}
        }
        l = gem;
    }

    // `float num11 = 1f - Main.shimmerDarken`.
    let dark = 1.0 - p.shimmer_darken;
    for v in l.iter_mut() {
        *v *= dark;
    }
    for i in 0..3 {
        if c[i] < l[i] {
            c[i] = l[i];
        }
    }
}

/// `TileLightScanner.ApplyHellLight` (`TileLightScanner.cs:3240`).
pub fn apply_hell_light(
    world: &World,
    tile: &Tile,
    x: i32,
    y: i32,
    p: &LightParams,
    c: &mut [f32; 3],
) {
    let glow = p.hell_glow;
    let mut l = [0.0f32; 3];
    let wall_light = wall_is_light(tile.wall);
    let open_sides = |dx: i32, dy: i32| world.tile(x + dx, y + dy).map_or(true, |t| t.liquid == 0);
    let transmits = !tile.active()
        || !crate::tiledata::TILE_NO_SUN_LIGHT.has(tile.kind)
        || ((tile.slope() != 0 || tile.half_brick())
            && open_sides(-1, 0)
            && open_sides(1, 0)
            && open_sides(0, -1)
            && open_sides(0, 1));
    if transmits
        && c[0] < glow
        && wall_light
        && tile.liquid < 200
        && (!tile.half_brick() || world.tile(x, y - 1).map_or(true, |t| t.liquid < 200))
    {
        l = [glow, glow * 0.6, glow * 0.2];
    }

    if (!tile.active() || tile.half_brick() || !crate::tiledata::TILE_NO_SUN_LIGHT.has(tile.kind))
        && ((88..=93).contains(&tile.wall) || tile.wall == 241)
        && tile.liquid < 255
    {
        l = [glow, glow * 0.6, glow * 0.2];
        let which = if tile.wall == 241 {
            6
        } else {
            tile.wall as i32 - 88
        };
        match which {
            0 => l = [l[0] * 0.9, l[1] * 0.15, l[2] * 0.9],
            1 => l = [l[0] * 0.9, l[1] * 0.9, l[2] * 0.15],
            2 => l = [l[0] * 0.15, l[1] * 0.15, l[2] * 0.9],
            3 => l = [l[0] * 0.15, l[1] * 0.9, l[2] * 0.15],
            4 => l = [l[0] * 0.9, l[1] * 0.15, l[2] * 0.15],
            5 => {
                let n = 0.2;
                let m = 0.7 - n;
                l = [
                    l[0] * (m + p.disco[0] as f32 / 255.0 * n),
                    l[1] * (m + p.disco[1] as f32 / 255.0 * n),
                    l[2] * (m + p.disco[2] as f32 / 255.0 * n),
                ];
            }
            6 => l = [l[0] * 0.9, l[1] * 0.5, 0.0],
            _ => {}
        }
    }
    for i in 0..3 {
        if c[i] < l[i] {
            c[i] = l[i];
        }
    }
}

/// `Main.wallLight[wall] || wall == 73 || wall == 227`
/// (`TileLightScanner.cs:3142`/`:3149`/`:3246`).
pub fn wall_is_light(wall: u16) -> bool {
    (wall as usize) < crate::tiledata::WALL_COUNT && crate::tiledata::WALL_LIGHT[wall as usize]
        || wall == 73
        || wall == 227
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tile::FLAG_ACTIVE;

    fn tile(kind: u16, fx: i16, fy: i16) -> Tile {
        let mut t = Tile::default();
        t.kind = kind;
        t.frame_x = fx;
        t.frame_y = fy;
        t.flags |= FLAG_ACTIVE;
        t
    }

    #[test]
    fn shimmer_and_crystal_walls_flicker_with_mouse_text_color() {
        let mut p = LightParams::default();
        p.mouse_text_color = 180; // (270 - 180) / 900 = 0.1
        let mut c = [0.0f32; 3];
        let mut rng = FastRandom::new(0).with_modifier(1, 1);
        let t = tile(0, 0, 0);
        apply_wall_light(&t, 0, &mut rng, &p, &mut c);
        assert_eq!(c, [0.0; 3]);

        let mut wall = Tile::default();
        wall.wall = 137;
        let mut c = [0.0f32; 3];
        apply_wall_light(&wall, 0, &mut rng, &p, &mut c);
        // `0.4 + delta * 0.6 + random * 0.0005`, times (1, 0.5, 0.1).
        assert!((c[0] - 0.46).abs() < 0.03, "crystal wall {c:?}");
        assert!((c[1] - c[0] * 0.5).abs() < 1e-4, "crystal wall {c:?}");
    }
}
