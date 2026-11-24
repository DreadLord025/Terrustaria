//! The `Floating Island Houses` pass (`WorldGen.cs:17859-17874`) +
//! `IslandHouse` (`WorldGen.cs:79203-79484`) — W4-d104 pass 28.
//!
//! Vanilla builds the houses in their own pass, over the
//! `GenVars.floatingIslandHouseX/Y/Style/skyLake` ledger the `Floating
//! Islands` pass records (`:13069-13111`); the port's compact islands
//! model used to build a shape-level plank box inline — this module
//! replaces that with the faithful sunplate house at the vanilla slot,
//! on the fresh `pass_rng("Floating Island Houses")` stream.
//!
//! Deviations (ANALYSIS "Island houses (W4-d104)"): the compact island
//! model records its own anchors (`cx`, the disc top row) and style 0
//! for classic worlds, so the fountain block (`islandStyle > 0`) and the
//! style-1 palm block never fire on a classic world — the arms are
//! ported verbatim for the seed lanes. The chest rides the port's
//! compact `AddBuriedChest` slice (the sky band of `:35871-36210` —
//! the scan guards, the floor walk, the style-13/2 `PlaceChest` and the
//! S1-A-4 stocking hook); the full loot-branch tree is the chest lane's.

use super::chest_stock::{stock_placed_chest, ChestStockHook};
use super::structures::{place_chest, resolved_chest_style};
use super::temple::{in_world, place_temple_door};
use super::Rng;
use crate::tile::FLAG_ACTIVE;
use crate::tiledata as td;
use crate::world::World;

/// `Main.wallDungeon` (`Main.cs`'s `wallDungeon[..] = true` rows — the
/// port's shared copy, `worldgen/evil.rs:111`).
const WALL_DUNGEON: [u16; 9] = [7, 8, 9, 94, 95, 96, 97, 98, 99];
/// `TileID.Sunplate` (`TileID.cs:1033`).
const TILE_SUNPLATE: u16 = 202;
/// `WallID.Sunplate` — the house interior wall (`WorldGen.cs:79222`).
const WALL_SUNPLATE: u16 = 82;

/// One island's house slot — the port's slice of the
/// `GenVars.floatingIslandHouseX/Y` + `skyLake` + `floatingIslandStyle`
/// arrays (`WorldGen.cs:13108-13111`).
#[derive(Clone, Copy, Debug)]
pub(crate) struct IslandHouseSlot {
    pub x: i32,
    pub y: i32,
    pub style: i32,
    pub sky_lake: bool,
}

/// `WorldGen.SolidTile(i, j)` (`WorldGen.cs:70111-70132`) at the pass's
/// window — `tileSolid[202]` is still true here (the Piles pass flips it
/// later, `:18802`); an out-of-world cell reads as the C#'s null tile.
fn solid_tile(world: &World, x: i32, y: i32) -> bool {
    let Some(t) = world.tile(x, y) else {
        return true;
    };
    t.active()
        && td::TILE_SOLID.has(t.kind)
        && !td::TILE_SOLID_TOP.has(t.kind)
        && !t.half_brick()
        && t.slope() == 0
        && !t.inactive()
}

/// `WorldGen.SolidTile2(i, j)` (`WorldGen.cs:70134-70151`, `:70492`).
fn solid_tile2(world: &World, x: i32, y: i32) -> bool {
    let Some(t) = world.tile(x, y) else {
        return true;
    };
    t.active() && td::TILE_SOLID.has(t.kind) && t.slope() == 0 && !t.half_brick() && !t.inactive()
}

