//! The gen-time chest-stocking seam (S1-A-4): the vanilla stocks every world
//! chest **inline with its placement** — `WorldGen.AddBuriedChest`
//! (`WorldGen.cs:35871-37580`) rolls `genRand` at the call site, interleaved
//! with every other worldgen consumer, and the family flags (`flag2`/`flag3`/
//! `flag4`/`flag6`/`flag8`/`flag13`) are world-state probes at the anchor.
//!
//! The port's load-time re-roll (`tr-app`'s `chest_gen.rs` scan) moved here:
//! each chest placement site resolves the anchor's context (`context_at` —
//! the same positional probes the C# computes inside `AddBuriedChest`) and
//! the depth band, then hands a [`ChestSiteEvent`] to the stocking hook. The
//! hook itself lives in `tr-app` (it carries the loot tables and the prefix
//! engine — the app-side port of `AddBuriedChest`'s stocking block) and draws
//! through [`WorldGenRand`], the pass's own stream, so the chest draws
//! interleave with the placing pass's tile draws exactly as the C#'s
//! interleave with `genRand` (the pass-granularity stand-in — the port's
//! per-pass streams are the `WorldGenerator.cs:497` model, recorded in
//! ANALYSIS "World chest loot (S1-A-4)").
//!
//! The RNG-order contract (S1-A-10) entry for the chest loot: stream = the
//! placing pass's stream (`srng` for every chest-placing pass), draw order =
//! the site's own draws (jitter, main-item rolls) then the hook's row/prefix
//! draws, once per successful placement.

use crate::rng::Xorshift64;
use crate::world::World;

/// The anchor's family flags — the port of `AddBuriedChest`'s positional
/// dispatch (the `flag*` arms of `:36010-36205`), resolved by [`context_at`]'s
/// probes the way the C# computes them from the world state at `(i, k)`.
#[derive(Clone, Copy, Debug, Default)]
pub struct ChestCtx {
    /// `flag4` — the jungle family (`chestStyle == 10`, `:36089`).
    pub jungle: bool,
    /// `flag6` — the living-tree family (wall 244 at the anchor,
    /// `:36135-36144`).
    pub living_tree: bool,
    /// The pyramid probe (wall 187 — the C#'s own wall probe for the
    /// structure's chest).
    pub pyramid: bool,
    /// `flag3`'s hardened-sand half (wall 216 — the C#'s
    /// `IsUndergroundDesert` probe's second wall).
    pub desert: bool,
    /// The mahogany probe (wall 63 — the C#'s 467-style-10 wall family).
    pub mahogany: bool,
    /// `flag2` — the ice family (the ice tile probes `:36064-36088`).
    pub ice: bool,
    /// The granite probe (walls 164/401).
    pub granite: bool,
    /// The idol probe (walls 464/241).
    pub idol: bool,
    /// The dungeon family (`Main.wallDungeon` at the anchor,
    /// `IsDungeon`'s probe).
    pub dungeon: bool,
    /// The sky probe (wall 73 — the skyware house family).
    pub sky: bool,
    /// `flag13` — the water family (the underwater chest probe).
    pub water: bool,
}

/// The anchor's depth band (`AddBuriedChest`'s four arms, `:36222`/`:36503`/
/// `:36803`/`:37162`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Band {
    Surface,
    Mid,
    Deep,
    Deepest,
}

/// `AddBuriedChest`'s band pads: the surface arm's `worldSurface + 25` and
/// the deep arm's `maxTilesY - 250` margins (`:36222`'s `num7` gates).
pub const SURFACE_BAND_PAD: i32 = 25;
pub const DEEP_BAND_PAD: i32 = 250;

pub fn band_for(y: i32, world_surface: f64, rock_layer: f64, height: i32) -> Band {
    let num7 = y as f64;
    if num7 < world_surface + SURFACE_BAND_PAD as f64 {
        Band::Surface
    } else if num7 < rock_layer {
        Band::Mid
    } else if num7 < height as f64 - DEEP_BAND_PAD as f64 {
        Band::Deep
    } else {
        Band::Deepest
    }
}

/// The family flags at `(x, y)` — the port of `AddBuriedChest`'s
/// world-state probes (the ±2-tile neighbourhood the C#'s wall/tile reads
/// cover; moved verbatim from the load-time sweep's `context_at`).
pub fn context_at(world: &World, x: i32, y: i32) -> ChestCtx {
    let near = |wx: u16| {
        (-2..=2).any(|dx| {
            (-2..=0).any(|dy| {
                world
                    .tile(x + dx, y + dy)
                    .is_some_and(|t| t.active() && t.kind == wx)
            })
        })
    };
    let wall_near = |wx: u16| {
        (-2..=2).any(|dx| {
            (-2..=0).any(|dy| {
                world
                    .tile(x + dx, y + dy)
                    .is_some_and(|t| t.wall == wx)
            })
        })
    };
    let wall_near_where = |pred: &dyn Fn(u16) -> bool| {
        (-2..=2).any(|dx| {
            (-2..=0).any(|dy| {
                world
                    .tile(x + dx, y + dy)
                    .is_some_and(|t| pred(t.wall))
            })
        })
    };
    let dungeon_wall = |wx: u16| crate::scene::is_dungeon_wall(wx);
    ChestCtx {
        jungle: near(60),
        living_tree: wall_near(244),
        pyramid: wall_near(187),
        desert: wall_near(216),
        mahogany: wall_near(63),
        ice: near(147) || near(161) || near(162) || near(197),
        granite: wall_near(164) || wall_near(401),
        idol: wall_near(464) || wall_near(241),
        dungeon: wall_near_where(&dungeon_wall),
        sky: wall_near(73),
        water: near(4869) || wall_near(24),
    }
}

