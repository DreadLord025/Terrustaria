//! `TileDrawing.DrawTrees`: canopy and branch sprites anchored to the
//! transparent marker tiles of a tree, their `GetWindCycle` sway
//! (`TileDrawing.cs:8137-8245`), and the ambient leaf gores of
//! `TileDrawing.EmitTreeLeaves` (`TileDrawing.cs:7779-7891`).
//!
//! W4-g68: the leaf PHYSICS moved to the `GorePool` — every spawn goes
//! through `Gore.NewGore*` and `Gore_UpdateLeaf` (`Gore.cs:1085-1209`,
//! `tr_core::gore::GorePool::update_leaf`) ticks it in the `gores` phase
//! like every other gore. The old private `LeafGore`/`tick_leaves` pair
//! (a 400-cap Vec with its own fall/rest approximation) is retired.
//! RNG: the gate draws ride the caller's `TileDrawing._rand` stream
//! (`FastRandom`, pinned per frame like the sparkle draws); the
//! `Main.rand` draws (`EmitTreeLeaves`' hallowed `Next(0, 28)` at
//! `TileDrawing.cs:7884` and every `NewGore` internal roll) ride a
//! `GoreRng` the caller injects — two streams, kept separate (the
//! W4-d60 census precedent).
use crate::assets;
use glam::Vec2;
use tr_core::gore::GorePool;
use tr_core::lighting::{LightMap, LightStyle};
use tr_core::tile_light::FastRandom;
use tr_core::tiledata as td;
use tr_core::wind;
use tr_core::{tree, UnifiedRandom, World, TILE_PX};
use tr_render::SpriteRenderer;

/// `WorldGen.SolidTile(Tile)` (`WorldGen.cs:69957-69974`): the STRICT test —
/// a null tile counts solid; otherwise active, not framed off, solid, not a
/// solid-top, not a half brick, not sloped. The 192 living-wood exemption is
/// the post-worldgen `tileSolid[192] = false` (`WorldGen.cs:16607`), the
/// `tr_core::gore::solid_tile` convention (kept local: that one is private).
fn solid_tile(tile: Option<&tr_core::tile::Tile>) -> bool {
    let Some(t) = tile else { return true };
    t.active()
        && !t.inactive()
        && t.kind != 192
        && td::TILE_SOLID.has(t.kind)
        && !td::TILE_SOLID_TOP.has(t.kind)
        && !t.half_brick()
        && t.slope() == 0
}

/// `TileDrawing.EmitLivingTreeLeaf` (`TileDrawing.cs:7549-7556`): the
/// below-spawn first, then the UNCONDITIONAL 1/2 sideways roll on
/// `TileDrawing._rand` (the W4-d60 flagged deviation: the old inline arm
/// gated the roll on a non-zero wind pick — retired with W4-g68).
#[allow(clippy::too_many_arguments)]
pub(crate) fn emit_living_tree_leaf(
    world: &World,
    x: i32,
    y: i32,
    leaf_gore_type: i32,
    wind: f32,
    rand: &mut FastRandom,
    main_rand: &mut UnifiedRandom,
    gore: &mut GorePool,
) {
    emit_living_tree_leaf_below(world, x, y, leaf_gore_type, wind, main_rand, gore);
    if rand.next_range(0, 2) == 0 {
        emit_living_tree_leaf_sideways(world, x, y, leaf_gore_type, wind, main_rand, gore);
    }
}

/// `EmitLivingTreeLeaf_Below` (`TileDrawing.cs:7558-7569`): the tile below
/// must be open (not strict-solid, no liquid); a strong wind needs TWO open
/// cells downwind. `NewGorePerfect` (zero velocity survives the spawn), the
/// paint column is the leaf tile's `color()`.
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
        let left_open =
            !solid_tile(world.tile(x - 1, y + 1)) && !solid_tile(world.tile(x - 2, y + 1));
        let right_open =
            !solid_tile(world.tile(x + 1, y + 1)) && !solid_tile(world.tile(x + 2, y + 1));
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

/// `EmitLivingTreeLeaf_Sideways` (`TileDrawing.cs:7571-7592`): the downwind
/// side (|wind| > 0.2), the −10 px left-shift quirk, paint column again.
/// Runs even in dead wind (`num = 0`, the tile's own cell).
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
            Vec2::new((x * 16 + 8 + 4 * num + num2) as f32, (y * 16 + 8) as f32),
            Vec2::ZERO,
            leaf_gore_type,
            1.0,
        );
        gore.slot_mut(n).frame.current_column =
            world.tile(x, y).map(|t| t.paint()).unwrap_or(0);
    }
}

/// `TileDrawing.EmitLiquidDrops` (`TileDrawing.cs:7594-7674`): the drip
/// gores under vine tips / stalactites / shimmer candles — called from the
/// `TileDrawing.Draw` switch (`:688-694`, kinds 373/374/375/461/709, then
/// `continue`), NOT from `DrawTiles_EmitParticles` (the W4-d60 deferral
/// note; the g30 special-points driver records the cells, this is the
/// emission body). W4-g68 ports the unclaimed arm.
///
/// RNG census (the two-stream rule): the gate `Next(num * 2)` rides
/// `TileDrawing._rand` (`rand`); the `Gore.NewGore` internals ride
/// `Main.rand` (`main_rand`) — the C# call runs outside every SwapRandom
/// block (`Main.cs:17811-17812`). The gate short-circuits: a wet tile
/// (`liquid != 0`) draws NOTHING.
///
/// `zone_corrupt`/`zone_crimson`/`zone_hallow` are the C#
/// `Main.SceneMetrics.Zone*` reads of the 461 shimmer-candle arm — the
/// caller passes the live scene metrics (three bools, not the whole
/// struct, so the witness fixtures stay small).
///
/// **WIRED (W4-g68 wiring wave)** at [`emit_liquid_drops_area`] — the
/// per-frame world pass in `frame/draw.rs` (the tile-cracks precedent),
/// NOT the G6-cached `draw_tiles` range. The C# call site is the
/// `TileDrawing.Draw` switch (`:693`); the g30 special-points driver's
/// `liquid_drop_cells` is the tile-pass-attach CONTRACT for the same
/// cells (UNION-CONTRACT-tilepass §5 d60 row, `task/W4-g31` tip
/// `c3027a38`): g30 records `(x, y, kind)` during the classify walk, this
/// body consumes them. Until the g-family lands on main the driver is
/// absent, so the live seam rescans the draw area itself — the same cells
/// the g30 registry would hold (kinds 373/374/375/461/709 are
/// non-solid, `Draw`'s `solidLayer: false` pass). When g30 lands, the
/// wiring wave swaps the rescan for the recorded cells; the emission
/// body and its RNG order are unchanged either way.
#[allow(clippy::too_many_arguments)]
pub(crate) fn emit_liquid_drops(
    world: &World,
    j: i32,
    i: i32,
    kind: u16,
    water_style: i32,
    zone_corrupt: bool,
    zone_crimson: bool,
    zone_hallow: bool,
    rand: &mut UnifiedRandom,
    main_rand: &mut UnifiedRandom,
    gore: &mut GorePool,
) {
    // `:7596-7608`: the frequency per emitter kind.
    let num: i32 = match kind {
        374 => 120,
        375 | 461 => 180,
        _ => 60,
    };
    // `:7609-7612`: the wet-tile refusal precedes the roll (no draw), then
    // the 1/(2*num) gate on `_rand` (`Next(num * 2)` — one stream step).
    let wet = world.tile(i, j).is_some_and(|t| t.liquid != 0);
    if wet || rand.next_max(num * 2) != 0 {
        return;
    }
    // `:7613-7625`: the anti-cluster box — the tile rect widened ±34 px in
    // X, −100/+300 in Y (Height = 400 from the −100 top); any live drip
    // gore intersecting it refuses the spawn (the strict XNA `Intersects`).
    let rect = (i * 16 - 34, j * 16 - 100, 16 + 68, 400);
    for k in 0..tr_core::gore::GORE_SLOTS {
        let g = gore.slot(k);
        if g.active
            && g.gore_type >= 0
            && (g.gore_type as usize) < tr_core::gore_id::GORE_COUNT
            && tr_core::gore_id::is_drip(g.gore_type as usize)
        {
            let b = (g.position.x as i32, g.position.y as i32, 16, 16);
            if rect.0 < b.0 + b.2 && rect.0 + rect.2 > b.0 && rect.1 < b.1 + b.3 && rect.1 + rect.3 > b.1 {
                return;
            }
        }
    }
    // `:7626-7671`: the drip type by water style, then the emitter overrides.
    let position = Vec2::new((i * 16) as f32, (j * 16) as f32);
    let mut drip_type = if water_style == 14 || water_style == 13 {
        706
    } else if water_style == 12 {
        1147
    } else if water_style > 1 {
        706 + water_style - 1
    } else {
        706
    };
    match kind {
        374 => drip_type = 716, // lava stalactite
        375 => drip_type = 717, // honey stalactite
        461 => {
            // The shimmer candle: biome-tinted (`:7652-7667`), the LAST
            // matching zone wins in C# order (corrupt → crimson → hallow).
            drip_type = 943;
            if zone_corrupt {
                drip_type = 1160;
            }
            if zone_crimson {
                drip_type = 1161;
            }
            if zone_hallow {
                drip_type = 1162;
            }
        }
        709 => drip_type = 1383, // the water candle
        _ => {}
    }
    // `:7672-7673`: `NewGore(position, default, type)` then the velocity
    // zeroing — the spawn's internal kick draws still fire.
    let n = gore.new_gore(main_rand, world, position, Vec2::ZERO, drip_type, 1.0);
    gore.slot_mut(n).velocity *= 0.0;
}

