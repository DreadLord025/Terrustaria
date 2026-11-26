//! The double-precision 2D vector + scalar math the dual-dungeon dither
//! snake rides — `ReLogic.Utilities.Vector2D` (`Vector2D.cs`) and the
//! `Terraria.Utils` scalar/extension helpers the snake calls
//! (`Utils.cs:107/:315/:350/:730/:1686/:1822/:1846`).
//!
//! W4-f24 lane B (the gated `Dual Dungeons Dither Snake` pass,
//! `WorldGen.cs:14755`). The C# snake geometry is f64 throughout
//! (`DungeonControlLine`/`DitherSnake`/`DitherSnakePass`); the port keeps
//! f64 so the tangent/normal maths and the Bayer-dithered paint stay
//! bit-faithful. Only the methods the snake family calls are ported —
//! the rest of `Vector2D` (Hermite/CatmullRom/Reflect/…) has no consumer
//! here and stays out (cited skip).
//!
//! UNWIRED until B6 lands the gated step: pure math, no RNG draw, no tile
//! write — both classic world-hash pins stay (`world_hash.rs`'s W4-f24
//! note); the `a == b` determinism halves are untouched.

/// `ReLogic.Utilities.Vector2D` (`Vector2D.cs:9-435`) — the f64 pair the
/// snake's control lines, tangents and normals ride. `Zero`/`UnitX`/`UnitY`
/// are the C# statics (`:25-31`); the operators map to the named methods
/// (`Add`/`Subtract`/`Multiply`/`Divide`/`Negate`, `:92-412`).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct Vec2 {
    pub x: f64,
    pub y: f64,
}

impl Vec2 {
    /// `Vector2D(double x, double y)` (`Vector2D.cs:35-39`).
    pub const fn new(x: f64, y: f64) -> Self {
        Vec2 { x, y }
    }

    /// `Vector2D.Zero` (`Vector2D.cs:25`).
    pub const ZERO: Vec2 = Vec2 { x: 0.0, y: 0.0 };
    /// `Vector2D.UnitX` (`Vector2D.cs:29`).
    pub const UNIT_X: Vec2 = Vec2 { x: 1.0, y: 0.0 };
    /// `Vector2D.UnitY` (`Vector2D.cs:31`).
    pub const UNIT_Y: Vec2 = Vec2 { x: 0.0, y: 1.0 };

    /// `Vector2D.Length()` (`Vector2D.cs:70-73`).
    pub fn length(self) -> f64 {
        (self.x * self.x + self.y * self.y).sqrt()
    }

    /// `Vector2D.LengthSquared()` (`Vector2D.cs:75-78`).
    pub fn length_squared(self) -> f64 {
        self.x * self.x + self.y * self.y
    }

    /// `Vector2D.Dot` (`Vector2D.cs:194-197`).
    pub fn dot(a: Vec2, b: Vec2) -> f64 {
        a.x * b.x + a.y * b.y
    }

    /// `Vector2D.Cross` (`Vector2D.cs:204-207`) — the scalar 2D cross.
    pub fn cross(a: Vec2, b: Vec2) -> f64 {
        a.x * b.y - a.y * b.x
    }

    /// `Vector2D.Distance` (`Vector2D.cs:138-143`).
    pub fn distance(a: Vec2, b: Vec2) -> f64 {
        let dx = a.x - b.x;
        let dy = a.y - b.y;
        (dx * dx + dy * dy).sqrt()
    }

    /// `Vector2D.Normalize` (`Vector2D.cs:294-300`) — divides by the length
    /// via the reciprocal (the C#'s `1.0 / sqrt(...)` multiply, kept).
    pub fn normalize(v: Vec2) -> Vec2 {
        let n = 1.0 / (v.x * v.x + v.y * v.y).sqrt();
        Vec2::new(v.x * n, v.y * n)
    }

    /// `Vector2D.Lerp` (`Vector2D.cs:222-225`) over the scalar `Lerp`
    /// (`:432-435`).
    pub fn lerp(a: Vec2, b: Vec2, amount: f64) -> Vec2 {
        Vec2::new(lerp(a.x, b.x, amount), lerp(a.y, b.y, amount))
    }

