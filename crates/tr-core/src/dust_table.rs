//! The per-type dust colour table: `Dust.GetAlpha` (`Dust.cs:2425-2727`),
//! `Dust.GetColor` (`Dust.cs:2729-2772`) and the two visual accessors
//! (`Dust.cs:2774-2790`).
//!
//! # Scope (S1-E-2)
//!
//! This is the *data* half of the ROADMAP row: the per-type **colour** and
//! **fade** columns as pure functions over a [`Dust`] record. The row's other
//! columns belong to the tasks that own their consumers, so nothing is
//! duplicated here:
//!
//! - **gravity / `noGravity` / velocity scaling** - the spawn-time half is
//!   already verbatim in [`crate::particles::DustPool::new_dust`]
//!   (`Dust.cs:214-252`, S1-E-1); the per-tick half is the `UpdateDust` chain
//!   (`Dust.cs:1819-1828`) and is sequenced by S1-E-3.
//! - **light** - the ~70 `Lighting.AddLight` arms spread over
//!   `Dust.cs:485-2050` need the entity-light seam, i.e. S1-E-5.
//! - **the id names** - `DustID`'s 196 named constants are transcribed at
//!   `tr-app::id_tables::DUST_ID` (S1-F-6); the port keeps no second copy.
//!
//! Two arms are documented drops, both because the port has no equivalent
//! global: type 157 reads `Main.mouseTextColor` (`Dust.cs:2630`) and type 327
//! reads `Main.rand` + `Main.Disco*` (`Dust.cs:2658-2675`). Both fall through
//! to the default formula instead of returning their own colour.
//!
//! # Numeric conventions
//!
//! [`Color`] carries sRGB bytes, so every XNA `new Color(int, int, int, int)`
//! becomes a truncating `as u8` (the ctor byte-casts, wrapping past 255 - the
//! same convention the 213 draw already relies on, `minecart.rs:1143-1146`),
//! and `Color * float` scales all four channels before that cast. `Color.Lerp`
//! is MonoGame's float lerp + `(int)` cast per channel.

use crate::particles::{Color, Dust};

/// `(byte)` of an XNA `new Color(int, …)` argument: truncate, wrap past 255.
const fn byte(v: i32) -> u8 {
    v as u8
}

/// XNA `Color * float` (`MonoGame Color.cs`): every channel scales, then the
/// `(int)` cast of the ctor.
fn scale_color(c: Color, s: f32) -> Color {
    Color::new(
        byte((c.r as f32 * s) as i32),
        byte((c.g as f32 * s) as i32),
        byte((c.b as f32 * s) as i32),
        byte((c.a as f32 * s) as i32),
    )
}

/// XNA `Color.Lerp` (`MonoGame Color.cs`): `MathHelper.Lerp` per channel, then
/// the ctor's truncating cast.
fn lerp_color(a: Color, b: Color, t: f32) -> Color {
    let ch = |x: u8, y: u8| {
        let (x, y) = (x as f32, y as f32);
        byte((x + (y - x) * t) as i32)
    };
    Color::new(ch(a.r, b.r), ch(a.g, b.g), ch(a.b, b.b), ch(a.a, b.a))
}

/// `Color.ToVector3` * `Color`'s `ToVector3`, back through
/// `new Color(Vector3)` (`Dust.cs:2590`, `:2654`). MonoGame builds that ctor
/// from the normalised channels, so this is `per-channel product / 255`.
fn mul_rgb(a: Color, b: Color) -> [u8; 3] {
    let ch = |x: u8, y: u8| byte(((x as f32 / 255.0) * (y as f32 / 255.0) * 255.0) as i32);
    [ch(a.r, b.r), ch(a.g, b.g), ch(a.b, b.b)]
}

