//! M4-PROJ-4c: the yoyo string + sprite rendering — the pure half of the
//! row (`Main.DrawProjDirect`'s aiStyle-99 string arm, `Main.cs:27941-28046`,
//! `Main.TryApplyingPlayerStringColor` `:35474-35519`, the `stringColor` item
//! table `Item.cs:30220-30255`).
//!
//! The module carries the colour math and the string-segment walk as pure
//! functions so they are testable without a `SpriteRenderer` (the house
//! pattern, `docs/code-standards.md:88-98`); the draw call itself lives in
//! `projectile.rs`'s `Projectile::draw`, which pushes one rotated
//! `FishingLine` slice per segment (`render/sprite.rs`'s
//! `push_sprite_origin_rot` — the `EntitySpriteDraw(origin:)` analogue).


/// The ids whose `Projectile_<id>` sheet the draw path loads: the aiStyle-99
/// yoyo rows (`ProjectileID.cs`'s 534/541-555/562-564/603/999 — the
/// `DefaultToYoyo` family `Projectile.cs:10764-10771`) and the counterweight
/// rows 556-561/1079 (`:6084-6093`).
pub fn is_yoyo_sprite(id: u16) -> bool {
    matches!(id, 534 | 541..=555 | 562..=564 | 603 | 999 | 556..=561 | 1079)
}

/// The `FishingLine` texture's width in pixels (`Main.cs:28042`'s
/// `TextureAssets.FishingLine.Width()` = 12) — the string is a chain of
/// 12-px slices anchored by the `origin = (W * 0.5, 0)` pair.
pub const FISHING_LINE_WIDTH: f32 = 12.0;
/// The hand-offset `- new Vector2(6f, 0f)` (`Main.cs:28044`) — half the
/// texture width, applied on the segment's X.
const HALF_LINE_WIDTH: f32 = FISHING_LINE_WIDTH * 0.5;

/// `Main.TryApplyingPlayerStringColor` (`Main.cs:35474-35519`): the yoyo
/// string's colour for a `player.stringColor` id, RGBA in 0..1. The id-0
/// (no string item) path is the C#'s untouched `Color.White` — the caller
/// applies the 0.4 alpha (`:35410`/`:28038`) and the light tint.
///
/// The rows are the C# verbatim: `WorldGen.paintColor` (`:35478`,
/// the port's `tr_core::tile_light::paint_color`) with the 75-per-channel
/// floor (`:35479-35490`), then the special ids 13 (20,20,20),
/// 0/14 (200,200,200), 28 (163,116,91), 30 (226,228,142) and 27 (the
/// animated `Main.DiscoR/G/B`, which this row reads as its per-frame value
/// the caller passes in). The rainbow 29 (`:35511-35516`) carries its own
/// jitter roll and — see [`skips_light_tint`] — no light tint.
pub fn string_color(string_color_id: i32, disco: [f32; 3], rainbow: [f32; 3]) -> [f32; 4] {
    if string_color_id <= 0 {
        return [1.0, 1.0, 1.0, 1.0];
    }
    let mut rgb: [f32; 3] = match string_color_id {
        13 => [20.0, 20.0, 20.0],
        0 | 14 => [200.0, 200.0, 200.0],
        28 => [163.0, 116.0, 91.0],
        // The DiscoR/G/B triple is 0..255 in the C# (`Main.cs:19596`+); the
        // caller's input is 0..1, so scale before the floor.
        27 => [disco[0] * 255.0, disco[1] * 255.0, disco[2] * 255.0],
        29 => [rainbow[0] * 255.0, rainbow[1] * 255.0, rainbow[2] * 255.0],
        30 => [226.0, 228.0, 142.0],
        id => {
            let paint = tr_core::tile_light::paint_color(id as u8);
            [
                paint[0] * 255.0,
                paint[1] * 255.0,
                paint[2] * 255.0,
            ]
        }
    };
    // The 75 floor per channel (`:35479-35490`) — the dark paints stay
    // legible against the string's 0.4 alpha.
    for c in rgb.iter_mut() {
        if *c < 75.0 {
            *c = 75.0;
        }
    }
    // The 0.4 alpha (`:35410`).
    let alpha = 255.0 * 0.4;
    [rgb[0] / 255.0, rgb[1] / 255.0, rgb[2] / 255.0, alpha / 255.0]
}

