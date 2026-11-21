//! W4-a6 — the permanent power-up consumables: `Player.ItemCheck`'s
//! `ItemCheck_UseLifeCrystal` / `UseLifeFruit` / `UseManaCrystal` /
//! `UseDemonHeart` (`Player.cs:45192-45320`) and their sibling
//! `ItemCheck_Use*` arms (`:45322-45423`, later commits).
//!
//! Every arm rides the C# mid-swing gate `itemAnimation > 0 &&
//! ItemTimeIsZero` (`:4064`): the use START (`ApplyItemAnimation`) ran on
//! the press tick, the grant fires on the first action tick inside the
//! animation and `ApplyItemTime` (`:4416-4419`) re-arms the cadence. The
//! machine is the boss-spawner / recall shape (`editing.rs`'s summon gate,
//! `recall_teleport::recall_item_check`): a press edge starts the swing,
//! the grant lands when `item_time` reads zero inside the running swing.
//! A REFUSED arm (cap reached) keeps re-checking each action tick — the
//! C# gate never consumes on a refusal — and the generic consumable tail
//! (`:43280-43326`) only pays the stack when `ApplyItemTime` ran
//! (`itemTimeMax != 0 && itemTime == itemTimeMax`), so a refused use does
//! NOT consume.
//!
//! The stack consume folds onto the grant tick (the food/potion arms'
//! `TurnToAir` convention, `potions.rs:474-487`); the C# runs it one block
//! later in the same tick (`:43280`), no observable difference in SP.
//! The `NetMessage.SendData` rows are MP-only — the port is single-player
//! (`chat.rs`'s netMode-0 note), so they fold out.
//!
//! Achievement deviation: the C#'s `HandleSpecialEvent` targets ONE row's
//! condition (`GetCondition("STAR_POWER", "Use").Complete()`); the port's
//! `notify_custom("Use")` completes every `Flag("Use")` row at once (the
//! existing `potions.rs:491` drowning convention — the registry has no
//! per-row targeting). Recorded in docs/ANALYSIS.md.

use crate::app::App;
use crate::item::ItemSlot;
use crate::registry;
use crate::world_sim::editing::start_swing_extras;

/// One 60 Hz tick in seconds (the `editing.rs` constant).
const ONE_TICK_SECS: f32 = 1.0 / 60.0;

/// The integer tick count off a seconds timer (`editing.rs::ticks_of`).
fn ticks_of(secs: f32) -> i32 {
    (secs * 60.0).round() as i32
}

/// The held item's prefixed `useTime` in seconds — `Player.ApplyItemTime`
/// → `SetItemTime(sItem.useTime)` (`Player.cs:4416-4419`, the INSTANCE
/// value; `recall_teleport::use_time_secs` convention).
fn use_time_secs(held: ItemSlot) -> f32 {
    crate::prefix::prefixed_stats(held.kind, held.prefix)
        .map(|s| s.use_time as f32 * ONE_TICK_SECS)
        .filter(|t| *t > 0.0)
        .unwrap_or_else(|| {
            registry::get(held.kind)
                .map(|d| d.use_time as f32 * ONE_TICK_SECS)
                .filter(|t| *t > 0.0)
                .unwrap_or_else(|| crate::item::use_animation_secs_of(held.kind, held.prefix))
        })
}

/// The power-up kinds this machine owns — the `ItemCheck_Use*` cluster's
/// ids (`Player.cs:43595-43598`'s call row).
pub(crate) fn is_powerup(kind: u16) -> bool {
    matches!(kind, 29 | 109 | 1291 | 3335)
}

/// `Player.ItemCheck_CheckCanUse_Inner`'s Demon Heart refusal
/// (`Player.cs:52051-52054`): `extraAccessory || !Main.expertMode` refuses
/// the use START — no swing, no consume. The crystal/fruit caps gate the
/// GRANT, not the start (the C# has no cap leg here).
fn use_start_refused(app: &App, kind: u16) -> bool {
    if kind == 3335 {
        return app
            .player
            .as_ref()
            .is_some_and(|p| p.extra_accessory || !p.buffs.expert_mode);
    }
    false
}

