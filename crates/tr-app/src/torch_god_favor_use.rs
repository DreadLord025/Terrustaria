//! W4-f33 — the Torch God's Favor (5043) permanent-upgrade USE path:
//! `Player.ItemCheck_UseTorchGodsFavor` (`Player.cs:45226-45243`), wired at
//! the `ItemCheck_OwnerOnlyCode` permanent-item seam (`:43600`, between
//! `ItemCheck_UseMinecartPowerUp` `:43599` and `ItemCheck_UseArtisanLoaf`
//! `:43601`).
//!
//! The gate (`:45228`): `type == 5043 && itemAnimation > 0 &&
//! !unlockedBiomeTorches && ItemTimeIsZero`. On it: `ApplyItemTime(sItem)`
//! (`:45230`), `unlockedBiomeTorches = true` (`:45231`), `UsingBiomeTorches =
//! true` (`:45232` → the `builderAccStatus[11] = 0` setter, `:3960-3963`),
//! `HandleSpecialEvent(24)` (`:45233` → `GAIN_TORCH_GODS_FAVOR` "Use",
//! `AchievementsHelper.cs:367-369`), and — single-player only (`netMode == 0`)
//! — a `SetDefaults(664)` NPC's bestiary kill credit (`:45234-45239`, the
//! "TorchGod" row, `NPCID.cs:12005`, `NPC.cs:17398-17402` lifeMax 20 boss).
//! The `SendData(4)` / `SendData(51, …, 5f)` tails (`:45240-45241`) are
//! net-only.
//!
//! ## The machine shape
//!
//! The press-edge / mid-swing `ItemTimeIsZero` machine is the recall arm's
//! (`recall_teleport::recall_item_check`, the boss-spawner precedent): the
//! use START (`ItemCheck_StartActualUse` → `ApplyItemAnimation`, `:51680`)
//! runs on the press tick, the grant fires on the first action tick inside
//! the animation and `ApplyItemTime` (`:4416-4419`) re-arms the cadence. The
//! 5043 is `useStyle 4` / `useTime = useAnimation = 30` (`Item.cs:40438-40448`),
//! `consumable`, `autoReuse: false` — one use per press, the grant on the
//! press tick itself (the start leaves `itemTime` at 0 while the animation
//! is full, `:4066` `ItemAnimationJustStarted`).
//!
//! ## The generic consumable tail
//!
//! `Player.cs:43280-43326`: `itemTimeMax != 0 && itemTime == itemTimeMax &&
//! item.consumable && !context.SkipItemConsumption` → the stack pays one
//! (`:43317-43320`), and a 0-stack re-arms `itemTime = itemAnimation`
//! (`:43323`) — the C# leaves the air slot until the next tick's `TurnToAir`
//! (`:42824-42828`); the port folds the air-out onto the consume tick (the
//! food/drink arms' convention, `potions.rs:474-487`). 5043 is neither ranged
//! nor a coin, `CanConsumeConsumableItem` is `return true` (`:5496-5499`) and
//! `ItemID.Sets.ForceConsumption` (`ItemID.cs:413-427`) does not list it, so
//! `flag7` stays true. The tail only pays when `ApplyItemTime` ran this tick
//! (`itemTime == itemTimeMax`) — a REFUSED use (already unlocked) never
//! consumes.
//!
//! ## Scope / deviations (ANALYSIS "The Torch God event + favor")
//!
//! * The unlock is PERMANENT (`unlockedBiomeTorches`, the `.plr` field
//!   `:54777`/`:55241`); the `.trplayer` sidecar carries it (version 7) so a
//!   world reload keeps the biome torches. The builder toggle
//!   (`builderAccStatus[11]`) folds to 0/on in play (the port's
//!   `using_biome_torches` reads the unlock directly, the `player_lights.rs`
//!   convention) — the use arm still writes the slot verbatim through
//!   `Player::set_using_biome_torches`.
//! * The bestiary credit is the free-form `"TorchGod"` string
//!   (`NPCID.Search.GetName(664)`); the port's `KillTracker` map carries it
//!   without a catalog row (the 664 NPC is not a spawnable `NpcKind`).
//!   `SetKillCountDirectly`'s new-key `TryGrantingBestiary100PercentAchievement`
//!   hook (`NPCKillsTracker.cs:46-49`) runs through the App's net-aware seam.
//! * `HandleSpecialEvent(24)` rides the achievement dispatcher
//!   (`AchievementManager::handle_special_event` → `complete_condition`, the
//!   NAMED `GetCondition("GAIN_TORCH_GODS_FAVOR", "Use").Complete()` — not the
//!   broadcast `notify_custom("Use")`, which would complete every "Use" row).
//! * The use-start `UseSound` (Item4, `Item.cs:40444`) plays through
//!   `start_swing_extras` (`Player.cs:51681-51698`); its case-2 pitch draw
//!   rides the `UpdatePlayers` stream (`Main.cs:17754`). The grant itself
//!   draws NOTHING.

