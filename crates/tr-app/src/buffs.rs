//! Player buff slots — the `buffType[44]` / `buffTime[44]` engine (`Player.AddBuff`
//! family, `Player.DelBuff`, the `UpdateBuffs` timer tick and the end-of-update
//! immune purge).
//!
//! Source map (all line numbers into `decompiled/Terraria/`):
//! - `Player.AddBuff` / helpers: `Player.cs:5263-5447`
//! - `Player.DelBuff`: `Player.cs:5451-5468`, `Player.ClearBuff`: `Player.cs:5474`
//! - `Player.FindBuffIndex`: `Player.cs:5250-5262`
//! - `UpdateBuffs` timer head: `Player.cs:9855-9876`
//! - end-of-update immune purge: `Player.cs:26012-26018`
//! - `Player.maxBuffs = 44`: `Player.cs:57106`; `BuffID.Count = 401`: `BuffID.cs:997`
//! - the `Main.debuff` / `lightPet` / `vanityPet` / `meleeBuff` / `pvpBuff` tables are
//!   extracted verbatim from `Main.cs` (`debuff[20]` at `Main.cs:8867`, `pvpBuff[20]`
//!   at `:8941`, `meleeBuff[71]` at `:8961`, `vanityPet[40]` at `:9214`, `lightPet[19]`
//!   at `:9288`).
//!
//! Per-buff *effects* (the long `UpdateBuffs` chain, `Player.cs:9875-12000`) are NOT
//! here — they land with M2-BUFF-3/4. This module is only the slot engine.

/// `Player.maxBuffs` (`Player.cs:57106`).
pub const MAX_BUFFS: usize = 44;
/// `BuffID.Count` (`BuffID.cs:997`) — the size of `Player.buffImmune`.
pub const BUFF_ID_COUNT: usize = 401;

/// `Main.debuff` — 76 entries, verbatim from `Main.cs:8867+`. Drives the
/// "first non-debuff slot" scan of `AddBuff_ActuallyTryToAddTheBuff`
/// (`Player.cs:5325-5337`); the empty slot id 0 is not in the set.
const DEBUFF_IDS: [i32; 76] = [
    20, 21, 22, 23, 24, 25, 28, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 43, 44, 46, 47, 67, 68,
    69, 70, 72, 80, 86, 87, 88, 89, 94, 103, 119, 120, 137, 144, 145, 146, 147, 148, 149, 153,
    156, 157, 158, 160, 163, 164, 169, 183, 186, 189, 194, 195, 196, 197, 199, 203, 204, 215,
    320, 321, 323, 324, 332, 333, 334, 344, 350, 353, 395, 397, 398, 399, 400,
];

/// `Main.lightPet` — 12 entries, verbatim from `Main.cs:9288+`.
const LIGHT_PET_IDS: [i32; 12] = [19, 27, 57, 101, 102, 152, 155, 190, 201, 294, 298, 299];

/// `Main.buffNoTimeDisplay` (`Main.cs:9074-9199`'s writes) — the duration
/// text hides for these (`Main.TryGetBuffTime`, `Main.cs:43290-43305`).
const BUFF_NO_TIME_DISPLAY: [i32; 124] = [
    19, 27, 28, 29, 34, 37, 38, 40, 41, 42, 43, 45,
    49, 60, 62, 64, 68, 81, 82, 83, 93, 95, 96, 97,
    98, 99, 100, 101, 102, 125, 126, 133, 134, 135, 136, 139,
    140, 150, 159, 161, 163, 165, 170, 171, 172, 182, 186, 187,
    188, 199, 200, 201, 202, 213, 214, 216, 217, 219, 258, 259,
    260, 261, 262, 263, 264, 266, 267, 268, 271, 274, 284, 285,
    286, 287, 288, 289, 290, 291, 292, 293, 294, 295, 296, 297,
    298, 299, 300, 301, 302, 303, 304, 312, 317, 322, 325, 327,
    328, 329, 330, 331, 334, 335, 341, 345, 348, 349, 351, 352,
    353, 354, 355, 356, 365, 366, 371, 372, 373, 382, 385, 386,
    389, 390, 393, 394,
];