/// The mid-swing grant dispatch — the C# call order
/// (`Player.cs:43595-43598`): LifeCrystal → LifeFruit → ManaCrystal →
/// DemonHeart. Returns true when the grant fired (the stack pays one).
fn run_powerup_arm(app: &mut App, held: ItemSlot) -> bool {
    if ticks_of(app.swing) <= 0 || ticks_of(app.item_time) != 0 {
        return false;
    }
    let fired = match held.kind {
        29 => use_life_crystal(app),
        1291 => use_life_fruit(app),
        109 => use_mana_crystal(app),
        3335 => use_demon_heart(app),
        _ => false,
    };
    if fired {
        // `ApplyItemTime(sItem)` (`:45310`/`:45278`) — the ACTION cadence
        // re-arms; the swing keeps running. Read off the pre-consume slot.
        app.item_time = use_time_secs(held);
        consume_one(app);
    }
    fired
}

/// The generic consumable tail (`Player.cs:43280-43326`): one per granted
/// use — `CanConsumeConsumableItem` is `return true` (`:5496-5499`), the
/// crystals are neither ranged nor coins nor ForceConsumption. The
/// 0-stack slot airs out on the consume tick (the food arm's `TurnToAir`
/// fold, `potions.rs:474-487`).
fn consume_one(app: &mut App) {
    let slot = &mut app.inventory[app.selected];
    if slot.count > 0 {
        slot.count -= 1; // `:43317-43320`
    }
    if slot.count == 0 {
        *slot = crate::item::EMPTY_SLOT; // `:42824-42828`
    }
}

/// `Player.ItemCheck_UseLifeCrystal` (`Player.cs:45306-45320`):
/// `statLifeMax < 400` → `statLifeMax += 20; statLifeMax2 += 20;
/// statLife += 20` + `HealEffect(20)`. `HandleSpecialEvent(this, 0)`
/// (`:45318`) is a NO-OP — the switch has no case 0
/// (`AchievementsHelper.cs:288-379`).
fn use_life_crystal(app: &mut App) -> bool {
    let Some(p) = app.player.as_mut() else {
        return false;
    };
    if p.life_max_base >= 400 {
        return false;
    }
    p.life_max_base += 20;
    p.max_hp += 20;
    p.hp += 20;
    // `HealEffect(20)` (`:45316` → `:4522-4529`): the RAW 20 text — the
    // crystal arm carries NO cap on `statLife` (the C# adds straight).
    crate::potions::spawn_heal_text(app, 20);
    true
}

/// `Player.ItemCheck_UseLifeFruit` (`Player.cs:45290-45304`):
/// `statLifeMax >= 400 && statLifeMax < 500` → `statLifeMax += 5;
/// statLifeMax2 += 5; statLife += 5` + `HealEffect(5)` +
/// `HandleSpecialEvent(2)` — GET_A_LIFE's "Use" flag, plus TOPPED_OFF when
/// both caps are full (`AchievementsHelper.cs:297-303`).
fn use_life_fruit(app: &mut App) -> bool {
    let (life_after, mana_base) = {
        let Some(p) = app.player.as_mut() else {
            return false;
        };
        if !(400..500).contains(&p.life_max_base) {
            return false;
        }
        p.life_max_base += 5;
        p.max_hp += 5;
        p.hp += 5;
        // `HealEffect(5)` (`:45300`) — the RAW 5 text, no statLife cap.
        crate::potions::spawn_heal_text(app, 5);
        (p.life_max_base, p.mana_max_base)
    };
    let _ = app.achievements.notify_custom("Use");
    if life_after == 500 && mana_base == 200 {
        let _ = app.achievements.notify_custom("Use");
    }
    true
}

