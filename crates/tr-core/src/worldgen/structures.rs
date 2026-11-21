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

/// `TileID.Hellforge` (`TileID.cs`: 77) — the 3×2 forge the hell houses
/// host (`WorldGen.cs:60079-60083` routes it through `Place3x2`).
pub(crate) const TILE_HELLFORGE: u16 = 77;

/// The `Hellforge` pass (`WorldGen.cs:18187-18234`, W4-d104 pass 30):
/// `maxTilesX / 200` forges, each a retry loop — the `Next(1, maxTilesX)`
/// column + the `Next(maxTilesY - 250, maxTilesY - 30)` row, the wall
/// 13/14 (the hell-house brick walls) gate, the drop to the first active
/// cell (`j < maxTilesY - 20` bound), then `PlaceTile(num3, j - 1, 77)`
/// (the `Place3x2` route). The 10000-failure valve ends the retry. The
/// C#'s `try/catch` covers the out-of-bounds reads — the port's
/// bounds-checked `tile` reads stand in (a miss counts as a failure).
pub(crate) fn pass_hellforge(world: &mut World, rng: &mut super::Rng) -> u32 {
    let num = world.width / 200;
    let mut placed = 0u32;
    for _ in 0..num {
        let mut flag = false;
        let mut num2 = 0;
        while !flag {
            let num3 = rng.range(1, world.width);
            let mut j = rng.range(world.height - 250, world.height - 30);
            let wall = world.tile(num3, j).map(|t| t.wall).unwrap_or(0);
            if wall == 13 || wall == 14 {
                // The drop scan (`:18205-18208`): the first active cell,
                // bounded by `maxTilesY - 20`.
                while !world.tile(num3, j).is_some_and(|t| t.active()) && j < world.height - 20 {
                    j += 1;
                }
                j -= 1;
                // `PlaceTile(num3, j, 77)` — the `Place3x2` route
                // (`:60079-60083`), style 0.
                super::evil::place_3x2(world, num3, j, TILE_HELLFORGE, 0);
                if world.tile(num3, j).is_some_and(|t| t.kind == TILE_HELLFORGE) {
                    flag = true;
                    placed += 1;
                } else {
                    num2 += 1;
                    if num2 >= 10000 {
                        flag = true;
                    }
                }
            }
            // The C#'s wall-miss arm draws again with no failure count
            // (only the place-failure and the catch arms bump `num2`).
        }
    }
    placed
}

// --- Piles (W4-d104 pass 35) -------------------------------------------------

/// `Main.tileDungeon` (`Main.cs:7992-7997`).
fn pile_tile_dungeon(kind: u16) -> bool {
    matches!(kind, 41 | 43 | 44 | 677 | 678 | 679)
}

/// `Main.wallDungeon` (`Main.cs:10596-10607`).
fn pile_wall_dungeon(wall: u16) -> bool {
    matches!(wall, 7 | 8 | 9 | 94..=99)
}

/// `WorldGen.GetPileGenerationAttempts` (`WorldGen.cs:23647-23655`):
/// `maxTilesX / 2`, the skyblock /10.
fn pile_attempts(world: &World) -> i32 {
    let mut num = world.width / 2;
    if world.seeds.contains(crate::seeds::SeedFlags::SKYBLOCK) {
        num /= 10;
    }
    num
}

/// The Piles window's `SolidTile2` floor predicate
/// (`WorldGen.cs:70134-70151`): the pass flips the cloud/sunplate/geyser/
/// boulder kinds non-solid (`:18794-18805`), so the pile seats on them
/// refuse. The port's table is static — the window rides the predicate.
fn pile_floor(world: &World, x: i32, y: i32) -> bool {
    let Some(t) = world.tile(x, y) else {
        return true;
    };
    if matches!(t.kind, 379 | 229 | 190 | 196 | 189 | 717 | 718 | 719 | 202 | 460 | 484) {
        return false;
    }
    t.active()
        && crate::tiledata::TILE_SOLID.has(t.kind)
        && t.slope() == 0
        && !t.half_brick()
        && !t.inactive()
}

