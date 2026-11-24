//! The dual-dungeons dither snake (W4-c30) тАФ the port of
//! `DitherSnake.cs:10-199` (the control-line chain + the border/room
//! queries), `DitherSnakePass.cs:13-145` (the snake builder, the paint
//! driver and the 4x4 Bayer matrix), `DungeonControlLine.cs:9-285` (the
//! per-line paint + the tangent/self-intersection machinery), plus the
//! setup slices the pass consumes: the dual-dungeon style lists
//! (`DungeonCrawler.cs:111-127` over `DungeonGenerationStyles.cs`'s
//! statics тАФ the paint-relevant fields only), `CreatePotentialDungeonBounds`
//! (`DungeonUtils.cs:2067-2087` with `DungeonBounds.cs`'s clamped
//! setters/hitbox) and the Reset dual block (`WorldGen.cs:11441-11466`).
//!
//! Structure mapping: the C#'s `DitherSnake : List<DungeonControlLine>`
//! with per-line `Prev`/`Next` object links becomes a `Vec<ControlLine>`
//! whose neighbours are `index ┬▒ 1` (the `Add` override links consecutive
//! entries only, `DitherSnake.cs:16-26`); `Style` identity comparisons
//! (`==` on the shared statics) become the `StyleId` enum's equality тАФ the
//! style-0 entry is one per dungeon colour, so two lists' style-0 entries
//! never meet inside one snake. The per-pass `genRand` is the port's
//! isolated `pass_rng("Dual Dungeons Dither Snake")` stream (the recorded
//! per-pass deviation тАФ a classic world never runs the pass).

// The border/room query half (`position_along_snake`, the circle/line
// border tests, `room_position_inside_border`, `is_circle_inside_border_
// with_style`) has no consumer until the dual-dungeon crawler lands
// (W4-c32+) тАФ the paint driver exercises the rest. Kept compiled and
// citation-complete, dead-code-silenced until the crawler wires them.
#![allow(dead_code)]

use super::dungeon::DungeonColor;
// Restored from 37c0cdc82:21 — lost when 59ae0d914 rebuilt this file from the
// land-P parent; circle_test_points (kept as a train-only extra) needs it.
use super::snake_math::Vec2;
use super::Rng;
use crate::tile::{Tile, FLAG_ACTIVE};
use crate::world::World;

/// `DitherSnakePass._bayerDither` (`DitherSnakePass.cs:135-144`) тАФ the 4x4
/// ordered-dither matrix, indexed `[x % 4, y % 4]` with the row-major
/// literal below (the C# `new double[4,4]` initializer is row-major).
pub(crate) const BAYER_DITHER: [[f64; 4]; 4] = [
    [0.0, 0.5, 0.125, 0.625],
    [0.75, 0.25, 0.875, 0.375],
    [0.1875, 0.6875, 0.0625, 0.5625],
    [0.9375, 0.4375, 0.8125, 0.3125],
];

/// The per-dungeon inputs `ApplyPass` reads from
/// `GenVars.CurrentDungeonGenVars` (`DitherSnakePass.cs:25-30`): the outer
/// potential bounds, the side, the location and the style-list count.
/// Restored from the introducing commit `fd8166659:511-520` — the 59ae0d914
/// rebuild dropped this seam while dungeon_genvars.rs:24/255/261 still uses it.
#[derive(Clone, Debug)]
pub(crate) struct DitherSnakeDungeon {
    pub outer: super::dungeon_bounds::DungeonBounds,
    pub dungeon_side: i32,
    pub dungeon_location: i32,
    pub style_count: usize,
}

/// `DualDungeonUnbreakableWallTiers` (`DualDungeonUnbreakableWallTiers.cs:17-25`).
pub(crate) mod wall_tiers {
    pub const EARLY_GAME: i32 = 0;
    pub const EVIL_BOSS: i32 = 1;
    pub const JUNGLE_BOSS: i32 = 2;
    pub const DUNGEON: i32 = 3;
    pub const HALLOW: i32 = 4;
    pub const TEMPLE: i32 = 5;
}

/// The style identity of one `DungeonGenerationStyleData` entry тАФ the
/// fields `DungeonControlLine.Paint`/`SkipPaintForEdge` read. The C#
/// compares `Style` by reference (the shared statics); the port compares
/// this enum. `Dungeon0` carries the rolled colour's brick/wall pair
/// (`GetCurrentDungeonStyle`, `DungeonGenerationStyles.cs:95-135` over
/// `DungeonCrawler.cs:63-92`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum StyleId {
    /// `GetCurrentDungeonStyle` тАФ tier Dungeon(3), EdgeDither false.
    Dungeon0(DungeonColor),
    /// `DungeonGenerationStyles.Cavern` (`:307-351`).
    Cavern,
    /// `DungeonGenerationStyles.Corruption` (`:434-473`).
    Corruption,
    /// `DungeonGenerationStyles.Crimson` (`:474-513`).
    Crimson,
    /// `DungeonGenerationStyles.Jungle` (`:714-759`).
    Jungle,
    /// `DungeonGenerationStyles.Snow` (`:352-392`).
    Snow,
    /// `DungeonGenerationStyles.Desert` (`:393-433`).
    Desert,
    /// `DungeonGenerationStyles.Hallow` (`:553-593`).
    Hallow,
    /// `DungeonGenerationStyles.Temple` (`:760-797`) тАФ a `TempleStyleData`
    /// whose initializer sets no tier, so the field default тИТ1
    /// (`DungeonGenerationStyleData.cs:13`) stands.
    Temple,
}

/// The paint-relevant slice of `DungeonGenerationStyleData`
/// (`BrickTileType`/`BrickWallType`/`UnbreakableWallProgressionTier`/
/// `EdgeDither`). The furniture rows/sub-styles/feature gates are W4-c32's
/// table scope, not the snake's.
#[derive(Clone, Copy, Debug)]
pub(crate) struct SnakeStyle {
    pub id: StyleId,
    pub brick_tile: u16,
    pub brick_wall: u16,
    pub tier: i32,
    pub edge_dither: bool,
}

impl SnakeStyle {
    fn of(id: StyleId) -> SnakeStyle {
        // The statics' (Style, tier, tile, wall, EdgeDither) quintuples тАФ
        // DungeonGenerationStyles.cs anchors on each arm.
        let (brick_tile, brick_wall, tier, edge_dither) = match id {
            StyleId::Dungeon0(c) => (c.brick_tile(), c.brick_wall(), wall_tiers::DUNGEON, false),
            StyleId::Cavern => (38, 349, wall_tiers::EARLY_GAME, true), // :310-343
            StyleId::Corruption => (25, 3, wall_tiers::EVIL_BOSS, true), // :437-471
            StyleId::Crimson => (203, 83, wall_tiers::EVIL_BOSS, true), // :477-511
            StyleId::Jungle => (59, 64, wall_tiers::JUNGLE_BOSS, true), // :717-752
            StyleId::Snow => (161, 71, wall_tiers::EARLY_GAME, true),  // :355-390
            StyleId::Desert => (396, 187, wall_tiers::EARLY_GAME, false), // :396-431
            StyleId::Hallow => (117, 28, wall_tiers::HALLOW, true),    // :556-590
            StyleId::Temple => (226, 87, -1, false),                   // :760-795
        };
        SnakeStyle { id, brick_tile, brick_wall, tier, edge_dither }
    }
}

