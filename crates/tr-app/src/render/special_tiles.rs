//! S1-G-1e — the special-tile slice: `DrawSpecialTilesLegacy`
//! (`TileDrawing.cs:7893-8024`), the chest frame offsets
//! (`CacheSpecialDraws_Part2`, `TileDrawing.cs:4287-4330`, +0/38/76) and the
//! platform slope side-fills (`DrawBasicTile`, `TileDrawing.cs:1515-1545`,
//! glow arm `:1367-1401`).
//!
//! **UNWIRED module (code-only mode, 2026-11-18).** The draw-pass seam
//! (`render/tiles.rs::draw_tiles` + `tr_core::drawdata`) is contested by the
//! in-flight S1-G-1 siblings (vines G-1a, ropes G-1b, cracks G-1c, pylons
//! G-1d), so this slice lands as pure draw/decision functions with the
//! wiring deferred to the family's merge step — the `drawdata_anim.rs`
//! (S1-F-9b) pattern. Intended call sites:
//! * The chest frame offsets MOVED to `tr_render::chest_frames` and were
//!   WIRED by W4-d58 — `render/tiles.rs::draw_tiles` applies them after the
//!   drawdata dispatch, keyed on `chest::find_chest` over the anchor with
//!   the per-draw `_chestPositions` cache (`TileDrawing.cs:4300-4303`).
//! * [`platform_side_fill`] + [`push_platform_side_fill`] — the
//!   `TileID.Sets.Platforms && slope > 0` arm of `DrawBasicTile`, BEFORE the
//!   generic `push_slope` column path (the C# returns early, `:1544`).
//! * [`draw_sun_orb`] / [`draw_butterfly_cage`] / [`push_weapon_rack_item`] —
//!   the non-solid-layer special pass (`TileDrawing.cs:736`), fed by the
//!   `CacheSpecialDraws_Part1` legacy-point collection (`:4152-4255`).
//!
//! Deferrals (recorded in docs/ANALYSIS.md "S1-G-1e special tiles"):
//! * The tile-entity arms (395 item frame, 520 food platter, 471 weapons
//!   rack entity lookup) need the `TileEntity` store the port does not have
//!   yet — only the 334 weapon-rack DISPLAY decode maths is ported here.
//! * The butterfly cage frame input rides `Main.butterflyCageFrame[8, f]`,
//!   animated by `AnimateTiles_CritterCages` — the S1-F-9b deferred animator
//!   half; the caller passes the frame row until that lands.
//! * `Main.tileBatch.SetLayer(Layer_BehindTiles, …)` has no port analogue
//!   yet (the S1-G-13 layer-target family is unwired); the side-fill push
//!   draws into the world range and the layer decision is returned in
//!   [`PlatformSideFill`] for the wiring step.
#![allow(dead_code)] // UNWIRED — see the module doc; the wiring is the S1-G-1 merge step.

use tr_core::tiledata::{SET_BLOCKS_STAIRS, SET_PLATFORMS, TILE_SOLID};
use tr_render::SpriteRenderer;

// ---------------------------------------------------------------------------
// XNA byte maths (the G-3 verifier finding: TWO-step truncations)
// ---------------------------------------------------------------------------

/// XNA `Color.MultiplyRGBA` — per-channel `(a * b) >> 8` on the raw bytes
/// (integer truncation, NOT `/255` rounding). The first of the two
/// truncation steps; the float scale ([`xna_scale_color`]) is the second.
pub(crate) fn xna_multiply_rgba(a: [u8; 4], b: [u8; 4]) -> [u8; 4] {
    let m = |x: u8, y: u8| ((x as i32 * y as i32) >> 8) as u8;
    [m(a[0], b[0]), m(a[1], b[1]), m(a[2], b[2]), m(a[3], b[3])]
}

/// XNA `Color operator*(Color, float)` — `(int)(channel * scale)` per
/// channel, truncation toward zero (the second truncation step).
pub(crate) fn xna_scale_color(c: [u8; 4], scale: f32) -> [u8; 4] {
    let s = |x: u8| (x as f32 * scale) as i32 as u8;
    [s(c[0]), s(c[1]), s(c[2]), s(c[3])]
}

