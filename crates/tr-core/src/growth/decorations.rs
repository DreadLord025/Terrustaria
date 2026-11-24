//! The decorations family of the world-growth ticks (S1-K-3 split out of
//! `growth.rs`; behaviour frozen - verbatim move). The cattail passes
//! (`WorldGen.CheckCatTail` `:58744-58930`, `GrowCatTail` `:58974-59082`),
//! `GrowCheckSeaweed` (`:58917-58973`), the dungeon `GrowWeb` (`:74088-74122`),
//! the dye plants (`PlantDye` `:71286-71436`, `PlaceDye` `:53988-54065`) and
//! Abigail's flower (`TryGrowingAbigailsFlower` `:73049-73073` + its ground
//! and distance rules `:73026-73046`/`:73076-73095`). The public seams stay
//! re-exported from `growth`.

use crate::tile::FLAG_ACTIVE;
use crate::UnifiedRandom;
use crate::world::World;

/// `WorldGen.catTailDistance` (`WorldGen.cs:4401`, seeded to 8 at `:86957`):
/// the deepest a cattail stalk may stand above the water surface.
const CAT_TAIL_DISTANCE: i32 = 8;

/// `Main.tile[x, y]` read as the C# `null` check plus the accessor: `None`
/// stands for the out-of-world cell the source returns early on.
#[inline]
fn cell(world: &World, x: i32, y: i32) -> Option<crate::tile::Tile> {
    world.tile(x, y).copied()
}

/// `Tile.nactive()` (`Tile.cs:240-247`): the active bit set with the
/// actuated-off bit clear.
#[inline]
fn nactive(t: &crate::tile::Tile) -> bool {
    t.active() && !t.inactive()
}

/// `WorldGen.CheckCatTail` (`WorldGen.cs:58744-58930`): the cattail (519)
/// validity sweep. It walks the stalk's column — down to the ground, back up
/// through the water — and marks the run invalid when the water column is too
/// shallow (`liquid < 127`, rolled at `Next(4)`), the stalk stands more than
/// [`CAT_TAIL_DISTANCE`] above the water, the ground type has no frame row, or
/// the tip frames disagree; an invalid run is killed whole (a 4-tile look-back
/// above the stalk start through the ground), a valid one gets its soil frame
/// row in `frameY`.
pub fn check_cat_tail(world: &mut World, x: i32, j: i32, rng: &mut UnifiedRandom) {
    if cell(world, x, j).is_none() {
        return;
    }
    let max_y = world.height as i32 - 50;
    let mut flag = false;
    let mut num2 = j;
    // `:58751-58768`: the ground scan — down to the first cell that is active,
    // `tileSolid` and not `tileSolidTop`.
    while num2 < max_y
        && !cell(world, x, num2).is_some_and(|t| {
            t.active()
                && crate::tiledata::TILE_SOLID.has(t.kind)
                && !crate::tiledata::TILE_SOLID_TOP.has(t.kind)
        })
    {
        let Some(t) = cell(world, x, num2) else { return };
        if t.active() && t.kind != 519 {
            flag = true;
        }
        if !t.active() {
            break;
        }
        num2 += 1;
        if cell(world, x, num2).is_none() {
            return;
        }
    }
    let mut num = num2 - 1;
    if cell(world, x, num).is_none() {
        return;
    }
    // `:58769-58783`: the water column walk — anything but the cattail itself
    // or non-water liquid invalidates the run.
    while num > 50 && cell(world, x, num).is_some_and(|t| t.liquid > 0) {
        let Some(t) = cell(world, x, num) else { return };
        if (t.active() && t.kind != 519) || t.liquid_kind() != crate::tile::LiquidKind::Water {
            flag = true;
        }
        num -= 1;
        if cell(world, x, num).is_none() {
            return;
        }
    }
    num += 1;
    if cell(world, x, num).is_none() {
        return;
    }
    let num3 = num;
    if num2 - num3 > CAT_TAIL_DISTANCE {
        flag = true;
    }
    // `:58795-58824`: the soil's frame row. The pair (2, 477) shares row 0;
    // the sand 53 row 18; the corrupt/crimson/hallow grounds row 54; the
    // evil/crimson grass row 72; the mushroom grass row 90.
    let ground = cell(world, x, num2).map(|t| t.kind).unwrap_or(0);
    let num4: i32 = match ground {
        2 | 477 => 0,
        53 => 18,
        199 | 234 | 662 => 54,
        23 | 112 | 661 => 72,
        70 => 90,
        _ => -1,
    };
    if !cell(world, x, num2).is_some_and(|t| nactive(&t)) {
        flag = true;
    }
    if num4 < 0 {
        flag = true;
    }
    // `:58826-58840`: the stalk's lowest cell — the topmost 519 within
    // `num3..=num2 - 1` when the cell directly above the ground is free.
    num = num2 - 1;
    if cell(world, x, num).is_some_and(|t| !t.active()) {
        let mut n5 = num;
        while n5 >= num3 {
            let Some(t) = cell(world, x, n5) else { return };
            if t.active() && t.kind == 519 {
                num = n5;
                break;
            }
            n5 -= 1;
        }
    }
    while cell(world, x, num).is_some_and(|t| t.active() && t.kind == 519) {
        num -= 1;
    }
    num += 1;
    // `:58841-58847`: the shallow-water and the deep-tip rolls — the C#'s
    // `&&` short-circuits, so the `Next(4)` draw only happens when the liquid
    // test passes.
    if cell(world, x, num2 - 1).is_some_and(|t| t.liquid < 127) && rng.next_max(4) == 0 {
        flag = true;
    }
    if cell(world, x, num).is_some_and(|t| t.frame_x >= 180 && t.liquid > 127) && rng.next_max(4) == 0
    {
        flag = true;
    }
    // `:58848-58863`: the tip frame agreement.
    if cell(world, x, num).is_some_and(|t| t.frame_x > 18) {
        if let Some(above) = cell(world, x, num2 - 1) {
            let cur = cell(world, x, num).map(|t| t.frame_x).unwrap_or(0);
            if !(36..=72).contains(&above.frame_x) {
                flag = true;
            } else if cur < 90 {
                flag = true;
            } else if (108..=162).contains(&cur) {
                if let Some(t) = world.tile_mut(x, num) {
                    t.frame_x = 90;
                }
            }
        }
    }
    // `:58864-58872`: a dry cell four above the stalk base under a 519 three
    // above it means the run is floating out of the water.
    if num2 > num + 4
        && cell(world, x, num + 4).is_some_and(|t| t.liquid == 0)
        && cell(world, x, num + 3).is_some_and(|t| t.kind == 519)
    {
        flag = true;
    }
    if flag {
        // `:58874-58890`: the kill sweep — every 519 from four above the
        // lowest stalk cell down to the ground dies and reframes.
        let mut num6 = num3;
        if num < num3 {
            num6 = num;
        }
        for i in (num6 - 4)..=num2 {
            if cell(world, x, i).is_some_and(|t| t.active() && t.kind == 519) {
                // W4-g83: the cascade rides the caller's ACTIVE genRand
                // stream (the growth tick's UpdateWorldGen).
                crate::edit::kill_tile(world, x, i, false, rng);
                crate::framing::reframe_area(world, x, i, 1, true);
            }
        }
    } else {
        // `:58891-58928`: the frame-row writeback over the stalk run.
        if cell(world, x, num).is_some_and(|t| i32::from(t.frame_y) == num4) {
            return;
        }
        for k in num..num2 {
            if cell(world, x, k).is_some_and(|t| t.active() && t.kind == 519) {
                if let Some(t) = world.tile_mut(x, k) {
                    t.frame_y = num4 as i16;
                }
            }
        }
    }
}

