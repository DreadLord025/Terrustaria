//! The world structures — the compact ports of the `GenPassNameID` passes
//! (`ID/GenPassNameID.cs`): the jungle shrines (`JungleShrines`,
//! `WorldGen.cs:15977+`), the shrine + underground chests
//! (`ChestsInJungleShrines` `:17211+`), the demon/crimson altars
//! (`DemonAndCrimsonAltars` `:15750-15800`), the pyramids (`Pyramid`,
//! `DunesAndPyramidLocations` `:15367+`) and the pots
//! (`PotsGraveyardsAndBoulderPiles` `:17996+`).
//!
//! Tiles (`TileID.cs`): the rich mahogany brick 119 (the shrine shell — the
//! C#'s `GenVars.jungleHut` pick `:11319`), the torch 4 (the style 3), the
//! chest 21 (2×2 frame-important), the demon altar 26 (3×2, the style by
//! the evil flavour), the pot 28, the sandstone brick 480 (the pyramid
//! hull). Walls (`WallID.cs`): the shrine interiors 24/42/45/10, the
//! sandstone 187.
//!
//! Deviations (ANALYSIS "Structures (M1.4-WGEN-2)"): the chest inventories
//! ride the storage task (the chests place as tiles only — the C#
//! `Chest.CreateChest` book is the M7-TE follow-up); the shrine/altar
//! layouts are shape-level (the C#'s exact brick patterns are not 1:1); the
//! pyramid corridors are one floor (the C#'s multi-chamber crawl is the
//! follow-up); the boulder piles and the graveyards are out of scope.

use super::{TILE_DIRT, TILE_STONE, WALL_DIRT};
use super::biomes::{TILE_JUNGLE_GRASS, TILE_MUD};
use crate::tile::{FLAG_ACTIVE, Tile};
use crate::world::World;

pub(crate) const TILE_SHRINE_BRICK: u16 = 119;
pub(crate) const TILE_TORCH: u16 = 4;
pub(crate) const TILE_CHEST: u16 = 21;
pub(crate) const TILE_ALTAR: u16 = 26;
pub(crate) const TILE_POT: u16 = 28;
pub(crate) const TILE_PYRAMID_BRICK: u16 = 480;

pub(crate) const WALL_SHRINE: u16 = 24;

/// The chest 2×2: the anchor tile `(x, y)` with the frame corners
/// (`TileObject`'s chest frames — frame_x 0/18, frame_y 0/18).
pub(crate) fn place_chest(world: &mut World, x: i32, y: i32) -> bool {
    for (dx, dy, fx, fy) in [
        (0i32, 0i32, 0i16, 0i16),
        (1, 0, 18, 0),
        (0, 1, 0, 18),
        (1, 1, 18, 18),
    ] {
        let (px, py) = (x + dx, y + dy);
        if px < 1 || py < 1 || px >= world.width - 1 || py >= world.height - 1 {
            return false;
        }
        if world.tile(px, py).is_some_and(|t| t.active() && t.kind != 0) {
            return false;
        }
    }
    for (dx, dy, fx, fy) in [
        (0i32, 0i32, 0i16, 0i16),
        (1, 0, 18, 0),
        (0, 1, 0, 18),
        (1, 1, 18, 18),
    ] {
        let i = world.index(x + dx, y + dy);
        let t = &mut world.tiles[i];
        t.kind = TILE_CHEST;
        t.frame_x = fx;
        t.frame_y = fy;
        t.flags |= FLAG_ACTIVE;
        t.liquid = 0;
    }
    true
}

