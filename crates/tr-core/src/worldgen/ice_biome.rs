//! The `Generate Ice Biome` pass (`WorldGen.cs:12312-12422`, W4-d101 pass
//! 8): the snow column walk that retypes the terrain into the snow biome.
//! The Reset-side origin draws (`:11378-11414`) live in
//! `pipeline.rs::reset_genvars`; this module walks the rows.

use super::noise::Rng;
use super::pipeline::GenVars;
use super::terrain::Layers;
use super::{TILE_ICE, TILE_SNOW};
use crate::seeds::SeedFlags;
use crate::world::World;

/// The retype switch (`WorldGen.cs:12361-12373`, `:12400-12412`): dirt 0 /
/// grass 2 / corrupt grass 23 / clay 40 / sand 53 → snow 147, stone 1 →
/// ice 161, and a dirt wall 2 → snow wall 40. The C# reads `.type`/`.wall`
/// WITHOUT an active check — an inactive cell's stored kind retypes too;
/// kept verbatim (the port's `Tile.kind` persists across the active flag).
fn retype(world: &mut World, x: i32, y: i32) {
    let Some(t) = world.tile_mut(x, y) else {
        return; // the C# would throw out of bounds; real worlds stay inside
    };
    if t.wall == 2 {
        t.wall = 40;
    }
    match t.kind {
        0 | 2 | 23 | 40 | 53 => t.kind = TILE_SNOW,
        1 => t.kind = TILE_ICE,
        _ => {}
    }
}

