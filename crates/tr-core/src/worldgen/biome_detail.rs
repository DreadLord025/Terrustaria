//! The biome-detail passes — the compact ports of the `GenPassNameID`
//! decorators: the ocean caves (`OceanCaves` `WorldGen.cs:15167-15191`), the
//! fragile ice over the ice-biome water (`FragileIceOverIceBiomeWater`
//! `:16667-16690`), the underground jungle trees (`UndergroundJungleTrees`
//! `:17840-17858` + `GrowUndergroundTree` `:30849-31299`), the wet surface
//! jungle (`SurfaceWaterInJungle` `:15810-15833`), the jungle arms of the
//! plants pass (`GlowingMushroomPlantsUndergroundAndJunglePlants`
//! `:20122-20164`), the detritus sweep (`JunglePlantsPart2` `:20165-20192`)
//! and the jungle mud-wall conversion (`DirtWallsIntoMudWallsInJungle…`
//! `:20813-20866`).
//!
//! Tiles (`TileID.cs`): the vines 52, the jungle vines 62, the jungle plants
//! 61, the jungle thorns 69, the glowing mushroom 71, the mushroom vines 528,
//! the ice 161, the lihzahrd brick 226, the plant detritus 233. The
//! underground jungle "canopy" is regular tile-5 tree frames on the 22 px
//! trunk grid (`tree.rs`) — the kind-6 "`TreeBranches`" the pre-M5 port wrote
//! does not exist in the decompile (the ORES-1 finding; `mod.rs`'s ore-pair
//! test carried a wall filter only because of it).
//!
//! Deviations (ANALYSIS "Biome detail passes (M1.4-WGEN-3)", Phase 7 rows):
//! the ocean caves reuse the port's blob carving aimed at the ocean floor
//! (the C# `oceanCave`'s winding shaft shape is approximated); the mushroom
//! half of the plants pass stays shape-level (M5-BIOME-2's row); the
//! hallowed/crimson vine arms wait for the hallow (M5-HARDFORCE).

use super::biomes::{TILE_JUNGLE_GRASS, TILE_MUSHROOM_GRASS};
use super::Rng;
use crate::tile::{
    FLAG_ACTIVE, FLAG_INACTIVE, FLAG_LIQUID_HONEY, FLAG_LIQUID_LAVA, FLAG_LIQUID_SHIMMER, Tile,
};
use crate::tiledata::{TILE_SOLID, TILE_SOLID_TOP};
use crate::world::World;

pub(crate) const TILE_VINES: u16 = 52;
pub(crate) const TILE_JUNGLE_VINES: u16 = 62;
pub(crate) const TILE_JUNGLE_PLANTS: u16 = 61;
pub(crate) const TILE_GLOWING_MUSHROOM: u16 = 71;
pub(crate) const TILE_ICE: u16 = 161;

/// The tile/wall ids the M5-BIOME phase-5 passes write (`TileID.cs` /
/// `WallID.cs`).
const TILE_TREE: u16 = 5; // TileID.Trees (TileID.cs:639)
const TILE_JUNGLE_THORNS: u16 = 69; // TileID.JungleThorns (TileID.cs:767)
const TILE_JUNGLE_PLANTS2: u16 = 74; // TileID.JunglePlants2 (TileID.cs:777)
const TILE_LIHZAHRD_BRICK: u16 = 226; // TileID.LihzahrdBrick (TileID.cs:1081)
const TILE_PLANT_DETRITUS: u16 = 233; // TileID.PlantDetritus (TileID.cs:1095)
const WALL_DIRT: u16 = 2; // WallID.DirtUnsafe (WallID.cs:109)
const WALL_MUD_UNSAFE: u16 = 15; // WallID.MudUnsafe (WallID.cs:135)
const WALL_CAVE6_UNSAFE: u16 = 59; // WallID.Cave6Unsafe (WallID.cs:223)

fn active_at(world: &World, x: i32, y: i32) -> bool {
    world.tile(x, y).is_some_and(|t| t.active())
}

/// `Tile.nactive()` — active and not wired-inactive.
fn nactive(t: &Tile) -> bool {
    t.active() && (t.flags & FLAG_INACTIVE) == 0
}

/// The ocean caves (`OceanCaves`, `:15167-15191`): one winding shaft per
/// ocean side (a `Next(3)` gate per side) from the surface down past the
/// ocean floor, carved wide enough to walk. The shaft reuses the port's
/// blob carve aimed at the beach ramps. Shape-level placeholder — the
/// faithful `WorldGen.oceanCave` port replaces it (M5-OCEAN phase 3).
pub(crate) fn pass_ocean_caves(world: &mut World, surface: &[i32], rng: &mut Rng) {
    for side in 0..2 {
        if rng.range(0, 3) != 0 {
            continue;
        }
        let x = if side == 0 {
            rng.range(55, 95)
        } else {
            rng.range(world.width - 95, world.width - 55)
        };
        // The surface scan (`:15182-15184`): the first active tile down.
        let mut y = 0;
        while y < world.height - 2 && !active_at(world, x, y) {
            y += 1;
        }
        if y >= world.height - 4 {
            continue;
        }
        // The winding shaft down past the sea floor, then the terminal room.
        let bottom = (surface[x as usize] + 60).min(world.height - 60);
        let mut cx = x;
        let mut cy = y;
        while cy < bottom {
            let w = if cy < y + 12 { 1 } else { 2 };
            for dx in -w..=w {
                let px = cx + dx;
                if px < 2 || px >= world.width - 2 {
                    continue;
                }
                let i = world.index(px, cy);
                world.tiles[i] = Tile::default();
            }
            cy += 1;
            if rng.range(0, 3) == 0 {
                cx += rng.range(-1, 2);
            }
        }
        // The terminal bubble room.
        let r = rng.range(5, 9);
        for px in (cx - r)..=(cx + r) {
            for py in (cy - r)..=(cy + r) {
                if px < 2 || py < 2 || px >= world.width - 2 || py >= world.height - 2 {
                    continue;
                }
                let dist = ((px - cx) * (px - cx) + (py - cy) * (py - cy)) as f32;
                if dist.sqrt() <= r as f32 {
                    let i = world.index(px, py);
                    world.tiles[i] = Tile::default();
                }
            }
        }
    }
}

/// The fragile-ice pockets (`FragileIceOverIceBiomeWater`, `:16667-16690`):
/// the ice band's water tiles break their ice skin — the port carves the
/// 1-3 tile holes into the ice sheet above the water pockets, so the frozen
/// lakes read as broken.
pub(crate) fn pass_fragile_ice(world: &mut World, snow_start: i32, rng: &mut Rng) {
    let y0 = world.world_surface as i32;
    let y1 = world.height - 100;
    for x in 10..world.width - 10 {
        if x < snow_start {
            continue;
        }
        for y in y0..y1.min(world.height - 2) {
            let Some(t) = world.tile(x, y) else {
                continue;
            };
            if t.liquid == 0 || t.active() {
                continue;
            }
            // The water tile under an ice sheet: the skin opens.
            if !active_at(world, x, y - 1) {
                continue;
            }
            let above = world.tile(x, y - 1).unwrap();
            if above.kind != TILE_ICE || rng.range(0, 4) != 0 {
                continue;
            }
            let width = rng.range(1, 4);
            for dx in 0..width {
                let px = x + dx;
                if px >= world.width - 2 {
                    break;
                }
                if world
                    .tile(px, y - 1)
                    .is_some_and(|t| t.active() && t.kind == TILE_ICE)
                {
                    let i = world.index(px, y - 1);
                    world.tiles[i] = Tile::default();
                }
            }
        }
    }
}

// --- the underground jungle trees --------------------------------------------

/// `WorldGen.EmptyTileCheck` (`WorldGen.cs:31453-31535`), the full port:
/// bounds fail first; per tile only active tiles matter; `ignoreID −1`
/// rejects everything, `11`/`71` reject every other kind, and a
/// `CommonSapling` ignore-id (`TileID.cs:493` — {20, 590, 595, 615})
/// tolerates the sapling set plus the hardcoded plant list (`:31503-31528`).
/// A non-sapling ignore-id other than 11/71 leaves `flag` false and the
/// body falls through per tile — the decompile's own (lax) shape, ported
/// verbatim. `growth.rs`'s `empty_tile_check` only exempts the ignore kind
/// itself, which is the wrong semantics for the underground trees
/// (the phase discovery Gap 2; the Decision-#13 local-slice precedent).
fn empty_tile_check(
    world: &World,
    start_x: i32,
    end_x: i32,
    start_y: i32,
    end_y: i32,
    ignore_id: i32,
) -> bool {
    if start_x < 0 || end_x >= world.width as i32 || start_y < 0 || end_y >= world.height as i32 {
        return false;
    }
    // `flag = ignoreID != -1 && CommonSapling[ignoreID]` (`:31471-31475`).
    let sapling_ignore = matches!(ignore_id, 20 | 590 | 595 | 615);
    for x in start_x..=end_x {
        for y in start_y..=end_y {
            let Some(t) = world.tile(x, y) else {
                continue;
            };
            if !t.active() {
                continue;
            }
            match ignore_id {
                -1 => return false,
                11 | 71 => {
                    if t.kind as i32 != ignore_id {
                        return false;
                    }
                    continue;
                }
                _ => {}
            }
            if sapling_ignore && !matches!(t.kind, 20 | 590 | 595 | 615) {
                // The plant exemption switch (`:31503-31528`).
                if matches!(
                    t.kind,
                    3 | 24 | 32 | 61 | 62 | 69 | 71 | 73 | 74 | 82 | 83 | 84 | 110 | 113 | 184
                        | 201 | 233 | 352 | 485 | 529 | 530 | 637 | 655
                ) {
                    continue;
                }
                return false;
            }
        }
    }
    true
}

