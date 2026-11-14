//! Cellular liquid simulation, ported from `Liquid.UpdateLiquid` / `Liquid.Update`.
//!
//! Terraria keeps a bounded list of *active* liquid tiles instead of scanning the
//! whole world. Every second game tick it walks one `curMaxLiquid / cycles` stride
//! of that list (a list shorter than one stride is re-walked in full); each entry
//! falls into the cell below and levels out with up to six horizontal neighbours.
//! Entries retire after eight passes without a change, and cells that need work are
//! re-queued through [`LiquidSystem::add_water`] (the original `AddWater` plus its
//! overflow `LiquidBuffer`).

use crate::tile::{LiquidKind, Tile, FLAG_LIQUID_HONEY, FLAG_LIQUID_LAVA, FLAG_LIQUID_SHIMMER};
use crate::tiledata as td;
use crate::world::World;

/// `Liquid.maxLiquid`: active-list capacity.
pub const MAX_ACTIVE: usize = 25_000;
/// `Liquid.maxLiquidBuffer`: overflow queue for extra woken tiles.
pub const MAX_BUFFER: usize = 50_000;
/// `Liquid.cycles`' default (`Liquid.cs:88`/`:1673`): number of `update` calls
/// a full list pass takes. The gfx-quality pass overrides it per update at
/// runtime (`Main.cs:17078` — `17 - 10 * gfxQuality`), so the runtime value
/// rides the [`LiquidSystem::cycles`] field; the const stays the generator /
/// pre-quality default the unit tests pin.
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
    /// `Liquid.delay` (`Liquid.cs:63`): the per-entry flow throttle — lava
    /// flows every 6th tick, honey every 11th (`Liquid.cs:484-490`/`:513-522`
    /// gate the movement, never the reactions).
    delay: u8,
}

pub struct LiquidSystem {
    width: i32,
    height: i32,
    active: Vec<Entry>,
    /// One bit per tile: `Tile.checkingLiquid` from the original.
    checking: Vec<u64>,
    /// `LiquidBuffer`: cells woken while the active list is full.
    buffer: Vec<(i32, i32)>,
    /// `Liquid.wetCounter`: the 1-based window index inside the current cycle.
    wet_counter: usize,
    /// `Liquid.curMaxLiquid`: 25 000 in single-player. Multiplayer scales it by the
    /// active-player count (`Liquid.cs:998`), which stays out of scope here.
    cur_max_liquid: usize,
    /// `Liquid.cycles` (`Liquid.cs:25`): the runtime value the gfx-quality
    /// pass overrides (`Main.cs:17078`); the default is the statics' 10.
    cycles: usize,
    /// Set when any level changed during the current [`LiquidSystem::update`].
    dirty: bool,
    /// `Liquid.quickFall` (`Liquid.cs:34`): the quick-settle mode sets it and
    /// forces every entry's `delay = 10` in the pass loop (`:1086-1092`), so
    /// lava/honey flow every tick. The runtime pass keeps it `false`; the
    /// `QuickWater`/panic family (R-11-7) flips it.
    quick_fall: bool,
    /// `WorldGen.remixWorldGen` — the desert-strip arm's gate
    /// (`Liquid.cs:1447`); the R-12-5 plumbing owns it, `false` at runtime.
    remix_world_gen: bool,
    /// `WorldGen.isGeneratingOrLoadingWorld` — the merge-tile branch selector
    /// (`Liquid.cs:1333`/`:1447`). The port's settlement pass never runs
    /// during generation yet (R-12-5 plumbs it), so it stays `false`.
    generating: bool,
    /// The `WorldGen.PlayLiquidChangeSound` events (`Liquid.cs:1345-1347`,
    /// S1-E-25): one row per live-game merge tile — `(kind, x, y)` in tile
    /// coordinates, drained by the audio caller after the pass. The C# plays
    /// the cue inline (`!Main.gameMenu && !isGeneratingOrLoadingWorld`);
    /// the port's engine is pure, so the caller's seam carries it.
    merge_events: Vec<(TileChangeType, i32, i32)>,
}

/// One `PlayLiquidChangeSound` row (`WorldGen.cs:4632-4650`): the merge kind
/// and the merge-tile's tile coordinates (the C#'s `x`/`y`; the sound lands
/// at `x * 16 + 8, y * 16 + 8` — `count` is the default 1).
pub type LiquidMergeEvent = (TileChangeType, i32, i32);

impl LiquidSystem {
    /// `Liquid.maxLiquid` (`Liquid.cs:1672`) — the active-list capacity the
    /// quality pass scales `curMaxLiquid` against.
    pub fn max_active(&self) -> usize {
        MAX_ACTIVE
    }

    /// `Liquid.cycles` — the runtime value the gfx-quality pass overrides
    /// (`Main.cs:17078`), read by the pass stride.
    pub fn cycles(&self) -> usize {
        self.cycles
    }

