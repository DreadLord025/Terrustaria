//! The cyan-theme colour maths behind the modern crafting UI: a faithful port of
//! `Main.rgbToHsl` / `Main.hslToRgb` / `Main.hue2rgb` (`Main.cs:47885-47980`) and
//! `Utils.ShiftHue` / `Utils.ShiftBlueToCyanTheme` (`Utils.cs:412-426`).
//!
//! The original mixes `float` (`rgbToHsl`) with `double` (`hslToRgb` / `hue2rgb`),
//! uses `Math.Round` (**banker's rounding**, ties-to-even) for the output bytes and
//! XNA `Color * float` **truncation** for the multipliers. Both are ported exactly:
//! a rounding slip shifts every themed colour by one byte.

/// `Main.rgbToHsl` (`Main.cs:47944-47980`): `(hue, saturation, lightness)`, all 0..1.
///
/// The original computes in `float`; the arithmetic is kept in `f32` here and only
/// widened to `f64` on return (the `hslToRgb` signature's type).
pub fn rgb_to_hsl(rgb: [u8; 4]) -> (f64, f64, f64) {
    // `float num = (int)newColor.R; num /= 255f;` (`Main.cs:47946-47951`).
    let mut num = rgb[0] as f32;
    let mut num2 = rgb[1] as f32;
    let mut num3 = rgb[2] as f32;
    num /= 255.0;
    num2 /= 255.0;
    num3 /= 255.0;
    // `num4 = Math.Max(Math.Max(num, num2), num3)` (`:47952`).
    let num4 = num.max(num2).max(num3);
    // `num5 = Math.Min(Math.Min(num, num2), num3)` (`:47953`).
    let num5 = num.min(num2).min(num3);
    let mut num6 = 0.0f32; // hue
    let num7 = (num4 + num5) / 2.0; // lightness (`:47955`)
    let y; // saturation
    if num4 == num5 {
        // `num6 = (y = 0f);` (`Main.cs:47959`).
        num6 = 0.0;
        y = 0.0;
    } else {
        let num8 = num4 - num5;
        y = if num7 > 0.5 {
            num8 / (2.0 - num4 - num5) // `:47964`
        } else {
            num8 / (num4 + num5)
        };
        if num4 == num {
            num6 = (num2 - num3) / num8 + if num2 < num3 { 6.0 } else { 0.0 }; // `:47967`
        }
        if num4 == num2 {
            num6 = (num3 - num) / num8 + 2.0; // `:47971`
        }
        if num4 == num3 {
            num6 = (num - num2) / num8 + 4.0; // `:47975`
        }
        num6 /= 6.0; // `:47977`
    }
    (num6 as f64, y as f64, num7 as f64)
}

/// `Main.hue2rgb` (`Main.cs:47919-47942`), in `double` as the original does.
pub fn hue2rgb(c: f64, t1: f64, t2: f64) -> f64 {
    let mut c = c;
    if c < 0.0 {
        c += 1.0;
    }
    if c > 1.0 {
        c -= 1.0;
    }
    if 6.0 * c < 1.0 {
        return t1 + (t2 - t1) * 6.0 * c; // `:47931`
    }
    if 2.0 * c < 1.0 {
        return t2; // `:47935`
    }
    if 3.0 * c < 2.0 {
        return t1 + (t2 - t1) * (2.0 / 3.0 - c) * 6.0; // `:47939`
    }
    t1
}

