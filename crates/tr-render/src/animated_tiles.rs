//! W4-d56 (animated tiles draw pass, draw-side): the per-tile light chain
//! every animated tile passes through in `TileDrawing` — the vision-changer
//! floors (`DrawAnimatedTile_AdjustForVisionChangers`,
//! `TileDrawing.cs:8481-8552`, also `DrawSingleTile` :897-963), its three
//! predicates (`IsTileDangerous` :4435-4467, `Main.IsTileSpelunkable`
//! `Main.cs:21783-21798`, `Main.IsTileBiomeSightable` `Main.cs:21800-21818`)
//! and `DrawTiles_GetLightOverride` (`TileDrawing.cs:6624-6686`).
//!
//! The torch/candle flame quads (`tr-core/flame.rs`, TILE-DRAW-2) and the
//! waterfall/lavafall fall-block rows (`tr-core/drawdata.rs` case-326 group +
//! the wired `TileAnimator`, S1-G-1f) already render; this module ports the
//! light-override half of the animated-tile draw path those passes share.
//!
//! **UNWIRED module (code-only, W4-d56).** The `render/tiles.rs::draw_tiles`
//! seam is contested by the in-flight W4-d57/d58/d59/d60 tile-draw siblings;
//! like W4-d58's phase-1 `chest_frames`, the pure decision maths lands in
//! `tr-render` first (no `tr-core` dependency — every world/engine input is
//! a parameter). Intended call sites, in `DrawSingleTile` order:
//! * [`is_tile_dangerous`] / [`is_tile_spelunkable`] / [`is_tile_biome_sightable`]
//!   + [`vision_changer_light`] — after `GetTileDrawData` and before the
//!   `flag` light test (`TileDrawing.cs:897-963`); the grass/vine deferred
//!   passes call the same chain (`DrawGrass` :8348, `DrawMultiTileVines`
//!   :8976). The dust rolls are returned as [`VisionDust`] requests in RNG
//!   order — the caller owns the `_rand` stream (the d60 particles
//!   convention); this module never rolls RNG.
//! * [`light_override`] — right after the vision floors, before the light
//!   test (`:969`), and in `DrawGrass` (:8349).
//!
//! Deviations (recorded in docs/ANALYSIS.md "The animated-tile light chain
//! (W4-d56)"):
//! * `Main.getGoodWorld` / `Main.dontStarveWorld` / `Main.remixWorld` ride
//!   parameters; the caller passes the world flags.
//! * The `TileID.Sets` tables are inlined verbatim from `TileID.cs` (the
//!   `tr-render` crate cannot read `tr-core::tiledata`); the census
//!   witnesses pin every row against the C# factory lists.
//! * Case 83/84's `WorldGen.IsAlchemyPlantHarvestable` probe rides the
//!   `herb_harvestable` parameter (the port's `growth::herbs` predicate).
//! * `Color operator*(Color, float)` truncates per channel toward zero
//!   (the `xna_scale_color` convention, W4-d58's `special_tiles.rs`); the
//!   481-483 arm's alpha restore matches the C# byte copy.

/// `TileID.Sets.Boulders` (`TileID.cs:504`):
/// `CreateBoolSet(138, 484, 664, 665, 711, 712, 713, 714, 715, 716)`.
const BOULDERS: [u16; 10] = [138, 484, 664, 665, 711, 712, 713, 714, 715, 716];

/// `TileID.Sets.TouchDamageImmediate[type] > 0` (`TileID.cs:609`):
/// `CreateIntSet(0, 32,10, 69,17, 80,6, 352,10, 655,100, 48,60, 232,80,
/// 484,25, 750,70)` — the positive rows only.
const TOUCH_DAMAGE_IMMEDIATE: [u16; 9] = [32, 48, 69, 80, 232, 352, 484, 655, 750];

/// `TileID.Sets.TouchDamageHot` (`TileID.cs:607`):
/// `CreateBoolSet(37, 58, 76, 684, 230)`.
const TOUCH_DAMAGE_HOT: [u16; 5] = [37, 58, 76, 684, 230];

/// `Main.tileSpelunker[type]` (`Main.cs:9420-9467` + `:9651`): the 49 true
/// rows, in source order (the order is immaterial to the lookup; kept for
/// the census witness).
const TILE_SPELUNKER: [u16; 49] = [
    6, 7, 8, 9, 12, 21, 467, 441, 468, 28, 107, 108, 111, 63, 64, 65, 66, 67, 68, 566, 166, 167,
    168, 169, 178, 211, 221, 222, 223, 236, 702, 37, 407, 227, 84, 83, 404, 242, 240, 246, 245,
    105, 349, 337, 531, 506, 751, 752, 639,
];

