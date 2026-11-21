//! W4-d16 — the kite string + tail geometry (`Main.DrawKite`,
//! `Main.cs:37134-37569`, the `DrawProjDirect` aiStyle-160 arm
//! `:28054-28058`). The pure-math half of the draw (the `yoyo_string.rs`
//! precedent): the per-kind tail parameter table (`:37161-37341`), the
//! 12-px string walk with the velocity-swap sag (`:37356-37453`) and the
//! per-tail ribbon point chains over the `oldPos` history ring
//! (`:37454-37534`). The sprite emission lives in
//! `Projectile::draw_kite` (`projectile.rs`).
//!
//! Deviations (ANALYSIS "W4-d16 kite residuals"): the C# kites carry
//! `TrailCacheLength` 60/65/80 (`ProjectileID.cs:522`); the port's fixed
//! 10-entry ring is the recorded trail-density deviation (the 640
//! precedent) — ring indices past the port's depth read as the unwritten
//! zero entries (the tail collapses to the head, exactly like the C#'s
//! first frames). The `num18` wind-remap's first two assignments
//! (`:37157-37158`) are dead stores — the final `Remap(localAI[0], 0,
//! 300, 0, 1)` (`:37159`) is the live value; the no-tail kinds' `num6`
//! spin (`:37331`) is zeroed on the next line (`:37332`) — both fold to
//! their final values. CODE-ONLY MODE: the witnesses are WRITTEN NOT RUN.

use glam::Vec2;

use crate::projectile::get_lerp_value;

/// `Utils.Remap` clamped (`Utils.cs`'s `GetLerpValue`-based form).
fn remap(value: f32, from_min: f32, from_max: f32, to_min: f32, to_max: f32) -> f32 {
    to_min + (to_max - to_min) * get_lerp_value(from_min, from_max, value)
}

/// The per-kind tail parameters (`Main.cs:37138-37341`'s locals + switch).
/// Field names carry the C# local numbers for the citation walk.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct KiteTailParams {
    /// `value2` — the `TextureAssets.Extra[id]` tail sheet.
    pub extra: u16,
    /// `num` — the tail sheet's frame count.
    pub frames: i32,
    /// `num12` — the frame-wrap restart column.
    pub wrap_start: i32,
    /// `num2`'s ±π/2-by-direction flag (the kinds that set it).
    pub quarter_turn: bool,
    /// `num3` — the segments per tail.
    pub segs: i32,
    /// `num4` — the history-ring stride.
    pub stride: i32,
    /// `num5` — the max segment length (the −1 sentinel never occurs).
    pub max_len: f32,
    /// `num6`'s per-direction sway coefficient (π/8 or π/12; 0 = none).
    pub sway: f32,
    /// `num7` — the tail anchor's X offset.
    pub x_off: f32,
    /// `num8` — the tail anchor's Y offset.
    pub y_off: f32,
    /// `num9` — the segment index where `num5` switches to `num10`.
    pub switch_at: i32,
    /// `num10` — the switched max length.
    pub alt_len: f32,
    /// `num11` — the tail's X extent (the wind stretch).
    pub extent: f32,
    /// `num13` — the tail count.
    pub tails: i32,
    /// `num14` — the second X offset add.
    pub x_off2: f32,
    /// `num15` — the second Y offset add.
    pub y_off2: f32,
    /// `flag` — draw the FishingLine links between the tail points.
    pub link: bool,
    /// `flag2` — the wrap restart subtracts one column.
    pub wrap_minus1: bool,
    /// `num16` — the calm-rotation coefficient.
    pub calm_rot: f32,
}