// ---------------------------------------------------------------------------
// Chest frames — CacheSpecialDraws_Part2 (TileDrawing.cs:4287-4330)
// ---------------------------------------------------------------------------
//
// W4-d58 (R2 chest slice): the frame-selection maths MOVED to
// `tr_render::chest_frames` (`chest_anchor`/`chest_frame_adjust`/
// `ChestFrameAdjust`) and is WIRED — `render/tiles.rs::draw_tiles` applies
// it after the drawdata dispatch (the C# `:993` position), keyed on
// `chest::find_chest` over the anchor with the per-draw `_chestPositions`
// cache. The i43 sim-side lid animator (`chest::update_chest_frames`) feeds
// `WorldChest.frame`; the draw side reads it without RNG.

// ---------------------------------------------------------------------------
// Platform slope side-fills — DrawBasicTile (TileDrawing.cs:1515-1545)
// ---------------------------------------------------------------------------

/// The neighbour-cell facts the side-fill gates read. `invisible` is
/// `Tile.invisibleBlock()` (`Tile.cs:528-531`, `bTileHeader3 & 0x20`) — the
/// port's `Tile` does not carry that bit yet, so the wiring passes `false`
/// (recorded deviation, ANALYSIS "S1-G-1e special tiles").
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct FillNeighbor {
    pub active: bool,
    pub kind: u16,
    pub slope: u8,
    pub half_brick: bool,
    pub invisible: bool,
}

impl FillNeighbor {
    /// Snapshot a world tile; the invisible bit reads `false` until the
    /// port's `Tile` carries `bTileHeader3` (see the struct doc).
    pub(crate) fn from_tile(t: &tr_core::tile::Tile) -> Self {
        Self {
            active: t.active(),
            kind: t.kind,
            slope: t.slope(),
            half_brick: t.half_brick(),
            invisible: false,
        }
    }
}

/// `Tile.blockType()` (`Tile.cs:202-214`): half brick → 1, slope n → n+1,
/// full block → 0.
fn block_type(half_brick: bool, slope: u8) -> u8 {
    if half_brick {
        return 1;
    }
    if slope > 0 {
        return slope + 1;
    }
    0
}

/// The side-fill draw decision: a 16×16 sprite one cell BELOW the platform
/// (`normalTilePosition + (0, 16)`), on the behind-tiles layer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct PlatformSideFill {
    /// The source rect: `(198|324|162|306, drawData.tileFrameY, 16, 16)`
    /// (`TileDrawing.cs:1527-1531` / `:1537-1541`). The glow arm
    /// (`:1375-1383` / `:1389-1397`) reuses the SAME X pick on the glow
    /// rect, keeping the glow rect's Y/size.
    pub src: [i32; 4],
    /// `SetLayer(Layer_BehindTiles, 0)` (`:1526`/`:1536`; glow arm layer 1,
    /// `:1384`/`:1398`) — the port has no tile layers yet (S1-G-13).
    pub behind_tiles: bool,
}

/// `DrawBasicTile`'s platform-slope arm (`TileDrawing.cs:1515-1545`).
/// `below`/`left_below`/`right_below` are `Main.tile[tileX, tileY+1]`,
/// `[tileX-1, tileY+1]`, `[tileX+1, tileY+1]` (`:1519-1521`). The caller
/// draws the platform's normal rect FIRST (`:1523`) and returns early —
/// the generic slope-column path never runs for platforms.
pub(crate) fn platform_side_fill(
    slope: u8,
    frame_y: i16,
    below: &FillNeighbor,
    left_below: &FillNeighbor,
    right_below: &FillNeighbor,
    show_invisible_blocks: bool,
) -> Option<PlatformSideFill> {
    // The `(!tile.active() || … || !BlocksStairs[tile.type])` tail shared
    // by both arms (`:1524` / `:1534`), with the per-arm blockType veto.
    let below_allows = |veto_block_type: u8| {
        !below.active
            || (!show_invisible_blocks && below.invisible)
            || (block_type(below.half_brick, below.slope) != 0
                && block_type(below.half_brick, below.slope) != veto_block_type)
            || !SET_BLOCKS_STAIRS.has(below.kind)
    };
    if slope == 1
        && right_below.active
        && (show_invisible_blocks || !right_below.invisible)
        && TILE_SOLID.has(right_below.kind)
        && right_below.slope != 2
        && !right_below.half_brick
        && below_allows(5)
    {
        // TileDrawing.cs:1527-1531 — an unsloped platform neighbour uses
        // the platform-specific fill column 324, else 198.
        let x = if SET_PLATFORMS.has(right_below.kind) && right_below.slope == 0 {
            324
        } else {
            198
        };
        return Some(PlatformSideFill {
            src: [x, frame_y as i32, 16, 16],
            behind_tiles: true,
        });
    }
    if slope == 2
        && left_below.active
        && (show_invisible_blocks || !left_below.invisible)
        && TILE_SOLID.has(left_below.kind)
        && left_below.slope != 1
        && !left_below.half_brick
        && below_allows(4)
    {
        // TileDrawing.cs:1537-1541
        let x = if SET_PLATFORMS.has(left_below.kind) && left_below.slope == 0 {
            306
        } else {
            162
        };
        return Some(PlatformSideFill {
            src: [x, frame_y as i32, 16, 16],
            behind_tiles: true,
        });
    }
    None
}

