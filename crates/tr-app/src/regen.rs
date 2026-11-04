//! Life and mana regeneration — the `Player.UpdateLifeRegen`
//! (`Player.cs:18942-19391`) and `Player.UpdateManaRegen` /
//! `ApplyManaRegenerationDelay` (`Player.cs:19380-19562`) accumulator engines.
//!
//! Everything the port does not have yet is an *input* field with a neutral
//! default: the DoT branches (M2-BUFF-3), the campfire/heart-lantern auras
//! (M2-BUFF-5), the item multipliers (M3-EQUIP) and sitting/sleeping
//! (M7-INTERACT). The engine already applies them in the original order, so
//! those tasks only fill the inputs. Known deferrals, cited inline.

use tr_core::damage::round_bankers;

/// `lifeRegenCount` yields +1 life per 120 (`Player.cs:19286`).
const LIFE_CHUNK: i32 = 120;
/// The fast-lane drain chunk: −600 counter per 5-dmg hit (`Player.cs:19315`).
const FAST_LANE_CHUNK: i32 = 600;
/// `manaRegenCount` yields +1 mana per 120 (`Player.cs:19538`).
const MANA_CHUNK: i32 = 120;
/// The natural-ramp timer caps here (`Player.cs:19227`).
const LIFE_REGEN_TIME_CAP: f32 = 3600.0;

/// The regen accumulators (`Player.lifeRegen`, `lifeRegenTime`,
/// `lifeRegenCount`, `manaRegen`, `manaRegenDelay`, `manaRegenCount` —
/// `Player.cs:1930-1946`).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct RegenState {
    pub life_regen: i32,
    pub life_regen_time: f32,
    pub life_regen_count: i32,
    pub mana_regen: i32,
    pub mana_regen_delay: f32,
    pub mana_regen_count: i32,
}

/// The drain lanes' death attribution — the `PlayerDeathReason.ByOther(N)`
/// numbers the drain loops pass to `KillMe` (`Player.cs:19321-19331` fast,
/// `:19362-19385` generic). The caller precomputes each lane's priority chain
/// (fast 7 → 12 → 8, generic 22 → 9 → 10 → 23 → 8) into
/// [`RegenInputs::dot_death_fast`] / [`RegenInputs::dot_death_generic`]; the
/// engine returns whichever reason its lane killed with
/// (`death.rs::death_text_for` resolves the text).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DotDeath {
    /// `ByOther(7)` — suffocating (fast lane).
    Suffocation,
    /// `ByOther(8)` — both lanes' fallback (`Player.cs:19331,19382`).
    #[default]
    Burned,
    /// `ByOther(9)` — poisoned or venom (generic lane).
    Poison,
    /// `ByOther(10)` — electrified (generic lane).
    Electrified,
    /// `ByOther(12)` — tongued (fast lane).
    Tongued,
    /// `ByOther(22)` — vampireBurningInSunlight (generic lane).
    VampireSunlight,
    /// `ByOther(23)` — chlorophyteSpore (generic lane).
    ChlorophyteSpore,
}

