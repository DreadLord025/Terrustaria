//! The Old One's Army's VICTORY slice (`GameContent/Events/DD2Event.cs`):
//! the win-side medal table, the wave-break skip and the crystal-drop
//! budget. Pure decision logic only — the NPC sweeps, the `Item.NewItem`
//! spawns, the broadcasts and the victory-scene ai flips are the app-side
//! seam (the driver in `world_sim/npcs.rs` wires them, S1-E-12b). The LOSS
//! arms (`ReportLoss`/`LoseInvasionMessage`) are a separate slice and are
//! deliberately not touched here.
//!
//! RNG order: every function here draws nothing. `DropMedals`/the crystal
//! drops spend their `Main.rand` draws inside `Item.NewItem` itself
//! (`Item.cs:49654-49727`) — the caller must spawn through the same
//! `spawn_item_drop_px*` seam (`world_sim/items.rs`) so the pinned
//! velocity/phase triplet order stays intact. The witness tests below pin
//! that the decisions consume no stream.

/// `DD2Event.WinInvasionInternal` (`:314-342`) — the victory's medal drop:
/// tier 1 → 3, tier 2 → 15, tier 3 → 60 (`:328-340`). `DropMedals` (`:802-813`)
/// drops one `Item.NewItem(..., 3817, numberOfMedals)` stack per live
/// Eternia Crystal (548); `None` here = no drop. The tier flags
/// `DownedInvasionT1/2/3` (`:316-327`) latch at `>= tier`, not `==`.
pub fn victory_medals(difficulty: i32) -> Option<i32> {
    // The three `if (OngoingDifficulty == n)` statements in source order.
    if difficulty == 1 {
        return Some(3); // `:330`
    }
    if difficulty == 2 {
        return Some(15); // `:334`
    }
    if difficulty == 3 {
        // `:338`'s `AchievementsHelper.NotifyProgressionEvent(23)` is an
        // achievement-side effect with no port counterpart yet (deferred).
        return Some(60); // `:339`
    }
    None
}

/// `DD2Event.WinInvasionInternal`'s downed-tier latches (`:316-327`):
/// each `if (OngoingDifficulty >= n)` sets its flag — a tier-3 win latches
/// all three. The flags persist (`Save` `:151-156`).
pub fn victory_downed_flags(difficulty: i32) -> (bool, bool, bool) {
    (difficulty >= 1, difficulty >= 2, difficulty >= 3)
}

/// `DD2Event.CheckProgress`'s wave-complete medal rows (`:405-449`), read
/// AFTER the wave advance (`num3` is the NEW wave). Tier 1 pays on waves
/// 5 then 4 (`:407-414`), tier 2 on 7/6/5 (`:418-429`), tier 3 on 7/6/5/4
/// (`:433-448`). The rows are independent `if`s but a single wave matches
/// at most one, so the sum is the C#'s total. No drop on other waves.
pub fn wave_complete_medals(difficulty: i32, new_wave: i32) -> Option<i32> {
    if difficulty == 1 {
        if new_wave == 5 {
            return Some(1); // `:409`
        }
        if new_wave == 4 {
            return Some(1); // `:413`
        }
    }
    if difficulty == 2 {
        if new_wave == 7 {
            return Some(6); // `:420`
        }
        if new_wave == 6 {
            return Some(3); // `:424`
        }
        if new_wave == 5 {
            return Some(1); // `:428`
        }
    }
    if difficulty == 3 {
        if new_wave == 7 {
            return Some(25); // `:435`
        }
        if new_wave == 6 {
            return Some(11); // `:439`
        }
        if new_wave == 5 {
            return Some(3); // `:443`
        }
        if new_wave == 4 {
            return Some(1); // `:447`
        }
    }
    None
}

