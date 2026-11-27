//! The Empress of Light's dedicated NPC draw (`Main.DrawNPCDirect_HallowBoss`,
//! `Main.cs:26471-26652`, dispatched from `DrawNPCDirect`'s case 636 at
//! `:24150-24152`) + the arm-frame picker (`DrawNPCDirect_GetHallowBossArmFrame`,
//! `:26654-26807`) + the 636 `GetAlpha` tint arm (`NPC.cs:93501-93505`).
//!
//! W4-d125 remainders 2+5 (the OurFavoriteColor tint + the trail/draw work).
//! The census correction: `Main.OurFavoriteColor` is a STATIC
//! `new Color(255, 231, 69)` (`Main.cs:66748`, the only write in the whole
//! decompile) — there is NO hue-cycle on the static itself. The hue-cycle
//! the brief means is the arm's GLOW colour (`:26611-26614`):
//! `hslToRgb((sin(GlobalTimeWrappedHourly * 2π * 0.5) * 0.5 + 0.5) * 0.08
//! + 0.6, 1, 0.5)`, A=0, ×0.6 — and the ENRAGED branch (`:26615-26620`)
//! replaces it with `OurFavoriteColor`, A=0, ×0.3. Both then scale by
//! `num4 * Opacity` (`:26621`).
//!
//! Pure fns over byte colours (the XNA two-step truncation convention,
//! `render/special_tiles.rs`'s `xna_*` helpers): the wiring in
//! `render/entities.rs` maps the light colour to bytes, runs the plan and
//! pushes the quads.
//!
//! Deliberate deviations (ANALYSIS "The empress draw (W4-d125)"):
//! * The five tall overlay strips (`Extra_157` the shader-pass body copy,
//!   `Extra_158/160` the arms, `Extra_159` the 11-frame wing strip,
//!   `Extra_187` the 8-frame phase-2 wings) are NOT packed — each overflows
//!   the solo atlas headroom (+64..+214 px, `tools/_d125_sheets_measure.js`);
//!   their draws skip until the repack wave (research/309 §2). The halo
//!   ring `Extra_188` (166×174) IS packed (+25 px, 8141/8141).
//! * The `GameShaders.Misc["HallowBoss"]` body pass (`:26599-26610`) is a
//!   compiled HLSL effect not in the decompile — the port draws the body
//!   once with the plain tint (the pillar-glow precedent).
//! * The state-8/9/10 ghost copies (`:26523-26594`) read
//!   `Matrix.CreateRotationX/Y/Z` 3-D spins + `Main.timeForVisualEffects`;
//!   ported as pure geometry here (the plan carries them), the wiring
//!   pushes them only when the overlay sheets exist.
//! * `Main.expertMode` rides the caller's flag (the player-buff mirror,
//!   the `npc.rs` adapter's convention).

use crate::render::special_tiles::{xna_multiply_rgba, xna_scale_color};
use tr_core::scene::get_lerp_value;
use tr_core::tile_light::hsl_to_rgb;

/// `Main.OurFavoriteColor` (`Main.cs:66748`) — the static UI gold.
pub(crate) const OUR_FAVORITE_COLOR: [u8; 4] = [255, 231, 69, 255];

/// XNA `Color.Lerp` per channel, truncating cast (the `dust_table.rs`
/// convention — MonoGame's float lerp + `(int)`).
fn lerp_bytes(a: [u8; 4], b: [u8; 4], t: f32) -> [u8; 4] {
    let ch = |x: u8, y: u8| (x as f32 + (y as f32 - x as f32) * t) as u8;
    [ch(a[0], b[0]), ch(a[1], b[1]), ch(a[2], b[2]), ch(a[3], b[3])]
}

