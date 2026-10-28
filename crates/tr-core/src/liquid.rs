//! Cellular liquid simulation, ported from `Liquid.UpdateLiquid` / `Liquid.Update`.
//!
//! Terraria keeps a bounded list of *active* liquid tiles instead of scanning the
//! whole world. Every second game tick it pumps one tenth of that list; each entry
//! falls into the cell below and levels out with up to six horizontal neighbours.
//! Entries retire after eight passes without a change, and cells that need work are
//! re-queued through [`LiquidSystem::add_water`] (the original `AddWater` plus its
//! overflow `LiquidBuffer`).

use crate::tile::{FLAG_LIQUID_HONEY, FLAG_LIQUID_LAVA, FLAG_LIQUID_SHIMMER, LiquidKind, Tile};
use crate::tiledata as td;
use crate::world::World;

/// `Liquid.maxLiquid`: active-list capacity.
pub const MAX_ACTIVE: usize = 25_000;
/// `Liquid.maxLiquidBuffer`: overflow queue for extra woken tiles.
pub const MAX_BUFFER: usize = 50_000;
/// `Liquid.cycles`: number of `update` calls a full list pass takes.
pub const CYCLES: usize = 10;
/// `Liquid` removes entries after this many passes without a change.
const KILL_PASSES: u16 = 8;
const KILL_DEAD: u16 = 999;

#[derive(Clone, Copy, Debug)]
struct Entry {
    x: i32,
    y: i32,
    /// Consecutive passes without a change; [`KILL_DEAD`] retires immediately.
    kill: u16,
}

pub struct LiquidSystem {
    width: i32,
    height: i32,
    active: Vec<Entry>,
    /// One bit per tile: `Tile.checkingLiquid` from the original.
    checking: Vec<u64>,
    /// `LiquidBuffer`: cells woken while the active list is full.
    buffer: Vec<(i32, i32)>,
    /// First entry not yet updated in the current pass.
    cursor: usize,
    /// Active-list length when the current pass started; new entries wait it out.
    pass_len: usize,
    /// Set when any level changed during the current [`LiquidSystem::update`].
    dirty: bool,
}

impl LiquidSystem {
    pub fn new(world: &World) -> Self {
        let bits = world.tiles.len().div_ceil(64);
        Self {
            width: world.width,
            height: world.height,
            active: Vec::new(),
            checking: vec![0; bits],
            buffer: Vec::new(),
            cursor: 0,
            pass_len: 0,
            dirty: false,
        }
    }

    pub fn active_len(&self) -> usize {
        self.active.len()
    }

    pub fn buffered_len(&self) -> usize {
        self.buffer.len()
    }

    #[inline]
    fn bit(&self, x: i32, y: i32) -> usize {
        y as usize * self.width as usize + x as usize
    }

    #[inline]
    fn is_checking(&self, x: i32, y: i32) -> bool {
        let i = self.bit(x, y);
        (self.checking[i >> 6] >> (i & 63)) & 1 != 0
    }

    #[inline]
    fn set_checking(&mut self, x: i32, y: i32, v: bool) {
        let i = self.bit(x, y);
        if v {
            self.checking[i >> 6] |= 1 << (i & 63);
        } else {
            self.checking[i >> 6] &= !(1 << (i & 63));
        }
    }

    /// `Liquid.AddWater`: queues one cell if it holds liquid and no entry owns it.
    pub fn add_water(&mut self, world: &World, x: i32, y: i32) {
        if x < 0 || y < 0 || x >= self.width || y >= self.height || self.is_checking(x, y) {
            return;
        }
        let Some(t) = world.tile(x, y) else { return };
        if t.liquid == 0 || is_solid(t) {
            return;
        }
        if self.active.len() >= MAX_ACTIVE {
            if self.buffer.len() < MAX_BUFFER {
                self.buffer.push((x, y));
            }
            return;
        }
        self.set_checking(x, y, true);
        self.active.push(Entry { x, y, kill: 0 });
    }

