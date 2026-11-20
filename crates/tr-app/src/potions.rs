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
//! (`Player.cs:5766-6017`) lands with W4-a5: the food-priority ladder
//! (`quick_buff_pick_best_food_item`), the slot-order buff ladder and the
//! last-used `UseSound` play (`quick_buff`). The `ShouldUseItem` /
//! `ShouldBotherUsingThisBuff` exclusions + the `:5879-5890` mana-pay are
//! the W4-a5 phase-2 slice (the phase-1 ladder grants verbatim).
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
pub fn apply_potion_delay(
    p: &mut Player,
    def: &ItemDef,
    times: &PotionDelayTimes,
    rng: &mut UnifiedRandom,
    dont_starve: bool,
    remix: bool,
) {
    if def.id == 3001 {
        // `:51857`: `Next(2400, 4200 + 1)` — the upper bound is EXCLUSIVE in
        // the C# `Next(min, max)`, the +1 makes 4200 reachable.
        p.potion_delay = rng.next_range(2400, 4200 + 1);
        if p.p_stone {
            // `:51860`: `(int)((float)potionDelay * 0.75f)`.
            p.potion_delay = (p.potion_delay as f32 * PHILOSOPHER_STONE_MULTIPLIER) as i32;
        }
        p.buffs.add_buff(POTION_SICKNESS, p.potion_delay); // `:51862`
    } else if def.id == 227 || def.id == 126 {
        p.potion_delay = times.restoration; // `:51866`
        p.buffs.add_buff(POTION_SICKNESS, p.potion_delay); // `:51867`
    } else if def.id == 1912 {
        p.potion_delay = EGGNOG_DELAY; // `:51871` (`eggnogDelayTime`, unscaled)
        p.buffs.add_buff(POTION_SICKNESS, p.potion_delay); // `:51872`
        try_to_reset_hunger_to_neutral(p, dont_starve, remix); // `:51873`
    } else if def.id == 5 {
        p.potion_delay = times.mushroom; // `:51877`
        p.buffs.add_buff(POTION_SICKNESS, p.potion_delay); // `:51878`
        try_to_reset_hunger_to_neutral(p, dont_starve, remix); // `:51879`
    } else {
        p.potion_delay = times.potion; // `:51883`
        p.buffs.add_buff(POTION_SICKNESS, p.potion_delay); // `:51884`
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
/// the world seed flags the caller hands in; the emote/sound halves are
/// UI/audio seams (out of scope, the buff writes are the state change).
/// NOTE: like the C#, the follow-up `UpdateHungerBuffs` skips every slot
/// whose timer was just zeroed (`:12102-12105`) — the call is a structural
/// no-op on the reset slots, kept for the source shape.
pub fn try_to_reset_hunger_to_neutral(p: &mut Player, dont_starve: bool, remix: bool) {
    let mut flag = false;
    for i in 0..crate::buffs::MAX_BUFFS {
        let bt = p.buffs.buff_type[i];
        if bt > 0 && p.buffs.buff_time[i] > 0 && matches!(bt, 332 | 333 | 334) {
            p.buffs.buff_time[i] = 0; // `:12088`
            flag = true; // `:12089`
        }
    }
    if flag {
        update_hunger_buffs(p, dont_starve, remix); // `:12092-12095`
    }
}

/// `Player.UpdateHungerBuffs` (`Player.cs:12098-12150`). The port snapshots
/// the live (type, time) pairs first because `add_buff` compacts slots — the
/// observable writes (the expiry grants + the zeroing) land on the same buff
/// TYPES. The 332 expiry's remix+dontStarve arm grants 28800 (`:12110-12113`),
/// every other world 18000 (`:12116`).
fn update_hunger_buffs(p: &mut Player, dont_starve: bool, remix: bool) {
    let slots: Vec<(i32, i32)> = (0..crate::buffs::MAX_BUFFS)
        .filter(|&i| p.buffs.buff_type[i] > 0 && p.buffs.buff_time[i] > 0)
        .map(|i| (p.buffs.buff_type[i], p.buffs.buff_time[i]))
        .collect();
    for (bt, time) in slots {
        // Re-find by type: an earlier AddBuff may have compacted the slot.
        let Some(slot) = p.buffs.find_buff_index(bt) else {
            continue;
        };
        match bt {
            332 => {
                if time <= 2 {
                    // `:12108-12117`: remix AND dontStarve → 28800, else 18000.
                    let t = if remix && dont_starve { 28800 } else { 18000 };
                    p.buffs.add_buff(333, t);
                }
                if !dont_starve {
                    // Re-find AFTER the add: `AddBuff(333)`'s fed-state
                    // cleanup (`:5277-5286`, 332/333/334 are all IsFedState)
                    // may have deleted + compacted the 332 slot — the C#
                    // writes `buffTime[i]` on whatever now occupies `i`,
                    // the port's type re-find lands the same zeroing on the
                    // live 332 when it survived (the dontStarve arm).
                    if let Some(s) = p.buffs.find_buff_index(332) {
                        p.buffs.buff_time[s] = 0; // `:12120-12123`
                    }
                }
            }
            333 => {
                if time <= 2 {
                    p.buffs.add_buff(334, 5); // `:12131`/`:12135` (both arms 5)
                }
                if !dont_starve {
                    if let Some(s) = p.buffs.find_buff_index(333) {
                        p.buffs.buff_time[s] = 0; // `:12140-12143`
                    }
                }
            }
            334 => {
                if !dont_starve {
                    p.buffs.buff_time[slot] = 0; // `:12145-12148`
                }
            }
            _ => {}
        }
    }
}

/// The buff-94 Mana Sickness effect arm (`Player.cs:10318-10327`):
/// `manaSick = true` and `manaSickReduction = 0.25f * buffTime /
/// manaSickTime`. The `DebugOptions.ManaPotionDelay` mirror (`:10320-10323`)
/// is constants-off (false retail static). Called from the per-tick buff
/// scan for every live 94 slot; the LAST live slot wins, like the C# chain.
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
    {
        let Some(p) = app.player.as_mut() else {
            return false;
        };
        let times = PotionDelayTimes::rebuild(p.p_stone);
        let rng = app.rngs.stream(crate::app::RngKey::UpdatePlayers);
        if def.potion {
            apply_potion_delay(p, def, &times, rng, dont_starve, remix);
        }
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
        // (`AchievementsHelper.cs:370-372`).
        let _ = app.achievements.notify_custom("Use");
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
    let (heal_text, drowning) = {
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
        if def.potion {
            apply_potion_delay(p, def, &times, rng, dont_starve, remix); // `:5517-5520`
        }
        let heal_text = apply_life_and_or_mana(p, def, rng); // `:5521`
        if def.id == 5 {
            try_to_reset_hunger_to_neutral(p, dont_starve, remix); // `:5522-5525`
        }
        // `:5526-5534`: the buff grant WITH the buffTime 0 → 3600 fallback.
        if def.buff_type > 0 {
            let time = if def.buff_time == 0 { 3600 } else { def.buff_time };
            p.buffs.add_buff(def.buff_type, time);
        }
        let drowning = def.id == 126 && p.breath == 0; // `:5543-5546`
        (heal_text, drowning)
    };
    if let Some(amount) = heal_text {
        spawn_heal_text(app, amount);
    }
    // `:5535-5542`: `CanConsumeConsumableItem` is true → stack--, TurnToAir.
    loc.consume(app);
    if drowning {
        let _ = app.achievements.notify_custom("Use");
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
    let heal_text = {
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
            let rng = app.rngs.stream(crate::app::RngKey::UpdatePlayers);
            app.audio.play_use_sound(idx, x, y, pitch, rng); // `:5613`
        }
        let rng = app.rngs.stream(crate::app::RngKey::UpdatePlayers);
        if def.potion {
            apply_potion_delay(p, def, &times, rng, dont_starve, remix); // `:5614-5617`
        }
        apply_life_and_or_mana(p, def, rng) // `:5618`
    };
    if let Some(amount) = heal_text {
        spawn_heal_text(app, amount);
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
fn spawn_heal_text(app: &mut App, amount: i32) {
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
    let rng = app.rngs.stream(crate::app::RngKey::UpdatePlayers);
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

// ---- QuickBuff (W4-a5 phase 1: Player.cs:5766-5870) -----------------------
//
// The B-key buff hotkey. Phase 1 ports the ladder proper: the gates, the
// food-priority pick (`QuickBuff_PickBestFoodItem` :5918-5963 + the
// `FindFoodPriority` table :5965-5974), the 58-slot inventory scan, the
// void-bag scan and the last-used `UseSound` play (:5842-5845). The
// `ShouldUseItem` slice here is the :5869 row gates + the `TryStartUse`
// can-use gate reduced to the potion arm (the `quick_heal` precedent);
// the :5878-5914 exclusions (ShouldBotherUsingThisBuff, the mana pay, the
// pet/603 refusals, the buff-27 trio roll) are the W4-a5 phase-2 slice.

/// `Player.QuickBuff_FindFoodPriority` (`Player.cs:5965-5974`): the three
/// Well Fed tiers rank 1/2/3 (the [`crate::buffs::is_well_fed`] set),
/// everything else 0.
fn quick_buff_find_food_priority(buff_type: i32) -> i32 {
    match buff_type {
        26 => 1,
        206 => 2,
        207 => 3,
        _ => 0,
    }
}

/// `Player.QuickBuff_PickBestFoodItem` (`Player.cs:5918-5963`): the live
/// well-fed tier sets the floor (`num = priority + 1`, so only a STRICTLY
/// higher tier re-feeds), then the 58 inventory slots + the void bag are
/// scanned for the best food: priority ≥ floor, and among equals the
/// LONGER `buffTime` wins (`item.buffTime < item2.buffTime || num3 > num`).
/// The port's 58 slots span the grid + coin + ammo rows (the `quick_heal`
/// convention); `useVoidBag` reads the per-tick `void_vault_enabled`
/// refresh (`Player.cs:24550-24553`).
pub fn quick_buff_pick_best_food_item(app: &App) -> Option<QuickSlot> {
    let p = app.player.as_ref()?;
    let mut num = 0; // `:5920`
    for i in 0..crate::buffs::MAX_BUFFS {
        if p.buffs.buff_time[i] >= 1 {
            let num2 = quick_buff_find_food_priority(p.buffs.buff_type[i]);
            if num <= num2 {
                num = num2 + 1; // `:5927-5930`
            }
        }
    }
    // (loc, the stored item's buffTime, its priority) — the C# keeps the
    // Item reference; the port keeps the two fields the compare reads.
    let mut best: Option<(QuickSlot, i32, i32)> = None;
    let mut consider = |slot: ItemSlot, loc: QuickSlot| {
        if slot.count == 0 || slot.kind == 0 {
            return; // `!item2.IsAir` (`Item.cs:385-395`)
        }
        let Some(def) = registry::get(slot.kind) else {
            return;
        };
        let num3 = quick_buff_find_food_priority(def.buff_type);
        // `:5939`: num3 > 0 && num3 >= num && (item == null ||
        // item.buffTime < item2.buffTime || num3 > num).
        let take = num3 > 0
            && num3 >= num
            && match best {
                None => true,
                Some((_, best_time, _)) => best_time < def.buff_time || num3 > num,
            };
        if take {
            best = Some((loc, def.buff_time, num3));
            num = num3;
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
    best.map(|(loc, _, _)| loc)
}

/// `Player.QuickBuff_ShouldUseItem` (`Player.cs:5866-5916`) — the W4-a5
/// phase-1 slice: the `:5869` row gates (stack/type/buffType/summon) and
/// the `:5873` `ItemCheck_TryStartUse` can-use gate reduced to the potion
/// arm (the `quick_heal` precedent — buff potions are NOT potion-flagged,
/// `Item.cs:4858-4871`, so they pass through sickness). Returns the
/// `btype` the C# writes through its `out` param. The `:5878-5914`
/// exclusions + mana pay land with phase 2.
fn quick_buff_should_use_item(app: &App, loc: QuickSlot) -> Option<i32> {
    let slot = loc.slot(app);
    if slot.count == 0 || slot.kind == 0 {
        return None; // `:5869` stack <= 0 || type <= 0
    }
    let def = registry::get(slot.kind)?;
    if def.buff_type <= 0 || def.summon {
        return None; // `:5869` buffType <= 0 || summon
    }
    let p = app.player.as_ref()?;
    if potion_use_blocked(p, def) {
        return None; // `:5873`
    }
    Some(def.buff_type) // `:5877`
}

/// `Player.QuickBuff_UseItemForBuff` (`Player.cs:5848-5864`): the buffTime
/// 0 → 3600 fallback (`:5850-5854`), `AddBuff(btype, num)` and the
/// consumable stack-- / TurnToAir (`:5856-5863`).
fn quick_buff_use_item_for_buff(app: &mut App, loc: QuickSlot, btype: i32) {
    let slot = loc.slot(app);
    let Some(def) = registry::get(slot.kind) else {
        return;
    };
    let time = if def.buff_time == 0 { 3600 } else { def.buff_time };
    if let Some(p) = app.player.as_mut() {
        p.buffs.add_buff(btype, time); // `:5855`
    }
    if def.consumable {
        loc.consume(app); // `:5856-5863`
    }
}

/// One `QuickBuff_ShouldUseItem` → dismount → sound-record → use step of
/// the `:5803-5819`/`:5824-5840` loops. Returns whether the item was used
/// (the C# loop's `if` body ran).
fn quick_buff_ladder_step(
    app: &mut App,
    loc: QuickSlot,
    last_sound: &mut Option<(u16, f32)>,
) -> bool {
    let Some(btype) = quick_buff_should_use_item(app, loc) else {
        return false;
    };
    let held = loc.slot(app);
    if let Some(p) = app.player.as_mut() {
        // `:5808-5811`/`:5829-5832`: `Mount.DismountsOnItemUse` →
        // `TryEarlyDismount` (the mount helper carries its own gates).
        p.try_early_dismount(&app.world);
    }
    if let Some(def) = registry::get(held.kind) {
        // `:5812`/`:5833`: `legacySoundStyle = item2.UseSound` — the LAST
        // used item's sound wins; it plays once at the tail (`:5842`).
        *last_sound = def.use_sound.map(|idx| (idx, def.use_sound_pitch));
    }
    quick_buff_use_item_for_buff(app, loc, btype); // `:5813`/`:5834`
    true
}

/// The `CountBuffs() == Player.maxBuffs` full-bar read (`Player.cs:5773`,
/// `:5800`, `:5814`, `:5821`, `:5835`) — the leading-run count, NOT a
/// census (see [`crate::buffs::BuffSlots::count_buffs`]).
fn quick_buff_bar_full(app: &App) -> bool {
    app.player
        .as_ref()
        .is_some_and(|p| p.buffs.count_buffs() == crate::buffs::MAX_BUFFS as i32)
}

/// `Player.QuickBuff` (`Player.cs:5766-5846`): the B-key buff hotkey. The
/// gates (`:5768`): pending inventory actions (the cursor-stack stand-in)
/// / cursed / CCed / dead / spectating (no port field — single-player,
/// constants-off like `recall_teleport.rs`'s note) / the full buff bar.
/// Then the best food (`:5777-5799`, the TryStartUse gate, the early
/// dismount, the buff grant WITH the 3600 fallback, the consume), the
/// 58-slot buff ladder (`:5800-5820`) and the void-bag ladder
/// (`:5821-5841`), each breaking at the full bar. The LAST used item's
/// `UseSound` plays ONCE at `base.position` (`:5842-5845`, the top-left
/// like the other quick-use paths). Returns true when any item was used
/// (the C# is void; the bool is the port's latch-witness seam).
pub fn quick_buff(app: &mut App) -> bool {
    {
        let Some(p) = app.player.as_ref() else {
            return false;
        };
        if app.mouse_item.count > 0
            || p.buffs.find_buff_index(23).is_some()
            || p.cced()
            || p.dead
        {
            return false; // `:5768-5771`
        }
        if p.buffs.count_buffs() == crate::buffs::MAX_BUFFS as i32 {
            return false; // `:5773-5776`
        }
    }
    let mut used = false;
    let mut last_sound: Option<(u16, f32)> = None; // `LegacySoundStyle`
    // The food arm (`:5777-5799`).
    if let Some(loc) = quick_buff_pick_best_food_item(app) {
        let held = loc.slot(app);
        if let Some(def) = registry::get(held.kind) {
            let can_use = app
                .player
                .as_ref()
                .is_some_and(|p| !potion_use_blocked(p, def)); // `:5778`
            if can_use {
                if let Some(p) = app.player.as_mut() {
                    p.try_early_dismount(&app.world); // `:5780-5783`
                }
                // `:5784`: `legacySoundStyle = item.UseSound`.
                last_sound = def.use_sound.map(|idx| (idx, def.use_sound_pitch));
                // `:5785-5798`: the grant + consume (the same body as
                // UseItemForBuff, with the item's own buffType).
                quick_buff_use_item_for_buff(app, loc, def.buff_type);
                used = true;
            }
        }
    }
    // The 58-slot ladder (`:5800-5820`): the port's grid + coin + ammo
    // rows in C# inventory order.
    let inv_n = app.inventory.len();
    let coin_n = app.equip.coins.len();
    let ammo_n = app.equip.ammo.len();
    if !quick_buff_bar_full(app) {
        for i in 0..(inv_n + coin_n + ammo_n) {
            let loc = if i < inv_n {
                QuickSlot::Inventory(i)
            } else if i < inv_n + coin_n {
                QuickSlot::Coin(i - inv_n)
            } else {
                QuickSlot::Ammo(i - inv_n - coin_n)
            };
            if quick_buff_ladder_step(app, loc, &mut last_sound) {
                used = true;
                if quick_buff_bar_full(app) {
                    break; // `:5814-5817`
                }
            }
        }
    }
    // The void-bag ladder (`:5821-5841`), gated on `useVoidBag()`.
    let bag_enabled = app.player.as_ref().is_some_and(|p| p.void_vault_enabled);
    let bag_n = app.player.as_ref().map_or(0, |p| p.banks[3].len());
    if !quick_buff_bar_full(app) && bag_enabled {
        for j in 0..bag_n {
            if quick_buff_ladder_step(app, QuickSlot::VoidBag(j), &mut last_sound) {
                used = true;
                if quick_buff_bar_full(app) {
                    break; // `:5835-5838`
                }
            }
        }
    }
    // `:5842-5845`: `SoundEngine.PlaySound(legacySoundStyle, base.position)`
    // — ONE play of the last used item's sound, at the top-left.
    if let Some((idx, pitch)) = last_sound {
        let pos = app.player.as_ref().map(|p| (p.x, p.y));
        if let Some((x, y)) = pos {
            let rng = app.rngs.stream(crate::app::RngKey::UpdatePlayers);
            app.audio.play_use_sound(idx, x, y, pitch, rng);
        }
    }
    used
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

    // ---- QuickBuff phase 1 (W4-a5, Player.cs:5766-5870) — WRITTEN NOT RUN ----

    /// The food floor: a live tier blocks equal/lower foods, only a
    /// strictly higher tier re-feeds (`:5922-5932` num = priority + 1).
    #[test]
    fn quick_buff_food_pick_respects_the_live_tier_floor() {
        let mut app = drink_app();
        // No live food: the highest-priority food wins (4029 Pizza, 207).
        app.inventory[0] = slot(4009, 1); // Apple, 26, 18000
        app.inventory[1] = slot(4029, 1); // Pizza, 207, 28800
        assert_eq!(
            quick_buff_pick_best_food_item(&app),
            Some(QuickSlot::Inventory(1))
        );
        // A live 206 floor = 3: the Pizza (207) still wins.
        app.player.as_mut().unwrap().buffs.add_buff(206, 1000);
        assert_eq!(
            quick_buff_pick_best_food_item(&app),
            Some(QuickSlot::Inventory(1))
        );
        // A live 207 floor = 4: nothing outranks it — no food pick.
        app.player.as_mut().unwrap().buffs.add_buff(207, 1000);
        assert_eq!(quick_buff_pick_best_food_item(&app), None);
    }

    /// Among equal-priority foods the longer buffTime wins (`:5939`).
    #[test]
    fn quick_buff_food_pick_prefers_the_longer_buff_time_at_equal_tier() {
        let mut app = drink_app();
        app.inventory[0] = slot(967, 1); // Marshmallow, 26, 3600
        app.inventory[1] = slot(2425, 1); // Cooked Fish, 26, 28800
        assert_eq!(
            quick_buff_pick_best_food_item(&app),
            Some(QuickSlot::Inventory(1)),
            "28800 > 3600 at the same tier"
        );
    }

    /// The void bag joins the food scan only while enabled (`:5946-5961`).
    #[test]
    fn quick_buff_food_pick_scans_the_void_bag_when_enabled() {
        let mut app = drink_app();
        app.player.as_mut().unwrap().banks[3][2] = slot(4029, 1);
        assert_eq!(quick_buff_pick_best_food_item(&app), None, "bag disabled");
        app.player.as_mut().unwrap().void_vault_enabled = true;
        assert_eq!(
            quick_buff_pick_best_food_item(&app),
            Some(QuickSlot::VoidBag(2))
        );
    }

    /// The ladder grants every distinct buff potion, consumes the stacks,
    /// skips summon staffs, and plays ONE use sound at the tail.
    #[test]
    fn quick_buff_ladder_grants_consumes_and_skips_summons() {
        let mut app = drink_app();
        app.inventory[0] = slot(292, 2); // Ironskin, buff 5, 28800
        app.inventory[1] = slot(290, 1); // Swiftness, buff 3, 28800
        app.inventory[2] = slot(1309, 1); // Slime Staff — summon, skipped
        assert!(quick_buff(&mut app));
        let p = app.player.as_ref().unwrap();
        let i5 = p.buffs.find_buff_index(5).expect("Ironskin granted");
        assert_eq!(p.buffs.buff_time[i5], 28800);
        assert!(p.buffs.find_buff_index(3).is_some(), "Swiftness granted");
        assert!(p.buffs.find_buff_index(64).is_none(), "the summon skipped");
        assert_eq!(app.inventory[0].count, 1);
        assert_eq!(app.inventory[1].count, 0, "TurnToAir at 0");
        assert_eq!(app.inventory[1].kind, 0);
        assert_eq!(app.inventory[2].count, 1, "the staff untouched");
        // No swing on the quick path (the C# QuickBuff has no animation).
        assert_eq!(app.swing, 0.0);
    }

    /// The buffTime 0 → 3600 fallback (`:5785-5789`/`:5850-5854`).
    #[test]
    fn quick_buff_zero_buff_time_falls_back_to_3600() {
        let mut app = drink_app();
        // 115 Shadow Orb: buffType 19 (light pet), buffTime 0. Phase 1 has
        // no pet exclusion yet — the grant rides the fallback.
        app.inventory[0] = slot(115, 1);
        assert!(quick_buff(&mut app));
        let p = app.player.as_ref().unwrap();
        let i = p.buffs.find_buff_index(19).expect("the 19 grant");
        assert_eq!(p.buffs.buff_time[i], 3600);
        assert_eq!(app.inventory[0].count, 1, "not consumable — no drain");
    }

    /// The food arm feeds first, then the ladder; the well-fed grant's
    /// IsFedState cleanup keeps only the new tier.
    #[test]
    fn quick_buff_feeds_the_best_food_before_the_buff_ladder() {
        let mut app = drink_app();
        app.inventory[0] = slot(292, 1); // Ironskin
        app.inventory[1] = slot(4029, 1); // Pizza, 207
        assert!(quick_buff(&mut app));
        let p = app.player.as_ref().unwrap();
        assert!(p.buffs.find_buff_index(207).is_some(), "the food fed");
        assert!(p.buffs.find_buff_index(5).is_some(), "the ladder ran too");
        assert_eq!(app.inventory[1].count, 0);
        assert_eq!(app.inventory[0].count, 0);
    }

    /// The gates (`:5768-5776`): dead / cursed / CCed / cursor stack /
    /// the full buff bar (CountBuffs' leading-run quirk included).
    #[test]
    fn quick_buff_gates_dead_cursed_cced_cursor_and_the_full_bar() {
        let mut app = drink_app();
        app.inventory[0] = slot(292, 1);
        app.player.as_mut().unwrap().dead = true;
        assert!(!quick_buff(&mut app));
        let mut app2 = drink_app();
        app2.inventory[0] = slot(292, 1);
        app2.player.as_mut().unwrap().buffs.add_buff(23, 600);
        assert!(!quick_buff(&mut app2));
        let mut app3 = drink_app();
        app3.inventory[0] = slot(292, 1);
        app3.player.as_mut().unwrap().frozen = true;
        assert!(!quick_buff(&mut app3), "CCed (frozen) blocks");
        let mut app4 = drink_app();
        app4.inventory[0] = slot(292, 1);
        app4.mouse_item = slot(2, 1);
        assert!(!quick_buff(&mut app4));
        // The full bar: 44 leading non-empty slots (CountBuffs == 44).
        let mut app5 = drink_app();
        app5.inventory[0] = slot(292, 1);
        {
            let p = app5.player.as_mut().unwrap();
            for i in 0..crate::buffs::MAX_BUFFS {
                p.buffs.buff_type[i] = 5;
                p.buffs.buff_time[i] = 100;
            }
        }
        assert!(!quick_buff(&mut app5), "the full bar refuses");
        assert_eq!(app5.inventory[0].count, 1);
    }

    /// `CountBuffs`' verbatim quirk (`Player.cs:5483-5494`): the read index
    /// is the RUNNING COUNT, so a hole mid-bar stops the count — a plain
    /// census would say 44, the C# says 22.
    #[test]
    fn count_buffs_counts_the_leading_run_not_a_census() {
        let mut b = crate::buffs::BuffSlots::default();
        // Fill every slot, then punch a hole at 22.
        for i in 0..crate::buffs::MAX_BUFFS {
            b.buff_type[i] = 5;
            b.buff_time[i] = 100;
        }
        assert_eq!(b.count_buffs(), 44);
        b.buff_type[22] = 0;
        b.buff_time[22] = 0;
        assert_eq!(b.count_buffs(), 22, "the leading run stops at the hole");
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
}