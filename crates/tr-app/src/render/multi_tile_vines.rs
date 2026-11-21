//! `TileDrawing.DrawMultiTileVines` / `DrawMultiTileVinesInWind` — the
//! hanging-object bucket (W4-g84 follow-up, the general-roster census). C#
//! never draws these cells through `DrawSingleTile`: the draw-area switch
//! registers the object's top-left cell as a `TileCounterType.MultiTileVine`
//! special point (`TileDrawing.cs:549-599`), and `PostDrawTiles` draws the
//! object (`DrawMultiTileVines` `:8829-8888`, the size arm `:8845-8884`;
//! `DrawMultiTileVinesInWind` `:9238-9603`) about its TOP-centre anchor.
//!
//! The roster (the registration census vs the port's prior kind arms):
//! 34 chandeliers, 95/126/444 Chinese lanterns / disco ball / bee hive,
//! 454 pigronata, 42/270/271/572/581/660 hanging bottles + jars, 91 banners,
//! 465/591/592 war-table banner / suspended pots / suspended brazier, and
//! 698 the dead-cells display jar. Before this module every one of them drew
//! through the static-generic raised arm — no wind lean, no platform lift,
//! no jar glow / lid / item-icon specials.
//!
//! The port keeps the fused row-major tile pass and draws each cell inline
//! with the special pass's per-cell transform (the sunflower / bulb
//! precedent, the documented z-order approximation). The object's top-left
//! is reconstructed from the frame grid (`col = frameX/18 % sizeX`,
//! `row = frameY/18 % sizeY` — the `CheckChand`/`Check2xX`/`CheckSuper`
//! pattern, `WorldGen.cs:52808-52842`/`:38001-38028`/`:48140-48141`).
//!
//! Deviations (ANALYSIS "The MultiTileVines roster (W4-g84 follow-up)"):
//! - the deferred special-point ordering is approximated inline;
//! - the `DrawAnimatedTile_AdjustForVisionChangers` /
//!   `DrawTiles_GetLightOverride` chain (`:9487-9488`) is the d56 unwired
//!   lane — identity outside the unported vision potions + Glow coating;
//! - the `_rand.Next(4)` flag5 roll + dust (`:9485-9492`) are the d60
//!   unwired lane — adding the roll would change the `_rand` stream order;
//! - the shared flame seed (`:9457`/`:9590-9594`) latches the object's
//!   first cell in the C#'s x-outer/y-inner walk (the top-left); the inline
//!   pass derives it from the top-left cell directly (same value, the
//!   z-order approximation);
//! - the 698 item icon's `ItemSlot.DrawItemIcon` colour chain
//!   (`GetItemLight`/`GetAlpha`/glowMask, `:9580`) is the d57 furniture
//!   lane — the icon draws with the environment light tint only.

use tr_core::tiledata as td;
use tr_core::wind::Wind;
use tr_core::World;

/// `DrawMultiTileVines`' per-type `(sizeX, sizeY)` (`TileDrawing.cs:8845-8884`).
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

/// Whether `kind` registers a MultiTileVine special point
/// (`TileDrawing.cs:549-599`). The 549 seaweed is a ReverseVine (bucket 7),
/// NOT this bucket, and the VineThreads (52/62/115/205/382/528/636/638) are
/// the strip pass (`render/vines.rs`).
pub(crate) fn is_multi_tile_vine(kind: u16) -> bool {
    matches!(
        kind,
        34 | 95 | 126 | 444 | 454 | 42 | 270 | 271 | 572 | 581 | 660 | 91 | 465 | 591 | 592
            | 698
    )
}

/// The object's top-left reconstructed from the cell's frame grid:
/// `col = frameX/18 % sizeX`, `row = frameY/18 % sizeY`
/// (`WorldGen.CheckChand :52808-52842`, `Check2xX :38001-38028`,
/// `CheckSuper :48140-48141`). Returns `(topLeftX, topLeftY, sizeX, sizeY)`.
pub(crate) fn vine_top_left(
    kind: u16,
    tx: i32,
    ty: i32,
    frame_x: i16,
    frame_y: i16,
) -> (i32, i32, i32, i32) {
    let (sx, sy) = multi_tile_vine_size(kind);
    let col = (frame_x as i32 / 18).rem_euclid(sx);
    let row = (frame_y as i32 / 18).rem_euclid(sy);
    (tx - col, ty - row, sx, sy)
}

