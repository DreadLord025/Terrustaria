//! TILE-DRAW-2 glow bind (W4-f15): `DrawSingleTile`'s `glowTexture` draw
//! (`TileDrawing.cs:1343-1403`) — the SECOND `GetTileDrawData` switch's
//! glow record (`drawdata_cases_a::GlowData`) bound to its GlowMask sheet
//! and drawn right after the cell sprite + `DrawSingleTile_Flames`
//! (the flames hook runs inside `DrawBasicTile`, `:1723`; the glow bind
//! follows it at `:1343`).
//!
//! Order and maths, verbatim:
//! * 412 LunarCraftingStation cross-fade (`:1345-1357`): two draws, rows
//!   `tileFrame[412]/60` and `(row+1)%4` at `+row*54` rect Y, colours
//!   `glowColor*(1-t)` / `glowColor*t` with `t = (tileFrame%60)/60f` —
//!   XNA's per-byte truncating float scale (both truncation steps ride
//!   `special_tiles::xna_scale_color`).
//! * Everything else (`:1358-1402`): ONE draw at
//!   `(tileX*16 - (tileWidth-16)/2, tileY*16 + tileTop)` — note this is
//!   NOT the sprite's `vector`: no halfBrickHeight, no draw_dx/dy, and no
//!   `DoNotAdjustDrawPositionBasedOnTileWidth` exemption (711's glow
//!   centres even though its sprite does not).
//! * Platforms (`TileID.Sets.Platforms`) use the sprite anchor `vector`
//!   instead (`:1361-1364`), and a sloped platform adds a second glow
//!   side-fill one cell below (`:1367-1400`) with the SAME neighbour
//!   gates and X picks as `DrawBasicTile`'s side-fill (`:1524`/`:1534` —
//!   shared decision: `special_tiles::platform_side_fill`), keeping the
//!   glow rect's Y/size (`:1375-1383`/`:1389-1397`). The neighbour
//!   `invisibleBlock()` reads ride `FillNeighbor::from_tile`'s documented
//!   `false` stand-in (the coating bit is not on the port's `Tile` yet —
//!   ANALYSIS "S1-G-1e special tiles"). The gate's trailing `tile`
//!   clauses read the BELOW cell — the C# `Tile tile = Main.tile[tileX,
//!   tileY + 1]` local shadows the drawn cell (`:1369`, as `:1519`).
//!
//! Sheet binding: `mask > 0` → `Images/Glow_{mask}`
//! (`AssetInitializer.cs:352-354`); `mask == 0` is the tile-sheet sentinel
//! (637's `GetTileDrawTexture` — raw sheet, painted-sheet deviation — and
//! 711's `TextureAssets.Tile[711]`, `:4937`).
//!
//! Deviations (docs/ANALYSIS.md "TILE-DRAW-2 glow bind"): the
//! `Layer_Tiles`/`Layer_BehindTiles` targets (`:1365`/`:1384`/`:1398`) are
//! unported (S1-G-13 family) — both quads ride the row-major world stream
//! at the C# dispatch point. Alpha-zero tints keep alpha 0: the sprite
//! pipeline implements XNA's premultiplied AlphaBlend (`sprite.wgsl`
//! rgb-factor-One), so no flame-style alpha lift is needed.

use crate::assets;
use crate::drawdata_cases_a::GlowData;
use crate::render::entities::flip_uv;
use crate::render::special_tiles::{platform_side_fill, xna_scale_color, FillNeighbor};
use tr_core::drawdata::TileDrawData;
use tr_core::tiledata::SET_PLATFORMS;
use tr_core::{World, TILE_PX};
use tr_render::{Atlas, SpriteRenderer};

/// One neighbour snapshot for the platform side-fill gates
/// (`Main.tile[x, y+1]` etc., `TileDrawing.cs:1369-1371`); an out-of-world
/// cell reads as the empty default tile.
fn neighbor(world: &World, x: i32, y: i32) -> FillNeighbor {
    world
        .tile(x, y)
        .map(FillNeighbor::from_tile)
        .unwrap_or_default()
}

/// The 412 cross-fade rows/weights (`TileDrawing.cs:1347-1349`):
/// `num20 = tileFrame/60`, `num21 = (num20+1)%4`,
/// `num22 = (tileFrame%60)/60f` — draws `(num20, 1-num22)`, `(num21, num22)`.
pub(crate) fn lunar_crossfade(tile_frame: i32) -> [(i32, f32); 2] {
    let num20 = tile_frame / 60;
    let num21 = (num20 + 1) % 4;
    let num22 = (tile_frame % 60) as f32 / 60.0;
    [(num20, 1.0 - num22), (num21, num22)]
}