/// `WorldGen.GrowCatTail` (`WorldGen.cs:58974-59082`): one growth step of a
/// water-column cattail — the bottom cell's `frameX` advances 0 → 18 →
/// `18 * Next(2, 5)` and then the tip chain (90 → 108/126..162) which spawns
/// the next 519 cell above with the matching paired frame, so the stalk gains
/// one tile per successful tip step.
pub fn grow_cat_tail(world: &mut World, x: i32, j: i32, rng: &mut UnifiedRandom) {
    let max_y = world.height as i32 - 50;
    let mut num = j;
    // `:58980-58983`: up through the liquid.
    while num > 50 && cell(world, x, num).is_some_and(|t| t.liquid > 0) {
        num -= 1;
    }
    // `:58985-58989`: down to the ground.
    let mut i = num + 1;
    while i < max_y
        && !cell(world, x, i).is_some_and(|t| {
            t.active()
                && crate::tiledata::TILE_SOLID.has(t.kind)
                && !crate::tiledata::TILE_SOLID_TOP.has(t.kind)
        })
    {
        i += 1;
    }
    num = i - 1;
    // `:58990-58994`: back down to the stalk's own bottom cell.
    while cell(world, x, num).is_some_and(|t| t.active() && t.kind == 519) {
        num -= 1;
    }
    num += 1;
    if !cell(world, x, num).is_some_and(|t| t.active() && t.kind == 519) {
        return;
    }
    // `:58996-59004`: a full tip (frameX 90) with a cuttable tile above it
    // clears that tile first.
    if cell(world, x, num).is_some_and(|t| t.frame_x == 90)
        && cell(world, x, num - 1).is_some_and(|t| t.active() && crate::tileobject::tile_cut(t.kind))
    {
        // W4-g83: the cascade rides the caller's ACTIVE genRand stream.
        crate::edit::kill_tile(world, x, num - 1, false, rng);
    }
    if cell(world, x, num - 1).is_some_and(|t| t.active()) {
        return;
    }
    let frame_x = cell(world, x, num).map(|t| t.frame_x).unwrap_or(0);
    let frame_y = cell(world, x, num).map(|t| t.frame_y).unwrap_or(0);
    let paint = cell(world, x, num).map(|t| t.paint()).unwrap_or(0);
    // The `CopyPaintAndCoating` target (`:59035`/`:59059`/`:59075`): the new
    // cell above inherits the base cell's paint.
    let place_above = |world: &mut World, frame_x_above: i16| {
        if let Some(t) = world.tile_mut(x, num - 1) {
            t.flags |= FLAG_ACTIVE;
            t.kind = 519;
            t.frame_x = frame_x_above;
            t.frame_y = frame_y;
            t.set_paint(paint);
        }
    };
    if frame_x == 0 {
        // `:59006-59012`: the first segment.
        if let Some(t) = world.tile_mut(x, num) {
            t.frame_x = 18;
        }
        crate::framing::reframe_area(world, x, num, 1, true);
    } else if frame_x == 18 {
        // `:59013-59030`: the stalk grows its paired tip cell above.
        let len = 18 * rng.next_range(2, 5) as i16;
        if let Some(t) = world.tile_mut(x, num) {
            t.frame_x = len;
        }
        place_above(world, 90);
        crate::framing::reframe_area(world, x, num, 1, true);
    } else if frame_x == 90 {
        // `:59031-59078`: the tip arms — the submerged arm takes the 108 tip,
        // the dry arm dies at `Next(3)` or takes a 126/144/162 tip with its
        // 180/198/216 partner.
        if cell(world, x, num - 1).is_some_and(|t| t.liquid == 0) {
            let wet_below = cell(world, x, num).is_some_and(|t| t.liquid > 0)
                || cell(world, x, num + 1).is_some_and(|t| t.liquid > 0)
                || cell(world, x, num + 2).is_some_and(|t| t.liquid > 0);
            if !cell(world, x, num - 2).is_some_and(|t| t.active()) && wet_below && rng.next_max(3) == 0
            {
                if let Some(t) = world.tile_mut(x, num) {
                    t.frame_x = 108;
                }
                place_above(world, 90);
            } else {
                let pick = rng.next_max(3) as i16;
                if let Some(t) = world.tile_mut(x, num) {
                    t.frame_x = 126 + pick * 18;
                }
                place_above(world, 180 + pick * 18);
            }
        } else {
            if let Some(t) = world.tile_mut(x, num) {
                t.frame_x = 108;
            }
            place_above(world, 90);
        }
    }
    // `:59080`: the final reframe of the cell pair, `resetFrame: false`.
    crate::framing::reframe_area(world, x, num - 1, 1, false);
}

/// `WorldGen.GrowCheckSeaweed` (`WorldGen.cs:58917-58973`): the seaweed (549)
/// on sand. Out of water (`liquid < 200` on the cell or the one above) a stand
/// dies at `genRand.Next(2)`; in water the arm stands down on an occupied or
/// over-dense neighbourhood (the 9×13 box's 30-cap) and otherwise places the
/// next 549 above when the water below the surface is deep enough
/// (`k - y < 17 - Next(20)`).
pub fn grow_check_seaweed(world: &mut World, x: i32, y: i32, rng: &mut UnifiedRandom) {
    let Some(here) = cell(world, x, y) else { return };
    if (here.kind == 549 && here.liquid < 200)
        || cell(world, x, y - 1).is_some_and(|t| t.liquid < 200)
    {
        // `:58925-58933`: the dry-out kill — the cascade's framing dispatch
        // (incl. the neighbouring seaweed tips' re-rolls) rides the SAME
        // active genRand stream (W4-g83 contract §2).
        if here.active() && here.kind == 549 && rng.next_max(2) == 0 {
            crate::edit::kill_tile(world, x, y, false, rng);
        }
        return;
    }
    // `:58936-58940`: the growth gates — the `Next(1)` of `maxValue` 1 still
    // draws (vanilla consumes the roll), and both `y - 2`/`y - 3` must be full
    // water.
    let blocked = cell(world, x, y - 1).is_some_and(|t| t.active())
        || cell(world, x, y - 2).is_some_and(|t| t.active());
    let roll = rng.next_max(1) != 0;
    if blocked
        || roll
        || !cell(world, x, y - 2).is_some_and(|t| t.liquid == 255)
        || !cell(world, x, y - 3).is_some_and(|t| t.liquid == 255)
    {
        return;
    }
    // `:58941-58954`: the 9×13 density box's 30-cap.
    let mut count = 0;
    for i in (x - 4)..=(x + 4) {
        for j in y..=(y + 12) {
            if cell(world, i, j).is_some_and(|t| t.active() && t.kind == 549) {
                count += 1;
                if count > 30 {
                    return;
                }
            }
        }
    }
    // `:58955-58961`: the depth to the sea floor.
    let max_y = world.height as i32 - 50;
    let mut k = y;
    while k < max_y && !crate::edit::solid_tile(world, x, k) {
        k += 1;
    }
    // `:58962-58968`: the `17 - Next(20)` window and the write.
    if k - y < 17 - rng.next_max(20) {
        if let Some(t) = world.tile_mut(x, y - 1) {
            if !t.active() {
                t.kind = 549;
                t.flags |= FLAG_ACTIVE;
            }
        }
        crate::framing::reframe_area(world, x, y - 1, 1, false);
    }
}

