//! The UI scissor stack — the port of XNA's `GraphicsDevice.ScissorRectangle`
//! as `UIElement.Draw` drives it (`Terraria/UI/UIElement.cs:222-237`).
//!
//! The C# swaps the device's scissor rect for the duration of a
//! `DrawChildren` call and restores the previous one afterwards:
//!
//! ```text
//! if (overflowHidden) {
//!     Rectangle clippingRectangle = this.GetClippingRectangle(spriteBatch);
//!     spriteBatch.GraphicsDevice.ScissorRectangle = clippingRectangle;   // :225-227
//!     ...
//! }
//! this.DrawChildren(spriteBatch);                                        // :230
//! if (overflowHidden) {
//!     spriteBatch.GraphicsDevice.ScissorRectangle = scissorRectangle;     // :233-235
//!     ...
//! }
//! ```
//!
//! and `GetClippingRectangle` intersects its own rect with the *current*
//! rectangle before setting it (`:265-268`), so nested `OverflowHidden`
//! elements clip to the intersection — exactly [`ClipStack::push`].
//!
//! The port's renderer batches every screen-space sprite into one instanced
//! draw call, so the scissor cannot be a device state: the current rect is
//! stamped into each `SpriteInstance` (`clip`) and the fragment shader discards
//! the fragments outside it. A pixel mask at the fragment stage is the faithful
//! equivalent of XNA's scissor test (unlike shrinking the quad geometry, which
//! would move the half-texel sampling inset and shift edge pixels).

/// A screen-space rect `[x, y, w, h]`, in framebuffer pixels (the UI camera is
/// identity at zoom 1, so `ui::in_rect`'s space is this one).
pub type Rect = [f32; 4];

/// The unclipped rect: every fragment passes. `f32::MAX` rather than the screen
/// size so no caller has to know the surface dimensions.
pub const NO_CLIP: Rect = [
    f32::MIN / 2.0,
    f32::MIN / 2.0,
    f32::MAX,
    f32::MAX,
];

/// The intersection of two `[x, y, w, h]` rects (`GetClippingRectangle`'s
/// `Utils.Clamp` chain, `:265-268`). Touching or disjoint rects give a zero-size
/// rect, which clips everything — the C# can produce the same rectangle (the
/// clamp chain never inverts it).
pub fn intersect(a: Rect, b: Rect) -> Rect {
    let x0 = a[0].max(b[0]);
    let y0 = a[1].max(b[1]);
    let x1 = (a[0] + a[2]).min(b[0] + b[2]);
    let y1 = (a[1] + a[3]).min(b[1] + b[3]);
    [x0, y0, (x1 - x0).max(0.0), (y1 - y0).max(0.0)]
}

/// `GraphicsDevice.ScissorRectangle` as a stack: [`push`](Self::push) replaces
/// the current rect with its intersection (the `GetClippingRectangle` rule),
/// [`pop`](Self::pop) restores the enclosing one (`:233-235`).
#[derive(Clone, Debug)]
pub struct ClipStack {
    stack: Vec<Rect>,
    current: Rect,
}

impl Default for ClipStack {
    fn default() -> Self {
        Self::new()
    }
}

impl ClipStack {
    pub fn new() -> Self {
        Self {
            stack: Vec::new(),
            current: NO_CLIP,
        }
    }

    /// The rect every sprite pushed right now is clipped to.
    pub fn current(&self) -> Rect {
        self.current
    }

    /// `GraphicsDevice.ScissorRectangle = GetClippingRectangle(...)` — the new
    /// rect is the intersection with the enclosing one (`:265-268`).
    pub fn push(&mut self, rect: Rect) {
        self.stack.push(self.current);
        self.current = intersect(self.current, rect);
    }

    /// `GraphicsDevice.ScissorRectangle = scissorRectangle` (`:234`) — restores
    /// the rect saved by the matching [`push`](Self::push). A `pop` without a
    /// `push` is a no-op (the C# always pairs them).
    pub fn pop(&mut self) {
        if let Some(prev) = self.stack.pop() {
            self.current = prev;
        }
    }

    /// How deep the stack is — the number of open `OverflowHidden` elements.
    pub fn depth(&self) -> usize {
        self.stack.len()
    }

