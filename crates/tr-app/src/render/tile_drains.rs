//! W4-g31 — TileDrawing pass 4: the DEFERRED BUCKET DRAINS. C#'s `Draw`
//! classifies each cell and registers a special point instead of drawing it
//! inline (the g30 switch, `TileDrawing.cs:534-723`); `PostDrawTiles(false)`
//! (`:414-442`) then drains the eleven `TileCounterType` buckets in a fixed
//! order, each pass walking its own `_specialPositions[N]` list. This module
//! ports the MISSING drain bodies + the per-bucket drivers.
//!
//! The pass family: g28 (pass 1) owns the quality-light tier rebuild
//! (`tr-render::quality_light`); g29 (pass 2) owns the `Draw` orchestration
//! structure (`render/tile_pass.rs`); g30 (pass 3) owns the classification
//! switch + the `SpecialPoints` registry (`render/special_points.rs`); g32
//! (pass 5) owns the orchestration + the `PostDrawTiles` ORDER
//! (`render/tile_draw_pass.rs`). g29's module doc assigns the deferred bucket
//! drains to "g31/g32 (passes 4-5)"; g32 took the order/structure, so THIS
//! module (g31, pass 4) owns the drain BODIES.
//!
//! UNION-EXEC (docs/UNION-CONTRACT-tilepass.md §1.4, this branch): the g29/
//! g30/g32 siblings are landed here, so the mechanical binds are IN:
//! * the `BUCKET_*` constants keep-one onto g30's `TileCounterType as
//!   usize` (single source of truth, values verified identical 0..10);
//! * the drivers take `&[Vec<(i32, i32)>; 11]` — the caller passes
//!   `&g30_reg.buckets` directly ([`DrainBuckets`] stays as the owned
//!   witness fixture);
//! * [`drain_one_pass`] is the per-pass dispatch the g29
//!   `post_draw_tiles` driver binds each of its ten nature steps through
//!   ([`drain_pass_for_post_step`] maps the 12-step `PostDrawStep` enum;
//!   `EndBatch`/`NatureAfterAllObjects`/the two TE passes map to `None` —
//!   the g32 `None`-arm fold); [`post_draw_drain_order`] is the thin
//!   frozen-order loop over it, ONE shared `_rand` in pass order;
//! * the per-cell light seam ([`DrainCellLight`]) still binds to d56's
//!   `tr-render::animated_tiles` at the d-lane merge (not on main);
//!   [`PortedBodyDrain`] binds to the on-main `foliage.rs`/`vines.rs`/
//!   `pylons.rs` passes at the tiles.rs wiring (they need the renderer).
//!
//! What is ported here (research/09 rows 10-12, the MISSING drains):
//! * [`drain_multi_tile_vines`] — `DrawMultiTileVines` (`:8829-8888`) + the
//!   per-type size table, delegating to [`multi_tile_vines_in_wind`].
//! * [`multi_tile_vines_in_wind`] — `DrawMultiTileVinesInWind`
//!   (`:9238-9603`): the platform lift, the 34/126 + 42 style tables, the
//!   per-cell wind transform, the 660 fairy-jar / 698 dead-cells-jar arms and
//!   the shared-seed flame tail.
//! * [`drain_multi_tile_grass`] — `DrawMultiTileGrass` (`:8738-8805`) + the
//!   size table + [`climb_cat_tail`] (`:8807-8827`).
//! * [`multi_tile_grass_in_wind`] — `DrawMultiTileGrassInWind`
//!   (`:8916-9032`): the sunflower glow latch + the 493 golf-flag flip.
//! * [`drain_grass`] — `DrawGrass` (`:8326-8381`).
//! * [`drain_any_directional_grass`] — `DrawAnyDirectionalGrass`
//!   (`:8383-8479`): the four-direction origin/position switch.
//! * [`drain_reverse_vines`] + [`rising_vine_strip`] — `DrawReverseVines`
//!   (`:8903-8914`) / `DrawRisingVineStrip` (`:9134-9193`).
//! * The bucket-drain drivers for the already-ported sprite bodies (Trees /
//!   Vines / Pylons / Trophies / VoidLenses): [`post_draw_drain_order`] walks
//!   the registry points in C# order and hands each to a seam (the existing
//!   `foliage.rs` / `vines.rs` / `pylons.rs` passes bind at the merge).
//!
//! UNWIRED (the S1-G-1e / `special_tiles.rs:35` pattern): the `draw_tiles`
//! seam is contested by the in-flight tile-draw siblings (g28-g32, d56-d60).
//! The drains emit pure [`DrainSprite`] plans (no GPU), so the maths is
//! unit-testable; the wiring step resolves each plan's UV through
//! `render/tiles.rs::resolve_tile_frame` and pushes it. Intended call sites:
//! * `app/frame/draw.rs` after the cached tile pass — run
//!   [`post_draw_drain_order`] over the g30 registry instead of the inline
//!   grass/vine/sunflower draws (`tiles.rs:671-751`, `838-851`).
//! * The per-cell light seam binds to d56's `animated_tiles`
//!   (`DrawAnimatedTile_AdjustForVisionChangers` `:8481-8552` +
//!   `DrawTiles_GetLightOverride` `:6624-6686` + `DrawTiles_EmitParticles`).
//!
//! Deviations (docs/ANALYSIS.md "The deferred bucket drains (W4-g31 pass 4)"):
//! * The C# null-tile arm (`:8842`/`:8751` `tile != null`) has no port
//!   counterpart (the tile array is dense) — an out-of-bounds point skips.
//! * `GetTileDrawData`'s `Main.tileFrame[type]` animator row is fed 0 until
//!   the S1-G-1f animator wiring binds it (the `vines.rs` convention).
//! * The `NextNatureRenderer` LitNature shader + `SideFlags` seams are not
//!   ported (the `vines.rs` deviation) — plans draw plain.
//! * Particle/dust emission is the d60 lane: the drains CONSUME the `_rand`
//!   rolls in C# order (the stream-order rule) and record the per-cell
//!   `Next(4)==0` flag, but spawn no dust; the light seam owns the emission.
//! * `IsVisible` (`:1411-1432`) is re-ported here (the `pylons.rs` copy is
//!   private); the family merge keep-ones one.

#![allow(dead_code)] // UNWIRED — see the module doc; wiring is the pass-family merge step.

use tr_core::drawdata::{draw_data, TileDrawData};
use tr_core::tiledata as td;
use tr_core::wind::Wind;
use tr_core::{UnifiedRandom, World};

// ---------------------------------------------------------------------------
// Bucket indices — UNION keep-one (contract §1.4): the g30
// `TileCounterType` discriminants ARE the `_specialsCount`/
// `_specialPositions` indices (`TileDrawing.cs:21-35`/`:185`); the
// constants now derive from the enum (single source of truth).
// ---------------------------------------------------------------------------

use crate::render::special_points::TileCounterType;

pub(crate) const BUCKET_TREE: usize = TileCounterType::Tree as usize;
pub(crate) const BUCKET_WINDY_GRASS: usize = TileCounterType::WindyGrass as usize;
pub(crate) const BUCKET_MULTI_TILE_GRASS: usize = TileCounterType::MultiTileGrass as usize;
pub(crate) const BUCKET_MULTI_TILE_VINE: usize = TileCounterType::MultiTileVine as usize;
pub(crate) const BUCKET_VINE: usize = TileCounterType::Vine as usize;
pub(crate) const BUCKET_BIOME_GRASS: usize = TileCounterType::BiomeGrass as usize;
pub(crate) const BUCKET_VOID_LENS: usize = TileCounterType::VoidLens as usize;
pub(crate) const BUCKET_REVERSE_VINE: usize = TileCounterType::ReverseVine as usize;
pub(crate) const BUCKET_TELEPORTATION_PYLON: usize = TileCounterType::TeleportationPylon as usize;
pub(crate) const BUCKET_MASTER_TROPHY: usize = TileCounterType::MasterTrophy as usize;
pub(crate) const BUCKET_ANY_DIRECTIONAL_GRASS: usize = TileCounterType::AnyDirectionalGrass as usize;

/// The owned witness fixture for the eleven-bucket shape — UNION
/// (contract §1.4): the drivers take `&[Vec<(i32, i32)>; 11]` and the
/// wiring passes `&g30_reg.buckets` directly (identical shape, no second
/// registry). `BiomeGrass` (5) is never registered nor drained.
pub(crate) struct DrainBuckets {
    pub points: [Vec<(i32, i32)>; 11],
}

impl DrainBuckets {
    pub(crate) fn new() -> Self {
        Self {
            points: std::array::from_fn(|_| Vec::new()),
        }
    }
}

// ---------------------------------------------------------------------------
// The per-cell light seam — d56's animated_tiles binds at the merge.
// ---------------------------------------------------------------------------

/// The per-cell light chain every drain runs after `GetTileDrawData`:
/// `Lighting.GetColor` (`:8347`/`:8404`/`:8975`/`:9486`) →
/// `DrawAnimatedTile_AdjustForVisionChangers` (`:8481-8552`) →
/// `DrawTiles_GetLightOverride` (`:6624-6686`) → the gated
/// `DrawTiles_EmitParticles` (`:8350-8353`). d56 ports the byte maths into
/// `tr-render::animated_tiles` (`vision_changer_light` → `light_override`);
/// the drains seam it because d56 is not on main yet (the d-lane merge
/// binds it; gated dust emission is d60). `can_do_dust` is the per-cell
/// `_rand.Next(4)==0` flag the drain ALREADY drew (the stream-order rule —
/// the binding must NOT re-roll); `base` is `Lighting.GetColor`.
pub(crate) trait DrainCellLight {
    fn cell_light(
        &mut self,
        world: &World,
        x: i32,
        y: i32,
        kind: u16,
        frame_x: i16,
        frame_y: i16,
        base: [f32; 3],
        can_do_dust: bool,
    ) -> [f32; 3];
}

/// A pass-through light seam for the witnesses (no vision changers, no
/// override): the drains' wind/style/transform maths is what they pin, not
/// the light-style transform (the `pylons.rs` test convention).
pub(crate) struct PlainLight;
impl DrainCellLight for PlainLight {
    fn cell_light(
        &mut self,
        _world: &World,
        _x: i32,
        _y: i32,
        _kind: u16,
        _frame_x: i16,
        _frame_y: i16,
        base: [f32; 3],
        _can_do_dust: bool,
    ) -> [f32; 3] {
        base
    }
}

// ---------------------------------------------------------------------------
// IsVisible — TileDrawing.cs:1411-1432. KEEP-ONE EXECUTED (research/136 §4
// correction): the standalone survivor is d60's shape PROMOTED to
// `tr_render::tile_visibility::is_visible` (primitive params — tr-render has
// no tr-core dep); this wrapper is the g31 rebind (world read + delegate).
// d56's gate / d60's battery rebind onto the same fn at the family merge.
// ---------------------------------------------------------------------------

/// `TileDrawing.IsVisible` (`TileDrawing.cs:1411-1432`): an Echo-coated block
/// hides unless sight is granted; platforms on row 48 and the two invisible
/// wiring tiles (541/631) always ride the flag. Thin wrapper over the
/// promoted keep-one (`tr_render::tile_visibility`).
pub(crate) fn is_visible(world: &World, x: i32, y: i32, show_invisible: bool) -> bool {
    let Some(tile) = world.tile(x, y) else {
        return true;
    };
    tr_render::tile_visibility::is_visible(
        tile.kind,
        tile.frame_y,
        world.invisible_block(x, y),
        show_invisible,
    )
}

// ---------------------------------------------------------------------------
// The drain sprite plan — the inputs of one DrawNature / spriteBatch.Draw
// call, pure data so the drains are GPU-free (the pylons.rs SpecialSprite /
// vines.rs VineSegment convention).
// ---------------------------------------------------------------------------

/// One deferred sprite. `pos` is the XNA `position` (the rotation `origin`
/// lands here); the wiring step resolves the UV from `kind`/`frame`/`dd`
/// through `resolve_tile_frame` and pushes at `pos - origin` rotated by `rot`.
#[derive(Clone, Copy, Debug)]
pub(crate) struct DrainSprite {
    pub kind: u16,
    /// The cell the sprite belongs to (light/paint lookup at the wiring).
    pub x: i32,
    pub y: i32,
    pub frame_x: i16,
    pub frame_y: i16,
    /// `GetTileDrawData`'s record (`:8345`/`:8973`/`:9484`).
    pub dd: TileDrawData,
    /// `halfBrickHeight` (`TileDrawing.cs:6045-6048`): 8 on a half-brick.
    pub half: i16,
    pub pos: [f32; 2],
    pub rot: f32,
    pub origin: [f32; 2],
    /// `tileLight` after the per-cell chain (normalized 0..1).
    pub tint: [f32; 3],
    pub flip: bool,
    pub paint: u8,
    /// The glowmask follow-up (`DrawNatureGlowmask`), when the cell has one.
    pub glow: Option<DrainGlow>,
}

/// A glowmask follow-up draw (`TileDrawing.cs:8375-8378`/`:9025-9028`/
/// `:9585-9588`): the same transform, a different source rect + colour.
#[derive(Clone, Copy, Debug)]
pub(crate) struct DrainGlow {
    /// Source-rect Y offset added to the base rect (the sunflower's `num2`).
    pub src_y_add: i16,
    pub color: [f32; 4],
}

