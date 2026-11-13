//! Player simulation (plan stage A4): the deterministic input assembly and the
//! fixed 60 Hz `Player::update` stepping with the render interpolation the
//! camera follows.

use glam::Vec2;
use winit::keyboard::KeyCode;

use crate::app::App;
use crate::inv::{ARMOR, MISC};
use crate::item::ItemSlot;
use crate::player::{Player, PlayerInput};
use crate::projectile::proj_data;

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
            // `:10018-10019` — the Archery Potion: the speed flag and its
            // `arrowDamage *= 1.1f` half (S1-C-4: the multiplier's bow arm
            // reads it now).
            16 => {
                p.archery = true;
                p.arrow_damage *= 1.1;
            }
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
            // `:13697-13712` — the Shroomite helmets: the ranged crit and the
            // ammo-family damage ×1.12 (S1-C-4: the multiplier's arms read
            // them now).
            1546 => {
                p.ranged_crit += 5;
                p.arrow_damage *= 1.12;
            }
            1547 => {
                p.ranged_crit += 5;
                p.bullet_damage *= 1.12;
            }
            1548 => {
                p.ranged_crit += 5;
                p.rocket_damage *= 1.12;
            }
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
                p.arrow_damage_additive_stack += 0.1;
            }
            4006 | 1321 => {
                p.magic_quiver = true;
                p.arrow_damage_additive_stack += 0.1;
            }
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

/// The summon-cap half of `Player.UpdateEquips` (M4-PROJ-8 phase 2): the
/// armor rows (`GrantArmorBenefits` over `GetEffectiveArmor(0..10)`,
/// `Player.cs:13258-13948`) and the held items (`ApplyEquipFunctional` over
/// the accessory slots 3..10 + the port's `miscEquips` host, `:15324-15329`)
/// raise `maxMinions`/`maxTurrets`. ONLY the budget deltas port — the
/// `minionDamage`/`whipRange`/`minionKB` halves have no port consumer yet
/// (M3-EQUIP-3 / the M4 follow-ups) and the DD2 turret-set rows (`:13007`,
/// `:13133-13230`, the `dd2Accessory` flag `:13010-13013`) sit outside the
/// phase's cited scope (M4-EVENT-5). Recorded in the phase discovery.
pub(crate) fn grant_summon_caps(
    p: &mut Player,
    armor: &[ItemSlot; ARMOR],
    misc: &[ItemSlot; MISC],
    world: &tr_core::World,
) {
    // `Main.tenthAnniversaryWorld` — the port reads the world's seed flag
    // (`World.seeds`, the `world_sim/npcs.rs:2895` stand-in pattern); the
    // save-header wiring lands with M5-SEED-1, so a loaded .wld carries it
    // only through world creation's anniversary seed option until then.
    let tenth_anniversary = world
        .seeds
        .contains(tr_core::seeds::SeedFlags::TENTH_ANNIVERSARY);
    // `GrantArmorBenefits` (`:13103+`): the armor-piece and accessory slots
    // 0..10, each row once per equipped kind.
    for slot in &armor[..10] {
        match slot.kind {
            238 if tenth_anniversary => p.max_minions += 1, // :13256-13259
            3777 | 3778 | 4873 | 4899 | 5524 | 5068 | 2361 | 2362 | 3267 | 1832 | 1834 => {
                p.max_minions += 1
            }
            1158..=1161 | 2370..=2371 | 2372 => p.max_minions += 1,
            3381 => {
                // `:13871-13876`: +1 minion and +1 turret together.
                p.max_minions += 1;
                p.max_turrets += 1;
            }
            3382 | 3383 => p.max_minions += 2, // :13877-13881
            1833 => p.max_minions += 2,        // :13941-13945
            _ => {}
        }
    }
    // `ApplyEquipFunctional`'s held summon items (`:15322-15331`): the
    // Pygmy Necklace (1845) and the Papyrus Scarab (1864) — the 1167 row's
    // minionKB/minionDamage halves carry no budget delta.
    for slot in armor[3..10].iter().chain(misc.iter()) {
        match slot.kind {
            1864 | 1845 => p.max_minions += 1,
            _ => {}
        }
    }
}

