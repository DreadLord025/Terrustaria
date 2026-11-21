//! The `Floating Island Houses` pass (`WorldGen.cs:17859-17874`) +
//! `IslandHouse` (`:79203-79484`): the sunplate-202 house over each
//! non-lake island anchor the Floating Islands pass recorded
//! (`GenVars.numIslandHouses`/`skyLake`/`floatingIslandHouseX/Y/Style`),
//! with the door, the skyware chest, the fountain, the table/chairs, the
//! banners and the desert island's palms.
//!
//! Deviations: the `PlaceTile`/`Place*` `SquareTileFrame` calls ride the
//! deferred post-pipeline framing seam (the frame-important boot statics
//! keep the stored frames); the drunk crimson-half split and the
//! remix/anniversary door/table styles ride their seed flags (the classic
//! arms are the ported path); the `SwitchFountain` wiring-skip arm
//! (`:51186-51197`) is moot at gen time (`Wiring.running` false); the
//! `TryGrowingTreeByType(323)` palm arm rides the shared
//! `ocean::grow_palm_tree` (the worldgen-stream port). The chest stocking
//! rides the S1-A-4 hook through the `AddBuriedChest` core.

use super::buried_chest::{add_buried_chest, ChestSeedFlags};
use super::noise::Rng;
use crate::tile::{FLAG_ACTIVE, FLAG_HALF_BRICK};
use crate::tiledata::{TILE_SOLID, TILE_SOLID_TOP};
use crate::world::World;

/// `Main.wallDungeon` (`Main.cs:10598-10606`) — the `evil.rs` slice.
const WALL_DUNGEON: [u16; 9] = [7, 8, 9, 94, 95, 96, 97, 98, 99];

/// `WorldGen.SolidTile` (`WorldGen.cs:70111-70131`) — the `edit/kill.rs`
/// slice (OOB reads false).
fn solid_tile(world: &World, x: i32, y: i32) -> bool {
    world.tile(x, y).is_some_and(|t| {
        t.active()
            && !t.inactive()
            && TILE_SOLID.has(t.kind)
            && !TILE_SOLID_TOP.has(t.kind)
            && !t.half_brick()
            && t.slope() == 0
    })
}

/// `WorldGen.SolidTile2` (`WorldGen.cs:70492-70509`) — OOB reads solid.
fn solid_tile2(world: &World, x: i32, y: i32) -> bool {
    let Some(t) = world.tile(x, y) else {
        return true;
    };
    t.active()
        && TILE_SOLID.has(t.kind)
        && t.slope() == 0
        && !t.half_brick()
        && !t.inactive()
}

/// `WorldGen.PlaceDoor` (`WorldGen.cs:31773-31803`): the three-row door
/// from its TOP row — the nactive solid anchor two rows above and the
/// `SolidTile` floor two rows below, each row's frameX rolling the swing
/// (`Next(3) * 18`).
fn place_door(world: &mut World, x: i32, y: i32, style: i32, rng: &mut Rng) -> bool {
    let num = style / 36;
    let num2 = style % 36;
    let num3 = 54 * num;
    let num4 = 54 * num2;
    let anchored = world
        .tile(x, y - 2)
        .is_some_and(|t| t.active() && !t.inactive() && TILE_SOLID.has(t.kind))
        && solid_tile(world, x, y + 2);
    if !anchored {
        return false;
    }
    for row in 0..3 {
        if let Some(t) = world.tile_mut(x, y + row) {
            t.flags |= FLAG_ACTIVE;
            t.kind = 10;
            t.frame_y = (num4 + row * 18) as i16;
            t.frame_x = (num3 + rng.range(0, 3) * 18) as i16;
        }
    }
    true
}

