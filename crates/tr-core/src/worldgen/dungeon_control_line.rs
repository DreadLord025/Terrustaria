//! `DungeonControlLine` (`GameContent/Biomes/DungeonControlLine.cs:9-285`)
//! — one segment of the dual-dungeon dither snake: a start/end point pair
//! with radii, tangents and normals, a progression stage and a biome style.
//! `Paint` stamps the style's brick tile + wall into every cell of the
//! segment's capsule, Bayer-dithering the style transition at the segment
//! ends and the unbreakable-wall tier at the capsule edge.
//!
//! W4-f24 lane B (the gated `Dual Dungeons Dither Snake` pass,
//! `WorldGen.cs:14755`). The C# links lines into a doubly-linked list via
//! `Next`/`Prev` object references and compares styles by reference
//! (`this.Prev.Style != this.Style`); the port keeps the lines in a `Vec`
//! and models the links as neighbour indices (`index ± 1`, the order
//! `DitherSnake.Add` builds) and the style as an index into the shared
//! styles slice — same identity semantics, no reference cycles.
//!
//! The `NormalizedDistanceSafeFromDither` static
//! (`DungeonControlLine.cs:35`, set from
//! `GenVars.DualDungeon_NormalizedDistanceSafeFromDither`,
//! `WorldGen.cs:11465`: `1.0 - 0.075 * (maxTilesY / 1200.0)`) rides as an
//! explicit parameter so the geometry stays a pure function of its inputs.
//!
//! UNWIRED until B6: no pipeline step calls `Paint` yet — both classic
//! world-hash pins stay; the `a == b` halves are untouched. The
//! `SquareTileFrame`/`SquareWallFrame` calls ride the port's deferred
//! post-pipeline framing seam (the brick kinds merge through their sets).

use super::dungeon_bounds::Rect;
use super::snake_math::{lerp, remap, Vec2};
use crate::tile::{Tile, FLAG_ACTIVE};
use crate::world::World;

/// `DungeonControlLine.StyleTransitionDitherWidth` (`:37`) — the const is
/// 0.5; the dither arms below use the literal 0.25/0.75 split the C# writes.
/// `DungeonControlLine.BorderWidth` (`:39`) — the 4-tile safe border the
/// `IsInsideBorder`/`GetPotentialRoomPosition` maths subtract.
const BORDER_WIDTH: f64 = 4.0;

/// The style fields `Paint`/`GetStyleWithDitheredTransition`/`SkipPaintForEdge`
/// read (`DungeonGenerationStyleData.cs:11-95`). The full style table (the
/// furniture rows, chests, windows) is c32's lane + B5; this slice carries
/// only what the snake's paint path consumes.
#[derive(Clone, Copy, Debug)]
pub(crate) struct SnakeStyle {
    /// `BrickTileType` (`DungeonGenerationStyleData.cs:15`).
    pub brick_tile: u16,
    /// `BrickWallType` (`:21`).
    pub brick_wall: u16,
    /// `UnbreakableWallProgressionTier` (`:13`, default −1).
    pub unbreakable_tier: i32,
    /// `EdgeDither` (`:91`).
    pub edge_dither: bool,
}

/// `DualDungeonUnbreakableWallTiers.EarlyGame`
/// (`DualDungeonUnbreakableWallTiers.cs:19`).
const EARLY_GAME: i32 = 0;

/// `DitherSnakePass._bayerDither` (`DitherSnakePass.cs:137-143`) — the 4×4
/// ordered-dither matrix, indexed `[x % 4, y % 4]` (row-major as written).
pub(crate) const BAYER_DITHER: [[f64; 4]; 4] = [
    [0.0, 0.5, 0.125, 0.625],
    [0.75, 0.25, 0.875, 0.375],
    [0.1875, 0.6875, 0.0625, 0.5625],
    [0.9375, 0.4375, 0.8125, 0.3125],
];

