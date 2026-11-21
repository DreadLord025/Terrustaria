//! W4-g88 — the Life/Mana Crystal permanent upgrade use paths:
//! `Player.ItemCheck_UseLifeCrystal` (`Player.cs:45306-45320`) and
//! `Player.ItemCheck_UseManaCrystal` (`Player.cs:45274-45288`), wired at
//! the `ItemCheck_OwnerOnlyCode` permanent-item seam (`:43595` / `:43597`,
//! around `ItemCheck_UseLifeFruit` `:43596` — the C# call order).
//!
//! The Life Crystal gate (`:45308`): `type == 29 && itemAnimation > 0 &&
//! statLifeMax < 400 && ItemTimeIsZero`. On it: `ApplyItemTime(sItem)`
//! (`:45310`), `statLifeMax += 20` (`:45311`), `statLifeMax2 += 20`
//! (`:45312`), `statLife += 20` (`:45313` — RAW, no clamp at the use
//! site), `HealEffect(20)` for the local player (`:45314-45317`) and
//! `HandleSpecialEvent(this, 0)` (`:45318`).
//!
//! The Mana Crystal gate (`:45276`): `type == 109 && itemAnimation > 0 &&
//! statManaMax < 200 && ItemTimeIsZero`. On it: `ApplyItemTime(sItem)`
//! (`:45278`), `statManaMax += 20` (`:45279`), `statManaMax2 += 20`
//! (`:45280`), `statMana += 20` (`:45281` — RAW), `ManaEffect(20)` for
//! the local player (`:45282-45285`) and `HandleSpecialEvent(this, 1)`
//! (`:45286` → STAR_POWER + the TOPPED_OFF 500/200 check,
//! `AchievementsHelper.cs:290-296`).
//!
//! ## The event-0 finding (verified against the source)
//!
//! `AchievementsHelper.HandleSpecialEvent`'s switch
//! (`AchievementsHelper.cs:288-379`) has **NO case 0** — the Life
//! Crystal's dispatch is a vanilla no-op. There is no life-crystal
//! achievement row (no BIGGER_THAN_LIFE exists anywhere in the
//! decompiled source). The port calls the same dispatcher with 0 (the
//! `_ => return` arm), keeping the source shape without inventing a
//! completion. STAR_POWER's load-time grant (`statManaMax > 20`,
//! `:148-151`) is the OnEnterWorld lane's scope, not this use.
//!
//! ## Port mapping
//!
//! * `statLifeMax` → `Player.life_max_base` (the permanent cap, the
//!   W4-g88 Life Fruit precedent); `statLifeMax2` → the port's single
//!   `max_hp` (the effective-max convention). BOTH raise by 20; the gate
//!   reads the permanent field, never a buff-inflated max.
//! * `statManaMax` → `Player.mana_max_base` (the pre-existing permanent
//!   field, `player.rs` — the per-tick `ResetEffects` rebase
//!   `statManaMax2 = statManaMax` `:18389` IS ported, unlike the life
//!   half); `statManaMax2` → the effective `mana_max`. BOTH raise by 20.
//! * The press-edge / per-tick machine is the SHARED
//!   `life_fruit::permanent_upgrade_item_check` (the recall-machine
//!   shape: StartActualUse leaves itemTime 0, the grant lands on the
//!   press tick; autoReuse false — `Item.cs:1724-1734` stocks 29 and
//!   `:2669-2679` stocks 109 with `useTime 30 / useAnimation 30 /
//!   useStyle 4`; UseSound Item4 / Item29).
//! * `HealEffect(20)` (`:4522-4529`) spawns the HealLife combat text
//!   through the shared `potions::spawn_heal_text` (UpdatePlayers).
//!   `ManaEffect(20)` (`:4531-4538`) queues through
//!   `Player::queue_mana_effect` — the HealMana text drains in
//!   `world_sim/player.rs`'s player tick (the cuffs/potion precedent).
//! * The consume is the generic consumable tail (`:43280-43326`) via
//!   the shared `life_fruit::consume_one` (the drink seam's TurnToAir
//!   fold).

use crate::app::App;
use crate::item::ItemSlot;
use crate::life_fruit::{consume_one, permanent_upgrade_item_check, ticks_of, use_time_secs};

/// `ItemID.LifeCrystal` 29 (`ItemID.cs:1741`, `Player.cs:45308`).
pub(crate) const LIFE_CRYSTAL: u16 = 29;

