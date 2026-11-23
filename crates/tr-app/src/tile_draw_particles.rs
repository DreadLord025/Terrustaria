//! The tile-draw particle layer — `TileDrawing.DrawTiles_EmitParticles`
//! (`TileDrawing.cs:6688-7547`) plus its private helpers
//! `EmitLivingTreeLeaf`/`_Below`/`_Sideways` (`:7549-7592`),
//! `EmitAlchemyHerbParticles` (`:9605-9655`) and `IsVisible`
//! (`:1411-1432`). W4-d60, CODE-ONLY MODE (no compiler invoked; the
//! witness tests below are written, not run).
//!
//! ## What this is
//!
//! Every drawn tile rolls a battery of low-probability gates once per
//! frame and, on a hit, spawns dust/gore particles or requests an
//! orchestrator particle: the star-cloud night sparkles (718), the
//! fog-machine plume (565), chimney smoke (406) and balloon-machine
//! bubbles (452), the living-wood leaf shed (192/384), the lava-bubble
//! emitters (42 case 51 / 93 case 45 / 100 case 45 / 34 case 51 / 372 —
//! dust 242), torch smoke (4), herb bloom sparkles (83), ore/gem shine
//! (the `Main.tileShine` chain) and the shimmer sparkle on lit solids.
//! The call site is `DrawSingleTile` (`:965-968`): only when
//! `_isActiveAndNotPaused` and (`!Lighting.UpdateEveryFrame` or the
//! per-tile `FastRandom(TileFrameSeed).WithModifier(x, y).Next(4) == 0`
//! quarter-roll) — the caller owns that gate; this module is the body.
//!
//! The grass SWAY draws themselves landed with TILE-DRAW-3
//! (`render/tiles.rs` + `tr_core::wind`); this module adds the particle
//! EMISSION layer that rides the same wind inputs (`WindForVisuals`,
//! `_leafFrequency`, the `WindGrid` gust window).
//!
//! ## RNG census — TWO streams, kept separate (S1-A-10)
//!
//! The C# interleaves two `UnifiedRandom`s here:
//! * `TileDrawing._rand` (`:104`, seeded `:191`) — every gate `Next`,
//!   the post-spawn slot tweaks (718/215/592/165/...), the gore scale
//!   ARGUMENTS (244/406/452/139/37), the 100/42 position picks, the
//!   nature-fly `NextFloat`/`Next(600)`/`Next(48)`, the rubble
//!   `Next(1200)`/`Next(10)`/`Next(2)`, the shine `Next(tileShine)` and
//!   shimmer `Next(500)`, the leaf `Next(2)`, and the DISCARDED
//!   `Next(2)` of type 646 (`:7256-7259` — a draw with no consumer).
//! * `Main.rand` — the `Dust.NewDust`/`Gore.NewGore` pool internals
//!   (`Dust.cs:75-256`, `Gore.cs:1286-1462`), `Player.RollLuck`
//!   (`Luck.cs:5-16` rolls `Main.rand`, NOT `_rand`), the 565
//!   `NewGorePerfect` arguments (`:6778`) and the 711
//!   `NextVector2Circular` pairs (`:6876-6877`).
//!
//! [`draw_tiles_emit_particles`] takes both (`rand` = `_rand`,
//! `main_rand` = `Main.rand`) and draws in C# order. TILE-DRAW-3 folded
//! the two streams into one pinned `FastRandom` for the inline leaf arm
//! (ANALYSIS "Wind sway and tree leaves"); this module keeps them
//! separate — the wiring wave must reconcile the inline 192 arm
//! (`render/tiles.rs:1078-1122`) with [`emit_living_tree_leaf`] (the
//! inline arm gates on `sway_enabled`, which the C# does NOT, and draws
//! the sideways `Next(2)` only when the wind pick is non-zero — the C#
//! draws it unconditionally after the below-spawn).
//!
//! `WorldGen.genRand` appears only inside `KillTile_MakeTileDust`
//! (`WorldGen.cs:67032` passim) — the rubble arm's spawn body is the
//! d54 (dust-sites-world) seam, deferred wholesale below.
//!
//! ## Deferred seams (dependency notes)
//!
//! * **d50 (dust-type-table)** — parallel branch; this module only
//!   CALLS `DustPool::new_dust` (S1-E-1, landed) and never touches the
//!   per-type colour table (`tr_core::dust_table`, S1-E-2 landed), so
//!   the two lanes do not collide.
//! * **d54 (dust-sites-world)** — `WorldGen.KillTile_MakeTileDust`
//!   (`WorldGen.cs:66687-68600+`) is a ~2000-line type table with its
//!   own `Main.rand`/`genRand` draws. The rubble gate block
//!   (`:6901-6951`) is ported here verbatim (its three `_rand` draws
//!   included); the per-dust spawn body is returned as a
//!   [`RubbleDustPlan`] (count + the surface `flag4` scale cut) for the
//!   d54 wiring. Skipping the body is NOT stream-neutral: each
//!   `KillTile_MakeTileDust` call draws on `Main.rand` (the `NewDust`
//!   internals) and some types on `genRand` — the census is recorded
//!   here so the wiring can replay it.
//! * **d56/d57 (tile-draw animated / TE furniture)** —
//!   `EmitLiquidDrops` (`:7594-7674`, the drip gores under stalactites)
//!   is called from `CacheSpecialDraws` (`:693`), NOT from
//!   `DrawTiles_EmitParticles`; it rides the special-tile cache lane.
//!   The danger-sense / spelunker / biome-sight dust arms
//!   (`:911-962`) are `DrawSingleTile` prologue arms, also out of this
//!   body's scope.
//! * **ParticleOrchestrator** — unported; the four request sites
//!   (PooFly 31, RainbowBoulder3 49, RainbowBoulder2 48, NatureFly 53,
//!   `ParticleOrchestraType.cs:36/54/53/58`) return
//!   [`ParticleRequest`]s carrying the net-module byte kind
//!   (`app/netmodules.rs::Particles`).
//! * **`Tile.invisibleBlock`** — the `bTileHeader3` bit is unported
//!   (the S1-G-1e deviation); [`is_visible`] reads it as `false`, so
//!   only the 19-row-48 / 541 / 631 arms can hide a tile.
//! * **`Main.tileFrame[type]`** (the 452 balloon colour row) — the
//!   animator is unported (`drawdata.rs` convention: 0 until then);
//!   the caller passes the live row as `EmitCtx::tile_frame_anim`.
//!
//! ## Faithfulness notes
//!
//! * The 215 gate (`:6780`) draws `Next(3)` even when `frameY != 0`
//!   (the `frameY < 36 && Next(3) == 0 && frameY == 0` order) — kept.
//! * The torch gate (`:7105`) draws `Next(40)` BEFORE the `frameX < 66`
//!   test — kept; same for 93/100/34 (`:7128/:7171/:7260`).
//! * The 592/406/452 arms are one `if/else-if` chain (`:6796-6847`),
//!   and the 22→shimmer tail (`:7319-7546`) is one long else-if chain:
//!   a FAILED gate falls through to the later arms (a type-22 tile
//!   whose `Next(400)` misses still rolls the shine `Next(1150)`).
//!   Both chains are kept as chains.
//! * The 71/72 paint gate (`:7362`) reads `tileCache.color() == 0`
//!   BEFORE its `Next(500)` — kept.
//! * The shine arm's `Next(tileShine)` (`:7429`) is drawn BEFORE the
//!   frame-window `||` terms — kept; the light `<= 20` early return
//!   (`:7415-7418`) precedes it.
//! * `MathHelper.Clamp(frameY / 22, 0, 23)` (torch, `:7107`) is an
//!   INTEGER divide then a float clamp — `(frame_y / 22).clamp(0, 23)`.
//! * The 139/497 arms read the RAW `tileCache.frameX/frameY`, not the
//!   `GetTileDrawData`-resolved `tileFrameX/Y` the other arms use —
//!   both are parameters here.
//! * `Vector2` ctor arguments evaluate left to right (the 565 velocity
//!   sum draws Circular-then-CircularEdge, `:6778`).
//! * The leaf gores' `Frame.CurrentColumn = tile.color()` writes ride
//!   the returned slot — the sentinel included (the C#
//!   `NewGorePerfect(...).Frame` writes `Main.gore[600]` on a refusal).

use glam::Vec2;
use tr_core::gore::GorePool;
use tr_core::growth::{WeatherCtx, is_alchemy_plant_harvestable};
use tr_core::particles::{Color, DustPool, SpawnCtx};
use tr_core::tile::Tile;
use tr_core::tiledata as td;
use tr_core::scene::get_lerp_value;
use tr_core::unified_random::UnifiedRandom;
use tr_core::wind::{WindGrid, wall_allows_wind};
use tr_core::world::World;

/// The `TileDrawing._rand` draw seam (the gate stream). One method per
/// C# conversion; [`UnifiedRandom`] implements it verbatim. The census
/// witnesses use a recording stand-in (the `projectile_ai_misc.rs`
/// CensusRng shape).
pub trait EmitRand {
    /// `Next(max)` — `[0, max)`.
    fn next_max(&mut self, max: i32) -> i32;
    /// `Next(min, max)` — `[min, max)`.
    fn next_range(&mut self, min: i32, max: i32) -> i32;
    /// `Utils.NextFloat` (`Utils.cs:1198-1201`).
    fn next_float(&mut self) -> f32;
    /// `Utils.NextFloatDirection` (`Utils.cs:1203-1206`).
    fn next_float_direction(&mut self) -> f32;
}

impl EmitRand for UnifiedRandom {
    fn next_max(&mut self, max: i32) -> i32 {
        UnifiedRandom::next_max(self, max)
    }
    fn next_range(&mut self, min: i32, max: i32) -> i32 {
        UnifiedRandom::next_range(self, min, max)
    }
    fn next_float(&mut self) -> f32 {
        UnifiedRandom::next_float(self)
    }
    fn next_float_direction(&mut self) -> f32 {
        UnifiedRandom::next_float_direction(self)
    }
}

/// The `Main` statics the body reads beyond the world/tile inputs.
/// (`WeatherCtx` has no `Debug` derive, so this struct carries none.)
#[derive(Clone, Copy)]
pub struct EmitCtx {
    /// `Main.WindForVisuals` (= `windSpeedCurrent`, `Main.cs:2823`).
    pub wind: f32,
    /// `Main.timeForVisualEffects` (`Main.cs:1181`) — the 139 arm's
    /// `(int) % 7` cadence.
    pub time_for_visual_effects: f64,
    /// `TileDrawing._leafFrequency` (`:76`, the `UpdateLeafFrequency`
    /// table ×7 — `tr_core::wind::leaf_frequency`).
    pub leaf_frequency: i32,
    /// `Main.player[Main.myPlayer].luck` (the 718 `RollLuck` gates).
    pub luck: f32,
    /// `Main.shimmerAlpha` (the lit-solid sparkle arm, `:7530`).
    pub shimmer_alpha: f32,
    /// `Main.tileFrame[typeCache]` — the 452 balloon colour row
    /// (`:6842`); 0 until the animator lands (module docs).
    pub tile_frame_anim: i32,
    /// `Main.DiscoR/G/B` (the rainbow-torch dust colour, `:7124`).
    pub disco: [u8; 3],
    /// `_shouldShowInvisibleBlocks` (the echo-coating sight,
    /// `IsVisible` `:1429`).
    pub should_show_invisible_blocks: bool,
    /// The weather clock `IsAlchemyPlantHarvestable` reads
    /// (`growth/herbs.rs::WeatherCtx`).
    pub weather: WeatherCtx,
}

