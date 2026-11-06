//! The Journey-mode powers — `GameContent/Creative/CreativePowers.cs` +
//! `CreativePowerManager.cs`. The vanilla registers ~20 powers across the
//! four state families (the per-player/shared toggles, the buttons, the
//! sliders); the port v1 carries the registry with the shared/per-player
//! toggle+button+slider states and the first functional powers: the time
//! freeze, the time-rate slider, the day/night/noon/midnight buttons.

/// The power id (`CreativePowerManager`'s registration order — the
/// `DefaultPowersEnabledIdStart = 18` prefix for the defaults).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PowerId {
    FreezeTime,
    TimeRate,
    SetDay,
    SetNight,
    SetNoon,
    SetMidnight,
    Godmode,
    FarPlacementRange,
    StopBiomeSpread,
}

/// One power's state (`ASharedTogglePower`/`ASharedButtonPower`/
/// `ASharedSliderPower`'s value halves — the per-player split lands with
/// the multiplayer pass; single-player folds them).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PowerState {
    /// The toggle's boolean (`_shouldBeActive`).
    Toggle(bool),
    /// The button's momentary press (`_shouldBeActive`'s one-shot).
    Button,
    /// The slider's f32 in [min, max] (`_sliderCurrentValue`).
    Slider(f32),
}

/// The registry — `CreativePowerManager._powersById`'s port (the v1
/// carries the powers the sim consumes).
#[derive(Clone, Debug)]
pub struct CreativePowers {
    states: Vec<(PowerId, PowerState)>,
}

impl CreativePowers {
    pub fn new() -> Self {
        Self {
            states: vec![
                (PowerId::FreezeTime, PowerState::Toggle(false)),
                (PowerId::TimeRate, PowerState::Slider(1.0)),
                (PowerId::Godmode, PowerState::Toggle(false)),
                (PowerId::FarPlacementRange, PowerState::Toggle(false)),
                (PowerId::StopBiomeSpread, PowerState::Toggle(false)),
            ],
        }
    }

    /// `TogglePower.SetActive`'s shape — a toggle flips, a slider sets.
    pub fn set(&mut self, id: PowerId, state: PowerState) {
        for (eid, estate) in self.states.iter_mut() {
            if *eid == id {
                *estate = state;
                return;
            }
        }
        self.states.push((id, state));
    }

    pub fn toggle(&self, id: PowerId) -> bool {
        for (eid, estate) in &self.states {
            if *eid == id {
                if let PowerState::Toggle(v) = estate {
                    return *v;
                }
            }
        }
        false
    }

    pub fn slider(&self, id: PowerId) -> f32 {
        for (eid, estate) in &self.states {
            if *eid == id {
                if let PowerState::Slider(v) = estate {
                    return *v;
                }
            }
        }
        1.0
    }

    /// `ModifyTimeRate`'s effect on the clock tick (`CreativePowers.cs:
    /// 861-1000`): the slider 0..24x, plus `FreezeTime`'s full stop.
    /// Returns the world clock's tick multiplier (0 = frozen).
    pub fn time_rate(&self) -> f32 {
        if self.toggle(PowerId::FreezeTime) {
            return 0.0;
        }
        self.slider(PowerId::TimeRate)
    }

    /// `StartDayImmediately`/`StartNightImmediately`'s targets
    /// (`CreativePowers.cs:769-860`): the day and the night endpoints
    /// (`Main.time = 0` with `dayTime` flipped).
    pub fn button_target(&self, id: PowerId) -> Option<(bool, f64)> {
        match id {
            PowerId::SetDay => Some((true, 0.0)),
            PowerId::SetNight => Some((false, 0.0)),
            PowerId::SetNoon => Some((true, 27000.0)),
            PowerId::SetMidnight => Some((false, 16200.0)),
            _ => None,
        }
    }
}

impl Default for CreativePowers {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_powers_toggle_and_slide() {
        let mut p = CreativePowers::new();
        // The freeze stops the clock.
        assert!(!p.toggle(PowerId::FreezeTime));
        assert!((p.time_rate() - 1.0).abs() < 0.001, "the default rate 1x");
        p.set(PowerId::FreezeTime, PowerState::Toggle(true));
        assert!(p.toggle(PowerId::FreezeTime));
        assert!((p.time_rate() - 0.0).abs() < 0.001, "the freeze stops");
        // The rate slider.
        p.set(PowerId::TimeRate, PowerState::Slider(5.0));
        p.set(PowerId::FreezeTime, PowerState::Toggle(false));
        assert!((p.time_rate() - 5.0).abs() < 0.001, "the rate 5x");
    }

    #[test]
    fn the_time_buttons_target_the_clock_endpoints() {
        // `StartDayImmediately` etc. (`CreativePowers.cs:769-860`): the
        // (dayTime, time) endpoints.
        let p = CreativePowers::new();
        assert_eq!(p.button_target(PowerId::SetDay), Some((true, 0.0)));
        assert_eq!(p.button_target(PowerId::SetNight), Some((false, 0.0)));
        assert_eq!(p.button_target(PowerId::SetNoon), Some((true, 27000.0)));
        assert_eq!(p.button_target(PowerId::SetMidnight), Some((false, 16200.0)));
        assert_eq!(p.button_target(PowerId::Godmode), None);
    }
}