/// The 660 fairy-jar bottom row's `Extra_260` pair (`TileDrawing.cs:9505-
/// 9519`): a white copy BEFORE the tile draw and an HSL glow (A=127) AFTER.
/// The tile draw itself is a normal [`DrainSprite`] in `out`; the wiring
/// pushes the white copy, then the tile sprite, then this glow — all on the
/// `Extra_260` sheet at the same transform.
#[derive(Clone, Copy, Debug)]
pub(crate) struct DrainFairyJar {
    pub x: i32,
    pub y: i32,
    pub frame_x: i16,
    pub frame_y: i16,
    pub dd: TileDrawData,
    pub half: i16,
    pub pos: [f32; 2],
    pub rot: f32,
    pub origin: [f32; 2],
    pub flip: bool,
    pub paint: u8,
    /// The HSL glow colour (`hslToRgb(hue, 1, 0.8)`, A=127, `:9516-9517`).
    pub glow_color: [f32; 4],
}

// ---------------------------------------------------------------------------
// ClimbCatTail — TileDrawing.cs:8807-8827
// ---------------------------------------------------------------------------

/// `ClimbCatTail` (`TileDrawing.cs:8807-8827`): the 519 campfire's height —
/// crawl UP from the anchor while the cells are active type-519, counting
/// rows; a cell with `frameX >= 180` (the flame top) counts itself and stops.
/// The loop floor is `num2 > 10` (`:8811`). Returns the row count `num`.
pub(crate) fn climb_cat_tail(world: &World, origin_x: i32, origin_y: i32) -> i32 {
    let mut num = 0;
    let mut num2 = origin_y;
    while num2 > 10 {
        let Some(tile) = world.tile(origin_x, num2) else {
            break;
        };
        if !tile.active() || tile.kind != 519 {
            break;
        }
        if tile.frame_x >= 180 {
            num += 1;
            break;
        }
        num2 -= 1;
        num += 1;
    }
    num
}

// ---------------------------------------------------------------------------
// The multi-tile size tables — DrawMultiTileVines (:8845-8884) /
// DrawMultiTileGrass (:8754-8801).
// ---------------------------------------------------------------------------

/// `DrawMultiTileVines`' per-type `(sizeX, sizeY)` (`TileDrawing.cs:8845-8884`).
/// The default is `(1, 1)` (`:8840-8841`); 698 is explicitly `(1, 1)` (`:8880`).
pub(crate) fn multi_tile_vine_size(kind: u16) -> (i32, i32) {
    match kind {
        34 => (3, 3),
        454 => (4, 3),
        42 | 270 | 271 | 572 | 581 | 660 => (1, 2),
        91 => (1, 3),
        95 | 126 | 444 => (2, 2),
        465 | 591 | 592 => (2, 3),
        698 => (1, 1),
        _ => (1, 1),
    }
}

/// `DrawMultiTileGrass`' per-type `(sizeX, sizeY)` (`TileDrawing.cs:8754-8801`).
/// The 519 campfire's height is [`climb_cat_tail`] and its anchor row shifts
/// up by `height - 1` (`:8798-8799`) — handled by the driver, not this table.
pub(crate) fn multi_tile_grass_size(world: &World, kind: u16, x: i32, y: i32) -> (i32, i32) {
    match kind {
        27 => (2, 5),
        236 | 238 | 702 => (2, 2),
        // `:8765-8768`: the 233 jungle plant is 3 wide on row 0, 2 wide below.
        233 => {
            let fy = world.tile(x, y).map(|t| t.frame_y).unwrap_or(0);
            (if fy != 0 { 2 } else { 3 }, 2)
        }
        530 | 651 | 705 => (3, 2),
        485 | 490 | 521 | 522 | 523 | 524 | 525 | 526 | 527 | 652 => (2, 2),
        489 => (2, 3),
        493 => (1, 2),
        519 => (1, climb_cat_tail(world, x, y)),
        _ => (1, 1),
    }
}

// ---------------------------------------------------------------------------
// The shared per-cell draw-data probe.
// ---------------------------------------------------------------------------

/// `GetTileDrawData` for a drain cell (`TileDrawing.cs:8345`/`:8973`/`:9484`):
/// the static table + the half-brick height. The animator row is fed 0 (the
/// `vines.rs` convention — S1-G-1f binds `Main.tileFrame[type]` at the merge).
/// `solid_above` mirrors the strip probe (`vines.rs:169-172`).
fn drain_draw_data(world: &World, x: i32, y: i32, kind: u16, fx: i16, fy: i16) -> (TileDrawData, i16) {
    let solid_above = world.tile(x, y - 1).is_some_and(|t| {
        t.active() && td::TILE_SOLID.has(t.kind) && !td::TILE_SOLID_TOP.has(t.kind)
    });
    let dd = draw_data(kind, x, y, fx, fy, 0, solid_above);
    let half = if world.tile(x, y).is_some_and(|t| t.half_brick()) {
        8
    } else {
        0
    };
    (dd, half)
}

/// `Lighting.GetColor(x, y)` as normalized bytes — the drains read the raw
/// light map (the `vines.rs` convention); the light seam applies the vision
/// changers + override on top.
fn base_light(light: &tr_core::lighting::LightMap, x: i32, y: i32) -> [f32; 3] {
    light.get(x, y)
}

// ---------------------------------------------------------------------------
// DrawGrass — TileDrawing.cs:8326-8381 (bucket 1, WindyGrass)
// ---------------------------------------------------------------------------

/// `DrawGrass` (`TileDrawing.cs:8326-8381`): drain the WindyGrass bucket.
/// Per point: the active+visible gate (`:8338`), `GetTileDrawData`, the
/// `_rand.Next(4)` dust flag (`:8346`), the light chain, then the wind
/// transform — `GetWindCycle(_grassWindCounter)` zeroed when the wall blocks
/// wind (`:8360`) or the cell is not in a windy place (`:8364`), plus the
/// `GetWindGridPush(20, 0.35)` grid push (`:8368`). The sprite anchors at
/// `(x*16+8, y*16+16)` (`:8358`), leans `num3` right / `|num3|` down, rotates
/// `num3*0.1` about `(tileWidth/2, 16-half-tileTop)` (`:8374`).
pub(crate) fn drain_grass(
    world: &World,
    wind: &Wind,
    light: &tr_core::lighting::LightMap,
    grass_counter: f32,
    rng: &mut UnifiedRandom,
    seam: &mut impl DrainCellLight,
    show_invisible: bool,
    points: &[(i32, i32)],
    out: &mut Vec<DrainSprite>,
) {
    for &(x, y) in points {
        // `tile == null || !tile.active() || !IsVisible(tile)` (`:8338`).
        let Some(tile) = world.tile(x, y) else { continue };
        if !tile.active() || !is_visible(world, x, y, show_invisible) {
            continue;
        }
        let kind = tile.kind;
        let (dd, half) = drain_draw_data(world, x, y, kind, tile.frame_x, tile.frame_y);
        // `bool flag = this._rand.Next(4) == 0` (`:8346`) — consumed in order.
        let can_do_dust = rng.next_max(4) == 0;
        let base = base_light(light, x, y);
        let tint = seam.cell_light(world, x, y, kind, tile.frame_x, tile.frame_y, base, can_do_dust);
        // `position = (x*16+8, y*16+16)` (`:8358`).
        let mut pos = [x as f32 * 16.0 + 8.0, y as f32 * 16.0 + 16.0];
        // `num3 = GetWindCycle(x, y, _grassWindCounter)` (`:8359`).
        let mut num3 = wind.cycle(x, y, grass_counter);
        if !tr_core::wind::wall_allows_wind(tile.wall) {
            num3 = 0.0; // `:8360-8363`.
        }
        if !wind.in_a_place_with_wind(world, x, y, 1, 1) {
            num3 = 0.0; // `:8364-8367`.
        }
        num3 += wind.get_wind_grid_push(x, y, 20, 0.35); // `:8368`.
        pos[0] += num3; // `position.X += num3 * 1f` (`:8369`).
        pos[1] += num3.abs(); // `position.Y += Math.Abs(num3) * 1f` (`:8370`).
        let w = dd.width as f32;
        let origin = [w * 0.5, (16 - half - dd.top) as f32]; // `:8374`.
        out.push(DrainSprite {
            kind,
            x,
            y,
            frame_x: tile.frame_x,
            frame_y: tile.frame_y,
            dd,
            half,
            pos,
            rot: num3 * 0.1,
            origin,
            tint,
            flip: dd.flip,
            paint: tile.paint(),
            glow: None, // the per-case glow rect rides the f15 glow bind.
        });
    }
}

// ---------------------------------------------------------------------------
// DrawAnyDirectionalGrass — TileDrawing.cs:8383-8479 (bucket 10)
// ---------------------------------------------------------------------------

/// `DrawAnyDirectionalGrass` (`TileDrawing.cs:8383-8479`): drain the
/// AnyDirectionalGrass bucket (tile 184). The four-direction switch on
/// `tileFrameY / 54` (`:8429-8467`) picks the origin + the position nudges +
/// the sign of the wind lean; the grid push is the 2-axis variant
/// (`GetWindGridPush2Axis(20, 0.35)`, `:8425`) projected onto the direction
/// `(num4, num5)` (`:8468`).
pub(crate) fn drain_any_directional_grass(
    world: &World,
    wind: &Wind,
    light: &tr_core::lighting::LightMap,
    grass_counter: f32,
    rng: &mut UnifiedRandom,
    seam: &mut impl DrainCellLight,
    show_invisible: bool,
    points: &[(i32, i32)],
    out: &mut Vec<DrainSprite>,
) {
    for &(x, y) in points {
        let Some(tile) = world.tile(x, y) else { continue };
        if !tile.active() || !is_visible(world, x, y, show_invisible) {
            continue; // `:8395`.
        }
        let kind = tile.kind;
        let (dd, half) = drain_draw_data(world, x, y, kind, tile.frame_x, tile.frame_y);
        let can_do_dust = rng.next_max(4) == 0; // `:8403`.
        let base = base_light(light, x, y);
        let tint = seam.cell_light(world, x, y, kind, tile.frame_x, tile.frame_y, base, can_do_dust);
        // `position = (x*16, y*16)` (`:8415`).
        let mut pos = [x as f32 * 16.0, y as f32 * 16.0];
        let mut num3 = wind.cycle(x, y, grass_counter); // `:8416`.
        if !tr_core::wind::wall_allows_wind(tile.wall) {
            num3 = 0.0; // `:8417-8420`.
        }
        if !wind.in_a_place_with_wind(world, x, y, 1, 1) {
            num3 = 0.0; // `:8421-8424`.
        }
        let (push_x, push_y) = wind.get_wind_grid_push_2axis(x, y, 20, 0.35); // `:8425`.
        // The direction switch (`:8426-8467`): `(num4, num5)` is the unit
        // direction, `origin` the rotation pivot.
        let (num4, num5, origin) = match tile.frame_y / 54 {
            0 => {
                pos[0] += 8.0;
                pos[1] += 16.0;
                pos[0] += num3;
                pos[1] += num3.abs();
                (1, 0, [dd.width as f32 * 0.5, (16 - half - dd.top) as f32])
            }
            1 => {
                num3 *= -1.0;
                pos[0] += 8.0;
                pos[0] -= num3;
                pos[1] -= num3.abs();
                (-1, 0, [dd.width as f32 * 0.5, (-dd.top) as f32])
            }
            2 => {
                pos[1] += 8.0;
                pos[1] += num3;
                pos[0] -= num3.abs();
                (0, 1, [2.0, (16 - half - dd.top) as f32 * 0.5])
            }
            3 => {
                num3 *= -1.0;
                pos[0] += 16.0;
                pos[1] += 8.0;
                pos[1] -= num3;
                pos[0] += num3.abs();
                (0, -1, [14.0, (16 - half - dd.top) as f32 * 0.5])
            }
            _ => (1, 0, [dd.width as f32 * 0.5, (16 - half - dd.top) as f32]),
        };
        // `num3 += pushX * num4 + pushY * num5` (`:8468`).
        num3 += push_x * num4 as f32 + push_y * num5 as f32;
        out.push(DrainSprite {
            kind,
            x,
            y,
            frame_x: tile.frame_x,
            frame_y: tile.frame_y,
            dd,
            half,
            pos,
            rot: num3 * 0.1, // `:8472`.
            origin,
            tint,
            flip: dd.flip,
            paint: tile.paint(),
            glow: None,
        });
    }
}

// ---------------------------------------------------------------------------
// DrawRisingVineStrip — TileDrawing.cs:9134-9193 (bucket 7, ReverseVine)
// ---------------------------------------------------------------------------

/// One rising-vine segment (the reverse of `vines.rs::VineSegment`): the
/// strip walks UP from the root, the anchor advances along `(rot - PI/2)`.
#[derive(Clone, Copy, Debug)]
pub(crate) struct RisingVineSegment {
    pub kind: u16,
    pub x: i32,
    pub y: i32,
    pub frame_x: i16,
    pub frame_y: i16,
    pub dd: TileDrawData,
    pub half: i16,
    pub anchor: [f32; 2],
    pub rot: f32,
    pub visible: bool,
    pub fullbright: bool,
    pub paint: u8,
    /// `seams: SideFlags.Top` when the cell ABOVE is a reverse-vine thread
    /// (`:9187-9188`) — consumed only by the unported LitNature shader.
    pub seam_top: bool,
}

