//! Tile and wall editing (plan stage A4): the per-frame mining/placing loop,
//! the break/place helpers behind it and the replicated-edit applier the net
//! layer feeds.

use glam::Vec2;
use tr_core::tiledata as td;
use tr_core::{TILE_PX, World, edit};

use crate::app::App;
use crate::item::ItemSlot;
use tr_core::loot::DropRng;
use crate::npc::{Npc, NpcKind};
use crate::player::Player;
use crate::projectile::Projectile;
use crate::achievements;
use crate::registry;

use super::items::spawn_item_drop;

/// The `Player.ApplyItemAnimation` tail (`Player.cs:4449-4451`): the swing
/// carries the item's `useStyle` and the current aim angle — the per-style
/// body dispatch and the held-item draw read both.
pub(crate) fn start_swing_extras(app: &mut App, held_kind: u16) {
    app.swing_style = crate::item::use_style(held_kind);
    let (sw, sh) = app.screen_size();
    let mouse_world = Vec2::new(
        app.camera.x + (app.mouse_pos.x - sw * 0.5) / app.zoom,
        app.camera.y + (app.mouse_pos.y - sh * 0.5) / app.zoom,
    );
    let (cx, cy) = app
        .player
        .as_ref()
        .map(|p| p.center())
        .unwrap_or((app.camera.x, app.camera.y));
    app.swing_aim = (mouse_world - Vec2::new(cx, cy)).y.atan2((mouse_world - Vec2::new(cx, cy)).x);
}

