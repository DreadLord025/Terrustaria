//! The `Statues` pass (`WorldGen.cs:16853-16970`) + `SetupStatueList`
//! (`:4449-4488`) + `PlaceStatueTrap` (`:4490-4513`) + the `Place2xX`
//! (`:39663-39722`) / `Place1x1` (`:45188-45235`) slices the pass rides:
//! the statue cycle over the cave band, the wired pressure-plate traps for
//! the four trap statues.
//!
//! Deviations: the `Main.starGame` count multiplier (`:16869-16872`) is an
//! unported mode; the `dualDungeons` arm of the ocean re-draw loop
//! (`:16899`, `DungeonUtils.InAnyPotentialDungeonBounds`) is skipped — the
//! port has no potential-dungeon-bounds ledger (the dungeon is fixed
//! Right); the skyblock count halving / budget-10 arms (`:16874-16878`)
//! are moot — the step adapter gates plain skyblock out
//! (`denySomeGeneration`, `:16855`); the `PlaceTile`/`Place2xX`
//! `SquareTileFrame` calls ride the deferred post-pipeline framing seam
//! (105/349/135 are frame-important boot statics, `Main.cs:7707`/`:7727`/
//! `:7721` — the framer keeps the stored frames).

use super::noise::Rng;
use crate::tile::{FLAG_ACTIVE, FLAG_INACTIVE, LiquidKind};
use crate::tiledata::TILE_SOLID;
use crate::world::World;

/// `WorldGen.beachDistance` (`WorldGen.cs`, the 380 the ocean passes read —
/// the `evil.rs::BEACH_DISTANCE` slice).
const BEACH_DISTANCE: i32 = 380;

/// `GenVars.StatuesWithTraps` (`GenVars.cs:474`: {4, 7, 10, 18}).
const STATUES_WITH_TRAPS: [usize; 4] = [4, 7, 10, 18];

/// `WorldGen.SetupStatueList` (`WorldGen.cs:4449-4488`): the 44 base
/// (105, style) rows with the two overrides (34 → the jug 349 style 0,
/// 43 → style 50), then the 29 appended styles — 73 entries, verbatim
/// order (the pass cycles it in order).
fn statue_list() -> Vec<(u16, i32)> {
    let mut list: Vec<(u16, i32)> = (0..44).map(|i| (105u16, i)).collect();
    list[34] = (349, 0);
    list[43] = (105, 50);
    for style in [
        63, 64, 65, 66, 68, 69, 70, 71, 72, 73, 75, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61,
        62, 77, 78, 67, 74, 37, 2,
    ] {
        list.push((105, style));
    }
    list
}

/// `WorldGen.SolidTile2(int, int)` (`WorldGen.cs:70492-70509`): active,
/// `tileSolid`, slope 0, no half-brick, not inactive; out-of-bounds reads
/// as solid. (The platform half-brick/top-slope alternative is moot — the
/// half-brick veto below it always fires; the `mini_biomes.rs::solid_tile2`
/// slice.)
fn solid_tile2(world: &World, x: i32, y: i32) -> bool {
    let Some(t) = world.tile(x, y) else {
        return true;
    };
    t.active()
        && TILE_SOLID.has(t.kind)
        && t.slope() == 0
        && !t.half_brick()
        && (t.flags & FLAG_INACTIVE) == 0
}

/// `WorldGen.oceanDepths` (`WorldGen.cs:10963-10973`) — the `evil.rs`
/// slice: above the ocean level and inside the ±beach margins.
fn ocean_depths(world: &World, x: i32, y: i32, ocean_level: f64) -> bool {
    if y as f64 > ocean_level {
        return false;
    }
    x < BEACH_DISTANCE || x > world.width - BEACH_DISTANCE
}