/// One `ParticleOrchestrator.RequestParticleSpawn(clientOnly: true, ...)`
/// (`ParticleOrchestraSettings.cs:16-29`). `kind` is the
/// `ParticleOrchestraType` byte the net module carries
/// (`app/netmodules.rs::Particles`): PooFly 31, RainbowBoulder2 48,
/// RainbowBoulder3 49, NatureFly 53 (`ParticleOrchestraType.cs:36/53/54/58`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ParticleRequest {
    pub kind: u8,
    pub position: Vec2,
    pub movement: Vec2,
}

/// The rubble arm's deferred spawn body (`:6937-6950`): `count`
/// `WorldGen.KillTile_MakeTileDust(i, j, tileCache)` calls, each followed
/// by `position.Y += 8`, `velocity *= 0.1` and (when `small`)
/// `scale -= 0.3`. The d54 seam owns the body + its `Main.rand`/`genRand`
/// draws (module docs).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RubbleDustPlan {
    pub count: i32,
    pub small: bool,
}

/// The body's outputs beyond the pool writes.
#[derive(Default, Debug)]
pub struct EmitOut {
    pub particles: Vec<ParticleRequest>,
    pub rubble_dust: Option<RubbleDustPlan>,
}

// ---------------------------------------------------------------------------
// Data tables (hand-transcribed; the tiledata generator's regeneration pass
// owns the eventual move — the wind.rs SWAYS_IN_WIND_BASIC precedent).
// ---------------------------------------------------------------------------

/// `Main.tileShine` (`Main.cs:7134-7135/:7188/:7506-7549/:9652/:9657/
/// :10001/:10008/:10015/:10072/:10177/:10291`): the ore/gem shine
/// denominators; 0 = no shine arm.
pub fn tile_shine(kind: u16) -> i32 {
    match kind {
        221 => 925,
        222 => 875,
        223 => 825,
        129 => 300,
        22 | 6 => 1150,
        7 => 1100,
        8 => 1000,
        9 => 1050,
        166 => 1125,
        167 => 1075,
        168 => 1025,
        169 => 975,
        617 => 400,
        178 | 211 => 500,
        12 | 639 => 300,
        21 | 467 | 441 | 468 => 1200,
        63 | 64 | 65 | 66 | 67 | 68 | 566 => 900,
        45 | 680 => 1900,
        46 | 681 => 2000,
        47 | 682 => 2100,
        122 | 686 => 1800,
        121 | 685 => 1850,
        125 => 600,
        109 | 110 | 116 | 117 => 9000,
        118 => 8000,
        107 => 950,
        108 => 900,
        111 => 850,
        239 => 1100,
        346 => 2000,
        347 | 370 => 1900,
        348 => 1800,
        407 => 1000,
        204 => 1150,
        _ => 0,
    }
}

/// `TorchID.Dust` (`TorchID.cs:179-184`): the per-torch-style smoke dust.
pub fn torch_dust(torch_id: i32) -> i32 {
    const DUST: [i32; 24] = [
        6, 59, 60, 61, 62, 63, 64, 65, 75, 135, 158, 169, 156, 234, 66, 242, 293, 294, 295,
        296, 297, 298, 307, 310,
    ];
    DUST[torch_id as usize]
}

/// `TileID.Sets.SpawnsNatureFlies` (`TileID.cs:538`).
pub fn spawns_nature_flies(kind: u16) -> bool {
    matches!(
        kind,
        3 | 20 | 24 | 27 | 73 | 201 | 80 | 110 | 529 | 530 | 595 | 615 | 637
    )
}

/// `TileID.Sets.MakesRubbleDust` (`TileID.cs:480`).
pub fn makes_rubble_dust(kind: u16) -> bool {
    matches!(
        kind,
        0 | 668
            | 40
            | 59
            | 123
            | 224
            | 1
            | 25
            | 203
            | 117
            | 2
            | 60
            | 70
            | 23
            | 661
            | 199
            | 662
            | 109
            | 477
            | 492
            | 633
            | 53
            | 112
            | 234
            | 116
            | 397
            | 398
            | 399
            | 402
            | 396
            | 400
            | 401
            | 403
            | 57
            | 147
            | 161
            | 163
            | 200
            | 164
            | 539
            | 182
            | 180
            | 179
            | 534
            | 381
            | 183
            | 181
            | 625
            | 536
            | 627
            | 191
            | 192
            | 383
            | 384
            | 315
            | 367
            | 368
            | 495
            | 56
            | 666
            | 229
            | 230
            | 189
            | 196
            | 460
            | 717
            | 718
            | 719
    )
}

// ---------------------------------------------------------------------------
// WorldGen probes (the private-helper precedent: stray.rs::solid_tile3,
// gore.rs::solid_tile — each lane keeps its own copy of the small probe).
// ---------------------------------------------------------------------------

/// `WorldGen.SolidTile3(int, int)` (`WorldGen.cs:70465-70483`): the
/// LOOSE solid test — `nactive && tileSolid && !tileSolidTop`, no
/// slope/half-brick term; `InWorld(i, j, 1)` refuses the 1-tile border.
fn solid_tile3(world: &World, x: i32, y: i32) -> bool {
    if x < 1 || x >= world.width - 1 || y < 1 || y >= world.height - 1 {
        return false;
    }
    let Some(t) = world.tile(x, y) else {
        return false;
    };
    t.active() && !t.inactive() && td::TILE_SOLID.has(t.kind) && !td::TILE_SOLID_TOP.has(t.kind)
}

/// `WorldGen.SolidTile(Tile)` (`WorldGen.cs:69957-69974`): the STRICT
/// test — null counts solid; active, not framed off, solid, not a
/// solid-top, not a half brick, not sloped. The 192 living-wood
/// exemption is the post-worldgen `tileSolid[192] = false`
/// (`WorldGen.cs:16607`), the `gore.rs::solid_tile` convention.
fn solid_tile(tile: Option<&Tile>) -> bool {
    let Some(t) = tile else { return true };
    t.active()
        && !t.inactive()
        && t.kind != 192
        && td::TILE_SOLID.has(t.kind)
        && !td::TILE_SOLID_TOP.has(t.kind)
        && !t.half_brick()
        && t.slope() == 0
}

/// `WorldGen.ActiveAndWalkableTile` (`WorldGen.cs:70300-70318`): the
/// null tile counts walkable (the C# `tile == null → true` arm); the
/// out-of-bounds catch returns false.
fn active_and_walkable(world: &World, x: i32, y: i32) -> bool {
    match world.tile(x, y) {
        None => false,
        Some(t) => t.active() && td::TILE_SOLID.has(t.kind) && !t.inactive(),
    }
}

/// `WorldGen.AnyLiquidAt(int, int)` (`WorldGen.cs:70254-70274`):
/// `InWorld(x, y, 5)` then `liquid > 0` (any type).
fn any_liquid_at(world: &World, x: i32, y: i32) -> bool {
    if x < 5 || x >= world.width - 5 || y < 5 || y >= world.height - 5 {
        return false;
    }
    world.tile(x, y).is_some_and(|t| t.liquid > 0)
}

/// `WorldGen.GetTopLeftAndStyles(ref x, ref y, w, h, fx, fy)`
/// (`WorldGen.cs:51777-51790`): snap `(x, y)` to the multi-tile's top
/// left; the returned Point (the style) is unused by the 617 arm.
fn snap_top_left(world: &World, x: &mut i32, y: &mut i32, w: i32, h: i32, fx: i32, fy: i32) {
    let t = world.tile(*x, *y).copied().unwrap_or_default();
    if fx != 0 {
        *x -= t.frame_x as i32 / fx % w;
    }
    if fy != 0 {
        *y -= t.frame_y as i32 / fy % h;
    }
}

/// `Utils.NextVector2Circular` (`Utils.cs:1266-1269`): one angle draw
/// then one radial draw, on the given stream (the `editing.rs`
/// precedent).
fn next_vector2_circular(rng: &mut UnifiedRandom, hw: f32, hh: f32) -> Vec2 {
    let angle = rng.next_float() * std::f32::consts::TAU;
    Vec2::from_angle(angle) * Vec2::new(hw, hh) * rng.next_float()
}

/// `Utils.NextVector2CircularEdge` (`Utils.cs:1271-1274`): one angle
/// draw, no radial.
fn next_vector2_circular_edge(rng: &mut UnifiedRandom, hw: f32, hh: f32) -> Vec2 {
    let angle = rng.next_float() * std::f32::consts::TAU;
    Vec2::from_angle(angle) * Vec2::new(hw, hh)
}

/// `TileDrawing.IsVisible` (`TileDrawing.cs:1411-1432`). The
/// `invisibleBlock()` bit is unported (module docs) — read as false.
pub fn is_visible(tile: &Tile, should_show_invisible_blocks: bool) -> bool {
    let mut flag = false; // tile.invisibleBlock() — unported bit
    match tile.kind {
        19 => {
            if tile.frame_y as i32 / 18 == 48 {
                flag = true;
            }
        }
        541 | 631 => flag = true,
        _ => {}
    }
    if flag {
        return should_show_invisible_blocks;
    }
    true
}

// ---------------------------------------------------------------------------
// The body — DrawTiles_EmitParticles (TileDrawing.cs:6688-7547).
// ---------------------------------------------------------------------------

