//! S1-E-7 wave F — the `Player.UpdateBuffs` FRAME (`Player.cs:9855-12069`):
//! the statements AROUND the arm chain that no arm-wave's range includes.
//!
//! Claim + live-set census: `docs/research/28f-wave-f-claim.md`. Every one
//! of the 254 `buffType[j]` arm heads in the region is claimed by the
//! landed waves A-D or the in-flight c17/c18/d71/d73/d74 claims — this
//! wave re-ports NO arm id. The genuinely unclaimed remainder is the frame:
//!
//! - the soulDrain head (`:9857-9860`): `AddBuff(151, 2)` while the
//!   Spirit Flame's drain counter runs;
//! - the DontStarve head (`:9861-9864`): `UpdateStarvingState(true)`
//!   (helper `:12152-12182`) — the 332 Well Fed re-grant when no fed-state
//!   buff is live;
//! - the per-tick `UpdateHungerBuffs()` tail (`:12046`, helper
//!   `:12098-12150`) — the 332→333→334 expiry ladder. The helper's slot
//!   logic is the keep-one survivor in `potions.rs` (the drink path's
//!   `try_to_reset_hunger_to_neutral` already calls it); this module adds
//!   only the missing PER-TICK call site + the emote/sound requests the
//!   potions port deferred;
//! - the luckPotion diff tail (`:12047-12051`): `luckNeedsSync` +
//!   `oldLuckPotion` bookkeeping;
//! - the lavaOpacity fade ladder (`:12052-12067`): the 0.04/tick fade
//!   between 0.4 (lavaVision + lavaWet) and 1.0.
//!
//! The loop preamble (`:9865-9874`, the slot skip + the
//! `TimeLeftDoesNotDecrease` decrement) is ALREADY live
//! (`buffs.rs::update_buffs_tick`) — witness-only here, not re-ported.
//!
//! Disjoint-wave design (the wave-B/C/D/E/G/H seam pattern): a PURE
//! accumulator — `run_buff_frame` folds one tick's frame writes into
//! `FrameEffects` + `FrameState`; the hunger slot writes run on
//! `BuffSlots` through the `potions.rs` keep-one helper. Wiring into the
//! live tick rides the waves' merge step — until then the module is
//! test-only: no runtime behaviour change, no RNG or fixed-step-order
//! change. `player.rs` is untouched.
//!
//! RNG census of the frame: ZERO direct `Main.rand` draws (`:9857-9864`,
//! `:12046-12067`, `:12098-12182` scanned). `AddBuff` draws no rand
//! (`buffs.rs::add_buff` is the verbatim port); the emote makes are
//! deterministic (`emotes.rs::new_bubble` mirrors `EmoteBubble.NewBubble`
//! `:116-131`) — recorded as REQUESTS; the `SoundID.Hungry` play
//! (`:12138`, `SoundID.cs:1711`'s trackable) rides the audio seam —
//! recorded as a REQUEST.

use crate::buffs::{BuffSlots, MAX_BUFFS};

/// `Player.lavaOpacity`'s initial value (`Player.cs:1534`: `= 1f`).
pub const LAVA_OPACITY_INITIAL: f32 = 1.0;
/// The fade floor while lavaVision + lavaWet (`Player.cs:12052-12057`).
pub const LAVA_OPACITY_FLOOR: f32 = 0.4;
/// The fade step per tick, both directions (`Player.cs:12054`/`:12062`).
pub const LAVA_OPACITY_STEP: f32 = 0.04;
/// The Well Fed grant while no fed-state buff is live
/// (`Player.cs:12171`/`:12175`): 18000 normally, 28800 under remix+dontStarve.
pub const STARVING_FED_TIME: i32 = 18_000;
pub const STARVING_FED_TIME_REMIX: i32 = 28_800;