    pub fn set_cycles(&mut self, cycles: usize) {
        self.cycles = cycles.max(1);
    }

    /// `Liquid.curMaxLiquid` — the quality-scaled active-list window.
    pub fn cur_max_liquid(&self) -> usize {
        self.cur_max_liquid
    }

    pub fn set_cur_max_liquid(&mut self, cur_max_liquid: usize) {
        self.cur_max_liquid = cur_max_liquid.max(1);
    }

    pub fn new(world: &World) -> Self {
        let bits = world.tiles.len().div_ceil(64);
        Self {
            width: world.width,
            height: world.height,
            active: Vec::new(),
            checking: vec![0; bits],
            buffer: Vec::new(),
            wet_counter: 0,
            cur_max_liquid: MAX_ACTIVE,
            cycles: CYCLES,
            dirty: false,
            quick_fall: false,
            generating: false,
            remix_world_gen: false,
            merge_events: Vec::new(),
        }
    }

    pub fn active_len(&self) -> usize {
        self.active.len()
    }

    /// The `PlayLiquidChangeSound` rows this pass queued (`Liquid.cs:1345-1347`),
    /// drained by the audio caller — the C# plays them inline during the
    /// merge (`!Main.gameMenu && !isGeneratingOrLoadingWorld`).
    pub fn drain_merge_events(&mut self) -> Vec<LiquidMergeEvent> {
        std::mem::take(&mut self.merge_events)
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
        self.active.push(Entry {
            x,
            y,
            kill: 0,
            delay: 0,
        });
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
        // `Liquid.cs:1069-1106`: the window is a fixed `curMaxLiquid / cycles`
        // stride and the `numLiquid` clamp re-walks the *whole* list whenever it is
        // shorter than one stride, so a small active list is simulated every call
        // instead of once per ten.
        self.wet_counter += 1;
        let stride = self.cur_max_liquid / self.cycles;
        let mut start = stride * (self.wet_counter - 1);
        let mut end = stride * self.wet_counter;
        if self.wet_counter == self.cycles {
            end = self.active.len();
        }
        if end > self.active.len() {
            end = self.active.len();
            self.wet_counter = self.cycles;
        }
        if start > end {
            start = end;
        }
        self.dirty = false;
        for i in start..end {
            let (x, y) = (self.active[i].x, self.active[i].y);
            // `Liquid.UpdateLiquid`'s quickFall pass forces every entry's
            // delay to 10 (`Liquid.cs:1086-1089`) so the throttle gates
            // never fire.
            let delay = if self.quick_fall {
                10
            } else {
                self.active[i].delay
            };
            let (kill, delay) = self.update_tile(world, x, y, self.active[i].kill, delay);
            self.active[i].kill = kill;
            self.active[i].delay = delay;
        }
        if self.wet_counter >= self.cycles {
            self.wet_counter = 0;
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

    /// `Liquid.Update` for one cell. Returns `(kill, delay)`.
    fn update_tile(
        &mut self,
        world: &mut World,
        x: i32,
        y: i32,
        kill: u16,
        delay: u8,
    ) -> (u16, u8) {
        let i = world.index(x, y);
        let start = world.tiles[i];
        if is_solid(&start) || start.liquid == 0 {
            return (KILL_DEAD, delay);
        }
        let kind = kind_code(start.liquid_kind());

        // --- `Liquid.Update`'s reaction block (`Liquid.cs:475-560`) -----------
        // A lava cell checks itself first; every other cell wakes its lava
        // neighbours and falls through to the honey arm, then the shimmer
        // arm. The reactions run EVERY tick — the delay gate below
        // throttles only the flow, and `quickFall` bypasses it
        // (`Liquid.cs:484-490`/`:513-522`).
        let mut flow_delay = delay;
        if kind == 1 {
            self.lava_check(world, x, y);
            if !self.quick_fall {
                if delay < 5 {
                    return (kill, delay + 1);
                }
                flow_delay = 0;
            }
        } else {
            for (nx, ny) in [(x - 1, y), (x + 1, y), (x, y - 1), (x, y + 1)] {
                if world
                    .tile(nx, ny)
                    .is_some_and(|t| t.liquid > 0 && t.liquid_kind() == LiquidKind::Lava)
                {
                    self.add_water(world, nx, ny);
                }
            }
            if kind == 2 {
                self.honey_check(world, x, y);
                if !self.quick_fall {
                    if delay < 10 {
                        return (kill, delay + 1);
                    }
                    flow_delay = 0;
                }
            } else {
                for (nx, ny) in [(x - 1, y), (x + 1, y), (x, y - 1), (x, y + 1)] {
                    if world
                        .tile(nx, ny)
                        .is_some_and(|t| t.liquid > 0 && t.liquid_kind() == LiquidKind::Honey)
                    {
                        self.add_water(world, nx, ny);
                    }
                }
                if kind == 3 {
                    self.shimmer_check(world, x, y);
                } else {
                    for (nx, ny) in [(x - 1, y), (x + 1, y), (x, y - 1), (x, y + 1)] {
                        if world.tile(nx, ny).is_some_and(|t| {
                            t.liquid > 0 && t.liquid_kind() == LiquidKind::Shimmer
                        }) {
                            self.add_water(world, nx, ny);
                        }
                    }
                }
            }
        }

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
                (kill.saturating_add(1), flow_delay)
            } else {
                self.add_water(world, x, y - 1);
                (0, flow_delay)
            }
        } else {
            (kill.saturating_add(1), flow_delay)
        }
    }