/// `Player.ItemCheck_UseDemonHeart` (`Player.cs:45192-45200`):
/// `!extraAccessory && Main.expertMode` → `extraAccessory = true` (the 6th
/// accessory slot, `IsItemSlotUnlockedAndUsable` `:12587`). The start
/// refusal (`:52051-52054`) already shut the used/non-expert case; this
/// leg mirrors the C#'s own double gate. The `SendData(4)` sync (`:45198`)
/// is MP-only.
fn use_demon_heart(app: &mut App) -> bool {
    let Some(p) = app.player.as_mut() else {
        return false;
    };
    if p.extra_accessory || !p.buffs.expert_mode {
        return false;
    }
    p.extra_accessory = true;
    true
}

/// `Player.ItemCheck_UseManaCrystal` (`Player.cs:45274-45288`):
/// `statManaMax < 200` → `statManaMax += 20; statManaMax2 += 20;
/// statMana += 20` + `ManaEffect(20)` + `HandleSpecialEvent(1)` —
/// STAR_POWER's "Use" flag, plus TOPPED_OFF when both caps are full
/// (`AchievementsHelper.cs:290-296`).
fn use_mana_crystal(app: &mut App) -> bool {
    let (life_base, mana_after) = {
        let Some(p) = app.player.as_mut() else {
            return false;
        };
        if p.mana_max_base >= 200 {
            return false;
        }
        p.mana_max_base += 20;
        p.mana_max += 20;
        p.mana += 20;
        // `ManaEffect(20)` (`:45284` → `:4531-4538`): the pending-text seam
        // the world_sim player pass drains (`world_sim/player.rs:1400`).
        p.queue_mana_effect(20);
        (p.life_max_base, p.mana_max_base)
    };
    let _ = app.achievements.notify_custom("Use");
    if life_base == 500 && mana_after == 200 {
        let _ = app.achievements.notify_custom("Use");
    }
    true
}

/// The power-up `ItemCheck` machine — called once per tick from
/// `update_editing` beside the boss-spawner block. `press_edge` is the
/// caller's `prev_mine_released && !prev_mine_down` pair (the
/// `recall_item_check` convention). Returns true when it STARTED a use
/// this tick (the caller consumes the tick like the boss arms); a live
/// swing's per-tick grant runs without consuming.
pub(crate) fn powerup_item_check(app: &mut App, press_edge: bool) -> bool {
    let held = app.inventory[app.selected];
    if held.count == 0 || !is_powerup(held.kind) {
        return false;
    }
    // `:42598-42603` CCed / the dead player runs UpdateDead, not ItemCheck
    // (the recall machine's gate).
    if app.player.as_ref().is_some_and(|p| p.dead || p.cced()) {
        return false;
    }
    if ticks_of(app.swing) > 0 {
        run_powerup_arm(app, held);
        return false;
    }
    if app.mine_down && !app.inventory_open && press_edge {
        if use_start_refused(app, held.kind) {
            return false;
        }
        // `ItemCheck_StartActualUse` → `ApplyItemAnimation` (`:51680`):
        // the swing lasts useAnimation; `itemTime` stays 0 (the `:42836`
        // decrement only runs when > 0), so the grant's `ItemTimeIsZero`
        // leg fires THIS tick — the boss-spawner start shape.
        app.swing_max = crate::item::use_animation_secs_of(held.kind, held.prefix);
        app.swing = app.swing_max;
        start_swing_extras(app, held.kind);
        app.attack_cd = app.swing_max;
        app.reuse_delay = crate::prefix::prefixed_stats(held.kind, held.prefix)
            .map_or(0.0, |s| s.reuse_delay as f32 * ONE_TICK_SECS);
        app.item_time = 0.0;
        run_powerup_arm(app, held);
        app.mine_target = None;
        app.mine_timer = 0.0;
        return true;
    }
    false
}

