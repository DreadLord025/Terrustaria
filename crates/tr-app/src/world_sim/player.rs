//! Player simulation (plan stage A4): the deterministic input assembly and the
//! fixed 60 Hz `Player::update` stepping with the render interpolation the
//! camera follows.

use glam::Vec2;
use winit::keyboard::KeyCode;

use crate::app::App;
use crate::inv::{ARMOR, MISC};
use crate::item::ItemSlot;
use crate::player::{Player, PlayerInput};

/// The ammo-mastery grant seam (M3-ITEM-6, Phase 3) — the port's home for the
/// C#'s `Player.ResetEffects` clear + `UpdateBuffs`/`UpdateEquips` re-derive of
/// the ten ammo flags and the cycling mode.
///
/// The clear (`ResetEffects`, `Player.cs:18390-18393`, `:18450-18451`, `:18479`,
/// `:18605`, `:18664`) and the two grant families run in the C#'s own order:
/// `UpdateBuffs` (`:25566`) before `UpdateEquips` (`:25633`). The seam lives in
/// `world_sim` rather than inside `Player::update` because it reads the App's
/// equipment arrays (`App.equip`), which the fixed-step tick does not see — the
/// vanilla hosts it inside `Player.Update` over `this.armor`/`this.miscEquips`.
///
/// Scan scopes, transcribed from `UpdateEquips` (`Player.cs:12892-12960`):
/// - the buff flags (16/93/112) from the buff slots, like the C#'s buff loop
///   (`:10018`, `:10080`, `:10195`);
/// - the ammo-cost armor-set flags from `GetEffectiveArmor(0..10)` — the port's
///   `equip.armor[0..10]` (3 armor + 7 accessories, `ui/inventory.rs:395-421`);
/// - the quiver / Sharp Barb / cycler accessories from `ApplyEquipFunctional`'s
///   accessory slots 3..10 (`:12946-12951`) — extended with the port's five
///   `misc_equips` rows, the functional-accessory host the C#'s `miscEquips`
///   (pet/light/cart/mount/hook) does not carry. The superset is a documented
///   deviation: a quiver in `miscEquips[0]` grants nothing in the vanilla,
///   because the port has no other home for an 8th+ accessory.
pub(crate) fn grant_ammo_mastery(
    p: &mut Player,
    armor: &[ItemSlot; ARMOR],
    misc: &[ItemSlot; MISC],
) {
    // The clear: every flag false, every count zero, the mode None — the grants
    // below re-set them from this tick's equipment (the house `ResetEffects`
    // pattern, the `pinned_*` grant's shape in `player.rs:1717-1785`).
    p.magic_quiver = false;
    p.has_molten_quiver = false;
    p.acc_sharp_barb = false;
    p.ammo_box = false;
    p.ammo_potion = false;
    p.huntress_ammo_cost90 = false;
    p.chloro_ammo_cost80 = false;
    p.ammo_cost80 = false;
    p.ammo_cost75 = false;
    p.archery = false;
    p.ammo_cycling.mode = crate::ammo::AmmoCyclingMode::None;
    // `UpdateBuffs` first (`Player.cs:25566`, before `UpdateEquips` `:25633`):
    // the three buff flags the C# grants inside the buff loop's `else if` chain.
    for bt in p.buffs.active_types() {
        match bt {
            // `:10018` — the Archery Potion. Its `arrowDamage *= 1.1f` half has
            // no port consumer yet (M3-EQUIP-3), so only the speed flag lands.
            16 => p.archery = true,
            // `:10080` — the Ammo Box buff.
            93 => p.ammo_box = true,
            // `:10195` — the Ammo Reservation buff.
            112 => p.ammo_potion = true,
            _ => {}
        }
    }
    // `UpdateEquips` → `GrantArmorBenefits` over the ten functional armor slots
    // (`GetEffectiveArmor(0..10)`, `Player.cs:12919-12940`): the armor-set
    // ammo-cost flags. The stat halves (rangedDamage/crit adds) are
    // M3-EQUIP-3's and stay out of this layer.
    for slot in &armor[..10] {
        match slot.kind {
            // `:13179`, `:13634` — the Huntress's Jerkin / the Crystal Ninja set.
            3804 | 4983 => p.huntress_ammo_cost90 = true,
            // `:13649` — the Chlorophyte Helmet.
            1002 => p.chloro_ammo_cost80 = true,
            // `:13715`, `:13222` — the Shroomite Breastplate / the Huntress set.
            1549 | 3878 => p.ammo_cost80 = true,
            // `:13908` — the Vortex Breastplate.
            2758 => p.ammo_cost75 = true,
            _ => {}
        }
    }
    // `UpdateEquips` → `ApplyEquipFunctional` over the accessory slots 3..10
    // (`:12946-12951`), plus the port's `misc_equips` host (see the doc): the
    // quivers, the Sharp Barb and the four cycler accessories.
    for slot in armor[3..10].iter().chain(misc.iter()) {
        match slot.kind {
            // `:14465`, `:14483`, `:14596`, `:15333` — the Magic Quiver family.
            // 4002 Molten / 6181 Phoenix also convert the wooden arrow
            // (`hasMoltenQuiver`, `:14467`, `:14596`); 4006 Stalker's and 1321
            // the plain quiver only grant the speed/roll flag.
            4002 | 6181 => {
                p.magic_quiver = true;
                p.has_molten_quiver = true;
            }
            4006 | 1321 => p.magic_quiver = true,
            // `:14501` — the Sharp Barb.
            6160 => p.acc_sharp_barb = true,
            // `:14528-14544` — the cycler accessories grant the mode (a later
            // matching slot wins, like the C#'s `switch` in slot order).
            6168 => p.ammo_cycling.mode = crate::ammo::AmmoCyclingMode::AmmoSlots,
            6169 => p.ammo_cycling.mode = crate::ammo::AmmoCyclingMode::FullInventory,
            6193 => p.ammo_cycling.mode = crate::ammo::AmmoCyclingMode::Random,
            // `:14541` — the "no cycling" accessory resets the mode.
            6194 => p.ammo_cycling.mode = crate::ammo::AmmoCyclingMode::None,
            _ => {}
        }
    }
}

