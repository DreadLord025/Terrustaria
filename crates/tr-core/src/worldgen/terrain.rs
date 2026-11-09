//! The surface height map and the terrain fill.
//!
//! [`surface_height_map`] shapes the per-column surface row (hills, the spawn
//! plateau, desert dunes, the flattened lake span and the ocean beach ramps);
//! [`fill_terrain`] paints the dirt/sand/stone/ash columns under it and derives
//! the `world_surface` / `rock_layer` anchors from `TerrainPass.ApplyPass`.
//!
//! M5-SEED-2: [`fill_terrain`] carries the `surfaceIsInSpace` terrain
//! parameters (see the `space_*` bindings there for the C# band mapping).

use super::noise::Noise;
use super::{
    TILE_ASH, TILE_DIRT, TILE_GRASS, TILE_HARDENED_SAND, TILE_ICE, TILE_SAND, TILE_SANDSTONE,
    TILE_SNOW, TILE_STONE, WALL_DIRT, WALL_SANDSTONE, WALL_STONE,
};
use crate::seeds::MicroSeedFlags;
use crate::tile::{Tile, FLAG_ACTIVE};
use crate::world::World;

/// Everything the surface-height pass computes that the later passes consume.
#[derive(Clone)]
pub(crate) struct SurfaceMap {
    /// Per-column surface row; the lake pass lowers its columns to the carved floor.
    pub(crate) heights: Vec<i32>,
    pub(crate) spawn_center: i32,
    pub(crate) desert_enabled: bool,
    pub(crate) desert_start: i32,
    pub(crate) desert_end: i32,
    pub(crate) lake_center: i32,
    pub(crate) lake_half: i32,
    pub(crate) lake_level: i32,
    /// The vanilla beach GenVars (`GenVars.cs:90-92`): the inland end of the
    /// left beach and the inland start of the right one, drawn in `Reset`
    /// (`WorldGen.cs:11415-11440`) — the port draws them at the head of the
    /// terrain step (the first consumer) and every beach consumer reads them
    /// from here.
    pub(crate) left_beach_end: i32,
    pub(crate) right_beach_start: i32,
    pub(crate) sea_level: i32,
}

/// Layer anchors derived from the finished surface: `world_surface`/`rock_layer`
/// land on the world, the rest scopes the cave, ore and lava passes.
#[derive(Clone, Copy)]
pub(crate) struct Layers {
    pub(crate) avg_surface: f64,
    pub(crate) world_surface: f64,
    pub(crate) rock_layer: f64,
    pub(crate) hell_top: i32,
    /// The `GenVars` surface/rock quartet the ore bands key on
    /// (`TerrainPass.cs:230-234`): min/max of the per-column surface and of the
    /// per-column rock start, both raw.
    ///
    /// The C# ≥ 20 recentering (`TerrainPass.cs:217-228`) is deliberately not
    /// applied: it guards a degenerate case that needs the C# rock line (6-35 %
    /// of the world height below the surface) — the model's rock line sits only
    /// `dirt_depth` (16-22) below the surface, so the guard would fire on every
    /// world and drag `surface_high` below the mean surface, shifting the
    /// surface ore bands underground. A normal C# world never recenters, so the
    /// raw quartet is what reproduces the normal-world band geometry
    /// (ANALYSIS "OresAndShinies fidelity (M5-ORES-1)").
    pub(crate) surface_low: f64,
    pub(crate) surface_high: f64,
    pub(crate) rock_low: f64,
    pub(crate) rock_high: f64,
}