/// `TileID.Sets.CorruptBiomeSight` (`TileID.cs:578`):
/// `CreateBoolSet(23, 661, 25, 112, 163, 398, 400, 636, 24, 32)`.
const CORRUPT_BIOME_SIGHT: [u16; 10] = [23, 661, 25, 112, 163, 398, 400, 636, 24, 32];

/// `TileID.Sets.CrimsonBiomeSight` (`TileID.cs:587`):
/// `CreateBoolSet(199, 662, 203, 234, 200, 399, 401, 205, 201, 352)`.
const CRIMSON_BIOME_SIGHT: [u16; 10] = [199, 662, 203, 234, 200, 399, 401, 205, 201, 352];

/// `TileID.Sets.HallowBiomeSight` (`TileID.cs:582`):
/// `CreateBoolSet(109, 492, 117, 116, 164, 402, 403, 115, 110, 113)`.
const HALLOW_BIOME_SIGHT: [u16; 10] = [109, 492, 117, 116, 164, 402, 403, 115, 110, 113];

/// `IsTileDangerous_CheckTouchDamage` (`TileDrawing.cs:4449-4457`): the
/// immediate-touch-damage rows, minus cactus 80 outside the Don't Starve
/// world (`!Main.dontStarveWorld` clears it).
fn touch_damage(type_cache: u16, dont_starve_world: bool) -> bool {
    let flag = TOUCH_DAMAGE_IMMEDIATE.contains(&type_cache);
    if flag && type_cache == 80 && !dont_starve_world {
        return false;
    }
    flag
}

/// `IsTileDangerous_CheckHot` (`TileDrawing.cs:4459-4467`): the hot rows,
/// cleared wholesale by the Obsidian Skin / Lava Waders `fireWalk` flag.
fn touch_hot(type_cache: u16, fire_walk: bool) -> bool {
    if fire_walk {
        return false;
    }
    TOUCH_DAMAGE_HOT.contains(&type_cache)
}

/// `Minecart.IsPressurePlate` (`Minecart.cs:549-560`): a track cell whose
/// frameX is 20 or 21 (the pressure-plate track styles). The C# also
/// checks `tile.active()`; every `IsTileDangerous` call site draws an
/// active tile (`DrawSingleTile`/`DrawGrass` gate on active first), so
/// the port's callers guarantee it.
fn is_pressure_plate_track(frame_x: i16) -> bool {
    frame_x == 20 || frame_x == 21
}

/// `TileDrawing.IsTileDangerous` (`TileDrawing.cs:4435-4447`): the Dangersense
/// highlight predicate. The first flag list applies to every tile; the
/// second group only when the tile is flat (`slope == 0`) and not actuated
/// (`!inActive`), with the thin-ice 162 exception gated on `!iceSkate`.
#[allow(clippy::too_many_arguments)]
pub fn is_tile_dangerous(
    type_cache: u16,
    frame_x: i16,
    slope: u8,
    inactive: bool,
    ice_skate: bool,
    fire_walk: bool,
    good_world: bool,
    dont_starve_world: bool,
) -> bool {
    // `TileDrawing.cs:4437` — the unconditional list.
    let mut flag = type_cache == 135
        || type_cache == 137
        || BOULDERS.contains(&type_cache)
        || type_cache == 141
        || type_cache == 210
        || type_cache == 442
        || type_cache == 443
        || type_cache == 444
        || type_cache == 411
        || type_cache == 485
        || type_cache == 85
        || type_cache == 654
        || (type_cache == 314 && is_pressure_plate_track(frame_x));
    flag = flag || (good_world && type_cache == 230) || type_cache == 58;
    // `TileDrawing.cs:4438-4445` — the flat-and-active group.
    if slope == 0 && !inactive {
        flag = flag
            || touch_damage(type_cache, dont_starve_world)
            || type_cache == 483
            || type_cache == 482
            || type_cache == 481
            || type_cache == 51
            || type_cache == 229
            || touch_hot(type_cache, fire_walk);
        if !ice_skate {
            flag = flag || type_cache == 162;
        }
    }
    flag
}

/// `Main.IsTileSpelunkable` (`Main.cs:21783-21798`): the Spelunker highlight
/// predicate — the `tileSpelunker` table plus the two 185/186 frame windows
/// (the grave-marker and statue treasure rows).
pub fn is_tile_spelunkable(type_cache: u16, frame_x: i16, frame_y: i16) -> bool {
    if TILE_SPELUNKER.contains(&type_cache) {
        return true;
    }
    if type_cache == 185 && frame_y == 18 && (576..=882).contains(&frame_x) {
        return true;
    }
    if type_cache == 186 && (864..=1170).contains(&frame_x) {
        return true;
    }
    false
}

