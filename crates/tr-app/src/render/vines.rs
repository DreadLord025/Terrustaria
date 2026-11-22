//! `TileDrawing.DrawVines` / `DrawVineStrip` (S1-G-1 wave A): the deferred
//! vine-strip pass. C# never draws a `TileID.Sets.VineThreads` cell through
//! `DrawSingleTile` — the draw-area switch crawls to the strip's root and
//! registers a `TileCounterType.Vine` special point (`TileDrawing.cs:536-545`,
//! `CrawlToTopOfVineAndAddSpecialPoint` `:744-762`), and `PostDrawTiles` draws
//! the whole strip after `DrawTrees` (`TileDrawing.cs:428-429`, `DrawVines`
//! `:8890-8901`, `DrawVineStrip` `:9034-9132`).
//!
//! The port keeps the fused row-major tile pass: the vine cells are skipped
//! inline (their roots collected in draw-area order, deduplicated like
//! `_vineRootsPositions`), and this pass runs right after the foliage pass —
//! the C#'s `DrawTrees` → `DrawVines` order. The reverse-vine (549) rising
//! strip (`DrawReverseVines`/`DrawRisingVineStrip`, `TileDrawing.cs:8903-8914`,
//! `:9134-9193`) lands with W4-g83 below — WIRED on this branch into the
//! fused pass's own uncontested seams (`tiles.rs`'s root collection + the
//! `draw_vines` tail, the C# `case 549` switch arm `:546-548` and the
//! `PostDrawTiles` `:430` position); the g31-union tile-pass rebind is
//! `docs/research/123-vine-wire-contract.md` §1. The multi-tile-vine pass
//! (chandeliers/banners, `DrawMultiTileVines(:InWind)`) is the furniture
//! lane (d57) and stays out.
//!
//! Deviations (docs/ANALYSIS.md "The vine strip pass"): the `NextNatureRenderer`
//! LitNature dawn/dusk shader is not ported — segments draw plain, like its
//! `DrawWithoutShader` arm (`NextNatureRenderer.cs:111-119`); the `seam_bottom`
//! flag is computed and carried for that future port. The `biomeSight` recolour
//! + its `Dust.NewDust` roll (`TileDrawing.cs:9089-9115`) are skipped (no biome
//! sight in the port; adding the roll would change the RNG stream order).
use crate::assets;
use crate::render::entities::flip_uv;
use crate::render::tiles::resolve_tile_frame;
use std::collections::HashSet;
use tr_core::drawdata::{draw_data, TileDrawData};
use tr_core::lighting::{LightMap, LightStyle};
use tr_core::tiledata as td;
use tr_core::wind::Wind;
use tr_core::World;
use tr_render::{Atlas, SpriteRenderer};

/// `TileID.Sets.VineThreads` (`TileID.cs:544`):
/// `CreateBoolSet(382, 62, 115, 205, 52, 528, 636, 638)`.
pub(crate) const VINE_THREADS: [u16; 8] = [382, 62, 115, 205, 52, 528, 636, 638];

/// `TileID.Sets.VineThreads[type]`.
#[inline]
pub(crate) fn is_vine_thread(kind: u16) -> bool {
    VINE_THREADS.contains(&kind)
}

/// `CrawlToTopOfVineAndAddSpecialPoint` (`TileDrawing.cs:744-762`): crawl up
/// from the vine cell until a tile whose bottom edge accepts an attachment
/// (or an inactive cell) is found; the root is the row below it. The root is
/// registered once per frame (`_vineRootsPositions.Contains`, `:757-760`) —
/// the caller's set is the per-frame list `ClearSpecialBlockCounts` resets
/// (`TileDrawing.cs:390`).
pub(crate) fn collect_vine_root(
    world: &World,
    x: i32,
    j: i32,
    seen: &mut HashSet<(i32, i32)>,
    roots: &mut Vec<(i32, i32)>,
) {
    let mut y = j;
    // `for (int num = j - 1; num > 0; num--)` (`:747`).
    for num in (1..j).rev() {
        // `WorldGen.BottomEdgeCanBeAttachedTo(i, num) || !tile.active()`
        // (`:750`); an out-of-world cell reads as attachable, like the C#'s
        // `tile == null` arm (`WorldGen.cs:70445-70448`).
        let attach = tr_core::torch::bottom_edge_can_be_attached_to(world, x, num);
        let active = world.tile(x, num).is_some_and(|t| t.active());
        if attach || !active {
            y = num + 1;
            break;
        }
    }
    if seen.insert((x, y)) {
        roots.push((x, y));
    }
}

/// One drawn segment of a vine strip — the inputs of the `DrawNature` call
/// (`TileDrawing.cs:9124`). Pure world/wind state, so the strip maths is
/// unit-testable without a GPU.
#[derive(Clone, Copy, Debug)]
pub(crate) struct VineSegment {
    pub kind: u16,
    pub x: i32,
    pub y: i32,
    pub frame_x: i16,
    pub frame_y: i16,
    /// `GetTileDrawData`'s record for the cell (`:9082`).
    pub dd: TileDrawData,
    /// `halfBrickHeight` (`TileDrawing.cs:6045-6048`): 8 on a half-brick.
    pub half: i16,
    /// The XNA `position` — where the rotation `origin` lands (`:9083`,
    /// the running `vector` of `:9038`/`:9130`).
    pub anchor: [f32; 2],
    /// `num7 = num2 * num4 * windCycle + num5` (`:9088`).
    pub rot: f32,
    /// `IsVisible(tile)` (`:9121`): an Echo-coated segment draws nothing but
    /// the strip still walks through it.
    pub visible: bool,
    /// `tile.fullbrightBlock()` → `Color.White` (`:9084-9087`).
    pub fullbright: bool,
    pub paint: u8,
    /// `seams: SideFlags.Bottom` when the next cell is a vine thread too
    /// (`:9123-9124`) — consumed only by the unported LitNature shader.
    pub seam_bottom: bool,
}