/// One drawn tile's particle battery, in C# order. `i`/`j` are the tile
/// coordinates (the C# signature is `(j, i, ...)` — this port takes
/// `(world, i, j, ...)`); `fx`/`fy` are the `GetTileDrawData`-RESOLVED
/// `tileFrameX/Y` (`:967`), while the 139/497 arms read the RAW
/// `tile.frame_x/frame_y` (`:6964/:7018`). `light` is the tile light as
/// sRGB bytes (`Lighting.GetColor`, possibly bumped by the sight arms).
/// `rand` = `TileDrawing._rand`, `main_rand` = `Main.rand` (module docs).
#[allow(clippy::too_many_arguments)]
pub fn draw_tiles_emit_particles(
    world: &World,
    i: i32,
    j: i32,
    tile: &Tile,
    fx: i32,
    fy: i32,
    light: Color,
    ctx: &EmitCtx,
    rand: &mut impl EmitRand,
    main_rand: &mut UnifiedRandom,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    wind_grid: &WindGrid,
    out: &mut EmitOut,
) {
    let ty = tile.kind;
    let flag = is_visible(tile, ctx.should_show_invisible_blocks); // :6690
    let max_value = ctx.leaf_frequency / 4; // :6691
    let px = (i * 16) as f32;
    let py = (j * 16) as f32;

    // :6692-6751 — the 718 StarCloud night sparkles (three luck gates).
    if ty == 718 && !world.day_time && rand.next_max(3) == 0 && !solid_tile3(world, i, j - 1) {
        star_cloud_sparkles(i, j, ctx, rand, main_rand, dust, dust_ctx, world, gore);
    }
    // :6752-6774 — the 244 pumpkin (18,18) candle: five gore tiers.
    if ty == 244 && fx == 18 && fy == 18 && rand.next_max(2) == 0 {
        pumpkin_candle_gore(px, py, rand, main_rand, gore, world);
    }
    // :6775-6779 — the 565 FogMachine plume (Main.rand arguments).
    if ty == 565 && fx == 0 && fy == 18 && rand.next_max(3) == 0 {
        // ToWorldCoordinates() = (i*16+8, j*16+8) (Utils.cs:1762-1765).
        let vector = Vec2::new(px + 8.0, py + 8.0);
        let scale = 8.0 + main_rand.next_float() * 1.6;
        let velocity = (next_vector2_circular(main_rand, 0.7, 0.25) * 0.4
            + next_vector2_circular_edge(main_rand, 1.0, 0.4) * 0.1)
            * 4.0;
        gore.new_gore_perfect(
            main_rand,
            world,
            vector + Vec2::new(0.0, -18.0),
            velocity,
            1202,
            scale,
        );
    }
    // :6780-6795 — the 215 campfire: Next(3) drawn BEFORE the frameY==0
    // re-test (the C# && order).
    if ty == 215 && fy < 36 && rand.next_max(3) == 0 && fy == 0 {
        campfire_smoke(px, py - 4.0, fx, rand, main_rand, dust, dust_ctx, world);
    }
    // :6796-6847 — the 592/406/452 else-if chain.
    if ty == 592 && fy == 18 && rand.next_max(3) == 0 {
        campfire_smoke(px, py + 4.0, fx, rand, main_rand, dust, dust_ctx, world);
    } else if ty == 406 && fy == 54 && fx == 0 && rand.next_max(3) == 0 {
        chimney_smoke(px, py, ctx.wind, rand, main_rand, gore, world);
    } else if ty == 452 && fy == 0 && fx == 0 && rand.next_max(3) == 0 {
        balloon_machine(px, py, ctx, rand, main_rand, gore, world);
    }
    // :6848-6855 — the living-tree leaf blocks (192 wood / 384 leaf).
    if ty == 192 && rand.next_max(max_value) == 0 {
        emit_living_tree_leaf(world, i, j, 910, ctx.wind, rand, main_rand, gore);
    }
    if ty == 384 && rand.next_max(max_value) == 0 {
        emit_living_tree_leaf(world, i, j, 914, ctx.wind, rand, main_rand, gore);
    }
    // :6856-6862 — the 666/712 poo tiles: PooFly (orchestrator 31).
    if (ty == 666 || ty == 712)
        && tile.liquid == 0
        && j - 1 > 0
        && rand.next_max(100) == 0
        && !active_and_walkable(world, i, j - 1)
        && !any_liquid_at(world, i, j - 1)
    {
        out.particles.push(ParticleRequest {
            kind: 31, // ParticleOrchestraType.PooFly
            position: Vec2::new(px + 8.0, py - 8.0),
            movement: Vec2::ZERO,
        });
    }
    // :6863-6880 — the 711 RainbowBoulder (0,0): the two boulder arms.
    if ty == 711 && fx == 0 && fy == 0 {
        if rand.next_max(45) == 0 {
            out.particles.push(ParticleRequest {
                kind: 49, // RainbowBoulder3
                position: Vec2::new(px + 16.0, py + 16.0),
                movement: Vec2::ZERO,
            });
        }
        if rand.next_max(3) != 0 {
            out.particles.push(ParticleRequest {
                kind: 48, // RainbowBoulder2
                position: Vec2::new(px + 16.0, py + 16.0)
                    + next_vector2_circular(main_rand, 16.0, 16.0),
                movement: next_vector2_circular(main_rand, 1.0, 0.5) * 0.5,
            });
        }
    }
    // :6881-6900 — the nature-fly set: wind-scaled gate + gust window.
    if spawns_nature_flies(ty) && tile.liquid == 0 {
        let lerp = get_lerp_value(0.08, 0.18, ctx.wind.abs(), true);
        if rand.next_float() < lerp + 0.3 {
            let mut flag2 = rand.next_max(600) == 0;
            if !flag2 {
                let wt = wind_grid.get_wind_time(i, j, 8);
                flag2 = wt.time_left > 0 && rand.next_max(48) == 0;
            }
            if flag2 {
                out.particles.push(ParticleRequest {
                    kind: 53, // NatureFly
                    position: Vec2::new(px + 8.0, py),
                    movement: Vec2::ZERO,
                });
            }
        }
    }
    // :6901-6951 — the rubble-dust gate (the spawn body is the d54 seam).
    if rand.next_max(1200) == 0 {
        let mut flag3 = j + 1 < 0;
        let mut flag4 = false;
        let mut num8 = 3;
        if (j as f64) < world.world_surface {
            if rand.next_max(10) != 0 {
                flag3 = true;
            } else {
                num8 -= 1;
                flag4 = true;
            }
        }
        if !makes_rubble_dust(ty) {
            flag3 = true;
        }
        if !flag3 && active_and_walkable(world, i, j + 1) {
            flag3 = true;
        }
        if !flag3 && !wall_allows_wind(tile.wall) {
            if rand.next_max(2) == 0 {
                flag3 = true;
            } else {
                num8 -= 1;
            }
        }
        if !flag3 {
            out.rubble_dust = Some(RubbleDustPlan {
                count: num8,
                small: flag4,
            });
        }
    }
    // :6952-6955 — the IsVisible early return.
    if !flag {
        return;
    }
    // :6956-6961 — the 238 glowing-mushroom spore.
    if ty == 238 && rand.next_max(10) == 0 {
        let n = dust.new_dust(
            main_rand, dust_ctx, world, Vec2::new(px, py), 16, 16, 168, 0.0, 0.0, 0,
            Color::default(), 1.0,
        );
        let d = dust.slot_mut(n);
        d.no_gravity = true;
        d.alpha = 200;
    }
    // :6962-7045 — the 139/463/497/165/42 else-if chain.
    if ty == 139 {
        // The RAW tileCache frames (:6964), not the resolved fx/fy.
        if tile.frame_x == 36
            && tile.frame_y as i32 % 36 == 0
            && ctx.time_for_visual_effects as i32 % 7 == 0
            && rand.next_max(3) == 0
        {
            lantern_petals(px, py, ctx.wind, rand, main_rand, gore, world);
        }
    } else if ty == 463 {
        glowing_moss_dust(px, py, fx, fy, rand, main_rand, dust, dust_ctx, world);
    } else if ty == 497 {
        if tile.frame_y as i32 / 40 == 31 && tile.frame_y as i32 % 40 == 0 {
            for _ in 0..1 {
                if rand.next_max(10) == 0 {
                    let n = dust.new_dust_direct(
                        main_rand, dust_ctx, world, Vec2::new(px, py + 8.0), 16, 12, 43,
                        0.0, 0.0, 0, Color::default(), 1.0,
                    );
                    let d = dust.slot_mut(n);
                    d.no_gravity = true;
                    d.alpha = 254;
                    d.color = Color::WHITE;
                    d.scale = 0.7;
                    d.velocity = Vec2::ZERO;
                    d.no_light = true;
                }
            }
        }
    } else if ty == 165 && (162..=214).contains(&fx) && fy == 72 {
        if rand.next_max(60) == 0 {
            let n = dust.new_dust(
                main_rand, dust_ctx, world, Vec2::new(px + 2.0, py + 6.0), 8, 4, 153,
                0.0, 0.0, 0, Color::default(), 1.0,
            );
            let d = dust.slot_mut(n);
            d.scale -= rand.next_max(3) as f32 * 0.1;
            d.velocity.y = 0.0;
            d.velocity.x *= 0.05;
            d.alpha = 100;
        }
    } else if ty == 42 && fx == 0 {
        lantern_dust_gore(px, py, fy, rand, main_rand, dust, dust_ctx, gore, world);
    }
    // :7105-7127 — the 4 torch smoke (Next(40) BEFORE the frameX gate).
    if ty == 4 && rand.next_max(40) == 0 && fx < 66 {
        let torch_id = (tile.frame_y as i32 / 22).clamp(0, 23); // MathHelper.Clamp :7107
        let dust_type = torch_dust(torch_id);
        let pos = match fx {
            22 => Vec2::new(px + 6.0, py),
            44 => Vec2::new(px + 2.0, py),
            _ => Vec2::new(px + 4.0, py),
        };
        let n = dust.new_dust(
            main_rand, dust_ctx, world, pos, 4, 4, dust_type, 0.0, 0.0, 100,
            Color::default(), 1.0,
        );
        let d = dust.slot_mut(n);
        if rand.next_max(3) != 0 {
            d.no_gravity = true;
        }
        d.velocity *= 0.3;
        d.velocity.y -= 1.5;
        if dust_type == 66 {
            d.color = Color::new(ctx.disco[0], ctx.disco[1], ctx.disco[2], 255);
            d.no_gravity = true;
        }
    }
    // :7128-7170 — the 93 lava-bubble chandelier.
    if ty == 93 && rand.next_max(40) == 0 && fx == 0 {
        if fy / 18 % 3 == 0 {
            let dust_type = match fy / 54 {
                0 | 6 | 7 | 8 | 10 | 14 | 15 | 16 => 6,
                20 => 59,
                44 => 57,
                45 => 242,
                _ => -1,
            };
            if dust_type != -1 {
                rising_smoke_gate(px + 4.0, py + 2.0, 4, 4, dust_type, rand, main_rand, dust, dust_ctx, world);
            }
        }
    }
    // :7171-7215 — the 100 lava-bubble candelabra (the 2-of-3 X picks).
    if ty == 100 && rand.next_max(40) == 0 && fx < 36 {
        if fy / 18 % 2 == 0 {
            let dust_type = match fy / 36 {
                0 | 5 | 7 | 8 | 10 | 12 | 14 | 15 | 16 => 6,
                20 => 59,
                44 => 57,
                45 => 242,
                _ => -1,
            };
            if dust_type != -1 {
                let pos = if fx == 0 {
                    if rand.next_max(3) != 0 {
                        Vec2::new(px + 14.0, py + 2.0)
                    } else {
                        Vec2::new(px + 4.0, py + 2.0)
                    }
                } else if rand.next_max(3) != 0 {
                    Vec2::new(px, py + 2.0)
                } else {
                    Vec2::new(px + 6.0, py + 2.0)
                };
                rising_smoke_gate(pos.x, pos.y, 4, 4, dust_type, rand, main_rand, dust, dust_ctx, world);
            }
        }
    }
    // :7216-7225 — the 98 candle.
    if ty == 98 && rand.next_max(40) == 0 && fy == 0 && fx == 0 {
        rising_smoke_gate(px + 12.0, py + 2.0, 4, 4, 6, rand, main_rand, dust, dust_ctx, world);
    }
    // :7226-7240 — the 49 lava bubble (the small/large scale split).
    if ty == 49 && fx == 0 && rand.next_max(2) == 0 {
        lava_bubble(px, py, 172, rand, main_rand, dust, dust_ctx, world);
    }
    // :7241-7255 — the 372 shimmer bubble.
    if ty == 372 && fx == 0 && rand.next_max(2) == 0 {
        lava_bubble(px, py, 242, rand, main_rand, dust, dust_ctx, world);
    }
    // :7256-7259 — the 646 DISCARDED Next(2) (a draw with no consumer).
    if ty == 646 && fx == 0 {
        let _ = rand.next_max(2);
    }
    // :7260-7310 — the 34 lava-bubble chandelier (the style offset).
    if ty == 34 && rand.next_max(40) == 0 && fx % 108 < 54 {
        let mut style = fy / 54;
        if fx >= 108 {
            style += 37 * (fx / 108);
        }
        let col = fx / 18 % 3;
        if fy / 18 % 3 == 1 && col != 1 {
            let dust_type = match style {
                0..=5 | 12 | 13 | 16 | 19 | 21 => 6,
                25 => 59,
                50 => 57,
                51 => 242,
                _ => -1,
            };
            if dust_type != -1 {
                rising_smoke_gate(px, py + 2.0, 14, 6, dust_type, rand, main_rand, dust, dust_ctx, world);
            }
        }
    }
    // :7311-7318 — the 83 herb bloom sparkles.
    if ty == 83 {
        let style = fx / 18;
        if is_alchemy_plant_harvestable(world, style, j, &ctx.weather) {
            emit_alchemy_herb_particles(px, py, style, rand, main_rand, dust, dust_ctx, world);
        }
    }
    // :7319-7546 — the long else-if tail: ambient dust, ore shine, shimmer.
    ambient_dust_tail(world, i, j, tile, fx, fy, light, ctx, rand, main_rand, dust, dust_ctx);
}

// ---------------------------------------------------------------------------
// Per-arm helpers (each cites its C# block).
// ---------------------------------------------------------------------------