/// The W4-g68 wiring seam: the `TileDrawing.Draw` switch's drip arm
/// (`TileDrawing.cs:688-694`) as a per-frame world pass — the tile-cracks
/// precedent (`frame/draw.rs`, right after the cached tile range), so the
/// emission cadence is per-DRAW like the C#, never per-G6-rebuild, and the
/// `WorldFrameKey` stays untouched.
///
/// The C# walks `GetScreenDrawArea`'s rect per `Draw` call
/// (`TileDrawing.cs:503-505`: rows `firstTileY..lastTileY+4` exclusive,
/// cols `firstTileX-2..lastTileX+2` exclusive, area clamps 4 / max−4 at
/// `:6558-6573`). The port's stand-in rect is the `draw_tiles` view rect
/// (camera ± screen/2/zoom, ±1 tile) — the same convention the wind-grid
/// `EnsureWindGridSize` port uses; the deviation is recorded in ANALYSIS.
/// Row-major (y outer, x inner) like the C# loop, so the shared `_rand`
/// gate draws fire in the C#'s order.
///
/// The layer filter: the five drip kinds are non-solid (no
/// `Main.tileSolid` write, no `DrawTileInSolidLayer` row — `TileID.cs:516`
/// lists only 11/470/475/78/579), so they ride the `solidLayer: false`
/// `Draw` pass; the port's fused tile pass has no layer split, this scan
/// IS that pass's drip arm. `tile.active()` is the `:517` gate (the
/// actuated/inactive bit does NOT gate the C# emission — only `active()`).
///
/// RNG: `rand` is the App's PERSISTENT `TileDrawing._rand` stand-in
/// (`TileDrawing.cs:104/:191` — one stream for the renderer's lifetime;
/// the per-frame `FastRandom` pin in `draw_tiles` remains the leaf arm's
/// documented determinism stand-in until the d60 wiring reconciles it).
/// `main_rand` is the bare global `Main.rand` — the emission runs outside
/// every `SwapRandom` block (`Main.cs:17811-17812`).
#[allow(clippy::too_many_arguments)]
pub(crate) fn emit_liquid_drops_area(
    world: &World,
    camera: Vec2,
    zoom: f32,
    screen: Vec2,
    water_style: i32,
    zone_corrupt: bool,
    zone_crimson: bool,
    zone_hallow: bool,
    rand: &mut UnifiedRandom,
    main_rand: &mut UnifiedRandom,
    gore: &mut GorePool,
) {
    let half = screen * 0.5 / zoom;
    let x0 = (((camera.x - half.x) / TILE_PX).floor() as i32 - 1).clamp(4, world.width - 5);
    let x1 = (((camera.x + half.x) / TILE_PX).ceil() as i32 + 1).clamp(4, world.width - 5);
    let y0 = (((camera.y - half.y) / TILE_PX).floor() as i32 - 1).clamp(4, world.height - 5);
    let y1 = (((camera.y + half.y) / TILE_PX).ceil() as i32 + 1).clamp(4, world.height - 5);
    for ty in y0..=y1 {
        for tx in x0..=x1 {
            let Some(t) = world.tile(tx, ty) else {
                continue;
            };
            // `:517`: active tiles only; the switch arm (`:688-694`) is
            // the five drip kinds, then `continue` (no sprite here — the
            // tile itself draws in the cached range).
            if !t.active() || !matches!(t.kind, 373 | 374 | 375 | 461 | 709) {
                continue;
            }
            emit_liquid_drops(
                world,
                ty,
                tx,
                t.kind,
                water_style,
                zone_corrupt,
                zone_crimson,
                zone_hallow,
                rand,
                main_rand,
                gore,
            );
        }
    }
}

