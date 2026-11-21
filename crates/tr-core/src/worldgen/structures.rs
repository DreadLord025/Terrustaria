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
//! Deviations (ANALYSIS "Structures (M1.4-WGEN-2)" + "Jungle residuals
//! (W4-c31)"): the chest inventories ride the storage task (the chests place
//! as tiles only — the C# `Chest.CreateChest` book is the M7-TE follow-up);
//! the jungle shrines are now FAITHFUL (W4-c31 — the `GenVars.jungleHut`
//! material, the proximity veto, the torch/floor/pillars/roof sweeps); the
//! altar layouts stay shape-level; the pyramid corridors are one floor (the
//! C#'s multi-chamber crawl is the follow-up); the boulder piles and the
//! graveyards are out of scope.

use super::{TILE_DIRT, TILE_STONE, WALL_DIRT};
use super::biomes::{TILE_JUNGLE_GRASS, TILE_MUD};
use crate::tile::{FLAG_ACTIVE, Tile};
use crate::world::World;

pub(crate) const TILE_CHEST: u16 = 21;
pub(crate) const TILE_ALTAR: u16 = 26;
pub(crate) const TILE_POT: u16 = 28;
pub(crate) const TILE_PYRAMID_BRICK: u16 = 480;

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

/// `JungleShrines` (`WorldGen.cs:15977-16131`), the faithful port (W4-c31
/// jungle residuals — replaces the shape-level hut): the `GenVars.jungleHut`
/// material shell, the dungeon-opposite-half origin search with the
/// `num4 > maxTilesX*10` retry valve, the 30-radius hive/temple proximity
/// veto, `structures.CanPlace(area, 1)`, the carved interior with the
/// hut-specific wall, the 100-try torch, the floor carve, the mud pillars,
/// the shrinking roof rows and the `JChestX/Y` publication. Returns the
/// chest spots (the `GenVars.JChestX/Y` slice). Draws on its own fresh
/// `pass_rng("Jungle Chests")` slot (the shape-level pass rode the shared
/// `srng`; the isolation is the recorded stream change).
///
/// Deviations: the port has no global `StructureMap`, so the cross-pass
/// protected-rect intersection is carried by a pass-local `protected` vec
/// (the shrine rects accumulate via `AddProtectedStructure`); the hive/
/// temple avoidance is the C#'s own 30-radius tile/wall proximity veto.
/// The `tileSolid[137] = false` window (`:16129`) has no mutable port
/// table — the dart-trap solidity is read by the temple/trap passes at
/// their own slots (the hive.rs window precedent). The sub-vanilla valve:
/// when the y band `[(worldSurface+rockLayer)/2, maxTilesY-400)` collapses
/// (a 512-tall world), the C# `Next` would throw — the port skips the pass.
pub(crate) fn pass_jungle_shrines(
    world: &mut World,
    dungeon_x: i32,
    world_surface: f64,
    rock_layer: f64,
    jungle_hut: u16,
    rng: &mut super::Rng,
) -> Vec<(i32, i32)> {
    let mut chests = Vec::new();
    let width = world.width;
    let height = world.height;
    // The y band (`:15982`/`:15996`): `Next((int)(worldSurface + rockLayer)
    // / 2, maxTilesY - 400)`. The sub-vanilla valve — an empty band cannot
    // exist in-world (the C# throws; the port skips before any draw).
    let y_lo = ((world_surface + rock_layer) / 2.0) as i32;
    let y_hi = height - 400;
    if y_hi <= y_lo {
        return chests;
    }
    // The hut material → interior wall map (`:16026-16045`).
    let wall: u16 = match jungle_hut {
        119 => 23,
        120 => 24,
        158 => 42,
        175 => 45,
        45 => 10,
        _ => 0,
    };
    // The two initial draws (`:15981-15982`) — overwritten in the loop but
    // they consume the stream, so they are drawn to keep the schedule.
    let _ = rng.range(40, width - 40);
    let _ = rng.range(y_lo, y_hi);
    // The hut count (`:15983`): `Next(7, 12) * maxTilesX / 4200.0`.
    let count = rng.range(7, 12) as f64 * (width as f64 / 4200.0);
    // The port's dungeon is fixed Right (`plan_spans`' `dungeon_x`, Decision
    // M5-BIOME #10), so `dungeonSide <= Left` is false and the `num +=
    // maxTilesX/2` offset never fires — the shrines sit in the left half
    // (the jungle's dungeon-opposite side). The branch is carried for the
    // shape; `dungeon_x > width/2` selects it.
    let dungeon_left = dungeon_x <= width / 2;
    let mut protected: Vec<(i32, i32, i32, i32)> = Vec::new();
    let mut num4 = 0i32;
    let mut i = 0f64;
    while i < count {
        let mut flag = true;
        while flag {
            num4 += 1;
            let mut num = rng.range(40, width / 2 - 40);
            if dungeon_left {
                num += width / 2;
            }
            let num2 = rng.range(y_lo, y_hi);
            let num5 = rng.range(2, 4);
            let num6 = rng.range(2, 4);
            // `Rectangle area = (num - num5 - 1, num2 - num6 - 1, num5 + 1,
            // num6 + 1)` (`:15999`) — the C#'s own (small) CanPlace rect.
            let area = (num - num5 - 1, num2 - num6 - 1, num5 + 1, num6 + 1);
            if world
                .tile(num, num2)
                .is_some_and(|t| t.active() && t.kind == TILE_JUNGLE_GRASS)
            {
                flag = false;
                // The 30-radius proximity veto (`:16004-16017`): stride-3
                // scan for hive/temple/shrine tiles (225/229/226/119/120)
                // and the hive/temple walls (86/87).
                let num7 = 30;
                let mut j = num - num7;
                while j < num + num7 {
                    let mut k = num2 - num7;
                    while k < num2 + num7 {
                        if let Some(t) = world.tile(j, k) {
                            if t.active()
                                && matches!(t.kind, 225 | 229 | 226 | 119 | 120)
                            {
                                flag = true;
                            }
                            if t.wall == 86 || t.wall == 87 {
                                flag = true;
                            }
                        }
                        k += 3;
                    }
                    j += 3;
                }
                if !can_place_padded(world, area, 1, &protected) {
                    flag = true;
                }
            }
            if !flag {
                // The brick shell (`:16046-16055`): active, jungleHut,
                // liquid 0, lava off.
                for l in (num - num5 - 1)..=(num + num5 + 1) {
                    for m in (num2 - num6 - 1)..=(num2 + num6 + 1) {
                        if let Some(t) = world.tile_mut(l, m) {
                            t.flags |= FLAG_ACTIVE;
                            t.kind = jungle_hut;
                            t.liquid = 0;
                            t.flags &= !crate::tile::FLAG_LIQUID_LAVA;
                        }
                    }
                }
                // The carved interior (`:16056-16063`): inactive, hut wall.
                for n in (num - num5)..=(num + num5) {
                    for m in (num2 - num6)..=(num2 + num6) {
                        if let Some(t) = world.tile_mut(n, m) {
                            t.flags &= !FLAG_ACTIVE;
                            t.wall = wall;
                        }
                    }
                }
                // The torch (`:16064-16076`): 100 tries, `PlaceTile(4,
                // style 3)`; the first active-4 cell ends the loop.
                let mut placed_torch = false;
                let mut tries = 0;
                while !placed_torch && tries < 100 {
                    tries += 1;
                    let tx = rng.range(num - num5, num + num5 + 1);
                    let ty = rng.range(num2 - num6, num2 + num6 - 2);
                    if crate::edit::place_torch(world, tx, ty, 3) {
                        placed_torch = true;
                    }
                }
                // The floor carve (`:16077-16090`): the bottom three rows
                // inactive (the C# runs two overlapping sweeps; the second
                // is a subset — one sweep over the union is identical).
                for l in (num - num5 - 1)..=(num + num5 + 1) {
                    for m in (num2 + num6 - 2)..=(num2 + num6) {
                        if let Some(t) = world.tile_mut(l, m) {
                            t.flags &= !FLAG_ACTIVE;
                        }
                    }
                }
                // The mud pillars (`:16091-16102`): up to 4 mud cells below
                // the floor per column, stopping at the first active cell.
                for l in (num - num5 - 1)..=(num + num5 + 1) {
                    let mut m = num2 + num6 + 2;
                    let mut fill = 4;
                    while fill > 0 && m < height {
                        if world.tile(l, m).is_some_and(|t| t.active()) {
                            break;
                        }
                        if let Some(t) = world.tile_mut(l, m) {
                            t.flags |= FLAG_ACTIVE;
                            t.kind = TILE_MUD;
                        }
                        m += 1;
                        fill -= 1;
                    }
                }
                // The shrinking roof rows (`:16103-16114`).
                let mut half = num5 - rng.range(1, 3);
                let mut row = num2 - num6 - 2;
                while half > -1 {
                    for l in (num - half - 1)..=(num + half + 1) {
                        if let Some(t) = world.tile_mut(l, row) {
                            t.flags |= FLAG_ACTIVE;
                            t.kind = jungle_hut;
                        }
                    }
                    half -= rng.range(1, 3);
                    row -= 1;
                }
                // `JChestX/Y[numJChests] = (num, num2)` + the protected
                // rect (`:16115-16118`); `num4 = 0` resets the retry valve.
                chests.push((num, num2));
                protected.push(area);
                num4 = 0;
            } else if num4 > width * 10 {
                // The retry valve (`:16121-16126`): `i++`, reset, break.
                i += 1.0;
                num4 = 0;
                break;
            }
        }
        i += 1.0;
    }
    chests
}