/// Draw the side-fill sprite: `normalTilePosition + (0, 16)`, the platform
/// sheet rect from [`PlatformSideFill::src`] (`TileDrawing.cs:1532`/`:1542`).
/// `sheet_uv` maps platform-sheet pixels to atlas UVs (the caller's
/// `Atlas::uv("Tiles_19", …)` equivalent); the tint is `drawData.finalColor`.
pub(crate) fn push_platform_side_fill(
    r: &mut SpriteRenderer,
    sheet_uv: impl Fn(f32, f32, f32, f32) -> Option<[f32; 4]>,
    pos: [f32; 2],
    fill: &PlatformSideFill,
    color: [f32; 4],
) {
    let [sx, sy, sw, sh] = fill.src;
    if let Some(uv) = sheet_uv(sx as f32, sy as f32, sw as f32, sh as f32) {
        r.push_sprite([pos[0], pos[1] + 16.0], [sw as f32, sh as f32], uv, color);
    }
}

// ---------------------------------------------------------------------------
// DrawSpecialTilesLegacy (TileDrawing.cs:7893-8024)
// ---------------------------------------------------------------------------

/// The 237 arm (`TileDrawing.cs:7909-7912`): the Lihzahrd sun orb, drawn
/// from the tile at `(x*16 + 8, y*16 - 36)` rotating about the texture
/// centre with `Main.sunCircle`, tinted
/// `Color(mouseTextColor, mouseTextColor, mouseTextColor, 0)`.
/// `uv`/`size` are the full `TextureAssets.SunOrb` texture (the caller's
/// decode); the origin is the INTEGER `Width()/2, Height()/2` (`:7911`).
#[allow(clippy::too_many_arguments)]
pub(crate) fn draw_sun_orb(
    r: &mut SpriteRenderer,
    uv: [f32; 4],
    size: [f32; 2],
    screen_pos: [f32; 2],
    off: [f32; 2],
    tx: i32,
    ty: i32,
    mouse_text_color: u8,
    sun_circle: f32,
) {
    // TileDrawing.cs:7911 — the anchor position and centre origin.
    let pos = [
        (tx * 16) as f32 - screen_pos[0] + 8.0 + off[0],
        (ty * 16) as f32 - screen_pos[1] - 36.0 + off[1],
    ];
    let origin = [(size[0] as i32 / 2) as f32, (size[1] as i32 / 2) as f32];
    let c = mouse_text_color as f32 / 255.0;
    // The C# draws with the origin at `pos`: the unrotated top-left is
    // `pos - origin` (scale 1).
    r.push_sprite_origin_rot(
        [pos[0] - origin[0], pos[1] - origin[1]],
        size,
        uv,
        [c, c, c, 0.0],
        sun_circle,
        origin,
    );
}

/// `GetWaterAnimalCageFrame` (`TileDrawing.cs:6528-6533`):
/// `(x - frameX/18)/2 * ((y - frameY/18)/3) % Main.cageFrames` — C# integer
/// division truncates toward zero, `Main.cageFrames = 25`
/// (`Main.cs:66901`, the port's `tile_light::CAGE_FRAMES`).
pub(crate) fn water_animal_cage_frame(x: i32, y: i32, frame_x: i16, frame_y: i16) -> i32 {
    let num = x - frame_x as i32 / 18;
    let num2 = y - frame_y as i32 / 18;
    (num / 2 * (num2 / 3)) % tr_core::tile_light::CAGE_FRAMES as i32
}

