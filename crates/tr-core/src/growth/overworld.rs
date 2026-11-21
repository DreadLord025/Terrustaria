//! The OVERWORLD arms of `WorldGen.UpdateWorld_GrassGrowth`
//! (`WorldGen.cs:74510-74866`, the `switch (num10)` after the underground
//! slice's `return` at `:74508`) — W4-g42 dispatcher follow-up D2. The port's
//! `grass.rs::update_world_grass_growth` carries the UNDERGROUND slice only;
//! the overground dispatcher (`growth.rs::update_world_overground_tile`) used
//! to route every grass kind through it, so the overworld plant rolls, the
//! jungle tree/plant arms and the overworld spread ladder never ran (the
//! case-70 precedent, T-3). This module lands the remaining overworld arms
//! verbatim: the shared forest/evil/hallow/golf arm (`:74513-74702`), the
//! jungle case-60 (`:74752-74813`), the jungle-plant case-226 (`:74815-74828`)
//! and the ash case-633 (`:74829-74865`). The case-70 arm stays
//! `growth.rs::overworld_mushroom_growth` (T-3).
//!
//! Deviations (ANALYSIS): `Main.halloween` has no port field — the pumpkin
//! arm's `Next(75)` gate folds to false (the seasonal-constants-off
//! precedent), so the draw never fires; `SquareTileFrame` re-frames and the
//! `netMode == 2` sends are the framer's / MP's job (the T-3 precedent).
//! `WorldGen.expertMode` is single-difficulty normal (the port precedent):
//! the underground life-fruit odds stay 40 and its scan radius 60.

use crate::tile::FLAG_ACTIVE;
use crate::UnifiedRandom;
use crate::world::World;

use super::grass::spread_grass;

