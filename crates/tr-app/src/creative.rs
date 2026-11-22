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
    /// `ModifyWindDirectionAndStrength` (`CreativePowers.cs:1179-1309`,
    /// vanilla id 6): the slider state is the *derived* wind target
    /// (`Lerp(-0.8, 0.8, cache)`, `:1190`) — not per-world content.
    WindStrength,
    /// `ModifyRainPower` (`:1311-1439`, vanilla id 7): the raw [0,1] cache
    /// (`GetSliderValueInner` reads the live `Main.cloudAlpha`, `:1332-1335`,
    /// so the cache itself is the state) — not per-world content.
    RainStrength,
    /// `FreezeRainPower` (`:1507-1538`, vanilla id 9).
    FreezeRain,
    /// `FreezeWindDirectionAndStrength` (`:1474-1505`, vanilla id 10).
    FreezeWind,
    /// `DifficultySliderPower` (`:1002-1177`, vanilla id 12): the derived
    /// `StrengthMultiplierToGiveNPCs` (`UpdateInfoFromSliderValueCache`,
    /// `:1018-1028`), consumed by `Main.UpdateCreativeGameModeOverride`
    /// (`Main.cs:17294-17305`).
    Difficulty,
    /// `SpawnRateSliderPerPlayerPower` (`:1573-1737`, vanilla id 14): the
    /// raw [0,1] cache, default 0.5 (`_sliderDefaultValue`, `:1580`).
    SpawnRate,
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
                // The weather powers' fresh state: the wind target 0
                // (`Main.windSpeedTarget` starts 0), the rain cache 0 (no
                // event), the freezes off (`ASharedTogglePower.Reset`,
                // `CreativePowers.cs:466-470`).
                (PowerId::WindStrength, PowerState::Slider(0.0)),
                (PowerId::RainStrength, PowerState::Slider(0.0)),
                (PowerId::FreezeRain, PowerState::Toggle(false)),
                (PowerId::FreezeWind, PowerState::Toggle(false)),
                // `DifficultySliderPower.Reset` (`:1012-1016`): cache 0 →
                // strength 0.5 (`UpdateInfoFromSliderValueCache`).
                (PowerId::Difficulty, PowerState::Slider(0.5)),
                // `_sliderDefaultValue = 0.5f` (`:1580`, `ResetForPlayer`
                // `:360-368`).
                (PowerId::SpawnRate, PowerState::Slider(0.5)),
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

    /// `GodmodePower` (`CreativePowers.cs:692-727`): the toggle the sim reads
    /// as `Player.creativeGodMode` (`Player.cs:25493-25497`).
    pub fn god_mode(&self) -> bool {
        self.toggle(PowerId::Godmode)
    }

    /// `FarPlacementRangePower` (`CreativePowers.cs:730-767`): while it is on,
    /// `Player.ResetEffects` doubles the tile range (`Player.cs:18815-18828`).
    pub fn far_placement_range(&self) -> bool {
        self.toggle(PowerId::FarPlacementRange)
    }

    /// `StopBiomeSpreadPower.Enabled` (`CreativePowers.cs:1540-1571`).
    pub fn stop_biome_spread(&self) -> bool {
        self.toggle(PowerId::StopBiomeSpread)
    }

    /// `WorldGen.AllowedToSpreadInfections` as the growth/spread drivers see
    /// it: `WorldGen.UpdateWorld` seeds it to `true` and then overwrites it
    /// with `!power.Enabled` whenever the power is unlocked
    /// (`WorldGen.cs:71502-71507`). Single-player, the power is always
    /// unlocked (`GetIsUnlocked`'s `return true`), so the toggle is the
    /// whole rule.
    pub fn allowed_to_spread_infections(&self) -> bool {
        !self.stop_biome_spread()
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

    /// `FreezeRainPower.Enabled` (`CreativePowers.cs:1507-1538`) — the rain
    /// event machine's gate (`Main.cs:64866`/`:64893`).
    pub fn freeze_rain(&self) -> bool {
        self.toggle(PowerId::FreezeRain)
    }

    /// `FreezeWindDirectionAndStrength.Enabled` (`:1474-1505`) — the wind
    /// target-walk gate (`Main.cs:58854`).
    pub fn freeze_wind(&self) -> bool {
        self.toggle(PowerId::FreezeWind)
    }

    /// `DifficultySliderPower.StrengthMultiplierToGiveNPCs`
    /// (`:1004`, `UpdateInfoFromSliderValueCache` `:1018-1028`): the derived
    /// NPC strength multiplier, the `Main._gameModeDifficultyOverride`
    /// source (`Main.cs:17294-17305`).
    pub fn difficulty_strength(&self) -> f32 {
        self.slider(PowerId::Difficulty)
    }

    /// `Main.UpdateCreativeGameModeOverride` (`Main.cs:17294-17305`):
    /// `Main._gameModeDifficultyOverride` — `Some(strength)` while in a
    /// Journey world with the (always-unlocked) difficulty power, else
    /// `None` (the world's own `GameMode` answers `Main.Difficulty`).
    pub fn game_mode_difficulty_override(&self, journey_mode: bool) -> Option<f32> {
        journey_mode.then(|| self.difficulty_strength())
    }

    /// `SpawnRateSliderPerPlayerPower.GetShouldDisableSpawnsFor`
    /// (`:1584-1595`): the local player's cache at exactly 0 disables
    /// enemy spawns (`NPC.CanSpawnEnemiesNear`, `NPC.cs:350-357`).
    pub fn spawn_rate_disabled(&self) -> bool {
        self.slider(PowerId::SpawnRate) == 0.0
    }

    /// `SpawnRateSliderPerPlayerPower.GetRemappedSliderValueFor`
    /// (`:222-231`) over [`spawn_rate_multiplier`]: the local player's
    /// remapped x0.1..10 spawn-rate factor (`NPC.cs:762-770`).
    pub fn spawn_rate_multiplier(&self) -> f32 {
        spawn_rate_multiplier(self.slider(PowerId::SpawnRate))
    }
}