/// The 620 arm's halo tint (`TileDrawing.cs:8013-8017`):
/// `new Color(127,127,127,0).MultiplyRGBA(hslToRgb((t + j/6) % 1, 1, 0.5))`,
/// then `*= 1f - num10 * 0.5f` with `num10 = 1` (`:8009`) → `* 0.5`, then
/// `A = 0`. TWO truncation steps: the `>> 8` byte multiply, then the
/// `(int)(byte * 0.5)` float scale (the G-3 verifier finding).
pub(crate) fn cage_halo_color(j: u32, global_time_wrapped_hourly: f32) -> [f32; 4] {
    let hsl = tr_core::tile_light::hsl_to_rgb(
        (global_time_wrapped_hourly + j as f32 / 6.0) % 1.0,
        1.0,
        0.5,
    );
    // The palette returns exact `byte / 255.0` floats; `round` recovers the
    // byte without the f32 divide/multiply round-trip landing a hair below
    // the integer (a plain `as i32` could truncate 126.0 - 3e-5 to 125).
    let hsl_bytes = [
        (hsl[0] * 255.0).round() as u8,
        (hsl[1] * 255.0).round() as u8,
        (hsl[2] * 255.0).round() as u8,
        255,
    ];
    let multiplied = xna_multiply_rgba([127, 127, 127, 0], hsl_bytes);
    let mut scaled = xna_scale_color(multiplied, 0.5);
    scaled[3] = 0; // `color4.A = 0` (TileDrawing.cs:8017)
    [
        scaled[0] as f32 / 255.0,
        scaled[1] as f32 / 255.0,
        scaled[2] as f32 / 255.0,
        0.0,
    ]
}

/// The 620 arm (`TileDrawing.cs:8001-8022`): the butterfly critter cage.
/// `Extra[202]` sheet, source `(0, 34 * butterflyCageFrame[8, f], 32, 32)`,
/// base position `(x*16, y*16 + 2) + off`; a white base draw, six HSL halo
/// copies on a radius-4 ring (`(2*num10 + 2)`, `num10 = 1`), then a
/// `Color(255,255,255,0) * 0.1f` overlay. `cage_frame` is the animated
/// `Main.butterflyCageFrame[8, waterAnimalCageFrame]` row — the
/// `AnimateTiles_CritterCages` half deferred by S1-F-9b; the caller passes
/// it. `sheet_uv` maps `Extra[202]` pixels to atlas UVs. The C#'s
/// `Main.critterCage = true` proximity flag (`:8005`) gates the deferred
/// cage animator — it rides the wiring step with the animator itself.
#[allow(clippy::too_many_arguments)]
pub(crate) fn draw_butterfly_cage(
    r: &mut SpriteRenderer,
    sheet_uv: impl Fn(f32, f32, f32, f32) -> Option<[f32; 4]>,
    screen_pos: [f32; 2],
    off: [f32; 2],
    tx: i32,
    ty: i32,
    cage_frame: i32,
    global_time_wrapped_hourly: f32,
) {
    // TileDrawing.cs:8010-8012 — the base draw.
    let Some(uv) = sheet_uv(0.0, 34.0 * cage_frame as f32, 32.0, 32.0) else {
        return;
    };
    let vector = [
        (tx * 16) as f32 - screen_pos[0] + off[0],
        (ty * 16) as f32 - screen_pos[1] + 2.0 + off[1],
    ];
    r.push_sprite(vector, [32.0, 32.0], uv, [1.0, 1.0, 1.0, 1.0]);
    // TileDrawing.cs:8013-8020 — the six halo copies. `num9 = 6`,
    // `num10 = 1` → ring radius `2*1 + 2 = 4`; `ToRotationVector2` is
    // `(cos, sin)` (Utils.cs:1665-1668).
    for j in 0..6u32 {
        let angle = j as f32 / 6.0 * (std::f32::consts::PI * 2.0);
        let position2 = [
            vector[0] + angle.cos() * 4.0,
            vector[1] + angle.sin() * 4.0,
        ];
        r.push_sprite(
            position2,
            [32.0, 32.0],
            uv,
            cage_halo_color(j, global_time_wrapped_hourly),
        );
    }
    // TileDrawing.cs:8021 — `new Color(255,255,255,0) * 0.1f` → (25,25,25,0).
    let overlay = xna_scale_color([255, 255, 255, 0], 0.1);
    r.push_sprite(
        vector,
        [32.0, 32.0],
        uv,
        [
            overlay[0] as f32 / 255.0,
            overlay[1] as f32 / 255.0,
            overlay[2] as f32 / 255.0,
            0.0,
        ],
    );
}

