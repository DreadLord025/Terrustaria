//! The `Traps` pass (`WorldGen.cs:18657-18786`, W4-d104 pass 34) +
//! `placeTrap` (`:8922-9513`, the dart/boulder/explosive/geyser cases) +
//! `PlaceSandTrap` (`:35463-35735`) + the boulder-proximity helpers
//! (`:8806-8860`).
//!
//! The pass scatters `width * 0.05` traps over the underground (the
//! `getGoodWorldGen` ×1.5), each a 1150-attempt retry onto a wall-less
//! cell below the surface outside the ocean band, then `width * 0.003`
//! sand traps onto the wall-187 desert columns. The `tileSolid[379]`
//! geyser window rides the predicates (the temple `spike_solid`
//! precedent); the `placingTraps` flag has no port-side consumer.
//!
//! Deviations (ANALYSIS "Traps (W4-d104)"): the noTraps seed arms (the
//! ×100/×5/×2 counts, the TNT barrels, the boulder-solidity flips, the
//! anniversary type-1 override) are the seed lane's (d107) — the classic
//! arm ports verbatim; the `starGame` multiplier is unported (no star
//! game in the port); the shimmer-position veto reads the port's
//! `GenVars.shimmer_position` (Zero until the shimmer lane publishes).

use super::Rng;
use crate::tile::{LiquidKind, Tile, FLAG_ACTIVE, FLAG_ACTUATOR, FLAG_HALF_BRICK};
use crate::tiledata as td;
use crate::world::World;

/// `TileID.Sets.Boulders` — the boulder family the `SetBoulderSolidity`
/// flips and the nearby-scans veto (`TileID.cs`).
fn is_boulder(kind: u16) -> bool {
    td::SET_BOULDERS.has(kind)
}

/// `WorldGen.SolidTile` (`WorldGen.cs:70111-70132`) under the Traps
/// window: `tileSolid[379]` (the geyser) is false for the whole pass
/// (`:18661`, restored `:18785`); a null tile reads solid.
fn solid_trap(world: &World, x: i32, y: i32) -> bool {
    let Some(t) = world.tile(x, y) else {
        return true;
    };
    if t.kind == 379 {
        return false;
    }
    t.active()
        && td::TILE_SOLID.has(t.kind)
        && !td::TILE_SOLID_TOP.has(t.kind)
        && !t.half_brick()
        && t.slope() == 0
        && !t.inactive()
}

/// `WorldGen.SolidTile2` (`WorldGen.cs:70134-70151`) under the window.
fn solid2_trap(world: &World, x: i32, y: i32) -> bool {
    let Some(t) = world.tile(x, y) else {
        return true;
    };
    if t.kind == 379 {
        return false;
    }
    t.active() && td::TILE_SOLID.has(t.kind) && t.slope() == 0 && !t.half_brick() && !t.inactive()
}

/// `WorldGen.SolidTileAllowBottomSlope` (`WorldGen.cs:70167-70190`)
/// under the window.
fn solid_allow_bottom_slope_trap(world: &World, x: i32, y: i32) -> bool {
    let Some(t) = world.tile(x, y) else {
        return true;
    };
    if t.kind == 379 {
        return false;
    }
    t.active()
        && (td::TILE_SOLID.has(t.kind) || td::TILE_SOLID_TOP.has(t.kind))
        && (!t.top_slope() || td::SET_PLATFORMS.has(t.kind))
        && !t.half_brick()
        && !t.inactive()
}

/// `WorldGen.placeTrap_CanContinue` (`WorldGen.cs:8909-8920`): the
/// wall-350 veto (the dual-dungeon bounds half is the seed lane's).
fn trap_can_continue(world: &World, x: i32, y: i32) -> bool {
    world.tile(x, y).map(|t| t.wall).unwrap_or(0) != 350
}

/// `WorldGen.IsTileNearby` (`WorldGen.cs:8862-8884`).
fn is_tile_nearby(world: &World, x: i32, y: i32, kind: u16, distance: i32) -> bool {
    let step = if kind == 235 { 3 } else { 1 };
    let mut i = x - distance;
    while i <= x + distance {
        for j in (y - distance)..=(y + distance) {
            if world.tile(i, j).is_some_and(|t| t.active() && t.kind == kind) {
                return true;
            }
        }
        i += step;
    }
    false
}

/// `WorldGen.AreAnyTilesInSetNearby` (`WorldGen.cs:8787-8804`) over the
/// boulder set.
fn boulder_nearby(world: &World, x: i32, y: i32, distance: i32) -> bool {
    for i in (x - distance)..=(x + distance) {
        for j in (y - distance)..=(y + distance) {
            if world.tile(i, j).is_some_and(|t| t.active() && is_boulder(t.kind)) {
                return true;
            }
        }
    }
    false
}

/// `WorldGen.CanGeneratePressurePlateAt` (`WorldGen.cs:11029-11049`).
fn can_pressure_plate_at(world: &World, i: i32, j: i32) -> bool {
    if i < 3 || j < 3 || i >= world.width - 3 || j >= world.height - 3 {
        return false;
    }
    if !solid_allow_bottom_slope_trap(world, i, j + 1) {
        return false;
    }
    let below = world.tile(i, j + 1).map(|t| (t.kind, t.wall)).unwrap_or((0, 0));
    if is_boulder(below.0) || below.1 == 350 {
        return false;
    }
    true
}