/// `Main.buffNoTimeDisplay[b]` — see [`BUFF_NO_TIME_DISPLAY`].
pub fn is_no_time_display(buff_type: i32) -> bool {
    in_set(&BUFF_NO_TIME_DISPLAY, buff_type)
}

/// `BuffID.Sets.NurseCannotRemoveDebuff` (`BuffID.cs:52`): the debuffs the
/// Nurse refuses to remove — they neither clear at the heal nor count in
/// `GetNurseHealCost`'s price (`Main.cs:40061-40064`/`:40147`).
// Sorted for `in_set`'s binary search — the C# `CreateBoolSet` order is
// preserved in the comment below.
const NURSE_CANNOT_REMOVE_DEBUFF: [i32; 20] = [
    // CreateBoolSet(28, 34, 87, 89, 21, 86, 199, 332, 333, 334, 165, 146,
    //               48, 158, 157, 350, 215, 147, 321, 43)
    21, 28, 34, 43, 48, 86, 87, 89, 146, 147, 157, 158, 165, 199, 215, 321,
    332, 333, 334, 350,
];

/// `BuffID.Sets.NurseCannotRemoveDebuff[b]` — see [`NURSE_CANNOT_REMOVE_DEBUFF`].
pub fn nurse_cannot_remove(buff_type: i32) -> bool {
    in_set(&NURSE_CANNOT_REMOVE_DEBUFF, buff_type)
}

/// `Main.vanityPet` — 74 entries, verbatim from `Main.cs:9214+`.
const VANITY_PET_IDS: [i32; 74] = [
    40, 41, 42, 45, 50, 51, 52, 53, 54, 55, 56, 61, 65, 66, 81, 82, 84, 85, 91, 92, 127, 136,
    154, 191, 200, 202, 217, 218, 219, 258, 259, 260, 261, 262, 264, 266, 267, 268, 274, 284,
    285, 286, 287, 288, 289, 290, 291, 292, 293, 295, 296, 297, 300, 301, 302, 303, 304, 317,
    327, 328, 329, 330, 331, 341, 345, 349, 351, 352, 354, 356, 371, 372, 373, 382,
];

/// `Main.meleeBuff` — 8 entries, verbatim from `Main.cs:8961+` (the flask buffs).
const MELEE_BUFF_IDS: [i32; 8] = [71, 73, 74, 75, 76, 77, 78, 79];

/// `Main.pvpBuff` — 20 entries, verbatim from `Main.cs:8941+`. Only consulted by the
/// multiplayer branch of `AddBuff` (`Player.cs:5269-5275`), which the single-player
/// port does not reach yet; kept as data for M6-NET.
const PVP_BUFF_IDS: [i32; 20] = [
    20, 24, 30, 31, 36, 39, 44, 69, 70, 103, 119, 120, 137, 320, 323, 324, 397, 398, 399, 400,
];

fn in_set(set: &[i32], buff_type: i32) -> bool {
    buff_type > 0 && set.binary_search(&buff_type).is_ok()
}

pub fn is_debuff(buff_type: i32) -> bool {
    in_set(&DEBUFF_IDS, buff_type)
}

pub fn is_light_pet(buff_type: i32) -> bool {
    in_set(&LIGHT_PET_IDS, buff_type)
}

pub fn is_vanity_pet(buff_type: i32) -> bool {
    in_set(&VANITY_PET_IDS, buff_type)
}

pub fn is_melee_buff(buff_type: i32) -> bool {
    in_set(&MELEE_BUFF_IDS, buff_type)
}

#[allow(dead_code)]
pub fn is_pvp_buff(buff_type: i32) -> bool {
    in_set(&PVP_BUFF_IDS, buff_type)
}

/// `BuffID.Sets.IsFedState` (`BuffID.cs:44`).
fn is_fed_state(buff_type: i32) -> bool {
    matches!(buff_type, 26 | 206 | 207 | 332 | 333 | 334)
}