/// `WorldGen.PlaceSmallPile` (`WorldGen.cs:46815-46873`) under the Piles
/// window — the seat reads [`pile_floor`], the 185 boulder veto
/// (`InvalidTileForPilesOrSpeleothems`) is inert during worldgen (D20).
fn place_small_pile_window(
    world: &mut World,
    x: i32,
    y: i32,
    pile_style: i32,
    pile_size: i32,
) -> bool {
    let frame_y = (pile_size * 18) as i16;
    if world
        .tile(x, y)
        .is_some_and(|t| t.liquid > 0 && t.liquid_kind() == crate::tile::LiquidKind::Lava)
    {
        return false;
    }
    if pile_size == 1 {
        let frame_x = (pile_style * 36) as i16;
        let seat = pile_floor(world, x, y + 1)
            && pile_floor(world, x + 1, y + 1)
            && !world.tile(x, y).is_some_and(|t| t.active())
            && !world.tile(x + 1, y).is_some_and(|t| t.active());
        if !seat {
            return false;
        }
        for (i, fx) in [(x, frame_x), (x + 1, frame_x + 18)] {
            if let Some(t) = world.tile_mut(i, y) {
                t.kind = 185;
                t.flags |= FLAG_ACTIVE;
                t.frame_x = fx;
                t.frame_y = frame_y;
            }
        }
        true
    } else if pile_floor(world, x, y + 1) && !world.tile(x, y).is_some_and(|t| t.active()) {
        if let Some(t) = world.tile_mut(x, y) {
            t.kind = 185;
            t.flags |= FLAG_ACTIVE;
            t.frame_x = (pile_style * 18) as i16;
            t.frame_y = frame_y;
        }
        true
    } else {
        false
    }
}

/// The `PlaceTile(num4, j, 186/187, style)` route — both pile kinds go
/// through `Place3x2` (`WorldGen.cs:59993-60002`).
fn place_pile(world: &mut World, x: i32, y: i32, kind: u16, style: i32) {
    super::evil::place_3x2(world, x, y, kind, style);
}

/// The small-pile cluster the large-pile scatters append
/// (`:18910-18924` etc.): `Next(1, 5)` satellites at `±Next(-10, 11)` /
/// `-Next(5)`, each dropped to its floor and `PlaceSmallPile(Next(12, 36),
/// 0, 185)`.
fn pile_cluster(world: &mut World, x: i32, y: i32, rng: &mut super::Rng) {
    let num7 = rng.range(1, 5);
    for _ in 0..num7 {
        let num8 = x + rng.range(-10, 11);
        let mut l = y - rng.range(0, 5);
        if !world.tile(num8, l).is_some_and(|t| t.active()) {
            while !world.tile(num8, l + 1).is_some_and(|t| t.active()) && l < world.height - 5 {
                l += 1;
            }
            place_small_pile_window(world, num8, l, rng.range(12, 36), 0);
        }
    }
}

