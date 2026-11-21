//! W4-d16 — the kite items' use-path (`ItemID.Sets.IsAKite`,
//! `ID/ItemID.cs:411` — the SAME 24 ids as `PlaceTileOnAltUse` `:412` and
//! `HasAProjectileThatHasAUsabilityCheck`): the LMB launch
//! (`ItemCheck_Shoot`'s generic tail `Player.cs:49437` — the kite kinds
//! carry no per-item arm), the RMB channel arm (`ItemCheck_OwnerOnlyCode`
//! `:43537-43541` — `altFunctionUse == 2 && holdStyle != 0` →
//! `StartChanneling()`, the PARAMETERLESS overload `:52952-52956` — no
//! cancel key), the channel release (`ItemCheck`'s flag5 `:42792-42794`
//! reads `controlUseTile` for kites, NOT `controlUseItem`), the holdStyle
//! hack (`ItemCheck_HackHoldStyles` `:52858-52874`) and the use refusals
//! (`ItemCheck_CheckCanUse_Inner`'s pulley gate `:52059-52062` +
//! `CheckUsability` `Projectile.cs:53466-53472` via
//! `ItemCheck_CheckUsabilityOfProjectiles` `:52355-52366` — a live kite
//! refuses EVERY new use, LMB and RMB alike). The RMB anchor placement
//! rides the generic tile route (`PlaceThing_Tiles` `:39743` — the kite
//! rows are `createTile = 723`, `Item.DefaultToKite` `Item.cs:47837`).
//!
//! Skips (ANALYSIS "W4-d16 kite residuals"): the per-slot `holdStyle`
//! mutation has no port item-instance field — the scan derives the flag
//! per tick from the pool (the fishing-bobber hack's shape, `:52844-52856`);
//! the hack's `ChangeDir` is render-side (the `kite_step` facing write
//! precedent); `TryInterruptingItemUsage` (`Player.cs:24162-24245`) +
//! `Projectile.Interrupt` (`:22179-22188`) have NO caller in the
//! decompiled tree (census — the hotbar-switch kill rides the AI's
//! held-shoot mismatch gate instead); the afk auto-kite
//! (`PickItemSelectionOverride` `:17421-17424`) keeps the hotbar.rs:67
//! afkCounter skip. CODE-ONLY MODE: the witnesses are WRITTEN NOT RUN.

use glam::Vec2;

use crate::app::App;
use crate::item::ItemSlot;
use crate::projectile_ai_kites::KITE_KINDS;
use crate::registry;

/// `ItemID.Sets.IsAKite` (`ID/ItemID.cs:411`): the 24 kite item ids —
/// 4367-4371, 4379, 4610-4613, 4648-4651, 4669-4671, 4674-4677, 4681,
/// 4683-4684 (`Item.cs:37239-37244`/`:37281-37282`/`:38246-38253`/
/// `:38275-38276`/`:38359-38363`/`:38501-38508`/`:38525-38536`/
/// `:38551-38553`/`:38563-38567`).
pub(super) const KITE_ITEMS: [u16; 24] = [
    4367, 4368, 4369, 4370, 4371, 4379, 4610, 4611, 4612, 4613, 4648, 4649,
    4650, 4651, 4669, 4670, 4671, 4674, 4675, 4676, 4677, 4681, 4683, 4684,
];

pub(super) fn is_kite(kind: u16) -> bool {
    KITE_ITEMS.contains(&kind)
}

/// `ItemCheck_HackHoldStyles`' kite half (`Player.cs:52858-52874`):
/// holdStyle 1 while a same-shoot projectile of this owner is live and
/// the use timers are zero (`!ItemTimeIsZero || itemAnimation != 0`
/// returns early with holdStyle 0). The port derives the flag per tick
/// (the module-doc skip — no item-instance field).
pub(super) fn kite_hold_out(app: &App, held: ItemSlot) -> bool {
    if !is_kite(held.kind) || held.count == 0 {
        return false;
    }
    if app.item_time > 0.0 || app.swing > 0.0 {
        return false;
    }
    let Some(shoot) = registry::projectile(held.kind) else {
        return false;
    };
    app.projectiles.slots().iter().any(|p| {
        p.active && p.owner == app.local_id as u8 && p.kind == shoot
    })
}

/// `ItemCheck_CheckCanUse_Inner`'s kite refusals: the pulley gate
/// (`Player.cs:52059-52062`) and `CheckUsability`
/// (`Projectile.cs:53466-53472` — aiStyle 160 sets `currentlyUsable =
/// false` for ANY live same-owner kite; the
/// `HasAProjectileThatHasAUsabilityCheck` set is the same 24 ids).
pub(super) fn kite_use_refused(app: &App, held: ItemSlot) -> bool {
    if !is_kite(held.kind) || held.count == 0 {
        return false;
    }
    if app.player.as_ref().is_some_and(|p| p.pulley) {
        return true;
    }
    app.projectiles.slots().iter().any(|p| {
        p.active
            && p.owner == app.local_id as u8
            && KITE_KINDS.contains(&p.kind)
    })
}

