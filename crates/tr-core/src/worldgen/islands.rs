//! The floating islands — the verbatim port of the `FloatingIslands` pass
//! (`WorldGen.cs:12927-13122`) + the `FloatingIslandHouses` pass
//! (`:17859-17874`) → `IslandHouse` (`:79203-79484`), W4-g44 (the census +
//! residuals list: ANALYSIS "Sky lakes / floating-island residuals").
//!
//! The disc BODIES stay the compact M1.4-WGEN-4 stand-ins (`CloudIsland`
//! `:78132-78538` / `CloudLake` `:78539-79202`'s shrinking-ring walks are
//! the recorded shape-level deviation): the elliptical cloud 189 disc with
//! the ragged rim, the grass cap, the rain-cloud 196 lake bowl. The one
//! CloudIsland slice the house pass consumes — the wall-73 interior flood
//! (`:78421-78442`) — lands verbatim here (`flood_cloud_walls`).
//!
//! The pass mechanics are verbatim: the count `(int)(maxTilesX * 0.0008)`
//! with the errorWorld triple, the `skyLakes` budget (Reset,
//! `:11182-11190`) appended as forced-lake tail islands (`num2 >= num`),
//! the extraFloatingIslands multipliers, the spawn-exclusion re-draw loop
//! (150 → 100), the house-anchor spacing (180 → 100, strict bounds), the
//! column-scan y-band with the `worldSurfaceLow - 50` clamp, the
//! noSurface/errorWorld y-arms, the width attempt budget, the GenVars
//! island ledger and the post-loop `num3 * 2` house cap. The houses are
//! the separate `FloatingIslandHouses` step: sunplate 202 / disc wall 82,
//! the door-side flip, the ground scan, the style-9 door, the chest cycle
//! (S1-A-4), the wall-21 windows, the table/chairs/banners.
//!
//! The drunk snow/desert variant islands (`SnowCloudIsland`
//! `:77346-77737` / `DesertCloudIsland` `:77739-78130` — one shared port,
//! the bodies are line-identical bar four constants) and the full
//! remix/getGood/anniversary style tables (`:13090-13103` + the
//! `IslandHouse` door/furniture/window/banner switches) land with the
//! fountain (`:79377-79392`) and palm (`:79462-79483`) house arms.
//!
//! Deferred (ANALYSIS): the skyblock worldgen terrain (f24 owns
//! `GenerateSkyBlockWorld`; the island-side skyblock arms — the `/42`
//! budget, the fixed y line, the clear window, the deny gate — are
//! here), `SquareTileFrame` after the house writes (the chain's framing
//! passes run later — the temple precedent).

use super::pipeline::GenVars;
use crate::seeds::{MicroSeedFlags, SeedFlags};
use crate::tile::{FLAG_ACTIVE, FLAG_HALF_BRICK, Tile};
use crate::tiledata as td;
use crate::world::World;

pub(crate) const TILE_CLOUD: u16 = 189;
pub(crate) const TILE_RAIN_CLOUD: u16 = 196;
/// `WallID.Cloud` (`WallID.cs`) — the cloud-disc interior wall 73
/// (`flood_cloud_walls`). The island HOUSE interiors carry the sunplate
/// wall 82 (the g44 IslandHouse survivor, research/87).
pub(crate) const WALL_CLOUD: u16 = 73;
/// `TileID.Sunplate` (`TileID.cs:1033`) — the island house's frame.
const TILE_SUNPLATE: u16 = 202;
/// `WallID.DiscWall` (`WallID.cs:269`) — the house interior.
const WALL_DISC: u16 = 82;
/// `WallID.Glass` (`WallID.cs:147`) — the house window strips.
const WALL_GLASS: u16 = 21;

fn put_cloud(world: &mut World, x: i32, y: i32, kind: u16) {
    if x < 2 || y < 2 || x >= world.width - 2 || y >= world.height - 2 {
        return;
    }
    let i = world.index(x, y);
    let t = &mut world.tiles[i];
    t.kind = kind;
    t.flags |= FLAG_ACTIVE;
    t.liquid = 0;
    t.set_slope(0);
}

/// The `CloudIsland` wall-73 flood (`WorldGen.cs:78421-78442`), verbatim
/// over the port's disc bounds: a cell takes the cloud wall when its 3×3
/// neighbourhood is all active with no foreign wall (`:78430`). This is
/// the slice `IslandHouse`'s ground scan (`:79241`'s `wall > 0`) consumes.
/// `SquareWallFrame` is the chain-framing deviation (the module doc).
fn flood_cloud_walls(world: &mut World, cx: i32, cy: i32, radius: i32) {
    let (x0, x1) = (cx - radius - 20, cx + radius + 20);
    let (y0, y1) = (cy - radius / 2 - 20, cy + radius / 2 + 20);
    for x in x0..=x1 {
        for y in y0..=y1 {
            if x < 1 || y < 1 || x >= world.width - 1 || y >= world.height - 1 {
                continue;
            }
            let mut clear = true;
            for nx in (x - 1)..=(x + 1) {
                for ny in (y - 1)..=(y + 1) {
                    let ok = world.tile(nx, ny).is_some_and(|t| {
                        t.active() && (t.wall == 0 || t.wall == WALL_CLOUD)
                    });
                    if !ok {
                        clear = false;
                    }
                }
            }
            if clear {
                let i = world.index(x, y);
                world.tiles[i].wall = WALL_CLOUD;
            }
        }
    }
}

/// The `CloudIsland` disc (`:78132+`): the elliptical cloud body with the
/// grass cap on the top surface. `lake` islands carve the rain-cloud bowl
/// with the water instead of the grass cap's centre. The compact body is
/// the M1.4-WGEN-4 shape deviation; the wall flood is verbatim.
fn build_island(world: &mut World, cx: i32, cy: i32, radius: i32, lake: bool, rng: &mut super::Rng) {
    for dx in -radius..=radius {
        for dy in -radius / 2..=radius / 2 {
            let px = cx + dx;
            let py = cy + dy;
            if px < 2 || py < 2 || px >= world.width - 2 || py >= world.height - 2 {
                continue;
            }
            // The ellipse: the horizontal radius full, the vertical halved.
            let ex = dx as f32 / radius as f32;
            let ey = dy as f32 / (radius as f32 * 0.5);
            let d = (ex * ex + ey * ey).sqrt();
            if d > 1.0 {
                continue;
            }
            // The wobble on the boundary (`Next(80, 120) * 0.01`'s ragged
            // rim — the cells at 0.82..1.0 roll 50/50).
            if d > 0.82 && rng.range(0, 2) == 0 {
                continue;
            }
            put_cloud(world, px, py, TILE_CLOUD);
        }
    }
    // The grass cap on the top surface (the compact 2-row cap — the C#'s
    // per-column ragged grass line is the shape deviation).
    for dx in -(radius - 2)..=(radius - 2) {
        for dy in 0..2 {
            let px = cx + dx;
            let py = cy - radius / 2 + dy;
            if px < 2 || py < 2 || px >= world.width - 2 {
                continue;
            }
            if world
                .tile(px, py)
                .is_some_and(|t| t.active() && t.kind == TILE_CLOUD)
            {
                let i = world.index(px, py);
                world.tiles[i].kind = if dy == 0 { 2 } else { 0 };
                world.tiles[i].flags |= FLAG_ACTIVE;
            }
        }
    }
    if lake {
        // The rain-cloud bowl with the water (`CloudLake`'s compact shape).
        let half = (radius / 3).max(4);
        for dx in -half..=half {
            for dy in 0..4 {
                let px = cx + dx;
                let py = cy - radius / 2 + 2 + dy;
                if px < 2 || py < 2 || px >= world.width - 2 {
                    continue;
                }
                let i = world.index(px, py);
                if dy == 3 || dx.abs() == half {
                    let t = &mut world.tiles[i];
                    t.kind = TILE_RAIN_CLOUD;
                    t.flags |= FLAG_ACTIVE;
                    t.set_slope(0);
                } else {
                    let t = &mut world.tiles[i];
                    *t = Tile::default();
                    t.liquid = 255;
                    t.set_liquid_kind(crate::tile::LiquidKind::Water);
                }
            }
        }
    }
    // The wall-73 interior flood (`:78421-78442`) — the house ground scan
    // keys on it. Runs for both bodies (the C# lake carries its own flood
    // at `:78800-78824`; the port's single flood covers both).
    flood_cloud_walls(world, cx, cy, radius);
}

/// `WorldGen.SolidTile` (`WorldGen.cs:70111-70131`) — the variant-blob
/// predicate: active, solid, NOT solid-top, no half/slope/actuation
/// (the `WillWaterPlacedHereStayPut` floor test).
fn solid_block(world: &World, x: i32, y: i32) -> bool {
    world.tile(x, y).is_some_and(|t| {
        t.active()
            && !t.inactive()
            && td::TILE_SOLID.has(t.kind)
            && !td::TILE_SOLID_TOP.has(t.kind)
            && !t.half_brick()
            && t.slope() == 0
    })
}

/// `WorldGen.WillWaterPlacedHereStayPut` (`WorldGen.cs:78530-78537`):
/// the cell below AND both side cells are solid non-top tiles or already
/// hold full liquid.
fn will_water_placed_here_stay_put(world: &World, x: i32, y: i32) -> bool {
    let held = |px: i32, py: i32| {
        solid_block(world, px, py) || world.tile(px, py).is_some_and(|t| t.liquid == 255)
    };
    held(x, y + 1) && held(x - 1, y) && held(x + 1, y)
}

/// `SecretSeed.surfaceIsInSpace`'s cloud-clear column strip
/// (`WorldGen.cs:77405-77423` / `:77798-77816`): under
/// surfaceIsInSpace && !errorWorld every cloud tile above the ragged
/// surface line deactivates before the ring writes.
fn surface_in_space_clear(world: &mut World, active: bool, k: i32, cap: f64) {
    if !active {
        return;
    }
    let cap = cap as i32;
    for l in 0..=cap {
        if let Some(t) = world.tile_mut(k, l) {
            if td::SET_CLOUDS.has(t.kind) {
                t.flags &= !FLAG_ACTIVE;
            }
        }
    }
}

/// The shared drift step of both ring walks (`:77456-77473` etc.):
/// the centre moves, the x-drift accumulates `Next(-20,21)*0.05` clamped
/// to ±1, the y-drift pins to −0.2 past either bound (the C# writes
/// −0.2 on BOTH arms — kept verbatim).
fn drift(v: &mut (f64, f64), dv: &mut (f64, f64), rng: &mut super::Rng) {
    v.0 += dv.0;
    v.1 += dv.1;
    dv.0 += rng.range(-20, 21) as f64 * 0.05;
    if dv.0 > 1.0 {
        dv.0 = 1.0;
    }
    if dv.0 < -1.0 {
        dv.0 = -1.0;
    }
    if dv.1 > 0.2 {
        dv.1 = -0.2;
    }
    if dv.1 < -0.2 {
        dv.1 = -0.2;
    }
}

/// The variant island's first ring walk (`SnowCloudIsland`
/// `:77348-77474` / `DesertCloudIsland` `:77741-77867`): the
/// `Next(100,150)` budget shrinking by `Next(4)` per ring over
/// `Next(20,30)` rings, the ragged per-column surface line, the drifting
/// ellipse (`|dx|, 3|dy|` vs `num2 * 0.4`) writing cloud 189, tracking
/// the body bounds. Returns `(min_x, max_x, min_y, max_y)`.
fn variant_body_ring(
    world: &mut World,
    i: i32,
    j: i32,
    space_clear: bool,
    rng: &mut super::Rng,
) -> (i32, i32, i32, i32) {
    let mut num = rng.range(100, 150) as f64;
    let mut num3 = rng.range(20, 30);
    let (mut num4, mut num5, mut num6, mut num7) = (i, i, j, j);
    let mut v = (i as f64, j as f64);
    let mut dv = (rng.range(-20, 21) as f64 * 0.2, 0.0);
    while dv.0 > -2.0 && dv.0 < 2.0 {
        dv.0 = rng.range(-20, 21) as f64 * 0.2;
    }
    dv.1 = rng.range(-20, -10) as f64 * 0.02;
    while num > 0.0 && num3 > 0 {
        num -= rng.range(0, 4) as f64;
        num3 -= 1;
        let num8 = ((v.0 - num * 0.5) as i32).max(0);
        let num9 = ((v.0 + num * 0.5) as i32).min(world.width);
        let num10 = ((v.1 - num * 0.5) as i32).max(0);
        let num11 = ((v.1 + num * 0.5) as i32).min(world.height);
        let num2 = num * rng.range(80, 120) as f64 * 0.01;
        let mut num12 = v.1 + 1.0;
        for k in num8..num9 {
            if rng.range(0, 2) == 0 {
                num12 += rng.range(-1, 2) as f64;
            }
            num12 = num12.clamp(v.1, v.1 + 2.0);
            surface_in_space_clear(world, space_clear, k, num12.clamp(num10 as f64, num11 as f64));
            for m in num10..num11 {
                if m as f64 <= num12 {
                    continue;
                }
                let dx = (k as f64 - v.0).abs();
                let dy = (m as f64 - v.1).abs() * 3.0;
                if (dx * dx + dy * dy).sqrt() < num2 * 0.4 {
                    num4 = num4.min(k);
                    num5 = num5.max(k);
                    num6 = num6.min(m);
                    num7 = num7.max(m);
                    // `:77450-77452`: RAW active + type 189 (no liquid/
                    // slope reset); SquareTileFrame rides the chain-
                    // framing deviation.
                    if let Some(t) = world.tile_mut(k, m) {
                        t.flags |= FLAG_ACTIVE;
                        t.kind = TILE_CLOUD;
                    }
                }
            }
        }
        drift(&mut v, &mut dv, rng);
    }
    (num4, num5, num6, num7)
}

