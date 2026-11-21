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
/// (`TileObject`'s chest frames — the port's `frame_x = 18 * style`
/// convention, `tileobject.rs:2072-2082`). `style` is the C#'s resolved
/// `num9` (`:35978-35988`): the caller's `chestStyle` when `>= 0`, else the
/// `flag15` default (1 — the gold chest — below `worldSurface + 25`).
pub(crate) fn place_chest(world: &mut World, x: i32, y: i32, style: i32) -> bool {
    let base = (18 * style) as i16;
    for (dx, dy, fx, fy) in [
        (0i32, 0i32, base, 0i16),
        (1, 0, base + 18, 0),
        (0, 1, base, 18),
        (1, 1, base + 18, 18),
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
        (0i32, 0i32, base, 0i16),
        (1, 0, base + 18, 0),
        (0, 1, base, 18),
        (1, 1, base + 18, 18),
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

/// The resolved chest style — `AddBuriedChest`'s `num9` arm
/// (`WorldGen.cs:35978-35988`): the caller's `chestStyle` when it is
/// `>= 0`, else `1` (the gold chest) once the anchor sits below
/// `worldSurface + 25` (`flag15`; the `mainItemInChest > 0` disjunct rides
/// the same default). The remix variant of `flag15` stays out (M5-SEED-2).
pub(crate) fn resolved_chest_style(chest_style: i32, main_item: u16, y: i32, world_surface: f64) -> i32 {
    if chest_style >= 0 {
        return chest_style;
    }
    let flag15 = y as f64 >= world_surface + super::chest_stock::SURFACE_BAND_PAD as f64;
    if flag15 || main_item > 0 {
        1
    } else {
        0
    }
}

/// `GetNextJungleChestItem` (`WorldGen.cs:10170-10196`): the jungle chests'
/// main item — the `JungleItemCount % 4` cycle over the living-wood family
/// {211, 212, 213, 964} with the two rare overrides (`Next(15) == 0` →
/// 2292, else `Next(20) == 0` → 3017), the counter advancing once per
/// item (`:10196`). The vanilla draw order is exact: the cycle pick is
/// branch-free, the `Next(15)` gate rolls first, the `Next(20)` gate only
/// rolls on its else. Drawn at the shrine-chest site per spot (S1-A-4 —
/// the C# calls it before `AddBuriedChest`, even for a failed place).
pub(crate) fn get_next_jungle_chest_item(cycle: &mut u32, rng: &mut super::Rng) -> u16 {
    let mut item = match *cycle % 4 {
        0 => 211,
        1 => 212,
        2 => 213,
        _ => 964,
    };
    if rng.range(0, 15) == 0 {
        item = 2292;
    } else if rng.range(0, 20) == 0 {
        item = 3017;
    }
    *cycle += 1;
    item
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
/// layer scatter the rest (the count `maxTilesX / 40` shape). S1-A-4: the
/// shrine arm draws the C#'s per-spot sequence — the jungle item
/// (`GetNextJungleChestItem`, `:10170-10196`), the `Next(2)` x-jitter
/// (`:17220`) — and both arms stock inline through the hook, exactly
/// `AddBuriedChest`'s shape.
pub(crate) fn pass_chests(
    world: &mut World,
    shrine_spots: &[(i32, i32)],
    jungle_cycle: &mut u32,
    rng: &mut super::Rng,
    mut stock: Option<&mut super::chest_stock::ChestStockHook<'_>>,
) -> u32 {
    let mut placed = 0u32;
    for (x, y) in shrine_spots {
        // `:17219-17220` — the item draw and the `Next(2)` jitter ride the
        // stream per attempted spot, before the placement.
        let main = get_next_jungle_chest_item(jungle_cycle, rng);
        let jitter = rng.range(0, 2);
        let cx = *x + jitter;
        let cy = *y + 1;
        let style = 10; // `chestStyle: 10` (`:17220`) — the jungle chest.
        if place_chest(world, cx, cy, style) {
            super::chest_stock::stock_placed_chest(
                world,
                stock.as_deref_mut(),
                rng,
                cx,
                cy,
                style,
                TILE_CHEST,
                main,
                false,
            );
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
        // `AddBuriedChest(num6, num7, 0, false, -1, false, 0)` (`:17024`) —
        // the plain gold-chest arm: main item 0, the `num9` default.
        let style = resolved_chest_style(-1, 0, y, world.world_surface);
        if place_chest(world, x, y, style) {
            super::chest_stock::stock_placed_chest(
                world,
                stock.as_deref_mut(),
                rng,
                x,
                y,
                style,
                TILE_CHEST,
                0,
                false,
            );
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
    mut stock: Option<&mut super::chest_stock::ChestStockHook<'_>>,
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
        // The pyramid's own chest (`WorldGen.cs:27853-27873`): the main
        // item's double roll — `Next(3)`, a `0` re-rolls `Next(3)`, the
        // tenth-anniversary `0 → 1` (M5-SEED-1 keeps it out) — then the
        // style-1 place and the inline stock (S1-A-4).
        let mut roll = rng.range(0, 3);
        if roll == 0 {
            roll = rng.range(0, 3);
        }
        let main = match roll {
            0 => 848, // `Pharaoh'sMask`
            1 => 857, // `SandstorminaBottle`
            _ => 934, // `FlyingCarpet`
        };
        let style = resolved_chest_style(1, main, base - 2, world.world_surface);
        if place_chest(world, x0 - 2, base - 2, style) {
            super::chest_stock::stock_placed_chest(
                world,
                stock.as_deref_mut(),
                rng,
                x0 - 2,
                base - 2,
                style,
                TILE_CHEST,
                main,
                false,
            );
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
    // The honey patches ride the hive pass (`WorldGen.cs:15946-15968`):
    // each placed hive's `Next(5)` patches share its stream, so the rects
    // list and the rng cross the hive body.
    let mut protected: Vec<(i32, i32, i32, i32)> = Vec::new();
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
        // The hive's honey patches (`WorldGen.cs:15946-15968`).
        super::mini_biomes::place_honey_patches_around(world, cx, cy, &mut protected, rng);
        protected.push((cx - r, cy - r, 2 * r, 2 * r));
        placed += 1;
    }
    placed
}

/// `HellFort` (`WorldGen.cs:32683+`, the tile 75 / the wall 14 defaults):
/// the brick boxes in the hell layer — the shell with the carved rooms, the
/// lava pools on the floors and the chests.
pub(crate) fn pass_hell_forts(
    world: &mut World,
    hell_top: i32,
    rng: &mut super::Rng,
    mut stock: Option<&mut super::chest_stock::ChestStockHook<'_>>,
) -> u32 {
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
        // The chest in the upper room — the hell house's own chest
        // (`WorldGen.cs:29290-29341`, the non-remix arm): the main item is
        // `832`, a `Next(3) == 0` swaps in `4281` (`:29292-29297`), the
        // style is the caller's `chestStyle: 12` (`:29341`). S1-A-4: the
        // stock rides inline.
        let main = if rng.range(0, 3) == 0 { 4281 } else { 832 };
        let style = resolved_chest_style(12, main, cy - hh / 2 - 1, world.world_surface);
        if place_chest(world, cx - 3, cy - hh / 2 - 1, style) {
            super::chest_stock::stock_placed_chest(
                world,
                stock.as_deref_mut(),
                rng,
                cx - 3,
                cy - hh / 2 - 1,
                style,
                TILE_CHEST,
                main,
                false,
            );
            placed += 1;
        }
    }
    placed
}

// --- Life Crystals (W4-g36 A7) -------------------------------------------------

/// `TileID.LifeCrystal` (`TileID.cs:609`) / `TileID.ManaCrystal`
/// (`TileID.cs:1435`) — the 2×2 heart / star.
const TILE_LIFE_CRYSTAL: u16 = 12;
const TILE_MANA_CRYSTAL: u16 = 639;
/// `Main.offLimitBorderTiles` (`Main.cs:66581`) — the 40-tile placement
/// margin.
const OFF_LIMIT_BORDER: i32 = 40;
/// `Main.wallDungeon` (`Main.cs`) — the nine dungeon walls.
const WALL_DUNGEON: [u16; 9] = [7, 8, 9, 94, 95, 96, 97, 98, 99];

/// `Tile.anyLava` / `Tile.anyShimmer` (`Tile.cs:410-435`): liquid > 0 AND
/// the kind bit.
fn any_lava(t: &Tile) -> bool {
    t.liquid > 0 && t.flags & crate::tile::FLAG_LIQUID_LAVA != 0
}
fn any_shimmer(t: &Tile) -> bool {
    t.liquid > 0 && t.flags & crate::tile::FLAG_LIQUID_SHIMMER != 0
}

/// `WorldGen.AddLifeCrystal` (`WorldGen.cs:31912-31979`) /
/// `AddManaCrystal` (`:31981-32048`) — the two bodies are identical but
/// for the placed kind (12 / 639) and the skyblock proximity kind
/// (12 / 639); `kind` selects both. The column scan drops from `j` to the
/// first active solid tile, then the 2×2 seat validation: no lava/shimmer
/// in the two cells above the floor, `EmptyTileCheck(i-1, i, num-1, num)`
/// (ignore −1 — every active cell refuses), no dungeon wall behind the
/// right foot, both floor cells solid (`nactive` + tileSolid), the
/// blockType slope/half-brick flatten, then the four frame writes.
/// RNG-free. `skyblock` enables the `IsTileNearby(i, num, kind, 50)` veto
/// (`:31935-31938` / `:32004-32007`) — the life crystal refuses near
/// another life crystal (12), the mana crystal near another mana (639).
fn add_crystal(world: &mut World, i: i32, j: i32, kind: u16, skyblock: bool) -> bool {
    for k in j..world.height {
        let floor = world.tile(i, k);
        if !floor.is_some_and(|t| t.active() && crate::tiledata::TILE_SOLID.has(t.kind)) {
            continue;
        }
        let num = k - 1;
        // The lava / shimmer vetoes (`:31919-31926`).
        let above = world.tile(i, num - 1).copied().unwrap_or_default();
        let above_left = world.tile(i - 1, num - 1).copied().unwrap_or_default();
        if any_lava(&above) || any_lava(&above_left) || any_shimmer(&above) {
            return false;
        }
        // `EmptyTileCheck(i-1, i, num-1, num)` — ignore −1 (`:31927`).
        if !super::biome_detail::empty_tile_check(world, i - 1, i, num - 1, num, -1) {
            return false;
        }
        // The dungeon-wall veto (`:31931`).
        if WALL_DUNGEON.contains(&world.tile(i, num).map_or(0, |t| t.wall)) {
            return false;
        }
        // The skyblock proximity veto (`:31935-31938`).
        if skyblock && is_tile_nearby(world, i, num, kind, 50) {
            return false;
        }
        // The two floor cells (`:31939-31948`): nactive + tileSolid.
        let left = world.tile(i - 1, num + 1).copied().unwrap_or_default();
        let right = world.tile(i, num + 1).copied().unwrap_or_default();
        let solid_floor = |t: &Tile| {
            t.active() && !t.inactive() && crate::tiledata::TILE_SOLID.has(t.kind)
        };
        if !solid_floor(&left) || !solid_floor(&right) {
            return false;
        }
        // The blockType flatten (`:31949-31958`).
        for (dx, t) in [(-1i32, left), (0, right)] {
            if crate::tileobject::block_type(&t) != 0 {
                if let Some(cell) = world.tile_mut(i + dx, num + 1) {
                    cell.set_slope(0);
                    cell.flags &= !crate::tile::FLAG_HALF_BRICK;
                }
            }
        }
        // The four frame writes (`:31959-31974`).
        for (dx, dy, fx, fy) in [(-1i32, -1i32, 0i16, 0i16), (0, -1, 18, 0), (-1, 0, 0, 18), (0, 0, 18, 18)] {
            let idx = world.index(i + dx, num + dy);
            let t = &mut world.tiles[idx];
            t.flags |= FLAG_ACTIVE;
            t.kind = kind;
            t.frame_x = fx;
            t.frame_y = fy;
        }
        return true;
    }
    false
}

/// `WorldGen.IsTileNearby` (`WorldGen.cs:8862-8884`): the ±distance box
/// scan for an active tile of `kind` (the 235 stride-3 special case is
/// not reached here — the crystal kinds are 12/639).
fn is_tile_nearby(world: &World, x: i32, y: i32, kind: u16, distance: i32) -> bool {
    for i in (x - distance)..=(x + distance) {
        for j in (y - distance)..=(y + distance) {
            if world.tile(i, j).is_some_and(|t| t.active() && t.kind == kind) {
                return true;
            }
        }
    }
    false
}

/// The Life Crystals pass (`GenPassNameID.LifeCrystals`,
/// `WorldGen.cs:16755-16852`): `(int)(width*height * 2e-5)` life crystals
/// (×1.2 anniversary), each a retry loop dropping from a random
/// `(worldSurface*2 + rockLayer)/3 .. height-300` row (remix:
/// `worldSurface .. height-400`) until `AddLifeCrystal` seats one or the
/// 10000-attempt cap (skyblock: 10) fires. The errorWorld/noSurface mana
/// loop (`:16812-16849`) is the same shape at half density with
/// `AddManaCrystal`. The `Skyblock.denySomeGeneration` gate is the
/// caller's. Own `pass_rng("Life Crystals")` stream.
///
/// Deferred seed arms (documented, the whole-seed-body convention):
/// `DoErrorWorldShuffleBlocks` (`:16759-16762`, `:1094-1140`), the
/// `getGoodWorldGen` `tileSolid[56] = false` toggle (`:16763-16766`),
/// `NotTheBees` (`:16767-16770`, `:25107-25300`), and the closing
/// `tileSolid[225] = false` (`:16850`) — the port's boot tileSolid table
/// stands (the `biome_detail::pass_underground_jungle_trees` toggle-model
/// note). `Main.starGame`'s `starGameMath(0.2)` multiplier is 1.0 at gen
/// time (`starsHit == 0`, `Main.cs:62834-62846`) — a no-op.
pub(crate) fn pass_life_crystals(
    world: &mut World,
    world_surface: f64,
    rock_layer: f64,
    anniversary: bool,
    remix: bool,
    skyblock: bool,
    error_or_no_surface: bool,
    rng: &mut super::Rng,
) {
    let area = (world.width * world.height) as f64;
    let mut count = area * 2e-5;
    if anniversary {
        count *= 1.2;
    }
    // `num2 = 10000`, skyblock 10 (`:16781-16785`).
    let cap = if skyblock { 10 } else { 10000 };
    let y_lo = ((world_surface * 2.0 + rock_layer) / 3.0) as i32;
    let y_hi = world.height - 300;
    let (remix_lo, remix_hi) = (world_surface as i32, world.height - 400);
    // Sub-vanilla guard: the y-band collapses on the tiny test worlds
    // (the `pass_altars` idiom) — skip before any draw.
    if y_hi <= y_lo || (remix && remix_hi <= remix_lo) {
        return;
    }
    for _ in 0..(count as i32) {
        let mut attempts = 0i32;
        loop {
            let j = if remix {
                rng.range(remix_lo, remix_hi)
            } else {
                rng.range(y_lo, y_hi)
            };
            let x = rng.range(OFF_LIMIT_BORDER, world.width - OFF_LIMIT_BORDER);
            if add_crystal(world, x, j, TILE_LIFE_CRYSTAL, skyblock) {
                break;
            }
            attempts += 1;
            if attempts >= cap {
                break;
            }
        }
    }
    // The errorWorld/noSurface mana loop (`:16812-16849`).
    if error_or_no_surface {
        let mut mana_count = area * 1e-5;
        if anniversary {
            mana_count *= 1.2;
        }
        for _ in 0..(mana_count as i32) {
            let mut attempts = 0i32;
            loop {
                let j = if remix {
                    rng.range(remix_lo, remix_hi)
                } else {
                    rng.range(y_lo, y_hi)
                };
                let x = rng.range(OFF_LIMIT_BORDER, world.width - OFF_LIMIT_BORDER);
                if add_crystal(world, x, j, TILE_MANA_CRYSTAL, skyblock) {
                    break;
                }
                attempts += 1;
                if attempts >= cap {
                    break;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A stone world with a flat floor at row `floor` — the crystal seat.
    fn floor_world(w: i32, h: i32, floor: i32) -> World {
        let mut world = World::new("lc".into(), w, h);
        for x in 0..w {
            for y in floor..h {
                let i = world.index(x, y);
                world.tiles[i].kind = 1;
                world.tiles[i].flags |= FLAG_ACTIVE;
            }
        }
        world
    }

    /// W4-g36 A7: `AddLifeCrystal` seats the 2×2 heart on a flat floor —
    /// the four frames (0/0, 18/0, 0/18, 18/18) at kind 12.
    #[test]
    fn a7_add_life_crystal_seats_the_heart() {
        let mut w = floor_world(120, 120, 60);
        assert!(add_crystal(&mut w, 60, 40, TILE_LIFE_CRYSTAL, false));
        // The floor is row 60, so num = 59; the heart occupies rows 58-59.
        assert_eq!(w.tile(59, 58).unwrap().kind, TILE_LIFE_CRYSTAL);
        assert_eq!((w.tile(59, 58).unwrap().frame_x, w.tile(59, 58).unwrap().frame_y), (0, 0));
        assert_eq!((w.tile(60, 58).unwrap().frame_x, w.tile(60, 58).unwrap().frame_y), (18, 0));
        assert_eq!((w.tile(59, 59).unwrap().frame_x, w.tile(59, 59).unwrap().frame_y), (0, 18));
        assert_eq!((w.tile(60, 59).unwrap().frame_x, w.tile(60, 59).unwrap().frame_y), (18, 18));
    }

    /// W4-g36 A7: lava above the floor refuses the seat; a dungeon wall
    /// behind the right foot refuses too.
    #[test]
    fn a7_lava_and_dungeon_wall_refuse() {
        let mut w = floor_world(120, 120, 60);
        let i = w.index(60, 58);
        w.tiles[i].liquid = 100;
        w.tiles[i].flags |= crate::tile::FLAG_LIQUID_LAVA;
        assert!(!add_crystal(&mut w, 60, 40, TILE_LIFE_CRYSTAL, false));
        // Clear the lava, add a dungeon wall behind the right foot (row 59).
        let mut w2 = floor_world(120, 120, 60);
        w2.tiles[w2.index(60, 59)].wall = 7;
        assert!(!add_crystal(&mut w2, 60, 40, TILE_LIFE_CRYSTAL, false));
    }

    /// W4-g36 A7: the pass is deterministic (a == b) and places crystals
    /// on a vanilla-sized world; the sub-vanilla y-band guard skips the
    /// tiny worlds without a draw.
    #[test]
    fn a7_the_pass_replays_and_guards_the_tiny_worlds() {
        let mut a = floor_world(4200, 1200, 400);
        let mut b = floor_world(4200, 1200, 400);
        pass_life_crystals(&mut a, 360.0, 500.0, false, false, false, false, &mut super::super::Rng(42));
        pass_life_crystals(&mut b, 360.0, 500.0, false, false, false, false, &mut super::super::Rng(42));
        assert_eq!(a.tiles, b.tiles, "a == b");
        assert!(
            a.tiles.iter().any(|t| t.kind == TILE_LIFE_CRYSTAL),
            "the pass must seat life crystals"
        );
        // The 512-tall world: y_hi = 212 <= y_lo — the guard skips.
        let mut small = floor_world(512, 512, 400);
        let before = small.tiles.clone();
        let mut rng = super::super::Rng(42);
        pass_life_crystals(&mut small, 100.0, 220.0, false, false, false, false, &mut rng);
        assert_eq!(small.tiles, before, "the y-band guard writes nothing");
        assert_eq!(rng.0, super::super::Rng(42).0, "the guard draws nothing");
    }

    /// W4-g36 A7: the errorWorld/noSurface mana loop seats mana crystals
    /// (639) at half density.
    #[test]
    fn a7_the_mana_loop_seats_stars() {
        let mut w = floor_world(4200, 1200, 400);
        pass_life_crystals(&mut w, 360.0, 500.0, false, false, false, true, &mut super::super::Rng(7));
        assert!(
            w.tiles.iter().any(|t| t.kind == TILE_MANA_CRYSTAL),
            "the mana loop must seat mana crystals"
        );
    }
}