use crate::app::App;
use crate::item::ItemSlot;
use crate::world_sim::editing::start_swing_extras;

/// `ItemID.TorchGodsFavor` (`ItemID.cs:11769`).
pub(crate) const TORCH_GODS_FAVOR: u16 = 5043;

/// One 60 Hz tick in seconds — the fixed step the item timers pace against.
const ONE_TICK_SECS: f32 = 1.0 / 60.0;

/// The C#'s INTEGER tick comparisons out of the port's per-frame seconds
/// timers (`editing.rs::ticks_of`): `round(secs * 60)`.
fn ticks_of(secs: f32) -> i32 {
    (secs * 60.0).round() as i32
}

/// The held item's prefixed `useTime` in seconds (`Player.ApplyItemTime` →
/// `SetItemTime(sItem.useTime)`, `:4416-4419`, the INSTANCE value) — the
/// `recall_teleport::use_time_secs` convention.
fn use_time_secs(held: ItemSlot) -> f32 {
    crate::prefix::prefixed_stats(held.kind, held.prefix)
        .map(|s| s.use_time as f32 * ONE_TICK_SECS)
        .filter(|t| *t > 0.0)
        .unwrap_or_else(|| crate::item::use_time_secs(held.kind))
}

/// `Player.ItemCheck_UseTorchGodsFavor` (`Player.cs:45226-45243`) — the
/// mid-swing grant. Returns true when the unlock fired (the caller consumes
/// the tick like the other permanent-upgrade arms). The gate reads the
/// post-decrement `itemAnimation`/`itemTime` the `update_editing` head decay
/// left (`:42814`/`:42836` run before `ItemCheck_OwnerOnlyCode` `:42854`).
fn run_favor_arm(app: &mut App, held: ItemSlot) -> bool {
    // `:45228` — itemAnimation > 0 && !unlockedBiomeTorches && ItemTimeIsZero.
    if ticks_of(app.swing) <= 0 || ticks_of(app.item_time) != 0 {
        return false;
    }
    let Some(p) = app.player.as_mut() else {
        return false;
    };
    if p.unlocked_biome_torches {
        return false;
    }
    // `:45230` — ApplyItemTime(sItem): itemTime = itemTimeMax = useTime.
    app.item_time = use_time_secs(held);
    // `:45231-45232` — the permanent unlock + the UsingBiomeTorches setter
    // (`builderAccStatus[11] = 0`, `:3960-3963`).
    p.unlocked_biome_torches = true;
    p.set_using_biome_torches(true);
    // `:45233` — HandleSpecialEvent(24) → GAIN_TORCH_GODS_FAVOR "Use"
    // (`AchievementsHelper.cs:367-369`), the NAMED condition.
    let _ = app.achievements.handle_special_event(24);
    // `:45234-45239` — the single-player bestiary kill credit: a fresh
    // `SetDefaults(664)` NPC's "TorchGod" row. `Main.netMode == 0` is the
    // port's `NetRole::Off`; a host/client skips it (the C# gate).
    if matches!(app.net, crate::app::NetRole::Off) {
        app.bestiary.kills.register_kill("TorchGod");
        // `NPCKillsTracker.SetKillCountDirectly`'s new-key hook
        // (`NPCKillsTracker.cs:46-49`): the 100% completion check. The
        // net-aware App seam (a host broadcasts the `98` wire).
        let _ = app.try_grant_bestiary_100_net();
    }
    // `:45240-45241` — SendData(4) / SendData(51, …, 5f): net-only.
    // `:43280-43326` — the generic consumable tail: ApplyItemTime just ran
    // (itemTime == itemTimeMax), so the stack pays one; the 0-stack re-arms
    // itemTime = itemAnimation (`:43323`) then airs out (the TurnToAir fold).
    consume_one(app);
    true
}

/// The generic consumable tail (`Player.cs:43280-43326`) folded onto the
/// grant tick: one per use, then the 0-stack `itemTime = itemAnimation`
/// re-arm (`:43323`) + TurnToAir (`:42824-42828`, the drink seam's
/// convention `potions.rs:474-487`).
fn consume_one(app: &mut App) {
    let slot = &mut app.inventory[app.selected];
    if slot.count > 0 {
        slot.count -= 1; // `:43317-43320`
    }
    if slot.count == 0 {
        // `:43323` — the empty stack re-arms the action cadence to the
        // remaining animation BEFORE the slot airs out.
        app.item_time = app.swing;
        *slot = crate::item::EMPTY_SLOT; // `:42824-42828`
    }
}

