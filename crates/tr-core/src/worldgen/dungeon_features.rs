//! The dungeon's global feature passes that run after the crawl —
//! `DungeonCrawler.MakeDungeon`'s tail (`DungeonCrawler.cs:463-505`).
//!
//! The port's crawler (`worldgen::dungeon`) writes brick/wall/carved cells
//! directly, so there are no `DungeonRoom`/`DungeonHall` objects for the
//! feature passes to hang off. [`DungeonFootprint`] recovers the one piece of
//! state they need from the written cells — the bounding box of the dungeon
//! (`DungeonBounds.SetBounds`, grown per hall and room) — and each pass below
//! is a port of its `DungeonGlobal*` counterpart.
//!
//! Ported so far: `DungeonGlobalSpikes` (`MakeDungeon`'s `:0.75` step).
//! Out: the doors and platforms (they need the door/platform lists that
//! `DungeonRoom/Hall.CalculatePlatformsAndDoors` builds), the chests
//! (M5-CHEST-gen), the wall variants, the lights/traps/furniture/paintings/
//! banners, and the entrance-protection terms of
//! `DungeonData.CanGenerateFeatureAt` — the port has no entrance object, so
//! only the world-bounds and `wall != 350` terms are kept (`DungeonData.cs:124-136`).

use crate::tile::{Tile, FLAG_ACTIVE};
use crate::world::World;

use super::dungeon::HallSeed;

/// `TileID.Spike` — the wooden dungeon spike.
pub const SPIKE: u16 = 48;
/// `TileID.WoodenSpike` — the dual-dungeon (blue dungeon) spike.
pub const WOODEN_SPIKE: u16 = 232;
/// `WallID.LihzahrdBrickUnsafe`-adjacent wall the feature passes refuse
/// (`DungeonData.CanGenerateFeatureAt`, `DungeonData.cs:130`).
pub const UNBREAKABLE_WALL: u16 = 350;

/// The dungeon's bounding box, as `DungeonBounds` tracks it — `Left`/`Right`/
/// `Top`/`Bottom`. `MakeDungeon` inflates it by 25 before the lights pass
/// (`:492`), which the spikes pass runs before.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DungeonFootprint {
    pub left: i32,
    pub right: i32,
    pub top: i32,
    pub bottom: i32,
    /// `DungeonData.wallVariants[0]` — the wall the feature passes test
    /// against (`DungeonGlobalSpikes.cs:30`).
    pub wall: u16,
    /// `genVars.brickCrackedTileType` — never a spike support
    /// (`Spikes_CanSupportSpike`, `:243`).
    pub cracked: u16,
}

impl DungeonFootprint {
    /// Recover the box from the cells the crawl painted: every dungeon-wall
    /// cell inside `region` (the crawler's own search window).
    pub fn scan(
        world: &World,
        region: (i32, i32, i32, i32),
        wall: u16,
        cracked: u16,
    ) -> Option<Self> {
        let (x0, y0, x1, y1) = region;
        let (mut left, mut right) = (i32::MAX, i32::MIN);
        let (mut top, mut bottom) = (i32::MAX, i32::MIN);
        for y in y0.max(0)..y1.min(world.height) {
            for x in x0.max(0)..x1.min(world.width) {
                if world.tiles[world.index(x, y)].wall == wall {
                    left = left.min(x);
                    right = right.max(x);
                    top = top.min(y);
                    bottom = bottom.max(y);
                }
            }
        }
        if left > right {
            return None;
        }
        Some(DungeonFootprint {
            left,
            right,
            top,
            bottom,
            wall,
            cracked,
        })
    }

    /// `DungeonData.CanGenerateFeatureAt`'s ported terms.
    fn can_generate_at(&self, world: &World, x: i32, y: i32) -> bool {
        let i = world.index(x, y);
        in_world(world, x, y, 5) && world.tiles[i].wall != UNBREAKABLE_WALL
    }

    /// `DungeonGlobalSpikes.Spikes_CanSupportSpike` (`:230-245`).
    fn can_support_spike(&self, world: &World, x: i32, y: i32) -> bool {
        if !in_world(world, x, y, 0) {
            return false;
        }
        let t = world.tiles[world.index(x, y)];
        if !t.active() {
            return false;
        }
        // `Main.tileFrameImportant[tile.type]` is a *static* TileID table in
        // the C#; the port's `World.frame_important` is a per-world mirror of
        // the placed objects (`framing.rs` clones it), and the dungeon's own
        // `fill_brick` marks every brick it writes in it. Ask the tile data
        // table instead, so a brick is (correctly) not frame-important.
        if frame_important_kind(t.kind) || crate::tileobject::tile_cut(t.kind) {
            return false;
        }
        t.kind != self.cracked
    }
}

/// `Main.tileFrameImportant[type]` from the port's static tile-object table.
fn frame_important_kind(kind: u16) -> bool {
    crate::tileobject::def_for_tile(kind).is_some_and(|d| d.frame_important)
}

