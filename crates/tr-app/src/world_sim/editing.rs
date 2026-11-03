//! Tile and wall editing (plan stage A4): the per-frame mining/placing loop,
//! the break/place helpers behind it and the replicated-edit applier the net
//! layer feeds.

use glam::Vec2;
use tr_core::tiledata as td;
use tr_core::{TILE_PX, World, edit};

use crate::app::App;
use crate::projectile::Projectile;
use crate::registry;

use super::items::spawn_item_drop;

pub(crate) fn update_editing(app: &mut App, dt: f32, screen: Vec2) {
    app.swing = (app.swing - dt).max(0.0);
    // `Player.releaseUseItem = !controlUseItem` runs every tick
    // (`Player.cs:42822`), so the press/release edges stay current even on
    // frames where the cursor is over the HUD or a panel.
    app.place_prev_down = app.place_down;
    app.mine_prev_down = app.mine_down;
    app.place_released = !app.place_down;
    app.mine_released = !app.mine_down;
    // The `itemTime` accumulator (`Player.ApplyItemTime`, `Player.cs:4416`):
    // the *action* cadence inside a swing — placement fires whenever it hits
    // zero while the swing animation is still running.
    app.item_time = (app.item_time - dt).max(0.0);
    if let Some((x, y)) = app.dev_break.take() {
        break_tile(app, x, y);
    }
    for (x, y, kind) in std::mem::take(&mut app.dev_place) {
        if edit::place_tile(&mut app.world, x, y, kind) {
            app.liquids.wake_around(&app.world, x, y, 3);
            app.world_key = None;
        }
    }
    // `Player.cs:24798-24807`: while the map is fullscreen `controlUseItem`/
    // `controlUseTile` are cleared, so no mining, placing or attacking happens
    // behind it; the left button drags the map instead (`update_map_input`).
    if app.map.as_ref().is_some_and(|m| m.open) {
        app.edit_target = None;
        app.mine_target = None;
        app.mine_timer = 0.0;
        return;
    }
    // `MinimapFrame.Update` sets `Main.LocalPlayer.mouseInterface = true`
    // while a zoom button is hovered (`MinimapFrame.cs:164`), which zeroes
    // the use-item / use-tile inputs for the frame — so mining, placing and
    // attacking are suppressed behind the frame the same way the fullscreen
    // map suppresses them above (`handle_minimap_clicks` keeps this flag
    // current, having run earlier this frame).
    if app.minimap_hover.is_some() {
        app.edit_target = None;
        app.mine_target = None;
        app.mine_timer = 0.0;
        return;
    }
    let m = app.mouse_pos;
    let hud_top = screen.y - 96.0;
    if m.x < 0.0
        || m.y < 0.0
        || m.x > screen.x
        || m.y > screen.y
        || (m.y > hud_top && m.x < 300.0)
        || (app.debug && m.x < 780.0 && m.y < 340.0)
    {
        app.edit_target = None;
        app.mine_target = None;
        app.mine_timer = 0.0;
        return;
    }
    let world_pos = app.camera + (m - screen * 0.5) / app.zoom;
    let tx = (world_pos.x / TILE_PX).floor() as i32;
    let ty = (world_pos.y / TILE_PX).floor() as i32;
    if !app.world.in_bounds(tx, ty) {
        app.edit_target = None;
        return;
    }
    let Some(player) = &app.player else {
        app.edit_target = None;
        return;
    };
    let (pcx, pcy) = player.center();
    let dx = tx as f32 * TILE_PX + 8.0 - pcx;
    let dy = ty as f32 * TILE_PX + 8.0 - pcy;
    let reach = 6.0 * TILE_PX;
    if dx * dx + dy * dy > reach * reach {
        app.edit_target = None;
        app.mine_target = None;
        app.mine_timer = 0.0;
        return;
    }
    app.edit_target = Some((tx, ty));
    let held = app.inventory[app.selected];
    let held_weapon = held.count > 0 && registry::is_weapon(held.kind);
    if app.mine_down && held_weapon && !app.inventory_open {
        // Weapons use LMB toward the cursor instead of mining. The shuriken
        // is not `autoReuse` (`Item.cs:1869-1885`): one throw per press edge
        // (`Player.cs:42733` `controlUseItem && releaseUseItem && itemAnimation == 0`).
        let throw_ready = registry::get(held.kind).is_some_and(|d| d.auto_reuse)
            || (app.mine_released && !app.mine_prev_down);
        if app.attack_cd <= 0.0 && throw_ready {
            let (pcx, pcy) = player.center();
            let dir = (world_pos - Vec2::new(pcx, pcy)).normalize_or_zero();
            if dir != Vec2::ZERO {
                let spawn = Vec2::new(pcx, pcy) + dir * 10.0;
                app.projectiles.push(Projectile::shuriken(
                    spawn.x - 3.0,
                    spawn.y - 3.0,
                    dir.x * 11.0,
                    dir.y * 11.0,
                ));
                app.inventory[app.selected].count =
                    app.inventory[app.selected].count.saturating_sub(1);
                // `Player.ApplyItemAnimation` (`Player.cs:4449-4451`): the swing
                // lasts the item's `useAnimation` ticks; `attack_cd` matches so
                // the throw repeats only when the swing is over.
                app.swing_max = crate::item::use_animation_secs(held.kind);
                app.swing = app.swing_max;
                app.attack_cd = app.swing_max;
            }
        }
        return;
    }
    // Melee: LMB over an NPC within four tiles hits it instead of mining.
    if app.mine_down && app.attack_cd <= 0.0 && !app.inventory_open {
        let ax = world_pos.x;
        let ay = world_pos.y;
        let reach2 = 4.0 * TILE_PX;
        if (ax - pcx).powi(2) + (ay - pcy).powi(2) <= reach2 * reach2 {
            if let Some(n) = app.npcs.iter_mut().find(|n| n.contains(ax, ay)) {
                // `Player.ItemCheck_MeleeHitNPCs` -> `NPC.StrikeNPC`:
                // `DamageVar` -> `CalculateDamageNPCsTake` with the NPC's
                // defense (`NPC.cs:81226`; armor pen joins when the item
                // data carries it — M3-TOOL).
                let raw = if held_weapon {
                    registry::damage(held.kind) as f32
                } else {
                    15.0
                };
                let mut loot_rng = crate::world_sim::npcs::LootRng(&mut app.item_rng);
                let dmg = tr_core::damage::damage_var(&mut loot_rng, raw, 0.0);
                let dmg = tr_core::damage::strike_npc_damage(dmg, n.defense(), false);
                // `NPC.StrikeNPC` plays `this.HitSound` at `base.position`
                // (`NPC.cs:81546`) so the hit is placed at the NPC corner.
                let (nx, ny) = (n.x, n.y);
                // The melee stand-in's knockback: a mid-sword 5.5
                // (`Item.knockBack` for the copper shortsword class); the
                // per-item values land with M3-ITEM.
                n.hurt(dmg, pcx, 5.5, false);
                app.audio.play_at("NPC_Hit_1", nx, ny, 1.0);
                // One swing per `useAnimation` (`Player.cs:4449-4451`).
                app.swing_max = crate::item::use_animation_secs(held.kind);
                app.swing = app.swing_max;
                app.attack_cd = app.swing_max;
                app.mine_target = None;
                app.mine_timer = 0.0;
                return;
            }
        }
    }
    let Some(tile) = app.world.tile(tx, ty).copied() else {
        app.edit_target = None;
        return;
    };
    let important = app
        .world
        .frame_important
        .get(tile.kind as usize)
        .copied()
        .unwrap_or(false)
        && tile.kind != 80; // cacti have fixed frames but are mineable
    let held_wall = held.count > 0 && registry::place_wall(held.kind).is_some();

    if app.mine_down && tile.active() && !important && !app.inventory_open {
        if app.swing <= 0.0 {
            app.swing_max = crate::item::use_animation_secs(held.kind);
            app.swing = app.swing_max;
        }
        if app.mine_target == Some((tx, ty)) {
            app.mine_timer += dt;
        } else {
            app.mine_target = Some((tx, ty));
            app.mine_timer = 0.0;
        }
        let hardness = if tile.kind == 1 || (6..=9).contains(&tile.kind) {
            0.55
        } else {
            0.25
        };
        if app.mine_timer >= hardness {
            break_tile(app, tx, ty);
            app.mine_target = None;
            app.mine_timer = 0.0;
        }
    } else if app.mine_down && !tile.active() && tile.wall != 0 && !app.inventory_open {
        // No block in the cell: mining it removes the exposed wall.
        if app.swing <= 0.0 {
            app.swing_max = crate::item::use_animation_secs(held.kind);
            app.swing = app.swing_max;
        }
        if app.mine_target == Some((tx, ty)) {
            app.mine_timer += dt;
        } else {
            app.mine_target = Some((tx, ty));
            app.mine_timer = 0.0;
        }
        let hardness = if tile.wall == 1 { 0.7 } else { 0.35 };
        if app.mine_timer >= hardness {
            break_wall(app, tx, ty);
            app.mine_target = None;
            app.mine_timer = 0.0;
        }
    } else {
        app.mine_target = None;
        app.mine_timer = 0.0;
    }

    // `Player.ItemCheck` pacing: the swing animation lasts `useAnimation`
    // (`Player.cs:4443-4451`); the *action* re-fires whenever `itemTime`
    // reaches zero while the animation still runs and the button is held
    // (`Player.cs:39743` `ItemTimeIsZero && itemAnimation > 0 &&
    // controlUseItem`, `ApplyItemTime = useTime`, `Player.cs:4416-4418`) —
    // dirt places every 10 ticks inside the 15-tick swing, walls every 7.
    // An `autoReuse` kind restarts the swing the moment it ends
    // (`Player.cs:42819-42822`); a non-autoReuse kind waits for a release.
    let auto_reuse = registry::get(held.kind).is_some_and(|d| d.auto_reuse);
    let timings = registry::get(held.kind);
    let swing_secs = crate::item::use_animation_secs(held.kind);
    let action_secs = timings
        .map(|d| d.use_time as f32 / 60.0)
        .filter(|t| *t > 0.0)
        .unwrap_or(swing_secs);
    let swing_over = app.swing <= 0.0;
    let reuse_ready = auto_reuse || (app.place_released && !app.place_prev_down);
    let starting = swing_over && reuse_ready;
    let inside_swing = app.swing > 0.0 && app.item_time <= 0.0;
    if app.place_down && !app.inventory_open && (starting || inside_swing) {
        let used = if held_wall {
            if can_place_wall(app, tx, ty) {
                place_wall_item(app, tx, ty, registry::place_wall(held.kind).unwrap_or(1));
                true
            } else {
                false
            }
        } else if !held_weapon && can_place(app, tx, ty) {
            match registry::place_tile(held.kind) {
                Some(kind) => {
                    place_tile(app, tx, ty, kind);
                    true
                }
                None => false,
            }
        } else {
            false
        };
        if used {
            if starting {
                app.swing_max = swing_secs;
                app.swing = app.swing_max;
            }
            app.item_time = action_secs;
        }
    }
}