/// Per-tick inputs gathered by the caller (one 60 Hz tick of player state).
/// The manual [`Default`] matters: `extra_ramp_scale` is a *multiplier* whose
/// neutral value is 1.0, not the additive 0.0 of every other field.
#[derive(Debug, Clone, Copy)]
pub struct RegenInputs {
    /// `statLifeMax2` — scales the final life add (`Player.cs:19262`).
    pub max_life: i32,
    /// `statManaMax2`.
    pub max_mana: i32,
    // — debuffs (M2-BUFF-3). The C# branches (`:18951-19100`) each clamp a
    // positive regen to 0, reset the ramp time and subtract their DoT; folding
    // them into one negative sum keeps the drain rate identical, but the chunk
    // size and the death attribution are lane-specific (below) — the caller
    // also derives `dot_fast_lane` and the two death reasons.
    pub dot_regen: i32,
    pub dot_resets_time: bool,
    /// The fast-lane condition `burned || suffocating || (tongued && expert)`
    /// (`:19313`): −600/5 drain chunks with their own death reason and an
    /// early return before the generic lane.
    pub dot_fast_lane: bool,
    /// The fast-lane `KillMe` reason, precomputed by the caller (7 → 12 → 8).
    pub dot_death_fast: DotDeath,
    /// The generic-lane `KillMe` reason, precomputed by the caller
    /// (22 → 9 → 10 → 23 → 8).
    pub dot_death_generic: DotDeath,
    // — auras (M2-BUFF-5) —
    /// `Player.SceneMetrics.HasCampfire`: +1 regen (`:19170`), ×1.1 ramp.
    pub campfire: bool,
    /// `Player.SceneMetrics.HasHeartLantern`: +2 regen (`:19174`).
    pub heart_lantern: bool,
    /// The mana-regen buff: delay cap 20 (`:19388`), the still/grapple triple
    /// and `num9 = 1.0` (`:19471`).
    /// `Player.manaRegenBonus` — the star-in-bottle's +10 (`:19448` adds it
    /// to the base `statManaMax2 / 3 + 1` line).
    pub mana_regen_bonus: i32,
    pub mana_regen_buff: bool,
    // — items / misc states (M3-EQUIP) —
    /// soulDrain / heartyMeal / honey / shinyStone-still regen adds
    /// (`:19156-19168`), folded by the owner.
    pub extra_regen: i32,
    /// aegis +0.2, soulDrain +2, shinyStone-still +4, honey +2 time adds
    /// (`:19112-19137`).
    pub extra_time: f32,
    /// crimson ×1.5 / shinyStone ×1.1 / rabid ramp scale (`:19247-19258`).
    pub extra_ramp_scale: f32,
    /// palladium's +4 straight into the counter (`:19264`).
    pub extra_count: i32,
    // — world / difficulty —
    /// Sitting/sleeping: time +3, ramp ×1.3 (`:19232-19236`) — M7-INTERACT.
    pub sitting_or_sleeping: bool,
    pub expert_mode: bool,
    pub well_fed: bool,
    // — movement —
    /// `velocity.X != 0f && grappling[0] < 0` (`:19246`).
    pub moving_x: bool,
    /// `IsConsideredStandingStill` (`:3323`): |v.X| < 0.05 && |v.Y| < 0.05.
    pub standing_still: bool,
    /// `grappling[0] >= 0` — hooked onto the grapple system (M2-PHYS-6).
    pub grappling: bool,
}

impl Default for RegenInputs {
    fn default() -> Self {
        Self {
            max_life: 0,
            max_mana: 0,
            dot_regen: 0,
            dot_resets_time: false,
            dot_fast_lane: false,
            dot_death_fast: DotDeath::Burned,
            dot_death_generic: DotDeath::Burned,
            campfire: false,
            heart_lantern: false,
            mana_regen_bonus: 0,
            mana_regen_buff: false,
            extra_regen: 0,
            extra_time: 0.0,
            extra_ramp_scale: 1.0,
            extra_count: 0,
            sitting_or_sleeping: false,
            expert_mode: false,
            well_fed: false,
            moving_x: false,
            standing_still: false,
            grappling: false,
        }
    }
}