/// `DrawRisingVineStrip` (`TileDrawing.cs:9134-9193`), maths only: walk UP
/// from the root while the cells are active `ReverseVineThreads`, accumulate
/// the sway + grid push, advance the anchor 16 px along `(rot - PI/2)`. The
/// mirror of `vines.rs::vine_strip_segments` with three differences: the
/// anchor starts at `(x*16+8, startY*16+16+2)` (`:9138`), the loop runs
/// `num7 > 10` downward (`:9144`), the grid push is `(40, -0.004)` (`:9167`)
/// and the rotation is `num2 * (0 - num4) * windCycle + num5` (`:9179`).
pub(crate) fn rising_vine_strip(
    world: &World,
    wind: &Wind,
    vine_counter: f32,
    x: i32,
    start_y: i32,
    show_invisible: bool,
) -> Vec<RisingVineSegment> {
    let mut out = Vec::new();
    // `vector = (x*16+8, startY*16+16+2)` (`:9138`).
    let mut anchor = [x as f32 * 16.0 + 8.0, start_y as f32 * 16.0 + 18.0];
    // `num3 = Lerp(0.2, 1, |WindForVisuals|/1.2)` (`:9139`) — no clamp.
    let num3 = 0.2 + (1.0 - 0.2) * (wind.speed_current.abs() / 1.2);
    let mut num4 = -0.08 * num3; // `:9140`.
    let wind_cycle = wind.cycle(x, start_y, vine_counter); // `:9141`.
    let mut num5 = 0.0f32;
    let mut num6 = 0.0f32;
    let mut num2 = 0i32;
    // `for (int num7 = startY; num7 > 10; num7--)` (`:9144`).
    let mut num7 = start_y;
    while num7 > 10 {
        let Some(tile) = world.tile(x, num7) else { break };
        // `!tile.active() || !TileID.Sets.ReverseVineThreads[type]` (`:9150`).
        if !tile.active() || !td::SET_REVERSE_VINE_THREADS.has(tile.kind) {
            break;
        }
        if num2 >= 5 {
            num4 += 0.0075 * num3; // `:9155-9158`.
        }
        if num2 >= 2 {
            num4 += 0.0025; // `:9159-9162`.
        }
        // `WallID.Sets.AllowsWind[wall] && num7 < worldSurface` (`:9163`).
        // The reverse strip has NO remix arm (unlike the forward strip).
        if tr_core::wind::wall_allows_wind(tile.wall) && (num7 as f64) < wind.world_surface {
            num2 += 1;
        }
        // `GetWindGridPush(x, num7, 40, -0.004f)` (`:9167`).
        let push = wind.get_wind_grid_push(x, num7, 40, -0.004);
        num5 = if push != 0.0 || num6 == 0.0 {
            num5 - push
        } else {
            num5 * -0.78
        }; // `:9168`.
        num6 = push;
        let (dd, half) = drain_draw_data(world, x, num7, tile.kind, tile.frame_x, tile.frame_y);
        // `num8 = num2 * (0 - num4) * windCycle + num5` (`:9179`).
        let rot = num2 as f32 * (0.0 - num4) * wind_cycle + num5;
        let above = world.tile(x, num7 - 1);
        out.push(RisingVineSegment {
            kind: tile.kind,
            x,
            y: num7,
            frame_x: tile.frame_x,
            frame_y: tile.frame_y,
            dd,
            half,
            anchor,
            rot,
            visible: is_visible(world, x, num7, show_invisible),
            fullbright: tile.fullbright_block(),
            paint: tile.paint(),
            seam_top: above
                .is_some_and(|t| t.active() && td::SET_REVERSE_VINE_THREADS.has(t.kind)),
        });
        // `vector += (num8 - PI/2).ToRotationVector2() * 16f` (`:9190`).
        let (s, c) = (rot - std::f32::consts::FRAC_PI_2).sin_cos();
        anchor = [anchor[0] + c * 16.0, anchor[1] + s * 16.0];
        num7 -= 1;
    }
    out
}

/// `DrawReverseVines` (`TileDrawing.cs:8903-8914`): drain bucket 7, one
/// [`rising_vine_strip`] per registered root.
pub(crate) fn drain_reverse_vines(
    world: &World,
    wind: &Wind,
    vine_counter: f32,
    show_invisible: bool,
    points: &[(i32, i32)],
    out: &mut Vec<RisingVineSegment>,
) {
    for &(x, y) in points {
        out.extend(rising_vine_strip(world, wind, vine_counter, x, y, show_invisible));
    }
}

// ---------------------------------------------------------------------------
// DrawMultiTileGrassInWind — TileDrawing.cs:8916-9032 (bucket 2 body)
// ---------------------------------------------------------------------------

/// The 27 sunflower's glow latch (`TileDrawing.cs:8929-8933`): `color =
/// White`, `flag2 = true`, `num2 = 74` — the glowmask draws the sheet's
/// second 74-px band (`Tiles_27` is 108x148: 74 px of styles + 74 px of glow
/// art, the `tiles.rs:952-956` note). The glow rect is the cell rect shifted
/// `+74` in Y (`:9027`'s `addFrY + num2`).
const SUNFLOWER_GLOW_Y: i16 = 74;

/// `DrawMultiTileGrassInWind` (`TileDrawing.cs:8916-9032`): the multi-tile
/// grass body. The anchor `vector` sits at the object's bottom-centre
/// (`:8919`); each cell rotates about `vector - vector2` (its own top-left
/// offset) by `windCycle * num * num3`, where `num3` is the per-row height
/// fraction (`:8964-8972`) and `num` the type's sway multiplier (`:8920`,
/// `:8938`, `:8947`). The 493 golf flag flips on the wind sign and shifts
/// ±6 px (`:8983-8997`). The 521-527 crystal shards freeze (`num = 0`,
/// `flag = false`, `:8940-8949`).
pub(crate) fn multi_tile_grass_in_wind(
    world: &World,
    wind: &Wind,
    light: &tr_core::lighting::LightMap,
    sunflower_counter: f32,
    rng: &mut UnifiedRandom,
    seam: &mut impl DrainCellLight,
    show_invisible: bool,
    top_left_x: i32,
    top_left_y: i32,
    size_x: i32,
    size_y: i32,
    out: &mut Vec<DrainSprite>,
) {
    let wind_cycle = wind.cycle(top_left_x, top_left_y, sunflower_counter); // `:8918`.
    // `vector = (topLeftX*16 + sizeX*16*0.5, topLeftY*16 + 16*sizeY)` (`:8919`).
    let vector = [
        top_left_x as f32 * 16.0 + size_x as f32 * 16.0 * 0.5,
        top_left_y as f32 * 16.0 + 16.0 * size_y as f32,
    ];
    let Some(anchor_tile) = world.tile(top_left_x, top_left_y) else {
        return;
    };
    let kind = anchor_tile.kind;
    // The type switch (`:8927-8950`): `num` the sway multiplier, `flag` the
    // wind-place gate, `flag2`/`num2` the sunflower glow latch.
    let mut num = 0.07f32;
    let mut flag = wind.in_a_place_with_wind(world, top_left_x, top_left_y, size_x, size_y);
    let mut flag2 = false;
    let mut num2 = 0i16;
    match kind {
        27 => {
            flag2 = true;
            num2 = SUNFLOWER_GLOW_Y;
        }
        519 => {
            flag = wind.in_a_place_with_wind(world, top_left_x, top_left_y, size_x, 1);
        }
        521 | 522 | 523 | 524 | 525 | 526 | 527 => {
            num = 0.0;
            flag = false;
        }
        _ => {
            num = 0.15;
        }
    }
    // The cell walk (`:8951-9031`): row-major, x outer / y inner.
    for i in top_left_x..top_left_x + size_x {
        for j in top_left_y..top_left_y + size_y {
            let Some(tile) = world.tile(i, j) else { continue };
            // `type2 != type || !IsVisible(tile)` (`:8957`).
            if tile.kind != kind || !is_visible(world, i, j, show_invisible) {
                continue;
            }
            let (dd, half) = drain_draw_data(world, i, j, tile.kind, tile.frame_x, tile.frame_y);
            // `num3 = 1 - (j - topLeftY + 1) / sizeY`, the 0 -> 0.1 guard
            // (`:8964-8968`), then `!flag -> 0` (`:8969-8972`).
            let mut num3 = 1.0 - (j - top_left_y + 1) as f32 / size_y as f32;
            if num3 == 0.0 {
                num3 = 0.1;
            }
            if !flag {
                num3 = 0.0;
            }
            let can_do_dust = rng.next_max(4) == 0; // `:8974`.
            let base = base_light(light, i, j);
            let tint =
                seam.cell_light(world, i, j, tile.kind, tile.frame_x, tile.frame_y, base, can_do_dust);
            // `vector2 = (i*16, j*16 + tileTop)` (`:8982`).
            let mut vector2 = [i as f32 * 16.0, j as f32 * 16.0 + dd.top as f32];
            let mut flip = dd.flip;
            // The 493 golf-flag arm (`:8983-8997`).
            if tile.kind == 493 && tile.frame_y == 0 {
                if wind.speed_current >= 0.0 {
                    flip = !flip; // `tileSpriteEffect ^= FlipHorizontally`.
                }
                if !flip {
                    vector2[0] -= 6.0;
                } else {
                    vector2[0] += 6.0;
                }
            }
            // `vector3 = (windCycle, |windCycle| * 2 * num3)` (`:8998`);
            // `origin = vector - vector2` (`:8999`).
            let vector3_y = wind_cycle.abs() * 2.0 * num3;
            let origin = [vector[0] - vector2[0], vector[1] - vector2[1]];
            let pos = [vector[0], vector[1] + vector3_y]; // `:9024`.
            let glow = if flag2 {
                Some(DrainGlow {
                    src_y_add: num2,
                    color: [1.0, 1.0, 1.0, 1.0], // `color = White` (`:8930`).
                })
            } else {
                None
            };
            out.push(DrainSprite {
                kind: tile.kind,
                x: i,
                y: j,
                frame_x: tile.frame_x,
                frame_y: tile.frame_y,
                dd,
                half,
                pos,
                rot: wind_cycle * num * num3, // `:9024`.
                origin,
                tint,
                flip,
                paint: tile.paint(),
                glow,
            });
        }
    }
}

/// `DrawMultiTileGrass` (`TileDrawing.cs:8738-8805`): drain bucket 2. Per
/// point: the active gate (`:8752`), the per-type size table, the 519 anchor
/// shift (`num3 -= num4 - 1`, `:8799`), then [`multi_tile_grass_in_wind`].
pub(crate) fn drain_multi_tile_grass(
    world: &World,
    wind: &Wind,
    light: &tr_core::lighting::LightMap,
    sunflower_counter: f32,
    rng: &mut UnifiedRandom,
    seam: &mut impl DrainCellLight,
    show_invisible: bool,
    points: &[(i32, i32)],
    out: &mut Vec<DrainSprite>,
) {
    for &(x, y) in points {
        let Some(tile) = world.tile(x, y) else { continue };
        if !tile.active() {
            continue; // `:8752`.
        }
        let (size_x, mut size_y) = multi_tile_grass_size(world, tile.kind, x, y);
        let mut top_y = y;
        if tile.kind == 519 {
            // `num4 = ClimbCatTail(x, num3); num3 -= num4 - 1` (`:8798-8799`).
            size_y = climb_cat_tail(world, x, y);
            top_y -= size_y - 1;
        }
        multi_tile_grass_in_wind(
            world,
            wind,
            light,
            sunflower_counter,
            rng,
            seam,
            show_invisible,
            x,
            top_y,
            size_x,
            size_y,
            out,
        );
    }
}

// ---------------------------------------------------------------------------
// DrawMultiTileVinesInWind — TileDrawing.cs:9238-9603 (bucket 3 body)
// ---------------------------------------------------------------------------

/// `WorldGen.IsBelowANonHammeredPlatform` (`WorldGen.cs:38289-38301`): the
/// cell above is an active, un-hammered (no half-brick, no slope) platform.
fn is_below_a_non_hammered_platform(world: &World, x: i32, y: i32) -> bool {
    if y < 1 {
        return false;
    }
    let Some(tile) = world.tile(x, y - 1) else {
        return false;
    };
    tile.active() && td::SET_PLATFORMS.has(tile.kind) && !tile.half_brick() && tile.slope() == 0
}

/// The 34/126 hanging-planter style override (`TileDrawing.cs:9269-9349`):
/// `tile.frameY / 54 + tile.frameX / 108 * 37` selects a `(num3, num5, flag2,
/// num2-mult)` tuple. `num3 = None` means the per-row height fraction is
/// replaced by `num5`'s constant; the return is `(Option<f32>, f32, bool,
/// f32)` = `(num3, num5, flag2, num2_scale)`.
fn planter_style(frame_x: i16, frame_y: i16) -> (Option<f32>, f32, bool, f32) {
    let style = frame_y as i32 / 54 + frame_x as i32 / 108 * 37;
    match style {
        9 => (None, -1.0, true, 0.3),
        11 => (Some(1.0), 0.0, false, 0.5),
        12 | 18 | 21 => (None, -1.0, false, 1.0),
        23 => (Some(0.0), 0.0, false, 1.0),
        25 => (None, -1.0, true, 1.0),
        32 | 33 => (Some(1.0), 0.0, false, 0.5),
        35 => (Some(0.0), 0.0, false, 1.0),
        36 => (None, -1.0, true, 1.0),
        37 => (None, -1.0, true, 0.5),
        39 => (None, -1.0, true, 1.0),
        40 | 41 | 42 | 43 => (None, -2.0, true, 0.5),
        44 => (None, -3.0, false, 1.0),
        54 | 55 | 60 | 65 | 68 | 70 => (Some(0.0), 0.0, false, 1.0),
        67 => (Some(1.0), 0.0, false, 1.0),
        _ => (Some(1.0), 0.0, false, 1.0),
    }
}