/// `:6694-6750` — the 718 StarCloud's three luck-gated sparkles. The
/// `RollLuck` gates roll `Main.rand` (`Luck.cs:5-16`); the position /
/// scale / velocity tweaks roll `_rand`; the pool internals roll
/// `Main.rand`. The second gore of each pair is a fresh `NewGore`
/// (its own kicks) then a field copy of the first.
#[allow(clippy::too_many_arguments)]
fn star_cloud_sparkles(
    i: i32,
    j: i32,
    ctx: &EmitCtx,
    rand: &mut impl EmitRand,
    main_rand: &mut UnifiedRandom,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    world: &World,
    gore: &mut GorePool,
) {
    let px = (i * 16) as f32;
    let py = (j * 16) as f32;
    if tr_core::loot::roll_luck(main_rand, ctx.luck, 100) == 0 {
        let n = gore.new_gore(
            main_rand,
            world,
            Vec2::new(px + rand.next_max(16) as f32, py - 12.0),
            Vec2::ZERO,
            16,
            1.0,
        );
        {
            let g = gore.slot_mut(n);
            g.scale *= rand.next_float() * 0.5 + 0.75;
            g.velocity *= 0.2;
            g.velocity.y -= rand.next_range(5, 31) as f32 * 0.1;
            if rand.next_max(5) == 0 {
                g.velocity.y -= rand.next_range(5, 41) as f32 * 0.1;
            }
            if rand.next_max(3) == 0 {
                g.velocity *= 0.5;
            }
            g.velocity /= g.scale;
        }
        let (scale, pos, vel) = {
            let g = gore.slot(n);
            (g.scale, g.position, g.velocity)
        };
        let n2 = gore.new_gore(main_rand, world, Vec2::new(px, py), Vec2::ZERO, 16, 1.0);
        let g2 = gore.slot_mut(n2);
        g2.scale = scale;
        g2.position = pos;
        g2.velocity = vel;
    }
    if tr_core::loot::roll_luck(main_rand, ctx.luck, 60) == 0 {
        let n = gore.new_gore(
            main_rand,
            world,
            Vec2::new(px + rand.next_max(16) as f32, py - 12.0),
            Vec2::ZERO,
            17,
            1.0,
        );
        {
            let g = gore.slot_mut(n);
            g.scale *= rand.next_float() * 0.5 + 0.75;
            g.velocity *= 0.2;
            g.velocity.y -= rand.next_range(5, 41) as f32 * 0.1;
            if rand.next_max(5) == 0 {
                g.velocity.y -= rand.next_range(5, 51) as f32 * 0.1;
            }
            if rand.next_max(3) == 0 {
                g.velocity *= 0.5;
            }
            g.velocity /= g.scale;
        }
        let (scale, pos, vel) = {
            let g = gore.slot(n);
            (g.scale, g.position, g.velocity)
        };
        let n2 = gore.new_gore(main_rand, world, Vec2::new(px, py), Vec2::ZERO, 17, 1.0);
        let g2 = gore.slot_mut(n2);
        g2.scale = scale;
        g2.position = pos;
        g2.velocity = vel;
    }
    if tr_core::loot::roll_luck(main_rand, ctx.luck, 30) == 0 {
        let n = dust.new_dust(
            main_rand, dust_ctx, world, Vec2::new(px, py - 2.0), 1, 1, 58, 0.0, 0.0, 150,
            Color::default(), 1.0,
        );
        let d = dust.slot_mut(n);
        d.scale *= rand.next_float() * 0.5 + 0.75;
        d.color = Color::new(255, 255, 255, 0);
        d.velocity *= 0.2;
        d.velocity.y -= rand.next_range(5, 51) as f32 * 0.1;
        if rand.next_max(5) == 0 {
            d.velocity.y -= rand.next_range(5, 61) as f32 * 0.1;
        }
        if rand.next_max(3) == 0 {
            d.velocity *= 0.5;
        }
        d.velocity /= d.scale;
    }
}

/// `:6754-6773` — the 244 pumpkin candle's five-tier gore chain
/// (415/414/413/412/411, the scale arguments on `_rand`).
fn pumpkin_candle_gore(
    px: f32,
    py: f32,
    rand: &mut impl EmitRand,
    main_rand: &mut UnifiedRandom,
    gore: &mut GorePool,
    world: &World,
) {
    let pos = Vec2::new(px + 8.0, py + 8.0);
    let tier = if rand.next_max(500) == 0 {
        Some(415)
    } else if rand.next_max(250) == 0 {
        Some(414)
    } else if rand.next_max(80) == 0 {
        Some(413)
    } else if rand.next_max(10) == 0 {
        Some(412)
    } else if rand.next_max(3) == 0 {
        Some(411)
    } else {
        None
    };
    if let Some(t) = tier {
        let scale = rand.next_range(51, 101) as f32 * 0.01;
        gore.new_gore(main_rand, world, pos, Vec2::ZERO, t, scale);
    }
}

/// `:6782-6794` / `:6798-6810` — the shared 215/592 campfire smoke
/// (the 592 twin is byte-identical bar the base Y the caller passes).
#[allow(clippy::too_many_arguments)]
fn campfire_smoke(
    px: f32,
    base_y: f32,
    fx: i32,
    rand: &mut impl EmitRand,
    main_rand: &mut UnifiedRandom,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    world: &World,
) {
    let n = dust.new_dust(
        main_rand, dust_ctx, world, Vec2::new(px + 2.0, base_y), 4, 8, 31, 0.0, 0.0, 100,
        Color::default(), 1.0,
    );
    let d = dust.slot_mut(n);
    if fx == 0 {
        d.position.x += rand.next_max(8) as f32;
    }
    if fx == 36 {
        d.position.x -= rand.next_max(8) as f32;
    }
    d.alpha += rand.next_max(100);
    d.velocity *= 0.2;
    d.velocity.y -= 0.5 + rand.next_max(10) as f32 * 0.1;
    d.fade_in = 0.5 + rand.next_max(10) as f32 * 0.1;
}

/// `:6814-6832` — the 406 chimney smoke: the wind pushes left only,
/// the gore type is `Next(825, 828)` and the scale tier is 1/4, 1/2,
/// else (each tier's `NextFloat` on `_rand`).
fn chimney_smoke(
    px: f32,
    py: f32,
    wind: f32,
    rand: &mut impl EmitRand,
    main_rand: &mut UnifiedRandom,
    gore: &mut GorePool,
    world: &World,
) {
    let position = Vec2::new(px + 16.0, py + 8.0);
    let mut velocity = Vec2::ZERO;
    if wind < 0.0 {
        velocity.x = 0.0 - wind;
    }
    let t = rand.next_range(825, 828);
    if rand.next_max(4) == 0 {
        let scale = rand.next_float() * 0.2 + 0.2;
        gore.new_gore(main_rand, world, position, velocity, t, scale);
    } else if rand.next_max(2) == 0 {
        let scale = rand.next_float() * 0.3 + 0.3;
        gore.new_gore(main_rand, world, position, velocity, t, scale);
    } else {
        let scale = rand.next_float() * 0.4 + 0.4;
        gore.new_gore(main_rand, world, position, velocity, t, scale);
    }
}

/// `:6836-6846` — the 452 balloon machine: the gore type is
/// `907 + Main.tileFrame[452] / 5` (the animator row, 0 until ported),
/// a 1/2 gate, scale `NextFloat() * 0.4 + 0.4`.
fn balloon_machine(
    px: f32,
    py: f32,
    ctx: &EmitCtx,
    rand: &mut impl EmitRand,
    main_rand: &mut UnifiedRandom,
    gore: &mut GorePool,
    world: &World,
) {
    let position = Vec2::new(px + 16.0, py + 8.0);
    let mut velocity = Vec2::ZERO;
    if ctx.wind < 0.0 {
        velocity.x = 0.0 - ctx.wind;
    }
    let t = 907 + ctx.tile_frame_anim / 5;
    if rand.next_max(2) == 0 {
        let scale = rand.next_float() * 0.4 + 0.4;
        gore.new_gore(main_rand, world, position, velocity, t, scale);
    }
}

/// `EmitLivingTreeLeaf` (`TileDrawing.cs:7549-7556`): the below-spawn
/// first, then the 1/2 sideways roll (UNCONDITIONAL — the inline
/// `render/tiles.rs` arm gates it on the wind pick, a deviation the
/// wiring must reconcile).
#[allow(clippy::too_many_arguments)]
pub fn emit_living_tree_leaf(
    world: &World,
    x: i32,
    y: i32,
    leaf_gore_type: i32,
    wind: f32,
    rand: &mut impl EmitRand,
    main_rand: &mut UnifiedRandom,
    gore: &mut GorePool,
) {
    emit_living_tree_leaf_below(world, x, y, leaf_gore_type, wind, main_rand, gore);
    if rand.next_max(2) == 0 {
        emit_living_tree_leaf_sideways(world, x, y, leaf_gore_type, wind, main_rand, gore);
    }
}

/// `EmitLivingTreeLeaf_Below` (`:7558-7569`): the tile below must be
/// open (not strict-solid, no liquid); a strong wind needs TWO open
/// cells downwind. The gore's paint column is the leaf tile's colour.
fn emit_living_tree_leaf_below(
    world: &World,
    x: i32,
    y: i32,
    leaf_gore_type: i32,
    wind: f32,
    main_rand: &mut UnifiedRandom,
    gore: &mut GorePool,
) {
    let below = world.tile(x, y + 1);
    if !solid_tile(below) && below.is_none_or(|t| t.liquid == 0) {
        let left_open = !solid_tile(world.tile(x - 1, y + 1))
            && !solid_tile(world.tile(x - 2, y + 1));
        let right_open = !solid_tile(world.tile(x + 1, y + 1))
            && !solid_tile(world.tile(x + 2, y + 1));
        if (!(wind < -0.2) || left_open) && (!(wind > 0.2) || right_open) {
            let n = gore.new_gore_perfect(
                main_rand,
                world,
                Vec2::new((x * 16) as f32, (y * 16 + 16) as f32),
                Vec2::ZERO,
                leaf_gore_type,
                1.0,
            );
            gore.slot_mut(n).frame.current_column =
                world.tile(x, y).map(|t| t.paint()).unwrap_or(0);
        }
    }
}

/// `EmitLivingTreeLeaf_Sideways` (`:7571-7592`): the downwind side
/// (|wind| > 0.2), the −10 px left-shift quirk, paint column again.
fn emit_living_tree_leaf_sideways(
    world: &World,
    x: i32,
    y: i32,
    leaf_gore_type: i32,
    wind: f32,
    main_rand: &mut UnifiedRandom,
    gore: &mut GorePool,
) {
    let num = if wind > 0.2 {
        1
    } else if wind < -0.2 {
        -1
    } else {
        0
    };
    let side = world.tile(x + num, y);
    if !solid_tile(side) && side.is_none_or(|t| t.liquid == 0) {
        let num2 = if num == -1 { -10 } else { 0 };
        let n = gore.new_gore_perfect(
            main_rand,
            world,
            Vec2::new(
                (x * 16 + 8 + 4 * num + num2) as f32,
                (y * 16 + 8) as f32,
            ),
            Vec2::ZERO,
            leaf_gore_type,
            1.0,
        );
        gore.slot_mut(n).frame.current_column =
            world.tile(x, y).map(|t| t.paint()).unwrap_or(0);
    }
}

/// `:6966-6979` — the 139 hanging-lantern petals (570/571/572): the
/// wind-blown velocity with the ±50 % jitters, the per-type X shift.
fn lantern_petals(
    px: f32,
    py: f32,
    wind: f32,
    rand: &mut impl EmitRand,
    main_rand: &mut UnifiedRandom,
    gore: &mut GorePool,
    world: &World,
) {
    let t = rand.next_range(570, 573);
    let mut position = Vec2::new(px + 8.0, py - 8.0);
    let mut velocity = Vec2::new(wind * 2.0, -0.5);
    velocity.x *= 1.0 + rand.next_range(-50, 51) as f32 * 0.01;
    velocity.y *= 1.0 + rand.next_range(-50, 51) as f32 * 0.01;
    if t == 572 {
        position.x -= 8.0;
    }
    if t == 571 {
        position.x -= 4.0;
    }
    gore.new_gore(main_rand, world, position, velocity, t, 0.8);
}