/// `Player.UpdateLifeRegen` (`Player.cs:18942-19391`), minus the dust/combat
/// text visuals and the per-debuff branches (inputs, above). A DoT drain
/// death returns the lane's [`DotDeath`] attribution for the caller's
/// `KillMe`; hp clamps at 0 as the fallback and the drain stops (the C# loop
/// would keep chunking a dead player — one death per tick here).
pub fn update_life_regen(
    state: &mut RegenState,
    in_: &RegenInputs,
    life: &mut i32,
) -> Option<DotDeath> {
    // `ResetEffects` zeroes the per-tick regen (`Player.cs:18333`).
    state.life_regen = 0;
    if in_.dot_regen != 0 {
        if state.life_regen > 0 {
            state.life_regen = 0;
        }
        if in_.dot_resets_time {
            state.life_regen_time = 0.0;
        }
        state.life_regen += in_.dot_regen;
    } else if in_.dot_resets_time {
        // Bleed blocks the natural ramp without draining (`:19178-19181`).
        state.life_regen_time = 0.0;
    }
    state.life_regen_time += 1.0; // (`:19112`)
    state.life_regen_time += in_.extra_time;
    state.life_regen += in_.extra_regen;
    if in_.campfire {
        state.life_regen += 1;
    }
    if in_.heart_lantern {
        state.life_regen += 2;
    }
    // The starving lane early-returns in the C# as well (the regen setup
    // `:19139-19158` and the drain lane `:19337-19355`) — it arrives with
    // M2-BUFF-7. The burned/suffocation fast lane landed below.
    let mut num3 = 0.0f32;
    for t in [300.0, 600.0, 900.0, 1200.0, 1500.0, 1800.0, 2400.0, 3000.0] {
        if state.life_regen_time >= t {
            num3 += 1.0;
        }
    }
    if state.life_regen_time >= LIFE_REGEN_TIME_CAP {
        num3 += 1.0;
        state.life_regen_time = LIFE_REGEN_TIME_CAP;
    }
    if in_.sitting_or_sleeping {
        state.life_regen_time += 3.0;
        num3 *= 1.3;
    }
    num3 = if in_.moving_x {
        num3 * 0.5
    } else {
        num3 * 1.25
    };
    num3 *= in_.extra_ramp_scale;
    if in_.campfire {
        num3 *= 1.1;
    }
    if in_.expert_mode && !in_.well_fed {
        num3 /= 2.0;
    }
    // `(int)Math.Round(num3 * (statLifeMax2 / 400f * 0.85f + 0.15f))` — banker's
    // rounding on the f32 product (`:19262`).
    state.life_regen +=
        round_bankers((num3 * (in_.max_life as f32 / 400.0f32 * 0.85f32 + 0.15f32)) as f64) as i32;
    state.life_regen_count += state.life_regen;
    state.life_regen_count += in_.extra_count;
    while state.life_regen_count >= LIFE_CHUNK {
        state.life_regen_count -= LIFE_CHUNK;
        if *life < in_.max_life {
            *life += 1;
        }
        if *life > in_.max_life {
            *life = in_.max_life;
        }
    }
    // Already gone — the drain stops before the loop (the sim caller gates on
    // hp > 0; the ≤0 clamp below stays the fallback for direct engine users).
    if *life <= 0 {
        return None;
    }
    // The fast drain lane (`:19313-19334`): burned || suffocating ||
    // (tongued && expert) — −600 counter chunks of 5 dmg, then an early
    // return that keeps the generic lane out of this tick. Death resolves
    // `KillMe(ByOther(dot_death_fast))` (`:19319-19333`).
    if in_.dot_fast_lane {
        while state.life_regen_count <= -FAST_LANE_CHUNK {
            state.life_regen_count += FAST_LANE_CHUNK;
            *life -= 5; // `HurtLifeRegen(5)` (`:19316-19318`) — no iframes.
            if *life <= 0 {
                // Deviation: the C# loop keeps chunking a dead player; the
                // port stops at the first hp ≤ 0 crossing — one death per
                // tick (docs/ANALYSIS.md, Phase 4).
                *life = 0;
                return Some(in_.dot_death_fast);
            }
        }
        return None;
    }
    // The generic negative drain (`:19357-19385`): at most 4 hp per examined
    // chunk, scaling with how deep the counter is; death resolves
    // `KillMe(ByOther(dot_death_generic))` (`:19361-19384`).
    while state.life_regen_count <= -LIFE_CHUNK {
        let num11 = (state.life_regen_count / -LIFE_CHUNK).min(4);
        state.life_regen_count += LIFE_CHUNK * num11;
        *life -= num11;
        if *life <= 0 {
            // Deviation: see the fast lane — stop at the first kill.
            *life = 0;
            return Some(in_.dot_death_generic);
        }
    }
    None
}