/// The `PlaceTile` case-10 door dispatch (`WorldGen.cs:59843-59883`): the
/// up branch (j−1/j−2 clear, j−3 solid) places from j−1, else the down
/// branch (j+1/j+2 clear, j+3 solid) from j+1.
fn place_tile_door(world: &mut World, x: i32, y: i32, style: i32, rng: &mut Rng) {
    let clear = |w: &World, px: i32, py: i32| !w.tile(px, py).is_some_and(|t| t.active());
    let solid = |w: &World, px: i32, py: i32| {
        w.tile(px, py).is_some_and(|t| t.active() && TILE_SOLID.has(t.kind))
    };
    if clear(world, x, y - 1) && clear(world, x, y - 2) && solid(world, x, y - 3) {
        place_door(world, x, y - 1, style, rng);
    } else if !clear(world, x, y + 1) || !clear(world, x, y + 2) || !solid(world, x, y + 3) {
        return;
    } else {
        place_door(world, x, y + 1, style, rng);
    }
}

/// `WorldGen.Place3x2` (`WorldGen.cs:52066-52195`) default path for the
/// table 14 / the evil-altar-style 469: the 3×2 body empty, the floor
/// `SolidTile2` under all three columns, `frameX = 54*style + col*18`,
/// `frameY = row*18`.
fn place_3x2(world: &mut World, x: i32, y: i32, kind: u16, style: i32) {
    if x < 5 || x > world.width - 5 || y < 5 || y > world.height - 5 {
        return;
    }
    let mut ok = true;
    for col in (x - 1)..(x + 2) {
        for row in (y - 1)..(y + 1) {
            if world.tile(col, row).is_some_and(|t| t.active()) {
                ok = false;
            }
        }
        if !solid_tile2(world, col, y + 1) {
            ok = false;
        }
    }
    if !ok {
        return;
    }
    let base = 54 * style;
    for col in 0..3 {
        for row in 0..2 {
            if let Some(t) = world.tile_mut(x - 1 + col, y - 1 + row) {
                t.flags |= FLAG_ACTIVE;
                t.kind = kind;
                t.frame_x = (base + col * 18) as i16;
                t.frame_y = (row * 18) as i16;
            }
        }
    }
}

/// `WorldGen.Place1x2` (`WorldGen.cs:46652-46679`) for the chair 15: the
/// `SolidTile2` floor + the top cell empty, `frameY = 40*style + row*18`,
/// frameX 0 (the sapling swing roll is kind-20 only).
fn place_chair(world: &mut World, x: i32, y: i32, style: i32) {
    if !solid_tile2(world, x, y + 1) || world.tile(x, y - 1).is_some_and(|t| t.active()) {
        return;
    }
    let base = style * 40;
    for row in 0..2 {
        if let Some(t) = world.tile_mut(x, y - 1 + row) {
            t.flags |= FLAG_ACTIVE;
            t.kind = 15;
            t.frame_y = (base + row * 18) as i16;
            t.frame_x = 0;
        }
    }
}

/// `WorldGen.PlaceBanner` (`WorldGen.cs:46258-46300`): the 3-row banner
/// hung from a nactive solid non-top ceiling, three free rows below.
fn place_banner(world: &mut World, x: i32, y: i32, style: i32) {
    let mut frame_x = style * 18;
    let mut frame_y = 0;
    let mut wrapped = style;
    while wrapped >= 111 {
        wrapped -= 111;
        frame_x -= 1998;
        frame_y += 54;
    }
    let hangs = world.tile(x, y - 1).is_some_and(|t| {
        t.active() && !t.inactive() && TILE_SOLID.has(t.kind) && !TILE_SOLID_TOP.has(t.kind)
    });
    if !hangs
        || world.tile(x, y).is_some_and(|t| t.active())
        || world.tile(x, y + 1).is_some_and(|t| t.active())
        || world.tile(x, y + 2).is_some_and(|t| t.active())
    {
        return;
    }
    for row in 0..3 {
        if let Some(t) = world.tile_mut(x, y + row) {
            t.flags |= FLAG_ACTIVE;
            t.kind = 91;
            t.frame_x = frame_x as i16;
            t.frame_y = (frame_y + row * 18) as i16;
        }
    }
}