/// `WorldGen.Place2xX` (`WorldGen.cs:39663-39722`) for the statue kinds
/// 105/349: the style-34 → 349 swap, the style/55 sheet offset, three body
/// rows empty over both columns, the `SolidTile2` floor under both, then
/// the 3×2 frame writes (`frameX = style*36 + col*18`, `frameY = row*18`).
fn place_2xx_statue(world: &mut World, x: i32, y: i32, mut kind: u16, mut style: i32) {
    let mut num = style * 36;
    let mut num2 = 0;
    let num3 = 3;
    // `:39668-39673` — the jug swap.
    if kind == 105 && style == 34 {
        kind = 349;
        style = 0;
        num = 0;
    }
    // `:39674-39679` — the second sprite sheet (styles 55+).
    if kind == 105 {
        let num4 = style / 55;
        num -= 1980 * num4;
        num2 += 54 * num4;
    }
    let _ = style;
    // `:39688-39707` — the 3×2 clearance.
    let mut flag = true;
    for row in (y - num3 + 1)..(y + 1) {
        for col in x..(x + 2) {
            if world.tile(col, row).is_some_and(|t| t.active()) {
                flag = false;
            }
        }
    }
    // `:39708-39720` — the floor gate and the writes.
    if flag && solid_tile2(world, x, y + 1) && solid_tile2(world, x + 1, y + 1) {
        for j in 0..num3 {
            for (col, fx) in [(x, num), (x + 1, num + 18)] {
                if let Some(t) = world.tile_mut(col, y - num3 + 1 + j) {
                    t.flags |= FLAG_ACTIVE;
                    t.frame_y = (num2 + j * 18) as i16;
                    t.frame_x = fx as i16;
                    t.kind = kind;
                }
            }
        }
    }
}

/// `WorldGen.Place1x1` (`WorldGen.cs:45188-45235`) for the pressure plate
/// 135 (the `PlaceStatueTrap` consumer): the `InWorld(x, y, 2)` guard, the
/// `SolidTile2` floor + empty-cell gate, the default frame arm
/// (`frameY = style*18`, frameX untouched). The mechanism/boulder veto
/// (`:45214`) is moot for 135 over a solid floor.
fn place_1x1_plate(world: &mut World, x: i32, y: i32) {
    if x < 2 || y < 2 || x >= world.width - 2 || y >= world.height - 2 {
        return;
    }
    if solid_tile2(world, x, y + 1) && !world.tile(x, y).is_some_and(|t| t.active()) {
        if let Some(t) = world.tile_mut(x, y) {
            t.flags |= FLAG_ACTIVE;
            t.kind = 135;
            t.frame_y = 0; // style 0, the default arm (`:45230-45232`)
        }
    }
}

/// `WorldUtils.WireLine(start, end)` (`WorldUtils.cs:109-129`) — the
/// `mini_biomes.rs::wire_line` slice: the endpoint captures happen BEFORE
/// the swaps (the horizontal run rides the ORIGINAL start.Y, the vertical
/// run the ORIGINAL end.X).
fn wire_line(world: &mut World, start: (i32, i32), end: (i32, i32)) {
    let point = start;
    let point2 = end;
    let (mut sx, mut sy) = start;
    let (mut ex, mut ey) = end;
    if ex < sx {
        std::mem::swap(&mut ex, &mut sx);
    }
    if ey < sy {
        std::mem::swap(&mut ey, &mut sy);
    }
    for i in sx..=ex {
        world.wires.set(i, point.1, crate::wiring::WireColor::Red, true);
    }
    for j in sy..=ey {
        world.wires.set(point2.0, j, crate::wiring::WireColor::Red, true);
    }
}

/// `WorldGen.PlaceStatueTrap` (`WorldGen.cs:4490-4513`): the 21×21 scan
/// around the statue for an air cell over a `SolidTile2` floor (not the
/// bubble 162), the plate place, and the red wire run back to the statue
/// on success. The `actuallyNoTrapsForRealIMeanIt` veto and the
/// `InWorld(x, y, 20)` guard ride the head.
fn place_statue_trap(world: &mut World, x: i32, y: i32, actually_no_traps: bool) {
    if actually_no_traps
        || x < 20
        || y < 20
        || x >= world.width - 20
        || y >= world.height - 20
    {
        return;
    }
    for i in -10..=10 {
        for j in -10..=10 {
            let below_solid = world
                .tile(x + i, y + j + 1)
                .is_some_and(|t| solid_tile2(world, x + i, y + j + 1) && t.kind != 162);
            let here_air = !world.tile(x + i, y + j).is_some_and(|t| t.active());
            if here_air && below_solid {
                // `:4504` — `PlaceTile(x+i, y+j, 135, mute)` (not forced;
                // 135 is non-solid so the PlaceTile gate passes).
                place_1x1_plate(world, x + i, y + j);
                if world
                    .tile(x + i, y + j)
                    .is_some_and(|t| t.active() && t.kind == 135)
                {
                    wire_line(world, (x, y), (x + i, y + j));
                    return;
                }
            }
        }
    }
}