    /// Queues every liquid cell in a square around an edit, so breaking or placing a
    /// block that borders a pool starts the flow without scanning the whole world.
    pub fn wake_around(&mut self, world: &World, cx: i32, cy: i32, r: i32) {
        for y in (cy - r).max(0)..=(cy + r).min(self.height - 1) {
            for x in (cx - r).max(0)..=(cx + r).min(self.width - 1) {
                self.add_water(world, x, y);
            }
        }
    }

    /// One `Liquid.UpdateLiquid` call. Tick this at 30 Hz, like the original
    /// (every second 60 Hz world tick).
    pub fn update(&mut self, world: &mut World) {
        if self.active.is_empty() {
            self.refill_from_buffer(world);
            if self.active.is_empty() {
                return;
            }
        }
        if self.pass_len == 0 {
            self.pass_len = self.active.len();
            self.cursor = 0;
        }
        let n = self.pass_len;
        let slice = n.div_ceil(CYCLES).max(1);
        let end = (self.cursor + slice).min(n);
        self.dirty = false;
        for i in self.cursor..end {
            let (x, y) = (self.active[i].x, self.active[i].y);
            let kill = self.update_tile(world, x, y, self.active[i].kill);
            self.active[i].kill = kill;
        }
        self.cursor = end;
        if self.cursor >= n {
            self.end_pass(world);
        }
        if self.dirty {
            world.revision = world.revision.wrapping_add(1);
        }
    }

    /// `wetCounter >= cycles` bookkeeping: retire dead entries, promote buffered cells.
    fn end_pass(&mut self, world: &World) {
        let old = std::mem::take(&mut self.active);
        for e in old {
            if e.kill < KILL_PASSES {
                self.active.push(e);
            } else {
                self.set_checking(e.x, e.y, false);
            }
        }
        self.cursor = 0;
        self.pass_len = 0;
        self.refill_from_buffer(world);
    }

    fn refill_from_buffer(&mut self, world: &World) {
        let limit = self.active.len().max(1);
        let mut promoted = 0;
        while promoted < limit && !self.buffer.is_empty() && self.active.len() < MAX_ACTIVE {
            let (x, y) = self.buffer.remove(0);
            let before = self.active.len();
            self.add_water(world, x, y);
            if self.active.len() > before {
                promoted += 1;
            }
        }
    }