/// `DungeonControlLine` (`DungeonControlLine.cs:9-285`). `index` is the
/// line's position in the snake `Vec` (the C# `Index`, set by
/// `DitherSnake.Add`); `prev`/`next` are the neighbour indices (`None` at
/// the ends). `style` indexes the shared styles slice.
#[derive(Clone, Copy, Debug)]
pub(crate) struct ControlLine {
    pub index: usize,
    pub start: Vec2,
    pub end: Vec2,
    pub start_tangent: Vec2,
    pub end_tangent: Vec2,
    pub start_normal: Vec2,
    pub end_normal: Vec2,
    pub cross_tangent: f64,
    pub start_radius: f64,
    pub end_radius: f64,
    pub normalized_line_direction: Vec2,
    pub line_length: f64,
    pub style: usize,
    pub progression_stage: i32,
    pub curve_line: bool,
}

impl ControlLine {
    /// `DungeonControlLine(start, end, startRadius, endRadius,
    /// progressionStage, style)` (`:58-69`) — the tangents/normals start
    /// zero (the C# fields default) until `SetTangents`/`CacheNormals`.
    pub fn new(
        index: usize,
        start: Vec2,
        end: Vec2,
        start_radius: f64,
        end_radius: f64,
        progression_stage: i32,
        style: usize,
    ) -> Self {
        let v = end - start;
        let line_length = v.length();
        let normalized_line_direction = v.safe_normalize(Vec2::UNIT_X);
        ControlLine {
            index,
            start,
            end,
            start_tangent: Vec2::ZERO,
            end_tangent: Vec2::ZERO,
            start_normal: Vec2::ZERO,
            end_normal: Vec2::ZERO,
            cross_tangent: 0.0,
            start_radius,
            end_radius,
            normalized_line_direction,
            line_length,
            style,
            progression_stage,
            curve_line: false,
        }
    }

    /// `DungeonControlLine.Center` (`:51`).
    pub fn center(&self) -> Vec2 {
        (self.end + self.start) / 2.0
    }

    fn prev(&self) -> Option<usize> {
        self.index.checked_sub(1)
    }

    fn next(&self, len: usize) -> Option<usize> {
        if self.index + 1 < len {
            Some(self.index + 1)
        } else {
            None
        }
    }

    /// `CacheNormals()` (`:71-76`) — the start normal is the start tangent
    /// rotated −90° `(y, -x)`, the end normal the end tangent rotated +90°
    /// `(-y, x)`; `CrossTangent` is their cross.
    fn cache_normals(&mut self) {
        self.start_normal = Vec2::new(self.start_tangent.y, -self.start_tangent.x);
        self.end_normal = Vec2::new(-self.end_tangent.y, self.end_tangent.x);
        self.cross_tangent = Vec2::cross(self.start_tangent, self.end_tangent);
    }

    /// `CanPaint(int x, int y, out distance, out normalizedLineProgress)`
    /// (`:78-115`) — returns `Some((distance, progress))` when the point
    /// projects onto this segment (or onto an end cap with no neighbour),
    /// `None` when a neighbour owns it. The `num`/`num2` tangent dots and
    /// the `num6` blend fraction are verbatim.
    pub fn can_paint(&self, lines: &[ControlLine], x: i32, y: i32) -> Option<(f64, f64)> {
        let p = Vec2::new(x as f64, y as f64);
        let value = p - self.start;
        let mut num = Vec2::dot(value, self.start_tangent);
        if num < 0.0 {
            if self.prev().is_some() {
                return None;
            }
            return Some((value.length(), 0.0));
        }
        let value2 = p - self.end;
        let num2 = Vec2::dot(value2, self.end_tangent);
        if num2 < 0.0 {
            if self.next(lines.len()).is_some() {
                return None;
            }
            return Some((value2.length(), 1.0));
        }
        let num3 = Vec2::dot(value, self.start_normal);
        let num4 = Vec2::dot(value2, self.end_normal);
        let num5 = (num + num2) / 2.0;
        num *= num;
        let num6 = num / (num + num2 * num2);
        let distance =
            (num3 * (1.0 - num6) + num4 * num6 - num5 * self.cross_tangent * num6 * (1.0 - num6))
                .abs();
        Some((distance, num6))
    }