/// The shared overworld grass arm (`WorldGen.cs:74513-74702`): kinds
/// 2/23/32/109/199/352/477/492/661/662 (the 633 plant row at `:74574` is
/// dead code — 633 rides its own case at `:74829`, never this arm's label
/// set). `allowed` is `WorldGen.AllowedToSpreadInfections`.
pub(super) fn overworld_grass_growth(
    world: &mut World,
    i: i32,
    j: i32,
    min_i: i32,
    max_i: i32,
    min_j: i32,
    max_j: i32,
    allowed: bool,
    rng: &mut UnifiedRandom,
) {
    // `:74271`: the entry guard binds the overworld switch too.
    if !in_world_10(world, i, j) {
        return;
    }
    let Some(t) = world.tile(i, j) else {
        return;
    };
    // The callers gate on `nactive()` (`:72216`/`:73252`) — an actuated
    // grass grows nothing (W4-g42 GROWTH FINAL).
    if t.inactive() {
        return;
    }
    let mut num10 = t.kind as i32;
    // The halloween pumpkin arm (`:74524-74546`): `Main.halloween &&
    // Next(75) == 0 && (2 || 109)` — the port carries no halloween flag, the
    // gate folds false and the draw is never spent (ANALYSIS).
    // The plant-above roll (`:74547-74590`): the cell above must be inactive
    // AND dry; the else-if ladder draws ONE Next per kind (2 at Next(12),
    // the rest at Next(10)) and only when the kind matches.
    let above_free = !world.tile(i, min_j).is_some_and(|t| t.active())
        && world.tile(i, min_j).is_some_and(|t| t.liquid == 0);
    if above_free {
        let mut num17: i32 = -1;
        if num10 == 2 && rng.next_max(12) == 0 {
            num17 = 3;
        } else if num10 == 23 && rng.next_max(10) == 0 {
            num17 = 24;
        } else if num10 == 199 && rng.next_max(10) == 0 {
            num17 = 201;
        } else if num10 == 661 && rng.next_max(10) == 0 {
            num17 = 24;
        } else if num10 == 662 && rng.next_max(10) == 0 {
            num17 = 201;
        } else if num10 == 109 && rng.next_max(10) == 0 {
            num17 = 110;
        }
        if num17 != -1 {
            place_flower_above(world, i, j, min_j, num17 as u16, rng);
        }
    }
    // The thorn/golf remap (`:74592-74614`): 32→23 and 352→199 return early
    // when the infection spread is frozen — BEFORE any spread work.
    match num10 {
        32 => {
            num10 = 23;
            if !allowed {
                return;
            }
        }
        352 => {
            num10 = 199;
            if !allowed {
                return;
            }
        }
        477 => num10 = 2,
        492 => num10 = 109,
        _ => {}
    }
    // The grass pair (`:74615-74626`): the evil grasses carry their mud twin.
    let mut grass = num10;
    let mut num18: i32 = -1;
    if num10 == 23 || num10 == 661 {
        grass = 23;
        num18 = 661;
    }
    if num10 == 199 || num10 == 662 {
        grass = 199;
        num18 = 662;
    }
    // `flag8` (`:74627`): the infection-side shortcut — with the spread on,
    // the infection/hallow/golf-hallow grasses judge EVERY active neighbour.
    let flag8 = allowed
        && matches!(num10, 23 | 199 | 109 | 492 | 661 | 662)
        && in_world_10(world, i, j);
    for num19 in min_i..max_i {
        for num20 in min_j..max_j {
            if !in_world_10(world, num19, num20) || (i == num19 && j == num20) {
                continue;
            }
            let Some(n) = world.tile(num19, num20) else {
                continue;
            };
            if !n.active() {
                continue;
            }
            let type2 = n.kind as i32;
            // `:74637`: without flag8 only dirt (and the mud twin's soil)
            // proceeds.
            if !flag8 && type2 != 0 && (num18 == -1 || type2 != 59) {
                continue;
            }
            // The evil block (`:74642-74665`).
            if type2 == 0
                || (num18 > -1 && type2 == 59)
                || (matches!(num10, 23 | 661 | 199 | 662)
                    && matches!(type2, 2 | 109 | 477 | 492))
            {
                spread_grass(world, num19, num20, 0, grass as u16);
                if num18 > -1 {
                    spread_grass(world, num19, num20, 59, num18 as u16);
                }
                if allowed && matches!(num10, 23 | 199 | 661 | 662) {
                    spread_grass(world, num19, num20, 2, grass as u16);
                    spread_grass(world, num19, num20, 109, grass as u16);
                    spread_grass(world, num19, num20, 477, grass as u16);
                    spread_grass(world, num19, num20, 492, grass as u16);
                    if num18 > -1 {
                        spread_grass(world, num19, num20, 60, num18 as u16);
                    }
                }
            }
            // The hallow block (`:74666-74695`): dirt, or the hallow grasses
            // converting the plain/evil grasses.
            if type2 == 0
                || (matches!(num10, 109 | 492) && matches!(type2, 2 | 477 | 23 | 199))
            {
                spread_grass(world, num19, num20, 0, grass as u16);
                if num10 == 109 {
                    spread_grass(world, num19, num20, 2, grass as u16);
                }
                match num10 {
                    492 => spread_grass(world, num19, num20, 477, grass as u16),
                    109 => spread_grass(world, num19, num20, 477, 492),
                    _ => {}
                }
                if matches!(num10, 492 | 109) && allowed {
                    spread_grass(world, num19, num20, 23, 109);
                }
                if matches!(num10, 492 | 109) && allowed {
                    spread_grass(world, num19, num20, 199, 109);
                }
            }
        }
    }
}

/// The overworld plant write (`PlaceTile(i, minJ, num17, mute: true)` +
/// `CopyPaintAndCoating`, `:74580-74588`): the `WorldGen.PlaceTile` flower
/// arm ([`place_flower_tile`]) then the caller's paint copy gated on the
/// placed type (`type == num17` — the thorn conversions skip it).
fn place_flower_above(
    world: &mut World,
    i: i32,
    j: i32,
    min_j: i32,
    num: u16,
    rng: &mut UnifiedRandom,
) {
    place_flower_tile(world, i, min_j, num, rng);
    if world.tile(i, min_j).is_some_and(|t| t.active() && t.kind == num) {
        copy_paint_from(world, i, min_j, i, j);
    }
}