/// `BuffID.Sets.TimeLeftDoesNotDecrease` (`BuffID.cs:47`).
fn time_left_does_not_decrease(buff_type: i32) -> bool {
    matches!(buff_type, 28 | 334 | 29 | 159 | 150 | 93 | 348 | 366)
}

/// `BuffID.Sets.BuffTimeIsExtendedWithGameDifficulty` (`BuffID.cs:50`).
fn buff_time_is_extended_with_game_difficulty(buff_type: i32) -> bool {
    matches!(
        buff_type,
        20 | 22 | 23 | 24 | 323 | 30 | 31 | 32 | 33 | 35 | 36 | 39 | 44 | 324 | 46 | 47 | 69
            | 70 | 80
    )
}

/// `BuffID.Sets.BuffTimeIsExtendedByDeadCellsPotionStationBuff` (`BuffID.cs:51`).
fn buff_time_is_extended_by_dead_cells_potion_station(buff_type: i32) -> bool {
    (1..=18).contains(&buff_type)
        || (104..=117).contains(&buff_type)
        || matches!(buff_type, 121 | 122 | 123 | 124 | 257 | 343)
}

/// `BuffID.Sets.AddBuffTimeAdditivelyToCap` (`BuffID.cs:53`,
/// `CreateIntSet(0, 94, 600, 383, 43200)`): returns the additive cap, 0 = none.
fn add_buff_time_additively_to_cap(buff_type: i32) -> i32 {
    match buff_type {
        94 => 600,
        383 => 43_200,
        _ => 0,
    }
}

/// The player's 44 buff slots plus the `buffImmune` table.
///
/// `resist_cold` / `dead_cells_potion_station` / `expert_mode` mirror
/// `Player.resistCold`, `Player.deadCellsPotionStation` and `Main.expertMode`
/// (`Player.cs:5424-5447`). The port has no sources for them yet, so they stay
/// `false` until those systems land — the `AddBuff` chain already applies their
/// rules exactly as written.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuffSlots {
    /// `Player.buffType[44]`; 0 = empty slot.
    pub buff_type: [i32; MAX_BUFFS],
    /// `Player.buffTime[44]`, in ticks.
    pub buff_time: [i32; MAX_BUFFS],
    /// `Player.buffImmune[BuffID.Count]` (`Player.cs:1514`, `BuffID.cs:997`).
    pub buff_immune: [bool; BUFF_ID_COUNT],
    pub resist_cold: bool,
    pub dead_cells_potion_station: bool,
    pub expert_mode: bool,
}

impl Default for BuffSlots {
    fn default() -> Self {
        Self {
            buff_type: [0; MAX_BUFFS],
            buff_time: [0; MAX_BUFFS],
            buff_immune: [false; BUFF_ID_COUNT],
            resist_cold: false,
            dead_cells_potion_station: false,
            expert_mode: false,
        }
    }
}

impl BuffSlots {
    /// `Player.AddBuff` (`Player.cs:5263-5295`). The `Main.netMode == 1` remote
    /// branch (`:5269-5275`, msg 55 + `Main.pvpBuff`) is unreachable in the
    /// single-player port — deviation recorded in ANALYSIS "Buff slots (M2-BUFF-1)".
    pub fn add_buff(&mut self, buff_type: i32, time: i32) {
        if self.buff_immune[buff_type as usize] {
            return;
        }
        if is_fed_state(buff_type) {
            for i in 0..MAX_BUFFS {
                if is_fed_state(self.buff_type[i]) {
                    self.del_buff(i);
                }
            }
        }
        let time = self.add_buff_determine_buff_time_to_add(buff_type, time);
        if !self.add_buff_try_updating_existing_buff_time(buff_type, time) {
            self.add_buff_remove_old_pet_buffs_of_matching_type(buff_type);
            self.add_buff_remove_old_melee_buffs_of_matching_type(buff_type);
            self.add_buff_reduce_time_of_related_buffs(buff_type);
            self.add_buff_actually_try_to_add_the_buff(buff_type, time);
        }
    }