/// The underground jungle trees (`UndergroundJungleTrees`,
/// `WorldGen.cs:17840-17858`): every tile in `[worldSurface − 1,
/// maxTilesY − 350)` rolls `Next(10)` and grows a tree on jungle grass —
/// the full sweep, not the old span-restricted approximation (vanilla gates
/// on the anchor tile, so the sweep reaches only where 60 exists). The
/// `Main.tileSolid[379] = true` toggle (`:17844`, Bubble, `TileID.cs:1387`)
/// has no port consumer and no mutable tileSolid table — skipped (the plan's
/// toggle-model note; ANALYSIS row in Phase 7). The `drunkWorldGen` arm is a
/// secret-seed branch (out of scope). Vanilla re-seeds every pass
/// (`WorldGenerator.cs:497`); the port draws from `pass_rng("Jungle Trees")`.
pub(crate) fn pass_underground_jungle_trees(world: &mut World, world_surface: f64, rng: &mut Rng) {
    for i in 0..world.width {
        for j in (world_surface as i32 - 1)..(world.height - 350) {
            if rng.range(0, 10) != 0 {
                continue;
            }
            grow_underground_tree(world, i, j, rng, 0);
        }
    }
}

/// `WorldGen.GrowUndergroundTree` (`WorldGen.cs:30849-31299`): tile-5
/// trunks, roots and branch markers on the 22 px tree grid — the frame
/// table, the duplicate-branch re-roll, the neighbour-driven base
/// orientation and the `Next(8)` canopy fork are verbatim. The tile writes
/// mutate the stored fields (the wall/liquid behind the cave tree survive,
/// as in vanilla); `RangeFrame`'s 3×3 `TileFrame` walk (`:31294`) is the
/// runtime framer's job here, and `TileFrameImportant` has no case 5
/// (`:84964+`), so the stored frames are final in vanilla too.
fn grow_underground_tree(world: &mut World, i: i32, y: i32, rng: &mut Rng, tree_height_addon: i32) {
    // The anchor gate (`:30851`): tile 60, nactive, no half-brick/slope, and
    // at least one horizontal neighbour also 60.
    let Some(anchor) = world.tile(i, y) else {
        return;
    };
    if anchor.kind != TILE_JUNGLE_GRASS || !nactive(anchor) || anchor.half_brick() || anchor.slope() != 0
    {
        return;
    }
    let neighbour = |dx: i32| {
        world
            .tile(i + dx, y)
            .is_some_and(|t| t.active() && t.kind == TILE_JUNGLE_GRASS)
    };
    if !neighbour(-1) && !neighbour(1) {
        return;
    }
    // The height draws (`:30855-30861`): `num2 = Next(5, 15) + addon`, the
    // canvas `num3 = num2 + 2` (+5 more on tile 60 — always the anchor here;
    // the C# re-checks the type).
    let height = rng.range(5, 15) + tree_height_addon;
    let mut canvas = height + 2;
    if anchor.kind == TILE_JUNGLE_GRASS {
        canvas += 5;
    }
    // `EmptyTileCheck(i - 1, i + 1, y - num3, y - 1, 20)` (`:30862`).
    if !empty_tile_check(world, i - 1, i + 1, y - canvas, y - 1, 20) {
        return;
    }
    // The trunk loop (`:30866-31117`): per row the variant `num4 = Next(3)`,
    // the style `num5 = Next(10)` (top/bottom rows forced 0; a branch style
    // may not repeat the previous row's side), then the frame table, then the
    // branch tiles with their own two draws.
    let mut left_last = false;
    let mut right_last = false;
    for j in (y - height)..y {
        let mut num4 = rng.range(0, 3) as i16;
        let mut num5 = rng.range(0, 10);
        if j == y - 1 || j == y - height {
            num5 = 0;
        }
        while ((num5 == 5 || num5 == 7) && left_last) || ((num5 == 6 || num5 == 7) && right_last) {
            num5 = rng.range(0, 10);
        }
        left_last = false;
        right_last = false;
        if num5 == 5 || num5 == 7 {
            left_last = true;
        }
        if num5 == 6 || num5 == 7 {
            right_last = true;
        }
        let dy = num4 * 22;
        let (fx, fy) = match num5 {
            1 => (0, 66 + dy),
            2 => (22, dy),
            3 => (44, 66 + dy),
            4 => (22, 66 + dy),
            5 => (88, dy),
            6 => (66, 66 + dy),
            7 => (110, 66 + dy),
            _ => (0, dy),
        };
        place_tree_tile(world, i, j, fx, fy);
        // The left branch (`:31032-31073`).
        if num5 == 5 || num5 == 7 {
            let num4 = rng.range(0, 3) as i16;
            if rng.range(0, 3) < 2 {
                place_tree_tile(world, i - 1, j, 44, 198 + num4 * 22);
            } else {
                place_tree_tile(world, i - 1, j, 66, num4 * 22);
            }
        }
        // The right branch (`:31074-31116`).
        if num5 == 6 || num5 == 7 {
            let num4 = rng.range(0, 3) as i16;
            if rng.range(0, 3) < 2 {
                place_tree_tile(world, i + 1, j, 66, 198 + num4 * 22);
            } else {
                place_tree_tile(world, i + 1, j, 88, 66 + num4 * 22);
            }
        }
    }
    // The base/root orientation (`:31118-31158`): `num6 = Next(3)` remapped
    // by the grassy neighbours (dirt 2 / corrupt grass 23 / jungle grass 60 /
    // grass 109 / snow 147), then the roots and the base row.
    let mut num6 = rng.range(0, 3);
    let grassy = |dx: i32| {
        world.tile(i + dx, y).is_some_and(|t| {
            nactive(t) && !t.half_brick() && t.slope() == 0 && matches!(t.kind, 2 | 23 | 60 | 109 | 147)
        })
    };
    let flag3 = grassy(-1);
    let flag4 = grassy(1);
    if !flag3 {
        if num6 == 0 {
            num6 = 2;
        }
        if num6 == 1 {
            num6 = 3;
        }
    }
    if !flag4 {
        if num6 == 0 {
            num6 = 1;
        }
        if num6 == 2 {
            num6 = 3;
        }
    }
    if flag3 && !flag4 {
        num6 = 2;
    }
    if flag4 && !flag3 {
        num6 = 1;
    }
    if num6 == 0 || num6 == 1 {
        let num4 = rng.range(0, 3) as i16;
        place_tree_tile(world, i + 1, y - 1, 22, 132 + num4 * 22);
    }
    if num6 == 0 || num6 == 2 {
        let num4 = rng.range(0, 3) as i16;
        place_tree_tile(world, i - 1, y - 1, 44, 132 + num4 * 22);
    }
    let num4 = rng.range(0, 3) as i16;
    match num6 {
        0 => place_tree_tile(world, i, y - 1, 88, 132 + num4 * 22),
        1 => place_tree_tile(world, i, y - 1, 0, 132 + num4 * 22),
        2 => place_tree_tile(world, i, y - 1, 66, 132 + num4 * 22),
        _ => {} // num6 3: no grassy neighbour — the base keeps its trunk frame
    }
    // The top-row rewrite (`:31256-31293`): 7/8 keep the canopy marker
    // (frameX 22), 1/8 the plain trunk art (frameX 0) — the jungle tree's
    // "small" variant. There are no leaf tiles anywhere in the pass.
    let canopy = rng.range(0, 8) != 0;
    let num4 = rng.range(0, 3) as i16;
    place_tree_tile(
        world,
        i,
        y - height,
        if canopy { 22 } else { 0 },
        198 + num4 * 22,
    );
}

/// The tree-tile write (`Main.tile[x, y].active(true); .type = 5;
/// .frameX = …; .frameY = …`): a field mutation, not a replace — the
/// background wall and the liquid behind the cave tree survive, as in
/// vanilla. The writes sit inside the `EmptyTileCheck` canvas (or overwrite
/// the branch cells the C# also overwrites unchecked), so the bounds guard
/// is defensive only.
fn place_tree_tile(world: &mut World, x: i32, y: i32, frame_x: i16, frame_y: i16) {
    if x < 0 || y < 0 || x >= world.width || y >= world.height {
        return;
    }
    let idx = world.index(x, y);
    let t = &mut world.tiles[idx];
    t.kind = TILE_TREE;
    t.frame_x = frame_x;
    t.frame_y = frame_y;
    t.flags |= FLAG_ACTIVE;
}

// --- the wet surface jungle ---------------------------------------------------

/// The wet surface jungle (`SurfaceWaterInJungle` "Wet Jungle",
/// `WorldGen.cs:15810-15833`): no RNG — per column, scanning down from
/// `GenVars.worldSurfaceLow` while `j < worldSurface − 1.0`, the first
/// active tile breaks the scan; on jungle grass the two tiles above fill
/// with water (`liquid = 255`, `liquidType(0)` = clear the lava/honey/
/// shimmer bits).
pub(crate) fn pass_wet_jungle(world: &mut World, world_surface_low: f64, world_surface: f64) {
    for i in 0..world.width {
        let mut j = world_surface_low as i32;
        while (j as f64) < world_surface - 1.0 {
            let Some(t) = world.tile(i, j) else {
                break;
            };
            if t.active() {
                if t.kind == TILE_JUNGLE_GRASS {
                    for dy in 1..=2 {
                        let idx = world.index(i, j - dy);
                        let t = &mut world.tiles[idx];
                        t.liquid = 255;
                        t.flags &= !(FLAG_LIQUID_LAVA | FLAG_LIQUID_HONEY | FLAG_LIQUID_SHIMMER);
                    }
                }
                break;
            }
            j += 1;
        }
    }
}

// --- the jungle plant arms ----------------------------------------------------