/// The variant island's underside blob loop (`:77476-77507` /
/// `:77869-77900`): blobs of `Next(4,8)` radius hanging below the body,
/// 1-in-4 taking the variant kind (snow cloud 460 / rain cloud 196).
/// The C# `for` increment reads `num17` only AFTER the first body pass
/// assigned it — the port's while-loop keeps that order (no primed
/// draw).
#[allow(clippy::too_many_arguments)]
fn variant_underside_blobs(
    world: &mut World,
    bounds: (i32, i32, i32, i32),
    blob_kind: u16,
    rng: &mut super::Rng,
) {
    let (num4, num5, num6, num7) = bounds;
    let mut num17 = 0; // assigned by the body before the step reads it
    let mut n = num4 + rng.range(0, 5);
    while n < num5 {
        let mut num16 = num7;
        while super::temple::in_world(world, n, num16, 15)
            && !world.tile(n, num16).is_some_and(|t| t.active())
        {
            num16 -= 1;
        }
        num16 += rng.range(-3, 4);
        num17 = rng.range(4, 8);
        let num18 = if rng.range(0, 4) == 0 { blob_kind } else { TILE_CLOUD };
        for num19 in (n - num17)..=(n + num17) {
            for num20 in (num16 - num17)..=(num16 + num17) {
                if num20 <= num6 {
                    continue;
                }
                let dx = (num19 - n).abs() as f64;
                let dy = (num20 - num16).abs() as f64 * 2.0;
                if (dx * dx + dy * dy).sqrt() < (num17 + rng.range(0, 2)) as f64 {
                    // The blob writes are RAW (`:77500-77502`): active +
                    // type, no liquid/slope reset (unlike the body ring).
                    if let Some(t) = world.tile_mut(num19, num20) {
                        t.flags |= FLAG_ACTIVE;
                        t.kind = num18;
                    }
                }
            }
        }
        n += rng.range(num17, (num17 as f64 * 1.5) as i32);
    }
}

/// The variant island's second ring walk (`:77508-77591` /
/// `:77901-77984`): a `Next(80,95)` budget over `Next(10,15)` rings
/// centred on the body's TOP row, RETYPING cloud 189 cells inside the
/// ellipse to the surface kind (snow 147 / sand 53).
fn variant_surface_ring(
    world: &mut World,
    i: i32,
    top: i32,
    surface_kind: u16,
    rng: &mut super::Rng,
) {
    let mut num = rng.range(80, 95) as f64;
    let mut num3 = rng.range(10, 15);
    let mut v = (i as f64, top as f64);
    let mut dv = (rng.range(-20, 21) as f64 * 0.2, 0.0);
    while dv.0 > -2.0 && dv.0 < 2.0 {
        dv.0 = rng.range(-20, 21) as f64 * 0.2;
    }
    dv.1 = rng.range(-20, -10) as f64 * 0.02;
    while num > 0.0 && num3 > 0 {
        num -= rng.range(0, 4) as f64;
        num3 -= 1;
        let num8 = ((v.0 - num * 0.5) as i32).max(0);
        let num9 = ((v.0 + num * 0.5) as i32).min(world.width);
        let num10 = (top - 1).max(0);
        let num11 = ((v.1 + num * 0.5) as i32).min(world.height);
        let num2 = num * rng.range(80, 120) as f64 * 0.01;
        let mut num23 = v.1 + 1.0;
        for num24 in num8..num9 {
            if rng.range(0, 2) == 0 {
                num23 += rng.range(-1, 2) as f64;
            }
            num23 = num23.clamp(v.1, v.1 + 2.0);
            for num25 in num10..num11 {
                if num25 as f64 <= num23 {
                    continue;
                }
                let dx = (num24 as f64 - v.0).abs();
                let dy = (num25 as f64 - v.1).abs() * 3.0;
                if (dx * dx + dy * dy).sqrt() < num2 * 0.4
                    && world.tile(num24, num25).is_some_and(|t| t.kind == TILE_CLOUD)
                {
                    let idx = world.index(num24, num25);
                    world.tiles[idx].kind = surface_kind;
                }
            }
        }
        drift(&mut v, &mut dv, rng);
    }
}

/// The variant island's surface-pocket loop (`:77592-77629` /
/// `:77985-78022`): scan for surface-kind columns, punch cloud-189
/// pockets of `Next(2,5)` radius into them. The C#'s `continue` on an
/// exhausted scan re-tests the outer `while (num28 < num5)` and exits —
/// the port's `break` is the same hop.
fn variant_surface_pockets(
    world: &mut World,
    bounds: (i32, i32, i32, i32),
    surface_kind: u16,
    rng: &mut super::Rng,
) {
    let (num4, num5, num6, num7) = bounds;
    let mut num28 = num4 + rng.range(0, 5);
    while num28 < num5 {
        let mut num29 = num7;
        while num28 < num5
            && !world
                .tile(num28, num29)
                .is_some_and(|t| t.active() && t.kind == surface_kind)
        {
            num29 -= 1;
            if num29 < num6 {
                num29 = num7;
                num28 += rng.range(1, 4);
            }
        }
        if num28 >= num5 {
            break; // `:77605-77608`: the C# continue re-tests the outer while
        }
        num29 += rng.range(0, 4);
        let num30 = rng.range(2, 5);
        for num32 in (num28 - num30)..=(num28 + num30) {
            for num33 in (num29 - num30)..=(num29 + num30) {
                if num33 <= num6 {
                    continue;
                }
                let dx = (num32 - num28).abs() as f64;
                let dy = (num33 - num29).abs() as f64 * 2.0;
                if (dx * dx + dy * dy).sqrt() < num30 as f64 {
                    if let Some(t) = world.tile_mut(num32, num33) {
                        t.kind = TILE_CLOUD; // `:77622` sets type only
                    }
                }
            }
        }
        num28 += rng.range(num30, (num30 as f64 * 1.5) as i32);
    }
}

/// The variant island's per-column water pass (`:77652-77696` /
/// `:78045-78089`): find each column's top surface row, 1-in-10 carve a
/// `Next(1,3)`-wide 2/3-row pool, 1-in-5 wet the surface cell.
fn variant_column_water(
    world: &mut World,
    bounds: (i32, i32, i32, i32),
    rng: &mut super::Rng,
) {
    let (num4, num5, num6, num7) = bounds;
    for num40 in num4..=num5 {
        // `:77655`: scan down from num6-10 for the first active row
        // below. The C# scan is unbounded (it always lands on terrain);
        // the port stops at the world floor (the sub-vanilla valve).
        let mut num41 = num6 - 10;
        while num41 + 1 < world.height
            && !world.tile(num40, num41 + 1).is_some_and(|t| t.active())
        {
            num41 += 1;
        }
        if num41 >= num7
            || !world.tile(num40, num41 + 1).is_some_and(|t| t.kind == TILE_CLOUD)
        {
            continue;
        }
        if rng.range(0, 10) == 0 {
            let num42 = rng.range(1, 3);
            for num43 in (num40 - num42)..=(num40 + num42) {
                // `:77667-77673`: the top row.
                if world.tile(num43, num41).is_some_and(|t| t.kind == TILE_CLOUD)
                    && will_water_placed_here_stay_put(world, num43, num41)
                {
                    if let Some(t) = world.tile_mut(num43, num41) {
                        t.flags &= !FLAG_ACTIVE;
                        t.liquid = 255;
                        t.set_liquid_kind(crate::tile::LiquidKind::Water);
                    }
                }
                // `:77674-77680`: the second row.
                if world.tile(num43, num41 + 1).is_some_and(|t| t.kind == TILE_CLOUD)
                    && will_water_placed_here_stay_put(world, num43, num41 + 1)
                {
                    if let Some(t) = world.tile_mut(num43, num41 + 1) {
                        t.flags &= !FLAG_ACTIVE;
                        t.liquid = 255;
                        t.set_liquid_kind(crate::tile::LiquidKind::Water);
                    }
                }
                // `:77681-77687`: the third row, the inner columns only.
                if num43 > num40 - num42
                    && num43 < num40 + 2
                    && world.tile(num43, num41 + 2).is_some_and(|t| t.kind == TILE_CLOUD)
                    && will_water_placed_here_stay_put(world, num43, num41 + 2)
                {
                    if let Some(t) = world.tile_mut(num43, num41 + 2) {
                        t.flags &= !FLAG_ACTIVE;
                        t.liquid = 255;
                        t.set_liquid_kind(crate::tile::LiquidKind::Water);
                    }
                }
            }
        }
        if rng.range(0, 5) == 0 && will_water_placed_here_stay_put(world, num40, num41) {
            if let Some(t) = world.tile_mut(num40, num41) {
                t.liquid = 255;
            }
        }
        if let Some(t) = world.tile_mut(num40, num41) {
            t.set_liquid_kind(crate::tile::LiquidKind::Water);
        }
    }
}

/// The variant island's satellite puffs (`:77697-77736` /
/// `:78090-78129`): `Next(4)`+1 small clouds above the body, 50/50 the
/// variant kind, each with a water cell on its top surface.
fn variant_satellites(
    world: &mut World,
    bounds: (i32, i32, i32, i32),
    puff_kind: u16,
    rng: &mut super::Rng,
) {
    let (num4, num5, num6, _num7) = bounds;
    let num44 = rng.range(0, 4);
    for _num45 in 0..=num44 {
        let num46 = rng.range(num4 - 5, num5 + 5);
        let num47 = num6 - rng.range(20, 40);
        let num48 = rng.range(4, 8);
        let num49 = if rng.range(0, 2) == 0 { puff_kind } else { TILE_CLOUD };
        for num50 in (num46 - num48)..=(num46 + num48) {
            for num51 in (num47 - num48)..=(num47 + num48) {
                let dx = (num50 - num46).abs() as f64;
                let dy = (num51 - num47).abs() as f64 * 2.0;
                if (dx * dx + dy * dy).sqrt() < (num48 + rng.range(-1, 2)) as f64 {
                    // `:77716-77718`: RAW active + type.
                    if let Some(t) = world.tile_mut(num50, num51) {
                        t.flags |= FLAG_ACTIVE;
                        t.kind = num49;
                    }
                }
            }
        }
        for num54 in (num46 - num48 + 2)..=(num46 + num48 - 2) {
            let mut num55 = num47 - num48;
            while num55 < world.height
                && !world.tile(num54, num55).is_some_and(|t| t.active())
            {
                num55 += 1;
            }
            if will_water_placed_here_stay_put(world, num54, num55) {
                if let Some(t) = world.tile_mut(num54, num55) {
                    t.flags &= !FLAG_ACTIVE;
                    t.liquid = 255;
                    t.set_liquid_kind(crate::tile::LiquidKind::Water);
                }
            }
        }
    }
}

/// `SnowCloudIsland` (`WorldGen.cs:77346-77737`) / `DesertCloudIsland`
/// (`:77739-78130`), verbatim: the two bodies are line-identical except
/// four constants — the underside/satellite kind (460 snow cloud vs 196
/// rain cloud) and the surface kind (147 snow vs 53 sand). One port,
/// `snow` selects the pair. The wall-73 flood rides the shared
/// `flood_cloud_walls` (the C# bounds `num4..num5 × num6..num7 ±20` are
/// the body bbox; the port's disc-radius box is the compact shape's
/// equivalent — recorded). Returns the body bbox for the flood.
fn variant_cloud_island(
    world: &mut World,
    i: i32,
    j: i32,
    snow: bool,
    space_clear: bool,
    rng: &mut super::Rng,
) {
    let (blob_kind, surface_kind) = if snow { (460u16, 147u16) } else { (196, 53) };
    let bounds = variant_body_ring(world, i, j, space_clear, rng);
    variant_underside_blobs(world, bounds, blob_kind, rng);
    variant_surface_ring(world, i, bounds.2, surface_kind, rng);
    variant_surface_pockets(world, bounds, surface_kind, rng);
    // `:77630-77651`: the wall-73 flood over the body bbox ±20.
    flood_cloud_walls_bbox(world, bounds);
    variant_column_water(world, bounds, rng);
    variant_satellites(world, bounds, blob_kind, rng);
}

/// The wall-73 flood at the variant body's bbox (`:77630-77651` /
/// `:78023-78044`) — the same 3×3 test as `flood_cloud_walls`, the C#
/// bbox bounds instead of the compact disc's radius box.
fn flood_cloud_walls_bbox(world: &mut World, bounds: (i32, i32, i32, i32)) {
    let (num4, num5, num6, num7) = bounds;
    for x in (num4 - 20)..=(num5 + 20) {
        for y in (num6 - 20)..=(num7 + 20) {
            if x < 1 || y < 1 || x >= world.width - 1 || y >= world.height - 1 {
                continue;
            }
            let mut clear = true;
            for nx in (x - 1)..=(x + 1) {
                for ny in (y - 1)..=(y + 1) {
                    let ok = world.tile(nx, ny).is_some_and(|t| {
                        t.active() && (t.wall == 0 || t.wall == WALL_CLOUD)
                    });
                    if !ok {
                        clear = false;
                    }
                }
            }
            if clear {
                let i = world.index(x, y);
                world.tiles[i].wall = WALL_CLOUD;
            }
        }
    }
}

/// `SecretSeed.Variations.errorWorldAdjustment` (`WorldGen.cs:329-336`):
/// `< 1` active seeds → 4, else `(int)(value * ((count + 3) / 4))` — the
/// C#'s INTEGER division inside the cast.
pub(crate) fn error_world_adjustment(value: f64, world: &World) -> i32 {
    let count = crate::seeds::active_secret_seed_count(world.micro_seeds, world.seeds);
    if count < 1 {
        return 4;
    }
    (value * ((count + 3) / 4) as f64) as i32
}