/// The 334 weapon-rack display decode (`TileDrawing.cs:7913-7943`):
/// `frameX >= 5000` packs the item — `type = frameX % 5000 - 100`, the
/// style count is the number of 5000 chunks (`num4`, flip at `>= 3`), and
/// the prefix rides the RIGHT neighbour's `frameX`
/// (`< 25000 ? -10000 : -25000`, `:7923-7924`). `None` below the 5000 gate.
pub(crate) fn weapon_rack_decode(frame_x: i16, right_frame_x: i16) -> Option<(i32, i32, bool)> {
    if frame_x < 5000 {
        return None;
    }
    // TileDrawing.cs:7915-7922
    let mut num3 = frame_x as i32;
    let mut num4 = 0;
    let item_type = num3 % 5000 - 100;
    while num3 >= 5000 {
        num4 += 1;
        num3 -= 5000;
    }
    // TileDrawing.cs:7923-7924
    let r = right_frame_x as i32;
    let prefix = if r < 25000 { r - 10000 } else { r - 25000 };
    // TileDrawing.cs:7939-7943
    Some((item_type, prefix, num4 >= 3))
}

/// The 334 arm's 40 px display clamp (`TileDrawing.cs:7933-7938`):
/// `scale = 1`, shrunk to `40/max-dimension` when the source exceeds 40 px
/// (the `width <= height` tie picks the HEIGHT divisor), then
/// `*= item.scale`.
pub(crate) fn weapon_rack_display_scale(width: i32, height: i32, item_scale: f32) -> f32 {
    let mut num5 = 1.0f32;
    if width > 40 || height > 40 {
        num5 = if width <= height {
            40.0 / height as f32
        } else {
            40.0 / width as f32
        };
    }
    num5 * item_scale
}

/// The 334 arm's item draw (`TileDrawing.cs:7945`): anchored at
/// `(x*16 + 24, y*16 + 8) + off`, origin the INTEGER `(width/2, height/2)`,
/// scaled by [`weapon_rack_display_scale`], flipped on style counts >= 3.
/// The item-texture resolve (`netDefaults`/`Prefix`/`itemAnimations`) and
/// the `item.color` second pass (`:7946-7949`) ride the wiring step — the
/// caller passes the resolved source `uv`, its pixel `size`, the final
/// `scale`/`flip` and the light colour.
#[allow(clippy::too_many_arguments)]
pub(crate) fn push_weapon_rack_item(
    r: &mut SpriteRenderer,
    uv: [f32; 4],
    size: [f32; 2],
    screen_pos: [f32; 2],
    off: [f32; 2],
    tx: i32,
    ty: i32,
    scale: f32,
    flip: bool,
    color: [f32; 4],
) {
    let pos = [
        (tx * 16) as f32 - screen_pos[0] + 24.0 + off[0],
        (ty * 16) as f32 - screen_pos[1] + 8.0 + off[1],
    ];
    let origin = [(size[0] as i32 / 2) as f32, (size[1] as i32 / 2) as f32];
    let uv = if flip {
        [uv[2], uv[1], uv[0], uv[3]]
    } else {
        uv
    };
    // TileBatch.Draw with origin + scale: top-left = pos - origin * scale.
    r.push_sprite(
        [pos[0] - origin[0] * scale, pos[1] - origin[1] * scale],
        [size[0] * scale, size[1] * scale],
        uv,
        color,
    );
}

