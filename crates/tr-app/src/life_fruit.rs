//! W4-g88 — the Life Fruit permanent upgrade use path:
//! `Player.ItemCheck_UseLifeFruit` (`Player.cs:45290-45304`), wired at the
//! `ItemCheck_OwnerOnlyCode` permanent-item seam (`:43596`, between
//! `ItemCheck_UseLifeCrystal` `:43595` and `ItemCheck_UseManaCrystal`
//! `:43597` — both siblings landed in `crystals.rs`).
//!
//! The gate (`:45292`): `type == 1291 && itemAnimation > 0 &&
//! statLifeMax >= 400 && statLifeMax < 500 && ItemTimeIsZero`. On it:
//! `ApplyItemTime(sItem)` (`:45294`), `statLifeMax += 5` (`:45295`),
//! `statLifeMax2 += 5` (`:45296`), `statLife += 5` (`:45297` — RAW, no clamp
//! at the use site), `HealEffect(5)` for the local player (`:45298-45301`)
//! and `HandleSpecialEvent(2)` (`:45302` → GET_A_LIFE + the TOPPED_OFF
//! 500/200 check, `AchievementsHelper.cs:298-305`).
//!
//! ## Port mapping
//!
//! * `statLifeMax` → `Player.life_max_base` (the permanent cap, the
//!   `mana_max_base` precedent); `statLifeMax2` → the port's single `max_hp`
//!   (the `player.rs:3597` effective-max convention). BOTH raise by 5, so
//!   the gate reads the permanent field and never a buff-inflated max.
//! * The press-edge / per-tick machine shape is `recall_item_check`'s
//!   (`recall_teleport.rs`): the use START (`ItemCheck_StartActualUse` →
//!   `ApplyItemAnimation` + the Item129 `UseSound`, `Player.cs:51680-51698`)
//!   on the `controlUseItem && releaseUseItem && itemAnimation == 0` edge
//!   (`:42733`), the grant on the first action tick inside the animation
//!   (`ItemTimeIsZero`, `StartActualUse` leaves `itemTime` 0 — the boss
//!   spawner precedent `editing.rs:2021-2028`). `autoReuse` is false
//!   (`Item.cs`'s 1291 row), so one press = one fruit.
//! * The consume is the generic consumable tail (`:43280-43326`): one per
//!   use, `CanConsumeConsumableItem` true, no ForceConsumption entry; the
//!   0-stack TurnToAir folds onto the consume tick (the drink seam's
//!   convention, `potions.rs:474-487`).
//! * `HealEffect(5)` (`:4522-4529`) spawns the HealLife combat text through
//!   the shared `potions::spawn_heal_text` (the UpdatePlayers stream).
//!
//! ## The Plantera question (recorded deviation from the task brief)
//!
//! The C# USE gate has NO boss flag — only the `[400, 500)` life window.
//! The "Plantera-gated" availability is worldgen-side: Life Fruit tiles
//! grow at `WorldGen.cs:74449-74478` under `Main.hardMode &&
//! NPC.downedMechBossAny` (NOT downedPlantBoss — the brief's shorthand;
//! the growth gate is re-verified here against the source). Tile growth
//! and the 236-tile break drop (`:48027-48030` → `Item.NewItem` 1291) are
//! the worldgen/framing lanes' scope; this module owns the USE. No
//! invented boss gate is added to the use (golden rule: port the C#).
//!
//! ## Persistence
//!
//! Vanilla persists `statLifeMax` in the `.plr` (`Player.cs:54773` write /
//! `:55220-55224` read-clamp-500 — `player_file.rs` already round-trips it).
//! The port's live-session save is the `.trplayer` sidecar (`persist.rs`),
//! whose v7 line carries `life_max_base` + the effective `max_hp`; the
//! load path (`app/mod.rs`) applies both before the hp clamp.

use crate::app::App;
use crate::item::ItemSlot;
use crate::world_sim::editing::start_swing_extras;

/// `ItemID.LifeFruit` 1291 (`ItemID.cs`, `Player.cs:45292`).
pub(crate) const LIFE_FRUIT: u16 = 1291;

/// One 60 Hz tick in seconds (the `recall_teleport.rs` convention).
const ONE_TICK_SECS: f32 = 1.0 / 60.0;