/// The switch (`Main.cs:37161-37341`) over the stock defaults
/// (`:37138-37155`). `None` for non-kite kinds.
pub(crate) fn kite_tail_params(kind: u16) -> Option<KiteTailParams> {
    // The stock locals (`:37138-37155`): Extra[103], 15 frames, 10 segs,
    // stride 5, len 10, (−14, −2), no switch, extent 8, 1 tail, no adds,
    // link on, no wrap-minus, no calm rot.
    let mut p = KiteTailParams {
        extra: 103, frames: 15, wrap_start: 0, quarter_turn: false,
        segs: 10, stride: 5, max_len: 10.0, sway: 0.0,
        x_off: -14.0, y_off: -2.0, switch_at: -1, alt_len: -1.0,
        extent: 8.0, tails: 1, x_off2: 0.0, y_off2: 0.0,
        link: true, wrap_minus1: false, calm_rot: 0.0,
    };
    match kind {
        771 => {
            p.extra = 104; p.frames = 12; p.wrap_start = 12;
            p.quarter_turn = true; p.segs = 12; p.max_len = 22.0;
            p.sway = std::f32::consts::PI / 8.0;
            p.x_off = -8.0; p.y_off = -6.0; p.switch_at = 10; p.alt_len = 8.0;
            p.extent = 12.0; p.calm_rot = std::f32::consts::PI / 16.0;
        }
        822 => {
            p.extra = 132; p.frames = 7; p.wrap_start = 7;
            p.quarter_turn = true; p.segs = 7; p.max_len = 22.0;
            p.sway = std::f32::consts::PI / 12.0;
            p.x_off = -20.0; p.y_off = -6.0; p.extent = 12.0;
            p.calm_rot = std::f32::consts::PI / 16.0;
        }
        850 => {
            p.extra = 147; p.frames = 8; p.wrap_start = 8;
            p.quarter_turn = true; p.segs = 8; p.max_len = 22.0;
            p.sway = std::f32::consts::PI / 12.0;
            p.x_off = -20.0; p.y_off = -38.0; p.extent = 12.0;
            p.x_off2 = 6.0; p.y_off2 = 2.0; p.link = false;
            p.calm_rot = std::f32::consts::PI * 5.0 / 32.0;
        }
        823 | 846 => {
            p.extra = if kind == 846 { 146 } else { 133 };
            p.frames = 6; p.wrap_start = 6;
            p.quarter_turn = true; p.segs = 6; p.max_len = 40.0;
            p.sway = std::f32::consts::PI / 12.0;
            p.x_off = -16.0; p.y_off = -10.0; p.extent = 30.0;
            p.calm_rot = std::f32::consts::PI * 3.0 / 16.0;
        }
        827 | 844 => {
            p.extra = if kind == 844 { 144 } else { 135 };
            p.frames = 4; p.wrap_start = 3;
            p.quarter_turn = true; p.segs = 16; p.max_len = 10.0;
            p.sway = std::f32::consts::PI / 12.0;
            p.y_off = -4.0; p.stride = 4; p.wrap_minus1 = true;
            p.calm_rot = std::f32::consts::PI / 32.0;
        }
        828 | 829 => {
            p.extra = if kind == 829 { 137 } else { 136 };
            p.frames = 2; p.wrap_start = 1;
            p.quarter_turn = true; p.segs = 6; p.max_len = 10.0;
            p.sway = std::f32::consts::PI / 12.0;
            p.extent = 10.0; p.y_off = -4.0; p.tails = 3;
            p.link = false; p.wrap_minus1 = true;
            p.calm_rot = std::f32::consts::PI * 3.0 / 16.0;
        }
        852 => {
            p.extra = 148; p.frames = 2; p.wrap_start = 1;
            p.quarter_turn = true; p.segs = 6; p.max_len = 10.0;
            p.sway = std::f32::consts::PI / 12.0;
            p.extent = 10.0; p.y_off = -4.0; p.tails = 4;
            p.link = false; p.wrap_minus1 = true;
            p.calm_rot = std::f32::consts::PI * 5.0 / 16.0;
        }
        830 | 838 => {
            p.extra = if kind == 838 { 139 } else { 138 };
            p.frames = 3; p.wrap_start = 3;
            p.quarter_turn = true; p.segs = 3; p.max_len = 60.0;
            p.alt_len = 40.0; p.switch_at = 3;
            p.sway = std::f32::consts::PI / 12.0;
            p.extent = 50.0; p.y_off = -20.0; p.x_off2 = -10.0;
            p.calm_rot = std::f32::consts::PI / 4.0;
        }
        843 => {
            p.extra = 140; p.frames = 2; p.wrap_start = 2;
            p.quarter_turn = true; p.segs = 2; p.max_len = 30.0;
            p.sway = std::f32::consts::PI / 12.0;
            p.extent = 20.0; p.y_off = -16.0; p.x_off2 = -10.0;
            p.calm_rot = std::f32::consts::PI / 4.0;
        }
        845 => {
            p.extra = 145; p.frames = 3; p.wrap_start = 3;
            p.quarter_turn = true; p.segs = 3; p.max_len = 42.0;
            p.alt_len = 50.0; p.switch_at = 2;
            p.sway = std::f32::consts::PI / 12.0;
            p.extent = 30.0; p.y_off = -8.0; p.x_off2 = -10.0;
        }
        // `:37325-37333` — the no-tail five: num13 = 0; the num6 spin
        // is zeroed on the next line (the dead-store fold, module doc).
        824 | 826 | 839 | 840 | 853 => p.tails = 0,
        // `:37334-37340` — the five colour kites keep the stock tail;
        // only the wrap column moves ((type − 766)·3 + 3).
        766..=770 => p.wrap_start = (kind as i32 - 766) * 3 + 3,
        _ => return None,
    }
    Some(p)
}