/// `DungeonCrawler.SetupDungeonGenVarVariables`'s dual-dungeon style lists
/// (`DungeonCrawler.cs:111-127`): dungeon 0 walks Cavern тЖТ the world's evil
/// тЖТ Jungle тЖТ the colour style; dungeon 1 walks Snow тЖТ Desert тЖТ Hallow тЖТ
/// Temple. The classic single-entry list (`:129-132`) never reaches the
/// snake pass (it is dualDungeons-gated).
pub(crate) fn dual_style_list(current_dungeon: i32, crimson: bool, color: DungeonColor) -> Vec<SnakeStyle> {
    let ids: [StyleId; 4] = if current_dungeon % 2 == 0 {
        let evil = if crimson { StyleId::Crimson } else { StyleId::Corruption };
        [StyleId::Cavern, evil, StyleId::Jungle, StyleId::Dungeon0(color)]
    } else {
        [StyleId::Snow, StyleId::Desert, StyleId::Hallow, StyleId::Temple]
    };
    ids.iter().map(|&id| SnakeStyle::of(id)).collect()
}

/// `SnakeOrientation` (`SnakeOrientation.cs:3-9`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SnakeOrientation {
    Unknown,
    Top,
    Center,
    Bottom,
}

/// The C# `Vector2D` (double pair) тАФ only the operations the snake family
/// uses (`Vector2D.cs`: Length :70-73, Dot :194-197, Cross :204-207, Lerp
/// :222-225/:432-435, Normalize :294-300; `Utils.cs`: SafeNormalize
/// :1846-1853, RotatedBy :1686-1695, ToPoint :1822-1825).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct Vec2D {
    pub x: f64,
    pub y: f64,
}

impl Vec2D {
    pub const ZERO: Vec2D = Vec2D { x: 0.0, y: 0.0 };
    pub const UNIT_X: Vec2D = Vec2D { x: 1.0, y: 0.0 };

    pub fn new(x: f64, y: f64) -> Vec2D {
        Vec2D { x, y }
    }

    fn length(self) -> f64 {
        (self.x * self.x + self.y * self.y).sqrt()
    }

    fn dot(self, o: Vec2D) -> f64 {
        self.x * o.x + self.y * o.y
    }

    fn cross(self, o: Vec2D) -> f64 {
        self.x * o.y - self.y * o.x
    }

    /// `Vector2D.Normalize` (`:294-300`) тАФ the reciprocal-sqrt multiply,
    /// NOT a divide (bit-exactness with the C# expression order).
    fn normalize(self) -> Vec2D {
        let n = 1.0 / (self.x * self.x + self.y * self.y).sqrt();
        Vec2D::new(self.x * n, self.y * n)
    }

    /// `Utils.SafeNormalize` (`Utils.cs:1846-1853`): only the exact zero
    /// vector falls back (no NaN guard on the double overload).
    fn safe_normalize(self, default: Vec2D) -> Vec2D {
        if self == Vec2D::ZERO {
            default
        } else {
            self.normalize()
        }
    }

    /// `Utils.RotatedBy` (`Utils.cs:1686-1695`) тАФ rotate about `center`.
    fn rotated_by(self, radians: f64, center: Vec2D) -> Vec2D {
        let (c, s) = (radians.cos(), radians.sin());
        let v = self - center;
        Vec2D::new(center.x + v.x * c - v.y * s, center.y + v.x * s + v.y * c)
    }

    /// `Utils.Lerp` (`Utils.cs:107-110`) per component (`Vector2D.Lerp`).
    fn lerp(self, o: Vec2D, t: f64) -> Vec2D {
        Vec2D::new(lerp(self.x, o.x, t), lerp(self.y, o.y, t))
    }

    /// `Utils.ToPoint` (`Utils.cs:1822-1825`) тАФ the `(int)` truncation.
    fn to_point(self) -> (i32, i32) {
        (self.x as i32, self.y as i32)
    }
}

impl std::ops::Add for Vec2D {
    type Output = Vec2D;
    fn add(self, o: Vec2D) -> Vec2D {
        Vec2D::new(self.x + o.x, self.y + o.y)
    }
}

impl std::ops::Sub for Vec2D {
    type Output = Vec2D;
    fn sub(self, o: Vec2D) -> Vec2D {
        Vec2D::new(self.x - o.x, self.y - o.y)
    }
}

impl std::ops::Neg for Vec2D {
    type Output = Vec2D;
    fn neg(self) -> Vec2D {
        Vec2D::new(-self.x, -self.y)
    }
}

/// `operator *(Vector2D, Vector2D)` (`Vector2D.cs:384-389`) тАФ componentwise.
impl std::ops::Mul for Vec2D {
    type Output = Vec2D;
    fn mul(self, o: Vec2D) -> Vec2D {
        Vec2D::new(self.x * o.x, self.y * o.y)
    }
}

impl std::ops::Mul<f64> for Vec2D {
    type Output = Vec2D;
    fn mul(self, s: f64) -> Vec2D {
        Vec2D::new(self.x * s, self.y * s)
    }
}

impl std::ops::Div<f64> for Vec2D {
    type Output = Vec2D;
    fn div(self, s: f64) -> Vec2D {
        Vec2D::new(self.x / s, self.y / s)
    }
}

/// `Utils.Lerp(double, double, double)` (`Utils.cs:107-110`).
pub(crate) fn lerp(a: f64, b: f64, t: f64) -> f64 {
    a + (b - a) * t
}

/// `Utils.Remap` with `clamped: true` (`Utils.cs:350-353` +
/// `GetLerpValue` `:730-758` тАФ the clamp arms, then the unclamped fraction).
fn remap(v: f64, from_min: f64, from_max: f64, to_min: f64, to_max: f64) -> f64 {
    let t = if from_min < from_max {
        if v < from_min {
            0.0
        } else if v > from_max {
            1.0
        } else {
            (v - from_min) / (from_max - from_min)
        }
    } else if v < from_max {
        1.0
    } else if v > from_min {
        0.0
    } else {
        (v - from_min) / (from_max - from_min)
    };
    lerp(to_min, to_max, t)
}

/// One `DungeonControlLine` (`DungeonControlLine.cs:9-285`). `Index` is the
/// snake-vec position; `Prev`/`Next` are `index тИТ 1`/`index + 1` (the `Add`
/// override chains consecutive lines only). The normals/cross-tangent are
/// cached by `Paint(Rectangle)`/`IsSelfIntersecting` in the C#; the port
/// recomputes them per query (a pure function of the tangents тАФ same
/// values, no stored-state drift).
#[derive(Clone, Debug)]
pub(crate) struct ControlLine {
    pub start: Vec2D,
    pub end: Vec2D,
    pub start_tangent: Vec2D,
    pub end_tangent: Vec2D,
    pub start_radius: f64,
    pub end_radius: f64,
    /// `NormalizedLineDirection` тАФ `SafeNormalize(UnitX)` in the ctor
    /// (`:66-68`).
    pub direction: Vec2D,
    pub style: SnakeStyle,
    /// `ProgressionStage` (`:64`) тАФ the style-band index the builder passes.
    pub progression_stage: i32,
    pub curve_line: bool,
}

impl ControlLine {
    /// The public ctor (`DungeonControlLine.cs:58-69`).
    pub fn new(
        start: Vec2D,
        end: Vec2D,
        start_radius: f64,
        end_radius: f64,
        progression_stage: i32,
        style: SnakeStyle,
    ) -> ControlLine {
        let v = end - start;
        ControlLine {
            start,
            end,
            start_tangent: Vec2D::ZERO,
            end_tangent: Vec2D::ZERO,
            start_radius,
            end_radius,
            direction: v.safe_normalize(Vec2D::UNIT_X),
            style,
            progression_stage,
            curve_line: false,
        }
    }

    /// `Center` (`:51`) тАФ `(End + Start) / 2.0`.
    fn center(&self) -> Vec2D {
        (self.end + self.start) / 2.0
    }

    fn start_normal(&self) -> Vec2D {
        // `CacheNormals` (`:71-76`).
        Vec2D::new(self.start_tangent.y, 0.0 - self.start_tangent.x)
    }