/// `Main.IsTileBiomeSightable` (`Main.cs:21800-21818`): the Biome Sight
/// highlight predicate. Returns the sight colour bytes (RGB; the C# writes
/// a `Color` whose alpha defaults to 255) or `None`. The `remixWorld`
/// exceptions: 474 counts corrupt, 195 crimson.
pub fn is_tile_biome_sightable(type_cache: u16, remix_world: bool) -> Option<[u8; 3]> {
    if CORRUPT_BIOME_SIGHT.contains(&type_cache) || (remix_world && type_cache == 474) {
        return Some([200, 100, 240]);
    }
    if CRIMSON_BIOME_SIGHT.contains(&type_cache) || (remix_world && type_cache == 195) {
        return Some([255, 100, 100]);
    }
    if HALLOW_BIOME_SIGHT.contains(&type_cache) {
        return Some([255, 160, 240]);
    }
    None
}

/// One `Dust.NewDust` request the vision floors emit
/// (`TileDrawing.cs:8497-8550`), in RNG-consumption order. The caller owns
/// the `_rand` stream and must reproduce the per-arm roll conditions
/// exactly:
/// * danger (:8497): `(_isActiveAndNotPaused & canDoDust) && Next(30) == 0`
///   — the roll runs ONLY when active AND dust-allowed;
/// * spelunker (:8516): `_isActiveAndNotPaused && ((Next(60) == 0) &
///   canDoDust)` — the bitwise `&` means the roll runs whenever active,
///   EVEN when dust is suppressed (the RNG consumption is the effect);
/// * biome (:8543): `(_isActiveAndNotPaused & canDoDust) && Next(480) == 0`
///   — like danger.
/// The hit results feed [`vision_changer_light`]; this module never rolls
/// RNG itself.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct VisionDust {
    /// The `Dust.NewDust` type (60 danger / 204 spelunker / 267 biome).
    pub dust_type: i32,
    /// `Alpha` (100 / 150 / 150).
    pub alpha: i32,
    /// `Scale` — all three arms pass 0.3f.
    pub scale: f32,
    /// The biome arm's `newColor: sightColor` override; the other arms
    /// pass `default(Color)` (transparent black, `None` here — the dust
    /// spawn's own colour pick).
    pub color: Option<[u8; 3]>,
    /// `fadeIn = 1f` — all three arms.
    pub fade_in: bool,
    /// `velocity *= 0.1f` — all three arms.
    pub velocity_scale: f32,
    /// `noLight = true` — the danger and spelunker arms.
    pub no_light: bool,
    /// `noGravity = true` — the danger and biome arms.
    pub no_gravity: bool,
    /// `noLightEmittance = true` — the biome arm only.
    pub no_light_emittance: bool,
}

/// The `_rand.Next(n)` gate denominators of the three vision arms
/// (`TileDrawing.cs:8497`/`:8516`/`:8543`) — the caller rolls these in
/// order on its own stream, then feeds the hits back through
/// [`vision_changer_light`]'s dust outputs.
pub const DANGER_DUST_GATE: i32 = 30;
pub const SPELUNKER_DUST_GATE: i32 = 60;
pub const BIOME_DUST_GATE: i32 = 480;

/// The vision floors' output: the adjusted light bytes plus the dust
/// requests whose gates hit (in C# order — danger, spelunker, biome).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct VisionLight {
    /// `tileLight` after the floors (RGBA bytes).
    pub light: [u8; 4],
    /// The dust requests, one per hit gate, in source order.
    pub dust: [Option<VisionDust>; 3],
}