/// `DrawVineStrip` (`TileDrawing.cs:9034-9132`), maths only: walk down from
/// the root while the cells are active vine threads, accumulate the sway
/// rotation and the `WindGrid` push, and advance the anchor 16 px along the
/// rotated down-direction each segment.
pub(crate) fn vine_strip_segments(
    world: &World,
    wind: &Wind,
    vine_counter: f32,
    x: i32,
    start_y: i32,
) -> Vec<VineSegment> {
    let mut out = Vec::new();
    // `vector = new Vector2(x * 16 + 8, startY * 16 - 2)` (`:9038`).
    let mut anchor = [x as f32 * 16.0 + 8.0, start_y as f32 * 16.0 - 2.0];
    // `num3 = MathHelper.Lerp(0.2f, 1f, |WindForVisuals| / 1.2f)` (`:9039`) —
    // MathHelper.Lerp does NOT clamp, so a 1.24 storm overshoots past 1.
    let num3 = 0.2 + (1.0 - 0.2) * (wind.speed_current.abs() / 1.2);
    // `num4 = -0.08f * num3` (`:9040`).
    let mut num4 = -0.08 * num3;
    // `GetWindCycle(x, startY, _vineWindCounter)` (`:9041`).
    let wind_cycle = wind.cycle(x, start_y, vine_counter);
    // `num5`/`num6`: the accumulated grid push and the previous cell's push
    // (`:9042-9043`). The C#'s `num` segment counter (`:9036`, `:9056`) is
    // never read by the maths — only `num2` (the wind-exposed count) is.
    let mut num5 = 0.0f32;
    let mut num6 = 0.0f32;
    let mut num2 = 0i32;
    // `for (int i = startY; i < Main.maxTilesY - 10; i++)` (`:9044`).
    for i in start_y..world.height - 10 {
        let Some(tile) = world.tile(x, i) else {
            break; // `tile == null` (`:9047-9050`).
        };
        // `!tile.active() || !TileID.Sets.VineThreads[type]` (`:9052`).
        if !tile.active() || !is_vine_thread(tile.kind) {
            break;
        }
        // The per-segment sway ladder (`:9057-9064`): the exposed-count
        // thresholds widen the rotation step as the strip hangs lower.
        if num2 >= 5 {
            num4 += 0.0075 * num3;
        }
        if num2 >= 2 {
            num4 += 0.0025;
        }
        // `WallID.Sets.AllowsWind[tile.wall]` above the surface (below it in
        // a remix world) counts as wind-exposed (`:9065-9075`).
        let exposed = if wind.remix_world {
            (i as f64) > wind.world_surface
        } else {
            (i as f64) < wind.world_surface
        };
        if tr_core::wind::wall_allows_wind(tile.wall) && exposed {
            num2 += 1;
        }
        // `GetWindGridPush(x, i, 20, 0.01f)` (`:9076`) and the push
        // accumulator (`:9077-9078`): a live push subtracts, a dead one
        // bounces the accumulated sway by -0.78.
        let push = wind.get_wind_grid_push(x, i, 20, 0.01);
        num5 = if push != 0.0 || num6 == 0.0 {
            num5 - push
        } else {
            num5 * -0.78
        };
        num6 = push;
        // `GetTileDrawData` (`:9082`) — the strip never runs the
        // `DrawSingleTile` positional switch, so no world corrections.
        let solid_above = world.tile(x, i - 1).is_some_and(|t| {
            t.active() && td::TILE_SOLID.has(t.kind) && !td::TILE_SOLID_TOP.has(t.kind)
        });
        let dd = draw_data(tile.kind, x, i, tile.frame_x, tile.frame_y, 0, solid_above);
        let half = if tile.half_brick() { 8 } else { 0 };
        // `num7 = num2 * num4 * windCycle + num5` (`:9088`).
        let rot = num2 as f32 * num4 * wind_cycle + num5;
        let below = world.tile(x, i + 1);
        out.push(VineSegment {
            kind: tile.kind,
            x,
            y: i,
            frame_x: tile.frame_x,
            frame_y: tile.frame_y,
            dd,
            half,
            anchor,
            rot,
            visible: !world.invisible_block(x, i),
            fullbright: tile.fullbright_block(),
            paint: tile.paint(),
            seam_bottom: below.is_some_and(|t| t.active() && is_vine_thread(t.kind)),
        });
        // `vector += (num7 + PI/2).ToRotationVector2() * 16f` (`:9130`,
        // `Utils.cs:1665-1668`: `(cos, sin)`).
        let (s, c) = (rot + std::f32::consts::FRAC_PI_2).sin_cos();
        anchor = [anchor[0] + c * 16.0, anchor[1] + s * 16.0];
    }
    out
}

/// `DrawVines` (`TileDrawing.cs:8890-8901`): one `DrawVineStrip` per
/// registered root, in registration (draw-area row-major) order. Returns the
/// quad count.
#[allow(clippy::too_many_arguments)]
pub(crate) fn draw_vines(
    r: &mut SpriteRenderer,
    world: &World,
    lookups: &mut assets::Lookups,
    atlas: Option<&Atlas>,
    light: &LightMap,
    style: &LightStyle,
    anim_time: f32,
    wind: &Wind,
    vine_counter: f32,
    roots: &[(i32, i32)],
) -> u32 {
    let mut quads = 0u32;
    for &(x, start_y) in roots {
        let segments = vine_strip_segments(world, wind, vine_counter, x, start_y);
        for seg in segments {
            // `GetTileDrawTexture` null → `break` (`:9116-9120`); the port's
            // unframed-cell guard (`resolve_tile_frame`) is the same "no
            // texture" state.
            let Some(tf) = resolve_tile_frame(
                lookups,
                atlas,
                false,
                0,
                seg.kind,
                seg.frame_x,
                seg.frame_y,
                &seg.dd,
            ) else {
                break;
            };
            if !seg.visible {
                continue; // `IsVisible` (`:9121`) — the walk still advances.
            }
            // `Lighting.GetColor(x, i)` (`:9081`) — the strip applies NO
            // `GetFinalLight` actuation ghost; fullbright → White
            // (`:9084-9087`).
            let l = if seg.fullbright {
                [1.0; 3]
            } else {
                light.get(seg.x, seg.y)
            };
            let tint = style.shade(l, seg.x, seg.y, anim_time);
            let w = seg.dd.width as f32;
            let h = (seg.dd.height - seg.half as u16) as f32;
            // `origin = (tileWidth / 2, halfBrickHeight - tileTop)`
            // (`:9124`); XNA places the origin AT `position`, so the
            // unrotated top-left is `anchor - origin`.
            let origin = [w * 0.5, (seg.half - seg.dd.top) as f32];
            let pos = [seg.anchor[0] - origin[0], seg.anchor[1] - origin[1]];
            let fx = seg.dd.frame_x_override.unwrap_or(seg.frame_x) + seg.dd.add_fr_x;
            let fy = seg.dd.frame_y_override.unwrap_or(seg.frame_y) + seg.dd.add_fr_y;
            r.push_sprite_origin_rot_paint(
                pos,
                [w, h],
                flip_uv(tf.uv, seg.dd.flip),
                [tint[0], tint[1], tint[2], 1.0],
                seg.rot,
                origin,
                seg.paint,
            );
            quads += 1;
            // The glowmask arm (`:9125-9128`): only case 638 sets a
            // `glowTexture` in the vine family — `GlowMask[327]`, the same
            // rect, `Color.Lerp(White, tileLight, 0.75)`
            // (`TileDrawing.cs:6080-6084`). The glow texture is never painted
            // (`DrawNatureGlowmask` draws the raw sheet).
            if seg.kind == 638 {
                if let Some(glow) =
                    lookups.glow_uv("Glow_327", fx as i32, fy as i32, w as u32, h as u32)
                {
                    let gc = [
                        1.0 + (tint[0] - 1.0) * 0.75,
                        1.0 + (tint[1] - 1.0) * 0.75,
                        1.0 + (tint[2] - 1.0) * 0.75,
                        1.0,
                    ];
                    r.push_sprite_origin_rot(
                        pos,
                        [w, h],
                        flip_uv(glow, seg.dd.flip),
                        gc,
                        seg.rot,
                        origin,
                    );
                    quads += 1;
                }
            }
        }
    }
    quads
}