/// The C# INTEGER tick comparisons out of the port's seconds timers
/// (`Player.cs:4064` `ItemTimeIsZero => itemTime == 0`). Shared with the
/// crystal machines (`crystals.rs`, the `:43595`/`:43597` siblings).
pub(crate) fn ticks_of(secs: f32) -> i32 {
    (secs * 60.0).round() as i32
}

/// The held item's prefixed `useTime` in seconds — `ApplyItemTime`
/// (`Player.cs:4416-4419` → `SetItemTime(useTime)`), the instance value
/// (the `recall_teleport.rs::use_time_secs` convention). 1291 stocks
/// `useTime 30 / useAnimation 30 / useStyle 4` (`itemdata.rs:1457`); the
/// crystals stock the same 30/30/4 (`Item.cs:1724-1734` / `:2669-2679`).
/// Shared with `crystals.rs` (the `:43595`/`:43597` siblings).
pub(crate) fn use_time_secs(held: ItemSlot) -> f32 {
    crate::prefix::prefixed_stats(held.kind, held.prefix)
        .map(|s| s.use_time as f32 * ONE_TICK_SECS)
        .filter(|t| *t > 0.0)
        .unwrap_or_else(|| crate::item::use_time_secs(held.kind))
}

/// `ItemCheck_UseLifeFruit`'s gate + body (`Player.cs:45292-45303`), run on
/// an action tick inside the animation. Returns true when the fruit fired.
fn try_use_life_fruit(app: &mut App, held: ItemSlot) -> bool {
    // `:45292`: the [400, 500) PERMANENT-cap window + ItemTimeIsZero. The
    // caller owns the `itemAnimation > 0` half (the swing gate).
    let in_window = app
        .player
        .as_ref()
        .is_some_and(|p| (400..500).contains(&p.life_max_base));
    if !in_window || ticks_of(app.item_time) != 0 {
        return false;
    }
    // `:45294`: ApplyItemTime(sItem).
    app.item_time = use_time_secs(held);
    // `:45295-45297`: both maxima +5, life +5 RAW (the use site does not
    // clamp — the next tick's `statLife > statLifeMax2` clamp owns it).
    let (life_max, mana_max) = {
        let Some(p) = app.player.as_mut() else {
            return false;
        };
        p.life_max_base += 5; // `:45295` statLifeMax += 5
        p.max_hp += 5; // `:45296` statLifeMax2 += 5
        p.hp += 5; // `:45297` statLife += 5
        (p.life_max_base, p.mana_max_base)
    };
    // `:45298-45301`: HealEffect(5) — the local-player HealLife text (the
    // MP SendData 35 broadcast `:4525-4528` has no port).
    crate::potions::spawn_heal_text(app, 5);
    // `:45302`: HandleSpecialEvent(2) → GET_A_LIFE + TOPPED_OFF at 500/200.
    app.achievements
        .handle_special_event_life_mana(2, life_max, mana_max);
    // `:43280-43326`: the generic consumable tail — one per use (the
    // `potions.rs:474-487` fold: count--, then TurnToAir on the 0-stack).
    consume_one(app);
    true
}

/// The generic consumable tail (`Player.cs:43280-43326`) folded onto the
/// grant tick: one per use (`CanConsumeConsumableItem` true, no
/// ForceConsumption entry), then TurnToAir on the 0-stack
/// (`:42824-42828`, the drink seam's convention `potions.rs:474-487`).
/// Shared by the fruit and both crystals.
pub(crate) fn consume_one(app: &mut App) {
    let slot = &mut app.inventory[app.selected];
    if slot.count > 0 {
        slot.count -= 1; // `:43317-43320`
    }
    if slot.count == 0 {
        *slot = crate::item::EMPTY_SLOT; // `:42824-42828`
    }
}

/// The Life Fruit `ItemCheck` machine — called once per tick from
/// `update_editing` at the permanent-item seam (after the recall cluster,
/// `Player.cs:43595-43597` order). `press_edge` is the caller's true
/// previous-frame pair (the `recall_item_check` convention). Returns true
/// when it STARTED a use this tick (the caller consumes the tick).
pub(crate) fn life_fruit_item_check(app: &mut App, press_edge: bool) -> bool {
    permanent_upgrade_item_check(app, press_edge, LIFE_FRUIT, try_use_life_fruit)
}

