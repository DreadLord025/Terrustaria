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

/// The `SettleWaterAt` dungeon re-type carry (`Liquid.cs:188-191`) — the
/// liquid module's view of `GenVars.dungeonGenVars`
/// (`DungeonUtils.InAnyPotentialDungeonBounds`, `DungeonUtils.cs:2095-2108`:
/// per-entry OUTER `ContainsWithFluff(x, y, 0)` = `Contains`;
/// `IsConsideredDungeonWall(wall, allDungeons: true)`, `:1894-1915`: the
/// base `Main.wallDungeon` set ∪ each entry's `isDungeonWall` extension).
///
/// The producer side is the f24/c30 lane: `create_potential_dungeon_bounds`
/// (`dungeon_genvars.rs`, `DungeonUtils.cs:2067-2087`) computes the rects;
/// the merger converts each `DualDungeonSetup.dungeons[i].outer` hitbox to
/// a `(left, top, right, bottom)` half-open row here (XNA `Right`/`Bottom`
/// exclusive), plus that dungeon's wall-set extension
/// (`DungeonCrawler.cs:155-181` — the style quads' `BrickWallType` +
/// `ExtraWallTypes` over the cloned `Main.wallDungeon`; on a classic world
/// `isDungeonWall == Main.wallDungeon`, so the extension is empty).
#[derive(Clone, Debug, Default)]
pub struct GenDungeonContext {
    /// One outer potential-bounds hitbox per `GenVars.dungeonGenVars`
    /// entry: `(left, top, right, bottom)`, half-open (`Contains` is
    /// `left <= x < right && top <= y < bottom`). Empty = the C# empty
    /// list — `InAnyPotentialDungeonBounds` is false everywhere.
    pub outer_bounds: Vec<(i32, i32, i32, i32)>,
    /// The union of the per-entry `isDungeonWall` extensions BEYOND
    /// `Main.wallDungeon` (the dual-dungeon style walls). The base set
    /// rides `dungeon_features::is_dungeon_wall`.
    pub extra_dungeon_walls: Vec<u16>,
}

impl GenDungeonContext {
    /// `DungeonUtils.InAnyPotentialDungeonBounds(x, y)` (`:2089-2108`) —
    /// fluff 0 / inner false: any entry's outer hitbox contains the cell.
    fn in_any_bounds(&self, x: i32, y: i32) -> bool {
        self.outer_bounds
            .iter()
            .any(|&(l, t, r, b)| l <= x && x < r && t <= y && y < b)
    }