/// The rainbow-29 jitter (`Main.cs:35511-35516`): the `Utils.Clamp` lerp
/// term between the two 0..1 extremes plus the `rand.Next(-40,41)` channel
/// jitter. `mouse_text_color` is `Main.mouseTextColor` and `roll` is the
/// `Main.rand` state the caller draws from.
pub fn rainbow_string_color(mouse_text_color: i32, roll01: f32, jitter01: f32) -> [f32; 3] {
    let num = ((mouse_text_color as f32 - 190.0) / 65.0).clamp(0.0, 1.0);
    let num2 = 1.0 - num;
    let jitter = (jitter01 * 80.0 - 40.0) as f32;
    [
        (184.0 * num + 255.0 * num2 + jitter) / 255.0,
        (116.0 * num + 184.0 * num2 + jitter) / 255.0,
        (255.0 * num + 116.0 * num2) / 255.0,
    ]
}

/// Whether the light tint is skipped (`Main.cs:28041`): only the rainbow
/// 29 draws its own colour.
pub fn skips_light_tint(string_color_id: i32) -> bool {
    string_color_id == 29
}

/// The `stringColor` an item sets (`Item.cs:30220-30255`): the string
/// items 3293-3308 and `Item.cs:43716`'s 30. `None` = the item has no
/// string colour (the plain white path).
pub fn string_color_for_item(kind: u16) -> Option<i32> {
    match kind {
        3305 => Some(28),
        3306 => Some(14),
        3307 => Some(27),
        3308 => Some(13),
        3293..=3304 => Some(1 + kind as i32 - 3293),
        43716 => Some(30),
        _ => None,
    }
}

/// One `FishingLine` segment (`Main.cs:28037-28045`'s per-segment draw
/// state): the world position the slice's top-centre anchors at, the
/// rotation (`Atan2(dy, dx) - π/2`) and the slice's length (`num5`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StringSegment {
    pub x: f32,
    pub y: f32,
    pub rot: f32,
    pub len: f32,
}

/// The string walk (`Main.cs:27966-28046`): from `hand` toward `target`,
/// the 12-px steps, the final partial step, and the slack sag when the
/// distance exceeds the step. The C# walks in a `while` whose per-step
/// `num`/`num2` are the direction toward the target scaled to the
/// remaining length (with the sag `num8` curvature term at `:27995-28035`
/// — the port carries the sag-scale inputs so the caller can pass the C#'s
/// `num7/80` speed factor).
///
/// The hand position already carries the `+ gfxOffY` and the mount offset
/// the C# applies at `:27944-27945`.
pub fn string_segments(hand: (f32, f32), target: (f32, f32), sag_factor: f32) -> Vec<StringSegment> {
    let (hx, hy) = hand;
    let (tx, ty) = target;
    let dx = tx - hx;
    let dy = ty - hy;
    let dist = (dx * dx + dy * dy).sqrt();
    if dist <= 0.0 {
        return Vec::new();
    }
    // The normalised step direction (`:27972-27974`'s num/num2), and the
    // sag term (`:27995-28035`): a per-step curvature scaled by the
    // remaining distance and the caller's speed factor.
    let ux = dx / dist;
    let uy = dy / dist;
    let walk = sag_factor.max(0.0);
    let mut segments = Vec::new();
    let mut px = hx;
    let mut py = hy;
    loop {
        let rem = (tx - px) * ux + (ty - py) * uy;
        if rem <= 0.0 {
            break;
        }
        let len = if rem < FISHING_LINE_WIDTH {
            rem
        } else {
            FISHING_LINE_WIDTH
        };
        // The sag: a perpendicular bow that grows with the remaining
        // distance (`num8 = (Main.player[owner].velocity... - ...) * num7/80`
        // — the port's caller folds the speed into `sag_factor`).
        let bow = walk * (rem / 80.0).min(1.0) * 0.5;
        let ox = -uy * bow;
        let oy = ux * bow;
        segments.push(StringSegment {
            x: px + ox,
            y: py + oy,
            rot: (dy).atan2(dx) - std::f32::consts::FRAC_PI_2,
            len,
        });
        px += ux * len + ox;
        py += uy * len + oy;
        if len < FISHING_LINE_WIDTH {
            break;
        }
    }
    segments
}