pub(crate) fn break_wall(app: &mut App, tx: i32, ty: i32) {
    let Some(wall) = edit::break_wall(&mut app.world, tx, ty) else {
        return;
    };
    if let Some(id) = registry::by_wall(wall) {
        spawn_item_drop(app, id, tx, ty);
    }
    app.audio
        .play_at("Dig_2", tx as f32 * TILE_PX, ty as f32 * TILE_PX, 1.0);
    app.request_edit(tx, ty, 2, 0);
    app.world_key = None;
}

pub(crate) fn can_place_wall(app: &App, tx: i32, ty: i32) -> bool {
    let Some(t) = app.world.tile(tx, ty) else {
        return false;
    };
    if t.wall != 0
        || app.inventory[app.selected].count == 0
        || registry::is_weapon(app.inventory[app.selected].kind)
    {
        return false;
    }
    // A neighbouring wall or block (including one in the cell itself) supports it.
    if t.active() && (t.kind as usize) < td::TILE_COUNT && td::TILE_SOLID.has(t.kind) {
        return true;
    }
    [(0, -1), (0, 1), (-1, 0), (1, 0)].iter().any(|&(dx, dy)| {
        app.world
            .tile(tx + dx, ty + dy)
            .is_some_and(|n| n.wall != 0)
    })
}