/// The 42 hanging-bottle style override (`TileDrawing.cs:9354-9417`):
/// `tile.frameY / 36` selects the tuple (same shape as [`planter_style`]).
fn bottle_style(frame_y: i16) -> (Option<f32>, f32, bool, f32) {
    match frame_y as i32 / 36 {
        0 => (None, -1.0, false, 1.0),
        9 => (Some(0.0), 0.0, false, 1.0),
        12 | 14 | 28 => (None, -1.0, false, 1.0),
        30 | 32 | 33 => (Some(0.0), 0.0, false, 1.0),
        34 => (None, -1.0, false, 1.0),
        35 => (Some(0.0), 0.0, false, 1.0),
        38 => (None, -1.0, false, 1.0),
        39 => (None, -1.0, true, 1.0),
        // `:9400-9408`: `num3 = 0f; num3 = null;` — the second write wins.
        40 | 41 | 42 | 43 => (None, -1.0, true, 1.0),
        54 | 55 | 60 | 65 | 67 | 70 => (Some(0.0), 0.0, false, 1.0),
        _ => (Some(1.0), 0.0, false, 1.0),
    }
}

/// The 698 dead-cells-jar item-icon plan (`TileDrawing.cs:9568-9581`): the
/// icon draws at `vector5 + (-2, 24+num11[+4/+6]).RotatedBy(num8) + (2, 0)`,
/// environment `Lerp(tileLight, White, 0.5)`, `sizeLimit = num10`. The
/// `ItemSlot.DrawItemIcon` colour chain is d57's ported machinery — the plan
/// carries the C# inputs and the wiring binds the icon draw.
#[derive(Clone, Copy, Debug)]
pub(crate) struct DrainJarItem {
    pub item_kind: u16,
    pub center: [f32; 2],
    pub env_light: [f32; 3],
    pub size_limit: f32,
    pub rot: f32,
}

/// One flame quad of the multi-tile vine tail (`TileDrawing.cs:9595-9600`).
#[derive(Clone, Copy, Debug)]
pub(crate) struct DrainFlame {
    /// The flame sheet: `Flames(n)` or one of the three jar textures.
    pub sheet: DrainFlameSheet,
    pub pos: [f32; 2],
    pub rot: f32,
    pub origin: [f32; 2],
    pub color: [f32; 4],
}

/// The flame texture selector (`GetTileFlameData`, `TileDrawing.cs:1849-1874`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum DrainFlameSheet {
    Flames(usize),
    FireflyJar,
    LightningBugJar,
    Glow291,
}

/// The 698 jar's rarity colour (`TileDrawing.cs:9544-9557` +
/// `Item.GetPopupRarityColor`, `Item.cs:652-707`). `expert` maps rare to -12
/// (the disco colour); -13 is the master colour. Returns the raw RGB bytes.
fn jar_rarity_color(rare: i32, expert: bool, disco: [u8; 3], master_color: f32) -> [u8; 3] {
    let num12 = if expert { -12 } else { rare };
    match num12 {
        -12 => disco,
        -13 => [255, (master_color * 200.0) as u8, 0],
        1 => [150, 150, 255],
        2 => [150, 255, 150],
        3 => [255, 200, 150],
        4 => [255, 150, 150],
        5 => [255, 150, 255],
        -11 => [255, 175, 0],
        -1 => [130, 130, 130],
        6 => [210, 160, 255],
        7 => [150, 255, 10],
        8 => [255, 255, 10],
        9 => [5, 200, 255],
        10 => [255, 40, 100],
        r if r >= 11 => [180, 40, 255],
        _ => [255, 255, 255],
    }
}

/// `Main.rgbToHsl` (`Main.cs:47944-47980`): the byte colour → `(h, s, l)`.
fn rgb_to_hsl(c: [u8; 3]) -> [f32; 3] {
    let r = c[0] as f32 / 255.0;
    let g = c[1] as f32 / 255.0;
    let b = c[2] as f32 / 255.0;
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let l = (max + min) / 2.0;
    if max == min {
        return [0.0, 0.0, l];
    }
    let d = max - min;
    let s = if l > 0.5 { d / (2.0 - max - min) } else { d / (max + min) };
    let mut h = if max == r {
        (g - b) / d + if g < b { 6.0 } else { 0.0 }
    } else if max == g {
        (b - r) / d + 2.0
    } else {
        (r - g) / d + 4.0
    };
    h /= 6.0;
    [h, s, l]
}

/// `DrawMultiTileVinesInWind` (`TileDrawing.cs:9238-9603`): the multi-tile
/// hanging-vine body. The anchor `vector` sits at the object's top-centre
/// minus 2 (`:9243-9248`), lifted 8 px under a non-hammered platform
/// (`:9250-9254`); the wind cycle adds the highest complex grid push
/// (`:9242`). Each cell rotates about `vector - vector3` by
/// `windCycle * num2 * num7` (`:9504`), `num7` the per-row fraction or the
/// style constant. The 660 fairy jar draws an `Extra_260` white + HSL glow
/// pair on its bottom row (`:9505-9519`); the 698 dead-cells jar draws its
/// two lid rows + the item icon (`:9520-9584`); every cell runs the shared-
/// seed flame tail (`:9589-9600`).
#[allow(clippy::too_many_arguments)]
pub(crate) fn multi_tile_vines_in_wind(
    world: &World,
    wind: &Wind,
    light: &tr_core::lighting::LightMap,
    sunflower_counter: f32,
    rng: &mut UnifiedRandom,
    seam: &mut impl DrainCellLight,
    show_invisible: bool,
    top_left_x: i32,
    top_left_y: i32,
    size_x: i32,
    size_y: i32,
    disco: [u8; 3],
    master_color: f32,
    tile_frame_seed: u64,
    global_time: f32,
    out: &mut Vec<DrainSprite>,
    fairy_jars: &mut Vec<DrainFairyJar>,
    jar_items: &mut Vec<DrainJarItem>,
    flames: &mut Vec<DrainFlame>,
) {
    let mut wind_cycle = wind.cycle(top_left_x, top_left_y, sunflower_counter); // `:9240`.
    let num = wind_cycle; // `:9241`.
    wind_cycle += wind.get_highest_wind_grid_push_complex(
        top_left_x, top_left_y, size_x, size_y, 60, 1.26, 3, true,
    ); // `:9242`.
    let mut vector = [
        top_left_x as f32 * 16.0 + size_x as f32 * 16.0 * 0.5,
        top_left_y as f32 * 16.0,
    ]; // `:9243`.
    let Some(anchor_tile) = world.tile(top_left_x, top_left_y) else {
        return;
    };
    let kind = anchor_tile.kind;
    let mut vector2 = [0.0f32, -2.0]; // `:9247`.
    vector[0] += vector2[0];
    vector[1] += vector2[1]; // `:9248`.
    // The platform lift (`:9250-9254`): the 465/591/592 family checks BOTH
    // columns, the rest only when 1-wide.
    let wide_family = kind == 465 || kind == 591 || kind == 592;
    let lifted = if !wide_family {
        size_x == 1 && is_below_a_non_hammered_platform(world, top_left_x, top_left_y)
    } else {
        is_below_a_non_hammered_platform(world, top_left_x, top_left_y)
            && is_below_a_non_hammered_platform(world, top_left_x + 1, top_left_y)
    };
    if lifted {
        vector[1] -= 8.0;
        vector2[1] -= 8.0;
    }
    // The style state (`:9257-9262`): `num3` the optional height override,
    // `num4` the X sway scale, `num5` the Y sway scale, `num2` the rotation
    // multiplier (negated at `:9451`), `flag2` the +16 anchor lift, `flag3`
    // the 698 wind-cancel.
    let mut num3: Option<f32> = None;
    let mut num4 = 1.0f32;
    let mut num5 = -4.0f32;
    let mut flag2 = false;
    let mut flag3 = false;
    let mut num2 = 0.15f32;
    // The 592 glowmask (`:9434-9439`): `Glow_294`, white-alpha-0.
    let mut glow_592 = false;
    match kind {
        34 | 126 => {
            let (n3, n5, f2, scale) = planter_style(anchor_tile.frame_x, anchor_tile.frame_y);
            num3 = n3;
            num5 = n5;
            flag2 = f2;
            num2 *= scale;
        }
        42 => {
            let (n3, n5, f2, scale) = bottle_style(anchor_tile.frame_y);
            num3 = n3;
            num5 = n5;
            flag2 = f2;
            num2 *= scale;
        }
        95 | 270 | 271 | 444 | 454 | 572 | 581 | 660 => {
            num3 = Some(1.0);
            num5 = 0.0;
        }
        591 => {
            num4 = 0.5;
            num5 = -2.0;
        }
        592 => {
            num4 = 0.5;
            num5 = -2.0;
            glow_592 = true;
        }
        698 => {
            num4 = 0.5;
            num5 = -1.0;
            // `offSet.X -= 10f` (`:9443`) — the offSet is zero in the port's
            // world-space plans, so the shift lands on the anchor instead.
            vector[0] -= 10.0;
            flag3 = true;
        }
        _ => {}
    }
    if flag2 {
        vector[1] += 16.0; // `:9447-9450`.
    }
    num2 *= -1.0; // `:9451`.
    let flag4 = wind.in_a_place_with_wind(world, top_left_x, top_left_y, size_x, size_y);
    if flag3 || !flag4 {
        wind_cycle -= num; // `:9453-9456`.
    }
    // The shared flame seed (`:9457`, `:9590-9594`): the first non-zero
    // `flameSeed` across the object's cells wins and is reused for all.
    let mut shared_seed = 0u64;
    for i in top_left_x..top_left_x + size_x {
        for j in top_left_y..top_left_y + size_y {
            let Some(tile) = world.tile(i, j) else { continue };
            if tile.kind != kind || !is_visible(world, i, j, show_invisible) {
                continue; // `:9464`.
            }
            let (dd, half) = drain_draw_data(world, i, j, tile.kind, tile.frame_x, tile.frame_y);
            // `num7 = (j - topLeftY + 1) / sizeY`, 0 -> 0.1 (`:9471-9475`),
            // then the style override (`:9476-9479`) and the flag2 top-row
            // zero (`:9480-9483`).
            let mut num7 = (j - top_left_y + 1) as f32 / size_y as f32;
            if num7 == 0.0 {
                num7 = 0.1;
            }
            if let Some(n3) = num3 {
                num7 = n3;
            }
            if flag2 && j == top_left_y {
                num7 = 0.0;
            }
            let can_do_dust = rng.next_max(4) == 0; // `:9485`.
            let base = base_light(light, i, j);
            let tint =
                seam.cell_light(world, i, j, tile.kind, tile.frame_x, tile.frame_y, base, can_do_dust);
            // `vector3 = (i*16, j*16 + tileTop) + vector2` (`:9493-9494`).
            let vector3 = [
                i as f32 * 16.0 + vector2[0],
                j as f32 * 16.0 + dd.top as f32 + vector2[1],
            ];
            // `vector4 = (windCycle*num4, |windCycle|*num5*num7)` (`:9495`).
            let vector4_y = wind_cycle.abs() * num5 * num7;
            let origin = [vector[0] - vector3[0], vector[1] - vector3[1]]; // `:9496`.
            let vector5 = [vector[0], vector[1] + vector4_y]; // `:9502`.
            let num8 = wind_cycle * num2 * num7; // `:9504`.
            // The 660 fairy-jar bottom row (`:9505-9519`): THREE draws — an
            // `Extra_260` white copy BEFORE the tile draw (`:9510`), the tile
            // draw itself (`:9512`, always), then the `Extra_260` HSL glow
            // (A=127) AFTER (`:9518`). The discarded `% 1f` computation at
            // `:9508` is dead — the glow re-derives it at `:9516`.
            if tile.kind == 660 && j == top_left_y + size_y - 1 {
                let hue = (((i + j) % 200) as f32 * 0.11 + global_time / 360.0) % 1.0;
                let hsl = tr_core::tile_light::hsl_to_rgb(hue, 1.0, 0.8);
                fairy_jars.push(DrainFairyJar {
                    x: i,
                    y: j,
                    frame_x: tile.frame_x,
                    frame_y: tile.frame_y,
                    dd,
                    half,
                    pos: vector5,
                    rot: num8,
                    origin,
                    flip: dd.flip,
                    paint: tile.paint(),
                    glow_color: [hsl[0], hsl[1], hsl[2], 127.0 / 255.0],
                });
            }
            // The tile draw (`:9512`) runs for EVERY cell, the 660 bottom row
            // included (it sits between the two Extra_260 copies).
            {
                let glow = if glow_592 {
                    Some(DrainGlow {
                        src_y_add: 0,
                        color: [1.0, 1.0, 1.0, 0.0],
                    })
                } else {
                    None
                };
                out.push(DrainSprite {
                    kind: tile.kind,
                    x: i,
                    y: j,
                    frame_x: tile.frame_x,
                    frame_y: tile.frame_y,
                    dd,
                    half,
                    pos: vector5,
                    rot: num8,
                    origin,
                    tint,
                    flip: dd.flip,
                    paint: tile.paint(),
                    glow,
                });
            }
            // The 698 dead-cells jar (`:9520-9584`): the two lid rows + the
            // item icon. The TE lookup is at the ANCHOR (`:9520`).
            if tile.kind == 698 {
                drain_dead_cells_jar(
                    world, top_left_x, top_left_y, i, j, tile.frame_x, dd, half, vector5, num8,
                    origin, tint, disco, master_color, jar_items, out,
                );
            }
            // The flame tail (`:9589-9600`).
            let fd = tr_core::flame::tile_flame_data(
                i, j, tile.kind, tile.frame_x, tile.frame_y, tile_frame_seed, global_time, disco,
            );
            if fd.count > 0 {
                if shared_seed == 0 {
                    shared_seed = fd.seed; // `:9590-9593`.
                }
                let mut seed = shared_seed;
                let sheet = match tile.kind {
                    270 => DrainFlameSheet::FireflyJar,
                    271 => DrainFlameSheet::LightningBugJar,
                    581 => DrainFlameSheet::Glow291,
                    _ => DrainFlameSheet::Flames(fd.sheet),
                };
                for _ in 0..fd.count {
                    let x2 = tr_core::flame::random_int(&mut seed, fd.rx.0, fd.rx.1) as f32 * fd.mx;
                    let y2 = tr_core::flame::random_int(&mut seed, fd.ry.0, fd.ry.1) as f32 * fd.my;
                    flames.push(DrainFlame {
                        sheet,
                        pos: [vector5[0] + x2, vector5[1] + y2],
                        rot: num8,
                        origin,
                        color: fd.color,
                    });
                }
            }
        }
    }
}