/// `ModifyWindDirectionAndStrength.UpdateInfoFromSliderValueCache`
/// (`CreativePowers.cs:1188-1191`):
/// `windSpeedCurrent = windSpeedTarget = Lerp(-0.8, 0.8, cache)`.
pub fn wind_target_from_cache(v: f32) -> f32 {
    -0.8 + 1.6 * v
}

/// `ModifyWindDirectionAndStrength.GetSliderValueInner` (`:1193-1196`):
/// `Utils.GetLerpValue(-0.8, 0.8, Main.windSpeedTarget)` — UNCLAMPED
/// (`Utils.cs:315-343`'s default `clamped = false`): the target walk can
/// push past ±0.8 (`Main.cs:58935`'s ±0.85 draws), and the slider reads
/// the out-of-range lerp verbatim.
pub fn wind_cache_from_target(target: f32) -> f32 {
    (target + 0.8) / 1.6
}

/// `DifficultySliderPower.UpdateInfoFromSliderValueCache`
/// (`CreativePowers.cs:1018-1028`): cache <= 0.33 remaps [0,0.33]→[0.5,1],
/// else [0.33,1]→[1,3] (`Utils.Remap` clamps, `Utils.cs:345-348`); then
/// `Math.Round(strength * 20f) / 20f` — banker's rounding to 1/20 steps.
/// The codec owns the math (`wld::sections::creative_powers`) so the wire
/// and the UI derive the same strength.
pub fn difficulty_strength_from_cache(v: f32) -> f32 {
    tr_core::wld::sections::creative_powers::difficulty_strength_from_cache(v)
}