    /// Drops every push (a new frame starts unclipped).
    pub fn clear(&mut self) {
        self.stack.clear();
        self.current = NO_CLIP;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_empty_stack_never_clips() {
        let stack = ClipStack::new();
        assert_eq!(stack.current(), NO_CLIP);
        assert_eq!(stack.depth(), 0);
        // Any realistic rect survives intersection with the unclipped sentinel
        // unchanged — a 4096x4096 window's UI sprites all sit well inside it.
        assert_eq!(intersect(NO_CLIP, [100.0, 200.0, 300.0, 150.0]), [100.0, 200.0, 300.0, 150.0]);
        assert_eq!(intersect(NO_CLIP, [-500.0, -20.0, 4096.0, 2160.0]), [-500.0, -20.0, 4096.0, 2160.0]);
    }

    #[test]
    fn a_push_clips_until_the_matching_pop() {
        let mut stack = ClipStack::new();
        stack.push([100.0, 200.0, 300.0, 150.0]);
        assert_eq!(stack.current(), [100.0, 200.0, 300.0, 150.0]);
        assert_eq!(stack.depth(), 1);
        stack.pop();
        assert_eq!(stack.current(), NO_CLIP);
        assert_eq!(stack.depth(), 0);
    }

    #[test]
    fn a_nested_push_intersects_with_the_enclosing_rect() {
        // `GetClippingRectangle` intersects with the current device rect
        // (`UIElement.cs:265-268`), so a child can only ever shrink the visible
        // area — never widen it.
        let mut stack = ClipStack::new();
        stack.push([100.0, 200.0, 300.0, 150.0]); // 100..400 x 200..350
        stack.push([0.0, 0.0, 200.0, 500.0]); // 0..200 x 0..500
        assert_eq!(stack.current(), [100.0, 200.0, 100.0, 150.0]);
        stack.pop();
        assert_eq!(stack.current(), [100.0, 200.0, 300.0, 150.0]);
    }

    #[test]
    fn a_disjoint_nested_push_clips_everything() {
        let mut stack = ClipStack::new();
        stack.push([0.0, 0.0, 100.0, 100.0]);
        stack.push([500.0, 500.0, 100.0, 100.0]);
        assert_eq!(stack.current(), [500.0, 500.0, 0.0, 0.0]);
        stack.pop();
        assert_eq!(stack.current(), [0.0, 0.0, 100.0, 100.0]);
    }

    #[test]
    fn the_intersection_never_goes_negative() {
        // The C# clamp chain (`:261-268`) pins width/height at 0; an inverted
        // rect would make the shader's comparisons clip the wrong half.
        assert_eq!(intersect([0.0, 0.0, 10.0, 10.0], [20.0, 20.0, 5.0, 5.0]), [20.0, 20.0, 0.0, 0.0]);
    }

    #[test]
    fn clear_drops_every_open_push() {
        let mut stack = ClipStack::new();
        stack.push([0.0, 0.0, 10.0, 10.0]);
        stack.push([0.0, 0.0, 5.0, 5.0]);
        stack.clear();
        assert_eq!(stack.current(), NO_CLIP);
        assert_eq!(stack.depth(), 0);
    }

    #[test]
    fn the_fragment_test_keeps_the_left_and_top_edges() {
        // `sprite.wgsl`'s `fs_main` discards when
        // `p.x < clip.x || p.y < clip.y || p.x >= clip.x + clip.z || p.y >= clip.y + clip.w`,
        // which is XNA's scissor rule (a pixel is kept iff `left <= x < right`).
        // Mirroring the expression here pins the convention: an off-by-one on
        // either edge is a one-pixel seam on every clipped list.
        fn kept(clip: Rect, x: f32, y: f32) -> bool {
            !(x < clip[0] || y < clip[1] || x >= clip[0] + clip[2] || y >= clip[1] + clip[3])
        }
        let clip = [100.0, 200.0, 50.0, 20.0]; // 100..150 x 200..220
        assert!(kept(clip, 100.0, 200.0), "top-left corner is inside");
        assert!(kept(clip, 149.0, 219.0), "just inside the far corner");
        assert!(!kept(clip, 99.0, 210.0), "left is exclusive");
        assert!(!kept(clip, 110.0, 199.0), "top is exclusive");
        assert!(!kept(clip, 150.0, 210.0), "right is exclusive");
        assert!(!kept(clip, 110.0, 220.0), "bottom is exclusive");
        // A zero-size intersection (disjoint ancestors) keeps nothing.
        assert!(!kept([100.0, 200.0, 0.0, 0.0], 100.0, 200.0));
    }
}