/// `DrawAnimatedTile_AdjustForVisionChangers` (`TileDrawing.cs:8481-8552`),
/// the byte floors only: Dangersense lifts R to 255 / G,B to 50; Spelunker
/// lifts R to 200 / G to 170 (B untouched); Biome Sight lifts each channel
/// to the sight colour's byte. Each floor is a `max`, applied in that
/// order. `danger_hit`/`spelunker_hit`/`biome_hit` are the caller's
/// `Next(30)/Next(60)/Next(480) == 0` results, only consumed when
/// `active_and_not_paused && can_do_dust` (the C# `_isActiveAndNotPaused &
/// canDoDust` gates, :8497/:8516/:8543 — the spelunker arm's bitwise `&`
/// still ROLLS its gate whenever active even with dust suppressed; see
/// [`VisionDust`] for the exact per-arm roll conditions).
#[allow(clippy::too_many_arguments)]
pub fn vision_changer_light(
    light: [u8; 4],
    danger: bool,
    spelunkable: bool,
    sight_color: Option<[u8; 3]>,
    active_and_not_paused: bool,
    can_do_dust: bool,
    danger_hit: bool,
    spelunker_hit: bool,
    biome_hit: bool,
) -> VisionLight {
    let mut l = light;
    let mut dust = [None; 3];
    let dust_ok = active_and_not_paused && can_do_dust;
    // `TileDrawing.cs:8483-8505` — the Dangersense floor.
    if danger {
        l[0] = l[0].max(255);
        l[1] = l[1].max(50);
        l[2] = l[2].max(50);
        if dust_ok && danger_hit {
            dust[0] = Some(VisionDust {
                dust_type: 60,
                alpha: 100,
                scale: 0.3,
                color: None,
                fade_in: true,
                velocity_scale: 0.1,
                no_light: true,
                no_gravity: true,
                no_light_emittance: false,
            });
        }
    }
    // `TileDrawing.cs:8506-8523` — the Spelunker floor.
    if spelunkable {
        l[0] = l[0].max(200);
        l[1] = l[1].max(170);
        if dust_ok && spelunker_hit {
            dust[1] = Some(VisionDust {
                dust_type: 204,
                alpha: 150,
                scale: 0.3,
                color: None,
                fade_in: true,
                velocity_scale: 0.1,
                no_light: true,
                no_gravity: false,
                no_light_emittance: false,
            });
        }
    }
    // `TileDrawing.cs:8524-8551` — the Biome Sight floor.
    if let Some(sc) = sight_color {
        l[0] = l[0].max(sc[0]);
        l[1] = l[1].max(sc[1]);
        l[2] = l[2].max(sc[2]);
        if dust_ok && biome_hit {
            dust[2] = Some(VisionDust {
                dust_type: 267,
                alpha: 150,
                scale: 0.3,
                color: Some(sc),
                fade_in: true,
                velocity_scale: 0.1,
                no_light: false,
                no_gravity: true,
                no_light_emittance: true,
            });
        }
    }
    VisionLight { light: l, dust }
}

/// XNA `Color operator*(Color, float)` — `(int)(channel * scale)` per
/// channel, truncation toward zero (the `special_tiles.rs` convention).
fn xna_scale_color(c: [u8; 4], scale: f32) -> [u8; 4] {
    let s = |x: u8| (x as f32 * scale) as i32 as u8;
    [s(c[0]), s(c[1]), s(c[2]), s(c[3])]
}

/// `DrawTiles_GetLightOverride` (`TileDrawing.cs:6624-6686`): the per-type
/// light rewrites applied AFTER the vision floors and BEFORE the draw's
/// light test (`DrawSingleTile` :969, `DrawGrass` :8349).
///
/// Inputs: the light bytes (RGBA), the tile's frame pair, `fullbright`
/// (`tileCache.fullbrightBlock()`), `mouse_text_color`
/// (`Main.mouseTextColor`), `global_time` (`Main.GlobalTimeWrappedHourly`)
/// and `herb_harvestable` (the caller's
/// `WorldGen.IsAlchemyPlantHarvestable(tileFrameX / 18, j)` probe, only
/// read for type 83).
#[allow(clippy::too_many_arguments)]
pub fn light_override(
    kind: u16,
    frame_x: i16,
    frame_y: i16,
    fullbright: bool,
    light: [u8; 4],
    mouse_text_color: u8,
    global_time: f32,
    herb_harvestable: bool,
) -> [u8; 4] {
    // `TileDrawing.cs:6626-6629`.
    if fullbright {
        return [255, 255, 255, 255];
    }
    let mut l = light;
    let m = mouse_text_color as i32;
    match kind {
        // `:6632-6634` — the echo block and the luminite brick draw white.
        541 | 631 => return [255, 255, 255, 255],
        // `:6635-6640` — the martian-conduit platform row 48 draws white.
        19 => {
            if frame_y / 18 == 48 {
                return [255, 255, 255, 255];
            }
        }
        // `:6641-6651` — a harvestable fireblossom (style 5) glows with the
        // UI text pulse: A = mouseTextColor/2, G = B = mouseTextColor.
        83 => {
            let style = frame_x as i32 / 18;
            if herb_harvestable && style == 5 {
                l[3] = (m / 2) as u8;
                l[1] = mouse_text_color;
                l[2] = mouse_text_color;
            }
        }
        // `:6652-6666` — the grown fireblossom 84 column 6 blends the pulse
        // into G/B: `(mouseTextColor + channel*2)/3`, kept only if brighter.
        84 => {
            if frame_x as i32 / 18 == 6 {
                let b4 = ((m + l[1] as i32 * 2) / 3) as u8;
                let b5 = ((m + l[2] as i32 * 2) / 3) as u8;
                if b4 > l[1] {
                    l[1] = b4;
                }
                if b5 > l[2] {
                    l[2] = b5;
                }
            }
        }
        // `:6667-6673` — the lily 61 / pink lily 703 frameX 144 column
        // repaints every channel with `245 - mouseTextColor*1.5`. The C#
        // `(byte)` cast of the (negative for mouseTextColor >= 164) float
        // truncates toward zero to int then wraps unchecked — Rust's
        // saturating float→u8 `as` would clamp to 0 instead, so the cast
        // goes through i32 (the `xna_scale_color` convention).
        61 | 703 => {
            if frame_x == 144 {
                let v = (245.0 - mouse_text_color as f32 * 1.5) as i32 as u8;
                l = [v, v, v, v];
            }
        }
        // `:6674-6683` — the cracked dungeon bricks 481/482/483 breathe:
        // `1 + sin(GlobalTimeWrappedHourly / 1.5 * 2π) * 0.15`, RGB scaled
        // with the per-channel truncation, alpha restored.
        481 | 482 | 483 => {
            let num = 1.0
                + (global_time / 1.5 * std::f32::consts::TAU).sin() * 0.15;
            let a = l[3];
            l = xna_scale_color(l, num);
            l[3] = a;
        }
        _ => {}
    }
    l
}