/// One FishingLine string segment (`Main.cs:37447-37452`): the world
/// anchor, the rotation (`ToRotation() − π/2`), the source height
/// (0 = the full sheet — only the FINAL segment takes `(int)num24`)
/// and the distance shade (`Remap(1 − d/num21, 0, 1, 0.5, 1)`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct KiteStringSeg {
    pub x: f32,
    pub y: f32,
    pub rot: f32,
    pub height: f32,
    pub shade: f32,
}

/// The string walk (`Main.cs:37356-37453`). Returns the segments and the
/// FINAL `vector2` (the tail loop's `num30` gate reads its Y, `:37457`).
/// The velocity swap (`:37362-37365`) feeds the sag only.
pub(crate) fn kite_string_walk(
    center: Vec2,
    anchor: Vec2,
    vel: Vec2,
) -> (Vec<KiteStringSeg>, Vec2) {
    let mut velocity = vel;
    if velocity.x.abs() > velocity.y.abs() {
        std::mem::swap(&mut velocity.x, &mut velocity.y);
    }
    let mut vector = anchor;
    let mut vector2 = center - vector;
    let num20 = vector2.length();
    let mut num21 = center.distance(anchor);
    if num21 == 0.0 {
        num21 = 1.0;
    }
    let mut segs = Vec::new();
    if num20 == 0.0 {
        return (segs, vector2);
    }
    vector -= vector2 * (12.0 / num20);
    vector2 = center - vector;
    loop {
        let mut num24 = 12.0f32;
        let num25 = vector2.length();
        let num26 = num25;
        let mut last = false;
        if num25.is_nan() || num25 == 0.0 {
            break;
        }
        if num25 < 20.0 {
            last = true;
            vector += vector2.normalize_or_zero() * num24;
            vector2 = center - vector;
            num24 = vector2.length() + 4.0;
        } else {
            vector += vector2 * (12.0 / num25);
            vector2 = center - vector;
            if num26 > 12.0 {
                // `:37407-37444` — the sag: the speed-damped 0.3 base,
                // the distance ramp, the below/above split.
                let mut num27 = 0.3f32;
                let mut num28 = velocity.x.abs() + velocity.y.abs();
                if num28 > 16.0 {
                    num28 = 16.0;
                }
                num27 *= 1.0 - num28 / 16.0;
                num28 = num26 / 80.0;
                if num28 > 1.0 {
                    num28 = 1.0;
                }
                num27 *= num28;
                if num27 < 0.0 {
                    num27 = 0.0;
                }
                if vector2.y > 0.0 {
                    vector2.y *= 1.0 + num27;
                    vector2.x *= 1.0 - num27;
                } else {
                    num28 = velocity.x.abs() / 3.0;
                    if num28 > 1.0 {
                        num28 = 1.0;
                    }
                    num27 *= num28 - 0.5;
                    if num27 > 0.0 {
                        num27 *= 2.0;
                    }
                    vector2.y *= 1.0 + num27;
                    vector2.x *= 1.0 - num27;
                }
            }
        }
        let rot = vector2.to_angle() - std::f32::consts::FRAC_PI_2;
        let shade = remap(1.0 - center.distance(vector) / num21, 0.0, 1.0, 0.5, 1.0);
        segs.push(KiteStringSeg {
            x: vector.x,
            y: vector.y,
            rot,
            height: if last { num24 as i32 as f32 } else { 0.0 },
            shade,
        });
        if last {
            break;
        }
    }
    (segs, vector2)
}