/// `Player.ApplyManaRegenerationDelay` (`Player.cs:19380-19397`). The caller
/// (the mana spend — M3-ITEM-4) invokes it whenever mana is consumed.
pub fn apply_mana_regen_delay(state: &mut RegenState, max_mana: i32, mana: i32, buff: bool) {
    state.mana_regen_delay =
        (((1.0f32 - mana as f32 / max_mana as f32) * 60.0 * 4.0 + 45.0) * 0.7) as i32 as f32;
    if buff && state.mana_regen_delay > 20.0 {
        state.mana_regen_delay = 20.0;
    }
}

/// `Player.UpdateManaRegen` (`Player.cs:19418-19562`). Returns true when this
/// tick refilled the last missing mana point (the full-bar sound + dust in the
/// C# — M6-AUDIO polish). `manaRegenDelayBonus` and the arcane crystal have no
/// sources yet and are folded out (noted in ANALYSIS).
pub fn update_mana_regen(state: &mut RegenState, in_: &RegenInputs, mana: &mut i32) -> bool {
    if state.mana_regen_delay > 0.0 {
        state.mana_regen_delay -= 1.0;
        if in_.standing_still || in_.grappling || in_.mana_regen_buff {
            state.mana_regen_delay -= 1.0;
        }
    }
    let mut refilled = false;
    if state.mana_regen_delay <= 0.0 {
        // `Player.cs:19448`: `statManaMax2 / 3 + manaRegenBonus + 1`.
        let num5 = in_.max_mana / 3 + in_.mana_regen_bonus + 1;
        state.mana_regen = num5;
        if in_.standing_still || in_.grappling || in_.mana_regen_buff {
            state.mana_regen += num5;
        }
        let num6 = *mana as f32 / in_.max_mana as f32;
        let num9: f32 = if in_.mana_regen_buff { 1.0 } else { 0.5 };
        let num10 = num9 + (1.0 - num9) * num6;
        state.mana_regen = (state.mana_regen as f32 * num10) as i32;
        if state.mana_regen < 2 {
            state.mana_regen = 2;
        }
    } else {
        state.mana_regen = 0;
    }
    state.mana_regen_count += state.mana_regen;
    while state.mana_regen_count >= MANA_CHUNK {
        state.mana_regen_count -= MANA_CHUNK;
        if *mana < in_.max_mana {
            *mana += 1;
            refilled = true;
        }
        if *mana >= in_.max_mana {
            *mana = in_.max_mana;
        }
    }
    refilled
}

#[cfg(test)]
mod tests {
    use super::*;

    fn inputs(max_life: i32, max_mana: i32) -> RegenInputs {
        RegenInputs {
            max_life,
            max_mana,
            ..RegenInputs::default()
        }
    }

    #[test]
    fn ramp_tiers_build_up_over_five_standing_seconds() {
        // max 400 → scale = 1.0; standing ×1.25. Tier 1 → 1.25 → 1 hp add.
        let mut st = RegenState::default();
        let mut life = 300;
        let in_ = inputs(400, 20);
        for _ in 0..299 {
            let _ = update_life_regen(&mut st, &in_, &mut life);
        }
        assert_eq!(st.life_regen_time, 299.0);
        let _ = update_life_regen(&mut st, &in_, &mut life);
        assert_eq!(st.life_regen, 1); // 1 tier × 1.25 → banker's(1.25) = 1
        assert_eq!(life, 300); // count = 1 < 120 — nothing yet
    }