/// `WorldGen.IsBelowANonHammeredPlatform` (`WorldGen.cs:38289-38301`): the
/// cell above is an active, un-hammered (no half-brick, no slope) platform.
pub(crate) fn is_below_a_non_hammered_platform(world: &World, x: i32, y: i32) -> bool {
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
/// replaced by `num5`'s constant. Returns `(Option<f32>, f32, bool, f32)`.
pub(crate) fn planter_style(frame_x: i16, frame_y: i16) -> (Option<f32>, f32, bool, f32) {
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
pub(crate) fn bottle_style(frame_y: i16) -> (Option<f32>, f32, bool, f32) {
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

/// One hanging-object cell's wind transform — the `Draw` call's position,
/// rotation and pivot (`TileDrawing.cs:9493-9512`). Pure world/wind state,
/// so the maths is unit-testable without a GPU (the `BulbSway` shape).
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct VineSway {
    /// The unrotated sprite top-left (`vector3 + (0, vector4.Y)`,
    /// `:9493-9502`): the cell's `vector3 = (i*16, j*16 + tileTop) +
    /// vector2` shifted by the vertical sway. The pivot lands at
    /// `pos + origin = vector5` (the anchor + vertical sway).
    pub pos: [f32; 2],
    /// `num8 = windCycle * num2 * num7` (`:9504`) — the rotation about the
    /// top-centre anchor.
    pub rot: f32,
    /// `origin = vector - vector3` (`:9496`) — the rotation pivot relative
    /// to the cell sprite's top-left.
    pub origin: [f32; 2],
    /// The 592 suspended brazier draws `GlowMask[294]` over the cell
    /// (`:9437-9438`, `:9585-9588`).
    pub glow_592: bool,
    /// The 660 shimmerfly jar's bottom row draws the `Extra_260` white +
    /// HSL glow pair (`:9505-9519`).
    pub fairy_660: bool,
    /// The 698 dead-cells jar draws its two lid rows + the item icon
    /// (`:9520-9584`).
    pub jar_698: bool,
}

/// The wind transform of the hanging-object cell at `(tx, ty)` whose
/// object's top-left is `(tlx, tly)` with size `(sx, sy)`. Reads the anchor
/// tile for the style switch; `tile_top` is the cell's `GetTileDrawData`
/// `tileTop`. The anchor `vector` sits at the object's top-centre minus 2
/// (`:9243-9248`), lifted 8 px under a non-hammered platform (`:9250-9254`).
#[allow(clippy::too_many_arguments)]
pub(crate) fn vine_sway(
    wind: &Wind,
    world: &World,
    tx: i32,
    ty: i32,
    tlx: i32,
    tly: i32,
    sx: i32,
    sy: i32,
    tile_top: i16,
    sunflower_wind_counter: f32,
) -> VineSway {
    let kind = world.tile(tlx, tly).map(|t| t.kind).unwrap_or(0);
    let anchor_fx = world.tile(tlx, tly).map(|t| t.frame_x).unwrap_or(0);
    let anchor_fy = world.tile(tlx, tly).map(|t| t.frame_y).unwrap_or(0);
    // `windCycle = GetWindCycle(...)` (`:9240`); `num = windCycle` (`:9241`).
    let base_cycle = wind.cycle(tlx, tly, sunflower_wind_counter);
    let mut wind_cycle = base_cycle;
    // `windCycle += GetHighestWindGridPushComplex(tlx, tly, sx, sy, 60,
    // 1.26, 3, swapLoopDir: true)` (`:9242`).
    wind_cycle +=
        wind.get_highest_wind_grid_push_complex(tlx, tly, sx, sy, 60, 1.26, 3, true);
    // `vector = (tlx*16 + sx*16*0.5, tly*16)` (`:9243`).
    let mut vector = [tlx as f32 * 16.0 + sx as f32 * 8.0, tly as f32 * 16.0];
    // `vector2 = (0, -2); vector += vector2` (`:9247-9248`).
    let mut vector2 = [0.0f32, -2.0];
    vector[1] += vector2[1];
    // The platform lift (`:9250-9254`): the 465/591/592 family checks BOTH
    // columns, the rest only when 1-wide.
    let wide_family = kind == 465 || kind == 591 || kind == 592;
    let lifted = if !wide_family {
        sx == 1 && is_below_a_non_hammered_platform(world, tlx, tly)
    } else {
        is_below_a_non_hammered_platform(world, tlx, tly)
            && is_below_a_non_hammered_platform(world, tlx + 1, tly)
    };
    if lifted {
        vector[1] -= 8.0;
        vector2[1] -= 8.0;
    }
    // The style state (`:9257-9262`): `num3` the optional height override,
    // `num5` the Y sway scale, `flag2` the +16 anchor lift, `flag3` the 698
    // wind-cancel, `num2` the rotation multiplier (negated at `:9451`).
    // `num4` the X sway scale feeds `vector4.X` (`:9495`) which the C#
    // DISCARDS (only `vector4.Y` is used, `:9502`) — a dead write, kept out.
    let mut num3: Option<f32> = None;
    let mut num5 = -4.0f32;
    let mut flag2 = false;
    let mut flag3 = false;
    let mut num2 = 0.15f32;
    let mut glow_592 = false;
    let mut off_x = 0.0f32;
    match kind {
        34 | 126 => {
            let (n3, n5, f2, scale) = planter_style(anchor_fx, anchor_fy);
            num3 = n3;
            num5 = n5;
            flag2 = f2;
            num2 *= scale;
        }
        42 => {
            let (n3, n5, f2, scale) = bottle_style(anchor_fy);
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
            num5 = -2.0;
        }
        592 => {
            num5 = -2.0;
            glow_592 = true;
        }
        698 => {
            num5 = -1.0;
            // `offSet.X -= 10f` (`:9443`) — shifts the anchor AND the cell
            // (the origin cancels), so the whole jar draws 10 px left.
            off_x = -10.0;
            flag3 = true;
        }
        _ => {}
    }
    if flag2 {
        vector[1] += 16.0; // `:9447-9450`.
    }
    num2 *= -1.0; // `:9451`.
    // `flag4 = InAPlaceWithWind(tlx, tly, sx, sy)`; `flag3 || !flag4 ->
    // windCycle -= num` (`:9452-9456`) — leaves only the grid push.
    let flag4 = wind.in_a_place_with_wind(world, tlx, tly, sx, sy);
    if flag3 || !flag4 {
        wind_cycle -= base_cycle;
    }
    // `num7 = (j - topLeftY + 1) / sizeY`, 0 -> 0.1 (`:9471-9475`), then the
    // style override (`:9476-9479`) and the flag2 top-row zero (`:9480-9483`).
    let row = ty - tly;
    let mut num7 = (row + 1) as f32 / sy as f32;
    if num7 == 0.0 {
        num7 = 0.1;
    }
    if let Some(n3) = num3 {
        num7 = n3;
    }
    if flag2 && row == 0 {
        num7 = 0.0;
    }
    // `vector3 = (i*16, j*16 + tileTop) + vector2` (`:9493-9494`).
    let vector3 = [
        tx as f32 * 16.0 + vector2[0] + off_x,
        ty as f32 * 16.0 + tile_top as f32 + vector2[1],
    ];
    // `vector4.Y = |windCycle| * num5 * num7` (`:9495`); `vector5 = vector +
    // (0, vector4.Y)` (`:9502`). The unrotated top-left is `vector5 - origin
    // = vector3 + (0, vector4.Y)`.
    let dy = wind_cycle.abs() * num5 * num7;
    let origin = [vector[0] + off_x - vector3[0], vector[1] - vector3[1]];
    VineSway {
        pos: [vector3[0], vector3[1] + dy],
        rot: wind_cycle * num2 * num7, // `num8` (`:9504`).
        origin,
        glow_592,
        fairy_660: kind == 660 && row == sy - 1,
        jar_698: kind == 698,
    }
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

/// The full per-cell draw of a hanging-object cell — the sprite, the 592
/// glowmask, the 660 fairy-jar pair, the 698 jar lids + item icon, and the
/// shared-seed flame tail (`TileDrawing.cs:9505-9600`). `sw` is the cell's
/// [`vine_sway`]; `tf`/`dd` the resolved frame + draw data; `tint` the RAW
/// `Lighting.GetColor` (no actuation ghost); `shared_seed` the object's
/// latched flame seed (the top-left cell's `frame_seed_for`, the C#'s
/// x-outer/y-inner first-cell latch `:9590-9594`). Returns the quad count.
#[allow(clippy::too_many_arguments)]
pub(crate) fn draw_vine_cell(
    r: &mut tr_render::SpriteRenderer,
    lookups: &mut crate::assets::Lookups,
    atlas: Option<&tr_render::Atlas>,
    world: &World,
    tile: &tr_core::Tile,
    dd: &tr_core::drawdata::TileDrawData,
    tf: crate::assets::FrameTex,
    sw: &VineSway,
    tlx: i32,
    tly: i32,
    tx: i32,
    ty: i32,
    tint: [f32; 3],
    anim_time: f32,
    params: &tr_core::lighting::LightParams,
    shared_seed: u64,
) -> u32 {
    use crate::render::player_layers::flip_uv;
    let mut quads = 0u32;
    let kind = tile.kind;
    let (dw, dh) = (dd.width as f32, dd.height as f32);
    let uv = flip_uv(tf.uv, dd.flip);
    // `vector5 = vector + (0, vector4.Y)` — the pivot lands here.
    let vector5 = [sw.pos[0] + sw.origin[0], sw.pos[1] + sw.origin[1]];
    let cell_rect_uv = uv;
    // The 660 fairy-jar bottom row (`:9505-9519`): an `Extra_260` white copy
    // BEFORE the tile draw, the tile draw itself, then the HSL glow (A=127).
    if sw.fairy_660 {
        if let Some(extra) = r.sheets.get("Extra_260").copied() {
            r.push_sprite_origin_rot(
                sw.pos, [dw, dh], extra, [1.0, 1.0, 1.0, 1.0], sw.rot, sw.origin,
            );
            quads += 1;
        }
    }
    // The tile draw (`:9512`) — every cell, the 660 bottom row included.
    r.push_sprite_origin_rot_paint(
        sw.pos, [dw, dh], cell_rect_uv, [tint[0], tint[1], tint[2], 1.0],
        sw.rot, sw.origin, tile.paint(),
    );
    quads += 1;
    if sw.fairy_660 {
        if let Some(extra) = r.sheets.get("Extra_260").copied() {
            // `hue = ((i+j)%200 * 0.11 + timeForVisualEffects/360) % 1`
            // (`:9516`); `hslToRgb(hue, 1, 0.8)`, A=127 (`:9517`).
            let hue = (((tx + ty) % 200) as f32 * 0.11 + params.time_for_visuals / 360.0) % 1.0;
            let hsl = tr_core::tile_light::hsl_to_rgb(hue, 1.0, 0.8);
            r.push_sprite_origin_rot(
                sw.pos, [dw, dh], extra, [hsl[0], hsl[1], hsl[2], 127.0 / 255.0],
                sw.rot, sw.origin,
            );
            quads += 1;
        }
    }
    // The 698 dead-cells jar (`:9520-9584`): the two lid rows + the item icon.
    if sw.jar_698 {
        quads += draw_dead_cells_jar(
            r, lookups, atlas, world, tile, dd, tf, sw, tlx, tly, tx, ty, tint,
            vector5, params,
        );
    }
    // The 592 glowmask (`:9585-9588`): `Glow_294`, white-alpha-0, same rect.
    if sw.glow_592 {
        let fx = dd.frame_x_override.unwrap_or(tile.frame_x) + dd.add_fr_x;
        let fy = dd.frame_y_override.unwrap_or(tile.frame_y) + dd.add_fr_y;
        if let Some(glow) = lookups.glow_uv("Glow_294", fx as i32, fy as i32, dw as u32, dh as u32) {
            r.push_sprite_origin_rot(
                sw.pos, [dw, dh], glow, [1.0, 1.0, 1.0, 0.0], sw.rot, sw.origin,
            );
            quads += 1;
        }
    }
    // The shared-seed flame tail (`:9589-9600`): `GetTileFlameData` per cell,
    // the object's first non-zero seed latched and reused. The flame rect is
    // the cell rect (`tileFrameX + addFrX, tileFrameY + addFrY`).
    let fd = tr_core::flame::tile_flame_data(
        tx, ty, kind, tile.frame_x, tile.frame_y,
        tr_core::flame::tile_frame_seed_at(anim_time), params.global_time, params.disco,
    );
    if fd.count > 0 {
        let mut seed = if shared_seed != 0 { shared_seed } else { fd.seed };
        let fx = (dd.frame_x_override.unwrap_or(tile.frame_x) + dd.add_fr_x) as i32;
        let fy = (dd.frame_y_override.unwrap_or(tile.frame_y) + dd.add_fr_y) as i32;
        // The flame texture: the three jar types use their own sheets
        // (`GetTileFlameData :1849-1874`), the rest `Flames[fd.sheet]`.
        let flame_uv = |sx: i32, sy: i32| -> Option<[f32; 4]> {
            match kind {
                270 => lookups.glow_uv("FireflyJar", sx, sy, dw as u32, dh as u32),
                271 => lookups.glow_uv("LightningBugJar", sx, sy, dw as u32, dh as u32),
                581 => lookups.glow_uv("Glow_291", sx, sy, dw as u32, dh as u32),
                _ => lookups.flame_uv(fd.sheet, sx, sy, dw as u32, dh as u32),
            }
        };
        for _ in 0..fd.count {
            let dx = tr_core::flame::random_int(&mut seed, fd.rx.0, fd.rx.1) as f32 * fd.mx;
            let dy = tr_core::flame::random_int(&mut seed, fd.ry.0, fd.ry.1) as f32 * fd.my;
            if let Some(fuv) = flame_uv(fx, fy) {
                r.push_sprite_origin_rot(
                    [sw.pos[0] + dx, sw.pos[1] + dy], [dw, dh], fuv, fd.color,
                    sw.rot, sw.origin,
                );
                quads += 1;
            }
        }
    }
    quads
}

/// The 698 jar's lid rows + item icon (`TileDrawing.cs:9520-9584`). The TE
/// lookup is at the ANCHOR (`:9520`); the jar texture draws three stacked
/// rects (base, +46, +92), the bottom two carrying rarity-derived colours.
/// The item icon's centre is the rotated `(-2, 24+num11)` offset. The
/// `ItemSlot.DrawItemIcon` colour chain (`GetItemLight`/`GetAlpha`/glowMask)
/// is the d57 furniture lane — the icon draws with the environment tint.
#[allow(clippy::too_many_arguments)]
fn draw_dead_cells_jar(
    r: &mut tr_render::SpriteRenderer,
    lookups: &mut crate::assets::Lookups,
    atlas: Option<&tr_render::Atlas>,
    world: &World,
    tile: &tr_core::Tile,
    dd: &tr_core::drawdata::TileDrawData,
    tf: crate::assets::FrameTex,
    sw: &VineSway,
    tlx: i32,
    tly: i32,
    tx: i32,
    ty: i32,
    tint: [f32; 3],
    vector5: [f32; 2],
    params: &tr_core::lighting::LightParams,
) -> u32 {
    use crate::render::player_layers::flip_uv;
    let mut quads = 0u32;
    let (dw, dh) = (dd.width as f32, dd.height as f32);
    // The TE item at the ANCHOR (`:9520`). The TeItem payload carries no
    // rare/expert — the registry's `rare` feeds the colour (the d57 runtime
    // deferral keeps expert false); an empty jar keeps the default
    // `(150,150,255)` lid (`num12 = 1`).
    let item = world
        .tile_entities
        .get_at(tlx as i16, tly as i16)
        .and_then(|e| match &e.data {
            tr_core::tile_entities::TeData::Item(Some(it)) if !it.is_air() => Some(*it),
            _ => None,
        });
    // `num9 = tileFrameX / 38` (`:9523`) — the dd-remapped frameX column.
    let remapped_fx = dd.frame_x_override.unwrap_or(tile.frame_x);
    let num9 = remapped_fx as i32 / 38;
    let (num10, num11) = match num9 {
        1 => (18.0f32, 0i32),
        2 => (20.0, 0),
        _ => (22.0, -1),
    }; // `:9524-9538`.
    let (rare, expert) = match item {
        Some(it) => (crate::registry::rare(it.kind), false),
        None => (1, false),
    };
    let color3 = jar_rarity_color(rare, expert, params.disco, 1.0);
    let hsl = crate::ui::color::rgb_to_hsl([color3[0], color3[1], color3[2], 255]);
    let mut color4 = color3;
    // `color3 *= 0.25f` (`:9562`) — the XNA byte scale (truncation).
    let c3 = [
        (color3[0] as f32 * 0.25) as u8,
        (color3[1] as f32 * 0.25) as u8,
        (color3[2] as f32 * 0.25) as u8,
    ];
    // `if (num12 != -1) color4 = hslToRgb(h, clamp(s+0.5), clamp(l), 127)`
    // (`:9563-9566`).
    if rare != -1 {
        let s = (hsl.1 as f32 + 0.5).clamp(0.0, 1.0);
        let l = (hsl.2 as f32).clamp(0.0, 1.0);
        let rgb = tr_core::tile_light::hsl_to_rgb(hsl.0 as f32, s, l);
        color4 = [
            (rgb[0] * 255.0) as u8,
            (rgb[1] * 255.0) as u8,
            (rgb[2] * 255.0) as u8,
        ];
    }
    // The +92 row (`value3`, `:9542`, `:9567`): tinted `color3 * 0.25`. The
    // lid rects sit BELOW the 44 px base frame on the jar sheet, so they
    // resolve via `rect_uv` at `frameY + 92` / `+ 46` (the sunflower-glow
    // precedent, `tiles.rs:957-965`), not a sub-rect of the base frame.
    let base_fx = dd.frame_x_override.unwrap_or(tile.frame_x) + dd.add_fr_x;
    let base_fy = dd.frame_y_override.unwrap_or(tile.frame_y) + dd.add_fr_y;
    let lid_uv = |dy: i16| -> Option<[f32; 4]> {
        atlas.and_then(|a| {
            lookups
                .rect_uv(a, 698, base_fx, base_fy + dy, dd.width as u32, dd.height as u32)
                .map(|f| flip_uv(f.uv, dd.flip))
        })
    };
    if let Some(bottom_uv) = lid_uv(92) {
        r.push_sprite_origin_rot(
            sw.pos, [dw, dh], bottom_uv,
            [c3[0] as f32 / 255.0, c3[1] as f32 / 255.0, c3[2] as f32 / 255.0, 1.0],
            sw.rot, sw.origin,
        );
        quads += 1;
    }
    // The item icon (`:9568-9581`).
    if let Some(it) = item {
        let mut spin_y = 24.0 + num11 as f32;
        match num9 {
            1 => spin_y += 4.0,
            2 => spin_y += 6.0,
            _ => {}
        }
        let (s, c) = sw.rot.sin_cos();
        let spin = [-2.0, spin_y];
        let rotated = [spin[0] * c - spin[1] * s, spin[0] * s + spin[1] * c];
        let center = [vector5[0] + rotated[0] + 2.0, vector5[1] + rotated[1]];
        let env = [
            tint[0] + (1.0 - tint[0]) * 0.5,
            tint[1] + (1.0 - tint[1]) * 0.5,
            tint[2] + (1.0 - tint[2]) * 0.5,
        ];
        if let Some((iuv, iw, ih)) = lookups.item_icon(it.kind) {
            // `DrawItem_GetColorAndScale` (`ItemSlot.cs:2966-2975`): the
            // longest side clamped to `sizeLimit` before the item scale.
            let scale = crate::registry::get(it.kind).map_or(1.0, |d| d.scale);
            let longest = iw.max(ih);
            let limit = if longest > num10 { num10 / longest } else { 1.0 };
            let s = scale * limit;
            let (w, h) = (iw * s, ih * s);
            r.push_ui_sprite(
                [center[0] - w * 0.5, center[1] - h * 0.5], [w, h], iuv,
                [env[0], env[1], env[2], 1.0],
            );
            quads += 1;
        }
    }
    // The +46 row (`rectangle2`, `:9540`, `:9583`): tinted `color4`, A=255.
    if let Some(mid_uv) = lid_uv(46) {
        r.push_sprite_origin_rot(
            sw.pos, [dw, dh], mid_uv,
            [color4[0] as f32 / 255.0, color4[1] as f32 / 255.0, color4[2] as f32 / 255.0, 1.0],
            sw.rot, sw.origin,
        );
        quads += 1;
    }
    let _ = (tf, tx, ty);
    quads
}

#[cfg(test)]
mod tests {
    use super::*;
    use tr_core::tile::FLAG_ACTIVE;

    fn empty_world() -> World {
        let mut w = World::new("t".into(), 40, 40);
        w.compute_column_surface();
        w.compute_chunks();
        w
    }

    fn windy() -> Wind {
        let mut wind = Wind::new(7);
        wind.speed_current = 0.5;
        wind.world_surface = f64::INFINITY;
        wind
    }

    /// A hanging object with its top-left at `(tlx, tly)`, `sx`×`sy` cells.
    fn vine_world(kind: u16, tlx: i32, tly: i32, sx: i32, sy: i32) -> World {
        let mut w = empty_world();
        for row in 0..sy {
            for col in 0..sx {
                let i = w.index(tlx + col, tly + row);
                w.tiles[i].kind = kind;
                w.tiles[i].flags = FLAG_ACTIVE;
                w.tiles[i].frame_x = (18 * col) as i16;
                w.tiles[i].frame_y = (18 * row) as i16;
            }
        }
        w.compute_chunks();
        w
    }

    // W4-g84 witness V1 — the size table (`TileDrawing.cs:8845-8884`) and the
    // registration roster (`:549-599`): every MultiTileVine kind is covered,
    // the 549 seaweed (ReverseVine) and the VineThreads are NOT.
    #[test]
    fn w4_g84_vine_size_table_and_roster() {
        assert_eq!(multi_tile_vine_size(34), (3, 3));
        assert_eq!(multi_tile_vine_size(454), (4, 3));
        assert_eq!(multi_tile_vine_size(42), (1, 2));
        assert_eq!(multi_tile_vine_size(270), (1, 2));
        assert_eq!(multi_tile_vine_size(660), (1, 2));
        assert_eq!(multi_tile_vine_size(91), (1, 3));
        assert_eq!(multi_tile_vine_size(95), (2, 2));
        assert_eq!(multi_tile_vine_size(126), (2, 2));
        assert_eq!(multi_tile_vine_size(444), (2, 2));
        assert_eq!(multi_tile_vine_size(465), (2, 3));
        assert_eq!(multi_tile_vine_size(591), (2, 3));
        assert_eq!(multi_tile_vine_size(592), (2, 3));
        assert_eq!(multi_tile_vine_size(698), (1, 1));
        for k in [34, 95, 126, 444, 454, 42, 270, 271, 572, 581, 660, 91, 465, 591, 592, 698] {
            assert!(is_multi_tile_vine(k), "kind {k} registers MultiTileVine");
        }
        assert!(!is_multi_tile_vine(549)); // ReverseVine, bucket 7
        assert!(!is_multi_tile_vine(52)); // VineThreads strip
        assert!(!is_multi_tile_vine(238)); // MultiTileGrass
    }

    // W4-g84 witness V2 — the top-left reconstruction: every cell of a 3×3
    // chandelier and a 2×3 brazier reconstructs the SAME top-left from its
    // frame grid (`col = frameX/18 % sizeX`, `row = frameY/18 % sizeY`).
    #[test]
    fn w4_g84_vine_top_left_reconstruction() {
        for row in 0..3 {
            for col in 0..3 {
                let (tlx, tly, sx, sy) =
                    vine_top_left(34, 10 + col, 20 + row, (18 * col) as i16, (18 * row) as i16);
                assert_eq!((tlx, tly, sx, sy), (10, 20, 3, 3));
            }
        }
        for row in 0..3 {
            for col in 0..2 {
                let (tlx, tly, sx, sy) =
                    vine_top_left(592, 10 + col, 20 + row, (18 * col) as i16, (18 * row) as i16);
                assert_eq!((tlx, tly, sx, sy), (10, 20, 2, 3));
            }
        }
        // A style-shifted chandelier (frameX 108, frameY 54*37 wrap) still
        // reconstructs via the modulo grid.
        let (tlx, tly, _, _) = vine_top_left(34, 11, 21, 108 + 18, 54 + 18);
        assert_eq!((tlx, tly), (10, 20));
    }

    // W4-g84 witness V3 — the planter / bottle style tables verbatim
    // (`TileDrawing.cs:9269-9349` / `:9354-9417`), including the 42-style
    // `num3 = 0; num3 = null` dead-write (the second write wins → None).
    #[test]
    fn w4_g84_vine_style_tables() {
        // 34/126: style = frameY/54 + frameX/108*37.
        assert_eq!(planter_style(0, 9 * 54), (None, -1.0, true, 0.3));
        assert_eq!(planter_style(0, 11 * 54), (Some(1.0), 0.0, false, 0.5));
        assert_eq!(planter_style(0, 23 * 54), (Some(0.0), 0.0, false, 1.0));
        assert_eq!(planter_style(0, 40 * 54), (None, -2.0, true, 0.5));
        assert_eq!(planter_style(0, 44 * 54), (None, -3.0, false, 1.0));
        assert_eq!(planter_style(0, 67 * 54), (Some(1.0), 0.0, false, 1.0));
        // frameX 108 adds 37 to the style index.
        assert_eq!(planter_style(108, 0), planter_style(0, 37 * 54));
        // 42: style = frameY/36.
        assert_eq!(bottle_style(0), (None, -1.0, false, 1.0));
        assert_eq!(bottle_style(9 * 36), (Some(0.0), 0.0, false, 1.0));
        assert_eq!(bottle_style(39 * 36), (None, -1.0, true, 1.0));
        // The dead-write: 40-43 → None (not Some(0.0)).
        assert_eq!(bottle_style(40 * 36), (None, -1.0, true, 1.0));
        assert_eq!(bottle_style(54 * 36), (Some(0.0), 0.0, false, 1.0));
    }

    // W4-g84 witness V4 — the platform lift (`:9250-9254`): a 1-wide bottle
    // under a non-hammered platform lifts 8 px (the anchor AND vector2); a
    // hammered (sloped) platform does not. The 2-wide 591/592 family needs
    // BOTH columns under platforms.
    #[test]
    fn w4_g84_vine_platform_lift() {
        let mut w = vine_world(270, 10, 20, 1, 2);
        // A platform (19) above the top-left.
        let i = w.index(10, 19);
        w.tiles[i].kind = 19;
        w.tiles[i].flags = FLAG_ACTIVE;
        let wind = windy();
        let lifted = vine_sway(&wind, &w, 10, 20, 10, 20, 1, 2, 0, 0.0);
        let no_platform = vine_sway(&wind, &vine_world(270, 10, 20, 1, 2), 10, 20, 10, 20, 1, 2, 0, 0.0);
        // The lift shifts the anchor up 8 → the pivot Y grows by 8.
        assert_eq!(lifted.origin[1] - no_platform.origin[1], 8.0);
        // A sloped platform does not lift.
        let i = w.index(10, 19);
        w.tiles[i].set_slope(1);
        let sloped = vine_sway(&wind, &w, 10, 20, 10, 20, 1, 2, 0, 0.0);
        assert_eq!(sloped.origin[1], no_platform.origin[1]);
    }

    // W4-g84 witness V5 — the 698 wind-cancel + 10 px left shift
    // (`:9440-9445`, `:9453-9456`): flag3 forces `windCycle -= num`, leaving
    // only the grid push (zero with no wind-grid activity), and the whole
    // jar draws 10 px left. The 592 glow flag is set.
    #[test]
    fn w4_g84_vine_jar_cancel_and_shift() {
        let w = vine_world(698, 10, 20, 1, 1);
        let wind = windy();
        let sw = vine_sway(&wind, &w, 10, 20, 10, 20, 1, 1, 0, 0.0);
        assert!(sw.jar_698);
        // No wind-grid push → windCycle 0 → no sway.
        assert_eq!(sw.rot, 0.0);
        // The 10 px left shift lands on the draw position.
        let plain = vine_sway(&wind, &vine_world(270, 10, 20, 1, 2), 10, 20, 10, 20, 1, 2, 0, 0.0);
        assert_eq!(sw.pos[0] - plain.pos[0], -10.0);
        let brazier = vine_sway(&wind, &vine_world(592, 10, 20, 2, 3), 10, 20, 10, 20, 2, 3, 0, 0.0);
        assert!(brazier.glow_592);
    }

    // W4-g84 witness V6 — the per-row num7 ladder + the flag2 top-row zero:
    // a 3-row banner (91) has num7 1/3, 2/3, 1 (bottom); the rot scales with
    // num7. The 660 fairy flag fires only on the bottom row.
    #[test]
    fn w4_g84_vine_num7_ladder_and_fairy_row() {
        let w = vine_world(91, 10, 20, 1, 3);
        let wind = windy();
        let top = vine_sway(&wind, &w, 10, 20, 10, 20, 1, 3, 0, 0.0);
        let mid = vine_sway(&wind, &w, 10, 21, 10, 20, 1, 3, 0, 0.0);
        let bot = vine_sway(&wind, &w, 10, 22, 10, 20, 1, 3, 0, 0.0);
        // num7 = 1/3, 2/3, 1 → rot ratio 1:2:3 (ulp tolerance).
        assert!((mid.rot - top.rot * 2.0).abs() < 1e-5);
        assert!((bot.rot - top.rot * 3.0).abs() < 1e-5);
        assert!(!top.fairy_660 && !bot.fairy_660); // kind 91, not 660
        let fj = vine_world(660, 10, 20, 1, 2);
        let fj_top = vine_sway(&wind, &fj, 10, 20, 10, 20, 1, 2, 0, 0.0);
        let fj_bot = vine_sway(&wind, &fj, 10, 21, 10, 20, 1, 2, 0, 0.0);
        assert!(!fj_top.fairy_660);
        assert!(fj_bot.fairy_660);
    }
}