/// `WorldGen.PlaceTile`'s flower arm (`WorldGen.cs:59507-59581`, kinds
/// 3/24/110/201/637): the `IsFitToPlaceFlowerIn` soil gate, the thorn
/// conversions (24→32 / 201→352 at `Next(13)`), the planter-box style list,
/// the wall-gated odd-frame rolls and the plain `Next(6)` frame. RNG order
/// is the C# chain verbatim; the `SquareTileFrame` re-frame is the framer's
/// job (ANALYSIS). Shared by the overworld plant rolls and the underground
/// slice's `PlaceTile(i, minJ, num3)` (`:74366`).
pub(super) fn place_flower_tile(world: &mut World, i: i32, j: i32, num: u16, rng: &mut UnifiedRandom) {
    // `:59454-59470`: the PlaceTile liquid gate — 3/24/201/637 sit in the
    // `:59468` refusal list (110 does NOT). The overworld caller already
    // proved the cell dry; the underground slice's roll has no liquid gate.
    if matches!(num, 3 | 24 | 201 | 637) && world.tile(i, j).is_some_and(|t| t.liquid > 0) {
        return;
    }
    if !is_fit_to_place_flower_in(world, i, j, num) {
        return;
    }
    // `:59511-59522`: the thorn conversions draw Next(13) for 24/201 only.
    if num == 24 && rng.next_max(13) == 0 {
        write_plant(world, i, j, 32, 0);
        return;
    }
    if num == 201 && rng.next_max(13) == 0 {
        write_plant(world, i, j, 352, 0);
        return;
    }
    // `:59523-59542`: the planter boxes 78/380/579 — the 22-style list, the
    // 21+ multiples re-roll Next(3).
    let below_kind = world.tile(i, j + 1).map(|t| t.kind).unwrap_or(0);
    if matches!(below_kind, 78 | 380 | 579) {
        let mut num2 = rng.next_from_slice(&[
            6, 7, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 24, 27, 30, 33, 36, 39, 42,
        ]);
        if matches!(num2, 21 | 24 | 27 | 30 | 33 | 36 | 39 | 42) {
            num2 += rng.next_max(3);
        }
        write_plant(world, i, j, num, (num2 * 18) as i16);
        return;
    }
    // `:59543-59578`: the wall arm — both the cell's and the soil's wall in
    // `WallID.Sets.AllowsPlantsToGrow`.
    let wall_ok = world
        .tile(i, j)
        .is_some_and(|t| super::trees::allows_plants_to_grow(t.wall))
        && world
            .tile(i, j + 1)
            .is_some_and(|t| super::trees::allows_plants_to_grow(t.wall));
    if wall_ok {
        // `:59545`: `Next(50) == 0 || ((24 || 201) && Next(40) == 0)` — the
        // second draw only for the evil flowers when the first misses.
        if rng.next_max(50) == 0 || (matches!(num, 24 | 201) && rng.next_max(40) == 0) {
            let fx = if num == 201 { 270 } else { 144 };
            write_plant(world, i, j, num, fx);
        } else if rng.next_max(35) == 0
            || world.tile(i, j).is_some_and(|t| (63..=70).contains(&t.wall))
        {
            // `:59558-59571`: the per-flower style lists.
            let mut num3 = rng.next_from_slice(&[6, 7, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20]);
            if num == 201 {
                num3 = rng.next_from_slice(&[
                    6, 7, 8, 9, 10, 11, 12, 13, 14, 16, 17, 18, 19, 20, 21, 22,
                ]);
            }
            if num == 637 {
                num3 = rng.next_from_slice(&[6, 7, 8, 9, 10]);
            }
            write_plant(world, i, j, num, (num3 * 18) as i16);
        } else {
            // `:59573-59578`: the plain frame roll.
            let fx = (rng.next_max(6) * 18) as i16;
            write_plant(world, i, j, num, fx);
        }
    }
}

/// The plant cell write (`PlaceTile`'s `tile.active(true); tile.type = num;
/// tile.frameX = …`): the dispatcher's free-above gate proved the cell
/// inactive, so the C# `:59473-59476` `Clear(Tile | TilePaint | Slope)` ran
/// first — the frames/paint/slope reset, the wall and liquid survive.
pub(super) fn write_plant(world: &mut World, i: i32, j: i32, kind: u16, frame_x: i16) {
    let idx = world.index(i, j);
    world.tiles[idx].flags |= FLAG_ACTIVE;
    world.tiles[idx].kind = kind;
    world.tiles[idx].frame_x = frame_x;
    world.tiles[idx].frame_y = 0;
    world.tiles[idx].meta = 0;
    if world.frame_important.len() <= kind as usize {
        world.frame_important.resize(crate::tiledata::TILE_COUNT, false);
    }
    world.frame_important[kind as usize] = true;
}