/// `TileID.Sets.ReverseVineThreads[type]` (`TileID.cs:545`,
/// `CreateBoolSet(549)`): the seaweed is the only rising vine.
#[inline]
pub(crate) fn is_reverse_vine_thread(kind: u16) -> bool {
    td::SET_REVERSE_VINE_THREADS.has(kind)
}

/// `CrawlToBottomOfReverseVineAndAddSpecialPoint` (`TileDrawing.cs:764-782`):
/// crawl DOWN from the reverse-vine cell until a tile whose top edge accepts
/// an attachment (or an inactive cell) is found; the root is the row ABOVE it
/// (`y = k - 1`, `:772`). The root registers once per frame
/// (`_reverseVineRootsPositions.Contains`, `:777-780`) — the caller's set is
/// the per-frame list `ClearSpecialBlockCounts` resets (`TileDrawing.cs:391`).
///
/// Unlike the forward crawl (`:747` starts at `j - 1`), the reverse crawl
/// starts AT `j` (`for (int k = j; k < Main.maxTilesY; k++)`, `:767`) and
/// runs to `maxTilesY` (no `- 1`); if no break fires the root stays `j`.
pub(crate) fn collect_reverse_vine_root(
    world: &World,
    x: i32,
    j: i32,
    seen: &mut HashSet<(i32, i32)>,
    roots: &mut Vec<(i32, i32)>,
) {
    let mut y = j;
    // `for (int k = j; k < Main.maxTilesY; k++)` (`:767`).
    for k in j..world.height {
        // `WorldGen.TopEdgeCanBeAttachedTo(i, k) || !tile.active()` (`:770`);
        // an out-of-world cell reads as attachable (the `tile == null` arm,
        // `WorldGen.cs:70385-70388`).
        let attach = tr_core::torch::top_edge_can_be_attached_to(world, x, k);
        let active = world.tile(x, k).is_some_and(|t| t.active());
        if attach || !active {
            y = k - 1;
            break;
        }
    }
    if seen.insert((x, y)) {
        roots.push((x, y));
    }
}

/// One drawn segment of a rising vine strip — the inputs of the `DrawNature`
/// call (`TileDrawing.cs:9188`). Pure world/wind state, so the strip maths is
/// unit-testable without a GPU.
#[derive(Clone, Copy, Debug)]
pub(crate) struct RisingVineSegment {
    pub kind: u16,
    pub x: i32,
    pub y: i32,
    pub frame_x: i16,
    pub frame_y: i16,
    /// `GetTileDrawData`'s record for the cell (`:9173`).
    pub dd: TileDrawData,
    /// `halfBrickHeight` (`TileDrawing.cs:6045-6048`): 8 on a half-brick.
    pub half: i16,
    /// The XNA `position` — where the rotation `origin` lands (`:9174`, the
    /// running `vector` of `:9138`/`:9190`).
    pub anchor: [f32; 2],
    /// `num8 = num2 * (0 - num4) * windCycle + num5` (`:9179`).
    pub rot: f32,
    /// `IsVisible(tile)` (`:9185`): an Echo-coated segment draws nothing but
    /// the strip still walks through it.
    pub visible: bool,
    /// `tile.fullbrightBlock()` → `Color.White` (`:9175-9178`).
    pub fullbright: bool,
    pub paint: u8,
    /// `seams: SideFlags.Top` when the cell ABOVE is a reverse vine too
    /// (`:9187-9188`) — consumed only by the unported LitNature shader.
    pub seam_top: bool,
}