/// `:6984-7014` — the 463 glowing moss: the (0,54) four-roll smoke
/// burst and the (0|36,18) 1/13 sparkle (the 274 dust rides the
/// `NextFloatDirection` Y).
#[allow(clippy::too_many_arguments)]
fn glowing_moss_dust(
    px: f32,
    py: f32,
    fx: i32,
    fy: i32,
    rand: &mut impl EmitRand,
    main_rand: &mut UnifiedRandom,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    world: &World,
) {
    if fy == 54 && fx == 0 {
        for _ in 0..4 {
            if rand.next_max(2) != 0 {
                let n = dust.new_dust_direct(
                    main_rand, dust_ctx, world, Vec2::new(px + 4.0, py), 36, 8, 16,
                    0.0, 0.0, 0, Color::default(), 1.0,
                );
                let d = dust.slot_mut(n);
                d.no_gravity = true;
                d.alpha = 140;
                d.fade_in = 1.2;
                d.velocity = Vec2::ZERO;
            }
        }
    }
    if fy == 18 && (fx == 0 || fx == 36) {
        for _ in 0..1 {
            if rand.next_max(13) == 0 {
                let n = dust.new_dust_direct(
                    main_rand, dust_ctx, world, Vec2::new(px, py), 8, 8, 274,
                    0.0, 0.0, 0, Color::default(), 1.0,
                );
                let d = dust.slot_mut(n);
                d.position = Vec2::new(px + 8.0, py + 8.0);
                d.position.x += if fx == 36 { 4.0 } else { -4.0 };
                d.no_gravity = true;
                d.alpha = 128;
                d.fade_in = 1.2;
                d.no_light = true;
                d.velocity = Vec2::new(0.0, rand.next_float_direction() * 1.2);
            }
        }
    }
}

/// `:7048-7102` — the 42 hanging lantern's odd-row arms: case 7
/// (the 58 smoke + the 16/17 gore pair), 29 (59), 50 (57), 51 (242 —
/// the lava-bubble tint).
#[allow(clippy::too_many_arguments)]
fn lantern_dust_gore(
    px: f32,
    py: f32,
    fy: i32,
    rand: &mut impl EmitRand,
    main_rand: &mut UnifiedRandom,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    gore: &mut GorePool,
    world: &World,
) {
    let row = fy / 36;
    if fy / 18 % 2 != 1 {
        return;
    }
    match row {
        7 => {
            if rand.next_max(50) == 0 {
                let n = dust.new_dust(
                    main_rand, dust_ctx, world, Vec2::new(px + 4.0, py + 4.0), 8, 8, 58,
                    0.0, 0.0, 150, Color::default(), 1.0,
                );
                dust.slot_mut(n).velocity *= 0.5;
            }
            if rand.next_max(100) == 0 {
                let t = rand.next_range(16, 18);
                let n = gore.new_gore(
                    main_rand, world, Vec2::new(px - 2.0, py - 4.0), Vec2::ZERO, t, 1.0,
                );
                let g = gore.slot_mut(n);
                g.scale *= 0.7;
                g.velocity *= 0.25;
            }
        }
        29 => {
            if rand.next_max(40) == 0 {
                rising_smoke_gate(px + 4.0, py, 8, 8, 59, rand, main_rand, dust, dust_ctx, world);
            }
        }
        50 => {
            if rand.next_max(10) == 0 {
                rising_smoke_gate(px + 4.0, py, 8, 8, 57, rand, main_rand, dust, dust_ctx, world);
            }
        }
        51 => {
            if rand.next_max(40) == 0 {
                rising_smoke_gate(px + 4.0, py + 2.0, 4, 4, 242, rand, main_rand, dust, dust_ctx, world);
            }
        }
        _ => {}
    }
}

/// The shared rising-smoke tail (`:7069-7075` passim): the 2-of-3
/// `noGravity` roll on `_rand`, `velocity *= 0.3`, `Y -= 1.5`.
#[allow(clippy::too_many_arguments)]
fn rising_smoke_gate(
    x: f32,
    y: f32,
    w: i32,
    h: i32,
    dust_type: i32,
    rand: &mut impl EmitRand,
    main_rand: &mut UnifiedRandom,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    world: &World,
) {
    let n = dust.new_dust(
        main_rand, dust_ctx, world, Vec2::new(x, y), w, h, dust_type, 0.0, 0.0, 100,
        Color::default(), 1.0,
    );
    let d = dust.slot_mut(n);
    if rand.next_max(3) != 0 {
        d.no_gravity = true;
    }
    d.velocity *= 0.3;
    d.velocity.y -= 1.5;
}

/// `:7228-7239` / `:7243-7254` — the 49/372 lava bubbles: the 1/3
/// small (scale 0.5) vs 2/3 large (0.9 + noGravity) split.
#[allow(clippy::too_many_arguments)]
fn lava_bubble(
    px: f32,
    py: f32,
    dust_type: i32,
    rand: &mut impl EmitRand,
    main_rand: &mut UnifiedRandom,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    world: &World,
) {
    let n = dust.new_dust(
        main_rand, dust_ctx, world, Vec2::new(px + 4.0, py - 4.0), 4, 4, dust_type,
        0.0, 0.0, 100, Color::default(), 1.0,
    );
    let d = dust.slot_mut(n);
    if rand.next_max(3) == 0 {
        d.scale = 0.5;
    } else {
        d.scale = 0.9;
        d.no_gravity = true;
    }
    d.velocity *= 0.3;
    d.velocity.y -= 1.5;
}

/// `EmitAlchemyHerbParticles` (`TileDrawing.cs:9605-9655`): the
/// per-style bloom sparkles — 0 daybloom (19), 1 moonglow (41),
/// 3 deathweed (14 + 27, TWO independent gates), 4 waterleaf (16),
/// 5 fireblossom (6), 6 shiverthorn (43, cyan).
#[allow(clippy::too_many_arguments)]
pub fn emit_alchemy_herb_particles(
    px: f32,
    py: f32,
    style: i32,
    rand: &mut impl EmitRand,
    main_rand: &mut UnifiedRandom,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
    world: &World,
) {
    if style == 0 && rand.next_max(100) == 0 {
        let n = dust.new_dust(
            main_rand, dust_ctx, world, Vec2::new(px, py - 4.0), 16, 16, 19, 0.0, 0.0, 160,
            Color::default(), 0.1,
        );
        let d = dust.slot_mut(n);
        d.velocity.x /= 2.0;
        d.velocity.y /= 2.0;
        d.no_gravity = true;
        d.fade_in = 1.0;
    }
    if style == 1 && rand.next_max(100) == 0 {
        dust.new_dust(
            main_rand, dust_ctx, world, Vec2::new(px, py), 16, 16, 41, 0.0, 0.0, 250,
            Color::default(), 0.8,
        );
    }
    if style == 3 {
        if rand.next_max(200) == 0 {
            let n = dust.new_dust(
                main_rand, dust_ctx, world, Vec2::new(px, py), 16, 16, 14, 0.0, 0.0, 100,
                Color::default(), 0.2,
            );
            dust.slot_mut(n).fade_in = 1.2;
        }
        if rand.next_max(75) == 0 {
            let n = dust.new_dust(
                main_rand, dust_ctx, world, Vec2::new(px, py), 16, 16, 27, 0.0, 0.0, 100,
                Color::default(), 1.0,
            );
            let d = dust.slot_mut(n);
            d.velocity.x /= 2.0;
            d.velocity.y /= 2.0;
        }
    }
    if style == 4 && rand.next_max(150) == 0 {
        let n = dust.new_dust(
            main_rand, dust_ctx, world, Vec2::new(px, py), 16, 8, 16, 0.0, 0.0, 0,
            Color::default(), 1.0,
        );
        let d = dust.slot_mut(n);
        d.velocity.x /= 3.0;
        d.velocity.y /= 3.0;
        d.velocity.y -= 0.7;
        d.alpha = 50;
        d.scale *= 0.1;
        d.fade_in = 0.9;
        d.no_gravity = true;
    }
    if style == 5 && rand.next_max(40) == 0 {
        let n = dust.new_dust(
            main_rand, dust_ctx, world, Vec2::new(px, py - 6.0), 16, 16, 6, 0.0, 0.0, 0,
            Color::default(), 1.5,
        );
        let d = dust.slot_mut(n);
        d.velocity.y -= 2.0;
        d.no_gravity = true;
    }
    if style == 6 && rand.next_max(30) == 0 {
        let n = dust.new_dust(
            main_rand, dust_ctx, world, Vec2::new(px, py), 16, 16, 43, 0.0, 0.0, 254,
            Color::new(50, 255, 255, 255), 0.5,
        );
        dust.slot_mut(n).velocity *= 0.0;
    }
}

