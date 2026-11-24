//! Potion USE machinery (S1-P-i37) — the `Player.ItemCheck` potion arm
//! (`Player.cs:42756-42758` + `:42867-42883`), `ApplyPotionDelay`
//! (`Player.cs:51853-51886`), `ApplyLifeAndOrMana` (`Player.cs:51888-51952`),
//! the buff-potion grant (`Player.cs:42876-42883`), the generic consumable
//! tail (`Player.cs:43280-43326`), the quick-use family (`QuickHeal`
//! `Player.cs:5501-5596`, `QuickMana` `Player.cs:5598-5649`) and the
//! per-tick sickness clocks (`potionDelay`/`manaPotionDelay` decrements
//! `Player.cs:24718-24724`, the buff-21 mirror `:11719-11722`, the buff-94
//! Mana Sickness arm `:10318-10327`).
//!
//! Scope note (census): the buff EFFECTS themselves (Ironskin +8 defense,
//! Swiftness +0.25 move speed, …) already landed with S1-E-7 waves A-D
//! (`player.rs`'s buff scan + `buff_effects*.rs`); this module only ports
//! the USE side — the sickness gates, the durations/buff ids the item rows
//! carry, the heal/mana restore and the stack consume. `QuickBuff`
//! (`Player.cs:5766-6017`) is census-MISSING and deferred (its food-priority
//! + ShouldBotherUsingThisBuff + mana-pay table is a separate slice).
//!
//! Coordination: `QuickHeal`'s H-key EDGE + the hotbar chooser also land on
//! the i54 branch (`hotbar.rs` + `world_sim/player.rs`); this module is the
//! canonical USE machinery (`Player.QuickHeal` proper) — the merge takes
//! i54's edge latch calling THIS `quick_heal`, not its inline drink copy.
//! The mana-flower auto-QuickMana inside `CheckMana` (`Player.cs:52588-
//! 52609`) is i19's seam: it calls this module's `quick_mana`. The magic-
//! damage sickness TAIL (`apply_magic_damage_sickness_tail` below) is
//! exported for i19's `Update` fold (`Player.cs:25900-25911`).
//!
//! Deliberate seams (ANALYSIS "Potion use (S1-P-i37)"):
//! - the mid-swing `ItemTimeIsZero` heal/buff window (`:42856-42883`) folds
//!   onto the press tick (the food arm's existing swing seam) — the RNG
//!   ORDER is preserved (delay roll before heal roll), only the tick shifts;
//! - the `AdjustRemainingPotionSickness` hook sits after the port's whole
//!   equip fold rather than strictly after `UpdateEquips` (`:25640`) — no
//!   other pass writes buff-21 timers, so the rescale is order-identical.
//!
//! C# source map (all line numbers into `decompiled/Terraria/`):
//! - `Item.potionDelay = 3600`, `restorationDelay = 2700`, `eggnogDelay =
//!   2400`, `mushroomDelay = 1800`: `Item.cs:49968-49971`.
//! - `Player.manaSickTime = 300`: `Player.cs:57101`;
//!   `PhilosopherStoneDurationMultiplier = 0.75f`: `Player.cs:57112`.
//! - the delay-time fields' per-tick rebuild + pStone scaling:
//!   `Player.cs:25477-25486`; `AdjustRemainingPotionSickness`:
//!   `Player.cs:28939-28952` (called at `:25640-25643` on a pStone change).
//! - `DebugOptions.ManaPotionDelay` is a false static ctor assignment
//!   (`Testing/DebugOptions.cs:28` field, `:86` ctor) — its arms stay
//!   constants-off, cited inline. `DebugOptions.ManaV2` is TRUE in the
//!   shipped build (`:26` field, `:85` ctor) — but that lane (cast-at-0-mana,
//!   the ×2.5 slow, `CheckMana`'s flower arm `:52588-52608`) is the i19
//!   magic-mana worker's machinery; THIS module only ports the ManaV2-
//!   independent halves: the full-mana QuickMana gate reads `!ManaV2`
//!   (`:5600`) so it does NOT block, and the `:51946-51948` UndoSlowMagicUse
//!   arm is noted at its site.
//!
//! RNG order (the only draws in the use path, both Strange Brew 3001):
//! 1. `ApplyPotionDelay`: `Main.rand.Next(2400, 4200 + 1)` (`:51857`) —
//!    drawn at the use START (`:42758`), BEFORE the heal;
//! 2. `ApplyLifeAndOrMana`: `Main.rand.Next(healLife, 120 + 1)` (`:51895`)
//!    then `Main.rand.NextFloat()` (`:51898`, the i-frame roll 240/120/60
//!    at ≤0.1 / ≤0.3 / ≤0.6).
//! All ride the `UpdatePlayers` named stream (`Player.ItemCheck` under the
//! `Main.cs:17754` swap); the callers hand it in.

use tr_core::UnifiedRandom;

use crate::app::App;
use crate::buffs::BuffSlots;
use crate::item::ItemSlot;
use crate::player::Player;
use crate::registry::{self, ItemDef};

/// `BuffID.PotionSickness` (`ID/BuffID.cs:213`).
pub const POTION_SICKNESS: i32 = 21;
/// `BuffID.ManaSickness` (`ID/BuffID.cs:359`).
pub const MANA_SICKNESS: i32 = 94;
/// `Player.manaSickTime = 300` (`Player.cs:57101`) — the Mana Sickness
/// duration `ApplyLifeAndOrMana` grants (`:51938`) and the reduction's
/// denominator (`:10326`).
pub const MANA_SICK_TIME: i32 = 300;
/// `Item.potionDelay = 3600` (`Item.cs:49968`) — 60 s.
pub const POTION_DELAY: i32 = 3600;
/// `Item.restorationDelay = 2700` (`Item.cs:49969`) — 45 s.
pub const RESTORATION_DELAY: i32 = 2700;
/// `Item.eggnogDelay = 2400` (`Item.cs:49970`) — 40 s.
pub const EGGNOG_DELAY: i32 = 2400;
/// `Item.mushroomDelay = 1800` (`Item.cs:49971`) — 30 s.
pub const MUSHROOM_DELAY: i32 = 1800;
/// `Player.PhilosopherStoneDurationMultiplier = 0.75f` (`Player.cs:57112`).
pub const PHILOSOPHER_STONE_MULTIPLIER: f32 = 0.75;

/// The `Player.potionDelayTime`/`restorationDelayTime`/`mushroomDelayTime`
/// trio (`Player.cs:2886-2892`), rebuilt every tick from the `Item` statics
/// and scaled by 0.75 under the Philosopher's Stone family
/// (`Player.cs:25477-25486`). `eggnogDelayTime` is NOT scaled (`:25478-25480`
/// rebuilds only the three; the eggnog field keeps its `Item.eggnogDelay`
/// initializer, `:2890`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PotionDelayTimes {
    pub potion: i32,
    pub restoration: i32,
    pub mushroom: i32,
}

impl PotionDelayTimes {
    /// `Player.cs:25478-25486`: the bases, then the pStone `(int)(f32 *
    /// 0.75f)` truncation on the three scaled rows.
    pub fn rebuild(p_stone: bool) -> Self {
        let mut t = Self {
            potion: POTION_DELAY,
            restoration: RESTORATION_DELAY,
            mushroom: MUSHROOM_DELAY,
        };
        if p_stone {
            t.potion = (t.potion as f32 * PHILOSOPHER_STONE_MULTIPLIER) as i32;
            t.restoration = (t.restoration as f32 * PHILOSOPHER_STONE_MULTIPLIER) as i32;
            t.mushroom = (t.mushroom as f32 * PHILOSOPHER_STONE_MULTIPLIER) as i32;
        }
        t
    }
}

/// `Player.ApplyPotionDelay` (`Player.cs:51853-51886`): the Potion Sickness
/// duration by item type, then `AddBuff(21, potionDelay)`. The Strange Brew
/// 3001 draws `Next(2400, 4201)` FIRST (`:51857`) and scales the roll by
/// 0.75 under pStone (`:51858-51861`); the Restoration 227/126 pair rides
/// `restorationDelayTime` (`:51864-51868`); the Eggnog 1912 and the Mushroom
/// 5 ride their own times AND reset hunger (`:51869-51880`); everything
/// else rides `potionDelayTime` (`:51881-51885`).
///
/// W4-e10 — returns the hunger transition flag: true when the reset's
/// `UpdateHungerBuffs` fired the 333-expiry arm, whose `PlaySound(
/// SoundID.Hungry)` (`Player.cs:12138`, UNPOSITIONED) the app-level caller
/// plays (the Player has no audio device — the `pending_hurt_sound`
/// handoff pattern, folded to a return value on this pure path).
pub fn apply_potion_delay(
    p: &mut Player,
    def: &ItemDef,
    times: &PotionDelayTimes,
    rng: &mut UnifiedRandom,
    dont_starve: bool,
    remix: bool,
) -> bool {
    if def.id == 3001 {
        // `:51857`: `Next(2400, 4200 + 1)` — the upper bound is EXCLUSIVE in
        // the C# `Next(min, max)`, the +1 makes 4200 reachable.
        p.potion_delay = rng.next_range(2400, 4200 + 1);
        if p.p_stone {
            // `:51860`: `(int)((float)potionDelay * 0.75f)`.
            p.potion_delay = (p.potion_delay as f32 * PHILOSOPHER_STONE_MULTIPLIER) as i32;
        }
        p.buffs.add_buff(POTION_SICKNESS, p.potion_delay); // `:51862`
        false
    } else if def.id == 227 || def.id == 126 {
        p.potion_delay = times.restoration; // `:51866`
        p.buffs.add_buff(POTION_SICKNESS, p.potion_delay); // `:51867`
        false
    } else if def.id == 1912 {
        p.potion_delay = EGGNOG_DELAY; // `:51871` (`eggnogDelayTime`, unscaled)
        p.buffs.add_buff(POTION_SICKNESS, p.potion_delay); // `:51872`
        try_to_reset_hunger_to_neutral(p, dont_starve, remix) // `:51873`
    } else if def.id == 5 {
        p.potion_delay = times.mushroom; // `:51877`
        p.buffs.add_buff(POTION_SICKNESS, p.potion_delay); // `:51878`
        try_to_reset_hunger_to_neutral(p, dont_starve, remix) // `:51879`
    } else {
        p.potion_delay = times.potion; // `:51883`
        p.buffs.add_buff(POTION_SICKNESS, p.potion_delay); // `:51884`
        false
    }
}

/// `Player.ApplyLifeAndOrMana` (`Player.cs:51888-51952`): the heal/mana
/// restore with the max clamps, the Strange Brew's random heal + i-frame
/// roll, the HealLife combat text and the Mana Sickness grant. Returns the
/// queued heal-text amount (`HealEffect`, `:51929-51932`) — the caller owns
/// `Main.combatText`; the mana text rides the existing
/// `pending_mana_effect` seam (`ManaEffect`, `:51950`).
///
/// RNG order: the Strange Brew heal roll `Next(healLife, 121)` (`:51895`)
/// THEN the i-frame `NextFloat()` (`:51898`). Both only for type 3001.
pub fn apply_life_and_or_mana(
    p: &mut Player,
    def: &ItemDef,
    rng: &mut UnifiedRandom,
) -> Option<i32> {
    let mut num = def.heal_life; // `:51890`
    let mut heal_mana = def.heal_mana; // `:51891`
    if def.id == 3001 {
        // `:51892-51917`: the Strange Brew heals a RANDOM amount in
        // [healLife, 120] and rolls bonus i-frames on the local player.
        num = rng.next_range(def.heal_life, 120 + 1); // `:51895`
        // `Main.myPlayer == whoAmI` is always true in the single-player port.
        let num2 = rng.next_float(); // `:51898`
        let mut num3 = 0;
        if num2 <= 0.1 {
            num3 = 240; // `:51900-51903`
        } else if num2 <= 0.3 {
            num3 = 120; // `:51904-51907`
        } else if num2 <= 0.6 {
            num3 = 60; // `:51908-51911`
        }
        if num3 > 0 {
            p.set_immune_time_for_all_types(num3); // `:51912-51915`
        }
    }
    // `:51918`: `healMana = (int)((float)healMana * 1f)` — the identity
    // multiply (a ManaV2 hook in the source), truncation included.
    heal_mana = (heal_mana as f32 * 1.0f32) as i32;
    p.hp += num; // `:51919` `statLife += num`
    p.mana += heal_mana; // `:51920` `statMana += healMana`
    if p.hp > p.max_hp {
        p.hp = p.max_hp; // `:51921-51924`
    }
    if p.mana > p.mana_max {
        p.mana = p.mana_max; // `:51925-51928`
    }
    let heal_text = if num > 0 {
        Some(num) // `:51929-51932` `HealEffect(num)` (myPlayer always)
    } else {
        None
    };
    if heal_mana <= 0 {
        return heal_text; // `:51933-51936`
    }
    p.mana_potion_delay = 0; // `:51937`
    p.buffs.add_buff(MANA_SICKNESS, MANA_SICK_TIME); // `:51938`
    // `:51939-51943`: `DebugOptions.ManaPotionDelay` is false (static ctor
    // `Testing/DebugOptions.cs:86`) — the 8*60 delay + re-grant stay
    // constants-off.
    // `:51944-51951`: `ManaEffect(healMana)` queues the HealMana text. The
    // `DebugOptions.ManaV2 && slowMagicUse` -> `UndoSlowMagicUse` arm
    // (`:51946-51948`) IS live in the shipped build (ManaV2 true, `:85`) but
    // `slowMagicUse`/the animation rescale are the i19 mana worker's fields
    // — that half lands with its `CheckMana` fold (coordination note).
    p.queue_mana_effect(heal_mana); // `:51950`
    heal_text
}

/// `Player.TryToResetHungerToNeutral` (`Player.cs:12081-12096`): zero the
/// timers of the 332/333/334 hunger buffs, then `UpdateHungerBuffs`
/// (`:12098-12150`). The `Main.dontStarveWorld`/`Main.remixWorld` arms read
/// the world seed flags the caller hands in; the emote half is a UI seam.
/// NOTE: like the C#, the follow-up `UpdateHungerBuffs` skips every slot
/// whose timer was just zeroed (`:12102-12105`) — the call is a structural
/// no-op on the reset slots, kept for the source shape. W4-e10: returns
/// the 333-expiry flag (the `SoundID.Hungry` play, `:12138`) for the
/// app-level caller.
pub fn try_to_reset_hunger_to_neutral(p: &mut Player, dont_starve: bool, remix: bool) -> bool {
    let mut flag = false;
    for i in 0..crate::buffs::MAX_BUFFS {
        let bt = p.buffs.buff_type[i];
        if bt > 0 && p.buffs.buff_time[i] > 0 && matches!(bt, 332 | 333 | 334) {
            p.buffs.buff_time[i] = 0; // `:12088`
            flag = true; // `:12089`
        }
    }
    if flag {
        update_hunger_buffs(p, dont_starve, remix) // `:12092-12095`
    } else {
        false
    }
}