/// The glowing-mushroom and jungle plants
/// (`GlowingMushroomPlantsUndergroundAndJunglePlants`, `:20122-20164`): the
/// tile-70 mushroom arm keeps the port's shape-level placement (the mushroom
/// half is M5-BIOME-2's row); the jungle arms are faithful — tile 60 grows
/// plant 61 on every free cell, tile 226 (the temple brick) draws `Next(5)`
/// against the `TooManyJunglePlantsNearby` density gate (`:20155`). The
/// `PlaceTile(…, 61)` internals draw too (`:59581-59625`); the pass runs on
/// its own `pass_rng("Glowing Mushrooms and Jungle Plants")`.
pub(crate) fn pass_mushroom_and_jungle_plants(
    world: &mut World,
    world_surface: f64,
    rock_layer: f64,
    rng: &mut Rng,
) {
    for i in 5..world.width - 5 {
        for j in 5..world.height - 5 {
            let Some(t) = world.tile(i, j) else {
                continue;
            };
            if !t.active() {
                continue;
            }
            let kind = t.kind; // copied — the arms below mutate (i, j − 1) only
            let above_active = active_at(world, i, j - 1);
            if j >= world_surface as i32 && kind == TILE_MUSHROOM_GRASS && !above_active {
                // `:20133-20147`: the glowing mushroom on the free cell (the
                // triple tree retry collapses to the plant placement — the
                // shape-level arm M5-BIOME-2 owns).
                let idx = world.index(i, j - 1);
                world.tiles[idx] = Tile {
                    kind: TILE_GLOWING_MUSHROOM,
                    frame_x: 0,
                    ..Default::default()
                };
                world.tiles[idx].flags |= FLAG_ACTIVE;
            }
            // The second empty check re-read the tile (`:20149`) — the
            // mushroom arm may have filled the cell above.
            if active_at(world, i, j - 1) {
                continue;
            }
            if kind == TILE_JUNGLE_GRASS {
                // `:20151-20154`.
                place_tile_jungle_plant(world, i, j - 1, rock_layer, world_surface, rng);
            } else if kind == TILE_LIHZAHRD_BRICK
                && rng.range(0, 5) == 0
                && !too_many_jungle_plants_nearby(world, i, j - 1, 2)
            {
                // `:20155-20158`.
                place_tile_jungle_plant(world, i, j - 1, rock_layer, world_surface, rng);
            }
        }
    }
}

/// `WorldGen.PlaceTile`'s type-61 arm (`WorldGen.cs:59581-59625`) — the
/// placement behind both jungle arms of the plants pass. The below-gate
/// (active, no slope/half-brick, 60 or 226) skips every draw when it fails;
/// otherwise `Next(16)`, `Next(60)` and `Next(230)` always draw (the `&
/// flag2` conjunctions and left-operand `&&`s still evaluate their left
/// sides), then `Next(15)`, then the taken arm's tail draws. The rare
/// `Next(16)==0 && j > worldSurface && !flag` arm places jungle thorns 69
/// with its stored frames (`SquareTileFrame` `:59591` is the runtime
/// framer's job); the bloom frames 144/162 need `j > rockLayer` (`flag2`),
/// which the temple brick never gets.
fn place_tile_jungle_plant(
    world: &mut World,
    i: i32,
    j: i32,
    rock_layer: f64,
    world_surface: f64,
    rng: &mut Rng,
) {
    // The below-gate (`:59583-59585`).
    let Some(below) = world.tile(i, j + 1) else {
        return;
    };
    if j + 1 >= world.height
        || !below.active()
        || below.slope() != 0
        || below.half_brick()
        || (below.kind != TILE_JUNGLE_GRASS && below.kind != TILE_LIHZAHRD_BRICK)
    {
        return;
    }
    let flag = below.kind == TILE_LIHZAHRD_BRICK;
    // `flag2 = j > Main.rockLayer` (the remix disjuncts are out of scope),
    // forced false on the brick (`:59586-59589`).
    let mut flag2 = j as f64 > rock_layer;
    if flag {
        flag2 = false;
    }
    let idx = world.index(i, j);
    if rng.range(0, 16) == 0 && j as f64 > world_surface && !flag {
        let t = &mut world.tiles[idx];
        t.kind = TILE_JUNGLE_THORNS;
        t.flags |= FLAG_ACTIVE;
    } else if rng.range(0, 60) == 0 && flag2 {
        let t = &mut world.tiles[idx];
        t.kind = TILE_JUNGLE_PLANTS;
        t.frame_x = 144;
        t.flags |= FLAG_ACTIVE;
    } else if rng.range(0, 230) == 0 && flag2 {
        let t = &mut world.tiles[idx];
        t.kind = TILE_JUNGLE_PLANTS;
        t.frame_x = 162;
        t.flags |= FLAG_ACTIVE;
    } else if rng.range(0, 15) == 0 && !flag {
        let t = &mut world.tiles[idx];
        t.kind = TILE_JUNGLE_PLANTS;
        t.flags |= FLAG_ACTIVE;
        if rng.range(0, 3) != 0 {
            t.frame_x = (rng.range(0, 2) * 18 + 108) as i16;
        } else {
            t.frame_x = (rng.range(0, 13) * 18 + 180) as i16;
        }
    } else {
        let t = &mut world.tiles[idx];
        t.kind = TILE_JUNGLE_PLANTS;
        t.frame_x = (rng.range(0, 6) * 18) as i16;
        t.flags |= FLAG_ACTIVE;
    }
}

/// `WorldGen.TooManyJunglePlantsNearby` (`WorldGen.cs:73164-73187`): the
/// ±9×±5 box `Utils.Clamp`'d into `[10, max−1−10]`, counting active 61/74
/// tiles; true once the count exceeds `maxCount` (2).
fn too_many_jungle_plants_nearby(world: &World, i: i32, j: i32, max_count: i32) -> bool {
    let lo_x = (i - 9).clamp(10, world.width as i32 - 1 - 10);
    let hi_x = (i + 9).clamp(10, world.width as i32 - 1 - 10);
    let lo_y = (j - 5).clamp(10, world.height as i32 - 1 - 10);
    let hi_y = (j + 5).clamp(10, world.height as i32 - 1 - 10);
    let mut count = 0;
    for x in lo_x..=hi_x {
        for y in lo_y..=hi_y {
            if world
                .tile(x, y)
                .is_some_and(|t| t.active() && (t.kind == TILE_JUNGLE_PLANTS || t.kind == TILE_JUNGLE_PLANTS2))
            {
                count += 1;
                if count > max_count {
                    return true;
                }
            }
        }
    }
    false
}

/// `JunglePlantsPart2` "Jungle Plants" (`WorldGen.cs:20165-20192`):
/// `maxTilesX · 100` attempts — the x in `[40, maxTilesX/2 − 40)` (the
/// dungeon-side offset `:20173-20176` never fires: the port's dungeon is
/// fixed Right, the Phase-3 Decision #10), the y walk down from
/// `Next(maxTilesY − 300)` to the first active tile; on jungle grass the
/// detritus 233 places with `Next(8)` styling, retrying `Next(12)` at
/// styleY 1 when the first attempt left no 233 (the C# reads `.type != 233`
/// regardless of active — a pre-existing 233 at the anchor suppresses the
/// retry, quirk ported). Runs on `pass_rng("Jungle Plants")`.
pub(crate) fn pass_jungle_plants_part2(world: &mut World, rng: &mut Rng) {
    for _ in 0..(world.width * 100) {
        // The dungeon sits right (`Decision #10`) — no `+ maxTilesX / 2`.
        let x = rng.range(40, world.width / 2 - 40);
        let mut j = rng.range(0, world.height - 300);
        while !active_at(world, x, j) && j < world.height - 300 {
            j += 1;
        }
        let Some(t) = world.tile(x, j) else {
            continue;
        };
        if t.active() && t.kind == TILE_JUNGLE_GRASS {
            let j = j - 1;
            let style_x = rng.range(0, 8);
            place_jungle_plant_detritus(world, x, j, TILE_PLANT_DETRITUS, style_x, 0);
            let placed = world
                .tile(x, j)
                .is_some_and(|t| t.kind == TILE_PLANT_DETRITUS);
            if !placed {
                let style_x = rng.range(0, 12);
                place_jungle_plant_detritus(world, x, j, TILE_PLANT_DETRITUS, style_x, 1);
            }
        }
    }
}

/// `WorldGen.PlaceJunglePlant` (`WorldGen.cs:47779-47944`), the two shapes
/// the jungle-plants pass runs (type 233 at styleY 0 and 1): the overlay
/// scan (active tiles allowed only in the plant set {61, 703, 62, 655, 69,
/// 74} plus pile 185 at frameY 0; the styleY>0 branch's `type != 236/702/238`
/// clauses all hold for 233), the solid-jungle-grass ground check under the
/// box, then the frame write — 3×2 at `54·styleX` (styleY 0) or 2×2 at
/// `36·styleX` with the +36 row (styleY > 0). `KillTile` collapses into the
/// overwrite (the box is fully rewritten on success) and
/// `CopyPaintAndCoating` is a no-op (paint is always 0 during generation).
fn place_jungle_plant_detritus(
    world: &mut World,
    x: i32,
    y: i32,
    kind: u16,
    style_x: i32,
    style_y: i32,
) {
    /// The styleY>0 overlay/ground scan (`:47801-47823`): 2 columns.
    fn scan_2x2(world: &World, x: i32, y: i32) -> bool {
        for cx in (x - 1)..(x + 1) {
            for cy in (y - 1)..(y + 1) {
                if let Some(t) = world.tile(cx, cy) {
                    if t.active()
                        && !matches!(t.kind, 61 | 703 | 62 | 655 | 69 | 74)
                        && !(t.kind == 185 && t.frame_y == 0)
                    {
                        return false;
                    }
                }
            }
            if !solid_tile(world, cx, y + 1) || !world.tile(cx, y + 1).is_some_and(|t| t.kind == TILE_JUNGLE_GRASS)
            {
                return false;
            }
        }
        true
    }

    /// The styleY==0 overlay/ground scan (`:47871-47893`): 3 columns.
    fn scan_3x2(world: &World, x: i32, y: i32) -> bool {
        for cx in (x - 1)..(x + 2) {
            for cy in (y - 1)..(y + 1) {
                if let Some(t) = world.tile(cx, cy) {
                    if t.active()
                        && !matches!(t.kind, 61 | 703 | 62 | 69 | 655 | 74)
                        && !(t.kind == 185 && t.frame_y == 0)
                    {
                        return false;
                    }
                }
            }
            if !solid_tile(world, cx, y + 1) || !world.tile(cx, y + 1).is_some_and(|t| t.kind == TILE_JUNGLE_GRASS)
            {
                return false;
            }
        }
        true
    }

    /// The frame write over the box — active + kind + frames, everything
    /// else preserved (the C# mutates the same fields).
    fn write(world: &mut World, cx: i32, cy: i32, kind: u16, fx: i16, fy: i16) {
        let idx = world.index(cx, cy);
        let t = &mut world.tiles[idx];
        t.kind = kind;
        t.frame_x = fx;
        t.frame_y = fy;
        t.flags |= FLAG_ACTIVE;
    }

    if style_y > 0 || kind == 236 || kind == 238 || kind == 702 {
        // The 2×2 branch (`:47781-47864`); `num = Y2` for 233 (the 95/126
        // shift does not apply).
        if x < 5 || x > world.width - 5 || y < 5 || y > world.height - 5 {
            return;
        }
        if !scan_2x2(world, x, y) {
            return;
        }
        let fx = (36 * style_x) as i16;
        write(world, x - 1, y - 1, kind, fx, 36);
        write(world, x, y - 1, kind, fx + 18, 36);
        write(world, x - 1, y, kind, fx, 54);
        write(world, x, y, kind, fx + 18, 54);
    } else {
        // The 3×2 branch (`:47865-47943`).
        if x < 5 || x > world.width - 5 || y < 5 || y > world.height - 5 {
            return;
        }
        if !scan_3x2(world, x, y) {
            return;
        }
        let fx = (54 * style_x) as i16;
        write(world, x - 1, y - 1, kind, fx, 0);
        write(world, x, y - 1, kind, fx + 18, 0);
        write(world, x + 1, y - 1, kind, fx + 36, 0);
        write(world, x - 1, y, kind, fx, 18);
        write(world, x, y, kind, fx + 18, 18);
        write(world, x + 1, y, kind, fx + 36, 18);
    }
}

