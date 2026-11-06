//! Player simulation (plan stage A4): the deterministic input assembly and the
//! fixed 60 Hz `Player::update` stepping with the render interpolation the
//! camera follows.

use glam::Vec2;
use winit::keyboard::KeyCode;

use crate::app::App;
use crate::player::PlayerInput;

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
        let step = 1.0 / 60.0;
        if cart_pin && !p.cart_mounted {
            p.mount_cart();
        }
        app.player_step.tick(dt, |_| {
            p.update_with_grapple(&app.world, input, grapple_for_tick);
            // `Player.cs:28237-28241` + `Minecart.HitTrackSwitch`
            // (`Minecart.cs:1281-1288`): the pressure-track trip fires
            // `Wiring.HitSwitch` at the magnet tile (net message 59 is out of
            // scope — local only).
            if let Some((i, j)) = p.take_switch_trip() {
                app.wiring.hit_switch(&mut app.world, i, j);
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
    if map_open {
        app.update_map_input();
    }
}