/// `Player.UpdateHungerBuffs` (`Player.cs:12098-12150`). The port snapshots
/// the live (type, time) pairs first because `add_buff` compacts slots — the
/// observable writes (the expiry grants + the zeroing) land on the same buff
/// TYPES. The 332 expiry's remix+dontStarve arm grants 28800 (`:12110-12113`),
/// every other world 18000 (`:12116`). W4-e10: returns true when the
/// 333-expiry arm fired (`:12127-12139`) — its `PlaySound(SoundID.Hungry)`
/// (`:12138`) is the app-level caller's (the emote `:12137` stays a UI seam).
fn update_hunger_buffs(p: &mut Player, dont_starve: bool, remix: bool) -> bool {
    let mut hungry = false;
    let slots: Vec<(i32, i32)> = (0..crate::buffs::MAX_BUFFS)
        .filter(|&i| p.buffs.buff_type[i] > 0 && p.buffs.buff_time[i] > 0)
        .map(|i| (p.buffs.buff_type[i], p.buffs.buff_time[i]))
/// every other world 18000 (`:12116`).
fn update_hunger_buffs(p: &mut Player, dont_starve: bool, remix: bool) {
    update_hunger_buffs_slots(&mut p.buffs, dont_starve, remix);
}

/// The slot-level keep-one of `UpdateHungerBuffs` (S1-E-7 wave F): the
/// per-tick frame call site (`buff_effects_frame.rs`, `Player.cs:12046`)
/// runs the SAME ladder on `BuffSlots` — no second copy of the expiry
/// logic. The emote/sound requests the frame adds are gated on the same
/// pre-expiry timers this ladder reads.
pub fn update_hunger_buffs_slots(slots: &mut BuffSlots, dont_starve: bool, remix: bool) {
    let live: Vec<(i32, i32)> = (0..crate::buffs::MAX_BUFFS)
        .filter(|&i| slots.buff_type[i] > 0 && slots.buff_time[i] > 0)
        .map(|i| (slots.buff_type[i], slots.buff_time[i]))
        .collect();
    for (bt, time) in live {
        // Re-find by type: an earlier AddBuff may have compacted the slot.
        let Some(slot) = slots.find_buff_index(bt) else {
            continue;
        };
        match bt {
            332 => {
                if time <= 2 {
                    // `:12108-12117`: remix AND dontStarve → 28800, else 18000.
                    let t = if remix && dont_starve { 28800 } else { 18000 };
                    slots.add_buff(333, t);
                }
                if !dont_starve {
                    // Re-find AFTER the add: `AddBuff(333)`'s fed-state
                    // cleanup (`:5277-5286`, 332/333/334 are all IsFedState)
                    // may have deleted + compacted the 332 slot — the C#
                    // writes `buffTime[i]` on whatever now occupies `i`,
                    // the port's type re-find lands the same zeroing on the
                    // live 332 when it survived (the dontStarve arm).
                    if let Some(s) = slots.find_buff_index(332) {
                        slots.buff_time[s] = 0; // `:12120-12123`
                    }
                }
            }
            333 => {
                if time <= 2 {
                    p.buffs.add_buff(334, 5); // `:12131`/`:12135` (both arms 5)
                    // `:12138`: `SoundEngine.PlaySound(SoundID.Hungry)` —
                    // UNPOSITIONED, the case-42 trackable (one variation,
                    // no gameplay draw; the façade NextFloat audio-only).
                    hungry = true;
                    slots.add_buff(334, 5); // `:12131`/`:12135` (both arms 5)
                }
                if !dont_starve {
                    if let Some(s) = slots.find_buff_index(333) {
                        slots.buff_time[s] = 0; // `:12140-12143`
                    }
                }
            }
            334 => {
                if !dont_starve {
                    slots.buff_time[slot] = 0; // `:12145-12148`
                }
            }
            _ => {}
        }
    }
    hungry
}

/// The buff-94 Mana Sickness effect arm (`Player.cs:10318-10327`):
/// `manaSick = true` and `manaSickReduction = 0.25f * buffTime /
/// manaSickTime`. The `DebugOptions.ManaPotionDelay` mirror (`:10320-10323`)
/// is constants-off (false retail static). Called from the per-tick buff
/// scan for every live 94 slot; the LAST live slot wins, like the C# chain.
/// W4-f49 stacking census: the grant stacks ADDITIVELY on the drink —
/// `AddBuff_TryUpdatingExistingBuffTime` (`:5410-5418`) reads
/// `BuffID.Sets.AddBuffTimeAdditivelyToCap[94] = 600` (`BuffID.cs:53`):
/// `buffTime += 300` capped at 600, so a second drink inside the window
/// lifts the timer (and, on the next scan, the reduction) instead of
/// refreshing it. The reduction can therefore EXCEED 0.25 — up to 0.5 at
/// the 600 cap; it is one ASSIGN per scan tick (`:10326`), not a multiply,
/// so the damage cut never compounds across ticks. The 94 id is absent
/// from `BuffTimeIsExtendedWithGameDifficulty` (`BuffID.cs:50`) — expert
/// worlds do NOT extend it. 94 is NOT `Main.persistentBuff` — the death
/// sweep (`:16995-17002`) drops it.
pub fn apply_mana_sickness_arm(p: &mut Player, buff_time: i32) {
    p.mana_sick = true; // `:10325`
    p.mana_sick_reduction = 0.25f32 * (buff_time as f32 / MANA_SICK_TIME as f32); // `:10324-10326`
}

/// The `Player.Update` magic-damage tail (`Player.cs:25900-25911`):
/// `if (manaSick) magicDamage *= 1f - manaSickReduction;` then
/// `magicDamage *= manaHeat` (the `0f == 0f` branch always taken). The
/// manaHeat multiplier stays 1f in the use path — `ApplyManaPotionManaHeat`
/// (`Player.cs:51968-52004`) has NO caller in the retail source (verified:
/// the only references are its own definition and `TryApplyManaHeat`, which
/// is reached only by the mana-star pickups `:35071`/`:35087`, an unported
/// lane), so drinking a mana potion grants NO ManaHeat buff.
pub fn apply_magic_damage_sickness_tail(p: &mut Player) {
    if p.mana_sick {
        p.magic_damage *= 1.0f32 - p.mana_sick_reduction; // `:25900-25903`
    }
    p.magic_damage *= p.mana_heat; // `:25904-25907`
}

/// `Player.AdjustRemainingPotionSickness` (`Player.cs:28939-28952`): on a
/// pStone change (`:25640-25643`), rescale the live buff-21 timer —
/// `pStone ? time * 0.75 : time / 0.75`, truncated. The myPlayer gate is
/// always true in the single-player port.
pub fn adjust_remaining_potion_sickness(p: &mut Player) {
    if let Some(i) = p.buffs.find_buff_index(POTION_SICKNESS) {
        let t = p.buffs.buff_time[i] as f32;
        let scaled = if p.p_stone {
            t * PHILOSOPHER_STONE_MULTIPLIER
        } else {
            t / PHILOSOPHER_STONE_MULTIPLIER
        };
        p.buffs.buff_time[i] = scaled as i32; // `:28949`
    }
}

/// `Player.ItemCheck_CheckCanUse_Inner`'s potion gate (`Player.cs:52217-
/// 52220`): `if (sItem.potion && this.potionDelay > 0) flag = false;`.
pub fn potion_use_blocked(p: &Player, def: &ItemDef) -> bool {
    def.potion && p.potion_delay > 0
}

/// W4-f49 — `Player.CheckMana` WITH the mana-flower auto-drink lane
/// (`Player.cs:52576-52617`), the App-level twin of [`crate::regen::check_mana`]
/// for callers that can reach the inventory: the entry clear (`:52578`), the
/// enough-mana pay (`:52579-52586`), then the flower arm (`:52588-52609`) —
/// `QuickMana()` (`:52592`, the C#'s vestigial `if (true)` wrapper included)
/// and the post-drink RE-CHECK/pay (`:52594-52601`); a failed/absent drink
/// falls to the SAME ManaV2 empty arm as the plain lane (`:52602-52606` /
/// `:52610-52614`: drain to 0, `DoRunOutOfManaEffect`, true). The entry
/// clear runs BEFORE the drink, so the drink's `UndoSlowMagicUse` gate
/// (`:51946`) sees `slowMagicUse == false` — the mid-swing timer shrink
/// never fires on the flower path (the C# order, witnessed).
/// `block_quick_mana` (`CheckMana`'s third arg `:52576`) is false at every
/// call site in the source (`ActuallyPayMana` `:52650`, the Last Prism
/// interval pay `Projectile.cs:66613`); a true caller keeps the plain V2
/// empty arm with NO drink.
pub fn check_mana_with_flower(app: &mut App, amount: i32, pay: bool, block_quick_mana: bool) -> bool {
    check_mana_with_flower_on(app, amount, pay, block_quick_mana, crate::app::RngKey::UpdatePlayers)
}

/// W4-f49 — [`check_mana_with_flower`] with an explicit named-stream key for
/// the flower drink's draws (the Last Prism interval pay runs under
/// `UpdateProjectiles`, `Main.cs:17775`; the weapon use path under
/// `UpdatePlayers`, `:17754`).
pub fn check_mana_with_flower_on(
    app: &mut App,
    amount: i32,
    pay: bool,
    block_quick_mana: bool,
    key: crate::app::RngKey,
) -> bool {
    let (num, flower) = {
        let Some(p) = app.player.as_mut() else {
            return false;
        };
        p.slow_magic_use = false; // `:52578`
        let num = (amount as f32 * p.mana_cost) as i32; // `:52579`
        if p.mana >= num {
            if pay {
                p.mana -= num; // `:52584`
            }
            return true; // `:52586`
        }
        (num, p.mana_flower)
    };
    if flower && !block_quick_mana {
        quick_mana_on(app, key); // `:52590-52593` — the drink's own gates decide
        let Some(p) = app.player.as_mut() else {
            return false;
        };
        if p.mana >= num {
            if pay {
                p.mana -= num; // `:52596-52599`
            }
            return true; // `:52600`
        }
        // `:52602-52606` — the flower lane's ManaV2 twin of the empty arm.
        p.mana = 0;
        p.slow_magic_use = true; // `DoRunOutOfManaEffect` `:52619-52622`
        return true;
    }
    // No flower (or blocked): the plain lane's V2 empty arm (`:52610-52614`).
    let Some(p) = app.player.as_mut() else {
        return false;
    };
    p.mana = 0;
    p.slow_magic_use = true;
    true
}

/// W4-f49 MANA FINAL — the CheckMana constants-copy convention for the
/// AI-lane ports (research/85 FINAL-STATUS): the d-lanes' projectile AI
/// bodies call `crate::regen::check_mana` (the pure pool-side pay) and,
/// where the flower lane is owed, record the shared
/// `wants_prism_flower_drink` intent — the pass-tail drain re-runs the
/// FULL CheckMana through [`check_mana_with_flower_on`] on the
/// `UpdateProjectiles` stream. The five R1-census sites:
/// - `Projectile.cs:66204` (AI_075 type 439, Laser Machinegun) — the AI
///   body is STILL unported (the i22 OUT list; no d-lane covers the 75
///   holder's 439 arm). The site rides that lane; the constants below
///   are its pay's shape when it lands.
/// - `Projectile.cs:36943` (AI_208, Glacier Fang 1115) — d21 LANDED the
///   body (`projectile_ai_201_210.rs::ai_glacier_fang_shot`, tip
///   9fa9e9ba0): the every-20-tick `ai[0] <= 60` charge pay rides
///   `check_mana` on the held row's mana (6153: 10).
/// - `Projectile.cs:42662` (AI_185, Life Drain / Soul Drain 476) — d19
///   LANDED the body (`projectile_ai_sword_styles.rs::ai_life_drain`,
///   tip 1054c396a): the first-tick overlap pay rides `check_mana` on
///   the held row's mana (3006: 10), then latches `localAI[0] = 1`.
/// - `Projectile.cs:44567` (AI_100, Medusa Head 535) — d10 LANDED the
///   body (`projectile_ai_91_100.rs::ai_medusa_head`, tip 1f0b60e96):
///   the fire-cadence pay rides `check_mana` on the held row's mana
///   (3269: 15); the `!flag ||` short-circuit never skips it.
/// - `Player.cs:56870` (CommandForbiddenStorm) — the storm command's
///   lane (deferred, ANALYSIS): the flat 20-mana pay + the
///   ApplyManaRegenerationDelay follow-up. The constants-copy below
///   pins the pay's shape for that lane.
/// The held-row mana values are the generated itemdata rows (the
/// constants-copy: the AI lanes read `registry::item(...).mana`, never
/// a hand-typed number — these constants are the WITNESS pins only).
pub const LASER_MACHINEGUN_MANA: i32 = 6; // itemdata 2795 (Item.cs row)
pub const GLACIER_FANG_MANA: i32 = 10; // itemdata 6153
pub const SOUL_DRAIN_MANA: i32 = 10; // itemdata 3006
pub const MEDUSA_HEAD_MANA: i32 = 15; // itemdata 3269
/// `Player.CommandForbiddenStorm`'s flat pay (`Player.cs:56870`:
/// `this.CheckMana(20, pay: true)`) — NOT a held-row read.
pub const FORBIDDEN_STORM_MANA: i32 = 20;

/// Whether the held kind rides the potion/buff drink arm: a `potion`-flagged
/// item, one carrying healLife/healMana, or a buff consumable in the
/// EatFood/DrinkLiquid styles (2/9). The summon staffs carry buffType too
/// but use style 5 and are handled by the shoot path (`:42764`'s gate).
/// The WellFed food kinds (26/206/207) are the FOOD arm's (editing.rs's
/// `can_eat` branch runs first and owns the 14400 fallback) — excluded here
/// so a well-fed player's food never re-consumes through the drink seam.
fn drinkable(def: &ItemDef) -> bool {
    def.consumable
        && !matches!(def.buff_type, 26 | 206 | 207)
        && (def.potion
            || def.heal_life > 0
            || def.heal_mana > 0
            || (def.buff_type > 0 && matches!(def.use_style, 2 | 9)))
}

/// One potion/buff-item USE through the port's swing seam — the C#
/// `ItemCheck` order: the can-use gate (`:52217`), `ApplyPotionDelay` at the
/// use start (`:42756-42758`), the `ItemCheck_StartActualUse` sound
/// (`:51680-51698`), then at `ItemTimeIsZero` the `ApplyLifeAndOrMana` heal
/// (`:42867-42875`), the `buffType > 0` grant (`:42876-42883`, the 90/27
/// exclusions) and the generic consumable tail (`:43280-43326`). The port
/// folds the mid-swing `ItemTimeIsZero` window onto the press tick (the
/// swing/`attack_cd` seam the food arm already uses), so the heal + buff +
/// consume land together; the RNG order is the source's (delay roll →
/// use-sound pitch roll → heal roll → HealEffect jitter). Returns true when
/// the use fired.
pub fn use_potion_item(app: &mut App, held: ItemSlot) -> bool {
    let Some(def) = registry::get(held.kind) else {
        return false;
    };
    if !drinkable(def) {
        return false;
    }
    let dont_starve = app
        .world
        .seeds
        .contains(tr_core::seeds::SeedFlags::DONT_STARVE);
    let remix = app
        .world
        .seeds
        .contains(tr_core::seeds::SeedFlags::REMIX);
    // The gates (immutable reads).
    {
        let Some(p) = app.player.as_ref() else {
            return false;
        };
        // `:52217-52220`: the Potion Sickness clock blocks every `potion`-
        // flagged item (buff potions are NOT potion-flagged in the source —
        // `Item.cs:4858-4871`'s Ironskin row — so they drink through sickness).
        if potion_use_blocked(p, def) {
            return false;
        }
        // `:52145-52148`: the cursed gate (buff 23 arms `noItems`/`cursed`,
        // `:11731-11735`).
        if p.buffs.find_buff_index(23).is_some() {
            return false;
        }
    }
    // Phase 1 — the use START: `ApplyPotionDelay` (`:42756-42758`, the
    // Strange Brew delay roll FIRST) then `ItemCheck_StartActualUse`'s sound
    // (`:51680-51698`). The animation/timers arm here too — the heal phase
    // below reads nothing from them, so the order is the C#'s.
    let hungry = {
        let Some(p) = app.player.as_mut() else {
            return false;
        };
        let times = PotionDelayTimes::rebuild(p.p_stone);
        let rng = app.rngs.stream(crate::app::RngKey::UpdatePlayers);
        if def.potion {
            apply_potion_delay(p, def, &times, rng, dont_starve, remix)
        } else {
            false
        }
    };
    // W4-e10 — the Eggnog/Mushroom reset's 333-expiry arm fired inside
    // `ApplyPotionDelay` (`Player.cs:12138`: `PlaySound(SoundID.Hungry)` —
    // UNPOSITIONED, `SoundEngine.cs:93`'s x=-1/y=-1 defaults, the case-42
    // trackable façade: the `GetRandomPitch` NextFloat is audio-only, no
    // gameplay draw). Played HERE — after the delay phase, before the
    // swing seam — the C#'s position in the call chain.
    if hungry {
        let rng = app.rngs.stream(crate::app::RngKey::UpdatePlayers);
        app.audio.play_voice_style("Hungry", -1.0, -1.0, 0.0, rng);
    }
    // The swing seam (`ApplyItemAnimation`, `Player.cs:4449-4451` + the
    // sound). The reuse lock (`SetItemAnimation`'s tail, `:4496`) rides the
    // use start like the weapon arm's.
    app.swing_max = crate::item::use_animation_secs_of(held.kind, held.prefix);
    app.swing = app.swing_max;
    crate::world_sim::editing::start_swing_extras(app, held.kind);
    app.attack_cd = app.swing_max;
    app.reuse_delay = crate::prefix::prefixed_stats(held.kind, held.prefix)
        .map_or(0.0, |s| s.reuse_delay as f32 / 60.0);
    app.mine_target = None;
    app.mine_timer = 0.0;
    // Phase 2 — the mid-swing window folded onto this tick: the heal/mana
    // restore (`:42867-42875`), the buff grant (`:42876-42883`) and the
    // Bottled Water drowning check (`:42871-42874`).
    let (heal_text, drowning) = {
        let Some(p) = app.player.as_mut() else {
            return false;
        };
        let rng = app.rngs.stream(crate::app::RngKey::UpdatePlayers);
        let heal_text = if def.heal_life > 0 || def.heal_mana > 0 {
            apply_life_and_or_mana(p, def, rng)
        } else {
            None
        };
        let drowning = def.id == 126 && p.breath == 0;
        // The buffTime 0 fallback is NOT applied here — the C# passes
        // item.buffTime verbatim on the ItemCheck path (the 3600 fallback is
        // QuickHeal/QuickBuff-only, `:5528-5532`/`:5785-5789`).
        if def.buff_type > 0 && def.buff_type != 90 && def.buff_type != 27 {
            p.buffs.add_buff(def.buff_type, def.buff_time);
        }
        (heal_text, drowning)
    };
    // The HealLife text (`HealEffect`, `:4522-4529`) + the ManaEffect text
    // (the pending seam the world_sim player pass drains).
    if let Some(amount) = heal_text {
        spawn_heal_text(app, amount);
    }
    // `:51944-51949`: the ManaV2 `UndoSlowMagicUse` arm (the i19 fold —
    // the i37 coordination note deferred it to the mana worker; it lives
    // inside `ApplyLifeAndOrMana`, so the QuickMana path `:5618` rides it
    // too). The restore above just ran; a mid-swing drink stretches the
    // timers back by `GetSlowMagicUseRate()` = 1/2.5 (`:51954-51966`, int
    // truncation, floor 2 ticks), then clears the flag. The gate is
    // `itemAnimation != 0 && itemTime != 0` — the swing seam above armed
    // the animation, so the press-tick drink's itemTime (0) keeps the arm
    // a no-op there, exactly like the C#'s use-start order.
    if def.heal_mana > 0 {
        undo_slow_magic_use(app);
    }
    // `:43280-43326`: the generic consumable tail — one per use; the
    // ForceConsumption set (`ItemID.cs:413-427`) does not touch the drink
    // kinds, `CanConsumeConsumableItem` is `return true` (`:5496-5499`).
    // The C# leaves the 0-stack slot until the next tick's IsAir check
    // (`:42824-42828` `TurnToAir`) — `item.IsAir` is `stack <= 0`
    // (`Item.cs:385-395`), so the use gates already refuse it; the port
    // folds the TurnToAir onto the consume tick (the food arm's convention).
    let slot = &mut app.inventory[app.selected];
    if slot.count > 0 {
        slot.count -= 1; // `:43317-43320`
    }
    if slot.count == 0 {
        *slot = crate::item::EMPTY_SLOT; // `:42824-42828`
    }
    if drowning {
        // HandleSpecialEvent(25) → DRINK_BOTTLED_WATER_WHILE_DROWNING "Use"
        // (`AchievementsHelper.cs:370-372`). W4-c36: the C# names ONE row's
        // condition (`GetCondition(row, "Use").Complete()`), never a
        // flag-wide broadcast — the old `notify_custom("Use")` also closed
        // STAR_POWER/GET_A_LIFE/TOPPED_OFF/… on a drowned sip.
        let _ = app
            .achievements
            .complete_condition("DRINK_BOTTLED_WATER_WHILE_DROWNING", "Use");
    }
    true
}

/// `Player.QuickHeal_GetItemToUse_TryChoosingItem` (`Player.cs:5568-5596`):
/// the best-fit healing potion for the missing life. The Restoration 227
/// gets a +30 forgiveness when it would overheal (`:5575-5582`); the
/// selection prefers the smallest non-negative overshoot, falling back to
/// the largest under-heal while nothing covers the gap (`:5583-5595`).
fn quick_heal_try_choosing(
    life_difference: i32,
    best: &mut Option<(ItemSlot, i32)>,
    next: ItemSlot,
) {
    let Some(def) = registry::get(next.kind) else {
        return;
    };
    // `:5570`: stack > 0, type > 0, potion, healLife > 0.
    if next.count == 0 || next.kind == 0 || !def.potion || def.heal_life <= 0 {
        return;
    }
    let mut num = def.heal_life - life_difference; // `:5574`
    if next.kind == 227 && num < 0 {
        num += 30; // `:5577`
        if num > 0 {
            num = 0; // `:5578-5581`
        }
    }
    match best {
        None => {
            // `bestDifference` starts at `-statLifeMax2` (`:5553`); the C#
            // `bestDifference < 0` branch takes any `num > bestDifference`.
            // The port models the initial sentinel as None and accepts the
            // first candidate outright (every real num > -statLifeMax2).
            *best = Some((next, num));
        }
        Some((_, best_diff)) => {
            if *best_diff < 0 {
                if num > *best_diff {
                    *best = Some((next, num)); // `:5585-5589`
                }
            } else if num < *best_diff && num >= 0 {
                *best = Some((next, num)); // `:5591-5595`
            }
        }
    }
}

/// A quick-use slot location: the inventory grid, the coin/ammo rows or the
/// void bag (bank4). The C# `inventory[0..58]` spans all three port arrays.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum QuickSlot {
    Inventory(usize),
    Coin(usize),
    Ammo(usize),
    VoidBag(usize),
}

