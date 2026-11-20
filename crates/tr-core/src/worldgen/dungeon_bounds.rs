//! `DungeonBounds` (`GameContent/Generation/Dungeon/DungeonBounds.cs:8-337`)
//! — the clamped, hitbox-cached rectangle the dual-dungeon dither snake
//! paints into and the potential-dungeon bounds queries read.
//!
//! W4-f24 lane B (the gated `Dual Dungeons Dither Snake` pass,
//! `WorldGen.cs:14755`). The C# clamps every bound setter to the world
//! margins (`Left`/`Right` to `[10, maxTilesX-10]`, `Top`/`Bottom` to
//! `[10, maxTilesY-10]`, `:53-99`) and caches a `Rectangle` hitbox on
//! `SetBounds`/`CalculateHitbox` (`:249-256/:310-322`); `Hitbox` returns
//! `Rectangle.Empty` until a hitbox exists (`:21-31`). The port carries the
//! world size on the struct (the C# reads the `Main.maxTilesX/Y` statics)
//! and keeps the `Option<Rect>` hitbox so the "no hitbox yet" arms
//! (`Contains`/`ContainsWithFluff`/`Intersects`, `:145-194`) stay faithful.
//!
//! Only the members the snake family + `CreatePotentialDungeonBounds` /
//! `InAnyPotentialDungeonBounds` call are ported; the line-intersection
//! helpers (`IntersectsWithLineThreePointCheck`, `:196-217`) and
//! `RandomPointInBounds` (`:103-106`) have no consumer in this lane and
//! stay out (cited skip).
//!
//! UNWIRED until B6: pure geometry, no RNG draw, no tile write — both
//! classic world-hash pins stay; the `a == b` halves are untouched.

use super::snake_math::clamp_i;

/// `Microsoft.Xna.Framework.Rectangle` — the integer hitbox the bounds
/// cache. XNA's `Right = X + Width`, `Bottom = Y + Height` (exclusive);
/// `Contains` is `Left <= x < Right && Top <= y < Bottom`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct Rect {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

impl Rect {
    pub const EMPTY: Rect = Rect { x: 0, y: 0, width: 0, height: 0 };

    pub fn new(x: i32, y: i32, width: i32, height: i32) -> Self {
        Rect { x, y, width, height }
    }

    pub fn left(&self) -> i32 {
        self.x
    }
    pub fn top(&self) -> i32 {
        self.y
    }
    pub fn right(&self) -> i32 {
        self.x + self.width
    }
    pub fn bottom(&self) -> i32 {
        self.y + self.height
    }

    /// `Rectangle.Contains(int, int)` — the half-open XNA test.
    pub fn contains(&self, x: i32, y: i32) -> bool {
        self.x <= x && x < self.right() && self.y <= y && y < self.bottom()
    }

    /// `Rectangle.Union` (`DungeonControlLine.Paint(Rectangle)` builds the
    /// per-line paint box from two 1×1 rects, `DungeonControlLine.cs:223`).
    pub fn union(a: Rect, b: Rect) -> Rect {
        let left = a.left().min(b.left());
        let top = a.top().min(b.top());
        let right = a.right().max(b.right());
        let bottom = a.bottom().max(b.bottom());
        Rect::new(left, top, right - left, bottom - top)
    }

    /// `Rectangle.Intersect` — the overlap, or `Empty` when disjoint.
    pub fn intersect(a: Rect, b: Rect) -> Rect {
        let left = a.left().max(b.left());
        let top = a.top().max(b.top());
        let right = a.right().min(b.right());
        let bottom = a.bottom().min(b.bottom());
        if right > left && bottom > top {
            Rect::new(left, top, right - left, bottom - top)
        } else {
            Rect::EMPTY
        }
    }

    /// `Rectangle.Inflate(int, int)` — grow by the amount on every side;
    /// XNA clamps a negative resulting width/height to 0.
    pub fn inflate(&self, h: i32, v: i32) -> Rect {
        Rect::new(
            self.x - h,
            self.y - v,
            (self.width + 2 * h).max(0),
            (self.height + 2 * v).max(0),
        )
    }
}