/// The `Statues` pass (`WorldGen.cs:16853-16970`). The skyblock gate lives
/// in the step adapter. Sub-vanilla size guard (the mining-explosives
/// convention): the x band needs > 40 wide, the y band needs height − 300
/// above its floor, and a remix world's discarded redraw band needs
/// height − 400 above the surface — the C# `Next` throws otherwise.
pub(crate) fn pass_statues(
    world: &mut World,
    world_surface: f64,
    rock_layer: f64,
    remix: bool,
    no_traps: bool,
    anniversary_or_bees: bool,
    actually_no_traps: bool,
    rng: &mut Rng,
) {
    let band_lo = ((world_surface * 2.0 + rock_layer) as i32) / 3;
    if world.width <= 40
        || world.height - 300 <= band_lo
        || (remix && world.height - 400 <= world_surface as i32)
    {
        return;
    }
    let list = statue_list();
    // `:16858-16868` — the count: `statueList.Length * 2 * width/4200`,
    // the no-traps ×15 (÷5 under anniversary/bees).
    let mut num = 0usize;
    let num2 = world.width as f64 / 4200.0;
    let mut num3 = ((list.len() * 2) as f64 * num2) as i32;
    if no_traps {
        num3 *= 15;
        if anniversary_or_bees {
            num3 /= 5;
        }
    }
    let num4 = 10000;
    let ocean_level = (world_surface + rock_layer) / 2.0 + 40.0;
    for _ in 0..num3 {
        // `:16881-16886` — the cycle and the current statue's kind/style.
        if num >= list.len() {
            num = 0;
        }
        let (x_kind, y_style) = list[num];
        let mut flag = false;
        let mut num5 = 0;
        while !flag {
            // `:16892-16897` — the point pair; the remix redraw is
            // CONSUMED AND DISCARDED (the C# quirk, verbatim).
            let mut num6 = rng.range(20, world.width - 20);
            let mut num7 = rng.range(band_lo, world.height - 300);
            if remix {
                let _ = rng.range(world_surface as i32, world.height - 400);
            }
            // `:16898-16912` — the ocean re-draw loop (budget 100; the
            // dualDungeons arm is unported — the module note).
            let mut num8 = 100;
            while ocean_depths(world, num6, num7, ocean_level) {
                num8 -= 1;
                if num8 <= 0 {
                    break;
                }
                num6 = rng.range(20, world.width - 20);
                num7 = rng.range(band_lo, world.height - 300);
                if remix {
                    let _ = rng.range(world_surface as i32, world.height - 400);
                }
            }
            if num8 <= 0 {
                flag = true;
                continue;
            }
            // `:16919-16926` — the down-scan to the first active cell.
            while !world.tile(num6, num7).is_some_and(|t| t.active()) {
                num7 += 1;
                if num7 >= world.height {
                    break;
                }
            }
            if num7 >= world.height {
                flag = true;
                continue;
            }
            num7 -= 1;
            // `:16934-16947` — the pre-place vetoes: a frame-important
            // non-cut tile or shimmer in the cell refuses the place.
            let cell = world.tile(num6, num7);
            let mut flag2 = true;
            if let Some(t) = cell {
                if t.active()
                    && world.frame_important.get(t.kind as usize).copied().unwrap_or(false)
                    && !crate::tileobject::tile_cut(t.kind)
                {
                    flag2 = false;
                }
                if t.liquid > 0 && t.liquid_kind() == LiquidKind::Shimmer {
                    flag2 = false;
                }
            }
            if flag2 {
                // `:16946` — `PlaceTile(num6, num7, x, mute, forced, -1, y)`:
                // the 488 statue-peace veto (`:59345-59348`), then the
                // Place2xX arm (`:60065-60078`).
                let peace = world
                    .tile(num6, num7)
                    .is_some_and(|t| t.active() && t.kind == 488);
                if !peace {
                    place_2xx_statue(world, num6, num7, x_kind, y_style);
                }
            }
            // `:16948-16964` — the post-place read: success advances the
            // cycle (and wires the trap statues), failure burns the budget.
            let placed = world
                .tile(num6, num7)
                .is_some_and(|t| t.active() && t.kind == x_kind);
            if placed {
                flag = true;
                if STATUES_WITH_TRAPS.contains(&num) {
                    place_statue_trap(world, num6, num7, actually_no_traps);
                }
                num += 1;
            } else {
                num5 += 1;
                if num5 >= num4 {
                    flag = true;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A stone world with a 6-wide × 5-row air pocket over the floor row
    /// 64 — a statue at x=50, y=63 clears its 3×2 body (rows 61-63) and
    /// stands on the solid row 64.
    fn pocket_world() -> World {
        let mut world = World::new("st".into(), 200, 500);
        for y in 0..500 {
            for x in 0..200 {
                let i = world.index(x, y);
                world.tiles[i].kind = 1;
                world.tiles[i].flags |= FLAG_ACTIVE;
            }
        }
        for y in 59..64 {
            for x in 47..54 {
                let i = world.index(x, y);
                world.tiles[i].flags &= !FLAG_ACTIVE;
            }
        }
        world.frame_important = vec![false; crate::tiledata::TILE_COUNT];
        world.frame_important[105] = true;
        world.frame_important[349] = true;
        world.frame_important[135] = true;
        world
    }

    #[test]
    fn statue_list_matches_the_csharp_setup() {
        let list = statue_list();
        assert_eq!(list.len(), 73);
        assert_eq!(list[0], (105, 0));
        assert_eq!(list[34], (349, 0));
        assert_eq!(list[43], (105, 50));
        assert_eq!(list[44], (105, 63));
        assert_eq!(list[72], (105, 2));
    }

    #[test]
    fn place_2xx_writes_the_3x2_frames() {
        let mut world = pocket_world();
        place_2xx_statue(&mut world, 50, 63, 105, 3);
        for row in 0..3 {
            for col in 0..2 {
                let t = world.tile(50 + col, 61 + row).unwrap();
                assert!(t.active() && t.kind == 105);
                assert_eq!(t.frame_x, (3 * 36 + col * 18) as i16);
                assert_eq!(t.frame_y, (row * 18) as i16);
            }
        }
    }

    #[test]
    fn place_2xx_style_34_swaps_to_the_jug() {
        let mut world = pocket_world();
        place_2xx_statue(&mut world, 50, 63, 105, 34);
        let t = world.tile(50, 63).unwrap();
        assert_eq!(t.kind, 349);
        assert_eq!((t.frame_x, t.frame_y), (0, 36));
    }

    #[test]
    fn place_2xx_style_55_shifts_the_sheet() {
        let mut world = pocket_world();
        place_2xx_statue(&mut world, 50, 63, 105, 55);
        let t = world.tile(50, 61).unwrap();
        // num = 55*36 − 1980 = 0, num2 = 54.
        assert_eq!((t.frame_x, t.frame_y), (0, 54));
    }

    #[test]
    fn statue_trap_places_the_plate_and_the_wire() {
        let mut world = pocket_world();
        place_2xx_statue(&mut world, 50, 63, 105, 4);
        place_statue_trap(&mut world, 50, 63, false);
        // The scan finds the pocket floor beside the statue — a plate 135
        // exists within the ±10 box and a red wire connects it.
        let plate = (40..=60)
            .flat_map(|x| (53..=73).map(move |y| (x, y)))
            .find(|&(x, y)| {
                world
                    .tile(x, y)
                    .is_some_and(|t| t.active() && t.kind == 135)
            });
        assert!(plate.is_some(), "the plate places");
        let (px, py) = plate.unwrap();
        assert!(
            world.wires.get(px, py, crate::wiring::WireColor::Red),
            "the wire reaches the plate"
        );
    }

    #[test]
    fn statues_pass_is_deterministic_for_a_seed() {
        let base = pocket_world();
        let mut a = base.clone();
        let mut b = base.clone();
        pass_statues(&mut a, 20.0, 100.0, false, false, false, false, &mut Rng(42));
        pass_statues(&mut b, 20.0, 100.0, false, false, false, false, &mut Rng(42));
        assert_eq!(a.tiles, b.tiles, "a == b");
        // WireState has no PartialEq — compare the red-wire cells directly.
        for y in 0..a.height {
            for x in 0..a.width {
                assert_eq!(
                    a.wires.get(x, y, crate::wiring::WireColor::Red),
                    b.wires.get(x, y, crate::wiring::WireColor::Red),
                    "a == b (wires at {x},{y})"
                );
            }
        }
    }

    #[test]
    fn statues_size_guard_draws_nothing() {
        let mut world = World::new("tiny".into(), 30, 100);
        let mut rng = Rng(0x5EED);
        pass_statues(&mut world, 20.0, 50.0, false, false, false, false, &mut rng);
        assert_eq!(rng.0, 0x5EED, "the size guard draws nothing");
    }
}