    /// `DungeonUtils.IsConsideredDungeonWall(wall, allDungeons: true)`
    /// (`:1894-1915`) — the base `Main.wallDungeon` set (`wall > 0` guard
    /// included) or any entry's extension.
    fn is_dungeon_wall(&self, wall: u16) -> bool {
        crate::worldgen::dungeon_features::is_dungeon_wall(wall)
            || self.extra_dungeon_walls.contains(&wall)
    }
}

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
    /// `Liquid.skipCount` (`Liquid.cs:19`): the runtime tick divider the
    /// `WorldGen.UpdateWorld` driver walks (`WorldGen.cs:71526-71534`) —
    /// `skipCount++; if (skipCount > 1) { UpdateLiquid(); skipCount = 0; }`,
    /// i.e. one pass per two world ticks (30 Hz). The counter belongs to the
    /// DRIVER, not the pass: the gen settle loops and the load drain call
    /// [`LiquidSystem::update`] directly and never touch it (the C# statics
    /// keep it across those drains too). `ReInit` zeroes it (`:85`).
    skip_count: i32,
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
    /// `Main.Setting_UseReducedMaxLiquids` (`Main.cs:1665`) — the
    /// `slowliquids=1` launch flag. Caps `curMaxLiquid` at 5000 (`ReInit`
    /// `Liquid.cs:98-101`, the gfx pair `Main.cs:17080-17083`/`:17108-17111`)
    /// and ORs `numLiquid > 2000` into the `quickFall` derivation
    /// (`Liquid.cs:1057-1060`). Default false = stock behaviour.
    reduced_max_liquids: bool,
    /// `WorldGen.remixWorldGen` — the desert-strip arm's gate
    /// (`Liquid.cs:1447`); the R-12-5 plumbing owns it, `false` at runtime.
    /// W4-f25: the `Settle Liquids` pass sets it from the seed flags.
    remix_world_gen: bool,
    /// `WorldGen.isGeneratingOrLoadingWorld` — the merge-tile branch selector
    /// (`Liquid.cs:1333`/`:1447`) and the `SettleWaterAt` re-type gate
    /// (`:178`). The port's settlement pass never runs during generation
    /// yet (R-12-5 plumbs it), so it stays `false`.
    generating: bool,
    /// `WorldGen.generatingWorld` — the STRICT generation flag (true only
    /// during worldgen, false during the world-LOAD settle). It gates the
    /// `SettleWaterAt` dungeon re-type arm (`Liquid.cs:188`) and
    /// `ShimmerRemoveWater` (`WorldGen.cs:34825`), neither of which the
    /// load settle (`isGeneratingOrLoadingWorld` true, `generatingWorld`
    /// false) may run. Distinct from [`Self::generating`] above. The
    /// predicates themselves live in [`GenDungeonContext`] (the f24/c30
    /// bounds carry) — an un-armed context keeps the arm a no-op, exactly
    /// like the C# empty `GenVars.dungeonGenVars` list.
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
    /// `GenVars.shimmerPosition` (`GenVars.cs:282`) — the g36-A1 GenVars
    /// slot (`worldgen/pipeline.rs` `shimmer_position`, written by the
    /// Shimmer step at the `WorldGen.cs:15241` seam). `QuickWater`'s
    /// `ShimmerRemoveWater` arms read it (`Liquid.cs:108`/`:138`). `None`
    /// mirrors the C# `Vector2D.Zero` reset (`WorldGen.cs:7039`) — an
    /// un-placed shimmer makes the removal circle a corner no-op.
    gen_shimmer_position: Option<(i32, i32)>,
    /// The `SettleWaterAt` dungeon re-type predicates (`Liquid.cs:188-191`):
    /// `DungeonUtils.InAnyPotentialDungeonBounds` + `IsConsideredDungeonWall
    /// (wall, allDungeons: true)` over the f24/c30 potential-bounds carry.
    /// `None` = no `GenVars.dungeonGenVars` entries (the C# empty list —
    /// the arm is dead), so the runtime/load settle stays a no-op.
    gen_dungeon: Option<GenDungeonContext>,
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
    /// `Liquid.worldGenTilesIgnoreWater`'s state (`Liquid.cs:75-81`): the
    /// doors 10/192 and the living-wood/leaf pair 191/190 lose solidity for
    /// the whole `Settle Liquids` pass (`WorldGen.cs:16156`/`:16207`,
    /// `:21263`/`:21306`). Gen-only — the runtime pass never arms it.
    /// (f25's `gen_ignore_water` export, carried onto the g47 machinery for
    /// the gen-module dedupe — research/252 step 6.)
    gen_ignore_water: bool,
    /// `Main.tileSolid[379]` (`Liquid.cs:451` vs `Main.cs:17753`/`:17799`):
    /// the bubble is solid for the liquid pass itself and not solid for the
    /// entity half of the tick — the C# clears it in
    /// `DoUpdateInWorld_Inner` (`:17753`, before the players) and the
    /// runtime `UpdateLiquid` re-arms it per cell. The app layer clears the
    /// flag at the tick head (the `:17753` seam).
    bubble_solid: bool,
    /// `Main.UnderworldLayer` (`Main.cs:2790`, `maxTilesY - 200`) — the depth
    /// gate of the water evaporation (`Liquid.cs:466`). Production seeds it
    /// from the world height in [`Self::new`]; the flow witnesses pin it to
    /// `i32::MAX` to isolate the fall/level mechanics from the band (the
    /// `tool.rs` test convention), since their fixture worlds are shorter
    /// than a vanilla underworld. NOT a gen field — runtime-faithful, never
    /// gated on `generating` (research/69 contract, item 5; research/252
    /// gate 3). W4-g46's evaporation graft (research/252 step 2).
    underworld_layer: i32,
    /// The 3-cell surface average's `WorldGen.genRand.Next(30)` nudge
    /// stream (`Liquid.cs:892-895`) — a per-consumer `UnifiedRandom` seeded
    /// from the world seed (the `place_rng` precedent; vanilla shares one
    /// `genRand`, documented in ANALYSIS). Drawn only when the average
    /// rounds to 254. W4-g46's nudge graft (research/252 step 3 — g46's
    /// stream pick wins over g45's `frame_rng`).
    surface_rng: crate::unified_random::UnifiedRandom,
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

    /// `Main.Setting_UseReducedMaxLiquids` (`Main.cs:1665`) — the
    /// `slowliquids=1` command-line flag (`Main.cs:5079-5083`, reset false
    /// at `static Main()` `:67008`; no config.json key). It caps
    /// `curMaxLiquid` at 5000 in `ReInit` (`Liquid.cs:98-101`) and the
    /// gfx-quality pair (`Main.cs:17080-17083`/`:17108-17111`), and ORs
    /// `numLiquid > 2000` into the pass-head `quickFall` derivation
    /// (`:1057-1060`). False keeps every arm byte-identical to stock.
    pub fn set_reduced_max_liquids(&mut self, reduced: bool) {
        self.reduced_max_liquids = reduced;
    }

    /// `Main.Setting_UseReducedMaxLiquids` — the gfx-quality pass reads it
    /// for the `2500 + 2500 * gfxQuality` budget (`Main.cs:17080-17083`)
    /// and the quickSettle override's flat 5000 (`:17108-17111`).
    pub fn reduced_max_liquids(&self) -> bool {
        self.reduced_max_liquids
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
        // `Liquid.cs:85` — `skipCount = 0`: the driver divider resets with
        // the world-load settle (the runtime tick starts a fresh 2-tick
        // window after `ReInit`).
        self.skip_count = 0;
        self.stuck_count = 0;
        self.stuck_amount = 0;
        self.cycles = CYCLES;
        self.cur_max_liquid = MAX_ACTIVE;
        // `Liquid.cs:98-101` — the reduced-liquids cap rides ReInit itself,
        // so the load settle (which never sees the gfx block) starts on it.
        if self.reduced_max_liquids {
            self.cur_max_liquid = 5000;
        }
        // The gen-context fields (`generating`/`generating_world`/
        // `gen_water_line`/`gen_shimmer_position`/`gen_dungeon`) are NOT
        // touched — the C# ReInit (`:83-102`) never resets the WorldGen /
        // GenVars statics; the load/gen drivers own their own arming.
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
    pub fn settle_loaded_world(&mut self, world: &mut World, rng: &mut crate::UnifiedRandom) {
        self.re_init();
        // `:733` — the load context: the merge selector reads
        // `isGeneratingOrLoadingWorld`, the re-type reads the `waterLine`
        // pin. `generating_world` stays down (load ≠ generation).
        self.generating = true;
        self.gen_water_line = self.height;
        self.quick_water(world, rng, -1, -1);
        self.water_check(world, rng);
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
            self.update(world, rng);
        }
        self.quick_settle = false;
        self.water_check(world, rng);
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
            skip_count: 0,
            quick_fall: false,
            quick_settle: false,
            reduced_max_liquids: false,
            generating: false,
            generating_world: false,
            // The load-settle pin (`WorldFile.cs:734`): `waterLine =
            // maxTilesY` makes the classic re-type arm dead on load. The gen
            // settle passes overwrite both from the terrain-fill products
            // (R-12-5); the runtime never reads them (`generating` is down).
            gen_water_line: world.height,
            gen_lava_line: 0,
            gen_shimmer_position: None,
            gen_dungeon: None,
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
            gen_ignore_water: false,
            bubble_solid: false,
            // `Main.UnderworldLayer = maxTilesY - 200` (`Main.cs:2790`).
            underworld_layer: world.height - 200,
            surface_rng: crate::unified_random::UnifiedRandom::new(world.seed as i32),
        }
    }

    /// The flow witnesses' constructor: the fixture worlds are shorter than a
    /// vanilla underworld, so `height - 200` would sit above every cell and
    /// evaporate the fixtures. The sentinel parks the evaporation band out of
    /// reach (the `tool.rs` `i32::MAX` convention), isolating the fall/level
    /// mechanics under test. The evaporation witnesses use [`Self::new`].
    /// (W4-g46, research/252 step 2 — every flow witness must use this.)
    #[cfg(test)]
    pub(crate) fn new_flow_fixture(world: &World) -> Self {
        let mut sys = Self::new(world);
        sys.underworld_layer = i32::MAX;
        sys
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

    /// `Main.tileSolid[379] = true` (`Liquid.cs:114`, `QuickWater`'s head):
    /// the bubble is solid for the whole pre-settle sweep. The runtime
    /// `UpdateLiquid` re-arms it per pass ([`Self::update`]).
    pub fn set_bubble_solid(&mut self, v: bool) {
        self.bubble_solid = v;
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

    pub(crate) fn set_checking(&mut self, x: i32, y: i32, v: bool) {
        let i = self.bit(x, y);
        if v {
            self.checking[i >> 6] |= 1 << (i & 63);
        } else {
            self.checking[i >> 6] &= !(1 << (i & 63));
        }
    }

    /// `Liquid.AddWater` (`Liquid.cs:1160-1207`): queues one cell if it holds
    /// liquid and no entry owns it, then kills a liquid-death object standing
    /// in the cell. `&mut World` because the death kill (`WorldGen.KillTile`,
    /// `:1201`) runs INLINE — the C# kills before `AddWater` returns, so the
    /// cascade's tile edits are visible to the rest of the caller's visit.
    /// (W4-g45 round 2 `daa1cdb0`, carried per research/252 step 5 with the
    /// `edit_kills` loot upgrade — g45b's inline kill discarded the record.)
    pub fn add_water(&mut self, world: &mut World, rng: &mut crate::UnifiedRandom, x: i32, y: i32) {
        if x < 0 || y < 0 || x >= self.width || y >= self.height || self.is_checking(x, y) {
            return;
        }
        // `Liquid.cs:1163` — the 5-tile world-border exclusion: the outer
        // five rows/columns never join the active list (vanilla worlds are
        // >= 1200 tiles wide/tall, so the band is pure margin).
        if x < 5 || y < 5 || x >= self.width - 5 || y >= self.height - 5 {
            return;
        }
        let Some(t) = world.tile(x, y).copied() else {
            return;
        };
        // `Liquid.cs:1163` — `nactive() && tileSolid[type] && type != 546 &&
        // !tileSolidTop[type]`: the grate 546 is exempt EVEN outside the pass
        // (the stock `Main.tileSolid[546] = true`, `Main.cs:10124`); the
        // pass-scoped boulder/bubble toggles ride `tile_is_solid` (S1-L-14).
        if t.liquid == 0 || (self.tile_is_solid(&t) && t.kind != 546) {
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
        // `Liquid.cs:1183-1206` — the submerged-object death check. An
        // inactive tile (or a generating/loading world) returns right after
        // the queue; otherwise lava reads `CheckLavaDeath`, every other
        // liquid — water, honey AND shimmer — reads `CheckWaterDeath`
        // (there is no shimmer-specific AddWater: the one function serves
        // all four kinds). The Tile overloads resolve the def from the
        // cell's frames (`TileObjectData.cs:5083-5119`, W4-e11's seam);
        // a death kills the tile INLINE (`WorldGen.KillTile(x, y)`, the
        // default noItem=false). LOOT UPGRADE (research/252 step 5): g45b
        // discarded the kill record — the port pushes the `TileKill` onto
        // `edit_kills`, the same loot seam the DelWater keep-one uses (the
        // liquid tick resolves the drops on the UpdateLiquid stream).
        // The `:1183` gate reads `isGeneratingOrLoadingWorld` — the port's
        // `generating` field IS that flag (armed during the LOAD settle
        // too), so the gate is already right (g47c reconciliation).
        // The MP `NetSendLiquid`/`SendData(17)` arms are unported.
        if !t.active() || self.generating {
            return;
        }
        let dies = if t.liquid_kind() == LiquidKind::Lava {
            crate::tileobject::check_lava_death(&t)
        } else {
            crate::tileobject::check_water_death(&t)
        };
        if dies {
            if let Some(kill) = crate::edit::kill_tile(world, x, y, false, rng) {
                self.edit_kills.push(kill);
            }
            // The cell keeps its liquid (`remove_tile` preserves it), so
            // the queued entry settles on as the C#'s does.
        }
    }

    /// Queues every liquid cell in a square around an edit, so breaking or placing a
    /// block that borders a pool starts the flow without scanning the whole world.
    pub fn wake_around(&mut self, world: &mut World, rng: &mut crate::UnifiedRandom, cx: i32, cy: i32, r: i32) {
        for y in (cy - r).max(0)..=(cy + r).min(self.height - 1) {
            for x in (cx - r).max(0)..=(cx + r).min(self.width - 1) {
                self.add_water(world, rng, x, y);
            }
        }
    }

    /// One `WorldGen.UpdateWorld` liquid-driver tick (`WorldGen.cs:71526-
    /// 71534`): `Liquid.skipCount++; if (skipCount > 1) { UpdateLiquid();
    /// skipCount = 0; }` — the 30 Hz pump, VERBATIM. Returns whether the
    /// pass ran this tick.
    ///
    /// The C# wraps the pass in `using (Main.SwapRandom("UpdateLiquid"))`
    /// (`:71529`); the port's engine draws only the g46 254-nudge
    /// `surface_rng` stream inside `update`, so the named-stream swap is
    /// the CALLER's seam for the loot: `world_sim/liquids.rs` resolves the
    /// drained `edit_kills` on `RngKey::UpdateLiquid`, exactly the stream
    /// the C# `KillTile` drops draw under the swap. The
    /// `isGeneratingOrLoadingWorld` early-return (`:71497-71500`) is the
    /// caller's own — the port's frame loop never runs under generation
    /// (worldgen completes before the loop starts).
    pub fn update_world_tick(&mut self, world: &mut World, rng: &mut crate::UnifiedRandom) -> bool {
        self.skip_count += 1;
        if self.skip_count > 1 {
            self.update(world, rng);
            self.skip_count = 0;
            true
        } else {
            false
        }
    }

    /// One `Liquid.UpdateLiquid` call. Tick this at 30 Hz, like the original
    /// (every second 60 Hz world tick).
    /// `rng` is the ACTIVE `genRand` stream of the liquid tick — the C#
    /// `UpdateLiquid` swap (`WorldGen.cs:71528`, the port's
    /// `RngKey::UpdateLiquid`): the merge-tile `KillTile`'s framing dispatch
    /// (W4-g83 contract §2) draws its seaweed tip re-rolls on it.
    pub fn update(&mut self, world: &mut World, rng: &mut crate::UnifiedRandom) {
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
                self.quick_water(world, rng, self.panic_y, self.panic_y);
                self.panic_y -= 1;
                if self.panic_y < 3 {
                    // `:1034-1037` — the settle finished: drop the mode
                    // and re-scan the grid (WaterCheck).
                    self.panic_counter = 0;
                    self.panic_mode = false;
                    self.water_check(world, rng);
                }
            }
            self.ignore_solids = false;
            return;
        }
        // `Liquid.cs:1056-1068` — the pass head derives `quickFall` from
        // `quickSettle` every call; `Setting_UseReducedMaxLiquids` ORs in
        // the `numLiquid > 2000` overload arm (`:1057-1060`).
        self.quick_fall =
            self.quick_settle || (self.reduced_max_liquids && self.active.len() > 2000);
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
            let (kill, delay) = self.update_tile(world, x, y, self.active[i].kill, delay, rng);
            self.active[i].kill = kill;
            self.active[i].delay = delay;
            if self.quick_fall {
                self.set_skip(x, y, false);
            }
        }
        if self.wet_counter >= self.cycles {
            self.wet_counter = 0;
            self.end_pass(world, rng);
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
    fn end_pass(&mut self, world: &mut World, rng: &mut crate::UnifiedRandom) {
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
                self.del_water(world, rng, l);
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
            self.add_water(world, rng, x, y);
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
                    self.del_water(world, rng, k);
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
    /// KEEP-ONE (W4-g47 vs W4-g45/g46, ANALYSIS + research/69): this body is
    /// canonical — the `Check*` kills ride `edit_kills` (g45's
    /// `check_alch_tile`/`check_lily_pad` wrappers execute the kill but
    /// discard the drop record, losing the herb/lily-pad loot), and the
    /// swap-remove lives in one body the sweep and the stuck self-clear
    /// share (g45 splits it into `retire_sweep`; the two shapes are
    /// equivalent). Adopted from g45: the edge-safe sub-2 probe, the live
    /// side re-reads (`:1519-1528` reads the tile references AFTER the
    /// drain), the explicit lily-pad arm order, and the grass ring's
    /// chunk-dirty marking. Adopted from g46 (research/69 row 1): the
    /// `square_tile_frame_record` loot seam on the grass-ring / lily
    /// reframe arms (g46's `retire_kills` queue dedupes onto `edit_kills`
    /// — one queue, one app drain); the g46c graft keeps THIS body.
    fn del_water(&mut self, world: &mut World, rng: &mut crate::UnifiedRandom, l: usize) {
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
                    self.add_water(world, rng, nx, y);
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
            self.add_water(world, rng, x, y - 1);
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
                    self.add_water(world, rng, nx, y);
                }
            }
            // `Liquid.cs:1529-1569` — the reaction tail. The lava arm
            // re-checks the merge and converts the grass ring; honey and
            // shimmer only re-check. The kind reads the LIVE cell — the
            // merge may have emptied it (the C# holds the tile reference).
            let tail_kind = kind_code(world.tiles[i].liquid_kind());
            if tail_kind == 1 {
                self.lava_check(world, x, y, rng);
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
                            // g46's record seam (research/69 row 1): the
                            // reframe cascade's pots/herbs/piles ride
                            // `edit_kills` — `square_tile_frame_default`
                            // would discard them.
                            self.edit_kills
                                .push(crate::edit::square_tile_frame_record(world, gx, gy, rng));
                        } else if matches!(t.kind, 60 | 70 | 661 | 662) {
                            world.tiles[j].kind = 59;
                            world.mark_chunk(gx, gy);
                            self.dirty = true;
                            self.edit_kills
                                .push(crate::edit::square_tile_frame_record(world, gx, gy, rng));
                        }
                    }
                }
            } else if tail_kind == 2 {
                self.honey_check(world, x, y, rng);
            } else if tail_kind == 3 {
                self.shimmer_check(world, x, y, rng);
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
            if let Some(kill) = crate::edit::check_alch(world, x, y, rng) {
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
                    crate::edit::check_lily_pad(world, x, y, |k| self.kind_solid(k), rng)
                {
                    self.edit_kills.push(kill);
                }
            } else {
                let below = world.tile(x, y + 1).copied().unwrap_or_default();
                let above = world.tile(x, y - 1).copied().unwrap_or_default();
                if below.liquid < 255 || above.liquid > 0 {
                    // The reframe arm's cascade records ride the same seam
                    // (g46's `square_tile_frame_record`, research/69).
                    self.edit_kills
                        .push(crate::edit::square_tile_frame_record(world, x, y, rng));
                } else if let Some(kill) =
                    crate::edit::check_lily_pad(world, x, y, |k| self.kind_solid(k), rng)
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
        rng: &mut crate::UnifiedRandom,
    ) -> (u16, u8) {
        let i = world.index(x, y);
        let start = world.tiles[i];
        if self.tile_is_solid(&start) || start.liquid == 0 {
            return (KILL_DEAD, delay);
        }
        let kind = kind_code(start.liquid_kind());

        // --- underworld water evaporation (`Liquid.cs:466-474`) --------------
        // Below `Main.UnderworldLayer` (`Main.cs:2790`, `maxTilesY - 200`) a
        // WATER cell (`liquidType() == 0`, the type bits — not `anyWater`)
        // sheds `min(2, liquid)` units per update. Lava/honey/shimmer never
        // evaporate. The evaporation runs BEFORE the reaction block and the
        // flow, and a cell it empties retires immediately (`:475-478`,
        // `kill = 999`). `start` keeps the pre-evaporation level for the kill
        // bookkeeping at the tail (the C#'s `byte liquid = tile5.liquid`
        // captured at `:464`). W4-g46 graft (research/252 step 2).
        if y > self.underworld_layer && kind == 0 && world.tiles[i].liquid > 0 {
            let b = world.tiles[i].liquid.min(2);
            world.tiles[i].liquid -= b;
            world.mark_chunk(x, y);
            self.dirty = true;
            if world.tiles[i].liquid == 0 {
                return (KILL_DEAD, delay);
            }
        }

        // --- `Liquid.Update`'s reaction block (`Liquid.cs:475-560`) -----------
        // A lava cell checks itself first; every other cell wakes its lava
        // neighbours and falls through to the honey arm, then the shimmer
        // arm. The reactions run EVERY tick — the delay gate below
        // throttles only the flow, and `quickFall` bypasses it
        // (`Liquid.cs:484-490`/`:513-522`).
        let mut flow_delay = delay;
        if kind == 1 {
            self.lava_check(world, x, y, rng);
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
                    self.add_water(world, rng, nx, ny);
                }
            }
            if kind == 2 {
                self.honey_check(world, x, y, rng);
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
                        self.add_water(world, rng, nx, ny);
                    }
                }
                if kind == 3 {
                    self.shimmer_check(world, x, y, rng);
                } else {
                    for (nx, ny) in [(x - 1, y), (x + 1, y), (x, y - 1), (x, y + 1)] {
                        if world.tile(nx, ny).is_some_and(|t| {
                            t.liquid > 0 && t.liquid_kind() == LiquidKind::Shimmer
                        }) {
                            self.add_water(world, rng, nx, ny);
                        }
                    }
                }
            }
        }

        // --- fall through the cell below ---------------------------------------
        // W4-g45 doc-sweep carry (8d581953, adopted onto the g47 base): the
        // C# reads the LIVE `tile5.liquid` here (`Liquid.cs:570-575`), not
        // the entry snapshot — the reaction block above may already have
        // consumed this cell (the `LiquidCheck` side/up merge zeroes it,
        // `:1256-1261`), and the g46 evap graft sheds units at the head.
        // The pre-read `start.liquid` resurrected the consumed/shed units
        // on every fall. `start` stays the kill-bookkeeping capture
        // (`:464`) — the tail's 255→254 compare still reads it.
        let own_now = world.tiles[i].liquid;
        if let Some(below) = world.tile(x, y + 1).copied() {
            if !self.tile_is_solid(&below)
                && (below.liquid == 0 || kind_code(below.liquid_kind()) == kind)
                && below.liquid < 255
            {
                let mut amount = 255 - below.liquid;
                if amount > own_now {
                    amount = own_now;
                }
                // The original keeps a 255 source intact when only one unit moves.
                let keep_one = amount == 1 && own_now == 255;
                let j = world.index(x, y + 1);
                if !keep_one {
                    world.tiles[i].liquid = own_now - amount;
                }
                world.tiles[j].liquid += amount;
                set_kind(&mut world.tiles[j], kind);
                world.mark_chunk(x, y + 1);
                world.mark_chunk(x, y);
                self.dirty = true;
                self.add_water(world, rng, x, y + 1);
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
                    self.add_water(world, rng, x - 1, y);
                    self.add_water(world, rng, x + 1, y);
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
                    let equal = set_level(world, rng, self, x - 1, y, avg, kind)
                        + set_level(world, rng, self, x + 1, y, avg, kind)
                        + set_level(world, rng, self, x - 2, y, avg, kind)
                        + set_level(world, rng, self, x + 2, y, avg, kind)
                        + set_level(world, rng, self, x - 3, y, avg, kind)
                        + set_level(world, rng, self, x + 3, y, avg, kind);
                    // `Liquid.cs:748-771` — the extra wake pass (W4-g45
                    // carry, research/252 step 4), run BEFORE the conditional
                    // centre write: re-queue each neighbour when its
                    // (post-`set_level`) level OR the centre's level differs
                    // from the average. The neighbours all hold `avg` now,
                    // so the clause reduces to the centre-diff test; the
                    // centre still holds `own` here (the C# reads the
                    // pre-write tile5).
                    if own as i32 != avg {
                        for nx in [x - 1, x + 1, x - 2, x + 2, x - 3, x + 3] {
                            self.add_water(world, rng, nx, y);
                        }
                    }
                    if equal < 6 || !above_full {
                        new_own = avg;
                    }
                } else if f4 && f5 {
                    let avg = round_level(l + r + l2 + r2 + own as f32 + bias, 5);
                    let equal = set_level(world, rng, self, x - 1, y, avg, kind)
                        + set_level(world, rng, self, x + 1, y, avg, kind)
                        + set_level(world, rng, self, x - 2, y, avg, kind)
                        + set_level(world, rng, self, x + 2, y, avg, kind);
                    // `Liquid.cs:821-836` — the 5-neighbour extra wake pass
                    // (W4-g45 carry, research/252 step 4), the same
                    // centre-diff reduction as the 7-arm above.
                    if own as i32 != avg {
                        for nx in [x - 1, x + 1, x - 2, x + 2] {
                            self.add_water(world, rng, nx, y);
                        }
                    }
                    if equal < 4 || !above_full {
                        new_own = avg;
                    }
                } else if f4 {
                    let avg = round_level(l + r + l2 + own as f32 + bias, 4);
                    set_level(world, rng, self, x - 1, y, avg, kind);
                    set_level(world, rng, self, x + 1, y, avg, kind);
                    set_level(world, rng, self, x - 2, y, avg, kind);
                    new_own = avg;
                } else if f5 {
                    let avg = round_level(l + r + r2 + own as f32 + bias, 4);
                    set_level(world, rng, self, x - 1, y, avg, kind);
                    set_level(world, rng, self, x + 1, y, avg, kind);
                    set_level(world, rng, self, x + 2, y, avg, kind);
                    new_own = avg;
                } else {
                    // The three-cell arm (`Liquid.cs:889-909`): the only
                    // `WorldGen.genRand` draw in the liquid file —
                    // `num == 254 && genRand.Next(30) == 0 → 255` (`:892-895`).
                    // Deviation (research/11 #5, ANALYSIS): vanilla shares one
                    // `genRand` across all consumers; the port keeps a
                    // per-consumer stream (`LiquidSystem::surface_rng`, seeded
                    // from the world seed in `new`, the `place_rng` precedent),
                    // so the draw never interleaves with other systems. The
                    // roll runs exactly once per 3-cell average, BEFORE any
                    // neighbour write — the C# order. W4-g46 graft
                    // (research/252 step 3 — g46's stream pick, NOT g45's
                    // `frame_rng`).
                    let mut avg = round_level(l + r + own as f32 + bias, 3);
                    if avg == 254 && self.surface_rng.next_max(30) == 0 {
                        avg = 255;
                    }
                    set_level(world, rng, self, x - 1, y, avg, kind);
                    set_level(world, rng, self, x + 1, y, avg, kind);
                    new_own = avg;
                }
            } else if f2 {
                let l = left.map_or(0, |t| t.liquid) as f32;
                let avg = round_level(l + own as f32 + bias, 2);
                set_level(world, rng, self, x - 1, y, avg, kind);
                new_own = avg;
            } else if f3 {
                let r = right.map_or(0, |t| t.liquid) as f32;
                let avg = round_level(r + own as f32 + bias, 2);
                set_level(world, rng, self, x + 1, y, avg, kind);
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
                self.add_water(world, rng, x, y - 1);
                (0, flow_delay)
            }
        } else {
            (kill.saturating_add(1), flow_delay)
        }
    }

    /// `Liquid.LavaCheck` (`Liquid.cs:1445-1459`): the generation-only
    /// underground-desert lava strip, then the merge check.
    fn lava_check(&mut self, world: &mut World, x: i32, y: i32, rng: &mut crate::UnifiedRandom) {
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
        self.liquid_check(world, x, y, 1, false, rng);
    }

    /// `Liquid.HoneyCheck` (`Liquid.cs:1460-1463`).
    fn honey_check(&mut self, world: &mut World, x: i32, y: i32, rng: &mut crate::UnifiedRandom) {
        self.liquid_check(world, x, y, 2, false, rng);
    }

    /// `Liquid.ShimmerCheck` (`Liquid.cs:1465-1468`).
    fn shimmer_check(&mut self, world: &mut World, x: i32, y: i32, rng: &mut crate::UnifiedRandom) {
        self.liquid_check(world, x, y, 3, false, rng);
    }

    /// `Liquid.LiquidCheck` (`Liquid.cs:1225-1373`) — the whole merge
    /// surface. The side/up arms pool the different-type neighbour liquid
    /// (zeroing it even when the merge gate then refuses — the C#'s own
    /// consumption) and merge at the checking cell; the down arm burns the
    /// cut plant below, honours the container override and merges at the
    /// cell below. `createMergeTilesDuringGen` rides the worldgen caller
    /// (`WorldGen.cs:22480` passes `true`).
    pub(crate) fn liquid_check(
        &mut self,
        world: &mut World,
        x: i32,
        y: i32,
        this_liquid_type: u8,
        create_merge_tiles_during_gen: bool,
        rng: &mut crate::UnifiedRandom,
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
                    rng,
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
                // W4-g83: the cascade's framing dispatch rides the liquid
                // tick's ACTIVE genRand stream (the UpdateLiquid swap).
                crate::edit::kill_tile(world, x, y + 1, false, rng);
                // `Main.netMode == 2` → `NetMessage.SendData(17, …)` (`:1309`)
                // — no multiplayer in the port.
            }
            // W4-g45 doc-sweep carry (c8908a55, adopted onto the g47 base):
            // the C# `tile4` is a LIVE reference — the gate (`:1286`) and
            // the merge-type reads (`:1303`) run AFTER the burn, so a
            // `tileCut` kind the kill just cleared passes on
            // `!tile4.active()` even when it is not `tileObsidianKill` (the
            // seven cut kinds outside the set: 444/518/519/549/637/638/711).
            // The pre-kill copy refused those; re-read the cell. The kill
            // keeps the liquid + type bits (`remove_tile`, `WorldGen.cs:63885`
            // clears everything but wall/liquid), so the amount/type reads
            // below see the same liquid the C# does. `flag` stays the PRE-kill
            // read — the C# computes it at `:1274`, before `:1278`'s burn.
            let down = world.tile(x, y + 1).copied().unwrap_or_default();
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
                    rng,
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
        rng: &mut crate::UnifiedRandom,
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
                // W4-g83: the cascade's framing dispatch rides the liquid
                // tick's ACTIVE genRand stream (the UpdateLiquid swap).
                crate::edit::kill_tile(world, x, y, false, rng);
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
    rng: &mut crate::UnifiedRandom,
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
        sys.add_water(world, rng, x, y);
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

    /// `GenVars.shimmerPosition` (`GenVars.cs:282`) — the gen settle
    /// driver copies the g36-A1 GenVars slot here (`gv.shimmer_position`,
    /// the `WorldGen.cs:15241` Shimmer-step write) before its `QuickWater`
    /// calls, so the `ShimmerRemoveWater` arms (`Liquid.cs:108`/`:138`)
    /// read the placed position. `None` = the C# `Vector2D.Zero` reset
    /// (`WorldGen.cs:7039`) — a corner no-op. The runtime/load settle
    /// never arms it (`generating_world` stays down regardless).
    pub fn set_gen_shimmer_position(&mut self, pos: Option<(i32, i32)>) {
        self.gen_shimmer_position = pos;
    }

    /// The `SettleWaterAt` dungeon re-type carry — the gen settle driver
    /// converts the f24/c30 `DualDungeonSetup` outer hitboxes + style-wall
    /// extensions into a [`GenDungeonContext`] (the `GenVars.dungeonGenVars`
    /// stand-in). `None` keeps the arm dead (the C# empty list). Read only
    /// while `generating_world` is up (`Liquid.cs:188`).
    pub fn set_gen_dungeon_context(&mut self, ctx: Option<GenDungeonContext>) {
        self.gen_dungeon = ctx;
    }

    // ---- the f25 gen-module export surface (research/252 step 6) -----------
    // f25's `worldgen/settle_liquids.rs` drives these on its per-pass
    // LiquidSystem; carried onto the g47 machinery so the g46c graft dedupes
    // the gen-local copies instead of keeping a second liquid engine.
    // f25's `set_generating(remix)` maps onto [`Self::set_gen_settle_context`]
    // above (which also arms the STRICT generatingWorld flag + the re-type
    // lines — the fuller C# context).

    /// `Liquid.tilesIgnoreWater` (`Liquid.cs:69-73`): the grate 546 and the
    /// boulder family lose solidity while `ignore` is up. The runtime
    /// `UpdateLiquid` wraps itself in it (`:986`/`:1157`); the gen callers
    /// (`QuickWater`, `SettleWaterAt`, `WaterCheck`) drive it directly.
    pub fn tiles_ignore_water(&mut self, ignore: bool) {
        self.ignore_solids = ignore;
    }

    /// `Liquid.worldGenTilesIgnoreWater` (`Liquid.cs:75-81`): the doors
    /// 10/192 and the living-wood/leaf pair 191/190 lose solidity for the
    /// whole gen settle pass (`WorldGen.cs:16156`/`:16207`).
    pub fn world_gen_tiles_ignore_water(&mut self, ignore: bool) {
        self.gen_ignore_water = ignore;
    }

    /// `Liquid.numLiquid + LiquidBuffer.numLiquidBuffer` — the gen settle
    /// loop's per-round budget base (`WorldGen.cs:16164`/`:21271`).
    pub fn num_liquid_total(&self) -> usize {
        self.active.len() + self.buffer.len()
    }

    /// `WorldGen.ClearPendingLiquid` (`WorldGen.cs:80496-80513`): drop
    /// every active entry and buffered cell, clearing their
    /// `checkingLiquid` bits. The gen settle tail (`:16205`/`:21304`) —
    /// the `Invariant.Assert(numLiquid == 0)` (`WorldGenerator.cs:508`)
    /// holds after it. The skip bits are NOT touched (the C# leaves them;
    /// `AddWater` clears them on the next wake, `Liquid.cs:1173`).
    pub fn clear_pending_liquid(&mut self) {
        if self.active.is_empty() && self.buffer.is_empty() {
            return;
        }
        let active_cells: Vec<(i32, i32)> = self.active.iter().map(|e| (e.x, e.y)).collect();
        for (x, y) in &active_cells {
            self.set_checking(*x, *y, false);
        }
        let buffer_cells: Vec<(i32, i32)> = self.buffer.clone();
        for &(x, y) in &buffer_cells {
            self.set_checking(x, y, false);
        }
        self.active.clear();
        self.buffer.clear();
    }

    /// The reaction check the `SettleWaterAt` tail's `AttemptToMove*`
    /// family calls (`Liquid.cs:284-286`): kind 1 → `LavaCheck`, 2 →
    /// `HoneyCheck`, 3 → `ShimmerCheck` (`Liquid.cs:1445-1468`). The g47
    /// `attempt_to_move_*` methods carry their own arms; this export is
    /// for a gen-local `settle_water_at` copy that stays gen-local
    /// (research/252 step 6's keep-both option).
    pub fn liquid_reaction_check(&mut self, world: &mut World, rng: &mut crate::UnifiedRandom, x: i32, y: i32, kind: u8) {
        match kind {
            1 => self.lava_check(world, x, y, rng),
            2 => self.honey_check(world, x, y, rng),
            3 => self.shimmer_check(world, x, y, rng),
            _ => {}
        }
    }

    /// The gen settle loop (`WorldGen.cs:16158-16206`, the
    /// `SettleLiquidsPart2AndNotTheBees` twin `:21265-21305` is
    /// classic-identical): `WaterCheck` re-registration, `quickSettle` up,
    /// 10 outer rounds — each budgets `num3 * 5` `UpdateLiquid` calls
    /// (`num3` = the active+buffer count at the round head, re-raised
    /// whenever the live count exceeds it) and runs until `numLiquid == 0`
    /// or the budget breaks — `WaterCheck` between the rounds, then
    /// `ClearPendingLiquid` + `quickSettle` down.
    ///
    /// The caller arms the context first (`set_gen_settle_context` +
    /// `world_gen_tiles_ignore_water(true)` + the `QuickWater(3)`
    /// pre-settle); the `extraLiquid` round bump (`:16173-16176`) is the
    /// special-seed arm (d107/c30) — classic breaks straight out. The
    /// progress fractions (`:16179`/`:16184-16199`/`:16203`) drive the
    /// loading bar only: no tile effect, no draw. The gen driver drains/
    /// discards `edit_kills` at the pass tail (research/252 step 7 — the
    /// records must not leak across the per-pass instance).
    pub fn gen_settle_loop(&mut self, world: &mut World, rng: &mut crate::UnifiedRandom) {
        self.water_check(world, rng); // `:16158`
        self.quick_settle = true; // `:16160`
        let mut num = 0;
        let num2 = 10; // `:16161`
        while num < num2 {
            let mut num3 = self.num_liquid_total() as i64; // `:16164`
            num += 1;
            let mut num5 = num3 * 5; // `:16167`
            while self.active_len() > 0 {
                // `:16168`
                num5 -= 1;
                if num5 < 0 {
                    break; // `:16170-16178` (the extraLiquid bump is deferred)
                }
                let live = self.num_liquid_total() as i64;
                if live > num3 {
                    num3 = live; // `:16180-16183`
                }
                self.update(world, rng); // `:16200`
            }
            self.water_check(world, rng); // `:16202`
        }
        self.clear_pending_liquid(); // `:16205`
        self.quick_settle = false; // `:16206`
    }

    /// `Liquid.QuickWater` (`Liquid.cs:104-148`) — the bulk "settle this
    /// rectangle": walk the rows bottom-up and settle every wet cell.
    /// `verbose` is the progress-display parameter (no port consumer).
    ///
    /// The `ShimmerRemoveWater` arms (`:106-108`/`:136-138`) run before
    /// and after the sweep — `generatingWorld`-gated inside the helper
    /// (`WorldGen.cs:34825`), reading [`Self::gen_shimmer_position`] (the
    /// g36-A1 GenVars slot). The remaining generation arms stay inert for
    /// recorded reasons (ANALYSIS W4-g47): the noTraps `SetBoulderSolidity`
    /// swap (`:109-112`/`:139-142`) is subsumed by the pass-scoped
    /// `tilesIgnoreWater(true)` (`:115`) the port already raises (the same
    /// nine boulder kinds, [`BOULDER_SOLIDS`]); `LiquidInteractionsCleanup`
    /// (`:144-147`) is `generatingWorld && !skyblock`-gated and strips the
    /// gen merge tiles 56/659 — it rides the gen settle driver (f25's
    /// `settle_liquids.rs` carries it). The runtime/panic/load settle runs
    /// the solid core below (the shimmer arms early-return, the load
    /// context keeps `generating_world` down).
    pub fn quick_water(&mut self, world: &mut World, rng: &mut crate::UnifiedRandom, min_y: i32, max_y: i32) {
        // `:106-113` — the pre-sweep generation arm.
        self.shimmer_remove_water(world);
        self.bubble_solid = true;
        self.ignore_solids = true;
        let min_y = if min_y == -1 { 3 } else { min_y };
        let max_y = if max_y == -1 { self.height - 3 } else { max_y };
        for y in (min_y..=max_y).rev() {
            for x in 4..self.width - 4 {
                if world.tile(x, y).is_some_and(|t| t.liquid != 0) {
                    self.settle_water_at(world, rng, x, y);
                }
            }
        }
        self.ignore_solids = false;
        // `:136-143` — the post-sweep generation arm.
        self.shimmer_remove_water(world);
    }

    /// `Liquid.SettleWaterAt` (`Liquid.cs:150-289`) — drain one cell's
    /// liquid straight down, then spread it across the row it lands on,
    /// repeating until the mass is placed. The generation-only re-type arm
    /// (`:178-192`) runs while [`Self::generating`] is up: a dropped water
    /// cell landing below `GenVars.waterLine` (classic) or the remix band
    /// re-types to lava; the dungeon arm (`generatingWorld`-gated) re-types
    /// back to water inside the f24/c30 potential bounds behind a dungeon
    /// wall ([`Self::set_gen_dungeon_context`]; an un-armed context keeps
    /// it dead). The runtime settle keeps `generating` down (arm inert);
    /// the world-LOAD settle arms `generating` but pins
    /// `gen_water_line = maxTilesY` (`WorldFile.cs:734`) and keeps
    /// `generating_world` down, so both re-type arms stay inert — the load
    /// keeps the origin's type. The landing cell's merge checks run
    /// through the three `AttemptToMove*` helpers.
    fn settle_water_at(&mut self, world: &mut World, rng: &mut crate::UnifiedRandom, origin_x: i32, origin_y: i32) {
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
                // `:188-191` — the dungeon re-type back to water, gated on
                // the STRICT `generatingWorld` AND
                // `InAnyPotentialDungeonBounds(num, num2)` AND
                // `IsConsideredDungeonWall(tile.wall, allDungeons: true)`.
                // `tile` is the ORIGIN reference (`:152`), so the wall read
                // is the origin cell's — f25's gen-local copy reads the
                // landing cell instead (a deviation the merger reconciles
                // onto this form). The predicates ride the f24/c30
                // potential-bounds carry ([`GenDungeonContext`]); an
                // un-armed context is the C# empty `dungeonGenVars` list —
                // the arm stays dead.
                if self.generating_world {
                    if let Some(ctx) = &self.gen_dungeon {
                        if ctx.in_any_bounds(x, y) && ctx.is_dungeon_wall(origin.wall) {
                            kind = 0;
                        }
                    }
                }
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
            self.attempt_to_move_lava(world, rng, x, y, had_lava);
            self.attempt_to_move_honey(world, rng, x, y, had_honey);
            self.attempt_to_move_shimmer(world, rng, x, y, had_shimmer);
        }
        self.ignore_solids = false;
    }

    /// `Liquid.AttemptToMoveLava` (`Liquid.cs:339-385`) — the landing
    /// cell's lava merge check against the first different-type
    /// neighbour in left/right/up/down order.
    fn attempt_to_move_lava(&mut self, world: &mut World, rng: &mut crate::UnifiedRandom, x: i32, y: i32, had_lava: bool) {
        for (nx, ny) in [(x - 1, y), (x + 1, y), (x, y - 1), (x, y + 1)] {
            let t = world.tile(nx, ny).copied().unwrap_or_default();
            if t.liquid > 0 && (t.liquid_kind() == LiquidKind::Lava) != had_lava {
                if had_lava {
                    self.lava_check(world, x, y, rng);
                } else {
                    self.lava_check(world, nx, ny, rng);
                }
                return;
            }
        }
    }

    /// `Liquid.AttemptToMoveHoney` (`Liquid.cs:291-337`).
    fn attempt_to_move_honey(&mut self, world: &mut World, rng: &mut crate::UnifiedRandom, x: i32, y: i32, had_honey: bool) {
        for (nx, ny) in [(x - 1, y), (x + 1, y), (x, y - 1), (x, y + 1)] {
            let t = world.tile(nx, ny).copied().unwrap_or_default();
            if t.liquid > 0 && (t.liquid_kind() == LiquidKind::Honey) != had_honey {
                if had_honey {
                    self.honey_check(world, x, y, rng);
                } else {
                    self.honey_check(world, nx, ny, rng);
                }
                return;
            }
        }
    }

    /// `Liquid.AttemptToMoveShimmer` (`Liquid.cs:387-433`).
    fn attempt_to_move_shimmer(&mut self, world: &mut World, rng: &mut crate::UnifiedRandom, x: i32, y: i32, had_shimmer: bool) {
        for (nx, ny) in [(x - 1, y), (x + 1, y), (x, y - 1), (x, y + 1)] {
            let t = world.tile(nx, ny).copied().unwrap_or_default();
            if t.liquid > 0 && (t.liquid_kind() == LiquidKind::Shimmer) != had_shimmer {
                if had_shimmer {
                    self.shimmer_check(world, x, y, rng);
                } else {
                    self.shimmer_check(world, nx, ny, rng);
                }
                return;
            }
        }
    }

    /// `WorldGen.ShimmerRemoveWater` (`WorldGen.cs:34823-34851`) — inside
    /// the 150-radius circle around `GenVars.shimmerPosition` (rows
    /// `sy-150..=sy+75`, columns `sx±150`, the truncated `Math.Sqrt` <
    /// 150 distance test), every NON-shimmer liquid zeroes and every
    /// tombstone 162 deactivates. STRICT `generatingWorld`-gated (`:34825`)
    /// — the load settle never runs it. The position rides
    /// [`Self::gen_shimmer_position`], the g36-A1 GenVars slot
    /// (`pipeline.rs` `gv.shimmer_position`, the `WorldGen.cs:15241`
    /// write); `None` mirrors the C# `Vector2D.Zero` reset
    /// (`WorldGen.cs:7039`) — the circle sits at the top-left corner,
    /// off-world for every real shimmer placement, a no-op.
    fn shimmer_remove_water(&mut self, world: &mut World) {
        if !self.generating_world {
            return;
        }
        let Some((sx, sy)) = self.gen_shimmer_position else {
            return;
        };
        let num = 150;
        for i in (sy - num)..=(sy + num / 2) {
            for j in (sx - num)..=(sx + num) {
                // `WorldGen.InWorld(j, i)` (`WorldGen.cs:9688-9695`).
                if j < 0 || i < 0 || j >= self.width || i >= self.height {
                    continue;
                }
                let dx = (j - sx).abs() as f64;
                let dy = (i - sy).abs() as f64;
                if (dx * dx + dy * dy).sqrt() < num as f64 {
                    let idx = world.index(j, i);
                    if world.tiles[idx].liquid_kind() != LiquidKind::Shimmer
                        && world.tiles[idx].liquid != 0
                    {
                        world.tiles[idx].liquid = 0;
                        world.mark_chunk(j, i);
                    }
                    if world.tiles[idx].kind == 162 && world.tiles[idx].active() {
                        world.tiles[idx].flags &= !crate::tile::FLAG_ACTIVE;
                        world.mark_chunk(j, i);
                    }
                }
            }
        }
    }

    /// `WorldGen.WaterCheck` (`WorldGen.cs:80415-80494`) — the post-settle
    /// grid scan: drop the cellular lists, wipe `checkingLiquid`, zero the
    /// liquid inside solid tiles (the bubble 379 keeps its own), kill the
    /// liquid-death objects, and re-wake every cell that can still flow.
    /// The liquid-death kills ride the `edit_kills` seam.
    pub fn water_check(&mut self, world: &mut World, rng: &mut crate::UnifiedRandom) {
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
                            if let Some(kill) = crate::edit::kill_tile(world, x, y, false, rng) {
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
                            self.add_water(world, rng, x, y);
                        }
                    }
                    let left = world.tile(x - 1, y).copied().unwrap_or_default();
                    let right = world.tile(x + 1, y).copied().unwrap_or_default();
                    if self.tile_open(&left) && left.liquid != t.liquid {
                        self.add_water(world, rng, x, y);
                    } else if self.tile_open(&right) && right.liquid != t.liquid {
                        self.add_water(world, rng, x, y);
                    }
                    if t.liquid_kind() == LiquidKind::Lava {
                        let up = world.tile(x, y - 1).copied().unwrap_or_default();
                        if (left.liquid > 0 && left.liquid_kind() != LiquidKind::Lava)
                            || (right.liquid > 0 && right.liquid_kind() != LiquidKind::Lava)
                            || (up.liquid > 0 && up.liquid_kind() != LiquidKind::Lava)
                            || (below.liquid > 0 && below.liquid_kind() != LiquidKind::Lava)
                        {
                            self.add_water(world, rng, x, y);
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
    pub fn tile_is_solid(&self, t: &Tile) -> bool {
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
        // `Liquid.worldGenTilesIgnoreWater` (`Liquid.cs:75-81`): the doors
        // 10/192 and the living-wood/leaf pair 191/190 lose solidity for
        // the whole gen settle pass — the C# pokes `Main.tileSolid` itself,
        // so the fold rides the raw-table read every consumer shares.
        if self.gen_ignore_water && matches!(kind, 10 | 192 | 191 | 190) {
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


    /// The `Settle Liquids` pass arms the generation selectors for the whole
    /// settle: `WorldGen.isGeneratingOrLoadingWorld` (the merge-tile branch,
    /// `Liquid.cs:1333`) and `WorldGen.remixWorldGen` (the desert-strip gate,
    /// `Liquid.cs:1447`). The per-pass instance stands in for the C# statics.
    pub fn set_generating(&mut self, remix: bool) {
        self.generating = true;
        self.remix_world_gen = remix;
    }

}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tile::FLAG_ACTIVE;

    // W4-g83: the liquid pass threads the ACTIVE genRand stream (the
    // UpdateLiquid swap) into the merge-tile KillTile's framing dispatch.
    // The settlement witnesses pin levels/merges, not streams — a fresh
    // seed-0 stream per call.
    fn rng() -> crate::UnifiedRandom {
        crate::UnifiedRandom::new(0)
    }

    #[test]
    fn the_water_falls_through_the_grate_during_the_pass() {
        // S1-L-14 (`Liquid.cs:69-73`/`:986`/`:1157`): the runtime pass lifts
        // the grate 546's solidity (`Main.tileSolid[546] = !ignoreSolids`,
        // the stock `true` at `Main.cs:10124`), so water above a grate
        // lands under it. W4-g45: the fixture is widened past AddWater's
        // 5-tile border (`:1163`) — the shelf moves to y=11, the water to
        // y=10, the wake cell to (10,10).
        let mut w = World::new("liquid".into(), 21, 23);
        // A grate shelf at y=11, water above it at y=10.
        for x in 9..12 {
            let i = w.index(x, 11);
            w.tiles[i] = Tile { kind: 546, ..Default::default() };
            w.tiles[i].flags |= FLAG_ACTIVE;
            let j = w.index(x, 10);
            w.tiles[j].liquid = 255;
            w.mark_chunk(x, 10);
        }
        w.frame_important = vec![false; td::TILE_COUNT];
        w.compute_column_surface();
        w.compute_chunks();
        let mut sys = LiquidSystem::new_flow_fixture(&w);
        sys.add_water(&mut w, &mut rng(), 10, 10);
        // The water delay gate moves a cell every ~6 ticks (`Liquid.cs:484`).
        for _ in 0..40 {
            sys.update(&mut w, &mut rng());
        }
        let above: i32 = (9..12).map(|x| w.tiles[w.index(x, 10)].liquid as i32).sum();
        let mut below: i64 = 0;
        for x in 0..21 {
            for y in 12..23 {
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
        // for the entity half of the tick. W4-g45: the fixture is widened
        // past AddWater's 5-tile border (shelf y=11, water y=10).
        let mut w = World::new("liquid".into(), 21, 23);
        for x in 9..12 {
            let i = w.index(x, 11);
            w.tiles[i] = Tile { kind: 379, ..Default::default() };
            w.tiles[i].flags |= FLAG_ACTIVE;
            let j = w.index(x, 10);
            w.tiles[j].liquid = 255;
            w.mark_chunk(x, 10);
        }
        w.frame_important = vec![false; td::TILE_COUNT];
        w.compute_column_surface();
        w.compute_chunks();
        let mut sys = LiquidSystem::new_flow_fixture(&w);
        sys.add_water(&mut w, &mut rng(), 10, 10);
        for _ in 0..8 {
            sys.update(&mut w, &mut rng());
        }
        let above: i32 = (9..12).map(|x| w.tiles[w.index(x, 10)].liquid as i32).sum();
        let below: i32 = (9..12).map(|x| w.tiles[w.index(x, 12)].liquid as i32).sum();
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
        // water during the pass. W4-g45: the fixture is widened past
        // AddWater's 5-tile border (shelf y=11, water y=10).
        let mut w = World::new("liquid".into(), 21, 23);
        for x in 9..12 {
            let i = w.index(x, 11);
            w.tiles[i] = Tile { kind: 138, ..Default::default() };
            w.tiles[i].flags |= FLAG_ACTIVE;
            let j = w.index(x, 10);
            w.tiles[j].liquid = 255;
            w.mark_chunk(x, 10);
        }
        w.frame_important = vec![false; td::TILE_COUNT];
        w.compute_column_surface();
        w.compute_chunks();
        let mut sys = LiquidSystem::new_flow_fixture(&w);
        sys.add_water(&mut w, &mut rng(), 10, 10);
        for _ in 0..40 {
            sys.update(&mut w, &mut rng());
        }
        let mut below: i64 = 0;
        for x in 0..21 {
            for y in 12..23 {
                below += w.tiles[w.index(x, y)].liquid as i64;
            }
        }
        let below = below as i32;
        assert!(below > 0, "the water fell past the boulder ({below})");
    }

    fn basin(width: i32, wall_y: i32) -> World {
        // W4-g45 AddWater border guard (`Liquid.cs:1163`): the scene is
        // embedded at a (+6,+6) offset in a world 12 wider / 12 taller, so
        // every liquid cell clears the 5-tile world-border exclusion. The
        // floor rows move to `wall_y+6..=wall_y+8`; callers shift their
        // coordinates by (+6,+6).
        let mut w = World::new("liquid".into(), width + 12, wall_y + 12);
        for x in 0..width + 12 {
            for y in wall_y + 6..wall_y + 9 {
                let i = w.index(x, y);
                w.tiles[i] = Tile {
                    kind: 1,
                    ..Default::default()
                };
                w.tiles[i].flags |= FLAG_ACTIVE;
            }
        }
        w.frame_important = vec![false; td::TILE_COUNT];
        // Containment walls at the scene's left/right edge (x=5 and
        // x=width+6): the original 9-wide basin relied on the world edge to
        // hold the pool; embedded in a wider world the same job needs solid
        // walls. `open`/`same` read a solid wall exactly like the
        // out-of-world `None` (both refuse the row arms).
        for y in 0..wall_y + 12 {
            for x in [5, width + 6] {
                let i = w.index(x, y);
                w.tiles[i] = Tile { kind: 1, ..Default::default() };
                w.tiles[i].flags |= FLAG_ACTIVE;
            }
        }
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
            sys.update(w, &mut rng());
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
        // consumed. W4-g45: the basin scene is embedded at (+6,+6) past
        // AddWater's 5-tile border.
        let mut w = basin(9, 8);
        put_as(&mut w, 10, 10, 30, 1);
        put_as(&mut w, 11, 10, 30, 0);
        let mut sys = LiquidSystem::new_flow_fixture(&w);
        sys.add_water(&mut w, &mut rng(), 10, 10);
        sys.add_water(&mut w, &mut rng(), 11, 10);
        run(&mut sys, &mut w, 1);
        assert_eq!(w.tile(10, 10).unwrap().kind, 56, "the lava cell became obsidian");
        assert_eq!(w.tile(10, 10).unwrap().liquid, 0);
        assert_eq!(w.tile(11, 10).unwrap().liquid, 0, "the pooled water is consumed");
    }

    #[test]
    fn water_and_honey_merge_into_a_honey_block() {
        let mut w = basin(9, 8);
        put_as(&mut w, 10, 10, 30, 2);
        put_as(&mut w, 11, 10, 30, 0);
        let mut sys = LiquidSystem::new_flow_fixture(&w);
        sys.add_water(&mut w, &mut rng(), 10, 10);
        sys.add_water(&mut w, &mut rng(), 11, 10);
        run(&mut sys, &mut w, 1);
        assert_eq!(w.tile(10, 10).unwrap().kind, 229);
        assert_eq!(w.tile(11, 10).unwrap().liquid, 0);
    }

    #[test]
    fn honey_and_lava_merge_into_crisp_honey_once() {
        // The honey cell checks first (`:1375-1439`'s lava arm → 230); the
        // merge tile is not `tileObsidianKill`, so the lava cell's own check
        // afterwards finds an active non-kill tile and refuses — one merge.
        let mut w = basin(9, 8);
        put_as(&mut w, 10, 10, 30, 2);
        put_as(&mut w, 11, 10, 30, 1);
        let mut sys = LiquidSystem::new_flow_fixture(&w);
        sys.add_water(&mut w, &mut rng(), 10, 10);
        sys.add_water(&mut w, &mut rng(), 11, 10);
        run(&mut sys, &mut w, 1);
        assert_eq!(w.tile(10, 10).unwrap().kind, 230);
        assert_eq!(w.tile(11, 10).unwrap().liquid, 0);
        assert_eq!(w.tile(11, 10).unwrap().kind, 0, "no second merge tile");
    }

    #[test]
    fn shimmer_consumes_the_water_into_a_shimmer_block() {
        let mut w = basin(9, 8);
        put_as(&mut w, 10, 10, 30, 3);
        put_as(&mut w, 11, 10, 30, 0);
        let mut sys = LiquidSystem::new_flow_fixture(&w);
        sys.add_water(&mut w, &mut rng(), 10, 10);
        sys.add_water(&mut w, &mut rng(), 11, 10);
        run(&mut sys, &mut w, 1);
        assert_eq!(w.tile(10, 10).unwrap().kind, 659);
        assert_eq!(w.tile(11, 10).unwrap().liquid, 0);
    }

    #[test]
    fn the_merge_gate_needs_24_units() {
        // `Liquid.cs:1325`: `num >= 24` — below it the pooled neighbours
        // stay zeroed (the C#'s own consumption) and the checking cell
        // keeps its liquid; the lava flow then sits in the delay gate
        // (`:484-490`).
        let mut w = basin(9, 8);
        put_as(&mut w, 10, 10, 30, 1);
        put_as(&mut w, 11, 10, 20, 0);
        let mut sys = LiquidSystem::new_flow_fixture(&w);
        sys.add_water(&mut w, &mut rng(), 10, 10);
        sys.add_water(&mut w, &mut rng(), 11, 10);
        run(&mut sys, &mut w, 1);
        assert_eq!(w.tile(10, 10).unwrap().kind, 0, "no obsidian");
        assert_eq!(w.tile(10, 10).unwrap().liquid, 30, "the lava stays");
        assert_eq!(w.tile(11, 10).unwrap().liquid, 0, "the pool is consumed");
    }

    #[test]
    fn the_down_arm_merges_below_the_lava_cell() {
        // `Liquid.cs:1300-1371`: no different-type side/up neighbour, the
        // cell below holds a different liquid >= 24 → the merge lands at
        // `(x, y+1)`.
        let mut w = basin(9, 8);
        put_as(&mut w, 10, 10, 30, 1);
        put_as(&mut w, 10, 11, 30, 0);
        let mut sys = LiquidSystem::new_flow_fixture(&w);
        sys.add_water(&mut w, &mut rng(), 10, 10);
        sys.add_water(&mut w, &mut rng(), 10, 11);
        run(&mut sys, &mut w, 1);
        assert_eq!(w.tile(10, 11).unwrap().kind, 56);
        assert_eq!(w.tile(10, 10).unwrap().liquid, 0);
        assert_eq!(w.tile(10, 11).unwrap().liquid, 0);
    }

    #[test]
    fn lava_burns_the_submerged_cut_plant_below() {
        // `Liquid.cs:1305-1312`: a non-water liquid burns the `tileCut`
        // plant below before the merge gate reads it.
        let mut w = basin(9, 8);
        let i = w.index(10, 11);
        w.tiles[i] = Tile {
            kind: 3,
            ..Default::default()
        };
        w.tiles[i].flags |= FLAG_ACTIVE;
        w.tiles[i].liquid = 30;
        put_as(&mut w, 10, 10, 30, 1);
        let mut sys = LiquidSystem::new_flow_fixture(&w);
        sys.add_water(&mut w, &mut rng(), 10, 10);
        run(&mut sys, &mut w, 1);
        assert_eq!(w.tile(10, 11).unwrap().kind, 56, "the plant burned, the merge landed");
        assert_eq!(w.tile(10, 10).unwrap().liquid, 0);
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
        // non-`tileObsidianKill` cell — stone here. W4-g45: the scene rides
        // the embedded basin coordinates (+6,+6).
        let mut w = basin(9, 8);
        let chest = w.index(10, 10);
        w.tiles[chest] = Tile {
            kind: 21,
            ..Default::default()
        };
        w.tiles[chest].flags |= FLAG_ACTIVE;
        w.tiles[chest].liquid = 30;
        let stone = w.index(10, 11);
        w.tiles[stone] = Tile {
            kind: 1,
            ..Default::default()
        };
        w.tiles[stone].flags |= FLAG_ACTIVE;
        w.tiles[stone].liquid = 30;
        set_kind(&mut w.tiles[stone], 1);
        let mut sys = LiquidSystem::new_flow_fixture(&w);
        sys.liquid_check(&mut w, 10, 10, 0, false, &mut rng());
        assert_eq!(w.tile(10, 11).unwrap().kind, 56, "the stone cell was merged into");
        assert_eq!(w.tile(10, 10).unwrap().liquid, 0);
    }

    #[test]
    fn the_live_merge_queues_its_sound_and_generation_stays_silent() {
        // `Liquid.cs:1345-1347`: the runtime branch plays
        // `PlayLiquidChangeSound(liquidChangeType, x, y)` — the same
        // container-merge fixture as above reports
        // `(LavaWater, 10, 11)` once, then the queue drains empty; the
        // generation arm (`:1333`'s strip branch) queues nothing.
        let mut w = basin(9, 8);
        let chest = w.index(10, 10);
        w.tiles[chest] = Tile {
            kind: 21,
            ..Default::default()
        };
        w.tiles[chest].flags |= FLAG_ACTIVE;
        w.tiles[chest].liquid = 30;
        let stone = w.index(10, 11);
        w.tiles[stone] = Tile {
            kind: 1,
            ..Default::default()
        };
        w.tiles[stone].flags |= FLAG_ACTIVE;
        w.tiles[stone].liquid = 30;
        set_kind(&mut w.tiles[stone], 1);
        let mut sys = LiquidSystem::new_flow_fixture(&w);
        sys.liquid_check(&mut w, 10, 10, 0, false, &mut rng());
        assert_eq!(
            sys.drain_merge_events(),
            vec![(TileChangeType::LavaWater, 10, 11)],
            "the live merge reports its liquid-change sound"
        );
        assert!(sys.drain_merge_events().is_empty(), "drained");

        // The generation arm: `isGeneratingOrLoadingWorld` blocks the sound
        // (`Liquid.cs:1345`'s gate) — the same merge queues nothing.
        let mut sys = LiquidSystem::new_flow_fixture(&w);
        sys.generating = true;
        sys.liquid_check(&mut w, 10, 10, 0, false, &mut rng());
        assert!(
            sys.drain_merge_events().is_empty(),
            "generation never queues a liquid-change sound"
        );
    }

    #[test]
    fn water_falls_and_fills_the_pit_bottom() {
        let mut w = basin(9, 8);
        put(&mut w, 10, 10, 255);
        let mut sys = LiquidSystem::new_flow_fixture(&w);
        sys.add_water(&mut w, &mut rng(), 10, 10);
        run(&mut sys, &mut w, 400);
        assert_eq!(
            w.tile(10, 14).unwrap().liquid,
            0,
            "water must not enter the stone floor"
        );
        for x in 6..15 {
            let l = w.tile(x, 13).unwrap().liquid;
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
        for x in 9..=13 {
            put(&mut w, x, 11, 200);
        }
        let mut sys = LiquidSystem::new_flow_fixture(&w);
        sys.wake_around(&mut w, &mut rng(), 11, 11, 4);
        run(&mut sys, &mut w, 400);
        let levels: Vec<i32> = (6..17)
            .map(|x| w.tile(x, 11).unwrap().liquid as i32)
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
        put(&mut w, 10, 11, 255);
        for y in 8..12 {
            let i = w.index(11, y);
            w.tiles[i] = Tile {
                kind: 1,
                ..Default::default()
            };
            w.tiles[i].flags |= FLAG_ACTIVE;
        }
        let mut sys = LiquidSystem::new_flow_fixture(&w);
        sys.add_water(&mut w, &mut rng(), 10, 11);
        run(&mut sys, &mut w, 300);
        for y in 8..12 {
            assert_eq!(
                w.tile(12, y).unwrap().liquid,
                0,
                "leaked through the wall at y{y}"
            );
        }
    }

    #[test]
    fn a_gap_in_the_dam_drains_into_the_lower_chamber() {
        // W4-g45: the 20x12 dam scene is embedded at (+6,+6) in a 32x24
        // world so every liquid cell clears AddWater's 5-tile border.
        let mut w = World::new("dam".into(), 32, 24);
        for x in 6..26 {
            for y in 14..18 {
                let i = w.index(x, y);
                w.tiles[i] = Tile {
                    kind: 1,
                    ..Default::default()
                };
                w.tiles[i].flags |= FLAG_ACTIVE;
            }
        }
        // Right chamber floor is two tiles lower.
        for x in 17..26 {
            for y in 14..16 {
                let i = w.index(x, y);
                w.tiles[i] = Tile::default();
            }
        }
        for y in 9..14 {
            let i = w.index(16, y);
            w.tiles[i] = Tile {
                kind: 1,
                ..Default::default()
            };
            w.tiles[i].flags |= FLAG_ACTIVE;
        }
        for x in 6..16 {
            put(&mut w, x, 13, 255);
        }
        // Containment walls where the original world edges stood (x=-1 /
        // x=20 → solid x=5 / x=26): a solid wall reads `open == false`
        // exactly like the out-of-world None did.
        for y in 0..24 {
            for x in [5, 26] {
                let i = w.index(x, y);
                w.tiles[i] = Tile { kind: 1, ..Default::default() };
                w.tiles[i].flags |= FLAG_ACTIVE;
            }
        }
        w.frame_important = vec![false; td::TILE_COUNT];
        w.compute_column_surface();
        w.compute_chunks();
        let before = total(&w);

        let hole = w.index(16, 13);
        w.tiles[hole] = Tile::default();
        let mut sys = LiquidSystem::new_flow_fixture(&w);
        sys.wake_around(&mut w, &mut rng(), 16, 13, 4);
        run(&mut sys, &mut w, 1200);

        let right: i64 = (17..26)
            .flat_map(|x| [15, 16].map(|y| w.tile(x, y).unwrap().liquid as i64))
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
        put(&mut w, 8, 9, 100);
        let mut sys = LiquidSystem::new_flow_fixture(&w);
        for _ in 0..5 {
            sys.add_water(&mut w, &mut rng(), 8, 9);
        }
        assert_eq!(sys.active_len(), 1);
        sys.add_water(&mut w, &mut rng(), 8, 8);
        assert_eq!(sys.active_len(), 1, "dry cells are not queued");
    }

    /// A full-width row of equal levels is static: every `update_tile` walk returns
    /// an unchanged cell, so `kill` grows by exactly one per walk and the entry
    /// retires once `kill` reaches the threshold. That turns the *window* into a
    /// directly observable quantity. W4-g45: the row sits at y=11 inside the
    /// embedded basin (the 5-tile border cleared on both axes).
    fn static_row(width: i32) -> (World, LiquidSystem) {
        let mut w = basin(width, 6);
        for x in 6..width + 6 {
            put(&mut w, x, 11, 100);
        }
        let mut sys = LiquidSystem::new_flow_fixture(&w);
        for x in 6..width + 6 {
            sys.add_water(&mut w, &mut rng(), x, 11);
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
        let i = w.index(10, 14);
        w.tiles[i].flags |= crate::tile::FLAG_INACTIVE;
        w.frame_important = vec![false; td::TILE_COUNT];
        put(&mut w, 10, 10, 255);
        let mut sys = LiquidSystem::new_flow_fixture(&w);
        sys.add_water(&mut w, &mut rng(), 10, 10);
        run(&mut sys, &mut w, 400);
        assert!(
            w.tile(10, 14).unwrap().liquid > 0,
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

    // W4-g83 compose: the liquid pass threads the caller's ACTIVE genRand
    // stream; these witnesses pin levels/merges, not streams - a fresh
    // seed-0 stream per call (the tests::rng convention).
    fn rng() -> crate::UnifiedRandom {
        crate::UnifiedRandom::new(0)
    }
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
        let mut sys = LiquidSystem::new_flow_fixture(&w);
        sys.add_water(&mut w, &mut rng(), 30, 50);
        for _ in 0..10 {
            sys.update(&mut w, &mut rng());
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
        let mut sys = LiquidSystem::new_flow_fixture(&w);
        sys.add_water(&mut w, &mut rng(), 31, 69);
        for _ in 0..12 {
            sys.update(&mut w, &mut rng());
        }
        let occupied: Vec<u8> = (5..=58)
            .map(|x| w.tiles[w.index(x, 69)].liquid)
            .filter(|m| *m > 0)
            .collect();
        assert!(occupied.len() >= 24, "the film spread across the floor");
        // The interior cells settle to the same thickness.
        for x in 24..=39 {
                let __bi = w.index(x, 69);
            assert_eq!(
                w.tiles[__bi].liquid, 14,
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
        let mut sys = LiquidSystem::new_flow_fixture(&w);
        sys.add_water(&mut w, &mut rng(), 30, 69);
        sys.add_water(&mut w, &mut rng(), 31, 69);
        for _ in 0..12 {
            sys.update(&mut w, &mut rng());
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
        let mut sys = LiquidSystem::new_flow_fixture(&w);
        sys.add_water(&mut w, &mut rng(), 30, 40);
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
            sys.update(&mut w, &mut rng());
        }
        assert_eq!(total(&w), before, "the shaft conserves byte-exactly");
            let __bi = w.index(30, 79);
        assert!(
            w.tiles[__bi].liquid == 200
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
        let mut sys2 = LiquidSystem::new_flow_fixture(&w2);
        sys2.add_water(&mut w2, &mut rng(), 31, 69);
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
            sys2.update(&mut w2, &mut rng());
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

    /// W4-g47 retire (the g45d co-lane carry, 8d581953's fix adopted) — the
    /// fall arm reads the LIVE post-reaction level (`Liquid.cs:570-575`),
    /// not the `:464` entry snapshot. The discriminator here is the
    /// reaction consumption: lava 30 beside water 30 merges on the same
    /// visit (`LiquidCheck`'s side arm zeroes the checking cell + lands
    /// obsidian 56, `:1256-1270`); the fall arm must then move the LIVE 0,
    /// not the pre-merge 30 — the old `start.liquid` read resurrected the
    /// consumed lava into the cell below. quickSettle bypasses the lava
    /// delay gate so the fall runs on the merge visit. The g45 evap-form
    /// twin (`the_fall_arm_moves_the_post_evaporation_level`) rides
    /// `liquid_merge_cadence` (research/69 A9.2 row 3). SEAL re-pins
    /// (A9.2 row 2 + the re-pin rider): `new_flow_fixture` per the §11
    /// sentinel convention, `add_water(&mut w)` per the union's g45b
    /// guarded signature — the g47 scene (30,50) is border-safe, kept
    /// as-is. Written, NOT run.
    #[test]
    fn the_fall_arm_reads_the_live_post_reaction_level() {
        let mut w = shelf_world();
        stone_row(&mut w, 20, 43, 70);
        let i = w.index(30, 50);
        w.tiles[i].liquid = 30;
        set_kind(&mut w.tiles[i], 1);
        let j = w.index(29, 50);
        w.tiles[j].liquid = 30; // water beside the lava -> the side merge
        let mut sys = LiquidSystem::new_flow_fixture(&w);
        sys.set_quick_settle(true);
        sys.add_water(&mut w, &mut rng(), 30, 50);
        sys.update(&mut w, &mut rng());
        assert_eq!(w.tiles[w.index(30, 50)].kind, 56, "the merge landed obsidian");
        assert_eq!(w.tiles[w.index(30, 50)].liquid, 0, "the merge consumed the lava");
            let __bi = w.index(30, 51);
        assert_eq!(
            w.tiles[__bi].liquid, 0,
            "the fall moved the LIVE 0, not the pre-merge 30"
        );
    }

    /// W4-g46 SEAL merged shimmer-flow pin (research/69 A9.2 row 4 — ONE
    /// pin, both assert families: the g47 closeout `4c81731b`
    /// `shimmer_falls_every_visit_ungated` shelf fixture + `entry.delay`
    /// zero assert, and the g45 `99a4f8ccf`
    /// `shimmer_falls_every_visit_like_water` cadence-trace asserts).
    /// Shimmer has NO delay gate (`Liquid.cs:542-545`: the arm runs
    /// `ShimmerCheck` and falls straight through — no `delay++`/return;
    /// only lava `:483-491` and honey `:514-522` throttle). A shimmer
    /// column falls one tile per two passes like water (the down arm's
    /// `skipLiquid` pair, `:586-587`, cleared next visit `:1096-1104`),
    /// and its `delay` field stays 0 through every visit. The per-liquid
    /// census verdict: water/shimmer ungated, lava 1-in-6, honey 1-in-11.
    /// §11 sentinel switch: `new_flow_fixture` per the g46 convention
    /// (the evap arm is water-only either way, `:466`
    /// `liquidType() == 0`). RE-PIN: the g47 fixture seeded (30,0) — the
    /// union's AddWater 5-tile border (`:1163`, g45b guards) refuses it,
    /// so the scene shifts +6 down (source y=6, walls y=6..=36, floor
    /// y=36; the fall lands y=11 after ten passes); `add_water` re-pinned
    /// `&mut w`. The lava column would still sit at y=6 after ten visits
    /// (its first fall lands on visit 6, the second on 19); honey's first
    /// fall is visit 11. Written, NOT run.
    #[test]
    fn shimmer_falls_every_visit_ungated() {
        let mut w = shelf_world();
        for y in 6..=36 {
            stone_row(&mut w, 29, 29, y);
            stone_row(&mut w, 31, 31, y);
        }
        stone_row(&mut w, 30, 30, 36);
        let i = w.index(30, 6);
        w.tiles[i].liquid = 255;
        set_kind(&mut w.tiles[i], 3);
        let mut sys = LiquidSystem::new_flow_fixture(&w);
        sys.add_water(&mut w, &mut rng(), 30, 6);
        for _ in 0..10 {
            sys.update(&mut w, &mut rng());
        }
        assert_eq!(w.tiles[w.index(30, 11)].liquid, 255, "10 passes -> 5 tiles down");
        assert_eq!(w.tiles[w.index(30, 6)].liquid, 0, "the source drained");
            let __bi = w.index(30, 16);
        assert_eq!(
            w.tiles[__bi].liquid, 0,
            "no ungated every-visit flow"
        );
        assert_eq!(
            kind_code(w.tiles[w.index(30, 11)].liquid_kind()),
            3,
            "the shimmer type rides the fall"
        );
        let entry = sys.active.iter().find(|e| (e.x, e.y) == (30, 11)).unwrap();
        assert_eq!(entry.delay, 0, "no delay gate ever armed");
    }
}

/// W4-g47 — the quick-settle witnesses: the two `Liquid.Update` arms
/// (`Liquid.cs:588-591`/`:944-948`) and the pass head's `quickFall`
/// derivation (`:1056-1068`). Written, NOT run (code-only mode).
#[cfg(test)]
mod liquid_quick_settle {
    use super::*;

    // W4-g83 compose: the liquid pass threads the caller's ACTIVE genRand
    // stream; these witnesses pin levels/merges, not streams - a fresh
    // seed-0 stream per call (the tests::rng convention).
    fn rng() -> crate::UnifiedRandom {
        crate::UnifiedRandom::new(0)
    }
    use crate::tile::FLAG_ACTIVE;

    /// A walled shaft: stone at `x = cx-1`/`cx+1` down to the floor row,
    /// so only the down arm can run (the `mass_conserved` fixture shape).
    /// W4-g45: the scene is embedded at (+6,+6) (world +12/+12, floor at
    /// `floor_y+6`) so every cell clears AddWater's 5-tile border.
    fn shaft(width: i32, height: i32, floor_y: i32, cx: i32) -> World {
        let mut w = World::new("liquid_qs".into(), width + 12, height + 12);
        for y in 0..=floor_y + 6 {
            for x in [cx + 5, cx + 7] {
                let i = w.index(x, y);
                w.tiles[i] = Tile { kind: 1, ..Default::default() };
                w.tiles[i].flags |= FLAG_ACTIVE;
            }
        }
        for x in 0..width + 12 {
            let i = w.index(x, floor_y + 6);
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
        // The source's down arm wakes the cell below AND sets the
        // skip flags, but the quickFall loop bypasses the skip check
        // (`:1083-1090`), so an independently queued below cell would be
        // walked again and drain itself. Seed only the source: the below
        // cell is a transfer target, not a separate visit.
        let mut w = shaft(5, 40, 30, 2);
        put_as(&mut w, 8, 16, 255, 0);
        put_as(&mut w, 8, 17, 253, 0);
        let mut sys = LiquidSystem::new_flow_fixture(&w);
        sys.set_quick_settle(true);
        sys.add_water(&mut w, &mut rng(), 8, 16);
        sys.update(&mut w, &mut rng());
        assert_eq!(w.tiles[w.index(8, 16)].liquid, 255, ":590 clamps > 250");
        assert_eq!(w.tiles[w.index(8, 17)].liquid, 255, "the fed cell filled");

        let mut w2 = shaft(5, 40, 30, 2);
        put_as(&mut w2, 8, 16, 255, 0);
        put_as(&mut w2, 8, 17, 253, 0);
        let mut sys2 = LiquidSystem::new_flow_fixture(&w2);
        sys2.add_water(&mut w2, &mut rng(), 8, 16);
        sys2.update(&mut w2, &mut rng());
        assert_eq!(w2.tiles[w2.index(8, 16)].liquid, 253, "no clamp at runtime");
    }

    #[test]
    fn quick_settle_promotes_the_254_settle_step() {
        // `Liquid.cs:942-952`: the row arm averages 255 with a 253
        // neighbour to 254 — the 255→254 settle step. Quick-settle
        // promotes the cell straight back to 255 (`:946`); the runtime
        // pass keeps the 254 for the retirement sweep's own promotion.
        // W4-g45: the basin scene rides the (+6,+6) embed.
        let basin = |w: &mut World| {
            for x in 0..21 {
                for y in 14..17 {
                    let i = w.index(x, y);
                    w.tiles[i] = Tile { kind: 1, ..Default::default() };
                    w.tiles[i].flags |= FLAG_ACTIVE;
                }
            }
            let wall = w.index(11, 13);
            w.tiles[wall] = Tile { kind: 1, ..Default::default() };
            w.tiles[wall].flags |= FLAG_ACTIVE;
            // Containment walls where the world edges held the pool.
            for y in 0..17 {
                for x in [5, 15] {
                    let i = w.index(x, y);
                    w.tiles[i] = Tile { kind: 1, ..Default::default() };
                    w.tiles[i].flags |= FLAG_ACTIVE;
                }
            }
            w.frame_important = vec![false; td::TILE_COUNT];
            w.compute_column_surface();
            w.compute_chunks();
            put_as(w, 10, 13, 255, 0);
            put_as(w, 9, 13, 253, 0);
        };
        let mut w = World::new("liquid_qs".into(), 21, 23);
        basin(&mut w);
        let mut sys = LiquidSystem::new_flow_fixture(&w);
        sys.set_quick_settle(true);
        sys.add_water(&mut w, &mut rng(), 10, 13);
        sys.update(&mut w, &mut rng());
        assert_eq!(w.tiles[w.index(10, 13)].liquid, 255, ":946 promotes 254");
        assert_eq!(w.tiles[w.index(9, 13)].liquid, 254, "the neighbour averaged");

        let mut w2 = World::new("liquid_qs".into(), 21, 23);
        basin(&mut w2);
        let mut sys2 = LiquidSystem::new_flow_fixture(&w2);
        sys2.add_water(&mut w2, &mut rng(), 10, 13);
        sys2.update(&mut w2, &mut rng());
        assert_eq!(w2.tiles[w2.index(10, 13)].liquid, 254, "runtime keeps the step");
    }

    #[test]
    fn the_pass_head_derives_quick_fall_from_quick_settle() {
        // `Liquid.cs:1056-1068` + `:1083-1090`: quickSettle forces
        // quickFall, whose loop sets delay = 10 — the lava column falls
        // on the FIRST visit instead of waiting out the 5-visit gate.
        let mut w = shaft(5, 40, 30, 2);
        put_as(&mut w, 8, 6, 255, 1);
        let mut sys = LiquidSystem::new_flow_fixture(&w);
        sys.set_quick_settle(true);
        sys.add_water(&mut w, &mut rng(), 8, 6);
        sys.update(&mut w, &mut rng());
        assert!(sys.quick_fall, ":1063 sets quickFall");
        assert_eq!(w.tiles[w.index(8, 7)].liquid, 255, "the delay gate is bypassed");
        sys.set_quick_settle(false);
        sys.update(&mut w, &mut rng());
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

    // W4-g83 compose: the liquid pass threads the caller's ACTIVE genRand
    // stream; these witnesses pin levels/merges, not streams - a fresh
    // seed-0 stream per call (the tests::rng convention).
    fn rng() -> crate::UnifiedRandom {
        crate::UnifiedRandom::new(0)
    }
    use crate::tile::FLAG_ACTIVE;

    fn basin(width: i32, wall_y: i32) -> World {
        // W4-g45: the (+6,+6) embed past AddWater's 5-tile border — world
        // +12/+12, floor rows at `wall_y+6..=wall_y+8`, containment walls
        // at the scene edges (the world edge's old job).
        let mut w = World::new("liquid_settle".into(), width + 12, wall_y + 12);
        for x in 0..width + 12 {
            for y in wall_y + 6..wall_y + 9 {
                let i = w.index(x, y);
                w.tiles[i] = Tile { kind: 1, ..Default::default() };
                w.tiles[i].flags |= FLAG_ACTIVE;
            }
        }
        for y in 0..wall_y + 12 {
            for x in [5, width + 6] {
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
        // A sealed 1-wide cup: walls at x=9/11 up to y=13, floor at y=14
        // (the (+6,+6) embed).
        for y in 0..14 {
            for x in [9, 11] {
                let i = w.index(x, y);
                w.tiles[i] = Tile { kind: 1, ..Default::default() };
                w.tiles[i].flags |= FLAG_ACTIVE;
            }
        }
        put_as(&mut w, 10, 13, 254, 0);
        let mut sys = LiquidSystem::new_flow_fixture(&w);
        sys.add_water(&mut w, &mut rng(), 10, 13);
        for _ in 0..8 {
            sys.update(&mut w, &mut rng());
        }
        assert_eq!(w.tiles[w.index(10, 13)].liquid, 255, ":1115 promotes the 254");
        assert_eq!(sys.active_len(), 0, "the entry retired");
    }

    #[test]
    fn delwater_drains_a_thin_film_into_the_open_side() {
        // `Liquid.cs:1499-1504`: a < 20 cell empties when an open side
        // holds less. DelWater is called directly (the retirement tail's
        // entry point) so the row arm never spreads the film first.
        let mut w = basin(9, 8);
        put_as(&mut w, 10, 13, 10, 2); // honey — the type-clear is visible
        let mut sys = LiquidSystem::new_flow_fixture(&w);
        sys.add_water(&mut w, &mut rng(), 10, 13);
        sys.active[0].kill = KILL_PASSES;
        sys.del_water(&mut w, &mut rng(), 0);
        assert_eq!(w.tiles[w.index(10, 13)].liquid, 0, ":1503 empties the film");
        assert_eq!(sys.active_len(), 0, "the entry is gone");
            let __bi = w.index(10, 13);
        assert_eq!(
            w.tiles[__bi].liquid_kind(),
            LiquidKind::Water,
            "an emptied cell loses its type (:1517)"
        );
    }

    #[test]
    fn delwater_keeps_a_full_cell_over_an_open_below_alive() {
        // `Liquid.cs:1506-1509`: a >= 20 cell with an open non-full cell
        // below resets kill and stays active — the fall has work left.
        let mut w = basin(9, 8);
        put_as(&mut w, 10, 11, 200, 0);
        let mut sys = LiquidSystem::new_flow_fixture(&w);
        sys.add_water(&mut w, &mut rng(), 10, 11);
        sys.active[0].kill = KILL_PASSES;
        sys.del_water(&mut w, &mut rng(), 0);
        assert_eq!(sys.active_len(), 1, "the keep-alive holds the entry");
        assert_eq!(sys.active[0].kill, 0, ":1508 resets the kill count");
        assert_eq!(w.tiles[w.index(10, 11)].liquid, 200, "nothing drained");
    }

    #[test]
    fn delwater_swap_remove_rides_kill_but_not_delay() {
        // `Liquid.cs:1575-1579`: the last entry's x/y/kill move into the
        // retired slot — `delay` keeps the SLOT's value (the C# never
        // copies it).
        let mut w = basin(9, 8);
        // Two sealed cups so both cells are static; the second is the
        // swap donor (the (+6,+6) embed).
        for y in 0..14 {
            for x in [9, 11, 13] {
                let i = w.index(x, y);
                w.tiles[i] = Tile { kind: 1, ..Default::default() };
                w.tiles[i].flags |= FLAG_ACTIVE;
            }
        }
        put_as(&mut w, 10, 13, 100, 0);
        put_as(&mut w, 12, 13, 100, 2);
        let mut sys = LiquidSystem::new_flow_fixture(&w);
        sys.add_water(&mut w, &mut rng(), 10, 13);
        sys.add_water(&mut w, &mut rng(), 12, 13);
        sys.active[0].kill = KILL_PASSES;
        sys.active[0].delay = 3;
        sys.active[1].kill = 5;
        sys.active[1].delay = 9;
        sys.del_water(&mut w, &mut rng(), 0);
        assert_eq!(sys.active_len(), 1);
        assert_eq!((sys.active[0].x, sys.active[0].y), (12, 13), "the donor moved");
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
        // unchanged (no new wakes pollute the buffer). The (+6,+6) embed.
        for y in 0..14 {
            for x in [9, 11] {
                let i = w.index(x, y);
                w.tiles[i] = Tile { kind: 1, ..Default::default() };
                w.tiles[i].flags |= FLAG_ACTIVE;
            }
        }
        put_as(&mut w, 10, 13, 100, 0);
        put_as(&mut w, 8, 13, 100, 0);
        put_as(&mut w, 12, 13, 100, 0);
        let mut sys = LiquidSystem::new_flow_fixture(&w);
        // Shrink the window so the second wake overflows.
        sys.set_cur_max_liquid(2);
        sys.add_water(&mut w, &mut rng(), 10, 13);
        sys.add_water(&mut w, &mut rng(), 8, 13); // active full (len 1 >= 2 - 1) -> buffer
        sys.add_water(&mut w, &mut rng(), 12, 13); // -> buffer
        assert_eq!(sys.active_len(), 1);
        assert_eq!(sys.buffered_len(), 2);
        assert!(sys.is_checking(8, 13), "AddBuffer marks the bit");
        assert!(sys.is_checking(12, 13));
        // The dedupe: a buffered cell does not queue twice.
        sys.add_water(&mut w, &mut rng(), 8, 13);
        assert_eq!(sys.buffered_len(), 2);
        // Restore the window so the promotion can actually queue, and
        // drive one full cycle: num8 = min(numLiquid 1, buffer 2) = 1 —
        // slot 0 promotes, DelBuffer(0) moves the LAST entry into it.
        sys.set_cur_max_liquid(MAX_ACTIVE);
        sys.set_cycles(1);
        sys.update(&mut w, &mut rng());
        assert_eq!(sys.buffered_len(), 1, "one promotion drained the head");
        assert_eq!(sys.buffer[0], (12, 13), "the last entry moved into slot 0");
        assert_eq!(sys.active_len(), 2, "the promoted cell re-queued");
        assert!(sys.is_checking(8, 13), "the promoted cell is active again");
        assert!(sys.is_checking(12, 13), "the still-buffered cell keeps its bit");
    }

    #[test]
    fn the_promotion_budget_is_the_post_sweep_active_count_clamped_to_the_buffer() {
        // R-12-5 driver census (`Liquid.cs:1120-1124`): `num8 =
        // curMaxLiquid - (curMaxLiquid - numLiquid)` is numLiquid AFTER
        // the retirement sweep (the C#'s own algebra), clamped to
        // numLiquidBuffer — a buffer longer than the active list drains
        // only numLiquid entries per cycle, the rest wait for the next.
        // Five sealed 1-wide cups (x = 7,10,13,16,19 — the (+6,+6) embed)
        // so every walked cell stays unchanged and no diffusion wakes
        // pollute the counts.
        let mut w = basin(15, 8);
        for y in 0..14 {
            for x in [6, 8, 9, 11, 12, 14, 15, 17, 18, 20] {
                let i = w.index(x, y);
                w.tiles[i] = Tile { kind: 1, ..Default::default() };
                w.tiles[i].flags |= FLAG_ACTIVE;
            }
        }
        for x in [7, 10, 13, 16, 19] {
            put_as(&mut w, x, 13, 100, 0);
        }
        let mut sys = LiquidSystem::new_flow_fixture(&w);
        sys.set_cur_max_liquid(2);
        sys.add_water(&mut w, &mut rng(), 7, 13);
        for x in [10, 13, 16, 19] {
            sys.add_water(&mut w, &mut rng(), x, 13); // active full -> all four buffer
        }
        assert_eq!((sys.active_len(), sys.buffered_len()), (1, 4));
        sys.set_cur_max_liquid(MAX_ACTIVE);
        sys.set_cycles(1);
        sys.update(&mut w, &mut rng());
        // num8 = min(numLiquid 1, buffer 4) = 1: ONE promotion this cycle;
        // DelBuffer(0) moved the LAST entry (19,13) into slot 0.
        assert_eq!(sys.buffered_len(), 3, "the budget clamps to numLiquid");
        assert_eq!(sys.active_len(), 2);
        assert_eq!(sys.buffer[0], (19, 13), "the swap-remove head");
        // The next cycle walks both sealed cells unchanged (no retire),
        // num8 = min(2, 3) = 2 — two more promote: (19,13) then (16,13).
        sys.update(&mut w, &mut rng());
        assert_eq!(sys.buffered_len(), 1, "the budget grew with the list");
        assert_eq!(sys.active_len(), 4);
        assert_eq!(sys.buffer[0], (13, 13), "the one still waiting");
    }

    #[test]
    fn the_promotion_delbuffers_even_when_addwater_refuses_the_cell() {
        // R-12-5 driver census (`Liquid.cs:1125-1130`): the promotion loop
        // clears checkingLiquid, calls AddWater and DelBuffer(0)
        // UNCONDITIONALLY — a buffered cell whose liquid drained away
        // before its promotion is refused by AddWater (`:1163` liquid==0)
        // but still leaves the buffer (no stuck slot).
        let mut w = basin(9, 8);
        for y in 0..14 {
            for x in [7, 9, 11] {
                let i = w.index(x, y);
                w.tiles[i] = Tile { kind: 1, ..Default::default() };
                w.tiles[i].flags |= FLAG_ACTIVE;
            }
        }
        put_as(&mut w, 10, 13, 100, 0);
        put_as(&mut w, 8, 13, 100, 0);
        let mut sys = LiquidSystem::new_flow_fixture(&w);
        sys.set_cur_max_liquid(2);
        sys.add_water(&mut w, &mut rng(), 10, 13);
        sys.add_water(&mut w, &mut rng(), 8, 13); // -> buffer
        // The buffered cell dries out before its promotion (an edit
        // drained it; the buffer holds coordinates, not a mass snapshot).
        let __bi = w.index(8, 13);
        w.tiles[__bi].liquid = 0;
        sys.set_cur_max_liquid(MAX_ACTIVE);
        sys.set_cycles(1);
        sys.update(&mut w, &mut rng());
        assert_eq!(sys.buffered_len(), 0, "DelBuffer ran despite the refusal");
        assert_eq!(sys.active_len(), 1, "AddWater refused the dry cell");
        assert!(!sys.is_checking(8, 13), ":1127 cleared the bit first");
    }

    #[test]
    fn the_stuck_band_self_clears_after_10000_cycles() {
        // `Liquid.cs:1131-1149`: an active count pinned inside ±50 of
        // stuckAmount for 10 000 consecutive cycles force-retires the
        // whole list with `stuck` up (the keep-alive branch disabled).
        let mut w = basin(9, 8);
        put_as(&mut w, 10, 11, 200, 0);
        let mut sys = LiquidSystem::new_flow_fixture(&w);
        sys.add_water(&mut w, &mut rng(), 10, 11);
        // Pin the band: the count (1) sits inside ±50 of the pinned 1.
        sys.stuck_amount = 1;
        sys.stuck_count = 9_999;
        // Age the entry into the retirement sweep; the keep-alive holds
        // it there (open below, `stuck` still down), then the band check
        // trips the 10 000th cycle and the self-clear runs with `stuck`
        // up — the keep-alive branch disabled.
        sys.active[0].kill = KILL_PASSES;
        sys.end_pass(&mut w, &mut rng());
        assert_eq!(sys.active_len(), 0, "the self-clear emptied the list");
        assert_eq!(sys.stuck_count, 0, ":1142 resets the counter");
        assert!(!sys.stuck, ":1141 drops the flag after the clear");
        // The >= 20 cell had an open below — without `stuck` it would
        // have stayed alive; with it the tail ran and the cell kept its
        // liquid (no side/below drain applies at >= 20).
        assert_eq!(w.tiles[w.index(10, 11)].liquid, 200);
    }

    #[test]
    fn the_stuck_band_repins_when_the_count_leaves_it() {
        // `Liquid.cs:1145-1148`: outside the band the counter resets and
        // stuckAmount re-pins to the live count.
        let mut w = basin(9, 8);
        put_as(&mut w, 10, 13, 100, 0);
        let mut sys = LiquidSystem::new_flow_fixture(&w);
        sys.add_water(&mut w, &mut rng(), 10, 13);
        sys.stuck_amount = 500; // the count 1 is far outside ±50
        sys.stuck_count = 42;
        sys.end_pass(&mut w, &mut rng());
        assert_eq!(sys.stuck_count, 0, "the band exit resets the counter");
        assert_eq!(sys.stuck_amount, sys.active_len() as i32, "re-pinned");
    }

    #[test]
    fn delwater_retires_a_submerged_herb_through_check_alch() {
        // `Liquid.cs:1580-1583` + `WorldGen.cs:46002-46005`: a Daybloom
        // (82, style 0) whose own cell holds WATER dies when its liquid
        // entry retires; the kill rides the edit_kills seam. (The cell must
        // not hold lava: AddWater's inline CheckLavaDeath would kill the
        // herb before the DelWater tail ever read `tileAlch`.)
        let mut w = basin(9, 8);
        let i = w.index(10, 13);
        w.tiles[i] = Tile { kind: 82, frame_x: 0, ..Default::default() };
        w.tiles[i].flags |= FLAG_ACTIVE;
        put_as(&mut w, 10, 13, 30, 0);
        // The soil below is the basin floor (stone 1) — not in the
        // style-0 set either, so the herb dies on the soil arm as well.
        let mut sys = LiquidSystem::new_flow_fixture(&w);
        sys.add_water(&mut w, &mut rng(), 10, 13);
        sys.active[0].kill = KILL_PASSES;
        // The water reaction tail merges nothing here (no lava nearby).
        sys.del_water(&mut w, &mut rng(), 0);
        assert!(!w.tiles[w.index(10, 13)].active(), "the herb died");
        let kills = sys.drain_edit_kills();
        assert_eq!(kills.len(), 1, "one CheckAlch kill record");
        assert!(
            kills[0].herb_events.iter().any(|h| h.kind == 82 && h.x == 10 && h.y == 13),
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
        // The pad is a TileFrameImportant tile (`WorldGen.cs:84966-84968`);
        // `move_lily_pad`'s post-move `SquareTileFrame` runs `CheckLilyPad`
        // for it, not the generic connected framing. The fixture must mark
        // the kind or `after_edit`'s reframe_area overwrites frameY.
        w.frame_important[518] = true;
        let pad = w.index(10, 12);
        w.tiles[pad] = Tile { kind: 518, frame_x: 0, frame_y: 0, ..Default::default() };
        w.tiles[pad].flags |= FLAG_ACTIVE;
        put_as(&mut w, 10, 13, 255, 0);
        let soil = w.index(10, 14);
        w.tiles[soil] = Tile { kind: 2, ..Default::default() };
        w.tiles[soil].flags |= FLAG_ACTIVE;
        let mut sys = LiquidSystem::new_flow_fixture(&w);
        // The pad cell carries a token 1 so it queues (the C# reaches
        // this tail when a pad cell was queued while wet and then
        // dried); the < 2 drain empties it before the lily-pad arm.
        w.tiles[pad].liquid = 1;
        sys.add_water(&mut w, &mut rng(), 10, 12);
        sys.active[0].kill = KILL_PASSES;
        sys.del_water(&mut w, &mut rng(), 0);
        assert_eq!(w.tiles[w.index(10, 13)].kind, 518, "the pad sank onto the water");
        assert!(w.tiles[w.index(10, 13)].active());
        assert_eq!(w.tiles[w.index(10, 13)].frame_y, 0, "the grass row");
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

    // W4-g83 compose: the liquid pass threads the caller's ACTIVE genRand
    // stream; these witnesses pin levels/merges, not streams - a fresh
    // seed-0 stream per call (the tests::rng convention).
    fn rng() -> crate::UnifiedRandom {
        crate::UnifiedRandom::new(0)
    }
    use crate::tile::FLAG_ACTIVE;

    /// A tall walled shaft so the settle's `maxTilesY - 5` fall bound and
    /// the `x in [4, width-4)` column range both have room. W4-g45: the
    /// scene is embedded at (+6,+6) (world +12/+12, floor at `floor_y+6`)
    /// so every woken cell clears AddWater's 5-tile border; callers shift
    /// their coordinates by (+6,+6).
    fn shaft(width: i32, height: i32, floor_y: i32, cx: i32) -> World {
        let mut w = World::new("liquid_fs".into(), width + 12, height + 12);
        for y in 0..=floor_y + 6 {
            for x in [cx + 5, cx + 7] {
                let i = w.index(x, y);
                w.tiles[i] = Tile { kind: 1, ..Default::default() };
                w.tiles[i].flags |= FLAG_ACTIVE;
            }
        }
        for x in 0..width + 12 {
            let i = w.index(x, floor_y + 6);
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
        put_as(&mut w, 10, 16, 200, 0);
        let mut sys = LiquidSystem::new_flow_fixture(&w);
        sys.settle_water_at(&mut w, &mut rng(), 10, 16);
        assert_eq!(w.tiles[w.index(10, 16)].liquid, 0, "the origin drained");
        assert_eq!(w.tiles[w.index(10, 55)].liquid, 200, "the blob landed on the floor");
        let total: i32 = w.tiles.iter().map(|t| t.liquid as i32).sum();
        assert_eq!(total, 200, "the settle conserves the mass");
    }

    #[test]
    fn settle_water_at_tops_up_the_partial_cell_below() {
        // `Liquid.cs:216-229`: a same-type non-full cell below takes
        // `min(255 - below, mass)` before the row spread runs.
        let mut w = shaft(9, 60, 50, 4);
        put_as(&mut w, 10, 16, 100, 0);
        put_as(&mut w, 10, 55, 200, 0);
        let mut sys = LiquidSystem::new_flow_fixture(&w);
        sys.settle_water_at(&mut w, &mut rng(), 10, 16);
        assert_eq!(w.tiles[w.index(10, 55)].liquid, 255, "the partial cell filled");
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
        let b = w.index(10, 26);
        w.tiles[b] = Tile { kind: 379, ..Default::default() };
        w.tiles[b].flags |= FLAG_ACTIVE;
        w.tiles[b].liquid = 100;
        let mut sys = LiquidSystem::new_flow_fixture(&w);
        sys.settle_water_at(&mut w, &mut rng(), 10, 26);
        assert_eq!(w.tiles[b].liquid, 100, "the bubble keeps its liquid");
        sys.settle_water_at(&mut w, &mut rng(), 10, 36); // empty
        assert_eq!(w.tiles[w.index(10, 36)].liquid, 0);
    }

    #[test]
    fn quick_water_settles_the_whole_column() {
        // `Liquid.cs:124-134`: the bottom-up row walk settles every wet
        // cell in the rectangle.
        let mut w = shaft(9, 60, 50, 4);
        put_as(&mut w, 10, 16, 100, 0);
        put_as(&mut w, 10, 18, 100, 0);
        let mut sys = LiquidSystem::new_flow_fixture(&w);
        sys.quick_water(&mut w, &mut rng(), 9, 56);
        // Both blobs land on the floor row, stacking to 200.
        assert_eq!(w.tiles[w.index(10, 55)].liquid, 200);
        assert_eq!(w.tiles[w.index(10, 16)].liquid, 0);
        assert_eq!(w.tiles[w.index(10, 18)].liquid, 0);
    }

    #[test]
    fn the_landing_merge_check_fires_on_the_different_type_side() {
        // `Liquid.cs:282-287` + `:339-351`: the landing cell's lava check
        // runs against a different-type side neighbour — water landing
        // beside lava merges into obsidian at the lava cell.
        let mut w = shaft(11, 60, 50, 5);
        // The landing row's right wall cell is opened so the lava can sit
        // BESIDE the landing column: `LiquidCheck` refuses a solid cell
        // (`:1227`), and the shaft's x=12 column is that wall. Clear its
        // active bit, then pour 30 lava in.
        let li = w.index(12, 55);
        w.tiles[li] = Tile { liquid: 30, ..Default::default() };
        set_kind(&mut w.tiles[li], 1);
        w.mark_chunk(12, 55);
        put_as(&mut w, 11, 16, 30, 0);
        let mut sys = LiquidSystem::new_flow_fixture(&w);
        sys.settle_water_at(&mut w, &mut rng(), 11, 16);
        // The water lands at (11,55); the lava at (12,55) is a different-type
        // side, so AttemptToMoveLava checks the lava cell -> obsidian.
        assert_eq!(w.tiles[w.index(12, 55)].kind, 56, "the lava cell merged");
    }

    #[test]
    fn water_check_zeroes_liquid_inside_solids_and_wakes_the_open() {
        // `WorldGen.cs:80426-80432` + `:80449-80460`: liquid inside a
        // solid tile zeroes; an open cell with a non-full open below wakes.
        let mut w = shaft(9, 60, 50, 4);
        // Liquid trapped inside a floor stone cell.
        let s = w.index(10, 56);
        w.tiles[s].liquid = 100;
        // An open wet cell above the floor.
        put_as(&mut w, 10, 54, 100, 0);
        let mut sys = LiquidSystem::new_flow_fixture(&w);
        sys.water_check(&mut w, &mut rng());
        assert_eq!(w.tiles[s].liquid, 0, "the solid's liquid zeroed");
        assert!(sys.active_len() > 0, "the open cell woke");
        assert!(sys.is_checking(10, 54), "the woken cell is marked");
    }

    #[test]
    fn water_check_kills_a_submerged_water_death_object() {
        // `WorldGen.cs:80435-80448`: a torch (4, WaterDeath) under water
        // dies; the kill rides the edit_kills seam.
        let mut w = shaft(9, 60, 50, 4);
        let t = w.index(10, 54);
        w.tiles[t] = Tile { kind: 4, ..Default::default() };
        w.tiles[t].flags |= FLAG_ACTIVE;
        w.tiles[t].liquid = 100;
        let mut sys = LiquidSystem::new_flow_fixture(&w);
        sys.water_check(&mut w, &mut rng());
        assert!(!w.tiles[t].active(), "the torch died");
        assert!(!sys.drain_edit_kills().is_empty(), "the kill record queued");
    }

    #[test]
    fn re_init_resets_the_cadence_and_lists() {
        // `Liquid.cs:83-102`: every counter/window/list back to stock.
        let mut w = shaft(9, 60, 50, 4);
        put_as(&mut w, 10, 16, 100, 0);
        let mut sys = LiquidSystem::new_flow_fixture(&w);
        sys.add_water(&mut w, &mut rng(), 10, 16);
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
        assert!(!sys.is_checking(10, 16), "the checking bit cleared");
    }

    #[test]
    fn the_panic_settle_drains_five_rows_per_update() {
        // `Liquid.cs:1022-1054`: panicMode walks five QuickWater rows
        // bottom-up per update; reaching row 2 drops the mode.
        let mut w = shaft(9, 60, 50, 4);
        put_as(&mut w, 10, 55, 100, 0);
        let mut sys = LiquidSystem::new_flow_fixture(&w);
        sys.panic_mode = true;
        sys.panic_y = 8; // five rows: 8,7,6,5,4 -> panic_y lands at 3
        sys.update(&mut w, &mut rng());
        // The five-row budget is spent with the cursor at row 3, still in
        // panic mode — the `:1030-1034` finish arm only runs on the next
        // call, when the row-3 QuickWater drops panic_y below 3.
        assert!(sys.panic_mode, "five rows drained, row 3 still pending");
        assert_eq!(sys.panic_y, 3);
        sys.update(&mut w, &mut rng());
        assert!(!sys.panic_mode, "row 3 finishes the settle");
        assert_eq!(sys.panic_y, 2);
        // The blob at row 55 is below the walked band and sits in the sealed
        // cup (solid below and both sides), so the settle leaves it intact
        // and the mode-drop WaterCheck does NOT re-wake it.
        assert_eq!(w.tiles[w.index(10, 55)].liquid, 100, "the blob is untouched");
        assert_eq!(sys.active_len(), 0, "the sealed cup stays asleep");
    }

    #[test]
    fn the_overflow_buffer_trips_the_panic_after_3600_updates() {
        // `Liquid.cs:1009-1020`: a >= 90%-full overflow buffer for 3601
        // updates starts the panic.
        let mut w = shaft(9, 60, 50, 4);
        let mut sys = LiquidSystem::new_flow_fixture(&w);
        // Fill the buffer to the 90% band without real cells (the gate
        // reads only the length).
        sys.buffer = vec![(0, 0); (MAX_BUFFER as f64 * 0.9) as usize];
        sys.panic_counter = 3600;
        sys.update(&mut w, &mut rng());
        assert!(sys.panic_mode, "the 3601st update starts the panic");
        // `:975` starts at `maxTilesY - 3`, then the SAME update's
        // `:1025-1029` loop drains five rows, so the cursor sits at
        // `maxTilesY - 8`.
        assert_eq!(sys.panic_y, sys.height - 8, ":975 start + five drained rows");
        assert_eq!(sys.active_len(), 0, ":971 drops the active list");
        assert_eq!(sys.buffered_len(), 0, ":972 drops the buffer");
    }

    #[test]
    fn the_load_settle_drains_a_suspended_blob_to_the_floor() {
        // `WorldFile.cs:733-764`: QuickWater drops the suspended blob,
        // WaterCheck re-wakes the floor cell, the quickSettle drain walks
        // it to rest, and the flag comes back down.
        let mut w = shaft(9, 60, 50, 4);
        put_as(&mut w, 10, 16, 200, 0);
        let mut sys = LiquidSystem::new_flow_fixture(&w);
        sys.settle_loaded_world(&mut w, &mut rng());
        assert_eq!(w.tiles[w.index(10, 16)].liquid, 0, "the origin drained");
        assert_eq!(w.tiles[w.index(10, 55)].liquid, 200, "the blob landed");
        assert!(!sys.quick_settle(), ":760 drops the flag");
        let total: i32 = w.tiles.iter().map(|t| t.liquid as i32).sum();
        assert_eq!(total, 200, "the load settle conserves the mass");
    }

    // ---- R-12-5 runtime driver witnesses (written, NOT run) ----------------

    #[test]
    fn the_update_world_driver_runs_the_pass_every_second_tick() {
        // `WorldGen.cs:71526-71534`: `skipCount++; if (skipCount > 1)
        // { UpdateLiquid(); skipCount = 0; }` — the first driver tick only
        // counts, the second runs the pass and resets. The window phase
        // matches calling `update` directly on every second tick.
        let mut w = shaft(9, 60, 50, 4);
        put_as(&mut w, 10, 16, 200, 0);
        let mut sys = LiquidSystem::new_flow_fixture(&w);
        sys.add_water(&mut w, &mut rng(), 10, 16);
        assert!(!sys.update_world_tick(&mut w, &mut rng()), "tick 1 only counts");
        assert_eq!(sys.skip_count, 1);
        assert_eq!(w.tiles[w.index(10, 16)].liquid, 200, "no pass yet");
        assert!(sys.update_world_tick(&mut w, &mut rng()), "tick 2 runs the pass");
        assert_eq!(sys.skip_count, 0, ":71533 resets the counter");
        assert_eq!(w.tiles[w.index(10, 16)].liquid, 0, "the blob fell");

        // A twin driven by direct `update` every second tick stays
        // byte-identical across a longer run (the divider adds no phase).
        let mut w2 = shaft(9, 60, 50, 4);
        put_as(&mut w2, 10, 16, 200, 0);
        let mut sys2 = LiquidSystem::new_flow_fixture(&w2);
        sys2.add_water(&mut w2, &mut rng(), 10, 16);
        // Match the twin's phase: the two ticks above already ran ONE pass
        // (tick 1 counts, tick 2 runs), so the direct-update twin takes its
        // first pass before the loop.
        sys2.update(&mut w2, &mut rng());
        for tick in 1..=40 {
            sys.update_world_tick(&mut w, &mut rng());
            if tick % 2 == 0 {
                sys2.update(&mut w2, &mut rng());
            }
        }
        assert_eq!(w.tiles, w2.tiles, "the driver is a pure 2-tick divider");
        assert_eq!(sys.active_len(), sys2.active_len());
    }

    #[test]
    fn re_init_resets_the_driver_divider() {
        // `Liquid.cs:85` — `ReInit` zeroes `skipCount`, so a world swap
        // starts the runtime pump on a fresh window.
        let mut w = shaft(9, 60, 50, 4);
        let mut sys = LiquidSystem::new_flow_fixture(&w);
        assert!(!sys.update_world_tick(&mut w, &mut rng()));
        assert_eq!(sys.skip_count, 1);
        sys.re_init();
        assert_eq!(sys.skip_count, 0, ":85");
        assert!(!sys.update_world_tick(&mut w, &mut rng()), "the fresh window counts again");
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

    // W4-g83 compose: the liquid pass threads the caller's ACTIVE genRand
    // stream; these witnesses pin levels/merges, not streams - a fresh
    // seed-0 stream per call (the tests::rng convention).
    fn rng() -> crate::UnifiedRandom {
        crate::UnifiedRandom::new(0)
    }
    use crate::tile::FLAG_ACTIVE;

    fn shaft(width: i32, height: i32, floor_y: i32, cx: i32) -> World {
        // W4-g45: the (+6,+6) embed (world +12/+12, floor at `floor_y+6`)
        // so every cell AddWater touches clears the 5-tile border; the
        // callers' coordinates and the re-type line pins shift with it.
        let mut w = World::new("liquid_sw".into(), width + 12, height + 12);
        for y in 0..=floor_y + 6 {
            for x in [cx + 5, cx + 7] {
                let i = w.index(x, y);
                w.tiles[i] = Tile { kind: 1, ..Default::default() };
                w.tiles[i].flags |= FLAG_ACTIVE;
            }
        }
        for x in 0..width + 12 {
            let i = w.index(x, floor_y + 6);
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
        let mut sys = LiquidSystem::new_flow_fixture(&w);
        sys.settle_loaded_world(&mut w, &mut rng());
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
        put_as(&mut w, 10, 16, 200, 0);
        let mut sys = LiquidSystem::new_flow_fixture(&w);
        assert!(!sys.generating, "runtime starts un-armed");
        assert_eq!(sys.gen_water_line, w.height, "the maxTilesY load pin");
        sys.settle_loaded_world(&mut w, &mut rng());
        assert!(!sys.generating, ":764 clears isGeneratingOrLoadingWorld");
        assert!(!sys.generating_world, "load is not generation");
        assert!(!sys.quick_settle(), ":760 drops quickSettle");
        // The blob settled as WATER — the re-type arm stayed dead on load.
            let __bi = w.index(10, 55);
        assert_eq!(
            w.tiles[__bi].liquid_kind(),
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
        put_as(&mut w, 10, 16, 200, 0);
        let mut sys = LiquidSystem::new_flow_fixture(&w);
        sys.set_gen_settle_context(26, 36, false); // waterLine 26 < landing 55
        sys.quick_water(&mut w, &mut rng(), -1, -1);
        assert_eq!(w.tiles[w.index(10, 55)].liquid, 200, "the mass landed");
            let __bi = w.index(10, 55);
        assert_eq!(
            w.tiles[__bi].liquid_kind(),
            LiquidKind::Lava,
            ":186 re-types below the waterLine"
        );
    }

    #[test]
    fn the_gen_settle_keeps_water_above_the_line_and_honey_everywhere() {
        // `Liquid.cs:184` (`num2 > waterLine` false above the line) and
        // `:178` (`!flag && !flag2` — honey/shimmer never re-type).
        let mut w = shaft(9, 60, 50, 4);
        put_as(&mut w, 10, 16, 200, 0);
        let mut sys = LiquidSystem::new_flow_fixture(&w);
        sys.set_gen_settle_context(61, 66, false); // waterLine 61 > landing 55
        sys.quick_water(&mut w, &mut rng(), -1, -1);
            let __bi = w.index(10, 55);
        assert_eq!(
            w.tiles[__bi].liquid_kind(),
            LiquidKind::Water,
            "above the waterLine keeps water"
        );

        let mut w2 = shaft(9, 60, 50, 4);
        put_as(&mut w2, 10, 16, 200, 2); // honey origin
        let mut sys2 = LiquidSystem::new_flow_fixture(&w2);
        sys2.set_gen_settle_context(26, 36, false);
        sys2.quick_water(&mut w2, &mut rng(), -1, -1);
            let __bi = w2.index(10, 55);
        assert_eq!(
            w2.tiles[__bi].liquid_kind(),
            LiquidKind::Honey,
            ":178 honey keeps its type even below the line"
        );
    }

    #[test]
    fn delwater_sub2_dry_at_the_world_edge_does_not_panic() {
        // g45's edge fix (`2249cf59`): the C#'s `Main.tile[nx, y]` is null
        // at the edge; the port probes through `tile` before indexing so a
        // negative `nx` cannot wrap. A sub-2 cell at x=0 retires cleanly.
        // W4-g45: AddWater now refuses the border band, so the entry is
        // seeded directly (the DelWater probe itself is what's under test).
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
        let mut sys = LiquidSystem::new_flow_fixture(&w);
        sys.active.push(Entry { x: 0, y: 9, kill: KILL_PASSES, delay: 0 });
        sys.set_checking(0, 9, true);
        sys.del_water(&mut w, &mut rng(), 0); // must not panic on the x-1 probe
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
        let g = w.index(11, 55);
        w.tiles[g] = Tile { kind: 2, ..Default::default() };
        w.tiles[g].flags |= FLAG_ACTIVE;
        put_as(&mut w, 10, 55, 100, 1); // lava on the floor
        let mut sys = LiquidSystem::new_flow_fixture(&w);
        sys.add_water(&mut w, &mut rng(), 10, 55);
        sys.active[0].kill = KILL_PASSES;
        sys.del_water(&mut w, &mut rng(), 0);
        assert_eq!(w.tiles[g].kind, 0, "the grass ring converted to dirt");
        assert!(sys.dirty, "the ring write dirtied the pass (g45's marking)");
    }

    #[test]
    fn reduced_max_liquids_caps_reinit_at_5000() {
        // `Liquid.cs:98-101`: ReInit's `curMaxLiquid = maxLiquid` is
        // overridden to 5000 under `Setting_UseReducedMaxLiquids`; the
        // stock flag keeps the full window.
        let w = shaft(9, 60, 50, 4);
        let mut sys = LiquidSystem::new(&w);
        sys.set_reduced_max_liquids(true);
        sys.re_init();
        assert_eq!(sys.cur_max_liquid(), 5000, ":100 caps the window");
        sys.set_reduced_max_liquids(false);
        sys.re_init();
        assert_eq!(sys.cur_max_liquid(), MAX_ACTIVE, "stock keeps maxLiquid");
    }

    #[test]
    fn reduced_max_liquids_ors_numliquid_over_2000_into_quick_fall() {
        // `Liquid.cs:1057-1060`: the pass-head flag is
        // `quickSettle || (reduced && numLiquid > 2000)`. The fixture
        // stages the active count directly (2001 empty entries — the
        // derivation reads only the count before the window walk).
        let mut w = shaft(9, 60, 50, 4);
        let mut sys = LiquidSystem::new(&w);
        sys.set_cycles(1);
        sys.set_cur_max_liquid(MAX_ACTIVE);
        sys.active = (0..2001)
            .map(|_| Entry { x: 10, y: 16, kill: KILL_DEAD, delay: 0 })
            .collect();
        sys.set_reduced_max_liquids(true);
        sys.update(&mut w, &mut rng());
        assert!(sys.quick_fall, ":1059 the >2000 overload arm fires");
        // Without the flag the same count keeps quickFall down.
        sys.set_reduced_max_liquids(false);
        sys.active = (0..2001)
            .map(|_| Entry { x: 10, y: 16, kill: KILL_DEAD, delay: 0 })
            .collect();
        sys.update(&mut w, &mut rng());
        assert!(!sys.quick_fall, "stock derives quickFall from quickSettle only");
    }

    #[test]
    fn the_gen_settle_dungeon_arm_retypes_lava_back_to_water() {
        // `Liquid.cs:188-191`: a gen-settle drop re-typed to lava below the
        // waterLine flips BACK to water when the landing cell sits inside
        // a potential-dungeon outer bounds AND the ORIGIN cell's wall is a
        // dungeon wall (the C# `tile` reference is the origin, `:152`).
        let mut w = shaft(9, 60, 50, 4);
        let __bi = w.index(10, 16);
        w.tiles[__bi].wall = 7; // the origin's dungeon wall
        put_as(&mut w, 10, 16, 200, 0);
        let mut sys = LiquidSystem::new(&w);
        sys.set_gen_settle_context(26, 36, false); // waterLine 26 < landing 55
        sys.set_gen_dungeon_context(Some(GenDungeonContext {
            outer_bounds: vec![(0, 46, 21, 61)], // covers the landing (10,55)
            extra_dungeon_walls: vec![],
        }));
        sys.quick_water(&mut w, &mut rng(), -1, -1);
        assert_eq!(w.tiles[w.index(10, 55)].liquid, 200, "the mass landed");
            let __bi = w.index(10, 55);
        assert_eq!(
            w.tiles[__bi].liquid_kind(),
            LiquidKind::Water,
            ":190 the dungeon arm re-types back to water"
        );
    }

    #[test]
    fn the_dungeon_arm_needs_bounds_and_wall_and_stays_dead_unarmed() {
        // The three gates (`Liquid.cs:188`): no context (the C# empty
        // dungeonGenVars list), out-of-bounds landing, or a non-dungeon
        // origin wall each keep the lava re-type.
        for (bounds, wall) in [
            (None, 7u16),                       // no GenVars entries
            (Some((0, 0, 21, 26)), 7),          // landing (10,55) outside
            (Some((0, 46, 21, 61)), 0),         // origin has no wall
        ] {
            let mut w = shaft(9, 60, 50, 4);
            let __bi = w.index(10, 16);
            w.tiles[__bi].wall = wall;
            put_as(&mut w, 10, 16, 200, 0);
            let mut sys = LiquidSystem::new(&w);
            sys.set_gen_settle_context(26, 36, false);
            sys.set_gen_dungeon_context(bounds.map(|(l, t, r, b)| GenDungeonContext {
                outer_bounds: vec![(l, t, r, b)],
                extra_dungeon_walls: vec![],
            }));
            sys.quick_water(&mut w, &mut rng(), -1, -1);
                let __bi = w.index(10, 55);
            assert_eq!(
                w.tiles[__bi].liquid_kind(),
                LiquidKind::Lava,
                "the arm stayed dead (bounds {bounds:?}, wall {wall})"
            );
        }
    }

    #[test]
    fn shimmer_remove_water_clears_the_circle_but_keeps_shimmer() {
        // `WorldGen.cs:34829-34850`: inside the 150-radius circle around
        // the placed shimmer, non-shimmer liquid zeroes and tombstone 162
        // deactivates; shimmer liquid survives; outside the circle nothing
        // moves. The position rides the g36-A1 GenVars slot.
        let mut w = World::new("shimmer_rw".into(), 400, 400);
        w.frame_important = vec![false; td::TILE_COUNT];
        w.compute_column_surface();
        w.compute_chunks();
        put_as(&mut w, 200, 200, 100, 0); // at the centre: water dies
        put_as(&mut w, 210, 200, 100, 3); // shimmer survives
        put_as(&mut w, 200, 280, 100, 0); // dy 80 > 75 row bound: outside
        put_as(&mut w, 360, 200, 100, 0); // dx 160 > 150: outside
        let tomb = w.index(205, 200);
        w.tiles[tomb] = Tile { kind: 162, ..Default::default() };
        w.tiles[tomb].flags |= FLAG_ACTIVE;
        let mut sys = LiquidSystem::new(&w);
        sys.set_gen_settle_context(0, 0, false); // generating_world up
        sys.set_gen_shimmer_position(Some((200, 200)));
        sys.shimmer_remove_water(&mut w);
        assert_eq!(w.tiles[w.index(200, 200)].liquid, 0, "the centre water died");
        assert_eq!(w.tiles[w.index(210, 200)].liquid, 100, ":34840 shimmer keeps its liquid");
        assert_eq!(w.tiles[w.index(200, 280)].liquid, 100, "below the +75 row bound");
        assert_eq!(w.tiles[w.index(360, 200)].liquid, 100, "outside the 150 radius");
        assert!(!w.tiles[tomb].active(), ":34846 the tombstone deactivated");
    }

    #[test]
    fn shimmer_remove_water_is_dead_on_load_and_without_a_position() {
        // `WorldGen.cs:34825-34828`: the STRICT generatingWorld gate — the
        // load settle (generating up, generating_world down) never runs
        // it; a None position (the Vector2D.Zero reset, `:7039`) no-ops.
        let mut w = World::new("shimmer_dead".into(), 400, 400);
        w.frame_important = vec![false; td::TILE_COUNT];
        w.compute_column_surface();
        w.compute_chunks();
        put_as(&mut w, 200, 200, 100, 0);
        let mut sys = LiquidSystem::new(&w);
        sys.generating = true; // the load context, generating_world down
        sys.set_gen_shimmer_position(Some((200, 200)));
        sys.shimmer_remove_water(&mut w);
        assert_eq!(w.tiles[w.index(200, 200)].liquid, 100, "load never removes");
        sys.set_gen_settle_context(0, 0, false); // now generating_world up
        sys.set_gen_shimmer_position(None);
        sys.shimmer_remove_water(&mut w);
        assert_eq!(w.tiles[w.index(200, 200)].liquid, 100, "no position, no removal");
    }
}

// --- W4-g46: the underworld evaporation pins (`Liquid.cs:466-474`) ----------
// Re-grafted onto the g47 base (research/252 step 2 + the witness re-pin
// table): g46's four pins KEEP, the g45 duplicates DROP
// (`water_evaporates_two_per_visit_below_the_underworld_layer` — the g45
// `>= 0` guard semantics were dropped for g46's faithful `y > layer` form);
// the layer-row boundary witness is RE-PINNED in `liquid_merge_cadence`
// below onto the g46 unguarded gate.

/// A stone cell at `(x, y)` — the evaporation fixtures' sealed pocket.
#[cfg(test)]
fn evap_stone(w: &mut World, x: i32, y: i32) {
    let i = w.index(x, y);
    w.tiles[i] = Tile { kind: 1, ..Default::default() };
    w.tiles[i].flags |= crate::tile::FLAG_ACTIVE;
}

/// A sealed 1-cell pocket at `(x, y)` (stone left/right/below, open above)
/// holding `amount` of liquid `kind`. The pocket isolates the evaporation
/// arm from the fall/level arms so the ledger is exact.
#[cfg(test)]
fn evap_pocket(w: &mut World, x: i32, y: i32, amount: u8, kind: u8) {
    evap_stone(w, x - 1, y);
    evap_stone(w, x + 1, y);
    evap_stone(w, x, y + 1);
    let i = w.index(x, y);
    w.tiles[i].liquid = amount;
    set_kind(&mut w.tiles[i], kind);
    w.mark_chunk(x, y);
}

#[cfg(test)]
mod liquid_evaporation {
    use super::*;

    // W4-g83 compose: the liquid pass threads the caller's ACTIVE genRand
    // stream; these witnesses pin levels/merges, not streams - a fresh
    // seed-0 stream per call (the tests::rng convention).
    fn rng() -> crate::UnifiedRandom {
        crate::UnifiedRandom::new(0)
    }

    fn evap_world() -> World {
        // W4-g45: the pocket scenes ride a (+6,+6) embed (26 wide / 256
        // tall — the layer moves to 56); every woken cell clears the
        // AddWater border.
        let mut w = World::new("evap".into(), 26, 256);
        w.frame_important = vec![false; td::TILE_COUNT];
        w.compute_column_surface();
        w.compute_chunks();
        w
    }

    fn run(sys: &mut LiquidSystem, w: &mut World, n: usize) {
        for _ in 0..n {
            sys.update(w, &mut rng());
        }
    }

    #[test]
    fn underworld_water_evaporates_two_per_update() {
        // `Liquid.cs:466-474`: below `UnderworldLayer` (`maxTilesY - 200`) a
        // water cell sheds `min(2, liquid)` per update. A 256-tall world's
        // layer is y=56; the sealed pocket at y=66 is inside the band.
        let mut w = evap_world();
        evap_pocket(&mut w, 16, 66, 10, 0);
        let mut sys = LiquidSystem::new(&w);
        sys.add_water(&mut w, &mut rng(), 16, 66);
        // 10 → 8 → 6 → 4 → 2 → 0: five updates empty the cell.
        run(&mut sys, &mut w, 1);
        assert_eq!(w.tile(16, 66).unwrap().liquid, 8, "one update sheds 2");
        run(&mut sys, &mut w, 4);
        assert_eq!(w.tile(16, 66).unwrap().liquid, 0, "five updates empty it");
        assert_eq!(sys.active_len(), 0, "the emptied cell retired (kill=999)");
    }

    #[test]
    fn a_sub_two_film_evaporates_whole() {
        // `Liquid.cs:468-472`: `b = min(2, liquid)` — a 1-unit film sheds its
        // single unit, never underflowing the byte.
        let mut w = evap_world();
        evap_pocket(&mut w, 16, 66, 1, 0);
        let mut sys = LiquidSystem::new(&w);
        sys.add_water(&mut w, &mut rng(), 16, 66);
        run(&mut sys, &mut w, 1);
        assert_eq!(w.tile(16, 66).unwrap().liquid, 0, "the 1-unit film is gone");
        assert_eq!(sys.active_len(), 0);
    }

    #[test]
    fn water_above_the_underworld_layer_does_not_evaporate() {
        // The gate is `y > UnderworldLayer` (`:466`): a pocket at y=46 in the
        // same 256-tall world (layer 56) keeps every unit.
        let mut w = evap_world();
        evap_pocket(&mut w, 16, 46, 10, 0);
        let mut sys = LiquidSystem::new(&w);
        sys.add_water(&mut w, &mut rng(), 16, 46);
        run(&mut sys, &mut w, 8);
        assert_eq!(w.tile(16, 46).unwrap().liquid, 10, "no evaporation above the layer");
    }

    #[test]
    fn lava_honey_and_shimmer_never_evaporate() {
        // `liquidType() == 0` (`:466`) — only water evaporates; the other
        // three kinds keep their mass inside the band.
        for kind in [1u8, 2, 3] {
            let mut w = evap_world();
            evap_pocket(&mut w, 16, 66, 10, kind);
            let mut sys = LiquidSystem::new(&w);
            sys.add_water(&mut w, &mut rng(), 16, 66);
            run(&mut sys, &mut w, 8);
            assert_eq!(
                w.tile(16, 66).unwrap().liquid, 10,
                "kind {kind} must not evaporate"
            );
        }
    }

    #[test]
    fn the_retire_sweep_splices_backwards_and_the_lily_tail_runs_on_drained_cells() {
        // `Liquid.cs:1109-1119` walks the list backwards; `:1580-1598` runs the
        // alch/lily tail even when the cell drained to 0 (no early return).
        // A lily pad 518 over a full (255) below with a dry above takes the
        // `CheckLilyPad` arm; here the pad's below is empty so the pad sinks.
        // RE-RUN note (research/252 witness table): the g47 splice runs INSIDE
        // `del_water` before the tail — the drained-cell tail equivalence
        // holds either way (the tail never AddWaters). W4-g45: the scene
        // rides the (+6,+6) embed (36 wide / 266 tall).
        let mut w = World::new("lily".into(), 36, 266);
        w.frame_important = vec![false; td::TILE_COUNT];
        w.frame_important[518] = true;
        w.compute_column_surface();
        w.compute_chunks();
        // A dirt floor at y=76; the pad sits at y=74 with a water film, the
        // cell below (y=75) empty. The pad's liquid drains (sub-20 film, open
        // below), then the lily tail sinks it onto y=75.
        evap_stone(&mut w, 21, 76);
        let __bi = w.index(21, 76);
        w.tiles[__bi].kind = 2; // dirt → frame row 0
        let pad = w.index(21, 74);
        w.tiles[pad] = Tile { kind: 518, ..Default::default() };
        w.tiles[pad].flags |= crate::tile::FLAG_ACTIVE;
        w.tiles[pad].liquid = 5;
        let mut sys = LiquidSystem::new(&w);
        sys.add_water(&mut w, &mut rng(), 21, 74);
        // Let the film drain and the entry retire (kill reaches 8).
        run(&mut sys, &mut w, 40);
        // The pad moved down onto y=75 (the empty cell below), origin cleared.
        assert_eq!(w.tile(21, 75).unwrap().kind, 518, "the pad sank one cell");
        assert!(!w.tile(21, 74).unwrap().active(), "the origin went inactive");
    }

    #[test]
    fn surface_rng_is_the_world_seeded_unified_stream() {
        // RNG census: the ONLY `WorldGen.genRand` draw in `Liquid.cs` is the
        // 3-cell surface nudge `Next(30)` (`:892`). The port keeps it on a
        // per-consumer `UnifiedRandom` seeded from the world seed (the
        // `place_rng` precedent; vanilla shares one `genRand`, ANALYSIS). This
        // pins the stream to the standard world-seeded `UnifiedRandom` — the
        // determinism contract — draw-for-draw against an independent instance.
        let mut w = World::new("nudge".into(), 20, 250);
        w.seed = 42;
        let sys = LiquidSystem::new(&w);
        let mut reference = crate::unified_random::UnifiedRandom::new(42);
        let mut rng = sys.surface_rng.clone();
        for i in 0..32 {
            assert_eq!(rng.next_max(30), reference.next_max(30), "nudge draw {i}");
        }
        // A different world seed gives a different stream (no hard-coded seed).
        let mut w2 = World::new("nudge".into(), 20, 250);
        w2.seed = 7;
        let sys2 = LiquidSystem::new(&w2);
        let mut reference2 = crate::unified_random::UnifiedRandom::new(7);
        let mut rng2 = sys2.surface_rng.clone();
        for i in 0..8 {
            assert_eq!(rng2.next_max(30), reference2.next_max(30), "seed-7 draw {i}");
        }
    }
}

/// W4-g45 carry-forward (union re-fold, research/252 step 4) — the non-water
/// cadence pins and the down-arm/shimmer merge witnesses: the per-entry
/// `delay` throttle (lava 1-in-6, honey 1-in-11), the `quickFall` bypass, the
/// layer-row evaporation boundary (RE-PINNED onto the g46 unguarded
/// `y > height - 200` semantics), the down-arm sub-24 drain and the
/// shimmer-side merge arms. The g45 evaporation duplicates dropped per
/// research/252 (g46's pins above win). Written, NOT run (code-only mode).
#[cfg(test)]
mod liquid_merge_cadence {
    use super::*;

    // W4-g83 compose: the liquid pass threads the caller's ACTIVE genRand
    // stream; these witnesses pin levels/merges, not streams - a fresh
    // seed-0 stream per call (the tests::rng convention).
    fn rng() -> crate::UnifiedRandom {
        crate::UnifiedRandom::new(0)
    }
    use crate::tile::FLAG_ACTIVE;

    /// A stone floor row at `floor_y` (the `basin` fixture shape, tall
    /// enough for the underworld band when `height >= 200`). W4-g45: the
    /// scene is embedded at a (+6,+6) offset — the world is 12 wider / 12
    /// taller and the floor moves to `floor_y+6`, so every liquid cell
    /// clears AddWater's 5-tile border (`Liquid.cs:1163`).
    fn floor_world(width: i32, height: i32, floor_y: i32) -> World {
        let mut w = World::new("cadence".into(), width + 12, height + 12);
        for x in 0..width + 12 {
            let i = w.index(x, floor_y + 6);
            w.tiles[i] = Tile { kind: 1, ..Default::default() };
            w.tiles[i].flags |= FLAG_ACTIVE;
        }
        w.frame_important = vec![false; td::TILE_COUNT];
        w.compute_column_surface();
        w.compute_chunks();
        w
    }

    /// A one-wide walled shaft: only the down arm can run, so a falling
    /// column's position is a direct cadence clock. The walls fence the row
    /// arms exactly like the world edge did before the border widening.
    fn shaft(width: i32, height: i32, floor_y: i32, cx: i32) -> World {
        let mut w = floor_world(width, height, floor_y);
        for y in 0..floor_y + 6 {
            for x in [cx + 5, cx + 7] {
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
            sys.update(w, &mut rng());
        }
    }

    #[test]
    fn lava_falls_once_every_six_visits() {
        // `Liquid.cs:484-490`: the lava arm returns while `delay < 5`
        // (five throttle visits), then resets `delay = 0` and flows — so the
        // FIRST fall lands on visit 6. The short-list stride re-walk
        // (`:1077-1082`) makes every visit count.
        let mut w = shaft(5, 120, 30, 2);
        put_as(&mut w, 8, 6, 255, 1);
        let mut sys = LiquidSystem::new_flow_fixture(&w);
        sys.add_water(&mut w, &mut rng(), 8, 6);
        run(&mut sys, &mut w, 12);
        assert_eq!(w.tiles[w.index(8, 7)].liquid, 255, "one fall in twelve visits");
        assert_eq!(w.tiles[w.index(8, 18)].liquid, 0, "no ungated flow");
        // The steady cadence is EIGHT visits, not six: visit 7 consumes the
        // two skip flags the transfer set (`:586-587`/`:1096-1104`), and
        // the DelWater swap-remove (`:1577-1579`) copies x/y/kill only — the
        // slot keeps the dead entry's delay 0, dropping the new entry's
        // increment. Visits 8-13 throttle, visit 14 flows.
        run(&mut sys, &mut w, 2);
        assert_eq!(w.tiles[w.index(8, 8)].liquid, 255, "the second fall on visit 14");
        // Then every eight visits: the third lands on visit 22.
        run(&mut sys, &mut w, 8);
        assert_eq!(w.tiles[w.index(8, 9)].liquid, 255, "the third fall on visit 22");
    }

    #[test]
    fn honey_falls_once_every_eleven_visits() {
        // `Liquid.cs:513-522`: the honey arm returns while `delay < 10`,
        // so the first fall lands on the eleventh visit.
        let mut w = shaft(5, 120, 30, 2);
        put_as(&mut w, 8, 6, 255, 2);
        let mut sys = LiquidSystem::new_flow_fixture(&w);
        sys.add_water(&mut w, &mut rng(), 8, 6);
        run(&mut sys, &mut w, 10);
        assert_eq!(w.tiles[w.index(8, 6)].liquid, 255, "ten throttled visits");
        run(&mut sys, &mut w, 1);
        assert_eq!(w.tiles[w.index(8, 7)].liquid, 255, "the first fall on visit 11");
        // Visit 12 consumes the two skip flags; the swap-remove drops the
        // new entry's delay increment (the slot keeps 0), so visits 13-23
        // throttle (ten) and the SECOND fall lands on visit 24 — the steady
        // honey gap is twelve throttle visits + the skip visit.
        run(&mut sys, &mut w, 13);
        assert_eq!(w.tiles[w.index(8, 8)].liquid, 255, "the second fall on visit 24");
    }

    #[test]
    fn quick_fall_bypasses_both_delay_gates() {
        // `Liquid.cs:1083-1090`: the quickFall loop forces `delay = 10`
        // before every Update, so the lava arm's `delay < 5` AND the honey
        // arm's `delay < 10` gates never fire — the column falls every
        // visit like water. The quickFall loop also clears the fall's
        // skipLiquid flag right after the update (`:1089`), so no visit is
        // lost to it.
        // Union re-pin: `update` derives `quick_fall = quick_settle` at the
        // pass head (`:1056-1068`, g47), so the witness arms quick_settle
        // instead of pinning the derived field directly.
        // W4-g47 closeout carry (research/252 §11): the g47
        // `quick_fall_bypasses_both_delay_gates` (lava + honey twins)
        // dedupes onto this shaft fixture — the union's lava-only
        // `quick_fall_bypasses_the_delay_gate` name retired, the honey
        // twin added; the sentinel convention kept.
        for kind in [1u8, 2u8] {
            let mut w = shaft(5, 120, 30, 2);
            put_as(&mut w, 8, 6, 255, kind);
            let mut sys = LiquidSystem::new_flow_fixture(&w);
            sys.set_quick_settle(true);
            sys.add_water(&mut w, &mut rng(), 8, 6);
            run(&mut sys, &mut w, 12);
                let __bi = w.index(8, 18);
            assert_eq!(
                w.tiles[__bi].liquid, 255,
                "kind {kind}: quickFall flows every visit"
            );
        }
    }

    #[test]
    fn the_delay_gate_never_throttles_the_merge() {
        // `Liquid.cs:480-490`: `LavaCheck` runs BEFORE the delay gate —
        // the merge lands on the very first visit even though the flow
        // would be throttled for five more.
        let mut w = floor_world(9, 120, 8);
        put_as(&mut w, 10, 10, 30, 1);
        put_as(&mut w, 11, 10, 30, 0);
        let mut sys = LiquidSystem::new_flow_fixture(&w);
        sys.add_water(&mut w, &mut rng(), 10, 10);
        run(&mut sys, &mut w, 1);
        assert_eq!(w.tiles[w.index(10, 10)].kind, 56, "the merge ignores the delay");
    }

    #[test]
    fn water_at_the_underworld_layer_row_does_not_evaporate() {
        // `Liquid.cs:466` is a strict `y > UnderworldLayer`: the layer row
        // itself (y = 212 in the widened 412-tall world) keeps its water.
        // The floor at y = 213 pins the cell (the shaft call's floor_y is
        // re-pinned 207 so the +6 embed lands the floor one row below the
        // layer), and the stone walls fence the row arms, so only the
        // evaporation gate is tested.
        // RE-PIN (research/252 witness table): the g46 unguarded
        // `y > height - 200` gate — `LiquidSystem::new` seeds the layer,
        // no fixture sentinel.
        let mut w = shaft(9, 400, 207, 4);
        put_as(&mut w, 10, 212, 100, 0);
        let mut sys = LiquidSystem::new(&w);
        sys.add_water(&mut w, &mut rng(), 10, 212);
        run(&mut sys, &mut w, 5);
        assert_eq!(w.tiles[w.index(10, 212)].liquid, 100, "the layer row is exempt");
    }

    #[test]
    fn the_down_arm_drains_a_sub_24_cell_into_the_liquid_below() {
        // `Liquid.cs:1290-1298`: the down arm's `tile5.liquid < 24` branch
        // zeroes the checking cell AND re-types it water (`liquidType(0)`)
        // — no merge tile; the liquid below survives untouched.
        let mut w = floor_world(9, 120, 8);
        put_as(&mut w, 10, 10, 20, 1);
        put_as(&mut w, 10, 11, 30, 0);
        let mut sys = LiquidSystem::new_flow_fixture(&w);
        sys.liquid_check(&mut w, 10, 10, 1, false, &mut rng());
        assert_eq!(w.tiles[w.index(10, 10)].liquid, 0, "the sub-24 cell drained");
        assert_eq!(kind_code(w.tiles[w.index(10, 10)].liquid_kind()), 0, "re-typed water");
        assert_eq!(w.tiles[w.index(10, 11)].liquid, 30, "the pool below survives");
        assert_eq!(w.tiles[w.index(10, 11)].kind, 0, "no merge tile");
        assert!(sys.drain_merge_events().is_empty(), "the drain is silent");
    }

    #[test]
    fn the_water_cell_down_arm_burns_no_plant() {
        // `Liquid.cs:1278`'s `thisLiquidType != 0` gate: water does not
        // burn the cut plant below; the sub-24 drain still runs.
        let mut w = floor_world(9, 120, 8);
        let plant = w.index(10, 11);
        w.tiles[plant] = Tile { kind: 3, ..Default::default() };
        w.tiles[plant].flags |= FLAG_ACTIVE;
        w.tiles[plant].liquid = 30;
        set_kind(&mut w.tiles[plant], 1);
        put_as(&mut w, 10, 10, 20, 0);
        let mut sys = LiquidSystem::new_flow_fixture(&w);
        sys.liquid_check(&mut w, 10, 10, 0, false, &mut rng());
        assert_eq!(w.tiles[plant].kind, 3, "the plant survives water");
        assert!(w.tiles[plant].active(), "still active");
        assert_eq!(w.tiles[w.index(10, 10)].liquid, 0, "the drain ran");
    }

    #[test]
    fn shimmer_merges_with_lava_and_honey_into_659() {
        // `Liquid.cs:1395-1409`/`:1411-1426`: the shimmer cell's lava arm
        // and honey arm both land 659 (`GetLiquidMergeTypes`' shimmer
        // rows), consuming the pooled neighbour.
        for other in [1u8, 2] {
            let mut w = floor_world(9, 120, 8);
            put_as(&mut w, 10, 10, 30, 3);
            put_as(&mut w, 11, 10, 30, other);
            let mut sys = LiquidSystem::new_flow_fixture(&w);
            sys.add_water(&mut w, &mut rng(), 10, 10);
            run(&mut sys, &mut w, 1);
            assert_eq!(w.tiles[w.index(10, 10)].kind, 659, "shimmer + {other} -> 659");
            assert_eq!(w.tiles[w.index(11, 10)].liquid, 0, "the pool consumed");
        }
    }

    /// W4-g45 doc-sweep (`8d581953b`, SEAL carry) — the fall arm reads the
    /// LIVE post-evaporation level (`Liquid.cs:570-575`), not the entry
    /// snapshot (`:464`'s `byte liquid` capture is the kill-bookkeeping
    /// value only). A 100-water cell one row above the floor INSIDE the
    /// band evaporates to 98 on visit 1, then the down arm moves the 98 —
    /// the pre-fix port moved the pre-evap 100, resurrecting the two shed
    /// units on every fall. (The union's `own_now` form carries the fix;
    /// this is the discriminating witness.) Written, NOT run.
    #[test]
    fn the_fall_arm_moves_the_post_evaporation_level() {
        // shaft(5, 400, 350, 2): floor at 356, layer at 212; the cell at
        // (8, 354) sits two rows above the floor, deep in the band, with
        // the open (8, 355) below it.
        let mut w = shaft(5, 400, 350, 2);
        put_as(&mut w, 8, 354, 100, 0);
        let mut sys = LiquidSystem::new(&w);
        sys.add_water(&mut w, &mut rng(), 8, 354);
        run(&mut sys, &mut w, 1);
        assert_eq!(w.tiles[w.index(8, 354)].liquid, 0, "the source drained");
            let __bi = w.index(8, 355);
        assert_eq!(
            w.tiles[__bi].liquid, 98,
            "the fall moved the post-evap 98, not the pre-evap 100"
        );
        // Visit 2 clears the skip flag; visit 3 the landed cell evaporates
        // 98 -> 96 and stays (the floor below is solid, the walls fence the
        // row arms).
        run(&mut sys, &mut w, 2);
        assert_eq!(w.tiles[w.index(8, 355)].liquid, 96, "the band keeps shedding");
    }

    // SEAL row-4 dedupe (research/69 A9.2): the g45 `99a4f8ccf` shimmer-flow
    // twin `shimmer_falls_every_visit_like_water` DROPS here — its
    // cadence-trace asserts (the drained source, the five-falls-in-ten
    // landing, the no-every-visit-flow zero, the type ride) merged into
    // `liquid_sim::shimmer_falls_every_visit_ungated` (ONE pin, both
    // assert families, the g47 shelf fixture).
}

/// W4-g45 carry-forward (union re-fold per research/252's witness table) —
/// the DelWater/sweep/buffer/stuck witnesses that survive the dedupe
/// against the g47 `liquid_settle` module: the delay-inheritance sweep
/// fixture (the sharper form of g47's
/// `delwater_swap_remove_rides_kill_but_not_delay`), the lava grass-ring
/// burn (the mud-60 ring fixture — now ALSO asserting the g47-adopted
/// `square_tile_frame_record` records on `edit_kills`), the dry-pad sink
/// (the grass-family ground fixture), the buffer promotion's delay-0
/// assertion and the kill-0 stuck force-clear. Dropped per the table: the
/// g45 duplicates `the_sweep_promotes_a_254_cell_to_255` /
/// `the_keep_alive_branch_rescues_a_draining_cell` (g47's
/// `the_retirement_sweep_promotes_254_before_delwater` /
/// `delwater_keeps_a_full_cell_over_an_open_below_alive` keep those pins)
/// and `the_alch_tail_kills_an_unsupported_herb` (g47's
/// `delwater_retires_a_submerged_herb_through_check_alch` carries the same
/// pin WITH the herb-drop assertion). Written, NOT run (code-only mode).
#[cfg(test)]
mod liquid_delwater {
    use super::*;

    // W4-g83 compose: the liquid pass threads the caller's ACTIVE genRand
    // stream; these witnesses pin levels/merges, not streams - a fresh
    // seed-0 stream per call (the tests::rng convention).
    fn rng() -> crate::UnifiedRandom {
        crate::UnifiedRandom::new(0)
    }
    use crate::tile::FLAG_ACTIVE;

    /// W4-g45 round 2: the scene is embedded at (+6,+6) (world +12/+12,
    /// floor at `floor_y+6`) so every cell clears AddWater's 5-tile border.
    fn floor_world(width: i32, height: i32, floor_y: i32) -> World {
        let mut w = World::new("delwater".into(), width + 12, height + 12);
        for x in 0..width + 12 {
            let i = w.index(x, floor_y + 6);
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
        put_as(&mut w, 12, 13, 1, 1);
        put_as(&mut w, 14, 10, 100, 0);
        let mut sys = LiquidSystem::new_flow_fixture(&w);
        sys.add_water(&mut w, &mut rng(), 12, 13);
        sys.add_water(&mut w, &mut rng(), 14, 10);
        sys.active[0].delay = 4;
        sys.active[0].kill = KILL_PASSES;
        sys.active[1].delay = 0;
        sys.active[1].kill = 0;
        sys.end_pass(&mut w, &mut rng());
        assert_eq!(sys.active_len(), 1, "the film retired, the column survived");
        assert_eq!(sys.active[0].x, 14, "the survivor's coordinates");
        assert_eq!(sys.active[0].delay, 4, "the survivor inherited the slot's delay");
        assert_eq!(w.tiles[w.index(12, 13)].liquid, 0, "the film dried (`:1481`)");
    }

    #[test]
    fn lava_retirement_burns_the_grass_ring() {
        // `Liquid.cs:1532-1560`: the retired lava cell's 3×3 ring turns the
        // grass family (2/23/109/199/477/492) to dirt 0 and the mud/ash
        // family (60/70/661/662) to 59. Interior coordinates keep the
        // SquareTileFrame 5-tile margin clear. Union re-pin (research/252):
        // the converts now queue `square_tile_frame_record` cascades on
        // `edit_kills` (the g46 loot seam g47 adopted at d3a30ebf) — the
        // ring witness asserts the RECORDS, not just the conversion.
        let mut w = floor_world(17, 120, 8);
        put_as(&mut w, 14, 13, 255, 1);
        put_tile(&mut w, 13, 13, 2);
        put_tile(&mut w, 15, 13, 2);
        put_tile(&mut w, 14, 12, 109);
        put_tile(&mut w, 14, 14, 60); // mud in the ring -> 59
        let mut sys = LiquidSystem::new_flow_fixture(&w);
        sys.add_water(&mut w, &mut rng(), 14, 13);
        sys.active[0].kill = KILL_PASSES;
        sys.end_pass(&mut w, &mut rng());
        assert_eq!(w.tiles[w.index(13, 13)].kind, 0, "grass -> dirt");
        assert_eq!(w.tiles[w.index(15, 13)].kind, 0, "grass -> dirt");
        assert_eq!(w.tiles[w.index(14, 12)].kind, 0, "hallowed grass -> dirt");
        assert_eq!(w.tiles[w.index(14, 14)].kind, 59, "the ring mud -> 59");
        assert_eq!(sys.drain_edit_kills().len(), 4, "one record per convert");
    }

    #[test]
    fn the_lily_pad_tail_sinks_a_dry_pad_into_the_cell_below() {
        // `Liquid.cs:1584-1597`: a retired 518 cell with an empty below and
        // no liquid above runs CheckLilyPad — the dry pad sinks one row. The
        // ground walk must find a grass family (dirt-grass 2 here → frame row
        // 0); the empty cell below the pad takes it. Union re-pin: the g47
        // `check_lily_pad(world, x, y, raw_solid)` seam carries the move;
        // the pad's sink queues no kill record (the move returns None).
        let mut w = floor_world(17, 120, 8);
        // 518 is a TileFrameImportant tile (`WorldGen.cs:84966-84968`); the
        // reframe arm dispatches to CheckLilyPad only when the flag is up.
        w.frame_important[518] = true;
        put_tile(&mut w, 14, 12, 518);
        put_tile(&mut w, 14, 14, 2); // dirt-grass ground for the frame walk
        let mut sys = LiquidSystem::new_flow_fixture(&w);
        // The pad holds no liquid; add_water refuses it, so seed the entry
        // directly (the C# reaches DelWater through a kill sweep on a cell
        // that once held liquid).
        sys.active.push(Entry { x: 14, y: 12, kill: KILL_PASSES, delay: 0 });
        sys.set_checking(14, 12, true);
        sys.end_pass(&mut w, &mut rng());
        assert!(!w.tiles[w.index(14, 12)].active(), "the source pad deactivated");
        assert_eq!(w.tiles[w.index(14, 13)].kind, 518, "the pad sank into (14,13)");
        assert!(w.tiles[w.index(14, 13)].active(), "the destination is active");
    }

    #[test]
    fn the_buffer_promotion_clears_checking_and_resets_delay() {
        // `Liquid.cs:1125-1130` + `LiquidBuffer.cs:11-20`: a buffered cell is
        // marked checkingLiquid; the cycle-end promotion clears it, AddWater
        // re-queues with delay 0, DelBuffer swap-removes the slot. The
        // promotion count is the live active count (`num8 = numLiquid`), so a
        // survivor must remain after the sweep for the buffer to drain.
        // Union note: the g47 `the_buffer_overflow_marks_checking_and_the_
        // promotion_swap_removes` pins the swap-remove order; this witness
        // keeps the unique delay-0 assertion.
        let mut w = floor_world(15, 120, 30);
        put_as(&mut w, 11, 10, 100, 0); // survivor: open below, keep-alive
        put_as(&mut w, 12, 13, 1, 0); // film: retires in the sweep
        put_as(&mut w, 14, 13, 100, 0); // overflow: buffered
        let mut sys = LiquidSystem::new_flow_fixture(&w);
        sys.cur_max_liquid = 3; // active cap = 2
        sys.add_water(&mut w, &mut rng(), 11, 10);
        sys.add_water(&mut w, &mut rng(), 12, 13);
        sys.add_water(&mut w, &mut rng(), 14, 13);
        assert_eq!(sys.active_len(), 2, "two cells fit the active cap");
        assert_eq!(sys.buffered_len(), 1, "the third cell overflowed");
        assert!(sys.is_checking(14, 13), "the buffered cell is checking");
        sys.active[1].kill = KILL_PASSES; // the film retires
        sys.end_pass(&mut w, &mut rng());
        assert_eq!(sys.buffered_len(), 0, "the buffer drained");
        assert_eq!(sys.active_len(), 2, "survivor + promoted cell");
        assert!(sys.is_checking(14, 13), "the promoted cell is checking again");
        let promoted = sys.active.iter().find(|e| e.x == 14).expect("promoted");
        assert_eq!(promoted.delay, 0, "AddWater re-queued with delay 0");
    }

    #[test]
    fn the_stuck_window_force_clears_after_10000_cycles() {
        // `Liquid.cs:1131-1149`: an active count pinned within ±50 of
        // stuckAmount for 10 000 cycle-ends force-clears the list. The cell
        // is a draining water column (open below) with kill 0, so the normal
        // sweep leaves it; only the force-clear (stuck up, keep-alive
        // disabled) drains it. Union note: g47's
        // `the_stuck_band_self_clears_after_10000_cycles` ages the entry to
        // kill 8 first; this witness keeps the unique kill-0 survivor form.
        let mut w = floor_world(15, 120, 30);
        put_as(&mut w, 13, 10, 100, 0);
        let mut sys = LiquidSystem::new_flow_fixture(&w);
        sys.add_water(&mut w, &mut rng(), 13, 10);
        sys.stuck_amount = 1;
        sys.stuck_count = 9_999;
        sys.end_pass(&mut w, &mut rng());
        assert_eq!(sys.active_len(), 0, "the force-clear drained the list");
        assert_eq!(sys.stuck_count, 0, "the force-clear reset the counter");
        assert!(!sys.stuck, "stuck lowered after the clear");
    }
}

/// W4-g45 carry-forward (union re-fold) — the 3-neighbour `genRand` 254→255
/// nudge (`Liquid.cs:892-895`) and the 7-neighbour extra wake pass
/// (`:748-771`). RE-PIN (research/252 witness table): the nudge rides the g46
/// per-consumer `LiquidSystem::surface_rng` (world-seeded), NOT g45's
/// shared `World.frame_rng`; the seed pins below re-derive on that stream.
/// The extra-wake witness is RE-DERIVED onto the 7-arm (the C#'s 3-cell arm
/// has NO extra wakes — `:889-909` — so g45's original fixture could not
/// observe them). Written, NOT run (code-only mode).
#[cfg(test)]
mod liquid_nudge_wakes {
    use super::*;

    // W4-g83 compose: the liquid pass threads the caller's ACTIVE genRand
    // stream; these witnesses pin levels/merges, not streams - a fresh
    // seed-0 stream per call (the tests::rng convention).
    fn rng() -> crate::UnifiedRandom {
        crate::UnifiedRandom::new(0)
    }
    use crate::tile::FLAG_ACTIVE;

    /// W4-g45 round 2: the scene is embedded at (+6,+6) (world +12/+12,
    /// floor at `floor_y+6`) so every cell clears AddWater's 5-tile border.
    fn floor_world(width: i32, height: i32, floor_y: i32) -> World {
        let mut w = World::new("nudge".into(), width + 12, height + 12);
        for x in 0..width + 12 {
            let i = w.index(x, floor_y + 6);
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
        // `UnifiedRandom` draw is 0 (the RNG census), so the settled 254 row
        // promotes. Three equal 254 cells average to 254. RE-PIN: the draw
        // rides `surface_rng`, seeded from `world.seed` in `new`.
        let mut w = floor_world(15, 120, 8);
        w.seed = 35;
        put_as(&mut w, 12, 13, 254, 0);
        put_as(&mut w, 13, 13, 254, 0);
        put_as(&mut w, 14, 13, 254, 0);
        let mut sys = LiquidSystem::new_flow_fixture(&w);
        sys.add_water(&mut w, &mut rng(), 13, 13);
        sys.update(&mut w, &mut rng());
        assert_eq!(w.tiles[w.index(13, 13)].liquid, 255, "the zero draw promoted 254");
    }

    #[test]
    fn the_254_nudge_keeps_254_on_a_nonzero_draw() {
        // Seed 0's first `Next(30)` draw is 21 (the census), so the 254 row
        // stays 254 — the nudge only fires on a zero. RE-PIN: `surface_rng`.
        let mut w = floor_world(15, 120, 8);
        w.seed = 0;
        put_as(&mut w, 12, 13, 254, 0);
        put_as(&mut w, 13, 13, 254, 0);
        put_as(&mut w, 14, 13, 254, 0);
        let mut sys = LiquidSystem::new_flow_fixture(&w);
        sys.add_water(&mut w, &mut rng(), 13, 13);
        sys.update(&mut w, &mut rng());
        assert_eq!(w.tiles[w.index(13, 13)].liquid, 254, "the nonzero draw kept 254");
    }

    #[test]
    fn the_nudge_draw_only_advances_the_stream_on_a_254() {
        // The C# consumes the shared `genRand` draw ONLY when the average is
        // 254 (`:892`'s `num == 254f` guard). A non-254 settle leaves the
        // stream untouched; a 254 settle advances it by exactly one draw.
        // RE-PIN: the observed stream is `surface_rng`.
        let mut w = floor_world(15, 120, 8);
        w.seed = 0;
        put_as(&mut w, 12, 13, 100, 0);
        put_as(&mut w, 13, 13, 100, 0);
        put_as(&mut w, 14, 13, 100, 0);
        let mut sys = LiquidSystem::new_flow_fixture(&w);
        let before = sys.surface_rng.peek();
        sys.add_water(&mut w, &mut rng(), 13, 13);
        sys.update(&mut w, &mut rng());
        assert_eq!(sys.surface_rng.peek(), before, "a non-254 settle drew nothing");
    }

    #[test]
    fn the_extra_wake_requeues_an_unqueued_neighbour_on_a_centre_change() {
        // `Liquid.cs:748-771`: the 7-arm's extra wake re-queues a neighbour
        // when the CENTRE differs from the average, EVEN IF that neighbour's
        // own level already equals the average (so `set_level` did not wake
        // it). RE-DERIVED fixture (the C#'s 3-cell arm has no extra wakes —
        // `:889-909` — so g45's original 3-cell fixture could not observe
        // them): the centre (14,13) holds 100, five row neighbours hold 20
        // and the far neighbour (17,13) holds the fixed point of the
        // average, n = Round((100 + 5*20 + n)/7) → n = 33 (Round(233/7) =
        // 33). The 7-arm averages to 33; `set_level` writes 33 into the five
        // 20-cells (waking them) but sees NO change at (17,13) (33 == 33)
        // and does not wake it. The centre (100) differs from the average
        // (33), so the extra wake — reading the PRE-write centre — re-queues
        // (17,13). Without the extra wake (17,13) would stay unqueued.
        let mut w = floor_world(17, 120, 8);
        put_as(&mut w, 14, 13, 100, 0);
        for nx in [11, 12, 13, 15, 16] {
            put_as(&mut w, nx, 13, 20, 0);
        }
        put_as(&mut w, 17, 13, 33, 0); // the fixed-point neighbour
        let mut sys = LiquidSystem::new_flow_fixture(&w);
        sys.add_water(&mut w, &mut rng(), 14, 13);
        assert_eq!(sys.active_len(), 1, "only the centre is queued");
        assert!(!sys.is_checking(17, 13), "the far neighbour starts unqueued");
        sys.update(&mut w, &mut rng());
        assert_eq!(w.tiles[w.index(17, 13)].liquid, 33, "set_level saw no change");
        assert!(sys.is_checking(17, 13), "the extra wake re-queued the neighbour");
    }
}

/// W4-g45 round 2 (union re-fold per research/252 step 5) — the
/// `Liquid.AddWater` death-guards (`Liquid.cs:1163`, `:1183-1206`): the
/// 5-tile world-border exclusion, the `type != 546` carve-out, and the
/// `CheckWaterDeath`/`CheckLavaDeath` → `KillTile` of a submerged object
/// (the Tile overloads, `TileObjectData.cs:5083-5119` — W4-e11's symbols).
/// There is NO shimmer-specific AddWater: honey and shimmer ride the
/// `CheckWaterDeath` else-arm, lava rides `CheckLavaDeath`. LOOT UPGRADE
/// (research/252 witness table): the inline-death witness asserts the
/// queued `TileKill` on `edit_kills` instead of g45b's discard. The
/// fixtures are embedded past the border. Written, NOT run (code-only).
#[cfg(test)]
mod liquid_addwater_guards {
    use super::*;

    // W4-g83 compose: the liquid pass threads the caller's ACTIVE genRand
    // stream; these witnesses pin levels/merges, not streams - a fresh
    // seed-0 stream per call (the tests::rng convention).
    fn rng() -> crate::UnifiedRandom {
        crate::UnifiedRandom::new(0)
    }
    use crate::tile::FLAG_ACTIVE;

    /// A 32x24 world with a stone floor at y=18 — every probe cell sits
    /// inside the 5-tile border (x/y in 5..27 / 5..19).
    fn guarded_world() -> World {
        let mut w = World::new("addwater".into(), 32, 24);
        for x in 0..32 {
            let i = w.index(x, 18);
            w.tiles[i] = Tile { kind: 1, ..Default::default() };
            w.tiles[i].flags |= FLAG_ACTIVE;
        }
        w.frame_important = vec![false; td::TILE_COUNT];
        w.compute_column_surface();
        w.compute_chunks();
        w
    }

    fn wet(w: &mut World, x: i32, y: i32, amount: u8, kind: u8) {
        let i = w.index(x, y);
        w.tiles[i].liquid = amount;
        set_kind(&mut w.tiles[i], kind);
        w.mark_chunk(x, y);
    }

    fn object(w: &mut World, x: i32, y: i32, kind: u16) {
        let i = w.index(x, y);
        w.tiles[i] = Tile { kind, ..Default::default() };
        w.tiles[i].flags |= FLAG_ACTIVE;
        w.mark_chunk(x, y);
    }

    #[test]
    fn the_five_tile_border_refuses_the_outer_band() {
        // `Liquid.cs:1163`: `x >= maxTilesX - 5 || y >= maxTilesY - 5 ||
        // x < 5 || y < 5` returns before the queue — all four sides.
        let mut w = guarded_world();
        for (x, y) in [(4, 10), (27, 10), (10, 4), (10, 19)] {
            wet(&mut w, x, y, 100, 0);
        }
        let mut sys = LiquidSystem::new_flow_fixture(&w);
        for (x, y) in [(4, 10), (27, 10), (10, 4), (10, 19)] {
            sys.add_water(&mut w, &mut rng(), x, y);
        }
        assert_eq!(sys.active_len(), 0, "the border band never queues");
        // The first interior cell on each axis queues (the guard is `< 5`
        // / `>= size - 5`, so 5 and size-6 pass). The bottom interior row
        // (y = size-6 = 18) is the solid floor row, which AddWater's solid
        // arm refuses — open x=20 there so the border pass itself is seen.
        let hole = w.index(20, 18);
        w.tiles[hole] = Tile::default();
        for (x, y) in [(5, 10), (26, 10), (10, 5), (20, 18)] {
            wet(&mut w, x, y, 100, 0);
            sys.add_water(&mut w, &mut rng(), x, y);
        }
        assert_eq!(sys.active_len(), 4, "the inner edge of the band queues");
    }

    #[test]
    fn the_grate_546_is_carved_out_of_the_solid_refusal() {
        // `Liquid.cs:1163`'s `tile.type != 546`: an active solid grate WITH
        // liquid queues even outside the pass (stock `tileSolid[546] =
        // true`, `Main.cs:10124`); a plain solid stone cell does not.
        let mut w = guarded_world();
        object(&mut w, 10, 10, 546);
        wet(&mut w, 10, 10, 100, 0);
        object(&mut w, 12, 10, 1);
        wet(&mut w, 12, 10, 100, 0);
        let mut sys = LiquidSystem::new_flow_fixture(&w);
        assert!(!sys.ignore_solids, "outside the pass the grate is stock-solid");
        sys.add_water(&mut w, &mut rng(), 10, 10);
        sys.add_water(&mut w, &mut rng(), 12, 10);
        assert_eq!(sys.active_len(), 1, "the grate queues, the stone refuses");
        assert!(sys.is_checking(10, 10));
        assert!(!sys.is_checking(12, 10));
    }

    #[test]
    fn water_kills_a_submerged_torch() {
        // `Liquid.cs:1195-1201`: the else-arm `CheckWaterDeath(tile)` —
        // torch 4 resolves to its own def (`water_death: true`,
        // `uses_global_liquid_checks: false`, registry D_4), so the
        // submerged torch dies INLINE and keeps its liquid. LOOT UPGRADE
        // (research/252): the kill record queues on `edit_kills` — g45b
        // discarded it.
        let mut w = guarded_world();
        object(&mut w, 10, 10, 4);
        wet(&mut w, 10, 10, 100, 0);
        let mut sys = LiquidSystem::new_flow_fixture(&w);
        sys.add_water(&mut w, &mut rng(), 10, 10);
        assert!(!w.tiles[w.index(10, 10)].active(), "the torch died");
        assert_eq!(w.tiles[w.index(10, 10)].liquid, 100, "the liquid survives the kill");
        assert_eq!(sys.active_len(), 1, "the cell is queued before the death check");
        assert!(
            !sys.drain_edit_kills().is_empty(),
            "the inline death's TileKill queues on the loot seam"
        );
    }

    #[test]
    fn the_death_arms_read_different_flags() {
        // `Liquid.cs:1188-1198`: the lava arm reads `CheckLavaDeath`, the
        // else-arm `CheckWaterDeath` — DIFFERENT flags on the same cell.
        // Torch 4 style 0 (registry D_4: `water_death: true`, `lava_death:
        // true`) dies in both; the water-torch sub-tile 8 at frameY 176
        // (D_4_S8_A3: both false — the frame decode flips the flags on the
        // SAME kind) survives both.
        for (frame_y, liquid, dies) in
            [(0i16, 0u8, true), (0, 1, true), (176, 0, false), (176, 1, false)]
        {
            let mut w = guarded_world();
            object(&mut w, 10, 10, 4);
            let __bi = w.index(10, 10);
            w.tiles[__bi].frame_y = frame_y;
            wet(&mut w, 10, 10, 100, liquid);
            let mut sys = LiquidSystem::new_flow_fixture(&w);
            sys.add_water(&mut w, &mut rng(), 10, 10);
            assert_eq!(
                !w.tiles[w.index(10, 10)].active(),
                dies,
                "torch frameY {frame_y} in liquid {liquid}: dies = {dies}"
            );
        }
        // The global-table fallback (`TileObjectData.cs:5090-5093`): plant
        // 3 is a natural object (`uses_global_liquid_checks: true`), so the
        // arms read `Main.tileWaterDeath` (no 3) / `Main.tileLavaDeath`
        // (has 3): survives water, dies in lava — the two arms diverge on
        // one kind through the GLOBAL tables, not a def.
        for (liquid, dies) in [(0u8, false), (1, true)] {
            let mut w = guarded_world();
            object(&mut w, 10, 10, 3);
            wet(&mut w, 10, 10, 100, liquid);
            let mut sys = LiquidSystem::new_flow_fixture(&w);
            sys.add_water(&mut w, &mut rng(), 10, 10);
            assert_eq!(
                !w.tiles[w.index(10, 10)].active(),
                dies,
                "plant 3 in liquid {liquid}: dies = {dies}"
            );
        }
        // The def WINS over the global table: pile 372 (D_372:
        // `water_death: false`, non-global, no sub-tiles/alternates)
        // survives water even though `Main.tileWaterDeath` has 372 — the
        // resolved-def arm answers before the fallback
        // (`TileObjectData.cs:5090-5094`, the d114 discriminator kind).
        let mut w = guarded_world();
        object(&mut w, 10, 10, 372);
        wet(&mut w, 10, 10, 100, 0);
        let mut sys = LiquidSystem::new_flow_fixture(&w);
        sys.add_water(&mut w, &mut rng(), 10, 10);
        assert!(w.tiles[w.index(10, 10)].active(), "the def's false beats the global true");
    }

    #[test]
    fn honey_and_shimmer_ride_the_water_death_arm() {
        // `Liquid.cs:1188-1198`: only `tile.lava()` takes the lava arm —
        // honey (2) and shimmer (3) both fall to `CheckWaterDeath`. The
        // torch's `water_death: true` kills it under either kind.
        for kind in [2u8, 3] {
            let mut w = guarded_world();
            object(&mut w, 10, 10, 4);
            wet(&mut w, 10, 10, 100, kind);
            let mut sys = LiquidSystem::new_flow_fixture(&w);
            sys.add_water(&mut w, &mut rng(), 10, 10);
            assert!(
                !w.tiles[w.index(10, 10)].active(),
                "kind {kind} killed the torch through CheckWaterDeath"
            );
        }
    }

    #[test]
    fn an_open_water_cell_survives_and_queues() {
        // `Liquid.cs:1183-1186`: `!tile.active()` returns right after the
        // queue — no death check reads an empty cell.
        let mut w = guarded_world();
        wet(&mut w, 10, 10, 100, 0);
        let mut sys = LiquidSystem::new_flow_fixture(&w);
        sys.add_water(&mut w, &mut rng(), 10, 10);
        assert_eq!(sys.active_len(), 1);
        assert!(!w.tiles[w.index(10, 10)].active(), "still an open cell");
        assert_eq!(w.tiles[w.index(10, 10)].liquid, 100);
    }

    #[test]
    fn generation_queues_without_the_death_check() {
        // `Liquid.cs:1183`'s `WorldGen.isGeneratingOrLoadingWorld`: during
        // generation the queue lands but the kill never runs (the port's
        // `generating` field IS that flag — armed during the LOAD settle
        // too, the g47c reconciliation).
        let mut w = guarded_world();
        object(&mut w, 10, 10, 4);
        wet(&mut w, 10, 10, 100, 0);
        let mut sys = LiquidSystem::new_flow_fixture(&w);
        sys.generating = true;
        sys.add_water(&mut w, &mut rng(), 10, 10);
        assert_eq!(sys.active_len(), 1, "the cell queued");
        assert!(w.tiles[w.index(10, 10)].active(), "generation never kills");
    }

    #[test]
    fn the_promotion_runs_the_death_check_too() {
        // `Liquid.cs:1127-1128`: the cycle-end promotion calls AddWater —
        // so a buffered torch cell dies when it promotes, exactly like a
        // direct wake. Cap 2: one survivor + one film; the sweep retires
        // the film, freeing the slot the torch promotes into.
        let mut w = guarded_world();
        wet(&mut w, 8, 10, 100, 0); // survivor: open below, keep-alive
        wet(&mut w, 9, 10, 1, 0); // film: the sweep dries + retires it
        object(&mut w, 10, 10, 4);
        wet(&mut w, 10, 10, 100, 0); // overflow: buffered
        let mut sys = LiquidSystem::new_flow_fixture(&w);
        sys.cur_max_liquid = 3; // active cap = 2
        sys.add_water(&mut w, &mut rng(), 8, 10);
        sys.add_water(&mut w, &mut rng(), 9, 10);
        sys.add_water(&mut w, &mut rng(), 10, 10);
        assert_eq!(sys.buffered_len(), 1, "the torch overflowed");
        assert!(w.tiles[w.index(10, 10)].active(), "buffering never kills");
        sys.active[1].kill = KILL_PASSES; // the film retires in the sweep
        sys.end_pass(&mut w, &mut rng());
        assert_eq!(sys.buffered_len(), 0, "the buffer drained");
        assert!(!w.tiles[w.index(10, 10)].active(), "the promotion's AddWater killed the torch");
        assert_eq!(w.tiles[w.index(10, 10)].liquid, 100, "the liquid survives");
        assert_eq!(sys.active_len(), 2, "survivor + the promoted torch entry");
        // The C# AddWater queues BEFORE the kill and never removes the
        // entry — the killed cell's entry stays queued this cycle (the
        // next sweep's solid-kill retires it).
        assert!(
            sys.active.iter().any(|e| e.x == 10 && e.y == 10),
            "the killed cell's entry stays queued this cycle"
        );
    }

    #[test]
    fn the_promotion_count_is_pinned_before_the_loop() {
        // `Liquid.cs:1120-1129`: `num8 = min(numLiquid, numLiquidBuffer)`
        // is pinned BEFORE the loop and the loop runs unconditionally.
        // A buffered cell that went DRY while buffered is refused by
        // AddWater (`:1163`'s `liquid == 0`), but its promotion slot still
        // runs: `DelBuffer(0)` swap-removes it and the tail takes slot 0.
        // Cap 1: one survivor + buffered [A(dry), B]. num8 = min(1, 2) = 1:
        // A's slot clears its checking bit, AddWater refuses the dry cell,
        // DelBuffer swaps B into slot 0 → buffer [B]. The port's old extra
        // `active.len() < cap` loop guard skipped the whole loop at the
        // cap and left the dead A buffered with its checking bit set.
        let mut w = guarded_world();
        wet(&mut w, 8, 10, 100, 0); // survivor: keep-alive
        wet(&mut w, 10, 10, 100, 0); // A: buffered, then goes dry
        wet(&mut w, 12, 10, 100, 0); // B: stays buffered
        let mut sys = LiquidSystem::new_flow_fixture(&w);
        sys.cur_max_liquid = 2; // active cap = 1
        sys.add_water(&mut w, &mut rng(), 8, 10);
        sys.add_water(&mut w, &mut rng(), 10, 10);
        sys.add_water(&mut w, &mut rng(), 12, 10);
        assert_eq!(sys.active_len(), 1);
        assert_eq!(sys.buffered_len(), 2);
        let __bi = w.index(10, 10);
        w.tiles[__bi].liquid = 0; // A dries while buffered
        sys.end_pass(&mut w, &mut rng());
        assert_eq!(sys.active_len(), 1, "the dry cell never queues");
        assert_eq!(sys.buffer, vec![(12, 10)], "DelBuffer swapped B into slot 0");
        assert!(!sys.is_checking(10, 10), "A's promotion cleared its checking bit");
        assert!(sys.is_checking(12, 10), "B stays marked");
    }

    #[test]
    fn add_water_and_the_death_check_draw_no_rng() {
        // RNG census: AddWater itself draws NOTHING — the border/solid
        // gates, the queue and the death check are RNG-free. The kill
        // cascade's framing re-rolls ride `World.frame_rng` (the genRand
        // stand-in, `WorldGen.cs:81705-81709`) only where the C# rolls:
        // `reframe_area`'s centre reset needs an ACTIVE centre, and the
        // killed cell is inactive — an isolated torch death in an empty
        // neighbourhood consumes no draw. (A cascade that kills or
        // re-frames further cells advances the stream exactly as the
        // DelWater witnesses pin.)
        let mut w = guarded_world();
        w.frame_rng = crate::unified_random::UnifiedRandom::new(42);
        wet(&mut w, 8, 10, 100, 0);
        let before = w.frame_rng.peek();
        let mut sys = LiquidSystem::new_flow_fixture(&w);
        sys.add_water(&mut w, &mut rng(), 8, 10);
        assert_eq!(w.frame_rng.peek(), before, "a plain wake draws nothing");
        object(&mut w, 10, 10, 4);
        wet(&mut w, 10, 10, 100, 0);
        sys.add_water(&mut w, &mut rng(), 10, 10);
        assert!(!w.tiles[w.index(10, 10)].active(), "the torch died");
        assert_eq!(
            w.frame_rng.peek(),
            before,
            "the isolated death's reframe has no active centre to re-roll"
        );
    }
}

/// R-12-5 driver 3 witnesses: the f25 gen-module export surface carried onto
/// the g47 machinery (research/252 step 6) + the shared gen settle-loop
/// driver (`WorldGen.cs:16158-16206`). Union re-pin: the fixtures ride the
/// (+6,+6) embed past AddWater's 5-tile border. Written, NOT run (code-only
/// mode).
#[cfg(test)]
mod liquid_gen_driver {
    use super::*;

    // W4-g83 compose: the liquid pass threads the caller's ACTIVE genRand
    // stream; these witnesses pin levels/merges, not streams - a fresh
    // seed-0 stream per call (the tests::rng convention).
    fn rng() -> crate::UnifiedRandom {
        crate::UnifiedRandom::new(0)
    }
    use crate::tile::FLAG_ACTIVE;

    /// A walled shaft wide enough for the gen sweep's `x in [4, width-4)`
    /// column range and deep enough for its `maxTilesY - 5` fall bound.
    /// W4-g45: the (+6,+6) embed (world +12/+12, floor at `floor_y+6`).
    fn shaft(width: i32, height: i32, floor_y: i32, cx: i32) -> World {
        let mut w = World::new("liquid_gd".into(), width + 12, height + 12);
        for y in 0..=floor_y + 6 {
            for x in [cx + 5, cx + 7] {
                let i = w.index(x, y);
                w.tiles[i] = Tile { kind: 1, ..Default::default() };
                w.tiles[i].flags |= FLAG_ACTIVE;
            }
        }
        for x in 0..width + 12 {
            let i = w.index(x, floor_y + 6);
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
    fn world_gen_tiles_ignore_water_lifts_the_door_family() {
        // `Liquid.cs:75-81`: the doors 10/192 + living wood/leaf 191/190
        // lose solidity while the gen flag is up; the runtime toggles do
        // NOT touch them (the two families are disjoint).
        let mut w = shaft(9, 60, 50, 4);
        let mut sys = LiquidSystem::new(&w);
        for kind in [10u16, 192, 191, 190] {
            let t = Tile { kind, flags: FLAG_ACTIVE, ..Default::default() };
            assert!(sys.tile_is_solid(&t), "kind {kind} solid stock");
        }
        sys.world_gen_tiles_ignore_water(true);
        for kind in [10u16, 192, 191, 190] {
            let t = Tile { kind, flags: FLAG_ACTIVE, ..Default::default() };
            assert!(!sys.tile_is_solid(&t), "kind {kind} lifted for the pass");
        }
        sys.world_gen_tiles_ignore_water(false);
        let t = Tile { kind: 10, flags: FLAG_ACTIVE, ..Default::default() };
        assert!(sys.tile_is_solid(&t), ":16207 restores the family");
    }

    #[test]
    fn clear_pending_liquid_drops_both_lists_and_their_checking_bits() {
        // `WorldGen.cs:80496-80513`: both lists drop, every entry's
        // checkingLiquid clears; the skip bits stay (AddWater clears them
        // on the next wake). An empty pair is a no-op.
        let mut w = shaft(9, 60, 50, 4);
        put_as(&mut w, 10, 16, 100, 0);
        put_as(&mut w, 10, 26, 100, 0);
        let mut sys = LiquidSystem::new(&w);
        sys.set_cur_max_liquid(2);
        sys.add_water(&mut w, &mut rng(), 10, 16);
        sys.add_water(&mut w, &mut rng(), 10, 26); // overflow -> buffer
        assert_eq!(sys.num_liquid_total(), 2);
        sys.set_skip(10, 16, true);
        sys.clear_pending_liquid();
        assert_eq!(sys.num_liquid_total(), 0, ":80510-80511");
        assert!(!sys.is_checking(10, 16), ":80503");
        assert!(!sys.is_checking(10, 26), ":80508");
        assert!(sys.is_skip(10, 16), "the C# leaves the skip bit");
        assert_eq!(w.tiles[w.index(10, 16)].liquid, 100, "no tile mutation");
        sys.clear_pending_liquid(); // the empty-pair early return
    }

    #[test]
    fn the_gen_settle_loop_drains_the_lists_and_drops_quick_settle() {
        // `WorldGen.cs:16158-16206`: WaterCheck re-registers, the 10-round
        // budget loop drains numLiquid to 0, WaterCheck runs between the
        // rounds, ClearPendingLiquid + quickSettle=false close it. The
        // caller arms the gen context + QuickWater pre-settle first.
        let mut w = shaft(9, 60, 50, 4);
        put_as(&mut w, 10, 16, 200, 0);
        let mut sys = LiquidSystem::new(&w);
        sys.set_gen_settle_context(w.height, 0, false);
        sys.world_gen_tiles_ignore_water(true);
        sys.quick_water(&mut w, &mut rng(), 9, -1);
        sys.gen_settle_loop(&mut w, &mut rng());
        assert!(!sys.quick_settle(), ":16206");
        assert_eq!(sys.num_liquid_total(), 0, ":16205 + the exit invariant");
        assert_eq!(w.tiles[w.index(10, 16)].liquid, 0, "the origin drained");
        let total: i32 = w.tiles.iter().map(|t| t.liquid as i32).sum();
        assert_eq!(total, 200, "the gen settle conserves the mass");
        // The blob landed on the floor and settled to 255-or-less rows.
        assert!(w.tiles[w.index(10, 55)].liquid > 0, "the floor holds the mass");
    }

    #[test]
    fn the_gen_settle_loop_on_a_diffusing_film_still_clears_the_tail() {
        // `WorldGen.cs:16162-16205`: whatever the rounds do — drain to 0
        // or break on the `num3 * 5` budget (`:16167-16178`, ported
        // verbatim; the extraLiquid bump is the special-seed arm) — the
        // tail ALWAYS runs ClearPendingLiquid + quickSettle=false. A wide
        // alternating film exercises the diffusion wakes between rounds.
        // W4-g45: the film sits at x 10..42 in the widened 52x42 world.
        let mut w = World::new("liquid_budget".into(), 52, 42);
        for x in 0..52 {
            let i = w.index(x, 31);
            w.tiles[i] = Tile { kind: 1, ..Default::default() };
            w.tiles[i].flags |= FLAG_ACTIVE;
        }
        w.frame_important = vec![false; td::TILE_COUNT];
        w.compute_column_surface();
        w.compute_chunks();
        for x in 10..42 {
            let i = w.index(x, 30);
            w.tiles[i].liquid = if x % 2 == 0 { 3 } else { 2 };
        }
        let mut sys = LiquidSystem::new(&w);
        sys.set_gen_settle_context(w.height, 0, false);
        sys.gen_settle_loop(&mut w, &mut rng());
        assert_eq!(sys.num_liquid_total(), 0, "the tail clears regardless");
        assert!(!sys.quick_settle());
    }
}

/// W4-g45 doc-sweep (`c8908a55`, SEAL carry) — the `LiquidCheck` down arm's
/// LIVE post-burn reads (`Liquid.cs:1278-1306`): the C# `tile4` is a
/// reference, so the merge gate (`:1286`) and the merge-type reads
/// (`:1303`) see the cell AFTER the `tileCut` burn (`:1280`). The seven cut
/// kinds OUTSIDE `tileObsidianKill` (444 BeeHive, 518 LilyPad, 519 Cattail,
/// 549 Seaweed, 637 AshPlants, 638 AshVines, 711 — the generated bitset
/// membership, hand-computed) pass the gate on `!tile4.active()` once the
/// burn cleared them; the pre-kill copy the port used to read refused the
/// drain/merge the C# performs. The existing plant-3 witness
/// (`the_water_cell_down_arm_burns_no_plant`'s lava sibling) uses plant 3,
/// which IS `tileObsidianKill` — it passes either way and cannot
/// discriminate. Written, NOT run (code-only mode).
#[cfg(test)]
mod liquid_downarm_live {
    use super::*;

    // W4-g83 compose: the liquid pass threads the caller's ACTIVE genRand
    // stream; these witnesses pin levels/merges, not streams - a fresh
    // seed-0 stream per call (the tests::rng convention).
    fn rng() -> crate::UnifiedRandom {
        crate::UnifiedRandom::new(0)
    }
    use crate::tile::FLAG_ACTIVE;

    /// The `liquid_merge_cadence` fixture shape: (+6,+6) embed past
    /// AddWater's 5-tile border, floor at `floor_y+6`, stock frame tables.
    fn floor_world(width: i32, height: i32, floor_y: i32) -> World {
        let mut w = World::new("downarm".into(), width + 12, height + 12);
        for x in 0..width + 12 {
            let i = w.index(x, floor_y + 6);
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

    /// An active submerged object below the checking cell.
    fn plant_below(w: &mut World, x: i32, y: i32, kind: u16, amount: u8, liquid: u8) {
        let i = w.index(x, y);
        w.tiles[i] = Tile { kind, ..Default::default() };
        w.tiles[i].flags |= FLAG_ACTIVE;
        w.tiles[i].liquid = amount;
        set_kind(&mut w.tiles[i], liquid);
        w.mark_chunk(x, y);
    }

    #[test]
    fn the_burned_non_obsidian_kill_plant_lets_the_sub_24_drain_run() {
        // `Liquid.cs:1286-1298`: lava 20 over Seaweed 549 (tileCut, NOT
        // tileObsidianKill) holding water 30. The burn clears the plant;
        // the LIVE gate passes on `!tile4.active()` and the sub-24 arm
        // drains the lava cell + re-types it water. The pre-kill copy
        // refused (active 549, not obsidianKill) and kept the lava.
        let mut w = floor_world(9, 120, 8);
        plant_below(&mut w, 10, 11, 549, 30, 0);
        put_as(&mut w, 10, 10, 20, 1);
        let mut sys = LiquidSystem::new_flow_fixture(&w);
        sys.liquid_check(&mut w, 10, 10, 1, false, &mut rng());
        assert!(!w.tiles[w.index(10, 11)].active(), "the seaweed burned");
        assert_eq!(w.tiles[w.index(10, 10)].liquid, 0, "the sub-24 cell drained");
        assert_eq!(kind_code(w.tiles[w.index(10, 10)].liquid_kind()), 0, "re-typed water");
        assert_eq!(w.tiles[w.index(10, 11)].liquid, 30, "the pool below survives");
        assert_eq!(w.tiles[w.index(10, 11)].kind, 0, "no merge tile under 24");
    }

    #[test]
    fn the_burned_lily_pad_lets_the_full_merge_land_below() {
        // `Liquid.cs:1299-1306`: lava 30 over LilyPad 518 (tileCut, NOT
        // tileObsidianKill) holding water 30. The LIVE post-burn reads see
        // an inactive cell with its water intact (`remove_tile` keeps the
        // liquid + type bits, `WorldGen.cs:63885`), so the >=24 arm lands
        // obsidian 56 at (x, y+1) — `GetLiquidMergeTypes(1, water)` reads
        // the surviving water type. The pre-kill copy refused the merge.
        let mut w = floor_world(9, 120, 8);
        plant_below(&mut w, 10, 11, 518, 30, 0);
        put_as(&mut w, 10, 10, 30, 1);
        let mut sys = LiquidSystem::new_flow_fixture(&w);
        sys.liquid_check(&mut w, 10, 10, 1, false, &mut rng());
        assert_eq!(w.tiles[w.index(10, 11)].kind, 56, "the merge landed below");
        assert_eq!(w.tiles[w.index(10, 10)].liquid, 0, "the lava consumed");
        assert_eq!(w.tiles[w.index(10, 11)].liquid, 0, "the water consumed");
    }

    #[test]
    fn the_honey_arm_burns_and_merges_the_same_way() {
        // The burn gate is `thisLiquidType != 0` (`:1278`) — honey rides
        // it too: honey 30 over Cattail 519 with water 30 lands the honey
        // block 229 below (`GetLiquidMergeTypes(2, water)`).
        let mut w = floor_world(9, 120, 8);
        plant_below(&mut w, 10, 11, 519, 30, 0);
        put_as(&mut w, 10, 10, 30, 2);
        let mut sys = LiquidSystem::new_flow_fixture(&w);
        sys.liquid_check(&mut w, 10, 10, 2, false, &mut rng());
        assert_eq!(w.tiles[w.index(10, 11)].kind, 229, "honey + water -> 229");
        assert_eq!(w.tiles[w.index(10, 10)].liquid, 0);
        assert_eq!(w.tiles[w.index(10, 11)].liquid, 0);
    }

    #[test]
    fn the_water_arm_never_burns_the_non_obsidian_kill_plant() {
        // `:1278`'s `thisLiquidType != 0` gate: water over Seaweed 549
        // holding LAVA 30 (a different type, so the `:1269` same-type early
        // return does not fire) burns nothing — the plant stays ACTIVE,
        // and the live gate refuses: active 549 is not obsidianKill and
        // the container flag is false. The water cell keeps its liquid.
        let mut w = floor_world(9, 120, 8);
        plant_below(&mut w, 10, 11, 549, 30, 1);
        put_as(&mut w, 10, 10, 30, 0);
        let mut sys = LiquidSystem::new_flow_fixture(&w);
        sys.liquid_check(&mut w, 10, 10, 0, false, &mut rng());
        assert!(w.tiles[w.index(10, 11)].active(), "water burns no plant");
        assert_eq!(w.tiles[w.index(10, 10)].liquid, 30, "the gate refused");
        assert_eq!(w.tiles[w.index(10, 11)].kind, 549, "no merge tile");
    }
}