/// `Skyblock.denyFloatingIslands` (`WorldGen.cs:3087-3096`): a skyblock
/// world without extraFloatingIslands denies both island passes. The
/// port's SKYBLOCK bit stands in for `WorldGen.skyblockWorldGen` (the
/// skyblock terrain itself is M5-SEED-1's recorded deviation).
fn deny_floating_islands(world: &World) -> bool {
    world.seeds.contains(SeedFlags::SKYBLOCK)
        && !world
            .micro_seeds
            .contains(MicroSeedFlags::EXTRA_FLOATING_ISLANDS)
}

/// `SecretSeed.Variations.noSurfaceNoFloatingIslands`
/// (`WorldGen.cs:169-178`): noSurface && !errorWorld → !extra.
fn no_surface_no_floating_islands(world: &World) -> bool {
    if world.micro_seeds.contains(MicroSeedFlags::NO_SURFACE)
        && !world.micro_seeds.contains(MicroSeedFlags::ERROR_WORLD)
    {
        return !world
            .micro_seeds
            .contains(MicroSeedFlags::EXTRA_FLOATING_ISLANDS);
    }
    false
}

/// `SecretSeed.Variations.extraFloatingIslandsNormalAmount`
/// (`WorldGen.cs:233-243`): an extraFloatingIslands world that is ALSO a
/// skyblock world keeps the normal (non-multiplied) island count.
fn extra_floating_islands_normal_amount(world: &World) -> bool {
    world
        .micro_seeds
        .contains(MicroSeedFlags::EXTRA_FLOATING_ISLANDS)
        && world.seeds.contains(SeedFlags::SKYBLOCK)
}

/// `SecretSeed.Variations.extraFloatingIslandsReducedAmount`
/// (`WorldGen.cs:245-255`): the halved multiplier when the seed list is
/// crowded — `activeSecretSeedCount < 6` reads `noSurface.Enabled`,
/// otherwise `true`.
fn extra_floating_islands_reduced_amount(world: &World) -> bool {
    if !world
        .micro_seeds
        .contains(MicroSeedFlags::EXTRA_FLOATING_ISLANDS)
        || crate::seeds::active_secret_seed_count(world.micro_seeds, world.seeds) < 6
    {
        world.micro_seeds.contains(MicroSeedFlags::NO_SURFACE)
    } else {
        true
    }
}

/// The `FloatingIslands` pass (`WorldGen.cs:12927-13122`), verbatim: the
/// count + multipliers, the width attempt budget, the spawn-exclusion
/// re-draw, the house-anchor spacing, the column-scan y-band, the
/// `num2 >= num` lake tail, the GenVars ledger writes and the post-loop
/// house cap. Returns the placed-island count (`GenVars.numIslandHouses`
/// after the cap). The disc bodies ride the compact `build_island`.
///
/// Sub-vanilla valves (deterministic skips where vanilla's 4200×1200
/// minimum never degenerates): a column with no active tile below row
/// 200 fails the site (`flag2` stays false — the C# rolls
/// `Next(90, -100)` and throws); an empty y-band skips the island before
/// the draw (the C# `Next(90, num6 - 100)` throws there too).
pub(crate) fn pass_floating_islands(
    world: &mut World,
    gv: &mut GenVars,
    rng: &mut super::Rng,
    surface_low: f64,
    crimson: bool,
) -> i32 {
    // `:12929`: the deny gates.
    if deny_floating_islands(world) || no_surface_no_floating_islands(world) {
        return 0;
    }
    // `:12931-12932`: the pass zeroes both counters.
    gv.num_island_houses = 0;
    gv.sky_island_house_count = 0;
    gv.island_house_x.clear();
    gv.island_house_y.clear();
    gv.island_style.clear();
    gv.sky_lake.clear();
    let extra = world
        .micro_seeds
        .contains(MicroSeedFlags::EXTRA_FLOATING_ISLANDS);
    let error = world.micro_seeds.contains(MicroSeedFlags::ERROR_WORLD);
    let no_surface = world.micro_seeds.contains(MicroSeedFlags::NO_SURFACE);
    let skyblock = world.seeds.contains(SeedFlags::SKYBLOCK);
    // `:12934-12939`: `num = (int)(maxTilesX * 0.0008)`, the errorWorld
    // triple BEFORE `num3 = num`.
    let mut num = (world.width as f64 * 0.0008) as i32;
    let mut num2 = 0;
    if error && error_world_adjustment(1.0, world) < 3 {
        num *= 3;
    }
    // `:12940-12954`: `num3` (the house cap's base) + the multipliers —
    // `skyLakes` takes the same multiplier as `num`.
    let num3 = num;
    let mut flag = false;
    if extra && !extra_floating_islands_normal_amount(world) {
        if extra_floating_islands_reduced_amount(world) {
            num *= 2;
            gv.sky_lakes *= 2;
        } else {
            num *= 10;
            gv.sky_lakes *= 10;
        }
    }
    // `:12955-12956`: `num4 = num + GenVars.skyLakes`.
    let num4 = num + gv.sky_lakes;
    let underworld_layer = world.height - 200; // `Main.cs:2790`
    for i in 0..num4 {
        // `:12959-12964`: the attempt budget — the width, `/42` skyblock.
        let mut num5 = world.width;
        if skyblock {
            num5 /= 42;
        }
        while {
            num5 -= 1;
            num5 > 0
        } {
            // `:12968-12979`: the x draw + the spawn-exclusion RE-DRAW
            // loop (150 → 100 under extra; errorWorld skips it entirely).
            let mut num7 =
                rng.range((world.width as f64 * 0.1) as i32, (world.width as f64 * 0.9) as i32);
            let num8 = if extra { 100 } else { 150 };
            if !error {
                while num7 > world.width / 2 - num8 && num7 < world.width / 2 + num8 {
                    num7 = rng.range(
                        (world.width as f64 * 0.1) as i32,
                        (world.width as f64 * 0.9) as i32,
                    );
                }
            }
            // `:12981-12993`: the spacing against the placed-island
            // anchors — strict bounds, 180 → 100 under multiplied extra.
            let num9 = if extra && !extra_floating_islands_normal_amount(world) {
                100
            } else {
                180
            };
            let mut flag2 = true;
            for j in 0..gv.num_island_houses as usize {
                let hx = gv.island_house_x[j];
                if num7 > hx - num9 && num7 < hx + num9 {
                    flag2 = false;
                    break;
                }
            }
            if !flag2 {
                continue;
            }
            // `:12996-13019`: the y anchor — noSurface's deep roll,
            // skyblock's fixed line, else the column scan from row 200.
            flag2 = false;
            let num6;
            if no_surface {
                // The sub-vanilla valve: `Next(200, UnderworldLayer - 200)`
                // throws on an inverted band (the C# never generates one).
                if underworld_layer - 200 < 200 {
                    continue;
                }
                num6 = rng.range(200, underworld_layer - 200);
                flag2 = true;
            } else if skyblock {
                num6 = (world.world_surface / 2.0 + 100.0) as i32;
                flag2 = true;
            } else {
                let mut found = None;
                for k in 200..(world.world_surface as i32) {
                    if world.tile(num7, k).is_some_and(|t| t.active()) {
                        found = Some(k);
                        break;
                    }
                }
                match found {
                    Some(k) => {
                        num6 = k;
                        flag2 = true;
                    }
                    None => continue, // the sub-vanilla valve (the C# throws)
                }
            }
            if !flag2 {
                continue;
            }
            // `:13022-13023`: `num11 = Next(90, num6 - 100)` — the empty
            // band skips (the sub-vanilla valve; `Next(90, 90)` returns 90
            // without a draw, the port's `range` mirrors it).
            if num6 - 100 < 90 {
                continue;
            }
            let mut num11 = rng.range(90, num6 - 100);
            // `:13024-13040`: the ±150/±75 clear window — SKYBLOCK only.
            if skyblock {
                let mut clear = true;
                for l in (num7 - 150)..=(num7 + 150) {
                    for m in (num11 - 75)..=(num11 + 75) {
                        if world.tile(l, m).is_some_and(|t| t.active()) {
                            clear = false;
                        }
                    }
                }
                if !clear {
                    continue;
                }
            }
            // `:13042-13049`: the surface-low clamp + the errorWorld
            // deep re-roll (`Next(4) != 0`).
            if !no_surface {
                num11 = num11.min(surface_low as i32 - 50);
                if error && rng.range(0, 4) != 0 {
                    // The sub-vanilla valve: the deep re-roll band
                    // `Next(worldSurface + 50, UnderworldLayer - 50)`
                    // throws inverted (the C# never generates one) — the
                    // port keeps the clamped y instead.
                    if underworld_layer - 50 > world.world_surface as i32 + 50 {
                        num11 =
                            rng.range(world.world_surface as i32 + 50, underworld_layer - 50);
                    }
                }
            }
            // `:13050-13065`: the lake decision — the `num2 >= num` tail
            // force, then the extra `i > 4` force-then-1-in-4 arm.
            let mut flag3 = num2 >= num;
            if extra && i > 4 {
                if !flag {
                    flag3 = true;
                } else if rng.range(0, 4) == 0 {
                    flag3 = true;
                }
            }
            // `:13066-13106`: the body. The drunk (non-remix) world rolls
            // snow 3 / desert 1 variant islands (`:13075-13087`); the
            // classic/remix arm runs CloudIsland with the num10 style
            // table (`:13090-13103`).
            let mut num10 = 0;
            let drunk = world.seeds.contains(SeedFlags::DRUNK);
            let remix = world.seeds.contains(SeedFlags::REMIX);
            let anniversary = world.seeds.contains(SeedFlags::TENTH_ANNIVERSARY);
            let get_good = world.seeds.contains(SeedFlags::FOR_THE_WORTHY);
            let not_the_bees = world.seeds.contains(SeedFlags::NOT_THE_BEES);
            let space_clear = world
                .micro_seeds
                .contains(MicroSeedFlags::SURFACE_IS_IN_SPACE)
                && !error;
            if flag3 {
                flag = true;
                // The compact disc's radius draw (the M1.4-WGEN-4
                // stand-in for `CloudLake`'s `Next(100,150)` ring
                // budget).
                let radius = rng.range(40, 61);
                build_island(world, num7, num11, radius, true, rng);
                gv.sky_lake.push(true);
                gv.island_house_y.push(num11 - radius / 2);
            } else {
                gv.sky_lake.push(false);
                if drunk && !remix {
                    // `:13077-13086`: the Next(2) snow/desert split — the
                    // VERBATIM variant bodies (their own ring budgets).
                    if rng.range(0, 2) == 0 {
                        num10 = 3;
                        variant_cloud_island(world, num7, num11, true, space_clear, rng);
                    } else {
                        num10 = 1;
                        variant_cloud_island(world, num7, num11, false, space_clear, rng);
                    }
                    // The variant disc hangs down from j like the C#
                    // CloudIsland — the ledger keeps num11 verbatim.
                    gv.island_house_y.push(num11);
                } else {
                    // `:13090-13103`: the num10 style table, verbatim.
                    if remix && drunk && (!anniversary || get_good) {
                        num10 = if gv.crimson_left && num7 < world.width / 2 {
                            5
                        } else if gv.crimson_left || num7 <= world.width / 2 {
                            4
                        } else {
                            5
                        };
                    } else if !not_the_bees || remix {
                        if get_good || (remix && !anniversary) {
                            num10 = if !crimson { 4 } else { 5 };
                        } else if anniversary {
                            num10 = 6;
                        }
                    }
                    let radius = rng.range(40, 61);
                    build_island(world, num7, num11, radius, false, rng);
                    gv.island_house_y.push(num11 - radius / 2);
                }
            }
            // `:13069`/`:13074` + `:13108-13112`: the ledger writes. The
            // C# stores `num11` — the disc TOP (CloudIsland hangs down
            // from j, `:78224`'s `m > num13 ≈ j`). The port's compact
            // disc is CENTERED at num11 (the M1.4-WGEN-4 shape), so its
            // top sits at `num11 - radius/2`; the ledger stores that row
            // so the verbatim IslandHouse ground scan (`j-15`, `:79238`)
            // reaches the grass cap exactly as the C#'s reaches its disc
            // top. The VERBATIM variant bodies hang from num11 like the
            // C# — their ledger keeps num11 (the y push above).
            gv.island_house_x.push(num7);
            gv.island_style.push(num10);
            gv.num_island_houses += 1;
            num2 += 1;
            // `:13041`: `num5 = -1` — the site succeeded, leave the budget
            // loop (the `while (--num5 > 0)` test fails on the next turn).
            num5 = -1;
        }
    }
    // `:13117-13120`: the house cap on the PRE-multiplier count — the
    // discs stay, the ledger truncates (the houses pass reads it).
    if extra && gv.num_island_houses > num3 * 2 {
        gv.num_island_houses = num3 * 2;
    }
    gv.num_island_houses
}

// --- the FloatingIslandHouses pass (`:17859-17874`) → IslandHouse ---------

/// The `IslandHouse` furniture writes — the `PlaceTile` arms the house
/// uses, each with its C# gate. `SquareTileFrame` is the chain-framing
/// deviation (the module doc).
mod furniture {
    use super::*;
    use crate::tiledata as td;

    /// `WorldGen.SolidTile` (`WorldGen.cs:70111-70133`) — the house-scan
    /// predicate; the port's `edit::solid_tile` twin (OOB → false).
    pub(super) fn solid(world: &World, x: i32, y: i32) -> bool {
        crate::edit::solid_tile(world, x, y)
    }