/// `WorldGen.IsFitToPlaceFlowerIn` (`WorldGen.cs:58231-58251`): the soil
/// below must be active, flat and of the flower's own family (the planter
/// boxes count for the plain flower 3); the ash soil 633 accepts only 637.
pub(super) fn is_fit_to_place_flower_in(world: &World, x: i32, y: i32, num: u16) -> bool {
    if y < 1 || y > world.height as i32 - 1 {
        return false;
    }
    let Some(t) = world.tile(x, y + 1) else {
        return false;
    };
    if !(t.active() && t.slope() == 0 && !t.half_brick()) {
        return false;
    }
    let soil_ok = (matches!(t.kind, 2 | 78 | 380 | 477 | 579) && num == 3)
        || (matches!(t.kind, 23 | 661) && num == 24)
        || (matches!(t.kind, 109 | 492) && num == 110)
        || (matches!(t.kind, 199 | 662) && num == 201);
    if !soil_ok {
        if t.kind == 633 {
            return num == 637;
        }
        return false;
    }
    true
}

/// `WorldGen.InWorld(i, j, 10)` — the 10-tile margin the spread scans use.
/// The `UpdateWorld_GrassGrowth` entry guard (`:74271`) sits BEFORE the
/// `if (underground)` split, so it binds the overworld switch too.
pub(super) fn in_world_10(world: &World, i: i32, j: i32) -> bool {
    i >= 10 && j >= 10 && i < world.width as i32 - 10 && j < world.height as i32 - 10
}

/// The overworld jungle arm (`WorldGen.cs:74752-74813`, `case 60`): the
/// `Next(7)` plant-61 roll, ELSE the `Next(500)` tree roll (the above cell
/// free or a 61/74/69 plant), ELSE the `Next(25)` jungle-rose roll (dry cell
/// above) — the if/else-if chain draws only until a branch is taken. The
/// 3×3 scan spreads the jungle grass over the mud 59 neighbours
/// (`:74794-74808`, no InWorld guard — the C# reads the raw cells).
pub(super) fn overworld_jungle_growth(
    world: &mut World,
    i: i32,
    j: i32,
    min_i: i32,
    max_i: i32,
    min_j: i32,
    max_j: i32,
    rng: &mut UnifiedRandom,
) {
    // `:74271`: the entry guard binds the overworld switch too.
    if !in_world_10(world, i, j) {
        return;
    }
    // The callers gate on `nactive()` (`:72216`/`:73252`).
    if world.tile(i, j).is_some_and(|t| t.inactive()) {
        return;
    }
    let above_active = world.tile(i, min_j).is_some_and(|t| t.active());
    // `:74754`: `!active && Next(7) == 0` → PlaceTile(61).
    if !above_active && rng.next_max(7) == 0 {
        // The `WorldGen.PlaceTile` num-61 arm (`:59582-59629`): the flat
        // 60/226 soil gate, then the thorn/bamboo/flower frame ladder.
        // The `:74757-74759` paint copy is gated on the cell taking.
        place_jungle_plant_tile(world, i, min_j, rng);
        if world.tile(i, min_j).is_some_and(|t| t.active()) {
            copy_paint_from(world, i, min_j, i, j);
        }
    } else if rng.next_max(500) == 0
        && (!above_active
            || world
                .tile(i, min_j)
                .is_some_and(|t| matches!(t.kind, 61 | 74 | 69)))
    {
        // `:74766-74772`: the tree roll. The `&& PlayerLOS` evaluates the
        // grow first — the FX gate is unported (the T-3 precedent).
        let _ = super::trees::try_growing_tree_by_type(world, 5, i, j, rng);
    } else if rng.next_max(25) == 0
        && world.tile(i, min_j).is_some_and(|t| t.liquid == 0)
    {
        // `:74773-74791`: the jungle rose 233 — the 3-wide style first
        // (`Next(8)`), the 2-wide fallback (`Next(12)`, styleY 1) when the
        // cell did not take.
        super::jungle::place_jungle_plant(world, i, min_j, 233, rng.next_max(8), 0, true);
        if !world.tile(i, min_j).is_some_and(|t| t.kind == 233) {
            super::jungle::place_jungle_plant(world, i, min_j, 233, rng.next_max(12), 1, true);
        }
    }
    // The 3×3 mud→jungle-grass scan (`:74794-74808`).
    for num21 in min_i..max_i {
        for num22 in min_j..max_j {
            if (i != num21 || j != num22)
                && world
                    .tile(num21, num22)
                    .is_some_and(|t| t.active() && t.kind == 59)
            {
                spread_grass(world, num21, num22, 59, 60);
            }
        }
    }
}