#[cfg(test)]
mod tests {
    use super::*;

    /// W4-d56 census witness: the inlined `TileID.Sets.Boulders` /
    /// `TouchDamageHot` / `TouchDamageImmediate` rows match the C# factory
    /// lists (`TileID.cs:504/:607/:609`) — every listed id is dangerous on
    /// a flat active tile, and a control id outside every list is not.
    #[test]
    fn danger_tables_match_the_csharp_sets() {
        for k in BOULDERS {
            assert!(is_tile_dangerous(k, 0, 0, false, true, true, false, false), "boulder {k}");
        }
        for k in TOUCH_DAMAGE_HOT {
            assert!(is_tile_dangerous(k, 0, 0, false, true, false, false, false), "hot {k}");
        }
        for k in TOUCH_DAMAGE_IMMEDIATE {
            // Cactus 80 needs the Don't Starve world (the :4452 carve-out).
            let ds = k == 80;
            assert!(is_tile_dangerous(k, 0, 0, false, true, true, false, ds), "touch {k}");
        }
        // A plain dirt block is never dangerous.
        assert!(!is_tile_dangerous(0, 0, 0, false, false, false, false, false));
        assert!(!is_tile_dangerous(1, 0, 0, false, false, false, false, false));
    }

    /// `TileDrawing.cs:4437` — the unconditional list fires even on a
    /// sloped or actuated tile; the flat-and-active group (:4438-4445)
    /// does not. The pressure-plate track 314 keys on frameX 20/21
    /// (`Minecart.cs:549-560`).
    #[test]
    fn danger_unconditional_list_ignores_slope_and_actuation() {
        for k in [135u16, 137, 141, 210, 442, 443, 444, 411, 485, 85, 654, 58] {
            assert!(is_tile_dangerous(k, 0, 3, true, true, true, false, false), "list {k}");
        }
        // The second group (thorns 51, hot 37, thin ice 162) needs flat+active.
        for k in [51u16, 229, 481, 482, 483, 37, 162] {
            assert!(is_tile_dangerous(k, 0, 0, false, false, false, false, false), "flat {k}");
            assert!(!is_tile_dangerous(k, 0, 1, false, false, false, false, false), "sloped {k}");
            assert!(!is_tile_dangerous(k, 0, 0, true, false, false, false, false), "actuated {k}");
        }
        // The track 314: only the pressure-plate frames.
        assert!(is_tile_dangerous(314, 20, 0, false, true, true, false, false));
        assert!(is_tile_dangerous(314, 21, 0, false, true, true, false, false));
        assert!(!is_tile_dangerous(314, 0, 0, false, true, true, false, false));
        assert!(!is_tile_dangerous(314, 22, 0, false, true, true, false, false));
    }

    /// `TileDrawing.cs:4441-4444/:4459-4466/:4452-4455` + `:4437` — the
    /// three accessory/world gates: iceSkate clears thin ice 162, fireWalk
    /// clears the whole hot set, getGoodWorld adds 230, dontStarveWorld
    /// keeps cactus 80 dangerous.
    #[test]
    fn danger_accessory_and_world_gates() {
        assert!(!is_tile_dangerous(162, 0, 0, false, true, false, false, false));
        assert!(is_tile_dangerous(162, 0, 0, false, false, false, false, false));
        for k in TOUCH_DAMAGE_HOT {
            // Meteorite 58 is ALSO in the unconditional list (:4437) —
            // fireWalk clears only the hot-set contribution.
            if k == 58 {
                assert!(is_tile_dangerous(k, 0, 0, false, true, true, false, false), "58 unconditional");
                continue;
            }
            assert!(!is_tile_dangerous(k, 0, 0, false, true, true, false, false), "fireWalk {k}");
        }
        assert!(is_tile_dangerous(230, 0, 0, false, true, true, true, false));
        assert!(!is_tile_dangerous(230, 0, 0, false, true, true, false, false));
        assert!(is_tile_dangerous(80, 0, 0, false, true, true, false, true));
        assert!(!is_tile_dangerous(80, 0, 0, false, true, true, false, false));
    }