/// `JungleShrines` (`WorldGen.cs:15977-16140`): the hut rooms inside the
/// underground jungle — the brick shell (the tile 119), the carved interior
/// with the shrine walls, a torch, the mud pillars below and the mud cap on
/// top. Returns the chest spots (the `GenVars.JChestX/Y` slice).
pub(crate) fn pass_jungle_shrines(
    world: &mut World,
    spans: &super::biomes::BiomeSpans,
    rock_layer: f64,
    rng: &mut super::Rng,
) -> Vec<(i32, i32)> {
    let mut chests = Vec::new();
    if spans.jungle_end <= spans.jungle_start {
        return chests;
    }
    let count = 7 + rng.range(0, 6) as i32;
    for _ in 0..count * 8 {
        // The caves carve the mud band before this pass — most spots miss;
        // the C#'s `num4` retry counter gives each hut ~30 re-rolls
        // (`:15991-16007`'s `while (flag)` loop).
        // The hut centre inside the underground jungle band (`:15985-15989`:
        // `Next(worldSurface + rockLayer) / 2 .. maxTilesY - 400` scaled to
        // the jungle's own span).
        let x_lo = spans.jungle_start + 10;
        let x_hi = (spans.jungle_end - 10).max(x_lo + 1);
        let x = rng.range(x_lo, x_hi);
        let y_lo = (world.world_surface as i32).max(spans.jungle_start.min(spans.jungle_end));
        let y_hi = (rock_layer as i32 + 120).min(world.height - 60);
        if y_hi <= y_lo {
            return chests;
        }
        let y = rng.range(y_lo, y_hi);
        let hw = rng.range(2, 4) as i32;
        let hh = rng.range(2, 4) as i32;
        if x - hw - 2 < 2 || x + hw + 2 >= world.width - 2 || y + hh + 4 >= world.height - 2 {
            continue;
        }
        // The mud must be at the hut spot (the shrines live in the jungle
        // mud, `:16007`: `tile.type == 60` guard — the port checks the mud
        // band's tile).
        if !world
            .tile(x, y)
            .is_some_and(|t| t.active() && matches!(t.kind, TILE_MUD | TILE_JUNGLE_GRASS))
        {
            continue;
        }
        // The count budget: one shrine per successful spot.
        if chests.len() >= count as usize {
            break;
        }
        // The shell: the solid brick rectangle.
        for px in (x - hw - 1)..=(x + hw + 1) {
            for py in (y - hh - 1)..=(y + hh + 1) {
                let i = world.index(px, py);
                let t = &mut world.tiles[i];
                t.kind = TILE_SHRINE_BRICK;
                t.flags |= FLAG_ACTIVE;
                t.liquid = 0;
                t.set_slope(0);
                t.wall = super::biomes::WALL_JUNGLE;
            }
        }
        // The interior: carved with the shrine walls.
        for px in (x - hw)..=(x + hw) {
            for py in (y - hh)..=(y + hh) {
                let i = world.index(px, py);
                world.tiles[i] = Tile::default();
                world.tiles[i].wall = WALL_SHRINE;
            }
        }
        // A torch on the interior wall (`:16060-16072`).
        for _ in 0..50 {
            let tx = rng.range(x - hw, x + hw + 1);
            let ty = rng.range(y - hh, y + hh - 1);
            let i = world.index(tx, ty);
            if !world.tiles[i].active() {
                world.tiles[i] = Tile {
                    kind: TILE_TORCH,
                    frame_x: (3 * 18) as i16,
                    ..Default::default()
                };
                world.tiles[i].flags |= FLAG_ACTIVE;
                break;
            }
        }
        // The mud pillars below the floor (`:16092-16106`).
        for px in (x - hw - 1)..=(x + hw + 1) {
            let mut py = y + hh + 2;
            let mut fill = 4;
            while fill > 0 && py < world.height - 2 {
                if world.tile(px, py).is_some_and(|t| t.active()) {
                    break;
                }
                let i = world.index(px, py);
                world.tiles[i].kind = TILE_MUD;
                world.tiles[i].flags |= FLAG_ACTIVE;
                py += 1;
                fill -= 1;
            }
        }
        // The sloped cap above (`:16108-16120`): the shrinking brick rows.
        let mut half = hw - rng.range(1, 3);
        let mut py = y - hh - 2;
        while half >= 0 {
            for px in (x - half - 1)..=(x + half + 1) {
                let i = world.index(px, py);
                world.tiles[i].kind = TILE_SHRINE_BRICK;
                world.tiles[i].flags |= FLAG_ACTIVE;
            }
            half -= rng.range(1, 3);
            py -= 1;
        }
        chests.push((x, y));
    }
    chests
}