    /// `Liquid.LavaCheck` (`Liquid.cs:1445-1459`): the generation-only
    /// underground-desert lava strip, then the merge check.
    fn lava_check(&mut self, world: &mut World, x: i32, y: i32) {
        if self.generating
            && !self.remix_world_gen
            && !world
                .micro_seeds
                .contains(crate::seeds::MicroSeedFlags::DUAL_DUNGEONS)
            && self.underground_desert_check(world, x, y)
        {
            // `Main.tile[i, j].lava(lava: true)` — the TYPE bits only; an
            // empty cell is pre-typed lava for whatever pours in later.
            for i in (x - 3)..=(x + 3) {
                for j in (y - 3)..=(y + 3) {
                    if i >= 0 && j >= 0 && i < self.width && j < self.height {
                        let idx = world.index(i, j);
                        set_kind(&mut world.tiles[idx], 1);
                    }
                }
            }
        }
        self.liquid_check(world, x, y, 1, false);
    }

    /// `Liquid.HoneyCheck` (`Liquid.cs:1460-1463`).
    fn honey_check(&mut self, world: &mut World, x: i32, y: i32) {
        self.liquid_check(world, x, y, 2, false);
    }

    /// `Liquid.ShimmerCheck` (`Liquid.cs:1465-1468`).
    fn shimmer_check(&mut self, world: &mut World, x: i32, y: i32) {
        self.liquid_check(world, x, y, 3, false);
    }

    /// `Liquid.LiquidCheck` (`Liquid.cs:1225-1373`) — the whole merge
    /// surface. The side/up arms pool the different-type neighbour liquid
    /// (zeroing it even when the merge gate then refuses — the C#'s own
    /// consumption) and merge at the checking cell; the down arm burns the
    /// cut plant below, honours the container override and merges at the
    /// cell below. `createMergeTilesDuringGen` rides the worldgen caller
    /// (`WorldGen.cs:22480` passes `true`).
    fn liquid_check(
        &mut self,
        world: &mut World,
        x: i32,
        y: i32,
        this_liquid_type: u8,
        create_merge_tiles_during_gen: bool,
    ) {
        // `WorldGen.InWorld(x, y, 1)` + `WorldGen.SolidTile(x, y)` (`:1227`).
        if x < 1 || y < 1 || x >= self.width - 1 || y >= self.height - 1 {
            return;
        }
        if self.check_solid(world, x, y) {
            return;
        }
        let left = world.tile(x - 1, y).copied().unwrap_or_default();
        let right = world.tile(x + 1, y).copied().unwrap_or_default();
        let up = world.tile(x, y - 1).copied().unwrap_or_default();
        let here = world.tile(x, y).copied().unwrap_or_default();

        // `Liquid.cs:1233-1236`: a side/up neighbour holds a liquid of a
        // DIFFERENT type.
        if any_other(&left, this_liquid_type)
            || any_other(&right, this_liquid_type)
            || any_other(&up, this_liquid_type)
        {
            let water_nearby = any_is(&left, 0) || any_is(&right, 0) || any_is(&up, 0);
            let lava_nearby = any_is(&left, 1) || any_is(&right, 1) || any_is(&up, 1);
            let honey_nearby = any_is(&left, 2) || any_is(&right, 2) || any_is(&up, 2);
            let shimmer_nearby = any_is(&left, 3) || any_is(&right, 3) || any_is(&up, 3);
            let mut num = 0u32;
            for t in [&left, &right, &up] {
                if tile_liquid_type(t) != this_liquid_type {
                    num += t.liquid as u32;
                }
            }
            for (dx, dy) in [(-1i32, 0), (1, 0), (0, -1)] {
                let idx = world.index(x + dx, y + dy);
                if tile_liquid_type(&world.tiles[idx]) != this_liquid_type {
                    world.tiles[idx].liquid = 0;
                }
            }
            let (liquid_merge_tile_type, liquid_merge_type) = get_liquid_merge_types(
                this_liquid_type,
                water_nearby,
                lava_nearby,
                honey_nearby,
                shimmer_nearby,
            );
            let i = world.index(x, y);
            if num >= 24
                && liquid_merge_type != this_liquid_type
                && (!here.active() || td::TILE_OBSIDIAN_KILL.has(here.kind))
            {
                world.tiles[i].liquid = 0;
                self.create_liquid_merge_tile(
                    world,
                    x,
                    y,
                    this_liquid_type,
                    liquid_merge_type,
                    liquid_merge_tile_type,
                    create_merge_tiles_during_gen,
                );
            }
        } else {
            // The down arm (`:1300-1371`).
            let down = world.tile(x, y + 1).copied().unwrap_or_default();
            if down.liquid == 0 || tile_liquid_type(&down) == this_liquid_type {
                return;
            }
            let flag = here.active()
                && td::SET_IS_A_CONTAINER.has(here.kind)
                && !td::SET_IS_A_CONTAINER.has(down.kind);
            if this_liquid_type != 0 && crate::tileobject::tile_cut(down.kind) {
                crate::edit::kill_tile(world, x, y + 1, false);
                // `Main.netMode == 2` → `NetMessage.SendData(17, …)` (`:1309`)
                // — no multiplayer in the port.
            }
            if !((!down.active() || td::TILE_OBSIDIAN_KILL.has(down.kind)) || flag) {
                return;
            }
            let i = world.index(x, y);
            if here.liquid < 24 {
                world.tiles[i].liquid = 0;
                set_kind(&mut world.tiles[i], 0);
                // `NetMessage.SendTileSquare(-1, x - 1, y, 3)` (`:1322`) — no
                // multiplayer in the port.
            } else {
                let (liquid_merge_tile_type, liquid_merge_type) = get_liquid_merge_types(
                    this_liquid_type,
                    tile_liquid_type(&down) == 0,
                    tile_liquid_type(&down) == 1,
                    tile_liquid_type(&down) == 2,
                    tile_liquid_type(&down) == 3,
                );
                world.tiles[i].liquid = 0;
                let j = world.index(x, y + 1);
                world.tiles[j].liquid = 0;
                self.create_liquid_merge_tile(
                    world,
                    x,
                    y + 1,
                    this_liquid_type,
                    liquid_merge_type,
                    liquid_merge_tile_type,
                    create_merge_tiles_during_gen,
                );
            }
        }
    }

