//! The `Dunes` pass (`DunesAndPyramidLocations`, `WorldGen.cs:11517-11579`)
//! with `DunesBiome` (`GameContent/Biomes/DunesBiome.cs`) — the surface dune
//! carver and the pyramid-slot anchors, ported under the golden rule.
//!
//! Call graph (the C# `genRand` is the port's per-pass [`Rng`]; `GenBase._random`
//! IS `WorldGen.genRand` — `GenBase.cs:8` — so every roll below comes off the
//! one stream, in the order the C# makes them):
//!
//! ```text
//! Dunes (WorldGen.cs:11517) — Count groups, each two dune carvers
//!   ├── DunesBiome.Place (:53-64)            — two heights + two width rolls
//!   │     └── PlaceSingle (:66-83)           — ridge hills + centre hills
//!   │           └── PlaceHill (:85-104)      — the 3-point quadratic Bézier
//!   │                 └── PlaceCurvedLine (:106-138) — the crest + sand fill
//!   └── the pyramid slot (:11564-11579)      — the column's first active row
//! ```
//!
//! Deviation: the port's per-pass `pass_rng` stream replaces the C# global
//! `genRand` (the documented re-seed model, `WorldGenerator.cs:497`); the
//! special-seed arms (`drunkWorldGen`'s forced pyramid chance,
//! `SecretSeed.noSurface`/`dualDungeons`) have no port model and are skipped.

use crate::tile::{Tile, FLAG_ACTIVE};
use crate::world::World;
use super::desert::Rect;
use super::noise::Rng;
use super::pipeline::GenVars;

/// `TileID.Sets.Clouds` (`TileID.cs:505`) — the surface scan skips these.
const CLOUD_KINDS: [u16; 6] = [189, 196, 460, 717, 718, 719];
/// `TileID.Sand` (`TileID.cs:735`) — the dune fill.
const TILE_SAND: u16 = 53;
/// The surface scan's start row (`SurfaceMap.cs:51`).
const SCAN_TOP: i32 = 50;

/// `SurfaceMap` (`Desert/SurfaceMap.cs:6-74`): the dune carver's own surface
/// line — the first non-cloud active tile per column.
#[derive(Clone)]
pub(crate) struct DuneSurface {
    heights: Vec<i32>,
    x: i32,
}

impl DuneSurface {
    /// `SurfaceMap.FromArea` (`SurfaceMap.cs:44-74`): per column, the first
    /// non-cloud active tile scanning y = 50 … 50 + maxTilesY/2; a column with
    /// no solid ends at `maxTilesY/2 + 50`.
    pub(crate) fn from_area(world: &World, start_x: i32, width: i32) -> Self {
        let scan = world.height / 2;
        let mut heights = Vec::with_capacity(width.max(0) as usize);
        for i in start_x..start_x + width {
            let mut found = false;
            let mut num2 = 0i32;
            let mut j = SCAN_TOP;
            while j < SCAN_TOP + scan {
                if let Some(t) = world.tile(i, j) {
                    if t.active() {
                        if CLOUD_KINDS.contains(&t.kind) {
                            found = false;
                        } else if !found {
                            num2 = j;
                            found = true;
                        }
                    }
                }
                if !found {
                    num2 = scan + SCAN_TOP;
                }
                j += 1;
            }
            heights.push(num2);
        }
        Self {
            heights,
            x: start_x,
        }
    }

    /// `this[absoluteX]` (`SurfaceMap.cs:19`).
    pub(crate) fn height(&self, x: i32) -> i32 {
        let idx = (x - self.x) as usize;
        if idx < self.heights.len() {
            self.heights[idx]
        } else {
            *self.heights.last().unwrap_or(&0)
        }
    }

    /// `Bottom` (`SurfaceMap.cs:34-38`): the deepest row, clamped to
    /// `Main.worldSurface - 10`.
    pub(crate) fn bottom(&self, world_surface: f64) -> i32 {
        let max = *self.heights.iter().max().unwrap_or(&0);
        max.min((world_surface - 10.0) as i32)
    }

    /// `Top` (`SurfaceMap.cs:39`): the shallowest row.
    pub(crate) fn top(&self) -> i32 {
        *self.heights.iter().min().unwrap_or(&0)
    }

    /// `Average` (`SurfaceMap.cs:40`).
    pub(crate) fn average(&self) -> f64 {
        let sum: i64 = self.heights.iter().map(|h| *h as i64).sum();
        sum as f64 / self.heights.len() as f64
    }
}

/// `DunesBiome.DunesDescription` (`DunesBiome.cs:12-37`): one dune's rect, its
/// surface line and its wind direction.
#[derive(Clone)]
struct DunesDescription {
    area: Rect,
    surface: DuneSurface,
    wind_left: bool,
}

