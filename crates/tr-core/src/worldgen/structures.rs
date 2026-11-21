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
//! Deviations (ANALYSIS "Structures (M1.4-WGEN-2)"): the chest inventories
//! ride the storage task (the chests place as tiles only — the C#
//! `Chest.CreateChest` book is the M7-TE follow-up); the shrine/altar
//! layouts are shape-level (the C#'s exact brick patterns are not 1:1);
//! the boulder piles and the graveyards are out of scope. The pyramid is
//! the faithful `Pyramid` port (W4-c33 — the 151 hull, the wall-34 sweep,
//! the entrance shaft, the zig-zag tunnel with the chamber and the
//! post-chamber surface tunnel); its secret-seed arms
//! (tenthAnniversary/dualDungeons/surfaceIsDesert/errorWorld) stay with
//! the seed lanes (d107).

use super::{TILE_DIRT, TILE_STONE, WALL_DIRT};
use super::biomes::{TILE_JUNGLE_GRASS, TILE_MUD};
use crate::tile::{FLAG_ACTIVE, Tile};
use crate::world::World;

pub(crate) const TILE_SHRINE_BRICK: u16 = 119;
pub(crate) const TILE_TORCH: u16 = 4;
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

/// `UnderwaterChests` / "Water Chests" (`WorldGen.cs:17245-17355`): the
/// `numOceanCaveTreasure` loop (the `NextFromList` {863,186,277,187,4404}
/// main, the 2.0→50.0 +0.1 widening jitter box, the `liquid > 250` gate,
/// `AddBuriedChest(..., 17, trySlope: true)`), then the `9 · width/4200`
/// scatter (the `Next(10)` 863 override + the num4 1→186 / 2→4404 / 3→277
/// / else 187 cycle, two water hunts per iteration — `Next(1,
/// UnderworldLayer)` then `Next(worldSurface, UnderworldLayer)` — each
/// with the 10000-attempt cap, `trySlope` = the beach-band test). The
/// extraLiquid/notTheBees/remix/tenthAnniversary arms stay with the seed
/// lanes (d107); `Main.UnderworldLayer` is `maxTilesY - 200`
/// (`Main.cs:2790`), `beachDistance` 380 (`:86914`). The pass's window:
/// 162 non-solid (after `Ice` `:16694`), 232 solid (until the temple
/// `:17708`).
pub(crate) fn pass_underwater_chests(
    world: &mut World,
    num_ocean_cave_treasure: i32,
    ocean_cave_treasure: &[(i32, i32); 2],
    rng: &mut super::Rng,
    mut stock: Option<&mut super::chest_stock::ChestStockHook<'_>>,
) -> u32 {
    let mut placed = 0u32;
    let underworld_layer = world.height - 200;
    // `:17250-17270`: the ocean-cave treasure anchors.
    for i in 0..num_ocean_cave_treasure as usize {
        // `:17252`: `NextFromList` — `objs[Next(len)]` (`Utils.cs:1213`).
        const MAINS: [u16; 5] = [863, 186, 277, 187, 4404];
        let main = MAINS[rng.range(0, MAINS.len() as i32) as usize];
        let mut found = false;
        let mut num = 2.0f64;
        while !found && num < 50.0 {
            num += 0.1;
            let (ax, ay) = ocean_cave_treasure[i];
            // `:17262-17264`: the jitter box + the x-fold (the truncating
            // casts ride the C#'s `(int)` semantics).
            let mut num2 = rng.range(ax - num as i32, ax + num as i32 + 1);
            let num3 = rng.range(ay - num as i32 / 2, ay + num as i32 / 2 + 1);
            num2 = if num2 >= world.width {
                (num2 as f64 + num / 2.0) as i32
            } else {
                (num2 as f64 - num / 2.0) as i32
            };
            // `:17265`: the water gate (`liquidType() == 0` — the classic
            // world's only liquid arm).
            if super::temple::in_world(world, num2, num3, 0)
                && world.tile(num2, num3).is_some_and(|t| {
                    t.liquid > 250 && t.liquid_kind() == crate::tile::LiquidKind::Water
                })
            {
                // `:17267`: style 17, trySlope true.
                found = add_buried_chest(
                    world,
                    num2,
                    num3,
                    main,
                    false,
                    17,
                    true,
                    false,
                    true,
                    stock.as_deref_mut(),
                    rng,
                );
                if found {
                    placed += 1;
                }
            }
        }
    }
    // `:17271-17353`: the scatter.
    let mut num4 = 0;
    let num5 = world.width as f64 / 4200.0;
    let mut j = 0;
    while (j as f64) < 9.0 * num5 {
        let mut num6: u16 = 0;
        num4 += 1;
        // `:17287-17316`: the 863 override + the item cycle.
        if rng.range(0, 10) == 0 {
            num6 = 863;
        } else {
            match num4 {
                1 => num6 = 186,
                2 => num6 = 4404,
                3 => num6 = 277,
                _ => {
                    num6 = 187;
                    num4 = 0;
                }
            }
        }
        // `:17317-17334`: the first water hunt (the full-depth band).
        let mut found = false;
        let mut num7 = 0;
        while !found {
            let mut num8 = rng.range(50, world.width - 50);
            let mut num9 = rng.range(1, underworld_layer);
            // `:17323-17327`: the re-draw loop — the retry's y-band starts
            // at 50 (the C#'s own asymmetry with the first draw's 1).
            while !water_at(world, num8, num9) {
                num8 = rng.range(50, world.width - 50);
                num9 = rng.range(50, underworld_layer);
            }
            let try_slope = num8 < super::ocean::BEACH_DISTANCE
                || num8 > world.width - super::ocean::BEACH_DISTANCE;
            found = add_buried_chest(
                world,
                num8,
                num9,
                num6,
                false,
                17,
                try_slope,
                false,
                true,
                stock.as_deref_mut(),
                rng,
            );
            if found {
                placed += 1;
            }
            num7 += 1;
            if num7 > 10000 {
                break;
            }
        }
        // `:17335-17352`: the second water hunt (the surface-down band).
        found = false;
        num7 = 0;
        while !found {
            let mut num10 = rng.range(50, world.width - 50);
            let mut num11 = rng.range(world.world_surface as i32, underworld_layer);
            while !water_at(world, num10, num11) {
                num10 = rng.range(50, world.width - 50);
                num11 = rng.range(world.world_surface as i32, underworld_layer);
            }
            let try_slope = num10 < super::ocean::BEACH_DISTANCE
                || num10 > world.width - super::ocean::BEACH_DISTANCE;
            found = add_buried_chest(
                world,
                num10,
                num11,
                num6,
                false,
                17,
                try_slope,
                false,
                true,
                stock.as_deref_mut(),
                rng,
            );
            if found {
                placed += 1;
            }
            num7 += 1;
            if num7 > 10000 {
                break;
            }
        }
        j += 1;
    }
    placed
}

