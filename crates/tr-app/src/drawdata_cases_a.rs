//! S1-F-9 wave D slice A — the remaining STATIC `GetTileDrawData` case ids.
//!
//! Two disjoint halves of `Terraria.GameContent.Drawing.TileDrawing`:
//!
//! 1. Three world-probing arms of the main type switch with no animator and
//!    no RNG: `case 518` LilyPad (`TileDrawing.cs:4604-4630`, the `tileTop`
//!    float rides the cell's liquid level and the half-brick/slope block
//!    below), `case 530` OasisPlants (`:4705-4722`) and `case 529` SeaOats
//!    (`:4827-4849`) — the two `WorldGen.GetBiomeInfluence` row picks.
//! 2. The SECOND switch of `GetTileDrawData` (`TileDrawing.cs:6049-6507`):
//!    the per-type glow-mask selection. Beyond picking the glow sheet it
//!    also carries unconditional source-frame maths the first switch does
//!    not have — the 1998/2016/2040/2014 style-stride unwinds (dressers 88,
//!    pianos 87, bookcases 101, hanging lanterns 42, grandfather clocks 104,
//!    lamps 93, sinks 172, chairs 15/497, thrones/benches 89) and two
//!    unconditional `tileTop` rewrites (lamps 93 `+= 2`, clocks 104 `= 2`).
//!    This module ports every case of that switch: 38 case ids
//!    (412, 657, 656, 701, 634, 637, 638, 568, 569, 570, 580, 564, 184,
//!    463, 19, 90, 79, 89, 100, 33, 15, 497, 34, 21, 467, 441, 468, 10,
//!    11, 88, 42, 87, 14, 93, 18, 101, 104, 172).
//! 3. W4-d62: `case 711` RainbowBoulder's FIRST-switch glow
//!    (`TileDrawing.cs:4937-4938`) — the only first-switch glow in the d62
//!    census range; it has no second-switch case, so the record folds into
//!    [`glow_switch`] with the [`TILE_SHEET_GLOW`] mask sentinel (the glow
//!    texture is the boulder's own tile sheet).
//!
//! Seam: like the wave-B animator and wave-C furniture modules, this rides
//! as a delegated pass AFTER `tr_core::drawdata::draw_data` +
//! `apply_world_corrections` (the C# glow switch runs after the main switch
//! and the campfire/half-brick epilogue, `TileDrawing.cs:6033-6049`); the
//! renderer calls [`apply_cases_a`] at the `draw_data` dispatch site
//! (`render/tiles.rs`). `drawdata.rs` itself is untouched — waves B/C
//! own it.
//!
//! C# reads its glow gates from the RAW `tileFrameX/tileFrameY` locals —
//! the glow switch never rewrites them — so the gates here use the tile's
//! stored framing, NOT the style-unwound rect coordinates. A style-block-1
//! dresser (`fx = 1998 + 24*54`) unwinds its frame but does NOT glow: its
//! raw `tileFrameX / 54` is 61, not 24.
//!
//! Rendering grid: all source rects stay on the 16 px / 18 px sheet grid
//! (2 px padding); the style-block strides (1998 = 37 columns of 54,
//! 2016 = 56 rows of 36, 2040 = 51 rows of 40, 2014 = 53 rows of 38) are
//! the C# constants verbatim.
//!
//! Documented deviations (recorded in docs/ANALYSIS.md "S1-F-9d cases A"):
//! - The glow TEXTURE binding is WIRED (TILE-DRAW-2, W4-f15): the renderer's
//!   `render/tile_glow.rs` consumes the record at the C# `DrawSingleTile`
//!   glow-bind site (`TileDrawing.cs:1343-1403`).
//! - `case 637`'s `glowTexture = GetTileDrawTexture(...)` (the per-tile
//!   painted sheet) and `case 184` column 220's `Main.DiscoR/G/B` colour are
//!   engine state the port does not route here; their rects still port.
//! - `case 564`'s unconditional `addFrY = 0` (`:6112`) rides THIS pass's
//!   `add_fr_y_delta` (W4-g34): the first switch keeps `tileFrame * 36` so
//!   the glow rect (`:6109`) reads the pre-clear row, then the delta zeroes
//!   it — the C# order. The old port folded the clear into `drawdata.rs` and
//!   the rect lost the animation row (d63 batch 4 / d64 R15 deviation).
//! - `Main.mouseTextColor` (cases 656/701/564) is the UI text pulse; the
//!   port passes 255 (the steady full-bright value) until the pulse rides in.
//! - `Main.martianLight` is 0 outside the Martian Madness event (not
//!   ported) — [`martian_glow`] takes the value so the event can feed it.
//!   painted sheet) binds the RAW `Tiles_637` sheet (no paint system on
//!   glow — recorded deviation); `case 184` column 220's `Main.DiscoR/G/B`
//!   colour rides the `disco` parameter (`LightParams.disco`).
//! - `case 564`'s unconditional `addFrY = 0` (`:6112`) is ALREADY applied by
//!   `drawdata.rs`'s 564 arm — this pass must not apply it twice. The glow
//!   RECT reads the PRE-clear row (`tileFrame * 36`, C# :6109 reads addFrY
//!   before :6112 clears it) via the `tile_frame` parameter.
//! - `Main.mouseTextColor` (cases 656/701/564) rides the `mouse_text_color`
//!   parameter (`LightParams.mouse_text_color`).
//! - `Main.martianLight` is the LIVE ±0.015 bounce (W4-g12 doc-sweep:
//!   `DoUpdate_AnimateTileGlows` ticks it unconditionally,
//!   `Main.cs:17145`/`:18251-18261` — the old "0 outside the event"
//!   reading was wrong); [`martian_glow`] takes the live value.

use tr_core::drawdata::TileDrawData;
use tr_core::tile::Tile;
use tr_core::tiledata::{SET_CORRUPT, SET_CRIMSON, SET_HALLOW, TILE_SOLID, TILE_SOLID_TOP};
use tr_core::world::World;

/// `GetTileDrawData`'s glow out-params (`TileDrawing.cs:4536-4538` starts
/// them null/empty/transparent): the glow-mask sheet index into
/// `TextureAssets.GlowMask`, the source rectangle on that sheet and the
/// draw colour as XNA bytes (alpha included — the glow colours use alpha 0
/// deliberately, XNA's premultiplied blend adds them in full).
/// `mask == 0` is the TILE-SHEET sentinel: the glow texture is the tile's
/// own sheet, not a GlowMask strip — case 637's `GetTileDrawTexture`
/// (`:6076`, painted-sheet deviation: the port binds the raw sheet) and
/// case 711's `TextureAssets.Tile[711]` (`:4937`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GlowData {
    pub mask: u16,
    /// `(tileFrameX [+ addFrX], tileFrameY [+ addFrY], tileWidth, tileHeight)`
    /// exactly as the C# builds it — some cases fold `% stride`, some add
    /// `addFrX/Y`, case 638 subtracts `halfBrickHeight` from the height.
    pub src: [i16; 4],
    pub color: [u8; 4],
}

/// The wave-D slice-A output: the optional glow record plus the frame/top
/// deltas the second switch applies on top of the first switch's result.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DrawDataCasesA {
    pub glow: Option<GlowData>,
    /// Added to `add_fr_x` / `add_fr_y` (the stride unwinds, C# `-=`/`+=`).
    pub add_fr_x_delta: i16,
    pub add_fr_y_delta: i16,
    /// Added to `top` (case 93 `tileTop += 2`).
    pub top_delta: i16,
    /// `tileTop = 2` rewrites (case 104) — `None` keeps the first switch's.
    pub top_override: Option<i16>,
}

impl DrawDataCasesA {
    /// Fold the pass into the table result (the renderer applies this once,
    /// after `apply_world_corrections`).
    pub fn apply(&self, data: &mut TileDrawData) {
        data.add_fr_x = data.add_fr_x.wrapping_add(self.add_fr_x_delta);
        data.add_fr_y = data.add_fr_y.wrapping_add(self.add_fr_y_delta);
        data.top = data.top.wrapping_add(self.top_delta);
        if let Some(t) = self.top_override {
            data.top = t;
        }
    }
}

/// `TileDrawing._martianGlow` (`TileDrawing.cs:498-499`):
/// `b = (byte)(100f + 150f * Main.martianLight)`, `Color(b, b, b, 0)`.
/// The byte cast truncates the float product. `martian_light` is the live
/// per-frame bounce (`Main.cs:18251-18261`, W4-g12 doc-sweep).
pub fn martian_glow(martian_light: f32) -> [u8; 4] {
    let b = (100.0 + 150.0 * martian_light) as u8;
    [b, b, b, 0]
}