    /// `Liquid.Update` for one cell. Returns the new kill counter.
    fn update_tile(&mut self, world: &mut World, x: i32, y: i32, kill: u16) -> u16 {
        let i = world.index(x, y);
        let start = world.tiles[i];
        if is_solid(&start) || start.liquid == 0 {
            return KILL_DEAD;
        }
        let kind = kind_code(start.liquid_kind());

        // --- fall through the cell below ---------------------------------------
        if let Some(below) = world.tile(x, y + 1).copied() {
            if !is_solid(&below)
                && (below.liquid == 0 || kind_code(below.liquid_kind()) == kind)
                && below.liquid < 255
            {
                let mut amount = 255 - below.liquid;
                if amount > start.liquid {
                    amount = start.liquid;
                }
                // The original keeps a 255 source intact when only one unit moves.
                let keep_one = amount == 1 && start.liquid == 255;
                let j = world.index(x, y + 1);
                if !keep_one {
                    world.tiles[i].liquid = start.liquid - amount;
                }
                world.tiles[j].liquid += amount;
                set_kind(&mut world.tiles[j], kind);
                world.mark_chunk(x, y + 1);
                world.mark_chunk(x, y);
                self.dirty = true;
                self.add_water(world, x, y + 1);
                if !keep_one {
                    self.add_water(world, x - 1, y);
                    self.add_water(world, x + 1, y);
                }
            }
        }

        // --- level out with the neighbours on this row --------------------------
        let own = world.tiles[i].liquid;
        if own > 0 {
            let left = world.tile(x - 1, y).copied();
            let right = world.tile(x + 1, y).copied();
            let left2 = world.tile(x - 2, y).copied();
            let left3 = world.tile(x - 3, y).copied();
            let right2 = world.tile(x + 2, y).copied();
            let right3 = world.tile(x + 3, y).copied();

            let same = |t: Option<Tile>| {
                t.is_some_and(|t| {
                    !is_solid(&t) && t.liquid > 0 && kind_code(t.liquid_kind()) == kind
                })
            };
            let open = |t: Option<Tile>| {
                t.is_some_and(|t| {
                    !is_solid(&t) && (t.liquid == 0 || kind_code(t.liquid_kind()) == kind)
                })
            };
            let f2 = open(left);
            let f3 = open(right);
            let mut f4 = same(left2);
            let mut f5 = same(right2);
            if own > 250 {
                f4 = false;
                f5 = false;
            }
            let bias = if own < 3 { -1.0 } else { 0.0 };
            let above_full = world.tile(x, y - 1).is_some_and(|t| t.liquid > 0);
            let mut new_own = own as i32;

            if f2 && f3 {
                let l = left.map_or(0, |t| t.liquid) as f32;
                let r = right.map_or(0, |t| t.liquid) as f32;
                let l2 = left2.map_or(0, |t| t.liquid) as f32;
                let r2 = right2.map_or(0, |t| t.liquid) as f32;
                if f4 && f5 && same(left3) && same(right3) {
                    let l3 = left3.map_or(0, |t| t.liquid) as f32;
                    let r3 = right3.map_or(0, |t| t.liquid) as f32;
                    let avg = round_level(l + r + l2 + r2 + l3 + r3 + own as f32 + bias, 7);
                    let equal = set_level(world, self, x - 1, y, avg, kind)
                        + set_level(world, self, x + 1, y, avg, kind)
                        + set_level(world, self, x - 2, y, avg, kind)
                        + set_level(world, self, x + 2, y, avg, kind)
                        + set_level(world, self, x - 3, y, avg, kind)
                        + set_level(world, self, x + 3, y, avg, kind);
                    if equal < 6 || !above_full {
                        new_own = avg;
                    }
                } else if f4 && f5 {
                    let avg = round_level(l + r + l2 + r2 + own as f32 + bias, 5);
                    let equal = set_level(world, self, x - 1, y, avg, kind)
                        + set_level(world, self, x + 1, y, avg, kind)
                        + set_level(world, self, x - 2, y, avg, kind)
                        + set_level(world, self, x + 2, y, avg, kind);
                    if equal < 4 || !above_full {
                        new_own = avg;
                    }
                } else if f4 {
                    let avg = round_level(l + r + l2 + own as f32 + bias, 4);
                    set_level(world, self, x - 1, y, avg, kind);
                    set_level(world, self, x + 1, y, avg, kind);
                    set_level(world, self, x - 2, y, avg, kind);
                    new_own = avg;
                } else if f5 {
                    let avg = round_level(l + r + r2 + own as f32 + bias, 4);
                    set_level(world, self, x - 1, y, avg, kind);
                    set_level(world, self, x + 1, y, avg, kind);
                    set_level(world, self, x + 2, y, avg, kind);
                    new_own = avg;
                } else {
                    let avg = round_level(l + r + own as f32 + bias, 3);
                    set_level(world, self, x - 1, y, avg, kind);
                    set_level(world, self, x + 1, y, avg, kind);
                    new_own = avg;
                }
            } else if f2 {
                let l = left.map_or(0, |t| t.liquid) as f32;
                let avg = round_level(l + own as f32 + bias, 2);
                set_level(world, self, x - 1, y, avg, kind);
                new_own = avg;
            } else if f3 {
                let r = right.map_or(0, |t| t.liquid) as f32;
                let avg = round_level(r + own as f32 + bias, 2);
                set_level(world, self, x + 1, y, avg, kind);
                new_own = avg;
            }

            if new_own != own as i32 {
                world.tiles[i].liquid = new_own as u8;
                set_kind(&mut world.tiles[i], kind);
                self.dirty = true;
            }
        }

        let now = world.tiles[i].liquid;
        if now != start.liquid {
            if now == 254 && start.liquid == 255 {
                kill.saturating_add(1)
            } else {
                self.add_water(world, x, y - 1);
                0
            }
        } else {
            kill.saturating_add(1)
        }
    }
}