/// `ChestsInJungleShrines` + the underground gold chests (`:17211+`): the
/// shrine spots take the chests first, then the cave floors in the cavern
/// layer scatter the rest (the count `maxTilesX / 40` shape).
pub(crate) fn pass_chests(
    world: &mut World,
    shrine_spots: &[(i32, i32)],
    rng: &mut super::Rng,
) -> u32 {
    let mut placed = 0u32;
    for (x, y) in shrine_spots {
        if place_chest(world, *x, y + 1) {
            placed += 1;
        }
    }
    let scattered = (world.width / 40).max(4);
    for _ in 0..scattered {
        let x = rng.range(40, world.width - 40);
        let y = rng.range(world.world_surface as i32 + 30, world.height - 220);
        // The floor scan: the air above a solid non-chest tile.
        if !world.tile(x, y).is_some_and(|t| !t.active()) {
            continue;
        }
        let below = world.tile(x, y + 1).is_some_and(|t| {
            t.active() && crate::tiledata::TILE_SOLID.has(t.kind) && t.kind != TILE_CHEST
        });
        if !below {
            continue;
        }
        if place_chest(world, x, y) {
            placed += 1;
        }
    }
    placed
}

/// `DemonAndCrimsonAltars` (`WorldGen.cs:15750-15800`): the count
/// `tiles × 3.3e-6`, the 3×2 altar 26 placed in the cavern band, skipping
/// the middle 10 % of the world and the ocean bands; the style rides the
/// evil flavour (the frame_y `18 · style`).
pub(crate) fn pass_altars(world: &mut World, crimson: bool, rng: &mut super::Rng) -> u32 {
    let count = ((world.width as f64 * world.height as f64) * 3.3e-6) as i32;
    let mut placed = 0u32;
    // `:15758`: `Next(281, maxTilesX - 3 - 280)` — the 281-tile ocean
    // margins; the tiny worlds collapse to the inner band without them.
    let x_lo = 281.min(world.width / 4);
    let x_hi = (world.width - 3 - 280).max(x_lo + 1);
    let keep_mid_band = world.width > 1200;
    for _ in 0..count {
        for _ in 0..100 {
            let x = rng.range(x_lo, x_hi);
            // `:15759-15762`: the middle 10 % band is excluded.
            let mid0 = (world.width as f64 * 0.45) as i32;
            let mid1 = (world.width as f64 * 0.55) as i32;
            if keep_mid_band && x > mid0 && x < mid1 {
                continue;
            }
            let y_lo = ((world.world_surface * 2.0 + world.rock_layer) / 3.0) as i32;
            let y_hi = ((world.rock_layer + (world.height - 350) as f64 * 2.0) / 3.0) as i32;
            if y_hi <= y_lo {
                return placed;
            }
            let y = rng.range(y_lo, y_hi);
            // `Place3x2` (`WorldGen.cs`): the altar needs a solid floor
            // under any of its feet and the body may carve the loose cells
            // (the C# places into the cave walls freely).
            let floor_ok = (0..3).any(|dx| {
                world
                    .tile(x + dx, y + 2)
                    .is_some_and(|t| t.active() && crate::tiledata::TILE_SOLID.has(t.kind))
            });
            if !floor_ok {
                continue;
            }
            let style = if crimson { 1 } else { 0 };
            for (dx, dy) in [(0i32, 0i32), (1, 0), (2, 0), (0, 1), (1, 1), (2, 1)] {
                let i = world.index(x + dx, y + dy);
                let t = &mut world.tiles[i];
                t.kind = TILE_ALTAR;
                t.frame_x = (dx * 18) as i16;
                t.frame_y = (dy * 18 + style * 36) as i16;
                t.flags |= FLAG_ACTIVE;
            }
            placed += 1;
            break;
        }
    }
    placed
}