/// `DD2Event.StartVictoryScene` (`:481-505`) — the scene's decision half:
/// `WonThisRun = true` (`:483`) always; the first Eternia Crystal (548)
/// flips to `ai[1] = 2, ai[0] = 2` (`:493-494`, `NPC.FindFirstNPC` order)
/// and EVERY live portal (549) to `ai[0] = 0, ai[1] = 1` (`:496-504`).
/// With no crystal (`FindFirstNPC == -1`) the C# returns before touching
/// any portal (`:489-492`). The returned plan is what the app applies;
/// `_damageTracker.Stop(won: true)` (`:484-487`) has no port counterpart
/// yet (the tracker is deferred with the scene's filters/music).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct VictoryScene<'a> {
    /// `DD2Event.WonThisRun = true` — set before the crystal lookup.
    pub won: bool,
    /// The first 548's slot gets `(ai0, ai1) = (2, 2)`; `None` = no crystal.
    pub crystal_slot: Option<usize>,
    /// Every live 549's slot gets `(ai0, ai1) = (0, 1)` — only when the
    /// crystal exists (the C#'s early `return`). The slice borrows the
    /// caller's live scan (S1-E-12c: the driver collects a `Vec`, not a
    /// static table).
    pub portal_slots: &'a [usize],
}

/// Builds the `StartVictoryScene` plan from the NPC scan the caller
/// already has: `crystal` = `NPC.FindFirstNPC(548)` (first active match,
/// `None` = -1), `portals` = the active 549 slots in `Main.npc` order.
pub fn victory_scene<'a>(crystal: Option<usize>, portals: &'a [usize]) -> VictoryScene<'a> {
    VictoryScene {
        won: true, // `:483` — unconditional, before the lookup
        crystal_slot: crystal,
        // `:489-492`: no crystal → return; the portal loop never runs.
        portal_slots: if crystal.is_some() { portals } else { &[] },
    }
}

/// `DD2Event.IsStandActive` (`:1765-1777`) — the click lands near a live
/// Eternia Crystal: `nPC.Bottom.Distance(target) < 36f` with
/// `target = (x*16+8, y*16+8)`. Squared-compare keeps the f32 maths exact
/// (`36f * 36f = 1296`); the C# returns on the FIRST active 548.
pub fn is_stand_active(stand_bottom: (f32, f32), click_tile: (i32, i32)) -> bool {
    let tx = (click_tile.0 * 16 + 8) as f32;
    let ty = (click_tile.1 * 16 + 8) as f32;
    let dx = stand_bottom.0 - tx;
    let dy = stand_bottom.1 - ty;
    dx * dx + dy * dy < 36.0 * 36.0
}

/// `DD2Event.AttemptToSkipWaitTime` (`:1795-1801`) — the wave-break skip:
/// `TimeLeftBetweenWaves > 60` → `SetEnemySpawningOnHold(60)`. Returns the
/// new hold value when the skip fires, `None` when refused. The
/// `RequestToSkipWaitTime` gate (`:1779-1793`) adds the stand check and
/// the `NPCDeath7` cue; single-player collapses its netMode arms to this
/// call. The skip is refused at exactly 60 (strict `>`).
pub fn attempt_skip_wait_time(time_left: i32) -> Option<i32> {
    if time_left > 60 {
        Some(60)
    } else {
        None
    }
}

/// `DD2Event._crystalsDropping_lastWave/_toDrop/_alreadyDropped`
/// (`:79-83`) — the crystal-drop budget's state; `StartInvasion` zeroes
/// all three (`:257-259`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CrystalDropState {
    pub last_wave: i32,
    pub to_drop: i32,
    pub already_dropped: i32,
}