/// Writes one averaged level into a cell. Returns 1 when it already held that level.
fn set_level(
    world: &mut World,
    sys: &mut LiquidSystem,
    x: i32,
    y: i32,
    level: i32,
    kind: u8,
) -> u32 {
    let Some(t) = world.tile(x, y).copied() else {
        return 0;
    };
    if t.liquid as i32 != level {
        let i = world.index(x, y);
        world.tiles[i].liquid = level as u8;
        set_kind(&mut world.tiles[i], kind);
        world.mark_chunk(x, y);
        sys.dirty = true;
        sys.add_water(world, x, y);
        0
    } else {
        1
    }
}

#[inline]
fn round_level(sum: f32, count: i32) -> i32 {
    (sum / count as f32).round_ties_even().clamp(0.0, 255.0) as i32
}

#[inline]
fn kind_code(k: LiquidKind) -> u8 {
    match k {
        LiquidKind::Water => 0,
        LiquidKind::Lava => 1,
        LiquidKind::Honey => 2,
        LiquidKind::Shimmer => 3,
    }
}

fn set_kind(t: &mut Tile, code: u8) {
    t.flags &= !(FLAG_LIQUID_LAVA | FLAG_LIQUID_HONEY | FLAG_LIQUID_SHIMMER);
    match code {
        1 => t.flags |= FLAG_LIQUID_LAVA,
        2 => t.flags |= FLAG_LIQUID_HONEY,
        3 => t.flags |= FLAG_LIQUID_SHIMMER,
        _ => {}
    }
}

