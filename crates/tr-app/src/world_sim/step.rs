//! The shared fixed-timestep stepper (plan stage A4): every simulation system
//! advances on a 60 Hz clock with a per-frame step cap, no matter the rendered
//! frame rate.

/// One fixed-timestep accumulator: [`tick`](Self::tick) runs a callback once
/// per accumulated `step` seconds, never more than `max_steps` times per frame
/// — the shared shape of the six `while acc >= step && steps < 8` loops that
/// used to live in `impl App`.
pub(crate) struct FixedStepper {
    accum: f32,
    step: f32,
    max_steps: u32,
}

impl FixedStepper {
    /// A stepper ticking every `step` seconds, at most `max_steps` times per
    /// frame.
    pub(crate) fn new(step: f32, max_steps: u32) -> Self {
        Self {
            accum: 0.0,
            step,
            max_steps,
        }
    }

    /// Adds `dt` to the accumulator and runs `f(step)` once per due step.
    /// Returns the number of steps run (the waterfall rescan gate reads it).
    pub(crate) fn tick(&mut self, dt: f32, mut f: impl FnMut(f32)) -> u32 {
        self.accum += dt;
        let mut steps = 0;
        while self.accum >= self.step && steps < self.max_steps {
            f(self.step);
            self.accum -= self.step;
            steps += 1;
        }
        steps
    }

    /// The residual accumulator in seconds — the render interpolation position
    /// between the last simulation step and now (`acc / step`).
    pub(crate) fn residual(&self) -> f32 {
        self.accum
    }

    /// Caps the backlog at `max_steps` worth of steps (the original
    /// `acc = acc.min(step * 8.0)` after each frame): a long stall drops the
    /// surplus instead of bursting on the next frame.
    pub(crate) fn cap_backlog(&mut self) {
        let cap = self.step * self.max_steps as f32;
        if self.accum > cap {
            self.accum = cap;
        }
    }

    /// The player stepper's variant: a backlog beyond `max_steps` steps resets
    /// to zero instead of clamping (`if acc > step * 8.0 { acc = 0.0 }`).
    pub(crate) fn drop_backlog(&mut self) {
        let cap = self.step * self.max_steps as f32;
        if self.accum > cap {
            self.accum = 0.0;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tick_runs_at_most_max_steps_per_frame() {
        let mut s = FixedStepper::new(1.0 / 60.0, 8);
        // A long stall runs the cap and the surplus backlog is clamped...
        assert_eq!(s.tick(1.0, |_| {}), 8);
        s.cap_backlog();
        assert!((s.residual() - 8.0 / 60.0).abs() < 1e-4, "{}", s.residual());
        // ...while the player stepper resets the backlog instead.
        let mut p = FixedStepper::new(1.0 / 60.0, 8);
        assert_eq!(p.tick(1.0, |_| {}), 8);
        p.drop_backlog();
        assert_eq!(p.residual(), 0.0);
        // The next frame's dues come out of the kept backlog.
        let mut s = FixedStepper::new(1.0 / 60.0, 8);
        assert_eq!(s.tick(1.5 / 60.0, |_| {}), 1);
        assert!((s.residual() - 0.5 / 60.0).abs() < 1e-4, "{}", s.residual());
    }
}