/// `DungeonBounds` (`DungeonBounds.cs:8-337`). `max_x`/`max_y` are the
/// `Main.maxTilesX/Y` the C# setters clamp against.
#[derive(Clone, Copy, Debug)]
pub(crate) struct DungeonBounds {
    max_x: i32,
    max_y: i32,
    bounds_left: i32,
    bounds_right: i32,
    bounds_top: i32,
    bounds_bottom: i32,
    hitbox: Option<Rect>,
}

impl DungeonBounds {
    /// `new DungeonBounds()` — the C# field initialisers are all zero and
    /// `_hitbox` is null (`DungeonBounds.cs:11-19`).
    pub fn new(max_x: i32, max_y: i32) -> Self {
        DungeonBounds {
            max_x,
            max_y,
            bounds_left: 0,
            bounds_right: 0,
            bounds_top: 0,
            bounds_bottom: 0,
            hitbox: None,
        }
    }

    /// `DungeonBounds.Hitbox` (`:21-31`) — `Rectangle.Empty` until set.
    pub fn hitbox(&self) -> Rect {
        self.hitbox.unwrap_or(Rect::EMPTY)
    }

    /// `DungeonBounds.HasHitbox()` (`:219-222`).
    pub fn has_hitbox(&self) -> bool {
        self.hitbox.is_some()
    }

    /// `X`/`Y`/`Width`/`Height` (`:33-39`).
    pub fn x(&self) -> i32 {
        self.bounds_left
    }
    pub fn y(&self) -> i32 {
        self.bounds_top
    }
    pub fn width(&self) -> i32 {
        self.bounds_right - self.bounds_left
    }
    pub fn height(&self) -> i32 {
        self.bounds_bottom - self.bounds_top
    }

    /// `Left`/`Right`/`Top`/`Bottom` getters (`:53-99`).
    pub fn left(&self) -> i32 {
        self.bounds_left
    }
    pub fn right(&self) -> i32 {
        self.bounds_right
    }
    pub fn top(&self) -> i32 {
        self.bounds_top
    }
    pub fn bottom(&self) -> i32 {
        self.bounds_bottom
    }

    /// `Left` setter (`:58-62`) — clamp to `[10, maxTilesX-10]`.
    pub fn set_left(&mut self, v: i32) {
        self.bounds_left = clamp_i(v, 10, self.max_x - 10);
    }
    /// `Right` setter (`:70-74`).
    pub fn set_right(&mut self, v: i32) {
        self.bounds_right = clamp_i(v, 10, self.max_x - 10);
    }
    /// `Top` setter (`:82-86`) — clamp to `[10, maxTilesY-10]`.
    pub fn set_top(&mut self, v: i32) {
        self.bounds_top = clamp_i(v, 10, self.max_y - 10);
    }
    /// `Bottom` setter (`:94-98`).
    pub fn set_bottom(&mut self, v: i32) {
        self.bounds_bottom = clamp_i(v, 10, self.max_y - 10);
    }

    /// `SetBounds(int, int, int, int)` (`:249-256`) — the C# sets Left,
    /// Right, Top, Bottom in that order, then `CalculateHitbox`.
    pub fn set_bounds(&mut self, min_x: i32, min_y: i32, max_x: i32, max_y: i32) {
        self.set_left(min_x);
        self.set_right(max_x);
        self.set_top(min_y);
        self.set_bottom(max_y);
        self.calculate_hitbox();
    }

    /// `CalculateHitbox()` (`:310-322`) — the degenerate-width/height
    /// bumps (`Right <= Left -> Left+1`, `Bottom <= Top -> Top+1`) run
    /// BEFORE the hitbox is built, and they mutate the stored bounds.
    pub fn calculate_hitbox(&mut self) -> Rect {
        if self.bounds_right <= self.bounds_left {
            self.bounds_right = self.bounds_left + 1;
        }
        if self.bounds_bottom <= self.bounds_top {
            self.bounds_bottom = self.bounds_top + 1;
        }
        let r = Rect::new(self.x(), self.y(), self.width(), self.height());
        self.hitbox = Some(r);
        r
    }

    /// `Inflate(int)` (`:108-111`) — `SetBounds(Left-a, Top-a, Right+a,
    /// Bottom+a)` (re-clamps + rebuilds the hitbox).
    pub fn inflate(&mut self, amount: i32) {
        let (l, t, r, b) = (self.left(), self.top(), self.right(), self.bottom());
        self.set_bounds(l - amount, t - amount, r + amount, b + amount);
    }