/// The 5043 `ItemCheck` machine, called once per tick from `update_editing`
/// (the permanent-item seam, `Player.cs:43600` — after the recall arm, before
/// the boss spawners, the C# `ItemCheck_OwnerOnlyCode` order). `press_edge` is
/// the caller's computed `prev_mine_released && !prev_mine_down`. Returns true
/// when it STARTED a use this tick (the caller consumes the tick).
///
/// The start gate is the C#'s `controlUseItem && releaseUseItem &&
/// itemAnimation == 0 && useStyle != 0` press edge (`Player.cs:42733`); the
/// 5043 is `autoReuse: false`. The per-tick arm runs whenever the swing is
/// live (`itemAnimation > 0`), even with the button released. `CCed` zeroes
/// the animation (`:42598-42603`); the dead player runs `UpdateDead` instead
/// of `ItemCheck`.
pub(crate) fn torch_gods_favor_item_check(app: &mut App, press_edge: bool) -> bool {
    let held = app.inventory[app.selected];
    if held.count == 0 || held.kind != TORCH_GODS_FAVOR {
        return false;
    }
    // `:42598-42603` CCed → itemAnimation = itemAnimationMax = 0, return.
    if app.player.as_ref().is_some_and(|p| p.dead || p.cced()) {
        app.swing = 0.0;
        app.swing_max = 0.0;
        app.item_time = 0.0;
        return false;
    }
    if ticks_of(app.swing) > 0 {
        // The mid-swing per-tick arm (the grant already fired on the start
        // tick for a fresh unlock; a refused use keeps re-checking).
        run_favor_arm(app, held);
        return false;
    }
    // The press edge: button down now, was up last tick.
    if app.mine_down && !app.inventory_open && press_edge {
        // `ItemCheck_StartActualUse` → ApplyItemAnimation (`:51680`): the
        // swing lasts useAnimation; the use-start sound (Item4) plays through
        // `start_swing_extras` (`:51681-51698`).
        app.swing_max = crate::item::use_animation_secs_of(held.kind, held.prefix);
        app.swing = app.swing_max;
        start_swing_extras(app, held.kind);
        app.attack_cd = app.swing_max;
        // The C# start tick sees itemTime == 0 (StartActualUse leaves it; the
        // :42836 decrement only runs when > 0) → the arm's ItemTimeIsZero leg
        // fires THIS tick (ApplyItemTime + the unlock).
        app.item_time = 0.0;
        run_favor_arm(app, held);
        app.mine_target = None;
        app.mine_timer = 0.0;
        return true;
    }
    false
}