/// `TileDrawing.DrawTrees`: canopy and branch sprites anchored to the transparent
/// marker tiles of a tree. The pass scans a slightly larger rect than the tile loop
/// because an 80x80 canopy sticks out above and beside its anchor. The sway is
/// `GetWindCycle(x, y, _treeWindCounter)` (`TileDrawing.cs:8137-8245`), skipped
/// when the marker tile sits in front of a wall (`flag`, line 8073), and every
/// top/branch rolls for an ambient leaf (`EmitTreeLeaves`).
#[allow(clippy::too_many_arguments)]
pub(crate) fn draw_tree_foliage(
    r: &mut SpriteRenderer,
    world: &World,
    lookups: &assets::Lookups,
    light: &LightMap,
    style: &LightStyle,
    anim_time: f32,
    wind: &tr_core::wind::Wind,
    tree_counter: f32,
    rng: &mut FastRandom,
    main_rand: &mut UnifiedRandom,
    gore: &mut GorePool,
    // `TileDrawing._isActiveAndNotPaused` (`TileDrawing.cs:7781-7784`).
    emit_effects: bool,
    x0: i32,
    y0: i32,
    x1: i32,
    y1: i32,
) -> u32 {
    let fx0 = (x0 - 4).max(0);
    let fx1 = (x1 + 4).min(world.width - 1);
    let fy0 = (y0 - 5).max(0);
    let fy1 = (y1 + 1).min(world.height - 1);
    let mut quads = 0;
    for ty in fy0..=fy1 {
        for tx in fx0..=fx1 {
            let Some(t) = world.tile(tx, ty) else {
                continue;
            };
            if !t.active() {
                continue;
            }
            // Palm canopy (`TileDrawing.cs:8254-8297`): kind 323 frameX 88..=132
            // draws only the top canopy — no generic arms, no branch sprite.
            if t.kind == 323 && (88..=132).contains(&t.frame_x) {
                let biome = tr_core::drawdata::palm_tree_biome(world, tx, ty);
                let Some(palm) = palm_canopy_data(biome, t.frame_x) else {
                    continue;
                };
                let Some(uv) =
                    lookups.foliage_uv(palm.style, true, palm.src_x, palm.src_y, palm.w, palm.h)
                else {
                    continue;
                };
                let c = style.shade(light.get(tx, ty), tx, ty, anim_time);
                let color = [c[0], c[1], c[2], 1.0];
                // `bool flag = tile.wall > 0` (`TileDrawing.cs:8073`).
                let cycle = if t.wall > 0 {
                    0.0
                } else {
                    wind.cycle(tx, ty, tree_counter)
                };
                let (pos, rot, origin) = palm_placement(
                    tx as f32 * TILE_PX,
                    ty as f32 * TILE_PX,
                    t.frame_y as i32,
                    palm.w as f32,
                    palm.h as f32,
                    palm.x_off as f32,
                    palm.y_off as f32,
                    cycle,
                );
                r.push_sprite_origin_rot(
                    pos,
                    [palm.w as f32, palm.h as f32],
                    uv,
                    color,
                    rot,
                    origin,
                );
                quads += 1;
                continue;
            }
            if !tree::is_tree(t.kind) {
                continue;
            }
            let Some(part) = tree::TreePart::marker(t.frame_x, t.frame_y) else {
                continue;
            };
            let Some(f) = tree::foliage_at(world, tx, ty, part, t.frame_y, t.kind) else {
                continue;
            };
            if lookups.foliage.get(f.style).is_none_or(|s| s.is_none()) {
                continue;
            }
            let c = style.shade(light.get(tx, ty), tx, ty, anim_time);
            let color = [c[0], c[1], c[2], 1.0];
            let px = tx as f32 * TILE_PX;
            let py = ty as f32 * TILE_PX;
            // `bool flag = tile.wall > 0` (`TileDrawing.cs:8073`): a tree in
            // front of a wall does not sway.
            let cycle = if t.wall > 0 {
                0.0
            } else {
                wind.cycle(tx, ty, tree_counter)
            };
            match part {
                tree::TreePart::Top => {
                    let sx = f.frame as i32 * (f.top_w as i32 + 2);
                    if let Some(uv) =
                        lookups.foliage_uv(f.style, true, sx, 0, f.top_w as u32, f.top_h as u32)
                    {
                        // `TileDrawing.cs:8136-8149`: the top leans right with
                        // the sway, sinks by its magnitude, and rotates about
                        // the trunk anchor by `cycle * 0.08`.
                        let (pos, rot) = top_placement(px, py, cycle, f.top_w, f.top_h);
                        r.push_sprite_origin_rot(
                            pos,
                            [f.top_w, f.top_h],
                            uv,
                            color,
                            rot,
                            [f.top_w * 0.5, f.top_h],
                        );
                        quads += 1;
                        // `EmitTreeLeaves(x, y, x, floorY3)` — the top's grass
                        // column is its own (`TileDrawing.cs:8114-8121`).
                        emit_tree_leaves(
                            world, wind, tx, ty, tx, rng, main_rand, gore, emit_effects,
                        );
                    }
                }
                tree::TreePart::BranchLeft => {
                    if let Some(uv) =
                        lookups.foliage_uv(f.style, false, 0, f.frame as i32 * 42, 40, 40)
                    {
                        // `TileDrawing.cs:8181-8197` (frameX == 44): the branch
                        // leans away from the trunk for positive sway and
                        // rotates about its trunk-side anchor by
                        // `cycle * 0.06`.
                        let (pos, rot) = branch_placement(part, px, py, cycle);
                        r.push_sprite_origin_rot(pos, [40.0, 40.0], uv, color, rot, [40.0, 24.0]);
                        quads += 1;
                        // `EmitTreeLeaves(x, y, x + 1, floorY2)` — the grass
                        // column is the trunk to the right
                        // (`TileDrawing.cs:8159-8166`).
                        emit_tree_leaves(
                            world, wind, tx, ty, tx + 1, rng, main_rand, gore, emit_effects,
                        );
                    }
                }
                tree::TreePart::BranchRight => {
                    if let Some(uv) =
                        lookups.foliage_uv(f.style, false, 42, f.frame as i32 * 42, 40, 40)
                    {
                        // `TileDrawing.cs:8229-8245` (frameX == 66): mirrored —
                        // the branch leans left for negative sway, origin on
                        // its left edge.
                        let (pos, rot) = branch_placement(part, px, py, cycle);
                        r.push_sprite_origin_rot(pos, [40.0, 40.0], uv, color, rot, [0.0, 30.0]);
                        quads += 1;
                        // `EmitTreeLeaves(x, y, x - 1, floorY)` — the trunk to
                        // the left (`TileDrawing.cs:8207-8214`).
                        emit_tree_leaves(
                            world, wind, tx, ty, tx - 1, rng, main_rand, gore, emit_effects,
                        );
                    }
                }
            }
        }
    }
    quads
}

/// `TileDrawing.EmitTreeLeaves` (`TileDrawing.cs:7779-7891`): the per-part roll
/// for one ambient leaf. `(tile_x, tile_y)` is the tree-part marker,
/// `grass_x` the trunk column (`x`, `x+1` for a frameX-44 branch and `x-1` for
/// a frameX-66 branch); the grass row is the ground tile the
/// `GetCommonTreeFoliageData` walk found. W4-g68: the spawn goes straight
/// into the gore pool via `NewGoreDirect` — the `RandomVector2` velocity
/// argument is discarded by the SpecialAI-3 leaf init (`Gore.cs:1393`) but
/// its two `Main.rand` draws still fire (argument evaluation order).
#[allow(clippy::too_many_arguments)]
fn emit_tree_leaves(
    world: &World,
    wind: &tr_core::wind::Wind,
    tile_x: i32,
    tile_y: i32,
    grass_x: i32,
    rng: &mut FastRandom,
    main_rand: &mut UnifiedRandom,
    gore: &mut GorePool,
    emit_effects: bool,
) {
    if !emit_effects {
        // `TileDrawing.cs:7781-7784`: `_isActiveAndNotPaused`.
        return;
    }
    let Some(marker) = world.tile(tile_x, tile_y) else {
        return;
    };
    if marker.liquid > 0 {
        // `TileDrawing.cs:7787-7790`.
        return;
    }
    // `WorldGen.GetTreeLeaf` (`WorldGen.cs:29384-29598`) with the ground tile of
    // the grass column (the 100-tile walk of `GetCommonTreeFoliageData`).
    let Some(ground) = ground_tile(world, grass_x, tile_y) else {
        return;
    };
    let (_, pass_style) = tree_leaf_pass_style(marker.kind, marker.frame_y, ground, grass_x);
    // `TileDrawing.cs:7793-7814`: mushroom/boreal/ash trees and unknown styles
    // never shed ambient leaves; hallowed trees (917-925, 1113-1121) shed half
    // as often.
    let hallowed = matches!(pass_style, 917..=925 | 1113..=1121);
    if pass_style == -1 || matches!(pass_style, 912 | 913 | 1278) {
        return;
    }
    // `TileDrawing.cs:7816-7833`: the frequency chain.
    let branch = tile_x != grass_x;
    let mut frequency = wind::leaf_frequency(wind.speed_current);
    if hallowed {
        frequency /= 2;
    }
    if !wind.blows_at_height(tile_y) {
        frequency = 10000;
    }
    if branch {
        frequency *= 3;
    }
    if rng.next_range(0, frequency) != 0 {
        return;
    }
    // The spawn point (`TileDrawing.cs:7834-7886`).
    let mut pos = Vec2::new(tile_x as f32 * 16.0 + 8.0, tile_y as f32 * 16.0 + 8.0);
    if branch {
        pos.x += (tile_x - grass_x) as f32 * 12.0;
        let variant = match marker.frame_y {
            220 => 1,
            242 => 2,
            _ => 0,
        };
        if marker.frame_x == 66 {
            pos += match variant {
                1 => Vec2::new(0.0, -6.0),
                2 => Vec2::new(0.0, 8.0),
                _ => Vec2::new(0.0, -6.0),
            };
        } else {
            pos += match variant {
                1 => Vec2::new(2.0, -6.0),
                2 => Vec2::new(6.0, -6.0),
                _ => Vec2::new(0.0, 4.0),
            };
        }
    } else {
        pos += Vec2::new(-16.0, -16.0);
        if hallowed {
            // `Main.rand.Next(0, 28) * 4` (`TileDrawing.cs:7884`) — Main.rand,
            // NOT `_rand` (the two-stream census).
            pos.y -= main_rand.next_range(0, 28) as f32 * 4.0;
        }
    }
    // `if (!WorldGen.SolidTile(vector.ToTileCoordinates()))` — the strict
    // test (`WorldGen.cs:70111-70121`), `(int)x >> 4` tile coordinates.
    let tx = (pos.x as i32) >> 4;
    let ty = (pos.y as i32) >> 4;
    if !solid_tile(world.tile(tx, ty)) {
        // `Gore.NewGoreDirect(vector, Utils.RandomVector2(Main.rand, -2, 2),
        // passStyle, 0.7f + Main.rand.NextFloat() * 0.6f)` (`:7889`): the
        // argument draws fire in order — velocity X, velocity Y, then the
        // scale roll — before `NewGore`'s own internals.
        let num3 = 2.0_f32;
        let vel = Vec2::new(
            (num3 - -num3) * main_rand.next_float() + -num3,
            (num3 - -num3) * main_rand.next_float() + -num3,
        );
        let scale = 0.7 + main_rand.next_float() * 0.6;
        let n = gore.new_gore_direct(main_rand, world, pos, vel, pass_style, scale);
        gore.slot_mut(n).frame.current_column = marker.paint();
    }
}