fn in_world(world: &World, x: i32, y: i32, margin: i32) -> bool {
    x >= margin && y >= margin && x < world.width - margin && y < world.height - margin
}

fn active(world: &World, x: i32, y: i32) -> bool {
    in_world(world, x, y, 0) && world.tiles[world.index(x, y)].active()
}

fn wall_is(world: &World, x: i32, y: i32, wall: u16) -> bool {
    in_world(world, x, y, 0) && world.tiles[world.index(x, y)].wall == wall
}

/// `Main.tile[x, y].type = type` — retype an existing tile, active flag left
/// alone (`DungeonGlobalSpikes.cs:78`, `:105`).
fn set_kind(world: &mut World, x: i32, y: i32, kind: u16) {
    if !in_world(world, x, y, 0) {
        return;
    }
    let i = world.index(x, y);
    world.tiles[i].kind = kind;
    mark_important(world, kind);
}

/// `Clear(TileDataType.Slope); type; active(true)` — a brand-new spike tile
/// (`DungeonGlobalSpikes.cs:81-84`).
fn make_spike(world: &mut World, x: i32, y: i32, kind: u16) {
    if !in_world(world, x, y, 0) {
        return;
    }
    let i = world.index(x, y);
    world.tiles[i].kind = kind;
    world.tiles[i].flags |= FLAG_ACTIVE;
    world.tiles[i].set_slope(0);
    mark_important(world, kind);
}

fn mark_important(world: &mut World, kind: u16) {
    if world.frame_important.len() > kind as usize {
        world.frame_important[kind as usize] = true;
    }
}

/// `DungeonGlobalSpikes.Spikes` (`DungeonGlobalSpikes.cs:31-244`).
///
/// Two identical-shaped passes: the first turns the *floors* it finds into
/// spike runs (walking the scan tile vertically to a solid tile, then growing
/// a horizontal run of 5..=12, each column up to three tiles tall); the second
/// does the ceilings (walking horizontally to a solid tile, then growing a
/// vertical run).
///
/// `good_world` is `WorldGen.getGoodWorldGen` (the fixed-boi seed), which
/// triples the count (`:41-44`). `global_feature_scalar` is
/// `DungeonData.globalFeatureScalar`, 1.0 for a legacy dungeon (`:363`).
///
/// Returns the number of runs placed (the C#'s `num5`), for tests.
pub fn place_spikes(
    world: &mut World,
    fp: &DungeonFootprint,
    world_surface: f64,
    global_feature_scalar: f64,
    good_world: bool,
    r: &mut HallSeed,
) -> usize {
    if fp.right <= fp.left || fp.bottom <= fp.top {
        return 0;
    }
    if (world_surface as i32) + 25 >= fp.bottom {
        return 0; // the C# would draw `Next(lo, hi)` with `lo >= hi`.
    }
    let num2 = world.width as f64 / 4200.0;
    // `(int)(42f * num2 * Math.Max(1.0, scalar * 0.25))` — the max is not
    // `mul_add` (that would floor to 0.25 and lose the `Max`).
    let mut num6 = (42.0 * num2 * (1.0f64).max(global_feature_scalar * 0.25)) as i32;
    if good_world {
        num6 *= 3;
    }
    let mut total = spikes_floor(world, fp, world_surface, num6, r);
    total += spikes_ceiling(world, fp, world_surface, num6, r);
    total
}

/// The C#'s first `while (num5 < num6)` — floor spikes (`:46-127`).
fn spikes_floor(
    world: &mut World,
    fp: &DungeonFootprint,
    world_surface: f64,
    num6: i32,
    r: &mut HallSeed,
) -> usize {
    let mut num3 = 0;
    let num4 = 1000;
    let mut num5 = 0usize;
    while num5 < num6.max(0) as usize {
        num3 += 1;
        let x0 = r.range(fp.left as u32, fp.right as u32) as i32;
        let mut y = r.range(world_surface as u32 + 25, fp.bottom as u32) as i32;
        let start_x = x0;
        let mut kind = SPIKE;
        let mut flag = true;
        let flag2 = wall_is(world, x0, y, fp.wall);
        if wall_is(world, x0, y, 87) {
            kind = WOODEN_SPIKE;
            flag = false;
        }
        if flag2 && !active(world, x0, y) {
            let mut step = 1;
            if r.next(2) == 0 {
                step = -1;
            }
            // `for (; !Main.tile[x, i].active(); i += num9) {}`.
            while in_world(world, x0, y, 0) && !active(world, x0, y) {
                y += step;
            }
            if active(world, x0 - 1, y)
                && active(world, x0 + 1, y)
                && fp.can_support_spike(world, x0 - 1, y)
                && !active(world, x0 - 1, y - step)
                && !active(world, x0 + 1, y - step)
            {
                num5 += 1;
                // Left arm (`:68-91`).
                let mut budget = r.range(5, 13) as i32;
                let mut x = x0;
                while active(world, x - 1, y)
                    && fp.can_support_spike(world, x - 1, y)
                    && active(world, x, y + step)
                    && active(world, x, y)
                    && !active(world, x, y - step)
                    && budget > 0
                {
                    if !fp.can_generate_at(world, x, y) || !fp.can_generate_at(world, x, y - step) {
                        x -= 1;
                        budget = 0;
                        continue;
                    }
                    set_kind(world, x, y, kind);
                    if !active(world, x - 1, y - step) && !active(world, x + 1, y - step) {
                        make_spike(world, x, y - step, kind);
                        if flag {
                            make_spike(world, x, y - step * 2, kind);
                        }
                    }
                    x -= 1;
                    budget -= 1;
                }
                // Right arm (`:92-124`).
                budget = r.range(5, 13) as i32;
                x = start_x + 1;
                while active(world, x + 1, y)
                    && fp.can_support_spike(world, x + 1, y)
                    && active(world, x, y + step)
                    && active(world, x, y)
                    && !active(world, x, y - step)
                    && budget > 0
                {
                    if !fp.can_generate_at(world, x, y) || !fp.can_generate_at(world, x, y - step) {
                        x += 1;
                        budget = 0;
                        continue;
                    }
                    set_kind(world, x, y, kind);
                    if !active(world, x - 1, y - step) && !active(world, x + 1, y - step) {
                        make_spike(world, x, y - step, kind);
                        if flag {
                            make_spike(world, x, y - step * 2, kind);
                        }
                    }
                    x += 1;
                    budget -= 1;
                }
            }
        }
        if num3 > num4 {
            num3 = 0;
            num5 += 1;
        }
    }
    num5
}