/// `:7319-7546` — the long else-if tail. A FAILED gate falls through
/// to the next arm (the C# chain), so a type-22 tile whose `Next(400)`
/// misses still rolls the shine `Next(1150)`. The final arm is the
/// shimmer sparkle on lit solids (`Main.tileSolid[type]`).
#[allow(clippy::too_many_arguments)]
fn ambient_dust_tail(
    world: &World,
    i: i32,
    j: i32,
    tile: &Tile,
    fx: i32,
    fy: i32,
    light: Color,
    ctx: &EmitCtx,
    rand: &mut impl EmitRand,
    main_rand: &mut UnifiedRandom,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
) {
    let ty = tile.kind;
    let pos = Vec2::new((i * 16) as f32, (j * 16) as f32);
    if ty == 22 && rand.next_max(400) == 0 {
        dust.new_dust(main_rand, dust_ctx, world, pos, 16, 16, 14, 0.0, 0.0, 0, Color::default(), 1.0);
    } else if (ty == 23 || ty == 24 || ty == 32) && rand.next_max(500) == 0 {
        dust.new_dust(main_rand, dust_ctx, world, pos, 16, 16, 14, 0.0, 0.0, 0, Color::default(), 1.0);
    } else if ty == 25 && rand.next_max(700) == 0 {
        dust.new_dust(main_rand, dust_ctx, world, pos, 16, 16, 14, 0.0, 0.0, 0, Color::default(), 1.0);
    } else if ty == 112 && rand.next_max(700) == 0 {
        dust.new_dust(main_rand, dust_ctx, world, pos, 16, 16, 14, 0.0, 0.0, 0, Color::default(), 1.0);
    } else if (ty == 31 || ty == 696) && rand.next_max(20) == 0 {
        if fx >= 36 {
            let n = dust.new_dust(main_rand, dust_ctx, world, pos, 16, 16, 5, 0.0, 0.0, 100, Color::default(), 1.0);
            let d = dust.slot_mut(n);
            d.velocity.y = 0.0;
            d.velocity.x *= 0.3;
        } else {
            dust.new_dust(main_rand, dust_ctx, world, pos, 16, 16, 14, 0.0, 0.0, 100, Color::default(), 1.0);
        }
    } else if (ty == 26 || ty == 695) && rand.next_max(20) == 0 {
        if fx >= 54 {
            let n = dust.new_dust(main_rand, dust_ctx, world, pos, 16, 16, 5, 0.0, 0.0, 100, Color::default(), 1.0);
            let d = dust.slot_mut(n);
            d.scale = 1.5;
            d.no_gravity = true;
            d.velocity *= 0.75;
        } else {
            dust.new_dust(main_rand, dust_ctx, world, pos, 16, 16, 14, 0.0, 0.0, 100, Color::default(), 1.0);
        }
    } else if (ty == 71 || ty == 72) && tile.paint() == 0 && rand.next_max(500) == 0 {
        dust.new_dust(main_rand, dust_ctx, world, pos, 16, 16, 41, 0.0, 0.0, 250, Color::default(), 0.8);
    } else if (ty == 17 || ty == 77 || ty == 133) && rand.next_max(40) == 0 {
        // The C# `(fx == 18) & (fy == 18)` is the bitwise bool & — both
        // operands pure, `&&` is equivalent.
        if fx == 18 && fy == 18 {
            let n = dust.new_dust(
                main_rand, dust_ctx, world, pos + Vec2::new(-4.0, -6.0), 8, 6, 6,
                0.0, 0.0, 100, Color::default(), 1.0,
            );
            if rand.next_max(3) != 0 {
                dust.slot_mut(n).no_gravity = true;
            }
        }
    } else if ty == 405 && rand.next_max(20) == 0 {
        if fx == 18 && fy == 18 {
            let n = dust.new_dust(
                main_rand, dust_ctx, world, pos + Vec2::new(-4.0, -6.0), 24, 10, 6,
                0.0, 0.0, 100, Color::default(), 1.0,
            );
            if rand.next_max(5) != 0 {
                dust.slot_mut(n).no_gravity = true;
            }
        }
    } else if ty == 37 && rand.next_max(250) == 0 {
        // The Scale argument is Next(3) — an INTEGER 0..2 (:7390).
        let scale = rand.next_max(3) as f32;
        let n = dust.new_dust(main_rand, dust_ctx, world, pos, 16, 16, 6, 0.0, 0.0, 0, Color::default(), scale);
        if dust.slot(n).scale > 1.0 {
            dust.slot_mut(n).no_gravity = true;
        }
    } else if (ty == 58 || ty == 76 || ty == 684) && rand.next_max(250) == 0 {
        let scale = rand.next_max(3) as f32;
        let n = dust.new_dust(main_rand, dust_ctx, world, pos, 16, 16, 6, 0.0, 0.0, 0, Color::default(), scale);
        let d = dust.slot_mut(n);
        if d.scale > 1.0 {
            d.no_gravity = true;
        }
        d.no_light = true;
    } else if ty == 61 || ty == 703 {
        if fx == 144 && rand.next_max(60) == 0 {
            let n = dust.new_dust(main_rand, dust_ctx, world, pos, 16, 16, 44, 0.0, 0.0, 250, Color::default(), 0.4);
            dust.slot_mut(n).fade_in = 0.7;
        }
    } else if tile_shine(ty) > 0 {
        shine_arm(world, i, j, tile, fx, light, ctx, rand, main_rand, dust, dust_ctx);
    } else if td::TILE_SOLID.has(ty)
        && ctx.shimmer_alpha > 0.0
        && (light.r > 20 || light.b > 20 || light.g > 20)
    {
        let num47 = light.r.max(light.g).max(light.b) as i32;
        if (rand.next_max(500) as f32) < 2.0 * ctx.shimmer_alpha {
            let scale = (num47 as f32 / 255.0 + 1.0) / 2.0;
            let n = dust.new_dust(main_rand, dust_ctx, world, pos, 16, 16, 43, 0.0, 0.0, 254, Color::WHITE, scale);
            dust.slot_mut(n).velocity *= 0.0;
        }
    }
}

/// `:7413-7529` — the ore/gem shine arm: the light gate, the
/// `Next(tileShine)` roll BEFORE the frame-window terms, the 617
/// trophy and 178 gemspark specials, the per-type colour map.
#[allow(clippy::too_many_arguments)]
fn shine_arm(
    world: &World,
    i: i32,
    j: i32,
    tile: &Tile,
    fx: i32,
    light: Color,
    ctx: &EmitCtx,
    rand: &mut impl EmitRand,
    main_rand: &mut UnifiedRandom,
    dust: &mut DustPool,
    dust_ctx: &SpawnCtx,
) {
    let ty = tile.kind;
    let pos = Vec2::new((i * 16) as f32, (j * 16) as f32);
    // :7415-7418 — the dark-light early return.
    if light.r <= 20 && light.b <= 20 && light.g <= 20 {
        return;
    }
    let num42 = light.r.max(light.g).max(light.b) as i32 / 30;
    // :7429 — the roll FIRST, then the frame windows.
    if rand.next_max(tile_shine(ty)) >= num42
        || ((ty == 21 || ty == 441)
            && (fx < 36 || fx >= 180)
            && (fx < 396 || fx > 409))
        || ((ty == 467 || ty == 468) && (fx < 144 || fx >= 180))
    {
        return;
    }
    let mut new_color = Color::WHITE;
    match ty {
        617 => {
            // :7436-7452 — the master trophy: the 3x4 top-left snap,
            // the hidden-middle row pick, the j >= row gate.
            let (mut x, mut y) = (i, j);
            snap_top_left(world, &mut x, &mut y, 3, 4, 18, 18);
            let mut num44 = y;
            let mid = world.tile(x + 1, y + 1).copied().unwrap_or_default();
            if !is_visible(&mid, ctx.should_show_invisible_blocks) {
                num44 = y + 3;
            }
            if j >= num44 {
                let n = dust.new_dust(main_rand, dust_ctx, world, pos, 16, 16, 43, 0.0, 0.0, 254, new_color, 0.5);
                dust.slot_mut(n).velocity *= 0.0;
            }
            return;
        }
        178 => {
            // :7453-7482 — the gemspark colour by style column.
            new_color = match fx / 18 {
                0 => Color::new(255, 0, 255, 255),
                1 | 6 => Color::new(255, 255, 0, 255),
                2 => Color::new(0, 0, 255, 255),
                3 => Color::new(0, 255, 0, 255),
                4 => Color::new(255, 0, 0, 255),
                5 => Color::new(255, 255, 255, 255),
                _ => new_color,
            };
            let n = dust.new_dust(main_rand, dust_ctx, world, pos, 16, 16, 43, 0.0, 0.0, 254, new_color, 0.5);
            dust.slot_mut(n).velocity *= 0.0;
            return;
        }
        63 => new_color = Color::new(0, 0, 255, 255),
        _ => {}
    }
    // :7487-7526 — the per-type colour overrides.
    match ty {
        64 | 12 | 665 | 204 => new_color = Color::new(255, 0, 0, 255),
        65 => new_color = Color::new(0, 255, 0, 255),
        66 | 566 => new_color = Color::new(255, 255, 0, 255),
        67 => new_color = Color::new(255, 0, 255, 255),
        68 => new_color = Color::new(255, 255, 255, 255),
        639 => new_color = Color::new(0, 0, 255, 255),
        211 => new_color = Color::new(50, 255, 100, 255),
        _ => {}
    }
    let n = dust.new_dust(main_rand, dust_ctx, world, pos, 16, 16, 43, 0.0, 0.0, 254, new_color, 0.5);
    dust.slot_mut(n).velocity *= 0.0;
}