/// `WorldGen.GetCommonTreeFoliageData`'s downward walk
/// (`WorldGen.cs:62882-63025`): the first non-tree tile in the column.
fn ground_tile(world: &World, x: i32, y: i32) -> Option<u16> {
    for dy in 0..100 {
        let t = world.tile(x, y + dy)?;
        if t.active() && !tree::is_tree(t.kind) {
            return Some(t.kind);
        }
    }
    None
}

/// `TileDrawing.cs:8254-8297`: the Palm canopy frame — the top sheet style, its
/// source rect (column from `frameX`, row from the biome) and size.
struct PalmCanopy {
    style: usize,
    src_x: i32,
    src_y: i32,
    w: u32,
    h: u32,
    x_off: i32,
    y_off: i32,
}

/// `TileDrawing.cs:8255-8279`: frameX 110 => column 1, 132 => 2, else 0; normal
/// palms (biome 0..=3) use style 15 / 80x80 / row `biome * 82` / offset 32,0;
/// Oasis palms (biome 4..=7) style 21 / 114x98 / row `(biome - 4) * 98` / 48,2.
/// Source X is `column * (width + 2)`. `None` outside the eight biomes.
fn palm_canopy_data(biome: i32, frame_x: i16) -> Option<PalmCanopy> {
    if !(0..=7).contains(&biome) {
        return None;
    }
    let column = match frame_x {
        110 => 1,
        132 => 2,
        _ => 0,
    };
    let (style, w, h, src_y, x_off, y_off) = if biome >= 4 {
        (21, 114, 98, (biome - 4) * 98, 48, 2)
    } else {
        (15, 80, 80, biome * 82, 32, 0)
    };
    Some(PalmCanopy {
        style,
        src_x: column * (w + 2),
        src_y,
        w: w as u32,
        h: h as u32,
        x_off,
        y_off,
    })
}

/// `TileDrawing.cs:8284-8291`: the canopy is drawn with `position` =
/// `(x*16 - xOff + frameY + w/2, y*16 + 16 + yOff)` and origin `(w/2, h)`, so
/// the unrotated top-left is `(x*16 - xOff + frameY, y*16 + 16 + yOff - h)`.
/// The sway adds `cycle*2` to X and `|cycle|*2` to Y and rotates
/// `cycle * 0.08`. Returns `(top_left, rotation, origin)`.
#[allow(clippy::too_many_arguments)]
fn palm_placement(
    px: f32,
    py: f32,
    frame_y: i32,
    w: f32,
    h: f32,
    x_off: f32,
    y_off: f32,
    cycle: f32,
) -> ([f32; 2], f32, [f32; 2]) {
    let pos = [
        px - x_off + frame_y as f32 + cycle * 2.0,
        py + 16.0 + y_off - h + cycle.abs() * 2.0,
    ];
    (pos, cycle * 0.08, [w * 0.5, h])
}

/// `TileDrawing.cs:8136-8149` (the `frameX == 22` top arm): the canopy centre
/// sits at `px + 8 - top_w/2 + cycle*2`, sinks by `py + 16 - top_h +
/// abs(cycle)*2`, and rotates `cycle * 0.08`. Returns `(position, rotation)`.
fn top_placement(px: f32, py: f32, cycle: f32, top_w: f32, top_h: f32) -> ([f32; 2], f32) {
    let pos = [
        px + 8.0 - top_w * 0.5 + cycle * 2.0,
        py + 16.0 - top_h + cycle.abs() * 2.0,
    ];
    let rot = cycle * 0.08;
    (pos, rot)
}

/// `TileDrawing.cs:8181-8197` (frameX == 44) and `:8229-8245` (frameX == 66):
/// the branch position starts at `(x*16 + 16, y*16 + 12)` (left) or
/// `(x*16, y*16 + 18)` (right, drawn at 40x40 from origin `(0, 30)`, which the
/// port folds into the same `py - 12` anchor); `X += num12` only when it leans
/// away from the trunk (`:8187-8190` left-positive, `:8235-8238` right-negative),
/// then `X ±= |num12| * 2` (`:8191` / `:8239`), rotating `num12 * num4`
/// (`cycle * 0.06`). Returns `(position, rotation)`.
fn branch_placement(part: tree::TreePart, px: f32, py: f32, cycle: f32) -> ([f32; 2], f32) {
    let mut pos = match part {
        tree::TreePart::BranchLeft => [px - 24.0, py - 12.0],
        tree::TreePart::BranchRight => [px, py - 12.0],
        _ => panic!("Invalid tree part for branch_placement: {part:?}"),
    };
    match part {
        tree::TreePart::BranchLeft => {
            if cycle > 0.0 {
                pos[0] += cycle;
            }
            pos[0] += cycle.abs() * 2.0;
        }
        tree::TreePart::BranchRight => {
            if cycle < 0.0 {
                pos[0] += cycle;
            }
            pos[0] -= cycle.abs() * 2.0;
        }
        _ => unreachable!(),
    }
    let rot = cycle * 0.06;
    (pos, rot)
}

fn tree_leaf_pass_style(top_kind: u16, frame_y: i16, ground: u16, x: i32) -> (i32, i32) {
    let mut tree_frame = match frame_y {
        220 => 1,
        242 => 2,
        _ => 0,
    };
    // `WorldGen.cs:29410-29434`: the top-tile type overrides (vanity/gem/ash).
    match top_kind {
        596 => return (tree_frame, 1248),
        616 => return (tree_frame, 1257),
        634 => return (tree_frame, 1278),
        583..=589 => return (tree_frame, 1249 + (top_kind - 583) as i32),
        _ => {}
    }
    let base = match ground {
        2 | 477 => 910,
        60 => 914,
        70 => 912,
        23 | 112 => 915,
        199 | 234 => 916,
        53 => 911,
        116 => 919,
        147 => 913,
        633 => 1278,
        109 | 492 => {
            // `WorldGen.cs:29467-29592`: the hallowed tables. `WorldGen
            // .hallowBG` is a generator property our worlds do not have; the
            // port's worlds never generate hallowed ground, so this arm is
            // data-complete but unreachable today.
            match x.rem_euclid(3) {
                1 => tree_frame += 3,
                2 => tree_frame += 6,
                _ => {}
            }
            let styles = [2, 1, 7, 4, 5, 6, 3, 8, 0];
            return (tree_frame, 917 + styles[tree_frame.clamp(0, 8) as usize]);
        }
        _ => return (tree_frame, -1),
    };
    (tree_frame, base)
}

#[cfg(test)]
mod tests {
    use super::*;

    const EPS: f32 = 1e-4;