/// The scatter's water probe (`:17323`/`:17341`): `liquid >= 250` and
/// water (`liquidType() == 0` on the classic path).
fn water_at(world: &World, x: i32, y: i32) -> bool {
    world
        .tile(x, y)
        .is_some_and(|t| t.liquid >= 250 && t.liquid_kind() == crate::tile::LiquidKind::Water)
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

#[cfg(test)]
mod tests {
    use super::*;

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

    /// W4-c33: the Water Chests pass — a water-filled basin over a sand
    /// floor takes style-17 chests (the frame base column 18·17 = 306)
    /// from both the treasure anchors and the scatter; the deterministic
    /// replay (a == b) pins the draw order.
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
            let mut rng = super::super::Rng(0x0CEA);
            let placed =
                pass_underwater_chests(&mut world, 2, &[(300, 120), (900, 120)], &mut rng, None);
            (world, placed)
        };
        let (a, pa) = run();
        let (b, pb) = run();
        assert_eq!(pa, pb);
        assert!(pa > 0, "the basin takes water chests ({pa})");
        for (ta, tb) in a.tiles.iter().zip(&b.tiles) {
            assert_eq!(ta, tb, "the a == b determinism half");
        }
        // Every placed chest is the style-17 water chest (frame base 306).
        let chests: Vec<_> = a
            .tiles
            .iter()
            .filter(|t| t.active() && t.kind == TILE_CHEST && t.frame_y == 0)
            .collect();
        assert!(!chests.is_empty());
        for c in chests {
            assert_eq!(c.frame_x, 306, "the style-17 frame column");
        }
    }
}