    #[test]
    fn banker_rounding_differs_from_half_up_on_midpoints() {
        // max 400, tier 1, moving: 1 × 0.5 = 0.5 → banker's → 0 (half-up: 1).
        let mut st = RegenState {
            life_regen_time: 300.0,
            ..Default::default()
        };
        let mut life = 300;
        let mut in_ = inputs(400, 20);
        in_.moving_x = true;
        let _ = update_life_regen(&mut st, &in_, &mut life);
        assert_eq!(st.life_regen, 0);
        // Tier 2 standing: 2 × 1.25 = 2.5 → banker's → 2 (half-up: 3).
        let mut st2 = RegenState {
            life_regen_time: 600.0,
            ..Default::default()
        };
        let mut life2 = 300;
        let _ = update_life_regen(&mut st2, &inputs(400, 20), &mut life2);
        assert_eq!(st2.life_regen, 2);
    }

    #[test]
    fn moving_halves_and_still_boosts_the_ramp() {
        let mut moving = RegenState::default();
        let mut still = RegenState::default();
        let mut life_a = 400;
        let mut life_b = 400;
        let mut in_ = inputs(400, 20);
        in_.moving_x = true;
        for _ in 0..300 {
            let _ = update_life_regen(&mut moving, &in_, &mut life_a);
        }
        for _ in 0..300 {
            let _ = update_life_regen(&mut still, &inputs(400, 20), &mut life_b);
        }
        assert_eq!(moving.life_regen, 0); // 1 × 0.5 → 0
        assert_eq!(still.life_regen, 1); // 1 × 1.25 → 1
        assert_eq!(still.life_regen_count, 1); // one point granted on tick 300
        assert_eq!(moving.life_regen_count, 0);
    }

    #[test]
    fn accumulator_yields_one_hp_per_120_and_caps() {
        // max 400, tier 8 (time 3000+), standing: 8 × 1.25 = 10 regen/tick —
        // the counter crosses 120 roughly every 12 ticks.
        let mut st = RegenState {
            life_regen_time: 3000.0,
            ..Default::default()
        };
        let mut life = 380;
        let in_ = inputs(400, 20);
        for _ in 0..24 {
            let _ = update_life_regen(&mut st, &in_, &mut life);
        }
        assert!(life > 380 && life <= 400);
        // Overfilling clamps at max.
        let mut st2 = RegenState {
            life_regen_count: 120 * 50,
            ..Default::default()
        };
        let mut life2 = 399;
        for _ in 0..3 {
            let _ = update_life_regen(&mut st2, &inputs(400, 20), &mut life2);
        }
        assert_eq!(life2, 400);
    }

    #[test]
    fn ramp_time_caps_at_3600_and_freezes_the_bonus() {
        let mut st = RegenState {
            life_regen_time: 3599.0,
            ..Default::default()
        };
        let mut life = 300;
        let _ = update_life_regen(&mut st, &inputs(400, 20), &mut life);
        // time hit 3600 inside this tick: tier 8 + the cap tier = 9 → 11.25.
        assert_eq!(st.life_regen_time, 3600.0);
        assert_eq!(st.life_regen, round_bankers(11.25) as i32);
        // The next tick stays at the cap — no further growth.
        let _ = update_life_regen(&mut st, &inputs(400, 20), &mut life);
        assert_eq!(st.life_regen, round_bankers(11.25) as i32);
    }

    #[test]
    fn dot_drains_clamp_positive_regen_and_reset_the_ramp() {
        let mut st = RegenState {
            life_regen_time: 2000.0,
            ..Default::default()
        };
        let mut life = 300;
        let mut in_ = inputs(400, 20);
        in_.dot_regen = -4; // poisoned
        in_.dot_resets_time = true;
        let _ = update_life_regen(&mut st, &in_, &mut life);
        assert_eq!(st.life_regen_time, 1.0); // reset, then +1
                                             // The reset also wiped the ramp — this tick is pure DoT.
        assert_eq!(st.life_regen, -4);
        // A heavy DoT keeps the counter negative and drains through the
        // generic −120 loop (max 4 hp per examined chunk, `:19360`).
        let mut st2 = RegenState::default();
        let mut life2 = 100;
        in_.dot_regen = -60;
        for _ in 0..20 {
            let _ = update_life_regen(&mut st2, &in_, &mut life2);
        }
        assert!(life2 < 100);
    }

