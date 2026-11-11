//! The row-major tile pass (`Main.DrawTiles`-shaped): wall layer, the greedy
//! dark-rectangle merge, slope/half/sliced block variants and the per-frame
//! draw statistics.
use crate::assets;
use crate::biome::calculate_water_style;
use crate::item::ItemDrop;
use crate::net;
use crate::npc::Npc;
use crate::player::{self, Player};
use crate::render::entities::{
    draw_npcs, draw_player, flip_uv, ArmorKinds, NpcAssets, PlayerAssets,
};
use crate::render::foliage::draw_tree_foliage;
use crate::render::liquids::{draw_tile_liquid_behind, draw_waters, LiquidPass};
use crate::ui::text::{hud_text, hud_text_width};
use glam::Vec2;
use std::time::Instant;
use tr_core::lighting::{LightMap, LightParams, LightStyle};
use tr_core::tiledata as td;
use tr_core::{tree, World, TILE_PX};
use tr_render::{Atlas, SpriteRenderer};

#[derive(Default, Clone, Copy, Debug)]
pub(crate) struct TileDrawStats {
    pub(crate) visible: u32,
    pub(crate) walls: u32,
    pub(crate) walls_culled: u32,
    pub(crate) dark_cells: u32,
    pub(crate) dark_rects: u32,
    pub(crate) chunks_skipped: u32,
    pub(crate) lit_tiles: u32,
    pub(crate) quads: u32,
    pub(crate) foliage: u32,
    pub(crate) liquids: u32,
    pub(crate) waterfalls: u32,
    pub(crate) items: u32,
    pub(crate) light_ms: f32,
    pub(crate) draw_ms: f32,
    pub(crate) instances: u32,
    pub(crate) world_reused: bool,
}

/// Scratch storage for the greedy dark-rectangle merge (A3/A4).
#[derive(Default)]
pub(crate) struct DrawScratch {
    /// Rectangles still being extended downwards: `[x0, x1, y0, y1]`.
    open: Vec<[i32; 4]>,
    next: Vec<[i32; 4]>,
    /// Dark x-runs of the current row.
    cur: Vec<(i32, i32)>,
    /// The waterfall `AddLight` sources of the previous frame
    /// (`WaterfallManager.AddLight`): vanilla's light engine consumes the
    /// previous frame's light list when `LightTiles` runs, so the lights are
    /// swapped here after each world pass and consumed by the next compute.
    pub(crate) waterfall_lights: Vec<tr_core::lighting::DynamicLight>,
    /// The waterfall quads walked this frame (`WaterfallManager.draw`), consumed
    /// by the tile pass below.
    pub(crate) waterfall_quads: Vec<tr_core::waterfall::WaterfallDraw>,
    /// The liquid rects of the last world-pass build (world pixels) — the
    /// WaterDistortion mask input. It is refreshed exactly when the G6-cached
    /// world range is rebuilt, so it always matches the liquid sprites the
    /// cached range still holds.
    pub(crate) water_rects: Vec<[f32; 4]>,
}

/// `WallID.EchoWall`. `WallDrawing.DrawWalls` skips it unless
/// `DebugOptions.ShowInvisibleWalls` is on, so it contributes neither a sprite nor a
/// black quad. (`tile.invisibleWall()` is checked too, but nothing in the original's
/// code ever sets that bit, and our packed tile does not store it.)
pub(crate) const WALL_ECHO: u16 = 318;

/// `WallDrawing.DrawWalls` draw test: a wall is visible when it exists and is not an
/// echo wall.
#[inline]
pub(crate) fn wall_visible(wall: u16) -> bool {
    wall != 0 && wall != WALL_ECHO
}

/// Vanilla stores light as bytes; a channel below `1/255` is zero. Such a cell is
/// guaranteed to rasterise to black whatever the texture colour, so it can be
/// replaced by one opaque black quad without changing a single pixel.
#[inline]
fn light_is_zero(l: [f32; 3]) -> bool {
    l[0] * 255.0 < 1.0 && l[1] * 255.0 < 1.0 && l[2] * 255.0 < 1.0
}

/// Extends open dark rectangles with this row's runs and emits the ones that end.
/// Returns the number of emitted quads.
fn merge_dark_row(
    r: &mut SpriteRenderer,
    white: [f32; 4],
    scratch: &mut DrawScratch,
    y: i32,
) -> u32 {
    let mut emitted = 0u32;
    scratch.next.clear();
    let mut oi = 0usize;
    let mut ci = 0usize;
    while oi < scratch.open.len() || ci < scratch.cur.len() {
        let take_old = if oi >= scratch.open.len() {
            false
        } else if ci >= scratch.cur.len() {
            true
        } else {
            let o = scratch.open[oi];
            let c = scratch.cur[ci];
            o[0] < c.0 || (o[0] == c.0 && o[1] != c.1)
        };
        if take_old {
            let o = scratch.open[oi];
            r.push_sprite(
                [o[0] as f32 * TILE_PX, o[2] as f32 * TILE_PX],
                [
                    (o[1] - o[0] + 1) as f32 * TILE_PX,
                    (o[3] - o[2] + 1) as f32 * TILE_PX,
                ],
                white,
                [0.0, 0.0, 0.0, 1.0],
            );
            emitted += 1;
            oi += 1;
        } else if oi >= scratch.open.len() || scratch.cur[ci].0 < scratch.open[oi][0] {
            let (s, e) = scratch.cur[ci];
            scratch.next.push([s, e, y, y]);
            ci += 1;
        } else {
            // Same x span: grow the rectangle one row down.
            let mut o = scratch.open[oi];
            o[3] = y;
            scratch.next.push(o);
            oi += 1;
            ci += 1;
        }
    }
    std::mem::swap(&mut scratch.open, &mut scratch.next);
    emitted
}

/// The drawn frame of one world cell — `TileDrawing`'s source-rect resolution
/// (`TileDrawing.cs:1007`): `GetTileDrawData`'s sizes/strides feed the sheet
/// lookup, tree trunks ride the 20x20 trunk grid, and plain 16x16 cells hit the
/// 18 px frame table. A cell whose frames were never written
/// (`frameX/frameY = -1` — the state `WorldGen.PlaceTile` leaves between the
/// tile write and `SquareTileFrame`; the old seed bug wrote it permanently,
/// `PlaceAlch` fixes it) draws nothing instead of sampling sheet-corner
/// garbage: vanilla never draws one, because framing runs before the draw.
fn resolve_tile_frame(
    lookups: &mut assets::Lookups,
    atlas: Option<&Atlas>,
    is_tree: bool,
    trunk_offset: i16,
    kind: u16,
    frame_x: i16,
    frame_y: i16,
    dd: &tr_core::drawdata::TileDrawData,
) -> Option<assets::FrameTex> {
    if frame_x < 0 || frame_y < 0 {
        return None;
    }
    if is_tree {
        // Tree trunks are 20x20 sprites on a 22 px grid.
        return atlas.and_then(|a| {
            lookups.rect_uv(
                a,
                kind,
                frame_x + trunk_offset,
                frame_y,
                tree::TRUNK_SIZE as u32,
                tree::TRUNK_SIZE as u32,
            )
        });
    }
    // Cases that rewrite C#'s `ref tileFrameX/Y` hand the final frame to us;
    // the source rectangle is built from it (`TileDrawing.cs:1007`).
    let fx = dd.frame_x_override.unwrap_or(frame_x);
    let fy = dd.frame_y_override.unwrap_or(frame_y);
    let frame = atlas.and_then(|a| {
        if dd.width != 16 || dd.height != 16 || dd.add_fr_x != 0 || dd.add_fr_y != 0 {
            lookups.rect_uv(
                a,
                kind,
                fx + dd.add_fr_x,
                fy + dd.add_fr_y,
                dd.width as u32,
                dd.height as u32,
            )
        } else {
            lookups.frame_tex(a, kind, fx, fy)
        }
    });
    frame.or_else(|| lookups.plain_tex(kind))
}

/// One cell of the sunflower's special-pass sway
/// (`TileDrawing.DrawMultiTileGrassInWind`, `TileDrawing.cs:8916-9041`).
#[derive(Clone, Copy, Debug, PartialEq)]
struct SunflowerSway {
    /// `vector3.Y` — the draw position's vertical offset
    /// `|windCycle| * 2 * num3` (`TileDrawing.cs:8985`); the horizontal offset
    /// comes from the rotation about the flower's bottom-center anchor.
    dy: f32,
    /// `windCycle * num * num3` with the type-27 factor `num = 0.07`
    /// (`TileDrawing.cs:8926`, `:9001`).
    rot: f32,
    /// `origin = vector - vector2` (`TileDrawing.cs:8990`) — the rotation pivot
    /// relative to the cell sprite's top-left, i.e. the flower's bottom-center
    /// anchor at `(topLeftX*16 + sizeX*8, topLeftY*16 + 16*sizeY)` with the
    /// special pass's `sizeX = 2, sizeY = 5` (`TileDrawing.cs:8748-8752`).
    pivot: [f32; 2],
}