/// `TileDrawing._meteorGlow` (`TileDrawing.cs:130`): `(100, 100, 100, 0)`.
pub const METEOR_GLOW: [u8; 4] = [100, 100, 100, 0];
/// [`GlowData::mask`] sentinel: the glow texture is the tile's OWN sheet
/// (`TextureAssets.Tile[kind]`), not a `TextureAssets.GlowMask` entry —
/// `case 711` RainbowBoulder's FIRST-switch glow (`TileDrawing.cs:4937`).
/// The glow pass (TILE-DRAW-2) resolves the sheet from the cell's kind.
pub const TILE_SHEET_GLOW: u16 = u16::MAX;
/// `TileDrawing._lavaMossGlow` (`:132`): `(150, 100, 50, 0)`.
pub const LAVA_MOSS_GLOW: [u8; 4] = [150, 100, 50, 0];
/// `TileDrawing._kryptonMossGlow` (`TileDrawing.cs:134`): `(0, 200, 0, 0)`.
pub const KRYPTON_MOSS_GLOW: [u8; 4] = [0, 200, 0, 0];
/// `TileDrawing._xenonMossGlow` (`TileDrawing.cs:136`): `(0, 180, 250, 0)`.
pub const XENON_MOSS_GLOW: [u8; 4] = [0, 180, 250, 0];
/// `TileDrawing._argonMossGlow` (`TileDrawing.cs:138`): `(225, 0, 125, 0)`.
pub const ARGON_MOSS_GLOW: [u8; 4] = [225, 0, 125, 0];
/// `TileDrawing._violetMossGlow` (`TileDrawing.cs:140`): `(150, 0, 250, 0)`.
pub const VIOLET_MOSS_GLOW: [u8; 4] = [150, 0, 250, 0];

/// `WorldGen.SolidTile` (`WorldGen.cs:70111-70132`): active, in
/// `Main.tileSolid`, not in `Main.tileSolidTop`, not half-brick, slope 0 and
/// not actuated-inactive. A null tile (outside the array) is `true`; the
/// port's `World::tile` returns `None` there, matching. The `noDoors`
/// parameter defaults to `false` (case 4's probe, `TileDrawing.cs:4672`,
/// passes nothing), so the door clause is not consulted here.
pub fn solid_tile(world: &World, x: i32, y: i32) -> bool {
    let Some(t) = world.tile(x, y) else {
        return true;
    };
    t.active()
        && TILE_SOLID.has(t.kind)
        && !TILE_SOLID_TOP.has(t.kind)
        && !t.half_brick()
        && t.slope() == 0
        && !t.inactive()
}

/// The `Tile.blockType()` pick of case 518 (`Tile.cs:202-214`): 1 =
/// half-brick, 2/3 = the down-slopes (`slope() + 1`), 0 = flat — the same
/// shape `drawdata.rs`'s `block_type` closure uses for case 726.
fn block_type(t: &Tile) -> i32 {
    if t.half_brick() {
        1
    } else {
        let slope = t.slope() as i32;
        if slope > 0 { slope + 1 } else { 0 }
    }
}

/// `GetTileDrawData` `case 518` — LilyPad (`TileDrawing.cs:4604-4630`):
/// `tileTop -= liquid/16 - 3`, clamped to 8 when the tile above is solid
/// and the level is deeper; a dry pad rides the liquid of the half-brick
/// (`-16 + max(8, liquid/16)`) or slope (`-4`) block below. Returns the
/// `tileTop` delta (the C# `tileTop -= num19`).
pub fn lily_pad_top_delta(world: &World, x: i32, y: i32, tile: &Tile) -> i16 {
    let mut num19 = tile.liquid as i32 / 16 - 3;
    if solid_tile(world, x, y - 1) && num19 > 8 {
        num19 = 8;
    }
    if tile.liquid == 0 {
        // `Framing.GetTileSafely(x, y + 1)` — an out-of-world cell is the
        // empty default tile (inactive), so the switch does not run.
        if let Some(below) = world.tile(x, y + 1) {
            if below.active() {
                match block_type(below) {
                    1 => num19 = -16 + (8).max(below.liquid as i32 / 16),
                    2 | 3 => num19 -= 4,
                    _ => {}
                }
            }
        }
    }
    (-num19) as i16
}

/// `WorldGen.GetBiomeInfluence` (`WorldGen.cs:47750-47777`): counts the
/// corrupt/crimson/hallow tiles in the inclusive rectangle; null cells are
/// skipped. Returns `(corrupt, crimson, hallowed)`.
pub fn biome_influence(world: &World, sx: i32, ex: i32, sy: i32, ey: i32) -> (i32, i32, i32) {
    let (mut corrupt, mut crimson, mut hallow) = (0, 0, 0);
    for i in sx..=ex {
        for j in sy..=ey {
            if let Some(t) = world.tile(i, j) {
                if SET_CORRUPT.has(t.kind) {
                    corrupt += 1;
                }
                if SET_CRIMSON.has(t.kind) {
                    crimson += 1;
                }
                if SET_HALLOW.has(t.kind) {
                    hallow += 1;
                }
            }
        }
    }
    (corrupt, crimson, hallow)
}

/// `GetTileDrawData` `case 530` — OasisPlants (`TileDrawing.cs:4705-4722`):
/// `addFrY += 36 * <biome pick>` over the 4-cell band under the plant's
/// anchor: 0 none, 1 hallow wins, 2 crimson wins, 3 corrupt wins (the C#
/// tie order: hallow first, then crimson, then corrupt). The static
/// `tileTop = 2` (`:4721`) already rides `drawdata.rs`'s 530 arm.
pub fn oasis_plants_add_fr_y(world: &World, x: i32, y: i32, fx: i32, fy: i32) -> i16 {
    let num35 = y - fy % 36 / 18 + 2;
    let num36 = x - fx % 54 / 18;
    let (corrupt, crimson, hallow) = biome_influence(world, num36, num36 + 3, num35, num35);
    let mut num37 = corrupt;
    if num37 < crimson {
        num37 = crimson;
    }
    if num37 < hallow {
        num37 = hallow;
    }
    let pick = if corrupt != 0 || crimson != 0 || hallow != 0 {
        if hallow == num37 {
            1
        } else if crimson != num37 {
            3
        } else {
            2
        }
    } else {
        0
    };
    (36 * pick) as i16
}

/// `GetTileDrawData` `case 529` — SeaOats (`TileDrawing.cs:4827-4849`):
/// `addFrY += 34 * <pick> - tileFrameY`; the pick is 1 on the beach band
/// when no evil/hallow tile sits below, else 2 hallow / 3 crimson /
/// 4 corrupt (same tie order as case 530). `beach_distance` is
/// `WorldGen.beachDistance`; the 32-px height / top −14 / parity flip
/// (`:4842-4847`) already ride `drawdata.rs`'s 529 arm.
pub fn sea_oats_add_fr_y(world: &World, x: i32, y: i32, fy: i32, beach_distance: i32) -> i16 {
    let num28 = y + 1;
    let (corrupt, crimson, hallow) = biome_influence(world, x, x, num28, num28);
    let mut num29 = corrupt;
    if num29 < crimson {
        num29 = crimson;
    }
    if num29 < hallow {
        num29 = hallow;
    }
    let pick = if corrupt == 0 && crimson == 0 && hallow == 0 {
        if x < beach_distance || x > world.width - beach_distance {
            1
        } else {
            0
        }
    } else if hallow == num29 {
        2
    } else if crimson != num29 {
        4
    } else {
        3
    };
    (34 * pick - fy) as i16
}