    /// `Contains(int, int)` (`:169-176`) — false until a hitbox exists.
    pub fn contains(&self, x: i32, y: i32) -> bool {
        match self.hitbox {
            Some(h) => h.contains(x, y),
            None => false,
        }
    }

    /// `ContainsWithFluff(int, int, int)` (`:145-157`) — fluff 0 delegates
    /// to `Contains`; otherwise the hitbox is inflated by the fluff.
    pub fn contains_with_fluff(&self, x: i32, y: i32, fluff: i32) -> bool {
        if fluff == 0 {
            return self.contains(x, y);
        }
        match self.hitbox {
            Some(h) => h.inflate(fluff, fluff).contains(x, y),
            None => false,
        }
    }

    /// `Reset()` (`:324-331`) — null the hitbox and zero the bounds
    /// (the setters are bypassed, so no clamping — verbatim).
    pub fn reset(&mut self) {
        self.hitbox = None;
        self.bounds_left = 0;
        self.bounds_right = 0;
        self.bounds_top = 0;
        self.bounds_bottom = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The bound setters clamp to the world margins and `CalculateHitbox`
    /// bumps a degenerate rect (`DungeonBounds.cs:53-99/:310-322`).
    #[test]
    fn w4_f24_bounds_clamp_and_hitbox_bump() {
        let mut b = DungeonBounds::new(4200, 1200);
        // Out-of-range setters clamp to [10, max-10].
        b.set_left(-50);
        b.set_right(99999);
        b.set_top(-5);
        b.set_bottom(99999);
        assert_eq!((b.left(), b.right(), b.top(), b.bottom()), (10, 4190, 10, 1190));
        // No hitbox until SetBounds/CalculateHitbox.
        assert!(!b.has_hitbox());
        assert_eq!(b.hitbox(), Rect::EMPTY);
        assert!(!b.contains(100, 100));
        b.set_bounds(100, 200, 300, 400);
        assert!(b.has_hitbox());
        assert_eq!(b.hitbox(), Rect::new(100, 200, 200, 200));
        assert!(b.contains(100, 200));
        assert!(!b.contains(300, 400)); // half-open right/bottom
        // Degenerate width/height bump to +1 (CalculateHitbox :312-319).
        let mut d = DungeonBounds::new(4200, 1200);
        d.set_bounds(100, 200, 100, 200);
        assert_eq!(d.right(), 101);
        assert_eq!(d.bottom(), 201);
        assert_eq!(d.hitbox(), Rect::new(100, 200, 1, 1));
    }

    /// `ContainsWithFluff` inflates the hitbox; fluff 0 delegates to
    /// `Contains` (`DungeonBounds.cs:145-157`).
    #[test]
    fn w4_f24_contains_with_fluff_inflates() {
        let mut b = DungeonBounds::new(4200, 1200);
        b.set_bounds(100, 100, 200, 200);
        assert!(b.contains_with_fluff(150, 150, 0));
        assert!(!b.contains_with_fluff(205, 150, 0));
        assert!(b.contains_with_fluff(205, 150, 10));
        assert!(!b.contains_with_fluff(215, 150, 10));
        // A bounds with no hitbox never contains, even with fluff.
        let empty = DungeonBounds::new(4200, 1200);
        assert!(!empty.contains_with_fluff(150, 150, 10));
    }

    /// `Rect.Union`/`Intersect`/`Inflate` — the paint-box maths
    /// (`DungeonControlLine.cs:223-225`).
    #[test]
    fn w4_f24_rect_union_intersect_inflate() {
        let a = Rect::new(0, 0, 1, 1);
        let b = Rect::new(10, 10, 1, 1);
        let u = Rect::union(a, b);
        assert_eq!(u, Rect::new(0, 0, 11, 11));
        let inflated = u.inflate(5, 5);
        assert_eq!(inflated, Rect::new(-5, -5, 21, 21));
        let clip = Rect::new(2, 2, 4, 4);
        assert_eq!(Rect::intersect(inflated, clip), clip);
        // Disjoint -> Empty.
        assert_eq!(Rect::intersect(a, b), Rect::EMPTY);
    }
}
