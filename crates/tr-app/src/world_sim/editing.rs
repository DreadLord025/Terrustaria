//! Tile and wall editing (plan stage A4): the per-frame mining/placing loop,
//! the break/place helpers behind it and the replicated-edit applier the net
//! layer feeds.

use glam::Vec2;
use tr_core::tiledata as td;
use tr_core::{TILE_PX, World, edit};

use crate::achievements;
use crate::app::App;
use crate::item::ItemSlot;
use crate::npc::{Npc, NpcKind};
use crate::player::Player;
use crate::projectile::Projectile;
use crate::registry;
use tr_core::loot::DropRng;

use super::items::spawn_item_drop;

/// One 60 Hz tick in seconds — the fixed step the swing/item timers are paced
/// against (`Player.Update`'s tick, `frame.rs`'s `FixedStepper::new(1.0 / 60.0)`).
const ONE_TICK_SECS: f32 = 1.0 / 60.0;

/// `AmmoID.Arrow` (40) and `AmmoID.Stake` (1836) (`ID/AmmoID.cs:161`, `:174`) —
/// the two categories `AmmoID.Sets.IsArrow` covers (`:115`), the gate the
/// `magicQuiver` knockback arm reads (`Player.cs:53547`).
const AMMO_ARROW: i32 = 40;
const AMMO_STAKE: i32 = 1836;

/// `ItemID.Sets.gunProj` (`ID/ItemID.cs:1327`) — the four endgame shooters
/// (`ItemID.cs`: 3475 VortexBeater, 3540 Phantasm, 3854 DD2PhoenixBow, 3930
/// Celeb2). `ItemCheck_Shoot` passes `dontConsume: true` for each
/// (`Player.cs:47071`), so their `PickAmmo` never drains; the call's result is
/// then thrown away — the shot's stats reset to the weapon's own raw values
/// (`:47073-47078`) and the spawn fires a FIXED gun projectile instead of the
/// resolved one (`:48986-49018`): 3475→615, 3930→714, 3540→630, 3854→705.
///
/// The scaled registry carries PLACEHOLDER rows for all four (identical
/// `use_ammo: 0`, damage 15, anim 25/25, Melee, `auto_reuse: false` — the
/// 683-row stand-in class M3-ITEM-5 documented), so their vanilla data is keyed
/// here on the item id, exactly like `ammo::not_consume`'s item-keyed arms; the
/// `tools/GenItemData` fix is a separate follow-up (ANALYSIS records it, Phase
/// 5). The port never routes them through the generic `Item.shoot` path either:
/// `ItemCheck_Shoot` runs `PickAmmo` for every `useAmmo > 0` item
/// (`Player.cs:47069`).
#[derive(Clone, Copy, Debug)]
struct GunProj {
    /// The key: the vanilla item id (`ItemID.cs`).
    kind: u16,
    /// `Item.shoot`'s fixed replacement (`Player.cs:48991`, `:48998`, `:49005`,
    /// `:49015`).
    projectile: u16,
    /// `Item.useAmmo` (`Item.cs:48996`): the ammo the have-ammo gate still
    /// requires.
    use_ammo: i32,
    /// `Item.damage` — the shot's raw damage (`:47075`).
    damage: i32,
    /// `Item.shootSpeed` — the shot's raw speed (`:47077`).
    shoot_speed: f32,
    /// `Item.knockBack` — the shot's raw knockback (`:47074`).
    knock_back: f32,
    /// `Item.useTime` (`Item.cs:49028`).
    use_time: i32,
    /// `Item.useAnimation` (`Item.cs:49029`) — the swing length and the cycler
    /// cooldown's base (`Player.cs:53570`).
    use_animation: i32,
    /// `Item.autoReuse` (`Item.cs:48997`).
    auto_reuse: bool,
    /// The item's damage class (`this.ranged = true` in all four case bodies) —
    /// the stat-multiplier and crit source the shot reads.
    class: registry::DamageClass,
}

/// The four rows above, keyed on the vanilla item id — the same id-keyed
/// convention `not_consume`'s chain uses (`ammo.rs`).
const GUN_PROG: [GunProj; 4] = [
    // 3475 Vortex Beater (`Item.cs:31159-31178`).
    GunProj {
        kind: 3475,
        projectile: 615,
        use_ammo: 97,
        damage: 50,
        shoot_speed: 20.0,
        knock_back: 2.0,
        use_time: 20,
        use_animation: 20,
        auto_reuse: true,
        class: registry::DamageClass::Ranged,
    },
    // 3540 Phantasm (`Item.cs:31728-31747`).
    GunProj {
        kind: 3540,
        projectile: 630,
        use_ammo: 40,
        damage: 50,
        shoot_speed: 20.0,
        knock_back: 2.0,
        use_time: 12,
        use_animation: 12,
        auto_reuse: true,
        class: registry::DamageClass::Ranged,
    },
    // 3854 DD2 Phoenix Bow (`Item.cs:34048-34065`).
    GunProj {
        kind: 3854,
        projectile: 705,
        use_ammo: 40,
        damage: 32,
        shoot_speed: 20.0,
        knock_back: 2.0,
        use_time: 18,
        use_animation: 18,
        auto_reuse: true,
        class: registry::DamageClass::Ranged,
    },
    // 3930 Celebration Mk2 (`Item.cs:34741-34748`).
    GunProj {
        kind: 3930,
        projectile: 714,
        use_ammo: 771,
        damage: 50,
        shoot_speed: 17.0,
        knock_back: 10.0,
        use_time: 6,
        use_animation: 6,
        auto_reuse: true,
        class: registry::DamageClass::Ranged,
    },
];

/// The `gunProj` row for the held item, if it is one of the four.
fn gun_proj_row(kind: u16) -> Option<&'static GunProj> {
    GUN_PROG.iter().find(|g| g.kind == kind)
}

/// The shooter's effective `sItem.useAmmo` (`Item.cs:48996`): the registry's
/// value, or the id-keyed vanilla one for the four `gunProj` items whose scaled
/// rows are placeholders. This is the gate `ItemCheck_Shoot` runs `PickAmmo`
/// behind (`Player.cs:47069`).
fn shooter_use_ammo(kind: u16) -> i32 {
    gun_proj_row(kind).map_or_else(
        || registry::get(kind).map_or(0, |d| d.use_ammo),
        |g| g.use_ammo,
    )
}

/// The shooter's `Item.autoReuse` (`Item.cs:48997`) — the registry's value, or
/// the id-keyed vanilla one for the four `gunProj` placeholders.
fn shooter_auto_reuse(kind: u16) -> bool {
    gun_proj_row(kind).map_or_else(
        || registry::get(kind).is_some_and(|d| d.auto_reuse),
        |g| g.auto_reuse,
    )
}

/// The shooter's swing length in seconds — `Player.ApplyItemAnimation`
/// (`Player.cs:4463-4495`) reading `Item.useAnimation` (`Item.cs:49029`) off the
/// PREFIXED instance, falling back to the raw kind. The four `gunProj` items
/// take their vanilla value from the id-keyed table.
fn shooter_swing_secs(kind: u16, prefix: u16) -> f32 {
    gun_proj_row(kind).map_or_else(
        || crate::item::use_animation_secs_of(kind, prefix),
        |g| g.use_animation as f32 * ONE_TICK_SECS,
    )
}

/// The shooter's action cadence in seconds — `Player.ApplyItemTime`
/// (`Player.cs:4416-4418` → `SetItemTime(sItem.useTime)`, `:4416-4419`):
/// `Item.useTime` (`Item.cs:49028`) read off the PREFIXED instance, with the
/// same fallback the swing start uses (a kind with no `useTime` paces off its
/// whole animation — the C#'s `Item.useTime` stock is 100,
/// `Item.cs:49028`-adjacent), and the vanilla value for the four `gunProj`
/// placeholders.
fn shooter_use_time_secs(kind: u16, prefix: u16, swing_secs: f32) -> f32 {
    gun_proj_row(kind).map_or_else(
        || {
            crate::prefix::prefixed_stats(kind, prefix)
                .map(|s| s.use_time as f32 * ONE_TICK_SECS)
                .filter(|t| *t > 0.0)
                .unwrap_or(swing_secs)
        },
        |g| g.use_time as f32 * ONE_TICK_SECS,
    )
}

/// The C#'s INTEGER tick comparisons out of the port's per-frame seconds
/// timers: `round(secs * 60)` at the 60 Hz fixed step. The C# reads
/// `itemAnimation` / `itemAnimationMax` / `useTime` as integers
/// (`Player.cs:4066`, `:53626`, `:53674`) while `app.swing` / `app.item_time`
/// decay by the frame's `dt`, so every window the roll chain reads is derived
/// from these counts — never from a raw float compare against "eight ticks",
/// which would drift with the frame time.
fn ticks_of(secs: f32) -> i32 {
    (secs * 60.0).round() as i32
}

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
    app.swing_aim = (mouse_world - Vec2::new(cx, cy))
        .y
        .atan2((mouse_world - Vec2::new(cx, cy)).x);
}