/// `WorldGen.SolidTile` (`WorldGen.cs:70111-70131`) — no RNG; an
/// out-of-bounds tile reads as solid (`Main.tile[i, j] == null → true`).
fn solid_tile(world: &World, x: i32, y: i32) -> bool {
    let Some(t) = world.tile(x, y) else {
        return true;
    };
    t.active()
        && TILE_SOLID.has(t.kind)
        && !TILE_SOLID_TOP.has(t.kind)
        && !t.half_brick()
        && t.slope() == 0
        && (t.flags & FLAG_INACTIVE) == 0
}

// --- the jungle mud walls -----------------------------------------------------

/// `DirtWallsIntoMudWallsInJungleAndJungleMinMax` "Muds Walls In Jungle"
/// (`WorldGen.cs:20813-20866`): the left→right and right→left surface scans
/// (rows `j < worldSurface + 20`, columns `[5, maxTilesX − 5)`) write the
/// returned `GenVars.jungleMinX/jungleMaxX` pair — 0/0 when no surface
/// jungle exists (the C# initialises the locals to 0 and assigns
/// unconditionally; the research §3.7's "−1" is the GenVars reset that this
/// pass overwrites). The conversion sweep turns walls 2 (dirt) and 59
/// (Cave6Unsafe) into 15 (MudUnsafe) across the band, with ragged edges:
/// the two outermost columns on each side draw `Next(2)` and the three
/// outermost `Next(3)` — both draws evaluate before the wall test (the C#
/// short-circuit order), the inner columns skip them. Runs on
/// `pass_rng("Muds Walls In Jungle")`.
pub(crate) fn pass_muds_walls_in_jungle(
    world: &mut World,
    world_surface: f64,
    rng: &mut Rng,
) -> (i32, i32) {
    // `for (int j = 0; (double)j < Main.worldSurface + 20.0; j++)` — the
    // scan and conversion band, one row count for all three loops.
    let rows = {
        let mut rows = 0i32;
        let mut j = 0i32;
        while (j as f64) < world_surface + 20.0 {
            rows += 1;
            j += 1;
        }
        rows
    };
    let mut min_x = 0;
    let mut max_x = 0;
    let mut found = false;
    for i in 5..world.width - 5 {
        for j in 0..rows {
            if world
                .tile(i, j)
                .is_some_and(|t| t.active() && t.kind == TILE_JUNGLE_GRASS)
            {
                min_x = i;
                found = true;
                break;
            }
        }
        if found {
            break;
        }
    }
    found = false;
    let mut i = world.width - 5;
    while i > 5 {
        for j in 0..rows {
            if world
                .tile(i, j)
                .is_some_and(|t| t.active() && t.kind == TILE_JUNGLE_GRASS)
            {
                max_x = i;
                found = true;
                break;
            }
        }
        if found {
            break;
        }
        i -= 1;
    }
    // The conversion sweep (`:20856-20865`).
    for l in min_x..=max_x {
        for m in 0..rows {
            if !((l >= min_x + 2 && l <= max_x - 2) || rng.range(0, 2) != 0) {
                continue;
            }
            if !((l >= min_x + 3 && l <= max_x - 3) || rng.range(0, 3) != 0) {
                continue;
            }
            let Some(t) = world.tile(l, m) else {
                continue;
            };
            if t.wall == WALL_DIRT || t.wall == WALL_CAVE6_UNSAFE {
                let idx = world.index(l, m);
                world.tiles[idx].wall = WALL_MUD_UNSAFE;
            }
        }
    }
    (min_x, max_x)
}

/// The vines (`ActionVines` + the grass-growth vine arms): the vines hang
/// from the grass overhangs — the jungle vines 62 under the jungle grass,
/// the regular vines 52 under the forest grass, both growing down to the
/// length caps (`Next(2..70)` per family — the port caps 12 for the
/// surface vines and 20 for the jungle).
pub(crate) fn pass_vines(world: &mut World, rng: &mut Rng) {
    for x in 5..world.width - 5 {
        for y in 5..world.height - 5 {
            let Some(t) = world.tile(x, y) else {
                continue;
            };
            if !t.active() {
                continue;
            }
            if world.tile(x, y + 1).is_some_and(|n| n.active()) {
                continue;
            }
            let (vine, cap) = match t.kind {
                TILE_JUNGLE_GRASS => (TILE_JUNGLE_VINES, 20),
                TILE_GRASS => (TILE_VINES, 12),
                TILE_MUSHROOM_GRASS => (528, 12),
                _ => continue,
            };
            // The overhang only: the grass cell must hang (the air below).
            if rng.range(0, 3) != 0 {
                continue;
            }
            let length = rng.range(2, cap);
            for dy in 1..=length {
                let py = y + dy;
                if py >= world.height - 2 || active_at(world, x, py) {
                    break;
                }
                let i = world.index(x, py);
                world.tiles[i] = Tile {
                    kind: vine,
                    frame_x: 0,
                    ..Default::default()
                };
                world.tiles[i].flags |= FLAG_ACTIVE;
            }
        }
    }
}

