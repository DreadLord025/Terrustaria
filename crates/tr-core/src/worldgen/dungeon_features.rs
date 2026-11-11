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