// ---------------------------------------------------------------------------
// Witnesses (W4-d60 code-only round — WRITTEN NOT RUN).
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use tr_core::tile::FLAG_ACTIVE;

    /// A scripted `TileDrawing._rand` stand-in: every draw is recorded
    /// (the `projectile_ai_misc.rs` CensusRng shape) and answered from
    /// a queue — an empty queue answers `next_max`/`next_range` with 0
    /// (the gate-passing value) and floats with 0.0.
    struct ScriptRand {
        log: Vec<String>,
        answers: std::collections::VecDeque<i32>,
        float_answers: std::collections::VecDeque<f32>,
    }

    impl ScriptRand {
        fn new() -> Self {
            Self {
                log: Vec::new(),
                answers: std::collections::VecDeque::new(),
                float_answers: std::collections::VecDeque::new(),
            }
        }
        fn with(mut self, answers: &[i32]) -> Self {
            self.answers.extend(answers.iter().copied());
            self
        }
        fn with_floats(mut self, answers: &[f32]) -> Self {
            self.float_answers.extend(answers.iter().copied());
            self
        }
    }

    impl EmitRand for ScriptRand {
        fn next_max(&mut self, max: i32) -> i32 {
            self.log.push(format!("N{max}"));
            self.answers.pop_front().unwrap_or(0)
        }
        fn next_range(&mut self, min: i32, max: i32) -> i32 {
            self.log.push(format!("R{min},{max}"));
            self.answers.pop_front().unwrap_or(min)
        }
        fn next_float(&mut self) -> f32 {
            self.log.push("F".into());
            self.float_answers.pop_front().unwrap_or(0.0)
        }
        fn next_float_direction(&mut self) -> f32 {
            self.log.push("FD".into());
            self.float_answers.pop_front().unwrap_or(0.0)
        }
    }

    fn world() -> World {
        World::new("t".into(), 300, 200)
    }

    fn put(w: &mut World, x: i32, y: i32, kind: u16, fx: i16, fy: i16) {
        let i = w.index(x, y);
        w.tiles[i] = Tile {
            kind,
            frame_x: fx,
            frame_y: fy,
            ..Default::default()
        };
        w.tiles[i].flags |= FLAG_ACTIVE;
    }

    fn ctx() -> EmitCtx {
        EmitCtx {
            wind: 0.0,
            time_for_visual_effects: 0.0,
            leaf_frequency: 14000,
            luck: 0.0,
            shimmer_alpha: 0.0,
            tile_frame_anim: 0,
            disco: [255, 0, 0],
            should_show_invisible_blocks: false,
            weather: WeatherCtx::default(),
        }
    }

    /// A spawn context whose cull never refuses (the tile sits inside
    /// the screen box) and whose budget keeps every pressure band off.
    fn dust_ctx() -> SpawnCtx {
        SpawnCtx {
            game_menu: false,
            game_paused: false,
            generating: false,
            screen_position: Vec2::new(-1000.0, -1000.0),
            screen_size: Vec2::new(10000.0, 10000.0),
            max_dust_to_draw: 6000,
        }
    }

    fn tile_at(w: &World, x: i32, y: i32) -> Tile {
        w.tile(x, y).copied().unwrap_or_default()
    }

    /// Run the body over one tile with the given scripts.
    fn run(
        w: &World,
        x: i32,
        y: i32,
        ctx: &EmitCtx,
        rand: &mut ScriptRand,
        main_seed: i32,
    ) -> (EmitOut, DustPool, GorePool, UnifiedRandom) {
        let t = tile_at(w, x, y);
        let mut dust = DustPool::new();
        let mut gore = GorePool::new();
        let mut main_rand = UnifiedRandom::new(main_seed);
        let mut out = EmitOut::default();
        draw_tiles_emit_particles(
            w,
            x,
            y,
            &t,
            t.frame_x as i32,
            t.frame_y as i32,
            Color::new(255, 255, 255, 255),
            ctx,
            rand,
            &mut main_rand,
            &mut dust,
            &dust_ctx(),
            &mut gore,
            &WindGrid::new(),
            &mut out,
        );
        (out, dust, gore, main_rand)
    }

    // --- data tables -------------------------------------------------------

    /// `Main.cs:7506-7549` passim — the shine denominators.
    #[test]
    fn the_tile_shine_table_matches_the_source_rows() {
        assert_eq!(tile_shine(6), 1150);
        assert_eq!(tile_shine(22), 1150);
        assert_eq!(tile_shine(7), 1100);
        assert_eq!(tile_shine(21), 1200);
        assert_eq!(tile_shine(109), 9000);
        assert_eq!(tile_shine(118), 8000);
        assert_eq!(tile_shine(617), 400);
        assert_eq!(tile_shine(178), 500);
        assert_eq!(tile_shine(211), 500);
        assert_eq!(tile_shine(204), 1150);
        assert_eq!(tile_shine(639), 300);
        assert_eq!(tile_shine(0), 0);
        assert_eq!(tile_shine(1), 0);
    }

    /// `TorchID.cs:179-184` — the 24-entry smoke table.
    #[test]
    fn the_torch_dust_table_matches_the_source_rows() {
        assert_eq!(torch_dust(0), 6);
        assert_eq!(torch_dust(1), 59);
        assert_eq!(torch_dust(14), 66); // the disco torch
        assert_eq!(torch_dust(15), 242);
        assert_eq!(torch_dust(23), 310);
    }

    /// `TileID.cs:538` / `:480` — the two sets.
    #[test]
    fn the_nature_fly_and_rubble_sets_match_the_source_rows() {
        for k in [3u16, 20, 24, 27, 73, 201, 80, 110, 529, 530, 595, 615, 637] {
            assert!(spawns_nature_flies(k), "flies {k}");
        }
        assert!(!spawns_nature_flies(2));
        assert!(!spawns_nature_flies(61));
        for k in [0u16, 1, 2, 192, 384, 718, 719, 666, 633, 477] {
            assert!(makes_rubble_dust(k), "rubble {k}");
        }
        assert!(!makes_rubble_dust(4)); // torches make no rubble
        assert!(!makes_rubble_dust(21));
    }

    /// `TileDrawing.cs:1411-1432` — the visibility arms.
    #[test]
    fn is_visible_hides_the_19_row48_and_the_541_631_pair() {
        let mut t = Tile {
            kind: 19,
            frame_y: 48 * 18,
            ..Default::default()
        };
        assert!(!is_visible(&t, false));
        assert!(is_visible(&t, true));
        t.frame_y = 0;
        assert!(is_visible(&t, false));
        t.kind = 541;
        assert!(!is_visible(&t, false));
        t.kind = 631;
        assert!(!is_visible(&t, false));
        t.kind = 1;
        assert!(is_visible(&t, false));
    }

    // --- RNG census --------------------------------------------------------

    /// `:7256-7259` — the 646 arm draws ONE discarded Next(2); the
    /// rubble Next(1200) (`:6901`) runs FIRST (the 646 arm is past it).
    #[test]
    fn the_646_arm_draws_one_discarded_next2() {
        let mut w = world();
        put(&mut w, 50, 50, 646, 0, 0);
        let mut rand = ScriptRand::new().with(&[1]); // N1200 -> 1 (refuse), N2 -> 0
        let (out, dust, gore, _) = run(&w, 50, 50, &ctx(), &mut rand, 1);
        assert_eq!(rand.log, vec!["N1200", "N2"]);
        assert!(out.particles.is_empty());
        assert!(out.rubble_dust.is_none());
        assert!(!dust.slot(0).active);
        assert!(!gore.slot(0).active);
    }

    /// `:7105-7127` — the torch gate draws Next(40) BEFORE the
    /// frameX < 66 test: a frameX-66 torch still consumes the roll.
    #[test]
    fn the_torch_gate_draws_before_the_framex_test() {
        let mut w = world();
        put(&mut w, 50, 50, 4, 66, 0);
        let mut rand = ScriptRand::new().with(&[1, 0]); // N1200 refuse, N40 pass
        let (_, dust, _, _) = run(&w, 50, 50, &ctx(), &mut rand, 1);
        assert_eq!(rand.log, vec!["N1200", "N40"]);
        assert!(!dust.slot(0).active, "frameX 66 never spawns");
    }

    /// `:7105-7127` — a passing torch spawn: the 2-of-3 noGravity roll
    /// rides `_rand` AFTER the pool's own Main.rand draws; the disco
    /// torch (style 14, dust 66) forces the colour + noGravity.
    #[test]
    fn the_disco_torch_dust_takes_the_disco_colour() {
        let mut w = world();
        put(&mut w, 50, 50, 4, 0, 14 * 22);
        let mut rand = ScriptRand::new().with(&[1, 0, 0]); // N1200, N40, N3
        let (_, dust, _, _) = run(&w, 50, 50, &ctx(), &mut rand, 1);
        assert_eq!(rand.log, vec!["N1200", "N40", "N3"]);
        let d = dust.slot(0);
        assert!(d.active);
        assert_eq!(d.dust_type, 66);
        assert_eq!(d.color, Color::new(255, 0, 0, 255));
        assert!(d.no_gravity);
        assert_eq!(d.alpha, 100);
    }

    /// `:7226-7240` — the 49 lava bubble: the 1/3 small arm (scale
    /// 0.5, gravity kept) vs the 2/3 large arm (0.9 + noGravity).
    #[test]
    fn the_lava_bubble_splits_small_and_large() {
        let mut w = world();
        put(&mut w, 50, 50, 49, 0, 0);
        // N1200 refuse, N2 pass, N3 -> 0 (small).
        let mut rand = ScriptRand::new().with(&[1, 0, 0]);
        let (_, dust, _, _) = run(&w, 50, 50, &ctx(), &mut rand, 1);
        assert_eq!(rand.log, vec!["N1200", "N2", "N3"]);
        let d = dust.slot(0);
        assert_eq!(d.dust_type, 172);
        assert_eq!(d.scale, 0.5);
        assert!(!d.no_gravity);
        // N3 -> 1 (large).
        let mut rand = ScriptRand::new().with(&[1, 0, 1]);
        let (_, dust, _, _) = run(&w, 50, 50, &ctx(), &mut rand, 1);
        let d = dust.slot(0);
        assert_eq!(d.scale, 0.9);
        assert!(d.no_gravity);
    }

    /// `:6780-6795` — the 215 campfire draws Next(3) even when the
    /// frameY == 0 re-test fails (the C# && order).
    #[test]
    fn the_campfire_gate_draws_before_the_framey_retest() {
        let mut w = world();
        put(&mut w, 50, 50, 215, 0, 18); // fy < 36 but != 0
        let mut rand = ScriptRand::new().with(&[0, 1]); // N3 pass, N1200 refuse
        let (_, dust, _, _) = run(&w, 50, 50, &ctx(), &mut rand, 1);
        assert_eq!(rand.log, vec!["N3", "N1200"]);
        assert!(!dust.slot(0).active);
    }

    /// `:6848-6855` + `:7549-7556` — the 192 leaf gate rolls
    /// leafFrequency/4, then the below-spawn, then the UNCONDITIONAL
    /// Next(2) sideways roll (the inline tiles.rs arm gates it on the
    /// wind pick — the deviation the wiring reconciles).
    #[test]
    fn the_living_wood_leaf_draws_the_sideways_roll_unconditionally() {
        let mut w = world();
        put(&mut w, 50, 50, 192, 0, 0);
        // N3500 pass (14000/4), N2 -> 1 (no sideways, drawn INSIDE the
        // leaf emit), N1200 refuse.
        let mut rand = ScriptRand::new().with(&[0, 1, 1]);
        let (_, _, gore, _) = run(&w, 50, 50, &ctx(), &mut rand, 1);
        assert_eq!(rand.log, vec!["N3500", "N2", "N1200"]);
        // The below-spawn fired (calm wind, open air below).
        assert!(gore.slot(0).active);
        assert_eq!(gore.slot(0).gore_type, 910);
        assert_eq!(gore.slot(0).position, Vec2::new(50.0 * 16.0, 50.0 * 16.0 + 16.0));
    }

    /// `:6863-6880` — the 711 boulder: the Next(45) and Next(3) gates
    /// both ride `_rand`; the boulder-2 position/movement draws ride
    /// Main.rand (two Circular pairs = four draws).
    #[test]
    fn the_rainbow_boulder_requests_ride_the_two_streams() {
        let mut w = world();
        put(&mut w, 50, 50, 711, 0, 0);
        // N45 -> 0 (boulder3), N3 -> 1 (boulder2), N1200 refuse.
        let mut rand = ScriptRand::new().with(&[0, 1, 1]);
        let mut main_before = UnifiedRandom::new(7);
        let (out, _, _, main_after) = run(&w, 50, 50, &ctx(), &mut rand, 7);
        assert_eq!(rand.log, vec!["N45", "N3", "N1200"]);
        assert_eq!(out.particles.len(), 2);
        assert_eq!(out.particles[0].kind, 49);
        assert_eq!(out.particles[0].position, Vec2::new(50.0 * 16.0 + 16.0, 50.0 * 16.0 + 16.0));
        assert_eq!(out.particles[1].kind, 48);
        // The four Main.rand draws: angle, radius, angle, radius.
        let a1 = main_before.next_float() * std::f32::consts::TAU;
        let r1 = main_before.next_float();
        let a2 = main_before.next_float() * std::f32::consts::TAU;
        let r2 = main_before.next_float();
        let want_pos = Vec2::new(50.0 * 16.0 + 16.0, 50.0 * 16.0 + 16.0)
            + Vec2::from_angle(a1) * Vec2::new(16.0, 16.0) * r1;
        let want_mov = Vec2::from_angle(a2) * Vec2::new(1.0, 0.5) * r2 * 0.5;
        assert_eq!(out.particles[1].position, want_pos);
        assert_eq!(out.particles[1].movement, want_mov);
        assert_eq!(main_before, main_after, "exactly four Main.rand draws");
    }

    /// `:6881-6900` — the nature-fly gate: NextFloat < lerp + 0.3,
    /// then Next(600); a miss consults the WindGrid window and draws
    /// Next(48) ONLY when the window is live.
    #[test]
    fn the_nature_fly_gust_gate_draws_next48_only_in_the_window() {
        let mut w = world();
        put(&mut w, 50, 50, 3, 0, 0); // grass
        // F -> 0.0 (< 0.3), N600 -> 1 (miss), empty grid -> no N48,
        // N1200 refuse.
        let mut rand = ScriptRand::new().with(&[1, 1]).with_floats(&[0.0]);
        let (out, _, _, _) = run(&w, 50, 50, &ctx(), &mut rand, 1);
        assert_eq!(rand.log, vec!["F", "N600", "N1200"]);
        assert!(out.particles.is_empty());
        // N600 -> 0 (hit): no grid consult, the request fires.
        let mut rand = ScriptRand::new().with(&[0, 1]).with_floats(&[0.0]);
        let (out, _, _, _) = run(&w, 50, 50, &ctx(), &mut rand, 1);
        assert_eq!(rand.log, vec!["F", "N600", "N1200"]);
        assert_eq!(out.particles.len(), 1);
        assert_eq!(out.particles[0].kind, 53);
        assert_eq!(out.particles[0].position, Vec2::new(50.0 * 16.0 + 8.0, 50.0 * 16.0));
    }

    /// The live-window half: a stamped gust cell makes the miss path
    /// draw Next(48) (`:6890`), and a 0 answer requests the fly.
    #[test]
    fn the_nature_fly_gust_window_draws_next48() {
        let mut w = world();
        put(&mut w, 50, 50, 3, 0, 0);
        let mut grid = WindGrid::new();
        grid.set_wind_time(50, 50, 1, 0); // stamp at game_time 0
        grid.update(); // game_time 1 -> time_left = 1 (> 0, window live)
        let t = tile_at(&w, 50, 50);
        let mut rand = ScriptRand::new().with(&[1, 0, 1]).with_floats(&[0.0]);
        let mut dust = DustPool::new();
        let mut gore = GorePool::new();
        let mut main_rand = UnifiedRandom::new(1);
        let mut out = EmitOut::default();
        draw_tiles_emit_particles(
            &w, 50, 50, &t, 0, 0, Color::WHITE, &ctx(), &mut rand, &mut main_rand,
            &mut dust, &dust_ctx(), &mut gore, &grid, &mut out,
        );
        assert_eq!(rand.log, vec!["F", "N600", "N48", "N1200"]);
        assert_eq!(out.particles.len(), 1);
        assert_eq!(out.particles[0].kind, 53);
    }

    /// `:6901-6951` — the rubble gate: above the surface the Next(10)
    /// arm cuts the count to 2 and arms the scale cut; the wall gate
    /// draws Next(2) only for a wind-blocking wall.
    #[test]
    fn the_rubble_gate_plans_the_count_and_the_surface_cut() {
        let mut w = world();
        w.world_surface = 100.0;
        put(&mut w, 50, 50, 1, 0, 0); // dirt, above the surface
        // N1200 pass, N10 -> 0 (surface arm: count 2, small), the
        // walkable-below probe fails (air), wall 0 allows wind (no N2).
        let mut rand = ScriptRand::new().with(&[0, 0]);
        let (out, _, _, _) = run(&w, 50, 50, &ctx(), &mut rand, 1);
        assert_eq!(rand.log, vec!["N1200", "N10"]);
        assert_eq!(out.rubble_dust, Some(RubbleDustPlan { count: 2, small: true }));
        // Below the surface: no N10; a wind-blocking wall draws N2.
        let mut w = world();
        w.world_surface = 10.0;
        put(&mut w, 50, 50, 1, 0, 0);
        w.tiles[w.index(50, 50)].wall = 1; // stone wall blocks wind
        let mut rand = ScriptRand::new().with(&[0, 1]); // N1200, N2 -> 1 (count 2)
        let (out, _, _, _) = run(&w, 50, 50, &ctx(), &mut rand, 1);
        assert_eq!(rand.log, vec!["N1200", "N2"]);
        assert_eq!(out.rubble_dust, Some(RubbleDustPlan { count: 2, small: false }));
    }

    /// `:7413-7432` — the shine arm: the dark-light return precedes
    /// the roll; the roll rides `_rand` BEFORE the frame windows; a
    /// passing roll spawns the 43 sparkle with zero velocity.
    #[test]
    fn the_shine_arm_rolls_before_the_frame_windows() {
        let mut w = world();
        put(&mut w, 50, 50, 6, 0, 0); // iron ore, shine 1150
        // Dark light: no draw at all.
        let t = tile_at(&w, 50, 50);
        let mut rand = ScriptRand::new().with(&[1]);
        let mut dust = DustPool::new();
        let mut gore = GorePool::new();
        let mut main_rand = UnifiedRandom::new(1);
        let mut out = EmitOut::default();
        draw_tiles_emit_particles(
            &w, 50, 50, &t, 0, 0, Color::new(10, 10, 10, 255), &ctx(), &mut rand,
            &mut main_rand, &mut dust, &dust_ctx(), &mut gore, &WindGrid::new(), &mut out,
        );
        // Only the rubble gate ran (N1200 -> 1 refuse); the shine arm
        // returned on the dark light BEFORE its Next(1150).
        assert_eq!(rand.log, vec!["N1200"]);
        assert!(!dust.slot(0).active);
        // Bright light: N1150 -> 0 (< 255/30 = 8) spawns the sparkle.
        let mut rand = ScriptRand::new().with(&[1, 0]);
        let (_, dust, _, _) = run(&w, 50, 50, &ctx(), &mut rand, 1);
        assert_eq!(rand.log, vec!["N1200", "N1150"]);
        let d = dust.slot(0);
        assert!(d.active);
        assert_eq!(d.dust_type, 43);
        assert_eq!(d.velocity, Vec2::ZERO);
        assert_eq!(d.alpha, 254);
    }

    /// `:7530-7545` — the shimmer arm: a lit solid with shimmerAlpha
    /// rolls Next(500) against 2*alpha; the scale is (maxLight/255+1)/2.
    #[test]
    fn the_shimmer_sparkle_scales_with_the_light() {
        let mut w = world();
        put(&mut w, 50, 50, 1, 0, 0); // dirt (solid, no shine)
        let mut c = ctx();
        c.shimmer_alpha = 1.0;
        // N1200 refuse, N500 -> 0 (< 2).
        let mut rand = ScriptRand::new().with(&[1, 0]);
        let (_, dust, _, _) = run(&w, 50, 50, &c, &mut rand, 1);
        assert_eq!(rand.log, vec!["N1200", "N500"]);
        let d = dust.slot(0);
        assert_eq!(d.dust_type, 43);
        // NewDust multiplies the input scale by 1 + Next(-20,21)*0.01
        // (Dust.cs:186-187) — the (maxLight/255+1)/2 = 1.0 input rides
        // that band.
        assert!(d.scale >= 0.8 && d.scale <= 1.2, "scale {}", d.scale);
        assert_eq!(d.velocity, Vec2::ZERO);
        assert_eq!(d.color, Color::WHITE);
    }

    /// `:9605-9655` — the herb styles: 3 deathweed draws TWO
    /// independent gates (200 then 75); 6 shiverthorn's 43 dust takes
    /// the cyan colour and zero velocity.
    #[test]
    fn the_deathweed_herb_draws_two_independent_gates() {
        let mut w = world();
        let mut rand = ScriptRand::new().with(&[0, 1]); // N200 pass, N75 miss
        let mut dust = DustPool::new();
        let mut main_rand = UnifiedRandom::new(1);
        emit_alchemy_herb_particles(
            800.0, 800.0, 3, &mut rand, &mut main_rand, &mut dust, &dust_ctx(), &w,
        );
        assert_eq!(rand.log, vec!["N200", "N75"]);
        assert!(dust.slot(0).active);
        assert_eq!(dust.slot(0).dust_type, 14);
        assert_eq!(dust.slot(0).fade_in, 1.2);
        assert!(!dust.slot(1).active);
    }

    #[test]
    fn the_shiverthorn_herb_sparkle_is_cyan_and_still() {
        let w = world();
        let mut rand = ScriptRand::new().with(&[0]); // N30 pass
        let mut dust = DustPool::new();
        let mut main_rand = UnifiedRandom::new(1);
        emit_alchemy_herb_particles(
            800.0, 800.0, 6, &mut rand, &mut main_rand, &mut dust, &dust_ctx(), &w,
        );
        assert_eq!(rand.log, vec!["N30"]);
        let d = dust.slot(0);
        assert_eq!(d.dust_type, 43);
        assert_eq!(d.color, Color::new(50, 255, 255, 255));
        assert_eq!(d.velocity, Vec2::ZERO);
        assert_eq!(d.alpha, 254);
    }

    /// `:6692-6751` — the 718 star cloud: the three RollLuck gates
    /// ride Main.rand (luck 0 = one Next(range) each); the position /
    /// scale / velocity tweaks ride `_rand`. The luck gates are
    /// probabilistic on Main.rand, so this witness scans seeds for one
    /// where the FIRST gate (RollLuck 100) passes, then pins the
    /// per-gore `_rand` census prefix + the second-gore field copy.
    #[test]
    fn the_star_cloud_sparkles_census_the_two_streams() {
        let mut w = world();
        w.day_time = false;
        put(&mut w, 50, 50, 718, 0, 0);
        // The first gore's `_rand` census (N5 -> 0 arms the extra kick,
        // N3 -> 0 halves the velocity): N3 gate, N16 pos, F scale,
        // R5,31 base kick, N5 gate, R5,41 extra kick, N3 gate.
        let prefix = ["N3", "N16", "F", "R5,31", "N5", "R5,41", "N3"];
        let mut found = None;
        for seed in 1..4000 {
            // Answers for the first gore's tweaks; the rest default to
            // 0 (gate-passing) which is harmless for the prefix proof.
            let mut rand = ScriptRand::new().with(&[0, 0, 0, 0]).with_floats(&[0.5]);
            let (_, _, gore, _) = run(&w, 50, 50, &ctx(), &mut rand, seed);
            if gore.slot(0).active && gore.slot(0).gore_type == 16 {
                found = Some((rand, gore));
                break;
            }
        }
        let (rand, gore) = found.expect("a seed whose RollLuck(100) gate passes");
        let got: Vec<&str> = rand.log.iter().map(|s| s.as_str()).collect();
        assert_eq!(&got[..prefix.len()], &prefix[..]);
        // The second gore of the pair is a fresh NewGore then a field
        // copy of the first (`:6709-6712`).
        assert!(gore.slot(1).active, "the second gore of the pair");
        assert_eq!(gore.slot(1).gore_type, 16);
        assert_eq!(gore.slot(1).position, gore.slot(0).position);
        assert_eq!(gore.slot(1).velocity, gore.slot(0).velocity);
        assert_eq!(gore.slot(1).scale, gore.slot(0).scale);
    }

    /// `:6752-6774` — the 244 pumpkin candle: the five-tier chain
    /// draws N500/N250/N80/N10/N3 in order until a hit; the scale
    /// argument R51,101 rides `_rand` after the tier pick. The 244
    /// arm is BEFORE the rubble gate, so N1200 comes last.
    #[test]
    fn the_pumpkin_candle_tiers_draw_in_order() {
        let mut w = world();
        put(&mut w, 50, 50, 244, 18, 18);
        // N2 gate pass; N500/N250/N80/N10 miss; N3 -> 0 (tier 411);
        // R51,101 scale; then N1200 refuse.
        let mut rand = ScriptRand::new().with(&[0, 1, 1, 1, 1, 0, 1]);
        let (_, _, gore, _) = run(&w, 50, 50, &ctx(), &mut rand, 1);
        assert_eq!(
            rand.log,
            vec!["N2", "N500", "N250", "N80", "N10", "N3", "R51,101", "N1200"]
        );
        assert!(gore.slot(0).active);
        assert_eq!(gore.slot(0).gore_type, 411);
        assert_eq!(gore.slot(0).position, Vec2::new(50.0 * 16.0 + 8.0, 50.0 * 16.0 + 8.0));
    }

    /// `:6812-6832` — the 406 chimney: the type draw R825,828 precedes
    /// the tier gates; the left-only wind push. The 406 arm is BEFORE
    /// the rubble gate.
    #[test]
    fn the_chimney_smoke_pushes_left_only() {
        let mut w = world();
        put(&mut w, 50, 50, 406, 0, 54);
        let mut c = ctx();
        c.wind = -0.5;
        // N3 pass, R825,828 -> 825, N4 -> 0 (small tier), F scale,
        // then N1200 refuse.
        let mut rand = ScriptRand::new().with(&[0, 825, 0, 1]).with_floats(&[0.5]);
        let (_, _, gore, _) = run(&w, 50, 50, &c, &mut rand, 1);
        assert_eq!(rand.log, vec!["N3", "R825,828", "N4", "F", "N1200"]);
        let g = gore.slot(0);
        assert_eq!(g.gore_type, 825);
        // The wind push is left-only: velocity.X starts at -wind = 0.5
        // before NewGore's ±2 X kick (Main.rand). The 825 type has no
        // velocity-reset arm, so the push survives.
        assert!(g.velocity.x >= 0.5 - 2.0 - 0.01);
    }

    /// `:6962-6980` — the 139 lantern petals read the RAW tile frames
    /// and the timeForVisualEffects % 7 cadence. The 139 arm is AFTER
    /// the rubble gate, so N1200 comes first.
    #[test]
    fn the_lantern_petals_need_the_raw_frames_and_the_seven_cadence() {
        let mut w = world();
        put(&mut w, 50, 50, 139, 36, 36);
        let mut c = ctx();
        c.time_for_visual_effects = 7.0; // % 7 == 0
        // N1200 refuse, N3 pass, R570,573 -> 572, R-50,51 X, R-50,51 Y.
        let mut rand = ScriptRand::new().with(&[1, 0, 572, 0, 0]);
        let (_, _, gore, _) = run(&w, 50, 50, &c, &mut rand, 1);
        assert_eq!(rand.log, vec!["N1200", "N3", "R570,573", "R-50,51", "R-50,51"]);
        let g = gore.slot(0);
        assert_eq!(g.gore_type, 572);
        // The 572 X shift: base (808, 792) - 8 = 800.
        assert_eq!(g.position.x, 50.0 * 16.0 + 8.0 - 8.0);
        // A non-cadence tick draws nothing for the 139 arm.
        c.time_for_visual_effects = 8.0;
        let mut rand = ScriptRand::new().with(&[1]);
        let (_, _, gore, _) = run(&w, 50, 50, &c, &mut rand, 1);
        assert_eq!(rand.log, vec!["N1200"]);
        assert!(!gore.slot(0).active);
    }
}