/// `WorldGen.GrowWeb` (`WorldGen.cs:74088-74122`): the dungeon web (51) — one
/// `Next(10)` per pick, then a `Next(2, 4)` radius whose box must touch a
/// solid; an inactive cell becomes a web and reframes. The port generates no
/// notTheBees/getGoodWorld seeds, so that box test (the missing 62 wall) is
/// dead here.
pub(super) fn grow_web(world: &mut World, i: i32, j: i32, rng: &mut UnifiedRandom) {
    if rng.next_max(10) != 0 {
        return;
    }
    let num = rng.next_range(2, 4);
    let mut flag = false;
    'outer: for k in (i - num)..=(i + num) {
        for l in (j - num)..=(j + num) {
            if crate::edit::solid_tile(world, k, l) {
                flag = true;
                break 'outer;
            }
        }
    }
    if flag && !world.tile(i, j).is_some_and(|t| t.active()) {
        if let Some(t) = world.tile_mut(i, j) {
            t.kind = 51;
            t.flags |= FLAG_ACTIVE;
        }
        crate::framing::reframe_area(world, i, j, 1, true);
    }
}

/// The kinds a dye plant may grow over or under (`WorldGen.PlantDye` and
/// `PlaceDye`'s free-side list, `WorldGen.cs:71300-71301`/`:53997-53998`): the
/// grass 3, the web 51, the flower vines 697/703, the lily 61, the jungle
/// flower 73, the flower 74 and the mushroom 184.
const DYE_FREE_SIDES: [u16; 8] = [3, 51, 697, 703, 61, 73, 74, 184];

/// `WorldGen.PlaceDye` (`WorldGen.cs:53988-54065`): the dye plant (227) write
/// with its `frameX = 34 * style` layout. The seat rules are per style: 7
/// hangs below a solid with a free cell below; 6 sits on the mushroom grass
/// 80 between two free cells; 5 needs a full water cell over the solid; 8-11
/// take any dry solid floor; 3/4 need an open (wall == 0) cell; the rest any
/// dry solid floor. Returns whether the write happened.
pub fn place_dye(world: &mut World, x: i32, y: i32, style: i32) -> bool {
    let Some(here) = cell(world, x, y) else { return false };
    let Some(below) = cell(world, x, y + 1) else { return false };
    let Some(above) = cell(world, x, y - 1) else { return false };
    let free = |t: &crate::tile::Tile| !t.active() || DYE_FREE_SIDES.contains(&t.kind);
    let mut flag = false;
    if style == 7 {
        if !free(&below) {
            return false;
        }
        if crate::edit::solid_tile(world, x, y - 1) && !below.active() {
            flag = true;
        }
    } else {
        if !free(&above) {
            return false;
        }
        if style == 6 {
            if below.active()
                && !below.inactive()
                && below.kind == 80
                && !cell(world, x - 1, y + 1).is_some_and(|t| t.active())
                && !cell(world, x + 1, y + 1).is_some_and(|t| t.active())
            {
                flag = true;
            }
        } else if crate::edit::solid_tile(world, x, y + 1) && !above.active() {
            match style {
                // `:54038-54044`: the water style needs full, non-lava/honey/
                // shimmer liquid in the plant's own cell.
                5 => {
                    flag = here.liquid == 255
                        && here.liquid_kind() == crate::tile::LiquidKind::Water;
                }
                // `:54045-54047`: the exotic styles take any dry floor.
                8 | 9 | 10 | 11 => flag = true,
                _ => {
                    if here.liquid != 0 {
                        // `:54050`: the liquid break.
                    } else if style == 3 || style == 4 {
                        if here.wall == 0 {
                            flag = true;
                        }
                    } else {
                        flag = true;
                    }
                }
            }
        }
    }
    if !flag {
        return false;
    }
    // `:54056-54064`: the write — active, no half brick/slope, `frameY` 0 and
    // the style column `34 * style`.
    if let Some(t) = world.tile_mut(x, y) {
        t.kind = 227;
        t.flags |= FLAG_ACTIVE;
        t.flags &= !crate::tile::FLAG_HALF_BRICK;
        t.set_slope(0);
        t.frame_y = 0;
        t.frame_x = (34 * style) as i16;
    }
    true
}

/// `WorldGen.TryGrowingAbigailsFlower` (`WorldGen.cs:73049-73073`): the
/// tombstone arm of `UpdateWorld` (`:72063-72067`). One `Main.rand.Next(30)`
/// gates it, then one or two spots are drawn in the 20x20 box around the grave
/// (`Math.Max(10, i - 10) .. Math.Min(maxTilesX - 10, i + 10)`, the same shape
/// on `j`) and the first one with valid Abigail ground, no flower within
/// `120` tiles and a successful `PlaceTile(624)` takes the flower. A Don't
/// Starve world rolls twice (`Main.dontStarveWorld`).
///
/// Deviation: the C# mixes `Main.rand` (the gate) with `WorldGen.genRand` (the
/// two spot draws); the port's single `rng` stream feeds both, like every other
/// growth arm (ANALYSIS, M5-SPREAD-1).
pub(super) fn try_growing_abigails_flower(world: &mut World, i: i32, j: i32, rng: &mut UnifiedRandom) -> bool {
    if rng.next_max(30) != 0 {
        return false;
    }
    let rolls = if world.seeds.contains(crate::seeds::SeedFlags::DONT_STARVE) {
        2
    } else {
        1
    };
    let max_x = world.width - 10;
    let max_y = world.height - 10;
    for _ in 0..rolls {
        let lo_x = (i - 10).max(10);
        let hi_x = (i + 10).min(max_x);
        let lo_y = (j - 10).max(10);
        let hi_y = (j + 10).min(max_y);
        // `genRand.Next(min, max)` needs `max > min`; a grave in the world's
        // clamp corner can collapse the box, and the C# would throw there.
        if hi_x <= lo_x || hi_y <= lo_y {
            return false;
        }
        let nx = rng.next_range(lo_x, hi_x);
        let ny = rng.next_range(lo_y, hi_y);
        if has_valid_ground_for_abigails_flower_below_spot(world, nx, ny)
            && no_nearby_abigails_flower(world, nx, ny)
            && crate::edit::place_tile(world, nx, ny, 624)
        {
            // `PlaceTile` clears the cell first, so the fresh flower carries
            // frame 0/0 (`WorldGen.cs:59488-59493`'s write for the 624/700
            // placement arm); nothing else frames a 1x1 flower.
            if let Some(t) = world.tile_mut(nx, ny) {
                t.frame_x = 0;
                t.frame_y = 0;
            }
            return true;
        }
    }
    false
}