/// `WorldGen.closeEnoughToDungeonToSpawnGhoulder` (`WorldGen.cs:8806-8828`):
/// the ±1000 dungeon-wall scan on the dungeon's side (the port's dungeon
/// side is fixed Right — M5-BIOME).
fn close_to_dungeon(world: &World, x: i32, y: i32) -> bool {
    // The port pins the dungeon Right, so the Left-side veto never fires
    // and the Right-side veto reads `x < width / 2`.
    if x < world.width / 2 {
        return false;
    }
    let num = 1000;
    let mut i = x - num;
    while i <= x + num {
        let mut j = y - num;
        while j <= y + num {
            if let Some(t) = world.tile(i, j) {
                if crate::scene::is_dungeon_wall(t.wall) {
                    return true;
                }
            }
            j += 10;
        }
        i += 10;
    }
    false
}

/// `WorldGen.closeEnoughToSpidersToSpawnSpiderBoulder`
/// (`WorldGen.cs:8830-8844`): the ±80 wall-62 scan.
fn close_to_spiders(world: &World, x: i32, y: i32) -> bool {
    let num = 80;
    let mut i = x - num;
    while i <= x + num {
        let mut j = y - num;
        while j <= y + num {
            if world.tile(i, j).is_some_and(|t| t.wall == 62) {
                return true;
            }
            j += 3;
        }
        i += 3;
    }
    false
}

/// `WorldGen.closeEnoughToLavaToSpawnLavaBoulder`
/// (`WorldGen.cs:8846-8860`): the ±60 lava scan.
fn close_to_lava(world: &World, x: i32, y: i32) -> bool {
    let num = 60;
    let mut i = x - num;
    while i <= x + num {
        let mut j = y - num;
        while j <= y + num {
            if world
                .tile(i, j)
                .is_some_and(|t| t.liquid > 0 && t.liquid_kind() == LiquidKind::Lava)
            {
                return true;
            }
            j += 3;
        }
        i += 3;
    }
    false
}

/// `WorldGen.oceanDepths` (`WorldGen.cs:10963-10973`) — the trap
/// scatter's ocean re-roll. Shared with the Piles pass (W4-d104 pass 35).
pub(crate) fn ocean_depths_trap(world: &World, x: i32, y: i32, ocean_level: f64) -> bool {
    if y as f64 > ocean_level {
        return false;
    }
    x < 380 || x > world.width - 380
}

/// `WorldGen.Place1x1` (`WorldGen.cs:45188-45233`) — the mechanism /
/// plate write: the `SolidTile2` seat + clear-cell gate (the
/// mechanism-over-boulder veto), then the frame write (the default
/// `frameY = style * 18`; the 36/144/239 `frameX = style * 18` arm).
fn place_1x1(world: &mut World, x: i32, y: i32, kind: u16, style: i32) {
    if x < 2 || y < 2 || x >= world.width - 2 || y >= world.height - 2 {
        return;
    }
    let below_kind = world.tile(x, y + 1).map(|t| t.kind).unwrap_or(0);
    let mechanism_veto = td::SET_IS_A_MECHANISM.has(kind) && is_boulder(below_kind);
    if mechanism_veto || !solid2_trap(world, x, y + 1) || world.tile(x, y).is_some_and(|t| t.active())
    {
        return;
    }
    if let Some(t) = world.tile_mut(x, y) {
        t.flags |= FLAG_ACTIVE;
        t.kind = kind;
        if matches!(kind, 36 | 144 | 239) {
            t.frame_x = (style * 18) as i16;
            t.frame_y = 0;
        } else {
            t.frame_y = (style * 18) as i16;
        }
    }
}

/// `WorldGen.Place2x2` (`WorldGen.cs:51330-51420`) — the boulder write:
/// the ±5 bounds, the 2×2 clear scan, the `nactive && (SolidTile2 ||
/// tileTable)` floor under both columns, then the frame grid at
/// `(x-1, y-1)` (`frameX = k*18`, `frameY = style*36 + l*18`). The
/// 95/126 sink and 132/172 arms stay with their callers.
fn place_2x2(world: &mut World, x: i32, y: i32, kind: u16, style: i32) {
    if x < 5 || x > world.width - 5 || y < 5 || y > world.height - 5 {
        return;
    }
    for i in (x - 1)..(x + 1) {
        for j in (y - 1)..(y + 1) {
            if world.tile(i, j).is_some_and(|t| t.active()) {
                return;
            }
        }
        let below = world.tile(i, y + 1);
        let ok = below.is_some_and(|t| {
            t.active() && !t.inactive() && (solid2_trap(world, i, y + 1) || crate::tileobject::is_table(t.kind))
        });
        if !ok {
            return;
        }
    }
    let (ox, oy) = (x - 1, y - 1);
    for k in 0..2 {
        for l in 0..2 {
            if let Some(t) = world.tile_mut(ox + k, oy + l) {
                t.flags |= FLAG_ACTIVE;
                t.frame_x = (k * 18) as i16;
                t.frame_y = (style * 36 + l * 18) as i16;
                t.kind = kind;
            }
        }
    }
}

