//! The tag-damage system — `Player.TagEffectStack` and its per-NPC state
//! (`Terraria/GameContent/Items/TagEffectStack.cs`, `TagEffectState.cs`), the
//! tagged-hit math (`Projectile.cs:12919-12933`, `TagDamageChanges.cs`) and
//! the vanilla tag table's consumers (`ItemID.cs:1473-1580`).
//!
//! The marks ride the *player*, like the C#: a whip hit tags the NPC in the
//! player's stack, and the summon/sentry shots that hit a tagged NPC take
//! the bonus. The per-NPC arrays are index-keyed (`Main.maxNPCs`), so the
//! port sizes them to the pool's NPC index space (256, like the pool's
//! `localNPCImmunity` rows).
//!
//! Out of scope until M4-PROJ-8: the `minion`/`sentry` shot kinds the hit
//! gate admits (the gate rejects every ported shot until then — the synthetic
//! tests set the flag directly), and the tag effects' projectile spawns
//! (the DarkHarvest's 916, the Firecracker's 918, the Starcrash's 1037, the
//! FlowerWhip's 1038, the Constellation's 1039 fan, the Eel's chain damage).
//! The pure math and the tables are `tr_core::whip`'s.

use tr_core::loot::DropRng;

/// `Main.maxNPCs`' index space as the pool's local immunity uses it
/// (`Projectile.cs`'s `localNPCImmunity = new int[Main.maxNPCs]`).
pub const NPC_INDEX_MAX: usize = 256;
/// `TagEffectStack.MaxEffects` (`TagEffectStack.cs:142`).
pub const MAX_EFFECTS: usize = 5;
/// `Player.maxTagEffects`'s default (`Player.cs:18517`, re-set by the
/// accessories in `ResetEffects` — M3-EQUIP).
pub const MAX_TAG_EFFECTS_DEFAULT: usize = 1;
/// `Player.tagEffectDuration`'s default (`Player.cs:18518`).
pub const TAG_EFFECT_DURATION_DEFAULT: f32 = 1.0;

/// One active tag effect — `TagEffectState.cs`. The two int arrays are the
/// per-NPC mark and proc windows, both `(int)(TagDuration ·
/// tagEffectDuration)` ticks (`:64`, `:70`).
#[derive(Clone, Debug)]
struct TagEffectState {
    effect: &'static tr_core::whip::TagEffect,
    /// `TimeLeftOnNPC` — the tag mark's remaining ticks.
    time_left: [i32; NPC_INDEX_MAX],
    /// `ProcTimeLeftOnNPC` — the proc window's remaining ticks.
    proc_left: [i32; NPC_INDEX_MAX],
}

impl TagEffectState {
    fn new() -> Self {
        Self {
            // The plain base effect — a never-tagged placeholder (`Activate`
            // replaces it before any use).
            effect: tr_core::whip::tag_effect(0).unwrap_or_else(|| {
                // A synthetic empty row (the vanilla table has no id-0 entry).
                tr_core::whip::tag_effect(4672).unwrap()
            }),
            time_left: [0; NPC_INDEX_MAX],
            proc_left: [0; NPC_INDEX_MAX],
        }
    }

    fn reset_npc_slot(&mut self, npc_index: usize) {
        if npc_index < NPC_INDEX_MAX {
            self.time_left[npc_index] = 0;
            self.proc_left[npc_index] = 0;
        }
    }
}

/// `Player.TagEffectStack` (`TagEffectStack.cs`) — up to 5 active effects,
/// the newest first, capped by the player's `maxTagEffects`.
#[derive(Clone, Debug)]
pub struct TagEffectStack {
    /// `_effectStates` — the 5 slots, `_activeEffectCount` of them live.
    states: [Option<TagEffectState>; MAX_EFFECTS],
    active: usize,
    /// `Player.maxTagEffects` (`Player.cs:1204`; the default 1).
    max_tag_effects: usize,
    /// `Player.tagEffectDuration` (`Player.cs:1206`; the default 1f).
    tag_effect_duration: f32,
}

impl Default for TagEffectStack {
    fn default() -> Self {
        Self {
            states: Default::default(),
            active: 0,
            max_tag_effects: MAX_TAG_EFFECTS_DEFAULT,
            tag_effect_duration: TAG_EFFECT_DURATION_DEFAULT,
        }
    }
}

impl TagEffectStack {
    /// The index of `tag_type` in the active stack, or −1 (`IndexOf`).
    fn index_of(&self, tag_type: u16) -> Option<usize> {
        (0..self.active).find(|&i| {
            self.states[i]
                .as_ref()
                .is_some_and(|s| s.effect.item_id == tag_type)
        })
    }