    /// `Utils.RotatedBy(this Vector2D, radians, center)`
    /// (`Utils.cs:1686-1695`) — rotate about `center` (default `Zero`).
    pub fn rotated_by(self, radians: f64, center: Vec2) -> Vec2 {
        let cos = radians.cos();
        let sin = radians.sin();
        let v = self - center;
        Vec2::new(
            center.x + v.x * cos - v.y * sin,
            center.y + v.x * sin + v.y * cos,
        )
    }

    /// `Utils.SafeNormalize(this Vector2D, defaultValue)`
    /// (`Utils.cs:1846-1853`) — the zero vector returns the default.
    pub fn safe_normalize(self, default: Vec2) -> Vec2 {
        if self == Vec2::ZERO {
            default
        } else {
            Vec2::normalize(self)
        }
    }

    /// `Utils.ToPoint(this Vector2D)` (`Utils.cs:1822-1825`) — the C#
    /// `(int)` truncation toward zero (NOT floor).
    pub fn to_point(self) -> (i32, i32) {
        (self.x as i32, self.y as i32)
    }
}

impl std::ops::Add for Vec2 {
    type Output = Vec2;
    fn add(self, rhs: Vec2) -> Vec2 {
        Vec2::new(self.x + rhs.x, self.y + rhs.y)
    }
}

impl std::ops::Sub for Vec2 {
    type Output = Vec2;
    fn sub(self, rhs: Vec2) -> Vec2 {
        Vec2::new(self.x - rhs.x, self.y - rhs.y)
    }
}

impl std::ops::Neg for Vec2 {
    type Output = Vec2;
    fn neg(self) -> Vec2 {
        Vec2::new(-self.x, -self.y)
    }
}

impl std::ops::Mul<f64> for Vec2 {
    type Output = Vec2;
    fn mul(self, s: f64) -> Vec2 {
        Vec2::new(self.x * s, self.y * s)
    }
}

/// `Vector2D.operator *(Vector2D, Vector2D)` (`Vector2D.cs:384-389`) — the
/// component-wise product the snake's `vector2D2 * new Vector2D(num20, y)`
/// uses (`DitherSnakePass.cs:89`).
impl std::ops::Mul<Vec2> for Vec2 {
    type Output = Vec2;
    fn mul(self, rhs: Vec2) -> Vec2 {
        Vec2::new(self.x * rhs.x, self.y * rhs.y)
    }
}

impl std::ops::Div<f64> for Vec2 {
    type Output = Vec2;
    fn div(self, s: f64) -> Vec2 {
        Vec2::new(self.x / s, self.y / s)
    }
}

/// `Utils.Lerp(double, double, double)` (`Utils.cs:107-110`, identical to
/// `Vector2D.Lerp` `Vector2D.cs:432-435`).
pub(crate) fn lerp(a: f64, b: f64, amount: f64) -> f64 {
    a + (b - a) * amount
}

/// `Utils.GetLerpValue(double, double, double, bool clamped = false)`
/// (`Utils.cs:730-757`) — the clamped arms keep the C#'s from/to ordering
/// (a reversed range flips the 0/1 returns).
pub(crate) fn get_lerp_value(from: f64, to: f64, t: f64, clamped: bool) -> f64 {
    if clamped {
        if from < to {
            if t < from {
                return 0.0;
            }
            if t > to {
                return 1.0;
            }
        } else {
            if t < to {
                return 1.0;
            }
            if t > from {
                return 0.0;
            }
        }
    }
    (t - from) / (to - from)
}

/// `Utils.Remap(double, …, bool clamped = true)` (`Utils.cs:350-353`) —
/// `Lerp(toMin, toMax, GetLerpValue(fromMin, fromMax, fromValue, clamped))`.
/// The C# default is `clamped: true`; the snake's calls
/// (`DitherSnakePass.cs:60`, `DungeonControlLine.cs:161/:166`) all take it.
pub(crate) fn remap(from: f64, from_min: f64, from_max: f64, to_min: f64, to_max: f64) -> f64 {
    lerp(to_min, to_max, get_lerp_value(from_min, from_max, from, true))
}