    /// `WorldGen.GetGemTreeFoliageData` / `GetVanityTreeFoliageData` /
    /// `GetAshTreeFoliageData` (`WorldGen.cs:62801-62879`): the style, canopy
    /// size, and the ground each variant accepts.
    #[test]
    fn variant_foliage_styles_match_worldgen() {
        let matches = [
            (583, (22, 116.0, 96.0)),
            (584, (23, 116.0, 96.0)),
            (585, (24, 116.0, 96.0)),
            (586, (25, 116.0, 96.0)),
            (587, (26, 116.0, 96.0)),
            (588, (27, 116.0, 96.0)),
            (589, (28, 116.0, 96.0)),
            (596, (29, 118.0, 96.0)),
            (616, (30, 118.0, 96.0)),
            (634, (31, 116.0, 96.0)),
        ];
        for &(kind, (style, top_w, top_h)) in &matches {
            // `GemTreeGroundTest` accepts stone/moss, `VanityTreeGroundTest`
            // grass, `AshTreeGroundTest` only ash.
            let ground = match kind {
                583..=589 => 1,
                634 => 633,
                _ => 2,
            };
            let f = tree::Foliage::for_ground(kind, ground, 198).unwrap();
            assert_eq!(
                (f.style, f.top_w, f.top_h),
                (style, top_w, top_h),
                "kind {kind}"
            );
        }

        // Invalid variant -> None
        assert!(tree::Foliage::for_ground(596, 70, 198).is_none());
        assert!(tree::Foliage::for_ground(634, 70, 198).is_none());
        assert!(tree::Foliage::for_ground(583, 70, 198).is_none());

        // Common -> Some
        let common = tree::Foliage::for_ground(5, 2, 198).unwrap();
        assert_eq!(common.style, 0);
    }

    /// `TileDrawing.cs:8136-8149` (the `frameX == 22` top arm): the canopy
    /// centre sits at `px + 8 - top_w/2 + cycle*2`, sinks by
    /// `py + 16 - top_h + abs(cycle)*2`, and rotates `cycle * 0.08`.
    #[test]
    fn top_placement_leans_right_and_sinks_with_the_sway() {
        let (pos, rot) = super::top_placement(32.0, 64.0, 0.25, 114.0, 96.0);
        assert!((pos[0] - (32.0 + 8.0 - 57.0 + 0.5)).abs() < EPS);
        assert!((pos[1] - (64.0 + 16.0 - 96.0 + 0.5)).abs() < EPS);
        assert!((rot - 0.25 * 0.08).abs() < EPS);

        // A negative cycle shifts left and still *sinks* (abs on Y).
        let (pos, rot) = super::top_placement(32.0, 64.0, -0.5, 80.0, 140.0);
        assert!((pos[0] - (32.0 + 8.0 - 40.0 - 1.0)).abs() < EPS);
        assert!((pos[1] - (64.0 + 16.0 - 140.0 + 1.0)).abs() < EPS);
        assert!((rot - -0.5 * 0.08).abs() < EPS);

        // No wind, no offset; zero cycle is the vertical rest pose. The
        // top-left still carries the `position - origin` fold of
        // `x*16+8, y*16+16` with origin `(top_w/2, top_h)`.
        let (pos, rot) = super::top_placement(0.0, 0.0, 0.0, 80.0, 80.0);
        assert!((pos[0] - (0.0 + 8.0 - 40.0)).abs() < EPS);
        assert!((pos[1] - (0.0 + 16.0 - 80.0)).abs() < EPS);
        assert!(rot.abs() < EPS);
    }

    /// `TileDrawing.cs:8181-8197` (frameX == 44) and `:8229-8245` (frameX ==
    /// 66): both branch arms rotate about their trunk-side anchor by
    /// `cycle * 0.06`.
    #[test]
    fn branch_rotations_are_cycle_times_point_06() {
        for cycle in [-0.75, -0.1, 0.0, 0.2, 1.5] {
            let (_, rot_l) = super::branch_placement(tree::TreePart::BranchLeft, 48.0, 96.0, cycle);
            let (_, rot_r) =
                super::branch_placement(tree::TreePart::BranchRight, 48.0, 96.0, cycle);
            assert!((rot_l - cycle * 0.06).abs() < EPS);
            assert!((rot_r - cycle * 0.06).abs() < EPS);
        }
    }

    /// The branch placements lean away from the trunk with the sway
    /// (`TileDrawing.cs:8181-8197` / `:8229-8245`), mirrored per side.
    #[test]
    fn branch_positions_lean_away_from_the_trunk() {
        // Left branch anchored at `px - 24, py - 12`; positive sway leans
        // further left (away) by `cycle + abs(cycle)*2`.
        let (pos, _) = super::branch_placement(tree::TreePart::BranchLeft, 48.0, 96.0, 0.5);
        assert!((pos[0] - (48.0 - 24.0 + 0.5 + 1.0)).abs() < EPS);
        assert!((pos[1] - 84.0).abs() < EPS);
        // Negative sway: no direction term, only the magnitude push.
        let (pos, _) = super::branch_placement(tree::TreePart::BranchLeft, 48.0, 96.0, -0.5);
        assert!((pos[0] - (48.0 - 24.0 + 1.0)).abs() < EPS);

        // Right branch anchored at `px, py - 12`, mirrored.
        let (pos, _) = super::branch_placement(tree::TreePart::BranchRight, 48.0, 96.0, 0.5);
        assert!((pos[0] - (48.0 - 1.0)).abs() < EPS);
        let (pos, _) = super::branch_placement(tree::TreePart::BranchRight, 48.0, 96.0, -0.5);
        assert!((pos[0] - (48.0 - 0.5 - 1.0)).abs() < EPS);
        assert!((pos[1] - 84.0).abs() < EPS);
    }

    /// `TileDrawing.cs:8254-8297`: Palm canopy parameters.
    /// Normal Palm (biome 0..3): style 15, 80x80, source row biome*82, offsets 32/0.
    /// Oasis Palm (biome 4..7): style 21, 114x98, source row (biome-4)*98, offsets 48/2.
    #[test]
    fn palm_canopy_data_maps_biome_to_style_size_row_offsets() {
        // Normal palms
        let d = super::palm_canopy_data(0, 88).unwrap();
        assert_eq!(
            (d.style, d.w, d.h, d.src_y, d.x_off, d.y_off),
            (15, 80, 80, 0, 32, 0)
        );
        let d = super::palm_canopy_data(3, 88).unwrap();
        assert_eq!(
            (d.style, d.w, d.h, d.src_y, d.x_off, d.y_off),
            (15, 80, 80, 3 * 82, 32, 0)
        );
        // Oasis palms
        let d = super::palm_canopy_data(4, 88).unwrap();
        assert_eq!(
            (d.style, d.w, d.h, d.src_y, d.x_off, d.y_off),
            (21, 114, 98, 0, 48, 2)
        );
        let d = super::palm_canopy_data(7, 88).unwrap();
        assert_eq!(
            (d.style, d.w, d.h, d.src_y, d.x_off, d.y_off),
            (21, 114, 98, 3 * 98, 48, 2)
        );
        // Invalid biome returns None
        assert!(super::palm_canopy_data(-1, 88).is_none());
        assert!(super::palm_canopy_data(8, 88).is_none());
    }

    /// FrameX 88 => column 0, 110 => 1, 132 => 2; others 0.
    #[test]
    fn palm_canopy_column_from_frame_x() {
        assert_eq!(super::palm_canopy_data(0, 88).unwrap().src_x, 0);
        assert_eq!(super::palm_canopy_data(0, 110).unwrap().src_x, 1 * (80 + 2));
        assert_eq!(super::palm_canopy_data(0, 132).unwrap().src_x, 2 * (80 + 2));
        assert_eq!(super::palm_canopy_data(0, 100).unwrap().src_x, 0);
        // Oasis uses 114 width
        assert_eq!(
            super::palm_canopy_data(4, 110).unwrap().src_x,
            1 * (114 + 2)
        );
    }