/// `WorldGen.HasValidGroundForAbigailsFlowerBelowSpot`
/// (`WorldGen.cs:73026-73046`): the spot is in-world with a 2-tile margin, the
/// tile below is active, its type is the mushroom grass 70, the ash grass 633
/// or a `Conversion.Grass` member, and that tile is solid allowing a bottom
/// slope.
fn has_valid_ground_for_abigails_flower_below_spot(world: &World, x: i32, y: i32) -> bool {
    // `WorldGen.InWorld(x, y, 2)` (`WorldGen.cs:3106-3115`).
    if x < 2 || x > world.width - 3 || y < 2 || y > world.height - 3 {
        return false;
    }
    let Some(below) = world.tile(x, y + 1) else {
        return false;
    };
    if !below.active() {
        return false;
    }
    if !(below.kind == 70 || below.kind == 633 || crate::tiledata::SET_GRASS.has(below.kind)) {
        return false;
    }
    crate::edit::solid_tile_allow_bottom_slope(world, x, y + 1)
}

/// `WorldGen.NoNearbyAbigailsFlower` (`WorldGen.cs:73076-73095`): no active
/// flower 624 in the `120`-tile box, clamped to the world's `10`-tile margin.
fn no_nearby_abigails_flower(world: &World, i: i32, j: i32) -> bool {
    let lo_x = (i - 120).clamp(10, world.width - 11);
    let hi_x = (i + 120).clamp(10, world.width - 11);
    let lo_y = (j - 120).clamp(10, world.height - 11);
    let hi_y = (j + 120).clamp(10, world.height - 11);
    for x in lo_x..=hi_x {
        for y in lo_y..=hi_y {
            if world.tile(x, y).is_some_and(|t| t.active() && t.kind == 624) {
                return false;
            }
        }
    }
    true
}

/// `WorldGen.plantDye` (`WorldGen.cs:71286-71436`): the dye-plant (227) sprout
/// on an `nactive` cell inside the 95-tile margins. Three passes — the surface
/// band (`j < worldSurface`), the underground band
/// (`worldSurface <= j < UnderworldLayer`) and the hanging style 7 below an
/// empty dirt cell. Each pass first refuses when the ±`num` box already holds
/// a 227 of its own family (the `frameX < 272` normal / `>= 272` exotic split)
/// and then dispatches on the soil kind to a style (2 dirt/109, 60 jungle,
/// 53 sand in full water, 80 mushroom grass, 0/1/59 dirt/stone/clay, the
/// full-water 5, the exotic `Next(8, 12)`).
pub fn plant_dye(world: &mut World, i: i32, j: i32, exotic: bool, rng: &mut UnifiedRandom) {
    let Some(here) = cell(world, i, j) else { return };
    if !here.active() || i < 95 || i > world.width - 95 || j < 95 || j > world.height - 95 {
        return;
    }
    let num = if exotic { 240 } else { 90 };
    let water_above_full = cell(world, i, j - 1).is_some_and(|t| t.liquid == 255)
        && cell(world, i, j - 2).is_some_and(|t| t.liquid == 255);
    // `:71300-71301`: the free-above gate shared by both bands.
    let free_above = |world: &World| {
        cell(world, i, j - 1).is_some_and(|t| {
            !t.active() || matches!(t.kind, 3 | 51 | 61 | 73 | 74 | 184 | 697 | 703)
        })
    };
    // The band 1 arm (`:71298-71377`): `j < worldSurface || remixWorldGen`
    // — the remix bypass lets the surface arm run at any depth (W4-d105
    // doc-sweep: the remix half was missing; the gen-time twin
    // `worldgen::surface::plant_dye` carries it).
    let remix = world.seeds.contains(crate::seeds::SeedFlags::REMIX);
    if ((j as f64) < world.world_surface || remix) && free_above(world) {
        if dye_density_blocked(world, i, j, num, exotic) {
            return;
        }
        if exotic {
            // `:71324-71336`: the grass/moss/dirt arm behind the lava/honey
            // veto.
            let mut ok = crate::tiledata::SET_GRASS.has(here.kind)
                || crate::tiledata::SET_CONVERSION_MOSS.has(here.kind)
                || here.kind == 0;
            if let Some(t) = cell(world, i, j - 1) {
                // `:71318-71321`: the veto is `liquid > 0 && (lava() ||
                // honey())` — water and shimmer pass (W4-d105 doc-sweep:
                // the prior `!= Water` also vetoed shimmer).
                if t.liquid > 0
                    && matches!(
                        t.liquid_kind(),
                        crate::tile::LiquidKind::Lava | crate::tile::LiquidKind::Honey
                    )
                {
                    ok = false;
                }
            }
            if ok {
                let style = rng.next_range(8, 12);
                place_dye(world, i, j - 1, style);
            }
        } else if here.kind == 2 || here.kind == 109 {
            // `:71339-71347`: the dirt/grass pair rolls 3 or 4.
            let style = if rng.next_max(4) == 0 { 4 } else { 3 };
            place_dye(world, i, j - 1, style);
        } else if here.kind == 60 {
            // `:71348-71356`: the jungle grass rolls 2 or 4.
            let style = if rng.next_max(2) == 0 { 2 } else { 4 };
            place_dye(world, i, j - 1, style);
        } else if here.kind == 53 && water_above_full {
            // `:71357-71360`: the sand arm's water style.
            place_dye(world, i, j - 1, 2);
        } else if here.kind == 80
            && !cell(world, i - 1, j - 1).is_some_and(|t| t.active())
            && !cell(world, i + 1, j - 1).is_some_and(|t| t.active())
        {
            // `:71361-71366`: the mushroom grass arm and its 11×21 evil box.
            let mut ok = true;
            for m in (i - 5)..=(i + 5) {
                for n in (j - 5)..=(j + 15) {
                    if cell(world, m, n).is_some_and(|t| t.active() && matches!(t.kind, 112 | 234)) {
                        ok = false;
                    }
                }
            }
            if ok {
                place_dye(world, i, j - 1, 6);
            }
        }
    }
    // `:71378-71381`: the underground band's gate —
    // `if (!(j >= worldSurface) && !remixWorldGen || j >= UnderworldLayer)
    // return;` i.e. the band runs only at/below the surface (or on remix)
    // and above the underworld layer. (W4-d105 doc-sweep: the remix half
    // was missing — the gen-time twin carries it.)
    if ((j as f64) < world.world_surface && !remix) || j >= world.height - 200 {
        return;
    }
    if free_above(world) {
        if dye_density_blocked(world, i, j, num, exotic) {
            return;
        }
        if exotic {
            if crate::tiledata::SET_GRASS.has(here.kind)
                || crate::tiledata::SET_CONVERSION_MOSS.has(here.kind)
                || here.kind == 0
            {
                let style = rng.next_range(8, 12);
                place_dye(world, i, j - 1, style);
            }
        } else if here.kind == 60 {
            // `:71394-71405`: the jungle arm's 2 / 0 / 1 cascade.
            if rng.next_max(2) == 0 {
                place_dye(world, i, j - 1, 2);
            } else if rng.next_max(2) == 0 {
                place_dye(world, i, j - 1, 0);
            } else {
                place_dye(world, i, j - 1, 1);
            }
        } else if matches!(here.kind, 0 | 1 | 59) {
            // `:71406-71414`: the dirt/stone/clay arm rolls 0 or 1.
            let style = if rng.next_max(2) == 0 { 0 } else { 1 };
            place_dye(world, i, j - 1, style);
        } else if water_above_full {
            // `:71415-71418`: the full-water style.
            place_dye(world, i, j - 1, 5);
        }
    }
    // `:71420-71436`: the hanging pass — only below an empty cell, refused by
    // any 227 in the ±num box, writing style 7 under a kind-0 cell.
    if cell(world, i, j + 1).is_some_and(|t| t.active()) || exotic {
        return;
    }
    for m in (i - num)..(i + num) {
        for n in (j - num)..(j + num) {
            if cell(world, m, n).is_some_and(|t| t.active() && t.kind == 227) {
                return;
            }
        }
    }
    if here.kind == 0 {
        place_dye(world, i, j + 1, 7);
    }
}