/// `MathHelper.Clamp(int, int, int)` / `Utils.Clamp` — the snake's bounds
/// setters (`DungeonBounds.cs:61/:73/:85/:97`) clamp to the world margins.
pub(crate) fn clamp_i(value: i32, min: i32, max: i32) -> i32 {
    if value < min {
        min
    } else if value > max {
        max
    } else {
        value
    }
}

/// `MathHelper.Clamp(double, double, double)` — the shimmer-island spawn
/// clamps (`WorldGen.cs:22621-22622`) and `SkyblockIsland`'s column clamp
/// (`:23420/:23455`).
pub(crate) fn clamp_f(value: f64, min: f64, max: f64) -> f64 {
    if value < min {
        min
    } else if value > max {
        max
    } else {
        value
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The scalar `Lerp`/`GetLerpValue`/`Remap` chain against hand-derived
    /// values (the C# `Utils.cs:107/:730/:350` formulas, not this code).
    #[test]
    fn w4_f24_scalar_math_matches_the_c_formulas() {
        assert_eq!(lerp(10.0, 20.0, 0.25), 12.5);
        // GetLerpValue clamped: t below from -> 0, above to -> 1.
        assert_eq!(get_lerp_value(0.0, 1.0, -0.5, true), 0.0);
        assert_eq!(get_lerp_value(0.0, 1.0, 1.5, true), 1.0);
        assert_eq!(get_lerp_value(0.0, 1.0, 0.5, true), 0.5);
        // Reversed range flips the clamped returns (Utils.cs:745-756).
        assert_eq!(get_lerp_value(1.0, 0.0, -0.5, true), 1.0);
        assert_eq!(get_lerp_value(1.0, 0.0, 1.5, true), 0.0);
        // Unclamped passes the raw fraction through.
        assert_eq!(get_lerp_value(0.0, 1.0, 1.5, false), 1.5);
        // Remap(0.5, 0,1, 10,20) = Lerp(10,20, 0.5) = 15.
        assert_eq!(remap(0.5, 0.0, 1.0, 10.0, 20.0), 15.0);
        // Remap clamps by default: 1.5 -> toMax.
        assert_eq!(remap(1.5, 0.0, 1.0, 10.0, 20.0), 20.0);
    }

    /// `Vector2D` dot/cross/normalize/rotated_by/safe_normalize against the
    /// ReLogic formulas (`Vector2D.cs:194/:204/:294`, `Utils.cs:1686/:1846`).
    #[test]
    fn w4_f24_vector_math_matches_the_relogic_formulas() {
        let a = Vec2::new(3.0, 4.0);
        assert_eq!(a.length(), 5.0);
        assert_eq!(a.length_squared(), 25.0);
        assert_eq!(Vec2::dot(a, Vec2::new(1.0, 0.0)), 3.0);
        assert_eq!(Vec2::cross(a, Vec2::new(1.0, 0.0)), -4.0);
        let n = Vec2::normalize(a);
        // `Normalize` divides via the `1/sqrt` reciprocal (`Vector2D.cs:296`),
        // so 3/5 lands one ulp above the `0.6` literal — compare the C#
        // double result within a tight tolerance, not bit-exactly.
        assert!((n.x - 0.6).abs() < 1e-12 && (n.y - 0.8).abs() < 1e-12);
        // SafeNormalize of zero returns the default (Utils.cs:1848-1851).
        assert_eq!(Vec2::ZERO.safe_normalize(Vec2::UNIT_X), Vec2::UNIT_X);
        assert_eq!(a.safe_normalize(Vec2::ZERO), n);
        // RotatedBy(PI/2) about zero maps (1,0) -> (0,1).
        let r = Vec2::UNIT_X.rotated_by(std::f64::consts::FRAC_PI_2, Vec2::ZERO);
        assert!((r.x - 0.0).abs() < 1e-12 && (r.y - 1.0).abs() < 1e-12);
        // ToPoint truncates toward zero (Utils.cs:1824), not floor.
        assert_eq!(Vec2::new(-1.9, 2.9).to_point(), (-1, 2));
        // Component-wise product (DitherSnakePass.cs:89).
        assert_eq!(Vec2::new(2.0, 3.0) * Vec2::new(4.0, 5.0), Vec2::new(8.0, 15.0));
    }
}