pub(crate) fn update_player(app: &mut App, dt: f32) {
    // Deterministic held input for screenshots (`TR_HOLD=right,jump`).
    static HOLD: std::sync::OnceLock<(bool, bool)> = std::sync::OnceLock::new();
    let (hold_right, hold_jump) = *HOLD.get_or_init(|| {
        let v = std::env::var("TR_HOLD")
            .unwrap_or_default()
            .to_ascii_lowercase();
        (v.contains("right"), v.contains("jump"))
    });
    // `Player.cs:24772-24803`: while the map is fullscreen every movement control
    // is zeroed, so the player stands still and the WASD keys pan the map.
    let map_open = app.map.as_ref().is_some_and(|m| m.open);
    let input = if map_open {
        PlayerInput {
            left: false,
            right: false,
            jump: false,
            down: false,
            hook: false,
        }
    } else {
        PlayerInput {
            left: app.keys.contains(&KeyCode::KeyA) || app.keys.contains(&KeyCode::ArrowLeft),
            right: hold_right
                || app.keys.contains(&KeyCode::KeyD)
                || app.keys.contains(&KeyCode::ArrowRight),
            jump: hold_jump
                || app.keys.contains(&KeyCode::Space)
                || app.keys.contains(&KeyCode::KeyW)
                || app.keys.contains(&KeyCode::ArrowUp),
            down: app.keys.contains(&KeyCode::KeyS) || app.keys.contains(&KeyCode::ArrowDown),
            // `Player.controlHook` (the quick-grapple key).
            hook: app.keys.contains(&KeyCode::KeyG),
        }
    };
    let (sw, sh) = app.screen_size();
    let mouse_world = Vec2::new(
        app.camera.x + (app.mouse_pos.x - sw * 0.5) / app.zoom,
        app.camera.y + (app.mouse_pos.y - sh * 0.5) / app.zoom,
    );
    let hook_type = *crate::grapple::HOOK_TYPE.get_or_init(|| {
        std::env::var("TR_HOOK")
            .ok()
            .and_then(|v| v.parse::<i32>().ok())
            .unwrap_or(13)
    });
    // `TR_CART` pins the minecart mount (the granting item, QuickMount and the
    // TrackHook paths are M2-MOUNT-3) — mounted idempotently like the
    // `TR_HOOK` parse above, so a dismount re-mounts on the next tick.
    static CART_PIN: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    let cart_pin = *CART_PIN.get_or_init(|| std::env::var("TR_CART").is_ok());
    if let Some(p) = app.player.as_mut() {
        // The `Player.SceneMetrics` snapshot (`Main.SceneMetrics` is the
        // global the source reads): the frame's `update_scene` scan feeds
        // the per-tick aura copy.
        p.auras = app.scene_metrics.auras();
        // The zone flags (`Player.ZoneSnow`-family) — the torch-luck table's
        // inputs (`Player.cs:17924-18036`), refreshed beside the auras.
        p.zones = app.scene_metrics.zones();
        // The held torch's style (`inventory[selectedItem].createTile == 4`,
        // `Player.cs:17917-17920`).
        p.held_torch_style = app
            .inventory
            .get(app.selected)
            .filter(|slot| slot.count > 0)
            .and_then(|slot| crate::registry::get(slot.kind))
            .filter(|def| {
                def.tile == Some(4) && (def.place_style as usize) < crate::luck::TORCH_ID_COUNT
            })
            .map(|def| def.place_style as u8);
        // `Player.statDefense` (`Player.UpdateEquips`): the equipped
        // pieces' defense plus the set bonus (M3-EQUIP-2).
        let armor_kinds: Vec<i32> = app
            .equip
            .armor
            .iter()
            .map(|slot| slot.kind as i32)
            .collect();
        let piece_def: i32 = app
            .equip
            .armor
            .iter()
            .filter(|slot| slot.kind != 0)
            .filter_map(|slot| crate::registry::get(slot.kind))
            .map(|d| d.defense as i32)
            .sum();
        let set = tr_core::armor_sets::set_bonus_defense(
            armor_kinds.first().copied().unwrap_or(0),
            armor_kinds.get(1).copied().unwrap_or(0),
            armor_kinds.get(2).copied().unwrap_or(0),
        );
        p.stat_defense = piece_def + set;
        // `Player.Update` -> `UpdateDead` (`Player.cs:16919`): the dead player
        // runs the respawn-timer tick instead of the normal simulation. The
        // boss check reads the live NPC list; the danger-NPC gate has no
        // `DangerThatPreventsOtherDangers` members in the ported set yet.
        if p.dead {
            let boss_near = p.boss_hinders(&app.npcs);
            let mut respawn_now = false;
            app.player_step.tick(dt, |_| {
                respawn_now = p.update_dead(input.jump, boss_near, false) || respawn_now;
            });
            if respawn_now {
                // `Player.Spawn`'s bed re-validation (`Player.cs:37571-37575`):
                // a broken bed or an invalid room reverts the spawn to the
                // world spawn (`SpawnX = -1` — the port clears `bed_spawn` and
                // rewrites the spawn fields); `CheckSpawn`'s failure text
                // rides the chat like the C#'s `Main.NewText`.
                if let Some((bx, by)) = app.world.bed_spawn {
                    if !crate::world_sim::editing::check_bed_spawn(
                        &app.world,
                        bx,
                        by,
                        &mut app.chat,
                    ) {
                        app.world.bed_spawn = None;
                        p.spawn_x = app.world.spawn_x as f32 * tr_core::TILE_PX + 8.0
                            - crate::player::WIDTH as f32 * 0.5;
                        p.spawn_y = app.world.spawn_y as f32 * tr_core::TILE_PX
                            - crate::player::HEIGHT as f32;
                    }
                }
                p.revive();
                app.npcs.clear();
                app.npc_timer = 4.0;
                if app.follow_player {
                    let (cx, cy) = p.center();
                    app.camera = Vec2::new(cx, cy);
                }
            }
            if app.follow_player {
                let (cx, cy) = p.center();
                app.camera = app.camera.lerp(Vec2::new(cx, cy), 1.0 - (-dt * 12.0).exp());
            }
            return;
        }
        // The quick-grapple fire: the `controlHook` press edge fires
        // `QuickGrapple` (`Player.cs:27480-27492` + `:6296-6478`) — one hook
        // out at a time (the vanilla `num7 = 1` recycle path for the
        // single-hook items), aimed at the mouse world position.
        static HOOK_PREV: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
        let hook_edge =
            input.hook && !HOOK_PREV.swap(input.hook, std::sync::atomic::Ordering::Relaxed);
        // `Player.QuickGrapple` (`Player.cs:6334-6337`): a mount not in the
        // CanUseHooks set dismounts before the hook fires — every row in this
        // table (the set is {54, 57-60}).
        if hook_edge && !p.dead && p.mount.is_active() {
            p.try_dismount(&app.world);
        }
        if hook_edge && !p.dead {
            let center = Vec2::new(
                p.x + crate::player::WIDTH * 0.5,
                p.y + crate::player::HEIGHT * 0.5,
            );
            app.grapple = Some(crate::grapple::GrappleHook::fire(
                hook_type,
                center,
                mouse_world,
                16.0,
            ));
        }
        // The quick-mount key: the `controlMount` press edge (the menu's
        // "Mount R" row, `Main.cMount` default "R") fires `QuickMount`
        // (`Player.cs:24888-24893` -> `:6045-6100`).
        static MOUNT_PREV: std::sync::atomic::AtomicBool =
            std::sync::atomic::AtomicBool::new(false);
        let mount_key = app.keys.contains(&KeyCode::KeyR);
        let mount_edge =
            mount_key && !MOUNT_PREV.swap(mount_key, std::sync::atomic::Ordering::Relaxed);
        if mount_edge && !map_open {
            if let Some(kind) = p.quick_mount(&app.world, &app.inventory) {
                // `ItemCheck_TryStartUse`'s animation (the saddle's
                // `useAnimation` — 20 ticks on every row).
                let secs = crate::item::use_animation_secs(kind);
                app.swing_max = app.swing_max.max(secs);
                app.swing = secs;
                app.attack_cd = app.attack_cd.max(secs);
            }
        }
        // The hook AI tick (`AI_007_GrapplingHooks`): the flight, the tile
        // latch, the retract; the attached hook feeds the pull.
        let grapple_snapshot = app.grapple.as_mut().and_then(|h| {
            let center = Vec2::new(
                p.x + crate::player::WIDTH * 0.5,
                p.y + crate::player::HEIGHT * 0.5,
            );
            if h.update(&app.world, center) {
                Some((h.pos, h.hook_type, h.ai == crate::grapple::HookAi::Attached))
            } else {
                None
            }
        });
        p.grappling_active = grapple_snapshot
            .as_ref()
            .is_some_and(|(_, _, attached)| *attached);
        if let Some((pos, hook_type, _)) = grapple_snapshot {
            p.grapple_hook_center = pos;
            p.grapple_hook_type = hook_type;
        } else {
            app.grapple = None;
        }
        let grapple_for_tick = grapple_snapshot
            .filter(|(_, _, attached)| *attached)
            .map(|(pos, hook_type, _)| (pos, hook_type));
        // `Player.armor`'s kinds snapshot: the update's grant pass reads the
        // equipped kinds (`UpdateEquips`, `Player.cs:12892` — the loop runs
        // inside `Player.Update` on the vanilla's own array) plus the rolled
        // prefixes for `GrantPrefixBenefits`.
        for i in 0..20 {
            p.equipped[i] = app.equip.armor[i].kind;
            p.equipped_prefix[i] = app.equip.armor[i].prefix;
        }
        // `IsVoidVaultEnabled`'s per-tick refresh (`Player.cs:24550-24553`
        // — `useVoidBag`'s scan :23171-23180; the port scans the grid plus
        // the coin/ammo rows the App holds).
        {
            let holds_lens = app
                .inventory
                .iter()
                .chain(app.equip.coins.iter())
                .chain(app.equip.ammo.iter())
                .any(|sl| sl.count > 0 && sl.kind == 4131);
            p.void_vault_enabled = holds_lens;
        }
        // `Player.dye`/`miscDyes`' kinds snapshot — `UpdateDyes`' input
        // through `GetEffectiveDye` (`Player.cs:5742-5764`; no loadouts in
        // the port, the slot itself is the source).
        for i in 0..10 {
            p.dye_slots[i] = app.equip.dye[i].kind;
        }
        for i in 0..5 {
            p.misc_dye_slots[i] = app.equip.misc_dyes[i].kind;
        }
        p.swing_aim = app.swing_aim;
        let step = 1.0 / 60.0;
        if cart_pin && !p.cart_mounted {
            p.mount_cart();
        }
        app.player_step.tick(dt, |_| {
            // `player.itemAnimation > 0` for the sleeping state — the same
            // gate the swing render uses (`self.swing > 0.0` with a held
            // item, `frame.rs:483`). The danger gate rides the M2-DEATH-2
            // stand-in (no `DangerThatPreventsOtherDangers` members yet).
            let item_in_use = app.swing > 0.0 && app.inventory[app.selected].count > 0;
            p.update_with_grapple(&app.world, input, grapple_for_tick, item_in_use, false);
            // DW-3.3/DW-3.4 — the ammo-mastery half of `Player.Update`
            // (`Player.UpdateBuffs` → `UpdateEquips` → … → `Update_AmmoCycler` →
            // `ItemCheck`, `:25566`/`:25633`/`:24566`/`:31827`), which the port
            // splits into its frame order `update_player` → `update_editing`
            // (`frame.rs:285`/`:481`): the flags clear and re-derive from this
            // tick's equipment and buffs, then the cycler's cooldown decrements
            // — both inside the fixed step, BEFORE the shoot path reads them.
            grant_ammo_mastery(p, &app.equip.armor, &app.equip.misc_equips);
            p.ammo_cycling.tick();
            // `Player.cs:28237-28241` + `Minecart.HitTrackSwitch`
            // (`Minecart.cs:1281-1288`): the pressure-track trip fires
            // `Wiring.HitSwitch` at the magnet tile (net message 59 is out of
            // scope — local only).
            if let Some((i, j)) = p.take_switch_trip() {
                app.wiring.hit_switch(&mut app.world, i, j);
            }
            // The wheel-spark drain: the tick queued fully-rolled dust-213
            // particles (`Minecart.WheelSparks` → `DelegateMethods.Minecart.Sparks`)
            // and the C# sound sites' cues; the pool is the App's `Main.dust`
            // stand-in and the cues go straight out.
            let sparks = p.take_cart_sparks();
            app.minecart_sparks.spawn(sparks);
            for cue in p.take_cart_sounds() {
                app.audio.play_at(cue.cue, cue.x, cue.y, 1.0);
            }
        });
        app.player_step.drop_backlog();
        // The jump-off consumed the hook (`RemoveAllGrapplingHooks`).
        if !p.grappling_active {
            app.grapple = None;
        }
        if app.follow_player {
            // Follow the interpolated position so the camera is smooth between
            // the 60 Hz simulation steps.
            let alpha = (app.player_step.residual() / step).clamp(0.0, 1.0);
            let render = p.interpolated(alpha);
            let (cx, cy) = render.center();
            app.camera = app.camera.lerp(Vec2::new(cx, cy), 1.0 - (-dt * 12.0).exp());
        }
    }
    crate::chest::tick_open(app); // `Player.HandleBeingInChestRange` (`Player.cs:25290`/:31015-31124)
    if map_open {
        app.update_map_input();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::flow::front_flow_tests::flow_app;
    use crate::inv::EquipSlots;
    use crate::item::EMPTY_SLOT;
    use crate::player::Player;
    use tr_core::tile::FLAG_ACTIVE;
    use tr_core::wiring::WireColor;
    use tr_core::{Tile, World};

    /// The ammo-mastery test rig: a plain standing player in an empty world,
    /// with one tick of `update_player`'s fixed step available.
    fn ammo_app() -> App {
        let mut app = flow_app(None, vec![], vec![]);
        app.world = World::new("ammo".into(), 64, 48);
        app.player = Some(Player::spawn(&app.world));
        app
    }

    /// One tick of the pass (the stepper runs its closure once per 1/60 s).
    fn tick_ammo(app: &mut App) {
        update_player(app, 1.0 / 60.0);
    }

    /// An equipment row carrying `kind`.
    fn slot(kind: u16) -> ItemSlot {
        ItemSlot {
            kind,
            name: crate::registry::name(kind),
            count: 1,
            prefix: 0,
        }
    }

    /// DW-3.3 — the grant seam (`ResetEffects`' clear + `UpdateBuffs`/
    /// `UpdateEquips`' re-derive, `Player.cs:18390-18664`, `:25566`, `:25633`):
    /// a Molten Quiver in a misc-equip slot grants `magicQuiver` +
    /// `hasMoltenQuiver` for that tick, and the next tick's clear wipes both
    /// once the accessory is gone. The buff flags and the armor-set flags ride
    /// the same seam, in the C#'s order.
    #[test]
    fn test_DW_3_3_the_grant_seam_derives_the_flags_from_the_equipment_and_clears_them() {
        let mut app = ammo_app();
        // `Player.cs:14465` — the Molten Quiver (4002) is both a magic and a
        // molten quiver (`:14467`); 4006 Stalker's is only the speed/roll half.
        app.equip.misc_equips[0] = slot(4002);
        tick_ammo(&mut app);
        {
            let p = app.player.as_ref().unwrap();
            assert!(p.magic_quiver, "4002 grants magicQuiver");
            assert!(p.has_molten_quiver, "4002 grants hasMoltenQuiver");
            assert!(!p.archery && !p.ammo_box && !p.ammo_potion, "no buffs yet");
            assert!(!p.ammo_cost80 && !p.ammo_cost75, "no armor set yet");
            // The flag struct the shooter wire reads carries both.
            let flags = p.ammo_flags();
            assert!(flags.magic_quiver && flags.has_molten_quiver);
            assert!(!flags.ammo_box, "the roll flags are still all false");
        }
        // Unequip: the per-tick clear leaves nothing behind (the trap the
        // `ResetEffects` pattern exists for — a latch, not a one-way grant).
        app.equip.misc_equips[0] = EMPTY_SLOT;
        tick_ammo(&mut app);
        {
            let p = app.player.as_ref().unwrap();
            assert!(
                !p.magic_quiver && !p.has_molten_quiver,
                "the seam clears per tick"
            );
        }
        // 4006 grants the roll/speed half only, 6181 both — like the vanilla.
        app.equip.misc_equips[1] = slot(4006);
        tick_ammo(&mut app);
        {
            let p = app.player.as_ref().unwrap();
            assert!(p.magic_quiver, "4006 Stalker's Quiver");
            assert!(!p.has_molten_quiver, "4006 is not a molten quiver");
        }
        app.equip.misc_equips[1] = EMPTY_SLOT;
        app.equip.misc_equips[0] = slot(6181);
        tick_ammo(&mut app);
        {
            let p = app.player.as_ref().unwrap();
            assert!(p.magic_quiver && p.has_molten_quiver, "6181 Phoenix Quiver");
        }
    }

    /// DW-3.3 (the buff half) — the three buff-granted flags (16 Archery,
    /// 93 Ammo Box, 112 Ammo Reservation; `Player.cs:10018`, `:10080`,
    /// `:10195`) derive from the buff slots, and the clear drops them when the
    /// buff runs out — the scan runs inside the per-tick grant seam, before the
    /// equipment scan, like the C#'s `UpdateBuffs` → `UpdateEquips`.
    #[test]
    fn test_DW_3_3_the_buff_scan_grants_and_the_clear_drops_the_three_buff_flags() {
        let mut app = ammo_app();
        {
            let p = app.player.as_mut().unwrap();
            p.buffs.add_buff(93, 60); // the Ammo Box
            p.buffs.add_buff(112, 60); // the Ammo Reservation
            p.buffs.add_buff(16, 60); // the Archery Potion
        }
        tick_ammo(&mut app);
        {
            let p = app.player.as_ref().unwrap();
            assert!(p.ammo_box, "buff 93 → the 1/5 roll arm");
            assert!(p.ammo_potion, "buff 112 → the 1/5 roll arm");
            assert!(p.archery, "buff 16 → the ×1.2 speed fuse");
            // An unrelated buff grants nothing.
            assert!(!p.huntress_ammo_cost90 && !p.ammo_cost80 && !p.chloro_ammo_cost80);
        }
        // The buffs expire: the clear leaves the flags false again.
        {
            let p = app.player.as_mut().unwrap();
            p.buffs.clear_buff(93);
            p.buffs.clear_buff(112);
            p.buffs.clear_buff(16);
        }
        tick_ammo(&mut app);
        {
            let p = app.player.as_ref().unwrap();
            assert!(
                !p.ammo_box && !p.ammo_potion && !p.archery,
                "the buffs are gone"
            );
        }
        // The buff grant and the equipment grant are independent: a buff AND a
        // quiver both land on the same tick.
        app.equip.misc_equips[0] = slot(4002);
        {
            let p = app.player.as_mut().unwrap();
            p.buffs.add_buff(93, 60);
        }
        tick_ammo(&mut app);
        {
            let p = app.player.as_ref().unwrap();
            assert!(p.ammo_box && p.magic_quiver, "both families on one tick");
        }
    }

    /// DW-3.3 (the armor half) — the armor-set ammo-cost flags grant from the
    /// ten functional armor slots (`GrantArmorBenefits` over
    /// `GetEffectiveArmor(0..10)`, `Player.cs:12919-12940`): 3804/4983 →
    /// `huntressAmmoCost90`, 1002 → `chloroAmmoCost80`, 1549/3878 →
    /// `ammoCost80`, 2758 → `ammoCost75` (`:13179`, `:13634`, `:13649`,
    /// `:13715`, `:13222`, `:13908`). The accessory slots carry the quiver
    /// family and the cycler accessories (`ApplyEquipFunctional` `:12946`).
    #[test]
    fn test_DW_3_3_the_armor_set_and_the_cycler_accessories_grant_their_flags() {
        let mut app = ammo_app();
        // The armor trio: a Chlorophyte Helmet (1002), a Shroomite Breastplate
        // (1549) and a Vortex Greaves-free leg slot.
        app.equip.armor[0] = slot(1002);
        app.equip.armor[1] = slot(1549);
        // The accessory row: the Huntress's Jerkin (3804) — an ammo-cost set
        // piece that in vanilla sits in a body slot, and its grant is not
        // slot-shape-gated, so an accessory slot grants it too.
        app.equip.armor[3] = slot(3804);
        // The four cycler accessories in the port's misc-equip host
        // (`Player.cs:14528-14544`): the LAST matching slot wins, like the C#'s
        // switch inside the slot loop.
        app.equip.misc_equips[0] = slot(6168);
        app.equip.misc_equips[1] = slot(6169);
        tick_ammo(&mut app);
        {
            let p = app.player.as_ref().unwrap();
            assert!(p.chloro_ammo_cost80, "1002 → chloroAmmoCost80");
            assert!(p.ammo_cost80, "1549 → ammoCost80");
            assert!(p.huntress_ammo_cost90, "3804 → huntressAmmoCost90");
            assert_eq!(
                p.ammo_cycling.mode,
                crate::ammo::AmmoCyclingMode::FullInventory,
                "6169 (the later slot) wins over 6168"
            );
        }
        // Unequipping the whole set clears every flag and the mode.
        let empty: EquipSlots = EquipSlots::empty();
        app.equip.armor = empty.armor;
        app.equip.misc_equips = empty.misc_equips;
        tick_ammo(&mut app);
        {
            let p = app.player.as_ref().unwrap();
            assert!(!p.chloro_ammo_cost80 && !p.ammo_cost80 && !p.huntress_ammo_cost90);
            assert_eq!(p.ammo_cycling.mode, crate::ammo::AmmoCyclingMode::None);
        }
        // The Random and the "no cycling" accessories (`:14537`, `:14541`).
        app.equip.misc_equips[2] = slot(6193);
        tick_ammo(&mut app);
        assert_eq!(
            app.player.as_ref().unwrap().ammo_cycling.mode,
            crate::ammo::AmmoCyclingMode::Random
        );
        app.equip.misc_equips[3] = slot(6194);
        tick_ammo(&mut app);
        assert_eq!(
            app.player.as_ref().unwrap().ammo_cycling.mode,
            crate::ammo::AmmoCyclingMode::None,
            "6194 resets the mode even after 6193 granted it"
        );
        // The Sharp Barb (6160) is the last fuse flag (`Player.cs:14501`).
        app.equip.misc_equips[0] = slot(6160);
        tick_ammo(&mut app);
        assert!(
            app.player.as_ref().unwrap().acc_sharp_barb,
            "6160 → accSharpBarb"
        );
    }

    /// DW-3.4 — the per-tick cycling tick (`Player.Update_AmmoCycler`,
    /// `:28419-28426`): the cooldown decrements EVERY tick and the offset resets
    /// at its expiry, with NO weapon held and no shot taken. The trap is the
    /// per-shot-only reading: hooked to the shoot path, an idle player's offset
    /// would never return to 0 and the next swing would keep cycling from the
    /// stale stack. The C# runs the method at the head of `Player.Update`
    /// (`:24566`) before `ItemCheck`; the port runs it inside the fixed step,
    /// before `update_editing`'s shoot path (`frame.rs:285`/`:481`).
    #[test]
    fn test_DW_3_4_the_idle_player_ticks_the_cycling_cooldown_down_to_the_offset_reset() {
        let mut app = ammo_app();
        // The Ammo Cycler (6168) worn in a misc-equip slot, so the mode
        // re-derives every tick like the vanilla's `ResetEffects` clear +
        // `ApplyEquipFunctional` re-grant (`Player.cs:18479`, `:14529`) — the
        // mode is equipment state, never a pick-time write.
        app.equip.misc_equips[0] = slot(6168);
        // A pick's aftermath: the offset mid-cycle and a fresh cooldown.
        {
            let p = app.player.as_mut().unwrap();
            p.ammo_cycling.offset = 4;
            p.ammo_cycling.cooldown = 3;
        }
        // No weapon is held, no ammo is in the inventory, nothing is clicked:
        // the tick still runs, because it is a tick pass, not a shot pass.
        tick_ammo(&mut app);
        {
            let p = app.player.as_ref().unwrap();
            assert_eq!((p.ammo_cycling.cooldown, p.ammo_cycling.offset), (2, 4));
            assert_eq!(
                p.ammo_cycling.mode,
                crate::ammo::AmmoCyclingMode::AmmoSlots,
                "the mode is re-derived from the accessory every tick"
            );
        }
        tick_ammo(&mut app);
        {
            let p = app.player.as_ref().unwrap();
            assert_eq!((p.ammo_cycling.cooldown, p.ammo_cycling.offset), (1, 4));
        }
        tick_ammo(&mut app);
        {
            let p = app.player.as_ref().unwrap();
            assert_eq!(
                (p.ammo_cycling.cooldown, p.ammo_cycling.offset),
                (0, 0),
                "the expiry resets the offset with no shot in between"
            );
        }
        // The idle pass keeps running: an already-idle cycler never moves the
        // offset back up, and the mode survives the tick.
        tick_ammo(&mut app);
        tick_ammo(&mut app);
        {
            let p = app.player.as_ref().unwrap();
            assert_eq!((p.ammo_cycling.cooldown, p.ammo_cycling.offset), (0, 0));
            assert_eq!(p.ammo_cycling.mode, crate::ammo::AmmoCyclingMode::AmmoSlots);
        }
        // Unequipping the accessory clears the mode (the `ResetEffects` half) —
        // the offset and the cooldown are NOT equipment state and stay put.
        app.equip.misc_equips[0] = EMPTY_SLOT;
        tick_ammo(&mut app);
        {
            let p = app.player.as_ref().unwrap();
            assert_eq!(p.ammo_cycling.mode, crate::ammo::AmmoCyclingMode::None);
            assert_eq!((p.ammo_cycling.cooldown, p.ammo_cycling.offset), (0, 0));
        }
    }

    /// Review probe (Phase 2 gate fix 2) — the `update_player` wiring
    /// consumption (`Player.cs:28237-28241` + `Minecart.HitTrackSwitch`,
    /// `Minecart.cs:1281-1288`): riding across a pressure track fires
    /// `Wiring::hit_switch` at the magnet tile exactly once per pass. The
    /// seeded red line flips the lamp once — a second trip would flip it
    /// back. The `TR_CART` env-pin half stays screenshot-covered like
    /// `TR_HOOK` (a same-process env test is order-fragile with `OnceLock`).
    #[test]
    fn update_player_trips_the_pressure_track_through_wiring() {
        let mut app = flow_app(None, vec![], vec![]);
        // A 32x32 ride world: a straight rail on row 12 with the pressure
        // tile (frame 21, `_trackType == 1`) at x 14; a red wire on its own
        // tile runs down to a lamp (422) — the trip's flood flips it to 421.
        app.world = World::new("cart".into(), 32, 32);
        for x in 4..=24 {
            let i = app.world.index(x, 12);
            app.world.tiles[i] = Tile {
                kind: 314,
                frame_x: if x == 14 { 21 } else { 1 },
                frame_y: -1,
                ..Default::default()
            };
            app.world.tiles[i].flags |= FLAG_ACTIVE;
        }
        app.world.wires.set(14, 12, WireColor::Red, true);
        // The wire line below the rail row (the cart's body never reaches it),
        // mirroring the `wiring.rs` flood fixtures (kind-419 carriers).
        for y in 13..=14 {
            let i = app.world.index(14, y);
            app.world.tiles[i] = Tile {
                kind: 419,
                ..Default::default()
            };
            app.world.tiles[i].flags |= FLAG_ACTIVE;
            app.world.wires.set(14, y, WireColor::Red, true);
        }
        let i = app.world.index(14, 15);
        app.world.tiles[i] = Tile {
            kind: 422,
            ..Default::default()
        };
        app.world.tiles[i].flags |= FLAG_ACTIVE;
        app.world.wires.set(14, 15, WireColor::Red, true);
        // Mount and seat the cart five tiles left of the pressure tile, on
        // the frame-1 rail line (`row * 16 + 6`).
        let mut p = Player::spawn(&app.world);
        p.mount_cart();
        p.x = 10.0 * 16.0 + 8.0 - 10.0;
        p.y = 12.0 * 16.0 + 6.0 - 52.0;
        p.prev_x = p.x;
        p.prev_y = p.y;
        p.vx = 6.0;
        app.player = Some(p);
        // Hold right: the ride crosses the pressure tile and keeps going.
        app.keys.insert(KeyCode::KeyD);
        for _ in 0..20 {
            update_player(&mut app, 1.0 / 60.0);
        }
        let lamp = app.world.tile(14, 15).unwrap().kind;
        assert_eq!(lamp, 421, "the trip fired once through app.wiring");
        assert!(
            app.player
                .as_ref()
                .is_some_and(|p| p.cart_mounted && p.on_track),
            "the ride survived the pass"
        );
    }

    /// DW-3.1 — the `world_sim` drain: the tick's queued spark particles land
    /// in the App's `minecart::Sparks` pool (the `Main.dust` stand-in) and the
    /// queued cues go out through `app.audio.play_at` (a silent no-op without
    /// an output device) — both pendings end up empty.
    #[test]
    fn update_player_drains_cart_sparks_into_the_pool() {
        let mut app = flow_app(None, vec![], vec![]);
        // A straight rail on row 12 (frame 1) — the ride fixture.
        app.world = World::new("cart".into(), 64, 32);
        for x in 4..=56 {
            let i = app.world.index(x, 12);
            app.world.tiles[i] = Tile {
                kind: 314,
                frame_x: 1,
                frame_y: -1,
                ..Default::default()
            };
            app.world.tiles[i].flags |= FLAG_ACTIVE;
        }
        let mut p = Player::spawn(&app.world);
        p.mount_cart();
        p.x = 12.0 * 16.0 + 8.0 - 10.0;
        p.y = 12.0 * 16.0 + 6.0 - 52.0;
        p.prev_x = p.x;
        p.prev_y = p.y;
        p.vx = 12.0;
        // An all-gates-pass stream seed: the braking tick queues 3 sparks.
        p.spark_rng = seed_with_spark_count(10.4, 3);
        app.player = Some(p);
        // The settle tick brings the previous tick's `onTrack` up.
        for _ in 0..2 {
            update_player(&mut app, 1.0 / 60.0);
        }
        // Brake: hold left.
        app.keys.insert(KeyCode::KeyA);
        update_player(&mut app, 1.0 / 60.0);
        let p = app.player.as_mut().unwrap();
        assert!(
            p.take_cart_sparks().is_empty(),
            "the drain emptied the queue"
        );
        assert!(p.take_cart_sounds().is_empty(), "the cues went out");
        assert_eq!(app.minecart_sparks.count(), 3, "the pool holds the sparks");
    }

    /// Finds a stream seed where one rolling call at `|vx| = vel_x` queues
    /// exactly `want` sparks — the gate rolls interleave with the 14-roll
    /// sparks (the rain.rs seed-search idiom, run against the same emitter
    /// constants the tick passes).
    fn seed_with_spark_count(vel_x: f32, want: usize) -> u64 {
        for s in 1u64.. {
            let mut sparks = Vec::new();
            let mut rng = s;
            crate::minecart::rolling_wheel_sparks(
                &mut Vec::new(),
                &mut sparks,
                &mut rng,
                glam::Vec2::new(100.0, 200.0),
                20,
                52,
                glam::Vec2::new(vel_x, 0.0),
                13.0,
                0.0,
            );
            if sparks.len() == want {
                return s;
            }
        }
        unreachable!()
    }

    #[test]
    fn the_revive_revalidates_the_bed_spawn() {
        // DW-3.3 — `Player.Spawn`'s bed re-validation (`Player.cs:37571-37575`):
        // an intact bed survives the revive, a destroyed one reverts the
        // spawn to the world spawn and clears the port's `bed_spawn`.
        let mut app = flow_app(None, vec![], vec![]);
        app.world = World::new("bed".into(), 64, 48);
        {
            let w = &mut app.world;
            for x in 0..64 {
                let i = w.index(x, 20);
                w.tiles[i] = Tile {
                    kind: 1,
                    ..Default::default()
                };
                w.tiles[i].flags |= FLAG_ACTIVE;
            }
            // A minimal valid NPC room around the bed (the wood frame +
            // background walls — the room check needs the 60-tile interior).
            for x in 14..=26 {
                for y in 12..=20 {
                    let i = w.index(x, y);
                    w.tiles[i].wall = 27;
                    let border = x == 14 || x == 26 || y == 12 || y == 20;
                    if border {
                        w.tiles[i] = Tile {
                            kind: 9,
                            frame_x: -1,
                            frame_y: -1,
                            ..Default::default()
                        };
                        w.tiles[i].flags |= FLAG_ACTIVE;
                    }
                }
            }
            // The bed's bottom cell at (20, 19); the spawn cell (20, 20).
            let bed = w.index(20, 19);
            w.tiles[bed] = Tile {
                kind: 79,
                frame_y: 18,
                ..Default::default()
            };
            w.tiles[bed].flags |= FLAG_ACTIVE;
            w.bed_spawn = Some((20, 20));
        }
        // The load rebuild: `Player::spawn` reads the validated bed spawn
        // (DW-3.2's sim-side arm).
        app.player = Some(Player::spawn(&app.world));
        let bed = app.world.index(20, 19);
        {
            let p = app.player.as_mut().unwrap();
            p.start_death(&app.world, false, false);
        }
        // The stepper caps the steps per call — feed the 600-tick respawn
        // timer in frame-sized bites.
        for _ in 0..700 {
            update_player(&mut app, 1.0 / 60.0);
        }
        {
            let p = app.player.as_ref().unwrap();
            assert!(!p.dead, "the revive fired");
            assert_eq!(p.spawn_x, 20.0 * tr_core::TILE_PX + 8.0 - 10.0);
        }
        assert_eq!(app.world.bed_spawn, Some((20, 20)), "the intact bed stays");
        // The bed breaks — the next death reverts to the world spawn.
        app.world.tiles[bed].flags &= !FLAG_ACTIVE;
        {
            let p = app.player.as_mut().unwrap();
            p.start_death(&app.world, false, false);
        }
        for _ in 0..700 {
            update_player(&mut app, 1.0 / 60.0);
        }
        let w_spawn = (app.world.spawn_x as f32 * tr_core::TILE_PX + 8.0
            - crate::player::WIDTH as f32 * 0.5);
        let p = app.player.as_ref().unwrap();
        assert_eq!(p.spawn_x, w_spawn, "the broken bed reverts the spawn");
        assert_eq!(app.world.bed_spawn, None);
    }
}