/// `WorldGen.Place2xX` (`WorldGen.cs:39663-39722`) for the fountain 207:
/// four body rows (`num3 = 4`, `:39684-39687`), the `SolidTile2` floor
/// under both columns, `frameX = 36*style + col*18`, `frameY = row*18`.
fn place_fountain(world: &mut World, x: i32, y: i32, style: i32) {
    let num3 = 4;
    let mut ok = true;
    for row in (y - num3 + 1)..(y + 1) {
        for col in x..(x + 2) {
            if world.tile(col, row).is_some_and(|t| t.active()) {
                ok = false;
            }
        }
    }
    if !solid_tile2(world, x, y + 1) || !solid_tile2(world, x + 1, y + 1) {
        ok = false;
    }
    if !ok {
        return;
    }
    for row in 0..num3 {
        for col in 0..2 {
            if let Some(t) = world.tile_mut(x + col, y - num3 + 1 + row) {
                t.flags |= FLAG_ACTIVE;
                t.kind = 207;
                t.frame_x = (36 * style + col * 18) as i16;
                t.frame_y = (row * 18) as i16;
            }
        }
    }
}

/// `WorldGen.SwitchFountain` (`WorldGen.cs:51150-51185`): the 2×4 frame-Y
/// toggle (+72 / −72) over the fountain's body — the wiring-skip tail is
/// moot at gen time.
fn switch_fountain(world: &mut World, i: i32, j: i32) {
    let (fx, fy) = world.tile(i, j).map_or((0, 0), |t| (t.frame_x, t.frame_y));
    let mut num3 = fx / 18;
    while num3 >= 2 {
        num3 -= 2;
    }
    let mut num4 = fy / 18;
    if num4 >= 4 {
        num4 -= 4;
    }
    let (num, num2) = (i - num3 as i32, j - num4 as i32);
    for k in num..num + 2 {
        for l in num2..num2 + 4 {
            if let Some(t) = world.tile_mut(k, l) {
                if t.active() && t.kind == 207 {
                    if t.frame_y < 72 {
                        t.frame_y += 72;
                    } else {
                        t.frame_y -= 72;
                    }
                }
            }
        }
    }
}