/// `Dust.GetAlpha(Color newColor)` (`Dust.cs:2425-2727`): the per-type colour
/// of one dust over the light colour the caller sampled at its position.
pub fn get_alpha(d: &Dust, new_color: Color) -> Color {
    // `:2427-2430`.
    if d.full_bright {
        return Color::WHITE;
    }
    // `:2431` - the alpha fraction the default path folds in.
    let num = (255 - d.alpha) as f32 / 255.0;
    // `:2432-2455` - the switch.
    match d.dust_type {
        323 => return Color::WHITE,                         // `:2434-2435`
        308 | 309 => return Color::new(225, 200, 250, 190), // `:2436-2438`
        324 => return scale_color(Color::new(225, 200, 250, 190), num), // `:2439-2440`
        299 | 300 | 301 | 305 => {
            // `:2441-2455` - the nested type switch; its `_` arm (`:2453`) is
            // unreachable for these four ids.
            return match d.dust_type {
                299 => Color::new(50, 255, 50, 200),
                300 => Color::new(50, 200, 255, 255),
                301 => Color::new(255, 50, 125, 200),
                305 => Color::new(200, 50, 200, 200),
                _ => Color::new(255, 150, 150, 200),
            };
        }
        _ => {}
    }
    // `:2456-2726` - the `default:` block.
    if d.dust_type == 304 {
        return scale_color(Color::WHITE, num); // `:2458-2461`
    }
    if d.dust_type == 306 {
        return scale_color(d.color, num); // `:2462-2465`
    }
    if d.dust_type == 292 {
        return Color::WHITE; // `:2466-2469`
    }
    if d.dust_type == 259 {
        return Color::new(230, 230, 230, 230); // `:2470-2473`
    }
    if d.dust_type == 261 {
        return Color::new(230, 230, 230, 115); // `:2474-2477`
    }
    if d.dust_type == 254 || d.dust_type == 255 {
        return Color::new(255, 255, 255, 0); // `:2478-2481`
    }
    if d.dust_type == 258 {
        return Color::new(150, 50, 50, 0); // `:2482-2485`
    }
    if d.dust_type == 263 || d.dust_type == 264 {
        // `:2486-2489` - integer halving on R and A, full +127 on G/B, then a
        // 0.5 scale of the byte-cast colour.
        let c = Color::new(
            byte(d.color.r as i32 / 2 + 127),
            byte(d.color.g as i32 + 127),
            byte(d.color.b as i32 + 127),
            byte(d.color.a as i32 / 8),
        );
        return scale_color(c, 0.5);
    }
    if d.dust_type == 235 {
        return Color::new(255, 255, 255, 0); // `:2490-2493`
    }
    if ((86..=91).contains(&d.dust_type)
        || d.dust_type == 262
        || d.dust_type == 286
        || d.dust_type == 138
        || d.dust_type == 325)
        && !d.no_light
    {
        return Color::new(255, 255, 255, 0); // `:2494-2497`
    }
    if d.dust_type == 213 || d.dust_type == 260 {
        let n = (d.scale / 2.5 * 255.0) as i32; // `:2500`
        return Color::new(byte(n), byte(n), byte(n), byte(n));
    }
    if d.dust_type == 64 && d.alpha == 255 && d.no_light {
        return Color::new(255, 255, 255, 0); // `:2503-2506`
    }
    if d.dust_type == 197 {
        return Color::new(250, 250, 250, 150); // `:2507-2510`
    }
    if (110..=114).contains(&d.dust_type)
        || d.dust_type == 311
        || d.dust_type == 312
        || d.dust_type == 313
    {
        return Color::new(200, 200, 200, 0); // `:2511-2514`
    }
    if d.dust_type == 204 {
        return Color::new(255, 255, 255, 0); // `:2515-2518`
    }
    if d.dust_type == 181 {
        return Color::new(200, 200, 200, 0); // `:2519-2522`
    }
    if d.dust_type == 182 || d.dust_type == 206 {
        return Color::new(255, 255, 255, 0); // `:2523-2526`
    }
    if d.dust_type == 159 {
        return Color::new(250, 250, 250, 50); // `:2527-2530`
    }
    if d.dust_type == 163 || d.dust_type == 205 {
        return Color::new(250, 250, 250, 0); // `:2531-2534`
    }
    if d.dust_type == 170 {
        return Color::new(200, 200, 200, 100); // `:2535-2538`
    }
    if d.dust_type == 180 {
        return Color::new(200, 200, 200, 0); // `:2539-2542`
    }
    if d.dust_type == 175 {
        return Color::new(200, 200, 200, 0); // `:2543-2546`
    }
    if d.dust_type == 183 {
        return Color::new(50, 0, 0, 0); // `:2547-2550`
    }
    if d.dust_type == 172 {
        return Color::new(250, 250, 250, 150); // `:2551-2554`
    }
    if d.dust_type == 160 || d.dust_type == 162 || d.dust_type == 164 || d.dust_type == 173 {
        let n = (250.0 * d.scale) as i32; // `:2557`
        return Color::new(byte(n), byte(n), byte(n), 0);
    }
    if d.dust_type == 92 || d.dust_type == 106 || d.dust_type == 107 {
        return Color::new(255, 255, 255, 0); // `:2560-2563`
    }
    if d.dust_type == 185 {
        return Color::new(200, 200, 255, 125); // `:2564-2567`
    }
    if d.dust_type == 127 || d.dust_type == 187 {
        return Color::new(new_color.r, new_color.g, new_color.b, 25); // `:2568-2571`
    }
    if d.dust_type == 156 || d.dust_type == 230 || d.dust_type == 234 {
        return Color::new(255, 255, 255, 0); // `:2572-2575`
    }
    if d.dust_type == 270 {
        return Color::new(
            byte(new_color.r as i32 / 2 + 127),
            byte(new_color.g as i32 / 2 + 127),
            byte(new_color.b as i32 / 2 + 127),
            25,
        ); // `:2576-2579`
    }
    if d.dust_type == 271 {
        return Color::new(
            byte(new_color.r as i32 / 2 + 127),
            byte(new_color.g as i32 / 2 + 127),
            byte(new_color.b as i32 / 2 + 127),
            127,
        ); // `:2580-2583`
    }
    if matches!(
        d.dust_type,
        6 | 242 | 174 | 135 | 75 | 20 | 21 | 231 | 169 | 130
            ..=134 | 158 | 293 | 294 | 295 | 296 | 297 | 298 | 307 | 310
    ) {
        return Color::new(new_color.r, new_color.g, new_color.b, 25); // `:2584-2587`
    }
    if d.dust_type == 278 {
        // `:2588-2593`.
        let [r, g, b] = mul_rgb(new_color, d.color);
        return Color::new(r, g, b, 25);
    }
    if (219..=223).contains(&d.dust_type) {
        let c = lerp_color(new_color, Color::WHITE, 0.5); // `:2594-2598`
        return Color::new(c.r, c.g, c.b, 25);
    }
    if d.dust_type == 226 || d.dust_type == 272 || d.dust_type == 228 {
        let c = lerp_color(new_color, Color::WHITE, 0.8); // `:2599-2608`
        return Color::new(c.r, c.g, c.b, 25);
    }
    if d.dust_type == 279 {
        // `:2609-2614` - the original alpha survives the lerp, then the whole
        // colour scales by `min(scale, 1)`.
        let a = new_color.a;
        let c = lerp_color(new_color, Color::WHITE, 0.8);
        return scale_color(Color::new(c.r, c.g, c.b, a), d.scale.min(1.0));
    }
    if d.dust_type == 229 || d.dust_type == 269 {
        let c = lerp_color(new_color, Color::WHITE, 0.6); // `:2615-2619`
        return Color::new(c.r, c.g, c.b, 25);
    }
    if (d.dust_type == 68 || d.dust_type == 70) && d.no_gravity {
        return Color::new(255, 255, 255, 0); // `:2620-2623`
    }
    // `:2627-2651` (type 157, `Main.mouseTextColor`) and `:2658-2676` (type
    // 327, `Main.rand` + `Main.Disco*`) are documented drops - see the module
    // doc. Both fall through to the default formula below.
    if d.dust_type == 284 {
        // `:2652-2657` - the Vector4 product; the alpha is overwritten.
        let [r, g, b] = mul_rgb(new_color, d.color);
        return Color::new(r, g, b, d.color.a);
    }
    if d.dust_type == 58 {
        return Color::new(255, 255, 255, 0); // `:2677-2680`
    }
    // `:2681-2711` - the slow-fade lists and the small constant returns.
    let num = if matches!(
        d.dust_type,
        15 | 274 | 20 | 21 | 29 | 35 | 41 | 44 | 27 | 45 | 55 | 56 | 57 | 58 | 73 | 74
    ) {
        (num + 3.0) / 4.0
    } else if d.dust_type == 43 {
        (num + 9.0) / 10.0
    } else {
        if (244..=247).contains(&d.dust_type) {
            return Color::new(255, 255, 255, 0); // `:2691-2694`
        }
        if d.dust_type == 66 {
            return Color::new(new_color.r, new_color.g, new_color.b, 0); // `:2695-2698`
        }
        if d.dust_type == 267 {
            return Color::new(d.color.r, d.color.g, d.color.b, 0); // `:2699-2702`
        }
        if d.dust_type == 71 {
            return Color::new(200, 200, 200, 0); // `:2703-2706`
        }
        if d.dust_type == 72 {
            return Color::new(200, 200, 200, 200); // `:2707-2710`
        }
        num
    };
    // `:2712-2724`.
    let r = byte((new_color.r as f32 * num) as i32);
    let g = byte((new_color.g as f32 * num) as i32);
    let b = byte((new_color.b as f32 * num) as i32);
    let a = byte((new_color.a as i32 - d.alpha).clamp(0, 255));
    Color::new(r, g, b, a)
}

