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
//! the evil flavour), the pot 28, the sandstone brick 151 (the pyramid
//! hull — `TileID.SandstoneBrick`, `TileID.cs:931`). Walls (`WallID.cs`):
//! the shrine interiors 24/42/45/10, the sandstone 187, the pyramid's
//! sandstone-brick wall 34.
//!
//! Deviations (ANALYSIS "Structures (M1.4-WGEN-2)" + "Jungle residuals
//! (W4-c31)"): the chest inventories ride the storage task (the chests place
//! as tiles only — the C# `Chest.CreateChest` book is the M7-TE follow-up);
//! the jungle shrines are now FAITHFUL (W4-c31 — the `GenVars.jungleHut`
//! material, the proximity veto, the torch/floor/pillars/roof sweeps); the
//! altar layouts stay shape-level; the boulder piles and the graveyards are
//! out of scope. The pyramid is the faithful `Pyramid` port (W4-c33 — the
//! 151 hull, the wall-34 sweep, the entrance shaft, the zig-zag tunnel with
//! the chamber and the post-chamber surface tunnel); its secret-seed arms
//! (tenthAnniversary/dualDungeons/surfaceIsDesert/errorWorld) stay with
//! the seed lanes (d107).

use super::{GenOptions, TILE_DIRT, TILE_STONE, WALL_DIRT};
use super::biomes::{TILE_JUNGLE_GRASS, TILE_MUD};
use super::terrain::Layers;
use crate::seeds::{MicroSeedFlags, SeedFlags};
use crate::tile::{
    FLAG_ACTIVE, FLAG_ACTUATOR, FLAG_HALF_BRICK, FLAG_INACTIVE, FLAG_LIQUID_LAVA, LiquidKind, Tile,
};
use crate::tiledata as td;
use crate::world::World;

pub(crate) const TILE_CHEST: u16 = 21;
pub(crate) const TILE_ALTAR: u16 = 26;
pub(crate) const TILE_POT: u16 = 28;
/// `TileID.SandstoneBrick` (`TileID.cs:931`) — the faithful `Pyramid` hull
/// (`WorldGen.cs:27671`: `ushort num = 151`). The shape-level port's 480
/// (BloodMoonMonolith) retired with W4-c33.
pub(crate) const TILE_PYRAMID_BRICK: u16 = 151;
/// `WallID.SandstoneBrick` = 34 — the pyramid's interior wall
/// (`WorldGen.cs:27672`).
pub(crate) const WALL_PYRAMID_BRICK: u16 = 34;
pub(crate) const TILE_SMALL_PILES: u16 = 185;
pub(crate) const TILE_BANNER: u16 = 91;

/// The chest 2×2: the anchor tile `(x, y)` with the frame corners
/// (`TileObject`'s chest frames — the port's `frame_x = 18 * style`
/// convention, `tileobject.rs:2072-2082`). `style` is the C#'s resolved
/// `num9` (`:35978-35988`): the caller's `chestStyle` when `>= 0`, else the
/// `flag15` default (1 — the gold chest — below `worldSurface + 25`).
pub(crate) fn place_chest(world: &mut World, x: i32, y: i32, style: i32) -> bool {
    place_chest_kind(world, x, y, TILE_CHEST, style)
}