impl DunesDescription {
    /// `CreateFromPlacement` (`:26-36`): the rect, the padded surface line and
    /// the `Next(2)` wind roll.
    fn create_from_placement(
        world: &World,
        origin_x: i32,
        origin_y: i32,
        width: i32,
        height: i32,
        rng: &mut Rng,
    ) -> Self {
        Self {
            area: Rect::new(origin_x - width / 2, origin_y - height / 2, width, height),
            surface: DuneSurface::from_area(world, origin_x - width / 2 - 20, width + 40),
            wind_left: rng.chance(2),
        }
    }
}

/// `DunesAndPyramidLocations` (`WorldGen.cs:11517-11579`): the pass body — the
/// `Count` dune groups over the map, each two carvers, plus the pyramid-slot
/// anchors (`GenVars.PyrX/PyrY/numPyr`, `:11570-11572`). Returns the anchors;
/// the driver appends them to `gv`.
pub(crate) fn dunes(
    world: &mut World,
    gv: &GenVars,
    snow_start: i32,
    world_surface: f64,
    count: i32,
    chance_of_pyramid: f64,
    rng: &mut Rng,
) -> Vec<(i32, i32)> {
    let mut anchors = Vec::new();
    let width = world.width;
    let height = world.height;
    for _ in 0..count {
        // `RandomWorldPoint(0, 500, 0, 500)` (`WorldGen.cs:27106-27113`) with
        // the jungle/centre/snow rejections (`:11539-11561`).
        let mut origin = (0i32, 0i32);
        let mut tries = 0;
        loop {
            // `RandomWorldPoint(0, 500, 0, 500)` (`WorldGen.cs:27106-27113`) —
            // the C# `Next(min, max)` degenerates to a value under `min` when
            // `max < min` (a sub-vanilla 512 test world); the port's `range`
            // requires `hi >= lo`, so the guard keeps the draw well-formed.
            origin = (
                rng.range(500, (width - 500).max(500)),
                rng.range(500, (height - 500).max(500)),
            );
            let near_jungle = (origin.0 - gv.jungle_origin_x).abs() < (600.0 * (width as f64 / 4200.0)) as i32;
            let near_center = (origin.0 - width / 2).abs() < 300;
            // The snow band (`terrain.rs:242`, the eastern third); the vanilla
            // `snowOriginLeft/Right` pair (`:11554`) has no port twin, so the
            // one-sided bound mirrors the compact band.
            let near_snow = origin.0 > snow_start - 300;
            tries += 1;
            let mut reject = near_jungle || near_center || near_snow;
            if tries >= width {
                reject = near_center;
            }
            if tries >= width * 2 {
                reject = false;
            }
            if !reject {
                break;
            }
        }
        dunes_biome_place(world, origin, world_surface, rng);
        // The pyramid slot (`:11564-11579`).
        if rng.next_f64() <= chance_of_pyramid {
            let num4 = rng.range(origin.0 - 200, origin.0 + 200);
            for j in 0..height {
                if let Some(t) = world.tile(num4, j) {
                    if t.active() {
                        anchors.push((num4, j + 20));
                        break;
                    }
                }
            }
        }
    }
    anchors
}

/// `DunesBiome.Place` (`DunesBiome.cs:53-64`): two dune carvers around the
/// origin — the heights `Next(60,100)` and the widths from the JSON
/// `SingleDunesWidth` 150-250 (`Configuration.json:26-31`).
fn dunes_biome_place(world: &mut World, origin: (i32, i32), world_surface: f64, rng: &mut Rng) {
    let height = rng.range(60, 100);
    let height2 = rng.range(60, 100);
    let random = rng.range(150, 251);
    let random2 = rng.range(150, 251);
    let left =
        DunesDescription::create_from_placement(world, origin.0 - random / 2 + 30, origin.1, random, height, rng);
    let right = DunesDescription::create_from_placement(
        world,
        origin.0 + random2 / 2 - 30,
        origin.1,
        random2,
        height2,
        rng,
    );
    place_single(world, &left, world_surface, rng);
    place_single(world, &right, world_surface, rng);
}