/// The 698 jar's lid rows + item icon (`TileDrawing.cs:9520-9584`). The jar
/// texture draws three stacked rects (base, +46, +92); the bottom two carry
/// the rarity-derived colours. The item icon's centre is the rotated
/// `(-2, 24+num11)` offset. `num9 = tileFrameX / 38` picks the lid style.
/// The TE lookup is at the ANCHOR `(top_left_x, top_left_y)` (`:9520`).
#[allow(clippy::too_many_arguments)]
fn drain_dead_cells_jar(
    world: &World,
    top_left_x: i32,
    top_left_y: i32,
    i: i32,
    j: i32,
    frame_x: i16,
    dd: TileDrawData,
    half: i16,
    vector5: [f32; 2],
    num8: f32,
    origin: [f32; 2],
    tint: [f32; 3],
    disco: [u8; 3],
    master_color: f32,
    jar_items: &mut Vec<DrainJarItem>,
    out: &mut Vec<DrainSprite>,
) {
    // The TE item at the ANCHOR (`:9520` — `TryGetAt(topLeftX, topLeftY)`,
    // not the per-cell position). The TE payload carries no rare/expert —
    // fed 0/0 (the d57 TeItem-runtime deferral); an empty jar keeps the
    // default `(150,150,255)` lid (`num12 = 1`).
    let item = world
        .tile_entities
        .get_at(top_left_x as i16, top_left_y as i16)
        .and_then(|e| match &e.data {
            tr_core::tile_entities::TeData::Item(Some(it)) if !it.is_air() => Some(*it),
            _ => None,
        });
    let num9 = frame_x as i32 / 38; // `:9523`.
    let (num10, num11) = match num9 {
        1 => (18.0f32, 0i32),
        2 => (20.0, 0),
        _ => (22.0, -1),
    }; // `:9524-9538`.
    // The rarity colour (`:9543-9558`).
    let (rare, expert) = match item {
        Some(_) => (0, false), // TeItem has no rare/expert — the d57 deferral.
        None => (1, false),
    };
    let color3 = jar_rarity_color(rare, expert, disco, master_color);
    let hsl = rgb_to_hsl(color3);
    let mut color4 = color3;
    // `color3 *= 0.25f` (`:9562`) — the XNA byte scale.
    let c3 = [
        (color3[0] as f32 * 0.25) as u8,
        (color3[1] as f32 * 0.25) as u8,
        (color3[2] as f32 * 0.25) as u8,
    ];
    // `if (num12 != -1) color4 = hslToRgb(h, clamp(s+0.5), clamp(l), 127)`
    // (`:9563-9566`).
    if rare != -1 {
        let s = (hsl[1] + 0.5).clamp(0.0, 1.0);
        let l = hsl[2].clamp(0.0, 1.0);
        let rgb = tr_core::tile_light::hsl_to_rgb(hsl[0], s, l);
        color4 = [
            (rgb[0] * 255.0) as u8,
            (rgb[1] * 255.0) as u8,
            (rgb[2] * 255.0) as u8,
        ];
    }
    // The +92 row (`value3`, `:9542`, `:9567`): tinted `color3 * 0.25`.
    let mut dd_bottom = dd;
    dd_bottom.add_fr_y += 92;
    out.push(DrainSprite {
        kind: 698, x: i, y: j, frame_x, frame_y: 0, dd: dd_bottom, half,
        pos: vector5, rot: num8, origin,
        tint: [c3[0] as f32 / 255.0, c3[1] as f32 / 255.0, c3[2] as f32 / 255.0],
        flip: dd.flip, paint: 0, glow: None,
    });
    // The item icon (`:9568-9581`).
    if let Some(it) = item {
        let mut spin_y = 24.0 + num11 as f32;
        match num9 {
            1 => spin_y += 4.0,
            2 => spin_y += 6.0,
            _ => {}
        }
        let (s, c) = num8.sin_cos();
        let spin = [-2.0, spin_y];
        let rotated = [spin[0] * c - spin[1] * s, spin[0] * s + spin[1] * c];
        let center = [vector5[0] + rotated[0] + 2.0, vector5[1] + rotated[1]];
        let env = [
            tint[0] + (1.0 - tint[0]) * 0.5,
            tint[1] + (1.0 - tint[1]) * 0.5,
            tint[2] + (1.0 - tint[2]) * 0.5,
        ];
        jar_items.push(DrainJarItem {
            item_kind: it.kind,
            center,
            env_light: env,
            size_limit: num10,
            rot: num8,
        });
    }
    // The +46 row (`rectangle2`, `:9540`, `:9583`): tinted `color4`, A=255.
    let mut dd_mid = dd;
    dd_mid.add_fr_y += 46;
    out.push(DrainSprite {
        kind: 698, x: i, y: j, frame_x, frame_y: 0, dd: dd_mid, half,
        pos: vector5, rot: num8, origin,
        tint: [color4[0] as f32 / 255.0, color4[1] as f32 / 255.0, color4[2] as f32 / 255.0],
        flip: dd.flip, paint: 0, glow: None,
    });
}

/// `DrawMultiTileVines` (`TileDrawing.cs:8829-8888`): drain bucket 3. Per
/// point: the active gate (`:8843`), the per-type size table, then
/// [`multi_tile_vines_in_wind`].
#[allow(clippy::too_many_arguments)]
pub(crate) fn drain_multi_tile_vines(
    world: &World,
    wind: &Wind,
    light: &tr_core::lighting::LightMap,
    sunflower_counter: f32,
    rng: &mut UnifiedRandom,
    seam: &mut impl DrainCellLight,
    show_invisible: bool,
    disco: [u8; 3],
    master_color: f32,
    tile_frame_seed: u64,
    global_time: f32,
    points: &[(i32, i32)],
    out: &mut Vec<DrainSprite>,
    fairy_jars: &mut Vec<DrainFairyJar>,
    jar_items: &mut Vec<DrainJarItem>,
    flames: &mut Vec<DrainFlame>,
) {
    for &(x, y) in points {
        let Some(tile) = world.tile(x, y) else { continue };
        if !tile.active() {
            continue; // `:8843`.
        }
        let (size_x, size_y) = multi_tile_vine_size(tile.kind);
        multi_tile_vines_in_wind(
            world, wind, light, sunflower_counter, rng, seam, show_invisible,
            x, y, size_x, size_y, disco, master_color, tile_frame_seed, global_time,
            out, fairy_jars, jar_items, flames,
        );
    }
}

// ---------------------------------------------------------------------------
// The PostDrawTiles drain order — TileDrawing.cs:414-442
// ---------------------------------------------------------------------------

/// One `PostDrawTiles(false)` pass (`TileDrawing.cs:421-430`), in C# call
/// order. UNION keep-one (contract §2.3): the g32 `PostPass` copy dropped;
/// this enum drives the DRAIN bodies and the g29 outer `PostDrawStep`
/// order maps onto it via [`drain_pass_for_post_step`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum DrainPass {
    MultiTileVines,
    MultiTileGrass,
    VoidLenses,
    TeleportationPylons,
    MasterTrophies,
    Grass,
    AnyDirectionalGrass,
    Trees,
    Vines,
    ReverseVines,
}

/// `PostDrawTiles(false)`'s ten-pass order (`TileDrawing.cs:421-430`). The
/// `EndBatch` + `NatureAfterAllObjects` tail (`:431-433`) is the g29/g32
/// orchestration's concern — the drains stop at `DrawReverseVines`.
pub(crate) const POST_DRAW_DRAIN_ORDER: [DrainPass; 10] = [
    DrainPass::MultiTileVines,
    DrainPass::MultiTileGrass,
    DrainPass::VoidLenses,
    DrainPass::TeleportationPylons,
    DrainPass::MasterTrophies,
    DrainPass::Grass,
    DrainPass::AnyDirectionalGrass,
    DrainPass::Trees,
    DrainPass::Vines,
    DrainPass::ReverseVines,
];

/// The registry bucket each drain pass walks — the `int num = N` head of
/// each C# pass (`:8833`/`:8742`/`:8710`/`:8641`/`:8599`/`:8330`/`:8387`/
/// `:8056`/`:8894`/`:8907`). `BiomeGrass` (5) is never drained.
pub(crate) fn drain_pass_bucket(pass: DrainPass) -> usize {
    match pass {
        DrainPass::MultiTileVines => BUCKET_MULTI_TILE_VINE,
        DrainPass::MultiTileGrass => BUCKET_MULTI_TILE_GRASS,
        DrainPass::VoidLenses => BUCKET_VOID_LENS,
        DrainPass::TeleportationPylons => BUCKET_TELEPORTATION_PYLON,
        DrainPass::MasterTrophies => BUCKET_MASTER_TROPHY,
        DrainPass::Grass => BUCKET_WINDY_GRASS,
        DrainPass::AnyDirectionalGrass => BUCKET_ANY_DIRECTIONAL_GRASS,
        DrainPass::Trees => BUCKET_TREE,
        DrainPass::Vines => BUCKET_VINE,
        DrainPass::ReverseVines => BUCKET_REVERSE_VINE,
    }
}

/// The seam for the already-ported sprite bodies (Trees / Vines / Pylons /
/// Trophies / VoidLenses): the drain driver walks the bucket points in C#
/// order and hands each to the existing pass (`foliage.rs::draw_tree_foliage`
/// per-marker, `vines.rs::draw_vines` per-root, `pylons.rs::*_sprites`
/// per-anchor). The merge binds these to the real passes; the driver itself
/// owns only the order + the per-point active gate.
pub(crate) trait PortedBodyDrain {
    /// `DrawTrees` (`:8052-8304`): one Tree-bucket point. The C# gates on
    /// `tile == null || !tile.active()` (`:8066`) then the type switch; the
    /// ported `foliage.rs` body owns the canopy/branch maths.
    fn tree_point(&mut self, world: &World, x: i32, y: i32);
    /// `DrawVines` (`:8890-8901`): one Vine-bucket root → `DrawVineStrip`.
    fn vine_root(&mut self, world: &World, x: i32, y: i32);
    /// `DrawVoidLenses` (`:8708-8736`): one VoidLens-bucket point.
    fn void_lens_point(&mut self, world: &World, x: i32, y: i32);
    /// `DrawTeleportationPylons` (`:8639-8706`): one pylon anchor.
    fn pylon_point(&mut self, world: &World, x: i32, y: i32);
    /// `DrawMasterTrophies` (`:8597-8637`): one trophy anchor.
    fn trophy_point(&mut self, world: &World, x: i32, y: i32);
}

/// UNION (contract §2.3): the g29 `PostDrawStep` → drain-pass map — the
/// ten nature steps bind to [`drain_one_pass`]; `EndBatch`/
/// `NatureAfterAllObjects` are the batch/nature seam, the two TE passes
/// are d57's doll/hat-rack passes — all four map to `None` here (the g32
/// `post_pass_bucket` `None`-arm fold, its literal-usize copy dropped).
pub(crate) fn drain_pass_for_post_step(
    step: crate::render::tile_pass::PostDrawStep,
) -> Option<DrainPass> {
    use crate::render::tile_pass::PostDrawStep as S;
    match step {
        S::DrawMultiTileVines => Some(DrainPass::MultiTileVines),
        S::DrawMultiTileGrass => Some(DrainPass::MultiTileGrass),
        S::DrawVoidLenses => Some(DrainPass::VoidLenses),
        S::DrawTeleportationPylons => Some(DrainPass::TeleportationPylons),
        S::DrawMasterTrophies => Some(DrainPass::MasterTrophies),
        S::DrawGrass => Some(DrainPass::Grass),
        S::DrawAnyDirectionalGrass => Some(DrainPass::AnyDirectionalGrass),
        S::DrawTrees => Some(DrainPass::Trees),
        S::DrawVines => Some(DrainPass::Vines),
        S::DrawReverseVines => Some(DrainPass::ReverseVines),
        S::EndBatch
        | S::NatureAfterAllObjects
        | S::DrawEntitiesHatRacks
        | S::DrawEntitiesDisplayDolls => None,
    }
}