/// `DrawRisingVineStrip` (`TileDrawing.cs:9134-9193`), maths only: walk UP
/// from the root while the cells are active reverse-vine threads, accumulate
/// the sway rotation and the `WindGrid` push, and advance the anchor 16 px
/// along the rotated up-direction each segment.
///
/// Differences from the hanging [`vine_strip_segments`], all verbatim:
/// * the anchor starts at `(x*16+8, startY*16+16+2)` (`:9138`, +16+2 not -2);
/// * the walk runs `num7 > 10` descending (`:9144`, not `< maxTilesY-10`);
/// * the exposure gate has NO remix arm — always `num7 < worldSurface`
///   (`:9163`);
/// * the grid push is `GetWindGridPush(x, num7, 40, -0.004f)` (`:9167`, the
///   40-window / negative force, not 20 / 0.01);
/// * the rotation negates the ladder: `num2 * (0 - num4) * cycle` (`:9179`);
/// * the anchor advances along `(rot - PI/2)` (`:9190`, not `+ PI/2`);
/// * the glow out-params are discarded (`:9173` `out var _`) — the rising
///   strip draws NO glowmask.
///
/// `show_invisible` is the g31 keep-one delta (contract §1.2, wire phase 2
/// seam 4): the caller's `Main.ShouldShowInvisibleBlocksAndWalls()` override
/// rides the FULL `IsVisible` read (`TileDrawing.cs:1411-1432` — the platform
/// row-48 + the 541/631 invisible-wiring arms always ride the flag), not the
/// bare `invisible_block` probe. The interim fused-pass caller passes
/// `false` (the pre-wire behaviour); the g31 drain binds its own
/// `show_invisible` at the union merge.
pub(crate) fn rising_vine_strip_segments(
    world: &World,
    wind: &Wind,
    vine_counter: f32,
    x: i32,
    start_y: i32,
    show_invisible: bool,
) -> Vec<RisingVineSegment> {
    let mut out = Vec::new();
    // `vector = new Vector2(x * 16 + 8, startY * 16 + 16 + 2)` (`:9138`).
    let mut anchor = [x as f32 * 16.0 + 8.0, start_y as f32 * 16.0 + 16.0 + 2.0];
    // `num3 = MathHelper.Lerp(0.2f, 1f, |WindForVisuals| / 1.2f)` (`:9139`).
    let num3 = 0.2 + (1.0 - 0.2) * (wind.speed_current.abs() / 1.2);
    // `num4 = -0.08f * num3` (`:9140`).
    let mut num4 = -0.08 * num3;
    // `GetWindCycle(x, startY, _vineWindCounter)` (`:9141`).
    let wind_cycle = wind.cycle(x, start_y, vine_counter);
    let mut num5 = 0.0f32;
    let mut num6 = 0.0f32;
    let mut num2 = 0i32;
    // `for (int num7 = startY; num7 > 10; num7--)` (`:9144`).
    let mut y = start_y;
    while y > 10 {
        let Some(tile) = world.tile(x, y) else {
            // `tile == null`: the C# `if (tile != null)` skips the body but
            // the loop still decrements (`:9146-9147`, `:9191`); a null cell
            // in the port's bounded world ends the strip.
            break;
        };
        // `!tile.active() || !TileID.Sets.ReverseVineThreads[type]` (`:9150`).
        if !tile.active() || !is_reverse_vine_thread(tile.kind) {
            break;
        }
        // The per-segment sway ladder (`:9155-9162`), same thresholds.
        if num2 >= 5 {
            num4 += 0.0075 * num3;
        }
        if num2 >= 2 {
            num4 += 0.0025;
        }
        // `WallID.Sets.AllowsWind[tile.wall] && num7 < worldSurface`
        // (`:9163`) — NO remix arm here.
        let exposed = (y as f64) < wind.world_surface;
        if tr_core::wind::wall_allows_wind(tile.wall) && exposed {
            num2 += 1;
        }
        // `GetWindGridPush(x, num7, 40, -0.004f)` (`:9167`) and the push
        // accumulator (`:9168`): a live push subtracts, a dead one bounces
        // the accumulated sway by -0.78.
        let push = wind.get_wind_grid_push(x, y, 40, -0.004);
        num5 = if push != 0.0 || num6 == 0.0 {
            num5 - push
        } else {
            num5 * -0.78
        };
        num6 = push;
        // `GetTileDrawData` (`:9173`) — the strip never runs the
        // `DrawSingleTile` positional switch, so no world corrections.
        let solid_above = world.tile(x, y - 1).is_some_and(|t| {
            t.active() && td::TILE_SOLID.has(t.kind) && !td::TILE_SOLID_TOP.has(t.kind)
        });
        let dd = draw_data(tile.kind, x, y, tile.frame_x, tile.frame_y, 0, solid_above);
        let half = if tile.half_brick() { 8 } else { 0 };
        // `num8 = num2 * (0 - num4) * windCycle + num5` (`:9179`).
        let rot = num2 as f32 * (0.0 - num4) * wind_cycle + num5;
        let above = world.tile(x, y - 1);
        out.push(RisingVineSegment {
            kind: tile.kind,
            x,
            y,
            frame_x: tile.frame_x,
            frame_y: tile.frame_y,
            dd,
            half,
            anchor,
            rot,
            // `IsVisible(tile)` (`:9185`) — the FULL read (the g31 keep-one
            // delta): the platform row-48 / 541 / 631 arms ride the flag.
            visible: is_visible(world, x, y, show_invisible),
            fullbright: tile.fullbright_block(),
            paint: tile.paint(),
            seam_top: above.is_some_and(|t| t.active() && is_reverse_vine_thread(t.kind)),
        });
        // `vector += (num8 - PI/2).ToRotationVector2() * 16f` (`:9190`,
        // `Utils.cs:1665-1668`: `(cos, sin)`).
        let (s, c) = (rot - std::f32::consts::FRAC_PI_2).sin_cos();
        anchor = [anchor[0] + c * 16.0, anchor[1] + s * 16.0];
        y -= 1;
    }
    out
}

/// `TileDrawing.IsVisible` (`TileDrawing.cs:1411-1432`) — the g31 keep-one
/// (contract §1.2): an Echo-coated block hides unless sight is granted
/// (`show_invisible` = `Main.ShouldShowInvisibleBlocksAndWalls()`); the
/// platform row-48 and the two invisible wiring tiles (541/631) always ride
/// the flag. Byte-shape identical to the g31 union's `tile_drains::is_visible`
/// (the family merge keep-ones one copy — the d56 `tr_render` bind).
#[inline]
pub(crate) fn is_visible(world: &World, x: i32, y: i32, show_invisible: bool) -> bool {
    let Some(tile) = world.tile(x, y) else {
        return true;
    };
    let mut flag = world.invisible_block(x, y);
    match tile.kind {
        19 if tile.frame_y / 18 == 48 => flag = true,
        541 | 631 => flag = true,
        _ => {}
    }
    !flag || show_invisible
}