/// `ItemID.ManaCrystal` 109 (`ItemID.cs:1901`, `Player.cs:45276`).
pub(crate) const MANA_CRYSTAL: u16 = 109;

/// `ItemCheck_UseLifeCrystal`'s gate + body (`Player.cs:45308-45319`),
/// run on an action tick inside the animation. Returns true when the
/// crystal fired.
fn try_use_life_crystal(app: &mut App, held: ItemSlot) -> bool {
    // `:45308`: the < 400 PERMANENT-cap gate + ItemTimeIsZero. The caller
    // owns the `itemAnimation > 0` half (the swing gate).
    let under_cap = app.player.as_ref().is_some_and(|p| p.life_max_base < 400);
    if !under_cap || ticks_of(app.item_time) != 0 {
        return false;
    }
    // `:45310`: ApplyItemTime(sItem).
    app.item_time = use_time_secs(held);
    // `:45311-45313`: both maxima +20, life +20 RAW (the use site does
    // not clamp — the next tick's `statLife > statLifeMax2` clamp owns
    // it). No 400-clamp on the add: a 390 cap would land 410, verbatim.
    let (life_max, mana_max) = {
        let Some(p) = app.player.as_mut() else {
            return false;
        };
        p.life_max_base += 20; // `:45311` statLifeMax += 20
        p.max_hp += 20; // `:45312` statLifeMax2 += 20
        p.hp += 20; // `:45313` statLife += 20
        (p.life_max_base, p.mana_max_base)
    };
    // `:45314-45317`: HealEffect(20) — the local-player HealLife text
    // (the MP SendData 35 broadcast `:4525-4528` has no port).
    crate::potions::spawn_heal_text(app, 20);
    // `:45318`: HandleSpecialEvent(this, 0) — the switch has NO case 0
    // (`AchievementsHelper.cs:288-379`): a vanilla no-op, kept for the
    // source shape (the dispatcher's `_ => return` arm).
    app.achievements
        .handle_special_event_life_mana(0, life_max, mana_max);
    // `:43280-43326`: the generic consumable tail — one per use.
    consume_one(app);
    true
}

/// The Life Crystal `ItemCheck` machine — called once per tick from
/// `update_editing` at the permanent-item seam BEFORE the Life Fruit
/// (`Player.cs:43595` then `:43596`). Returns true when it STARTED a use
/// this tick (the caller consumes the tick).
pub(crate) fn life_crystal_item_check(app: &mut App, press_edge: bool) -> bool {
    permanent_upgrade_item_check(app, press_edge, LIFE_CRYSTAL, try_use_life_crystal)
}

/// `ItemCheck_UseManaCrystal`'s gate + body (`Player.cs:45276-45287`),
/// run on an action tick inside the animation. Returns true when the
/// crystal fired.
fn try_use_mana_crystal(app: &mut App, held: ItemSlot) -> bool {
    // `:45276`: the < 200 PERMANENT-cap gate + ItemTimeIsZero. The caller
    // owns the `itemAnimation > 0` half (the swing gate).
    let under_cap = app.player.as_ref().is_some_and(|p| p.mana_max_base < 200);
    if !under_cap || ticks_of(app.item_time) != 0 {
        return false;
    }
    // `:45278`: ApplyItemTime(sItem).
    app.item_time = use_time_secs(held);
    // `:45279-45281`: both maxima +20, mana +20 RAW (no use-site clamp —
    // the next tick's `statMana > statManaMax2` clamp owns it, `:26000`).
    let (life_max, mana_max) = {
        let Some(p) = app.player.as_mut() else {
            return false;
        };
        p.mana_max_base += 20; // `:45279` statManaMax += 20
        p.mana_max += 20; // `:45280` statManaMax2 += 20
        p.mana += 20; // `:45281` statMana += 20
        // `:45282-45285`: ManaEffect(20) — the local-player HealMana
        // text queue (the MP SendData 43 broadcast `:4534-4537` has no
        // port); the drain spawns it in world_sim/player.rs's tick.
        p.queue_mana_effect(20);
        (p.life_max_base, p.mana_max_base)
    };
    // `:45286`: HandleSpecialEvent(this, 1) → STAR_POWER + TOPPED_OFF at
    // the full 500/200 caps (`AchievementsHelper.cs:290-296`).
    app.achievements
        .handle_special_event_life_mana(1, life_max, mana_max);
    // `:43280-43326`: the generic consumable tail — one per use.
    consume_one(app);
    true
}