impl QuickSlot {
    fn slot(self, app: &App) -> ItemSlot {
        match self {
            QuickSlot::Inventory(i) => app.inventory[i],
            QuickSlot::Coin(i) => app.equip.coins[i],
            QuickSlot::Ammo(i) => app.equip.ammo[i],
            QuickSlot::VoidBag(i) => app.player.as_ref().unwrap().banks[3][i],
        }
    }
    fn consume(self, app: &mut App) {
        let dec = |slot: &mut ItemSlot| {
            if slot.count > 0 {
                slot.count -= 1;
            }
            if slot.count == 0 {
                *slot = crate::item::EMPTY_SLOT; // `TurnToAir`
            }
        };
        match self {
            QuickSlot::Inventory(i) => dec(&mut app.inventory[i]),
            QuickSlot::Coin(i) => dec(&mut app.equip.coins[i]),
            QuickSlot::Ammo(i) => dec(&mut app.equip.ammo[i]),
            QuickSlot::VoidBag(i) => {
                if let Some(p) = app.player.as_mut() {
                    dec(&mut p.banks[3][i]);
                }
            }
        }
    }
}

/// `Player.QuickHeal_GetItemToUse` (`Player.cs:5549-5566`): scan the 58
/// inventory slots (the port's 50 grid + 4 coin + 4 ammo rows), then the
/// void bag's 40 when enabled. Returns the chosen slot's location.
pub fn quick_heal_get_item_to_use(app: &App) -> Option<QuickSlot> {
    let p = app.player.as_ref()?;
    let life_difference = p.max_hp - p.hp; // `:5551`
    let mut best: Option<(ItemSlot, i32)> = None;
    let mut best_loc: Option<QuickSlot> = None;
    let mut consider = |slot: ItemSlot, loc: QuickSlot| {
        let before = best;
        quick_heal_try_choosing(life_difference, &mut best, slot);
        if best != before {
            best_loc = Some(loc);
        }
    };
    for (i, slot) in app.inventory.iter().enumerate() {
        consider(*slot, QuickSlot::Inventory(i));
    }
    for (i, slot) in app.equip.coins.iter().enumerate() {
        consider(*slot, QuickSlot::Coin(i));
    }
    for (i, slot) in app.equip.ammo.iter().enumerate() {
        consider(*slot, QuickSlot::Ammo(i));
    }
    if p.void_vault_enabled {
        for (i, slot) in p.banks[3].iter().enumerate() {
            consider(*slot, QuickSlot::VoidBag(i));
        }
    }
    best_loc
}

/// `Player.QuickMana_GetItemToUse` (`Player.cs:5629-5649`): the FIRST
/// healMana > 0 item in slot order (no best-fit), skipping `potion`-flagged
/// items while the Potion Sickness clock runs (`:5633`/`:5642`).
pub fn quick_mana_get_item_to_use(app: &App) -> Option<QuickSlot> {
    let p = app.player.as_ref()?;
    let sick = p.potion_delay > 0;
    let ok = |slot: &ItemSlot| -> bool {
        let Some(def) = registry::get(slot.kind) else {
            return false;
        };
        slot.count > 0 && slot.kind > 0 && def.heal_mana > 0 && (!sick || !def.potion)
    };
    for (i, slot) in app.inventory.iter().enumerate() {
        if ok(slot) {
            return Some(QuickSlot::Inventory(i));
        }
    }
    for (i, slot) in app.equip.coins.iter().enumerate() {
        if ok(slot) {
            return Some(QuickSlot::Coin(i));
        }
    }
    for (i, slot) in app.equip.ammo.iter().enumerate() {
        if ok(slot) {
            return Some(QuickSlot::Ammo(i));
        }
    }
    if p.void_vault_enabled {
        for (i, slot) in p.banks[3].iter().enumerate() {
            if ok(slot) {
                return Some(QuickSlot::VoidBag(i));
            }
        }
    }
    None
}

/// `Player.QuickHeal` (`Player.cs:5501-5547`): the hotkey heal. The gates
/// (`:5503`): pending inventory actions / cursed / CCed / dead / full life /
/// `potionDelay > 0`. Then the best-fit item, the can-use gate, the mount
/// early-dismount, the use sound, the potion delay, the heal, the buff grant
/// WITH the 3600 fallback (`:5526-5534`), the consume and the Bottled Water
/// achievement. Returns true when a potion was drunk.
pub fn quick_heal(app: &mut App) -> bool {
    // The gates (`:5503`): `Main.LocalPlayerHasPendingInventoryActions()` —
    // the port's cursor item (`Main.mouseItem`) is the stand-in; cursed =
    // the live buff-23 slot.
    {
        let Some(p) = app.player.as_ref() else {
            return false;
        };
        if app.mouse_item.count > 0
            || p.buffs.find_buff_index(23).is_some()
            || p.cced()
            || p.dead
            || p.hp == p.max_hp
            || p.potion_delay > 0
        {
            return false;
        }
    }
    let Some(loc) = quick_heal_get_item_to_use(app) else {
        return false; // `:5507-5510` (item == null)
    };
    let held = loc.slot(app);
    let Some(def) = registry::get(held.kind) else {
        return false;
    };
    let dont_starve = app
        .world
        .seeds
        .contains(tr_core::seeds::SeedFlags::DONT_STARVE);
    let remix = app
        .world
        .seeds
        .contains(tr_core::seeds::SeedFlags::REMIX);
    let (heal_text, drowning, hungry) = {
        let Some(p) = app.player.as_mut() else {
            return false;
        };
        if potion_use_blocked(p, def) {
            return false; // `ItemCheck_TryStartUse`'s can-use gate
        }
        let times = PotionDelayTimes::rebuild(p.p_stone);
        // `:5512-5515`: the mount early-dismount (DismountsOnItemUse).
        if p.mount.is_active() && p.mount.dismount_on_item_use() {
            p.try_dismount(&app.world);
        }
        // `:5516`: `PlaySound(item.UseSound, base.position)` — the TOP-LEFT
        // (not Center), the quick-use sound site. The C# plays the sound
        // BEFORE ApplyPotionDelay (`:5517`), so the pitch draw precedes the
        // Strange Brew delay roll on this path (the ItemCheck path's order
        // is the reverse — delay at `:42758`, sound at `:51680`).
        if let Some((idx, pitch)) = crate::item::initial_use_sound(held.kind) {
            let (x, y) = (p.x, p.y);
            let rng = app.rngs.stream(crate::app::RngKey::UpdatePlayers);
            app.audio.play_use_sound(idx, x, y, pitch, rng);
        }
        let rng = app.rngs.stream(crate::app::RngKey::UpdatePlayers);
        let mut hungry = false;
        if def.potion {
            hungry = apply_potion_delay(p, def, &times, rng, dont_starve, remix); // `:5517-5520`
        }
        let heal_text = apply_life_and_or_mana(p, def, rng); // `:5521`
        if def.id == 5 {
            hungry |= try_to_reset_hunger_to_neutral(p, dont_starve, remix); // `:5522-5525`
        }
        // `:5526-5534`: the buff grant WITH the buffTime 0 → 3600 fallback.
        if def.buff_type > 0 {
            let time = if def.buff_time == 0 { 3600 } else { def.buff_time };
            p.buffs.add_buff(def.buff_type, time);
        }
        let drowning = def.id == 126 && p.breath == 0; // `:5543-5546`
        (heal_text, drowning, hungry)
    };
    // W4-e10 — the reset's 333-expiry arm fired (`Player.cs:12138`:
    // `PlaySound(SoundID.Hungry)` — UNPOSITIONED, the case-42 trackable
    // façade, no gameplay draw). The Player-only block above cannot reach
    // the audio device; the flag rides out and plays here, before the
    // heal text — the C#'s position inside the use call.
    if hungry {
        let rng = app.rngs.stream(crate::app::RngKey::UpdatePlayers);
        app.audio.play_voice_style("Hungry", -1.0, -1.0, 0.0, rng);
    }
    if let Some(amount) = heal_text {
        spawn_heal_text(app, amount);
    }
    // `:5535-5542`: `CanConsumeConsumableItem` is true → stack--, TurnToAir.
    loc.consume(app);
    if drowning {
        // `:5543-5546` → HandleSpecialEvent(25), the row-targeted form
        // (`AchievementsHelper.cs:370-372`, W4-c36 — the quick-heal sip must
        // not close the other "Use" rows).
        let _ = app
            .achievements
            .complete_condition("DRINK_BOTTLED_WATER_WHILE_DROWNING", "Use");
    }
    true
}