/// The per-tick context the frame reads besides the buff slots.
/// `whoAmI == Main.myPlayer` collapses to true in the single-player port,
/// matching every other ported `UpdateBuffs` piece.
#[derive(Clone, Copy, Debug, Default)]
pub struct FrameContext {
    /// `Player.soulDrain` (`Player.cs:820`) — the Spirit Flame's drain
    /// counter; the head grants buff 151 while it is positive (`:9857`).
    pub soul_drain: i32,
    /// `Main.dontStarveWorld` — the seed flag (`SeedFlags::DONT_STARVE`).
    pub dont_starve: bool,
    /// `Main.remixWorld` — the seed flag (`SeedFlags::REMIX`).
    pub remix: bool,
    /// `Player.luckPotion` (`Player.cs:1134`) — the buff-257 tier the
    /// wave-A scan derived this tick (`player.rs:6684`).
    pub luck_potion: u8,
    /// `Player.lavaVision` — the Lava Shark mount / Obsidian Rose grant.
    pub lava_vision: bool,
    /// `Entity.lavaWet` — the player stands in lava this tick.
    pub lava_wet: bool,
}

/// The persistent frame fields the port lacks — the C# `Player` fields
/// `oldLuckPotion` (`Player.cs:1136`, a byte) and `lavaOpacity`
/// (`Player.cs:1534`, `= 1f`). Neither is cleared by `ResetEffects`; the
/// caller keeps one of these per player across ticks.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FrameState {
    /// `Player.oldLuckPotion` — the previous tick's tier the diff reads.
    pub old_luck_potion: u8,
    /// `Player.lavaOpacity` — the lava draw-opacity factor.
    pub lava_opacity: f32,
}

impl Default for FrameState {
    fn default() -> Self {
        Self {
            old_luck_potion: 0,
            lava_opacity: LAVA_OPACITY_INITIAL, // `:1534`
        }
    }
}

/// One tick's accumulated frame writes. The buff grants/zeroings are NOT
/// here — they run on `BuffSlots` (the C# writes the slots directly); the
/// emote/sound halves are REQUESTS the wiring drains onto the live
/// `EmoteSystem`/audio seams (the wave-C pet-spawn precedent).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FrameEffects {
    /// `Player.luckNeedsSync = true` (`:12049`) — the tier changed; the
    /// packet-134 send itself is net-only (`luck.rs`'s documented seam).
    pub luck_needs_sync: bool,
    /// `EmoteBubble.MakeLocalPlayerEmote(N)` requests in C# order:
    /// 146 (the starving re-grant, `:12179`), 147 (332 expiry, `:12118`),
    /// 148 (333 expiry, `:12137`).
    pub emotes: Vec<i32>,
    /// `SoundEngine.PlaySound(SoundID.Hungry)` (`:12138`) — one request
    /// per 333 expiry; the trackable id rides the audio seam.
    pub hungry_sounds: u32,
}

/// `BuffID.Sets.IsFedState` (`BuffID.cs:44`: 26, 206, 207, 332, 333, 334).
/// The `buffs.rs` private `is_fed_state` is the AddBuff-cleanup copy; this
/// is the UpdateStarvingState scan's read (kept local — no buffs.rs edit).
fn is_fed_state(buff_type: i32) -> bool {
    matches!(buff_type, 26 | 206 | 207 | 332 | 333 | 334)
}

/// `Player.UpdateStarvingState` (`Player.cs:12152-12182`), verbatim: when
/// NO live fed-state buff exists, grant Well Fed 332 (28800 under
/// remix+dontStarve `:12169-12171`, else 18000 `:12175`) and — with
/// `withEmote` — request emote 146 (`:12177-12180`). The myPlayer gate
/// (`:12154`) always holds in the single-player port.
pub fn update_starving_state(
    slots: &mut BuffSlots,
    e: &mut FrameEffects,
    dont_starve: bool,
    remix: bool,
    with_emote: bool,
) {
    let mut fed = false;
    for i in 0..MAX_BUFFS {
        // `:12161`: `buffTime[i] > 0 && IsFedState[buffType[i]]` — the C#
        // reads no `buffType > 0` here (0 is not fed-state either way).
        if slots.buff_time[i] > 0 && is_fed_state(slots.buff_type[i]) {
            fed = true;
            break;
        }
    }
    if !fed {
        let t = if remix && dont_starve {
            STARVING_FED_TIME_REMIX // `:12171`
        } else {
            STARVING_FED_TIME // `:12175`
        };
        slots.add_buff(332, t);
        if with_emote {
            e.emotes.push(146); // `:12179`
        }
    }
}