    fn end_normal(&self) -> Vec2D {
        Vec2D::new(0.0 - self.end_tangent.y, self.end_tangent.x)
    }

    fn cross_tangent(&self) -> f64 {
        self.start_tangent.cross(self.end_tangent)
    }

    /// `CanPaint` (`:78-115`) тАФ returns `(distance, normalizedLineProgress)`
    /// or `None` when the point belongs to a neighbour line. `has_prev`/
    /// `has_next` stand in for the C#'s null link tests.
    fn can_paint(
        &self,
        x: i32,
        y: i32,
        has_prev: bool,
        has_next: bool,
    ) -> Option<(f64, f64)> {
        let p = Vec2D::new(x as f64, y as f64);
        let value = p - self.start;
        let mut num = value.dot(self.start_tangent);
        if num < 0.0 {
            if has_prev {
                return None;
            }
            return Some((value.length(), 0.0));
        }
        let value2 = p - self.end;
        let num2 = value2.dot(self.end_tangent);
        if num2 < 0.0 {
            if has_next {
                return None;
            }
            return Some((value2.length(), 1.0));
        }
        let num3 = value.dot(self.start_normal());
        let num4 = value2.dot(self.end_normal());
        let num5 = (num + num2) / 2.0;
        num *= num;
        let num6 = num / (num + num2 * num2);
        let distance =
            (num3 * (1.0 - num6) + num4 * num6 - num5 * self.cross_tangent() * num6 * (1.0 - num6))
                .abs();
        Some((distance, num6))
    }

    /// `GetStyleWithDitheredTransition` (`:157-171`) тАФ the Bayer-ordered
    /// blend across a style change. The caller hands the neighbour styles
    /// (the C# reads `Prev.Style`/`Next.Style`).
    fn style_with_dithered_transition(
        &self,
        x: i32,
        y: i32,
        progress: f64,
        prev: Option<SnakeStyle>,
        next: Option<SnakeStyle>,
    ) -> SnakeStyle {
        let bayer = BAYER_DITHER[(x % 4) as usize][(y % 4) as usize];
        if progress < 0.25 {
            if let Some(p) = prev {
                if p.id != self.style.id
                    && remap(progress, 0.0, 0.25, 0.5, 1.0) <= bayer
                {
                    return p;
                }
            }
        } else if progress > 0.75 {
            if let Some(n) = next {
                if n.id != self.style.id
                    && remap(progress, 0.75, 1.0, 0.0, 0.5) >= bayer
                {
                    return n;
                }
            }
        }
        self.style
    }

    /// `IsSelfIntersecting` (`:235-247`) тАФ the normals' cross-division
    /// test against the end radii.
    fn is_self_intersecting(&self) -> bool {
        let (sn, en) = (self.start_normal(), self.end_normal());
        let num = sn.cross(en);
        let value = self.end - self.start;
        let value2 = value.cross(en) / num;
        let value3 = value.cross(sn) / num;
        if !(value2.abs() < self.start_radius) {
            return value3.abs() < self.end_radius;
        }
        true
    }

    /// `AdjustTangentsToPreventSelfIntersection` (`:249-268`) тАФ the local
    /// half; the neighbour writes ride [`DitherSnake::adjust_tangents`].
    /// Returns `(changed, new_start, new_end)`.
    fn adjust_tangents(&self) -> (bool, Vec2D, Vec2D) {
        if !self.is_self_intersecting() {
            return (false, self.start_tangent, self.end_tangent);
        }
        let st = (self.start_tangent - self.end_tangent / 2.0).safe_normalize(Vec2D::ZERO);
        let et = (self.end_tangent - self.start_tangent / 2.0).safe_normalize(Vec2D::ZERO);
        (true, st, et)
    }

    /// `IsInsideBorder` (`:270-277`) тАФ `BorderWidth` 4 (`:39`).
    fn is_inside_border(&self, x: i32, y: i32, has_prev: bool, has_next: bool, safe: f64) -> bool {
        match self.can_paint(x, y, has_prev, has_next) {
            Some((distance, progress)) => {
                distance < lerp(self.start_radius, self.end_radius, progress) * safe - 4.0
            }
            None => false,
        }
    }

    /// `GetPotentialRoomPosition` (`:279-284`).
    fn potential_room_position(
        &self,
        along: f64,
        offset: f64,
        room_radius: i32,
        safe: f64,
    ) -> Vec2D {
        let dir = self
            .start_normal()
            .lerp(self.end_normal(), along)
            .safe_normalize(Vec2D::ZERO);
        let num = lerp(self.start_radius, self.end_radius, along) * safe - 4.0 - room_radius as f64;
        self.start.lerp(self.end, along) + dir * num * offset
    }

    /// `Paint(int, int)` (`:117-155`) тАФ the tile write: ClearEverything тЖТ
    /// active brick + style wall, then the unbreakable-wall 350 band (the
    /// `wallColor` tier byte, +16 on the inner 2 px). `safe` is
    /// `DungeonControlLine.NormalizedDistanceSafeFromDither`.
    #[allow(clippy::too_many_arguments)]
    fn paint_tile(
        &self,
        world: &mut World,
        rng: &mut Rng,
        x: i32,
        y: i32,
        has_prev: bool,
        has_next: bool,
        prev: Option<SnakeStyle>,
        next: Option<SnakeStyle>,
        safe: f64,
    ) {
        let Some((distance, progress)) = self.can_paint(x, y, has_prev, has_next) else {
            return;
        };
        let num = lerp(self.start_radius, self.end_radius, progress);
        let num2 = distance / num;
        if num2 > 1.0 {
            return;
        }
        let style = self.style_with_dithered_transition(x, y, progress, prev, next);
        if skip_paint_for_edge(world, rng, x, y, style, num2, safe) {
            return;
        }
        let i = world.index(x, y);
        // `Tile.ClearEverything` (`Tile.cs:110-121`) тАФ every header resets,
        // so the wall paint/coating extras byte clears too.
        world.tiles[i] = Tile {
            kind: style.brick_tile,
            wall: style.brick_wall,
            flags: FLAG_ACTIVE,
            ..Default::default()
        };
        world.set_extras_at(i, 0);
        if style.tier <= wall_tiers::EARLY_GAME {
            return;
        }
        let num3 = (num * safe) as i32;
        let num4 = distance - num3 as f64;
        if (-4.0..=0.0).contains(&num4) {
            let mut num5 = style.tier;
            if num4 <= -2.0 {
                num5 += 16;
            }
            world.tiles[i].wall = 350;
            world.set_wall_paint(x, y, num5 as u8);
        }
    }

    /// `Paint(Rectangle)` (`:217-233`) тАФ CacheNormals, the start/end point
    /// union inflated by `(int)max(radius)`, intersected with the bounds,
    /// then the column-major tile walk. `bounds` is the outer potential
    /// dungeon hitbox; `None` = `Rectangle.Empty` (no hitbox тАФ the C#
    /// intersect empties and the loops never run).
    #[allow(clippy::too_many_arguments)]
    fn paint_rect(
        &self,
        world: &mut World,
        rng: &mut Rng,
        index: usize,
        len: usize,
        bounds: Option<(i32, i32, i32, i32)>,
        prev: Option<SnakeStyle>,
        next: Option<SnakeStyle>,
        safe: f64,
    ) {
        let Some((bl, bt, br, bb)) = bounds else {
            return;
        };
        let num = self.start_radius.max(self.end_radius);
        let (sx, sy) = self.start.to_point();
        let (ex, ey) = self.end.to_point();
        // `Rectangle.Union` of the two 1x1 point rects, `Inflate((int)num)`.
        let (mut l, mut t) = (sx.min(ex) - num as i32, sy.min(ey) - num as i32);
        let (mut r, mut b) = (sx.max(ex) + num as i32, sy.max(ey) + num as i32);
        // `Rectangle.Intersect` тАФ the empty case leaves the loops vacuous.
        l = l.max(bl);
        t = t.max(bt);
        r = r.min(br);
        b = b.min(bb);
        for i in l..=r {
            for j in t..=b {
                self.paint_tile(world, rng, i, j, index > 0, index + 1 < len, prev, next, safe);
            }
        }
    }
}

