//! `DitherSnake` (`GameContent/Biomes/DitherSnake.cs:10-199`) — the ordered
//! list of [`ControlLine`] segments that forms the dual-dungeon's painted
//! spine, plus the border/room queries the dungeon layout reads.
//!
//! W4-f24 lane B (the gated `Dual Dungeons Dither Snake` pass,
//! `WorldGen.cs:14755`). The C# `DitherSnake : List<DungeonControlLine>`
//! links each line to its neighbours on `Add` (`:16-26`) and walks those
//! links in `GetLineContaining`/`SetTangents`; the port keeps a `Vec` and
//! models the links as neighbour indices (the order `Add` builds), so the
//! walks become index steps. `GetLineContaining`'s depth-3 recursion is
//! unrolled into a bounded loop (same arms, same order).
//!
//! The statics `CircleTestPoints` (12 unit vectors at 30° steps) and
//! `ExtraBuffer` (`1/cos(π/6)`) are the C# static ctor (`:193-198`); they
//! are computed once here as consts-by-fn (the trig is deterministic).
//!
//! UNWIRED until B6: pure geometry over the line list, no RNG draw, no
//! tile write — both classic world-hash pins stay; `a == b` untouched.

use super::dungeon_control_line::{adjust_tangents, ControlLine};
use super::snake_math::Vec2;

/// `SnakeOrientation` (`SnakeOrientation.cs:3-9`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SnakeOrientation {
    Unknown,
    Top,
    Center,
    Bottom,
}

/// `DitherSnake.ExtraBuffer` (`DitherSnake.cs:197`) — `1/cos(π/6)`.
fn extra_buffer() -> f64 {
    1.0 / (std::f64::consts::PI / 6.0).cos()
}

/// `DitherSnake.CircleTestPoints` (`:195-196`) — `UnitX.RotatedBy(2π·i/12)`
/// for i in 0..12.
fn circle_test_points() -> [Vec2; 12] {
    let mut pts = [Vec2::ZERO; 12];
    for (i, p) in pts.iter_mut().enumerate() {
        *p = Vec2::UNIT_X.rotated_by(std::f64::consts::TAU * i as f64 / 12.0, Vec2::ZERO);
    }
    pts
}

/// `DitherSnake` (`DitherSnake.cs:10-199`).
#[derive(Clone, Debug, Default)]
pub(crate) struct DitherSnake {
    pub lines: Vec<ControlLine>,
}

impl DitherSnake {
    pub fn new() -> Self {
        DitherSnake { lines: Vec::new() }
    }

    /// `DitherSnake.Add(DungeonControlLine)` (`:16-26`) — set the line's
    /// `Index` to the current count and append (the C# also wires
    /// `Next`/`Prev`; the port's neighbour indices are implicit in the
    /// append order, so no explicit link write is needed).
    pub fn add(&mut self, mut line: ControlLine) {
        line.index = self.lines.len();
        self.lines.push(line);
    }

    pub fn first(&self) -> &ControlLine {
        &self.lines[0]
    }

    pub fn last(&self) -> &ControlLine {
        &self.lines[self.lines.len() - 1]
    }

    /// `GetClosestLineTo(Vector2D)` (`:28-42`) — the line whose centre is
    /// nearest (strict `<`, so the first wins ties).
    pub fn closest_line_to(&self, pos: Vec2) -> usize {
        let mut result = 0;
        let mut best = f64::MAX;
        for (i, line) in self.lines.iter().enumerate() {
            let d = Vec2::distance(line.center(), pos);
            if d < best {
                result = i;
                best = d;
            }
        }
        result
    }