/// Shapes the per-column surface row before any tile is painted. The beach
/// ends come from the Reset draws (`WorldGen.cs:11415-11440`, via the terrain
/// step's GenVars).
pub(crate) fn surface_height_map(
    width: i32,
    height: i32,
    noise: &Noise,
    left_beach_end: i32,
    right_beach_start: i32,
) -> SurfaceMap {
    // --- surface height map -------------------------------------------------
    let avg = height as f32 * 0.30;
    let spawn_center = width / 2;
    // An inland desert west of spawn (small worlds skip it). The band is kept clear
    // of the spawn plateau and the lake.
    let desert_half = 170;
    let desert_center = spawn_center - 380;
    let desert_enabled = width >= 2000;
    let desert_start = if desert_enabled {
        (desert_center - desert_half).max(200)
    } else {
        0
    };
    let desert_end = if desert_enabled {
        (desert_center + desert_half).min(spawn_center - 120)
    } else {
        0
    };
    let mut surface = vec![0i32; width as usize];
    for x in 0..width {
        let fx = x as f32;
        // Large rolling hills plus small roughness.
        let hills = noise.fbm(fx * 0.0028, 11.0, 4, 0.5);
        let detail = noise.fbm(fx * 0.018, 71.0, 3, 0.5);
        let mut h = avg + hills * 42.0 + detail * 5.0;
        // Gentle spawn plateau so the player starts on flat ground.
        let dist = (x - spawn_center).abs() as f32;
        if dist < 45.0 {
            let t = 1.0 - dist / 45.0;
            h = h * (1.0 - t * 0.7) + avg * t * 0.7;
        }
        surface[x as usize] = (h.round() as i32).clamp(80, height - 340);
    }
    // Flatten the desert into smooth dunes so it reads differently from the forest.
    if desert_enabled {
        for x in (desert_start - 30)..(desert_end + 30) {
            if x < 0 || x >= width {
                continue;
            }
            let d = (x - desert_center).abs();
            let blend = if d <= desert_half {
                1.0
            } else {
                1.0 - (d - desert_half) as f32 / 30.0
            };
            let dune =
                avg + (x as f32 * 0.045).sin() * 5.0 + noise.value(x as f32 * 0.02, 9.0) * 4.0;
            surface[x as usize] = (surface[x as usize] as f32 * (1.0 - blend)
                + dune.clamp(80.0, (height - 340) as f32) * blend)
                .round() as i32;
        }
    }
    // Limit adjacent height differences to one tile so the whole surface can be
    // expressed with slope tiles instead of vertical stair-steps.
    for _ in 0..3 {
        for x in 1..width as usize {
            let prev = surface[x - 1];
            surface[x] = surface[x].clamp(prev - 1, prev + 1);
        }
        for x in (0..width as usize - 1).rev() {
            let next = surface[x + 1];
            surface[x] = surface[x].clamp(next - 1, next + 1);
        }
    }

    // A small forest lake near spawn, one tile below the local ground (like
    // `WorldGen.SonOfLakinater`): the span is flattened to the water line here and the
    // bowl itself is carved into the finished terrain by the `--- liquids ---` pass, so
    // caves/ore passes cannot punch through the floor.
    let lake_center = (spawn_center + 150).min(width - 80);
    let lake_half = 32;
    let lake_level = (surface[lake_center as usize] + 1).clamp(90, height - 300);
    // The two rim columns are flattened too, so the shore grass sits exactly at the
    // water line and no backdrop shows through a step.
    for dx in -(lake_half + 2)..=(lake_half + 2) {
        let x = lake_center + dx;
        if x < 2 || x >= width - 2 {
            continue;
        }
        surface[x as usize] = lake_level;
    }

    // Ocean beaches at both edges: the vanilla flat shelf. `TerrainPass`
    // clamps the near-beach surface flat and featureless (`TerrainPass.cs:
    // 131-134`'s beach band, `:84`'s `num2 = leftBeachEnd + FlatBeachPadding`
    // feature suppression) and `Beaches` later carves the ocean prism below
    // it (`WorldGen.cs:14888+`), anchoring the water line to the live surface.
    // The model keeps its own interior curve, so the shelf is the model's sea
    // level held flat over the beach width and blended out over 30 columns
    // (deviation: compact interior, vanilla shelf structure — ANALYSIS
    // "Ocean passes fidelity (M5-OCEAN)"); the old t²·30 ramp dip is gone,
    // the seafloor profile is `Beaches`' `TuneOceanDepth` curve from P2 on.
    let sea_level = (avg as i32 + 4).clamp(90, height - 300);
    let right_beach_end = width - right_beach_start;
    let shelf = |d: i32, beach_end: i32| -> f32 {
        if d < beach_end {
            1.0
        } else {
            (((beach_end + 30) - d) as f32 / 30.0).clamp(0.0, 1.0)
        }
    };
    for x in 0..(left_beach_end + 30).min(width) {
        let k = shelf(x, left_beach_end);
        surface[x as usize] = (surface[x as usize] as f32 * (1.0 - k) + sea_level as f32 * k)
            .round() as i32;
    }
    for x in (width - (right_beach_end + 30)).max(0)..width {
        let d = width - 1 - x;
        let k = shelf(d, right_beach_end);
        surface[x as usize] = (surface[x as usize] as f32 * (1.0 - k) + sea_level as f32 * k)
            .round() as i32;
    }

    SurfaceMap {
        heights: surface,
        spawn_center,
        desert_enabled,
        desert_start,
        desert_end,
        lake_center,
        lake_half,
        lake_level,
        left_beach_end,
        right_beach_start,
        sea_level,
    }
}

