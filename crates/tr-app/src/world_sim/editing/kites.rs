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
//! held-shoot mismatch gate instead). W4-d16 residual 8 LANDED the afk
//! auto-kite: the `afkCounter`/`afkCounterForKiting` fields
//! (`Player.cs:991-992`), the standing-still counter (`:25497-25514`),
//! the `selectedKite` scan (`RefreshAutoKitingFromItemTypeAndSlot`
//! `:12843-12849`) and the `PickItemSelectionOverride` afk arm
//! (`:17421-17428` — the 3600-tick gate, the holdStyle-0 use-button
//! force). CODE-ONLY MODE: the witnesses are WRITTEN NOT RUN.

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

/// `Player.AFKTimeNeededForAutoKiting` (`Player.cs:994`, the
/// `Initialize`/static assignment `:57104`): 3600 ticks = one minute.
pub(crate) const AFK_TIME_NEEDED_FOR_AUTO_KITING: i32 = 3600;

/// `Player.IsConsideredStandingStill` (`Player.cs:3323-3333`): both
/// velocity components under 0.05 in magnitude.
pub(crate) fn is_considered_standing_still(vx: f32, vy: f32) -> bool {
    vx.abs() < 0.05 && vy.abs() < 0.05
}

/// `Player.Update`'s afk-counter tail (`Player.cs:25497-25514`, W4-d16
/// residual 8): the standing-still + idle-animation (or held-kite) +
/// no-control gate bumps BOTH counters, else both zero; the local
/// player's carried mouse item zeroes the kiting counter (`:25507-25510`),
/// and petting/sitting/sleeping zero it too (`:25511-25514` — petting has
/// no port field, the sitting/sleeping pair stands). `item_animation` is
/// the tick count (the `ticks_of` convention); `control_torch` the
/// LeftShift smart-select key. Runs at the `update_player` seam AFTER the
/// `:25218` selection update and BEFORE the movement block — the
/// standing-still read is the PREVIOUS tick's velocity, exactly the C#
/// order (the seam comment, `world_sim/player.rs`).
pub(crate) fn afk_counter_tick(
    app: &mut App,
    item_animation: i32,
    control_torch: bool,
) {
    let Some(p) = app.player.as_mut() else {
        return;
    };
    let held = app.inventory[app.selected];
    let held_is_kite = held.count > 0 && is_kite(held.kind);
    let still = is_considered_standing_still(p.vx, p.vy)
        && (item_animation == 0 || held_is_kite)
        && !p.control_up
        && !p.control_down
        && !p.control_left
        && !p.control_right
        && !p.control_jump
        && !control_torch;
    if still {
        p.afk_counter += 1;
        p.afk_counter_for_kiting += 1;
    } else {
        p.afk_counter = 0;
        p.afk_counter_for_kiting = 0;
    }
    // `:25507-25510` — the local player's carried mouse item.
    if app.mouse_item.count > 0 {
        p.afk_counter_for_kiting = 0;
    }
    // `:25511-25514` — petting (no port field) / sitting / sleeping.
    if p.sitting || p.sleeping {
        p.afk_counter_for_kiting = 0;
    }
}

/// `Player.UpdateEquips`' kite scan (`Player.cs:12898` + `:12900-12905`
/// → `RefreshAutoKitingFromItemTypeAndSlot` `:12843-12849`): the FIRST
/// kite slot wins — the held item first, then the 0..57 inventory walk
/// (the port's 50-grid; coins/ammo 50..57 are never kites). The C#
/// `ResetEffects` clears `selectedKite = -1` (`:18432`) ahead of the
/// scan; the port's reset lives in the `Player::update` ResetEffects
/// block, so this only pins. `selected_kite <= -1` keeps the first pin.
pub(crate) fn refresh_selected_kite(app: &mut App) {
    let Some(p) = app.player.as_mut() else {
        return;
    };
    let held = app.inventory[app.selected];
    if p.selected_kite <= -1 && held.count > 0 && is_kite(held.kind) {
        p.selected_kite = app.selected as i32;
    }
    for (j, slot) in app.inventory.iter().enumerate() {
        if p.selected_kite > -1 {
            break;
        }
        if slot.count > 0 && is_kite(slot.kind) {
            p.selected_kite = j as i32;
        }
    }
}