/// The kite channel release (`ItemCheck`'s flag5, `Player.cs:42787-42807`
/// with the `:42792-42794` kite override — `flag5 = controlUseTile`):
/// while the held item is a kite, the latch holds on the RIGHT button,
/// not the left. Runs at the hoisted tick-head clear (the i23 seam) —
/// the generic `!mine_down` clear would drop a held RMB channel.
pub(super) fn kite_channel_release(app: &App, held: ItemSlot) -> bool {
    is_kite(held.kind)
        && held.count > 0
        && (!app.place_down
            || app.inventory_open
            || app.map.as_ref().is_some_and(|m| m.open)
            || app.minimap_hover.is_some()
            // `:42800-42803` — the buffered switch forces flag5 false for
            // EVERY item, kites included.
            || app.selection.has_buffered_change())
}

/// The RMB channel arm (`ItemCheck_OwnerOnlyCode` `Player.cs:43537-43541`:
/// `altFunctionUse == 2 && IsAKite && holdStyle != 0` → `StartChanneling()`
/// — the parameterless overload, NO cancel key). The port's alt-2 stand-in
/// is the right-button press edge (the M4-PROJ-8 targeting convention).
/// Returns whether the channel armed.
pub(super) fn kite_channel_arm(app: &mut App, held: ItemSlot, hold_out: bool) -> bool {
    if !is_kite(held.kind) || held.count == 0 || !hold_out {
        return false;
    }
    if !app.place_down || app.inventory_open {
        return false;
    }
    let Some(p) = app.player.as_mut() else {
        return false;
    };
    // `StartChanneling()` (`:52952-52956`): channel = true, the cancel
    // key DEFAULTS (no expected type/index — a tracked-shot death never
    // drops this channel; only the release/CC/switch does). The live
    // kite that made holdStyle 1 true also trips `CheckUsability` on any
    // NEW use — the arm itself is not a use start (the C# block runs
    // inside `ItemCheck_OwnerOnlyCode` regardless of flag2).
    p.channel = true;
    p.channel_item = held.kind;
    p.channel_proj_expected = 0;
    p.channel_index_expected = 0;
    p.channel_shoot = 0;
    true
}

/// The LMB launch (`ItemCheck_Shoot`'s generic tail `Player.cs:49437` —
/// the kite kinds carry no per-item arm; the spawn point is
/// `RotatedRelativePoint(MountedCenter)` `:47196`, the aim normalized to
/// `shootSpeed` `:47277-47307`, damage `GetWeaponDamage` (the rows are
/// −1 — the kite is a marker, not a shot), knockback `GetWeaponKnockback`).
/// The happy-windy-day achievement (`:49458-49461`,
/// `AchievementsHelper.cs:319-321` → FLY_A_KITE_ON_A_WINDY_DAY "Use")
/// fires on the spawn tick. Returns whether the launch fired.
pub(super) fn kite_launch(app: &mut App, held: ItemSlot, world_pos: Vec2) -> bool {
    let Some(def) = registry::get(held.kind) else {
        return false;
    };
    let Some(proj) = registry::projectile(held.kind) else {
        return false;
    };
    if kite_use_refused(app, held) {
        return false;
    }
    let (spawn, raw, kb, shot) = {
        let Some(p) = app.player.as_ref() else {
            return false;
        };
        let (mcx, mcy) = p.mounted_center();
        let spawn = p.rotated_relative_point(Vec2::new(mcx, mcy), 0);
        let dir = (world_pos - spawn).normalize_or_zero();
        if dir == Vec2::ZERO {
            return false; // the C# zero-aim arm fires nothing for kites
        }
        let fx = crate::prefix::prefixed_stats(held.kind, held.prefix);
        let (dmg, spd, kb) = fx.map_or(
            (def.damage, def.shoot_speed, def.knock_back),
            |s| (s.damage, s.shoot_speed, s.knock_back),
        );
        let raw = dmg as f32 * p.weapon_damage_multiplier(def.class);
        let kb = p.weapon_knockback(def, kb);
        (spawn, raw, kb, dir * spd)
    };
    let windy = app.wind.happy_windy_day;
    let slot = app.projectiles.spawn_with_data_ai_owned(
        proj, app.local_id as u8, spawn.x, spawn.y, shot.x, shot.y,
        raw as i32, kb, 0.0, 0.0,
    );
    if slot.is_some() && windy {
        let _ = app.achievements.notify_custom("Use");
    }
    slot.is_some()
}

