//! `Main.DrawSunAndMoon` (`Terraria/Main.cs:62848`) and `Main.SetBackColor`
//! (`Terraria/Main.cs:63452`) — the pure maths behind the sun and moon.
//!
//! Everything here is in *screen* pixels (the `SpriteBatch` space the original
//! uses): `x`/`y` are the sprite centres because `DrawSunAndMoon` draws with
//! `origin = value.Size() / 2`. The renderer turns these into world positions.

/// `Main.time` length of the day: `0..54000`.
pub const DAY_LENGTH: f64 = 54000.0;
/// `Main.time` length of the night: `0..32400`.
pub const NIGHT_LENGTH: f64 = 32400.0;

/// `DrawSunAndMoon` placement of one celestial body. The original computes the
/// sun and the moon with the *same* expressions, differing only in the period
/// (`54000` sun / `32400` moon) and in the day branch's `scale *= 1.1`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CelestialBody {
    /// `num3`/`num6`: centre x in screen pixels.
    pub x: f32,
    /// `num4`/`num7`: centre y in screen pixels (before `Main.sunModY`/`moonModY`,
    /// which is always 0 outside the title screen).
    pub y: f32,
    /// `num5`/`num8`: sprite scale.
    pub scale: f32,
    /// `rotation`/`num9`: clockwise radians around the centre.
    pub rotation: f32,
}

/// `DrawSunAndMoon` geometry for one body. `bg_top_y` is `(int)sceneArea.bgTopY`,
/// `tex_w` the source texture width and `total_width` the window width
/// (`SceneArea.totalWidth = Main.screenWidth`).
pub fn celestial_body(
    time: f64,
    period: f64,
    bg_top_y: i32,
    tex_w: f32,
    total_width: f32,
) -> CelestialBody {
    let t = time / period;
    // `(int)(Main.time / period * (totalWidth + width * 2)) - width`
    let span = (total_width + tex_w * 2.0) as f64;
    let x = (t * span) as i32 as f32 - tex_w;
    // `Math.Pow(Math.Abs(Main.time / period * 2.0 - 1.0), 2.0)`
    let d = (t * 2.0 - 1.0).abs().powi(2);
    let y = (bg_top_y as f64 + d * 250.0 + 180.0) as i32 as f32;
    CelestialBody {
        x,
        y,
        scale: (1.2 - d * 0.4) as f32,
        rotation: t as f32 * 2.0 - 7.3,
    }
}

/// `Main.bgTopY` for the in-game sky pass (`Main.cs:62040`), an `int`:
/// `(-screenPosition.Y) / (worldSurface * 16 - 600) * 200`. `view_top` is
/// `Main.screenPosition.Y`.
pub fn sky_bg_top_y(view_top: f32, world_surface: f64) -> i32 {
    ((-view_top) / (world_surface as f32 * 16.0 - 600.0) * 200.0) as i32
}

/// `SetBackColor` sun tint as sRGB bytes. Only the base (forest) day branches are
/// ported; corruption/crimson/graveyard/blood-moon modifiers are not modelled.
pub fn sun_color(time: f64) -> [u8; 3] {
    let b = |v: f64| v.clamp(0.0, 255.0) as u8;
    if time < 13500.0 {
        let t = time / 13500.0;
        [b(t * 200.0 + 55.0), b(t * 180.0 + 75.0), b(t * 250.0 + 5.0)]
    } else if time > 45900.0 {
        let t = 1.0 - (time / 54000.0 - 0.85) * 6.666_666_666_666_667;
        [b(t * 120.0 + 55.0), b(t * 100.0 + 25.0), b(t * 120.0 + 55.0)]
    } else if time > 37800.0 {
        let t = 1.0 - (time / 54000.0 - 0.7) * 6.666_666_666_666_667;
        [b(t * 80.0 + 175.0), b(t * 130.0 + 125.0), b(t * 100.0 + 155.0)]
    } else {
        [255, 255, 255]
    }
}

/// `SetBackColor` moon tint as sRGB bytes (forest, no blood moon).
pub fn moon_color(time: f64) -> [u8; 3] {
    let b = |v: f64| v.clamp(0.0, 255.0) as u8;
    if time < 16200.0 {
        let t = 1.0 - time / 16200.0;
        [b(t * 10.0 + 205.0), b(t * 70.0 + 155.0), b(t * 100.0 + 155.0)]
    } else {
        let t = (time / 32400.0 - 0.5) * 2.0;
        [b(t * 50.0 + 205.0), b(t * 100.0 + 155.0), b(t * 100.0 + 155.0)]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sun_peaks_centred_at_noon() {
        // Noon (time = 27000) is the vertex of the arc: centred, highest, full size.
        let b = celestial_body(27000.0, DAY_LENGTH, 0, 114.0, 1706.0);
        assert_eq!(b.x, 853.0); // window centre; `(0.5 * (1706 + 228)) - 114`
        assert_eq!(b.y, 180.0); // d = 0 -> bgTopY + 180
        assert_eq!(b.scale, 1.2);
        assert!((b.rotation - -6.3).abs() < 1e-6);
    }

    #[test]
    fn sun_rises_at_left_edge_offscreen() {
        // Sunrise (time = 0): one texture width left of the screen, at the arc base.
        let b = celestial_body(0.0, DAY_LENGTH, 0, 114.0, 1706.0);
        assert_eq!(b.x, -114.0);
        assert_eq!(b.y, 430.0); // d = 1 -> 0 + 250 + 180
        assert_eq!(b.scale, 0.8);
        assert!((b.rotation - -7.3).abs() < 1e-6);
    }

    #[test]
    fn moon_uses_the_night_period() {
        // Midnight (time = 16200 of 32400) mirrors noon for the moon.
        let b = celestial_body(16200.0, NIGHT_LENGTH, 0, 50.0, 1706.0);
        assert_eq!(b.x, 853.0); // (0.5 * (1706 + 100)) - 50
        assert_eq!(b.y, 180.0);
        assert_eq!(b.scale, 1.2);
    }

    #[test]
    fn bg_top_y_tracks_the_camera() {
        // At screenPosition.Y = 0 the sky anchor is 0; it drops 200 px as the
        // camera reaches the surface (worldSurface * 16 - 600).
        assert_eq!(sky_bg_top_y(0.0, 400.0), 0);
        assert_eq!(sky_bg_top_y(400.0 * 16.0 - 600.0, 400.0), -200);
    }

    #[test]
    fn colours_are_white_at_midday_and_blue_at_midnight() {
        assert_eq!(sun_color(27000.0), [255, 255, 255]);
        // Night past 16200: moon colour straightens towards white.
        let c = moon_color(32400.0);
        assert_eq!(c, [255, 255, 255]);
        // Early night is the bluish tint.
        let c = moon_color(0.0);
        assert_eq!(c, [215, 225, 255]);
    }
}