/// The tail ribbon point chains (`Main.cs:37454-37534`): one `Vec<Vec2>`
/// per tail (`num13`), each `segs + 1` points (the head + the history
/// walk). `frame_w` is the kite sprite frame's width (`rectangle.Width`),
/// `box_half` is `proj.Size / 2` (`vector3`). The mutable locals
/// (`num5/6/8/10/14/15`) persist ACROSS the tail iterations — the C#
/// switch mutates them in place (`:37489-37502`). Ring indices past the
/// port's 10-entry depth read as the unwritten zeros (the module-doc
/// deviation). Returns the chains and the FINAL `num6` (the sprite's
/// rotation add, `:37568`).
#[allow(clippy::too_many_arguments)]
pub(crate) fn kite_tail_chains(
    p: &KiteTailParams,
    center: Vec2,
    box_half: Vec2,
    frame_w: f32,
    rot: f32,
    dir: i32,
    local_ai0: f32,
    local_ai1: f32,
    wind: f32,
    visual_time: f64,
    string_end_y: f32,
    old_pos: &[Vec2],
    old_rot: &[f32],
    old_dir: &[i32],
) -> (Vec<Vec<Vec2>>, f32) {
    let num17 = wind.abs();
    // `:37157-37159` — the dead-store fold: the live num18 is the
    // localAI[0] remap.
    let num18 = remap(local_ai0, 0.0, 300.0, 0.0, 1.0);
    let num19 = local_ai1;
    let num29 = 0.5 + 0.5 * num17; // MathHelper.Lerp — UNCLAMPED
    let mut num30 = num17;
    if string_end_y >= -0.02 && string_end_y < 1.0 {
        num30 = get_lerp_value(0.2, 0.5, num17);
    }
    let mut num5 = p.max_len;
    let mut num6 = p.sway * dir as f32;
    let mut num8 = p.y_off;
    let mut num10 = p.alt_len;
    let mut num14 = p.x_off2;
    let mut num15 = p.y_off2;
    let mut chains = Vec::new();
    for i in 0..p.tails {
        // `:37467-37488` — the per-tail wind vector + offset shifts.
        let bob = (visual_time / 300.0 * std::f64::consts::TAU).sin() as f32;
        let extra = match i {
            1 => 0.5 * (1.0 - num18),
            2 => 1.0 * (1.0 - num18),
            3 => 1.5 * (1.0 - num18),
            _ => 0.0,
        };
        let mut vector4 = Vec2::new(
            num29 * p.extent * dir as f32,
            bob * num30 + extra,
        ) * 2.0;
        let mut num33 = p.x_off + num14;
        let mut num34 = num8 + num15;
        match i {
            1 => { num33 -= 8.0; num34 -= 8.0; }
            2 => { num33 -= 4.0; num34 -= 4.0; }
            3 => { num33 -= 12.0; num34 -= 12.0; }
            _ => {}
        }
        if num18 > 0.0 {
            vector4 = vector4.rotate(Vec2::from_angle(
                num18 * std::f32::consts::FRAC_PI_2 * dir as f32,
            ));
            num8 *= 1.0 - num18;
            num14 *= 1.0 - num18;
            num15 *= 1.0 - num18;
            let num36 = (1.0 - num19) * num18;
            num6 = num6 * (1.0 - num36)
                - std::f32::consts::FRAC_PI_4 * num36 * dir as f32
                + num36 * p.calm_rot * dir as f32;
            num5 *= 1.0 + (0.7 - 1.0) * num18; // Lerp(1, 0.7, num18)
            num10 *= 1.0 + (0.7 - 1.0) * num18;
        }
        let anchor_off = Vec2::new(
            (frame_w * 0.5 + num33) * dir as f32,
            num34,
        );
        let mut vector5 = center + anchor_off.rotate(Vec2::from_angle(rot + num6));
        let mut list = vec![vector5];
        let mut num37 = p.stride;
        let mut num38 = 1;
        while num37 < (p.segs + 1) * p.stride {
            if p.switch_at != -1 && p.switch_at == num38 {
                num5 = num10;
            }
            let idx = num37 as usize;
            let vector6 = old_pos.get(idx).copied().unwrap_or(Vec2::ZERO);
            if vector6 == Vec2::ZERO {
                list.push(vector5);
            } else {
                let spinningpoint = Vec2::new(
                    (frame_w * 0.5 + num33) * old_dir[idx] as f32,
                    num34,
                );
                let num39 = old_rot[idx] + num6;
                let mut vector7 = vector5
                    - (vector6
                        + box_half
                        + spinningpoint.rotate(Vec2::from_angle(num39))
                        + vector4 * (num38 + 1) as f32);
                let num40 = vector7.length();
                if num40 > num5 {
                    vector7 *= num5 / num40;
                }
                let v6 = vector5 - vector7;
                list.push(v6);
                vector5 = v6;
            }
            num37 += p.stride;
            num38 += 1;
        }
        chains.push(list);
    }
    (chains, num6)
}

