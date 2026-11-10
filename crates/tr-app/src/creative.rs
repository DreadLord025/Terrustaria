//! The Journey-mode powers — `GameContent/Creative/CreativePowers.cs` +
//! `CreativePowerManager.cs`. The vanilla registers ~20 powers across the
//! four state families (the per-player/shared toggles, the buttons, the
//! sliders); the port v1 carries the registry with the shared/per-player
//! toggle+button+slider states and the first functional powers: the time
//! freeze, the time-rate slider, the day/night/noon/midnight buttons.
//!
//! ## The wire ids (`PowerId` is not the wire)
//! `CreativePowerManager.Register` hands each power a `ushort` from a counter
//! (`CreativePowerManager.cs:35-48`) in `Initialize`'s order
//! (`CreativePowerManager.cs:86-104`), and that id is the one the world file
//! carries (`SaveToWorld:125-137`/`LoadFromWorld:139-151`). The port's
//! [`PowerId`] order is its own — the powers it added later — so the mapping
//! lives in the [`PERSISTENT_POWERS`] table below rather than in the enum's
//! discriminants.
//! Only the `IPersistentPerWorldContent` half of the registry is writable:
//! the four `Start*Immediately` buttons (ids 1-4) and `FarPlacementRangePower`
//! (11) are buttons, `GodmodePower` (5) and `SpawnRateSliderPerPlayerPower`
//! (14) are per-player, and `ModifyWindDirectionAndStrength` (6) /
//! `ModifyRainPower` (7) are sliders that do not implement the interface —
//! `SaveToWorld` never writes any of them, and `LoadFromWorld` aborts on the
//! ids it reads back.
//!
//! The byte codec itself is `tr_core::wld::sections::creative_powers`; the two
//! halves below are the state owner's seam for it (Phase 7's wiring calls
//! [`persistent_powers`] on save and [`apply_persistent`] on load).

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

/// One power's state (`ASharedTogglePower`'s value halves — the per-player
/// split lands with the multiplayer pass; single-player folds them).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PowerState {
    /// The toggle's boolean (`_shouldBeActive`).
    Toggle(bool),
    /// The button's momentary press (`_shouldBeActive`'s one-shot).
    Button,
    /// The slider's *derived* value: `ModifyTimeRate.TargetTimeRate`'s
    /// 1..24 (`CreativePowers.cs:877-880`'s remap of the `[0,1]`
    /// `_sliderCurrentValueCache`), not the cache itself — the codec
    /// (`tr_core::wld::sections::creative_powers`) owns that remap.
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

/// One `IPersistentPerWorldContent` registration of
/// `CreativePowerManager.cs:86-104`, as the world file's records carry it: the
/// registration-order `ushort`, the C# class and server-config name that own
/// it, and the port's power when the port has one.
pub struct PersistentPowerId {
    /// The u16 on the wire — `Register`'s counter
    /// (`CreativePowerManager.cs:35-55`).
    pub vanilla_id: u16,
    /// The C# power class (`CreativePowers.cs`): `FreezeTime` at `:1441`,
    /// `ModifyTimeRate` at `:861`, `FreezeRainPower` at `:1507`,
    /// `FreezeWindDirectionAndStrength` at `:1474`, `DifficultySliderPower` at
    /// `:1002`, `StopBiomeSpreadPower` at `:1540`.
    pub class_name: &'static str,
    /// The `Register` name — the server-config key (`time_setfrozen`, …).
    pub config_name: &'static str,
    /// The port's power, when the port carries it. `None` = the vanilla
    /// per-world power the port has not ported: the save seam never writes it
    /// (nothing invents an id) and the load seam drops its record.
    pub port: Option<PowerId>,
}

