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

/// W4-d52 — the eye's ambient blood drip (`NPC.cs:20062-20067`, site
/// 20064): the `Next(5) == 0` gate EVERY tick (after the rotation snap,
/// before the dawn check), then one type-5 `NewDust` at
/// `(x, y + height*0.25)`, box `(width, (int)(height*0.5))`, speeds
/// `(velocity.X, 2f)`; post-spawn `velocity.X *= 0.5`, `velocity.Y *= 0.1`.
pub fn ambient_drip(
    x: f32,
    y: f32,
    width: i32,
    height: f32,
    vx: f32,
    rng: &mut tr_core::UnifiedRandom,
    dust: &mut tr_core::particles::DustPool,
    ctx: &tr_core::particles::SpawnCtx,
    world: &tr_core::World,
) {
    if rng.next_max(5) != 0 {
        return;
    }
    let idx = dust.new_dust(
        rng,
        ctx,
        world,
        glam::Vec2::new(x, y + height * 0.25),
        width,
        (height * 0.5) as i32,
        5,
        vx,
        2.0,
        0,
        tr_core::particles::Color::default(),
        1.0,
    );
    let d = dust.slot_mut(idx);
    d.velocity.x *= 0.5;
    d.velocity.y *= 0.1;
}

/// W4-d52 — the servant burst's ten type-5 dusts (`NPC.cs:20189-20192`,
/// site 20191): `NewDust(vector2, 20, 20, 5, vector3.X*0.4,
/// vector3.Y*0.4)` ×10, no post-spawn writes. `origin` is vector2 (the
/// spawn point 10× the launch velocity ahead of the centre), `vel` is
/// vector3 (the normalized launch velocity).
pub fn servant_burst_dust(
    origin: (f32, f32),
    vel: (f32, f32),
    rng: &mut tr_core::UnifiedRandom,
    dust: &mut tr_core::particles::DustPool,
    ctx: &tr_core::particles::SpawnCtx,
    world: &tr_core::World,
) {
    for _ in 0..10 {
        dust.new_dust(
            rng,
            ctx,
            world,
            glam::Vec2::new(origin.0, origin.1),
            20,
            20,
            5,
            vel.0 * 0.4,
            vel.1 * 0.4,
            0,
            tr_core::particles::Color::default(),
            1.0,
        );
    }
}

/// W4-d52 — the spin hand-off burst (`NPC.cs:20364-20375`, sites
/// 20373/20379's burst half): the int `PlaySound(3)` queues first (the
/// caller's seam), then 2 × (3 gores: the velocity ARGUMENT pair
/// `Next(-30,31)*0.2` each + `NewGore`'s internal Y `Next(10,31)` / X
/// `Next(-20,21)` kicks — `Gore.cs:1327-1328`; types 8/7/6 are
/// `DangerousGore`, so the armed ChildSafety arm would draw `Next(11,14)`
/// + `NextFloat` per gore — the pool's gate is threaded), then 20 type-5
/// dusts with the same argument-pair recipe, then the case-15 roar
/// queues (the caller's seam). The gore visuals are d55's — the draws
/// stay (the plantera/style30 precedent).
pub fn spin_handoff_burst(
    x: f32,
    y: f32,
    width: i32,
    height: i32,
    child_safety_armed: bool,
    rng: &mut tr_core::UnifiedRandom,
    dust: &mut tr_core::particles::DustPool,
    ctx: &tr_core::particles::SpawnCtx,
    world: &tr_core::World,
) {
    // `:20365-20370` — 2 × 3 gores (types 8, 7, 6), argument draws first.
    for _ in 0..2 {
        for _gore_type in [8, 7, 6] {
            let _gx = rng.next_range(-30, 31) as f32 * 0.2;
            let _gy = rng.next_range(-30, 31) as f32 * 0.2;
            // Gore.cs:1327-1328 — the NewGore-internal kicks.
            let _ky = rng.next_range(10, 31) as f32 * 0.1;
            let _kx = rng.next_range(-20, 21) as f32 * 0.1;
            // Gore.cs:1334-1339 — the armed blood gate (types 8/7/6 are
            // outside SafeGore): the re-type draw + the scale NextFloat.
            if child_safety_armed {
                let _t = rng.next_range(11, 14);
                let _s = rng.next_float();
            }
        }
    }
    // `:20371-20374` — the 20 type-5 dusts (site 20373).
    for _ in 0..20 {
        let dx = rng.next_range(-30, 31) as f32 * 0.2;
        let dy = rng.next_range(-30, 31) as f32 * 0.2;
        dust.new_dust(
            rng,
            ctx,
            world,
            glam::Vec2::new(x, y),
            width,
            height,
            5,
            dx,
            dy,
            0,
            tr_core::particles::Color::default(),
            1.0,
        );
    }
}