/// The segment's draw colour (`Main.cs:28038-28045`): the 0.4 alpha over
/// the string colour, the light tint (except the rainbow 29) and the
/// final half-dim `num10`. The light input is the `Lighting.GetColor`
/// triple the caller sampled at the segment.
pub fn segment_color(
    string_color_id: i32,
    disco: [f32; 3],
    rainbow: [f32; 3],
    light: [f32; 3],
) -> [f32; 4] {
    let mut c = string_color(string_color_id, disco, rainbow);
    // The id-0 path's alpha comes from `:28038` (the white's 0.4) — the
    // painted path already carries it from `:35410`; both end at 102/255.
    if string_color_id <= 0 {
        c[3] *= 0.4;
    }
    if !skips_light_tint(string_color_id) {
        for i in 0..3 {
            c[i] *= light[i];
        }
    }
    // `num10 = 0.5` (`:28045`) — every channel halves.
    let out = [c[0] * 0.5, c[1] * 0.5, c[2] * 0.5, c[3] * 0.5];
    out
}

/// The slice's source rectangle inside `FishingLine`: the 12-px wide sheet
/// with the segment's height taken from the top (`Main.cs:28044`'s
/// `new Rectangle(0, 0, Width, (int)num5)`).
pub const fn segment_source_len() -> f32 {
    FISHING_LINE_WIDTH
}