    /// `Main.cs:9420-9467` + `:9651` — the 49-row tileSpelunker census:
    /// every row highlights, a control id does not.
    #[test]
    fn spelunker_table_census() {
        assert_eq!(TILE_SPELUNKER.len(), 49);
        for k in TILE_SPELUNKER {
            assert!(is_tile_spelunkable(k, 0, 0), "spelunker {k}");
        }
        assert!(!is_tile_spelunkable(0, 0, 0));
        assert!(!is_tile_spelunkable(1, 0, 0));
        assert!(!is_tile_spelunkable(185, 0, 0));
    }

    /// `Main.cs:21789-21796` — the 185/186 frame windows are inclusive on
    /// both ends; 185 also needs frameY == 18, 186 does not read frameY.
    #[test]
    fn spelunker_frame_windows_are_inclusive() {
        assert!(is_tile_spelunkable(185, 576, 18));
        assert!(is_tile_spelunkable(185, 882, 18));
        assert!(!is_tile_spelunkable(185, 575, 18));
        assert!(!is_tile_spelunkable(185, 883, 18));
        assert!(!is_tile_spelunkable(185, 600, 0));
        assert!(is_tile_spelunkable(186, 864, 999));
        assert!(is_tile_spelunkable(186, 1170, 0));
        assert!(!is_tile_spelunkable(186, 863, 0));
        assert!(!is_tile_spelunkable(186, 1171, 0));
    }

    /// `Main.cs:21800-21818` — the three biome colours verbatim, the
    /// corrupt-before-crimson-before-hallow check order, and the remix
    /// exceptions (474 corrupt, 195 crimson) gated on remixWorld.
    #[test]
    fn biome_sight_colours_and_remix_exceptions() {
        for k in CORRUPT_BIOME_SIGHT {
            assert_eq!(is_tile_biome_sightable(k, false), Some([200, 100, 240]), "corrupt {k}");
        }
        for k in CRIMSON_BIOME_SIGHT {
            assert_eq!(is_tile_biome_sightable(k, false), Some([255, 100, 100]), "crimson {k}");
        }
        for k in HALLOW_BIOME_SIGHT {
            assert_eq!(is_tile_biome_sightable(k, false), Some([255, 160, 240]), "hallow {k}");
        }
        assert_eq!(is_tile_biome_sightable(0, false), None);
        assert_eq!(is_tile_biome_sightable(474, false), None);
        assert_eq!(is_tile_biome_sightable(474, true), Some([200, 100, 240]));
        assert_eq!(is_tile_biome_sightable(195, false), None);
        assert_eq!(is_tile_biome_sightable(195, true), Some([255, 100, 100]));
    }

    /// `TileDrawing.cs:8483-8551` — the floors are per-channel maxima
    /// applied in danger → spelunker → biome order; a channel already
    /// above the floor keeps its value. The combined walk pins the order:
    /// spelunker's G 170 lifts danger's G 50, and the crimson sight's B
    /// 100 lifts the untouched B.
    #[test]
    fn vision_floors_are_ordered_maxima() {
        let out = vision_changer_light(
            [0, 0, 0, 255], true, true, Some([255, 100, 100]), false, false, false, false, false,
        );
        assert_eq!(out.light, [255, 170, 100, 255]);
        // Already-bright channels are never dimmed.
        let out = vision_changer_light(
            [255, 255, 255, 128], true, true, Some([200, 100, 240]), false, false, false, false, false,
        );
        assert_eq!(out.light, [255, 255, 255, 128]);
        // Each floor alone.
        let d = vision_changer_light([10, 10, 10, 255], true, false, None, false, false, false, false, false);
        assert_eq!(d.light, [255, 50, 50, 255]);
        let s = vision_changer_light([10, 10, 10, 255], false, true, None, false, false, false, false, false);
        assert_eq!(s.light, [200, 170, 10, 255]);
        let b = vision_changer_light([10, 10, 10, 255], false, false, Some([255, 160, 240]), false, false, false, false, false);
        assert_eq!(b.light, [255, 160, 240, 255]);
        // No predicate: the light passes through untouched.
        let n = vision_changer_light([10, 20, 30, 40], false, false, None, true, true, true, true, true);
        assert_eq!(n.light, [10, 20, 30, 40]);
        assert_eq!(n.dust, [None; 3]);
    }