    /// `GetStyleWithDitheredTransition(int x, int y, double progress)`
    /// (`:157-171`) — the first/last quarter of a segment Bayer-dithers
    /// toward the neighbour's style when it differs. Returns the style
    /// index to paint.
    fn style_with_dithered_transition(
        &self,
        lines: &[ControlLine],
        styles: &[SnakeStyle],
        x: i32,
        y: i32,
        progress: f64,
    ) -> usize {
        let bayer = BAYER_DITHER[(x % 4) as usize][(y % 4) as usize];
        if progress < 0.25 {
            if let Some(pi) = self.prev() {
                let prev = &lines[pi];
                if prev.style != self.style
                    && remap(progress, 0.0, 0.25, 0.5, 1.0) <= bayer
                {
                    return prev.style;
                }
            }
        } else if progress > 0.75 {
            if let Some(ni) = self.next(lines.len()) {
                let next = &lines[ni];
                if next.style != self.style
                    && remap(progress, 0.75, 1.0, 0.0, 0.5) >= bayer
                {
                    return next.style;
                }
            }
        }
        let _ = styles;
        self.style
    }

    /// `SkipPaintForEdge(int x, int y, style, normalizedDistanceForPoint)`
    /// (`:173-215`) — the capsule-edge dither. Inside the safe distance the
    /// cell always paints; beyond it the non-`EdgeDither` styles keep a
    /// hard 0.25 cutoff, the `EdgeDither` styles run the Bayer + the four
    /// `genRand` rarity rolls (NextFloat, Next(3), Next(2), Next(4)) in the
    /// C#'s order. `rng` is the pass stream (`WorldGen.genRand`).
    fn skip_paint_for_edge(
        world: &World,
        rng: &mut super::Rng,
        x: i32,
        y: i32,
        style: &SnakeStyle,
        normalized_distance: f64,
        safe_distance: f64,
    ) -> bool {
        if normalized_distance <= safe_distance {
            return false;
        }
        let num = 1.0 - (normalized_distance - safe_distance) / (1.0 - safe_distance);
        if !style.edge_dither {
            return num < 0.25;
        }
        if !in_world(world, x, y, 5) {
            return false;
        }
        // `tile != null && !tile.active()` — the port's tiles always exist
        // in-world, so this is the inactive test.
        if world.tile(x, y).is_some_and(|t| !t.active()) {
            return true;
        }
        let bayer = BAYER_DITHER[(x % 4) as usize][(y % 4) as usize];
        if num <= bayer {
            return true;
        }
        let num2 = lerp(0.0, 0.949999988079071, 1.0 - num);
        if rng.next_f64() <= num2 {
            return true;
        }
        if num <= 3.0 / 32.0 && rng.range(0, 3) != 0 {
            return true;
        }
        if num <= 0.125 && rng.range(0, 2) == 0 {
            return true;
        }
        if num <= 5.0 / 32.0 && rng.range(0, 4) == 0 {
            return true;
        }
        false
    }

    /// `Paint(int x, int y)` (`:117-155`) — stamp one cell: clear, set the
    /// dithered style's brick tile + wall, then the unbreakable-wall tier
    /// band (wall 350 + the tier paint, +16 in the inner 2 tiles).
    /// `safe_distance` is `NormalizedDistanceSafeFromDither`.
    pub fn paint_cell(
        &self,
        world: &mut World,
        rng: &mut super::Rng,
        lines: &[ControlLine],
        styles: &[SnakeStyle],
        x: i32,
        y: i32,
        safe_distance: f64,
    ) {
        // The C# reads the cached normals off the shared object; the port
        // reads them from the synced slice copy (paint_rect writes them in).
        let me = &lines[self.index];
        let Some((distance, progress)) = me.can_paint(lines, x, y) else {
            return;
        };
        let num = lerp(me.start_radius, me.end_radius, progress);
        let num2 = distance / num;
        if num2 > 1.0 {
            return;
        }
        let style_idx = me.style_with_dithered_transition(lines, styles, x, y, progress);
        let style = &styles[style_idx];
        if Self::skip_paint_for_edge(world, rng, x, y, style, num2, safe_distance) {
            return;
        }
        if !in_world(world, x, y, 0) {
            return;
        }
        let i = world.index(x, y);
        // `tile.ClearEverything()` (`Tile.cs:110-121`) then active + type +
        // wall — the port's Tile reset (frames 0, flags/meta cleared).
        world.tiles[i] = Tile {
            kind: style.brick_tile,
            wall: style.brick_wall,
            flags: FLAG_ACTIVE,
            ..Default::default()
        };
        if style.unbreakable_tier <= EARLY_GAME {
            return;
        }
        let num3 = (num * safe_distance) as i32;
        let num4 = distance - num3 as f64;
        if num4 >= -4.0 && num4 <= 0.0 {
            let mut num5 = style.unbreakable_tier;
            if num4 <= -2.0 {
                num5 += 16;
            }
            world.tiles[i].wall = 350;
            world.set_wall_paint(x, y, num5 as u8);
        }
    }