/// `StructureMap.CanPlace(area, padding)` (`StructureMap.cs:24-52`): the
/// bounds gate on the ORIGINAL area, then the protected-rect intersection
/// and the active-tile validity scan over the area inflated by `padding`
/// (`TileID.Sets.GeneralPlacementTiles`). The port's `protected` vec is the
/// structure-map stand-in (the pass-local shrine rects).
fn can_place_padded(
    world: &World,
    area: (i32, i32, i32, i32),
    padding: i32,
    protected: &[(i32, i32, i32, i32)],
) -> bool {
    let (ax, ay, aw, ah) = area;
    if ax < 0 || ay < 0 || ax + aw > world.width - 1 || ay + ah > world.height - 1 {
        return false;
    }
    let (rx, ry, rw, rh) = (ax - padding, ay - padding, aw + padding * 2, ah + padding * 2);
    for &(px, py, pw, ph) in protected {
        if rx < px + pw && rx + rw > px && ry < py + ph && ry + rh > py {
            return false;
        }
    }
    for x in rx..rx + rw {
        for y in ry..ry + rh {
            if world
                .tile(x, y)
                .is_some_and(|t| t.active() && !crate::tiledata::SET_GENERAL_PLACEMENT_TILES.has(t.kind))
            {
                return false;
            }
        }
    }
    true
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

/// RETIRED (W4-c31): the shape-level hive dome — replaced by the faithful
/// `HiveBiome` port (`hive::pass_beehives`, `HiveBiome.cs:11-383` + the
/// `Beehives`/`BeeLarvaInBeehives` passes). Kept for its test history
/// (`mini_biomes::the_hive_places_its_five_patches_on_the_ring` exercises
/// the retired shape); the pipeline no longer calls it.
#[allow(dead_code)]
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

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::noise::Rng;

    /// A mud world with a jungle-grass (60) band in the shrine y-range —
    /// the origin gate (`tile.type == 60`) passes inside it.
    fn jungle_world(width: i32, height: i32) -> World {
        let mut w = World::new("shrines".into(), width, height);
        for x in 0..width {
            for y in 0..height {
                let i = w.index(x, y);
                w.tiles[i] = Tile { kind: 59, wall: 15, ..Default::default() };
                w.tiles[i].flags |= FLAG_ACTIVE;
            }
        }
        for x in 40..width / 2 - 40 {
            for y in 300..500 {
                let i = w.index(x, y);
                w.tiles[i].kind = 60;
            }
        }
        w
    }

    /// The faithful pass places hut shells of the picked material with the
    /// matching interior wall, and returns one chest spot per hut.
    #[test]
    fn the_shrine_pass_places_the_hut_material_and_wall() {
        let mut w = jungle_world(1200, 1000);
        let spots = pass_jungle_shrines(&mut w, 1000, 250.0, 450.0, 119, &mut Rng(0x5EED));
        assert!(!spots.is_empty(), "the pass places huts on the grass band");
        for &(x, y) in &spots {
            let shell = w.tile(x - 3, y - 3).unwrap();
            assert!(shell.active() && shell.kind == 119, "the shell at ({x},{y})");
            let inner = w.tile(x, y).unwrap();
            assert!(!inner.active() && inner.wall == 23, "the interior wall");
        }
    }

    /// The a == b determinism witness: two runs over identical worlds draw
    /// identical streams and write identical tiles (the full Tile).
    #[test]
    fn the_shrine_pass_is_reproducible_for_a_seed() {
        let run = || {
            let mut w = jungle_world(1200, 1000);
            let spots = pass_jungle_shrines(&mut w, 1000, 250.0, 450.0, 120, &mut Rng(42));
            (w, spots)
        };
        let (a, sa) = run();
        let (b, sb) = run();
        assert_eq!(sa, sb, "the chest spots");
        assert_eq!(a.tiles, b.tiles, "a == b — the full tile state");
    }

    /// The sub-vanilla valve: a collapsed y band skips the pass before any
    /// draw (the C# `Next` would throw on the empty range).
    #[test]
    fn the_shrine_pass_skips_a_collapsed_band() {
        let mut w = jungle_world(1200, 512);
        let spots = pass_jungle_shrines(&mut w, 1000, 250.0, 450.0, 119, &mut Rng(7));
        assert!(spots.is_empty(), "the collapsed band places nothing");
    }

    /// The proximity veto: a hive tile (225) inside the 30-radius scan
    /// refuses the origin.
    #[test]
    fn the_shrine_pass_refuses_near_a_hive() {
        let mut w = jungle_world(1200, 1000);
        for x in 40..600 {
            for y in 300..500 {
                let i = w.index(x, y);
                w.tiles[i].kind = 225;
            }
        }
        let spots = pass_jungle_shrines(&mut w, 1000, 250.0, 450.0, 119, &mut Rng(3));
        assert!(spots.is_empty(), "the hive proximity veto refuses every origin");
    }
}