/// `DrawReverseVines` (`TileDrawing.cs:8903-8914`): one `DrawRisingVineStrip`
/// per registered reverse root, in registration (draw-area row-major) order.
/// Returns the quad count. WIRED (W4-g83 wiring phase): `tiles.rs` calls this
/// right after `draw_vines` (`PostDrawTiles` `DrawVines` → `DrawReverseVines`,
/// `:429-430`); the g31-union rebind maps it onto the `ReverseVines` post step
/// (`docs/research/123-vine-wire-contract.md` §1). `show_invisible` threads
/// the caller's sight override into the strip's `IsVisible` read (the g31
/// keep-one delta, seam 4); the interim fused-pass caller passes `false`.
#[allow(clippy::too_many_arguments)]
pub(crate) fn draw_reverse_vines(
    r: &mut SpriteRenderer,
    world: &World,
    lookups: &mut assets::Lookups,
    atlas: Option<&Atlas>,
    light: &LightMap,
    style: &LightStyle,
    anim_time: f32,
    wind: &Wind,
    vine_counter: f32,
    roots: &[(i32, i32)],
    show_invisible: bool,
) -> u32 {
    let mut quads = 0u32;
    for &(x, start_y) in roots {
        let segments =
            rising_vine_strip_segments(world, wind, vine_counter, x, start_y, show_invisible);
        for seg in segments {
            // `GetTileDrawTexture` null → `break` (`:9180-9184`); the port's
            // unframed-cell guard (`resolve_tile_frame`) is the same state.
            let Some(tf) = resolve_tile_frame(
                lookups,
                atlas,
                false,
                0,
                seg.kind,
                seg.frame_x,
                seg.frame_y,
                &seg.dd,
            ) else {
                break;
            };
            if !seg.visible {
                continue; // `IsVisible` (`:9185`) — the walk still advances.
            }
            // `Lighting.GetColor(x, num7)` (`:9172`) — fullbright → White
            // (`:9175-9178`); no `GetFinalLight` actuation ghost.
            let l = if seg.fullbright {
                [1.0; 3]
            } else {
                light.get(seg.x, seg.y)
            };
            let tint = style.shade(l, seg.x, seg.y, anim_time);
            let w = seg.dd.width as f32;
            let h = (seg.dd.height - seg.half as u16) as f32;
            // `origin = (tileWidth / 2, halfBrickHeight - tileTop +
            // tileHeight)` (`:9188`) — the rising strip pivots at the
            // sprite's BOTTOM-centre (+ tileHeight), not the top.
            let origin = [w * 0.5, (seg.half - seg.dd.top) as f32 + h];
            let pos = [seg.anchor[0] - origin[0], seg.anchor[1] - origin[1]];
            r.push_sprite_origin_rot_paint(
                pos,
                [w, h],
                flip_uv(tf.uv, seg.dd.flip),
                [tint[0], tint[1], tint[2], 1.0],
                seg.rot,
                origin,
                seg.paint,
            );
            quads += 1;
            // NO glowmask: `:9173` discards the glow out-params and `:9188`
            // is the only draw call in the visible arm.
        }
    }
    quads
}

#[cfg(test)]
mod tests {
    use super::*;
    use tr_core::tile::FLAG_ACTIVE;

    /// `TileID.Sets.ReverseVineThreads` (`TileID.cs:545`):
    /// `CreateBoolSet(549)` — the seaweed is the only rising vine.
    #[test]
    fn w4_g83_the_reverse_set_is_the_seaweed_alone() {
        assert!(td::SET_REVERSE_VINE_THREADS.has(549));
        for k in VINE_THREADS {
            assert!(!td::SET_REVERSE_VINE_THREADS.has(k));
        }
    }