/// The C#'s second `while (num5 < num6)` — ceiling spikes (`:131-239`): the
/// scan tile walks *horizontally* to a wall and the run grows vertically.
fn spikes_ceiling(
    world: &mut World,
    fp: &DungeonFootprint,
    world_surface: f64,
    num6: i32,
    r: &mut HallSeed,
) -> usize {
    let mut num3 = 0;
    let num4 = 1000;
    let mut num5 = 0usize;
    while num5 < num6.max(0) as usize {
        num3 += 1;
        let mut x = r.range(fp.left as u32, fp.right as u32) as i32;
        let y0 = r.range(world_surface as u32 + 25, fp.bottom as u32) as i32;
        let start_y = y0;
        let mut kind = SPIKE;
        let mut flag = true;
        let flag2 = wall_is(world, x, y0, fp.wall);
        if wall_is(world, x, y0, 87) {
            kind = WOODEN_SPIKE;
            flag = false;
        }
        if flag2 && !active(world, x, y0) {
            let mut step = 1;
            if r.next(2) == 0 {
                step = -1;
            }
            // `for (; j > 5 && j < Main.maxTilesX - 5 && !tile[j, i].active(); j += num13) {}`.
            while x > 5 && x < world.width - 5 && !active(world, x, y0) {
                x += step;
            }
            if active(world, x, y0 - 1)
                && active(world, x, y0 + 1)
                && fp.can_support_spike(world, x, y0 - 1)
                && !active(world, x - step, y0 - 1)
                && !active(world, x - step, y0 + 1)
            {
                num5 += 1;
                // Up arm (`:157-186`).
                let mut budget = r.range(5, 13) as i32;
                let mut y = y0;
                while active(world, x, y - 1)
                    && fp.can_support_spike(world, x, y - 1)
                    && active(world, x + step, y)
                    && active(world, x, y)
                    && !active(world, x - step, y)
                    && budget > 0
                {
                    if !fp.can_generate_at(world, x, y) || !fp.can_generate_at(world, x - step, y) {
                        y -= 1;
                        budget = 0;
                        continue;
                    }
                    set_kind(world, x, y, kind);
                    if !active(world, x - step, y - 1) && !active(world, x - step, y + 1) {
                        make_spike(world, x - step, y, kind);
                        if flag {
                            make_spike(world, x - step * 2, y, kind);
                        }
                    }
                    y -= 1;
                    budget -= 1;
                }
                // Down arm (`:187-235`).
                budget = r.range(5, 13) as i32;
                y = start_y + 1;
                while active(world, x, y + 1)
                    && fp.can_support_spike(world, x, y + 1)
                    && active(world, x + step, y)
                    && active(world, x, y)
                    && !active(world, x - step, y)
                    && budget > 0
                {
                    if !fp.can_generate_at(world, x, y) || !fp.can_generate_at(world, x - step, y) {
                        y += 1;
                        budget = 0;
                        continue;
                    }
                    set_kind(world, x, y, kind);
                    if !active(world, x - step, y - 1) && !active(world, x - step, y + 1) {
                        make_spike(world, x - step, y, kind);
                        if flag {
                            make_spike(world, x - step * 2, y, kind);
                        }
                    }
                    y += 1;
                    budget -= 1;
                }
            }
        }
        if num3 > num4 {
            num3 = 0;
            num5 += 1;
        }
    }
    num5
}