pub(crate) fn player_input(
    app: &App,
    binds: &[String; crate::input_map::BIND_COUNT],
    map_open: bool,
    hold_right: bool,
    hold_jump: bool,
) -> PlayerInput {
    if map_open {
        return PlayerInput {
            left: false,
            right: false,
            jump: false,
            down: false,
            hook: false,
        };
    }
    // `Player.controlLeft`/`controlRight`/`controlJump`/`controlDown` off
    // `Main.cLeft`/`cRight`/`cJump`/`cDown` (`Main.cs:67164-67167`). The arrow
    // keys are kept as a port convenience — vanilla binds only the four `c*`
    // rows — so a rebind adds a key rather than replacing the arrows.
    let held = |action: crate::input_map::Action, arrow: KeyCode| {
        app.keys
            .iter()
            .any(|k| crate::input_map::is_bound(binds, action, *k))
            || app.keys.contains(&arrow)
    };
    let any = |action: crate::input_map::Action| {
        app.keys
            .iter()
            .any(|k| crate::input_map::is_bound(binds, action, *k))
    };
    PlayerInput {
        left: held(crate::input_map::Action::Left, KeyCode::ArrowLeft),
        right: hold_right || held(crate::input_map::Action::Right, KeyCode::ArrowRight),
        // `cUp` also jumps: `Player.controlUp` feeds `controlJump` in the
        // vanilla's `Player.Update`, which is why the default `W` jumps.
        jump: hold_jump
            || held(crate::input_map::Action::Jump, KeyCode::ArrowUp)
            || any(crate::input_map::Action::Up),
        down: held(crate::input_map::Action::Down, KeyCode::ArrowDown),
        // `Player.controlHook` on `Main.cHook` (`Main.cs:67172`, default `E`);
        // the port keeps `G` as the extra quick-grapple key.
        hook: any(crate::input_map::Action::Hook) || app.keys.contains(&KeyCode::KeyG),
    }
}