    /// `TileDrawing.cs:8497/:8516/:8543` — the dust requests carry the
    /// per-arm NewDust arguments and flags verbatim, and only emit when
    /// the gate hit AND `_isActiveAndNotPaused & canDoDust`.
    #[test]
    fn vision_dust_requests_match_the_three_arms() {
        let out = vision_changer_light(
            [0, 0, 0, 255], true, true, Some([200, 100, 240]), true, true, true, true, true,
        );
        let d = out.dust[0].expect("danger dust");
        assert_eq!((d.dust_type, d.alpha, d.scale), (60, 100, 0.3));
        assert!(d.fade_in && d.no_light && d.no_gravity && !d.no_light_emittance);
        assert_eq!(d.color, None);
        let s = out.dust[1].expect("spelunker dust");
        assert_eq!((s.dust_type, s.alpha, s.scale), (204, 150, 0.3));
        assert!(s.fade_in && s.no_light && !s.no_gravity && !s.no_light_emittance);
        let b = out.dust[2].expect("biome dust");
        assert_eq!((b.dust_type, b.alpha, b.scale), (267, 150, 0.3));
        assert!(b.fade_in && !b.no_light && b.no_gravity && b.no_light_emittance);
        assert_eq!(b.color, Some([200, 100, 240]));
        // Paused or dust-suppressed: the floors still apply, no requests.
        for (active, can) in [(false, true), (true, false), (false, false)] {
            let out = vision_changer_light(
                [0, 0, 0, 255], true, true, Some([200, 100, 240]), active, can, true, true, true,
            );
            assert_eq!(out.light, [255, 170, 240, 255]);
            assert_eq!(out.dust, [None; 3]);
        }
        // Gate misses: no requests even when active.
        let out = vision_changer_light(
            [0, 0, 0, 255], true, true, Some([200, 100, 240]), true, true, false, false, false,
        );
        assert_eq!(out.dust, [None; 3]);
        // The gate denominators are the C# constants.
        assert_eq!((DANGER_DUST_GATE, SPELUNKER_DUST_GATE, BIOME_DUST_GATE), (30, 60, 480));
    }

    /// `TileDrawing.cs:6626-6640` — fullbrightBlock wins over every case;
    /// 541/631 draw white; the platform 19 only on row 48 (frameY/18).
    #[test]
    fn light_override_white_arms() {
        let dim = [10, 20, 30, 40];
        assert_eq!(light_override(0, 0, 0, true, dim, 225, 0.0, false), [255; 4]);
        assert_eq!(light_override(541, 0, 0, false, dim, 225, 0.0, false), [255; 4]);
        assert_eq!(light_override(631, 0, 0, false, dim, 225, 0.0, false), [255; 4]);
        assert_eq!(light_override(19, 0, 48 * 18, false, dim, 225, 0.0, false), [255; 4]);
        assert_eq!(light_override(19, 0, 47 * 18, false, dim, 225, 0.0, false), dim);
        // An unlisted kind passes through.
        assert_eq!(light_override(1, 0, 0, false, dim, 225, 0.0, false), dim);
    }

    /// `TileDrawing.cs:6641-6666` — the fireblossom pair: 83 needs BOTH
    /// the harvestable probe and style 5 (A = mouse/2, G = B = mouse, R
    /// untouched); 84 column 6 blends `(mouse + ch*2)/3` into G/B and
    /// keeps the channel when the blend is not brighter.
    #[test]
    fn light_override_fireblossom_arms() {
        let l = [10, 20, 30, 40];
        // 83: style 5 = frameX 90..107.
        assert_eq!(light_override(83, 90, 0, false, l, 225, 0.0, true), [10, 225, 225, 112]);
        assert_eq!(light_override(83, 90, 0, false, l, 225, 0.0, false), l);
        assert_eq!(light_override(83, 72, 0, false, l, 225, 0.0, true), l);
        // 84: column 6 = frameX 108..125. (225 + 20*2)/3 = 88 > 20;
        // (225 + 30*2)/3 = 95 > 30.
        assert_eq!(light_override(84, 108, 0, false, l, 225, 0.0, false), [10, 88, 95, 40]);
        // A channel already above the blend keeps its value.
        let bright = [10, 250, 250, 40];
        assert_eq!(light_override(84, 108, 0, false, bright, 225, 0.0, false), bright);
        assert_eq!(light_override(84, 90, 0, false, l, 225, 0.0, false), l);
    }