    /// A 40x60 world with a 3-segment jungle-vine strip hanging at x=10 from
    /// y=20, rooted under a dirt block at (10, 19).
    fn vine_world() -> World {
        let mut w = World::new("t".into(), 40, 60);
        let put = |w: &mut World, x: i32, y: i32, kind: u16| {
            let i = w.index(x, y);
            w.tiles[i].kind = kind;
            w.tiles[i].flags = FLAG_ACTIVE;
        };
        put(&mut w, 10, 19, 0); // dirt — the attachment root
        for y in 20..23 {
            put(&mut w, 10, y, 52);
        }
        w.compute_column_surface();
        w.compute_chunks();
        w
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

    // `CrawlToTopOfVineAndAddSpecialPoint` (`TileDrawing.cs:744-762`): every
    // cell of one strip resolves the SAME root (the row below the attachable
    // dirt), and the dedup list registers it once.
    #[test]
    fn s1_g_1a_the_crawl_finds_the_root_below_the_attachment() {
        let w = vine_world();
        let mut seen = HashSet::new();
        let mut roots = Vec::new();
        for y in 20..23 {
            collect_vine_root(&w, 10, y, &mut seen, &mut roots);
        }
        assert_eq!(roots, vec![(10, 20)]);
    }

    // A platform (solidTop) does NOT accept a bottom attachment
    // (`WorldGen.cs:70449` excludes `tileSolidTop`), so the crawl walks past
    // it to the first inactive cell — the root moves up one row.
    #[test]
    fn s1_g_1a_a_platform_root_does_not_stop_the_crawl() {
        let mut w = vine_world();
        let i = w.index(10, 19);
        w.tiles[i].kind = 19; // wooden platform
        let mut seen = HashSet::new();
        let mut roots = Vec::new();
        collect_vine_root(&w, 10, 21, &mut seen, &mut roots);
        assert_eq!(roots, vec![(10, 19)]);
    }

    // `BottomEdgeCanBeAttachedTo` (`WorldGen.cs:70440-70458`): solid +
    // not-solidTop + no bottom slope + not actuated off + not tileNoAttach.
    // NOTE: unlike the TOP edge, the bottom edge has NO halfBrick check.
    #[test]
    fn s1_g_1a_bottom_edge_attachment_gates() {
        use tr_core::torch::bottom_edge_can_be_attached_to as bottom;
        let mut w = vine_world();
        let i = w.index(10, 19);
        assert!(bottom(&w, 10, 19), "plain dirt attaches");
        // A half-brick dirt still attaches (the C# bottom edge never checks).
        w.tiles[i].flags |= tr_core::tile::FLAG_HALF_BRICK;
        assert!(bottom(&w, 10, 19), "half-brick dirt still attaches");
        w.tiles[i].flags &= !tr_core::tile::FLAG_HALF_BRICK;
        // Bottom slopes 3/4 refuse.
        for slope in [3u8, 4] {
            w.tiles[i].set_slope(slope);
            assert!(!bottom(&w, 10, 19), "slope {slope} refuses");
        }
        w.tiles[i].set_slope(1); // a TOP slope does not block the bottom edge
        assert!(bottom(&w, 10, 19));
        w.tiles[i].set_slope(0);
        // Actuated off refuses.
        w.tiles[i].flags |= tr_core::tile::FLAG_INACTIVE;
        assert!(!bottom(&w, 10, 19));
        w.tiles[i].flags &= !tr_core::tile::FLAG_INACTIVE;
        // Out of world reads as attachable (the `tile == null` arm).
        assert!(bottom(&w, 10, -1));
        assert!(bottom(&w, 1000, 19));
    }

    // Calm air (`TileDrawing.cs:9039-9043`, `:9088`, `:9130`): the cycle is 0
    // (the |wind| 0.08 gate), no grid stamps → every rotation is 0 and the
    // anchor walks straight down 16 px per segment from `(x*16+8, y*16-2)`.
    #[test]
    fn s1_g_1a_calm_strip_hangs_straight_down() {
        let w = vine_world();
        let wind = calm_wind();
        let segs = vine_strip_segments(&w, &wind, 0.0, 10, 20);
        assert_eq!(segs.len(), 3);
        for (k, seg) in segs.iter().enumerate() {
            assert_eq!(seg.rot, 0.0);
            assert_eq!(seg.anchor, [168.0, 318.0 + 16.0 * k as f32]);
            assert_eq!(seg.dd.top, -2, "the vine family is raised 2 px");
            assert!(
                seg.dd.flip,
                "x=10 is an even world column → FlipHorizontally"
            );
            assert!(seg.visible);
        }
        assert!(segs[0].seam_bottom && segs[1].seam_bottom);
        assert!(!segs[2].seam_bottom, "the last segment has no vine below");
    }

    // The sway ladder (`:9057-9064`, `:9088`): `num4` starts at
    // `-0.08 * num3`, gains `+0.0025` per segment once `num2 >= 2` and
    // `+0.0075 * num3` once `num2 >= 5`; `rot = num2 * num4 * cycle`.
    // Wall 0 allows wind and every cell is above the (infinite) surface, so
    // `num2 == segment index` when the rotation is computed.
    #[test]
    fn s1_g_1a_the_sway_ladder_widens_with_depth() {
        let mut w = vine_world();
        for y in 23..30 {
            let i = w.index(10, y);
            w.tiles[i].kind = 52;
            w.tiles[i].flags = FLAG_ACTIVE;
        }
        let wind = windy();
        let segs = vine_strip_segments(&w, &wind, 0.0, 10, 20);
        assert_eq!(segs.len(), 10);
        let num3 = 0.2 + 0.8 * (0.5f32 / 1.2);
        let cycle = wind.cycle(10, 20, 0.0);
        assert!(cycle != 0.0);
        for (k, seg) in segs.iter().enumerate() {
            let k = k as i32;
            // C# order per cell: the ladder updates with the PREVIOUS
            // exposed count, then the wall probe increments num2, then
            // `num7 = num2 * num4 * cycle` (`:9057-9088`). So segment k
            // rotates with num2 = k+1 and the ladder state after cells
            // 0..=k: +0.0025 for each j >= 2, +0.0075*num3 for each j >= 5.
            let num4 = -0.08 * num3
                + 0.0025 * (k - 1).max(0) as f32
                + 0.0075 * num3 * (k - 4).max(0) as f32;
            let expect = (k + 1) as f32 * num4 * cycle;
            assert!(
                (seg.rot - expect).abs() < 1e-5,
                "segment {k}: {} vs {expect}",
                seg.rot
            );
        }
        // The anchor advances along `(rot + PI/2)` (`:9130`).
        for pair in segs.windows(2) {
            let (s, c) = (pair[0].rot + std::f32::consts::FRAC_PI_2).sin_cos();
            assert!((pair[1].anchor[0] - pair[0].anchor[0] - c * 16.0).abs() < 1e-3);
            assert!((pair[1].anchor[1] - pair[0].anchor[1] - s * 16.0).abs() < 1e-3);
        }
    }

    // A wind-blocking wall (wall 1 is not in `WallID.Sets.AllowsWind`) keeps
    // `num2` at 0 → the ladder never widens and `rot` stays 0 even in wind
    // (`:9065-9075`, `:9088`).
    #[test]
    fn s1_g_1a_a_blocking_wall_freezes_the_ladder() {
        let mut w = vine_world();
        for y in 20..23 {
            let i = w.index(10, y);
            w.tiles[i].wall = 1;
        }
        let wind = windy();
        let segs = vine_strip_segments(&w, &wind, 0.0, 10, 20);
        for seg in &segs {
            assert_eq!(seg.rot, 0.0);
        }
    }

    // The remix world flips the exposure gate (`:9065-9071`): below the
    // surface counts, above it does not.
    #[test]
    fn s1_g_1a_remix_exposes_below_the_surface() {
        let w = vine_world();
        let mut wind = windy();
        wind.remix_world = true;
        wind.world_surface = 10.0; // the strip (y >= 20) is BELOW it
        let segs = vine_strip_segments(&w, &wind, 0.0, 10, 20);
        assert!(segs[2].rot != 0.0, "remix: below-surface cells sway");
        wind.world_surface = 100.0; // now the strip is ABOVE it
        let segs = vine_strip_segments(&w, &wind, 0.0, 10, 20);
        for seg in &segs {
            assert_eq!(seg.rot, 0.0);
        }
    }

    // The grid push accumulator (`:9076-9078`): a live push subtracts from
    // `num5`; the first dead cell after it bounces `num5 * -0.78`.
    #[test]
    fn s1_g_1a_the_grid_push_subtracts_then_bounces() {
        let w = vine_world();
        let mut wind = calm_wind(); // cycle 0 → rot == num5 exactly
        wind.wind_grid.set_size(40, 60);
        wind.wind_grid.set_wind_time(10, 21, 1, 0);
        for _ in 0..5 {
            wind.wind_grid.update();
        }
        // game_time 5, stamp at 0 → time_left 5 (< 20/2) → push 5*0.01*1.
        let segs = vine_strip_segments(&w, &wind, 0.0, 10, 20);
        assert_eq!(segs.len(), 3);
        assert_eq!(segs[0].rot, 0.0, "y=20 unstamped");
        assert!((segs[1].rot - -0.05).abs() < 1e-6, "y=21: 0 - 0.05");
        assert!(
            (segs[2].rot - 0.039).abs() < 1e-6,
            "y=22: -0.05 * -0.78 = 0.039"
        );
    }

    // The strip breaks on the first non-vine / inactive cell
    // (`:9052-9055`) and never walks past `maxTilesY - 10` (`:9044`).
    #[test]
    fn s1_g_1a_the_strip_stops_at_the_first_gap() {
        let mut w = vine_world();
        let i = w.index(10, 21);
        w.tiles[i].flags &= !FLAG_ACTIVE;
        let wind = calm_wind();
        let segs = vine_strip_segments(&w, &wind, 0.0, 10, 20);
        assert_eq!(segs.len(), 1);
        // A foreign active tile breaks it too.
        w.tiles[i].flags |= FLAG_ACTIVE;
        w.tiles[i].kind = 0;
        let segs = vine_strip_segments(&w, &wind, 0.0, 10, 20);
        assert_eq!(segs.len(), 1);
        // The bottom bound: a root at height-10 draws nothing.
        let segs = vine_strip_segments(&w, &wind, 0.0, 10, w.height - 10);
        assert!(segs.is_empty());
    }

    // An Echo-coated segment draws nothing but the walk continues
    // (`IsVisible`, `:9121` — the `continue` is inside the draw arm only).
    #[test]
    fn s1_g_1a_an_echo_segment_hides_but_does_not_break() {
        let mut w = vine_world();
        // `invisibleBlock` rides the extras table (sTileHeader coating bit).
        w.set_invisible_block(10, 21, true);
        let wind = calm_wind();
        let segs = vine_strip_segments(&w, &wind, 0.0, 10, 20);
        assert_eq!(segs.len(), 3);
        assert!(segs[0].visible && !segs[1].visible && segs[2].visible);
    }

    // ---- W4-g83: the rising (reverse-vine 549) strip ----

    /// A 40x60 world with a 3-segment seaweed strip RISING at x=10 from the
    /// root y=30 (cells 30/29/28), standing on a dirt floor at (10, 31).
    fn rising_vine_world() -> World {
        let mut w = World::new("t".into(), 40, 60);
        let put = |w: &mut World, x: i32, y: i32, kind: u16| {
            let i = w.index(x, y);
            w.tiles[i].kind = kind;
            w.tiles[i].flags = FLAG_ACTIVE;
        };
        put(&mut w, 10, 31, 0); // dirt — the top-edge attachment floor
        for y in 28..=30 {
            put(&mut w, 10, y, 549);
        }
        w.compute_column_surface();
        w.compute_chunks();
        w
    }

    // `CrawlToBottomOfReverseVineAndAddSpecialPoint` (`:764-782`): every cell
    // of one rising strip resolves the SAME root (the row ABOVE the
    // attachable dirt floor), and the dedup list registers it once. The
    // crawl starts AT j (`:767`), so the floor cell itself is the stop.
    #[test]
    fn w4_g83_the_reverse_crawl_finds_the_root_above_the_floor() {
        let w = rising_vine_world();
        let mut seen = HashSet::new();
        let mut roots = Vec::new();
        for y in 28..=30 {
            collect_reverse_vine_root(&w, 10, y, &mut seen, &mut roots);
        }
        assert_eq!(roots, vec![(10, 30)]);
    }

    // A lone reverse-vine cell with an inactive cell directly below roots at
    // its OWN row (`y = k - 1` with `k = j + 1`, `:772`) — the g30 fixture
    // contract.
    #[test]
    fn w4_g83_a_lone_reverse_cell_roots_at_its_own_row() {
        let mut w = rising_vine_world();
        // Remove the floor and the two upper cells: only (10, 30) stands,
        // (10, 31) is inactive.
        for y in [28, 29, 31] {
            let i = w.index(10, y);
            w.tiles[i].flags &= !FLAG_ACTIVE;
        }
        let mut seen = HashSet::new();
        let mut roots = Vec::new();
        collect_reverse_vine_root(&w, 10, 30, &mut seen, &mut roots);
        assert_eq!(roots, vec![(10, 30)]);
    }

    // Calm air (`:9139-9143`, `:9179`, `:9190`): the cycle is 0, no grid
    // stamps → every rotation is 0 and the anchor walks straight UP 16 px
    // per segment from `(x*16+8, startY*16+16+2)`.
    #[test]
    fn w4_g83_a_calm_rising_strip_climbs_straight_up() {
        let w = rising_vine_world();
        let wind = calm_wind();
        let segs = rising_vine_strip_segments(&w, &wind, 0.0, 10, 30, false);
        assert_eq!(segs.len(), 3);
        for (k, seg) in segs.iter().enumerate() {
            assert_eq!(seg.rot, 0.0);
            // startY=30 → anchor Y = 30*16+16+2 = 498, minus 16 per segment.
            assert_eq!(seg.anchor, [168.0, 498.0 - 16.0 * k as f32]);
            assert_eq!(seg.y, 30 - k as i32);
            assert!(seg.visible);
        }
        // The seam is TOP (the cell above), not bottom: cells 30/29 have a
        // reverse vine above, cell 28 does not.
        assert!(segs[0].seam_top && segs[1].seam_top);
        assert!(!segs[2].seam_top);
    }

    // The sway ladder negates the step (`num2 * (0 - num4) * cycle`, `:9179`)
    // and the anchor advances along `(rot - PI/2)` (`:9190`). Wall 0 allows
    // wind and every cell is above the (infinite) surface, so num2 == the
    // segment index when the rotation is computed.
    #[test]
    fn w4_g83_the_rising_sway_negates_the_ladder() {
        let mut w = rising_vine_world();
        for y in 21..28 {
            let i = w.index(10, y);
            w.tiles[i].kind = 549;
            w.tiles[i].flags = FLAG_ACTIVE;
        }
        let wind = windy();
        let segs = rising_vine_strip_segments(&w, &wind, 0.0, 10, 30, false);
        assert_eq!(segs.len(), 10);
        let num3 = 0.2 + 0.8 * (0.5f32 / 1.2);
        let cycle = wind.cycle(10, 30, 0.0);
        assert!(cycle != 0.0);
        for (k, seg) in segs.iter().enumerate() {
            let k = k as i32;
            // Same ladder state as the hanging strip, but the rotation is
            // num2 * (0 - num4) * cycle (`:9179`).
            let num4 = -0.08 * num3
                + 0.0025 * (k - 1).max(0) as f32
                + 0.0075 * num3 * (k - 4).max(0) as f32;
            let expect = (k + 1) as f32 * (0.0 - num4) * cycle;
            assert!(
                (seg.rot - expect).abs() < 1e-5,
                "segment {k}: {} vs {expect}",
                seg.rot
            );
        }
        // The anchor advances along `(rot - PI/2)` (`:9190`) — upward.
        for pair in segs.windows(2) {
            let (s, c) = (pair[0].rot - std::f32::consts::FRAC_PI_2).sin_cos();
            assert!((pair[1].anchor[0] - pair[0].anchor[0] - c * 16.0).abs() < 1e-3);
            assert!((pair[1].anchor[1] - pair[0].anchor[1] - s * 16.0).abs() < 1e-3);
            assert!(pair[1].anchor[1] < pair[0].anchor[1], "the strip climbs");
        }
    }

    // The rising exposure gate has NO remix arm (`:9163` always
    // `num7 < worldSurface`). In a remix world BELOW the surface the cycle
    // is live (`GetWindCycle`'s own remix gate passes, `:9141`), but the
    // rising exposure counts only `num7 < surface` — false here — so num2
    // stays 0 and the ladder never sways. The HANGING strip in the same
    // world DOES sway (its gate flips to `y > surface`, `:9065-9071`): the
    // contrast pins the rising gate's missing remix arm.
    #[test]
    fn w4_g83_the_rising_gate_ignores_remix() {
        let w = rising_vine_world();
        let mut wind = windy();
        wind.remix_world = true;
        wind.world_surface = 10.0; // the strip (y >= 28) is BELOW it
        assert!(wind.cycle(10, 30, 0.0) != 0.0, "remix below surface: cycle live");
        let segs = rising_vine_strip_segments(&w, &wind, 0.0, 10, 30, false);
        for seg in &segs {
            assert_eq!(seg.rot, 0.0, "rising exposure ignores remix → num2 = 0");
        }
        // The hanging 52 strip in the SAME remix-below-surface world sways.
        let hw = vine_world();
        let hsegs = vine_strip_segments(&hw, &wind, 0.0, 10, 20);
        assert!(hsegs[2].rot != 0.0, "hanging remix below surface sways");
    }

    // The grid push window is 40 / -0.004 (`:9167`), not the hanging strip's
    // 20 / 0.01: a stamp at game_time 5 leaves time_left 5 (< 40/2) → push
    // 5 * -0.004 * 1 = -0.02, subtracted from num5.
    #[test]
    fn w4_g83_the_rising_grid_push_uses_the_40_window() {
        let w = rising_vine_world();
        let mut wind = calm_wind(); // cycle 0 → rot == num5 exactly
        wind.wind_grid.set_size(40, 60);
        wind.wind_grid.set_wind_time(10, 29, 1, 0);
        for _ in 0..5 {
            wind.wind_grid.update();
        }
        let segs = rising_vine_strip_segments(&w, &wind, 0.0, 10, 30, false);
        assert_eq!(segs.len(), 3);
        assert_eq!(segs[0].rot, 0.0, "y=30 unstamped");
        assert!((segs[1].rot - 0.02).abs() < 1e-6, "y=29: 0 - (-0.02)");
        assert!(
            (segs[2].rot - -0.0156).abs() < 1e-6,
            "y=28: 0.02 * -0.78 = -0.0156"
        );
    }

    // The strip breaks on the first non-reverse / inactive cell (`:9150`)
    // and never walks at or below row 10 (`:9144` `num7 > 10`).
    #[test]
    fn w4_g83_the_rising_strip_stops_at_the_first_gap_and_row_10() {
        let mut w = rising_vine_world();
        let i = w.index(10, 29);
        w.tiles[i].flags &= !FLAG_ACTIVE;
        let wind = calm_wind();
        let segs = rising_vine_strip_segments(&w, &wind, 0.0, 10, 30, false);
        assert_eq!(segs.len(), 1);
        // A foreign active tile breaks it too.
        w.tiles[i].flags |= FLAG_ACTIVE;
        w.tiles[i].kind = 0;
        let segs = rising_vine_strip_segments(&w, &wind, 0.0, 10, 30, false);
        assert_eq!(segs.len(), 1);
        // The top bound: a root at row 10 draws nothing (`num7 > 10`).
        let segs = rising_vine_strip_segments(&w, &wind, 0.0, 10, 10, false);
        assert!(segs.is_empty());
    }

    // An Echo-coated segment draws nothing but the walk continues
    // (`IsVisible`, `:9185`).
    #[test]
    fn w4_g83_a_rising_echo_segment_hides_but_does_not_break() {
        let mut w = rising_vine_world();
        w.set_invisible_block(10, 29, true);
        let wind = calm_wind();
        let segs = rising_vine_strip_segments(&w, &wind, 0.0, 10, 30, false);
        assert_eq!(segs.len(), 3);
        assert!(segs[0].visible && !segs[1].visible && segs[2].visible);
    }

    /// W4-g83 seam 4 (the g31 keep-one delta, contract §1.2): the strip's
    /// `visible` field rides the FULL `IsVisible` (`TileDrawing.cs:1411-1432`)
    /// — the caller's `show_invisible` override un-hides the Echo segment,
    /// and the platform row-48 / 541 / 631 arms always ride the flag even
    /// when the cell is not Echo-coated. (The 549 strip itself never holds
    /// those kinds; the predicate is pinned directly — the same shape the
    /// g31 union's `tile_drains::is_visible` carries.)
    #[test]
    fn w4_g83_the_rising_visible_reads_the_full_is_visible() {
        let mut w = rising_vine_world();
        w.set_invisible_block(10, 29, true);
        let wind = calm_wind();
        // show_invisible = true: the Echo segment draws too.
        let segs = rising_vine_strip_segments(&w, &wind, 0.0, 10, 30, true);
        assert!(segs.iter().all(|s| s.visible), "the sight override un-hides");
        // The predicate's special arms (the g31 byte-shape): a platform on
        // row 48 and the 541/631 wiring tiles ride the flag unconditionally.
        let mut w2 = rising_vine_world();
        let i = w2.index(10, 29);
        w2.tiles[i].kind = 19;
        w2.tiles[i].frame_y = 48 * 18;
        assert!(!is_visible(&w2, 10, 29, false), "platform row 48 rides the flag");
        assert!(is_visible(&w2, 10, 29, true), "the override grants sight");
        for kind in [541u16, 631] {
            w2.tiles[i].kind = kind;
            assert!(!is_visible(&w2, 10, 29, false), "kind {kind} rides the flag");
        }
        // A plain 549 cell without the Echo coat is visible either way.
        w2.tiles[i].kind = 549;
        assert!(is_visible(&w2, 10, 29, false));
    }

    // The rising origin pivots at the sprite's BOTTOM-centre
    // (`halfBrickHeight - tileTop + tileHeight`, `:9188`): with the 549
    // drawdata (16x16, top 0, no half-brick) the origin Y is 16, not 0.
    #[test]
    fn w4_g83_the_rising_origin_is_the_bottom_centre() {
        let w = rising_vine_world();
        let wind = calm_wind();
        let segs = rising_vine_strip_segments(&w, &wind, 0.0, 10, 30, false);
        let seg = &segs[0];
        let h = (seg.dd.height - seg.half as u16) as f32;
        let origin_y = (seg.half - seg.dd.top) as f32 + h;
        assert_eq!(origin_y, 16.0, "16 px tall, top 0, no half-brick");
        assert_eq!(seg.dd.width as f32 * 0.5, 8.0);
    }
}