/// `SpawnRateSliderPerPlayerPower.RemapSliderValueToPowerValue`
/// (`CreativePowers.cs:1601-1608`): cache < 0.5 remaps [0,0.5]→[0.1,1],
/// else [0.5,1]→[1,10]. NO rounding — the consumers divide/multiply the
/// raw float (`NPC.cs:767-768`).
pub fn spawn_rate_multiplier(v: f32) -> f32 {
    if v < 0.5 {
        0.1 + 0.9 * (v / 0.5).clamp(0.0, 1.0)
    } else {
        1.0 + 9.0 * ((v - 0.5) / 0.5).clamp(0.0, 1.0)
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
        port: Some(PowerId::FreezeRain),
    },
    PersistentPowerId {
        vanilla_id: 10,
        class_name: "FreezeWindDirectionAndStrength",
        config_name: "wind_setfrozen",
        port: Some(PowerId::FreezeWind),
    },
    PersistentPowerId {
        vanilla_id: 12,
        class_name: "DifficultySliderPower",
        config_name: "setdifficulty",
        port: Some(PowerId::Difficulty),
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
    fn the_sim_powers_read_back_through_their_accessors() {
        // `GodmodePower`/`FarPlacementRangePower`/`StopBiomeSpreadPower` —
        // the three toggles the sim consumes (`CreativePowers.cs:692`,
        // `:730`, `:1540`).
        let mut p = CreativePowers::new();
        assert!(!p.god_mode());
        assert!(!p.far_placement_range());
        assert!(!p.stop_biome_spread());
        // `AllowedToSpreadInfections` starts `true` (`WorldGen.cs:71502`).
        assert!(p.allowed_to_spread_infections());
        p.set(PowerId::Godmode, PowerState::Toggle(true));
        p.set(PowerId::FarPlacementRange, PowerState::Toggle(true));
        p.set(PowerId::StopBiomeSpread, PowerState::Toggle(true));
        assert!(p.god_mode());
        assert!(p.far_placement_range());
        assert!(p.stop_biome_spread());
        assert!(!p.allowed_to_spread_infections(), "the spread stops");
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
                (9, "FreezeRainPower", "rain_setfrozen", Some(PowerId::FreezeRain)),
                (
                    10,
                    "FreezeWindDirectionAndStrength",
                    "wind_setfrozen",
                    Some(PowerId::FreezeWind)
                ),
                (12, "DifficultySliderPower", "setdifficulty", Some(PowerId::Difficulty)),
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
        // The per-player spawn-rate slider (14) rides the .plr, not the .wld.
        assert!(!ids.contains(&14));
    }

    #[test]
    fn test_DW_6_3_the_persistent_powers_seam_round_trips_the_ported_values() {
        // The ported persistent values: the freeze toggle, the time rate at
        // its maximum, the biome-spread freeze off — plus the weather
        // freezes and the difficulty strength at their fresh defaults
        // (`FreezeRainPower.Reset` false, `DifficultySliderPower.Reset`
        // cache 0 → strength 0.5).
        let mut powers = CreativePowers::new();
        powers.set(PowerId::FreezeTime, PowerState::Toggle(true));
        powers.set(PowerId::TimeRate, PowerState::Slider(24.0));
        let records = persistent_powers(&powers);
        assert_eq!(
            records.records,
            vec![
                (0, WldCreativeValue::Bool(true)),
                (8, WldCreativeValue::F32(24.0)),
                (9, WldCreativeValue::Bool(false)),
                (10, WldCreativeValue::Bool(false)),
                (12, WldCreativeValue::F32(0.5)),
                (13, WldCreativeValue::Bool(false)),
            ],
            "all six per-world powers, ascending"
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
        // The wind/rain *sliders* (6/7) and the per-player powers (5/11/14)
        // are not per-world content — nothing fakes an id for them.
        let mut powers = CreativePowers::new();
        powers.set(PowerId::FreezeTime, PowerState::Toggle(true));
        let records = persistent_powers(&powers);
        let ids: Vec<u16> = records.records.iter().map(|(id, _)| *id).collect();
        for absent in [5, 6, 7, 11, 14] {
            assert!(!ids.contains(&absent), "id {absent} must not be invented");
        }
        // A vanilla file that carries a non-persistent id still loads: the
        // codec's unknown-id break drops the tail, and the records read so
        // far land on the port's own powers.
        let vanilla = WldCreativePowers {
            records: vec![
                (9, WldCreativeValue::Bool(true)),
                (10, WldCreativeValue::Bool(true)),
                (12, WldCreativeValue::F32(1.0)),
                (0, WldCreativeValue::Bool(true)),
            ],
        };
        let mut loaded = CreativePowers::new();
        apply_persistent(&mut loaded, &vanilla);
        assert!(loaded.toggle(PowerId::FreezeTime), "id 0 still lands");
        assert!(loaded.freeze_rain(), "id 9 lands on the ported power");
        assert!(loaded.freeze_wind(), "id 10 lands");
        assert!(
            (loaded.difficulty_strength() - 1.0).abs() < 1e-6,
            "id 12 carries the derived strength (the codec owns the cache)"
        );
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
        assert_eq!(toggles[5], (13, WldCreativeValue::Bool(false)));
        assert_eq!(toggles[1].0, 8, "the rate rides the slider id");
        assert!(matches!(toggles[1].1, WldCreativeValue::F32(_)));
    }

    // ---- W4-c24 phase 4: the weather / spawn-rate / difficulty powers ------

    /// `ModifyWindDirectionAndStrength` (`CreativePowers.cs:1188-1196`):
    /// the cache ↔ target lerp pair, the UNCLAMPED read-back.
    #[test]
    fn the_wind_slider_lerps_the_target_both_ways() {
        assert!((wind_target_from_cache(0.0) + 0.8).abs() < 1e-6);
        assert!((wind_target_from_cache(0.5)).abs() < 1e-6);
        assert!((wind_target_from_cache(1.0) - 0.8).abs() < 1e-6);
        // `GetSliderValueInner`'s unclamped GetLerpValue: a walked target
        // past ±0.8 reads past [0,1] verbatim (`Main.cs:58935`'s ±0.85).
        assert!((wind_cache_from_target(0.8) - 1.0).abs() < 1e-6);
        assert!(wind_cache_from_target(0.85) > 1.0);
        assert!(wind_cache_from_target(-0.85) < 0.0);
        // The registry carries the derived target, not the cache.
        let mut p = CreativePowers::new();
        p.set(PowerId::WindStrength, PowerState::Slider(wind_target_from_cache(0.25)));
        assert!((p.slider(PowerId::WindStrength) + 0.4).abs() < 1e-6);
    }

    /// `DifficultySliderPower.UpdateInfoFromSliderValueCache`
    /// (`CreativePowers.cs:1018-1028`): the two-segment remap + the
    /// banker's-rounded 1/20 quantisation.
    #[test]
    fn the_difficulty_slider_remaps_and_quantises() {
        // The endpoints and the segment seam.
        assert_eq!(difficulty_strength_from_cache(0.0), 0.5);
        assert_eq!(difficulty_strength_from_cache(0.33), 1.0);
        assert_eq!(difficulty_strength_from_cache(1.0), 3.0);
        // The label clicks' exact caches (`Click_Normal/Expert/Master`,
        // `:1055-1070`): 0.66 → Remap(0.33,1,1,3) = 1.9850746 → *20 =
        // 39.701 → round 40 → 2.0; 1.0 → 3.0.
        assert_eq!(difficulty_strength_from_cache(0.66), 2.0);
        // The quantisation: 0.5 → Remap = 1.5074627 → *20 = 30.149 → 30 →
        // 1.5; a mid-segment cache lands on the nearest 1/20.
        assert_eq!(difficulty_strength_from_cache(0.5), 1.5);
        // Banker's rounding at an exact .5 tie: strength*20 = 10.5 rounds
        // to 10 (even), not 11. Remap(0,0.33,0.5,1) hits 0.525 at
        // v = 0.33*0.05 = 0.0165.
        assert_eq!(difficulty_strength_from_cache(0.0165), 0.5);
        // The override seam (`Main.cs:17294-17305`): Journey answers the
        // strength, a Classic world answers None (the world's GameMode).
        let p = CreativePowers::new();
        assert_eq!(p.game_mode_difficulty_override(false), None);
        assert_eq!(p.game_mode_difficulty_override(true), Some(0.5));
    }

    /// `SpawnRateSliderPerPlayerPower.RemapSliderValueToPowerValue`
    /// (`CreativePowers.cs:1601-1608`) + `GetShouldDisableSpawnsFor`
    /// (`:1584-1595`): the x0.1..10 remap and the exact-zero disable.
    #[test]
    fn the_spawn_rate_slider_remaps_and_disables_at_zero() {
        assert!((spawn_rate_multiplier(0.0) - 0.1).abs() < 1e-6);
        assert!((spawn_rate_multiplier(0.25) - 0.55).abs() < 1e-6);
        assert!((spawn_rate_multiplier(0.5) - 1.0).abs() < 1e-6);
        assert!((spawn_rate_multiplier(0.75) - 5.5).abs() < 1e-6);
        assert!((spawn_rate_multiplier(1.0) - 10.0).abs() < 1e-6);
        let mut p = CreativePowers::new();
        // The fresh default 0.5 (`_sliderDefaultValue`, `:1580`) is x1.
        assert!(!p.spawn_rate_disabled());
        assert!((p.spawn_rate_multiplier() - 1.0).abs() < 1e-6);
        // Exactly 0 disables (`GetShouldDisableSpawnsFor`, `:1592`).
        p.set(PowerId::SpawnRate, PowerState::Slider(0.0));
        assert!(p.spawn_rate_disabled());
        // A near-zero cache does NOT disable — only the exact 0.
        p.set(PowerId::SpawnRate, PowerState::Slider(0.001));
        assert!(!p.spawn_rate_disabled());
        assert!(p.spawn_rate_multiplier() > 0.1);
    }

    /// The freeze toggles ride the registry and the per-world seam
    /// (`FreezeRainPower.Save:1524`, `FreezeWindDirectionAndStrength
    /// .Save:1491`).
    #[test]
    fn the_weather_freezes_toggle_and_persist() {
        let mut p = CreativePowers::new();
        assert!(!p.freeze_rain());
        assert!(!p.freeze_wind());
        p.set(PowerId::FreezeRain, PowerState::Toggle(true));
        p.set(PowerId::FreezeWind, PowerState::Toggle(true));
        let records = persistent_powers(&p);
        assert!(records.records.contains(&(9, WldCreativeValue::Bool(true))));
        assert!(records.records.contains(&(10, WldCreativeValue::Bool(true))));
        let mut back = CreativePowers::new();
        apply_persistent(&mut back, &records);
        assert!(back.freeze_rain());
        assert!(back.freeze_wind());
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