/// `Main.hslToRgb` (`Main.cs:47890-47917`) + `Main.hue2rgb` (`:47919-47942`).
///
/// `Math.Round` returns a `double` and is **banker's rounding**, so the output bytes
/// use `f64::round_ties_even`. The `Saturation == 0f` test is on the `float` the
/// original receives, so the parameters are narrowed to `f32` for it and for the
/// `Luminosity + Saturation - Luminosity * Saturation` sub-expression, which the C#
/// casts to `double` **after** evaluating in `float` (`:47904`).
pub fn hsl_to_rgb(h: f64, s: f64, l: f64, a: u8) -> [u8; 4] {
    let hue = h as f32;
    let sat = s as f32;
    let lum = l as f32;
    if sat == 0.0 {
        // `r = (byte)Math.Round((double)Luminosity * 255.0);` (`Main.cs:47897-47899`).
        let v = (lum as f64 * 255.0).round_ties_even() as u8;
        return [v, v, v, a];
    }
    let num = hue as f64; // `double num = Hue;` (`:47903`)
    let num2 = if !(lum < 0.5) {
        // `(double)(Luminosity + Saturation - Luminosity * Saturation)` (`:47904`):
        // float arithmetic widened to double.
        (lum + sat - lum * sat) as f64
    } else {
        // `(double)Luminosity * (1.0 + (double)Saturation)` (`:47904`).
        lum as f64 * (1.0 + sat as f64)
    };
    let t = 2.0 * lum as f64 - num2; // `:47905`
    let c = num + 1.0 / 3.0; // `:47906`
    let c2 = num; // `:47907`
    let c3 = num - 1.0 / 3.0; // `:47908`
    let r = (hue2rgb(c, t, num2) * 255.0).round_ties_even() as u8; // `:47912`
    let g = (hue2rgb(c2, t, num2) * 255.0).round_ties_even() as u8; // `:47913`
    let b = (hue2rgb(c3, t, num2) * 255.0).round_ties_even() as u8; // `:47914`
    [r, g, b, a]
}

/// `Utils.ShiftHue` (`Utils.cs:412-421`).
///
/// `hueShift` is wrapped into `[0, 1)`, `luminanceShift` is clamped to `[0, 1]`, and
/// `saturationBoost` is passed to `hslToRgb` **unclamped** (`Utils.cs:420`).
pub fn shift_hue(rgb: [u8; 4], hue_shift: f32, lum_shift: f32, sat_boost: f32) -> [u8; 4] {
    let (h, s, l) = rgb_to_hsl(rgb);
    // `float num = (vector.X + hueShift) % 1f; if (num < 0f) num++;` (`:415-419`).
    let mut num = (h as f32 + hue_shift) % 1.0;
    if num < 0.0 {
        num += 1.0;
    }
    // `Main.hslToRgb(num, vector.Y + saturationBoost, Utils.Clamp(vector.Z +
    // luminanceShift, 0f, 1f), color.A)` (`:420`). Saturation is intentionally not
    // clamped; luminance uses `Utils.Clamp` (max then min, `Utils.cs:223-234`).
    let lum = (l as f32 + lum_shift).clamp(0.0, 1.0);
    hsl_to_rgb(num as f64, (s as f32 + sat_boost) as f64, lum as f64, rgb[3])
}

/// `Utils.ShiftBlueToCyanTheme` (`Utils.cs:423-426`) = `ShiftHue(c, -0.04, 0.04, 0.2)`.
pub fn shift_blue_to_cyan(rgb: [u8; 4]) -> [u8; 4] {
    shift_hue(rgb, -0.04, 0.04, 0.2)
}

/// XNA `Color * float`: every channel, alpha included, is **truncated**
/// (`(byte)(c * m)`), not rounded.
pub fn mul(rgb: [u8; 4], m: f32) -> [u8; 4] {
    let f = |c: u8| (c as f32 * m) as u8;
    [f(rgb[0]), f(rgb[1]), f(rgb[2]), f(rgb[3])]
}