/// `WorldGen.PlaceChest(x, y, type, notNearOtherChests, style)`
/// (`WorldGen.cs:57813-57846`), the kind-aware twin: the mahogany chest 467
/// shares the 21 frame grid (`TileObjectData`'s chest defs — 36 px per
/// style, 2×2), so only the written kind differs. The `AddBuriedChest`
/// core's remix sky arm (`:36190-36194`) and noTraps swap (`:36201-36205`)
/// place 467 through it (W4-g44, the d103 `buried_chest` graft).
pub(crate) fn place_chest_kind(world: &mut World, x: i32, y: i32, kind: u16, style: i32) -> bool {
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
        t.kind = kind;
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

/// `Pyramids` (`WorldGen.cs:15360-15460`): the `numPyr` loop over the
/// Dunes (`:11564-11579`) / Ocean Sand (`:11659-11664`) published anchors —
/// the >300/<width-300 gate, the dungeon-side 15 % exclusion (the port's
/// dungeon side is fixed Right, so only the second clause bites:
/// `num <= dungeonX - width·0.15`), the sand-column descent, the
/// nearest-slot 220 spacing (the drunk seed halves it — the port carries
/// that flag) and the `j--` before the `Pyramid` call. The
/// tenthAnniversary/dualDungeons/surfaceIsDesert/errorWorld arms stay with
/// the seed lanes (d107); the Skyblock gate rides `opts`.
#[allow(clippy::too_many_arguments)]
pub(crate) fn pass_pyramids(
    world: &mut World,
    pyr_x: &[i32],
    pyr_y: &[i32],
    num_pyr: i32,
    dungeon_x: i32,
    drunk: bool,
    rng: &mut super::Rng,
    stock: Option<&mut super::chest_stock::ChestStockHook<'_>>,
) -> u32 {
    let mut stock = stock;
    let mut placed = 0u32;
    // `:15369-15420`: the anchor loop.
    for i in 0..num_pyr as usize {
        let num = pyr_x[i];
        let mut j = pyr_y[i];
        let pyramid_min_depth = 75;
        let num2 = 125;
        // `:15380`: the margin + dungeon-side gate. `dungeonSide` is fixed
        // Right in the port (M5-BIOME), so `dungeonSide > Left` is always
        // true and `dungeonSide < Right` always false — the second clause
        // reduces to `!(num > dungeonX - maxTilesX·0.15)`.
        if !(num > 300
            && num < world.width - 300
            && !((num as f64) > (dungeon_x as f64) - (world.width as f64) * 0.15))
        {
            continue;
        }
        // `:15382-15384`: the sand-column descent.
        while !world.tile(num, j).is_some_and(|t| t.active()) && (j as f64) < world.world_surface {
            j += 1;
        }
        // `:15385`: the surface-line + sand gate.
        if (j as f64) >= world.world_surface
            || !world
                .tile(num, j)
                .is_some_and(|t| t.kind == super::TILE_SAND)
        {
            continue;
        }
        // `:15387-15401`: the nearest earlier slot, the 220 spacing.
        let mut num3 = world.width;
        for k in 0..i {
            let num4 = (num - pyr_x[k]).abs();
            if num4 < num3 {
                num3 = num4;
            }
        }
        let mut num5 = 220;
        if drunk {
            num5 /= 2;
        }
        if num3 >= num5 {
            // `:15403`: the one-row lift before the call.
            j -= 1;
            if pyramid(
                world,
                num,
                j,
                pyramid_min_depth,
                num2,
                rng,
                stock.as_deref_mut(),
            ) {
                placed += 1;
            }
        }
    }
    placed
}

/// `WorldGen.Pyramid` (`WorldGen.cs:27669-27971`): the faithful structure —
/// the 151 sandstone-brick hull, the wall-34 3×3 interior sweep, the
/// entrance shaft, the zig-zag tunnel with the single chamber (the Pharaoh
/// double-roll chest style 1, the 185 small piles, the four 91 banners, the
/// 28 pot row) and the post-chamber surface tunnel. `noTunnel` is only set
/// by the dualDungeons seed (d107) — the classic path always digs the
/// surface tunnel. The `SquareWallFrame`/`SquareTileFrame` calls ride the
/// port's framing convention (worldgen writes walls/tiles only — the
/// `desert_entrances.rs` precedent). Out-of-world cells skip (the C# would
/// throw; the pass's 300-tile margins keep x inside, the deep surface
/// tunnel can outrun a small world's height).
fn pyramid(
    world: &mut World,
    i: i32,
    j: i32,
    pyramid_min_depth: i32,
    pyramid_max_depth: i32,
    rng: &mut super::Rng,
    stock: Option<&mut super::chest_stock::ChestStockHook<'_>>,
) -> bool {
    let mut stock = stock;
    // `:27673-27676`: the already-a-pyramid guard (the wall-151 disjunct is
    // the C#'s own — verbatim).
    if world
        .tile(i, j)
        .is_some_and(|t| t.active() && (t.kind == TILE_PYRAMID_BRICK || t.wall == 151))
    {
        return false;
    }
    // `:27711-27714`: the apex jitter, the shaft offset, the depth roll.
    let num2 = j - rng.range(0, 7);
    let num3 = rng.range(9, 13);
    let mut num4 = 1;
    let num5 = j + rng.range(pyramid_min_depth, pyramid_max_depth);
    // `:27715-27725`: the stepped hull.
    for k in num2..num5 {
        for l in (i - num4)..(i + num4 - 1) {
            if let Some(t) = world.tile_mut(l, k) {
                t.kind = TILE_PYRAMID_BRICK;
                t.flags |= FLAG_ACTIVE;
                t.flags &= !crate::tile::FLAG_HALF_BRICK;
                t.set_slope(0);
            }
        }
        num4 += 1;
    }
    // `:27726-27747`: the wall-34 sweep — a cell takes the wall only when
    // its whole 3×3 neighbourhood is active hull brick.
    for m in (i - num4 - 5)..=(i + num4 + 5) {
        for n in (j - 1)..=(num5 + 1) {
            let mut full = true;
            for n6 in (m - 1)..=(m + 1) {
                for n7 in (n - 1)..=(n + 1) {
                    if !world
                        .tile(n6, n7)
                        .is_some_and(|t| t.active() && t.kind == TILE_PYRAMID_BRICK)
                    {
                        full = false;
                    }
                }
            }
            if full {
                if let Some(t) = world.tile_mut(m, n) {
                    t.wall = WALL_PYRAMID_BRICK;
                }
            }
        }
    }
    // `:27748-27757`: the entrance shaft's side, offset, height, budget.
    let mut num8 = 1;
    if rng.range(0, 2) == 0 {
        num8 = -1;
    }
    let mut num9 = i - num3 * num8;
    let mut num10 = j + num3;
    let num11 = rng.range(5, 8);
    let mut num12 = rng.range(20, 30);
    // `:27758-27785`: the shaft carve — dig down through the hull while the
    // column still bites brick, sand trailing above the dig.
    let mut digging = true;
    while digging {
        digging = false;
        let mut sand_above = false;
        for num13 in num10..=(num10 + num11) {
            let num14 = num9;
            if world
                .tile(num14, num13 - 1)
                .is_some_and(|t| t.active() && t.kind == super::TILE_SAND)
            {
                sand_above = true;
            }
            if world
                .tile(num14, num13)
                .is_some_and(|t| t.active() && t.kind == TILE_PYRAMID_BRICK)
            {
                if let Some(t) = world.tile_mut(num14, num13 + 1) {
                    t.wall = WALL_PYRAMID_BRICK;
                }
                if let Some(t) = world.tile_mut(num14 + num8, num13) {
                    t.wall = WALL_PYRAMID_BRICK;
                }
                if let Some(t) = world.tile_mut(num14, num13) {
                    t.flags &= !FLAG_ACTIVE;
                }
                digging = true;
            }
            if sand_above {
                if let Some(t) = world.tile_mut(num14, num13) {
                    t.kind = super::TILE_SAND;
                    t.flags |= FLAG_ACTIVE;
                    t.flags &= !crate::tile::FLAG_HALF_BRICK;
                    t.set_slope(0);
                }
            }
        }
        num9 -= num8;
    }
    // `:27786-27908`: the zig-zag tunnel with the chamber.
    num9 = i - num3 * num8;
    let mut first_turn = true;
    let mut chamber_done = false;
    let mut tunnelling = true;
    while tunnelling {
        for num15 in num10..=(num10 + num11) {
            if let Some(t) = world.tile_mut(num9, num15) {
                t.flags &= !FLAG_ACTIVE;
            }
        }
        num9 += num8;
        num10 += 1;
        num12 -= 1;
        if num10 >= num5 - num11 * 2 {
            num12 = 10;
        }
        if num12 <= 0 {
            let mut chamber_this_turn = false;
            if !first_turn && !chamber_done {
                // `:27808-27813`: the noTunnel seed arm ends the walk here
                // (d107); the classic path always builds the chamber.
                chamber_done = true;
                chamber_this_turn = true;
                // `:27814-27843`: the chamber carve — the tapered ends.
                let num16 = rng.range(7, 13);
                let mut num17 = rng.range(23, 28);
                let num18 = num17;
                let num19 = num9;
                while num17 > 0 {
                    for num20 in (num10 - num16 + num11)..=(num10 + num11) {
                        let carve = if num17 == num18 || num17 == 1 {
                            num20 >= num10 - num16 + num11 + 2
                        } else if num17 == num18 - 1
                            || num17 == 2
                            || num17 == num18 - 2
                            || num17 == 3
                        {
                            num20 >= num10 - num16 + num11 + 1
                        } else {
                            true
                        };
                        if carve {
                            if let Some(t) = world.tile_mut(num9, num20) {
                                t.flags &= !FLAG_ACTIVE;
                            }
                        }
                    }
                    num17 -= 1;
                    num9 += num8;
                }
                // `:27844-27851`: the chamber's span.
                let num21 = num9 - num8;
                let (num22, num23) = if num21 > num19 {
                    (num19, num21)
                } else {
                    (num21, num19)
                };
                // `:27852-27872`: the Pharaoh double-roll (the
                // tenthAnniversary 0→1 arm stays with d107).
                let mut roll = rng.range(0, 3);
                if roll == 0 {
                    roll = rng.range(0, 3);
                }
                let main = match roll {
                    0 => 848, // `Pharaoh'sMask`
                    1 => 857, // `SandstorminaBottle`
                    _ => 934, // `FlyingCarpet`
                };
                // `:27873`: `AddBuriedChest(…, notNearOtherChests: false,
                // 1, trySlope: false, 0)` — the shared placement half.
                add_buried_chest(
                    world,
                    (num22 + num23) / 2,
                    num10,
                    main,
                    false,
                    1,
                    false,
                    true,
                    true,
                    stock.as_deref_mut(),
                    rng,
                );
                // `:27874-27878`: the small piles (the x draw first, then
                // the style — the C# argument order).
                let num25 = rng.range(1, 10);
                for _ in 0..num25 {
                    let px = rng.range(num22, num23);
                    let style = rng.range(16, 19);
                    super::spider_gem_moss::place_small_pile(world, px, num10 + num11, style, 1);
                }
                // `:27879-27882`: the four banners — each `Next(4, 7)`
                // style draws at the call site even when the hang fails.
                for (bx, by) in [
                    (num22 + 2, num10 - num16 + num11 + 1),
                    (num22 + 3, num10 - num16 + num11),
                    (num23 - 2, num10 - num16 + num11 + 1),
                    (num23 - 3, num10 - num16 + num11),
                ] {
                    let style = rng.range(4, 7);
                    super::dungeon_features::place_banner(world, bx, by, TILE_BANNER, style);
                }
                // `:27883-27886`: the pot row.
                for num27 in num22..=num23 {
                    let style = rng.range(25, 28);
                    super::spider_gem_moss::place_pot(world, rng, num27, num10 + num11, style);
                }
            }
            // `:27888-27902`: the turn budgets.
            if first_turn {
                first_turn = false;
                num8 *= -1;
                num12 = rng.range(15, 20);
            } else if chamber_this_turn {
                num12 = rng.range(10, 15);
            } else {
                num8 *= -1;
                num12 = rng.range(20, 40);
            }
        }
        if num10 >= num5 - num11 {
            tunnelling = false;
        }
    }
    // `:27913-27969`: the post-chamber surface tunnel — the brick-lined
    // crawl with the dungeon-wall veto, the two per-row jitter draws and
    // the air-break / hard-cap exits.
    let mut num28 = rng.range(100, 200);
    let mut num29 = rng.range(500, 800);
    let mut crawling = true;
    let num30 = num11;
    num12 = rng.range(10, 50);
    if num8 == 1 {
        num9 -= num30;
    }
    let num31 = rng.range(5, 10);
    while crawling {
        num28 -= 1;
        num29 -= 1;
        num12 -= 1;
        // `:27928`: the C# for-condition re-draws `Next(0, 2)` EVERY
        // iteration (the upper jitter is not a constant) — the while form
        // keeps that draw order exactly, including the final failing test.
        let mut num32 = num9 - num31 - rng.range(0, 2);
        while num32 <= num9 + num30 + num31 + rng.range(0, 2) {
            let num33 = num10;
            let dungeon_wall = world
                .tile(num32, num33)
                .is_some_and(|t| crate::scene::is_dungeon_wall(t.wall));
            if num32 >= num9 && num32 <= num9 + num30 {
                if let Some(t) = world.tile_mut(num32, num33) {
                    t.flags &= !FLAG_ACTIVE;
                }
            } else if !dungeon_wall {
                if let Some(t) = world.tile_mut(num32, num33) {
                    t.kind = TILE_PYRAMID_BRICK;
                    t.flags |= FLAG_ACTIVE;
                    t.flags &= !crate::tile::FLAG_HALF_BRICK;
                    t.set_slope(0);
                }
            }
            if num32 >= num9 - 1 && num32 <= num9 + 1 + num30 && !dungeon_wall {
                if let Some(t) = world.tile_mut(num32, num33) {
                    t.wall = WALL_PYRAMID_BRICK;
                }
            }
            num32 += 1;
        }
        num10 += 1;
        num9 += num8;
        // `:27949-27959`: the air-break — the crawl stops once a whole row
        // under the mouth is open.
        if num28 <= 0 {
            crawling = false;
            for num34 in (num9 + 1)..=(num9 + num30 - 1) {
                if world.tile(num34, num10).is_some_and(|t| t.active()) {
                    crawling = true;
                }
            }
        }
        if num12 < 0 {
            num12 = rng.range(10, 50);
            num8 *= -1;
        }
        if num29 <= 0 {
            crawling = false;
        }
    }
    true
}

/// `WorldGen.AddBuriedChest`'s placement half (`WorldGen.cs:35877-36210` +
/// the slope-restore tail `:37554-37565`), generalised from the temple's
/// private port (W4-c33): the downward floor scan with the shimmer/larva
/// rejections, the `trySlope` top-slope flatten (the style-17 ±30 chest
/// exclusion box) and the ±2 boulder/altar veto, then the 2×2 place one
/// row above the floor and the S1-A-4 stocking hook. The placement itself
/// draws no rng on the classic path — the loot rolls ride the hook on the
/// pass's stream. The early rejections return WITHOUT restoring the
/// flattened slopes (the C#'s own quirk — the restore only runs after a
/// failed `PlaceChest`). `ice_solid`/`spike_solid` carry the pass's
/// `Main.tileSolid[162]`/`[232]` window (the Pyramids slot: both solid;
/// the Water Chests slot: 162 non-solid after `Ice` `:16694`, 232 solid
/// until the temple `:17708`).
#[allow(clippy::too_many_arguments)]
pub(crate) fn add_buried_chest(
    world: &mut World,
    x: i32,
    y: i32,
    main_item: u16,
    not_near: bool,
    chest_style: i32,
    try_slope: bool,
    ice_solid: bool,
    spike_solid: bool,
    mut stock: Option<&mut super::chest_stock::ChestStockHook<'_>>,
    rng: &mut super::Rng,
) -> bool {
    for k in y..(world.height - 10) {
        // `:35904-35905`: the saved slopes reset every row (the C# declares
        // them inside the loop).
        let mut saved_left: i32 = -1;
        let mut saved_here: i32 = -1;
        // `:35906-35913`: the shimmer / larva rejections.
        if world
            .tile(x, k)
            .is_some_and(|t| t.liquid > 0 && t.liquid_kind() == crate::tile::LiquidKind::Shimmer)
        {
            return false;
        }
        if world
            .tile(x, k)
            .is_some_and(|t| t.active() && t.kind == 231)
        {
            return false;
        }
        // `:35914-35944`: the trySlope flatten + the style-17 exclusion.
        // The gate reads the live `tileSolid` table — the caller's window
        // bits ride it (the Water Chests slot has 162 non-solid).
        if try_slope
            && world.tile(x, k).is_some_and(|t| {
                t.active()
                    && crate::tiledata::TILE_SOLID.has(t.kind)
                    && (t.kind != 162 || ice_solid)
                    && (t.kind != 232 || spike_solid)
                    && !crate::tiledata::TILE_SOLID_TOP.has(t.kind)
            })
        {
            if chest_style == 17 {
                for l in (x - 30)..=(x + 30) {
                    for m in (k - 30)..=(k + 30) {
                        if !super::temple::in_world(world, l, m, 5) {
                            return false;
                        }
                        if world
                            .tile(l, m)
                            .is_some_and(|t| t.active() && (t.kind == 21 || t.kind == 467))
                        {
                            return false;
                        }
                    }
                }
            }
            if world.tile(x - 1, k).is_some_and(|t| t.top_slope()) {
                saved_left = world.tile(x - 1, k).unwrap().slope() as i32;
                world.tile_mut(x - 1, k).unwrap().set_slope(0);
            }
            if world.tile(x, k).is_some_and(|t| t.top_slope()) {
                saved_here = world.tile(x, k).unwrap().slope() as i32;
                world.tile_mut(x, k).unwrap().set_slope(0);
            }
        }
        // `:35949-35959`: the ±2 boulder/altar veto.
        for n in (x - 2)..=(x + 2) {
            for m in (k - 2)..=(k + 2) {
                if super::temple::in_world(world, n, m, 100)
                    && world.tile(n, m).is_some_and(|t| {
                        t.active()
                            && (crate::tiledata::SET_BOULDERS.has(t.kind)
                                || t.kind == 26
                                || t.kind == 237)
                    })
                {
                    return false;
                }
            }
        }
        // `:35960-35963`: the floor scan.
        if !solid_tile_window(world, x, k, ice_solid, spike_solid) {
            continue;
        }
        // `:35964-35988`: the resolved style (the family arms that retype
        // the chest are loot-side — the classic callers here pre-set the
        // main item, so `num9` is the caller's style or the flag15 default).
        let style = resolved_chest_style(chest_style, main_item, k, world.world_surface);
        // `:36206-36209`: the place one row above the floor.
        if super::temple::place_buried_chest(world, x - 1, k - 1, not_near, style) {
            super::chest_stock::stock_placed_chest(
                world,
                stock.as_deref_mut(),
                rng,
                x - 1,
                k - 1,
                style,
                TILE_CHEST,
                main_item,
                not_near,
            );
            return true;
        }
        // `:37554-37565`: the slope restore after a failed place.
        if try_slope {
            if saved_left > -1 {
                if let Some(t) = world.tile_mut(x - 1, k) {
                    t.set_slope(saved_left as u8);
                }
            }
            if saved_here > -1 {
                if let Some(t) = world.tile_mut(x, k) {
                    t.set_slope(saved_here as u8);
                }
            }
        }
        return false;
    }
    false
}

/// `WorldGen.SolidTile` (`WorldGen.cs:70111-70133`) under the caller's
/// `tileSolid[162]`/`[232]` window — the shared `add_buried_chest`'s floor
/// test (the temple's `solid_p2` generalised with the ice bit).
fn solid_tile_window(world: &World, x: i32, y: i32, ice_solid: bool, spike_solid: bool) -> bool {
    let Some(t) = world.tile(x, y) else {
        // The C#'s null tile reads solid (`:70116`).
        return true;
    };
    if t.kind == 162 && !ice_solid {
        return false;
    }
    if t.kind == 232 && !spike_solid {
        return false;
    }
    t.active()
        && crate::tiledata::TILE_SOLID.has(t.kind)
        && !crate::tiledata::TILE_SOLID_TOP.has(t.kind)
        && !t.half_brick()
        && t.slope() == 0
        && (t.flags & crate::tile::FLAG_INACTIVE) == 0
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

/// `Beehives` (`WorldGen.cs:15927-15976`): 1-8 hive domes in the underground
/// jungle — the hive-block (225) shell with the honey pockets (the liquid 2)
/// inside. The larva anchors ride the `larva` ledger (`GenVars.larvaX/Y`,
/// `HiveBiome.CreateStandForLarva` `HiveBiome.cs:357-365`); the
/// `BeeLarvaInBeehives` pass (`WorldGen.cs:20868-20900`, W4-d105 lane 5 #45)
/// consumes them — the 3×4 clear centre + the 231 larva.
pub(crate) fn pass_beehives(
    world: &mut World,
    spans: &super::biomes::BiomeSpans,
    rock_layer: f64,
    rng: &mut super::Rng,
    larva: &mut Vec<(i32, i32)>,
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
        // The larva anchor in ~half the hives — `HiveBiome.CreateStandForLarva`
        // (`HiveBiome.cs:357-365`) records the centre into `GenVars.larvaX/Y`
        // (clamped to the 5-border, the 100-entry cap); the
        // `BeeLarvaInBeehives` pass places the 3×4 clear + the 231 later.
        if rng.range(0, 2) == 0 {
            let lx = cx.clamp(5, world.width - 5);
            let ly = cy.clamp(5, world.height - 5);
            if larva.len() < 100 {
                larva.push((lx, ly));
            }
        }
        // The hive's honey patches (`WorldGen.cs:15946-15968`).
        super::mini_biomes::place_honey_patches_around(world, cx, cy, &mut protected, rng);
        protected.push((cx - r, cy - r, 2 * r, 2 * r));
        placed += 1;
    }
    placed
}

/// `BeeLarvaInBeehives` (`WorldGen.cs:20868-20900`): the larva ledger the
/// hive pass published (`GenVars.numLarva`/`larvaX`/`larvaY`) — each anchor
/// gets the 3×4 clear centre (the three upper rows deactivated, the floor
/// row set to hive 225 flat) and the `PlaceTile(231)` larva. The
/// `Main.tileSolid[229/232/162] = true` writes (`:20872`/`:20897-20898`)
/// are boot statics the port's generated `TILE_SOLID` table already carries
/// (the hive 229, the spikes 232, the ice 162 are solid from boot), so they
/// are no-ops here.
pub(crate) fn pass_larva(world: &mut World, larva: &[(i32, i32)]) {
    for &(x, y) in larva {
        // `:20878-20894` — the 3×4 box.
        for j in x - 1..=x + 1 {
            for k in y - 2..=y + 1 {
                if k != y + 1 {
                    // The upper three rows: deactivate (the cell keeps its
                    // wall/liquid — the C# `active(false)` only).
                    if let Some(t) = world.tile_mut(j, k) {
                        t.flags &= !FLAG_ACTIVE;
                    }
                } else {
                    // The floor row: hive 225, flat.
                    if let Some(t) = world.tile_mut(j, k) {
                        t.flags |= FLAG_ACTIVE;
                        t.kind = TILE_HIVE;
                        t.set_slope(0);
                        t.flags &= !crate::tile::FLAG_HALF_BRICK;
                    }
                }
            }
        }
        // `:20895` — `PlaceTile(x, y, 231)`: the 3×3 larva object.
        place_larva(world, x, y);
    }
}

/// `WorldGen.PlaceTile`'s larva arm (`WorldGen.cs:59967-59970` → `Place3x3`
/// `:53534-53633`, the 231 bottom-support group): the 3×3 clear box above
/// the floor (`x-1..x+1`, `y-2..y`), the three floor cells
/// `SolidTileAllowBottomSlope`, then the nine writes at `(x-1+k, y-2+l)`
/// with `frameX k*18`, `frameY l*18`. The larva pass clears the box and
/// sets the hive floor first, so the gates pass; the liquid gate
/// (`:59454-59471` — 231 is NOT in the refusal list) passes every cell.
fn place_larva(world: &mut World, x: i32, y: i32) {
    // `:53541-53554` — the clear box.
    for i in x - 1..x + 2 {
        for j in y - 2..y + 1 {
            if world.tile(i, j).is_some_and(|t| t.active()) {
                return;
            }
        }
    }
    // `:53555-53567` — the floor seats (`SolidTile2`, the 231 group's
    // :53562 check; the hive floor is flat, so it reads solid).
    for k in x - 1..x + 2 {
        if !crate::edit::solid_tile2(world, k, y + 1) {
            return;
        }
    }
    // `:53594-53632` — the nine writes (style 0, num = -2).
    for k in 0..3 {
        for l in 0..3 {
            if let Some(t) = world.tile_mut(x - 1 + k, y - 2 + l) {
                t.flags |= FLAG_ACTIVE;
                t.kind = TILE_LARVA;
                t.frame_x = (k * 18) as i16;
                t.frame_y = (l * 18) as i16;
            }
        }
    }
    crate::edit::square_tile_frame_gen(world, x, y);
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

// --- W4-g38 C2: the Traps pass ------------------------------------------------
//
// `WorldGen.cs:18657-18787` (`GenPassNameID.Traps`) + `placeTrap`
// (`:8922-9513`) + `PlaceSandTrap` (`:35463-35739`) + `placeTNTBarrel`
// (`:8886-8907`). The dart/boulder/explosive/geyser traps on cave floors, the
// no-traps-seed TNT barrels, and the underground-desert sand traps.
//
// Deviations (documented, all flag-gated or port-model seams — a classic
// world draws exactly the C# schedule):
// - `Main.tileSolid[379] = false` for the pass (`:18661`) and the
//   `SetBoulderSolidity` toggles (`:11009-11020`) ride a pass-local
//   `boulder_solid` flag threaded into [`trap_solid`] (the port's
//   `TILE_SOLID` is a static set; the bubble 379 is always non-solid here).
// - `WorldGen.placingTraps` (`:18662`/`:18784`) is a UI-only flag
//   (`UIWorldLoad.cs:93`/`:108`) — no worldgen tile/RNG effect, not ported.
// - `Main.starGame` / `starGameMath` (`:18677`/`:18729`) is the runtime
//   star-count joke seed, not a worldgen option — skipped (no draw).
// - `GenVars.shimmerPosition` proximity (`:8933`) — no shimmer position in
//   the port yet (`spider_gem_moss.rs:75` precedent), skipped.
// - `placeTrap_CanContinue`'s dual-dungeon arm (`:8915`) — no dual-dungeon
//   bounds model (`mini_biomes.rs`/`ocean.rs:620` precedent), skipped; the
//   wall-350 veto is ported.
// - `trapDiag[type, 0/1]` counters (`:4351`) are diagnostics only — no port
//   state.
// - the drunk `Thread.Sleep(10)` delay loop (`:18767-18779`) draws nothing
//   and writes nothing — not ported.
// - `SquareTileFrame` re-frames ride the post-pipeline seam (the d106
//   convention); `KillTile` uses the generation-time strip
//   (`ocean.rs:1907`/`wavy.rs:57`).

/// `TileID.Sets.Boulders` (`TileID.cs:504`) — the kinds `SetBoulderSolidity`
/// toggles and the boulder-trap family upgrades between.
const TRAP_BOULDERS: [u16; 10] = [138, 484, 664, 665, 711, 712, 713, 714, 715, 716];

/// `WorldGen.InWorld(x, y, fluff)` (`WorldGen.cs:9688-9695`).
#[inline]
fn trap_in_world(world: &World, x: i32, y: i32, fluff: i32) -> bool {
    x >= fluff && x < world.width - fluff && y >= fluff && y < world.height - fluff
}

/// `WorldGen.SolidTile(int, int)` (`WorldGen.cs:70111-70131`) under the
/// Traps window: the bubble 379 is non-solid for the whole pass
/// (`Main.tileSolid[379] = false`, `:18661`), and the boulder family is
/// non-solid while `boulder_solid` is false (the `SetBoulderSolidity(false)`
/// windows). Out-of-bounds reads false (the C# catch); the walks guard
/// bounds before calling.
fn trap_solid(world: &World, x: i32, y: i32, boulder_solid: bool) -> bool {
    let Some(t) = world.tile(x, y) else {
        return false;
    };
    if t.kind == 379 {
        return false;
    }
    if !boulder_solid && TRAP_BOULDERS.contains(&t.kind) {
        return false;
    }
    t.active()
        && td::TILE_SOLID.has(t.kind)
        && !td::TILE_SOLID_TOP.has(t.kind)
        && !t.half_brick()
        && t.slope() == 0
        && !t.inactive()
}

/// `WorldGen.SolidTile2(int, int)` (`WorldGen.cs:70492-70509`): the
/// platform half-brick/top-slope exemption OR slope 0.
fn trap_solid2(world: &World, x: i32, y: i32) -> bool {
    let Some(t) = world.tile(x, y) else {
        return false;
    };
    t.active()
        && td::TILE_SOLID.has(t.kind)
        && ((td::SET_PLATFORMS.has(t.kind) && (t.half_brick() || t.top_slope()))
            || t.slope() == 0)
        && !t.half_brick()
        && !t.inactive()
}

/// `WorldGen.SolidTile2(Tile)` (`WorldGen.cs:70134-70150`) — the `Place2x2`
/// floor overload: no platform exemption, no solid-top exclusion.
fn trap_solid2_tile(t: &Tile) -> bool {
    t.active() && td::TILE_SOLID.has(t.kind) && t.slope() == 0 && !t.half_brick() && !t.inactive()
}

/// `WorldGen.PlatformProperTopFrame` (`WorldGen.cs:70153-70165`) — the flat
/// platform frame bands (width 18, `TileObjectData.PlatformFrameWidth`).
fn trap_platform_proper_top_frame(frame_x: i16) -> bool {
    let num = i32::from(frame_x) / 18;
    (0..=7).contains(&num) || (12..=16).contains(&num) || (25..=26).contains(&num)
}

/// `WorldGen.SolidTileAllowBottomSlope(int, int)` (`WorldGen.cs:70167-70190`):
/// out-of-world reads solid; active + (solid || solidTop) + (!topSlope ||
/// platform-proper-frame) + !halfBrick + !inActive.
fn trap_solid_allow_bottom_slope(world: &World, x: i32, y: i32) -> bool {
    if !trap_in_world(world, x, y, 0) {
        return true;
    }
    let Some(t) = world.tile(x, y) else {
        return true;
    };
    t.active()
        && (td::TILE_SOLID.has(t.kind) || td::TILE_SOLID_TOP.has(t.kind))
        && (!t.top_slope()
            || (td::SET_PLATFORMS.has(t.kind) && trap_platform_proper_top_frame(t.frame_x)))
        && !t.half_brick()
        && !t.inactive()
}

/// `WorldGen.placeTrap_CanContinue` (`WorldGen.cs:8909-8920`) — the
/// unbreakable wall 350 blocks the walk. The dual-dungeon arm is deferred
/// (no bounds model; see the section note).
fn trap_can_continue(world: &World, x: i32, y: i32) -> bool {
    world.tile(x, y).is_some_and(|t| t.wall != 350)
}

/// `Tile.nactive()` — active and not actuated.
fn trap_nactive(world: &World, x: i32, y: i32) -> bool {
    world
        .tile(x, y)
        .is_some_and(|t| t.active() && !t.inactive())
}

/// `WorldGen.IsTileNearby` (`WorldGen.cs:8862-8884`) — step 1 except tile
/// 235 (step 3); the traps pass never asks for 235.
fn trap_is_tile_nearby(world: &World, x: i32, y: i32, kind: u16, distance: i32) -> bool {
    let step = if kind == 235 { 3 } else { 1 };
    let mut i = x - distance;
    while i <= x + distance {
        for j in (y - distance)..=(y + distance) {
            if trap_in_world(world, i, j, 0)
                && world.tile(i, j).is_some_and(|t| t.active() && t.kind == kind)
            {
                return true;
            }
        }
        i += step;
    }
    false
}

/// `WorldGen.AreAnyTilesInSetNearby` (`WorldGen.cs:8787-8804`) over the
/// boulder set.
fn trap_boulder_nearby(world: &World, x: i32, y: i32, distance: i32) -> bool {
    for i in (x - distance)..=(x + distance) {
        for j in (y - distance)..=(y + distance) {
            if trap_in_world(world, i, j, 0)
                && world
                    .tile(i, j)
                    .is_some_and(|t| t.active() && TRAP_BOULDERS.contains(&t.kind))
            {
                return true;
            }
        }
    }
    false
}

/// `WorldGen.CanGeneratePressurePlateAt` (`WorldGen.cs:11029-11049`).
fn trap_can_pressure_plate(world: &World, x: i32, y: i32) -> bool {
    if !trap_in_world(world, x, y, 3) {
        return false;
    }
    if !trap_solid_allow_bottom_slope(world, x, y + 1) {
        return false;
    }
    let Some(below) = world.tile(x, y + 1) else {
        return false;
    };
    if TRAP_BOULDERS.contains(&below.kind) {
        return false;
    }
    if below.wall == 350 {
        return false;
    }
    true
}

/// `WorldGen.KillTile`'s generation-time strip (`WorldGen.cs:63517`, the
/// `isGeneratingOrLoadingWorld → noItem` path; `ocean.rs:1907`): the tile
/// goes inactive, type/frames cleared, wall + liquid kept. The item-drop /
/// net / `SquareTileFrame` arms do not exist in the port's worldgen.
fn trap_kill(world: &mut World, x: i32, y: i32) {
    if !trap_in_world(world, x, y, 0) {
        return;
    }
    let i = world.index(x, y);
    let t = &mut world.tiles[i];
    if !t.active() {
        return;
    }
    t.flags &= !FLAG_ACTIVE;
    t.kind = 0;
    t.frame_x = 0;
    t.frame_y = 0;
}

/// `WorldGen.Place1x1` (`WorldGen.cs:45188-45235`), the default style branch
/// the pressure plate 135 / explosive 141 ride: `InWorld(x, y, 2)`, the
/// mechanism-on-boulder veto, a `SolidTile2` floor and an empty cell, then
/// active + kind + `frameY = 18 * style` (frameX keeps its cleared 0).
fn trap_place_1x1(world: &mut World, x: i32, y: i32, kind: u16, style: i32) {
    if !trap_in_world(world, x, y, 2) {
        return;
    }
    let below_kind = world.tile(x, y + 1).map(|t| t.kind).unwrap_or(0);
    let mechanism = td::SET_IS_A_MECHANISM.has(kind);
    let veto_ok = !mechanism || !TRAP_BOULDERS.contains(&below_kind);
    if !veto_ok || !trap_solid2(world, x, y + 1) {
        return;
    }
    if world.tile(x, y).is_some_and(|t| t.active()) {
        return;
    }
    let i = world.index(x, y);
    let t = &mut world.tiles[i];
    t.flags |= FLAG_ACTIVE;
    t.kind = kind;
    t.frame_y = (18 * style) as i16;
}

/// `WorldGen.Place2x2` (`WorldGen.cs:51330-51423`), style 0 — the boulder /
/// TNT-barrel body: the 5-tile bounds gate, the 2×2 body `(x-1..x, y-1..y)`
/// empty, both floor columns `nactive && (SolidTile2(Tile) || tileTable)`,
/// then the 2×2 at `(x-1, y-1)` with `frameX = k*18`, `frameY = l*18`.
fn trap_place_2x2(world: &mut World, x: i32, y: i32, kind: u16) {
    if x < 5 || x > world.width - 5 || y < 5 || y > world.height - 5 {
        return;
    }
    for i in (x - 1)..x + 1 {
        for j in (y - 1)..y + 1 {
            if world.tile(i, j).is_some_and(|t| t.active()) {
                return;
            }
        }
        let Some(below) = world.tile(i, y + 1) else {
            return;
        };
        if !crate::tileobject::nactive(below)
            || (!trap_solid2_tile(below) && !crate::tileobject::is_tile_table(below.kind))
        {
            return;
        }
    }
    for k in 0..2 {
        for l in 0..2 {
            let i = world.index(x - 1 + k, y - 1 + l);
            let t = &mut world.tiles[i];
            t.flags |= FLAG_ACTIVE;
            t.frame_x = (k * 18) as i16;
            t.frame_y = (l * 18) as i16;
            t.kind = kind;
        }
    }
}

/// `WorldGen.PlaceTile(i, j, kind, mute, forced, -1, style)`
/// (`WorldGen.cs:59342-60295`), the slice the Traps pass runs for kinds
/// {1, 135, 137, 141, 138/484/664/711-716, 654}: the gen-time entry gate
/// (`forced || EmptyTile || !tileSolid[kind]` — no players/NPCs at gen, so
/// `EmptyTile = !active`), the inactive-cell `Clear(Tile|TilePaint|Slope)`,
/// the per-kind dispatch, and the `result = tile.active()` tail. The
/// `SquareTileFrame` re-frame rides the post-pipeline seam. Returns whether
/// the cell ended active.
fn trap_place_tile(world: &mut World, x: i32, y: i32, kind: u16, style: i32, forced: bool) -> bool {
    if !trap_in_world(world, x, y, 0) {
        return false;
    }
    let active = world.tile(x, y).is_some_and(|t| t.active());
    // `:59373` entry — the moss/grass special arms are other kinds' paths.
    if !(forced || !active || !td::TILE_SOLID.has(kind)) {
        return false;
    }
    // `:59473-59476`: an inactive cell clears tile + paint/coating + slope.
    if !active {
        let i = world.index(x, y);
        let t = &mut world.tiles[i];
        t.kind = 0;
        t.flags &= !FLAG_ACTIVE;
        t.frame_x = 0;
        t.frame_y = 0;
        t.set_paint(0);
        t.flags &= !crate::tile::FLAG_FULLBRIGHT;
        t.set_slope(0);
        t.flags &= !FLAG_HALF_BRICK;
        world.set_invisible_block(x, y, false);
    }
    // The dispatch (`:59738-59755`, `:60129-60140`, the default block).
    match kind {
        137 => {
            // `:59750-59755`: active + type + frameY = 18*style.
            let i = world.index(x, y);
            let t = &mut world.tiles[i];
            t.flags |= FLAG_ACTIVE;
            t.kind = 137;
            t.frame_y = (18 * style) as i16;
        }
        135 | 141 => trap_place_1x1(world, x, y, kind, style),
        138 | 484 | 664 | 711 | 712 | 713 | 714 | 715 | 716 | 654 => {
            trap_place_2x2(world, x, y, kind);
        }
        _ => {
            // The default block (`:60234-60248`): active + type (the
            // 53/396/397 paint arms are other kinds').
            let i = world.index(x, y);
            let t = &mut world.tiles[i];
            t.flags |= FLAG_ACTIVE;
            t.kind = kind;
        }
    }
    world.tile(x, y).is_some_and(|t| t.active())
}

/// The red-wire staircase walk (`WorldGen.cs:9180-9201` and peers): x first,
/// then y, laying the red bit at every step (`Main.tile[..].wire(true)`).
fn trap_wire_walk(world: &mut World, x0: i32, y0: i32, x1: i32, y1: i32) {
    let (mut wx, mut wy) = (x0, y0);
    while wx != x1 || wy != y1 {
        world.wires.set(wx, wy, crate::wiring::WireColor::Red, true);
        if wx > x1 {
            wx -= 1;
        }
        if wx < x1 {
            wx += 1;
        }
        world.wires.set(wx, wy, crate::wiring::WireColor::Red, true);
        if wy > y1 {
            wy -= 1;
        }
        if wy < y1 {
            wy += 1;
        }
        world.wires.set(wx, wy, crate::wiring::WireColor::Red, true);
    }
}

/// `Main.tileDungeon` (`Main.cs:7992-7997`) — the six dungeon brick kinds the
/// boulder-trap headroom veto reads.
const TRAP_DUNGEON_TILES: [u16; 6] = [41, 43, 44, 677, 678, 679];
/// `Main.wallDungeon` (`Main.cs:10598-10606`) — the ghoulder proximity scan.
const TRAP_DUNGEON_WALLS: [u16; 9] = [7, 8, 9, 94, 95, 96, 97, 98, 99];
/// `Main.tileCracked` (`Main.cs:7190-7192`) — the dart-trap wall search.
const TRAP_CRACKED: [u16; 3] = [481, 482, 483];

/// `WorldGen.closeEnoughToSpidersToSpawnSpiderBoulder`
/// (`WorldGen.cs:8830-8844`): wall 62 within 80, step 3.
fn trap_near_spiders(world: &World, x: i32, y: i32) -> bool {
    let mut i = x - 80;
    while i <= x + 80 {
        let mut j = y - 80;
        while j <= y + 80 {
            if trap_in_world(world, i, j, 0) && world.tile(i, j).is_some_and(|t| t.wall == 62) {
                return true;
            }
            j += 3;
        }
        i += 3;
    }
    false
}

/// `WorldGen.closeEnoughToDungeonToSpawnGhoulder`
/// (`WorldGen.cs:8806-8828`): the dungeon-side half gate, then a dungeon wall
/// within 1000, step 10. `dungeon_side` is −1 Left / +1 Right (the port's
/// `spans().dungeon_x` half, `DungeonSide.cs:13-15`).
fn trap_near_dungeon(world: &World, x: i32, y: i32, dungeon_side: i32) -> bool {
    if dungeon_side <= -1 && x > world.width / 2 {
        return false;
    }
    if dungeon_side >= 1 && x < world.width / 2 {
        return false;
    }
    let mut i = x - 1000;
    while i <= x + 1000 {
        let mut j = y - 1000;
        while j <= y + 1000 {
            if trap_in_world(world, i, j, 0)
                && world
                    .tile(i, j)
                    .is_some_and(|t| TRAP_DUNGEON_WALLS.contains(&t.wall))
            {
                return true;
            }
            j += 10;
        }
        i += 10;
    }
    false
}

/// `WorldGen.closeEnoughToLavaToSpawnLavaBoulder`
/// (`WorldGen.cs:8846-8860`): lava within 60, step 3.
fn trap_near_lava(world: &World, x: i32, y: i32) -> bool {
    let mut i = x - 60;
    while i <= x + 60 {
        let mut j = y - 60;
        while j <= y + 60 {
            if trap_in_world(world, i, j, 0)
                && world
                    .tile(i, j)
                    .is_some_and(|t| t.liquid > 0 && t.flags & FLAG_LIQUID_LAVA != 0)
            {
                return true;
            }
            j += 3;
        }
        i += 3;
    }
    false
}

/// The seed-flag bundle the Traps pass reads (the `WorldGen.*WorldGen`
/// booleans, `WorldGen.cs:11115-11123`).
#[derive(Clone, Copy)]
struct TrapSeeds {
    no_traps: bool,
    get_good: bool,
    bees: bool,
    remix: bool,
    // The drunk delay loop (`:18767-18779`) draws/writes nothing — the flag
    // is carried for the seed-bundle parity but never read.
    #[allow(dead_code)]
    drunk: bool,
    tenth: bool,
    dont_starve: bool,
    skyblock: bool,
    poo: bool,
}

impl TrapSeeds {
    fn from_opts(opts: &GenOptions) -> Self {
        TrapSeeds {
            no_traps: opts.flags.contains(SeedFlags::NO_TRAPS),
            get_good: opts.flags.contains(SeedFlags::FOR_THE_WORTHY),
            bees: opts.flags.contains(SeedFlags::NOT_THE_BEES),
            remix: opts.flags.contains(SeedFlags::REMIX),
            drunk: opts.flags.contains(SeedFlags::DRUNK),
            tenth: opts.flags.contains(SeedFlags::TENTH_ANNIVERSARY),
            dont_starve: opts.flags.contains(SeedFlags::DONT_STARVE),
            skyblock: opts.flags.contains(SeedFlags::SKYBLOCK),
            poo: opts.micro_flags.contains(MicroSeedFlags::POO_EVERYWHERE),
        }
    }
}

/// `WorldGen.placeTrap(x2, y2, type)` (`WorldGen.cs:8922-9513`): the
/// dart (0) / boulder (1) / explosive (2) / geyser (3) trap at a cave-floor
/// point. `type == -1` rolls the variant; the pass always passes −1.
/// `boulder_solid` mirrors the `SetBoulderSolidity` window (true on entry).
/// Returns whether a trap placed.
#[allow(clippy::too_many_arguments)]
fn place_trap(
    world: &mut World,
    x2: i32,
    y2: i32,
    mut kind: i32,
    rng: &mut super::Rng,
    seeds: &TrapSeeds,
    world_surface: f64,
    rock_layer: f64,
    lava_line: i32,
    dungeon_side: i32,
    boulder_pets: &mut i32,
) -> bool {
    // `:8924-8927`: actuallyNoTrapsForRealIMeanIt reports success without
    // placing (the pass's outer gate already skips it; kept for parity).
    if !trap_can_continue(world, x2, y2) {
        return false;
    }
    // `:8933` shimmer proximity — deferred (no shimmer position).
    let mut num = y2;
    let mut lava_here = false;
    let mut near_underworld = false;
    // `:8939-8942`: noTraps drops boulder solidity for the floor walk.
    let walk_solid = !seeds.no_traps;
    while !trap_solid(world, x2, num, walk_solid) {
        num += 1;
        if num > world.height - 10 {
            return false;
        }
        if !trap_can_continue(world, x2, num) {
            return false;
        }
        if num >= world.height - 300 {
            near_underworld = true;
        }
    }
    // `:8959-8962`: restore boulder solidity.
    num -= 1;
    if !seeds.no_traps && trap_is_tile_nearby(world, x2, num, 70, 20) {
        return false;
    }
    let floor_wall = world.tile(x2, num).map(|t| t.wall).unwrap_or(0);
    if floor_wall == 87 || floor_wall == 350 {
        return false;
    }
    if world
        .tile(x2, num)
        .is_some_and(|t| t.liquid > 0 && t.flags & FLAG_LIQUID_LAVA != 0)
    {
        lava_here = true;
    }
    // The variant roll (`:8980-9013`).
    if seeds.remix {
        if kind == -1 && rng.range(0, 20) == 0 {
            kind = 2;
        } else if kind == -1 && (num as f64) < rock_layer - 30.0 && rng.range(0, 3) != 0 {
            kind = 3;
        } else if kind == -1 {
            kind = rng.range(0, 2);
        }
    } else {
        if kind == -1 && rng.range(0, 20) == 0 {
            kind = 2;
        } else if kind == -1 && num > lava_line + 30 && rng.range(0, 6) != 0 {
            kind = 3;
        } else if kind == -1 {
            kind = rng.range(0, 2);
        }
        if seeds.no_traps && seeds.tenth && rng.range(0, 4) == 0 {
            kind = 1;
        }
    }
    if !trap_in_world(world, x2, num, 3) {
        return false;
    }
    if lava_here && kind != 3 {
        return false;
    }
    if near_underworld && kind != 3 {
        return false;
    }
    // The 3×3 box above the plate must be free (`:9026`) — the C# cell
    // order: rows num, num-1, num-2; cols x2, x2-1, x2+1.
    for dy in [0, -1, -2] {
        for dx in [0, -1, 1] {
            if trap_nactive(world, x2 + dx, num + dy) {
                return false;
            }
        }
    }
    if world
        .tile(x2, num + 1)
        .is_some_and(|t| t.active() && matches!(t.kind, 48 | 232))
    {
        return false;
    }
    if !trap_can_pressure_plate(world, x2, num) {
        return false;
    }
    // `:9039-9051`: a boulder near snow/ice downgrades to a dart.
    if kind == 1 {
        for i in (x2 - 3)..=(x2 + 3) {
            for j in (num - 3)..=(num + 3) {
                if world
                    .tile(i, j)
                    .is_some_and(|t| t.active() && matches!(t.kind, 147 | 161))
                {
                    kind = 0;
                }
            }
        }
    }
    if seeds.skyblock && kind == 2 {
        return false;
    }
    // `:9056-9059`: noTraps drops boulder solidity for the placement.
    let place_solid = !seeds.no_traps;
    match kind {
        0 => place_trap_dart(world, x2, num, rng, seeds, place_solid),
        1 => place_trap_boulder(
            world, x2, num, rng, seeds, world_surface, dungeon_side, boulder_pets,
        ),
        2 => place_trap_explosive(world, x2, num, rng, place_solid),
        3 => place_trap_geyser(world, x2, num, rng, seeds),
        _ => false,
    }
}

/// `placeTrap` case 0 — the dart trap (`WorldGen.cs:9062-9204`).
fn place_trap_dart(
    world: &mut World,
    x2: i32,
    num: i32,
    rng: &mut super::Rng,
    seeds: &TrapSeeds,
    boulder_solid: bool,
) -> bool {
    let num11 = num - rng.range(0, 3);
    // Walk left to the first solid/cracked cell (`:9066-9074`).
    let mut num10 = x2;
    loop {
        let stuck = trap_solid(world, num10, num11, boulder_solid)
            || world
                .tile(num10, num11)
                .is_some_and(|t| t.active() && TRAP_CRACKED.contains(&t.kind));
        if stuck {
            break;
        }
        num10 -= 1;
        if num10 < 0 || !trap_can_continue(world, num10, num11) {
            return false;
        }
    }
    let num12 = num10;
    // Walk right the same way (`:9077-9085`).
    num10 = x2;
    loop {
        let stuck = trap_solid(world, num10, num11, boulder_solid)
            || world
                .tile(num10, num11)
                .is_some_and(|t| t.active() && TRAP_CRACKED.contains(&t.kind));
        if stuck {
            break;
        }
        num10 += 1;
        if num10 >= world.width || !trap_can_continue(world, num10, num11) {
            return false;
        }
    }
    let num13 = num10;
    let left_dist = x2 - num12;
    let right_dist = num13 - x2;
    let mut left_ok = left_dist > 5 && left_dist < 50;
    let mut right_ok = right_dist > 5 && right_dist < 50;
    if left_ok && !trap_solid(world, num12, num11 + 1, boulder_solid) {
        left_ok = false;
    }
    if right_ok && !trap_solid(world, num13, num11 + 1, boulder_solid) {
        right_ok = false;
    }
    let bad = |w: &World, x: i32| {
        w.tile(x, num11)
            .is_some_and(|t| t.active() && matches!(t.kind, 10 | 48))
            || w.tile(x, num11 + 1)
                .is_some_and(|t| t.active() && matches!(t.kind, 10 | 48))
    };
    if left_ok && bad(world, num12) {
        left_ok = false;
    }
    if right_ok && bad(world, num13) {
        right_ok = false;
    }
    let (wall_x, dir) = if left_ok && right_ok {
        if rng.range(0, 2) == 0 {
            (num13, -1)
        } else {
            (num12, 1)
        }
    } else if right_ok {
        (num13, -1)
    } else if left_ok {
        (num12, 1)
    } else {
        return false;
    };
    if world.tile(wall_x, num11).is_some_and(|t| t.kind == 190) {
        return false;
    }
    if world.tile(wall_x, num11).is_some_and(|t| t.wall == 350) {
        return false;
    }
    // The plate: style 2 on a wall, else a 2..3 roll (`:9151-9158`).
    let floor_wall = world.tile(x2, num).map(|t| t.wall).unwrap_or(0);
    if floor_wall > 0 {
        trap_place_tile(world, x2, num, 135, 2, true);
    } else {
        let style = rng.range(2, 4);
        trap_place_tile(world, x2, num, 135, style, true);
    }
    trap_kill(world, wall_x, num11);
    // The dart style (`:9160-9168`): bees+noTraps+!remix rolls 1..2, liquid
    // nearby forces 1.
    let mut num17 = 0i32;
    if seeds.bees && seeds.no_traps && !seeds.remix {
        num17 = rng.range(1, 3);
        let liquid = |w: &World, x: i32| w.tile(x, num11).is_some_and(|t| t.liquid > 0);
        if liquid(world, wall_x) || liquid(world, wall_x - 1) || liquid(world, wall_x + 1) {
            num17 = 1;
        }
    }
    trap_place_tile(world, wall_x, num11, 137, num17, true);
    if num17 > 0 && seeds.dont_starve {
        if let Some(t) = world.tile_mut(wall_x, num11) {
            t.set_paint(27);
        }
    }
    if dir == 1 {
        if let Some(t) = world.tile_mut(wall_x, num11) {
            t.frame_x += 18;
        }
    }
    trap_wire_walk(world, x2, num, wall_x, num11);
    true
}

/// `placeTrap` case 1 — the boulder trap (`WorldGen.cs:9205-9405`).
#[allow(clippy::too_many_arguments)]
fn place_trap_boulder(
    world: &mut World,
    x2: i32,
    num: i32,
    rng: &mut super::Rng,
    seeds: &TrapSeeds,
    world_surface: f64,
    dungeon_side: i32,
    boulder_pets: &mut i32,
) -> bool {
    // The pet budget by world size (`:9207-9217`).
    let world_size = if world.width <= 4200 {
        0
    } else if world.width <= 6400 {
        1
    } else {
        2
    };
    let mut pet_budget = match world_size {
        1 => 4i32,
        2 => 6,
        _ => 2,
    };
    if seeds.no_traps {
        pet_budget *= 2;
    }
    let num21 = x2 + rng.range(-1, 2);
    let mut num22 = num - 8;
    // Scan up for a solid 6×4 headroom block (`:9222-9269`).
    loop {
        let mut all_solid = true;
        let mut dirt_stone_mud = 0i32;
        for i in (num21 - 2)..=(num21 + 3) {
            for j in num22..=(num22 + 3) {
                if !trap_solid(world, i, j, true) {
                    all_solid = false;
                }
                if !trap_can_continue(world, i, j) {
                    return false;
                }
                if let Some(t) = world.tile(i, j) {
                    if t.active() {
                        if t.kind == 226 || TRAP_DUNGEON_TILES.contains(&t.kind) {
                            return false;
                        }
                        if t.actuator() || TRAP_BOULDERS.contains(&t.kind) {
                            return false;
                        }
                        if matches!(t.kind, 0 | 1 | 59) {
                            dirt_stone_mud += 1;
                        }
                    }
                }
            }
        }
        num22 -= 1;
        if (num22 as f64) < world_surface {
            return false;
        }
        if all_solid && dirt_stone_mud > 2 {
            break;
        }
    }
    let drop = num - num22;
    if drop <= 5 || drop >= 40 {
        return false;
    }
    if trap_is_tile_nearby(world, num21, num22, 21, 4)
        || trap_is_tile_nearby(world, num21, num22, 467, 4)
    {
        return false;
    }
    if seeds.no_traps {
        if trap_boulder_nearby(world, num21, num22, 6) {
            return false;
        }
    } else if trap_boulder_nearby(world, num21, num22, 10) {
        return false;
    }
    // Carve the 2-wide drop shaft, sparing bubbles (`:9291-9300`).
    for i in num21..=(num21 + 1) {
        for j in num22..=num {
            if world.tile(i, j).map(|t| t.kind).unwrap_or(0) != 379 {
                trap_kill(world, i, j);
            }
        }
    }
    // Stone-ring the 6×6 headroom (`:9301-9310`).
    for i in (num21 - 2)..=(num21 + 3) {
        for j in (num22 - 2)..=(num22 + 3) {
            if trap_solid(world, i, j, true) {
                if let Some(t) = world.tile_mut(i, j) {
                    t.kind = 1;
                }
            }
        }
    }
    trap_place_tile(world, x2, num, 135, 7, true);
    trap_place_tile(world, num21, num22 + 2, 1, 0, false);
    trap_place_tile(world, num21 + 1, num22 + 2, 1, 0, false);
    // The boulder-variant roll (`:9314-9360`).
    let mut boulder_kind: u16 = 138;
    let mut max_value = 7i32;
    let mut max_value2 = 2i32;
    let mut max_value3 = 3i32;
    let mut max_value4 = 6i32;
    if seeds.no_traps {
        max_value2 = 1;
        max_value3 = 2;
        max_value4 = 4;
    }
    if seeds.no_traps && !seeds.remix {
        max_value = 5;
    }
    if rng.range(0, max_value2) == 0 && trap_near_spiders(world, num21, num22) {
        boulder_kind = 714;
    } else if rng.range(0, max_value4) == 0 && trap_near_dungeon(world, num21, num22, dungeon_side) {
        boulder_kind = 715;
    } else if rng.range(0, max_value3) == 0 && trap_near_lava(world, num21, num22) {
        boulder_kind = 713;
    } else if (seeds.bees || seeds.no_traps || seeds.poo) && rng.range(0, 5) == 0 {
        boulder_kind = 712;
    } else if seeds.tenth && rng.range(0, max_value) == 0 {
        boulder_kind = 711;
    } else if (seeds.tenth || seeds.no_traps) && rng.range(0, 3) == 0 {
        boulder_kind = 664;
    } else if rng.range(0, 25) == 0 {
        boulder_kind = 711;
    } else if rng.range(0, 20) == 0 {
        boulder_kind = 664;
    }
    if boulder_kind == 138 && *boulder_pets < pet_budget {
        *boulder_pets += 1;
        boulder_kind = 716;
    }
    trap_place_tile(world, num21 + 1, num22 + 1, boulder_kind, 0, false);
    num22 += 2;
    // The 2×3 actuated stone floor under the boulder (`:9368-9378`).
    for i in 0..2 {
        for j in 0..3 {
            let px = num21 + i;
            let py = num22 + j;
            trap_place_tile(world, px, py, 1, 0, false);
            world.wires.set(px, py, crate::wiring::WireColor::Red, true);
            if let Some(t) = world.tile_mut(px, py) {
                t.flags |= FLAG_ACTUATOR;
            }
        }
    }
    trap_wire_walk(world, x2, num, num21, num22);
    true
}

/// `placeTrap` case 2 — the explosive trap (`WorldGen.cs:9406-9463`).
fn place_trap_explosive(
    world: &mut World,
    x2: i32,
    num: i32,
    rng: &mut super::Rng,
    boulder_solid: bool,
) -> bool {
    let depth = rng.range(4, 7);
    let num5 = x2 + rng.range(-1, 2);
    let mut num6 = num;
    for _ in 0..depth {
        num6 += 1;
        if !trap_in_world(world, num5, num6, 5)
            || !trap_solid(world, num5, num6, boulder_solid)
            || !trap_can_continue(world, num5, num6)
        {
            return false;
        }
    }
    for i in (num5 - 2)..=(num5 + 2) {
        for j in (num6 - 2)..=(num6 + 2) {
            if !trap_in_world(world, i, j, 0)
                || !trap_solid(world, i, j, boulder_solid)
                || !trap_can_continue(world, i, j)
            {
                return false;
            }
        }
    }
    trap_kill(world, num5, num6);
    // The explosive write (`:9432-9435`): active + 141 + frameY 18*Next(2).
    let fy = (18 * rng.range(0, 2)) as i16;
    if let Some(t) = world.tile_mut(num5, num6) {
        t.flags |= FLAG_ACTIVE;
        t.kind = 141;
        t.frame_x = 0;
        t.frame_y = fy;
    }
    let style = rng.range(2, 4);
    trap_place_tile(world, x2, num, 135, style, true);
    trap_wire_walk(world, x2, num, num5, num6);
    // `:9461-9462`: trapDiag++ then `break` — the switch falls to the
    // `return false` tail (`:9512`), so the explosive reports failure even
    // though it placed. Ported verbatim.
    false
}

/// `placeTrap` case 3 — the geyser trap (`WorldGen.cs:9464-9506`).
fn place_trap_geyser(
    world: &mut World,
    x2: i32,
    num: i32,
    rng: &mut super::Rng,
    seeds: &TrapSeeds,
) -> bool {
    if world.tile(x2 + 1, num).is_some_and(|t| t.active()) {
        return false;
    }
    if world
        .tile(x2, num)
        .is_some_and(|t| t.liquid > 0 && t.flags & FLAG_LIQUID_LAVA == 0)
    {
        return false;
    }
    if !trap_can_continue(world, x2, num) {
        return false;
    }
    if seeds.no_traps && (seeds.tenth || seeds.bees) {
        if rng.range(0, 3) != 0 {
            return false;
        }
        if trap_is_tile_nearby(world, x2, num, 443, 30) {
            return false;
        }
    }
    for k in x2..=(x2 + 1) {
        if !trap_in_world(world, k, num + 1, 5)
            || !trap_solid(world, k, num + 1, !seeds.no_traps)
            || !trap_can_continue(world, k, num + 1)
        {
            return false;
        }
    }
    let style = rng.range(0, 2);
    for l in 0..2 {
        if let Some(t) = world.tile_mut(x2 + l, num) {
            t.flags |= FLAG_ACTIVE;
            t.kind = 443;
            t.frame_x = (18 * l + 36 * style) as i16;
            t.frame_y = 0;
        }
    }
    true
}

/// `WorldGen.placeTNTBarrel(x, y)` (`WorldGen.cs:8886-8907`): drop to the
/// first active tile (bounded by `maxTilesY - 350`), step up one, refuse on
/// shimmer, then `PlaceTile(x, num, 654)`.
fn place_tnt_barrel(world: &mut World, x: i32, y: i32) -> bool {
    let mut num = y;
    while !world.tile(x, num).is_some_and(|t| t.active()) {
        num += 1;
        if num > world.height - 350 {
            return false;
        }
    }
    num -= 1;
    if world
        .tile(x, num)
        .is_some_and(|t| t.liquid > 0 && t.liquid_kind() == crate::tile::LiquidKind::Shimmer)
    {
        return false;
    }
    trap_place_tile(world, x, num, 654, 0, false)
}

/// `Tile.ClearTile()` (`Tile.cs`): `ClearSlope` + `active(false)` +
/// `inActive(false)`.
fn trap_clear_tile(t: &mut Tile) {
    t.set_slope(0);
    t.flags &= !FLAG_HALF_BRICK;
    t.flags &= !FLAG_ACTIVE;
    t.flags &= !FLAG_INACTIVE;
}

/// `WorldGen.PlaceSandTrap(i, j)` (`WorldGen.cs:35463-35739`): the
/// underground-desert sand trap — a sandstone chamber with an actuated sand
/// plug wired to a pressure plate. `Main.UnderworldLayer` is `maxTilesY -
/// 200` (`Main.cs:2790`).
fn place_sand_trap(world: &mut World, i: i32, j: i32, rng: &mut super::Rng) -> bool {
    let underworld_layer = world.height - 200;
    // Drop to the first active tile (`:35473-35479`).
    let mut k = j;
    while !world.tile(i, k).is_some_and(|t| t.active()) && k < underworld_layer {
        k += 1;
    }
    if k >= underworld_layer {
        return false;
    }
    let seat = world.tile(i, k).copied();
    let Some(seat) = seat else {
        return false;
    };
    if !td::TILE_SOLID.has(seat.kind) || seat.half_brick() || seat.top_slope() {
        return false;
    }
    if !matches!(seat.kind, 53 | 397 | 396) || !matches!(seat.wall, 216 | 187) {
        return false;
    }
    k -= 1;
    let mut ceiling = -1i32;
    let half_width = rng.range(6, 12);
    let height = rng.range(6, 14);
    // The 50×50 exclusion scan (`:35492-35529`): no wire / chest / painting /
    // door / platform / chair / extractinator / track nearby.
    for l in (i - 25)..=(i + 25) {
        for m in (k - 25)..(k + 25) {
            let Some(t) = world.tile(l, m) else {
                continue;
            };
            if world.wires.get(l, m, crate::wiring::WireColor::Red) {
                return false;
            }
            if td::SET_BASIC_CHEST.has(t.kind) {
                return false;
            }
            if td::SET_PAINTINGS.has(t.kind) {
                return false;
            }
            if matches!(t.kind, 10 | 19 | 15 | 219 | 314) {
                return false;
            }
        }
    }
    // The 5×3 solid floor (`:35530-35539`).
    for n in (i - 2)..=(i + 2) {
        for m in (k + 1)..=(k + 3) {
            if !world
                .tile(n, m)
                .is_some_and(|t| t.active() && td::TILE_SOLID.has(t.kind))
            {
                return false;
            }
        }
    }
    if world.tile(i, k + 1).is_some_and(|t| t.kind == 162) {
        return false;
    }
    // Find the sandstone ceiling within 30 above (`:35544-35555`).
    for m in (0..30).map(|d| k - d) {
        if world.tile(i, m).is_some_and(|t| t.active()) {
            if world.tile(i, m).is_some_and(|t| t.kind == 396) {
                ceiling = m;
                break;
            }
            return false;
        }
    }
    if ceiling <= -1 {
        return false;
    }
    if k - ceiling < height + 4 {
        return false;
    }
    // The chamber must be mostly solid with no ore (`:35564-35590`).
    let mut solid_count = 0i32;
    let mid = (k + ceiling) / 2;
    for n in (i - half_width)..=(i + half_width) {
        if world
            .tile(n, mid)
            .is_some_and(|t| t.active() && td::TILE_SOLID.has(t.kind))
        {
            return false;
        }
        for m in (ceiling - height)..=ceiling {
            if let Some(t) = world.tile(n, m) {
                if t.active() {
                    if td::SET_ORE.has(t.kind) || t.kind == 404 {
                        return false;
                    }
                    if td::TILE_SOLID.has(t.kind) {
                        solid_count += 1;
                    }
                }
            }
        }
    }
    if (solid_count as f64) < ((half_width * 2 + 1) * (height + 1)) as f64 * 0.75 {
        return false;
    }
    // Build the chamber shell (`:35591-35644`).
    for n in (i - half_width - 1)..=(i + half_width + 1) {
        for m in (ceiling - height)..=ceiling {
            let was_solid = world
                .tile(n, m)
                .is_some_and(|t| t.active() && td::TILE_SOLID.has(t.kind));
            if m == ceiling {
                if let Some(t) = world.tile_mut(n, m) {
                    t.set_slope(0);
                    t.flags &= !FLAG_HALF_BRICK;
                    if !was_solid {
                        t.flags |= FLAG_ACTIVE;
                        t.kind = 396;
                    }
                }
            } else if m == ceiling - height {
                let above_solid = world
                    .tile(n, m - 1)
                    .is_some_and(|t| t.active() && td::TILE_SOLID.has(t.kind));
                if let Some(t) = world.tile_mut(n, m) {
                    trap_clear_tile(t);
                    t.flags |= FLAG_ACTIVE;
                    t.kind = if was_solid && above_solid { 397 } else { 396 };
                }
            } else if n == i - half_width - 1 || n == i + half_width + 1 {
                if let Some(t) = world.tile_mut(n, m) {
                    if !was_solid {
                        trap_clear_tile(t);
                        t.flags |= FLAG_ACTIVE;
                        t.kind = 396;
                    } else {
                        t.set_slope(0);
                        t.flags &= !FLAG_HALF_BRICK;
                    }
                }
            } else if let Some(t) = world.tile_mut(n, m) {
                trap_clear_tile(t);
                t.flags |= FLAG_ACTIVE;
                t.kind = 53;
            }
        }
    }
    // The side-column slope cleanup (`:35645-35677`): the slope/half-brick
    // arms read the LEFT column only; the solid-fill applies to both.
    let lo = (ceiling as f64 - height as f64 * 0.666) as i32;
    let hi = ceiling as f64 - height as f64 * 0.333;
    let left = i - half_width - 2;
    let right = i + half_width + 2;
    let mut m = lo;
    while (m as f64) <= hi {
        if (m as f64) < ceiling as f64 - height as f64 * 0.4 {
            if let Some(t) = world.tile_mut(left, m) {
                if t.bottom_slope() {
                    t.set_slope(0);
                }
            }
        } else if (m as f64) > ceiling as f64 - height as f64 * 0.6 {
            if let Some(t) = world.tile_mut(left, m) {
                if t.top_slope() {
                    t.set_slope(0);
                }
                t.flags &= !FLAG_HALF_BRICK;
            }
        } else if let Some(t) = world.tile_mut(left, m) {
            t.flags &= !FLAG_HALF_BRICK;
            t.set_slope(0);
        }
        for side in [left, right] {
            if !world
                .tile(side, m)
                .is_some_and(|t| t.active() && td::TILE_SOLID.has(t.kind))
            {
                if let Some(t) = world.tile_mut(side, m) {
                    t.flags |= FLAG_ACTIVE;
                    t.kind = 396;
                }
            }
        }
        m += 1;
    }
    // Flatten the four inner side columns (`:35678-35688`).
    for m in (ceiling - height)..=ceiling {
        for side in [
            i - half_width - 2,
            i - half_width - 1,
            i - half_width + 1,
            i - half_width + 2,
        ] {
            if let Some(t) = world.tile_mut(side, m) {
                t.set_slope(0);
                t.flags &= !FLAG_HALF_BRICK;
            }
        }
    }
    // The ceiling-row slope cleanup (`:35689-35697`).
    for n in (i - half_width - 1)..(i + half_width + 1) {
        let m = k - height - 1;
        if let Some(t) = world.tile_mut(n, m) {
            if t.bottom_slope() {
                t.set_slope(0);
            }
            t.flags &= !FLAG_HALF_BRICK;
        }
    }
    // Open the plug mouth and set the plate (`:35698-35702`).
    trap_kill(world, i - 2, k);
    trap_kill(world, i - 1, k);
    trap_kill(world, i + 1, k);
    trap_kill(world, i + 2, k);
    trap_place_tile(world, i, k, 135, 7, false);
    // The actuated sand plug + wire (`:35703-35737`).
    for n in (i - half_width)..=(i + half_width) {
        let nf = n as f64;
        let iff = i as f64;
        let hwf = half_width as f64;
        let mut mouth = k;
        if nf < iff - hwf * 0.8 || nf > iff + hwf * 0.8 {
            mouth = k - 3;
        } else if nf < iff - hwf * 0.6 || nf > iff + hwf * 0.6 {
            mouth = k - 2;
        } else if nf < iff - hwf * 0.4 || nf > iff + hwf * 0.4 {
            mouth = k - 1;
        }
        for m in ceiling..=k {
            if n == i {
                world.wires.set(i, m, crate::wiring::WireColor::Red, true);
            }
            if world
                .tile(n, m)
                .is_some_and(|t| t.active() && td::TILE_SOLID.has(t.kind))
            {
                // `num21 < num4 + num - 4` with `num = 6` (`:35469`).
                if m < ceiling + 2 {
                    if let Some(t) = world.tile_mut(n, m) {
                        t.flags |= FLAG_ACTUATOR;
                    }
                    world.wires.set(n, m, crate::wiring::WireColor::Red, true);
                } else if m < mouth {
                    trap_kill(world, n, m);
                }
            }
        }
    }
    true
}

/// `Skyblock.denySomeGeneration` (`WorldGen.cs:3133-3147`) — the Traps pass
/// gate (`:18659`). Mirrors `surface.rs`'s C1 copy (the shared helper is
/// private to that module).
fn trap_deny_some_generation(opts: &GenOptions) -> bool {
    if !opts.flags.contains(SeedFlags::SKYBLOCK) {
        return false;
    }
    let rebuilding = MicroSeedFlags::WORLD_IS_FROZEN
        | MicroSeedFlags::SURFACE_IS_DESERT
        | MicroSeedFlags::SURFACE_IS_MUSHROOMS
        | MicroSeedFlags::WORLD_IS_INFECTED
        | MicroSeedFlags::HALLOW_ON_THE_SURFACE
        | MicroSeedFlags::NO_INFECTION
        | MicroSeedFlags::EXTRA_FLOATING_ISLANDS
        | MicroSeedFlags::EXTRA_LIQUID;
    if opts.micro_flags.intersects(rebuilding) {
        return false;
    }
    !opts.micro_flags.contains(MicroSeedFlags::EXTRA_LIVING_TREES)
}

/// The `Traps` pass (`GenPassNameID.Traps`, `WorldGen.cs:18657-18787`):
/// `maxTilesX * 0.05` trap attempts (each up to 1150 point rolls), the
/// no-traps-seed TNT barrels, then `maxTilesX * 0.003` sand-trap attempts
/// (each up to 20000 point rolls). Draws only its own stream.
///
/// The `oceanDepths` re-roll (`:18710-18714`) uses `oceanLevel =
/// (worldSurface + rockLayer) / 2 + 40` (`:4447`) and `beachDistance` 380.
/// The sub-vanilla 512 test world collapses the `[200, width-200)` x band —
/// the port skips the trap loop there (the C# `Next` would throw), drawing
/// nothing, like the mining-explosives width guard.
#[allow(clippy::too_many_arguments)]
pub(crate) fn pass_traps(
    world: &mut World,
    layers: &Layers,
    lava_line: i32,
    dungeon_x: i32,
    rng: &mut super::Rng,
    opts: &GenOptions,
    boulder_pets: &mut i32,
) {
    // `:18659`: !denySomeGeneration && !actuallyNoTraps && (!bees || noTraps
    // || remix).
    if trap_deny_some_generation(opts) {
        return;
    }
    if opts.micro_flags.contains(MicroSeedFlags::ACTUALLY_NO_TRAPS) {
        return;
    }
    let seeds = TrapSeeds::from_opts(opts);
    if seeds.bees && !seeds.no_traps && !seeds.remix {
        return;
    }
    let world_surface = layers.world_surface;
    let rock_layer = layers.rock_layer;
    let dungeon_side = if dungeon_x > world.width / 2 { 1 } else { -1 };
    // The trap count (`:18668-18686`). starGame skipped (runtime joke seed).
    let mut count = world.width as f64 * 0.05;
    if seeds.no_traps {
        count = if (seeds.tenth || seeds.bees) && seeds.remix {
            count * 5.0
        } else if !seeds.tenth && !seeds.bees {
            count * 100.0
        } else {
            count * 2.0
        };
    } else if seeds.get_good {
        count *= 1.5;
    }
    let mut attempts_per = 1150i32;
    if seeds.skyblock {
        count /= 5.0;
        attempts_per = 100;
    }
    let ocean_level = (world_surface + rock_layer) / 2.0 + 40.0;
    let beach = super::ocean::ZONE_MARGIN;
    // The x band must be non-empty (sub-vanilla guard).
    let band_ok = 200 < world.width - 200;
    // `:18688`: `for (int i = 0; (double)i < num; i++)` — the loop runs
    // ceil(num) times (the biomes.rs/evil.rs `.ceil()` convention), NOT the
    // truncated count.
    let trap_count = count.ceil() as i32;
    if band_ok {
        for _ in 0..trap_count {
            for _ in 0..attempts_per {
                if seeds.no_traps {
                    let x = rng.range(50, world.width - 50);
                    let mut y = rng.range(50, world.height - 50);
                    if seeds.remix {
                        y = rng.range(50, world.height - 210);
                    }
                    let wall = world.tile(x, y).map(|t| t.wall).unwrap_or(0);
                    if ((y as f64) > world_surface || wall > 0)
                        && place_trap(
                            world, x, y, -1, rng, &seeds, world_surface, rock_layer,
                            lava_line, dungeon_side, boulder_pets,
                        )
                    {
                        break;
                    }
                } else {
                    let mut x = rng.range(200, world.width - 200);
                    let mut y = rng.range(world_surface as i32, world.height - 210);
                    // oceanDepths re-roll (`:18710-18714`).
                    while (y as f64) <= ocean_level
                        && (x < beach || x > world.width - beach)
                    {
                        x = rng.range(200, world.width - 200);
                        y = rng.range(world_surface as i32, world.height - 210);
                    }
                    let wall = world.tile(x, y).map(|t| t.wall).unwrap_or(0);
                    if wall == 0
                        && place_trap(
                            world, x, y, -1, rng, &seeds, world_surface, rock_layer,
                            lava_line, dungeon_side, boulder_pets,
                        )
                    {
                        break;
                    }
                }
            }
        }
    }
    // The no-traps TNT barrels (`:18722-18744`).
    if seeds.no_traps {
        let mut barrel_count = (world.width * 3) as f64;
        if seeds.tenth {
            barrel_count = (world.width / 3) as f64;
        }
        let barrels = barrel_count.ceil() as i32; // `:18733` `(double)k < num`
        if 50 < world.width - 50 {
            for _ in 0..barrels {
                let x = rng.range(50, world.width - 50);
                let y = if seeds.remix {
                    rng.range(
                        world_surface as i32,
                        ((world.height - 350) as f64 + rock_layer) as i32 / 2,
                    )
                } else {
                    rng.range(rock_layer as i32, world.height - 200)
                };
                place_tnt_barrel(world, x, y);
            }
        }
    }
    // The sand traps (`:18745-18766`).
    let mut sand_count = world.width as f64 * 0.003;
    if seeds.no_traps {
        sand_count *= 5.0;
    } else if seeds.get_good {
        sand_count *= 1.5;
    }
    let sand_traps = sand_count.ceil() as i32; // `:18754` `(double)l < num`
    let sand_x_lo = (world.width as f64 * 0.15) as i32;
    let sand_x_hi = (world.width as f64 * 0.85) as i32;
    if sand_x_lo < sand_x_hi {
        for _ in 0..sand_traps {
            for _ in 0..20000 {
                let x = rng.range(sand_x_lo, sand_x_hi);
                let y = rng.range(world_surface as i32 + 20, world.height - 210);
                let wall = world.tile(x, y).map(|t| t.wall).unwrap_or(0);
                if wall == 187 && place_sand_trap(world, x, y, rng) {
                    break;
                }
            }
        }
    }
    // `:18780-18785`: restore boulder solidity + placingTraps=false +
    // tileSolid[379]=true — all pass-local/UI state, no tile writes.
}

// ============================================================================
// W4-g38 C3 — the Piles pass (`GenPassNameID.Piles`, `WorldGen.cs:18788-19459`)
// ============================================================================
//
// Seven attempt ladders in C# order (cave piles 0.06w `:18811`, hell piles
// 0.01w `:18932`, surface 3×2 piles 0.03w `:18996`, house-interior 3×2 piles
// 0.0035w `:19058`, the big small-pile scatter 0.6w `:19130`, surface small
// piles 0.02w `:19291`, house-wall small piles 0.15w `:19371`), each attempt
// budgeted by `GetPileGenerationAttempts` (`:23647-23655`, width/2). The
// 186/187 large piles are 3×2 objects (`PlaceTile` → `Place3x2`); the 185
// small piles ride `PlaceSmallPile` (1×1 at `pileSize` 0, the 2×1 arm at 1).
//
// Pass-local solidity windows (modelled, no tile writes):
// * `:18794-18804` clears `tileSolid` for {379, 229, 190, 196, 189, 717, 718,
//   719, 202, 460, 484}; `:18805-18808` also clears the boulder family on
//   the no-traps seed (`SetBoulderSolidity(false)`, `:11009-11020`).
// * `pile_solid2` is the `SolidTile2(int,int)` read under that window — the
//   floor tests in `Place3x2`/`PlaceSmallPile`/`InvalidTileForPiles...`.
// * `:19446-19457` restores everything (and sets 192/225 solid — they were
//   never cleared; pass-local state, the port's TILE_SOLID is static).
//
// The float ladder counts (`:18810-18811` etc.): `(float)maxTilesX * 0.06f`
// with `for (int i = 0; (float)i < num2; i++)` — a float product and a ceil
// loop. The port computes the product in f32 and walks the same comparison
// (the biomes.rs/evil.rs `.ceil()` convention, here exact in f32).
//
// Sub-vanilla guard: the point bands `[25, w-25)` and the y bands need
// non-empty ranges — the C# `Next` throws on a collapsed band; the port
// skips the ladder after the count (the mining-explosives precedent).

/// The pass-window non-solid kinds (`:18794-18804`).
const PILE_NONSOLID: [u16; 11] = [379, 229, 190, 196, 189, 717, 718, 719, 202, 460, 484];

/// `WorldGen.SolidTile2(int, int)` (`WorldGen.cs:70492-70509`) under the
/// Piles pass's tileSolid window: the cleared kinds (plus the no-traps
/// boulder family) read non-solid; everything else is the static set.
fn pile_solid2(world: &World, x: i32, y: i32, no_traps: bool) -> bool {
    let Some(t) = world.tile(x, y) else {
        return false;
    };
    if PILE_NONSOLID.contains(&t.kind) {
        return false;
    }
    if no_traps && TRAP_BOULDERS.contains(&t.kind) && t.kind != 665 {
        // `SetBoulderSolidity` (`:11011-11019`) touches 138/484/664/711-716
        // — 665 is in `Sets.Boulders` but not in the solidity toggle, and
        // 484 is already in PILE_NONSOLID.
        return false;
    }
    t.active()
        && td::TILE_SOLID.has(t.kind)
        && ((td::SET_PLATFORMS.has(t.kind) && (t.half_brick() || t.top_slope()))
            || t.slope() == 0)
        && !t.half_brick()
        && !t.inactive()
}

/// `WorldGen.InvalidTileForPilesOrSpeleothems` (`WorldGen.cs:38979-38994`):
/// `InWorld(x, y, 2)` and an active `Sets.Boulders` tile below.
fn pile_invalid_below(world: &World, x: i32, y: i32) -> bool {
    if !trap_in_world(world, x, y, 2) {
        return false;
    }
    world
        .tile(x, y)
        .is_some_and(|t| t.active() && TRAP_BOULDERS.contains(&t.kind))
}

/// `WorldGen.Place3x2` (`WorldGen.cs:52066-52196`), the 186/187 path the
/// Piles pass runs — no RNG: the 5-tile bounds guard, the 3×2 clear scan,
/// the per-column default-case floor (`SolidTile2` under the pass window +
/// the 186/187 `InvalidTileForPilesOrSpeleothems` boulder veto `:52110`),
/// then the row-major frame write (`frameX = 54*style + dx*18`,
/// `frameY = dy*18`). The campfire liquid veto is dead for 186/187
/// (`Sets.Campfires`), the 26 altar veto and the 88 chest ledger are other
/// kinds' paths.
fn pile_place_3x2(world: &mut World, x: i32, y: i32, kind: u16, style: i32, no_traps: bool) {
    if x < 5 || x > world.width - 5 || y < 5 || y > world.height - 5 {
        return;
    }
    let mut ok = true;
    for i in (x - 1)..(x + 2) {
        for j in (y - 1)..(y + 1) {
            if world.tile(i, j).is_some_and(|t| t.active()) {
                ok = false;
            }
        }
        if pile_invalid_below(world, i, y + 1) {
            ok = false;
        }
        if !pile_solid2(world, i, y + 1, no_traps) {
            ok = false;
        }
    }
    if !ok {
        return;
    }
    let num2 = (54 * style) as i16;
    // The 3×2 write (`:52171-52194`), C# row-major order.
    for (dx, dy) in [(0i32, 0i32), (1, 0), (2, 0), (0, 1), (1, 1), (2, 1)] {
        if let Some(t) = world.tile_mut(x - 1 + dx, y - 1 + dy) {
            t.flags |= FLAG_ACTIVE;
            t.frame_y = (dy * 18) as i16;
            t.frame_x = num2 + (dx * 18) as i16;
            t.kind = kind;
        }
    }
}

/// `WorldGen.PlaceSmallPile(i, j, pileStyle, pileSize, 185)`
/// (`WorldGen.cs:46815-46873`): the `anyLava` veto (`Tile.cs:410-417`), the
/// 2×1 `pileSize == 1` arm (the scatter ladders roll it; the satellites
/// always pass 0) with its type-185 boulder-below veto (`:46847`), and the
/// 1×1 write (`frameX = 18*style`, `frameY = 18*size`).
fn pile_place_small(world: &mut World, x: i32, y: i32, style: i32, size: i32, no_traps: bool) -> bool {
    let frame_y = (size * 18) as i16;
    let mut frame_x = (style * 18) as i16;
    // `:46835`: anyLava (`Tile.cs:410-417` + `:345-348`) — the port's
    // canonical liquid decode (`liquid_kind()`, the C2 convention).
    let lava = world
        .tile(x, y)
        .is_some_and(|t| t.liquid > 0 && t.liquid_kind() == LiquidKind::Lava);
    if lava {
        return false;
    }
    if size == 1 {
        frame_x = (style * 36) as i16;
        let mut ok = pile_solid2(world, x, y + 1, no_traps)
            && pile_solid2(world, x + 1, y + 1, no_traps)
            && !world.tile(x, y).is_some_and(|t| t.active())
            && !world.tile(x + 1, y).is_some_and(|t| t.active());
        // `:46847`: the type-185 boulder-below veto on both floor columns.
        if ok && (pile_invalid_below(world, x, y + 1) || pile_invalid_below(world, x + 1, y + 1)) {
            ok = false;
        }
        if ok {
            for (i, fx) in [(x, frame_x), (x + 1, frame_x + 18)] {
                if let Some(t) = world.tile_mut(i, y) {
                    t.flags |= FLAG_ACTIVE;
                    t.frame_y = frame_y;
                    t.frame_x = fx;
                    t.kind = 185;
                }
            }
            return true;
        }
    } else if pile_solid2(world, x, y + 1, no_traps)
        && !world.tile(x, y).is_some_and(|t| t.active())
    {
        if let Some(t) = world.tile_mut(x, y) {
            t.flags |= FLAG_ACTIVE;
            t.frame_y = frame_y;
            t.frame_x = frame_x;
            t.kind = 185;
        }
        return true;
    }
    false
}

/// `WorldGen.PlaceTile(i, j, {186,187}, mute: true, forced: false, -1, style)`
/// (`WorldGen.cs:59342-60295`), the slice the Piles pass runs: the
/// empty/non-solid entry (`:59373` — `tileSolid[186/187]` is false, so the
/// entry always admits), the inactive-cell clear (`:59473-59476`), the
/// `Place3x2(i, j, kind, style)` dispatch (`:59994-60002`) and the
/// `result = tile.active()` tail (`:60255-60262`). The `SquareTileFrame`
/// re-frame rides the post-pipeline seam (the d106 convention).
fn pile_place_tile(world: &mut World, x: i32, y: i32, kind: u16, style: i32, no_traps: bool) {
    if !trap_in_world(world, x, y, 0) {
        return;
    }
    let active = world.tile(x, y).is_some_and(|t| t.active());
    if !active {
        let i = world.index(x, y);
        let t = &mut world.tiles[i];
        t.kind = 0;
        t.flags &= !FLAG_ACTIVE;
        t.frame_x = 0;
        t.frame_y = 0;
        t.set_paint(0);
        t.flags &= !crate::tile::FLAG_FULLBRIGHT;
        t.set_slope(0);
        t.flags &= !FLAG_HALF_BRICK;
        world.set_invisible_block(x, y, false);
    }
    pile_place_3x2(world, x, y, kind, style, no_traps);
}

/// `Tile.anyShimmer()` (`Tile.cs:428-435` + `:379-382`) — the port's
/// canonical liquid decode (`liquid_kind()`, the C2 geyser convention).
fn pile_any_shimmer(world: &World, x: i32, y: i32) -> bool {
    world
        .tile(x, y)
        .is_some_and(|t| t.liquid > 0 && t.liquid_kind() == LiquidKind::Shimmer)
}

/// The fall-to-ground scan every ladder runs (`:18829-18831` etc.): walk
/// down while the cell below is inactive, stopping at `maxTilesY - 5`.
fn pile_fall(world: &World, x: i32, mut y: i32) -> i32 {
    while !world.tile(x, y + 1).is_some_and(|t| t.active()) && y < world.height - 5 {
        y += 1;
    }
    y
}

/// The scattered small-pile tail the ladders share (`:18910-18925` etc.):
/// `Next(1, 5)` satellites, each an (x ± 10, y − Next(5)) point fallen to
/// ground, then `PlaceSmallPile(_, _, Next(12, 36), 0, 185)`.
fn pile_satellites(world: &mut World, x: i32, y: i32, rng: &mut super::Rng, no_traps: bool) {
    let n = rng.range(1, 5);
    for _ in 0..n {
        let sx = x + rng.range(-10, 11);
        let sy = y - rng.range(0, 5);
        if !world.tile(sx, sy).is_some_and(|t| t.active()) {
            let sy = pile_fall(world, sx, sy);
            pile_place_small(world, sx, sy, rng.range(12, 36), 0, no_traps);
        }
    }
}

/// The Piles pass (`GenPassNameID.Piles`, `WorldGen.cs:18788-19459`).
/// Draws, in C# order on the pass's own stream, the seven ladders below;
/// the skyblock budget divide (`GetPileGenerationAttempts` `/10`) rides the
/// attempt counter. The `denySomeGeneration` gate (`:18790`) skips the
/// whole pass without draws.
pub(crate) fn pass_piles(world: &mut World, layers: &Layers, rng: &mut super::Rng, opts: &GenOptions) {
    if trap_deny_some_generation(opts) {
        return;
    }
    let no_traps = opts.flags.contains(SeedFlags::NO_TRAPS);
    let skyblock = opts.flags.contains(SeedFlags::SKYBLOCK);
    let world_surface = layers.world_surface;
    // oceanLevel (`WorldGen.cs:10963-10974` reads `WorldGen.oceanLevel` =
    // `(worldSurface + rockLayer)/2 + 40`, the C2/evil.rs convention).
    let ocean_level = (world_surface + layers.rock_layer) / 2.0 + 40.0;
    let beach = super::ocean::ZONE_MARGIN;
    // `GetPileGenerationAttempts` (`:23647-23655`).
    let attempts = if skyblock { world.width / 2 / 10 } else { world.width / 2 };
    // The float ladder count (`:18810`): the C# float product + the
    // `(float)i < num2` ceil loop, exact in f32. Captures the width only —
    // the loop bodies mutate the world.
    let width = world.width;
    let ladder = |coeff: f32| -> i32 {
        let num2 = width as f32 * coeff;
        let mut i = 0i32;
        while (i as f32) < num2 {
            i += 1;
        }
        i
    };
    let band_ok = 25 < world.width - 25;

    // ---- Ladder 1: the cave piles (`:18809-18930`), 0.06w -----------------
    if band_ok && (world_surface as i32) < world.height - 300 {
        for _ in 0..ladder(0.06) {
            let mut tries = attempts;
            let mut placed = false;
            while !placed && tries > 0 {
                tries -= 1;
                let mut x = rng.range(25, world.width - 25);
                let mut y = rng.range(world_surface as i32, world.height - 300);
                while (y as f64) <= ocean_level && (x < beach || x > world.width - beach) {
                    x = rng.range(25, world.width - 25);
                    y = rng.range(world_surface as i32, world.height - 300);
                }
                if world.tile(x, y).is_some_and(|t| t.active()) {
                    continue;
                }
                let mut kind = 186u16;
                y = pile_fall(world, x, y);
                let wall = world.tile(x, y).map(|t| t.wall).unwrap_or(0);
                let below = world.tile(x, y + 1).copied().unwrap_or_default();
                if !below.active() {
                    continue;
                }
                // The style ladder (`:18836-18895`), C# if-chain order.
                let mut style = rng.range(0, 22);
                if (16..=22).contains(&style) {
                    style = rng.range(0, 22);
                }
                if (below.kind == 0 || below.kind == 1 || td::TILE_MOSS.has(below.kind))
                    && rng.range(0, 5) == 0
                {
                    style = rng.range(23, 29);
                    kind = 187;
                }
                if y > world.height - 300
                    || TRAP_DUNGEON_WALLS.contains(&wall)
                    || matches!(below.kind, 30 | 19 | 25 | 203)
                {
                    style = rng.range(0, 7);
                    kind = 186;
                }
                if matches!(below.kind, 147 | 161 | 162) {
                    style = rng.range(26, 32);
                    kind = 186;
                }
                if below.kind == 60 {
                    kind = 187;
                    style = rng.range(0, 6);
                }
                if matches!(below.kind, 57 | 58) && rng.range(0, 3) < 2 {
                    kind = 187;
                    style = rng.range(6, 9);
                }
                if below.kind == 226 {
                    kind = 187;
                    style = rng.range(18, 23);
                }
                if below.kind == 70 {
                    style = rng.range(32, 35);
                    kind = 186;
                }
                if matches!(below.kind, 396 | 397 | 404) {
                    style = rng.range(29, 35);
                    kind = 187;
                }
                if below.kind == 368 {
                    style = rng.range(35, 41);
                    kind = 187;
                }
                if below.kind == 367 {
                    style = rng.range(41, 47);
                    kind = 187;
                }
                if kind == 186 && (7..=15).contains(&style) && rng.range(0, 75) == 0 {
                    kind = 187;
                    style = 17;
                }
                // `:18896-18909`: the dungeon-wall 2/3 skip, the shimmer
                // veto, then the placement + success test.
                if TRAP_DUNGEON_WALLS.contains(&wall) && rng.range(0, 3) != 0 {
                    placed = true;
                } else {
                    if !pile_any_shimmer(world, x, y) {
                        pile_place_tile(world, x, y, kind, style, no_traps);
                    }
                    if world.tile(x, y).is_some_and(|t| matches!(t.kind, 186 | 187)) {
                        placed = true;
                    }
                    if placed && kind == 186 && style <= 7 {
                        pile_satellites(world, x, y, rng, no_traps);
                    }
                }
            }
        }
    }

    // ---- Ladder 2: the hell piles (`:18931-18994`), 0.01w -----------------
    if band_ok && world.height - 300 < world.height - 10 {
        for _ in 0..ladder(0.01) {
            let mut tries = attempts;
            let mut placed = false;
            while !placed && tries > 0 {
                tries -= 1;
                let x = rng.range(25, world.width - 25);
                let mut y = rng.range(world.height - 300, world.height - 10);
                if world.tile(x, y).is_some_and(|t| t.active()) {
                    continue;
                }
                let mut kind = 186u16;
                y = pile_fall(world, x, y);
                let wall = world.tile(x, y).map(|t| t.wall).unwrap_or(0);
                let below = world.tile(x, y + 1).copied().unwrap_or_default();
                if !below.active() {
                    continue;
                }
                // The style ladder (`:18952-18969`).
                let mut style = rng.range(0, 22);
                if (16..=22).contains(&style) {
                    style = rng.range(0, 22);
                }
                if y > world.height - 300
                    || TRAP_DUNGEON_WALLS.contains(&wall)
                    || matches!(below.kind, 30 | 19)
                {
                    style = rng.range(0, 7);
                }
                if matches!(below.kind, 57 | 58) && rng.range(0, 3) < 2 {
                    kind = 187;
                    style = rng.range(6, 9);
                }
                if matches!(below.kind, 147 | 161 | 162) {
                    style = rng.range(26, 32);
                }
                pile_place_tile(world, x, y, kind, style, no_traps);
                if world.tile(x, y).is_some_and(|t| matches!(t.kind, 186 | 187)) {
                    placed = true;
                }
                if placed && kind == 186 && style <= 7 {
                    pile_satellites(world, x, y, rng, no_traps);
                }
            }
        }
    }

    // ---- Ladder 3: the surface 3×2 piles (`:18995-19056`), 0.03w ----------
    if band_ok && 10 < world_surface as i32 {
        for _ in 0..ladder(0.03) {
            let mut tries = attempts;
            let mut placed = false;
            while !placed && tries > 0 {
                tries -= 1;
                let mut kind = 186u16;
                let mut x = rng.range(25, world.width - 25);
                let mut y = rng.range(10, world_surface as i32);
                while (y as f64) <= ocean_level && (x < beach || x > world.width - beach) {
                    x = rng.range(25, world.width - 25);
                    y = rng.range(10, world_surface as i32);
                }
                if world.tile(x, y).is_some_and(|t| t.active()) {
                    continue;
                }
                y = pile_fall(world, x, y);
                let wall = world.tile(x, y).map(|t| t.wall).unwrap_or(0);
                let below = world.tile(x, y + 1).copied().unwrap_or_default();
                if !below.active() {
                    continue;
                }
                // The style ladder (`:19021-19044`).
                let mut style = rng.range(7, 13);
                if y > world.height - 300
                    || TRAP_DUNGEON_WALLS.contains(&wall)
                    || matches!(below.kind, 30 | 19 | 25 | 204 | 234 | 112)
                    || TRAP_DUNGEON_TILES.contains(&below.kind)
                {
                    style = -1;
                }
                if matches!(below.kind, 147 | 161 | 162) {
                    style = rng.range(26, 32);
                }
                if below.kind == 53 {
                    kind = 187;
                    style = rng.range(52, 55);
                }
                let grass_side = |w: &World, sx: i32| {
                    w.tile(sx, y + 1).is_some_and(|t| t.active() && t.kind == 2)
                };
                if below.kind == 2 || grass_side(world, x - 1) || grass_side(world, x + 1) {
                    kind = 187;
                    style = rng.range(14, 17);
                }
                if matches!(below.kind, 151 | 274) {
                    kind = 186;
                    style = rng.range(0, 7);
                }
                if style >= 0 {
                    pile_place_tile(world, x, y, kind, style, no_traps);
                }
                // `:19049-19052`: `type == num19` — no active() test; the
                // seat's stale kind can match and end the attempt.
                if world.tile(x, y).is_some_and(|t| t.kind == kind) {
                    placed = true;
                }
            }
        }
    }

    // ---- Ladder 4: the house-interior piles (`:19057-19128`), 0.0035w -----
    if band_ok && 10 < world_surface as i32 {
        for _ in 0..ladder(0.0035) {
            let mut tries = attempts;
            let mut placed = false;
            while !placed && tries > 0 {
                tries -= 1;
                let x = rng.range(25, world.width - 25);
                let mut y = rng.range(10, world_surface as i32);
                let cell = world.tile(x, y).copied().unwrap_or_default();
                if cell.active() || cell.wall == 0 {
                    continue;
                }
                let mut kind = 186u16;
                y = pile_fall(world, x, y);
                let wall = world.tile(x, y).map(|t| t.wall).unwrap_or(0);
                let below = world.tile(x, y + 1).copied().unwrap_or_default();
                if !below.active() {
                    continue;
                }
                // The style ladder (`:19078-19100`).
                let mut style = rng.range(7, 13);
                if y > world.height - 300
                    || TRAP_DUNGEON_WALLS.contains(&wall)
                    || matches!(below.kind, 30 | 19)
                    || TRAP_DUNGEON_TILES.contains(&below.kind)
                {
                    style = -1;
                }
                if below.kind == 25 {
                    style = rng.range(0, 7);
                }
                if matches!(below.kind, 147 | 161 | 162) {
                    style = rng.range(26, 32);
                }
                let grass_side = |w: &World, sx: i32| {
                    w.tile(sx, y + 1).is_some_and(|t| t.active() && t.kind == 2)
                };
                if below.kind == 2 || grass_side(world, x - 1) || grass_side(world, x + 1) {
                    kind = 187;
                    style = rng.range(14, 17);
                }
                if matches!(below.kind, 151 | 274) {
                    kind = 186;
                    style = rng.range(0, 7);
                }
                if style >= 0 {
                    pile_place_tile(world, x, y, kind, style, no_traps);
                }
                if world.tile(x, y).is_some_and(|t| t.kind == kind) {
                    placed = true;
                }
                if placed && style <= 7 {
                    pile_satellites(world, x, y, rng, no_traps);
                }
            }
        }
    }

    // ---- Ladder 5: the big small-pile scatter (`:19129-19289`), 0.6w ------
    if band_ok && (world_surface as i32) < world.height - 20 {
        for _ in 0..ladder(0.6) {
            let mut tries = attempts;
            let mut placed = false;
            while !placed && tries > 0 {
                tries -= 1;
                let mut x = rng.range(25, world.width - 25);
                let mut y = rng.range(world_surface as i32, world.height - 20);
                // `:19140-19144`: the wall-87 (spider) half re-roll.
                if world.tile(x, y).is_some_and(|t| t.wall == 87) && rng.range(0, 2) == 0 {
                    x = rng.range(25, world.width - 25);
                    y = rng.range(world_surface as i32, world.height - 20);
                }
                while (y as f64) <= ocean_level && (x < beach || x > world.width - beach) {
                    x = rng.range(25, world.width - 25);
                    y = rng.range(world_surface as i32, world.height - 20);
                }
                if world.tile(x, y).is_some_and(|t| t.active()) {
                    continue;
                }
                y = pile_fall(world, x, y);
                let cell = world.tile(x, y).copied().unwrap_or_default();
                let below = world.tile(x, y + 1).copied().unwrap_or_default();
                if !below.active() {
                    continue;
                }
                // The size/style ladder (`:19159-19257`).
                let size = rng.range(0, 2);
                let mut style = rng.range(0, 36);
                if (28..=35).contains(&style) {
                    style = rng.range(0, 36);
                }
                if size == 1 {
                    style = rng.range(0, 25);
                    if (16..=24).contains(&style) {
                        style = rng.range(0, 25);
                    }
                }
                if y > world.height - 300 {
                    if size == 0 {
                        style = rng.range(12, 28);
                    }
                    if size == 1 {
                        style = rng.range(6, 16);
                    }
                }
                if TRAP_DUNGEON_WALLS.contains(&cell.wall)
                    || cell.wall == 87
                    || matches!(below.kind, 30 | 19 | 25 | 203)
                {
                    if size == 0 && style < 12 {
                        style += 12;
                    }
                    if size == 1 && style < 6 {
                        style += 6;
                    }
                    if size == 1 && style >= 17 {
                        style -= 10;
                    }
                }
                if matches!(below.kind, 147 | 161 | 162) {
                    if size == 0 && style < 12 {
                        style += 36;
                    }
                    if size == 1 && style >= 20 {
                        style += 6;
                    }
                    if size == 1 && style < 6 {
                        style += 25;
                    }
                }
                if cell.liquid == 0 && matches!(below.kind, 53 | 397 | 396) {
                    if size == 0 {
                        style = rng.range(73, 78);
                    }
                    if size == 1 {
                        style = rng.range(62, 65);
                    }
                }
                if matches!(below.kind, 151 | 274) {
                    if size == 0 {
                        style = rng.range(12, 28);
                    }
                    if size == 1 {
                        style = rng.range(12, 19);
                    }
                }
                if below.kind == 368 {
                    if size == 0 {
                        style = rng.range(60, 66);
                    }
                    if size == 1 {
                        style = rng.range(47, 53);
                    }
                }
                if below.kind == 367 {
                    if size == 0 {
                        style = rng.range(66, 72);
                    }
                    if size == 1 {
                        style = rng.range(53, 59);
                    }
                }
                // `:19258-19269`: the dungeon-tile skip, the dungeon-wall
                // 2/3 skip, the shimmer veto, then PlaceSmallPile.
                if TRAP_DUNGEON_TILES.contains(&below.kind) {
                    placed = false;
                } else if TRAP_DUNGEON_WALLS.contains(&cell.wall) && rng.range(0, 3) != 0 {
                    placed = true;
                } else if !pile_any_shimmer(world, x, y) {
                    placed = pile_place_small(world, x, y, style, size, no_traps);
                }
                if placed && size == 1 && (6..=15).contains(&style) {
                    pile_satellites(world, x, y, rng, no_traps);
                }
            }
        }
    }

    // ---- Ladder 6: the surface small piles (`:19290-19369`), 0.02w --------
    if band_ok && 15 < world_surface as i32 {
        for _ in 0..ladder(0.02) {
            let mut tries = attempts;
            let mut placed = false;
            while !placed && tries > 0 {
                tries -= 1;
                let mut x = rng.range(25, world.width - 25);
                let mut y = rng.range(15, world_surface as i32);
                while (y as f64) <= ocean_level && (x < beach || x > world.width - beach) {
                    x = rng.range(25, world.width - 25);
                    y = rng.range(15, world_surface as i32);
                }
                if world.tile(x, y).is_some_and(|t| t.active()) {
                    continue;
                }
                y = pile_fall(world, x, y);
                let cell = world.tile(x, y).copied().unwrap_or_default();
                let below = world.tile(x, y + 1).copied().unwrap_or_default();
                if !below.active() {
                    continue;
                }
                // The size/style ladder (`:19315-19361`).
                let size = rng.range(0, 2);
                let mut style = rng.range(0, 11);
                if size == 1 {
                    style = rng.range(0, 5);
                }
                if matches!(below.kind, 147 | 161 | 162) {
                    if size == 0 && style < 12 {
                        style += 36;
                    }
                    if size == 1 && style >= 20 {
                        style += 6;
                    }
                    if size == 1 && style < 6 {
                        style += 25;
                    }
                }
                if cell.liquid == 0 && matches!(below.kind, 53 | 397 | 396) {
                    if size == 0 {
                        style = rng.range(73, 77);
                    }
                    if size == 1 {
                        style = rng.range(62, 65);
                    }
                }
                if below.kind == 2 && size == 1 {
                    style = rng.range(38, 41);
                }
                if matches!(below.kind, 151 | 274) {
                    if size == 0 {
                        style = rng.range(12, 28);
                    }
                    if size == 1 {
                        style = rng.range(12, 19);
                    }
                }
                // `:19362`: the long veto chain — the dungeon wall, the
                // wood/ebonstone/shadewood/pearlwood family, the ore bars,
                // the cracked bricks, the dungeon tiles.
                if !TRAP_DUNGEON_WALLS.contains(&cell.wall)
                    && !matches!(
                        below.kind,
                        30 | 19 | 41 | 43 | 44 | 481 | 482 | 483 | 45 | 46 | 47 | 175 | 176
                            | 177 | 25 | 203
                    )
                    && !TRAP_DUNGEON_TILES.contains(&below.kind)
                {
                    placed = pile_place_small(world, x, y, style, size, no_traps);
                }
            }
        }
    }

    // ---- Ladder 7: the house-wall small piles (`:19370-19444`), 0.15w -----
    if band_ok && 15 < world_surface as i32 {
        for _ in 0..ladder(0.15) {
            let mut tries = attempts;
            let mut placed = false;
            while !placed && tries > 0 {
                tries -= 1;
                let x = rng.range(25, world.width - 25);
                let mut y = rng.range(15, world_surface as i32);
                let cell0 = world.tile(x, y).copied().unwrap_or_default();
                if cell0.active() || !matches!(cell0.wall, 2 | 40) {
                    continue;
                }
                y = pile_fall(world, x, y);
                let cell = world.tile(x, y).copied().unwrap_or_default();
                let below = world.tile(x, y + 1).copied().unwrap_or_default();
                if !below.active() {
                    continue;
                }
                // The size/style ladder (`:19390-19436`) — ladder 6's with
                // the same constants.
                let size = rng.range(0, 2);
                let mut style = rng.range(0, 11);
                if size == 1 {
                    style = rng.range(0, 5);
                }
                if matches!(below.kind, 147 | 161 | 162) {
                    if size == 0 && style < 12 {
                        style += 36;
                    }
                    if size == 1 && style >= 20 {
                        style += 6;
                    }
                    if size == 1 && style < 6 {
                        style += 25;
                    }
                }
                if cell.liquid == 0 && matches!(below.kind, 53 | 397 | 396) {
                    if size == 0 {
                        style = rng.range(73, 77);
                    }
                    if size == 1 {
                        style = rng.range(62, 65);
                    }
                }
                if below.kind == 2 && size == 1 {
                    style = rng.range(38, 41);
                }
                if matches!(below.kind, 151 | 274) {
                    if size == 0 {
                        style = rng.range(12, 28);
                    }
                    if size == 1 {
                        style = rng.range(12, 19);
                    }
                }
                // `:19437`: ladder 6's veto chain plus the full-sand-water
                // head `(liquid != 255 || below != 53 || wall != 0)`.
                if (cell.liquid != u8::MAX || below.kind != 53 || cell.wall != 0)
                    && !TRAP_DUNGEON_WALLS.contains(&cell.wall)
                    && !matches!(
                        below.kind,
                        30 | 19 | 41 | 43 | 44 | 481 | 482 | 483 | 45 | 46 | 47 | 175 | 176
                            | 177 | 25 | 203
                    )
                    && !TRAP_DUNGEON_TILES.contains(&below.kind)
                {
                    placed = pile_place_small(world, x, y, style, size, no_traps);
                }
            }
        }
    }
    // `:19446-19457`: the solidity restores (190/192/196/189/717-719/202/
    // 225/460/379 + SetBoulderSolidity(true)) — pass-local state, no tile
    // writes; the port's TILE_SOLID is static.
}

#[cfg(test)]
mod pile_tests {
    use super::*;
    use crate::tiledata::TILE_COUNT;

    /// A flat stone floor at row 40 with air above — the pile seat.
    fn floor_world() -> World {
        let mut w = World::new("piles".into(), 120, 120);
        w.frame_important = vec![false; TILE_COUNT];
        for x in 0..120 {
            for y in 40..120 {
                let i = w.index(x, y);
                w.tiles[i] = Tile {
                    kind: 1,
                    flags: FLAG_ACTIVE,
                    ..Default::default()
                };
            }
        }
        w
    }

    /// W4-g38 C3: `pile_solid2` honours the pass window — the cleared kinds
    /// (379/229/190/…) read non-solid, and on the no-traps seed the boulder
    /// family goes too (but not 665 — `SetBoulderSolidity` never touches it,
    /// and 665 is not solid in the static table anyway).
    #[test]
    fn w4_g38_c3_pile_solid2_window() {
        let mut w = floor_world();
        assert!(pile_solid2(&w, 60, 40, false));
        let i = w.index(60, 40);
        // A cleared kind: non-solid in both seed arms.
        w.tiles[i].kind = 379;
        assert!(!pile_solid2(&w, 60, 40, false));
        assert!(!pile_solid2(&w, 60, 40, true));
        // A boulder: solid on the classic seed, non-solid on no-traps.
        w.tiles[i].kind = 138;
        assert!(pile_solid2(&w, 60, 40, false));
        assert!(!pile_solid2(&w, 60, 40, true));
    }

    /// W4-g38 C3: `pile_place_3x2` writes the 3×2 frames
    /// (`frameX = 54*style + dx*18`) and vetoes a boulder floor
    /// (`InvalidTileForPilesOrSpeleothems`, `:52110`).
    #[test]
    fn w4_g38_c3_pile_place_3x2_frames_and_boulder_veto() {
        let mut w = floor_world();
        pile_place_3x2(&mut w, 60, 39, 186, 3, false);
        for dx in 0..3 {
            for dy in 0..2 {
                let t = w.tile(59 + dx, 38 + dy).unwrap();
                assert!(t.active());
                assert_eq!(t.kind, 186);
                assert_eq!(t.frame_x, (54 * 3 + dx * 18) as i16);
                assert_eq!(t.frame_y, (dy * 18) as i16);
            }
        }
        // A boulder under one column vetoes the whole write.
        let mut w2 = floor_world();
        let i = w2.index(60, 40);
        w2.tiles[i].kind = 138;
        pile_place_3x2(&mut w2, 60, 39, 186, 3, false);
        assert!(!w2.tile(59, 38).unwrap().active());
    }

    /// W4-g38 C3: `pile_place_small` — the 1×1 arm writes
    /// `frameX = 18*style` / `frameY = 18*size`, the lava veto refuses, and
    /// the 2×1 arm needs both floor columns.
    #[test]
    fn w4_g38_c3_pile_place_small_arms() {
        let mut w = floor_world();
        assert!(pile_place_small(&mut w, 60, 39, 12, 0, false));
        let t = w.tile(60, 39).unwrap();
        assert!(t.active());
        assert_eq!(t.kind, 185);
        assert_eq!(t.frame_x, 12 * 18);
        assert_eq!(t.frame_y, 0);
        // The 2×1 arm at (62, 39): both columns at row 40 are stone.
        assert!(pile_place_small(&mut w, 62, 39, 5, 1, false));
        assert_eq!(w.tile(62, 39).unwrap().frame_x, 5 * 36);
        assert_eq!(w.tile(63, 39).unwrap().frame_x, 5 * 36 + 18);
        // Lava refuses.
        let i = w.index(66, 39);
        w.tiles[i].liquid = 200;
        w.tiles[i].set_liquid_kind(LiquidKind::Lava);
        assert!(!pile_place_small(&mut w, 66, 39, 12, 0, false));
        assert!(!w.tile(66, 39).unwrap().active());
    }

    /// W4-g38 C3: `pile_fall` walks to the first active cell below, capped
    /// at `height - 5`.
    #[test]
    fn w4_g38_c3_pile_fall_walks_to_the_floor() {
        let w = floor_world();
        assert_eq!(pile_fall(&w, 60, 10), 39);
        // An already-seated point does not move.
        assert_eq!(pile_fall(&w, 60, 39), 39);
        // The bottom cap: an all-air column stops at height-5.
        let mut w2 = World::new("air".into(), 120, 120);
        w2.frame_important = vec![false; TILE_COUNT];
        assert_eq!(pile_fall(&w2, 60, 10), 115);
    }

    /// W4-g38 C3: the `denySomeGeneration` gate skips the whole pass without
    /// draws (the skyblock-without-rebuild arm).
    #[test]
    fn w4_g38_c3_pass_gate_skips_without_draws() {
        let layers = Layers {
            avg_surface: 40.0,
            world_surface: 40.0,
            rock_layer: 60.0,
            hell_top: 100,
            surface_low: 40.0,
            surface_high: 40.0,
            rock_low: 56.0,
            rock_high: 64.0,
        };
        let mut opts = GenOptions::default();
        opts.flags.insert(SeedFlags::SKYBLOCK);
        let mut w = floor_world();
        let before = w.tiles.clone();
        let mut rng = super::super::Rng(9);
        pass_piles(&mut w, &layers, &mut rng, &opts);
        assert_eq!(w.tiles, before);
        assert_eq!(rng.next_u32(), super::super::Rng(9).next_u32(), "no draw");
    }
}

#[cfg(test)]
mod trap_tests {
    use super::*;
    use crate::tiledata::TILE_COUNT;

    /// A flat stone floor at row 40 with air above — the cave-floor seat the
    /// trap walks find.
    fn floor_world() -> World {
        let mut w = World::new("traps".into(), 120, 120);
        w.frame_important = vec![false; TILE_COUNT];
        for x in 0..120 {
            for y in 40..120 {
                let i = w.index(x, y);
                w.tiles[i] = Tile {
                    kind: 1,
                    flags: FLAG_ACTIVE,
                    ..Default::default()
                };
            }
        }
        w
    }

    fn seeds_classic() -> TrapSeeds {
        TrapSeeds {
            no_traps: false,
            get_good: false,
            bees: false,
            remix: false,
            drunk: false,
            tenth: false,
            dont_starve: false,
            skyblock: false,
            poo: false,
        }
    }

    /// W4-g38 C2: `trap_solid` honours the pass-local bubble/boulder
    /// solidity windows (`Main.tileSolid[379]=false`, `SetBoulderSolidity`).
    #[test]
    fn w4_g38_c2_trap_solid_bubble_and_boulder_windows() {
        let mut w = floor_world();
        // Stone is solid in both windows.
        assert!(trap_solid(&w, 60, 40, true));
        assert!(trap_solid(&w, 60, 40, false));
        // A bubble 379 is never solid during the pass.
        let i = w.index(60, 40);
        w.tiles[i].kind = 379;
        assert!(!trap_solid(&w, 60, 40, true));
        assert!(!trap_solid(&w, 60, 40, false));
        // A boulder 138 is solid only while boulder_solid is true.
        w.tiles[i].kind = 138;
        assert!(trap_solid(&w, 60, 40, true));
        assert!(!trap_solid(&w, 60, 40, false));
    }

    /// W4-g38 C2: `trap_place_1x1` — the pressure plate needs a solid floor
    /// and an empty cell, and writes `frameY = 18 * style`.
    #[test]
    fn w4_g38_c2_place_1x1_plate_frames_and_floor() {
        let mut w = floor_world();
        trap_place_1x1(&mut w, 60, 39, 135, 7);
        let t = w.tile(60, 39).unwrap();
        assert!(t.active());
        assert_eq!(t.kind, 135);
        assert_eq!(t.frame_y, 126);
        // An occupied cell is refused.
        let before = w.tiles.clone();
        trap_place_1x1(&mut w, 60, 39, 135, 2);
        assert_eq!(w.tiles, before);
        // No floor → refused.
        trap_place_1x1(&mut w, 60, 20, 135, 2);
        assert!(!w.tile(60, 20).unwrap().active());
    }

    /// W4-g38 C2: `trap_place_2x2` — the boulder body writes the 2×2 frames
    /// at `(x-1, y-1)` and needs both floor columns solid.
    #[test]
    fn w4_g38_c2_place_2x2_boulder_frames() {
        let mut w = floor_world();
        trap_place_2x2(&mut w, 60, 39, 138);
        for (dx, dy) in [(0i32, 0i32), (1, 0), (0, 1), (1, 1)] {
            let t = w.tile(59 + dx, 38 + dy).unwrap();
            assert!(t.active());
            assert_eq!(t.kind, 138);
            assert_eq!(t.frame_x, (dx * 18) as i16);
            assert_eq!(t.frame_y, (dy * 18) as i16);
        }
        // A missing floor column vetoes the whole write.
        let mut w2 = floor_world();
        let i = w2.index(60, 40);
        w2.tiles[i] = Tile::default();
        trap_place_2x2(&mut w2, 60, 39, 138);
        assert!(!w2.tile(59, 38).unwrap().active());
    }

    /// W4-g38 C2: the dart trap places a plate + dart + red wire on a flat
    /// floor with solid walls to both sides.
    #[test]
    fn w4_g38_c2_dart_trap_places_plate_dart_and_wire() {
        let mut w = floor_world();
        // Build two stone walls at x=40 and x=80, rows 30..40, so the dart
        // walk finds a housing 5..50 tiles from the plate at x=60.
        for y in 30..40 {
            for x in [40, 80] {
                let i = w.index(x, y);
                w.tiles[i] = Tile { kind: 1, flags: FLAG_ACTIVE, ..Default::default() };
            }
        }
        let seeds = seeds_classic();
        let mut rng = super::super::Rng(11);
        // The plate seat is row 39 (air over the row-40 floor).
        let ok = place_trap_dart(&mut w, 60, 39, &mut rng, &seeds, true);
        assert!(ok, "the dart trap places");
        // The plate at (60, 39).
        assert!(w.tile(60, 39).is_some_and(|t| t.active() && t.kind == 135));
        // A dart 137 exists in one of the walls, wired to the plate.
        let dart = w.tiles.iter().any(|t| t.active() && t.kind == 137);
        assert!(dart, "a dart trap placed");
        assert!(w.wires.get(60, 39, crate::wiring::WireColor::Red));
    }

    /// W4-g38 C2: the pass gates — actuallyNoTraps and the bees gate skip
    /// without drawing; the sub-vanilla width guard skips the trap loop.
    #[test]
    fn w4_g38_c2_pass_gates_skip_without_draws() {
        let layers = Layers {
            avg_surface: 40.0,
            world_surface: 40.0,
            rock_layer: 60.0,
            hell_top: 100,
            surface_low: 40.0,
            surface_high: 40.0,
            rock_low: 56.0,
            rock_high: 64.0,
        };
        // actuallyNoTraps skips the whole pass (no draws).
        let mut opts = GenOptions::default();
        opts.micro_flags.insert(MicroSeedFlags::ACTUALLY_NO_TRAPS);
        let mut w = floor_world();
        let before = w.tiles.clone();
        let mut pets = 0;
        let mut rng = super::super::Rng(5);
        pass_traps(&mut w, &layers, 80, 100, &mut rng, &opts, &mut pets);
        assert_eq!(w.tiles, before);
        assert_eq!(rng.next_u32(), super::super::Rng(5).next_u32(), "no draw");
        // bees && !noTraps && !remix skips too.
        let mut bees = GenOptions::default();
        bees.flags.insert(SeedFlags::NOT_THE_BEES);
        let mut rng2 = super::super::Rng(5);
        pass_traps(&mut w, &layers, 80, 100, &mut rng2, &bees, &mut pets);
        assert_eq!(w.tiles, before);
        assert_eq!(rng2.next_u32(), super::super::Rng(5).next_u32(), "no draw");
    }
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
            // The interior carries the hut wall (23 for 119). A torch may
            // have landed on one interior cell (it keeps the wall), so the
            // wall — not the active bit — is the stable assertion.
            let inner = w.tile(x, y).unwrap();
            assert_eq!(inner.wall, 23, "the interior wall at ({x},{y})");
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

    /// A sand world: air above row 100, solid sand below, the surface line
    /// at 100 — the pyramid's sand-column gate and hull have ground to bite.
    fn sand_world(w: i32, h: i32) -> World {
        let mut world = World::new("pyr".into(), w, h);
        world.world_surface = 100.0;
        world.rock_layer = 300.0;
        for x in 0..w {
            for y in 100..h {
                let i = world.index(x, y);
                world.tiles[i].kind = super::super::TILE_SAND;
                world.tiles[i].flags |= FLAG_ACTIVE;
            }
        }
        world
    }

    /// W4-c33: the anchor loop's gates — a slot inside the 300 margins on
    /// sand builds the 151 hull (never the retired 480); a slot in the
    /// dungeon's 15 % band builds nothing.
    #[test]
    fn the_pyramid_anchor_loop_gates_and_builds_the_151_hull() {
        let mut world = sand_world(1200, 600);
        // The dungeon sits right (x 900): its 15 % exclusion band is
        // x > 900 - 180 = 720. The anchor at 500 passes; 800 fails.
        let mut rng = super::super::Rng(0x9E37);
        let placed = pass_pyramids(
            &mut world,
            &[500, 800],
            &[100, 100],
            2,
            900,
            false,
            &mut rng,
            None,
        );
        assert_eq!(placed, 1, "only the clear anchor builds");
        let brick = world
            .tiles
            .iter()
            .filter(|t| t.active() && t.kind == TILE_PYRAMID_BRICK)
            .count();
        assert!(brick > 500, "the 151 hull ({brick})");
        assert!(
            !world.tiles.iter().any(|t| t.active() && t.kind == 480),
            "the retired 480 hull never appears"
        );
        // The wall-34 sweep ran inside the hull.
        let walls = world
            .tiles
            .iter()
            .filter(|t| t.wall == WALL_PYRAMID_BRICK)
            .count();
        assert!(walls > 100, "the wall-34 interiors ({walls})");
    }

    /// W4-c33: the 220 spacing — two anchors closer than 220 build one
    /// pyramid; the drunk seed's 110 spacing admits the pair at 150 apart.
    #[test]
    fn the_pyramid_spacing_gate_halves_under_drunk() {
        let mut world = sand_world(1200, 600);
        let mut rng = super::super::Rng(7);
        let placed = pass_pyramids(
            &mut world,
            &[400, 550],
            &[100, 100],
            2,
            1100,
            false,
            &mut rng,
            None,
        );
        assert_eq!(placed, 1, "150 < 220 — the second slot skips");
        let mut world = sand_world(1200, 600);
        let mut rng = super::super::Rng(7);
        let placed = pass_pyramids(
            &mut world,
            &[400, 550],
            &[100, 100],
            2,
            1100,
            true,
            &mut rng,
            None,
        );
        assert_eq!(placed, 2, "drunk: 150 >= 110 — both slots build");
    }

    /// W4-c33: the chamber furniture — the style-1 chest (the Pharaoh
    /// family), the 185 piles, the 91 banners and the 28 pots all land
    /// inside a built pyramid.
    #[test]
    fn the_pyramid_chamber_carries_its_furniture() {
        let mut world = sand_world(1200, 600);
        let mut rng = super::super::Rng(0xC33);
        let placed = pass_pyramids(&mut world, &[500], &[100], 1, 1100, false, &mut rng, None);
        assert_eq!(placed, 1);
        let count = |kind: u16| {
            world
                .tiles
                .iter()
                .filter(|t| t.active() && t.kind == kind)
                .count()
        };
        assert!(
            count(TILE_CHEST) >= 4,
            "the chamber chest ({})",
            count(TILE_CHEST)
        );
        assert!(count(TILE_SMALL_PILES) > 0, "the small piles");
        assert!(count(TILE_BANNER) > 0, "the banners");
        assert!(count(TILE_POT) > 0, "the pot row");
        // The chest's style is 1 (the gold frame base column 18).
        let chest = world
            .tiles
            .iter()
            .find(|t| t.active() && t.kind == TILE_CHEST && t.frame_y == 0)
            .unwrap();
        assert_eq!(chest.frame_x % 36, 18, "the style-1 frame column");
    }
    /// W4-c33 witness PORTED onto d103's signature (research/120 section 5): a
    /// water-filled basin over a sand floor takes style-17 chests (frame
    /// base column 18x17 = 306) from both the treasure anchors and the
    /// scatter; the deterministic replay (a == b) pins the draw order.
    #[test]
    fn the_water_chests_pass_stocks_the_basin() {
        let basin = || {
            let mut world = World::new("wc".into(), 1200, 600);
            world.world_surface = 100.0;
            world.rock_layer = 300.0;
            for x in 0..1200 {
                for y in 140..600 {
                    let i = world.index(x, y);
                    world.tiles[i].kind = super::super::TILE_SAND;
                    world.tiles[i].flags |= FLAG_ACTIVE;
                }
                for y in 100..140 {
                    let i = world.index(x, y);
                    world.tiles[i].liquid = 255;
                }
            }
            world
        };
        let run = || {
            let mut world = basin();
            let mut gv = super::pipeline::GenVars::default();
            gv.num_ocean_cave_treasure = 2;
            gv.ocean_cave_treasure = [(300, 120), (900, 120)];
            let mut rng = super::super::Rng(0x0CEA);
            super::water_chests::pass_water_chests(
                &mut world,
                &mut gv,
                100.0,
                300.0,
                super::buried_chest::ChestSeedFlags::default(),
                false,
                &mut rng,
                None,
            );
            world
        };
        let a = run();
        let b = run();
        for (ta, tb) in a.tiles.iter().zip(&b.tiles) {
            assert_eq!(ta, tb, "the a == b determinism half");
        }
        // Every placed chest is the style-17 water chest (frame base 306).
        let chests: Vec<_> = a
            .tiles
            .iter()
            .filter(|t| t.active() && t.kind == TILE_CHEST && t.frame_y == 0)
            .collect();
        assert!(!chests.is_empty(), "the basin takes water chests");
        for c in chests {
            assert_eq!(c.frame_x, 306, "the style-17 frame column");
        }
    }
}


/// `TileID.Hellforge` (`TileID.cs`: 77) — the 3×2 forge the hell houses
/// host (`WorldGen.cs:60079-60083` routes it through `Place3x2`).
/// Restored from the coherent parent `f1f03a45e^2:691-693` (the constant
/// was lost in the union although all call sites survived).
pub(crate) const TILE_HELLFORGE: u16 = 77;

/// `WorldGen.PlaceTile(i, j, 77)` (`WorldGen.cs:59342-60295`), the slice
/// the Hellforge pass runs — the g37 B7 PlaceTile route grafted onto this
/// lane's correctly-ordered pass (research/86 §6.2; g37 research/75 §2.7
/// recommended the graft, its own slot ordering was the bug). The arms
/// that bind for kind 77:
/// - `:59345` the gen-time existing-488 veto;
/// - `:59373` the entry gate `forced || EmptyTile || !tileSolid[77]` —
///   `Main.tileSolid[77]` is FALSE (no `Main.cs` assignment; the port's
///   generated `TILE_SOLID` bit 77 is 0), so the entry ALWAYS admits,
///   even over an active solid cell (the Place3x2 body scan then
///   refuses); `Collision.EmptyTile` reduces to `!active` at gen time;
/// - `:59454-59471` the liquid gate — 77 is neither a torch nor in the
///   refusal list, so liquid never vetoes;
/// - `:59473-59476` the inactive-cell `Clear(Tile|TilePaint|Slope)`
///   (`Tile.cs:744-790` — type/active/frames zeroed, paint + coatings +
///   slope + halfBrick cleared; wall, liquid, wiring, actuator survive);
/// - `:59477-59482` the `ResetsHalfBrickPlacementAttempt[77]` arm (the
///   set is default-true, `TileID.cs:497` — 77 is NOT in the false
///   list): an active non-frame-important cell loses halfBrick + frames
///   even when the placement then fails;
/// - `:60079-60083` the 77 dispatch → `Place3x2` + `SquareTileFrame`;
/// - `:60255-60262` the tail: `TruncatesWalls[77]` is false
///   (`TileID.cs:595`), the second `SquareTileFrame` runs, `result` =
///   the centre cell ended active. The sound/dust block is gen-inert.
/// Returns the C# `result`; the pass itself checks `type == 77`.
fn place_tile_hellforge(world: &mut World, x: i32, y: i32) -> bool {
    // `:59345`: during generation an existing fallen log blocks placement.
    if world.tile(x, y).is_some_and(|t| t.active() && t.kind == 488) {
        return false;
    }
    // `:59349` (`77 < TileID.Count`) and `:59354` (bounds).
    if x < 0 || y < 0 || x >= world.width || y >= world.height {
        return false;
    }
    let active_here = world.tile(x, y).is_some_and(|t| t.active());
    // `:59373`: `forced(false) || EmptyTile || !tileSolid[77]` — the
    // third disjunct always admits (77 is non-solid); ported verbatim.
    let entry = !active_here || !crate::tiledata::TILE_SOLID.has(TILE_HELLFORGE);
    if !entry {
        return false;
    }
    // `:59375-59453`: the grass/moss/coral/drip guards — other kinds.
    // `:59454-59471`: the liquid gate passes for 77 (see the doc).
    if !active_here {
        // `:59473-59476`: `Clear(Tile | TilePaint | Slope)`.
        if let Some(t) = world.tile_mut(x, y) {
            t.kind = 0;
            t.flags &= !FLAG_ACTIVE;
            t.frame_x = 0;
            t.frame_y = 0;
            t.set_paint(0);
            t.flags &= !crate::tile::FLAG_FULLBRIGHT;
            t.set_slope(0);
            t.flags &= !crate::tile::FLAG_HALF_BRICK;
        }
        world.set_invisible_block(x, y, false);
    } else if !world
        .tile(x, y)
        .is_some_and(|t| crate::tileobject::frame_important(t.kind))
    {
        // `:59477-59482`: `ResetsHalfBrickPlacementAttempt[77]` is true —
        // an active non-frame-important cell loses halfBrick + frames.
        if let Some(t) = world.tile_mut(x, y) {
            t.flags &= !crate::tile::FLAG_HALF_BRICK;
            t.frame_y = 0;
            t.frame_x = 0;
        }
    }
    // `:60079-60083`: the 77 dispatch — `Place3x2` + `SquareTileFrame`.
    super::evil::place_3x2(world, x, y, TILE_HELLFORGE, 0);
    crate::edit::square_tile_frame_gen(world, x, y);
    // `:60255-60262`: the tail re-frame + result (`TruncatesWalls[77]`
    // is false — no `SquareWallFrame`).
    if world.tile(x, y).is_some_and(|t| t.active()) {
        crate::edit::square_tile_frame_gen(world, x, y);
        return true;
    }
    false
}


/// The `Hellforge` pass (`WorldGen.cs:18187-18234`, W4-d104 pass 30):
/// `maxTilesX / 200` forges, each a retry loop — the `Next(1, maxTilesX)`
/// column + the `Next(maxTilesY - 250, maxTilesY - 30)` row, the wall
/// 13/14 (the hell-house brick walls) gate, the drop to the first active
/// cell (`j < maxTilesY - 20` bound), then `PlaceTile(num3, j - 1, 77)`
/// (the full PlaceTile route — [`place_tile_hellforge`], the g37 B7
/// graft). The 10000-failure valve ends the retry. The C#'s `try/catch`
/// covers the out-of-bounds reads — the port's bounds-checked `tile`
/// reads stand in (a miss counts as a failure). The slot stays AFTER
/// `hell forts` (the wall-14 producer, `AddHellHouses` `:13840` inside
/// Underworld `:13645`) — g37's before-hell-forts slot never fires
/// (research/86 §1 B7 / §6.2).
pub(crate) fn pass_hellforge(world: &mut World, rng: &mut super::Rng) -> u32 {
    let num = world.width / 200;
    // The sub-vanilla guard (the g37 graft): the C#
    // `Next(maxTilesY - 250, maxTilesY - 30)` band goes negative below
    // height 250 (the C# throws); a zero count draws nothing.
    if world.height <= 250 || num <= 0 {
        return 0;
    }
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
                // `PlaceTile(num3, j, 77)` — the full route (the entry
                // gate, the Clear arm, the halfBrick reset, the Place3x2
                // dispatch + the SquareTileFrame calls).
                place_tile_hellforge(world, num3, j);
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
pub(crate) fn pass_piles_surface_bands(
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

    /// W4-d104 pass 30 graft (research/86 §6.2): the PlaceTile-77 Clear
    /// arm (`:59473-59476`) — an inactive seat carrying stale
    /// frames/paint/coating/slope/halfBrick is cleared BEFORE the
    /// Place3x2 write (the bare pre-graft route left them on the forge).
    #[test]
    fn w4_d104_30_the_placetile_clear_arm_cleans_the_seat() {
        let mut w = World::new("hf".into(), 100, 100);
        // A solid floor at row 61 under columns 49..52.
        for x in 49..52 {
            let i = w.index(x, 61);
            w.tiles[i].kind = TILE_STONE;
            w.tiles[i].flags |= FLAG_ACTIVE;
        }
        // The seat (50, 60): inactive but carrying residue.
        let i = w.index(50, 60);
        w.tiles[i].kind = TILE_STONE;
        w.tiles[i].frame_x = 99;
        w.tiles[i].frame_y = 54;
        w.tiles[i].set_paint(5);
        w.tiles[i].set_slope(2);
        w.tiles[i].flags |= crate::tile::FLAG_HALF_BRICK | crate::tile::FLAG_FULLBRIGHT;
        w.set_invisible_block(50, 60, true);
        assert!(place_tile_hellforge(&mut w, 50, 60), "the forge places");
        let t = w.tile(50, 60).unwrap();
        assert_eq!(t.kind, TILE_HELLFORGE);
        assert_eq!(t.paint(), 0, "Clear zeroed the paint");
        assert_eq!(t.slope(), 0, "Clear zeroed the slope");
        assert!(!t.half_brick(), "Clear dropped the halfBrick");
        assert!(!t.fullbright_block(), "Clear dropped the Glow coating");
        assert!(!w.invisible_block(50, 60), "Clear dropped the Echo coating");
        assert_eq!(t.frame_x, 18, "the Place3x2 centre frame (54*0 + 18)");
        assert_eq!(t.frame_y, 18, "the Place3x2 bottom-row frame");
    }

    /// W4-d104 pass 30 graft: the `ResetsHalfBrickPlacementAttempt[77]`
    /// arm (`:59477-59482`, the set is default-true — `TileID.cs:497`) —
    /// an ACTIVE non-frame-important seat loses halfBrick + frames even
    /// though the Place3x2 body scan then refuses the placement. The
    /// entry gate itself always admits (`tileSolid[77]` is false). The
    /// C# `result` is the centre-cell-active flag (`:60255-60262`), so
    /// it reads TRUE here — the pass's own `type == 77` check is what
    /// counts the failure.
    #[test]
    fn w4_d104_30_the_halfbrick_reset_fires_on_a_refused_seat() {
        let mut w = World::new("hf".into(), 100, 100);
        // An active half-bricked stone seat with a stale frame.
        let i = w.index(50, 60);
        w.tiles[i].kind = TILE_STONE;
        w.tiles[i].flags |= FLAG_ACTIVE | crate::tile::FLAG_HALF_BRICK;
        w.tiles[i].frame_x = 54;
        w.tiles[i].frame_y = 18;
        assert!(
            place_tile_hellforge(&mut w, 50, 60),
            "result = centre active (the stone seat stays active)"
        );
        let t = w.tile(50, 60).unwrap();
        assert_eq!(t.kind, TILE_STONE, "the body scan refused the forge");
        assert!(t.active());
        assert!(!t.half_brick(), "the reset dropped the halfBrick");
        assert_eq!(t.frame_x, 0, "the reset zeroed frameX");
        assert_eq!(t.frame_y, 0, "the reset zeroed frameY");
    }

    /// W4-d104 pass 30 graft: the gen-time existing-488 veto (`:59345`)
    /// refuses before any write.
    #[test]
    fn w4_d104_30_the_existing_log_vetoes() {
        let mut w = World::new("hf".into(), 100, 100);
        let i = w.index(50, 60);
        w.tiles[i].kind = 488;
        w.tiles[i].flags |= FLAG_ACTIVE | crate::tile::FLAG_HALF_BRICK;
        assert!(!place_tile_hellforge(&mut w, 50, 60));
        let t = w.tile(50, 60).unwrap();
        assert_eq!(t.kind, 488, "the log stays");
        assert!(t.half_brick(), "the veto ran before the reset arm");
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
        pass_piles_surface_bands(&mut a, 100.0, 300.0, &mut crate::worldgen::noise::Rng(42));
        pass_piles_surface_bands(&mut b, 100.0, 300.0, &mut crate::worldgen::noise::Rng(42));
        assert_eq!(a.tiles, b.tiles, "a == b: the pass stream replays");
        assert!(
            a.tiles.iter().any(|t| t.active() && matches!(t.kind, 185 | 186 | 187)),
            "a pile landed"
        );

    }

    /// A mud world with a hive-sized air pocket and a hive floor.
    fn hive_world(width: i32, height: i32) -> World {
        let mut w = World::new("larva".into(), width, height);
        w.frame_important = vec![false; crate::tiledata::TILE_COUNT];
        w.frame_important[TILE_LARVA as usize] = true;
        for x in 0..width {
            for y in 0..height {
                let i = w.index(x, y);
                w.tiles[i].kind = TILE_MUD;
                w.tiles[i].flags |= FLAG_ACTIVE;
            }
        }
        w
    }

    /// `pass_larva` (`WorldGen.cs:20868-20900`): the anchor's 3×4 box — the
    /// three upper rows cleared, the floor row hive 225, the 3×3 larva 231
    /// object with the k*18 / l*18 frames.
    #[test]
    fn the_larva_pass_places_the_3x3_object() {
        let mut w = hive_world(200, 200);
        // Clear the 3×4 box above the floor at (100, 101).
        for j in 99..=101 {
            for k in 98..=101 {
                let i = w.index(j, k);
                w.tiles[i].flags &= !FLAG_ACTIVE;
            }
        }
        pass_larva(&mut w, &[(100, 100)]);
        // The floor row (y 101) is hive 225.
        for j in 99..=101 {
            let t = w.tile(j, 101).unwrap();
            assert!(t.active() && t.kind == TILE_HIVE, "the floor at {j}");
        }
        // The 3×3 larva object at (99..101, 98..100).
        for k in 0..3 {
            for l in 0..3 {
                let t = w.tile(99 + k, 98 + l).unwrap();
                assert!(t.active() && t.kind == TILE_LARVA);
                assert_eq!(t.frame_x, (k * 18) as i16);
                assert_eq!(t.frame_y, (l * 18) as i16);
            }
        }
    }

    /// The empty ledger places nothing.
    #[test]
    fn the_larva_pass_with_no_anchors_is_a_noop() {
        let mut w = hive_world(200, 200);
        let before = w.tiles.iter().filter(|t| t.active() && t.kind == TILE_LARVA).count();
        pass_larva(&mut w, &[]);
        let after = w.tiles.iter().filter(|t| t.active() && t.kind == TILE_LARVA).count();
        assert_eq!(before, after);
    }

    /// The hive pass publishes the larva anchors into the ledger (the 1/2
    /// roll), clamped to the 5-border; the Larva pass then places them.
    #[test]
    fn the_hive_pass_publishes_the_larva_ledger() {
        let spans = super::super::biomes::BiomeSpans {
            jungle_start: 60,
            jungle_end: 340,
            dungeon_x: 40,
        };
        // Sweep seeds until the 1/2 larva rolls publish an anchor (the
        // living-trees test's convention for roll-gated content).
        let mut found = None;
        for seed in 1..20u32 {
            let mut w = hive_world(400, 300);
            w.world_surface = 75.0;
            w.rock_layer = 150.0;
            let mut rng = crate::worldgen::noise::Rng::from_seed(seed);
            let mut larva = Vec::new();
            let placed = pass_beehives(&mut w, &spans, 150.0, &mut rng, &mut larva);
            assert!(placed >= 1, "the hive placed: {placed}");
            // Every anchor is inside the 5-border.
            for &(x, y) in &larva {
                assert!(x >= 5 && x <= w.width - 5);
                assert!(y >= 5 && y <= w.height - 5);
            }
            // The larva tiles are NOT placed by the hive pass anymore —
            // the Larva pass owns them.
            let larva_tiles =
                w.tiles.iter().filter(|t| t.active() && t.kind == TILE_LARVA).count();
            assert_eq!(larva_tiles, 0, "the hive pass defers the larva placement");
            if !larva.is_empty() {
                found = Some((w, larva));
                break;
            }
        }
        let (mut w, larva) = found.expect("a seed in 1..20 published a larva anchor");
        // Running the Larva pass places the objects. Each anchor
        // contributes a whole 3×3 object or nothing (the Place3x3 gate is
        // all-or-nothing; overlapping boxes can overwrite cells).
        pass_larva(&mut w, &larva);
        let after = w.tiles.iter().filter(|t| t.active() && t.kind == TILE_LARVA).count();
        assert!(after > 0, "the larva pass placed the objects");
        assert!(after <= larva.len() * 9);
    }
}