/// The pass body (`WorldGen.cs:12312-12422`). `snowTop = worldSurface`;
/// the solid retype runs to `num = lavaLine - Next(160, 200)`, the fringe
/// drip (the `num5` random-depth walk) below it down to
/// `num2 = lavaLine` (remix: `maxTilesY - 250`, a SECOND `Next(160, 200)`
/// draw — the first always fires, `:12318` vs `:12323`). Per row: the two
/// drift draws, the i>0 averaging with the previous row's stored bounds
/// (C# int division truncates toward zero — Rust's `/` matches), the
/// dungeon-side bias `Next(4)` (the port pins dungeon Right, so always the
/// `++` arm — the fixed-side convention of the beach anchors), then the
/// per-column retype. Publishes `snowMinX/snowMaxX/snowBottom` for d102's
/// Slush sweep (`:14697`).
pub(crate) fn pass_ice_biome(world: &mut World, gv: &mut GenVars, rng: &mut Rng, layers: &Layers) {
    let height = world.height;
    let remix = world.seeds.contains(SeedFlags::REMIX);
    gv.snow_top = layers.world_surface as i32; // `:12317`
    // The C# allocates the row arrays at maxTilesY in Reset
    // (`WorldGen.cs:11178-11179`); the port sizes them here when unset.
    if gv.snow_min_x.len() != height as usize {
        gv.snow_min_x = vec![0; height as usize];
        gv.snow_max_x = vec![0; height as usize];
    }
    let mut num = gv.lava_line - rng.range(160, 200); // `:12318`
    let mut num2 = gv.lava_line;
    if remix {
        num2 = height - 250;
        num = num2 - rng.range(160, 200); // `:12323` — the second draw
    }
    let mut num3 = gv.snow_origin_left;
    let mut num4 = gv.snow_origin_right;
    let mut num5 = 10;
    for i in 0..=num2 - 140 {
        num3 += rng.range(-4, 4);
        num4 += rng.range(-3, 5);
        if i > 0 {
            num3 = (num3 + gv.snow_min_x[(i - 1) as usize]) / 2;
            num4 = (num4 + gv.snow_max_x[(i - 1) as usize]) / 2;
        }
        // The dungeon-side bias (`:12338-12350`): the port's dungeon sits
        // Right (the fixed-side convention), so the `>= Right` arm always
        // runs — one `Next(4)` per row, a hit nudges both bounds right.
        if rng.range(0, 4) == 0 {
            num3 += 1;
            num4 += 1;
        }
        gv.snow_min_x[i as usize] = num3;
        gv.snow_max_x[i as usize] = num4;
        for j in num3..num4 {
            if i < num {
                retype(world, j, i);
            } else {
                // The fringe drip (`:12377-12393`): the depth walk with
                // its two nested 1-in-3 widenings and the 0..50 clamps.
                num5 += rng.range(-3, 4);
                if rng.range(0, 3) == 0 {
                    num5 += rng.range(-4, 5);
                    if rng.range(0, 3) == 0 {
                        num5 += rng.range(-6, 7);
                    }
                }
                if num5 < 0 {
                    num5 = rng.range(0, 3);
                } else if num5 > 50 {
                    num5 = 50 - rng.range(0, 3);
                }
                for k in i..i + num5 {
                    retype(world, j, k);
                }
            }
        }
        if gv.snow_bottom < i {
            gv.snow_bottom = i; // `:12416-12419`
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tile::FLAG_ACTIVE;
    use crate::worldgen::{TILE_DIRT, TILE_STONE};

    fn canvas() -> (World, GenVars, Layers) {
        let (w, h) = (600, 700);
        let mut world = World::new("ice".into(), w, h);
        for x in 0..w {
            for y in 100..h {
                let i = world.index(x, y);
                world.tiles[i].kind = if y < 120 { TILE_DIRT } else { TILE_STONE };
                world.tiles[i].flags |= FLAG_ACTIVE;
                world.tiles[i].wall = 2;
            }
        }
        let mut gv = GenVars::default();
        gv.snow_origin_left = 250;
        gv.snow_origin_right = 350;
        gv.lava_line = 600;
        let layers = Layers {
            avg_surface: 100.0,
            world_surface: 105.0,
            rock_layer: 160.0,
            hell_top: h - 60,
            surface_low: 100.0,
            surface_high: 120.0,
            rock_low: 125.0,
            rock_high: 145.0,
        };
        (world, gv, layers)
    }

    /// The walk retypes the band into snow/ice, converts the dirt walls,
    /// and publishes the row bounds + snowTop/snowBottom
    /// (`WorldGen.cs:12317`, `:12351-12352`, `:12416-12419`).
    #[test]
    fn the_ice_walk_retypes_and_publishes_the_snow_bounds() {
        let (mut world, mut gv, layers) = canvas();
        pass_ice_biome(&mut world, &mut gv, &mut Rng(0x1CE), &layers);
        assert_eq!(gv.snow_top, 105);
        assert_eq!(gv.snow_bottom, 600 - 140);
        assert_eq!(gv.snow_min_x.len(), 700);
        // Snow and ice exist inside the walked band; the walls converted.
        let snow = world.tiles.iter().filter(|t| t.kind == TILE_SNOW).count();
        let ice = world.tiles.iter().filter(|t| t.kind == TILE_ICE).count();
        assert!(snow > 1000, "snow cells: {snow}");
        assert!(ice > 1000, "ice cells: {ice}");
        assert!(
            world.tiles.iter().any(|t| t.kind == TILE_SNOW && t.wall == 40),
            "the dirt wall converts to snow wall 40"
        );
        // Outside the band's reach nothing retyped (column 0 is far from
        // the 250..350 origin; the drift averages ±2/row toward it).
        assert!(
            (0..700).all(|y| world.tile(0, y).is_some_and(|t| t.kind != TILE_SNOW)),
            "the walk leaked to the far edge"
        );
    }

    /// The pass is deterministic for a seed (the a == b contract).
    #[test]
    fn the_ice_walk_is_deterministic_for_a_seed() {
        let run = || {
            let (mut world, mut gv, layers) = canvas();
            pass_ice_biome(&mut world, &mut gv, &mut Rng(0x1CE), &layers);
            (world.tiles, gv.snow_min_x, gv.snow_max_x, gv.snow_bottom)
        };
        assert_eq!(run(), run());
    }
}