    #[test]
    fn campfire_and_expert_modify_the_ramp() {
        let mut st = RegenState {
            life_regen_time: 3000.0,
            ..Default::default()
        };
        let mut life = 300;
        let mut in_ = inputs(400, 20);
        in_.campfire = true;
        let _ = update_life_regen(&mut st, &in_, &mut life);
        // 8 × 1.1 × 1.25 = 11.0, +1 campfire regen → 12.
        assert_eq!(st.life_regen, 12);
        let mut st2 = RegenState {
            life_regen_time: 3000.0,
            ..Default::default()
        };
        let mut life2 = 300;
        let mut in2 = inputs(400, 20);
        in2.expert_mode = true; // not well fed → ÷2
        let _ = update_life_regen(&mut st2, &in2, &mut life2);
        assert_eq!(st2.life_regen, 5); // banker's(8 × 1.25 / 2 = 5.0)
    }

    #[test]
    fn mana_delay_ramps_with_missing_mana_and_counts_down() {
        let mut st = RegenState::default();
        apply_mana_regen_delay(&mut st, 200, 200, false); // full
        assert_eq!(st.mana_regen_delay, 31.0); // (0×240+45)×0.7 = 31.5 → 31
        apply_mana_regen_delay(&mut st, 200, 100, false); // half
        assert_eq!(st.mana_regen_delay, 115.0); // (120+45)×0.7 = 115.5 → 115
                                                // Standing still counts the delay down twice per tick.
        let mut in_ = inputs(400, 200);
        in_.standing_still = true;
        let mut mana = 100;
        for _ in 0..57 {
            update_mana_regen(&mut st, &in_, &mut mana);
        }
        assert!(st.mana_regen_delay > 0.0);
        update_mana_regen(&mut st, &in_, &mut mana);
        assert!(st.mana_regen_delay <= 0.0); // 115 − 2×58 → negative
                                             // Once the delay is gone the regen line starts: base 200/3+1 = 67,
                                             // doubled while still → 134, ×(0.5+0.5·frac).
        assert!(st.mana_regen > 0);
    }

    #[test]
    fn mana_regen_floor_and_refill_per_120() {
        // Nearly full bar, not still: 67 × (0.5 + 0.5·0.99) ≈ 66 → floor n/a;
        // the floor matters at tiny max mana: max 20 → 7 → ×~0.5 → 3 ≥ 2.
        let mut st = RegenState::default();
        let mut mana = 19;
        let in_ = inputs(400, 20);
        for _ in 0..60 {
            update_mana_regen(&mut st, &in_, &mut mana);
        }
        assert_eq!(mana, 20);
        assert!(st.mana_regen_count >= 0 && st.mana_regen_count < 120);
    }

    #[test]
    fn mana_refill_returns_true_on_the_last_point() {
        let mut st = RegenState::default();
        let mut mana = 20;
        let mut in_ = inputs(400, 20);
        in_.standing_still = true;
        // Drain the delay instantly and fill from 19 to 20 — the tick that
        // grants the final point returns true (the C# sound hook).
        st.mana_regen_delay = 0.0;
        mana = 19;
        let mut refills = 0;
        for _ in 0..40 {
            if update_mana_regen(&mut st, &in_, &mut mana) {
                refills += 1;
            }
            if mana == 20 && refills > 0 {
                break;
            }
        }
        assert_eq!(mana, 20);
        assert_eq!(refills, 1);
    }