    /// `Liquid.CreateLiquidMergeTile` (`Liquid.cs:1330-1373`). The
    /// generation branch strips the row's liquid TYPE; the runtime branch
    /// lands the merge tile. `WorldGen.ReplaceTile` (`:1362`) is unported
    /// (the TileObjectData-anchor chain — see ANALYSIS), so the port takes
    /// the C#'s own fallback: `KillTile` + `PlaceTile(mute: true, forced:
    /// true)`. The `PlayLiquidChangeSound` (`:1346`, R-32-8's liquid-merge
    /// cues) and the MP tile-square messages have no port consumer; the
    /// `tileSolid[546]` swap (`:1360-1368`) guards the unported
    /// `ReplaceTile` path — the forced placement consults no solidity.
    fn create_liquid_merge_tile(
        &mut self,
        world: &mut World,
        x: i32,
        y: i32,
        this_liquid_type: u8,
        liquid_merge_type: u8,
        liquid_merge_tile_type: u16,
        create_merge_tiles_during_gen: bool,
    ) {
        // `WorldGen.GetLiquidChangeType` (`WorldGen.cs:4656-4684`) — the
        // change kind the merge reports. The live-game arm queues
        // `PlayLiquidChangeSound` (`Liquid.cs:1345-1347`'s
        // `!Main.gameMenu && !isGeneratingOrLoadingWorld` gate — the port's
        // settlement pass only runs in-game, so `!generating` is the gate);
        // the generation arm stays silent.
        let liquid_change_type = get_liquid_change_type(this_liquid_type, liquid_merge_type);
        if !self.generating {
            self.merge_events.push((liquid_change_type, x, y));
        }
        if self.generating && !create_merge_tiles_during_gen {
            let mut liquid_type: i32 = match liquid_merge_tile_type {
                229 => 0,
                230 => 1,
                56 => 1,
                659 => 3,
                _ => -1,
            };
            // `Main.UnderworldLayer` (`Main.cs:2790`, `maxTilesY - 200`).
            if y >= self.height - 200 {
                liquid_type = 1;
            }
            liquid_overwrite_strip(world, x, y, liquid_type);
        } else {
            let here = world.tile(x, y).copied().unwrap_or_default();
            if here.active() {
                crate::edit::kill_tile(world, x, y, false);
            }
            crate::edit::place_tile(world, x, y, liquid_merge_tile_type);
            // `WorldGen.SquareTileFrame(x, y)` (`:1366`) — the port's
            // `place_tile` already reframes the ring through `after_edit`.
        }
    }

    /// `WorldGen.UndergroundDesertCheck` (`Liquid.cs:1209-1223`): the
    /// desert's sandstone/hardened-sand walls (187/216) within the 7×7 box.
    fn underground_desert_check(&self, world: &World, x: i32, y: i32) -> bool {
        for i in (x - 3)..=(x + 3) {
            for j in (y - 3)..=(y + 3) {
                if i < 0
                    || j < 0
                    || i >= self.width
                    || j >= self.height
                {
                    continue;
                }
                let w = world.tiles[world.index(i, j)].wall;
                if w == 187 || w == 216 {
                    return true;
                }
            }
        }
        false
    }