/// `WorldGen.PlaceTile`'s num-61 arm (`WorldGen.cs:59582-59629`): the jungle
/// plant 61 on flat jungle grass 60 / jungle-plant 226 soil. The frame
/// ladder: the thorn 69 at `Next(16)` (surface-only, not on 226), the
/// bamboo-tip frames 144/162 at `Next(60)`/`Next(230)` (the `&` conjunction
/// draws even when the depth gate is off), the `Next(15)` odd-frame arm
/// (`Next(3)` then `Next(2)`/`Next(13)`) and the plain `Next(6)` frame.
/// `flag2` = below the rock layer (the port has no remix worldgen flag —
/// the REMIX seed flag stands in, the trees.rs precedent). Shared by the
/// overworld case-60/226 rolls and the underground slice's `PlaceTile`
/// (`:74366`); the caller's paint copy is gated on the placed type.
pub(super) fn place_jungle_plant_tile(world: &mut World, i: i32, j: i32, rng: &mut UnifiedRandom) {
    let Some(below) = world.tile(i, j + 1) else {
        return;
    };
    if !(j + 1 < world.height as i32
        && below.active()
        && below.slope() == 0
        && !below.half_brick()
        && matches!(below.kind, 60 | 226))
    {
        return;
    }
    let flag = below.kind == 226;
    let remix = world.seeds.contains(crate::seeds::SeedFlags::REMIX);
    let mut flag2 = (j as f64) > world.rock_layer || remix;
    if flag {
        flag2 = false;
    }
    // `:59592`: the thorn conversion — Next(16) drawn only above the world
    // surface and not on 226 (the `&&` order: roll, surface, !flag).
    if rng.next_max(16) == 0 && (j as f64) > world.world_surface && !flag {
        write_plant(world, i, j, 69, 0);
        return;
    }
    // `:59598-59609`: the `&` conjunctions — BOTH draws happen regardless of
    // flag2 (the C# `&` is non-short-circuiting).
    if (rng.next_max(60) == 0) & flag2 {
        write_plant(world, i, j, 61, 144);
        return;
    }
    if (rng.next_max(230) == 0) & flag2 {
        write_plant(world, i, j, 61, 162);
        return;
    }
    // `:59610-59628`: the odd-frame arm, then the plain frame.
    if rng.next_max(15) == 0 && !flag {
        let fx = if rng.next_max(3) != 0 {
            (rng.next_max(2) * 18 + 108) as i16
        } else {
            (rng.next_max(13) * 18 + 180) as i16
        };
        write_plant(world, i, j, 61, fx);
        return;
    }
    let fx = (rng.next_max(6) * 18) as i16;
    write_plant(world, i, j, 61, fx);
}

/// The overworld jungle-plant arm (`WorldGen.cs:74815-74828`, `case 226`):
/// the `Next(7)` plant-61 roll behind the free-above gate AND the
/// `TooManyJunglePlantsNearby` density veto (the C# `&&` order: the roll
/// draws before the scan). No spread scan — the 226 tile is not a grass.
pub(super) fn overworld_jungle_plant_growth(
    world: &mut World,
    i: i32,
    j: i32,
    min_j: i32,
    rng: &mut UnifiedRandom,
) {
    // `:74271`: the entry guard binds the overworld switch too.
    if !in_world_10(world, i, j) {
        return;
    }
    // The callers gate on `nactive()` (`:72216`/`:73252`).
    if world.tile(i, j).is_some_and(|t| t.inactive()) {
        return;
    }
    if !world.tile(i, min_j).is_some_and(|t| t.active())
        && rng.next_max(7) == 0
        && !super::jungle::too_many_jungle_plants_nearby(world, i, min_j, 2)
    {
        place_jungle_plant_tile(world, i, min_j, rng);
        // `:74819-74821`: the paint copy on the cell taking.
        if world.tile(i, min_j).is_some_and(|t| t.active()) {
            copy_paint_from(world, i, min_j, i, j);
        }
    }
}

/// `Tile.CopyPaintAndCoating(other)` — the meta byte (slope+paint) and the
/// extras byte (coating) copy the growth arms pair with their placements.
pub(super) fn copy_paint_from(world: &mut World, dx: i32, dy: i32, sx: i32, sy: i32) {
    let (meta, extras) = {
        let si = world.index(sx, sy);
        (world.tiles[si].meta, world.extras_at(si))
    };
    let di = world.index(dx, dy);
    world.tiles[di].meta = meta;
    world.set_extras_at(di, extras);
}