/// `Main.tileDungeon` (`Main.cs:10580`'s set) — the dungeon bricks, plus the
/// cracked variants (`TileID.Sets.CrackedBricks`).
pub const DUNGEON_TILES: &[u16] = &[41, 43, 44, 481, 482, 483];
/// `Main.wallDungeon` (`Main.cs:10598-10604`): the three base dungeon walls
/// and their six painted variants.
pub const DUNGEON_WALLS: &[u16] = &[7, 8, 9, 94, 95, 96, 97, 98, 99];

/// `DungeonUtils.IsConsideredDungeonTile` (`DungeonUtils.cs:1809-1830`) for a
/// single-dungeon world.
pub fn is_dungeon_tile(kind: u16) -> bool {
    DUNGEON_TILES.contains(&kind)
}

/// `DungeonUtils.IsConsideredDungeonWall` (`:1894-1915`).
pub fn is_dungeon_wall(wall: u16) -> bool {
    DUNGEON_WALLS.contains(&wall)
}

/// The style rolls `SetupDungeonDataVariables` makes for the dungeon's lights
/// (`DungeonCrawler.cs:213-280`): the three wall variants of the colour, the
/// three distinct lantern styles (`Next(7)` with the C#'s dedupe loops) and
/// `chandelierItemType`'s placement style (`DerivedPlacementDetails` for the
/// items 2652/2653/2654 = tile 34, styles 27/28/29).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DungeonLightStyles {
    pub wall_variants: [u16; 3],
    pub lantern: [i32; 3],
    pub chandelier_style: i32,
}

impl DungeonLightStyles {
    pub fn roll(wall: u16, r: &mut HallSeed) -> Self {
        // `switch (brickWallType)`, `DungeonCrawler.cs:213-231`.
        let wall_variants = match wall {
            8 => [8, 98, 99],
            9 => [9, 96, 97],
            _ => [7, 94, 95],
        };
        let chandelier_style = match wall {
            8 => 28,
            9 => 29,
            _ => 27,
        };
        let mut lantern = [0i32; 3];
        lantern[0] = r.next(7) as i32;
        lantern[1] = r.next(7) as i32;
        while lantern[1] == lantern[0] {
            lantern[1] = r.next(7) as i32;
        }
        lantern[2] = r.next(7) as i32;
        while lantern[2] == lantern[0] || lantern[2] == lantern[1] {
            lantern[2] = r.next(7) as i32;
        }
        DungeonLightStyles {
            wall_variants,
            lantern,
            chandelier_style,
        }
    }
}

fn kind_at(world: &World, x: i32, y: i32) -> u16 {
    if !in_world(world, x, y, 0) {
        return 0;
    }
    world.tiles[world.index(x, y)].kind
}

/// `WorldGen.SolidTile` (`WorldGen.cs:69957-69969`): active, solid,
/// non-solid-top, not a half brick, no slope.
fn solid(world: &World, x: i32, y: i32) -> bool {
    if !in_world(world, x, y, 0) {
        return false;
    }
    let t = &world.tiles[world.index(x, y)];
    t.active()
        && !t.inactive()
        && crate::tiledata::TILE_SOLID.has(t.kind)
        && !crate::tiledata::TILE_SOLID_TOP.has(t.kind)
        && !t.half_brick()
        && t.slope() == 0
}

/// `Collision.CanHit` reduced to the solid-tile line test the light pass needs
/// (`Collision.cs:3050`'s `CanHitLine` march, 1x1 boxes).
fn can_hit(world: &World, x1: i32, y1: i32, x2: i32, y2: i32) -> bool {
    let dx = (x2 - x1).abs();
    let dy = (y2 - y1).abs();
    let steps = dx.max(dy);
    if steps == 0 {
        return true;
    }
    for i in 0..=steps {
        let x = x1 + (x2 - x1) * i / steps;
        let y = y1 + (y2 - y1) * i / steps;
        if (x, y) != (x1, y1) && solid(world, x, y) {
            return false;
        }
    }
    true
}

/// `WorldGen.Place1x2Top` (`WorldGen.cs:46681-46704`): a lantern hanging under
/// a solid tile, two rows tall, `style * 36` into the sheet.
fn place_1x2_top(world: &mut World, x: i32, y: i32, kind: u16, style: i32) {
    if !in_world(world, x, y, 0) || !in_world(world, x, y + 1, 0) {
        return;
    }
    let t = &world.tiles[world.index(x, y - 1)];
    let above_ok = t.active()
        && !t.inactive()
        && crate::tiledata::TILE_SOLID.has(t.kind)
        && !crate::tiledata::TILE_SOLID_TOP.has(t.kind);
    if above_ok && !active(world, x, y + 1) {
        let frame_y = (style * 36) as i16;
        for (row, fy) in [(y, frame_y), (y + 1, frame_y + 18)] {
            let i = world.index(x, row);
            world.tiles[i].kind = kind;
            world.tiles[i].flags |= FLAG_ACTIVE;
            world.tiles[i].frame_x = 0;
            world.tiles[i].frame_y = fy;
            mark_important(world, kind);
        }
    }
}