/// The six per-world persistent powers in registration order — the whole set
/// `SaveToWorld:125-137` writes. Ascending by [`PersistentPowerId::vanilla_id`]
/// so the save seam emits the C#'s `_powersById` walk order directly.
pub const PERSISTENT_POWERS: &[PersistentPowerId] = &[
    PersistentPowerId {
        vanilla_id: 0,
        class_name: "FreezeTime",
        config_name: "time_setfrozen",
        port: Some(PowerId::FreezeTime),
    },
    PersistentPowerId {
        vanilla_id: 8,
        class_name: "ModifyTimeRate",
        config_name: "time_setspeed",
        port: Some(PowerId::TimeRate),
    },
    PersistentPowerId {
        vanilla_id: 9,
        class_name: "FreezeRainPower",
        config_name: "rain_setfrozen",
        // The port has no rain system yet, so the power does not exist.
        port: None,
    },
    PersistentPowerId {
        vanilla_id: 10,
        class_name: "FreezeWindDirectionAndStrength",
        config_name: "wind_setfrozen",
        // Ditto the wind system.
        port: None,
    },
    PersistentPowerId {
        vanilla_id: 12,
        class_name: "DifficultySliderPower",
        config_name: "setdifficulty",
        // The journey difficulty flip is M6-CREATIVE-1's, not this seam's.
        port: None,
    },
    PersistentPowerId {
        vanilla_id: 13,
        class_name: "StopBiomeSpreadPower",
        config_name: "biomespread_setfrozen",
        port: Some(PowerId::StopBiomeSpread),
    },
];

impl CreativePowers {
    /// The state a power carries, if the registry holds one.
    pub fn state(&self, id: PowerId) -> Option<&PowerState> {
        self.states
            .iter()
            .find(|(eid, _)| *eid == id)
            .map(|(_, state)| state)
    }
}

/// The save seam for world-file section 11: `SaveToWorld:125-137`'s input over
/// the port's [`CreativePowers`]. Walks [`PERSISTENT_POWERS`] (so the records
/// come out in the C#'s registration order) and emits a record only for the
/// powers the port carries — the vanilla persistent powers the port has not
/// ported stay unwritten rather than being faked with a default.
///
/// The payload's shape is the state's own variant, which is the same split the
/// C# expresses through the class's interface membership: a toggle's
/// `Enabled` is a `bool` (`FreezeTime.Save:1458`), a slider's
/// `_sliderCurrentValueCache` an `f32` (`ModifyTimeRate.Save:967`,
/// `DifficultySliderPower.Save:1162`). A slider's f32 here is the **derived
/// rate** (1..24, `ModifyTimeRate.TargetTimeRate`); the section codec remaps
/// it to the `[0,1]` cache the file carries in both directions — this seam
/// never sees the cache.
pub fn persistent_powers(powers: &CreativePowers) -> tr_core::wld::sections::WldCreativePowers {
    let mut records = Vec::new();
    for power in PERSISTENT_POWERS {
        let Some(port) = power.port else { continue };
        let value = match powers.state(port) {
            Some(PowerState::Toggle(v)) => tr_core::wld::sections::WldCreativeValue::Bool(*v),
            Some(PowerState::Slider(v)) => tr_core::wld::sections::WldCreativeValue::F32(*v),
            // A button has no per-world payload, and an unregistered power
            // would have to be faked: neither writes a record.
            Some(PowerState::Button) | None => continue,
        };
        records.push((power.vanilla_id, value));
    }
    tr_core::wld::sections::WldCreativePowers { records }
}