/// Paints the terrain columns under the surface row and derives the layer anchors.
pub(crate) fn fill_terrain(
    world: &mut World,
    width: i32,
    height: i32,
    noise: &Noise,
    map: &SurfaceMap,
) -> Layers {
    // --- terrain fill -------------------------------------------------------
    let surface = &map.heights;
    let left_beach_end = map.left_beach_end;
    let _right_beach_start = map.right_beach_start;
    let desert_enabled = map.desert_enabled;
    let desert_start = map.desert_start;
    let desert_end = map.desert_end;
    let in_desert = |x: i32| desert_enabled && x >= desert_start && x < desert_end;
    // Mean surface height, used as the anchor for cave/ore placement below.
    let avg_surface = surface.iter().sum::<i32>() as f64 / width as f64;
    // --- M5-SEED-2 `surfaceIsInSpace` (TerrainPass.cs:116-119, :187-191) ----
    //
    // The C# shapes the surface row `num3` with the clamp band
    // `[maxTilesY * num10, maxTilesY * num11]` — `num10 = 0.17`,
    // `num11 = 0.26` — and the seed takes two arms:
    //
    //   double num11 = 0.26;
    //   if (WorldGen.SecretSeed.surfaceIsInSpace.Enabled) num11 = 0.2199999988079071;
    //   ...                                              // :116-119
    //   else if (num3 > (double)Main.maxTilesY * num11) { num3 = ...; num2 = 0; }
    //   ...
    //   if (surfaceIsInSpace.Enabled && !noSurface.Enabled)
    //       TerrainPass.FillColumn(i, num3 - (double)Main.maxTilesY * 0.08, num4);
    //   else
    //       TerrainPass.FillColumn(i, num3, num4);      // :187-191
    //
    // so the surface row is allowed 0.04 * height higher and the solid column
    // starts 0.08 * height above that row. `DoSurfaceIsInSpace`
    // (`WorldGen.cs:2069-2085`, the cloud-wall wipe above `Main.worldSurface`)
    // lives in `micro_surface.rs` — this is the terrain half only.
    //
    // Mapping onto the port's compact parameters: the height map's band is
    // `.clamp(80, height - 340)` (`surface_height_map`), so `height - 340` is
    // the model's `num11 * maxTilesY` analogue (a flat offset that keeps the
    // underground deep enough for the compact cave/ore bands, NOT a fraction of
    // the height), and the C#'s 0.26 → 0.22 delta — 0.04 of the world height —
    // is subtracted from it below. Because the model's surface never reaches
    // that ceiling on any supported size, the ceiling arm does not bind in
    // practice: the load-bearing arm is the 8 % headroom, which raises the
    // terrain top to `heights[x] - 0.08 * height` (0.22 * height on a default
    // 1200-tall world, whose mean surface is 0.30 * height). Recorded as a
    // compact-model deviation (ANALYSIS).
    let space = world.micro_seeds.contains(MicroSeedFlags::SURFACE_IS_IN_SPACE);
    let space_headroom = if space {
        (height as f64 * 0.08) as i32
    } else {
        0
    };
    let space_band = if space {
        (height - 340 - (height as f64 * 0.04) as i32).max(8)
    } else {
        height - 340
    };
    // `TerrainPass.ApplyPass`: `Main.worldSurface = (int)(max(surfaceY) + 25.0)` — the
    // *deepest* column of the pre-carve surface, 25 tiles below it, not the mean. The
    // underground backdrop (`DrawBackground_DirtBackground`) and the surface parallax
    // (`DrawSurfaceBG`) are both anchored to this line, so it has to track the lowest
    // terrain or the dirt sheet ends up floating above the ground.
    let surface_max = surface.iter().max().copied().unwrap_or(0) as f64;
    let world_surface = surface_max + 25.0;
    // `TerrainPass.ApplyPass`: `Main.rockLayer = worldSurface + (int)((num8 - worldSurface) / 6) * 6`
    // with `num8` the deepest per-column rock start.
    let rock_raw = (height as f64 * 0.43).max(world_surface + 65.0);
    let rock_layer = world_surface + ((rock_raw - world_surface) / 6.0).floor() * 6.0;
    let hell_top = (height - 170) as i32;
    // The `GenVars` quartet (`TerrainPass.cs:73-76,88-91`): min/max of the
    // per-column surface (num5/num6) and of the per-column rock start (num7/num8).
    // The model's per-column rock start is the dirt→stone transition `top +
    // dirt_depth` (the C# walks a smoother num4 line — ANALYSIS "OresAndShinies
    // fidelity").
    let surface_low = surface.iter().min().copied().unwrap_or(0) as f64;
    let mut rock_low = f64::INFINITY;
    let mut rock_high = f64::NEG_INFINITY;

    // Eastern third of the world is a snow biome.
    let snow_start = width * 2 / 3;

    for x in 0..width {
        // `recorded` is the height-map row — the port's analogue of the C#'s
        // `num3` (the surface row the anchors and the layer bands derive
        // from). `top` is the fill's first solid row: `FillColumn`'s first
        // argument, which `surfaceIsInSpace` raises by the 8 % headroom
        // (`TerrainPass.cs:187-191`). A classic world has
        // `headroom == 0` and a non-binding band, so `top == recorded` and the
        // column is painted exactly as before.
        let recorded = surface[x as usize];
        let top = (recorded - space_headroom).min(space_band).max(2);
        let dirt_depth = 16 + (noise.value(x as f32 * 0.05, 3.0) * 6.0) as i32;
        // The dirt → stone transition stays on the anchor row, as the C#'s
        // rock line `num4` tracks `num3` and not the shifted fill top (the
        // vanilla dirt sheet under a `surfaceIsInSpace` world is therefore
        // `0.06 * height + dirt_depth` thick, and the port's stays
        // `dirt_depth + headroom`).
        let rock_line = (recorded + dirt_depth) as f64;
        rock_low = rock_low.min(rock_line);
        rock_high = rock_high.max(rock_line);
        for y in 0..height {
            let i = world.index(x, y);
            let mut tile = Tile::default();
            let desert = in_desert(x);
            if y >= top {
                if y == top {
                    // The sand-top rule inside the beach bands retired with
                    // M5-OCEAN phase 2: the faithful `Ocean Sand` pass
                    // (`WorldGen.cs:11580`) sandifies the tops itself (and
                    // the `Beaches` carve re-shapes the whole prism).
                    tile.kind = if x >= snow_start {
                        TILE_SNOW
                    } else if desert {
                        TILE_SAND
                    } else {
                        TILE_GRASS
                    };
                } else if x >= snow_start && y < rock_layer as i32 {
                    // The `IceBiome` pass (`WorldGen`): the snow band's
                    // underground is the ice shell down to the rock layer.
                    tile.kind = TILE_ICE;
                } else if y < recorded + dirt_depth {
                    tile.kind = if desert { TILE_SAND } else { TILE_DIRT };
                } else if desert && y < recorded + dirt_depth + 14 {
                    tile.kind = TILE_HARDENED_SAND;
                } else if desert && y < recorded + dirt_depth + 30 {
                    tile.kind = TILE_SANDSTONE;
                } else if y < hell_top {
                    tile.kind = TILE_STONE;
                } else {
                    tile.kind = TILE_ASH;
                }
                tile.flags |= FLAG_ACTIVE;
                if y > top + 2 {
                    tile.wall = if desert {
                        WALL_SANDSTONE
                    } else if (y as f64) < rock_layer {
                        WALL_DIRT
                    } else {
                        WALL_STONE
                    };
                }
            }
            world.tiles[i] = tile;
        }
    }

    Layers {
        avg_surface,
        world_surface,
        rock_layer,
        hell_top,
        surface_low,
        surface_high: surface_max,
        rock_low,
        rock_high,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// DW-2.1: `fill_terrain` fills the quartet from the finished columns — the
    /// raw min/max relations hold and every ore band's y-range stays non-empty
    /// (the C#-normal-world geometry, no recentering).
    #[test]
    fn fill_terrain_records_the_layer_quartet() {
        let mut world = World::new("test".into(), 400, 800);
        let noise = Noise { seed: 42 };
        let map = surface_height_map(400, 800, &noise, 140, 800 - 140);
        let layers = fill_terrain(&mut world, 400, 800, &noise, &map);

        let raw_max = map.heights.iter().copied().max().unwrap_or(0) as f64;
        let raw_min = map.heights.iter().copied().min().unwrap_or(0) as f64;
        assert_eq!(layers.surface_high, raw_max, "surface_high = max surface");
        assert_eq!(layers.surface_low, raw_min, "surface_low = min surface");
        assert!(layers.surface_low <= layers.avg_surface);
        assert!(layers.avg_surface <= layers.surface_high);
        // rock_low/rock_high are the raw min/max of the model's rock line
        // `top + dirt_depth` (dirt_depth ∈ 16..=22).
        let rock_line = |x: i32| {
            let d = 16 + (noise.value(x as f32 * 0.05, 3.0) * 6.0) as i32;
            (map.heights[x as usize] + d) as f64
        };
        let (mut lo, mut hi) = (f64::INFINITY, f64::NEG_INFINITY);
        for x in 0..400 {
            lo = lo.min(rock_line(x));
            hi = hi.max(rock_line(x));
        }
        assert_eq!(layers.rock_low, lo);
        assert_eq!(layers.rock_high, hi);
        // No recentering: surface_high keeps its raw value even though the
        // model's rock line sits above `surface_high + 20` on a shallow line.
        // Band geometry stays C#-normal-world: every band range is non-empty.
        assert!(layers.surface_high < layers.rock_high, "band 2/5/7");
        assert!(layers.rock_low < 800.0, "bands 3/6/8/9");
        assert!(layers.surface_low > 20.0, "band 11");
        assert_eq!(layers.world_surface, raw_max + 25.0);
        assert_eq!(layers.hell_top, 630);
    }

    /// M5-SEED-2 `surfaceIsInSpace` (`TerrainPass.cs:187-191`): the fill gains
    /// 8 % of the world height of headroom above the surface row — the terrain
    /// top rises, the layers stay on the anchor row, and the classic fill is
    /// unchanged.
    #[test]
    fn surface_is_in_space_raises_the_fill_by_eight_percent_of_the_height() {
        let (width, height) = (400, 800);
        let noise = Noise { seed: 42 };
        let mut classic = World::new("t".into(), width, height);
        classic.micro_seeds = MicroSeedFlags::none();
        let map = surface_height_map(width, height, &noise, 140, width - 140);
        let layers = fill_terrain(&mut classic, width, height, &noise, &map);
        // The classic column starts exactly at the height-map row.
        let top = map.heights[200];
        let ground_classic = classic
            .tile(200, top)
            .is_some_and(|t| t.active() && t.kind == TILE_GRASS);

        let mut space = World::new("t".into(), width, height);
        space.micro_seeds = crate::seeds::MicroSeedFlags::from(MicroSeedFlags::SURFACE_IS_IN_SPACE);
        let layers_space = fill_terrain(&mut space, width, height, &noise, &map);
        let headroom = (height as f64 * 0.08) as i32;
        assert_eq!(headroom, 64, "8 % of a 800-tall world");
        // The grass row now sits a full headroom above the anchor.
        assert!(ground_classic, "the classic column starts at the anchor");
        assert!(
            space
                .tile(200, top - headroom)
                .is_some_and(|t| t.active() && t.kind == TILE_GRASS),
            "the flagged column's grass row is a headroom higher"
        );
        assert!(
            space.tile(200, top).is_some_and(|t| t.active()),
            "the rows between the two are solid fill"
        );
        assert!(
            space.tile(200, top - headroom - 1).is_some_and(|t| !t.active()),
            "nothing above the new top row"
        );
        // The layer anchors stay on the height-map row (the C#'s `num5`/`num6`
        // read `num3`, not the shifted fill top).
        assert_eq!(layers_space.world_surface, layers.world_surface);
        assert_eq!(layers_space.rock_high, layers.rock_high);
        // The band ceiling arm does not bind here: the model's surface never
        // approaches `height - 340`.
        assert!(top < height - 340 - (height as f64 * 0.04) as i32);
    }

    /// The classic fill is byte-identical: the same map over two worlds (one
    /// with no micro flags, one with an unrelated bit set) paints the same
    /// tiles.
    #[test]
    fn the_classic_fill_ignores_the_unrelated_micro_flags() {
        let (width, height) = (400, 800);
        let noise = Noise { seed: 42 };
        let map = surface_height_map(width, height, &noise, 140, width - 140);
        let mut a = World::new("t".into(), width, height);
        let mut b = World::new("t".into(), width, height);
        b.micro_seeds = MicroSeedFlags::from(MicroSeedFlags::DIG_EXTRA_HOLES);
        let la = fill_terrain(&mut a, width, height, &noise, &map);
        let lb = fill_terrain(&mut b, width, height, &noise, &map);
        assert_eq!(a.tiles, b.tiles);
        assert_eq!(la.world_surface, lb.world_surface);
        assert_eq!(la.rock_layer, lb.rock_layer);
    }
}
