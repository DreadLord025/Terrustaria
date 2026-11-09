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

pub(crate) const TILE_HIVE: u16 = 225;
pub(crate) const TILE_LARVA: u16 = 231;
pub(crate) const TILE_HELL_BRICK: u16 = 75;
pub(crate) const WALL_HELL_BRICK: u16 = 14;

fn put_hive(world: &mut World, x: i32, y: i32, kind: u16) {
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

/// `Beehives` + `BeeLarvaInBeehives` (`WorldGen.cs:15927-16000`,
/// `AddBeeLarva` `:32050-32068`): 1-8 hive domes in the underground jungle —
/// the hive-block (225) shell with the honey pockets (the liquid 2) inside,
/// roughly half carrying the larva (231) in the 3×4 clear centre.
pub(crate) fn pass_beehives(
    world: &mut World,
    spans: &super::biomes::BiomeSpans,
    rock_layer: f64,
    rng: &mut super::Rng,
) -> u32 {
    if spans.jungle_end <= spans.jungle_start {
        return 0;
    }
    let count = 1 + rng.range(0, (5.0 * (world.width as f32 / 4200.0)) as i32 + 1)
        + rng.range(0, 3);
    let mut placed = 0u32;
    let y_lo = ((world.world_surface as i32 + rock_layer as i32) / 2).max(spans.jungle_start.min(spans.jungle_end));
    let y_hi = (world.height - 300).max(y_lo + 20);
    for _ in 0..count {
        let cx_lo = spans.jungle_start + 10;
        let cx_hi = (spans.jungle_end - 10).max(cx_lo + 1);
        let cx = rng.range(cx_lo, cx_hi);
        if y_hi <= y_lo {
            return placed;
        }
        let cy = rng.range(y_lo, y_hi);
        let r = rng.range(10, 18);
        // The dome: the hive shell with the honey core.
        for dx in -r..=r {
            for dy in -r..=r {
                let px = cx + dx;
                let py = cy + dy;
                let dist = ((dx * dx + dy * dy) as f32).sqrt();
                if dist > r as f32 {
                    continue;
                }
                if !world
                    .tile(px, py)
                    .is_some_and(|t| t.active() || t.liquid > 0)
                {
                    continue;
                }
                if dist < r as f32 - 3.0 {
                    // The core: the honey pockets (the liquid 2 = honey).
                    let i = world.index(px, py);
                    let t = &mut world.tiles[i];
                    if dist < r as f32 - 5.0 && rng.range(0, 3) != 0 {
                        *t = Tile::default();
                        t.liquid = 255;
                        t.set_liquid_kind(crate::tile::LiquidKind::Honey);
                    } else {
                        t.kind = 0;
                        t.flags &= !FLAG_ACTIVE;
                        t.liquid = 0;
                    }
                } else {
                    put_hive(world, px, py, TILE_HIVE);
                }
            }
        }
        // The larva in ~half the hives (`num5 = Next(5)` honey patches; the
        // larva ride the hive centre — `AddBeeLarva`'s 3×4 clear + the 231).
        if rng.range(0, 2) == 0 {
            for dx in -1..=1 {
                for dy in -2..=1 {
                    let px = cx + dx;
                    let py = cy + dy;
                    if dy == 1 {
                        let i = world.index(px, py);
                        let wall = world.tiles[i].wall;
                        world.tiles[i] = Tile {
                            kind: TILE_HIVE,
                            ..Default::default()
                        };
                        world.tiles[i].flags |= FLAG_ACTIVE;
                        world.tiles[i].wall = wall;
                    } else {
                        let i = world.index(px, py);
                        world.tiles[i] = Tile::default();
                    }
                }
            }
            let i = world.index(cx, cy);
            world.tiles[i] = Tile {
                kind: TILE_LARVA,
                ..Default::default()
            };
            world.tiles[i].flags |= FLAG_ACTIVE;
        }
        placed += 1;
    }
    placed
}

/// `HellFort` (`WorldGen.cs:32683+`, the tile 75 / the wall 14 defaults):
/// the brick boxes in the hell layer — the shell with the carved rooms, the
/// lava pools on the floors and the chests.
pub(crate) fn pass_hell_forts(world: &mut World, hell_top: i32, rng: &mut super::Rng) -> u32 {
    let count = 2 + (world.width / 4200) as i32;
    let mut placed = 0u32;
    let cy_lo = hell_top + 20;
    let cy_hi = world.height - 60;
    if cy_hi <= cy_lo {
        return placed;
    }
    for _ in 0..count {
        let cx = rng.range(200, (world.width - 200).max(201));
        let cy = rng.range(cy_lo, cy_hi);
        let hw = rng.range(10, 18);
        let hh = rng.range(6, 10);
        // The shell.
        for px in (cx - hw)..=(cx + hw) {
            for py in (cy - hh)..=(cy + hh) {
                if px < 2 || py < 2 || px >= world.width - 2 || py >= world.height - 2 {
                    continue;
                }
                if !world.tile(px, py).is_some_and(|t| t.active() || t.liquid > 0) {
                    continue;
                }
                put_hive(world, px, py, TILE_HELL_BRICK);
                let i = world.index(px, py);
                world.tiles[i].wall = WALL_HELL_BRICK;
            }
        }
        // The two floors of the carved rooms with the lava pools.
        for floor in 0..2 {
            let fy = cy - hh / 2 + floor * hh;
            for px in (cx - hw + 2)..=(cx + hw - 2) {
                for py in (fy - 3)..=fy {
                    if px < 2 || py < 2 || px >= world.width - 2 || py >= world.height - 2 {
                        continue;
                    }
                    let i = world.index(px, py);
                    world.tiles[i] = Tile::default();
                    world.tiles[i].wall = WALL_HELL_BRICK;
                }
                // The lava pool strip on the floor.
                if px % 7 != 0 {
                    let i = world.index(px, fy);
                    let t = &mut world.tiles[i];
                    t.liquid = 200;
                    t.set_liquid_kind(crate::tile::LiquidKind::Lava);
                }
            }
        }
        // The chest in the upper room.
        if place_chest(world, cx - 3, cy - hh / 2 - 1) {
            placed += 1;
        }
    }
    placed
}

// --- the statues (`Statues` `WorldGen.cs:16853-16970`) ------------------------

/// `SetupStatueList` (`WorldGen.cs:4449-4488`) — the `GenVars.statueList`
/// table (`GenVars.cs:276`) verbatim: the `for i < 44` loop `(105, i)` with
/// the two overwrites (`list[34] = (349, 0)`, `list[43] = (105, 50)`) and
/// the 29 appended styles, in their exact order. Each entry is
/// (tile type, style) — the style feeds `Place2xX`'s frame math. A const
/// table stands in for the C#'s pass-static list: `SetupStatueList` runs in
/// `Reset` (`WorldGen.cs:11228`) and again per cave house
/// (`HouseBuilder.cs:471`) — both calls write the same idempotent data, so
/// one table serves every consumer.
pub(crate) const STATUE_LIST: [(u16, i32); 73] = [
    (105, 0),
    (105, 1),
    (105, 2),
    (105, 3),
    (105, 4),
    (105, 5),
    (105, 6),
    (105, 7),
    (105, 8),
    (105, 9),
    (105, 10),
    (105, 11),
    (105, 12),
    (105, 13),
    (105, 14),
    (105, 15),
    (105, 16),
    (105, 17),
    (105, 18),
    (105, 19),
    (105, 20),
    (105, 21),
    (105, 22),
    (105, 23),
    (105, 24),
    (105, 25),
    (105, 26),
    (105, 27),
    (105, 28),
    (105, 29),
    (105, 30),
    (105, 31),
    (105, 32),
    (105, 33),
    (349, 0), // `list[34]` — the king-statue overwrite
    (105, 35),
    (105, 36),
    (105, 37),
    (105, 38),
    (105, 39),
    (105, 40),
    (105, 41),
    (105, 42),
    (105, 50), // `list[43]` — the second overwrite
    (105, 63),
    (105, 64),
    (105, 65),
    (105, 66),
    (105, 68),
    (105, 69),
    (105, 70),
    (105, 71),
    (105, 72),
    (105, 73),
    (105, 75),
    (105, 51),
    (105, 52),
    (105, 53),
    (105, 54),
    (105, 55),
    (105, 56),
    (105, 57),
    (105, 58),
    (105, 59),
    (105, 60),
    (105, 61),
    (105, 62),
    (105, 77),
    (105, 78),
    (105, 67),
    (105, 74),
    (105, 37),
    (105, 2),
];

/// `GenVars.StatuesWithTraps` (`GenVars.cs:474-475`) — the statueList
/// indices whose placement also arms a dart trap: {4, 7, 10, 18}.
pub(crate) const STATUES_WITH_TRAPS: [usize; 4] = [4, 7, 10, 18];

/// `WorldGen.Place2xX` (`WorldGen.cs:39663-39722`) for the statue family
/// (105/349, reached through `PlaceTile`'s case group `:60066-60077`):
/// the 2×3 body must be clear over both columns, the floor solid under
/// both (`SolidTile2`), then the style frame math — `frameX = style * 36`
/// with the `(105, 34) → (349, 0)` remap and the `style / 55` row offset
/// (`num -= 1980 * (style/55)`, the `frameY` base `54 * (style/55)`).
fn place_statue(world: &mut World, x: i32, y: i32, mut kind: u16, mut style: i32) -> bool {
    let num3 = 3;
    let mut num = style * 36;
    let mut num2 = 0;
    if kind == 105 && style == 34 {
        kind = 349;
        style = 0;
        num = 0;
    }
    if kind == 105 {
        let num4 = style / 55;
        num -= 1980 * num4;
        num2 += 54 * num4;
    }
    let mut ok = true;
    for row in (y - num3 + 1)..(y + 1) {
        for col in x..(x + 2) {
            if world.tile(col, row).is_some_and(|t| t.active()) {
                ok = false;
            }
        }
    }
    if !super::evil::solid_tile2(world, x, y + 1) || !super::evil::solid_tile2(world, x + 1, y + 1) {
        ok = false;
    }
    if !ok {
        return false;
    }
    for row in 0..num3 {
        for col in 0..2 {
            let (px, py) = (x + col, y - num3 + 1 + row);
            if !super::temple::in_world(world, px, py, 0) {
                continue;
            }
            let i = world.index(px, py);
            let t = &mut world.tiles[i];
            t.flags |= FLAG_ACTIVE;
            t.kind = kind;
            t.frame_y = (num2 + row * 18) as i16;
            t.frame_x = (if col == 0 { num } else { num + 18 }) as i16;
        }
    }
    true
}

/// `WorldUtils.WireLine` (`WorldUtils.cs:109-129`) — the L-shaped wire run:
/// the bounding box is normalized on the `end` copy while `point`/`point2`
/// keep the original coordinates, so the horizontal walk rides the start's
/// Y and the vertical walk the end's X. `PlaceWire` (`WorldGen.cs:58111`)
/// sets the first (red) wire colour, skipping cells already wired.
fn wire_line(world: &mut World, x: i32, y: i32, x2: i32, y2: i32) {
    let (mut start_x, mut start_y) = (x, y);
    let (mut end_x, mut end_y) = (x2, y2);
    if end_x < start_x {
        std::mem::swap(&mut end_x, &mut start_x);
    }
    if end_y < start_y {
        std::mem::swap(&mut end_y, &mut start_y);
    }
    for i in start_x..=end_x {
        if !world.wires.get(i, y, crate::wiring::WireColor::Red) {
            world.wires.set(i, y, crate::wiring::WireColor::Red, true);
        }
    }
    for j in start_y..=end_y {
        if !world.wires.get(x2, j, crate::wiring::WireColor::Red) {
            world.wires.set(x2, j, crate::wiring::WireColor::Red, true);
        }
    }
}

/// `WorldGen.PlaceStatueTrap` (`WorldGen.cs:4490-4513`): the ±10 scan for
/// the first empty cell over a solid non-162 support (`SolidTile2`), the
/// dart trap 135 (`Place1x1` — `PlaceTile`'s case group `:59550`), then
/// the wire back to the statue when the trap actually placed.
fn place_statue_trap(world: &mut World, x: i32, y: i32) {
    if !super::temple::in_world(world, x, y, 20) {
        return;
    }
    for i in -10..=10 {
        for j in -10..=10 {
            let (cx, cy) = (x + i, y + j);
            let (bx, by) = (x + i, y + j + 1);
            let support = world.tile(bx, by).copied();
            let empty = world.tile(cx, cy).is_some_and(|t| !t.active());
            // The C# reads `tile2` (the candidate) before `PlaceTile` and
            // re-reads the same reference after — the port re-reads the
            // cell, which the write mutates in place.
            if empty && support.is_some_and(|t| super::evil::solid_tile2(world, bx, by) && t.kind != 162)
            {
                if super::temple::in_world(world, cx, cy, 0) {
                    let idx = world.index(cx, cy);
                    let t = &mut world.tiles[idx];
                    t.flags |= FLAG_ACTIVE;
                    t.kind = 135;
                    t.frame_y = 0; // `Place1x1` style 0: `frameX = 0`, `frameY = 18 * style`
                }
                if world.tile(cx, cy).is_some_and(|t| t.active() && t.kind == 135) {
                    wire_line(world, x, y, cx, cy);
                    return;
                }
            }
        }
    }
}

/// The `Statues` pass (`WorldGen.cs:16853-16970`): `statueList.Length * 2 *
/// maxTilesX / 4200` statues, each placement on its own seat scan (a random
/// x in the `[20, maxTilesX - 20)` band, a random y between the surface
/// band and `maxTilesY - 300`, the `oceanDepths` re-roll, the scan down to
/// the first solid). The list cursor advances only on a placement and wraps
/// at `Length`; a failed seat burns one of 10000 attempts (`num4`). Its own
/// `pass_rng("Statues")` stream models the vanilla per-pass reseed.
///
/// Deviations, both matching this module's header convention: the
/// secret-seed arms (`noTrapsWorldGen`'s ×15, `skyblockWorldGen`, `starGame`)
/// park with M5-SEED-1; the seat's `tileFrameImportant && !tileCut` gate
/// (`:16846`) needs the `Main.tileFrameImportant`/`Main.tileCut` tables,
/// which the port does not carry yet — `Place2xX`'s clear-cell check
/// already refuses to overwrite an occupied seat, so the gate would only
/// skip a redundant `SquareTileFrame` on a busy cell. `Statue`'s paint
/// sweep (`WorldGen.cs:10563-10573`) and its `MarblePileWithStatues`
/// caller (`:10538`) ride the M5-BIOME-2 marble piles.
pub(crate) fn pass_statues(world: &mut World, rng: &mut super::Rng) -> u32 {
    // `num2 = maxTilesX / 4200.0`; `num3 = (int)(statueList.Length * 2 * num2)`.
    let num2 = world.width as f64 / 4200.0;
    let num3 = ((STATUE_LIST.len() * 2) as f64 * num2) as i32;
    // `num4 = 10000` (the skyblock override parks with M5-SEED-1).
    let num4 = 10000;
    // `oceanLevel` (`WorldGen.cs:4447`): `(worldSurface + rockLayer) / 2 + 40`.
    let ocean_level = (world.world_surface + world.rock_layer) / 2.0 + 40.0;
    // The y band floor `(int)(worldSurface * 2.0 + rockLayer) / 3` — the cast
    // binds to the sum, the division is integer division.
    let y_floor = ((world.world_surface * 2.0 + world.rock_layer) as i32) / 3;
    let mut placed = 0u32;
    let mut num = 0usize;
    for _ in 0..num3 {
        if num >= STATUE_LIST.len() {
            num = 0;
        }
        let (kind, style) = STATUE_LIST[num];
        let mut flag = false;
        let mut num5 = 0;
        while !flag {
            let mut num6 = rng.range(20, world.width - 20);
            let mut num7 = rng.range(y_floor, world.height - 300);
            let mut num8 = 100;
            while super::evil::ocean_depths(world, num6, num7, ocean_level) {
                num8 -= 1;
                if num8 <= 0 {
                    break;
                }
                num6 = rng.range(20, world.width - 20);
                num7 = rng.range(y_floor, world.height - 300);
            }
            if num8 <= 0 {
                flag = true;
            } else {
                while !world.tile(num6, num7).is_some_and(|t| t.active()) {
                    num7 += 1;
                    if num7 >= world.height {
                        break;
                    }
                }
                if num7 >= world.height {
                    flag = true;
                } else {
                    num7 -= 1;
                    // `PlaceTile` calls `SquareTileFrame` unconditionally
                    // (`:60077`), whatever `Place2xX` did with the seat.
                    place_statue(world, num6, num7, kind, style);
                    crate::edit::square_tile_frame_gen(world, num6, num7);
                    // The stale-reference re-read: `tile` is the same object
                    // `PlaceTile` mutated (`:16857`), so the check sees the
                    // placed statue.
                    let landed = world
                        .tile(num6, num7)
                        .is_some_and(|t| t.active() && t.kind == kind);
                    if landed {
                        flag = true;
                        if STATUES_WITH_TRAPS.contains(&num) {
                            place_statue_trap(world, num6, num7);
                        }
                        num += 1;
                        placed += 1;
                    } else {
                        num5 += 1;
                        if num5 >= num4 {
                            flag = true;
                        }
                    }
                }
            }
        }
    }
    placed
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tile::Tile;
    use crate::world::World;
    use crate::worldgen::{GenOptions, generate};

    /// A 3-row floor at y=30 — the `SolidTile2` seat. 60×60 so the
    /// statue-trap scan (the ±10 window behind `InWorld`'s 20-tile fluff,
    /// `WorldGen.cs:9688`) has room.
    fn test_world() -> World {
        let mut w = World::new("t".into(), 60, 60);
        for x in 2..58 {
            for y in 30..33 {
                let i = w.index(x, y);
                let t = &mut w.tiles[i];
                t.flags |= crate::tile::FLAG_ACTIVE;
                t.kind = 1; // dirt — `TILE_SOLID`
            }
        }
        w.world_surface = 25.0;
        w.rock_layer = 45.0;
        w
    }

    #[test]
    fn the_statue_list_transcribes_the_source_table() {
        // `SetupStatueList` (`WorldGen.cs:4449-4488`): 44 loop entries
        // (105, 0..43), the two overwrites, 29 appends.
        assert_eq!(STATUE_LIST.len(), 73);
        for (i, (kind, style)) in STATUE_LIST.iter().take(44).enumerate() {
            let expected = match i {
                34 => (349, 0),
                43 => (105, 50),
                _ => (105, i as i32),
            };
            assert_eq!((*kind, *style), expected, "list[{i}]");
        }
        assert_eq!(STATUE_LIST[34], (349, 0), "`list[34] = (349, 0)`");
        assert_eq!(STATUE_LIST[43], (105, 50), "`list[43] = (105, 50)`");
        // The appended tail in its exact order.
        let appended: Vec<i32> = STATUE_LIST[44..].iter().map(|&(_, s)| s).collect();
        assert_eq!(
            appended,
            vec![
                63, 64, 65, 66, 68, 69, 70, 71, 72, 73, 75, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60,
                61, 62, 77, 78, 67, 74, 37, 2
            ]
        );
        // Only index 34 leaves the 105 family.
        assert_eq!(
            STATUE_LIST.iter().filter(|&&(k, _)| k != 105).count(),
            1
        );
    }

    #[test]
    fn place_statue_writes_the_2x3_frames() {
        let mut w = test_world();
        // style 2 → `frameX = 2 * 36` over the two columns, `frameY` 0/18/36.
        assert!(place_statue(&mut w, 10, 29, 105, 2));
        for row in 0..3 {
            for col in 0..2 {
                let t = w.tile(10 + col, 27 + row).unwrap();
                assert!(t.active());
                assert_eq!(t.kind, 105);
                assert_eq!(t.frame_x, (36 * 2 + col * 18) as i16);
                assert_eq!(t.frame_y, (row * 18) as i16);
            }
        }
        // A style past 55 takes the row offset (54 + the −1980 x-shift).
        assert!(place_statue(&mut w, 14, 29, 105, 56));
        let t = w.tile(14, 27).unwrap();
        assert_eq!(t.frame_x, (56 * 36 - 1980) as i16, "`num -= 1980 * (style/55)`");
        assert_eq!(t.frame_y, 54, "`num2 += 54 * (style/55)`");
        // The (105, 34) remap lands as a 349 king statue at style 0.
        assert!(place_statue(&mut w, 18, 29, 105, 34));
        let t = w.tile(18, 27).unwrap();
        assert_eq!(t.kind, 349);
        assert_eq!(t.frame_x, 0);
    }

    #[test]
    fn place_statue_refuses_an_occupied_seat() {
        let mut w = test_world();
        // A torch in the body blocks the whole 2×3.
        let i = w.index(20, 28);
        w.tiles[i] = Tile {
            kind: 4,
            flags: crate::tile::FLAG_ACTIVE,
            ..Tile::default()
        };
        assert!(!place_statue(&mut w, 20, 29, 105, 1));
        assert!(w.tile(20, 27).is_some_and(|t| !t.active()), "nothing written");
        // No floor under the feet either.
        let mut bare = test_world();
        let i = bare.index(30, 30);
        bare.tiles[i] = Tile::default();
        let i = bare.index(31, 30);
        bare.tiles[i] = Tile::default();
        assert!(!place_statue(&mut bare, 30, 29, 105, 1));
    }

    #[test]
    fn place_statue_trap_arms_the_nearest_plate() {
        let mut w = test_world();
        place_statue(&mut w, 30, 29, 105, 3);
        place_statue_trap(&mut w, 30, 29);
        let traps: Vec<(i32, i32)> = (0..w.width)
            .flat_map(|x| (0..w.height).map(move |y| (x, y)))
            .filter(|&(x, y)| w.tile(x, y).is_some_and(|t| t.active() && t.kind == 135))
            .collect();
        assert_eq!(traps.len(), 1, "one dart trap, the first legal plate");
        let (tx, ty) = traps[0];
        // The plate sits on the floor row (the support is the row below).
        assert_eq!(ty, 29, "the empty cell above the solid row");
        // The L-shaped wire reaches the statue.
        assert!(
            w.wires.get(tx, ty, crate::wiring::WireColor::Red),
            "the plate itself is wired"
        );
        assert!(
            w.wires.get(30, 29, crate::wiring::WireColor::Red),
            "the statue cell is wired"
        );
    }

    #[test]
    fn the_statues_pass_places_the_table_in_a_default_world() {
        // `num3 = (int)(73 * 2 * 4200/4200) = 146` statues on the default
        // world — the cursor wraps exactly once, so the single 349 entry
        // (index 34) lands twice: 12 tiles of king statue.
        let w = generate(&GenOptions::default());
        let count = |kind: u16| w.tiles.iter().filter(|t| t.active() && t.kind == kind).count();
        assert_eq!(count(349), 12, "two king statues ({})", count(349));
        // The temple's own statues share tile 105; the pass's 105s sit on
        // top of that floor.
        assert!(count(105) > 800, "the statue tiles ({})", count(105));
        // Every statue carries its list style — the frames are never the
        // unset −1.
        let framed = w
            .tiles
            .iter()
            .filter(|t| t.active() && (t.kind == 105 || t.kind == 349))
            .all(|t| t.frame_x >= 0 && t.frame_y >= 0);
        assert!(framed, "the placed statues carry list frames");
    }
}