#[cfg(test)]
mod w4_d16_kite_draw_tests {
    //! W4-d16 — the DrawKite geometry witnesses. WRITTEN NOT RUN
    //! (code-only mode). The parameter table census, the string walk's
    //! 12-px steps + final partial height, the zero-distance refusal,
    //! the tail chain counts + the ring-zero collapse.
    use super::*;

    /// The per-kind table (`Main.cs:37161-37341`): the wyvern's 12-frame
    /// Extra[104], the colour kites' wrap columns, the no-tail five, the
    /// jellyfish trio's 3 tails, the 852's 4.
    #[test]
    fn w4_d16_kite_draw_the_parameter_table() {
        let wyvern = kite_tail_params(771).unwrap();
        assert_eq!((wyvern.extra, wyvern.frames, wyvern.wrap_start), (104, 12, 12));
        assert_eq!((wyvern.segs, wyvern.stride), (12, 5));
        assert_eq!((wyvern.max_len, wyvern.alt_len, wyvern.switch_at), (22.0, 8.0, 10));
        assert!(wyvern.quarter_turn && wyvern.link);
        for k in 766..=770 {
            let p = kite_tail_params(k).unwrap();
            assert_eq!(p.extra, 103, "the stock tail sheet");
            assert_eq!(p.wrap_start, (k as i32 - 766) * 3 + 3);
            assert_eq!(p.tails, 1);
        }
        for k in [824, 826, 839, 840, 853] {
            assert_eq!(kite_tail_params(k).unwrap().tails, 0, "kind {k}");
        }
        assert_eq!(kite_tail_params(828).unwrap().tails, 3);
        assert_eq!(kite_tail_params(829).unwrap().extra, 137);
        assert_eq!(kite_tail_params(852).unwrap().tails, 4);
        assert_eq!(kite_tail_params(844).unwrap().extra, 144);
        assert_eq!(kite_tail_params(846).unwrap().extra, 146);
        assert_eq!(kite_tail_params(838).unwrap().extra, 139);
        let shark = kite_tail_params(830).unwrap();
        assert_eq!((shark.max_len, shark.alt_len, shark.switch_at), (60.0, 40.0, 3));
        assert!(!kite_tail_params(850).unwrap().link);
        assert!(kite_tail_params(827).unwrap().wrap_minus1);
        assert!(kite_tail_params(999).is_none());
    }