/// `Player.QuickMana` (`Player.cs:5598-5627`): the hotkey mana restore. The
/// gates (`:5600`): pending actions / cursed / CCed / dead / full mana /
/// `manaPotionDelay > 0`. The full-mana arm is `!DebugOptions.ManaV2 &&
/// statMana == statManaMax2` — ManaV2 is TRUE in the shipped build
/// (`Testing/DebugOptions.cs:85`), so full mana does NOT block (the arm is
/// constants-off); `manaPotionDelay` stays 0 on the retail path
/// (`ManaPotionDelay` false, `:86`) but the gate is kept for structure.
/// Then the first healMana item, the can-use gate, the dismount, the sound,
/// the potion delay (mana potions are NOT potion-flagged, so this is a no-op
/// for them — kept for the C# structure), the restore and the consume.
pub fn quick_mana(app: &mut App) -> bool {
    quick_mana_on(app, crate::app::RngKey::UpdatePlayers)
}

/// W4-f49 — [`quick_mana`] with an explicit named-stream key. The hotkey /
/// mana-flower weapon path runs under `UpdatePlayers` (`Main.cs:17754`); the
/// Last Prism's interval-pay flower drink runs under `UpdateProjectiles`
/// (`Main.cs:17775`, the `AI_075` caller). Every draw + the heal text ride
/// the passed key so the two callers keep their own streams.
pub fn quick_mana_on(app: &mut App, key: crate::app::RngKey) -> bool {
    {
        let Some(p) = app.player.as_ref() else {
            return false;
        };
        if app.mouse_item.count > 0
            || p.buffs.find_buff_index(23).is_some()
            || p.cced()
            || p.dead
            // `:5600`: `!ManaV2 && statMana == statManaMax2` — ManaV2 true,
            // the full-mana refusal is constants-off (i19's machinery).
            || p.mana_potion_delay > 0
        {
            return false;
        }
    }
    let Some(loc) = quick_mana_get_item_to_use(app) else {
        return false;
    };
    let held = loc.slot(app);
    let Some(def) = registry::get(held.kind) else {
        return false;
    };
    let dont_starve = app
        .world
        .seeds
        .contains(tr_core::seeds::SeedFlags::DONT_STARVE);
    let remix = app
        .world
        .seeds
        .contains(tr_core::seeds::SeedFlags::REMIX);
    let (heal_text, hungry) = {
        let Some(p) = app.player.as_mut() else {
            return false;
        };
        if potion_use_blocked(p, def) {
            return false;
        }
        let times = PotionDelayTimes::rebuild(p.p_stone);
        if p.mount.is_active() && p.mount.dismount_on_item_use() {
            p.try_dismount(&app.world); // `:5609-5612`
        }
        if let Some((idx, pitch)) = crate::item::initial_use_sound(held.kind) {
            let (x, y) = (p.x, p.y);
            let rng = app.rngs.stream(key);
            app.audio.play_use_sound(idx, x, y, pitch, rng); // `:5613`
        }
        let rng = app.rngs.stream(crate::app::RngKey::UpdatePlayers);
        let hungry = if def.potion {
            apply_potion_delay(p, def, &times, rng, dont_starve, remix) // `:5614-5617`
        } else {
            false
        };
        (apply_life_and_or_mana(p, def, rng), hungry) // `:5618`
        let rng = app.rngs.stream(key);
        if def.potion {
            apply_potion_delay(p, def, &times, rng, dont_starve, remix); // `:5614-5617`
        }
        apply_life_and_or_mana(p, def, rng) // `:5618`
    };
    // W4-e10 — the reset's 333-expiry arm fired (`Player.cs:12138`), the
    // unpositioned case-42 Hungry trackable, no gameplay draw (the QuickHeal
    // twin's contract).
    if hungry {
        let rng = app.rngs.stream(crate::app::RngKey::UpdatePlayers);
        app.audio.play_voice_style("Hungry", -1.0, -1.0, 0.0, rng);
    }
    if let Some(amount) = heal_text {
        spawn_heal_text_on(app, amount, key);
    }
    // `:5618` → `ApplyLifeAndOrMana`'s ManaV2 `UndoSlowMagicUse` arm
    // (`:51946-51948`) — the QuickMana drink rides it too (the i19 fold).
    if def.heal_mana > 0 {
        undo_slow_magic_use(app);
    }
    loc.consume(app); // `:5619-5626`
    true
}

/// `Player.UndoSlowMagicUse` (`Player.cs:51954-51966`): the ManaV2
/// mid-swing mana-potion drink stretches the four use timers back by
/// `GetSlowMagicUseRate()` (= 1/2.5 while the flag is armed — the caller
/// gates on `slowMagicUse`, `:51946`), int-truncated and floored at 2
/// ticks, then clears the flag. The `itemAnimation != 0 && itemTime != 0`
/// gate keeps the press-tick drink (no action timer yet) a no-op. The
/// port's `app.swing`/`app.swing_max`/`app.item_time` stand in for the
/// C# timers (`itemTimeMax` has no port field — the cadence re-arms off
/// `useTime` at the next shot).
pub fn undo_slow_magic_use(app: &mut App) {
    let Some(p) = app.player.as_mut() else {
        return;
    };
    if !p.slow_magic_use {
        return;
    }
    if app.swing > 0.0 && app.item_time > 0.0 {
        let rate = 1.0 / crate::regen::SLOW_MAGIC_MULTIPLIER;
        let floor2 = 2.0 / 60.0;
        app.swing_max = (app.swing_max * rate).max(floor2);
        app.swing = (app.swing * rate).max(floor2);
        app.item_time = (app.item_time * rate).max(floor2);
        p.slow_magic_use = false;
    }
}

/// The HealLife combat-text spawn (`Player.HealEffect`, `Player.cs:4522-
/// 4529`): the player hitbox rect, the HealLife colour, the UpdatePlayers
/// stream. Shared by the ItemCheck and quick-use paths. Reads the player
/// coords into locals first so the pool/rng borrows stay disjoint.
pub(crate) fn spawn_heal_text(app: &mut App, amount: i32) {
    spawn_heal_text_on(app, amount, crate::app::RngKey::UpdatePlayers);
}

/// W4-f49 — [`spawn_heal_text`] with an explicit stream key (the prism
/// flower drink's text rides `UpdateProjectiles`, `Main.cs:17775`).
fn spawn_heal_text_on(app: &mut App, amount: i32, key: crate::app::RngKey) {
    let rng = app.rngs.stream(key);
    spawn_heal_text_with(app, amount, rng);
}