#[cfg(test)]
mod w4_d16_kite_use_tests {
    //! W4-d16 — the kite use-path witnesses. WRITTEN NOT RUN (code-only
    //! mode). The 24-id census vs the item rows, the holdStyle hack, the
    //! use refusals (pulley + live-kite CheckUsability), the LMB launch
    //! (one spawn, the achievement gate), the RMB channel arm + the
    //! flag5 release override.
    use super::*;
    use crate::app::flow::front_flow_tests::flow_app;
    use tr_core::World;

    fn kite_app(kind: u16) -> App {
        let mut w = World::new("t".into(), 64, 800);
        w.compute_column_surface();
        let mut app = flow_app(None, vec![], vec![]);
        app.world = w;
        app.local_id = 0;
        let mut pl = crate::player::Player::spawn(&app.world);
        pl.x = 30.0 * 16.0;
        pl.y = 250.0 * 16.0;
        app.player = Some(pl);
        app.camera = Vec2::new(30.0 * 16.0 + 8.0, 250.0 * 16.0 + 8.0);
        app.mouse_pos = Vec2::new(640.0, 360.0); // the 1280x720 centre
        app.inventory[app.selected] = ItemSlot { kind, name: "", count: 1, prefix: 0 };
        app.mine_down = false;
        app
    }

    /// The 24-id census (`ID/ItemID.cs:411`): every id has an item row
    /// whose shoot is a DefaultToKite kind, createTile 723, useStyle 1,
    /// useTime/useAnimation 15, shootSpeed 8, channel FALSE (the kite's
    /// channel is the parameterless `StartChanneling()` arm, not the
    /// item flag — `Item.DefaultToKite` `Item.cs:47828-47838` →
    /// `DefaultToThrownWeapon(projId, 15, 8f)` `:48455-48466`).
    #[test]
    fn w4_d16_kite_the_item_rows_match_default_to_kite() {
        assert_eq!(KITE_ITEMS.len(), KITE_KINDS.len());
        for (i, &item) in KITE_ITEMS.iter().enumerate() {
            let d = registry::get(item).unwrap_or_else(|| panic!("item {item}"));
            assert_eq!(d.projectile, Some(KITE_KINDS[i]), "item {item}");
            assert_eq!(d.tile, Some(723), "the anchor tile");
            assert_eq!(d.use_style, 1);
            assert_eq!((d.use_time, d.use_animation), (15, 15));
            assert_eq!(d.shoot_speed, 8.0);
            assert!(!d.channel, "the item flag stays false");
            assert!(!d.consumable && !d.auto_reuse);
            assert_eq!((d.width, d.height), (20, 28));
        }
        // The three sets are the SAME 24 ids (census: IsAKite `:411`,
        // PlaceTileOnAltUse `:412`, HasAProjectileThatHasAUsabilityCheck).
        assert!(is_kite(4367) && is_kite(4684) && !is_kite(4682));
    }

    /// The holdStyle hack (`Player.cs:52862-52874`): holdStyle 1 only
    /// while a same-shoot projectile lives AND the use timers are zero.
    #[test]
    fn w4_d16_kite_the_hold_style_hack() {
        let mut app = kite_app(4367); // shoot 766
        let held = app.inventory[app.selected];
        assert!(!kite_hold_out(&app, held), "no kite out");
        app.projectiles.spawn_with_data_ai_owned(766, 0, 500.0, 4000.0, 0.0, 0.0, -1, 0.0, 0.0, 0.0);
        assert!(kite_hold_out(&app, held), "the live kite holds");
        app.swing = 0.2;
        assert!(!kite_hold_out(&app, held), "the animation gate");
        app.swing = 0.0;
        app.item_time = 0.1;
        assert!(!kite_hold_out(&app, held), "the itemTime gate");
    }

    /// The use refusals: the pulley (`Player.cs:52059-52062`) and any
    /// live same-owner kite (`CheckUsability` `Projectile.cs:53468`).
    #[test]
    fn w4_d16_kite_the_use_refusals() {
        let mut app = kite_app(4367);
        let held = app.inventory[app.selected];
        assert!(!kite_use_refused(&app, held));
        app.player.as_mut().unwrap().pulley = true;
        assert!(kite_use_refused(&app, held), "the pulley refuses");
        app.player.as_mut().unwrap().pulley = false;
        // A FOREIGN owner's kite does not refuse (the owner gate).
        app.projectiles.spawn_with_data_ai_owned(766, 1, 500.0, 4000.0, 0.0, 0.0, -1, 0.0, 0.0, 0.0);
        assert!(!kite_use_refused(&app, held));
        app.projectiles.spawn_with_data_ai_owned(767, 0, 500.0, 4000.0, 0.0, 0.0, -1, 0.0, 0.0, 0.0);
        assert!(kite_use_refused(&app, held), "ANY own kite refuses (not just the held shoot)");
    }

