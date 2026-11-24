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

use super::dungeon_bounds::DungeonBounds;
use super::dungeon_control_line::{adjust_tangents, ControlLine, SnakeStyle};
use super::snake_math::{lerp, remap, Vec2};
use crate::world::World;

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

/// `DungeonSide.Left`/`Right` (`DungeonSide.cs:13-15`).
const DUNGEON_SIDE_LEFT: i32 = -1;

/// `DitherSnakePass.CalculateDungeonDitherSnake`
/// (`DitherSnakePass.cs:44-133`) — build the snake's control lines over the
/// dungeon's outer potential bounds. `dungeon_side` is the C#
/// `GenVars.CurrentDungeonGenVars.dungeonSide` (−1 Left / +1 Right);
/// `dungeon_location` its `dungeonLocation`; `styles` the
/// `dungeonGenerationStyles` list (the dual quad, B5); `safe_distance` the
/// `NormalizedDistanceSafeFromDither` (`WorldGen.cs:11465`). `rng` is the
/// pass stream (`WorldGen.genRand`) — the two `NextDouble` radius walks
/// (`:100-101`) are its only draws.
pub(crate) fn calculate_dungeon_dither_snake(
    rng: &mut super::Rng,
    max_tiles_x: i32,
    outer: &DungeonBounds,
    dungeon_side: i32,
    dungeon_location: i32,
    style_count: usize,
    safe_distance: f64,
) -> DitherSnake {
    let mut snake = DitherSnake::new();
    // `:50-51`: num = maxTilesX/4200, num2 = (int)(20·num).
    let num = max_tiles_x as f64 / 4200.0;
    let num2 = (20.0 * num) as i32;
    let count = style_count as i32;
    // `:53-62`: the normalized grid + the radius envelope.
    let num3 = 1.0 / (num2 - 1) as f64;
    let num4 = 1.0 / (count - 1) as f64;
    let num5 = outer.height() as f64 / count as f64;
    let num6 = 1.0 - safe_distance;
    let num7 = num5 / 2.0 * (1.0 + num6 / 2.0) - 1.0;
    let num8 = num7 - 0.1 * num * num5;
    let num9 = 0.05;
    let num10 = remap(num, 1.0, 2.0, 1.0, 1.5);
    let num11 = num7 + num5 / 2.0 * num10;
    let num12 = num7;
    // `:63-64`: the grid origin + extent inside the bounds.
    let grid_origin = Vec2::new(outer.x() as f64 + num11, outer.y() as f64 + num12);
    let grid_extent = Vec2::new(
        outer.width() as f64 - num11 * 2.0,
        outer.height() as f64 - num12 * 2.0,
    );
    // `:65`: the per-step radius clamp.
    let num13 = (num7 - num8).min(grid_extent.x * num3 / 2.0);
    let mut start_radius = num7;
    let mut start = Vec2::ZERO;
    let num14 = num2 * count;
    let num15 = num14 / count;
    let mut num16 = num7;
    let mut num17 = num7;
    let mut i = 0i32;
    while i < num14 {
        let num18 = i % num2;
        let num19 = i / num2;
        let mut num20 = num3 * num18 as f64;
        let y = num4 * num19 as f64;
        // `:80-88`: the serpentine direction (flip on odd rows, mirror x).
        let mut num21 = if dungeon_side == DUNGEON_SIDE_LEFT { 1 } else { -1 };
        if num19 % 2 == 1 {
            num21 *= -1;
        }
        if num21 < 0 {
            num20 = 1.0 - num20;
        }
        let mut point = grid_origin + grid_extent * Vec2::new(num20, y);
        if i == 0 {
            start = point;
            i += 1;
            continue;
        }
        // `:95-99`: the first row snaps to the dungeon location when the
        // grid point falls on the wrong side of it.
        if num19 == 0 && (point.x - dungeon_location as f64) * num21 as f64 < 0.0 {
            start = Vec2::new(dungeon_location as f64, point.y);
            i += 1;
            continue;
        }
        // `:100-103`: the two NextDouble radius walks + the y jitter.
        num16 = lerp((num16 - num13).max(num8), (num16 + num13).min(num7), rng.next_f64());
        num17 = lerp((num17 - num13).max(num8), (num17 + num13).min(num7), rng.next_f64());
        let num22 = (num16 + num17) / 2.0;
        point.y += (num16 - num17) / 2.0;
        let num23 = i / num15;
        snake.add(ControlLine::new(0, start, point, start_radius, num22, num23, num23 as usize));
        start = point;
        start_radius = num22;
        // `:109-125`: the row-end curve back to the next row's start.
        if num18 == num2 - 1 && num19 != count - 1 {
            let anchor = point;
            let center = Vec2::lerp(anchor, anchor + grid_extent * Vec2::new(0.0, num4), 0.5);
            let mut num24 = num9;
            while num24 < 0.5 {
                let mut curved = anchor.rotated_by(std::f64::consts::TAU * num24 * num21 as f64, center);
                curved.x = lerp(curved.x, center.x, 1.0 - num10);
                let mut line = ControlLine::new(0, start, curved, start_radius, num22, num23, num23 as usize);
                line.curve_line = true;
                snake.add(line);
                start = curved;
                start_radius = num22;
                num24 += num9;
            }
            i += 1;
        }
        i += 1;
    }
    // `:127-131`: the tangents + the dungeon-side end caps + the
    // self-intersection sweep.
    snake.set_tangents();
    let num25 = if dungeon_side == DUNGEON_SIDE_LEFT { 1 } else { -1 };
    let n = snake.lines.len();
    snake.lines[0].start_tangent = Vec2::UNIT_X * num25 as f64;
    snake.lines[n - 1].end_tangent = Vec2::UNIT_X * num25 as f64;
    snake.adjust_tangents_to_prevent_self_intersection();
    snake
}