/// The Mana Crystal `ItemCheck` machine — called once per tick from
/// `update_editing` at the permanent-item seam AFTER the Life Fruit
/// (`Player.cs:43596` then `:43597`). Returns true when it STARTED a use
/// this tick (the caller consumes the tick).
pub(crate) fn mana_crystal_item_check(app: &mut App, press_edge: bool) -> bool {
    permanent_upgrade_item_check(app, press_edge, MANA_CRYSTAL, try_use_mana_crystal)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::flow::front_flow_tests::flow_app;
    use crate::player::Player;
    use crate::registry;
    use tr_core::World;

    const ONE_TICK_SECS: f32 = 1.0 / 60.0;

    /// The life_fruit.rs rig convention: the crystal selected, audio
    /// MUTED (the sound draws are audio-only; muting keeps the RNG
    /// witnesses off the UpdatePlayers stream).
    fn rig(kind: u16, count: u32, life_max: i32) -> App {
        let mut app = flow_app(None, vec![], vec![]);
        app.world = World::new("crystal".into(), 64, 64);
        let mut p = Player::spawn(&app.world);
        p.life_max_base = life_max;
        p.max_hp = life_max;
        p.hp = life_max - 30; // headroom: the +20 must not clamp
        app.player = Some(p);
        app.inventory[app.selected] = ItemSlot {
            kind,
            name: registry::name(kind),
            count,
            prefix: 0,
        };
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

    fn press(app: &mut App) {
        app.mine_prev_down = false;
        app.mine_released = true;
        app.mine_down = true;
    }

    fn tick(app: &mut App) {
        app.swing = (app.swing - ONE_TICK_SECS).max(0.0);
        app.item_time = (app.item_time - ONE_TICK_SECS).max(0.0);
        life_crystal_item_check(app, false);
    }

    /// `:45308-45313`: at the fresh 100 cap the crystal grants +20 to
    /// BOTH maxima and life, re-arms itemTime = useTime (30) and
    /// consumes one.
    #[test]
    fn w4_g88_the_life_crystal_grants_twenty_at_one_hundred() {
        let mut app = rig(LIFE_CRYSTAL, 3, 100);
        let hp0 = app.player.as_ref().unwrap().hp;
        press(&mut app);
        assert!(life_crystal_item_check(&mut app, true), "the use started");
        let p = app.player.as_ref().unwrap();
        assert_eq!(p.life_max_base, 120, "statLifeMax += 20 (:45311)");
        assert_eq!(p.max_hp, 120, "statLifeMax2 += 20 (:45312)");
        assert_eq!(p.hp, hp0 + 20, "statLife += 20 (:45313)");
        assert_eq!(ticks_of(app.item_time), 30, "ApplyItemTime(useTime 30)");
        assert_eq!(app.inventory[app.selected].count, 2, "one consumed");
        assert!(app.swing > 0.0, "the HoldUp swing runs");
    }

    /// `:45308`: at 400 the gate is closed — no grant, no consume.
    #[test]
    fn w4_g88_at_four_hundred_the_life_crystal_is_refused() {
        let mut app = rig(LIFE_CRYSTAL, 3, 400);
        press(&mut app);
        life_crystal_item_check(&mut app, true);
        for _ in 0..40 {
            tick(&mut app);
        }
        let p = app.player.as_ref().unwrap();
        assert_eq!(p.life_max_base, 400, "no grant at the 400 cap");
        assert_eq!(app.inventory[app.selected].count, 3, "nothing consumed");
    }

    /// The full 100 → 400 ladder: fifteen crystals, one per press edge
    /// (autoReuse false), landing exactly 400 — then the fruit window
    /// opens (the `life_fruit.rs` lane).
    #[test]
    fn w4_g88_fifteen_crystals_climb_one_hundred_to_four_hundred() {
        let mut app = rig(LIFE_CRYSTAL, 15, 100);
        for expected in (120..=400).step_by(20) {
            press(&mut app);
            assert!(life_crystal_item_check(&mut app, true));
            app.mine_down = false;
            for _ in 0..31 {
                tick(&mut app);
            }
            assert_eq!(app.player.as_ref().unwrap().life_max_base, expected);
        }
        assert_eq!(app.inventory[app.selected].count, 0, "all fifteen consumed");
        assert_eq!(app.inventory[app.selected].kind, 0, "TurnToAir on 0-stack");
        press(&mut app);
        assert!(!life_crystal_item_check(&mut app, true), "air refuses");
    }

    /// `:45313` is RAW: at full life the hp overshoots by 20 (the next
    /// tick's clamp owns it).
    #[test]
    fn w4_g88_the_life_add_is_raw_at_the_use_site() {
        let mut app = rig(LIFE_CRYSTAL, 1, 100);
        app.player.as_mut().unwrap().hp = 100;
        press(&mut app);
        life_crystal_item_check(&mut app, true);
        let p = app.player.as_ref().unwrap();
        assert_eq!(p.hp, 120, "RAW +20, no use-site clamp (:45313)");
        assert_eq!(p.max_hp, 120, "the new cap covers it anyway");
    }

    /// `:45318` → HandleSpecialEvent(this, 0): the switch has NO case 0
    /// (`AchievementsHelper.cs:288-379`) — no achievement completes, not
    /// even the Use-flag fan-out rows.
    #[test]
    fn w4_g88_event_zero_completes_nothing() {
        let mut app = rig(LIFE_CRYSTAL, 1, 100);
        press(&mut app);
        life_crystal_item_check(&mut app, true);
        assert!(
            app.achievements.list.iter().all(|a| !a.completed),
            "event 0 is a vanilla no-op — no case 0 in the switch"
        );
        assert!(app.achievements.completed_events.is_empty());
    }

    /// The mid-swing refusal: while itemTime > 0 the per-tick arm waits
    /// (ItemTimeIsZero, `:45308`) — one grant per 30-tick cadence.
    #[test]
    fn w4_g88_one_grant_per_use_cadence() {
        let mut app = rig(LIFE_CRYSTAL, 5, 100);
        press(&mut app);
        life_crystal_item_check(&mut app, true);
        for _ in 0..29 {
            tick(&mut app);
        }
        assert_eq!(app.player.as_ref().unwrap().life_max_base, 120);
        assert_eq!(app.inventory[app.selected].count, 4, "exactly one consumed");
    }

    /// The CCed head (`:42598-42603`): a stoned player's swing zeroes
    /// and the crystal never fires.
    #[test]
    fn w4_g88_cced_zeroes_the_swing_and_refuses() {
        let mut app = rig(LIFE_CRYSTAL, 3, 100);
        app.player.as_mut().unwrap().buffs.add_buff(156, 600); // Petrified
        press(&mut app);
        assert!(!life_crystal_item_check(&mut app, true));
        assert_eq!(app.swing, 0.0);
        assert_eq!(app.player.as_ref().unwrap().life_max_base, 100);
        assert_eq!(app.inventory[app.selected].count, 3);
    }

    // ---- the Mana Crystal (`Player.cs:45274-45288`) ----

    /// A mana rig: the fresh 20/20 character (Player.Initialize's default,
    /// `Player.cs:56075`), the crystal selected, audio muted.
    fn mana_rig(count: u32, mana_max: i32) -> App {
        let mut app = rig(MANA_CRYSTAL, count, 100);
        let p = app.player.as_mut().unwrap();
        p.mana_max_base = mana_max;
        p.mana_max = mana_max;
        p.mana = mana_max - 20; // headroom: the +20 must not clamp
        app
    }

    fn mana_tick(app: &mut App) {
        app.swing = (app.swing - ONE_TICK_SECS).max(0.0);
        app.item_time = (app.item_time - ONE_TICK_SECS).max(0.0);
        mana_crystal_item_check(app, false);
    }

    /// `:45276-45281`: at the fresh 20 cap the crystal grants +20 to BOTH
    /// mana maxima and mana, re-arms itemTime (30) and consumes one.
    #[test]
    fn w4_g88_the_mana_crystal_grants_twenty_at_twenty() {
        let mut app = mana_rig(3, 20);
        let mana0 = app.player.as_ref().unwrap().mana;
        press(&mut app);
        assert!(mana_crystal_item_check(&mut app, true), "the use started");
        let p = app.player.as_ref().unwrap();
        assert_eq!(p.mana_max_base, 40, "statManaMax += 20 (:45279)");
        assert_eq!(p.mana_max, 40, "statManaMax2 += 20 (:45280)");
        assert_eq!(p.mana, mana0 + 20, "statMana += 20 (:45281)");
        assert_eq!(p.life_max_base, 100, "the life cap is untouched");
        assert_eq!(ticks_of(app.item_time), 30, "ApplyItemTime(useTime 30)");
        assert_eq!(app.inventory[app.selected].count, 2, "one consumed");
    }

    /// `:45276`: at 200 the gate is closed — no grant, no consume.
    #[test]
    fn w4_g88_at_two_hundred_the_mana_crystal_is_refused() {
        let mut app = mana_rig(3, 200);
        press(&mut app);
        mana_crystal_item_check(&mut app, true);
        for _ in 0..40 {
            mana_tick(&mut app);
        }
        let p = app.player.as_ref().unwrap();
        assert_eq!(p.mana_max_base, 200, "no grant at the 200 cap");
        assert_eq!(app.inventory[app.selected].count, 3, "nothing consumed");
    }

    /// The full 20 → 200 ladder: nine crystals, one per press edge
    /// (autoReuse false), landing exactly 200.
    #[test]
    fn w4_g88_nine_crystals_climb_twenty_to_two_hundred() {
        let mut app = mana_rig(9, 20);
        for expected in (40..=200).step_by(20) {
            press(&mut app);
            assert!(mana_crystal_item_check(&mut app, true));
            app.mine_down = false;
            for _ in 0..31 {
                mana_tick(&mut app);
            }
            assert_eq!(app.player.as_ref().unwrap().mana_max_base, expected);
        }
        assert_eq!(app.inventory[app.selected].count, 0, "all nine consumed");
        assert_eq!(app.inventory[app.selected].kind, 0, "TurnToAir on 0-stack");
    }

    /// `:45281` is RAW: at full mana the overshoot is not clamped at the
    /// use site (the next tick's `:26000` clamp owns it).
    #[test]
    fn w4_g88_the_mana_add_is_raw_at_the_use_site() {
        let mut app = mana_rig(1, 20);
        app.player.as_mut().unwrap().mana = 20;
        press(&mut app);
        mana_crystal_item_check(&mut app, true);
        let p = app.player.as_ref().unwrap();
        assert_eq!(p.mana, 40, "RAW +20, no use-site clamp (:45281)");
        assert_eq!(p.mana_max, 40, "the new cap covers it anyway");
    }

    /// `:45282-45285`: ManaEffect(20) queues the HealMana text — the
    /// one-shot the world_sim/player.rs drain spawns.
    #[test]
    fn w4_g88_mana_effect_queues_the_heal_mana_text() {
        let mut app = mana_rig(1, 20);
        press(&mut app);
        mana_crystal_item_check(&mut app, true);
        let p = app.player.as_mut().unwrap();
        assert_eq!(p.take_mana_effect(), Some(20), "ManaEffect(20) (:45284)");
        assert_eq!(p.take_mana_effect(), None, "the text is a one-shot");
    }

    /// `:45286` → HandleSpecialEvent(1): STAR_POWER completes on the first
    /// crystal; TOPPED_OFF waits for the full 500/200 caps.
    #[test]
    fn w4_g88_star_power_fires_and_topped_off_waits_for_the_caps() {
        let mut app = mana_rig(9, 20);
        let done = |app: &App, name: &str| {
            app.achievements.list.iter().any(|a| a.name == name && a.completed)
        };
        press(&mut app);
        mana_crystal_item_check(&mut app, true);
        assert!(done(&app, "STAR_POWER"), "the first crystal grants STAR_POWER");
        assert!(!done(&app, "TOPPED_OFF"), "100/40 is not topped off");
        assert!(!done(&app, "GET_A_LIFE"), "the named dispatch spares GET_A_LIFE");
        // Life already 500 (the fruit lane): the ninth crystal lands
        // TOPPED_OFF on the 500/200 check.
        app.player.as_mut().unwrap().life_max_base = 500;
        app.mine_down = false;
        for _ in 0..31 {
            mana_tick(&mut app);
        }
        for _ in 0..8 {
            press(&mut app);
            mana_crystal_item_check(&mut app, true);
            app.mine_down = false;
            for _ in 0..31 {
                mana_tick(&mut app);
            }
        }
        assert_eq!(app.player.as_ref().unwrap().mana_max_base, 200);
        assert!(done(&app, "TOPPED_OFF"), "500/200 completes TOPPED_OFF");
    }

    /// The CCed head applies to the mana machine too (`:42598-42603`).
    #[test]
    fn w4_g88_cced_zeroes_the_mana_swing_and_refuses() {
        let mut app = mana_rig(3, 20);
        app.player.as_mut().unwrap().buffs.add_buff(156, 600); // Petrified
        press(&mut app);
        assert!(!mana_crystal_item_check(&mut app, true));
        assert_eq!(app.swing, 0.0);
        assert_eq!(app.player.as_ref().unwrap().mana_max_base, 20);
        assert_eq!(app.inventory[app.selected].count, 3);
    }
}