pub(crate) fn update_editing(app: &mut App, dt: f32, screen: Vec2) {
    #[cfg(test)]
    if std::env::var("AMMO_TRACE").is_ok() {
        eprintln!("[trace-top] entered, screen={}x{}", screen.x, screen.y);
    }
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
    // frames where the cursor is over the HUD or a panel. The *previous*
    // frame's edges are captured before the overwrite: the use gates below
    // must compare this frame's press against last frame's state (the C#
    // `controlUseItem && releaseUseItem` pair), not against the same frame's.
    let prev_mine_down = app.mine_prev_down;
    let prev_mine_released = app.mine_released;
    let prev_place_down = app.place_prev_down;
    let prev_place_released = app.place_released;
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
    if app.player.as_ref().is_some_and(|p| p.frozen || p.webbed) {
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
        // The event items ride the same tick (`ItemCheck_UseEventItems`,
        // `Player.cs:43920-43940`): the Goblin Battle Standard 361 starts
        // the invasion through `CanStartInvasion(1, ignoreDelay: true)`.
        if held.kind == 361 && held.count > 0 && use_goblin_battle_standard(app) {
            return;
        }
        // `ItemCheck_UseBossSpawners`' Frost Legion arm (`Player.cs:43941-43956`):
        // the Snow Globe 602 — `CanStartInvasion(2, ignoreDelay: true)`, the
        // roar, then `invasionDelay = 0; StartInvasion(2)` while no invasion
        // is active.
        if held.kind == 602 && held.count > 0 && use_snow_globe(app) {
            return;
        }
        // `ItemCheck_UseBossSpawners`' Celestial Sigil arm (`Player.cs`:
        // 44028-44034`): the 3601 use gates on `downedGolemBoss && hardMode
        // && !AnyDanger && !AnyoneNearCultists` and calls
        // `WorldGen.StartImpendingDoom(720)` — the countdown's item value
        // (`NPC.cs:6037`).
        if held.kind == 3601 && held.count > 0 && use_celestial_sigil(app) {
            return;
        }
        // `ItemCheck_UseBossSpawners`' Pirate Invasion arm
        // (`Player.cs:43957-43971`): the Pirate Map 1315 — the
        // `CanStartInvasion(3, ignoreDelay: true)` gate, the roar, then
        // `invasionDelay = 0; StartInvasion(3)` while no invasion is
        // active.
        if held.kind == 1315 && held.count > 0 && use_pirate_map(app) {
            return;
        }
        // `ItemCheck_Wiring_Gate`'s Eternia arm (`Player.cs:32425-32444`):
        // the Etherian Crystal 3828 over the Eternia Crystal Stand 466 —
        // the gates (`no ongoing event`, `no crystal`, `no moon events`,
        // the 3828 in the inventory, `WouldFailSpawningHere`'s refusal),
        // the consume, then `DD2Event.SummonCrystal` (the frame offset's
        // spawn point, `StartInvasion`).
        if held.kind == 3828 && held.count > 0 && use_etherian_crystal(app) {
            return;
        }
        // `ItemCheck_UseBossSpawners`' pre-hardmode summons
        // (`Player.cs:43751-43900`): the Worm Food 70, the Slime Crown
        // 560, the Abeemination 1133, the Bloody Spine 1331 (the eye 43
        // keeps its own fn above).
        if held.count > 0 && use_boss_summon_item(app) {
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
    // The reach gate: the borrow ends at the statement (the arms below
    // re-borrow `app.player` individually, and the shooters' swing start
    // mutates `app` after the gate).
    let player_in_reach = app.player.as_ref().is_some_and(|p| {
        let (pcx, pcy) = p.center();
        let dx = tx as f32 * TILE_PX + 8.0 - pcx;
        let dy = ty as f32 * TILE_PX + 8.0 - pcy;
        let reach = 6.0 * TILE_PX;
        dx * dx + dy * dy <= reach * reach
    });
    if !player_in_reach {
        app.edit_target = None;
        app.mine_target = None;
        app.mine_timer = 0.0;
        return;
    }
    app.edit_target = Some((tx, ty));
    #[cfg(test)]
    if std::env::var("AMMO_TRACE").is_ok() {
        eprintln!("[trace] past the reach gate");
    }
    let held = app.inventory[app.selected];
    #[cfg(test)]
    eprintln!(
        "[flow-dbg] reached: kind={} mine_down={}",
        held.kind, app.mine_down
    );
    let held_weapon = held.count > 0 && registry::is_weapon(held.kind);
    // `Player.ItemCheck`'s consumable arms: the food (buff 26/206/207,
    // gated by `CanUseItem`'s wellFed check `:14353`), the healing potions
    // (`ApplyPotionDelay` + `ApplyLifeAndOrMana`) and the magic weapons'
    // mana pay (`CheckMana`).
    if app.mine_down && held.count > 0 && !app.inventory_open {
        let def = registry::get(held.kind);
        if def.is_some_and(|d| d.consumable && matches!(d.buff_type, 26 | 206 | 207)) {
            let Some(def) = def else { unreachable!() };
            // The sim-context player borrow is immutable; the fed flag was
            // derived by this tick's scan, so the gate reads it directly.
            let can_eat = !app.player.as_ref().is_some_and(|p| p.well_fed);
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
                app.swing_max = crate::item::use_animation_secs_of(held.kind, held.prefix);
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
        if def.is_some_and(|d| d.potion && d.heal_life > 0) {
            let Some(def) = def else { unreachable!() };
            let sick = app
                .player
                .as_ref()
                .and_then(|p| p.buffs.find_buff_index(21))
                .is_some();
            let heal = def.heal_life;
            if let Some(p) = app.player.as_mut() {
                p.hp = (p.hp + heal).min(p.max_hp);
                // `ApplyPotionDelay` (`Player.cs:51853`): the buff 21
                // rides `Item.potionDelay` = 3600 (60 s).
                p.buffs.add_buff(21, 3600);
            }
            app.inventory[app.selected].count = app.inventory[app.selected].count.saturating_sub(1);
            if app.inventory[app.selected].count == 0 {
                app.inventory[app.selected].kind = 0;
            }
            app.swing_max = crate::item::use_animation_secs_of(held.kind, held.prefix);
            app.swing = app.swing_max;
            start_swing_extras(app, held.kind);
            app.attack_cd = app.swing_max;
            app.mine_target = None;
            app.mine_timer = 0.0;
            return;
        }
    }
    // The held item's prefixed stats (`Item.Prefix`'s application on a
    // fresh item, recomputed — see prefix.rs): the swing pacing, the mana
    // cost, the throw damage and the crit all read the instance values.
    let held_fx = crate::prefix::prefixed_stats(held.kind, held.prefix);
    let item_crit = held_fx.map(|s| s.crit).unwrap_or(0);
    let held_use_ammo = shooter_use_ammo(held.kind);
    let throw_ready = shooter_auto_reuse(held.kind) || (app.mine_released && !app.mine_prev_down);
    if app.mine_down && held_weapon && !app.inventory_open {
        // Weapons use LMB toward the cursor instead of mining. The shuriken
        // is not `autoReuse` (`Item.cs:1869-1885`): one throw per press edge
        // (`Player.cs:42733` `controlUseItem && releaseUseItem && itemAnimation == 0`).
        let throw_ready = registry::get(held.kind).is_some_and(|d| d.auto_reuse)
            || (prev_mine_released && !prev_mine_down);
        // The ammo shooters take the shooter path below — the throw's
        // `Item.projectile` is the ammo's projectile for them, and the shot
        // fires at `ItemTimeIsZero` mid-swing, not on the press.
        #[cfg(test)]
        eprintln!(
            "[shooter-dbg] mine_down={} held_weapon={} cd={} reuse={} ready={throw_ready}",
            app.mine_down, held_weapon, app.attack_cd, app.reuse_delay
        );
        if held_use_ammo > 0 && app.attack_cd <= 0.0 && app.reuse_delay <= 0.0 && throw_ready {
            // The shooter's use start — regardless of the ammo
            // (`ItemCheck_TryStartUse` sets the timers before
            // `ItemCheck_Shoot`'s `canShoot` gate). The shot itself fires at
            // `ItemTimeIsZero` (the fire block below).
            app.swing_max = shooter_swing_secs(held.kind, held.prefix);
            app.swing = app.swing_max;
            start_swing_extras(app, held.kind);
            // `ItemCheck_StartActualUse` (`Player.cs:51657-51697`) sets the
            // animation and leaves `itemTime` alone — it is 0 when the press
            // tick's own decrements run (`:42814`, `:42836`), so the shot gate
            // fires the FIRST shot of the swing one tick later at
            // `itemAnimation == itemAnimationMax - 1`, which IS
            // `ItemAnimationJustStarted` (`:4066`). The port's decay sits at
            // the top of the frame, BEFORE this start block, so arming ONE
            // tick (not zero — a zero would fire here and then, at the still
            // undecayed `swing == max`) puts that first shot on the next frame
            // with the swing already at `max - 1`: the tick value the volley
            // windows below are derived from.
            app.item_time = ONE_TICK_SECS;
            app.attack_cd = app.swing_max;
            app.shot_fired = false;
            app.world_key = None;
        }
        if app.attack_cd <= 0.0 && app.reuse_delay <= 0.0 && throw_ready && held_use_ammo == 0 {
            // `Player.CheckMana` (`Player.cs:52576`): the magic items pay
            // `item.mana * manaCost` (the multiplier is the 1f base — the
            // accessories fold out); a failed check blocks the use. The pay
            // resets the mana regen delay (`apply_mana_regen_delay`).
            let mana_cost = held_fx.map(|s| s.mana).unwrap_or(0);
            // The mutable player borrow covers the whole throw (the mana
            // pay + the reads the block makes through it).
            let Some(p) = app.player.as_mut() else {
                unreachable!()
            };
            if mana_cost > 0 {
                // `Player.CheckMana`'s pay arm: a failed check blocks the use.
                if p.mana < mana_cost {
                    return;
                }
                p.mana -= mana_cost;
                crate::regen::apply_mana_regen_delay(&mut p.regen, p.mana_max, p.mana, false);
            }
            // `SetItemAnimation`'s tail (`Player.cs:4496`): the reuse lock
            // rides the use start.
            app.reuse_delay = held_fx.map_or(0.0, |s| s.reuse_delay as f32 / 60.0);
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
                    let (fx_dmg, fx_spd, fx_kb) = held_fx
                        .map_or((def.damage, def.shoot_speed, def.knock_back), |s| {
                            (s.damage, s.shoot_speed, s.knock_back)
                        });
                    let raw = fx_dmg as f32 * p.weapon_damage_multiplier(class);
                    // The crit rolls at throw time (the class crit + the
                    // item's crit; `Projectile.Damage` consumes it on hit) —
                    // the item stream lives here, not in the projectile step.
                    // The whips are the exception: `DefaultToWhip` sets no
                    // melee/ranged/magic flag, so the C# crit block
                    // (`Projectile.cs:12892-12899`) never fires for them —
                    // the vanilla whip crits only through the tag's
                    // `CritChance` (deviation in ANALYSIS).
                    let crit = if tr_core::whip::is_whip(proj) {
                        false
                    } else {
                        let mut loot_rng = crate::world_sim::npcs::LootRng(&mut app.item_rng);
                        tr_core::damage::crit_roll(&mut loot_rng, p.class_crit(class) + item_crit)
                    };
                    let slot = app.projectiles.spawn_with_data_owned(
                        proj,
                        app.local_id as u8,
                        spawn.x,
                        spawn.y,
                        dir.x * fx_spd,
                        dir.y * fx_spd,
                        raw as i32,
                        fx_kb,
                        0.0,
                        0.0,
                    );
                    if let Some(slot) = slot {
                        app.projectiles.set_crit(slot, crit);
                        // `ApplyStatsFromSource` (`Projectile.cs:11119-11141`):
                        // the whip's tag row off the firing item + the
                        // prefix's `bonusTagDamage`, plus
                        // `AI_165_Whip`'s `timeToFlyOut` inputs (the prefixed
                        // `itemAnimationMax` — 27/28/30/35 per whip).
                        let tag_type = if tr_core::whip::tag_effect(held.kind).is_some() {
                            held.kind
                        } else {
                            0
                        };
                        let bonus_tag = held_fx.map_or(0, |s| s.tag_damage);
                        let animation_max =
                            held_fx.map_or(def.use_animation as f32, |s| s.use_animation as f32);
                        app.projectiles
                            .set_whip_params(slot, tag_type, bonus_tag, animation_max);
                    }
                    if def.consumable {
                        app.inventory[app.selected].count =
                            app.inventory[app.selected].count.saturating_sub(1);
                    }
                    // `Player.ApplyItemAnimation` (`Player.cs:4449-4451`): the
                    // swing lasts the item's `useAnimation` ticks; `attack_cd`
                    // matches so the throw repeats only when the swing is over.
                    app.swing_max = crate::item::use_animation_secs_of(held.kind, held.prefix);
                    app.swing = app.swing_max;
                    start_swing_extras(app, held.kind);
                    app.attack_cd = app.swing_max;
                }
            }
        }

        // The ammo shooters fall through to the fire block below — the
        // vanilla's `ItemCheck_Shoot` runs inside `ItemCheck`, before the
        // mining/placing arms return.
        if held_use_ammo == 0 {
            return;
        }
    }
    if app.reuse_delay > 0.0 {
        app.reuse_delay = (app.reuse_delay - dt).max(0.0);
    }
    // Melee: LMB over an NPC within four tiles hits it instead of mining.
    // The ammo shooters skip it — they fire their `useAmmo` projectile
    // instead (`Player.ItemCheck_Shoot`'s gate, `Player.cs:47069-47075`). The
    // `held_use_ammo` read is the effective one from the top of the pass, so the
    // four `gunProj` placeholder rows are shooters here too.
    if app.mine_down
        && held_use_ammo == 0
        && app.attack_cd <= 0.0
        && app.reuse_delay <= 0.0
        && !app.inventory_open
    {
        let ax = world_pos.x;
        let ay = world_pos.y;
        let (pcx, pcy) = app.player.as_ref().map(|p| p.center()).unwrap_or((ax, ay));
        let reach2 = 4.0 * TILE_PX;
        if (ax - pcx).powi(2) + (ay - pcy).powi(2) <= reach2 * reach2 {
            let hit_idx = app.npcs.iter().position(|n| n.contains(ax, ay));
            if let Some(idx) = hit_idx {
                let n = &mut app.npcs[idx];
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
                let item_crit = held_fx.map(|s| s.crit).unwrap_or(0);
                let raw = if held_weapon {
                    held_fx.map_or(registry::damage(held.kind), |s| s.damage) as f32
                        * app
                            .player
                            .as_ref()
                            .map(|p| p.weapon_damage_multiplier(class))
                            .unwrap_or(1.0)
                } else {
                    15.0
                };
                // `Main.DamageVar(dmg, this.luck)` — the player-dealt roll
                // passes the raw value (`ProcessHitAgainstNPC`,
                // `Player.cs:44342`; damage taken negates it, `:31387`).
                let (luck, class_crit) = app
                    .player
                    .as_ref()
                    .map(|p| (p.luck(), p.class_crit(class)))
                    .unwrap_or((0.0, 0));
                let mut loot_rng = crate::world_sim::npcs::LootRng(&mut app.item_rng);
                let crit = tr_core::damage::crit_roll(&mut loot_rng, class_crit + item_crit);
                let dmg = tr_core::damage::damage_var(&mut loot_rng, raw, luck);
                let dmg = tr_core::damage::strike_npc_damage(dmg, n.defense(), crit);
                // `NPC.StrikeNPC` plays `this.HitSound` at `base.position`
                // (`NPC.cs:81546`) so the hit is placed at the NPC corner.
                let (nx, ny) = (n.x, n.y);
                // The melee stand-in's knockback: a mid-sword 5.5
                // (`Item.knockBack` for the copper shortsword class); the
                // per-item values land with M3-ITEM.
                n.hurt(dmg, pcx, 5.5, crit);
                // A joining client's strike goes to the host as `28`
                // (`NPC.StrikeNPC`'s `fromNet: false` send, `NPC.cs:81218`).
                app.send_npc_strike(idx, dmg, 5.5, crit, pcx >= nx);
                app.audio.play_at("NPC_Hit_1", nx, ny, 1.0);
                // One swing per `useAnimation` (`Player.cs:4449-4451`).
                app.swing_max = crate::item::use_animation_secs_of(held.kind, held.prefix);
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
        if def.is_some_and(|d| d.potion && d.heal_life > 0) {
            let Some(def) = def else { unreachable!() };
            let sick = app
                .player
                .as_ref()
                .and_then(|p| p.buffs.find_buff_index(21))
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
                app.swing_max = crate::item::use_animation_secs_of(held.kind, held.prefix);
                app.swing = app.swing_max;
                start_swing_extras(app, held.kind);
                app.attack_cd = app.swing_max;
                app.mine_target = None;
                app.mine_timer = 0.0;
                return;
            }
        }
    }
    // `Player.cs:42768-42771` (the use-success arm) + the can-use gate
    // `:52312-52314` -> `Mount.CanMount` (`Mount.cs:6581-6604`): a saddle
    // item equips its mount (a swap while mounted goes through the same
    // `SetMount` guards, `Mount.cs:6196-6213`). Sits after the weapon/potion
    // arms so the sim-context player borrow is already dead (NLL).
    if app.mine_down && held.count > 0 && !app.inventory_open {
        if let Some(m) = crate::mount::mount_type_of_item(held.kind) {
            if let Some(p) = app.player.as_mut() {
                if p.can_mount(&app.world, m) {
                    p.set_mount(m, p.wet);
                    let secs = crate::item::use_animation_secs(held.kind);
                    app.swing_max = secs;
                    app.swing = secs;
                    app.attack_cd = secs;
                    app.mine_target = None;
                    app.mine_timer = 0.0;
                    return;
                }
            }
        }
    }
    // `Player.cs:31695-31698` (`ItemCheck_ManageRightClickFeatures`) and
    // `:24992-24994`: using an item dismounts the mounts that say so — the
    // held item being the very same mount is exempt (`:31697`).
    if app.mine_down && !app.inventory_open {
        if let Some(p) = app.player.as_mut() {
            if p.mount.is_active() && p.mount.dismount_on_item_use() {
                let same = crate::mount::mount_type_of_item(held.kind) == Some(p.mount.kind());
                if !same {
                    p.try_dismount(&app.world);
                }
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
            tool_pick > 0
                && tr_core::tool::pick_can_mine(tile.kind, tool_pick as i32, below_surface)
        };
        if !can_mine {
            app.mine_target = None;
            app.mine_timer = 0.0;
        } else {
            if app.swing <= 0.0 {
                app.swing_max = crate::item::use_animation_secs_of(held.kind, held.prefix);
                app.swing = app.swing_max;
                start_swing_extras(app, held.kind);
            }
            // `PickTile_DetermineDamage` (`Player.cs:53830-53833`): a
            // `CanKillTile` refusal zeroes the pick damage — the swing
            // plays, the accumulator never reaches 100. (The C# fail tick's
            // `KillTile(fail: true)` sound/dust has no per-tick path in the
            // port.) `break_tile` re-checks for the other break sources.
            let refused = !edit::can_kill_tile(&app.world, tx, ty, chest_holds_items(app));
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
            app.swing_max = crate::item::use_animation_secs_of(held.kind, held.prefix);
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
            let hardness =
                (if tile.wall == 1 { 0.7 } else { 0.35 }) * 35.0 / tool_hammer.max(1) as f32;
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
    let reuse_ready = auto_reuse || (prev_place_released && !prev_place_down);
    let starting = swing_over && reuse_ready;
    let inside_swing = app.swing > 0.0 && app.item_time <= 0.0;
    // Tile interactions (`Player.Update` order: LookForTileInteractions
    // (`Player.cs:27430`) runs before ItemCheck (`:31827`)): one action per
    // right-click press — `UpdateReleaseUseTile` (`:29224`) clears
    // `releaseUseTile` right after the attempt, so the door branch fires on
    // the press edge. A handled interaction consumes the click; anything else
    // falls through to the place path unchanged.
    if app.place_down
        && !prev_place_down
        && !app.inventory_open
        && crate::world_sim::npcs::try_open_shop_at_cursor(app, world_pos)
    {
        return;
    }
    if app.place_down
        && !prev_place_down
        && !app.inventory_open
        && crate::chest::try_open_chest_at_cursor(app, world_pos)
    {
        return;
    }
    if app.place_down && !prev_place_down && !app.inventory_open && interact_tile(app, tx, ty) {
        return;
    }
    // `ItemCheck_Shoot` (`Player.cs:47056+`): the ammo shooters fire at
    // `ItemTimeIsZero` mid-swing (`Player.cs:43544`'s
    // `itemAnimation > 0 && ItemTimeIsZero` gate). `PickAmmo`
    // (`Player.cs:53458-53676`) picks the ammo, fuses the damage/speed and
    // drains it through the not-consume rolls; `canShoot = false` (no
    // matching ammo) fires nothing but the swing still played.
    // The swing decay and `item_time`'s decrement share this tick — the shot
    // fires on the tick `item_time` hits zero (`ItemTimeIsZero`,
    // `Player.cs:43544`), gated by the swing still running (`itemAnimation > 0`)
    // and a started swing — and it fires on EVERY such tick: `ApplyItemTime`
    // (`ItemCheck_Shoot`'s tail, `Player.cs:47184-47193` — `itemTime = useTime`)
    // re-arms the cadence after each shot, which is what lets a
    // `useTime < useAnimation` shooter (the Clockwork 4<12, the Fairy Queen
    // 2<30) fire its whole volley instead of one shot per swing.
    //
    // ## The tick semantics
    //
    // The C# compares `itemAnimation`, `itemAnimationMax` and `useTime` as
    // INTEGERS (`:4066`, `:53626`, `:53674`) while the port's timers are seconds
    // decaying per frame (`:45`/`:72`), so the wire derives the three counts
    // once, rounded at 60 Hz, and reads every window off those — never a raw
    // float compare against "eight ticks". `item_time` is read the same way so
    // a sub-tick float residue cannot slip a volley shot a frame late.
    let anim_max_ticks = ticks_of(app.swing_max);
    let use_time_ticks = ticks_of(shooter_use_time_secs(held.kind, held.prefix, app.swing_max));
    let remaining_anim_ticks = ticks_of(app.swing);
    let item_time_ticks = ticks_of(app.item_time);
    // `shot_fired` is now the per-SWING first-shot marker (cleared at the start
    // block): the C#'s `ItemAnimationJustStarted` is `itemAnimation ==
    // itemAnimationMax - 1` (`Player.cs:4066`), and the press-frame arming puts
    // the swing's first shot exactly there, so "this is the swing's first shot"
    // is the port's stand-in for that comparison.
    let just_started = !app.shot_fired;
    if held_weapon
        && held_use_ammo > 0
        && item_time_ticks <= 0
        && app.swing_max > 0.0
        && remaining_anim_ticks > 0
    {
        app.shot_fired = true;
        // `Player.ammoCyclingMode`/`ammoCyclingOffset` — per-tick granted by
        // `grant_ammo_mastery` (`Player.cs:14528-14544`), so the wire reads the
        // state the fixed step already derived.
        let mut cycling = app
            .player
            .as_ref()
            .map_or(crate::ammo::AmmoCycling::default(), |p| p.ammo_cycling);
        // `PickAmmo_IterateRange`'s Clentaminator override (`Player.cs:53708-53711`):
        // the two solution guns always take their FIRST matching stack,
        // whatever the mode and offset — forced here so `ammo::pick` stays
        // shooter-agnostic.
        if held.kind == 779 || held.kind == 5134 {
            cycling.offset = 0;
        }
        let cycling = (cycling.mode != crate::ammo::AmmoCyclingMode::None).then_some(&cycling);
        let Some(src) = crate::ammo::pick(&app.inventory, &app.equip, held_use_ammo, cycling)
        else {
            // `canShoot = false` (`Player.cs:53462-53466`): the use plays
            // out dry — no shot, no drain. `ItemCheck_Shoot` returns BEFORE
            // `ApplyItemTime` on this path (`Player.cs:47147`), so `itemTime`
            // stays zero and the dry swing keeps re-running the pick, exactly
            // like the vanilla.
            return;
        };
        let ammo_kind = crate::ammo::slot(src, &app.inventory, &app.equip).kind;
        let ammo_def = registry::get(ammo_kind);
        // The `gunProj` four (`ItemID.cs:1327`) run the same chain and then
        // discard its stat fuse (`:47073-47078`) and its projectile
        // (`:48986-49018`) — the wire keeps the resolved projectile only as the
        // roll chain's `projToShoot` key.
        let gun_proj = gun_proj_row(held.kind);
        // `Player.cs:53458-53568`: the resolution chain over the picked ammo —
        // the specific-pair table, the per-weapon overrides and the stat fuse.
        // `None` = nothing to resolve (an ammo kind the registry does not
        // carry): the shot is refused, the dry swing still played.
        let resolved = ammo_def.and_then(|ammo_def| {
            let Some(weapon_def) = registry::get(held.kind) else {
                return None;
            };
            let flags = app
                .player
                .as_ref()
                .map_or(crate::ammo::AmmoFlags::default(), |p| p.ammo_flags());
            // `speed`'s and `projToShoot`'s seeds are `sItem.shootSpeed` /
            // `sItem.shoot` (the INSTANCE's, `Player.cs:47058-47059`) — the
            // prefixed value for a prefixed weapon.
            let weapon_shoot_speed = gun_proj.map_or_else(
                || held_fx.map_or(weapon_def.shoot_speed, |s| s.shoot_speed),
                |g| g.shoot_speed,
            );
            crate::ammo::resolve_projectile(
                held.kind,
                ammo_kind,
                held_use_ammo,
                ammo_def.damage,
                ammo_def.shoot_speed,
                ammo_def.knock_back,
                registry::projectile(held.kind).map_or(0, |p| p as i32),
                weapon_shoot_speed,
                app.inventory[app.selected].kind,
                &flags,
            )
        });
        let Some(resolved) = resolved else {
            return;
        };
        let proj_to_shoot = resolved.projectile;
        // `Player.cs:53569-53577`: the cycling update, inside `PickAmmo` AFTER
        // the resolution chain and BEFORE the roll chain — so a `Random` mode's
        // `Main.rand.Next()` draw (`:53577`) precedes the roll arms' draws.
        // `!dontConsume` only: the four gunProj items never advance the cycler.
        let dont_consume = gun_proj.is_some();
        if !dont_consume && cycling.is_some() {
            if let Some(p) = app.player.as_mut() {
                // `ammoCyclingCooldown = sItem.useAnimation + 120` — the INSTANCE
                // animation ticks, the same count the swing max derives from.
                p.ammo_cycling.cooldown = anim_max_ticks + 120;
                p.ammo_cycling.offset += 1;
                if p.ammo_cycling.mode == crate::ammo::AmmoCyclingMode::Random {
                    // `ammoCyclingOffset = Main.rand.Next()` (`:53577`) — one
                    // unbounded draw from the same stream the roll arms below
                    // read, taken through the app's stream helper (`app::next_rand`
                    // is the splitmix round `ammo.rs::next` draws with), and
                    // masked to .NET's non-negative `Random.Next()` range.
                    p.ammo_cycling.offset =
                        (crate::app::next_rand(&mut app.item_rng) as u32 as i32) & i32::MAX;
                }
            }
        }
        // `Player.cs:53578-53682`: the drain behind the not-consume rolls, with
        // the windows the C# reads at this tick derived from the integer counts
        // above. `dontConsume` seeds the chain for the four gunProj items
        // (`ItemID.cs:1327`) — their own rolls still spend their draws.
        let flags = app
            .player
            .as_ref()
            .map_or(crate::ammo::AmmoFlags::default(), |p| p.ammo_flags());
        if !crate::ammo::not_consume(
            held.kind,
            held_use_ammo,
            proj_to_shoot,
            &crate::ammo::VolleyWindows {
                just_started,
                // `itemAnimation > sItem.useAnimation - 8` (`:53626`) — the
                // Fairy Queen's first-eight-ticks window.
                anim_gt_use_anim_minus_8: remaining_anim_ticks > anim_max_ticks - 8,
                // `itemAnimation < itemAnimationMax - sItem.useTime` (`:53674`)
                // — the spray/Clentaminator first-shot window.
                anim_lt_anim_max_minus_use_time: remaining_anim_ticks
                    < anim_max_ticks - use_time_ticks,
            },
            &flags,
            dont_consume,
            &mut app.item_rng,
        ) {
            let mut equip = std::mem::take(&mut app.equip);
            let mut inv = app.inventory;
            let drained = crate::ammo::drain(src, &mut inv, &mut equip);
            app.inventory = inv;
            app.equip = equip;
            if !drained {
                return;
            }
        }
        // The shot's stats. `Player.cs:53541-53568` fuses the ammo's own damage
        // (through the weapon's class multiplier) and shootSpeed onto the
        // weapon's; `Player.cs:47073-47078` then discards the whole fuse for the
        // four gunProj items, whose shot carries the weapon's own raw values.
        let Some(weapon_def) = registry::get(held.kind) else {
            return;
        };
        // The class the shot's multiplier and crit read: the weapon's own, or
        // the id-keyed vanilla one for the four `gunProj` placeholders (all
        // four are `ranged` in `Item.cs`).
        let class = gun_proj.map_or(weapon_def.class, |g| g.class);
        // The weapon's own (prefixed) instance values. `shoot_speed` is NOT
        // read here: the resolver's fused `speed` already starts from it
        // (`Player.cs:47058-47059`'s seed), and the gunProj reset overrides it
        // with the id-keyed vanilla value.
        let (w_dmg, w_kb, w_crit) = held_fx.map_or(
            (weapon_def.damage, weapon_def.knock_back, weapon_def.crit),
            |s| (s.damage, s.knock_back, s.crit),
        );
        if let Some(p) = app.player.as_mut() {
            let weapon_damage = w_dmg as f32 * p.weapon_damage_multiplier(class);
            let mut damage = weapon_damage;
            let mut speed = resolved.speed;
            let mut knock_back = w_kb;
            if let Some(gun) = gun_proj {
                // `Player.cs:47074-47078`: `KnockBack = sItem.knockBack;
                // Damage = weaponDamage; speed = sItem.shootSpeed`, and the
                // fixed projectile from `ItemCheck_Shoot`'s tail (`:48986-49018`).
                // `weaponDamage` is the class-multiplied weapon damage
                // (`Player.GetWeaponDamage`, the argument `ItemCheck_Shoot` was
                // called with). The weapon's own raw values come from the
                // id-keyed vanilla table — the scaled registry rows for these
                // four are place holders.
                damage = gun.damage as f32 * p.weapon_damage_multiplier(class);
                speed = gun.shoot_speed;
                knock_back = gun.knock_back;
            } else {
                if resolved.damage > 0 {
                    // `Damage += (int)(num * GetWeaponDamageMultiplier(item))`
                    // (`:53552-53555`) — the AMMO-side contribution the
                    // resolver fused.
                    damage += resolved.damage as f32 * p.weapon_damage_multiplier(class);
                }
                if resolved.projectile == 285 {
                    // `:53556-53559`: the Nanite-bullet arm scales the FUSED
                    // damage, so the weapon's own part takes the same factor
                    // (the resolver already applied it to its own half).
                    damage *= 1.25;
                }
                // `KnockBack *= 1.1f` (`:53547`) runs BEFORE
                // `KnockBack += item.knockBack` (`:53568`), so it scales the
                // weapon's own value only — `AmmoShot.knock_back` is the ammo
                // side the resolver added last.
                if flags.magic_quiver
                    && (held_use_ammo == AMMO_ARROW || held_use_ammo == AMMO_STAKE)
                {
                    knock_back *= 1.1;
                }
                knock_back += resolved.knock_back;
            }
            let (pcx, pcy) = p.center();
            let dir = (world_pos - Vec2::new(pcx, pcy)).normalize_or_zero();
            if dir == Vec2::ZERO {
                return;
            }
            let spawn = Vec2::new(pcx, pcy) + dir * 10.0;
            let crit = {
                let mut loot_rng = crate::world_sim::npcs::LootRng(&mut app.item_rng);
                tr_core::damage::crit_roll(&mut loot_rng, p.class_crit(class) + w_crit)
            };
            // origin/main’s owned spawner (the owner plumbing of the
            // projectile-ownership task) + this task’s resolution: the pair
            // table’s or the ammo’s own projectile, the gunProj fixed id when
            // the weapon is one of the four, and the fused knockback (the
            // weapon’s own + the ammo’s + the magicQuiver scale), which
            // supersedes the pre-fuse `w_kb` read.
            let slot = app.projectiles.spawn_with_data_owned(
                gun_proj.map_or(resolved.projectile, |g| g.projectile),
                app.local_id as u8,
                spawn.x,
                spawn.y,
                dir.x * speed,
                dir.y * speed,
                damage as i32,
                knock_back,
                0.0,
                0.0,
            );
            if let Some(slot) = slot {
                app.projectiles.set_crit(slot, crit);
            }
            app.audio.play_at("Item_1", pcx, pcy, 1.0);
        }
        // `ApplyItemTime` (`Player.cs:47184-47193` → `SetItemTime(useTime)`,
        // `:4416-4419`): the shot re-arms the action cadence, so the next shot
        // of the volley fires `useTime` ticks later — for as long as the swing
        // still runs (the gate above) and for every shot, not once per swing.
        // This is the whole difference between a one-shot-per-swing port and
        // the vanilla's volley.
        app.item_time = shooter_use_time_secs(held.kind, held.prefix, app.swing_max);
    }
    if app.place_down && !app.inventory_open && (starting || inside_swing) {
        // The extractinator arm first (`PlaceThing`'s order, `Player.cs:39586-39597`:
        // `PlaceThing_ItemInExtractinator` runs before the tile/wall routes). It
        // consumes the tick on its own terms — `ApplyItemTime` makes the swing
        // and the action time the SAME value (`Player.cs:4406-4413`, ×0.33 on
        // tile 642), so the generic place-route timings below must not clobber
        // it; `Some(secs)` is the applied action time.
        let mut extracted_secs = None;
        if !held_wall {
            extracted_secs = crate::world_sim::extract::use_extractinator(app, tx, ty, world_pos);
        }
        let used = if extracted_secs.is_some() {
            true
        } else if held_wall {
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
            if let Some(secs) = extracted_secs {
                // `SetItemTime` (`Player.cs:4406-4413`): swing == item time ==
                // useTime × multiplier (the extraction's own `ApplyItemTime`).
                app.swing_max = secs;
                app.swing = secs;
                app.item_time = secs;
            } else {
                if starting {
                    app.swing_max = swing_secs;
                    app.swing = app.swing_max;
                }
                app.item_time = action_secs;
            }
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
            .any(|c| c.tile == (x, y) && c.slots.iter().any(|s| s.count > 0))
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
    let kind = app
        .world
        .tile(tx, ty)
        .filter(|t| t.active())
        .map(|t| t.kind);
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
    let hit = app
        .world
        .tile(tx, ty)
        .filter(|t| t.active())
        .map(|t| HitCell {
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
        app.audio.play_at(
            "Dig_1",
            event.x as f32 * TILE_PX,
            event.y as f32 * TILE_PX,
            1.0,
        );
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
        // M5-SEED-1: `Main.dontStarveWorld` (`editing.rs`'s fruit/pot
        // branches, `:1437-1474`) — the world's own flag replaces the
        // hard-coded false.
        dont_starve: app
            .world
            .seeds
            .contains(tr_core::seeds::SeedFlags::DONT_STARVE),
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
        for (id, count) in herb_drops(
            herb.kind,
            herb.style,
            herb.y,
            staff_of_regrowth,
            ctx,
            main_rng,
        ) {
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
fn sword_pile_roll(frame_x: i32, rng: &mut super::npcs::LootRng<'_>) -> Option<u16> {
    if !(918..=970).contains(&frame_x) {
        return None;
    }
    Some(if rng.next_range(0, 30) == 0 {
        4144
    } else {
        989
    })
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
        if gen_rand.next_range(0, 2) == 0 { a } else { b }
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
        let potion = if y > ctx.underworld_layer || ctx.hard_mode {
            188
        } else {
            28
        };
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
fn split_coins(mut num17: f32, main_rng: &mut super::npcs::LootRng<'_>) -> Vec<(u16, i32)> {
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
/// `TileInteractionsUse`'s Lihzahrd Altar arm (`Player.cs:32755-32790`):
/// the gates are `!NPC.AnyNPCs(245) && Main.hardMode &&
/// NPC.downedPlantBoss`; the 58-slot inventory scan consumes one Power
/// Cell 1293; the roar is `SoundID.Roar` (15) and the summon is
/// `NPC.SpawnOnPlayer(plr, 245)`. The click is consumed whether or not
/// the gates pass (the C#'s `flag2 = true`).
fn use_golem_altar(app: &mut App) -> bool {
    let gated = app
        .npcs
        .iter()
        .any(|n| n.hp > 0 && n.kind == NpcKind::Golem)
        || !app.world.hard_mode
        || !app.world.downed_plant_boss;
    if !gated {
        // The inventory scan: the first 1293 row decrements; a zero
        // stack clears (`SetDefaults(0)`) exactly like the C#.
        for slot in 0..app.inventory.len() {
            if app.inventory[slot].kind == 1293 && app.inventory[slot].count > 0 {
                app.inventory[slot].count -= 1;
                if app.inventory[slot].count == 0 {
                    app.inventory[slot].kind = 0;
                }
                app.audio.play("Roar_0");
                spawn_golem_on_altar(app);
                break;
            }
        }
    }
    true
}

/// `NPC.SpawnOnPlayer`'s case-245 arm (`NPC.cs:80475-80531`): the
/// ±20-tile scan around the player for the altar's origin cell
/// (`type == 237 && frameX == 18 && frameY == 0`), the 50-tile solid
/// scans up and down that column (`WorldGen.SolidTile`'s
/// active/!solidTop/!halfBrick/slope-0 shape — an out-of-bounds cell
/// reads solid, the C# `null` case), then the Golem lands at the
/// column's `(up + down) / 3` pixel Y, targeting the player, with the
/// "HasAwoken" announcement. Returns false when no altar is in range
/// (the C#'s try/catch no-spawn path).
fn spawn_golem_on_altar(app: &mut App) -> bool {
    let Some(player) = app.player.as_ref() else {
        return false;
    };
    let (pcx, pcy) = player.center();
    let cx_t = (pcx / TILE_PX) as i32;
    let cy_t = (pcy / TILE_PX) as i32;
    let mut altar: Option<(i32, i32)> = None;
    for j in (cx_t - 20)..=(cx_t + 20) {
        for k in (cy_t - 20)..=(cy_t + 20) {
            if app
                .world
                .tile(j, k)
                .is_some_and(|t| t.active() && t.kind == 237 && t.frame_x == 18 && t.frame_y == 0)
            {
                altar = Some((j, k));
            }
        }
    }
    let Some((ax, ay)) = altar else {
        return false;
    };
    let solid = |j: i32, k: i32| -> bool {
        match app.world.tile(j, k) {
            None => true,
            Some(t) => {
                t.active()
                    && tr_core::tiledata::TILE_SOLID.has(t.kind)
                    && !tr_core::tiledata::TILE_SOLID_TOP.has(t.kind)
                    && !t.half_brick()
                    && t.slope() == 0
            }
        }
    };
    let mut up = ay - 15;
    for k in ((ay - 49).max(0)..=ay).rev() {
        if solid(ax, k) {
            up = k;
            break;
        }
    }
    let mut down = ay - 15;
    for k in ay..(ay + 50) {
        if solid(ax, k) {
            down = k;
            break;
        }
    }
    let spawn_x = ax as f32 * TILE_PX + 8.0;
    let spawn_y = ((up + up + down) / 3) as f32 * TILE_PX;
    let mut golem = Npc::new(NpcKind::Golem, spawn_x, spawn_y, 1, 0.0);
    golem.hp = golem.max_hp;
    app.npcs.push(golem);
    app.chat
        .new_text("The Golem has awoken!", crate::chat::COLOR_SERVER_MESSAGE);
    true
}

fn interact_tile(app: &mut App, tx: i32, ty: i32) -> bool {
    // The Lihzahrd Altar arm (`TileInteractionsUse`'s `type == 237`
    // branch, `Player.cs:32755-32790`): the right-click on the altar
    // consumes the click and, with a Power Cell 1293 in the INVENTORY
    // (the vanilla scans the inventory, not the held item), no living
    // Golem, hardmode and Plantera down, summons the boss. The arm runs
    // before the furniture dispatch — the altar has no `interact_at`
    // interaction of its own.
    if app
        .world
        .tile(tx, ty)
        .is_some_and(|t| t.active() && t.kind == 237)
    {
        return use_golem_altar(app);
    }
    let direction = app.player.as_ref().map(|p| p.facing).unwrap_or(1);
    // The entity half of `Collision.EmptyTile` (`Collision.cs:1359`): the
    // closing column must be free of the player and NPCs. Disjoint field
    // borrows — the world write happens inside `interact_at`.
    let (player, npcs) = (&app.player, &app.npcs);
    let cell_blocked = |x: i32, y: i32| cell_blocked_by_entity(player.as_ref(), npcs, x, y);
    let result = edit::interact_at(&mut app.world, tx, ty, direction, &cell_blocked);
    // The furniture interactions that touch the player/world state ahead of
    // the cue/drop mapping.
    if let edit::TileInteraction::BankOpen { bank } = result {
        // The bank tiles ride the same shared open block the world chests
        // use (`Player.cs:33230-33252` via `chest::toggle_open`, the
        // M3-CHEST-INT unify): the re-click closes (`chest = -1`, sound 11),
        // a fresh open plays 10, a switch plays 12. `OpenChest` rides inside
        // — `chest`/`chestX`/`chestY` (the C# origin decode,
        // `chest::bank_open_origin`) and `Main.playerInventory = true`
        // (`:33516-33525`); the close keeps the inventory open (the C#
        // never touches it here — the STORE-1 pair-close deviation is
        // gone). The prologue's sign/talk/chat clears (:33087-33096) are
        // the shop close, the world route's mapping (chest.rs:525).
        app.close_shop();
        let (ox, oy) = crate::chest::bank_open_origin(&app.world, tx, ty);
        let sound = crate::chest::toggle_open(app, ox, oy, bank);
        crate::chest::play_menu_sound(app, sound);
        app.world_key = None;
        return true;
    }
    match &result {
        edit::TileInteraction::ChairSit {
            seat_x,
            seat_y,
            direction,
            seat_offset,
        } => {
            // `Player.cs:32313-32330`: the 40-px snap gate
            // (`IsWithinSnappngRangeToTile`) wraps the `sitting.SitDown`
            // call; a refused snap still consumes the click.
            if let Some(p) = app.player.as_mut() {
                if within_snapping_range(p, tx, ty, 40.0) {
                    p.sit_down(&app.world, *seat_x, *seat_y, *direction, *seat_offset);
                }
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
                        p.spawn_y =
                            *spawn_y as f32 * tr_core::TILE_PX - crate::player::HEIGHT as f32;
                    }
                    app.chat
                        .new_text("Spawn point set!", crate::chat::COLOR_SERVER_MESSAGE);
                }
            }
            // The sleep half arrives as `BedSleep` below — this arm is the
            // spawn toggle (and the consumed-but-inert decode fallback).
            app.world_key = None;
            return true;
        }
        edit::TileInteraction::BedSleep {
            anchor_x,
            anchor_y,
            direction,
            bed_style,
        } => {
            // `Player.cs:32669-32677`: the 96-px snap gate wraps
            // `sleeping.StartSleeping`; a refused snap still consumes the
            // click.
            if let Some(p) = app.player.as_mut() {
                if within_snapping_range(p, tx, ty, 96.0) {
                    p.sleep_down(&app.world, *anchor_x, *anchor_y, *direction, *bed_style);
                }
            }
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

/// `Player.IsWithinSnappngRangeToTile` (`Player.cs:17365-17372`): the tile
/// centre to `(Center.X, Bottom.Y - 16)` distance — the sit (40 px) and
/// sleep (96 px) snap gates.
fn within_snapping_range(p: &crate::player::Player, tx: i32, ty: i32, distance: f32) -> bool {
    let cx = tx as f32 * TILE_PX + 8.0;
    let cy = ty as f32 * TILE_PX + 8.0;
    let dx = cx - (p.x + crate::player::WIDTH * 0.5);
    let dy = cy - (p.y + p.height - 16.0);
    (dx * dx + dy * dy).sqrt() <= distance
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
        edit::TileInteraction::BedSleep { .. } => (None, Vec::new()),
        edit::TileInteraction::ClockRead { .. } => (None, Vec::new()),
        // The bank open is consumed in `interact_tile` ahead of this
        // mapping (the SoundID 10/11/12 keys are absent from the cue set).
        edit::TileInteraction::BankOpen { .. } => (None, Vec::new()),
        // M7-EXTRACT-1: the extractinator is NOT a click-edge interaction in
        // vanilla — `PlaceThing_ItemInExtractinator` runs inside the held-item
        // swing (`Player.cs:39591`, `:41665`), so the press edge falls through
        // to the place/swing block, whose `world_sim/extract.rs` arm owns the
        // extraction (the held-item gate, the roll, the drop, the consumption).
        // Returning `None` here keeps `interact_tile`'s "not consumed" answer
        // and the chest/shop/door seam untouched.
        edit::TileInteraction::Extractinator => return None,
    };
    let events = object_drops
        .iter()
        .filter_map(|d| registry::tile_drop(d.tile_kind, 0).map(|id| (id, d.x, d.y)))
        .collect();
    Some((cue, events))
}

/// `Player.CheckSpawn` (`Player.cs:54533-54590`) without the messages — the
/// checks the revive path and the load rebuild share (the C# runs the same
/// `CheckSpawn` inside `Player.Spawn`, `:37571-37575`). `Err` carries the
/// failure reason the chat wrapper prints.
pub(crate) fn bed_spawn_valid(
    world: &tr_core::World,
    x: i32,
    y: i32,
) -> Result<(), BedSpawnFailure> {
    if x < 10 || x > world.width as i32 - 10 || y < 10 || y > world.width as i32 - 10 {
        return Err(BedSpawnFailure::Bounds);
    }
    if !world
        .tile(x, y - 1)
        .is_some_and(|t| t.active() && t.kind == 79)
    {
        return Err(BedSpawnFailure::Gone);
    }
    for i in x - 1..=x + 1 {
        for j in y - 3..y {
            if let Some(t) = world.tile(i, j) {
                // `CheckSpawn`'s `tileSolid[379]` window (`Player.cs:54535`,
                // the frame toggle `Main.cs:9468`/`:17753`/`:17799`): the
                // candelabra is solid for the whole check — the generated
                // `TILE_SOLID` already carries that base state.
                if (t.active() || t.has_any_liquid())
                    && td::TILE_SOLID.has(t.kind)
                    && !td::TILE_SOLID_TOP.has(t.kind)
                {
                    return Err(BedSpawnFailure::Obstructed);
                }
            }
        }
    }
    match tr_core::housing::check_room(world, x, y - 1) {
        Ok(_) => Ok(()),
        Err(reason) => Err(BedSpawnFailure::Room(reason)),
    }
}

/// The `CheckSpawn` failure kinds — the chat wrapper maps them to the
/// `Game.Bed*` texts (`Player.cs:54566`/`:54573-54587`).
pub(crate) enum BedSpawnFailure {
    Bounds,
    Gone,
    Obstructed,
    Room(tr_core::housing::RoomFailure),
}

/// `Player.CheckSpawn` (`Player.cs:54533-54590`): the bounds (the C# checks
/// `y` against `maxTilesX` — mirrored), the bed tile above the spawn cell,
/// the 3x3 area above free of solid non-solid-top tiles, and the room check.
/// The failure messages ride the chat like `Main.NewText`.
pub(crate) fn check_bed_spawn(
    world: &tr_core::World,
    x: i32,
    y: i32,
    chat: &mut crate::chat::ChatMonitor,
) -> bool {
    match bed_spawn_valid(world, x, y) {
        Ok(()) => true,
        Err(BedSpawnFailure::Obstructed) => {
            chat.new_text("The bed is obstructed.", crate::chat::COLOR_SERVER_MESSAGE);
            false
        }
        Err(BedSpawnFailure::Room(reason)) => {
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
        Err(_) => false,
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
        let mut npc =
            crate::npc::Npc::spawn_slime_variant(&app.world, 30, crate::npc::SlimeVariant::Blue)
                .expect("the floor row spawns the slime");
        npc.x = 30.0 * TILE_PX;
        npc.y = 260.0 * TILE_PX;
        app.npcs.push(npc);
        app
    }

    /// The bed rig: a floor row at y 60 with a style-0 bed placed at
    /// (40, 59) — columns 39-42, rows 58-59 (`place_item_tile`'s y is the
    /// bottom row) — and the player parked near the sleep anchor.
    fn bed_app() -> App {
        let mut w = World::new("t".into(), 80, 200);
        w.frame_important = vec![false; tr_core::tiledata::TILE_COUNT];
        for x in 0..80 {
            let i = w.index(x, 60);
            w.tiles[i] = Tile {
                kind: 1,
                ..Default::default()
            };
            w.tiles[i].flags |= FLAG_ACTIVE;
        }
        w.compute_column_surface();
        assert!(edit::place_item_tile(&mut w, 40, 59, 79, 0, -1));
        let mut app = flow_app(None, vec![], vec![]);
        app.world = w;
        let mut player = Player::spawn(&app.world);
        // The sleep anchor for the clicked top-half cell (41, 58)
        // (`GetSleepingTargetInfo`): column `41 - 36/18 + 1 = 40`, row `59`
        // → `(648, 960)` — the player's bottom starts 17.9 px from the tile
        // centre, inside the 96-px gate.
        player.x = 638.0;
        player.y = 918.0;
        app.player = Some(player);
        app
    }

    #[test]
    fn the_bed_sleep_half_snaps_within_96_px_and_toggles_off() {
        // DW-1.3/DW-1.6 — `Player.cs:32669-32677`: the 96-px gate wraps
        // `sleeping.StartSleeping`; the same-anchor re-click wakes
        // (`PlayerSleepingHelper.cs:167-171`); the pose rotation arms
        // (`:73-74`).
        let mut app = bed_app();
        assert!(interact_tile(&mut app, 41, 58));
        let p = app.player.as_ref().unwrap();
        assert!(p.sleeping);
        assert_eq!(p.facing, -1);
        assert_eq!(p.sleeping_style, 0);
        assert!((p.full_rotation - std::f32::consts::FRAC_PI_2).abs() < 1e-6);
        // The spawn cell stayed untouched (the sleep half is not the spawn
        // half, `IsHoveringOverABottomSideOfABed`).
        assert_eq!(app.world.bed_spawn, None);
        // The same anchor re-click wakes.
        assert!(interact_tile(&mut app, 41, 58));
        assert!(!app.player.as_ref().unwrap().sleeping);
    }

    #[test]
    fn the_bed_sleep_refuses_beyond_96_px_but_consumes_the_click() {
        // The gate at `Player.cs:32670-32676`: an out-of-range click still
        // sets `flag2 = true` (consumed, no fall-through to the place path)
        // but `StartSleeping` never runs.
        let mut app = bed_app();
        let p = app.player.as_mut().unwrap();
        p.x = 638.0 + 7.0 * 16.0; // ~130 px from the tile centre
        assert!(interact_tile(&mut app, 41, 58));
        assert!(!app.player.as_ref().unwrap().sleeping);
    }

    #[test]
    fn the_spawn_scan_counts_the_candelabra_solid() {
        // DW-1.4 — `CheckSpawn`'s `tileSolid[379]` window (`Player.cs:54535`,
        // the frame toggle `Main.cs:9468`/`:17753`/`:17799`): the candelabra
        // is solid for the whole check — one hovering inside the 3x3 box
        // above the spawn cell obstructs it (the generated `TILE_SOLID`
        // already carries that base state).
        let mut app = bed_app();
        // The spawn cell for the bottom-half click (40, 59): column
        // `39 + 2 = 41`, row `60`. The open-air room check fails on its own,
        // so the scan is isolated by its message.
        let mut chat = crate::chat::ChatMonitor::default();
        let _ = check_bed_spawn(&app.world, 41, 60, &mut chat);
        let open_air = chat.messages.last().unwrap().text.clone();
        // The candelabra at (40, 57) — inside the scan box (40..=42,
        // 57..=59), above the bed's top row.
        let i = app.world.index(40, 57);
        app.world.tiles[i] = Tile {
            kind: 379,
            ..Default::default()
        };
        app.world.tiles[i].flags |= FLAG_ACTIVE;
        let mut chat = crate::chat::ChatMonitor::default();
        assert!(!check_bed_spawn(&app.world, 41, 60, &mut chat));
        assert_eq!(chat.messages.last().unwrap().text, "The bed is obstructed.");
        assert_ne!(open_air, "The bed is obstructed.");
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
            prefix: 0,
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
    /// The shooter rig: the Wooden Bow held, the arrows in the ammo slots
    /// (and optionally the grid), the cursor to the right of the player.
    fn held_use_ammo_dbg(app: &App) -> i32 {
        crate::registry::get(app.inventory[app.selected].kind).map_or(0, |d| d.use_ammo)
    }

    fn shooter_app() -> App {
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
        // The Cobalt Repeater (auto_reuse, useAmmo 40, shoot 15): the press
        // fires without the release edge (`Player.cs`'s autoReuse gate).
        app.inventory[app.selected] = crate::item::ItemSlot {
            kind: 435,
            name: "Cobalt Repeater",
            count: 1,
            prefix: 0,
        };
        app.mine_down = true;
        app
    }

    #[test]
    fn the_bow_pulls_ammo_from_the_ammo_slots_first() {
        // `AmmoSlotOrder_Default` (`Player.cs:57141`): the ammo slots 54-57
        // precede the grid — the bow fires the ammo-slot stack, not the
        // grid's.
        let mut app = shooter_app();
        app.equip.ammo[0] = crate::item::ItemSlot {
            kind: 40,
            name: "Wooden Arrow",
            count: 7,
            prefix: 0,
        };
        app.inventory[20] = crate::item::ItemSlot {
            kind: 40,
            name: "Wooden Arrow",
            count: 99,
            prefix: 0,
        };
        let mut inv = app.inventory;
        let mut equip = std::mem::take(&mut app.equip);
        let pick = crate::ammo::pick(&inv, &equip, 40, None).unwrap();
        assert_eq!(pick, crate::ammo::AmmoSource::Ammo(0));
        crate::ammo::drain(pick, &mut inv, &mut equip);
        app.inventory = inv;
        app.equip = equip;
        assert_eq!(app.equip.ammo[0].count, 6, "the ammo slot drained");
        assert_eq!(app.inventory[20].count, 99, "the grid untouched");
    }

    #[test]
    fn the_bow_without_ammo_plays_a_dry_swing() {
        // `canShoot = false` (`Player.cs:53462-53466`): no ammo — the use
        // plays out (the swing starts) but nothing spawns and nothing drains.
        let mut app = shooter_app();
        let before = app.inventory[app.selected].count;
        // The swing start runs through the mine_down pass; the cursor sits
        // 80 px right of the screen centre (within the 6-tile reach).
        let (sw, sh) = app.screen_size();
        app.mouse_pos = Vec2::new(sw * 0.5 + 80.0, sh * 0.5);
        let screen = Vec2::new(sw, sh);
        update_editing(&mut app, 1.0 / 60.0, screen);
        // The swing started (the timers set).
        assert!(app.swing > 0.0, "the swing plays");
        assert_eq!(app.swing_style, 5, "the bow's useStyle is Shoot");
        // The fire tick: no ammo — no shot, no drain.
        for _ in 0..40 {
            update_editing(&mut app, 1.0 / 60.0, screen);
        }
        assert!(app.projectiles.slots().iter().all(|p| !p.active));
        assert_eq!(app.inventory[app.selected].count, before);
    }

    #[test]
    fn the_bow_with_arrows_fires_and_drains() {
        // `ItemCheck_Shoot` (`Player.cs:47056+`): the projectile = the ammo's
        // `Item.shoot` (the Wooden Arrow's 1), the damage = the weapon's
        // + the ammo's through the class multiplier, the speed = the
        // weapon's + the ammo's `shootSpeed`.
        let mut app = shooter_app();
        app.equip.ammo[0] = crate::item::ItemSlot {
            kind: 40,
            name: "Wooden Arrow",
            count: 7,
            prefix: 0,
        };
        let (sw, sh) = app.screen_size();
        // The cursor 80 px right of the screen centre: within the reach, the
        // aim direction +x.
        app.mouse_pos = Vec2::new(sw * 0.5 + 80.0, sh * 0.5);
        let screen = Vec2::new(sw, sh);
        // The swing start + the fire tick.
        update_editing(&mut app, 1.0 / 60.0, screen);
        for _ in 0..40 {
            update_editing(&mut app, 1.0 / 60.0, screen);
        }
        let fired: Vec<_> = app
            .projectiles
            .slots()
            .iter()
            .filter(|p| p.active)
            .collect();
        assert_eq!(fired.len(), 1, "one arrow");
        assert_eq!(
            fired[0].kind, 1,
            "the ammo's Item.shoot — the WoodenArrowFriendly"
        );
        assert!(fired[0].vel.x > 0.0, "flies at the cursor");
        assert_eq!(app.equip.ammo[0].count, 6, "the arrow drained");
    }

    // ---- Phase 4 (M3-ITEM-6): the integrated shooter wire -----------------

    /// A shooter rig: the world/reach/cursor layout of `shooter_app` with an
    /// explicit HELD kind and the ammo stacks the caller names (the first tuple
    /// goes in the ammo slot, the second-to-last in the grid rows). The cursor
    /// sits 80 px right of the screen centre — within the 6-tile reach, the aim
    /// direction +x.
    fn shooter_rig(kind: u16, name: &'static str, ammo: &[(usize, u16, u32)]) -> App {
        let mut app = shooter_app();
        app.inventory[app.selected] = crate::item::ItemSlot {
            kind,
            name,
            count: 1,
            prefix: 0,
        };
        for &(i, ammo_kind, count) in ammo {
            // The ammo-slot index and the grid rows share one numbering here:
            // a slot < 8 lands in `equip.ammo`, anything else in the grid.
            let slot = crate::item::ItemSlot {
                kind: ammo_kind,
                name: crate::registry::name(ammo_kind),
                count,
                prefix: 0,
            };
            if i < 8 {
                app.equip.ammo[i] = slot;
            } else {
                app.inventory[i] = slot;
            }
        }
        let (sw, sh) = app.screen_size();
        app.mouse_pos = Vec2::new(sw * 0.5 + 80.0, sh * 0.5);
        app
    }

    /// The live projectile count — the rig never kills them, so it only grows.
    fn shots(app: &App) -> usize {
        app.projectiles.slots().iter().filter(|p| p.active).count()
    }

    /// One frame of the shot wire at the exact 60 Hz step, recording
    /// `(remaining animation ticks, live shots)` AFTER the frame whenever a shot
    /// fired in it — the tick value the roll chain's windows read
    /// (`Player.cs:4066`, `:53626`, `:53674`), which the DW-4.1 edge assertions
    /// check instead of eyeballing.
    fn step(app: &mut App, screen: Vec2, log: &mut Vec<(i32, usize)>) {
        let before = shots(app);
        update_editing(app, ONE_TICK_SECS, screen);
        let after = shots(app);
        if after > before {
            log.push(((app.swing * 60.0).round() as i32, after));
        }
    }

    /// Hold the button for `n` frames (the press IS the first frame).
    fn hold(app: &mut App, screen: Vec2, n: usize, log: &mut Vec<(i32, usize)>) {
        for _ in 0..n {
            step(app, screen, log);
        }
    }

    /// The elapsed ticks of a log's shots — the C#'s `itemAnimation` values at
    /// the corresponding ticks (the press-frame arming puts the first shot at
    /// `max - 1`).
    fn shot_ticks(log: &[(i32, usize)]) -> Vec<i32> {
        log.iter().map(|&(tick, _)| tick).collect()
    }

    /// The fixed step's per-tick ammo-mastery pass, which the `update_editing`
    /// rig does not drive: `update_player`'s grant + cycler tick
    /// (`world_sim/player.rs`, the C#'s `UpdateBuffs` → `UpdateEquips` →
    /// `Update_AmmoCycler` → `ItemCheck` order, `Player.cs:25566`/`:25633`/
    /// `:24566`/`:31827`). Tests that arm the flags or a cycling mode call it
    /// before each press.
    fn mastery_tick(app: &mut App) {
        if let Some(p) = app.player.as_mut() {
            let armor = app.equip.armor;
            let misc = app.equip.misc_equips;
            crate::world_sim::player::grant_ammo_mastery(p, &armor, &misc);
            p.ammo_cycling.tick();
        }
    }

    /// A fresh press: the rig never decays `attack_cd` (that pass lives in
    /// `world_sim/npcs.rs`, outside `update_editing`), so a test that wants a
    /// second swing clears the two locks the start block reads and lets the
    /// held button repeat (`Player.cs:42819-42822`'s autoReuse restart).
    fn repress(app: &mut App) {
        app.attack_cd = 0.0;
        app.reuse_delay = 0.0;
    }

    #[test]
    fn test_DW_4_1_the_clockwork_swing_drains_one_ammo_of_three_shots() {
        // `Player.cs:53631` + `:4066`: the Clockwork's `useTime 4 < useAnimation
        // 12` fires 3 shots per swing at the animation ticks 11/7/3 — the first
        // is "just started" (`ItemAnimationJustStarted`) and consumes, the
        // other two save, so ONE ammo drains per swing. The port's per-shot
        // `ApplyItemTime` re-arm (`Player.cs:47184-47193`) is what makes the
        // other two shots happen at all.
        let mut app = shooter_rig(434, "Clockwork Assault Rifle", &[(0, 97, 99)]);
        let screen = app.screen_size().into();
        let mut log = Vec::new();
        hold(&mut app, screen, 20, &mut log);
        // The three shot ticks — the vanilla's 11/7/3, asserted rather than
        // eyeballed (the phase's core risk is exactly this alignment).
        assert_eq!(
            shot_ticks(&log),
            vec![11, 7, 3],
            "the Clockwork's shot ticks"
        );
        assert_eq!(log.len(), 3, "3 shots per swing");
        assert_eq!(shots(&app), 3, "3 musket balls spawned");
        assert_eq!(
            app.projectiles
                .slots()
                .iter()
                .find(|p| p.active)
                .unwrap()
                .kind,
            14,
            "the Musket Ball — the ammo's own Item.shoot"
        );
        assert_eq!(app.equip.ammo[0].count, 98, "exactly one ammo drained");
    }

    #[test]
    fn test_DW_4_1_the_fairy_queen_drains_eleven_of_fifteen_shots() {
        // `Player.cs:53635`: `itemAnimation > sItem.useAnimation - 8` — with the
        // Fairy Queen's anim 30 / useTime 2 the window is the ticks 29/27/25/23
        // (the first eight), so FOUR of the 15 shots save and ELEVEN drain.
        let mut app = shooter_rig(4953, "Fairy Queen Ranged Item", &[(0, 40, 99)]);
        let screen = app.screen_size().into();
        let mut log = Vec::new();
        hold(&mut app, screen, 34, &mut log);
        assert_eq!(
            shot_ticks(&log),
            (1..=29).rev().step_by(2).collect::<Vec<_>>(),
            "15 shots at 29/27/…/1 — the vanilla's `itemAnimation` values"
        );
        assert_eq!(app.equip.ammo[0].count, 99 - 11, "eleven ammo drained");
        // The four saved shots sit exactly inside the window, the eleven
        // drained ones outside it — the window's boundary is 30 - 8 = 22.
        let inside: Vec<_> = shot_ticks(&log).into_iter().filter(|&t| t > 22).collect();
        assert_eq!(inside, vec![29, 27, 25, 23], "the free shots");
        assert_eq!(
            app.projectiles
                .slots()
                .iter()
                .find(|p| p.active)
                .map(|p| p.kind),
            Some(932),
            "the Fairy Queen's shot (`:53503-53506`, the wooden-arrow default)"
        );
    }

    #[test]
    fn test_DW_4_1_the_fairy_queen_window_edges_are_tick_exact() {
        // The window's two edges, asserted at the gate rather than inferred: a
        // whole swing's worth of shots, each one stamped with the tick the wire
        // derived it from, must partition into "saved" (inside the window) and
        // "drained" (outside) at exactly `remaining > 22`. One tick either way
        // and the drained count changes — the phase's determinism risk.
        let mut app = shooter_rig(4953, "Fairy Queen Ranged Item", &[(0, 40, 99)]);
        let screen = app.screen_size().into();
        let mut log = Vec::new();
        hold(&mut app, screen, 34, &mut log);
        // The vanilla's per-shot save pattern, re-derived from the C# order:
        // the 4953 arm has no `Main.rand` (`Player.cs:53635`), so the four
        // window shots save with certainty and the rest drain.
        assert_eq!(log.len(), 15, "the volley's shot count");
        assert_eq!(log.iter().filter(|&&(t, _)| t > 22).count(), 4);
        assert_eq!(log.iter().filter(|&&(t, _)| t <= 22).count(), 11);
        assert_eq!(99 - app.equip.ammo[0].count, 11);
    }

    #[test]
    fn test_DW_4_2_the_gun_prog_four_fire_their_fixed_projectile_and_drain_nothing() {
        // `ItemID.cs:1327` + `Player.cs:47071-47078` / `:48986-49018`: the four
        // endgame shooters require their ammo, spend no roll draws on the drain
        // (`dontConsume` seeds the chain true — their own rollers still spend
        // theirs), fire a FIXED gun projectile and carry the weapon's own RAW
        // damage/speed/knockback, the ammo fuse discarded. Their registry rows
        // are placeholders (`use_ammo: 0`), so the wire keys their vanilla data
        // on the item id.
        let cases: [(u16, &str, i32, u16, i32, f32, f32); 4] = [
            (3475, "Vortex Beater", 97, 615, 50, 20.0, 2.0),
            (3540, "Phantasm", 40, 630, 50, 20.0, 2.0),
            (3854, "DD2 Phoenix Bow", 40, 705, 32, 20.0, 2.0),
            (3930, "Celeb2", 771, 714, 50, 17.0, 10.0),
        ];
        for (kind, name, use_ammo, projectile, damage, speed, knock_back) in cases {
            let mut app = shooter_rig(kind, name, &[(0, use_ammo as u16, 99)]);
            let screen = app.screen_size().into();
            let mut log = Vec::new();
            hold(&mut app, screen, 30, &mut log);
            let fired: Vec<_> = app
                .projectiles
                .slots()
                .iter()
                .filter(|p| p.active)
                .collect();
            assert_eq!(fired.len(), 1, "{name}: one shot per swing");
            assert_eq!(
                fired[0].kind, projectile,
                "{name}: the fixed gun projectile"
            );
            assert_eq!(
                fired[0].damage, damage,
                "{name}: the weapon's own raw damage (the fuse discarded)"
            );
            assert!(
                fired[0].vel.x > 0.0 && (fired[0].vel.length() - speed).abs() < 0.001,
                "{name}: the weapon's own raw shootSpeed ({speed}) at the cursor's \
                 direction — got {:?}",
                fired[0].vel
            );
            assert_eq!(
                fired[0].knock_back, knock_back,
                "{name}: the weapon's own raw knockback"
            );
            assert_eq!(
                app.equip.ammo[0].count, 99,
                "{name}: `dontConsume` — no ammo drained"
            );
        }
    }

    #[test]
    fn test_DW_4_2_the_gun_prog_four_stay_dry_without_ammo() {
        // `Player.cs:53462-53466`'s gate survives the gunProj reset: the ammo
        // is still REQUIRED — without it `canShoot` stays false and the swing
        // plays dry (no shot, no drain, no panic).
        for (kind, name) in [
            (3475u16, "Vortex Beater"),
            (3540, "Phantasm"),
            (3854, "DD2 Phoenix Bow"),
            (3930, "Celeb2"),
        ] {
            let mut app = shooter_rig(kind, name, &[]);
            let screen = app.screen_size().into();
            let mut log = Vec::new();
            hold(&mut app, screen, 30, &mut log);
            assert!(log.is_empty(), "{name}: dry — nothing fired");
            assert_eq!(shots(&app), 0, "{name}: nothing spawned");
        }
    }

    #[test]
    fn test_DW_4_3_the_ammo_slots_mode_picks_alternately_behind_the_offset() {
        // `Player.cs:53685-53726` + `:53569-53577`: with a cycling mode granted
        // the pick rotates behind the offset, and every successful pick without
        // `dontConsume` advances it. Two arrow stacks in the ammo slots: the
        // picks alternate 0/1/0/1 as the offset grows and wraps modulo the
        // count — the drain shows which stack was picked.
        let mut app = shooter_rig(435, "Cobalt Repeater", &[(0, 40, 12), (1, 40, 12)]);
        app.equip.misc_equips[0] = crate::item::ItemSlot {
            kind: 6168, // the Ammo Cycler → AmmoSlots (`Player.cs:14528-14544`)
            name: "Ammo Cycler",
            count: 1,
            prefix: 0,
        };
        let screen = app.screen_size().into();
        let mut log = Vec::new();
        for _ in 0..4 {
            mastery_tick(&mut app);
            hold(&mut app, screen, 26, &mut log);
            repress(&mut app);
        }
        assert_eq!(log.len(), 4, "one shot per swing (useTime == useAnimation)");
        assert_eq!(
            (app.equip.ammo[0].count, app.equip.ammo[1].count),
            (12 - 2, 12 - 2),
            "the picks alternated: two drains each"
        );
    }

    #[test]
    fn test_DW_4_3_the_cycling_cooldown_expires_and_resets_the_offset() {
        // `Player.cs:53569-53575` + `Update_AmmoCycler` (`:28419-28426`): each
        // successful non-dontConsume pick arms `cooldown = useAnimation + 120`
        // and advances the offset; the per-tick pass decrements the cooldown and
        // resets the offset at zero — so an idle cycler returns to the first
        // stack instead of freezing mid-rotation.
        let mut app = shooter_rig(435, "Cobalt Repeater", &[(0, 40, 12), (1, 40, 12)]);
        app.equip.misc_equips[0] = crate::item::ItemSlot {
            kind: 6168,
            name: "Ammo Cycler",
            count: 1,
            prefix: 0,
        };
        let screen = app.screen_size().into();
        let mut log = Vec::new();
        mastery_tick(&mut app);
        hold(&mut app, screen, 26, &mut log);
        // The Cobalt Repeater's `useAnimation` 23 (`itemdata.rs` — the vanilla's
        // `Item.cs:6488-6505`), so the cycler's cooldown is 23 + 120 ticks.
        {
            let p = app.player.as_ref().unwrap();
            assert_eq!(p.ammo_cycling.cooldown, 143, "useAnimation + 120");
            assert_eq!(p.ammo_cycling.offset, 1, "the pick advanced it");
        }
        // Still counting: the offset holds while the cooldown runs out.
        for _ in 0..142 {
            app.player.as_mut().unwrap().ammo_cycling.tick();
        }
        {
            let p = app.player.as_ref().unwrap();
            assert_eq!((p.ammo_cycling.cooldown, p.ammo_cycling.offset), (1, 1));
        }
        // The expiring tick resets the offset (`Player.cs:28419-28426`).
        app.player.as_mut().unwrap().ammo_cycling.tick();
        {
            let p = app.player.as_ref().unwrap();
            assert_eq!((p.ammo_cycling.cooldown, p.ammo_cycling.offset), (0, 0));
        }
        // The reset pick therefore lands back on the FIRST stack: without the
        // reset the offset 1 would have carried the pick to the second stack
        // (`11 / 11`), so `10 / 12` — the first stack twice — is the proof.
        mastery_tick(&mut app);
        repress(&mut app);
        hold(&mut app, screen, 26, &mut log);
        assert_eq!(
            (app.equip.ammo[0].count, app.equip.ammo[1].count),
            (12 - 2, 12 - 0),
            "after the reset the first stack drains again"
        );
    }

    #[test]
    fn test_DW_4_3_the_clentaminator_always_picks_the_first_stack() {
        // `PickAmmo_IterateRange`'s Clentaminator override (`Player.cs:53708-53711`):
        // 779/5134 force `num = 0` regardless of the mode and the offset, so
        // their sprays always come out of the first matching stack. The offset
        // here is armed to a value that WOULD rotate a normal weapon to the
        // second stack (`5 % 2 == 1`).
        // The Clentaminator's own volley is anim 30 / useTime 5 (`Item.cs:10501-10516`
        // → `itemdata.rs`): 6 shots at the ticks 29/24/19/14/9/4, of which the
        // first consumes — `:53667`'s `itemAnimation < itemAnimationMax - useTime`
        // window is FALSE on the first shot (29 < 25 is false) and true for
        // every later one — so exactly ONE solution drains per swing. The
        // picked stack, not the drain count, is what this case pins: the
        // resolved solution projectile 145 has no `proj_data` row yet (M4-PROJ's
        // AI), so the spray's shots are refused at the spawn and the drain is
        // the only observable.
        for (kind, name, accessory) in [
            (779u16, "Clentaminator", 6168u16),
            (779, "Clentaminator", 6193), // the Random cycler
            (5134, "Clentaminator2", 6168),
        ] {
            let mut app = shooter_rig(kind, name, &[(0, 780, 9), (1, 780, 9)]);
            app.equip.misc_equips[0] = crate::item::ItemSlot {
                kind: accessory,
                name: "Ammo Cycler",
                count: 1,
                prefix: 0,
            };
            let screen = app.screen_size().into();
            let mut log = Vec::new();
            for _ in 0..3 {
                // The offset re-armed before every swing: a value that WOULD rotate
                // a normal weapon to the second stack (`5 % 2 == 1`).
                mastery_tick(&mut app);
                app.player.as_mut().unwrap().ammo_cycling.offset = 5;
                repress(&mut app);
                hold(&mut app, screen, 34, &mut log);
            }
            // The second stack is NEVER picked — the override forces index 0. Only
            // the first stack's count varies, because the Clentaminator2 (5134)
            // saves its own first shots with a 1/3 roll (`Player.cs:53591`, its own
            // arm) while 779 drains on every swing's first shot.
            assert_eq!(
                app.equip.ammo[1].count, 9,
                "{name} (cycler {accessory}): the second stack is never picked"
            );
            assert!(
                app.equip.ammo[0].count < 9,
                "{name} (cycler {accessory}): at least one drain came out of the \
             first stack, offset 5 notwithstanding"
            );
        }
    }

    #[test]
    fn test_DW_4_3_the_cycling_pick_reads_the_fixed_step_grant() {
        // The grant is per-tick (`Player.ResetEffects` `:18479` /
        // `UpdateEquips` `:14528-14544`): the pick rotates only once the tick's
        // scan has granted the mode, which is why the wire reads the mode off
        // the Player instead of caching it. The offset is armed by hand here so
        // the two reads differ observably with ONE pick each.
        let mut app = shooter_rig(435, "Cobalt Repeater", &[(0, 40, 12), (1, 40, 12)]);
        app.equip.misc_equips[0] = crate::item::ItemSlot {
            kind: 6168,
            name: "Ammo Cycler",
            count: 1,
            prefix: 0,
        };
        let screen = app.screen_size().into();
        // No `mastery_tick` and the offset armed to 1: the accessory is equipped
        // but the tick's scan has not run, so the mode is the `None` default and
        // the pick ignores the offset — the first stack.
        app.player.as_mut().unwrap().ammo_cycling.offset = 1;
        let mut log = Vec::new();
        hold(&mut app, screen, 26, &mut log);
        assert_eq!(
            (app.equip.ammo[0].count, app.equip.ammo[1].count),
            (11, 12),
            "no grant yet → the first matching stack, offset ignored"
        );
        // The tick's scan grants the mode and the SAME armed offset now rotates
        // the pick to the second stack.
        mastery_tick(&mut app);
        repress(&mut app);
        hold(&mut app, screen, 26, &mut log);
        assert_eq!(
            (app.equip.ammo[0].count, app.equip.ammo[1].count),
            (11, 11),
            "the granted mode rotates the pick behind the offset"
        );
    }

    #[test]
    fn test_DW_4_3_a_random_cycling_pick_spends_one_extra_draw() {
        // The determinism invariant, pinned on the stream: the `Random` mode's
        // `ammoCyclingOffset = Main.rand.Next()` (`Player.cs:53577`) is ONE extra
        // draw on the shared `item_rng`, spent BEFORE the shot's own crit roll
        // (the C# updates the cycler inside `PickAmmo`, the roll arms after it).
        // The Cobalt Repeater carries no roll arm of its own and the ammo has no
        // flags, so its whole per-shot draw count is the cycler's one plus the
        // crit's one — asserted against a helper that advances the same splitmix
        // round, so a later refactor cannot silently add or drop a draw.
        let draws_after = |seed: u64, n: usize| {
            let mut state = seed;
            for _ in 0..n {
                crate::app::next_rand(&mut state);
            }
            state
        };
        // The same shot without the cycler: the crit roll's single draw.
        let mut plain = shooter_rig(435, "Cobalt Repeater", &[(0, 40, 12)]);
        let seed = 0x5EED_CA5Eu64;
        plain.item_rng = seed;
        let screen = plain.screen_size().into();
        let mut log = Vec::new();
        hold(&mut plain, screen, 26, &mut log);
        assert_eq!(log.len(), 1, "one shot");
        assert_eq!(
            plain.item_rng,
            draws_after(seed, 1),
            "the plain shot spends exactly the crit's draw"
        );
        // With the Random cycler: the cycler's draw comes first, the crit's
        // second — two rounds in total, in that order.
        let mut random = shooter_rig(435, "Cobalt Repeater", &[(0, 40, 12)]);
        random.equip.misc_equips[0] = crate::item::ItemSlot {
            kind: 6193, // the Ammo Cycler (Random) (`Player.cs:14538`)
            name: "Ammo Cycler",
            count: 1,
            prefix: 0,
        };
        random.item_rng = seed;
        let screen = random.screen_size().into();
        mastery_tick(&mut random);
        let mut log = Vec::new();
        hold(&mut random, screen, 26, &mut log);
        assert_eq!(log.len(), 1, "one shot");
        assert_eq!(
            random.item_rng,
            draws_after(seed, 2),
            "the Random cycler adds exactly one draw ahead of the crit roll"
        );
        assert_eq!(
            random.player.as_ref().unwrap().ammo_cycling.offset,
            (crate::app::next_rand(&mut { seed }) as u32 as i32) & i32::MAX,
            "the offset is that first draw, masked to .NET's non-negative range"
        );
    }

    // ---- the dirty test-plan rows -----------------------------------------

    #[test]
    fn test_dirty_a_volley_swing_without_ammo_stays_dry_and_spends_nothing() {
        // A volley shooter with no ammo: `PickAmmo` finds nothing on every tick
        // of the swing, `ItemCheck_Shoot` returns before `ApplyItemTime`
        // (`Player.cs:47147`), so `itemTime` stays zero and the gate re-runs
        // each frame — and NOTHING is drawn: no resolve, no roll, no crit, no
        // drain. The RNG stream's immobility is the assertion (the determinism
        // invariant).
        let mut app = shooter_rig(434, "Clockwork Assault Rifle", &[]);
        let rng_before = app.item_rng;
        let screen = app.screen_size().into();
        let mut log = Vec::new();
        update_editing(&mut app, ONE_TICK_SECS, screen);
        assert!(app.swing > 0.0, "the swing still played out");
        hold(&mut app, screen, 30, &mut log);
        assert!(log.is_empty(), "nothing fired");
        assert_eq!(shots(&app), 0);
        assert_eq!(app.item_rng, rng_before, "the stream is untouched");
    }

    #[test]
    fn test_dirty_a_random_cycling_pick_with_no_stacks_is_refused() {
        // `PickAmmo_IterateRange`'s `num2 == 0 → null` (`Player.cs:53702-53705`):
        // a cycler armed to Random with no matching stack anywhere refuses the
        // shot — no panic, no phantom pick, no drain, and NOT a zero-division
        // crash at `offset % count`.
        let mut app = shooter_rig(435, "Cobalt Repeater", &[]);
        app.equip.misc_equips[0] = crate::item::ItemSlot {
            kind: 6193, // the Ammo Cycler (Random) (`Player.cs:14538`)
            name: "Ammo Cycler",
            count: 1,
            prefix: 0,
        };
        mastery_tick(&mut app);
        let rng_before = app.item_rng;
        let screen = app.screen_size().into();
        let mut log = Vec::new();
        hold(&mut app, screen, 30, &mut log);
        assert!(log.is_empty(), "no shot");
        assert_eq!(app.item_rng, rng_before, "and no draw spent");
    }

    #[test]
    fn test_dirty_a_resolved_kind_without_a_proj_data_row_refuses_the_shot() {
        // The `proj_data` trap `projectile.rs` documents: a kind with no row
        // makes `spawn_with_data` decline silently. The Cobalt Repeater with
        // the Frostburn Arrow (988) resolves to projectile 172, which has no
        // row yet — its AI is M4-PROJ's — and the wire must refuse the shot
        // without panicking, while the ammo it already picked still drains (the
        // drain runs before the spawn, `Player.cs:53675` before the spawn).
        let mut app = shooter_rig(435, "Cobalt Repeater", &[(0, 988, 9)]);
        let screen = app.screen_size().into();
        let mut log = Vec::new();
        hold(&mut app, screen, 26, &mut log);
        assert_eq!(shots(&app), 0, "the spawn declined the unknown kind");
        assert_eq!(app.equip.ammo[0].count, 8, "the picked arrow still drained");
    }

    #[test]
    fn test_dirty_the_acorn_slingshot_drains_past_the_ammo_box() {
        // `Player.cs:53599` (the Ammo Box's 1/5 save) vs `:53671` (the Acorn
        // Slingshot's `flag = false`, the chain's LAST arm): with the Ammo Box
        // buff on every swing drains exactly one acorn, no matter how many times
        // the box's roll would have saved. The wire's `ammo_flags` read is what
        // makes the box's roll reachable at all.
        // The Acorn Slingshot's resolved projectile 1088 (its own `Item.shoot`,
        // the Acorn carrying none) has a row (`Projectile.cs:10145-10149`), so
        // the ten shots fire AND the ten acorns drain — the spawn is not the
        // observable any more, the drain past the box's saves is.
        let mut app = shooter_rig(5629, "Acorn Slingshot", &[(0, 27, 12)]);
        app.player.as_mut().unwrap().buffs.add_buff(93, 3600);
        let screen = app.screen_size().into();
        let mut log = Vec::new();
        for _ in 0..10 {
            mastery_tick(&mut app);
            hold(&mut app, screen, 26, &mut log);
            repress(&mut app);
        }
        assert_eq!(shots(&app), 10, "the acorn spawns — its row exists");
        assert_eq!(
            app.equip.ammo[0].count, 2,
            "ten acorns drained — the box never saves for the Acorn Slingshot"
        );
    }

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

    /// The whip use path (M4-PROJ-7): one press = one swing, the spawn carries
    /// the tag row, the whip never self-crits, and the swing lasts the item's
    /// `useAnimation`.
    fn whip_app(kind: u16) -> App {
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
        app.inventory[app.selected] = crate::item::ItemSlot {
            kind,
            name: "whip",
            count: 1,
            prefix: 0,
        };
        // The press edge is the test's: `mine_down` flips to true on the
        // frame under test (`Player.cs:42733`'s release-then-press gate).
        app.mine_down = false;
        app.mine_released = true;
        app
    }

    #[test]
    fn the_whip_press_swings_once_and_spawns_the_tagged_slot() {
        // The BlandWhip (4672 → 841): `autoReuse = false`, `useAnimation` 30,
        // `DefaultToWhip`'s slice, `crit` never rolled for the whip itself.
        let mut app = whip_app(4672);
        // One idle frame sets the edge state (`mine_prev_down` = false,
        // `mine_released` = true); the press on the next frame fires.
        mine_frames(&mut app, 1);
        app.mine_down = true;
        mine_frames(&mut app, 1);
        let spawned: Vec<_> = app
            .projectiles
            .slots()
            .iter()
            .filter(|p| p.active)
            .collect();
        assert_eq!(spawned.len(), 1, "one whip projectile");
        let p = spawned[0];
        assert_eq!(p.kind, 841);
        assert_eq!(p.tag_effect_type, 4672, "the tag row rides the slot");
        assert!(!p.crit, "the whip never self-crits");
        assert_eq!(p.local_hit_cooldown, -1, "the one-hit-per-NPC sentinel");
        // `timeToFlyOut = itemAnimationMax(30) * MaxUpdates(2)`.
        assert_eq!(
            p.whip_settings.map(|s| s.time_fly_out),
            Some(60.0),
            "the swing lives the useAnimation × MaxUpdates"
        );
        // The swing timer matches the item's `useAnimation` (30 ticks = 0.5 s).
        assert!((app.swing_max - 0.5).abs() < 1e-6, "the 30-tick swing");
        // The reuse lock: a held press does not throw a second whip.
        app.mine_released = false;
        mine_frames(&mut app, 60);
        let still = app.projectiles.slots().iter().filter(|p| p.active).count();
        assert!(still <= 1, "one swing per press edge (the past whip ended)");
    }

    #[test]
    fn the_prefixed_minion_tag_damage_reaches_the_slot() {
        // The minion prefixes 69-72 add `Item.bonusTagDamage`
        // (`Item.cs:575-598`); `prefix.rs` carries the row, the fire path
        // copies it onto the slot (`ApplyStatsFromSource`'s `:11127`).
        let mut app = whip_app(4678);
        app.inventory[app.selected].prefix = 71; // a minion prefix with tag damage
        mine_frames(&mut app, 1);
        app.mine_down = true;
        mine_frames(&mut app, 1);
        let p = app
            .projectiles
            .slots()
            .iter()
            .find(|p| p.active)
            .expect("the SwordWhip");
        let want = crate::prefix::prefixed_stats(4678, 71).unwrap().tag_damage;
        assert_eq!(p.bonus_tag_damage, want);
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
        assert!(
            app.world.tile(10, 20).unwrap().active(),
            "the support refuses"
        );
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
            slots: vec![crate::item::ItemSlot {
                kind: 9,
                name: crate::registry::name(9),
                count: 6,
                prefix: 0,
            }],
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
        assert!(
            !app2.world.tile(10, 19).unwrap().active(),
            "nothing written"
        );
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
        assert!(
            (1..=3).contains(&drops[1].1),
            "seeds 1-3, got {}",
            drops[1].1
        );
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
        assert!(is_alchemy_plant_harvestable(
            4,
            100,
            &herb_ctx(true, 0, false, true, 0.0, 0.0)
        ));
        assert!(is_alchemy_plant_harvestable(
            4,
            100,
            &herb_ctx(true, 0, false, false, 0.5, 0.0)
        ));
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
        assert_eq!(
            coin_value(&loot),
            num17 as i64,
            "coin total == replayed num17"
        );
    }

    // DW-3.3: the style multiplier drives the luck range and the coin scale.
    #[test]
    fn test_DW_3_3_style_multiplier_range_and_coins() {
        let close = |a: f32, b: f32| (a - b).abs() < 1e-6;
        // `num = (num * 2 + 1) / 3` after the band table.
        assert!(close(pot_style_multiplier(0, false), 1.0));
        assert!(close(
            pot_style_multiplier(4, false),
            (1.25 * 2.0 + 1.0) / 3.0
        ));
        assert!(close(pot_style_multiplier(25, false), 7.0), "10 → 21/3");
        assert!(
            close(pot_style_multiplier(28, false), 1.0),
            "hardmode-gated"
        );
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
        for cents in [
            1.0f32,
            50.0,
            51.0,
            99.0,
            100.0,
            5_050.0,
            123_456.0,
            2_500_000.0,
        ] {
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
        assert_eq!(
            run(),
            run(),
            "identical seeds and actions → identical drops"
        );
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
        assert!(
            !use_tile_item(&mut app, 10, 18),
            "no placement after the kill"
        );
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
    // ---- M3-STORE-1: the bank open toggle + the void-vault pickup -------

    #[test]
    fn test_store_1_the_piggy_right_click_toggles_the_bank() {
        // `Player.cs:33122-33138` + the shared open block (:33230-33252, the
        // M3-CHEST-INT unify routes it through `chest::toggle_open`): the
        // right-click opens chest -2 (the piggy), the re-click closes to -1;
        // the inventory opens with the bank (`OpenChest`'s
        // `Main.playerInventory = true`, `:33517`) and STAYS open on the
        // chest close — the C# close arm never touches `playerInventory`
        // (the world-chest panel's close behaves the same; the STORE-1
        // pair-close deviation is gone). The furniture interact tests
        // drive `interact_tile` directly (the route's press edge is latched
        // at `update_editing`'s head).
        let mut w = object_world();
        // The piggy rides ON a table: the table row at 20, the piggy at 19.
        set_tile(&mut w, 10, 20, 14);
        set_tile(&mut w, 11, 20, 14);
        set_tile(&mut w, 12, 20, 14);
        set_tile(&mut w, 10, 19, 29);
        set_tile(&mut w, 11, 19, 29);
        w.frame_important[29] = true;
        if let Some(t) = w.tile_mut(10, 19) {
            t.frame_x = 0;
        }
        if let Some(t) = w.tile_mut(11, 19) {
            t.frame_x = 18;
        }
        let mut app = flow_app(None, vec![], vec![]);
        app.world = w;
        app.player = Some(Player::spawn(&app.world));
        // The first click opens chest -2.
        assert!(interact_tile(&mut app, 10, 19));
        let p = app.player.as_ref().unwrap();
        assert_eq!(p.chest, -2, "the piggy opens chest -2");
        assert!(app.inventory_open, "the bank opens the inventory");
        assert_eq!(
            (p.chest_x, p.chest_y),
            (10, 19),
            "OpenChest sets the origin for the banks too (:33524-33525)"
        );
        // The re-click closes; the inventory stays open (C# 1:1).
        assert!(interact_tile(&mut app, 10, 19));
        assert_eq!(
            app.player.as_ref().unwrap().chest,
            -1,
            "the re-click closes"
        );
        assert!(app.inventory_open, "the close keeps the inventory open");
        assert_eq!(
            (
                app.player.as_ref().unwrap().chest_x,
                app.player.as_ref().unwrap().chest_y
            ),
            (10, 19),
            "the close keeps chestX/Y (:33236)"
        );
    }

    /// A container tile with its sheet frame set (the 2×2 chest shape's
    /// corners — `chest.rs`'s `place_chest` test fixture).
    fn set_container_tile(w: &mut World, x: i32, y: i32, kind: u16, fx: i16, fy: i16) {
        let i = w.index(x, y);
        w.tiles[i] = Tile {
            kind,
            ..Default::default()
        };
        w.tiles[i].flags |= FLAG_ACTIVE;
        w.tiles[i].frame_x = fx;
        w.tiles[i].frame_y = fy;
    }

    /// The world-position of tile `(x, y)`'s centre (the
    /// `chest::try_open_chest_at_cursor` route's `(px / TILE_PX).floor()`).
    fn world_click(x: i32, y: i32) -> Vec2 {
        Vec2::new(x as f32 * TILE_PX + 8.0, y as f32 * TILE_PX + 8.0)
    }

    /// DW-1.1 — opening a bank closes an open world chest (`Player.chest`,
    /// `Player.cs:2876`, the M3-CHEST-INT unify): the world chest opened by
    /// the right-click route (`chest::try_open_chest_at_cursor`, store index
    /// 0) is replaced when the bank tile's click runs the shared open block
    /// — the cross-close is free with one field and one open path.
    #[test]
    fn test_DW_1_1_opening_a_bank_closes_the_world_chest() {
        let mut w = object_world();
        // A world chest 2×2 at (30, 15) — the frame corners 0/18.
        set_container_tile(&mut w, 30, 15, 21, 0, 0);
        set_container_tile(&mut w, 31, 15, 21, 18, 0);
        set_container_tile(&mut w, 30, 16, 21, 0, 18);
        set_container_tile(&mut w, 31, 16, 21, 18, 18);
        // The piggy on its table row (the store-1 fixture shape).
        set_tile(&mut w, 10, 20, 14);
        set_tile(&mut w, 11, 20, 14);
        set_tile(&mut w, 12, 20, 14);
        set_container_tile(&mut w, 10, 19, 29, 0, 0);
        set_container_tile(&mut w, 11, 19, 29, 18, 0);
        w.frame_important[29] = true;
        let mut app = flow_app(None, vec![], vec![]);
        app.world = w;
        app.player = Some(Player::spawn(&app.world));

        // The world chest opens (store index 0, the lazy entry).
        assert!(crate::chest::try_open_chest_at_cursor(
            &mut app,
            world_click(30, 15)
        ));
        let p = app.player.as_ref().unwrap();
        assert_eq!(p.chest, 0, "the world chest is open");
        assert_eq!((p.chest_x, p.chest_y), (30, 15));

        // The piggy's click runs the shared block through the bank arm: the
        // world chest is replaced by the bank — one field, no second open
        // container.
        assert!(interact_tile(&mut app, 10, 19));
        let p = app.player.as_ref().unwrap();
        assert_eq!(p.chest, -2, "the bank replaced the world chest");
        assert_eq!((p.chest_x, p.chest_y), (10, 19), "the origin moved too");
        assert!(app.inventory_open, "the bank opened with the inventory");
    }

    /// DW-1.1 — the reverse: opening a world chest closes an open bank.
    #[test]
    fn test_DW_1_1_opening_a_world_chest_closes_the_bank() {
        let mut w = object_world();
        set_container_tile(&mut w, 30, 15, 21, 0, 0);
        set_container_tile(&mut w, 31, 15, 21, 18, 0);
        set_container_tile(&mut w, 30, 16, 21, 0, 18);
        set_container_tile(&mut w, 31, 16, 21, 18, 18);
        set_tile(&mut w, 10, 20, 14);
        set_tile(&mut w, 11, 20, 14);
        set_tile(&mut w, 12, 20, 14);
        set_container_tile(&mut w, 10, 19, 29, 0, 0);
        set_container_tile(&mut w, 11, 19, 29, 18, 0);
        w.frame_important[29] = true;
        let mut app = flow_app(None, vec![], vec![]);
        app.world = w;
        app.player = Some(Player::spawn(&app.world));

        // The bank opens first.
        assert!(interact_tile(&mut app, 10, 19));
        assert_eq!(app.player.as_ref().unwrap().chest, -2);
        // The world chest's right-column click: the origin decodes to
        // (30, 15), the bank closes (the shared block's `num76 != chest`
        // switch arm).
        assert!(crate::chest::try_open_chest_at_cursor(
            &mut app,
            world_click(31, 15)
        ));
        let p = app.player.as_ref().unwrap();
        assert_eq!(p.chest, 0, "the world chest replaced the bank");
        assert_eq!((p.chest_x, p.chest_y), (30, 15));
    }

    /// DW-1.3 — the bank open/close/re-click path goes through
    /// `chest::toggle_open` (`Player.cs:33230-33252`, the shared block): the
    /// fresh open sets `chest`/`chestX`/`chestY` — the C# origin decode lands
    /// on the piggy's left column even when the RIGHT tile is clicked
    /// (`Player.cs:33060-33079`) — and the inventory opens with it; the
    /// re-click closes (the inventory stays open — the C# close arm never
    /// touches `playerInventory`); a second bank's click switches (the
    /// sound-12 arm). The sound ids themselves are pinned by
    /// `chest::tests::test_DW_1_3_toggle_open_returns_the_shared_block_sound_ids`
    /// (`toggle_open`'s return contract; this arm plays them through
    /// `play_menu_sound`).
    #[test]
    fn test_DW_1_3_bank_clicks_route_through_toggle_open() {
        let mut w = object_world();
        // The piggy on its table row.
        set_tile(&mut w, 10, 20, 14);
        set_tile(&mut w, 11, 20, 14);
        set_tile(&mut w, 12, 20, 14);
        set_container_tile(&mut w, 10, 19, 29, 0, 0);
        set_container_tile(&mut w, 11, 19, 29, 18, 0);
        w.frame_important[29] = true;
        // A safe (97) 2×2 elsewhere — the switch target.
        set_container_tile(&mut w, 20, 17, 97, 0, 0);
        set_container_tile(&mut w, 21, 17, 97, 18, 0);
        set_container_tile(&mut w, 20, 18, 97, 0, 18);
        set_container_tile(&mut w, 21, 18, 97, 18, 18);
        w.frame_important[97] = true;
        let mut app = flow_app(None, vec![], vec![]);
        app.world = w;
        app.player = Some(Player::spawn(&app.world));

        // The RIGHT piggy tile's click: the origin decodes to (10, 19).
        assert!(interact_tile(&mut app, 11, 19));
        let p = app.player.as_ref().unwrap();
        assert_eq!(p.chest, -2, "the piggy opens chest -2");
        assert_eq!(
            (p.chest_x, p.chest_y),
            (10, 19),
            "the C# origin decode (:33060-33079)"
        );
        assert!(
            app.inventory_open,
            "OpenChest's playerInventory = true (:33517)"
        );
        // The re-click closes; the inventory stays open.
        assert!(interact_tile(&mut app, 10, 19));
        assert_eq!(
            app.player.as_ref().unwrap().chest,
            -1,
            "the re-click closes"
        );
        assert!(app.inventory_open, "the close keeps the inventory open");
        // A fresh piggy open, then the safe's click switches.
        assert!(interact_tile(&mut app, 10, 19));
        assert_eq!(app.player.as_ref().unwrap().chest, -2);
        assert!(interact_tile(&mut app, 20, 17));
        assert_eq!(
            app.player.as_ref().unwrap().chest,
            -3,
            "the switch opens the safe"
        );
    }

    #[test]
    fn test_store_1_the_void_lens_routes_pickups_into_bank4() {
        // `Player.cs:39395`'s pickup arm: holding 4131 (the Void Lens)
        // enables the vault (`IsVoidVaultEnabled`), `GetItem_VoidVault`
        // fills bank4 first.
        let mut app = mining_app(object_world(), 14, 260);
        app.inventory[20] = crate::item::ItemSlot {
            kind: 4131,
            name: "Void Lens",
            count: 1,
            prefix: 0,
        };
        // The per-tick refresh rides update_player; drive it directly here.
        app.player.as_mut().unwrap().void_vault_enabled = true;
        crate::world_sim::items::spawn_item_drop(&mut app, 9, 5, 14, 260);
        for _ in 0..40 {
            crate::world_sim::items::update_items(&mut app, 1.0 / 60.0);
        }
        let p = app.player.as_ref().unwrap();
        assert!(
            p.banks[3].iter().any(|sl| sl.kind == 9 && sl.count == 5),
            "the wood stack landed in bank4"
        );
        assert!(
            !app.inventory.iter().any(|sl| sl.kind == 9),
            "the inventory stayed out of it"
        );
    }

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
    }
}

/// `Player.ItemCheck_UseBossSpawners`'s eye arm (`Player.cs:43759-43771`):
/// the night gate, the roar (`SoundEngine.PlaySound(15, ...)`, `SoundID.Roar`
/// = 15), then `NPC.SpawnOnPlayer(onWhichPlayer, 4)`. The
/// `SummonItemCheck` (`Player.cs:42162-42176`) refuses while an EoC is
/// already alive; `ApplyItemTime`'s re-fire guard rides the swing timers
/// (the food/potion arms' shape).
/// `SummonItemCheck` (`Player.cs:42162-42179`): the item's own boss
/// living anywhere refuses the use — the pre-hardmode summon pairs.
fn summon_item_check(app: &App, item: u16) -> bool {
    let boss = match item {
        43 => NpcKind::EyeOfCthulhu,
        70 => NpcKind::EaterOfWorldsHead,
        560 => NpcKind::KingSlime,
        1133 => NpcKind::QueenBee,
        1331 => NpcKind::BrainOfCthulhu,
        // 544 refuses while EITHER twin lives (`:42168-42169`'s 125/126
        // pair); 556 rides the Destroyer 134, 557 the Prime 127.
        544 => NpcKind::Retinazer,
        556 => NpcKind::TheDestroyer,
        557 => NpcKind::SkeletronPrime,
        _ => return true,
    };
    !app.npcs
        .iter()
        .any(|n| n.hp > 0 && (n.kind == boss || (item == 544 && n.kind == NpcKind::Spazmatism)))
}

/// The pre-hardmode boss spawners (`ItemCheck_UseBossSpawners`,
/// `Player.cs:43751-43900`): the Worm Food 70 (`ZoneCorrupt` -> the EoW
/// head 13), the Slime Crown 560 (ungated -> the King 50), the
/// Abeemination 1133 (ungated -> the Queen Bee 222; the vanilla's
/// Item173 sound rides the roar — the port's bank holds the one), the
/// Bloody Spine 1331 (`ZoneCrimson` -> the Brain 266). The mech trio
/// 544/556/557 and the later
/// 4988/5120/5334 = the follow-ups. `NPC.SpawnOnPlayer` puts the boss at
/// the player; the EoW rides the worm engine (the chain assembles in the
/// worm pass); the Brain arrives alone (the Creepers' orbit = the
/// follow-up).
pub(crate) fn use_boss_summon_item(app: &mut App) -> bool {
    let item = app.inventory[app.selected].kind;
    if !matches!(item, 70 | 560 | 1133 | 1331 | 544 | 556 | 557) {
        return false;
    }
    if !summon_item_check(app, item) {
        return false;
    }
    // The biome gates (`:43775`'s ZoneCorrupt arm, `:43889`'s ZoneCrimson;
    // 560/1133 run ungated). The mech trio swaps the biome for the clock:
    // `!Main.IsItDay()` (`:43838`/`:43849`/`:43860`); the 1.4.5 item
    // variant's DisabledBossSummonVariant folds out (no variant system).
    let zone_ok = match item {
        70 => app.scene_metrics.zone_corrupt,
        1331 => app.scene_metrics.zone_crimson,
        544 | 556 | 557 => !app.world.day_time,
        _ => true,
    };
    if !zone_ok {
        return false;
    }
    let (px, py) = app
        .player
        .as_ref()
        .map(|p| p.center())
        .unwrap_or((0.0, 0.0));
    let xt = ((px / TILE_PX) as i32).clamp(2, app.world.width - 3);
    let yt = ((py / TILE_PX) as i32).clamp(2, app.world.height - 3);
    let boss = match item {
        70 => {
            let mut h = Npc::spawn_worm_head(&app.world, xt, yt, NpcKind::EaterOfWorldsHead);
            h.hp = h.max_hp;
            h.worm_left = 65;
            Some(h)
        }
        560 => {
            // SpawnOnPlayer's at-the-player semantics — the surface scan of
            // spawn_king_slime refuses the deep/indoor spawns the vanilla
            // allows.
            let mut k = Npc::new(NpcKind::KingSlime, px, py, 1, 0.0);
            k.hp = k.max_hp;
            Some(k)
        }
        1133 => {
            let mut b = Npc::new(NpcKind::QueenBee, px, py, 1, 0.0);
            b.hp = b.max_hp;
            Some(b)
        }
        // The mech trio (`:43838-43876`): the Eye spawns BOTH eyes
        // (SpawnOnPlayer(125) + SpawnOnPlayer(126)); the Worm rides the
        // 80-budget chain; the Skull lands as the Prime. The port pushes
        // the pair through the one returned boss + the direct list push.
        544 => {
            let mut a = Npc::new(NpcKind::Retinazer, px, py, 1, 0.0);
            a.hp = a.max_hp;
            let mut b = Npc::new(NpcKind::Spazmatism, px, py, 1, 0.0);
            b.hp = b.max_hp;
            app.npcs.push(a);
            Some(b)
        }
        556 => {
            let mut h = Npc::spawn_worm_head(&app.world, xt, yt, NpcKind::TheDestroyer);
            h.hp = h.max_hp;
            h.worm_left = 80;
            Some(h)
        }
        557 => {
            let mut b = Npc::new(NpcKind::SkeletronPrime, px, py, 1, 0.0);
            b.hp = b.max_hp;
            Some(b)
        }
        _ => {
            let mut b = Npc::new(NpcKind::BrainOfCthulhu, px, py, 1, 0.0);
            b.hp = b.max_hp;
            Some(b)
        }
    };
    let Some(boss) = boss else {
        return false;
    };
    app.audio.play("Roar_0");
    app.inventory[app.selected].count = app.inventory[app.selected].count.saturating_sub(1);
    if app.inventory[app.selected].count == 0 {
        app.inventory[app.selected].kind = 0;
    }
    app.npcs.push(boss);
    let held = app.inventory[app.selected];
    app.swing_max = crate::item::use_animation_secs(held.kind);
    app.swing = app.swing_max;
    app.attack_cd = app.swing_max;
    app.mine_target = None;
    app.mine_timer = 0.0;
    true
}

/// The Goblin Battle Standard (`Player.ItemCheck_UseEventItems`,
/// `Player.cs:43920-43940`): the use through `CanStartInvasion(1,
/// ignoreDelay: true)` — the refusal leaves the stack intact — then
/// `StartInvasion()` sizes `80 + 40 x the qualifying players` and the
/// stack pays one (`Item.consumable`); the roar rides the eye's sound.
pub(crate) fn use_goblin_battle_standard(app: &mut App) -> bool {
    let Some(player) = app.player.as_ref() else {
        return false;
    };
    let life_max = player.max_hp;
    if !crate::world_sim::npcs::can_start_invasion(&app.world, life_max) {
        return false;
    }
    crate::world_sim::npcs::start_invasion(&mut app.world, 1, 1);
    app.audio.play("Roar_0");
    app.inventory[app.selected].count = app.inventory[app.selected].count.saturating_sub(1);
    if app.inventory[app.selected].count == 0 {
        app.inventory[app.selected].kind = 0;
    }
    let held = app.inventory[app.selected];
    app.swing_max = crate::item::use_animation_secs(held.kind);
    app.swing = app.swing_max;
    app.attack_cd = app.swing_max;
    app.mine_target = None;
    app.mine_timer = 0.0;
    true
}

/// `ItemCheck_UseBossSpawners`' Frost Legion arm (`Player.cs:43941-43956`):
/// the Snow Globe (602) — the `CanStartInvasion(2, ignoreDelay: true)` gate
/// (`invasionType == 0` + a 200+ max-hp player; the delay folds out), the
/// roar (`SoundID.Roar` 15), then `invasionDelay = 0; StartInvasion(2)` —
/// the march state rides the World (the morning-roll resolve's shape).
/// The Celestial Sigil (`Player.cs:44028-44034`'s `ItemCheck_UseBossSpawners`
/// arm): the item 3601 gates on `downedGolemBoss && hardMode && !AnyDanger()
/// && !AnyoneNearCultists()` and calls `WorldGen.StartImpendingDoom(720)` —
/// the countdown's item value (`NPC.cs:6037`).
pub(crate) fn use_celestial_sigil(app: &mut App) -> bool {
    if app.world.moon_lord_countdown > 0 || app.world.lunar_apocalypse_is_up {
        return false;
    }
    if !app.world.downed_golem || !app.world.hard_mode {
        return false;
    }
    // `!AnyDanger()`: no invasion and no active event (`Main.cs:80246`).
    if app.world.invasion_type != 0 || app.world.dd2_event_ongoing {
        return false;
    }
    // `!AnyoneNearCultists()` (`NPC.cs:80290-80313`).
    if crate::world_sim::lunar::anyone_near_cultists(app) {
        return false;
    }
    app.world.moon_lord_countdown = crate::world_sim::lunar::SIGIL_COUNTDOWN;
    app.world.max_moon_lord_countdown = crate::world_sim::lunar::SIGIL_COUNTDOWN;
    crate::chat::broadcast_death_text(
        &mut app.chat,
        "The air is growing colder...",
        crate::chat::COLOR_COMMAND,
    );
    app.inventory[app.selected].count = app.inventory[app.selected].count.saturating_sub(1);
    if app.inventory[app.selected].count == 0 {
        app.inventory[app.selected].kind = 0;
    }
    let held = app.inventory[app.selected];
    app.swing_max = crate::item::use_animation_secs(held.kind);
    app.swing = app.swing_max;
    app.attack_cd = app.swing_max;
    app.mine_target = None;
    true
}

pub fn use_snow_globe(app: &mut App) -> bool {
    if app.world.invasion_type != 0 {
        return false;
    }
    let qualified = app
        .player
        .as_ref()
        .map(|p| p.max_hp >= 200)
        .unwrap_or(false);
    if !qualified {
        return false;
    }
    app.audio.play("Roar_0");
    app.world.invasion_delay = 0;
    app.world.invasion_goblins = true;
    app.world.invasion_type = 2;
    app.world.invasion_warn = 0;
    app.world.invasion_size = 80 + 40;
    app.world.invasion_size_start = app.world.invasion_size;
    let roll = (crate::app::frame::rand01(&mut app.item_rng) * 2.0) as u32;
    app.world.invasion_x = if roll == 0 {
        0.0
    } else {
        app.world.width as f64
    };
    app.inventory[app.selected].count = app.inventory[app.selected].count.saturating_sub(1);
    if app.inventory[app.selected].count == 0 {
        app.inventory[app.selected].kind = 0;
    }
    let held = app.inventory[app.selected];
    app.swing_max = crate::item::use_animation_secs(held.kind);
    app.swing = app.swing_max;
    app.attack_cd = app.swing_max;
    app.mine_target = None;
    app.mine_timer = 0.0;
    true
}

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
    app.inventory[app.selected].count = app.inventory[app.selected].count.saturating_sub(1);
    if app.inventory[app.selected].count == 0 {
        app.inventory[app.selected].kind = 0;
    }
    let held = app.inventory[app.selected];
    app.swing_max = crate::item::use_animation_secs_of(held.kind, held.prefix);
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
    fn the_boss_summon_items_gate_and_consume() {
        // `ItemCheck_UseBossSpawners` (`Player.cs:43751-43900`): the
        // ungated crown/abee, the corruption/crimson gates, the
        // SummonItemCheck refusal, the stack pay.
        let mut app = flow_app(None, vec![], vec![]);
        let mut player = crate::player::Player::spawn(&app.world);
        player.x = 64.0;
        player.y = 64.0;
        app.player = Some(player);
        // The Worm Food outside the corruption refuses free.
        app.inventory[app.selected] = ItemSlot {
            kind: 70,
            name: "Worm Food".into(),
            count: 1,
            prefix: 0,
        };
        assert!(!use_boss_summon_item(&mut app), "the corruption gate holds");
        assert_eq!(app.inventory[app.selected].count, 1, "the refusal is free");
        app.scene_metrics.zone_corrupt = true;
        assert!(use_boss_summon_item(&mut app));
        assert!(
            app.npcs
                .iter()
                .any(|n| n.kind == NpcKind::EaterOfWorldsHead && n.hp > 0)
        );
        assert_eq!(app.inventory[app.selected].count, 0, "the use paid one");
        // The Bloody Spine under the corruption refuses (the crimson gate).
        app.inventory[app.selected] = ItemSlot {
            kind: 1331,
            name: "Bloody Spine".into(),
            count: 1,
            prefix: 0,
        };
        assert!(!use_boss_summon_item(&mut app), "the crimson gate holds");
        app.scene_metrics.zone_crimson = true;
        assert!(use_boss_summon_item(&mut app));
        assert!(
            app.npcs
                .iter()
                .any(|n| n.kind == NpcKind::BrainOfCthulhu && n.hp > 0)
        );
        // The re-summon refuses while the boss lives (`SummonItemCheck`).
        app.inventory[app.selected] = ItemSlot {
            kind: 1331,
            name: "Bloody Spine".into(),
            count: 1,
            prefix: 0,
        };
        assert!(!use_boss_summon_item(&mut app), "the living brain refuses");
        assert_eq!(app.inventory[app.selected].count, 1);
        // The ungated Slime Crown: the first use summons the king.
        app.inventory[app.selected] = ItemSlot {
            kind: 560,
            name: "Slime Crown".into(),
            count: 1,
            prefix: 0,
        };
        assert!(use_boss_summon_item(&mut app));
        assert!(
            app.npcs
                .iter()
                .any(|n| n.kind == NpcKind::KingSlime && n.hp > 0)
        );
    }

    #[test]
    fn the_bloody_spine_brain_spawns_swarm_and_shields() {
        // `AI_054`'s init + the shield: the Bloody Spine's brain gets the
        // 20-Creeper swarm at the first state pass (`localAI[0]` 0 -> 1),
        // the shield holds while a Creeper lives (`dontTakeDamage`), and
        // the last Creeper's death frees the brain.
        let mut app = flow_app(None, vec![], vec![]);
        let mut player = crate::player::Player::spawn(&app.world);
        player.x = 64.0;
        player.y = 64.0;
        app.player = Some(player);
        app.scene_metrics.zone_crimson = true;
        app.inventory[app.selected] = ItemSlot {
            kind: 1331,
            name: "Bloody Spine".into(),
            count: 1,
            prefix: 0,
        };
        assert!(use_boss_summon_item(&mut app));
        crate::world_sim::npcs::update_brain_state(&mut app);
        let creepers = app
            .npcs
            .iter()
            .filter(|n| n.kind == NpcKind::Creeper && n.hp > 0)
            .count();
        assert_eq!(creepers, 20, "the init swarm");
        let brain = app
            .npcs
            .iter()
            .find(|n| n.kind == NpcKind::BrainOfCthulhu)
            .expect("the brain");
        assert!(brain.dont_take_damage, "the Creeper shield holds");
        assert!(app.world.brain_present);
        // The shielded brain refuses the hit (`StrikeNPC`'s no-op).
        let mut brain_mut = app
            .npcs
            .iter_mut()
            .find(|n| n.kind == NpcKind::BrainOfCthulhu)
            .unwrap();
        let hp_before = brain_mut.hp;
        assert!(
            !brain_mut.hurt(50, brain_mut.x - 10.0, 5.0, false),
            "the shield refuses"
        );
        assert_eq!(brain_mut.hp, hp_before);
        drop(brain_mut);
        // The last Creeper's death frees the brain.
        for n in app.npcs.iter_mut() {
            if n.kind == NpcKind::Creeper {
                n.hp = 0;
            }
        }
        crate::world_sim::npcs::update_brain_state(&mut app);
        let mut brain_mut = app
            .npcs
            .iter_mut()
            .find(|n| n.kind == NpcKind::BrainOfCthulhu)
            .unwrap();
        assert!(!brain_mut.dont_take_damage, "the shield falls");
        assert!(brain_mut.hurt(50, brain_mut.x - 10.0, 5.0, false));
        assert!(brain_mut.hp < hp_before);
    }

    #[test]
    fn the_mech_summons_night_gated_pair_and_refusals() {
        // `ItemCheck_UseBossSpawners`' mech arms (`Player.cs:43838-43876`):
        // day refuses free; night summons — the Eye spawns BOTH eyes, the
        // Worm the 80-budget chain, the Skull the Prime; the living-boss
        // refusal rides `SummonItemCheck`'s 125/126 pair.
        let mut app = flow_app(None, vec![], vec![]);
        let mut player = crate::player::Player::spawn(&app.world);
        player.x = 64.0;
        player.y = 64.0;
        app.player = Some(player);
        app.world.day_time = true;
        app.inventory[app.selected] = ItemSlot {
            kind: 544,
            name: "Mechanical Eye".into(),
            count: 1,
            prefix: 0,
        };
        assert!(!use_boss_summon_item(&mut app), "day refuses");
        assert_eq!(app.inventory[app.selected].count, 1, "the refusal is free");
        app.world.day_time = false;
        assert!(use_boss_summon_item(&mut app));
        assert!(
            app.npcs
                .iter()
                .any(|n| n.kind == NpcKind::Retinazer && n.hp > 0)
        );
        assert!(
            app.npcs
                .iter()
                .any(|n| n.kind == NpcKind::Spazmatism && n.hp > 0)
        );
        assert_eq!(app.inventory[app.selected].count, 0, "the use paid one");
        // Either twin living refuses the re-summon (`:42168-42169`).
        app.inventory[app.selected] = ItemSlot {
            kind: 544,
            name: "Mechanical Eye".into(),
            count: 1,
            prefix: 0,
        };
        assert!(!use_boss_summon_item(&mut app), "the living twins refuse");
        assert_eq!(app.inventory[app.selected].count, 1);
        // The Mechanical Worm rides the chain engine.
        app.inventory[app.selected] = ItemSlot {
            kind: 556,
            name: "Mechanical Worm".into(),
            count: 1,
            prefix: 0,
        };
        assert!(use_boss_summon_item(&mut app));
        let head = app
            .npcs
            .iter()
            .find(|n| n.kind == NpcKind::TheDestroyer)
            .expect("the destroyer head");
        assert_eq!(head.worm_left, 80, "the mech worm budget");
        // The Mechanical Skull lands the Prime.
        app.inventory[app.selected] = ItemSlot {
            kind: 557,
            name: "Mechanical Skull".into(),
            count: 1,
            prefix: 0,
        };
        assert!(use_boss_summon_item(&mut app));
        assert!(
            app.npcs
                .iter()
                .any(|n| n.kind == NpcKind::SkeletronPrime && n.hp > 0)
        );
    }
    #[test]
    fn the_golem_altar_consumes_a_cell_and_summons() {
        // `TileInteractionsUse`'s altar arm (`Player.cs:32755-32790`) +
        // `NPC.SpawnOnPlayer`'s case-245 column scan (`NPC.cs:80475-80531`):
        // the Power Cell 1293 in the inventory with hardmode and Plantera
        // down pays one cell and puts the Golem at the altar column's
        // `(up + up + down) / 3`; a missing gate refuses free.
        let mut app = flow_app(None, vec![], vec![]);
        let mut w = tr_core::World::new("t".into(), 64, 200);
        for x in 0..64 {
            for y in 120..200 {
                let i = w.index(x, y);
                w.tiles[i] = tr_core::Tile {
                    kind: 1,
                    ..Default::default()
                };
                w.tiles[i].flags |= tr_core::tile::FLAG_ACTIVE;
            }
        }
        // The altar's origin cell (237, frame 18/0) with a ceiling 10
        // above and a floor 6 below for the solid scans.
        for &(ax, ay, kind, fx, fy) in &[
            (30, 30, 237u16, 18i16, 0i16),
            (30, 20, 1, 0, 0),
            (30, 36, 1, 0, 0),
        ] {
            let i = w.index(ax, ay);
            w.tiles[i] = tr_core::Tile {
                kind,
                frame_x: fx,
                frame_y: fy,
                ..Default::default()
            };
            w.tiles[i].flags |= tr_core::tile::FLAG_ACTIVE;
        }
        w.compute_column_surface();
        app.world = w;
        let mut player = crate::player::Player::spawn(&app.world);
        player.x = 30.0 * 16.0;
        player.y = 30.0 * 16.0;
        app.player = Some(player);
        app.world.hard_mode = true;
        app.world.downed_plant_boss = true;
        // No cell: the click is consumed but nothing spawns.
        assert!(
            interact_tile(&mut app, 30, 30),
            "the altar consumes the click"
        );
        assert!(
            app.npcs.iter().all(|n| n.kind != NpcKind::Golem),
            "the missing cell refuses"
        );
        // The cell: one paid, the Golem at the (20 + 20 + 36)/3 row.
        app.inventory[app.selected] = ItemSlot {
            kind: 1293,
            name: "Lihzahrd Power Cell".into(),
            count: 2,
            prefix: 0,
        };
        assert!(interact_tile(&mut app, 30, 30));
        assert_eq!(app.inventory[app.selected].count, 1, "one cell paid");
        let golem = app
            .npcs
            .iter()
            .find(|n| n.kind == NpcKind::Golem)
            .expect("the Golem spawns");
        assert_eq!(golem.x, 30.0 * 16.0 + 8.0, "the altar column");
        assert_eq!(golem.y, 25.0 * 16.0, "the (20+20+36)/3 row");
        // A living Golem refuses the re-summon; hardmode off refuses too.
        interact_tile(&mut app, 30, 30);
        assert_eq!(app.inventory[app.selected].count, 1, "the living-boss gate");
        for n in app.npcs.iter_mut() {
            if n.kind == NpcKind::Golem {
                n.hp = 0;
            }
        }
        app.world.hard_mode = false;
        app.inventory[app.selected].count = 2;
        interact_tile(&mut app, 30, 30);
        assert_eq!(
            app.inventory[app.selected].count, 2,
            "the hardmode gate refuses free"
        );
        assert!(
            app.npcs
                .iter()
                .all(|n| n.kind != NpcKind::Golem || n.hp == 0),
            "the hardmode gate refuses"
        );
    }
    #[test]
    fn the_battle_standard_gates_and_sizing() {
        // `Main.CanStartInvasion` (`Main.cs:64414-64432`): the 200-life
        // gate; `StartInvasion(1)` (`:64435-64460`): 80 + 40 x qualifying.
        use crate::world_sim::npcs::{can_start_invasion, start_invasion};
        let mut w = tr_core::World::new("t".into(), 64, 64);
        assert!(!can_start_invasion(&w, 100), "the base life waits");
        assert!(can_start_invasion(&w, 200));
        start_invasion(&mut w, 1, 1);
        assert!(w.invasion_goblins);
        assert_eq!(w.invasion_size, 120, "80 + 40 x one qualifying player");
        start_invasion(&mut w, 1, 1);
        assert_eq!(w.invasion_size, 120, "the active invasion blocks");
    }

    #[test]
    fn the_battle_standard_starts_the_invasion_and_consumes() {
        // `ItemCheck_UseEventItems` (`Player.cs:43920-43940`): the use
        // starts the invasion + pays the stack; the refusal keeps it.
        let mut app = flow_app(None, vec![], vec![]);
        let mut player = crate::player::Player::spawn(&app.world);
        player.max_hp = 200;
        player.hp = player.max_hp;
        app.player = Some(player);
        app.inventory[app.selected] = ItemSlot {
            kind: 361,
            name: "Goblin Battle Standard".into(),
            count: 2,
            prefix: 0,
        };
        assert!(use_goblin_battle_standard(&mut app));
        assert!(app.world.invasion_goblins);
        assert_eq!(app.world.invasion_size, 120);
        assert_eq!(app.inventory[app.selected].count, 1, "the use paid one");
        assert!(
            !use_goblin_battle_standard(&mut app),
            "the active invasion refuses"
        );
        assert_eq!(app.inventory[app.selected].count, 1, "the refusal is free");
    }

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
        app.inventory[app.selected] = ItemSlot {
            kind: 43,
            name: "Suspicious Looking Eye".into(),
            count: 1,
            prefix: 0,
        };
        assert!(!use_suspicious_looking_eye(&mut app), "day refuses");
        app.world.day_time = false;
        assert!(use_suspicious_looking_eye(&mut app));
        assert_eq!(app.inventory[app.selected].count, 0, "the use consumed");
        assert!(
            app.npcs
                .iter()
                .any(|n| n.kind == NpcKind::EyeOfCthulhu && n.hp > 0)
        );
        app.inventory[app.selected] = ItemSlot {
            kind: 43,
            name: "Suspicious Looking Eye".into(),
            count: 1,
            prefix: 0,
        };
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
        let mut eye = crate::npc::Npc::spawn_eye_of_cthulhu(&app.world, 64.0, 64.0).unwrap();
        eye.hp = 0;
        app.npcs.push(eye);
        crate::world_sim::npcs::update_npcs(&mut app, 1.0 / 60.0);
        assert!(app.world.downed_eoc);
        assert!(!app.npcs.iter().any(|n| n.kind == NpcKind::EyeOfCthulhu));
    }
}

/// `ItemCheck_UseBossSpawners`' Pirate Invasion arm (`Player.cs:43957-43971`):
/// the Pirate Map (1315) — the `CanStartInvasion(3, ignoreDelay: true)` gate
/// (`invasionType == 0` + a 200+ max-hp player; the delay folds out), the
/// roar (`SoundID.Roar` 15), then `invasionDelay = 0; StartInvasion(3)` —
/// the march state rides the World (the morning-roll resolve's shape).
pub fn use_pirate_map(app: &mut App) -> bool {
    if app.world.invasion_type != 0 {
        return false;
    }
    /// `ItemCheck_Wiring_Gate`'s Eternia arm (`Player.cs:32425-32444`) — the
    /// Etherian Crystal (3828) placed over the Eternia Crystal Stand (466)
    /// starts the Old One's Army. The gates verbatim: no ongoing event, no
    /// crystal alive, no pumpkin/snow moon, the 3828 in the inventory, the
    /// stand's frame (`frameX/18*16`, `frameY/18*16` — the 3x3 stand
    /// anchor) gives the spawn point (+40, +64), then `ConsumeItem` and
    /// `DD2Event.SummonCrystal` (`StartInvasion` + the crystal NPC at the
    /// point + the starter-crystal drops).
    ///
    /// Deviations: `WouldFailSpawningHere` folds to the stand-tile check
    /// (the port has no lane machinery — the DD2Event framework's slice);
    /// `DropStarterCrystals`' per-player drops ride the item-drop pass.
    let qualified = app
        .player
        .as_ref()
        .map(|p| p.max_hp >= 200)
        .unwrap_or(false);
    if !qualified {
        return false;
    }
    app.audio.play("Roar_0");
    app.world.invasion_delay = 0;
    app.world.invasion_goblins = true;
    app.world.invasion_type = 3;
    app.world.invasion_warn = 0;
    app.world.invasion_size = 80 + 40 + 40 + 20;
    app.world.invasion_size_start = app.world.invasion_size;
    let roll = (crate::app::frame::rand01(&mut app.item_rng) * 2.0) as u32;
    app.world.invasion_x = if roll == 0 {
        0.0
    } else {
        app.world.width as f64
    };
    app.inventory[app.selected].count = app.inventory[app.selected].count.saturating_sub(1);
    if app.inventory[app.selected].count == 0 {
        app.inventory[app.selected].kind = 0;
    }
    let held = app.inventory[app.selected];
    app.swing_max = crate::item::use_animation_secs(held.kind);
    app.swing = app.swing_max;
    app.attack_cd = app.swing_max;
    app.mine_target = None;
    app.mine_timer = 0.0;
    true
}
pub fn use_etherian_crystal(app: &mut App) -> bool {
    if app.world.invasion_type != 0 || app.world.snow_moon || app.world.pumpkin_moon {
        return false;
    }
    let crystal_alive = app
        .npcs
        .iter()
        .any(|n| n.hp > 0 && n.kind == NpcKind::DD2EterniaCrystal);
    if crystal_alive {
        return false;
    }
    // The stand tile under the cursor.
    let m = app.mouse_pos;
    let (sw, sh) = app.screen_size();
    let screen = Vec2::new(sw, sh);
    let world_pos = app.camera + (m - screen * 0.5) / app.zoom;
    let tx = (world_pos.x / TILE_PX).floor() as i32;
    let ty = (world_pos.y / TILE_PX).floor() as i32;
    let tile = app.world.tile(tx, ty);
    let is_stand = tile.is_some_and(|t| t.active() && t.kind == 466);
    if !is_stand {
        return false;
    }
    let (fx, fy) = tile
        .map(|t| (t.frame_x as i32, t.frame_y as i32))
        .unwrap_or((0, 0));
    let (sx, sy) = (tx * 16 - fx / 18 * 16 + 40, ty * 16 - fy / 18 * 16 + 64);
    // The consume.
    app.inventory[app.selected].count = app.inventory[app.selected].count.saturating_sub(1);
    if app.inventory[app.selected].count == 0 {
        app.inventory[app.selected].kind = 0;
    }
    // The crystal + the event start.
    let mut crystal = Npc::new(
        NpcKind::DD2EterniaCrystal,
        sx as f32 - 20.0,
        sy as f32 - 20.0,
        1,
        0.0,
    );
    crystal.hp = crystal.max_hp;
    app.npcs.push(crystal);
    app.world.dd2_event_ongoing = true;
    app.world.dd2_wave = 1;
    let held = app.inventory[app.selected];
    app.swing_max = crate::item::use_animation_secs(held.kind);
    app.swing = app.swing_max;
    app.attack_cd = app.swing_max;
    true
}