    /// `WorldGen.SolidTile2` (`:70134-70150`) — the furniture floor gate.
    pub(super) fn solid2(world: &World, x: i32, y: i32) -> bool {
        crate::edit::solid_tile2(world, x, y)
    }

    fn put(world: &mut World, x: i32, y: i32, kind: u16, fx: i16, fy: i16) {
        if x < 0 || y < 0 || x >= world.width || y >= world.height {
            return;
        }
        let i = world.index(x, y);
        let t = &mut world.tiles[i];
        t.kind = kind;
        t.flags |= FLAG_ACTIVE;
        t.frame_x = fx;
        t.frame_y = fy;
        // The frame-important boot bits (`Main.cs:7637`/`:7641`/`:7649` —
        // the door 10, the table 14, the chair 15): the framer keeps the
        // stored frames (the temple/dungeon `mark_important` precedent).
        if world.frame_important.len() > kind as usize {
            world.frame_important[kind as usize] = true;
        }
    }

    /// `PlaceDoor` (`WorldGen.cs:31773-31803`): the ceiling/floor gate,
    /// then three rows with one `Next(3)` frame-variant roll EACH
    /// (`:31786`/`:31790`/`:31794`).
    pub(super) fn place_door(world: &mut World, x: i32, y: i32, style: i32, rng: &mut super::super::Rng) -> bool {
        let num3 = 54 * (style / 36);
        let num4 = 54 * (style % 36);
        let ceiling = world
            .tile(x, y - 2)
            .is_some_and(|t| t.active() && !t.inactive() && td::TILE_SOLID.has(t.kind));
        if !ceiling || !solid(world, x, y + 2) {
            return false;
        }
        for row in -1..=1 {
            let fx = (num3 + rng.range(0, 3) * 18) as i16;
            put(world, x, y + row, 10, fx, (num4 + (row + 1) * 18) as i16);
        }
        true
    }

    /// `PlaceTile`'s door arm (`WorldGen.cs:59843-59883`): the up-hang
    /// (two free rows under a solid ceiling) else the down-hang (two free
    /// rows over a solid floor), else the refusal.
    pub(super) fn place_tile_door(world: &mut World, x: i32, y: i32, style: i32, rng: &mut super::super::Rng) -> bool {
        let free = |w: &World, dy: i32| w.tile(x, y + dy).is_some_and(|t| !t.active());
        let solid_at = |w: &World, dy: i32| {
            w.tile(x, y + dy)
                .is_some_and(|t| t.active() && td::TILE_SOLID.has(t.kind))
        };
        // The C# branch conditions, evaluated before the mutable place.
        let up = free(world, -1) && free(world, -2) && solid_at(world, -3);
        let down = free(world, 1) && free(world, 2) && solid_at(world, 3);
        if up {
            place_door(world, x, y - 1, style, rng)
        } else if !down {
            false
        } else {
            place_door(world, x, y + 1, style, rng)
        }
    }

    /// `Place3x2`'s default path (`WorldGen.cs:52066-52195`) — the table
    /// 14 / the corrupt table 469 (`PlaceTile` case 469 → Place3x2,
    /// `:60000-60003`): the 3×2 body empty, the floor `SolidTile2` under
    /// all three columns, `frameX = 54 * style + col * 18`,
    /// `frameY = row * 18`. The 14/25 campfire-style offset and the
    /// boulder/pile vetoes never fire on the house's styles (7/5/2).
    pub(super) fn place_table(world: &mut World, x: i32, y: i32, kind: u16, style: i32) -> bool {
        if x < 5 || x > world.width - 5 || y < 5 || y > world.height - 5 {
            return false;
        }
        let mut ok = true;
        for col in (x - 1)..(x + 2) {
            for row in (y - 1)..(y + 1) {
                if world.tile(col, row).is_some_and(|t| t.active()) {
                    ok = false;
                }
            }
            if !solid2(world, col, y + 1) {
                ok = false;
            }
        }
        if !ok {
            return false;
        }
        for col in 0..3 {
            for row in 0..2 {
                put(
                    world,
                    x - 1 + col,
                    y - 1 + row,
                    kind,
                    (54 * style + col * 18) as i16,
                    (row * 18) as i16,
                );
            }
        }
        true
    }

    /// `Place1x2` (`WorldGen.cs:46652-46679`) — the chair 15: the floor
    /// `SolidTile2`, the top cell free, `frameY = 40 * style + row * 18`,
    /// `frameX = 0` (the sapling roll is kind-20 only).
    pub(super) fn place_chair(world: &mut World, x: i32, y: i32, style: i32) -> bool {
        if !solid2(world, x, y + 1) || world.tile(x, y - 1).is_some_and(|t| t.active()) {
            return false;
        }
        for row in 0..2 {
            put(world, x, y - 1 + row, 15, 0, (40 * style + row * 18) as i16);
        }
        true
    }

    /// `Place2xX` (`WorldGen.cs:39663-39722`) for the fountain 207
    /// (`num3 = 4`, `:39684-39687`): the 2×4 body empty, the floor
    /// `SolidTile2` under both columns, `frameX = 36*style + col*18`,
    /// `frameY = row*18`. The 105/104 statue/clock arms stay with their
    /// callers (the temple precedent).
    pub(super) fn place_fountain(world: &mut World, x: i32, y: i32, style: i32) -> bool {
        let num3 = 4;
        let mut ok = true;
        for row in (y - num3 + 1)..(y + 1) {
            for col in x..(x + 2) {
                if world.tile(col, row).is_some_and(|t| t.active()) {
                    ok = false;
                }
            }
        }
        if !solid2(world, x, y + 1) || !solid2(world, x + 1, y + 1) {
            ok = false;
        }
        if !ok {
            return false;
        }
        for row in 0..num3 {
            for col in 0..2 {
                put(
                    world,
                    x + col,
                    y - num3 + 1 + row,
                    207,
                    (36 * style + col * 18) as i16,
                    (row * 18) as i16,
                );
            }
        }
        true
    }

    /// `SwitchFountain` (`WorldGen.cs:51150-51197`): walk the frame back
    /// to the fountain's top-left cell, then flip `frameY` by ±72 over
    /// the 2×4 body (the on/off state). The `Wiring.running` SkipWire
    /// tail + `NetMessage` never fire during worldgen.
    pub(super) fn switch_fountain(world: &mut World, i: i32, j: i32) {
        let Some(anchor) = world.tile(i, j) else {
            return;
        };
        let mut num3 = anchor.frame_x / 18;
        while num3 >= 2 {
            num3 -= 2;
        }
        let mut num4 = anchor.frame_y / 18;
        if num4 >= 4 {
            num4 -= 4;
        }
        let (num, num2) = (i - num3 as i32, j - num4 as i32);
        for k in num..(num + 2) {
            for l in num2..(num2 + 4) {
                if let Some(t) = world.tile_mut(k, l) {
                    if t.active() && t.kind == 207 {
                        if t.frame_y < 72 {
                            t.frame_y += 72;
                        } else {
                            t.frame_y -= 72;
                        }
                    }
                }
            }
        }
    }
}