/// The per-tick `UpdateHungerBuffs()` tail (`Player.cs:12046`): the slot
/// ladder is the `potions.rs` keep-one helper (the drink path's
/// `try_to_reset_hunger_to_neutral` already calls it — no second copy);
/// this wrapper adds the emote/sound REQUESTS the potions port deferred
/// ("the emote/sound halves are UI/audio seams", `potions.rs:231-232`).
/// The requests mirror the helper's expiry gates exactly: a live 332 at
/// time <= 2 → emote 147 (`:12118`); a live 333 at time <= 2 → emote 148
/// + the Hungry sound (`:12137-12138`). The snapshot runs BEFORE the
/// ladder so the gates read the pre-expiry timers, like the C# loop.
pub fn update_hunger_buffs_frame(
    slots: &mut BuffSlots,
    e: &mut FrameEffects,
    dont_starve: bool,
    remix: bool,
) {
    for i in 0..MAX_BUFFS {
        let bt = slots.buff_type[i];
        let t = slots.buff_time[i];
        if bt <= 0 || t <= 0 {
            continue; // `:12102-12105`
        }
        if bt == 332 && t <= 2 {
            e.emotes.push(147); // `:12118` (myPlayer gate always true)
        } else if bt == 333 && t <= 2 {
            e.emotes.push(148); // `:12137`
            e.hungry_sounds += 1; // `:12138`
        }
    }
    crate::potions::update_hunger_buffs_slots(slots, dont_starve, remix);
}

/// The luckPotion diff tail (`Player.cs:12047-12051`): on a tier change,
/// raise `luckNeedsSync` and latch `oldLuckPotion`. The packet send is
/// net-only (`luck.rs:16-17`); the bookkeeping is the ported half.
pub fn luck_potion_diff_tail(ctx: &FrameContext, s: &mut FrameState, e: &mut FrameEffects) {
    if ctx.luck_potion != s.old_luck_potion {
        e.luck_needs_sync = true; // `:12049`
        s.old_luck_potion = ctx.luck_potion; // `:12050`
    }
}

/// The lavaOpacity fade ladder (`Player.cs:12052-12067`), verbatim: while
/// lavaVision AND lavaWet AND above the 0.4 floor, fade DOWN 0.04/tick
/// (clamped at 0.4); otherwise, while below 1.0, fade UP 0.04/tick
/// (clamped at 1.0). The f32 arithmetic keeps the C# float drift (0.04 is
/// not exact in binary — the clamp bounds make the drift observable only
/// mid-fade, exactly like the source).
pub fn lava_opacity_ladder(ctx: &FrameContext, s: &mut FrameState) {
    if ctx.lava_vision && ctx.lava_wet && s.lava_opacity > LAVA_OPACITY_FLOOR {
        s.lava_opacity -= LAVA_OPACITY_STEP; // `:12054`
        if s.lava_opacity < LAVA_OPACITY_FLOOR {
            s.lava_opacity = LAVA_OPACITY_FLOOR; // `:12055-12058`
        }
    } else if s.lava_opacity < 1.0 {
        s.lava_opacity += LAVA_OPACITY_STEP; // `:12062`
        if s.lava_opacity > 1.0 {
            s.lava_opacity = 1.0; // `:12063-12066`
        }
    }
}