/// The glow draw anchor (`TileDrawing.cs:1360-1364`): the plain centred
/// `(tileX*16 - (tileWidth-16)/2, tileY*16 + tileTop)` — NOT the sprite's
/// `vector` (no halfBrickHeight / draw_dx/dy / no_width_adjust exemption) —
/// except platforms, which reuse the sprite anchor (`:1363`).
pub(crate) fn glow_anchor(
    kind: u16,
    tx: i32,
    ty: i32,
    dd: &TileDrawData,
    sprite_anchor: [f32; 2],
) -> [f32; 2] {
    if SET_PLATFORMS.has(kind) {
        sprite_anchor
    } else {
        [
            tx as f32 * TILE_PX - (dd.width as f32 - 16.0) * 0.5,
            ty as f32 * TILE_PX + dd.top as f32,
        ]
    }
}

/// The glow-bind draw (`TileDrawing.cs:1343-1403`). `sprite_anchor` is the
/// caller's `vector` (`:1013` — `pos + draw_dx/dy`, `+ dd.top +
/// halfBrickHeight`), used ONLY by the platform arm (`:1363`); every other
/// kind re-derives the plain centred anchor (`:1360`). `tile_frame_412` is
/// `Main.tileFrame[412]` (`:1347-1349`). Returns the quad count pushed.
#[allow(clippy::too_many_arguments)]
pub(crate) fn draw_tile_glow(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    atlas: Option<&Atlas>,
    world: &World,
    kind: u16,
    tx: i32,
    ty: i32,
    dd: &TileDrawData,
    glow: &GlowData,
    sprite_anchor: [f32; 2],
    tile_frame_412: i32,
    show_invisible_blocks: bool,
) -> u32 {
    let [sx, sy, sw, sh] = glow.src;
    if sw <= 0 || sh <= 0 {
        return 0;
    }
    // The sheet pick: GlowMask strip or the tile's own sheet (mask 0).
    let uv = if glow.mask == 0 {
        let Some(a) = atlas else { return 0 };
        lookups
            .rect_uv(a, kind, sx, sy, sw as u32, sh as u32)
            .map(|f| f.uv)
    } else {
        let named = format!("Glow_{}", glow.mask);
        lookups.glow_uv(&named, sx as i32, sy as i32, sw as u32, sh as u32)
    };
    let Some(uv) = uv else { return 0 };
    let uv = flip_uv(uv, dd.flip);
    let size = [sw as f32, sh as f32];
    let tint = [
        glow.color[0] as f32 / 255.0,
        glow.color[1] as f32 / 255.0,
        glow.color[2] as f32 / 255.0,
        glow.color[3] as f32 / 255.0,
    ];
    let mut quads = 0;
    if kind == 412 {
        // `:1345-1357` — the two-row cross-fade.
        let pos = glow_anchor(kind, tx, ty, dd, sprite_anchor);
        for (row, scale) in lunar_crossfade(tile_frame_412) {
            let Some(row_uv) = lookups.glow_uv(
                &format!("Glow_{}", glow.mask),
                sx as i32,
                sy as i32 + row * 18 * 3,
                sw as u32,
                sh as u32,
            ) else {
                continue;
            };
            let c = xna_scale_color(glow.color, scale);
            r.push_sprite(
                pos,
                size,
                flip_uv(row_uv, dd.flip),
                [
                    c[0] as f32 / 255.0,
                    c[1] as f32 / 255.0,
                    c[2] as f32 / 255.0,
                    c[3] as f32 / 255.0,
                ],
            );
            quads += 1;
        }
        return quads;
    }
    // `:1360-1364` — the plain centred anchor, or `vector` on platforms.
    let platform = SET_PLATFORMS.has(kind);
    let pos = glow_anchor(kind, tx, ty, dd, sprite_anchor);
    // `SetLayer(Layer_Tiles, 1)` (`:1365`) — layer targets unported.
    r.push_sprite(pos, size, uv, tint);
    quads += 1;
    // `:1367-1400` — the sloped-platform glow side-fill, one cell below.
    let self_slope = world.tile(tx, ty).map(|t| t.slope()).unwrap_or(0);
    if platform && self_slope != 0 {
        if let Some(fill) = platform_side_fill(
            self_slope,
            0, // the Y is overridden below — the glow keeps its own rect Y
            &neighbor(world, tx, ty + 1),
            &neighbor(world, tx - 1, ty + 1),
            &neighbor(world, tx + 1, ty + 1),
            show_invisible_blocks,
        ) {
            // `:1375-1383`/`:1389-1397`: the fill's X pick on the GLOW
            // rect, keeping its Y/size.
            let fill_uv = if glow.mask == 0 {
                let Some(a) = atlas else { return quads };
                lookups
                    .rect_uv(a, kind, fill.src[0] as i16, sy, sw as u32, sh as u32)
                    .map(|f| f.uv)
            } else {
                lookups.glow_uv(
                    &format!("Glow_{}", glow.mask),
                    fill.src[0],
                    sy as i32,
                    sw as u32,
                    sh as u32,
                )
            };
            if let Some(fill_uv) = fill_uv {
                // `SetLayer(Layer_BehindTiles, 1)` (`:1384`/`:1398`).
                r.push_sprite(
                    [pos[0], pos[1] + 16.0],
                    size,
                    flip_uv(fill_uv, dd.flip),
                    tint,
                );
                quads += 1;
            }
        }
    }
    quads
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The 412 cross-fade (`TileDrawing.cs:1347-1349`): frame 0 → rows
    /// (0, 1.0) / (1, 0.0); frame 90 → row 1 at 0.5 both; frame 239 →
    /// rows 3 / 0 (the `%4` wrap) at 1/60 and 59/60.
    #[test]
    fn td2_lunar_crossfade_rows_and_weights() {
        assert_eq!(lunar_crossfade(0), [(0, 1.0), (1, 0.0)]);
        assert_eq!(lunar_crossfade(90), [(1, 0.5), (2, 0.5)]);
        let f = lunar_crossfade(239);
        assert_eq!((f[0].0, f[1].0), (3, 0));
        assert!((f[0].1 - 1.0 / 60.0).abs() < 1e-6);
        assert!((f[1].1 - 59.0 / 60.0).abs() < 1e-6);
    }

    /// The glow anchor is NOT the sprite vector (`:1360`): no
    /// halfBrickHeight, no draw_dx/dy, no no_width_adjust exemption —
    /// 711's glow centres while its sprite hangs left. Platforms alone
    /// reuse the sprite anchor (`:1363`).
    #[test]
    fn td2_glow_anchor_differs_from_the_sprite_vector() {
        let mut dd = TileDrawData {
            width: 18,
            height: 20,
            top: 2,
            ..Default::default()
        };
        dd.no_width_adjust = true; // 711
        let sprite = [10.0 * TILE_PX + 3.0, 20.0 * TILE_PX + 2.0 + 8.0];
        assert_eq!(
            glow_anchor(711, 10, 20, &dd, sprite),
            [10.0 * TILE_PX - 1.0, 20.0 * TILE_PX + 2.0],
        );
        // Platform 19: exactly the sprite anchor.
        assert_eq!(glow_anchor(19, 10, 20, &dd, sprite), sprite);
    }

    /// The side-fill decision is SHARED with DrawBasicTile's arm
    /// (`special_tiles::platform_side_fill`): a slope-1 platform over a
    /// full stone block right-below picks X 198; an unsloped platform
    /// neighbour picks 324 (`:1376-1383`). The glow keeps its own Y/size.
    #[test]
    fn td2_platform_glow_side_fill_gate_and_x_picks() {
        let mut w = World::new("t".into(), 40, 40);
        let put = |w: &mut World, x: i32, y: i32, kind: u16, slope: u8| {
            let i = w.index(x, y);
            w.tiles[i].kind = kind;
            w.tiles[i].flags = tr_core::tile::FLAG_ACTIVE;
            w.tiles[i].set_slope(slope);
        };
        put(&mut w, 10, 10, 19, 1);
        put(&mut w, 11, 11, 1, 0); // stone below-right
        let fill = platform_side_fill(
            1,
            0,
            &neighbor(&w, 10, 11),
            &neighbor(&w, 9, 11),
            &neighbor(&w, 11, 11),
            false,
        )
        .expect("slope-1 over stone fills");
        assert_eq!(fill.src[0], 198);
        put(&mut w, 11, 11, 19, 0); // unsloped platform neighbour
        let fill = platform_side_fill(
            1,
            0,
            &neighbor(&w, 10, 11),
            &neighbor(&w, 9, 11),
            &neighbor(&w, 11, 11),
            false,
        )
        .expect("platform neighbour fills");
        assert_eq!(fill.src[0], 324);
    }
}