/// Renderer tint: `[r/255, g/255, b/255, a/255]`.
pub fn tint(rgb: [u8; 4]) -> [f32; 4] {
    [
        rgb[0] as f32 / 255.0,
        rgb[1] as f32 / 255.0,
        rgb[2] as f32 / 255.0,
        rgb[3] as f32 / 255.0,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    // Expected values derived independently from `Utils.cs:412-426` /
    // `Main.cs:47890-47980` (float32 `rgbToHsl`, double `hslToRgb`, `Math.Round` =
    // ties-to-even, XNA `Color * float` truncation) -- never from this code.

    #[test]
    fn xna_color_mul_truncates_every_channel() {
        // 255 * 0.7 = 178.5 -> 178 (truncation, not 179).
        assert_eq!(mul([1, 2, 3, 255], 0.7), [0, 1, 2, 178]);
        assert_eq!(mul([63, 82, 151, 255], 0.7), [44, 57, 105, 178]);
        assert_eq!(mul([16, 36, 56, 180], 0.9), [14, 32, 50, 162]);
        // 229.5 -> 229 (truncation; ties-to-even would give 230).
        assert_eq!(mul([89, 116, 213, 255], 0.9), [80, 104, 191, 229]);
    }

    #[test]
    fn themed_colours_match_the_source_table() {
        // Panel background: `ShiftBlueToCyanTheme((63,82,151)*0.7f) * 0.8f`
        // (`NewCraftingUI.cs:326,328`).
        assert_eq!(
            mul(shift_blue_to_cyan(mul([63, 82, 151, 255], 0.7)), 0.8),
            [26, 64, 108, 142]
        );
        // Panel border: `ShiftBlueToCyanTheme(Color.Black) * 0.8f` (`:327,329`).
        assert_eq!(
            mul(shift_blue_to_cyan([0, 0, 0, 255]), 0.8),
            [9, 6, 7, 204]
        );
        // Craft-slot tint: `ShiftBlueToCyanTheme((16,36,56,180)*0.9f)`
        // (`ItemSlot.cs:2270-2271`, `:2299-2300`).
        assert_eq!(
            shift_blue_to_cyan(mul([16, 36, 56, 180], 0.9)),
            [10, 58, 74, 162]
        );
        // Separator: `ShiftBlueToCyanTheme((89,116,213,255)*0.9f)`
        // (`NewCraftingUI.cs:361`).
        assert_eq!(
            shift_blue_to_cyan(mul([89, 116, 213, 255], 0.9)),
            [73, 139, 218, 229]
        );
        // Search box: `ShiftBlueToCyanTheme((35,40,83,255))` (`UIWrappedSearchBar.cs:117-123`).
        assert_eq!(shift_blue_to_cyan([35, 40, 83, 255]), [27, 56, 111, 255]);
    }

    #[test]
    fn grey_round_trip_uses_the_zero_saturation_branch() {
        // `Main.cs:47895-47900`: `Saturation == 0f` returns `Luminosity*255` rounded
        // on every channel.
        let (h, s, l) = rgb_to_hsl([128, 128, 128, 255]);
        assert_eq!(h, 0.0);
        assert_eq!(s, 0.0);
        assert_eq!(hsl_to_rgb(h, s, l, 255), [128, 128, 128, 255]);
    }

    #[test]
    fn non_grey_round_trip_through_hsl() {
        // Pure red is an exact anchor: hsl = (0, 1, 0.5) and back.
        let (h, s, l) = rgb_to_hsl([255, 0, 0, 255]);
        assert_eq!((h, s, l), (0.0, 1.0, 0.5));
        assert_eq!(hsl_to_rgb(h, s, l, 255), [255, 0, 0, 255]);

        // A non-primary colour round-trips byte-exactly through the float32
        // `rgbToHsl` / double `hslToRgb` pair.
        let rgb = [63u8, 82, 151, 255];
        let (h, s, l) = rgb_to_hsl(rgb);
        assert_eq!(hsl_to_rgb(h, s, l, 255), rgb);
    }

    #[test]
    fn shift_hue_passes_the_unclamped_saturation() {
        // `Utils.cs:420` feeds `hslToRgb` `vector.Y + saturationBoost` with no clamp.
        // For (204,102,0) the source saturation is exactly 1.0, so the boost makes it
        // 1.2. Derived from the C# `hslToRgb` by hand: `num2 = 0.4*(1+1.2) = 0.88`,
        // `t = -0.08`, red reads the `2c < 1` branch -> `t2*255 = 224.4 -> 224`.
        // Clamping `s` back to 1.0 would leave red at 204.
        assert_eq!(shift_hue([204, 102, 0, 255], 0.0, 0.0, 0.2), [224, 102, 0, 255]);
    }

    #[test]
    fn tint_maps_bytes_to_unit_floats() {
        assert_eq!(tint([10, 58, 74, 162]), [
            10.0 / 255.0,
            58.0 / 255.0,
            74.0 / 255.0,
            162.0 / 255.0,
        ]);
    }
}