    /// Palm placement (`TileDrawing.cs:8284-8297`): the drawn position is
    /// `(x*16 - xOff + frameY + w/2, y*16 + 16 + yOff)` with origin `(w/2, h)`,
    /// so the unrotated top-left is `(x*16 - xOff + frameY, y*16 + 16 + yOff - h)`;
    /// the sway adds `cycle*2` to X, `|cycle|*2` to Y and rotates `cycle * 0.08`.
    #[test]
    fn palm_placement_matches_csharp_formula() {
        // Biome 0 (normal), frameY = 0, cycle = 0.
        let (pos, rot, origin) =
            super::palm_placement(10.0 * 16.0, 20.0 * 16.0, 0, 80.0, 80.0, 32.0, 0.0, 0.0);
        assert!((pos[0] - (160.0 - 32.0)).abs() < EPS);
        assert!((pos[1] - (320.0 + 16.0 - 80.0)).abs() < EPS);
        assert!(rot.abs() < EPS);
        assert_eq!(origin, [40.0, 80.0]);

        // cycle = 1.0: X += 2, Y += 2, rot = 0.08.
        let (pos, rot, _) =
            super::palm_placement(10.0 * 16.0, 20.0 * 16.0, 0, 80.0, 80.0, 32.0, 0.0, 1.0);
        assert!((pos[0] - (160.0 - 32.0 + 2.0)).abs() < EPS);
        assert!((pos[1] - (320.0 + 16.0 - 80.0 + 2.0)).abs() < EPS);
        assert!((rot - 0.08).abs() < EPS);

        // cycle = -1.0: X -= 2, Y += 2 (abs), rot = -0.08.
        let (pos, rot, _) =
            super::palm_placement(10.0 * 16.0, 20.0 * 16.0, 0, 80.0, 80.0, 32.0, 0.0, -1.0);
        assert!((pos[0] - (160.0 - 32.0 - 2.0)).abs() < EPS);
        assert!((pos[1] - (320.0 + 16.0 - 80.0 + 2.0)).abs() < EPS);
        assert!((rot - -0.08).abs() < EPS);

        // frameY shifts X by its own value (`position4.X` gets `frameY2`).
        let (pos, _, _) =
            super::palm_placement(10.0 * 16.0, 20.0 * 16.0, 22, 80.0, 80.0, 32.0, 0.0, 0.0);
        assert!((pos[0] - (160.0 - 32.0 + 22.0)).abs() < EPS);

        // Oasis (biome 4): 114x98, offsets 48/2.
        let (pos, rot, origin) =
            super::palm_placement(10.0 * 16.0, 20.0 * 16.0, 0, 114.0, 98.0, 48.0, 2.0, 0.0);
        assert!((pos[0] - (160.0 - 48.0)).abs() < EPS);
        assert!((pos[1] - (320.0 + 16.0 + 2.0 - 98.0)).abs() < EPS);
        assert!(rot.abs() < EPS);
        assert_eq!(origin, [57.0, 98.0]);
    }

    // ---- W4-g68: the living-wood leaf shed (WRITTEN NOT RUN) ----

    /// A 64x64 world with one active tile helper.
    fn leaf_world() -> World {
        World::new("leaf".into(), 64, 64)
    }

    fn place(w: &mut World, x: i32, y: i32, kind: u16) {
        let i = w.index(x, y);
        w.tiles[i].kind = kind;
        w.tiles[i].flags |= tr_core::tile::FLAG_ACTIVE;
    }

    fn live_leaves(pool: &GorePool) -> Vec<usize> {
        (0..tr_core::gore::GORE_SLOTS)
            .filter(|&i| pool.slot(i).active)
            .collect()
    }

    /// `EmitLivingTreeLeaf` (`TileDrawing.cs:7549-7556`): the below-spawn
    /// fires first, the sideways `Next(2)` is UNCONDITIONAL — in dead wind
    /// (`num = 0`) it probes the tile's own cell, which is a 192 leaf block
    /// (walk-through), so BOTH gores spawn. The old inline arm gated the
    /// sideways on a non-zero wind pick — the W4-d60 flagged deviation.
    #[test]
    fn w4_g68_dead_wind_sheds_below_and_sideways() {
        let mut w = leaf_world();
        place(&mut w, 10, 10, 192);
        let mut pool = GorePool::new();
        // Scan seeds for the sideways-hit roll; the gate stream is the
        // caller's, so any seed with next_range(0,2)==0 exercises the arm.
        let mut hit = None;
        for seed in 0..64u64 {
            let mut probe = FastRandom::new(seed);
            if probe.next_range(0, 2) == 0 {
                hit = Some(seed);
                break;
            }
        }
        let mut rand = FastRandom::new(hit.expect("a sideways-hit seed"));
        let mut main_rand = UnifiedRandom::new(7);
        emit_living_tree_leaf(&w, 10, 10, 910, 0.0, &mut rand, &mut main_rand, &mut pool);
        let live = live_leaves(&pool);
        assert_eq!(live.len(), 2, "below + sideways in dead wind");
        // Below: (x*16, y*16+16); sideways num=0: (x*16+8, y*16+8).
        let below = pool.slot(live[0]);
        assert_eq!(below.position, Vec2::new(160.0, 176.0));
        assert_eq!(below.gore_type, 910);
        assert_eq!(below.velocity, Vec2::ZERO, "NewGorePerfect keeps zero");
        let side = pool.slot(live[1]);
        assert_eq!(side.position, Vec2::new(168.0, 168.0));
        assert_eq!(side.velocity, Vec2::ZERO);
    }

    /// The sideways roll MISSES (`Next(2) == 1`): only the below-spawn.
    #[test]
    fn w4_g68_the_sideways_roll_can_miss() {
        let mut w = leaf_world();
        place(&mut w, 10, 10, 192);
        let mut hit = None;
        // The LCG's first draw from a small seed is itself small, so the
        // high half only appears once the seed passes ~5.5k; scan wider.
        for seed in 0..10_000u64 {
            let mut probe = FastRandom::new(seed);
            if probe.next_range(0, 2) == 1 {
                hit = Some(seed);
                break;
            }
        }
        let mut rand = FastRandom::new(hit.expect("a sideways-miss seed"));
        let mut pool = GorePool::new();
        let mut main_rand = UnifiedRandom::new(7);
        emit_living_tree_leaf(&w, 10, 10, 910, 0.0, &mut rand, &mut main_rand, &mut pool);
        assert_eq!(live_leaves(&pool).len(), 1, "only the below-spawn");
    }

    /// `EmitLivingTreeLeaf_Below` (`:7564`): a strong RIGHT wind needs the
    /// two downwind cells (x+1, x+2 at y+1) open; one solid blocks the
    /// below-spawn, but the sideways arm (its own probe at y) still fires.
    #[test]
    fn w4_g68_strong_wind_needs_two_open_downwind_cells() {
        let mut w = leaf_world();
        place(&mut w, 10, 10, 192);
        place(&mut w, 12, 11, 1); // dirt at x+2, y+1 blocks the below arm
        let mut pool = GorePool::new();
        let mut main_rand = UnifiedRandom::new(7);
        // Force the sideways roll to miss so only the below arm is tested.
        let mut miss = None;
        // Same wide scan: the small-seed first draw cannot reach the high
        // half of `Next(2)`.
        for seed in 0..10_000u64 {
            let mut probe = FastRandom::new(seed);
            if probe.next_range(0, 2) == 1 {
                miss = Some(seed);
                break;
            }
        }
        let mut rand = FastRandom::new(miss.unwrap());
        emit_living_tree_leaf(&w, 10, 10, 910, 0.5, &mut rand, &mut main_rand, &mut pool);
        assert_eq!(live_leaves(&pool).len(), 0, "the blocked below arm sheds nothing");
        // Open both cells: the below-spawn returns (fresh miss-seed stream
        // so the sideways still stays out of the count).
        let i = w.index(12, 11);
        w.tiles[i].flags &= !tr_core::tile::FLAG_ACTIVE;
        let mut rand = FastRandom::new(miss.unwrap());
        emit_living_tree_leaf(&w, 10, 10, 910, 0.5, &mut rand, &mut main_rand, &mut pool);
        assert_eq!(live_leaves(&pool).len(), 1);
    }