    /// `GetLineContaining(Vector2D, initialGuess, depth)` (`:44-63`) — walk
    /// the neighbour links while the point projects behind the current
    /// line's start (→ prev) or end (→ next); depth 3 gives up (None). The
    /// C# recursion is a bounded loop here (max 3 hops, same arm order).
    pub fn line_containing(&self, pos: Vec2, initial_guess: Option<usize>) -> Option<usize> {
        let mut guess = initial_guess.unwrap_or_else(|| self.closest_line_to(pos));
        for _ in 0..3 {
            let line = &self.lines[guess];
            if Vec2::dot(pos - line.start, line.start_tangent) < 0.0 {
                if let Some(pi) = guess.checked_sub(1) {
                    guess = pi;
                    continue;
                }
            }
            if Vec2::dot(pos - line.end, line.end_tangent) < 0.0 {
                if guess + 1 < self.lines.len() {
                    guess += 1;
                    continue;
                }
            }
            return Some(guess);
        }
        None
    }

    /// `GetPositionAlongSnake(Vector2D)` (`:65-73`) — the containing line's
    /// index plus its normalized progress (0.5 when the point can't paint).
    pub fn position_along_snake(&self, pos: Vec2) -> f64 {
        let Some(li) = self.line_containing(pos, None) else {
            // The C# dereferences a null line here (a point outside every
            // capsule); the port returns the closest index + 0.5 — the
            // dungeon layout never queries such a point (cited adaptation).
            return self.closest_line_to(pos) as f64 + 0.5;
        };
        let line = &self.lines[li];
        let progress = match line.can_paint(&self.lines, pos.x as i32, pos.y as i32) {
            Some((_, p)) => p,
            None => 0.5,
        };
        li as f64 + progress
    }

    /// `IsCircleInsideBorderWithStyle(style, center, radius)` (`:75-83`) —
    /// the closest line must carry the style, then the matching-style test.
    pub fn circle_inside_border_with_style(
        &self,
        style: usize,
        center: Vec2,
        radius: i32,
        safe_distance: f64,
    ) -> bool {
        let li = self.closest_line_to(center);
        if self.lines[li].style == style {
            return self.circle_inside_border_matching_style(li, center, radius, safe_distance);
        }
        false
    }

    /// `IsCircleInsideBorderWithMatchingStyle(nearbyLine, center, radius)`
    /// (`:85-103`) — all 12 buffered circle points must land in a line of
    /// the same style and inside its border.
    pub fn circle_inside_border_matching_style(
        &self,
        nearby: usize,
        center: Vec2,
        radius: i32,
        safe_distance: f64,
    ) -> bool {
        let num = radius as f64 * extra_buffer();
        let nearby_style = self.lines[nearby].style;
        for p in circle_test_points() {
            let point = center + p * num;
            let Some(li) = self.line_containing(point, Some(nearby)) else {
                return false;
            };
            if self.lines[li].style != nearby_style {
                return false;
            }
            let (px, py) = point.to_point();
            if !self.lines[li].is_inside_border(&self.lines, px, py, safe_distance) {
                return false;
            }
        }
        true
    }

    /// `GetRoomPositionInsideBorder(line, normalizedDistanceAlong,
    /// normalizedDistanceFrom, roomRadius, out orientation)` (`:105-139`) —
    /// try the offset room position, shrinking the offset ×0.8 up to four
    /// times until it fits the border; classify Top/Center/Bottom against
    /// the ±1 offset extremes, else fall back to the line centre.
    pub fn room_position_inside_border(
        &self,
        li: usize,
        normalized_distance_along: f64,
        mut normalized_distance_from: f64,
        room_radius: i32,
        safe_distance: f64,
    ) -> (Vec2, SnakeOrientation) {
        let line = &self.lines[li];
        let target = Vec2::lerp(line.start, line.end, normalized_distance_along);
        let p_neg = line.potential_room_position(normalized_distance_along, -1.0, room_radius, safe_distance);
        let p_pos = line.potential_room_position(normalized_distance_along, 1.0, room_radius, safe_distance);
        let (target2, target3) = if p_neg.y < p_pos.y { (p_neg, p_pos) } else { (p_pos, p_neg) };
        for _ in 0..4 {
            let candidate =
                line.potential_room_position(normalized_distance_along, normalized_distance_from, room_radius, safe_distance);
            if self.circle_inside_border_matching_style(li, candidate, room_radius, safe_distance) {
                let d = Vec2::distance(candidate, target);
                let d2 = Vec2::distance(candidate, target2);
                let d3 = Vec2::distance(candidate, target3);
                let orientation = if d < d2 && d < d3 {
                    SnakeOrientation::Center
                } else if d2 < d3 {
                    SnakeOrientation::Top
                } else {
                    SnakeOrientation::Bottom
                };
                return (candidate, orientation);
            }
            normalized_distance_from *= 0.8;
        }
        (line.center(), SnakeOrientation::Center)
    }

