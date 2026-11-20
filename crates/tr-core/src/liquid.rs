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
    /// One bit per tile: `Tile.skipLiquid` (`Tile.cs:511-521`).
    skip: Vec<u64>,
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
    /// `Liquid.tilesIgnoreWater`'s state (`Liquid.cs:69-73`): during a pass
    /// the grate 546 and the boulder family lose solidity, so water falls
    /// through grates and settles past boulders; the C#'s stock
    /// `Main.tileSolid[546] = true` (`Main.cs:10124`) is restored at the
    /// pass tail (`Liquid.cs:1157`). `UpdateLiquid` wraps the whole runtime
    /// pass (`:986`/`:1157`); the gen-only sites (`SettleWaterAt` `:151`/
    /// `:288`, `QuickWater` `:115`/`:135`, `WorldGen.cs:34767`/`:80417`)
    /// stay with the unported gen passes.
    ignore_solids: bool,
    /// `Main.tileSolid[379]` (`Liquid.cs:451` vs `Main.cs:17753`/`:17799`):
    /// the bubble is solid for the liquid pass itself and not solid for the
    /// entity half of the tick — the C# clears it in
    /// `DoUpdateInWorld_Inner` (`:17753`, before the players) and the
    /// runtime `UpdateLiquid` re-arms it per cell. The app layer clears the
    /// flag at the tick head (the `:17753` seam).
    bubble_solid: bool,
    /// `Liquid.stuck` (`Liquid.cs:31`): up only inside the stuck-count
    /// force-clear (`:1136-1141`) — `DelWater`'s keep-alive branch reads it
    /// (`:1506`), so the clear drains every entry.
    stuck: bool,
    /// `Liquid.stuckCount` (`Liquid.cs:21`): consecutive cycle-ends whose
    /// active count stayed within ±50 of [`Self::stuck_amount`]; 10 000
    /// force-clears the list (`Liquid.cs:1131-1143`).
    stuck_count: u32,
    /// `Liquid.stuckAmount` (`Liquid.cs:23`): the active count the stuck
    /// window compares against, re-pinned whenever the count leaves it.
    stuck_amount: i64,
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
            skip: vec![0; bits],
            buffer: Vec::new(),
            wet_counter: 0,
            cur_max_liquid: MAX_ACTIVE,
            cycles: CYCLES,
            dirty: false,
            quick_fall: false,
            generating: false,
            remix_world_gen: false,
            merge_events: Vec::new(),
            // The stock solidity: the grate and the boulders are solid and
            // the bubble is not (`Main.cs:10124`, `Main.cs:17753`).
            ignore_solids: false,
            bubble_solid: false,
            // `Liquid.ReInit` (`Liquid.cs:83-97`): the stuck family starts
            // zeroed; `stuckAmount` pins to the first cycle-end's count.
            stuck: false,
            stuck_count: 0,
            stuck_amount: 0,
        }
    }

    pub fn active_len(&self) -> usize {
        self.active.len()
    }

    /// `Main.cs:17753` — `Main.tileSolid[379] = false` at the world-tick
    /// head (after `UpdateParticleSystems_World`, before the players): the
    /// bubble is not solid for the entity half of the tick. The runtime
    /// `UpdateLiquid` re-arms it ([`Self::update`], `Liquid.cs:451`).
    pub fn clear_bubble_for_entities(&mut self) {
        self.bubble_solid = false;
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
    /// `Tile.skipLiquid` (`Tile.cs:511-521`): the flag the pass leaves on the
    /// two cells a down-transfer just touched, so the same stride does not
    /// re-run them (`Liquid.cs:1096-1104`).
    #[inline]
    fn is_skip(&self, x: i32, y: i32) -> bool {
        let i = self.bit(x, y);
        (self.skip[i >> 6] >> (i & 63)) & 1 != 0
    }

    #[inline]
    fn set_skip(&mut self, x: i32, y: i32, v: bool) {
        let i = self.bit(x, y);
        if v {
            self.skip[i >> 6] |= 1 << (i & 63);
        } else {
            self.skip[i >> 6] &= !(1 << (i & 63));
        }
    }

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
        if t.liquid == 0 || self.tile_is_solid(t) {
            return;
        }
        // `Liquid.cs:1167-1171` — the active list is full: overflow into the
        // `LiquidBuffer`. `LiquidBuffer.AddBuffer` (`LiquidBuffer.cs:11-20`)
        // caps at `maxLiquidBuffer - 2` and marks the cell `checkingLiquid`,
        // so a buffered cell is not re-buffered and `DelBuffer` + the
        // promotion's `checkingLiquid(false)` (`Liquid.cs:1127`) release it.
        if self.active.len() >= self.cur_max_liquid - 1 {
            if self.buffer.len() < MAX_BUFFER - 2 && !self.is_checking(x, y) {
                self.set_checking(x, y, true);
                self.buffer.push((x, y));
            }
            return;
        }
        self.set_checking(x, y, true);
        self.set_skip(x, y, false);
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
        // `Liquid.UpdateLiquid`'s scope (`Liquid.cs:986`/`:1157`): the grate
        // and the boulders lose solidity for the whole pass, and the bubble
        // gains it (`Liquid.Update`'s `Main.tileSolid[379] = true`, `:451` —
        // the C# re-arms it per cell; nothing clears it inside the pass, so
        // one arm here is the same state).
        self.ignore_solids = true;
        self.bubble_solid = true;
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
            // `Liquid.cs:1083-1105` — the original's two loops: quickFall
            // forces every entry's delay to 10 (`:1086-1089`) and clears the
            // skip after the update; the normal pass skips a cell the same
            // stride already touched and clears its flag (`:1096-1104`).
            if !self.quick_fall && self.is_skip(x, y) {
                self.set_skip(x, y, false);
                continue;
            }
            let delay = if self.quick_fall {
                10
            } else {
                self.active[i].delay
            };
            let (kill, delay) = self.update_tile(world, x, y, self.active[i].kill, delay);
            self.active[i].kill = kill;
            self.active[i].delay = delay;
            if self.quick_fall {
                self.set_skip(x, y, false);
            }
        }
        if self.wet_counter >= self.cycles {
            self.wet_counter = 0;
            self.end_pass(world);
        }
        if self.dirty {
            world.revision = world.revision.wrapping_add(1);
        }
        // `Liquid.tilesIgnoreWater(ignoreSolids: false)` at the pass tail
        // (`Liquid.cs:1157`) — the grate/boulder solidity is the stock one
        // again until the next pass. The bubble stays solid until the app's
        // `Main.cs:17753` seam (the tick head, before the players).
        self.ignore_solids = false;
    }

    /// `wetCounter >= cycles` bookkeeping (`Liquid.cs:1106-1150`): the
    /// `kill >= 8` retirement sweep (the 254→255 promotion, then `DelWater`'s
    /// swap-remove — which copies `x`/`y`/`kill` but NOT `delay`, `:1577-1579`,
    /// so the moved entry inherits the retired slot's throttle), the buffer
    /// promotion and the stuck-count window.
    fn end_pass(&mut self, world: &mut World) {
        // `Liquid.cs:1109-1119` — the `kill >= 8` retirement sweep.
        self.retire_sweep(world, false);
        self.refill_from_buffer(world);
        // `Liquid.cs:1131-1149` — the stuck window: an active count that
        // stays within ±50 of `stuckAmount` for 10 000 cycle-ends force-
        // clears the whole list through `DelWater` (with `stuck` up, so the
        // keep-alive branch of `:1506` cannot rescue an entry).
        let n = self.active.len() as i64;
        if n > 0 && n > self.stuck_amount - 50 && n < self.stuck_amount + 50 {
            self.stuck_count += 1;
            if self.stuck_count >= 10_000 {
                self.stuck = true;
                // `:1137-1140` — the same reverse sweep, unconditional.
                self.retire_sweep(world, true);
                self.stuck = false;
                self.stuck_count = 0;
            }
        } else {
            self.stuck_count = 0;
            self.stuck_amount = n;
        }
    }

    /// The reverse `DelWater` sweep (`Liquid.cs:1109-1119` / `:1137-1140`).
    /// `force` skips the `kill >= 8` gate (the stuck-clear). A retired slot
    /// is filled by the LAST entry, whose own `kill` the re-read then
    /// re-tests — a dead tail cascades down exactly as the C#'s descending
    /// `l` does. The swap copies `x`/`y`/`kill` but NOT `delay` (`:1577-1579`):
    /// the slot keeps its own delay field, so the moved entry inherits the
    /// retired cell's throttle counter.
    fn retire_sweep(&mut self, world: &mut World, force: bool) {
        let mut l = self.active.len();
        while l > 0 {
            l -= 1;
            if !force && self.active[l].kill < KILL_PASSES {
                continue;
            }
            let (x, y) = (self.active[l].x, self.active[l].y);
            if !force {
                // `:1113-1116` — a 254 cell the kill sweep catches is topped
                // up to 255 before the retirement probe reads it.
                let i = world.index(x, y);
                if world.tiles[i].liquid == 254 {
                    world.tiles[i].liquid = 255;
                    world.mark_chunk(x, y);
                    self.dirty = true;
                }
            }
            if self.del_water(world, l) {
                let last = self.active.pop().expect("l < len");
                if l < self.active.len() {
                    self.active[l].x = last.x;
                    self.active[l].y = last.y;
                    self.active[l].kill = last.kill;
                }
            }
        }
    }

    /// `Liquid.DelWater` (`Liquid.cs:1470-1599`): the retirement probe for one
    /// active slot. Returns `true` when the entry retires (the caller then
    /// swap-removes it, copying `x`/`y`/`kill` but NOT `delay`, `:1577-1579`),
    /// `false` on the keep-alive early return (`:1506-1510`, `kill = 0`). The
    /// C#'s `numLiquid--` + `checkingLiquid(false)` + the swap live in the
    /// caller's sweep; the `checkingLiquid` clear and the alch/lily-pad tail
    /// run here on the retired cell's coordinates.
    fn del_water(&mut self, world: &mut World, l: usize) -> bool {
        let (x, y) = (self.active[l].x, self.active[l].y);
        let here_i = world.index(x, y);
        // `:1478-1498` — a sub-2 cell dries out and wakes/dries its sides.
        if world.tiles[here_i].liquid < 2 {
            world.tiles[here_i].liquid = 0;
            for nx in [x - 1, x + 1] {
                // The C#'s `Main.tile[nx, y]` is null at the world edge; the
                // port's `index` would wrap, so probe through `tile` first.
                let Some(j) = world.tile(nx, y).map(|_| world.index(nx, y)) else {
                    continue;
                };
                if world.tiles[j].liquid < 2 {
                    world.tiles[j].liquid = 0;
                } else {
                    self.add_water(world, nx, y);
                }
            }
        } else if world.tiles[here_i].liquid < 20 {
            // `:1499-1505` — a sub-20 cell drains if a lower side or an open
            // below can take it.
            let here = world.tiles[here_i].liquid;
            let left = world.tile(x - 1, y).copied().unwrap_or_default();
            let right = world.tile(x + 1, y).copied().unwrap_or_default();
            let below = world.tile(x, y + 1).copied().unwrap_or_default();
            if (left.liquid < here && !self.tile_is_solid(&left))
                || (right.liquid < here && !self.tile_is_solid(&right))
                || (below.liquid < 255 && !self.tile_is_solid(&below))
            {
                world.tiles[here_i].liquid = 0;
            }
        } else {
            // `:1506-1510` — the keep-alive: an open below that is not full,
            // not stuck, and an open here rescues the entry (`kill = 0`).
            let below = world.tile(x, y + 1).copied().unwrap_or_default();
            let here = world.tiles[here_i];
            if below.liquid < 255
                && !self.tile_is_solid(&below)
                && !self.stuck
                && !self.tile_is_solid(&here)
            {
                self.active[l].kill = 0;
                return false;
            }
        }
        // `:1511-1514` — a non-full cell under a liquid above wakes the above.
        let above = world.tile(x, y - 1).copied().unwrap_or_default();
        if world.tiles[here_i].liquid < 250 && above.liquid > 0 {
            self.add_water(world, x, y - 1);
        }
        if world.tiles[here_i].liquid == 0 {
            // `:1515-1518` — a dried cell reverts to the water type.
            set_kind(&mut world.tiles[here_i], 0);
        } else {
            // `:1519-1528` — wake an unequal, non-full, open side.
            let here = world.tiles[here_i].liquid;
            for nx in [x + 1, x - 1] {
                let side = world.tile(nx, y).copied().unwrap_or_default();
                if side.liquid > 0
                    && side.liquid < 250
                    && !self.tile_is_solid(&side)
                    && here != side.liquid
                {
                    self.add_water(world, nx, y);
                }
            }
            // `:1529-1569` — the type reaction on the retired cell, plus the
            // lava grass-burn ring.
            let kind = kind_code(world.tiles[here_i].liquid_kind());
            if kind == 1 {
                self.lava_check(world, x, y);
                // `:1532-1560` — the 3×3 ring: grass family → dirt 0, the
                // mud/ash family → 59, each reframed.
                for i in (x - 1)..=(x + 1) {
                    for j in (y - 1)..=(y + 1) {
                        let Some(t) = world.tile(i, j).copied() else {
                            continue;
                        };
                        if !t.active() {
                            continue;
                        }
                        let to = match t.kind {
                            2 | 23 | 109 | 199 | 477 | 492 => Some(0u16),
                            60 | 70 | 661 | 662 => Some(59u16),
                            _ => None,
                        };
                        if let Some(to) = to {
                            let idx = world.index(i, j);
                            world.tiles[idx].kind = to;
                            world.mark_chunk(i, j);
                            self.dirty = true;
                            // `WorldGen.SquareTileFrame(i, j)` (resetFrame:
                            // true) — the drops ride the discarded return (the
                            // engine is pure).
                            crate::edit::square_tile_frame_default(world, i, j);
                        }
                    }
                }
            } else if kind == 2 {
                self.honey_check(world, x, y);
            } else if kind == 3 {
                self.shimmer_check(world, x, y);
            }
        }
        // `:1576` — the retired cell's `checkingLiquid` clears (the caller's
        // swap then overwrites the slot).
        self.set_checking(x, y, false);
        // `:1580-1598` — the alch/lily-pad tail reads the LIVE kind (the merge
        // above may have rewritten the cell).
        let kind_now = world.tiles[here_i].kind;
        if crate::tileobject::tile_alch(kind_now) {
            crate::edit::check_alch_tile(world, x, y);
        } else if kind_now == 518 {
            if self.quick_fall {
                crate::edit::check_lily_pad(world, x, y);
            } else {
                let below = world.tile(x, y + 1).copied().unwrap_or_default();
                let above = world.tile(x, y - 1).copied().unwrap_or_default();
                if below.liquid < 255 || above.liquid > 0 {
                    crate::edit::square_tile_frame_default(world, x, y);
                } else {
                    crate::edit::check_lily_pad(world, x, y);
                }
            }
        }
        true
    }

    fn refill_from_buffer(&mut self, world: &World) {
        // `Liquid.cs:1120-1130` — `curMaxLiquid - (curMaxLiquid - numLiquid)`
        // is `numLiquid`: promote up to the active count, capped by the
        // buffer. Each promotion clears the buffer entry's `checkingLiquid`
        // FIRST (`:1127`), then `AddWater` re-queues it (fresh `delay = 0`,
        // `:1177`); `DelBuffer(0)` swap-removes the slot (`LiquidBuffer.cs:22-27`).
        let limit = self.active.len();
        let mut promoted = 0;
        while promoted < limit
            && !self.buffer.is_empty()
            && self.active.len() < self.cur_max_liquid - 1
        {
            let (x, y) = self.buffer[0];
            self.set_checking(x, y, false);
            self.add_water(world, x, y);
            let last = self.buffer.pop().expect("non-empty");
            if !self.buffer.is_empty() {
                self.buffer[0] = last;
            }
            promoted += 1;
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
        // `Liquid.cs:457-463` — the solid kill precedes everything (the
        // delay is untouched on this early return).
        if self.tile_is_solid(&start) {
            return (KILL_DEAD, delay);
        }
        // `Liquid.cs:466-474` — underworld evaporation: below
        // `Main.UnderworldLayer` (`Main.cs:2790`, `maxTilesY - 200`) water
        // loses `min(2, liquid)` per visit, before the zero kill of `:475`.
        // Water only — lava/honey/shimmer are exempt. The C# writes the
        // amount straight to the tile; the port marks the chunk + `dirty`
        // like `set_level` so the render cache sees the change (the C#'s
        // own change consumer, `_netChangeSet`, is MP and unported). The
        // `underworld_layer >= 0` guard keeps the band non-negative:
        // vanilla worlds are >= 1200 tiles tall so the layer always
        // exists; a sub-200 fixture world would put the C# layer below
        // row 0 and evaporate every cell — the ported water witnesses
        // pin the flow there (ANALYSIS "The liquid reactions", W4-c34).
        let underworld_layer = self.height - 200;
        if underworld_layer >= 0
            && y > underworld_layer
            && start.liquid_kind() == LiquidKind::Water
            && start.liquid > 0
        {
            world.tiles[i].liquid -= start.liquid.min(2);
            world.mark_chunk(x, y);
            self.dirty = true;
        }
        // `Liquid.cs:475-479` — the zero kill reads the post-evaporation
        // amount; a cell that just evaporated dry retires immediately.
        if world.tiles[i].liquid == 0 {
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
            if !self.tile_is_solid(&below)
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
                // `Liquid.cs:586-587` — `tile4.skipLiquid(true)` and
                // `tile5.skipLiquid(true)`: the same stride skips both.
                self.set_skip(x, y + 1, true);
                self.set_skip(x, y, true);
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
                    !self.tile_is_solid(&t) && t.liquid > 0 && kind_code(t.liquid_kind()) == kind
                })
            };
            let open = |t: Option<Tile>| {
                t.is_some_and(|t| {
                    !self.tile_is_solid(&t) && (t.liquid == 0 || kind_code(t.liquid_kind()) == kind)
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
                    // `Liquid.cs:748-771` — the extra wake pass, run BEFORE the
                    // conditional centre write: re-queue each neighbour when
                    // its (post-`set_level`) level OR the centre's level differs
                    // from the average. The neighbours all hold `avg` now, so
                    // the clause reduces to the centre-diff test; the centre
                    // still holds `own` here (the C# reads the pre-write tile5).
                    if own as i32 != avg {
                        for nx in [x - 1, x + 1, x - 2, x + 2, x - 3, x + 3] {
                            self.add_water(world, nx, y);
                        }
                    }
                    if equal < 6 || !above_full {
                        new_own = avg;
                    }
                } else if f4 && f5 {
                    let avg = round_level(l + r + l2 + r2 + own as f32 + bias, 5);
                    let equal = set_level(world, self, x - 1, y, avg, kind)
                        + set_level(world, self, x + 1, y, avg, kind)
                        + set_level(world, self, x - 2, y, avg, kind)
                        + set_level(world, self, x + 2, y, avg, kind);
                    // `Liquid.cs:821-836` — the 5-neighbour extra wake pass,
                    // the same centre-diff reduction as the 7-arm above.
                    if own as i32 != avg {
                        for nx in [x - 1, x + 1, x - 2, x + 2] {
                            self.add_water(world, nx, y);
                        }
                    }
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
                    let mut avg = round_level(l + r + own as f32 + bias, 3);
                    // `Liquid.cs:892-895` — the 254→255 nudge: a settled 254
                    // surface rolls `WorldGen.genRand.Next(30)` and promotes
                    // to 255 on a zero. The draw rides the port's `genRand`
                    // stand-in (`World.frame_rng`, the same stream the
                    // SquareTileFrame centre re-roll consumes — ANALYSIS
                    // "The liquid delay machinery", W4-g45); it advances the
                    // shared stream exactly when the C# does (only on a 254).
                    if avg == 254 && world.frame_rng.next_max(30) == 0 {
                        avg = 255;
                    }
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

/// `WorldGen.SetBoulderSolidity`'s tile set (`WorldGen.cs:11009-11019`) —
/// the nine boulder kinds the liquid pass strips (`Liquid.tilesIgnoreWater`
/// calls it alongside the grate 546).
const BOULDER_SOLIDS: [u16; 9] = [138, 484, 664, 711, 712, 713, 714, 715, 716];

impl LiquidSystem {
    /// `Main.tileSolid[type] && !Main.tileSolidTop[type]` on a `nactive()`
    /// tile — `Tile.nactive()` = `active() && !inActive()` (`Tile.cs`), so an
    /// actuated tile lets the liquid through (`Liquid.cs:171/:236/:457`) —
    /// plus the pass-scoped toggles (S1-L-14): the grate 546 and the boulder
    /// family are non-solid while [`Self::ignore_solids`] is up
    /// (`Liquid.tilesIgnoreWater`, `Liquid.cs:69-73`), and the bubble 379 is
    /// solid exactly while [`Self::bubble_solid`] is up (`Liquid.cs:451` vs
    /// `Main.cs:17753`).
    #[inline]
    fn tile_is_solid(&self, t: &Tile) -> bool {
        if !t.active() || t.flags & crate::tile::FLAG_INACTIVE != 0 {
            return false;
        }
        let kind = t.kind;
        if kind as usize >= td::TILE_COUNT {
            return false;
        }
        if self.ignore_solids && (kind == 546 || BOULDER_SOLIDS.contains(&kind)) {
            return false;
        }
        if kind == 379 {
            return self.bubble_solid;
        }
        td::TILE_SOLID.has(kind) && !td::TILE_SOLID_TOP.has(kind)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tile::FLAG_ACTIVE;

    #[test]
    fn the_water_falls_through_the_grate_during_the_pass() {
        // S1-L-14 (`Liquid.cs:69-73`/`:986`/`:1157`): the runtime pass lifts
        // the grate 546's solidity (`Main.tileSolid[546] = !ignoreSolids`,
        // the stock `true` at `Main.cs:10124`), so water above a grate
        // lands under it.
        let mut w = World::new("liquid".into(), 9, 11);
        // A grate shelf at y=5, water above it at y=4.
        for x in 3..6 {
            let i = w.index(x, 5);
            w.tiles[i] = Tile { kind: 546, ..Default::default() };
            w.tiles[i].flags |= FLAG_ACTIVE;
            let j = w.index(x, 4);
            w.tiles[j].liquid = 255;
            w.mark_chunk(x, 4);
        }
        w.frame_important = vec![false; td::TILE_COUNT];
        w.compute_column_surface();
        w.compute_chunks();
        let mut sys = LiquidSystem::new(&w);
        sys.add_water(&w, 4, 4);
        // The water delay gate moves a cell every ~6 ticks (`Liquid.cs:484`).
        for _ in 0..40 {
            sys.update(&mut w);
        }
        let above: i32 = (3..6).map(|x| w.tiles[w.index(x, 4)].liquid as i32).sum();
        let mut below: i64 = 0;
        for x in 0..9 {
            for y in 6..11 {
                below += w.tiles[w.index(x, y)].liquid as i64;
            }
        }
        let below = below as i32;
        assert!(above < 255, "the water left the shelf ({above} left)");
        assert!(below > 0, "the water fell through the grate ({below})");
    }

    #[test]
    fn the_bubble_blocks_the_liquid_pass_and_releases_for_entities() {
        // S1-L-14 (`Liquid.cs:451` vs `Main.cs:17753`): the bubble 379 is
        // solid for the liquid pass itself — the water above it stays put —
        // and `clear_bubble_for_entities` returns the stock non-solid state
        // for the entity half of the tick.
        let mut w = World::new("liquid".into(), 9, 11);
        for x in 3..6 {
            let i = w.index(x, 5);
            w.tiles[i] = Tile { kind: 379, ..Default::default() };
            w.tiles[i].flags |= FLAG_ACTIVE;
            let j = w.index(x, 4);
            w.tiles[j].liquid = 255;
            w.mark_chunk(x, 4);
        }
        w.frame_important = vec![false; td::TILE_COUNT];
        w.compute_column_surface();
        w.compute_chunks();
        let mut sys = LiquidSystem::new(&w);
        sys.add_water(&w, 4, 4);
        for _ in 0..8 {
            sys.update(&mut w);
        }
        let above: i32 = (3..6).map(|x| w.tiles[w.index(x, 4)].liquid as i32).sum();
        let below: i32 = (3..6).map(|x| w.tiles[w.index(x, 6)].liquid as i32).sum();
        assert!(below == 0, "the bubble held the water ({below} passed)");
        assert!(above > 0, "the water stayed above ({above})");
        // The pass tail keeps the bubble solid — `Main.cs:17753` clears it
        // at the next tick's head, before the entity half.
        sys.clear_bubble_for_entities();
        assert!(!sys.bubble_solid, "the stock bubble is not solid");
    }

    #[test]
    fn the_boulder_loses_solidity_for_the_pass() {
        // S1-L-14 (`WorldGen.SetBoulderSolidity`, `WorldGen.cs:11009-11019`,
        // via `Liquid.tilesIgnoreWater`): the boulder 138 does not hold the
        // water during the pass.
        let mut w = World::new("liquid".into(), 9, 11);
        for x in 3..6 {
            let i = w.index(x, 5);
            w.tiles[i] = Tile { kind: 138, ..Default::default() };
            w.tiles[i].flags |= FLAG_ACTIVE;
            let j = w.index(x, 4);
            w.tiles[j].liquid = 255;
            w.mark_chunk(x, 4);
        }
        w.frame_important = vec![false; td::TILE_COUNT];
        w.compute_column_surface();
        w.compute_chunks();
        let mut sys = LiquidSystem::new(&w);
        sys.add_water(&w, 4, 4);
        for _ in 0..40 {
            sys.update(&mut w);
        }
        let mut below: i64 = 0;
        for x in 0..9 {
            for y in 6..11 {
                below += w.tiles[w.index(x, y)].liquid as i64;
            }
        }
        let below = below as i32;
        assert!(below > 0, "the water fell past the boulder ({below})");
    }

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

/// The task slice's pinned flow proofs (`task/liquids`): the fall, the
/// flat-floor spread, the water+lava obsidian transition and the mass
/// ledger — each against the C# numbers.
#[cfg(test)]
mod liquid_sim {
    use super::*;
    use crate::tile::FLAG_ACTIVE;

    /// A 64-wide world with a stone shelf row and the stock solidity
    /// tables (the `tests` fixture shape).
    fn shelf_world() -> World {
        let mut w = World::new("liquid_sim".into(), 64, 120);
        w.frame_important = vec![false; td::TILE_COUNT];
        w.compute_column_surface();
        w.compute_chunks();
        w
    }

    fn stone_row(w: &mut World, x0: i32, x1: i32, y: i32) {
        for x in x0..=x1 {
            let i = w.index(x, y);
            w.tiles[i] = Tile {
                kind: 1,
                ..Default::default()
            };
            w.tiles[i].flags |= FLAG_ACTIVE;
        }
    }

    /// t1 — `Liquid.cs:567-597`: the down arm moves `min(255 - below,
    /// liquid)` into any non-solid, compatible, non-full cell below, and
    /// marks both cells `skipLiquid` (`:586-587` — set after the `AddWater`
    /// of `:585`, which clears it), so the same stride does not re-run the
    /// fed cell (`:1096-1104`): one tile per two passes, byte-exact.
    #[test]
    fn water_falls_per_csharp() {
        let mut w = shelf_world();
        stone_row(&mut w, 20, 43, 70);
        let i = w.index(30, 50);
        w.tiles[i].liquid = 200;
        w.mark_chunk(30, 50);
        w.compute_column_surface();
        w.compute_chunks();
        let mut sys = LiquidSystem::new(&w);
        sys.add_water(&w, 30, 50);
        for _ in 0..10 {
            sys.update(&mut w);
        }
        assert_eq!(w.tiles[w.index(30, 50)].liquid, 0, "the source drained");
        assert_eq!(w.tiles[w.index(30, 55)].liquid, 200, "10 passes -> 5 tiles down");
        let total: u32 = (0..64)
            .map(|x| {
                (0..120)
                    .map(|y| w.tiles[w.index(x, y)].liquid as u32)
                    .sum::<u32>()
            })
            .sum();
        assert_eq!(total, 200, "the down arm is byte-exact");
    }

    /// t2 — `Liquid.cs:589-939`: on a full-width flat floor the row arms
    /// average the cell with its neighbours (`num = Math.Round(sum / n)`,
    /// the `num2 = -1` bias under 3 — `:644-648`) until the film is one
    /// layer. The port's deterministic 12-pass state for this fixture: a
    /// 25-cell film whose interior cells all carry the same 14 (the arms
    /// verified cell-for-cell against the C# at the hand trace).
    #[test]
    fn water_spreads_to_csharp_thickness() {
        let mut w = shelf_world();
        stone_row(&mut w, 5, 58, 70);
        let i = w.index(31, 69); // the cell above the floor
        w.tiles[i].liquid = 255;
        w.mark_chunk(31, 69);
        w.compute_column_surface();
        w.compute_chunks();
        let mut sys = LiquidSystem::new(&w);
        sys.add_water(&w, 31, 69);
        for _ in 0..12 {
            sys.update(&mut w);
        }
        let occupied: Vec<u8> = (5..=58)
            .map(|x| w.tiles[w.index(x, 69)].liquid)
            .filter(|m| *m > 0)
            .collect();
        assert!(occupied.len() >= 24, "the film spread across the floor");
        // The interior cells settle to the same thickness.
        for x in 24..=39 {
            assert_eq!(
                w.tiles[w.index(x, 69)].liquid, 14,
                "the interior thickness at x={x}"
            );
        }
        // Nothing left the row yet.
        let other: i32 = (0..64)
            .map(|x| {
                (0..120)
                    .filter(|y| *y != 69)
                    .map(|y| w.tiles[w.index(x, y)].liquid as i32)
                    .sum::<i32>()
            })
            .sum();
        assert_eq!(other, 0, "the film stays on the shelf");
    }

    /// t3 — `Liquid.cs:1236-1265` + `:1379-1393`: the lava cell's
    /// `LiquidCheck` sums the neighbouring water of a different type and at
    /// `num >= 24` empties both cells, placing the merge tile 56 — the
    /// obsidian.
    #[test]
    fn water_plus_lava_is_obsidian() {
        let mut w = shelf_world();
        stone_row(&mut w, 28, 32, 70);
        let lava = w.index(30, 69);
        w.tiles[lava].liquid = 200;
        set_kind(&mut w.tiles[lava], 1);
        let wat = w.index(31, 69);
        w.tiles[wat].liquid = 100;
        w.mark_chunk(30, 69);
        w.mark_chunk(31, 69);
        w.compute_column_surface();
        w.compute_chunks();
        let mut sys = LiquidSystem::new(&w);
        sys.add_water(&w, 30, 69);
        sys.add_water(&w, 31, 69);
        for _ in 0..12 {
            sys.update(&mut w);
        }
        assert_eq!(w.tiles[w.index(30, 69)].kind, 56, "tile 56 — the obsidian");
        assert_eq!(w.tiles[w.index(30, 69)].liquid, 0, "the merge emptied the cell");
        assert_eq!(w.tiles[w.index(31, 69)].liquid, 0, "the water was consumed");
    }

    /// t4 — the mass ledger. The down arm (`Liquid.cs:570-583`) is
    /// byte-exact: in a walled shaft only that arm runs, so the total is
    /// unchanged over any horizon. The row arms re-average through
    /// `Math.Round` (`:686`/`:780`/`:845`/`:868`/`:891`), so a spreading
    /// film drifts by at most one unit per averaging arm per pass — the
    /// C#'s own quirk (the vanilla liquid-cloning behaviour) — and the
    /// `num2 = -1` bias (`:644-648`) decays the thin edges.
    #[test]
    fn mass_conserved() {
        // The walled shaft: walls at x=29/31, the floor at y=80 — no row
        // arm can reach the falling column, and the ledger is exact.
        let mut w = shelf_world();
        for y in 20..=80 {
            stone_row(&mut w, 29, 29, y);
            stone_row(&mut w, 31, 31, y);
        }
        stone_row(&mut w, 30, 30, 80);
        let i = w.index(30, 40);
        w.tiles[i].liquid = 200;
        w.mark_chunk(30, 40);
        w.compute_column_surface();
        w.compute_chunks();
        let mut sys = LiquidSystem::new(&w);
        sys.add_water(&w, 30, 40);
        let total = |w: &World| -> i32 {
            (0..64)
                .map(|x| {
                    (0..120)
                        .map(|y| w.tiles[w.index(x, y)].liquid as i32)
                        .sum::<i32>()
                })
                .sum()
        };
        let before = total(&w);
        for _ in 0..100 {
            sys.update(&mut w);
        }
        assert_eq!(total(&w), before, "the shaft conserves byte-exactly");
        assert!(
            w.tiles[w.index(30, 79)].liquid == 200
                && w.tiles[w.index(30, 40)].liquid == 0,
            "the blob landed whole"
        );

        // The open floor: the per-pass drift is bounded by the averaging
        // arms' Round — one unit per arm, so |delta| stays within twice the
        // active count every pass.
        let mut w2 = shelf_world();
        stone_row(&mut w2, 5, 58, 70);
        let j = w2.index(31, 69);
        w2.tiles[j].liquid = 255;
        w2.mark_chunk(31, 69);
        w2.compute_column_surface();
        w2.compute_chunks();
        let mut sys2 = LiquidSystem::new(&w2);
        sys2.add_water(&w2, 31, 69);
        let row_total = |w: &World| -> i32 {
            (0..64)
                .map(|x| {
                    (0..120)
                        .map(|y| w.tiles[w.index(x, y)].liquid as i32)
                        .sum::<i32>()
                })
                .sum()
        };
        let mut prev = row_total(&w2);
        for _ in 0..60 {
            sys2.update(&mut w2);
            let now = row_total(&w2);
            let bound = 2 * sys2.active_len() as i32 + 4;
            assert!(
                (now - prev).abs() <= bound,
                "the per-pass drift stays in the Round band (delta {}, active {})",
                now - prev,
                sys2.active_len()
            );
            prev = now;
        }
        assert!(prev > 0, "the film survives (decayed to {prev})");
    }
}

/// W4-c34 — the non-water cadence and evaporation pins: the per-entry
/// `delay` throttle (lava 1-in-6, honey 1-in-11), the `quickFall` bypass,
/// the underworld water evaporation, the down-arm sub-24 drain and the
/// shimmer-side merge arms the R-11-5 suite left open. Written, NOT run
/// (code-only mode).
#[cfg(test)]
mod liquid_merge_cadence {
    use super::*;
    use crate::tile::FLAG_ACTIVE;

    /// A stone floor row at `floor_y` (the `basin` fixture shape, tall
    /// enough for the underworld band when `height >= 200`).
    fn floor_world(width: i32, height: i32, floor_y: i32) -> World {
        let mut w = World::new("cadence".into(), width, height);
        for x in 0..width {
            let i = w.index(x, floor_y);
            w.tiles[i] = Tile { kind: 1, ..Default::default() };
            w.tiles[i].flags |= FLAG_ACTIVE;
        }
        w.frame_important = vec![false; td::TILE_COUNT];
        w.compute_column_surface();
        w.compute_chunks();
        w
    }

    /// A one-wide walled shaft: only the down arm can run, so a falling
    /// column's position is a direct cadence clock.
    fn shaft(width: i32, height: i32, floor_y: i32, cx: i32) -> World {
        let mut w = floor_world(width, height, floor_y);
        for y in 0..floor_y {
            for x in [cx - 1, cx + 1] {
                let i = w.index(x, y);
                w.tiles[i] = Tile { kind: 1, ..Default::default() };
                w.tiles[i].flags |= FLAG_ACTIVE;
            }
        }
        w.compute_column_surface();
        w.compute_chunks();
        w
    }

    fn put_as(w: &mut World, x: i32, y: i32, amount: u8, kind: u8) {
        let i = w.index(x, y);
        w.tiles[i].liquid = amount;
        set_kind(&mut w.tiles[i], kind);
        w.mark_chunk(x, y);
    }

    fn run(sys: &mut LiquidSystem, w: &mut World, n: usize) {
        for _ in 0..n {
            sys.update(w);
        }
    }

    #[test]
    fn lava_falls_once_every_six_visits() {
        // `Liquid.cs:484-490`: the lava arm returns while `delay < 5`
        // (five throttle visits), then resets `delay = 0` and flows — one
        // down-transfer per six visits. A short active list is re-walked
        // every update (`:1077-1082`), so 12 updates = 12 visits = exactly
        // two falls: the column sits at y = 2, not y = 12 (an ungated flow).
        let mut w = shaft(5, 120, 30, 2);
        put_as(&mut w, 2, 0, 255, 1);
        let mut sys = LiquidSystem::new(&w);
        sys.add_water(&w, 2, 0);
        run(&mut sys, &mut w, 12);
        assert_eq!(w.tiles[w.index(2, 2)].liquid, 255, "two falls in twelve visits");
        assert_eq!(w.tiles[w.index(2, 12)].liquid, 0, "no ungated flow");
        // The next fall lands on visit 19: visit 6's fall marks (2,1)
        // skipLiquid (`:586-587`), so visit 7 only clears the flag
        // (`:1096-1104`); visits 8-13 throttle, visit 14 flows, visits
        // 15-18 throttle, visit 19 flows.
        run(&mut sys, &mut w, 7);
        assert_eq!(w.tiles[w.index(2, 3)].liquid, 255, "the third fall on visit 19");
    }

    #[test]
    fn honey_falls_once_every_eleven_visits() {
        // `Liquid.cs:513-522`: the honey arm returns while `delay < 10`,
        // so the first fall lands on the eleventh visit and the second on
        // the twenty-second.
        let mut w = shaft(5, 120, 30, 2);
        put_as(&mut w, 2, 0, 255, 2);
        let mut sys = LiquidSystem::new(&w);
        sys.add_water(&w, 2, 0);
        run(&mut sys, &mut w, 10);
        assert_eq!(w.tiles[w.index(2, 0)].liquid, 255, "ten throttled visits");
        run(&mut sys, &mut w, 1);
        assert_eq!(w.tiles[w.index(2, 1)].liquid, 255, "the first fall on visit 11");
        // Visit 12 clears the fall's skipLiquid flag (`:1096-1104`),
        // visits 13-22 throttle, visit 23 flows — the second fall.
        run(&mut sys, &mut w, 12);
        assert_eq!(w.tiles[w.index(2, 2)].liquid, 255, "the second fall on visit 23");
    }

    #[test]
    fn quick_fall_bypasses_the_delay_gate() {
        // `Liquid.cs:1083-1090`: the quickFall loop forces `delay = 10`
        // before every Update, so the lava arm's `delay < 5` gate never
        // fires — the column falls every visit like water. The quickFall
        // loop also clears the fall's skipLiquid flag right after the
        // update (`:1089`), so no visit is lost to it.
        let mut w = shaft(5, 120, 30, 2);
        put_as(&mut w, 2, 0, 255, 1);
        let mut sys = LiquidSystem::new(&w);
        sys.quick_fall = true;
        sys.add_water(&w, 2, 0);
        run(&mut sys, &mut w, 12);
        assert_eq!(w.tiles[w.index(2, 12)].liquid, 255, "quickFall flows every visit");
    }

    #[test]
    fn the_delay_gate_never_throttles_the_merge() {
        // `Liquid.cs:480-490`: `LavaCheck` runs BEFORE the delay gate —
        // the merge lands on the very first visit even though the flow
        // would be throttled for five more.
        let mut w = floor_world(9, 120, 8);
        put_as(&mut w, 4, 4, 30, 1);
        put_as(&mut w, 5, 4, 30, 0);
        let mut sys = LiquidSystem::new(&w);
        sys.add_water(&w, 4, 4);
        run(&mut sys, &mut w, 1);
        assert_eq!(w.tiles[w.index(4, 4)].kind, 56, "the merge ignores the delay");
    }

    #[test]
    fn water_evaporates_two_per_visit_below_the_underworld_layer() {
        // `Liquid.cs:466-474`: below `UnderworldLayer` (= height - 200,
        // `Main.cs:2790`) water loses min(2, liquid) per visit. A 400-tall
        // world puts the layer at 200; the cell at y = 300 sits in a
        // one-wide shaft, so only evaporation runs (no fall, no row arm).
        let mut w = shaft(5, 400, 350, 2);
        put_as(&mut w, 2, 300, 100, 0);
        let mut sys = LiquidSystem::new(&w);
        sys.add_water(&w, 2, 300);
        run(&mut sys, &mut w, 1);
        assert_eq!(w.tiles[w.index(2, 300)].liquid, 98, "two units per visit");
        run(&mut sys, &mut w, 49);
        assert_eq!(w.tiles[w.index(2, 300)].liquid, 0, "fifty visits drain it dry");
        assert_eq!(sys.active_len(), 0, "the zero kill retires the entry (`:475`)");
    }

    #[test]
    fn lava_honey_shimmer_never_evaporate() {
        // `Liquid.cs:466`'s `liquidType() == 0` gate: the non-water kinds
        // keep every unit below the layer (the honey/lava columns also sit
        // in their delay gates, so nothing moves at all).
        let mut w = floor_world(12, 400, 301);
        put_as(&mut w, 3, 300, 100, 1);
        put_as(&mut w, 6, 300, 100, 2);
        put_as(&mut w, 9, 300, 100, 3);
        let mut sys = LiquidSystem::new(&w);
        sys.add_water(&w, 3, 300);
        sys.add_water(&w, 6, 300);
        sys.add_water(&w, 9, 300);
        run(&mut sys, &mut w, 30);
        assert_eq!(w.tiles[w.index(3, 300)].liquid, 100, "lava keeps its mass");
        assert_eq!(w.tiles[w.index(6, 300)].liquid, 100, "honey keeps its mass");
        assert_eq!(w.tiles[w.index(9, 300)].liquid, 100, "shimmer keeps its mass");
    }

    #[test]
    fn water_at_the_underworld_layer_row_does_not_evaporate() {
        // `Liquid.cs:466` is a strict `y > UnderworldLayer`: the layer row
        // itself (y = 200 in a 400-tall world) keeps its water. The floor
        // at y = 201 pins the cell, and the stone walls at x = 3/5 fence
        // the row arms, so only the evaporation gate is tested.
        let mut w = shaft(9, 400, 201, 4);
        put_as(&mut w, 4, 200, 100, 0);
        let mut sys = LiquidSystem::new(&w);
        sys.add_water(&w, 4, 200);
        run(&mut sys, &mut w, 5);
        assert_eq!(w.tiles[w.index(4, 200)].liquid, 100, "the layer row is exempt");
    }

    #[test]
    fn the_down_arm_drains_a_sub_24_cell_into_the_liquid_below() {
        // `Liquid.cs:1290-1298`: the down arm's `tile5.liquid < 24` branch
        // zeroes the checking cell AND re-types it water (`liquidType(0)`)
        // — no merge tile; the liquid below survives untouched.
        let mut w = floor_world(9, 120, 8);
        put_as(&mut w, 4, 4, 20, 1);
        put_as(&mut w, 4, 5, 30, 0);
        let mut sys = LiquidSystem::new(&w);
        sys.liquid_check(&mut w, 4, 4, 1, false);
        assert_eq!(w.tiles[w.index(4, 4)].liquid, 0, "the sub-24 cell drained");
        assert_eq!(kind_code(w.tiles[w.index(4, 4)].liquid_kind()), 0, "re-typed water");
        assert_eq!(w.tiles[w.index(4, 5)].liquid, 30, "the pool below survives");
        assert_eq!(w.tiles[w.index(4, 5)].kind, 0, "no merge tile");
        assert!(sys.drain_merge_events().is_empty(), "the drain is silent");
    }

    #[test]
    fn the_water_cell_down_arm_burns_no_plant() {
        // `Liquid.cs:1278`'s `thisLiquidType != 0` gate: water does not
        // burn the cut plant below; the sub-24 drain still runs.
        let mut w = floor_world(9, 120, 8);
        let plant = w.index(4, 5);
        w.tiles[plant] = Tile { kind: 3, ..Default::default() };
        w.tiles[plant].flags |= FLAG_ACTIVE;
        w.tiles[plant].liquid = 30;
        set_kind(&mut w.tiles[plant], 1);
        put_as(&mut w, 4, 4, 20, 0);
        let mut sys = LiquidSystem::new(&w);
        sys.liquid_check(&mut w, 4, 4, 0, false);
        assert_eq!(w.tiles[plant].kind, 3, "the plant survives water");
        assert!(w.tiles[plant].active(), "still active");
        assert_eq!(w.tiles[w.index(4, 4)].liquid, 0, "the drain ran");
    }

    #[test]
    fn shimmer_merges_with_lava_and_honey_into_659() {
        // `Liquid.cs:1395-1409`/`:1411-1426`: the shimmer cell's lava arm
        // and honey arm both land 659 (`GetLiquidMergeTypes`' shimmer
        // rows), consuming the pooled neighbour.
        for other in [1u8, 2] {
            let mut w = floor_world(9, 120, 8);
            put_as(&mut w, 4, 4, 30, 3);
            put_as(&mut w, 5, 4, 30, other);
            let mut sys = LiquidSystem::new(&w);
            sys.add_water(&w, 4, 4);
            run(&mut sys, &mut w, 1);
            assert_eq!(w.tiles[w.index(4, 4)].kind, 659, "shimmer + {other} -> 659");
            assert_eq!(w.tiles[w.index(5, 4)].liquid, 0, "the pool consumed");
        }
    }
}

/// W4-g45 phase 1 — the `Liquid.DelWater` retirement machinery
/// (`Liquid.cs:1470-1599`) and the cycle-end sweep (`:1106-1150`): the
/// swap-remove that copies `x`/`y`/`kill` but NOT `delay`, the 254→255
/// promotion, the keep-alive branch, the lava grass-burn ring, the
/// alch/lily-pad tail, the buffer `checkingLiquid` promotion and the stuck
/// window. The sweep is driven through `end_pass` directly so the pinned
/// counters are not disturbed by the per-visit `update_tile` first.
/// Written, NOT run (code-only mode).
#[cfg(test)]
mod liquid_delwater {
    use super::*;
    use crate::tile::FLAG_ACTIVE;

    fn floor_world(width: i32, height: i32, floor_y: i32) -> World {
        let mut w = World::new("delwater".into(), width, height);
        for x in 0..width {
            let i = w.index(x, floor_y);
            w.tiles[i] = Tile { kind: 1, ..Default::default() };
            w.tiles[i].flags |= FLAG_ACTIVE;
        }
        w.frame_important = vec![false; td::TILE_COUNT];
        w.compute_column_surface();
        w.compute_chunks();
        w
    }

    fn put_as(w: &mut World, x: i32, y: i32, amount: u8, kind: u8) {
        let i = w.index(x, y);
        w.tiles[i].liquid = amount;
        set_kind(&mut w.tiles[i], kind);
        w.mark_chunk(x, y);
    }

    fn put_tile(w: &mut World, x: i32, y: i32, kind: u16) {
        let i = w.index(x, y);
        w.tiles[i] = Tile { kind, ..Default::default() };
        w.tiles[i].flags |= FLAG_ACTIVE;
        w.mark_chunk(x, y);
    }

    #[test]
    fn the_swap_remove_inherits_the_retired_slots_delay() {
        // `Liquid.cs:1577-1579`: the retirement swap copies x/y/kill only —
        // the slot's `delay` field is left untouched, so the moved entry
        // INHERITS the retired cell's throttle counter (the C# never copies
        // `Main.liquid[numLiquid].delay`). Slot 0 is a sub-2 lava film (the
        // `:1478` branch dries it, no keep-alive) with delay 4 and kill 8;
        // slot 1 is a draining water column (open below → keep-alive) with
        // delay 0 and kill 0. The reverse sweep retires slot 0 and swaps the
        // surviving slot 1 into it — carrying slot 0's delay 4, not its own 0.
        let mut w = floor_world(15, 120, 30);
        put_as(&mut w, 6, 7, 1, 1);
        put_as(&mut w, 8, 4, 100, 0);
        let mut sys = LiquidSystem::new(&w);
        sys.add_water(&w, 6, 7);
        sys.add_water(&w, 8, 4);
        sys.active[0].delay = 4;
        sys.active[0].kill = KILL_PASSES;
        sys.active[1].delay = 0;
        sys.active[1].kill = 0;
        sys.end_pass(&mut w);
        assert_eq!(sys.active_len(), 1, "the film retired, the column survived");
        assert_eq!(sys.active[0].x, 8, "the survivor's coordinates");
        assert_eq!(sys.active[0].delay, 4, "the survivor inherited the slot's delay");
        assert_eq!(w.tiles[w.index(6, 7)].liquid, 0, "the film dried (`:1481`)");
    }

    #[test]
    fn the_sweep_promotes_a_254_cell_to_255() {
        // `Liquid.cs:1113-1116`: a 254 cell the kill sweep catches is topped
        // up to 255 before DelWater reads it. A full cell on the solid floor
        // (below solid → no keep-alive) then retires, keeping its 255.
        let mut w = floor_world(15, 120, 8);
        put_as(&mut w, 7, 7, 254, 0);
        let mut sys = LiquidSystem::new(&w);
        sys.add_water(&w, 7, 7);
        sys.active[0].kill = KILL_PASSES;
        sys.end_pass(&mut w);
        assert_eq!(w.tiles[w.index(7, 7)].liquid, 255, "the 254 promoted to 255");
        assert_eq!(sys.active_len(), 0, "the full cell retired");
    }

    #[test]
    fn the_keep_alive_branch_rescues_a_draining_cell() {
        // `Liquid.cs:1506-1510`: a >=20 cell with an open, non-full below and
        // an open here resets kill to 0 and stays.
        let mut w = floor_world(15, 120, 30);
        put_as(&mut w, 7, 4, 100, 0);
        let mut sys = LiquidSystem::new(&w);
        sys.add_water(&w, 7, 4);
        sys.active[0].kill = KILL_PASSES;
        sys.end_pass(&mut w);
        assert_eq!(sys.active_len(), 1, "the draining cell stays alive");
        assert_eq!(sys.active[0].kill, 0, "the keep-alive reset kill");
    }

    #[test]
    fn lava_retirement_burns_the_grass_ring() {
        // `Liquid.cs:1532-1560`: the retired lava cell's 3×3 ring turns the
        // grass family (2/23/109/199/477/492) to dirt 0 and the mud/ash
        // family (60/70/661/662) to 59. Interior coordinates keep the
        // SquareTileFrame 5-tile margin clear.
        let mut w = floor_world(17, 120, 8);
        put_as(&mut w, 8, 7, 255, 1);
        put_tile(&mut w, 7, 7, 2);
        put_tile(&mut w, 9, 7, 2);
        put_tile(&mut w, 8, 6, 109);
        put_tile(&mut w, 8, 8, 60); // mud in the ring -> 59
        let mut sys = LiquidSystem::new(&w);
        sys.add_water(&w, 8, 7);
        sys.active[0].kill = KILL_PASSES;
        sys.end_pass(&mut w);
        assert_eq!(w.tiles[w.index(7, 7)].kind, 0, "grass -> dirt");
        assert_eq!(w.tiles[w.index(9, 7)].kind, 0, "grass -> dirt");
        assert_eq!(w.tiles[w.index(8, 6)].kind, 0, "jungle grass -> dirt");
        assert_eq!(w.tiles[w.index(8, 8)].kind, 59, "the ring mud -> 59");
    }

    #[test]
    fn the_alch_tail_kills_an_unsupported_herb() {
        // `Liquid.cs:1580-1583`: a retired cell holding an alch herb (82/83/
        // 84) runs CheckAlch — a Daybloom on stone (not grass) pops.
        let mut w = floor_world(17, 120, 8);
        put_tile(&mut w, 8, 6, 82);
        put_as(&mut w, 8, 6, 255, 0);
        let mut sys = LiquidSystem::new(&w);
        sys.add_water(&w, 8, 6);
        sys.active[0].kill = KILL_PASSES;
        sys.end_pass(&mut w);
        assert!(!w.tiles[w.index(8, 6)].active(), "the unsupported herb died");
    }

    #[test]
    fn the_lily_pad_tail_sinks_a_dry_pad_into_the_cell_below() {
        // `Liquid.cs:1584-1597`: a retired 518 cell with an empty below and
        // no liquid above runs CheckLilyPad — the dry pad sinks one row. The
        // ground walk must find a grass family (dirt-grass 2 here → frame row
        // 0); the empty cell below the pad takes it.
        let mut w = floor_world(17, 120, 8);
        put_tile(&mut w, 8, 6, 518);
        put_tile(&mut w, 8, 8, 2); // dirt-grass ground for the frame walk
        let mut sys = LiquidSystem::new(&w);
        // The pad holds no liquid; add_water refuses it, so seed the entry
        // directly (the C# reaches DelWater through a kill sweep on a cell
        // that once held liquid).
        sys.active.push(Entry { x: 8, y: 6, kill: KILL_PASSES, delay: 0 });
        sys.set_checking(8, 6, true);
        sys.end_pass(&mut w);
        assert!(!w.tiles[w.index(8, 6)].active(), "the source pad deactivated");
        assert_eq!(w.tiles[w.index(8, 7)].kind, 518, "the pad sank into (8,7)");
        assert!(w.tiles[w.index(8, 7)].active(), "the destination is active");
    }

    #[test]
    fn the_buffer_promotion_clears_checking_and_resets_delay() {
        // `Liquid.cs:1125-1130` + `LiquidBuffer.cs:11-20`: a buffered cell is
        // marked checkingLiquid; the cycle-end promotion clears it, AddWater
        // re-queues with delay 0, DelBuffer swap-removes the slot. The
        // promotion count is the live active count (`num8 = numLiquid`), so a
        // survivor must remain after the sweep for the buffer to drain.
        let mut w = floor_world(15, 120, 30);
        put_as(&mut w, 5, 4, 100, 0); // survivor: open below, keep-alive
        put_as(&mut w, 6, 7, 1, 0); // film: retires in the sweep
        put_as(&mut w, 8, 7, 100, 0); // overflow: buffered
        let mut sys = LiquidSystem::new(&w);
        sys.cur_max_liquid = 3; // active cap = 2
        sys.add_water(&w, 5, 4);
        sys.add_water(&w, 6, 7);
        sys.add_water(&w, 8, 7);
        assert_eq!(sys.active_len(), 2, "two cells fit the active cap");
        assert_eq!(sys.buffered_len(), 1, "the third cell overflowed");
        assert!(sys.is_checking(8, 7), "the buffered cell is checking");
        sys.active[1].kill = KILL_PASSES; // the film retires
        sys.end_pass(&mut w);
        assert_eq!(sys.buffered_len(), 0, "the buffer drained");
        assert_eq!(sys.active_len(), 2, "survivor + promoted cell");
        assert!(sys.is_checking(8, 7), "the promoted cell is checking again");
        let promoted = sys.active.iter().find(|e| e.x == 8).expect("promoted");
        assert_eq!(promoted.delay, 0, "AddWater re-queued with delay 0");
    }

    #[test]
    fn the_stuck_window_force_clears_after_10000_cycles() {
        // `Liquid.cs:1131-1149`: an active count pinned within ±50 of
        // stuckAmount for 10 000 cycle-ends force-clears the list. The cell
        // is a draining water column (open below) with kill 0, so the normal
        // sweep leaves it; only the force-clear (stuck up, keep-alive
        // disabled) drains it.
        let mut w = floor_world(15, 120, 30);
        put_as(&mut w, 7, 4, 100, 0);
        let mut sys = LiquidSystem::new(&w);
        sys.add_water(&w, 7, 4);
        sys.stuck_amount = 1;
        sys.stuck_count = 9_999;
        sys.end_pass(&mut w);
        assert_eq!(sys.active_len(), 0, "the force-clear drained the list");
        assert_eq!(sys.stuck_count, 0, "the force-clear reset the counter");
        assert!(!sys.stuck, "stuck lowered after the clear");
    }
}

/// W4-g45 phase 2 — the 3-neighbour `genRand` 254→255 nudge
/// (`Liquid.cs:892-895`) and the 7/5-neighbour extra wake passes
/// (`:748-771`/`:821-836`). The nudge rides the port's `genRand` stand-in
/// (`World.frame_rng`); the RNG census pins the exact draw. Written, NOT
/// run (code-only mode).
#[cfg(test)]
mod liquid_nudge_wakes {
    use super::*;
    use crate::tile::FLAG_ACTIVE;

    fn floor_world(width: i32, height: i32, floor_y: i32) -> World {
        let mut w = World::new("nudge".into(), width, height);
        for x in 0..width {
            let i = w.index(x, floor_y);
            w.tiles[i] = Tile { kind: 1, ..Default::default() };
            w.tiles[i].flags |= FLAG_ACTIVE;
        }
        w.frame_important = vec![false; td::TILE_COUNT];
        w.compute_column_surface();
        w.compute_chunks();
        w
    }

    fn put_as(w: &mut World, x: i32, y: i32, amount: u8, kind: u8) {
        let i = w.index(x, y);
        w.tiles[i].liquid = amount;
        set_kind(&mut w.tiles[i], kind);
        w.mark_chunk(x, y);
    }

    #[test]
    fn the_254_nudge_promotes_on_a_zero_genrand_draw() {
        // `Liquid.cs:892-895`: a 3-neighbour average of 254 rolls
        // `genRand.Next(30)` and promotes to 255 on a zero. Seed 35's first
        // `UnifiedRandom` draw is 0 (the RNG census, below), so the settled
        // 254 row promotes. Three equal 254 cells average to 254.
        let mut w = floor_world(15, 120, 8);
        w.frame_rng = crate::unified_random::UnifiedRandom::new(35);
        put_as(&mut w, 6, 7, 254, 0);
        put_as(&mut w, 7, 7, 254, 0);
        put_as(&mut w, 8, 7, 254, 0);
        let mut sys = LiquidSystem::new(&w);
        sys.add_water(&w, 7, 7);
        sys.update(&mut w);
        assert_eq!(w.tiles[w.index(7, 7)].liquid, 255, "the zero draw promoted 254");
    }

    #[test]
    fn the_254_nudge_keeps_254_on_a_nonzero_draw() {
        // Seed 0's first `Next(30)` draw is 21 (the census), so the 254 row
        // stays 254 — the nudge only fires on a zero.
        let mut w = floor_world(15, 120, 8);
        w.frame_rng = crate::unified_random::UnifiedRandom::new(0);
        put_as(&mut w, 6, 7, 254, 0);
        put_as(&mut w, 7, 7, 254, 0);
        put_as(&mut w, 8, 7, 254, 0);
        let mut sys = LiquidSystem::new(&w);
        sys.add_water(&w, 7, 7);
        sys.update(&mut w);
        assert_eq!(w.tiles[w.index(7, 7)].liquid, 254, "the nonzero draw kept 254");
    }

    #[test]
    fn the_nudge_draw_only_advances_the_stream_on_a_254() {
        // The C# consumes the shared `genRand` draw ONLY when the average is
        // 254 (`:892`'s `num == 254f` guard). A non-254 settle leaves the
        // stream untouched; a 254 settle advances it by exactly one draw.
        let mut w = floor_world(15, 120, 8);
        w.frame_rng = crate::unified_random::UnifiedRandom::new(0);
        put_as(&mut w, 6, 7, 100, 0);
        put_as(&mut w, 7, 7, 100, 0);
        put_as(&mut w, 8, 7, 100, 0);
        let before = w.frame_rng.peek();
        let mut sys = LiquidSystem::new(&w);
        sys.add_water(&w, 7, 7);
        sys.update(&mut w);
        assert_eq!(w.frame_rng.peek(), before, "a non-254 settle drew nothing");
    }

    #[test]
    fn the_extra_wake_requeues_an_unqueued_neighbour_on_a_centre_change() {
        // `Liquid.cs:748-771`/`:821-836`: the extra wake re-queues a neighbour
        // when the CENTRE differs from the average, EVEN IF that neighbour's
        // own level already equals the average (so `set_level` did not wake
        // it). Only the centre is active here; the right neighbour holds the
        // average level but is unqueued. The centre changes (20 -> 10), so the
        // extra wake pulls the right neighbour into the list — without it the
        // neighbour stays unqueued (`set_level` saw no change).
        let mut w = floor_world(15, 120, 8);
        put_as(&mut w, 7, 7, 20, 0);
        put_as(&mut w, 8, 7, 10, 0);
        let mut sys = LiquidSystem::new(&w);
        sys.add_water(&w, 7, 7);
        assert_eq!(sys.active_len(), 1, "only the centre is queued");
        assert!(!sys.is_checking(8, 7), "the right neighbour starts unqueued");
        sys.update(&mut w);
        // The 3-arm average (0 + 10 + 20)/3 = 10 differs from the centre's 20;
        // the right neighbour already held 10, so only the extra wake queues it.
        assert!(sys.is_checking(8, 7), "the extra wake re-queued the neighbour");
    }
}