/// `PlaceSingle` (`DunesBiome.cs:66-83`): the `Next(3)+8` ridge hills with the
/// triangular scale profile, then the `Next(2)+1` centre hills at 0.8.
fn place_single(world: &mut World, desc: &DunesDescription, world_surface: f64, rng: &mut Rng) {
    let num = rng.range(0, 3) + 8;
    for i in 0..num - 1 {
        let num2 = (2.0 / num as f64 * desc.area.w as f64) as i32;
        let num3 = (i as f64 / num as f64 * desc.area.w as f64) as i32
            + desc.area.x
            + num2 * 2 / 5
            + rng.range(-5, 6);
        let scale = (1.0 - (i as f64 / (num - 2) as f64 - 0.5).abs() * 2.0) * 0.3 + 0.2;
        place_hill(world, num3 - num2 / 2, num3 + num2 / 2, scale, desc, world_surface, rng);
    }
    let num4 = rng.range(0, 2) + 1;
    for _ in 0..num4 {
        let num5 = desc.area.w / 2;
        let num6 = desc.area.center_x() + rng.range(-10, 11);
        place_hill(world, num6 - num5 / 2, num6 + num5 / 2, 0.8, desc, world_surface, rng);
    }
}

/// `PlaceHill` (`DunesBiome.cs:85-104`): the 3-point Bézier — surface
/// endpoints around a wind-jittered apex lifted `35 * scale`, walked by two
/// `PlaceCurvedLine` halves.
fn place_hill(
    world: &mut World,
    start_x: i32,
    end_x: i32,
    scale: f64,
    desc: &DunesDescription,
    world_surface: f64,
    rng: &mut Rng,
) {
    let start = (start_x, desc.surface.height(start_x));
    let end = (end_x, desc.surface.height(end_x));
    let mut apex_x = (start.0 + end.0) / 2;
    let apex_y = (start.1 + end.1) / 2 - (35.0 * scale) as i32;
    let num = (end.0 - apex_x) / 4;
    let min_value = (end.0 - apex_x) / 16;
    // The wind offset roll rides the C# global `genRand` (`:95-99`), the same
    // stream as every other roll here (`GenBase._random` IS `genRand`).
    let offset = rng.range(min_value, num + 1);
    if desc.wind_left {
        apex_x -= offset;
    } else {
        apex_x += offset;
    }
    let handle = (0, (scale * 12.0) as i32);
    let handle_mirror = (-handle.0 / 2, -handle.1 / 2);
    let apex = (apex_x, apex_y);
    place_curved_line(world, start, apex, if desc.wind_left { handle } else { handle_mirror }, desc, world_surface);
    place_curved_line(world, apex, end, if desc.wind_left { handle_mirror } else { handle }, desc, world_surface);
}

/// `PlaceCurvedLine` (`DunesBiome.cs:106-138`): the quadratic Bézier walk —
/// clears the non-sand roof notch 10 rows above the crest, then fills sand
/// from the crest down to `Surface[x] + (int)(sqrt(distFromCentre) * 3.0)`.
fn place_curved_line(
    world: &mut World,
    start: (i32, i32),
    end: (i32, i32),
    anchor_offset: (i32, i32),
    desc: &DunesDescription,
    world_surface: f64,
) {
    let mut control = ((start.0 + end.0) / 2, (start.1 + end.1) / 2);
    control.0 += anchor_offset.0;
    control.1 += anchor_offset.1;
    let span = end.0 - start.0;
    if span == 0 {
        return;
    }
    let step = 0.5 / span as f64;
    let mut previous = (-1i32, -1i32);
    let mut num2 = 0.0;
    while num2 <= 1.0 {
        // `Vector2D.Lerp(Lerp(start, ctrl, t), Lerp(ctrl, end, t), t).ToPoint()`.
        let p = quadratic_point(start, control, end, num2);
        if p != previous {
            previous = p;
            let num3 = desc.area.w / 2 - (p.0 - desc.area.center_x()).abs();
            let num4 = desc.surface.height(p.0) + (num3.max(0) as f64).sqrt() as i32 * 3;
            // The roof notch (`:126-132`): non-sand tiles above the crest clear.
            for i in (p.1 - 10)..p.1 {
                if let Some(t) = world.tile(p.0, i) {
                    if t.active() && t.kind != TILE_SAND {
                        if let Some(t) = world.tile_mut(p.0, i) {
                            t.kind = 0;
                            t.flags = 0;
                            t.wall = 0;
                        }
                    }
                }
            }
            // The sand fill (`:135`): `ResetToType(53)` down to the skirt.
            for j in p.1..num4 {
                if let Some(t) = world.tile_mut(p.0, j) {
                    t.kind = TILE_SAND;
                    t.flags = FLAG_ACTIVE;
                    t.meta = 0;
                    t.liquid = 0;
                    t.frame_x = 0;
                    t.frame_y = 0;
                }
            }
        }
        num2 += step;
    }
    let _ = world_surface;
}