    /// `Player.AddBuff_DetermineBuffTimeToAdd` (`Player.cs:5424-5447`).
    fn add_buff_determine_buff_time_to_add(&self, buff_type: i32, time: i32) -> i32 {
        // C# `(int)((float)time * 1.2f)` truncates toward zero; `as i32` matches.
        if self.dead_cells_potion_station
            && buff_time_is_extended_by_dead_cells_potion_station(buff_type)
        {
            return (time as f32 * 1.2f32) as i32;
        }
        if self.expert_mode && buff_time_is_extended_with_game_difficulty(buff_type) {
            // `GameDifficultyData.DebuffTimeMultiplier.Sample(Main.Difficulty)`:
            // Classic 1 → Expert 2 → Master 2.5 (`GameDifficultyData.cs:96-100`).
            // The port has no difficulty modes yet, so expert is the only step.
            return (if self.expert_mode { 2.0f32 } else { 1.0f32 } * time as f32) as i32;
        }
        if self.resist_cold {
            match buff_type {
                46 => return time / 4,
                47 => return time / 2,
                _ => {}
            }
        }
        time
    }

    /// `Player.AddBuff_TryUpdatingExistingBuffTime` (`Player.cs:5399-5416`).
    fn add_buff_try_updating_existing_buff_time(&mut self, buff_type: i32, time: i32) -> bool {
        for i in 0..MAX_BUFFS {
            if self.buff_type[i] != buff_type {
                continue;
            }
            let cap = add_buff_time_additively_to_cap(buff_type);
            if cap > 0 {
                self.buff_time[i] += time;
                if self.buff_time[i] > cap {
                    self.buff_time[i] = cap;
                }
            } else if self.buff_time[i] < time {
                self.buff_time[i] = time;
            }
            return true;
        }
        false
    }

    /// `Player.AddBuff_RemoveOldPetBuffsOfMatchingType` (`Player.cs:5377-5397`).
    /// Unlike the melee loop below, the C# `for` here has **no** `i--`: after
    /// `DelBuff` compacts, the pet shifted into slot `i` is skipped.
    fn add_buff_remove_old_pet_buffs_of_matching_type(&mut self, buff_type: i32) {
        if is_light_pet(buff_type) {
            for i in 0..MAX_BUFFS {
                if is_light_pet(self.buff_type[i]) {
                    self.del_buff(i);
                }
            }
        }
        if !is_vanity_pet(buff_type) {
            return;
        }
        for j in 0..MAX_BUFFS {
            if is_vanity_pet(self.buff_type[j]) {
                self.del_buff(j);
            }
        }
    }

    /// `Player.AddBuff_RemoveOldMeleeBuffsOfMatchingType` (`Player.cs:5347-5360`).
    /// The C# `i--` after `DelBuff` rescans the index the compaction just filled.
    fn add_buff_remove_old_melee_buffs_of_matching_type(&mut self, buff_type: i32) {
        if !is_melee_buff(buff_type) {
            return;
        }
        let mut i = 0;
        while i < MAX_BUFFS {
            if self.buff_type[i] != buff_type && is_melee_buff(self.buff_type[i]) {
                self.del_buff(i);
            } else {
                i += 1;
            }
        }
    }

    /// `Player.AddBuff_ReduceTimeOfRelatedBuffs` (`Player.cs:5298-5317`): Warmth
    /// (124) shortens Chilled (46) to `max(5, t/4)` and Frozen (47) to
    /// `max(5, t/2)`, only when the time is above 5.
    fn add_buff_reduce_time_of_related_buffs(&mut self, buff_type: i32) {
        if buff_type != 124 {
            return;
        }
        for i in 0..MAX_BUFFS {
            let t = self.buff_time[i];
            if self.buff_type[i] == 46 && t > 5 {
                self.buff_time[i] = 5.max(t / 4);
            }
            if self.buff_type[i] == 47 && t > 5 {
                self.buff_time[i] = 5.max(t / 2);
            }
        }
    }