/// `WorldGen.IslandHouse` (`WorldGen.cs:79203-79484`). The chest arm rides
/// the `AddBuriedChest` core + the S1-A-4 stocking hook; the palm arm rides
/// `ocean::grow_palm_tree`.
#[allow(clippy::too_many_arguments)]
fn island_house(
    world: &mut World,
    gv: &mut super::pipeline::GenVars,
    i: i32,
    j: i32,
    island_style: i32,
    world_surface: f64,
    rock_layer: f64,
    flags: ChestSeedFlags,
    rng: &mut Rng,
    mut stock: Option<&mut super::chest_stock::ChestStockHook<'_>>,
) {
    // `:79205-79220` — the ±30 dungeon/evil/lihzahrd wall veto and the
    // drunk crimson-half split.
    let mut crimson = flags.crimson;
    for k in i - 30..=i + 30 {
        for l in j - 30..=j + 30 {
            if k >= 0 && l >= 0 && k < world.width && l < world.height {
                let wall = world.tile(k, l).map_or(0, |t| t.wall);
                if WALL_DUNGEON.contains(&wall) || wall == 87 || wall == 3 || wall == 83 {
                    return;
                }
            }
        }
    }
    if flags.drunk {
        crimson = (flags.crimson_left && i < world.width / 2)
            || (!flags.crimson_left && i > world.width / 2);
    }
    let kind: u16 = 202; // the sunplate block
    let wall: u16 = 82; // the sunplate wall
    // `:79224-79230` — the door side and the house dimensions.
    let mut num2 = 1;
    if rng.range(0, 2) == 0 {
        num2 = -1;
    }
    let num3 = rng.range(7, 12);
    let num4 = rng.range(5, 7);
    // `:79231-79247` — the floor scan at the door column.
    let scan_x = i + (num3 + 2) * num2;
    let mut floor_y = j;
    if (j as f64) > world_surface {
        floor_y = j - 5;
    } else {
        for m in j - 15..j + 30 {
            let t = world.tile(scan_x, m);
            if t.is_some_and(|t| {
                t.wall > 0
                    && t.active()
                    && solid_tile(world, scan_x, m)
                    && !matches!(t.kind, 192 | 191 | 384)
            }) {
                floor_y = m - 1;
                break;
            }
        }
    }
    // `:79249-79283` — the outer sunplate shell (the corner-cut top row).
    let mut num5 = (i - num3 - 1).max(0);
    let mut num6 = (i + num3 + 1).min(world.width);
    let mut num7 = (floor_y - num4 - 1).max(0);
    let mut num8 = (floor_y + 2).min(world.height);
    for n in num5..=num6 {
        for num9 in num7 - 1..num8 + 1 {
            if num9 != num7 - 1 || (n != num5 && n != num6) {
                if let Some(t) = world.tile_mut(n, num9) {
                    t.flags |= FLAG_ACTIVE;
                    t.liquid = 0;
                    t.kind = kind;
                    t.wall = 0;
                    t.flags &= !FLAG_HALF_BRICK;
                    t.set_slope(0);
                }
            }
        }
    }
    // `:79284-79314` — the interior carve with the sunplate wall.
    num5 = (i - num3).max(0);
    num6 = (i + num3).min(world.width);
    num7 = (floor_y - num4).max(0);
    num8 = (floor_y + 1).min(world.height);
    for num10 in num5..=num6 {
        for num11 in num7..num8 {
            if (num11 != num7 || (num10 != num5 && num10 != num6))
                && world.tile(num10, num11).is_some_and(|t| t.wall == 0)
            {
                if let Some(t) = world.tile_mut(num10, num11) {
                    t.flags &= !FLAG_ACTIVE;
                    t.wall = wall;
                }
            }
        }
    }
    // `:79315-79337` — the doorway carve + the door.
    let mut num12 = i + (num3 + 1) * num2;
    let num13 = floor_y;
    for num14 in num12 - 2..=num12 + 2 {
        for dy in 0..3 {
            if let Some(t) = world.tile_mut(num14, num13 - dy) {
                t.flags &= !FLAG_ACTIVE;
            }
        }
    }
    let door_style = if flags.remix && (!flags.anniversary || flags.get_good) {
        if crimson { 5 } else { 38 }
    } else {
        9
    };
    place_tile_door(world, num12, num13, door_style, rng);
    // `:79338-79347` — the far wall column.
    num12 = i + (num3 + 1) * -num2 - num2;
    for num15 in num7..=num8 + 1 {
        if let Some(t) = world.tile_mut(num12, num15) {
            t.flags |= FLAG_ACTIVE;
            t.liquid = 0;
            t.kind = kind;
            t.wall = 0;
            t.flags &= !FLAG_HALF_BRICK;
            t.set_slope(0);
        }
    }
    // `:79348-79376` — the skyware chest (the skyIslandHouseCount cycle).
    let mut num16 = gv.sky_island_house_count;
    if num16 > 3 {
        num16 = rng.range(0, 4) as u32;
    }
    let main: u16 = match num16 {
        0 => 159,
        1 => 65,
        2 => 158,
        _ => 2219,
    };
    let chest_style = if flags.get_good { 2 } else { 13 };
    if let Some(p) = add_buried_chest(
        world, i, num13 - 3, main, false, chest_style, false, 0,
        world_surface, rock_layer, flags,
        gv.desert_hive_high, gv.desert_hive_low,
        &mut gv.hell_chest, gv.hell_chest_item.len(), rng,
    ) {
        super::chest_stock::stock_placed_chest(
            world, stock.as_deref_mut(), rng, p.x, p.y, p.style, p.kind, main, false,
        );
    }
    // `:79377-79393` — the fountain search (islandStyle > 0).
    if island_style > 0 {
        for num17 in 0..100000 {
            let num18 = i + rng.range(-50, 51);
            let num19 = num13 + rng.range(0, 21);
            let kind_here = world.tile(num18, num19).map_or(0, |t| t.kind);
            if (num17 >= 50000 || kind_here != 202)
                && !world.tile(num18, num19).is_some_and(|t| t.active())
            {
                place_fountain(world, num18, num19, island_style);
                if world.tile(num18, num19).is_some_and(|t| t.active()) {
                    switch_fountain(world, num18, num19);
                    break;
                }
            }
        }
    }
    // `:79394` — the counter advance.
    gv.sky_island_house_count += 1;
    // `:79395-79419` — the glass-wall windows (the classic arm).
    if !flags.remix || (flags.anniversary && !flags.get_good) {
        let num20 = i - num3 / 2 + 1;
        let num21 = i + num3 / 2 - 1;
        let num22 = if num3 > 10 { 2 } else { 1 };
        let num23 = (num7 + num8) / 2 - 1;
        for num24 in num20 - num22..=num20 + num22 {
            for num25 in num23 - 1..=num23 + 1 {
                if let Some(t) = world.tile_mut(num24, num25) {
                    t.wall = 21;
                }
            }
        }
        for num26 in num21 - num22..=num21 + num22 {
            for num27 in num23 - 1..=num23 + 1 {
                if let Some(t) = world.tile_mut(num26, num27) {
                    t.wall = 21;
                }
            }
        }
    }
    // `:79420-79443` — the table + the two chairs (the left chair's frame
    // shift +18 on both rows).
    let num28 = i + (num3 / 2 + 1) * -num2;
    let (table_kind, table_style, chair_style) =
        if flags.remix && (!flags.anniversary || flags.get_good) {
            if crimson { (14u16, 5, 8) } else { (469, 2, 38) }
        } else {
            (14, 7, 10)
        };
    place_3x2(world, num28, num8 - 1, table_kind, table_style);
    place_chair(world, num28 - 2, num8 - 1, chair_style);
    place_chair(world, num28 + 2, num8 - 1, chair_style);
    for dy in 0..2 {
        if let Some(t) = world.tile_mut(num28 - 2, num8 - 1 - dy) {
            t.frame_x += 18;
        }
    }
    // `:79444-79461` — the three banners (the classic arm).
    if !flags.remix || (flags.anniversary && !flags.get_good) {
        place_banner(world, num5 + 1, num7, rng.range(7, 10));
        place_banner(world, num6 - 1, num7, rng.range(7, 10));
        let (i2, j2) = if num2 > 0 { (num5, num7 + 1) } else { (num6, num7 + 1) };
        place_banner(world, i2, j2, rng.range(7, 10));
    }
    // `:79462-79483` — the desert island's palms (islandStyle == 1).
    if island_style != 1 {
        return;
    }
    let mut num29 = rng.range(3, 6);
    for _ in 0..100000 {
        let num31 = i + rng.range(-50, 51);
        let num32 = num13 + rng.range(-10, 21);
        if !world.tile(num31, num32).is_some_and(|t| t.active()) {
            super::ocean::grow_palm_tree(world, num31, num32 + 1, 0, false, rng);
            if world.tile(num31, num32).is_some_and(|t| t.active()) {
                num29 -= 1;
            }
        }
        if num29 <= 0 {
            break;
        }
    }
}