/// The quadratic Bézier point at `t` (`Vector2D.Lerp` twice, `:121-122`).
fn quadratic_point(a: (i32, i32), b: (i32, i32), c: (i32, i32), t: f64) -> (i32, i32) {
    let lerp = |p: (i32, i32), q: (i32, i32)| {
        (
            (p.0 as f64 + (q.0 as f64 - p.0 as f64) * t) as i32,
            (p.1 as f64 + (q.1 as f64 - p.1 as f64) * t) as i32,
        )
    };
    let ab = lerp(a, b);
    let bc = lerp(b, c);
    lerp(ab, bc)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A flat stone world with the surface at row 40 — the dune fixture.
    fn flat(w: i32, h: i32) -> (World, GenVars) {
        let mut world = World::new("dunes".into(), w, h);
        for x in 0..w {
            for y in 40..h {
                let i = world.index(x, y);
                world.tiles[i].kind = 1;
                world.tiles[i].flags |= FLAG_ACTIVE;
            }
        }
        let mut gv = GenVars::default();
        gv.jungle_origin_x = -10_000;
        (world, gv)
    }

    #[test]
    fn surface_line_finds_the_first_solid_row() {
        let (world, _) = flat(200, 300);
        let line = DuneSurface::from_area(&world, 10, 50);
        // The scan starts at y = 50 (`SurfaceMap.cs:51`) — the flat fixture
        // fills from row 40, so the line reports 50.
        assert_eq!(line.height(10), 50);
        assert_eq!(line.height(59), 50);
        assert_eq!(line.average(), 50.0);
        assert_eq!(line.top(), 50);
        assert_eq!(line.bottom(world.world_surface), 50);
    }

    #[test]
    fn dunes_carve_sand_above_the_flat_surface() {
        let (mut world, gv) = flat(4200, 1200);
        let before = world
            .tiles
            .iter()
            .filter(|t| t.active() && t.kind == TILE_SAND)
            .count();
        let mut rng = Rng(0x5EED);
        let surface_cap = world.world_surface;
        let anchors = dunes(&mut world, &gv, 2800, surface_cap, 2, 0.8, &mut rng);;
        let after = world
            .tiles
            .iter()
            .filter(|t| t.active() && t.kind == TILE_SAND)
            .count();
        assert!(after > before, "the dunes drop sand ({before} -> {after})");
        // The sand sits at or above the flat surface row plus the dune skirt.
        let width = world.width as usize;
        let shallow = world.tiles.iter().enumerate().any(|(i, t)| {
            let x = (i % width) as i32;
            let y = (i / width) as i32;
            t.active() && t.kind == TILE_SAND && x > 600 && x < world.width - 600 && y < 40
        });
        assert!(shallow, "no dune crest above the flat row");
    }

    #[test]
    fn the_sqrt_skirt_deepens_the_centre_columns() {
        // One hill over a flat surface: the sand depth follows
        // `Surface[x] + (int)(sqrt(W/2 - |x - centreX|) * 3.0)` — deepest at the
        // dune centre (`DunesBiome.cs:134-137`).
        let (mut world, _) = flat(400, 400);
        let surface = DuneSurface::from_area(&mut world, 0, 400);
        let desc = DunesDescription {
            area: Rect::new(100, 20, 200, 60),
            surface,
            wind_left: true,
        };
        let mut rng = Rng(0x5EED);
        let cap = world.world_surface;
        place_hill(&mut world, 150, 250, 0.5, &desc, cap, &mut rng);
        let depth = |x: i32| {
            let mut n = 0;
            for y in 0..80 {
                if let Some(t) = world.tile(x, y) {
                    if t.active() && t.kind == TILE_SAND {
                        n += 1;
                    }
                }
            }
            n
        };
        let centre = depth(200);
        let edge = depth(151);
        assert!(centre >= edge, "centre {centre} < edge {edge}");
    }

    #[test]
    fn dunes_are_deterministic_for_a_seed() {
        // A 4200×1200 world: the origin rolls need the `Next(500, W-500)`
        // range the C# makes (`RandomWorldPoint`), so the fixture is the
        // default-world size.
        let (mut a, ga) = flat(4200, 1200);
        let (mut b, gb) = flat(4200, 1200);
        let mut ra = Rng(0xD1CE);
        let mut rb = Rng(0xD1CE);
        let cap_a = a.world_surface;
        let anchors_a = dunes(&mut a, &ga, 2800, cap_a, 2, 0.8, &mut ra);;
        let cap_b = b.world_surface;
        let anchors_b = dunes(&mut b, &gb, 2800, cap_b, 2, 0.8, &mut rb);;
        assert_eq!(anchors_a, anchors_b);
        assert_eq!(a.tiles.len(), b.tiles.len());
        for (n, (ta, tb)) in a.tiles.iter().zip(b.tiles.iter()).enumerate() {
            assert_eq!(
                (ta.kind, ta.active(), ta.wall),
                (tb.kind, tb.active(), tb.wall),
                "tile {n} diverges"
            );
        }
    }
}