/// `WorldGen.PlaceChand` (`WorldGen.cs:52721-52800`): the 3x3 chandelier.
fn place_chand(world: &mut World, x: i32, y: i32, kind: u16, style: i32) {
    if !in_world(world, x - 1, y, 0) || !in_world(world, x + 1, y + 2, 0) {
        return;
    }
    let mut ok = true;
    for i in (x - 1)..=(x + 1) {
        for j in y..=(y + 2) {
            if active(world, i, j) {
                ok = false;
            }
        }
    }
    let t = &world.tiles[world.index(x, y - 1)];
    let above = t.active()
        && !t.inactive()
        && crate::tiledata::TILE_SOLID.has(t.kind)
        && !crate::tiledata::TILE_SOLID_TOP.has(t.kind);
    if !above {
        ok = false;
    }
    if !ok {
        return;
    }
    let num2 = style / 36 * 18 * 6;
    let mut num3 = style * 18 * 3;
    if num2 >= 108 {
        num3 -= 54 * (num2 / 108) * 37;
    }
    for row in 0..3i32 {
        for col in 0..3i32 {
            let i = world.index(x - 1 + col, y + row);
            world.tiles[i].kind = kind;
            world.tiles[i].flags |= FLAG_ACTIVE;
            world.tiles[i].frame_x = (num2 + col * 18) as i16;
            world.tiles[i].frame_y = (num3 + row * 18) as i16;
            mark_important(world, kind);
        }
    }
}

/// `DungeonGlobalLights.Lights_GenerateSwitch` (`DungeonGlobalLights.cs:187-211`):
/// find a reachable spot below the light, place a switch (136), wire the
/// L-shaped path to the light's tile and sometimes flip the light's frames
/// (`Next(3) > 0`).
fn lights_generate_switch(world: &mut World, x: i32, y: i32, r: &mut HallSeed) {
    for _ in 0..1000 {
        let sx = x + r.range(0, 25) as i32 - 12;
        let sy = y + r.range(3, 21) as i32;
        if active(world, sx, sy)
            || active(world, sx, sy + 1)
            || !is_dungeon_tile(kind_at(world, sx - 1, sy))
            || !is_dungeon_tile(kind_at(world, sx + 1, sy))
            || !can_hit(world, sx, sy, x, y)
        {
            continue;
        }
        let supported = (solid(world, sx - 1, sy) && kind_at(world, sx - 1, sy) != 10)
            || (solid(world, sx + 1, sy) && kind_at(world, sx + 1, sy) != 10)
            || solid(world, sx, sy + 1);
        if supported
            && is_dungeon_wall(world.tiles[world.index(sx, sy)].wall)
            && (is_dungeon_tile(kind_at(world, sx - 1, sy))
                || is_dungeon_tile(kind_at(world, sx + 1, sy)))
        {
            let i = world.index(sx, sy);
            world.tiles[i].kind = SWITCH;
            world.tiles[i].flags |= FLAG_ACTIVE;
            world.tiles[i].frame_x = 0;
            world.tiles[i].frame_y = 0;
            mark_important(world, SWITCH);
        }
        if !active(world, sx, sy) {
            continue;
        }
        // The wire walk: x first, then y (`DungeonGlobalLights.cs:203-216`).
        let (mut wx, mut wy) = (sx, sy);
        while wx != x || wy != y {
            world
                .wires
                .set(wx, wy, crate::wiring::WireColor::Red, true);
            if wx > x {
                wx -= 1;
            }
            if wx < x {
                wx += 1;
            }
            world
                .wires
                .set(wx, wy, crate::wiring::WireColor::Red, true);
            if wy > y {
                wy -= 1;
            }
            if wy < y {
                wy += 1;
            }
            world
                .wires
                .set(wx, wy, crate::wiring::WireColor::Red, true);
        }
        if r.next(3) > 0 {
            for row in [y, y + 1] {
                if in_world(world, x, row, 0) {
                    let i = world.index(x, row);
                    world.tiles[i].frame_x = 18;
                }
            }
        }
        break;
    }
}