/// The overworld ash arm (`WorldGen.cs:74829-74865`, `case 633`): the
/// `Next(10)` plant-637 roll (the free-above gate first — no draw when the
/// cell is occupied), then the 3×3 ash-grass spread over the ash 57
/// neighbours. The paint copy is gated on the placed type (`:74834`).
pub(super) fn overworld_ash_growth(
    world: &mut World,
    i: i32,
    j: i32,
    min_i: i32,
    max_i: i32,
    min_j: i32,
    max_j: i32,
    rng: &mut UnifiedRandom,
) {
    // `:74271`: the entry guard binds the overworld switch too.
    if !in_world_10(world, i, j) {
        return;
    }
    // The callers gate on `nactive()` (`:72216`/`:73252`).
    if world.tile(i, j).is_some_and(|t| t.inactive()) {
        return;
    }
    if !world.tile(i, min_j).is_some_and(|t| t.active()) && rng.next_max(10) == 0 {
        place_flower_above(world, i, j, min_j, 637, rng);
    }
    // The 3×3 ash→ash-grass scan (`:74845-74859`).
    for num11 in min_i..max_i {
        for num12 in min_j..max_j {
            if (i != num11 || j != num12)
                && world
                    .tile(num11, num12)
                    .is_some_and(|t| t.active() && t.kind == 57)
            {
                spread_grass(world, num11, num12, 57, 633);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tile::Tile;

    fn world() -> World {
        let mut w = World::new("t".into(), 4200, 400);
        w.world_surface = 250.0;
        w.rock_layer = 300.0;
        w
    }

    fn put(w: &mut World, x: i32, y: i32, kind: u16) {
        let i = w.index(x, y);
        w.tiles[i] = Tile { kind, flags: FLAG_ACTIVE, ..Default::default() };
    }

    /// W4-g42 D2: the overworld kind-2 plant roll is `Next(12)` (NOT the
    /// underground table's maxValue 1) and the flower write rides the
    /// `PlaceTile` flower ladder (`:59507-59578`). Wall 0 IS in
    /// `WallID.Sets.AllowsPlantsToGrow` (`WallID.cs:97`), so the wall arm
    /// runs even with no wall: `Next(50)` → (miss) `Next(35)` → (miss)
    /// `Next(6)` plain frame. Stream-pinned per seed.
    #[test]
    fn w4_g42_d2_overworld_plain_grass_plant_roll_and_frame_draws() {
        let mut placed = false;
        for seed in 1i32..400 {
            let mut w = world();
            put(&mut w, 100, 200, 2); // the grass source, no walls
            let mut rng = UnifiedRandom::new(seed);
            let mut witness = UnifiedRandom::new(seed);
            overworld_grass_growth(&mut w, 100, 200, 99, 102, 199, 202, true, &mut rng);
            let roll = witness.next_max(12);
            if roll == 0 {
                // The flower ladder: Next(50), then Next(35) on the miss,
                // then the plain Next(6) frame (wall 0 is not 63..70).
                let odd = witness.next_max(50);
                if odd != 0 {
                    let listed = witness.next_max(35);
                    if listed != 0 {
                        let _frame = witness.next_max(6);
                    } else {
                        let _style = witness.next_max(14); // the 14-item list
                    }
                }
                assert!(
                    w.tile(100, 199).is_some_and(|t| t.active() && t.kind == 3),
                    "seed {seed}: the hit places the flower 3"
                );
                placed = true;
            } else {
                assert!(
                    !w.tile(100, 199).is_some_and(|t| t.active()),
                    "seed {seed}: the miss places nothing"
                );
            }
            assert_eq!(rng, witness, "seed {seed}: the draw chain matches");
        }
        assert!(placed, "some stream hits the Next(12) roll");
    }

    /// W4-g42 D2: the overworld spread ladder — the corrupt grass 23
    /// converts the plain grass 2 neighbour (`:74651`, the infection arm
    /// the underground slice never ran on the surface) and the mud twin
    /// 661 spreads over the mud 59 (`:74647`). The thorn remap 32→23
    /// returns early when the spread is frozen (`:74596-74599`).
    #[test]
    fn w4_g42_d2_overworld_evil_ladder_converts_grass_and_mud() {
        let mut w = world();
        put(&mut w, 100, 200, 23); // the corrupt source
        put(&mut w, 101, 200, 2); // the plain-grass neighbour
        put(&mut w, 102, 200, 59); // the mud neighbour (air above → exposed)
        let mut rng = UnifiedRandom::new(7);
        overworld_grass_growth(&mut w, 100, 200, 99, 103, 199, 201, true, &mut rng);
        assert_eq!(w.tile(101, 200).unwrap().kind, 23, "the plain grass corrupts");
        assert_eq!(w.tile(102, 200).unwrap().kind, 661, "the mud grows the corrupt twin");
        // The frozen spread: the thorn 32 remaps to 23 then RETURNS — no
        // conversion, and the Next(3) thorn roll of the caller still runs
        // (the C# returns from GrassGrowth, not from the caller).
        let mut w2 = world();
        put(&mut w2, 100, 200, 32);
        put(&mut w2, 101, 200, 2);
        let mut rng2 = UnifiedRandom::new(7);
        let mut witness = UnifiedRandom::new(7);
        overworld_grass_growth(&mut w2, 100, 200, 99, 102, 199, 202, false, &mut rng2);
        assert_eq!(w2.tile(101, 200).unwrap().kind, 2, "the frozen spread converts nothing");
        assert_eq!(rng2, witness, "the thorn arm draws nothing (no plant row for 32)");
    }

    /// W4-g42 D2: the overworld jungle case-60 draw ladder — the `Next(7)`
    /// plant roll, ELSE `Next(500)` tree, ELSE `Next(25)` rose (`:74754-
    /// 74791`). With the cell above OCCUPIED by a non-plant tile the first
    /// arm's gate fails without a draw, the Next(500) draws (a hit still
    /// misses the plant gate — the stone is no 61/74/69), the Next(25)
    /// draws; a rose-gate hit draws Next(8) + Next(12) (both placements
    /// refuse on the stone in the box — no further draws).
    #[test]
    fn w4_g42_d2_overworld_jungle_draw_order_when_the_above_is_blocked() {
        let mut w = world();
        put(&mut w, 100, 200, 60); // the jungle grass
        put(&mut w, 100, 199, 1); // a stone block above — no plant, no tree
        let mut rng = UnifiedRandom::new(11);
        let mut witness = UnifiedRandom::new(11);
        overworld_jungle_growth(&mut w, 100, 200, 99, 102, 199, 202, &mut rng);
        let _tree_roll = witness.next_max(500);
        let rose = witness.next_max(25);
        if rose == 0 {
            let _style3 = witness.next_max(8);
            let _style2 = witness.next_max(12);
        }
        assert_eq!(rng, witness, "Next(500), Next(25), then the rose styles on a hit");
        // The free-above control: the Next(7) draws FIRST; on a miss the
        // Next(500) follows, and on its miss the Next(25) rose gate (the
        // rose placements refuse — no solid-60 ground under the side
        // columns — so a rose hit draws Next(8) + Next(12) and no more).
        let mut saw_chain = false;
        for seed in 1i32..400 {
            let mut w2 = world();
            put(&mut w2, 100, 200, 60);
            let mut rng = UnifiedRandom::new(seed);
            let mut witness = UnifiedRandom::new(seed);
            overworld_jungle_growth(&mut w2, 100, 200, 99, 102, 199, 202, &mut rng);
            let plant = witness.next_max(7);
            if plant != 0 {
                let tree = witness.next_max(500);
                if tree != 0 {
                    let rose = witness.next_max(25);
                    if rose == 0 {
                        let _s3 = witness.next_max(8);
                        let _s2 = witness.next_max(12);
                    }
                    assert_eq!(rng, witness, "seed {seed}: Next(7), Next(500), Next(25) + rose styles");
                    saw_chain = true;
                }
            }
        }
        assert!(saw_chain, "some stream walks the full miss chain");
    }

    /// W4-g42 D2: the overworld ash case-633 — the `Next(10)` plant-637
    /// roll draws only when the cell above is free; on a hit the flower
    /// ladder runs (wall 0 is in the plant set — the Next(50)/Next(35)/
    /// Next(6) chain, the 637 style list is 5 items). The 3×3 scan spreads
    /// the ash grass over the ash 57 neighbours (`:74849-74851`).
    #[test]
    fn w4_g42_d2_overworld_ash_plant_roll_and_spread() {
        let mut hit = false;
        for seed in 1i32..400 {
            let mut w = world();
            put(&mut w, 100, 200, 633);
            put(&mut w, 101, 200, 57); // the ash neighbour
            let mut rng = UnifiedRandom::new(seed);
            let mut witness = UnifiedRandom::new(seed);
            overworld_ash_growth(&mut w, 100, 200, 99, 102, 199, 202, &mut rng);
            let roll = witness.next_max(10);
            if roll == 0 {
                let odd = witness.next_max(50);
                if odd != 0 {
                    let listed = witness.next_max(35);
                    if listed != 0 {
                        let _frame = witness.next_max(6);
                    } else {
                        let _style = witness.next_max(5); // the 637 list
                    }
                }
                hit = true;
            }
            assert_eq!(rng, witness, "seed {seed}: the draw chain matches");
            assert_eq!(w.tile(101, 200).unwrap().kind, 633, "the ash spreads over the ash soil");
        }
        assert!(hit, "some stream hits the Next(10) roll");
        // The occupied-above control: no draw at all.
        let mut w2 = world();
        put(&mut w2, 100, 200, 633);
        put(&mut w2, 100, 199, 1);
        let mut rng2 = UnifiedRandom::new(7);
        let witness2 = UnifiedRandom::new(7);
        overworld_ash_growth(&mut w2, 100, 200, 99, 102, 199, 202, &mut rng2);
        assert_eq!(rng2, witness2, "the occupied cell above spends no draw");
    }

    /// W4-g42 D2: the overworld case-226 density veto — the `Next(7)` roll
    /// draws BEFORE the `TooManyJunglePlantsNearby` scan (`:74816`'s `&&`
    /// order), and three 61/74 plants in the ±9×±5 box refuse the placement.
    #[test]
    fn w4_g42_d2_overworld_jungle_plant_density_veto_spends_the_roll() {
        let mut w = world();
        put(&mut w, 100, 200, 226);
        // Three jungle plants inside the box (the cap is 2).
        put(&mut w, 96, 198, 61);
        put(&mut w, 104, 198, 74);
        put(&mut w, 100, 196, 61);
        let mut placed_any = false;
        for seed in 1i32..200 {
            let mut rng = UnifiedRandom::new(seed);
            let mut witness = UnifiedRandom::new(seed);
            overworld_jungle_plant_growth(&mut w, 100, 200, 199, &mut rng);
            let _roll = witness.next_max(7);
            assert_eq!(rng, witness, "seed {seed}: the roll draws, the veto adds none");
            if w.tile(100, 199).is_some_and(|t| t.active()) {
                placed_any = true;
            }
        }
        assert!(!placed_any, "the crowded box never places");
    }

    /// W4-g42 GROWTH FINAL: the `:74271` InWorld(i,j,10) entry guard binds
    /// the OVERWORLD switch too (it sits before the `if (underground)`
    /// split) — a grass cell at the margin draws NOTHING. The `nactive()`
    /// caller gate (`:72216`/`:73252`) folds in as well: an actuated grass
    /// grows nothing through the arm.
    #[test]
    fn w4_g42_final_the_overworld_entry_guards_skip_margin_and_actuated() {
        // The margin: the kind-2 cell at (100, 9) — outside InWorld(10).
        let mut w = world();
        put(&mut w, 100, 9, 2);
        let mut rng = UnifiedRandom::new(5);
        let witness = UnifiedRandom::new(5);
        overworld_grass_growth(&mut w, 100, 9, 99, 102, 8, 11, true, &mut rng);
        assert_eq!(rng, witness, "the margin cell draws nothing");
        // The actuated control: the same cell in-world but actuated.
        let mut w = world();
        put(&mut w, 100, 200, 2);
        let i = w.index(100, 200);
        w.tiles[i].flags |= crate::tile::FLAG_INACTIVE;
        put(&mut w, 101, 200, 0); // the dirt neighbour the arm would convert
        let mut rng = UnifiedRandom::new(5);
        let witness = UnifiedRandom::new(5);
        overworld_grass_growth(&mut w, 100, 200, 99, 102, 199, 202, true, &mut rng);
        assert_eq!(rng, witness, "the actuated cell draws nothing");
        assert_eq!(w.tile(101, 200).unwrap().kind, 0, "no spread off the actuated grass");
    }
}