/// `PickItemSelectionOverride`'s afk auto-kite arm (`Player.cs:17421-17428`,
/// W4-d16 residual 8): past the 3600-tick gate with a pinned kite slot,
/// no `noItems` lock (the `isOperatingAnotherEntity`/`spectating` halves
/// have no port fields — the single-player defaults pass), the override
/// selects the kite slot and — while no kite is out (`holdStyle == 0`,
/// the pool-derived `kite_hold_out`) — forces `controlUseItem =
/// releaseUseItem = true` (the port's `mine_down` + the released-edge
/// pair, the `OnSelectionChanged` convention). Returns the override slot.
/// The C# runs the mouse-item arm (`:17412-17416`, slot 58 — unported)
/// and `SmartSelectLookup` (`:17417`) FIRST; the caller keeps that order.
pub(crate) fn afk_auto_kite_override(app: &mut App) -> Option<usize> {
    let gate = {
        let Some(p) = app.player.as_ref() else {
            return None;
        };
        p.afk_counter_for_kiting >= AFK_TIME_NEEDED_FOR_AUTO_KITING
            && p.selected_kite >= 0
            && !p.no_items
    };
    if !gate {
        return None;
    }
    let slot = app.player.as_ref().unwrap().selected_kite as usize;
    let held = app.inventory.get(slot).copied()?;
    // `:17424-17427` — the use-button force only while holdStyle 0.
    if !kite_hold_out(app, held) {
        app.mine_down = true;
        app.mine_released = true;
        app.mine_prev_down = false;
    }
    Some(slot)
}