/// `DungeonGlobalLights.Lights` (`DungeonGlobalLights.cs:27-181`): the
/// dungeon's lanterns (tile 42, two rows under a solid tile), chandeliers
/// (tile 34, 3x3), and a wired wall switch for each one.
///
/// `entrance` is `data.dungeonEntrance.Bounds` — the port's entrance shaft.
/// Returns the number of lights placed (the C#'s `num4`).
#[allow(clippy::too_many_arguments)]
pub fn place_lights(
    world: &mut World,
    fp: &DungeonFootprint,
    entrance: Option<(i32, i32, i32, i32)>,
    styles: &DungeonLightStyles,
    global_feature_scalar: f64,
    r: &mut HallSeed,
) -> usize {
    if fp.right <= fp.left || fp.bottom <= fp.top {
        return 0;
    }
    let num2 = world.width as f64 / 4200.0;
    let num5 = (28.0 * num2 * global_feature_scalar) as i32;
    let num3 = 1000;
    let mut attempts = 0;
    let mut placed = 0usize;
    while placed < num5.max(0) as usize {
        attempts += 1;
        let x = r.range(fp.left as u32, fp.right as u32) as i32;
        let y = r.range(fp.top as u32, fp.bottom as u32) as i32;
        if is_dungeon_wall(world.tiles[world.index(x, y)].wall) {
            let mut y2 = y;
            while y2 > fp.top {
                let ok = active(world, x, y2 - 1)
                    && is_dungeon_tile(kind_at(world, x, y2 - 1))
                    && fp.can_generate_at(world, x, y2)
                    && (entrance.is_some_and(|e| x >= e.0 && x < e.2 && y2 >= e.1 && y2 < e.3)
                        || is_dungeon_wall(world.tiles[world.index(x, y2)].wall));
                if !ok {
                    y2 -= 1;
                    continue;
                }
                // The 30x30 "already lit" box (`:44-58`).
                let mut flag = false;
                for i in (x - 15)..(x + 15) {
                    for j in (y2 - 15)..(y2 + 15) {
                        if in_world(world, i, j, 0) && matches!(kind_at(world, i, j), LANTERN | CHANDELIER)
                        {
                            flag = true;
                            break;
                        }
                    }
                    if flag {
                        break;
                    }
                }
                // The blocked-sides test (`:59-62`).
                if active(world, x - 1, y2)
                    || active(world, x + 1, y2)
                    || active(world, x - 1, y2 + 1)
                    || active(world, x + 1, y2 + 1)
                    || active(world, x, y2 + 2)
                {
                    flag = true;
                }
                if flag {
                    break;
                }
                // The chandelier arm (`:64-86`): a 1-in-7 roll where a 15-row
                // clearance below opens the 3x3.
                if r.next(7) == 0 {
                    let mut blocked = false;
                    for k in 0..15 {
                        if solid(world, x, y2 + k) {
                            blocked = true;
                            break;
                        }
                    }
                    if !blocked {
                        place_chand(world, x, y2, CHANDELIER, styles.chandelier_style);
                        if kind_at(world, x, y2) == CHANDELIER {
                            attempts = 0;
                            placed += 1;
                            lights_generate_switch(world, x, y2, r);
                            break;
                        }
                    }
                }
                // The lantern arm (`:87-139`).
                let wall = world.tiles[world.index(x, y2)].wall;
                let mut style = styles.lantern[0];
                if wall == styles.wall_variants[1] {
                    style = styles.lantern[1];
                }
                if wall == styles.wall_variants[2] {
                    style = styles.lantern[2];
                }
                if r.next(3) == 0 && (94..=105).contains(&wall) {
                    style = 53;
                }
                place_1x2_top(world, x, y2, LANTERN, style);
                if kind_at(world, x, y2) == LANTERN {
                    attempts = 0;
                    placed += 1;
                    lights_generate_switch(world, x, y2, r);
                }
                break;
            }
        }
        if attempts > num3 {
            attempts = 0;
            placed += 1;
        }
    }
    placed
}

/// `TileID.Lantern` — the hanging lantern.
pub const LANTERN: u16 = 42;
/// `TileID.Chandelier`.
pub const CHANDELIER: u16 = 34;
/// `TileID.Switch` — the wired wall switch the light pass leaves behind.
pub const SWITCH: u16 = 136;