/// The `Floating Island Houses` pass body (`WorldGen.cs:17859-17874`):
/// `for i < GenVars.numIslandHouses: if !skyLake[i] IslandHouse(x, y,
/// style)`. The `Skyblock.denyFloatingIslands` gate rides the step
/// adapter (the port's SKYBLOCK flag).
pub(crate) fn pass_floating_island_houses(
    world: &mut World,
    slots: &[IslandHouseSlot],
    rng: &mut Rng,
    mut stock: Option<&mut ChestStockHook<'_>>,
    get_good: bool,
    drunk: bool,
    remix: bool,
    anniversary: bool,
    crimson: bool,
    crimson_left: bool,
    sky_count: &mut u32,
) {
    for slot in slots {
        if !slot.sky_lake {
            island_house(
                world,
                slot.x,
                slot.y,
                slot.style,
                rng,
                stock.as_deref_mut(),
                get_good,
                drunk,
                remix,
                anniversary,
                crimson,
                crimson_left,
                sky_count,
            );
        }
    }
}

/// `WorldGen.IslandHouse(i, j, islandStyle)` (`WorldGen.cs:79203-79484`):
/// the sunplate house on the island's surface — the wall veto, the
/// direction/width/height rolls, the floor scan, the sunplate shell +
/// interior, the door, the chest, the fountain, the windows, the
/// table/chairs, the banners and the style-1 palms.
#[allow(clippy::too_many_arguments)]
fn island_house(
    world: &mut World,
    i: i32,
    j: i32,
    island_style: i32,
    rng: &mut Rng,
    stock: Option<&mut ChestStockHook<'_>>,
    get_good: bool,
    drunk: bool,
    remix: bool,
    anniversary: bool,
    crimson: bool,
    crimson_left: bool,
    sky_count: &mut u32,
) {
    // `:79205-79216`: the wall veto — a dungeon / 87 / 3 / 83 wall in the
    // ±30 box refuses the house.
    let mut flag = crimson;
    let num = 30;
    for k in (i - num)..=(i + num) {
        for l in (j - num)..=(j + num) {
            if in_world(world, k, l, 0) {
                let wall = world.tile(k, l).map(|t| t.wall).unwrap_or(0);
                if WALL_DUNGEON.contains(&wall) || wall == 87 || wall == 3 || wall == 83 {
                    return;
                }
            }
        }
    }
    // `:79217-79220`: the drunk half-split evil flavour.
    if drunk {
        flag = (crimson_left && i < world.width / 2)
            || (!crimson_left && i > world.width / 2);
    }
    let kind = TILE_SUNPLATE;
    let wall = WALL_SUNPLATE;
    // `:79223-79230`: the direction, width and height rolls.
    let mut vx = i as f64;
    let mut vy = j as f64;
    let mut num2 = 1;
    if rng.range(0, 2) == 0 {
        num2 = -1;
    }
    let num3 = rng.range(7, 12);
    let num4 = rng.range(5, 7);
    vx = (i + (num3 + 2) * num2) as f64;
    // `:79232-79247`: the floor row — below the surface it is j-5, else
    // the first walled solid cell in the j-15..j+30 column scan.
    if j as f64 > world.world_surface {
        vy = (j - 5) as f64;
    } else {
        for m in (j - 15)..(j + 30) {
            let t = world.tile(vx as i32, m);
            if let Some(t) = t {
                if t.wall > 0
                    && t.active()
                    && solid_tile(world, vx as i32, m)
                    && t.kind != 192
                    && t.kind != 191
                    && t.kind != 384
                {
                    vy = (m - 1) as f64;
                    break;
                }
            }
        }
    }
    vx = i as f64;
    // `:79249-79268`: the outer shell bounds (clamped to the world).
    let mut num5 = (vx - num3 as f64 - 1.0) as i32;
    let mut num6 = (vx + num3 as f64 + 1.0) as i32;
    let mut num7 = (vy - num4 as f64 - 1.0) as i32;
    let mut num8 = (vy + 2.0) as i32;
    num5 = num5.max(0);
    num6 = num6.min(world.width);
    num7 = num7.max(0);
    num8 = num8.min(world.height);
    // `:79269-79283`: the sunplate shell (the top corners skipped).
    for n in num5..=num6 {
        for num9 in (num7 - 1)..(num8 + 1) {
            if num9 != num7 - 1 || (n != num5 && n != num6) {
                if let Some(t) = world.tile_mut(n, num9) {
                    t.flags |= FLAG_ACTIVE;
                    t.liquid = 0;
                    t.kind = kind;
                    t.wall = 0;
                    t.flags &= !crate::tile::FLAG_HALF_BRICK;
                    t.set_slope(0);
                }
            }
        }
    }
    // `:79284-79314`: the interior carve + sunplate wall.
    num5 = (vx - num3 as f64) as i32;
    num6 = (vx + num3 as f64) as i32;
    num7 = (vy - num4 as f64) as i32;
    num8 = (vy + 1.0) as i32;
    num5 = num5.max(0);
    num6 = num6.min(world.width);
    num7 = num7.max(0);
    num8 = num8.min(world.height);
    for num10 in num5..=num6 {
        for num11 in num7..num8 {
            if (num11 != num7 || (num10 != num5 && num10 != num6))
                && world.tile(num10, num11).map(|t| t.wall).unwrap_or(0) == 0
            {
                if let Some(t) = world.tile_mut(num10, num11) {
                    t.flags &= !FLAG_ACTIVE;
                    t.wall = wall;
                }
            }
        }
    }
    // `:79315-79322`: the door-side clearance (three rows).
    let mut num12 = i + (num3 + 1) * num2;
    let num13 = vy as i32;
    for num14 in (num12 - 2)..=(num12 + 2) {
        for dy in 0..3 {
            if let Some(t) = world.tile_mut(num14, num13 - dy) {
                t.flags &= !FLAG_ACTIVE;
            }
        }
    }
    // `:79323-79337`: the door (the remix evil-flavour styles, else 9).
    if remix && (!anniversary || get_good) {
        let style = if flag { 5 } else { 38 };
        place_temple_door(world, num12, num13, style, rng);
    } else {
        place_temple_door(world, num12, num13, 9, rng);
    }
    // `:79338-79347`: the far sunplate column.
    num12 = i + (num3 + 1) * -num2 - num2;
    for num15 in num7..=(num8 + 1) {
        if let Some(t) = world.tile_mut(num12, num15) {
            t.flags |= FLAG_ACTIVE;
            t.liquid = 0;
            t.kind = kind;
            t.wall = 0;
            t.flags &= !crate::tile::FLAG_HALF_BRICK;
            t.set_slope(0);
        }
    }
    // `:79348-79376`: the chest main item (the skyIslandHouseCount cycle)
    // and the AddBuriedChest call (style 2 under getGoodWorldGen, else 13).
    let mut num16 = *sky_count;
    if num16 > 3 {
        num16 = rng.range(0, 4) as u32;
    }
    let main_item = match num16 {
        0 => 159,
        1 => 65,
        2 => 158,
        _ => 2219,
    };
    let chest_style = if get_good { 2 } else { 13 };
    add_sky_chest(world, i, num13 - 3, main_item, chest_style, rng, stock);
    // `:79377-79393`: the fountain (islandStyle > 0) — the 100000-attempt
    // scatter, Place2xX(207) + SwitchFountain.
    if island_style > 0 {
        for num17 in 0..100000 {
            let num18 = i + rng.range(-50, 51);
            let num19 = num13 + rng.range(0, 21);
            let kind_here = world.tile(num18, num19).map(|t| t.kind).unwrap_or(0);
            let active_here = world.tile(num18, num19).is_some_and(|t| t.active());
            if (num17 >= 50000 || kind_here != TILE_SUNPLATE) && !active_here {
                place_2x_x(world, num18, num19, 207, island_style);
                if world.tile(num18, num19).is_some_and(|t| t.active()) {
                    switch_fountain(world, num18, num19);
                    break;
                }
            }
        }
    }
    // `:79394`: the counter advances once per house, after the place.
    *sky_count += 1;
    // `:79395-79419`: the two windows (wall 21) on the non-remix arm.
    if !remix || (anniversary && !get_good) {
        let num20 = i - num3 / 2 + 1;
        let num21 = i + num3 / 2 - 1;
        let mut num22 = 1;
        if num3 > 10 {
            num22 = 2;
        }
        let num23 = (num7 + num8) / 2 - 1;
        for num24 in (num20 - num22)..=(num20 + num22) {
            for num25 in (num23 - 1)..=(num23 + 1) {
                if let Some(t) = world.tile_mut(num24, num25) {
                    t.wall = 21;
                }
            }
        }
        for num26 in (num21 - num22)..=(num21 + num22) {
            for num27 in (num23 - 1)..=(num23 + 1) {
                if let Some(t) = world.tile_mut(num26, num27) {
                    t.wall = 21;
                }
            }
        }
    }
    // `:79420-79443`: the table + two chairs (the remix evil-flavour
    // styles, else table 14 style 7 / chairs 15 style 10), then the
    // left-chair frameX bump.
    let num28 = i + (num3 / 2 + 1) * -num2;
    if remix && (!anniversary || get_good) {
        if flag {
            super::evil::place_3x2(world, num28, num8 - 1, 14, 5);
            place_1x2(world, num28 - 2, num8 - 1, 15, 8);
            place_1x2(world, num28 + 2, num8 - 1, 15, 8);
        } else {
            super::evil::place_3x2(world, num28, num8 - 1, 469, 2);
            place_1x2(world, num28 - 2, num8 - 1, 15, 38);
            place_1x2(world, num28 + 2, num8 - 1, 15, 38);
        }
    } else {
        super::evil::place_3x2(world, num28, num8 - 1, 14, 7);
        place_1x2(world, num28 - 2, num8 - 1, 15, 10);
        place_1x2(world, num28 + 2, num8 - 1, 15, 10);
    }
    if let Some(t) = world.tile_mut(num28 - 2, num8 - 1) {
        t.frame_x += 18;
    }
    if let Some(t) = world.tile_mut(num28 - 2, num8 - 2) {
        t.frame_x += 18;
    }
    // `:79444-79461`: the three banners (91) on the non-remix arm.
    if !remix || (anniversary && !get_good) {
        super::dungeon_features::place_banner(world, num5 + 1, num7, 91, rng.range(7, 10));
        super::dungeon_features::place_banner(world, num6 - 1, num7, 91, rng.range(7, 10));
        let (i2, j2) = if num2 > 0 { (num5, num7 + 1) } else { (num6, num7 + 1) };
        super::dungeon_features::place_banner(world, i2, j2, 91, rng.range(7, 10));
    }
    // `:79462-79483`: the style-1 palms (TryGrowingTreeByType 323).
    if island_style != 1 {
        return;
    }
    let mut num29 = rng.range(3, 6);
    for _ in 0..100000 {
        let num31 = i + rng.range(-50, 51);
        let num32 = num13 + rng.range(-10, 21);
        if !world.tile(num31, num32).is_some_and(|t| t.active()) {
            // `TryGrowingTreeByType(323, …)` (`:30343-30354`) is
            // `GrowPalmTree(x, y, 0, false)` — the ocean port carries it.
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

/// `WorldGen.AddBuriedChest(i, num13 - 3, mainItem, notNearOtherChests:
/// false, 13/2, trySlope: false, 0)`'s sky-band slice
/// (`WorldGen.cs:35871-36210`): the `chestTileType 0 → 21` default, the
/// floor scan (the 231 larva rejection, the ±2 altar 26/237 rejection —
/// the C#'s `Boulders` arm is inert during worldgen, D20), the
/// `SolidTile` floor gate, then the 2×2 `PlaceChest` at `(i - 1, k - 1)`
/// with the resolved style and the S1-A-4 stocking hook. The full
/// loot-branch tree (`flag2/3/4…`) never fires on the sky band (styles
/// 13/2, main item > 0, above the surface) — the chest lane owns it.
/// Deviation: the C#'s `anyShimmer()` veto (`:35904`) reads liquid-only
/// shimmer; the port's shimmer lane publishes no tiles before this pass
/// on a classic world, so the probe is deferred with that lane.
fn add_sky_chest(
    world: &mut World,
    x: i32,
    y: i32,
    main_item: u16,
    chest_style: i32,
    rng: &mut Rng,
    stock: Option<&mut ChestStockHook<'_>>,
) {
    for k in y..(world.height - 10) {
        // `:35908-35911`: the larva veto.
        if world.tile(x, k).is_some_and(|t| t.active() && t.kind == 231) {
            return;
        }
        // `:35950-35960`: the ±2 altar veto (`InWorld(n, m, 100)`).
        for n in (x - 2)..=(x + 2) {
            for m in (k - 2)..=(k + 2) {
                if in_world(world, n, m, 100)
                    && world
                        .tile(n, m)
                        .is_some_and(|t| t.active() && (t.kind == 26 || t.kind == 237))
                {
                    return;
                }
            }
        }
        // `:35961-35964`: the floor gate — `SolidTile(i, k)`.
        if !solid_tile(world, x, k) {
            continue;
        }
        // `:35968-35981`: `num9` resolves to the caller's style (13/2,
        // both >= 0); the chest places at `(i - 1, k - 1)` — the C#'s
        // BOTTOM-left anchor (`TileObject` chest origin `(0, 1)`,
        // `tileobject.rs`'s def test), so the body rides rows `k-2..k-1`
        // on the floor `k`. The compact `place_chest` is TOP-left — the
        // anchor maps to `(i - 1, k - 2)`.
        let style = resolved_chest_style(chest_style, main_item, k, world.world_surface);
        if place_chest(world, x - 1, k - 2, style) {
            stock_placed_chest(
                world,
                stock,
                rng,
                x - 1,
                k - 2,
                style,
                21,
                main_item,
                false,
            );
        }
        return;
    }
}

/// `WorldGen.Place2xX` (`WorldGen.cs:39663-39722`) — the 2-wide,
/// `num3`-tall furniture write: the clear scan over the body, the
/// `SolidTile2` floor under both columns, then the frame grid
/// (`frameX = style * 36 (+18 right)`, `frameY = j * 18`). The ported
/// window is the fountain call's (`type 207 → num3 = 4`); the 105/104
/// statue/clock arms stay with their callers.
fn place_2x_x(world: &mut World, x: i32, y: i32, kind: u16, style: i32) {
    let num = style * 36;
    let num3 = if kind == 207 { 4 } else { 3 };
    let mut flag = true;
    for i in (y - num3 + 1)..(y + 1) {
        for px in [x, x + 1] {
            if world.tile(px, i).is_some_and(|t| t.active()) {
                flag = false;
            }
        }
    }
    if flag && solid_tile2(world, x, y + 1) && solid_tile2(world, x + 1, y + 1) {
        for j in 0..num3 {
            let py = y - num3 + 1 + j;
            for (px, fx) in [(x, num), (x + 1, num + 18)] {
                if let Some(t) = world.tile_mut(px, py) {
                    t.flags |= FLAG_ACTIVE;
                    t.frame_y = (j * 18) as i16;
                    t.frame_x = fx as i16;
                    t.kind = kind;
                }
            }
        }
    }
}

/// `WorldGen.SwitchFountain` (`WorldGen.cs:51150-51193`): walk the frame
/// back to the fountain's top-left cell, then flip `frameY` by ±72 over
/// the 2×4 body (the on/off animation state). The `Wiring.running` half
/// never fires during worldgen (`Wiring.cs`'s runtime flag).
fn switch_fountain(world: &mut World, i: i32, j: i32) {
    let Some(anchor) = world.tile(i, j) else {
        return;
    };
    let mut num3 = anchor.frame_x / 18;
    while num3 >= 2 {
        num3 -= 2;
    }
    let mut num4 = anchor.frame_y / 18;
    if num4 >= 4 {
        num4 -= 4;
    }
    let (num, num2) = (i - num3 as i32, j - num4 as i32);
    for k in num..(num + 2) {
        for l in num2..(num2 + 4) {
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

/// `WorldGen.Place1x2` (`WorldGen.cs:46652-46679`): the chair-style 1×2 —
/// the `SolidTile2` floor under the anchor and a clear row above, then
/// the two-row write (`frameY = style * 40 (+18 bottom)`). The type-20
/// sapling `Next(3)` frame roll never fires on the house's chairs (15).
fn place_1x2(world: &mut World, x: i32, y: i32, kind: u16, style: i32) {
    if solid_tile2(world, x, y + 1) && !world.tile(x, y - 1).is_some_and(|t| t.active()) {
        let num = (style * 40) as i16;
        if let Some(t) = world.tile_mut(x, y - 1) {
            t.flags |= FLAG_ACTIVE;
            t.frame_y = num;
            t.frame_x = 0;
            t.kind = kind;
        }
        if let Some(t) = world.tile_mut(x, y) {
            t.flags |= FLAG_ACTIVE;
            t.frame_y = num + 18;
            t.frame_x = 0;
            t.kind = kind;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A small sky world: open air above a solid stone floor at `ground`.
    fn sky_world(width: i32, height: i32, ground: i32) -> World {
        let mut w = World::new("ih".into(), width, height);
        for x in 0..width {
            for y in ground..height {
                let i = w.index(x, y);
                w.tiles[i].kind = 1;
                w.tiles[i].flags |= FLAG_ACTIVE;
            }
        }
        w.world_surface = (ground + 25) as f64;
        w
    }

    fn count_kind(world: &World, kind: u16) -> usize {
        world
            .tiles
            .iter()
            .filter(|t| t.active() && t.kind == kind)
            .count()
    }

    /// W4-d104 pass 28: the pass builds the sunplate house at the vanilla
    /// slot — the shell (202), the interior wall (82), the chest (21) and
    /// the furniture (14/15) all land; two same-stream runs are byte-equal
    /// (the a == b determinism witness).
    #[test]
    fn w4_d104_28_the_house_builds_and_replays_byte_equal() {
        let slot = IslandHouseSlot { x: 150, y: 100, style: 0, sky_lake: false };
        let mut a = sky_world(300, 200, 120);
        let mut b = sky_world(300, 200, 120);
        let mut count_a = 0u32;
        let mut count_b = 0u32;
        pass_floating_island_houses(
            &mut a, &[slot], &mut Rng(42), None,
            false, false, false, false, false, false, &mut count_a,
        );
        pass_floating_island_houses(
            &mut b, &[slot], &mut Rng(42), None,
            false, false, false, false, false, false, &mut count_b,
        );
        assert_eq!(a.tiles, b.tiles, "a == b: the pass stream replays");
        assert_eq!(count_a, 1, "skyIslandHouseCount advances once (:79394)");
        assert!(count_kind(&a, TILE_SUNPLATE) > 0, "the sunplate shell");
        assert!(a.tiles.iter().any(|t| t.wall == WALL_SUNPLATE), "the interior wall");
        assert!(count_kind(&a, 21) > 0, "the sky chest");
        assert!(count_kind(&a, 14) > 0 && count_kind(&a, 15) > 0, "table + chairs");
    }

    /// W4-d104 pass 28: the skyLake slots build nothing (`:17868`'s
    /// `if (!GenVars.skyLake[i])` gate) and the wall veto refuses the
    /// house (`:79205-79216`).
    #[test]
    fn w4_d104_28_the_lake_slot_and_the_wall_veto_skip() {
        let mut w = sky_world(300, 200, 120);
        let mut count = 0u32;
        let lake = IslandHouseSlot { x: 150, y: 100, style: 0, sky_lake: true };
        pass_floating_island_houses(
            &mut w, &[lake], &mut Rng(42), None,
            false, false, false, false, false, false, &mut count,
        );
        assert_eq!(count_kind(&w, TILE_SUNPLATE), 0, "the lake slot builds nothing");
        assert_eq!(count, 0, "the counter stays (the gate precedes the house)");
        // A dungeon wall inside the ±30 box refuses the house.
        let i = w.index(150, 100);
        w.tiles[i].wall = 7;
        let house = IslandHouseSlot { x: 150, y: 100, style: 0, sky_lake: false };
        pass_floating_island_houses(
            &mut w, &[house], &mut Rng(42), None,
            false, false, false, false, false, false, &mut count,
        );
        assert_eq!(count_kind(&w, TILE_SUNPLATE), 0, "the wall veto refuses");
    }
}