    /// `Player.AddBuff_ActuallyTryToAddTheBuff` (`Player.cs:5325-5345`): find the
    /// first non-debuff slot, then the first empty slot from there; when the tail
    /// has no empty slot, evict the first non-debuff buff and retry. A bar of 44
    /// debuffs rejects everything (the first scan finds no non-debuff slot).
    fn add_buff_actually_try_to_add_the_buff(&mut self, buff_type: i32, time: i32) -> bool {
        let mut num: i32 = -1;
        while num == -1 {
            let mut num2: i32 = -1;
            for i in 0..MAX_BUFFS {
                if !is_debuff(self.buff_type[i]) {
                    num2 = i as i32;
                    break;
                }
            }
            if num2 == -1 {
                return false;
            }
            for j in num2 as usize..MAX_BUFFS {
                if self.buff_type[j] == 0 {
                    num = j as i32;
                    break;
                }
            }
            if num == -1 {
                self.del_buff(num2 as usize);
            }
        }
        self.buff_type[num as usize] = buff_type;
        self.buff_time[num as usize] = time;
        true
    }

    /// `Player.DelBuff` (`Player.cs:5451-5468`). The compaction loop runs to
    /// `maxBuffs - 1`, so the last slot is never scanned: deleting from a
    /// completely full bar leaves a hole at 42 while slot 43 keeps its buff.
    /// Reproduced verbatim — it is observable through where `AddBuff` places the
    /// next buff.
    pub fn del_buff(&mut self, b: usize) {
        self.buff_time[b] = 0;
        self.buff_type[b] = 0;
        let mut num = 0usize;
        for i in 0..MAX_BUFFS - 1 {
            if self.buff_time[i] != 0 && self.buff_type[i] != 0 {
                if num < i {
                    self.buff_time[num] = self.buff_time[i];
                    self.buff_type[num] = self.buff_type[i];
                    self.buff_time[i] = 0;
                    self.buff_type[i] = 0;
                }
                num += 1;
            }
        }
    }

    /// `Player.ClearBuff` (`Player.cs:5470-5480`).
    pub fn clear_buff(&mut self, buff_type: i32) {
        for i in 0..MAX_BUFFS {
            if self.buff_type[i] == buff_type {
                self.del_buff(i);
            }
        }
    }

    /// `Player.FindBuffIndex` (`Player.cs:5250-5262`).
    pub fn find_buff_index(&self, buff_type: i32) -> Option<usize> {
        if buff_type <= 0 || buff_type as usize >= BUFF_ID_COUNT || self.buff_immune[buff_type as usize]
        {
            return None;
        }
        (0..MAX_BUFFS).find(|&i| self.buff_time[i] >= 1 && self.buff_type[i] == buff_type)
    }

    /// The timer head of `Player.UpdateBuffs` (`Player.cs:9869-9876`): expired and
    /// empty slots are skipped, `TimeLeftDoesNotDecrease` ids never tick. The
    /// `whoAmI == Main.myPlayer` guard always holds in the single-player port.
    /// The mount refresh (`buffTime = 10`, `:9881-9884`) and the per-buff effect
    /// chain (`:9875-12000`) land with M2-MOUNT / M2-BUFF-3/4.
    pub fn update_buffs_tick(&mut self) {
        for j in 0..MAX_BUFFS {
            if self.buff_type[j] <= 0 || self.buff_time[j] <= 0 {
                continue;
            }
            if !time_left_does_not_decrease(self.buff_type[j]) {
                self.buff_time[j] -= 1;
            }
        }
    }

    /// The per-frame `buffImmune` reset (`Player.cs:25562-25565`): every entry
    /// clears at the head of `Player.Update`, before `UpdateBuffs` re-grants this
    /// frame's immunities (Wet: 24/323/67, `Player.cs:11740-11751`) and long
    /// before the end-of-update purge consumes them.
    pub fn reset_buff_immune(&mut self) {
        self.buff_immune = [false; BUFF_ID_COUNT];
    }