/// `IslandHouse` (`WorldGen.cs:79203-79484`), ALL arms: the dungeon-wall
/// abort, the drunk crimson-half split (`:79217-79220`), the door-side
/// flip + the size rolls, the ground scan, the sunplate shell + the
/// disc-wall interior, the door carve + the remix 5/38 / classic 9 door
/// (`:79323-79337`), the opposite wall column, the chest (the S1-A-4
/// cycle), the fountain scatter + SwitchFountain (islandStyle > 0,
/// `:79377-79392`), the window strips + banners under the classic /
/// anniversary-not-getGood gate, the remix evil-flavour table/chairs
/// (`:79421-79435`), the style-1 palms (`:79462-79483`). The chest
/// placement rides the FULL `buried_chest::add_buried_chest` core (the
/// d103 salvage graft, research/87 §KEEP-3) + the `stock_placed_chest`
/// hook — the down-scan vetoes (shimmer/larva-231/±2 boulder+altar),
/// the flag12 remix evil retype (`:36176-36196`) and the errorWorld
/// style shuffle all run at the house's call, exactly the C#.
#[allow(clippy::too_many_arguments)]
fn island_house(
    world: &mut World,
    i: i32,
    j: i32,
    island_style: i32,
    rng: &mut super::Rng,
    mut stock: Option<&mut super::chest_stock::ChestStockHook<'_>>,
    get_good: bool,
    drunk: bool,
    remix: bool,
    anniversary: bool,
    crimson: bool,
    crimson_left: bool,
    sky_count: &mut u32,
    gv_hell: &mut usize,
) {
    // `:79205-79216`: the ±30 dungeon-wall abort (`Main.wallDungeon` +
    // the temple wall 87 + the evil walls 3/83).
    for k in (i - 30)..=(i + 30) {
        for l in (j - 30)..=(j + 30) {
            if let Some(t) = world.tile(k, l) {
                if super::dungeon_features::is_dungeon_wall(t.wall)
                    || t.wall == 87
                    || t.wall == 3
                    || t.wall == 83
                {
                    return;
                }
            }
        }
    }
    // `:79217-79220`: the drunk half-split evil flavour — the remix
    // furniture arms read it.
    let mut flag = crimson;
    if drunk {
        flag = (crimson_left && i < world.width / 2)
            || (!crimson_left && i > world.width / 2);
    }
    // `:79224-79230`: the door-side flip + the size rolls (draw order).
    let num2 = if rng.range(0, 2) == 0 { -1 } else { 1 };
    let num3 = rng.range(7, 12);
    let num4 = rng.range(5, 7);
    // `:79231-79247`: the ground scan at the door-side column — the first
    // row in [j-15, j+30) with a wall, active and solid (the leaf/wood
    // kinds excluded). Below the surface line the C# takes j-5 flat; the
    // port's islands sit above it (the y clamp), so the scan is the arm
    // that runs — the j-5 branch is kept for the errorWorld deep islands.
    let scan_x = i + (num3 + 2) * num2;
    let mut ground_y = None;
    if j as f64 > world.world_surface {
        ground_y = Some(j - 5);
    } else {
        for m in (j - 15)..(j + 30) {
            let kind_ok = world.tile(scan_x, m).is_some_and(|t| {
                t.wall > 0
                    && t.active()
                    && t.kind != 192
                    && t.kind != 191
                    && t.kind != 384
            });
            if kind_ok && furniture::solid(world, scan_x, m) {
                ground_y = Some(m - 1);
                break;
            }
        }
    }
    // The C# leaves `vector2D.Y` at 0 when the scan finds nothing (the
    // house builds at the world top — a degenerate case the port's
    // discs never produce: the wall flood guarantees a hit). The port
    // skips instead of building at row 0 (the sub-vanilla valve).
    let Some(vy) = ground_y else {
        return;
    };
    // `:79249-79283`: the outer sunplate shell (the top corners skipped).
    let (num5, num6) = (i - num3 - 1, i + num3 + 1);
    let (num7, num8) = (vy - num4 - 1, vy + 2);
    for n in num5..=num6 {
        for y in (num7 - 1)..(num8 + 1) {
            if y == num7 - 1 && (n == num5 || n == num6) {
                continue;
            }
            if n < 0 || y < 0 || n >= world.width || y >= world.height {
                continue;
            }
            let idx = world.index(n, y);
            let t = &mut world.tiles[idx];
            t.kind = TILE_SUNPLATE;
            t.flags |= FLAG_ACTIVE;
            t.liquid = 0;
            t.wall = 0;
            t.flags &= !FLAG_HALF_BRICK;
            t.set_slope(0);
        }
    }
    // `:79284-79314`: the interior carve — the disc wall where the cell
    // carried no wall (the top corners skipped).
    let (ix0, ix1) = (i - num3, i + num3);
    let (iy0, iy1) = (vy - num4, vy + 1);
    for n in ix0..=ix1 {
        for y in iy0..iy1 {
            if y == iy0 && (n == ix0 || n == ix1) {
                continue;
            }
            if n < 0 || y < 0 || n >= world.width || y >= world.height {
                continue;
            }
            let idx = world.index(n, y);
            if world.tiles[idx].wall == 0 {
                let t = &mut world.tiles[idx];
                t.flags &= !FLAG_ACTIVE;
                t.wall = WALL_DISC;
            }
        }
    }
    // `:79315-79337`: the door carve (5 wide, 3 tall) + the door — the
    // remix (!anniversary || getGood) arm splits crimson 5 / corrupt 38
    // (`:79323-79332`), else the classic 9 (`:79336`).
    let mut num12 = i + (num3 + 1) * num2;
    let num13 = vy;
    for n in (num12 - 2)..=(num12 + 2) {
        for dy in 0..3 {
            if n < 0 || num13 - dy < 0 || n >= world.width || num13 - dy >= world.height {
                continue;
            }
            let idx = world.index(n, num13 - dy);
            world.tiles[idx].flags &= !FLAG_ACTIVE;
        }
    }
    let door_style = if remix && (!anniversary || get_good) {
        if flag { 5 } else { 38 }
    } else {
        9
    };
    furniture::place_tile_door(world, num12, num13, door_style, rng);
    // `:79338-79347`: the opposite wall column.
    num12 = i + (num3 + 1) * -num2 - num2;
    for y in iy0..=(iy1 + 1) {
        if num12 < 0 || y < 0 || num12 >= world.width || y >= world.height {
            continue;
        }
        let idx = world.index(num12, y);
        let t = &mut world.tiles[idx];
        t.kind = TILE_SUNPLATE;
        t.flags |= FLAG_ACTIVE;
        t.liquid = 0;
        t.wall = 0;
        t.flags &= !FLAG_HALF_BRICK;
        t.set_slope(0);
    }
    // `:79348-79376`: the chest — the `skyIslandHouseCount` cycle (the
    // `> 3 → Next(4)` re-roll), the style 13 / getGood 2, the anchor
    // `(i, num13 - 3)` through the full AddBuriedChest core.
    let mut num16 = *sky_count;
    if num16 > 3 {
        num16 = rng.range(0, 4) as u32;
    }
    let main = match num16 {
        0 => 159,
        1 => 65,
        2 => 158,
        _ => 2219,
    };
    let chest_style = if get_good { 2 } else { 13 };
    // `AddBuriedChest(i, num13 - 3, mainItemInChest, false, 13/2, false, 0)`
    // (`:79371`/`:79375`) through the FULL core (`:35877-36217` — the
    // research/87 §KEEP-3 salvage graft): the down-scan with the
    // shimmer/larva-231 aborts, the ±2 boulder/altar-26/237 exclusion
    // (fluff 100), the flag15 default, the errorWorld style shuffle and
    // the flag12 remix evil retype (crimson 43 wood / corrupt 3 mahogany
    // 467, `:36176-36196`) — the arms the compact stand-in omitted. On a
    // fresh sunplate island floor the vetoes are inert and the scan lands
    // on the floor row `num13 + 1`; the 2×2 occupies rows `num13 - 1`/
    // `num13` (the core returns the top-left `(i - 1, k - 2)`).
    let seed_flags = super::buried_chest::ChestSeedFlags {
        remix,
        get_good,
        drunk,
        anniversary,
        not_the_bees: world.seeds.contains(SeedFlags::NOT_THE_BEES),
        dont_starve: world.seeds.contains(SeedFlags::DONT_STARVE),
        no_traps: world.seeds.contains(SeedFlags::NO_TRAPS),
        error_world: world.micro_seeds.contains(MicroSeedFlags::ERROR_WORLD),
        crimson,
        crimson_left,
    };
    // `GenVars.desertHiveHigh/Low`'s Reset sentinels (`:11147-11148`:
    // maxTilesY / 0) until the hive bounds land; the desert arm never
    // fires on the sky band (main > 0, above the surface) either way.
    let placed = super::buried_chest::add_buried_chest(
        world,
        i,
        num13 - 3,
        main,
        false,
        chest_style,
        false,
        0,
        world.world_surface,
        world.rock_layer,
        seed_flags,
        world.height,
        0,
        gv_hell,
        5,
        rng,
    );
    if let Some(p) = placed {
        super::chest_stock::stock_placed_chest(
            world,
            stock.as_deref_mut(),
            rng,
            p.x,
            p.y,
            p.style,
            p.kind,
            main,
            false,
        );
    }
    // `:79377-79392`: the fountain scatter (islandStyle > 0 — the drunk
    // snow 3 / desert 1 islands and the remix/getGood styles 4/5/6): the
    // 100000-attempt search, the first 50000 refuse sunplate cells, then
    // Place2xX(207, islandStyle) + SwitchFountain on the first landing.
    if island_style > 0 {
        for num17 in 0..100000 {
            let num18 = i + rng.range(-50, 51);
            let num19 = num13 + rng.range(0, 21);
            let cell = world.tile(num18, num19);
            let kind_here = cell.map_or(0, |t| t.kind);
            let active_here = cell.is_some_and(|t| t.active());
            if (num17 >= 50000 || kind_here != TILE_SUNPLATE) && !active_here {
                furniture::place_fountain(world, num18, num19, island_style);
                if world.tile(num18, num19).is_some_and(|t| t.active()) {
                    furniture::switch_fountain(world, num18, num19);
                    break;
                }
            }
        }
    }
    // `:79394`: the counter increments once per house, after the place.
    *sky_count += 1;
    // `:79395-79419`: the wall-21 window strips — the classic /
    // anniversary-without-getGood arm (`:79395`).
    if !remix || (anniversary && !get_good) {
        let num20 = i - num3 / 2 + 1;
        let num21 = i + num3 / 2 - 1;
        let num22 = if num3 > 10 { 2 } else { 1 };
        let num23 = (iy0 + iy1) / 2 - 1;
        for n in (num20 - num22)..=(num20 + num22) {
            for y in (num23 - 1)..=(num23 + 1) {
                if n >= 0 && y >= 0 && n < world.width && y < world.height {
                    let idx = world.index(n, y);
                    world.tiles[idx].wall = WALL_GLASS;
                }
            }
        }
        for n in (num21 - num22)..=(num21 + num22) {
            for y in (num23 - 1)..=(num23 + 1) {
                if n >= 0 && y >= 0 && n < world.width && y < world.height {
                    let idx = world.index(n, y);
                    world.tiles[idx].wall = WALL_GLASS;
                }
            }
        }
    }
    // `:79420-79443`: the table + the two chairs — the remix
    // (!anniversary || getGood) arm takes crimson table 14 style 5 +
    // chairs 15 style 8, else the corrupt table 469 style 2 + chairs 38
    // (`:79421-79435`); the classic arm keeps 14/7 + 15/10
    // (`:79438-79440`).
    let num28 = i + (num3 / 2 + 1) * -num2;
    let (table_kind, table_style, chair_style) =
        if remix && (!anniversary || get_good) {
            if flag { (14u16, 5, 8) } else { (469, 2, 38) }
        } else {
            (14, 7, 10)
        };
    furniture::place_table(world, num28, iy1 - 1, table_kind, table_style);
    furniture::place_chair(world, num28 - 2, iy1 - 1, chair_style);
    furniture::place_chair(world, num28 + 2, iy1 - 1, chair_style);
    // `:79442-79443`: the left chair's frameX flip — UNCONDITIONAL in the
    // C# (it adds 18 to whatever sits at the two cells, even a failed
    // chair place).
    for dy in 0..2 {
        let (x, y) = (num28 - 2, iy1 - 1 - dy);
        if x >= 0 && y >= 0 && x < world.width && y < world.height {
            let idx = world.index(x, y);
            world.tiles[idx].frame_x += 18;
        }
    }
    // `:79444-79461`: the three banners (`Next(7,10)` styles) — the same
    // classic / anniversary gate as the windows (`:79444`).
    if !remix || (anniversary && !get_good) {
        super::dungeon_features::place_banner(world, ix0 + 1, iy0, 91, rng.range(7, 10));
        super::dungeon_features::place_banner(world, ix1 - 1, iy0, 91, rng.range(7, 10));
        let (bx, by) = if num2 > 0 { (ix0, iy0 + 1) } else { (ix1, iy0 + 1) };
        super::dungeon_features::place_banner(world, bx, by, 91, rng.range(7, 10));
    }
    // `:79462-79483`: the desert island's palms (islandStyle == 1): the
    // Next(3,6) budget, the 100000-attempt scatter, TryGrowingTreeByType
    // (323) = GrowPalmTree(x, y+1, 0, false) — the shared ocean port.
    if island_style != 1 {
        return;
    }
    let mut num29 = rng.range(3, 6);
    for _num30 in 0..100000 {
        let num31 = i + rng.range(-50, 51);
        let num32 = num13 + rng.range(-10, 21);
        if !world.tile(num31, num32).is_some_and(|t| t.active()) {
            super::ocean::grow_palm_tree(world, num31, num32 + 1, 0, false, rng);
            if world.tile(num31, num32).is_some_and(|t| t.active()) {
                num29 -= 1;
            }
        }
        if num29 <= 0 {
            break;
        }
    }
}

