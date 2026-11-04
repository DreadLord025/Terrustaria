//! Tile and wall editing (plan stage A4): the per-frame mining/placing loop,
//! the break/place helpers behind it and the replicated-edit applier the net
//! layer feeds.

use glam::Vec2;
use tr_core::tiledata as td;
use tr_core::{TILE_PX, World, edit};

use crate::app::App;
use crate::item::ItemSlot;
use crate::npc::Npc;
use crate::player::Player;
use crate::projectile::Projectile;
use crate::achievements;
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
    // `Player.ItemCheck`'s `!frozen && !webbed` gate (`Player.cs:3381`): a
    // frozen or webbed player cannot swing, throw, mine or place.
    if app
        .player
        .as_ref()
        .is_some_and(|p| p.frozen || p.webbed)
    {
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
    let item_crit = registry::get(held.kind).map(|d| d.crit).unwrap_or(0);
    if app.mine_down && held_weapon && !app.inventory_open {
        // Weapons use LMB toward the cursor instead of mining. The shuriken
        // is not `autoReuse` (`Item.cs:1869-1885`): one throw per press edge
        // (`Player.cs:42733` `controlUseItem && releaseUseItem && itemAnimation == 0`).
        let throw_ready = registry::get(held.kind).is_some_and(|d| d.auto_reuse)
            || (app.mine_released && !app.mine_prev_down);
        if app.attack_cd <= 0.0 && throw_ready {
            // `Player.ItemCheck_Shoot` (`Player.cs:47056+`): the generic
            // `Item.shoot` firing — the aim vector normalized to
            // `shootSpeed` (`:47274-47292`; the melee `speed /= meleeSpeed`
            // is ×1 at the port's default melee speed), the spawn at the
            // player center (`RotatedRelativePoint(MountedCenter)`), the
            // damage `weaponDamage` (the class multiplier) and the item's
            // knockback.
            let def = registry::get(held.kind);
            if let (Some(proj), Some(def)) = (registry::projectile(held.kind), def) {
                let (pcx, pcy) = player.center();
                let dir = (world_pos - Vec2::new(pcx, pcy)).normalize_or_zero();
                if dir != Vec2::ZERO {
                    let spawn = Vec2::new(pcx, pcy) + dir * 10.0;
                    let class = def.class;
                    let raw =
                        def.damage as f32 * player.weapon_damage_multiplier(class);
                    // The crit rolls at throw time (the class crit + the
                    // item's crit; `Projectile.Damage` consumes it on hit) —
                    // the item stream lives here, not in the projectile step.
                    let mut loot_rng = crate::world_sim::npcs::LootRng(&mut app.item_rng);
                    let crit =
                        tr_core::damage::crit_roll(&mut loot_rng, player.class_crit(class) + item_crit);
                    let slot = app.projectiles.spawn_with_data(
                        proj,
                        spawn.x,
                        spawn.y,
                        dir.x * def.shoot_speed,
                        dir.y * def.shoot_speed,
                        raw as i32,
                        def.knock_back,
                    );
                    if let Some(slot) = slot {
                        app.projectiles.set_crit(slot, crit);
                    }
                    if def.consumable {
                        app.inventory[app.selected].count =
                            app.inventory[app.selected].count.saturating_sub(1);
                    }
                    // `Player.ApplyItemAnimation` (`Player.cs:4449-4451`): the
                    // swing lasts the item's `useAnimation` ticks; `attack_cd`
                    // matches so the throw repeats only when the swing is over.
                    app.swing_max = crate::item::use_animation_secs(held.kind);
                    app.swing = app.swing_max;
                    app.attack_cd = app.swing_max;
                }
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
                // `Player.ItemCheck_MeleeHitNPCs`: the class multiplier, the
                // crit roll (`meleeCrit` + the item's crit), `DamageVar`, then
                // `CalculateDamageNPCsTake` with the NPC's defense
                // (`NPC.cs:81226`; armor pen joins when the item data carries
                // it — M3-TOOL).
                let class = if held_weapon {
                    registry::get(held.kind)
                        .map(|d| d.class)
                        .unwrap_or_default()
                } else {
                    registry::DamageClass::Melee
                };
                let item_crit = registry::get(held.kind).map(|d| d.crit).unwrap_or(0);
                let raw = if held_weapon {
                    registry::damage(held.kind) as f32 * player.weapon_damage_multiplier(class)
                } else {
                    15.0
                };
                let mut loot_rng = crate::world_sim::npcs::LootRng(&mut app.item_rng);
                let crit =
                    tr_core::damage::crit_roll(&mut loot_rng, player.class_crit(class) + item_crit);
                let dmg = tr_core::damage::damage_var(&mut loot_rng, raw, 0.0);
                let dmg = tr_core::damage::strike_npc_damage(dmg, n.defense(), crit);
                // `NPC.StrikeNPC` plays `this.HitSound` at `base.position`
                // (`NPC.cs:81546`) so the hit is placed at the NPC corner.
                let (nx, ny) = (n.x, n.y);
                // The melee stand-in's knockback: a mid-sword 5.5
                // (`Item.knockBack` for the copper shortsword class); the
                // per-item values land with M3-ITEM.
                n.hurt(dmg, pcx, 5.5, crit);
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
    // Tile interactions (`Player.Update` order: LookForTileInteractions
    // (`Player.cs:27430`) runs before ItemCheck (`:31827`)): one action per
    // right-click press — `UpdateReleaseUseTile` (`:29224`) clears
    // `releaseUseTile` right after the attempt, so the door branch fires on
    // the press edge. A handled interaction consumes the click; anything else
    // falls through to the place path unchanged.
    if app.place_down && !app.place_prev_down && !app.inventory_open && interact_tile(app, tx, ty) {
        return;
    }
    if app.place_down && !app.inventory_open && (starting || inside_swing) {
        let used = if held_wall {
            if can_place_wall(app, tx, ty) {
                place_wall_item(app, tx, ty, registry::place_wall(held.kind).unwrap_or(1));
                true
            } else {
                false
            }
        } else if !held_weapon {
            use_tile_item(app, tx, ty)
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
    // `AchievementsHelper.NotifyTileDestroyed` + the CustomInt "Pick"
    // counter (BULLDOZER) — the destroyed tile feeds both rows.
    let kind = app.world.tile(tx, ty).filter(|t| t.active()).map(|t| t.kind);
    if let Some(kind) = kind {
        let _ = app
            .achievements
            .notify(achievements::CondKind::TileDestroyed(kind));
        let _ = app
            .achievements
            .notify(achievements::CondKind::CustomInt("Pick", 10000));
    }
    let Some(kill) = edit::kill_tile(&mut app.world, tx, ty, false) else {
        return;
    };
    let drops = break_drop_events(&kill, tx, ty);
    if drops.is_empty() {
        app.request_edit(tx, ty, 0, 0);
        return;
    }
    for (id, x, y) in drops {
        spawn_item_drop(app, id, x, y);
        app.audio
            .play_at("Dig_1", x as f32 * TILE_PX, y as f32 * TILE_PX, 1.0);
    }
    app.request_edit(tx, ty, 0, 0);
    app.liquids.wake_around(&app.world, tx, ty, 3);
    // `WorldGen.TileFrame`'s falling check (`WorldGen.cs:81380`): the tile
    // above the break loses its support and the falling kind spawns the
    // sandfall projectile (`SpawnFallingBlockProjectile`, `:86039-86090`).
    if let Some(fall) = tr_core::falling::check_falling(&app.world, tx, ty - 1) {
        if let Some(t) = app.world.tile_mut(tx, ty - 1) {
            *t = Default::default();
        }
        let _ = app.projectiles.spawn_falling(
            fall.tile_kind,
            fall.proj_kind,
            fall.damage,
            fall.x,
            fall.y,
        );
    }
    app.world_key = None;
}

/// The item drops of one `kill_tile` result, resolved through
/// `registry::tile_drop` (`KillTile_DropItems` → `KillTile_GetItemDrops` for
/// the hit cell at `(tx, ty)`, the `Check*` `Item.NewItem` records for
/// destroyed objects — `WorldGen.cs:63860-63864`,
/// :86508/:49424/:43494/:47121/:57972). Registry kinds drop nothing per-cell
/// (`KillTile_GetItemDrops`'s empty case group, `WorldGen.cs:65748-66091`):
/// their item comes from the cascade record only. Free of `App` so the
/// mapping is unit-testable without the app harness.
fn break_drop_events(kill: &edit::TileKill, tx: i32, ty: i32) -> Vec<(u16, i32, i32)> {
    let mut events = Vec::new();
    if tr_core::tileobject::def_for_tile(kill.kind).is_none() {
        if let Some(id) = registry::tile_drop(kill.kind) {
            events.push((id, tx, ty));
        }
    }
    for drop in &kill.object_drops {
        if let Some(id) = registry::tile_drop(drop.tile_kind) {
            events.push((id, drop.x, drop.y));
        }
    }
    events
}

pub(crate) fn can_place(app: &App, tx: i32, ty: i32) -> bool {
    let Some(t) = app.world.tile(tx, ty) else {
        return false;
    };
    if t.active()
        || item_gate_blocks(
            app.inventory[app.selected].kind,
            app.inventory[app.selected].count,
        )
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

/// The item gate shared by both placement routes (the head of `can_place` and
/// of `use_tile_item`): an empty slot, a weapon or the legacy cactus kind
/// (80, `ItemID.Cactus`) never places. Free of `App` so the decision is
/// unit-testable without the app harness.
fn item_gate_blocks(kind: u16, count: u32) -> bool {
    count == 0 || registry::is_weapon(kind) || kind == 80
}

/// The routing decision of `use_tile_item`: the legacy neighbour/overlap
/// pre-checks (`can_place`) gate only kinds without a `TileObjectData`
/// registry entry — registry kinds are judged by `TileObject.CanPlace`
/// inside the dispatcher (the anchors decide — vanilla furniture can sit on
/// platforms or over the player, and `BlockPlacementIfOverPlayers`,
/// `Player.cs:41427`, covers none of the wooden set).
fn legacy_precheck_applies(kind: u16) -> bool {
    tr_core::tileobject::def_for_tile(kind).is_none()
}

/// What `TileObject.CanPlace` receives in vanilla (`Player.cs:39772`):
/// `Item.placeStyle` of the held item and `Player.direction`, the facing
/// defaulting to right (+1) with no player — the same default the replicated
/// `apply_edit` route sends (Decision #7).
fn style_and_facing(held_kind: u16, facing: Option<i32>) -> (i32, i32) {
    (registry::place_style(held_kind), facing.unwrap_or(1))
}

/// The world-write half of `place_tile`: the dispatcher write
/// (`edit::place_item_tile`) followed by the item consumption vanilla ties to
/// a successful `Place` (`Player.cs:40118-40126`) — a refused placement
/// keeps the item. Free of `App` so the consumption condition is
/// unit-testable without the app harness.
fn place_item_and_consume(
    world: &mut World,
    held: &mut ItemSlot,
    tx: i32,
    ty: i32,
    kind: u16,
    style: i32,
    dir: i32,
) -> bool {
    if !edit::place_item_tile(world, tx, ty, kind, style, dir) {
        return false;
    }
    held.count = held.count.saturating_sub(1);
    true
}

/// The tile-placement route of `Player.PlaceThing_Tiles` (`Player.cs:39721`):
/// the shared item gates, then `tr_core::edit::place_item_tile` dispatches
/// registry kinds through `TileObject.CanPlace`/`Place`
/// (`PlaceThing_Tiles_TryPlacing`, `Player.cs:39771-39778`) and everything else
/// through the legacy `WorldGen.PlaceTile` path with the app's
/// neighbour/overlap pre-checks (`can_place`).
fn use_tile_item(app: &mut App, tx: i32, ty: i32) -> bool {
    let held = app.inventory[app.selected];
    if item_gate_blocks(held.kind, held.count) {
        return false;
    }
    let Some(kind) = registry::place_tile(held.kind) else {
        return false;
    };
    if legacy_precheck_applies(kind) && !can_place(app, tx, ty) {
        return false;
    }
    place_tile(app, tx, ty, kind)
}

pub(crate) fn place_tile(app: &mut App, tx: i32, ty: i32, kind: u16) -> bool {
    // `Item.placeStyle` + `Player.direction` are what `TileObject.CanPlace`
    // receives in vanilla (`Player.cs:39772`); the dispatcher resolves style
    // alternates and click points from them.
    let (style, dir) = style_and_facing(
        app.inventory[app.selected].kind,
        app.player.as_ref().map(|p| p.facing),
    );
    if !place_item_and_consume(
        &mut app.world,
        &mut app.inventory[app.selected],
        tx,
        ty,
        kind,
        style,
        dir,
    ) {
        return false;
    }
    app.liquids.wake_around(&app.world, tx, ty, 3);
    app.audio
        .play_at("Dig_0", tx as f32 * TILE_PX, ty as f32 * TILE_PX, 1.0);
    app.request_edit(tx, ty, 1, kind);
    app.world_key = None;
    true
}

/// The right-click interaction hook (the port's `TileInteractionsUse`):
/// dispatches the world interaction at `(tx, ty)` through
/// [`tr_core::edit::interact_at`] and applies its effects — the door cue
/// (`SoundEngine.PlaySound(8/:9)`, `WorldGen.cs:37732/:31906`), the drop
/// spawns of any cleanup kills, the frame-cache invalidation. Returns whether
/// the click was consumed; `false` lets the caller fall through to the place
/// path.
fn interact_tile(app: &mut App, tx: i32, ty: i32) -> bool {
    let direction = app.player.as_ref().map(|p| p.facing).unwrap_or(1);
    // The entity half of `Collision.EmptyTile` (`Collision.cs:1359`): the
    // closing column must be free of the player and NPCs. Disjoint field
    // borrows — the world write happens inside `interact_at`.
    let (player, npcs) = (&app.player, &app.npcs);
    let cell_blocked = |x: i32, y: i32| cell_blocked_by_entity(player.as_ref(), npcs, x, y);
    let result = edit::interact_at(&mut app.world, tx, ty, direction, &cell_blocked);
    let Some((cue, drops)) = interaction_events(result) else {
        return false;
    };
    if let Some(cue) = cue {
        app.audio
            .play_at(cue, tx as f32 * TILE_PX, ty as f32 * TILE_PX, 1.0);
    }
    for (id, x, y) in drops {
        spawn_item_drop(app, id, x, y);
    }
    app.world_key = None;
    true
}

/// The App-free half of [`interact_tile`]: maps the dispatcher result to the
/// door cue and the drop spawns. `None` = no interaction (fall-through to the
/// place path); a blocked door yields drops without a cue (vanilla plays
/// `PlaySound(8/:9)` only on the success paths).
fn interaction_events(
    result: edit::TileInteraction,
) -> Option<(Option<&'static str>, Vec<(u16, i32, i32)>)> {
    let (cue, object_drops) = match result {
        edit::TileInteraction::None => return None,
        edit::TileInteraction::DoorOpened { object_drops } => (Some("Door_Opened"), object_drops),
        edit::TileInteraction::DoorClosed { object_drops } => (Some("Door_Closed"), object_drops),
        edit::TileInteraction::DoorBlocked { object_drops } => (None, object_drops),
    };
    let events = object_drops
        .iter()
        .filter_map(|d| registry::tile_drop(d.tile_kind).map(|id| (id, d.x, d.y)))
        .collect();
    Some((cue, events))
}

/// The entity half of `Collision.EmptyTile(i, j, ignoreTiles: true)`
/// (`Collision.cs:1359-1378`): the cell rectangle must not intersect any
/// player or NPC rectangle (XNA `Rectangle.Intersects` semantics — strict
/// overlap, touching edges are free). The port keeps no dead/ghost flags for
/// the single local player. Free of `App` so the gate is unit-testable
/// without the app harness.
fn cell_blocked_by_entity(player: Option<&Player>, npcs: &[Npc], x: i32, y: i32) -> bool {
    let (x0, y0) = (x as f32 * TILE_PX, y as f32 * TILE_PX);
    let (x1, y1) = (x0 + TILE_PX, y0 + TILE_PX);
    if let Some(p) = player {
        if x0 < p.x + crate::player::WIDTH
            && x1 > p.x
            && y0 < p.y + crate::player::HEIGHT
            && y1 > p.y
        {
            return true;
        }
    }
    npcs.iter().any(|n| {
        let (w, h) = n.kind.size();
        x0 < n.x + w && x1 > n.x && y0 < n.y + h && y1 > n.y
    })
}

/// Applies a replicated edit. `kind`: 0 break tile, 1 place tile, 2 break wall,
/// 3 place wall.
pub(crate) fn apply_edit(world: &mut World, x: i32, y: i32, kind: u8, tile: u16) {
    match kind {
        0 => {
            // Vanilla suppresses item drops on the replication side
            // (`Main.netMode != 1` gate, `WorldGen.cs:63860`); the cascade
            // itself must run so every client converges on the same world.
            edit::kill_tile(world, x, y, true);
        }
        1 => {
            // Replicated placements take the same route as the local player.
            // Vanilla replicates object placements with a dedicated message
            // (`NetMessage.SendObjectPlacement`) carrying style/alternate/
            // random/direction; the generic edit message cannot, so the
            // registry route converges with style 0 / facing right.
            edit::place_item_tile(world, x, y, tile, 0, 1);
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

#[cfg(test)]
mod tests {
    use super::*;
    use tr_core::tile::{FLAG_ACTIVE, Tile};

    /// 40x40 world with a stone floor at y = 20 — the core `edit.rs` fixture
    /// shape, deep enough inside the world for `CanPlace`'s 5-tile edge margin
    /// (`TileObject.cs:215`).
    fn object_world() -> World {
        let mut w = World::new("t".into(), 40, 40);
        w.frame_important = vec![false; td::TILE_COUNT];
        for x in 0..40 {
            let i = w.index(x, 20);
            w.tiles[i] = Tile {
                kind: 1,
                ..Default::default()
            };
            w.tiles[i].flags |= FLAG_ACTIVE;
        }
        w.compute_column_surface();
        w.compute_chunks();
        w
    }

    fn slot(kind: u16, count: u32) -> ItemSlot {
        ItemSlot {
            kind,
            name: "",
            count,
        }
    }

    // (a) the empty-slot/weapon gate shared by `use_tile_item` and `can_place`.
    #[test]
    fn item_gate_blocks_empty_slot_weapon_and_cactus() {
        // An empty slot never places, whatever the kind.
        assert!(item_gate_blocks(2, 0));
        // A weapon (shuriken, `ItemID.Shuriken` 42) never places.
        assert!(item_gate_blocks(42, 99));
        // The legacy cactus kind never places (`ItemID.Cactus` 80).
        assert!(item_gate_blocks(80, 99));
        // A stackable block item passes (dirt, `ItemID.DirtBlock` 2).
        assert!(!item_gate_blocks(2, 50));
        // The work bench item passes (`ItemID.WorkBench` 36).
        assert!(!item_gate_blocks(36, 1));
    }

    // (b) the registry-bypass condition of `use_tile_item`.
    #[test]
    fn registry_kinds_bypass_the_legacy_prechecks() {
        // The wooden set is registered (`def_for_tile`): the legacy
        // neighbour/overlap pre-checks must not gate it — the verdict belongs
        // to `TileObject.CanPlace` inside the dispatcher (`Player.cs:39772`).
        for tile in [10, 11, 14, 15, 18, 21] {
            assert!(!legacy_precheck_applies(tile), "tile {tile} registered");
        }
        // Unregistered kinds keep the legacy pre-checks.
        for tile in [0, 1, 4] {
            assert!(legacy_precheck_applies(tile), "tile {tile} legacy");
        }
        // The bypass keys on the *tile* kind the held item maps to: the work
        // bench item (36) places tile 18 and bypasses; dirt (2) places tile 0
        // and does not.
        assert_eq!(registry::place_tile(36), Some(18));
        assert!(!legacy_precheck_applies(registry::place_tile(36).unwrap()));
        assert_eq!(registry::place_tile(2), Some(0));
        assert!(legacy_precheck_applies(registry::place_tile(2).unwrap()));
    }

    // (c) the style/facing resolution `place_tile` feeds the dispatcher.
    #[test]
    fn style_and_facing_resolve_the_can_place_inputs() {
        // `Item.placeStyle` (0 for the whole wooden set, `Item.cs:173`) and
        // `Player.direction`; no player defaults to right (+1) — the same
        // default the replicated `apply_edit` route sends (Decision #7).
        assert_eq!(style_and_facing(36, Some(1)), (0, 1));
        assert_eq!(style_and_facing(36, Some(-1)), (0, -1));
        assert_eq!(style_and_facing(36, None), (0, 1));
        // An unregistered item id resolves like the registry accessor: style 0.
        assert_eq!(style_and_facing(9999, None), (0, 1));
    }

    // (d) item consumption is tied to a successful place (`Player.cs:40118-40126`).
    #[test]
    fn successful_place_consumes_one_item() {
        let mut w = object_world();
        let mut held = slot(36, 10);
        assert!(place_item_and_consume(&mut w, &mut held, 10, 19, 18, 0, 1));
        assert_eq!(held.count, 9);
        // The object is in the world: the 2x1 bench fills both cells.
        for x in [10, 11] {
            let t = w.tile(x, 19).unwrap();
            assert!(t.active() && t.kind == 18, "bench cell ({x}, 19)");
        }
    }

    #[test]
    fn refused_place_keeps_the_item_and_the_world() {
        let mut w = object_world();
        // A stone block occupies the bench's second cell: partial overlap.
        let i = w.index(11, 19);
        w.tiles[i] = Tile {
            kind: 1,
            ..Default::default()
        };
        w.tiles[i].flags |= FLAG_ACTIVE;
        let mut held = slot(36, 10);
        let before = w.revision;
        assert!(!place_item_and_consume(&mut w, &mut held, 10, 19, 18, 0, 1));
        assert_eq!(held.count, 10, "a refused placement keeps the item");
        assert_eq!(w.revision, before, "a refused placement writes nothing");
        assert!(!w.tile(10, 19).unwrap().active(), "origin cell untouched");
    }

    #[test]
    fn style_and_facing_flow_through_to_the_dispatcher_frames() {
        // The wrapper passes `style`/`dir` straight to `place_item_tile`:
        // placeStyle 1 shifts the bench one style column (36 px full width,
        // `TileObjectData.cs:2818-2825`), facing right picks the chair's
        // `PlaceRight` alternate (`TileObject.cs:209`).
        let mut w = object_world();
        let mut held = slot(36, 1);
        assert!(place_item_and_consume(&mut w, &mut held, 10, 19, 18, 1, 1));
        assert_eq!(w.tile(10, 19).unwrap().frame_x, 36);
        assert_eq!(w.tile(11, 19).unwrap().frame_x, 54);

        let mut w2 = object_world();
        assert!(place_item_and_consume(&mut w2, &mut held, 10, 19, 15, 0, 1));
        assert_eq!(w2.tile(10, 18).unwrap().frame_x, 18);
        let mut w3 = object_world();
        assert!(place_item_and_consume(
            &mut w3, &mut held, 10, 19, 15, 0, -1
        ));
        assert_eq!(w3.tile(10, 18).unwrap().frame_x, 0);
    }

    // ---- Phase 3 (DW-3.2): the break drop events ---------------------------

    #[test]
    fn test_DW_3_2_break_drop_events_legacy_tile_drops_at_the_hit_cell() {
        // Stone (tile 1) has no registry def: the per-cell drop table
        // (`KillTile_GetItemDrops`) maps it to the stone item at the hit
        // cell (`WorldGen.cs:63860-63864`).
        let kill = edit::TileKill {
            kind: 1,
            object_drops: vec![],
        };
        assert_eq!(break_drop_events(&kill, 7, 12), vec![(3, 7, 12)]);
    }

    #[test]
    fn test_DW_3_2_break_drop_events_object_cascades_resolve_items() {
        // Registry kinds drop nothing per-cell (`KillTile_GetItemDrops`'s
        // empty case group, `WorldGen.cs:65748-66091`): the item comes from
        // the cascade record only, mapped through `tile_drop`'s style-0
        // slice. The bench record carries the C# drop position
        // (origin x, trigger y); the open-door record maps to the
        // closed-door item (`GetDoorItem(0)` = 25).
        let kill = edit::TileKill {
            kind: 18,
            object_drops: vec![edit::ObjectDrop {
                tile_kind: 18,
                x: 10,
                y: 19,
            }],
        };
        assert_eq!(break_drop_events(&kill, 11, 19), vec![(36, 10, 19)]);

        let kill = edit::TileKill {
            kind: 1,
            object_drops: vec![edit::ObjectDrop {
                tile_kind: 11,
                x: 9,
                y: 18,
            }],
        };
        assert_eq!(
            break_drop_events(&kill, 9, 17),
            vec![(3, 9, 17), (25, 9, 18)],
            "legacy hit drop plus the chained open-door object drop"
        );
    }

    // ---- Phase 4 (DW-4.1/DW-4.3): the interaction hook ---------------------

    #[test]
    fn test_DW_4_1_interaction_events_map_results_to_cues_and_drops() {
        use edit::TileInteraction;
        // `None` = no interaction: the caller falls through to the place path
        // with no cue and no drops.
        assert!(interaction_events(TileInteraction::None).is_none());

        let (cue, events) = interaction_events(TileInteraction::DoorOpened {
            object_drops: vec![edit::ObjectDrop {
                tile_kind: 10,
                x: 10,
                y: 18,
            }],
        })
        .unwrap();
        assert_eq!(cue, Some("Door_Opened")); // PlaySound(8), WorldGen.cs:37732
        assert_eq!(events, vec![(25, 10, 18)]); // GetDoorItem(0) = 25

        let (cue, events) = interaction_events(TileInteraction::DoorClosed {
            object_drops: vec![],
        })
        .unwrap();
        assert_eq!(cue, Some("Door_Closed")); // PlaySound(9), WorldGen.cs:31906
        assert!(events.is_empty());

        // The blocked path carries cleanup drops with no cue — `PlaySound`
        // sits behind every failure return in the C# bodies.
        let (cue, events) = interaction_events(TileInteraction::DoorBlocked {
            object_drops: vec![edit::ObjectDrop {
                tile_kind: 14,
                x: 10,
                y: 17,
            }],
        })
        .unwrap();
        assert_eq!(cue, None);
        assert_eq!(events, vec![(32, 10, 17)]);
    }

    #[test]
    fn test_DW_4_3_cell_blocked_by_entity() {
        // The entity half of `Collision.EmptyTile(ignoreTiles: true)`
        // (`Collision.cs:1359-1378`): strict rectangle overlap against the
        // player and NPCs.
        let w = object_world();
        let mut player = Player::spawn(&w);
        player.x = 160.0;
        player.y = 304.0; // the 20x42 hitbox covers cell (10, 19)
        assert!(cell_blocked_by_entity(Some(&player), &[], 10, 19));
        assert!(!cell_blocked_by_entity(Some(&player), &[], 15, 19));
        assert!(!cell_blocked_by_entity(None, &[], 10, 19));

        // A slime spawned on the surface covers the cells its 30x24 body
        // intersects.
        let slime = Npc::spawn_slime(&w, 20).unwrap();
        assert!(cell_blocked_by_entity(
            None,
            std::slice::from_ref(&slime),
            20,
            19
        ));
        assert!(!cell_blocked_by_entity(
            None,
            std::slice::from_ref(&slime),
            5,
            19
        ));
    }
}