/// The load seam for world-file section 11: `LoadFromWorld:139-151`'s effect
/// over the port's [`CreativePowers`]. Each record is applied to the ported
/// power its id names; records for powers the port has not ported are dropped
/// (the vanilla state has no home here, and inventing a `PowerId` for them
/// would put an invented id on the wire the next time the world saves).
pub fn apply_persistent(
    powers: &mut CreativePowers,
    records: &tr_core::wld::sections::WldCreativePowers,
) {
    for (id, value) in &records.records {
        let Some(power) = PERSISTENT_POWERS.iter().find(|p| p.vanilla_id == *id) else {
            continue; // not per-world content: the codec's abort already dropped it
        };
        let Some(port) = power.port else { continue };
        let state = match *value {
            tr_core::wld::sections::WldCreativeValue::Bool(b) => PowerState::Toggle(b),
            tr_core::wld::sections::WldCreativeValue::F32(f) => PowerState::Slider(f),
        };
        powers.set(port, state);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tr_core::wld::sections::{WldCreativePowers, WldCreativeValue};

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
        assert_eq!(
            p.button_target(PowerId::SetMidnight),
            Some((false, 16200.0))
        );
        assert_eq!(p.button_target(PowerId::Godmode), None);
    }

    #[test]
    fn test_DW_6_3_the_vanilla_id_map_pins_the_six_persistent_powers() {
        // `CreativePowerManager.cs:86-104` — the registration order is the
        // wire, and the port's `PowerId` order is not. Each row is pinned by
        // id, C# class, server-config name and portedness.
        let rows: Vec<(u16, &str, &str, Option<PowerId>)> = PERSISTENT_POWERS
            .iter()
            .map(|p| (p.vanilla_id, p.class_name, p.config_name, p.port))
            .collect();
        assert_eq!(
            rows,
            vec![
                (0, "FreezeTime", "time_setfrozen", Some(PowerId::FreezeTime)),
                (
                    8,
                    "ModifyTimeRate",
                    "time_setspeed",
                    Some(PowerId::TimeRate)
                ),
                (9, "FreezeRainPower", "rain_setfrozen", None),
                (10, "FreezeWindDirectionAndStrength", "wind_setfrozen", None),
                (12, "DifficultySliderPower", "setdifficulty", None),
                (
                    13,
                    "StopBiomeSpreadPower",
                    "biomespread_setfrozen",
                    Some(PowerId::StopBiomeSpread)
                ),
            ]
        );
        // The table is in ascending id order, so the save seam's records
        // already walk in the C#'s `_powersById` order.
        let ids: Vec<u16> = PERSISTENT_POWERS.iter().map(|p| p.vanilla_id).collect();
        assert_eq!(ids, [0, 8, 9, 10, 12, 13]);
        // The ids are the *port's* power set's complement, not its order: the
        // port's buttons (1-4 in vanilla), godmode (5) and far placement (11)
        // are absent because they are not `IPersistentPerWorldContent`.
        assert!(!ids.contains(&1));
        assert!(!ids.contains(&5));
        assert!(!ids.contains(&11));
        // …and the two sliders that are not persistent either.
        assert!(!ids.contains(&6));
        assert!(!ids.contains(&7));
    }

    #[test]
    fn test_DW_6_3_the_persistent_powers_seam_round_trips_the_ported_values() {
        // The ported persistent values: the freeze toggle, the time rate at
        // its maximum, the biome-spread freeze off.
        let mut powers = CreativePowers::new();
        powers.set(PowerId::FreezeTime, PowerState::Toggle(true));
        powers.set(PowerId::TimeRate, PowerState::Slider(24.0));
        let records = persistent_powers(&powers);
        assert_eq!(
            records.records,
            vec![
                (0, WldCreativeValue::Bool(true)),
                (8, WldCreativeValue::F32(24.0)),
                (13, WldCreativeValue::Bool(false)),
            ],
            "freeze, rate, biome spread — the ported three, ascending"
        );
        // …and the load seam lands them back on a fresh registry.
        let mut restored = CreativePowers::new();
        apply_persistent(&mut restored, &records);
        assert!(restored.toggle(PowerId::FreezeTime));
        assert!((restored.slider(PowerId::TimeRate) - 24.0).abs() < 1e-6);
        assert!(!restored.toggle(PowerId::StopBiomeSpread));
        assert_eq!(
            persistent_powers(&restored),
            records,
            "the seam is a round-trip"
        );
    }

    #[test]
    fn test_DW_6_3_the_unported_vanilla_ids_are_never_written() {
        // Freeze rain (9), freeze wind (10) and the difficulty slider (12) are
        // per-world content in vanilla, but the port carries no such power, so
        // nothing fakes an id for them.
        let mut powers = CreativePowers::new();
        powers.set(PowerId::FreezeTime, PowerState::Toggle(true));
        let records = persistent_powers(&powers);
        let ids: Vec<u16> = records.records.iter().map(|(id, _)| *id).collect();
        for absent in [9, 10, 12] {
            assert!(!ids.contains(&absent), "id {absent} must not be invented");
        }
        // A vanilla file that carries them still loads: the records are read by
        // the codec and dropped by this seam, leaving the port's own powers
        // untouched.
        let vanilla = WldCreativePowers {
            records: vec![
                (9, WldCreativeValue::Bool(true)),
                (10, WldCreativeValue::Bool(true)),
                (12, WldCreativeValue::F32(0.33)),
                (0, WldCreativeValue::Bool(true)),
            ],
        };
        let mut loaded = CreativePowers::new();
        apply_persistent(&mut loaded, &vanilla);
        assert!(loaded.toggle(PowerId::FreezeTime), "id 0 still lands");
        assert_eq!(
            loaded.states.len(),
            CreativePowers::new().states.len(),
            "no state is created for the unported ids"
        );
    }

    #[test]
    fn test_DW_6_3_the_slider_stores_the_rate_and_the_codec_remaps_it() {
        // F3 (Phase-6 review N1): `ModifyTimeRate.Save:967` writes the
        // `[0,1]` `_sliderCurrentValueCache`, which the C# derives the rate
        // from (`UpdateInfoFromSliderValueCache:877-880`). The port's state
        // (this seam's input) holds the *rate* — `TargetTimeRate`'s port — so
        // the seam passes 0.5 through as-is and the section codec owns the
        // cache. This test pins the seam's half; the wire half is
        // `creative_powers::test_DW_7_6_*`.
        let mut powers = CreativePowers::new();
        powers.set(PowerId::TimeRate, PowerState::Slider(12.0));
        let records = persistent_powers(&powers);
        let rate = records
            .records
            .iter()
            .find(|(id, _)| *id == 8)
            .expect("the time rate rides id 8");
        assert_eq!(
            rate.1,
            WldCreativeValue::F32(12.0),
            "the seam carries the rate, not the cache"
        );
        let mut restored = CreativePowers::new();
        apply_persistent(&mut restored, &records);
        assert!((restored.slider(PowerId::TimeRate) - 12.0).abs() < 1e-6);
        assert!(
            (restored.time_rate() - 12.0).abs() < 1e-6,
            "the freeze is off"
        );
        // The rate is the slider's own range (`time_rate()` returns it
        // verbatim), so a port-side 0.5 means half a x — deliberately not a
        // vanilla-reachable state.
        powers.set(PowerId::TimeRate, PowerState::Slider(0.5));
        assert!((powers.time_rate() - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_DW_6_3_a_power_with_no_state_writes_no_record() {
        // A button power (no per-world payload in vanilla) and a power with no
        // registry entry at all write nothing rather than faking a default.
        let powers = CreativePowers {
            states: vec![(PowerId::SetDay, PowerState::Button)],
        };
        assert!(
            persistent_powers(&powers).records.is_empty(),
            "a button is not `IPersistentPerWorldContent`"
        );
    }

    #[test]
    fn test_DW_6_3_the_payload_shape_follows_the_state_variant() {
        // The C# decides the payload through the class's interface membership;
        // the port reads it off the state's variant — a toggle writes the
        // `bool Enabled` of `FreezeTime.Save:1458`, a slider the raw f32 of
        // `ModifyTimeRate.Save:967`. The two shapes must not cross.
        let toggles = CreativePowers::new();
        let toggles = persistent_powers(&toggles)
            .records
            .into_iter()
            .collect::<Vec<_>>();
        assert_eq!(toggles[0], (0, WldCreativeValue::Bool(false)));
        assert_eq!(toggles[2], (13, WldCreativeValue::Bool(false)));
        assert_eq!(toggles[1].0, 8, "the rate rides the slider id");
        assert!(matches!(toggles[1].1, WldCreativeValue::F32(_)));
    }

    /// The house rule's citation probe for this module's half of the section:
    /// the anchors the golden rule demands for the ported blocks.
    #[test]
    fn test_DW_6_4_the_citations_are_present() {
        let src = include_str!("creative.rs");
        assert!(
            src.contains("CreativePowerManager.cs:86-104"),
            "the registration order"
        );
        for anchor in [
            "CreativePowerManager.cs:35-48",
            "`SaveToWorld:125-137`",
            "`LoadFromWorld:139-151`",
            "FreezeTime.Save:1458",
            "ModifyTimeRate.Save:967",
            "DifficultySliderPower.Save:1162",
            "ModifyTimeRate.UpdateInfoFromSliderValueCache:877-880",
        ] {
            assert!(src.contains(anchor), "missing {anchor}");
        }
    }
}