/// `MathHelper.Lerp` (`Main.cs` draw arms) — unclamped float lerp.
fn lerp_f(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

/// `hslToRgb` bytes (the palette returns exact byte/255 floats; `round`
/// recovers the byte — the `cage_halo_color` convention).
fn hsl_bytes(h: f32, s: f32, l: f32) -> [u8; 4] {
    let c = hsl_to_rgb(h, s, l);
    [
        (c[0] * 255.0).round() as u8,
        (c[1] * 255.0).round() as u8,
        (c[2] * 255.0).round() as u8,
        255,
    ]
}

/// `NPC.GetAlpha`'s 636 arm (`NPC.cs:93501-93505`):
/// `Color.Lerp(newColor, White, 0.25) * Opacity`. `newColor` is the light
/// colour with A=255 (the `Lighting.GetColor` shape); `opacity` is
/// `(255 - alpha)/255` (`NPC.cs:6820-6825`).
pub(crate) fn empress_get_alpha(light: [u8; 3], opacity: f32) -> [u8; 4] {
    let lerped = lerp_bytes([light[0], light[1], light[2], 255], [255; 4], 0.25);
    xna_scale_color(lerped, opacity)
}

/// The arm's glow colour (`Main.cs:26611-26621`): the hue-cycled
/// `hslToRgb((num14 * 0.08 + 0.6) % 1, 1, 0.5)` with A=0 ×0.6, or the
/// enraged `OurFavoriteColor` A=0 ×0.3; then ×`num4 * Opacity`.
/// `num14 = sin(GlobalTimeWrappedHourly * 2π * 0.5) * 0.5 + 0.5`.
pub(crate) fn empress_glow_color(
    global_time_wrapped_hourly: f32,
    enraged: bool,
    num4: f32,
    opacity: f32,
) -> [u8; 4] {
    let num14 = (global_time_wrapped_hourly * std::f32::consts::TAU * 0.5).sin() * 0.5 + 0.5;
    let mut color3 = if enraged {
        OUR_FAVORITE_COLOR
    } else {
        hsl_bytes((num14 * 0.08 + 0.6) % 1.0, 1.0, 0.5)
    };
    color3[3] = 0;
    color3 = xna_scale_color(color3, if enraged { 0.3 } else { 0.6 });
    xna_scale_color(color3, num4 * opacity)
}

/// The hue-cycle pulse `num14` itself (the halo radius lerp reads it).
pub(crate) fn empress_glow_pulse(global_time_wrapped_hourly: f32) -> f32 {
    (global_time_wrapped_hourly * std::f32::consts::TAU * 0.5).sin() * 0.5 + 0.5
}

/// `DrawNPCDirect_GetHallowBossArmFrame` (`Main.cs:26654-26807`) verbatim:
/// `(armFrame_Count, armFrameToUseLeft, armFrameToUseRight)`. The frame
/// constants are the C#'s num..num7 = 0..6; the count is always 7.
pub(crate) fn hallow_boss_arm_frame(ai0: f32, ai1: f32, ai3: f32, expert: bool) -> (i32, i32, i32) {
    let (f0, f1, f2, f3, f4, f5, f6) = (0, 1, 2, 3, 4, 5, 6);
    let mut left = f0;
    let mut right = f0;
    let num8 = ai1;
    match ai0 as i32 {
        6 => {
            // `:26671` — both arms together.
            let f = if num8 < 6.0 {
                f2
            } else if num8 < 174.0 {
                f3
            } else if !(num8 < 180.0) {
                f0
            } else {
                f2
            };
            left = f;
            right = f;
        }
        0 => {
            // `:26674`.
            let f = if num8 < 106.0 {
                f1
            } else if !(num8 < 110.0) {
                f0
            } else {
                f2
            };
            left = f;
            right = f;
        }
        2 | 11 => {
            // `:26678` — left only.
            left = if num8 < 5.0 {
                f2
            } else if !(num8 < 65.0) {
                f2
            } else {
                f3
            };
        }
        5 => {
            // `:26681` — right only.
            right = if num8 < 6.0 {
                f2
            } else if !(num8 < 54.0) {
                f2
            } else {
                f3
            };
        }
        4 | 10 => {
            // `:26685`.
            let f = if num8 < 6.0 {
                f2
            } else if !(num8 < 54.0) {
                f2
            } else {
                f3
            };
            left = f;
            right = f;
        }
        8 | 9 => {
            // `:26687-26718` — the dash arms' ai[3]-sign swap. num16 = -1,
            // so `num15 == -1 * num16` is ai3 == 1 and `num15 == num16` is
            // ai3 == -1 (the C#'s literal shape kept).
            let num9 = if num8 < 10.0 {
                f2
            } else if num8 < 20.0 {
                f3
            } else if !(num8 < 30.0) {
                f5
            } else {
                f2
            };
            let mut num13 = num9;
            let mut num14 = num9;
            let num15 = ai3 as i32;
            let num16 = -1;
            if num8 < 30.0 {
                if num15 == -1 * num16 {
                    num14 = f1;
                }
                if num15 == num16 {
                    num13 = f1;
                }
            }
            let (num17, num18) = (f5, f6);
            if num15 == num16 && num14 == num17 {
                num14 = num18;
            }
            if num15 == -1 * num16 && num13 == num17 {
                num13 = num18;
            }
            left = num14;
            right = num13;
        }
        7 => {
            // `:26720-26802` — the sun dance's arm ladder. num10 = 40
            // expert / 60 classic, num12 = 5; the running offset num11.
            let num10 = if expert { 40 } else { 60 };
            let num12 = 5;
            // Each arm breaks out of the switch in the C#; the port uses
            // an early return from a closure to keep the ladder shape.
            let pick = || -> (i32, i32) {
                let mut num11 = 0;
                if num8 < (num11 + num12) as f32 {
                    return (f2, f0);
                }
                num11 += num12;
                if num8 < (num11 + num10 - num12) as f32 {
                    return (f3, f0);
                }
                num11 += num10 - num12;
                if num8 < (num11 + num12) as f32 {
                    return (f3, f2);
                }
                num11 += num12;
                if num8 < (num11 + num10 - num12) as f32 {
                    return (f3, f3);
                }
                num11 += num10 - num12;
                if num8 < (num11 + num10) as f32 {
                    return (f4, f3);
                }
                num11 += num10;
                if num8 < (num11 + num10) as f32 {
                    return (f4, f4);
                }
                num11 += num10;
                if expert {
                    if num8 < (num11 + num12) as f32 {
                        return (f3, f4);
                    }
                    num11 += num12;
                    if num8 < (num11 + num10 - num12) as f32 {
                        return (f1, f4);
                    }
                    num11 += num10 - num12;
                    if num8 < (num11 + num12) as f32 {
                        return (f1, f3);
                    }
                    num11 += num12;
                    if num8 < (num11 + num10 - num12) as f32 {
                        return (f1, f1);
                    }
                    num11 += num10 - num12;
                }
                if num8 >= num11 as f32 {
                    return (f2, f2);
                }
                // The C# falls through with the initial 0/0 (no arm
                // wrote before the final gate — the expert tail's last
                // increment leaves num11 > num8 only in the gap the
                // ladder above already covered).
                (f0, f0)
            };
            let (l, r) = pick();
            left = l;
            right = r;
        }
        // case 1 / 3: no writes (`:26803-26805`); default: none.
        _ => {}
    }
    (7, left, right)
}

/// The state-8/9/10 white-lerp + fade factors (`Main.cs:26504-26522`):
/// `(num4, num7, num8, num9, num5, num6)` — the body colour lerps toward
/// White by num7, num4 scales the glow, num5/num6 are the ghost counts.
pub(crate) fn hallow_boss_state_factors(ai0: f32, ai1: f32) -> (f32, f32, f32, f32, i32, i32) {
    let (mut num4, mut num5, mut num6) = (1.0f32, 0i32, 0i32);
    let (mut num7, mut num8, mut num9) = (0.0f32, 0.0f32, 0.0f32);
    let num = ai0 as i32;
    if num == 8 || num == 9 {
        // `:26506-26512`.
        num7 = get_lerp_value(0.0, 30.0, ai1, true) * get_lerp_value(90.0, 30.0, ai1, true);
        num8 = get_lerp_value(0.0, 30.0, ai1, true) * get_lerp_value(90.0, 70.0, ai1, true);
        num9 = get_lerp_value(0.0, 15.0, ai1, true) * get_lerp_value(45.0, 30.0, ai1, true);
        num4 *= 1.0 - num9;
        num5 = 4;
        num6 = 3;
    }
    if num == 10 {
        // `:26516-26521`.
        num7 = get_lerp_value(30.0, 90.0, ai1, true) * get_lerp_value(165.0, 90.0, ai1, true);
        num8 = get_lerp_value(0.0, 60.0, ai1, true) * get_lerp_value(180.0, 120.0, ai1, true);
        num9 = get_lerp_value(0.0, 60.0, ai1, true) * get_lerp_value(180.0, 120.0, ai1, true);
        num4 *= 1.0 - num9;
        num6 = 4;
    }
    (num4, num7, num8, num9, num5, num6)
}

/// The body colour after the state white-lerp (`:26509`/`:26519`):
/// `Color.Lerp(color, White, num7)` over the GetAlpha result.
pub(crate) fn hallow_boss_body_color(alpha_color: [u8; 4], num7: f32) -> [u8; 4] {
    lerp_bytes(alpha_color, [255; 4], num7)
}

/// One halo-ring ghost's offset (`Main.cs:26627-26629`/`:26636-26638`):
/// `rotation.ToRotationVector2().RotatedBy(num15 * 2π + π/4) *
/// MathHelper.Lerp(2, 8, num14)`. Returns the (dx, dy) added to the
/// centre draw position. `num15` walks 0, 0.25, 0.5, 0.75.
pub(crate) fn empress_halo_offset(rotation: f32, num15: f32, num14: f32) -> [f32; 2] {
    let angle = rotation + num15 * std::f32::consts::TAU + std::f32::consts::FRAC_PI_4;
    let radius = lerp_f(2.0, 8.0, num14);
    let (s, c) = angle.sin_cos();
    // `ToRotationVector2` = (cos, sin) — XNA's Y-down convention.
    [c * radius, s * radius]
}

/// The state-8/9 ghost ring (`Main.cs:26533-26543`): the i-th ghost's
/// hue, luminance, colour and spin offset. `i` runs -num6..=num6+num5
/// skipping 0. The 3-D spin (`Matrix.CreateRotationX/Y/Z` over
/// `Vector3.Forward`) is reproduced as the composed rotation applied to
/// (0,0,-1) — XNA's `Vector3.Forward` is (0,0,-1).
pub(crate) fn hallow_boss_dash_ghost(
    i: i32,
    ai1: f32,
    time_for_visual_effects: f32,
    num7: f32,
    num8: f32,
) -> ([u8; 4], [f32; 2]) {
    // `:26535-26542`.
    let hue = (i as f32 + 5.0) / 10.0;
    let mut num10 = 200.0f32;
    let num11 = time_for_visual_effects / 60.0;
    // The composed X/Y/Z rotation of Vector3.Forward (0,0,-1). XNA
    // matrices are row-vector; the port composes the same three
    // rotations in the C#'s multiplication order (X * Y * Z applied to
    // the row vector = Z then Y then X on the column form).
    let ax = (num11 - 0.3 + i as f32 * 0.1) * 0.7 * std::f32::consts::TAU;
    let ay = (num11 - 0.8 + i as f32 * 0.3) * 0.7 * std::f32::consts::TAU;
    let az = (num11 + i as f32 * 0.5) * 0.1 * std::f32::consts::TAU;
    let v = rot_xyz_forward(ax, ay, az);
    num10 += get_lerp_value(-1.0, 1.0, v[2], true) * 150.0;
    let spin = [v[0] * num10 * num7, v[1] * num10 * num7];
    let mut color2 = hsl_bytes(
        hue,
        1.0,
        lerp_f(0.5, 1.0, get_lerp_value(90.0, 0.0, ai1, true)),
    );
    color2 = xna_scale_color(color2, 0.8 * num8);
    color2[3] /= 3; // `color2.A /= 3` — the byte integer divide.
    // `position += spinningpoint.RotatedBy(ai[1] / 180 * 2π)` (`:26543`).
    let ra = ai1 / 180.0 * std::f32::consts::TAU;
    let (s, c) = ra.sin_cos();
    let off = [spin[0] * c - spin[1] * s, spin[0] * s + spin[1] * c];
    (color2, off)
}

/// `Vector3.Transform(Forward, CreateRotationX(ax) * CreateRotationY(ay) *
/// CreateRotationZ(az))` — XNA row-vector convention: the row vector
/// (0,0,-1) times the product matrix applies X first, then Y, then Z.
fn rot_xyz_forward(ax: f32, ay: f32, az: f32) -> [f32; 3] {
    let (sx, cx) = ax.sin_cos();
    let (sy, cy) = ay.sin_cos();
    let (sz, cz) = az.sin_cos();
    // v = (0,0,-1) * Rx: XNA CreateRotationX is
    // [1 0 0; 0 cx sx; 0 -sx cx] (row-major, row-vector multiply):
    // v1 = (0, (-1)*(-sx), (-1)*cx) = (0, sx, -cx).
    let v1 = [0.0, sx, -cx];
    // Ry = [cy 0 -sy; 0 1 0; sy 0 cy].
    // v2 = (v1x*cy + v1z*sy, v1y, -v1x*sy + v1z*cy).
    let v2 = [v1[0] * cy + v1[2] * sy, v1[1], -v1[0] * sy + v1[2] * cy];
    // Rz = [cz sz 0; -sz cz 0; 0 0 1].
    // v3 = (v2x*cz - v2y*sz, v2x*sz + v2y*cz, v2z).
    [v2[0] * cz - v2[1] * sz, v2[0] * sz + v2[1] * cz, v2[2]]
}

/// The state-10 ghost ring (`Main.cs:26545-26562`): the i-th ghost's
/// position offset + colour. `i` runs -num6..=num6+num5 skipping 0.
pub(crate) fn hallow_boss_sun_ghost(
    i: i32,
    ai1: f32,
    time_for_visual_effects: f32,
    num7: f32,
    num8: f32,
    num4: f32,
) -> ([u8; 4], [f32; 2]) {
    let off = if ai1 >= 90.0 {
        // `:26549-26555` — the elliptical orbit.
        let num12 = time_for_visual_effects / 90.0;
        let mut num13 = i;
        if num13 < 0 {
            num13 += 1;
        }
        let angle =
            (num13 as f32 + 0.5) * std::f32::consts::FRAC_PI_4 + std::f32::consts::TAU * num12;
        let (s, c) = angle.sin_cos();
        [c * 600.0 * num7, s * 150.0 * num7]
    } else {
        // `:26559` — the horizontal fan.
        [200.0 * i as f32 * num7, 0.0]
    };
    // `:26561-26562` — White * 0.8 * num8 * num4, A /= 3.
    let mut color2 = xna_scale_color([255; 4], 0.8 * num8 * num4);
    color2[3] /= 3;
    (color2, off)
}

/// The post-loop trail ghosts (`Main.cs:26564-26572`): the i-th ghost
/// (i > num6) rides `velocity * -3 * (i - 4) * lerpValue` and its colour
/// scales by `1 - num9`. Returns the offset; the caller scales the
/// ghost colour.
pub(crate) fn hallow_boss_trail_offset(i: i32, ai1: f32, vx: f32, vy: f32) -> Option<[f32; 2]> {
    let lerp_value = get_lerp_value(30.0, 70.0, ai1, true);
    if lerp_value == 0.0 {
        return None; // `continue` (`:26567-26570`).
    }
    Some([vx * -3.0 * (i as f32 - 4.0) * lerp_value, vy * -3.0 * (i as f32 - 4.0) * lerp_value])
}

/// The wing-strip frame (`Main.cs:26478`/`:26625`):
/// `(int)(localAI[0] / 4) % 11` for Extra_159, `% 8` for Extra_187.
pub(crate) fn hallow_boss_wing_frame(local_ai0: f32, rows: i32) -> i32 {
    (local_ai0 / 4.0) as i32 % rows
}

/// The 661 Lacewing's afterimage ring (`Main.cs:23270-23275` +
/// `:23429-23440`): six hue-cycled ghosts on a radius
/// `(2 + ai[2]) * num56 + 2` ring, the ring angle offset
/// `lerp(0, 3, GetLerpValue(0, 60, ai[2]))`, each ghost's colour
/// `GetAlpha(Color(127-alpha, .., 0).MultiplyRGBA(hslToRgb(
/// (GlobalTimeWrappedHourly + j/6) % 1, 1, 0.5))) * (1 - num56*0.5)`,
/// A=0. `num56 = max(cos(t % 2.4 / 2.4 * 2π)/2 + 0.5,
/// GetLerpValue(0, 60, ai[2]))`. Returns (num56, ring_angle, per-ghost
/// colour bytes, radius) for j in 0..6.
pub(crate) fn lacewing_afterimage(
    global_time_wrapped_hourly: f32,
    ai2: f32,
    alpha: u8,
    light: [u8; 3],
    j: u32,
) -> ([u8; 4], f32, f32, f32) {
    let num56 = ((global_time_wrapped_hourly % 2.4 / 2.4 * std::f32::consts::TAU).cos() / 2.0
        + 0.5)
        .max(get_lerp_value(0.0, 60.0, ai2, true));
    let num58 = lerp_f(0.0, 3.0, get_lerp_value(0.0, 60.0, ai2, true));
    let base = [
        127u8.saturating_sub(alpha),
        127u8.saturating_sub(alpha),
        127u8.saturating_sub(alpha),
        0,
    ];
    let hue = hsl_bytes((global_time_wrapped_hourly + j as f32 / 6.0) % 1.0, 1.0, 0.5);
    let multiplied = xna_multiply_rgba(base, hue);
    // `GetAlpha` over the multiplied colour: the generic arm's
    // `(255 - alpha)/255` scale on RGB + `A - alpha` (NPC.cs:93412-93416).
    let fade = (255 - alpha as i32) as f32 / 255.0;
    let mut alpha3 = [
        (multiplied[0] as f32 * fade) as u8,
        (multiplied[1] as f32 * fade) as u8,
        (multiplied[2] as f32 * fade) as u8,
        (multiplied[3] as i32 - alpha as i32).max(0) as u8,
    ];
    alpha3 = xna_scale_color(alpha3, 1.0 - num56 * 0.5);
    alpha3[3] = 0; // `alpha3.A = 0` (`:23436`).
    let _ = light; // the ring colour does not read the light (the mask shape).
    let angle = j as f32 / 6.0 * std::f32::consts::TAU + num58;
    let radius = (2.0 + ai2) * num56 + 2.0;
    (alpha3, angle, radius, num56)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The OurFavoriteColor static is the UI gold, NOT a hue cycle
    /// (`Main.cs:66748` — the only write in the decompile).
    #[test]
    fn the_favorite_color_is_the_static_gold() {
        assert_eq!(OUR_FAVORITE_COLOR, [255, 231, 69, 255]);
    }

    /// The 636 GetAlpha arm: Lerp(light, White, 0.25) * Opacity
    /// (`NPC.cs:93501-93505`), the two-step truncation.
    #[test]
    fn the_636_get_alpha_lerps_a_quarter_white_then_scales() {
        // Light (100, 100, 100): lerp 0.25 toward 255 → 138.75 → 138.
        // Opacity 1 → (138, 138, 138, 255).
        assert_eq!(empress_get_alpha([100, 100, 100], 1.0), [138, 138, 138, 255]);
        // Opacity 0.5 → (69, 69, 69, 127).
        assert_eq!(empress_get_alpha([100, 100, 100], 0.5), [69, 69, 69, 127]);
        // Full light stays white at opacity 1.
        assert_eq!(empress_get_alpha([255, 255, 255], 1.0), [255; 4]);
        // Opacity 0 zeroes every channel (the materialize fade).
        assert_eq!(empress_get_alpha([255, 255, 255], 0.0), [0; 4]);
    }

    /// The enrage glow swaps the hue cycle for OurFavoriteColor ×0.3
    /// (`Main.cs:26615-26620`); A stays 0 through both scales.
    #[test]
    fn the_enraged_glow_is_the_favorite_color_at_three_tenths() {
        let c = empress_glow_color(0.0, true, 1.0, 1.0);
        // (255, 231, 69) * 0.3 = (76.5, 69.3, 20.7) → (76, 69, 20), A 0.
        assert_eq!(c, [76, 69, 20, 0]);
        // num4 * Opacity scales again: 0.5 → (38, 34, 10).
        let c2 = empress_glow_color(0.0, true, 0.5, 1.0);
        assert_eq!(c2, [38, 34, 10, 0]);
    }

    /// The non-enraged glow hue-cycles in the 0.6..0.68 band
    /// (`Main.cs:26611-26614`): num14 ∈ [0,1] → hue ∈ [0.6, 0.68].
    #[test]
    fn the_calm_glow_hue_stays_in_the_blue_band() {
        for t in [0.0f32, 0.25, 0.5, 1.0, 1.7, 3.0] {
            let pulse = empress_glow_pulse(t);
            assert!((0.0..=1.0).contains(&pulse), "pulse {pulse}");
            let hue = (pulse * 0.08 + 0.6) % 1.0;
            assert!((0.6..=0.68).contains(&hue), "hue {hue}");
            let c = empress_glow_color(t, false, 1.0, 1.0);
            assert_eq!(c[3], 0, "the glow keeps A=0 (additive)");
            // ×0.6 of a 0.5-luminance hue: every channel ≤ 153.
            assert!(c[..3].iter().all(|&v| v <= 153));
        }
        // t=0.5 → sin(π/2)=1 → num14=1 → hue 0.68; t=1.5 → num14=0 → 0.6.
        assert_eq!(empress_glow_pulse(0.5), 1.0);
        assert!((empress_glow_pulse(1.5) - 0.0).abs() < 1e-6);
    }

    /// The arm-frame picker's state ladders (`Main.cs:26668-26806`).
    #[test]
    fn the_arm_frames_walk_the_vanilla_ladders() {
        // case 6: <6 → 2/2, <174 → 3/3, <180 → 2/2, else 0/0.
        assert_eq!(hallow_boss_arm_frame(6.0, 0.0, 0.0, false), (7, 2, 2));
        assert_eq!(hallow_boss_arm_frame(6.0, 100.0, 0.0, false), (7, 3, 3));
        assert_eq!(hallow_boss_arm_frame(6.0, 175.0, 0.0, false), (7, 2, 2));
        assert_eq!(hallow_boss_arm_frame(6.0, 180.0, 0.0, false), (7, 0, 0));
        // case 0: <106 → 1/1, <110 → 2/2, else 0/0.
        assert_eq!(hallow_boss_arm_frame(0.0, 50.0, 0.0, false), (7, 1, 1));
        assert_eq!(hallow_boss_arm_frame(0.0, 107.0, 0.0, false), (7, 2, 2));
        assert_eq!(hallow_boss_arm_frame(0.0, 110.0, 0.0, false), (7, 0, 0));
        // case 2/11: left only — <5 → 2, <65 → 3, else 2.
        assert_eq!(hallow_boss_arm_frame(2.0, 0.0, 0.0, false), (7, 2, 0));
        assert_eq!(hallow_boss_arm_frame(11.0, 30.0, 0.0, false), (7, 3, 0));
        assert_eq!(hallow_boss_arm_frame(2.0, 65.0, 0.0, false), (7, 2, 0));
        // case 5: right only.
        assert_eq!(hallow_boss_arm_frame(5.0, 30.0, 0.0, false), (7, 0, 3));
        // case 4/10: both.
        assert_eq!(hallow_boss_arm_frame(4.0, 30.0, 0.0, false), (7, 3, 3));
        // case 1/3: no writes.
        assert_eq!(hallow_boss_arm_frame(1.0, 50.0, 0.0, false), (7, 0, 0));
        assert_eq!(hallow_boss_arm_frame(3.0, 50.0, 0.0, false), (7, 0, 0));
    }

    /// The dash arms' ai[3]-sign swap (`:26690-26717`): ai3=1 swaps the
    /// LEFT frame to 1 (and 5→6), ai3=-1 the RIGHT.
    #[test]
    fn the_dash_arms_swap_on_the_ai3_sign() {
        // ai1=25 → num9=2 (neither <10 nor <20, and `!(num8<30)` is false,
        // so the else = num3 = 2; `Main.cs:26690-26696`); ai3=1 → left 1.
        assert_eq!(hallow_boss_arm_frame(8.0, 25.0, 1.0, false), (7, 1, 2));
        // ai3=-1 → left 2, right 1.
        assert_eq!(hallow_boss_arm_frame(9.0, 25.0, -1.0, false), (7, 2, 1));
        // ai3=0 → no swap.
        assert_eq!(hallow_boss_arm_frame(8.0, 25.0, 0.0, false), (7, 2, 2));
        // ai1 ≥ 30 → num9=5 (`!(num8 < 30)` true), so the <30 SWAP gate is
        // off; the 5→6 promotion is NOT gated, so ai3=1 still lifts the right
        // arm (num13==5 → 6, `Main.cs:26705-26714`).
        assert_eq!(hallow_boss_arm_frame(8.0, 30.0, 1.0, false), (7, 5, 6));
        // ai1=5 → num9=2; ai3=1 → left 1 (not 5, so no 5→6 promotion).
        assert_eq!(hallow_boss_arm_frame(8.0, 5.0, 1.0, false), (7, 1, 2));
    }

    /// The sun-dance ladder (`:26720-26802`): classic num10=60, expert 40.
    #[test]
    fn the_sun_dance_ladder_matches_both_difficulties() {
        // Classic: <5 → (2,0); <60 → (3,0); <65 → (3,2); <120 → (3,3);
        // <180 → (4,3); <240 → (4,4); ≥240 → (2,2).
        assert_eq!(hallow_boss_arm_frame(7.0, 0.0, 0.0, false), (7, 2, 0));
        assert_eq!(hallow_boss_arm_frame(7.0, 30.0, 0.0, false), (7, 3, 0));
        assert_eq!(hallow_boss_arm_frame(7.0, 62.0, 0.0, false), (7, 3, 2));
        assert_eq!(hallow_boss_arm_frame(7.0, 100.0, 0.0, false), (7, 3, 3));
        assert_eq!(hallow_boss_arm_frame(7.0, 150.0, 0.0, false), (7, 4, 3));
        assert_eq!(hallow_boss_arm_frame(7.0, 200.0, 0.0, false), (7, 4, 4));
        assert_eq!(hallow_boss_arm_frame(7.0, 240.0, 0.0, false), (7, 2, 2));
        // Expert: the extra tail arms at 160..240.
        assert_eq!(hallow_boss_arm_frame(7.0, 162.0, 0.0, true), (7, 3, 4));
        assert_eq!(hallow_boss_arm_frame(7.0, 180.0, 0.0, true), (7, 1, 4));
        assert_eq!(hallow_boss_arm_frame(7.0, 202.0, 0.0, true), (7, 1, 3));
        assert_eq!(hallow_boss_arm_frame(7.0, 220.0, 0.0, true), (7, 1, 1));
        assert_eq!(hallow_boss_arm_frame(7.0, 240.0, 0.0, true), (7, 2, 2));
    }

    /// The state factors (`:26504-26522`): the dash's num5=4/num6=3, the
    /// transform's num6=4, the fades multiply.
    #[test]
    fn the_state_factors_carry_the_ghost_counts() {
        let (num4, num7, _num8, num9, num5, num6) = hallow_boss_state_factors(8.0, 15.0);
        assert_eq!((num5, num6), (4, 3));
        // ai1=15: num7 = 0.5 * 1 = 0.5; num9 = 1 * 1 = 1 → num4 = 0.
        assert_eq!(num7, 0.5);
        assert_eq!(num9, 1.0);
        assert_eq!(num4, 0.0);
        let (_, _, _, _, num5, num6) = hallow_boss_state_factors(10.0, 90.0);
        assert_eq!((num5, num6), (0, 4));
        // No state → no ghosts, no fade.
        assert_eq!(hallow_boss_state_factors(2.0, 30.0), (1.0, 0.0, 0.0, 0.0, 0, 0));
    }

    /// The halo ring's four offsets (`:26627-26629`): radius lerp 2..8,
    /// the π/4 phase, the rotation carry.
    #[test]
    fn the_halo_ring_walks_four_quarters() {
        // rotation 0, num14=0 → radius 2; the first ghost at π/4.
        let o = empress_halo_offset(0.0, 0.0, 0.0);
        let expect = 2.0 * std::f32::consts::FRAC_PI_4.cos();
        assert!((o[0] - expect).abs() < 1e-5);
        assert!((o[1] - expect).abs() < 1e-5);
        // num14=1 → radius 8.
        let o = empress_halo_offset(0.0, 0.0, 1.0);
        assert!((o[0].hypot(o[1]) - 8.0).abs() < 1e-5);
        // The four ghosts are 90° apart.
        let a = empress_halo_offset(0.0, 0.0, 0.5);
        let b = empress_halo_offset(0.0, 0.25, 0.5);
        assert!((a[0] * b[0] + a[1] * b[1]).abs() < 1e-4, "orthogonal");
    }

    /// The wing frame counters (`:26478`/`:26625`): /4 then %11 / %8.
    #[test]
    fn the_wing_frames_divide_by_four_then_wrap() {
        assert_eq!(hallow_boss_wing_frame(0.0, 11), 0);
        assert_eq!(hallow_boss_wing_frame(16.0, 11), 4);
        assert_eq!(hallow_boss_wing_frame(44.0, 11), 0);
        assert_eq!(hallow_boss_wing_frame(43.0, 8), 2);
        assert_eq!(hallow_boss_wing_frame(32.0, 8), 0);
    }

    /// The dash ghost's colour truncations (`:26541-26542`): ×0.8×num8
    /// then A/=3 (the byte divide).
    #[test]
    fn the_dash_ghost_colour_truncates_twice() {
        let (c, off) = hallow_boss_dash_ghost(1, 15.0, 60.0, 1.0, 1.0);
        // hue 0.6, luminance lerp(0.5,1,GetLerp(90,0,15)=0.8333)=0.9167.
        // The exact bytes ride hsl_to_rgb's banker's rounding; the A
        // channel: 255 * 0.8 = 204, /3 = 68.
        assert_eq!(c[3], 68);
        assert!(c[..3].iter().all(|&v| v > 0));
        // The offset is finite and rides the spin radius (200..350).
        assert!(off[0].is_finite() && off[1].is_finite());
    }

    /// The sun ghost's two position arms (`:26547-26560`).
    #[test]
    fn the_sun_ghost_fans_then_orbits() {
        // ai1 < 90: the horizontal fan, 200 * i * num7.
        let (_, off) = hallow_boss_sun_ghost(2, 30.0, 0.0, 0.5, 1.0, 1.0);
        assert_eq!(off, [200.0, 0.0]);
        // The negative-i skip: i=-1 → num13=0 → the (0.5)·π/4 phase.
        let (_, off) = hallow_boss_sun_ghost(-1, 90.0, 0.0, 1.0, 1.0, 1.0);
        let expect_a = 0.5 * std::f32::consts::FRAC_PI_4;
        assert!((off[0] - expect_a.cos() * 600.0).abs() < 1e-3);
        assert!((off[1] - expect_a.sin() * 150.0).abs() < 1e-3);
    }

    /// The trail ghosts (`:26564-26572`): the 30..70 lerp gate, the
    /// velocity ×-3×(i-4) offset.
    #[test]
    fn the_trail_ghosts_gate_on_the_30_70_lerp() {
        assert!(hallow_boss_trail_offset(5, 30.0, 10.0, 0.0).is_none());
        let off = hallow_boss_trail_offset(5, 50.0, 10.0, 0.0).unwrap();
        // lerp = 0.5, i-4 = 1 → -15.
        assert_eq!(off, [-15.0, 0.0]);
        assert!(hallow_boss_trail_offset(5, 70.0, 10.0, 0.0).is_some());
    }

    /// The Lacewing ring (`:23270-23275`/`:23429-23440`): six ghosts,
    /// A=0, the radius rides ai[2], the hue walks j/6.
    #[test]
    fn the_lacewing_ring_is_six_additive_hues() {
        let (c0, a0, r0, num56) = lacewing_afterimage(0.0, 0.0, 0, [255; 3], 0);
        assert_eq!(c0[3], 0, "A=0 (additive)");
        assert!(num56 >= 0.0 && num56 <= 1.0);
        // ai2=0, t=0: num56 = cos(0)/2+0.5 = 1 → radius 2*1+2 = 4.
        assert_eq!(r0, 4.0);
        assert_eq!(a0, 0.0);
        // j=1 is 60° around.
        let (_, a1, _, _) = lacewing_afterimage(0.0, 0.0, 0, [255; 3], 1);
        assert!((a1 - std::f32::consts::TAU / 6.0).abs() < 1e-6);
        // ai2=60 raises the ring angle to 3 and the radius to 62*num56+2.
        let (_, _, r60, _) = lacewing_afterimage(0.0, 60.0, 0, [255; 3], 0);
        assert_eq!(r60, 62.0 * 1.0 + 2.0);
        // The alpha mask: alpha=255 → the 127-alpha base is 0 → black.
        let (c255, _, _, _) = lacewing_afterimage(0.0, 0.0, 255, [255; 3], 0);
        assert_eq!(&c255[..3], &[0, 0, 0]);
    }

    /// The 3-D spin reproduces XNA's row-vector Forward transform: at
    /// all-zero angles Forward stays (0,0,-1).
    #[test]
    fn the_spin_at_zero_angles_keeps_forward() {
        let v = rot_xyz_forward(0.0, 0.0, 0.0);
        assert!((v[0]).abs() < 1e-6);
        assert!((v[1]).abs() < 1e-6);
        assert!((v[2] + 1.0).abs() < 1e-6);
        // A pure X rotation by π/2 maps Forward (0,0,-1) to (0,1,0)
        // (XNA's row-vector Rx: y' = z*sx → -1*1 = ... the sign check).
        let v = rot_xyz_forward(std::f32::consts::FRAC_PI_2, 0.0, 0.0);
        assert!(v[0].abs() < 1e-6);
        assert!((v[1] - 1.0).abs() < 1e-5, "y {}", v[1]);
        assert!(v[2].abs() < 1e-5, "z {}", v[2]);
    }
}