    /// The sideways left-shift quirk (`:7586-7590`): wind < −0.2 gives
    /// `num = -1`, `num2 = -10` — position `x*16 + 8 − 4 − 10`.
    #[test]
    fn w4_g68_the_sideways_left_shift_quirk() {
        let mut w = leaf_world();
        place(&mut w, 10, 10, 192);
        // Block the below arm (solid directly under) so only sideways fires.
        place(&mut w, 10, 11, 1);
        let mut hit = None;
        for seed in 0..64u64 {
            let mut probe = FastRandom::new(seed);
            if probe.next_range(0, 2) == 0 {
                hit = Some(seed);
                break;
            }
        }
        let mut rand = FastRandom::new(hit.unwrap());
        let mut pool = GorePool::new();
        let mut main_rand = UnifiedRandom::new(7);
        emit_living_tree_leaf(&w, 10, 10, 910, -0.5, &mut rand, &mut main_rand, &mut pool);
        let live = live_leaves(&pool);
        assert_eq!(live.len(), 1);
        assert_eq!(
            pool.slot(live[0]).position,
            Vec2::new((10 * 16 + 8 - 4 - 10) as f32, (10 * 16 + 8) as f32)
        );
    }

    /// The paint column rides the gore frame (`:7566`/`:7590`):
    /// `Frame.CurrentColumn = Main.tile[x, y].color()`.
    #[test]
    fn w4_g68_the_paint_column_rides_the_frame() {
        let mut w = leaf_world();
        place(&mut w, 10, 10, 192);
        let i = w.index(10, 10);
        w.tiles[i].set_paint(5);
        let mut pool = GorePool::new();
        let mut main_rand = UnifiedRandom::new(7);
        emit_living_tree_leaf_below(&w, 10, 10, 910, 0.0, &mut main_rand, &mut pool);
        let live = live_leaves(&pool);
        assert_eq!(live.len(), 1);
        assert_eq!(pool.slot(live[0]).frame.current_column, 5);
    }

    /// The below arm refuses a liquid cell (`:7561` `tile.liquid <= 0`).
    #[test]
    fn w4_g68_liquid_below_blocks_the_shed() {
        let mut w = leaf_world();
        place(&mut w, 10, 10, 192);
        let i = w.index(10, 11);
        w.tiles[i].liquid = 100;
        let mut pool = GorePool::new();
        let mut main_rand = UnifiedRandom::new(7);
        emit_living_tree_leaf_below(&w, 10, 10, 910, 0.0, &mut main_rand, &mut pool);
        assert_eq!(live_leaves(&pool).len(), 0);
    }

    /// `EmitTreeLeaves`' two-stream census (`TileDrawing.cs:7884`, `:7889`):
    /// the hallowed height pick rolls `Main.rand`, NOT `_rand`; the spawn's
    /// `RandomVector2` pair + scale roll + `NewGore` internals all ride
    /// `Main.rand`. A forced frequency-1 gate isolates one spawn.
    #[test]
    fn w4_g68_emit_tree_leaves_rolls_main_rand_not_rand() {
        // A forest tree top marker (kind 5, frameX 22, frameY 198) over
        // grass (2) — passStyle 910, non-hallowed, so no height pick.
        let mut w = leaf_world();
        place(&mut w, 10, 20, 5);
        let i = w.index(10, 20);
        w.tiles[i].frame_x = 22;
        w.tiles[i].frame_y = 198;
        place(&mut w, 10, 24, 2); // the ground the walk finds
        let mut wind = tr_core::wind::Wind::new(7);
        wind.speed_current = 0.0;
        wind.world_surface = f64::INFINITY;
        // The gate: leaf_frequency(0) = 14000; find a FastRandom seed whose
        // first next_range(0, 14000) == 0 so the spawn fires deterministically.
        let mut gate_seed = None;
        for seed in 0..200_000u64 {
            let mut probe = FastRandom::new(seed);
            if probe.next_range(0, wind::leaf_frequency(0.0)) == 0 {
                gate_seed = Some(seed);
                break;
            }
        }
        let mut rng = FastRandom::new(gate_seed.expect("a gate-hit seed"));
        let rng_before = rng.seed;
        let mut main_rand = UnifiedRandom::new(7);
        let main_before = main_rand.clone();
        let mut pool = GorePool::new();
        emit_tree_leaves(&w, &wind, 10, 20, 10, &mut rng, &mut main_rand, &mut pool, true);
        // The gate consumed exactly one `_rand` draw; nothing else.
        let mut twin = FastRandom::new(gate_seed.unwrap());
        twin.next_range(0, wind::leaf_frequency(0.0));
        assert_eq!(rng.seed, twin.seed, "_rand: only the gate draw");
        assert_ne!(rng.seed, rng_before);
        // Main.rand advanced (the spawn's rolls), and a leaf gore exists.
        assert_ne!(main_rand, main_before, "Main.rand rolled the spawn");
        let live = live_leaves(&pool);
        assert_eq!(live.len(), 1);
        assert_eq!(pool.slot(live[0]).gore_type, 910);
        // The scale argument `0.7 + NextFloat()*0.6` is overwritten by the
        // SpecialAI-3 arm? No — the leaf arm does NOT touch scale, so the
        // spawn keeps the rolled scale in [0.7, 1.3).
        let s = pool.slot(live[0]).scale;
        assert!((0.7..1.3).contains(&s), "scale {s}");
    }

    /// The `_isActiveAndNotPaused` gate (`:7781-7784`): no draws, no spawn.
    #[test]
    fn w4_g68_the_paused_gate_draws_nothing() {
        let mut w = leaf_world();
        place(&mut w, 10, 20, 5);
        let mut wind = tr_core::wind::Wind::new(7);
        wind.world_surface = f64::INFINITY;
        let mut rng = FastRandom::new(3);
        let before = rng.seed;
        let mut main_rand = UnifiedRandom::new(7);
        let main_before = main_rand.clone();
        let mut pool = GorePool::new();
        emit_tree_leaves(&w, &wind, 10, 20, 10, &mut rng, &mut main_rand, &mut pool, false);
        assert_eq!(rng.seed, before, "the gate precedes every draw");
        assert_eq!(main_rand, main_before);
        assert_eq!(live_leaves(&pool).len(), 0);
    }

    // ---- W4-g68 deferred-4: EmitLiquidDrops (WRITTEN NOT RUN) ----

    /// Find a `_rand` seed whose first `Next(max)` is 0 (the gate hit).
    /// The gate stream is the persistent `UnifiedRandom` (`TileDrawing.
    /// _rand`, `TileDrawing.cs:104/:191`), not the per-frame FastRandom pin.
    fn gate_hit_seed(max: i32) -> i32 {
        (0..200_000i32)
            .find(|&s| UnifiedRandom::new(s).next_max(max) == 0)
            .expect("a gate-hit seed within 200k")
    }

    /// The plain-vine drip (`TileDrawing.cs:7594-7674`): kind 373 rolls
    /// `Next(120)` on `_rand`, spawns `NewGore((i*16, j*16), default, 706)`
    /// on `Main.rand` (water style 0 → 706), then zeroes the velocity —
    /// the drip sheet is 15 rows and lives 3× (`Gore.cs:1355-1360`).
    #[test]
    fn w4_g68_the_vine_tip_drips_water_gore_706() {
        let mut w = leaf_world();
        place(&mut w, 10, 10, 373);
        let mut rand = UnifiedRandom::new(gate_hit_seed(120));
        let mut main_rand = UnifiedRandom::new(7);
        let mut pool = GorePool::new();
        emit_liquid_drops(&w, 10, 10, 373, 0, false, false, false, &mut rand, &mut main_rand, &mut pool);
        let live = live_leaves(&pool);
        assert_eq!(live.len(), 1);
        let g = pool.slot(live[0]);
        assert_eq!(g.gore_type, 706);
        assert_eq!(g.position, Vec2::new(160.0, 160.0));
        assert_eq!(g.velocity, Vec2::ZERO, "the post-spawn *= 0");
        assert_eq!(g.frame.row_count, 15, "the IsDrip sheet");
        assert_eq!(g.time_left, tr_core::gore::GORE_TIME_DEFAULT * 3);
        assert!(g.behind_tiles);
    }