    /// The end-of-update immune purge (`Player.cs:26012-26018`). Like the C#, the
    /// loop does not rescan after `DelBuff`: the buff compacted into slot `i` is
    /// skipped for this frame.
    pub fn purge_immune_buffs(&mut self) {
        for i in 0..MAX_BUFFS {
            let t = self.buff_type[i];
            if t > 0 && self.buff_time[i] > 0 && self.buff_immune[t as usize] {
                self.del_buff(i);
            }
        }
    }

    /// Types of the occupied slots, in slot order (HUD/test helper).
    pub fn active_types(&self) -> Vec<i32> {
        self.buff_type
            .iter()
            .copied()
            .filter(|&t| t > 0)
            .collect()
    }

    /// Number of occupied slots (HUD/test helper).
    pub fn active_count(&self) -> usize {
        self.buff_type.iter().filter(|&&t| t > 0).count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Valid buff ids that are in none of the exclusion sets, ascending. Tests must
    /// use real ids: `buffImmune[type]` is a `BuffID.Count`-sized array in the C# too
    /// (`BuffID.cs:997`), so an out-of-range id would throw there as well.
    fn safe_ids() -> Vec<i32> {
        (1..BUFF_ID_COUNT as i32)
            .filter(|&id| {
                !is_debuff(id)
                    && !is_light_pet(id)
                    && !is_vanity_pet(id)
                    && !is_melee_buff(id)
                    && !is_fed_state(id)
                    && add_buff_time_additively_to_cap(id) == 0
                    && id != 124 // Warmth rewrites the 46/47 times
            })
            .collect()
    }

    /// Fill the first `n` slots with distinct positive buff ids; returns them.
    fn fill(b: &mut BuffSlots, n: usize) -> Vec<i32> {
        let ids: Vec<i32> = safe_ids().into_iter().take(n).collect();
        for (i, &id) in ids.iter().enumerate() {
            b.buff_type[i] = id;
            b.buff_time[i] = 100;
        }
        ids
    }

    #[test]
    fn add_to_empty_occupies_first_slot() {
        let mut b = BuffSlots::default();
        b.add_buff(2, 60);
        assert_eq!(b.buff_type[0], 2);
        assert_eq!(b.buff_time[0], 60);
        assert_eq!(b.active_count(), 1);
    }

    #[test]
    fn re_add_takes_max_not_additive() {
        let mut b = BuffSlots::default();
        b.add_buff(2, 60);
        b.add_buff(2, 30); // existing 60 > 30 → unchanged
        assert_eq!(b.buff_time[0], 60);
        b.add_buff(2, 90); // 90 > 60 → raised
        assert_eq!(b.buff_time[0], 90);
    }

    #[test]
    fn additive_caps_94_and_383() {
        let mut b = BuffSlots::default();
        b.add_buff(94, 500); // Mana Sickness cap 600 (BuffID.cs:53)
        b.add_buff(94, 500);
        assert_eq!(b.buff_time[0], 600);
        b.add_buff(383, 43_000); // luck kite cap 43200
        b.add_buff(383, 500);
        assert_eq!(b.buff_time[1], 43_200);
    }

    #[test]
    fn fed_state_wipes_previous_fed_buffs() {
        let mut b = BuffSlots::default();
        b.add_buff(206, 1000); // WellFed2
        b.add_buff(2, 1000); // a positive buff that must survive the wipe
        b.add_buff(26, 100); // WellFed — all IsFedState wiped first (Player.cs:5277-5286)
        assert!(!b.active_types().contains(&206));
        assert!(b.active_types().contains(&2));
        assert!(b.active_types().contains(&26));
    }

    #[test]
    fn light_and_vanity_pets_are_exclusive() {
        let mut b = BuffSlots::default();
        b.add_buff(19, 1000); // light pet
        b.add_buff(57, 1000); // other light pet replaces it
        assert!(b.active_types().contains(&57));
        assert!(!b.active_types().contains(&19));
        b.add_buff(40, 1000); // vanity pet
        b.add_buff(42, 1000); // other vanity pet replaces it
        assert!(b.active_types().contains(&42));
        assert!(!b.active_types().contains(&40));
        assert!(b.active_types().contains(&57)); // light pet untouched
    }

    #[test]
    fn melee_buffs_are_exclusive_with_rescan() {
        let mut b = BuffSlots::default();
        b.add_buff(73, 1000);
        b.add_buff(2, 1000); // a positive buff survives melee exclusivity
        b.add_buff(74, 1000); // removes 73 (Player.cs:5347-5360)
        let types = b.active_types();
        assert!(types.contains(&74) && types.contains(&2) && !types.contains(&73));
        b.add_buff(71, 1000); // weapon-imbue flask removes 74
        let types = b.active_types();
        assert_eq!(types.len(), 2);
        assert!(types.contains(&71) && types.contains(&2));
    }

    #[test]
    fn warmth_reduces_chilled_and_frozen() {
        let mut b = BuffSlots::default();
        b.add_buff(46, 100);
        b.add_buff(47, 100);
        b.add_buff(124, 60); // Warmth (Player.cs:5298-5317)
        let chilled = b.find_buff_index(46).unwrap();
        let frozen = b.find_buff_index(47).unwrap();
        assert_eq!(b.buff_time[chilled], 25); // max(5, 100/4)
        assert_eq!(b.buff_time[frozen], 50); // max(5, 100/2)
        // A second Warmth while already active only refreshes its timer —
        // `try_updating_existing_buff_time` short-circuits before the reduction
        // (Player.cs:5290-5293).
        b.add_buff(124, 60);
        assert_eq!(b.buff_time[b.find_buff_index(46).unwrap()], 25);
        // Fresh Warmth on a small Chilled time clamps to the floor of 5.
        let mut c = BuffSlots::default();
        c.add_buff(46, 6);
        c.add_buff(124, 60);
        assert_eq!(c.buff_time[c.find_buff_index(46).unwrap()], 5); // max(5, 6/4=1)
    }

    #[test]
    fn resist_cold_shortens_chilled_and_frozen_on_add() {
        let mut b = BuffSlots::default();
        b.resist_cold = true;
        b.add_buff(46, 100);
        assert_eq!(b.buff_time[0], 25); // Player.cs:5438-5447
        b.add_buff(47, 100);
        assert_eq!(b.buff_time[1], 50);
        b.resist_cold = false;
        b.add_buff(2, 100); // unaffected types pass through
        assert_eq!(b.buff_time[2], 100);
    }

    #[test]
    fn dead_cells_station_extends_by_1_2_truncating() {
        let mut b = BuffSlots::default();
        b.dead_cells_potion_station = true;
        b.add_buff(1, 100); // in BuffTimeIsExtendedByDeadCellsPotionStationBuff
        assert_eq!(b.buff_time[0], 120);
        b.add_buff(7, 7); // 8.4 → 8 (C# (int) cast truncates)
        assert_eq!(b.buff_time[1], 8);
        b.add_buff(100, 100); // not in the set
        assert_eq!(b.buff_time[2], 100);
    }

    #[test]
    fn full_positive_bar_evicts_first_positive_into_the_quirk_hole() {
        let mut b = BuffSlots::default();
        let ids = fill(&mut b, MAX_BUFFS);
        let extra = safe_ids()[MAX_BUFFS]; // a 45th distinct positive id
        b.add_buff(extra, 100);
        // DelBuff(0) compacts 1..=42 → 0..=41 and leaves a hole at 42 (the
        // `i < maxBuffs - 1` bound, Player.cs:5457); slot 43 keeps its buff.
        // The next AddBuff scan finds that hole, so the new buff lands at 42.
        assert_eq!(b.active_count(), MAX_BUFFS);
        assert!(b.active_types().contains(&extra));
        assert_eq!(b.buff_type[42], extra);
        assert_eq!(b.buff_type[43], ids[43]); // slot 43 keeps its original buff
        assert_eq!(b.buff_type[0], ids[1]); // first positive evicted
    }

    #[test]
    fn full_debuff_bar_rejects_everything() {
        let mut b = BuffSlots::default();
        for (i, &id) in DEBUFF_IDS.iter().enumerate().take(MAX_BUFFS) {
            b.buff_type[i] = id;
            b.buff_time[i] = 100;
        }
        b.add_buff(400, 100); // debuff not already present — no slot to evict
        b.add_buff(2, 100); // positive — same rejection (Player.cs:5331-5334)
        assert_eq!(b.active_count(), MAX_BUFFS);
        assert!(!b.active_types().contains(&500));
    }

    #[test]
    fn mixed_bar_debuff_evicts_a_positive_buff() {
        let mut b = BuffSlots::default();
        for i in 0..40 {
            b.buff_type[i] = DEBUFF_IDS[i]; // 40 debuffs
            b.buff_time[i] = 100;
        }
        let safe = safe_ids();
        for i in 40..44 {
            b.buff_type[i] = safe[i - 40]; // 4 positives
            b.buff_time[i] = 100;
        }
        b.add_buff(400, 100); // debuff not yet present — needs a slot
        assert_eq!(b.active_count(), MAX_BUFFS);
        assert!(b.active_types().contains(&400));
        // The first non-debuff slot (40) was evicted to make room.
        assert!(!b.active_types().contains(&safe[0]));
    }

    #[test]
    fn del_buff_quirk_leaves_hole_at_42_when_full() {
        let mut b = BuffSlots::default();
        let ids = fill(&mut b, MAX_BUFFS);
        b.del_buff(0);
        assert_eq!(b.buff_type[0], ids[1]); // compacted left
        assert_eq!(b.buff_type[41], ids[42]); // last scanned slot lands at 41
        assert_eq!(b.buff_type[42], 0); // hole — slot 42 is never scanned
        assert_eq!(b.buff_type[43], ids[43]); // slot 43 untouched
    }

    #[test]
    fn tick_decrements_and_honors_time_left_does_not_decrease() {
        let mut b = BuffSlots::default();
        b.add_buff(2, 3);
        b.add_buff(28, 100); // TimeLeftDoesNotDecrease (BuffID.cs:47)
        b.add_buff(159, 100); // also frozen
        for _ in 0..5 {
            b.update_buffs_tick();
        }
        assert_eq!(b.buff_time[0], 0); // ran out, stays at 0 until removed
        assert_eq!(b.buff_time[1], 100);
        assert_eq!(b.buff_time[2], 100);
    }

    #[test]
    fn immune_blocks_add_and_purge_removes_active() {
        let mut b = BuffSlots::default();
        b.buff_immune[24] = true; // OnFire immunity (Obsidian Skin sets this)
        b.add_buff(24, 100); // blocked by the first guard (Player.cs:5265)
        assert_eq!(b.active_count(), 0);
        b.add_buff(20, 100);
        b.buff_immune[20] = true;
        b.purge_immune_buffs(); // Player.cs:26012-26018
        assert_eq!(b.active_count(), 0);
    }

    #[test]
    fn find_buff_index_and_clear_buff() {
        let mut b = BuffSlots::default();
        assert_eq!(b.find_buff_index(2), None);
        b.add_buff(2, 60);
        b.add_buff(3, 60);
        let idx = b.find_buff_index(3).unwrap();
        assert_eq!(b.buff_type[idx], 3);
        b.buff_immune[2] = true;
        assert_eq!(b.find_buff_index(2), None); // immune hides the buff (Player.cs:5252)
        b.clear_buff(3);
        assert!(!b.active_types().contains(&3));
        assert!(b.active_types().contains(&2));
    }

    #[test]
    fn pvp_and_melee_sets_are_consistent_with_the_c_sharp_tables() {
        // Spot-check the verbatim extraction against known anchors.
        assert!(is_debuff(20) && is_debuff(400));
        assert!(!is_debuff(0) && !is_debuff(1));
        assert!(is_light_pet(19) && !is_light_pet(40));
        assert!(is_vanity_pet(40) && !is_vanity_pet(19));
        assert!(is_melee_buff(71) && is_melee_buff(79) && !is_melee_buff(72));
        assert!(is_pvp_buff(20) && is_pvp_buff(400));
    }
}