pub(crate) fn update_player(app: &mut App, dt: f32) {
    // Deterministic held input for screenshots (`TR_HOLD=right,jump`).
    let (hold_right, hold_jump) = crate::env::hold_input();
    // `Player.cs:24772-24803`: while the map is fullscreen every movement control
    // is zeroed, so the player stands still and the WASD keys pan the map.
    let map_open = app.map.as_ref().is_some_and(|m| m.open);
    // The bind table, copied out so the player borrow below does not conflict
    // (`Main.cUp`..`Main.cMount`, `Main.cs:67163-67176`).
    let binds = app.keybinds.clone();
    // `Player.Update`'s journey-power reads, copied out of the registry
    // before the player borrow: `ResetEffects` clears `creativeGodMode`
    // (`Player.cs:18789`) and the tick immediately re-reads the power
    // (`:25493-25497`); the same `ResetEffects` block rebuilds
    // `tileRangeX/Y` from `FarPlacementRangePower` (`:18815-18828`).
    let power_god_mode = app.creative_powers.god_mode();
    let power_far_placement = app.creative_powers.far_placement_range();
    let input = player_input(app, &binds, map_open, hold_right, hold_jump);
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
    // `TR_DIE`: the death-screen fixture — the local player dies on the first
    // tick past the skip lockout so `TR_SHOT` captures the respawn prompt.
    if crate::env::die_pin() && !app.player.as_ref().is_some_and(|p| p.dead) {
        let world = &app.world;
        if let Some(p) = app.player.as_mut() {
            p.start_death(world, false, false);
            // `Player.KillMe`'s hp clamp — the death screen's draw gate reads
            // `hp <= 0`, not `dead`.
            p.hp = 0;
            p.dead_time = 100;
            // The broadcast line (`Lang.inter[38]`, "You were slain..."),
            // which a real `Player.KillMe` would set through the death pass.
            app.death_text = crate::lang::inter(38);
        }
    }
    if let Some(p) = app.player.as_mut() {
        // `Player.ResetEffects`' journey block, re-run every tick: the godmode
        // flag (`:18789` clears, `:25493-25497` sets) and the far-placement
        // tile-range toggle (`:18815-18828`).
        p.creative_god_mode = power_god_mode;
        p.far_placement_range = power_far_placement;
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
        // `Player.statDefense` is derived inside `Player.Update` now
        // (`Player::update`'s reset block: `GrantArmorBenefits`' piece rows +
        // the set bonus + the buff arms, M3-EQUIP-5). Assigning it here used to
        // overwrite the buff and prefix arms the tick had just added, so an
        // Ironskin potion or a Warding accessory changed nothing.
        // `Player.Update` -> `UpdateDead` (`Player.cs:16919`): the dead player
        // runs the respawn-timer tick instead of the normal simulation. The
        // boss check reads the live NPC list; the danger-NPC gate has no
        // `DangerThatPreventsOtherDangers` members in the ported set yet.
        if p.dead {
            // `Player.AllowsRespawnTimerSkip`'s two NPC gates (`Player.cs:38962-38987`):
            // a boss hindering the respawn, and any NPC that blocks it
            // (`NearAnyNPCsThatBlockRespawn` — the live invasion's members;
            // the danger-set arm has no ported members yet).
            let boss_near = p.boss_hinders(&app.npcs);
            let invasion_live = app.world.invasion_size > 0;
            let danger_near = p.near_any_npcs_that_block_respawn(&app.npcs, invasion_live);
            app.respawn_skip_available = p.allows_respawn_timer_skip(boss_near, danger_near);
            let mut respawn_now = false;
            app.player_step.tick(dt, |_| {
                respawn_now =
                    p.update_dead(input.jump, boss_near, danger_near) || respawn_now;
                p.reset_projectile_caches();
                p.update_projectile_caches(&app.projectiles);            });
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
                app.respawn_skip_available = false;
                app.respawn_skip_frames = 0;
                app.npcs.clear();
                app.npc_timer = 4.0;
            }
            // S1-B-4: the camera moved to the draw path
            // (`DoDraw_UpdateCameraPosition`, `Main.cs:62590-62649`) — the
            // player pass no longer moves it (the C#'s update half reads the
            // previous frame's position), so the old lerp-follow is gone and
            // the draw-time snap owns every frame's camera.
            return;
        }
        // `AllowsRespawnTimerSkip` is a death-screen gate: alive again, the
        // flag and the prompt's counter clear (`Main.cs:45374-45376`'s else).
        if app.respawn_skip_available {
            app.respawn_skip_available = false;
            app.respawn_skip_frames = 0;
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
            // `QuickGrapple` (`Player.cs:6296-6478`): the fire-time gate first
            // (`:6399` — the in-flight count against `num6 − num5`), then the
            // oldest-hook recycle (`:6396`, `num7`), then the two kind cycles
            // (the Lunar 646→…→649 walk `:6425-6456` and the Dual 73→74 swap
            // `:6420-6424`), and only then the spawn (`:6458-6477`).
            let owner = app.local_id as u8;
            let (flight, attached) = app.projectiles.hook_counts(owner);
            let mut kind = hook_type as u16;
            if !crate::grapple::fire_allowed(flight, attached, kind) {
                return;
            }
            // The Dual Hook's colour swap: a blue head already out means red.
            let blue_out = app
                .projectiles
                .slots()
                .iter()
                .any(|pr| pr.active && pr.owner == owner && pr.kind == crate::projectile::DUAL_HOOK_BLUE);
            if kind == crate::projectile::DUAL_HOOK_BLUE && blue_out {
                kind = crate::grapple::dual_next_kind(true);
            }
            // The Lunar cycle: the highest-`timeLeft` in-flight kind steps.
            if kind == crate::projectile::LUNAR_HOOK_SOLAR {
                let best = app
                    .projectiles
                    .slots()
                    .iter()
                    .filter(|pr| {
                        pr.active
                            && pr.owner == owner
                            && (crate::projectile::LUNAR_HOOK_SOLAR..=crate::projectile::LUNAR_HOOK_STARDUST)
                                .contains(&pr.kind)
                    })
                    .max_by_key(|pr| pr.time_left)
                    .map(|pr| pr.kind);
                kind = crate::grapple::next_lunar_kind(best);
            }
            // The recycle: the lowest-`timeLeft` hook of this owner dies.
            if crate::grapple::recycles_oldest(flight, attached, kind) {
                if let Some(i) = app
                    .projectiles
                    .slots()
                    .iter()
                    .enumerate()
                    .filter(|(_, pr)| {
                        pr.active
                            && pr.owner == owner
                            && proj_data(pr.kind).is_some_and(|d| d.ai_style == 7)
                    })
                    .min_by_key(|(_, pr)| pr.time_left)
                    .map(|(i, _)| i)
                {
                    app.projectiles.kill(i);
                }
            }
            // `:6458-6477` — the aim at the mouse world position, the velocity
            // the unit aim times `item.shootSpeed` (the vanilla 16 for the Hook;
            // the per-item values ride with M3-EQUIP-3, `TR_HOOK` pins the kind).
            let center = Vec2::new(
                p.x + crate::player::WIDTH * 0.5,
                p.y + crate::player::HEIGHT * 0.5,
            );
            app.projectiles.spawn_hook(
                kind,
                center,
                mouse_world,
                16.0,
                0,
                0.0,
                owner,
                p.facing,
            );
        }
        // The quick-mount key: the `controlMount` press edge (the menu's
        // "Mount R" row, `Main.cMount` default "R") fires `QuickMount`
        // (`Player.cs:24888-24893` -> `:6045-6100`).
        // `Main.cMount` (`Main.cs:67176`, default `R`). The previous edge is an
        // `App` field, not a `static`, so two headless apps in one test process
        // cannot share it.
        let mount_key = app
            .keys
            .iter()
            .any(|k| crate::input_map::is_bound(&binds, crate::input_map::Action::Mount, *k));
        let mount_edge = mount_key && !app.mount_prev;
        app.mount_prev = mount_key;
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
        // The hook runs as an aiStyle-7 projectile in the pool
        // (`AI_007_GrapplingHooks`, stepped by `update_projectiles`); this is
        // only the bridge to `GrappleMovement`'s pull — the attached hook's
        // center, as the vanilla's `grappling[]`/`grapCount` reader. The
        // multi-hook registration replaces both in the next phase.
        // `AI_007`'s `grappling[]`/`grapCount` (`Projectile.cs:52705`/`:52862`):
        // the attached slots in slot order, handed to `GrappleMovement`.
        // The tuple's third element is the hook's rotated direction — only the
        // Squirrel Hook's steering reads it (`GetGrapplingForces :22803`).
        let (reg, count) = app.projectiles.hook_reg();
        let grapple_for_tick: Vec<(u16, Vec2, f32)> = reg[..count]
            .iter()
            .map(|&i| {
                let pr = &app.projectiles.slots()[i as usize];
                (pr.kind, pr.grapple_center, pr.rot)
            })
            .collect();
        p.grappling_active = !grapple_for_tick.is_empty();
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
        // `PlayerFrame()`'s held-item inputs (`Player.cs:36143-36146`): the C#
        // reads `inventory[selectedItem]` straight out of the player, the port
        // hands the summary in before the fixed step.
        {
            let held = app.inventory[app.selected];
            let kind = held.kind;
            let use_style = crate::item::use_style(kind);
            let hold_style = crate::item::hold_style(kind);
            let swinging = app.swing > 0.0 && held.count > 0;
            // `ItemCanBeVisuallyHeld`'s two reachable gates: the 55/56/61
            // mounts hold nothing (`MountID.Sets.DontHoldItems`,
            // `MountID.cs:46`) and the umbrella hold (4) is off while
            // mounted, roped or airborne (`Player.cs:50209-50224`).
            let mounted_blocks_hold =
                p.mount.is_active() && matches!(p.mount.kind(), 55 | 56 | 61);
            let can_visually_hold = !mounted_blocks_hold
                && (hold_style != 4
                    || (!p.pulley && p.vy == 0.0 && !p.mount.is_active()));
            // The hooks' centroid relative to the body centre
            // (`Player.cs:36502-36520`): only the body row leans on it.
            let mut dx = 0.0f32;
            let mut dy = 0.0f32;
            if p.grappling_active {
                let owner = app.local_id as u8;
                let mut n = 0.0f32;
                for pr in app.projectiles.slots() {
                    if !pr.active
                        || pr.owner != owner
                        || !crate::projectile::proj_data(pr.kind)
                            .is_some_and(|d| d.ai_style == 7)
                    {
                        continue;
                    }
                    dx += pr.pos.x + pr.w * 0.5;
                    dy += pr.pos.y + pr.h * 0.5;
                    n += 1.0;
                }
                if n > 0.0 {
                    dx = dx / n - (p.x + crate::player::WIDTH * 0.5);
                    dy = dy / n - (p.y + p.height * 0.5);
                }
            }
            p.frame_inputs = crate::player::FrameInputs {
                use_style,
                // `flag6` (`Player.cs:36146`): the hidden/raise-lamp/hold-orb
                // styles keep the locomotion rows.
                use_style_blocks_frames: !matches!(use_style, 10 | 14 | 15),
                item_type: kind as i32,
                hold_style: hold_style as i32,
                can_visually_hold,
                swing_active: swinging,
                swing_remaining: if swinging {
                    app.swing / app.swing_max.max(f32::EPSILON)
                } else {
                    0.0
                },
                aim_relative: app.swing_aim * p.facing as f32,
                // `!wet || !item.noWet` and `!happyFunTorchTime ||
                // createTile != 4` — the port has neither item field, so both
                // gates are open (documented in ANALYSIS).
                hold_style_dry: true,
                torch_hold_ok: true,
                grapple_dx: dx,
                grapple_dy: dy,
            };
        }
        let step = 1.0 / 60.0;
        if cart_pin && !p.cart_mounted {
            p.mount_cart();
        }
        let mut run_pixels = 0.0f32;
        app.player_step.tick(dt, |_| {
            // `player.itemAnimation > 0` for the sleeping state — the same
            // gate the swing render uses (`self.swing > 0.0` with a held
            // item, `frame.rs:483`). The danger gate rides the M2-DEATH-2
            // stand-in (no `DangerThatPreventsOtherDangers` members yet).
            let item_in_use = app.swing > 0.0 && app.inventory[app.selected].count > 0;
            // `Player.miscCounter++` (`Player.cs:31827`): the sandstorm leg
            // pose's 4-tick turn cadence reads it.
            p.misc_counter = p.misc_counter.wrapping_add(1);
            // `Player.cs:28162-28176,28233-28236`: `vector7` captures the
            // pre-movement position, `flag34` is false on a mount or while
            // tongued, and `HandleRunning` fires after movement when grounded.
            let run_x0 = p.x;
            let run_eligible = !p.mount.is_active()
                && !(0..crate::buffs::MAX_BUFFS)
                    .any(|i| p.buffs.buff_time[i] > 0 && p.buffs.buff_type[i] == 38);
            // The player stage's named stream (`Main.SwapRandom`, the C#
            // stage swap `Main.cs:17755-17757`): the tick's hurt rolls (the
            // dodge block, the lava `Hurt`) draw from it (S1-A-10).
            let rng = app.rngs.stream(crate::app::RngKey::UpdatePlayers);
            p.update_with_grapple(
                &app.world,
                input,
                &grapple_for_tick,
                item_in_use,
                false,
                app.rain.is_raining(),
                Some(&app.wind),
                rng,
            );
            if run_eligible && p.vy == 0.0 {
                run_pixels += (p.x - run_x0).abs();
            }
            // `Player::update_summon_buff_arms`): the recount
            p.update_projectile_caches(&app.projectiles);
            p.update_summon_buff_arms(&mut app.projectiles, &app.inventory);
            // DW-3.3/DW-3.4 — the ammo-mastery half of `Player.Update`
            // (`Player.UpdateBuffs` → `UpdateEquips` → … → `Update_AmmoCycler` →
            // `ItemCheck`, `:25566`/`:25633`/`:24566`/`:31827`), which the port
            // splits into its frame order `update_player` → `update_editing`
            // (`frame.rs:285`/`:481`): the flags clear and re-derive from this
            // tick's equipment and buffs, then the cycler's cooldown decrements
            // — both inside the fixed step, BEFORE the shoot path reads them.
            grant_ammo_mastery(p, &app.equip.armor, &app.equip.misc_equips);
            // The `UpdateEquips` summon-cap rows (`Player.cs:13254-13948`,
            // `:15324-15329`) — the armor/held half beside the ammo flags.
            grant_summon_caps(p, &app.equip.armor, &app.equip.misc_equips, &app.world);
            // `Player.cs:25709-25712` — a `maxTurrets` change drives the
            // `UpdateMaxTurrets` sweep (the lowest-`timeLeft` wipable sentry
            // dies while the budget holds); Phase 5's pool method runs it.
            if p.max_turrets_old != p.max_turrets {
                app.projectiles.update_max_turrets(
                    p.max_turrets,
                    app.local_id as u8,
                    tr_core::dd2::event_ongoing(),
                );
                p.max_turrets_old = p.max_turrets;
            }
            p.ammo_cycling.tick();
            // M4-PROJ-8 Phase 6 — `MinionRestTargetAim`'s invalidation half
            // (`Player.cs:57009-57011`): an unchaseable or >3000-px whip
            // target drops every tick.
            p.invalidate_minion_target(&app.npcs);
            // `Player.cs:27608-27614`: right after the NPC collision pass,
            // the tile-contact damage check (`GetHurtTile` +
            // `ApplyTouchDamage`) — the spike hazard the dungeon pass paints.
            crate::touch_damage::apply_touch_damage(
                &app.world,
                p,
                &mut app.item_rng,
                crate::player::WIDTH as i32,
                crate::player::HEIGHT as i32,
            );
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
            // `Main.UpdateWorld_Players`' tail (`Main.cs:17869-17873`): the
            // journey godmode player's three pools are refilled to their
            // maxima — `statLifeMax2`/`statManaMax2`/`breathMax` are the
            // port's `max_hp`/`mana_max`/`BREATH_MAX`.
            if p.creative_god_mode {
                p.hp = p.max_hp;
                p.mana = p.mana_max;
                p.breath = crate::player::BREATH_MAX;
            }
        });
        app.player_step.drop_backlog();
        if run_pixels > 0.0 {
            // `Player.cs:28233-28236` -> `AchievementsHelper.HandleRunning`
            // (`AchievementsHelper.cs:420-423`): grounded, unmounted running
            // advances MARATHON_MEDALIST's `Move` float accumulator.
            let _ = app.achievements.notify_custom_float("Move", run_pixels);
        }
        // `RemoveAllGrapplingHooks` (`Player.cs:33562-33566`): the jump-off
        // kills every aiStyle-7 slot this player owns. The pool-wide scan is
        // the source's own `for (i < 1000)` loop; the multi-hook phase moves it
        // into the pool beside the recycle tables.
        if !p.grappling_active {
            let owner = app.local_id as u8;
            let hooks: Vec<usize> = app
                .projectiles
                .slots()
                .iter()
                .enumerate()
                .filter(|(_, pr)| {
                    pr.active
                        && pr.owner == owner
                        && crate::projectile::proj_data(pr.kind)
                            .is_some_and(|d| d.ai_style == 7)
                })
                .map(|(i, _)| i)
                .collect();
            for i in hooks {
                app.projectiles.kill(i);
            }
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

    #[test]
    fn the_journey_godmode_power_refreshes_the_flag_and_refills_the_pools() {
        // The tick's creative reads (`Player.cs:18789` clears
        // `creativeGodMode`, `:25493-25497` re-reads `GodmodePower`) plus
        // `Main.UpdateWorld_Players`' refill (`Main.cs:17869-17873`).
        let mut app = ammo_app();
        app.creative_powers.set(
            crate::creative::PowerId::Godmode,
            crate::creative::PowerState::Toggle(true),
        );
        {
            let p = app.player.as_mut().unwrap();
            p.hp = 5;
            p.mana = 0;
            p.breath = 0;
        }
        tick_ammo(&mut app);
        let p = app.player.as_ref().unwrap();
        assert!(p.creative_god_mode, "the flag is refreshed from the power");
        assert_eq!(p.hp, p.max_hp, "statLife refilled to statLifeMax2");
        assert_eq!(p.mana, p.mana_max, "statMana refilled");
        assert_eq!(p.breath, crate::player::BREATH_MAX, "breath refilled");
        // Off again: the next tick clears the flag and the hit path is live.
        app.creative_powers.set(
            crate::creative::PowerId::Godmode,
            crate::creative::PowerState::Toggle(false),
        );
        tick_ammo(&mut app);
        let p = app.player.as_mut().unwrap();
        assert!(!p.creative_god_mode);
        p.hurt_cd = 0.0;
        assert!(p.hurt(10, 0.0, &mut tr_core::UnifiedRandom::default(), true), "the hurt lands once the power is off");
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
        let w_spawn =
            (app.world.spawn_x as f32 * tr_core::TILE_PX + 8.0 - crate::player::WIDTH as f32 * 0.5);
        let p = app.player.as_ref().unwrap();
        assert_eq!(p.spawn_x, w_spawn, "the broken bed reverts the spawn");
        assert_eq!(app.world.bed_spawn, None);
    }

    /// M4-PROJ-8 phase 2 — the summon seam's tick-level behaviour: the recount
    /// (`Player.cs:25565`), the dead-path recount (`:24752-24753`) and the
    /// `UpdateEquips` cap rows (`:13258-13948`/`:15324-15329`).
    mod summon_tests {
        use super::*;
        use crate::projectile::ProjectilePool;

        /// Deactivates a slot through the public key path (`key_of` +
        /// `kill_remote`) — the pool's `Kill` stand-in between ticks.
        fn kill_pool_slot(pool: &mut ProjectilePool, slot: usize) {
            let key = pool.key_of(slot);
            let (owner, slot, generation) = crate::net::projectile_key_parts(key);
            pool.kill_remote(slot, owner, generation);
        }

        /// The live pool's owned counts (owner 0 — the port's whoAmI) for the
        /// given kinds.
        fn live_counts(pool: &ProjectilePool, kinds: &[u16]) -> Vec<i32> {
            kinds
                .iter()
                .map(|&k| {
                    pool.slots()
                        .iter()
                        .filter(|p| p.active && p.owner == 0 && p.kind == k)
                        .count() as i32
                })
                .collect()
        }

        /// The live pool's highest `originalDamage` among owner-0 slots of
        /// `kind` (0 when none — the recount zeroed it).
        fn live_max(pool: &ProjectilePool, kind: u16) -> i32 {
            pool.slots()
                .iter()
                .filter(|p| p.active && p.owner == 0 && p.kind == kind)
                .map(|p| p.original_damage)
                .max()
                .unwrap_or(0)
        }

        /// DW-2.1 — the reset→recount order leaves every counter equal to the
        /// live pool after each tick, no drift over 600 ticks (with mid-run
        /// spawns and kills churning the pool). The budgets stay at the 1/1
        /// base with no cap sources, and `numMinions`/`slotsMinions` zero every
        /// tick (`:28279-28280`) until Phase 3's pool pass accumulates them.
        #[test]
        fn test_DW_2_1_the_recount_tracks_the_pool_over_600_ticks() {
            let mut app = ammo_app();
            {
                let pool = &mut app.projectiles;
                pool.spawn_with_data_ai_owned(373, 0, 100.0, 100.0, 0.0, 0.0, 10, 1.0, 0.0, 0.0);
                pool.spawn_with_data_ai_owned(308, 0, 130.0, 100.0, 0.0, 0.0, 10, 1.0, 0.0, 0.0);
                pool.spawn(
                    3,
                    0,
                    glam::Vec2::new(150.0, 100.0),
                    glam::Vec2::ZERO,
                    5,
                    1.0,
                    false,
                );
                // A foreign-owner slot — never counted (the C# `owner != i` skip).
                pool.spawn(
                    3,
                    1,
                    glam::Vec2::new(150.0, 130.0),
                    glam::Vec2::ZERO,
                    5,
                    1.0,
                    false,
                );
                let tiger1 = pool
                    .spawn(
                        831,
                        0,
                        glam::Vec2::new(160.0, 100.0),
                        glam::Vec2::ZERO,
                        40,
                        1.0,
                        false,
                    )
                    .unwrap();
                pool.set_original_damage(tiger1, 40);
                let tiger2 = pool
                    .spawn(
                        831,
                        0,
                        glam::Vec2::new(170.0, 100.0),
                        glam::Vec2::ZERO,
                        90,
                        1.0,
                        false,
                    )
                    .unwrap();
                pool.set_original_damage(tiger2, 90);
                let abigail = pool
                    .spawn(
                        970,
                        0,
                        glam::Vec2::new(180.0, 100.0),
                        glam::Vec2::ZERO,
                        55,
                        1.0,
                        false,
                    )
                    .unwrap();
                pool.set_original_damage(abigail, 55);
            }
            let kinds = [3u16, 266, 308, 373, 831, 970];
            for tick in 0..600 {
                if tick == 200 {
                    app.projectiles.spawn(
                        266,
                        0,
                        glam::Vec2::new(190.0, 100.0),
                        glam::Vec2::ZERO,
                        10,
                        1.0,
                        false,
                    );
                }
                if tick == 400 {
                    // Kill one storm tiger — the maximum drops to the survivor's.
                    let victim = app
                        .projectiles
                        .slots()
                        .iter()
                        .position(|p| p.active && p.owner == 0 && p.kind == 831)
                        .unwrap();
                    kill_pool_slot(&mut app.projectiles, victim);
                }
                tick_ammo(&mut app);
                let p = app.player.as_ref().unwrap();
                let expected = live_counts(&app.projectiles, &kinds);
                let got: Vec<i32> = kinds
                    .iter()
                    .map(|&k| p.owned_projectile_counts[usize::from(k)])
                    .collect();
                assert_eq!(got, expected, "tick {tick}: the counts mirror the pool");
                assert_eq!(
                    p.highest_storm_tiger_gem_original_damage,
                    live_max(&app.projectiles, 831),
                    "tick {tick}"
                );
                assert_eq!(
                    p.highest_abigail_counter_original_damage,
                    live_max(&app.projectiles, 970),
                    "tick {tick}"
                );
                assert_eq!(p.num_minions, 0, "tick {tick}: the per-tick reset");
                assert_eq!(p.slots_minions, 0.0);
                assert_eq!((p.max_minions, p.max_turrets), (1, 1), "tick {tick}");
            }
        }

        /// DW-2.1 dirty variant — death mid-tick: the counters reset
        /// (`Player.cs:17009-17010`) while the pool stays intact and the
        /// dead-path recount still runs (`:24752-24753`); the buff arms do not.
        #[test]
        fn test_DW_2_1_dead_player_resets_counters_and_keeps_the_pool() {
            let mut app = ammo_app();
            app.projectiles
                .spawn_with_data_ai_owned(373, 0, 100.0, 100.0, 0.0, 0.0, 10, 1.0, 0.0, 0.0);
            tick_ammo(&mut app);
            {
                let p = app.player.as_mut().unwrap();
                assert_eq!(p.owned_projectile_counts[373], 1);
                p.num_minions = 3;
                p.slots_minions = 2.5;
                p.buffs.add_buff(64, 100);
                p.start_death(&app.world, false, false);
            }
            tick_ammo(&mut app);
            let p = app.player.as_ref().unwrap();
            assert!(p.dead);
            assert_eq!(p.num_minions, 0, "UpdateDead zeroes it (:17009)");
            assert_eq!(p.slots_minions, 0.0);
            assert_eq!(
                p.owned_projectile_counts[373], 1,
                "the dead path still recounts (:24752-24753)"
            );
            assert_eq!(app.projectiles.active_count(), 1, "the pool is intact");
            // The buff arms did not run — the C# dead path returns before
            // `UpdateBuffs` (`:24755`), so no arm refreshed buff 64 and the flag
            // was never re-affirmed. The port's dead path runs no buff timer
            // either (the C#'s non-persistent-buff wipe at `:16995-17001` is a
            // death-system behavior outside this phase — recorded in the
            // discovery).
            let b = p.buffs.find_buff_index(64).unwrap();
            assert_eq!(p.buffs.buff_time[b], 100);
            assert!(!p.slime);
        }

        /// DW-2.3 — the cap buffs move the budgets by the verbatim deltas
        /// (`Player.cs:10175-10183`) and expiry returns to the 1/1 base; 150/348
        /// never tick down (`BuffID.Sets.TimeLeftDoesNotDecrease`).
        #[test]
        fn test_DW_2_3_cap_buffs_move_the_budgets_and_expire_to_base() {
            let mut app = ammo_app();
            app.player.as_mut().unwrap().buffs.add_buff(110, 3);
            tick_ammo(&mut app);
            assert_eq!(app.player.as_ref().unwrap().max_minions, 2);
            // The buff expires: the drop returns the base.
            for _ in 0..3 {
                tick_ammo(&mut app);
            }
            assert_eq!(app.player.as_ref().unwrap().max_minions, 1);
            // 150 (Bewitched) — same delta, and it never decrements.
            app.player.as_mut().unwrap().buffs.add_buff(150, 100);
            for _ in 0..5 {
                tick_ammo(&mut app);
            }
            {
                let p = app.player.as_ref().unwrap();
                assert_eq!(p.max_minions, 2);
                let b = p.buffs.find_buff_index(150).unwrap();
                assert_eq!(p.buffs.buff_time[b], 100, "TimeLeftDoesNotDecrease");
            }
            // 348 (War Table) — the sentry budget, also never-decreasing.
            app.player.as_mut().unwrap().buffs.add_buff(348, 100);
            tick_ammo(&mut app);
            {
                let p = app.player.as_ref().unwrap();
                assert_eq!(p.max_turrets, 2);
                assert_eq!(p.max_minions, 2, "150 still held");
            }
            // Both cleared — the per-tick reset returns the 1/1 base.
            {
                let p = app.player.as_mut().unwrap();
                p.buffs.clear_buff(150);
                p.buffs.clear_buff(348);
            }
            tick_ammo(&mut app);
            let p = app.player.as_ref().unwrap();
            assert_eq!((p.max_minions, p.max_turrets), (1, 1));
        }

        /// DW-2.3 — the armor rows (`Player.cs:13258-13948`) and the held items
        /// (`:15324-15329`) move the budgets by the verbatim deltas; unequipping
        /// returns to the base. 3777 +1, 3381 +1/+1, 3382/3383 +2, the
        /// 1158-1161 range +1, the held 1864/1845 +1, 1167 no delta.
        #[test]
        fn test_DW_2_3_the_armor_and_held_rows_move_the_budgets() {
            let mut app = ammo_app();
            app.equip.armor[0] = slot(3777);
            tick_ammo(&mut app);
            assert_eq!(app.player.as_ref().unwrap().max_minions, 2, "3777: +1");
            app.equip.armor[1] = slot(3381);
            tick_ammo(&mut app);
            let p = app.player.as_ref().unwrap();
            assert_eq!(p.max_minions, 3, "3381: +1");
            assert_eq!(p.max_turrets, 2, "3381: +1 turret");
            app.equip.armor[2] = slot(3382);
            tick_ammo(&mut app);
            assert_eq!(app.player.as_ref().unwrap().max_minions, 5, "3382: +2");
            // The held rows: 1864 in an accessory slot, 1845 in the port's
            // misc-equips host (the `ApplyEquipFunctional` scopes).
            app.equip.armor[3] = slot(1864);
            tick_ammo(&mut app);
            assert_eq!(app.player.as_ref().unwrap().max_minions, 6);
            app.equip.misc_equips[0] = slot(1845);
            tick_ammo(&mut app);
            assert_eq!(app.player.as_ref().unwrap().max_minions, 7);
            // The 1158-1161 range row (1159 also carries the minionDamage half
            // this phase does not port — only the budget delta lands).
            app.equip.armor[4] = slot(1159);
            tick_ammo(&mut app);
            assert_eq!(app.player.as_ref().unwrap().max_minions, 8);
            // A 1167 held item carries NO budget delta (only the minionKB/
            // minionDamage halves — out of scope).
            app.equip.misc_equips[1] = slot(1167);
            tick_ammo(&mut app);
            assert_eq!(app.player.as_ref().unwrap().max_minions, 8);
            // The double row: 3383.
            app.equip.armor[5] = slot(3383);
            tick_ammo(&mut app);
            assert_eq!(app.player.as_ref().unwrap().max_minions, 10);
            // Unequip everything — the per-tick reset returns the base.
            let empty = EquipSlots::empty();
            app.equip.armor = empty.armor;
            app.equip.misc_equips = empty.misc_equips;
            tick_ammo(&mut app);
            let p = app.player.as_ref().unwrap();
            assert_eq!((p.max_minions, p.max_turrets), (1, 1));
        }

        /// Review fix — the 238 row's +1 minion sits behind the C#'s
        /// `Main.tenthAnniversaryWorld` gate (`Player.cs:13256-13259`): a
        /// normal world grants nothing, the anniversary seed flag grants
        /// +1, and dropping the flag returns the base on the next tick. The
        /// port reads the gate from `World.seeds` (the
        /// `SeedFlags::TENTH_ANNIVERSARY` stand-in — no save-header wiring
        /// until M5-SEED-1).
        #[test]
        fn test_edge_the_238_row_is_anniversary_gated() {
            let mut app = ammo_app();
            app.equip.armor[0] = slot(238);
            tick_ammo(&mut app);
            assert_eq!(
                app.player.as_ref().unwrap().max_minions,
                1,
                "a normal world grants nothing for 238"
            );
            // The anniversary flag: the +1 lands.
            app.world.seeds = tr_core::seeds::SeedFlags::TENTH_ANNIVERSARY.into();
            tick_ammo(&mut app);
            assert_eq!(app.player.as_ref().unwrap().max_minions, 2);
            // Back to a normal world — the per-tick reset returns the base.
            app.world.seeds = tr_core::seeds::SeedFlags::none();
            tick_ammo(&mut app);
            assert_eq!(app.player.as_ref().unwrap().max_minions, 1);
        }
    }
}