pub(crate) fn update_editing(app: &mut App, dt: f32, screen: Vec2) {
    app.swing = (app.swing - dt).max(0.0);
    // `Player.itemRotation`'s per-tick update for the aimed styles (useStyle
    // 5 reads the mouse; `ItemCheck`'s shoot block, `Player.cs:47349-47376`).
    if app.swing > 0.0 && app.swing_style == 5 {
        let (sw, sh) = app.screen_size();
        let mouse_world = Vec2::new(
            app.camera.x + (app.mouse_pos.x - sw * 0.5) / app.zoom,
            app.camera.y + (app.mouse_pos.y - sh * 0.5) / app.zoom,
        );
        let (cx, cy) = app
            .player
            .as_ref()
            .map(|p| p.center())
            .unwrap_or((app.camera.x, app.camera.y));
        let d = mouse_world - Vec2::new(cx, cy);
        app.swing_aim = d.y.atan2(d.x);
    }
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
    // The boss spawners (`Player.ItemCheck_UseBossSpawners`,
    // `Player.cs:43742-43771`) fire before the world-editing gates — the C#
    // runs them in the item-use tick, not the cursor path; the eye arm takes
    // `app` wholesale (before the fn's `player` borrow).
    if app.mine_down && !app.inventory_open {
        let held = app.inventory[app.selected];
        if held.kind == 43 && held.count > 0 && use_suspicious_looking_eye(app) {
            return;
        }
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
    // `Player.ItemCheck`'s consumable arms: the food (buff 26/206/207,
    // gated by `CanUseItem`'s wellFed check `:14353`), the healing potions
    // (`ApplyPotionDelay` + `ApplyLifeAndOrMana`) and the magic weapons'
    // mana pay (`CheckMana`).
    if app.mine_down && held.count > 0 && !app.inventory_open {
        let def = registry::get(held.kind);
        if def.is_some_and(|d| {
            d.consumable && matches!(d.buff_type, 26 | 206 | 207)
        }) {
            let Some(def) = def else { unreachable!() };
            // The sim-context player borrow is immutable; the fed flag was
            // derived by this tick's scan, so the gate reads it directly.
            let can_eat = !player.well_fed;
            if can_eat {
                let time = if def.buff_time > 0 {
                    def.buff_time
                } else {
                    14400 // the 4-minute vanilla fallback
                };
                if let Some(p) = app.player.as_mut() {
                    p.buffs.add_buff(def.buff_type, time);
                    p.eat_food();
                }
                app.inventory[app.selected].count =
                    app.inventory[app.selected].count.saturating_sub(1);
                if app.inventory[app.selected].count == 0 {
                    app.inventory[app.selected].kind = 0;
                }
                app.swing_max = crate::item::use_animation_secs(held.kind);
                app.swing = app.swing_max;
                start_swing_extras(app, held.kind);
                app.attack_cd = app.swing_max;
                app.mine_target = None;
                app.mine_timer = 0.0;
                return;
            }
        }
        // The healing potions (`Player.cs:42756`'s `item.potion` arm +
        // `ApplyPotionDelay`/`ApplyLifeAndOrMana`): the Potion Sickness buff
        // 21 (3600 ticks, `Item.potionDelay`) gates the re-drink, the heal
        // clamps to max, the count decrements.
        let def = registry::get(held.kind);
        if def.is_some_and(|d| {
            d.potion && d.heal_life > 0
        }) {
            let Some(def) = def else { unreachable!() };
            let sick = player
                .buffs
                .find_buff_index(21)
                .is_some();
            if !sick {
                let heal = def.heal_life;
                if let Some(p) = app.player.as_mut() {
                    p.hp = (p.hp + heal).min(p.max_hp);
                    // `ApplyPotionDelay` (`Player.cs:51853`): the buff 21
                    // rides `Item.potionDelay` = 3600 (60 s).
                    p.buffs.add_buff(21, 3600);
                }
                app.inventory[app.selected].count =
                    app.inventory[app.selected].count.saturating_sub(1);
                if app.inventory[app.selected].count == 0 {
                    app.inventory[app.selected].kind = 0;
                }
                app.swing_max = crate::item::use_animation_secs(held.kind);
                app.swing = app.swing_max;
                start_swing_extras(app, held.kind);
                app.attack_cd = app.swing_max;
                app.mine_target = None;
                app.mine_timer = 0.0;
                return;
            }
        }
    }
    let item_crit = registry::get(held.kind).map(|d| d.crit).unwrap_or(0);
    if app.mine_down && held_weapon && !app.inventory_open {
        // Weapons use LMB toward the cursor instead of mining. The shuriken
        // is not `autoReuse` (`Item.cs:1869-1885`): one throw per press edge
        // (`Player.cs:42733` `controlUseItem && releaseUseItem && itemAnimation == 0`).
        let throw_ready = registry::get(held.kind).is_some_and(|d| d.auto_reuse)
            || (app.mine_released && !app.mine_prev_down);
        if app.attack_cd <= 0.0 && app.reuse_delay <= 0.0 && throw_ready {
            // `Player.CheckMana` (`Player.cs:52576`): the magic items pay
            // `item.mana * manaCost` (the multiplier is the 1f base — the
            // accessories fold out); a failed check blocks the use. The pay
            // resets the mana regen delay (`apply_mana_regen_delay`).
            let mana_cost = registry::get(held.kind).map(|d| d.mana).unwrap_or(0);
            // The mutable player borrow covers the whole throw (the mana
            // pay + the reads the block makes through it).
            let Some(p) = app.player.as_mut() else { unreachable!() };
            if mana_cost > 0 {
                // `Player.CheckMana`'s pay arm: a failed check blocks the use.
                if p.mana < mana_cost {
                    return;
                }
                p.mana -= mana_cost;
                crate::regen::apply_mana_regen_delay(
                    &mut p.regen,
                    p.mana_max,
                    p.mana,
                    false,
                );
            }
            // `SetItemAnimation`'s tail (`Player.cs:4496`): the reuse lock
            // rides the use start.
            if let Some(d) = registry::get(held.kind) {
                app.reuse_delay = d.reuse_delay as f32 / 60.0;
            }
            // `Player.ItemCheck_Shoot` (`Player.cs:47056+`): the generic
            // `Item.shoot` firing — the aim vector normalized to
            // `shootSpeed` (`:47274-47292`; the melee `speed /= meleeSpeed`
            // is ×1 at the port's default melee speed), the spawn at the
            // player center (`RotatedRelativePoint(MountedCenter)`), the
            // damage `weaponDamage` (the class multiplier) and the item's
            // knockback.
            let def = registry::get(held.kind);
            if let (Some(proj), Some(def)) = (registry::projectile(held.kind), def) {
                let (pcx, pcy) = p.center();
                let dir = (world_pos - Vec2::new(pcx, pcy)).normalize_or_zero();
                if dir != Vec2::ZERO {
                    let spawn = Vec2::new(pcx, pcy) + dir * 10.0;
                    let class = def.class;
                    let raw =
                        def.damage as f32 * p.weapon_damage_multiplier(class);
                    // The crit rolls at throw time (the class crit + the
                    // item's crit; `Projectile.Damage` consumes it on hit) —
                    // the item stream lives here, not in the projectile step.
                    let mut loot_rng = crate::world_sim::npcs::LootRng(&mut app.item_rng);
                    let crit =
                        tr_core::damage::crit_roll(&mut loot_rng, p.class_crit(class) + item_crit);
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
                    start_swing_extras(app, held.kind);
                    app.attack_cd = app.swing_max;
                }
            }
        }
        return;
    }
    if app.reuse_delay > 0.0 {
        app.reuse_delay = (app.reuse_delay - dt).max(0.0);
    }
    // Melee: LMB over an NPC within four tiles hits it instead of mining.
    if app.mine_down && app.attack_cd <= 0.0 && app.reuse_delay <= 0.0 && !app.inventory_open {
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
                // `Main.DamageVar(dmg, this.luck)` — the player-dealt roll
                // passes the raw value (`ProcessHitAgainstNPC`,
                // `Player.cs:44342`; damage taken negates it, `:31387`).
                let luck = player.luck();
                let mut loot_rng = crate::world_sim::npcs::LootRng(&mut app.item_rng);
                let crit =
                    tr_core::damage::crit_roll(&mut loot_rng, player.class_crit(class) + item_crit);
                let dmg = tr_core::damage::damage_var(&mut loot_rng, raw, luck);
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
                start_swing_extras(app, held.kind);
                app.attack_cd = app.swing_max;
                app.mine_target = None;
                app.mine_timer = 0.0;
                return;
            }
        }
        // The healing potions (`Player.cs:42756`'s `item.potion` arm +
        // `ApplyPotionDelay`/`ApplyLifeAndOrMana`): the Potion Sickness buff
        // 21 (3600 ticks, `Item.potionDelay`) gates the re-drink, the heal
        // clamps to max, the count decrements.
        let def = registry::get(held.kind);
        if def.is_some_and(|d| {
            d.potion && d.heal_life > 0
        }) {
            let Some(def) = def else { unreachable!() };
            let sick = player
                .buffs
                .find_buff_index(21)
                .is_some();
            if !sick {
                let heal = def.heal_life;
                if let Some(p) = app.player.as_mut() {
                    p.hp = (p.hp + heal).min(p.max_hp);
                    // `ApplyPotionDelay` (`Player.cs:51853`): the buff 21
                    // rides `Item.potionDelay` = 3600 (60 s).
                    p.buffs.add_buff(21, 3600);
                }
                app.inventory[app.selected].count =
                    app.inventory[app.selected].count.saturating_sub(1);
                if app.inventory[app.selected].count == 0 {
                    app.inventory[app.selected].kind = 0;
                }
                app.swing_max = crate::item::use_animation_secs(held.kind);
                app.swing = app.swing_max;
                start_swing_extras(app, held.kind);
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
    let held_wall = held.count > 0 && registry::place_wall(held.kind).is_some();

    // No `frame_important` gate on this branch — vanilla's LMB mining chain
    // has none: `tileFrameImportant` is read only by the placement/replacement
    // paths (`Player.cs:40019`, `:40608`), never by the pick tick
    // `ItemCheck_UseMiningTools_ActuallyUseMiningTool` → `PickTile`
    // (`Player.cs:45656`, `:53757`). The former cactus carve-out is gone with
    // it: cactus 80 is `tileAxe` in C# (`Main.cs:7210`), so vanilla LMB-pick
    // does not break it — and since the tool tiers landed on main
    // (`tr_core::tool`, merged 2026-11-04) the axe gate covers kind 80 the
    // same way, closing the TO-5-era pick-breakability deviation.
    if app.mine_down && tile.active() && !app.inventory_open {
        // The tool tiers (`GetPickaxeDamage`'s gates via `tr_core::tool`):
        // the axe tiles chop with an axe, everything else mines with a pick
        // whose power clears the tile's tier; the tier also paces the dig
        // (the baseline timings assume the copper pick's 35).
        let tool_pick = registry::get(held.kind).map(|d| d.pick).unwrap_or(0);
        let tool_axe = registry::get(held.kind).map(|d| d.axe).unwrap_or(0);
        let axe_tile = tr_core::tool::is_axe_tile(tile.kind);
        let below_surface = (ty as f64) > app.world.world_surface;
        let can_mine = if axe_tile {
            tool_axe > 0
        } else {
            tool_pick > 0 && tr_core::tool::pick_can_mine(tile.kind, tool_pick as i32, below_surface)
        };
        if !can_mine {
            app.mine_target = None;
            app.mine_timer = 0.0;
        } else {
            if app.swing <= 0.0 {
                app.swing_max = crate::item::use_animation_secs(held.kind);
                app.swing = app.swing_max;
                start_swing_extras(app, held.kind);
            }
            // `PickTile_DetermineDamage` (`Player.cs:53830-53833`): a
            // `CanKillTile` refusal zeroes the pick damage — the swing
            // plays, the accumulator never reaches 100. (The C# fail tick's
            // `KillTile(fail: true)` sound/dust has no per-tick path in the
            // port.) `break_tile` re-checks for the other break sources.
            let refused =
                !edit::can_kill_tile(&app.world, tx, ty, chest_holds_items(app));
            if refused {
                app.mine_target = None;
                app.mine_timer = 0.0;
            } else if app.mine_target == Some((tx, ty)) {
                app.mine_timer += dt;
            } else {
                app.mine_target = Some((tx, ty));
                app.mine_timer = 0.0;
            }
            let base = if tile.kind == 1 || (6..=9).contains(&tile.kind) {
                0.55
            } else {
                0.25
            };
            // `Player.pickSpeed`'s WellFed discount (`:11945`) — the lower
            // the faster; the port folds it into the dig time.
            let fed_speed = 1.0
                - app
                    .player
                    .as_ref()
                    .map(|p| p.pick_speed_bonus)
                    .unwrap_or(0.0);
            let hardness = if axe_tile {
                base * 7.0 / tool_axe.max(1) as f32
            } else {
                base * tr_core::tool::pick_speed_scale(tool_pick as i32) * fed_speed.max(0.25)
            };
            if app.mine_timer >= hardness {
                break_tile(app, tx, ty);
                app.mine_target = None;
                app.mine_timer = 0.0;
            }
        }
    } else if app.mine_down && !tile.active() && tile.wall != 0 && !app.inventory_open {
        // No block in the cell: mining it removes the exposed wall.
        if app.swing <= 0.0 {
            app.swing_max = crate::item::use_animation_secs(held.kind);
            app.swing = app.swing_max;
            start_swing_extras(app, held.kind);
        }
        if app.mine_target == Some((tx, ty)) {
            app.mine_timer += dt;
        } else {
            app.mine_target = Some((tx, ty));
            app.mine_timer = 0.0;
        }
        // The walls need a hammer (`Main.tileHammer`'s rule — the port
        // gates on the held hammer power alone) and the hammer paces the
        // dig (the baseline assumes the copper hammer's 35).
        let tool_hammer = registry::get(held.kind).map(|d| d.hammer).unwrap_or(0);
        if tool_hammer <= 0 {
            app.mine_target = None;
            app.mine_timer = 0.0;
        } else if tile.active() && tile.kind == 26 {
            // The demon/crimson altar (`KillTile`'s case 26): the hammer
            // smash runs the hardmode ore tier (`WorldGen.cs:48920+`) —
            // the hammer-power gate (≥ 80) and the WoF prerequisite are
            // the follow-ups; the port smashes in hardmode.
            if app.mine_timer >= 0.35 && app.world.hard_mode {
                app.mine_timer = 0.0;
                let altars = app.world.altars_smashed;
                let mut rng = app.growth_rng;
                rng.next_seed_in_place();
                let ore = tr_core::ores::smash_altar(
                    &mut app.world,
                    &mut app.ore_tiers,
                    altars,
                    &mut rng,
                );
                app.world.altars_smashed += 1;
                app.audio
                    .play_at("Dig_1", tx as f32 * TILE_PX, ty as f32 * TILE_PX, 1.0);
                app.request_edit(tx, ty, 2, 0);
                let _ = ore;
                app.world_key = None;
            }
        } else {
            let hardness = (if tile.wall == 1 { 0.7 } else { 0.35 })
                * 35.0
                / tool_hammer.max(1) as f32;
            if app.mine_timer >= hardness {
                break_wall(app, tx, ty);
                app.mine_target = None;
                app.mine_timer = 0.0;
            }
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
    // `Player.channel` (`Item.cs:52960-52965` sets it; `Player.cs:43576`'s
    // `if (!this.channel)` guards the animation decrement): while the
    // button is held the item's animation HOLDS instead of ending — the
    // channel persists until the release.
    let channels = timings.is_some_and(|d| d.channel) && app.mine_down && !held_weapon;
    if channels && app.swing <= 0.0 {
        // The hold: the animation stays at the last frame.
        app.swing = 0.2;
        app.attack_cd = app.attack_cd.max(0.2);
    }
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
    if app.place_down
        && !app.place_prev_down
        && !app.inventory_open
        && crate::world_sim::npcs::try_open_shop_at_cursor(app, world_pos)
    {
        return;
    }
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
        spawn_item_drop(app, id, 1, tx, ty);
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

/// `Chest.CanDestroyChest`'s item scan (`Chest.cs:609-623`) against the
/// app's in-memory chest store: any slot with a positive stack at the
/// anchor refuses the break. The C#'s `type > 0` half is implied — the
/// store only holds real stacks. The store stays empty until the
/// M3-CHEST-UI seam fills it, so the refusal passes today.
fn chest_holds_items(app: &App) -> impl Fn(i32, i32) -> bool + '_ {
    move |x, y| {
        app.world_chests
            .iter()
            .any(|c| c.tile == (x, y) && c.slots.iter().any(|&(_, stack)| stack > 0))
    }
}

pub(crate) fn break_tile(app: &mut App, tx: i32, ty: i32) {
    // The three `CanKillTile` refusals (`WorldGen.cs:62219-62326`) gate
    // every break source — the C# `KillTile` entry duplicates them through
    // `CheckTileBreakability` (the 1/2 flavours, `:63537-63545`) and
    // `Chest.DestroyChest` (:63695). Vanilla answers the pick with the fail
    // tick's sound/dust instead of the break; the port has no per-tick
    // path, so the refusal is silent.
    if !edit::can_kill_tile(&app.world, tx, ty, chest_holds_items(app)) {
        return;
    }
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
    // The hit cell's frames — the C# `tileCache` of `KillTile_GetItemDrops`
    // (`WorldGen.cs:63964`): the per-cell drops read them after the tile is
    // already cleared (the planter-box style `frameY / 18`, the herb style
    // `frameX / 18`).
    let hit = app.world.tile(tx, ty).filter(|t| t.active()).map(|t| HitCell {
        x: tx,
        y: ty,
        kind: t.kind,
        frame_x: t.frame_x,
        frame_y: t.frame_y,
    });
    let Some(kill) = edit::kill_tile(&mut app.world, tx, ty, false) else {
        return;
    };
    // `CheckPot`'s break sound by style (:56856-56867, `pot_break_sound`) —
    // SoundID 4/6/13 have no ported cues, so the generic break cue stands in
    // at the event cell (ANALYSIS entry, phase 5).
    for event in &kill.events {
        app.audio
            .play_at("Dig_1", event.x as f32 * TILE_PX, event.y as f32 * TILE_PX, 1.0);
    }
    // The breaking player's staff bonus (`GetPlayerForTile(x, y)
    // .staffOfRegrowthBonus`, `WorldGen.cs:65689` — the flag the placement
    // path sets for held 213/5295): the port reads the held slot at the
    // break moment.
    let staff_bonus = staff_of_regrowth(app.inventory[app.selected].kind);
    let ctx = drop_ctx(app);
    let (drops, portals) = {
        let mut main_rng = super::npcs::LootRng(&mut app.item_rng);
        break_drop_events(&kill, &app.world, hit, staff_bonus, &mut main_rng, &ctx)
    };
    for (id, count, x, y) in drops {
        spawn_item_drop(app, id, count, x, y);
        app.audio
            .play_at("Dig_1", x as f32 * TILE_PX, y as f32 * TILE_PX, 1.0);
    }
    // The Coin Portal projectile of the pot loot (:57176-57182); the port
    // has no 518 family yet — `spawn_with_data` declines it and the break
    // still returns like the C# (recorded deviation, phase 5).
    for (px, py) in portals {
        let _ = app.projectiles.spawn_with_data(
            518,
            px as f32 * TILE_PX + 16.0,
            py as f32 * TILE_PX + 16.0,
            0.0,
            -12.0,
            0,
            0.0,
        );
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

/// The world/player inputs the drop resolvers read. Every C# flag with no
/// port state defaults to `false` (`Main.expertMode/dontStarveWorld` — the
/// world modes are not persisted yet; remix/getGood/tenth/vampire never reach
/// the resolvers). Built once per break from `App` ([`drop_ctx`]); the
/// `Default` zeroes serve the record-only test paths.
#[derive(Default)]
pub(crate) struct DropCtx {
    /// `Main.dayTime/moonPhase/bloodMoon/time` (`World`) and
    /// `Main.raining/cloudAlpha` (`Rain`).
    pub day_time: bool,
    pub moon_phase: u8,
    pub blood_moon: bool,
    pub time: f64,
    pub raining: bool,
    pub cloud_alpha: f32,
    /// `Main.worldSurface/rockLayer/UnderworldLayer/maxTilesY/hardMode`
    /// (`Main.cs:2790`: `UnderworldLayer = maxTilesY - 200`).
    pub world_surface: f64,
    pub rock_layer: f64,
    pub underworld_layer: i32,
    pub max_tiles_y: i32,
    pub hard_mode: bool,
    /// `Main.expertMode` / `Main.dontStarveWorld` — no port state (tests flip
    /// them; ANALYSIS entry, phase 5).
    pub expert_mode: bool,
    pub dont_starve: bool,
    /// The closest player (`Player.FindClosest` — the local one): the hearts
    /// gate `statLife < statLifeMax2` (:57512) and the torch-stack scan
    /// (:57482-57497, capped at the 20/30 threshold the C# loop breaks at).
    /// No player (headless shots) reads full life + 0 torches.
    pub player_hurt: bool,
    pub torches: i32,
    /// `player2.Zone*` — the scene metrics refresh at the player centre;
    /// `ZoneGlowshroom` has no port metric (false, phase-5 ANALYSIS).
    pub zone_hallow: bool,
    pub zone_crimson: bool,
    pub zone_corrupt: bool,
    pub zone_glowshroom: bool,
}

/// Builds [`DropCtx`] from the app state at the break moment.
fn drop_ctx(app: &App) -> DropCtx {
    let (player_hurt, torches) = match &app.player {
        Some(p) => (
            p.hp < p.max_hp,
            app.inventory
                .iter()
                .filter(|slot| {
                    slot.count > 0
                        && registry::place_tile(slot.kind)
                            .is_some_and(|tile| td::SET_TORCHES.has(tile))
                })
                .map(|slot| slot.count as i32)
                .sum::<i32>()
                .min(20),
        ),
        None => (false, 0),
    };
    DropCtx {
        day_time: app.world.day_time,
        moon_phase: app.world.moon_phase,
        blood_moon: app.world.blood_moon,
        time: app.world.time,
        raining: app.rain.is_raining(),
        cloud_alpha: app.rain.visual_alpha(),
        world_surface: app.world.world_surface,
        rock_layer: app.world.rock_layer,
        underworld_layer: app.world.height - 200,
        max_tiles_y: app.world.height,
        hard_mode: app.world.hard_mode,
        expert_mode: false,
        dont_starve: false,
        player_hurt,
        torches,
        zone_hallow: app.scene_metrics.zone_hallow,
        zone_crimson: app.scene_metrics.zone_crimson,
        zone_corrupt: app.scene_metrics.zone_corrupt,
        zone_glowshroom: false,
    }
}

/// The hit cell's drop-relevant state, captured before the kill clears the
/// tile (the C# `tileCache` of `KillTile_GetItemDrops`, `WorldGen.cs:63964`).
struct HitCell {
    x: i32,
    y: i32,
    kind: u16,
    frame_x: i16,
    frame_y: i16,
}

/// The item drops of one `kill_tile` result: the hit cell's per-cell
/// `KillTile_DropItems` rolls (`WorldGen.cs:63860-63864`), the `Check*`
/// `Item.NewItem` records (:86508/:49424/:43494/:47121/:57972), every pot
/// event's `SpawnThingsFromPot` loot (:57011-57014), every cascade-killed
/// herb's rolls (:65674-65698) and every 2x1 pile's loot block
/// (:47130-47245). `(item, stack, x, y)` in spawn order, plus the Coin
/// Portal projectile requests. Free of `App` — the RNG streams come in as
/// parameters (`genRand` = the seeded `FastRandom`, `Main.rand` = the
/// [`super::npcs::LootRng`] over the app's `item_rng`, the owner's seam).
fn break_drop_events(
    kill: &edit::TileKill,
    world: &World,
    hit: Option<HitCell>,
    staff_of_regrowth: bool,
    main_rng: &mut super::npcs::LootRng<'_>,
    ctx: &DropCtx,
) -> (Vec<(u16, u32, i32, i32)>, Vec<(i32, i32)>) {
    let mut events = Vec::new();
    // The hit cell's per-cell drop (`KillTile_DropItems` for the dying cell —
    // the C# `tileCache` carries the frames; captured pre-kill in
    // [`break_tile`]).
    if let Some(hit) = &hit {
        events.extend(cell_drops(hit, staff_of_regrowth, main_rng, ctx));
    }
    // The cascade's `Check*` records: the check's own `Item.NewItem` (the
    // sunflower's 63) or the object's item through `registry::tile_drop`.
    for drop in &kill.object_drops {
        let id = drop
            .item
            .map(|item| item as u16)
            .or_else(|| registry::tile_drop(drop.tile_kind, 0));
        if let Some(id) = id {
            events.push((id, 1, drop.x, drop.y));
        }
    }
    // `SpawnThingsFromPot` per pot event: one `item_rng` throw seeds the
    // `genRand` stream (the owner's contract), `Main.rand` rolls consume the
    // stream directly, in the C# order. The Coin Portal roll returns a
    // projectile request instead of loot.
    let mut portals = Vec::new();
    for event in &kill.events {
        let cell = world.tile(event.x, event.y);
        let liquid = cell.map(|t| t.liquid).unwrap_or(0);
        let wall = cell.map(|t| t.wall).unwrap_or(0);
        let mut gen_stream =
            tr_core::tile_light::FastRandom::new(crate::app::next_rand(main_rng.0));
        let (loot, portal) = spawn_things_from_pot(
            ctx,
            event.style,
            event.x,
            event.y,
            liquid,
            wall,
            &mut gen_stream,
            main_rng,
        );
        if portal {
            // `Projectile.NewProjectile(…, i * 16 + 16, j * 16 + 16, 0f,
            // -12f, 518, 0, 0f, Main.myPlayer)` (:57176-57182).
            portals.push((event.x, event.y));
        }
        for (id, count) in loot {
            events.push((id, count.max(1) as u32, event.x, event.y));
        }
    }
    // The cascade-killed herb cells: the per-cell herb rolls (`Main.rand`
    // only — no `genRand` rolls, no seed throw).
    for herb in &kill.herb_events {
        for (id, count) in herb_drops(herb.kind, herb.style, herb.y, staff_of_regrowth, ctx, main_rng)
        {
            events.push((id, count.max(1) as u32, herb.x, herb.y));
        }
    }
    // The broken piles' loot blocks: 185's 2x1 coin/ore table on the seeded
    // stream (`genRand` counts, `WorldGen.cs:47142-47245`), 187's
    // sword-in-stone roll on the item stream (`Main.rand.Next(30)`,
    // `:49828-49839`).
    for pile in &kill.pile_events {
        if pile.kind == 187 {
            if let Some(id) = sword_pile_roll(pile.frame_x, main_rng) {
                events.push((id, 1, pile.x, pile.y));
            }
            continue;
        }
        let mut gen_stream =
            tr_core::tile_light::FastRandom::new(crate::app::next_rand(main_rng.0));
        for (id, count) in pile_loot(pile.frame_x, &mut gen_stream) {
            events.push((id, count.max(1) as u32, pile.x, pile.y));
        }
    }
    (events, portals)
}

/// `Check3x2`'s `case 187` (`WorldGen.cs:49828-49839`): only the
/// sword-in-stone styles (origin `frameX` 918..=970) drop anything —
/// `Main.rand.Next(30) == 0` mints the Terragrim 4144, otherwise the
/// Enchanted Sword 989. Every other 187 frame sits in the empty
/// `GetItemDrops` group (:65759-65815).
fn sword_pile_roll(
    frame_x: i32,
    rng: &mut super::npcs::LootRng<'_>,
) -> Option<u16> {
    if !(918..=970).contains(&frame_x) {
        return None;
    }
    Some(if rng.next_range(0, 30) == 0 { 4144 } else { 989 })
}

/// The per-cell slice of `KillTile_GetItemDrops` (`WorldGen.cs:63964-66092`)
/// for the dying cell: 78 → 222 (:64711-64713), 380 → `3215 + frameY / 18`
/// (:65211-65213), the herb stages 83/84 → the RNG rolls (:65674-65698);
/// 27/28/82 and the piles are the empty case group (:65759-65815) — their
/// loot is the cascade record / the pot event — and registered furniture
/// drops through its `Check*` record only (no per-cell double spawn). The
/// unregistered legacy kinds keep the `registry::tile_drop` fallback (dirt →
/// 2, stone → 3, the cactus arm).
fn cell_drops(
    hit: &HitCell,
    staff_of_regrowth: bool,
    main_rng: &mut super::npcs::LootRng<'_>,
    ctx: &DropCtx,
) -> Vec<(u16, u32, i32, i32)> {
    let (x, y) = (hit.x, hit.y);
    let rolls: Vec<(u16, i32)> = match hit.kind {
        78 => vec![(222, 1)],
        380 => vec![((3215 + i32::from(hit.frame_y) / 18) as u16, 1)],
        83 | 84 => herb_drops(
            hit.kind,
            i32::from(hit.frame_x) / 18,
            y,
            staff_of_regrowth,
            ctx,
            main_rng,
        ),
        _ if tr_core::tileobject::def_for_tile(hit.kind).is_some() => Vec::new(),
        _ => registry::tile_drop(hit.kind, hit.frame_y)
            .into_iter()
            .map(|id| (id, 1))
            .collect(),
    };
    rolls
        .into_iter()
        .map(|(id, count)| (id, count.max(1) as u32, x, y))
        .collect()
}

/// `IsAlchemyPlantHarvestable` (`WorldGen.cs:66143-66174`): the per-style
/// bloom window — 0 Daybloom by day, 1 Moonglow by night, 3 Deathweed on a
/// blood moon (or the new moon), 4 Waterleaf in the rain (`Main.cloudAlpha`
/// included), 5 Fireblossom past 40500 ticks unless it rains while shallow.
/// The `remixWorld` branch has no port state (skipped — ANALYSIS, phase 5).
fn is_alchemy_plant_harvestable(style: i32, y: i32, ctx: &DropCtx) -> bool {
    if style == 0 && ctx.day_time {
        return true;
    }
    if style == 1 && !ctx.day_time {
        return true;
    }
    if style == 3 && !ctx.day_time && (ctx.blood_moon || ctx.moon_phase == 0) {
        return true;
    }
    if style == 4 && (ctx.raining || ctx.cloud_alpha > 0.0) {
        return true;
    }
    if style == 5 {
        let deep = y as f64 > ctx.world_surface;
        // `(!Main.raining | flag) && Main.time > 40500.0` — the C# bitwise
        // `|` on bools; both operands are side-effect free.
        if (!ctx.raining || deep) && ctx.time > 40500.0 {
            return true;
        }
    }
    false
}

/// `IsHarvestableHerbWithSeed` (`WorldGen.cs:66118-66141`): the immature 82
/// never seeds; 84 always; 83 only inside its bloom window.
fn is_harvestable_herb_with_seed(kind: u16, style: i32, y: i32, ctx: &DropCtx) -> bool {
    match kind {
        84 => true,
        83 => is_alchemy_plant_harvestable(style, y, ctx),
        _ => false,
    }
}

/// The per-cell herb rolls of `KillTile_GetItemDrops`'s `case 83: case 84:`
/// arm (`WorldGen.cs:65674-65698`): the herb `313 + num` (6 → 2358) always
/// drops (stack 1, `1..3` with the Staff of Regrowth); the seeds `307 + num`
/// (6 → 2357) drop `1..6` with the staff, else `1..4` when
/// `IsHarvestableHerbWithSeed`, else never. All rolls are `Main.rand`
/// (`Main.cs` drops carry no `genRand` rolls) — the `item_rng` stream.
fn herb_drops(
    kind: u16,
    style: i32,
    y: i32,
    staff_of_regrowth: bool,
    ctx: &DropCtx,
    main_rng: &mut super::npcs::LootRng<'_>,
) -> Vec<(u16, i32)> {
    // The case labels are 83/84 only — the immature 82 sits in the empty
    // case group (:65759-65815) and drops nothing.
    if kind == 82 {
        return Vec::new();
    }
    let (mut herb_id, mut seed_id) = (313 + style, 307 + style);
    if style == 6 {
        herb_id = 2358;
        seed_id = 2357;
    }
    let harvestable = is_harvestable_herb_with_seed(kind, style, y, ctx);
    let mut drops = Vec::with_capacity(2);
    if staff_of_regrowth {
        // `dropItemStack = Main.rand.Next(1, 3); secondaryItemStack =
        // Main.rand.Next(1, 6);` — the roll order is the C#'s.
        let stack = main_rng.next_range(1, 3);
        drops.push((herb_id as u16, stack));
        let seeds = main_rng.next_range(1, 6);
        drops.push((seed_id as u16, seeds));
    } else if harvestable {
        drops.push((herb_id as u16, 1));
        let seeds = main_rng.next_range(1, 4);
        drops.push((seed_id as u16, seeds));
    } else {
        drops.push((herb_id as u16, 1));
    }
    drops
}

/// `WorldGen.GetFruitForPot` (`WorldGen.cs:57018-57087`): the dontStarve
/// fruit pairs by pot style, all on the `genRand` stream. (The `InWorld`
/// head guard never binds — the pot cell is inside the cascade's 5-tile
/// margin.)
fn get_fruit_for_pot(pot_style: i32, gen_rand: &mut tr_core::tile_light::FastRandom) -> u16 {
    let pick = |gen_rand: &mut tr_core::tile_light::FastRandom, a: u16, b: u16| {
        if gen_rand.next_range(0, 2) == 0 {
            a
        } else {
            b
        }
    };
    if (4..=6).contains(&pot_style) {
        return pick(gen_rand, 4286, 4295);
    }
    if (7..=9).contains(&pot_style) || (28..=30).contains(&pot_style) {
        return pick(gen_rand, 4294, 4292);
    }
    if ((25..=27).contains(&pot_style)) || ((34..=36).contains(&pot_style)) {
        return pick(gen_rand, 4283, 4287);
    }
    if (16..=18).contains(&pot_style) {
        return pick(gen_rand, 4284, 4289);
    }
    if (22..=24).contains(&pot_style) {
        return pick(gen_rand, 4296, 4285);
    }
    if (13..=15).contains(&pot_style) {
        return pick(gen_rand, 5277, 5278);
    }
    match gen_rand.next_range(0, 5) {
        0 => 4009,
        1 => 4293,
        2 => 4282,
        3 => 4290,
        _ => 4291,
    }
}

/// The style multiplier table of `SpawnThingsFromPot`'s head
/// (`WorldGen.cs:57099-57159`): 0-3 neutral, then the per-band factors (the
/// hardmode band 28-30 only raises it in hardmode), folded by
/// `num = (num * 2 + 1) / 3` (:57159).
fn pot_style_multiplier(style: i32, hard_mode: bool) -> f32 {
    let mut num: f32 = 1.0;
    if (4..=6).contains(&style) {
        num = 1.25;
    } else if (7..=9).contains(&style) {
        num = 1.75;
    } else if (10..=12).contains(&style) {
        num = 1.9;
    } else if (13..=15).contains(&style) {
        num = 2.1;
    } else if (16..=18).contains(&style) {
        num = 1.6;
    } else if (19..=21).contains(&style) {
        num = 3.5;
    } else if (22..=24).contains(&style) {
        num = 1.6;
    } else if (25..=27).contains(&style) {
        num = 10.0;
    } else if (28..=30).contains(&style) && hard_mode {
        num = 4.0;
    } else if (31..=33).contains(&style) {
        num = 2.0;
    } else if (34..=36).contains(&style) {
        num = 1.25;
    }
    (num * 2.0 + 1.0) / 3.0
}

/// `int range = (int)(500f / ((num + 1f) / 2f))` (`WorldGen.cs:57171`) — the
/// Coin Portal's luck range (`Player.GetClosestRollLuck`, :57176).
fn pot_luck_range(num: f32) -> i32 {
    (500.0f32 / ((num + 1.0) / 2.0)) as i32
}

/// `WorldGen.SpawnThingsFromPot` (`WorldGen.cs:57089-57811`), the full roll
/// order. `gen_rand` is the event's `genRand` stream (`FastRandom` seeded
/// from one `item_rng` throw, the owner's contract), `main_rng` the
/// `Main.rand` stream (the app `item_rng`). `liquid`/`wall` are the pot
/// origin cell's post-kill state (KillTile keeps liquid and wall) for the
/// glowstick branch and the dungeon key. Returns the loot plus the Coin
/// Portal flag (the caller spawns the 518 projectile).
///
/// The special-world branches skip WITHOUT consuming rolls (C# `&&`
/// short-circuits): the Beenade (tenth anniversary :57167), the dontStarve
/// fruit (behind `ctx.dont_starve`), the getGoodWorld bomb, every
/// remixWorld branch and the netMode-2 wormhole. The boss-downed `×1.1`
/// flags (:57706-57750) have no port state — all false.
fn spawn_things_from_pot(
    ctx: &DropCtx,
    style: i32,
    _x: i32,
    y: i32,
    liquid: u8,
    wall: u16,
    gen_rand: &mut tr_core::tile_light::FastRandom,
    main_rng: &mut super::npcs::LootRng<'_>,
) -> (Vec<(u16, i32)>, bool) {
    // :57090-57095 — the depth flags (`flag` caves, `flag2` above the
    // underworld; the remix rewrites have no port state).
    let flag = (y as f64) < ctx.rock_layer;
    let flag2 = y < ctx.underworld_layer;
    let flag4 = (34..=36).contains(&style);
    // :57099-57171 — the style multiplier and the luck range.
    let range = pot_luck_range(pot_style_multiplier(style, ctx.hard_mode));
    // :57176-57182 — the Coin Portal (`Player.GetClosestRollLuck` →
    // `Luck.RollLuck(luck, range)` with the port's luck 0 = one
    // `Main.rand.Next(range)`, `Luck.cs:5-12`, `Player.cs:18236-18239`);
    // the caller spawns the projectile (`NewProjectile(…, 518, …)`).
    if main_rng.next_range(0, range) == 0 {
        return (Vec::new(), true);
    }
    // :57184-57182 — the dungeon key: `genRand.Next(35)` consumes first, the
    // dungeon-wall/depth checks judge after.
    if gen_rand.next_range(0, 35) == 0
        && tr_core::scene::is_dungeon_wall(wall)
        && y as f64 > ctx.world_surface
    {
        return (vec![(327, 1)], false);
    }
    // :57189-57192 — the dontStarve fruit (`flag3` = below surface +
    // `Main.dontStarveWorld`).
    if ctx.dont_starve && y as f64 > ctx.world_surface && gen_rand.next_range(0, 20) == 0 {
        let stack = gen_rand.next_range(1, 3);
        return (vec![(get_fruit_for_pot(style, gen_rand), stack)], false);
    }
    // :57194-57199 (getGoodWorld bomb), :57200-57241 (remix slimes/stars),
    // :57243-57247 (remix center rope) — skipped, no port state.
    // :57248-57462 — the potion table: `genRand.Next(45) == 0 || (
    // Main.rand.Next(45) == 0 && expertMode)` — the `Main.rand` roll is
    // consumed whenever the `genRand` roll misses, expert or not.
    if gen_rand.next_range(0, 45) == 0 || (main_rng.next_range(0, 45) == 0 && ctx.expert_mode) {
        return (potion_table(ctx, y, flag, flag2, gen_rand), false);
    }
    // :57464-57466 — the netMode-2 wormhole potion: skipped (single player).
    // :57469-57475 — `num11 = Main.rand.Next(7)`, expert shifts it down one
    // (no roll), and the full-torch inventory scan only flips `flag5`.
    let mut num11 = main_rng.next_range(0, 7);
    if ctx.expert_mode {
        num11 -= 1;
    }
    let flag5 = ctx.torches < 20;
    // :57498-57516 — the hearts (only while hurt).
    if num11 == 0 && ctx.player_hurt {
        let mut drops = vec![(58, 1)];
        if main_rng.next_range(0, 2) == 0 {
            drops.push((58, 1));
        }
        if ctx.expert_mode {
            if main_rng.next_range(0, 2) == 0 {
                drops.push((58, 1));
            }
            if main_rng.next_range(0, 2) == 0 {
                drops.push((58, 1));
            }
        }
        return (drops, false);
    }
    // :57518-57523 — the vampire-seed wood: skipped (no port state).
    // :57531-57591 — the torches (num11 == 1, or 0 while the player is below
    // the torch threshold).
    if num11 == 1 || (num11 == 0 && flag5) {
        return (vec![torch_drop(ctx, style, liquid, main_rng)], false);
    }
    // :57590-57607 — the arrows and the healing potions.
    if num11 == 2 {
        let stack = main_rng.next_range(10, 21);
        let mut arrow = 40;
        if flag && gen_rand.next_range(0, 2) == 0 {
            arrow = if !ctx.hard_mode { 42 } else { 168 };
        }
        if y > ctx.underworld_layer {
            arrow = 265;
        } else if ctx.hard_mode {
            // `Main.rand.Next(2) != 0 ? 47 : (SavedOreTiers.Silver != 168 ?
            // 278 : 4915)` — hardmode ore tiers are not ported.
            arrow = if main_rng.next_range(0, 2) != 0 {
                47
            } else {
                278
            };
        }
        return (vec![(arrow, stack)], false);
    }
    if num11 == 3 {
        let potion = if y > ctx.underworld_layer || ctx.hard_mode { 188 } else { 28 };
        let mut count = 1;
        if ctx.expert_mode && main_rng.next_range(0, 3) != 0 {
            count += 1;
        }
        return (vec![(potion, count)], false);
    }
    // :57608-57643 — the bombs (styles 34-36 anywhere above the underworld,
    // otherwise the deep cells); a miss falls through to the rope/coins.
    if num11 == 4 && (flag4 || flag2) {
        let bomb = if flag4 { 4423 } else { 166 };
        let mut count = main_rng.next_range(0, 4) + 1;
        if ctx.expert_mode {
            count += main_rng.next_range(0, 4);
        }
        return (vec![(bomb, count)], false);
    }
    // :57644-57647 — the pre-hardmode underground rope.
    if (num11 == 4 || num11 == 5) && y < ctx.underworld_layer && !ctx.hard_mode {
        return (vec![(965, main_rng.next_range(20, 41))], false);
    }
    // :57649-57810 — the coins.
    (
        coin_drops(
            ctx,
            pot_style_multiplier(style, ctx.hard_mode),
            y,
            flag,
            gen_rand,
            main_rng,
        ),
        false,
    )
}

/// The potion table of [`spawn_things_from_pot`] (`WorldGen.cs:57249-57461`):
/// one `genRand` pick per depth band, the longer-than-`else` arms (2322/
/// 2329 rolls) run for every `num >= 7` beside their own item.
fn potion_table(
    ctx: &DropCtx,
    y: i32,
    flag: bool,
    flag2: bool,
    gen_rand: &mut tr_core::tile_light::FastRandom,
) -> Vec<(u16, i32)> {
    let mut drops = Vec::with_capacity(2);
    if (y as f64) < ctx.world_surface {
        // :57250-57285 — the surface band.
        let num7 = gen_rand.next_range(0, 10);
        drops.push((
            match num7 {
                0 => 292,
                1 => 298,
                2 => 299,
                3 => 290,
                4 => 2322,
                5 => 2324,
                6 => 2325,
                _ => 2350,
            },
            1,
        ));
        if num7 >= 7 {
            let stack = gen_rand.next_range(1, 3);
            drops.push((2350, stack));
        }
    } else if flag {
        // :57286-57329 — the cave band.
        let num8 = gen_rand.next_range(0, 11);
        drops.push((
            match num8 {
                0 => 289,
                1 => 298,
                2 => 299,
                3 => 290,
                4 => 303,
                5 => 291,
                6 => 304,
                7 => 2322,
                8 => 2329,
                _ => 2350,
            },
            1,
        ));
        if num8 >= 7 {
            let stack = gen_rand.next_range(1, 3);
            drops.push((2350, stack));
        }
    } else if flag2 {
        // :57330-57397 — above the underworld.
        let num9 = gen_rand.next_range(0, 15);
        drops.push((
            match num9 {
                0 => 296,
                1 => 295,
                2 => 299,
                3 => 302,
                4 => 303,
                5 => 305,
                6 => 301,
                7 => 302,
                8 => 297,
                9 => 304,
                10 => 2322,
                11 => 2323,
                12 => 2327,
                13 => 2329,
                _ => 2350,
            },
            1,
        ));
        if num9 >= 7 {
            let stack = gen_rand.next_range(1, 3);
            drops.push((2350, stack));
        }
        if gen_rand.next_range(0, 15) == 0 {
            drops.push((4870, 1));
        }
    } else {
        // :57398-57461 — the deep band.
        let num10 = gen_rand.next_range(0, 14);
        drops.push((
            match num10 {
                0 => 296,
                1 => 295,
                2 => 293,
                3 => 288,
                4 => 294,
                5 => 297,
                6 => 304,
                7 => 305,
                8 => 301,
                9 => 302,
                10 => 288,
                11 => 300,
                12 => 2323,
                13 => 2326,
                _ => 300,
            },
            1,
        ));
        if gen_rand.next_range(0, 5) == 0 {
            drops.push((4870, 1));
        }
    }
    drops
}

/// The torch drop of [`spawn_things_from_pot`] (`WorldGen.cs:57532-57591`):
/// the base count plus the expert/vampire/biome bonuses — the first true
/// biome/style arm consumes its `Main.rand.Next(2, 7)` — then the liquid
/// swap (glowstick in water, the desert's 286).
fn torch_drop(
    ctx: &DropCtx,
    style: i32,
    liquid: u8,
    main_rng: &mut super::npcs::LootRng<'_>,
) -> (u16, i32) {
    let mut num14 = main_rng.next_range(2, 7);
    if ctx.expert_mode {
        num14 += main_rng.next_range(1, 7);
    }
    let mut torch = 8;
    let mut glowstick = 282;
    if ctx.zone_hallow {
        num14 += main_rng.next_range(2, 7);
        torch = 4387;
    } else if (22..=24).contains(&style) || ctx.zone_crimson {
        num14 += main_rng.next_range(2, 7);
        torch = 4386;
    } else if (16..=18).contains(&style) || ctx.zone_corrupt {
        num14 += main_rng.next_range(2, 7);
        torch = 4385;
    } else if (7..=9).contains(&style) {
        num14 = ((num14 + main_rng.next_range(2, 7)) as f32 * 1.5) as i32;
        torch = 4388;
    } else if (4..=6).contains(&style) {
        torch = 974;
        glowstick = 286;
    } else if (34..=36).contains(&style) {
        num14 += main_rng.next_range(2, 7);
        torch = 4383;
    } else if ctx.zone_glowshroom {
        num14 += main_rng.next_range(2, 7);
        torch = 5293;
    }
    if liquid > 0 {
        (glowstick, num14)
    } else {
        (torch, num14)
    }
}

/// The coin tail of [`spawn_things_from_pot`] (`WorldGen.cs:57649-57810`):
/// the `genRand` base, the depth/roll/expert/style multipliers and the
/// denomination split. The twelve boss-downed `×1.1` flags have no port
/// state (all false).
fn coin_drops(
    ctx: &DropCtx,
    num: f32,
    y: i32,
    flag: bool,
    gen_rand: &mut tr_core::tile_light::FastRandom,
    main_rng: &mut super::npcs::LootRng<'_>,
) -> Vec<(u16, i32)> {
    let mut num17: f32 = 200.0 + gen_rand.next_range(-100, 101) as f32;
    if (y as f64) < ctx.world_surface {
        num17 *= 0.5;
    } else if flag {
        num17 *= 0.75;
    } else if y > ctx.max_tiles_y - 250 {
        num17 *= 1.25;
    }
    num17 *= 1.0 + main_rng.next_range(-20, 21) as f32 * 0.01;
    if main_rng.next_range(0, 4) == 0 {
        num17 *= 1.0 + main_rng.next_range(5, 11) as f32 * 0.01;
    }
    if main_rng.next_range(0, 8) == 0 {
        num17 *= 1.0 + main_rng.next_range(10, 21) as f32 * 0.01;
    }
    if main_rng.next_range(0, 12) == 0 {
        num17 *= 1.0 + main_rng.next_range(20, 41) as f32 * 0.01;
    }
    if main_rng.next_range(0, 16) == 0 {
        num17 *= 1.0 + main_rng.next_range(40, 81) as f32 * 0.01;
    }
    if main_rng.next_range(0, 20) == 0 {
        num17 *= 1.0 + main_rng.next_range(50, 101) as f32 * 0.01;
    }
    if ctx.expert_mode {
        num17 *= 2.5;
        if main_rng.next_range(0, 2) == 0 {
            num17 *= 1.25;
        }
        if main_rng.next_range(0, 3) == 0 {
            num17 *= 1.5;
        }
        if main_rng.next_range(0, 4) == 0 {
            num17 *= 1.75;
        }
    }
    num17 *= num;
    split_coins(num17, main_rng)
}

/// The denomination split that closes `SpawnThingsFromPot`
/// (`WorldGen.cs:57752-57810`): platinum/gold/silver/copper with the `> 50`
/// shrinks — the gate roll is consumed only above 50 (C# `&&`), the copper's
/// second divisor is `Next(4) + 1`.
fn split_coins(
    mut num17: f32,
    main_rng: &mut super::npcs::LootRng<'_>,
) -> Vec<(u16, i32)> {
    let mut drops = Vec::new();
    while (num17 as i32) > 0 {
        // The >50 shrinks: `if (num > 50 && Main.rand.Next(2) == 0)` — the
        // gate roll is consumed only above 50 (C# `&&`).
        if num17 > 1_000_000.0 {
            let mut n = (num17 / 1_000_000.0) as i32;
            if n > 50 && main_rng.next_range(0, 2) == 0 {
                n /= main_rng.next_range(0, 3) + 1;
            }
            if main_rng.next_range(0, 2) == 0 {
                n /= main_rng.next_range(0, 3) + 1;
            }
            num17 -= (1_000_000 * n) as f32;
            drops.push((74, n));
        } else if num17 > 10_000.0 {
            let mut n = (num17 / 10_000.0) as i32;
            if n > 50 && main_rng.next_range(0, 2) == 0 {
                n /= main_rng.next_range(0, 3) + 1;
            }
            if main_rng.next_range(0, 2) == 0 {
                n /= main_rng.next_range(0, 3) + 1;
            }
            num17 -= (10_000 * n) as f32;
            drops.push((73, n));
        } else if num17 > 100.0 {
            let mut n = (num17 / 100.0) as i32;
            if n > 50 && main_rng.next_range(0, 2) == 0 {
                n /= main_rng.next_range(0, 3) + 1;
            }
            if main_rng.next_range(0, 2) == 0 {
                n /= main_rng.next_range(0, 3) + 1;
            }
            num17 -= (100 * n) as f32;
            drops.push((72, n));
        } else {
            let mut n = num17 as i32;
            if n > 50 && main_rng.next_range(0, 2) == 0 {
                n /= main_rng.next_range(0, 3) + 1;
            }
            if main_rng.next_range(0, 2) == 0 {
                n /= main_rng.next_range(0, 4) + 1;
            }
            if n < 1 {
                n = 1;
            }
            num17 -= n as f32;
            drops.push((71, n));
        }
    }
    drops
}

/// The 2x1 pile loot table of `Check2x1` (`WorldGen.cs:47130-47245`): the
/// coin/ore sub-styles keyed on the origin `frameX`, counts on the `genRand`
/// stream. The `if` ranges are disjoint, so at most one block runs.
fn pile_loot(frame_x: i32, gen_rand: &mut tr_core::tile_light::FastRandom) -> Vec<(u16, i32)> {
    let mut drops = Vec::new();
    if (576..=610).contains(&frame_x) {
        // The copper pile: one guaranteed pair plus the six conditional
        // pairs (:47142-47176).
        drops.push((71, gen_rand.next_range(10, 100)));
        drops.push((71, gen_rand.next_range(20, 100)));
        if gen_rand.next_range(0, 3) != 0 {
            drops.push((71, gen_rand.next_range(10, 100)));
        }
        if gen_rand.next_range(0, 3) != 0 {
            drops.push((71, gen_rand.next_range(20, 100)));
        }
        if gen_rand.next_range(0, 2) == 0 {
            drops.push((71, gen_rand.next_range(10, 100)));
        }
        if gen_rand.next_range(0, 2) == 0 {
            drops.push((71, gen_rand.next_range(20, 100)));
        }
        if gen_rand.next_range(0, 3) == 0 {
            drops.push((71, gen_rand.next_range(10, 100)));
        }
        if gen_rand.next_range(0, 3) == 0 {
            drops.push((71, gen_rand.next_range(20, 100)));
        }
        if gen_rand.next_range(0, 4) == 0 {
            drops.push((71, gen_rand.next_range(10, 100)));
        }
        if gen_rand.next_range(0, 4) == 0 {
            drops.push((71, gen_rand.next_range(20, 100)));
        }
        if gen_rand.next_range(0, 5) == 0 {
            drops.push((71, gen_rand.next_range(10, 100)));
        }
        if gen_rand.next_range(0, 5) == 0 {
            drops.push((71, gen_rand.next_range(20, 100)));
        }
    } else if (612..=646).contains(&frame_x) {
        drops.push((72, gen_rand.next_range(10, 100)));
        if gen_rand.next_range(0, 2) == 0 {
            drops.push((72, gen_rand.next_range(20, 100)));
        }
        if gen_rand.next_range(0, 3) == 0 {
            drops.push((72, gen_rand.next_range(30, 100)));
        }
    } else if (648..=682).contains(&frame_x) {
        drops.push((73, gen_rand.next_range(1, 6)));
        if gen_rand.next_range(0, 2) == 0 {
            drops.push((73, gen_rand.next_range(2, 6)));
        }
        if gen_rand.next_range(0, 3) == 0 {
            drops.push((73, gen_rand.next_range(3, 6)));
        }
    } else if (684..=718).contains(&frame_x) {
        drops.push((181, gen_rand.next_range(1, 4)));
    } else if (720..=754).contains(&frame_x) {
        drops.push((180, gen_rand.next_range(1, 4)));
    } else if (756..=790).contains(&frame_x) {
        drops.push((177, gen_rand.next_range(1, 4)));
    } else if (792..=826).contains(&frame_x) {
        drops.push((179, gen_rand.next_range(1, 4)));
    } else if (828..=862).contains(&frame_x) {
        drops.push((178, gen_rand.next_range(1, 4)));
    } else if (864..=898).contains(&frame_x) {
        drops.push((182, gen_rand.next_range(1, 4)));
    }
    drops
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
    // The plant rules of `PlaceThing_Tiles_BlockPlacementForAssortedThings`
    // (`Player.cs:41036-41360`) run on the non-`TileObject` placement route
    // before the can-place chain — seeds (82) are forced onto that route by
    // `Player.cs:39771-39772` even though the registry has the def. The two
    // C# halves split by the held item: the Staff of Regrowth sets
    // `staffOfRegrowthBonus` (:41038-41041) and takes the kill+replant block
    // (:41262-41293, `canPlace` false unless the target was 0/1/38); every
    // other item takes the `flag2` tileCut rule (:41298-41331) — the kill
    // plants nothing, so the clicked seed then places itself through the
    // normal route below and is consumed.
    if assorted_placement_route(kind) {
        if staff_of_regrowth(held.kind) {
            if !adjust_plants(app, tx, ty) {
                return false;
            }
        } else if !cut_plants_for_placement(app, tx, ty, kind) {
            return false;
        }
    }
    if legacy_precheck_applies(kind) && !can_place(app, tx, ty) {
        return false;
    }
    place_tile(app, tx, ty, kind)
}

/// The C# placement route the plant rules ride (`Player.cs:39771-39772`):
/// `TileObjectData.CustomPlace(tileToCreate, placeStyle) && tileToCreate != 82
/// && tileToCreate != 227 && tileToCreate != 4` sends 82/227/4 to the legacy
/// route even when registered. The port's registry matches `CustomPlace`, and
/// 227/4 are unregistered — so the predicate is "unregistered or 82".
fn assorted_placement_route(kind: u16) -> bool {
    tr_core::tileobject::def_for_tile(kind).is_none() || kind == 82
}

/// The held-item gate of `staffOfRegrowthBonus` (`Player.cs:41038-41041`, the
/// placement route; `:44649-44651`, the swing cut) — items 213/5295.
fn staff_of_regrowth(kind: u16) -> bool {
    matches!(kind, 213 | 5295)
}

/// The Staff-of-Regrowth half of
/// `Player.PlaceThing_Tiles_BlockPlacementForAssortedThings`
/// (`Player.cs:41038-41041`, `:41262-41293`): with the staff held, clicking
/// over a plant destroys it — saplings 3/73 and the blooming herb 84 always,
/// the mature 83 only when `IsAlchemyPlantHarvestable` — and the blooming
/// herb auto-replants (`try_replanting_herbs`). The harvest's drops ride the
/// same `break_tile` kill path as every other break source. Returns the C#
/// `canPlace` at the method's return: the staff's own branch sets it true
/// only over 0/1/38 (:41045-41048; an inactive cell reads as type 0), so no
/// grass is placed into a cell the staff just harvested.
fn adjust_plants(app: &mut App, tx: i32, ty: i32) -> bool {
    // :41045-41048 — `if (flag) { if (type == 0 || 1 || 38) canPlace = true; }`
    let can_place = app
        .world
        .tile(tx, ty)
        .map_or(true, |t| !t.active() || matches!(t.kind, 0 | 1 | 38));
    if let Some(t) = app.world.tile(tx, ty).copied() {
        if t.active() {
            match t.kind {
                3 | 73 | 84 => {
                    let herb_style = i32::from(t.frame_x) / 18;
                    let replant = t.kind == 84;
                    break_tile(app, tx, ty);
                    // `if (!Main.tile[...].active()) { if (type == 84)
                    // TryReplantingHerbs(herbStyle); }` (:41270-41278).
                    if replant && app.world.tile(tx, ty).is_some_and(|t| !t.active()) {
                        try_replanting_herbs(app, tx, ty, herb_style);
                    }
                }
                83 => {
                    // :41283-41292 — the kill is unconditional once the bloom
                    // window holds, and the replant follows without a check.
                    let herb_style = i32::from(t.frame_x) / 18;
                    if is_alchemy_plant_harvestable(herb_style, ty, &drop_ctx(app)) {
                        break_tile(app, tx, ty);
                        try_replanting_herbs(app, tx, ty, herb_style);
                    }
                }
                _ => {}
            }
        }
    }
    can_place
}

/// The `flag2` half of
/// `Player.PlaceThing_Tiles_BlockPlacementForAssortedThings`
/// (`Player.cs:41298-41331`) — the rule every non-staff placement item runs
/// on the legacy route: an active target that is `tileCut`,
/// `BreakableWhenPlacing` or `IsADripTile` and not the held item's own tile
/// is killed when it does not stand on 78/380/579 (`num9`), or — potted —
/// only for the rule's named plants (saplings, vines, drips, 165,
/// harvestable alch herbs) with an alch-placing item (`flag8`, the seeds).
/// No replant here: the killed cell takes the normal placement route below,
/// so the clicked seed plants itself (`PlaceAlch`) and is consumed. Returns
/// the C# `canPlace`: `false` refuses the placement and the plant stands.
fn cut_plants_for_placement(app: &mut App, tx: i32, ty: i32, create_tile: u16) -> bool {
    let Some(t) = app.world.tile(tx, ty).copied() else {
        return true;
    };
    // :41303 — the block only judges active cuttable targets.
    if !t.active() {
        return true;
    }
    if !(tr_core::tileobject::tile_cut(t.kind)
        || tr_core::tileobject::breakable_when_placing(t.kind)
        || tr_core::tileobject::is_drip_tile(t.kind))
    {
        return true;
    }
    // :41327-41330 — the held item's own tile refuses instead of cutting.
    if t.kind == create_tile {
        return false;
    }
    // :41307 — the support types below, read raw (no `active()` in the C#).
    let on_pot = app
        .world
        .tile(tx, ty + 1)
        .is_some_and(|s| matches!(s.kind, 78 | 380 | 579));
    // :41308-41313 — `(flag3 | flag4 | flag5 | flag6 | flag7) & flag8`.
    let ctx = drop_ctx(app);
    let named_plant = matches!(t.kind, 3 | 73)
        || tr_core::tileobject::is_vine(t.kind)
        || tr_core::tileobject::is_drip_tile(t.kind)
        || t.kind == 165
        || (tr_core::tileobject::tile_alch(t.kind)
            && is_harvestable_herb_with_seed(t.kind, i32::from(t.frame_x) / 18, ty, &ctx));
    let alch_item = tr_core::tileobject::tile_alch(create_tile);
    // `num9 || ((flag3|flag4|flag5|flag6|flag7) & flag8)` (:41314-41321).
    if !on_pot || (named_plant && alch_item) {
        break_tile(app, tx, ty);
        true
    } else {
        false // :41322-41325 — the potted plant stands; the placement refuses.
    }
}

/// `Player.TryReplantingHerbs` (`Player.cs:45824-45830`): the support below
/// must be active and 78/380/579 (579 = RockGolemHead — the decompile's
/// text, kept 1:1; `PlaceAlch`'s soil sets refuse it, so the gate alone
/// never plants on 579), then a direct `WorldGen.PlaceTile(x, y, 82, …,
/// herbStyle)` — no item consumed, no sound.
fn try_replanting_herbs(app: &mut App, tx: i32, ty: i32, herb_style: i32) {
    let supported = app
        .world
        .tile(tx, ty + 1)
        .is_some_and(|s| s.active() && matches!(s.kind, 78 | 380 | 579));
    if !supported {
        return;
    }
    if edit::place_item_tile(&mut app.world, tx, ty, 82, herb_style, 1) {
        app.request_edit(tx, ty, 1, 82);
        app.world_key = None;
    }
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
    // The furniture interactions that touch the player/world state ahead of
    // the cue/drop mapping.
    match &result {
        edit::TileInteraction::ChairSit { seat_x, seat_y, direction } => {
            if let Some(p) = app.player.as_mut() {
                p.sit_down(*seat_x, *seat_y, *direction);
            }
            app.world_key = None;
            return true;
        }
        edit::TileInteraction::BedInteract {
            set_spawn,
            spawn_x,
            spawn_y,
        } => {
            if *set_spawn {
                // `Player.cs:32678-32691`: the same cell toggles the spawn
                // off; `CheckSpawn` gates the change with its messages.
                if app.world.bed_spawn == Some((*spawn_x, *spawn_y)) {
                    app.world.bed_spawn = None;
                    // `RemoveSpawn` → `FindSpawn`'s re-scan; the port resets
                    // to the generated spawn.
                    if let Some(p) = app.player.as_mut() {
                        p.spawn_x = app.world.spawn_x as f32 * tr_core::TILE_PX + 8.0
                            - crate::player::WIDTH as f32 * 0.5;
                        p.spawn_y = app.world.spawn_y as f32 * tr_core::TILE_PX
                            - crate::player::HEIGHT as f32;
                    }
                    app.chat
                        .new_text("Spawn point removed!", crate::chat::COLOR_SERVER_MESSAGE);
                } else if check_bed_spawn(&app.world, *spawn_x, *spawn_y, &mut app.chat) {
                    app.world.bed_spawn = Some((*spawn_x, *spawn_y));
                    // `ChangeSpawn` (`Player.cs:54635`).
                    if let Some(p) = app.player.as_mut() {
                        p.spawn_x = *spawn_x as f32 * tr_core::TILE_PX + 8.0
                            - crate::player::WIDTH as f32 * 0.5;
                        p.spawn_y = *spawn_y as f32 * tr_core::TILE_PX
                            - crate::player::HEIGHT as f32;
                    }
                    app.chat
                        .new_text("Spawn point set!", crate::chat::COLOR_SERVER_MESSAGE);
                }
            }
            // The top half (`sleeping.StartSleeping`) is consumed but inert —
            // the sleep state is not ported yet (documented in ANALYSIS).
            app.world_key = None;
            return true;
        }
        edit::TileInteraction::ClockRead { text } => {
            app.chat.new_text(text, crate::chat::COLOR_SERVER_MESSAGE);
            return true;
        }
        edit::TileInteraction::CandleToggled => {
            app.world_key = None;
            return true;
        }
        _ => {}
    }
    let Some((cue, drops)) = interaction_events(result) else {
        return false;
    };
    if let Some(cue) = cue {
        app.audio
            .play_at(cue, tx as f32 * TILE_PX, ty as f32 * TILE_PX, 1.0);
    }
    for (id, x, y) in drops {
        spawn_item_drop(app, id, 1, x, y);
    }
    app.world_key = None;
    true
}

/// The App-free half of [`interact_tile`]: maps the dispatcher result to the
/// door/dial cue and the drop spawns. `None` = no interaction (fall-through
/// to the place path); a blocked door or a refused dial yields no cue
/// (vanilla plays `PlaySound(8/:9)`/`SoundID.Item4` only on the fired paths),
/// while the click still counts as consumed.
fn interaction_events(
    result: edit::TileInteraction,
) -> Option<(Option<&'static str>, Vec<(u16, i32, i32)>)> {
    let (cue, object_drops) = match result {
        edit::TileInteraction::None => return None,
        edit::TileInteraction::DoorOpened { object_drops } => (Some("Door_Opened"), object_drops),
        edit::TileInteraction::DoorClosed { object_drops } => (Some("Door_Closed"), object_drops),
        edit::TileInteraction::DoorBlocked { object_drops } => (None, object_drops),
        // `SoundID.Item4` on both dial arms (`Player.cs:32642`/:32652); the
        // key is the vanilla asset name (see `audio::SFX_CUES`). The dials
        // kill no tiles, so there are never drops.
        edit::TileInteraction::DialStarted { .. } => (Some("Item_4"), Vec::new()),
        // The refused gate (`Player.cs:32638`): still consumed (`flag2 = true`,
        // `:32637`) — no cue, no drops, no state change.
        edit::TileInteraction::DialRefused => (None, Vec::new()),
        // The furniture interactions are consumed in `interact_tile` ahead of
        // this mapping; the arms only keep the match exhaustive.
        edit::TileInteraction::CandleToggled => (None, Vec::new()),
        edit::TileInteraction::ChairSit { .. } => (None, Vec::new()),
        edit::TileInteraction::BedInteract { .. } => (None, Vec::new()),
        edit::TileInteraction::ClockRead { .. } => (None, Vec::new()),
    };
    let events = object_drops
        .iter()
        .filter_map(|d| registry::tile_drop(d.tile_kind, 0).map(|id| (id, d.x, d.y)))
        .collect();
    Some((cue, events))
}

/// `Player.CheckSpawn` (`Player.cs:54533-54590`): the bounds (the C# checks
/// `y` against `maxTilesX` — mirrored), the bed tile above the spawn cell,
/// the 3x3 area above free of solid non-solid-top tiles, and the room check.
/// The failure messages ride the chat like `Main.NewText`.
fn check_bed_spawn(
    world: &tr_core::World,
    x: i32,
    y: i32,
    chat: &mut crate::chat::ChatMonitor,
) -> bool {
    if x < 10 || x > world.width as i32 - 10 || y < 10 || y > world.width as i32 - 10 {
        return false;
    }
    if !world
        .tile(x, y - 1)
        .is_some_and(|t| t.active() && t.kind == 79)
    {
        return false;
    }
    for i in x - 1..=x + 1 {
        for j in y - 3..y {
            if let Some(t) = world.tile(i, j) {
                if (t.active() || t.has_any_liquid())
                    && td::TILE_SOLID.has(t.kind)
                    && !td::TILE_SOLID_TOP.has(t.kind)
                {
                    chat.new_text("The bed is obstructed.", crate::chat::COLOR_SERVER_MESSAGE);
                    return false;
                }
            }
        }
    }
    match tr_core::housing::check_room(world, x, y - 1) {
        Ok(_) => true,
        Err(reason) => {
            let text = match reason {
                tr_core::housing::RoomFailure::TooCloseToWorldEdge => {
                    "This bed is too close to the edge of the world."
                }
                tr_core::housing::RoomFailure::RoomIsTooBig => "This room is too big.",
                tr_core::housing::RoomFailure::RoomIsTooSmall => "This room is too small.",
                tr_core::housing::RoomFailure::HoleInWallIsTooBig => {
                    "This room has holes in its walls."
                }
                tr_core::housing::RoomFailure::TooManyUnsafeWalls => {
                    "This room has too many unsafe walls."
                }
                _ => "This room is not valid housing.",
            };
            chat.new_text(text, crate::chat::COLOR_SERVER_MESSAGE);
            false
        }
    }
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
            // The `KillTile` entry refuses the same three `CanKillTile`
            // cases (`CheckTileBreakability`, `:63537-63545`); no chest
            // store is reachable from a bare world, so that case passes
            // (the store is empty until M3-CHEST-UI anyway).
            if tr_core::edit::can_kill_tile(world, x, y, |_, _| false) {
                edit::kill_tile(world, x, y, true);
            }
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

    /// DW-3.2 — the melee-hit rig: the `mining_app` shape with a blue slime
    /// parked under the screen-centre cursor and no tile there, so the
    /// `mine_down` arm takes the NPC branch (`ItemCheck_MeleeHitNPCs` →
    /// `ProcessHitAgainstNPC`). A floor row at y 300 gives the spawner a
    /// surface; the cursor tile (30, 260) stays air so nothing is mined.
    fn melee_app() -> App {
        let mut w = World::new("t".into(), 64, 800);
        for x in 0..64 {
            let i = w.index(x, 300);
            w.tiles[i] = Tile {
                kind: 1,
                ..Default::default()
            };
            w.tiles[i].flags |= FLAG_ACTIVE;
        }
        w.compute_column_surface();
        let mut app = mining_app(w, 30, 260);
        let mut npc = crate::npc::Npc::spawn_slime_variant(
            &app.world,
            30,
            crate::npc::SlimeVariant::Blue,
        )
        .expect("the floor row spawns the slime");
        npc.x = 30.0 * TILE_PX;
        npc.y = 260.0 * TILE_PX;
        app.npcs.push(npc);
        app
    }

    #[test]
    fn test_DW_3_2_melee_hit_passes_the_player_luck_to_damage_var() {
        // DW-3.2 — `Main.DamageVar(dmg, this.luck)` with the RAW value for
        // player-dealt damage (`ProcessHitAgainstNPC`, `Player.cs:44342`;
        // damage taken negates it, `:31387` — that contact site is not wired
        // in the port yet). At luck 1.0 the reroll branch always fires
        // (2 extra draws), so the item stream diverges from luck 0 while the
        // hit lands either way.
        let mut a = melee_app();
        let hp_0 = a.npcs[0].hp;
        mine_frames(&mut a, 1);
        assert!(a.npcs[0].hp < hp_0, "the melee hit landed");
        let state_0 = a.item_rng;

        let mut b = melee_app();
        b.player.as_mut().unwrap().luck_state.luck = 1.0;
        mine_frames(&mut b, 1);
        assert!(b.npcs[0].hp < hp_0, "the lucky hit landed too");
        assert_ne!(b.item_rng, state_0, "luck 1.0 rerolled DamageVar");
    }

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
    /// The App-free [`break_drop_events`] harness for the record tests: a
    /// `Default` ctx (the record paths never read it) and a fixed `item_rng`.
    fn drop_events(
        kill: &edit::TileKill,
        hit: Option<HitCell>,
    ) -> (Vec<(u16, u32, i32, i32)>, Vec<(i32, i32)>) {
        let mut rng = 0u64;
        let mut main = crate::world_sim::npcs::LootRng(&mut rng);
        break_drop_events(
            kill,
            &object_world(),
            hit,
            false,
            &mut main,
            &DropCtx::default(),
        )
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
        // cell (`WorldGen.cs:63860-63864`) — the hit state carries the
        // dying cell's position and frames (the C# `tileCache`).
        let kill = edit::TileKill {
            kind: 1,
            ..Default::default()
        };
        let hit = Some(HitCell {
            x: 7,
            y: 12,
            kind: 1,
            frame_x: 0,
            frame_y: 0,
        });
        let (drops, portals) = drop_events(&kill, hit);
        assert!(portals.is_empty());
        assert_eq!(drops, vec![(3, 1, 7, 12)]);
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
                ..Default::default()
            }],
            ..Default::default()
        };
        let hit = Some(HitCell {
            x: 11,
            y: 19,
            kind: 18,
            frame_x: 0,
            frame_y: 0,
        });
        let (drops, portals) = drop_events(&kill, hit);
        assert!(portals.is_empty());
        assert_eq!(drops, vec![(36, 1, 10, 19)]);

        let kill = edit::TileKill {
            kind: 1,
            object_drops: vec![edit::ObjectDrop {
                tile_kind: 11,
                x: 9,
                y: 18,
                ..Default::default()
            }],
            ..Default::default()
        };
        let hit = Some(HitCell {
            x: 9,
            y: 17,
            kind: 1,
            frame_x: 0,
            frame_y: 0,
        });
        assert_eq!(
            drop_events(&kill, hit).0,
            vec![(3, 1, 9, 17), (25, 1, 9, 18)],
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
                ..Default::default()
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
                ..Default::default()
            }],
        })
        .unwrap();
        assert_eq!(cue, None);
        assert_eq!(events, vec![(32, 10, 17)]);
    }

    #[test]
    fn dw_2_1_interaction_events_map_dial_results_to_cue() {
        use edit::TileInteraction;
        // The fired dial plays `SoundID.Item4` and spawns no drops — the
        // dial arms kill no tiles (`Player.cs:32642`/:32652).
        let (cue, events) =
            interaction_events(TileInteraction::DialStarted { sundial: true }).unwrap();
        assert_eq!(cue, Some("Item_4"));
        assert!(events.is_empty());
        let (cue, events) =
            interaction_events(TileInteraction::DialStarted { sundial: false }).unwrap();
        assert_eq!(cue, Some("Item_4"));
        assert!(events.is_empty());
        // The refused gate stays consumed-but-silent (`flag2 = true`,
        // `Player.cs:32637`; no `PlaySound` behind the `:32638`/:32648 gate).
        let (cue, events) = interaction_events(TileInteraction::DialRefused).unwrap();
        assert_eq!(cue, None);
        assert!(events.is_empty());
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

    // ---- Phase 1 (M1.4-TO-5): the ungated LMB mining branch ----------------

    use crate::app::flow::front_flow_tests::flow_app;

    const MINE_SCREEN: Vec2 = Vec2::new(1280.0, 720.0);

    /// A headless mining rig: `world` staged by the caller, the player parked
    /// three tiles right of the target, the camera pinned so the screen-centre
    /// cursor points at `(tx, ty)` — everything the mining branch of
    /// `update_editing` reads (the HUD/early-return gates, the reach circle,
    /// the held slot). The held Iron Pickaxe clears the tool-tier gates from
    /// main (pick 40, `tr_core::tool`); the axe test swaps in the Iron Axe;
    /// the pick carries no projectile so the weapon branch falls through, and
    /// `flow_app`'s NPC list is empty so the melee branch does too.
    fn mining_app(world: World, tx: i32, ty: i32) -> App {
        let mut app = flow_app(None, vec![], vec![]);
        app.world = world;
        let mut player = Player::spawn(&app.world);
        player.x = (tx + 3) as f32 * TILE_PX;
        player.y = ty as f32 * TILE_PX;
        app.player = Some(player);
        app.camera = Vec2::new(tx as f32 * TILE_PX + 8.0, ty as f32 * TILE_PX + 8.0);
        app.mouse_pos = Vec2::new(640.0, 360.0); // screen centre == the camera
        app.inventory[app.selected].kind = 1; // the Iron Pickaxe (pick 40)
        app.inventory[app.selected].count = 1;
        app.mine_down = true;
        app
    }

    fn mine_frames(app: &mut App, n: usize) {
        for _ in 0..n {
            update_editing(app, 1.0 / 60.0, MINE_SCREEN);
        }
    }

    fn set_tile(w: &mut World, x: i32, y: i32, kind: u16) {
        let i = w.index(x, y);
        w.tiles[i] = Tile {
            kind,
            ..Default::default()
        };
        w.tiles[i].flags |= FLAG_ACTIVE;
    }

    /// DW-1.1: with `frame_important` set — the exact condition the removed
    /// gate read — the closed door now runs the mining branch to `break_tile`:
    /// the whole object cascades away and the door item drops, the same result
    /// the other break sources already produce (vanilla's pick tick has no
    /// `frame_important` gate, `Player.cs:45656` → `PickTile :53757`).
    #[test]
    fn test_DW_1_1_frame_important_door_breaks_through_the_mining_branch() {
        let mut w = object_world();
        w.frame_important[10] = true;
        // A doorway: a stone ceiling above the 3-tall door (both `SolidTile`
        // anchors `CheckDoorClosed` verifies), the floor comes with the fixture.
        set_tile(&mut w, 10, 16, 1);
        assert!(edit::place_item_tile(&mut w, 10, 17, 10, 0, 1));
        for y in 17..=19 {
            let t = w.tile(10, y).unwrap();
            assert!(t.active() && t.kind == 10, "door cell (10, {y}) placed");
        }
        let mut app = mining_app(w, 10, 18);
        mine_frames(&mut app, 40); // hardness 0.25 s × 35/40 ≈ 13 frames
        for y in 17..=19 {
            assert!(
                !app.world.tile(10, y).unwrap().active(),
                "door cell (10, {y}) cascaded away"
            );
        }
        assert_eq!(app.items.len(), 1, "one object drop from the cascade");
        assert_eq!(app.items[0].kind, registry::tile_drop(10, 0).unwrap());
        assert_eq!(app.items[0].kind, 25, "GetDoorItem(0) — the wooden door");
    }

    /// DW-1.1: the chest likewise — `frame_important[21]` no longer gates the
    /// branch; breaking one cell destroys the whole object and drops its item.
    #[test]
    fn test_DW_1_1_frame_important_chest_breaks_through_the_mining_branch() {
        let mut w = object_world();
        w.frame_important[21] = true;
        assert!(edit::place_item_tile(&mut w, 10, 19, 21, 0, 1));
        for x in [10, 11] {
            for y in [18, 19] {
                let t = w.tile(x, y).unwrap();
                assert!(t.active() && t.kind == 21, "chest cell ({x}, {y}) placed");
            }
        }
        let mut app = mining_app(w, 10, 18);
        mine_frames(&mut app, 40);
        for x in [10, 11] {
            for y in [18, 19] {
                assert!(
                    !app.world.tile(x, y).unwrap().active(),
                    "chest cell ({x}, {y}) cascaded away"
                );
            }
        }
        assert_eq!(app.items.len(), 1);
        assert_eq!(app.items[0].kind, registry::tile_drop(21, 0).unwrap());
        assert_eq!(app.items[0].kind, 48, "the chest item");
    }

    /// M1.4-TO-6: the cell under a chest refuses the mining break
    /// (`CanKillTile`'s container-family arm, `WorldGen.cs:62259-62270`) —
    /// the support and the object stay, no drops.
    #[test]
    fn the_support_under_a_chest_refuses_the_mining_break() {
        let mut w = object_world();
        assert!(edit::place_item_tile(&mut w, 10, 19, 21, 0, 1));
        let mut app = mining_app(w, 10, 20); // the support cell
        mine_frames(&mut app, 40);
        assert!(app.world.tile(10, 20).unwrap().active(), "the support refuses");
        for x in [10, 11] {
            for y in [18, 19] {
                assert!(
                    app.world.tile(x, y).unwrap().active(),
                    "chest cell ({x}, {y}) stands"
                );
            }
        }
        assert!(app.items.is_empty());
    }

    /// M1.4-TO-6: a chest holding items refuses its own break
    /// (`:62311-62323` through `CanDestroyChest`); emptying the store
    /// restores the DW-1.1 behaviour.
    #[test]
    fn a_stocked_chest_refuses_the_mining_break_until_emptied() {
        let mut w = object_world();
        assert!(edit::place_item_tile(&mut w, 10, 19, 21, 0, 1));
        let mut app = mining_app(w, 10, 18);
        app.world_chests.push(crate::craft::WorldChest {
            tile: (10, 18),
            slots: vec![(9, 6)],
        });
        mine_frames(&mut app, 40);
        for x in [10, 11] {
            for y in [18, 19] {
                assert!(
                    app.world.tile(x, y).unwrap().active(),
                    "stocked chest cell ({x}, {y}) refuses"
                );
            }
        }
        assert!(app.items.is_empty());
        app.world_chests.clear();
        mine_frames(&mut app, 40);
        for x in [10, 11] {
            for y in [18, 19] {
                assert!(
                    !app.world.tile(x, y).unwrap().active(),
                    "the emptied chest breaks as in DW-1.1"
                );
            }
        }
        assert_eq!(app.items.len(), 1);
        assert_eq!(app.items[0].kind, 48, "the chest item");
    }

    /// M1.4-TO-6: the locked golden door refuses `break_tile`
    /// (`:62288-62296` through `IsLockedDoor`); the top cell's frames are
    /// forced to the lock row — the break is silent and the object stays.
    #[test]
    fn the_locked_door_refuses_the_break() {
        let mut w = object_world();
        // The doorway's SolidTile anchors, as in DW-1.1.
        set_tile(&mut w, 10, 16, 1);
        assert!(edit::place_item_tile(&mut w, 10, 17, 10, 0, 1));
        w.tile_mut(10, 17).unwrap().frame_y = 594;
        let mut app = mining_app(w, 10, 17);
        break_tile(&mut app, 10, 17);
        for y in 17..=19 {
            assert!(
                app.world.tile(10, y).unwrap().active(),
                "the locked door refuses ({y})"
            );
        }
        assert!(app.items.is_empty());
    }

    /// DW-1.2: the removed cactus carve-out is behaviour-neutral and the
    /// merged tool tiers restore vanilla's axe rule — kind 80 is `tileAxe`
    /// (`Main.cs:7210`), so the rig holds the Iron Axe: the cactus breaks,
    /// drops per cell and runs no object cascade (`check_object`'s
    /// `_ => None`).
    #[test]
    fn test_DW_1_2_cactus_80_still_breaks_with_the_per_cell_drop() {
        let mut w = object_world();
        w.frame_important[80] = true;
        set_tile(&mut w, 10, 19, 80);
        let mut app = mining_app(w, 10, 19);
        app.inventory[app.selected].kind = 10; // the Iron Axe (axe 9)
        mine_frames(&mut app, 40);
        assert!(!app.world.tile(10, 19).unwrap().active(), "cactus broke");
        assert!(
            app.world.tile(10, 20).unwrap().active(),
            "the floor below survives — no cascade for kind 80"
        );
        assert_eq!(app.items.len(), 1, "the per-cell drop only");
        assert_eq!(app.items[0].kind, registry::tile_drop(80, 0).unwrap());
        assert_eq!(app.items[0].kind, 276, "the cactus item");
    }

    /// DW-1.2: a plain non-frameImportant tile keeps its pre-change behaviour —
    /// the stone floor breaks after its tool-scaled hardness (0.55 s at the
    /// baseline, ×35/40 for the held Iron Pickaxe) and drops the stone item
    /// at the hit cell.
    #[test]
    fn test_DW_1_2_non_frame_important_tile_still_breaks_as_before() {
        let w = object_world(); // frame_important[1] = false
        let mut app = mining_app(w, 20, 20);
        mine_frames(&mut app, 60); // stone hardness 0.55 s × 35/40 ≈ 29 frames
        assert!(!app.world.tile(20, 20).unwrap().active(), "stone broke");
        assert_eq!(app.items.len(), 1);
        assert_eq!(app.items[0].kind, 3, "the stone item, per-cell drop");
    }

    /// DW-1.5: releasing the button before the hardness threshold leaves the
    /// tile intact and resets the mining state — the unchanged `else` semantics.
    #[test]
    fn test_DW_1_5_releasing_mine_down_resets_the_mine_state() {
        let w = object_world();
        let mut app = mining_app(w, 20, 20);
        mine_frames(&mut app, 10); // 9/60 s into the pick-scaled stone hardness
        assert_eq!(app.mine_target, Some((20, 20)));
        assert!(app.mine_timer > 0.0 && app.mine_timer < 0.55);
        app.mine_down = false;
        mine_frames(&mut app, 1);
        assert!(app.world.tile(20, 20).unwrap().active(), "tile intact");
        assert_eq!(app.mine_target, None);
        assert_eq!(app.mine_timer, 0.0);
    }

    /// DW-1.5: opening the inventory mid-accumulation interrupts the same way.
    #[test]
    fn test_DW_1_5_opening_the_inventory_resets_the_mine_state() {
        let w = object_world();
        let mut app = mining_app(w, 20, 20);
        mine_frames(&mut app, 10);
        assert_eq!(app.mine_target, Some((20, 20)));
        app.inventory_open = true;
        mine_frames(&mut app, 1);
        assert!(app.world.tile(20, 20).unwrap().active(), "tile intact");
        assert_eq!(app.mine_target, None);
        assert_eq!(app.mine_timer, 0.0);
    }

    /// A frame_important kind without a registry entry (torch 4) breaks as a
    /// single cell with its `by_tile` drop — the plan's stated edge case for
    /// the ungated branch (no `Check*` routine exists for kind 4).
    #[test]
    fn torch_frame_important_kind_breaks_single_cell_with_its_item() {
        let mut w = object_world();
        w.frame_important[4] = true;
        set_tile(&mut w, 10, 19, 4);
        let mut app = mining_app(w, 10, 19);
        mine_frames(&mut app, 40);
        assert!(!app.world.tile(10, 19).unwrap().active(), "torch broke");
        assert_eq!(app.items.len(), 1);
        assert_eq!(app.items[0].kind, registry::tile_drop(4, 0).unwrap());
        assert_eq!(app.items[0].kind, 8, "the torch item");
    }

    // ---- M1.4-TO-4 phase 1 (DW-1.1): the seed route through the app --------

    /// `object_world` with the floor repainted to grass — a style-0 soil of
    /// `PlaceAlch` (`WorldGen.cs:45395-45397`).
    fn grass_floor_world() -> World {
        let mut w = object_world();
        for x in 0..40 {
            set_tile(&mut w, x, 20, 2);
        }
        w.compute_column_surface();
        w.compute_chunks();
        w
    }

    #[test]
    fn test_DW_1_1_seeds_plant_through_the_app_route() {
        // The routing flip: tile 82 is registered now, so the legacy
        // neighbour/overlap pre-checks no longer gate the seed — vanilla
        // excludes it from the TileObject route (`Player.cs:39772`) and
        // `PlaceAlch`'s soil/liquid rules decide instead.
        assert!(!legacy_precheck_applies(82));
        // The registry resolves the seed's tile and placeStyle.
        assert_eq!(registry::place_tile(307), Some(82));
        assert_eq!(registry::place_style(307), 0);
        assert_eq!(registry::place_style(312), 5, "Fireblossom Seeds");
        // The full item route: grass soil, Daybloom Seeds held — one item
        // consumed, the herb written with the style's frames.
        let mut app = flow_app(None, vec![], vec![]);
        app.world = grass_floor_world();
        app.inventory[app.selected] = slot(307, 5);
        assert!(use_tile_item(&mut app, 10, 19));
        assert_eq!(app.inventory[app.selected].count, 4, "one seed consumed");
        let t = *app.world.tile(10, 19).unwrap();
        assert!(t.active() && t.kind == 82, "the herb is in the world");
        assert_eq!((t.frame_x, t.frame_y), (0, 0), "frameX = 18*0");
        // Refused soil keeps the item (stone is not in style 0's set).
        let mut app2 = flow_app(None, vec![], vec![]);
        app2.world = object_world();
        app2.inventory[app2.selected] = slot(307, 5);
        assert!(!use_tile_item(&mut app2, 10, 19));
        assert_eq!(app2.inventory[app2.selected].count, 5, "the seed is kept");
        assert!(!app2.world.tile(10, 19).unwrap().active(), "nothing written");
    }

    #[test]
    fn test_DW_1_3_planter_box_items_place_through_the_app_route() {
        // The planter boxes keep the legacy pre-checks (no registry entry)
        // and land in the case-380 write with `frameY = 18 * placeStyle`.
        assert!(legacy_precheck_applies(380));
        let mut app = flow_app(None, vec![], vec![]);
        app.world = object_world();
        app.inventory[app.selected] = slot(3215, 3); // Day Bloom Planter Box
        assert!(use_tile_item(&mut app, 10, 19));
        assert_eq!(app.inventory[app.selected].count, 2);
        let t = *app.world.tile(10, 19).unwrap();
        assert!(t.active() && t.kind == 380);
        assert_eq!(t.frame_y, 0, "placeStyle 0");
        // Fire Blossom Planter Box (placeStyle 7) → frameY 126.
        let mut app2 = flow_app(None, vec![], vec![]);
        app2.world = object_world();
        app2.inventory[app2.selected] = slot(3222, 1);
        assert!(use_tile_item(&mut app2, 12, 19));
        assert_eq!(app2.world.tile(12, 19).unwrap().frame_y, 126);
    }

    // ---- Phase 3 (M1.4-TO-4): drops + RNG streams ---------------------------

    /// A staged break rig: the world swapped in, a local player present (the
    /// `FindClosest` stand-in), the item stream pinned for determinism.
    fn break_app(world: World, seed: u64) -> App {
        let mut app = flow_app(None, vec![], vec![]);
        app.world = world;
        app.player = Some(Player::spawn(&app.world));
        app.item_rng = seed;
        app
    }

    /// A synthetic herb cell (the core fixture construction): kind 82/83/84,
    /// `frameX = 18 * style`, frameY 0 (`Main.cs:7685-7687` frame-important).
    fn set_herb(w: &mut World, x: i32, y: i32, kind: u16, style: i32) {
        let i = w.index(x, y);
        w.tiles[i] = Tile {
            kind,
            frame_x: (18 * style) as i16,
            frame_y: 0,
            ..Default::default()
        };
        w.tiles[i].flags |= FLAG_ACTIVE;
        if w.frame_important.len() <= kind as usize {
            w.frame_important.resize(td::TILE_COUNT, false);
        }
        for k in [82u16, 83, 84] {
            w.frame_important[k as usize] = true;
        }
    }

    /// A synthetic 2x2 loot pot of `style` (frameY = 36 * style, the origin
    /// at `(x, y)` — `CheckPot`'s frame reconstruction).
    fn set_pot(w: &mut World, x: i32, y: i32, style: i32) {
        for (dx, dy) in [(0i32, 0i32), (1, 0), (0, 1), (1, 1)] {
            let i = w.index(x + dx, y + dy);
            w.tiles[i] = Tile {
                kind: 28,
                frame_x: (dx * 18) as i16,
                frame_y: (36 * style + dy * 18) as i16,
                ..Default::default()
            };
            w.tiles[i].flags |= FLAG_ACTIVE;
        }
        if w.frame_important.len() <= 28 {
            w.frame_important.resize(td::TILE_COUNT, false);
        }
        w.frame_important[28] = true;
    }

    /// A synthetic 3x2 tile-187 pile: `frameX = 18*dx + 54*style`, `frameY =
    /// 18*dy` (`Main.cs:7415-7417` frame-important). Style 17 is the
    /// sword-in-stone family (`WorldGen.cs:49828-49839`'s 918..=970 gate).
    fn set_large_pile_187(w: &mut World, x: i32, y: i32, style: i32) {
        for dy in 0..2i32 {
            for dx in 0..3i32 {
                let i = w.index(x + dx, y + dy);
                w.tiles[i] = Tile {
                    kind: 187,
                    frame_x: (dx * 18 + style * 54) as i16,
                    frame_y: (dy * 18) as i16,
                    ..Default::default()
                };
                w.tiles[i].flags |= FLAG_ACTIVE;
            }
        }
        if w.frame_important.len() <= 187 {
            w.frame_important.resize(td::TILE_COUNT, false);
        }
        w.frame_important[187] = true;
    }

    /// The resolver harness: `gen_seed` seeds the `genRand` stream, `seed`
    /// the `Main.rand` stream; depth bands per `WorldGen`'s scales in
    /// [`pot_ctx`] (surface 200, rock 400, underworld 800).
    fn run_pot(
        ctx: &DropCtx,
        style: i32,
        y: i32,
        gen_seed: u64,
        seed: u64,
        liquid: u8,
        wall: u16,
    ) -> (Vec<(u16, i32)>, bool) {
        let mut state = seed;
        let mut main = crate::world_sim::npcs::LootRng(&mut state);
        let mut gen_stream = tr_core::tile_light::FastRandom::new(gen_seed);
        spawn_things_from_pot(ctx, style, 10, y, liquid, wall, &mut gen_stream, &mut main)
    }

    /// A `DropCtx` with the vanilla depth bands and no special flags.
    fn pot_ctx() -> DropCtx {
        DropCtx {
            world_surface: 200.0,
            rock_layer: 400.0,
            underworld_layer: 800,
            max_tiles_y: 1000,
            ..Default::default()
        }
    }

    /// The first `gen_seed` whose stream misses both the dungeon key (35)
    /// and the potion arm (45) — the clean "falls through to num11" stream.
    fn gen_seed_clean() -> u64 {
        for seed in 0u64..10_000 {
            let mut g = tr_core::tile_light::FastRandom::new(seed);
            if g.next_range(0, 35) != 0 && g.next_range(0, 45) != 0 {
                return seed;
            }
        }
        panic!("no clean gen_seed seed");
    }

    /// The first `Main.rand` seed whose replay of the portal roll
    /// (`Next(range)`), the potion arm's `Next(45)` and `Next(7)` lands
    /// `num11` on `want` — the C# draw order with `expertMode`'s shift.
    fn main_seed_for_num11(want: i32, range: i32, expert: bool) -> u64 {
        for seed in 0u64..1_000_000 {
            let mut state = seed;
            let mut main = crate::world_sim::npcs::LootRng(&mut state);
            if main.next_range(0, range) == 0 {
                continue; // the Coin Portal
            }
            if main.next_range(0, 45) == 0 {
                continue; // the potion arm's main roll
            }
            let mut num11 = main.next_range(0, 7);
            if expert {
                num11 -= 1;
            }
            if num11 == want {
                return seed;
            }
        }
        panic!("no seed for num11 == {want}");
    }

    fn herb_ctx(day: bool, moon: u8, blood: bool, rain: bool, alpha: f32, time: f64) -> DropCtx {
        DropCtx {
            day_time: day,
            moon_phase: moon,
            blood_moon: blood,
            raining: rain,
            cloud_alpha: alpha,
            time,
            world_surface: 200.0,
            ..Default::default()
        }
    }

    fn herb_rolls(ctx: &DropCtx, kind: u16, style: i32, staff: bool) -> Vec<(u16, i32)> {
        let mut state = 7u64;
        let mut main = crate::world_sim::npcs::LootRng(&mut state);
        herb_drops(kind, style, 250, staff, ctx, &mut main)
    }

    /// The coin value of a split, in copper.
    fn coin_value(loot: &[(u16, i32)]) -> i64 {
        loot.iter()
            .map(|(id, count)| match id {
                71 => *count as i64,
                72 => *count as i64 * 100,
                73 => *count as i64 * 10_000,
                _ => *count as i64 * 1_000_000,
            })
            .sum()
    }

    // DW-3.2: the herb harvest matrix (`KillTile_GetItemDrops` :65674-65698).
    #[test]
    fn test_DW_3_2_herb_drop_matrix() {
        let day = herb_ctx(true, 0, false, false, 0.0, 0.0);
        // Blooming 84: the herb always, seeds `Next(1, 4)` (1-3).
        let drops = herb_rolls(&day, 84, 0, false);
        assert_eq!(drops[0], (313, 1), "Daybloom herb, stack 1");
        assert_eq!(drops[1].0, 307);
        assert!((1..=3).contains(&drops[1].1), "seeds 1-3, got {}", drops[1].1);
        // The staff swaps the roll set: herbs `Next(1, 3)`, seeds `Next(1, 6)`.
        let drops = herb_rolls(&day, 84, 0, true);
        assert!((1..=2).contains(&drops[0].1), "staff herbs 1-2");
        assert!((1..=5).contains(&drops[1].1), "staff seeds 1-5");
        // Mature 83 outside the bloom window: the herb alone, no seeds.
        let night = herb_ctx(false, 5, false, false, 0.0, 0.0);
        assert_eq!(herb_rolls(&night, 83, 0, false), vec![(313, 1)]);
        // Mature 83 inside the window (Daybloom by day): herb + seeds.
        assert_eq!(herb_rolls(&day, 83, 0, false).len(), 2);
        // The immature 82 drops nothing at all — the case arm is 83/84 only
        // and 82 sits in the empty case group (:65759-65815).
        assert_eq!(herb_rolls(&day, 82, 0, false), Vec::new());
        // The Shiverthorn pair (style 6 → 2358/2357).
        let drops = herb_rolls(&day, 84, 6, false);
        assert_eq!(drops[0].0, 2358);
        assert_eq!(drops[1].0, 2357);
    }

    // DW-3.2: `IsAlchemyPlantHarvestable` per style (`WorldGen.cs:66143-66174`).
    #[test]
    fn test_DW_3_2_is_alchemy_plant_harvestable_matrix() {
        let day = herb_ctx(true, 0, false, false, 0.0, 0.0);
        // The new moon (phase 0) — Deathweed's other window.
        let night = herb_ctx(false, 0, false, false, 0.0, 0.0);
        // 0 Daybloom by day, 1 Moonglow by night.
        assert!(is_alchemy_plant_harvestable(0, 100, &day));
        assert!(!is_alchemy_plant_harvestable(0, 100, &night));
        assert!(is_alchemy_plant_harvestable(1, 100, &night));
        assert!(!is_alchemy_plant_harvestable(1, 100, &day));
        // 3 Deathweed: night AND (blood moon OR moonPhase 0).
        assert!(is_alchemy_plant_harvestable(3, 100, &night));
        let blood = herb_ctx(false, 4, true, false, 0.0, 0.0);
        assert!(is_alchemy_plant_harvestable(3, 100, &blood));
        let wrong_moon = herb_ctx(false, 5, false, false, 0.0, 0.0);
        assert!(!is_alchemy_plant_harvestable(3, 100, &wrong_moon));
        // 4 Waterleaf: rain or cloudAlpha.
        assert!(is_alchemy_plant_harvestable(4, 100, &herb_ctx(true, 0, false, true, 0.0, 0.0)));
        assert!(is_alchemy_plant_harvestable(4, 100, &herb_ctx(true, 0, false, false, 0.5, 0.0)));
        assert!(!is_alchemy_plant_harvestable(4, 100, &day));
        // 5 Fireblossom: time > 40500 and not (raining while shallow).
        let late = herb_ctx(true, 0, false, false, 0.0, 40500.5);
        assert!(is_alchemy_plant_harvestable(5, 250, &late), "deep and late");
        let early = herb_ctx(true, 0, false, false, 0.0, 40499.0);
        assert!(!is_alchemy_plant_harvestable(5, 250, &early));
        // `(!Main.raining | flag)`: not raining passes at any depth, rain
        // needs the deep flag.
        assert!(
            is_alchemy_plant_harvestable(5, 250, &herb_ctx(true, 0, false, true, 0.9, 40500.5)),
            "raining but deep passes"
        );
        assert!(
            !is_alchemy_plant_harvestable(5, 100, &herb_ctx(true, 0, false, true, 0.9, 40500.5)),
            "raining and above the surface fails"
        );
        // Styles 2 and 6 have no window.
        assert!(!is_alchemy_plant_harvestable(2, 100, &day));
        assert!(!is_alchemy_plant_harvestable(6, 100, &day));
    }

    // DW-3.2: the observable break path — mining a blooming herb drops the
    // herb and its seeds, an immature one nothing.
    #[test]
    fn test_DW_3_2_mining_blooming_herb_drops_herb_and_seeds() {
        let mut w = object_world();
        set_herb(&mut w, 10, 19, 84, 3); // Deathweed blooming
        let mut app = break_app(w, 0xABCD);
        break_tile(&mut app, 10, 19);
        let kinds: Vec<u16> = app.items.iter().map(|it| it.kind).collect();
        assert!(kinds.contains(&316), "Deathweed herb 316 in {kinds:?}");
        assert!(kinds.contains(&310), "Deathweed seeds 310 in {kinds:?}");

        // The immature 82 drops nothing.
        let mut w = object_world();
        set_herb(&mut w, 10, 19, 82, 3);
        let mut app = break_app(w, 0xABCD);
        break_tile(&mut app, 10, 19);
        assert!(app.items.is_empty(), "82 never drops");
    }

    // DW-3.2: the cascade channel — breaking the soil under a mature herb
    // drops through `TileKill.herb_events` (the app resolves the rolls).
    #[test]
    fn test_DW_3_2_support_break_herb_drops_via_the_event_channel() {
        let mut w = object_world();
        set_herb(&mut w, 10, 19, 83, 0); // Daybloom, mature
        let mut app = break_app(w, 0xABCD);
        break_tile(&mut app, 10, 20); // the soil dies, the herb follows
        let kinds: Vec<u16> = app.items.iter().map(|it| it.kind).collect();
        assert!(kinds.contains(&3), "the stone drop of the hit cell");
        assert!(kinds.contains(&313), "the herb via the event channel");
    }

    // DW-3.3: the pot loot flows through the whole break path.
    #[test]
    fn test_DW_3_3_breaking_pot_spawns_loot() {
        let mut w = object_world();
        set_pot(&mut w, 10, 18, 0);
        let mut app = break_app(w, 0x5EED);
        break_tile(&mut app, 10, 18);
        for (dx, dy) in [(0i32, 0i32), (1, 0), (0, 1), (1, 1)] {
            assert!(
                !app.world.tile(10 + dx, 18 + dy).unwrap().active(),
                "pot cell gone"
            );
        }
        assert!(!app.items.is_empty(), "the loot spawns at the event cell");
        for it in &app.items {
            assert!(
                !matches!(
                    it.kind,
                    51 | 166..=174 | 197..=204 | 315 | 316 | 698..=703 | 1122..=1124
                ),
                "gore shard {} must not spawn as an item",
                it.kind
            );
        }
    }

    // DW-3.3 (merge fix): `Check3x2`'s `case 187` (`WorldGen.cs:49828-49839`)
    // — the sword-in-stone styles (origin frameX 918..=970) drop exactly one
    // sword: `Main.rand.Next(30) == 0` mints the Terragrim 4144, else the
    // Enchanted Sword 989. Regular 187 frames drop nothing (the empty
    // `GetItemDrops` group :65813-65815).
    #[test]
    fn test_DW_3_3_sword_shrine_pile_drops_one_sword() {
        let mut w = object_world();
        set_large_pile_187(&mut w, 10, 18, 17);
        let mut app = break_app(w, 0x5EED);
        break_tile(&mut app, 10, 18);
        for dy in 0..2i32 {
            for dx in 0..3i32 {
                assert!(
                    !app.world.tile(10 + dx, 18 + dy).unwrap().active(),
                    "pile cell ({},{}) gone",
                    10 + dx,
                    18 + dy
                );
            }
        }
        assert_eq!(app.items.len(), 1, "exactly one sword drop");
        assert!(
            matches!(app.items[0].kind, 989 | 4144),
            "the Enchanted Sword or the 1/30 Terragrim, got {}",
            app.items[0].kind
        );
    }

    #[test]
    fn test_DW_3_3_regular_187_pile_drops_nothing() {
        let mut w = object_world();
        set_large_pile_187(&mut w, 10, 18, 0);
        let mut app = break_app(w, 0x5EED);
        break_tile(&mut app, 10, 18);
        assert!(app.items.is_empty(), "no drop outside the sword frames");
    }

    // DW-3.3: the branch order on fixed seeds — each branch isolated by
    // replaying the C# draw order and asserting the resolver's output
    // matches an independent replay of the same draws.
    #[test]
    fn test_DW_3_3_pot_branch_order_on_fixed_seeds() {
        let ctx = pot_ctx();
        let gen_seed = gen_seed_clean();
        let surface_y = 100; // above world_surface 200

        // Coin Portal: the first main draw == 0 → no items, the portal flag.
        let mut portal_seed = 0u64;
        for seed in 0u64..100_000 {
            let mut state = seed;
            let mut main = crate::world_sim::npcs::LootRng(&mut state);
            if main.next_range(0, 500) == 0 {
                portal_seed = seed;
                break;
            }
        }
        let (loot, portal) = run_pot(&ctx, 0, surface_y, gen_seed, portal_seed, 0, 0);
        assert!(portal, "the portal roll fires on the found seed");
        assert!(loot.is_empty(), "the portal returns without loot");

        // Dungeon key: `genRand.Next(35) == 0` with a dungeon wall below the
        // surface → exactly item 327, before any potion roll.
        let mut key_gen = 0u64;
        for seed in 0u64..100_000 {
            let mut g = tr_core::tile_light::FastRandom::new(seed);
            if g.next_range(0, 35) == 0 {
                key_gen = seed;
                break;
            }
        }
        let mut key_main = 0u64;
        for seed in 0u64..100_000 {
            let mut state = seed;
            let mut main = crate::world_sim::npcs::LootRng(&mut state);
            if main.next_range(0, 500) != 0 {
                key_main = seed;
                break;
            }
        }
        let (loot, portal) = run_pot(&ctx, 0, 250, key_gen, key_main, 0, 7);
        assert!(!portal);
        assert_eq!(loot, vec![(327, 1)], "the golden key");
        // Without the dungeon wall the branch misses and the flow continues.
        let (loot, _) = run_pot(&ctx, 0, 250, key_gen, key_main, 0, 1);
        assert_ne!(loot, vec![(327, 1)]);

        // Potions: `genRand.Next(45) == 0` → the surface band's table.
        let mut potion_gen = 0u64;
        for seed in 1u64..100_000 {
            let mut g = tr_core::tile_light::FastRandom::new(seed);
            if g.next_range(0, 35) != 0 && g.next_range(0, 45) == 0 {
                potion_gen = seed;
                break;
            }
        }
        let (loot, _) = run_pot(&ctx, 0, surface_y, potion_gen, key_main, 0, 0);
        // Replay the surface table: `genRand.Next(10)` after the 45-roll.
        let mut g = tr_core::tile_light::FastRandom::new(potion_gen);
        let _ = g.next_range(0, 35);
        let _ = g.next_range(0, 45);
        let num7 = g.next_range(0, 10);
        let expected_item = match num7 {
            0 => 292,
            1 => 298,
            2 => 299,
            3 => 290,
            4 => 2322,
            5 => 2324,
            6 => 2325,
            _ => 2350,
        };
        assert_eq!(loot[0].0, expected_item, "surface potion table num7={num7}");
        if num7 >= 7 {
            assert_eq!(loot.len(), 2, "the >=7 arm adds the 2350 stack");
            assert_eq!(loot[1].0, 2350);
            assert!((1..=2).contains(&loot[1].1));
        }

        // Torches (num11 == 1): the base 8, count `Next(2, 7)`; in liquid it
        // swaps to the glowstick 282.
        let torch_seed = main_seed_for_num11(1, 500, false);
        let (loot, _) = run_pot(&ctx, 0, surface_y, gen_seed, torch_seed, 0, 0);
        assert_eq!(loot[0].0, 8, "the plain torch");
        assert!((2..=6).contains(&loot[0].1), "count 2-6, got {}", loot[0].1);
        let (loot, _) = run_pot(&ctx, 0, surface_y, gen_seed, torch_seed, 128, 0);
        assert_eq!(loot[0].0, 282, "glowstick in liquid");

        // Arrows (num11 == 2): 40 × `Next(10, 21)` above the surface.
        let arrow_seed = main_seed_for_num11(2, 500, false);
        let mut state = arrow_seed;
        let mut main = crate::world_sim::npcs::LootRng(&mut state);
        let _ = main.next_range(0, 500);
        let _ = main.next_range(0, 45);
        let _ = main.next_range(0, 7);
        let stack = main.next_range(10, 21);
        let (loot, _) = run_pot(&ctx, 0, surface_y, gen_seed, arrow_seed, 0, 0);
        assert_eq!(loot, vec![(40, stack)], "wooden arrows above ground");

        // Healing potions (num11 == 3): one lesser healing 28.
        let heal_seed = main_seed_for_num11(3, 500, false);
        let (loot, _) = run_pot(&ctx, 0, surface_y, gen_seed, heal_seed, 0, 0);
        assert_eq!(loot, vec![(28, 1)]);

        // Bombs (num11 == 4 above the underworld): 166 × `Next(4) + 1`.
        let bomb_seed = main_seed_for_num11(4, 500, false);
        let mut state = bomb_seed;
        let mut main = crate::world_sim::npcs::LootRng(&mut state);
        let _ = main.next_range(0, 500);
        let _ = main.next_range(0, 45);
        let _ = main.next_range(0, 7);
        let bombs = main.next_range(0, 4) + 1;
        let (loot, _) = run_pot(&ctx, 0, surface_y, gen_seed, bomb_seed, 0, 0);
        assert_eq!(loot, vec![(166, bombs)]);

        // Rope (num11 == 5 above the underworld, pre-hardmode): 965 ×
        // `Next(20, 41)`.
        let rope_seed = main_seed_for_num11(5, 500, false);
        let mut state = rope_seed;
        let mut main = crate::world_sim::npcs::LootRng(&mut state);
        let _ = main.next_range(0, 500);
        let _ = main.next_range(0, 45);
        let _ = main.next_range(0, 7);
        let rope = main.next_range(20, 41);
        let (loot, _) = run_pot(&ctx, 0, surface_y, gen_seed, rope_seed, 0, 0);
        assert_eq!(loot, vec![(965, rope)]);

        // Hearts (num11 == 0 while hurt): 58s, the second behind `Next(2)`.
        let mut hurt = pot_ctx();
        hurt.player_hurt = true;
        let heart_seed = main_seed_for_num11(0, 500, false);
        let mut state = heart_seed;
        let mut main = crate::world_sim::npcs::LootRng(&mut state);
        let _ = main.next_range(0, 500);
        let _ = main.next_range(0, 45);
        let _ = main.next_range(0, 7);
        let second = main.next_range(0, 2) == 0;
        let (loot, _) = run_pot(&hurt, 0, surface_y, gen_seed, heart_seed, 0, 0);
        assert!(loot.iter().all(|(id, _)| *id == 58), "hearts only");
        assert_eq!(loot.len(), if second { 2 } else { 1 });

        // Coins (num11 == 6): every drop is a coin and the total equals the
        // replayed `num17` — the C# split subtracts exactly, so the pushed
        // stacks sum to the pot's value.
        let coin_seed = main_seed_for_num11(6, 500, false);
        let (loot, _) = run_pot(&ctx, 0, 900, gen_seed, coin_seed, 0, 0); // deep band
        assert!(!loot.is_empty(), "the coin branch always drops");
        assert!(loot.iter().all(|(id, _)| (71..=74).contains(id)));
        // Replay: gen `Next(35)`/`Next(45)` misses, the coin base; main
        // portal/potion/num11 draws, then the depth (deep ×1.25), the ±20 %
        // roll and the five bonus gates.
        let mut g = tr_core::tile_light::FastRandom::new(gen_seed);
        let _ = g.next_range(0, 35);
        let _ = g.next_range(0, 45);
        let mut num17 = 200.0f32 + g.next_range(-100, 101) as f32;
        let mut state = coin_seed;
        let mut main = crate::world_sim::npcs::LootRng(&mut state);
        let _ = main.next_range(0, 500);
        let _ = main.next_range(0, 45);
        let _ = main.next_range(0, 7);
        num17 *= 1.25; // y 900 > maxTilesY 1000 - 250
        num17 *= 1.0 + main.next_range(-20, 21) as f32 * 0.01;
        if main.next_range(0, 4) == 0 {
            num17 *= 1.0 + main.next_range(5, 11) as f32 * 0.01;
        }
        if main.next_range(0, 8) == 0 {
            num17 *= 1.0 + main.next_range(10, 21) as f32 * 0.01;
        }
        if main.next_range(0, 12) == 0 {
            num17 *= 1.0 + main.next_range(20, 41) as f32 * 0.01;
        }
        if main.next_range(0, 16) == 0 {
            num17 *= 1.0 + main.next_range(40, 81) as f32 * 0.01;
        }
        if main.next_range(0, 20) == 0 {
            num17 *= 1.0 + main.next_range(50, 101) as f32 * 0.01;
        }
        num17 *= pot_style_multiplier(0, false);
        assert_eq!(coin_value(&loot), num17 as i64, "coin total == replayed num17");
    }

    // DW-3.3: the style multiplier drives the luck range and the coin scale.
    #[test]
    fn test_DW_3_3_style_multiplier_range_and_coins() {
        let close = |a: f32, b: f32| (a - b).abs() < 1e-6;
        // `num = (num * 2 + 1) / 3` after the band table.
        assert!(close(pot_style_multiplier(0, false), 1.0));
        assert!(close(pot_style_multiplier(4, false), (1.25 * 2.0 + 1.0) / 3.0));
        assert!(close(pot_style_multiplier(25, false), 7.0), "10 → 21/3");
        assert!(close(pot_style_multiplier(28, false), 1.0), "hardmode-gated");
        assert!(close(pot_style_multiplier(28, true), 3.0), "4 → 9/3");
        assert_eq!(pot_luck_range(1.0), 500);
        assert_eq!(pot_luck_range(7.0), 125);
        // The ×10 style 25 multiplies the same deep pot ×7 — the total
        // equals the replayed `num17`.
        let ctx = pot_ctx();
        let gen_seed = gen_seed_clean();
        let range = pot_luck_range(pot_style_multiplier(25, false));
        let seed = main_seed_for_num11(6, range, false);
        let (rich, _) = run_pot(&ctx, 25, 900, gen_seed, seed, 0, 0);
        let mut g = tr_core::tile_light::FastRandom::new(gen_seed);
        let _ = g.next_range(0, 35);
        let _ = g.next_range(0, 45);
        let mut num17 = 200.0f32 + g.next_range(-100, 101) as f32;
        let mut state = seed;
        let mut main = crate::world_sim::npcs::LootRng(&mut state);
        let _ = main.next_range(0, range);
        let _ = main.next_range(0, 45);
        let _ = main.next_range(0, 7);
        num17 *= 1.25; // deep
        num17 *= 1.0 + main.next_range(-20, 21) as f32 * 0.01;
        if main.next_range(0, 4) == 0 {
            num17 *= 1.0 + main.next_range(5, 11) as f32 * 0.01;
        }
        if main.next_range(0, 8) == 0 {
            num17 *= 1.0 + main.next_range(10, 21) as f32 * 0.01;
        }
        if main.next_range(0, 12) == 0 {
            num17 *= 1.0 + main.next_range(20, 41) as f32 * 0.01;
        }
        if main.next_range(0, 16) == 0 {
            num17 *= 1.0 + main.next_range(40, 81) as f32 * 0.01;
        }
        if main.next_range(0, 20) == 0 {
            num17 *= 1.0 + main.next_range(50, 101) as f32 * 0.01;
        }
        num17 *= pot_style_multiplier(25, false);
        assert_eq!(coin_value(&rich), num17 as i64, "style 25 scales by ×7");
    }

    // DW-3.5: expert mode — the num11 shift turns the torch arm into hearts,
    // the coins pay ×2.5 plus three extra rolls, torches gain `Next(1, 7)`.
    #[test]
    fn test_DW_3_5_expert_shifts_and_coin_multipliers() {
        let ctx = pot_ctx();
        let mut expert = pot_ctx();
        expert.expert_mode = true;
        let gen_seed = gen_seed_clean();
        // A seed that lands num11 == 1 non-expert lands 0 under expert —
        // torches become hearts while hurt.
        expert.player_hurt = true;
        let seed = main_seed_for_num11(1, 500, false);
        let (loot, _) = run_pot(&expert, 0, 100, gen_seed, seed, 0, 0);
        assert!(
            loot.iter().all(|(id, _)| *id == 58),
            "expert shifts num11 1 → 0: torches became hearts"
        );
        // The coin multiplier: the same coin-branch seed pays more.
        let seed = main_seed_for_num11(6, 500, false);
        let (classic, _) = run_pot(&ctx, 0, 900, gen_seed, seed, 0, 0);
        let (exp, _) = run_pot(&expert, 0, 900, gen_seed, seed, 0, 0);
        assert!(
            coin_value(&exp) > coin_value(&classic),
            "expert pays more: {} vs {}",
            coin_value(&exp),
            coin_value(&classic)
        );
    }

    #[test]
    fn test_DW_3_5_expert_torch_bonus() {
        let ctx = pot_ctx();
        let mut expert = pot_ctx();
        expert.expert_mode = true;
        let gen_seed = gen_seed_clean();
        let seed = main_seed_for_num11(1, 500, false);
        // Replay both streams: classic `Next(2, 7)`, expert adds `Next(1, 7)`.
        let mut state = seed;
        let mut main = crate::world_sim::npcs::LootRng(&mut state);
        let _ = main.next_range(0, 500);
        let _ = main.next_range(0, 45);
        let _ = main.next_range(0, 7);
        let base = main.next_range(2, 7);
        let bonus = main.next_range(1, 7);
        let (loot, _) = run_pot(&expert, 0, 100, gen_seed, seed, 0, 0);
        assert_eq!(loot, vec![(8, base + bonus)], "expert torch bonus");
        let (loot, _) = run_pot(&ctx, 0, 100, gen_seed, seed, 0, 0);
        assert_eq!(loot, vec![(8, base)]);
    }

    // DW-3.5: the >50 coin shrink — a 51-copper pot splits when the gate
    // roll hits and stays whole otherwise; value is always conserved.
    #[test]
    fn test_DW_3_5_coin_split_shrinks_above_50() {
        let mut shrunk = 0u64;
        for seed in 0u64..100_000 {
            let mut state = seed;
            let mut main = crate::world_sim::npcs::LootRng(&mut state);
            // The copper arm at 51: the gate `Next(2)` fires only above 50.
            if main.next_range(0, 2) == 0 {
                shrunk = seed;
                break;
            }
        }
        let mut state = shrunk;
        let mut main = crate::world_sim::npcs::LootRng(&mut state);
        let drops = split_coins(51.0, &mut main);
        assert_eq!(coin_value(&drops) as f32, 51.0, "value conserved");
        assert!(
            drops.len() > 1 || drops[0].1 < 51,
            "the >50 gate divided the stack"
        );
        // A seed whose gate misses keeps the 51 whole (`Next(2) != 0`).
        let mut whole = 0u64;
        for seed in 0u64..100_000 {
            let mut state = seed;
            let mut main = crate::world_sim::npcs::LootRng(&mut state);
            if main.next_range(0, 2) != 0 {
                whole = seed;
                break;
            }
        }
        let mut state = whole;
        let mut main = crate::world_sim::npcs::LootRng(&mut state);
        assert_eq!(split_coins(51.0, &mut main), vec![(71, 51)]);
        // Sweep: conservation holds across magnitudes and denominations.
        for cents in [1.0f32, 50.0, 51.0, 99.0, 100.0, 5_050.0, 123_456.0, 2_500_000.0] {
            let mut state = 42u64;
            let mut main = crate::world_sim::npcs::LootRng(&mut state);
            let drops = split_coins(cents, &mut main);
            assert_eq!(coin_value(&drops), cents as i64, "split of {cents}");
            assert!(drops.iter().all(|(_, count)| *count >= 1));
        }
    }

    // DW-3.4: two runs with the same seed and the same action sequence give
    // identical drop sets.
    #[test]
    fn test_DW_3_4_two_runs_same_seed_identical_drops() {
        let run = || {
            let mut w = object_world();
            set_pot(&mut w, 10, 18, 0);
            set_herb(&mut w, 14, 19, 84, 2);
            set_herb(&mut w, 15, 19, 83, 0);
            let mut app = break_app(w, 0xF00D);
            break_tile(&mut app, 10, 18); // the pot
            break_tile(&mut app, 14, 19); // the blooming herb (hit cell)
            break_tile(&mut app, 15, 20); // the mature herb via its soil
            let mut drops: Vec<(u16, u32)> =
                app.items.iter().map(|it| (it.kind, it.count)).collect();
            drops.sort_unstable();
            drops
        };
        assert_eq!(run(), run(), "identical seeds and actions → identical drops");
    }

    // DW-3.6 (amended): the seed route is the `flag2` tileCut rule
    // (`Player.cs:41302-41340`), not the staff's replant block — a
    // non-potted cuttable plant dies to any placement item regardless of
    // bloom (`num9`), and the clicked seed then plants ITSELF through the
    // normal placement route: consumed, its own style, stage 0.
    #[test]
    fn test_DW_3_6_seed_on_non_potted_herb_kills_and_plants_the_seed() {
        let mut w = object_world();
        // Grass soil (the Daybloom set) with an out-of-bloom mature 83.
        let i = w.index(10, 19);
        w.tiles[i] = Tile {
            kind: 2,
            ..Default::default()
        };
        w.tiles[i].flags |= FLAG_ACTIVE;
        set_herb(&mut w, 10, 18, 83, 0);
        let mut app = break_app(w, 0xBEA5);
        app.world.day_time = false; // night: `IsAlchemyPlantHarvestable(0)` false
        app.inventory[app.selected] = slot(307, 5); // Daybloom seeds
        // The cut rule kills (no pot below), then the seed plants itself.
        assert!(use_tile_item(&mut app, 10, 18));
        let t = app.world.tile(10, 18).unwrap();
        assert!(t.active() && t.kind == 82, "the seed planted itself");
        assert_eq!(t.frame_x, 0, "the seed's own style, stage 0");
        // The seed was consumed by its own placement.
        assert_eq!(app.inventory[app.selected].count, 4);
        // The kill surfaced the herb drop; the out-of-bloom 83 drops no seeds.
        let kinds: Vec<u16> = app.items.iter().map(|it| it.kind).collect();
        assert!(kinds.contains(&313), "the herb drop in {kinds:?}");
        assert!(!kinds.contains(&307), "no seeds out of bloom");
    }

    // DW-3.6 (amended): a potted herb in bloom — the kill needs
    // `flag7` (harvestable) & `flag8` (the seeds) on top of the pot support
    // (`Player.cs:41307-41316`); the seed then plants itself into the pot
    // and is consumed.
    #[test]
    fn test_DW_3_6_seed_on_potted_blooming_herb_kills_and_plants_the_seed() {
        let mut w = object_world();
        let i = w.index(10, 19);
        w.tiles[i] = Tile {
            kind: 78,
            ..Default::default()
        };
        w.tiles[i].flags |= FLAG_ACTIVE;
        w.frame_important[78] = true;
        set_herb(&mut w, 10, 18, 84, 0);
        let mut app = break_app(w, 0xBEA5);
        app.inventory[app.selected] = slot(307, 5);
        assert!(use_tile_item(&mut app, 10, 18));
        let t = app.world.tile(10, 18).unwrap();
        assert!(
            t.active() && t.kind == 82 && t.frame_x == 0,
            "the seed's own plant"
        );
        assert_eq!(app.inventory[app.selected].count, 4, "the seed consumed");
        // The harvest drops ride the same kill path (84 always harvestable).
        let kinds: Vec<u16> = app.items.iter().map(|it| it.kind).collect();
        assert!(
            kinds.contains(&313) && kinds.contains(&307),
            "herb + seed drops in {kinds:?}"
        );
    }

    // DW-3.6 (amended): a potted out-of-bloom 83 — `num9` false and `flag7`
    // false, so the C# kill rule fails and `canPlace = false`
    // (`Player.cs:41322-41325`): the plant stands and the seed is kept.
    #[test]
    fn test_DW_3_6_seed_on_potted_out_of_bloom_herb_is_refused() {
        let mut w = object_world();
        let i = w.index(10, 19);
        w.tiles[i] = Tile {
            kind: 78,
            ..Default::default()
        };
        w.tiles[i].flags |= FLAG_ACTIVE;
        w.frame_important[78] = true;
        set_herb(&mut w, 10, 18, 83, 0);
        let mut app = break_app(w, 0xBEA5);
        app.world.day_time = false; // night: not harvestable
        app.inventory[app.selected] = slot(307, 5);
        assert!(!use_tile_item(&mut app, 10, 18));
        let t = app.world.tile(10, 18).unwrap();
        assert!(t.active() && t.kind == 83, "the mature herb stands");
        assert_eq!(app.inventory[app.selected].count, 5, "the seed kept");
        assert!(app.items.is_empty(), "no harvest");
    }

    // DW-3.6 (amended): the Staff-of-Regrowth keeps the kill+replant block
    // (`Player.cs:41262-41293` — `TryReplantingHerbs`'s only C# call sites)
    // and `canPlace` stays false, so no grass is placed into the harvested
    // cell; the staff rolls 1-2 herbs + 1-5 seeds (`WorldGen.cs:65689-65693`).
    #[test]
    fn test_DW_3_6_staff_click_harvests_replants_and_refuses_placement() {
        let mut w = object_world();
        let i = w.index(10, 19);
        w.tiles[i] = Tile {
            kind: 78,
            ..Default::default()
        };
        w.tiles[i].flags |= FLAG_ACTIVE;
        w.frame_important[78] = true;
        set_herb(&mut w, 10, 18, 84, 0);
        let mut app = break_app(w, 0xBEA5);
        app.inventory[app.selected] = slot(213, 1); // Staff of Regrowth
        assert!(!use_tile_item(&mut app, 10, 18), "no placement after the kill");
        let t = app.world.tile(10, 18).unwrap();
        assert!(t.active() && t.kind == 82 && t.frame_x == 0, "replanted");
        let kinds: Vec<u16> = app.items.iter().map(|it| it.kind).collect();
        assert!(
            kinds.contains(&313) && kinds.contains(&307),
            "staff rolls in {kinds:?}"
        );
        // The staff item is not consumed.
        assert_eq!(app.inventory[app.selected].count, 1);
    }

    // DW-3.6: the Staff-of-Regrowth replant support matrix 78/380/579
    // (`Player.cs:45826`, called from the `flag` block :41279/:41290).
    #[test]
    fn test_DW_3_6_replant_support_78_380_579() {
        let replant_world = |support: u16| {
            let mut w = object_world();
            let i = w.index(10, 19);
            w.tiles[i] = Tile {
                kind: support,
                frame_x: 0,
                frame_y: 0,
                ..Default::default()
            };
            w.tiles[i].flags |= FLAG_ACTIVE;
            w.frame_important[support as usize] = true;
            set_herb(&mut w, 10, 18, 84, 4);
            w
        };
        // 78 and 380 replant the harvested herb (the style survives).
        for support in [78u16, 380] {
            let mut app = break_app(replant_world(support), 1);
            assert!(!adjust_plants(&mut app, 10, 18), "no placement canPlace");
            let t = app.world.tile(10, 18).unwrap();
            assert!(
                t.active() && t.kind == 82 && t.frame_x == 72,
                "support {support} replants style 4"
            );
        }
        // 579 passes the gate but `PlaceAlch`'s soil sets refuse it — the
        // decompile's text kept 1:1 (the plan Notes).
        let mut app = break_app(replant_world(579), 1);
        assert!(!adjust_plants(&mut app, 10, 18));
        assert!(
            !app.world.tile(10, 18).unwrap().active(),
            "579 cannot host the replant"
        );
        // Plain dirt is no support at all (the gate fails).
        let mut w = object_world();
        set_herb(&mut w, 10, 18, 84, 0);
        let mut app = break_app(w, 1);
        assert!(!adjust_plants(&mut app, 10, 18));
        assert!(!app.world.tile(10, 18).unwrap().active());
    }

    // Beyond the DW list: `Player.cs:41327-41330` — the held item's own tile
    // refuses instead of cutting (seeds on the immature 82).
    #[test]
    fn seed_on_its_own_immature_tile_is_refused() {
        let mut w = object_world();
        set_herb(&mut w, 10, 18, 82, 0);
        let mut app = break_app(w, 1);
        app.inventory[app.selected] = slot(307, 5);
        assert!(!use_tile_item(&mut app, 10, 18));
        let t = app.world.tile(10, 18).unwrap();
        assert!(t.active() && t.kind == 82, "the young plant stands");
        assert_eq!(app.inventory[app.selected].count, 5, "the seed kept");
    }

    // Beyond the DW list: the `drop_ctx` torch scan mirrors the C# inventory
    // loop — torch items by their placed tile, capped at the 20 threshold.
    #[test]
    fn drop_ctx_counts_torch_stacks_to_the_threshold() {
        let mut app = flow_app(None, vec![], vec![]);
        app.world = object_world();
        // The scan is the closest player's inventory — the rig needs one.
        app.player = Some(Player::spawn(&app.world));
        app.inventory[0] = slot(8, 30); // the torch item (places tile 4)
        app.inventory[1] = slot(8, 30);
        let ctx = drop_ctx(&app);
        assert_eq!(ctx.torches, 20, "60 torches cap at the 20 threshold");
        app.inventory[1] = slot(2, 30); // dirt is no torch
        assert_eq!(drop_ctx(&app).torches, 20, "one 30-stack still caps");
        app.inventory[0] = slot(2, 30);
        assert_eq!(drop_ctx(&app).torches, 0, "no torches → flag5's true side");
    }

    // Beyond the DW list: mining a planter box drops its own style row.
    #[test]
    fn planter_box_break_drops_the_frame_row_item() {
        let mut w = object_world();
        let i = w.index(10, 19);
        w.tiles[i] = Tile {
            kind: 380,
            frame_x: 0,
            frame_y: 54, // style 3 → item 3218
            ..Default::default()
        };
        w.tiles[i].flags |= FLAG_ACTIVE;
        w.frame_important[380] = true;
        let mut app = break_app(w, 1);
        break_tile(&mut app, 10, 19);
        assert_eq!(app.items.len(), 1);
        assert_eq!(app.items[0].kind, 3218, "3215 + frameY / 18");
    }

    // Beyond the DW list: the 2x1 copper pile (frameX 576) loots copper
    // coins through the pile event channel.
    #[test]
    fn copper_pile_break_spawns_copper_coins() {
        let mut w = object_world();
        let i = w.index(10, 19);
        w.tiles[i] = Tile {
            kind: 185,
            frame_x: 576,
            frame_y: 18,
            ..Default::default()
        };
        w.tiles[i].flags |= FLAG_ACTIVE;
        let i = w.index(11, 19);
        w.tiles[i] = Tile {
            kind: 185,
            frame_x: 594,
            frame_y: 18,
            ..Default::default()
        };
        w.tiles[i].flags |= FLAG_ACTIVE;
        w.frame_important[185] = true;
        let mut app = break_app(w, 0xC01A);
        break_tile(&mut app, 10, 20); // the support dies, the pile follows
        assert!(!app.items.is_empty(), "the pile loot spawned");
        // The hit cell (stone floor) drops its own item; every pile loot
        // entry is copper with the `Next(10, 100)` counts.
        assert!(
            app.items
                .iter()
                .filter(|it| it.kind != 3)
                .all(|it| it.kind == 71 && (10..=99).contains(&(it.count as i32))),
            "the copper pile drops copper only"
        );
        assert!(app.items.iter().any(|it| it.kind == 71));
    }

    // Beyond the DW list: the dontStarve fruit arm — `GetFruitForPot`
    // (:57018-57087) behind its `flag3` gate, the pair on the genRand stream.
    #[test]
    fn fruit_branch_dont_starve_picks_the_style_pair() {
        let mut ctx = pot_ctx();
        ctx.dont_starve = true;
        // The first gen seed hitting `Next(20) == 0` after the 35-miss.
        let mut fruit_gen = 0u64;
        for seed in 0u64..100_000 {
            let mut g = tr_core::tile_light::FastRandom::new(seed);
            if g.next_range(0, 35) != 0 && g.next_range(0, 20) == 0 {
                fruit_gen = seed;
                break;
            }
        }
        let mut key_main = 0u64;
        for cand in 0u64..100_000 {
            let mut state = cand;
            let mut main = crate::world_sim::npcs::LootRng(&mut state);
            if main.next_range(0, 500) != 0 {
                key_main = cand;
                break;
            }
        }
        // Style 4 → the 4286/4295 pair.
        let (loot, portal) = run_pot(&ctx, 4, 250, fruit_gen, key_main, 0, 0);
        assert!(!portal);
        assert_eq!(loot.len(), 1, "one fruit stack");
        assert!(
            matches!(loot[0].0, 4286 | 4295),
            "style 4 pair, got {}",
            loot[0].0
        );
        assert!((1..=2).contains(&loot[0].1), "stack Next(1, 3)");
        // Without dontStarveWorld the branch is unreachable (no roll).
        let ctx2 = pot_ctx();
        let (loot, _) = run_pot(&ctx2, 4, 250, fruit_gen, key_main, 0, 0);
        assert!(!loot.iter().any(|(id, _)| matches!(id, 4286 | 4295)));
    }

    // Beyond the DW list: the `num9` arm — any placement item cuts a
    // non-potted cuttable plant (`Player.cs:41307-41316`); the seed's own
    // placement then refuses on the sapling's stone floor (not a Daybloom
    // soil), the seed kept.
    #[test]
    fn seed_click_cuts_a_sapling_and_its_own_planting_refuses_on_stone() {
        let mut w = object_world();
        let i = w.index(10, 19);
        w.tiles[i] = Tile {
            kind: 3,
            frame_x: 0,
            frame_y: 0,
            ..Default::default()
        };
        w.tiles[i].flags |= FLAG_ACTIVE;
        let mut app = break_app(w, 1);
        app.inventory[app.selected] = slot(307, 5);
        assert!(
            !use_tile_item(&mut app, 10, 19),
            "the seed's placement refuses"
        );
        assert!(
            !app.world.tile(10, 19).unwrap().active(),
            "the sapling was cut"
        );
        assert!(
            !app.world.tile(10, 18).unwrap().active(),
            "nothing planted above"
        );
        assert_eq!(app.inventory[app.selected].count, 5, "the seed kept");
    }

    // Beyond the DW list: the Staff of Regrowth seeds a mature 83 even
    // outside its bloom window (the staff branch precedes the harvest test,
    // `WorldGen.cs:65689-65693`).
    #[test]
    fn staff_seeds_the_mature_herb_outside_the_window() {
        let night = herb_ctx(false, 5, false, false, 0.0, 0.0);
        let drops = herb_rolls(&night, 83, 0, true);
        assert_eq!(drops.len(), 2, "herb + seeds despite the closed window");
        assert_eq!(drops[0].0, 313);
        assert_eq!(drops[1].0, 307);
    }}

/// `Player.ItemCheck_UseBossSpawners`'s eye arm (`Player.cs:43759-43771`):
/// the night gate, the roar (`SoundEngine.PlaySound(15, ...)`, `SoundID.Roar`
/// = 15), then `NPC.SpawnOnPlayer(onWhichPlayer, 4)`. The
/// `SummonItemCheck` (`Player.cs:42162-42176`) refuses while an EoC is
/// already alive; `ApplyItemTime`'s re-fire guard rides the swing timers
/// (the food/potion arms' shape).
pub(crate) fn use_suspicious_looking_eye(app: &mut App) -> bool {
    let eye_alive = app
        .npcs
        .iter()
        .any(|n| n.kind == NpcKind::EyeOfCthulhu && n.hp > 0);
    if app.world.day_time || eye_alive {
        return false;
    }
    let Some(player) = app.player.as_ref() else {
        return false;
    };
    let (pcx, pcy) = player.center();
    let Some(eye) = Npc::spawn_eye_of_cthulhu(&app.world, pcx, pcy) else {
        return false;
    };
    app.audio.play("Roar_0");
    app.inventory[app.selected].count =
        app.inventory[app.selected].count.saturating_sub(1);
    if app.inventory[app.selected].count == 0 {
        app.inventory[app.selected].kind = 0;
    }
    let held = app.inventory[app.selected];
    app.swing_max = crate::item::use_animation_secs(held.kind);
    app.swing = app.swing_max;
    app.attack_cd = app.swing_max;
    app.mine_target = None;
    app.mine_timer = 0.0;
    app.npcs.push(eye);
    true
}

#[cfg(test)]
mod loot_summon_tests {
    use super::*;
    use crate::app::flow::front_flow_tests::flow_app;

    #[test]
    fn the_suspicious_eye_summons_only_at_night_and_once() {
        // `ItemCheck_UseBossSpawners`' gates: day → refused; night → the
        // roar + the spawn + the consumption; alive → refused
        // (`SummonItemCheck`).
        let mut app = flow_app(None, vec![], vec![]);
        let mut player = crate::player::Player::spawn(&app.world);
        player.x = 64.0;
        player.y = 64.0;
        app.player = Some(player);
        app.world.day_time = true;
        app.inventory[app.selected] =
            ItemSlot { kind: 43, name: "Suspicious Looking Eye".into(), count: 1 };
        assert!(
            !use_suspicious_looking_eye(&mut app),
            "day refuses"
        );
        app.world.day_time = false;
        assert!(use_suspicious_looking_eye(&mut app));
        assert_eq!(app.inventory[app.selected].count, 0, "the use consumed");
        assert!(app
            .npcs
            .iter()
            .any(|n| n.kind == NpcKind::EyeOfCthulhu && n.hp > 0));
        app.inventory[app.selected] =
            ItemSlot { kind: 43, name: "Suspicious Looking Eye".into(), count: 1 };
        assert!(
            !use_suspicious_looking_eye(&mut app),
            "SummonItemCheck refuses while alive"
        );
        assert_eq!(app.inventory[app.selected].count, 1);
    }

    #[test]
    fn the_eoc_kill_sets_the_downed_flag() {
        // `NPC.downedBoss1 = true` rides the death pass; the loot resolves
        // against the full database (the executor skips the unregistered
        // items).
        let mut app = flow_app(None, vec![], vec![]);
        let mut player = crate::player::Player::spawn(&app.world);
        player.x = 64.0;
        player.y = 64.0;
        app.player = Some(player);
        app.world.day_time = false;
        let mut eye =
            crate::npc::Npc::spawn_eye_of_cthulhu(&app.world, 64.0, 64.0).unwrap();
        eye.hp = 0;
        app.npcs.push(eye);
        crate::world_sim::npcs::update_npcs(&mut app, 1.0 / 60.0);
        assert!(app.world.downed_eoc);
        assert!(!app.npcs.iter().any(|n| n.kind == NpcKind::EyeOfCthulhu));
    }
}