    /// `SetTangents()` (`:141-152`) — the first line's start tangent is its
    /// direction; each interior joint averages the two directions
    /// (SafeNormalize default Zero) and shares the negated tangent; the
    /// last line's end tangent is the negation of its direction.
    pub fn set_tangents(&mut self) {
        let n = self.lines.len();
        if n == 0 {
            return;
        }
        self.lines[0].start_tangent = self.lines[0].normalized_line_direction;
        for i in 0..n - 1 {
            let avg = (self.lines[i].normalized_line_direction
                + self.lines[i + 1].normalized_line_direction)
                .safe_normalize(Vec2::ZERO);
            self.lines[i + 1].start_tangent = avg;
            self.lines[i].end_tangent = -avg;
        }
        self.lines[n - 1].end_tangent = -self.lines[n - 1].normalized_line_direction;
    }

    /// `AdjustTangentsToPreventSelfIntersection()` (`:154-171`) — up to 100
    /// sweeps; each sweep adjusts every self-intersecting line and stops
    /// early when a full sweep changes nothing.
    pub fn adjust_tangents_to_prevent_self_intersection(&mut self) {
        for _ in 0..100 {
            let mut changed = false;
            for i in 0..self.lines.len() {
                if adjust_tangents(&mut self.lines, i) {
                    changed = true;
                }
            }
            if !changed {
                break;
            }
        }
    }

    /// `IsLineInsideBorder(from, to, halfWidth)` (`:173-181`) — both offset
    /// edges (the perpendicular ±halfWidth) must stay inside.
    pub fn line_inside_border_half_width(
        &self,
        from: Vec2,
        to: Vec2,
        half_width: i32,
        safe_distance: f64,
    ) -> bool {
        let offset = (to - from)
            .safe_normalize(Vec2::UNIT_X)
            .rotated_by(std::f64::consts::FRAC_PI_2, Vec2::ZERO)
            * half_width as f64;
        if self.line_inside_border(from + offset, to + offset, safe_distance) {
            return self.line_inside_border(from - offset, to - offset, safe_distance);
        }
        false
    }

    /// `IsLineInsideBorder(from, to)` (`:183-191`) — plot the integer line
    /// (`Utils.PlotLine`, jump=true) and require every cell to land in a
    /// border line; the walk stops at the first failure.
    pub fn line_inside_border(&self, from: Vec2, to: Vec2, safe_distance: f64) -> bool {
        let (x0, y0) = from.to_point();
        let (x1, y1) = to.to_point();
        let mut guess = self.closest_line_to(from);
        let mut ok = true;
        plot_line_bool(x0, y0, x1, y1, |x, y| {
            let pos = Vec2::new(x as f64, y as f64);
            match self.line_containing(pos, Some(guess)) {
                Some(li) => {
                    guess = li;
                    self.lines[li].is_inside_border(&self.lines, x, y, safe_distance)
                }
                None => false,
            }
        }, &mut ok);
        ok
    }
}