    /// `LimitActiveEffects` (`:33-40`): the tail (oldest) effects deactivate
    /// until `active <= limit`.
    fn limit_active(&mut self, limit: usize) {
        let limit = limit.min(MAX_EFFECTS);
        while self.active > limit {
            self.active -= 1;
            self.states[self.active] = None;
        }
    }

    /// `TrySetActiveEffect` (`:42-58`): a known effect moves to the front; a
    /// new one evicts the oldest when the player's cap is reached and takes
    /// the front slot.
    pub fn try_set_active(&mut self, tag_type: u16) {
        let Some(effect) = tr_core::whip::tag_effect(tag_type) else {
            return;
        };
        let existing = self.index_of(tag_type);
        let num = match existing {
            Some(i) => i,
            None => {
                self.limit_active(self.max_tag_effects.saturating_sub(1));
                let num = self.active;
                let mut state = TagEffectState::new();
                state.effect = effect;
                self.states[num] = Some(state);
                self.active += 1;
                num
            }
        };
        // The swap loop moves the effect to the front (`:54-57`).
        let mut num = num;
        while num > 0 {
            self.states.swap(num, num - 1);
            num -= 1;
        }
    }

    /// `ApplyTagToNPC` — the mark's duration (`TagEffectState.cs:62-66`).
    fn mark_duration(&self) -> i32 {
        (self.duration_of_front() as f32 * self.tag_effect_duration) as i32
    }

    /// The front effect's `TagDuration` (`WhipTagEffect.cs:17-21`'s field,
    /// overridden by the Constellation's 480).
    fn duration_of_front(&self) -> i32 {
        self.states[0]
            .as_ref()
            .map(|s| s.effect.tag_duration)
            .unwrap_or(tr_core::whip::TAG_DURATION_GENERIC)
    }

    /// `TryApplyTagToNPC` (`:80-87`): the whip-immunity gate, the activation
    /// and the mark write.
    pub fn try_apply_tag_to_npc(
        &mut self,
        tag_type: u16,
        npc_index: usize,
        npc_kind: crate::npc::NpcKind,
    ) {
        if tr_core::whip::tag_effect(tag_type).is_none() {
            return;
        }
        // `NPCDebuffImmunityData`'s `ImmuneToWhips` rows
        // (`GameContent/Items/WhipTagEffect.cs:24-35`, `NPCID.cs`).
        if crate::npc_buffs::whip_immune(npc_kind) {
            return;
        }
        self.try_set_active(tag_type);
        self.time_left_write(0, npc_index, self.mark_duration());
    }

    /// `IsNPCTagged` (`TagEffectState.cs:23-26` via `TagEffectStack.cs:60-68`).
    pub fn is_npc_tagged(&self, tag_type: u16, npc_index: usize) -> bool {
        match self.index_of(tag_type) {
            Some(i) => self.states[i]
                .as_ref()
                .is_some_and(|s| npc_index < NPC_INDEX_MAX && s.time_left[npc_index] > 0),
            None => false,
        }
    }

    /// `CanProcOnNPC` (`:70-78`).
    pub fn can_proc(&self, tag_type: u16, npc_index: usize) -> bool {
        match self.index_of(tag_type) {
            Some(i) => self.states[i]
                .as_ref()
                .is_some_and(|s| npc_index < NPC_INDEX_MAX && s.proc_left[npc_index] > 0),
            None => false,
        }
    }

    /// `TryEnableProcOnNPC` (`:89-96`).
    pub fn try_enable_proc(&mut self, tag_type: u16, npc_index: usize) {
        if let Some(i) = self.index_of(tag_type) {
            let duration = self.duration_of_front();
            self.proc_left_write(
                i,
                npc_index,
                (duration as f32 * self.tag_effect_duration) as i32,
            );
        }
    }

    /// `RemoveAllProcs` (`:98-105`).
    pub fn remove_all_procs(&mut self, tag_type: u16) {
        if let Some(i) = self.index_of(tag_type) {
            if let Some(s) = self.states[i].as_mut() {
                s.proc_left = [0; NPC_INDEX_MAX];
            }
        }
    }

    /// `Player.ResetNPCSlotData` (`Player.cs:44208`) — the spawn-time clear.
    pub fn reset_npc_slot(&mut self, npc_index: usize) {
        for s in self.states[..self.active].iter_mut().flatten() {
            s.reset_npc_slot(npc_index);
        }
    }

    /// `TagEffectStack.Update` (`:131-138`): the cap and every state's
    /// per-NPC timers decay once per player tick.
    pub fn update_tick(&mut self) {
        self.limit_active(self.max_tag_effects);
        for s in self.states[..self.active].iter_mut().flatten() {
            for v in s.time_left.iter_mut() {
                if *v > 0 {
                    *v -= 1;
                }
            }
            for v in s.proc_left.iter_mut() {
                if *v > 0 {
                    *v -= 1;
                }
            }
        }
    }