/// The `plantDye` density scan (`WorldGen.cs:71310-71322`): any 227 in the
/// `±num` box counts — the exotic pass matches the `frameX >= 272` styles and
/// the normal pass the `frameX < 272` ones.
fn dye_density_blocked(world: &World, i: i32, j: i32, num: i32, exotic: bool) -> bool {
    let min_i = (i - num).clamp(1, world.width - 2);
    let max_i = (i + num).clamp(1, world.width - 2);
    let min_j = (j - num).clamp(1, world.height - 2);
    let max_j = (j + num).clamp(1, world.height - 2);
    for k in min_i..max_i {
        for l in min_j..max_j {
            if cell(world, k, l).is_some_and(|t| {
                t.active()
                    && t.kind == 227
                    && (!exotic || t.frame_x >= 272)
                    && (exotic || t.frame_x < 272)
            }) {
                return true;
            }
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::growth::test_fixtures::{put, world};
    use crate::growth::{update_world_overground_tile, update_world_underground_tile};
    use crate::tile::Tile;

    /// Sets a cell's wall (the wall-arm tests' natural/evil walls).
    fn set_wall(w: &mut World, x: i32, y: i32, wall: u16) {
        let i = w.index(x, y);
        w.tiles[i].wall = wall;
    }

    #[test]
    fn the_cat_tail_frame_row_follows_the_soil() {
        // `CheckCatTail`'s soil table (`:58797-58819`): the frame row is 0 on
        // dirt/grass (2/477), 18 on sand (53), 54 on the corrupt/crimson/hallow
        // grounds (199/234/662), 72 on the evil grass (23/112) and 90 on the
        // mushroom grass (70). A valid run writes the row into every stalk
        // cell's `frameY` (`:58891-58928`).
        for (soil, row) in [(2u16, 0i16), (53, 18), (199, 54), (234, 54), (23, 72), (70, 90)] {
            let mut w = world();
            w.frame_important = vec![false; crate::tiledata::TILE_COUNT];
            w.frame_important[519] = true;
            // The water column: 195..=200, six deep, well inside the 8-tile
            // distance cap, with the stalk's only cell at 201 on the soil.
            for y in 195..=200 {
                let i = w.index(100, y);
                w.tiles[i].liquid = 200;
            }
            put(&mut w, 100, 202, soil, 0);
            put(&mut w, 100, 201, 519, 0);
            let i = w.index(100, 201);
            w.tiles[i].liquid = 200;
            let mut rng = UnifiedRandom::new(7);
            check_cat_tail(&mut w, 100, 201, &mut rng);
            assert_eq!(
                w.tile(100, 201).map(|t| t.frame_y),
                Some(row),
                "soil {soil} writes frameY {row}"
            );
        }
    }

    #[test]
    fn the_cat_tail_grows_a_base_and_a_tip() {
        // `GrowCatTail` (`:58974-59082`): the base cell walks 0 → 18 →
        // `18 * Next(2, 5)` and spawns the paired 90 tip above, so the stalk
        // gains one cell per armed tip step; the base keeps a 36..=72 row (the
        // `CheckCatTail` frame agreement, `:58848-58863`).
        let mut w = world();
        w.frame_important = vec![false; crate::tiledata::TILE_COUNT];
        w.frame_important[519] = true;
        put(&mut w, 100, 202, 53, 0); // the sand ground
        put(&mut w, 100, 201, 519, 0); // the stalk's base
        for y in 196..=201 {
            let i = w.index(100, y);
            w.tiles[i].liquid = 200;
        }
        let mut rng = UnifiedRandom::new(11);
        for _ in 0..4 {
            grow_cat_tail(&mut w, 100, 201, &mut rng);
        }
        let stalk: Vec<i16> = (190..=201)
            .filter(|&y| w.tile(100, y).is_some_and(|t| t.active() && t.kind == 519))
            .map(|y| w.tile(100, y).map(|t| t.frame_x).unwrap_or(0))
            .collect();
        assert!(stalk.len() >= 2, "the stalk gains a cell above the base");
        let base = w.tile(100, 201).map(|t| t.frame_x).unwrap_or(0);
        assert!(
            [36i16, 54, 72].contains(&base),
            "the base takes a 36/54/72 row, got {base}"
        );
    }

    #[test]
    fn the_cat_tail_dies_out_of_water() {
        // `CheckCatTail` (`:58841-58845`): the cell just above the ground with
        // `liquid < 127` rolls `Next(4)` and an invalid run is killed whole
        // (`:58874-58890`) — a dry stalk never survives a long stream.
        let mut w = world();
        put(&mut w, 100, 202, 2, 0); // dry dirt
        put(&mut w, 100, 201, 519, 0); // the stalk
        let mut rng = UnifiedRandom::new(5);
        for _ in 0..64 {
            check_cat_tail(&mut w, 100, 201, &mut rng);
            if !w.tile(100, 201).is_some_and(|t| t.active() && t.kind == 519) {
                break;
            }
        }
        assert!(
            !w.tile(100, 201).is_some_and(|t| t.active() && t.kind == 519),
            "the dry stalk is killed"
        );
    }

    #[test]
    fn the_seaweed_grows_only_in_full_water() {
        // `GrowCheckSeaweed` (`:58917-58973`): the cell must sit in water
        // (`liquid >= 200`), the two cells above must be full water
        // (`liquid == 255`, `:58938`), the `Next(1)` gate always passes and
        // the write needs the `17 - Next(20)` depth window.
        let mut w = world();
        w.frame_important = vec![false; crate::tiledata::TILE_COUNT];
        w.frame_important[549] = true;
        put(&mut w, 100, 210, 53, 0); // the sand floor
        for y in 200..=209 {
            let i = w.index(100, y);
            w.tiles[i].liquid = 255;
        }
        let mut placed = false;
        for seed in 1..64u64 {
            let mut rng = UnifiedRandom::new(seed as i32);
            grow_check_seaweed(&mut w, 100, 209, &mut rng);
            if w.tile(100, 208).is_some_and(|t| t.active() && t.kind == 549) {
                placed = true;
                break;
            }
        }
        assert!(placed, "the seaweed grows above the sand in full water");

        // The same cell with a half-full water column: no write ever.
        let mut w2 = world();
        put(&mut w2, 100, 210, 53, 0);
        for y in 200..=209 {
            let i = w2.index(100, y);
            w2.tiles[i].liquid = 200;
        }
        for seed in 1..64u64 {
            let mut rng = UnifiedRandom::new(seed as i32);
            grow_check_seaweed(&mut w2, 100, 209, &mut rng);
            assert!(
                !w2.tile(100, 208).is_some_and(|t| t.active()),
                "no growth under a 200-level column"
            );
        }
    }

    #[test]
    fn the_seaweed_dies_out_of_water() {
        // `:58925-58933`: a 549 with `liquid < 200` on itself or the cell above
        // rolls `Next(2)` and dies.
        let mut w = world();
        w.frame_important = vec![false; crate::tiledata::TILE_COUNT];
        w.frame_important[549] = true;
        put(&mut w, 100, 210, 53, 0);
        put(&mut w, 100, 209, 549, 0);
        for seed in 1..64u64 {
            let mut rng = UnifiedRandom::new(seed as i32);
            grow_check_seaweed(&mut w, 100, 209, &mut rng);
            if !w.tile(100, 209).is_some_and(|t| t.active()) {
                break;
            }
        }
        assert!(
            !w.tile(100, 209).is_some_and(|t| t.active()),
            "the dry stand is killed"
        );
    }

    #[test]
    fn the_web_grows_on_a_dungeon_wall() {
        // `GrowWeb` (`:74088-74122`): `Next(10)` per pick, a `Next(2, 4)`
        // radius whose box must touch a solid, then the 51 write in the empty
        // cell. The dispatcher arm (`:73803-73806`) gates on wall 62 and no
        // liquid.
        let mut w = world();
        w.frame_important = vec![false; crate::tiledata::TILE_COUNT];
        for x in 98..=102 {
            put(&mut w, x, 208, 1, 0); // the solid floor inside `Next(2, 4)`
        }
        for x in 98..=102 {
            set_wall(&mut w, x, 205, 62);
        }
        let mut placed = false;
        for seed in 1..64u64 {
            let mut rng = UnifiedRandom::new(seed as i32);
            update_world_underground_tile(&mut w, 100, 205, 3, true, true, false, &mut rng);
            if w.tile(100, 205).is_some_and(|t| t.active() && t.kind == 51) {
                placed = true;
                break;
            }
        }
        assert!(placed, "the web fills an empty wall-62 cell");

        // Without the wall the arm never fires.
        let mut w2 = world();
        for x in 98..=102 {
            put(&mut w2, x, 208, 1, 0);
        }
        for seed in 1..64u64 {
            let mut rng = UnifiedRandom::new(seed as i32);
            update_world_underground_tile(&mut w2, 100, 205, 3, true, true, false, &mut rng);
            assert!(
                !w2.tile(100, 205).is_some_and(|t| t.active()),
                "no web without the 62 wall"
            );
        }
    }

    #[test]
    fn the_dye_plant_styles_follow_the_soil() {
        // `plantDye` band 1 (`:71303-71366`): the dirt/grass pair rolls style
        // 3 or 4 (`frameX` 102/136), the jungle grass style 2/4, the mushroom
        // grass 6; `PlaceDye` (`:53988-54065`) writes `frameX = 34 * style`.
        let mut w = world();
        w.world_surface = 250.0;
        put(&mut w, 100, 220, 2, 0);
        let mut seen = Vec::new();
        for k in 0..2048u64 {
            // The sweep spreads the seeds with the 64-bit golden ratio (the
            // FastRandom-era fixture kept; the i32 truncation preserves the
            // distinct low bits, so consecutive k still land on new styles).
            let seed = k.wrapping_mul(0x9E37_79B9_7F4A_7C15);
            let mut rng = UnifiedRandom::new(seed as i32);
            plant_dye(&mut w, 100, 220, false, &mut rng);
            let fx = w.tile(100, 219).map(|t| t.frame_x).unwrap_or(0);
            if !seen.contains(&fx) {
                seen.push(fx);
            }
            // clear the plant so the density scan stays quiet
            if let Some(t) = w.tile_mut(100, 219) {
                t.flags &= !FLAG_ACTIVE;
            }
        }
        seen.sort_unstable();
        assert!(
            seen.contains(&102) && seen.contains(&136),
            "the dirt pair takes both 3 (102) and 4 (136), saw {seen:?}"
        );

        // The jungle grass 60 rolls style 2 or 4.
        let mut w2 = world();
        w2.world_surface = 250.0;
        put(&mut w2, 100, 220, 60, 0);
        let mut jungle = Vec::new();
        for k in 0..2048u64 {
            let seed = k.wrapping_mul(0x9E37_79B9_7F4A_7C15);
            let mut rng = UnifiedRandom::new(seed as i32);
            plant_dye(&mut w2, 100, 220, false, &mut rng);
            jungle.push(w2.tile(100, 219).map(|t| t.frame_x).unwrap_or(0));
            if let Some(t) = w2.tile_mut(100, 219) {
                t.flags &= !FLAG_ACTIVE;
            }
        }
        assert!(
            jungle.contains(&68) && jungle.contains(&136),
            "the jungle arm takes 2 (68) and 4 (136), saw {jungle:?}"
        );

        // The mushroom grass 80: style 6 (204) with free sides.
        let mut w3 = world();
        w3.world_surface = 250.0;
        put(&mut w3, 100, 220, 80, 0);
        let mut rng = UnifiedRandom::new(3);
        plant_dye(&mut w3, 100, 220, false, &mut rng);
        assert_eq!(
            w3.tile(100, 219).map(|t| (t.kind, t.frame_x)),
            Some((227, 204)),
            "the mushroom grass takes style 6"
        );
    }

    #[test]
    fn the_dye_plant_water_and_hanging_styles() {
        // `:71415-71418`: the underground band's full-water style 5 (170) for a
        // non-jungle soil over a solid, and `:71420-71436`: the hanging style 7
        // (238) written into the empty cell BELOW a kind-0 cell.
        let mut w = world();
        w.world_surface = 100.0; // the cell sits in the underground band
        put(&mut w, 100, 150, 53, 0); // sand
        put(&mut w, 100, 151, 1, 0); // the solid floor
        for y in 148..=149 {
            let i = w.index(100, y);
            w.tiles[i].liquid = 255;
        }
        let mut rng = UnifiedRandom::new(5);
        plant_dye(&mut w, 100, 150, false, &mut rng);
        assert_eq!(
            w.tile(100, 149).map(|t| (t.kind, t.frame_x)),
            Some((227, 170)),
            "the full-water cell takes style 5"
        );

        // The hanging pass: a kind-0 cell with a solid above and an empty cell
        // below.
        let mut w2 = world();
        w2.world_surface = 100.0;
        put(&mut w2, 100, 150, 0, 0); // the active dirt block the pick lands on
        // A solid above stands band 2's free-above gate down, so the hanging
        // pass is not blocked by the 227 band 2 would have written.
        put(&mut w2, 100, 149, 1, 0);
        let mut rng = UnifiedRandom::new(5);
        plant_dye(&mut w2, 100, 150, false, &mut rng);
        assert_eq!(
            w2.tile(100, 151).map(|t| (t.kind, t.frame_x)),
            Some((227, 238)),
            "the hanging style 7 lands below the dirt"
        );
    }

    #[test]
    fn the_dye_plant_exotic_styles() {
        // `:71324-71336`: the exotic arm takes `Next(8, 12)` (styles 8-11,
        // `frameX` 272..374) on grass/moss/dirt, and the density scan keeps the
        // two families apart (`frameX >= 272` counts as exotic).
        let mut w = world();
        w.world_surface = 250.0;
        put(&mut w, 100, 220, 2, 0);
        let mut rng = UnifiedRandom::new(9);
        plant_dye(&mut w, 100, 220, true, &mut rng);
        let fx = w.tile(100, 219).map(|t| t.frame_x).unwrap_or(0);
        assert!(
            (272..=374).contains(&fx) && fx % 34 == 0,
            "the exotic arm writes a 272..374 style column, got {fx}"
        );
        // The exotic density scan stops a second exotic plant in the box.
        let mut w2 = world();
        w2.world_surface = 250.0;
        put(&mut w2, 100, 220, 2, 0);
        put(&mut w2, 130, 219, 227, 272);
        let mut rng = UnifiedRandom::new(9);
        plant_dye(&mut w2, 100, 220, true, &mut rng);
        assert!(
            !w2.tile(100, 219).is_some_and(|t| t.active()),
            "the exotic density scan blocks the neighbour"
        );
    }

    /// W4-d105 doc-sweep: the remix bypass (`WorldGen.cs:71298` /
    /// `:71378`) — on a remix world the surface arm runs at any depth and
    /// the underground gate no longer refuses the above-surface band. The
    /// gen-time twin (`worldgen::surface::plant_dye`) already carried both
    /// halves; the runtime growth twin was missing them.
    #[test]
    fn w4_d105_the_runtime_dye_carries_the_remix_bypass() {
        // A grass cell BELOW the surface (j 220 > world_surface 100): on a
        // classic world band 1 refuses (j >= surface) and band 2's grass
        // dispatch has no grass arm — nothing plants. On remix, band 1's
        // `|| remixWorldGen` lets the grass pick run.
        let mut w = world();
        w.world_surface = 100.0;
        put(&mut w, 100, 220, 2, 0);
        let mut rng = UnifiedRandom::new(9);
        plant_dye(&mut w, 100, 220, false, &mut rng);
        assert!(
            !w.tile(100, 219).is_some_and(|t| t.active() && t.kind == 227),
            "classic: the below-surface grass cell plants nothing"
        );
        let mut w = world();
        w.world_surface = 100.0;
        w.seeds |= crate::seeds::SeedFlags::REMIX;
        put(&mut w, 100, 220, 2, 0);
        let mut rng = UnifiedRandom::new(9);
        plant_dye(&mut w, 100, 220, false, &mut rng);
        assert!(
            w.tile(100, 219).is_some_and(|t| t.active() && t.kind == 227),
            "remix: band 1 runs below the surface"
        );
        // The underground gate's remix half (`:71378`): on a classic world
        // an above-surface cell returns before band 2; on remix band 1
        // runs first. An exotic plant on a dirt cell above the surface:
        // classic band 1 refuses, band 2 runs the exotic arm (one
        // Next(8,12) draw + write); remix runs band 1's exotic arm and
        // band 2 bails on the fresh 227 (one draw + write). Same stream
        // position, same cell — the write side both paths must reach.
        let mut classic = world();
        classic.world_surface = 250.0;
        put(&mut classic, 100, 220, 0, 0);
        let mut remix = world();
        remix.world_surface = 250.0;
        remix.seeds |= crate::seeds::SeedFlags::REMIX;
        put(&mut remix, 100, 220, 0, 0);
        let mut r1 = UnifiedRandom::new(9);
        let mut r2 = UnifiedRandom::new(9);
        plant_dye(&mut classic, 100, 220, true, &mut r1);
        plant_dye(&mut remix, 100, 220, true, &mut r2);
        assert_eq!(r1.next(), r2.next(), "the exotic draw count matches");
        assert!(
            classic.tile(100, 219).is_some_and(|t| t.active() && t.kind == 227)
                && remix.tile(100, 219).is_some_and(|t| t.active() && t.kind == 227),
            "both worlds plant the exotic style"
        );
    }

    /// W4-d105 doc-sweep: the exotic arm's liquid veto
    /// (`WorldGen.cs:71318-71321`) is `lava() || honey()` only — shimmer
    /// passes (the prior `!= Water` port also vetoed shimmer).
    #[test]
    fn w4_d105_the_exotic_veto_is_lava_and_honey_only() {
        use crate::tile::LiquidKind;
        // Shimmer above the grass cell: the exotic plant still lands.
        let mut w = world();
        w.world_surface = 250.0;
        put(&mut w, 100, 220, 2, 0);
        let i = w.index(100, 219);
        w.tiles[i].liquid = 100;
        w.tiles[i].set_liquid_kind(LiquidKind::Shimmer);
        let mut rng = UnifiedRandom::new(9);
        plant_dye(&mut w, 100, 220, true, &mut rng);
        assert!(
            w.tile(100, 219).is_some_and(|t| t.active() && t.kind == 227),
            "shimmer does not veto the exotic arm"
        );
        // Honey above: the veto holds.
        let mut w = world();
        w.world_surface = 250.0;
        put(&mut w, 100, 220, 2, 0);
        let i = w.index(100, 219);
        w.tiles[i].liquid = 100;
        w.tiles[i].set_liquid_kind(LiquidKind::Honey);
        let mut rng = UnifiedRandom::new(9);
        plant_dye(&mut w, 100, 220, true, &mut rng);
        assert!(
            !w.tile(100, 219).is_some_and(|t| t.active() && t.kind == 227),
            "honey vetoes the exotic arm"
        );
        // Lava above: the veto holds.
        let mut w = world();
        w.world_surface = 250.0;
        put(&mut w, 100, 220, 2, 0);
        let i = w.index(100, 219);
        w.tiles[i].liquid = 100;
        w.tiles[i].set_liquid_kind(LiquidKind::Lava);
        let mut rng = UnifiedRandom::new(9);
        plant_dye(&mut w, 100, 220, true, &mut rng);
        assert!(
            !w.tile(100, 219).is_some_and(|t| t.active() && t.kind == 227),
            "lava vetoes the exotic arm"
        );
    }

    #[test]
    fn the_dye_plants_arrive_through_the_dispatchers() {
        // The heads (`:72219-72228` overground `Next(3000)`, `:73255-73262`
        // underground `Next(2500)`) sit behind every `nactive` pick.
        let mut w = world();
        w.world_surface = 100.0;
        put(&mut w, 100, 150, 1, 0); // stone in the underground band
        let mut seen = false;
        for seed in 1..=40000u64 {
            let mut rng = UnifiedRandom::new(seed as i32);
            update_world_underground_tile(&mut w, 100, 150, 3, true, true, false, &mut rng);
            if w.tile(100, 149).is_some_and(|t| t.active() && t.kind == 227) {
                seen = true;
                break;
            }
        }
        assert!(seen, "the underground dye head plants a 227");

        // The overground twin (`:72219-72228`, `Next(3000)`) on a surface
        // dirt cell.
        let mut w2 = world();
        w2.world_surface = 200.0;
        put(&mut w2, 100, 190, 2, 0);
        let mut seen = false;
        for k in 0..40000u64 {
            let seed = k.wrapping_mul(0x9E37_79B9_7F4A_7C15);
            let mut rng = UnifiedRandom::new(seed as i32);
            update_world_overground_tile(&mut w2, 100, 190, true, false, 0.0, 0.0, &mut rng);
            if w2.tile(100, 189).is_some_and(|t| t.active() && t.kind == 227) {
                seen = true;
                break;
            }
        }
        assert!(seen, "the overground dye head plants a 227");
    }

    #[test]
    fn the_tombstone_grows_abigails_flower() {
        // `WorldGen.TryGrowingAbigailsFlower` (`:73049-73073`) from the
        // tombstone arm of `UpdateWorld` (`:72063-72067`): a 1/30 gate, then a
        // spot in the 20x20 box around the grave whose ground is
        // mushroom/ash/conversion grass and which holds no flower within 120.
        let mut w = world();
        for x in 100..=140 {
            put(&mut w, x, 150, 2, 0); // the grass floor
        }
        put(&mut w, 120, 149, 85, 0); // the grave
        let mut flower = None;
        for k in 0..40000u64 {
            let seed = k.wrapping_mul(0x9E37_79B9_7F4A_7C15);
            let mut rng = UnifiedRandom::new(seed as i32);
            update_world_overground_tile(&mut w, 120, 149, true, false, 0.0, 0.0, &mut rng);
            flower = flower.or((110..130).find_map(|x| {
                (140..160)
                    .find(|&y| w.tile(x, y).is_some_and(|t| t.active() && t.kind == 624))
                    .map(|y| (x, y))
            }));
            if flower.is_some() {
                break;
            }
        }
        let (fx, fy) = flower.expect("the grave grows a flower");
        // The 20x20 draw box (`Math.Max(10, i - 10) .. Math.Min(maxX - 10, i + 10)`).
        assert!((110..130).contains(&fx), "x {fx} inside the draw box");
        assert!((140..160).contains(&fy), "y {fy} inside the draw box");
        // The flower sits ON the grass row, never inside it.
        assert_eq!(fy, 149, "the flower stands above the grass");
        assert!(
            w.tile(fx, fy + 1).is_some_and(|t| t.kind == 2),
            "the flower's ground is grass"
        );
        // Frame 0/0 (`WorldGen.cs:59488-59493`), not an autotile frame.
        let t = *w.tile(fx, fy).unwrap();
        assert_eq!((t.frame_x, t.frame_y), (0, 0), "the flower stays unframed");
        // One grave, one flower: the 120-tile box refuses a second.
        let before = (100..160)
            .flat_map(|x| (140..160).map(move |y| (x, y)))
            .filter(|&(x, y)| w.tile(x, y).is_some_and(|t| t.active() && t.kind == 624))
            .count();
        assert_eq!(before, 1);
        for k in 0..20000u64 {
            let seed = k.wrapping_mul(0x9E37_79B9_7F4A_7C15).wrapping_add(7);
            let mut rng = UnifiedRandom::new(seed as i32);
            update_world_overground_tile(&mut w, 120, 149, true, false, 0.0, 0.0, &mut rng);
        }
        let after = (100..160)
            .flat_map(|x| (140..160).map(move |y| (x, y)))
            .filter(|&(x, y)| w.tile(x, y).is_some_and(|t| t.active() && t.kind == 624))
            .count();
        assert_eq!(after, 1, "the nearby-flower rule blocks the second");
    }

    #[test]
    fn the_abigails_flower_ground_and_distance_rules() {
        // `HasValidGroundForAbigailsFlowerBelowSpot` (`:73026-73046`): the
        // mushroom grass 70, the ash grass 633 and every `Conversion.Grass`
        // member pass; dirt, sand and air do not.
        let mut w = world();
        for (kind, ok) in [(2u16, true), (70, true), (633, true), (199, true), (0, false), (53, false), (1, false)]
        {
            let i = w.index(200, 151);
            w.tiles[i] = Tile {
                kind,
                ..Default::default()
            };
            w.tiles[i].flags |= FLAG_ACTIVE;
            assert_eq!(
                has_valid_ground_for_abigails_flower_below_spot(&w, 200, 150),
                ok,
                "ground kind {kind}"
            );
        }
        // Air below is no ground, and the 2-tile margin is refused
        // (`WorldGen.InWorld(x, y, 2)`).
        let i = w.index(200, 151);
        w.tiles[i].flags &= !FLAG_ACTIVE;
        assert!(!has_valid_ground_for_abigails_flower_below_spot(&w, 200, 150));
        put(&mut w, 200, 151, 2, 0);
        assert!(!has_valid_ground_for_abigails_flower_below_spot(&w, 1, 150));
        assert!(!has_valid_ground_for_abigails_flower_below_spot(&w, 200, 1));
        // `NoNearbyAbigailsFlower` (`:73076-73095`): the 120-tile box.
        put(&mut w, 300, 150, 624, 0);
        assert!(!no_nearby_abigails_flower(&w, 200, 150), "100 away is too close");
        assert!(no_nearby_abigails_flower(&w, 100, 150), "200 away is clear");
        // The gate: a non-zero `Main.rand.Next(30)` draws once and writes
        // nothing.
        let mut w2 = world();
        for x in 100..=140 {
            put(&mut w2, x, 150, 2, 0);
        }
        let mut rng = UnifiedRandom::new(4);
        let placed = try_growing_abigails_flower(&mut w2, 120, 149, &mut rng);
        if !placed {
            // The draw-count observable: the stream left its pristine state.
            assert_ne!(rng, UnifiedRandom::new(4), "the gate always draws");
        }
        assert!(
            (100..160)
                .flat_map(|x| (140..160).map(move |y| (x, y)))
                .filter(|&(x, y)| w2.tile(x, y).is_some_and(|t| t.active() && t.kind == 624))
                .count()
                <= 1
        );
    }
}