    /// The LMB launch: one press spawns ONE kite at the mounted centre,
    /// the aim scaled to shootSpeed 8; the refused second press spawns
    /// nothing. The windy-day achievement fires on the spawn tick.
    #[test]
    fn w4_d16_kite_the_lmb_launch() {
        let mut app = kite_app(4367);
        // The cursor 80 px right + 20 px above the player's mounted
        // centre (the aim is computed from the SPAWN point, not the
        // camera — `Player.cs:47277-47278`).
        let (mcx, mcy) = app.player.as_ref().unwrap().mounted_center();
        app.mouse_pos = Vec2::new(
            640.0 + (mcx + 80.0 - app.camera.x),
            360.0 + (mcy - 20.0 - app.camera.y),
        );
        let held = app.inventory[app.selected];
        let world_pos = Vec2::new(mcx + 80.0, mcy - 20.0);
        assert!(kite_launch(&mut app, held, world_pos), "the launch fires");
        let live: Vec<_> = app.projectiles.slots().iter().filter(|p| p.active && p.kind == 766).collect();
        assert_eq!(live.len(), 1);
        assert_eq!(live[0].owner, 0);
        let want = Vec2::new(80.0, -20.0).normalize() * 8.0;
        assert!((live[0].vel - want).length() < 1e-3, "the aim × 8");
        // The second launch refuses (CheckUsability).
        assert!(!kite_launch(&mut app, held, world_pos), "the live kite refuses");
        // The windy-day achievement (`Player.cs:49458-49461`).
        let mut app2 = kite_app(4367);
        app2.wind.happy_windy_day = true;
        let held2 = app2.inventory[app2.selected];
        let wp2 = app2.camera + Vec2::new(80.0, -20.0);
        assert!(kite_launch(&mut app2, held2, wp2));
        assert!(
            app2.achievements.take_completed_events().contains(&"FLY_A_KITE_ON_A_WINDY_DAY"),
            "the windy-day kite achievement"
        );
        // The calm day fires nothing.
        let mut app3 = kite_app(4367);
        let held3 = app3.inventory[app3.selected];
        let wp3 = app3.camera + Vec2::new(80.0, -20.0);
        assert!(kite_launch(&mut app3, held3, wp3));
        assert!(app3.achievements.take_completed_events().is_empty());
    }

    /// The RMB channel arm (`Player.cs:43537-43541`): a live kite +
    /// holdStyle 1 + the right button arms the PARAMETERLESS channel
    /// (no cancel key); the release override reads `controlUseTile`.
    #[test]
    fn w4_d16_kite_the_rmb_channel_and_release() {
        let mut app = kite_app(4367);
        let held = app.inventory[app.selected];
        app.projectiles.spawn_with_data_ai_owned(766, 0, 500.0, 4000.0, 0.0, 0.0, -1, 0.0, 0.0, 0.0);
        assert!(kite_hold_out(&app, held));
        app.place_down = true;
        assert!(kite_channel_arm(&mut app, held, true), "the arm fires");
        let p = app.player.as_ref().unwrap();
        assert!(p.channel);
        assert_eq!(p.channel_item, 4367);
        assert_eq!(p.channel_proj_expected, 0, "the parameterless overload — no cancel key");
        assert_eq!(p.channel_index_expected, 0);
        // The LMB release does NOT drop the kite channel (flag5 reads
        // controlUseTile); the RMB release does.
        app.mine_down = false;
        assert!(!kite_channel_release(&app, held), "the LMB is not flag5 for kites");
        app.place_down = false;
        assert!(kite_channel_release(&app, held), "the RMB release drops it");
        // The buffered switch drops it too (`:42800-42803`).
        app.place_down = true;
        app.selection.buffered = 1;
        assert!(kite_channel_release(&app, held), "the buffered switch");
    }

    /// The dispatch wiring: one editing tick with the LMB press edge
    /// launches the kite and starts the swing; the held follow-up ticks
    /// spawn nothing (the refusal + the swing gate).
    #[test]
    fn w4_d16_kite_the_editing_tick_launches_once() {
        let mut app = kite_app(4367);
        app.mouse_pos = Vec2::new(640.0 + 80.0, 360.0);
        app.mine_down = true;
        app.mine_prev_down = false;
        app.mine_released = true;
        let screen = Vec2::new(1280.0, 720.0);
        super::super::update_editing(&mut app, 1.0 / 60.0, screen);
        let count = |a: &App| a.projectiles.slots().iter().filter(|p| p.active && p.kind == 766).count();
        assert_eq!(count(&app), 1, "the press launches one");
        assert!(app.swing > 0.0, "the useStyle-1 swing starts");
        for _ in 0..20 {
            super::super::update_editing(&mut app, 1.0 / 60.0, screen);
        }
        assert_eq!(count(&app), 1, "the held button never re-launches");
    }
}