/// `Utils.PlotLine(Point, Point, TileActionAttempt, jump=true)`
/// (`Utils.cs:2233-2292`) — the Bresenham walk that stops at the first
/// `false` plot. The port's `desert_entrances::plot_line` plots every cell
/// (void callback); this variant carries the early-exit the border query
/// needs. `ok` is set false when a plot returns false.
fn plot_line_bool(
    x0: i32,
    y0: i32,
    x1: i32,
    y1: i32,
    mut plot: impl FnMut(i32, i32) -> bool,
    ok: &mut bool,
) {
    if x0 == x1 && y0 == y1 {
        *ok = plot(x0, y0);
        return;
    }
    let steep = (y1 - y0).abs() > (x1 - x0).abs();
    let (mut ax0, mut ay0, mut ax1, mut ay1) = (x0, y0, x1, y1);
    if steep {
        std::mem::swap(&mut ax0, &mut ay0);
        std::mem::swap(&mut ax1, &mut ay1);
    }
    let num = (ax1 - ax0).abs();
    let num2 = (ay1 - ay0).abs();
    let mut num3 = num / 2;
    let mut num4 = ay0;
    let num5 = if ax0 < ax1 { 1 } else { -1 };
    let num6 = if ay0 < ay1 { 1 } else { -1 };
    let mut i = ax0;
    while i != ax1 {
        let (px, py) = if steep { (num4, i) } else { (i, num4) };
        if !plot(px, py) {
            *ok = false;
            return;
        }
        num3 -= num2;
        if num3 >= 0 {
            i += num5;
            continue;
        }
        num4 += num6;
        // jump=true: no second plot (Utils.cs:2275-2288).
        num3 += num;
        i += num5;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn snake_two() -> DitherSnake {
        let mut s = DitherSnake::new();
        s.add(ControlLine::new(0, Vec2::new(0.0, 0.0), Vec2::new(10.0, 0.0), 10.0, 10.0, 0, 0));
        s.add(ControlLine::new(0, Vec2::new(10.0, 0.0), Vec2::new(20.0, 0.0), 10.0, 10.0, 0, 0));
        s.set_tangents();
        s
    }

    /// `Add` reindexes and `SetTangents` averages the joint
    /// (`DitherSnake.cs:16-26/:141-152`).
    #[test]
    fn w4_f24_add_reindexes_and_set_tangents() {
        let s = snake_two();
        assert_eq!(s.lines[0].index, 0);
        assert_eq!(s.lines[1].index, 1);
        // Straight collinear segments: every tangent is +X.
        assert_eq!(s.lines[0].start_tangent, Vec2::UNIT_X);
        assert_eq!(s.lines[0].end_tangent, -Vec2::UNIT_X);
        assert_eq!(s.lines[1].start_tangent, Vec2::UNIT_X);
        assert_eq!(s.lines[1].end_tangent, -Vec2::UNIT_X);
    }

    /// `GetClosestLineTo` picks the nearer centre; `GetLineContaining`
    /// walks to the segment that owns the point (`:28-63`).
    #[test]
    fn w4_f24_closest_and_containing() {
        let s = snake_two();
        assert_eq!(s.closest_line_to(Vec2::new(15.0, 1.0)), 1);
        assert_eq!(s.closest_line_to(Vec2::new(5.0, 1.0)), 0);
        assert_eq!(s.line_containing(Vec2::new(15.0, 0.0), None), Some(1));
        assert_eq!(s.line_containing(Vec2::new(5.0, 0.0), None), Some(0));
    }

    /// `ExtraBuffer` is 1/cos(π/6) and the 12 circle points are unit
    /// vectors (`DitherSnake.cs:193-198`).
    #[test]
    fn w4_f24_circle_statics() {
        let b = extra_buffer();
        assert!((b - 1.1547005383792515).abs() < 1e-12);
        let pts = circle_test_points();
        assert_eq!(pts.len(), 12);
        for p in pts {
            assert!((p.length() - 1.0).abs() < 1e-12);
        }
        assert!((pts[0].x - 1.0).abs() < 1e-12 && pts[0].y.abs() < 1e-12);
    }

    /// The bool PlotLine stops at the first false plot
    /// (`Utils.cs:2256-2291`).
    #[test]
    fn w4_f24_plot_line_bool_early_exit() {
        let mut visited = Vec::new();
        let mut ok = true;
        plot_line_bool(0, 0, 5, 0, |x, y| {
            visited.push((x, y));
            x < 2
        }, &mut ok);
        assert!(!ok);
        assert_eq!(visited, vec![(0, 0), (1, 0), (2, 0)]);
    }
}