/// The `Piles` pass (`WorldGen.cs:18788-19458`, W4-d104 pass 35): the
/// seven decor scatters — the large-pile underground (`width*0.06`), the
/// large-pile deep (`width*0.01`), the large-pile surface (`width*0.03`),
/// the large-pile walled-surface (`width*0.0035`), the small-pile
/// underground (`width*0.6`), the small-pile surface (`width*0.02`), the
/// small-pile dirt/snow-wall surface (`width*0.15`). The
/// `Skyblock.denySomeGeneration` gate rides the step adapter; the
/// `tileSolid` window rides [`pile_floor`].
pub(crate) fn pass_piles(
    world: &mut World,
    world_surface: f64,
    rock_layer: f64,
    rng: &mut super::Rng,
) {
    // The sub-vanilla width guard: the column draw `Next(25, width-25)`
    // throws on a ≤50-wide world.
    if world.width <= 50 {
        return;
    }
    let ocean_level = (world_surface + rock_layer) / 2.0 + 40.0;
    let attempts = pile_attempts(world);
    // --- scatter 1: the large-pile underground (`:18809-18927`) ---------
    let num2 = (world.width as f32 * 0.06) as i32;
    for _ in 0..num2 {
        let mut num3 = attempts;
        let mut flag = false;
        while !flag && num3 > 0 {
            num3 -= 1;
            let mut num4 = rng.range(25, world.width - 25);
            let mut j = rng.range(world_surface as i32, world.height - 300);
            while super::traps::ocean_depths_trap(world, num4, j, ocean_level) {
                num4 = rng.range(25, world.width - 25);
                j = rng.range(world_surface as i32, world.height - 300);
            }
            if !world.tile(num4, j).is_some_and(|t| t.active()) {
                let mut num5: u16 = 186;
                while !world.tile(num4, j + 1).is_some_and(|t| t.active()) && j < world.height - 5 {
                    j += 1;
                }
                let wall = world.tile(num4, j).map(|t| t.wall).unwrap_or(0);
                let shimmer = world
                    .tile(num4, j)
                    .is_some_and(|t| t.liquid > 0 && t.liquid_kind() == crate::tile::LiquidKind::Shimmer);
                let Some(below) = world.tile(num4, j + 1).copied() else {
                    continue;
                };
                if below.active() {
                    let mut num6 = rng.range(0, 22);
                    if (16..=22).contains(&num6) {
                        num6 = rng.range(0, 22);
                    }
                    if (below.kind == 0 || below.kind == 1 || crate::tiledata::TILE_MOSS.has(below.kind))
                        && rng.range(0, 5) == 0
                    {
                        num6 = rng.range(23, 29);
                        num5 = 187;
                    }
                    if j > world.height - 300
                        || pile_wall_dungeon(wall)
                        || below.kind == 30
                        || below.kind == 19
                        || below.kind == 25
                        || below.kind == 203
                    {
                        num6 = rng.range(0, 7);
                        num5 = 186;
                    }
                    if matches!(below.kind, 147 | 161 | 162) {
                        num6 = rng.range(26, 32);
                        num5 = 186;
                    }
                    if below.kind == 60 {
                        num5 = 187;
                        num6 = rng.range(0, 6);
                    }
                    if matches!(below.kind, 57 | 58) && rng.range(0, 3) < 2 {
                        num5 = 187;
                        num6 = rng.range(6, 9);
                    }
                    if below.kind == 226 {
                        num5 = 187;
                        num6 = rng.range(18, 23);
                    }
                    if below.kind == 70 {
                        num6 = rng.range(32, 35);
                        num5 = 186;
                    }
                    if matches!(below.kind, 396 | 397 | 404) {
                        num6 = rng.range(29, 35);
                        num5 = 187;
                    }
                    if below.kind == 368 {
                        num6 = rng.range(35, 41);
                        num5 = 187;
                    }
                    if below.kind == 367 {
                        num6 = rng.range(41, 47);
                        num5 = 187;
                    }
                    if num5 == 186 && (7..=15).contains(&num6) && rng.range(0, 75) == 0 {
                        num5 = 187;
                        num6 = 17;
                    }
                    if pile_wall_dungeon(wall) && rng.range(0, 3) != 0 {
                        flag = true;
                    } else {
                        // `:18903`: the `anyShimmer` veto.
                        if !shimmer {
                            place_pile(world, num4, j, num5, num6);
                        }
                        if world.tile(num4, j).is_some_and(|t| t.kind == 186 || t.kind == 187) {
                            flag = true;
                        }
                        if flag && num5 == 186 && num6 <= 7 {
                            pile_cluster(world, num4, j, rng);
                        }
                    }
                }
            }
        }
    }
    // --- scatter 2: the large-pile deep (`:18928-18993`) ----------------
    let num2 = (world.width as f32 * 0.01) as i32;
    for _ in 0..num2 {
        let mut num9 = attempts;
        let mut flag2 = false;
        while !flag2 && num9 > 0 {
            num9 -= 1;
            let mut num10 = rng.range(25, world.width - 25);
            let mut n = rng.range(world.height - 300, world.height - 10);
            if !world.tile(num10, n).is_some_and(|t| t.active()) {
                let mut num11: u16 = 186;
                while !world.tile(num10, n + 1).is_some_and(|t| t.active()) && n < world.height - 5 {
                    n += 1;
                }
                let wall = world.tile(num10, n).map(|t| t.wall).unwrap_or(0);
                let Some(below) = world.tile(num10, n + 1).copied() else {
                    continue;
                };
                if below.active() {
                    let mut num12 = rng.range(0, 22);
                    if (16..=22).contains(&num12) {
                        num12 = rng.range(0, 22);
                    }
                    if n > world.height - 300 || pile_wall_dungeon(wall) || below.kind == 30 || below.kind == 19 {
                        num12 = rng.range(0, 7);
                    }
                    if matches!(below.kind, 57 | 58) && rng.range(0, 3) < 2 {
                        num11 = 187;
                        num12 = rng.range(6, 9);
                    }
                    if matches!(below.kind, 147 | 161 | 162) {
                        num12 = rng.range(26, 32);
                    }
                    place_pile(world, num10, n, num11, num12);
                    if world.tile(num10, n).is_some_and(|t| t.kind == 186 || t.kind == 187) {
                        flag2 = true;
                    }
                    if flag2 && num11 == 186 && num12 <= 7 {
                        pile_cluster(world, num10, n, rng);
                    }
                }
            }
        }
    }
    // --- scatter 3: the large-pile surface (`:18994-19054`) -------------
    let num2 = (world.width as f32 * 0.03) as i32;
    for _ in 0..num2 {
        let mut num18 = attempts;
        let mut flag3 = false;
        while !flag3 && num18 > 0 {
            num18 -= 1;
            let mut num19: u16 = 186;
            let mut num20 = rng.range(25, world.width - 25);
            let mut num21 = rng.range(10, world_surface as i32);
            while super::traps::ocean_depths_trap(world, num20, num21, ocean_level) {
                num20 = rng.range(25, world.width - 25);
                num21 = rng.range(10, world_surface as i32);
            }
            if !world.tile(num20, num21).is_some_and(|t| t.active()) {
                while !world.tile(num20, num21 + 1).is_some_and(|t| t.active()) && num21 < world.height - 5 {
                    num21 += 1;
                }
                let wall = world.tile(num20, num21).map(|t| t.wall).unwrap_or(0);
                let Some(below) = world.tile(num20, num21 + 1).copied() else {
                    continue;
                };
                if below.active() {
                    let mut num22 = rng.range(7, 13);
                    if num21 > world.height - 300
                        || pile_wall_dungeon(wall)
                        || matches!(below.kind, 30 | 19 | 25 | 204 | 234 | 112)
                        || pile_tile_dungeon(below.kind)
                    {
                        num22 = -1;
                    }
                    if matches!(below.kind, 147 | 161 | 162) {
                        num22 = rng.range(26, 32);
                    }
                    if below.kind == 53 {
                        num19 = 187;
                        num22 = rng.range(52, 55);
                    }
                    let grass_side = |world: &World, px: i32| {
                        world.tile(px, num21 + 1).is_some_and(|t| t.active() && t.kind == 2)
                    };
                    if below.kind == 2 || grass_side(world, num20 - 1) || grass_side(world, num20 + 1) {
                        num19 = 187;
                        num22 = rng.range(14, 17);
                    }
                    if matches!(below.kind, 151 | 274) {
                        num19 = 186;
                        num22 = rng.range(0, 7);
                    }
                    if num22 >= 0 {
                        place_pile(world, num20, num21, num19, num22);
                    }
                    if world.tile(num20, num21).is_some_and(|t| t.kind == num19) {
                        flag3 = true;
                    }
                }
            }
        }
    }
    // --- scatter 4: the large-pile walled surface (`:19055-19125`) ------
    let num2 = (world.width as f32 * 0.0035) as i32;
    for _ in 0..num2 {
        let mut num24 = attempts;
        let mut flag4 = false;
        while !flag4 && num24 > 0 {
            num24 -= 1;
            let mut num25 = rng.range(25, world.width - 25);
            let mut num26 = rng.range(10, world_surface as i32);
            if world.tile(num25, num26).is_some_and(|t| !t.active() && t.wall > 0) {
                let mut num27: u16 = 186;
                while !world.tile(num25, num26 + 1).is_some_and(|t| t.active()) && num26 < world.height - 5 {
                    num26 += 1;
                }
                let wall = world.tile(num25, num26).map(|t| t.wall).unwrap_or(0);
                let Some(below) = world.tile(num25, num26 + 1).copied() else {
                    continue;
                };
                if below.active() {
                    let mut num28 = rng.range(7, 13);
                    if num26 > world.height - 300
                        || pile_wall_dungeon(wall)
                        || below.kind == 30
                        || below.kind == 19
                        || pile_tile_dungeon(below.kind)
                    {
                        num28 = -1;
                    }
                    if below.kind == 25 {
                        num28 = rng.range(0, 7);
                    }
                    if matches!(below.kind, 147 | 161 | 162) {
                        num28 = rng.range(26, 32);
                    }
                    let grass_side = |world: &World, px: i32| {
                        world.tile(px, num26 + 1).is_some_and(|t| t.active() && t.kind == 2)
                    };
                    if below.kind == 2 || grass_side(world, num25 - 1) || grass_side(world, num25 + 1) {
                        num27 = 187;
                        num28 = rng.range(14, 17);
                    }
                    if matches!(below.kind, 151 | 274) {
                        num27 = 186;
                        num28 = rng.range(0, 7);
                    }
                    if num28 >= 0 {
                        place_pile(world, num25, num26, num27, num28);
                    }
                    if world.tile(num25, num26).is_some_and(|t| t.kind == num27) {
                        flag4 = true;
                    }
                    if flag4 && num28 <= 7 {
                        pile_cluster(world, num25, num26, rng);
                    }
                }
            }
        }
    }
    // --- scatter 5: the small-pile underground (`:19126-19287`) ---------
    let num2 = (world.width as f32 * 0.6) as i32;
    for _ in 0..num2 {
        let mut num34 = attempts;
        let mut flag5 = false;
        while !flag5 && num34 > 0 {
            num34 -= 1;
            let mut num35 = rng.range(25, world.width - 25);
            let mut num36 = rng.range(world_surface as i32, world.height - 20);
            if world.tile(num35, num36).is_some_and(|t| t.wall == 87) && rng.range(0, 2) == 0 {
                num35 = rng.range(25, world.width - 25);
                num36 = rng.range(world_surface as i32, world.height - 20);
            }
            while super::traps::ocean_depths_trap(world, num35, num36, ocean_level) {
                num35 = rng.range(25, world.width - 25);
                num36 = rng.range(world_surface as i32, world.height - 20);
            }
            if !world.tile(num35, num36).is_some_and(|t| t.active()) {
                while !world.tile(num35, num36 + 1).is_some_and(|t| t.active()) && num36 < world.height - 5 {
                    num36 += 1;
                }
                let (wall, liquid, shimmer) = world
                    .tile(num35, num36)
                    .map(|t| {
                        (
                            t.wall,
                            t.liquid,
                            t.liquid > 0 && t.liquid_kind() == crate::tile::LiquidKind::Shimmer,
                        )
                    })
                    .unwrap_or((0, 0, false));
                let Some(below) = world.tile(num35, num36 + 1).copied() else {
                    continue;
                };
                if below.active() {
                    let num37 = rng.range(0, 2);
                    let mut num38 = rng.range(0, 36);
                    if (28..=35).contains(&num38) {
                        num38 = rng.range(0, 36);
                    }
                    if num37 == 1 {
                        num38 = rng.range(0, 25);
                        if (16..=24).contains(&num38) {
                            num38 = rng.range(0, 25);
                        }
                    }
                    if num36 > world.height - 300 {
                        if num37 == 0 {
                            num38 = rng.range(12, 28);
                        }
                        if num37 == 1 {
                            num38 = rng.range(6, 16);
                        }
                    }
                    if pile_wall_dungeon(wall)
                        || wall == 87
                        || matches!(below.kind, 30 | 19 | 25 | 203)
                    {
                        if num37 == 0 && num38 < 12 {
                            num38 += 12;
                        }
                        if num37 == 1 && num38 < 6 {
                            num38 += 6;
                        }
                        if num37 == 1 && num38 >= 17 {
                            num38 -= 10;
                        }
                    }
                    if matches!(below.kind, 147 | 161 | 162) {
                        if num37 == 0 && num38 < 12 {
                            num38 += 36;
                        }
                        if num37 == 1 && num38 >= 20 {
                            num38 += 6;
                        }
                        if num37 == 1 && num38 < 6 {
                            num38 += 25;
                        }
                    }
                    if liquid == 0 && matches!(below.kind, 53 | 397 | 396) {
                        if num37 == 0 {
                            num38 = rng.range(73, 78);
                        }
                        if num37 == 1 {
                            num38 = rng.range(62, 65);
                        }
                    }
                    if matches!(below.kind, 151 | 274) {
                        if num37 == 0 {
                            num38 = rng.range(12, 28);
                        }
                        if num37 == 1 {
                            num38 = rng.range(12, 19);
                        }
                    }
                    if below.kind == 368 {
                        if num37 == 0 {
                            num38 = rng.range(60, 66);
                        }
                        if num37 == 1 {
                            num38 = rng.range(47, 53);
                        }
                    }
                    if below.kind == 367 {
                        if num37 == 0 {
                            num38 = rng.range(66, 72);
                        }
                        if num37 == 1 {
                            num38 = rng.range(53, 59);
                        }
                    }
                    if pile_tile_dungeon(below.kind) {
                        flag5 = false;
                    } else if pile_wall_dungeon(wall) && rng.range(0, 3) != 0 {
                        flag5 = true;
                    } else if !shimmer {
                        // `:19269`: the `anyShimmer` veto.
                        flag5 = place_small_pile_window(world, num35, num36, num38, num37);
                    }
                    if flag5 && num37 == 1 && (6..=15).contains(&num38) {
                        pile_cluster(world, num35, num36, rng);
                    }
                }
            }
        }
    }
    // --- scatter 6: the small-pile surface (`:19288-19366`) -------------
    let num2 = (world.width as f32 * 0.02) as i32;
    for _ in 0..num2 {
        let mut num44 = attempts;
        let mut flag6 = false;
        while !flag6 && num44 > 0 {
            num44 -= 1;
            let mut num45 = rng.range(25, world.width - 25);
            let mut num46 = rng.range(15, world_surface as i32);
            while super::traps::ocean_depths_trap(world, num45, num46, ocean_level) {
                num45 = rng.range(25, world.width - 25);
                num46 = rng.range(15, world_surface as i32);
            }
            if !world.tile(num45, num46).is_some_and(|t| t.active()) {
                while !world.tile(num45, num46 + 1).is_some_and(|t| t.active()) && num46 < world.height - 5 {
                    num46 += 1;
                }
                let (wall, liquid) = world
                    .tile(num45, num46)
                    .map(|t| (t.wall, t.liquid))
                    .unwrap_or((0, 0));
                let Some(below) = world.tile(num45, num46 + 1).copied() else {
                    continue;
                };
                if below.active() {
                    let num47 = rng.range(0, 2);
                    let mut num48 = rng.range(0, 11);
                    if num47 == 1 {
                        num48 = rng.range(0, 5);
                    }
                    if matches!(below.kind, 147 | 161 | 162) {
                        if num47 == 0 && num48 < 12 {
                            num48 += 36;
                        }
                        if num47 == 1 && num48 >= 20 {
                            num48 += 6;
                        }
                        if num47 == 1 && num48 < 6 {
                            num48 += 25;
                        }
                    }
                    if liquid == 0 && matches!(below.kind, 53 | 397 | 396) {
                        if num47 == 0 {
                            num48 = rng.range(73, 77);
                        }
                        if num47 == 1 {
                            num48 = rng.range(62, 65);
                        }
                    }
                    if below.kind == 2 && num47 == 1 {
                        num48 = rng.range(38, 41);
                    }
                    if matches!(below.kind, 151 | 274) {
                        if num47 == 0 {
                            num48 = rng.range(12, 28);
                        }
                        if num47 == 1 {
                            num48 = rng.range(12, 19);
                        }
                    }
                    if !pile_wall_dungeon(wall)
                        && !matches!(below.kind, 30 | 19 | 41 | 43 | 44 | 481 | 482 | 483 | 45 | 46 | 47 | 175 | 176 | 177 | 25 | 203)
                        && !pile_tile_dungeon(below.kind)
                    {
                        flag6 = place_small_pile_window(world, num45, num46, num48, num47);
                    }
                }
            }
        }
    }
    // --- scatter 7: the small-pile dirt/snow-wall surface (`:19367-19440`)
    let num2 = (world.width as f32 * 0.15) as i32;
    for _ in 0..num2 {
        let mut num50 = attempts;
        let mut flag7 = false;
        while !flag7 && num50 > 0 {
            num50 -= 1;
            let mut num51 = rng.range(25, world.width - 25);
            let mut num52 = rng.range(15, world_surface as i32);
            if world.tile(num51, num52).is_some_and(|t| !t.active() && (t.wall == 2 || t.wall == 40)) {
                while !world.tile(num51, num52 + 1).is_some_and(|t| t.active()) && num52 < world.height - 5 {
                    num52 += 1;
                }
                let (wall, liquid) = world
                    .tile(num51, num52)
                    .map(|t| (t.wall, t.liquid))
                    .unwrap_or((0, 0));
                let Some(below) = world.tile(num51, num52 + 1).copied() else {
                    continue;
                };
                if below.active() {
                    let num53 = rng.range(0, 2);
                    let mut num54 = rng.range(0, 11);
                    if num53 == 1 {
                        num54 = rng.range(0, 5);
                    }
                    if matches!(below.kind, 147 | 161 | 162) {
                        if num53 == 0 && num54 < 12 {
                            num54 += 36;
                        }
                        if num53 == 1 && num54 >= 20 {
                            num54 += 6;
                        }
                        if num53 == 1 && num54 < 6 {
                            num54 += 25;
                        }
                    }
                    if liquid == 0 && matches!(below.kind, 53 | 397 | 396) {
                        if num53 == 0 {
                            num54 = rng.range(73, 77);
                        }
                        if num53 == 1 {
                            num54 = rng.range(62, 65);
                        }
                    }
                    if below.kind == 2 && num53 == 1 {
                        num54 = rng.range(38, 41);
                    }
                    if matches!(below.kind, 151 | 274) {
                        if num53 == 0 {
                            num54 = rng.range(12, 28);
                        }
                        if num53 == 1 {
                            num54 = rng.range(12, 19);
                        }
                    }
                    if (liquid != u8::MAX || below.kind != 53 || wall != 0)
                        && !pile_wall_dungeon(wall)
                        && !matches!(below.kind, 30 | 19 | 41 | 43 | 44 | 481 | 482 | 483 | 45 | 46 | 47 | 175 | 176 | 177 | 25 | 203)
                        && !pile_tile_dungeon(below.kind)
                    {
                        flag7 = place_small_pile_window(world, num51, num52, num54, num53);
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// W4-d104 pass 30: the forge lands on a hell-house wall column's
    /// floor, and two same-stream runs replay byte-equal (a == b).
    #[test]
    fn w4_d104_30_the_hellforge_places_and_replays() {
        let build = || {
            let mut w = World::new("hf".into(), 400, 600);
            // A hell-house block: wall 14 over rows 400..500, the floor
            // at 500 (the drop scan stops there).
            for x in 100..140 {
                for y in 400..500 {
                    let i = w.index(x, y);
                    w.tiles[i].wall = 14;
                }
                for y in 500..600 {
                    let i = w.index(x, y);
                    w.tiles[i].kind = TILE_HELL_BRICK;
                    w.tiles[i].flags |= FLAG_ACTIVE;
                    w.tiles[i].wall = 14;
                }
            }
            w
        };
        let mut a = build();
        let mut b = build();
        let pa = pass_hellforge(&mut a, &mut crate::worldgen::noise::Rng(42));
        let pb = pass_hellforge(&mut b, &mut crate::worldgen::noise::Rng(42));
        assert_eq!(a.tiles, b.tiles, "a == b: the pass stream replays");
        assert_eq!(pa, pb);
        assert!(pa > 0, "the forge placed ({pa})");
        assert!(
            a.tiles.iter().any(|t| t.active() && t.kind == TILE_HELLFORGE),
            "the 77 tiles exist"
        );
    }

    /// W4-d104 pass 35: the pile scatters land decor on the cave floors
    /// and two same-stream runs replay byte-equal (a == b).
    #[test]
    fn w4_d104_35_the_piles_place_and_replay() {
        let build = || {
            let mut w = World::new("pi".into(), 900, 600);
            // A stone floor at row 200 with air above (the drop scans
            // seat the piles on it).
            for x in 0..w.width {
                for y in 200..w.height {
                    let i = w.index(x, y);
                    w.tiles[i].kind = 1;
                    w.tiles[i].flags |= FLAG_ACTIVE;
                }
            }
            w.world_surface = 100.0;
            w
        };
        let mut a = build();
        let mut b = build();
        pass_piles(&mut a, 100.0, 300.0, &mut crate::worldgen::noise::Rng(42));
        pass_piles(&mut b, 100.0, 300.0, &mut crate::worldgen::noise::Rng(42));
        assert_eq!(a.tiles, b.tiles, "a == b: the pass stream replays");
        assert!(
            a.tiles.iter().any(|t| t.active() && matches!(t.kind, 185 | 186 | 187)),
            "a pile landed"
        );
    }
}