/// `DungeonControlLine.SkipPaintForEdge` (`:173-215`) тАФ the edge fade:
/// inside the safe band nothing skips; outside it the non-dither styles
/// hard-cut at 0.25, the dither styles Bayer-skip, then the four RNG
/// frays (`NextFloat тЙд 0.95┬╖(1тИТnum)`, `Next(3) != 0` under 3/32,
/// `Next(2) == 0` under 1/8, `Next(4) == 0` under 5/32). `NextFloat`'s
/// port convention is `next_f64` (the recorded fraction stand-in).
fn skip_paint_for_edge(
    world: &World,
    rng: &mut Rng,
    x: i32,
    y: i32,
    style: SnakeStyle,
    normalized_distance: f64,
    safe: f64,
) -> bool {
    if normalized_distance <= safe {
        return false;
    }
    let num = 1.0 - (normalized_distance - safe) / (1.0 - safe);
    if !style.edge_dither {
        return num < 0.25;
    }
    if !super::temple::in_world(world, x, y, 5) {
        return false;
    }
    let tile = &world.tiles[world.index(x, y)];
    if !tile.active() {
        return true;
    }
    if num <= BAYER_DITHER[(x % 4) as usize][(y % 4) as usize] {
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

/// `DitherSnake` (`DitherSnake.cs:10-199`) тАФ the linked chain as a vec.
#[derive(Clone, Debug, Default)]
pub(crate) struct DitherSnake {
    pub lines: Vec<ControlLine>,
}

impl DitherSnake {
    /// The `Add` override (`:16-26`) тАФ index = position, links consecutive.
    pub fn add(&mut self, line: ControlLine) {
        self.lines.push(line);
    }

    /// `GetClosestLineTo` (`:28-42`) тАФ the first strictly-closest centre.
    fn closest_line_to(&self, pos: Vec2D) -> usize {
        let mut result = 0;
        let mut num = f64::MAX;
        for (i, item) in self.lines.iter().enumerate() {
            let num2 = (item.center() - pos).length();
            if num2 < num {
                result = i;
                num = num2;
            }
        }
        result
    }

    /// `GetLineContaining` (`:44-63`) тАФ the depth-3 tangent walk; `None`
    /// at depth 3. `guess` defaults to the closest line.
    fn line_containing(&self, pos: Vec2D, guess: Option<usize>, depth: i32) -> Option<usize> {
        let guess = guess.unwrap_or_else(|| self.closest_line_to(pos));
        if depth == 3 {
            return None;
        }
        let line = &self.lines[guess];
        if (pos - line.start).dot(line.start_tangent) < 0.0 && guess > 0 {
            return self.line_containing(pos, Some(guess - 1), depth + 1);
        }
        if (pos - line.end).dot(line.end_tangent) < 0.0 && guess + 1 < self.lines.len() {
            return self.line_containing(pos, Some(guess + 1), depth + 1);
        }
        Some(guess)
    }

    /// `GetPositionAlongSnake` (`:65-73`) тАФ index + progress, the failed
    /// CanPaint falling back to 0.5. (The C# dereferences a null
    /// lineContaining unguarded; the depth-3 walk cannot return null for
    /// a point the closest-line guess starts from with < 3 lines of drift,
    /// and the port keeps the same reachable behaviour.)
    pub fn position_along_snake(&self, pos: Vec2D) -> f64 {
        let idx = self.line_containing(pos, None, 0).expect("line containing");
        let line = &self.lines[idx];
        let progress = match line.can_paint(pos.x as i32, pos.y as i32, idx > 0, idx + 1 < self.lines.len()) {
            Some((_, p)) => p,
            None => 0.5,
        };
        idx as f64 + progress
    }

    /// `IsCircleInsideBorderWithStyle` (`:75-83`).
    pub fn is_circle_inside_border_with_style(
        &self,
        style: StyleId,
        center: Vec2D,
        radius: i32,
        safe: f64,
    ) -> bool {
        let nearby = self.closest_line_to(center);
        if self.lines[nearby].style.id == style {
            return self.is_circle_inside_border_with_matching_style(nearby, center, radius, safe);
        }
        false
    }

    /// `IsCircleInsideBorderWithMatchingStyle` (`:85-103`) тАФ the 12 test
    /// points on the `ExtraBuffer`-inflated circle (`:193-198`:
    /// `1/cos(╧А/6)`), each walked from the nearby line.
    pub fn is_circle_inside_border_with_matching_style(
        &self,
        nearby: usize,
        center: Vec2D,
        radius: i32,
        safe: f64,
    ) -> bool {
        let num = radius as f64 * (1.0 / (std::f64::consts::PI / 6.0).cos());
        let style = self.lines[nearby].style.id;
        for i in 0..12 {
            // `CircleTestPoints[i]` тАФ UnitX rotated by 2╧А┬╖i/12 (`:195-196`).
            let point = Vec2D::UNIT_X.rotated_by(std::f64::consts::PI * 2.0 * i as f64 / 12.0, Vec2D::ZERO);
            let p = center + point * num;
            let Some(idx) = self.line_containing(p, Some(nearby), 0) else {
                return false;
            };
            if self.lines[idx].style.id != style {
                return false;
            }
            let (px, py) = p.to_point();
            if !self.lines[idx].is_inside_border(px, py, idx > 0, idx + 1 < self.lines.len(), safe) {
                return false;
            }
        }
        true
    }

    /// `GetRoomPositionInsideBorder` (`:105-139`) тАФ the four shrinking
    /// attempts (├Ч0.8), the Top/Center/Bottom classification against the
    /// two extreme offsets, the centre fallback.
    pub fn room_position_inside_border(
        &self,
        line_idx: usize,
        along: f64,
        mut from: f64,
        room_radius: i32,
        safe: f64,
    ) -> (Vec2D, SnakeOrientation) {
        let line = &self.lines[line_idx];
        let target = line.start.lerp(line.end, along);
        let p1 = line.potential_room_position(along, -1.0, room_radius, safe);
        let p2 = line.potential_room_position(along, 1.0, room_radius, safe);
        let target2 = if p1.y < p2.y { p1 } else { p2 };
        let target3 = if p1.y > p2.y { p1 } else { p2 };
        for _ in 0..4 {
            let p3 = line.potential_room_position(along, from, room_radius, safe);
            if self.is_circle_inside_border_with_matching_style(line_idx, p3, room_radius, safe) {
                let num = (p3 - target).length();
                let num2 = (p3 - target2).length();
                let num3 = (p3 - target3).length();
                let orientation = if num < num2 && num < num3 {
                    SnakeOrientation::Center
                } else if num2 < num3 {
                    SnakeOrientation::Top
                } else {
                    SnakeOrientation::Bottom
                };
                return (p3, orientation);
            }
            from *= 0.8;
        }
        (line.center(), SnakeOrientation::Center)
    }

    /// `SetTangents` (`:141-152`) тАФ the averaged joint tangents; the last
    /// line's end tangent is `-direction`.
    pub fn set_tangents(&mut self) {
        let n = self.lines.len();
        self.lines[0].start_tangent = self.lines[0].direction;
        for i in 0..n - 1 {
            let joint = (self.lines[i].direction + self.lines[i + 1].direction)
                .safe_normalize(Vec2D::ZERO);
            self.lines[i + 1].start_tangent = joint;
            self.lines[i].end_tangent = -joint;
        }
        self.lines[n - 1].end_tangent = -self.lines[n - 1].direction;
    }

    /// `AdjustTangentsToPreventSelfIntersection` (`:154-171`) тАФ up to 100
    /// sweeps until no line changes; the neighbour writes (`Prev.End =
    /// тИТStart`, `Next.Start = тИТEnd`, `DungeonControlLine.cs:257-266`)
    /// apply only when the link exists, the local tangent only then too.
    pub fn adjust_tangents_to_prevent_self_intersection(&mut self) {
        for _ in 0..100 {
            let mut changed = false;
            for i in 0..self.lines.len() {
                let (did, st, et) = self.lines[i].adjust_tangents();
                if !did {
                    continue;
                }
                changed = true;
                if i > 0 {
                    self.lines[i].start_tangent = st;
                    self.lines[i - 1].end_tangent = -st;
                }
                if i + 1 < self.lines.len() {
                    self.lines[i].end_tangent = et;
                    self.lines[i + 1].start_tangent = -et;
                }
            }
            if !changed {
                break;
            }
        }
    }

    /// `IsLineInsideBorder(from, to, halfWidth)` (`:173-181`) тАФ the two
    /// offset rails of the half-width band.
    pub fn is_line_inside_border_width(
        &self,
        from: Vec2D,
        to: Vec2D,
        half_width: i32,
        safe: f64,
    ) -> bool {
        let offset = (to - from)
            .safe_normalize(Vec2D::UNIT_X)
            .rotated_by(std::f64::consts::PI / 2.0, Vec2D::ZERO)
            * half_width as f64;
        if self.is_line_inside_border(from + offset, to + offset, safe) {
            return self.is_line_inside_border(from - offset, to - offset, safe);
        }
        false
    }

    /// `IsLineInsideBorder(from, to)` (`:183-191`) тАФ the Bresenham walk
    /// (`Utils.PlotLine`, `jump: true`) carrying the containing-line guess
    /// forward; any failed cell aborts. The port's shared `plot_line`
    /// (`desert_entrances.rs`) plots without a return value, so this keeps
    /// the same walk with the bool short-circuit inline (`Utils.cs:2238-
    /// 2292`).
    pub fn is_line_inside_border(&self, from: Vec2D, to: Vec2D, safe: f64) -> bool {
        let (x0, y0) = from.to_point();
        let (x1, y1) = to.to_point();
        let mut line = Some(self.closest_line_to(from));
        let mut ok = true;
        let mut plot = |x: i32, y: i32| -> bool {
            line = self.line_containing(Vec2D::new(x as f64, y as f64), line, 0);
            match line {
                Some(idx) => self.lines[idx].is_inside_border(
                    x,
                    y,
                    idx > 0,
                    idx + 1 < self.lines.len(),
                    safe,
                ),
                None => false,
            }
        };
        if x0 == x1 && y0 == y1 {
            return plot(x0, y0);
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
            ok = if steep { plot(num4, i) } else { plot(i, num4) };
            if !ok {
                return false;
            }
            num3 -= num2;
            if num3 >= 0 {
                i += num5;
                continue;
            }
            num4 += num6;
            num3 += num;
            i += num5;
        }
        true
    }
}

/// `DitherSnakePass.CalculateDungeonDitherSnake` (`DitherSnakePass.cs:44-133`)
/// тАФ the serpentine control-point walk: `num2 = (int)(20┬╖maxTilesX/4200)`
/// columns per style band, the row direction flipping per band and with
/// the dungeon side, the two radius random-walks (`NextDouble` lerp within
/// ┬▒num13 of the running radii), the row-0 dungeon-location clamp, the
/// turnaround arcs (`num9 = 0.05` steps to < 0.5 тАФ ten f64 accumulations:
/// 0.05, 0.1, 0.15000000000000002, тАж 0.49999999999999994 < 0.5, the tenth
/// 0.5499тАж stops; the C# loop draws the same ten), the
/// `1 тИТ num10` x-squash), then SetTangents + the ┬▒UnitX end pins + the
/// self-intersection sweeps. The `progress.Set` calls are the boundary
/// feed's business (no draw). `bounds` = (X, Y, Width, Height) of the
/// outer potential dungeon bounds.
#[allow(clippy::too_many_arguments)]
fn extra_buffer() -> f64 {
    1.0 / (std::f64::consts::PI / 6.0).cos()
}

fn circle_test_points() -> [Vec2; 12] {
    let mut pts = [Vec2::ZERO; 12];
    for (i, p) in pts.iter_mut().enumerate() {
        *p = Vec2::UNIT_X.rotated_by(std::f64::consts::TAU * i as f64 / 12.0, Vec2::ZERO);
    }
    pts
}

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

pub(crate) fn calculate_dungeon_dither_snake(
    rng: &mut Rng,
    max_tiles_x: i32,
    styles: &[SnakeStyle],
    bounds: (i32, i32, i32, i32),
    dungeon_side_left: bool,
    dungeon_location: i32,
    safe: f64,
) -> DitherSnake {
    let mut snake = DitherSnake::default();
    let (bx, by, bw, bh) = bounds;
    let num = max_tiles_x as f64 / 4200.0;
    let num2 = (20.0 * num) as i32;
    let count = styles.len() as i32;
    let num3 = 1.0 / (num2 - 1) as f64;
    let num4 = 1.0 / (count - 1) as f64;
    let num5 = bh as f64 / count as f64;
    let num6 = 1.0 - safe;
    let num7 = num5 / 2.0 * (1.0 + num6 / 2.0) - 1.0;
    let num8 = num7 - 0.1 * num * num5;
    let num9 = 0.05;
    let num10 = remap(num, 1.0, 2.0, 1.0, 1.5);
    let num11 = num7 + num5 / 2.0 * num10;
    let num12 = num7;
    let origin = Vec2D::new(bx as f64 + num11, by as f64 + num12);
    let extent = Vec2D::new(bw as f64 - num11 * 2.0, bh as f64 - num12 * 2.0);
    let num13 = (num7 - num8).min(extent.x * num3 / 2.0);
    // `_ = dungeonGenerationStyles[0]` (`:66`) тАФ a read, no effect.
    let mut start_radius = num7;
    let mut start = Vec2D::ZERO;
    let num14 = num2 * count;
    let num15 = num14 / styles.len() as i32;
    let mut num16 = num7;
    let mut num17 = num7;
    let mut i = 0;
    while i < num14 {
        let num18 = i % num2;
        let num19 = i / num2;
        let mut num20 = num3 * num18 as f64;
        let y = num4 * num19 as f64;
        let mut num21 = if dungeon_side_left { 1 } else { -1 };
        if num19 % 2 == 1 {
            num21 *= -1;
        }
        if num21 < 0 {
            num20 = 1.0 - num20;
        }
        let mut point = origin + extent * Vec2D::new(num20, y);
        if i == 0 {
            start = point;
            i += 1;
            continue;
        }
        if num19 == 0 && (point.x - dungeon_location as f64) * (num21 as f64) < 0.0 {
            start = Vec2D::new(dungeon_location as f64, point.y);
            i += 1;
            continue;
        }
        num16 = lerp(num8.max(num16 - num13), num7.min(num16 + num13), rng.next_f64());
        num17 = lerp(num8.max(num17 - num13), num7.min(num17 + num13), rng.next_f64());
        let num22 = (num16 + num17) / 2.0;
        point.y += (num16 - num17) / 2.0;
        let num23 = i / num15;
        let style = styles[num23 as usize];
        snake.add(ControlLine::new(start, point, start_radius, num22, num23, style));
        start = point;
        start_radius = num22;
        if num18 == num2 - 1 && num19 != count - 1 {
            // The turnaround arc (`:109-125`): rotate the row-end point
            // about the half-row-down centre, squash x toward the centre,
            // one CurveLine per 0.05 step; the arc consumes the next
            // column's iteration (`i++` at `:124`).
            let anchor = point;
            let center = anchor.lerp(anchor + extent * Vec2D::new(0.0, num4), 0.5);
            let mut num24 = num9;
            while num24 < 0.5 {
                point = anchor.rotated_by(std::f64::consts::PI * 2.0 * num24 * num21 as f64, center);
                point.x = lerp(point.x, center.x, 1.0 - num10);
                let mut line = ControlLine::new(start, point, start_radius, num22, num23, style);
                line.curve_line = true;
                snake.add(line);
                start = point;
                start_radius = num22;
                num24 += num9;
            }
            i += 1;
        }
        i += 1;
    }
    snake.set_tangents();
    let num25 = if dungeon_side_left { 1.0 } else { -1.0 };
    let last = snake.lines.len() - 1;
    snake.lines[0].start_tangent = Vec2D::UNIT_X * num25;
    snake.lines[last].end_tangent = Vec2D::UNIT_X * num25;
    snake.adjust_tangents_to_prevent_self_intersection();
    snake
}

/// `DitherSnakePass.GenerateDungeonDitherSnake` (`DitherSnakePass.cs:33-42`)
/// тАФ paint every line over the outer bounds' hitbox.
fn generate_dungeon_dither_snake(
    world: &mut World,
    rng: &mut Rng,
    snake: &DitherSnake,
    bounds: Option<(i32, i32, i32, i32)>,
    safe: f64,
) {
    let len = snake.lines.len();
    for (i, line) in snake.lines.iter().enumerate() {
        let prev = if i > 0 { Some(snake.lines[i - 1].style) } else { None };
        let next = if i + 1 < len { Some(snake.lines[i + 1].style) } else { None };
        line.paint_rect(world, rng, i, len, bounds, prev, next, safe);
    }
}

/// The Reset dual block's dither constant (`WorldGen.cs:11465` тЖТ
/// `GenVars.cs:305-315` тЖТ `DungeonControlLine.NormalizedDistanceSafeFromDither`):
/// `1 тИТ 0.075┬╖maxTilesY/1200`.
pub(crate) fn normalized_distance_safe_from_dither(max_tiles_y: i32) -> f64 {
    1.0 - 0.075 * (max_tiles_y as f64 / 1200.0)
}

/// The Reset dungeon-location draw (`WorldGen.cs:11441-11447` /
/// `:11454-11461`): left = `Next(leftBeachEnd + 50, (int)(maxTilesX┬╖0.2))`,
/// right = `Next((int)(maxTilesX┬╖0.8), rightBeachStart тИТ 50)`
/// (`dungeonBeachPadding` 50, `GenVars.cs:452`).
pub(crate) fn roll_dungeon_location(
    rng: &mut Rng,
    max_tiles_x: i32,
    left_beach_end: i32,
    right_beach_start: i32,
    side_left: bool,
) -> i32 {
    if side_left {
        rng.range(left_beach_end + 50, (max_tiles_x as f64 * 0.2) as i32)
    } else {
        rng.range((max_tiles_x as f64 * 0.8) as i32, right_beach_start - 50)
    }
}

/// `DungeonUtils.CreatePotentialDungeonBounds` (`DungeonUtils.cs:2067-2087`)
/// with the crawler's call args (`DungeonCrawler.cs:109`: percentInMiddle
/// 0.1f, percentOnEdges 0.05f) тАФ the outer bounds only (the inner bounds
/// have no snake consumer). `percentOnTop` defaults to
/// `(worldSurface + 10)/maxTilesY` (no `DungeonEntranceIsUnderground` seed
/// in the port), `percentOnBottom` to `(UnderworldLayer тИТ 10)/maxTilesY`
/// (`Main.cs:2790`: maxTilesY тИТ 200). Returns the bounds as the C#
/// `DungeonBounds` exposes them тАФ (X, Y, Width, Height) with
/// Width = Right тИТ Left, Height = Bottom тИТ Top (`DungeonBounds.cs:33-39`) тАФ
/// plus the hitbox the `CalculateHitbox` degenerate-expand produces
/// (`DungeonBounds.cs:310-322`), as inclusive (left, top, right, bottom).
pub(crate) fn create_potential_dungeon_bounds(
    max_tiles_x: i32,
    max_tiles_y: i32,
    world_surface: f64,
    left_dungeon: bool,
) -> ((i32, i32, i32, i32), Option<(i32, i32, i32, i32)>) {
    let percent_in_middle = 0.10000000149011612f64; // the C# float literal
    let percent_on_edges = 0.05000000074505806f64;
    let percent_on_top = (world_surface + 10.0) / max_tiles_y as f64;
    let percent_on_bottom = (max_tiles_y as f64 - 200.0 - 10.0) / max_tiles_y as f64;
    let num = percent_in_middle / 2.0;
    let num2 = if left_dungeon {
        (max_tiles_x as f64 * percent_on_edges) as i32
    } else {
        (max_tiles_x as f64 * (0.5 + num)) as i32
    };
    let num3 = if left_dungeon {
        (max_tiles_x as f64 * (0.5 - num)) as i32
    } else {
        max_tiles_x - (max_tiles_x as f64 * percent_on_edges) as i32
    };
    let num4 = (max_tiles_y as f64 * percent_on_top) as i32;
    let num5 = (max_tiles_y as f64 * percent_on_bottom) as i32;
    // The clamped setters (`DungeonBounds.cs:53-99` тАФ MathHelper.Clamp then
    // the (int) truncation; the values here are already integral).
    let clamp_x = |v: i32| v.clamp(10, max_tiles_x - 10);
    let clamp_y = |v: i32| v.clamp(10, max_tiles_y - 10);
    let (mut l, mut r) = (clamp_x(num2), clamp_x(num3));
    let (mut t, mut b) = (clamp_y(num4), clamp_y(num5));
    // `CalculateHitbox` (`:310-322`): the degenerate expand mutates the
    // stored bounds; the hitbox is (X, Y, Width, Height) with the exclusive
    // Right/Bottom the paint loops walk inclusively.
    if r <= l {
        r = l + 1;
    }
    if b <= t {
        b = t + 1;
    }
    // The stored bounds' X/Y/Width/Height accessors (`:33-39`) feed
    // `CalculateDungeonDitherSnake`; the hitbox rectangle's exclusive
    // Right/Bottom become the inclusive loop ends the paint walk uses.
    ((l, t, r - l, b - t), Some((l, t, r - 1, b - 1)))
}

/// `DitherSnakePass.ApplyPass` (`DitherSnakePass.cs:22-31`) тАФ dungeon 0
/// calculates + paints, then dungeon 1. The port's step adapter supplies
/// the per-dungeon (styles, bounds, side, location) tuples the Reset/Dunes
/// setup drew; the C#'s `progress.Set` fractions ride the boundary feed.
#[allow(clippy::too_many_arguments)]
pub(crate) fn apply_dither_snake_pass(
    world: &mut World,
    rng: &mut Rng,
    max_tiles_x: i32,
    dungeons: &[(Vec<SnakeStyle>, (i32, i32, i32, i32), Option<(i32, i32, i32, i32)>, bool, i32)],
    safe: f64,
) -> Vec<DitherSnake> {
    let mut snakes = Vec::new();
    for (styles, bounds, hitbox, side_left, location) in dungeons {
        let snake = calculate_dungeon_dither_snake(
            rng,
            max_tiles_x,
            styles,
            *bounds,
            *side_left,
            *location,
            safe,
        );
        generate_dungeon_dither_snake(world, rng, &snake, *hitbox, safe);
        snakes.push(snake);
    }
    snakes
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The Bayer matrix is the C# row-major literal, verbatim
    /// (`DitherSnakePass.cs:137-143`).
    #[test]
    fn the_bayer_matrix_matches_the_csharp_literal() {
        assert_eq!(BAYER_DITHER[0], [0.0, 0.5, 0.125, 0.625]);
        assert_eq!(BAYER_DITHER[1], [0.75, 0.25, 0.875, 0.375]);
        assert_eq!(BAYER_DITHER[2], [0.1875, 0.6875, 0.0625, 0.5625]);
        assert_eq!(BAYER_DITHER[3], [0.9375, 0.4375, 0.8125, 0.3125]);
    }

    /// The style quintuples against `DungeonGenerationStyles.cs`'s statics
    /// (the anchors on each arm) тАФ the fields `Paint`/`SkipPaintForEdge` read.
    #[test]
    fn the_style_table_matches_the_csharp_statics() {
        let s = SnakeStyle::of(StyleId::Cavern);
        assert_eq!((s.brick_tile, s.brick_wall, s.tier, s.edge_dither), (38, 349, 0, true));
        let s = SnakeStyle::of(StyleId::Corruption);
        assert_eq!((s.brick_tile, s.brick_wall, s.tier, s.edge_dither), (25, 3, 1, true));
        let s = SnakeStyle::of(StyleId::Crimson);
        assert_eq!((s.brick_tile, s.brick_wall, s.tier, s.edge_dither), (203, 83, 1, true));
        let s = SnakeStyle::of(StyleId::Jungle);
        assert_eq!((s.brick_tile, s.brick_wall, s.tier, s.edge_dither), (59, 64, 2, true));
        let s = SnakeStyle::of(StyleId::Snow);
        assert_eq!((s.brick_tile, s.brick_wall, s.tier, s.edge_dither), (161, 71, 0, true));
        let s = SnakeStyle::of(StyleId::Desert);
        assert_eq!((s.brick_tile, s.brick_wall, s.tier, s.edge_dither), (396, 187, 0, false));
        let s = SnakeStyle::of(StyleId::Hallow);
        assert_eq!((s.brick_tile, s.brick_wall, s.tier, s.edge_dither), (117, 28, 4, true));
        // TempleStyleData sets no tier тАФ the тИТ1 field default stands
        // (`DungeonGenerationStyleData.cs:13`).
        let s = SnakeStyle::of(StyleId::Temple);
        assert_eq!((s.brick_tile, s.brick_wall, s.tier, s.edge_dither), (226, 87, -1, false));
        let s = SnakeStyle::of(StyleId::Dungeon0(DungeonColor::Green));
        assert_eq!((s.brick_tile, s.brick_wall, s.tier, s.edge_dither), (43, 8, 3, false));
    }

    /// The dual lists (`DungeonCrawler.cs:111-127`): dungeon 0 walks
    /// Cavern тЖТ evil тЖТ Jungle тЖТ colour; dungeon 1 Snow тЖТ Desert тЖТ Hallow тЖТ
    /// Temple; the evil arm follows `WorldGen.crimson`.
    #[test]
    fn the_dual_style_lists_follow_the_crawler() {
        let d0 = dual_style_list(0, false, DungeonColor::Blue);
        let ids: Vec<StyleId> = d0.iter().map(|s| s.id).collect();
        assert_eq!(
            ids,
            vec![StyleId::Cavern, StyleId::Corruption, StyleId::Jungle, StyleId::Dungeon0(DungeonColor::Blue)]
        );
        let d0c = dual_style_list(0, true, DungeonColor::Pink);
        assert_eq!(d0c[1].id, StyleId::Crimson);
        assert_eq!(d0c[3].id, StyleId::Dungeon0(DungeonColor::Pink));
        let d1 = dual_style_list(1, false, DungeonColor::Blue);
        let ids: Vec<StyleId> = d1.iter().map(|s| s.id).collect();
        assert_eq!(ids, vec![StyleId::Snow, StyleId::Desert, StyleId::Hallow, StyleId::Temple]);
    }

    /// `ApplyPass`'s snake store (`DitherSnakePass.cs:26/:29`): one snake
    /// per dungeon, dungeon 0 first тАФ the a == b determinism over the
    /// stored control chains.
    #[test]
    fn the_pass_stores_one_snake_per_dungeon() {
        let dungeons: Vec<_> = (0..2)
            .map(|d| {
                let styles = dual_style_list(d, false, DungeonColor::Blue);
                let side_left = d == 1;
                let (bounds, hitbox) =
                    create_potential_dungeon_bounds(4200, 1200, 300.0, side_left);
                (styles, bounds, hitbox, side_left, if side_left { 500 } else { 3600 })
            })
            .collect();
        let run = || {
            let mut world = World::new("t".into(), 4200, 1200);
            apply_dither_snake_pass(&mut world, &mut Rng(0x5EED), 4200, &dungeons, 0.925)
        };
        let a = run();
        let b = run();
        assert_eq!(a.len(), 2);
        for (sa, sb) in a.iter().zip(&b) {
            assert_eq!(sa.lines.len(), sb.lines.len());
            for (la, lb) in sa.lines.iter().zip(&sb.lines) {
                assert_eq!(la.start, lb.start);
                assert_eq!(la.end, lb.end);
                assert_eq!(la.style.id, lb.style.id);
            }
        }
        // Dungeon 0's chain carries the Cavern/evil/Jungle/colour ids,
        // dungeon 1's the Snow/Desert/Hallow/Temple ids.
        assert!(a[0].lines.iter().any(|l| l.style.id == StyleId::Cavern));
        assert!(a[1].lines.iter().any(|l| l.style.id == StyleId::Temple));
    }

    /// `NormalizedDistanceSafeFromDither` (`WorldGen.cs:11465`):
    /// `1 тИТ 0.075┬╖maxTilesY/1200` тАФ 0.925 on the default 1200-high world.
    #[test]
    fn the_safe_constant_matches_the_reset_draw() {
        assert_eq!(normalized_distance_safe_from_dither(1200), 1.0 - 0.075);
        assert_eq!(normalized_distance_safe_from_dither(2400), 1.0 - 0.15);
    }

    /// The bounds math (`DungeonUtils.cs:2067-2087` + the clamped setters):
    /// on a 4200├Ч1200 world with worldSurface 300, the left dungeon spans
    /// x 210..1889 тАФ the C# float literals matter: `(0.5 тИТ 0.05f)` is
    /// 0.44999999925494194, ├Ч4200 = 1889.9999968тАж тЖТ (int) 1889 тАФ y 310..990
    /// ((300+10)/1200┬╖1200, (1200тИТ210)/1200┬╖1200).
    #[test]
    fn the_potential_bounds_match_the_csharp_clamps() {
        let ((x, y, w, h), hitbox) = create_potential_dungeon_bounds(4200, 1200, 300.0, true);
        assert_eq!((x, y, w, h), (210, 310, 1679, 680));
        assert_eq!(hitbox, Some((210, 310, 1888, 989)));
        let ((x, y, w, h), _) = create_potential_dungeon_bounds(4200, 1200, 300.0, false);
        // Right: (0.5+0.05f)┬╖4200 = 2310.0000031тАж тЖТ 2310 .. 4200 тИТ 210 = 3990.
        assert_eq!((x, y, w, h), (2310, 310, 1680, 680));
    }

    /// `CanPaint`'s interior algebra on an axis-aligned line
    /// (`DungeonControlLine.cs:78-115`): start (0,0), end (10,0), tangents
    /// ┬▒UnitX тАФ a point mid-line at distance 3 has progress 0.5 and
    /// distance 3; before the start (no prev) it clamps to progress 0.
    #[test]
    fn can_paint_matches_the_line_algebra() {
        let style = SnakeStyle::of(StyleId::Cavern);
        let mut line = ControlLine::new(
            Vec2D::new(0.0, 0.0),
            Vec2D::new(10.0, 0.0),
            10.0,
            10.0,
            0,
            style,
        );
        line.start_tangent = Vec2D::UNIT_X;
        line.end_tangent = Vec2D::new(-1.0, 0.0);
        let (d, p) = line.can_paint(5, 3, true, true).unwrap();
        assert_eq!(p, 0.5);
        assert_eq!(d, 3.0);
        // Before the start with a prev тЖТ not ours.
        assert!(line.can_paint(-1, 0, true, true).is_none());
        // Without a prev тЖТ the endpoint clamp, distance = |value|.
        let (d, p) = line.can_paint(-3, 4, false, true).unwrap();
        assert_eq!((d, p), (5.0, 0.0));
    }

    /// The border query (`IsInsideBorder`, `:270-277`): distance <
    /// lerp(radius)┬╖safe тИТ 4. With radius 10 and safe 0.925 the border is
    /// 5.25 тАФ distance 3 inside, distance 6 outside.
    #[test]
    fn the_border_uses_the_safe_band_minus_four() {
        let style = SnakeStyle::of(StyleId::Cavern);
        let mut line = ControlLine::new(
            Vec2D::new(0.0, 0.0),
            Vec2D::new(10.0, 0.0),
            10.0,
            10.0,
            0,
            style,
        );
        line.start_tangent = Vec2D::UNIT_X;
        line.end_tangent = Vec2D::new(-1.0, 0.0);
        assert!(line.is_inside_border(5, 3, true, true, 0.925));
        assert!(!line.is_inside_border(5, 6, true, true, 0.925));
    }

    /// `SetTangents` (`DitherSnake.cs:141-152`): the joint tangent is the
    /// safe-normalized direction sum; the last end tangent is тИТdirection.
    #[test]
    fn set_tangents_averages_the_joints() {
        let style = SnakeStyle::of(StyleId::Cavern);
        let mut snake = DitherSnake::default();
        snake.add(ControlLine::new(Vec2D::ZERO, Vec2D::new(10.0, 0.0), 5.0, 5.0, 0, style));
        snake.add(ControlLine::new(Vec2D::new(10.0, 0.0), Vec2D::new(10.0, 10.0), 5.0, 5.0, 0, style));
        snake.set_tangents();
        assert_eq!(snake.lines[0].start_tangent, Vec2D::UNIT_X);
        // (UnitX + UnitY).SafeNormalize = (тИЪ2/2, тИЪ2/2).
        let j = snake.lines[0].end_tangent;
        assert!((j.x + std::f64::consts::FRAC_1_SQRT_2).abs() < 1e-15);
        assert!((j.y + std::f64::consts::FRAC_1_SQRT_2).abs() < 1e-15);
        assert_eq!(snake.lines[1].start_tangent, -j);
        assert_eq!(snake.lines[1].end_tangent, Vec2D::new(0.0, -1.0));
    }

    /// The builder's a == b determinism: the same seed and args produce the
    /// same control points, radii and tangents (`DitherSnakePass.cs:44-133`
    /// тАФ the two `NextDouble` walks and the arc chain).
    #[test]
    fn the_builder_is_deterministic_for_a_seed() {
        let styles = dual_style_list(0, false, DungeonColor::Blue);
        let bounds = (210, 310, 1680, 680);
        let build = || {
            calculate_dungeon_dither_snake(&mut Rng(0x0CEA), 4200, &styles, bounds, true, 500, 0.925)
        };
        let a = build();
        let b = build();
        assert_eq!(a.lines.len(), b.lines.len());
        for (la, lb) in a.lines.iter().zip(&b.lines) {
            assert_eq!(la.start, lb.start);
            assert_eq!(la.end, lb.end);
            assert_eq!(la.start_radius, lb.start_radius);
            assert_eq!(la.end_radius, lb.end_radius);
            assert_eq!(la.start_tangent, lb.start_tangent);
            assert_eq!(la.end_tangent, lb.end_tangent);
            assert_eq!(la.progression_stage, lb.progression_stage);
            assert_eq!(la.curve_line, lb.curve_line);
        }
        // 20 columns ├Ч 4 bands = 80 iterations, minus the row-0 clamp skips
        // and plus 3 turnaround arcs ├Ч 10 segments тИТ 3 consumed iterations.
        // The exact count is a function of the location clamp; assert the
        // shape invariants instead: тЙе 4 lines, one per band minimum, the
        // end pins ┬▒UnitX (side Left тЖТ +1).
        assert!(a.lines.len() >= 4);
        assert_eq!(a.lines[0].start_tangent, Vec2D::UNIT_X);
        assert_eq!(a.lines.last().unwrap().end_tangent, Vec2D::UNIT_X);
        // The stages walk 0..3 in order (never decreasing).
        let stages: Vec<i32> = a.lines.iter().map(|l| l.progression_stage).collect();
        assert!(stages.windows(2).all(|w| w[0] <= w[1]));
        assert_eq!(*stages.first().unwrap(), 0);
        assert_eq!(*stages.last().unwrap(), 3);
    }

    /// The paint driver writes the style bricks + walls and the 350 band
    /// (`DungeonControlLine.Paint`, `:117-155`), deterministically (a == b
    /// over the tile grid).
    #[test]
    fn the_paint_driver_writes_the_band_deterministically() {
        let styles = dual_style_list(0, false, DungeonColor::Blue);
        let bounds = (100, 100, 400, 400);
        let hitbox = Some((100, 100, 499, 499));
        let paint = || {
            let mut world = World::new("t".into(), 600, 600);
            let mut rng = Rng(0xBEEF);
            let snake = calculate_dungeon_dither_snake(
                &mut rng, 4200, &styles, bounds, true, 200, 0.925,
            );
            generate_dungeon_dither_snake(&mut world, &mut rng, &snake, hitbox, 0.925);
            world
        };
        let a = paint();
        let b = paint();
        assert_eq!(a.tiles, b.tiles, "a == b over the painted grid");
        let band = a.tiles.iter().filter(|t| t.wall == 350).count();
        assert!(band > 0, "the unbreakable band paints ({band})");
        // A 350-band cell carries the tier byte in the wall paint
        // (`wallColor((byte)num5)`) тАФ JungleBoss 2 (+16 on the inner 2px).
        let painted = a
            .tiles
            .iter()
            .enumerate()
            .filter(|(_, t)| t.wall == 350)
            .map(|(i, _)| a.extras_at(i) & 0x1f)
            .collect::<Vec<_>>();
        assert!(painted.iter().all(|&p| p == 1 || p == 2 || p == 3 || p == 17 || p == 18 || p == 19));
        // The bricks are active with the style wall pair.
        let bricks = a.tiles.iter().filter(|t| t.active() && t.kind == 38).count();
        assert!(bricks > 0, "the Cavern band paints ({bricks})");
    }

    /// The location rolls (`WorldGen.cs:11443/:11447`): left draws inside
    /// [leftBeachEnd+50, 0.2┬╖width), right inside [0.8┬╖width,
    /// rightBeachStartтИТ50).
    #[test]
    fn the_location_rolls_stay_in_their_bands() {
        for seed in [1u64, 7, 42, 0x0CEA] {
            let mut rng = Rng(seed);
            let left = roll_dungeon_location(&mut rng, 4200, 380, 3800, true);
            assert!((430..840).contains(&left), "left {left}");
            let right = roll_dungeon_location(&mut rng, 4200, 380, 3800, false);
            assert!((3360..3750).contains(&right), "right {right}");
        }
    }
}