/// One chest placement's stocking request — everything the C#'s
/// `AddBuriedChest(i, j, mainItemInChest, notNearOtherChests, chestStyle, …)`
/// call site knows, plus the world-state reads the stocking block makes.
#[derive(Clone, Copy, Debug)]
pub struct ChestSiteEvent {
    /// The chest's anchor (left) cell — the placement site's `(i, j)`.
    pub x: i32,
    pub y: i32,
    /// `chestStyle` — the C#'s per-site argument (the jungle shrines' 10,
    /// the pyramid's 1, the hell houses' 12, the sky houses' 13/2, the
    /// spider caves' 15, the scatter's -1). The stocking's style rows read
    /// it verbatim.
    pub chest_style: i32,
    /// The chest tile kind (21 the wooden chest, 467 the mahogany pair).
    pub chest_tile_type: u16,
    /// `mainItemInChest` — the C#'s pre-set main item (the jungle/sky/
    /// pyramid/hell callers roll it before the call; 0 = none).
    pub main_item: u16,
    /// `notNearOtherChests` — the 30-tile exclusion some callers pass.
    pub not_near_other_chests: bool,
    /// `num7 > maxTilesY - 205` — the hell band's gate (`:36223`).
    pub hell: bool,
    /// The anchor's resolved family flags ([`context_at`]).
    pub ctx: ChestCtx,
    /// The anchor's depth band ([`band_for`]).
    pub band: Band,
    /// The band inputs the stocking's deeper arms read (the C#'s
    /// `Main.worldSurface` / `Main.rockLayer` / `Main.maxTilesY`).
    pub world_surface: f64,
    pub rock_layer: f64,
    pub height: i32,
}

/// The pass's stream, as the stocking sees it — the C#'s `genRand` draws
/// (`Next(max)`, `Next(min, max)`) narrowed to the two shapes the stocking
/// block and the prefix engine use. Implemented by the port's pass stream
/// ([`Xorshift64`], the `WorldGenerator.cs:497` per-pass stand-in).
pub trait WorldGenRand {
    /// `UnifiedRandom.Next(max)` — `[0, max)`.
    fn next(&mut self, max: i32) -> i32;
    /// `UnifiedRandom.Next(min, max)` — `[min, max)`.
    fn range(&mut self, lo: i32, hi: i32) -> i32;
}

impl WorldGenRand for Xorshift64 {
    fn next(&mut self, max: i32) -> i32 {
        if max <= 0 {
            return 0;
        }
        self.range(0, max)
    }

    fn range(&mut self, lo: i32, hi: i32) -> i32 {
        Xorshift64::range(self, lo, hi)
    }
}

/// The stocking hook — the tr-app port of `AddBuriedChest`'s stocking block.
/// Returns the rolled rows (the chest's slots, in order); the placement site
/// writes them into `World.chests` as the chest's record. Sites take
/// `Option<&mut ChestStockHook>` (a reborrow of `Streams`'s slot).
pub type ChestStockHook<'a> =
    dyn FnMut(&ChestSiteEvent, &mut dyn WorldGenRand) -> Vec<crate::save::ChestItem> + 'a;

/// The one call every chest placement site makes after a successful
/// `place_chest`: resolve the anchor's context and band (the C#'s
/// `AddBuriedChest` world-state probes), hand the event to the hook, and
/// write the rolled rows into `World.chests`. Without a hook (the tr-core
/// tests' `generate` path) the chest stays unstocked — no record, no draws.
pub(crate) fn stock_placed_chest(
    world: &mut World,
    hook: Option<&mut ChestStockHook<'_>>,
    rng: &mut Xorshift64,
    x: i32,
    y: i32,
    chest_style: i32,
    chest_tile_type: u16,
    main_item: u16,
    not_near_other_chests: bool,
) {
    let Some(hook) = hook else {
        return;
    };
    let ctx = context_at(world, x, y);
    let hell = y > world.height - 205;
    let band = band_for(y, world.world_surface, world.rock_layer, world.height);
    let ev = ChestSiteEvent {
        x,
        y,
        chest_style,
        chest_tile_type,
        main_item,
        not_near_other_chests,
        hell,
        ctx,
        band,
        world_surface: world.world_surface,
        rock_layer: world.rock_layer,
        height: world.height,
    };
    let rows = hook(&ev, rng);
    world.chests.push(crate::save::ChestRecord {
        x,
        y,
        name: String::new(),
        slots: rows,
    });
}