/// The second `GetTileDrawData` switch (`TileDrawing.cs:6049-6507`) — the
/// glow-mask pick plus its unconditional frame/top maths. Inputs are the
/// tile's stored framing (`fx`/`fy` = `tileCache.frameX/frameY` — the C#
/// gates read these raw locals, never rewritten by this switch), the FIRST
/// switch's final `addFrX/addFrY` (`add_fr_x`/`add_fr_y`, the C# reads the
/// post-switch locals), the final `width`/`height`/`half_brick_height` and
/// the shaded cell colour (`color`, cases 637/638 lerp against it).
/// `mouse_text_color` is `Main.mouseTextColor` (`LightParams`),
/// `martian_light` is `Main.martianLight` (the live bounce, W4-g12) and
/// `disco` is `Main.DiscoR/G/B` (case 184 column 220, `:6149`).
///
/// `case 564`'s `addFrY = 0` (`:6112`) rides the returned `add_fr_y_delta`
/// (W4-g34): the rect reads the pre-clear `add_fr_y` the first switch left
/// (`tileFrame * 36`), then the delta zeroes it — the C# order.
/// `case 564`'s `addFrY = 0` (`:6112`) is intentionally NOT returned — the
/// first switch's port already folds it (`drawdata.rs` case 564). Its glow
/// RECT reads the PRE-clear animation row: `pre_clear_add_fr_y` is
/// `Main.tileFrame[564] * 36` (the shared 217/218 arm value the C# still
/// holds at `:6109` before `:6112` zeroes it).
#[allow(clippy::too_many_arguments)]
pub fn glow_switch(
    kind: u16,
    fx: i32,
    fy: i32,
    add_fr_x: i32,
    add_fr_y: i32,
    width: i32,
    height: i32,
    half_brick_height: i32,
    color: [u8; 4],
    mouse_text_color: u8,
    martian_light: f32,
    disco: [u8; 3],
    pre_clear_add_fr_y: i32,
) -> DrawDataCasesA {
    let mut out = DrawDataCasesA::default();
    let martian = martian_glow(martian_light);
    // The common rect shapes; the C# builds each `new Rectangle(...)`
    // inline, these name the three spellings it uses.
    let rect_plain = |x: i32, y: i32, h: i32| [x as i16, y as i16, width as i16, h as i16];
    let rect_framed =
        |h: i32| [(fx + add_fr_x) as i16, (fy + add_fr_y) as i16, width as i16, h as i16];
    let rect_anim_y = |x: i32, h: i32| [x as i16, (fy + add_fr_y) as i16, width as i16, h as i16];
    // `Color.Lerp(Color.White, color, 0.75f)` — XNA lerps each byte
    // channel with the float amount (truncating on the byte write).
    let lerp_white_75 = |c: [u8; 4]| -> [u8; 4] {
        let l = |a: u8, b: u8| (a as f32 + (b as f32 - a as f32) * 0.75) as u8;
        [l(255, c[0]), l(255, c[1]), l(255, c[2]), l(255, c[3])]
    };
    let pulse = mouse_text_color as f32 / 255.0;
    match kind {
        // `case 412` (:6051-6055): GlowMask 202, framed row, opaque white.
        412 => out.glow = Some(GlowData { mask: 202, src: rect_anim_y(fx, height), color: [255, 255, 255, 255] }),
        // `case 657` (:6056-6063): GlowMask 330 only from frame row 54.
        657 => {
            if fy >= 54 {
                out.glow = Some(GlowData { mask: 330, src: rect_anim_y(fx, height), color: [255, 255, 255, 255] });
            }
        }
        // `case 656 / 701` (:6064-6069): GlowMask 329, white * text pulse.
        656 | 701 => {
            let v = (255.0 * pulse) as u8;
            out.glow = Some(GlowData { mask: 329, src: rect_anim_y(fx, height), color: [v, v, v, 0] });
        }
        // `case 634` (:6070-6074): GlowMask 315 (the ash-tree variant).
        634 => out.glow = Some(GlowData { mask: 315, src: rect_anim_y(fx, height), color: [255, 255, 255, 255] }),
        // `case 637` (:6075-6079): the glow texture is the painted tile
        // sheet (deviation: not bound here), rect adds addFrX, colour
        // lerps 75% toward the cell light.
        637 => out.glow = Some(GlowData { mask: 0, src: rect_framed(height), color: lerp_white_75(color) }),
        // `case 638` (:6080-6084): GlowMask 327, framed, height minus the
        // half-brick cut, same 75% lerp.
        638 => out.glow = Some(GlowData { mask: 327, src: rect_framed(height - half_brick_height), color: lerp_white_75(color) }),
        // `case 568/569/570` (:6085-6099): the three firefly-bottle glows.
        568 => out.glow = Some(GlowData { mask: 268, src: rect_anim_y(fx, height), color: [255, 255, 255, 255] }),
        569 => out.glow = Some(GlowData { mask: 269, src: rect_anim_y(fx, height), color: [255, 255, 255, 255] }),
        570 => out.glow = Some(GlowData { mask: 270, src: rect_anim_y(fx, height), color: [255, 255, 255, 255] }),
        // `case 580` (:6100-6104): butterfly jar, (225, 110, 110, 0).
        580 => out.glow = Some(GlowData { mask: 289, src: rect_anim_y(fx, height), color: [225, 110, 110, 0] }),
        // `case 564` (:6105-6113): GlowMask 267 below frame column 36,
        // (200, 200, 200, 0) * text pulse. The rect reads the PRE-clear
        // `addFrY` (`tileFrame * 36`, the first switch's 217/218/564 arm);
        // the trailing `addFrY = 0` (:6112) rides the delta below, applied
        // AFTER the rect is built — W4-g34 deviation fix (d63 batch 4 /
        // d64 R15): the old port folded the clear into the first switch and
        // the rect lost the animation row.
        // (200, 200, 200, 0) * text pulse; the rect Y reads the PRE-clear
        // addFrY (`tileFrame * 36`) — :6109 runs before :6112's reset, and
        // the reset itself is already applied by drawdata.rs's 564 arm.
        564 => {
            if fx < 36 {
                let v = (200.0 * pulse) as u8;
                let src = [fx as i16, (fy + pre_clear_add_fr_y) as i16, width as i16, height as i16];
                out.glow = Some(GlowData { mask: 267, src, color: [v, v, v, 0] });
            }
            // :6112 — unconditional `addFrY = 0`; the delta zeroes whatever
            // the first switch left (the C# assigns, it does not subtract a
            // fixed row, so this is `-add_fr_y`).
            out.add_fr_y_delta = (-add_fr_y) as i16;
        }
        // `case 184` (:6114-6151): LongMoss columns pick the moss glow,
        // rect uses the RAW frame (no addFr*). Column 220 is the disco
        // moss — `new Color(Main.DiscoR, Main.DiscoG, Main.DiscoB)`
        // (:6149), alpha 255 from the byte ctor.
        184 => {
            let src = rect_plain(fx, fy, height);
            let col = match fx {
                110 => Some(LAVA_MOSS_GLOW),
                132 => Some(KRYPTON_MOSS_GLOW),
                154 => Some(XENON_MOSS_GLOW),
                176 => Some(ARGON_MOSS_GLOW),
                198 => Some(VIOLET_MOSS_GLOW),
                220 => Some([disco[0], disco[1], disco[2], 255]),
                _ => None,
            };
            if let Some(col) = col {
                out.glow = Some(GlowData { mask: 127, src, color: col });
            }
        }
        // `case 463` (:6152-6156): DefendersForge, (127, 127, 127, 0).
        463 => out.glow = Some(GlowData { mask: 243, src: rect_anim_y(fx, height), color: [127, 127, 127, 0] }),
        // `case 19` (:6157-6173): platforms — the martian conduit (row 26)
        // and meteor conduit (row 27) glow rows, `% 18` folds the row.
        19 => {
            let num46 = fy / 18;
            let src = rect_plain(fx, fy % 18, height);
            if num46 == 26 {
                out.glow = Some(GlowData { mask: 65, src, color: martian });
            }
            if num46 == 27 {
                out.glow = Some(GlowData { mask: 112, src, color: METEOR_GLOW });
            }
        }
        // `case 90` (:6174-6190): bathtubs, rows 27/28 of the 36 grid.
        90 => {
            let num46 = fy / 36;
            let src = rect_plain(fx, fy % 36, height);
            if num46 == 27 {
                out.glow = Some(GlowData { mask: 52, src, color: martian });
            }
            if num46 == 28 {
                out.glow = Some(GlowData { mask: 113, src, color: METEOR_GLOW });
            }
        }
        // `case 79` (:6191-6207): beds, rows 27/28 of the 36 grid.
        79 => {
            let num46 = fy / 36;
            let src = rect_plain(fx, fy % 36, height);
            if num46 == 27 {
                out.glow = Some(GlowData { mask: 53, src, color: martian });
            }
            if num46 == 28 {
                out.glow = Some(GlowData { mask: 114, src, color: METEOR_GLOW });
            }
        }
        // `case 89` (:6208-6227): benches/thrones — the 1998 style stride
        // unwinds FIRST, then columns 29/30 glow (`fx % 54` rect; the gate
        // reads the raw `fx / 54`).
        89 => {
            let num46 = fx / 54;
            let num49 = fx / 1998;
            out.add_fr_x_delta = (-1998 * num49) as i16;
            out.add_fr_y_delta = (36 * num49) as i16;
            let src = rect_plain(fx % 54, fy, height);
            if num46 == 29 {
                out.glow = Some(GlowData { mask: 66, src, color: martian });
            }
            if num46 == 30 {
                out.glow = Some(GlowData { mask: 123, src, color: METEOR_GLOW });
            }
        }
        // `case 100` (:6228-6235): candelabras, column 0 row 27 (36 grid).
        100 => {
            if fx / 36 == 0 && fy / 36 == 27 {
                out.glow = Some(GlowData { mask: 68, src: rect_plain(fx, fy % 36, height), color: martian });
            }
        }
        // `case 33` (:6236-6243): candles — column 0, row 26 of the 22 grid
        // (the candle sheet's own 22 px stride, not 18).
        33 => {
            if fx / 18 == 0 && fy / 22 == 26 {
                out.glow = Some(GlowData { mask: 61, src: rect_plain(fx, fy % 22, height), color: martian });
            }
        }
        // `case 15 / 497` (:6244-6267): chairs/toilets — the 2040 stride
        // (51 rows of 40) unwinds; only kind 15 glows, rows 32/33.
        15 | 497 => {
            let num46 = fy / 40;
            let num53 = num46 / 51;
            out.add_fr_y_delta = (-2040 * num53) as i16;
            out.add_fr_x_delta = (36 * num53) as i16;
            if kind == 15 {
                let src = rect_plain(fx, fy % 40, height);
                if num46 == 32 {
                    out.glow = Some(GlowData { mask: 54, src, color: martian });
                }
                if num46 == 33 {
                    out.glow = Some(GlowData { mask: 116, src, color: METEOR_GLOW });
                }
            }
        }
        // `case 34` (:6268-6275): chandeliers, column-block 0 row 33 (54).
        34 => {
            if fx / 54 == 0 && fy / 54 == 33 {
                out.glow = Some(GlowData { mask: 55, src: rect_plain(fx, fy % 54, height), color: martian });
            }
        }
        // `case 21 / 467` (:6276-6293): chests — columns 48/49 (36 grid).
        21 | 467 => {
            let num46 = fx / 36;
            let src = rect_plain(fx % 36, fy, height);
            if num46 == 48 {
                out.glow = Some(GlowData { mask: 56, src, color: martian });
            }
            if num46 == 49 {
                out.glow = Some(GlowData { mask: 117, src, color: METEOR_GLOW });
            }
        }
        // `case 441 / 468` (:6294-6311): fake chests share the chest rows.
        441 | 468 => {
            let num46 = fx / 36;
            let src = rect_plain(fx % 36, fy, height);
            if num46 == 48 {
                out.glow = Some(GlowData { mask: 56, src, color: martian });
            }
            if num46 == 49 {
                out.glow = Some(GlowData { mask: 117, src, color: METEOR_GLOW });
            }
        }
        // `case 10` (:6312-6322): closed doors, fx < 54 and row 32 (54).
        10 => {
            let num46 = fy / 54;
            if fx < 54 && num46 == 32 {
                out.glow = Some(GlowData { mask: 57, src: rect_plain(fx, fy % 54, height), color: martian });
            }
        }
        // `case 11` (:6323-6342): open doors, fx < 54, rows 32/33.
        11 => {
            let num46 = fy / 54;
            if fx < 54 {
                let src = rect_plain(fx, fy % 54, height);
                if num46 == 32 {
                    out.glow = Some(GlowData { mask: 58, src, color: martian });
                }
                if num46 == 33 {
                    out.glow = Some(GlowData { mask: 119, src, color: METEOR_GLOW });
                }
            }
        }
        // `case 88` (:6343-6362): dressers — 1998 stride, columns 24/25.
        88 => {
            let num46 = fx / 54;
            let num55 = fx / 1998;
            out.add_fr_x_delta = (-1998 * num55) as i16;
            out.add_fr_y_delta = (36 * num55) as i16;
            let src = rect_plain(fx % 54, fy, height);
            if num46 == 24 {
                out.glow = Some(GlowData { mask: 59, src, color: martian });
            }
            if num46 == 25 {
                out.glow = Some(GlowData { mask: 120, src, color: METEOR_GLOW });
            }
        }
        // `case 42` (:6363-6376): hanging lanterns — 2016 stride (56 rows
        // of 36), row 33.
        42 => {
            let num46 = fy / 36;
            let num54 = fy / 2016;
            out.add_fr_y_delta = (-2016 * num54) as i16;
            out.add_fr_x_delta = (36 * num54) as i16;
            if num46 == 33 {
                out.glow = Some(GlowData { mask: 63, src: rect_plain(fx, fy % 36, height), color: martian });
            }
        }
        // `case 87` (:6377-6396): pianos — 1998 stride, columns 26/27.
        87 => {
            let num46 = fx / 54;
            let num52 = fx / 1998;
            out.add_fr_x_delta = (-1998 * num52) as i16;
            out.add_fr_y_delta = (36 * num52) as i16;
            let src = rect_plain(fx % 54, fy, height);
            if num46 == 26 {
                out.glow = Some(GlowData { mask: 64, src, color: martian });
            }
            if num46 == 27 {
                out.glow = Some(GlowData { mask: 121, src, color: METEOR_GLOW });
            }
        }
        // `case 14` (:6397-6413): tables — columns 31/32 (54 grid), no
        // stride unwind.
        14 => {
            let num46 = fx / 54;
            let src = rect_plain(fx % 54, fy, height);
            if num46 == 31 {
                out.glow = Some(GlowData { mask: 67, src, color: martian });
            }
            if num46 == 32 {
                out.glow = Some(GlowData { mask: 124, src, color: METEOR_GLOW });
            }
        }
        // `case 93` (:6414-6428): lamps — 1998 stride AND `tileTop += 2`,
        // row 27.
        93 => {
            let num46 = fy / 54;
            let num51 = fy / 1998;
            out.add_fr_y_delta = (-1998 * num51) as i16;
            out.add_fr_x_delta = (36 * num51) as i16;
            out.top_delta = 2;
            if num46 == 27 {
                out.glow = Some(GlowData { mask: 62, src: rect_plain(fx, fy % 54, height), color: martian });
            }
        }
        // `case 18` (:6429-6445): workbenches — columns 27/28 (36 grid).
        18 => {
            let num46 = fx / 36;
            let src = rect_plain(fx % 36, fy, height);
            if num46 == 27 {
                out.glow = Some(GlowData { mask: 69, src, color: martian });
            }
            if num46 == 28 {
                out.glow = Some(GlowData { mask: 125, src, color: METEOR_GLOW });
            }
        }
        // `case 101` (:6446-6465): bookcases — 1998 stride with a 72 (not
        // 36) row step, columns 28/29.
        101 => {
            let num46 = fx / 54;
            let num50 = fx / 1998;
            out.add_fr_x_delta = (-1998 * num50) as i16;
            out.add_fr_y_delta = (72 * num50) as i16;
            let src = rect_plain(fx % 54, fy, height);
            if num46 == 28 {
                out.glow = Some(GlowData { mask: 60, src, color: martian });
            }
            if num46 == 29 {
                out.glow = Some(GlowData { mask: 115, src, color: METEOR_GLOW });
            }
        }
        // `case 104` (:6466-6486): grandfather clocks — 2016 stride with a
        // 90 row step AND `tileTop = 2`, columns 24/25 (36 grid).
        104 => {
            let num46 = fx / 36;
            let num48 = fx / 2016;
            out.add_fr_x_delta = (-2016 * num48) as i16;
            out.add_fr_y_delta = (90 * num48) as i16;
            out.top_override = Some(2);
            let src = rect_plain(fx % 36, fy, height);
            if num46 == 24 {
                out.glow = Some(GlowData { mask: 51, src, color: martian });
            }
            if num46 == 25 {
                out.glow = Some(GlowData { mask: 118, src, color: METEOR_GLOW });
            }
        }
        // `case 172` (:6487-6506): sinks — 2014 stride (53 rows of 38),
        // rows 28/29.
        172 => {
            let num46 = fy / 38;
            let num47 = fy / 2014;
            out.add_fr_y_delta = (-2014 * num47) as i16;
            out.add_fr_x_delta = (36 * num47) as i16;
            let src = rect_plain(fx, fy % 38, height);
            if num46 == 28 {
                out.glow = Some(GlowData { mask: 88, src, color: martian });
            }
            if num46 == 29 {
                out.glow = Some(GlowData { mask: 122, src, color: METEOR_GLOW });
            }
        }
        // `case 711` (:4931-4939) — the FIRST switch's glow, folded in here:
        // 711 has no second-switch case and no other pass writes its glow,
        // so the record is identical at this seam. The texture is the
        // boulder's own tile sheet (`TextureAssets.Tile[711]`, the
        // TILE_SHEET_GLOW sentinel), the rect the framed spelling
        // `(tileFrameX + addFrX, tileFrameY + addFrY, tileWidth, tileHeight)`
        // (:4938), and `glowColor` keeps the :4538 default
        // `Color.Transparent` = (255, 255, 255, 0) — the case never writes it.
        711 => out.glow = Some(GlowData { mask: TILE_SHEET_GLOW, src: rect_framed(height), color: [255, 255, 255, 0] }),
        _ => {}
    }
    out
}