/// `DD2Event.ShouldDropCrystals` (`:815-912`) — the per-kill crystal
/// budget. Called from `NPC.checkDead`'s DD2 arms (`NPC.cs:79246`,
/// `:79274`): true → `Item.NewItem(source, Center, 3822)` (one crystal).
///
/// Order (the C# verbatim):
/// 1. `GetInvasionStatus` → `(current_wave, required, current_kills)`.
/// 2. While `_lastWave < currentWave`: `_lastWave++`, then subtract the
///    PREVIOUS wave's `_toDrop` from `_alreadyDropped` if positive
///    (`:821-824`), then the per-tier wave table re-sets `_toDrop`
///    (`:825-899`). The switch reads `currentWave` (the wave BEFORE the
///    increment) and has no default arm — an unlisted wave keeps the old
///    `_toDrop`.
/// 3. `num = currentKillCount / requiredKillCount` (f32 divide).
/// 4. `_alreadyDropped < _toDrop * num` → `_alreadyDropped++`, true.
///
/// Constants-off: the `Main.netMode != 0 && Main.expertMode` balance
/// rescale (`:901-904`) — single-player + no game mode, so it never fires
/// (recorded in ANALYSIS). Draws no RNG: the drop's rolls live in
/// `Item.NewItem` on the caller's stream.
pub fn should_drop_crystals(
    state: &mut CrystalDropState,
    difficulty: i32,
    current_wave: i32,
    required_kills: i32,
    current_kills: i32,
) -> bool {
    if state.last_wave < current_wave {
        state.last_wave += 1; // `:820`
        if state.already_dropped > 0 {
            state.already_dropped -= state.to_drop; // `:823` — the OLD budget
        }
        // `:825-899`: the three tier switches, no default arm.
        state.to_drop = match difficulty {
            1 => match current_wave {
                1 | 2 => 20,
                3 | 4 => 30,
                5 => 40,
                _ => state.to_drop,
            },
            2 => match current_wave {
                1..=5 => 20,
                6 | 7 => 30,
                _ => state.to_drop,
            },
            3 => match current_wave {
                1..=4 => 20,
                5..=7 => 30,
                _ => state.to_drop,
            },
            _ => state.to_drop,
        };
    }
    let num = current_kills as f32 / required_kills as f32; // `:905`
    if (state.already_dropped as f32) < (state.to_drop as f32) * num {
        state.already_dropped += 1; // `:908`
        return true;
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::UnifiedRandom;

    // --- medals -------------------------------------------------------

    #[test]
    fn the_victory_medals_are_3_15_60() {
        // `WinInvasionInternal` (`:328-340`).
        assert_eq!(victory_medals(1), Some(3));
        assert_eq!(victory_medals(2), Some(15));
        assert_eq!(victory_medals(3), Some(60));
        assert_eq!(victory_medals(0), None);
        assert_eq!(victory_medals(4), None);
    }

    #[test]
    fn the_victory_latches_every_tier_at_or_below() {
        // `:316-327`: `>= 1` / `>= 2` / `>= 3`, not `==`.
        assert_eq!(victory_downed_flags(1), (true, false, false));
        assert_eq!(victory_downed_flags(2), (true, true, false));
        assert_eq!(victory_downed_flags(3), (true, true, true));
        assert_eq!(victory_downed_flags(0), (false, false, false));
    }

    #[test]
    fn the_wave_complete_medal_table() {
        // `CheckProgress` (`:405-449`) — the NEW wave's rows.
        // Tier 1: waves 5 and 4 pay 1; nothing else.
        assert_eq!(wave_complete_medals(1, 4), Some(1));
        assert_eq!(wave_complete_medals(1, 5), Some(1));
        for w in [1, 2, 3, 6, 7] {
            assert_eq!(wave_complete_medals(1, w), None, "t1 wave {w}");
        }
        // Tier 2: 7→6, 6→3, 5→1.
        assert_eq!(wave_complete_medals(2, 7), Some(6));
        assert_eq!(wave_complete_medals(2, 6), Some(3));
        assert_eq!(wave_complete_medals(2, 5), Some(1));
        for w in [1, 2, 3, 4, 8] {
            assert_eq!(wave_complete_medals(2, w), None, "t2 wave {w}");
        }
        // Tier 3: 7→25, 6→11, 5→3, 4→1.
        assert_eq!(wave_complete_medals(3, 7), Some(25));
        assert_eq!(wave_complete_medals(3, 6), Some(11));
        assert_eq!(wave_complete_medals(3, 5), Some(3));
        assert_eq!(wave_complete_medals(3, 4), Some(1));
        for w in [1, 2, 3, 8] {
            assert_eq!(wave_complete_medals(3, w), None, "t3 wave {w}");
        }
    }

    // --- victory scene -------------------------------------------------

    #[test]
    fn the_victory_scene_flips_crystal_and_portals() {
        // `StartVictoryScene` (`:481-505`): won latches, the first 548
        // gets (2,2), every 549 gets (0,1).
        static PORTALS: &[usize] = &[3, 7];
        let plan = victory_scene(Some(5), PORTALS);
        assert!(plan.won);
        assert_eq!(plan.crystal_slot, Some(5));
        assert_eq!(plan.portal_slots, &[3, 7]);
    }

    #[test]
    fn no_crystal_means_no_portal_flips_but_won_still_latches() {
        // `:483` sets WonThisRun BEFORE the `:489-492` early return.
        static PORTALS: &[usize] = &[3, 7];
        let plan = victory_scene(None, PORTALS);
        assert!(plan.won);
        assert_eq!(plan.crystal_slot, None);
        assert!(plan.portal_slots.is_empty());
    }

    // --- wave-break skip ------------------------------------------------

    #[test]
    fn the_skip_clamps_the_hold_to_60_only_above_60() {
        // `AttemptToSkipWaitTime` (`:1795-1801`): strict `> 60`.
        assert_eq!(attempt_skip_wait_time(1800), Some(60));
        assert_eq!(attempt_skip_wait_time(61), Some(60));
        assert_eq!(attempt_skip_wait_time(60), None);
        assert_eq!(attempt_skip_wait_time(0), None);
        assert_eq!(attempt_skip_wait_time(-5), None);
    }

    #[test]
    fn the_stand_radius_is_36px_from_the_bottom() {
        // `IsStandActive` (`:1765-1777`): Bottom.Distance(target) < 36.
        // target for tile (10, 10) = (168, 168).
        assert!(is_stand_active((168.0, 168.0), (10, 10)));
        assert!(is_stand_active((168.0 + 35.9, 168.0), (10, 10)));
        assert!(
            !is_stand_active((168.0 + 36.0, 168.0), (10, 10)),
            "strict <"
        );
        assert!(!is_stand_active((168.0 + 30.0, 168.0 + 30.0), (10, 10)));
    }

    // --- crystal budget --------------------------------------------------

    #[test]
    fn the_crystal_budget_paces_drops_by_kill_progress() {
        // `ShouldDropCrystals` (`:815-912`), tier 1 wave 1: budget 20,
        // required 60. At 30 kills the pace is 20 * 0.5 = 10 drops.
        let mut s = CrystalDropState::default();
        let mut drops = 0;
        for kills in 0..=30 {
            if should_drop_crystals(&mut s, 1, 1, 60, kills) {
                drops += 1;
            }
        }
        assert_eq!(drops, 10, "20 * 30/60");
        assert_eq!(s.last_wave, 1);
        assert_eq!(s.to_drop, 20);
        assert_eq!(s.already_dropped, 10);
        // Past the pace: no more drops at the same progress.
        assert!(!should_drop_crystals(&mut s, 1, 1, 60, 30));
        assert_eq!(s.already_dropped, 10);
        // Full wave: the whole budget of 20 lands.
        let mut s = CrystalDropState::default();
        let mut drops = 0;
        for kills in 0..=60 {
            if should_drop_crystals(&mut s, 1, 1, 60, kills) {
                drops += 1;
            }
        }
        assert_eq!(drops, 20);
    }

    #[test]
    fn the_wave_advance_subtracts_the_old_budget_then_sets_the_new() {
        // `:818-844`: lastWave++ first, subtract the OLD toDrop while
        // alreadyDropped > 0, then the tier-1 table (wave 2 → 20).
        let mut s = CrystalDropState {
            last_wave: 1,
            to_drop: 20,
            already_dropped: 20,
        };
        // Wave 2 just started (0 kills): already 20 - 20 = 0, pace 0 → false.
        assert!(!should_drop_crystals(&mut s, 1, 2, 80, 0));
        assert_eq!(s.last_wave, 2);
        assert_eq!(s.to_drop, 20);
        assert_eq!(s.already_dropped, 0);
        // Mid-wave-2: 40/80 → pace 10.
        assert!(should_drop_crystals(&mut s, 1, 2, 80, 40));
        assert_eq!(s.already_dropped, 1);
        // Wave 3's budget is 30 (`:836`); the overshoot carries negative.
        s.already_dropped = 25;
        assert!(!should_drop_crystals(&mut s, 1, 3, 100, 0));
        assert_eq!(s.to_drop, 30);
        assert_eq!(s.already_dropped, 5, "25 - 20 (the OLD wave-2 budget)");
    }

    #[test]
    fn the_tier_tables_match_the_switches() {
        // `:825-899`: tier 1 = 20/20/30/30/40, tier 2 = 20×5 then 30/30,
        // tier 3 = 20×4 then 30/30/30. The `:818` guard is an `if`, not a
        // loop — lastWave advances ONE per call — so walk the waves in
        // order (0 kills → no drop, alreadyDropped stays 0).
        let t1 = [20, 20, 30, 30, 40];
        let t2 = [20, 20, 20, 20, 20, 30, 30];
        let t3 = [20, 20, 20, 20, 30, 30, 30];
        for (tier, table) in [(1, &t1[..]), (2, &t2[..]), (3, &t3[..])] {
            let mut s = CrystalDropState::default();
            for (i, &budget) in table.iter().enumerate() {
                let wave = i as i32 + 1;
                should_drop_crystals(&mut s, tier, wave, 100, 0);
                assert_eq!(s.to_drop, budget, "tier {tier} wave {wave}");
                assert_eq!(s.last_wave, wave);
            }
        }
    }

    #[test]
    fn an_unlisted_wave_keeps_the_old_budget() {
        // The switches have no default arm (`:827-898`): tier 1 wave 6
        // keeps wave 5's 40.
        let mut s = CrystalDropState {
            last_wave: 5,
            to_drop: 40,
            already_dropped: 0,
        };
        should_drop_crystals(&mut s, 1, 6, 1, 0);
        assert_eq!(s.to_drop, 40);
        assert_eq!(s.last_wave, 6);
    }

    #[test]
    fn the_zero_required_divide_matches_the_c_float_semantics() {
        // `:905-906`: required 0 → num = 0/0 = NaN (or x/0 = +inf); the
        // `<` compare is false for NaN and true for +inf with a positive
        // budget. Rust's f32 matches IEEE here — pin both.
        let mut s = CrystalDropState::default();
        assert!(!should_drop_crystals(&mut s, 1, 1, 0, 0), "0/0 = NaN");
        let mut s = CrystalDropState {
            last_wave: 1,
            to_drop: 20,
            already_dropped: 0,
        };
        assert!(should_drop_crystals(&mut s, 1, 1, 0, 5), "5/0 = +inf");
    }

    // --- no-double-with-loss witness ---------------------------------------

    #[test]
    fn the_won_gate_refuses_the_kill_progress_like_the_lost_gate() {
        // `CheckProgress`'s gate (`DD2Event.cs:364`) refuses on EITHER
        // decided flag: once `WonThisRun` latches (the win sentinel), no
        // further kill advances the wave — the same refusal `LostThisRun`
        // gets. Victory never double-fires with loss (S1-E-12c).
        let won = crate::dd2::check_progress(true, false, true, 0, 1, 59.0, 59.0, 1.0, false);
        assert!(won.is_none(), "the won gate refuses");
        let lost = crate::dd2::check_progress(true, true, false, 0, 1, 59.0, 59.0, 1.0, false);
        assert!(lost.is_none(), "the lost gate refuses");
        // Both decided: still refused (the win/loss are mutually exclusive
        // outcomes, but the gate reads them independently).
        let both = crate::dd2::check_progress(true, true, true, 0, 1, 59.0, 59.0, 1.0, false);
        assert!(both.is_none());
        // Undecided: the budget-passing kill advances.
        let open = crate::dd2::check_progress(true, false, false, 0, 1, 59.0, 59.0, 1.0, false);
        assert_eq!(open, Some(1));
    }

    // --- RNG witness ------------------------------------------------------

    #[test]
    fn the_victory_decisions_draw_no_rng() {
        // Witness: none of the decision functions takes or touches a
        // stream; the drops' rolls belong to Item.NewItem on the caller's
        // stream (world_sim/items.rs's pinned triplet). Run every decision
        // against a seeded stream and prove it stayed pristine.
        let mut rng = UnifiedRandom::new(4242);
        let expect = UnifiedRandom::new(4242);
        let _ = victory_medals(3);
        let _ = victory_downed_flags(3);
        let _ = wave_complete_medals(3, 7);
        let _ = victory_scene(Some(0), &[]);
        let _ = is_stand_active((0.0, 0.0), (0, 0));
        let _ = attempt_skip_wait_time(1800);
        let mut s = CrystalDropState::default();
        for kills in 0..=60 {
            let _ = should_drop_crystals(&mut s, 1, 1, 60, kills);
        }
        assert_eq!(rng, expect, "the decisions consumed no draws");
        // The stream itself still advances normally afterwards.
        let a = rng.next_max(100);
        let mut fresh = UnifiedRandom::new(4242);
        assert_eq!(a, fresh.next_max(100));
    }
}