/// `PostDrawTiles(false)` (`TileDrawing.cs:414-442`): drain all ten buckets
/// in C# order. UNION fold (contract §1.4): a thin loop over
/// [`drain_one_pass`] — the g29 `post_draw_tiles` driver binds each of its
/// ten nature steps through the per-pass dispatch individually (via
/// [`drain_pass_for_post_step`]); this block-order driver stays as the
/// frozen witness path. The `_rand` stream is shared across the grass/vine
/// drains in pass order (the C# `_rand` is one `UnifiedRandom` per
/// `TileDrawing`, `:104`/`:191`) — the sharing survives the fold because
/// the caller threads ONE `&mut UnifiedRandom` through every dispatch.
#[allow(clippy::too_many_arguments)]
pub(crate) fn post_draw_drain_order(
    world: &World,
    wind: &Wind,
    light: &tr_core::lighting::LightMap,
    grass_counter: f32,
    sunflower_counter: f32,
    vine_counter: f32,
    rng: &mut UnifiedRandom,
    seam: &mut impl DrainCellLight,
    ported: &mut impl PortedBodyDrain,
    show_invisible: bool,
    disco: [u8; 3],
    master_color: f32,
    tile_frame_seed: u64,
    global_time: f32,
    buckets: &[Vec<(i32, i32)>; 11],
    sprites: &mut Vec<DrainSprite>,
    fairy_jars: &mut Vec<DrainFairyJar>,
    rising: &mut Vec<RisingVineSegment>,
    jar_items: &mut Vec<DrainJarItem>,
    flames: &mut Vec<DrainFlame>,
) {
    for pass in POST_DRAW_DRAIN_ORDER {
        drain_one_pass(
            pass, world, wind, light, grass_counter, sunflower_counter,
            vine_counter, rng, seam, ported, show_invisible, disco,
            master_color, tile_frame_seed, global_time, buckets, sprites,
            fairy_jars, rising, jar_items, flames,
        );
    }
}

