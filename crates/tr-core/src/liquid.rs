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
    /// `Liquid.quickFall` (`Liquid.cs:33`): derived at the head of every
    /// pass from `quickSettle` (`:1056-1068`); the quick-settle mode forces
    /// every entry's `delay = 10` in the pass loop (`:1083-1090`), so
    /// lava/honey flow every tick, and clears the skip flag after each
    /// update instead of honouring it.
    quick_fall: bool,
    /// `Liquid.quickSettle` (`Liquid.cs:35`): the bulk-settle mode the
    /// world-load settle (`WorldFile.cs:738-760`) and the worldgen settle
    /// loops (`WorldGen.cs:16160`/`:21267`) flip around their
    /// `UpdateLiquid` drains. Three behaviour arms: the fall's remainder
    /// clamps to 255 (`Liquid.cs:588-591`), the 254 settle step promotes
    /// straight to 255 (`:944-948`), and the pass head derives `quickFall`
    /// from it (`:1056-1063`). The gfx-quality pass reads it for the
    /// full-window / single-cycle override (`Main.cs:17105-17113`).
    quick_settle: bool,
    /// `WorldGen.remixWorldGen` — the desert-strip arm's gate
    /// (`Liquid.cs:1447`); the R-12-5 plumbing owns it, `false` at runtime.
    remix_world_gen: bool,
    /// `WorldGen.isGeneratingOrLoadingWorld` — the merge-tile branch selector
    /// (`Liquid.cs:1333`/`:1447`) and the `SettleWaterAt` re-type gate
    /// (`:178`). The port's settlement pass never runs during generation
    /// yet (R-12-5 plumbs it), so it stays `false`.
    generating: bool,
    /// `WorldGen.generatingWorld` — the STRICT generation flag (true only
    /// during worldgen, false during the world-LOAD settle). It gates the
    /// `SettleWaterAt` dungeon re-type arm (`Liquid.cs:188`), which the
    /// load settle (`isGeneratingOrLoadingWorld` true, `generatingWorld`
    /// false) must NOT run. Distinct from [`Self::generating`] above; the
    /// dungeon arm itself is a recorded no-op (no ported
    /// `InAnyPotentialDungeonBounds`), so the flag rides for R-12-5.
    #[allow(dead_code)] // read by the dungeon arm when R-12-5 ports its predicates
    generating_world: bool,
    /// `GenVars.waterLine` (`GenVars.cs:62`) — the classic re-type boundary:
    /// a settled drop landing BELOW it becomes lava (`Liquid.cs:184-187`).
    /// The world-load settle pins it to `maxTilesY` (`WorldFile.cs:734`),
    /// which makes the arm dead on load; the gen settle passes carry the
    /// terrain-fill product. Read only while [`Self::generating`] is up.
    gen_water_line: i32,
    /// `GenVars.lavaLine` (`GenVars.cs:64`) — the remix re-type boundary
    /// (`Liquid.cs:182`); read only while [`Self::generating`] and
    /// [`Self::remix_world_gen`] are up.
    gen_lava_line: i32,
    /// `Liquid.stuckCount` (`Liquid.cs:21`): consecutive full cycles whose
    /// active count stayed within ±50 of `stuck_amount` (`:1131-1148`).
    stuck_count: i32,
    /// `Liquid.stuckAmount` (`Liquid.cs:23`): the active count the stuck
    /// band compares against; re-pinned whenever the count leaves the band.
    stuck_amount: i32,
    /// `Liquid.stuck` (`Liquid.cs:27`): up only during the 10 000-cycle
    /// self-clear (`:1136-1141`) — it disables `DelWater`'s keep-alive
    /// branch (`:1506`) so the pathological pool empties itself.
    stuck: bool,
    /// `Liquid.panicCounter` (`Liquid.cs:39`): consecutive updates whose
    /// overflow buffer sits at ≥ 90% of `maxLiquidBuffer` (`:1009-1020`);
    /// past 3600 the pass starts the panic force-settle.
    panic_counter: i32,
    /// `Liquid.panicMode` (`Liquid.cs:41`): the force-settle is running —
    /// the pass walks `QuickWater` rows from `panic_y` down instead of the
    /// cellular update (`:1022-1054`).
    panic_mode: bool,
    /// `Liquid.panicY` (`Liquid.cs:43`): the next row the panic settle
    /// walks (five rows per update, bottom-up).
    panic_y: i32,
    /// The `WorldGen.PlayLiquidChangeSound` events (`Liquid.cs:1345-1347`,
    /// S1-E-25): one row per live-game merge tile — `(kind, x, y)` in tile
    /// coordinates, drained by the audio caller after the pass. The C# plays
    /// the cue inline (`!Main.gameMenu && !isGeneratingOrLoadingWorld`);
    /// the port's engine is pure, so the caller's seam carries it.
    merge_events: Vec<(TileChangeType, i32, i32)>,
    /// The `CheckAlch`/`CheckLilyPad` kills `DelWater`'s retirement tail
    /// performs (`Liquid.cs:1580-1598`): the C# spawns the drops inline
    /// (`KillTile` → `KillTile_DropItems`); the port's engine is pure, so
    /// the records queue here and the liquid tick resolves them through
    /// the same `TileKill` drop seam the mining path uses.
    edit_kills: Vec<crate::edit::TileKill>,
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

    /// `Liquid.quickSettle` (`Liquid.cs:35`) — the bulk-settle mode the
    /// world-load settle (`WorldFile.cs:738`/`:760`) and the worldgen
    /// settle loops (`WorldGen.cs:16160`/`:16206`, `:21267`/`:21305`)
    /// flip around their `UpdateLiquid` drains. `Liquid.ReInit` clears
    /// it (`Liquid.cs:93`).
    pub fn quick_settle(&self) -> bool {
        self.quick_settle
    }

    pub fn set_quick_settle(&mut self, quick_settle: bool) {
        self.quick_settle = quick_settle;
    }

    /// `Liquid.ReInit` (`Liquid.cs:83-102`) — the world-load reset: every
    /// cadence/stuck/panic counter back to zero, the window and cycles to
    /// the stock defaults, the active list emptied. The port's per-tile
    /// `checkingLiquid`/`skipLiquid` bits clear with the list (the C#
    /// leaves them on the tiles, but `WaterCheck` — the load tail — wipes
    /// them across the grid, so the combined load sequence matches).
    pub fn re_init(&mut self) {
        self.stuck_count = 0;
        self.stuck_amount = 0;
        self.cycles = CYCLES;
        self.cur_max_liquid = MAX_ACTIVE;
        self.active.clear();
        self.checking.iter_mut().for_each(|w| *w = 0);
        self.skip.iter_mut().for_each(|w| *w = 0);
        self.buffer.clear();
        self.stuck = false;
        self.quick_fall = false;
        self.quick_settle = false;
        self.wet_counter = 0;
        self.panic_counter = 0;
        self.panic_mode = false;
        self.panic_y = 0;
    }

    /// `Liquid.StartPanic` (`Liquid.cs:966-981`) — the overflow buffer
    /// stayed ≥ 90% full for 3600 updates: drop the cellular lists, raise
    /// `panicMode` and start the force-settle at the bottom row. The C#
    /// also pins `GenVars.waterLine = maxTilesY` (the gen-only settle
    /// re-type reads it); the port's runtime settle never re-types, so the
    /// pin has no runtime consumer.
    fn start_panic(&mut self) {
        if !self.panic_mode {
            self.active.clear();
            self.checking.iter_mut().for_each(|w| *w = 0);
            self.buffer.clear();
            self.panic_counter = 0;
            self.panic_mode = true;
            self.panic_y = self.height - 3;
        }
    }

    /// The world-load settle (`IO/WorldFile.cs:733-764`): `QuickWater`
    /// drains the grid, `WaterCheck` re-wakes the flow cells, then the
    /// `quickSettle` drain walks `UpdateLiquid` until the lists empty
    /// (capped at 100 000 calls), and a final `WaterCheck` settles the
    /// remainder. The generation-only `ShimmerRemoveWater`/boulder arms
    /// inside `QuickWater` stay with the unported gen passes.
    ///
    /// Wired into the load path at [`crate::app::App::new`] (the shared
    /// boot + `swap_world` construction — the port's `WorldFile.LoadWorld`
    /// tail). The C# `GenVars.waterLine = maxTilesY` pin (`:734`) makes the
    /// `SettleWaterAt` re-type arm dead on load (`num2 > waterLine` never
    /// holds); the port's `gen_water_line` carries the same pin, so even
    /// with the load context armed below the re-type is inert — the two
    /// reach the identical no-re-type result.
    ///
    /// The C# runs the whole settle under `isGeneratingOrLoadingWorld = true`
    /// (`:733`/`:764`). The port's `generating` field stands in for that flag
    /// in the merge selector (`create_liquid_merge_tile`, `Liquid.cs:1329`),
    /// so it is armed for the drain: a load-time merge takes the
    /// `LiquidOverwriteStrip` arm and stays silent, exactly as vanilla — not
    /// the runtime tile-placement + sound arm. The STRICT `generatingWorld`
    /// flag stays down (load is not generation), keeping the lava
    /// underground-desert strip (`Liquid.cs:1447`) and the dungeon re-type
    /// (`:188`) inert. Both context flags drop back at `:764`.
    ///
    /// RNG: the settle draws the caller-pinned `world.frame_rng` (the port's
    /// `genRand` stand-in) ONLY through the `SquareTileFrame` reset rolls
    /// inside the kill cascades (`DelWater`'s lava grass ring / lily-pad
    /// reframe, `WaterCheck`'s liquid-death kills). A dry world consumes
    /// zero draws, so the save/load round-trip pins do not move; a wet world
    /// with objects near liquid re-pins `frame_rng` (ANALYSIS, W4-g47 — the
    /// re-pin obligation).
    pub fn settle_loaded_world(&mut self, world: &mut World) {
        self.re_init();
        // `:733` — the load context: the merge selector reads
        // `isGeneratingOrLoadingWorld`, the re-type reads the `waterLine`
        // pin. `generating_world` stays down (load ≠ generation).
        self.generating = true;
        self.gen_water_line = self.height;
        self.quick_water(world, -1, -1);
        self.water_check(world);
        self.quick_settle = true;
        // The drain runs on the `ReInit` stock pair (cycles 10, the full
        // window): the C# gfx block that would rewrite it
        // (`Main.cs:17076-17091`) and the `quickSettle` override
        // (`:17105-17113`) are BOTH gated on `!isGeneratingOrLoadingWorld`,
        // which is `true` for the whole load (`WorldFile.cs:733`/`:764`).
        // The runtime override consumer lives in `gfx_quality::apply`
        // (W4-g47 census); it never fires during this drain.
        let mut calls = 0;
        while !self.active.is_empty() && calls < 100_000 {
            calls += 1;
            self.update(world);
        }
        self.quick_settle = false;
        self.water_check(world);
        // `:764` — the load context drops; the runtime sim resumes.
        self.generating = false;
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
            quick_settle: false,
            generating: false,
            generating_world: false,
            // The load-settle pin (`WorldFile.cs:734`): `waterLine =
            // maxTilesY` makes the classic re-type arm dead on load. The gen
            // settle passes overwrite both from the terrain-fill products
            // (R-12-5); the runtime never reads them (`generating` is down).
            gen_water_line: world.height,
            gen_lava_line: 0,
            stuck_count: 0,
            stuck_amount: 0,
            stuck: false,
            panic_counter: 0,
            panic_mode: false,
            panic_y: 0,
            remix_world_gen: false,
            merge_events: Vec::new(),
            edit_kills: Vec::new(),
            // The stock solidity: the grate and the boulders are solid and
            // the bubble is not (`Main.cs:10124`, `Main.cs:17753`).
            ignore_solids: false,
            bubble_solid: false,
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

    /// The `DelWater` retirement-tail kills (`Liquid.cs:1580-1598`) — the
    /// herb/lily-pad `Check*` deaths whose drops the C# spawns inline;
    /// drained by the liquid tick and resolved through the `TileKill`
    /// drop seam.
    pub fn drain_edit_kills(&mut self) -> Vec<crate::edit::TileKill> {
        std::mem::take(&mut self.edit_kills)
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
        // `Liquid.cs:1167-1171` — the overflow threshold is
        // `curMaxLiquid - 1`; `LiquidBuffer.AddBuffer`
        // (`LiquidBuffer.cs:11-20`) caps at `maxLiquidBuffer - 2` and
        // marks the buffered cell `checkingLiquid` so later wakes dedupe
        // against it (the promotion at `:1127` clears the bit first).
        if self.active.len() >= self.cur_max_liquid - 1 {
            if self.buffer.len() < MAX_BUFFER - 2 {
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
        // `Liquid.cs:1005-1054` — the runtime panic gate (the port's sim
        // never runs under generation, so `!isGeneratingOrLoadingWorld`
        // holds): an overflow buffer at ≥ 90% of maxLiquidBuffer for
        // 3600 consecutive updates starts the force-settle; while
        // panicMode is up the pass walks five QuickWater rows bottom-up
        // and returns (the cellular update is skipped).
        if !self.panic_mode {
            if self.buffer.len() as f64 >= MAX_BUFFER as f64 * 0.9 {
                self.panic_counter += 1;
                if self.panic_counter > 3600 {
                    self.start_panic();
                }
            } else {
                self.panic_counter = 0;
            }
        }
        if self.panic_mode {
            let mut rows = 0;
            while self.panic_y >= 3 && rows < 5 {
                rows += 1;
                self.quick_water(world, self.panic_y, self.panic_y);
                self.panic_y -= 1;
                if self.panic_y < 3 {
                    // `:1034-1037` — the settle finished: drop the mode
                    // and re-scan the grid (WaterCheck).
                    self.panic_counter = 0;
                    self.panic_mode = false;
                    self.water_check(world);
                }
            }
            self.ignore_solids = false;
            return;
        }
        // `Liquid.cs:1056-1068` — the pass head derives `quickFall` from
        // `quickSettle` every call. (The `Setting_UseReducedMaxLiquids`
        // `numLiquid > 2000` OR-arm lands with the reduced-liquids field.)
        self.quick_fall = self.quick_settle;
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

    /// `wetCounter >= cycles` bookkeeping (`Liquid.cs:1106-1149`): the
    /// `kill >= 8` retirement sweep (descending, swap-remove `DelWater`
    /// with the 254→255 promotion at `:1113-1116`), the buffer promotion
    /// counts (`:1120-1130`), then the stuck detection (`:1131-1149`).
    fn end_pass(&mut self, world: &mut World) {
        // `Liquid.cs:1109-1119` — the descending retirement walk: a
        // retired cell sitting at 254 promotes to 255 first, then
        // `DelWater`'s own drain rules decide whether it really leaves.
        let mut l = self.active.len();
        while l > 0 {
            l -= 1;
            if self.active[l].kill >= KILL_PASSES {
                let (x, y) = (self.active[l].x, self.active[l].y);
                let i = world.index(x, y);
                if world.tiles[i].liquid == 254 {
                    world.tiles[i].liquid = 255;
                    world.mark_chunk(x, y);
                    self.dirty = true;
                }
                self.del_water(world, l);
            }
        }
        // `Liquid.cs:1120-1130` — `num8 = curMaxLiquid - (curMaxLiquid -
        // numLiquid)` is `numLiquid` (the C#'s own algebra), clamped to
        // the buffer count; each promotion clears the buffered cell's
        // `checkingLiquid` first so `AddWater` re-queues it, then
        // `LiquidBuffer.DelBuffer(0)` swap-removes the slot
        // (`LiquidBuffer.cs:22-27` — the LAST entry moves into slot 0,
        // not a FIFO shift).
        let mut limit = self.active.len().min(self.buffer.len());
        while limit > 0 {
            limit -= 1;
            let (x, y) = self.buffer[0];
            self.set_checking(x, y, false);
            self.add_water(world, x, y);
            let last = self.buffer.len() - 1;
            self.buffer[0] = self.buffer[last];
            self.buffer.pop();
        }
        // `Liquid.cs:1131-1149` — the stuck band: 10 000 consecutive
        // cycles inside ±50 of the pinned count self-clears the whole
        // list (`stuck` up disables `DelWater`'s keep-alive branch).
        let n = self.active.len() as i32;
        if n > 0 && n > self.stuck_amount - 50 && n < self.stuck_amount + 50 {
            self.stuck_count += 1;
            if self.stuck_count >= 10_000 {
                self.stuck = true;
                let mut k = self.active.len();
                while k > 0 {
                    k -= 1;
                    self.del_water(world, k);
                }
                self.stuck = false;
                self.stuck_count = 0;
            }
        } else {
            self.stuck_count = 0;
            self.stuck_amount = n;
        }
    }

    /// `Liquid.DelWater` (`Liquid.cs:1470-1599`) — retire active entry
    /// `l`. The drain rules decide first: a < 2 cell empties itself and
    /// its < 2 side neighbours (waking the fuller ones); a < 20 cell
    /// empties when a side or the cell below can take it; a >= 20 cell
    /// with an open, non-full cell below stays alive (`kill = 0`) unless
    /// the stuck self-clear is running. Then the wake/type bookkeeping,
    /// the lava reaction tail (the grass conversions + `HoneyCheck` /
    /// `ShimmerCheck`), the swap-remove and the herb / lily-pad checks.
    ///
    /// KEEP-ONE (W4-g47 vs W4-g45, ANALYSIS): this body is canonical — the
    /// `Check*` kills ride `edit_kills` (g45's `check_alch_tile`/
    /// `check_lily_pad` wrappers execute the kill but discard the drop
    /// record, losing the herb/lily-pad loot), and the swap-remove lives in
    /// one body the sweep and the stuck self-clear share (g45 splits it into
    /// `retire_sweep`; the two shapes are equivalent). Adopted from g45: the
    /// edge-safe sub-2 probe, the live side re-reads (`:1519-1528` reads the
    /// tile references AFTER the drain), the explicit lily-pad arm order,
    /// and the grass ring's chunk-dirty marking.
    fn del_water(&mut self, world: &mut World, l: usize) {
        let (x, y) = (self.active[l].x, self.active[l].y);
        let i = world.index(x, y);
        let here = world.tiles[i];
        let left = world.tile(x - 1, y).copied().unwrap_or_default();
        let right = world.tile(x + 1, y).copied().unwrap_or_default();
        let down = world.tile(x, y + 1).copied().unwrap_or_default();
        // `Liquid.cs:1478-1498` — the < 2 drain.
        if here.liquid < 2 {
            world.tiles[i].liquid = 0;
            world.mark_chunk(x, y);
            self.dirty = true;
            for nx in [x - 1, x + 1] {
                // The C#'s `Main.tile[nx, y]` is null at the world edge;
                // probe through `tile` before indexing (g45's edge fix,
                // `2249cf59` — the port's `index` would wrap a negative nx).
                let Some(t) = world.tile(nx, y).copied() else { continue };
                if t.liquid < 2 {
                    let j = world.index(nx, y);
                    world.tiles[j].liquid = 0;
                    world.mark_chunk(nx, y);
                } else {
                    self.add_water(world, nx, y);
                }
            }
        } else if here.liquid < 20 {
            // `Liquid.cs:1499-1505` — a side lower than the cell (and
            // open) or an open non-full cell below drains it.
            let side_takes = |t: Tile| t.liquid < here.liquid && self.tile_open(&t);
            let down_takes = down.liquid < 255 && self.tile_open(&down);
            if side_takes(left) || side_takes(right) || down_takes {
                world.tiles[i].liquid = 0;
                world.mark_chunk(x, y);
                self.dirty = true;
            }
        } else if down.liquid < 255
            && self.tile_open(&down)
            && !self.stuck
            && self.tile_open(&here)
        {
            // `Liquid.cs:1506-1510` — the keep-alive: the cell can still
            // fall, so it stays active with a fresh kill count.
            self.active[l].kill = 0;
            return;
        }
        // Re-read: the drain arms above may have emptied the cell.
        let here = world.tiles[i];
        // `Liquid.cs:1511-1518` — wake the cell above a non-full pool;
        // an emptied cell loses its liquid TYPE.
        if here.liquid < 250 && world.tile(x, y - 1).is_some_and(|t| t.liquid > 0) {
            self.add_water(world, x, y - 1);
        }
        if here.liquid == 0 {
            set_kind(&mut world.tiles[i], 0);
        } else {
            // `Liquid.cs:1519-1528` — wake the unequal < 250 open sides.
            // The C# reads the tile REFERENCES here (after the drain arms
            // may have emptied a side), so re-read live rather than reuse
            // the entry snapshots (g45's live-read form).
            for nx in [x + 1, x - 1] {
                let t = world.tile(nx, y).copied().unwrap_or_default();
                if t.liquid > 0
                    && t.liquid < 250
                    && self.tile_open(&t)
                    && here.liquid != t.liquid
                {
                    self.add_water(world, nx, y);
                }
            }
            // `Liquid.cs:1529-1569` — the reaction tail. The lava arm
            // re-checks the merge and converts the grass ring; honey and
            // shimmer only re-check. The kind reads the LIVE cell — the
            // merge may have emptied it (the C# holds the tile reference).
            let tail_kind = kind_code(world.tiles[i].liquid_kind());
            if tail_kind == 1 {
                self.lava_check(world, x, y);
                for gx in (x - 1)..=(x + 1) {
                    for gy in (y - 1)..=(y + 1) {
                        let Some(t) = world.tile(gx, gy).copied() else { continue };
                        if !t.active() {
                            continue;
                        }
                        let j = world.index(gx, gy);
                        if matches!(t.kind, 2 | 23 | 109 | 199 | 477 | 492) {
                            world.tiles[j].kind = 0;
                            // The kind write dirties the chunk for the draw
                            // rebuild (g45's marking; `square_tile_frame`'s
                            // `after_edit` reframes but does not mark).
                            world.mark_chunk(gx, gy);
                            self.dirty = true;
                            crate::edit::square_tile_frame_default(world, gx, gy);
                        } else if matches!(t.kind, 60 | 70 | 661 | 662) {
                            world.tiles[j].kind = 59;
                            world.mark_chunk(gx, gy);
                            self.dirty = true;
                            crate::edit::square_tile_frame_default(world, gx, gy);
                        }
                    }
                }
            } else if tail_kind == 2 {
                self.honey_check(world, x, y);
            } else if tail_kind == 3 {
                self.shimmer_check(world, x, y);
            }
        }
        // `Liquid.cs:1575-1579` — the swap-remove: the last entry moves
        // into the slot (its kill rides along; `delay` is NOT copied —
        // the C# leaves the slot's own value).
        self.set_checking(x, y, false);
        let last = self.active.len() - 1;
        self.active[l].x = self.active[last].x;
        self.active[l].y = self.active[last].y;
        self.active[l].kill = self.active[last].kill;
        self.active.pop();
        // `Liquid.cs:1580-1598` — the herb check, or the lily-pad arm. The
        // kind reads the LIVE cell (`tile4` is a reference — the lava
        // tail's grass ring may have rewritten it). The checks' kills/drops
        // ride the pass's `edit_kills` seam (the C# spawns the items
        // inline; the port's engine is pure).
        let tail_tile_kind = world.tiles[i].kind;
        if crate::tileobject::tile_alch(tail_tile_kind) {
            if let Some(kill) = crate::edit::check_alch(world, x, y) {
                self.edit_kills.push(kill);
            }
        } else if tail_tile_kind == 518 {
            // `Liquid.cs:1586-1597` — the three arms in the C#'s order
            // (g45's explicit form): quickFall → CheckLilyPad; else a
            // non-full below OR a wet above → SquareTileFrame; else
            // CheckLilyPad. The kills ride `edit_kills` (g47's seam — g45's
            // `check_lily_pad` wrapper discards the drop record).
            if self.quick_fall {
                if let Some(kill) =
                    crate::edit::check_lily_pad(world, x, y, |k| self.kind_solid(k))
                {
                    self.edit_kills.push(kill);
                }
            } else {
                let below = world.tile(x, y + 1).copied().unwrap_or_default();
                let above = world.tile(x, y - 1).copied().unwrap_or_default();
                if below.liquid < 255 || above.liquid > 0 {
                    crate::edit::square_tile_frame_default(world, x, y);
                } else if let Some(kill) =
                    crate::edit::check_lily_pad(world, x, y, |k| self.kind_solid(k))
                {
                    self.edit_kills.push(kill);
                }
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
        if self.tile_is_solid(&start) || start.liquid == 0 {
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
                // `Liquid.cs:586-587` — `tile4.skipLiquid(true)` and
                // `tile5.skipLiquid(true)`: the same stride skips both.
                self.set_skip(x, y + 1, true);
                self.set_skip(x, y, true);
                // `Liquid.cs:588-596` — quick-settle clamps the source
                // remainder (> 250) straight to 255 and skips the side
                // wakes; otherwise the two side cells wake (unless the
                // 255-keep-one quirk held the source).
                if self.quick_settle && world.tiles[i].liquid > 250 {
                    world.tiles[i].liquid = 255;
                    world.mark_chunk(x, y);
                    self.dirty = true;
                } else if !keep_one {
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
                // `Liquid.cs:942-952` — the 255→254 settle step counts a
                // kill pass; quick-settle promotes straight back to 255
                // (the pool reads settled a pass earlier).
                if self.quick_settle {
                    world.tiles[i].liquid = 255;
                    world.mark_chunk(x, y);
                    self.dirty = true;
                }
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
    /// The generation-settle context (R-12-5's plumbing seam): arm the
    /// `SettleWaterAt` re-type (`Liquid.cs:178-192`) for a worldgen settle
    /// pass. `water_line`/`lava_line` are the `GenVars` terrain-fill products
    /// (`GenVars.cs:62-64`); `remix` selects the remix band (`:182`) over the
    /// classic boundary (`:184`). The world-LOAD settle never calls this —
    /// it leaves `generating` down and `gen_water_line` at the `maxTilesY`
    /// pin (`WorldFile.cs:734`), so the re-type is inert exactly as the C#
    /// load's `waterLine = maxTilesY` makes it. Runtime settles never arm it.
    pub fn set_gen_settle_context(&mut self, water_line: i32, lava_line: i32, remix: bool) {
        self.generating = true;
        self.generating_world = true;
        self.remix_world_gen = remix;
        self.gen_water_line = water_line;
        self.gen_lava_line = lava_line;
    }

    /// `Liquid.QuickWater` (`Liquid.cs:104-148`) — the bulk "settle this
    /// rectangle": walk the rows bottom-up and settle every wet cell.
    /// `verbose` is the progress-display parameter (no port consumer).
    ///
    /// The outer generation arms are inert in the port, each for a recorded
    /// reason (ANALYSIS W4-g47): `ShimmerRemoveWater` (`:108`/`:138`) early-
    /// returns unless `generatingWorld` (`WorldGen.cs:34825`) and reads
    /// `GenVars.shimmerPosition`, which the compact pipeline never places;
    /// the noTraps `SetBoulderSolidity` swap (`:109-112`/`:139-142`) is
    /// subsumed by the pass-scoped `tilesIgnoreWater(true)` (`:115`) the port
    /// already raises (the same nine boulder kinds, [`BOULDER_SOLIDS`]);
    /// `LiquidInteractionsCleanup` (`:144-147`) is `generatingWorld &&
    /// !skyblock`-gated and strips the gen merge tiles 56/659 — it rides the
    /// unported gen settle driver. The runtime/panic/load settle runs the
    /// solid core below.
    pub fn quick_water(&mut self, world: &mut World, min_y: i32, max_y: i32) {
        self.bubble_solid = true;
        self.ignore_solids = true;
        let min_y = if min_y == -1 { 3 } else { min_y };
        let max_y = if max_y == -1 { self.height - 3 } else { max_y };
        for y in (min_y..=max_y).rev() {
            for x in 4..self.width - 4 {
                if world.tile(x, y).is_some_and(|t| t.liquid != 0) {
                    self.settle_water_at(world, x, y);
                }
            }
        }
        self.ignore_solids = false;
    }

    /// `Liquid.SettleWaterAt` (`Liquid.cs:150-289`) — drain one cell's
    /// liquid straight down, then spread it across the row it lands on,
    /// repeating until the mass is placed. The generation-only re-type arm
    /// (`:178-192`) runs while [`Self::generating`] is up: a dropped water
    /// cell landing below `GenVars.waterLine` (classic) or the remix band
    /// re-types to lava; the dungeon arm (`generatingWorld`-gated) is a
    /// recorded no-op (no ported `InAnyPotentialDungeonBounds`). The runtime
    /// settle keeps `generating` down (arm inert); the world-LOAD settle arms
    /// `generating` but pins `gen_water_line = maxTilesY` (`WorldFile.cs:734`),
    /// so the classic boundary never fires — both keep the origin's type.
    /// The landing cell's merge checks run through the three
    /// `AttemptToMove*` helpers.
    fn settle_water_at(&mut self, world: &mut World, origin_x: i32, origin_y: i32) {
        self.ignore_solids = true;
        let Some(origin) = world.tile(origin_x, origin_y).copied() else {
            self.ignore_solids = false;
            return;
        };
        if origin.liquid == 0 || (origin.active() && origin.kind == 379) {
            self.ignore_solids = false;
            return;
        }
        let mut x = origin_x;
        let mut y = origin_y;
        let had_lava = origin.liquid_kind() == LiquidKind::Lava;
        let had_honey = origin.liquid_kind() == LiquidKind::Honey;
        let had_shimmer = origin.liquid_kind() == LiquidKind::Shimmer;
        let mut mass = origin.liquid as i32;
        let mut kind = kind_code(origin.liquid_kind());
        let oi = world.index(origin_x, origin_y);
        world.tiles[oi].liquid = 0;
        let mut first_drop = true;
        loop {
            // `:169-177` — fall through the empty open cells below. `flag4`
            // is per-iteration in the C# (reset at `:170`), so track it here.
            let mut fell = false;
            while y < self.height - 5 {
                let below = world.tile(x, y + 1).copied().unwrap_or_default();
                if below.liquid == 0 && self.tile_open(&below) {
                    y += 1;
                    first_drop = false;
                    fell = true;
                } else {
                    break;
                }
            }
            // `:178-192` — the generation-only re-type. Honey/shimmer origins
            // keep their type (`!flag && !flag2`); a dropped water cell
            // re-types to lava below the boundary. The world-load settle
            // pins `waterLine = maxTilesY` (`WorldFile.cs:734`), so the
            // classic arm is dead there; the runtime never enters (`generating`
            // down). The remix arm reads `lavaLine`/`rockLayer`/`oceanDepths`.
            if self.generating && fell && !had_honey && !had_shimmer {
                if self.remix_world_gen {
                    // `:182` — the remix band: below lavaLine AND (above
                    // rockLayer-80 OR near the bottom), outside the ocean.
                    let rock = world.rock_layer;
                    kind = if (y as f64) > self.gen_lava_line as f64
                        && ((y as f64) < rock - 80.0 || y > self.height - 350)
                        && !crate::scene::ocean_depths(
                            world.world_surface,
                            world.rock_layer,
                            x,
                            self.width,
                            y,
                        ) {
                        1
                    } else {
                        0
                    };
                } else if y > self.gen_water_line {
                    // `:184-187` — the classic waterLine boundary.
                    kind = 1;
                }
                // `:188-191` — the dungeon re-type back to water is gated on
                // `generatingWorld` AND `InAnyPotentialDungeonBounds` +
                // `IsConsideredDungeonWall`; neither predicate is ported
                // (ANALYSIS W4-g47), so the arm is a recorded no-op.
            }
            // `:193-272` — the row spread: alternate left/right from the
            // landing column, topping up same-type non-full cells below
            // and stopping at walls / wet cells.
            let mut dir = -1i32;
            let mut step = 0i32;
            let mut last_dir = -1i32;
            let mut last_step = 0i32;
            let mut hit_right = false;
            let mut hit_left = false;
            let mut can_fall = false;
            loop {
                let cx = x + step * dir;
                let cell = world.tile(cx, y).copied().unwrap_or_default();
                if cell.liquid == 0 {
                    last_dir = dir;
                    last_step = step;
                }
                if dir == -1 && cx < 5 {
                    hit_left = true;
                } else if dir == 1 && cx > self.width - 5 {
                    hit_right = true;
                }
                let below = world.tile(cx, y + 1).copied().unwrap_or_default();
                if below.liquid != 0
                    && below.liquid != 255
                    && kind_code(below.liquid_kind()) == kind
                {
                    let mut take = 255 - below.liquid as i32;
                    if take > mass {
                        take = mass;
                    }
                    let j = world.index(cx, y + 1);
                    world.tiles[j].liquid += take as u8;
                    world.mark_chunk(cx, y + 1);
                    mass -= take;
                    if mass == 0 {
                        break;
                    }
                }
                if y < self.height - 5 && below.liquid == 0 && self.tile_open(&below) {
                    can_fall = true;
                    break;
                }
                let next = world.tile(x + (step + 1) * dir, y).copied().unwrap_or_default();
                if (next.liquid != 0 && (!first_drop || dir != 1))
                    || (next.active() && self.kind_solid(next.kind) && !td::TILE_SOLID_TOP.has(next.kind))
                {
                    if dir == 1 {
                        hit_right = true;
                    } else {
                        hit_left = true;
                    }
                }
                if hit_left && hit_right {
                    break;
                }
                if hit_right {
                    dir = -1;
                    step += 1;
                } else if hit_left {
                    if dir == 1 {
                        step += 1;
                    }
                    dir = 1;
                } else {
                    if dir == 1 {
                        step += 1;
                    }
                    dir = -dir;
                }
            }
            x += last_step * last_dir;
            if mass == 0 || !can_fall {
                break;
            }
            y += 1;
        }
        // `:280-287` — place the remaining mass and run the merge checks.
        let li = world.index(x, y);
        world.tiles[li].liquid = mass as u8;
        set_kind(&mut world.tiles[li], kind);
        world.mark_chunk(x, y);
        self.dirty = true;
        if world.tiles[li].liquid > 0 {
            self.attempt_to_move_lava(world, x, y, had_lava);
            self.attempt_to_move_honey(world, x, y, had_honey);
            self.attempt_to_move_shimmer(world, x, y, had_shimmer);
        }
        self.ignore_solids = false;
    }

    /// `Liquid.AttemptToMoveLava` (`Liquid.cs:339-385`) — the landing
    /// cell's lava merge check against the first different-type
    /// neighbour in left/right/up/down order.
    fn attempt_to_move_lava(&mut self, world: &mut World, x: i32, y: i32, had_lava: bool) {
        for (nx, ny) in [(x - 1, y), (x + 1, y), (x, y - 1), (x, y + 1)] {
            let t = world.tile(nx, ny).copied().unwrap_or_default();
            if t.liquid > 0 && (t.liquid_kind() == LiquidKind::Lava) != had_lava {
                if had_lava {
                    self.lava_check(world, x, y);
                } else {
                    self.lava_check(world, nx, ny);
                }
                return;
            }
        }
    }

    /// `Liquid.AttemptToMoveHoney` (`Liquid.cs:291-337`).
    fn attempt_to_move_honey(&mut self, world: &mut World, x: i32, y: i32, had_honey: bool) {
        for (nx, ny) in [(x - 1, y), (x + 1, y), (x, y - 1), (x, y + 1)] {
            let t = world.tile(nx, ny).copied().unwrap_or_default();
            if t.liquid > 0 && (t.liquid_kind() == LiquidKind::Honey) != had_honey {
                if had_honey {
                    self.honey_check(world, x, y);
                } else {
                    self.honey_check(world, nx, ny);
                }
                return;
            }
        }
    }

    /// `Liquid.AttemptToMoveShimmer` (`Liquid.cs:387-433`).
    fn attempt_to_move_shimmer(&mut self, world: &mut World, x: i32, y: i32, had_shimmer: bool) {
        for (nx, ny) in [(x - 1, y), (x + 1, y), (x, y - 1), (x, y + 1)] {
            let t = world.tile(nx, ny).copied().unwrap_or_default();
            if t.liquid > 0 && (t.liquid_kind() == LiquidKind::Shimmer) != had_shimmer {
                if had_shimmer {
                    self.shimmer_check(world, x, y);
                } else {
                    self.shimmer_check(world, nx, ny);
                }
                return;
            }
        }
    }

    /// `WorldGen.WaterCheck` (`WorldGen.cs:80415-80494`) — the post-settle
    /// grid scan: drop the cellular lists, wipe `checkingLiquid`, zero the
    /// liquid inside solid tiles (the bubble 379 keeps its own), kill the
    /// liquid-death objects, and re-wake every cell that can still flow.
    /// The liquid-death kills ride the `edit_kills` seam.
    pub fn water_check(&mut self, world: &mut World) {
        self.ignore_solids = true;
        self.active.clear();
        self.buffer.clear();
        for x in 1..self.width - 1 {
            for y in (1..self.height - 1).rev() {
                self.set_checking(x, y, false);
                let i = world.index(x, y);
                let t = world.tiles[i];
                if t.liquid == 0 {
                    continue;
                }
                if crate::tileobject::nactive(&t)
                    && self.kind_solid(t.kind)
                    && !td::TILE_SOLID_TOP.has(t.kind)
                {
                    // `:80426-80432` — liquid inside a solid tile zeroes
                    // (the bubble 379 keeps its own).
                    if t.kind != 379 {
                        world.tiles[i].liquid = 0;
                        world.mark_chunk(x, y);
                    }
                } else {
                    // `:80433-80490` — the open-cell arm.
                    // `:80435-80448` — the liquid-death object kill.
                    if t.active() {
                        let dies = if t.liquid_kind() == LiquidKind::Lava {
                            crate::tileobject::check_lava_death(&t)
                        } else {
                            crate::tileobject::check_water_death(&t)
                        };
                        if dies {
                            if let Some(kill) = crate::edit::kill_tile(world, x, y, false) {
                                self.edit_kills.push(kill);
                            }
                        }
                    }
                    // `:80449-80489` — the wake arms.
                    let below = world.tile(x, y + 1).copied().unwrap_or_default();
                    if self.tile_open(&below) && below.liquid < 255 {
                        if below.liquid > 250 {
                            let j = world.index(x, y + 1);
                            world.tiles[j].liquid = 255;
                            world.mark_chunk(x, y + 1);
                        } else {
                            self.add_water(world, x, y);
                        }
                    }
                    let left = world.tile(x - 1, y).copied().unwrap_or_default();
                    let right = world.tile(x + 1, y).copied().unwrap_or_default();
                    if self.tile_open(&left) && left.liquid != t.liquid {
                        self.add_water(world, x, y);
                    } else if self.tile_open(&right) && right.liquid != t.liquid {
                        self.add_water(world, x, y);
                    }
                    if t.liquid_kind() == LiquidKind::Lava {
                        let up = world.tile(x, y - 1).copied().unwrap_or_default();
                        if (left.liquid > 0 && left.liquid_kind() != LiquidKind::Lava)
                            || (right.liquid > 0 && right.liquid_kind() != LiquidKind::Lava)
                            || (up.liquid > 0 && up.liquid_kind() != LiquidKind::Lava)
                            || (below.liquid > 0 && below.liquid_kind() != LiquidKind::Lava)
                        {
                            self.add_water(world, x, y);
                        }
                    }
                }
            }
        }
        self.ignore_solids = false;
    }
}

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
        self.kind_solid(t.kind) && !td::TILE_SOLID_TOP.has(t.kind)
    }

    /// The raw `Main.tileSolid[type]` read with the pass-scoped toggles —
    /// `DelWater`'s drain rules and `CheckLilyPad`'s support walk read the
    /// table directly (no `nactive`/`solidTop` fold), so the
    /// grate/boulder/bubble swaps must ride here too.
    #[inline]
    fn kind_solid(&self, kind: u16) -> bool {
        if kind as usize >= td::TILE_COUNT {
            return false;
        }
        if self.ignore_solids && (kind == 546 || BOULDER_SOLIDS.contains(&kind)) {
            return false;
        }
        if kind == 379 {
            return self.bubble_solid;
        }
        td::TILE_SOLID.has(kind)
    }

    /// `!tile.nactive() || !Main.tileSolid[tile.type] ||
    /// Main.tileSolidTop[tile.type]` — the "open for liquid" test
    /// `DelWater`'s drain rules use (`Liquid.cs:1501`/`:1506`/`:1521`).
    #[inline]
    fn tile_open(&self, t: &Tile) -> bool {
        !t.active()
            || t.flags & crate::tile::FLAG_INACTIVE != 0
            || !self.kind_solid(t.kind)
            || td::TILE_SOLID_TOP.has(t.kind)
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

/// W4-g47 — the quick-settle witnesses: the two `Liquid.Update` arms
/// (`Liquid.cs:588-591`/`:944-948`) and the pass head's `quickFall`
/// derivation (`:1056-1068`). Written, NOT run (code-only mode).
#[cfg(test)]
mod liquid_quick_settle {
    use super::*;
    use crate::tile::FLAG_ACTIVE;

    /// A walled shaft: stone at `x = cx-1`/`cx+1` down to the floor row,
    /// so only the down arm can run (the `mass_conserved` fixture shape).
    fn shaft(width: i32, height: i32, floor_y: i32, cx: i32) -> World {
        let mut w = World::new("liquid_qs".into(), width, height);
        for y in 0..=floor_y {
            for x in [cx - 1, cx + 1] {
                let i = w.index(x, y);
                w.tiles[i] = Tile { kind: 1, ..Default::default() };
                w.tiles[i].flags |= FLAG_ACTIVE;
            }
        }
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
    fn quick_settle_clamps_the_fall_remainder_to_255() {
        // `Liquid.cs:588-591`: after the down transfer the source holds
        // 253 (> 250) — quick-settle writes 255 back (the vanilla
        // settle-mode clone quirk); the normal pass keeps 253.
        let mut w = shaft(5, 40, 30, 2);
        put_as(&mut w, 2, 10, 255, 0);
        put_as(&mut w, 2, 11, 253, 0);
        let mut sys = LiquidSystem::new(&w);
        sys.set_quick_settle(true);
        sys.add_water(&w, 2, 10);
        sys.add_water(&w, 2, 11);
        sys.update(&mut w);
        assert_eq!(w.tiles[w.index(2, 10)].liquid, 255, ":590 clamps > 250");
        assert_eq!(w.tiles[w.index(2, 11)].liquid, 255, "the fed cell filled");

        let mut w2 = shaft(5, 40, 30, 2);
        put_as(&mut w2, 2, 10, 255, 0);
        put_as(&mut w2, 2, 11, 253, 0);
        let mut sys2 = LiquidSystem::new(&w2);
        sys2.add_water(&w2, 2, 10);
        sys2.add_water(&w2, 2, 11);
        sys2.update(&mut w2);
        assert_eq!(w2.tiles[w2.index(2, 10)].liquid, 253, "no clamp at runtime");
    }

    #[test]
    fn quick_settle_promotes_the_254_settle_step() {
        // `Liquid.cs:942-952`: the row arm averages 255 with a 253
        // neighbour to 254 — the 255→254 settle step. Quick-settle
        // promotes the cell straight back to 255 (`:946`); the runtime
        // pass keeps the 254 for the retirement sweep's own promotion.
        let basin = |w: &mut World| {
            for x in 0..9 {
                for y in 8..11 {
                    let i = w.index(x, y);
                    w.tiles[i] = Tile { kind: 1, ..Default::default() };
                    w.tiles[i].flags |= FLAG_ACTIVE;
                }
            }
            let wall = w.index(5, 7);
            w.tiles[wall] = Tile { kind: 1, ..Default::default() };
            w.tiles[wall].flags |= FLAG_ACTIVE;
            w.frame_important = vec![false; td::TILE_COUNT];
            w.compute_column_surface();
            w.compute_chunks();
            put_as(w, 4, 7, 255, 0);
            put_as(w, 3, 7, 253, 0);
        };
        let mut w = World::new("liquid_qs".into(), 9, 11);
        basin(&mut w);
        let mut sys = LiquidSystem::new(&w);
        sys.set_quick_settle(true);
        sys.add_water(&w, 4, 7);
        sys.update(&mut w);
        assert_eq!(w.tiles[w.index(4, 7)].liquid, 255, ":946 promotes 254");
        assert_eq!(w.tiles[w.index(3, 7)].liquid, 254, "the neighbour averaged");

        let mut w2 = World::new("liquid_qs".into(), 9, 11);
        basin(&mut w2);
        let mut sys2 = LiquidSystem::new(&w2);
        sys2.add_water(&w2, 4, 7);
        sys2.update(&mut w2);
        assert_eq!(w2.tiles[w2.index(4, 7)].liquid, 254, "runtime keeps the step");
    }

    #[test]
    fn the_pass_head_derives_quick_fall_from_quick_settle() {
        // `Liquid.cs:1056-1068` + `:1083-1090`: quickSettle forces
        // quickFall, whose loop sets delay = 10 — the lava column falls
        // on the FIRST visit instead of waiting out the 5-visit gate.
        let mut w = shaft(5, 40, 30, 2);
        put_as(&mut w, 2, 0, 255, 1);
        let mut sys = LiquidSystem::new(&w);
        sys.set_quick_settle(true);
        sys.add_water(&w, 2, 0);
        sys.update(&mut w);
        assert!(sys.quick_fall, ":1063 sets quickFall");
        assert_eq!(w.tiles[w.index(2, 1)].liquid, 255, "the delay gate is bypassed");
        sys.set_quick_settle(false);
        sys.update(&mut w);
        assert!(!sys.quick_fall, ":1067 clears quickFall");
    }
}

/// W4-g47 — the settle-side witnesses: `DelWater`'s drain rules and
/// swap-remove (`Liquid.cs:1470-1599`), the retirement sweep's 254→255
/// promotion (`:1109-1119`), the `LiquidBuffer.AddBuffer`/`DelBuffer`
/// counts (`LiquidBuffer.cs:11-27`), and the stuck band's 10 000-cycle
/// self-clear (`:1131-1149`). Written, NOT run (code-only mode).
#[cfg(test)]
mod liquid_settle {
    use super::*;
    use crate::tile::FLAG_ACTIVE;

    fn basin(width: i32, wall_y: i32) -> World {
        let mut w = World::new("liquid_settle".into(), width, wall_y + 3);
        for x in 0..width {
            for y in wall_y..wall_y + 3 {
                let i = w.index(x, y);
                w.tiles[i] = Tile { kind: 1, ..Default::default() };
                w.tiles[i].flags |= FLAG_ACTIVE;
            }
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
    fn the_retirement_sweep_promotes_254_before_delwater() {
        // `Liquid.cs:1113-1116`: a retired cell sitting at 254 becomes
        // 255 first. A 254 cell directly over the floor in a sealed cup
        // is static (the fall sees a solid below, the row arms see
        // walls), so eight unchanged walks retire it and the sweep's
        // promotion lands before DelWater reads it.
        let mut w = basin(9, 8);
        // A sealed 1-wide cup: walls at x=3/5 up to y=7, floor at y=8.
        for y in 0..8 {
            for x in [3, 5] {
                let i = w.index(x, y);
                w.tiles[i] = Tile { kind: 1, ..Default::default() };
                w.tiles[i].flags |= FLAG_ACTIVE;
            }
        }
        put_as(&mut w, 4, 7, 254, 0);
        let mut sys = LiquidSystem::new(&w);
        sys.add_water(&w, 4, 7);
        for _ in 0..8 {
            sys.update(&mut w);
        }
        assert_eq!(w.tiles[w.index(4, 7)].liquid, 255, ":1115 promotes the 254");
        assert_eq!(sys.active_len(), 0, "the entry retired");
    }

    #[test]
    fn delwater_drains_a_thin_film_into_the_open_side() {
        // `Liquid.cs:1499-1504`: a < 20 cell empties when an open side
        // holds less. DelWater is called directly (the retirement tail's
        // entry point) so the row arm never spreads the film first.
        let mut w = basin(9, 8);
        put_as(&mut w, 4, 7, 10, 2); // honey — the type-clear is visible
        let mut sys = LiquidSystem::new(&w);
        sys.add_water(&w, 4, 7);
        sys.active[0].kill = KILL_PASSES;
        sys.del_water(&mut w, 0);
        assert_eq!(w.tiles[w.index(4, 7)].liquid, 0, ":1503 empties the film");
        assert_eq!(sys.active_len(), 0, "the entry is gone");
        assert_eq!(
            w.tiles[w.index(4, 7)].liquid_kind(),
            LiquidKind::Water,
            "an emptied cell loses its type (:1517)"
        );
    }

    #[test]
    fn delwater_keeps_a_full_cell_over_an_open_below_alive() {
        // `Liquid.cs:1506-1509`: a >= 20 cell with an open non-full cell
        // below resets kill and stays active — the fall has work left.
        let mut w = basin(9, 8);
        put_as(&mut w, 4, 5, 200, 0);
        let mut sys = LiquidSystem::new(&w);
        sys.add_water(&w, 4, 5);
        sys.active[0].kill = KILL_PASSES;
        sys.del_water(&mut w, 0);
        assert_eq!(sys.active_len(), 1, "the keep-alive holds the entry");
        assert_eq!(sys.active[0].kill, 0, ":1508 resets the kill count");
        assert_eq!(w.tiles[w.index(4, 5)].liquid, 200, "nothing drained");
    }

    #[test]
    fn delwater_swap_remove_rides_kill_but_not_delay() {
        // `Liquid.cs:1575-1579`: the last entry's x/y/kill move into the
        // retired slot — `delay` keeps the SLOT's value (the C# never
        // copies it).
        let mut w = basin(9, 8);
        // Two sealed cups so both cells are static; the second is the
        // swap donor.
        for y in 0..8 {
            for x in [3, 5, 7] {
                let i = w.index(x, y);
                w.tiles[i] = Tile { kind: 1, ..Default::default() };
                w.tiles[i].flags |= FLAG_ACTIVE;
            }
        }
        put_as(&mut w, 4, 7, 100, 0);
        put_as(&mut w, 6, 7, 100, 2);
        let mut sys = LiquidSystem::new(&w);
        sys.add_water(&w, 4, 7);
        sys.add_water(&w, 6, 7);
        sys.active[0].kill = KILL_PASSES;
        sys.active[0].delay = 3;
        sys.active[1].kill = 5;
        sys.active[1].delay = 9;
        sys.del_water(&mut w, 0);
        assert_eq!(sys.active_len(), 1);
        assert_eq!((sys.active[0].x, sys.active[0].y), (6, 7), "the donor moved");
        assert_eq!(sys.active[0].kill, 5, "the donor's kill rides along");
        assert_eq!(sys.active[0].delay, 3, "the slot's own delay stays");
    }

    #[test]
    fn the_buffer_overflow_marks_checking_and_the_promotion_swap_removes() {
        // `LiquidBuffer.cs:11-27`: AddBuffer caps at maxLiquidBuffer - 2
        // and marks checkingLiquid; the promotion (`Liquid.cs:1125-1130`)
        // clears the bit, re-queues, and DelBuffer(0) moves the LAST
        // buffered entry into slot 0 (not a FIFO shift).
        let mut w = basin(9, 8);
        // The active cell sits in a sealed cup so the pass walks it
        // unchanged (no new wakes pollute the buffer).
        for y in 0..8 {
            for x in [3, 5] {
                let i = w.index(x, y);
                w.tiles[i] = Tile { kind: 1, ..Default::default() };
                w.tiles[i].flags |= FLAG_ACTIVE;
            }
        }
        put_as(&mut w, 4, 7, 100, 0);
        put_as(&mut w, 2, 7, 100, 0);
        put_as(&mut w, 6, 7, 100, 0);
        let mut sys = LiquidSystem::new(&w);
        // Shrink the window so the second wake overflows.
        sys.set_cur_max_liquid(2);
        sys.add_water(&w, 4, 7);
        sys.add_water(&w, 2, 7); // active full (len 1 >= 2 - 1) -> buffer
        sys.add_water(&w, 6, 7); // -> buffer
        assert_eq!(sys.active_len(), 1);
        assert_eq!(sys.buffered_len(), 2);
        assert!(sys.is_checking(2, 7), "AddBuffer marks the bit");
        assert!(sys.is_checking(6, 7));
        // The dedupe: a buffered cell does not queue twice.
        sys.add_water(&w, 2, 7);
        assert_eq!(sys.buffered_len(), 2);
        // Restore the window so the promotion can actually queue, and
        // drive one full cycle: num8 = min(numLiquid 1, buffer 2) = 1 —
        // slot 0 promotes, DelBuffer(0) moves the LAST entry into it.
        sys.set_cur_max_liquid(MAX_ACTIVE);
        sys.set_cycles(1);
        sys.update(&mut w);
        assert_eq!(sys.buffered_len(), 1, "one promotion drained the head");
        assert_eq!(sys.buffer[0], (6, 7), "the last entry moved into slot 0");
        assert_eq!(sys.active_len(), 2, "the promoted cell re-queued");
        assert!(sys.is_checking(2, 7), "the promoted cell is active again");
        assert!(sys.is_checking(6, 7), "the still-buffered cell keeps its bit");
    }

    #[test]
    fn the_stuck_band_self_clears_after_10000_cycles() {
        // `Liquid.cs:1131-1149`: an active count pinned inside ±50 of
        // stuckAmount for 10 000 consecutive cycles force-retires the
        // whole list with `stuck` up (the keep-alive branch disabled).
        let mut w = basin(9, 8);
        put_as(&mut w, 4, 5, 200, 0);
        let mut sys = LiquidSystem::new(&w);
        sys.add_water(&w, 4, 5);
        // Pin the band: the count (1) sits inside ±50 of the pinned 1.
        sys.stuck_amount = 1;
        sys.stuck_count = 9_999;
        // Age the entry into the retirement sweep; the keep-alive holds
        // it there (open below, `stuck` still down), then the band check
        // trips the 10 000th cycle and the self-clear runs with `stuck`
        // up — the keep-alive branch disabled.
        sys.active[0].kill = KILL_PASSES;
        sys.end_pass(&mut w);
        assert_eq!(sys.active_len(), 0, "the self-clear emptied the list");
        assert_eq!(sys.stuck_count, 0, ":1142 resets the counter");
        assert!(!sys.stuck, ":1141 drops the flag after the clear");
        // The >= 20 cell had an open below — without `stuck` it would
        // have stayed alive; with it the tail ran and the cell kept its
        // liquid (no side/below drain applies at >= 20).
        assert_eq!(w.tiles[w.index(4, 5)].liquid, 200);
    }

    #[test]
    fn the_stuck_band_repins_when_the_count_leaves_it() {
        // `Liquid.cs:1145-1148`: outside the band the counter resets and
        // stuckAmount re-pins to the live count.
        let mut w = basin(9, 8);
        put_as(&mut w, 4, 7, 100, 0);
        let mut sys = LiquidSystem::new(&w);
        sys.add_water(&w, 4, 7);
        sys.stuck_amount = 500; // the count 1 is far outside ±50
        sys.stuck_count = 42;
        sys.end_pass(&mut w);
        assert_eq!(sys.stuck_count, 0, "the band exit resets the counter");
        assert_eq!(sys.stuck_amount, sys.active_len() as i32, "re-pinned");
    }

    #[test]
    fn delwater_retires_a_submerged_herb_through_check_alch() {
        // `Liquid.cs:1580-1583` + `WorldGen.cs:46002-46005`: a Daybloom
        // (82, style 0) whose own cell holds lava dies when its liquid
        // entry retires; the kill rides the edit_kills seam.
        let mut w = basin(9, 8);
        let i = w.index(4, 7);
        w.tiles[i] = Tile { kind: 82, frame_x: 0, ..Default::default() };
        w.tiles[i].flags |= FLAG_ACTIVE;
        put_as(&mut w, 4, 7, 30, 1);
        // The soil below is the basin floor (stone 1) — not in the
        // style-0 set either, so the herb dies on the soil arm as well.
        let mut sys = LiquidSystem::new(&w);
        sys.add_water(&w, 4, 7);
        sys.active[0].kill = KILL_PASSES;
        // The lava reaction tail merges nothing here (no water nearby).
        sys.del_water(&mut w, 0);
        assert!(!w.tiles[w.index(4, 7)].active(), "the herb died");
        let kills = sys.drain_edit_kills();
        assert_eq!(kills.len(), 1, "one CheckAlch kill record");
        assert!(
            kills[0].herb_events.iter().any(|h| h.kind == 82 && h.x == 4 && h.y == 7),
            "the herb drop roll rides the record"
        );
    }

    #[test]
    fn delwater_sinks_a_dry_lily_pad_onto_the_full_water_below() {
        // `Liquid.cs:1590-1597`: a retired 518 cell whose below is full
        // (255) and whose above is dry takes the CheckLilyPad arm; the
        // pad's own cell is dry, so it sinks onto the cell below
        // (`WorldGen.cs:59275-59291`) when that cell is not active. The
        // support soil (grass 2) picks frameY 0 (:59229-59231). The world
        // must be taller than y + 50 — the support walk's `maxTilesY - 50`
        // bound (:59219) otherwise stops at the pad itself.
        let mut w = basin(9, 60);
        let pad = w.index(4, 6);
        w.tiles[pad] = Tile { kind: 518, frame_x: 0, frame_y: 0, ..Default::default() };
        w.tiles[pad].flags |= FLAG_ACTIVE;
        put_as(&mut w, 4, 7, 255, 0);
        let soil = w.index(4, 8);
        w.tiles[soil] = Tile { kind: 2, ..Default::default() };
        w.tiles[soil].flags |= FLAG_ACTIVE;
        let mut sys = LiquidSystem::new(&w);
        // The pad cell carries a token 1 so it queues (the C# reaches
        // this tail when a pad cell was queued while wet and then
        // dried); the < 2 drain empties it before the lily-pad arm.
        w.tiles[pad].liquid = 1;
        sys.add_water(&w, 4, 6);
        sys.active[0].kill = KILL_PASSES;
        sys.del_water(&mut w, 0);
        assert_eq!(w.tiles[w.index(4, 7)].kind, 518, "the pad sank onto the water");
        assert!(w.tiles[w.index(4, 7)].active());
        assert_eq!(w.tiles[w.index(4, 7)].frame_y, 0, "the grass row");
        assert!(!w.tiles[pad].active(), "the origin deactivated");
        assert_eq!(w.tiles[pad].kind, 0);
    }
}

/// W4-g47 — the force-settle witnesses: `QuickWater`/`SettleWaterAt`
/// (`Liquid.cs:104-289`), the `AttemptToMove*` merge checks (`:291-433`),
/// `WaterCheck` (`WorldGen.cs:80415-80494`), `ReInit` (`Liquid.cs:83-102`)
/// and the panic force-settle (`:966-1054`). Written, NOT run (code-only).
#[cfg(test)]
mod liquid_force_settle {
    use super::*;
    use crate::tile::FLAG_ACTIVE;

    /// A tall walled shaft so the settle's `maxTilesY - 5` fall bound and
    /// the `x in [4, width-4)` column range both have room.
    fn shaft(width: i32, height: i32, floor_y: i32, cx: i32) -> World {
        let mut w = World::new("liquid_fs".into(), width, height);
        for y in 0..=floor_y {
            for x in [cx - 1, cx + 1] {
                let i = w.index(x, y);
                w.tiles[i] = Tile { kind: 1, ..Default::default() };
                w.tiles[i].flags |= FLAG_ACTIVE;
            }
        }
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
    fn settle_water_at_drops_the_blob_to_the_floor() {
        // `Liquid.cs:169-177` + `:280`: the origin's mass falls through
        // the empty open column and lands whole on the floor row.
        let mut w = shaft(9, 60, 50, 4);
        put_as(&mut w, 4, 10, 200, 0);
        let mut sys = LiquidSystem::new(&w);
        sys.settle_water_at(&mut w, 4, 10);
        assert_eq!(w.tiles[w.index(4, 10)].liquid, 0, "the origin drained");
        assert_eq!(w.tiles[w.index(4, 49)].liquid, 200, "the blob landed on the floor");
        let total: i32 = w.tiles.iter().map(|t| t.liquid as i32).sum();
        assert_eq!(total, 200, "the settle conserves the mass");
    }

    #[test]
    fn settle_water_at_tops_up_the_partial_cell_below() {
        // `Liquid.cs:216-229`: a same-type non-full cell below takes
        // `min(255 - below, mass)` before the row spread runs.
        let mut w = shaft(9, 60, 50, 4);
        put_as(&mut w, 4, 10, 100, 0);
        put_as(&mut w, 4, 49, 200, 0);
        let mut sys = LiquidSystem::new(&w);
        sys.settle_water_at(&mut w, 4, 10);
        assert_eq!(w.tiles[w.index(4, 49)].liquid, 255, "the partial cell filled");
        // The remaining 45 lands on the row above (the fall stops at the
        // now-full cell, the spread finds no open side, mass places at the
        // landing column).
        let total: i32 = w.tiles.iter().map(|t| t.liquid as i32).sum();
        assert_eq!(total, 300, "the mass is conserved across the top-up");
    }

    #[test]
    fn settle_water_at_keeps_the_bubble_and_skips_empty() {
        // `Liquid.cs:154-157`: an empty cell or an active bubble 379 is
        // left untouched.
        let mut w = shaft(9, 60, 50, 4);
        let b = w.index(4, 20);
        w.tiles[b] = Tile { kind: 379, ..Default::default() };
        w.tiles[b].flags |= FLAG_ACTIVE;
        w.tiles[b].liquid = 100;
        let mut sys = LiquidSystem::new(&w);
        sys.settle_water_at(&mut w, 4, 20);
        assert_eq!(w.tiles[b].liquid, 100, "the bubble keeps its liquid");
        sys.settle_water_at(&mut w, 4, 30); // empty
        assert_eq!(w.tiles[w.index(4, 30)].liquid, 0);
    }

    #[test]
    fn quick_water_settles_the_whole_column() {
        // `Liquid.cs:124-134`: the bottom-up row walk settles every wet
        // cell in the rectangle.
        let mut w = shaft(9, 60, 50, 4);
        put_as(&mut w, 4, 10, 100, 0);
        put_as(&mut w, 4, 12, 100, 0);
        let mut sys = LiquidSystem::new(&w);
        sys.quick_water(&mut w, 3, 50);
        // Both blobs land on the floor row, stacking to 200.
        assert_eq!(w.tiles[w.index(4, 49)].liquid, 200);
        assert_eq!(w.tiles[w.index(4, 10)].liquid, 0);
        assert_eq!(w.tiles[w.index(4, 12)].liquid, 0);
    }

    #[test]
    fn the_landing_merge_check_fires_on_the_different_type_side() {
        // `Liquid.cs:282-287` + `:339-351`: the landing cell's lava check
        // runs against a different-type side neighbour — water landing
        // beside lava merges into obsidian at the lava cell.
        let mut w = shaft(11, 60, 50, 5);
        // Widen the floor row so the lava sits beside the landing column.
        put_as(&mut w, 6, 49, 30, 1);
        put_as(&mut w, 5, 10, 30, 0);
        let mut sys = LiquidSystem::new(&w);
        sys.settle_water_at(&mut w, 5, 10);
        // The water lands at (5,49); the lava at (6,49) is a different-type
        // side, so AttemptToMoveLava checks the lava cell -> obsidian.
        assert_eq!(w.tiles[w.index(6, 49)].kind, 56, "the lava cell merged");
    }

    #[test]
    fn water_check_zeroes_liquid_inside_solids_and_wakes_the_open() {
        // `WorldGen.cs:80426-80432` + `:80449-80460`: liquid inside a
        // solid tile zeroes; an open cell with a non-full open below wakes.
        let mut w = shaft(9, 60, 50, 4);
        // Liquid trapped inside a floor stone cell.
        let s = w.index(4, 50);
        w.tiles[s].liquid = 100;
        // An open wet cell above the floor.
        put_as(&mut w, 4, 48, 100, 0);
        let mut sys = LiquidSystem::new(&w);
        sys.water_check(&mut w);
        assert_eq!(w.tiles[s].liquid, 0, "the solid's liquid zeroed");
        assert!(sys.active_len() > 0, "the open cell woke");
        assert!(sys.is_checking(4, 48), "the woken cell is marked");
    }

    #[test]
    fn water_check_kills_a_submerged_water_death_object() {
        // `WorldGen.cs:80435-80448`: a torch (4, WaterDeath) under water
        // dies; the kill rides the edit_kills seam.
        let mut w = shaft(9, 60, 50, 4);
        let t = w.index(4, 48);
        w.tiles[t] = Tile { kind: 4, ..Default::default() };
        w.tiles[t].flags |= FLAG_ACTIVE;
        w.tiles[t].liquid = 100;
        let mut sys = LiquidSystem::new(&w);
        sys.water_check(&mut w);
        assert!(!w.tiles[t].active(), "the torch died");
        assert!(!sys.drain_edit_kills().is_empty(), "the kill record queued");
    }

    #[test]
    fn re_init_resets_the_cadence_and_lists() {
        // `Liquid.cs:83-102`: every counter/window/list back to stock.
        let mut w = shaft(9, 60, 50, 4);
        put_as(&mut w, 4, 10, 100, 0);
        let mut sys = LiquidSystem::new(&w);
        sys.add_water(&w, 4, 10);
        sys.set_cycles(3);
        sys.set_cur_max_liquid(100);
        sys.set_quick_settle(true);
        sys.stuck_count = 5;
        sys.panic_counter = 9;
        sys.re_init();
        assert_eq!(sys.active_len(), 0);
        assert_eq!(sys.buffered_len(), 0);
        assert_eq!(sys.cycles(), CYCLES);
        assert_eq!(sys.cur_max_liquid(), MAX_ACTIVE);
        assert!(!sys.quick_settle());
        assert_eq!(sys.stuck_count, 0);
        assert_eq!(sys.panic_counter, 0);
        assert!(!sys.is_checking(4, 10), "the checking bit cleared");
    }

    #[test]
    fn the_panic_settle_drains_five_rows_per_update() {
        // `Liquid.cs:1022-1054`: panicMode walks five QuickWater rows
        // bottom-up per update; reaching row 3 drops the mode.
        let mut w = shaft(9, 60, 50, 4);
        put_as(&mut w, 4, 49, 100, 0);
        let mut sys = LiquidSystem::new(&w);
        sys.panic_mode = true;
        sys.panic_y = 8; // five rows: 8,7,6,5,4 -> panic_y lands at 3
        sys.update(&mut w);
        assert!(!sys.panic_mode, "the settle finished at row 3");
        assert_eq!(sys.panic_y, 3);
        // The blob at row 49 was below the walked band, so it stays; the
        // mode-drop WaterCheck woke it.
        assert!(sys.active_len() > 0, "WaterCheck re-woke the grid");
    }

    #[test]
    fn the_overflow_buffer_trips_the_panic_after_3600_updates() {
        // `Liquid.cs:1009-1020`: a >= 90%-full overflow buffer for 3601
        // updates starts the panic.
        let mut w = shaft(9, 60, 50, 4);
        let mut sys = LiquidSystem::new(&w);
        // Fill the buffer to the 90% band without real cells (the gate
        // reads only the length).
        sys.buffer = vec![(0, 0); (MAX_BUFFER as f64 * 0.9) as usize];
        sys.panic_counter = 3600;
        sys.update(&mut w);
        assert!(sys.panic_mode, "the 3601st update starts the panic");
        assert_eq!(sys.panic_y, sys.height - 3, ":975 starts at the bottom row");
        assert_eq!(sys.active_len(), 0, ":971 drops the active list");
        assert_eq!(sys.buffered_len(), 0, ":972 drops the buffer");
    }

    #[test]
    fn the_load_settle_drains_a_suspended_blob_to_the_floor() {
        // `WorldFile.cs:733-764`: QuickWater drops the suspended blob,
        // WaterCheck re-wakes the floor cell, the quickSettle drain walks
        // it to rest, and the flag comes back down.
        let mut w = shaft(9, 60, 50, 4);
        put_as(&mut w, 4, 10, 200, 0);
        let mut sys = LiquidSystem::new(&w);
        sys.settle_loaded_world(&mut w);
        assert_eq!(w.tiles[w.index(4, 10)].liquid, 0, "the origin drained");
        assert_eq!(w.tiles[w.index(4, 49)].liquid, 200, "the blob landed");
        assert!(!sys.quick_settle(), ":760 drops the flag");
        let total: i32 = w.tiles.iter().map(|t| t.liquid as i32).sum();
        assert_eq!(total, 200, "the load settle conserves the mass");
    }
}

/// W4-g47 settle-wiring witnesses: the load hook's dry-world no-op (the
/// round-trip pin guarantee), the load context flags (`WorldFile.cs:733`/
/// `:764`), the `SettleWaterAt` gen re-type arm (`Liquid.cs:178-192`) and the
/// DelWater keep-one reconciliation (g47 canonical + g45's live reads / ring
/// marking / explicit lily arm). Written, NOT run (code-only mode).
#[cfg(test)]
mod liquid_settle_wiring {
    use super::*;
    use crate::tile::FLAG_ACTIVE;

    fn shaft(width: i32, height: i32, floor_y: i32, cx: i32) -> World {
        let mut w = World::new("liquid_sw".into(), width, height);
        for y in 0..=floor_y {
            for x in [cx - 1, cx + 1] {
                let i = w.index(x, y);
                w.tiles[i] = Tile { kind: 1, ..Default::default() };
                w.tiles[i].flags |= FLAG_ACTIVE;
            }
        }
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
    fn the_load_settle_on_a_dry_world_is_a_byte_and_rng_noop() {
        // The round-trip pin guarantee (ANALYSIS W4-g47): a dry world's
        // settle mutates zero tiles, bumps zero revisions and consumes zero
        // `frame_rng` draws — so wiring it into `App::new` cannot move the
        // save/load pins (`saveglue.rs`) or the framing RNG stream.
        let mut w = shaft(9, 60, 50, 4);
        w.seed_frame_rng();
        let tiles_before = w.tiles.clone();
        let rev_before = w.revision;
        let rng_before = w.frame_rng.clone();
        let mut sys = LiquidSystem::new(&w);
        sys.settle_loaded_world(&mut w);
        assert_eq!(w.tiles, tiles_before, "no tile mutated");
        assert_eq!(w.revision, rev_before, "no revision bump");
        assert_eq!(w.frame_rng, rng_before, "no genRand stand-in draw");
        assert_eq!(sys.active_len(), 0, "nothing woke");
        assert!(!sys.generating, ":764 drops the load context");
    }

    #[test]
    fn the_load_settle_arms_and_drops_the_load_context() {
        // `WorldFile.cs:733`/`:764`: the drain runs under
        // `isGeneratingOrLoadingWorld` (the port's `generating`), with the
        // STRICT `generatingWorld` down and `waterLine = maxTilesY`. After
        // the settle the context drops back to the runtime state.
        let mut w = shaft(9, 60, 50, 4);
        put_as(&mut w, 4, 10, 200, 0);
        let mut sys = LiquidSystem::new(&w);
        assert!(!sys.generating, "runtime starts un-armed");
        assert_eq!(sys.gen_water_line, w.height, "the maxTilesY load pin");
        sys.settle_loaded_world(&mut w);
        assert!(!sys.generating, ":764 clears isGeneratingOrLoadingWorld");
        assert!(!sys.generating_world, "load is not generation");
        assert!(!sys.quick_settle(), ":760 drops quickSettle");
        // The blob settled as WATER — the re-type arm stayed dead on load.
        assert_eq!(
            w.tiles[w.index(4, 49)].liquid_kind(),
            LiquidKind::Water,
            "the waterLine pin keeps the load settle type-preserving"
        );
    }

    #[test]
    fn the_gen_settle_retypes_a_deep_water_drop_to_lava() {
        // `Liquid.cs:184-187`: a gen-settle water drop landing below
        // `GenVars.waterLine` re-types to lava. The context arms `generating`
        // + a real waterLine (NOT the load's maxTilesY pin).
        let mut w = shaft(9, 60, 50, 4);
        put_as(&mut w, 4, 10, 200, 0);
        let mut sys = LiquidSystem::new(&w);
        sys.set_gen_settle_context(20, 30, false); // waterLine 20 < landing 49
        sys.quick_water(&mut w, -1, -1);
        assert_eq!(w.tiles[w.index(4, 49)].liquid, 200, "the mass landed");
        assert_eq!(
            w.tiles[w.index(4, 49)].liquid_kind(),
            LiquidKind::Lava,
            ":186 re-types below the waterLine"
        );
    }

    #[test]
    fn the_gen_settle_keeps_water_above_the_line_and_honey_everywhere() {
        // `Liquid.cs:184` (`num2 > waterLine` false above the line) and
        // `:178` (`!flag && !flag2` — honey/shimmer never re-type).
        let mut w = shaft(9, 60, 50, 4);
        put_as(&mut w, 4, 10, 200, 0);
        let mut sys = LiquidSystem::new(&w);
        sys.set_gen_settle_context(55, 60, false); // waterLine 55 > landing 49
        sys.quick_water(&mut w, -1, -1);
        assert_eq!(
            w.tiles[w.index(4, 49)].liquid_kind(),
            LiquidKind::Water,
            "above the waterLine keeps water"
        );

        let mut w2 = shaft(9, 60, 50, 4);
        put_as(&mut w2, 4, 10, 200, 2); // honey origin
        let mut sys2 = LiquidSystem::new(&w2);
        sys2.set_gen_settle_context(20, 30, false);
        sys2.quick_water(&mut w2, -1, -1);
        assert_eq!(
            w2.tiles[w2.index(4, 49)].liquid_kind(),
            LiquidKind::Honey,
            ":178 honey keeps its type even below the line"
        );
    }

    #[test]
    fn delwater_sub2_dry_at_the_world_edge_does_not_panic() {
        // g45's edge fix (`2249cf59`): the C#'s `Main.tile[nx, y]` is null
        // at the edge; the port probes through `tile` before indexing so a
        // negative `nx` cannot wrap. A sub-2 cell at x=0 retires cleanly.
        let mut w = World::new("edge".into(), 9, 12);
        for x in 0..9 {
            let i = w.index(x, 10);
            w.tiles[i] = Tile { kind: 1, ..Default::default() };
            w.tiles[i].flags |= FLAG_ACTIVE;
        }
        w.frame_important = vec![false; td::TILE_COUNT];
        w.compute_column_surface();
        w.compute_chunks();
        put_as(&mut w, 0, 9, 1, 0); // sub-2 at the left edge
        let mut sys = LiquidSystem::new(&w);
        sys.add_water(&w, 0, 9);
        sys.active[0].kill = KILL_PASSES;
        sys.del_water(&mut w, 0); // must not panic on the x-1 probe
        assert_eq!(w.tiles[w.index(0, 9)].liquid, 0, "the edge film dried");
        assert_eq!(sys.active_len(), 0, "the entry retired");
    }

    #[test]
    fn delwater_lava_grass_ring_converts_and_dirties_the_pass() {
        // g45's ring marking: the grass→dirt conversion sets `dirty` so the
        // pass bumps the world revision (and marks the converted chunk for
        // the draw rebuild). A retired lava cell beside a grass tile flips
        // it to dirt 0. The lava cell sits on the floor (solid below) so the
        // >=20 keep-alive cannot rescue it and the reaction tail runs.
        let mut w = shaft(9, 60, 50, 4);
        // Grass 2 on the floor row beside the landing column.
        let g = w.index(5, 49);
        w.tiles[g] = Tile { kind: 2, ..Default::default() };
        w.tiles[g].flags |= FLAG_ACTIVE;
        put_as(&mut w, 4, 49, 100, 1); // lava on the floor
        let mut sys = LiquidSystem::new(&w);
        sys.add_water(&w, 4, 49);
        sys.active[0].kill = KILL_PASSES;
        sys.del_water(&mut w, 0);
        assert_eq!(w.tiles[g].kind, 0, "the grass ring converted to dirt");
        assert!(sys.dirty, "the ring write dirtied the pass (g45's marking)");
    }
}