    /// `Paint(Rectangle dungeonBounds)` (`:217-233`) — cache the normals,
    /// build the per-line paint box (the union of the two endpoint 1×1
    /// rects inflated by the max radius), clip to the dungeon bounds, and
    /// paint every cell column-major (`i` outer, `j` inner — the C# order).
    pub fn paint_rect(
        &mut self,
        world: &mut World,
        rng: &mut super::Rng,
        lines: &[ControlLine],
        styles: &[SnakeStyle],
        dungeon_bounds: Rect,
        safe_distance: f64,
    ) {
        self.cache_normals();
        let num = self.start_radius.max(self.end_radius);
        let (sx, sy) = self.start.to_point();
        let (ex, ey) = self.end.to_point();
        let value = Rect::union(Rect::new(sx, sy, 1, 1), Rect::new(ex, ey, 1, 1));
        let value = value.inflate(num as i32, num as i32);
        let rectangle = Rect::intersect(value, dungeon_bounds);
        // The C# `Paint(Rectangle)` mutates the shared line object's normals
        // (`CacheNormals`) and every cell reads them through the list links;
        // the port syncs the cached normals into a slice copy the cell loop
        // reads, then writes the painted line back.
        let mut local = lines.to_vec();
        local[self.index] = *self;
        let idx = self.index;
        for i in rectangle.left()..=rectangle.right() {
            for j in rectangle.top()..=rectangle.bottom() {
                let line = local[idx];
                line.paint_cell(world, rng, &local, styles, i, j, safe_distance);
            }
        }
        *self = local[idx];
    }

    /// `IsSelfIntersecting()` (`:235-247`) — the capsule's two normal-line
    /// intersections fall inside the radii.
    pub fn is_self_intersecting(&mut self) -> bool {
        self.cache_normals();
        let num = Vec2::cross(self.start_normal, self.end_normal);
        let value = self.end - self.start;
        let value2 = Vec2::cross(value, self.end_normal) / num;
        let value3 = Vec2::cross(value, self.start_normal) / num;
        if !(value2.abs() < self.start_radius) {
            return value3.abs() < self.end_radius;
        }
        true
    }

    /// `IsInsideBorder(Point)` (`:270-277`) — the point is inside the safe
    /// capsule border (the radius lerp × safe distance − the 4-tile border).
    pub fn is_inside_border(&self, lines: &[ControlLine], x: i32, y: i32, safe_distance: f64) -> bool {
        if let Some((distance, progress)) = self.can_paint(lines, x, y) {
            return distance
                < lerp(self.start_radius, self.end_radius, progress) * safe_distance - BORDER_WIDTH;
        }
        false
    }

    /// `GetPotentialRoomPosition(normalizedDistanceAlong, normalizedOffset,
    /// roomRadius)` (`:279-284`) — a room centre offset along the lerped
    /// normal by the safe-radius minus the border minus the room radius.
    pub fn potential_room_position(
        &self,
        normalized_distance_along: f64,
        normalized_offset: f64,
        room_radius: i32,
        safe_distance: f64,
    ) -> Vec2 {
        let normal = Vec2::lerp(self.start_normal, self.end_normal, normalized_distance_along)
            .safe_normalize(Vec2::ZERO);
        let num = lerp(self.start_radius, self.end_radius, normalized_distance_along)
            * safe_distance
            - BORDER_WIDTH
            - room_radius as f64;
        Vec2::lerp(self.start, self.end, normalized_distance_along) + normal * num * normalized_offset
    }
}