    /// `WorldGen.SolidTile` (`WorldGen.cs:69957-69965`): nactive solid, not
    /// a solid-top, not a half-brick, slope 0.
    fn check_solid(&self, world: &World, x: i32, y: i32) -> bool {
        world.tile(x, y).is_some_and(|t| {
            t.active()
                && t.flags & crate::tile::FLAG_INACTIVE == 0
                && td::TILE_SOLID.has(t.kind)
                && !td::TILE_SOLID_TOP.has(t.kind)
                && !t.half_brick()
                && t.slope() == 0
        })
    }
}

/// `Tile.liquidType()`-style type getter — the two header bits, regardless
/// of the amount (`Tile.cs:344-434`).
fn tile_liquid_type(t: &Tile) -> u8 {
    kind_code(t.liquid_kind())
}

/// `Tile.anyWater()`/`anyLava()`/`anyHoney()`/`anyShimmer()` — the amount
/// check rides in front (`Tile.cs:401-434`).
fn any_is(t: &Tile, kind: u8) -> bool {
    t.liquid > 0 && tile_liquid_type(t) == kind
}

/// The `Liquid.cs:1233-1236` condition — a neighbour holding a liquid of a
/// different type.
fn any_other(t: &Tile, this_liquid_type: u8) -> bool {
    t.liquid > 0 && tile_liquid_type(t) != this_liquid_type
}

/// `Liquid.GetLiquidMergeTypes` (`Liquid.cs:1375-1439`): the merge target
/// per (this liquid, the nearby kinds) — the checks run in water → lava →
/// honey → shimmer order, so a later match wins. Returns
/// `(liquidMergeTileType, liquidMergeType)`.
fn get_liquid_merge_types(
    this_liquid_type: u8,
    water_nearby: bool,
    lava_nearby: bool,
    honey_nearby: bool,
    shimmer_nearby: bool,
) -> (u16, u8) {
    let mut liquid_merge_tile_type: u16 = 56;
    let mut liquid_merge_type: u8 = this_liquid_type;
    if this_liquid_type != 0 && water_nearby {
        liquid_merge_tile_type = match this_liquid_type {
            1 => 56,
            2 => 229,
            3 => 659,
            _ => liquid_merge_tile_type,
        };
        liquid_merge_type = 0;
    }
    if this_liquid_type != 1 && lava_nearby {
        liquid_merge_tile_type = match this_liquid_type {
            0 => 56,
            2 => 230,
            3 => 659,
            _ => liquid_merge_tile_type,
        };
        liquid_merge_type = 1;
    }
    if this_liquid_type != 2 && honey_nearby {
        liquid_merge_tile_type = match this_liquid_type {
            0 => 229,
            1 => 230,
            3 => 659,
            _ => liquid_merge_tile_type,
        };
        liquid_merge_type = 2;
    }
    if this_liquid_type != 3 && shimmer_nearby {
        liquid_merge_tile_type = 659;
        liquid_merge_type = 3;
    }
    (liquid_merge_tile_type, liquid_merge_type)
}

/// `WorldGen.GetLiquidChangeType` (`WorldGen.cs:4656-4684`) — the
/// `TileChangeType` the merge reports (the sound + MP consumers).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TileChangeType {
    None,
    LavaWater,
    HoneyWater,
    HoneyLava,
    ShimmerWater,
    ShimmerLava,
    ShimmerHoney,
}

fn get_liquid_change_type(liquid_type: u8, other_liquid_type: u8) -> TileChangeType {
    match (liquid_type, other_liquid_type) {
        (0, 1) | (1, 0) => TileChangeType::LavaWater,
        (0, 2) | (2, 0) => TileChangeType::HoneyWater,
        (1, 2) | (2, 1) => TileChangeType::HoneyLava,
        (0, 3) | (3, 0) => TileChangeType::ShimmerWater,
        (1, 3) | (3, 1) => TileChangeType::ShimmerLava,
        (2, 3) | (3, 2) => TileChangeType::ShimmerHoney,
        _ => TileChangeType::None,
    }
}