/// The sway state of the sunflower cell at `(tx, ty)` whose flower's top-left
/// is reconstructed from the frame grid: `frameX = 36*style + 18*col`,
/// `frameY = 18*row` (`StyleHorizontal`, `RandomStyleRange = 3`,
/// `TileObjectData.cs:4851-4864`).
fn sunflower_sway(
    wind: &tr_core::wind::Wind,
    world: &World,
    tx: i32,
    ty: i32,
    frame_x: i16,
    frame_y: i16,
    sunflower_wind_counter: f32,
) -> SunflowerSway {
    let col = (frame_x.rem_euclid(36) / 18) as i32;
    let row = (frame_y / 18) as i32;
    let top_left_x = tx - col;
    let top_left_y = ty - row;
    // `GetWindCycle(topLeftX, topLeftY, _sunflowerWindCounter)` (`:8918`); the
    // surface-band and |wind| 0.08..0.18 gates live inside `Wind::cycle`.
    let cycle = wind.cycle(top_left_x, top_left_y, sunflower_wind_counter);
    // `flag = InAPlaceWithWind(topLeftX, topLeftY, sizeX, sizeY)` (`:8923`)
    // with the special pass's `sizeY = 5` — the probe reaches one row into the
    // flower's anchor ground.
    let in_wind = wind.in_a_place_with_wind(world, top_left_x, top_left_y, 2, 5);
    // `num3 = 1 - (j - topLeftY + 1) / sizeY` per row (`:8952`), the 0-row
    // guard `num3 == 0 -> 0.1` (`:8957`), and `!flag -> 0` (`:8961`).
    let mut num3 = 1.0 - (row + 1) as f32 / 5.0;
    if num3 == 0.0 {
        num3 = 0.1;
    }
    if !in_wind {
        num3 = 0.0;
    }
    SunflowerSway {
        dy: cycle.abs() * 2.0 * num3,
        rot: cycle * 0.07 * num3,
        pivot: [
            (top_left_x * 16 + 16 - tx * 16) as f32,
            (top_left_y * 16 + 80 - ty * 16) as f32,
        ],
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn draw_tiles(
    r: &mut SpriteRenderer,
    world: &World,
    lookups: &mut assets::Lookups,
    atlas: Option<&Atlas>,
    camera: Vec2,
    zoom: f32,
    screen: Vec2,
    params: &LightParams,
    light: &mut LightMap,
    scratch: &mut DrawScratch,
    style: &LightStyle,
    anim_time: f32,
    waterfall: &mut tr_core::waterfall::WaterfallManager,
    player: Option<&Player>,
    player_assets: &PlayerAssets,
    // `DrawPlayer_27_HeldItem`'s input: the selected item kind plus the
    // remaining use fraction (`PlayerDrawLayers.cs:3077`); `None` = empty hand.
    held: Option<(u16, f32)>,
    // `Player.armor[0..3]` item kinds (head/body/legs) — the armor layers.
    armor: ArmorKinds,
    npcs: &[Npc],
    npc_assets: &NpcAssets,
    items: &[ItemDrop],
    remote: &[net::PlayerNet],
    edit_target: Option<(i32, i32)>,
    // `Main.liquidAlpha` (`Main.cs:1103`): the per-water-style cross-fade alphas
    // the `DrawWaters` ramp drives; the liquid and waterfall passes consume it.
    liquid_alpha: &[f32; 15],
    // The frame's deterministic seed for the `TrySparkling` draws.
    sparkle_seed: u64,
    // The wind state and the sway counters (`TileDrawing.DrawGrass`/`DrawTrees`;
    // the sunflower's own counter rides its special pass,
    // `TileDrawing.DrawMultiTileGrassInWind`).
    wind: &tr_core::wind::Wind,
    grass_wind_counter: f32,
    tree_wind_counter: f32,
    sunflower_wind_counter: f32,
    // `EmitTreeLeaves` spawns collected by the foliage pass, drained by the
    // caller into the leaf particle list.
    leaf_spawns: &mut Vec<crate::render::foliage::LeafSpawn>,
) -> TileDrawStats {
    let t_total = Instant::now();
    // `TileDrawing._rand` (`TileDrawing.cs:191`): the leaf-roll stream, pinned
    // per frame like the sparkle draws.
    let mut leaf_rng = tr_core::tile_light::FastRandom::new(
        sparkle_seed ^ 0x9E37_79B9_7F4A_7C15,
    );
    let half = screen * 0.5 / zoom;
    let min = camera - half;
    let max = camera + half;
    let x0 = ((min.x / TILE_PX).floor() as i32 - 1).clamp(0, world.width - 1);
    let x1 = ((max.x / TILE_PX).ceil() as i32 + 1).clamp(0, world.width - 1);
    let y0 = ((min.y / TILE_PX).floor() as i32 - 1).clamp(0, world.height - 1);
    let y1 = ((max.y / TILE_PX).ceil() as i32 + 1).clamp(0, world.height - 1);

    // `Main.waterStyle` is a single biome value for the whole frame, exactly like
    // `Main.CalculateWaterStyle` (the original evaluates it from the player's biome).
    let mut water_style = calculate_water_style(world, camera);
    // `TR_WATER_STYLE=<n>`: pin the water style so the per-style passes, the
    // `liquidAlpha` cross-fade and the biome liquid sheets stay screenshotable.
    if let Some(n) = crate::env::water_style() {
        water_style = n.min(14);
    }
    let t_light = Instant::now();
    // Vanilla applies the waterfall lights one frame late (the engine keeps the
    // previous frame's `AddLight` list when `LightTiles` runs); `scratch`
    // carries exactly that list.
    light.compute_into_with_lights(
        world,
        x0,
        y0,
        x1,
        y1,
        params,
        &scratch.waterfall_lights,
        waterfall.lights_key(),
    );
    let light_ms = t_light.elapsed().as_secs_f32() * 1000.0;

    if let Some((lx, ly)) = crate::env::light_at() {
        use std::sync::atomic::{AtomicBool, Ordering};
        static ONCE: AtomicBool = AtomicBool::new(false);
        if !ONCE.swap(true, Ordering::Relaxed) {
            let c = light.get(lx, ly);
            let tile = world.tile(lx, ly).copied().unwrap_or_default();
            log::info!(
                "light at {lx},{ly} = {:.3},{:.3},{:.3} (tile {} wall {} liquid {})",
                c[0],
                c[1],
                c[2],
                tile.kind,
                tile.wall,
                tile.liquid
            );
        }
    }

    let width = world.width as usize;
    let white = r.white_uv();
    let mut stats = TileDrawStats {
        lit_tiles: light.lit_tiles,
        light_ms,
        ..Default::default()
    };
    let chunk = tr_core::world::CHUNK;
    // Vanilla's quality thresholds (`TileDrawing.Draw`, default gfxQuality 0.5):
    // above the high mark blocks get nine light slices, above the medium mark four,
    // below it a single flat quad.
    const HIGH_LIGHT: f32 = 142.0 / 255.0;
    const MEDIUM_LIGHT: f32 = 26.0 / 255.0;

    // `LiquidRenderer.InternalPrepareDraw` runs once per frame; both the opaque
    // background pass and the translucent foreground pass consume this record.
    let liquid_draws = tr_core::liquid_draw::build(world, x0, y0, x1, y1);
    // M1.4-BG-18 phase 4: the same records feed the WaterDistortion mask.
    scratch.water_rects = crate::render::liquids::water_mask_rects(&liquid_draws);
    if crate::env::dump_liquid() {
        let cam_x = camera.x as i32 / 16;
        let cam_y = camera.y as i32 / 16;
        let mut n_vis = 0;
        let mut n_tot = 0;
        for d in &liquid_draws {
            if (d.x - cam_x).abs() < 40 && (d.y - cam_y).abs() < 40 {
                n_tot += 1;
                if d.visible {
                    n_vis += 1;
                }
            }
        }
        log::info!(
            "LIQDUMP near cam {},{}: total={} visible={}",
            cam_x,
            cam_y,
            n_tot,
            n_vis
        );
        for d in &liquid_draws {
            if d.visible && (d.x - cam_x).abs() < 40 && (d.y - cam_y).abs() < 40 {
                log::info!(
                    "  vis tile {},{} type{} lvl{} sw{} sh{} fy{} fx{} sx{} sy{}",
                    d.x,
                    d.y,
                    d.liquid_type,
                    d.src_w,
                    d.src_h,
                    d.is_surface as i32,
                    d.src_y,
                    d.src_x,
                    d.offset_x,
                    d.offset_y
                );
            }
        }
    }
    // `Main.DrawLiquid(bg: true, ...)`: water is drawn before walls and tiles at
    // `Opacity` (usually 1.0), so the parallax background never shows through it.
    if !crate::env::no_liqback() {
        stats.liquids += draw_waters(
            r,
            lookups,
            light,
            style,
            anim_time,
            &liquid_draws,
            LiquidPass::Background,
            water_style,
            liquid_alpha,
        );
    }

    // `Main.DoDraw_Waterfalls`: vanilla draws falls after the non-solid tile pass
    // and before solid tiles. This renderer keeps one row-major tile pass, so the
    // falls are emitted here; tile quads drawn later occlude their base, while a
    // wall behind a fall is drawn over it (documented approximation). The quads
    // were walked once per frame in the pre-draw phase (`App::frame`), which also
    // collects their `AddLight` list for the light map — keeping the walk out of
    // the G6-cached path makes the light feed deterministic per frame index.
    for d in std::mem::take(&mut scratch.waterfall_quads) {
        let Some(uv) = lookups.waterfall_uv(d.texture as usize, d.source) else {
            continue;
        };
        r.push_sprite(
            d.pos,
            [d.source[2] as f32, d.source[3] as f32],
            flip_uv(uv, d.flip),
            d.color,
        );
        stats.waterfalls += 1;
    }

    // `Main.DrawWalls`: the whole wall layer is drawn before the tile layer. Each
    // frame is a 32x32 sprite placed 8 px up/left of the cell, so neighbouring walls
    // join without seams; drawing it interleaved with tiles would let the overhang
    // cut into tiles drawn earlier in the row.
    if crate::env::walls_enabled() {
        for ty in y0..=y1 {
            for tx in x0..=x1 {
                let tile = &world.tiles[ty as usize * width + tx as usize];
                if !wall_visible(tile.wall) {
                    continue;
                }
                let Some(wf) = lookups.wall_tex(tile.wall, tile.wall_col(), tile.wall_row()) else {
                    continue;
                };
                if tile.active()
                    && tile_hides_wall(
                        lookups,
                        atlas,
                        tile.kind,
                        tile.frame_x,
                        tile.frame_y,
                        tile.slope(),
                        tile.half_brick(),
                    )
                {
                    stats.walls_culled += 1;
                    continue;
                }
                // `WallDrawing.cs:91`: Echo-coated walls are hidden unless the
                // Echo Monolith or an Echo item grants sight (the port has
                // neither yet, so they stay hidden; wall 318 is handled above).
                if world.invisible_wall(tx, ty) {
                    continue;
                }
                // `WallDrawing.DrawWalls` skips black cells; the tile pass merges the
                // fully covered ones into black rectangles.
                if light_is_zero(light.get(tx, ty)) {
                    continue;
                }
                // The 32 px sprite is centred on the 16 px cell (`x*16 - 8`), like
                // `new Vector2(j * 16 - 8, i * 16 - 8)` in the original.
                stats.quads += push_wall_lit(
                    r,
                    [tx as f32 * TILE_PX - 8.0, ty as f32 * TILE_PX - 8.0],
                    wf.uv,
                    light,
                    style,
                    tx,
                    ty,
                    anim_time,
                    world.wall_paint(tx, ty),
                    world.fullbright_wall(tx, ty),
                );
                stats.walls += 1;
            }
        }
    }

    scratch.open.clear();
    for ty in y0..=y1 {
        scratch.cur.clear();
        let mut run_start = -1i32;
        let cy = ty / chunk;
        let row_base = ty as usize * width;
        for cx in (x0 / chunk)..=(x1 / chunk) {
            if !world.chunk_has(cx, cy) {
                stats.chunks_skipped += 1;
                if run_start >= 0 {
                    let sx = (cx * chunk).max(x0);
                    scratch.cur.push((run_start, sx - 1));
                    run_start = -1;
                }
                continue;
            }
            let sx = (cx * chunk).max(x0);
            let ex = ((cx + 1) * chunk - 1).min(x1);
            for tx in sx..=ex {
                let tile = &world.tiles[row_base + tx as usize];
                let mut l = light.get(tx, ty);
                // `Tile.actColor` (`Tile.cs:272-290`) via
                // `TileDrawing.GetFinalLight` (`TileDrawing.cs:4380-4397`):
                // an actuated (inactive) tile draws ghosted — RGB × 0.4.
                if tile.flags & tr_core::tile::FLAG_INACTIVE != 0 {
                    l = [l[0] * 0.4, l[1] * 0.4, l[2] * 0.4];
                }
                let active = tile.active();
                let is_tree = active && tree::is_tree(tile.kind);
                let marker = if is_tree {
                    tree::TreePart::marker(tile.frame_x, tile.frame_y)
                } else {
                    None
                };

                // `GetTileDrawData`: per-type sprite size/top/flip/frame data
                // (torches 20x20 with `tileTop 4`, plants 20 px, herbs,
                // candles, palms, frame strides...). `tile_frame` is
                // `Main.tileFrame[kind]`; the animator is not ported, so the
                // vanilla t=0 state is passed.
                let dd = if active && marker.is_none() && !is_tree {
                    let solid_above = world.tile(tx, ty - 1).is_some_and(|t| {
                        t.active() && td::TILE_SOLID.has(t.kind) && !td::TILE_SOLID_TOP.has(t.kind)
                    });
                    let mut dd = tr_core::drawdata::draw_data(
                        tile.kind,
                        tx,
                        ty,
                        tile.frame_x,
                        tile.frame_y,
                        0,
                        solid_above,
                    );
                    // The world-probing frame corrections C# applies in
                    // `GetTileDrawData` (palm biome row 323, dye plants 227)
                    // and `DrawSingleTile` (cactus 80, cog 272).
                    dd.apply_world_corrections(
                        world,
                        tile.kind,
                        tx,
                        ty,
                        tile.frame_x,
                        tile.frame_y,
                        0,
                    );
                    dd
                } else {
                    tr_core::drawdata::TileDrawData::default()
                };

                // Resolve the drawn tile frame once; reused for opacity and drawing.
                let mut tile_frame = None;
                if active && marker.is_none() {
                    let trunk_offset = if is_tree && tile.frame_x >= 0 && tile.frame_y >= 0 {
                        tree::trunk_sheet_offset_at(world, tx, ty) as i16
                    } else {
                        0
                    };
                    tile_frame = resolve_tile_frame(
                        lookups,
                        atlas,
                        is_tree,
                        trunk_offset,
                        tile.kind,
                        tile.frame_x,
                        tile.frame_y,
                        &dd,
                    );
                }
                let solid = (tile.kind as usize) < td::TILE_COUNT && td::TILE_SOLID.has(tile.kind);
                // A3/A4: fully black cell -> greedy merged rectangle.
                // Vanilla paints black coverage from two places, both under
                // `TileDrawingBase.DrawOwnBlacks`:
                //  * `TileDrawing.DrawSingleTile`: every *active* tile whose light is
                //    exactly zero emits a black quad behind its sprite (`DrawBlack`).
                //  * `WallDrawing.DrawWalls`: every cell with a wall that is not a
                //    `FullTile` (block already covering it) emits a black quad.
                // That coverage is what hides the parallax background inside unlit
                // caves. A 32x32 wall sheet frame is mostly transparent outside its
                // 16x16 core, so `frame.opaque` is false; a wall has to count as
                // covering on its own. A fully dark tile renders identically whether
                // its sprite is drawn or skipped (both are solid black), so the cell
                // is folded into one merged rectangle — except flame/glow tiles:
                // `DrawSingleTile_Flames` draws their self-lit overlay over the
                // black sprite (`TileDrawing.cs:985-1016` sets `flag` for
                // `Main.tileFlame`/glow types regardless of light), so their cells
                // must keep their own quad.
                let flame_tile = active
                    && (tr_core::flame::tile_flame(tile.kind)
                        || tr_core::flame::has_glow_branch(tile.kind));
                if light_is_zero(l)
                    && !flame_tile
                    && (active || (crate::env::walls_enabled() && wall_visible(tile.wall)))
                {
                    if run_start < 0 {
                        run_start = tx;
                    }
                    stats.dark_cells += 1;
                    continue;
                }
                if run_start >= 0 {
                    scratch.cur.push((run_start, tx - 1));
                    run_start = -1;
                }

                // `DrawSingleTile`'s `vector` (`TileDrawing.cs:1010-1014`) plus
                // the switch's per-type nudges (`:1020-1159`, `dd.draw_dx/dy`);
                // the width centring stays per-arm (`:1013`'s `num5`).
                let pos = [
                    tx as f32 * TILE_PX + dd.draw_dx as f32,
                    ty as f32 * TILE_PX + dd.draw_dy as f32,
                ];
                // Walls are drawn in their own pass above (`Main.DrawWalls`).
                // `DrawTile_LiquidBehindTile` (solid layer): a slope that stands in
                // liquid draws the water mask behind its open half, so the shoreline
                // is water instead of the parallax background.
                if solid && tile_frame.is_none_or(|f| !f.opaque) {
                    stats.quads += draw_tile_liquid_behind(
                        r,
                        world,
                        lookups,
                        light,
                        style,
                        anim_time,
                        tx,
                        ty,
                        tile,
                        water_style,
                    );
                }
                // The switch's early `return` arms (751/752 off their (0,0)
                // frame, the palm's frond columns 323): no sprite at all.
                if dd.skip_draw {
                    continue;
                }
                // `DrawSingleTile` hands the track tiles to
                // `DrawTile_MinecartTrack` (`TileDrawing.cs:1166-1169`,
                // `:3786-3821`) instead of the generic path: the stored
                // `frame_x`/`frame_y` are frame IDS (0..39), not autotile
                // frames, and the arm draws two layers plus the hanging ties
                // and the bumpers from `Minecart.GetSourceRect`.
                if tile.kind == 314 {
                    stats.quads += draw_minecart_track(
                        r, world, lookups, atlas, l, style, tx, ty, anim_time, tile,
                    );
                    continue;
                }
                let Some(tf) = tile_frame else {
                    continue;
                };
                // `TileDrawing.IsVisible` (`TileDrawing.cs:1411-1432`): an
                // Echo-coated block is hidden unless the Echo Monolith or an
                // Echo item grants sight (the port has neither yet, so the
                // blocks stay hidden).
                if world.invisible_block(tx, ty) {
                    continue;
                }
                stats.visible += 1;
                // `TileDrawing.DrawGrass` (`TileDrawing.cs:8326-8375`): the
                // `SwaysInWindBasic` tiles draw with the grass wind counter —
                // position shifted by `(cycle, |cycle|)` and rotated
                // `cycle * 0.1` about `(tileWidth / 2, 16 - tileTop)`. The
                // row-major loop draws them inline instead of deferring them to
                // a special pass (documented z-order approximation); the
                // `WindGrid` gust push is not ported. Vanilla's type-227 frame
                // exceptions apply (`ShouldSwayInWind`, `TileDrawing.cs:7712`).
                // `TileDrawing.DrawGrass` (`TileDrawing.cs:8326-8375`): the
                // `SwaysInWindBasic` tiles draw with the grass wind counter —
                // position shifted by `(cycle, |cycle|)` and rotated
                // `cycle * 0.1` about `(tileWidth / 2, 16 - tileTop)`. The
                // row-major loop draws them inline instead of deferring them to
                // a special pass (documented z-order approximation); the
                // `WindGrid` gust push is not ported. Vanilla's type-227 frame
                // exceptions apply (`ShouldSwayInWind`, `TileDrawing.cs:7712`).
                let sway = if tr_core::wind::SWAYS_IN_WIND_BASIC.has(tile.kind)
                    && !matches!(tile.frame_x, 204 | 238 | 408 | 442 | 476)
                    && wind.sway_enabled
                {
                    let mut num3 = wind.cycle(tx, ty, grass_wind_counter);
                    if !tr_core::wind::wall_allows_wind(tile.wall) {
                        num3 = 0.0;
                    }
                    if !wind.in_a_place_with_wind(world, tx, ty, 1, 1) {
                        num3 = 0.0;
                    }
                    num3
                } else {
                    0.0
                };
                if is_tree {
                    // 20x20 trunk sprite, drawn like `TileDrawing` does: the 20 px
                    // sprite is centred on the 16 px cell (position shifted by -2),
                    // which is what makes branches meet the trunk exactly.
                    push_lit(
                        r,
                        [pos[0] - 2.0, pos[1]],
                        [tree::TRUNK_SIZE, tree::TRUNK_SIZE],
                        tf.uv,
                        l,
                        style,
                        tx,
                        ty,
                        anim_time,
                        tile.paint(),
                        &dd,
                    );
                    stats.quads += 1;
                } else if tile.kind == 27 {
                    // The sunflower never draws through `DrawSingleTile` — C#
                    // registers the flower's top-left cell as a MultiTileGrass
                    // special (`TileDrawing.cs:600-606`) and
                    // `DrawMultiTileGrass`/`DrawMultiTileGrassInWind`
                    // (`TileDrawing.cs:8721-8805`, `:8916-9041`) draw the 2x4
                    // from `_sunflowerWindCounter` (the cadence
                    // `1/420 + num*5/420`, `:374`) about the flower's
                    // bottom-center. The row-major loop draws each cell inline
                    // with the same per-cell transform (the documented z-order
                    // approximation, as for the grass sway); `windCycle = 0`
                    // degenerates to the static draw, exactly like the special
                    // pass with no wind.
                    let sw = sunflower_sway(
                        wind,
                        world,
                        tx,
                        ty,
                        tile.frame_x,
                        tile.frame_y,
                        sunflower_wind_counter,
                    );
                    let (dw, dh) = (dd.width as f32, dd.height as f32);
                    let tint = style.shade(l, tx, ty, anim_time);
                    // `vector2 + (0, vector3.Y)` in world pixels — the XNA call
                    // places the pivot `origin` at `vector + (0, vector3.Y)`
                    // (`:8985-9001`), so the sprite's top-left lands at
                    // `(i*16, j*16 + tileTop + dy)`; type 27 has `tileTop = 0`.
                    r.push_sprite_origin_rot(
                        [pos[0], pos[1] + sw.dy],
                        [dw, dh],
                        tf.uv,
                        [tint[0], tint[1], tint[2], 1.0],
                        sw.rot,
                        sw.pivot,
                    );
                    stats.quads += 1;
                    // The glowmask: case 27 sets `flag2`/`num2 = 74`
                    // (`:8924-8928`) — the same transform draws the sheet region
                    // 74 px below the cell with `Color.White` (`:9004-9008`),
                    // the face glow (Tiles_27.xnb is 108x148: 74 px of styles +
                    // 74 px of glow art).
                    if let Some(a) = atlas {
                        let glow = lookups.rect_uv(
                            a,
                            tile.kind,
                            dd.frame_x_override.unwrap_or(tile.frame_x) + dd.add_fr_x,
                            dd.frame_y_override.unwrap_or(tile.frame_y) + dd.add_fr_y + 74,
                            dd.width as u32,
                            dd.height as u32,
                        );
                        if let Some(glow) = glow {
                            r.push_sprite_origin_rot(
                                [pos[0], pos[1] + sw.dy],
                                [dw, dh],
                                glow.uv,
                                [1.0, 1.0, 1.0, 1.0],
                                sw.rot,
                                sw.pivot,
                            );
                            stats.quads += 1;
                        }
                    }
                } else if sway != 0.0 {
                    // `DrawGrass`'s swaying sprite: shifted by `(cycle, |cycle|)`
                    // and rotated `cycle * 0.1` about
                    // `(tileWidth / 2, 16 - tileTop)` (`TileDrawing.cs:8369-8374`).
                    let (dw, dh) = (dd.width as f32, dd.height as f32);
                    let sprite_pos = [pos[0] - (dw - 16.0) * 0.5, pos[1] + dd.top as f32];
                    let origin = [dw * 0.5, 16.0 - dd.top as f32];
                    let tint = style.shade(l, tx, ty, anim_time);
                    r.push_sprite_origin_rot(
                        [sprite_pos[0] + sway, sprite_pos[1] + sway.abs()],
                        [dw, dh],
                        flip_uv(tf.uv, dd.flip),
                        [tint[0], tint[1], tint[2], 1.0],
                        sway * 0.1,
                        origin,
                    );
                    stats.quads += 1;
                } else if dd.top != 0 || dd.width != 16 || dd.height != 16 || dd.flip {
                    // `x*16 - (tileWidth - 16) / 2`, `y*16 + tileTop`; horizontal flip
                    // swaps the source columns. The centring is skipped for the
                    // `DoNotAdjustDrawPositionBasedOnTileWidth` kinds (711,
                    // `TileDrawing.cs:1010-1015`).
                    let (dw, dh) = (dd.width as f32, dd.height as f32);
                    let num5 = if dd.no_width_adjust {
                        0.0
                    } else {
                        (dw - 16.0) * 0.5
                    };
                    push_lit(
                        r,
                        [pos[0] - num5, pos[1] + dd.top as f32],
                        [dw, dh],
                        flip_uv(tf.uv, dd.flip),
                        l,
                        style,
                        tx,
                        ty,
                        anim_time,
                        tile.paint(),
                        &dd,
                    );
                    stats.quads += 1;
                } else if tile.slope() != 0 {
                    stats.quads +=
                        push_slope(r, pos, tf.uv, l, style, tx, ty, anim_time, tile.slope());
                } else if tile.half_brick() {
                    // The C#'s half-brick arm rebuilds its own rect from
                    // `tileFrameX/Y + addFrX/addFrY` (`TileDrawing.cs:1298`) —
                    // case 80's biome shift lands on `normalTileRect` only, so
                    // the half-brick draw never carries `src_y_shift`.
                    push_half(r, pos, tf.uv, l, style, tx, ty, anim_time);
                    stats.quads += 1;
                } else if style.sliced_blocks && solid && l[0].max(l[1]).max(l[2]) > MEDIUM_LIGHT {
                    stats.quads +=
                        push_sliced(r, pos, tf.uv, light, style, tx, ty, anim_time, HIGH_LIGHT);
                } else if sway != 0.0 {
                    // The plain-size sway members (e.g. the sunflower 113, whose
                    // `TileDrawData` is the default): the same `DrawGrass`
                    // transform with the 16x16 origin `(8, 16)`.
                    let tint = style.shade(l, tx, ty, anim_time);
                    r.push_sprite_origin_rot(
                        [pos[0] + sway, pos[1] + sway.abs()],
                        [TILE_PX, TILE_PX],
                        tf.uv,
                        [tint[0], tint[1], tint[2], 1.0],
                        sway * 0.1,
                        [8.0, 16.0],
                    );
                    stats.quads += 1;
                } else {
                    let flat_uv = flat_src_uv(&dd, tile, tf, atlas, lookups);
                    push_lit(
                        r,
                        pos,
                        [TILE_PX, TILE_PX],
                        flat_uv,
                        l,
                        style,
                        tx,
                        ty,
                        anim_time,
                        tile.paint(),
                        &dd,
                    );
                    stats.quads += 1;
                    // `TileDrawing.cs:6849-6855` + `EmitLivingTreeLeaf`
                    // (`TileDrawing.cs:7549-7590`): leaf blocks shed leaf gores
                    // below and downwind at `leafFrequency / 4`.
                    if tile.kind == 192
                        && wind.sway_enabled
                        && leaf_rng.next_range(0, tr_core::wind::leaf_frequency(wind.speed_current) / 4)
                            == 0
                    {
                        let wind_v = wind.speed_current;
                        // Below: the tile's bottom edge, if open.
                        let below_open = !world
                            .tile(tx, ty + 1)
                            .is_some_and(|t| t.active() && td::TILE_SOLID.has(t.kind) && t.kind != 192)
                            && world.tile(tx, ty + 1).is_none_or(|t| t.liquid == 0);
                        if below_open {
                            leaf_spawns.push(crate::render::foliage::LeafSpawn {
                                pos: glam::Vec2::new(tx as f32 * 16.0, ty as f32 * 16.0 + 16.0),
                                style: 910,
                            });
                        }
                        // Sideways downwind (|wind| > 0.2), 1/2 of the rolls.
                        let sideways = if wind_v > 0.2 {
                            1
                        } else if wind_v < -0.2 {
                            -1
                        } else {
                            0
                        };
                        if sideways != 0
                            && leaf_rng.next_range(0, 2) == 0
                            && !world
                                .tile(tx + sideways, ty)
                                .is_some_and(|t| t.active() && td::TILE_SOLID.has(t.kind) && t.kind != 192)
                            && world.tile(tx + sideways, ty).is_none_or(|t| t.liquid == 0)
                        {
                            let shift = if sideways < 0 { -10 } else { 0 };
                            leaf_spawns.push(crate::render::foliage::LeafSpawn {
                                pos: glam::Vec2::new(
                                    tx as f32 * 16.0 + 8.0 + 4.0 * sideways as f32 + shift as f32,
                                    ty as f32 * 16.0 + 8.0,
                                ),
                                style: 910,
                            });
                        }
                    }
                }
                // `TileDrawing.DrawSingleTile` closes with `DrawSingleTile_Flames`
                // (`TileDrawing.cs:1723`): the flame/glow overlays draw right
                // after the tile sprite, in the same row-major order, with their
                // own self-lit colours (the light map does not tint them).
                stats.quads +=
                    draw_tile_flames(r, lookups, world, tile, &dd, tx, ty, params, anim_time);
            }
        }
        if run_start >= 0 {
            scratch.cur.push((run_start, x1));
        }
        stats.dark_rects += merge_dark_row(r, white, scratch, ty);
    }
    // Emit rectangles that reach the bottom edge of the visible area.
    for o in scratch.open.drain(..) {
        r.push_sprite(
            [o[0] as f32 * TILE_PX, o[2] as f32 * TILE_PX],
            [
                (o[1] - o[0] + 1) as f32 * TILE_PX,
                (o[3] - o[2] + 1) as f32 * TILE_PX,
            ],
            white,
            [0.0, 0.0, 0.0, 1.0],
        );
        stats.dark_rects += 1;
    }

    if !crate::env::no_foliage() {
        stats.foliage = draw_tree_foliage(
            r,
            world,
            lookups,
            light,
            style,
            anim_time,
            wind,
            tree_wind_counter,
            &mut leaf_rng,
            leaf_spawns,
            x0,
            y0,
            x1,
            y1,
        );
    }
    for it in items {
        stats.items += it.draw(r, lookups, light, style, anim_time);
    }
    stats.quads += draw_npcs(
        r, npc_assets, npcs, light, style, anim_time, camera, zoom, screen,
    );
    if let Some(p) = player {
        stats.quads += draw_player(
            r,
            player_assets,
            lookups,
            p,
            held,
            armor,
            light,
            style,
            anim_time,
            camera,
            zoom,
            screen,
        );
    }
    for rp in remote {
        // The `netOffset` accumulator (the C#'s `LegacyPlayerRenderer.cs:304`:
        // `drawPlayer.position + drawPlayer.netOffset`): the snapshot position
        // plus the decaying visual lag. In single-player/hosting the offsets
        // are zero, so the drawn pixels are unchanged.
        let mut p = Player::spawn(world);
        let rx = rp.x + rp.off_x;
        let ry = rp.y + rp.off_y;
        p.x = rx;
        p.y = ry;
        p.prev_x = rx;
        p.prev_y = ry;
        p.vx = rp.vx;
        p.vy = rp.vy;
        p.facing = rp.facing;
        p.on_ground = rp.on_ground;
        p.leg_frame = rp.leg_frame;
        p.body_frame = rp.body_frame;
        p.gfx_off_y = 0.0;
        stats.quads += draw_player(
            r,
            player_assets,
            lookups,
            &p,
            None,
            [0, 0, 0],
            light,
            style,
            anim_time,
            camera,
            zoom,
            screen,
        );
        let sx = (p.x + player::WIDTH * 0.5 - camera.x) * zoom + screen.x * 0.5;
        let sy = (p.y - 14.0 - camera.y) * zoom + screen.y * 0.5;
        let tw = hud_text_width(r, lookups, &rp.name, 0.8);
        hud_text(
            r,
            lookups,
            sx + 1.0 - tw * 0.5,
            sy + 1.0,
            0.8,
            [0.0, 0.0, 0.0, 1.0],
            &rp.name,
        );
        hud_text(
            r,
            lookups,
            sx - tw * 0.5,
            sy,
            0.8,
            [1.0, 1.0, 1.0, 1.0],
            &rp.name,
        );
    }
    // `Main.DrawLiquid(bg: false, ...)` runs after entities: the foreground water
    // layer tints everything that stands in it.
    if !crate::env::no_liqfront() {
        stats.liquids += draw_waters(
            r,
            lookups,
            light,
            style,
            anim_time,
            &liquid_draws,
            LiquidPass::Foreground,
            water_style,
            liquid_alpha,
        );
    }
    if let Some((tx, ty)) = edit_target {
        let white = r.white_uv();
        let pos = [tx as f32 * TILE_PX, ty as f32 * TILE_PX];
        let c = [1.0, 1.0, 1.0, 0.55];
        for (dx, dy, w, h) in [
            (0.0, 0.0, 16.0, 1.0),
            (0.0, 15.0, 16.0, 1.0),
            (0.0, 0.0, 1.0, 16.0),
            (15.0, 0.0, 1.0, 16.0),
        ] {
            r.push_sprite([pos[0] + dx, pos[1] + dy], [w, h], white, c);
        }
    }
    stats.instances = stats.quads
        + stats.walls
        + stats.dark_rects
        + stats.foliage
        + stats.liquids
        + stats.waterfalls
        + stats.items;
    stats.draw_ms = t_total.elapsed().as_secs_f32() * 1000.0;
    stats
}

/// `WallDrawing.DrawWalls`: a 32x32 wall sprite lit by the cell's four corner colours
/// (`Lighting.GetCornerColors`). Flat-lit cells stay a single quad.
#[allow(clippy::too_many_arguments)]
fn push_wall_lit(
    r: &mut SpriteRenderer,
    pos: [f32; 2],
    uv: [f32; 4],
    light: &LightMap,
    style: &LightStyle,
    tx: i32,
    ty: i32,
    time: f32,
    paint: u8,
    fullbright: bool,
) -> u32 {
    let mut corners = light.corner4(tx, ty);
    if fullbright {
        // `WallDrawing.cs:96-99`: a Glow-coated wall samples white light
        // (`vertices = WallDrawing._glowPaintColors`).
        corners = [[1.0; 3]; 4];
    }
    let mut lo = f32::MAX;
    let mut hi = f32::MIN;
    for c in &corners {
        for ch in c {
            lo = lo.min(*ch);
            hi = hi.max(*ch);
        }
    }
    let tint = |c: [f32; 3]| {
        let c = style.shade(c, tx, ty, time);
        [c[0].min(1.0), c[1].min(1.0), c[2].min(1.0), 1.0]
    };
    if hi - lo < 1.5 / 255.0 {
        r.push_sprite_paint(pos, [32.0, 32.0], uv, tint(corners[0]), 0.0, paint);
        return 1;
    }
    let mut quads = 0;
    for (i, u) in [0.25f32, 0.75].iter().enumerate() {
        for (j, v) in [0.25f32, 0.75].iter().enumerate() {
            let x = 16.0 * i as f32;
            let y = 16.0 * j as f32;
            r.push_sprite_paint(
                [pos[0] + x, pos[1] + y],
                [16.0, 16.0],
                sub_uv_frac(uv, x, y, 16.0, 16.0, 32.0, 32.0),
                tint(bilerp_corners(corners, *u, *v)),
                0.0,
                paint,
            );
            quads += 1;
        }
    }
    quads
}

/// `WallDrawing.FullTile`: a full, opaque, unsloped block hides its wall completely.
fn tile_hides_wall(
    lookups: &mut assets::Lookups,
    atlas: Option<&Atlas>,
    kind: u16,
    frame_x: i16,
    frame_y: i16,
    slope: u8,
    half_brick: bool,
) -> bool {
    // `WorldGen.cs:15715`: leaf blocks are non-solid after generation, so they
    // never `FullTile` — the wall draws behind them.
    if tree::is_tree(kind) || kind == 192 || slope != 0 || half_brick || frame_x < 0 || frame_y < 0
    {
        return false;
    }
    let Some(atlas) = atlas else {
        return false;
    };
    match lookups.frame_tex(atlas, kind, frame_x, frame_y) {
        Some(f) => f.opaque,
        None => lookups.plain_tex(kind).is_some_and(|f| f.opaque),
    }
}

/// `WorldGen.SolidTile`: active, solid, not a platform, and a full block.
#[inline]
pub(crate) fn solid_full(world: &World, x: i32, y: i32) -> bool {
    world.tile(x, y).is_some_and(|t| {
        t.active()
            && td::TILE_SOLID.has(t.kind)
            && !td::TILE_SOLID_TOP.has(t.kind)
            && !t.half_brick()
            && t.slope() == 0
    })
}

/// Bilinear sample of `LightMap::corner4` (TL, TR, BL, BR).
#[inline]
pub(crate) fn bilerp_corners(c: [[f32; 3]; 4], u: f32, v: f32) -> [f32; 3] {
    let mut out = [0.0f32; 3];
    for k in 0..3 {
        let top = c[0][k] + (c[1][k] - c[0][k]) * u;
        let bottom = c[2][k] + (c[3][k] - c[2][k]) * u;
        out[k] = top + (bottom - top) * v;
    }
    out
}

/// Tint and push one flat-lit sprite.
#[allow(clippy::too_many_arguments)]
#[inline]
/// The plain flat arm's source rect: C#'s `normalTileRect`, the ONE rectangle
/// `TileDrawing.DrawSingleTile`'s switch mutates (`TileDrawing.cs:1007`) and
/// which only `DrawBasicTile`'s plain arm draws (`:1523`). That is why case
/// 80's biome-cactus row shift (`src_y_shift`, `:1132-1147`) applies here and
/// to no other arm — the half-brick, slice, slope and edge arms rebuild their
/// rects from `tileFrameX/Y + addFrX/addFrY` and stay unshifted.
fn flat_src_uv(
    dd: &tr_core::drawdata::TileDrawData,
    tile: &tr_core::tile::Tile,
    tf: assets::FrameTex,
    atlas: Option<&Atlas>,
    lookups: &assets::Lookups,
) -> [f32; 4] {
    if dd.src_y_shift == 0 {
        return tf.uv;
    }
    atlas
        .and_then(|a| {
            lookups.rect_uv(
                a,
                tile.kind,
                dd.frame_x_override.unwrap_or(tile.frame_x) + dd.add_fr_x,
                dd.frame_y_override.unwrap_or(tile.frame_y) + dd.add_fr_y + dd.src_y_shift,
                dd.width.max(16) as u32,
                dd.height.max(16) as u32,
            )
        })
        .unwrap_or(tf)
        .uv
}

/// `TileDrawing.DrawTile_MinecartTrack` (`TileDrawing.cs:3786-3824`): the
/// minecart track — the back layer (`frameY`), the front layer (`frameX`), the
/// hanging ties below the cell for the "down" side connections (frames 36/37)
/// and the bumper above it (frames 38/39), each layer carrying its own paint
/// from `Minecart.TrackColors` (`Minecart.cs:1321-1437`). The C# first draws
/// `DrawTile_BackRope` (`:3766-3784`) — the rope strand a hanging track hangs
/// from; the port has no `WorldGen.IsRope`/`GetRopeEnds`, so that strand is
/// parked (ANALYSIS, TILE-DRAW-7) and the track layers draw on their own.
#[allow(clippy::too_many_arguments)]
fn draw_minecart_track(
    r: &mut SpriteRenderer,
    world: &World,
    lookups: &assets::Lookups,
    atlas: Option<&Atlas>,
    light: [f32; 3],
    style: &LightStyle,
    tx: i32,
    ty: i32,
    time: f32,
    tile: &tr_core::tile::Tile,
) -> u32 {
    let Some(atlas) = atlas else {
        return 0;
    };
    let c = style.shade(light, tx, ty, time);
    let mut quads = 0;
    for (rect, dy, paint) in crate::minecart::track_layers(world, tx, ty, tile) {
        let Some(uv) = lookups.rect_uv(
            atlas,
            tile.kind,
            rect[0] as i16,
            rect[1] as i16,
            rect[2] as u32,
            rect[3] as u32,
        ) else {
            continue;
        };
        r.push_sprite_paint(
            [tx as f32 * TILE_PX, (ty + dy) as f32 * TILE_PX],
            [TILE_PX, TILE_PX],
            uv.uv,
            [c[0], c[1], c[2], 1.0],
            0.0,
            paint,
        );
        quads += 1;
    }
    quads
}

fn push_lit(
    r: &mut SpriteRenderer,
    pos: [f32; 2],
    size: [f32; 2],
    uv: [f32; 4],
    light: [f32; 3],
    style: &LightStyle,
    tx: i32,
    ty: i32,
    time: f32,
    paint: u8,
    dd: &tr_core::drawdata::TileDrawData,
) {
    // `finalColor = tileLight * 0.5f` for the cobwebs (`TileDrawing.cs:1078`)
    // and the shard alpha (`:1091`) ride the shaded light.
    let c = style.shade(light, tx, ty, time);
    let k = dd.light_scale;
    r.push_sprite_paint(
        pos,
        size,
        uv,
        [c[0] * k, c[1] * k, c[2] * k, dd.src_alpha],
        0.0,
        paint,
    );
}

/// Sub-rectangle of a 16x16 frame's UV rect, in tile pixels.
#[inline]
fn sub_uv(uv: [f32; 4], x: f32, y: f32, w: f32, h: f32) -> [f32; 4] {
    sub_uv_frac(uv, x, y, w, h, 16.0, 16.0)
}

/// `sub_uv` over a destination rect that is not 16x16.
#[inline]
pub(crate) fn sub_uv_frac(
    uv: [f32; 4],
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    full_w: f32,
    full_h: f32,
) -> [f32; 4] {
    let du = (uv[2] - uv[0]) / full_w;
    let dv = (uv[3] - uv[1]) / full_h;
    [
        uv[0] + du * x,
        uv[1] + dv * y,
        uv[0] + du * (x + w),
        uv[1] + dv * (y + h),
    ]
}

/// `DrawBasicTile` slope path: eight 2 px columns plus a 2 px strip, sampling the
/// tile's own 16x16 frame. Slopes 1/2 are ground slopes, 3/4 ceiling slopes.
#[allow(clippy::too_many_arguments)]
fn push_slope(
    r: &mut SpriteRenderer,
    pos: [f32; 2],
    uv: [f32; 4],
    light: [f32; 3],
    style: &LightStyle,
    tx: i32,
    ty: i32,
    time: f32,
    slope: u8,
) -> u32 {
    let c = style.shade(light, tx, ty, time);
    let color = [c[0], c[1], c[2], 1.0];
    let mut quads = 0;
    for i in 0..8u8 {
        let f = i as f32 * 2.0;
        let (dx, dy, sx, sy, w, h) = match slope {
            1 => (f, f, f, 0.0, 2.0, 14.0 - f),
            2 => (14.0 - f, f, 14.0 - f, 0.0, 2.0, 14.0 - f),
            3 => (f, 0.0, f, f, 2.0, 16.0 - f),
            _ => (14.0 - f, 0.0, 14.0 - f, f, 2.0, 16.0 - f),
        };
        if h <= 0.0 {
            continue;
        }
        r.push_sprite(
            [pos[0] + dx, pos[1] + dy],
            [w, h],
            sub_uv(uv, sx, sy, w, h),
            color,
        );
        quads += 1;
    }
    let strip_y = if slope <= 2 { 14.0 } else { 0.0 };
    r.push_sprite(
        [pos[0], pos[1] + strip_y],
        [16.0, 2.0],
        sub_uv(uv, 0.0, strip_y, 16.0, 2.0),
        color,
    );
    quads + 1
}

/// Half bricks keep the top half of the art and sit on the bottom 8 px of the cell.
fn push_half(
    r: &mut SpriteRenderer,
    pos: [f32; 2],
    uv: [f32; 4],
    light: [f32; 3],
    style: &LightStyle,
    tx: i32,
    ty: i32,
    time: f32,
) {
    let c = style.shade(light, tx, ty, time);
    r.push_sprite(
        [pos[0], pos[1] + 8.0],
        [16.0, 8.0],
        sub_uv(uv, 0.0, 0.0, 16.0, 8.0),
        [c[0], c[1], c[2], 1.0],
    );
}

/// Bug 19: the half-brick arm with `DrawSingleTile`'s local rectangle Y shift
/// (`TileDrawing.cs:1298` draws `rectangle`, which case 80 mutated) — the
/// source row moves by `src_y`, the draw box stays the top 8 rows.
/// `DrawSingleTile_SlicedBlock`: the block becomes a 4/8/4 grid whose outer slices
/// take light from the adjacent tile, smoothing the seams between blocks.
#[allow(clippy::too_many_arguments)]
fn push_sliced(
    r: &mut SpriteRenderer,
    pos: [f32; 2],
    uv: [f32; 4],
    light: &LightMap,
    style: &LightStyle,
    tx: i32,
    ty: i32,
    time: f32,
    high_threshold: f32,
) -> u32 {
    const SLICE_POS: [(f32, f32, f32, f32); 9] = [
        (0.0, 0.0, 4.0, 4.0),
        (4.0, 0.0, 8.0, 4.0),
        (12.0, 0.0, 4.0, 4.0),
        (0.0, 4.0, 4.0, 8.0),
        (4.0, 4.0, 8.0, 8.0),
        (12.0, 4.0, 4.0, 8.0),
        (0.0, 12.0, 4.0, 4.0),
        (4.0, 12.0, 8.0, 4.0),
        (12.0, 12.0, 4.0, 4.0),
    ];
    let own = light.get(tx, ty);
    let mut quads = 0;
    if own[0].max(own[1]).max(own[2]) > high_threshold {
        let slices = light.slice9(tx, ty);
        for (i, &(x, y, w, h)) in SLICE_POS.iter().enumerate() {
            let c = style.shade(slices[i], tx, ty, time);
            r.push_sprite(
                [pos[0] + x, pos[1] + y],
                [w, h],
                sub_uv(uv, x, y, w, h),
                [c[0], c[1], c[2], 1.0],
            );
            quads += 1;
        }
    } else {
        let slices = light.quad4(tx, ty);
        const QUAD_POS: [(f32, f32); 4] = [(0.0, 0.0), (8.0, 0.0), (0.0, 8.0), (8.0, 8.0)];
        for (i, &(x, y)) in QUAD_POS.iter().enumerate() {
            let c = style.shade(slices[i], tx, ty, time);
            r.push_sprite(
                [pos[0] + x, pos[1] + y],
                [8.0, 8.0],
                sub_uv(uv, x, y, 8.0, 8.0),
                [c[0], c[1], c[2], 1.0],
            );
            quads += 1;
        }
    }
    quads
}

/// XNA's `SpriteBatch` blends premultiplied (`TileBatch.Begin` passes a null
/// blend state → `BlendState.AlphaBlend`, `TileBatch.cs:217`): with a tint
/// alpha of 0 the flame colour is added to the destination in full
/// (`out.rgb = tex.rgb * tint + dst`). The renderer's single alpha blend
/// cannot express `src = One`, so alpha-zero tints are pushed with alpha 1 —
/// on the flame art's opaque texels the two blend equations agree exactly,
/// and the transparent texels are discarded in both. The partial-alpha tints
/// (the furniture/jar colours with `a > 0`) keep their alpha, which dims the
/// flame slightly against vanilla's full add.
#[inline]
/// `DrawSingleTile_Flames` (`TileDrawing.cs:2684-3491`): the tile-flame
/// spread (`Main.tileFlame` types — torches, candles, chandeliers...) and the
/// per-type glow overlays, drawn right after the tile sprite with self-lit
/// colours. The glow branches precede the flame block in the C# `if` chain,
/// but no tile type matches both, so the two vecs can push in either order.
#[allow(clippy::too_many_arguments)]
fn draw_tile_flames(
    r: &mut SpriteRenderer,
    lookups: &mut assets::Lookups,
    world: &World,
    tile: &tr_core::Tile,
    dd: &tr_core::drawdata::TileDrawData,
    tx: i32,
    ty: i32,
    params: &LightParams,
    anim_time: f32,
) -> u32 {
    use tr_core::flame::{self, FlameEnv, GlowSheet};
    let kind = tile.kind;
    if !flame::tile_flame(kind) && !flame::has_glow_branch(kind) {
        return 0;
    }
    let env = FlameEnv {
        tile_frame_seed: flame::tile_frame_seed_at(anim_time),
        global_time: params.global_time,
        disco: params.disco,
        mouse_text_color: params.mouse_text_color,
        moon_phase: world.moon_phase,
        sundial_cooldown: params.sundial_cooldown,
        moondial_cooldown: params.moondial_cooldown,
    };
    let anchor = [
        tx as f32 * TILE_PX - (dd.width as f32 - 16.0) * 0.5,
        ty as f32 * TILE_PX + dd.top as f32,
    ];
    let mut quads = 0u32;

    // The glow branches (`TileDrawing.cs:2684-2825`, `:3463-3491`).
    for g in flame::glow_quads(kind, tx, ty, tile.frame_x, tile.frame_y, dd, &env) {
        let (uv, size) = match g.sheet {
            GlowSheet::Flames(sheet) => (
                lookups.flame_uv(
                    sheet,
                    g.src[0],
                    g.src[1],
                    g.src[2].max(1) as u32,
                    g.src[3].max(1) as u32,
                ),
                [g.src[2] as f32, g.src[3] as f32],
            ),
            GlowSheet::GlowMask(id) if g.src[2] == 0 => {
                // `value3.Frame()`: the whole sheet at its own pixel size.
                let name = format!("Glow_{id}");
                match lookups.glow_frame_uv(&name) {
                    Some((uv, w, h)) => (Some(uv), [w, h]),
                    None => (None, [0.0; 2]),
                }
            }
            GlowSheet::GlowMask(id) => (
                lookups.glow_uv(
                    &format!("Glow_{id}"),
                    g.src[0],
                    g.src[1],
                    g.src[2] as u32,
                    g.src[3] as u32,
                ),
                [g.src[2] as f32, g.src[3] as f32],
            ),
            GlowSheet::Named(name) if g.src[2] == 0 => match lookups.glow_frame_uv(name) {
                Some((uv, w, h)) => (Some(uv), [w, h]),
                None => (None, [0.0; 2]),
            },
            GlowSheet::Named(name) => (
                lookups.glow_uv(name, g.src[0], g.src[1], g.src[2] as u32, g.src[3] as u32),
                [g.src[2] as f32, g.src[3] as f32],
            ),
        };
        let Some(uv) = uv else { continue };
        r.push_sprite([anchor[0] + g.dx, anchor[1] + g.dy], size, uv, g.color);
        quads += 1;
    }

    // The `Main.tileFlame` spread (`TileDrawing.cs:2826-3462`).
    if flame::tile_flame(kind) {
        let sheet = flame::flame_sheet(kind);
        for q in flame::flame_quads(
            kind,
            tx,
            ty,
            tile.frame_x,
            tile.frame_y,
            dd.add_fr_x,
            dd.add_fr_y,
            env.tile_frame_seed,
            env.global_time,
            env.disco,
        ) {
            let Some(uv) = lookups.flame_uv(
                sheet,
                q.src_x as i32,
                q.src_y as i32,
                dd.width as u32,
                dd.height as u32,
            ) else {
                continue;
            };
            r.push_sprite(
                [anchor[0] + q.dx, anchor[1] + q.dy],
                [dd.width as f32, dd.height as f32],
                uv,
                q.color,
            );
            quads += 1;
        }
    }
    quads
}

#[cfg(test)]
mod tests {
    use super::*;
    use tr_core::wind::Wind;

    /// 40x40 world, no tiles — the `sunflower_sway` probe needs `World::tile`
    /// inside the 2x5 `InAPlaceWithWind` window to answer "no liquid, no
    /// blocking wall" (the `object_world` fixture shape without the floor).
    fn empty_world() -> World {
        let mut w = World::new("t".into(), 40, 40);
        w.compute_column_surface();
        w.compute_chunks();
        w
    }

    /// A wind above the 0.08..0.18 gate (so the cycle lives) under an infinite
    /// surface (so the surface band never zeroes it).
    fn windy() -> Wind {
        let mut wind = Wind::new(7);
        wind.speed_current = 0.5;
        wind.world_surface = f64::INFINITY;
        wind
    }

    // The per-cell transform of `DrawMultiTileGrassInWind`
    // (`TileDrawing.cs:8916-9041`): `num3 = 1 - (j - topLeftY + 1) / 5` per row
    // (0.8/0.6/0.4/0.2 for the flower's four rows), the vertical offset
    // `|windCycle| * 2 * num3`, the rotation `windCycle * 0.07 * num3` about
    // the flower's bottom-center anchor `(topLeftX*16 + 16, topLeftY*16 + 80)`
    // — the special pass's `sizeX = 2, sizeY = 5` (`:8748-8752`).
    #[test]
    fn test_DW_4_1_sunflower_sway_transform_per_row() {
        let wind = windy();
        let world = empty_world();
        // A style-0 flower with its top-left at (10, 20): the four rows' cells
        // carry frameY 0/18/36/54 and the two columns frameX 0/18.
        let mut per_row: Vec<SunflowerSway> = Vec::new();
        for row in 0..4_i32 {
            for col in 0..2_i32 {
                let (tx, ty) = (10 + col, 20 + row);
                let fx = (18 * col) as i16;
                let fy = (18 * row) as i16;
                let sw = sunflower_sway(&wind, &world, tx, ty, fx, fy, 0.0);
                // The pivot reconstructs the same anchor from every cell:
                // (topLeftX*16 + 16 - tx*16, topLeftY*16 + 80 - ty*16).
                assert_eq!(
                    sw.pivot,
                    [(16 - 16 * col) as f32, (80 - 16 * row) as f32],
                    "pivot of cell ({tx}, {ty})"
                );
                per_row.push(sw);
            }
        }
        // The rows scale exactly 0.8 : 0.6 : 0.4 : 0.2 — dy ratios 4:3:2:1.
        let (d0, d1, d2, d3) = (
            per_row[0].dy,
            per_row[2].dy,
            per_row[4].dy,
            per_row[6].dy,
        );
        assert!(d3 > 0.0, "the bottom row still sways (num3 = 0.2)");
        assert!((d0 / d3 - 4.0).abs() < 1e-4, "{d0} / {d3}");
        assert!((d1 / d3 - 3.0).abs() < 1e-4, "{d1} / {d3}");
        assert!((d2 / d3 - 2.0).abs() < 1e-4, "{d2} / {d3}");
        // |rot| / dy = 0.07 / 2 for every cell, whatever the cycle's sign.
        for sw in &per_row {
            assert!(
                (sw.rot.abs() / sw.dy - 0.035).abs() < 1e-4,
                "rot {} dy {}",
                sw.rot,
                sw.dy
            );
        }
        // A style-1 flower (frameX offset 36) reconstructs the same top-left.
        let sw1 = sunflower_sway(&wind, &world, 11, 20, 54, 0, 0.0);
        assert_eq!(sw1.pivot, [0.0, 80.0]);
        let sw1_bottom = sunflower_sway(&wind, &world, 10, 23, 36, 54, 0.0);
        assert_eq!(sw1_bottom.pivot, [16.0, 32.0]);
    }

    // The counter moves the flower: feeding the cadence
    // `1/420 + num*5/420` (`TileDrawing.cs:374`) changes both the vertical
    // offset and the rotation of the same cell.
    #[test]
    fn test_DW_4_1_sunflower_sway_tracks_the_counter() {
        let wind = windy();
        let world = empty_world();
        let before = sunflower_sway(&wind, &world, 10, 20, 0, 0, 0.0);
        // One in-game second of the fixed-step advance at |wind| = 0.5.
        let mut counter = 0.0_f32;
        for _ in 0..60 {
            tr_core::wind::advance_sunflower_counter(&mut counter, 0.5);
        }
        let after = sunflower_sway(&wind, &world, 10, 20, 0, 0, counter);
        assert_ne!(before.dy, after.dy, "the vertical offset tracks the counter");
        assert_ne!(before.rot, after.rot, "the rotation tracks the counter");
    }

    // `GetWindCycle`'s gates freeze the flower (`TileDrawing.cs:7676-7700`):
    // below the surface band, with sway disabled, or outside
    // `InAPlaceWithWind(topLeft, 2, 5)` (`:8923`, `:8961`) every cell draws
    // statically.
    #[test]
    fn sunflower_sway_gates_freeze_the_flower() {
        let world = empty_world();
        // Below the surface band (y >= worldSurface): cycle 0.
        let mut wind = windy();
        wind.world_surface = 25.0;
        let sw = sunflower_sway(&wind, &world, 10, 30, 0, 0, 0.0);
        assert_eq!(sw.dy, 0.0);
        assert_eq!(sw.rot, 0.0);
        // Sway disabled (`SettingsEnabled_TilesSwayInWind`): cycle 0.
        let mut wind = windy();
        wind.sway_enabled = false;
        let sw = sunflower_sway(&wind, &world, 10, 20, 0, 0, 0.0);
        assert_eq!(sw.dy, 0.0);
        assert_eq!(sw.rot, 0.0);
        // A wind-blocking wall inside the 2x5 probe window (`WallID.Sets.
        // AllowsWind` fails for wall 1): `num3 = 0` — dy and rot are 0 even
        // though the cycle is not.
        let mut wind = windy();
        wind.world_surface = f64::INFINITY;
        let mut walled = empty_world();
        let i = walled.index(10, 22);
        walled.tiles[i].wall = 1;
        let sw = sunflower_sway(&wind, &walled, 11, 20, 18, 0, 0.0);
        assert_eq!(sw.dy, 0.0);
        assert_eq!(sw.rot, 0.0);
        assert_eq!(sw.pivot, [0.0, 80.0]);
    }

    fn sheet_atlas(names: &[&str], w: u32, h: u32) -> Atlas {
        let images = names
            .iter()
            .map(|&name| tr_render::AtlasImage {
                name: name.to_string(),
                width: w,
                height: h,
                rgba: vec![180; (w * h * 4) as usize],
                layout: tr_render::SheetLayout::Tile,
            })
            .collect();
        Atlas::build(images, 1024)
    }

    // Bug 19/19b: the cactus draw. `DrawSingleTile` case 80 shifts the source
    // rectangle down by the ground sand's biome row (`TileDrawing.cs:1132-1147`),
    // and only the plain flat arm draws that rect (`:1523`). A port that drops
    // the shift shows the wrong cactus sprite in the evil/hallow/crimson desert.
    #[test]
    fn test_BUG19b_the_cactus_flat_draw_carries_the_biome_row_shift() {
        let atlas = sheet_atlas(&["Tiles_80"], 270, 252);
        let mut lookups = assets::Lookups::build(
            &atlas,
            [(80_u16, "Tiles_80".to_string())].into_iter(),
            std::iter::empty(),
            None,
            None,
            None,
            None,
            None,
        );
        let tf = lookups
            .frame_tex(&atlas, 80, 0, 0)
            .expect("the cactus frame resolves");
        let mut yy = |ground: u16| {
            let mut w = World::new("t".into(), 40, 40);
            for (x, y, kind) in [(20_i32, 19_i32, 80_u16), (20, 20, ground)] {
                let i = w.index(x, y);
                w.tiles[i].kind = kind;
                w.tiles[i].flags = tr_core::tile::FLAG_ACTIVE;
            }
            let mut dd = tr_core::drawdata::TileDrawData::default();
            let tile = *w.tile(20, 19).unwrap();
            dd.apply_world_corrections(&w, 80, 20, 19, 0, 0, 0);
            let uv = flat_src_uv(&dd, &tile, tf, Some(&atlas), &lookups);
            (dd.src_y_shift, uv[1])
        };
        let (plain_shift, plain_y) = yy(53); // normal sand
        let (evil_shift, evil_y) = yy(112); // ebonstone sand
        assert_eq!(plain_shift, 0, "normal sand keeps the cell row");
        assert_eq!(evil_shift, 54, "evil sand moves the sprite 54 px down");
        assert!(evil_y > plain_y, "the evil draw samples a lower row");
        assert!(
            (evil_y - plain_y) * 252.0 > 50.0,
            "the shift reaches the UV: {} vs {}",
            evil_y,
            plain_y
        );
        // `src_y_shift == 0` must reproduce the plain frame exactly.
        assert_eq!(plain_y, tf.uv[1], "no shift, no UV change");
    }

    // A tile whose frames were never written (`frameX/frameY = -1`, the state
    // the old seed bug left behind) resolves no frame at all — the plain-sheet
    // fallback that used to draw sheet-corner garbage is gated out, while the
    // same kind with real frames still resolves.
    #[test]
    fn test_DW_4_3_unframed_tile_draws_nothing() {
        let atlas = sheet_atlas(&["Tiles_27", "Tiles_82"], 126, 148);
        let mut lookups = assets::Lookups::build(
            &atlas,
            [(27_u16, "Tiles_27".to_string()), (82, "Tiles_82".to_string())]
                .into_iter(),
            std::iter::empty(),
            None,
            None,
            None,
            None,
            None,
        );
        let dd = tr_core::drawdata::TileDrawData::default();
        // The fallback WOULD have produced a sprite (the plain sheet cell
        // exists) — the guard is what keeps the unframed tile blank.
        assert!(lookups.plain_tex(27).is_some());
        assert!(lookups.plain_tex(82).is_some());
        for (kind, fx, fy) in [(27_u16, -1_i16, -1_i16), (82, -1, -1), (82, 0, -1)] {
            let frame = resolve_tile_frame(
                &mut lookups,
                Some(&atlas),
                false,
                0,
                kind,
                fx,
                fy,
                &dd,
            );
            assert!(frame.is_none(), "kind {kind} frame ({fx},{fy}) draws nothing");
        }
        // Real frames still resolve: the herbs sit on the 18 px grid, the
        // sunflower's bottom row draws the 16x18 slice.
        let herb = resolve_tile_frame(&mut lookups, Some(&atlas), false, 0, 82, 18, 0, &dd);
        assert!(herb.is_some(), "herb frame (18,0) resolves");
        let bottom = resolve_tile_frame(
            &mut lookups,
            Some(&atlas),
            false,
            0,
            27,
            0,
            54,
            &tr_core::drawdata::TileDrawData {
                height: 18,
                ..Default::default()
            },
        );
        assert!(bottom.is_some(), "sunflower bottom row resolves");
    }
}
