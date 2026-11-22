//! The jungle-plant placement helpers of the grass-growth dispatcher
//! (W4-g42 dispatcher follow-up D3): `WorldGen.PlaceJunglePlant`
//! (`WorldGen.cs:47779-47944`) and `TooManyJunglePlantsNearby`
//! (`:73164-73187`). The runtime consumers are the underground tail switch
//! (the Plantera bulb 238 / life fruit 236 / jungle rose 233 arms,
//! `:74416-74495`) and the overworld jungle case-60 rose arm (`:74773-74791`).
//!
//! Deviations (ANALYSIS): the `SquareTileFrame` re-frames the callers pair
//! with the placement are the framer's job (the T-3 precedent); the
//! `netMode == 2` sends are MP's. `KillTile(fail:false, effectOnly:false,
//! noItem:true)` rides the port's RNG-free `edit::kill_tile` (the edit.rs
//! no-rng precedent).

use crate::tile::FLAG_ACTIVE;
use crate::world::World;

/// `WorldGen.TooManyJunglePlantsNearby` (`WorldGen.cs:73164-73187`): the
/// ±9×±5 clamped box counts the jungle plants 61/74; more than `max_count`
/// (default 2) refuses. RNG-free.
pub(super) fn too_many_jungle_plants_nearby(world: &World, i: i32, j: i32, max_count: i32) -> bool {
    let lo_x = (i - 9).clamp(10, world.width as i32 - 1 - 10);
    let hi_x = (i + 9).clamp(10, world.width as i32 - 1 - 10);
    let lo_y = (j - 5).clamp(10, world.height as i32 - 1 - 10);
    let hi_y = (j + 5).clamp(10, world.height as i32 - 1 - 10);
    let mut count = 0;
    for k in lo_x..=hi_x {
        for l in lo_y..=hi_y {
            // The C# `Main.tile[k, l]` OOB read throws inside the growth
            // tick's try/catch; the port's bounds check skips the cell.
            if k < 0 || l < 0 || k >= world.width as i32 || l >= world.height as i32 {
                continue;
            }
            if world
                .tile(k, l)
                .is_some_and(|t| t.active() && matches!(t.kind, 61 | 74))
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

/// `WorldGen.PlaceJunglePlant` (`WorldGen.cs:47779-47944`): the jungle rose
/// 233 (3×2, `styleY == 0`) or the 2×2 plants (236 life fruit / 238 Plantera
/// bulb / 702, or any `styleY > 0` rose). The box must be clear of everything
/// except the jungle-plant family (61/703/62/655/69/74, the 185 pile at
/// frameY 0, and the 233 rose for the 236/238/702 replacements) and stand on
/// solid jungle grass 60 under EVERY column; the clear cells are killed
/// (`noItem`) and the object written with the 36-px (2×2) or 54-px (3×2)
/// style columns. `inherit_paint` copies the soil cell's paint/coating onto
/// every object cell. RNG-free — the style draws ride the callers.
pub(super) fn place_jungle_plant(
    world: &mut World,
    x2: i32,
    y2: i32,
    kind: u16,
    style_x: i32,
    style_y: i32,
    inherit_paint: bool,
) {
    if style_y > 0 || matches!(kind, 236 | 238 | 702) {
        // The 2×2 arm (`:47781-47864`). The 95/126 row bump never fires for
        // the dispatcher's kinds (233/236/238), kept for completeness.
        let mut num = y2;
        if matches!(kind, 95 | 126) {
            num += 1;
        }
        if x2 < 5 || x2 > world.width as i32 - 5 || num < 5 || num > world.height as i32 - 5 {
            return;
        }
        let mut flag = true;
        for i in (x2 - 1)..(x2 + 1) {
            for j in (num - 1)..(num + 1) {
                if let Some(t) = world.tile(i, j) {
                    if t.active()
                        && !matches!(t.kind, 61 | 703 | 62 | 655 | 69 | 74)
                        && !(kind == 236 && t.kind == 233)
                        && !(kind == 702 && t.kind == 233)
                        && !(kind == 238 && t.kind == 233)
                        && !(t.kind == 185 && t.frame_y == 0)
                    {
                        flag = false;
                    }
                    // `:47806-47809`: the 98 DyeVat liquid veto (not a
                    // dispatcher kind, kept verbatim).
                    if kind == 98 && t.liquid > 0 {
                        flag = false;
                    }
                }
            }
            // `:47815-47818`: solid jungle grass under the column.
            if !crate::edit::solid_tile(world, i, num + 1)
                || !world.tile(i, num + 1).is_some_and(|t| t.kind == 60)
            {
                flag = false;
            }
        }
        if !flag {
            return;
        }
        // `:47824-47833`: the kill sweep (noItem).
        for k in (x2 - 1)..(x2 + 1) {
            for l in (num - 1)..(num + 1) {
                if world.tile(k, l).is_some_and(|t| t.active()) {
                    let _ = crate::edit::kill_tile(world, k, l, true);
                }
            }
        }
        // `:47834-47855`: the 2×2 write — frameX 36·styleX (+18 right),
        // frameY 36 (+18 bottom), 0 for the 236/238/702 single-row plants.
        let num2 = (36 * style_x) as i16;
        let num3: i16 = if matches!(kind, 236 | 238 | 702) { 0 } else { 36 };
        for (dx, dy) in [(0i32, 0i32), (1, 0), (0, 1), (1, 1)] {
            let idx = world.index(x2 - 1 + dx, num - 1 + dy);
            world.tiles[idx].flags |= FLAG_ACTIVE;
            world.tiles[idx].kind = kind;
            world.tiles[idx].frame_x = num2 + (dx as i16) * 18;
            world.tiles[idx].frame_y = num3 + (dy as i16) * 18;
        }
        boot_frame(world, kind);
        if inherit_paint {
            copy_soil_paint(world, x2, num + 1, &[(x2 - 1, num - 1), (x2, num - 1), (x2 - 1, num), (x2, num)]);
        }
    } else {
        // The 3×2 arm (`:47865-47943`).
        if x2 < 5 || x2 > world.width as i32 - 5 || y2 < 5 || y2 > world.height as i32 - 5 {
            return;
        }
        let mut flag2 = true;
        for m in (x2 - 1)..(x2 + 2) {
            for n in (y2 - 1)..(y2 + 1) {
                if let Some(t) = world.tile(m, n) {
                    if t.active()
                        && !matches!(t.kind, 61 | 703 | 62 | 69 | 655 | 74)
                        && !(t.kind == 185 && t.frame_y == 0)
                    {
                        flag2 = false;
                    }
                }
            }
            if !crate::edit::solid_tile(world, m, y2 + 1)
                || !world.tile(m, y2 + 1).is_some_and(|t| t.kind == 60)
            {
                flag2 = false;
            }
        }
        if !flag2 {
            return;
        }
        for num4 in (x2 - 1)..(x2 + 2) {
            for num5 in (y2 - 1)..(y2 + 1) {
                if world.tile(num4, num5).is_some_and(|t| t.active()) {
                    let _ = crate::edit::kill_tile(world, num4, num5, true);
                }
            }
        }
        // `:47908-47932`: the 3×2 write — frameX 54·styleX + 18·column,
        // frameY 0/18.
        let num6 = (54 * style_x) as i16;
        for (dx, dy) in [(0i32, 0i32), (1, 0), (2, 0), (0, 1), (1, 1), (2, 1)] {
            let idx = world.index(x2 - 1 + dx, y2 - 1 + dy);
            world.tiles[idx].flags |= FLAG_ACTIVE;
            world.tiles[idx].kind = kind;
            world.tiles[idx].frame_x = num6 + (dx as i16) * 18;
            world.tiles[idx].frame_y = (dy as i16) * 18;
        }
        boot_frame(world, kind);
        if inherit_paint {
            copy_soil_paint(
                world,
                x2,
                y2 + 1,
                &[
                    (x2 - 1, y2 - 1),
                    (x2, y2 - 1),
                    (x2 + 1, y2 - 1),
                    (x2 - 1, y2),
                    (x2, y2),
                    (x2 + 1, y2),
                ],
            );
        }
    }
}

/// `CopyPaintAndCoating(other)` over the object cells (`:47856-47863` /
/// `:47933-47942`) — the soil cell under the centre column is the source.
fn copy_soil_paint(world: &mut World, sx: i32, sy: i32, cells: &[(i32, i32)]) {
    let (meta, extras) = {
        let si = world.index(sx, sy);
        (world.tiles[si].meta, world.extras_at(si))
    };
    for &(x, y) in cells {
        let di = world.index(x, y);
        world.tiles[di].meta = meta;
        world.set_extras_at(di, extras);
    }
}

/// The frame-important boot bit (the growth.rs plant-write precedent).
fn boot_frame(world: &mut World, kind: u16) {
    if world.frame_important.len() <= kind as usize {
        world.frame_important.resize(crate::tiledata::TILE_COUNT, false);
    }
    world.frame_important[kind as usize] = true;
}
