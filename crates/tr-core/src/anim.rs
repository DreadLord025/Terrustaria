//! Shared animation primitives — the small C# idioms every animated system in
//! Terraria rebuilds by hand. Keeping them in one place makes the frame counts
//! and thresholds auditable against the decompiled source.
//!
//! The simulation owns all animation state (the original advances
//! `frameCounter`/`itemAnimation` inside `Update`, never in draw code); the
//! renderer only reads the resulting frame indices.

/// The `frameCounter` idiom: accumulate `speed` every tick and emit one frame
/// step each time the accumulated value crosses `threshold`:
///
/// ```csharp
/// legFrameCounter += Math.Abs(velocity.X) * 1.3;
/// while (legFrameCounter > 8.0) { legFrameCounter -= 8.0; legFrame.Y += Height; }
/// ```
/// (`Player.cs:36259-36264` for walking, `Player.cs:36222-36228` for swimming
/// with speed `2.0`, and the same shape in `NPC.FindFrame`, `NPC.cs:66684`.)
///
/// The C# uses a `while`, so a fast entity can cross the threshold several
/// times in one tick; [`FrameCounter::advance`] returns that exact count.
#[derive(Clone, Debug)]
pub struct FrameCounter {
    value: f32,
    threshold: f32,
}

impl FrameCounter {
    pub fn new(threshold: f32) -> Self {
        debug_assert!(threshold > 0.0, "threshold must be positive");
        Self { value: 0.0, threshold }
    }

    /// Accumulate `speed` and return how many frame steps this tick crosses.
    pub fn advance(&mut self, speed: f32) -> u32 {
        self.value += speed;
        let mut steps = 0u32;
        while self.value > self.threshold {
            self.value -= self.threshold;
            steps += 1;
        }
        steps
    }

    /// The C# `frameCounter = 0.0` reset on idle / airborne / mount branches.
    pub fn reset(&mut self) {
        self.value = 0.0;
    }

    pub fn value(&self) -> f32 {
        self.value
    }
}

/// Wrap a frame row inside the C# band idiom
///
/// ```csharp
/// if (legFrame.Y < Height * first) legFrame.Y = Height * last;
/// else if (legFrame.Y > Height * last) legFrame.Y = Height * first;
/// ```
/// (`Player.cs:36265-36268`: the walk band rows `7..=19`.) Rows outside the
/// band snap to the nearest edge, rows inside pass through, so callers can
/// `row += step` freely and re-wrap every tick.
pub fn wrap_row_band(row: i32, first: i32, last: i32) -> i32 {
    debug_assert!(first <= last);
    if row < first {
        last
    } else if row > last {
        first
    } else {
        row
    }
}

/// The `itemAnimation` thirds used by the useStyle-1 body pose
/// (`Player.cs:36290-36305`): the body holds row 1 for the first two thirds of
/// the swing, then rows 2 and 3. `remaining` is `itemAnimation /
/// itemAnimationMax` (1 at the swing start, 0 at the end).
pub fn use_style1_body_row(remaining: f32) -> u32 {
    if remaining < 0.333 {
        3
    } else if remaining < 0.666 {
        2
    } else {
        1
    }
}

/// The UI text pulse behind `Main.mouseTextColor`
/// (`Main.cs`: `mouseTextColorCounter += 0.017`-ish per tick,
/// `mouseTextColor = (int)(155.0 + Math.Sin(counter) * 155.0)` clamped — see
/// `Main.Update`) resampled per frame instead of per tick; UI code uses it for
/// the breathing alpha of titles and hovered labels. Returns the value in the
/// C# range `[centre - amplitude, centre + amplitude]`.
pub fn sine_pulse(centre: f32, amplitude: f32, omega: f32, time: f32) -> f32 {
    centre + amplitude * time.sin() * omega
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counter_emits_one_step_per_threshold() {
        // Player.cs:36259-36264: walking at |vx| = 2 accumulates 2.6/tick, so a
        // row advance happens every ceil-crossover of 8.0 — roughly every 3rd
        // tick; the counter must carry the remainder.
        let mut c = FrameCounter::new(8.0);
        let mut steps = 0;
        for _ in 0..4 {
            steps += c.advance(2.0 * 1.3);
        }
        assert_eq!(steps, 1);
        // The remainder survives: 2.6 * 4 - 8 = 2.4, so the next cycle again
        // takes 3 ticks.
        assert!((c.value() - 2.4).abs() < 1e-4);
    }

    #[test]
    fn counter_carries_multiple_steps_for_fast_speeds() {
        // The C# `while` drains the whole surplus in one tick.
        let mut c = FrameCounter::new(8.0);
        assert_eq!(c.advance(30.0), 3);
        assert!(c.value() < 8.0);
    }

    #[test]
    fn counter_reset_matches_the_idle_branch() {
        let mut c = FrameCounter::new(8.0);
        c.advance(5.0);
        c.reset();
        assert_eq!(c.value(), 0.0);
    }

    #[test]
    fn band_wrap_snaps_to_the_opposite_edge() {
        // Player.cs:36265-36268: rows 7..=19.
        assert_eq!(wrap_row_band(6, 7, 19), 19);
        assert_eq!(wrap_row_band(20, 7, 19), 7);
        assert_eq!(wrap_row_band(7, 7, 19), 7);
        assert_eq!(wrap_row_band(19, 7, 19), 19);
    }

    #[test]
    fn use_style1_body_rows_follow_the_thirds() {
        // Player.cs:36290-36305.
        assert_eq!(use_style1_body_row(1.0), 1);
        assert_eq!(use_style1_body_row(0.666), 1);
        assert_eq!(use_style1_body_row(0.665), 2);
        assert_eq!(use_style1_body_row(0.333), 2);
        assert_eq!(use_style1_body_row(0.332), 3);
        assert_eq!(use_style1_body_row(0.0), 3);
    }

    #[test]
    fn sine_pulse_stays_inside_the_amplitude() {
        for t in 0..600 {
            let v = sine_pulse(200.0, 55.0, 0.05, t as f32 * 0.1);
            assert!((145.0..=255.0).contains(&v), "v = {v}");
        }
    }
}
