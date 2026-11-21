//! W4-g88 — the Life Crystal permanent upgrade use path:
//! `Player.ItemCheck_UseLifeCrystal` (`Player.cs:45306-45320`), wired at
//! the `ItemCheck_OwnerOnlyCode` permanent-item seam (`:43595`, BEFORE
//! `ItemCheck_UseLifeFruit` `:43596` — the C# call order).
//!
//! The gate (`:45308`): `type == 29 && itemAnimation > 0 &&
//! statLifeMax < 400 && ItemTimeIsZero`. On it: `ApplyItemTime(sItem)`
//! (`:45310`), `statLifeMax += 20` (`:45311`), `statLifeMax2 += 20`
//! (`:45312`), `statLife += 20` (`:45313` — RAW, no clamp at the use
//! site), `HealEffect(20)` for the local player (`:45314-45317`) and
//! `HandleSpecialEvent(this, 0)` (`:45318`).
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
//! * The press-edge / per-tick machine is the SHARED
//!   `life_fruit::permanent_upgrade_item_check` (the recall-machine
//!   shape: StartActualUse leaves itemTime 0, the grant lands on the
//!   press tick; autoReuse false — `Item.cs:1724-1734` stocks 29 with
//!   `useTime 30 / useAnimation 30 / useStyle 4 / UseSound Item4`).
//! * `HealEffect(20)` (`:4522-4529`) spawns the HealLife combat text
//!   through the shared `potions::spawn_heal_text` (UpdatePlayers).
//! * The consume is the generic consumable tail (`:43280-43326`) via
//!   the shared `life_fruit::consume_one` (the drink seam's TurnToAir
//!   fold).

use crate::app::App;
use crate::item::ItemSlot;
use crate::life_fruit::{consume_one, permanent_upgrade_item_check, ticks_of, use_time_secs};

/// `ItemID.LifeCrystal` 29 (`ItemID.cs:1741`, `Player.cs:45308`).
pub(crate) const LIFE_CRYSTAL: u16 = 29;

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
}