/// `ExposedGemsInIceBiome` (`WorldGen.cs:20697-20735`): `maxTilesX · 0.25`
/// attempts — the y between `(worldSurface + rockLayer) / 2` and the lava
/// line, the x in the snow band, the anchor tile from the snow/ice family
/// (147/161/162/224); the gem patch (the tile 178, the styles 0-5 weighted
/// `Next(12)`: <3 → 0, <6 → 1, <8 → 2, <10 → 3, ≥11 → 5, else 4) fills the
/// `Next(1,4)`-half-sized rectangle of the free cells.
pub(crate) fn pass_exposed_gems(
    world: &mut World,
    snow_start: i32,
    rock_layer: f64,
    rng: &mut Rng,
) {
    let attempts = (world.width as f64 * 0.25) as i32;
    let y_lo = ((world.world_surface + rock_layer) / 2.0) as i32;
    let y_hi = (world.height - 170).min(world.height - 2);
    if y_hi <= y_lo {
        return;
    }
    for _ in 0..attempts {
        let y = rng.range(y_lo, y_hi);
        // The snow band's span at this row: the port's snow band is the
        // eastern third (the fill's `snow_start`), the gems scan inside it.
        let x = rng.range(snow_start.max(40), world.width - 40);
        let Some(t) = world.tile(x, y) else {
            continue;
        };
        if !t.active() || !matches!(t.kind, 147 | 161 | 162 | 224) {
            continue;
        }
        // The patch rectangle: the `Next(1, 4)` half-sizes.
        let left = rng.range(1, 4);
        let right = rng.range(1, 4);
        let up = rng.range(1, 4);
        let down = rng.range(1, 4);
        // The style weights (`:20718-20724`).
        let roll = rng.range(0, 12);
        let style = if roll < 3 {
            0
        } else if roll < 6 {
            1
        } else if roll < 8 {
            2
        } else if roll < 10 {
            3
        } else if roll >= 11 {
            5
        } else {
            4
        };
        for px in (x - left)..(x + right) {
            for py in (y - up)..(y + down) {
                if px < 40 || py < 40 || px >= world.width - 40 || py >= world.height - 40 {
                    continue;
                }
                if world.tile(px, py).is_some_and(|t| !t.active()) {
                    let i = world.index(px, py);
                    let t = &mut world.tiles[i];
                    t.kind = 178;
                    t.frame_x = (style * 18) as i16;
                    t.frame_y = 0;
                    t.flags |= FLAG_ACTIVE;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tile::FLAG_LIQUID_LAVA;

    /// A flat jungle patch: dirt walls everywhere, mud below row `surface`,
    /// jungle grass 60 across `x0..=x1` on the surface row, air above.
    fn jungle_patch_world(width: i32, height: i32, surface: i32, x0: i32, x1: i32) -> World {
        let mut w = World::new("t".into(), width, height);
        for x in 0..width {
            for y in 0..height {
                let idx = w.index(x, y);
                w.tiles[idx].wall = WALL_DIRT;
                if y > surface && y <= surface + 10 {
                    w.tiles[idx].kind = 59; // mud under the grass
                    w.tiles[idx].flags |= FLAG_ACTIVE;
                }
            }
        }
        for x in x0..=x1 {
            let idx = w.index(x, surface);
            w.tiles[idx].kind = TILE_JUNGLE_GRASS;
            w.tiles[idx].flags |= FLAG_ACTIVE;
        }
        w
    }

    /// The test tile setter — a fn item, so the borrows stay local.
    fn set_kind(w: &mut World, x: i32, y: i32, kind: u16, active: bool) {
        let idx = w.index(x, y);
        w.tiles[idx].kind = kind;
        if active {
            w.tiles[idx].flags |= FLAG_ACTIVE;
        } else {
            w.tiles[idx].flags &= !FLAG_ACTIVE;
        }
    }

    /// The test tile accessor — the two-statement borrow pattern.
    fn tile_mut(w: &mut World, x: i32, y: i32) -> &mut Tile {
        let idx = w.index(x, y);
        &mut w.tiles[idx]
    }

    // --- DW-5.1: EmptyTileCheck -------------------------------------------------

    /// DW-5.1: the faithful `EmptyTileCheck` — bounds fail, `ignoreID −1`
    /// rejects every active tile, the sapling ignore 20 tolerates the
    /// CommonSapling set {20, 590, 595, 615} and the plant list, everything
    /// else active blocks; inactive/liquid-only tiles never matter.
    #[test]
    fn test_DW_5_1_empty_tile_check_ignore_20() {
        let mut w = World::new("t".into(), 30, 40);
        // Bounds.
        assert!(!empty_tile_check(&w, -1, 5, 5, 9, 20));
        assert!(!empty_tile_check(&w, 5, 30, 5, 9, 20));
        assert!(!empty_tile_check(&w, 5, 9, -1, 9, 20));
        assert!(!empty_tile_check(&w, 5, 9, 5, 40, 20));
        // An empty box is empty.
        assert!(empty_tile_check(&w, 5, 9, 5, 9, 20));
        // A liquid-only box is empty (only active tiles matter).
        tile_mut(&mut w, 6, 6).liquid = 255;
        assert!(empty_tile_check(&w, 5, 9, 5, 9, 20));
        // The CommonSapling set and the plant list are tolerated.
        for kind in [
            20u16, 590, 595, 615, 3, 24, 32, 61, 62, 69, 71, 73, 74, 82, 83, 84, 110, 113, 184,
            201, 233, 352, 485, 529, 530, 637, 655,
        ] {
            let mut w2 = World::new("t".into(), 30, 40);
            set_kind(&mut w2, 6, 6, kind, true);
            assert!(
                empty_tile_check(&w2, 5, 9, 5, 9, 20),
                "kind {kind} must be tolerated"
            );
        }
        // Any other active kind blocks — a tree, dirt, stone.
        for kind in [5u16, 2, 1, 59, 60] {
            let mut w2 = World::new("t".into(), 30, 40);
            set_kind(&mut w2, 6, 6, kind, true);
            assert!(
                !empty_tile_check(&w2, 5, 9, 5, 9, 20),
                "kind {kind} must block"
            );
        }
        // ignoreID −1 rejects every active tile; the liquid-only box stays empty.
        set_kind(&mut w, 6, 6, 20, true);
        assert!(!empty_tile_check(&w, 5, 9, 5, 9, -1));
        set_kind(&mut w, 6, 6, 0, false);
        tile_mut(&mut w, 6, 6).liquid = 255;
        assert!(empty_tile_check(&w, 5, 9, 5, 9, -1));
    }

    // --- DW-5.1: GrowUndergroundTree ---------------------------------------------

    /// DW-5.1: the tree frames per the C# table — the full draw-order replay
    /// (trunk rows with the forced-0 top/bottom and the duplicate-branch
    /// re-roll, the branch tiles with their two draws, the neighbour-driven
    /// base orientation, the `Next(8)` canopy fork) computed independently in
    /// the test, then compared tile-for-tile against the port's world.
    #[test]
    fn test_DW_5_1_grow_underground_tree_frames_and_ledger() {
        let mut world = jungle_patch_world(40, 80, 60, 18, 22);
        let mut rng = Rng(0x5EED_1234);
        grow_underground_tree(&mut world, 20, 60, &mut rng, 0);

        // The independent C# transcription over the same stream.
        let mut replay = Rng(0x5EED_1234);
        let height = replay.range(5, 15); // num2 (:30856)
        // num3 = num2 + 2 + 5 (tile 60) — no draw (:30857-30861).
        let mut expected: Vec<(i32, i32, u16, i16, i16)> = Vec::new(); // (x, y, kind, fx, fy)
        let mut left_last = false;
        let mut right_last = false;
        for j in (60 - height)..60 {
            let mut num4 = replay.range(0, 3) as i16; // (:30873)
            let mut num5 = replay.range(0, 10); // (:30874)
            if j == 59 || j == 60 - height {
                num5 = 0;
            }
            while ((num5 == 5 || num5 == 7) && left_last)
                || ((num5 == 6 || num5 == 7) && right_last)
            {
                num5 = replay.range(0, 10);
            }
            left_last = num5 == 5 || num5 == 7;
            right_last = num5 == 6 || num5 == 7;
            let dy = num4 * 22;
            let (fx, fy) = match num5 {
                1 => (0, 66 + dy),
                2 => (22, dy),
                3 => (44, 66 + dy),
                4 => (22, 66 + dy),
                5 => (88, dy),
                6 => (66, 66 + dy),
                7 => (110, 66 + dy),
                _ => (0, dy),
            };
            expected.push((20, j, 5, fx, fy));
            if left_last {
                let num4 = replay.range(0, 3) as i16; // (:31036)
                if replay.range(0, 3) < 2 {
                    // (:31037)
                    expected.push((19, j, 5, 44, 198 + num4 * 22));
                } else {
                    expected.push((19, j, 5, 66, num4 * 22));
                }
            }
            if right_last {
                let num4 = replay.range(0, 3) as i16; // (:31080)
                if replay.range(0, 3) < 2 {
                    // (:31081)
                    expected.push((21, j, 5, 66, 198 + num4 * 22));
                } else {
                    expected.push((21, j, 5, 88, 66 + num4 * 22));
                }
            }
        }
        // The neighbours (19/21, 60) are both jungle grass → flag3 = flag4,
        // no remaps (:31121-31158); num6 survives as drawn.
        let num6 = replay.range(0, 3); // (:31118)
        if num6 == 0 || num6 == 1 {
            let num4 = replay.range(0, 3) as i16;
            expected.push((21, 59, 5, 22, 132 + num4 * 22));
        }
        if num6 == 0 || num6 == 2 {
            let num4 = replay.range(0, 3) as i16;
            expected.push((19, 59, 5, 44, 132 + num4 * 22));
        }
        let num4 = replay.range(0, 3) as i16; // (:31201)
        match num6 {
            0 => expected.push((20, 59, 5, 88, 132 + num4 * 22)),
            1 => expected.push((20, 59, 5, 0, 132 + num4 * 22)),
            2 => expected.push((20, 59, 5, 66, 132 + num4 * 22)),
            _ => {}
        }
        let canopy = replay.range(0, 8) != 0; // (:31256)
        let num4 = replay.range(0, 3) as i16;
        expected.push((20, 60 - height, 5, if canopy { 22 } else { 0 }, 198 + num4 * 22));

        // The ledger: the streams consumed exactly the same draws.
        assert_eq!(rng.0, replay.0, "the draw ledger");
        // The tiles: the writes land in order (the top-row rewrite lands
        // after the trunk row, so a map with last-write-wins semantics).
        let mut want: std::collections::BTreeMap<(i32, i32), (u16, i16, i16)> = Default::default();
        for (x, y, kind, fx, fy) in expected {
            want.insert((x, y), (kind, fx, fy));
        }
        for ((x, y), (kind, fx, fy)) in &want {
            let t = world.tile(*x, *y).unwrap();
            assert!(
                t.active() && t.kind == *kind && t.frame_x == *fx && t.frame_y == *fy,
                "at {x},{y}: got ({},{},{},{}), want ({kind},{fx},{fy})",
                t.active(),
                t.kind,
                t.frame_x,
                t.frame_y
            );
        }
        for x in 17..=23 {
            for y in (60 - height - 1)..61 {
                if !want.contains_key(&(x, y)) {
                    let t = world.tile(x, y).unwrap();
                    assert!(
                        !(t.active() && t.kind == 5),
                        "unexpected tree tile at {x},{y}"
                    );
                    assert_ne!(t.kind, 6, "kind 6 at {x},{y}");
                }
            }
        }
        // The mutation contract: the wall behind the base row survives.
        assert_eq!(world.tile(20, 59).unwrap().wall, WALL_DIRT);
        // The anchor grass is untouched.
        assert_eq!(world.tile(20, 60).unwrap().kind, TILE_JUNGLE_GRASS);
    }

    /// DW-5.1: the anchor gates draw nothing — no 60, no horizontal 60
    /// neighbour, a half-brick or a sloped anchor all return before the
    /// height draw; a blocked canvas costs exactly the height draw.
    #[test]
    fn test_DW_5_1_grow_underground_tree_gates() {
        // No neighbour: (20, 60) is the only grass.
        let mut w = jungle_patch_world(40, 80, 60, 20, 20);
        let mut rng = Rng(7);
        grow_underground_tree(&mut w, 20, 60, &mut rng, 0);
        assert_eq!(rng.0, Rng(7).0, "the no-neighbour gate draws nothing");

        // A half-brick anchor.
        let mut w = jungle_patch_world(40, 80, 60, 19, 21);
        tile_mut(&mut w, 20, 60).flags |= crate::tile::FLAG_HALF_BRICK;
        let mut rng = Rng(7);
        grow_underground_tree(&mut w, 20, 60, &mut rng, 0);
        assert_eq!(rng.0, Rng(7).0, "the half-brick gate draws nothing");

        // A sloped anchor.
        let mut w = jungle_patch_world(40, 80, 60, 19, 21);
        tile_mut(&mut w, 20, 60).set_slope(1);
        let mut rng = Rng(7);
        grow_underground_tree(&mut w, 20, 60, &mut rng, 0);
        assert_eq!(rng.0, Rng(7).0, "the slope gate draws nothing");

        // A blocked canvas: a stone tile inside the box for every drawable
        // height (the canvas top spans [39, 48], so 50 is always inside) —
        // exactly the height draw happens, no tile is written.
        let mut w = jungle_patch_world(40, 80, 60, 19, 21);
        set_kind(&mut w, 20, 50, 1, true);
        let mut rng = Rng(7);
        grow_underground_tree(&mut w, 20, 60, &mut rng, 0);
        let mut replay = Rng(7);
        replay.range(5, 15);
        assert_eq!(rng.0, replay.0, "only the height draw");
        assert!(!w.tiles.iter().any(|t| t.active() && t.kind == 5));
    }

    /// DW-5.1: the driver sweeps every column in
    /// `[worldSurface − 1, maxTilesY − 350)` — the per-tile `Next(10)` draw
    /// happens unconditionally (a world with no jungle grass at all still
    /// consumes the full sweep and places nothing).
    #[test]
    fn test_DW_5_1_driver_sweeps_all_columns() {
        let mut w = World::new("t".into(), 30, 400);
        for x in 0..30 {
            for y in 300..400 {
                let idx = w.index(x, y);
                w.tiles[idx].kind = 1;
                w.tiles[idx].flags |= FLAG_ACTIVE;
            }
        }
        let mut rng = Rng(9);
        pass_underground_jungle_trees(&mut w, 40.0, &mut rng);
        // j runs [39, 50) → 11 rows × 30 columns of unconditional draws.
        let mut replay = Rng(9);
        for _ in 0..(30 * 11) {
            replay.range(0, 10);
        }
        assert_eq!(rng.0, replay.0, "the sweep ledger");
        assert!(!w.tiles.iter().any(|t| t.active() && (t.kind == 5 || t.kind == 6)));
    }

    // --- DW-5.2: WetJungle --------------------------------------------------------

    /// DW-5.2: `SurfaceWaterInJungle` — exactly the 2 tiles above a scanned
    /// surface-grass column fill with water 255 (the water type); the scan
    /// breaks on the first active tile, so grass under an obstruction and
    /// columns whose first active is not 60 stay dry, and the window ends at
    /// `worldSurface − 1.0`.
    #[test]
    fn test_DW_5_2_wet_jungle_two_water_tiles() {
        let mut w = World::new("t".into(), 30, 60);
        // worldSurfaceLow = 20, worldSurface = 40 — the scan window [20, 39].
        let surface_low = 20.0f64;
        let world_surface = 40.0f64;
        // A clean grass column at (5, 30): water at 28/29, none at 27.
        // A grass column under a tree trunk at (8, 26..29): dry.
        // A column whose first active is stone at (11, 25): dry.
        // A grass column below the window at (14, 45): dry.
        // A grass column at the window start (17, 20): water at 18/19.
        set_kind(&mut w, 5, 30, TILE_JUNGLE_GRASS, true);
        set_kind(&mut w, 8, 30, TILE_JUNGLE_GRASS, true);
        for y in 26..=29 {
            set_kind(&mut w, 8, y, 5, true);
        }
        set_kind(&mut w, 11, 25, 1, true);
        set_kind(&mut w, 14, 45, TILE_JUNGLE_GRASS, true);
        set_kind(&mut w, 17, 20, TILE_JUNGLE_GRASS, true);

        pass_wet_jungle(&mut w, surface_low, world_surface);

        let water = |x: i32, y: i32| {
            w.tile(x, y)
                .map(|t| t.liquid == 255 && t.flags & FLAG_LIQUID_LAVA == 0)
                .unwrap_or(false)
        };
        assert!(water(5, 29) && water(5, 28), "the wet column");
        assert!(!water(5, 27), "exactly two");
        assert!(!water(8, 29) && !water(8, 28), "the tree column");
        assert!(!water(11, 24) && !water(11, 23), "the stone column");
        assert!(!water(14, 44) && !water(14, 43), "below the window");
        assert!(water(17, 19) && water(17, 18), "at the window start");
        // The grass anchors are untouched (no active flag on the water).
        assert!(!w.tile(5, 29).unwrap().active());
    }

    // --- DW-5.2: the plant arms ----------------------------------------------------

    /// DW-5.2: `PlaceTile`'s type-61 arm — the exact draw order
    /// (`Next(16)`, `Next(60)`, `Next(230)`, `Next(15)`, then the taken
    /// arm's tail) and the frame bands, for both the jungle-grass anchor
    /// (flag2 = j > rockLayer) and the temple-brick anchor (flag forced, no
    /// thorns/blooms); the below-gate failure draws nothing.
    #[test]
    fn test_DW_5_2_place_tile_jungle_plant_ledger() {
        // Below-gate failure: the below tile is dirt — no draws.
        let mut w = World::new("t".into(), 30, 40);
        let idx = w.index(10, 11);
        w.tiles[idx].kind = 2;
        w.tiles[idx].flags |= FLAG_ACTIVE;
        let mut rng = Rng(3);
        place_tile_jungle_plant(&mut w, 10, 10, 100.0, 20.0, &mut rng);
        assert_eq!(rng.0, Rng(3).0, "the below-gate failure draws nothing");

        // A grass anchor below the rock layer (j = 10 < 100 → flag2 false):
        // the thorns arm is blocked by the surface check, the blooms by
        // flag2 — only the flower band and the plain arm are live.
        let mut seen = std::collections::HashSet::new();
        for seed in 0..400u64 {
            let mut w = World::new("t".into(), 30, 40);
            set_kind(&mut w, 10, 11, TILE_JUNGLE_GRASS, true);
            let mut rng = Rng(seed);
            place_tile_jungle_plant(&mut w, 10, 10, 100.0, 20.0, &mut rng);
            // The replay: four unconditional draws, then the tail. flag2 =
            // (j > rockLayer) = false, so the 144/162 arms never fire.
            let mut replay = Rng(seed);
            let _r16 = replay.range(0, 16);
            let _r60 = replay.range(0, 60);
            let _r230 = replay.range(0, 230);
            let r15 = replay.range(0, 15);
            let (kind, fx) = if r15 == 0 {
                if replay.range(0, 3) != 0 {
                    (61u16, (replay.range(0, 2) * 18 + 108) as i16)
                } else {
                    (61, (replay.range(0, 13) * 18 + 180) as i16)
                }
            } else {
                (61, (replay.range(0, 6) * 18) as i16)
            };
            assert_eq!(rng.0, replay.0, "seed {seed} ledger");
            let t = w.tile(10, 10).unwrap();
            assert!(
                t.active() && t.kind == kind && t.frame_x == fx,
                "seed {seed}: got ({},{},{}) want ({kind},{fx})",
                t.active(),
                t.kind,
                t.frame_x
            );
            seen.insert(fx);
        }
        // The plain band (0) and the flower band (≥ 108) both occurred.
        assert!(seen.contains(&0) && seen.iter().any(|&fx| fx >= 108));
        // The blooms + thorns arm: an anchor ABOVE the rock layer and the
        // surface (j = 30 > 100? no — put rock_layer at 20): flag2 true.
        let mut seen_bloom = false;
        let mut hit_thorns = false;
        for seed in 0..2000u64 {
            let mut w = World::new("t".into(), 30, 40);
            set_kind(&mut w, 10, 31, TILE_JUNGLE_GRASS, true);
            let mut rng = Rng(seed);
            place_tile_jungle_plant(&mut w, 10, 30, 20.0, 20.0, &mut rng);
            let mut replay = Rng(seed);
            let r16 = replay.range(0, 16);
            // j = 30 > worldSurface 20 and > rockLayer 20 → all arms live.
            // The draws are lazy: the tail draws only when the earlier
            // arms are evaluated and not taken (the C# else-if chain).
            let expected = if r16 == 0 {
                (TILE_JUNGLE_THORNS, 0i16)
            } else {
                let r60 = replay.range(0, 60);
                if r60 == 0 {
                    (TILE_JUNGLE_PLANTS, 144)
                } else {
                    let r230 = replay.range(0, 230);
                    if r230 == 0 {
                        (TILE_JUNGLE_PLANTS, 162)
                    } else {
                        let r15 = replay.range(0, 15);
                        if r15 == 0 {
                            if replay.range(0, 3) != 0 {
                                (TILE_JUNGLE_PLANTS, (replay.range(0, 2) * 18 + 108) as i16)
                            } else {
                                (TILE_JUNGLE_PLANTS, (replay.range(0, 13) * 18 + 180) as i16)
                            }
                        } else {
                            (TILE_JUNGLE_PLANTS, (replay.range(0, 6) * 18) as i16)
                        }
                    }
                }
            };
            assert_eq!(rng.0, replay.0, "seed {seed} ledger");
            let t = w.tile(10, 30).unwrap();
            assert!(
                t.active() && t.kind == expected.0 && t.frame_x == expected.1,
                "seed {seed}: got ({},{},{}) want {:?}",
                t.active(),
                t.kind,
                t.frame_x,
                expected
            );
            if expected.0 == TILE_JUNGLE_THORNS {
                hit_thorns = true;
            }
            if expected.1 == 144 || expected.1 == 162 {
                seen_bloom = true;
            }
        }
        assert!(hit_thorns, "the thorns arm never fired in 2000 seeds");
        assert!(seen_bloom, "the bloom arms never fired in 2000 seeds");
    }

    /// DW-5.2: `TooManyJunglePlantsNearby` — the ±9×±5 box clamped into the
    /// world, counting active 61/74, true strictly above `maxCount`.
    #[test]
    fn test_DW_5_2_too_many_jungle_plants_nearby() {
        let mut w = World::new("t".into(), 40, 40);
        assert!(!too_many_jungle_plants_nearby(&w, 20, 20, 2));
        set_kind(&mut w, 18, 18, 61, true);
        set_kind(&mut w, 22, 19, TILE_JUNGLE_PLANTS2, true);
        assert!(
            !too_many_jungle_plants_nearby(&w, 20, 20, 2),
            "two plants == max, not too many"
        );
        set_kind(&mut w, 20, 16, 61, true);
        assert!(too_many_jungle_plants_nearby(&w, 20, 20, 2));
        // Out of the box (±10 columns away) does not count.
        let mut w2 = World::new("t".into(), 40, 40);
        set_kind(&mut w2, 5, 20, 61, true);
        set_kind(&mut w2, 5, 21, 61, true);
        set_kind(&mut w2, 5, 22, 61, true);
        assert!(!too_many_jungle_plants_nearby(&w2, 20, 20, 2));
        // The clamp: an anchor at the world edge still scans a valid box.
        assert!(!too_many_jungle_plants_nearby(&w2, 0, 0, 2));
    }

    /// DW-5.2: the plants pass arms — 60 grows 61 on every free cell
    /// (including below the surface), 226 draws `Next(5)` against the
    /// density gate, 70 keeps the shape-level mushroom, occupied cells stay
    /// untouched; the per-cell draw ledger replays the C# order and the
    /// placed tiles must match the replay's per-anchor outcomes.
    #[test]
    fn test_DW_5_2_mushroom_and_jungle_plants_arms() {
        let mut w = World::new("t".into(), 40, 80);
        // Grass with free cells above at 10/60 and 12/60 (below the surface).
        // Grass with a block above at 14/60. Mushroom grass at 16/60. Brick
        // 226 at 18/60. All within the sweep's [5, width-5) × [5, height-5).
        for (x, kind) in [(10u16, 60u16), (12, 60), (14, 60), (16, 70), (18, 226)] {
            let idx = w.index(x as i32, 60);
            w.tiles[idx].kind = kind;
            w.tiles[idx].flags |= FLAG_ACTIVE;
        }
        let idx = w.index(14, 59);
        w.tiles[idx].kind = 1;
        w.tiles[idx].flags |= FLAG_ACTIVE;
        let mut rng = Rng(0xC0FFEE);
        pass_mushroom_and_jungle_plants(&mut w, 50.0, 100.0, &mut rng);
        // The transcription: the pass draws only at the anchors — the
        // PlaceTile-61 calls at (10/12, 59) (four unconditional draws plus
        // the taken arm's tail; j = 59 > worldSurface 50 and < rockLayer
        // 100), the stone-blocked (14, 59) draws nothing, the mushroom arm
        // draws nothing, and the brick anchor draws Next(5) then, when the
        // gate passed (the density box holds ≤ the two earlier plants —
        // count 2 == max, not too many), the same PlaceTile draws.
        let plant_draws = |replay: &mut Rng| -> (u16, i16) {
            // Lazy draws: the tail draws only when the earlier arms are
            // evaluated and not taken (the C# else-if chain); j = 59 >
            // worldSurface 50 makes the thorns arm live, flag2 = 59 > 100
            // is false so the blooms can never fire.
            let r16 = replay.range(0, 16);
            if r16 == 0 && 59 > 50 {
                return (TILE_JUNGLE_THORNS, 0);
            }
            let r60 = replay.range(0, 60);
            if r60 == 0 && false {
                return (TILE_JUNGLE_PLANTS, 144);
            }
            let r230 = replay.range(0, 230);
            if r230 == 0 && false {
                return (TILE_JUNGLE_PLANTS, 162);
            }
            let r15 = replay.range(0, 15);
            if r15 == 0 {
                if replay.range(0, 3) != 0 {
                    (TILE_JUNGLE_PLANTS, (replay.range(0, 2) * 18 + 108) as i16)
                } else {
                    (TILE_JUNGLE_PLANTS, (replay.range(0, 13) * 18 + 180) as i16)
                }
            } else {
                (TILE_JUNGLE_PLANTS, (replay.range(0, 6) * 18) as i16)
            }
        };
        let mut replay = Rng(0xC0FFEE);
        let mut outcomes: Vec<(i32, u16, i16, bool)> = Vec::new(); // (x, kind, fx, placed)
        for i in 5..35 {
            for j in 5..75 {
                match (i, j) {
                    (10, 60) | (12, 60) => {
                        let (kind, fx) = plant_draws(&mut replay);
                        outcomes.push((i, kind, fx, true));
                    }
                    (18, 60) => {
                        let gate = replay.range(0, 5) == 0;
                        if gate {
                            let (kind, fx) = plant_draws(&mut replay);
                            outcomes.push((i, kind, fx, true));
                        } else {
                            outcomes.push((i, 0, 0, false));
                        }
                    }
                    _ => {}
                }
            }
        }
        assert_eq!(rng.0, replay.0, "the pass ledger");
        // The placed tiles match the replay's per-anchor decisions.
        for (x, kind, fx, placed) in &outcomes {
            let t = w.tile(*x, 59).unwrap();
            if *placed {
                assert!(
                    t.active() && t.kind == *kind && t.frame_x == *fx,
                    "column {x}: got ({},{},{}) want ({kind},{fx})",
                    t.active(),
                    t.kind,
                    t.frame_x
                );
            } else {
                assert!(!t.active(), "column {x} must stay empty");
            }
        }
        // The blocked cell stayed stone.
        assert_eq!(w.tile(14, 59).unwrap().kind, 1);
        // The mushroom arm kept the shape-level placement.
        let t = w.tile(16, 59).unwrap();
        assert!(t.active() && t.kind == 71 && t.frame_x == 0);
        // Both grass anchors placed something (61 or the rare 69).
        assert!(outcomes.iter().any(|&(x, _, _, p)| x == 10 && p));
    }

    // --- DW-5.2: JunglePlantsPart2 -------------------------------------------------

    /// The grass-row setter for the detritus fixtures.
    fn grass_row(w: &mut World, x0: i32, x1: i32, y: i32) {
        for x in x0..=x1 {
            set_kind(w, x, y, TILE_JUNGLE_GRASS, true);
        }
    }

    /// DW-5.2: `PlaceJunglePlant` — the 3×2 (styleY 0) and 2×2 (styleY 1)
    /// frames, the overlay scan (foreign actives block, allowed plants do
    /// not), the solid-jungle-grass ground check and the bounds. The box
    /// rows for an anchor at y=29 are 28/29; the ground row 30 stays grass.
    #[test]
    fn test_DW_5_2_place_jungle_plant_detritus() {
        // 3×2 success: ground at y=30 across 19..=21, anchor (20, 29).
        let mut w = World::new("t".into(), 40, 40);
        grass_row(&mut w, 19, 21, 30);
        place_jungle_plant_detritus(&mut w, 20, 29, TILE_PLANT_DETRITUS, 2, 0);
        for (dx, fx) in [(-1i32, 108i16), (0, 126), (1, 144)] {
            let t = w.tile(20 + dx, 28).unwrap();
            assert!(t.active() && t.kind == 233 && t.frame_x == fx && t.frame_y == 0);
            let t = w.tile(20 + dx, 29).unwrap();
            assert!(t.active() && t.kind == 233 && t.frame_x == fx && t.frame_y == 18);
        }
        assert_eq!(w.tile(20, 30).unwrap().kind, TILE_JUNGLE_GRASS);
        // A foreign active in the box blocks.
        let mut w = World::new("t".into(), 40, 40);
        grass_row(&mut w, 19, 21, 30);
        set_kind(&mut w, 20, 29, 1, true);
        place_jungle_plant_detritus(&mut w, 20, 29, TILE_PLANT_DETRITUS, 0, 0);
        assert_eq!(w.tile(20, 29).unwrap().kind, 1);
        // An allowed plant in the box does not block (61 is tolerated, then
        // overwritten).
        let mut w = World::new("t".into(), 40, 40);
        grass_row(&mut w, 19, 21, 30);
        set_kind(&mut w, 20, 29, 61, true);
        place_jungle_plant_detritus(&mut w, 20, 29, TILE_PLANT_DETRITUS, 0, 0);
        assert_eq!(w.tile(20, 29).unwrap().kind, 233);
        // The ground check: one column not on jungle grass blocks.
        let mut w = World::new("t".into(), 40, 40);
        grass_row(&mut w, 19, 20, 30); // column 21 is bare dirt-wall air
        place_jungle_plant_detritus(&mut w, 20, 29, TILE_PLANT_DETRITUS, 0, 0);
        assert!(!w.tile(20, 29).unwrap().active());
        // 2×2 (styleY 1): frames 36·styleX / +18 columns, rows 36/54.
        let mut w = World::new("t".into(), 40, 40);
        grass_row(&mut w, 19, 20, 30);
        place_jungle_plant_detritus(&mut w, 20, 29, TILE_PLANT_DETRITUS, 3, 1);
        let t = w.tile(19, 28).unwrap();
        assert!(t.active() && t.kind == 233 && t.frame_x == 108 && t.frame_y == 36);
        let t = w.tile(20, 28).unwrap();
        assert!(t.active() && t.kind == 233 && t.frame_x == 126 && t.frame_y == 36);
        let t = w.tile(19, 29).unwrap();
        assert!(t.active() && t.kind == 233 && t.frame_x == 108 && t.frame_y == 54);
        let t = w.tile(20, 29).unwrap();
        assert!(t.active() && t.kind == 233 && t.frame_x == 126 && t.frame_y == 54);
        // Bounds.
        let mut w = World::new("t".into(), 40, 40);
        grass_row(&mut w, 3, 5, 30);
        place_jungle_plant_detritus(&mut w, 4, 29, TILE_PLANT_DETRITUS, 0, 0);
        assert!(!w.tile(4, 29).unwrap().active());
    }

    /// DW-5.2: the detritus sweep — the per-attempt draw ledger
    /// (`Next(40, w/2−40)`, `Next(h−300)`, the walk, `Next(8)`, the retry
    /// `Next(12)` when the first attempt left no 233), then the exact
    /// 233-cell map. The transcription runs on the fixture's own state
    /// (the grass strip) plus the placements it has replayed so far — the
    /// pass's own writes are what later walks and box scans see.
    #[test]
    fn test_DW_5_2_jungle_plants_part2_ledger() {
        // A 200×420 world with a grass strip across 44..=51 at y=100 (the
        // walk cap is 420 − 300 = 120, so the strip must sit above it): the
        // walk-down from an air start lands on it; anchors x ∈ [45, 50]
        // pass the 3×2 ground check, x = 51 only the 2×2 retry, x = 44
        // nothing (column 43 is bare).
        let mut w = World::new("t".into(), 200, 420);
        grass_row(&mut w, 44, 51, 100);
        let mut rng = Rng(0xBEEF);
        pass_jungle_plants_part2(&mut w, &mut rng);

        // The transcription over the same stream. The fixture's only base
        // actives are the strip cells; the replayed placements add 233s.
        let strip = |x: i32, j: i32| j == 100 && (44..=51).contains(&x);
        let mut replay = Rng(0xBEEF);
        let mut expected: std::collections::BTreeSet<(i32, i32)> = Default::default();
        let landing_kind = |x: i32, j: i32, placed: &std::collections::BTreeSet<(i32, i32)>| {
            if placed.contains(&(x, j)) {
                TILE_PLANT_DETRITUS
            } else if strip(x, j) {
                TILE_JUNGLE_GRASS
            } else {
                0u16
            }
        };
        for _ in 0..(200 * 100) {
            let x = replay.range(40, 200 / 2 - 40);
            let mut j = replay.range(0, 420 - 300);
            while landing_kind(x, j, &expected) == 0 && j < 420 - 300 {
                j += 1;
            }
            // The C# landing check: active && type == 60. A placed 233 or
            // the cap (air) stops the walk short of the grass.
            if landing_kind(x, j, &expected) != TILE_JUNGLE_GRASS {
                continue;
            }
            let j = j - 1;
            let _style = replay.range(0, 8);
            // The 3×2 scan: the box rows j−1/j must hold no active tile
            // (233 is not in the allowed set) and the ground row j+1 must
            // be grass across the three columns.
            let clear = |x0: i32, x1: i32| {
                (x0..=x1).all(|cx| {
                    !expected.contains(&(cx, j - 1))
                        && !expected.contains(&(cx, j))
                        && !strip(cx, j - 1)
                        && !strip(cx, j)
                })
            };
            let ground = |x0: i32, x1: i32| (x0..=x1).all(|cx| strip(cx, j + 1));
            if ground(x - 1, x + 1) && clear(x - 1, x + 1) {
                for cx in (x - 1)..=(x + 1) {
                    expected.insert((cx, j - 1));
                    expected.insert((cx, j));
                }
            } else if !expected.contains(&(x, j)) {
                // The C# retry gate `.type != 233` (`:20185`): an anchor
                // already holding a 233 from an overlapping earlier
                // placement suppresses the retry draw.
                let _style2 = replay.range(0, 12);
                if ground(x - 1, x) && clear(x - 1, x) {
                    for cx in (x - 1)..=x {
                        expected.insert((cx, j - 1));
                        expected.insert((cx, j));
                    }
                }
            }
        }
        assert_eq!(rng.0, replay.0, "the detritus ledger");
        // The world's 233 map equals the transcription's placements.
        let mut world_detritus: std::collections::BTreeSet<(i32, i32)> = Default::default();
        for x in 0..200 {
            for y in 0..420 {
                if w.tile(x, y).is_some_and(|t| t.active() && t.kind == TILE_PLANT_DETRITUS) {
                    world_detritus.insert((x, y));
                }
            }
        }
        assert_eq!(world_detritus, expected);
        assert!(!expected.is_empty(), "no detritus placed");
    }

    // The replay helpers — read-only mirrors the test drives the world with.
    fn replay_tile_active(w: &World, x: i32, y: i32) -> bool {
        w.tile(x, y).is_some_and(|t| t.active())
    }
    fn replay_tile_kind(w: &World, x: i32, y: i32) -> Option<u16> {
        w.tile(x, y).map(|t| t.kind)
    }
    /// The detritus box scan's ground arm: every column sits on jungle grass.
    fn detritus_ground(w: &World, x0: i32, x1: i32, ground_row: i32) -> bool {
        (x0..=x1).all(|cx| replay_tile_kind(w, cx, ground_row) == Some(TILE_JUNGLE_GRASS))
    }
    /// The detritus box scan's overlay arm: the two box rows hold no active
    /// tile (the fixture has no allowed plants).
    fn detritus_clear(w: &World, x0: i32, x1: i32, top_row: i32) -> bool {
        (x0..=x1).all(|cx| {
            !(replay_tile_active(w, cx, top_row) || replay_tile_active(w, cx, top_row + 1))
        })
    }

    // --- DW-5.2: MudsWallsInJungle --------------------------------------------------

    /// DW-5.2: the wall conversion — the min/max surface-60 scans, walls
    /// 2/59 → 15 inside the band with the ragged edge draws (replayed),
    /// walls 1/64 and out-of-band walls untouched, and the bounds returned
    /// for `GenVars`. With a 21-wide band the gate geometry is exact:
    /// columns 20/21/39/40 draw `Next(2)`+`Next(3)`, 22/38 draw `Next(3)`
    /// only, 23..=37 sit inside both ranges and draw nothing; every
    /// converted cell must carry wall 15, every skipped cell its original
    /// wall.
    #[test]
    fn test_DW_5_2_muds_walls_in_jungle() {
        let mut w = World::new("t".into(), 60, 60);
        let world_surface = 20.0f64;
        // Grass columns 20..=40 at y=10; walls 2/59 across 19..=41 in the
        // band rows, the stone/jungle-wall controls outside, wall 2 below
        // the band as the out-of-band control.
        for x in 20..=40 {
            let idx = w.index(x, 10);
            w.tiles[idx].kind = TILE_JUNGLE_GRASS;
            w.tiles[idx].flags |= FLAG_ACTIVE;
        }
        for x in 19..=41 {
            for y in 0..40 {
                tile_mut(&mut w, x, y).wall = if (x + y) % 5 == 0 {
                    WALL_CAVE6_UNSAFE
                } else {
                    WALL_DIRT
                };
            }
        }
        for y in 0..40 {
            tile_mut(&mut w, 19, y).wall = 1; // stone wall — untouched
            tile_mut(&mut w, 42, y).wall = 64; // jungle wall — untouched
        }
        for y in 40..60 {
            tile_mut(&mut w, 30, y).wall = WALL_DIRT; // below the band
        }

        let mut rng = Rng(0x11);
        let (min_x, max_x) = pass_muds_walls_in_jungle(&mut w, world_surface, &mut rng);
        assert_eq!((min_x, max_x), (20, 40), "the scan bounds");

        // The transcription of the conversion sweep: per (l, m) the two
        // short-circuit gates (`(inner) || Next(…)` — the draw only when the
        // range disjunct is false) and the wall test.
        let rows = 40; // j < 20.0 + 20.0
        let mut replay = Rng(0x11);
        let mut converted: std::collections::BTreeSet<(i32, i32)> = Default::default();
        for l in min_x..=max_x {
            for m in 0..rows {
                let inner2 = l >= min_x + 2 && l <= max_x - 2;
                if !(inner2 || replay.range(0, 2) != 0) {
                    continue;
                }
                let inner3 = l >= min_x + 3 && l <= max_x - 3;
                if !(inner3 || replay.range(0, 3) != 0) {
                    continue;
                }
                converted.insert((l, m));
            }
        }
        assert_eq!(rng.0, replay.0, "the edge-draw ledger");
        // The doubly-inner columns pass both gates through the range
        // disjuncts — no draws (proven by the ledger), full conversion.
        let inner_cols: Vec<i32> = ((min_x + 3)..=(max_x - 3))
            .filter(|&l| l >= min_x + 2 && l <= max_x - 2)
            .collect();
        assert!(!inner_cols.is_empty());
        for l in &inner_cols {
            for m in 0..rows {
                assert!(converted.contains(&(*l, m)), "inner column {l} row {m}");
            }
        }
        // Every cell matches the transcription.
        for l in min_x..=max_x {
            for m in 0..rows {
                let original = if (l + m) % 5 == 0 {
                    WALL_CAVE6_UNSAFE
                } else {
                    WALL_DIRT
                };
                let wall = w.tile(l, m).unwrap().wall;
                if converted.contains(&(l, m)) {
                    assert_eq!(wall, WALL_MUD_UNSAFE, "column {l} row {m}");
                } else {
                    assert_eq!(wall, original, "column {l} row {m} kept");
                }
            }
        }
        // The controls are untouched.
        for y in 0..40 {
            assert_eq!(w.tile(19, y).unwrap().wall, 1);
            assert_eq!(w.tile(42, y).unwrap().wall, 64);
        }
        for y in 40..60 {
            assert_eq!(w.tile(30, y).unwrap().wall, WALL_DIRT);
        }
    }

    /// DW-5.2: the no-jungle case — the scans leave the locals at the C#'s
    /// 0/0 (not the GenVars −1 reset), the bounds come back 0/0, and the
    /// conversion sweep still runs its gates on column 0.
    #[test]
    fn test_DW_5_2_muds_walls_no_jungle_sentinel() {
        let mut w = World::new("t".into(), 40, 60);
        for x in 0..40 {
            for y in 0..60 {
                tile_mut(&mut w, x, y).wall = WALL_DIRT;
            }
        }
        let mut rng = Rng(5);
        let (min_x, max_x) = pass_muds_walls_in_jungle(&mut w, 20.0, &mut rng);
        assert_eq!((min_x, max_x), (0, 0), "the 0/0 sentinel");
        // Column 0: both range disjuncts false → Next(2) and (when it
        // passes) Next(3) draw per row; the row converts on double pass.
        let mut replay = Rng(5);
        let mut converted = Vec::new();
        for m in 0..40 {
            if replay.range(0, 2) == 0 {
                continue;
            }
            if replay.range(0, 3) == 0 {
                continue;
            }
            converted.push(m);
        }
        assert_eq!(rng.0, replay.0, "the column-0 gate ledger");
        for m in 0..40 {
            let wall = w.tile(0, m).unwrap().wall;
            if converted.contains(&m) {
                assert_eq!(wall, WALL_MUD_UNSAFE, "column 0 row {m}");
            } else {
                assert_eq!(wall, WALL_DIRT, "column 0 row {m} kept");
            }
        }
    }
}