/// W4-d16 residual 9 — the kite-anchor placement hook
/// (`TileObjectData.CallPostPlacementPlayerHook`,
/// `TileObjectData.cs:5269-5283` → `TEKiteAnchor.Hook_AfterPlacement`,
/// `TEKiteAnchor.cs:41-44` → `TELeashedEntityAnchorWithItem.
/// PlaceFromPlayerPlacementHook`, `TELeashedEntityAnchorWithItem.cs:49-67`
/// → `TELeashedEntityAnchor.PlaceFromPlayerPlacementHook`,
/// `TELeashedEntityAnchor.cs:19-28` → `TileEntity.Place`,
/// `DataStructures/TileEntity.cs:99-108`). The 723 row's
/// `hook_post_place_my_player` carries the callback name with
/// `hook: None` (the generated registry's documented-callback form,
/// `tileobject.rs` module doc); this is the runtime body the app's
/// `place_tile` seam calls after a successful `TileObject.Place`.
///
/// The single-player path (`Main.netMode != 1`): `TileEntity.Place`
/// creates the anchor entity at the cell (the port's
/// `TileEntities::place` — the `ValidTile` kind-723 check +
/// the position-free gate), then `InsertItem(num2)` stores the kite
/// item (`:64`, `TELeashedEntityAnchorWithItem.cs:32-36`). `item_type`
/// is the C# `num2 = heldItem.type` capture (`:53`) — the held kind
/// BEFORE the consumption, threaded from the `place_tile` seam (the
/// slot may be air by the time the hook runs). The hook's own stack
/// consumption (`:54-57`, `!consumable && --stack <= 0 → TurnToAir`)
/// is the C#'s ONLY decrement for the TileObject route
/// (`TileObject.Place` itself never touches the stack); the port's
/// `place_item_and_consume` generic decrement is the stand-in for it
/// (one drain per successful place, the same net effect), so this hook
/// does NOT consume again. The `RespawnLeashedEntity` half (`:35` →
/// `CreateLeashedEntity` → `LeashedKite.NewInstance().SetDefaults
/// (item.shoot)`) is M7-TE-2 — the entity + the stored item stand, the
/// leashed-kite visual unported. Returns whether the entity placed.
pub(crate) fn kite_anchor_place_hook(app: &mut App, tx: i32, ty: i32, item_type: u16) -> bool {
    if item_type == 0 || !is_kite(item_type) {
        return false;
    }
    // `TileEntity.Place` → `TEKiteAnchor.IsTileValidForEntity`
    // (`TEKiteAnchor.cs:21-29`: the cell active + kind 723) + the
    // position-free gate (`TileEntitiesManager.CheckValidTile`).
    // The `place` gate without the self-borrow: the duplicate position and the
    // tile-validity checks run first, then the unchecked insert (same gates as
    // `TileEntities::place`, which needs `&World` while borrowing itself).
    let te_kind = tr_core::tile_entities::TeKind::KiteAnchor;
    if app.world.tile_entities.get_at(tx as i16, ty as i16).is_some() {
        return false;
    }
    if !te_kind.is_tile_valid(&app.world, tx, ty) {
        return false;
    }
    let id = app.world.tile_entities.place_unchecked(te_kind, tx as i16, ty as i16);
    // `InsertItem(num2)` (`:64`): the anchor stores the kite item type
    // (the `TeData::Anchor` payload is the itemType int).
    if let Some(entity) = app.world.tile_entities.by_id_mut(id) {
        entity.data = tr_core::tile_entities::TeData::Anchor(item_type as i32);
    }
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

    /// Residual 8 — the afk-counter gate (`Player.cs:25497-25514`): the
    /// standing-still + idle + no-control tick bumps both counters; any
    /// control, motion or (non-kite) swing zeroes them; a held KITE keeps
    /// counting through its swing (`IsAKite[HeldItem]`); the carried
    /// mouse item and sitting/sleeping zero the kiting counter only.
    #[test]
    fn w4_d16_kite_the_afk_counter_gate() {
        let mut app = kite_app(4367);
        {
            let p = app.player.as_mut().unwrap();
            p.vx = 0.0;
            p.vy = 0.0;
        }
        afk_counter_tick(&mut app, 0, false);
        let p = app.player.as_ref().unwrap();
        assert_eq!((p.afk_counter, p.afk_counter_for_kiting), (1, 1));
        // The motion refusal (`IsConsideredStandingStill`, `:3327-3329`).
        app.player.as_mut().unwrap().vx = 0.05;
        afk_counter_tick(&mut app, 0, false);
        let p = app.player.as_ref().unwrap();
        assert_eq!((p.afk_counter, p.afk_counter_for_kiting), (0, 0), "0.05 is NOT < 0.05");
        app.player.as_mut().unwrap().vx = 0.0;
        // The control refusals (`:25497`'s !control chain).
        app.player.as_mut().unwrap().control_jump = true;
        afk_counter_tick(&mut app, 0, false);
        assert_eq!(app.player.as_ref().unwrap().afk_counter, 0);
        app.player.as_mut().unwrap().control_jump = false;
        afk_counter_tick(&mut app, 0, true);
        assert_eq!(app.player.as_ref().unwrap().afk_counter, 0, "controlTorch refuses");
        // The swing refusal — EXCEPT the held kite (`IsAKite[HeldItem]`).
        afk_counter_tick(&mut app, 5, false);
        assert_eq!(app.player.as_ref().unwrap().afk_counter, 1, "the held kite counts through the swing");
        app.inventory[app.selected] = ItemSlot { kind: 1, name: "", count: 1, prefix: 0 };
        afk_counter_tick(&mut app, 5, false);
        assert_eq!(app.player.as_ref().unwrap().afk_counter, 0, "the pickaxe swing refuses");
        app.inventory[app.selected] = ItemSlot { kind: 4367, name: "", count: 1, prefix: 0 };
        // The mouse-item zero (`:25507-25510`) — kiting only.
        afk_counter_tick(&mut app, 0, false);
        app.mouse_item = ItemSlot { kind: 1, name: "", count: 1, prefix: 0 };
        afk_counter_tick(&mut app, 0, false);
        let p = app.player.as_ref().unwrap();
        assert_eq!((p.afk_counter, p.afk_counter_for_kiting), (2, 0));
        app.mouse_item = crate::item::EMPTY_SLOT;
        // The sitting zero (`:25511-25514`).
        app.player.as_mut().unwrap().sitting = true;
        afk_counter_tick(&mut app, 0, false);
        let p = app.player.as_ref().unwrap();
        assert_eq!((p.afk_counter, p.afk_counter_for_kiting), (3, 0));
    }

    /// Residual 8 — the selectedKite scan (`RefreshAutoKitingFromItemTypeAndSlot`
    /// `Player.cs:12843-12849`): the held item wins first (`:12898`), then
    /// the lowest inventory slot (`:12905`); the −1 clear is ResetEffects'
    /// (`:18432`, the Player::update block).
    #[test]
    fn w4_d16_kite_the_selected_kite_scan() {
        let mut app = kite_app(4367);
        app.player.as_mut().unwrap().selected_kite = -1;
        refresh_selected_kite(&mut app);
        assert_eq!(app.player.as_ref().unwrap().selected_kite, app.selected as i32, "the held kite");
        // The held pickaxe: the scan finds the inventory kite.
        let mut app2 = kite_app(1);
        app2.inventory[7] = ItemSlot { kind: 4379, name: "", count: 1, prefix: 0 };
        app2.inventory[3] = ItemSlot { kind: 4367, name: "", count: 1, prefix: 0 };
        app2.player.as_mut().unwrap().selected_kite = -1;
        refresh_selected_kite(&mut app2);
        assert_eq!(app2.player.as_ref().unwrap().selected_kite, 3, "the FIRST kite slot");
        // No kite anywhere: stays −1.
        let mut app3 = kite_app(1);
        app3.player.as_mut().unwrap().selected_kite = -1;
        refresh_selected_kite(&mut app3);
        assert_eq!(app3.player.as_ref().unwrap().selected_kite, -1);
    }

    /// Residual 8 — the afk auto-kite override (`PickItemSelectionOverride`
    /// `Player.cs:17421-17428`): the 3600 gate + the pinned slot select;
    /// holdStyle 0 forces the use button (the press-edge pair), a live
    /// kite (holdStyle 1) keeps the button untouched; the noItems lock
    /// refuses.
    #[test]
    fn w4_d16_kite_the_afk_auto_kite_override() {
        let mut app = kite_app(4367);
        {
            let p = app.player.as_mut().unwrap();
            p.selected_kite = app.selected as i32;
            p.afk_counter_for_kiting = AFK_TIME_NEEDED_FOR_AUTO_KITING - 1;
        }
        assert_eq!(afk_auto_kite_override(&mut app), None, "one tick short");
        app.player.as_mut().unwrap().afk_counter_for_kiting = AFK_TIME_NEEDED_FOR_AUTO_KITING;
        app.mine_down = false;
        assert_eq!(afk_auto_kite_override(&mut app), Some(app.selected), "the gate fires");
        assert!(app.mine_down && app.mine_released && !app.mine_prev_down, "the use-button force");
        // A live kite (holdStyle 1): the override selects but never forces.
        app.projectiles.spawn_with_data_ai_owned(766, 0, 500.0, 4000.0, 0.0, 0.0, -1, 0.0, 0.0, 0.0);
        app.mine_down = false;
        app.mine_released = false;
        assert_eq!(afk_auto_kite_override(&mut app), Some(app.selected));
        assert!(!app.mine_down, "holdStyle 1 keeps the button");
        // The noItems refusal.
        app.player.as_mut().unwrap().no_items = true;
        assert_eq!(afk_auto_kite_override(&mut app), None);
        // The unpinned slot refuses.
        app.player.as_mut().unwrap().no_items = false;
        app.player.as_mut().unwrap().selected_kite = -1;
        assert_eq!(afk_auto_kite_override(&mut app), None);
    }

    /// Residual 9 — the 723 anchor row census (`TileObjectData.cs:4488-4517`):
    /// the 1x1 Style1x1 base, the bottom solid anchor + the four alternates
    /// (top/left/right/wall), the `TEKiteAnchor.Hook_AfterPlacement`
    /// post-place callback, frame_important. The generated registry row
    /// (`tileobject/registry.rs` D_723) matches.
    #[test]
    fn w4_d16_kite_the_anchor_tile_row() {
        let def = tr_core::tileobject::def_for_tile(723).expect("the 723 row");
        assert_eq!((def.width, def.height), (1, 1));
        assert_eq!(def.origin, tr_core::tileobject::Point16::new(0, 0));
        assert!(def.uses_custom_can_place, "the TileObject route");
        assert!(def.frame_important);
        assert_eq!(def.alternates.len(), 4, "the top/left/right/wall alternates");
        assert_eq!(
            def.hook_post_place_my_player.callback,
            "TEKiteAnchor.Hook_AfterPlacement"
        );
        assert_eq!(def.hook_post_place_my_player.bad_return, -1);
        assert!(def.hook_post_place_my_player.processed_coordinates);
        // The bottom anchor (43 = SolidTile|SolidWithTop|SolidSide|AlternateTile).
        assert_eq!(def.anchor_bottom.ty, 43);
        // The kite items all place 723 (`Item.DefaultToKite`, `Item.cs:47837`).
        for &item in KITE_ITEMS.iter() {
            assert_eq!(registry::get(item).unwrap().tile, Some(723), "item {item}");
        }
    }

    /// Residual 9 — the anchor place hook (`TELeashedEntityAnchorWithItem.
    /// PlaceFromPlayerPlacementHook` → `TileEntity.Place` + `InsertItem`):
    /// a placed 723 cell gains a KiteAnchor entity storing the held kite
    /// item; the position-free gate refuses a second entity on the cell.
    #[test]
    fn w4_d16_kite_the_anchor_place_hook() {
        let mut app = kite_app(4367);
        // A 723 anchor cell at (10, 10) — active, kind 723.
        let i = app.world.index(10, 10);
        app.world.tiles[i] = tr_core::tile::Tile { kind: 723, ..Default::default() };
        app.world.tiles[i].flags |= tr_core::tile::FLAG_ACTIVE;
        assert!(kite_anchor_place_hook(&mut app, 10, 10, 4367), "the hook places");
        let te = app.world.tile_entities.get_at(10, 10).expect("the entity");
        assert_eq!(te.kind, tr_core::tile_entities::TeKind::KiteAnchor);
        assert_eq!(te.data, tr_core::tile_entities::TeData::Anchor(4367), "the stored kite");
        // The position-free gate refuses a duplicate.
        assert!(!kite_anchor_place_hook(&mut app, 10, 10, 4367), "the cell is taken");
        // A non-kite item refuses.
        assert!(!kite_anchor_place_hook(&mut app, 11, 10, 1));
        // The ValidTile gate: a non-723 cell refuses.
        let j = app.world.index(12, 10);
        app.world.tiles[j] = tr_core::tile::Tile { kind: 1, ..Default::default() };
        app.world.tiles[j].flags |= tr_core::tile::FLAG_ACTIVE;
        assert!(!kite_anchor_place_hook(&mut app, 12, 10, 4367), "the kind gate");
    }

    /// Residual 9 — the anchor kill chain (`WorldGen.cs:63691-63694` +
    /// `:63873-63876`): breaking the 723 cell removes the entity and drops
    /// the stored kite item through `object_drops`; an empty anchor
    /// (itemType 0) drops nothing; the type mismatch leaves a foreign
    /// entity untouched.
    #[test]
    fn w4_d16_kite_the_anchor_kill_chain() {
        let mut w = World::new("t".into(), 40, 40);
        let i = w.index(10, 10);
        w.tiles[i] = tr_core::tile::Tile { kind: 723, ..Default::default() };
        w.tiles[i].flags |= tr_core::tile::FLAG_ACTIVE;
        let id = w.tile_entities.place_unchecked(tr_core::tile_entities::TeKind::KiteAnchor, 10, 10);
        w.tile_entities.by_id_mut(id).unwrap().data =
            tr_core::tile_entities::TeData::Anchor(4367);
        let mut rng = tr_core::UnifiedRandom::new(0);
        let kill = tr_core::edit::kill_tile(&mut w, 10, 10, false, &mut rng).expect("the break");
        assert!(w.tile_entities.get_at(10, 10).is_none(), "the entity removed");
        assert_eq!(kill.object_drops.len(), 1, "the stored-item drop");
        assert_eq!(kill.object_drops[0].item, Some(4367));
        assert_eq!((kill.object_drops[0].x, kill.object_drops[0].y), (10, 10));
        // The empty anchor drops nothing but still removes the entity.
        let j = w.index(11, 10);
        w.tiles[j] = tr_core::tile::Tile { kind: 723, ..Default::default() };
        w.tiles[j].flags |= tr_core::tile::FLAG_ACTIVE;
        w.tile_entities.place_unchecked(tr_core::tile_entities::TeKind::KiteAnchor, 11, 10);
        let kill2 = tr_core::edit::kill_tile(&mut w, 11, 10, false, &mut rng).expect("the break");
        assert!(w.tile_entities.get_at(11, 10).is_none());
        assert!(kill2.object_drops.is_empty(), "itemType 0 drops nothing");
        // A plain 723 cell with no entity breaks cleanly (no drop).
        let k = w.index(12, 10);
        w.tiles[k] = tr_core::tile::Tile { kind: 723, ..Default::default() };
        w.tiles[k].flags |= tr_core::tile::FLAG_ACTIVE;
        let kill3 = tr_core::edit::kill_tile(&mut w, 12, 10, false, &mut rng).expect("the break");
        assert!(kill3.object_drops.is_empty());
    }

    /// Residual 8 — the fully-reeled kite kill zeroes both afk counters
    /// (`Projectile.cs:48597-48598`): the auto-kite re-arms only after
    /// another afk minute.
    #[test]
    fn w4_d16_kite_the_reeled_kill_zeroes_the_afk_counters() {
        let w = World::new("t".into(), 300, 200);
        let mut pl = crate::player::Player::spawn(&w);
        pl.afk_counter = 500;
        pl.afk_counter_for_kiting = AFK_TIME_NEEDED_FOR_AUTO_KITING;
        let mut p = crate::projectile::Projectile::default();
        p.active = true;
        p.kind = 766;
        p.owner = 0;
        p.w = 4.0;
        p.h = 4.0;
        p.tile_collide = true;
        p.ai[0] = 4.0; // the fully-reeled string
        p.local_ai[2] = 4.0;
        let (pcx, pcy) = pl.center();
        // Beyond the 4-px string: the reel snap fires, num10 == num kills.
        p.pos = Vec2::new(pcx - 2.0, pcy - 100.0 - 2.0);
        let ctx = crate::projectile_ai_kites::KiteContext {
            selected_shoot: 766, item_animation: 0, control_use_item: false,
        };
        struct NoRng;
        impl tr_core::loot::DropRng for NoRng {
            fn next_range(&mut self, min: i32, _max: i32) -> i32 { min }
            fn next_float(&mut self) -> f32 { 0.0 }
        }
        crate::projectile_ai_kites::kite_step(
            &mut p, &w, Some(&mut pl), &ctx, (0.0, 0.0, 0.0, 0.0),
            &mut NoRng, 0, 3, 3,
        );
        assert!(!p.active, "the fully-reeled kill");
        assert_eq!((pl.afk_counter, pl.afk_counter_for_kiting), (0, 0), "the afk resets");
    }
}