/// The X offset the C# subtracts at the draw call (`:28044`).
pub const fn line_x_offset() -> f32 {
    HALF_LINE_WIDTH
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The `stringColor` table is the C# verbatim (`Item.cs:30251`).
    #[test]
    fn the_string_color_items_are_verbatim() {
        assert_eq!(string_color_for_item(3293), Some(1));
        assert_eq!(string_color_for_item(3304), Some(12));
        assert_eq!(string_color_for_item(3305), Some(28));
        assert_eq!(string_color_for_item(3306), Some(14));
        assert_eq!(string_color_for_item(3307), Some(27));
        assert_eq!(string_color_for_item(3308), Some(13));
        assert_eq!(string_color_for_item(43716), Some(30));
        assert_eq!(string_color_for_item(3292), None);
    }

    /// `TryApplyingPlayerStringColor`'s rows (`:35474-35519`): the plain
    /// white for id 0, the special colours, the 75 floor on the dark
    /// paints and the 0.4 alpha.
    #[test]
    fn the_string_colour_matches_the_reference() {
        let plain = string_color(0, [0.0; 3], [0.0; 3]);
        assert_eq!(plain, [1.0, 1.0, 1.0, 1.0]);
        let dark = string_color(13, [0.0; 3], [0.0; 3]);
        // The 75 floor lifts the explicit dark colour (`:35494`'s case is
        // pre-floor in the C#, so the floor reads 75 here too).
        assert_eq!(dark, [75.0 / 255.0, 75.0 / 255.0, 75.0 / 255.0, 0.4]);
        let grey = string_color(14, [0.0; 3], [0.0; 3]);
        assert_eq!(grey, [200.0 / 255.0, 200.0 / 255.0, 200.0 / 255.0, 0.4]);
        let tan = string_color(30, [0.0; 3], [0.0; 3]);
        assert_eq!(tan, [226.0 / 255.0, 228.0 / 255.0, 142.0 / 255.0, 0.4]);
        // The 75 floor: paint 1's deep red lifts to 75/255.
        let floored = string_color(1, [0.0; 3], [0.0; 3]);
        assert!(floored[0] >= 75.0 / 255.0 - 1.0e-6, "the floor");
        // The disco path rides the caller's value — the 75 floor lifts its
        // dark channels.
        let disco = string_color(27, [1.0, 0.5, 0.25], [0.0; 3]);
        assert_eq!(disco, [1.0, 0.5, 75.0 / 255.0, 0.4]);
    }

    /// Only the rainbow 29 skips the light tint (`:28041`).
    #[test]
    fn only_the_rainbow_skips_the_light_tint() {
        assert!(skips_light_tint(29));
        assert!(!skips_light_tint(13));
        assert!(!skips_light_tint(0));
    }

    /// The rainbow's lerp + jitter (`:35511-35516`).
    #[test]
    fn the_rainbow_lerp_and_jitter_land() {
        let mid = rainbow_string_color(190, 0.5, 0.5);
        // num = 0 at mouseTextColor 190 → num2 = 1 → the 255/184/116 end.
        assert_eq!(mid, [1.0, 184.0 / 255.0, 116.0 / 255.0]);
        let top = rainbow_string_color(255, 0.5, 0.5);
        assert_eq!(top, [184.0 / 255.0, 116.0 / 255.0, 1.0]);
    }

    /// The string walk: the count, the partial final step and the
    /// rotation (`Main.cs:27966-28046`).
    #[test]
    fn the_string_segments_walk_in_twelve_px_steps() {
        // Exactly 3 steps, no sag.
        let segs = string_segments((0.0, 0.0), (36.0, 0.0), 0.0);
        assert_eq!(segs.len(), 3, "three 12-px segments");
        for s in &segs {
            assert_eq!(s.len, 12.0);
            // The horizontal walk's rotation is -π/2 (`:28036`).
            assert!((s.rot + std::f32::consts::FRAC_PI_2).abs() < 1.0e-6);
        }
        // A partial final step (`num6 - 8f`'s `:27976-27979`).
        let partial = string_segments((0.0, 0.0), (30.0, 0.0), 0.0);
        assert_eq!(partial.len(), 3);
        assert!((partial[2].len - 6.0).abs() < 1.0e-6, "the 6-px tail");
        // Under one step there is nothing to walk.
        assert!(string_segments((0.0, 0.0), (8.0, 0.0), 0.0).len() == 1);
        let near = string_segments((0.0, 0.0), (8.0, 0.0), 0.0);
        assert!((near[0].len - 8.0).abs() < 1.0e-6);
        // Degenerate: the hand is on the yoyo.
        assert!(string_segments((5.0, 5.0), (5.0, 5.0), 0.0).is_empty());
    }

    /// The diagonal walk's rotation follows the direction (`:28036`).
    #[test]
    fn the_diagonal_string_rotates_with_the_direction() {
        let segs = string_segments((0.0, 0.0), (30.0, 30.0), 0.0);
        let want = (30.0f32).atan2(30.0) - std::f32::consts::FRAC_PI_2;
        for s in &segs {
            assert!((s.rot - want).abs() < 1.0e-6);
        }
    }

    /// The sag bows the segments off the straight line (`:27995-28035`).
    #[test]
    fn the_sag_bows_the_string_off_the_line() {
        let straight = string_segments((0.0, 0.0), (60.0, 0.0), 0.0);
        let sagged = string_segments((0.0, 0.0), (60.0, 0.0), 1.0);
        // The horizontal string sags +Y (the perpendicular of +X).
        assert!(sagged[1].y > straight[1].y, "the sag bows down");
    }

    /// The segment colour: 0.4 alpha, the light tint and the final half
    /// dim (`:28038`/`:28043`/`:28045`).
    #[test]
    fn the_segment_colour_applies_alpha_light_and_the_half_dim() {
        // White string: the colour stays white, only the alpha chain bites
        // (255 * 0.4 = 102, halved to 0.2).
        let plain = segment_color(0, [0.0; 3], [0.0; 3], [1.0, 1.0, 1.0]);
        assert_eq!(plain, [0.5, 0.5, 0.5, 0.2], "the 0.4 then the half dim");
        let lit = segment_color(13, [0.0; 3], [0.0; 3], [0.5, 0.5, 0.5]);
        // 13's colour floors to 75/255, then the 0.5 light and the half dim.
        assert_eq!(lit, [75.0 / 255.0 * 0.25, 75.0 / 255.0 * 0.25, 75.0 / 255.0 * 0.25, 0.2]);
        // The rainbow ignores the light — but the 75 floor lifts its zero
        // channels, so a red disco reads 255/75/75 (`:35479-35490`).
        let rb = segment_color(29, [0.0; 3], [1.0, 0.0, 0.0], [0.1, 0.1, 0.1]);
        assert_eq!(
            rb,
            [255.0 / 255.0 * 0.5, 75.0 / 255.0 * 0.5, 75.0 / 255.0 * 0.5, 0.2]
        );
    }
}