/// The delegated dispatch arm the renderer calls after `draw_data` +
/// `apply_world_corrections` (`render/tiles.rs`): case 518's float, the
/// oasis/sea-oats biome rows, the FIRST-switch 711 glow and the second
/// (glow) switch. `color` is the cell light (`Lighting.GetColor`,
/// `TileDrawing.cs:4539`) as XNA bytes — the 637/638 lerps use it.
/// `tile_frame` is `Main.tileFrame[kind]` (`:4533`) — case 564's glow rect
/// reads its PRE-clear `* 36` row. Returns the glow record for the glow
/// bind (`render/tile_glow.rs`, `TileDrawing.cs:1343-1403`); the frame/top
/// deltas are folded into `data` in place.
#[allow(clippy::too_many_arguments)]
pub fn apply_cases_a(
    data: &mut TileDrawData,
    world: &World,
    tile: &Tile,
    x: i32,
    y: i32,
    color: [u8; 4],
    mouse_text_color: u8,
    martian_light: f32,
    disco: [u8; 3],
    tile_frame: i32,
) -> Option<GlowData> {
    let kind = tile.kind;
    let fx = tile.frame_x as i32;
    let fy = tile.frame_y as i32;
    // `case 711` — RainbowBoulder's FIRST-switch glow
    // (`TileDrawing.cs:4931-4939`): `glowTexture = Tile[711]`, framed rect,
    // colour left at the init `Color.Transparent` (`:4538`). Mask 0 is the
    // tile-sheet sentinel. The transparent tint draws nothing, but the
    // record keeps the cell out of the black merge (`flag`, `:971`).
    let first_switch_glow = if kind == 711 {
        Some(GlowData {
            mask: 0,
            src: [
                (fx + data.add_fr_x as i32) as i16,
                (fy + data.add_fr_y as i32) as i16,
                data.width as i16,
                data.height as i16,
            ],
            color: [0, 0, 0, 0],
        })
    } else {
        None
    };
    match kind {
        518 => {
            data.top = data
                .top
                .wrapping_add(lily_pad_top_delta(world, x, y, tile));
        }
        530 => {
            data.add_fr_y = data
                .add_fr_y
                .wrapping_add(oasis_plants_add_fr_y(world, x, y, fx, fy));
        }
        529 => {
            // `WorldGen.beachDistance` (WorldGen.cs:86914) — the same 380
            // constant drawdata.rs uses for the palm probe.
            data.add_fr_y = data
                .add_fr_y
                .wrapping_add(sea_oats_add_fr_y(world, x, y, fy, 380));
        }
        _ => {}
    }
    let half_brick_height = if tile.half_brick() { 8 } else { 0 };
    let pass = glow_switch(
        kind,
        fx,
        fy,
        data.add_fr_x as i32,
        data.add_fr_y as i32,
        data.width as i32,
        data.height as i32,
        half_brick_height,
        color,
        mouse_text_color,
        martian_light,
        disco,
        // The shared 217/218 arm value (`TileDrawing.cs:5523-5528`) the
        // C# still holds when case 564's glow rect reads it (`:6109`).
        tile_frame * 36,
    );
    pass.apply(data);
    pass.glow.or(first_switch_glow)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tr_core::drawdata::draw_data;
    use tr_core::tile::{Tile, FLAG_ACTIVE, FLAG_HALF_BRICK};

    fn put(w: &mut World, x: i32, y: i32, kind: u16) {
        let i = w.index(x, y);
        let mut t = Tile::default();
        t.kind = kind;
        t.flags |= FLAG_ACTIVE;
        w.tiles[i] = t;
    }

    fn tile_of(kind: u16, fx: i16, fy: i16) -> Tile {
        let mut t = Tile::default();
        t.kind = kind;
        t.frame_x = fx;
        t.frame_y = fy;
        t.flags |= FLAG_ACTIVE;
        t
    }

    /// The full pipeline the renderer runs: draw_data + this pass.
    fn dd(kind: u16, x: i32, y: i32, fx: i16, fy: i16) -> (TileDrawData, Option<GlowData>) {
        let w = World::new("t".into(), 64, 64);
        let t = tile_of(kind, fx, fy);
        let mut d = draw_data(kind, x, y, fx, fy, 0, false);
        let g = apply_cases_a(&mut d, &w, &t, x, y, [255, 255, 255, 255], 255, 0.0, [255, 0, 0], 0);
        (d, g)
    }

    // ---- case 518: the LilyPad float (TileDrawing.cs:4604-4630) ----

    #[test]
    fn test_DW_F9d_lily_pad_liquid_levels() {
        let w = World::new("t".into(), 64, 64);
        // liquid 0, no block below: num19 = -3 → top += 3.
        let t = tile_of(518, 0, 0);
        assert_eq!(lily_pad_top_delta(&w, 10, 10, &t), 3);
        // liquid 255: 255/16 - 3 = 12; no solid above → top -= 12.
        let mut t = tile_of(518, 0, 0);
        t.liquid = 255;
        assert_eq!(lily_pad_top_delta(&w, 10, 10, &t), -12);
        // liquid 128: 8 - 3 = 5 → top -= 5.
        t.liquid = 128;
        assert_eq!(lily_pad_top_delta(&w, 10, 10, &t), -5);
        // Solid above clamps deep levels to 8 → top -= 8.
        let mut w = World::new("t".into(), 64, 64);
        put(&mut w, 10, 9, 1); // stone: solid, not solid-top
        let mut t = tile_of(518, 0, 0);
        t.liquid = 255;
        assert_eq!(lily_pad_top_delta(&w, 10, 10, &t), -8);
        // A platform above is solid-TOP → no clamp.
        put(&mut w, 10, 9, 19);
        assert_eq!(lily_pad_top_delta(&w, 10, 10, &t), -12);
        // Half-brick above is not SolidTile → no clamp.
        let i = w.index(10, 9);
        w.tiles[i] = tile_of(1, 0, 0);
        w.tiles[i].flags |= FLAG_HALF_BRICK;
        assert_eq!(lily_pad_top_delta(&w, 10, 10, &t), -12);
    }

    #[test]
    fn test_DW_F9d_lily_pad_block_below() {
        // Dry pad over a half-brick: -16 + max(8, liquid/16).
        let mut w = World::new("t".into(), 64, 64);
        let i = w.index(10, 11);
        w.tiles[i] = tile_of(1, 0, 0);
        w.tiles[i].flags |= FLAG_HALF_BRICK;
        let t = tile_of(518, 0, 0);
        // liquid 0 below: -16 + 8 = -8 → top -= -8 = +8.
        assert_eq!(lily_pad_top_delta(&w, 10, 10, &t), 8);
        // liquid 255 below: -16 + 15 = -1 → top += 1.
        w.tiles[i].liquid = 255;
        assert_eq!(lily_pad_top_delta(&w, 10, 10, &t), 1);
        // Dry pad over a slope-1 block (blockType 2): num19 = -3 - 4 = -7
        // → top += 7. Slope 2 (blockType 3) is the same arm.
        for slope in [1u8, 2] {
            w.tiles[i] = tile_of(1, 0, 0);
            w.tiles[i].set_slope(slope);
            assert_eq!(lily_pad_top_delta(&w, 10, 10, &t), 7, "slope {slope}");
        }
        // Flat block below (blockType 0): unchanged +3.
        w.tiles[i] = tile_of(1, 0, 0);
        assert_eq!(lily_pad_top_delta(&w, 10, 10, &t), 3);
        // Inactive block below: nactive() false → unchanged +3.
        w.tiles[i].flags &= !FLAG_ACTIVE;
        assert_eq!(lily_pad_top_delta(&w, 10, 10, &t), 3);
        // The pad's own liquid wins over the block below (C# only probes
        // when liquid == 0).
        let mut t = tile_of(518, 0, 0);
        t.liquid = 64;
        w.tiles[i] = tile_of(1, 0, 0);
        w.tiles[i].flags |= FLAG_HALF_BRICK;
        assert_eq!(lily_pad_top_delta(&w, 10, 10, &t), -1); // 64/16-3 = 1
    }

    #[test]
    fn test_DW_F9d_lily_pad_case_wiring() {
        // The dispatch arm folds the delta into draw_data's result: the
        // static table keeps 16x16, top 0 + 3 = 3 on a dry pad.
        let (d, g) = dd(518, 10, 10, 0, 0);
        assert_eq!((d.width, d.height, d.top), (16, 16, 3));
        assert_eq!(g, None);
    }

    // ---- cases 529/530: the biome-influence rows ----

    #[test]
    fn test_DW_F9d_biome_influence_counts() {
        let mut w = World::new("t".into(), 64, 64);
        put(&mut w, 5, 5, 23); // corrupt grass
        put(&mut w, 6, 5, 25); // corrupt stone
        put(&mut w, 7, 5, 199); // crimson grass
        put(&mut w, 8, 5, 109); // hallow grass
        put(&mut w, 9, 5, 1); // stone: none of the sets
        assert_eq!(biome_influence(&w, 5, 8, 5, 5), (2, 1, 1));
        // Out-of-world cells are skipped (null in C#).
        assert_eq!(biome_influence(&w, 1000, 1002, 5, 5), (0, 0, 0));
    }

    #[test]
    fn test_DW_F9d_oasis_plants_pick() {
        // Anchor maths: num35 = y - fy%36/18 + 2, num36 = x - fx%54/18.
        let mut w = World::new("t".into(), 1024, 64);
        // No evil/hallow below → pick 0.
        assert_eq!(oasis_plants_add_fr_y(&w, 10, 10, 0, 0), 0);
        // Hallow wins ties (checked first).
        put(&mut w, 10, 12, 109);
        put(&mut w, 11, 12, 23);
        assert_eq!(oasis_plants_add_fr_y(&w, 10, 10, 0, 0), 36);
        // Crimson beats corrupt when hallow absent.
        let mut w = World::new("t".into(), 1024, 64);
        put(&mut w, 10, 12, 199);
        put(&mut w, 11, 12, 23);
        assert_eq!(oasis_plants_add_fr_y(&w, 10, 10, 0, 0), 72);
        // Corrupt alone → 3.
        let mut w = World::new("t".into(), 1024, 64);
        put(&mut w, 10, 12, 23);
        assert_eq!(oasis_plants_add_fr_y(&w, 10, 10, 0, 0), 108);
        // The anchor shifts with the sub-cell frame: fx 18 moves the band
        // one column left, fy 18 moves it one row up.
        let mut w = World::new("t".into(), 1024, 64);
        put(&mut w, 9, 11, 109);
        assert_eq!(oasis_plants_add_fr_y(&w, 10, 10, 18, 18), 36);
        assert_eq!(oasis_plants_add_fr_y(&w, 10, 10, 0, 0), 0);
    }

    #[test]
    fn test_DW_F9d_sea_oats_pick_and_beach() {
        let mut w = World::new("t".into(), 1024, 64);
        // Inland, no biome → 0 - fy.
        assert_eq!(sea_oats_add_fr_y(&w, 500, 10, 0, 380), 0);
        assert_eq!(sea_oats_add_fr_y(&w, 500, 10, 34, 380), -34);
        // Beach band (x < 380): pick 1.
        assert_eq!(sea_oats_add_fr_y(&w, 100, 10, 0, 380), 34);
        assert_eq!(sea_oats_add_fr_y(&w, 1000, 10, 0, 380), 34); // > 1024-380
        // Hallow below → 2 even on the beach.
        put(&mut w, 100, 11, 109);
        assert_eq!(sea_oats_add_fr_y(&w, 100, 10, 0, 380), 68);
        // Crimson → 3, corrupt → 4.
        put(&mut w, 100, 11, 199);
        assert_eq!(sea_oats_add_fr_y(&w, 100, 10, 0, 380), 102);
        put(&mut w, 100, 11, 23);
        assert_eq!(sea_oats_add_fr_y(&w, 100, 10, 0, 380), 136);
        // The wiring keeps the first switch's 32-tall / top -14 flip shape
        // and adds the row. A world-size inland cell (x 500 of 1024, no
        // biome below) → addFrY 0.
        let w = World::new("t".into(), 1024, 64);
        let t = tile_of(529, 0, 0);
        let mut d = draw_data(529, 500, 10, 0, 0, 0, false);
        let g = apply_cases_a(&mut d, &w, &t, 500, 10, [255; 4], 255, 0.0, [255, 0, 0], 0);
        assert_eq!((d.width, d.height, d.top, d.add_fr_y), (16, 32, -14, 0));
        assert_eq!(g, None);
        // A beach cell (x 100) picks row 1: addFrY += 34.
        let mut d = draw_data(529, 100, 10, 0, 0, 0, false);
        apply_cases_a(&mut d, &w, &t, 100, 10, [255; 4], 255, 0.0, [255, 0, 0], 0);
        assert_eq!(d.add_fr_y, 34);
    }

    // ---- the glow switch: per-case coverage ----

    #[test]
    fn test_DW_F9d_glow_simple_masks() {
        // 412/568/569/570/463/580: unconditional masks on the 16x16 sheet
        // grid, rect (fx, fy+addFrY) with addFrY 0 at t=0.
        for (kind, mask, col) in [
            (412u16, 202u16, [255, 255, 255, 255]),
            (568, 268, [255, 255, 255, 255]),
            (569, 269, [255, 255, 255, 255]),
            (570, 270, [255, 255, 255, 255]),
            (463, 243, [127, 127, 127, 0]),
            (580, 289, [225, 110, 110, 0]),
        ] {
            let (_, g) = dd(kind, 3, 7, 18, 36);
            let g = g.unwrap();
            assert_eq!(g.mask, mask, "kind {kind}");
            assert_eq!(g.color, col, "kind {kind}");
            assert_eq!(g.src, [18, 36, 16, 16], "kind {kind}");
        }
        // 634 is the ash-tree variant: 20x20 from the first switch's tree
        // arm (drawdata.rs :4650-4662), so its glow rect is 20x20.
        let (d, g) = dd(634, 3, 7, 18, 36);
        let g = g.unwrap();
        assert_eq!((g.mask, g.color), (315, [255, 255, 255, 255]));
        assert_eq!(g.src, [18, 36, 20, 20]);
        assert_eq!((d.width, d.height), (20, 20));
        // 657 gates on fy >= 54.
        assert!(dd(657, 0, 0, 0, 36).1.is_none());
        let g = dd(657, 0, 0, 0, 54).1.unwrap();
        assert_eq!((g.mask, g.color), (330, [255, 255, 255, 255]));
    }

    #[test]
    fn test_DW_F9d_glow_pulse_and_lerp() {
        // 656/701: white * mouseTextColor/255; at 255 full white, alpha 0.
        let (_, g) = dd(656, 0, 0, 0, 0);
        assert_eq!(g.unwrap().color, [255, 255, 255, 0]);
        let t = tile_of(656, 0, 0);
        let w = World::new("t".into(), 64, 64);
        let mut d = draw_data(656, 0, 0, 0, 0, 0, false);
        let g = apply_cases_a(&mut d, &w, &t, 0, 0, [255; 4], 128, 0.0, [255, 0, 0], 0).unwrap();
        assert_eq!(g.color, [128, 128, 128, 0]);
        // 564: (200,200,200,0) * pulse below column 36; none at/above 36.
        // W4-g34: the first switch keeps tileFrame*36, the glow rect reads it
        // pre-clear, then the pass's delta zeroes addFrY (the C# order).
        let (d, g) = dd(564, 0, 0, 18, 0);
        assert_eq!(d.add_fr_y, 0); // tile_frame 0 -> 0, delta 0
        assert_eq!(g.unwrap().color, [200, 200, 200, 0]);
        assert!(dd(564, 0, 0, 36, 0).1.is_none());
        // With a live animation row the rect carries it but the draw clears.
        let t = tile_of(564, 18, 0);
        let w = World::new("t".into(), 64, 64);
        let mut d = draw_data(564, 0, 0, 18, 0, 3, false);
        assert_eq!(d.add_fr_y, 108); // first switch: 3 * 36
        let g = apply_cases_a(&mut d, &w, &t, 0, 0, [255; 4], 255, 0.0, [255, 0, 0], 3).unwrap();
        assert_eq!(g.src[1], 108); // rect y = fy + pre-clear addFrY
        assert_eq!(d.add_fr_y, 0); // :6112 clear via the delta
        // 638: 75% lerp toward the cell colour, height minus half-brick.
        let mut t = tile_of(638, 0, 0);
        t.flags |= FLAG_HALF_BRICK;
        let w = World::new("t".into(), 64, 64);
        let mut d = draw_data(638, 0, 0, 0, 0, 0, false);
        let g = apply_cases_a(&mut d, &w, &t, 0, 0, [0, 0, 0, 255], 255, 0.0, [255, 0, 0], 0).unwrap();
        // lerp(255, 0, .75) = 63.75 → 63 (byte truncation).
        assert_eq!(g.color, [63, 63, 63, 255]);
        assert_eq!(g.src[3], 16 - 8);
        // 637: same lerp, framed rect, mask 0 (the painted sheet is not
        // bound here — deviation). The first switch's plant arm makes it
        // 20 px tall.
        let t = tile_of(637, 0, 0);
        let mut d = draw_data(637, 0, 0, 0, 0, 0, false);
        let g = apply_cases_a(&mut d, &w, &t, 0, 0, [0, 0, 0, 255], 255, 0.0, [255, 0, 0], 0).unwrap();
        assert_eq!(g.color, [63, 63, 63, 255]);
        assert_eq!(g.src, [0, 0, 16, 20]);
        assert_eq!(g.mask, 0);
    }

    #[test]
    fn test_DW_F9d_glow_platform_conduits() {
        // case 19 (TileDrawing.cs:6157-6173): rows 26/27 of the 18 grid,
        // rect Y folds % 18; martian glow b=100 at martianLight 0.
        let (_, g) = dd(19, 0, 0, 36, 26 * 18);
        let g = g.unwrap();
        assert_eq!((g.mask, g.src), (65, [36, 0, 16, 16]));
        assert_eq!(g.color, [100, 100, 100, 0]);
        let (_, g) = dd(19, 0, 0, 36, 27 * 18 + 4);
        let g = g.unwrap();
        assert_eq!((g.mask, g.src), (112, [36, 4, 16, 16]));
        assert_eq!(g.color, METEOR_GLOW);
        // Other rows: no glow.
        assert!(dd(19, 0, 0, 0, 25 * 18).1.is_none());
        assert!(dd(19, 0, 0, 0, 28 * 18).1.is_none());
        // martianLight 1 → b = 250.
        assert_eq!(martian_glow(1.0), [250, 250, 250, 0]);
        assert_eq!(martian_glow(0.5), [175, 175, 175, 0]);
    }

    #[test]
    fn test_DW_F9d_glow_row_gated_furniture() {
        // Bathtubs 90 / beds 79: rows 27/28 of the 36 grid.
        for kind in [90u16, 79] {
            let m = if kind == 90 { (52, 113) } else { (53, 114) };
            let g = dd(kind, 0, 0, 0, 27 * 36 + 8).1.unwrap();
            assert_eq!((g.mask, g.src[1]), (m.0, 8), "kind {kind}");
            let g = dd(kind, 0, 0, 0, 28 * 36).1.unwrap();
            assert_eq!(g.mask, m.1, "kind {kind}");
            assert!(dd(kind, 0, 0, 0, 26 * 36).1.is_none(), "kind {kind}");
        }
        // Candelabras 100: column 0 AND row 27.
        assert!(dd(100, 0, 0, 36, 27 * 36).1.is_none());
        assert_eq!(dd(100, 0, 0, 0, 27 * 36).1.unwrap().mask, 68);
        // Candles 33: column 0, row 26 of the 22 grid.
        assert_eq!(dd(33, 0, 0, 0, 26 * 22 + 6).1.unwrap().src[1], 6);
        assert!(dd(33, 0, 0, 18, 26 * 22).1.is_none());
        // Chandeliers 34: block 0, row 33 of 54.
        assert_eq!(dd(34, 0, 0, 0, 33 * 54).1.unwrap().mask, 55);
        assert!(dd(34, 0, 0, 54, 33 * 54).1.is_none());
        // Doors 10/11: fx < 54 gate.
        assert!(dd(10, 0, 0, 54, 32 * 54).1.is_none());
        assert_eq!(dd(10, 0, 0, 0, 32 * 54).1.unwrap().mask, 57);
        assert_eq!(dd(11, 0, 0, 0, 32 * 54).1.unwrap().mask, 58);
        assert_eq!(dd(11, 0, 0, 0, 33 * 54).1.unwrap().mask, 119);
        assert!(dd(11, 0, 0, 54, 33 * 54).1.is_none());
        // Chests 21/467 + fake chests 441/468: columns 48/49 of 36.
        for kind in [21u16, 467, 441, 468] {
            let g = dd(kind, 0, 0, 48 * 36 + 18, 0).1.unwrap();
            assert_eq!((g.mask, g.src[0]), (56, 18), "kind {kind}");
            let g = dd(kind, 0, 0, 49 * 36, 0).1.unwrap();
            assert_eq!(g.mask, 117, "kind {kind}");
            assert!(dd(kind, 0, 0, 47 * 36, 0).1.is_none(), "kind {kind}");
        }
        // Tables 14: columns 31/32 of 54. Workbenches 18: 27/28 of 36.
        assert_eq!(dd(14, 0, 0, 31 * 54, 0).1.unwrap().mask, 67);
        assert_eq!(dd(14, 0, 0, 32 * 54, 0).1.unwrap().mask, 124);
        assert_eq!(dd(18, 0, 0, 27 * 36, 0).1.unwrap().mask, 69);
        assert_eq!(dd(18, 0, 0, 28 * 36, 0).1.unwrap().mask, 125);
        // Chairs 15: rows 32/33 of 40; toilets 497 share the arm but never
        // glow (the `typeCache == 15` gate, :6251).
        assert_eq!(dd(15, 0, 0, 0, 32 * 40).1.unwrap().mask, 54);
        assert_eq!(dd(15, 0, 0, 0, 33 * 40).1.unwrap().mask, 116);
        assert!(dd(497, 0, 0, 0, 32 * 40).1.is_none());
        assert!(dd(15, 0, 0, 0, 31 * 40).1.is_none());
    }

    #[test]
    fn test_DW_F9d_glow_stride_unwinds() {
        // The C# glow gates read the RAW tileFrameX — a style-block-1
        // dresser (fx 1998 + 24*54) unwinds its frame but its raw column
        // is 61, NOT 24, so it does NOT glow. The unwinds run regardless.
        // Dressers 88 (TileDrawing.cs:6343-6362).
        let (d, g) = dd(88, 0, 0, 1998, 0);
        assert_eq!((d.add_fr_x, d.add_fr_y), (-1998, 36));
        assert_eq!(g, None);
        let (d, g) = dd(88, 0, 0, (1998 + 24 * 54) as i16, 0);
        assert_eq!((d.add_fr_x, d.add_fr_y), (-1998, 36));
        assert_eq!(g, None, "raw column 61 is not a glow column");
        // Block-0 glow columns 24/25 (no unwind at fx < 1998).
        let (d, g) = dd(88, 0, 0, (24 * 54) as i16, 0);
        let g = g.unwrap();
        assert_eq!((g.mask, g.src[0]), (59, 0));
        assert_eq!((d.add_fr_x, d.add_fr_y), (0, 0));
        assert_eq!(dd(88, 0, 0, (25 * 54) as i16, 0).1.unwrap().mask, 120);
        // Pianos 87: columns 26/27, same raw-gate rule.
        let (d, g) = dd(87, 0, 0, (1998 + 26 * 54) as i16, 0);
        assert_eq!((d.add_fr_x, d.add_fr_y), (-1998, 36));
        assert_eq!(g, None);
        assert_eq!(dd(87, 0, 0, (26 * 54) as i16, 0).1.unwrap().mask, 64);
        assert_eq!(dd(87, 0, 0, (27 * 54) as i16, 0).1.unwrap().mask, 121);
        // Benches 89: columns 29/30.
        let (d, g) = dd(89, 0, 0, (1998 + 29 * 54) as i16, 0);
        assert_eq!((d.add_fr_x, d.add_fr_y), (-1998, 36));
        assert_eq!(g, None);
        assert_eq!(dd(89, 0, 0, (29 * 54) as i16, 0).1.unwrap().mask, 66);
        assert_eq!(dd(89, 0, 0, (30 * 54) as i16, 0).1.unwrap().mask, 123);
        // Bookcases 101: the row step is 72, not 36.
        let (d, g) = dd(101, 0, 0, (1998 + 28 * 54) as i16, 0);
        assert_eq!((d.add_fr_x, d.add_fr_y), (-1998, 72));
        assert_eq!(g, None);
        let (d, g) = dd(101, 0, 0, (28 * 54) as i16, 0);
        assert_eq!((d.add_fr_x, d.add_fr_y), (0, 0));
        assert_eq!(g.unwrap().mask, 60);
        assert_eq!(dd(101, 0, 0, (29 * 54) as i16, 0).1.unwrap().mask, 115);
        // Two style blocks stack: fx 2*1998 → -3996 / +72 (dressers).
        let (d, _) = dd(88, 0, 0, (2 * 1998) as i16, 0);
        assert_eq!((d.add_fr_x, d.add_fr_y), (-3996, 72));
    }

    #[test]
    fn test_DW_F9d_glow_y_strides_and_tops() {
        // Same raw-frame rule for the Y strides: the gate reads the raw
        // tileFrameY row, so block-1 frames unwind but never glow.
        // Hanging lanterns 42 (TileDrawing.cs:6363-6376): fy 2016 unwinds
        // -2016/+36; row 33 of block 1 does NOT glow (raw row 89).
        let (d, g) = dd(42, 0, 0, 0, 2016);
        assert_eq!((d.add_fr_x, d.add_fr_y), (36, -2016));
        assert_eq!(g, None);
        let (d, g) = dd(42, 0, 0, 0, (2016 + 33 * 36) as i16);
        assert_eq!((d.add_fr_x, d.add_fr_y), (36, -2016));
        assert_eq!(g, None);
        // Block-0 row 33 glows without the unwind.
        let (d, g) = dd(42, 0, 0, 0, (33 * 36) as i16);
        assert_eq!((d.add_fr_x, d.add_fr_y), (0, 0));
        assert_eq!(g.unwrap().mask, 63);
        // Chairs 15/497: the 2040 stride; block-1 rows unwind, no glow.
        let (d, g) = dd(15, 0, 0, 0, (2040 + 32 * 40) as i16);
        assert_eq!((d.add_fr_x, d.add_fr_y), (36, -2040));
        assert_eq!(g, None);
        let (d, g) = dd(497, 0, 0, 0, (2040 + 32 * 40) as i16);
        assert_eq!((d.add_fr_x, d.add_fr_y), (36, -2040));
        assert_eq!(g, None, "497 unwinds but never glows");
        // Lamps 93: stride AND tileTop += 2 on EVERY frame; block-0 row 27
        // glows (raw row 27), block-1 row 27 does not.
        let (d, g) = dd(93, 0, 0, 0, (1998 + 27 * 54) as i16);
        assert_eq!((d.add_fr_x, d.add_fr_y, d.top), (36, -1998, 2));
        assert_eq!(g, None);
        let (d, g) = dd(93, 0, 0, 0, (27 * 54) as i16);
        assert_eq!((d.add_fr_x, d.add_fr_y, d.top), (0, 0, 2));
        assert_eq!(g.unwrap().mask, 62);
        // Grandfather clocks 104: 2016 stride, 90 row step, tileTop = 2
        // everywhere; block-0 columns 24/25 glow.
        let (d, g) = dd(104, 0, 0, (2016 + 24 * 36) as i16, 0);
        assert_eq!((d.add_fr_x, d.add_fr_y, d.top), (-2016, 90, 2));
        assert_eq!(g, None);
        let (d, g) = dd(104, 0, 0, (24 * 36) as i16, 0);
        assert_eq!((d.add_fr_x, d.add_fr_y, d.top), (0, 0, 2));
        assert_eq!(g.unwrap().mask, 51);
        assert_eq!(dd(104, 0, 0, (25 * 36) as i16, 0).1.unwrap().mask, 118);
        // Sinks 172: 2014 stride (53 rows of 38), block-0 rows 28/29 glow;
        // the first switch's 18-px seam height survives on the seam row.
        let (d, g) = dd(172, 0, 0, 0, (2014 + 28 * 38) as i16);
        assert_eq!((d.add_fr_x, d.add_fr_y), (36, -2014));
        assert_eq!(g, None);
        let (d, g) = dd(172, 0, 0, 0, (28 * 38) as i16);
        assert_eq!((d.add_fr_x, d.add_fr_y), (0, 0));
        let g = g.unwrap();
        assert_eq!((g.mask, g.src[1]), (88, (28 * 38 % 38) as i16));
        assert_eq!(dd(172, 0, 0, 0, (29 * 38) as i16).1.unwrap().mask, 122);
        // The seam row (fy % 38 == 18) keeps height 18 in the glow rect.
        let g = dd(172, 0, 0, 0, (28 * 38 + 18) as i16).1.unwrap();
        assert_eq!(g.src[3], 18);
    }

    #[test]
    fn test_DW_F9d_glow_long_moss_columns() {
        // case 184 (:6114-6151): the five moss colours by column, raw
        // frame rect (no addFr*), 20 px wide from the first switch.
        for (fx, col) in [
            (110i16, LAVA_MOSS_GLOW),
            (132, KRYPTON_MOSS_GLOW),
            (154, XENON_MOSS_GLOW),
            (176, ARGON_MOSS_GLOW),
            (198, VIOLET_MOSS_GLOW),
        ] {
            let (d, g) = dd(184, 0, 0, fx, 18);
            let g = g.unwrap();
            assert_eq!((g.mask, g.color), (127, col), "fx {fx}");
            assert_eq!(g.src, [fx, 18, 20, 16], "fx {fx}");
            assert_eq!(d.width, 20);
        }
        // Non-moss columns: no glow.
        assert!(dd(184, 0, 0, 88, 0).1.is_none());
        // The disco column 220 takes Main.DiscoR/G/B (alpha 255).
        let w = World::new("t".into(), 64, 64);
        let t = tile_of(184, 220, 0);
        let mut d = draw_data(184, 0, 0, 220, 0, 0, false);
        let g = apply_cases_a(&mut d, &w, &t, 0, 0, [255; 4], 255, 0.0, [12, 34, 56], 0)
            .unwrap();
        assert_eq!((g.mask, g.color), (127, [12, 34, 56, 255]));
    }

    /// TILE-DRAW-2 (W4-f15): 564's glow rect reads the PRE-clear animation
    /// row — C# :6109 reads `addFrY = tileFrame*36` before :6112 zeroes it.
    #[test]
    fn test_TD2_glow_564_rect_reads_pre_clear_row() {
        let w = World::new("t".into(), 64, 64);
        let t = tile_of(564, 18, 0);
        let mut d = draw_data(564, 0, 0, 18, 0, 3, false);
        let g = apply_cases_a(&mut d, &w, &t, 0, 0, [255; 4], 255, 0.0, [255, 0, 0], 3)
            .unwrap();
        // Sprite: addFrY stays cleared; glow rect Y carries 3 * 36 = 108.
        assert_eq!(d.add_fr_y, 0);
        assert_eq!(g.src, [18, 108, 16, 16]);
        assert_eq!(g.mask, 267);
    }

    /// TILE-DRAW-2 (W4-f15): 711's FIRST-switch glow record
    /// (`TileDrawing.cs:4931-4939`): the glow texture is the tile sheet
    /// (`TILE_SHEET_GLOW`, the array sentinel the draw pass maps back to
    /// `TextureAssets.Tile`), framed rect on the post-first-switch metrics
    /// (18 wide on later columns, 20 tall), colour stays the init
    /// `Color.Transparent` (`:4538`).
    #[test]
    fn test_TD2_glow_711_first_switch_record() {
        let (d, g) = dd(711, 0, 0, 0, 0);
        assert_eq!(d.width, 16);
        let g = g.unwrap();
        assert_eq!((g.mask, g.color), (TILE_SHEET_GLOW, [255, 255, 255, 0]));
        assert_eq!(g.src, [0, 0, 16, 20]);
        let (_, g) = dd(711, 0, 0, 18, 36);
        let g = g.unwrap();
        assert_eq!(g.src, [18, 36, 18, 20]);
    }

    #[test]
    fn test_DW_d62_rainbow_boulder_first_switch_glow() {
        // case 711 (TileDrawing.cs:4931-4939): the glow texture is the
        // boulder's own sheet (TILE_SHEET_GLOW), the rect is the framed
        // spelling, glowColor stays Color.Transparent (255,255,255,0).
        let (d, g) = dd(711, 0, 0, 0, 0);
        let g = g.unwrap();
        assert_eq!(g.mask, TILE_SHEET_GLOW);
        assert_eq!(g.src, [0, 0, 16, 20]); // fx 0 keeps width 16
        assert_eq!(g.color, [255, 255, 255, 0]);
        assert_eq!((d.width, d.height), (16, 20));
        // Later frame columns: width 18 rides the glow rect too (:4938
        // reads the post-`tileWidth = 18` local).
        let (d, g) = dd(711, 0, 0, 18, 0);
        assert_eq!(g.unwrap().src, [18, 0, 18, 20]);
        assert_eq!((d.width, d.height), (18, 20));
        // The animator row folds into the rect Y (addFrY = tileFrame*38).
        let w = World::new("t".into(), 64, 64);
        let t = tile_of(711, 0, 0);
        let mut d = draw_data(711, 0, 0, 0, 0, 2, false);
        let g = apply_cases_a(&mut d, &w, &t, 0, 0, [255; 4], 255, 0.0, [255, 0, 0], 0).unwrap();
        assert_eq!(g.src, [0, 76, 16, 20]);
    }

    #[test]
    fn test_DW_F9d_non_glow_kinds_untouched() {
        // Kinds outside both switches keep the table result byte-for-byte.
        for kind in [1u16, 2, 3, 19 + 1, 40, 137, 326, 751] {
            let w = World::new("t".into(), 64, 64);
            let t = tile_of(kind, 18, 36);
            let base = draw_data(kind, 3, 7, 18, 36, 0, false);
            let mut d = base;
            let g = apply_cases_a(&mut d, &w, &t, 3, 7, [255; 4], 255, 0.0, [255, 0, 0], 0);
            assert_eq!(d, base, "kind {kind}");
            assert_eq!(g, None, "kind {kind}");
        }
    }

    #[test]
    fn test_DW_F9d_i16_extremes_do_not_panic() {
        // The stride unwinds at the i16 frame extremes wrap like C#'s
        // short arithmetic (the deltas ride i32 then cast).
        for kind in [88u16, 87, 89, 101, 104, 42, 93, 15, 497, 172, 518, 529, 530] {
            let (d, _) = dd(kind, 5, 3, 32767, 32767);
            let _ = d;
        }
        // fx 32767 / 1998 = 16 → addFrX -31968, addFrY 576 (dressers).
        let (d, _) = dd(88, 0, 0, 32767, 0);
        assert_eq!((d.add_fr_x, d.add_fr_y), (-31968, 576));
    }

    /// W4-g34 deviation-fix witness (WRITTEN NOT RUN — code-only mode):
    /// `solid_tile` is the FULL `WorldGen.SolidTile` (`WorldGen.cs:70111-
    /// 70132`) — the case-4 probe now feeds from it (`render/tiles.rs`), so
    /// the half-brick / slope / inactive / null clauses all veto. Kind 1
    /// (StoneBlock) is solid and not solid-top.
    #[test]
    fn solid_tile_is_the_full_worldgen_predicate() {
        use tr_core::tile::FLAG_INACTIVE;
        let mut w = World::new("t".into(), 64, 64);
        // A null tile (outside the array) is true (:70115-70118).
        assert!(solid_tile(&w, -1, -1));
        // An inactive (air) cell is false.
        assert!(!solid_tile(&w, 10, 10));
        // A plain solid block is true.
        put(&mut w, 10, 9, 1);
        assert!(solid_tile(&w, 10, 9));
        // Half-brick vetoes (:70119).
        let i = w.index(10, 9);
        w.tiles[i].flags |= FLAG_HALF_BRICK;
        assert!(!solid_tile(&w, 10, 9));
        w.tiles[i].flags &= !FLAG_HALF_BRICK;
        // Slope vetoes.
        w.tiles[i].set_slope(1);
        assert!(!solid_tile(&w, 10, 9));
        w.tiles[i].set_slope(0);
        // Actuated-inactive vetoes.
        w.tiles[i].flags |= FLAG_INACTIVE;
        assert!(!solid_tile(&w, 10, 9));
        w.tiles[i].flags &= !FLAG_INACTIVE;
        assert!(solid_tile(&w, 10, 9));
        // A solid-top kind (platform 19) vetoes even when active.
        put(&mut w, 11, 9, 19);
        assert!(!solid_tile(&w, 11, 9));
    }
}
