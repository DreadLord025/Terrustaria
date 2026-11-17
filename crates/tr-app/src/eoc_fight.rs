//! The Eye of Cthulhu's `aiStyle == 4` phase machine (`NPC.cs:19960-20796`),
//! factored into pure predicates: the hp thresholds that flip `ai[0]`, the
//! dash speeds, the servant cadence and the dawn flee. `Npc::update_eye_of_cthulhu`
//! consumes these so the numbers have one home and a test.

/// `ai[0] == 0` — the hover / dive / dash chain (`NPC.cs:20075-20286`).
pub const PHASE_HOVER: f32 = 0.0;
/// `ai[0] == 1` — the first spin phase (`NPC.cs:20288-20371`).
pub const PHASE_SPIN: f32 = 1.0;
/// `ai[0] == 2` — the second spin phase (`:20346-20360`).
pub const PHASE_SPIN2: f32 = 2.0;
/// `ai[0] == 3` — the final dash machine, `defense = 0` (`:20372-20796`).
pub const PHASE_FINAL: f32 = 3.0;

/// `float num27 = 0.5f` (`NPC.cs:20270-20274`): the classic hp fraction
/// where the hover arm hands over to the spin.
pub const PHASE2_HP_CLASSIC: f32 = 0.5;
/// `if (Main.expertMode) num27 = 0.65f` (`NPC.cs:20272-20274`).
pub const PHASE2_HP_EXPERT: f32 = 0.65;

/// `float num22 = 6f` — the phase-1 dive speed (`NPC.cs:20140-20144`).
pub const DASH_SPEED_HOVER: f32 = 6.0;
/// `float num42 = 6.8f` — the final phase's dive speed (`NPC.cs:20478-20483`).
pub const DASH_SPEED_FINAL: f32 = 6.8;

/// `ai[1] >= 100f` — one spin phase's length (`NPC.cs:20314-20316`).
pub const SPIN_TICKS: i32 = 100;

/// `this.ai[3] >= 110f` — the classic hover's servant timer
/// (`NPC.cs:20136-20138`).
pub const SERVANT_PERIOD_CLASSIC: i32 = 110;
/// `num16 *= 0.4f` under `Main.expertMode` → 44 ticks (`NPC.cs:20128-20132`).
pub const SERVANT_PERIOD_EXPERT: i32 = 44;

/// `if ((float)this.life < (float)this.lifeMax * num27)` (`NPC.cs:20271`):
/// strict `<`, classic 50% / expert 65%, checked every hover tick.
pub fn enters_spin_phase(life: i32, life_max: i32, expert: bool) -> bool {
    let fraction = if expert {
        PHASE2_HP_EXPERT
    } else {
        PHASE2_HP_CLASSIC
    };
    (life as f32) < (life_max as f32) * fraction
}

/// The dive speed of a phase: `num22 = 6f` for `ai[0] == 0`
/// (`NPC.cs:20140-20144`) and `num42 = 6.8f` for `ai[0] == 3`
/// (`NPC.cs:20478-20483`).
pub fn dash_speed(phase: f32) -> f32 {
    if phase == PHASE_FINAL {
        DASH_SPEED_FINAL
    } else {
        DASH_SPEED_HOVER
    }
}

/// The dive velocity: `num25 = num22 / sqrt(len)` then `velocity = delta *
/// num25` (`NPC.cs:20151-20158`, `:20491-20496`).
pub fn dash_velocity(from: (f32, f32), to: (f32, f32), phase: f32) -> (f32, f32) {
    let (dx, dy) = (to.0 - from.0, to.1 - from.1);
    let len = (dx * dx + dy * dy).sqrt();
    if len == 0.0 {
        // The C# would divide by zero; a coincident aim has no direction.
        return (0.0, 0.0);
    }
    let k = dash_speed(phase) / len;
    (dx * k, dy * k)
}

/// The servant burst cadence (`NPC.cs:20128-20132`): 110 classic, 44 expert.
pub fn servant_period(expert: bool) -> i32 {
    if expert {
        SERVANT_PERIOD_EXPERT
    } else {
        SERVANT_PERIOD_CLASSIC
    }
}

/// `if (this.ai[3] >= num16)` (`NPC.cs:20133-20135`).
pub fn servant_due(timer: f32, expert: bool) -> bool {
    timer >= servant_period(expert) as f32
}

/// `if (Main.IsItDay() | dead)` (`NPC.cs:20066-20073`): the climb-away and
/// `EncourageDespawn(10)`.
pub fn flees_at_dawn(is_day: bool, player_dead: bool) -> bool {
    is_day || player_dead
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `NPC.cs:20270-20284`: the 2800-hp eye flips `ai[0] = 1` strictly below
    /// half health; the expert gate moves to 65%.
    #[test]
    fn phase2_threshold() {
        assert!(!enters_spin_phase(1400, 2800, false), "exactly 50% still hovers");
        assert!(enters_spin_phase(1399, 2800, false), "below 50% spins");
        assert!(enters_spin_phase(1, 2800, false));
        // Expert: `num27 = 0.65f` → 1820 hp is the boundary.
        assert!(!enters_spin_phase(1820, 2800, true));
        assert!(enters_spin_phase(1819, 2800, true));
    }

    /// `NPC.cs:20140-20158` / `:20478-20496`: the dive locks the normalized
    /// aim at 6 px/tick (phase 1) or 6.8 px/tick (phase 3).
    #[test]
    fn dash_velocity_pinned() {
        let (vx, vy) = dash_velocity((0.0, 0.0), (3.0, 4.0), PHASE_HOVER);
        assert!((vx - 3.6).abs() < 1e-5, "phase 1 vx {vx}");
        assert!((vy - 4.8).abs() < 1e-5, "phase 1 vy {vy}");
        assert_eq!(dash_speed(PHASE_HOVER), 6.0);
        assert_eq!(dash_speed(PHASE_FINAL), 6.8);
        let (vx, vy) = dash_velocity((0.0, 0.0), (3.0, 4.0), PHASE_FINAL);
        assert!((vx - 4.08).abs() < 1e-5, "phase 3 vx {vx}");
        assert!((vy - 5.44).abs() < 1e-5, "phase 3 vy {vy}");
        // The eye's own 100x110 box is not the aim point: the C# aims from
        // the centre, so a zero-length delta yields no NaN.
        assert_eq!(dash_velocity((5.0, 5.0), (5.0, 5.0), PHASE_HOVER), (0.0, 0.0));
    }

    /// `NPC.cs:20128-20138`: the hover's `ai[3]` burst fires at 110 ticks;
    /// the expert `num16 *= 0.4f` makes it 44.
    #[test]
    fn servants_on_schedule() {
        assert_eq!(servant_period(false), 110);
        assert_eq!(servant_period(true), 44);
        assert!(!servant_due(109.0, false));
        assert!(servant_due(110.0, false));
        assert!(!servant_due(43.0, true));
        assert!(servant_due(44.0, true));
    }

    /// `NPC.cs:20066-20073`: `Main.IsItDay() | dead` is the whole gate.
    #[test]
    fn the_eye_flees_at_dawn() {
        assert!(flees_at_dawn(true, false));
        assert!(flees_at_dawn(false, true));
        assert!(flees_at_dawn(true, true));
        assert!(!flees_at_dawn(false, false));
    }
}