/// `DitherSnakePass.GenerateDungeonDitherSnake`
/// (`DitherSnakePass.cs:33-42`) — paint every line into the dungeon's outer
/// bounds hitbox. `rng` is the pass stream (the edge-dither rarity rolls).
pub(crate) fn generate_dungeon_dither_snake(
    world: &mut World,
    rng: &mut super::Rng,
    snake: &mut DitherSnake,
    styles: &[SnakeStyle],
    outer: &DungeonBounds,
    safe_distance: f64,
) {
    let bounds = outer.hitbox();
    for i in 0..snake.lines.len() {
        let mut line = snake.lines[i];
        line.paint_rect(world, rng, &snake.lines, styles, bounds, safe_distance);
        snake.lines[i] = line;
    }
}

/// `DitherSnakePass.ApplyPass` (`DitherSnakePass.cs:22-31`) — the two
/// dungeons' snakes: dungeon 0 over progress [0, 0.5), dungeon 1 over
/// [0.5, 1]. Each calculates then paints its own snake into its own outer
/// bounds. The port runs the two halves back-to-back (the progress window
/// is loading-screen-only, no behavioural role). `dungeons` carries the
/// per-dungeon (outer bounds, side, location, style count) the producer
/// chain (B5) publishes; `safe_distance` is the world-wide
/// `NormalizedDistanceSafeFromDither`.
pub(crate) fn apply_dither_snake_pass(
    world: &mut World,
    rng: &mut super::Rng,
    dungeons: &[DitherSnakeDungeon],
    styles: &[SnakeStyle],
    safe_distance: f64,
) -> Vec<DitherSnake> {
    let mut snakes = Vec::new();
    for d in dungeons {
        let mut snake = calculate_dungeon_dither_snake(
            rng,
            world.width,
            &d.outer,
            d.dungeon_side,
            d.dungeon_location,
            d.style_count,
            safe_distance,
        );
        generate_dungeon_dither_snake(world, rng, &mut snake, styles, &d.outer, safe_distance);
        snakes.push(snake);
    }
    snakes
}

/// The per-dungeon inputs `ApplyPass` reads from
/// `GenVars.CurrentDungeonGenVars` (`DitherSnakePass.cs:25-30`): the outer
/// potential bounds, the side, the location and the style-list count.
#[derive(Clone, Debug)]
pub(crate) struct DitherSnakeDungeon {
    pub outer: DungeonBounds,
    pub dungeon_side: i32,
    pub dungeon_location: i32,
    pub style_count: usize,
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

    /// `CalculateDungeonDitherSnake` builds a non-empty snake with the
    /// dungeon-side end caps and consumes exactly two NextDouble draws per
    /// non-skipped grid point (`DitherSnakePass.cs:44-133`). The a == b
    /// replay pins the determinism (the radius walks ride the pass stream).
    #[test]
    fn w4_f24_calculate_snake_is_deterministic() {
        let mut outer = DungeonBounds::new(4200, 1200);
        // A right-side dungeon's outer bounds (CreatePotentialDungeonBounds
        // shape, DungeonUtils.cs:2079-2084): x in [0.51·W, W-0.05·W].
        outer.set_bounds(2142, 100, 3990, 1030);
        let safe = 1.0 - 0.075 * (1200.0 / 1200.0);
        let build = || {
            let mut rng = crate::worldgen::noise::Rng::from_seed(42);
            calculate_dungeon_dither_snake(&mut rng, 4200, &outer, 1, 3990, 4, safe)
        };
        let a = build();
        let b = build();
        assert!(!a.lines.is_empty());
        assert_eq!(a.lines.len(), b.lines.len());
        for (la, lb) in a.lines.iter().zip(b.lines.iter()) {
            assert_eq!(la.start, lb.start);
            assert_eq!(la.end, lb.end);
            assert_eq!(la.start_radius, lb.start_radius);
            assert_eq!(la.end_radius, lb.end_radius);
            assert_eq!(la.start_tangent, lb.start_tangent);
            assert_eq!(la.end_tangent, lb.end_tangent);
        }
        // The dungeon-side end caps: right side (num25 = -1) -> -UnitX.
        assert_eq!(a.lines[0].start_tangent, -Vec2::UNIT_X);
        assert_eq!(a.last().end_tangent, -Vec2::UNIT_X);
        // Every line's style index is in the 4-style quad range.
        assert!(a.lines.iter().all(|l| l.style < 4));
    }
}