/// One tick of the `UpdateBuffs` frame, in the C# statement order:
/// the soulDrain head (`:9857-9860`), the DontStarve head (`:9861-9864`),
/// [the arm chain — owned by waves A-H, NOT here], the per-tick hunger
/// tail (`:12046`), the luck diff (`:12047-12051`), the lava fade
/// (`:12052-12067`). The loop preamble's timer decrement
/// (`buffs.rs::update_buffs_tick`) runs BEFORE this in the wiring, like
/// the C# `for j` head precedes the tails.
pub fn run_buff_frame(ctx: &FrameContext, slots: &mut BuffSlots, s: &mut FrameState) -> FrameEffects {
    let mut e = FrameEffects::default();
    if ctx.soul_drain > 0 {
        slots.add_buff(151, 2); // `:9859`
    }
    if ctx.dont_starve {
        update_starving_state(slots, &mut e, ctx.dont_starve, ctx.remix, true); // `:9863`
    }
    update_hunger_buffs_frame(slots, &mut e, ctx.dont_starve, ctx.remix); // `:12046`
    luck_potion_diff_tail(ctx, s, &mut e); // `:12047-12051`
    lava_opacity_ladder(ctx, s); // `:12052-12067`
    e
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ctx() -> FrameContext {
        FrameContext::default()
    }

    // ---- the soulDrain head (:9857-9860) --------------------------------

    #[test]
    fn s1_e_7f_soul_drain_head_grants_151_for_two_ticks() {
        let mut c = ctx();
        c.soul_drain = 3;
        let mut slots = BuffSlots::default();
        let mut s = FrameState::default();
        let _ = run_buff_frame(&c, &mut slots, &mut s);
        let i = slots.find_buff_index(151).expect("the head grants 151");
        assert_eq!(slots.buff_time[i], 2, ":9859 AddBuff(151, 2)");
    }

    #[test]
    fn s1_e_7f_soul_drain_zero_grants_nothing() {
        let mut slots = BuffSlots::default();
        let mut s = FrameState::default();
        let _ = run_buff_frame(&ctx(), &mut slots, &mut s);
        assert!(slots.find_buff_index(151).is_none(), ":9857 gate is > 0");
    }

    #[test]
    fn s1_e_7f_soul_drain_refreshes_every_tick() {
        // The C# re-grants per tick while the counter runs; AddBuff's
        // existing-buff branch keeps the max (2), never stacks.
        let mut c = ctx();
        c.soul_drain = 1;
        let mut slots = BuffSlots::default();
        let mut s = FrameState::default();
        for _ in 0..5 {
            let _ = run_buff_frame(&c, &mut slots, &mut s);
        }
        let i = slots.find_buff_index(151).unwrap();
        assert_eq!(slots.buff_time[i], 2, "the 2-tick pin, not 10");
    }

    // ---- UpdateStarvingState (:12152-12182) ------------------------------

    #[test]
    fn s1_e_7f_starving_state_grants_well_fed_when_no_fed_buff() {
        let mut slots = BuffSlots::default();
        let mut e = FrameEffects::default();
        update_starving_state(&mut slots, &mut e, true, false, true);
        let i = slots.find_buff_index(332).expect("the 332 re-grant");
        assert_eq!(slots.buff_time[i], 18_000, ":12175 (no remix)");
        assert_eq!(e.emotes, vec![146], ":12179 withEmote");
    }

    #[test]
    fn s1_e_7f_starving_state_remix_row_grants_28800() {
        let mut slots = BuffSlots::default();
        let mut e = FrameEffects::default();
        update_starving_state(&mut slots, &mut e, true, true, true);
        let i = slots.find_buff_index(332).unwrap();
        assert_eq!(slots.buff_time[i], 28_800, ":12171 remix AND dontStarve");
    }

    #[test]
    fn s1_e_7f_starving_state_skips_while_any_fed_buff_lives() {
        // Every IsFedState id (BuffID.cs:44) blocks the re-grant.
        for fed in [26, 206, 207, 332, 333, 334] {
            let mut slots = BuffSlots::default();
            slots.add_buff(fed, 100);
            let mut e = FrameEffects::default();
            update_starving_state(&mut slots, &mut e, true, false, true);
            assert!(e.emotes.is_empty(), "fed {fed} blocks the emote");
            if fed != 332 {
                assert!(slots.find_buff_index(332).is_none(), "fed {fed} blocks 332");
            }
        }
    }

    #[test]
    fn s1_e_7f_starving_state_dead_fed_slot_does_not_block() {
        // :12161 reads buffTime > 0 — a zeroed fed slot is not live.
        let mut slots = BuffSlots::default();
        slots.add_buff(26, 100);
        let i = slots.find_buff_index(26).unwrap();
        slots.buff_time[i] = 0;
        let mut e = FrameEffects::default();
        update_starving_state(&mut slots, &mut e, true, false, true);
        assert!(slots.find_buff_index(332).is_some(), "the dead slot is skipped");
    }

    #[test]
    fn s1_e_7f_starving_state_without_emote_is_silent() {
        let mut slots = BuffSlots::default();
        let mut e = FrameEffects::default();
        update_starving_state(&mut slots, &mut e, true, false, false);
        assert!(slots.find_buff_index(332).is_some());
        assert!(e.emotes.is_empty(), "withEmote=false (:12177 gate)");
    }

    #[test]
    fn s1_e_7f_dont_starve_head_only_runs_under_the_seed() {
        // :9861 gates the whole head on Main.dontStarveWorld.
        let mut slots = BuffSlots::default();
        let mut s = FrameState::default();
        let _ = run_buff_frame(&ctx(), &mut slots, &mut s);
        assert!(slots.find_buff_index(332).is_none(), "no seed, no re-grant");
        let mut c = ctx();
        c.dont_starve = true;
        let _ = run_buff_frame(&c, &mut slots, &mut s);
        assert!(slots.find_buff_index(332).is_some(), "the seed arms the head");
    }

    // ---- the per-tick UpdateHungerBuffs tail (:12046 + :12098-12150) -----

    #[test]
    fn s1_e_7f_hunger_tail_expires_332_into_333_with_emote_147() {
        // The missing PER-TICK call site: a 332 at time 2 expires on its
        // own (no drink involved) — 333 granted at 18000, emote 147.
        let mut slots = BuffSlots::default();
        slots.add_buff(332, 2);
        let mut e = FrameEffects::default();
        update_hunger_buffs_frame(&mut slots, &mut e, false, false);
        let i = slots.find_buff_index(333).expect("the expiry ladder runs");
        assert_eq!(slots.buff_time[i], 18_000, ":12116");
        assert!(slots.find_buff_index(332).is_none(), "the fed cleanup took it");
        assert_eq!(e.emotes, vec![147], ":12118");
        assert_eq!(e.hungry_sounds, 0, "the sound is the 333 expiry only");
    }

    #[test]
    fn s1_e_7f_hunger_tail_expires_333_into_334_with_emote_148_and_sound() {
        let mut slots = BuffSlots::default();
        slots.add_buff(333, 2);
        let mut e = FrameEffects::default();
        update_hunger_buffs_frame(&mut slots, &mut e, false, false);
        let i = slots.find_buff_index(334).expect("333 → 334");
        assert_eq!(slots.buff_time[i], 5, ":12131/:12135 both arms 5");
        assert_eq!(e.emotes, vec![148], ":12137");
        assert_eq!(e.hungry_sounds, 1, ":12138 SoundID.Hungry");
    }

    #[test]
    fn s1_e_7f_hunger_tail_remix_row_and_dont_starve_hold() {
        // remix+dontStarve: the 332 expiry grants 28800 (:12112) and the
        // blind zeroing is gated off (:12120) — the fed cleanup still
        // deletes the 332 as the 333 lands (the potions.rs witness).
        let mut slots = BuffSlots::default();
        slots.add_buff(332, 2);
        let mut e = FrameEffects::default();
        update_hunger_buffs_frame(&mut slots, &mut e, true, true);
        let i = slots.find_buff_index(333).unwrap();
        assert_eq!(slots.buff_time[i], 28_800, ":12112");
        // Under dontStarve the 334 keeps its timer (:12145 gate).
        let mut q = BuffSlots::default();
        q.add_buff(334, 5);
        let mut e2 = FrameEffects::default();
        update_hunger_buffs_frame(&mut q, &mut e2, true, false);
        let j = q.find_buff_index(334).unwrap();
        assert_eq!(q.buff_time[j], 5, "dontStarve keeps the 334 alive");
    }

    #[test]
    fn s1_e_7f_hunger_tail_zeroes_334_outside_dont_starve() {
        let mut slots = BuffSlots::default();
        slots.add_buff(334, 5);
        let mut e = FrameEffects::default();
        update_hunger_buffs_frame(&mut slots, &mut e, false, false);
        assert!(slots.find_buff_index(334).is_none(), ":12147 zeroes it");
        assert!(e.emotes.is_empty(), "no emote on the 334 arm");
    }

    #[test]
    fn s1_e_7f_hunger_tail_long_timers_do_not_expire() {
        let mut slots = BuffSlots::default();
        slots.add_buff(332, 10_000);
        let mut e = FrameEffects::default();
        // `Player.cs:12108-12123`: the 332 row is dontStarve-world only —
        // outside it the `if (!Main.dontStarveWorld) buffTime[i] = 0` arm
        // zeroes even a 10 000-timer. The fixture arms the world that owns
        // the buff, so the long timer is the thing under test.
        update_hunger_buffs_frame(&mut slots, &mut e, true, false);
        let i = slots.find_buff_index(332).unwrap();
        assert_eq!(slots.buff_time[i], 10_000, "time > 2: no expiry");
        assert!(e.emotes.is_empty());
        assert!(e.hungry_sounds == 0);
    }

    // ---- the luckPotion diff tail (:12047-12051) -------------------------

    #[test]
    fn s1_e_7f_luck_diff_raises_sync_on_change_and_latches() {
        let mut c = ctx();
        let mut s = FrameState::default();
        let mut e = FrameEffects::default();
        luck_potion_diff_tail(&c, &mut s, &mut e);
        assert!(!e.luck_needs_sync, "0 == 0: no diff");
        c.luck_potion = 2;
        luck_potion_diff_tail(&c, &mut s, &mut e);
        assert!(e.luck_needs_sync, ":12049 the tier changed");
        assert_eq!(s.old_luck_potion, 2, ":12050 the latch");
        let mut e2 = FrameEffects::default();
        luck_potion_diff_tail(&c, &mut s, &mut e2);
        assert!(!e2.luck_needs_sync, "the latch holds: no second sync");
        c.luck_potion = 0; // expiry drops the tier back
        let mut e3 = FrameEffects::default();
        luck_potion_diff_tail(&c, &mut s, &mut e3);
        assert!(e3.luck_needs_sync, "the drop is a diff too");
        assert_eq!(s.old_luck_potion, 0);
    }

    // ---- the lavaOpacity ladder (:12052-12067) ---------------------------

    #[test]
    fn s1_e_7f_lava_opacity_fades_down_to_the_floor() {
        let mut c = ctx();
        c.lava_vision = true;
        c.lava_wet = true;
        let mut s = FrameState::default();
        assert_eq!(s.lava_opacity, 1.0, ":1534 the initial value");
        let mut min = 1.0f32;
        for _ in 0..100 {
            lava_opacity_ladder(&c, &mut s);
            min = min.min(s.lava_opacity);
        }
        // `Player.cs:12052-12066`: the down arm's clamp pins 0.4, but the
        // `else if (lavaOpacity < 1f)` up arm does NOT re-check the pair —
        // so at the floor the next frame rises one step. The ladder is a
        // 0.40 <-> 0.44 bounce; an even 100 down-frames ends on 0.44.
        assert_eq!(min, 0.4, "the :12057 clamp holds at 0.4");
        assert_eq!(s.lava_opacity, 0.44, "the next frame's unconditional up step");
    }

    #[test]
    fn s1_e_7f_lava_opacity_fades_back_up_without_the_pair() {
        let mut c = ctx();
        c.lava_vision = true;
        c.lava_wet = true;
        let mut s = FrameState { lava_opacity: 0.4, ..FrameState::default() };
        // Leaving the lava breaks the pair — the else arm fades up.
        c.lava_wet = false;
        for _ in 0..100 {
            lava_opacity_ladder(&c, &mut s);
        }
        assert_eq!(s.lava_opacity, 1.0, "the :12065 clamp holds at 1.0");
    }

    #[test]
    fn s1_e_7f_lava_opacity_one_step_is_004() {
        let mut c = ctx();
        c.lava_vision = true;
        c.lava_wet = true;
        let mut s = FrameState::default();
        lava_opacity_ladder(&c, &mut s);
        assert_eq!(s.lava_opacity, 1.0f32 - 0.04f32, ":12054 one step");
        let mut up = FrameState { lava_opacity: 0.5, ..FrameState::default() };
        lava_opacity_ladder(&ctx(), &mut up);
        assert_eq!(up.lava_opacity, 0.5f32 + 0.04f32, ":12062 one step up");
    }

    #[test]
    fn s1_e_7f_lava_opacity_at_floor_with_pair_is_stable() {
        // `Player.cs:12052-12066`: the down arm needs `> 0.4f`; at exactly
        // 0.4 its else-if (`lavaOpacity < 1f`, with NO pair re-check) runs
        // the up arm, so the floor steps to 0.44 — the source's one-frame
        // bounce, not a fixed point.
        let mut c = ctx();
        c.lava_vision = true;
        c.lava_wet = true;
        let mut s = FrameState { lava_opacity: 0.4, ..FrameState::default() };
        lava_opacity_ladder(&c, &mut s);
        assert_eq!(s.lava_opacity, 0.44, "the source's unconditional up step");
    }

    // ---- frame order + the already-live preamble -------------------------

    #[test]
    fn s1_e_7f_frame_order_head_before_hunger_tail() {
        // The DontStarve head grants 332 at 18000; the SAME tick's hunger
        // tail must see it as long-lived (no expiry, no emote 147) — the
        // C# statement order (:9863 before :12046).
        let mut c = ctx();
        c.dont_starve = true;
        let mut slots = BuffSlots::default();
        let mut s = FrameState::default();
        let e = run_buff_frame(&c, &mut slots, &mut s);
        let i = slots.find_buff_index(332).unwrap();
        assert_eq!(slots.buff_time[i], 18_000);
        assert_eq!(e.emotes, vec![146], "only the starving emote, not 147");
    }

    #[test]
    fn s1_e_7f_preamble_decrement_is_the_live_buffs_tick() {
        // Witness-only: the loop preamble (:9865-9874) is buffs.rs's
        // update_buffs_tick — 151 ticks down, the TimeLeftDoesNotDecrease
        // ids (28/334/29/159/150/93/348/366, BuffID.cs:47) never do.
        let mut slots = BuffSlots::default();
        slots.add_buff(151, 2);
        slots.add_buff(334, 5);
        slots.update_buffs_tick();
        let i = slots.find_buff_index(151).unwrap();
        assert_eq!(slots.buff_time[i], 1, ":9873 the decrement");
        let j = slots.find_buff_index(334).unwrap();
        assert_eq!(slots.buff_time[j], 5, ":9871 the never-tick set");
    }

    #[test]
    fn s1_e_7f_the_frame_draws_no_rng() {
        // RNG census witness: the whole frame runs WITHOUT a random
        // source — every grant is a deterministic AddBuff, every fade is
        // float arithmetic. Mirror stream: two identical runs produce
        // identical slots/state/effects (no hidden draw exists to differ).
        let mut c = ctx();
        c.soul_drain = 2;
        c.dont_starve = true;
        c.remix = true;
        c.lava_vision = true;
        c.lava_wet = true;
        c.luck_potion = 3;
        let run = || {
            let mut slots = BuffSlots::default();
            slots.add_buff(333, 2);
            let mut s = FrameState::default();
            let e = run_buff_frame(&c, &mut slots, &mut s);
            (slots, s, e)
        };
        assert_eq!(run(), run(), "the frame is a pure deterministic fold");
    }
}