    /// The string walk (`:37384-37453`): a 100-px vertical string from
    /// the anchor BELOW the kite walks 12-px steps; the first step backs
    /// off 12 (`:37381`), the final partial segment carries the
    /// `(int)num24` height; the shade ramps 0.5 → 1 toward the kite.
    #[test]
    fn w4_d16_kite_draw_the_string_walk() {
        let center = Vec2::new(100.0, 0.0);
        let anchor = Vec2::new(100.0, 100.0);
        let (segs, end) = kite_string_walk(center, anchor, Vec2::ZERO);
        assert!(!segs.is_empty());
        // Every non-final segment is a full-sheet slice (height 0).
        for s in &segs[..segs.len() - 1] {
            assert_eq!(s.height, 0.0);
        }
        let last = segs.last().unwrap();
        assert!(last.height > 0.0, "the final partial height");
        // The walk ends at the kite (the final vector2 is short).
        assert!(end.length() < 20.0);
        // The shade is monotone toward the kite (the distance remap).
        assert!(segs.first().unwrap().shade <= last.shade + 1e-4);
        // The zero-distance refusal (`:37375-37378`).
        let (none, end0) = kite_string_walk(center, center, Vec2::ZERO);
        assert!(none.is_empty());
        assert_eq!(end0, Vec2::ZERO);
    }

    /// The tail chains (`:37463-37534`): one chain per tail, each
    /// `segs + 1` points; an EMPTY history ring collapses every point to
    /// the head anchor (the C# zero-entry arm, `:37514-37517`).
    #[test]
    fn w4_d16_kite_draw_the_tail_chains() {
        let p = kite_tail_params(766).unwrap();
        let center = Vec2::new(100.0, 100.0);
        let old_pos = [Vec2::ZERO; 10];
        let old_rot = [0.0; 10];
        let old_dir = [0; 10];
        let (chains, num6) = kite_tail_chains(
            &p, center, Vec2::splat(2.0), 30.0, 0.0, 1,
            0.0, 0.0, 0.0, 0.0, 5.0, &old_pos, &old_rot, &old_dir,
        );
        assert_eq!(chains.len(), 1, "the stock single tail");
        assert_eq!(chains[0].len(), (p.segs + 1) as usize);
        // The empty ring: every point is the head anchor.
        let head = chains[0][0];
        assert!(chains[0].iter().all(|v| *v == head));
        assert_eq!(num6, 0.0, "the colour kites carry no sway");
        // The no-tail five produce nothing.
        let p0 = kite_tail_params(824).unwrap();
        let (none, _) = kite_tail_chains(
            &p0, center, Vec2::splat(2.0), 30.0, 0.0, 1,
            0.0, 0.0, 0.0, 0.0, 5.0, &old_pos, &old_rot, &old_dir,
        );
        assert!(none.is_empty());
        // The jellyfish trio: 3 chains.
        let p3 = kite_tail_params(828).unwrap();
        let (three, _) = kite_tail_chains(
            &p3, center, Vec2::splat(2.0), 30.0, 0.0, 1,
            0.0, 0.0, 0.0, 0.0, 5.0, &old_pos, &old_rot, &old_dir,
        );
        assert_eq!(three.len(), 3);
    }

    /// A filled ring walks the history: the second point sits within
    /// `max_len` of the head (the `:37524-37527` clamp).
    #[test]
    fn w4_d16_kite_draw_the_history_clamp() {
        let p = kite_tail_params(766).unwrap();
        let center = Vec2::new(100.0, 100.0);
        let mut old_pos = [Vec2::ZERO; 10];
        for (i, slot) in old_pos.iter_mut().enumerate() {
            *slot = Vec2::new(100.0 - 500.0, 100.0 + i as f32);
        }
        let old_rot = [0.0; 10];
        let old_dir = [1; 10];
        let (chains, _) = kite_tail_chains(
            &p, center, Vec2::splat(2.0), 30.0, 0.0, 1,
            0.0, 0.0, 0.0, 0.0, 5.0, &old_pos, &old_rot, &old_dir,
        );
        let d = chains[0][1].distance(chains[0][0]);
        assert!(d <= p.max_len + 1e-3, "the segment clamp: {d}");
    }
}