/// W4-d52 — the spin phase's per-tick blood drip (`NPC.cs:20379`, site
/// 20379): ONE type-5 `NewDust(base.position, width, height, 5,
/// Next(-30,31)*0.2, Next(-30,31)*0.2)` every spin tick, AFTER the
/// hand-off block, before the 0.98 damp.
pub fn spin_tick_dust(
    x: f32,
    y: f32,
    width: i32,
    height: i32,
    rng: &mut tr_core::UnifiedRandom,
    dust: &mut tr_core::particles::DustPool,
    ctx: &tr_core::particles::SpawnCtx,
    world: &tr_core::World,
) {
    let dx = rng.next_range(-30, 31) as f32 * 0.2;
    let dy = rng.next_range(-30, 31) as f32 * 0.2;
    dust.new_dust(
        rng,
        ctx,
        world,
        glam::Vec2::new(x, y),
        width,
        height,
        5,
        dx,
        dy,
        0,
        tr_core::particles::Color::default(),
        1.0,
    );
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

    // ---- W4-d52 dust witnesses (WRITTEN, NOT RUN — code-only mode) ----

    fn dust_world() -> tr_core::World {
        tr_core::World::new("t".into(), 200, 200)
    }

    fn dust_ctx(w: &tr_core::World) -> tr_core::particles::SpawnCtx {
        tr_core::particles::SpawnCtx {
            game_menu: false,
            game_paused: false,
            generating: false,
            screen_position: glam::Vec2::ZERO,
            screen_size: glam::Vec2::new(w.width as f32 * 16.0, w.height as f32 * 16.0),
            max_dust_to_draw: 6000,
        }
    }

    /// One type-5 `NewDust` default block on an empty pool under the
    /// 50 % band (`Dust.cs:153-193`): the two position rolls, the
    /// velocity pair, the frame `Next(3)`, the scale roll — type 5 has
    /// no per-type tweak and is not a ChildSafety dust.
    fn type5_draws(r: &mut tr_core::UnifiedRandom, box_w: i32, box_h: i32) {
        r.next_max(box_w.max(5) - 4);
        r.next_max(box_h.max(5) - 4);
        r.next_range(-20, 21);
        r.next_range(-20, 21);
        r.next_max(3);
        r.next_range(-20, 21);
    }

    /// `NPC.cs:20062-20066` — the drip's `Next(5)` gate draws EVERY tick;
    /// on a pass one type-5 block (box 100 x 55 for the 100x110 eye).
    /// The mirror replays 300 ticks from one seed.
    #[test]
    fn w4_d52_ambient_drip_gate_draws_every_tick() {
        let w = dust_world();
        let ctx = dust_ctx(&w);
        let mut rng = tr_core::UnifiedRandom::new(52);
        let mut probe = tr_core::UnifiedRandom::new(52);
        let mut dust = tr_core::particles::DustPool::new();
        let mut passes = 0;
        for i in 0..300 {
            ambient_drip(1600.0, 1600.0, 100, 110.0, 3.0, &mut rng, &mut dust, &ctx, &w);
            if probe.next_max(5) == 0 {
                passes += 1;
                type5_draws(&mut probe, 100, 55);
            }
            assert_eq!(rng.peek(), probe.peek(), "drip stream @{i}");
            for s in dust.slots_mut() {
                s.active = false;
            }
        }
        assert!(passes > 20 && passes < 100, "the 1-in-5 gate fired {passes}/300");
    }

    /// `NPC.cs:20064-20066` — the drip's fields: position
    /// `(x, y + height*0.25)` + the block rolls, velocity
    /// `(roll*0.1 + vx) * 0.5` / `(roll*0.1 + 2) * 0.1`.
    #[test]
    fn w4_d52_ambient_drip_writes_the_scaled_velocity() {
        let w = dust_world();
        let ctx = dust_ctx(&w);
        let mut rng = tr_core::UnifiedRandom::new(0);
        let mut dust = tr_core::particles::DustPool::new();
        loop {
            ambient_drip(1600.0, 1600.0, 100, 110.0, 3.0, &mut rng, &mut dust, &ctx, &w);
            if dust.slot(0).active {
                break;
            }
        }
        let mut probe = tr_core::UnifiedRandom::new(0);
        // Replay the failed gates, then the passing block.
        loop {
            if probe.next_max(5) == 0 {
                break;
            }
        }
        let px = 1600.0 + probe.next_max(96) as f32 + 4.0;
        let py = 1600.0 + 110.0 * 0.25 + probe.next_max(51) as f32 + 4.0;
        let vx = (probe.next_range(-20, 21) as f32 * 0.1 + 3.0) * 0.5;
        let vy = (probe.next_range(-20, 21) as f32 * 0.1 + 2.0) * 0.1;
        let d = dust.slot(0);
        assert_eq!(d.dust_type, 5);
        assert_eq!((d.position.x, d.position.y), (px, py));
        assert!((d.velocity.x - vx).abs() < 1e-6);
        assert!((d.velocity.y - vy).abs() < 1e-6);
    }

    /// `NPC.cs:20189-20192` — the servant burst: exactly ten type-5
    /// blocks (20x20), speeds the launch velocity * 0.4, no post writes.
    #[test]
    fn w4_d52_servant_burst_draws_ten_blocks() {
        let w = dust_world();
        let ctx = dust_ctx(&w);
        let mut rng = tr_core::UnifiedRandom::new(52);
        let mut probe = tr_core::UnifiedRandom::new(52);
        let mut dust = tr_core::particles::DustPool::new();
        servant_burst_dust((1600.0, 1600.0), (3.0, -4.0), &mut rng, &mut dust, &ctx, &w);
        for _ in 0..10 {
            type5_draws(&mut probe, 20, 20);
        }
        assert_eq!(rng.peek(), probe.peek(), "burst stream");
        assert_eq!(dust.slots().iter().filter(|s| s.active).count(), 10);
        // The first dust's velocity carries the 0.4-scaled launch.
        let mut probe2 = tr_core::UnifiedRandom::new(52);
        probe2.next_max(16);
        probe2.next_max(16);
        let vx = probe2.next_range(-20, 21) as f32 * 0.1 + 3.0 * 0.4;
        let vy = probe2.next_range(-20, 21) as f32 * 0.1 + -4.0 * 0.4;
        let d = dust.slot(0);
        assert!((d.velocity.x - vx).abs() < 1e-6);
        assert!((d.velocity.y - vy).abs() < 1e-6);
    }

    /// `NPC.cs:20365-20374` — the hand-off burst: 2 x 3 gores (types
    /// 8/7/6, DangerousGore — the armed ChildSafety arm adds the
    /// `Next(11,14)` re-type + the `NextFloat` scale per gore,
    /// `Gore.cs:1334-1339`), each with the argument pair
    /// `Next(-30,31)` x2 then the `Gore.cs:1327-1328` Y/X kicks; then 20
    /// type-5 blocks with the same argument-pair recipe. Both
    /// ChildSafety states mirrored.
    #[test]
    fn w4_d52_spin_handoff_burst_draws_match_the_oracle() {
        let w = dust_world();
        let ctx = dust_ctx(&w);
        for armed in [false, true] {
            let mut rng = tr_core::UnifiedRandom::new(52);
            let mut probe = tr_core::UnifiedRandom::new(52);
            let mut dust = tr_core::particles::DustPool::new();
            spin_handoff_burst(1600.0, 1600.0, 100, 110, armed, &mut rng, &mut dust, &ctx, &w);
            for _ in 0..2 {
                for _gore in 0..3 {
                    probe.next_range(-30, 31);
                    probe.next_range(-30, 31);
                    probe.next_range(10, 31);
                    probe.next_range(-20, 21);
                    if armed {
                        probe.next_range(11, 14);
                        probe.next_float();
                    }
                }
            }
            for _ in 0..20 {
                probe.next_range(-30, 31);
                probe.next_range(-30, 31);
                type5_draws(&mut probe, 100, 110);
            }
            assert_eq!(rng.peek(), probe.peek(), "handoff stream (armed={armed})");
            assert_eq!(dust.slots().iter().filter(|s| s.active).count(), 20);
        }
    }

    /// `NPC.cs:20379` — the per-tick spin drip: one argument pair + one
    /// type-5 block EVERY tick, no gate. The mirror replays 100 ticks.
    #[test]
    fn w4_d52_spin_tick_dust_draws_every_tick() {
        let w = dust_world();
        let ctx = dust_ctx(&w);
        let mut rng = tr_core::UnifiedRandom::new(52);
        let mut probe = tr_core::UnifiedRandom::new(52);
        let mut dust = tr_core::particles::DustPool::new();
        for i in 0..100 {
            spin_tick_dust(1600.0, 1600.0, 100, 110, &mut rng, &mut dust, &ctx, &w);
            probe.next_range(-30, 31);
            probe.next_range(-30, 31);
            type5_draws(&mut probe, 100, 110);
            assert_eq!(rng.peek(), probe.peek(), "spin drip stream @{i}");
            for s in dust.slots_mut() {
                s.active = false;
            }
        }
    }
}
