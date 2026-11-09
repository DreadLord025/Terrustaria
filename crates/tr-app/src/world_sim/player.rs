//! Player simulation (plan stage A4): the deterministic input assembly and the
//! fixed 60 Hz `Player::update` stepping with the render interpolation the
//! camera follows.

use glam::Vec2;
use winit::keyboard::KeyCode;

use crate::app::App;
use crate::player::PlayerInput;

pub(crate) fn update_player(app: &mut App, dt: f32) {
    // Deterministic held input for screenshots (`TR_HOLD=right,jump`).
    let (hold_right, hold_jump) = crate::env::hold_input();
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
    let hook_type = crate::env::hook_type();
    // `TR_CART` pins the minecart mount (the granting item, QuickMount and the
    // TrackHook paths are M2-MOUNT-3) — mounted idempotently like the
    // `TR_HOOK` parse above, so a dismount re-mounts on the next tick.
    let cart_pin = crate::env::cart_pin();
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
    use crate::player::Player;
    use tr_core::tile::FLAG_ACTIVE;
    use tr_core::wiring::WireColor;
    use tr_core::{Tile, World};

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