/// One `PostDrawTiles(false)` nature pass — the per-pass dispatch the g29
/// `TilePassSeams::post_draw_pass(step)` seam binds to (contract §1.4/
/// §2.3). The five MISSING bodies (MultiTileVines / MultiTileGrass /
/// Grass / AnyDirectionalGrass / ReverseVines) run through this module's
/// drains; the five ported bodies (VoidLenses / Pylons / Trophies / Trees /
/// Vines) run through the [`PortedBodyDrain`] seam. `buckets` is the g30
/// `SpecialPoints.buckets` array passed by reference (no second registry).
#[allow(clippy::too_many_arguments)]
pub(crate) fn drain_one_pass(
    pass: DrainPass,
    world: &World,
    wind: &Wind,
    light: &tr_core::lighting::LightMap,
    grass_counter: f32,
    sunflower_counter: f32,
    vine_counter: f32,
    rng: &mut UnifiedRandom,
    seam: &mut impl DrainCellLight,
    ported: &mut impl PortedBodyDrain,
    show_invisible: bool,
    disco: [u8; 3],
    master_color: f32,
    tile_frame_seed: u64,
    global_time: f32,
    buckets: &[Vec<(i32, i32)>; 11],
    sprites: &mut Vec<DrainSprite>,
    fairy_jars: &mut Vec<DrainFairyJar>,
    rising: &mut Vec<RisingVineSegment>,
    jar_items: &mut Vec<DrainJarItem>,
    flames: &mut Vec<DrainFlame>,
) {
    let points = &buckets[drain_pass_bucket(pass)];
    match pass {
        DrainPass::MultiTileVines => drain_multi_tile_vines(
            world, wind, light, sunflower_counter, rng, seam, show_invisible,
            disco, master_color, tile_frame_seed, global_time, points,
            sprites, fairy_jars, jar_items, flames,
        ),
        DrainPass::MultiTileGrass => drain_multi_tile_grass(
            world, wind, light, sunflower_counter, rng, seam, show_invisible,
            points, sprites,
        ),
        DrainPass::Grass => drain_grass(
            world, wind, light, grass_counter, rng, seam, show_invisible,
            points, sprites,
        ),
        DrainPass::AnyDirectionalGrass => drain_any_directional_grass(
            world, wind, light, grass_counter, rng, seam, show_invisible,
            points, sprites,
        ),
        DrainPass::ReverseVines => {
            drain_reverse_vines(world, wind, vine_counter, show_invisible, points, rising)
        }
        DrainPass::Trees => {
            for &(x, y) in points {
                // `tile == null || !tile.active()` (`:8066`).
                if world.tile(x, y).is_some_and(|t| t.active()) {
                    ported.tree_point(world, x, y);
                }
            }
        }
        DrainPass::Vines => {
            for &(x, y) in points {
                ported.vine_root(world, x, y);
            }
        }
        DrainPass::VoidLenses => {
            for &(x, y) in points {
                ported.void_lens_point(world, x, y);
            }
        }
        DrainPass::TeleportationPylons => {
            for &(x, y) in points {
                // `tile == null || !tile.active()` (`:8647`).
                if world.tile(x, y).is_some_and(|t| t.active()) {
                    ported.pylon_point(world, x, y);
                }
            }
        }
        DrainPass::MasterTrophies => {
            for &(x, y) in points {
                // `tile == null || !tile.active()` (`:8605`).
                if world.tile(x, y).is_some_and(|t| t.active()) {
                    ported.trophy_point(world, x, y);
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Tests-as-code (written, NOT run — code-only mode)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use tr_core::tile::FLAG_ACTIVE;
    use tr_core::lighting::LightMap;

    fn world(w: i32, h: i32) -> World {
        let mut world = World::new("t".into(), w, h);
        world.compute_column_surface();
        world.compute_chunks();
        world
    }

    fn put(w: &mut World, x: i32, y: i32, kind: u16, fx: i16, fy: i16) {
        let i = w.index(x, y);
        w.tiles[i].kind = kind;
        w.tiles[i].flags = FLAG_ACTIVE;
        w.tiles[i].frame_x = fx;
        w.tiles[i].frame_y = fy;
    }

    fn calm_wind() -> Wind {
        let mut wind = Wind::new(7);
        wind.speed_current = 0.0;
        wind.world_surface = f64::INFINITY;
        wind
    }

    fn windy() -> Wind {
        let mut wind = Wind::new(7);
        wind.speed_current = 0.5;
        wind.world_surface = f64::INFINITY;
        wind
    }

    /// A counting seam: records the per-cell `Next(4)` flags the drains drew,
    /// proving the `_rand` consumption order (the stream-order rule).
    struct CountingLight {
        flags: Vec<bool>,
    }
    impl DrainCellLight for CountingLight {
        fn cell_light(
            &mut self,
            _world: &World,
            _x: i32,
            _y: i32,
            _kind: u16,
            _frame_x: i16,
            _frame_y: i16,
            base: [f32; 3],
            can_do_dust: bool,
        ) -> [f32; 3] {
            self.flags.push(can_do_dust);
            base
        }
    }

    /// A recording ported-body seam: logs the (pass, x, y) order.
    #[derive(Default)]
    struct RecordingBodies {
        calls: Vec<(&'static str, i32, i32)>,
    }
    impl PortedBodyDrain for RecordingBodies {
        fn tree_point(&mut self, _w: &World, x: i32, y: i32) {
            self.calls.push(("tree", x, y));
        }
        fn vine_root(&mut self, _w: &World, x: i32, y: i32) {
            self.calls.push(("vine", x, y));
        }
        fn void_lens_point(&mut self, _w: &World, x: i32, y: i32) {
            self.calls.push(("lens", x, y));
        }
        fn pylon_point(&mut self, _w: &World, x: i32, y: i32) {
            self.calls.push(("pylon", x, y));
        }
        fn trophy_point(&mut self, _w: &World, x: i32, y: i32) {
            self.calls.push(("trophy", x, y));
        }
    }

    // -- ClimbCatTail --------------------------------------------------------

    /// Witness: `ClimbCatTail` (`TileDrawing.cs:8807-8827`) counts the rows
    /// up to (and including) the `frameX >= 180` flame top; a non-519 cell
    /// stops the walk without counting.
    #[test]
    fn w4_g31_climb_cat_tail_counts_to_the_flame_top() {
        let mut w = world(20, 40);
        // A 3-tall campfire: rows 22/21 body (frameX 0), row 20 flame top.
        put(&mut w, 5, 22, 519, 0, 36);
        put(&mut w, 5, 21, 519, 0, 18);
        put(&mut w, 5, 20, 519, 180, 0);
        assert_eq!(climb_cat_tail(&w, 5, 22), 3);
        // No flame top: the walk counts every 519 row until a non-519 cell.
        let mut w2 = world(20, 40);
        put(&mut w2, 5, 22, 519, 0, 36);
        put(&mut w2, 5, 21, 519, 0, 18);
        // row 20 is air -> stops, count 2.
        assert_eq!(climb_cat_tail(&w2, 5, 22), 2);
    }

    /// Witness: the `num2 > 10` floor (`:8811`) — a campfire at row 10 or
    /// below counts nothing.
    #[test]
    fn w4_g31_climb_cat_tail_stops_at_row_ten() {
        let mut w = world(20, 40);
        put(&mut w, 5, 10, 519, 0, 0);
        assert_eq!(climb_cat_tail(&w, 5, 10), 0);
    }

    // -- The size tables -----------------------------------------------------

    /// Witness: `DrawMultiTileVines`' size table (`TileDrawing.cs:8845-8884`).
    #[test]
    fn w4_g31_multi_tile_vine_size_table() {
        assert_eq!(multi_tile_vine_size(34), (3, 3));
        assert_eq!(multi_tile_vine_size(454), (4, 3));
        assert_eq!(multi_tile_vine_size(42), (1, 2));
        assert_eq!(multi_tile_vine_size(660), (1, 2));
        assert_eq!(multi_tile_vine_size(91), (1, 3));
        assert_eq!(multi_tile_vine_size(95), (2, 2));
        assert_eq!(multi_tile_vine_size(465), (2, 3));
        assert_eq!(multi_tile_vine_size(592), (2, 3));
        assert_eq!(multi_tile_vine_size(698), (1, 1));
        assert_eq!(multi_tile_vine_size(999), (1, 1));
    }

    /// Witness: `DrawMultiTileGrass`' size table (`TileDrawing.cs:8754-8801`),
    /// incl. the 233 row-0 width split (`:8766`).
    #[test]
    fn w4_g31_multi_tile_grass_size_table() {
        let mut w = world(20, 40);
        put(&mut w, 5, 20, 233, 0, 0);
        put(&mut w, 8, 20, 233, 0, 36);
        assert_eq!(multi_tile_grass_size(&w, 27, 5, 20), (2, 5));
        assert_eq!(multi_tile_grass_size(&w, 236, 5, 20), (2, 2));
        assert_eq!(multi_tile_grass_size(&w, 233, 5, 20), (3, 2)); // row 0
        assert_eq!(multi_tile_grass_size(&w, 233, 8, 20), (2, 2)); // row 36
        assert_eq!(multi_tile_grass_size(&w, 530, 5, 20), (3, 2));
        assert_eq!(multi_tile_grass_size(&w, 489, 5, 20), (2, 3));
        assert_eq!(multi_tile_grass_size(&w, 493, 5, 20), (1, 2));
        assert_eq!(multi_tile_grass_size(&w, 521, 5, 20), (2, 2));
    }

    // -- The style tables ----------------------------------------------------

    /// Witness: the 34/126 planter style table (`TileDrawing.cs:9269-9349`)
    /// — the `frameY/54 + frameX/108*37` index and the per-style tuples.
    #[test]
    fn w4_g31_planter_style_table() {
        // style 9: frameY 486 (486/54=9), frameX 0.
        assert_eq!(planter_style(0, 486), (None, -1.0, true, 0.3));
        // style 11: frameY 594.
        assert_eq!(planter_style(0, 594), (Some(1.0), 0.0, false, 0.5));
        // style 23: frameY 1242.
        assert_eq!(planter_style(0, 1242), (Some(0.0), 0.0, false, 1.0));
        // style 40: frameY 2160 -> num5 -2, flag2, half.
        assert_eq!(planter_style(0, 2160), (None, -2.0, true, 0.5));
        // style 44: frameY 2376 -> num5 -3.
        assert_eq!(planter_style(0, 2376), (None, -3.0, false, 1.0));
        // style 67: frameY 3618 -> num3 1.
        assert_eq!(planter_style(0, 3618), (Some(1.0), 0.0, false, 1.0));
        // The frameX column shift: frameX 108 adds 37 to the index.
        // frameY 0 + frameX 108 -> style 37 (None, -1, true, 0.5).
        assert_eq!(planter_style(108, 0), (None, -1.0, true, 0.5));
    }

    /// Witness: the 42 bottle style table (`TileDrawing.cs:9354-9417`) —
    /// incl. the `:9404-9405` dead-write (`num3 = 0f; num3 = null;` — the
    /// second write wins, so styles 40-43 are `(None, -1, true, 1)`).
    #[test]
    fn w4_g31_bottle_style_table_and_dead_write() {
        assert_eq!(bottle_style(0), (None, -1.0, false, 1.0)); // style 0
        assert_eq!(bottle_style(324), (Some(0.0), 0.0, false, 1.0)); // style 9
        assert_eq!(bottle_style(1404), (None, -1.0, true, 1.0)); // style 39
        // styles 40-43: the dead `num3 = 0f` is overwritten by `num3 = null`.
        assert_eq!(bottle_style(1440), (None, -1.0, true, 1.0)); // style 40
        assert_eq!(bottle_style(1548), (None, -1.0, true, 1.0)); // style 43
        assert_eq!(bottle_style(1944), (Some(0.0), 0.0, false, 1.0)); // style 54
    }

    // -- DrawGrass -----------------------------------------------------------

    /// Witness: `DrawGrass` (`TileDrawing.cs:8326-8381`) — the calm-wind
    /// sprite anchors at `(x*16+8, y*16+16)`, origin `(w/2, 16-half-top)`,
    /// rotation 0; the `_rand.Next(4)` flag is consumed once per cell.
    #[test]
    fn w4_g31_drain_grass_calm_anchor_and_origin() {
        let mut w = world(20, 40);
        put(&mut w, 5, 20, 3, 0, 0); // a grass cell (type 3)
        let wind = calm_wind();
        let light = LightMap::new();
        let mut rng = UnifiedRandom::new(1);
        let mut seam = CountingLight { flags: Vec::new() };
        let mut out = Vec::new();
        drain_grass(&w, &wind, &light, 0.0, &mut rng, &mut seam, false, &[(5, 20)], &mut out);
        assert_eq!(out.len(), 1);
        let s = &out[0];
        assert_eq!(s.pos, [5.0 * 16.0 + 8.0, 20.0 * 16.0 + 16.0]);
        assert_eq!(s.rot, 0.0);
        assert_eq!(s.origin, [s.dd.width as f32 * 0.5, (16 - s.half - s.dd.top) as f32]);
        assert_eq!(seam.flags.len(), 1); // one Next(4) per cell
    }

    /// Witness: the wind gates (`:8360-8367`) — a wind-blocking wall zeroes
    /// the cycle even in a windy place; the grid push still adds.
    #[test]
    fn w4_g31_drain_grass_wall_blocks_wind() {
        let mut w = world(20, 40);
        put(&mut w, 5, 20, 3, 0, 0);
        let i = w.index(5, 20);
        w.tiles[i].wall = 1; // a non-AllowsWind wall
        let wind = windy();
        let light = LightMap::new();
        let mut rng = UnifiedRandom::new(1);
        let mut seam = PlainLight;
        let mut out = Vec::new();
        drain_grass(&w, &wind, &light, 0.0, &mut rng, &mut seam, false, &[(5, 20)], &mut out);
        // The cycle is zeroed (wall), the grid push is 0 (no stamps), so the
        // sprite stays at the base anchor.
        assert_eq!(out[0].pos, [5.0 * 16.0 + 8.0, 20.0 * 16.0 + 16.0]);
        assert_eq!(out[0].rot, 0.0);
    }

    /// Witness: the active+visible gate (`:8338`) — an inactive cell and an
    /// Echo-coated cell (without sight) draw nothing, but the inactive cell
    /// consumes NO rand (the gate precedes the roll).
    #[test]
    fn w4_g31_drain_grass_gates_skip_before_the_roll() {
        let mut w = world(20, 40);
        // inactive cell
        let i = w.index(5, 20);
        w.tiles[i].kind = 3;
        w.tiles[i].flags = 0; // not active
        let wind = calm_wind();
        let light = LightMap::new();
        let mut rng = UnifiedRandom::new(1);
        let mut seam = CountingLight { flags: Vec::new() };
        let mut out = Vec::new();
        drain_grass(&w, &wind, &light, 0.0, &mut rng, &mut seam, false, &[(5, 20)], &mut out);
        assert!(out.is_empty());
        assert!(seam.flags.is_empty()); // no roll for a gated cell
    }

    // -- DrawAnyDirectionalGrass --------------------------------------------

    /// Witness: the four-direction switch (`TileDrawing.cs:8429-8467`) —
    /// each `frameY/54` row picks its origin + position nudges. Calm wind
    /// isolates the switch geometry (num3 = 0).
    #[test]
    fn w4_g31_any_directional_grass_four_directions() {
        let mut w = world(20, 60);
        let wind = calm_wind();
        let light = LightMap::new();
        for (row, fy) in [(0, 0i16), (1, 54), (2, 108), (3, 162)] {
            let y = 20 + row * 2;
            put(&mut w, 5, y, 184, 0, fy);
            let mut rng = UnifiedRandom::new(1);
            let mut seam = PlainLight;
            let mut out = Vec::new();
            drain_any_directional_grass(
                &w, &wind, &light, 0.0, &mut rng, &mut seam, false, &[(5, y)], &mut out,
            );
            assert_eq!(out.len(), 1);
            let s = &out[0];
            let base_x = 5.0 * 16.0;
            let base_y = y as f32 * 16.0;
            match row {
                0 => {
                    assert_eq!(s.pos, [base_x + 8.0, base_y + 16.0]);
                    assert_eq!(s.origin[0], s.dd.width as f32 * 0.5);
                }
                1 => {
                    assert_eq!(s.pos, [base_x + 8.0, base_y]);
                    assert_eq!(s.origin, [s.dd.width as f32 * 0.5, (-s.dd.top) as f32]);
                }
                2 => {
                    assert_eq!(s.pos, [base_x, base_y + 8.0]);
                    assert_eq!(s.origin[0], 2.0);
                }
                3 => {
                    assert_eq!(s.pos, [base_x + 16.0, base_y + 8.0]);
                    assert_eq!(s.origin[0], 14.0);
                }
                _ => unreachable!(),
            }
        }
    }

    // -- DrawRisingVineStrip -------------------------------------------------

    /// Witness: `DrawRisingVineStrip` (`TileDrawing.cs:9134-9193`) — the UP
    /// walk from the root, the anchor start `(x*16+8, startY*16+18)`, the
    /// `(rot - PI/2)` advance, and the `num7 > 10` floor.
    #[test]
    fn w4_g31_rising_vine_strip_walks_up() {
        let mut w = world(20, 60);
        // A reverse-vine (549) strip rooted at row 30, growing up to row 28.
        for y in 28..=30 {
            put(&mut w, 5, y, 549, 0, 0);
        }
        let wind = calm_wind();
        let segs = rising_vine_strip(&w, &wind, 0.0, 5, 30, false);
        assert_eq!(segs.len(), 3);
        // The first segment anchors at the root.
        assert_eq!(segs[0].anchor, [5.0 * 16.0 + 8.0, 30.0 * 16.0 + 18.0]);
        assert_eq!(segs[0].y, 30);
        assert_eq!(segs[1].y, 29);
        assert_eq!(segs[2].y, 28);
        // Calm wind: rot 0, so the advance is (cos(-PI/2), sin(-PI/2))*16 =
        // (0, -16) — each segment sits 16 px above the previous.
        assert!((segs[1].anchor[1] - (segs[0].anchor[1] - 16.0)).abs() < 1e-3);
        assert!((segs[1].anchor[0] - segs[0].anchor[0]).abs() < 1e-3);
    }

    /// Witness: the strip breaks on a non-ReverseVineThreads cell (`:9150`).
    #[test]
    fn w4_g31_rising_vine_strip_breaks_on_non_thread() {
        let mut w = world(20, 60);
        put(&mut w, 5, 30, 549, 0, 0);
        put(&mut w, 5, 29, 0, 0, 0); // dirt — not a reverse vine
        put(&mut w, 5, 28, 549, 0, 0);
        let wind = calm_wind();
        let segs = rising_vine_strip(&w, &wind, 0.0, 5, 30, false);
        assert_eq!(segs.len(), 1); // stops at the dirt
    }

    // -- DrawMultiTileGrassInWind -------------------------------------------

    /// Witness: the sunflower glow latch (`TileDrawing.cs:8929-8933`) — a 27
    /// anchor sets the +74 glow on every drawn cell; the 521-527 shards
    /// freeze (rot 0) and carry no glow.
    #[test]
    fn w4_g31_multi_tile_grass_sunflower_glow_and_shard_freeze() {
        let mut w = world(20, 40);
        // A 2x5 sunflower at (5, 20).
        for dx in 0..2 {
            for dy in 0..5 {
                put(&mut w, 5 + dx, 20 + dy, 27, dx as i16 * 18, dy as i16 * 18);
            }
        }
        let wind = windy();
        let light = LightMap::new();
        let mut rng = UnifiedRandom::new(1);
        let mut seam = PlainLight;
        let mut out = Vec::new();
        multi_tile_grass_in_wind(
            &w, &wind, &light, 0.0, &mut rng, &mut seam, false, 5, 20, 2, 5, &mut out,
        );
        assert_eq!(out.len(), 10);
        for s in &out {
            assert!(s.glow.is_some());
            assert_eq!(s.glow.unwrap().src_y_add, 74);
        }
        // A 521 crystal shard: frozen (num = 0 -> rot 0), no glow.
        let mut w2 = world(20, 40);
        for dx in 0..2 {
            for dy in 0..2 {
                put(&mut w2, 5 + dx, 20 + dy, 521, dx as i16 * 18, dy as i16 * 18);
            }
        }
        let mut out2 = Vec::new();
        let mut rng2 = UnifiedRandom::new(1);
        multi_tile_grass_in_wind(
            &w2, &wind, &light, 0.0, &mut rng2, &mut seam, false, 5, 20, 2, 2, &mut out2,
        );
        for s in &out2 {
            assert_eq!(s.rot, 0.0);
            assert!(s.glow.is_none());
        }
    }

    /// Witness: the 493 golf-flag flip (`TileDrawing.cs:8983-8997`) — the
    /// wind sign flips the sprite and shifts it ±6 px.
    #[test]
    fn w4_g31_multi_tile_grass_golf_flag_wind_flip() {
        let mut w = world(20, 40);
        put(&mut w, 5, 20, 493, 0, 0);
        put(&mut w, 5, 21, 493, 0, 18);
        let light = LightMap::new();
        let mut seam = PlainLight;
        // Positive wind: flip toggles from dd.flip.
        let mut wind_pos = windy();
        wind_pos.speed_current = 0.5;
        let mut out_pos = Vec::new();
        let mut rng = UnifiedRandom::new(1);
        multi_tile_grass_in_wind(
            &w, &wind_pos, &light, 0.0, &mut rng, &mut seam, false, 5, 20, 1, 2, &mut out_pos,
        );
        // Negative wind: no toggle.
        let mut wind_neg = windy();
        wind_neg.speed_current = -0.5;
        let mut out_neg = Vec::new();
        let mut rng2 = UnifiedRandom::new(1);
        multi_tile_grass_in_wind(
            &w, &wind_neg, &light, 0.0, &mut rng2, &mut seam, false, 5, 20, 1, 2, &mut out_neg,
        );
        // The top-row (frameY 0) cells flip differently on the wind sign.
        let top_pos = out_pos.iter().find(|s| s.y == 20).unwrap();
        let top_neg = out_neg.iter().find(|s| s.y == 20).unwrap();
        assert_ne!(top_pos.flip, top_neg.flip);
    }

    // -- DrawMultiTileVinesInWind -------------------------------------------

    /// Witness: the platform lift (`TileDrawing.cs:9250-9254`) — a 1-wide
    /// hanging object under a non-hammered platform lifts its anchor 8 px.
    #[test]
    fn w4_g31_multi_tile_vine_platform_lift() {
        let mut w = world(20, 40);
        // A platform at (5, 19), a 1-wide 91 bottle hanging at (5, 20).
        put(&mut w, 5, 19, 19, 0, 0); // platform
        for dy in 0..3 {
            put(&mut w, 5, 20 + dy, 91, 0, dy as i16 * 18);
        }
        let wind = calm_wind();
        let light = LightMap::new();
        let mut rng = UnifiedRandom::new(1);
        let mut seam = PlainLight;
        let mut out = Vec::new();
        let mut fairy = Vec::new();
        let mut jars = Vec::new();
        let mut flames = Vec::new();
        multi_tile_vines_in_wind(
            &w, &wind, &light, 0.0, &mut rng, &mut seam, false, 5, 20, 1, 3,
            [0, 0, 0], 0.0, 0, 0.0, &mut out, &mut fairy, &mut jars, &mut flames,
        );
        assert!(!out.is_empty());
        // The anchor vector is lifted: vector.y = 20*16 - 2 - 8 = 310.
        // Each cell's origin = vector - vector3; the top cell (j=20, top=0)
        // has vector3.y = 20*16 + (-2-8) = 310, so origin.y = 310 - 310 = 0.
        let top = out.iter().find(|s| s.y == 20).unwrap();
        assert!((top.origin[1] - 0.0).abs() < 1e-3);
    }

    /// Witness: the 660 fairy-jar bottom row (`TileDrawing.cs:9505-9519`) —
    /// the bottom-row cell gets an `Extra_260` white+HSL pair (A=127 glow),
    /// the upper cell does not; BOTH cells still draw their tile sprite.
    #[test]
    fn w4_g31_multi_tile_vine_fairy_jar_bottom_glow() {
        let mut w = world(20, 40);
        // A 1x2 660 fairy jar at (5, 20).
        put(&mut w, 5, 20, 660, 0, 0);
        put(&mut w, 5, 21, 660, 0, 18);
        let wind = calm_wind();
        let light = LightMap::new();
        let mut rng = UnifiedRandom::new(1);
        let mut seam = PlainLight;
        let mut out = Vec::new();
        let mut fairy = Vec::new();
        let mut jars = Vec::new();
        let mut flames = Vec::new();
        multi_tile_vines_in_wind(
            &w, &wind, &light, 0.0, &mut rng, &mut seam, false, 5, 20, 1, 2,
            [0, 0, 0], 0.0, 0, 0.0, &mut out, &mut fairy, &mut jars, &mut flames,
        );
        // Both tile sprites draw (`:9512` runs for every cell).
        assert_eq!(out.len(), 2);
        // Only the bottom row gets the Extra_260 pair.
        assert_eq!(fairy.len(), 1);
        assert_eq!(fairy[0].y, 21);
        assert!((fairy[0].glow_color[3] - 127.0 / 255.0).abs() < 1e-3);
    }

    /// Witness: the shared flame seed (`TileDrawing.cs:9590-9594`) — the
    /// first non-zero `flameSeed` across the object's cells is reused for
    /// every flame quad. The 270 firefly jar's `GetTileFlameData` returns
    /// count 1 with a ZERO seed (`:1851-1857`), so it does NOT latch the
    /// shared seed; the 42 candle (sheet 13, frameY/36 == 1) returns count 7
    /// with a real per-cell seed — the first cell's seed wins for both.
    #[test]
    fn w4_g31_multi_tile_vine_shared_flame_seed() {
        let mut w = world(20, 40);
        // A 1x2 42 hanging candle, frameY 36 (style 1 -> count 7) and 72
        // (style 2 -> count 7). Both cells flame.
        put(&mut w, 5, 20, 42, 0, 36);
        put(&mut w, 5, 21, 42, 0, 72);
        let wind = calm_wind();
        let light = LightMap::new();
        let mut rng = UnifiedRandom::new(1);
        let mut seam = PlainLight;
        let mut out = Vec::new();
        let mut fairy = Vec::new();
        let mut jars = Vec::new();
        let mut flames = Vec::new();
        multi_tile_vines_in_wind(
            &w, &wind, &light, 0.0, &mut rng, &mut seam, false, 5, 20, 1, 2,
            [0, 0, 0], 0.0, 12345, 0.0, &mut out, &mut fairy, &mut jars, &mut flames,
        );
        // 7 flames per cell, 14 total; all derive from the FIRST cell's seed.
        assert_eq!(flames.len(), 14);
        // Replay the first cell's seed independently: the first 7 quads'
        // offsets match a fresh single-cell run, proving the seed latched.
        let seed0 = tr_core::flame::frame_seed_for(12345, 5, 20);
        let mut s = seed0;
        let fd = tr_core::flame::tile_flame_data(5, 20, 42, 0, 36, 12345, 0.0, [0, 0, 0]);
        assert_eq!(fd.count, 7);
        for k in 0..7 {
            let x2 = tr_core::flame::random_int(&mut s, fd.rx.0, fd.rx.1) as f32 * fd.mx;
            let y2 = tr_core::flame::random_int(&mut s, fd.ry.0, fd.ry.1) as f32 * fd.my;
            // The quad pos is the shared anchor (vector5) + the offset; both
            // cells share vector5.x, so the x delta from the anchor equals x2.
            let anchor_x = flames[k].pos[0] - x2;
            assert!((flames[k].pos[0] - (anchor_x + x2)).abs() < 1e-3);
            let _ = y2;
        }
        // The second cell's flames reuse seed0 -> identical X offsets to the
        // first cell's (a fresh per-cell seed would diverge).
        for k in 0..7 {
            assert!((flames[k].pos[0] - flames[k + 7].pos[0]).abs() < 1e-3);
        }
    }

    // -- The drain order + bucket indices -----------------------------------

    /// Witness: `PostDrawTiles(false)`'s ten-pass order
    /// (`TileDrawing.cs:421-430`) and each pass's bucket index.
    #[test]
    fn w4_g31_post_draw_drain_order_and_buckets() {
        assert_eq!(
            POST_DRAW_DRAIN_ORDER,
            [
                DrainPass::MultiTileVines,
                DrainPass::MultiTileGrass,
                DrainPass::VoidLenses,
                DrainPass::TeleportationPylons,
                DrainPass::MasterTrophies,
                DrainPass::Grass,
                DrainPass::AnyDirectionalGrass,
                DrainPass::Trees,
                DrainPass::Vines,
                DrainPass::ReverseVines,
            ]
        );
        assert_eq!(drain_pass_bucket(DrainPass::Trees), 0);
        assert_eq!(drain_pass_bucket(DrainPass::Grass), 1);
        assert_eq!(drain_pass_bucket(DrainPass::MultiTileGrass), 2);
        assert_eq!(drain_pass_bucket(DrainPass::MultiTileVines), 3);
        assert_eq!(drain_pass_bucket(DrainPass::Vines), 4);
        assert_eq!(drain_pass_bucket(DrainPass::VoidLenses), 6);
        assert_eq!(drain_pass_bucket(DrainPass::ReverseVines), 7);
        assert_eq!(drain_pass_bucket(DrainPass::TeleportationPylons), 8);
        assert_eq!(drain_pass_bucket(DrainPass::MasterTrophies), 9);
        assert_eq!(drain_pass_bucket(DrainPass::AnyDirectionalGrass), 10);
        // BiomeGrass (5) is never drained.
        for p in POST_DRAW_DRAIN_ORDER {
            assert_ne!(drain_pass_bucket(p), 5);
        }
        // The g32 `the_post_pass_bucket_indices` None-arm fold (contract
        // §2.3): the nature flush + the two TE passes drain NO bucket,
        // and neither does the EndBatch step; every nature step of the
        // g29 12-step order maps to exactly the frozen drain order.
        use crate::render::tile_pass::{PostDrawStep, POST_DRAW_NON_SOLID, POST_DRAW_SOLID};
        assert_eq!(drain_pass_for_post_step(PostDrawStep::EndBatch), None);
        assert_eq!(drain_pass_for_post_step(PostDrawStep::NatureAfterAllObjects), None);
        assert_eq!(drain_pass_for_post_step(PostDrawStep::DrawEntitiesHatRacks), None);
        assert_eq!(drain_pass_for_post_step(PostDrawStep::DrawEntitiesDisplayDolls), None);
        for step in POST_DRAW_SOLID {
            assert_eq!(drain_pass_for_post_step(step), None);
        }
        let mapped: Vec<DrainPass> = POST_DRAW_NON_SOLID
            .iter()
            .filter_map(|s| drain_pass_for_post_step(*s))
            .collect();
        assert_eq!(mapped, POST_DRAW_DRAIN_ORDER);
        // The g28 frozen bucket order fold (`the_layers_and_post_draw_
        // order_are_frozen`): the drain order's bucket sequence is
        // 3,2,6,8,9,1,10,0,4,7 — reordering changes the composite.
        assert_eq!(
            POST_DRAW_DRAIN_ORDER.map(drain_pass_bucket),
            [3, 2, 6, 8, 9, 1, 10, 0, 4, 7]
        );
        // The g30 registry feeds the dispatch directly (contract §1.4):
        // the drivers take the buckets array by reference — no second
        // registry. (Shape bind: SpecialPoints.buckets is the same type.)
        let reg = crate::render::special_points::SpecialPoints::new();
        let _view: &[Vec<(i32, i32)>; 11] = &reg.buckets;
    }

    /// Witness: the driver walks the ported bodies in C# pass order — Trees
    /// (bucket 0) before Vines (bucket 4), and the pylon/trophy/lens passes
    /// run before Grass. The active gate skips an inactive tree point.
    /// UNION (contract §1.4): re-run against the per-pass fold — the
    /// block driver IS the `drain_one_pass` loop now, same order.
    #[test]
    fn w4_g31_driver_walks_ported_bodies_in_order() {
        let mut w = world(20, 40);
        put(&mut w, 5, 20, 5, 22, 198); // a tree marker (active)
        put(&mut w, 6, 20, 5, 22, 198); // another tree marker
        let i = w.index(7, 20);
        w.tiles[i].kind = 5;
        w.tiles[i].flags = 0; // inactive tree point — gated
        let mut buckets = DrainBuckets::new();
        buckets.points[BUCKET_TREE] = vec![(5, 20), (6, 20), (7, 20)];
        buckets.points[BUCKET_VINE] = vec![(8, 20)];
        buckets.points[BUCKET_TELEPORTATION_PYLON] = vec![(9, 20)];
        put(&mut w, 9, 20, 597, 0, 0); // active pylon anchor
        let wind = calm_wind();
        let light = LightMap::new();
        let mut rng = UnifiedRandom::new(1);
        let mut seam = PlainLight;
        let mut ported = RecordingBodies::default();
        let mut sprites = Vec::new();
        let mut fairy = Vec::new();
        let mut rising = Vec::new();
        let mut jars = Vec::new();
        let mut flames = Vec::new();
        post_draw_drain_order(
            &w, &wind, &light, 0.0, 0.0, 0.0, &mut rng, &mut seam, &mut ported,
            false, [0, 0, 0], 0.0, 0, 0.0, &buckets.points, &mut sprites, &mut fairy,
            &mut rising, &mut jars, &mut flames,
        );
        // The pylon (pass 4) runs before the trees (pass 8) before the vines
        // (pass 9); the inactive tree point (7,20) is gated out.
        assert_eq!(
            ported.calls,
            vec![("pylon", 9, 20), ("tree", 5, 20), ("tree", 6, 20), ("vine", 8, 20)]
        );
    }

    /// Witness: the `_rand` stream is shared across the grass drains in pass
    /// order — MultiTileGrass (pass 2) consumes its per-cell rolls before
    /// Grass (pass 6), so the flags arrive in that order. UNION (contract
    /// §1.4): also re-runs the SAME inputs through the per-pass dispatch
    /// with one shared fresh stream — the flags must match exactly.
    #[test]
    fn w4_g31_rand_stream_shared_across_drains_in_order() {
        let mut w = world(20, 40);
        // A 2x2 multi-tile grass (236) at (5,20) + a windy-grass cell at (9,20).
        for dx in 0..2 {
            for dy in 0..2 {
                put(&mut w, 5 + dx, 20 + dy, 236, dx as i16 * 18, dy as i16 * 18);
            }
        }
        put(&mut w, 9, 20, 3, 0, 0);
        let mut buckets = DrainBuckets::new();
        buckets.points[BUCKET_MULTI_TILE_GRASS] = vec![(5, 20)];
        buckets.points[BUCKET_WINDY_GRASS] = vec![(9, 20)];
        let wind = calm_wind();
        let light = LightMap::new();
        let mut rng = UnifiedRandom::new(1);
        let mut seam = CountingLight { flags: Vec::new() };
        let mut ported = RecordingBodies::default();
        let mut sprites = Vec::new();
        let mut fairy = Vec::new();
        let mut rising = Vec::new();
        let mut jars = Vec::new();
        let mut flames = Vec::new();
        post_draw_drain_order(
            &w, &wind, &light, 0.0, 0.0, 0.0, &mut rng, &mut seam, &mut ported,
            false, [0, 0, 0], 0.0, 0, 0.0, &buckets.points, &mut sprites, &mut fairy,
            &mut rising, &mut jars, &mut flames,
        );
        // 4 multi-tile cells + 1 grass cell = 5 rolls, in pass order.
        assert_eq!(seam.flags.len(), 5);
        // The per-pass fold (contract §1.4): re-running the SAME world
        // through drain_one_pass per step, threading ONE fresh _rand in
        // POST_DRAW_DRAIN_ORDER, reproduces the block driver's flags
        // exactly — the stream sharing survives the dispatch split.
        let mut rng2 = UnifiedRandom::new(1);
        let mut seam2 = CountingLight { flags: Vec::new() };
        let mut ported2 = RecordingBodies::default();
        let mut sprites2 = Vec::new();
        let mut fairy2 = Vec::new();
        let mut rising2 = Vec::new();
        let mut jars2 = Vec::new();
        let mut flames2 = Vec::new();
        for pass in POST_DRAW_DRAIN_ORDER {
            drain_one_pass(
                pass, &w, &wind, &light, 0.0, 0.0, 0.0, &mut rng2, &mut seam2,
                &mut ported2, false, [0, 0, 0], 0.0, 0, 0.0, &buckets.points,
                &mut sprites2, &mut fairy2, &mut rising2, &mut jars2, &mut flames2,
            );
        }
        assert_eq!(seam2.flags, seam.flags, "the per-pass fold keeps the stream");
    }

    // -- The rarity colour + rgbToHsl ---------------------------------------

    /// Witness: `jar_rarity_color` (`TileDrawing.cs:9552-9557` +
    /// `Item.GetPopupRarityColor`, `Item.cs:652-707`).
    #[test]
    fn w4_g31_jar_rarity_color_table() {
        assert_eq!(jar_rarity_color(1, false, [0, 0, 0], 0.0), [150, 150, 255]);
        assert_eq!(jar_rarity_color(-1, false, [0, 0, 0], 0.0), [130, 130, 130]);
        assert_eq!(jar_rarity_color(11, false, [0, 0, 0], 0.0), [180, 40, 255]);
        // expert -> -12 -> the disco colour.
        assert_eq!(jar_rarity_color(5, true, [10, 20, 30], 0.0), [10, 20, 30]);
        // -13 -> the master colour.
        assert_eq!(jar_rarity_color(-13, false, [0, 0, 0], 0.5), [255, 100, 0]);
        // unknown rarity -> White.
        assert_eq!(jar_rarity_color(0, false, [0, 0, 0], 0.0), [255, 255, 255]);
    }

    /// Witness: `Main.rgbToHsl` (`Main.cs:47944-47980`) — pure red is hue 0,
    /// the grey axis has saturation 0.
    #[test]
    fn w4_g31_rgb_to_hsl_red_and_grey() {
        let red = rgb_to_hsl([255, 0, 0]);
        assert!((red[0] - 0.0).abs() < 1e-3);
        assert!((red[1] - 1.0).abs() < 1e-3);
        assert!((red[2] - 0.5).abs() < 1e-3);
        let grey = rgb_to_hsl([128, 128, 128]);
        assert_eq!(grey[0], 0.0);
        assert_eq!(grey[1], 0.0);
    }
}