#[allow(dead_code)]
fn _uses_tile(_t: Tile) {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tile::FLAG_ACTIVE;

    /// A `width`-wide, 120-tall box of dungeon wall whose ceiling (rows
    /// `..=26`) and floor (rows `floor_y..`) are solid brick — the shape
    /// `place_spikes` expects. The spike budget is `42 * width / 4200`, so a
    /// 4200-wide fixture gets 42 runs per pass.
    fn room(width: i32, floor_y: i32) -> (World, DungeonFootprint) {
        let mut w = World::new("spike-test".into(), width, 120);
        let wall = 7u16;
        let brick = 41u16;
        for x in 4..width - 4 {
            for y in 20..110 {
                let i = w.index(x, y);
                w.tiles[i].wall = wall;
                if y >= floor_y || y <= 26 {
                    w.tiles[i].kind = brick;
                    w.tiles[i].flags |= FLAG_ACTIVE;
                }
            }
        }
        let fp = DungeonFootprint {
            left: 8,
            right: width - 8,
            top: 24,
            bottom: floor_y,
            wall,
            cracked: 481,
        };
        (w, fp)
    }

    /// A long dungeon corridor: brick ceiling at rows 58-60, air 61-63, brick
    /// floor 64-70, dungeon wall 7 through the air band — the shape the light
    /// pass hangs lanterns in. `width` 4200 gives the 28-light budget.
    fn corridor(width: i32) -> (World, DungeonFootprint) {
        let mut w = World::new("light-test".into(), width, 120);
        for x in 4..width - 4 {
            for y in 0..120 {
                let i = w.index(x, y);
                w.tiles[i].wall = 7;
                if (58..=60).contains(&y) || y >= 64 {
                    w.tiles[i].kind = 41;
                    w.tiles[i].flags |= FLAG_ACTIVE;
                }
            }
        }
        let fp = DungeonFootprint {
            left: 8,
            right: width - 8,
            top: 40,
            bottom: 64,
            wall: 7,
            cracked: 481,
        };
        (w, fp)
    }

    fn spike_count(w: &World) -> usize {
        w.tiles.iter().filter(|t| t.kind == SPIKE && t.active()).count()
    }

    /// `DungeonGlobalSpikes.Spikes`: the floor half grows runs on the floor
    /// line (`:68-91`) — the support rows stay brick and the spikes stand one
    /// and two tiles above them.
    #[test]
    fn the_floor_spike_run_stands_on_the_floor() {
        let (mut w, fp) = room(4200, 80);
        let mut r = HallSeed::new(0x5EED);
        let placed = place_spikes(&mut w, &fp, 40.0, 1.0, false, &mut r);
        assert!(placed > 0, "the pass places runs");
        let mut on_floor = 0;
        let mut on_ceiling = 0;
        for y in 0..w.height {
            for x in 0..w.width {
                let t = w.tiles[w.index(x, y)];
                if t.kind == SPIKE && t.active() {
                    if y > 40 {
                        assert!(
                            (78..=80).contains(&y),
                            "a floor spike at ({x},{y}) is off the floor line"
                        );
                        on_floor += 1;
                    } else {
                        // The ceiling half's runs (`:157-235`) sit one and two
                        // tiles *below* the ceiling row.
                        assert!(
                            (26..=28).contains(&y),
                            "a ceiling spike at ({x},{y}) is off the ceiling line"
                        );
                        on_ceiling += 1;
                    }
                }
            }
        }
        assert!(on_floor > 0, "the floor half writes its runs");
        assert!(on_ceiling > 0, "and the ceiling half writes its own");
    }

    /// `DungeonGlobalLights.Lights`: a lantern hangs two rows under a dungeon
    /// brick ceiling, framed `style * 36` / `+18` (`WorldGen.cs:46694-46701`).
    #[test]
    fn a_lantern_hangs_under_the_dungeon_ceiling() {
        let (mut w, fp) = corridor(4200);
        let mut r = HallSeed::new(0x11);
        let styles = DungeonLightStyles::roll(fp.wall, &mut r);
        let placed = place_lights(&mut w, &fp, None, &styles, 1.0, &mut r);
        assert!(placed > 0, "the pass places lights");
        let mut lanterns = 0;
        for y in 0..w.height {
            for x in 0..w.width {
                let t = w.tiles[w.index(x, y)];
                if t.kind == LANTERN && t.active() {
                    lanterns += 1;
                    // Rows 0 and 1 of the 1x2-top object: frame_y = style*36
                    // and +18, frame_x 0.
                    assert_eq!(t.frame_x, 0);
                    let row = (t.frame_y as i32 - styles.lantern[0] * 36).abs();
                    assert!(
                        row == 0 || row == 18,
                        "frame_y {} is not the lantern's own row",
                        t.frame_y
                    );
                    assert!(
                        (61..=62).contains(&y),
                        "a lantern at ({x},{y}) hangs off the ceiling row"
                    );
                }
            }
        }
        assert!(lanterns >= 2, "and the 1x2-top pair is written");
    }

    /// `Lights_GenerateSwitch`: a switch lands in the reachable nook below the
    /// light and a red wire runs from it back to the light's tile
    /// (`DungeonGlobalLights.cs:187-216`).
    #[test]
    fn the_switch_wires_back_to_its_light() {
        // An open room (rows 61..=80 free) with two brick pillars at columns
        // 29 and 31 — the 1-wide nook a switch needs (dungeon tiles on both
        // sides plus a solid floor below).
        let mut w = World::new("switch-test".into(), 60, 120);
        for x in 0..60 {
            for y in 0..120 {
                let i = w.index(x, y);
                w.tiles[i].wall = 7;
                let solid_row = y <= 60 || y >= 81;
                let pillar = (x == 29 || x == 31) && (64..=80).contains(&y);
                if solid_row || pillar {
                    w.tiles[i].kind = 41;
                    w.tiles[i].flags |= FLAG_ACTIVE;
                }
            }
        }
        let mut r = HallSeed::new(0x33);
        lights_generate_switch(&mut w, 30, 62, &mut r);
        let switch = (0..w.height)
            .flat_map(|y| (0..w.width).map(move |x| (x, y)))
            .find(|&(x, y)| {
                let t = w.tiles[w.index(x, y)];
                t.kind == SWITCH && t.active()
            });
        let (sx, sy) = switch.expect("the switch lands in the nook");
        assert_eq!(sx, 30, "the only nook column");
        assert!((64..=80).contains(&sy));
        // The wire walk sets the switch's cell, the light's cell and the L
        // between them (`:203-216`).
        assert!(
            w.wires.get(sx, sy, crate::wiring::WireColor::Red),
            "the switch is wired"
        );
        let mut wired_up = false;
        for y in 62..=sy {
            if w.wires.get(30, y, crate::wiring::WireColor::Red) {
                wired_up = true;
            }
        }
        assert!(wired_up, "the path back toward the light is wired");
    }

    /// The style roll (`DungeonCrawler.cs:213-280`): the colour's wall
    /// variants, three *distinct* lantern styles, the colour's chandelier
    /// style.
    #[test]
    fn the_light_styles_roll_the_colour_family() {
        let mut r = HallSeed::new(5);
        let blue = DungeonLightStyles::roll(7, &mut r);
        assert_eq!(blue.wall_variants, [7, 94, 95]);
        assert_eq!(blue.chandelier_style, 27);
        assert!(blue.lantern.iter().all(|&s| (0..7).contains(&s)));
        assert!(blue.lantern[0] != blue.lantern[1] && blue.lantern[1] != blue.lantern[2]);
        assert!(blue.lantern[0] != blue.lantern[2], "three distinct styles");
        let green = DungeonLightStyles::roll(8, &mut r);
        assert_eq!(green.wall_variants, [8, 98, 99]);
        assert_eq!(green.chandelier_style, 28);
        let pink = DungeonLightStyles::roll(9, &mut r);
        assert_eq!(pink.wall_variants, [9, 96, 97]);
        assert_eq!(pink.chandelier_style, 29);
    }

    /// `WorldGen.PlaceChand`'s 3x3 write with its frame math
    /// (`WorldGen.cs:52745-52760`): style 27 → frame_y 1458 rows, frame_x
    /// 0/18/36 across.
    #[test]
    fn a_chandelier_is_a_three_by_three_block() {
        let (mut w, _fp) = corridor(120);
        // A solid ceiling row and air below at column 20.
        let mut r = HallSeed::new(1);
        place_chand(&mut w, 20, 61, CHANDELIER, 27);
        let mut cells = 0;
        for row in 61..64 {
            for col in 19..22 {
                let t = w.tiles[w.index(col, row)];
                if t.kind == CHANDELIER && t.active() {
                    cells += 1;
                    assert_eq!(t.frame_y, (1458 + (row - 61) * 18) as i16);
                    assert_eq!(t.frame_x, ((col - 19) * 18) as i16);
                }
            }
        }
        assert_eq!(cells, 9, "all nine cells");
        let _ = &mut r;
    }

    /// The same seed replays the same tiles — the pass draws only from its own
    /// stream, so a world is reproducible.
    #[test]
    fn the_spike_pass_is_seeded() {
        let (mut a, fp) = room(120, 80);
        let (mut b, _) = room(120, 80);
        let mut ra = HallSeed::new(99);
        let mut rb = HallSeed::new(99);
        let na = place_spikes(&mut a, &fp, 40.0, 1.0, false, &mut ra);
        let nb = place_spikes(&mut b, &fp, 40.0, 1.0, false, &mut rb);
        assert_eq!(na, nb);
        let pa: Vec<(u16, u16)> = a.tiles.iter().map(|t| (t.kind, t.wall)).collect();
        let pb: Vec<(u16, u16)> = b.tiles.iter().map(|t| (t.kind, t.wall)).collect();
        assert_eq!(pa, pb, "identical worlds");
    }

    /// Cracked bricks can never be a spike support
    /// (`Spikes_CanSupportSpike`, `:243`) — the runs stop dead on them.
    #[test]
    fn cracked_bricks_support_no_spikes() {
        let (mut w, fp) = room(4200, 80);
        // Both support lines: the floor row the first half tests (`:62`) and
        // the ceiling row the second half tests (`:149`).
        for x in 0..w.width {
            for y in [80, 26] {
                let i = w.index(x, y);
                w.tiles[i].kind = fp.cracked;
            }
        }
        let mut r = HallSeed::new(7);
        // The C#'s `num5` counts timed-out attempts too (`:126`), so the
        // return value is not the signal here — the tiles are.
        place_spikes(&mut w, &fp, 40.0, 1.0, false, &mut r);
        assert_eq!(spike_count(&w), 0);
    }

    /// `world_surface + 25` above the box's bottom: the C# would draw an empty
    /// `Next(lo, hi)` range, so the pass does nothing.
    #[test]
    fn a_box_above_the_surface_scan_places_nothing() {
        let (mut w, fp) = room(120, 80);
        let mut r = HallSeed::new(1);
        assert_eq!(place_spikes(&mut w, &fp, 90.0, 1.0, false, &mut r), 0);
    }

    /// The run budget (`:37-44`): `42 * maxTilesX / 4200`, two halves, each
    /// attempt capped at 1000 scans.
    #[test]
    fn the_run_budget_scales_with_the_world() {
        let (mut w, fp) = room(8400, 80);
        let mut r = HallSeed::new(3);
        let placed = place_spikes(&mut w, &fp, 30.0, 1.0, false, &mut r);
        assert!(
            (40..=200).contains(&placed),
            "two passes of 84 runs each, with timeouts: {placed}"
        );
    }

}