/// `WorldGen.InWorld(int, int, int fluff)` (`WorldGen.cs:9688-9695`).
fn in_world(world: &World, x: i32, y: i32, fluff: i32) -> bool {
    x >= fluff && x < world.width - fluff && y >= fluff && y < world.height - fluff
}

/// `AdjustTangentsToPreventSelfIntersection()` (`:249-268`) — when the line
/// self-intersects, pull each tangent half-way away from the other and
/// propagate the negated tangent to the neighbour. Lives on the snake
/// slice because it mutates `prev`/`next` (the C# object links).
pub(crate) fn adjust_tangents(lines: &mut [ControlLine], index: usize) -> bool {
    let mut line = lines[index];
    // `IsSelfIntersecting` calls `CacheNormals` on the shared object even
    // when it returns false — the cached normals persist either way.
    let intersecting = line.is_self_intersecting();
    lines[index] = line;
    if !intersecting {
        return false;
    }
    let start_tangent = (line.start_tangent - line.end_tangent / 2.0).safe_normalize(Vec2::ZERO);
    let end_tangent = (line.end_tangent - line.start_tangent / 2.0).safe_normalize(Vec2::ZERO);
    if let Some(pi) = line.prev() {
        line.start_tangent = start_tangent;
        lines[pi].end_tangent = -start_tangent;
    }
    if let Some(ni) = line.next(lines.len()) {
        line.end_tangent = end_tangent;
        lines[ni].start_tangent = -end_tangent;
    }
    lines[index] = line;
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    fn line(index: usize, start: Vec2, end: Vec2, style: usize) -> ControlLine {
        ControlLine::new(index, start, end, 10.0, 10.0, 0, style)
    }

    /// `CanPaint` projects a point onto the segment and blends the normal
    /// distance (`DungeonControlLine.cs:78-115`); a point past an end with
    /// a neighbour returns None.
    #[test]
    fn w4_f24_can_paint_projects_and_blends() {
        // A horizontal segment (0,0)->(10,0), tangents as SetTangents writes
        // them for a single line: start +X, end -X (the negated direction).
        let mut a = line(0, Vec2::new(0.0, 0.0), Vec2::new(10.0, 0.0), 0);
        a.start_tangent = Vec2::UNIT_X;
        a.end_tangent = -Vec2::UNIT_X;
        a.cache_normals();
        let lines = [a];
        // Midpoint (5,0): distance 0, progress 0.5.
        let (d, p) = a.can_paint(&lines, 5, 0).unwrap();
        assert!(d.abs() < 1e-9);
        assert!((p - 0.5).abs() < 1e-9);
        // (5,3): distance 3, progress 0.5.
        let (d, _) = a.can_paint(&lines, 5, 3).unwrap();
        assert!((d - 3.0).abs() < 1e-9);
        // Before the start with no prev: end-cap, progress 0.
        let (_, p) = a.can_paint(&lines, -2, 0).unwrap();
        assert_eq!(p, 0.0);
    }

    /// The Bayer matrix is the C# static ctor verbatim
    /// (`DitherSnakePass.cs:137-143`).
    #[test]
    fn w4_f24_bayer_matrix_is_verbatim() {
        assert_eq!(BAYER_DITHER[0][0], 0.0);
        assert_eq!(BAYER_DITHER[0][2], 0.125);
        assert_eq!(BAYER_DITHER[3][0], 0.9375);
        assert_eq!(BAYER_DITHER[2][3], 0.5625);
    }

    /// `IsInsideBorder` subtracts the safe-distance radius and the 4-tile
    /// border (`DungeonControlLine.cs:270-277`).
    #[test]
    fn w4_f24_is_inside_border_safe_radius() {
        let mut a = line(0, Vec2::new(0.0, 0.0), Vec2::new(10.0, 0.0), 0);
        a.start_tangent = Vec2::UNIT_X;
        a.end_tangent = -Vec2::UNIT_X;
        a.cache_normals();
        let lines = [a];
        // radius 10, safe 0.9 -> 9 - 4 = 5; (5,4) inside, (5,6) outside.
        assert!(a.is_inside_border(&lines, 5, 4, 0.9));
        assert!(!a.is_inside_border(&lines, 5, 6, 0.9));
    }
}