pub(crate) fn place_wall_item(app: &mut App, tx: i32, ty: i32, wall: u16) {
    if edit::place_wall(&mut app.world, tx, ty, wall) {
        app.inventory[app.selected].count = app.inventory[app.selected].count.saturating_sub(1);
        app.audio
            .play_at("Dig_0", tx as f32 * TILE_PX, ty as f32 * TILE_PX, 1.0);
        app.request_edit(tx, ty, 3, wall);
        app.world_key = None;
    }
}

pub(crate) fn break_tile(app: &mut App, tx: i32, ty: i32) {
    let Some(kind) = edit::break_tile(&mut app.world, tx, ty) else {
        return;
    };
    let Some(id) = registry::tile_drop(kind) else {
        app.request_edit(tx, ty, 0, 0);
        return;
    };
    spawn_item_drop(app, id, tx, ty);
    app.audio
        .play_at("Dig_1", tx as f32 * TILE_PX, ty as f32 * TILE_PX, 1.0);
    app.request_edit(tx, ty, 0, 0);
    app.liquids.wake_around(&app.world, tx, ty, 3);
    app.world_key = None;
}

pub(crate) fn can_place(app: &App, tx: i32, ty: i32) -> bool {
    let Some(t) = app.world.tile(tx, ty) else {
        return false;
    };
    if t.active()
        || app.inventory[app.selected].count == 0
        || registry::is_weapon(app.inventory[app.selected].kind)
        || app.inventory[app.selected].kind == 80
    {
        return false;
    }
    if let Some(p) = &app.player {
        let (x0, y0) = (tx as f32 * TILE_PX, ty as f32 * TILE_PX);
        if x0 < p.x + crate::player::WIDTH
            && x0 + TILE_PX > p.x
            && y0 < p.y + crate::player::HEIGHT
            && y0 + TILE_PX > p.y
        {
            return false;
        }
    }
    [(0, -1), (0, 1), (-1, 0), (1, 0)].iter().any(|&(dx, dy)| {
        app.world.tile(tx + dx, ty + dy).is_some_and(|n| {
            n.wall != 0
                || (n.active() && (n.kind as usize) < td::TILE_COUNT && td::TILE_SOLID.has(n.kind))
        })
    })
}

pub(crate) fn place_tile(app: &mut App, tx: i32, ty: i32, kind: u16) {
    if edit::place_tile(&mut app.world, tx, ty, kind) {
        app.inventory[app.selected].count = app.inventory[app.selected].count.saturating_sub(1);
        app.liquids.wake_around(&app.world, tx, ty, 3);
        app.audio
            .play_at("Dig_0", tx as f32 * TILE_PX, ty as f32 * TILE_PX, 1.0);
        app.request_edit(tx, ty, 1, kind);
        app.world_key = None;
    }
}

/// Applies a replicated edit. `kind`: 0 break tile, 1 place tile, 2 break wall,
/// 3 place wall.
pub(crate) fn apply_edit(world: &mut World, x: i32, y: i32, kind: u8, tile: u16) {
    match kind {
        0 => {
            edit::break_tile(world, x, y);
        }
        1 => {
            edit::place_tile(world, x, y, tile);
        }
        2 => {
            edit::break_wall(world, x, y);
        }
        3 => {
            edit::place_wall(world, x, y, tile);
        }
        _ => {}
    }
}