    /// The emitter overrides (`:7644-7671`): 374 → 716 (lava, `Next(240)`),
    /// 375 → 717 (honey, `Next(360)`), 709 → 1383 (candle); the 461 shimmer
    /// candle is 943 plain, biome-tinted 1160/1161/1162 with the LAST zone
    /// in C# order winning (hallow over crimson over corrupt).
    #[test]
    fn w4_g68_the_drip_type_overrides_and_shimmer_zones() {
        let mut w = leaf_world();
        for (kind, gate, want) in [(374u16, 240, 716), (375, 360, 717), (709, 120, 1383)] {
            place(&mut w, 10, 10, kind);
            let mut rand = UnifiedRandom::new(gate_hit_seed(gate));
            let mut main_rand = UnifiedRandom::new(7);
            let mut pool = GorePool::new();
            emit_liquid_drops(&w, 10, 10, kind, 0, false, false, false, &mut rand, &mut main_rand, &mut pool);
            let live = live_leaves(&pool);
            assert_eq!(live.len(), 1, "kind {kind}");
            assert_eq!(pool.slot(live[0]).gore_type, want, "kind {kind}");
        }
        place(&mut w, 10, 10, 461);
        for (zones, want) in [
            ((false, false, false), 943),
            ((true, false, false), 1160),
            ((false, true, false), 1161),
            ((true, true, true), 1162), // hallow last
        ] {
            let mut rand = UnifiedRandom::new(gate_hit_seed(360));
            let mut main_rand = UnifiedRandom::new(7);
            let mut pool = GorePool::new();
            emit_liquid_drops(&w, 10, 10, 461, 0, zones.0, zones.1, zones.2, &mut rand, &mut main_rand, &mut pool);
            let live = live_leaves(&pool);
            assert_eq!(live.len(), 1);
            assert_eq!(pool.slot(live[0]).gore_type, want, "zones {zones:?}");
        }
        // Water styles (`:7628-7643`): 13/14 → 706, 12 → 1147, >1 → 705+style.
        place(&mut w, 10, 10, 373);
        for (style, want) in [(13, 706), (14, 706), (12, 1147), (5, 710)] {
            let mut rand = UnifiedRandom::new(gate_hit_seed(120));
            let mut main_rand = UnifiedRandom::new(7);
            let mut pool = GorePool::new();
            emit_liquid_drops(&w, 10, 10, 373, style, false, false, false, &mut rand, &mut main_rand, &mut pool);
            let live = live_leaves(&pool);
            assert_eq!(live.len(), 1);
            assert_eq!(pool.slot(live[0]).gore_type, want, "style {style}");
        }
    }

    /// The wet-tile refusal draws NOTHING (`:7609`'s `liquid != 0`
    /// short-circuit precedes the `Next`), and the anti-cluster box
    /// (`:7613-7625`) refuses a second drip near a live one: the box is
    /// x−34..x+50, y−100..y+300 around the tile.
    #[test]
    fn w4_g68_wet_tiles_and_nearby_drips_refuse() {
        let mut w = leaf_world();
        place(&mut w, 10, 10, 373);
        let i = w.index(10, 10);
        w.tiles[i].liquid = 100;
        let mut rand = UnifiedRandom::new(gate_hit_seed(120));
        let before = rand.clone();
        let mut main_rand = UnifiedRandom::new(7);
        let main_before = main_rand.clone();
        let mut pool = GorePool::new();
        emit_liquid_drops(&w, 10, 10, 373, 0, false, false, false, &mut rand, &mut main_rand, &mut pool);
        assert_eq!(rand, before, "the wet refusal precedes the roll");
        assert_eq!(main_rand, main_before);
        assert_eq!(live_leaves(&pool).len(), 0);
        // Dry again: the first drip lands, the second (one tile right,
        // inside the ±34 px box) refuses.
        w.tiles[i].liquid = 0;
        let mut rand = UnifiedRandom::new(gate_hit_seed(120));
        emit_liquid_drops(&w, 10, 10, 373, 0, false, false, false, &mut rand, &mut main_rand, &mut pool);
        assert_eq!(live_leaves(&pool).len(), 1);
        let mut rand = UnifiedRandom::new(gate_hit_seed(120));
        emit_liquid_drops(&w, 10, 11, 373, 0, false, false, false, &mut rand, &mut main_rand, &mut pool);
        assert_eq!(live_leaves(&pool).len(), 1, "the box refuses the neighbour");
        // Outside the box BELOW (the rect ends at y*16 + 300; row 30 starts
        // at 480 > 460): a second drip lands. The C# argument order is
        // (j=row, i=col).
        let mut rand = UnifiedRandom::new(gate_hit_seed(120));
        emit_liquid_drops(&w, 30, 10, 373, 0, false, false, false, &mut rand, &mut main_rand, &mut pool);
        assert_eq!(live_leaves(&pool).len(), 2);
    }

    // ---- W4-g68 wiring wave: the per-frame area driver (WRITTEN NOT RUN) ----

    /// The wiring seam (`TileDrawing.cs:688-694` via the `Draw` walk): the
    /// area scan finds ONLY active drip-kind tiles, feeds them in row-major
    /// order on the SHARED `_rand` stream, and the emission is per-call —
    /// two calls roll two gates (the per-draw cadence, not per-rebuild).
    #[test]
    fn w4_g68_the_area_driver_scans_drip_kinds_row_major() {
        let mut w = leaf_world();
        // Two drip emitters in different rows/cols + decoys: an inactive
        // 373, a non-drip kind, a wet 373 (its refusal still draws nothing).
        place(&mut w, 12, 10, 373);
        place(&mut w, 10, 12, 374);
        place(&mut w, 14, 10, 373);
        let i = w.index(14, 10);
        w.tiles[i].flags &= !tr_core::tile::FLAG_ACTIVE; // inactive: skipped
        place(&mut w, 11, 11, 1); // stone: not a drip kind
        place(&mut w, 13, 12, 375);
        let i = w.index(13, 12);
        w.tiles[i].liquid = 200; // wet: the refusal precedes the roll
        // The shared gate stream: seed so the FIRST Next(120) hits (the
        // row-10 col-12 vine). The 374's Next(240) and the wet 375 draw
        // whatever follows — the census pins the stream SHAPE, not luck.
        let mut rand = UnifiedRandom::new(gate_hit_seed(120));
        let mut main_rand = UnifiedRandom::new(7);
        let mut pool = GorePool::new();
        let cam = Vec2::new(11.0 * 16.0, 11.0 * 16.0);
        let screen = Vec2::new(640.0, 480.0);
        emit_liquid_drops_area(
            &w, cam, 1.0, screen, 0, false, false, false,
            &mut rand, &mut main_rand, &mut pool,
        );
        let live = live_leaves(&pool);
        assert_eq!(live.len(), 1, "only the dry active vine hit its gate");
        assert_eq!(pool.slot(live[0]).gore_type, 706);
        assert_eq!(pool.slot(live[0]).position, Vec2::new(192.0, 160.0));
        // The stream advanced past the first gate: the 374 (row 12) and
        // the wet-375 refusal ordering rode the SAME stream — a twin that
        // draws Next(120) then Next(240) matches (the wet tile draws
        // NOTHING, so no third gate).
        let mut twin = UnifiedRandom::new(gate_hit_seed(120));
        twin.next_max(120);
        twin.next_max(240);
        assert_eq!(rand, twin, "row-major shared-stream order: 373 then 374, wet 375 draws nothing");
        // A second call = a second draw: fresh gates on the SAME streams.
        let before = rand.clone();
        emit_liquid_drops_area(
            &w, cam, 1.0, screen, 0, false, false, false,
            &mut rand, &mut main_rand, &mut pool,
        );
        assert_ne!(rand, before, "the per-draw cadence re-rolls every call");
    }
}