/// `WorldGen.KillTile` during generation — the port's `edit::kill_tile`
/// with `no_item: true` (the C# forces noItem while generating,
/// `:63545-63548`).
fn kill(world: &mut World, x: i32, y: i32) {
    crate::edit::kill_tile(world, x, y, true);
}

/// The red-wire walk between the plate and the trap
/// (`placeTrap`'s three identical loops, `:9186-9204` etc.): x first,
/// then y, a wire bit on every visited cell.
fn wire_walk(world: &mut World, x2: i32, num: i32, tx: i32, ty: i32) {
    let (mut wx, mut wy) = (x2, num);
    while wx != tx || wy != ty {
        world.wires.set(wx, wy, crate::wiring::WireColor::Red, true);
        if wx > tx {
            wx -= 1;
        }
        if wx < tx {
            wx += 1;
        }
        world.wires.set(wx, wy, crate::wiring::WireColor::Red, true);
        if wy > ty {
            wy -= 1;
        }
        if wy < ty {
            wy += 1;
        }
        world.wires.set(wx, wy, crate::wiring::WireColor::Red, true);
    }
}

/// `WorldGen.placeTrap` (`WorldGen.cs:8922-9513`) — the shared head
/// (the can-continue gate, the shimmer veto, the drop scan with the
/// near-underworld latch, the boulder-nearby / wall / lava gates, the
/// type roll, the 3×3 clear box, the spike-below veto, the pressure-
/// plate gate) then the four cases. `type` starts −1 (the pass's
/// argument); the classic type roll is the non-remix arm (`:8988-9004`).
#[allow(clippy::too_many_arguments)]
fn place_trap(
    world: &mut World,
    x2: i32,
    y2: i32,
    world_surface: f64,
    _rock_layer: f64,
    lava_line: i32,
    shimmer: (i32, i32),
    boulder_pets: &mut i32,
    rng: &mut Rng,
) -> bool {
    if !trap_can_continue(world, x2, y2) {
        return false;
    }
    // `:8933-8936`: the shimmer-position veto.
    let (sx, sy) = (shimmer.0 as f64, shimmer.1 as f64);
    let dx = x2 as f64 - sx;
    let dy = y2 as f64 - sy;
    if (dx * dx + dy * dy).sqrt() < 100.0 {
        return false;
    }
    let mut num = y2;
    let mut lava_here = false;
    let mut near_underworld = false;
    // `:8942-8957`: the drop scan.
    while !solid_trap(world, x2, num) {
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
    num -= 1;
    // `:8962-8977`: the boulder-nearby, wall and lava gates.
    if is_tile_nearby(world, x2, num, 70, 20) {
        return false;
    }
    let floor = world.tile(x2, num).map(|t| (t.wall, t.liquid, t.liquid_kind()));
    let Some((floor_wall, floor_liquid, floor_kind)) = floor else {
        return false;
    };
    if floor_wall == 87 || floor_wall == 350 {
        return false;
    }
    if floor_liquid > 0 && floor_kind == LiquidKind::Lava {
        lava_here = true;
    }
    // `:8988-9004`: the classic type roll.
    let mut kind_type: i32 = -1;
    if rng.range(0, 20) == 0 {
        kind_type = 2;
    } else if num > lava_line + 30 && rng.range(0, 6) != 0 {
        kind_type = 3;
    } else {
        kind_type = rng.range(0, 2);
    }
    // `:9009-9024`: the bounds + lava + underworld gates.
    if x2 < 3 || num < 3 || x2 >= world.width - 3 || num >= world.height - 3 {
        return false;
    }
    if lava_here && kind_type != 3 {
        return false;
    }
    if near_underworld && kind_type != 3 {
        return false;
    }
    // `:9026-9029`: the 3×3 clear box above the plate.
    for dx in -1..=1 {
        for dy in -2..=0 {
            if world
                .tile(x2 + dx, num + dy)
                .is_some_and(|t| t.active() && !t.inactive())
            {
                return false;
            }
        }
    }
    // `:9030-9034`: the spike-below veto.
    if world
        .tile(x2, num + 1)
        .is_some_and(|t| t.active() && (t.kind == 48 || t.kind == 232))
    {
        return false;
    }
    if !can_pressure_plate_at(world, x2, num) {
        return false;
    }
    match kind_type {
        0 => trap_dart(world, x2, num, floor_wall, rng),
        1 => trap_boulder(world, x2, num, world_surface, boulder_pets, rng),
        2 => trap_explosive(world, x2, num, rng),
        3 => trap_geyser(world, x2, num, rng),
        _ => false,
    }
}

/// `placeTrap` case 0 (`WorldGen.cs:9057-9201`) — the dart trap: the
/// left/right wall walks at the `num - Next(3)` row, the 5..50 distance
/// gates, the door/spike housing vetoes, the plate (style 2 on a wall,
/// else `Next(2, 4)`), the `KillTile` niche, the 137 dart (the +18
/// frameX when the wall is left of the plate), the wire walk.
fn trap_dart(world: &mut World, x2: i32, num: i32, floor_wall: u16, rng: &mut Rng) -> bool {
    let num11 = num - rng.range(0, 3);
    let cracked = |world: &World, x: i32| {
        world
            .tile(x, num11)
            .is_some_and(|t| t.active() && matches!(t.kind, 481 | 482 | 483))
    };
    // The left walk (`:9061-9070`).
    let mut num10 = x2;
    while !solid_trap(world, num10, num11) && !cracked(world, num10) {
        num10 -= 1;
        if num10 < 0 || !trap_can_continue(world, num10, num11) {
            return false;
        }
    }
    let num12 = num10;
    // The right walk (`:9072-9081`).
    num10 = x2;
    while !solid_trap(world, num10, num11) && !cracked(world, num10) {
        num10 += 1;
        if num10 >= world.width || !trap_can_continue(world, num10, num11) {
            return false;
        }
    }
    let num13 = num10;
    let num14 = x2 - num12;
    let num15 = num13 - x2;
    let mut flag3 = num14 > 5 && num14 < 50;
    let mut flag4 = num15 > 5 && num15 < 50;
    if flag3 && !solid_trap(world, num12, num11 + 1) {
        flag3 = false;
    }
    if flag4 && !solid_trap(world, num13, num11 + 1) {
        flag4 = false;
    }
    let bad_housing = |world: &World, x: i32| {
        world.tile(x, num11).is_some_and(|t| t.active() && (t.kind == 10 || t.kind == 48))
            || world
                .tile(x, num11 + 1)
                .is_some_and(|t| t.active() && (t.kind == 10 || t.kind == 48))
    };
    if flag3 && bad_housing(world, num12) {
        flag3 = false;
    }
    if flag4 && bad_housing(world, num13) {
        flag4 = false;
    }
    // `:9113-9140`: the side pick.
    let (wall_x, from_left) = if flag3 && flag4 {
        if rng.range(0, 2) == 0 {
            (num13, false)
        } else {
            (num12, true)
        }
    } else if flag4 {
        (num13, false)
    } else if flag3 {
        (num12, true)
    } else {
        return false;
    };
    if world.tile(wall_x, num11).is_some_and(|t| t.kind == 190) {
        return false;
    }
    if world.tile(wall_x, num11).map(|t| t.wall).unwrap_or(0) == 350 {
        return false;
    }
    // `:9153-9161`: the plate.
    let plate_style = if floor_wall > 0 { 2 } else { rng.range(2, 4) };
    place_1x1(world, x2, num, 135, plate_style);
    // `:9162`: the dart's niche.
    kill(world, wall_x, num11);
    // `:9171`: the dart (the classic num17 = 0 — the notTheBees/noTraps
    // arm is the seed lane's).
    place_1x1(world, wall_x, num11, 137, 0);
    if from_left {
        if let Some(t) = world.tile_mut(wall_x, num11) {
            t.frame_x += 18;
        }
    }
    wire_walk(world, x2, num, wall_x, num11);
    true
}

/// `placeTrap` case 1 (`WorldGen.cs:9202-9399`) — the boulder trap: the
/// upward 5×4 solid-body scan (the dungeon/actuator/boulder vetoes, the
/// dirt/stone/mud count), the 5..40 height gate, the chest-nearby veto,
/// the boulder-nearby veto, the shaft carve, the stone rim, the plate
/// (style 7), the two stone supports, the boulder-type roll chain, the
/// pet-boulder override, the 2×3 actuated stone grid, the wire walk.
fn trap_boulder(
    world: &mut World,
    x2: i32,
    num: i32,
    world_surface: f64,
    boulder_pets: &mut i32,
    rng: &mut Rng,
) -> bool {
    // `:9204-9210`: the pet cap by world size (`GetWorldSize`
    // `:6559-6570`: ≤4200 → 0, ≤6400 → 1, else 2).
    let num20 = if world.width <= 4200 {
        2
    } else if world.width <= 6400 {
        4
    } else {
        6
    };
    let num21 = x2 + rng.range(-1, 2);
    let mut num22 = num - 8;
    // `:9221-9264`: the upward body scan.
    loop {
        let mut flag6 = true;
        let mut num23 = 0;
        for num24 in (num21 - 2)..=(num21 + 3) {
            for num25 in num22..=(num22 + 3) {
                if !solid_trap(world, num24, num25) {
                    flag6 = false;
                }
                if !trap_can_continue(world, num24, num25) {
                    return false;
                }
                if let Some(t) = world.tile(num24, num25) {
                    if t.active() {
                        // `Main.tileDungeon` (`Main.cs:7992-7997`).
                        if t.kind == 226 || matches!(t.kind, 41 | 43 | 44 | 677 | 678 | 679) {
                            return false;
                        }
                        if t.actuator() || is_boulder(t.kind) {
                            return false;
                        }
                        if t.kind == 0 || t.kind == 1 || t.kind == 59 {
                            num23 += 1;
                        }
                    }
                }
            }
        }
        num22 -= 1;
        if (num22 as f64) < world_surface {
            return false;
        }
        if flag6 && num23 > 2 {
            break;
        }
    }
    // `:9265-9281`: the height + chest + boulder gates.
    if num - num22 <= 5 || num - num22 >= 40 {
        return false;
    }
    if is_tile_nearby(world, num21, num22, 21, 4) || is_tile_nearby(world, num21, num22, 467, 4) {
        return false;
    }
    if boulder_nearby(world, num21, num22, 10) {
        return false;
    }
    // `:9282-9292`: the shaft carve.
    for num26 in num21..=(num21 + 1) {
        for num27 in num22..=num {
            if world.tile(num26, num27).map(|t| t.kind).unwrap_or(0) != 379 {
                kill(world, num26, num27);
            }
        }
    }
    // `:9293-9302`: the stone rim.
    for num28 in (num21 - 2)..=(num21 + 3) {
        for num29 in (num22 - 2)..=(num22 + 3) {
            if solid_trap(world, num28, num29) {
                if let Some(t) = world.tile_mut(num28, num29) {
                    t.kind = 1;
                }
            }
        }
    }
    // `:9303-9305`: the plate + the two stone supports.
    place_1x1(world, x2, num, 135, 7);
    place_stone(world, num21, num22 + 2);
    place_stone(world, num21 + 1, num22 + 2);
    // `:9306-9355`: the boulder-type roll chain (the classic maxValues
    // 7/2/3/6; the noTraps overrides are the seed lane's).
    let mut num30: u16 = 138;
    if rng.range(0, 2) == 0 && close_to_spiders(world, num21, num22) {
        num30 = 714;
    } else if rng.range(0, 6) == 0 && close_to_dungeon(world, num21, num22) {
        num30 = 715;
    } else if rng.range(0, 3) == 0 && close_to_lava(world, num21, num22) {
        num30 = 713;
    } else if rng.range(0, 25) == 0 {
        num30 = 711;
    } else if rng.range(0, 20) == 0 {
        num30 = 664;
    }
    // `:9356-9360`: the pet boulder.
    if num30 == 138 && *boulder_pets < num20 {
        *boulder_pets += 1;
        num30 = 716;
    }
    place_2x2(world, num21 + 1, num22 + 1, num30, 0);
    // `:9362-9375`: the 2×3 actuated stone grid.
    num22 += 2;
    for num31 in 0..2 {
        for num32 in 0..3 {
            let (num33, num34) = (num21 + num31, num22 + num32);
            place_stone(world, num33, num34);
            world.wires.set(num33, num34, crate::wiring::WireColor::Red, true);
            if let Some(t) = world.tile_mut(num33, num34) {
                t.flags |= FLAG_ACTUATOR;
            }
        }
    }
    wire_walk(world, x2, num, num21, num22);
    true
}

/// `PlaceTile(x, y, 1, mute: true)` — the plain stone write (the
/// `PlaceTile` default arm `:60234-60249`: active + type, no frames).
fn place_stone(world: &mut World, x: i32, y: i32) {
    if let Some(t) = world.tile_mut(x, y) {
        t.flags |= FLAG_ACTIVE;
        t.kind = 1;
    }
}

/// `placeTrap` case 2 (`WorldGen.cs:9400-9459`) — the explosive trap:
/// the `Next(4, 7)`-deep column walk, the ±2 solid box, the `KillTile`
/// niche, the 141 explosive (`frameY = 18 * Next(2)`), the plate
/// (`Next(2, 4)`), the wire walk.
fn trap_explosive(world: &mut World, x2: i32, num: i32, rng: &mut Rng) -> bool {
    let num4 = rng.range(4, 7);
    let num5 = x2 + rng.range(-1, 2);
    let mut num6 = num;
    for _ in 0..num4 {
        num6 += 1;
        if num5 < 5
            || num6 < 5
            || num5 >= world.width - 5
            || num6 >= world.height - 5
            || !solid_trap(world, num5, num6)
            || !trap_can_continue(world, num5, num6)
        {
            return false;
        }
    }
    for n in (num5 - 2)..=(num5 + 2) {
        for num7 in (num6 - 2)..=(num6 + 2) {
            if n < 0
                || num7 < 0
                || n >= world.width
                || num7 >= world.height
                || !solid_trap(world, n, num7)
                || !trap_can_continue(world, n, num7)
            {
                return false;
            }
        }
    }
    kill(world, num5, num6);
    if let Some(t) = world.tile_mut(num5, num6) {
        t.flags |= FLAG_ACTIVE;
        t.kind = 141;
        t.frame_x = 0;
        t.frame_y = (18 * rng.range(0, 2)) as i16;
    }
    place_1x1(world, x2, num, 135, rng.range(2, 4));
    wire_walk(world, x2, num, num5, num6);
    // The C# `break`s out of the switch to the shared `return false`
    // (`:9508-9512`) — the explosive case reports failure even on a
    // successful place (the trapDiag success bump is the only effect).
    false
}

/// `placeTrap` case 3 (`WorldGen.cs:9460-9500`) — the geyser trap: the
/// right-cell clear gate, the non-lava liquid veto, the two-column
/// solid floor, the 443 pair (`frameX = 18*l + 36*Next(2)`).
fn trap_geyser(world: &mut World, x2: i32, num: i32, rng: &mut Rng) -> bool {
    if world.tile(x2 + 1, num).is_some_and(|t| t.active()) {
        return false;
    }
    if world
        .tile(x2, num)
        .is_some_and(|t| t.liquid > 0 && t.liquid_kind() != LiquidKind::Lava)
    {
        return false;
    }
    if !trap_can_continue(world, x2, num) {
        return false;
    }
    for k in x2..=(x2 + 1) {
        let num2 = num + 1;
        if k < 5
            || num2 < 5
            || k >= world.width - 5
            || num2 >= world.height - 5
            || !solid_trap(world, k, num2)
            || !trap_can_continue(world, k, num2)
        {
            return false;
        }
    }
    let num3 = rng.range(0, 2);
    for l in 0..2 {
        if let Some(t) = world.tile_mut(x2 + l, num) {
            t.flags |= FLAG_ACTIVE;
            t.kind = 443;
            t.frame_x = (18 * l + 36 * num3) as i16;
            t.frame_y = 0;
        }
    }
    true
}

/// `Tile.ClearTile` (`Tile.cs:123-128`): ClearSlope + active(false) +
/// inActive(false).
fn clear_tile(t: &mut Tile) {
    t.set_slope(0);
    t.flags &= !(FLAG_HALF_BRICK | FLAG_ACTIVE | crate::tile::FLAG_INACTIVE);
}

/// `WorldGen.PlaceSandTrap` (`WorldGen.cs:35463-35735`): the desert
/// sand trap — the drop scan to the underworld layer, the sand/hardened/
/// sandstone + wall-216/187 gate, the ±25 wire/chest/painting/door/
/// platform/chair/219/314 veto box, the ±2×+1..+3 solid floor, the
/// 162 veto, the 30-up sandstone-396 ceiling search, the height gate,
/// the ±num5 mid-row clear + the ore/404 veto + the 75 % solid count,
/// the shell write (the 396 floor/ceiling, the 397 rim, the 53 fill),
/// the side-column slope tidy, the plate (style 7), the actuated/wired
/// sand column with the carve band.
fn place_sand_trap(world: &mut World, i: i32, j: i32, rng: &mut Rng) -> bool {
    let num = 6;
    let num2 = 4;
    let num3 = 25;
    let underworld = world.height - 200;
    // `:35472-35477`: the drop scan.
    let mut k = j;
    while !world.tile(i, k).is_some_and(|t| t.active()) && k < underworld {
        k += 1;
    }
    if k >= underworld {
        return false;
    }
    // `:35478-35485`: the floor + sand/wall gates.
    let Some(floor) = world.tile(i, k) else {
        return false;
    };
    if !td::TILE_SOLID.has(floor.kind) || floor.half_brick() || floor.top_slope() {
        return false;
    }
    if !matches!(floor.kind, 53 | 397 | 396) || !matches!(floor.wall, 216 | 187) {
        return false;
    }
    k -= 1;
    // `:35487-35519`: the ±25 veto box.
    let mut num4 = -1;
    let num5 = rng.range(6, 12);
    let num6 = rng.range(6, 14);
    for l in (i - num3)..=(i + num3) {
        for m in (k - num3)..(k + num3) {
            let Some(t) = world.tile(l, m) else {
                continue;
            };
            if world.wires.get(l, m, crate::wiring::WireColor::Red)
                || world.wires.get(l, m, crate::wiring::WireColor::Blue)
                || world.wires.get(l, m, crate::wiring::WireColor::Green)
                || world.wires.get(l, m, crate::wiring::WireColor::Yellow)
            {
                return false;
            }
            if matches!(t.kind, 21 | 467)
                || td::SET_PAINTINGS.has(t.kind)
                || matches!(t.kind, 10 | 19 | 15 | 219 | 314)
            {
                return false;
            }
        }
    }
    // `:35520-35531`: the solid floor + the 162 veto.
    for n in (i - 2)..=(i + 2) {
        for num7 in (k + 1)..=(k + 3) {
            if !world
                .tile(n, num7)
                .is_some_and(|t| t.active() && td::TILE_SOLID.has(t.kind))
            {
                return false;
            }
        }
    }
    if world.tile(i, k + 1).is_some_and(|t| t.kind == 162) {
        return false;
    }
    // `:35532-35545`: the 30-up sandstone ceiling search.
    let mut num8 = k;
    while num8 > k - 30 {
        if let Some(t) = world.tile(i, num8) {
            if t.active() {
                if t.kind == 396 {
                    num4 = num8;
                    break;
                }
                return false;
            }
        }
        num8 -= 1;
    }
    if num4 <= -1 {
        return false;
    }
    if k - num4 < num6 + num2 {
        return false;
    }
    // `:35561-35585`: the mid-row clear + the ore veto + the solid count.
    let mut num9 = 0;
    let num10 = (k + num4) / 2;
    for num11 in (i - num5)..=(i + num5) {
        if world
            .tile(num11, num10)
            .is_some_and(|t| t.active() && td::TILE_SOLID.has(t.kind))
        {
            return false;
        }
        for num12 in (num4 - num6)..=num4 {
            if let Some(t) = world.tile(num11, num12) {
                if t.active() {
                    if td::SET_ORE.has(t.kind) || t.kind == 404 {
                        return false;
                    }
                    if td::TILE_SOLID.has(t.kind) {
                        num9 += 1;
                    }
                }
            }
        }
    }
    if (num9 as f64) < ((num5 * 2 + 1) * (num6 + 1)) as f64 * 0.75 {
        return false;
    }
    // `:35586-35634`: the shell write.
    for num13 in (i - num5 - 1)..=(i + num5 + 1) {
        for num14 in (num4 - num6)..=num4 {
            let flag = world
                .tile(num13, num14)
                .is_some_and(|t| t.active() && td::TILE_SOLID.has(t.kind));
            if num14 == num4 {
                if let Some(t) = world.tile_mut(num13, num14) {
                    t.set_slope(0);
                    t.flags &= !FLAG_HALF_BRICK;
                    if !flag {
                        t.flags |= FLAG_ACTIVE;
                        t.kind = 396;
                    }
                }
            } else if num14 == num4 - num6 {
                let above_solid = world
                    .tile(num13, num14 - 1)
                    .is_some_and(|t| t.active() && td::TILE_SOLID.has(t.kind));
                if let Some(t) = world.tile_mut(num13, num14) {
                    clear_tile(t);
                    t.flags |= FLAG_ACTIVE;
                    t.kind = if flag && above_solid { 397 } else { 396 };
                }
            } else if num13 == i - num5 - 1 || num13 == i + num5 + 1 {
                if let Some(t) = world.tile_mut(num13, num14) {
                    if !flag {
                        clear_tile(t);
                        t.flags |= FLAG_ACTIVE;
                        t.kind = 396;
                    } else {
                        t.set_slope(0);
                        t.flags &= !FLAG_HALF_BRICK;
                    }
                }
            } else if let Some(t) = world.tile_mut(num13, num14) {
                clear_tile(t);
                t.flags |= FLAG_ACTIVE;
                t.kind = 53;
            }
        }
    }
    // `:35635-35675`: the side-column slope tidy.
    let lo = (num4 as f64 - num6 as f64 * 0.666) as i32;
    let hi = num4 as f64 - num6 as f64 * 0.333;
    let mut num15 = lo;
    while (num15 as f64) <= hi {
        for side in [i - num5 - 2, i + num5 + 2] {
            if side == i - num5 - 2 {
                if (num15 as f64) < num4 as f64 - num6 as f64 * 0.4 {
                    if let Some(t) = world.tile_mut(side, num15) {
                        if t.bottom_slope() {
                            t.set_slope(0);
                        }
                    }
                } else if (num15 as f64) > num4 as f64 - num6 as f64 * 0.6 {
                    if let Some(t) = world.tile_mut(side, num15) {
                        if t.top_slope() {
                            t.set_slope(0);
                        }
                        t.flags &= !FLAG_HALF_BRICK;
                    }
                } else if let Some(t) = world.tile_mut(side, num15) {
                    t.flags &= !FLAG_HALF_BRICK;
                    t.set_slope(0);
                }
            }
            if !world
                .tile(side, num15)
                .is_some_and(|t| t.active() && td::TILE_SOLID.has(t.kind))
            {
                if let Some(t) = world.tile_mut(side, num15) {
                    t.flags |= FLAG_ACTIVE;
                    t.kind = 396;
                }
            }
        }
        num15 += 1;
    }
    // `:35676-35693`: the inner-column slope clear + the top-row tidy.
    for num16 in (num4 - num6)..=num4 {
        for side in [i - num5 - 2, i - num5 - 1, i - num5 + 1, i - num5 + 2] {
            if let Some(t) = world.tile_mut(side, num16) {
                t.set_slope(0);
                t.flags &= !FLAG_HALF_BRICK;
            }
        }
    }
    for num17 in (i - num5 - 1)..(i + num5 + 1) {
        let num18 = k - num6 - 1;
        if let Some(t) = world.tile_mut(num17, num18) {
            if t.bottom_slope() {
                t.set_slope(0);
            }
            t.flags &= !FLAG_HALF_BRICK;
        }
    }
    // `:35694-35698`: the plate row carve + the plate.
    kill(world, i - 2, k);
    kill(world, i - 1, k);
    kill(world, i + 1, k);
    kill(world, i + 2, k);
    place_1x1(world, i, k, 135, 7);
    // `:35699-35733`: the actuated/wired sand column.
    for num19 in (i - num5)..=(i + num5) {
        let mut num20 = k;
        let off = (num19 - i).abs() as f64;
        let band = num5 as f64;
        if off > band * 0.8 {
            num20 = k - 3;
        } else if off > band * 0.6 {
            num20 = k - 2;
        } else if off > band * 0.4 {
            num20 = k - 1;
        }
        for num21 in num4..=k {
            if num19 == i && num21 <= k {
                world.wires.set(i, num21, crate::wiring::WireColor::Red, true);
            }
            if world
                .tile(num19, num21)
                .is_some_and(|t| t.active() && td::TILE_SOLID.has(t.kind))
            {
                if num21 < num4 + num - 4 {
                    if let Some(t) = world.tile_mut(num19, num21) {
                        t.flags |= FLAG_ACTUATOR;
                    }
                    world.wires.set(num19, num21, crate::wiring::WireColor::Red, true);
                } else if num21 < num20 {
                    kill(world, num19, num21);
                }
            }
        }
    }
    true
}

/// The `Traps` pass (`WorldGen.cs:18657-18786`, W4-d104 pass 34): the
/// classic trap scatter (`width * 0.05`, the `getGoodWorldGen` ×1.5,
/// the skyblock /5 + 100-budget) then the sand-trap scatter
/// (`width * 0.003`, the ×1.5). The noTraps arms (the count multipliers,
/// the TNT barrels, the boulder-solidity flips) and the drunk
/// progress-only loop are the seed lane's; the `starGame` multiplier is
/// unported.
#[allow(clippy::too_many_arguments)]
pub(crate) fn pass_traps(
    world: &mut World,
    world_surface: f64,
    rock_layer: f64,
    lava_line: i32,
    get_good: bool,
    shimmer: (i32, i32),
    boulder_pets: &mut i32,
    rng: &mut Rng,
) {
    // The sub-vanilla width guard: the trap column draw
    // `Next(200, width - 200)` throws on a ≤400-wide world.
    if world.width <= 400 {
        return;
    }
    let ocean_level = (world_surface + rock_layer) / 2.0 + 40.0;
    let skyblock = world.seeds.contains(crate::seeds::SeedFlags::SKYBLOCK);
    // `:18666-18686`: the trap count.
    let mut num = world.width as f64 * 0.05;
    if get_good {
        num *= 1.5;
    }
    let mut num2 = 1150;
    if skyblock {
        num /= 5.0;
        num2 = 100;
    }
    // `:18688-18719`: the trap scatter.
    let mut i = 0.0;
    while i < num {
        for _ in 0..num2 {
            let mut num5 = rng.range(200, world.width - 200);
            let mut num6 = rng.range(world_surface as i32, world.height - 210);
            while ocean_depths_trap(world, num5, num6, ocean_level) {
                num5 = rng.range(200, world.width - 200);
                num6 = rng.range(world_surface as i32, world.height - 210);
            }
            if world.tile(num5, num6).map(|t| t.wall).unwrap_or(0) == 0
                && place_trap(
                    world, num5, num6, world_surface, rock_layer, lava_line, shimmer,
                    boulder_pets, rng,
                )
            {
                break;
            }
        }
        i += 1.0;
    }
    // `:18747-18765`: the sand-trap scatter.
    let mut num = world.width as f64 * 0.003;
    if get_good {
        num *= 1.5;
    }
    let mut l = 0.0;
    while l < num {
        for _ in 0..20000 {
            let num7 = rng.range((world.width as f64 * 0.15) as i32, (world.width as f64 * 0.85) as i32);
            let num8 = rng.range(world_surface as i32 + 20, world.height - 210);
            if world.tile(num7, num8).map(|t| t.wall).unwrap_or(0) == 187
                && place_sand_trap(world, num7, num8, rng)
            {
                break;
            }
        }
        l += 1.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A stone underground with a dirt floor row at 100, wall-less so
    /// the scatter's `wall == 0` gate can place.
    fn trap_world() -> World {
        let mut w = World::new("tr".into(), 900, 600);
        for x in 0..w.width {
            for y in 100..w.height {
                let i = w.index(x, y);
                w.tiles[i].kind = 1;
                w.tiles[i].flags |= FLAG_ACTIVE;
            }
        }
        w
    }

    /// W4-d104 pass 34: the geyser trap (case 3) places its 443 pair on
    /// a clear floor, and two same-stream runs replay byte-equal.
    #[test]
    fn w4_d104_34_the_geyser_places_and_replays() {
        let mut a = trap_world();
        let mut b = trap_world();
        // Clear the 3x3 box above (450, 99) so the head gates pass.
        for w in [&mut a, &mut b] {
            for dx in -1..=1 {
                for dy in -2..=0 {
                    let i = w.index(450 + dx, 99 + dy);
                    w.tiles[i].flags &= !FLAG_ACTIVE;
                }
            }
        }
        let mut pa = 0;
        let mut pb = 0;
        pass_traps(&mut a, 90.0, 200.0, 500, false, (0, 0), &mut pa, &mut crate::worldgen::noise::Rng(42));
        pass_traps(&mut b, 90.0, 200.0, 500, false, (0, 0), &mut pb, &mut crate::worldgen::noise::Rng(42));
        assert_eq!(a.tiles, b.tiles, "a == b: the pass stream replays");
        assert_eq!(pa, pb, "the boulder-pet counter replays");
    }

    /// W4-d104 pass 34: the dart case wires a plate to a wall dart.
    #[test]
    fn w4_d104_34_the_dart_case_wires_the_plate() {
        let mut w = trap_world();
        // A cave at rows 97..=99 (every possible `num - Next(3)` row)
        // with solid wall columns at x=430 and x=470 down to the floor.
        for x in 431..470 {
            for y in 97..=99 {
                let i = w.index(x, y);
                w.tiles[i].flags &= !FLAG_ACTIVE;
            }
        }
        for x in [430, 470] {
            for y in 95..100 {
                let i = w.index(x, y);
                w.tiles[i].kind = 1;
                w.tiles[i].flags |= FLAG_ACTIVE;
            }
        }
        let ok = trap_dart(&mut w, 450, 99, 0, &mut crate::worldgen::noise::Rng(7));
        assert!(ok, "the dart placed");
        assert!(w.tile(450, 99).is_some_and(|t| t.active() && t.kind == 135), "the plate");
        let dart = [430, 470]
            .iter()
            .any(|&x| (97..=99).any(|y| w.tile(x, y).is_some_and(|t| t.active() && t.kind == 137)));
        assert!(dart, "the wall dart");
    }
}