    #[test]
    fn grapple_counts_as_still_for_mana() {
        let mut st = RegenState::default();
        st.mana_regen_delay = 1.0;
        let mut mana = 100;
        let mut in_ = inputs(400, 200);
        in_.grappling = true;
        update_mana_regen(&mut st, &in_, &mut mana);
        assert!(st.mana_regen_delay <= 0.0); // −1 base −1 grapple
                                             // The regen line doubles while grappled: (200/3+1)×2 = 134, then
                                             // ×(0.5+0.5·(100/200)) = ×0.75 → 100.
        assert_eq!(st.mana_regen, 100);
    }

    #[test]
    fn fast_lane_takes_exactly_five_per_600_chunk_and_returns_early() {
        // DW-1.1 — Player.cs:19313-19334: `while count <= -600` chunks of
        // HurtLifeRegen(5), then `return` keeps the generic −120 lane out of
        // the tick.
        let mut in_ = inputs(400, 20);
        in_.dot_fast_lane = true;
        // One chunk at −720 leaves −120 in the counter: the generic lane
        // would take 1 more hp — exactly 5 proves the early return.
        let mut st = RegenState {
            life_regen_count: -720,
            ..Default::default()
        };
        let mut life = 100;
        let _ = update_life_regen(&mut st, &in_, &mut life);
        assert_eq!(life, 95);
        assert_eq!(st.life_regen_count, -120);
        // The `<=` boundary: −599 does not chunk, −600 does.
        let mut st2 = RegenState {
            life_regen_count: -599,
            ..Default::default()
        };
        let mut life2 = 100;
        let _ = update_life_regen(&mut st2, &in_, &mut life2);
        assert_eq!(life2, 100);
        let mut st3 = RegenState {
            life_regen_count: -600,
            ..Default::default()
        };
        let mut life3 = 100;
        let _ = update_life_regen(&mut st3, &in_, &mut life3);
        assert_eq!(life3, 95);
        // A deep counter chunks repeatedly: −1300 → two 5-dmg chunks.
        let mut st4 = RegenState {
            life_regen_count: -1300,
            ..Default::default()
        };
        let mut life4 = 100;
        let _ = update_life_regen(&mut st4, &in_, &mut life4);
        assert_eq!(life4, 90);
        assert_eq!(st4.life_regen_count, -100);
    }

    #[test]
    fn generic_lane_chunks_are_at_most_four_per_120() {
        // DW-1.2 — Player.cs:19357-19363: num11 = Min(count/−120, 4) hp per
        // chunk; the counter absorbs 120*num11 before the next chunk.
        let chunk = |start: i32, life: i32| -> (i32, i32, Option<DotDeath>) {
            let mut st = RegenState {
                life_regen_count: start,
                ..Default::default()
            };
            let mut hp = life;
            let death = update_life_regen(&mut st, &inputs(400, 20), &mut hp);
            (hp, st.life_regen_count, death)
        };
        assert_eq!(chunk(-119, 100), (100, -119, None)); // below the lane
        assert_eq!(chunk(-120, 100), (99, 0, None)); // one 1-hp chunk
        assert_eq!(chunk(-121, 100), (99, -1, None)); // division truncates
        assert_eq!(chunk(-240, 100), (98, 0, None)); // one 2-hp chunk
        assert_eq!(chunk(-360, 100), (97, 0, None));
        assert_eq!(chunk(-480, 100), (96, 0, None)); // the 4-hp cap
        assert_eq!(chunk(-601, 100), (95, -1, None)); // capped: chunks 4 + 1
    }