// ---- witnesses (W4-f33) — WRITTEN NOT RUN (code-only mode, no compiler) ----
//
// The grant's gates, the ApplyItemTime cadence, the consumable tail, the
// achievement + bestiary side effects and the RNG silence — each pinned to
// the C# lines above. The audio is MUTED in the rig so the use-start Item4
// case-2 pitch draw stays off the UpdatePlayers stream (the attenuation gate
// refuses before the draw, `sound.rs` — the recall rig's convention); the
// grant itself draws nothing, so the stream stays pristine either way.
#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::flow::front_flow_tests::flow_app;
    use crate::player::Player;
    use crate::registry;

    /// A headless rig holding `count` favors, audio muted (the use-start cue
    /// draws nothing on the UpdatePlayers stream — the grant is silent).
    fn rig(count: u32) -> App {
        let mut app = flow_app(None, vec![], vec![]);
        let p = Player::spawn(&app.world);
        app.player = Some(p);
        app.inventory[app.selected] = ItemSlot {
            kind: TORCH_GODS_FAVOR,
            name: registry::name(TORCH_GODS_FAVOR),
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

    #[test]
    fn w4_f33_the_favor_use_unlocks_the_biome_torches_and_pays_one() {
        // `Player.cs:45228-45232` — the gate fires on the press tick:
        // ApplyItemTime(30), unlockedBiomeTorches = true, UsingBiomeTorches
        // (builderAccStatus[11] = 0); the consumable tail pays one.
        let mut app = rig(2);
        press(&mut app);
        assert!(torch_gods_favor_item_check(&mut app, true), "the use started");
        let p = app.player.as_ref().unwrap();
        assert!(p.unlocked_biome_torches, "the permanent unlock");
        assert!(p.using_biome_torches(), "UsingBiomeTorches = true");
        assert_eq!(p.builder_acc_status[11], 0);
        // ApplyItemTime: itemTime = useTime = 30 ticks (0.5 s).
        assert_eq!(ticks_of(app.item_time), 30);
        // The swing is the 30-tick animation; the stack paid one (2 → 1).
        assert_eq!(ticks_of(app.swing), 30);
        assert_eq!(app.inventory[app.selected].count, 1);
        assert_eq!(app.inventory[app.selected].kind, TORCH_GODS_FAVOR);
    }

    #[test]
    fn w4_f33_the_favor_use_grants_the_achievement_and_the_bestiary_credit() {
        // `:45233` HandleSpecialEvent(24) → GAIN_TORCH_GODS_FAVOR "Use";
        // `:45234-45239` the SP bestiary kill credit "TorchGod" (NPC 664).
        let mut app = rig(1);
        press(&mut app);
        assert!(torch_gods_favor_item_check(&mut app, true));
        assert!(
            app.achievements
                .list
                .iter()
                .any(|a| a.name == "GAIN_TORCH_GODS_FAVOR" && a.completed),
            "the named Use condition completed the row"
        );
        // The named complete leaves the OTHER "Use" rows open (the broadcast
        // notify_custom would have closed them — the dispatcher's point).
        assert!(
            app.achievements
                .list
                .iter()
                .any(|a| a.name == "STAR_POWER" && !a.completed),
            "STAR_POWER's Use row stays open"
        );
        assert_eq!(app.bestiary.kills.get_kill_count("TorchGod"), 1);
        // The 0-stack airs out (the TurnToAir fold) and re-arms itemTime to
        // the remaining animation (`:43323`).
        assert_eq!(app.inventory[app.selected].kind, 0);
        assert_eq!(app.inventory[app.selected].count, 0);
        assert_eq!(ticks_of(app.item_time), ticks_of(app.swing));
    }

    #[test]
    fn w4_f33_the_favor_use_is_once_ever_and_refusal_is_free() {
        // `:45228` !unlockedBiomeTorches — a second use refuses (no consume,
        // no re-grant); the stack keeps its count.
        let mut app = rig(3);
        press(&mut app);
        assert!(torch_gods_favor_item_check(&mut app, true));
        assert_eq!(app.inventory[app.selected].count, 2);
        // A fresh press while already unlocked: the swing starts but the arm
        // refuses — the consumable tail never runs (itemTime stays 0).
        app.swing = 0.0;
        app.item_time = 0.0;
        app.attack_cd = 0.0;
        press(&mut app);
        assert!(torch_gods_favor_item_check(&mut app, true), "the swing starts");
        assert_eq!(app.inventory[app.selected].count, 2, "the refusal is free");
        assert_eq!(ticks_of(app.item_time), 0, "no ApplyItemTime on refusal");
        assert_eq!(app.bestiary.kills.get_kill_count("TorchGod"), 1, "one credit");
    }

    #[test]
    fn w4_f33_the_favor_use_needs_the_press_edge_and_a_live_stack() {
        // No press edge → no start; an empty slot → no machine.
        let mut app = rig(1);
        app.mine_down = true; // held, but no edge
        assert!(!torch_gods_favor_item_check(&mut app, false));
        assert!(!app.player.as_ref().unwrap().unlocked_biome_torches);
        assert_eq!(app.inventory[app.selected].count, 1);
        // Empty hand.
        app.inventory[app.selected] = crate::item::EMPTY_SLOT;
        press(&mut app);
        assert!(!torch_gods_favor_item_check(&mut app, true));
    }

    #[test]
    fn w4_f33_the_favor_use_draws_nothing_on_the_update_players_stream() {
        // The grant itself draws NOTHING (the unlock + achievement + bestiary
        // are state writes); the use-start Item4 cue is muted in the rig, so
        // its case-2 pitch draw stays off the stream. Every RngKey stays at
        // its world-seed birth state.
        let mut app = rig(1);
        let before: Vec<_> = crate::app::RngKey::ALL
            .iter()
            .map(|k| (*k, app.rngs.stream(*k).clone()))
            .collect();
        press(&mut app);
        assert!(torch_gods_favor_item_check(&mut app, true));
        for (k, state) in before {
            assert_eq!(
                app.rngs.stream(k),
                &state,
                "the {:?} stream stayed pristine",
                k
            );
        }
    }

    #[test]
    fn w4_f33_a_dead_or_cced_player_cannot_use_the_favor() {
        // `:42598-42603` CCed zeroes the animation; the dead player runs
        // UpdateDead, not ItemCheck.
        let mut app = rig(1);
        app.player.as_mut().unwrap().dead = true;
        press(&mut app);
        assert!(!torch_gods_favor_item_check(&mut app, true));
        assert!(!app.player.as_ref().unwrap().unlocked_biome_torches);
        assert_eq!(app.inventory[app.selected].count, 1);
    }
}