/// The pyramid (`DunesAndPyramidLocations` `:15367+`): 1-2 stepped hulls in
/// the desert — the sandstone-brick shell with the carved chambers, the
/// shrine walls and a chest in the base room.
pub(crate) fn pass_pyramids(
    world: &mut World,
    desert_start: i32,
    desert_end: i32,
    surface: &[i32],
    rng: &mut super::Rng,
) -> u32 {
    if desert_end - desert_start < 120 {
        return 0;
    }
    let mut placed = 0u32;
    let count = 1 + rng.range(0, 2) as i32;
    for _ in 0..count {
        let x0 = rng.range(desert_start + 60, desert_end - 60);
        let base = surface[x0 as usize];
        let height = rng.range(16, 26);
        let half = height;
        // The stepped hull (`Pyramid`'s triangle shape).
        for step in 0..height {
            let y = base - height + step;
            let w = half - step;
            for x in (x0 - w)..=(x0 + w) {
                if x < 2 || y < 2 || x >= world.width - 2 {
                    continue;
                }
                let i = world.index(x, y);
                let t = &mut world.tiles[i];
                t.kind = TILE_PYRAMID_BRICK;
                t.flags |= FLAG_ACTIVE;
                t.liquid = 0;
                t.set_slope(0);
                t.wall = super::WALL_SANDSTONE;
            }
        }
        // The base chamber: carved inside the hull's bottom.
        for x in (x0 - half / 2)..=(x0 + half / 2) {
            for y in (base - 4)..=base {
                let i = world.index(x, y);
                if x <= x0 - half / 2 + 1 || x >= x0 + half / 2 - 1 {
                    continue;
                }
                world.tiles[i] = Tile::default();
                world.tiles[i].wall = super::WALL_SANDSTONE;
            }
        }
        // The entrance tunnel from the desert surface into the chamber.
        let ex = x0 + half / 2;
        for y in (base - 3)..=base {
            for dx in 0..(half / 2 + 2) {
                let i = world.index(ex + dx, y);
                if ex + dx < world.width - 2 {
                    world.tiles[i] = Tile::default();
                    world.tiles[i].wall = super::WALL_SANDSTONE;
                }
            }
        }
        if place_chest(world, x0 - 2, base - 2) {
            placed += 1;
        }
    }
    placed
}

/// The pots arm of `PotsGraveyardsAndBoulderPiles` (`WorldGen.cs:17996+`):
/// the pot 28 scattered on the cave floors across the underground.
pub(crate) fn pass_pots(world: &mut World, rng: &mut super::Rng) -> u32 {
    let count = ((world.width as f64 * world.height as f64) * 1.2e-4) as i32;
    let mut placed = 0u32;
    for _ in 0..count {
        let x = rng.range(40, world.width - 40);
        let y = rng.range(world.world_surface as i32, world.height - 260);
        // The cave floors are sparse in the picked span — the spot scans
        // down up to 20 rows for the first air-over-solid seat.
        let mut seat = None;
        for dy in 0..20 {
            let py = y + dy;
            let below = world.tile(x, py + 1).is_some_and(|t| {
                t.active() && crate::tiledata::TILE_SOLID.has(t.kind)
            });
            let air = world.tile(x, py).is_none_or(|t| !t.active());
            if below && air {
                seat = Some(py);
                break;
            }
        }
        let Some(y) = seat else {
            continue;
        };
        let style = rng.range(0, 7);
        {
            let i = world.index(x, y);
            let t = &mut world.tiles[i];
            t.kind = TILE_POT;
            t.frame_x = (style * 18) as i16;
            t.frame_y = 0;
            t.flags |= FLAG_ACTIVE;
            placed += 1;
        }
    }
    placed
}

const _: (u16, u16, u16) = (TILE_DIRT, TILE_STONE, WALL_DIRT);