/// The permanent-upgrade `ItemCheck` machine shared by the Life Fruit
/// (`:45290-45304`), the Life Crystal (`:45306-45320`) and the Mana Crystal
/// (`:45274-45288`) — all three ride the same `ItemCheck_OwnerOnlyCode`
/// slot (`:43595-43597`), the same useStyle-4 HoldUp start, the same
/// `itemAnimation > 0 && ItemTimeIsZero` grant gate and the same
/// autoReuse-false one-use-per-press contract. `grant` is the item's
/// `ItemCheck_Use*` gate + body, run on an action tick inside the
/// animation. The machine shape is `recall_item_check`'s (the boss-spawner
/// ItemTimeIsZero fold): StartActualUse leaves itemTime 0, so a granted
/// use lands on the press tick itself.
pub(crate) fn permanent_upgrade_item_check(
    app: &mut App,
    press_edge: bool,
    kind: u16,
    grant: fn(&mut App, ItemSlot) -> bool,
) -> bool {
    let held = app.inventory[app.selected];
    if held.kind != kind || held.count == 0 {
        return false;
    }
    // `:42598-42603` CCed → the animation zeroes and ItemCheck returns;
    // the dead player runs UpdateDead instead (the recall machine's head).
    if app.player.as_ref().is_some_and(|p| p.dead || p.cced()) {
        app.swing = 0.0;
        app.swing_max = 0.0;
        app.item_time = 0.0;
        return false;
    }
    if ticks_of(app.swing) > 0 {
        // The per-tick arm (`:43595-43597` run every tick of the
        // animation): the grant fires when ItemTimeIsZero inside the swing.
        grant(app, held);
        return false;
    }
    // The press edge (`:42733`): useStyle 4 (HoldUp) ≠ 0, autoReuse false.
    if app.mine_down && !app.inventory_open && press_edge {
        // ItemCheck_StartActualUse → ApplyItemAnimation + the item's
        // UseSound (`:51680-51698`, the shared start_swing_extras seam).
        app.swing_max = crate::item::use_animation_secs_of(held.kind, held.prefix);
        app.swing = app.swing_max;
        start_swing_extras(app, held.kind);
        app.attack_cd = app.swing_max;
        // StartActualUse leaves itemTime 0 → the grant fires THIS tick
        // (the boss-spawner precedent, `editing.rs:2021-2028`).
        app.item_time = 0.0;
        grant(app, held);
        app.mine_target = None;
        app.mine_timer = 0.0;
        return true;
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::flow::front_flow_tests::flow_app;
    use crate::player::Player;
    use crate::registry;
    use tr_core::World;

    /// A headless rig with the fruit selected and the audio MUTED (the
    /// sound draws are audio-only; muting keeps the RNG witnesses off the
    /// UpdatePlayers stream — the `recall_teleport.rs` rig convention).
    fn rig(count: u32, life_max: i32) -> App {
        let mut app = flow_app(None, vec![], vec![]);
        app.world = World::new("fruit".into(), 64, 64);
        let mut p = Player::spawn(&app.world);
        p.life_max_base = life_max;
        p.max_hp = life_max;
        p.hp = life_max - 30; // headroom: the +5 must not clamp
        app.player = Some(p);
        app.inventory[app.selected] = ItemSlot {
            kind: LIFE_FRUIT,
            name: registry::name(LIFE_FRUIT),
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
        life_fruit_item_check(app, false);
    }

    /// `:45292-45297`: at 400 the fruit grants +5 to BOTH maxima and life,
    /// re-arms itemTime = useTime (30) and consumes one.
    #[test]
    fn w4_g88_the_fruit_grants_five_at_four_hundred() {
        let mut app = rig(3, 400);
        let hp0 = app.player.as_ref().unwrap().hp;
        press(&mut app);
        assert!(life_fruit_item_check(&mut app, true), "the use started");
        let p = app.player.as_ref().unwrap();
        assert_eq!(p.life_max_base, 405, "statLifeMax += 5 (:45295)");
        assert_eq!(p.max_hp, 405, "statLifeMax2 += 5 (:45296)");
        assert_eq!(p.hp, hp0 + 5, "statLife += 5 (:45297)");
        assert_eq!(ticks_of(app.item_time), 30, "ApplyItemTime(useTime 30)");
        assert_eq!(app.inventory[app.selected].count, 2, "one consumed");
        assert!(app.swing > 0.0, "the HoldUp swing runs");
    }

    /// `:45292`: below 400 the gate refuses — no grant, no consume, and
    /// (the machine's start arm still swings, the C# StartActualUse runs
    /// for any useStyle ≠ 0 item) the per-tick grant never fires.
    #[test]
    fn w4_g88_below_four_hundred_the_fruit_is_refused() {
        let mut app = rig(3, 395);
        press(&mut app);
        life_fruit_item_check(&mut app, true);
        for _ in 0..40 {
            tick(&mut app);
        }
        let p = app.player.as_ref().unwrap();
        assert_eq!(p.life_max_base, 395, "no grant below 400");
        assert_eq!(app.inventory[app.selected].count, 3, "nothing consumed");
    }

    /// `:45292`: at 500 the window is closed — the cap holds.
    #[test]
    fn w4_g88_at_five_hundred_the_cap_holds() {
        let mut app = rig(3, 500);
        press(&mut app);
        life_fruit_item_check(&mut app, true);
        for _ in 0..40 {
            tick(&mut app);
        }
        let p = app.player.as_ref().unwrap();
        assert_eq!(p.life_max_base, 500, "no grant at the 500 cap");
        assert_eq!(p.max_hp, 500);
        assert_eq!(app.inventory[app.selected].count, 3, "nothing consumed");
    }

    /// The full 400 → 500 ladder: twenty fruits, one per press edge (the
    /// autoReuse-false one-use-per-press contract), landing exactly 500.
    #[test]
    fn w4_g88_twenty_fruits_climb_four_hundred_to_five_hundred() {
        let mut app = rig(20, 400);
        for expected in (405..=500).step_by(5) {
            press(&mut app);
            assert!(life_fruit_item_check(&mut app, true));
            // Run the animation out (30 ticks) so the next press edge is
            // clean — the swing gate `itemAnimation == 0` (:42733).
            app.mine_down = false;
            for _ in 0..31 {
                tick(&mut app);
            }
            assert_eq!(app.player.as_ref().unwrap().life_max_base, expected);
        }
        assert_eq!(app.inventory[app.selected].count, 0, "all twenty consumed");
        assert_eq!(app.inventory[app.selected].kind, 0, "TurnToAir on 0-stack");
        // A twenty-first press finds air — the machine refuses.
        press(&mut app);
        assert!(!life_fruit_item_check(&mut app, true));
    }

    /// `:45297` is RAW: the +5 life is not clamped at the use site (the
    /// next tick's clamp owns it) — at full life the hp overshoots by 5.
    #[test]
    fn w4_g88_the_life_add_is_raw_at_the_use_site() {
        let mut app = rig(1, 400);
        app.player.as_mut().unwrap().hp = 400;
        press(&mut app);
        life_fruit_item_check(&mut app, true);
        let p = app.player.as_ref().unwrap();
        assert_eq!(p.hp, 405, "RAW +5, no use-site clamp (:45297)");
        assert_eq!(p.max_hp, 405, "the new cap covers it anyway");
    }

    /// `:45302` → HandleSpecialEvent(2): GET_A_LIFE completes on the first
    /// fruit; TOPPED_OFF waits for the full 500/200 caps.
    #[test]
    fn w4_g88_get_a_life_fires_and_topped_off_waits_for_the_caps() {
        let mut app = rig(20, 400);
        app.player.as_mut().unwrap().mana_max_base = 200;
        press(&mut app);
        life_fruit_item_check(&mut app, true);
        let done = |app: &App, name: &str| {
            app.achievements.list.iter().any(|a| a.name == name && a.completed)
        };
        assert!(done(&app, "GET_A_LIFE"), "the first fruit grants GET_A_LIFE");
        assert!(!done(&app, "TOPPED_OFF"), "405/200 is not topped off");
        assert!(!done(&app, "STAR_POWER"), "the named dispatch spares STAR_POWER");
        // Climb to 500 — the twentieth fruit lands TOPPED_OFF.
        app.mine_down = false;
        for _ in 0..31 {
            tick(&mut app);
        }
        for _ in 0..19 {
            press(&mut app);
            life_fruit_item_check(&mut app, true);
            app.mine_down = false;
            for _ in 0..31 {
                tick(&mut app);
            }
        }
        assert_eq!(app.player.as_ref().unwrap().life_max_base, 500);
        assert!(done(&app, "TOPPED_OFF"), "500/200 completes TOPPED_OFF");
    }

    /// The mid-swing refusal: while itemTime > 0 the per-tick arm waits
    /// (ItemTimeIsZero, `:45292`) — a second grant cannot fire inside the
    /// 30-tick cadence even with the button held.
    #[test]
    fn w4_g88_one_grant_per_use_cadence() {
        let mut app = rig(5, 400);
        press(&mut app);
        life_fruit_item_check(&mut app, true);
        // Hold the button through the whole animation (no fresh edge).
        for _ in 0..29 {
            tick(&mut app);
        }
        assert_eq!(app.player.as_ref().unwrap().life_max_base, 405);
        assert_eq!(app.inventory[app.selected].count, 4, "exactly one consumed");
    }

    /// The CCed head (`:42598-42603`): a stoned player's swing zeroes and
    /// the fruit never fires.
    #[test]
    fn w4_g88_cced_zeroes_the_swing_and_refuses() {
        let mut app = rig(3, 400);
        app.player.as_mut().unwrap().buffs.add_buff(156, 600); // Petrified
        press(&mut app);
        assert!(!life_fruit_item_check(&mut app, true));
        assert_eq!(app.swing, 0.0);
        assert_eq!(app.player.as_ref().unwrap().life_max_base, 400);
        assert_eq!(app.inventory[app.selected].count, 3);
    }

    /// The permanent upgrade survives the session: the v7 `.trplayer`
    /// sidecar carries the raised cap (`Player.SavePlayer`'s `statLifeMax`
    /// write `:54773`, the port's `persist::save`), and the load pair feeds
    /// `App::new`'s apply block (`app/mod.rs`, before the hp clamp).
    #[test]
    fn w4_g88_the_upgrade_round_trips_through_the_sidecar() {
        let mut app = rig(2, 400);
        press(&mut app);
        life_fruit_item_check(&mut app, true);
        let dir = crate::test_world_helpers::test_scratch("life-fruit-persist");
        let path = dir.join("fruit.trwld");
        let p = app.player.as_ref().unwrap();
        let inv: [(u16, u32); 50] =
            std::array::from_fn(|i| (app.inventory[i].kind, app.inventory[i].count));
        crate::persist::save(&path, p, inv, 0, [0; 3], &[0; 10], &[]).unwrap();
        let saved = crate::persist::load(&path).expect("the v7 sidecar");
        assert_eq!(saved.life_max, Some((405, 405)), "the permanent +5 saved");
        assert_eq!(saved.hp, p.hp, "the raised life saved");
        // The loader's apply order (app/mod.rs): maxima first, THEN the hp
        // clamp — a reloaded 405-cap character keeps its full saved hp.
        let mut fresh = Player::spawn(&app.world);
        if let Some((base, effective)) = saved.life_max {
            fresh.life_max_base = base;
            fresh.max_hp = effective;
        }
        fresh.hp = saved.hp.clamp(1, fresh.max_hp);
        assert_eq!(fresh.life_max_base, 405);
        assert_eq!(fresh.max_hp, 405);
        assert_eq!(fresh.hp, saved.hp);
        // And the gate reads the RELOADED permanent cap: the next fruit
        // still fires (405 < 500).
        app.player = Some(fresh);
        app.swing = 0.0;
        app.item_time = 0.0;
        app.mine_down = false;
        for _ in 0..31 {
            tick(&mut app);
        }
        press(&mut app);
        assert!(life_fruit_item_check(&mut app, true));
        assert_eq!(app.player.as_ref().unwrap().life_max_base, 410);
        crate::test_world_helpers::remove_scratch(&dir);
    }
}