// ---------------------------------------------------------------------------
// Tests-as-code (written, NOT run — code-only mode)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -- XNA byte maths ------------------------------------------------------

    #[test]
    fn xna_multiply_truncates_with_shift_not_rounding() {
        // 127 * 255 = 32385, >> 8 = 126 (NOT 127 — the /255 rounding trap).
        assert_eq!(xna_multiply_rgba([127, 127, 127, 0], [255, 0, 128, 255]), [126, 0, 63, 0]);
    }

    #[test]
    fn xna_scale_truncates_toward_zero() {
        // (int)(126 * 0.5) = 63; (int)(255 * 0.1) = 25 (25.5 truncates).
        assert_eq!(xna_scale_color([126, 255, 1, 0], 0.5), [63, 127, 0, 0]);
        assert_eq!(xna_scale_color([255, 255, 255, 0], 0.1), [25, 25, 25, 0]);
    }

    #[test]
    fn cage_halo_is_the_two_step_truncation() {
        // hue 0 → hsl bytes (255, 0, 0): (127*255)>>8 = 126, (int)(126*0.5)
        // = 63; alpha forced 0. One combined float multiply would give
        // 127*0.5 = 63.5 → a different byte after quantisation.
        let c = cage_halo_color(0, 0.0);
        assert_eq!(c, [63.0 / 255.0, 0.0, 0.0, 0.0]);
    }

    // -- chest frames --------------------------------------------------------
    // W4-d58: the chest_frame_adjust/chest_anchor witnesses moved with the
    // functions into `tr_render::chest_frames::tests` (keep-one).

    // -- platform side-fills ---------------------------------------------------

    fn solid(kind: u16) -> FillNeighbor {
        FillNeighbor { active: true, kind, slope: 0, half_brick: false, invisible: false }
    }

    #[test]
    fn slope1_fills_from_198_when_the_right_below_is_solid() {
        // TileDrawing.cs:1524-1532 — stone (kind 1) below-right.
        let f = platform_side_fill(1, 0, &FillNeighbor::default(), &FillNeighbor::default(), &solid(1), false);
        assert_eq!(f, Some(PlatformSideFill { src: [198, 0, 16, 16], behind_tiles: true }));
    }

    #[test]
    fn slope1_fills_from_324_for_an_unsloped_platform() {
        // TileDrawing.cs:1528-1531 — platform kind 19, slope 0.
        let f = platform_side_fill(1, 18, &FillNeighbor::default(), &FillNeighbor::default(), &solid(19), false);
        assert_eq!(f.unwrap().src, [324, 18, 16, 16]);
        // A SLOPED platform neighbour keeps the generic 198 column.
        let mut sloped = solid(19);
        sloped.slope = 1;
        let f2 = platform_side_fill(1, 0, &FillNeighbor::default(), &FillNeighbor::default(), &sloped, false);
        assert_eq!(f2.unwrap().src[0], 198);
    }

    #[test]
    fn slope2_fills_from_162_and_306() {
        // TileDrawing.cs:1534-1542 — the mirror arm.
        let f = platform_side_fill(2, 0, &FillNeighbor::default(), &solid(1), &FillNeighbor::default(), false);
        assert_eq!(f.unwrap().src, [162, 0, 16, 16]);
        let f2 = platform_side_fill(2, 0, &FillNeighbor::default(), &solid(19), &FillNeighbor::default(), false);
        assert_eq!(f2.unwrap().src[0], 306);
    }

    #[test]
    fn the_opposite_slope_and_half_brick_neighbours_veto() {
        // slope 1 refuses a slope-2 right neighbour (:1524 `slope() != 2`).
        let mut n = solid(1);
        n.slope = 2;
        assert!(platform_side_fill(1, 0, &FillNeighbor::default(), &FillNeighbor::default(), &n, false).is_none());
        // …and a half brick.
        let mut h = solid(1);
        h.half_brick = true;
        assert!(platform_side_fill(1, 0, &FillNeighbor::default(), &FillNeighbor::default(), &h, false).is_none());
        // slope 2 refuses slope 1 (:1534).
        let mut m = solid(1);
        m.slope = 1;
        assert!(platform_side_fill(2, 0, &FillNeighbor::default(), &m, &FillNeighbor::default(), false).is_none());
    }

    #[test]
    fn a_stair_blocking_full_block_below_vetoes_the_fill() {
        // Closed tall gate (386, BlocksStairs) directly below, full block
        // (blockType 0) → the tail's last clause fails (:1524).
        let below = solid(386);
        assert!(platform_side_fill(1, 0, &below, &FillNeighbor::default(), &solid(1), false).is_none());
        // The same gate sloped (blockType != 0/5) allows the fill.
        let mut sloped_gate = solid(386);
        sloped_gate.slope = 1; // blockType 2
        assert!(platform_side_fill(1, 0, &sloped_gate, &FillNeighbor::default(), &solid(1), false).is_some());
        // A slope-4 gate (blockType 5) vetoes the slope-1 arm…
        let mut steep = solid(386);
        steep.slope = 4; // blockType 5
        assert!(platform_side_fill(1, 0, &steep, &FillNeighbor::default(), &solid(1), false).is_none());
        // …but not the slope-2 arm (its veto is blockType 4).
        assert!(platform_side_fill(2, 0, &steep, &solid(1), &FillNeighbor::default(), false).is_some());
        // A non-BlocksStairs kind below never vetoes.
        assert!(platform_side_fill(1, 0, &solid(1), &FillNeighbor::default(), &solid(1), false).is_some());
    }

    #[test]
    fn slopes_3_4_and_inactive_neighbours_draw_no_fill() {
        assert!(platform_side_fill(3, 0, &FillNeighbor::default(), &solid(1), &solid(1), false).is_none());
        assert!(platform_side_fill(4, 0, &FillNeighbor::default(), &solid(1), &solid(1), false).is_none());
        let mut inactive = solid(1);
        inactive.active = false;
        assert!(platform_side_fill(1, 0, &FillNeighbor::default(), &FillNeighbor::default(), &inactive, false).is_none());
    }

    #[test]
    fn block_type_matches_tile_cs() {
        // Tile.cs:202-214.
        assert_eq!(block_type(false, 0), 0);
        assert_eq!(block_type(true, 0), 1);
        assert_eq!(block_type(true, 3), 1); // half brick wins
        assert_eq!(block_type(false, 1), 2);
        assert_eq!(block_type(false, 4), 5);
    }

    // -- DrawSpecialTilesLegacy ----------------------------------------------

    #[test]
    fn water_cage_frame_is_the_truncating_product_mod_25() {
        // TileDrawing.cs:6528-6533: (x - fx/18)/2 * ((y - fy/18)/3) % 25.
        assert_eq!(water_animal_cage_frame(4, 6, 0, 0), 4); // 2 * 2
        assert_eq!(water_animal_cage_frame(10, 20, 18, 36), 24); // 4 * 6
        assert_eq!(water_animal_cage_frame(20, 30, 0, 0), 0); // 10 * 10 = 100 % 25
        // Truncation toward zero on negatives (C# int division).
        assert_eq!(water_animal_cage_frame(-3, 6, 0, 0), -2); // (-3/2)*2 = -1*2
    }

    #[test]
    fn weapon_rack_decode_unpacks_type_prefix_and_flip() {
        // One 5000 chunk: item 1, no flip.
        assert_eq!(weapon_rack_decode(5101, 10005), Some((1, 5, false)));
        // Two chunks: still no flip (num4 >= 3 gates it).
        assert_eq!(weapon_rack_decode(10101, 10000), Some((1, 0, false)));
        // Three chunks: flip.
        assert_eq!(weapon_rack_decode(15101, 25081), Some((1, 81, true)));
        // Below the gate: nothing.
        assert_eq!(weapon_rack_decode(4999, 10000), None);
    }

    #[test]
    fn weapon_rack_scale_clamps_to_40px() {
        // TileDrawing.cs:7933-7938.
        assert_eq!(weapon_rack_display_scale(20, 20, 1.0), 1.0);
        assert_eq!(weapon_rack_display_scale(80, 40, 1.0), 0.5); // width divisor
        assert_eq!(weapon_rack_display_scale(40, 80, 1.0), 0.5); // height divisor
        assert_eq!(weapon_rack_display_scale(60, 60, 1.0), 40.0 / 60.0); // tie → height
        assert_eq!(weapon_rack_display_scale(80, 80, 0.5), 0.25); // *= item.scale
    }
}