    /// `TagEffectStack.ModifyHit` (`:107-113` → `TagEffectState.cs:97-107`):
    /// the tagged-hit arm and, while the proc window stands, the proc arm.
    /// Only summon/sentry shots pass `can_run_hit_effects`
    /// (`GameContent/Items/WhipTagEffect.cs:75-86`) — the caller checks it.
    pub fn modify_hit(
        &self,
        npc_index: usize,
        tag_type: u16,
        changes: &mut tr_core::whip::TagDamageChanges,
        rng: &mut dyn DropRng,
    ) {
        let Some(i) = self.index_of(tag_type) else {
            return;
        };
        let Some(state) = self.states[i].as_ref() else {
            return;
        };
        let tagged = npc_index < NPC_INDEX_MAX && state.time_left[npc_index] > 0;
        let proc_armed = npc_index < NPC_INDEX_MAX && state.proc_left[npc_index] > 0;
        if tagged {
            tr_core::whip::modify_tagged_hit(state.effect, changes, rng);
        }
        if tagged && proc_armed {
            // The proc arm's pure arms (the SwordWhip's flat, the
            // Firecracker's multiplier, the Eel's bolt count).
            tr_core::whip::modify_proc_hit(state.effect, changes, 0);
        }
    }

    /// `TagEffectStack.OnHit` (`:115-121` → `TagEffectState.cs:109-119`):
    /// the `OnTaggedHit` bodies are the deferred spawns; the proc arm's
    /// consume decision lands here (`ClearProcOnNPC` `:116`).
    pub fn on_hit(&mut self, npc_index: usize, tag_type: u16, proj_kind: u16) {
        let Some(i) = self.index_of(tag_type) else {
            return;
        };
        let Some(state) = self.states[i].as_ref() else {
            return;
        };
        let tagged = npc_index < NPC_INDEX_MAX && state.time_left[npc_index] > 0;
        let proc_armed = npc_index < NPC_INDEX_MAX && state.proc_left[npc_index] > 0;
        if tagged && proc_armed && tr_core::whip::proc_hit_consumes(state.effect, proj_kind) {
            if let Some(s) = self.states[i].as_mut() {
                s.proc_left[npc_index] = 0;
            }
        }
    }

    /// The active (front-first) tag types, for the debug panel.
    pub fn active_types(&self) -> Vec<u16> {
        self.states[..self.active]
            .iter()
            .flatten()
            .map(|s| s.effect.item_id)
            .collect()
    }

    // --- the array write helpers (the front-slot borrow ends first) ---------

    fn time_left_write(&mut self, state: usize, npc_index: usize, value: i32) {
        if let Some(s) = self.states[state].as_mut() {
            if npc_index < NPC_INDEX_MAX {
                s.time_left[npc_index] = value;
            }
        }
    }

    fn proc_left_write(&mut self, state: usize, npc_index: usize, value: i32) {
        if let Some(s) = self.states[state].as_mut() {
            if npc_index < NPC_INDEX_MAX {
                s.proc_left[npc_index] = value;
            }
        }
    }
}