/// W4-f37 — [`spawn_heal_text`] with the caller's stream — the nurse-heal
/// click (`NPCChatText_DoNurseHeal`, `Main.cs:40112`) draws the two
/// `CombatText.NewText` position jitters (`CombatText.cs:88-89`) on the
/// bare-`Main.rand` stand-in (`app.rngs.global()`, the S1-A-11 §4
/// click convention), NOT the UpdatePlayers swap.
pub(crate) fn spawn_heal_text_on(app: &mut App, amount: i32, rng: &mut tr_core::UnifiedRandom) {
pub(crate) fn spawn_heal_text(app: &mut App, amount: i32) {
/// bare-`Main.rand` stand-in (`app.rngs.global()`, the S1-A-11 §4 click
/// convention), NOT the UpdatePlayers swap. (batch-L compose: f49's
/// stream-KEY wrapper + f37's stream-REF core, one body.)
pub(crate) fn spawn_heal_text_with(app: &mut App, amount: i32, rng: &mut tr_core::UnifiedRandom) {
    let Some(p) = app.player.as_ref() else {
        return;
    };
    let (x, y, height) = (p.x, p.y, p.height);
    let text = amount.to_string();
    let size = match app.lookups.combat_text_font.as_ref() {
        Some(f) => [crate::ui::text::font_width(f, &text, 1.0), f.line_spacing],
        None => [text.chars().count() as f32 * 8.0, 10.0],
    };
    let location = crate::combat_text::Rect {
        x,
        y,
        width: crate::player::WIDTH,
        height,
    };
    crate::combat_text::spawn_amount(
        &mut app.combat_texts,
        location,
        crate::combat_text::HEAL_LIFE,
        amount,
        false,
        false,
        size,
        1.0,
        rng,
    );
}

// ---- witnesses (S1-P-i37) — WRITTEN NOT RUN (code-only swarm, no compiler) ----
//
// The RNG expectations below were derived from a JS mirror of
// `tr_core::UnifiedRandom` (the .NET subtractive stream) validated against
// the crate's own `GOLDEN_42` prefix (unified_random.rs:264-274) — the
// mirror reproduced all 8 pinned draws exactly before the seeds were
// picked. The audio fixtures pin the listener either silent (`sound_volume
// 0` — the attenuation gate refuses BEFORE the pitch draw, sound.rs:82) or
// on the player (audible — the Item3 style-3 default pitch arm draws
// `Next(-6, 7)`, sound.rs:307-316), so the draw counts are exact.

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::flow::front_flow_tests::flow_app;
    use crate::item::ItemSlot;
    use crate::player::{Player, PlayerInput};
    use tr_core::UnifiedRandom;

    fn slot(kind: u16, count: u32) -> ItemSlot {
        ItemSlot {
            kind,
            name: registry::name(kind),
            count,
            prefix: 0,
        }
    }

    /// A headless drink rig: the flow app's seed-0 streams, a spawned player
    /// at 100/100 life and 20/20 mana, the SILENT listener (no pitch draws).
    fn drink_app() -> App {
        let mut app = flow_app(None, vec![], vec![]);
        app.player = Some(Player::spawn(&app.world));
        app.audio.set_listener(crate::sound::Listener {
            camera_center: (0.0, 0.0),
            sound_volume: 0.0,
            ambient_volume: 1.0,
            quiet_ambient_sounds: false,
            game_menu: false,
            generating_or_loading: false,
            server: false,
        });
        app
    }

    fn player_with(kind: u16) -> Player {
        let w = crate::test_world_helpers::ground();
        let mut p = Player::spawn(&w);
        p.equipped[3] = kind;
        for _ in 0..2 {
            p.update(&w, PlayerInput::default());
        }
        p
    }

    // ---- PotionDelayTimes / the Item statics ------------------------------

    #[test]
    fn delay_times_rebuild_matches_the_item_statics() {
        // `Item.cs:49968-49971` + `Player.cs:25478-25480`.
        let t = PotionDelayTimes::rebuild(false);
        assert_eq!((t.potion, t.restoration, t.mushroom), (3600, 2700, 1800));
        assert_eq!(EGGNOG_DELAY, 2400);
    }

    #[test]
    fn p_stone_scales_the_three_rows_by_f32_truncation() {
        // `Player.cs:25481-25486`: `(int)((float)t * 0.75f)`.
        let t = PotionDelayTimes::rebuild(true);
        assert_eq!((t.potion, t.restoration, t.mushroom), (2700, 2025, 1350));
        // The eggnog field is NOT rebuilt/scaled (`:25478-25480` rebuilds
        // only the three; `eggnogDelayTime` keeps `Item.eggnogDelay`).
        assert_eq!(EGGNOG_DELAY, 2400);
    }

    // ---- ApplyPotionDelay (Player.cs:51853-51886) -------------------------

    #[test]
    fn apply_potion_delay_the_per_kind_rows() {
        let w = crate::test_world_helpers::ground();
        let mut rng = UnifiedRandom::new(0);
        let times = PotionDelayTimes::rebuild(false);
        let rows: [(u16, i32); 5] = [
            (28, 3600),   // the else arm `:51883`
            (227, 2700),  // restoration `:51866`
            (126, 2700),  // bottled water shares the restoration row
            (1912, 2400), // eggnog `:51871` (unscaled)
            (5, 1800),    // mushroom `:51877`
        ];
        for (kind, want) in rows {
            let mut p = Player::spawn(&w);
            let def = registry::get(kind).unwrap();
            apply_potion_delay(&mut p, def, &times, &mut rng, false, false);
            assert_eq!(p.potion_delay, want, "kind {kind}");
            let i = p.buffs.find_buff_index(POTION_SICKNESS).unwrap();
            assert_eq!(p.buffs.buff_time[i], want, "buff 21 rides the delay");
        }
    }

    #[test]
    fn apply_potion_delay_p_stone_scales_the_rows_not_eggnog() {
        let w = crate::test_world_helpers::ground();
        let mut rng = UnifiedRandom::new(0);
        let times = PotionDelayTimes::rebuild(true);
        for (kind, want) in [(28u16, 2700i32), (227, 2025), (5, 1350), (1912, 2400)] {
            let mut p = Player::spawn(&w);
            p.p_stone = true;
            apply_potion_delay(
                &mut p,
                registry::get(kind).unwrap(),
                &times,
                &mut rng,
                false,
                false,
            );
            assert_eq!(p.potion_delay, want, "kind {kind} under pStone");
        }
    }

    #[test]
    fn strange_brew_rolls_the_delay_first_and_scales_it() {
        // Seed 9 (JS-mirror derived): `Next(2400, 4201)` = 3170, then the
        // pStone truncation `(int)(3170f * 0.75f)` = 2377 (`:51857-51861`).
        let w = crate::test_world_helpers::ground();
        let mut p = Player::spawn(&w);
        let mut rng = UnifiedRandom::new(9);
        let mut twin = UnifiedRandom::new(9);
        let times = PotionDelayTimes::rebuild(false);
        apply_potion_delay(
            &mut p,
            registry::get(3001).unwrap(),
            &times,
            &mut rng,
            false,
            false,
        );
        assert_eq!(p.potion_delay, 3170);
        assert_eq!(p.potion_delay, twin.next_range(2400, 4201));
        assert_eq!(rng, twin, "exactly ONE draw at the delay");
        let i = p.buffs.find_buff_index(POTION_SICKNESS).unwrap();
        assert_eq!(p.buffs.buff_time[i], 3170);
        // pStone: same roll, ×0.75 truncated.
        let mut q = Player::spawn(&w);
        q.p_stone = true;
        let mut rng2 = UnifiedRandom::new(9);
        apply_potion_delay(
            &mut q,
            registry::get(3001).unwrap(),
            &times,
            &mut rng2,
            false,
            false,
        );
        assert_eq!(q.potion_delay, 2377);
    }

    // ---- ApplyLifeAndOrMana (Player.cs:51888-51952) -----------------------

    #[test]
    fn flat_heal_clamps_and_spawns_the_heal_text_amount() {
        let w = crate::test_world_helpers::ground();
        let mut p = Player::spawn(&w);
        p.hp = 90;
        let mut rng = UnifiedRandom::new(0);
        let got = apply_life_and_or_mana(&mut p, registry::get(188).unwrap(), &mut rng);
        assert_eq!(got, Some(100), "the TEXT keeps the raw num (`:51929`)");
        assert_eq!(p.hp, 100, "the life clamps at statLifeMax2 (`:51921`)");
        assert!(p.buffs.find_buff_index(MANA_SICKNESS).is_none());
        assert_eq!(rng, UnifiedRandom::new(0), "a flat heal draws NOTHING");
    }

    #[test]
    fn mana_potion_restores_clamps_and_grants_buff_94() {
        // The Lesser Mana Potion 110 (`Item.cs:2680-2692`): healMana 50,
        // NOT potion-flagged. `:51919-51938`.
        let w = crate::test_world_helpers::ground();
        let mut p = Player::spawn(&w);
        p.mana = 0;
        p.mana_max = 200;
        let mut rng = UnifiedRandom::new(0);
        let got = apply_life_and_or_mana(&mut p, registry::get(110).unwrap(), &mut rng);
        assert_eq!(got, None, "healLife 0 -> no HealLife text");
        assert_eq!(p.mana, 50);
        assert_eq!(p.mana_potion_delay, 0, "`:51937` (ManaPotionDelay off)");
        let i = p
            .buffs
            .find_buff_index(MANA_SICKNESS)
            .expect("buff 94, NOT 89");
        assert_eq!(p.buffs.buff_time[i], MANA_SICK_TIME);
        assert_eq!(p.take_mana_effect(), Some(50), "the ManaEffect queue");
        assert_eq!(p.potion_delay, 0, "no Potion Sickness from a mana potion");
        // The clamp arm (`:51925-51928`).
        let mut q = Player::spawn(&w);
        q.mana = 190;
        q.mana_max = 200;
        apply_life_and_or_mana(&mut q, registry::get(110).unwrap(), &mut rng);
        assert_eq!(q.mana, 200);
    }

    #[test]
    fn strange_brew_heal_roll_then_iframe_roll_in_order() {
        // Seed 9: delay 3170 (consumed by ApplyPotionDelay first), then
        // `Next(70, 121)` = 93 and `NextFloat()` = 0.0589 <= 0.1 -> 240
        // i-frames (`:51895-51915`). RNG ORDER: heal roll BEFORE the float.
        let w = crate::test_world_helpers::ground();
        let mut p = Player::spawn(&w);
        p.hp = 10;
        p.max_hp = 400; // headroom: the rolled heal must not clamp
        let mut rng = UnifiedRandom::new(9);
        let mut twin = UnifiedRandom::new(9);
        let times = PotionDelayTimes::rebuild(false);
        apply_potion_delay(
            &mut p,
            registry::get(3001).unwrap(),
            &times,
            &mut rng,
            false,
            false,
        );
        let got = apply_life_and_or_mana(&mut p, registry::get(3001).unwrap(), &mut rng);
        assert_eq!(got, Some(93));
        assert_eq!(p.hp, 103);
        assert_eq!(twin.next_range(2400, 4201), 3170);
        assert_eq!(twin.next_range(70, 121), 93);
        let f = twin.next_float();
        assert!(f <= 0.1, "the fixture seed rolls the 240 arm, got {f}");
        assert_eq!(rng, twin, "delay -> heal -> float, exactly three draws");
        assert!(p.immune);
        assert_eq!(p.immune_time, 240);
    }

    #[test]
    fn strange_brew_iframe_table_arms() {
        // Seeds 1/2/3 (exact-stream derived, gate V4 re-pin — the original
        // JS-mirror values 0.166/0.467/0.865 mis-modelled the subtractive
        // index pair): after the heal roll `Next(70,121)`, the floats are
        // 0.11074 / 0.40416 / 0.69758 -> the 120 / 60 / none arms
        // (`:51900-51911`).
        let w = crate::test_world_helpers::ground();
        for (seed, want) in [(1i32, 120i32), (2, 60), (3, 0)] {
            let mut p = Player::spawn(&w);
            let mut rng = UnifiedRandom::new(seed);
            apply_life_and_or_mana(&mut p, registry::get(3001).unwrap(), &mut rng);
            if want > 0 {
                assert_eq!(p.immune_time, want, "seed {seed}");
                assert!(p.immune);
            } else {
                assert!(!p.immune, "seed {seed}: >0.6 grants no i-frames");
            }
        }
    }

    // ---- the sickness arms + the pStone rescale ---------------------------

    #[test]
    fn mana_sickness_arm_assigns_the_quarter_scale_reduction() {
        // `:10324-10326`: `0.25f * buffTime / 300`.
        let w = crate::test_world_helpers::ground();
        let mut p = Player::spawn(&w);
        apply_mana_sickness_arm(&mut p, 300);
        assert!(p.mana_sick);
        assert!((p.mana_sick_reduction - 0.25).abs() < 1e-6);
        apply_mana_sickness_arm(&mut p, 150);
        assert!((p.mana_sick_reduction - 0.125).abs() < 1e-6);
    }

    #[test]
    fn magic_damage_tail_multiplies_the_sickness_and_heat() {
        // `:25900-25907` (the i19 fold's consumer; exported here).
        let w = crate::test_world_helpers::ground();
        let mut p = Player::spawn(&w);
        p.magic_damage = 1.2;
        p.mana_sick = true;
        p.mana_sick_reduction = 0.25;
        apply_magic_damage_sickness_tail(&mut p);
        assert!((p.magic_damage - 0.9).abs() < 1e-6, "1.2 * 0.75");
        // manaHeat stays 1 on the use path (no ApplyManaPotionManaHeat caller).
        assert_eq!(p.mana_heat, 1.0);
    }

    #[test]
    fn adjust_remaining_potion_sickness_rescales_both_directions() {
        // `:28939-28952`: pStone ? t*0.75 : t/0.75, truncated.
        let w = crate::test_world_helpers::ground();
        let mut p = Player::spawn(&w);
        p.buffs.add_buff(POTION_SICKNESS, 3600);
        p.p_stone = true;
        adjust_remaining_potion_sickness(&mut p);
        let i = p.buffs.find_buff_index(POTION_SICKNESS).unwrap();
        assert_eq!(p.buffs.buff_time[i], 2700);
        p.p_stone = false;
        adjust_remaining_potion_sickness(&mut p);
        let i = p.buffs.find_buff_index(POTION_SICKNESS).unwrap();
        assert_eq!(p.buffs.buff_time[i], 3600);
        // No buff: a quiet no-op.
        let mut q = Player::spawn(&w);
        adjust_remaining_potion_sickness(&mut q);
    }

    #[test]
    fn the_stone_family_grants_p_stone_and_a_change_rescales_the_live_buff() {
        // 535 (`:15574-15577`), 860 (`:15150-15153`), 6185 (`:14615-14620`).
        for kind in [535u16, 860, 6185] {
            let p = player_with(kind);
            assert!(p.p_stone, "kind {kind} arms pStone");
        }
        // Equip mid-sickness: the `:25640-25643` change hook rescales the
        // live buff-21 timer. Tick order: the buff decrements FIRST
        // (`update_buffs_tick`), the scan mirrors it, then the post-equip
        // adjust reads the decremented value — 3600 -> 3599 (tick 1) ->
        // 3598 (tick 2's decrement) -> `(int)(3598f * 0.75f)` = 2698.
        let w = crate::test_world_helpers::ground();
        let mut p = Player::spawn(&w);
        p.buffs.add_buff(POTION_SICKNESS, 3600);
        p.update(&w, PlayerInput::default()); // tick: 3599, no pStone
        assert_eq!(p.potion_delay, 3599, "the buff-21 mirror (`:11721`)");
        p.equipped[3] = 535;
        p.update(&w, PlayerInput::default());
        let i = p.buffs.find_buff_index(POTION_SICKNESS).unwrap();
        assert_eq!(p.buffs.buff_time[i], 2698, "(int)(3598f * 0.75f)");
        // Unequip: ÷0.75 truncation (2697 after tick 3's decrement -> 3596).
        p.equipped[3] = 0;
        p.update(&w, PlayerInput::default());
        let i = p.buffs.find_buff_index(POTION_SICKNESS).unwrap();
        assert_eq!(p.buffs.buff_time[i], 3596, "(int)(2697f / 0.75f)");
    }

    #[test]
    fn the_buff_94_scan_arms_the_sickness_and_the_reset_clears_it() {
        let w = crate::test_world_helpers::ground();
        let mut p = Player::spawn(&w);
        p.buffs.add_buff(MANA_SICKNESS, 300);
        p.update(&w, PlayerInput::default());
        assert!(p.mana_sick);
        // buffTime decremented to 299 before the scan read it.
        assert!((p.mana_sick_reduction - 0.25 * 299.0 / 300.0).abs() < 1e-5);
        // Expiry: the flag drops on the next tick (`ResetEffects` `:18592`).
        let i = p.buffs.find_buff_index(MANA_SICKNESS).unwrap();
        p.buffs.buff_time[i] = 1;
        p.update(&w, PlayerInput::default());
        assert!(!p.mana_sick, "the expired 94 grants nothing");
    }

    #[test]
    fn the_dead_tick_zeroes_both_sickness_clocks() {
        // `Player.UpdateDead` `:17018-17019`.
        let w = crate::test_world_helpers::ground();
        let mut p = Player::spawn(&w);
        p.potion_delay = 1234;
        p.mana_potion_delay = 42;
        p.update_dead(false, false, false, false);
        assert_eq!((p.potion_delay, p.mana_potion_delay), (0, 0));
    }

    // ---- the can-use gate --------------------------------------------------

    #[test]
    fn potion_use_blocked_reads_the_clock_not_the_buff_slot() {
        let w = crate::test_world_helpers::ground();
        let mut p = Player::spawn(&w);
        let heal = registry::get(188).unwrap();
        let buff = registry::get(292).unwrap();
        assert!(!potion_use_blocked(&p, heal));
        p.potion_delay = 1;
        assert!(potion_use_blocked(&p, heal), "`:52217-52220`");
        assert!(
            !potion_use_blocked(&p, buff),
            "buff potions are NOT potion-flagged"
        );
    }

    // ---- the ItemCheck drink seam (use_potion_item) ------------------------

    #[test]
    fn use_healing_potion_heals_grants_sickness_and_consumes_one() {
        let mut app = drink_app();
        app.inventory[0] = slot(188, 3);
        app.selected = 0;
        app.player.as_mut().unwrap().hp = 50;
        let held = app.inventory[0];
        assert!(use_potion_item(&mut app, held));
        let p = app.player.as_ref().unwrap();
        assert_eq!(p.hp, 100);
        assert_eq!(p.potion_delay, 3600);
        let i = p.buffs.find_buff_index(POTION_SICKNESS).unwrap();
        assert_eq!(p.buffs.buff_time[i], 3600);
        assert_eq!(app.inventory[0].count, 2);
        assert!(app.swing > 0.0, "the swing seam armed");
        assert!(app.attack_cd > 0.0);
        // The HealLife text spawned (silent listener: only the 2 jitter
        // draws moved the stream — seed 0: 5 then 14).
        let t = app.combat_texts.iter().find(|t| t.active).expect("heal text");
        assert_eq!(t.color, crate::combat_text::HEAL_LIFE);
        assert_eq!(t.text, "100");
        let mut twin = UnifiedRandom::new(0);
        twin.next_range(-10, 11);
        twin.next_range(-21, 22);
        assert_eq!(*app.rngs.stream(crate::app::RngKey::UpdatePlayers), twin);
    }

    #[test]
    fn use_is_blocked_while_the_sickness_clock_runs() {
        let mut app = drink_app();
        app.inventory[0] = slot(188, 3);
        app.selected = 0;
        app.player.as_mut().unwrap().hp = 50;
        app.player.as_mut().unwrap().potion_delay = 100;
        let held = app.inventory[0];
        assert!(!use_potion_item(&mut app, held));
        assert_eq!(app.player.as_ref().unwrap().hp, 50);
        assert_eq!(app.inventory[0].count, 3, "no consume on a refused use");
        assert_eq!(
            *app.rngs.stream(crate::app::RngKey::UpdatePlayers),
            UnifiedRandom::new(0),
            "a refused use draws nothing"
        );
    }

    #[test]
    fn buff_potions_grant_their_buff_and_drink_through_sickness() {
        // Ironskin 292: buffType 5, buffTime 28800 (`Item.cs:4858-4871`),
        // NOT potion-flagged — the sickness clock does not gate it.
        let mut app = drink_app();
        app.inventory[0] = slot(292, 2);
        app.selected = 0;
        app.player.as_mut().unwrap().potion_delay = 3000;
        let held = app.inventory[0];
        assert!(use_potion_item(&mut app, held));
        let p = app.player.as_ref().unwrap();
        let i = p.buffs.find_buff_index(5).expect("Ironskin granted");
        assert_eq!(p.buffs.buff_time[i], 28800, "the row's buffTime verbatim");
        assert_eq!(p.potion_delay, 3000, "no NEW sickness from a buff potion");
        assert_eq!(app.inventory[0].count, 1);
    }

    /// W4-e26 — the Luck Potion factor end to end: the three items
    /// (`Item.cs:37773-37811` — 4477/4478/4479, buff 257 at
    /// 18000/36000/54000) drink through the shared seam, the `UpdateBuffs`
    /// arm (`Player.cs:10285-10301`) tiers by REMAINING time (>36000 → 3,
    /// >18000 → 2, else 1), and `RecalculateLuck` weights the tier ×0.1
    /// (`:29140`). No new RNG draws — the buff grant itself is draw-free.
    #[test]
    fn w4_e26_the_luck_potions_feed_the_tiered_luck_factor() {
        for (kind, time, tier) in [(4477u16, 18000i32, 1u8), (4478, 36000, 2), (4479, 54000, 3)] {
            let mut app = drink_app();
            app.inventory[0] = slot(kind, 2);
            app.selected = 0;
            let held = app.inventory[0];
            assert!(use_potion_item(&mut app, held), "{kind} drinks");
            let p = app.player.as_ref().unwrap();
            let i = p.buffs.find_buff_index(257).expect("Lucky granted");
            assert_eq!(p.buffs.buff_time[i], time, "{kind}'s buffTime verbatim");
            assert_eq!(p.potion_delay, 0, "not potion-flagged, no sickness");
            // One player tick derives the tier + recalculates.
            let w = app.world.clone();
            let p = app.player.as_mut().unwrap();
            p.update(&w, PlayerInput::default());
            assert_eq!(p.luck_state.luck_potion, tier, "{kind} tiers to {tier}");
            let want = tier as f32 * 0.1;
            assert!(
                (p.luck() - want).abs() < 1e-6,
                "{kind}: luck {} vs {want}",
                p.luck()
            );
        }
        // The strict `>` boundaries (`:10289`/`:10293`): exactly 36000
        // tiers 2, exactly 18000 tiers 1 — the C# tests are strict. The
        // timer head (`:9869-9876`) decrements BEFORE the effect chain
        // reads, so seed time+1 to probe the exact boundary value.
        let w = crate::test_world_helpers::ground();
        for (time, tier) in [(36001, 3u8), (36000, 2), (18001, 2), (18000, 1), (60, 1)] {
            let mut p = Player::spawn(&w);
            p.buffs.add_buff(257, time + 1);
            p.update(&w, PlayerInput::default());
            assert_eq!(p.luck_state.luck_potion, tier, "remaining {time}");
        }
        // Expiry: `ResetEffects` zeroes `luckPotion` (`:18814`) and the
        // gone buff grants no tier — the factor drops to 0 next tick.
        let mut p = Player::spawn(&w);
        p.buffs.add_buff(257, 54000);
        p.update(&w, PlayerInput::default());
        assert_eq!(p.luck_state.luck_potion, 3);
        let i = p.buffs.find_buff_index(257).unwrap();
        p.buffs.buff_time[i] = 0;
        p.buffs.buff_type[i] = 0;
        p.update(&w, PlayerInput::default());
        assert_eq!(p.luck_state.luck_potion, 0, "expiry clears the tier");
        assert_eq!(p.luck(), 0.0);
    }

    #[test]
    fn use_mana_potion_grants_94_and_queues_the_mana_text() {
        let mut app = drink_app();
        app.inventory[0] = slot(110, 2);
        app.selected = 0;
        let p = app.player.as_mut().unwrap();
        p.mana = 0;
        p.mana_max = 200;
        let held = app.inventory[0];
        assert!(use_potion_item(&mut app, held));
        let p = app.player.as_mut().unwrap();
        assert_eq!(p.mana, 50);
        assert!(p.buffs.find_buff_index(MANA_SICKNESS).is_some());
        assert_eq!(p.take_mana_effect(), Some(50));
        assert_eq!(p.potion_delay, 0);
        assert_eq!(app.inventory[0].count, 1);
    }

    #[test]
    fn the_last_potion_turns_the_slot_to_air() {
        let mut app = drink_app();
        app.inventory[0] = slot(188, 1);
        app.selected = 0;
        app.player.as_mut().unwrap().hp = 50;
        let held = app.inventory[0];
        assert!(use_potion_item(&mut app, held));
        assert_eq!(app.inventory[0], crate::item::EMPTY_SLOT);
    }

    #[test]
    fn strange_brew_use_draws_delay_pitch_heal_float_jitter_in_order() {
        // AUDIBLE listener pinned on the player: the Item3 pitch draw
        // `Next(-6,7)` sits BETWEEN the delay roll and the heal roll
        // (`:42758` -> `:51680` -> `:42869`). Seed 0 (JS-mirror derived):
        // 3707, 4, 109, 0.558 (-> 60 i-frames), -6, 3.
        let mut app = drink_app();
        let (px, py) = {
            let p = app.player.as_ref().unwrap();
            (p.x, p.y)
        };
        app.audio.set_listener(crate::sound::Listener {
            camera_center: (px, py),
            sound_volume: 1.0,
            ambient_volume: 1.0,
            quiet_ambient_sounds: false,
            game_menu: false,
            generating_or_loading: false,
            server: false,
        });
        app.inventory[0] = slot(3001, 2);
        app.selected = 0;
        let p = app.player.as_mut().unwrap();
        p.hp = 10;
        p.max_hp = 400; // headroom: the rolled heal must not clamp
        let held = app.inventory[0];
        assert!(use_potion_item(&mut app, held));
        let p = app.player.as_ref().unwrap();
        assert_eq!(p.potion_delay, 3707);
        assert_eq!(p.hp, 119, "10 + the rolled 109");
        assert_eq!(p.immune_time, 60, "0.3 < 0.558 <= 0.6");
        let mut twin = UnifiedRandom::new(0);
        assert_eq!(twin.next_range(2400, 4201), 3707);
        assert_eq!(twin.next_range(-6, 7), 4);
        assert_eq!(twin.next_range(70, 121), 109);
        let f = twin.next_float();
        assert!(f > 0.3 && f <= 0.6);
        assert_eq!(twin.next_range(-10, 11), -6);
        assert_eq!(twin.next_range(-21, 22), 3);
        assert_eq!(*app.rngs.stream(crate::app::RngKey::UpdatePlayers), twin);
    }

    #[test]
    fn bottled_water_while_drowning_completes_the_achievement() {
        // `:42871-42874` -> HandleSpecialEvent(25) -> the "Use" flag
        // (`AchievementsHelper.cs:370-372`).
        let mut app = drink_app();
        app.inventory[0] = slot(126, 2);
        app.selected = 0;
        let p = app.player.as_mut().unwrap();
        p.hp = 50;
        p.breath = 0;
        let held = app.inventory[0];
        assert!(use_potion_item(&mut app, held));
        // The "Use" flag is shared (`AchievementsHelper.cs:347`'s
        // GetCondition(row, "Use")) — several rows complete; the pin is the
        // drowning row's membership, not an exclusive queue.
        let events = app.achievements.take_completed_events();
        assert!(
            events.contains(&"DRINK_BOTTLED_WATER_WHILE_DROWNING"),
            "the drowning row completed: {events:?}"
        );
        // Breath > 0: no event.
        let mut app2 = drink_app();
        app2.inventory[0] = slot(126, 2);
        app2.selected = 0;
        app2.player.as_mut().unwrap().hp = 50;
        let held2 = app2.inventory[0];
        assert!(use_potion_item(&mut app2, held2));
        assert!(app2.achievements.take_completed_events().is_empty());
    }

    #[test]
    fn the_cursed_buff_refuses_the_drink() {
        let mut app = drink_app();
        app.inventory[0] = slot(188, 2);
        app.selected = 0;
        app.player.as_mut().unwrap().hp = 50;
        app.player.as_mut().unwrap().buffs.add_buff(23, 600);
        let held = app.inventory[0];
        assert!(!use_potion_item(&mut app, held));
        assert_eq!(app.inventory[0].count, 2);
    }

    // ---- QuickHeal_GetItemToUse (Player.cs:5549-5596) ----------------------

    #[test]
    fn quick_heal_chooser_prefers_the_smallest_overheal() {
        let mut app = drink_app();
        let p = app.player.as_mut().unwrap();
        p.max_hp = 400;
        p.hp = 300; // missing 100
        app.inventory[0] = slot(28, 1); // 50 -> -50
        app.inventory[1] = slot(227, 1); // 90 -> -10 (+30 grace -> 0)
        app.inventory[2] = slot(499, 1); // 150 -> +50
        let loc = quick_heal_get_item_to_use(&app).expect("a potion found");
        assert_eq!(loc, QuickSlot::Inventory(1), "the 227 grace wins at 0");
    }

    #[test]
    fn quick_heal_chooser_falls_back_to_the_largest_underheal() {
        let mut app = drink_app();
        let p = app.player.as_mut().unwrap();
        p.max_hp = 400;
        p.hp = 100; // missing 300 — nothing covers it
        app.inventory[0] = slot(28, 1); // -250
        app.inventory[1] = slot(188, 1); // -200
        app.inventory[2] = slot(499, 1); // -150
        let loc = quick_heal_get_item_to_use(&app).expect("a potion found");
        assert_eq!(loc, QuickSlot::Inventory(2), "the largest under-heal");
    }

    #[test]
    fn quick_heal_chooser_scans_the_coin_ammo_rows_and_the_void_bag() {
        let mut app = drink_app();
        let p = app.player.as_mut().unwrap();
        p.hp = 50;
        app.equip.ammo[2] = slot(188, 1);
        assert_eq!(
            quick_heal_get_item_to_use(&app),
            Some(QuickSlot::Ammo(2)),
            "the 58-slot inventory spans the ammo row"
        );
        // The void bag only when enabled (`useVoidBag`, `:23171-23181`).
        let mut app2 = drink_app();
        let p2 = app2.player.as_mut().unwrap();
        p2.hp = 50;
        p2.banks[3][7] = slot(188, 1);
        assert_eq!(quick_heal_get_item_to_use(&app2), None, "bag disabled");
        app2.player.as_mut().unwrap().void_vault_enabled = true;
        assert_eq!(
            quick_heal_get_item_to_use(&app2),
            Some(QuickSlot::VoidBag(7))
        );
    }

    #[test]
    fn quick_heal_drinks_the_chosen_potion_and_consumes_it() {
        let mut app = drink_app();
        app.player.as_mut().unwrap().hp = 50;
        app.inventory[3] = slot(227, 2);
        assert!(quick_heal(&mut app));
        let p = app.player.as_ref().unwrap();
        assert_eq!(p.hp, 100, "50 + 90 clamped");
        assert_eq!(p.potion_delay, 2700, "the restoration row");
        assert_eq!(app.inventory[3].count, 1);
        // The HealLife text: silent listener, seed-0 jitter (5, 14).
        let t = app.combat_texts.iter().find(|t| t.active).expect("heal text");
        assert_eq!(t.text, "90");
        assert_eq!(t.color, crate::combat_text::HEAL_LIFE);
        // NO swing on the quick path (the C# QuickHeal has no animation).
        assert_eq!(app.swing, 0.0);
    }

    #[test]
    fn quick_heal_gates_full_life_sickness_dead_and_cursed() {
        // Full life (`:5503`).
        let mut app = drink_app();
        app.inventory[0] = slot(188, 1);
        assert!(!quick_heal(&mut app), "hp == max_hp blocks");
        // Sickness.
        let mut app2 = drink_app();
        app2.inventory[0] = slot(188, 1);
        let p = app2.player.as_mut().unwrap();
        p.hp = 50;
        p.potion_delay = 1;
        assert!(!quick_heal(&mut app2));
        // Dead.
        let mut app3 = drink_app();
        app3.inventory[0] = slot(188, 1);
        let p = app3.player.as_mut().unwrap();
        p.hp = 50;
        p.dead = true;
        assert!(!quick_heal(&mut app3));
        // Cursed (buff 23).
        let mut app4 = drink_app();
        app4.inventory[0] = slot(188, 1);
        let p = app4.player.as_mut().unwrap();
        p.hp = 50;
        p.buffs.add_buff(23, 600);
        assert!(!quick_heal(&mut app4));
        // The cursor stack (LocalPlayerHasPendingInventoryActions stand-in).
        let mut app5 = drink_app();
        app5.inventory[0] = slot(188, 1);
        app5.player.as_mut().unwrap().hp = 50;
        app5.mouse_item = slot(2, 1);
        assert!(!quick_heal(&mut app5));
    }

    // ---- QuickMana (Player.cs:5598-5649) -----------------------------------

    #[test]
    fn quick_mana_chooser_takes_the_first_heal_mana_item() {
        let mut app = drink_app();
        app.inventory[5] = slot(110, 1);
        app.inventory[2] = slot(189, 1);
        assert_eq!(
            quick_mana_get_item_to_use(&app),
            Some(QuickSlot::Inventory(2)),
            "slot order, no best-fit"
        );
        // While sick, potion-flagged healMana items are skipped (`:5633`) —
        // the mana potions are not potion-flagged, so they stay eligible.
        app.player.as_mut().unwrap().potion_delay = 100;
        assert_eq!(
            quick_mana_get_item_to_use(&app),
            Some(QuickSlot::Inventory(2))
        );
    }

    #[test]
    fn quick_mana_drinks_at_full_mana_under_the_shipped_mana_v2() {
        // `:5600`: `!ManaV2 && statMana == statManaMax2` — ManaV2 TRUE
        // (`DebugOptions.cs:85`), so the full-mana arm does NOT block.
        let mut app = drink_app();
        app.inventory[0] = slot(110, 2);
        let p = app.player.as_mut().unwrap();
        p.mana = p.mana_max; // full bar
        assert!(quick_mana(&mut app), "the V2 build drinks at full mana");
        let p = app.player.as_ref().unwrap();
        assert_eq!(p.mana, p.mana_max, "the clamp keeps the bar full");
        assert!(p.buffs.find_buff_index(MANA_SICKNESS).is_some());
        assert_eq!(app.inventory[0].count, 1, "the stack still drains");
    }

    #[test]
    fn quick_mana_gates_dead_cursed_and_the_mana_potion_delay() {
        let mut app = drink_app();
        app.inventory[0] = slot(110, 1);
        app.player.as_mut().unwrap().dead = true;
        assert!(!quick_mana(&mut app));
        let mut app2 = drink_app();
        app2.inventory[0] = slot(110, 1);
        app2.player.as_mut().unwrap().buffs.add_buff(23, 600);
        assert!(!quick_mana(&mut app2));
        let mut app3 = drink_app();
        app3.inventory[0] = slot(110, 1);
        app3.player.as_mut().unwrap().mana_potion_delay = 1;
        assert!(!quick_mana(&mut app3), "the gate stays for structure");
    }

    // ---- the hunger reset (Player.cs:12081-12150) --------------------------

    #[test]
    fn mushroom_and_eggnog_zero_the_hunger_timers() {
        let w = crate::test_world_helpers::ground();
        for kind in [5u16, 1912] {
            let mut p = Player::spawn(&w);
            p.buffs.add_buff(332, 10000);
            let mut rng = UnifiedRandom::new(0);
            let times = PotionDelayTimes::rebuild(false);
            apply_potion_delay(
                &mut p,
                registry::get(kind).unwrap(),
                &times,
                &mut rng,
                false,
                false,
            );
            // The reset zeroes the live hunger slot; the follow-up
            // UpdateHungerBuffs skips zeroed slots (`:12102-12105`), so no
            // 333 is granted — exactly the C# observable.
            assert!(p.buffs.find_buff_index(332).is_none(), "kind {kind}");
            assert!(p.buffs.find_buff_index(333).is_none());
        }
    }

    #[test]
    fn hunger_expiry_grants_the_next_tier_with_the_remix_row() {
        // `UpdateHungerBuffs` direct: a 332 at time 2 expires into 333 —
        // 18000 normally (`:12116`), 28800 under remix+dontStarve (`:12112`).
        // NOTE: `AddBuff(333)`'s IsFedState cleanup (`:5277-5286`, 332/333/334
        // are all fed-state) deletes the expiring 332 as it grants the 333,
        // so only the 333 survives — the dontStarve flag gates the blind
        // `buffTime[i] = 0` write, not the fed-state deletion.
        let w = crate::test_world_helpers::ground();
        let mut p = Player::spawn(&w);
        p.buffs.add_buff(332, 2);
        update_hunger_buffs(&mut p, false, false);
        let i = p.buffs.find_buff_index(333).unwrap();
        assert_eq!(p.buffs.buff_time[i], 18000);
        assert!(p.buffs.find_buff_index(332).is_none(), "the fed cleanup took it");
        let mut q = Player::spawn(&w);
        q.buffs.add_buff(332, 2);
        update_hunger_buffs(&mut q, true, true);
        let i = q.buffs.find_buff_index(333).unwrap();
        assert_eq!(q.buffs.buff_time[i], 28800, "the remix+dontStarve row");
    }

    // ---- W4-e10: the Hungry cue (Player.cs:12138) --------------------------

    /// W4-e10 — the 333-expiry flag: true ONLY on the `time <= 2` arm
    /// (`Player.cs:12127-12139`), false on a fresh 333 and on the 332 tier.
    #[test]
    fn w4_e10_the_hungry_flag_rides_only_the_333_expiry() {
        let w = crate::test_world_helpers::ground();
        let mut p = Player::spawn(&w);
        p.buffs.add_buff(333, 2);
        assert!(update_hunger_buffs(&mut p, false, false), "the expiry arm");
        assert!(p.buffs.find_buff_index(334).is_some(), "334 granted (:12135)");
        let mut q = Player::spawn(&w);
        q.buffs.add_buff(333, 1000);
        assert!(!update_hunger_buffs(&mut q, false, false), "a fresh 333");
        let mut r = Player::spawn(&w);
        r.buffs.add_buff(332, 2);
        assert!(!update_hunger_buffs(&mut r, false, false), "the 332 tier is silent");
    }

    /// W4-e10 — the Eggnog drink's reset fires the Hungry chime
    /// (`Player.cs:12138` via `ApplyPotionDelay` `:51873`): the audible
    /// listener plays it UNPOSITIONED (case-42 trackable), the gameplay
    /// stream sees only the Item3 use-sound pitch draw + the heal rolls —
    /// the façade adds NO gameplay draw. The silent twin plays nothing.
    #[test]
    fn w4_e10_the_eggnog_reset_plays_the_hungry_chime_draw_free() {
        // The audible rig: a live 333 at time 2, the listener on the player.
        let mut app = drink_app();
        let (px, py) = {
            let p = app.player.as_ref().unwrap();
            (p.x, p.y)
        };
        app.audio.set_listener(crate::sound::Listener {
            camera_center: (px, py),
            sound_volume: 1.0,
            ambient_volume: 1.0,
            quiet_ambient_sounds: false,
            game_menu: false,
            generating_or_loading: false,
            server: false,
        });
        app.inventory[0] = slot(1912, 2);
        app.selected = 0;
        let p = app.player.as_mut().unwrap();
        p.hp = 10;
        p.buffs.add_buff(333, 2);
        let before = app.audio.played_count();
        let mut mirror = app.rngs.stream(crate::app::RngKey::UpdatePlayers).clone();
        // The Eggnog delay is a constant (2400, no roll) and its heal is
        // the fixed 80 (`ApplyLifeAndOrMana` rolls ONLY for the Strange
        // Brew 3001) — the gameplay draws are the Item128 use-sound's
        // case-2 default pitch (`:51680`), then the heal text's jitter
        // pair; the Hungry façade between them draws NOTHING here.
        let held = app.inventory[0];
        assert!(use_potion_item(&mut app, held));
        assert_eq!(
            app.audio.played_count(),
            before + 2,
            "the Item use sound + the Hungry chime"
        );
        let p = app.player.as_ref().unwrap();
        assert_eq!(p.potion_delay, EGGNOG_DELAY);
        assert!(p.buffs.find_buff_index(334).is_some(), "the 333 expired");
        assert!(p.buffs.find_buff_index(333).is_none());
        // The gameplay stream twin: pitch draw, then the heal-text jitter
        // pair — NO draw for the trackable façade.
        let _ = mirror.next_range(-6, 7); // the Item128 use pitch (default arm)
        let _ = mirror.next_range(-10, 11); // the heal text jitter x
        let _ = mirror.next_range(-21, 22); // the heal text jitter y
        assert_eq!(
            *app.rngs.stream(crate::app::RngKey::UpdatePlayers),
            mirror,
            "the Hungry façade adds no gameplay draw"
        );
        // The silent twin: the gate refuses both plays, the stream matches
        // the draw-free chain (no pitch, the jitter pair only).
        let mut app2 = drink_app();
        app2.inventory[0] = slot(1912, 2);
        app2.selected = 0;
        let p = app2.player.as_mut().unwrap();
        p.hp = 10;
        p.buffs.add_buff(333, 2);
        let before2 = app2.audio.played_count();
        let mut mirror2 = app2.rngs.stream(crate::app::RngKey::UpdatePlayers).clone();
        assert!(use_potion_item(&mut app2, app2.inventory[0]));
        assert_eq!(app2.audio.played_count(), before2, "silent: no plays");
        let _ = mirror2.next_range(-10, 11);
        let _ = mirror2.next_range(-21, 22);
        assert_eq!(
            *app2.rngs.stream(crate::app::RngKey::UpdatePlayers),
            mirror2
        );
    }

    /// W4-e10 — the QuickHeal Mushroom path (`Player.cs:5522-5525` →
    /// `TryToResetHungerToNeutral` → `UpdateHungerBuffs` `:12138`): the
    /// reset's expiry chime plays once, unpositioned, draw-free.
    #[test]
    fn w4_e10_the_quick_heal_mushroom_reset_plays_hungry() {
        let mut app = drink_app();
        let (px, py) = {
            let p = app.player.as_ref().unwrap();
            (p.x, p.y)
        };
        app.audio.set_listener(crate::sound::Listener {
            camera_center: (px, py),
            sound_volume: 1.0,
            ambient_volume: 1.0,
            quiet_ambient_sounds: false,
            game_menu: false,
            generating_or_loading: false,
            server: false,
        });
        app.inventory[0] = slot(5, 2);
        let p = app.player.as_mut().unwrap();
        p.hp = 50;
        p.buffs.add_buff(333, 2);
        let before = app.audio.played_count();
        assert!(quick_heal(&mut app));
        assert_eq!(
            app.audio.played_count(),
            before + 2,
            "the Mushroom use sound + the Hungry chime"
        );
        let p = app.player.as_ref().unwrap();
        assert_eq!(p.potion_delay, 1800, "the mushroom delay row");
        assert!(p.buffs.find_buff_index(334).is_some());
    // ---- W4-f49: the mana-flower auto-drink lane + the sickness stacking ----
    // WRITTEN NOT RUN (code-only mode, no compiler invoked).

    /// `Player.CheckMana`'s flower lane (`Player.cs:52588-52609`): a short
    /// bar with the Mana Flower armed drinks the FIRST healMana potion
    /// (`QuickMana` `:52592`) and re-pays the scaled cost (`:52594-52601`).
    /// The Lesser Mana Potion 110 restores 50 capped at the 20 max; the
    /// Ruby Staff's 9-mana pay then leaves 11. The drink grants Mana
    /// Sickness 94 for 300 (`ApplyLifeAndOrMana` `:51938`) and consumes one.
    #[test]
    fn w4_f49_the_flower_drinks_and_repays_on_a_short_bar() {
        let mut app = drink_app();
        app.inventory[0] = slot(110, 3); // Lesser Mana Potion, healMana 50
        app.selected = 0;
        {
            let p = app.player.as_mut().unwrap();
            p.mana_flower = true;
            p.mana = 0; // short of the 9-mana pay
        }
        assert!(check_mana_with_flower(&mut app, 9, true, false));
        let p = app.player.as_ref().unwrap();
        assert_eq!(p.mana, 11, "50 restored capped at 20, then 9 paid");
        assert!(!p.slow_magic_use, "the drink covered the cost — no slow cast");
        let i = p.buffs.find_buff_index(MANA_SICKNESS).expect("the drink grants 94");
        assert_eq!(p.buffs.buff_time[i], 300);
        assert_eq!(app.inventory[0].count, 2, "one potion consumed");
    }

    /// The flower lane with NO potion available: the failed drink falls to
    /// the SAME ManaV2 empty arm as the plain lane (`:52602-52606`) — drain
    /// to 0, arm the slow cast, return true (never refuse).
    #[test]
    fn w4_f49_the_flower_without_a_potion_keeps_the_v2_empty_arm() {
        let mut app = drink_app();
        app.selected = 0; // empty inventory — no healMana item
        {
            let p = app.player.as_mut().unwrap();
            p.mana_flower = true;
            p.mana = 3; // short of 9, and nothing to drink
        }
        assert!(check_mana_with_flower(&mut app, 9, true, false));
        let p = app.player.as_ref().unwrap();
        assert_eq!(p.mana, 0, "the V2 empty arm drains the bar");
        assert!(p.slow_magic_use, "DoRunOutOfManaEffect armed");
        assert!(p.buffs.find_buff_index(MANA_SICKNESS).is_none(), "no drink, no 94");
    }

    /// `blockQuickMana = true` (CheckMana's third arg, `:52576`): the flower
    /// lane is skipped entirely — the plain V2 empty arm runs with NO drink
    /// even though the flower is armed and a potion sits in the inventory.
    #[test]
    fn w4_f49_block_quick_mana_skips_the_flower_drink() {
        let mut app = drink_app();
        app.inventory[0] = slot(110, 3);
        app.selected = 0;
        {
            let p = app.player.as_mut().unwrap();
            p.mana_flower = true;
            p.mana = 0;
        }
        assert!(check_mana_with_flower(&mut app, 9, true, true));
        let p = app.player.as_ref().unwrap();
        assert_eq!(p.mana, 0);
        assert!(p.slow_magic_use);
        assert_eq!(app.inventory[0].count, 3, "the blocked lane drinks nothing");
    }

    /// The enough-mana arm (`:52580-52586`) returns BEFORE the flower gate:
    /// a full bar pays the scaled cost and never drinks, even with the
    /// flower armed. The entry clear (`:52578`) drops a stale slowMagicUse.
    #[test]
    fn w4_f49_the_enough_mana_arm_never_drinks() {
        let mut app = drink_app();
        app.inventory[0] = slot(110, 3);
        app.selected = 0;
        {
            let p = app.player.as_mut().unwrap();
            p.mana_flower = true;
            p.mana = 20;
            p.slow_magic_use = true; // stale — the entry clear drops it
        }
        assert!(check_mana_with_flower(&mut app, 9, true, false));
        let p = app.player.as_ref().unwrap();
        assert_eq!(p.mana, 11, "20 − 9, no drink");
        assert!(!p.slow_magic_use, "the entry clear (:52578)");
        assert_eq!(app.inventory[0].count, 3, "no consume");
        assert!(p.buffs.find_buff_index(MANA_SICKNESS).is_none());
    }

    /// The mana-flower drink's `UndoSlowMagicUse` gate (`:51946`) sees
    /// `slowMagicUse == false`: CheckMana's entry clear (`:52578`) runs
    /// BEFORE the flower lane, so the mid-swing timer shrink never fires on
    /// the flower path (the C# order). A stale armed flag + a running swing
    /// must NOT shrink the timers through the flower drink.
    #[test]
    fn w4_f49_the_flower_drink_does_not_shrink_the_swing_timers() {
        let mut app = drink_app();
        app.inventory[0] = slot(110, 3);
        app.selected = 0;
        app.swing = 0.5;
        app.swing_max = 0.5;
        app.item_time = 0.5;
        {
            let p = app.player.as_mut().unwrap();
            p.mana_flower = true;
            p.mana = 0;
            p.slow_magic_use = true; // stale — cleared at the entry
        }
        assert!(check_mana_with_flower(&mut app, 9, true, false));
        // The entry clear dropped the flag before the drink, so
        // UndoSlowMagicUse's gate (`:51946`) refused — the timers are intact.
        assert!((app.swing - 0.5).abs() < 1e-6, "no 1/2.5 shrink");
        assert!((app.item_time - 0.5).abs() < 1e-6);
    }

    /// W4-f49 stacking census: the Mana Sickness grant stacks ADDITIVELY to
    /// the 600 cap (`AddBuff_TryUpdatingExistingBuffTime` `:5410-5418`,
    /// `BuffID.Sets.AddBuffTimeAdditivelyToCap[94] = 600`, `BuffID.cs:53`).
    /// Two drinks inside the window lift the timer 300 → 600; a third caps.
    /// The reduction is one ASSIGN per scan (`:10326`), so it can exceed
    /// 0.25 (up to 0.5 at the cap) but never compounds across ticks.
    #[test]
    fn w4_f49_mana_sickness_stacks_additively_to_the_six_hundred_cap() {
        let w = crate::test_world_helpers::ground();
        let mut p = Player::spawn(&w);
        p.buffs.add_buff(MANA_SICKNESS, MANA_SICK_TIME); // 300
        let i = p.buffs.find_buff_index(MANA_SICKNESS).unwrap();
        assert_eq!(p.buffs.buff_time[i], 300);
        p.buffs.add_buff(MANA_SICKNESS, MANA_SICK_TIME); // +300 → 600
        assert_eq!(p.buffs.buff_time[i], 600, "additive, not a refresh");
        p.buffs.add_buff(MANA_SICKNESS, MANA_SICK_TIME); // +300 → capped 600
        assert_eq!(p.buffs.buff_time[i], 600, "the 600 cap");
        // The scan assigns 0.25 × 600/300 = 0.5 — the stacked reduction.
        apply_mana_sickness_arm(&mut p, 600);
        assert!((p.mana_sick_reduction - 0.5).abs() < 1e-6);
        // One assign per tick: re-running the arm does NOT compound.
        apply_mana_sickness_arm(&mut p, 600);
        assert!((p.mana_sick_reduction - 0.5).abs() < 1e-6, "assign, not multiply");
        // 94 is absent from BuffTimeIsExtendedWithGameDifficulty — expert
        // worlds do not extend it (the add_buff path keeps 300).
        let mut q = Player::spawn(&w);
        q.buffs.expert_mode = true;
        q.buffs.add_buff(MANA_SICKNESS, MANA_SICK_TIME);
        let j = q.buffs.find_buff_index(MANA_SICKNESS).unwrap();
        assert_eq!(q.buffs.buff_time[j], 300, "no expert extension for 94");
    }

    /// W4-f49 lane 12 drink-while-sick census — the FULL interaction: mana
    /// potions are NOT `potion`-flagged (`Item.cs`'s 110 row sets no
    /// `potion`), so the Potion Sickness clock (`:52217-52220`) never gates
    /// them, and Mana Sickness itself has NO use gate anywhere in
    /// `ItemCheck` — a sick player drinks freely. Each drink's `AddBuff(94,
    /// 300)` (`:51938`) stacks ADDITIVELY to the 600 cap
    /// (`AddBuff_TryUpdatingExistingBuffTime` `:5410-5418`,
    /// `BuffID.cs:53`), the next buff scan assigns `0.25 × time/300`
    /// (`:10326` — up to 0.5 at the cap), and the recalc tail multiplies
    /// `magicDamage` by `1 − reduction` ONCE per tick (`:25900-25903`).
    /// The `manaPotionDelay = 0` write (`:51937`) keeps the QuickMana gate
    /// open (the retail `ManaPotionDelay` debug flag is false).
    /// WRITTEN NOT RUN (code-only).
    #[test]
    fn w4_f49_drinking_while_mana_sick_stacks_the_timer_and_deepens_the_cut() {
        let mut app = drink_app();
        app.inventory[0] = slot(110, 5);
        app.selected = 0;
        {
            let p = app.player.as_mut().unwrap();
            p.mana = 0;
            p.mana_max = 200;
        }
        // Drink 1 — grants 94 @ 300 even with a Potion Sickness clock up
        // (the mana potion is not potion-flagged, `:52217` never fires).
        app.player.as_mut().unwrap().potion_delay = 3000;
        let held = app.inventory[0];
        assert!(use_potion_item(&mut app, held));
        {
            let p = app.player.as_ref().unwrap();
            let i = p.buffs.find_buff_index(MANA_SICKNESS).unwrap();
            assert_eq!(p.buffs.buff_time[i], 300);
            assert_eq!(p.mana, 50);
            assert_eq!(p.mana_potion_delay, 0, ":51937 keeps QuickMana open");
        }
        // Drink 2 WHILE SICK — no gate refuses; the timer stacks to 600.
        app.swing = 0.0;
        app.attack_cd = 0.0;
        let held = app.inventory[0];
        assert!(use_potion_item(&mut app, held), "no sick-drink gate exists");
        {
            let p = app.player.as_ref().unwrap();
            let i = p.buffs.find_buff_index(MANA_SICKNESS).unwrap();
            assert_eq!(p.buffs.buff_time[i], 600, "additive to the cap");
            assert_eq!(p.mana, 100);
        }
        // Drink 3 — capped at 600, never beyond.
        app.swing = 0.0;
        app.attack_cd = 0.0;
        let held = app.inventory[0];
        assert!(use_potion_item(&mut app, held));
        {
            let p = app.player.as_ref().unwrap();
            let i = p.buffs.find_buff_index(MANA_SICKNESS).unwrap();
            assert_eq!(p.buffs.buff_time[i], 600, "the 600 cap");
        }
        // The scan + recalc: the stacked timer halves magic damage (0.5 at
        // the cap), ONE multiply per tick — not compounded. The 601 grant
        // leaves 600 after the tick-head decrement (`:9871-9873`), the
        // editing.rs i19 witness convention.
        let w = crate::test_world_helpers::ground();
        {
            let mut p = Player::spawn(&w);
            p.buffs.add_buff(MANA_SICKNESS, 601);
            p.update(&w, PlayerInput::default());
            assert!(
                (p.mana_sick_reduction - 0.5).abs() < 1e-4,
                "0.25 × 600/300 after the head decrement: {}",
                p.mana_sick_reduction
            );
            assert!(
                (p.magic_damage - 0.5).abs() < 1e-4,
                "the tail cut once: {}",
                p.magic_damage
            );
            // A second tick at the same timer re-derives from the reset 1.0
            // base — still 0.5, never 0.25 (no compounding).
            p.buffs.buff_time[p.buffs.find_buff_index(MANA_SICKNESS).unwrap()] = 601;
            p.update(&w, PlayerInput::default());
            assert!(
                (p.magic_damage - 0.5).abs() < 1e-4,
                "assign-per-tick, not multiply: {}",
                p.magic_damage
            );
        }
        // The QuickMana path drinks while sick too (the chooser's only
        // sickness gate is the POTION clock on potion-flagged rows).
        let mut app2 = drink_app();
        app2.inventory[0] = slot(110, 2);
        {
            let p = app2.player.as_mut().unwrap();
            p.mana = 0;
            p.mana_max = 200;
            p.buffs.add_buff(MANA_SICKNESS, 600);
            p.mana_sick = true;
            p.mana_sick_reduction = 0.5;
        }
        assert!(quick_mana(&mut app2), "QuickMana has no mana-sick gate");
        let p = app2.player.as_ref().unwrap();
        let i = p.buffs.find_buff_index(MANA_SICKNESS).unwrap();
        assert_eq!(p.buffs.buff_time[i], 600, "the quick drink stacks to the cap");
    }

    /// The flower drink's draws ride the PASSED stream key: the weapon path
    /// uses UpdatePlayers, the Last Prism interval pay uses UpdateProjectiles
    /// (`Main.cs:17775`). A pure mana potion (healLife 0) queues only the
    /// ManaEffect text (drained by the items pass, not here) and the silent
    /// listener spends no use-sound pitch — so the drink is draw-free on BOTH
    /// keys in this rig; the census pins that neither stream moves.
    #[test]
    fn w4_f49_the_flower_drink_rides_the_passed_stream_key() {
        let mut app = drink_app();
        app.inventory[0] = slot(110, 3);
        app.selected = 0;
        {
            let p = app.player.as_mut().unwrap();
            p.mana_flower = true;
            p.mana = 0;
        }
        assert!(check_mana_with_flower_on(
            &mut app,
            9,
            true,
            false,
            crate::app::RngKey::UpdateProjectiles,
        ));
        // healLife 0 → no HealEffect text; silent listener → no pitch draw.
        // The ManaEffect (healMana) rides `pending_mana_effect`, drained by
        // the items pass — not a stream draw here. Both keys stay at seed 0.
        assert_eq!(
            *app.rngs.stream(crate::app::RngKey::UpdateProjectiles),
            UnifiedRandom::new(0),
            "the mana-potion flower drink is draw-free under a silent listener"
        );
        assert_eq!(
            *app.rngs.stream(crate::app::RngKey::UpdatePlayers),
            UnifiedRandom::new(0),
            "UpdatePlayers untouched"
        );
        // The ManaEffect amount queued for the items-pass drain (the RAW 50).
        assert_eq!(app.player.as_mut().unwrap().take_mana_effect(), Some(50));
    }

    /// W4-f49 residuals — the mana-potion TIER census: the four rows'
    /// healMana against the C# `SetDefaults` arms (ItemID.cs:1903/2061/
    /// 2683/6101 → Item.cs:2680-2691/3701-3713/7336-7348/22346-22358):
    /// 110 Lesser 50, 189 Mana 100, 500 Greater 200, 2209 Super 400. All
    /// four are useStyle 9 / consumable / useTurn, NONE `potion`-flagged
    /// (no Potion Sickness — only Mana Sickness 94 via `:51938`), and the
    /// drink through `ApplyLifeAndOrMana` restores the RAW amount capped
    /// at `statManaMax2` (`:51920-51928`), queues the RAW ManaEffect
    /// (`:51950`) and keeps `manaPotionDelay = 0` (`:51937`). WRITTEN NOT
    /// RUN (code-only).
    #[test]
    fn w4_f49_the_mana_potion_tiers_match_the_csharp_rows() {
        // (id, C# healMana, Item.cs SetDefaults line).
        let rows = [(110u16, 50i32), (189, 100), (500, 200), (2209, 400)];
        for (id, heal) in rows {
            let def = registry::get(id).expect("the generated row exists");
            assert_eq!(def.heal_mana, heal, "item {id}: the C# healMana");
            assert_eq!(def.heal_life, 0, "item {id}: mana-only");
            assert_eq!(def.use_style, 9, "item {id}: the drink style");
            assert!(def.consumable && def.use_turn, "item {id}");
            assert_eq!(def.use_time, 17, "item {id}");
            assert_eq!(def.use_animation, 17, "item {id}");
            assert!(!def.potion, "item {id}: NOT potion-flagged (no 21)");
            // The drink: cap at max, RAW ManaEffect, sickness 300, delay 0.
            let w = crate::test_world_helpers::ground();
            let mut p = Player::spawn(&w);
            p.mana_max = 300;
            p.mana = 300 - heal / 2; // the cap eats half the restore
            let mut rng = UnifiedRandom::new(0);
            let heal_text = apply_life_and_or_mana(&mut p, def, &mut rng);
            assert_eq!(heal_text, None, "item {id}: no HealLife text");
            assert_eq!(p.mana, 300, "item {id}: capped at statManaMax2");
            assert_eq!(p.take_mana_effect(), Some(heal), "item {id}: the RAW text");
            let i = p.buffs.find_buff_index(MANA_SICKNESS).expect("94 granted");
            assert_eq!(p.buffs.buff_time[i], MANA_SICK_TIME);
            assert!(p.buffs.find_buff_index(POTION_SICKNESS).is_none());
            assert_eq!(p.mana_potion_delay, 0, ":51937");
            assert!(drinkable(def), "item {id}: rides the drink seam");
        }
        // The chooser takes the FIRST healMana row in slot order — a Super
        // behind a Lesser does NOT win (QuickMana_GetItemToUse :5629-5649
        // is first-fit, not best-fit).
        let mut app = drink_app();
        app.inventory[0] = slot(110, 1);
        app.inventory[1] = slot(2209, 1);
        {
            let p = app.player.as_mut().unwrap();
            p.mana_max = 500;
            p.mana = 0;
        }
        assert!(quick_mana(&mut app));
        assert_eq!(app.player.as_ref().unwrap().mana, 50, "the Lesser won");
        assert_eq!(app.inventory[0].count, 0);
        assert_eq!(app.inventory[1].count, 1, "the Super untouched");
    }

    /// W4-f49 MANA FINAL item 2 — the five R1-census CheckMana sites'
    /// pay shapes, armed through the SHARED machinery via the
    /// constants-copy (the d-lane AI bodies read `registry::item(...).mana`,
    /// never a hand-typed number; the constants are the witness pins).
    /// The d-lanes LANDED three of the five bodies on their own branches
    /// (d21 AI_208 Glacier Fang 1115, d19 AI_185 Life Drain 476, d10
    /// AI_100 Medusa Head 535) — each already calls `regen::check_mana`;
    /// the 439 Laser Machinegun (AI_075's holder arm) and the
    /// CommandForbiddenStorm stay their lanes'. This witness pins the
    /// pay contract those bodies ride: the generated row's mana, the
    /// `manaCost` scale, the shipped ManaV2 empty arm (drain to 0, slow
    /// cast, never refuse), and the flower lane's full re-check.
    /// WRITTEN NOT RUN (code-only).
    #[test]
    fn w4_f49_the_five_checkmana_sites_pay_through_the_shared_machine() {
        // The four held-row sites: (item id, the constants-copy pin).
        let rows = [
            (2795u16, LASER_MACHINEGUN_MANA), // 439 holder, AI_075 :66204
            (6153, GLACIER_FANG_MANA),        // 1115, AI_208 :36943 (d21)
            (3006, SOUL_DRAIN_MANA),          // 476, AI_185 :42662 (d19)
            (3269, MEDUSA_HEAD_MANA),         // 535, AI_100 :44567 (d10)
        ];
        for (id, pin) in rows {
            let def = registry::get(id).expect("the generated row exists");
            assert_eq!(def.mana, pin, "item {id}: the constants-copy pin");
            // The enough-mana pay scales by manaCost (`:52579`).
            let mut mana = 100;
            let mut slow = false;
            assert!(crate::regen::check_mana(&mut mana, &mut slow, pin, 0.5, true));
            assert_eq!(mana, 100 - (pin as f32 * 0.5) as i32, "item {id}: scaled");
            assert!(!slow, "the exact pay never arms the slow cast");
            // The short bar takes the ManaV2 empty arm (`:52610-52614`):
            // drain to 0, arm the slow cast, return true (never refuse).
            let mut mana = 1;
            let mut slow = false;
            assert!(crate::regen::check_mana(&mut mana, &mut slow, pin, 1.0, true));
            assert_eq!(mana, 0, "item {id}: the V2 empty arm drains");
            assert!(slow, "item {id}: DoRunOutOfManaEffect armed");
        }
        // The flower lane's full re-check (the pass-tail drain convention):
        // a short bar with the flower armed drinks the first healMana
        // potion and re-pays. The Soul Drain's 10-mana pay at manaCost 1
        // needs 10; the Lesser 50 restores capped at 20, then 10 paid → 10.
        let mut app = drink_app();
        app.inventory[0] = slot(110, 2);
        app.selected = 0;
        {
            let p = app.player.as_mut().unwrap();
            p.mana_flower = true;
            p.mana = 0;
        }
        assert!(check_mana_with_flower_on(
            &mut app,
            SOUL_DRAIN_MANA,
            true,
            false,
            crate::app::RngKey::UpdateProjectiles,
        ));
        let p = app.player.as_ref().unwrap();
        assert_eq!(p.mana, 10, "50 capped at 20, then the 10-mana pay");
        assert!(!p.slow_magic_use, "the drink covered the cost");
        // CommandForbiddenStorm's FLAT 20 pay (`Player.cs:56870`) — NOT a
        // held-row read — plus the ApplyManaRegenerationDelay follow-up
        // (`:56873`). The storm command's lane owns the spawn; this pins
        // the pay + the delay reset the constants-copy carries.
        let mut mana = 100;
        let mut slow = false;
        assert!(crate::regen::check_mana(
            &mut mana,
            &mut slow,
            FORBIDDEN_STORM_MANA,
            1.0,
            true,
        ));
        assert_eq!(mana, 80, "the flat 20 pay");
        let mut st = crate::regen::RegenState::default();
        crate::regen::apply_mana_regen_delay(&mut st, 200, mana, false);
        assert_eq!(st.mana_regen_delay, 4.0, "the V2 pin after the pay (:56873)");
    }
}