    #[test]
    fn generic_lane_death_reasons_follow_the_lane_split() {
        // DW-1.2 — Player.cs:19319-19331 (fast: 7/12/8) and :19361-19384
        // (generic: 22/9/10/23/8): the engine returns the caller-precomputed
        // reason of the killing lane — one unit per number. The priority
        // *selection* itself is Phase 2's derivation.
        let kill = |fast: bool, reason: DotDeath| -> Option<DotDeath> {
            let mut st = RegenState {
                life_regen_count: if fast { -600 } else { -120 },
                ..Default::default()
            };
            let mut in_ = inputs(400, 20);
            in_.dot_fast_lane = fast;
            if fast {
                in_.dot_death_fast = reason;
            } else {
                in_.dot_death_generic = reason;
            }
            let mut hp = 1; // the first chunk kills in both lanes
            update_life_regen(&mut st, &in_, &mut hp)
        };
        for reason in [DotDeath::Suffocation, DotDeath::Tongued, DotDeath::Burned] {
            assert_eq!(kill(true, reason), Some(reason));
        }
        for reason in [
            DotDeath::VampireSunlight,
            DotDeath::Poison,
            DotDeath::Electrified,
            DotDeath::ChlorophyteSpore,
            DotDeath::Burned,
        ] {
            assert_eq!(kill(false, reason), Some(reason));
        }
    }

    #[test]
    fn counter_minus_2000_yields_several_chunks_and_one_reason() {
        // DW-1.4 (dirty) — Player.cs:19315/19357: a deeply negative counter
        // chunks repeatedly in one tick; at most one death reason comes out.
        // Generic: −2000 on the counter → four capped 4-hp chunks.
        let mut st = RegenState::default();
        let mut hp = 100;
        let mut in_ = inputs(400, 20);
        in_.dot_regen = -2000; // one-tick dump onto the counter
        let death = update_life_regen(&mut st, &in_, &mut hp);
        assert_eq!(hp, 84); // 4 × 4 hp
        assert_eq!(st.life_regen_count, -80); // −2000 + 480×4
        assert_eq!(death, None);
        // Fast lane killing mid-way: hp 12 → chunks 7 / 2 / dead — the loop
        // stops at the kill and returns exactly one reason.
        let mut st2 = RegenState::default();
        let mut hp2 = 12;
        in_.dot_fast_lane = true;
        in_.dot_death_fast = DotDeath::Suffocation;
        let death2 = update_life_regen(&mut st2, &in_, &mut hp2);
        assert_eq!(hp2, 0);
        assert_eq!(death2, Some(DotDeath::Suffocation));
        assert_eq!(st2.life_regen_count, -200); // three 600-chunks, then stop
    }

    #[test]
    fn drain_stops_when_hp_already_gone() {
        // Plan edge — hp already 0: the drain stops before the loop and no
        // death reason is manufactured for an already-dead player.
        let mut st = RegenState {
            life_regen_count: -2000,
            ..Default::default()
        };
        let mut hp = 0;
        let mut in_ = inputs(400, 20);
        in_.dot_fast_lane = true;
        let death = update_life_regen(&mut st, &in_, &mut hp);
        assert_eq!(death, None);
        assert_eq!(hp, 0);
        assert_eq!(st.life_regen_count, -2000); // untouched
    }

    #[test]
    fn no_second_chunk_after_death() {
        // Plan dirty — hp ≤ 0 mid-loop: no further chunk after the kill.
        let mut st = RegenState {
            life_regen_count: -120,
            ..Default::default()
        };
        let mut hp = 1;
        let death = update_life_regen(&mut st, &inputs(400, 20), &mut hp);
        assert_eq!(death, Some(DotDeath::Burned)); // the default generic reason
        assert_eq!(hp, 0);
        assert_eq!(st.life_regen_count, 0); // the killing chunk already added 120
                                            // The stop is visible with a deeper counter: one 600-chunk kills hp 3
                                            // and the loop quits with the remainder in the counter.
        let mut st2 = RegenState {
            life_regen_count: -1300,
            ..Default::default()
        };
        let mut hp2 = 3;
        let mut in2 = inputs(400, 20);
        in2.dot_fast_lane = true;
        let death2 = update_life_regen(&mut st2, &in2, &mut hp2);
        assert_eq!(death2, Some(DotDeath::Burned));
        assert_eq!(hp2, 0);
        assert_eq!(st2.life_regen_count, -700); // one chunk, then stop
    }
}