/// The `Floating Island Houses` pass (`WorldGen.cs:17859-17874`): one
/// `IslandHouse` per non-lake ledger entry. The skyblock gate
/// (`denyFloatingIslands`, `:17861`/`:3087-3097`) rides the step adapter.
#[allow(clippy::too_many_arguments)]
pub(crate) fn pass_floating_island_houses(
    world: &mut World,
    gv: &mut super::pipeline::GenVars,
    world_surface: f64,
    rock_layer: f64,
    flags: ChestSeedFlags,
    rng: &mut Rng,
    mut stock: Option<&mut super::chest_stock::ChestStockHook<'_>>,
) {
    for idx in 0..gv.num_island_houses as usize {
        if !gv.sky_lake[idx] {
            island_house(
                world,
                gv,
                gv.floating_island_house_x[idx],
                gv.floating_island_house_y[idx],
                gv.floating_island_style[idx],
                world_surface,
                rock_layer,
                flags,
                rng,
                stock.as_deref_mut(),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A sky world with one recorded island anchor over a sunplate disc.
    fn island_world() -> (World, super::super::pipeline::GenVars) {
        let mut world = World::new("ih".into(), 800, 500);
        // A sunplate disc at rows 96-104, cols 380-420 (the island body).
        for y in 96..105 {
            for x in 380..421 {
                let i = world.index(x, y);
                world.tiles[i].kind = 189;
                world.tiles[i].flags |= FLAG_ACTIVE;
            }
        }
        let mut gv = super::super::pipeline::GenVars::default();
        gv.num_island_houses = 1;
        gv.sky_lake[0] = false;
        gv.floating_island_house_x[0] = 400;
        gv.floating_island_house_y[0] = 100;
        gv.floating_island_style[0] = 0;
        (world, gv)
    }

    #[test]
    fn island_house_builds_the_sunplate_shell() {
        let (mut world, mut gv) = island_world();
        pass_floating_island_houses(
            &mut world, &mut gv, 300.0, 400.0, ChestSeedFlags::default(), &mut Rng(42), None,
        );
        let sunplate = world.tiles.iter().filter(|t| t.active() && t.kind == 202).count();
        assert!(sunplate > 50, "the sunplate shell lands ({sunplate})");
        assert_eq!(gv.sky_island_house_count, 1, "the counter advances");
    }

    #[test]
    fn island_house_skips_the_lake_entries() {
        let (mut world, mut gv) = island_world();
        gv.sky_lake[0] = true;
        pass_floating_island_houses(
            &mut world, &mut gv, 300.0, 400.0, ChestSeedFlags::default(), &mut Rng(42), None,
        );
        let sunplate = world.tiles.iter().filter(|t| t.active() && t.kind == 202).count();
        assert_eq!(sunplate, 0, "the lake entry builds nothing");
        assert_eq!(gv.sky_island_house_count, 0);
    }

    #[test]
    fn island_house_vetoes_the_dungeon_wall() {
        let (mut world, mut gv) = island_world();
        let i = world.index(400, 100);
        world.tiles[i].wall = 7;
        pass_floating_island_houses(
            &mut world, &mut gv, 300.0, 400.0, ChestSeedFlags::default(), &mut Rng(42), None,
        );
        let sunplate = world.tiles.iter().filter(|t| t.active() && t.kind == 202).count();
        assert_eq!(sunplate, 0, "the dungeon wall vetoes the house");
    }

    #[test]
    fn island_houses_pass_is_deterministic_for_a_seed() {
        let (base, gvb) = island_world();
        let mut a = base.clone();
        let mut b = base;
        let mut gva = gvb.clone();
        let mut gvb2 = gvb;
        pass_floating_island_houses(&mut a, &mut gva, 300.0, 400.0, ChestSeedFlags::default(), &mut Rng(42), None);
        pass_floating_island_houses(&mut b, &mut gvb2, 300.0, 400.0, ChestSeedFlags::default(), &mut Rng(42), None);
        assert_eq!(a.tiles, b.tiles, "a == b");
    }
}