/// `Liquid.LiquidOverwriteStrip` (`Liquid.cs:1352-1361`): the generation
/// branch re-types the row's liquid cells left and right of the merge
/// point (the `x` cell is written by both walks, as in the C#).
fn liquid_overwrite_strip(world: &mut World, x: i32, y: i32, liquid_type: i32) {
    // `liquidType(int)`'s bit math (`Tile.cs`) keeps exactly the two low
    // bits — the C#'s `-1` default lands on the shimmer code.
    let code = (liquid_type & 3) as u8;
    let mut n = x;
    while n >= 0 && world.tile(n, y).is_some_and(|t| t.liquid > 0) {
        let idx = world.index(n, y);
        set_kind(&mut world.tiles[idx], code);
        n -= 1;
    }
    let mut i = x;
    while i < world.width && world.tile(i, y).is_some_and(|t| t.liquid > 0) {
        let idx = world.index(i, y);
        set_kind(&mut world.tiles[idx], code);
        i += 1;
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

/// `Main.tileSolid[type] && !Main.tileSolidTop[type]` on a `nactive()` tile —
/// `Tile.nactive()` = `active() && !inActive()` (`Tile.cs`), so an actuated
/// tile lets the liquid through (`Liquid.cs:171/:236/:457`).
#[inline]
fn is_solid(t: &Tile) -> bool {
    t.active()
        && t.flags & crate::tile::FLAG_INACTIVE == 0
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

    fn put_as(w: &mut World, x: i32, y: i32, amount: u8, kind: u8) {
        let i = w.index(x, y);
        w.tiles[i].liquid = amount;
        set_kind(&mut w.tiles[i], kind);
        w.mark_chunk(x, y);
    }

    #[test]
    fn water_and_lava_merge_into_obsidian_at_the_lava_cell() {
        // `Liquid.cs:1249-1330`: the lava cell's check pools the
        // different-type side/up neighbours (zeroing them) and at
        // `num >= 24` lands tile 56 on the lava cell — both liquids
        // consumed.
        let mut w = basin(9, 8);
        put_as(&mut w, 4, 4, 30, 1);
        put_as(&mut w, 5, 4, 30, 0);
        let mut sys = LiquidSystem::new(&w);
        sys.add_water(&w, 4, 4);
        sys.add_water(&w, 5, 4);
        run(&mut sys, &mut w, 1);
        assert_eq!(w.tile(4, 4).unwrap().kind, 56, "the lava cell became obsidian");
        assert_eq!(w.tile(4, 4).unwrap().liquid, 0);
        assert_eq!(w.tile(5, 4).unwrap().liquid, 0, "the pooled water is consumed");
    }

    #[test]
    fn water_and_honey_merge_into_a_honey_block() {
        let mut w = basin(9, 8);
        put_as(&mut w, 4, 4, 30, 2);
        put_as(&mut w, 5, 4, 30, 0);
        let mut sys = LiquidSystem::new(&w);
        sys.add_water(&w, 4, 4);
        sys.add_water(&w, 5, 4);
        run(&mut sys, &mut w, 1);
        assert_eq!(w.tile(4, 4).unwrap().kind, 229);
        assert_eq!(w.tile(5, 4).unwrap().liquid, 0);
    }

    #[test]
    fn honey_and_lava_merge_into_crisp_honey_once() {
        // The honey cell checks first (`:1375-1439`'s lava arm → 230); the
        // merge tile is not `tileObsidianKill`, so the lava cell's own check
        // afterwards finds an active non-kill tile and refuses — one merge.
        let mut w = basin(9, 8);
        put_as(&mut w, 4, 4, 30, 2);
        put_as(&mut w, 5, 4, 30, 1);
        let mut sys = LiquidSystem::new(&w);
        sys.add_water(&w, 4, 4);
        sys.add_water(&w, 5, 4);
        run(&mut sys, &mut w, 1);
        assert_eq!(w.tile(4, 4).unwrap().kind, 230);
        assert_eq!(w.tile(5, 4).unwrap().liquid, 0);
        assert_eq!(w.tile(5, 4).unwrap().kind, 0, "no second merge tile");
    }

    #[test]
    fn shimmer_consumes_the_water_into_a_shimmer_block() {
        let mut w = basin(9, 8);
        put_as(&mut w, 4, 4, 30, 3);
        put_as(&mut w, 5, 4, 30, 0);
        let mut sys = LiquidSystem::new(&w);
        sys.add_water(&w, 4, 4);
        sys.add_water(&w, 5, 4);
        run(&mut sys, &mut w, 1);
        assert_eq!(w.tile(4, 4).unwrap().kind, 659);
        assert_eq!(w.tile(5, 4).unwrap().liquid, 0);
    }

    #[test]
    fn the_merge_gate_needs_24_units() {
        // `Liquid.cs:1325`: `num >= 24` — below it the pooled neighbours
        // stay zeroed (the C#'s own consumption) and the checking cell
        // keeps its liquid; the lava flow then sits in the delay gate
        // (`:484-490`).
        let mut w = basin(9, 8);
        put_as(&mut w, 4, 4, 30, 1);
        put_as(&mut w, 5, 4, 20, 0);
        let mut sys = LiquidSystem::new(&w);
        sys.add_water(&w, 4, 4);
        sys.add_water(&w, 5, 4);
        run(&mut sys, &mut w, 1);
        assert_eq!(w.tile(4, 4).unwrap().kind, 0, "no obsidian");
        assert_eq!(w.tile(4, 4).unwrap().liquid, 30, "the lava stays");
        assert_eq!(w.tile(5, 4).unwrap().liquid, 0, "the pool is consumed");
    }

    #[test]
    fn the_down_arm_merges_below_the_lava_cell() {
        // `Liquid.cs:1300-1371`: no different-type side/up neighbour, the
        // cell below holds a different liquid >= 24 → the merge lands at
        // `(x, y+1)`.
        let mut w = basin(9, 8);
        put_as(&mut w, 4, 4, 30, 1);
        put_as(&mut w, 4, 5, 30, 0);
        let mut sys = LiquidSystem::new(&w);
        sys.add_water(&w, 4, 4);
        sys.add_water(&w, 4, 5);
        run(&mut sys, &mut w, 1);
        assert_eq!(w.tile(4, 5).unwrap().kind, 56);
        assert_eq!(w.tile(4, 4).unwrap().liquid, 0);
        assert_eq!(w.tile(4, 5).unwrap().liquid, 0);
    }

    #[test]
    fn lava_burns_the_submerged_cut_plant_below() {
        // `Liquid.cs:1305-1312`: a non-water liquid burns the `tileCut`
        // plant below before the merge gate reads it.
        let mut w = basin(9, 8);
        let i = w.index(4, 5);
        w.tiles[i] = Tile {
            kind: 3,
            ..Default::default()
        };
        w.tiles[i].flags |= FLAG_ACTIVE;
        w.tiles[i].liquid = 30;
        put_as(&mut w, 4, 4, 30, 1);
        let mut sys = LiquidSystem::new(&w);
        sys.add_water(&w, 4, 4);
        run(&mut sys, &mut w, 1);
        assert_eq!(w.tile(4, 5).unwrap().kind, 56, "the plant burned, the merge landed");
        assert_eq!(w.tile(4, 4).unwrap().liquid, 0);
    }

    #[test]
    fn get_liquid_merge_types_follows_the_cs_priority() {
        // `Liquid.cs:1375-1439`: water → lava → honey → shimmer, a later
        // match wins; the own-type check never fires.
        assert_eq!(get_liquid_merge_types(1, true, false, false, false), (56, 0));
        assert_eq!(get_liquid_merge_types(2, true, false, false, false), (229, 0));
        assert_eq!(get_liquid_merge_types(2, false, true, false, false), (230, 1));
        assert_eq!(get_liquid_merge_types(1, false, false, true, false), (230, 2));
        assert_eq!(get_liquid_merge_types(0, false, true, false, false), (56, 1));
        assert_eq!(get_liquid_merge_types(0, false, false, true, false), (229, 2));
        assert_eq!(get_liquid_merge_types(0, false, false, false, true), (659, 3));
        assert_eq!(get_liquid_merge_types(1, false, false, false, true), (659, 3));
        assert_eq!(get_liquid_merge_types(2, false, false, false, true), (659, 3));
        // The honey match wins over the water one — and a shimmer cell
        // always merges into the shimmer block (its own arm is skipped:
        // `thisLiquidType != 3` fails).
        assert_eq!(get_liquid_merge_types(3, true, false, true, false), (659, 2));
        // The own type never merges.
        assert_eq!(get_liquid_merge_types(0, true, false, false, false), (56, 0));
    }

    #[test]
    fn get_liquid_change_type_maps_the_pairs() {
        // `WorldGen.cs:4656-4684`.
        assert_eq!(get_liquid_change_type(0, 1), TileChangeType::LavaWater);
        assert_eq!(get_liquid_change_type(0, 2), TileChangeType::HoneyWater);
        assert_eq!(get_liquid_change_type(1, 2), TileChangeType::HoneyLava);
        assert_eq!(get_liquid_change_type(0, 3), TileChangeType::ShimmerWater);
        assert_eq!(get_liquid_change_type(1, 3), TileChangeType::ShimmerLava);
        assert_eq!(get_liquid_change_type(2, 3), TileChangeType::ShimmerHoney);
        assert_eq!(get_liquid_change_type(0, 0), TileChangeType::None);
    }

    #[test]
    fn the_container_flag_forces_the_down_merge() {
        // `Liquid.cs:1304` + `:1314`: a container (chest 21 / shell pile 467
        // / bed 88) above lets the merge proceed into an active
        // non-`tileObsidianKill` cell — stone here.
        let mut w = basin(9, 8);
        let chest = w.index(4, 4);
        w.tiles[chest] = Tile {
            kind: 21,
            ..Default::default()
        };
        w.tiles[chest].flags |= FLAG_ACTIVE;
        w.tiles[chest].liquid = 30;
        let stone = w.index(4, 5);
        w.tiles[stone] = Tile {
            kind: 1,
            ..Default::default()
        };
        w.tiles[stone].flags |= FLAG_ACTIVE;
        w.tiles[stone].liquid = 30;
        set_kind(&mut w.tiles[stone], 1);
        let mut sys = LiquidSystem::new(&w);
        sys.liquid_check(&mut w, 4, 4, 0, false);
        assert_eq!(w.tile(4, 5).unwrap().kind, 56, "the stone cell was merged into");
        assert_eq!(w.tile(4, 4).unwrap().liquid, 0);
    }

    #[test]
    fn the_live_merge_queues_its_sound_and_generation_stays_silent() {
        // `Liquid.cs:1345-1347`: the runtime branch plays
        // `PlayLiquidChangeSound(liquidChangeType, x, y)` — the same
        // container-merge fixture as above reports
        // `(LavaWater, 4, 5)` once, then the queue drains empty; the
        // generation arm (`:1333`'s strip branch) queues nothing.
        let mut w = basin(9, 8);
        let chest = w.index(4, 4);
        w.tiles[chest] = Tile {
            kind: 21,
            ..Default::default()
        };
        w.tiles[chest].flags |= FLAG_ACTIVE;
        w.tiles[chest].liquid = 30;
        let stone = w.index(4, 5);
        w.tiles[stone] = Tile {
            kind: 1,
            ..Default::default()
        };
        w.tiles[stone].flags |= FLAG_ACTIVE;
        w.tiles[stone].liquid = 30;
        set_kind(&mut w.tiles[stone], 1);
        let mut sys = LiquidSystem::new(&w);
        sys.liquid_check(&mut w, 4, 4, 0, false);
        assert_eq!(
            sys.drain_merge_events(),
            vec![(TileChangeType::LavaWater, 4, 5)],
            "the live merge reports its liquid-change sound"
        );
        assert!(sys.drain_merge_events().is_empty(), "drained");

        // The generation arm: `isGeneratingOrLoadingWorld` blocks the sound
        // (`Liquid.cs:1345`'s gate) — the same merge queues nothing.
        let mut sys = LiquidSystem::new(&w);
        sys.generating = true;
        sys.liquid_check(&mut w, 4, 4, 0, false);
        assert!(
            sys.drain_merge_events().is_empty(),
            "generation never queues a liquid-change sound"
        );
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

    /// A full-width row of equal levels is static: every `update_tile` walk returns
    /// an unchanged cell, so `kill` grows by exactly one per walk and the entry
    /// retires once `kill` reaches the threshold. That turns the *window* into a
    /// directly observable quantity.
    fn static_row(width: i32) -> (World, LiquidSystem) {
        let mut w = basin(width, 6);
        for x in 0..width {
            put(&mut w, x, 5, 100);
        }
        let mut sys = LiquidSystem::new(&w);
        for x in 0..width {
            sys.add_water(&w, x, 5);
        }
        (w, sys)
    }

    #[test]
    fn a_short_active_list_is_walked_in_full_every_update() {
        // `Liquid.cs:1077-1082`: when `numLiquid <= curMaxLiquid/cycles` the end of
        // the window is clamped to `numLiquid` and `wetCounter` jumps to `cycles`,
        // so a short list is simulated — and its dead entries retired — on *every*
        // call, not once per ten as the old `ceil(len/10)` slice did.
        let (mut w, mut sys) = static_row(12);
        assert_eq!(sys.active_len(), 12);
        run(&mut sys, &mut w, 8);
        assert_eq!(
            sys.active_len(),
            0,
            "a short list must retire on pass 8, i.e. be walked every update"
        );
    }

    #[test]
    fn a_long_active_list_walks_one_stride_per_update() {
        // `Liquid.cs:1070-1082`: the window is a fixed `curMaxLiquid / cycles`
        // stride (2 500). A 2 600-entry list therefore walks `[0,2500)` on the odd
        // updates and the clamped tail `[2500,2600)` on the even ones, so both
        // strides reach the eight-pass retire threshold on the 16th update. The old
        // `ceil(len/10)` slice would need ~80 updates for the same.
        let width = 2600;
        let (mut w, mut sys) = static_row(width);
        assert_eq!(sys.active_len(), width as usize);
        run(&mut sys, &mut w, 15);
        assert_eq!(
            sys.active_len(),
            width as usize,
            "nothing retires before the eighth pass of both strides"
        );
        run(&mut sys, &mut w, 1);
        assert_eq!(
            sys.active_len(),
            0,
            "the 2 500 stride retires on pass 16, not the 80th an `ceil(len/10)` slice needs"
        );
    }

    #[test]
    fn the_actuated_floor_lets_water_through() {
        // `Liquid`'s solid tests read `nactive()` (`Liquid.cs:171/:236/:457`
        // — `active && !inActive`): an actuated basin floor seeps.
        let mut w = basin(9, 8);
        let i = w.index(4, 8);
        w.tiles[i].flags |= crate::tile::FLAG_INACTIVE;
        w.frame_important = vec![false; td::TILE_COUNT];
        put(&mut w, 4, 4, 255);
        let mut sys = LiquidSystem::new(&w);
        sys.add_water(&w, 4, 4);
        run(&mut sys, &mut w, 400);
        assert!(
            w.tile(4, 8).unwrap().liquid > 0,
            "water entered the actuated floor tile"
        );
    }
}