/// `Main.tileSolid[type] && !Main.tileSolidTop[type]` on an active tile.
#[inline]
fn is_solid(t: &Tile) -> bool {
    t.active()
        && (t.kind as usize) < td::TILE_COUNT
        && td::TILE_SOLID.has(t.kind)
        && !td::TILE_SOLID_TOP.has(t.kind)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tile::FLAG_ACTIVE;

    fn basin(width: i32, wall_y: i32) -> World {
        let mut w = World::new("liquid".into(), width, wall_y + 3);
        for x in 0..width {
            for y in wall_y..wall_y + 3 {
                let i = w.index(x, y);
                w.tiles[i] = Tile {
                    kind: 1,
                    ..Default::default()
                };
                w.tiles[i].flags |= FLAG_ACTIVE;
            }
        }
        w.frame_important = vec![false; td::TILE_COUNT];
        w.compute_column_surface();
        w.compute_chunks();
        w
    }

    fn put(w: &mut World, x: i32, y: i32, amount: u8) {
        let i = w.index(x, y);
        w.tiles[i].liquid = amount;
        w.mark_chunk(x, y);
    }

    fn total(w: &World) -> i64 {
        w.tiles.iter().map(|t| t.liquid as i64).sum()
    }

    fn run(sys: &mut LiquidSystem, w: &mut World, n: usize) {
        for _ in 0..n {
            sys.update(w);
        }
    }

    #[test]
    fn water_falls_and_fills_the_pit_bottom() {
        let mut w = basin(9, 8);
        put(&mut w, 4, 4, 255);
        let mut sys = LiquidSystem::new(&w);
        sys.add_water(&w, 4, 4);
        run(&mut sys, &mut w, 400);
        assert_eq!(
            w.tile(4, 8).unwrap().liquid,
            0,
            "water must not enter the stone floor"
        );
        for x in 0..9 {
            let l = w.tile(x, 7).unwrap().liquid;
            assert!(l > 20 && l < 40, "row level x{x} = {l}");
        }
        assert!(
            (total(&w) - 255).abs() <= 3,
            "mass drifted to {}",
            total(&w)
        );
    }

    #[test]
    fn pool_levels_out_and_settles() {
        let mut w = basin(11, 6);
        for x in 3..=7 {
            put(&mut w, x, 5, 200);
        }
        let mut sys = LiquidSystem::new(&w);
        sys.wake_around(&w, 5, 5, 4);
        run(&mut sys, &mut w, 400);
        let levels: Vec<i32> = (0..11)
            .map(|x| w.tile(x, 5).unwrap().liquid as i32)
            .collect();
        let max = *levels.iter().max().unwrap();
        let min = *levels.iter().filter(|&&v| v > 0).min().unwrap();
        assert!(max - min <= 2, "not flat: {levels:?}");
        assert!(
            sys.active_len() < 20,
            "still churning: {}",
            sys.active_len()
        );
    }

    #[test]
    fn stone_walls_block_the_spread() {
        let mut w = basin(9, 6);
        put(&mut w, 4, 5, 255);
        for y in 2..6 {
            let i = w.index(5, y);
            w.tiles[i] = Tile {
                kind: 1,
                ..Default::default()
            };
            w.tiles[i].flags |= FLAG_ACTIVE;
        }
        let mut sys = LiquidSystem::new(&w);
        sys.add_water(&w, 4, 5);
        run(&mut sys, &mut w, 300);
        for y in 2..6 {
            assert_eq!(
                w.tile(6, y).unwrap().liquid,
                0,
                "leaked through the wall at y{y}"
            );
        }
    }

    #[test]
    fn a_gap_in_the_dam_drains_into_the_lower_chamber() {
        let mut w = World::new("dam".into(), 20, 12);
        for x in 0..20 {
            for y in 8..12 {
                let i = w.index(x, y);
                w.tiles[i] = Tile {
                    kind: 1,
                    ..Default::default()
                };
                w.tiles[i].flags |= FLAG_ACTIVE;
            }
        }
        // Right chamber floor is two tiles lower.
        for x in 11..20 {
            for y in 8..10 {
                let i = w.index(x, y);
                w.tiles[i] = Tile::default();
            }
        }
        for y in 3..8 {
            let i = w.index(10, y);
            w.tiles[i] = Tile {
                kind: 1,
                ..Default::default()
            };
            w.tiles[i].flags |= FLAG_ACTIVE;
        }
        for x in 0..10 {
            put(&mut w, x, 7, 255);
        }
        w.frame_important = vec![false; td::TILE_COUNT];
        w.compute_column_surface();
        w.compute_chunks();
        let before = total(&w);

        let hole = w.index(10, 7);
        w.tiles[hole] = Tile::default();
        let mut sys = LiquidSystem::new(&w);
        sys.wake_around(&w, 10, 7, 4);
        run(&mut sys, &mut w, 1200);

        let right: i64 = (11..20)
            .flat_map(|x| [9, 10].map(|y| w.tile(x, y).unwrap().liquid as i64))
            .sum();
        assert!(right > 1500, "right chamber only got {right}");
        assert!(
            (total(&w) - before).abs() <= before / 10,
            "mass drifted {} -> {}",
            before,
            total(&w)
        );
    }

    #[test]
    fn add_water_deduplicates_tiles() {
        let mut w = basin(6, 4);
        put(&mut w, 2, 3, 100);
        let mut sys = LiquidSystem::new(&w);
        for _ in 0..5 {
            sys.add_water(&w, 2, 3);
        }
        assert_eq!(sys.active_len(), 1);
        sys.add_water(&w, 2, 2);
        assert_eq!(sys.active_len(), 1, "dry cells are not queued");
    }
}