/// `Dust.GetColor(Color newColor)` (`Dust.cs:2729-2772`): the overlay pass -
/// `this.color - (255 - newColor)` per channel, clamped to `0..=255`.
pub fn get_color(d: &Dust, new_color: Color) -> Color {
    if d.dust_type == 284 {
        return Color::TRANSPARENT; // `:2731-2734`
    }
    let ch = |c: u8, n: u8| byte((c as i32 - (255 - n as i32)).clamp(0, 255));
    Color::new(
        ch(d.color.r, new_color.r),
        ch(d.color.g, new_color.g),
        ch(d.color.b, new_color.b),
        ch(d.color.a, new_color.a),
    )
}

/// `Dust.GetVisualRotation` (`Dust.cs:2774-2781`): 304 draws unrotated.
pub fn get_visual_rotation(d: &Dust) -> f32 {
    if d.dust_type == 304 {
        return 0.0;
    }
    d.rotation
}

/// `Dust.GetVisualScale` (`Dust.cs:2783-2790`): 304 draws at scale 1.
pub fn get_visual_scale(d: &Dust) -> f32 {
    if d.dust_type == 304 {
        return 1.0;
    }
    d.scale
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A dust of `t` at `scale`, every other field at its default.
    fn at(t: i32) -> Dust {
        Dust {
            dust_type: t,
            scale: 1.0,
            ..Dust::default()
        }
    }

    /// The light colour the draw pass sampled: mid grey, opaque.
    const LIGHT: Color = Color::new(128, 128, 128, 255);

    /// S1-E-2 - the `GetAlpha` `switch` arms (`Dust.cs:2432-2455`) and the
    /// `fullBright` short-circuit (`:2427-2430`).
    #[test]
    fn s1_e_2_get_alpha_switch_and_full_bright() {
        assert_eq!(get_alpha(&at(323), LIGHT), Color::WHITE, ":2434");
        assert_eq!(
            get_alpha(&at(308), LIGHT),
            Color::new(225, 200, 250, 190),
            ":2436"
        );
        assert_eq!(
            get_alpha(&at(309), LIGHT),
            Color::new(225, 200, 250, 190),
            ":2437"
        );
        // 324 scales that colour by the alpha fraction `(255-alpha)/255`.
        let d = Dust {
            alpha: 0,
            ..at(324)
        };
        assert_eq!(
            get_alpha(&d, LIGHT),
            Color::new(225, 200, 250, 190),
            ":2440"
        );
        let d = Dust {
            alpha: 255,
            ..at(324)
        };
        assert_eq!(get_alpha(&d, LIGHT), Color::TRANSPARENT, ":2440 alpha 255");
        // The four-colour nested switch (`:2448-2453`).
        assert_eq!(get_alpha(&at(299), LIGHT), Color::new(50, 255, 50, 200));
        assert_eq!(get_alpha(&at(300), LIGHT), Color::new(50, 200, 255, 255));
        assert_eq!(get_alpha(&at(301), LIGHT), Color::new(255, 50, 125, 200));
        assert_eq!(get_alpha(&at(305), LIGHT), Color::new(200, 50, 200, 200));
        // `fullBright` wins over every arm (`:2427`).
        let d = Dust {
            full_bright: true,
            ..at(299)
        };
        assert_eq!(get_alpha(&d, LIGHT), Color::WHITE);
    }

    /// S1-E-2 - the constant-colour `default:` arms
    /// (`Dust.cs:2458-2623`), each asserted against the C# literal.
    #[test]
    fn s1_e_2_get_alpha_constant_arms() {
        let cases: &[(i32, Color)] = &[
            (292, Color::new(255, 255, 255, 255)), // `:2466`
            (259, Color::new(230, 230, 230, 230)), // `:2470`
            (261, Color::new(230, 230, 230, 115)), // `:2474`
            (254, Color::new(255, 255, 255, 0)),   // `:2478`
            (255, Color::new(255, 255, 255, 0)),   // `:2478`
            (258, Color::new(150, 50, 50, 0)),     // `:2482`
            (235, Color::new(255, 255, 255, 0)),   // `:2490`
            (197, Color::new(250, 250, 250, 150)), // `:2507`
            (204, Color::new(255, 255, 255, 0)),   // `:2515`
            (181, Color::new(200, 200, 200, 0)),   // `:2519`
            (182, Color::new(255, 255, 255, 0)),   // `:2523`
            (206, Color::new(255, 255, 255, 0)),   // `:2523`
            (159, Color::new(250, 250, 250, 50)),  // `:2527`
            (163, Color::new(250, 250, 250, 0)),   // `:2531`
            (205, Color::new(250, 250, 250, 0)),   // `:2531`
            (170, Color::new(200, 200, 200, 100)), // `:2535`
            (180, Color::new(200, 200, 200, 0)),   // `:2539`
            (175, Color::new(200, 200, 200, 0)),   // `:2543`
            (183, Color::new(50, 0, 0, 0)),        // `:2547`
            (172, Color::new(250, 250, 250, 150)), // `:2551`
            (92, Color::new(255, 255, 255, 0)),    // `:2560`
            (106, Color::new(255, 255, 255, 0)),   // `:2560`
            (107, Color::new(255, 255, 255, 0)),   // `:2560`
            (185, Color::new(200, 200, 255, 125)), // `:2564`
            (156, Color::new(255, 255, 255, 0)),   // `:2572`
            (230, Color::new(255, 255, 255, 0)),   // `:2572`
            (234, Color::new(255, 255, 255, 0)),   // `:2572`
            (58, Color::new(255, 255, 255, 0)),    // `:2677`
            (244, Color::new(255, 255, 255, 0)),   // `:2691`
            (247, Color::new(255, 255, 255, 0)),   // `:2691`
            (71, Color::new(200, 200, 200, 0)),    // `:2703`
            (72, Color::new(200, 200, 200, 200)),  // `:2707`
        ];
        for &(t, want) in cases {
            assert_eq!(get_alpha(&at(t), LIGHT), want, "type {t}");
        }
        // `:2511` - the 110..=114 / 311..=313 grey family.
        for t in [110, 111, 112, 113, 114, 311, 312, 313] {
            assert_eq!(
                get_alpha(&at(t), LIGHT),
                Color::new(200, 200, 200, 0),
                "type {t}"
            );
        }
        // `:2584` - the 25-alpha "light through" family.
        for t in [
            6, 242, 174, 135, 75, 20, 21, 231, 169, 130, 131, 132, 133, 134, 158, 293, 294, 295,
            296, 297, 298, 307, 310,
        ] {
            assert_eq!(
                get_alpha(&at(t), LIGHT),
                Color::new(128, 128, 128, 25),
                "type {t}"
            );
        }
        // `:2494` - the noLight gate on the gem/torch family.
        for t in [86, 87, 88, 89, 90, 91, 262, 286, 138, 325] {
            assert_eq!(
                get_alpha(&at(t), LIGHT),
                Color::new(255, 255, 255, 0),
                "type {t} lit"
            );
            let d = Dust {
                no_light: true,
                ..at(t)
            };
            assert_ne!(
                get_alpha(&d, LIGHT),
                Color::new(255, 255, 255, 0),
                "type {t} noLight"
            );
        }
        // `:2503` - type 64 needs both alpha 255 and noLight.
        let d = Dust {
            alpha: 255,
            no_light: true,
            ..at(64)
        };
        assert_eq!(get_alpha(&d, LIGHT), Color::new(255, 255, 255, 0));
        assert_ne!(
            get_alpha(
                &Dust {
                    alpha: 254,
                    no_light: true,
                    ..at(64)
                },
                LIGHT
            ),
            Color::new(255, 255, 255, 0)
        );
        // `:2620` - 68/70 need noGravity.
        for t in [68, 70] {
            let d = Dust {
                no_gravity: true,
                ..at(t)
            };
            assert_eq!(
                get_alpha(&d, LIGHT),
                Color::new(255, 255, 255, 0),
                "type {t}"
            );
            assert_ne!(
                get_alpha(&at(t), LIGHT),
                Color::new(255, 255, 255, 0),
                "type {t} gravity"
            );
        }
    }

    /// S1-E-2 - the formula arms: `this.color`/`scale`/`newColor` operands
    /// (`Dust.cs:2458-2657`).
    #[test]
    fn s1_e_2_get_alpha_formula_arms() {
        // `:2462` - 306 is `this.color * num`.
        let d = Dust {
            alpha: 0,
            color: Color::new(40, 80, 120, 200),
            ..at(306)
        };
        assert_eq!(get_alpha(&d, LIGHT), Color::new(40, 80, 120, 200));
        let d = Dust {
            alpha: 51,
            color: Color::new(40, 80, 120, 200),
            ..at(306)
        };
        // (255-51)/255 = 0.8 → 32 / 64 / 96 / 160.
        assert_eq!(get_alpha(&d, LIGHT), Color::new(32, 64, 96, 160));
        // `:2458` - 304 is `Color.White * num`.
        assert_eq!(get_alpha(&at(304), LIGHT), Color::WHITE);
        // `:2486` - 263/264 halve R and A, +127 G/B, then scale 0.5.
        let d = Dust {
            color: Color::new(100, 50, 25, 80),
            ..at(263)
        };
        assert_eq!(get_alpha(&d, LIGHT), Color::new(88, 88, 76, 5));
        // `:2500` - 213/260 derive the grey from scale/2.5.
        let d = Dust {
            scale: 2.5,
            ..at(213)
        };
        assert_eq!(get_alpha(&d, LIGHT), Color::WHITE);
        let d = Dust {
            scale: 1.25,
            ..at(260)
        };
        assert_eq!(get_alpha(&d, LIGHT), Color::new(127, 127, 127, 127));
        // `:2557` - 160/162/164/173 are 250*scale grey.
        let d = Dust {
            scale: 0.5,
            ..at(162)
        };
        assert_eq!(get_alpha(&d, LIGHT), Color::new(125, 125, 125, 0));
        // `:2568` - 127/187 pass the light colour through with alpha 25.
        assert_eq!(get_alpha(&at(127), LIGHT), Color::new(128, 128, 128, 25));
        // `:2576`/`:2580` - 270/271 halve the light and add 127.
        assert_eq!(get_alpha(&at(270), LIGHT), Color::new(191, 191, 191, 25));
        assert_eq!(get_alpha(&at(271), LIGHT), Color::new(191, 191, 191, 127));
        // `:2595` - 219..=223 lerp the light halfway to white.
        assert_eq!(get_alpha(&at(219), LIGHT), Color::new(191, 191, 191, 25));
        // `:2601` - 226/272/228 lerp 0.8 of the way.
        assert_eq!(get_alpha(&at(228), LIGHT), Color::new(229, 229, 229, 25));
        // `:2613` - 229/269 lerp 0.6.
        assert_eq!(get_alpha(&at(269), LIGHT), Color::new(204, 204, 204, 25));
        // `:2609` - 279 keeps the light's own alpha through the lerp, then
        // scales the whole colour by min(scale, 1).
        let d = Dust {
            scale: 0.5,
            ..at(279)
        };
        assert_eq!(get_alpha(&d, LIGHT), Color::new(114, 114, 114, 127));
        let d = Dust {
            scale: 1.0,
            ..at(279)
        };
        assert_eq!(get_alpha(&d, LIGHT), Color::new(229, 229, 229, 255));
        // `:2590`/`:2654` - the Vector3/Vector4 products, 278 vs 284's alpha.
        let d = Dust {
            color: Color::new(255, 128, 0, 16),
            ..at(278)
        };
        assert_eq!(get_alpha(&d, LIGHT), Color::new(128, 64, 0, 25));
        let d = Dust {
            color: Color::new(255, 128, 0, 16),
            ..at(284)
        };
        assert_eq!(get_alpha(&d, LIGHT), Color::new(128, 64, 0, 16));
        // `:2695`/`:2699` - 66 uses the light's rgb, 267 its own.
        assert_eq!(get_alpha(&at(66), LIGHT), Color::new(128, 128, 128, 0));
        let d = Dust {
            color: Color::new(1, 2, 3, 200),
            ..at(267)
        };
        assert_eq!(get_alpha(&d, LIGHT), Color::new(1, 2, 3, 0));
    }

    /// S1-E-2 - the default tail: the two slow-fade divisors and the
    /// `newColor * num`, `A - alpha` formula (`Dust.cs:2681-2724`).
    #[test]
    fn s1_e_2_get_alpha_default_tail() {
        let light = Color::new(200, 100, 50, 255);
        // `:2712-2724` - alpha 0 means num 1, so the light passes through.
        assert_eq!(get_alpha(&at(63), light), light);
        // `:2681` - the 16-type list fades at a quarter rate:
        // alpha 127 → num 0.50196 → (num+3)/4 = 0.8754.
        let d = Dust {
            alpha: 127,
            ..at(15)
        };
        assert_eq!(get_alpha(&d, light), Color::new(175, 87, 43, 128));
        // `:2685` - type 43 fades at a tenth rate: (0.50196+9)/10 = 0.95.
        let d = Dust {
            alpha: 127,
            ..at(43)
        };
        assert_eq!(get_alpha(&d, light), Color::new(190, 95, 47, 128));
        // `:2715` - the alpha channel is `newColor.A - alpha`, clamped.
        let d = Dust {
            alpha: 40,
            ..at(63)
        };
        assert_eq!(get_alpha(&d, Color::new(255, 255, 255, 20)).a, 0);
        let d = Dust {
            alpha: 40,
            ..at(63)
        };
        assert_eq!(get_alpha(&d, Color::new(255, 255, 255, 255)).a, 215);
    }

    /// S1-E-2 - `GetColor` (`Dust.cs:2729-2772`) and the two visual
    /// accessors (`:2774-2790`).
    #[test]
    fn s1_e_2_get_color_and_visual_accessors() {
        // `:2731` - 284 is transparent on the overlay pass.
        assert_eq!(get_color(&at(284), Color::WHITE), Color::TRANSPARENT);
        // The default: `color - (255 - newColor)` per channel.
        let d = Dust {
            color: Color::new(200, 100, 50, 25),
            ..at(63)
        };
        assert_eq!(get_color(&d, Color::WHITE), d.color);
        assert_eq!(get_color(&d, Color::TRANSPARENT), Color::TRANSPARENT);
        // Half light: 200-127=73, 100-127<0→0, 50-127<0→0, 25-127<0→0.
        assert_eq!(
            get_color(&d, Color::new(128, 128, 128, 128)),
            Color::new(73, 0, 0, 0)
        );
        // `:2776`/`:2785` - 304 draws unrotated at scale 1; everyone else
        // reports its own record.
        let d = Dust {
            rotation: 0.75,
            scale: 0.25,
            ..at(304)
        };
        assert_eq!(get_visual_rotation(&d), 0.0);
        assert_eq!(get_visual_scale(&d), 1.0);
        let d = Dust {
            rotation: 0.75,
            scale: 0.25,
            ..at(63)
        };
        assert_eq!(get_visual_rotation(&d), 0.75);
        assert_eq!(get_visual_scale(&d), 0.25);
    }
}