    /// `TileDrawing.cs:6667-6673` — the lily column repaints all four
    /// channels with the unchecked byte cast of `245 - mouse*1.5`: the
    /// negative float truncates toward zero to int then wraps (225 →
    /// -92.5 → -92 → 164), it does NOT clamp to 0.
    #[test]
    fn light_override_lily_wraps_the_negative_byte_cast() {
        for k in [61u16, 703] {
            assert_eq!(light_override(k, 144, 0, false, [10, 20, 30, 40], 225, 0.0, false), [164; 4]);
            // mouse 163: 245 - 244.5 = 0.5 → 0.
            assert_eq!(light_override(k, 144, 0, false, [10, 20, 30, 40], 163, 0.0, false), [0; 4]);
            // mouse 164: 245 - 246 = -1 → 255.
            assert_eq!(light_override(k, 144, 0, false, [10, 20, 30, 40], 164, 0.0, false), [255; 4]);
            // Other columns pass through.
            assert_eq!(light_override(k, 126, 0, false, [10, 20, 30, 40], 225, 0.0, false), [10, 20, 30, 40]);
        }
    }

    /// `TileDrawing.cs:6674-6683` — the cracked-brick breathe scales RGB
    /// by `1 + sin(t/1.5*2π)*0.15` with per-channel truncation and
    /// restores alpha; at t=0 the factor is exactly 1 (identity).
    #[test]
    fn light_override_cracked_bricks_breathe_and_restore_alpha() {
        for k in [481u16, 482, 483] {
            // t = 0.375 → sin(π/2) = 1 → factor ≈1.15: the f32 products
            // round to 115.0 / 230.0 / 287.5 before the byte truncation
            // (100→115, 200→230, 250→287).
            assert_eq!(light_override(k, 0, 0, false, [100, 200, 250, 77], 225, 0.375, false), [115, 230, 287, 77]);
            // t = 1.125 → sin(3π/2) = -1 → factor 0.85: 100→85, 200→170, 250→212.
            assert_eq!(light_override(k, 0, 0, false, [100, 200, 250, 9], 225, 1.125, false), [85, 170, 212, 9]);
            // t = 0 → factor 1 → identity.
            assert_eq!(light_override(k, 0, 0, false, [10, 20, 30, 77], 225, 0.0, false), [10, 20, 30, 77]);
        }
    }

    /// W4-d56 frame witness (draw-chain order, `DrawSingleTile`
    /// `TileDrawing.cs:897-969`): the vision floors run FIRST and
    /// `DrawTiles_GetLightOverride` consumes their OUTPUT — a cracked
    /// brick 481 under a Spelunker floor gets R200/G170 and then the
    /// breathe scale on the floored bytes (200*1.15→230, 170*1.15→195),
    /// not on the raw light. fullbrightBlock still wins (:6626).
    #[test]
    fn the_light_chain_floors_then_overrides() {
        let raw = [0u8, 0, 0, 255];
        // 481 is NOT in tileSpelunker — the floor comes from the caller's
        // predicate on a different cell; here the floor is fed directly.
        assert!(!is_tile_spelunkable(481, 0, 0));
        let floored = vision_changer_light(raw, false, true, None, false, false, false, false, false).light;
        assert_eq!(floored, [200, 170, 0, 255]);
        let out = light_override(481, 0, 0, false, floored, 225, 0.375, false);
        assert_eq!(out, [230, 195, 0, 255]);
        // The reverse order (override first, floors second) would read
        // [200, 170, 0, 255] — the regression pin.
        let wrong = vision_changer_light(
            light_override(481, 0, 0, false, raw, 225, 0.375, false),
            false, true, None, false, false, false, false, false,
        ).light;
        assert_ne!(out, wrong);
        // fullbright wins over any floor result.
        assert_eq!(light_override(481, 0, 0, true, floored, 225, 0.375, false), [255; 4]);
    }

    /// W4-d56 frame witness (the animated-tile families share the chain):
    /// a torch 4 / candle 33 / waterfall 326 / lavafall 327 cell with no
    /// vision buff and no override case passes the light through both
    /// stages byte-for-byte — the flame/fall-block draw paths
    /// (`flame.rs`, drawdata case-326 group) see the same light the
    /// pre-W4-d56 pass fed them.
    #[test]
    fn the_animated_families_pass_the_chain_unchanged_without_buffs() {
        for k in [4u16, 33, 42, 49, 326, 327, 328, 329, 345, 351, 421, 422, 458, 459, 708] {
            let raw = [123, 45, 67, 255];
            let danger = is_tile_dangerous(k, 0, 0, false, false, false, false, false);
            let sp = is_tile_spelunkable(k, 0, 0);
            let sight = is_tile_biome_sightable(k, false);
            assert!(!danger && !sp && sight.is_none(), "family {k}");
            let v = vision_changer_light(raw, danger, sp, sight, true, true, false, false, false);
            assert_eq!(v.light, raw, "family {k} floors");
            assert_eq!(light_override(k, 0, 0, false, v.light, 225, 0.375, false), raw, "family {k} override");
        }
    }
}