// ---- witnesses (W4-a6) — WRITTEN NOT RUN (code-only swarm, no compiler) ----
//
// The rigs mirror `editing.rs::potion_press_app` / `recall_teleport`'s: the
// flow app's 8x8 air world, the player parked at the centre, the camera on
// the cursor cell, the SILENT listener so the use-sound draws nothing. The
// press edge is armed by one idle frame (`mine_prev_down = false`,
// `mine_released = true`), then `mine_down = true`.
#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::flow::front_flow_tests::flow_app;
    use crate::player::Player;
    use glam::Vec2;
    use tr_core::TILE_PX;

    const SCREEN: Vec2 = Vec2::new(1280.0, 720.0);

    fn rig(kind: u16, count: u32) -> App {
        let mut app = flow_app(None, vec![], vec![]);
        let mut p = Player::spawn(&app.world);
        p.x = 4.0 * TILE_PX;
        p.y = 4.0 * TILE_PX;
        app.player = Some(p);
        app.camera = Vec2::new(4.0 * TILE_PX + 8.0, 4.0 * TILE_PX + 8.0);
        app.mouse_pos = Vec2::new(640.0, 360.0);
        app.audio.set_listener(crate::sound::Listener {
            camera_center: (app.camera.x, app.camera.y),
            sound_volume: 0.0,
            ambient_volume: 1.0,
            quiet_ambient_sounds: false,
            game_menu: false,
            generating_or_loading: false,
            server: false,
        });
        app.inventory[app.selected] = ItemSlot {
            kind,
            name: registry::name(kind),
            count,
            prefix: 0,
        };
        app.mine_down = false;
        app.mine_prev_down = false;
        app.mine_released = true;
        app
    }

    fn tick(app: &mut App) {
        crate::world_sim::editing::update_editing(app, 1.0 / 60.0, SCREEN);
    }

    /// Life Crystal 29: +20 to the permanent base, the effective max and
    /// the live pool; one consumed; the RAW-20 HealLife text.
    #[test]
    fn life_crystal_grants_twenty_and_consumes_one() {
        let mut app = rig(29, 3);
        tick(&mut app); // idle: the press edge arms
        app.mine_down = true;
        tick(&mut app); // the use
        let p = app.player.as_ref().unwrap();
        assert_eq!(p.life_max_base, 120, "statLifeMax += 20");
        assert_eq!(p.max_hp, 120, "statLifeMax2 += 20");
        assert_eq!(p.hp, 120, "statLife += 20");
        assert_eq!(app.inventory[app.selected].count, 2, "one consumed");
        let t = app.combat_texts.iter().find(|t| t.active).expect("heal text");
        assert_eq!(t.text, "20");
        assert_eq!(t.color, crate::combat_text::HEAL_LIFE);
    }

    /// Holding LMB grants ONCE per press (the `:42733` press edge), not
    /// every tick — the stack drains by one, not thirty.
    #[test]
    fn holding_lmb_uses_one_crystal_not_every_tick() {
        let mut app = rig(29, 30);
        tick(&mut app);
        app.mine_down = true;
        for _ in 0..120 {
            tick(&mut app);
        }
        let p = app.player.as_ref().unwrap();
        assert_eq!(p.life_max_base, 120, "ONE grant per press");
        assert_eq!(app.inventory[app.selected].count, 29);
    }

    /// The 400 cap: a character already at 400 refuses the grant AND the
    /// consume (the C# `statLifeMax < 400` gate; `itemTimeMax` stays 0).
    #[test]
    fn life_crystal_at_cap_refuses_and_keeps_the_stack() {
        let mut app = rig(29, 3);
        {
            let p = app.player.as_mut().unwrap();
            p.life_max_base = 400;
            p.max_hp = 400;
            p.hp = 400;
        }
        tick(&mut app);
        app.mine_down = true;
        tick(&mut app);
        let p = app.player.as_ref().unwrap();
        assert_eq!(p.life_max_base, 400, "no grant at the cap");
        assert_eq!(app.inventory[app.selected].count, 3, "no consume on refusal");
    }

    /// Fifteen crystals walk 100 → 400 and then stop (the cap).
    #[test]
    fn life_crystals_stop_at_four_hundred() {
        let mut app = rig(29, 20);
        for _ in 0..20 {
            // A fresh press each use: release, idle, press.
            app.mine_down = false;
            tick(&mut app);
            app.attack_cd = 0.0;
            app.reuse_delay = 0.0;
            app.swing = 0.0;
            app.mine_down = true;
            tick(&mut app);
        }
        let p = app.player.as_ref().unwrap();
        assert_eq!(p.life_max_base, 400, "15 grants then the cap");
        assert_eq!(app.inventory[app.selected].count, 5, "20 - 15");
    }

    /// Mana Crystal 109: +20 to base/effective/live mana, one consumed,
    /// the ManaEffect text queued, STAR_POWER's "Use" flag fired.
    #[test]
    fn mana_crystal_grants_twenty_and_consumes_one() {
        let mut app = rig(109, 3);
        tick(&mut app);
        app.mine_down = true;
        tick(&mut app);
        let p = app.player.as_ref().unwrap();
        assert_eq!(p.mana_max_base, 40, "statManaMax += 20");
        assert_eq!(p.mana_max, 40, "statManaMax2 += 20");
        assert_eq!(p.mana, 40, "statMana += 20");
        assert_eq!(app.inventory[app.selected].count, 2);
        assert_eq!(p.take_mana_effect(), Some(20), "ManaEffect(20) queued");
        let star = app.achievements.list.iter().find(|a| a.name == "STAR_POWER").unwrap();
        assert!(star.completed, "HandleSpecialEvent(1) → STAR_POWER");
    }

    /// The 200 mana cap refuses the grant and the consume.
    #[test]
    fn mana_crystal_at_cap_refuses() {
        let mut app = rig(109, 3);
        {
            let p = app.player.as_mut().unwrap();
            p.mana_max_base = 200;
            p.mana_max = 200;
            p.mana = 200;
        }
        tick(&mut app);
        app.mine_down = true;
        tick(&mut app);
        let p = app.player.as_ref().unwrap();
        assert_eq!(p.mana_max_base, 200);
        assert_eq!(app.inventory[app.selected].count, 3, "no consume on refusal");
    }

    /// The permanent mana base survives the per-tick `statManaMax2` rebase
    /// (`ResetEffects` `:18389`) — the crystal's +20 is not wiped next tick.
    #[test]
    fn mana_crystal_base_survives_the_rebase() {
        let mut app = rig(109, 3);
        tick(&mut app);
        app.mine_down = true;
        tick(&mut app);
        let p = app.player.as_mut().unwrap();
        p.update(&app.world, crate::player::PlayerInput::default());
        assert_eq!(p.mana_max_base, 40, "the permanent cap persists");
        assert_eq!(p.mana_max, 40, "the rebase reads the grown base");
    }

    /// Life Fruit 1291: +5 to base/effective/live life at the 400 gate,
    /// one consumed, the RAW-5 heal text, GET_A_LIFE's "Use" flag.
    #[test]
    fn life_fruit_grants_five_at_four_hundred() {
        let mut app = rig(1291, 3);
        {
            let p = app.player.as_mut().unwrap();
            p.life_max_base = 400;
            p.max_hp = 400;
            p.hp = 400;
        }
        tick(&mut app);
        app.mine_down = true;
        tick(&mut app);
        let p = app.player.as_ref().unwrap();
        assert_eq!(p.life_max_base, 405, "statLifeMax += 5");
        assert_eq!(p.max_hp, 405);
        assert_eq!(p.hp, 405);
        assert_eq!(app.inventory[app.selected].count, 2);
        let t = app.combat_texts.iter().find(|t| t.active).expect("heal text");
        assert_eq!(t.text, "5");
        let life = app.achievements.list.iter().find(|a| a.name == "GET_A_LIFE").unwrap();
        assert!(life.completed, "HandleSpecialEvent(2) → GET_A_LIFE");
    }

    /// Life Fruit below 400 refuses (the `statLifeMax >= 400` gate) — no
    /// grant, no consume.
    #[test]
    fn life_fruit_below_four_hundred_refuses() {
        let mut app = rig(1291, 3);
        tick(&mut app);
        app.mine_down = true;
        tick(&mut app);
        let p = app.player.as_ref().unwrap();
        assert_eq!(p.life_max_base, 100, "no grant below 400");
        assert_eq!(app.inventory[app.selected].count, 3, "no consume on refusal");
    }

    /// Life Fruit at 500 refuses (the `statLifeMax < 500` gate).
    #[test]
    fn life_fruit_at_five_hundred_refuses() {
        let mut app = rig(1291, 3);
        {
            let p = app.player.as_mut().unwrap();
            p.life_max_base = 500;
            p.max_hp = 500;
            p.hp = 500;
        }
        tick(&mut app);
        app.mine_down = true;
        tick(&mut app);
        assert_eq!(app.player.as_ref().unwrap().life_max_base, 500);
        assert_eq!(app.inventory[app.selected].count, 3);
    }

    /// Twenty fruits walk 400 → 500 and then stop.
    #[test]
    fn life_fruits_stop_at_five_hundred() {
        let mut app = rig(1291, 25);
        {
            let p = app.player.as_mut().unwrap();
            p.life_max_base = 400;
            p.max_hp = 400;
            p.hp = 400;
        }
        for _ in 0..25 {
            app.mine_down = false;
            tick(&mut app);
            app.attack_cd = 0.0;
            app.reuse_delay = 0.0;
            app.swing = 0.0;
            app.mine_down = true;
            tick(&mut app);
        }
        assert_eq!(app.player.as_ref().unwrap().life_max_base, 500);
        assert_eq!(app.inventory[app.selected].count, 5, "25 - 20");
    }

    /// Demon Heart 3335 in expert mode: extraAccessory flips true, one
    /// consumed (the 6th accessory slot unlock).
    #[test]
    fn demon_heart_unlocks_the_extra_slot_in_expert() {
        let mut app = rig(3335, 2);
        app.player.as_mut().unwrap().buffs.expert_mode = true;
        tick(&mut app);
        app.mine_down = true;
        tick(&mut app);
        assert!(app.player.as_ref().unwrap().extra_accessory);
        assert_eq!(app.inventory[app.selected].count, 1);
    }

    /// Demon Heart outside expert mode refuses the use START
    /// (`:52051-52054`) — no swing, no grant, no consume.
    #[test]
    fn demon_heart_outside_expert_refuses() {
        let mut app = rig(3335, 2);
        tick(&mut app);
        app.mine_down = true;
        tick(&mut app);
        assert!(!app.player.as_ref().unwrap().extra_accessory);
        assert_eq!(app.inventory[app.selected].count, 2, "no consume");
        assert_eq!(app.swing, 0.0, "no swing on the refused start");
    }

    /// A second Demon Heart refuses (extraAccessory already set) — the
    /// stack keeps its remainder.
    #[test]
    fn a_second_demon_heart_refuses() {
        let mut app = rig(3335, 2);
        app.player.as_mut().unwrap().buffs.expert_mode = true;
        tick(&mut app);
        app.mine_down = true;
        tick(&mut app);
        assert_eq!(app.inventory[app.selected].count, 1);
        // Release + re-press with the flag already set.
        app.mine_down = false;
        tick(&mut app);
        app.attack_cd = 0.0;
        app.reuse_delay = 0.0;
        app.swing = 0.0;
        app.mine_down = true;
        tick(&mut app);
        assert_eq!(app.inventory[app.selected].count, 1, "the second refuses");
    }
}