/// The `FloatingIslandHouses` pass (`WorldGen.cs:17859-17874`): a house
/// on every non-lake island of the ledger, in placement order. The
/// `Skyblock.denyFloatingIslands` gate (`:17861`) matches. The ledger
/// rides a snapshot (the C# reads the GenVars statics while `IslandHouse`
/// mutates only `skyIslandHouseCount` — the port's `&mut GenVars` carries
/// that counter back the same way). The seed flags ride the world
/// (`Reset` `:11115-11123`) + the caller's crimson (`WorldGen.crimson`,
/// the `opts.crimson` twin).
pub(crate) fn pass_floating_island_houses(
    world: &mut World,
    gv: &mut GenVars,
    rng: &mut super::Rng,
    mut stock: Option<&mut super::chest_stock::ChestStockHook<'_>>,
    crimson: bool,
) {
    if deny_floating_islands(world) {
        return;
    }
    let get_good = world.seeds.contains(SeedFlags::FOR_THE_WORTHY);
    let drunk = world.seeds.contains(SeedFlags::DRUNK);
    let remix = world.seeds.contains(SeedFlags::REMIX);
    let anniversary = world.seeds.contains(SeedFlags::TENTH_ANNIVERSARY);
    let crimson_left = gv.crimson_left;
    let count = gv.num_island_houses as usize;
    let ledger: Vec<(i32, i32, i32, bool)> = (0..count)
        .map(|idx| {
            (
                gv.island_house_x[idx],
                gv.island_house_y[idx],
                gv.island_style[idx],
                gv.sky_lake[idx],
            )
        })
        .collect();
    for (x, y, style, lake) in ledger {
        // `:17868-17871`: the lake skip.
        if lake {
            continue;
        }
        island_house(
            world,
            x,
            y,
            style,
            rng,
            stock.as_deref_mut(),
            get_good,
            drunk,
            remix,
            anniversary,
            crimson,
            crimson_left,
            &mut gv.sky_island_house_count,
            &mut gv.hell_chest,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::worldgen::noise::Rng;

    /// A wide sky world with the ground far below the island band: the
    /// column scan (`:13010-13018`) finds the ground row 520, the y roll
    /// `Next(90, 420)` stays well above it.
    fn sky_world(width: i32, height: i32, ground: i32) -> World {
        let mut w = World::new("fi".into(), width, height);
        for x in 0..width {
            for y in ground..height {
                let i = w.index(x, y);
                w.tiles[i].kind = 1;
                w.tiles[i].flags |= FLAG_ACTIVE;
            }
        }
        w.world_surface = (ground + 25) as f64;
        w
    }

    fn gv_with_lakes(width: i32) -> GenVars {
        let mut gv = GenVars::default();
        // The Reset skyLakes block (`WorldGen.cs:11182-11190`).
        gv.sky_lakes = 1;
        if width > 8000 {
            gv.sky_lakes += 1;
        }
        if width > 6000 {
            gv.sky_lakes += 1;
        }
        gv
    }

    fn count_kind(world: &World, kind: u16) -> usize {
        world
            .tiles
            .iter()
            .filter(|t| t.active() && t.kind == kind)
            .count()
    }

    /// A guaranteed house ground: a wide solid dirt slab carrying the
    /// cloud wall, so `IslandHouse`'s ground scan (`:79238-79247`) always
    /// finds `wall > 0 && active && SolidTile` regardless of the compact
    /// disc's exact cap geometry. The slab top row is `top`; the scan
    /// lands on it and the house floor sits at `top`.
    fn house_ground(w: &mut World, cx: i32, top: i32) {
        for x in (cx - 40)..=(cx + 40) {
            for y in top..=(top + 6) {
                let i = w.index(x, y);
                w.tiles[i].kind = 0; // dirt — solid, non-solid-top
                w.tiles[i].flags |= FLAG_ACTIVE;
                w.tiles[i].wall = WALL_CLOUD;
            }
        }
    }

    /// The count formula + the skyLakes budget (`:12934`, `:11182-11190`,
    /// `:12955`): 4200 → 3+1, 6400 → 5+2, 8400 → 6+3 islands, the tail
    /// `skyLakes` of them lakes (`num2 >= num`, `:13051`).
    #[test]
    fn the_count_formula_and_the_sky_lakes_tail() {
        for (width, dry, lakes) in [(4200i32, 3usize, 1usize), (6400, 5, 2), (8400, 6, 3)] {
            let mut w = sky_world(width, 1200, 520);
            let mut gv = gv_with_lakes(width);
            let placed =
                pass_floating_islands(&mut w, &mut gv, &mut Rng(42), 520.0, false);
            assert_eq!(placed as usize, dry + lakes, "width {width}");
            assert_eq!(gv.sky_lake.len(), dry + lakes);
            // The FIRST `num` islands are dry, the tail `skyLakes` lakes.
            for (idx, lake) in gv.sky_lake.iter().enumerate() {
                assert_eq!(*lake, idx >= dry, "width {width} island {idx}");
            }
        }
    }

    /// The errorWorld triple (`:12936-12939`): a lone errorWorld seed has
    /// `errorWorldAdjustment(1.0) = 1 < 3` → `num *= 3` BEFORE `num3`, so
    /// the 4200 world attempts 9+1 islands and the house cap base triples.
    #[test]
    fn the_error_world_triple_carries_into_the_cap_base() {
        let mut w = sky_world(4200, 1200, 520);
        w.micro_seeds |= MicroSeedFlags::ERROR_WORLD;
        assert_eq!(error_world_adjustment(1.0, &w), 1);
        let mut gv = gv_with_lakes(4200);
        pass_floating_islands(&mut w, &mut gv, &mut Rng(7), 520.0, false);
        // 9 dry + 1 lake attempted; the deep y re-roll may bury some, but
        // every placed island past index 8 is a lake (the num2 >= 9 tail).
        assert!(gv.sky_lake.len() >= 2, "the tripled count places islands");
        let dry = gv.sky_lake.iter().filter(|l| !**l).count();
        assert!(dry <= 9, "at most the tripled num dry islands");
        // The errorWorld spawn-exclusion skip (`:12974`): an island MAY
        // sit over spawn — the classic control never does (below).
    }

    /// The spawn exclusion (`:12969-12979`): the classic 150 half-width
    /// re-draw keeps every anchor out of the spawn band; the extra flag
    /// narrows it to 100.
    #[test]
    fn the_spawn_exclusion_keeps_the_anchors_out() {
        let mut w = sky_world(4200, 1200, 520);
        let mut gv = gv_with_lakes(4200);
        pass_floating_islands(&mut w, &mut gv, &mut Rng(42), 520.0, false);
        assert!(!gv.island_house_x.is_empty());
        for x in &gv.island_house_x {
            assert!(
                *x <= 2100 - 150 || *x >= 2100 + 150,
                "anchor {x} inside the classic spawn band"
            );
        }
    }

    /// The spacing (`:12981-12993`): strict bounds — two anchors exactly
    /// 180 apart are legal, 179 are not. The classic run never places a
    /// pair closer than 180.
    #[test]
    fn the_anchor_spacing_is_strict() {
        let mut w = sky_world(4200, 1200, 520);
        let mut gv = gv_with_lakes(4200);
        pass_floating_islands(&mut w, &mut gv, &mut Rng(99), 520.0, false);
        for a in 0..gv.island_house_x.len() {
            for b in (a + 1)..gv.island_house_x.len() {
                let d = (gv.island_house_x[a] - gv.island_house_x[b]).abs();
                assert!(d >= 180, "anchors {a}/{b} only {d} apart");
            }
        }
    }

    /// The extraFloatingIslands multipliers (`:12942-12954`) + the house
    /// cap (`:13117-13120`): the 10x arm attempts 30+10 islands, the
    /// ledger truncates to `num3 * 2 = 6`; the crowded 2x arm attempts
    /// 6+2 and caps at 6 too; the skyblock passthrough keeps 3+1.
    #[test]
    fn the_multipliers_and_the_post_loop_house_cap() {
        let mut w = sky_world(4200, 1200, 520);
        w.micro_seeds |= MicroSeedFlags::EXTRA_FLOATING_ISLANDS;
        let mut gv = gv_with_lakes(4200);
        pass_floating_islands(&mut w, &mut gv, &mut Rng(42), 520.0, false);
        assert_eq!(gv.sky_lakes, 10, "the 10x arm multiplies skyLakes");
        assert!(gv.island_house_x.len() > 6, "the discs keep placing");
        assert_eq!(gv.num_island_houses, 6, "the cap is num3 * 2");

        let mut crowded = sky_world(4200, 1200, 520);
        crowded.micro_seeds |= MicroSeedFlags::EXTRA_FLOATING_ISLANDS
            | MicroSeedFlags::PAINT_EVERYTHING_GRAY
            | MicroSeedFlags::ERROR_WORLD
            | MicroSeedFlags::RAINS_FOR_A_YEAR
            | MicroSeedFlags::RANDOM_SPAWN
            | MicroSeedFlags::ADD_TELEPORTERS;
        assert_eq!(
            crate::seeds::active_secret_seed_count(crowded.micro_seeds, crowded.seeds),
            6
        );
        let mut gv2 = gv_with_lakes(4200);
        pass_floating_islands(&mut crowded, &mut gv2, &mut Rng(42), 520.0, false);
        assert_eq!(gv2.sky_lakes, 2, "the reduced 2x arm");

        let mut skyblock = sky_world(4200, 1200, 520);
        skyblock.micro_seeds |= MicroSeedFlags::EXTRA_FLOATING_ISLANDS;
        skyblock.seeds |= SeedFlags::SKYBLOCK;
        let mut gv3 = gv_with_lakes(4200);
        pass_floating_islands(&mut skyblock, &mut gv3, &mut Rng(42), 520.0, false);
        assert_eq!(gv3.sky_lakes, 1, "the skyblock passthrough keeps 1");
        // The unmultiplied 3+1 attempt budget — the ±150/±75 clear window
        // may reject sites, so the placed count is at most 4.
        assert!(gv3.island_house_x.len() <= 4, "3 + 1, unmultiplied");
    }

    /// The pass gates (`:12929`): noSurface alone skips the whole pass
    /// without a draw; skyblock without extra denies it; noSurface WITH
    /// extra runs (the deep y arm, `:12998-13001`).
    #[test]
    fn the_deny_gates_and_the_no_surface_arm() {
        let mut w = sky_world(4200, 1200, 520);
        w.micro_seeds |= MicroSeedFlags::NO_SURFACE;
        let mut gv = gv_with_lakes(4200);
        let mut rng = Rng(42);
        let before = rng.0;
        assert_eq!(pass_floating_islands(&mut w, &mut gv, &mut rng, 520.0, false), 0);
        assert_eq!(rng.0, before, "the gated pass draws nothing");

        let mut sb = sky_world(4200, 1200, 520);
        sb.seeds |= SeedFlags::SKYBLOCK;
        let mut gv2 = gv_with_lakes(4200);
        assert_eq!(
            pass_floating_islands(&mut sb, &mut gv2, &mut Rng(42), 520.0, false),
            0,
            "skyblock without extra denies the islands"
        );

        let mut deep = sky_world(4200, 1200, 520);
        deep.micro_seeds |= MicroSeedFlags::NO_SURFACE | MicroSeedFlags::EXTRA_FLOATING_ISLANDS;
        let mut gv3 = gv_with_lakes(4200);
        pass_floating_islands(&mut deep, &mut gv3, &mut Rng(42), 520.0, false);
        assert!(!gv3.island_house_y.is_empty(), "noSurface+extra runs");
        // The deep roll `Next(200, UnderworldLayer - 200)` = Next(200, 800).
        for y in &gv3.island_house_y {
            assert!((200..800).contains(y), "the noSurface y {y} is deep");
        }
    }

    /// The houses pass (`:17859-17874`): a house on every non-lake island,
    /// the sunplate 202 frame + the disc wall 82 interior, the chest cycle
    /// walking 159/65/158/2219 with the `> 3 → Next(4)` re-roll, the
    /// counter advancing once per house.
    #[test]
    fn the_houses_pass_builds_sunplate_and_walks_the_chest_cycle() {
        let mut w = sky_world(4200, 1200, 520);
        let mut gv = GenVars::default();
        // Five dry islands + one lake, hand-ledgered over built discs.
        let xs = [600i32, 1000, 1400, 1800, 2600, 3000];
        for (idx, x) in xs.iter().enumerate() {
            build_island(&mut w, *x, 200, 50, idx == 5, &mut Rng(5));
            house_ground(&mut w, *x, 205);
            gv.island_house_x.push(*x);
            gv.island_house_y.push(200);
            gv.island_style.push(0);
            gv.sky_lake.push(idx == 5);
        }
        gv.num_island_houses = 6;
        let mut mains: Vec<u16> = Vec::new();
        let mut styles: Vec<i32> = Vec::new();
        {
            let mut hook = |ev: &super::super::chest_stock::ChestSiteEvent,
                            _rng: &mut dyn super::super::chest_stock::WorldGenRand|
             -> Vec<crate::save::ChestItem> {
                mains.push(ev.main_item);
                styles.push(ev.chest_style);
                Vec::new()
            };
            pass_floating_island_houses(
                &mut w,
                &mut gv,
                &mut Rng(11),
                Some(&mut hook),
                false,
            );
        }
        // Five houses (the lake island carries none), the cycle in order;
        // the fifth re-rolls Next(4) — deterministic on the fixed seed.
        assert_eq!(mains.len(), 5, "the lake island has no house");
        assert_eq!(&mains[..4], &[159, 65, 158, 2219]);
        assert!([159, 65, 158, 2219].contains(&mains[4]));
        assert!(styles.iter().all(|s| *s == 13), "the classic style 13");
        assert_eq!(gv.sky_island_house_count, 5, "once per house");
        assert!(count_kind(&w, TILE_SUNPLATE) > 100, "the sunplate shells");
        assert!(count_kind(&w, 9) == 0, "no planks — the old box is gone");
        let disc_walls = w.tiles.iter().filter(|t| t.wall == WALL_DISC).count();
        assert!(disc_walls > 50, "the disc-wall interiors");
        // The lake island's disc carries no sunplate above its centre.
        let lake_house = (2950..=3050)
            .any(|x| (150..=210).any(|y| w.tile(x, y).is_some_and(|t| t.active() && t.kind == TILE_SUNPLATE)));
        assert!(!lake_house, "the lake island carries no house");
    }

    /// The getGood chest style (`:79369-79372`): style 2 under
    /// FOR_THE_WORTHY, 13 otherwise.
    #[test]
    fn the_get_good_world_takes_chest_style_2() {
        let mut w = sky_world(4200, 1200, 520);
        w.seeds |= SeedFlags::FOR_THE_WORTHY;
        let mut gv = GenVars::default();
        build_island(&mut w, 1000, 200, 50, false, &mut Rng(5));
        house_ground(&mut w, 1000, 205);
        gv.island_house_x.push(1000);
        gv.island_house_y.push(200);
        gv.island_style.push(0);
        gv.sky_lake.push(false);
        gv.num_island_houses = 1;
        let mut styles: Vec<i32> = Vec::new();
        {
            let mut hook = |ev: &super::super::chest_stock::ChestSiteEvent,
                            _rng: &mut dyn super::super::chest_stock::WorldGenRand|
             -> Vec<crate::save::ChestItem> {
                styles.push(ev.chest_style);
                Vec::new()
            };
            pass_floating_island_houses(&mut w, &mut gv, &mut Rng(11), Some(&mut hook), false);
        }
        assert_eq!(styles, vec![2]);
    }

    /// The dungeon-wall abort (`:79207-79216`): a dungeon wall within ±30
    /// of the anchor refuses the whole house.
    #[test]
    fn the_dungeon_wall_aborts_the_house() {
        let mut w = sky_world(4200, 1200, 520);
        let mut gv = GenVars::default();
        build_island(&mut w, 1000, 200, 50, false, &mut Rng(5));
        let i = w.index(1020, 210);
        w.tiles[i].wall = 7; // a base dungeon wall inside the ±30 box
        gv.island_house_x.push(1000);
        gv.island_house_y.push(200);
        gv.island_style.push(0);
        gv.sky_lake.push(false);
        gv.num_island_houses = 1;
        pass_floating_island_houses(&mut w, &mut gv, &mut Rng(11), None, false);
        assert_eq!(count_kind(&w, TILE_SUNPLATE), 0, "the abort builds nothing");
        assert_eq!(gv.sky_island_house_count, 0, "the counter never advances");
    }

    /// The `a == b` determinism witness: the two passes over the same seed
    /// produce byte-identical worlds (the world-hash fold), classic and
    /// extraFloatingIslands alike — the RNG world-pass slots add no
    /// entropy source.
    #[test]
    fn the_island_passes_are_reproducible_for_a_seed() {
        for extra in [false, true] {
            let run = || {
                let mut w = sky_world(4200, 1200, 520);
                if extra {
                    w.micro_seeds |= MicroSeedFlags::EXTRA_FLOATING_ISLANDS;
                }
                let mut gv = gv_with_lakes(4200);
                pass_floating_islands(&mut w, &mut gv, &mut Rng(42), 520.0, false);
                pass_floating_island_houses(&mut w, &mut gv, &mut Rng(42), None, false);
                super::super::world_hash::world_hash(&w)
            };
            let a = run();
            let b = run();
            assert_eq!(a, b, "extra={extra}: the island passes must be deterministic");
        }
    }

    /// The drunk variant split (`:13075-13087`): a drunk (non-remix)
    /// world's dry islands carry style 3 (snow) or 1 (desert) — never 0
    /// — and the variant bodies write their surface kinds (snow 147 /
    /// sand 53) plus the variant blob kinds (snow cloud 460 / rain
    /// cloud 196). The ledger y keeps `num11` (the bodies hang down).
    #[test]
    fn the_drunk_world_splits_snow_and_desert_islands() {
        let mut w = sky_world(4200, 1200, 520);
        w.seeds |= SeedFlags::DRUNK;
        let mut gv = gv_with_lakes(4200);
        pass_floating_islands(&mut w, &mut gv, &mut Rng(42), 520.0, false);
        let dry_styles: Vec<i32> = gv
            .island_style
            .iter()
            .zip(&gv.sky_lake)
            .filter(|(_, lake)| !**lake)
            .map(|(s, _)| *s)
            .collect();
        assert!(!dry_styles.is_empty(), "the drunk world places dry islands");
        assert!(
            dry_styles.iter().all(|s| *s == 3 || *s == 1),
            "every dry island is snow 3 or desert 1: {dry_styles:?}"
        );
        assert!(count_kind(&w, 147) > 0 || count_kind(&w, 53) > 0, "the variant surfaces");
        assert!(count_kind(&w, 460) > 0 || count_kind(&w, 196) > 0, "the variant blobs");
        // The drunk+remix world takes the CloudIsland style-table arm
        // instead (the `!remix` gate, `:13075`).
        let mut wr = sky_world(4200, 1200, 520);
        wr.seeds |= SeedFlags::DRUNK | SeedFlags::REMIX;
        let mut gvr = gv_with_lakes(4200);
        gvr.crimson_left = false;
        pass_floating_islands(&mut wr, &mut gvr, &mut Rng(42), 520.0, false);
        for (s, lake) in gvr.island_style.iter().zip(&gvr.sky_lake) {
            if !lake {
                assert!(*s == 4 || *s == 5, "the drunk+remix split is 4/5: {s}");
            }
        }
    }

    /// The num10 style table (`:13090-13103`): getGood alone → 4
    /// (corruption) / 5 (crimson); anniversary alone → 6; notTheBees
    /// alone → 0; remix alone → 4/5; remix+anniversary (no getGood) →
    /// the anniversary 6 falls through the inner gates.
    #[test]
    fn the_num10_style_table_arms() {
        let cases: [(u16, bool, &[i32]); 5] = [
            (SeedFlags::FOR_THE_WORTHY, false, &[4]),
            (SeedFlags::FOR_THE_WORTHY, true, &[5]),
            (SeedFlags::TENTH_ANNIVERSARY, false, &[6]),
            (SeedFlags::NOT_THE_BEES, false, &[0]),
            (SeedFlags::REMIX, true, &[5]),
        ];
        for (seed, crimson, expected) in cases {
            let mut w = sky_world(4200, 1200, 520);
            w.seeds |= seed;
            let mut gv = gv_with_lakes(4200);
            pass_floating_islands(&mut w, &mut gv, &mut Rng(42), 520.0, crimson);
            let dry: Vec<i32> = gv
                .island_style
                .iter()
                .zip(&gv.sky_lake)
                .filter(|(_, lake)| !**lake)
                .map(|(s, _)| *s)
                .collect();
            assert!(!dry.is_empty(), "seed {seed} places dry islands");
            for s in dry {
                assert!(expected.contains(&s), "seed {seed} crimson {crimson}: style {s}");
            }
        }
        // remix + anniversary without getGood (no drunk): the inner
        // `getGood || (remix && !anniversary)` gate reads false, the
        // `else if anniversary` arm fires → num10 = 6 (`:13100-13103`).
        let mut w = sky_world(4200, 1200, 520);
        w.seeds |= SeedFlags::REMIX | SeedFlags::TENTH_ANNIVERSARY;
        let mut gv = gv_with_lakes(4200);
        pass_floating_islands(&mut w, &mut gv, &mut Rng(42), 520.0, false);
        for (s, lake) in gv.island_style.iter().zip(&gv.sky_lake) {
            if !lake {
                assert_eq!(*s, 6, "remix+anniversary takes the anniversary 6");
            }
        }
    }

    /// The fountain arm (`:79377-79392`): a style-3 island house places
    /// a fountain 207 (the 2×4 body, style 3 → frameX 108/126) and
    /// SwitchFountain flips its frameY +72 (the ON state).
    #[test]
    fn the_variant_island_house_places_a_switched_fountain() {
        let mut w = sky_world(4200, 1200, 520);
        let mut gv = GenVars::default();
        build_island(&mut w, 1000, 200, 50, false, &mut Rng(5));
        house_ground(&mut w, 1000, 205);
        gv.island_house_x.push(1000);
        gv.island_house_y.push(200);
        gv.island_style.push(3);
        gv.sky_lake.push(false);
        gv.num_island_houses = 1;
        pass_floating_island_houses(&mut w, &mut gv, &mut Rng(11), None, false);
        let fountain_cells: Vec<_> = w
            .tiles
            .iter()
            .filter(|t| t.active() && t.kind == 207)
            .collect();
        assert_eq!(fountain_cells.len(), 8, "the 2×4 fountain body");
        assert!(
            fountain_cells.iter().all(|t| t.frame_y >= 72),
            "SwitchFountain flipped the body ON (+72)"
        );
        assert!(
            fountain_cells.iter().all(|t| t.frame_x == 108 || t.frame_x == 126),
            "style 3 → frameX 36*3 (+18)"
        );
    }

    /// The palm arm (`:79462-79483`): a style-1 (desert) island house
    /// grows palms 323 over the sand surface — the Next(3,6) budget.
    #[test]
    fn the_desert_island_house_grows_palms() {
        let mut w = sky_world(4200, 1200, 520);
        let mut gv = GenVars::default();
        build_island(&mut w, 1000, 200, 50, false, &mut Rng(5));
        house_ground(&mut w, 1000, 205);
        // A sand strip around the house for the palm ground test
        // (`GrowPalmTree` accepts 53/234/116/112, `:27148`).
        for x in 940..=1060 {
            let i = w.index(x, 205);
            w.tiles[i].kind = 53;
        }
        // Clear the sky above the sand flanks (the trunk clearance box
        // is ±1 × up to 20 rows, `:27157`) — the compact disc's body
        // would otherwise block every palm inside the house band.
        for x in 940..=1060 {
            for y in 160..205 {
                let i = w.index(x, y);
                w.tiles[i] = Tile::default();
            }
        }
        gv.island_house_x.push(1000);
        gv.island_house_y.push(200);
        gv.island_style.push(1);
        gv.sky_lake.push(false);
        gv.num_island_houses = 1;
        pass_floating_island_houses(&mut w, &mut gv, &mut Rng(11), None, false);
        let palms = count_kind(&w, 323);
        assert!(palms > 0, "the desert island grows palms ({palms})");
        // The fountain also fires (style 1 > 0).
        assert!(count_kind(&w, 207) == 8, "the style-1 fountain too");
    }

    /// The remix furniture switch (`:79323-79332`, `:79421-79435`): a
    /// remix world without anniversary takes the evil-flavour door
    /// (crimson 5 / corrupt 38), the crimson table 14 style 5 + chairs
    /// 15 style 8 or the corrupt table 469 style 2 + chairs 38, and
    /// SKIPS the windows + banners (`:79395`/`:79444`). The drunk
    /// half-split (`:79217-79220`) picks the side by anchor x.
    #[test]
    fn the_remix_house_takes_the_evil_flavour_furniture() {
        // Corruption side (crimson=false): table 469 style 2, chairs 38.
        let mut w = sky_world(4200, 1200, 520);
        w.seeds |= SeedFlags::REMIX;
        let mut gv = GenVars::default();
        build_island(&mut w, 1000, 200, 50, false, &mut Rng(5));
        house_ground(&mut w, 1000, 205);
        gv.island_house_x.push(1000);
        gv.island_house_y.push(200);
        gv.island_style.push(0); // style 0 skips the fountain scatter (furniture-only witness)
        gv.sky_lake.push(false);
        gv.num_island_houses = 1;
        pass_floating_island_houses(&mut w, &mut gv, &mut Rng(11), None, false);
        assert!(count_kind(&w, 469) == 6, "the corrupt table 469 (3×2)");
        assert!(count_kind(&w, 14) == 0, "no classic table");
        let chairs: Vec<_> = w.tiles.iter().filter(|t| t.active() && t.kind == 15).collect();
        assert!(!chairs.is_empty(), "the chairs land");
        // Style 38 → frameY base 40*38 = 1520 (+18 bottom, +18 the flip
        // is frameX not frameY).
        assert!(chairs.iter().all(|t| t.frame_y == 1520 || t.frame_y == 1538));
        // No glass windows / banners on the remix arm.
        assert_eq!(w.tiles.iter().filter(|t| t.wall == WALL_GLASS).count(), 0);
        assert_eq!(count_kind(&w, 91), 0, "no banners on the remix arm");

        // Crimson side (crimson=true): table 14 style 5, chairs 8.
        let mut wc = sky_world(4200, 1200, 520);
        wc.seeds |= SeedFlags::REMIX;
        let mut gvc = GenVars::default();
        build_island(&mut wc, 1000, 200, 50, false, &mut Rng(5));
        house_ground(&mut wc, 1000, 205);
        gvc.island_house_x.push(1000);
        gvc.island_house_y.push(200);
        gvc.island_style.push(0); // furniture-only witness (no fountain scatter)
        gvc.sky_lake.push(false);
        gvc.num_island_houses = 1;
        pass_floating_island_houses(&mut wc, &mut gvc, &mut Rng(11), None, true);
        assert!(count_kind(&wc, 469) == 0, "no corrupt table on the crimson side");
        let tables: Vec<_> = wc.tiles.iter().filter(|t| t.active() && t.kind == 14).collect();
        assert_eq!(tables.len(), 6, "the crimson table 14 (3×2)");
        assert!(tables.iter().all(|t| (270..=306).contains(&(t.frame_x as i32))));
        let chairs: Vec<_> = wc.tiles.iter().filter(|t| t.active() && t.kind == 15).collect();
        assert!(chairs.iter().all(|t| t.frame_y == 320 || t.frame_y == 338));
    }

    /// The drunk half-split (`:79217-79220`) + the anniversary override:
    /// a drunk+remix world picks the evil flavour by the crimsonLeft
    /// half; anniversary WITHOUT getGood restores the classic furniture
    /// (`:79323`/`:79421` read `!anniversary || getGood` false).
    #[test]
    fn the_drunk_half_split_and_the_anniversary_override() {
        // drunk+remix, crimsonLeft=true: the LEFT half is crimson.
        let mut w = sky_world(4200, 1200, 520);
        w.seeds |= SeedFlags::DRUNK | SeedFlags::REMIX;
        let mut gv = GenVars::default();
        gv.crimson_left = true;
        build_island(&mut w, 1000, 200, 50, false, &mut Rng(5));
        house_ground(&mut w, 1000, 205);
        gv.island_house_x.push(1000); // < 2100 → the crimson half
        gv.island_house_y.push(200);
        gv.island_style.push(0); // furniture-only witness
        gv.sky_lake.push(false);
        gv.num_island_houses = 1;
        // crimson=false overall — the drunk split overrides it.
        pass_floating_island_houses(&mut w, &mut gv, &mut Rng(11), None, false);
        assert!(count_kind(&w, 14) == 6, "the left half takes the crimson table");
        assert!(count_kind(&w, 469) == 0);

        // remix+anniversary without getGood: the classic furniture arm.
        let mut wa = sky_world(4200, 1200, 520);
        wa.seeds |= SeedFlags::REMIX | SeedFlags::TENTH_ANNIVERSARY;
        let mut gva = GenVars::default();
        build_island(&mut wa, 1000, 200, 50, false, &mut Rng(5));
        house_ground(&mut wa, 1000, 205);
        gva.island_house_x.push(1000);
        gva.island_house_y.push(200);
        gva.island_style.push(0);
        gva.sky_lake.push(false);
        gva.num_island_houses = 1;
        pass_floating_island_houses(&mut wa, &mut gva, &mut Rng(11), None, false);
        assert!(count_kind(&wa, 14) == 6, "the anniversary keeps table 14");
        let chairs: Vec<_> = wa.tiles.iter().filter(|t| t.active() && t.kind == 15).collect();
        assert!(chairs.iter().all(|t| t.frame_y == 400 || t.frame_y == 418), "style 10");
        assert!(wa.tiles.iter().any(|t| t.wall == WALL_GLASS), "the windows return");
        assert!(count_kind(&wa, 91) > 0, "the banners return");
    }

    /// The variant bodies' `a == b` witness: the drunk world's verbatim
    /// SnowCloudIsland/DesertCloudIsland walks (the ring budgets, the
    /// drift draws, the blob/pocket/water/satellite chains) replay
    /// byte-identically, with the houses' fountain/palm arms included.
    #[test]
    fn the_drunk_variant_passes_are_reproducible_for_a_seed() {
        let run = || {
            let mut w = sky_world(4200, 1200, 520);
            w.seeds |= SeedFlags::DRUNK;
            let mut gv = gv_with_lakes(4200);
            pass_floating_islands(&mut w, &mut gv, &mut Rng(7), 520.0, false);
            pass_floating_island_houses(&mut w, &mut gv, &mut Rng(7), None, false);
            super::super::world_hash::world_hash(&w)
        };
        assert_eq!(run(), run(), "the drunk variant passes must be deterministic");
    }

    /// The full-core graft (research/87 §KEEP-3): the house chest runs
    /// `buried_chest::add_buried_chest` — the plain remix world's flag12
    /// arm (`:36176-36196`) retypes the sky chest: crimson keeps the wood
    /// 21 at style 43, corruption takes the mahogany 467 at style 3. The
    /// stocking event carries the RESOLVED style/kind (the compact
    /// stand-in always reported 13/21).
    #[test]
    fn the_remix_house_chest_takes_the_evil_retype() {
        for (crimson, kind, style) in [(true, 21u16, 43i32), (false, 467, 3)] {
            let mut w = sky_world(4200, 1200, 520);
            w.seeds |= SeedFlags::REMIX;
            let mut gv = GenVars::default();
            build_island(&mut w, 1000, 200, 50, false, &mut Rng(5));
            house_ground(&mut w, 1000, 205);
            gv.island_house_x.push(1000);
            gv.island_house_y.push(200);
            gv.island_style.push(0);
            gv.sky_lake.push(false);
            gv.num_island_houses = 1;
            let mut events: Vec<(i32, u16)> = Vec::new();
            {
                let mut hook = |ev: &super::super::chest_stock::ChestSiteEvent,
                                _rng: &mut dyn super::super::chest_stock::WorldGenRand|
                 -> Vec<crate::save::ChestItem> {
                    events.push((ev.chest_style, ev.chest_tile_type));
                    Vec::new()
                };
                pass_floating_island_houses(&mut w, &mut gv, &mut Rng(11), Some(&mut hook), crimson);
            }
            assert_eq!(events, vec![(style, kind)], "crimson={crimson}");
            assert!(count_kind(&w, kind) >= 4, "the retyped chest places");
        }
        // The classic world keeps the wood style 13 (the control).
        let mut w = sky_world(4200, 1200, 520);
        let mut gv = GenVars::default();
        build_island(&mut w, 1000, 200, 50, false, &mut Rng(5));
        house_ground(&mut w, 1000, 205);
        gv.island_house_x.push(1000);
        gv.island_house_y.push(200);
        gv.island_style.push(0);
        gv.sky_lake.push(false);
        gv.num_island_houses = 1;
        let mut events: Vec<(i32, u16)> = Vec::new();
        {
            let mut hook = |ev: &super::super::chest_stock::ChestSiteEvent,
                            _rng: &mut dyn super::super::chest_stock::WorldGenRand|
             -> Vec<crate::save::ChestItem> {
                events.push((ev.chest_style, ev.chest_tile_type));
                Vec::new()
            };
            pass_floating_island_houses(&mut w, &mut gv, &mut Rng(11), Some(&mut hook), false);
        }
        assert_eq!(events, vec![(13, 21)]);
    }

    /// The core's larva veto is INERT on a fresh sunplate floor (the
    /// research/87 #15 note, now pinned through the full core): the shell
    /// writes the solid floor at `vy+2` in the chest column, so the
    /// down-scan lands there BEFORE any bulb below it — the chest places
    /// and the house counts. The veto itself fires only when the scan
    /// column carries a live 231 above the first solid row (the
    /// errorWorld deep-island case — pinned at the core level in
    /// `buried_chest.rs::buried_chest_larva_column_aborts`).
    #[test]
    fn the_larva_below_the_sunplate_floor_is_inert() {
        let mut w = sky_world(4200, 1200, 520);
        let mut gv = GenVars::default();
        build_island(&mut w, 1000, 200, 50, false, &mut Rng(5));
        house_ground(&mut w, 1000, 205);
        // A bulb below the house floor (the shell's bottom row is vy+2,
        // the scan's first solid) — the down-scan never reaches it.
        let b = w.index(1000, 212);
        w.tiles[b].kind = 231;
        w.tiles[b].flags |= FLAG_ACTIVE;
        gv.island_house_x.push(1000);
        gv.island_house_y.push(200);
        gv.island_style.push(0);
        gv.sky_lake.push(false);
        gv.num_island_houses = 1;
        let mut events = 0usize;
        {
            let mut hook = |_ev: &super::super::chest_stock::ChestSiteEvent,
                            _rng: &mut dyn super::super::chest_stock::WorldGenRand|
             -> Vec<crate::save::ChestItem> {
                events += 1;
                Vec::new()
            };
            pass_floating_island_houses(&mut w, &mut gv, &mut Rng(11), Some(&mut hook), false);
        }
        assert_eq!(events, 1, "the floor lands before the bulb — the chest places");
        assert_eq!(gv.sky_island_house_count, 1, "the house counts");
        assert!(count_kind(&w, TILE_SUNPLATE) > 0, "the house builds");
        // The bulb survives (the shell/carve never reach row 212).
        assert!(w.tile(1000, 212).is_some_and(|t| t.active() && t.kind == 231));
    }

    /// The errorWorld style shuffle (`:35986-36014`) draws on the house's
    /// stream: with ERROR_WORLD stamped, the resolved chest style leaves
    /// the fixed 13 on some seeds (the Next(2) gate + the 1/3 Next(52)
    /// re-roll) — the placement draw the compact stand-in never made.
    #[test]
    fn the_error_world_house_chest_shuffles_the_style() {
        let mut saw_other = false;
        let mut saw_13 = false;
        for seed in 1..64u64 {
            let mut w = sky_world(4200, 1200, 520);
            w.micro_seeds |= MicroSeedFlags::ERROR_WORLD;
            let mut gv = GenVars::default();
            build_island(&mut w, 1000, 200, 50, false, &mut Rng(5));
            house_ground(&mut w, 1000, 205);
            gv.island_house_x.push(1000);
            gv.island_house_y.push(200);
            gv.island_style.push(0);
            gv.sky_lake.push(false);
            gv.num_island_houses = 1;
            let mut styles: Vec<i32> = Vec::new();
            {
                let mut hook = |ev: &super::super::chest_stock::ChestSiteEvent,
                                _rng: &mut dyn super::super::chest_stock::WorldGenRand|
                 -> Vec<crate::save::ChestItem> {
                    styles.push(ev.chest_style);
                    Vec::new()
                };
                pass_floating_island_houses(&mut w, &mut gv, &mut Rng(seed), Some(&mut hook), false);
            }
            if let Some(s) = styles.first() {
                saw_13 |= *s == 13;
                saw_other |= *s != 13;
            }
        }
        assert!(saw_13, "the Next(2) gate keeps 13 on some seeds");
        assert!(saw_other, "the shuffle leaves 13 on others");
    }

    /// The skyblock island-side arms (`:12960-12962`, `:13003-13006`,
    /// `:13024-13040`): skyblock + extraFloatingIslands runs the pass
    /// with the fixed y anchor `worldSurface/2 + 100` (no column scan),
    /// the `/42` attempt budget and the ±150/±75 clear window. The
    /// skyblock TERRAIN is f24's `GenerateSkyBlockWorld` lane — these
    /// arms only gate/position the island discs (the claim boundary:
    /// ANALYSIS "Sky lakes / floating-island residuals", the f24 read).
    #[test]
    fn the_skyblock_island_side_arms() {
        let mut w = sky_world(4200, 1200, 520);
        w.seeds |= SeedFlags::SKYBLOCK;
        w.micro_seeds |= MicroSeedFlags::EXTRA_FLOATING_ISLANDS;
        let mut gv = gv_with_lakes(4200);
        pass_floating_islands(&mut w, &mut gv, &mut Rng(42), 520.0, false);
        // The fixed anchor: num6 = (545/2 + 100) = 372, the y roll
        // Next(90, 272), the surfaceLow clamp min(y, 470) is inert here.
        let anchor = (w.world_surface / 2.0 + 100.0) as i32;
        for (y, lake) in gv.island_house_y.iter().zip(&gv.sky_lake) {
            // The compact disc's ledger stores y - radius/2 (radius
            // 40..60 → the top offset 20..30); the lake tail included.
            let _ = lake;
            assert!(
                (90 - 30..anchor - 100).contains(y),
                "the skyblock y {y} rides the fixed anchor {anchor}"
            );
        }
        // A blocked sky rejects the site: an active tile inside every
        // possible ±150/±75 window band forces the clear-window refusal
        // (the budget exhausts, no island places).
        let mut wb = sky_world(4200, 1200, 520);
        wb.seeds |= SeedFlags::SKYBLOCK;
        wb.micro_seeds |= MicroSeedFlags::EXTRA_FLOATING_ISLANDS;
        for x in 0..4200 {
            for y in 0..400 {
                let i = wb.index(x, y);
                wb.tiles[i].kind = 189;
                wb.tiles[i].flags |= FLAG_ACTIVE;
            }
        }
        let mut gvb = gv_with_lakes(4200);
        pass_floating_islands(&mut wb, &mut gvb, &mut Rng(42), 520.0, false);
        assert_eq!(gvb.num_island_houses, 0, "the clear window rejects every site");
    }

    /// W4-g44 biome-closeout (sky-lake WATER physics, WRITTEN NOT RUN):
    /// the compact `CloudLake` bowl's water SURVIVES settle. The gen
    /// settle (`WorldGen.cs:16156-16207`, g47's `gen_settle_loop` +
    /// `water_check`/`quick_water` machinery) never drains an enclosed
    /// pool: `WaterCheck` (`:80415-80494`) zeroes liquid only INSIDE a
    /// solid tile (`:80426-80432` — the bowl cells are inactive water)
    /// and wakes a cell only when the cell below is open and not full,
    /// or a side cell is open with a DIFFERENT level (`:80449-80470`) —
    /// the bowl's rain-cloud 196 floor/sides are `tileSolid` true
    /// (`Main.cs:7118`), the water rows are uniformly 255, so no wake
    /// arm fires and `SettleWaterAt`'s fall/spread walks
    /// (`Liquid.cs:171-278`) have nowhere to go. The waterLine re-type
    /// (`Liquid.cs:184-187`) needs the fall arm (`flag4`) — inert on an
    /// enclosed lake, and the load-settle pins waterLine = maxTilesY
    /// anyway (g47 `settle_loaded_world`). This witness replays the
    /// bowl through the landed runtime `LiquidSystem` (the same
    /// `UpdateLiquid` engine the gen loop drives) and pins the mass.
    #[test]
    fn w4_g44_the_sky_lake_water_survives_settle() {
        let mut w = sky_world(4200, 1200, 520);
        build_island(&mut w, 1000, 200, 50, true, &mut Rng(5));
        w.compute_column_surface();
        w.compute_chunks();
        // The bowl: rows 177-179 water, row 180 the rain-cloud floor,
        // x 984/1016 the rain-cloud sides (half = 50/3 = 16).
        let bowl_cells: Vec<(i32, i32)> = (985..=1015)
            .flat_map(|x| (177..=179).map(move |y| (x, y)))
            .collect();
        let mass = |w: &World| -> i64 {
            bowl_cells.iter().map(|&(x, y)| w.tile(x, y).unwrap().liquid as i64).sum()
        };
        let before = mass(&w);
        assert_eq!(before, 31 * 3 * 255, "the bowl is 31x3 full water");
        for &(x, y) in &bowl_cells {
            let t = w.tile(x, y).unwrap();
            assert!(!t.active(), "the water cells are inactive");
            assert_eq!(t.liquid_kind(), crate::tile::LiquidKind::Water);
            // `WillWaterPlacedHereStayPut` holds on every water cell:
            // the floor/sides are solid 196 or full-water neighbours.
            assert!(will_water_placed_here_stay_put(&w, x, y), "({x},{y})");
        }
        // Settle: wake every bowl cell (the WaterCheck registration the
        // gen loop would do) and drain the engine to quiescence.
        let mut sys = crate::liquid::LiquidSystem::new(&w);
        for &(x, y) in &bowl_cells {
            sys.add_water(&w, x, y);
        }
        for _ in 0..400 {
            sys.update(&mut w);
        }
        assert_eq!(mass(&w), before, "the enclosed lake keeps every unit");
        // No water leaked below the floor or past the sides.
        for x in 980..=1020 {
            for y in 181..=210 {
                assert_eq!(w.tile(x, y).unwrap().liquid, 0, "no leak at ({x},{y})");
            }
        }
        // The retired list is empty or static — the lake is settled, not
        // flowing (the kill-passes retirement, `Liquid.cs:1120-1130`).
        for _ in 0..200 {
            sys.update(&mut w);
        }
        assert_eq!(mass(&w), before, "still full after the retirement tail");
    }

    /// W4-g44 biome-closeout (island-house NPC housing census, WRITTEN
    /// NOT RUN): the GENERATED IslandHouse is not yet a move-in-ready
    /// room — exactly as vanilla. Two C# facts the port keeps 1:1:
    /// (1) the door carve (`:79317-79322`) clears 5 columns × 3 rows but
    /// `PlaceTile(10)` (`:79336` → `:59843-59883` → `PlaceDoor`
    /// `:31773-31803`) fills ONE column — the two flank columns stay
    /// open, so the room flood escapes through them (`CheckRoom`'s fill
    /// runs past the wall-82 cells until the edge/size cut); (2) the
    /// house places table/chairs/door/banners but NO torch — `RoomNeeds`
    /// (`:5679-5730`) stays incomplete until the player adds a light.
    /// The housing VALIDITY fix is the wall table (housing.rs
    /// `is_house_wall` → the full `Main.wallHouse`, wall 82 included):
    /// once the player seals the flanks and adds a torch the sky room
    /// scores (the housing.rs witness). The d118/d119 co-lanes own the
    /// move-in drivers; their `is_house_wall` copies need the same
    /// full-table dedupe at the merge (research/87 closeout §3).
    #[test]
    fn w4_g44_the_generated_island_house_ships_open_flanks_and_no_torch() {
        let mut w = sky_world(4200, 1200, 520);
        let mut gv = GenVars::default();
        build_island(&mut w, 1000, 200, 50, false, &mut Rng(5));
        house_ground(&mut w, 1000, 205);
        gv.island_house_x.push(1000);
        gv.island_house_y.push(200);
        gv.island_style.push(0);
        gv.sky_lake.push(false);
        gv.num_island_houses = 1;
        pass_floating_island_houses(&mut w, &mut gv, &mut Rng(11), None, false);
        // The house interior carries the disc wall 82 (the C# `wall`
        // argument, `:79311`) — the housing fix's subject.
        let disc = w.tiles.iter().filter(|t| t.wall == WALL_DISC).count();
        assert!(disc > 50, "the disc-wall interior: {disc}");
        // No torch anywhere in the house band (the C# places none).
        let torches = (960..=1040)
            .flat_map(|x| (160..=215).map(move |y| (x, y)))
            .filter(|&(x, y)| w.tile(x, y).is_some_and(|t| t.active() && t.kind == 4))
            .count();
        assert_eq!(torches, 0, "the generated house ships no light source");
        // The door carve's flank columns stay open: find the door column
        // (the closed door 10 stack) and check the ±2 columns at the
        // door's rows — at least one flank cell is inactive (the carve
        // the single-column PlaceDoor never fills).
        let door_col = (960..=1040)
            .find(|&x| (160..=215).any(|y| w.tile(x, y).is_some_and(|t| t.active() && t.kind == 10)))
            .expect("the house places a door");
        let door_row = (160..=215)
            .find(|&y| w.tile(door_col, y).is_some_and(|t| t.active() && t.kind == 10))
            .unwrap();
        let open_flanks = [-2i32, -1, 1, 2]
            .iter()
            .filter(|&&dx| {
                (-2..=2).any(|dy| {
                    w.tile(door_col + dx, door_row + dy)
                        .is_some_and(|t| !t.active())
                })
            })
            .count();
        assert!(open_flanks >= 2, "the 5-wide carve keeps open flanks: {open_flanks}");
        // RoomNeeds on the generated house's own furniture is incomplete
        // (the torch flag) — the scan of the house's furniture kinds.
        let kinds: Vec<u16> = (960..=1040)
            .flat_map(|x| (160..=215).map(move |y| (x, y)))
            .filter_map(|(x, y)| w.tile(x, y).filter(|t| t.active()).map(|t| t.kind))
            .collect();
        let flags = crate::housing::room_needs_flags(&kinds);
        assert!(flags.table && flags.chair && flags.door, "the house furniture");
        assert!(!flags.torch, "no torch — RoomNeeds incomplete until the player adds one");
    }
}