/// The `Projectile.cs:12924` line, applied to one hit: the tag's flat add
/// through the type's `SummonTagDamageMultiplier`, the extra flat, then the
/// total multiplier. `crit` comes back `Some` when the tag forced a crit
/// (`WhipTagEffect.cs:69-72`).
pub fn apply_tag_damage(
    base: i32,
    changes: &tr_core::whip::TagDamageChanges,
    bonus_tag_damage: i32,
    proj_kind: u16,
) -> i32 {
    if !changes.add_projectile_tag_damage {
        return base;
    }
    let inner = (changes.highest_added_base_damage as f32
        + (changes.added_base_damage - changes.highest_added_base_damage) as f32 * 1.0
        + bonus_tag_damage as f32)
        * tr_core::whip::summon_tag_damage_multiplier(proj_kind);
    ((base + inner as i32 + changes.added_flat_damage) as f32 * changes.total_damage_multiplier)
        as i32
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::npc::NpcKind;

    fn stack() -> TagEffectStack {
        TagEffectStack::default()
    }

    #[test]
    fn the_tag_mark_runs_the_general_240_and_expires() {
        let mut s = stack();
        s.try_apply_tag_to_npc(4672, 3, NpcKind::Zombie);
        assert!(s.is_npc_tagged(4672, 3), "the mark lands");
        // 240 ticks at the 1.0 default duration.
        for _ in 0..239 {
            s.update_tick();
            assert!(s.is_npc_tagged(4672, 3), "still marked at {:?}", 239);
        }
        s.update_tick();
        assert!(!s.is_npc_tagged(4672, 3), "the mark expires at 240");
    }

    #[test]
    fn the_constellation_row_runs_480() {
        let mut s = stack();
        s.try_apply_tag_to_npc(5479, 0, NpcKind::Zombie);
        for _ in 0..479 {
            s.update_tick();
        }
        assert!(s.is_npc_tagged(5479, 0), "the 480 mark holds");
        s.update_tick();
        assert!(!s.is_npc_tagged(5479, 0), "and expires at 480");
    }

    #[test]
    fn the_eviction_keeps_the_newest_and_caps_at_five() {
        let mut s = stack();
        // The default `maxTagEffects` is 1: each new tag evicts the previous.
        s.try_apply_tag_to_npc(4672, 0, NpcKind::Zombie);
        s.try_apply_tag_to_npc(5074, 0, NpcKind::Zombie);
        assert_eq!(s.active_types(), vec![5074], "the newest is front");
        // Raise the cap (the accessory arm, M3-EQUIP) to see the 5-slot cap.
        s.max_tag_effects = MAX_EFFECTS;
        for item in [5688u16, 5473, 5474, 5475, 5476, 4913, 5074] {
            s.try_apply_tag_to_npc(item, 0, NpcKind::Zombie);
        }
        assert_eq!(s.active_types().len(), MAX_EFFECTS, "the 5-slot cap");
        // The newest stays front.
        assert_eq!(s.active_types()[0], 5074);
    }

    #[test]
    fn the_whip_immune_kinds_refuse_the_tag() {
        let mut s = stack();
        // `NPCID.cs`'s 11 `ImmuneToWhips` rows; the ported roster has 392/491.
        s.try_apply_tag_to_npc(4672, 0, NpcKind::MartianSaucer);
        assert!(!s.is_npc_tagged(4672, 0), "the saucer is whip-immune");
        s.try_apply_tag_to_npc(4672, 1, NpcKind::FlyingDutchman);
        assert!(!s.is_npc_tagged(4672, 1), "the pirate ship too");
        s.try_apply_tag_to_npc(4672, 2, NpcKind::Zombie);
        assert!(s.is_npc_tagged(4672, 2), "the rest take the tag");
    }

    #[test]
    fn the_spawn_reset_clears_the_marks() {
        let mut s = stack();
        s.try_apply_tag_to_npc(4672, 5, NpcKind::Zombie);
        s.try_enable_proc(4672, 5);
        assert!(s.can_proc(4672, 5));
        // `Player.ResetNPCSlotData` — an NPC respawned into index 5.
        s.reset_npc_slot(5);
        assert!(!s.is_npc_tagged(4672, 5));
        assert!(!s.can_proc(4672, 5));
    }

    #[test]
    fn the_tagged_shot_takes_the_bonus_and_the_proc_arms() {
        let mut s = stack();
        s.try_apply_tag_to_npc(4672, 7, NpcKind::Zombie);
        let mut changes = tr_core::whip::TagDamageChanges::none();
        s.modify_hit(7, 4672, &mut changes, &mut FixedRoll(50));
        // The BlandWhip's TagDamage 4, the highest tracking, no crit at 50.
        assert!(changes.add_projectile_tag_damage);
        assert_eq!(changes.added_base_damage, 4);
        assert_eq!(changes.highest_added_base_damage, 4);
        assert_eq!(apply_tag_damage(100, &changes, 0, 1), 104);
        // An untagged NPC takes nothing.
        let mut none = tr_core::whip::TagDamageChanges::none();
        s.modify_hit(8, 4672, &mut none, &mut FixedRoll(50));
        assert!(!none.add_projectile_tag_damage);
        assert_eq!(apply_tag_damage(100, &none, 0, 1), 100);
        // The proc window: the first-hit latches (`:13907`), the proc arm
        // adds on top, and the consuming classes clear the window.
        s.try_enable_proc(4672, 7);
        assert!(s.can_proc(4672, 7));
        s.on_hit(7, 4672, 1);
        // The plain whips' `OnProcHit` returns false — the window stands.
        assert!(s.can_proc(4672, 7), "the plain whip keeps the window");
        // The Firecracker's proc consumes.
        let mut fc = stack();
        fc.try_apply_tag_to_npc(4912, 7, NpcKind::Zombie);
        fc.try_enable_proc(4912, 7);
        let mut fc_changes = tr_core::whip::TagDamageChanges::none();
        fc.modify_hit(7, 4912, &mut fc_changes, &mut FixedRoll(50));
        assert_eq!(fc_changes.total_damage_multiplier, 2.75);
        fc.on_hit(7, 4912, 1);
        assert!(!fc.can_proc(4912, 7), "the Firecracker consumes");
    }

    struct FixedRoll(i32);
    impl DropRng for FixedRoll {
        fn next_range(&mut self, _min: i32, _max: i32) -> i32 {
            self.0
        }
        fn next_float(&mut self) -> f32 {
            0.5
        }
    }
}
